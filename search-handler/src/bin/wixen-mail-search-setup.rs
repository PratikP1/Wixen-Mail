//! Put Wixen Mail's message store into the Windows Search index, take it back
//! out, and say which of those is true right now.
//!
//! This program exists so somebody can try the search handler end to end and
//! watch what happens. Until it was written, the handler could be built and
//! tested and there was no way to find out whether Windows would do anything
//! with it.
//!
//! Everything that decides anything is in the library beside this file and is
//! tested there. What is left here is the order things happen in and the words
//! that reach the screen.
//!
//! # Nothing printed here can carry somebody's mail
//!
//! The one rule this program has. It prints the identifier of the account it is
//! working on and its own URL prefix, both of which it built itself. It never
//! prints a URL out of the index, a rule belonging to another application, or
//! anything read from the message store, because all three carry real names of
//! real things on somebody's machine. The two calls that could have are
//! [`wixen_mail_search::scope::describe_url_being_indexed`] and the rule count
//! in [`wixen_mail_search::scope::ScopeState`], and both are written to make
//! that impossible rather than trusted not to.

use std::path::PathBuf;
use std::process::ExitCode;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use wixen_mail_search::com::{account, crawl_scope, exports, library};
use wixen_mail_search::registration::PROTOCOL_PROGID;
use wixen_mail_search::scope::{ScopePlan, plan_for};
use wixen_mail_search::setup::{Action, HELP, Request, parse};

/// It worked.
const DONE: u8 = 0;
/// It did not work.
const FAILED: u8 = 1;
/// The command line was wrong.
const MISTYPED: u8 = 2;

fn main() -> ExitCode {
    let asked = match parse(std::env::args().skip(1)) {
        Ok(asked) => asked,
        Err(wrong) => {
            eprintln!("{wrong}");
            eprintln!();
            eprintln!("{HELP}");
            return ExitCode::from(MISTYPED);
        }
    };

    if asked.action == Action::Help {
        println!("{HELP}");
        return ExitCode::from(DONE);
    }

    // Apartment threaded, which is what a program with a single thread doing one
    // thing at a time wants, and what the search manager is happy to be called
    // on. Uninitialised at the end so the search service sees the connection
    // closed rather than dropped.
    let started = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if started.is_err() {
        eprintln!("could not start COM (code {:#010X})", started.0);
        return ExitCode::from(FAILED);
    }
    let outcome = run(&asked);
    unsafe { CoUninitialize() };

    ExitCode::from(outcome)
}

/// Do the one thing that was asked.
fn run(asked: &Request) -> u8 {
    let Some(user) = whose_mail(asked) else {
        eprintln!(
            "could not work out which account to set this up for. Give one with \
             --user, as a security identifier such as S-1-5-21-...",
        );
        return FAILED;
    };
    let plan = match plan_for(&user) {
        Ok(plan) => plan,
        Err(wrong) => {
            eprintln!("{user}: {wrong}");
            return FAILED;
        }
    };

    println!("Account: {user}");
    println!("URL prefix: {}", plan.prefix);
    println!(
        "Running with administrator rights: {}",
        match account::is_elevated() {
            Some(true) => "yes",
            Some(false) => "no",
            None => "Windows would not say",
        }
    );
    println!();

    match asked.action {
        Action::Status => status(&plan),
        Action::Install => install(asked, &plan),
        Action::Uninstall => uninstall(asked, &plan),
        Action::AddScope => add_scope(&plan),
        Action::RemoveScope => remove_scope(&plan),
        Action::RegisterClasses => register_classes(asked),
        Action::UnregisterClasses => unregister_classes(asked),
        Action::Reindex => reindex(&plan),
        // Answered before COM was started, because it needs none of this.
        Action::Help => DONE,
    }
}

/// Say what is set up now and change nothing.
fn status(plan: &ScopePlan) -> u8 {
    println!("Registry");
    println!(
        "  Library Windows would load: {}",
        exports::registered_library().unwrap_or_else(|| "nothing registered".to_string())
    );
    println!(
        "  Handler registered for the wixen-mail scheme: {}",
        match exports::registered_scheme_handler() {
            Some(name) if name == PROTOCOL_PROGID => "yes".to_string(),
            Some(name) => format!("yes, but as {name} rather than {PROTOCOL_PROGID}"),
            None => "no".to_string(),
        }
    );
    println!();

    println!("Windows Search");
    match crawl_scope::read_state(plan) {
        Ok(state) => println!("{}", indented(&state.to_string())),
        Err(wrong) => {
            eprintln!("  {wrong}");
            return FAILED;
        }
    }
    println!();

    println!("The indexer");
    match crawl_scope::report() {
        Ok(report) => {
            println!("  Doing: {}", report.status);
            if let Some(because) = report.paused_because {
                println!("  Paused because: {because}");
            }
            println!("  {}", report.busy_with);
            println!(
                "  Items in the whole index, everything on this computer: {}",
                report.items_in_the_whole_index
            );
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            return FAILED;
        }
    }
    println!();
    println!(
        "To find out whether any mail has really been indexed, see the \"Did it \
         find anything\" section of the README. This tool cannot answer that: \
         there is no supported way to ask the catalog how many items came from \
         one place."
    );

    DONE
}

/// Register the classes, then tell the indexer to look.
///
/// In that order, and it matters. A crawl scope rule pointing at a scheme with
/// no handler is a rule the indexer will act on and fail at, once per item, with
/// nothing to say why.
fn install(asked: &Request, plan: &ScopePlan) -> u8 {
    match register_classes(asked) {
        DONE => add_scope(plan),
        failed => failed,
    }
}

/// Undo all of it.
///
/// The other way round from installing, for the same reason: the window in which
/// the two disagree should be the harmless one, where a handler is registered
/// and nothing asks it anything.
fn uninstall(asked: &Request, plan: &ScopePlan) -> u8 {
    let scope = remove_scope(plan);
    // Kept going even when the rule could not be removed. Stopping here would
    // leave a machine that is registered and out of scope, which is a state
    // nothing reports and nothing fixes.
    let classes = unregister_classes(asked);

    match scope == DONE && classes == DONE {
        true => DONE,
        false => FAILED,
    }
}

/// Write the registry entries that make Windows Search aware of the handler.
///
/// Its own command because the installer needs this half on its own. It runs
/// elevated, where [`add_scope`] has to run as the person who started setup.
fn register_classes(asked: &Request) -> u8 {
    let Some(path) = where_the_library_is(asked) else {
        eprintln!("could not work out where the library is. Give one with --library.");
        return FAILED;
    };

    println!("Registering {}", path.display());
    match library::register(&path) {
        Ok(()) => {
            println!("  The classes and the wixen-mail scheme are registered.");
            DONE
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            FAILED
        }
    }
}

/// Take those registry entries back out.
fn unregister_classes(asked: &Request) -> u8 {
    let Some(path) = where_the_library_is(asked) else {
        eprintln!("could not work out where the library is. Give one with --library.");
        return FAILED;
    };

    println!("Unregistering {}", path.display());
    match library::unregister(&path) {
        Ok(()) => {
            println!("  The registry entries are gone.");
            println!(
                "  Anything already in the Windows Search index stays there until the \
                 index is rebuilt. Indexing Options, Advanced, Rebuild does that, and \
                 it takes hours."
            );
            DONE
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            FAILED
        }
    }
}

/// Tell the indexer to look.
fn add_scope(plan: &ScopePlan) -> u8 {
    println!("Adding the crawl scope rule");
    match crawl_scope::add(plan) {
        Ok(()) => {
            println!("  Done.");
            println!();
            println!(
                "Everything the indexer now takes from this mail store lands in the \
                 Windows Search index, which is not encrypted and can be read by any \
                 software running on this computer."
            );
            println!(
                "The indexer decides for itself when to start. It can be minutes, and \
                 longer if the computer is busy or on battery. Run status to watch."
            );
            DONE
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            FAILED
        }
    }
}

/// Tell it to stop.
fn remove_scope(plan: &ScopePlan) -> u8 {
    println!("Taking the crawl scope rule out");
    match crawl_scope::remove(plan) {
        Ok(()) => {
            println!("  Done. The indexer will not ask about this mail again.");
            println!("  What is already in the index stays there until the index is rebuilt.");
            DONE
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            FAILED
        }
    }
}

/// Ask the indexer to visit this handler's URLs again.
fn reindex(plan: &ScopePlan) -> u8 {
    println!("Asking the indexer to look at these URLs again");
    match crawl_scope::reindex(plan) {
        Ok(()) => {
            println!("  Asked. This affects only this handler's URLs, not the rest of the index.");
            DONE
        }
        Err(wrong) => {
            eprintln!("  {wrong}");
            FAILED
        }
    }
}

/// Whose mail this run is about.
fn whose_mail(asked: &Request) -> Option<String> {
    asked.user.clone().or_else(account::current_user_sid)
}

/// Where the handler library is for this run.
fn where_the_library_is(asked: &Request) -> Option<PathBuf> {
    asked.library.clone().or_else(library::beside_this_program)
}

/// Put a block of lines under a heading.
fn indented(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

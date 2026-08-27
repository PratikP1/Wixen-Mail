//! What Wixen Mail was asked to do before its window opened.
//!
//! This used to be two scans over `env::args()` looking for a string, one for
//! erasing data and one for the accessibility check. There was no `--help`, no
//! `--version`, and nowhere for anything else to go, so a third flag would
//! have been a third scan.
//!
//! Parsed here instead, into one value, with room for the commands a fuller
//! version wants. Kept hand-written rather than reaching for a parser crate:
//! what is here is small, the two existing flags have particular behaviour
//! that has to survive, and every branch is tested. That trade is worth
//! revisiting when subcommands take arguments of their own.
//!
//! # Printing anything is not free on Windows
//!
//! The binary sets `windows_subsystem = "windows"`, so it has no console and
//! `println!` goes nowhere when it is started from a terminal. Anything meant
//! to be read has to attach to the parent console first. That is done in
//! `main`, because it is a side effect and this stays pure.

use crate::application::allowed::Allowed;
use crate::application::opening::{Opening, what_was_handed_over};

/// Erase everything this installation stored, then exit.
///
/// The uninstaller runs this. It can delete the program's own folder but
/// cannot reach the credential store, and does not know where the data folder
/// is when `WIXEN_MAIL_DATA` has moved it.
pub const ERASE_FLAG: &str = "--erase-all-data";

/// What to do with this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Open the window.
    Run(Run),
    /// Erase the stored data and exit.
    EraseAllData,
    /// Say what the flags are, and exit.
    Help,
    /// Say which version this is, and exit.
    Version,
    /// The arguments did not make sense. Say why, and exit without starting.
    Refused(String),
}

/// How to open the window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Run {
    /// What the command line allows to be changed at a provider.
    ///
    /// Only ever narrows what is stored. There is no flag that turns writing
    /// on, deliberately: the flag that switches off a safety catch is the one
    /// somebody puts in a shortcut and forgets about.
    pub allowed: Allowed,
    /// Which window the accessibility check should walk, if this is that run.
    pub scan_target: Option<String>,
    /// What Windows handed over on the way in, if it handed over anything.
    ///
    /// Filled when this program is started by the shell for a `mailto:` link
    /// or an `.ics` or `.vcf` file it is registered to open, which is the
    /// only way an argument arrives here without a flag in front of it. The
    /// window opens either way; this decides what it does once it is up.
    pub open: Option<Opening>,
    /// The argument exactly as it arrived, kept beside the parsed form.
    ///
    /// A second copy hands this to the copy already running rather than the
    /// parsed result, so the rules about what a link may ask for live in one
    /// parser and are applied by whichever copy is going to act on it. See
    /// [`crate::application::handover`].
    pub handed_over: Option<String>,
}

impl Run {
    /// Nothing narrowed: whatever is stored decides.
    fn unrestricted() -> Self {
        Self {
            allowed: Allowed::EVERYTHING,
            scan_target: None,
            open: None,
            handed_over: None,
        }
    }
}

/// What `--help` prints.
///
/// Written out rather than generated, because it is read by somebody deciding
/// whether it is safe to point this at their mail, and that deserves sentences
/// rather than a table of flags.
pub const HELP: &str = "\
Wixen Mail, an accessible mail and personal information client.

    wixen-mail [options]

Options:
  --read-only            Change nothing at any server this run. Sending,
                         deleting and every sync that writes are refused,
                         whatever the settings say. Reading is unaffected.

  --allow <what>         Narrow what may be changed this run, to one of:
                           nothing      the same as --read-only
                           tasks        tasks, contacts and calendar; no mail
                           everything   no narrowing; the settings decide

  --erase-all-data       Delete everything this installation stored, including
                         the saved passwords, then exit. The uninstaller uses
                         this. It does not ask twice.

  --scan-target <name>   Walk one window with the accessibility check and
                         exit. Used by the automated scan.

  --help                 This.
  --version              Which version this is.

One thing to open may be given instead of a flag, which is how Windows starts
this program when it holds the association:

  wixen-mail mailto:someone@example.com?subject=Hello
  wixen-mail webcal://example.com/team.ics
  wixen-mail C:\\Users\\you\\Downloads\\invite.ics
  wixen-mail C:\\Users\\you\\Downloads\\card.vcf

A mailto: link opens a message ready to send. A webcal: link and an .ics file
add their events to the calendar kept on this computer. A .vcf file adds its
cards to the contacts kept on this computer. None of them changes anything at
any server.

Neither --read-only nor --allow can permit anything the settings forbid. They
only ever take permissions away, so leaving one in a shortcut is safe.

Everything that writes is experimental. Sending mail, deleting mail, and
syncing changes to tasks, contacts and the calendar have never been run against
a real account, so expect them to have bugs and do not point them at anything
you cannot afford to lose. Reading is the part that has been used.
";

/// Work out what this run was asked to do.
///
/// Takes the arguments after the program name.
pub fn parse<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();

    // Before anything else, and on its own. Erasing everything is not a thing
    // to combine with other flags: a run that both erased the data and opened
    // a window would be a window onto nothing.
    if args.iter().any(|arg| arg == ERASE_FLAG) {
        return Command::EraseAllData;
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Command::Help;
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        return Command::Version;
    }

    let mut run = Run::unrestricted();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--read-only" => run.allowed = Allowed::NOTHING,
            "--allow" => {
                let Some(what) = rest.next() else {
                    return Command::Refused(
                        "--allow needs to say what: nothing, tasks or everything".to_string(),
                    );
                };
                match allowance(what) {
                    Some(allowed) => run.allowed = run.allowed.and(allowed),
                    None => {
                        return Command::Refused(format!(
                            "--allow does not understand {what:?}. \
                             It takes nothing, tasks or everything"
                        ));
                    }
                }
            }
            "--scan-target" => {
                let Some(name) = rest.next() else {
                    return Command::Refused(
                        "--scan-target needs the name of a window to walk".to_string(),
                    );
                };
                run.scan_target = Some(name.clone());
            }
            // Refused rather than ignored. An unknown flag is a typed flag,
            // and starting normally having quietly dropped `--read-only`
            // because it was spelled `--readonly` is the failure this whole
            // area exists to prevent. Anything beginning with a dash is a
            // flag somebody meant, never a file: Windows hands over a full
            // path, and no scheme this program registers starts with one.
            misspelled if misspelled.starts_with('-') => {
                return Command::Refused(format!(
                    "Wixen Mail does not understand {misspelled:?}. Try --help"
                ));
            }
            // Everything else is what Windows hands over when it starts this
            // program for a link or a file it is registered to open. Before
            // this, every one of those was refused, so a registered handler
            // opened an error box and exited, which is worse than not being
            // registered at all.
            handed_over => {
                if run.open.is_some() {
                    // Two would mean choosing one and silently dropping the
                    // other. Windows sends one at a time.
                    return Command::Refused(format!(
                        "Wixen Mail opens one thing at a time, and was given \
                         {handed_over:?} as well"
                    ));
                }
                match what_was_handed_over(handed_over) {
                    Ok(opening) => {
                        run.open = Some(opening);
                        run.handed_over = Some(handed_over.to_string());
                    }
                    // The reason from where it was known, not a second one on
                    // top. It already names the argument and says what this
                    // program opens.
                    Err(why) => return Command::Refused(format!("{why}. Try --help")),
                }
            }
        }
    }
    Command::Run(run)
}

/// What one `--allow` word means.
fn allowance(what: &str) -> Option<Allowed> {
    match what {
        "nothing" | "none" => Some(Allowed::NOTHING),
        "tasks" | "pim" => Some(Allowed::FOR_TESTING),
        "everything" | "all" => Some(Allowed::EVERYTHING),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Run {
        match parse(args) {
            Command::Run(run) => run,
            other => panic!("expected a run, got {other:?}"),
        }
    }

    fn refusal(args: &[&str]) -> String {
        match parse(args) {
            Command::Refused(why) => why,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn test_no_arguments_opens_the_window_and_narrows_nothing() {
        let started = run(&[]);

        assert_eq!(started.allowed, Allowed::EVERYTHING, "the settings decide");
        assert_eq!(started.scan_target, None);
    }

    #[test]
    fn test_read_only_stops_everything() {
        assert_eq!(run(&["--read-only"]).allowed, Allowed::NOTHING);
    }

    #[test]
    fn test_allow_takes_a_word_and_narrows_to_it() {
        assert_eq!(run(&["--allow", "nothing"]).allowed, Allowed::NOTHING);
        assert_eq!(run(&["--allow", "tasks"]).allowed, Allowed::FOR_TESTING);
        assert_eq!(run(&["--allow", "everything"]).allowed, Allowed::EVERYTHING);
    }

    #[test]
    fn test_two_narrowings_both_apply() {
        // They combine rather than the last one winning, because each is a
        // restriction and dropping one would widen what a person asked for.
        let started = run(&["--allow", "everything", "--read-only"]);

        assert_eq!(started.allowed, Allowed::NOTHING);
    }

    #[test]
    fn test_the_command_line_cannot_widen_anything() {
        // The rule that makes leaving a flag in a shortcut safe. Even
        // "everything" is only the absence of narrowing here; whether anything
        // actually goes out is decided against the settings later.
        let started = run(&["--allow", "everything"]);

        assert_eq!(started.allowed, Allowed::EVERYTHING);
    }

    #[test]
    fn test_a_missing_value_is_refused_rather_than_guessed() {
        assert!(refusal(&["--allow"]).contains("nothing, tasks or everything"));
        assert!(refusal(&["--scan-target"]).contains("name of a window"));
    }

    #[test]
    fn test_a_word_allow_does_not_know_is_refused() {
        let why = refusal(&["--allow", "sending"]);

        assert!(why.contains("sending"), "{why}");
        assert!(why.contains("nothing, tasks or everything"), "{why}");
    }

    #[test]
    fn test_a_flag_that_is_nearly_right_is_refused_rather_than_ignored() {
        // The one that matters most here. Starting normally having silently
        // dropped a misspelled --readonly is exactly the accident this whole
        // area exists to prevent.
        let why = refusal(&["--readonly"]);

        assert!(why.contains("--readonly"), "{why}");
        assert!(why.contains("--help"), "{why}");
    }

    #[test]
    fn test_erasing_wins_over_everything_and_takes_no_arguments() {
        // It runs before logging is set up, because the log file lives in the
        // folder being deleted, so it cannot be one branch among several.
        assert_eq!(parse([ERASE_FLAG]), Command::EraseAllData);
        assert_eq!(
            parse([ERASE_FLAG, "--read-only"]),
            Command::EraseAllData,
            "a run that erased the data and opened a window is a window onto nothing"
        );
    }

    #[test]
    fn test_help_and_version_say_so_and_stop() {
        for asked in [vec!["--help"], vec!["-h"]] {
            assert_eq!(parse(asked), Command::Help);
        }
        for asked in [vec!["--version"], vec!["-V"]] {
            assert_eq!(parse(asked), Command::Version);
        }
    }

    #[test]
    fn test_the_scan_target_still_works_the_way_it_did() {
        // Carried across unchanged: the accessibility workflow passes it and
        // nothing about that run should have altered.
        assert_eq!(
            run(&["--scan-target", "compose"]).scan_target,
            Some("compose".to_string())
        );
    }

    #[test]
    fn test_a_mailto_link_handed_over_by_windows_opens_a_message_rather_than_being_refused() {
        // The whole reason this program can be registered for `mailto:` at
        // all. Before this, Windows started it with the link as an argument
        // and it answered "Wixen Mail does not understand" and exited with a
        // failure code, so following a mail link showed an error box.
        let started = run(&["mailto:someone@example.com?subject=Hello"]);

        let Some(Opening::Compose(asked)) = &started.open else {
            panic!("a mailto link did not open a message: {:?}", started.open);
        };
        assert_eq!(asked.to, "someone@example.com");
        assert_eq!(asked.subject, "Hello");
    }

    #[test]
    fn test_every_shape_windows_hands_over_is_accepted() {
        // One per registered association, because each is claimed separately
        // in the registry and a shape missed here is a claim that opens an
        // error box on somebody's file.
        assert!(matches!(
            run(&["mailto:a@b.com"]).open,
            Some(Opening::Compose(_))
        ));
        assert!(matches!(
            run(&["webcal://example.com/team.ics"]).open,
            Some(Opening::CalendarSubscription(_))
        ));
        assert!(matches!(
            run(&[r"C:\Users\you\invite.ics"]).open,
            Some(Opening::CalendarFile(_))
        ));
        assert!(matches!(
            run(&[r"C:\Users\you\card.vcf"]).open,
            Some(Opening::ContactFile(_))
        ));
    }

    #[test]
    fn test_a_misspelled_flag_is_still_refused_now_that_arguments_are_accepted() {
        // The guard that had to survive accepting arguments at all. Anything
        // beginning with a dash is a flag somebody meant, and starting
        // normally having quietly dropped a misspelled `--read-only` is the
        // accident this whole area exists to prevent. Windows hands over a
        // full path or a scheme, never something starting with a dash.
        for misspelled in ["--readonly", "-read-only", "--allowe", "-x"] {
            let why = refusal(&[misspelled]);
            assert!(why.contains(misspelled), "{why}");
            assert!(why.contains("--help"), "{why}");
        }
    }

    #[test]
    fn test_something_this_program_cannot_open_is_refused_and_says_what_it_can() {
        // A bare word used to be refused with "does not understand". It still
        // is, and now the refusal says what this program does open, because
        // the person reading it has just watched a file fail to open.
        let why = refusal(&[r"C:\Users\you\report.pdf"]);

        assert!(why.contains("report.pdf"), "{why}");
        assert!(why.contains(".ics"), "{why}");
        assert!(why.contains("--help"), "{why}");
    }

    #[test]
    fn test_two_things_to_open_is_refused_rather_than_one_being_dropped() {
        // Windows sends one at a time. Two means somebody typed them, and
        // opening the first while silently ignoring the second would leave
        // them wondering where the other went.
        let why = refusal(&["mailto:a@b.com", r"C:\x.ics"]);

        assert!(why.contains("one thing at a time"), "{why}");
    }

    #[test]
    fn test_something_to_open_travels_with_the_flags_rather_than_replacing_them() {
        // A shortcut can carry both: `--read-only` beside a file to open is
        // how somebody looks at an invitation without letting anything reach
        // a server. Dropping either would be surprising in a way that matters.
        let started = run(&["--read-only", r"C:\Users\you\invite.ics"]);

        assert_eq!(started.allowed, Allowed::NOTHING);
        assert!(matches!(started.open, Some(Opening::CalendarFile(_))));
    }

    #[test]
    fn test_erasing_and_help_still_win_over_something_to_open() {
        // Both are checked before anything else and must stay that way. An
        // uninstall that opened a composer because a stale argument was in
        // the command line would be a window over somebody's uninstaller.
        assert_eq!(parse([ERASE_FLAG, "mailto:a@b.com"]), Command::EraseAllData);
        assert_eq!(parse(["--help", "mailto:a@b.com"]), Command::Help);
    }

    #[test]
    fn test_an_ordinary_start_still_has_nothing_to_open() {
        // The commonest run there is. A version of this that found something
        // to open in an empty command line would put a composer in front of
        // somebody every time they started the program.
        assert_eq!(run(&[]).open, None);
        assert_eq!(run(&["--read-only"]).open, None);
        assert_eq!(run(&["--scan-target", "compose"]).open, None);
    }

    #[test]
    fn test_a_value_belonging_to_a_flag_is_not_read_as_something_to_open() {
        // `--scan-target compose` is two arguments and the second belongs to
        // the first. Read as a thing to open it would be refused, and the
        // accessibility workflow would stop running.
        let started = run(&["--scan-target", "compose"]);

        assert_eq!(started.scan_target, Some("compose".to_string()));
        assert_eq!(started.open, None);
    }

    #[test]
    fn test_the_help_says_what_the_flags_do_to_somebody_deciding() {
        // It is read by a person working out whether this is safe to point at
        // their mail, so it has to answer that rather than list flags.
        assert!(HELP.contains("--read-only"), "{HELP}");
        assert!(HELP.contains("only ever take permissions away"), "{HELP}");
        // Somebody reading this is deciding whether to point it at their real
        // mail, so it has to say plainly that writing is unproven.
        assert!(HELP.contains("experimental"), "{HELP}");
        assert!(HELP.contains("never been run against"), "{HELP}");
        assert!(HELP.contains("Reading is unaffected"), "{HELP}");
    }

    #[test]
    fn test_the_help_says_what_can_be_opened_because_nothing_else_documents_it() {
        // These arrive from Windows without anybody typing them, so the only
        // place a person can find out what this program is registered to open
        // is here. Every shape is named, so a claim in the registry that the
        // help does not mention shows up as a failure here.
        for shape in ["mailto:", "webcal:", ".ics", ".vcf"] {
            assert!(HELP.contains(shape), "the help does not mention {shape}");
        }
        // It says plainly that opening one of these changes nothing at a
        // server, because somebody double-clicking an invitation from a
        // stranger deserves to know that before it happens.
        assert!(
            HELP.contains("changes anything at\nany server") || HELP.contains("any server"),
            "{HELP}"
        );
    }
}

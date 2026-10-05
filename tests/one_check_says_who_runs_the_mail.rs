//! Every place that decides whether an account is Gmail or Microsoft asks the
//! one check, `application::who_runs_the_mail::WhoRunsTheMail` (GAP-06, 13-44.5).
//!
//! Four places used to decide it for themselves and gave four answers, so a
//! Google Workspace or Microsoft 365 account on its own domain was Gmail to one
//! and nobody to the next. Two readings keep it one place.
//!
//! # What it reads
//!
//! The half of every `.rs` file under `src` a release build compiles
//! (`common::what_ships`). The first reading takes each caller below by its
//! signature, from there to the brace that closes it at the signature's own
//! indentation, and asks that it names the check, or the function that asks
//! it. The second drops comment lines and every space from each file and asks
//! that the shapes of deciding for oneself appear only where `THE_SHAPES`
//! allows them. A companion hands both readings planted text with each fault
//! in it, so a reading that stopped finding its anchor cannot pass by finding
//! nothing.
//!
//! # Why it lives here
//!
//! The callers are in several files, two of them named by more than a hundred
//! guard records, and a census over the whole of `src` belongs to none of them.
//! Its records carry `suite = "one_check_says_who_runs_the_mail"`, so a commit
//! touching the code one of them breaks runs it.
//!
//! # What it cannot see
//!
//! It runs at a commit touching a file one of its records names, at the
//! phase's full gate and in CI's Test Suite, so a new reading added alone in
//! another file waits for one of those. It reads text: a reading spelled in a
//! shape `THE_SHAPES` does not list, such as a match arm on `"gmail"` or a
//! comparison with `Some("Gmail")`, passes. And whether the answer is right is
//! the check's own cases' to say, not this file's.

use wixen_mail::common::what_ships::what_ships;

/// The check itself, the one file every shape is allowed in.
const THE_CHECK: &str = "src/application/who_runs_the_mail.rs";

/// One place that decides, and what it must name.
struct Caller {
    file: &'static str,
    /// The function's signature, or `None` where the whole file is read.
    signature: Option<&'static str>,
    names: &'static str,
}

/// Every place that treats Gmail or Microsoft differently, by what it asks.
/// A plan that adds a caller adds a line here.
const THE_CALLERS: &[Caller] = &[
    Caller {
        file: "src/application/reporting_junk.rs",
        signature: Some("pub fn of("),
        names: "WhoRunsTheMail::of(",
    },
    Caller {
        file: "src/application/mail_auth.rs",
        signature: Some("pub fn provider_of("),
        names: "WhoRunsTheMail::of(",
    },
    Caller {
        file: "src/presentation/wx_account_manager.rs",
        signature: Some("fn run_oauth_flow("),
        names: "provider_of(",
    },
    Caller {
        file: "src/presentation/wx_app.rs",
        signature: Some("fn choose_folders("),
        names: "WhoRunsTheMail::of(",
    },
    Caller {
        file: "src/presentation/wx_app.rs",
        signature: Some("fn report_the_chosen_as_junk("),
        names: "AccountKind::of(",
    },
    // The account editor's app password advice and its button, by Pratik's
    // answer of 2026-09-30, and the feedback report's account kinds.
    Caller {
        file: "src/presentation/wx_account_manager.rs",
        signature: Some("fn password_box_description("),
        names: "WhoRunsTheMail::from_what_is_known(",
    },
    Caller {
        file: "src/presentation/wx_account_manager.rs",
        signature: Some("fn where_to_get_an_app_password("),
        names: "WhoRunsTheMail::from_what_is_known(",
    },
    Caller {
        file: "src/service/this_machine.rs",
        signature: Some("pub fn providers("),
        names: "WhoRunsTheMail::of(",
    },
    // Who empties an account's Trash (13-44.6, D12): the decision maps the
    // check's answer, the mail check asks it of the account, and the account
    // editor of what is typed.
    Caller {
        file: "src/application/emptying_the_trash.rs",
        signature: Some("pub fn who_empties_the_trash("),
        names: "WhoRunsTheMail::",
    },
    Caller {
        file: "src/presentation/wx_account_manager.rs",
        signature: Some("fn show_who_empties_the_trash("),
        names: "WhoRunsTheMail::from_what_is_known(",
    },
    Caller {
        file: "src/presentation/wx_app.rs",
        signature: Some("fn spawn_mail_sync("),
        names: "WhoRunsTheMail::of(",
    },
    // And the close asks it of each account set to empty as Wixen Mail
    // closes (13-44.7).
    Caller {
        file: "src/presentation/wx_app.rs",
        signature: Some("fn emptying_the_trash_on_the_way_out("),
        names: "WhoRunsTheMail::of(",
    },
    // The three followers name the function rather than call it in one
    // shape: the notes backend hands it to `and_then`.
    Caller {
        file: "src/application/new_item.rs",
        signature: None,
        names: "mail_auth::provider_of",
    },
    Caller {
        file: "src/application/notes_backend.rs",
        signature: None,
        names: "mail_auth::provider_of",
    },
    Caller {
        file: "src/application/looking_people_up.rs",
        signature: None,
        names: "mail_auth::provider_of",
    },
    // Whether an account's calendars, contacts and tasks are Google's to ask
    // (14-01, #22): until then each sync asked Google whenever a key was
    // present, whoever ran the account's mail.
    Caller {
        file: "src/application/who_holds_the_calendars.rs",
        signature: Some("pub fn may_google_be_asked<"),
        names: "WhoRunsTheMail::of(",
    },
    // And the one Google token answer, which chooses between an account's
    // mail sign-in and its separate one for calendars, contacts and tasks
    // (14-03, route B), asks that answer before either.
    Caller {
        file: "src/application/who_holds_the_calendars.rs",
        signature: Some("pub async fn a_google_token_with("),
        names: "may_google_be_asked(",
    },
];

/// A way of deciding for oneself, as it reads with every space removed, and
/// the files it may appear in with the reason for each.
struct Shape {
    shape: &'static str,
    allowed: &'static [(&'static str, &'static str)],
}

const THE_SHAPES: &[Shape] = &[
    Shape {
        shape: "OAuthService::detect_provider(",
        allowed: &[(THE_CHECK, "the address, read second")],
    },
    Shape {
        shape: ".provider.as_deref()",
        allowed: &[
            (THE_CHECK, "the name an account was saved with, read last"),
            (
                "src/service/this_machine.rs",
                "the feedback report names an account somebody else runs by the name it \
                 was saved with, after asking the check",
            ),
        ],
    },
    Shape {
        shape: "eq_ignore_ascii_case(\"imap.gmail.com\")",
        allowed: &[],
    },
    Shape {
        shape: "eq_ignore_ascii_case(\"outlook.office365.com\")",
        allowed: &[],
    },
    Shape {
        shape: "eq_ignore_ascii_case(\"gmail\")",
        allowed: &[
            (THE_CHECK, "the saved name compared"),
            (
                "src/service/oauth.rs",
                "the sign-in compares provider names when it builds a sign-in address and \
                 refreshes a token, after the check chose the name",
            ),
        ],
    },
    Shape {
        shape: "eq_ignore_ascii_case(\"outlook\")",
        allowed: &[
            (THE_CHECK, "the saved name compared"),
            (
                "src/service/oauth.rs",
                "the sign-in compares provider names, after the check chose the name",
            ),
        ],
    },
];

/// Every Rust file under a folder: its path with forward slashes, and the
/// half a release build compiles.
fn every_source_file_under(folder: &std::path::Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(folder).unwrap_or_else(|why| panic!("{folder:?}: {why}"));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_source_file_under(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text =
                std::fs::read_to_string(&path).unwrap_or_else(|why| panic!("{path:?}: {why}"));
            found.push((
                path.display().to_string().replace('\\', "/"),
                what_ships(&text.replace("\r\n", "\n")),
            ));
        }
    }
    found
}

/// One function's text, from its signature to the brace that closes it at
/// the signature's own indentation.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source
        .find(signature)
        .ok_or(format!("{signature} is not here, so this reads nothing"))?;
    let line_start = source[..at].rfind('\n').map_or(0, |newline| newline + 1);
    let indent: String = source[line_start..at]
        .chars()
        .take_while(|letter| *letter == ' ')
        .collect();
    let rest = &source[at..];
    let closing = format!("\n{indent}}}\n");
    let ends = rest
        .find(&closing)
        .map_or(rest.len(), |end| end + closing.len());
    Ok(rest[..ends].to_string())
}

/// Whether a caller in this file names what it must.
fn asks_the_check(path: &str, source: &str, caller: &Caller) -> Result<(), String> {
    let read = match caller.signature {
        Some(signature) => body_of(source, signature).map_err(|why| format!("{path}: {why}"))?,
        None => source.to_string(),
    };
    match read.contains(caller.names) {
        true => Ok(()),
        false => Err(format!(
            "{path}: {} decides Gmail or Microsoft without naming {}",
            caller.signature.unwrap_or("the file"),
            caller.names
        )),
    }
}

/// A file's text with comment lines dropped and every space removed.
fn compacted(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|letter| !letter.is_whitespace())
        .collect()
}

/// Whether a shape of deciding for oneself appears in this file only where
/// it is allowed.
fn decides_only_where_allowed(path: &str, source: &str, shape: &Shape) -> Result<(), String> {
    let allowed = shape.allowed.iter().any(|(file, _)| *file == path);
    match allowed || !compacted(source).contains(shape.shape) {
        true => Ok(()),
        false => Err(format!(
            "{path} decides Gmail or Microsoft for itself with {}; ask \
             application::who_runs_the_mail::WhoRunsTheMail instead",
            shape.shape
        )),
    }
}

/// Every complaint both readings have over these files.
fn complaints_over(files: &[(String, String)], callers: &[Caller]) -> Vec<String> {
    let mut complaints = Vec::new();
    for caller in callers {
        match files.iter().find(|(path, _)| path == caller.file) {
            Some((path, source)) => complaints.extend(asks_the_check(path, source, caller).err()),
            None => complaints.push(format!(
                "{} is not here, so this reads nothing",
                caller.file
            )),
        }
    }
    for (path, source) in files {
        for shape in THE_SHAPES {
            complaints.extend(decides_only_where_allowed(path, source, shape).err());
        }
    }
    complaints
}

/// The caller with this signature, for a companion to plant a fault into.
fn the_caller(signature: &str) -> &'static Caller {
    THE_CALLERS
        .iter()
        .find(|caller| caller.signature == Some(signature))
        .unwrap_or_else(|| panic!("no caller {signature}"))
}

/// One planted file, standing where `path` would.
fn planted(path: &str, text: &str) -> Vec<(String, String)> {
    vec![(path.to_string(), text.to_string())]
}

#[test]
fn test_every_place_that_decides_gmail_or_microsoft_asks_the_one_check() {
    let files = every_source_file_under(std::path::Path::new("src"));
    assert!(
        files.iter().any(|(path, _)| path == THE_CHECK),
        "the walk did not find {THE_CHECK}, so it reads nothing"
    );

    let complaints = complaints_over(&files, THE_CALLERS);

    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

#[test]
fn test_the_reading_refuses_a_place_that_decides_for_itself() {
    let alone = |caller: &Caller| {
        [Caller {
            file: caller.file,
            signature: caller.signature,
            names: caller.names,
        }]
    };

    let faults = [
        (
            "a file outside the allowed ones reading the saved name",
            complaints_over(
                &planted(
                    "src/application/somewhere.rs",
                    "pub fn is_gmail(account: &Account) -> bool {\n    \
                     account.provider.as_deref() == Some(\"Gmail\")\n}\n",
                ),
                &[],
            ),
            ".provider.as_deref()",
        ),
        (
            "a browser sign-in asking the address alone",
            complaints_over(
                &planted(
                    "src/presentation/wx_account_manager.rs",
                    "fn run_oauth_flow(account: &mut Account) -> OAuthFlowResult {\n    \
                     let provider = match crate::service::oauth::OAuthService::detect_provider(\
                     &account.email) {\n        Some(p) => p,\n        None => return \
                     OAuthFlowResult::Failed(String::new()),\n    };\n}\n",
                ),
                &alone(the_caller("fn run_oauth_flow(")),
            ),
            "fn run_oauth_flow( decides Gmail or Microsoft without naming provider_of(",
        ),
        (
            "a folder chooser comparing with Gmail's server",
            complaints_over(
                &planted(
                    "src/presentation/wx_app.rs",
                    "fn choose_folders(app: AppHandles<'_>) {\n    let is_gmail = \
                     account.imap_server.eq_ignore_ascii_case(\"imap.gmail.com\");\n}\n",
                ),
                &alone(the_caller("fn choose_folders(")),
            ),
            "eq_ignore_ascii_case(\"imap.gmail.com\")",
        ),
        (
            "a mail check's provider that does not name the check",
            complaints_over(
                &planted(
                    "src/application/mail_auth.rs",
                    "pub fn provider_of(account: &Account) -> Option<String> {\n    \
                     account.provider.clone()\n}\n",
                ),
                &alone(the_caller("pub fn provider_of(")),
            ),
            "pub fn provider_of( decides Gmail or Microsoft without naming WhoRunsTheMail::of(",
        ),
        (
            "a feedback report naming accounts by the saved name alone",
            complaints_over(
                &planted(
                    "src/service/this_machine.rs",
                    "pub fn providers(accounts: &[Account]) -> Vec<String> {\n    \
                     accounts.iter().filter_map(|account| account.provider.clone()).collect()\n}\n",
                ),
                &alone(the_caller("pub fn providers(")),
            ),
            "pub fn providers( decides Gmail or Microsoft without naming WhoRunsTheMail::of(",
        ),
        (
            "a file comparing a provider with outlook",
            complaints_over(
                &planted(
                    "src/application/somewhere.rs",
                    "fn is_microsoft(name: &str) -> bool {\n    \
                     name.eq_ignore_ascii_case(\"outlook\")\n}\n",
                ),
                &[],
            ),
            "eq_ignore_ascii_case(\"outlook\")",
        ),
    ];

    for (fault, complaints, expected) in faults {
        assert!(
            complaints.iter().any(|said| said.contains(expected)),
            "{fault} was not refused for {expected}: {complaints:?}"
        );
    }
}

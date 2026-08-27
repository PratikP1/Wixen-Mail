//! What the setup tool was asked to do.
//!
//! Split out from the tool itself so the part that decides can be tested. The
//! tool is the one place a person touches this crate directly, and a command
//! line that quietly does the wrong thing is worse here than almost anywhere
//! else: two of these commands change what a system service crawls for
//! everybody on the machine.

use std::path::PathBuf;

/// Everything the tool can be asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Say what is set up now, changing nothing.
    Status,
    /// Register the classes, then tell the indexer to look.
    Install,
    /// Tell the indexer to stop looking, then unregister the classes.
    Uninstall,
    /// Only the half that tells the indexer to look.
    AddScope,
    /// Only the half that tells it to stop.
    RemoveScope,
    /// Only the half that writes the registry entries.
    ///
    /// Split out for the installer, which cannot use `install`. The two halves
    /// want different accounts: the registry entries go under
    /// `HKEY_LOCAL_MACHINE` and need the elevated installer, while the crawl
    /// scope rule names one person's mail and has to be added as the person who
    /// started setup, not as whichever administrator they typed at the prompt.
    RegisterClasses,
    /// Only the half that takes the registry entries out.
    UnregisterClasses,
    /// Ask the indexer to visit this handler's URLs again.
    Reindex,
    /// Print what the tool can do.
    Help,
}

/// One run of the tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub action: Action,
    /// Whose mail this is about, as a security identifier.
    ///
    /// `None` means the account running the tool. That is not always the right
    /// one: entering another person's administrator details at the elevation
    /// prompt runs this as them, and the rule would then name their mail rather
    /// than the mail of the person signed in.
    pub user: Option<String>,
    /// Where the handler library is.
    ///
    /// `None` means beside the tool, which is where an install puts both.
    pub library: Option<PathBuf>,
}

/// Why a command line could not be understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageError {
    /// A word that is not one of the commands.
    NotACommand(String),
    /// An option that needs a value and did not get one.
    MissingValue(&'static str),
    /// An option that is not one of ours.
    NotAnOption(String),
    /// Two commands in one run.
    ///
    /// Refused rather than resolved. `install uninstall` could reasonably mean
    /// either, and guessing means somebody who typed both watches the machine
    /// end up in a state they did not ask for.
    MoreThanOneCommand,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotACommand(word) => write!(f, "{word} is not one of the commands"),
            Self::MissingValue(option) => write!(f, "{option} needs a value after it"),
            Self::NotAnOption(word) => write!(f, "{word} is not one of the options"),
            Self::MoreThanOneCommand => {
                write!(f, "give one command at a time, not two")
            }
        }
    }
}

/// What the tool prints when asked what it can do.
pub const HELP: &str = "\
wixen-mail-search-setup: put Wixen Mail's message store into the Windows Search
index, take it back out, and say which of those is true right now.

Read this first. Everything the indexer takes lands in the Windows Search index.
That index is not encrypted. It is a database under ProgramData that
any software on this computer can read, and it keeps its own copy of the subjects
and the message text. Turning this on means somebody's mail is readable outside
Wixen Mail, by anything on the machine, until the index is rebuilt.

Commands:
  status         say what is set up now and what the indexer is doing.
                 Changes nothing and needs no special rights.
  install        register the handler and tell the indexer to look.
                 Needs an administrator prompt.
  uninstall      undo all of it. Needs an administrator prompt.
  add-scope      only the half that tells the indexer to look.
  remove-scope   only the half that tells it to stop.
  register-classes
                 only the half that writes the registry entries.
                 Needs an administrator prompt.
  unregister-classes
                 only the half that takes them out.
                 Needs an administrator prompt.
  reindex        ask the indexer to visit this handler's URLs again.
  help           print this.

The two halves are separate commands because the installer runs them as
different accounts. The registry entries need the elevated installer. The crawl
scope rule names one person's mail, so it is added as whoever started setup.

Options:
  --user <SID>   whose mail this is about, as a security identifier.
                 Defaults to the account running this tool.
  --library <path>
                 where wixen_mail_search.dll is. Defaults to beside this tool.

Exit codes: 0 it worked, 1 it failed, 2 the command line was wrong.
";

/// Read a command line.
///
/// `arguments` is what came after the program's own name.
pub fn parse<I, S>(arguments: I) -> Result<Request, UsageError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut action = None;
    let mut user = None;
    let mut library = None;
    let mut words = arguments.into_iter();

    while let Some(word) = words.next() {
        let word = word.as_ref();
        match word {
            "--user" => {
                user = Some(value_after("--user", &mut words)?);
            }
            "--library" => {
                library = Some(PathBuf::from(value_after("--library", &mut words)?));
            }
            "-h" | "--help" => set_once(&mut action, Action::Help)?,
            _ if word.starts_with('-') => return Err(UsageError::NotAnOption(word.to_string())),
            _ => set_once(&mut action, command(word)?)?,
        }
    }

    Ok(Request {
        // No command at all prints the help. There is no useful default here,
        // and every other choice would be a change to the machine that nobody
        // typed.
        action: action.unwrap_or(Action::Help),
        user,
        library,
    })
}

/// The value belonging to an option.
fn value_after<I, S>(option: &'static str, words: &mut I) -> Result<String, UsageError>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    words
        .next()
        .map(|value| value.as_ref().to_string())
        // An option followed by the next option is a value somebody meant to
        // type and did not. Reading "--library --user" as a library called
        // "--user" would look up a file that cannot exist and report a missing
        // library rather than a missing value.
        .filter(|value| !value.starts_with('-'))
        .ok_or(UsageError::MissingValue(option))
}

/// Every command, and the word that asks for it.
///
/// One list, read by the parser and by the test that checks the help mentions
/// them all. Written twice, a command could be added to the parser and never
/// appear in the help, or appear in the help and not work.
pub const COMMANDS: [(&str, Action); 9] = [
    ("status", Action::Status),
    ("install", Action::Install),
    ("uninstall", Action::Uninstall),
    ("add-scope", Action::AddScope),
    ("remove-scope", Action::RemoveScope),
    ("register-classes", Action::RegisterClasses),
    ("unregister-classes", Action::UnregisterClasses),
    ("reindex", Action::Reindex),
    ("help", Action::Help),
];

/// Which command a word names.
///
/// Matched without regard to case. These are typed at a Windows prompt, which
/// does not care about the case of anything else on the line, and refusing
/// `Status` would be a refusal nobody expects.
fn command(word: &str) -> Result<Action, UsageError> {
    COMMANDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map(|(_, action)| *action)
        .ok_or_else(|| UsageError::NotACommand(word.to_string()))
}

/// Take a command, or refuse a second one.
fn set_once(action: &mut Option<Action>, asked: Action) -> Result<(), UsageError> {
    match action {
        Some(_) => Err(UsageError::MoreThanOneCommand),
        None => {
            *action = Some(asked);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_each_command_is_reachable_from_the_command_line_and_named_in_the_help() {
        // A command that cannot be typed is a command that does not exist, and
        // one the help does not mention is one nobody finds. Both lists come
        // from COMMANDS so a command added to the parser and left out of the
        // help fails here rather than going unnoticed.
        for (typed, expected) in COMMANDS {
            assert_eq!(
                parse([typed])
                    .unwrap_or_else(|e| panic!("{typed} gave {e}"))
                    .action,
                expected,
                "{typed}"
            );
            assert!(HELP.contains(typed), "the help does not mention {typed}");
        }
    }

    #[test]
    fn test_running_the_tool_with_nothing_prints_the_help_and_changes_nothing() {
        // The two useful things to do by default would be to install or to
        // report, and installing is a change to what a system service crawls
        // for everybody on the machine. Nobody types a bare command expecting
        // that.
        let empty: [&str; 0] = [];

        assert_eq!(parse(empty).expect("no arguments").action, Action::Help);
    }

    #[test]
    fn test_two_commands_in_one_run_are_refused_rather_than_resolved() {
        // "install uninstall" could mean either. Picking one leaves the machine
        // in a state the person did not ask for and gives them no sign of it,
        // because the run reports success.
        assert_eq!(
            parse(["install", "uninstall"]),
            Err(UsageError::MoreThanOneCommand)
        );
        assert_eq!(
            parse(["status", "--user", "S-1-5-18", "reindex"]),
            Err(UsageError::MoreThanOneCommand)
        );
    }

    #[test]
    fn test_a_word_that_is_not_a_command_is_refused_rather_than_ignored() {
        // A typo has to stop the run. Ignoring it would fall through to the
        // help, which looks like the tool working, and the person would think
        // they had installed something.
        assert_eq!(
            parse(["instal"]),
            Err(UsageError::NotACommand("instal".to_string()))
        );
        assert_eq!(
            parse(["--verbose"]),
            Err(UsageError::NotAnOption("--verbose".to_string()))
        );
    }

    #[test]
    fn test_an_option_left_without_its_value_is_refused() {
        // "--library --user S-1-5-18" reads as a library called "--user" unless
        // this is checked, and the failure then blames a missing file rather
        // than the line that was typed.
        assert_eq!(
            parse(["install", "--user"]),
            Err(UsageError::MissingValue("--user"))
        );
        assert_eq!(
            parse(["install", "--library"]),
            Err(UsageError::MissingValue("--library"))
        );
        assert_eq!(
            parse(["install", "--library", "--user", "S-1-5-18"]),
            Err(UsageError::MissingValue("--library"))
        );
    }

    #[test]
    fn test_the_person_and_the_library_can_both_be_named_wherever_they_appear() {
        // A command in the middle of the options is how people really type, and
        // an argument reader that only accepts one order sends somebody looking
        // for a mistake they did not make.
        let expected = Request {
            action: Action::Install,
            user: Some("S-1-5-21-99-1001".to_string()),
            library: Some(PathBuf::from(
                r"C:\Program Files\Wixen Mail\wixen_mail_search.dll",
            )),
        };

        for order in [
            vec![
                "install",
                "--user",
                "S-1-5-21-99-1001",
                "--library",
                r"C:\Program Files\Wixen Mail\wixen_mail_search.dll",
            ],
            vec![
                "--library",
                r"C:\Program Files\Wixen Mail\wixen_mail_search.dll",
                "install",
                "--user",
                "S-1-5-21-99-1001",
            ],
        ] {
            assert_eq!(
                parse(order.clone()).unwrap_or_else(|e| panic!("{order:?} gave {e}")),
                expected
            );
        }
    }

    #[test]
    fn test_a_command_typed_in_any_case_is_understood() {
        // Windows prompts do not care about case anywhere else on the line.
        assert_eq!(parse(["STATUS"]).expect("shouting").action, Action::Status);
        assert_eq!(
            parse(["Add-Scope"]).expect("mixed").action,
            Action::AddScope
        );
    }

    #[test]
    fn test_the_help_says_the_index_is_not_encrypted_before_it_says_anything_else() {
        // This is the one thing somebody has to know before running install,
        // and the tool is the place a person meets this feature without a
        // screen to warn them. A help text that leads with the commands buries
        // it.
        let warning = HELP
            .find("not encrypted")
            .expect("the help does not say the index is not encrypted");
        let commands = HELP.find("Commands:").expect("the help lists no commands");

        assert!(
            warning < commands,
            "the warning comes after the command list"
        );
        assert!(HELP.contains("any software on this computer can read"));
    }

    #[test]
    fn test_the_help_says_which_commands_need_an_administrator() {
        // Needing to elevate is the first thing that will go wrong, and a
        // person who does not know decides the tool is broken.
        assert!(HELP.contains("Needs an administrator prompt."));
        assert!(HELP.contains("Changes nothing and needs no special rights."));
    }
}

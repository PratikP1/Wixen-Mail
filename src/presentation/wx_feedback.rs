//! Send Feedback: the window that asks what a report is about, asks the
//! questions that fit, and shows the exact message before anything goes
//! (#64, #71, #78, ALPHA-02).
//!
//! Nothing here decides what a report says. The categories, their questions,
//! what is ticked, where a report goes and the message it becomes are
//! [`crate::application::feedback_report`]'s answers, and this window shows
//! them. What it decides is how the window behaves, in functions that take
//! values and return values and that the cases below drive:
//! [`what_the_doors_do`], [`reticked`], [`sender_of`], [`log_to_read`],
//! [`from_line`] and [`payload_text`]; and [`keep_a_copy`], the one that
//! writes, driven against a temporary folder.
//!
//! Sending is not here either. Send's handler is bound by the main window,
//! which owns the one sending path, and it queues what [`FeedbackDialog::composed`]
//! answers, which is what the payload box shows.
//!
//! What a test builds and reads, and what it cannot:
//! `tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs` builds
//! this window and reads its controls over MSAA. The handlers bound to a
//! choice, a box or a field are one line each that calls a named method,
//! [`FeedbackDialog::follow_the_category`], [`FeedbackDialog::tick`] and
//! [`FeedbackDialog::refresh`], which the target calls directly; that a
//! native event reaches the line is not proved by it. How the window sounds
//! is on the ledger for the tester.

use crate::application::allowed::SETTINGS_SECTION;
use crate::application::feedback_report::{
    Category, Composed, Fact, Facts, Include, LogFile, Report, compose, github_page,
};
use crate::data::account::Account;
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::theme;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use wxdragon::prelude::*;

/// The window's title.
pub const TITLE: &str = "Send Feedback";

/// What the GitHub button says for every category but a security concern.
pub const ISSUE_PAGE_LABEL: &str = "Open the &GitHub issue page";

/// What it says for a security concern.
pub const PRIVATE_PAGE_LABEL: &str = "Open &GitHub's private reporting page";

/// The account a report is sent from, and whether it may send.
#[derive(Debug, Clone)]
pub struct Sender {
    pub account: Account,
    /// Whether it is the account marked as the default, rather than the one
    /// in use because none is marked.
    pub is_default: bool,
    /// What `allowed_for(account).mail` answered when the window opened.
    pub allowed: bool,
}

/// Everything the window opens on.
#[derive(Debug, Clone)]
pub struct Opening {
    pub facts: Facts,
    pub log: Option<LogFile>,
    pub sender: Option<Sender>,
    /// Names the files a send writes; taken when the window opens, so the
    /// payload box names the file that will be attached.
    pub stamp: String,
}

/// Whether Send can be pressed, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendDoor {
    Open,
    Shut(String),
}

/// What the three ways out of the window do for one category and account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doors {
    pub send: SendDoor,
    pub github_label: &'static str,
    pub github_page: &'static str,
}

/// What Send and the GitHub button do.
pub fn what_the_doors_do(category: Category, account: Option<&Account>, allowed: bool) -> Doors {
    let _ = (category, account, allowed, SETTINGS_SECTION, github_page);
    Doors {
        send: SendDoor::Shut(String::new()),
        github_label: "",
        github_page: "",
    }
}

/// What is ticked after the category changes.
pub fn reticked(current: Include, touched: &[Fact], category: Category) -> Include {
    let _ = (touched, category);
    current
}

/// The account a report goes from.
pub fn sender_of(
    default_id: Option<&str>,
    active_id: Option<&str>,
    accounts: &[Account],
) -> Option<(Account, bool)> {
    let _ = (default_id, active_id, accounts);
    None
}

/// Which day's log the excerpt is taken from.
pub fn log_to_read(today: Option<LogFile>, yesterday: Option<LogFile>) -> Option<LogFile> {
    let _ = (today, yesterday);
    None
}

/// The name the log gives a day's file.
pub fn log_file_name(prefix: &str, day: chrono::NaiveDate) -> String {
    let _ = (prefix, day);
    String::new()
}

/// The From line of the payload box.
pub fn from_line(sender: Option<&Sender>) -> String {
    let _ = sender;
    String::new()
}

/// What the payload box shows.
pub fn payload_text(composed: &Composed, from: &str) -> String {
    let _ = (composed, from);
    String::new()
}

/// Write the copy of a report, and its excerpt when it has one.
pub fn keep_a_copy(
    dir: &Path,
    stamp: &str,
    composed: &Composed,
    payload: &str,
) -> std::result::Result<Vec<PathBuf>, String> {
    let _ = (dir, stamp, composed, payload);
    Ok(Vec::new())
}

/// The window, with its controls public so a test can read them.
#[derive(Clone)]
pub struct FeedbackDialog {
    pub dialog: Dialog,
    pub category: Choice,
    pub questions: Vec<(StaticText, TextCtrl)>,
    pub includes: Vec<(Fact, CheckBox)>,
    pub reply_to: TextCtrl,
    pub payload: TextCtrl,
    pub send: Button,
    pub copy: Button,
    pub github: Button,
    pub cancel: Button,
    pub why_not: StaticText,
    opening: Rc<Opening>,
    touched: Rc<RefCell<Vec<Fact>>>,
}

impl FeedbackDialog {
    /// The category chosen.
    pub fn chosen(&self) -> Category {
        Category::Problem
    }

    /// Choose a category, as the choice does when somebody moves through it.
    pub fn choose(&self, category: Category) {
        let _ = (category, &self.touched);
    }

    /// Tick or untick a box as the person does.
    pub fn tick(&self, fact: Fact, ticked: bool) {
        let _ = (fact, ticked);
    }

    /// The report as the window holds it.
    pub fn report(&self) -> Report {
        Report {
            category: self.chosen(),
            answers: Vec::new(),
            include: Include::for_category(self.chosen()),
            reply_to: String::new(),
            log: self.opening.log.clone(),
            stamp: self.opening.stamp.clone(),
        }
    }

    /// The message the report becomes.
    pub fn composed(&self) -> Composed {
        compose(&self.report(), &self.opening.facts)
    }

    /// What the payload box should hold now.
    pub fn payload_now(&self) -> String {
        String::new()
    }

    /// Put the report as it stands in the payload box.
    pub fn refresh(&self) {}

    /// The account the report goes from.
    pub fn sender(&self) -> Option<&Sender> {
        self.opening.sender.as_ref()
    }

    /// What the doors do for the category chosen.
    pub fn doors(&self) -> Doors {
        what_the_doors_do(self.chosen(), None, false)
    }

    /// Say under the buttons why Send did not work.
    pub fn say_why(&self, why: &str) {
        let _ = why;
    }
}

/// Build the window without showing it. Stubbed: every control is built bare,
/// unnamed and unticked, and nothing is laid out or bound.
pub fn build_feedback_dialog<W: WxWidget>(
    parent: &W,
    opening: Opening,
    palette: Option<theme::Palette>,
) -> FeedbackDialog {
    let _ = (
        palette,
        set_accessible_name,
        set_accessible_name_and_description,
    );
    let dialog = Dialog::builder(parent, TITLE).build();
    let field = || TextCtrl::builder(&dialog).build();
    FeedbackDialog {
        category: Choice::builder(&dialog).build(),
        questions: vec![(StaticText::builder(&dialog).with_label("").build(), field())],
        includes: Vec::new(),
        reply_to: field(),
        payload: field(),
        send: Button::builder(&dialog).with_label("&Send").build(),
        copy: Button::builder(&dialog)
            .with_label("&Copy to clipboard")
            .build(),
        github: Button::builder(&dialog)
            .with_label(ISSUE_PAGE_LABEL)
            .build(),
        cancel: Button::builder(&dialog).with_label("Cancel").build(),
        why_not: StaticText::builder(&dialog).with_label("").build(),
        dialog,
        opening: Rc::new(opening),
        touched: Rc::new(RefCell::new(Vec::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::feedback_report::{
        Attachment, ISSUE_PAGE, PRIVATE_REPORTING_PAGE, SECURITY_ADDRESS,
    };

    fn account(id: &str, email: &str) -> Account {
        Account {
            id: id.to_string(),
            ..Account::new("Mail".to_string(), email.to_string())
        }
    }

    fn log(date: &str, text: &str) -> LogFile {
        LogFile {
            file_name: format!("wixen-mail.{date}.log"),
            date: date.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn test_with_no_account_send_is_shut_and_says_so_and_the_other_doors_stay() {
        let doors = what_the_doors_do(Category::Problem, None, true);

        assert_eq!(
            doors.send,
            SendDoor::Shut(
                "No account is set up to send from. Copy the report to the clipboard, or \
                 open the page on GitHub."
                    .to_string()
            )
        );
        assert_eq!(doors.github_page, ISSUE_PAGE);
        assert_eq!(doors.github_label, "Open the &GitHub issue page");
    }

    #[test]
    fn test_with_sending_forbidden_send_is_shut_and_names_where_to_allow_it() {
        let dana = account("a", "dana@example.org");

        let doors = what_the_doors_do(Category::Feature, Some(&dana), false);

        assert_eq!(
            doors.send,
            SendDoor::Shut(
                "Sending is turned off under Settings, Allow Changes. Copy the report to the \
                 clipboard, or open the page on GitHub."
                    .to_string()
            )
        );
    }

    #[test]
    fn test_every_category_can_send_from_an_account_that_may_send() {
        let dana = account("a", "dana@example.org");

        for category in Category::ALL {
            assert_eq!(
                what_the_doors_do(category, Some(&dana), true).send,
                SendDoor::Open,
                "{category:?}"
            );
        }
    }

    #[test]
    fn test_a_security_concern_is_offered_the_private_page_whether_send_is_open_or_not() {
        let dana = account("a", "dana@example.org");

        for (who, allowed) in [(Some(&dana), true), (None, true), (Some(&dana), false)] {
            let doors = what_the_doors_do(Category::Security, who, allowed);
            assert_eq!(doors.github_page, PRIVATE_REPORTING_PAGE);
            assert_ne!(doors.github_page, ISSUE_PAGE);
            assert_eq!(doors.github_label, "Open &GitHub's private reporting page");
        }
    }

    #[test]
    fn test_a_box_nobody_changed_follows_the_category_and_a_changed_one_stays() {
        let on_open = Include::for_category(Category::Problem);

        let security = reticked(on_open, &[], Category::Security);
        let and_back = reticked(security, &[], Category::Question);
        let kept = reticked(
            on_open.with(Fact::Windows, true),
            &[Fact::Windows, Fact::LogExcerpt],
            Category::Security,
        );

        assert!(!security.log_excerpt, "{security:?}");
        assert!(and_back.log_excerpt, "{and_back:?}");
        assert!(kept.log_excerpt && kept.windows, "{kept:?}");
    }

    #[test]
    fn test_a_report_goes_from_the_default_account_or_else_the_one_in_use() {
        let accounts = vec![account("a", "a@example.org"), account("b", "b@example.org")];

        let default = sender_of(Some("b"), Some("a"), &accounts);
        let in_use = sender_of(None, Some("a"), &accounts);
        let gone = sender_of(Some("z"), None, &accounts);

        assert_eq!(
            default.map(|(a, d)| (a.id, d)),
            Some(("b".to_string(), true))
        );
        assert_eq!(
            in_use.map(|(a, d)| (a.id, d)),
            Some(("a".to_string(), false))
        );
        assert!(gone.is_none());
    }

    #[test]
    fn test_the_excerpt_is_from_today_unless_today_has_nothing_written() {
        let today = log("2026-09-23", "INFO started\n");
        let yesterday = log("2026-09-22", "INFO older\n");

        let read = log_to_read(Some(today.clone()), Some(yesterday.clone()));
        let empty = log_to_read(Some(log("2026-09-23", "\n")), Some(yesterday.clone()));
        let missing = log_to_read(None, Some(yesterday.clone()));

        assert_eq!(read, Some(today));
        assert_eq!(empty, Some(yesterday.clone()));
        assert_eq!(missing, Some(yesterday));
        assert_eq!(log_to_read(None, None), None);
    }

    #[test]
    fn test_a_days_log_is_named_the_way_the_log_names_it() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 9, 3).expect("a date");

        assert_eq!(
            log_file_name("wixen-mail", day),
            "wixen-mail.2026-09-03.log"
        );
    }

    #[test]
    fn test_the_payload_names_where_it_goes_who_sends_it_and_what_is_attached() {
        let composed = Composed {
            to: SECURITY_ADDRESS,
            subject: "[Wixen Mail] Report a security concern".to_string(),
            body: "The body.\n".to_string(),
            attachment: Some(Attachment {
                file_name: "1-log-excerpt.txt".to_string(),
                text: "The last 1 lines.\nINFO done\n".to_string(),
            }),
        };

        let shown = payload_text(&composed, "dana@example.org, your default account");

        assert_eq!(
            shown,
            "To: security@wixen.app\nFrom: dana@example.org, your default account\n\
             Subject: [Wixen Mail] Report a security concern\n\nThe body.\n\
             \nAttached file, 1-log-excerpt.txt:\nThe last 1 lines.\nINFO done\n"
        );
    }

    #[test]
    fn test_the_from_line_says_which_account_and_why_that_one() {
        let dana = account("a", "dana@example.org");
        let marked = Sender {
            account: dana.clone(),
            is_default: true,
            allowed: true,
        };
        let in_use = Sender {
            is_default: false,
            ..marked.clone()
        };

        assert_eq!(
            from_line(Some(&marked)),
            "dana@example.org, your default account"
        );
        assert_eq!(
            from_line(Some(&in_use)),
            "dana@example.org, the account in use, since none is marked as the default"
        );
        assert_eq!(from_line(None), "no account, so Send cannot be used");
    }

    #[test]
    fn test_a_copy_is_kept_for_every_report_and_the_excerpt_only_when_it_goes() {
        let dir = tempfile::TempDir::new().expect("a folder");
        let feedback = dir.path().join("logs").join("feedback");
        let with_excerpt = Composed {
            to: "support@wixen.app",
            subject: "s".to_string(),
            body: "b\n".to_string(),
            attachment: Some(Attachment {
                file_name: "7-log-excerpt.txt".to_string(),
                text: "excerpt\n".to_string(),
            }),
        };
        let without = Composed {
            attachment: None,
            ..with_excerpt.clone()
        };

        let attached = keep_a_copy(&feedback, "7", &with_excerpt, "payload 7").expect("kept");
        let none = keep_a_copy(&feedback, "8", &without, "payload 8").expect("kept");

        assert_eq!(attached, vec![feedback.join("7-log-excerpt.txt")]);
        assert!(none.is_empty(), "{none:?}");
        assert_eq!(
            std::fs::read_to_string(feedback.join("7-report.txt"))
                .ok()
                .as_deref(),
            Some("payload 7")
        );
        assert_eq!(
            std::fs::read_to_string(feedback.join("7-log-excerpt.txt"))
                .ok()
                .as_deref(),
            Some("excerpt\n")
        );
        assert!(feedback.join("8-report.txt").exists());
        assert!(!feedback.join("8-log-excerpt.txt").exists());
    }

    #[test]
    fn test_a_copy_that_cannot_be_written_says_so_and_that_nothing_went() {
        let dir = tempfile::TempDir::new().expect("a folder");
        let a_file = dir.path().join("taken");
        std::fs::write(&a_file, "not a folder").expect("a file");
        let composed = Composed {
            to: "support@wixen.app",
            subject: "s".to_string(),
            body: "b\n".to_string(),
            attachment: None,
        };

        let refused = keep_a_copy(&a_file, "9", &composed, "payload");

        assert!(
            refused
                .as_ref()
                .is_err_and(|why| why.contains("Nothing was sent")),
            "{refused:?}"
        );
    }
}

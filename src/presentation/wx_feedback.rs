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
///
/// Send is open for any category, a security concern included (Pratik,
/// 2026-09-23), when there is an account and it may send; otherwise it is
/// shut with one sentence saying why, and Copy and the GitHub button stay.
/// The GitHub button opens [`github_page`]'s answer, so a security concern is
/// offered the private reporting page, beside Send whether Send is open or
/// not, and never the public issue page.
pub fn what_the_doors_do(category: Category, account: Option<&Account>, allowed: bool) -> Doors {
    let send = match (account, allowed) {
        (None, _) => SendDoor::Shut(
            "No account is set up to send from. Copy the report to the clipboard, or open \
             the page on GitHub."
                .to_string(),
        ),
        (Some(_), false) => SendDoor::Shut(format!(
            "Sending is turned off under Settings, {SETTINGS_SECTION}. Copy the report to \
             the clipboard, or open the page on GitHub."
        )),
        (Some(_), true) => SendDoor::Open,
    };
    Doors {
        send,
        github_label: match category {
            Category::Security => PRIVATE_PAGE_LABEL,
            _ => ISSUE_PAGE_LABEL,
        },
        github_page: github_page(category),
    }
}

/// What is ticked after the category changes.
///
/// A box the person changed stays as they left it; every other box follows
/// the new category's defaults. The planner's choice, said so it can be
/// overruled: somebody who moved to a security concern has not chosen to
/// send a log, and somebody who ticked the box has.
pub fn reticked(current: Include, touched: &[Fact], category: Category) -> Include {
    Fact::ALL.into_iter().fold(
        Include::for_category(category),
        |include, fact| match touched.contains(&fact) {
            true => include.with(fact, current.includes(fact)),
            false => include,
        },
    )
}

/// The account a report goes from: the default account, or the account in
/// use when none is marked as the default.
pub fn sender_of(
    default_id: Option<&str>,
    active_id: Option<&str>,
    accounts: &[Account],
) -> Option<(Account, bool)> {
    crate::application::notes_backend::default_account(default_id, accounts)
        .map(|account| (account.clone(), true))
        .or_else(|| {
            active_id
                .and_then(|id| accounts.iter().find(|account| account.id == id))
                .map(|account| (account.clone(), false))
        })
}

/// Which day's log the excerpt is taken from: today's, or yesterday's when
/// today's is missing or empty, which is a program started today that has
/// written nothing yet.
pub fn log_to_read(today: Option<LogFile>, yesterday: Option<LogFile>) -> Option<LogFile> {
    let written = |log: &LogFile| !log.text.trim().is_empty();
    today.filter(written).or(yesterday.filter(written))
}

/// The name the log gives a day's file. The log rolls over by the date in
/// UTC, so the day handed in is a UTC day.
pub fn log_file_name(prefix: &str, day: chrono::NaiveDate) -> String {
    format!("{prefix}.{}.log", day.format("%Y-%m-%d"))
}

/// The From line of the payload box.
pub fn from_line(sender: Option<&Sender>) -> String {
    match sender {
        None => "no account, so Send cannot be used".to_string(),
        Some(sender) if sender.is_default => {
            format!("{}, your default account", sender.account.email)
        }
        Some(sender) => format!(
            "{}, the account in use, since none is marked as the default",
            sender.account.email
        ),
    }
}

/// What the payload box shows: exactly what Send queues, the attached file
/// included.
pub fn payload_text(composed: &Composed, from: &str) -> String {
    let mut text = format!(
        "To: {}\nFrom: {from}\nSubject: {}\n\n{}",
        composed.to, composed.subject, composed.body
    );
    if let Some(attached) = &composed.attachment {
        text.push_str(&format!(
            "\nAttached file, {}:\n{}",
            attached.file_name, attached.text
        ));
    }
    text
}

/// Write the copy of a report, and its excerpt when it has one, and answer
/// the files to attach.
///
/// The copy is written for every report, because it is what somebody sends
/// again from if a report bounces. The excerpt is written only when the
/// report carries one, which is only when its box is ticked.
pub fn keep_a_copy(
    dir: &Path,
    stamp: &str,
    composed: &Composed,
    payload: &str,
) -> std::result::Result<Vec<PathBuf>, String> {
    let could_not = |what: &Path, why: std::io::Error| {
        format!(
            "The report could not be kept in {}: {why}. Nothing was sent.",
            what.display()
        )
    };
    std::fs::create_dir_all(dir).map_err(|why| could_not(dir, why))?;
    let copy = dir.join(format!("{stamp}-report.txt"));
    std::fs::write(&copy, payload).map_err(|why| could_not(&copy, why))?;
    let mut attachments = Vec::new();
    if let Some(attached) = &composed.attachment {
        let excerpt = dir.join(&attached.file_name);
        std::fs::write(&excerpt, &attached.text).map_err(|why| could_not(&excerpt, why))?;
        attachments.push(excerpt);
    }
    Ok(attachments)
}

/// The window, with its controls public so a test can read them.
#[derive(Clone)]
pub struct FeedbackDialog {
    pub dialog: Dialog,
    pub category: Choice,
    /// One label and one field per question the most-asking category asks.
    pub questions: Vec<(StaticText, TextCtrl)>,
    /// One box per fact, in [`Fact::ALL`]'s order.
    pub includes: Vec<(Fact, CheckBox)>,
    pub reply_to: TextCtrl,
    pub payload: TextCtrl,
    pub send: Button,
    pub copy: Button,
    pub github: Button,
    pub cancel: Button,
    /// Under the buttons: why Send cannot be used, or why it did not work.
    pub why_not: StaticText,
    opening: Rc<Opening>,
    touched: Rc<RefCell<Vec<Fact>>>,
}

impl FeedbackDialog {
    /// The category chosen.
    pub fn chosen(&self) -> Category {
        self.category
            .get_selection()
            .and_then(|at| Category::ALL.get(at as usize).copied())
            .unwrap_or(Category::Problem)
    }

    /// Choose a category, as the choice does when somebody moves through it.
    pub fn choose(&self, category: Category) {
        if let Some(at) = Category::ALL.iter().position(|each| *each == category) {
            self.category.set_selection(at as u32);
        }
        self.follow_the_category();
    }

    /// Everything that changes with the category: the questions, the boxes
    /// nobody has changed, the doors and the payload.
    pub fn follow_the_category(&self) {
        let category = self.chosen();
        let asked = category.questions();
        for (at, (label, field)) in self.questions.iter().enumerate() {
            match asked.get(at) {
                Some(question) => {
                    label.set_label(question.asked);
                    set_accessible_name_and_description(field, question.asked, question.prompt);
                    label.show(true);
                    field.show(true);
                }
                None => {
                    label.show(false);
                    field.show(false);
                }
            }
        }
        let ticked = reticked(self.include(), &self.touched.borrow(), category);
        for (fact, check) in &self.includes {
            check.set_value(ticked.includes(*fact));
        }
        self.open_the_doors();
        self.refresh();
        self.dialog.layout();
    }

    /// Tick or untick a box as the person does, which the category then
    /// leaves alone.
    pub fn tick(&self, fact: Fact, ticked: bool) {
        if let Some((_, check)) = self.includes.iter().find(|(each, _)| *each == fact) {
            check.set_value(ticked);
        }
        let mut touched = self.touched.borrow_mut();
        if !touched.contains(&fact) {
            touched.push(fact);
        }
        drop(touched);
        self.refresh();
    }

    /// What the boxes say now.
    fn include(&self) -> Include {
        self.includes.iter().fold(
            Include::for_category(self.chosen()),
            |include, (fact, check)| include.with(*fact, check.get_value()),
        )
    }

    /// The report as the window holds it.
    pub fn report(&self) -> Report {
        let category = self.chosen();
        Report {
            category,
            answers: self
                .questions
                .iter()
                .take(category.questions().len())
                .map(|(_, field)| field.get_value())
                .collect(),
            include: self.include(),
            reply_to: self.reply_to.get_value(),
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
        payload_text(&self.composed(), &from_line(self.opening.sender.as_ref()))
    }

    /// Put the report as it stands in the payload box.
    pub fn refresh(&self) {
        self.payload.change_value(&self.payload_now());
    }

    /// The account the report goes from.
    pub fn sender(&self) -> Option<&Sender> {
        self.opening.sender.as_ref()
    }

    /// What the doors do for the category chosen.
    pub fn doors(&self) -> Doors {
        let sender = self.opening.sender.as_ref();
        what_the_doors_do(
            self.chosen(),
            sender.map(|sender| &sender.account),
            sender.is_some_and(|sender| sender.allowed),
        )
    }

    fn open_the_doors(&self) {
        let doors = self.doors();
        self.github.set_label(doors.github_label);
        set_accessible_name(&self.github, &doors.github_label.replace('&', ""));
        match doors.send {
            SendDoor::Open => {
                self.send.enable(true);
                self.why_not.set_label("");
            }
            SendDoor::Shut(why) => {
                self.send.enable(false);
                self.why_not.set_label(&why);
            }
        }
    }

    /// Say under the buttons why Send did not work, keeping everything typed.
    pub fn say_why(&self, why: &str) {
        self.why_not.set_label(why);
        self.dialog.layout();
    }
}

/// Build the window without showing it.
///
/// Opens on the first category, with its boxes ticked by
/// [`Include::for_category`], so the log excerpt is ticked on open, and with
/// the focus on the category. The access keys are W, R, B, S, C and G, one
/// each; the question fields and the boxes are reached by Tab.
pub fn build_feedback_dialog<W: WxWidget>(
    parent: &W,
    opening: Opening,
    palette: Option<theme::Palette>,
) -> FeedbackDialog {
    let dialog = Dialog::builder(parent, TITLE)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let category_label = StaticText::builder(&dialog)
        .with_label("&What is this about")
        .build();
    let labels: Vec<String> = Category::ALL
        .iter()
        .map(|category| category.label().to_string())
        .collect();
    let category = Choice::builder(&dialog)
        .with_choices(labels)
        .with_selection(Some(0))
        .build();
    set_accessible_name_and_description(
        &category,
        "What is this about",
        "The questions below change with what you choose",
    );
    sizer.add(&category_label, 0, SizerFlag::All, 4);
    sizer.add(&category, 0, SizerFlag::Expand | SizerFlag::All, 4);

    let most_asked = Category::ALL
        .iter()
        .map(|category| category.questions().len())
        .max()
        .unwrap_or(1);
    let questions: Vec<(StaticText, TextCtrl)> = (0..most_asked)
        .map(|_| {
            let label = StaticText::builder(&dialog).with_label("").build();
            let field = TextCtrl::builder(&dialog)
                .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
                .with_size(Size::new(520, 90))
                .build();
            sizer.add(&label, 0, SizerFlag::All, 4);
            sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 4);
            (label, field)
        })
        .collect();

    let include_heading = StaticText::builder(&dialog)
        .with_label("What to include")
        .build();
    sizer.add(&include_heading, 0, SizerFlag::All, 4);
    let includes: Vec<(Fact, CheckBox)> = Fact::ALL
        .into_iter()
        .map(|fact| {
            let check = CheckBox::builder(&dialog).with_label(fact.sends()).build();
            set_accessible_name(&check, fact.sends());
            sizer.add(&check, 0, SizerFlag::All, 4);
            (fact, check)
        })
        .collect();

    let reply_label = StaticText::builder(&dialog)
        .with_label("How to &reach you")
        .build();
    let reply_to = TextCtrl::builder(&dialog)
        .with_value(
            opening
                .sender
                .as_ref()
                .map(|sender| sender.account.email.as_str())
                .unwrap_or_default(),
        )
        .build();
    set_accessible_name_and_description(
        &reply_to,
        "How to reach you",
        "Used only to answer you. Leave it empty if you do not want an answer",
    );
    sizer.add(&reply_label, 0, SizerFlag::All, 4);
    sizer.add(&reply_to, 0, SizerFlag::Expand | SizerFlag::All, 4);

    let payload_label = StaticText::builder(&dialog)
        .with_label("What will &be sent")
        .build();
    let payload = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly | TextCtrlStyle::WordWrap)
        .with_size(Size::new(520, 200))
        .build();
    set_accessible_name_and_description(
        &payload,
        "What will be sent",
        "Exactly the message Send puts in your Outbox, and the file it attaches",
    );
    sizer.add(&payload_label, 0, SizerFlag::All, 4);
    sizer.add(&payload, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let send = Button::builder(&dialog).with_label("&Send").build();
    set_accessible_name(&send, "Send");
    let copy = Button::builder(&dialog)
        .with_label("&Copy to clipboard")
        .build();
    set_accessible_name(&copy, "Copy to clipboard");
    let github = Button::builder(&dialog)
        .with_label(ISSUE_PAGE_LABEL)
        .build();
    let cancel = Button::builder(&dialog)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    for button in [&send, &copy, &github, &cancel] {
        buttons.add(button, 0, SizerFlag::All, 4);
    }
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    let why_not = StaticText::builder(&dialog).with_label("").build();
    sizer.add(&why_not, 0, SizerFlag::Expand | SizerFlag::All, 8);

    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        for (_, field) in &questions {
            theme::paint(field, palette.second_surface());
        }
        theme::paint(&reply_to, palette.second_surface());
        theme::paint(&payload, palette.second_surface());
    }

    let feedback = FeedbackDialog {
        dialog,
        category,
        questions,
        includes,
        reply_to,
        payload,
        send,
        copy,
        github,
        cancel,
        why_not,
        opening: Rc::new(opening),
        touched: Rc::new(RefCell::new(Vec::new())),
    };
    bind_the_refreshes(&feedback);
    feedback.follow_the_category();

    feedback.dialog.set_sizer_and_fit(sizer, true);
    feedback.category.set_focus();
    feedback
}

/// Each handler is one line calling a named method, so the method a test
/// calls is the whole of what the event does.
fn bind_the_refreshes(feedback: &FeedbackDialog) {
    let following = feedback.clone();
    feedback
        .category
        .on_selection_changed(move |_| following.follow_the_category());
    for (fact, check) in &feedback.includes {
        let (ticking, fact, box_read) = (feedback.clone(), *fact, *check);
        check.on_toggled(move |_| ticking.tick(fact, box_read.get_value()));
    }
    let fields = feedback
        .questions
        .iter()
        .map(|(_, field)| field)
        .chain(std::iter::once(&feedback.reply_to));
    for field in fields {
        let refreshing = feedback.clone();
        field.on_text_changed(move |_| refreshing.refresh());
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

        // The address through its constant, so the one line Pratik corrects
        // is the only place in `src` that spells it.
        assert_eq!(
            shown,
            format!(
                "To: {SECURITY_ADDRESS}\nFrom: dana@example.org, your default account\n\
                 Subject: [Wixen Mail] Report a security concern\n\nThe body.\n\
                 \nAttached file, 1-log-excerpt.txt:\nThe last 1 lines.\nINFO done\n"
            )
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

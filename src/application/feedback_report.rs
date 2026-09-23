//! A feedback report, composed from what somebody typed and what this
//! machine knows, before anything is sent (#64, #71, ALPHA-02).
//!
//! Everything here is a value in and a value out. The dialog in
//! `presentation::wx_feedback` shows [`compose`]'s answer in a box the person
//! can read before pressing Send, and Send queues that same answer, so what is
//! shown is what is sent. Reading the log and asking Windows about itself
//! happen outside this module, in `service::this_machine` and the presentation
//! layer, and arrive here as plain text.
//!
//! Two addresses, each written once. [`SUPPORT_ADDRESS`] takes five of the six
//! categories. [`SECURITY_ADDRESS`] takes a security concern, on Pratik's
//! answer of 2026-09-23 that security reports go to an address of their own;
//! he did not name it, so `security@wixen.app` is the planner's assumption for
//! him to correct, and correcting it is that one line and the pages that name
//! it.
//!
//! A security report differs in three ways, each held by a case below: it goes
//! to its own address, its subject carries the category and none of the
//! concern, and its log excerpt starts unticked, because a log can hold what
//! the concern is about and [`redact`] masks addresses and subjects only.

use crate::common::logging::mask_email;

/// Where five of the six categories go.
pub const SUPPORT_ADDRESS: &str = "support@wixen.app";

/// Where a security concern goes. The planner's assumption of 2026-09-23, for
/// Pratik to correct; nothing else in the tree spells it.
pub const SECURITY_ADDRESS: &str = "security@wixen.app";

/// GitHub's page for opening an issue, which lets the person choose a template.
pub const ISSUE_PAGE: &str = "https://github.com/PratikP1/Wixen-Mail/issues/new/choose";

/// GitHub's private vulnerability reporting page, switched on for this
/// repository, where a report is seen by the maintainers alone.
pub const PRIVATE_REPORTING_PAGE: &str =
    "https://github.com/PratikP1/Wixen-Mail/security/advisories/new";

/// How many lines of the log an excerpt carries, from the end.
pub const EXCERPT_LINES: usize = 200;

/// What a report is about, chosen first, because it decides the questions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Problem,
    Feature,
    ScreenReaderBarrier,
    Question,
    Security,
    Other,
}

/// One question the dialog asks, with the one-line prompt said under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Question {
    pub asked: &'static str,
    pub prompt: &'static str,
}

impl Category {
    /// Every category, in the order the dialog offers them.
    pub const ALL: [Category; 6] = [
        Category::Problem,
        Category::Feature,
        Category::ScreenReaderBarrier,
        Category::Question,
        Category::Security,
        Category::Other,
    ];

    /// What the dialog calls it, in the words #64 gives.
    pub fn label(self) -> &'static str {
        ""
    }

    /// The questions that fit it. Only the first needs an answer.
    pub fn questions(self) -> &'static [Question] {
        &[]
    }
}

/// One fact about this machine a report can carry, in the order the dialog
/// offers the boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fact {
    Version,
    Windows,
    ScreenReader,
    LogExcerpt,
    Providers,
}

impl Fact {
    /// Every fact, in the dialog's order.
    pub const ALL: [Fact; 5] = [
        Fact::Version,
        Fact::Windows,
        Fact::ScreenReader,
        Fact::LogExcerpt,
        Fact::Providers,
    ];

    /// What ticking this box sends, as the box's label says it.
    pub fn sends(self) -> &'static str {
        ""
    }
}

/// Which facts go with a report.
///
/// No `Default`: the only way to get one is [`Include::for_category`], so no
/// call site can take a default that ignores what the report is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Include {
    pub version: bool,
    pub windows: bool,
    pub screen_reader: bool,
    pub log_excerpt: bool,
    pub providers: bool,
}

impl Include {
    /// What is ticked when a category is chosen: the version and the log
    /// excerpt for five categories (Pratik's decision on #71), the version
    /// alone for a security concern.
    pub fn for_category(category: Category) -> Include {
        let _ = category;
        Include {
            version: false,
            windows: false,
            screen_reader: false,
            log_excerpt: false,
            providers: false,
        }
    }

    /// Whether one fact is ticked.
    pub fn includes(&self, fact: Fact) -> bool {
        let _ = fact;
        false
    }

    /// The same, with one fact ticked or not.
    pub fn with(self, fact: Fact, ticked: bool) -> Include {
        let _ = (fact, ticked);
        self
    }
}

/// Where a report goes.
pub fn where_it_goes(category: Category) -> &'static str {
    let _ = category;
    ""
}

/// Which GitHub page is offered beside Send.
pub fn github_page(category: Category) -> &'static str {
    let _ = category;
    ""
}

/// What this machine says about itself, gathered before the dialog opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub version: String,
    pub windows_build: String,
    pub display_language: String,
    /// The screen reader running and its version, when one was found.
    pub screen_reader: Option<(String, String)>,
    /// The kinds of account set up, never their addresses.
    pub providers: Vec<String>,
}

/// The log file a report may carry the end of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFile {
    /// The file's own name, such as `wixen-mail.2026-09-23.log`.
    pub file_name: String,
    /// The day it covers.
    pub date: String,
    pub text: String,
}

/// What the person filled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub category: Category,
    /// One answer per question, in the question's order.
    pub answers: Vec<String>,
    pub include: Include,
    /// Where the person would like an answer, when they gave one.
    pub reply_to: String,
    /// The log to take the excerpt from, when there is one.
    pub log: Option<LogFile>,
    /// Names the files a send writes, such as `20260923-184512`.
    pub stamp: String,
}

/// A file sent with the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub file_name: String,
    pub text: String,
}

/// The message a report becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composed {
    pub to: &'static str,
    pub subject: String,
    pub body: String,
    pub attachment: Option<Attachment>,
}

/// The message a report becomes, with no I/O.
pub fn compose(report: &Report, facts: &Facts) -> Composed {
    let _ = (report, facts);
    Composed {
        to: "",
        subject: String::new(),
        body: String::new(),
        attachment: None,
    }
}

/// The lines of a log with every address masked and every subject removed.
///
/// The boundary between the log and the outside, so it errs towards masking:
/// anything shaped `local@host` is treated as an address.
pub fn redact(lines: &[String]) -> Vec<String> {
    let _ = (lines, mask_email);
    Vec::new()
}

/// The last `n` lines of a text.
pub fn last_lines(text: &str, n: usize) -> Vec<String> {
    let _ = (text, n);
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            version: "1.0.0-alpha.1+7.gabc1234".to_string(),
            windows_build: "Windows 11, build 26200".to_string(),
            display_language: "en-GB".to_string(),
            screen_reader: Some(("NVDA".to_string(), "2025.3.0.1".to_string())),
            providers: vec!["Gmail".to_string(), "IMAP".to_string()],
        }
    }

    fn log() -> LogFile {
        LogFile {
            file_name: "wixen-mail.2026-09-23.log".to_string(),
            date: "2026-09-23".to_string(),
            text: "INFO sync started\nWARN could not reach alice@example.com\nINFO done\n"
                .to_string(),
        }
    }

    fn report(category: Category, first_answer: &str) -> Report {
        Report {
            category,
            answers: vec![first_answer.to_string()],
            include: Include::for_category(category),
            reply_to: String::new(),
            log: Some(log()),
            stamp: "20260923-184512".to_string(),
        }
    }

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|line| line.to_string()).collect()
    }

    // ── The categories and their questions ──────────────────────────────

    #[test]
    fn test_each_category_is_offered_in_the_words_the_tester_gave() {
        let labels: Vec<&str> = Category::ALL.iter().map(|c| c.label()).collect();

        assert_eq!(
            labels,
            [
                "Report a problem",
                "Request a feature",
                "Something is hard to use with a screen reader",
                "Ask a question",
                "Report a security concern",
                "Something else",
            ]
        );
    }

    #[test]
    fn test_a_problem_asks_two_questions_and_every_other_category_asks_one() {
        for category in Category::ALL {
            let wanted = match category {
                Category::Problem => 2,
                _ => 1,
            };
            let questions = category.questions();
            assert_eq!(questions.len(), wanted, "{category:?}");
            for question in questions {
                assert!(
                    question.asked.ends_with('?') && !question.prompt.trim().is_empty(),
                    "{category:?} asks {question:?}"
                );
            }
        }
        assert_eq!(
            Category::Problem.questions()[0].asked,
            "What were you doing, and what did you hear or see?"
        );
        assert_eq!(
            Category::Problem.questions()[1].asked,
            "What did you expect instead?"
        );
    }

    // ── What is ticked ──────────────────────────────────────────────────

    #[test]
    fn test_five_categories_tick_the_version_and_the_log_excerpt_and_nothing_else() {
        for category in Category::ALL
            .into_iter()
            .filter(|c| *c != Category::Security)
        {
            assert_eq!(
                Include::for_category(category),
                Include {
                    version: true,
                    windows: false,
                    screen_reader: false,
                    log_excerpt: true,
                    providers: false,
                },
                "{category:?}"
            );
        }
    }

    #[test]
    fn test_a_security_concern_ticks_the_version_alone_and_leaves_the_log_unticked() {
        assert_eq!(
            Include::for_category(Category::Security),
            Include {
                version: true,
                windows: false,
                screen_reader: false,
                log_excerpt: false,
                providers: false,
            }
        );
    }

    #[test]
    fn test_one_fact_is_ticked_and_read_back_without_touching_the_others() {
        let problem = Include::for_category(Category::Problem);

        let changed = problem
            .with(Fact::Windows, true)
            .with(Fact::LogExcerpt, false);

        let read: Vec<bool> = Fact::ALL.iter().map(|f| changed.includes(*f)).collect();
        assert_eq!(read, [true, true, false, false, false]);
    }

    #[test]
    fn test_each_box_says_what_it_sends_in_a_sentence_of_its_own() {
        let said: Vec<&str> = Fact::ALL.iter().map(|f| f.sends()).collect();

        for (fact, sentence) in Fact::ALL.iter().zip(&said) {
            assert!(!sentence.trim().is_empty(), "{fact:?} says nothing");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            said.len(),
            "two boxes say the same: {said:?}"
        );
        assert!(
            Fact::LogExcerpt
                .sends()
                .contains("addresses and subjects hidden"),
            "{}",
            Fact::LogExcerpt.sends()
        );
        assert!(
            Fact::Providers.sends().contains("without their addresses"),
            "{}",
            Fact::Providers.sends()
        );
    }

    // ── Where it goes ───────────────────────────────────────────────────

    #[test]
    fn test_a_security_concern_goes_to_its_own_address_and_the_rest_to_support() {
        for category in Category::ALL {
            let wanted = match category {
                Category::Security => "security@wixen.app",
                _ => "support@wixen.app",
            };
            assert_eq!(where_it_goes(category), wanted, "{category:?}");
        }
    }

    #[test]
    fn test_a_security_concern_is_offered_the_private_page_and_never_the_issue_page() {
        for category in Category::ALL {
            let wanted = match category {
                Category::Security => {
                    "https://github.com/PratikP1/Wixen-Mail/security/advisories/new"
                }
                _ => "https://github.com/PratikP1/Wixen-Mail/issues/new/choose",
            };
            assert_eq!(github_page(category), wanted, "{category:?}");
        }
    }

    // ── What the report becomes ─────────────────────────────────────────

    #[test]
    fn test_a_subject_is_the_category_and_the_first_line_of_the_first_answer() {
        let composed = compose(
            &report(
                Category::Problem,
                "The reader went quiet\nafter I pressed Enter",
            ),
            &facts(),
        );

        assert_eq!(
            composed.subject,
            "[Wixen Mail] Report a problem: The reader went quiet"
        );
    }

    #[test]
    fn test_a_subject_carries_at_most_eighty_characters_of_the_answer() {
        let long = "é".repeat(100);

        let composed = compose(&report(Category::Feature, &long), &facts());

        assert_eq!(
            composed.subject,
            format!("[Wixen Mail] Request a feature: {}", "é".repeat(80))
        );
    }

    #[test]
    fn test_a_security_subject_carries_the_category_and_none_of_the_concern() {
        let composed = compose(
            &report(Category::Security, "Tokens are written to the log"),
            &facts(),
        );

        assert_eq!(composed.subject, "[Wixen Mail] Report a security concern");
        assert!(!composed.subject.contains("Tokens"), "{}", composed.subject);
    }

    #[test]
    fn test_the_report_goes_where_its_category_says_and_its_last_line_names_that_address() {
        for (category, address) in [
            (Category::Problem, SUPPORT_ADDRESS),
            (Category::Security, SECURITY_ADDRESS),
        ] {
            let composed = compose(&report(category, "Something"), &facts());

            assert_eq!(composed.to, address, "{category:?}");
            let last = composed.body.trim_end().lines().last().unwrap_or_default();
            assert!(
                last.contains(address) && last.contains("copy"),
                "{category:?} ends {last:?}"
            );
        }
    }

    #[test]
    fn test_each_answer_sits_under_its_question_in_order() {
        let mut problem = report(Category::Problem, "I pressed Delete");
        problem.answers.push("The message to go".to_string());

        let body = compose(&problem, &facts()).body;

        let first =
            body.find("What were you doing, and what did you hear or see?\nI pressed Delete");
        let second = body.find("What did you expect instead?\nThe message to go");
        assert!(
            matches!((first, second), (Some(a), Some(b)) if a < b),
            "{body}"
        );
    }

    #[test]
    fn test_a_ticked_fact_is_carried_and_an_unticked_one_is_not() {
        let mut problem = report(Category::Problem, "Something");
        problem.include = problem.include.with(Fact::ScreenReader, true);

        let body = compose(&problem, &facts()).body;

        assert!(body.contains("Version: 1.0.0-alpha.1+7.gabc1234"), "{body}");
        assert!(body.contains("Screen reader: NVDA 2025.3.0.1"), "{body}");
        assert!(!body.contains("Windows 11"), "{body}");
        assert!(!body.contains("Gmail"), "{body}");
    }

    #[test]
    fn test_a_reply_line_is_carried_only_when_one_was_given() {
        let mut given = report(Category::Question, "How do I?");
        given.reply_to = "me@example.org".to_string();

        let with = compose(&given, &facts()).body;
        let without = compose(&report(Category::Question, "How do I?"), &facts()).body;

        assert!(with.contains("Reply to: me@example.org"), "{with}");
        assert!(!without.contains("Reply to"), "{without}");
    }

    #[test]
    fn test_a_problem_carries_the_redacted_excerpt_under_a_line_naming_its_file_and_day() {
        let composed = compose(&report(Category::Problem, "Something"), &facts());

        let attachment = composed.attachment.expect("a problem carries the excerpt");
        assert_eq!(attachment.file_name, "20260923-184512-log-excerpt.txt");
        let header = attachment.text.lines().next().unwrap_or_default();
        assert!(
            header.contains("wixen-mail.2026-09-23.log") && header.contains("2026-09-23"),
            "{header}"
        );
        assert!(
            attachment.text.contains("al***@example.com"),
            "{}",
            attachment.text
        );
        assert!(!attachment.text.contains("alice@"), "{}", attachment.text);
        assert!(
            composed.body.contains("20260923-184512-log-excerpt.txt"),
            "the body does not name what is attached: {}",
            composed.body
        );
    }

    #[test]
    fn test_a_security_concern_carries_no_excerpt_unless_it_is_ticked() {
        let security = report(Category::Security, "Something");

        let unticked = compose(&security, &facts());
        let ticked = compose(
            &Report {
                include: security.include.with(Fact::LogExcerpt, true),
                ..security.clone()
            },
            &facts(),
        );

        assert_eq!(unticked.attachment, None);
        assert!(!unticked.body.contains("log-excerpt"), "{}", unticked.body);
        assert!(ticked.attachment.is_some(), "ticked, the excerpt goes");
    }

    // ── Redaction ───────────────────────────────────────────────────────

    #[test]
    fn test_an_address_in_the_middle_of_a_line_is_masked() {
        let redacted = redact(&lines(&["WARN could not reach alice@example.com today"]));

        assert_eq!(
            redacted,
            lines(&["WARN could not reach al***@example.com today"])
        );
    }

    #[test]
    fn test_two_addresses_on_one_line_are_both_masked() {
        let redacted = redact(&lines(&["from=jo@a.org to=<carol.d@b.net>"]));

        assert_eq!(redacted, lines(&["from=***@a.org to=<ca***@b.net>"]));
    }

    #[test]
    fn test_whatever_follows_a_subject_is_removed() {
        let redacted = redact(&lines(&[
            "INFO queued Subject: Test results for Dana",
            "DEBUG saved subject=\"Rent\" folder=Inbox",
        ]));

        assert_eq!(
            redacted,
            lines(&[
                "INFO queued Subject: [subject redacted]",
                "DEBUG saved subject=[subject redacted]",
            ])
        );
    }

    #[test]
    fn test_a_line_with_no_address_and_no_subject_is_kept_as_it_was() {
        let kept = lines(&["INFO folder Archive/2026 synced, 40 messages"]);

        assert_eq!(redact(&kept), kept);
    }

    #[test]
    fn test_the_excerpt_is_the_end_of_the_log() {
        let text = (1..=250).map(|n| format!("line {n}\n")).collect::<String>();

        let tail = last_lines(&text, EXCERPT_LINES);

        assert_eq!(tail.len(), 200);
        assert_eq!(tail.first().map(String::as_str), Some("line 51"));
        assert_eq!(tail.last().map(String::as_str), Some("line 250"));
        assert_eq!(last_lines("one\ntwo\n", 200), lines(&["one", "two"]));
    }
}

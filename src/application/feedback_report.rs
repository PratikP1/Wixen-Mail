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
        match self {
            Category::Problem => "Report a problem",
            Category::Feature => "Request a feature",
            Category::ScreenReaderBarrier => "Something is hard to use with a screen reader",
            Category::Question => "Ask a question",
            Category::Security => "Report a security concern",
            Category::Other => "Something else",
        }
    }

    /// The questions that fit it. Only the first needs an answer.
    pub fn questions(self) -> &'static [Question] {
        match self {
            Category::Problem => &[
                Question {
                    asked: "What were you doing, and what did you hear or see?",
                    prompt: "The steps in order, and what your screen reader said, if anything.",
                },
                Question {
                    asked: "What did you expect instead?",
                    prompt: "What should have happened. You can leave this empty.",
                },
            ],
            Category::Feature => &[Question {
                asked: "What would you like Wixen Mail to do?",
                prompt: "Say what you would use it for, so we can decide how it should work.",
            }],
            Category::ScreenReaderBarrier => &[Question {
                asked: "What is hard to use, and what does your screen reader say?",
                prompt: "Name the window or control, and the keys you pressed.",
            }],
            Category::Question => &[Question {
                asked: "What would you like to know?",
                prompt: "Ask in your own words.",
            }],
            Category::Security => &[Question {
                asked: "What is the concern, and how could somebody see it happen?",
                prompt: "This goes to the security address only. The subject line says \
                         nothing about it.",
            }],
            Category::Other => &[Question {
                asked: "What would you like to tell us?",
                prompt: "Anything that did not fit the other choices.",
            }],
        }
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
        match self {
            Fact::Version => "The version of Wixen Mail you are running",
            Fact::Windows => "Your Windows version and display language",
            Fact::ScreenReader => "Which screen reader is running, and its version",
            Fact::LogExcerpt => {
                "The last 200 lines of the program's log, with addresses and subjects hidden"
            }
            Fact::Providers => "Which kinds of mail account you have, without their addresses",
        }
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
        Include {
            version: true,
            windows: false,
            screen_reader: false,
            log_excerpt: category != Category::Security,
            providers: false,
        }
    }

    /// Whether one fact is ticked.
    pub fn includes(&self, fact: Fact) -> bool {
        match fact {
            Fact::Version => self.version,
            Fact::Windows => self.windows,
            Fact::ScreenReader => self.screen_reader,
            Fact::LogExcerpt => self.log_excerpt,
            Fact::Providers => self.providers,
        }
    }

    /// The same, with one fact ticked or not.
    pub fn with(mut self, fact: Fact, ticked: bool) -> Include {
        let field = match fact {
            Fact::Version => &mut self.version,
            Fact::Windows => &mut self.windows,
            Fact::ScreenReader => &mut self.screen_reader,
            Fact::LogExcerpt => &mut self.log_excerpt,
            Fact::Providers => &mut self.providers,
        };
        *field = ticked;
        self
    }
}

/// Where a report goes.
pub fn where_it_goes(category: Category) -> &'static str {
    match category {
        Category::Security => SECURITY_ADDRESS,
        _ => SUPPORT_ADDRESS,
    }
}

/// Which GitHub page is offered beside Send.
pub fn github_page(category: Category) -> &'static str {
    match category {
        Category::Security => PRIVATE_REPORTING_PAGE,
        _ => ISSUE_PAGE,
    }
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

/// How many characters of the first answer a subject carries.
const SUBJECT_CHARACTERS: usize = 80;

/// The message a report becomes, with no I/O.
///
/// The body is the answers under their questions, then each ticked fact on a
/// line of its own, then the reply line when one was given, then the line
/// naming the attachment when there is one, then one closing line naming
/// where the report went.
pub fn compose(report: &Report, facts: &Facts) -> Composed {
    let to = where_it_goes(report.category);
    let attachment = excerpt(report);
    let mut parts = vec![answered(report)];
    let carried = facts_carried(report.include, facts);
    if !carried.is_empty() {
        parts.push(carried.join("\n"));
    }
    let reply_to = report.reply_to.trim();
    if !reply_to.is_empty() {
        parts.push(format!("Reply to: {reply_to}"));
    }
    if let (Some(attached), Some(log)) = (&attachment, &report.log) {
        parts.push(format!(
            "Attached: {}, the end of this computer's log for {}, with addresses and \
             subjects hidden.",
            attached.file_name, log.date
        ));
    }
    parts.push(format!(
        "Sent with Wixen Mail's Send Feedback to {to}. A copy of this report is kept on \
         the sender's computer."
    ));
    Composed {
        to,
        subject: subject(report),
        body: parts.join("\n\n") + "\n",
        attachment,
    }
}

/// The subject: the category, and for every category but a security concern
/// the first line of the first answer.
fn subject(report: &Report) -> String {
    let label = report.category.label();
    let first_line = report
        .answers
        .first()
        .and_then(|answer| answer.lines().map(str::trim).find(|line| !line.is_empty()))
        .unwrap_or_default();
    match report.category {
        // A subject is shown in message lists and notifications, where a
        // body is read only when opened.
        Category::Security => format!("[Wixen Mail] {label}"),
        _ if first_line.is_empty() => format!("[Wixen Mail] {label}"),
        _ => {
            let bounded: String = first_line.chars().take(SUBJECT_CHARACTERS).collect();
            format!("[Wixen Mail] {label}: {bounded}")
        }
    }
}

/// Each question with its answer under it.
fn answered(report: &Report) -> String {
    report
        .category
        .questions()
        .iter()
        .enumerate()
        .map(|(at, question)| {
            let answer = report
                .answers
                .get(at)
                .map(|answer| answer.trim())
                .filter(|answer| !answer.is_empty())
                .unwrap_or("(no answer)");
            format!("{}\n{answer}", question.asked)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// One line per ticked fact, the log excerpt aside, which is an attachment.
fn facts_carried(include: Include, facts: &Facts) -> Vec<String> {
    Fact::ALL
        .into_iter()
        .filter(|fact| include.includes(*fact))
        .filter_map(|fact| match fact {
            Fact::Version => Some(format!("Version: {}", facts.version)),
            Fact::Windows => Some(format!(
                "Windows: {}, display language {}",
                facts.windows_build, facts.display_language
            )),
            Fact::ScreenReader => Some(match &facts.screen_reader {
                Some((name, version)) => format!("Screen reader: {name} {version}"),
                None => "Screen reader: none found running".to_string(),
            }),
            Fact::Providers => Some(match facts.providers.is_empty() {
                true => "Mail accounts: none set up".to_string(),
                false => format!("Mail accounts: {}", facts.providers.join(", ")),
            }),
            Fact::LogExcerpt => None,
        })
        .collect()
}

/// The end of the log, redacted, when the excerpt is ticked and there is a log.
fn excerpt(report: &Report) -> Option<Attachment> {
    if !report.include.log_excerpt {
        return None;
    }
    let log = report.log.as_ref()?;
    let lines = redact(&last_lines(&log.text, EXCERPT_LINES));
    let header = format!(
        "The last {} lines of {}, the log for {}, with addresses and subjects hidden.",
        lines.len(),
        log.file_name,
        log.date
    );
    Some(Attachment {
        file_name: format!("{}-log-excerpt.txt", report.stamp),
        text: std::iter::once(header)
            .chain(lines)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    })
}

/// The lines of a log with every address masked and every subject removed.
///
/// The boundary between the log and the outside, so it errs towards masking:
/// anything shaped `local@host` is treated as an address, and everything on a
/// line after a subject marker goes, whatever else was written after it.
pub fn redact(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| without_addresses(&without_subject(line)))
        .collect()
}

/// What a subject is replaced with.
const SUBJECT_REDACTED: &str = "[subject redacted]";

/// A line cut after the first subject marker, in any case.
fn without_subject(line: &str) -> String {
    // Lowercasing ASCII keeps every byte where it was, so a position found in
    // the lowered line is a position in the line.
    let lowered = line.to_ascii_lowercase();
    let cut = [("subject:", " "), ("subject=", "")]
        .into_iter()
        .filter_map(|(marker, gap)| lowered.find(marker).map(|at| (at + marker.len(), gap)))
        .min_by_key(|(end, _)| *end);
    match cut {
        Some((end, gap)) => format!("{}{gap}{SUBJECT_REDACTED}", &line[..end]),
        None => line.to_string(),
    }
}

fn is_local_part(c: char) -> bool {
    c.is_alphanumeric() || "._%+-'".contains(c)
}

fn is_host_part(c: char) -> bool {
    c.is_alphanumeric() || ".-".contains(c)
}

/// A line with every `local@host` masked by the rule the log already uses.
fn without_addresses(line: &str) -> String {
    let mut masked = String::with_capacity(line.len());
    let mut copied_to = 0;
    for (at, _) in line.match_indices('@') {
        if at < copied_to {
            continue;
        }
        let start = line[copied_to..at]
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_local_part(*c))
            .last()
            .map_or(at, |(offset, _)| copied_to + offset);
        let after = &line[at + 1..];
        let end = at
            + 1
            + after
                .char_indices()
                .find(|(_, c)| !is_host_part(*c))
                .map_or(after.len(), |(offset, _)| offset);
        if start == at || end == at + 1 {
            continue;
        }
        masked.push_str(&line[copied_to..start]);
        masked.push_str(&mask_email(&line[start..end]));
        copied_to = end;
    }
    masked.push_str(&line[copied_to..]);
    masked
}

/// The last `n` lines of a text.
pub fn last_lines(text: &str, n: usize) -> Vec<String> {
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(n)..]
        .iter()
        .map(|line| line.to_string())
        .collect()
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

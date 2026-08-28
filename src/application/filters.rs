//! Filter engine
//!
//! Rule-based message filtering and organization.

use crate::common::Result;
use crate::data::message_cache::{CachedMessage, MessageFilterRule};
use regex::RegexBuilder;

/// Filter action types
#[derive(Debug, Clone)]
pub enum FilterAction {
    MoveToFolder(String),
    AddTag(String),
    MarkAsRead,
    MarkAsUnread,
    Star,
    Unstar,
    Delete,
}

/// Message filter rule
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub id: String,
    pub name: String,
    /// Message field to evaluate ("subject", "from", or "to")
    pub field: String,
    /// Match type ("contains", "equals", "starts_with", "regex", "is_true", etc.)
    pub match_type: String,
    /// Case-insensitive "contains" match text
    pub pattern: String,
    /// Whether match should be case-sensitive for string comparisons
    pub case_sensitive: bool,
    pub action: FilterAction,
    pub enabled: bool,
}

/// Every field a rule may name, so a caller can ask before it runs.
///
/// A rule naming anything else cannot be evaluated, and what the engine
/// answers about a message is no, because that is the only safe answer: a
/// "not contains" reading as true would fire on the whole mailbox.
///
/// Safe is not the same as legible. No is also what a field this does know
/// says about a message that does not match, so nothing downstream can tell a
/// rule that found nothing from one this build cannot read at all. A saved
/// search naming a misspelled field then reads as an empty folder, which is
/// the failure that never gets reported: somebody believes they have no mail
/// from their manager.
///
/// So the list is written down and a test below requires it to hold exactly
/// the names the reading handles, in both directions. A name added to one and
/// not the other is the shape this exists to stop.
pub const A_FIELD_A_RULE_MAY_NAME: [&str; 11] = [
    "subject",
    "from",
    "to",
    "cc",
    "date",
    "message_id",
    "body_plain",
    "body_html",
    "read",
    "starred",
    "deleted",
];

/// Whether a rule naming this field is one this build can evaluate.
pub fn a_rule_may_name(field: &str) -> bool {
    A_FIELD_A_RULE_MAY_NAME.contains(&field)
}

/// Filter engine for automatic message processing
#[derive(Default)]
pub struct FilterEngine {
    rules: Vec<FilterRule>,
}

impl FilterEngine {
    /// Create a new filter engine
    pub fn new() -> Result<Self> {
        Ok(Self { rules: Vec::new() })
    }

    /// Add a filter rule
    pub fn add_rule(&mut self, rule: FilterRule) -> Result<()> {
        self.rules.push(rule);
        Ok(())
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[FilterRule] {
        &self.rules
    }

    /// Evaluate all enabled rules against a message and return matched actions
    pub fn evaluate_message(&self, message: &CachedMessage) -> Vec<FilterAction> {
        self.rules
            .iter()
            .filter(|rule| rule.enabled && Self::matches(rule, message))
            .map(|rule| rule.action.clone())
            .collect()
    }

    /// Convert persisted rules into runtime rules for execution
    pub fn load_from_persisted(&mut self, rules: &[MessageFilterRule]) {
        self.rules = rules.iter().filter_map(Self::from_persisted_rule).collect();
    }

    pub fn matches(rule: &FilterRule, message: &CachedMessage) -> bool {
        fn bool_to_str(value: bool) -> &'static str {
            if value { "true" } else { "false" }
        }

        // An unknown field is a rule this version cannot evaluate, and the
        // only safe answer is no. Anything else, in particular a "not
        // contains" reading as true, would turn a rule naming a field we do
        // not have into one that fires on the whole mailbox.
        //
        // Safe, and not the whole answer: no is also what a field this does
        // know says about a message that does not match, so a caller cannot
        // tell "nothing matched" from "this cannot be evaluated at all". A
        // saved search naming a field this build has never heard of reads as
        // an empty folder, which is the wrong thing in the one way that never
        // gets reported. [`A_FIELD_A_RULE_MAY_NAME`] is how a caller asks
        // first, and there is a test below that it and this list agree.
        let known_field = match rule.field.as_str() {
            "subject" => Some(Some(message.subject.as_str())),
            "from" => Some(Some(message.from_addr.as_str())),
            "to" => Some(Some(message.to_addr.as_str())),
            "cc" => Some(message.cc.as_deref()),
            "date" => Some(Some(message.date.as_str())),
            "message_id" => Some(Some(message.message_id.as_str())),
            "body_plain" => Some(message.body_plain.as_deref()),
            "body_html" => Some(message.body_html.as_deref()),
            "read" => Some(Some(bool_to_str(message.read))),
            "starred" => Some(Some(bool_to_str(message.starred))),
            "deleted" => Some(Some(bool_to_str(message.deleted))),
            _ => None,
        };
        let Some(present) = known_field else {
            return false;
        };
        // An absent field is an empty one, not a reason to stop.
        //
        // Leaving early here meant "cc is empty" was false for every message
        // that had no cc, which is the only case the rule was written for, and
        // "body is empty" could never fire on a message whose body had not
        // been downloaded. Both are the exact situations someone writes such a
        // rule to catch.
        let target_text = present.unwrap_or("");

        let lhs = if rule.case_sensitive {
            target_text.to_string()
        } else {
            target_text.to_lowercase()
        };
        let rhs = if rule.case_sensitive {
            rule.pattern.clone()
        } else {
            rule.pattern.to_lowercase()
        };

        match rule.match_type.as_str() {
            "contains" => lhs.contains(&rhs),
            "not_contains" => !lhs.contains(&rhs),
            "equals" => lhs == rhs,
            "not_equals" => lhs != rhs,
            "starts_with" => lhs.starts_with(&rhs),
            "ends_with" => lhs.ends_with(&rhs),
            "is_empty" => lhs.trim().is_empty(),
            "is_not_empty" => !lhs.trim().is_empty(),
            "is_true" => lhs == "true",
            "is_false" => lhs == "false",
            // Built from the rule's own pattern rather than the pre-lowercased
            // `rhs`, because lowercasing a pattern would corrupt escapes like
            // \S and \W. Case folding belongs to the regex engine.
            // Bounded, because a rule can be imported rather than typed, and
            // a pattern that compiles to something enormous would take the
            // window down with it. A megabyte is far more than any real rule
            // and far less than enough to hurt.
            "regex" => match RegexBuilder::new(&rule.pattern)
                .case_insensitive(!rule.case_sensitive)
                .size_limit(1 << 20)
                .build()
            {
                Ok(regex) => regex.is_match(target_text),
                Err(e) => {
                    tracing::warn!(
                        "Invalid regex pattern '{}' in rule '{}': {}",
                        rule.pattern,
                        rule.name,
                        e
                    );
                    false
                }
            },
            _ => false,
        }
    }

    pub fn from_persisted_rule(rule: &MessageFilterRule) -> Option<FilterRule> {
        let action = match rule.action_type.as_str() {
            "move_to_folder" => FilterAction::MoveToFolder(Self::validated_action_value(
                rule.action_value.as_ref(),
            )?),
            "add_tag" => {
                FilterAction::AddTag(Self::validated_action_value(rule.action_value.as_ref())?)
            }
            "mark_as_read" => FilterAction::MarkAsRead,
            "mark_as_unread" => FilterAction::MarkAsUnread,
            "star" => FilterAction::Star,
            "unstar" => FilterAction::Unstar,
            "delete" => FilterAction::Delete,
            _ => return None,
        };

        Some(FilterRule {
            id: rule.id.clone(),
            name: rule.name.clone(),
            field: rule.field.clone(),
            match_type: rule.match_type.clone(),
            pattern: rule.pattern.clone(),
            case_sensitive: rule.case_sensitive,
            action,
            enabled: rule.enabled,
        })
    }

    pub fn validated_action_value(value: Option<&String>) -> Option<String> {
        let value = value?.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }
}

/// What a message's matching rules add up to.
///
/// Rules are written one at a time and a message can match several, so the
/// actions arrive as a list that can contradict itself: one rule says read,
/// the next says unread, a third says move it and a fourth says delete it.
/// Carrying them out in order would do all four, leaving the message wherever
/// the last write happened to land it.
///
/// Settled into one answer first, so what happens to a message is decided
/// before anything is written and can be described before it is done.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Whether to mark it read, if any rule said so.
    pub read: Option<bool>,
    /// Whether to flag it, if any rule said so.
    pub starred: Option<bool>,
    /// Where it goes, if any rule moves it.
    pub move_to: Option<String>,
    /// What to label it, in the order the rules appear, without repeats.
    pub tags: Vec<String>,
    /// Whether it goes to the trash.
    pub delete: bool,
}

impl Outcome {
    /// Whether this does anything at all.
    pub fn is_nothing(&self) -> bool {
        self == &Self::default()
    }

    /// Whether carrying this out changes anything on the server.
    ///
    /// Moving and deleting do. Marking read and flagging are written back by
    /// the flag sync, which has its own gate, so they are not counted here.
    pub fn touches_the_server(&self) -> bool {
        self.delete || self.move_to.is_some()
    }
}

/// Settle a message's matching actions into one answer.
///
/// Later rules win over earlier ones on the questions that have a single
/// answer, because that is the order they are written in and the order somebody
/// reading their own rule list expects.
///
/// Deleting wins outright and drops the rest. Moving a message and then
/// deleting it is two writes to reach the same place, and moving a message that
/// is about to be deleted puts it in the trash from a folder somebody never saw
/// it in.
pub fn settle(actions: &[FilterAction]) -> Outcome {
    let mut outcome = Outcome::default();
    for action in actions {
        match action {
            FilterAction::Delete => {
                return Outcome {
                    delete: true,
                    ..Outcome::default()
                };
            }
            FilterAction::MarkAsRead => outcome.read = Some(true),
            FilterAction::MarkAsUnread => outcome.read = Some(false),
            FilterAction::Star => outcome.starred = Some(true),
            FilterAction::Unstar => outcome.starred = Some(false),
            FilterAction::MoveToFolder(folder) => outcome.move_to = Some(folder.clone()),
            FilterAction::AddTag(tag) => {
                if !outcome.tags.iter().any(|held| held == tag) {
                    outcome.tags.push(tag.clone());
                }
            }
        }
    }
    outcome
}

#[cfg(test)]
mod tests {

    /// A rule naming a specific field, for the absent-field cases.
    fn field_rule(field: &str, match_type: &str, pattern: &str) -> FilterRule {
        FilterRule {
            id: "r1".to_string(),
            name: "test".to_string(),
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: pattern.to_string(),
            case_sensitive: false,
            action: FilterAction::MarkAsRead,
            enabled: true,
        }
    }

    #[test]
    fn test_an_absent_field_counts_as_empty() {
        // "cc is empty" on a message with no cc has to be true. It returned
        // false, because an absent field left the matcher before it reached the
        // match type at all, so a rule for "no cc" could never fire and a rule
        // for "not empty" was right by accident.
        let message = super::tests::message_with_subject("Anything");
        assert!(message.cc.is_none());
        assert!(
            FilterEngine::matches(&field_rule("cc", "is_empty", ""), &message),
            "a message with no cc did not count as having an empty cc"
        );
        assert!(!FilterEngine::matches(
            &field_rule("cc", "is_not_empty", ""),
            &message
        ));
    }

    #[test]
    fn test_an_absent_body_counts_as_empty_too() {
        // Same shape, and it matters more: a message whose body has not been
        // downloaded is exactly the case a "body is empty" rule is written for.
        let message = super::tests::message_with_subject("Anything");
        assert!(FilterEngine::matches(
            &field_rule("body_plain", "is_empty", ""),
            &message
        ));
        assert!(FilterEngine::matches(
            &field_rule("body_html", "is_empty", ""),
            &message
        ));
    }

    #[test]
    fn test_an_absent_field_still_fails_the_content_match_types() {
        // Absent is empty, not a match for everything. A "contains" rule must
        // not start firing on every message with no cc.
        let message = super::tests::message_with_subject("Anything");
        for match_type in ["contains", "equals", "starts_with", "ends_with"] {
            assert!(
                !FilterEngine::matches(&field_rule("cc", match_type, "anything"), &message),
                "{} matched an absent field",
                match_type
            );
        }
        // And "not contains" is true of an absent field, because it does not
        // contain it.
        assert!(FilterEngine::matches(
            &field_rule("cc", "not_contains", "anything"),
            &message
        ));
    }

    #[test]
    fn test_an_unknown_field_matches_nothing_rather_than_everything() {
        // A rule naming a field this version does not know must not silently
        // become "true for all mail" and start moving the whole inbox.
        let message = super::tests::message_with_subject("Anything");
        for match_type in ["contains", "is_empty", "not_contains", "not_equals"] {
            assert!(
                !FilterEngine::matches(&field_rule("invented_field", match_type, ""), &message),
                "{} fired on a field that does not exist",
                match_type
            );
        }
    }

    #[test]
    fn test_a_pattern_too_large_to_compile_is_refused_rather_than_hanging() {
        // A rule can be imported, so the pattern is not always something the
        // user typed. A compile that eats memory would take the window with it.
        let message = super::tests::message_with_subject("Anything");
        let huge = format!("(?:{}){{100}}", "a|b|c|d|e|f|g|h".repeat(200));
        assert!(!FilterEngine::matches(
            &field_rule("subject", "regex", &huge),
            &message
        ));
    }
    use super::*;

    #[test]
    fn test_filter_engine_creation() {
        let engine = FilterEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_filter_engine_evaluates_message() {
        let mut engine = FilterEngine::new().unwrap();
        engine
            .add_rule(FilterRule {
                id: "r1".to_string(),
                name: "Mark newsletter as read".to_string(),
                field: "subject".to_string(),
                match_type: "contains".to_string(),
                pattern: "newsletter".to_string(),
                case_sensitive: false,
                action: FilterAction::MarkAsRead,
                enabled: true,
            })
            .unwrap();

        let message = CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".to_string(),
            subject: "Weekly Newsletter".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };

        let actions = engine.evaluate_message(&message);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], FilterAction::MarkAsRead));
    }

    #[test]
    fn test_filter_match_types() {
        let mut engine = FilterEngine::new().unwrap();
        engine
            .add_rule(FilterRule {
                id: "r2".to_string(),
                name: "Starts with Re".to_string(),
                field: "subject".to_string(),
                match_type: "starts_with".to_string(),
                pattern: "Re:".to_string(),
                case_sensitive: true,
                action: FilterAction::Star,
                enabled: true,
            })
            .unwrap();

        let message = CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".to_string(),
            subject: "Re: Project Update".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: Some("Update".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };

        let actions = engine.evaluate_message(&message);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], FilterAction::Star));
    }

    // ── Rule matching ───────────────────────────────────────────────────

    fn rule(match_type: &str, pattern: &str, case_sensitive: bool) -> FilterRule {
        FilterRule {
            id: "r1".into(),
            name: "test rule".into(),
            field: "subject".into(),
            match_type: match_type.into(),
            pattern: pattern.into(),
            case_sensitive,
            action: FilterAction::Star,
            enabled: true,
        }
    }

    pub(super) fn message_with_subject(subject: &str) -> CachedMessage {
        CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: String::new(),
            subject: subject.into(),
            from_addr: "sender@example.com".into(),
            to_addr: "me@example.com".into(),
            cc: None,
            date: String::new(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    #[test]
    fn test_case_insensitive_regex_actually_ignores_case() {
        // A rule marked case-insensitive has to behave that way for every
        // match type, or the checkbox is lying to the user.
        let matched = FilterEngine::matches(
            &rule("regex", "URGENT", false),
            &message_with_subject("this is urgent"),
        );
        assert!(matched, "case-insensitive regex rule did not ignore case");
    }

    #[test]
    fn test_case_sensitive_regex_respects_case() {
        assert!(!FilterEngine::matches(
            &rule("regex", "URGENT", true),
            &message_with_subject("this is urgent")
        ));
        assert!(FilterEngine::matches(
            &rule("regex", "URGENT", true),
            &message_with_subject("this is URGENT")
        ));
    }

    #[test]
    fn test_case_insensitive_contains_ignores_case() {
        assert!(FilterEngine::matches(
            &rule("contains", "URGENT", false),
            &message_with_subject("this is urgent")
        ));
    }

    #[test]
    fn test_invalid_regex_does_not_match_and_does_not_panic() {
        assert!(!FilterEngine::matches(
            &rule("regex", "([unclosed", false),
            &message_with_subject("anything")
        ));
    }

    #[test]
    fn test_unknown_match_type_never_matches() {
        assert!(!FilterEngine::matches(
            &rule("wishful_thinking", "x", false),
            &message_with_subject("x")
        ));
    }

    #[test]
    fn test_regex_matching_a_hostile_subject_terminates() {
        // The regex crate is linear time by construction, so this is a
        // regression guard rather than a fix: if anyone swaps in a
        // backtracking engine, this is where it shows up.
        let subject = "a".repeat(5_000);
        assert!(FilterEngine::matches(
            &rule("regex", "(a+)+$", false),
            &message_with_subject(&subject)
        ));
    }

    #[test]
    fn test_every_stored_action_becomes_the_action_it_names() {
        // The other half of a rule: what it does. Found by mutation testing,
        // which deleted the arms for moving, labelling, marking unread,
        // flagging and unflagging without a single test noticing. A rule that
        // matched correctly and then did nothing would have looked fine here.
        use crate::data::message_cache::MessageFilterRule;

        let stored = |action_type: &str, value: Option<&str>| MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "A rule".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: action_type.into(),
            action_value: value.map(str::to_string),
            enabled: true,
            created_at: "2026-08-01T00:00:00Z".into(),
        };

        let read = |action_type: &str, value: Option<&str>| {
            FilterEngine::from_persisted_rule(&stored(action_type, value)).map(|r| r.action)
        };

        assert!(matches!(
            read("move_to_folder", Some("Receipts")),
            Some(FilterAction::MoveToFolder(ref to)) if to == "Receipts"
        ));
        assert!(matches!(
            read("add_tag", Some("Work")),
            Some(FilterAction::AddTag(ref name)) if name == "Work"
        ));
        assert!(matches!(
            read("mark_as_read", None),
            Some(FilterAction::MarkAsRead)
        ));
        assert!(matches!(
            read("mark_as_unread", None),
            Some(FilterAction::MarkAsUnread)
        ));
        assert!(matches!(read("star", None), Some(FilterAction::Star)));
        assert!(matches!(read("unstar", None), Some(FilterAction::Unstar)));
        assert!(matches!(read("delete", None), Some(FilterAction::Delete)));
        // A rule this version does not understand is dropped rather than
        // guessed at. Guessing would carry out something nobody asked for on
        // somebody's mail.
        assert!(read("teleport", None).is_none());
    }

    #[test]
    fn test_an_action_that_needs_a_value_and_has_none_is_dropped() {
        // Moving to a folder with no folder named, or labelling with no label.
        // Both would otherwise become an action carried out against an empty
        // name, which on a move means a folder called nothing.
        use crate::data::message_cache::MessageFilterRule;

        let empty = |action_type: &str, value: Option<&str>| MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "A rule".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: action_type.into(),
            action_value: value.map(str::to_string),
            enabled: true,
            created_at: "2026-08-01T00:00:00Z".into(),
        };

        for action_type in ["move_to_folder", "add_tag"] {
            for value in [None, Some(""), Some("   ")] {
                assert!(
                    FilterEngine::from_persisted_rule(&empty(action_type, value)).is_none(),
                    "{action_type} with {value:?} was accepted"
                );
            }
        }
        assert_eq!(
            FilterEngine::validated_action_value(Some(&" Receipts ".to_string())).as_deref(),
            Some("Receipts"),
            "a name with spaces around it was not tidied"
        );
    }

    #[test]
    fn test_every_way_of_matching_says_yes_when_it_should() {
        // Found by mutation testing: only "contains", "starts_with",
        // "is_empty" and "regex" were ever exercised. Deleting the arm for
        // "equals", "not_equals", "ends_with", "is_not_empty", "is_true" or
        // "is_false" changed nothing any test could see, so six of the eleven
        // ways a rule can be written had never been run.
        let message = message_with_subject("Quarterly report");

        for (match_type, pattern) in [
            ("contains", "Quarterly"),
            ("not_contains", "Annual"),
            ("equals", "Quarterly report"),
            ("not_equals", "Something else"),
            ("starts_with", "Quarterly"),
            ("ends_with", "report"),
            ("is_not_empty", ""),
            ("regex", "^Quarterly"),
        ] {
            assert!(
                FilterEngine::matches(&rule(match_type, pattern, false), &message),
                "{match_type} looking for {pattern:?} did not match"
            );
        }
    }

    #[test]
    fn test_every_way_of_matching_says_no_when_it_should() {
        // The other half. Without it an arm that always answered yes would
        // pass the test above.
        let message = message_with_subject("Quarterly report");

        for (match_type, pattern) in [
            ("contains", "Annual"),
            ("not_contains", "Quarterly"),
            ("equals", "Quarterly"),
            ("not_equals", "Quarterly report"),
            ("starts_with", "report"),
            ("ends_with", "Quarterly"),
            ("is_empty", ""),
            ("regex", "^report"),
        ] {
            assert!(
                !FilterEngine::matches(&rule(match_type, pattern, false), &message),
                "{match_type} looking for {pattern:?} matched when it should not"
            );
        }
    }

    #[test]
    fn test_a_rule_can_ask_whether_a_flag_is_set() {
        // "is_true" and "is_false" are how a rule names a flag rather than
        // text: read, starred, deleted. Both arms were untested, so both would
        // have gone on answering the same way whichever flag was asked about.
        let mut message = message_with_subject("Quarterly report");
        message.read = true;
        message.starred = false;

        let asked = |field: &str, match_type: &str| {
            let mut named = rule(match_type, "", false);
            named.field = field.to_string();
            FilterEngine::matches(&named, &message)
        };

        assert!(
            asked("read", "is_true"),
            "a read message did not answer is_true"
        );
        assert!(!asked("read", "is_false"));
        assert!(
            asked("starred", "is_false"),
            "an unflagged message did not answer is_false"
        );
        assert!(!asked("starred", "is_true"));
    }

    #[test]
    fn test_the_engine_hands_back_the_rules_it_was_given() {
        // Found by mutation testing: nothing asked what get_rules returned, so
        // an engine reporting an empty list would have passed every test. It is
        // what the sync checks before deciding whether to look at arriving mail
        // at all, so an empty answer means rules that silently never run.
        let mut engine = FilterEngine::default();
        engine
            .add_rule(rule("contains", "Invoice", false))
            .expect("the rule is added");

        assert_eq!(engine.get_rules().len(), 1);
        assert_eq!(engine.get_rules()[0].pattern, "Invoice");
    }

    #[test]
    fn test_a_rule_can_name_any_field_the_engine_claims_to_know() {
        // Found by mutation testing: deleting the arms for "from", "to",
        // "date" and "message_id" changed nothing any test could see, so four
        // of the fields a rule may name had never been matched against. "From"
        // is the field most rules are written on.
        let mut message = message_with_subject("Quarterly report");
        message.from_addr = "billing@example.com".into();
        message.to_addr = "accounts@example.com".into();
        message.date = "2026-07-31T09:00:00+00:00".into();
        message.message_id = "inv-4021@example.com".into();
        message.cc = Some("audit@example.com".into());
        message.body_plain = Some("The invoice is attached.".into());

        for (field, pattern) in [
            ("subject", "Quarterly"),
            ("from", "billing@"),
            ("to", "accounts@"),
            ("cc", "audit@"),
            ("date", "2026-07-31"),
            ("message_id", "inv-4021"),
            ("body_plain", "invoice"),
            ("read", "false"),
            ("starred", "false"),
            ("deleted", "false"),
        ] {
            let mut named = rule("contains", pattern, false);
            named.field = field.into();
            assert!(
                FilterEngine::matches(&named, &message),
                "a rule on {field} looking for {pattern} did not match"
            );
        }
    }

    #[test]
    fn test_a_rule_on_one_field_does_not_match_another_fields_text() {
        // The other half. Without it, an arm reading the wrong field would
        // pass the test above whenever both happened to hold the pattern.
        let mut message = message_with_subject("Quarterly report");
        message.from_addr = "billing@example.com".into();

        let mut on_from = rule("contains", "Quarterly", false);
        on_from.field = "from".into();

        assert!(!FilterEngine::matches(&on_from, &message));
    }

    #[test]
    fn test_a_message_matching_nothing_has_nothing_done_to_it() {
        assert!(settle(&[]).is_nothing());
    }

    #[test]
    fn test_the_later_rule_wins_when_two_disagree() {
        // Written in order, read in order. Carrying both out would leave the
        // message in whichever state the last write happened to land in, which
        // is the same answer by accident rather than on purpose.
        let settled = settle(&[FilterAction::MarkAsRead, FilterAction::MarkAsUnread]);

        assert_eq!(settled.read, Some(false));
    }

    #[test]
    fn test_deleting_drops_everything_else() {
        // Moving a message that is about to be deleted puts it in the trash
        // from a folder nobody ever saw it in.
        let settled = settle(&[
            FilterAction::MoveToFolder("Receipts".into()),
            FilterAction::Star,
            FilterAction::Delete,
        ]);

        assert!(settled.delete);
        assert_eq!(settled.move_to, None);
        assert_eq!(settled.starred, None);
    }

    #[test]
    fn test_nothing_written_after_a_delete_brings_the_message_back() {
        let settled = settle(&[FilterAction::Delete, FilterAction::MarkAsRead]);

        assert!(settled.delete);
        assert_eq!(settled.read, None);
    }

    #[test]
    fn test_tags_add_up_and_are_not_repeated() {
        let settled = settle(&[
            FilterAction::AddTag("work".into()),
            FilterAction::AddTag("urgent".into()),
            FilterAction::AddTag("work".into()),
        ]);

        assert_eq!(settled.tags, vec!["work", "urgent"]);
    }

    #[test]
    fn test_only_moving_and_deleting_count_as_touching_the_server() {
        // What the permission gate asks. Marking read and flagging go through
        // the flag sync, which has its own.
        assert!(settle(&[FilterAction::Delete]).touches_the_server());
        assert!(settle(&[FilterAction::MoveToFolder("Receipts".into())]).touches_the_server());
        assert!(!settle(&[FilterAction::MarkAsRead]).touches_the_server());
        assert!(!settle(&[FilterAction::AddTag("work".into())]).touches_the_server());
    }

    #[test]
    fn test_anchored_regex_sees_the_whole_field() {
        assert!(FilterEngine::matches(
            &rule("regex", "^Invoice #\\d+$", true),
            &message_with_subject("Invoice #4021")
        ));
    }
}

#[cfg(test)]
mod the_fields_a_rule_may_name {
    use super::*;

    /// A rule asking whether a field is empty.
    ///
    /// The one match type that answers the same way for every field, so the
    /// answer says whether the field was read at all rather than anything
    /// about this particular message.
    fn asking_about(field: &str, match_type: &str) -> FilterRule {
        FilterRule {
            id: "asking".to_string(),
            name: "asking".to_string(),
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: String::new(),
            case_sensitive: false,
            action: FilterAction::MarkAsRead,
            enabled: true,
        }
    }

    #[test]
    fn test_every_field_the_list_names_is_one_the_reading_really_handles() {
        // The direction that matters most. A name on the list that the
        // reading does not handle is worse than one missing from it: a caller
        // asks first, is told the field is fine, runs the search, and gets an
        // empty folder. Somebody then believes they have no mail from their
        // manager.
        let message = super::tests::message_with_subject("Anything");

        for field in A_FIELD_A_RULE_MAY_NAME {
            let known = FilterEngine::matches(&asking_about(field, "is_not_empty"), &message)
                || FilterEngine::matches(&asking_about(field, "is_empty"), &message);
            assert!(
                known,
                "{field} is on the list of fields a rule may name and the reading answers no to \
                 both is_empty and is_not_empty, which is what it answers for a field it has \
                 never heard of"
            );
        }
    }

    #[test]
    fn test_a_field_the_reading_does_not_handle_is_not_on_the_list() {
        // The other direction, so the list cannot quietly grow past what the
        // reading does. Spelled the ways somebody really gets it wrong.
        for made_up in ["sender", "Subject", "body", "subj", "attachments", ""] {
            assert!(
                !a_rule_may_name(made_up),
                "{made_up:?} is named as a field a rule may use and the reading does not handle it"
            );
        }
    }

    #[test]
    fn test_asking_first_tells_the_two_answers_apart() {
        // The whole point. Without this a search naming a misspelled field and
        // a search that really found nothing are one answer.
        assert!(a_rule_may_name("from"));
        assert!(!a_rule_may_name("frmo"));
    }
}

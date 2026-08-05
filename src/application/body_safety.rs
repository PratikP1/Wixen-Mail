//! Reading the message itself to see whether it is what it says it is.
//!
//! The third of the three sources a verdict comes from, and the only one that
//! looks at what was written rather than at what somebody else concluded about
//! it. Filters are good at bulk and blunt about the message written to one
//! person, which is the one that costs somebody their bank account.
//!
//! # Why it is here rather than at each sync
//!
//! Because both syncs need it and they have to agree. Mail arriving over IMAP
//! has had this reading since the body fetch was written; mail arriving over
//! POP had none of it, so the same message got a warning on one account and
//! silence on the other. Two copies of a safety judgement that must agree is
//! the shape that has already cost this project a birthday, a website and the
//! login name in the From header of every message.
//!
//! # Nothing here leaves this computer
//!
//! No network, no file read, no account. It is string analysis over text that
//! is already in hand, which is what makes it safe to have on unless somebody
//! turns it off. The link check against Google's lists is a separate setting,
//! off unless asked for, and says separately what it sends.

use crate::service::safety::Verdict;

/// What the setting for this does, in the words for what happens.
///
/// Beside a box that does send something, so which is which has to be plain.
/// One sentence per idea, and what it costs last.
pub const LOOKING_AT_THE_MESSAGE_ITSELF: &str = "Wixen Mail reads each message on this computer and marks it when something \
     looks wrong: a link whose words and address disagree, an address made to \
     look like somebody else's, or pressure to act at once. Nothing is sent \
     anywhere and no account is needed. Turning it off means those messages \
     arrive with nothing said about them.";

/// What our own reading of a message makes of it.
///
/// `Verdict::ordinary()` where nothing was found, and never worse than
/// [`crate::service::safety::Safety::Suspicious`], which is a rule the analysis
/// holds rather than one applied here.
///
/// Both halves of the body are handed over, because a message can carry either
/// or both: text alone is the ordinary case for anything a person typed, and a
/// message with only a formatted part would otherwise have nothing read at all.
pub fn from_body(
    from: &str,
    subject: &str,
    body_plain: Option<&str>,
    body_html: Option<&str>,
) -> Verdict {
    // The key file this looks for is only on a computer upgraded from a version
    // that encrypted passwords, so most installations have none and the service
    // is built without one. It is not needed for reading a message, and failing
    // to build it must not mean failing to read: that would be a silent gap on
    // every fresh installation.
    let Ok(security) = crate::service::security::SecurityService::new() else {
        return Verdict::ordinary();
    };
    let Ok(report) =
        security.analyze_message_security(from, subject, body_plain.unwrap_or_default(), body_html)
    else {
        return Verdict::ordinary();
    };

    crate::service::safety::from_analysis(report.phishing_risk.into(), &report.phishing_indicators)
}

/// Add what was just found to what the stored message already says.
///
/// Merged rather than replaced, worst winning and every reason kept. A message
/// can be both in a filter's junk pile and carrying a link that lies about
/// where it goes, and the reader deserves both sentences.
///
/// Finding nothing writes nothing. Not listed is not the same as safe, so a
/// clean answer from one source must never talk over a warning from another.
pub fn merge_into(
    cache: &crate::data::message_cache::MessageCache,
    message_row_id: i64,
    found: &Verdict,
) -> crate::common::Result<()> {
    if found.level == crate::service::safety::Safety::Ordinary {
        return Ok(());
    }
    let merged = cache
        .message_safety(message_row_id)
        .unwrap_or_default()
        .and(found.clone());
    cache.set_message_safety(message_row_id, &merged)
}

#[cfg(test)]
mod tests {
    use super::{LOOKING_AT_THE_MESSAGE_ITSELF, from_body};
    use crate::service::safety::{Safety, Verdict};

    /// A message whose visible link text and actual address disagree, sent with
    /// the pressure that goes with one.
    fn a_message_pretending_to_be_the_bank() -> (&'static str, &'static str, &'static str) {
        (
            "Security <noreply@paypa1.example>",
            "Urgent: your account is suspended",
            "<p>Visit <a href=\"http://192.0.2.7/collect\">https://www.yourbank.example</a></p>",
        )
    }

    #[test]
    fn test_a_deceptive_link_is_marked_as_worth_looking_at() {
        let (from, subject, html) = a_message_pretending_to_be_the_bank();

        let verdict = from_body(from, subject, Some("Visit your bank."), Some(html));

        assert_eq!(verdict.level, Safety::Suspicious);
        assert!(!verdict.reasons.is_empty(), "marked with no reason given");
    }

    #[test]
    fn test_an_ordinary_message_is_left_alone() {
        // The other half, and the one that decides whether the marking is worth
        // having. An inbox where every row announces a warning is an inbox
        // where nobody hears the one that mattered.
        let verdict = from_body(
            "Ada Lovelace <ada@example.com>",
            "Notes on the engine",
            Some("The numbers are in the attachment. See you Thursday."),
            None,
        );

        assert_eq!(verdict.level, Safety::Ordinary);
        assert!(verdict.reasons.is_empty());
    }

    #[test]
    fn test_a_message_with_no_body_at_all_is_left_alone() {
        // Both empty, which is what a message whose body has not been stored
        // looks like. Marking one suspicious for having nothing in it would put
        // a warning on every message this computer has not read yet.
        assert_eq!(
            from_body("ada@example.com", "Notes", None, None).level,
            Safety::Ordinary
        );
        assert_eq!(
            from_body("ada@example.com", "Notes", Some(""), Some("")).level,
            Safety::Ordinary
        );
    }

    #[test]
    fn test_the_plain_half_alone_is_enough_to_be_read() {
        // A message with no formatted part is the ordinary case for anything a
        // person typed, and it is exactly the shape a targeted message takes.
        let verdict = from_body(
            "noreply@example.com",
            "Urgent: immediate action required",
            Some("Reply to this email with your details. http://192.0.2.7/collect"),
            None,
        );

        assert_eq!(verdict.level, Safety::Suspicious);
    }

    #[test]
    fn test_the_formatted_half_alone_is_enough_to_be_read() {
        // The other way round, and the one that used to be missed: a message
        // carrying only a formatted part had nothing read at all if the plain
        // half was what was looked at.
        let (from, subject, html) = a_message_pretending_to_be_the_bank();

        assert_eq!(
            from_body(from, subject, None, Some(html)).level,
            Safety::Suspicious
        );
    }

    #[test]
    fn test_the_two_halves_saying_the_same_thing_reads_the_same_as_one() {
        // Some senders put identical text in both. Reading it twice must not
        // make a message look worse than the same message sent once.
        let text = "Reply to this email. http://192.0.2.7/collect";
        let once = from_body("noreply@example.com", "Notes", Some(text), None);
        let twice = from_body("noreply@example.com", "Notes", Some(text), Some(text));

        assert_eq!(once.level, twice.level);
        assert_eq!(once.reasons, twice.reasons);
    }

    #[test]
    fn test_a_message_from_nobody_is_still_read() {
        // A message with no From at all is a message this cannot ask about the
        // sender, not a reason to skip the rest of the reading.
        let (_, subject, html) = a_message_pretending_to_be_the_bank();

        assert_eq!(
            from_body("", subject, Some(""), Some(html)).level,
            Safety::Suspicious
        );
    }

    #[test]
    fn test_a_message_with_no_subject_is_still_read() {
        // An empty subject is a message with nothing in that field, not a
        // reason to stop reading the rest of it.
        let (from, _, html) = a_message_pretending_to_be_the_bank();

        assert_eq!(
            from_body(
                from,
                "",
                Some("Urgent: your account is suspended. Visit your bank."),
                Some(html)
            )
            .level,
            Safety::Suspicious
        );
    }

    #[test]
    fn test_a_verdict_from_the_body_is_never_worse_than_suspicious() {
        // The existing guard against crying wolf, restated here because this is
        // now the only way in. Calling something a phishing attempt on a score
        // is how a warning becomes the thing people click past; the provider's
        // filter and the sender's own authentication are the two allowed to use
        // that word.
        let verdict = from_body(
            "Security <noreply@paypa1.example>",
            "Urgent: verify your account, your password expires, wire transfer",
            Some("Reply to this email. http://192.0.2.7/xn--collect"),
            Some("<a href=\"http://192.0.2.7/x\">https://yourbank.example</a>"),
        );

        assert!(verdict.level <= Safety::Suspicious, "{:?}", verdict.level);
    }

    #[test]
    fn test_something_found_later_is_added_to_what_was_already_known() {
        // The verdict is three sources merged, and this is how a fourth one
        // arriving after the message was stored joins them. Replacing instead
        // would throw away what a provider's own filter said.
        let cache = a_cache("adds_to_what_was_known");
        let (folder, row) = a_stored_message(&cache, Safety::Spam, "A filter said so.");

        super::merge_into(
            &cache,
            row,
            &Verdict {
                level: Safety::Suspicious,
                reasons: vec!["A link goes somewhere else.".to_string()],
            },
        )
        .expect("the merge runs");

        let stored = cache.message_safety(row).expect("the verdict");
        assert_eq!(stored.level, Safety::Spam, "the worse of the two lost");
        assert_eq!(
            stored.reasons.len(),
            2,
            "a reason was thrown away: {:?}",
            stored.reasons
        );
        let _ = folder;
    }

    #[test]
    fn test_finding_nothing_does_not_clear_a_warning_already_there() {
        // Not listed is not the same as safe. A clean answer from one source
        // must not talk over a filter that said this was junk.
        let cache = a_cache("keeps_what_was_known");
        let (_, row) = a_stored_message(&cache, Safety::Spam, "A filter said so.");

        super::merge_into(&cache, row, &Verdict::ordinary()).expect("the merge runs");

        let stored = cache.message_safety(row).expect("the verdict");
        assert_eq!(stored.level, Safety::Spam);
        assert_eq!(stored.reasons, vec!["A filter said so.".to_string()]);
    }

    /// A cache in a directory of its own.
    fn a_cache(what_for: &str) -> crate::data::message_cache::MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock that has passed 1970")
            .as_nanos();
        crate::data::message_cache::MessageCache::new(
            std::env::temp_dir().join(format!("wixen_body_safety_{what_for}_{nanos}")),
            None,
        )
        .expect("a cache to open")
    }

    /// One stored message already carrying a verdict.
    fn a_stored_message(
        cache: &crate::data::message_cache::MessageCache,
        level: Safety,
        reason: &str,
    ) -> (i64, i64) {
        let folder = cache
            .save_folder(&crate::data::message_cache::CachedFolder {
                id: 0,
                account_id: "acct".to_string(),
                name: "Inbox".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let row = cache
            .upsert_message(&crate::data::message_cache::IncomingMessage {
                folder_id: folder,
                uid: 1,
                message_id: "<one@example.com>".to_string(),
                subject: "Notes".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-07-26T10:00:00+00:00".to_string(),
                internal_date: None,
                size_bytes: Some(64),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: Verdict {
                    level,
                    reasons: vec![reason.to_string()],
                },
                gmail_message_id: None,
                labels: None,
                receipt_to: None,
                pop_uidl: None,
            })
            .expect("a stored message");
        (folder, row)
    }

    #[test]
    fn test_the_label_says_that_nothing_leaves_this_computer() {
        // The whole reason this can be on by default. It is beside a box that
        // does send something, so saying which is which is the difference
        // between an informed choice and a guess.
        assert!(LOOKING_AT_THE_MESSAGE_ITSELF.contains("Nothing is sent"));
    }

    #[test]
    fn test_the_label_reads_as_sentences_rather_than_a_wrapped_literal() {
        // A wrapped literal that loses its continuations keeps every space of
        // the indenting, and this one is read aloud. Runs of stray spaces are
        // silences in the middle of a sentence.
        assert!(
            !LOOKING_AT_THE_MESSAGE_ITSELF.contains("  "),
            "{LOOKING_AT_THE_MESSAGE_ITSELF}"
        );
    }
}

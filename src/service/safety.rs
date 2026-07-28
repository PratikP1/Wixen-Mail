//! What is already known about whether a message is safe.
//!
//! The most reliable free phishing and spam detection available to this
//! application is the detection that has already happened. Gmail, Outlook and
//! every host running SpamAssassin filter mail before it reaches us, and they
//! record the answer in the headers. Reading it costs nothing, needs no account
//! with anybody, and sends no part of somebody's mail to a third party.
//!
//! That last point decides the design. Asking an outside service whether a link
//! is a phishing site means handing that service the links from private
//! correspondence, which is not a trade to make quietly on somebody's behalf.
//!
//! Three sources feed a verdict, and the worst one wins:
//!
//! 1. The provider's own filter, from the headers here.
//! 2. Which folder the message is in, since junk is a verdict too.
//! 3. Our own checks, for the things filters do not flag, such as a link whose
//!    text and target disagree.

/// How much trouble a message looks like.
///
/// Ordered worst last, so merging verdicts is a maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Safety {
    /// Nothing said otherwise.
    #[default]
    Ordinary,
    /// Something is off, but nothing called it an attack.
    Suspicious,
    /// A filter put it in the junk pile.
    Spam,
    /// It is pretending to come from somebody it does not come from.
    Phishing,
}

impl Safety {
    /// What the message list column shows, and reads aloud.
    ///
    /// Empty for an ordinary message. Like the other flag columns, it costs
    /// listening time only when it has something to say.
    pub fn label(self) -> &'static str {
        match self {
            Safety::Ordinary => "",
            Safety::Suspicious => "Suspicious",
            Safety::Spam => "Spam",
            Safety::Phishing => "Phishing",
        }
    }

    /// Whether this is worth interrupting somebody to say.
    pub fn worth_announcing(self) -> bool {
        self != Safety::Ordinary
    }

    /// How it is written in the database.
    ///
    /// Words rather than numbers, so a stored mailbox can be read by somebody
    /// looking at it with a SQLite browser, and so inserting a level later
    /// cannot renumber the ones already written.
    pub fn as_str(self) -> &'static str {
        match self {
            Safety::Ordinary => "ordinary",
            Safety::Suspicious => "suspicious",
            Safety::Spam => "spam",
            Safety::Phishing => "phishing",
        }
    }

    /// Read it back.
    ///
    /// Anything unrecognised is ordinary. A row written by a newer version
    /// should not make an older one refuse to show the mailbox.
    pub fn from_stored(stored: &str) -> Self {
        match stored {
            "suspicious" => Safety::Suspicious,
            "spam" => Safety::Spam,
            "phishing" => Safety::Phishing,
            _ => Safety::Ordinary,
        }
    }
}

/// A verdict, and why.
///
/// The reasons are what the notification bar shows and what gets read out, so
/// they are sentences about the message rather than the names of headers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Verdict {
    pub level: Safety,
    pub reasons: Vec<String>,
}

impl Verdict {
    /// An ordinary message with nothing to say about it.
    pub fn ordinary() -> Self {
        Self::default()
    }

    fn flagged(level: Safety, reason: &str) -> Self {
        Self {
            level,
            reasons: vec![reason.to_string()],
        }
    }

    /// Combine two verdicts, keeping the worse level and both sets of reasons.
    ///
    /// A message can be in the junk folder *and* fail its sender's anti-forgery
    /// check, and somebody deciding whether to trust it wants both facts.
    pub fn and(mut self, other: Verdict) -> Self {
        self.level = self.level.max(other.level);
        for reason in other.reasons {
            if !self.reasons.contains(&reason) {
                self.reasons.push(reason);
            }
        }
        self
    }

    /// One sentence for the notification bar and for speech.
    pub fn summary(&self) -> String {
        match self.level {
            Safety::Ordinary => String::new(),
            Safety::Suspicious => {
                format!("This message looks suspicious. {}", self.reasons.join(" "))
            }
            Safety::Spam => format!(
                "This message was marked as spam. {}",
                self.reasons.join(" ")
            ),
            Safety::Phishing => format!(
                "Warning: this message looks like a phishing attempt. {}",
                self.reasons.join(" ")
            ),
        }
    }
}

/// The verdict a filter already reached, read out of the message headers.
pub fn from_headers(headers: &str) -> Verdict {
    let fields = unfold(headers);
    let mut verdict = Verdict::ordinary();

    for (name, value) in &fields {
        let found = match name.as_str() {
            "x-spam-flag" => spam_flag(value),
            "x-spam-status" => spam_status(value),
            "x-forefront-antispam-report" | "x-microsoft-antispam" => microsoft_report(value),
            "authentication-results" => authentication_results(value),
            _ => None,
        };
        if let Some(found) = found {
            verdict = verdict.and(found);
        }
    }

    verdict
}

/// The verdict from our own reading of the message.
///
/// The third source, and the only one that looks at the message rather than at
/// what somebody else concluded about it. Filters are good at bulk and blunt
/// about the targeted message written to one person, which is the one that
/// costs somebody their bank account. This catches what they miss: a link whose
/// text and target disagree, a lookalike domain, a raw IP address, a reply-to
/// pointing somewhere the sender is not.
///
/// Deliberately never worse than [`Safety::Suspicious`] on its own. Calling
/// something a phishing attempt on a heuristic score is how a warning becomes
/// the thing people click past; the provider's filter and DMARC are the two
/// sources allowed to say that word. Ours says "look at this".
pub fn from_analysis(risk: PhishingRisk, indicators: &[String]) -> Verdict {
    let level = match risk {
        PhishingRisk::High | PhishingRisk::Medium => Safety::Suspicious,
        PhishingRisk::Low | PhishingRisk::None => return Verdict::ordinary(),
    };

    Verdict {
        level,
        reasons: indicators.iter().map(|it| as_sentence(it)).collect(),
    }
}

/// How likely our own checks think a message is an impersonation.
///
/// Mirrors `service::security::PhishingRiskLevel`, converted at the boundary so
/// this module does not depend on the analyser's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhishingRisk {
    None,
    Low,
    Medium,
    High,
}

/// Turn an indicator into something worth reading aloud.
///
/// The analyser writes them as notes to a developer: "Sender/response
/// instruction mismatch". Read out to somebody deciding whether to trust an
/// email, that is a phrase to decode rather than a fact to act on.
fn as_sentence(indicator: &str) -> String {
    let plain = match indicator {
        "Sender/response instruction mismatch" => {
            "A reply to this would go to a different address from the one it claims to be from."
        }
        "Contains URL using raw IP address" => {
            "A link points at a bare numeric address rather than a name."
        }
        "Contains punycode-like domain (possible homograph)" => {
            "A link uses a domain built to look like a different one."
        }
        "Detected deceptive link text/href mismatch" => {
            "A link says it goes one place and goes somewhere else."
        }
        other if other.starts_with("Urgency or account pressure phrase") => {
            "The wording pushes for an urgent response, which is how these messages work."
        }
        // Anything added to the analyser later still reaches somebody, as
        // itself, rather than being dropped for not being on this list.
        other => return format!("{}.", other.trim_end_matches('.')),
    };
    plain.to_string()
}

/// The verdict implied by the message sitting in a junk folder.
pub fn from_folder(is_junk_folder: bool) -> Verdict {
    if is_junk_folder {
        Verdict::flagged(
            Safety::Spam,
            "Your mail provider put it in the junk folder.",
        )
    } else {
        Verdict::ordinary()
    }
}

/// SpamAssassin and most hosts that run it.
fn spam_flag(value: &str) -> Option<Verdict> {
    value.trim().eq_ignore_ascii_case("yes").then(|| {
        Verdict::flagged(
            Safety::Spam,
            "Your mail provider's filter marked it as spam.",
        )
    })
}

/// `X-Spam-Status: Yes, score=8.1 required=5.0 tests=...`
fn spam_status(value: &str) -> Option<Verdict> {
    let verdict = value.split(',').next()?.trim();
    verdict.eq_ignore_ascii_case("yes").then(|| {
        Verdict::flagged(
            Safety::Spam,
            "Your mail provider's filter marked it as spam.",
        )
    })
}

/// Exchange and Outlook, which report confidence levels rather than a flag.
///
/// SCL is how sure the filter is that it is spam, on a scale where anything
/// from 5 up is treated as spam and -1 means it was trusted outright. PCL is
/// the same idea for phishing.
fn microsoft_report(value: &str) -> Option<Verdict> {
    let mut verdict = Verdict::ordinary();

    if let Some(pcl) = tagged_number(value, "PCL")
        && pcl >= 4
    {
        verdict = verdict.and(Verdict::flagged(
            Safety::Phishing,
            "Your mail provider's filter rated it a likely phishing attempt.",
        ));
    }
    if let Some(scl) = tagged_number(value, "SCL")
        && scl >= 5
    {
        verdict = verdict.and(Verdict::flagged(
            Safety::Spam,
            "Your mail provider's filter rated it as spam.",
        ));
    }

    (verdict.level != Safety::Ordinary).then_some(verdict)
}

/// Pull `NAME:12` out of a semicolon separated report.
fn tagged_number(value: &str, tag: &str) -> Option<i32> {
    value.split(';').find_map(|part| {
        let (name, number) = part.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case(tag)
            .then(|| number.trim().parse().ok())
            .flatten()
    })
}

/// What the receiving server made of the sender's own anti-forgery records.
///
/// DMARC is the one that matters. It is the domain owner saying "mail from me
/// looks like this", so a failure means the message did not come from where it
/// says. SPF alone failing is routine: every forwarded message and every
/// mailing list breaks it, and treating that as an attack would cry wolf on
/// half an inbox.
fn authentication_results(value: &str) -> Option<Verdict> {
    let lower = value.to_lowercase();

    if result_is(&lower, "dmarc", "fail") {
        return Some(Verdict::flagged(
            Safety::Phishing,
            "The address it claims to be from failed that domain's own anti-forgery check.",
        ));
    }
    if result_is(&lower, "spf", "fail") && result_is(&lower, "dkim", "fail") {
        return Some(Verdict::flagged(
            Safety::Suspicious,
            "Neither of the sender's two anti-forgery checks passed.",
        ));
    }

    None
}

fn result_is(lower: &str, method: &str, outcome: &str) -> bool {
    lower.split(';').any(|part| {
        let part = part.trim();
        part.strip_prefix(method)
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(|rest| rest.trim_start().starts_with(outcome))
            .unwrap_or(false)
    })
}

/// Split headers into `(lowercase name, value)`, joining folded lines.
///
/// A header may be continued on the next line if that line starts with
/// whitespace, and a long `Authentication-Results` almost always is.
fn unfold(headers: &str) -> Vec<(String, String)> {
    let mut fields: Vec<(String, String)> = Vec::new();

    for line in headers.lines() {
        if line.starts_with([' ', '\t']) {
            if let Some((_, value)) = fields.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            fields.push((name.trim().to_lowercase(), value.trim().to_string()));
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_ordinary_message_says_nothing() {
        let verdict = from_headers("Subject: lunch\r\nFrom: friend@example.com\r\n");

        assert_eq!(verdict.level, Safety::Ordinary);
        assert!(verdict.reasons.is_empty());
        assert_eq!(verdict.summary(), "");
    }

    #[test]
    fn test_the_spam_flag_is_read() {
        let verdict = from_headers("X-Spam-Flag: YES\r\n");

        assert_eq!(verdict.level, Safety::Spam);
    }

    #[test]
    fn test_a_spam_flag_of_no_is_not_spam() {
        assert_eq!(from_headers("X-Spam-Flag: NO\r\n").level, Safety::Ordinary);
    }

    #[test]
    fn test_the_spam_status_verdict_is_read_and_its_score_ignored() {
        // The score is the filter's working, not its answer. Reading it and
        // applying our own threshold would second-guess a filter that knows
        // more about the sender than we do.
        assert_eq!(
            from_headers("X-Spam-Status: Yes, score=8.1 required=5.0\r\n").level,
            Safety::Spam
        );
        assert_eq!(
            from_headers("X-Spam-Status: No, score=-2.6 required=5.0\r\n").level,
            Safety::Ordinary
        );
    }

    #[test]
    fn test_a_microsoft_spam_confidence_level_is_read() {
        let verdict = from_headers("X-Forefront-Antispam-Report: CIP:1.2.3.4;CTRY:US;SCL:9;\r\n");

        assert_eq!(verdict.level, Safety::Spam);
    }

    #[test]
    fn test_a_low_spam_confidence_level_is_not_spam() {
        // -1 means the filter trusted it outright, and 1 means it looked fine.
        for report in ["SCL:-1", "SCL:1"] {
            assert_eq!(
                from_headers(&format!("X-Forefront-Antispam-Report: {report};\r\n")).level,
                Safety::Ordinary,
                "{report} was treated as spam"
            );
        }
    }

    #[test]
    fn test_a_microsoft_phishing_confidence_level_outranks_its_spam_level() {
        let verdict = from_headers("X-Forefront-Antispam-Report: SCL:9;PCL:6;\r\n");

        assert_eq!(verdict.level, Safety::Phishing);
    }

    #[test]
    fn test_failing_dmarc_is_treated_as_impersonation() {
        // DMARC failing is the domain owner's own records saying this did not
        // come from them, which is the definition of the thing.
        let verdict = from_headers(
            "Authentication-Results: mx.google.com; spf=pass; dkim=pass; dmarc=fail header.from=paypal.com\r\n",
        );

        assert_eq!(verdict.level, Safety::Phishing);
    }

    #[test]
    fn test_spf_failing_on_its_own_is_not_reported() {
        // Every forwarded message and every mailing list breaks SPF. Calling
        // that an attack would flag half an inbox and teach people to ignore
        // the warning.
        let verdict =
            from_headers("Authentication-Results: mx.google.com; spf=fail; dkim=pass\r\n");

        assert_eq!(verdict.level, Safety::Ordinary);
    }

    #[test]
    fn test_both_checks_failing_is_worth_a_word() {
        let verdict =
            from_headers("Authentication-Results: mx.google.com; spf=fail; dkim=fail\r\n");

        assert_eq!(verdict.level, Safety::Suspicious);
    }

    #[test]
    fn test_a_folded_header_is_read_whole() {
        // Authentication-Results is long and is nearly always wrapped. Reading
        // only the first line would miss the dmarc result on most real mail.
        let verdict = from_headers(
            "Authentication-Results: mx.google.com;\r\n       spf=pass;\r\n       dmarc=fail header.from=bank.example\r\n",
        );

        assert_eq!(verdict.level, Safety::Phishing);
    }

    #[test]
    fn test_header_names_are_matched_whatever_their_case() {
        assert_eq!(from_headers("x-spam-flag: yes\r\n").level, Safety::Spam);
        assert_eq!(from_headers("X-SPAM-FLAG: Yes\r\n").level, Safety::Spam);
    }

    #[test]
    fn test_the_worst_signal_decides() {
        let verdict = from_headers(
            "X-Spam-Flag: YES\r\nAuthentication-Results: mx.google.com; dmarc=fail\r\n",
        );

        assert_eq!(verdict.level, Safety::Phishing);
        assert_eq!(verdict.reasons.len(), 2, "both reasons should survive");
    }

    #[test]
    fn test_our_own_checks_never_call_something_phishing_on_their_own() {
        // A heuristic score saying "phishing" is how a warning becomes the
        // thing people learn to click past. Only the provider's filter and
        // DMARC get to use that word.
        for risk in [PhishingRisk::High, PhishingRisk::Medium] {
            let verdict =
                from_analysis(risk, &["Detected deceptive link text/href mismatch".into()]);

            assert_eq!(verdict.level, Safety::Suspicious, "{risk:?}");
        }
    }

    #[test]
    fn test_a_low_score_is_not_worth_saying_anything_about() {
        for risk in [PhishingRisk::Low, PhishingRisk::None] {
            assert_eq!(
                from_analysis(risk, &["something".into()]).level,
                Safety::Ordinary
            );
        }
    }

    #[test]
    fn test_an_indicator_is_turned_into_something_worth_hearing() {
        // "Sender/response instruction mismatch" is a note to a developer. It
        // is read out to somebody deciding whether to trust an email.
        let verdict = from_analysis(
            PhishingRisk::High,
            &["Sender/response instruction mismatch".to_string()],
        );

        let reason = &verdict.reasons[0];
        assert!(reason.contains("reply"), "got {reason}");
        assert!(reason.ends_with('.'), "not a sentence: {reason}");
    }

    #[test]
    fn test_an_indicator_nobody_has_reworded_still_reaches_somebody() {
        // Anything added to the analyser later should read badly rather than
        // vanish. A dropped warning is worse than an awkward one.
        let verdict = from_analysis(PhishingRisk::High, &["Something new".to_string()]);

        assert_eq!(verdict.reasons, vec!["Something new.".to_string()]);
    }

    #[test]
    fn test_our_checks_merge_with_the_providers_rather_than_replacing_them() {
        let provider = from_headers(
            "X-Spam-Flag: YES
",
        );
        let ours = from_analysis(
            PhishingRisk::High,
            &["Detected deceptive link text/href mismatch".to_string()],
        );

        let both = provider.and(ours);

        // Spam outranks suspicious, and both reasons survive.
        assert_eq!(both.level, Safety::Spam);
        assert_eq!(both.reasons.len(), 2);
    }

    #[test]
    fn test_the_junk_folder_is_a_verdict_too() {
        // Gmail does not add a spam header. It moves the message, and that is
        // the whole of what it tells an IMAP client.
        assert_eq!(from_folder(true).level, Safety::Spam);
        assert_eq!(from_folder(false).level, Safety::Ordinary);
    }

    #[test]
    fn test_reasons_are_sentences_rather_than_header_names() {
        // These get read out. "X-Forefront-Antispam-Report SCL 9" is not
        // something to say to somebody deciding whether to trust an email.
        let verdict = from_headers(
            "X-Spam-Flag: YES\r\nAuthentication-Results: a; dmarc=fail\r\nX-Forefront-Antispam-Report: SCL:9;\r\n",
        );

        for reason in &verdict.reasons {
            assert!(
                !reason.contains("X-") && !reason.contains("SCL"),
                "reason reads like a header: {reason}"
            );
            assert!(reason.ends_with('.'), "not a sentence: {reason}");
        }
    }

    #[test]
    fn test_merging_keeps_the_worse_level_and_both_reasons() {
        let junk = from_folder(true);
        let forged = from_headers("Authentication-Results: a; dmarc=fail\r\n");

        let both = junk.and(forged);

        assert_eq!(both.level, Safety::Phishing);
        assert_eq!(both.reasons.len(), 2);
    }

    #[test]
    fn test_the_same_reason_is_not_said_twice() {
        // A host can set both X-Spam-Flag and X-Spam-Status, and hearing the
        // same sentence twice sounds like two separate problems.
        let verdict = from_headers("X-Spam-Flag: YES\r\nX-Spam-Status: Yes, score=9\r\n");

        assert_eq!(verdict.reasons.len(), 1);
    }

    #[test]
    fn test_only_a_flagged_message_takes_up_room_in_the_column() {
        assert_eq!(Safety::Ordinary.label(), "");
        assert_eq!(Safety::Spam.label(), "Spam");
        assert_eq!(Safety::Phishing.label(), "Phishing");
        assert!(!Safety::Ordinary.worth_announcing());
        assert!(Safety::Phishing.worth_announcing());
    }

    #[test]
    fn test_every_level_survives_a_trip_through_the_database() {
        for level in [
            Safety::Ordinary,
            Safety::Suspicious,
            Safety::Spam,
            Safety::Phishing,
        ] {
            assert_eq!(Safety::from_stored(level.as_str()), level);
        }
    }

    #[test]
    fn test_a_level_written_by_a_newer_version_does_not_break_the_mailbox() {
        // Showing the message as ordinary is wrong but harmless. Refusing to
        // list the folder because one row is unfamiliar is not.
        assert_eq!(Safety::from_stored("catastrophic"), Safety::Ordinary);
        assert_eq!(Safety::from_stored(""), Safety::Ordinary);
    }

    #[test]
    fn test_the_phishing_summary_leads_with_the_warning() {
        // Somebody may stop listening after the first few words, so the worst
        // news goes first rather than after a preamble.
        let summary = from_headers("Authentication-Results: a; dmarc=fail\r\n").summary();

        assert!(summary.starts_with("Warning:"), "got {summary}");
    }
}

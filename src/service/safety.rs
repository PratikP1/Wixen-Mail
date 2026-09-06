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
//! Four sources feed a verdict, and the worst one wins. This said three until
//! link checking was built, and the fourth had been joining the merge for some
//! time by then:
//!
//! 1. The provider's own filter, from the headers here. What the receiving
//!    server made of the sender's own published anti-forgery rules arrives in
//!    the same headers and is a different judge reaching a different kind of
//!    answer, so it says so rather than sounding like the filter.
//! 2. Which folder the message is in, since junk is a verdict too.
//! 3. Our own checks, for the things filters do not flag, such as a link whose
//!    text and target disagree.
//! 4. Google Safe Browsing, when somebody has switched link checking on. Off
//!    unless asked for, because it is the one source that sends anything
//!    anywhere. `service::safebrowsing` says separately what it sends.
//!
//! **Every sentence any of them contributes says which of them reached it**,
//! and no source says its own name twice in one bar. That is what makes a bar
//! with four sentences in it worth listening to rather than four facts
//! somebody has to sort out for themselves, and it is guardrail 5: feedback
//! distinct and bounded, where bounded is the half that is easy to lose. Three
//! of the four named themselves already, in three different grammatical
//! places. This program's own reading named nobody at all, so a guess made on
//! this computer was read out in the same voice as a filter's verdict.

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
    ///
    /// Written as `Self::default()` on purpose: a mutation test replacing
    /// this whole body with `Default::default()` survives, and it always
    /// will, because the two are the same call. This function takes no
    /// argument to tell them apart by, and the return type fixes what
    /// `Default::default()` resolves to as `<Verdict as Default>::default()`,
    /// identically to `Self::default()`. There is no invariant to assert here
    /// the way there is for an operator swap on disjoint bits elsewhere in
    /// this sweep: nothing external could ever make the two calls diverge,
    /// so nothing is asserted, only written down.
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
///
/// And it says so in this program's name. Never sounding like the provider is
/// the same argument as never saying "phishing", one step further: a guess
/// wearing somebody else's authority is the thing being guarded against, and
/// "A link points at a bare numeric address rather than a name" wore it by
/// saying nothing at all about where it came from.
///
/// One sentence however many things were found, which is the other half and
/// the half that is easy to lose. Guardrail 5 asks for feedback that is
/// distinct *and* bounded. Attribution written as a clause on every sentence
/// would give somebody hearing five findings the same eight words five times,
/// which is repetition rather than attribution.
pub fn from_analysis(risk: PhishingRisk, indicators: &[String]) -> Verdict {
    let level = match risk {
        PhishingRisk::High | PhishingRisk::Medium => Safety::Suspicious,
        PhishingRisk::Low | PhishingRisk::None => return Verdict::ordinary(),
    };
    let found: Vec<String> = indicators.iter().map(|it| as_a_finding(it)).collect();
    if found.is_empty() {
        // A risk score with nothing behind it. A sentence saying this program
        // read the message and found, then stopping, is worse than silence.
        return Verdict::ordinary();
    }

    Verdict {
        level,
        reasons: vec![format!(
            "Wixen Mail read this message on your computer and found {}.",
            one_after_another(&found)
        )],
    }
}

/// The words the settings screen uses for the same reading.
///
/// `application::body_safety::LOOKING_AT_THE_MESSAGE_ITSELF` opens "Wixen Mail
/// reads each message on this computer", and this sentence opens the same way
/// on purpose: somebody who chose to leave that setting on meets the same words
/// again when it finds something, so the warning and the setting are one thing
/// rather than two.
///
/// A list read out loud rather than a comma-separated one. "a and b" for two,
/// "a, b and c" for more, which is how somebody would say it.
fn one_after_another(found: &[String]) -> String {
    match found {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
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
///
/// A thing found rather than a sentence, because these are read out inside one
/// sentence naming the reading that found them. Written to follow "and found",
/// so they are noun phrases: "a link pointing at a bare numeric address"
/// rather than "A link points at a bare numeric address."
fn as_a_finding(indicator: &str) -> String {
    let plain = match indicator {
        "Sender/response instruction mismatch" => {
            "a reply that would go to a different address from the one this claims to be from"
        }
        "Contains URL using raw IP address" => {
            "a link pointing at a bare numeric address rather than a name"
        }
        "Contains punycode-like domain (possible homograph)" => {
            "a link using a domain built to look like a different one"
        }
        "Detected deceptive link text/href mismatch" => {
            "a link that says it goes one place and goes somewhere else"
        }
        other if other.starts_with("Urgency or account pressure phrase") => {
            "wording that pushes for an urgent response, which is how these messages work"
        }
        // Anything added to the analyser later still reaches somebody, as
        // itself, rather than being dropped for not being on this list. It
        // reads badly in the middle of a sentence, and an awkward warning is
        // better than a dropped one.
        other => return other.trim_end_matches('.').to_string(),
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
///
/// One sentence when both are set, rather than two opening with the same six
/// words. This is one filter saying two things about one message, and hearing
/// "Your mail provider's filter" twice in a row is repetition rather than a
/// second fact. Guardrail 5 again, and the same reason [`from_analysis`] says
/// everything it found in one sentence.
fn microsoft_report(value: &str) -> Option<Verdict> {
    let phishing = tagged_number(value, "PCL").is_some_and(|pcl| pcl >= 4);
    let spam = tagged_number(value, "SCL").is_some_and(|scl| scl >= 5);

    match (phishing, spam) {
        (true, true) => Some(Verdict::flagged(
            Safety::Phishing,
            "Your mail provider's filter rated it a likely phishing attempt, and as spam.",
        )),
        (true, false) => Some(Verdict::flagged(
            Safety::Phishing,
            "Your mail provider's filter rated it a likely phishing attempt.",
        )),
        (false, true) => Some(Verdict::flagged(
            Safety::Spam,
            "Your mail provider's filter rated it as spam.",
        )),
        (false, false) => None,
    }
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
///
/// Both sentences name the sender's own domain as the thing this message
/// failed, and that is the attribution rather than a flourish. This is a
/// different judge from the spam filter above, reaching a different kind of
/// answer: a filter has an opinion about what a message contains, and these
/// records are the sender's own domain publishing what its mail looks like, so
/// failing them is a fact rather than a judgement. Somebody deciding whether to
/// trust a message is owed that difference, and opening these with the
/// provider's name would have hidden it behind the same words.
fn authentication_results(value: &str) -> Option<Verdict> {
    let lower = value.to_lowercase();

    if result_is(&lower, "dmarc", "fail") {
        return Some(Verdict::flagged(
            Safety::Phishing,
            "The sender's own domain publishes what its mail should look like, and this \
             message does not match it.",
        ));
    }
    if result_is(&lower, "spf", "fail") && result_is(&lower, "dkim", "fail") {
        return Some(Verdict::flagged(
            Safety::Suspicious,
            "The sender's own domain publishes two anti-forgery records, and this message \
             passed neither.",
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
    fn test_every_reason_is_said_as_something_worth_hearing() {
        // The analyser writes its indicators as notes to a developer. Read out
        // to somebody deciding whether to trust a message, "Sender/response
        // instruction mismatch" is a phrase to decode rather than a fact to
        // act on. The urgency one carries the phrase it found on the end, so
        // it is matched by its beginning, and that was the arm with nothing
        // watching it.
        let urgent = as_a_finding("Urgency or account pressure phrase: 'verify now'");
        assert!(
            urgent.contains("pushes for an urgent response"),
            "an urgency reason was read out as its own label: {urgent}"
        );

        // Anything the analyser learns later still reaches somebody as itself,
        // rather than being dropped for not being on the list.
        assert_eq!(
            as_a_finding("Something nobody has written a sentence for"),
            "Something nobody has written a sentence for"
        );
        assert_eq!(
            as_a_finding("Already ends in a full stop."),
            "Already ends in a full stop"
        );
    }

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
        // vanish. A dropped warning is worse than an awkward one, and the
        // developer's own words in the middle of a sentence about what this
        // program found is exactly the awkward one.
        let verdict = from_analysis(PhishingRisk::High, &["Something new".to_string()]);

        assert_eq!(
            verdict.reasons,
            vec![
                "Wixen Mail read this message on your computer and found Something new."
                    .to_string()
            ]
        );
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

    /// The openings that say which of the four things judged a message.
    ///
    /// Written down here rather than asserted one sentence at a time, because
    /// the question is about the bar as a whole: somebody hearing four
    /// sentences has to be able to sort them by who said them, and a sentence
    /// belonging to none of these is one they cannot place.
    ///
    /// Google's is at the end of its sentence rather than the start, because
    /// their terms fix the wording wherever a warning derived from their data
    /// is shown. That is the one attribution this project does not get to
    /// choose, so it is matched anywhere in the sentence.
    const WHO_A_SENTENCE_MAY_SAY_JUDGED_IT: [&str; 4] = [
        "Your mail provider's filter",
        "Your mail provider put it",
        "The sender's own domain",
        "Wixen Mail read this message",
    ];

    /// Whether a sentence says which of them reached it.
    fn says_who_judged_it(reason: &str) -> bool {
        WHO_A_SENTENCE_MAY_SAY_JUDGED_IT
            .iter()
            .any(|who| reason.starts_with(who))
            || reason.contains("Google Safe Browsing")
    }

    #[test]
    fn test_this_programs_own_reading_says_that_it_was_this_program_that_read_it() {
        // The whole of the second half of criterion 6, and the repudiation
        // this is about. `from_analysis`'s own doc already argues that this
        // program's reading is never allowed to say the word phishing, because
        // a heuristic wearing the provider's authority is how a warning becomes
        // the thing people click past. Sounding like the provider is the same
        // argument one step further: "A link points at a bare numeric address"
        // named nobody, so a guess and a filter's verdict were read out in the
        // same voice.
        let ours = from_analysis(
            PhishingRisk::High,
            &["Contains URL using raw IP address".to_string()],
        );

        assert_eq!(ours.reasons.len(), 1);
        assert!(
            ours.reasons[0].starts_with("Wixen Mail read this message"),
            "this program's own reading does not say it was this program: {:?}",
            ours.reasons[0]
        );
        assert!(
            !ours.reasons[0].contains("provider"),
            "this program's own reading reads as though the provider said it: {:?}",
            ours.reasons[0]
        );
    }

    #[test]
    fn test_several_things_found_here_are_one_sentence_rather_than_one_each() {
        // Guardrail 5's second half, which is the one that is easy to lose:
        // feedback must be distinct AND bounded. Attribution added a clause to
        // every sentence would give somebody hearing five findings the same
        // eight words five times, which is repetition rather than attribution
        // and is exactly how a warning bar becomes something people talk past.
        let ours = from_analysis(
            PhishingRisk::High,
            &[
                "Contains URL using raw IP address".to_string(),
                "Detected deceptive link text/href mismatch".to_string(),
                "Urgency or account pressure phrase: 'verify now'".to_string(),
            ],
        );

        assert_eq!(
            ours.reasons.len(),
            1,
            "three findings from one source were said as {} sentences: {:?}",
            ours.reasons.len(),
            ours.reasons
        );
        let said = &ours.reasons[0];
        for found in [
            "bare numeric address",
            "goes one place",
            "pushes for an urgent response",
        ] {
            assert!(
                said.contains(found),
                "a finding was dropped on the way into one sentence: {found:?} is not in \
                 {said:?}"
            );
        }
        assert_eq!(
            said.matches("Wixen Mail").count(),
            1,
            "the source is repeated inside one sentence: {said:?}"
        );
    }

    #[test]
    fn test_a_filter_that_rated_it_both_ways_says_so_in_one_sentence() {
        // The other place the same repetition was already happening. A
        // Microsoft report carrying both a phishing and a spam confidence
        // produced two sentences opening with the same six words.
        let verdict = from_headers("X-Forefront-Antispam-Report: SCL:9;PCL:6;\r\n");

        assert_eq!(verdict.level, Safety::Phishing);
        assert_eq!(
            verdict.reasons.len(),
            1,
            "one filter said two things and was quoted twice: {:?}",
            verdict.reasons
        );
    }

    #[test]
    fn test_the_senders_own_records_are_named_as_what_the_message_failed() {
        // Not the provider's filter, which is a different judge reaching a
        // different kind of answer. A filter has an opinion about the contents;
        // published anti-forgery records are the sender's own domain saying
        // what its mail looks like, and a message failing them is a fact rather
        // than a judgement. Somebody deciding whether to trust a message is
        // owed the difference.
        let forged = from_headers("Authentication-Results: mx.google.com; dmarc=fail\r\n");

        assert!(
            forged.reasons[0].starts_with("The sender's own domain"),
            "a failed anti-forgery check does not say whose records failed: {:?}",
            forged.reasons[0]
        );
        assert!(
            !forged.reasons[0].contains("filter"),
            "a failed anti-forgery check reads as the provider's spam filter: {:?}",
            forged.reasons[0]
        );
    }

    #[test]
    fn test_every_sentence_in_a_bar_carrying_three_sources_says_which_one_said_it() {
        // The bar as a whole, which is the thing somebody hears. Three sources
        // at once: a filter's header, the folder it was put in, and this
        // program's own reading.
        let bar = from_headers("X-Spam-Flag: YES\r\n")
            .and(from_folder(true))
            .and(from_analysis(
                PhishingRisk::High,
                &["Detected deceptive link text/href mismatch".to_string()],
            ));

        assert_eq!(
            bar.reasons.len(),
            3,
            "three sources did not give three sentences: {:?}",
            bar.reasons
        );
        for reason in &bar.reasons {
            assert!(
                says_who_judged_it(reason),
                "a sentence in the bar says nothing about which of them reached it: {reason:?}"
            );
        }
    }

    #[test]
    fn test_merging_keeps_every_sentence_with_the_one_that_said_it() {
        // The worst-wins merge takes two lists and makes one, and the order
        // they end up in is not the order anything wrote them. A sentence that
        // carried its source in a field beside it could come away from that
        // field here; carrying it in the sentence is what makes that
        // impossible, and this is the assertion that says so.
        let ours = from_analysis(
            PhishingRisk::High,
            &["Contains URL using raw IP address".to_string()],
        );
        let theirs = from_headers("X-Spam-Flag: YES\r\n");

        for merged in [ours.clone().and(theirs.clone()), theirs.and(ours)] {
            assert_eq!(merged.reasons.len(), 2);
            for reason in &merged.reasons {
                assert!(
                    says_who_judged_it(reason),
                    "a sentence lost its source in the merge: {reason:?}"
                );
            }
        }
    }

    #[test]
    fn test_an_ordinary_message_still_says_nothing_at_all() {
        // The case that covers nearly all mail, restated because this is a
        // change to what the sentences say and any word added to this one is a
        // word paid for on every message somebody ever receives.
        assert_eq!(from_analysis(PhishingRisk::None, &[]).summary(), "");
        assert_eq!(
            from_analysis(
                PhishingRisk::Low,
                &["Contains URL using raw IP address".to_string()]
            )
            .reasons,
            Vec::<String>::new()
        );
        assert_eq!(Verdict::ordinary().summary(), "");
    }

    #[test]
    fn test_the_phishing_summary_leads_with_the_warning() {
        // Somebody may stop listening after the first few words, so the worst
        // news goes first rather than after a preamble.
        let summary = from_headers("Authentication-Results: a; dmarc=fail\r\n").summary();

        assert!(summary.starts_with("Warning:"), "got {summary}");
    }
}

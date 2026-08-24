//! Read receipts: who asked to be told you opened their mail, and whether to
//! tell them.
//!
//! A sender can put `Disposition-Notification-To` on a message (RFC 8098, and
//! `Return-Receipt-To` from before it) asking the receiving client to send back
//! a note saying it was displayed. Nothing obliges a client to, and the choice
//! belongs to the person reading, not the person who wrote.
//!
//! # Why the default is never
//!
//! A receipt tells the sender three things: that the address is real, that
//! somebody is behind it, and roughly when they were at their desk. To a
//! marketer that is a confirmed lead; to somebody sending bulk mail it is a
//! working address worth selling; to somebody targeting one person it is a
//! presence signal. None of that is what the person reading their mail was
//! trying to say.
//!
//! So nothing is sent unless somebody chose it, and the setting says what it
//! costs rather than which is recommended.
//!
//! # Why "always" is still not always
//!
//! Two cases are refused whatever the setting says.
//!
//! Mail in the junk folder, because a receipt to a spammer is the one reply
//! that makes the address more valuable, and it would be sent by a client
//! nobody had asked anything.
//!
//! And a request that names an address different from the one the mail came
//! from. That is the shape of a message written to make a receipt go somewhere
//! its sender chose, and it is how the mechanism gets used as a beacon.
//! RFC 3798 raises exactly this and says the client should not send
//! automatically. It becomes a question rather than a refusal, because there
//! are honest reasons for it too: a mailing list, or somebody who sends from
//! one address and reads at another.

/// What somebody has chosen about receipts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Policy {
    /// Never tell anybody. The default, and what happens with no setting.
    #[default]
    Never,
    /// Ask each time, and do nothing unless the answer is yes.
    Ask,
    /// Send one whenever it is asked for, except the two refusals above.
    Always,
}

impl Policy {
    /// How the setting stores itself, and reads back.
    pub const fn as_str(self) -> &'static str {
        match self {
            Policy::Never => "never",
            Policy::Ask => "ask",
            Policy::Always => "always",
        }
    }

    /// Read a stored setting, falling back to the private answer.
    ///
    /// Anything unrecognised is `Never`. A setting file somebody has edited by
    /// hand, or one written by a later version, must not turn confirmations on
    /// by accident.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "ask" => Policy::Ask,
            "always" => Policy::Always,
            _ => Policy::Never,
        }
    }

    /// What the choice says in the settings screen.
    pub const fn spoken(self) -> &'static str {
        match self {
            Policy::Never => {
                "Never tell senders when you read their mail. \
                 Nobody learns that your address is live or when you are at your desk"
            }
            Policy::Ask => "Ask each time. Nothing is sent unless you say yes to that message",
            Policy::Always => {
                "Send a receipt whenever one is asked for. \
                 Junk mail and requests pointing at a different address are still refused"
            }
        }
    }

    /// Every choice, so the settings screen and its tests cover the set.
    pub const ALL: [Policy; 3] = [Policy::Never, Policy::Ask, Policy::Always];
}

/// A sender asking to be told the message was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Where the sender asked the receipt to go.
    pub notify: String,
}

/// What to do about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Say nothing, and say nothing about saying nothing.
    Ignore,
    /// Tell the person, and send only if they say yes.
    Ask { notify: String, why: &'static str },
    /// Send it.
    Send { notify: String },
}

/// Whether the request and the sender agree about who should be told.
///
/// A receipt going somewhere other than where the mail came from is the shape
/// that gets used as a beacon, so it is never automatic.
fn same_party(notify: &str, from: &str) -> bool {
    address_of(notify).eq_ignore_ascii_case(&address_of(from))
}

/// The bare address out of a header value that may carry a display name.
///
/// What goes in the envelope when a receipt is sent, as well as what the
/// same-party check compares. The raw header value used to be handed to the
/// envelope, and a sender who wrote a display name, which Thunderbird does by
/// default, made every receipt fail with "Invalid email user": an envelope
/// address is a bare address and nothing else.
pub fn address_of(value: &str) -> String {
    let value = value.trim();
    if let Some(open) = value.rfind('<')
        && let Some(close) = value[open..].find('>')
    {
        return value[open + 1..open + close].trim().to_ascii_lowercase();
    }
    value.to_ascii_lowercase()
}

/// What to do about a message that may have asked for a receipt.
///
/// `from` is the address the message came from, and `in_junk` says whether it
/// is in the junk folder. Both are needed to answer, which is why this takes
/// them rather than reading a header on its own.
pub fn answer(policy: Policy, request: Option<&Request>, from: &str, in_junk: bool) -> Answer {
    let Some(request) = request else {
        return Answer::Ignore;
    };
    if policy == Policy::Never {
        return Answer::Ignore;
    }
    if in_junk {
        // The one reply that makes a spammer's list more valuable.
        return Answer::Ignore;
    }
    if !same_party(&request.notify, from) {
        return Answer::Ask {
            notify: request.notify.clone(),
            why: "the receipt would go to a different address from the one that sent the message",
        };
    }
    match policy {
        Policy::Always => Answer::Send {
            notify: request.notify.clone(),
        },
        Policy::Ask | Policy::Never => Answer::Ask {
            notify: request.notify.clone(),
            why: "the sender asked to be told when you read this",
        },
    }
}

/// What to tell somebody a message asked for, in one sentence.
///
/// Said whatever the setting is, including `Never`, because "this sender wanted
/// to know when you read it" is a fact about the message worth having. What
/// changes with the setting is whether anything is sent, not whether the person
/// is told what was asked.
pub fn noticed(request: &Request, from: &str) -> String {
    if same_party(&request.notify, from) {
        "The sender asked to be told when you read this.".to_string()
    } else {
        format!(
            "The sender asked for a read receipt to be sent to {}, \
             which is not the address this came from.",
            address_of(&request.notify)
        )
    }
}

/// What a receipt is being sent about.
#[derive(Debug, Clone)]
pub struct About {
    /// This receipt's own `Message-ID`, brackets and all.
    ///
    /// Passed in rather than made here, so this stays a function whose output
    /// a test can read: a random value generated inside would make every one
    /// of those tests unrepeatable.
    pub own_id: String,
    /// The address the receipt goes to, as the sender wrote it.
    pub notify: String,
    /// The account's own address, which is what was read.
    pub reader: String,
    /// The original message's subject, for the human-readable part.
    pub subject: String,
    /// Its `Message-ID`, which is how the sender matches the receipt up.
    pub message_id: Option<String>,
    /// When it was read, as RFC 5322 spells a date.
    pub read_at: String,
}

/// The boundary between the two parts of a receipt.
///
/// Fixed rather than random, which is allowed: it only has to not appear in the
/// content, and the content here is generated and short. A fixed one also makes
/// the whole message reproducible, so the tests can read it.
const PART_BOUNDARY: &str = "wixen-mail-disposition-notification";

/// The whole receipt, as the bytes that go on the wire.
///
/// RFC 8098: a `multipart/report` with a sentence somebody could read and a
/// `message/disposition-notification` part a machine reads. Built here rather
/// than through the message builder because the structure is fixed, small, and
/// worth being able to read in a test.
///
/// It says `manual-action` and `MDN-sent-manually` whatever the setting is,
/// because that is the truth: nothing here is sent without somebody having
/// chosen, either for this message or once in the settings. Claiming
/// `automatic-action` would be telling the sender their message triggered
/// something, which is the thing this application does not do.
pub fn message(about: &About) -> Vec<u8> {
    // Every value below except the reader's own address is a stranger's text,
    // read off the message being answered: the subject, the address to notify,
    // and the message's own identifier. All three reach here decoded, so an
    // encoded word carrying a carriage return and line feed arrives as real
    // ones, and written onto a header line straight that starts a header of
    // the sender's choosing on mail going out of the reader's account.
    //
    // Through the same stripper the ordinary message builder uses, which has
    // a regression test on this exact shape. This builder never got it.
    use crate::application::draft_message::without_line_breaks;
    let mut out = String::new();

    out.push_str(&format!("From: {}\r\n", about.reader));
    out.push_str(&format!("To: {}\r\n", without_line_breaks(&about.notify)));
    out.push_str(&format!(
        "Subject: Read: {}\r\n",
        without_line_breaks(&about.subject)
    ));
    // Its own, which is a different thing from the two headers below that
    // name the message it is about.
    out.push_str(&format!("Message-ID: {}\r\n", about.own_id));
    out.push_str(&format!("Date: {}\r\n", about.read_at));
    if let Some(id) = about.message_id.as_deref().map(without_line_breaks) {
        // So the sender's client files it against the message it is about
        // rather than as loose mail.
        out.push_str(&format!("In-Reply-To: {id}\r\n"));
        out.push_str(&format!("References: {id}\r\n"));
    }
    out.push_str("MIME-Version: 1.0\r\n");
    out.push_str(&format!(
        "Content-Type: multipart/report; report-type=disposition-notification; \
         boundary=\"{PART_BOUNDARY}\"\r\n"
    ));
    out.push_str("\r\n");

    out.push_str(&format!("--{PART_BOUNDARY}\r\n"));
    out.push_str("Content-Type: text/plain; charset=utf-8\r\n\r\n");
    out.push_str(&format!(
        "Your message \"{}\" was displayed on {}.\r\n\
         This is no guarantee that it was read or understood.\r\n",
        about.subject, about.read_at
    ));
    out.push_str("\r\n");

    out.push_str(&format!("--{PART_BOUNDARY}\r\n"));
    out.push_str("Content-Type: message/disposition-notification\r\n\r\n");
    out.push_str(&format!(
        "Reporting-UA: Wixen Mail; {}\r\n",
        crate::common::version::current()
    ));
    out.push_str(&format!("Final-Recipient: rfc822; {}\r\n", about.reader));
    if let Some(id) = about.message_id.as_deref() {
        out.push_str(&format!("Original-Message-ID: {id}\r\n"));
    }
    out.push_str("Disposition: manual-action/MDN-sent-manually; displayed\r\n");
    out.push_str("\r\n");

    out.push_str(&format!("--{PART_BOUNDARY}--\r\n"));
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about() -> About {
        About {
            notify: "Ada <ada@example.com>".to_string(),
            reader: "charles@example.com".to_string(),
            subject: "Notes on the engine".to_string(),
            own_id: "<r-1@example.com>".to_string(),
            message_id: Some("<note-1@example.com>".to_string()),
            read_at: "Mon, 20 Jul 2026 10:00:00 +0000".to_string(),
        }
    }

    fn built() -> String {
        String::from_utf8(message(&about())).expect("valid text")
    }

    #[test]
    fn test_the_receipt_is_a_report_with_both_parts() {
        let raw = built();

        assert!(
            raw.contains("multipart/report; report-type=disposition-notification"),
            "{raw}"
        );
        assert!(raw.contains("Content-Type: text/plain"), "{raw}");
        assert!(
            raw.contains("Content-Type: message/disposition-notification"),
            "{raw}"
        );
        assert!(
            raw.contains(&format!("--{PART_BOUNDARY}--")),
            "the report is closed: {raw}"
        );
    }

    #[test]
    fn test_it_says_a_person_chose_to_send_it() {
        // Not `automatic-action`. Nothing here goes without somebody having
        // decided, and claiming otherwise would tell the sender their message
        // set something off by arriving.
        let raw = built();

        assert!(raw.contains("manual-action/MDN-sent-manually"), "{raw}");
        assert!(!raw.contains("automatic-action"), "{raw}");
    }

    #[test]
    fn test_it_is_filed_against_the_message_it_is_about() {
        // Without these the receipt arrives as loose mail and the sender has
        // to work out which message it answers.
        let raw = built();

        assert!(raw.contains("In-Reply-To: <note-1@example.com>"), "{raw}");
        assert!(
            raw.contains("Original-Message-ID: <note-1@example.com>"),
            "{raw}"
        );
    }

    #[test]
    fn test_a_receipt_carries_an_identifier_of_its_own() {
        // A receipt is mail leaving this machine with somebody's address on
        // it, and it had no identifier either, so the sender's client had
        // nothing to file it under except the headers naming the message it is
        // about. Its own identifier and those two are three different things,
        // and this fails if the new header is confused with either.
        let raw = built();

        assert!(raw.contains("Message-ID: <r-1@example.com>"), "{raw}");
        assert!(raw.contains("In-Reply-To: <note-1@example.com>"), "{raw}");
        assert!(
            raw.contains("Original-Message-ID: <note-1@example.com>"),
            "{raw}"
        );
    }

    #[test]
    fn test_a_message_with_no_id_still_produces_a_receipt() {
        // Some senders write none. A receipt that cannot be filed is still
        // better than a panic or an empty header.
        let mut nameless = about();
        nameless.message_id = None;

        let raw = String::from_utf8(message(&nameless)).expect("valid text");

        assert!(!raw.contains("In-Reply-To:"), "{raw}");
        assert!(!raw.contains("Original-Message-ID:"), "{raw}");
        assert!(raw.contains("Disposition:"), "{raw}");
    }

    #[test]
    fn test_the_wording_does_not_claim_it_was_read() {
        // It says displayed, because that is all a mail client can know. A
        // receipt claiming somebody read something is a claim about a person.
        let raw = built();

        assert!(raw.contains("no guarantee"), "{raw}");
    }

    #[test]
    fn test_it_says_which_client_sent_it_by_the_name_on_the_box() {
        assert!(built().contains("Reporting-UA: Wixen Mail;"));
    }

    fn asked(notify: &str) -> Request {
        Request {
            notify: notify.to_string(),
        }
    }

    #[test]
    fn test_nothing_is_sent_when_nothing_was_asked() {
        assert_eq!(
            answer(Policy::Always, None, "ada@example.com", false),
            Answer::Ignore
        );
    }

    #[test]
    fn test_the_default_tells_nobody_anything() {
        // A receipt confirms the address is live and somebody is behind it.
        // That is not something to give away because a stranger asked.
        assert_eq!(
            answer(
                Policy::Never,
                Some(&asked("ada@example.com")),
                "ada@example.com",
                false
            ),
            Answer::Ignore
        );
    }

    #[test]
    fn test_never_is_what_an_unreadable_setting_means() {
        // A settings file edited by hand, or written by a later version, must
        // not turn confirmations on by accident.
        assert_eq!(Policy::from_stored("yes please"), Policy::Never);
        assert_eq!(Policy::from_stored(""), Policy::Never);
        assert_eq!(Policy::default(), Policy::Never);
    }

    #[test]
    fn test_a_stored_choice_reads_back_as_itself() {
        for policy in Policy::ALL {
            assert_eq!(Policy::from_stored(policy.as_str()), policy);
        }
    }

    #[test]
    fn test_junk_is_never_answered() {
        // A receipt to a spammer is the one reply that makes the address worth
        // more, and it would go without anybody having been asked.
        assert_eq!(
            answer(
                Policy::Always,
                Some(&asked("spam@example.net")),
                "spam@example.net",
                true
            ),
            Answer::Ignore
        );
    }

    #[test]
    fn test_a_receipt_pointed_somewhere_else_is_never_automatic() {
        // The shape that gets used as a beacon: the message comes from one
        // address and asks for the confirmation to go to another.
        let answer = answer(
            Policy::Always,
            Some(&asked("tracker@elsewhere.example")),
            "Ada <ada@example.com>",
            false,
        );

        assert!(
            matches!(answer, Answer::Ask { .. }),
            "asked rather than sent: {answer:?}"
        );
    }

    #[test]
    fn test_always_sends_when_the_sender_is_asking_for_itself() {
        assert_eq!(
            answer(
                Policy::Always,
                Some(&asked("Ada <ada@example.com>")),
                "ada@example.com",
                false
            ),
            Answer::Send {
                notify: "Ada <ada@example.com>".to_string()
            }
        );
    }

    #[test]
    fn test_ask_asks_even_when_everything_agrees() {
        let answer = answer(
            Policy::Ask,
            Some(&asked("ada@example.com")),
            "ada@example.com",
            false,
        );

        assert!(matches!(answer, Answer::Ask { .. }), "{answer:?}");
    }

    #[test]
    fn test_a_display_name_does_not_make_two_addresses_different() {
        // "Ada Lovelace <ada@example.com>" and "ada@example.com" are the same
        // party, and treating them as different would question every ordinary
        // request.
        assert!(same_party(
            "Ada Lovelace <ada@example.com>",
            "ada@example.com"
        ));
        assert!(same_party("ADA@EXAMPLE.COM", "ada@example.com"));
        assert!(!same_party("ada@example.com", "eve@example.net"));
    }

    #[test]
    fn test_what_is_asked_is_said_whatever_the_setting() {
        // Including under Never. That a sender wanted to know is a fact about
        // the message, and worth having whether or not anything is sent.
        let ordinary = noticed(&asked("ada@example.com"), "Ada <ada@example.com>");
        assert!(ordinary.contains("asked to be told"), "{ordinary}");

        let odd = noticed(&asked("tracker@elsewhere.example"), "ada@example.com");
        assert!(odd.contains("tracker@elsewhere.example"), "{odd}");
        assert!(odd.contains("not the address this came from"), "{odd}");
    }

    #[test]
    fn test_every_choice_says_what_it_costs() {
        // "Recommended" on its own is something to click past.
        for policy in Policy::ALL {
            let said = policy.spoken();
            assert!(said.len() > 30, "{policy:?}: {said}");
        }
    }
}

#[cfg(test)]
mod nothing_a_stranger_wrote_starts_a_header {
    use super::*;

    fn about(subject: &str, notify: &str, message_id: Option<&str>) -> About {
        About {
            own_id: "<receipt@wixen-mail.invalid>".to_string(),
            notify: notify.to_string(),
            reader: "me@example.com".to_string(),
            subject: subject.to_string(),
            message_id: message_id.map(str::to_string),
            read_at: "Mon, 24 Aug 2026 10:00:00 +0000".to_string(),
        }
    }

    /// Every line of the built receipt down to the blank line that ends the
    /// headers.
    fn header_lines(built: &[u8]) -> Vec<String> {
        let text = String::from_utf8(built.to_vec()).expect("the receipt is text");
        let (headers, _) = text.split_once("\r\n\r\n").expect("headers end");
        headers.lines().map(str::to_string).collect()
    }

    #[test]
    fn test_a_subject_carrying_a_line_break_does_not_become_a_header() {
        // The subject is a stranger's text, and it reaches here decoded: an
        // encoded word carrying =0D=0A is legal and mail-parser turns it into
        // a real carriage return and line feed. Written onto a header line
        // straight, that starts a header of the sender's choosing on mail
        // going out of the reader's own account. The builder that makes
        // ordinary mail strips these and has a test using this exact shape;
        // this one never got it.
        let built = message(&about(
            "Invoice\r\nBcc: harvest@evil.example",
            "sam@example.com",
            None,
        ));

        assert!(
            !header_lines(&built)
                .iter()
                .any(|line| line.to_ascii_lowercase().starts_with("bcc:")),
            "a subject started a header of its own:\n{}",
            String::from_utf8_lossy(&built)
        );
    }

    #[test]
    fn test_neither_the_address_nor_the_message_id_can_start_one_either() {
        // Both are read straight off the message being answered, so both are
        // the sender's text as much as the subject is.
        let built = message(&about(
            "Ordinary",
            "sam@example.com\r\nBcc: harvest@evil.example",
            Some("<m1@x>\r\nBcc: also@evil.example"),
        ));

        assert!(
            !header_lines(&built)
                .iter()
                .any(|line| line.to_ascii_lowercase().starts_with("bcc:")),
            "a header was smuggled in:\n{}",
            String::from_utf8_lossy(&built)
        );
    }
}

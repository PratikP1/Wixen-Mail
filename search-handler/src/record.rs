//! One message, reduced to what the indexer is told about it.
//!
//! Nothing in here reaches out to anything. It is the layer where every
//! decision about what Windows learns from a message is made, so it is the
//! layer worth testing, and the COM objects around it only carry the answers
//! across.
//!
//! Whatever this module hands over is copied into the Windows Search index and
//! kept there. That index is not encrypted and other software on the machine
//! can read it, so the list below is the list of things somebody agrees to
//! share when they turn this on.

use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Storage::EnhancedStorage::{
    PKEY_ItemNameDisplay, PKEY_Kind, PKEY_Message_CcAddress, PKEY_Message_DateReceived,
    PKEY_Message_FromAddress, PKEY_Message_ToAddress, PKEY_Size, PKEY_Subject,
};

/// What a message with no subject is called.
///
/// The same words the main application's message list uses. Somebody who finds
/// a message through Windows Search and then goes looking for it in Wixen Mail
/// should be reading the same name in both places, and two spellings of "no
/// subject" is exactly the sort of difference that makes a person think they
/// have found a different message.
pub const NO_SUBJECT: &str = "No subject";

/// What Windows files this item under.
///
/// `System.Kind` takes a value from a fixed list Windows knows, and "email" is
/// the one that puts an item behind the Mail filter in an Explorer search.
/// Leaving it out does not fail; it files every message as unclassified, which
/// is worse, because the item is in the index and does not appear where
/// somebody looks for it.
const KIND: &str = "email";

/// One message as the indexer will see it.
///
/// Deliberately not the main application's message type. This crate is loaded
/// into a Microsoft process and reads a database read only; it has no business
/// carrying the fields that only matter to something that can write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The number that identifies the message inside its folder.
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    /// When it was sent, in seconds since the start of 1970.
    ///
    /// Optional because the cache keeps the date as text as the server wrote
    /// it, and a server can write something that is not a date. A message with
    /// an unreadable date is still worth finding.
    pub sent: Option<i64>,
    pub body: String,
}

/// A value handed to the indexer.
///
/// Three shapes rather than a string for everything, because a date stored as
/// text cannot be sorted or filtered by date, and a size stored as text sorts
/// as "10" before "9".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Text(String),
    Number(u64),
    /// Seconds since the start of 1970.
    Moment(i64),
    /// A child's URL together with when it last changed.
    ///
    /// This is not a property of a message. It is how a container tells the
    /// indexer what is inside it, and it lives here because it is still a
    /// value being handed over and the code that turns values into Windows
    /// types should only have one list to work from.
    UrlAndMoment {
        url: String,
        modified: i64,
    },
}

impl Message {
    /// What Explorer shows as this item's name.
    pub fn display_name(&self) -> &str {
        match self.subject.trim().is_empty() {
            true => NO_SUBJECT,
            false => &self.subject,
        }
    }

    /// The words a plain search actually looks through.
    ///
    /// The subject goes in as text as well as being a property of its own.
    /// Somebody searching for a word does not know or care whether it was in
    /// the subject line, and a search that only matched the body would miss
    /// the message they can see in front of them.
    pub fn searchable_text(&self) -> String {
        let mut text = String::with_capacity(self.subject.len() + self.body.len() + 1);
        text.push_str(&self.subject);
        if !self.subject.is_empty() && !self.body.is_empty() {
            text.push('\n');
        }
        text.push_str(&self.body);
        text
    }

    /// Everything Windows is told about this message besides its text.
    ///
    /// A field with nothing in it is left out rather than handed over blank.
    /// An empty string is a value, and it puts "from: nobody" in a column
    /// where showing nothing at all is the honest answer.
    pub fn properties(&self) -> Vec<(PROPERTYKEY, Value)> {
        let words = [
            (PKEY_Subject, self.subject.as_str()),
            (PKEY_ItemNameDisplay, self.display_name()),
            (PKEY_Message_FromAddress, self.from.as_str()),
            (PKEY_Message_ToAddress, self.to.as_str()),
            (PKEY_Message_CcAddress, self.cc.as_str()),
            (PKEY_Kind, KIND),
        ];

        let mut properties: Vec<(PROPERTYKEY, Value)> = words
            .into_iter()
            .filter(|(_, value)| !value.trim().is_empty())
            .map(|(key, value)| (key, Value::Text(value.to_string())))
            .collect();

        // Measured in the text that is actually handed over, not in whatever
        // the message weighed on the server. This is the only size the indexer
        // can check its own work against.
        properties.push((
            PKEY_Size,
            Value::Number(self.searchable_text().len() as u64),
        ));

        if let Some(sent) = self.sent {
            properties.push((PKEY_Message_DateReceived, Value::Moment(sent)));
        }

        properties
    }
}

/// How far ahead of the start of 1601 the start of 1970 is, in seconds.
const SECONDS_FROM_1601_TO_1970: i64 = 11_644_473_600;

/// How many of Windows' time units make one second.
const TICKS_PER_SECOND: i64 = 10_000_000;

/// Turn a moment the cache stores into the count Windows keeps time in.
///
/// `None` when the moment is outside what Windows can hold. A date header can
/// say anything at all, and a moment that wrapped around would file a message
/// at a plausible looking wrong time, which is harder to notice and harder to
/// explain than a message filed with no time.
pub fn windows_ticks(seconds_since_1970: i64) -> Option<u64> {
    seconds_since_1970
        .checked_add(SECONDS_FROM_1601_TO_1970)
        .filter(|since_1601| *since_1601 >= 0)
        .and_then(|since_1601| since_1601.checked_mul(TICKS_PER_SECOND))
        .and_then(|ticks| u64::try_from(ticks).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Storage::EnhancedStorage::{
        PKEY_ItemNameDisplay, PKEY_Kind, PKEY_Message_CcAddress, PKEY_Message_DateReceived,
        PKEY_Message_FromAddress, PKEY_Message_ToAddress, PKEY_Size, PKEY_Subject,
    };

    fn a_message() -> Message {
        Message {
            uid: 4211,
            subject: "Quarterly report".to_string(),
            from: "a@example.com".to_string(),
            to: "b@example.com".to_string(),
            cc: String::new(),
            sent: Some(1_700_000_000),
            body: "The figures are attached.".to_string(),
        }
    }

    #[test]
    fn test_a_moment_is_turned_into_the_count_windows_keeps_time_in() {
        // Windows counts ten millionths of a second since the start of 1601
        // and the cache counts seconds since the start of 1970. Getting the
        // offset or the scale wrong puts every message in the wrong century,
        // which sorts and filters wrongly everywhere without ever looking like
        // an error.
        assert_eq!(windows_ticks(0), Some(116_444_736_000_000_000));
        assert_eq!(windows_ticks(1_700_000_000), Some(133_444_736_000_000_000));
    }

    #[test]
    fn test_a_moment_windows_cannot_hold_is_refused_rather_than_wrapped_around() {
        // A date header can say anything at all, and the cache stores what the
        // server wrote. Wrapping would file a message at a plausible looking
        // wrong time, which is worse than filing it with no time.
        assert_eq!(windows_ticks(-11_644_473_601), None, "before 1601");
        assert_eq!(windows_ticks(i64::MAX), None, "too far ahead to count");
        assert_eq!(windows_ticks(i64::MIN), None, "too far back to count");
        // And the first moment Windows can count is still countable.
        assert_eq!(windows_ticks(-11_644_473_600), Some(0));
    }

    #[test]
    fn test_a_message_offers_every_property_it_has_a_value_for() {
        // These are the columns somebody sees in an Explorer result and the
        // fields "from:" and "subject:" searches look in. A property left out
        // is not a cosmetic loss: the message stops being findable that way,
        // and there is nothing on screen to say why.
        let offered: Vec<_> = a_message()
            .properties()
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        for expected in [
            PKEY_Subject,
            PKEY_ItemNameDisplay,
            PKEY_Message_FromAddress,
            PKEY_Message_ToAddress,
            PKEY_Message_DateReceived,
            PKEY_Size,
            PKEY_Kind,
        ] {
            assert!(
                offered.contains(&expected),
                "{:?} pid {} was not offered",
                expected.fmtid,
                expected.pid
            );
        }
    }

    #[test]
    fn test_an_empty_field_is_left_out_rather_than_handed_over_blank() {
        // A blank value is not the same as no value. Writing one puts an empty
        // string into the index where a reader expects an address, and an
        // Explorer column then shows a message as being from nobody instead of
        // showing nothing at all.
        let offered: Vec<_> = a_message()
            .properties()
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        assert!(
            !offered.contains(&PKEY_Message_CcAddress),
            "an empty carbon copy field was still offered"
        );
    }

    #[test]
    fn test_a_message_with_no_readable_date_still_offers_its_other_properties() {
        // The date column in the cache is text, and text from a mail server is
        // sometimes not a date at all. Losing the subject of every message with
        // an odd Date header would be a much bigger loss than losing the date.
        let undated = Message {
            sent: None,
            ..a_message()
        };
        let offered: Vec<_> = undated
            .properties()
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        assert!(!offered.contains(&PKEY_Message_DateReceived));
        assert!(offered.contains(&PKEY_Subject));
    }

    #[test]
    fn test_the_kind_says_email_so_windows_files_it_with_the_mail() {
        // System.Kind is what puts an item under "Email" in the Explorer
        // search filters. Without it a message is filed as unclassified and
        // never appears when somebody narrows a search to mail.
        let kind = a_message()
            .properties()
            .into_iter()
            .find(|(key, _)| *key == PKEY_Kind)
            .map(|(_, value)| value);

        assert_eq!(kind, Some(Value::Text("email".to_string())));
    }

    #[test]
    fn test_the_display_name_falls_back_to_words_rather_than_being_blank() {
        // A message with no subject is ordinary. Its display name is what
        // Explorer shows as the item's name, and an empty one leaves a row
        // somebody cannot read or tell apart from its neighbours.
        let unsubjected = Message {
            subject: String::new(),
            ..a_message()
        };

        let name = unsubjected
            .properties()
            .into_iter()
            .find(|(key, _)| *key == PKEY_ItemNameDisplay)
            .map(|(_, value)| value);

        assert_eq!(name, Some(Value::Text(NO_SUBJECT.to_string())));
        assert!(!NO_SUBJECT.is_empty());
    }

    #[test]
    fn test_the_searchable_text_carries_the_subject_and_the_body() {
        // The text chunks are what a plain word search actually looks at. The
        // subject is included as text as well as being a property, because a
        // search for a word in the subject line should find the message
        // whether or not the person thought to search a subject field.
        let text = a_message().searchable_text();

        assert!(text.contains("Quarterly report"), "{text}");
        assert!(text.contains("The figures are attached."), "{text}");
    }

    #[test]
    fn test_a_message_is_measured_by_the_text_the_indexer_will_read() {
        // The indexer asks how big an item is before deciding what to do with
        // it. Reporting zero for every message makes a mailbox look empty in
        // any view sorted or filtered by size.
        let size = a_message()
            .properties()
            .into_iter()
            .find(|(key, _)| *key == PKEY_Size)
            .map(|(_, value)| value);

        assert_eq!(
            size,
            Some(Value::Number(a_message().searchable_text().len() as u64))
        );
    }
}

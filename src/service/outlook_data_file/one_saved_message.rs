//! One message Outlook saved as a file of its own: a `.msg`.
//!
//! Not yet written: the failing cases below come first.

use super::HowMuchToAllow;
use std::io::{Read, Seek};

/// How a saved Outlook message begins: the eight bytes every Compound File
/// Binary container opens with.
pub const HOW_A_SAVED_MESSAGE_BEGINS: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// One saved message, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedOutlookMessage {
    /// The message, as the bytes of one message.
    pub mail: Vec<u8>,
}

/// Why a saved message was not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyItWasNotRead {
    NotAnOutlookMessage,
    AnotherKind(&'static str),
    NotAKindThisProgramKeeps,
    DamagedPartway,
    TooLarge,
}

impl WhyItWasNotRead {
    /// What to say about it.
    pub fn sentence(&self) -> String {
        unimplemented!("13-47 task 3")
    }
}

impl From<WhyItWasNotRead> for crate::common::Error {
    fn from(why: WhyItWasNotRead) -> Self {
        crate::common::Error::InPlainWords(why.sentence())
    }
}

/// Read one saved message.
pub fn read<R: Read + Seek>(
    _from: R,
    _allowed: HowMuchToAllow,
) -> Result<SavedOutlookMessage, WhyItWasNotRead> {
    unimplemented!("13-47 task 3")
}

#[cfg(test)]
mod tests {
    use super::super::{
        BODY, CODE_PAGE_OF_THE_TEXT, DISPLAY_NAME, EMAIL_ADDRESS, MESSAGE_CLASS, RECIPIENT_KIND,
        SENDER_ADDRESS, SENDER_ADDRESS_KIND, SENDER_NAME, SENDER_SMTP_ADDRESS, SENT_AT,
        SMTP_ADDRESS, SUBJECT,
    };
    use super::*;
    use crate::common::types::EmailAddress;
    use std::cell::Cell;
    use std::io::{Cursor, SeekFrom, Write};
    use std::rc::Rc;

    // ── Building a saved message by hand ────────────────────────────────────
    //
    // This is the format as MS-OXMSG describes it, read on learn.microsoft.com
    // on 2026-10-03: a property stream named `__properties_version1.0` in each
    // storage, a 32-byte header at the top and an 8-byte one in a recipient's
    // storage, then one 16-byte entry a property, with the value of anything
    // longer than eight bytes in a stream of its own named for its tag. Four
    // real Outlook files read the same way, and none is copied in here.

    /// One value as a saved message holds it.
    enum Held {
        /// Text in Unicode, written as Outlook writes it, two bytes a letter.
        Unicode(&'static str),
        /// Text one byte a letter, in whichever alphabet the message names.
        EightBit(&'static [u8]),
        /// A whole number.
        Whole(i32),
        /// A moment, in Outlook's own count.
        Moment(i64),
    }

    type Properties = Vec<(u16, Held)>;

    /// A saved message to be built.
    #[derive(Default)]
    struct ASavedMessage {
        top: Properties,
        recipients: Vec<Properties>,
    }

    /// The type numbers MS-OXCDATA 2.11.1 gives each kind of value.
    const UNICODE: u16 = 0x001F;
    const EIGHT_BIT: u16 = 0x001E;
    const WHOLE: u16 = 0x0003;
    const MOMENT: u16 = 0x0040;

    /// A saved message written into a container in memory, by the writer the
    /// same package carries.
    fn written(saved: &ASavedMessage) -> Vec<u8> {
        let mut file =
            cfb::CompoundFile::create(Cursor::new(Vec::new())).expect("a container in memory");
        let recipients = u32::try_from(saved.recipients.len()).expect("a few recipients");
        let mut header = vec![0u8; 8];
        header.extend_from_slice(&recipients.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&recipients.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&[0u8; 8]);
        properties_written(&mut file, "", header, &saved.top);
        for (number, recipient) in saved.recipients.iter().enumerate() {
            let storage = format!("/__recip_version1.0_#{number:08X}");
            file.create_storage(&storage)
                .expect("a recipient's storage");
            properties_written(&mut file, &storage, vec![0u8; 8], recipient);
        }
        file.flush().expect("the container written");
        file.into_inner().into_inner()
    }

    /// One storage's property stream, and a stream for each value too long
    /// to sit in its entry.
    fn properties_written(
        file: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
        storage: &str,
        header: Vec<u8>,
        properties: &Properties,
    ) {
        let mut stream = header;
        for (id, held) in properties {
            let (kind, value) = match held {
                Held::Whole(number) => (WHOLE, Fixed(i64::from(*number).to_le_bytes())),
                Held::Moment(steps) => (MOMENT, Fixed(steps.to_le_bytes())),
                Held::Unicode(text) => (
                    UNICODE,
                    Streamed(text.encode_utf16().flat_map(u16::to_le_bytes).collect(), 2),
                ),
                Held::EightBit(bytes) => (EIGHT_BIT, Streamed(bytes.to_vec(), 1)),
            };
            let tag = (u32::from(*id) << 16) | u32::from(kind);
            stream.extend_from_slice(&tag.to_le_bytes());
            stream.extend_from_slice(&6u32.to_le_bytes());
            match value {
                Fixed(bytes) => stream.extend_from_slice(&bytes),
                Streamed(bytes, terminator) => {
                    let size = u32::try_from(bytes.len()).expect("a short value") + terminator;
                    stream.extend_from_slice(&size.to_le_bytes());
                    stream.extend_from_slice(&0u32.to_le_bytes());
                    file.create_stream(format!("{storage}/__substg1.0_{tag:08X}"))
                        .expect("a value's stream")
                        .write_all(&bytes)
                        .expect("a value written");
                }
            }
        }
        file.create_stream(format!("{storage}/__properties_version1.0"))
            .expect("the property stream")
            .write_all(&stream)
            .expect("the properties written");
    }

    /// Where a value goes: in its entry, or in a stream of its own with how
    /// much its stated size adds for the terminator MS-OXMSG counts.
    enum WhereItGoes {
        Fixed([u8; 8]),
        Streamed(Vec<u8>, u32),
    }
    use WhereItGoes::{Fixed, Streamed};

    /// The way Outlook counts to a moment: hundred-nanosecond steps from the
    /// first of January 1601.
    fn steps_at(rfc3339: &str) -> i64 {
        let when = chrono::DateTime::parse_from_rfc3339(rfc3339).expect("a moment a test named");
        (when.timestamp() + 11_644_473_600) * 10_000_000
    }

    /// A mail message's class, the one thing every saved message here has.
    fn mail() -> (u16, Held) {
        (MESSAGE_CLASS, Held::Unicode("IPM.Note"))
    }

    /// What reading a file that is all here says.
    fn read_all_of(bytes: Vec<u8>) -> Result<SavedOutlookMessage, WhyItWasNotRead> {
        read(Cursor::new(bytes), HowMuchToAllow::default())
    }

    /// The message a saved one became, read back by this program's own reader.
    fn read_back(saved: &ASavedMessage) -> crate::service::mime::ParsedMessage {
        let read = read_all_of(written(saved)).expect("the saved message was read");
        crate::service::mime::parse(&read.mail).expect("what came out is a message")
    }

    fn person(address: &str, name: &str) -> EmailAddress {
        EmailAddress::new(address.to_string(), Some(name.to_string()))
    }

    // ── Reading one ─────────────────────────────────────────────────────────

    #[test]
    fn test_a_saved_outlook_message_reads_as_the_message_it_holds() {
        // The whole promise: a message somebody dragged out of Outlook comes
        // back as the message it was, through the same mapping a data file's
        // messages go through, so the two cannot come to read differently.
        // The body ends in the null Outlook may write after Unicode text,
        // which is not a letter and is taken off.
        let saved = ASavedMessage {
            top: vec![
                mail(),
                (SUBJECT, Held::Unicode("\u{1}\u{4}Re: The engine")),
                (
                    BODY,
                    Held::Unicode("The engine weaves algebraic patterns.\0"),
                ),
                (SENDER_NAME, Held::Unicode("Ada Lovelace")),
                (SENDER_SMTP_ADDRESS, Held::Unicode("ada@example.com")),
                (SENT_AT, Held::Moment(steps_at("2026-08-24T10:00:00Z"))),
            ],
            recipients: vec![
                vec![
                    (DISPLAY_NAME, Held::Unicode("Charles Babbage")),
                    (SMTP_ADDRESS, Held::Unicode("charles@example.com")),
                    (RECIPIENT_KIND, Held::Whole(1)),
                ],
                vec![
                    (DISPLAY_NAME, Held::Unicode("Mary Somerville")),
                    (SMTP_ADDRESS, Held::Unicode("mary@example.com")),
                    (RECIPIENT_KIND, Held::Whole(2)),
                ],
            ],
        };

        let message = read_back(&saved);

        assert_eq!(message.subject, "Re: The engine");
        assert_eq!(
            message.from,
            vec![person("ada@example.com", "Ada Lovelace")]
        );
        assert_eq!(
            message.to,
            vec![person("charles@example.com", "Charles Babbage")]
        );
        assert_eq!(
            message.cc,
            vec![person("mary@example.com", "Mary Somerville")]
        );
        assert_eq!(
            message.body_plain.as_deref(),
            Some("The engine weaves algebraic patterns.")
        );
        assert_eq!(message.date.as_deref(), Some("2026-08-24T10:00:00Z"));
    }

    #[test]
    fn test_eight_bit_text_is_read_in_the_alphabet_the_file_names() {
        // An older saved message holds its text one byte a letter, in the
        // alphabet the computer that wrote it was set to, and names that
        // alphabet among its fixed values. Read in Western European instead,
        // this Russian subject would come out as "Ïðèâåò". The recipients sit
        // beside the message and are read in the message's alphabet too.
        let saved = ASavedMessage {
            top: vec![
                (MESSAGE_CLASS, Held::EightBit(b"IPM.Note")),
                (CODE_PAGE_OF_THE_TEXT, Held::Whole(1251)),
                (SUBJECT, Held::EightBit(b"\xcf\xf0\xe8\xe2\xe5\xf2")),
                (
                    BODY,
                    Held::EightBit(b"\xc4\xee\xe1\xf0\xfb\xe9 \xe4\xe5\xed\xfc\0"),
                ),
            ],
            recipients: vec![vec![
                (DISPLAY_NAME, Held::EightBit(b"\xc8\xe2\xe0\xed")),
                (EMAIL_ADDRESS, Held::EightBit(b"ivan@example.com")),
                (RECIPIENT_KIND, Held::Whole(1)),
            ]],
        };

        let message = read_back(&saved);

        assert_eq!(message.subject, "Привет");
        assert_eq!(message.body_plain.as_deref(), Some("Добрый день"));
        assert_eq!(message.to, vec![person("ivan@example.com", "Иван")]);
    }

    #[test]
    fn test_an_exchange_sender_with_no_internet_address_keeps_the_name_and_the_address_written() {
        // Mail that never left a company names its sender by an address only
        // that company's server understands, with no internet address beside
        // it. The data file's rule keeps both rather than losing the name,
        // and a saved message goes through the same rule.
        let saved = ASavedMessage {
            top: vec![
                mail(),
                (SUBJECT, Held::Unicode("The quarterly figures")),
                (SENDER_NAME, Held::Unicode("Ada Lovelace")),
                (
                    SENDER_ADDRESS,
                    Held::Unicode("/O=ENGINE/OU=EXCHANGE/CN=RECIPIENTS/CN=ADA"),
                ),
                (SENDER_ADDRESS_KIND, Held::Unicode("EX")),
            ],
            recipients: Vec::new(),
        };

        let message = read_back(&saved);

        assert_eq!(message.from.len(), 1, "{:?}", message.from);
        assert_eq!(message.from[0].name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(
            message.from[0].address,
            "/O=ENGINE/OU=EXCHANGE/CN=RECIPIENTS/CN=ADA"
        );
    }

    // ── Refusing one ────────────────────────────────────────────────────────

    #[test]
    fn test_a_file_that_is_not_a_container_is_not_an_outlook_message() {
        // A saved message somebody renamed, an Outlook data file, an empty
        // file: none of them begins the way a container does, and each is
        // told it is not a saved message rather than that it is damaged.
        for bytes in [
            b"From: ada@example.com\r\nSubject: Hello\r\n\r\nHello.\r\n".to_vec(),
            b"!BDN and the rest of an Outlook data file".to_vec(),
            Vec::new(),
        ] {
            let refused = read_all_of(bytes);
            assert_eq!(refused, Err(WhyItWasNotRead::NotAnOutlookMessage));
            assert_eq!(
                refused.unwrap_err().sentence(),
                "That file is not a saved Outlook message."
            );
        }
    }

    #[test]
    fn test_a_container_with_no_properties_is_not_an_outlook_message() {
        // Office's older documents are the same kind of container. One with
        // no property stream at its top is not a message, whatever it is.
        let mut file =
            cfb::CompoundFile::create(Cursor::new(Vec::new())).expect("a container in memory");
        file.create_stream("/WordDocument")
            .expect("a stream")
            .write_all(b"not a message")
            .expect("written");
        file.flush().expect("the container written");

        assert_eq!(
            read_all_of(file.into_inner().into_inner()),
            Err(WhyItWasNotRead::NotAnOutlookMessage)
        );
    }

    #[test]
    fn test_a_container_that_stops_partway_is_damaged() {
        // A copy cut short: it begins like a saved message and does not hold
        // what it says it holds. Told it is not a saved message at all,
        // somebody goes looking for a different file; told it is damaged,
        // they fetch another copy of this one.
        let words = "The engine weaves algebraic patterns. ".repeat(2_000);
        let saved = ASavedMessage {
            top: vec![
                mail(),
                (SUBJECT, Held::Unicode("A long one")),
                (BODY, Held::Unicode(Box::leak(words.into_boxed_str()))),
            ],
            recipients: Vec::new(),
        };
        let mut bytes = written(&saved);
        bytes.truncate(bytes.len() / 2);

        let refused = read_all_of(bytes);

        assert_eq!(refused, Err(WhyItWasNotRead::DamagedPartway));
        assert_eq!(
            refused.unwrap_err().sentence(),
            "That saved Outlook message is damaged partway through, so it was not read."
        );
    }

    /// A reader that counts every byte taken from what it wraps.
    struct Counted<R> {
        inner: R,
        taken: Rc<Cell<u64>>,
    }

    impl<R: Read> Read for Counted<R> {
        fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
            let came = self.inner.read(into)?;
            self.taken.set(self.taken.get() + came as u64);
            Ok(came)
        }
    }

    impl<R: Seek> Seek for Counted<R> {
        fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(to)
        }
    }

    #[test]
    fn test_more_than_the_limit_is_refused_and_never_held() {
        // A stranger's file can hold a body of any size. Reading it whole and
        // then finding it too large is the failure the limit exists to stop,
        // so what is counted is what was taken from the file: an eight
        // megabyte body against a 64 kilobyte limit must be refused having
        // read far less than the body.
        const LIMIT: u64 = 64 * 1024;
        let body = "a".repeat(4 * 1024 * 1024);
        let saved = ASavedMessage {
            top: vec![
                mail(),
                (SUBJECT, Held::Unicode("Too much")),
                (BODY, Held::Unicode(Box::leak(body.into_boxed_str()))),
            ],
            recipients: Vec::new(),
        };
        let bytes = written(&saved);
        let taken = Rc::new(Cell::new(0));
        let allowed = HowMuchToAllow {
            most_one_item_comes_to: LIMIT,
            ..HowMuchToAllow::default()
        };

        let refused = read(
            Counted {
                inner: Cursor::new(bytes),
                taken: Rc::clone(&taken),
            },
            allowed,
        );

        assert_eq!(refused, Err(WhyItWasNotRead::TooLarge));
        assert_eq!(
            refused.unwrap_err().sentence(),
            "That saved Outlook message is larger than Wixen Mail will read, so it was left \
             where it is."
        );
        assert!(
            taken.get() < 2 * 1024 * 1024,
            "{} bytes were taken from a file refused at {LIMIT}",
            taken.get()
        );
    }

    #[test]
    fn test_an_appointment_saved_as_a_file_is_refused_by_name() {
        // Outlook saves its other kinds as .msg files too. Each is refused by
        // what it is, so somebody holding an appointment knows why it did
        // not arrive among their messages, and the class is sorted by the
        // data file's own rule, so the two readers agree on what is mail.
        for (class, kind) in [
            ("IPM.Appointment", "appointment"),
            ("IPM.Contact", "contact"),
            ("IPM.Task", "task"),
            ("IPM.StickyNote", "note"),
        ] {
            let saved = ASavedMessage {
                top: vec![
                    (MESSAGE_CLASS, Held::Unicode(class)),
                    (SUBJECT, Held::Unicode("Lunch")),
                ],
                recipients: Vec::new(),
            };
            assert_eq!(
                read_all_of(written(&saved)),
                Err(WhyItWasNotRead::AnotherKind(kind)),
                "for {class}"
            );
        }
        assert_eq!(
            WhyItWasNotRead::AnotherKind("appointment").sentence(),
            "That is an Outlook appointment, not a message. Wixen Mail reads saved messages \
             from .msg files."
        );
    }

    #[test]
    fn test_a_saved_item_of_no_kind_this_program_keeps_is_refused_as_such() {
        // A meeting request, a delivery report: Outlook items that are none
        // of the kinds this program keeps, and none of the four it can name.
        for class in ["IPM.Schedule.Meeting.Request", "REPORT.IPM.Note.NDR"] {
            let saved = ASavedMessage {
                top: vec![(MESSAGE_CLASS, Held::Unicode(class))],
                recipients: Vec::new(),
            };
            let refused = read_all_of(written(&saved));
            assert_eq!(
                refused,
                Err(WhyItWasNotRead::NotAKindThisProgramKeeps),
                "for {class}"
            );
            assert_eq!(
                refused.unwrap_err().sentence(),
                "That is a saved Outlook item of a kind Wixen Mail does not keep."
            );
        }
    }

    #[test]
    fn test_every_refusal_is_a_sentence_and_names_no_machinery() {
        // Each is read aloud. A fragment runs into whatever is spoken next,
        // and a sentence naming the container or its streams tells somebody
        // about this program's insides instead of about their message.
        for why in [
            WhyItWasNotRead::NotAnOutlookMessage,
            WhyItWasNotRead::AnotherKind("appointment"),
            WhyItWasNotRead::AnotherKind("contact"),
            WhyItWasNotRead::AnotherKind("task"),
            WhyItWasNotRead::AnotherKind("note"),
            WhyItWasNotRead::NotAKindThisProgramKeeps,
            WhyItWasNotRead::DamagedPartway,
            WhyItWasNotRead::TooLarge,
        ] {
            let said = why.sentence();
            assert!(said.ends_with('.'), "not a sentence: {said}");
            let lowered = said.to_lowercase();
            for machinery in ["property", "stream", "storage", "compound", "cfb", "mapi"] {
                assert!(!lowered.contains(machinery), "{machinery} in: {said}");
            }
            assert_eq!(crate::common::Error::from(why).to_string(), said);
        }
    }
}

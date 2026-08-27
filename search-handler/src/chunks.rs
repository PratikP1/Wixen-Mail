//! What the filter hands the indexer, in the order it goes.
//!
//! Windows asks for one chunk at a time and then asks that chunk for either
//! words or a value. That is a small state machine with several ways to get it
//! subtly wrong, and every one of those ways shows up as a mailbox that indexes
//! badly rather than as a crash. So the machine lives here as plain Rust with
//! tests, and the COM object in [`crate::com`] does nothing but translate.

use crate::record::{Message, Value, windows_ticks};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Storage::EnhancedStorage::{
    PKEY_Search_Contents, PKEY_Search_UrlToIndex, PKEY_Search_UrlToIndexWithModificationTime,
};

/// One thing handed to the indexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Chunk {
    /// Words for the full text index.
    Text(String),
    /// A named value, such as the sender or the date.
    Property { key: PROPERTYKEY, value: Value },
}

/// Something inside a container, on its way to being enumerated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Child {
    pub url: String,
    /// When it last changed, in seconds since the start of 1970.
    pub modified: Option<i64>,
}

/// Everything the indexer is told about one message.
///
/// Properties first, then the words. The indexer takes chunks in order, so
/// this way an item is already described by the time its body arrives, and a
/// body that fails part way through still leaves something findable.
pub fn for_message(message: &Message) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = message
        .properties()
        .into_iter()
        .map(|(key, value)| Chunk::Property { key, value })
        .collect();

    let text = message.searchable_text();
    if !text.trim().is_empty() {
        chunks.push(Chunk::Text(text));
    }

    chunks
}

/// Everything the indexer is told about what is inside a container.
///
/// A child is named with its change time when there is one, because that lets
/// the indexer skip an item it already has without asking again. A child whose
/// time cannot be read is still named, without one: indexing it more often
/// than needed costs time, and leaving it out means somebody's message is
/// never findable at all.
pub fn for_children(children: &[Child]) -> Vec<Chunk> {
    children
        .iter()
        // A time Windows cannot hold counts as no time. The check belongs here
        // rather than where the value is built, because the chunk announces
        // which property it carries before the value is asked for, and the two
        // have to agree.
        .map(
            |child| match child.modified.filter(|when| windows_ticks(*when).is_some()) {
                Some(modified) => Chunk::Property {
                    key: PKEY_Search_UrlToIndexWithModificationTime,
                    value: Value::UrlAndMoment {
                        url: child.url.clone(),
                        modified,
                    },
                },
                None => Chunk::Property {
                    key: PKEY_Search_UrlToIndex,
                    value: Value::Text(child.url.clone()),
                },
            },
        )
        .collect()
}

/// What the indexer is told about the chunk it has just moved to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marker {
    /// The chunk's number. Never zero, because zero means "no chunk".
    pub id: u32,
    /// Which property this chunk carries, or the contents property for words.
    pub attribute: PROPERTYKEY,
    /// Whether to ask this chunk for words or for a value.
    pub is_text: bool,
}

/// The answer to asking a chunk for something.
///
/// Three answers rather than an option, because Windows distinguishes "this
/// chunk has none of that" from "you have had all of it" and acts differently
/// on each. Folding them together makes the indexer either give up early or
/// ask again forever.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken<T> {
    Some(T),
    AlreadyGiven,
    WrongKind,
}

/// A cursor over the chunks, shaped the way Windows reads them.
#[derive(Debug)]
pub struct Walk {
    chunks: Vec<Chunk>,
    /// Which chunk is current. Nothing is current until the first move.
    at: Option<usize>,
    /// The current chunk's words, in the sixteen bit units Windows counts in.
    text: Vec<u16>,
    /// How many of those units have already been handed over.
    text_taken: usize,
    /// Whether the current chunk's value has already been handed over.
    value_taken: bool,
}

/// The first half of a character that takes two units to write.
const LEADING_HALF: std::ops::RangeInclusive<u16> = 0xD800..=0xDBFF;

impl Walk {
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            chunks,
            at: None,
            text: Vec::new(),
            text_taken: 0,
            value_taken: false,
        }
    }

    /// Move to the next chunk. `None` once there are none left, every time.
    pub fn advance(&mut self) -> Option<Marker> {
        let next = match self.at {
            None => 0,
            Some(at) => at + 1,
        };

        let (attribute, is_text, text) = match self.chunks.get(next)? {
            Chunk::Text(words) => (PKEY_Search_Contents, true, words.encode_utf16().collect()),
            Chunk::Property { key, .. } => (*key, false, Vec::new()),
        };

        self.at = Some(next);
        self.text = text;
        self.text_taken = 0;
        self.value_taken = false;

        Some(Marker {
            id: chunk_number(next),
            attribute,
            is_text,
        })
    }

    /// Take as many of the current chunk's words as will fit in `room` units.
    ///
    /// `room` is counted the way Windows counts a buffer, in sixteen bit
    /// units rather than characters, and a character outside the basic set
    /// takes two of them. A pair is never split, so a buffer too small to hold
    /// the next character takes nothing rather than half of it. That is also
    /// the answer to a buffer of no room at all: nothing fits, and nothing is
    /// lost. Neither case comes up with the buffers the indexer really uses.
    pub fn take_text(&mut self, room: usize) -> Taken<Vec<u16>> {
        if !matches!(self.current(), Some(Chunk::Text(_))) {
            return Taken::WrongKind;
        }
        if self.text_taken >= self.text.len() {
            return Taken::AlreadyGiven;
        }

        let left = self.text.len() - self.text_taken;
        let mut fits = room.min(left);
        if fits > 0 && fits < left && LEADING_HALF.contains(&self.text[self.text_taken + fits - 1])
        {
            fits -= 1;
        }

        let piece = self.text[self.text_taken..self.text_taken + fits].to_vec();
        self.text_taken += fits;
        Taken::Some(piece)
    }

    /// Take the current chunk's value, once.
    pub fn take_value(&mut self) -> Taken<Value> {
        let value = match self.current() {
            Some(Chunk::Property { value, .. }) => value.clone(),
            _ => return Taken::WrongKind,
        };

        match self.value_taken {
            true => Taken::AlreadyGiven,
            false => {
                self.value_taken = true;
                Taken::Some(value)
            }
        }
    }

    fn current(&self) -> Option<&Chunk> {
        self.at.and_then(|at| self.chunks.get(at))
    }
}

/// What a chunk at this position is called.
///
/// One based, because Windows keeps zero for "no chunk" and a chunk numbered
/// zero is one the indexer cannot refer back to.
const fn chunk_number(position: usize) -> u32 {
    position as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Message;
    use windows::Win32::Storage::EnhancedStorage::{
        PKEY_Search_Contents, PKEY_Search_UrlToIndex, PKEY_Search_UrlToIndexWithModificationTime,
        PKEY_Subject,
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

    fn text_of(chunks: &[Chunk]) -> String {
        chunks
            .iter()
            .filter_map(|chunk| match chunk {
                Chunk::Text(text) => Some(text.as_str()),
                Chunk::Property { .. } => None,
            })
            .collect()
    }

    #[test]
    fn test_a_message_hands_over_its_properties_before_its_words() {
        // The indexer reads chunks in the order they are given. Properties
        // first is what Microsoft's own samples do, and it means the item is
        // already described by the time the body arrives, so a body that fails
        // part way through still leaves a findable item behind.
        let chunks = for_message(&a_message());
        let first_text = chunks
            .iter()
            .position(|chunk| matches!(chunk, Chunk::Text(_)))
            .expect("a message with a body should produce text");
        let last_property = chunks
            .iter()
            .rposition(|chunk| matches!(chunk, Chunk::Property { .. }))
            .expect("a message should produce properties");

        assert!(last_property < first_text, "{chunks:#?}");
        assert!(text_of(&chunks).contains("Quarterly report"));
        assert!(text_of(&chunks).contains("The figures are attached."));
    }

    #[test]
    fn test_a_message_with_nothing_to_read_still_describes_itself() {
        // An empty message is ordinary: a calendar invitation with no body, or
        // a message whose body has not been fetched yet. Handing over an empty
        // text chunk makes the indexer do work for nothing, and dropping the
        // properties too would leave the message unfindable by sender or date.
        let empty = Message {
            subject: String::new(),
            body: String::new(),
            ..a_message()
        };
        let chunks = for_message(&empty);

        assert!(
            !chunks.iter().any(|chunk| matches!(chunk, Chunk::Text(_))),
            "an empty message produced a text chunk: {chunks:#?}"
        );
        assert!(
            chunks
                .iter()
                .any(|chunk| matches!(chunk, Chunk::Property { .. }))
        );
    }

    #[test]
    fn test_a_container_names_each_child_together_with_when_it_changed() {
        // Microsoft's page says giving the time alongside the url lets the
        // indexer decide whether it needs the item at all, without a second
        // round trip through the accessor. On a mailbox with thousands of
        // messages that is the difference between a crawl and a stall.
        let chunks = for_children(&[Child {
            url: "wixen-mail://localhost/work/INBOX/7".to_string(),
            modified: Some(1_700_000_000),
        }]);

        assert_eq!(
            chunks,
            vec![Chunk::Property {
                key: PKEY_Search_UrlToIndexWithModificationTime,
                value: Value::UrlAndMoment {
                    url: "wixen-mail://localhost/work/INBOX/7".to_string(),
                    modified: 1_700_000_000,
                },
            }]
        );
    }

    #[test]
    fn test_a_child_whose_time_is_unknown_is_still_named() {
        // The cache keeps the date as the server wrote it, so some rows have
        // no time anybody can read. Skipping those children would hide the
        // message completely, which is a far worse answer than indexing it
        // once more often than needed.
        let chunks = for_children(&[Child {
            url: "wixen-mail://localhost/work/INBOX/7".to_string(),
            modified: None,
        }]);

        assert_eq!(
            chunks,
            vec![Chunk::Property {
                key: PKEY_Search_UrlToIndex,
                value: Value::Text("wixen-mail://localhost/work/INBOX/7".to_string()),
            }]
        );
    }

    #[test]
    fn test_a_child_whose_time_windows_cannot_hold_is_named_without_one() {
        // A date header can say the year 200000. The time would be dropped
        // later, when it is turned into the count Windows keeps, and by then
        // the chunk already says it carries one. The indexer would be handed a
        // property whose value never arrives, and the message would go missing
        // rather than simply being indexed more often than needed.
        let chunks = for_children(&[Child {
            url: "wixen-mail://localhost/work/INBOX/7".to_string(),
            modified: Some(i64::MAX),
        }]);

        assert_eq!(
            chunks,
            vec![Chunk::Property {
                key: PKEY_Search_UrlToIndex,
                value: Value::Text("wixen-mail://localhost/work/INBOX/7".to_string()),
            }]
        );
    }

    #[test]
    fn test_a_container_that_enumerates_nothing_is_allowed_to_be_empty() {
        // An account with no folders yet, or a folder nothing has synced into.
        // This has to be a clean empty answer rather than an error, because
        // the indexer reads an error as a broken handler and can stop asking.
        assert_eq!(for_children(&[]), Vec::new());
    }

    #[test]
    fn test_every_chunk_gets_its_own_number_and_none_of_them_is_zero() {
        // The indexer uses the chunk number to tie text back to the chunk it
        // came from. Zero is reserved for "no chunk", and repeating a number
        // makes two different chunks look like one.
        let mut walk = Walk::new(for_message(&a_message()));
        let mut seen = Vec::new();

        while let Some(marker) = walk.advance() {
            assert_ne!(marker.id, 0);
            assert!(!seen.contains(&marker.id), "chunk {} came twice", marker.id);
            seen.push(marker.id);
        }

        assert!(seen.len() > 1, "only {} chunks", seen.len());
    }

    #[test]
    fn test_a_text_chunk_is_marked_as_text_and_a_property_chunk_is_not() {
        // This one flag decides whether the indexer asks for words or for a
        // value. Getting it the wrong way round means asking the wrong
        // question and being told there is nothing there.
        let mut walk = Walk::new(vec![
            Chunk::Property {
                key: PKEY_Subject,
                value: Value::Text("Quarterly report".to_string()),
            },
            Chunk::Text("The figures are attached.".to_string()),
        ]);

        let property = walk.advance().expect("first chunk");
        assert!(!property.is_text);
        assert_eq!(property.attribute, PKEY_Subject);

        let text = walk.advance().expect("second chunk");
        assert!(text.is_text);
        assert_eq!(
            text.attribute, PKEY_Search_Contents,
            "text chunks are the contents property"
        );
    }

    #[test]
    fn test_text_is_handed_over_in_pieces_that_fit_the_buffer_and_loses_nothing() {
        // The indexer supplies the buffer and it is usually much smaller than
        // a message body. Everything has to come out across the calls, in
        // order, exactly once.
        let mut walk = Walk::new(vec![Chunk::Text("abcdefg".to_string())]);
        walk.advance().expect("the text chunk");

        let mut rebuilt = String::new();
        loop {
            match walk.take_text(3) {
                Taken::Some(units) => rebuilt.push_str(&String::from_utf16_lossy(&units)),
                Taken::AlreadyGiven => break,
                Taken::WrongKind => panic!("a text chunk reported as not text"),
            }
        }

        assert_eq!(rebuilt, "abcdefg");
    }

    #[test]
    fn test_a_character_made_of_two_units_is_never_cut_in_half() {
        // Windows counts the buffer in sixteen bit units, and characters
        // outside the basic set take two of them. Splitting a pair puts a
        // lone half into the index, which is not text in any language and
        // makes the surrounding words unsearchable.
        let mut walk = Walk::new(vec![Chunk::Text("a\u{1F600}b".to_string())]);
        walk.advance().expect("the text chunk");

        let first = match walk.take_text(2) {
            Taken::Some(units) => units,
            other => panic!("{other:?}"),
        };
        assert_eq!(first.len(), 1, "the pair was cut to fill the buffer");

        let mut rebuilt = String::from_utf16_lossy(&first);
        while let Taken::Some(units) = walk.take_text(2) {
            rebuilt.push_str(&String::from_utf16_lossy(&units));
        }
        assert_eq!(rebuilt, "a\u{1F600}b");
    }

    #[test]
    fn test_a_buffer_with_no_room_takes_nothing_and_loses_nothing() {
        // A zero sized buffer is not something the indexer is expected to
        // send, and answering it by dropping the chunk would silently lose a
        // whole message body. Nothing fits, nothing is consumed.
        let mut walk = Walk::new(vec![Chunk::Text("abc".to_string())]);
        walk.advance().expect("the text chunk");

        assert_eq!(walk.take_text(0), Taken::Some(Vec::new()));
        assert_eq!(
            walk.take_text(8),
            Taken::Some("abc".encode_utf16().collect())
        );
    }

    #[test]
    fn test_a_buffer_too_small_for_the_next_character_takes_nothing_rather_than_half() {
        // One unit of room in front of a two unit character. Taking the first
        // half would put a meaningless value in the index and corrupt the rest
        // of the chunk; taking both would write past the end of a buffer
        // inside a Microsoft process. Neither is acceptable, so nothing goes.
        let mut walk = Walk::new(vec![Chunk::Text("\u{1F600}b".to_string())]);
        walk.advance().expect("the text chunk");

        assert_eq!(walk.take_text(1), Taken::Some(Vec::new()));
        // And nothing was consumed, so a bigger buffer still gets it all.
        let mut rebuilt = String::new();
        while let Taken::Some(units) = walk.take_text(8) {
            rebuilt.push_str(&String::from_utf16_lossy(&units));
        }
        assert_eq!(rebuilt, "\u{1F600}b");
    }

    #[test]
    fn test_text_that_has_all_been_given_says_so_rather_than_repeating_itself() {
        // The indexer keeps calling until it is told to stop. Returning the
        // last piece again would loop forever inside a Microsoft process.
        let mut walk = Walk::new(vec![Chunk::Text("abc".to_string())]);
        walk.advance().expect("the text chunk");

        assert!(matches!(walk.take_text(8), Taken::Some(_)));
        assert_eq!(walk.take_text(8), Taken::AlreadyGiven);
        assert_eq!(walk.take_text(8), Taken::AlreadyGiven);
    }

    #[test]
    fn test_a_value_is_handed_over_once_and_then_reported_as_spent() {
        // Same reason as the text above, and the same loop if it is wrong.
        let mut walk = Walk::new(vec![Chunk::Property {
            key: PKEY_Subject,
            value: Value::Text("Quarterly report".to_string()),
        }]);
        walk.advance().expect("the property chunk");

        assert_eq!(
            walk.take_value(),
            Taken::Some(Value::Text("Quarterly report".to_string()))
        );
        assert_eq!(walk.take_value(), Taken::AlreadyGiven);
    }

    #[test]
    fn test_asking_a_chunk_for_the_wrong_thing_says_which_rather_than_going_quiet() {
        // Windows has two different answers here, one meaning "this chunk has
        // none of that" and one meaning "you have had it all". They are not
        // interchangeable: the first tells the indexer to move on, the second
        // tells it to ask again with the other question.
        let mut text_walk = Walk::new(vec![Chunk::Text("abc".to_string())]);
        text_walk.advance().expect("the text chunk");
        assert_eq!(text_walk.take_value(), Taken::WrongKind);

        let mut value_walk = Walk::new(vec![Chunk::Property {
            key: PKEY_Subject,
            value: Value::Number(7),
        }]);
        value_walk.advance().expect("the property chunk");
        assert_eq!(value_walk.take_text(8), Taken::WrongKind);
    }

    #[test]
    fn test_nothing_can_be_read_before_the_first_chunk_has_been_asked_for() {
        // Windows requires a chunk to be selected before its contents can be
        // read. Answering from a chunk nobody selected would hand back the
        // wrong message's words with the right message's identity.
        let mut walk = Walk::new(for_message(&a_message()));

        assert_eq!(walk.take_text(8), Taken::WrongKind);
        assert_eq!(walk.take_value(), Taken::WrongKind);
    }

    #[test]
    fn test_walking_past_the_end_keeps_saying_there_is_no_more() {
        // The indexer stops on the first refusal, but a handler that answered
        // differently the second time would be a bug nobody could reproduce.
        let mut walk = Walk::new(vec![Chunk::Text("abc".to_string())]);

        assert!(walk.advance().is_some());
        assert!(walk.advance().is_none());
        assert!(walk.advance().is_none());
    }
}

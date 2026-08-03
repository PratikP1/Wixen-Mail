//! The five flags a mail server keeps on a message, spelled once.
//!
//! These names were written out at every place that reads or writes one, and
//! they have to agree: the name sent to the server, the name the reply is
//! matched against, and the name the cache turns into a column. Nothing checked
//! that they did.
//!
//! That is the kind of mistake that does not fail. Star a message and the list
//! writes its own column, so the star appears and the row reads back starred.
//! The name that went to the server is a separate string, and a typo in it
//! means the server was never told. The person finds out from another device,
//! days later, when the star is not there and there is nothing to say why.
//!
//! Constants rather than a set of values, because the same calls also carry
//! keywords a provider invented, which no fixed list can hold.

/// The message has been read.
pub const SEEN: &str = "\\Seen";

/// The message is flagged for attention, which is what starring sets.
pub const FLAGGED: &str = "\\Flagged";

/// The message has been answered.
pub const ANSWERED: &str = "\\Answered";

/// The message is a draft.
pub const DRAFT: &str = "\\Draft";

/// The message is marked for removal.
pub const DELETED: &str = "\\Deleted";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_names_are_the_ones_the_protocol_uses() {
        // Written out a second time on purpose, and only here. A typo in one of
        // these changes what reaches every mail server at once, so changing one
        // should mean changing this line as well.
        assert_eq!(SEEN, "\\Seen");
        assert_eq!(FLAGGED, "\\Flagged");
        assert_eq!(ANSWERED, "\\Answered");
        assert_eq!(DRAFT, "\\Draft");
        assert_eq!(DELETED, "\\Deleted");
    }

    #[test]
    fn test_a_flag_name_is_what_both_readers_accept() {
        // The point of the one definition. A name that the writer sends and the
        // readers do not recognise is a change that never leaves this computer.
        let message = crate::service::protocols::imap::ImapMessage {
            flags: vec![
                FLAGGED.to_string(),
                SEEN.to_string(),
                ANSWERED.to_string(),
                DRAFT.to_string(),
                DELETED.to_string(),
            ],
            ..Default::default()
        };

        assert!(message.flagged());
        assert!(message.seen());
        assert!(message.answered());
        assert!(message.draft());
        assert!(message.deleted());
    }
}

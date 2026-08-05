//! Working out who a reply goes to.
//!
//! Three different questions, and getting them confused sends mail to the wrong
//! people. Replying to a mailing list message should reach the list, because
//! that is what the sender asked for by setting `Reply-To` and what everyone in
//! the thread expects. Replying to one person on that list should reach only
//! them, and doing it by accident in the other direction is the mistake that
//! puts a private answer in front of two thousand strangers.
//!
//! So the modes are separate keys rather than one key that guesses, and each
//! one is named for what it does rather than for how it is spelled.

/// Which reply was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyMode {
    /// Where the sender asked replies to go.
    ///
    /// `Reply-To` when the sender set one, and `From` otherwise. On a mailing
    /// list that is the list, which is the answer people expect from the
    /// ordinary reply key.
    Default,
    /// Everyone the message reached, wherever replies go, and the person who
    /// wrote it.
    ///
    /// The author is included even when they asked for replies to go elsewhere,
    /// which on a mailing list means they get a copy of the answer to their own
    /// message. Decided rather than discovered.
    All,
    /// The person who wrote it, whatever `Reply-To` says.
    ///
    /// The way to answer one person on a list without answering the list.
    Sender,
}

impl ReplyMode {
    /// How this reply is announced when the compose window opens.
    ///
    /// Said out loud because the three keys differ only by a modifier, and the
    /// cost of hitting the wrong one is a private message sent to a list.
    pub fn description(&self) -> &'static str {
        match self {
            ReplyMode::Default => "Reply",
            ReplyMode::All => "Reply to all",
            ReplyMode::Sender => "Reply to sender only",
        }
    }
}

/// Who a reply is addressed to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplyRecipients {
    pub to: String,
    pub cc: String,
}

impl ReplyRecipients {
    /// How many people this reply reaches.
    ///
    /// Announced with the mode, because "Reply to all, 34 recipients" is the
    /// one piece of information that stops the wrong key being an accident
    /// somebody only learns about afterwards.
    pub fn count(&self) -> usize {
        split_addresses(&self.to).len() + split_addresses(&self.cc).len()
    }
}

/// What a reply is about to do, in one sentence, before the window opens.
///
/// Said out loud because the three keys differ only by a modifier and the cost
/// of the wrong one is a private answer in front of a mailing list. One
/// recipient is named, because "Reply to sender only, Ada Lovelace" is the
/// difference from a reply to all and is otherwise invisible until the message
/// has gone. More than one is counted rather than listed: reading out
/// thirty-four addresses is a wall somebody has to sit through.
pub fn announcement(mode: ReplyMode, recipients: &ReplyRecipients) -> String {
    let everyone: Vec<String> = split_addresses(&recipients.to)
        .into_iter()
        .chain(split_addresses(&recipients.cc))
        .collect();

    match everyone.as_slice() {
        [] => format!("{}, nobody to send to", mode.description()),
        [only] => format!("{}, {}", mode.description(), name_or_address(only)),
        many => format!("{}, {} recipients", mode.description(), many.len()),
    }
}

/// How to refer to one recipient out loud.
///
/// The display name where the address carries one, because that is what the
/// person is called; the address itself where it does not, rather than a guess
/// made from the local part, which would announce a name nobody chose.
fn name_or_address(address: &str) -> String {
    let trimmed = address.trim();
    let Some(open) = trimmed.find('<') else {
        return trimmed.to_string();
    };
    let name = trimmed[..open].trim().trim_matches('"').trim();
    if name.is_empty() {
        return key_of(trimmed);
    }
    name.to_string()
}

/// What one message looks like for the purpose of replying.
#[derive(Debug, Clone, Default)]
pub struct RepliedTo<'a> {
    pub from: &'a str,
    pub reply_to: &'a str,
    pub to: &'a str,
    pub cc: &'a str,
}

/// Work out the recipients of a reply.
///
/// `own_addresses` are the addresses of the accounts set up here, which are
/// dropped from a reply-all: nobody wants their own copy, and on a thread they
/// have answered twice their address would otherwise appear twice.
pub fn reply_recipients(
    message: &RepliedTo<'_>,
    own_addresses: &[String],
    mode: ReplyMode,
) -> ReplyRecipients {
    let answer_to = match mode {
        // Ignoring Reply-To is the whole point of this one.
        ReplyMode::Sender => message.from.trim(),
        _ if message.reply_to.trim().is_empty() => message.from.trim(),
        _ => message.reply_to.trim(),
    };

    let to = split_addresses(answer_to);
    if mode != ReplyMode::All {
        return ReplyRecipients {
            to: join(&to),
            cc: String::new(),
        };
    }

    // What is already spoken for: the addresses this reply is going to, and
    // our own.
    let mut seen: Vec<String> = to.iter().map(|a| key_of(a)).collect();
    seen.extend(own_addresses.iter().map(|a| key_of(a)));

    // The person who wrote it comes first, because a reply to all should reach
    // them. When they set a `Reply-To`, which is the mailing list case, the
    // reply is addressed to the list and they are not otherwise anywhere in
    // it, so without this the one person the answer is for is the one person
    // who does not get it. Decided rather than discovered: some lists treat a
    // personal copy as rude, and reaching the author was judged the lesser
    // cost of the two.
    //
    // Skipped when they are already the addressee, which is every message
    // without a `Reply-To`, and skipped when they are you.
    let mut cc = Vec::new();
    for address in split_addresses(message.from)
        .into_iter()
        .chain(split_addresses(message.to))
        .chain(split_addresses(message.cc))
    {
        let key = key_of(&address);
        if key.is_empty() || seen.contains(&key) {
            continue;
        }
        seen.push(key);
        cc.push(address);
    }

    ReplyRecipients {
        to: join(&to),
        cc: join(&cc),
    }
}

/// Split a header value into its addresses.
///
/// Commas inside a quoted display name do not separate addresses, and neither
/// do commas inside angle brackets. `"Smith, John" <j@example.com>` is one
/// person, and splitting it in two produces two recipients that do not exist
/// and a message that bounces.
fn split_addresses(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_angles = false;

    for ch in value.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '<' if !in_quotes => {
                in_angles = true;
                current.push(ch);
            }
            '>' if !in_quotes => {
                in_angles = false;
                current.push(ch);
            }
            ',' | ';' if !in_quotes && !in_angles => {
                push_trimmed(&mut out, &current);
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    push_trimmed(&mut out, &current);
    out
}

fn push_trimmed(out: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
}

/// The address itself, lowercased, for comparing two spellings of one person.
///
/// "Ada Lovelace <ada@example.com>" and "ADA@EXAMPLE.COM" are the same
/// recipient, and a reply-all that lists both sends two copies.
fn key_of(address: &str) -> String {
    let trimmed = address.trim();
    let inner = match (trimmed.find('<'), trimmed.rfind('>')) {
        (Some(open), Some(close)) if close > open => &trimmed[open + 1..close],
        _ => trimmed,
    };
    inner.trim().to_lowercase()
}

fn join(addresses: &[String]) -> String {
    addresses.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list_message() -> RepliedTo<'static> {
        RepliedTo {
            from: "Ada Lovelace <ada@example.com>",
            reply_to: "rust-users@lists.example.org",
            to: "rust-users@lists.example.org",
            cc: "Charles Babbage <charles@example.com>",
        }
    }

    fn plain_message() -> RepliedTo<'static> {
        RepliedTo {
            from: "Ada Lovelace <ada@example.com>",
            reply_to: "",
            to: "me@example.com, Grace Hopper <grace@example.com>",
            cc: "Charles Babbage <charles@example.com>",
        }
    }

    fn own_message() -> RepliedTo<'static> {
        RepliedTo {
            from: "me@example.com",
            reply_to: "rust-users@lists.example.org",
            to: "rust-users@lists.example.org",
            cc: "Charles Babbage <charles@example.com>",
        }
    }

    fn mine() -> Vec<String> {
        vec!["me@example.com".to_string()]
    }

    #[test]
    fn test_the_ordinary_reply_to_a_list_reaches_the_list() {
        // What the sender asked for and what everyone in the thread expects.
        let reply = reply_recipients(&list_message(), &mine(), ReplyMode::Default);
        assert_eq!(reply.to, "rust-users@lists.example.org");
        assert!(reply.cc.is_empty());
    }

    #[test]
    fn test_replying_to_the_sender_only_never_reaches_the_list() {
        // The mistake this mode exists to prevent is the other one: a private
        // answer arriving in front of everybody subscribed.
        let reply = reply_recipients(&list_message(), &mine(), ReplyMode::Sender);
        assert_eq!(reply.to, "Ada Lovelace <ada@example.com>");
        assert!(
            !reply.to.contains("lists.example.org"),
            "the list crept back in"
        );
        assert!(reply.cc.is_empty());
    }

    #[test]
    fn test_reply_to_all_on_a_list_also_reaches_the_person_who_wrote_it() {
        // This was a characterisation test recording that the author was left
        // out. That is now a decision rather than an accident: a reply to all
        // reaches everyone the message reached and the person who wrote it,
        // even when they asked for replies to go to a list.
        let reply = reply_recipients(&list_message(), &mine(), ReplyMode::All);

        assert_eq!(reply.to, "rust-users@lists.example.org");
        assert!(reply.cc.contains("charles@example.com"), "{}", reply.cc);
        assert!(
            reply.cc.to_lowercase().contains("ada@example.com"),
            "the author is left out of a reply to all: {}",
            reply.cc
        );
    }

    #[test]
    fn test_reply_to_all_does_not_copy_the_author_who_is_already_the_addressee() {
        // With no Reply-To the author is already in To, and copying them there
        // as well sends one person two copies of the same reply.
        let reply = reply_recipients(&plain_message(), &mine(), ReplyMode::All);
        assert_eq!(reply.to, "Ada Lovelace <ada@example.com>");
        assert_eq!(
            reply.cc.to_lowercase().matches("ada@example.com").count(),
            0,
            "the author was copied as well as addressed: {}",
            reply.cc
        );
    }

    #[test]
    fn test_reply_to_all_on_your_own_message_does_not_copy_you() {
        // Answering your own message to a list is common, and your own address
        // is the one address a reply to all has always left out.
        let reply = reply_recipients(&own_message(), &mine(), ReplyMode::All);
        assert!(
            !reply.cc.to_lowercase().contains("me@example.com"),
            "sent to self: {}",
            reply.cc
        );
        assert!(reply.cc.contains("charles@example.com"), "{}", reply.cc);
    }

    #[test]
    fn test_an_ordinary_reply_goes_to_whoever_wrote_it() {
        let reply = reply_recipients(&plain_message(), &mine(), ReplyMode::Default);
        assert_eq!(reply.to, "Ada Lovelace <ada@example.com>");
    }

    #[test]
    fn test_reply_to_all_keeps_the_other_recipients() {
        // The bug this replaces: reply-all sent an empty Cc, so it did exactly
        // what plain reply did and quietly dropped everybody else.
        let reply = reply_recipients(&plain_message(), &mine(), ReplyMode::All);
        assert_eq!(reply.to, "Ada Lovelace <ada@example.com>");
        assert!(reply.cc.contains("grace@example.com"));
        assert!(reply.cc.contains("charles@example.com"));
    }

    #[test]
    fn test_reply_to_all_leaves_you_out() {
        // Your own copy is noise, and on a thread you have answered before your
        // address would appear more than once.
        let reply = reply_recipients(&plain_message(), &mine(), ReplyMode::All);
        assert!(
            !reply.cc.to_lowercase().contains("me@example.com"),
            "sent to self: {}",
            reply.cc
        );
    }

    #[test]
    fn test_reply_to_all_does_not_send_two_copies_to_one_person() {
        // The same person spelled two ways is still one person.
        let message = RepliedTo {
            from: "Ada Lovelace <ada@example.com>",
            reply_to: "",
            to: "ADA@EXAMPLE.COM, Grace <grace@example.com>",
            cc: "ada@example.com",
        };
        let reply = reply_recipients(&message, &mine(), ReplyMode::All);
        assert_eq!(
            reply.cc.to_lowercase().matches("ada@example.com").count(),
            0,
            "the sender was copied as well as addressed: {}",
            reply.cc
        );
        assert!(reply.cc.contains("grace@example.com"));
    }

    #[test]
    fn test_a_display_name_with_a_comma_is_one_person() {
        // Splitting it in two produces two recipients who do not exist and a
        // message that bounces.
        let message = RepliedTo {
            from: "\"Smith, John\" <john@example.com>",
            reply_to: "",
            to: "\"Doe, Jane\" <jane@example.com>, bob@example.com",
            cc: "",
        };
        let reply = reply_recipients(&message, &[], ReplyMode::All);
        assert_eq!(reply.to, "\"Smith, John\" <john@example.com>");
        assert!(reply.cc.contains("\"Doe, Jane\" <jane@example.com>"));
        assert!(reply.cc.contains("bob@example.com"));
        assert_eq!(reply.count(), 3);
    }

    #[test]
    fn test_a_less_than_sign_inside_a_quoted_name_does_not_start_an_address() {
        // Inside quotes it is just a character somebody typed. Treated as the
        // start of an address it would swallow every comma after it and turn a
        // whole header into one recipient that does not exist.
        assert_eq!(
            split_addresses("\"a<b\", c@example.com"),
            vec!["\"a<b\"".to_string(), "c@example.com".to_string()]
        );
    }

    #[test]
    fn test_a_greater_than_sign_inside_a_quoted_name_does_not_close_an_address() {
        // The mirror of the rule on the opening bracket. Closing early would
        // let the comma after it split one address into two.
        assert_eq!(
            split_addresses("<\"a>b\",c@example.com>"),
            vec!["<\"a>b\",c@example.com>".to_string()]
        );
    }

    #[test]
    fn test_a_comma_inside_angle_brackets_does_not_separate_addresses() {
        // The half of the splitting rule nothing watched. Splitting here makes
        // two recipients out of one and both of them bounce.
        assert_eq!(
            split_addresses("<a,b@example.com>, c@example.com"),
            vec!["<a,b@example.com>".to_string(), "c@example.com".to_string()]
        );
    }

    #[test]
    fn test_an_address_with_the_brackets_the_wrong_way_round_is_left_alone() {
        // Slicing from the opening bracket to a closing one that came first is
        // a panic, on a header a stranger wrote.
        assert_eq!(key_of("a>b<c"), "a>b<c");
    }

    #[test]
    fn test_semicolons_separate_recipients_too() {
        // Some clients and some people write them.
        let message = RepliedTo {
            from: "ada@example.com",
            reply_to: "",
            to: "a@example.com; b@example.com",
            cc: "",
        };
        let reply = reply_recipients(&message, &[], ReplyMode::All);
        assert!(reply.cc.contains("a@example.com"));
        assert!(reply.cc.contains("b@example.com"));
    }

    #[test]
    fn test_a_message_with_nobody_to_reply_to_produces_nothing() {
        // Better than a compose window addressed to an empty string, which
        // fails at the server with an error nobody can act on.
        let empty = RepliedTo::default();
        for mode in [ReplyMode::Default, ReplyMode::All, ReplyMode::Sender] {
            let reply = reply_recipients(&empty, &mine(), mode);
            assert!(reply.to.is_empty(), "{mode:?} invented a recipient");
            assert_eq!(reply.count(), 0);
        }
    }

    #[test]
    fn test_a_blank_reply_to_header_is_not_an_address() {
        // Present but empty is not somewhere to send mail.
        let message = RepliedTo {
            from: "ada@example.com",
            reply_to: "   ",
            ..Default::default()
        };
        let reply = reply_recipients(&message, &[], ReplyMode::Default);
        assert_eq!(reply.to, "ada@example.com");
    }

    #[test]
    fn test_the_reach_of_a_reply_is_countable() {
        // Announced with the mode, because the three keys differ by a modifier
        // and "34 recipients" is what stops the wrong one being a surprise.
        let reply = reply_recipients(&plain_message(), &mine(), ReplyMode::All);
        assert_eq!(reply.count(), 3);
        let one = reply_recipients(&plain_message(), &mine(), ReplyMode::Sender);
        assert_eq!(one.count(), 1);
    }

    #[test]
    fn test_a_reply_to_one_person_says_who_it_is_going_to() {
        // The difference between answering one person and answering a list is
        // the whole point of the sender-only key, and it is otherwise
        // invisible until the message has gone.
        let one = reply_recipients(&list_message(), &mine(), ReplyMode::Sender);
        assert_eq!(
            announcement(ReplyMode::Sender, &one),
            "Reply to sender only, Ada Lovelace"
        );
    }

    #[test]
    fn test_a_reply_to_one_person_with_no_name_says_their_address() {
        let one = reply_recipients(
            &RepliedTo {
                from: "charles@example.com",
                ..Default::default()
            },
            &mine(),
            ReplyMode::Default,
        );
        assert_eq!(
            announcement(ReplyMode::Default, &one),
            "Reply, charles@example.com"
        );
    }

    #[test]
    fn test_a_reply_to_many_says_how_many_rather_than_listing_them() {
        // Reading out thirty-four addresses before the window opens is not
        // information, it is a wall somebody has to sit through.
        let many = reply_recipients(&plain_message(), &mine(), ReplyMode::All);
        assert_eq!(
            announcement(ReplyMode::All, &many),
            "Reply to all, 3 recipients"
        );
    }

    #[test]
    fn test_a_reply_to_nobody_says_so_rather_than_counting_to_zero() {
        let none = reply_recipients(&RepliedTo::default(), &mine(), ReplyMode::All);
        assert_eq!(
            announcement(ReplyMode::All, &none),
            "Reply to all, nobody to send to"
        );
    }

    #[test]
    fn test_each_mode_says_what_it_is() {
        assert_eq!(ReplyMode::Default.description(), "Reply");
        assert_eq!(ReplyMode::All.description(), "Reply to all");
        assert_eq!(ReplyMode::Sender.description(), "Reply to sender only");
    }

    #[test]
    fn test_splitting_never_panics_on_a_malformed_header() {
        // Headers come from strangers.
        for value in [
            "",
            ",",
            ";;;",
            "<",
            ">",
            "<<>>",
            "a>b<c",
            ">x<",
            "\"unclosed <a@example.com>",
            "a@example.com,",
            ",a@example.com",
            "\u{4f60}\u{597d} <ni@example.com>",
        ] {
            let message = RepliedTo {
                from: value,
                reply_to: value,
                to: value,
                cc: value,
            };
            for mode in [ReplyMode::Default, ReplyMode::All, ReplyMode::Sender] {
                let _ = reply_recipients(&message, &mine(), mode);
            }
        }
    }
}

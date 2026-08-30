//! Where each account sits in the list, and what moving one says.
//!
//! D-14. Accounts appear in the order they were added and can be moved with
//! Alt+Shift+Up and Alt+Shift+Down, with the new position announced as it
//! moves.
//!
//! # Nothing here reaches a server
//!
//! The order of somebody's accounts is a fact about their own list. No mail
//! protocol has a notion of it, so there is nothing to send and nowhere to send
//! it, and a command that quietly opened a connection to record a preference
//! would be doing something nobody asked for on a metered or watched network.
//! FOLDER-03 says the same of pinning a folder, and the two are one shape.
//!
//! # Why it says something when it does nothing
//!
//! Moving the first account up cannot move anything. For somebody working by
//! ear, a keystroke that does nothing and says nothing is indistinguishable
//! from a keystroke that was not received, so they press it again. It answers
//! instead, and the answer says where the account already is.

/// Which way an account is being moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
}

/// What moving an account came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    /// Every account's id, in the order they should now be stored in.
    ///
    /// The whole list rather than the two that swapped. Writing only the pair
    /// would leave the rest with no stored position, and a list half ordered by
    /// choice and half by arrival reorders itself the next time an account is
    /// added.
    pub order: Vec<String>,
    /// What to say, whether or not anything moved.
    pub say: String,
    /// Whether anything actually changed, so a caller can skip the writing.
    pub moved: bool,
}

/// Move one account up or down the list.
///
/// `accounts` is every account as `(id, name)` in the order they sit in now.
pub fn moved(accounts: &[(String, String)], which: &str, direction: Move) -> Moved {
    let order: Vec<String> = accounts.iter().map(|(id, _)| id.clone()).collect();
    let howmany = accounts.len();

    let Some(at) = accounts.iter().position(|(id, _)| id == which) else {
        // Nothing is chosen, or the chosen row is not an account. Said rather
        // than ignored, because the alternative is a key that sometimes works
        // and never explains itself.
        return Moved {
            order,
            say: "Choose an account first. Move Account Up and Move Account Down \
                  act on the account branch the cursor is on."
                .to_string(),
            moved: false,
        };
    };
    let name = accounts[at].1.clone();

    let swap_with = match direction {
        Move::Up if at == 0 => {
            return Moved {
                order,
                say: format!("{name} is already first of {howmany}."),
                moved: false,
            };
        }
        Move::Down if at + 1 == howmany => {
            return Moved {
                order,
                say: format!("{name} is already last of {howmany}."),
                moved: false,
            };
        }
        Move::Up => at - 1,
        Move::Down => at + 1,
    };

    let mut order = order;
    order.swap(at, swap_with);
    Moved {
        // One past the index, because somebody counting a list of three says
        // first, second, third rather than nought, one, two.
        say: format!("{name}, {} of {howmany}.", swap_with + 1),
        order,
        moved: true,
    }
}

/// Where the command that moves an account is written, so the check below can
/// read it.
///
/// A path rather than a module, because what has to be read is the shipping
/// half of a file and `what_ships` is the one answer to which half that is.
#[cfg(test)]
const WHERE_THE_COMMAND_LIVES: &str = "src/presentation/wx_app.rs";

/// The name of that command, which is what marks off the part to read.
#[cfg(test)]
const THE_COMMAND: &str = "fn move_the_chosen_account";

/// Reordering accounts never reaches a server.
///
/// D-14 says so and FOLDER-03 says the same of pinning a folder. A comment
/// saying it is not a check: the whole point is that nothing here opens a
/// connection, and a connection nobody opened leaves no trace for a behaviour
/// test to find. So this reads the source, and the companion test below proves
/// the reading can see such a call when there is one.
///
/// Call syntax rather than bare words, because the sentences explaining this
/// rule name the very things the rule forbids. A check that fires on the
/// paragraph explaining it is a check somebody switches off.
#[cfg(test)]
mod nothing_here_reaches_a_server {
    use super::{THE_COMMAND, WHERE_THE_COMMAND_LIVES};

    /// How a call out of this machine is spelled, in call syntax.
    const A_CALL_THAT_LEAVES_THIS_MACHINE: [&str; 5] = [
        "crate::service::",
        "reqwest::",
        "spawn_mail_sync(",
        "a_session_at(",
        "super::service::",
    ];

    /// The body of one function, from its signature to the margin brace that
    /// ends it.
    fn the_body_of(source: &str, signature: &str) -> String {
        let from = source
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} to be in the file"));
        let rest = &source[from..];
        let ends = rest.find("\n}").map(|at| at + 2).unwrap_or(rest.len());
        rest[..ends].to_string()
    }

    fn calls_out_of_this_machine(text: &str) -> Vec<String> {
        text.lines()
            .enumerate()
            .filter(|(_, line)| {
                A_CALL_THAT_LEAVES_THIS_MACHINE
                    .iter()
                    .any(|call| line.contains(call))
            })
            .map(|(number, line)| format!("{}: {}", number + 1, line.trim()))
            .collect()
    }

    #[test]
    fn test_moving_an_account_makes_no_call_that_leaves_this_machine() {
        let deciding = crate::common::what_ships::what_ships(
            &std::fs::read_to_string("src/application/account_order.rs")
                .expect("the module that decides a move"),
        );
        let doing = the_body_of(
            &crate::common::what_ships::what_ships(
                &std::fs::read_to_string(WHERE_THE_COMMAND_LIVES).expect("the command"),
            ),
            THE_COMMAND,
        );

        let found: Vec<String> = calls_out_of_this_machine(&deciding)
            .into_iter()
            .chain(calls_out_of_this_machine(&doing))
            .collect();

        assert!(
            found.is_empty(),
            "{} call(s) that leave this machine sit in the path that moves an \
             account. Where somebody's accounts sit is a fact about their own \
             list, no mail protocol has a notion of it, and a command that \
             opened a connection to record a preference would be doing \
             something nobody asked for:\n  {}",
            found.len(),
            found.join("\n  ")
        );
    }

    #[test]
    fn test_the_reading_can_see_such_a_call_when_there_is_one() {
        // Without this the test above passes just as well when the reading has
        // stopped reading anything, and a check that fails two ways without
        // saying which is worse than no check.
        let pretend = "fn move_the_chosen_account(app: AppHandles) {\n    \
                       crate::service::protocols::imap::a_session_at(host);\n}";
        assert_eq!(
            calls_out_of_this_machine(&the_body_of(pretend, THE_COMMAND)).len(),
            1
        );
        // And it does not fire on the paragraph that explains the rule, which
        // names every one of these things in prose.
        assert!(
            calls_out_of_this_machine(
                "/// This never calls the service layer, reqwest, or any mail \
                 protocol: no imap, no smtp, no pop3, no caldav."
            )
            .is_empty()
        );
    }

    #[test]
    fn test_the_command_this_reads_is_still_there_under_that_name() {
        // A renamed function would make the reading above find nothing and
        // report a clean result over a file it never looked into.
        let source = std::fs::read_to_string(WHERE_THE_COMMAND_LIVES).expect("the command");
        assert!(
            source.contains(THE_COMMAND),
            "{THE_COMMAND} has been renamed, so the check that reads it is \
             reading nothing"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three() -> Vec<(String, String)> {
        [("a", "Work"), ("b", "Home"), ("c", "Old")]
            .iter()
            .map(|(id, name)| (id.to_string(), name.to_string()))
            .collect()
    }

    fn ids(moved: &Moved) -> Vec<&str> {
        moved.order.iter().map(String::as_str).collect()
    }

    #[test]
    fn test_moving_an_account_down_swaps_it_with_the_one_after_it() {
        let after = moved(&three(), "a", Move::Down);
        assert_eq!(ids(&after), vec!["b", "a", "c"]);
        assert!(after.moved);
    }

    #[test]
    fn test_moving_an_account_up_swaps_it_with_the_one_before_it() {
        let after = moved(&three(), "c", Move::Up);
        assert_eq!(ids(&after), vec!["a", "c", "b"]);
        assert!(after.moved);
    }

    #[test]
    fn test_a_move_says_the_name_and_where_it_now_sits_among_how_many() {
        assert_eq!(moved(&three(), "a", Move::Down).say, "Work, 2 of 3.");
        assert_eq!(moved(&three(), "c", Move::Up).say, "Old, 2 of 3.");
        // The far end, so the count and the position are not the same number
        // by accident in every case this test looks at.
        assert_eq!(moved(&three(), "b", Move::Down).say, "Home, 3 of 3.");
    }

    #[test]
    fn test_moving_the_first_account_up_changes_nothing_and_says_so() {
        let after = moved(&three(), "a", Move::Up);
        assert_eq!(ids(&after), vec!["a", "b", "c"], "nothing moved");
        assert!(!after.moved);
        assert_eq!(after.say, "Work is already first of 3.");
    }

    #[test]
    fn test_moving_the_last_account_down_changes_nothing_and_says_so() {
        let after = moved(&three(), "c", Move::Down);
        assert_eq!(ids(&after), vec!["a", "b", "c"], "nothing moved");
        assert!(!after.moved);
        assert_eq!(after.say, "Old is already last of 3.");
    }

    #[test]
    fn test_an_account_that_is_not_in_the_list_is_answered_rather_than_ignored() {
        let after = moved(&three(), "nobody", Move::Up);
        assert_eq!(ids(&after), vec!["a", "b", "c"]);
        assert!(!after.moved);
        assert!(
            after.say.contains("Choose an account first"),
            "{}",
            after.say
        );
    }

    #[test]
    fn test_one_account_is_both_first_and_last_and_neither_key_pretends_otherwise() {
        let only = vec![("a".to_string(), "Work".to_string())];
        assert_eq!(
            moved(&only, "a", Move::Up).say,
            "Work is already first of 1."
        );
        assert_eq!(
            moved(&only, "a", Move::Down).say,
            "Work is already last of 1."
        );
    }

    #[test]
    fn test_no_account_is_lost_or_repeated_by_a_move() {
        // The property worth having whatever the wording is. A swap that wrote
        // the same id into both places would pass every count above.
        for (which, direction) in [
            ("a", Move::Down),
            ("b", Move::Up),
            ("b", Move::Down),
            ("c", Move::Up),
        ] {
            let after = moved(&three(), which, direction);
            let mut sorted = after.order.clone();
            sorted.sort();
            assert_eq!(sorted, vec!["a", "b", "c"], "{which} {direction:?}");
        }
    }
}

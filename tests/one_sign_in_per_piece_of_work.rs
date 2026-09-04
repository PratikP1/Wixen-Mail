//! How many places sign in to a mail server for themselves.
//!
//! `application::mail_session::a_session_at` exists so that signing in to an
//! account's own mail server is one piece of code rather than several. The main
//! window mostly does not use it. It builds a `MailController`, connects, issues
//! one command and disconnects again, once per command, so marking a single
//! message read is a TLS handshake, a CAPABILITY, a LOGIN and a SELECT.
//!
//! `SCALE-02` carried that number as a list of line numbers, and it went stale
//! twice without anything noticing. It said eight, at eight lines. By 2026-09-03
//! every one of those eight lines had moved and none of them was a sign-in any
//! more, and the truth was twelve. The list was measured again that day and
//! written into the phase research, and it was stale again by 2026-09-04: the
//! file had grown by twenty-one lines and seven of the twelve had moved with it.
//!
//! So the number lives here, where changing it fails a test. What is written
//! down is the total and the day it was counted. Where the sign-ins are is found
//! by reading the tree, because a list of line numbers is the artefact whose
//! staleness is the reason for this file.
//!
//! What this cannot see. It reads source, so it says where a sign-in is written
//! and not whether that code is ever reached. It says nothing about how long a
//! session is held open, whether one is reused, or what a failed connection
//! does. Those are plan 03-06's, and they are why this number is expected to
//! fall.

use std::cmp::Ordering;
use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The file that signs in for itself. Every site counted here is in it.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// How many places build their own connection instead of asking the helper.
///
/// Counted on 2026-09-04, twelve, and eleven later the same day. Neither is a
/// target, both are measurements: the requirement said eight and was wrong by
/// half, twice, which is why the number is written here beside the code that
/// re-derives it rather than in a document.
///
/// It is expected to fall the rest of the way, and plan 03-06 is what makes it
/// fall. That plan holds one session open for an account rather than for one
/// piece of work, and signs in again when the server drops it, so the sites
/// below stop dialling for themselves. The first to go was the one the
/// requirement names, marking a message read. When the last goes this is nought
/// and the comment above it says so.
const SIGN_INS_THAT_GO_ROUND_THE_HELPER: usize = 11;

/// The line `what_ships` looks at by its exact text, which is why it is the one
/// line left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// A place that builds a mail controller and signs in with it.
#[derive(Debug, PartialEq, Eq)]
struct SignsInForItself {
    /// The line the controller is built on, in the file as it stands.
    built_at: usize,
    /// The line the sign-in is asked for on.
    connected_at: usize,
}

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// `what_ships` deletes the lines a release build does not compile, so the text
/// it hands back is numbered differently from the file it read: a site at line
/// 40 of it can sit at line 200 of the file, and a failure message saying 40
/// sends somebody to the wrong place. The numbers are carried through the cut by
/// writing each line's own number into a trailing comment before the cut and
/// reading it back after.
///
/// A comment is invisible to everything `what_ships` decides. It takes comments
/// and quoted text out before counting the brackets that end an item, and the
/// one line whose exact text it looks at is `#[cfg(test)]`, which is left
/// unmarked here for that reason. A line left unmarked is a line it never hands
/// back, so nothing arrives below without a number, and the panic says so rather
/// than dropping it.
fn the_shipping_lines_of(source: &str) -> Vec<(usize, String)> {
    let numbered: Vec<String> = source
        .lines()
        .enumerate()
        .map(|(at, line)| match line.trim() == THE_TEST_ATTRIBUTE {
            true => line.to_string(),
            false => format!("{line}{THE_LINE_NUMBER}{}", at + 1),
        })
        .collect();

    what_ships(&numbered.join("\n"))
        .lines()
        .map(|line| {
            let (text, at) = line
                .rsplit_once(THE_LINE_NUMBER)
                .unwrap_or_else(|| panic!("a line came back from the cut unnumbered: {line}"));
            let at: usize = at
                .parse()
                .unwrap_or_else(|e| panic!("a line came back carrying '{at}' as its number: {e}"));
            (at, text.to_string())
        })
        .collect()
}

/// The part of a line that a comment cannot reach.
///
/// A census that reads whole lines is answered by the prose explaining the code
/// rather than by the code. One written in this project on 2026-09-04 read a
/// whole match arm for a string and so read the comment saying why that string
/// was deliberately not used, and failed correct code with a message stating the
/// opposite of the truth. So the comment comes off before anything is looked
/// for.
///
/// What this cannot see: a `/* */` comment, and a `//` inside a string literal,
/// which truncates the line early. Neither can invent a sign-in, because a
/// sign-in needs a construction and a connect on two different lines of real
/// code. Truncation can only hide one, and no line in the file this reads puts a
/// slashed string before either half.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// Every place in `source` that builds a mail controller and signs in with it.
///
/// Two facts together, because either on its own is answered wrongly. A
/// `MailController::new()` is not a sign-in: the send loop builds one for SMTP
/// and the POP3 reader builds one it never points at IMAP, and both would be
/// counted by a census that looked only for the construction. A `connect_imap`
/// on its own would count the helper's, which is the one sign-in that is
/// supposed to be there.
///
/// A controller's window runs from where it is built to where the next one is
/// built. That is what stops a controller that is never connected from claiming
/// the sign-in belonging to the one after it, and what stops one sign-in being
/// counted for two controllers.
///
/// A site that goes through the helper has neither half, so it is not counted
/// and nothing here has to name the helper to leave it out.
fn the_sign_ins_that_go_round_the_helper(source: &str) -> Vec<SignsInForItself> {
    let shipping = the_shipping_lines_of(source);
    let code: Vec<(usize, &str)> = shipping
        .iter()
        .map(|(at, line)| (*at, code_of(line)))
        .collect();

    let built: Vec<usize> = code
        .iter()
        .enumerate()
        .filter(|(_, (_, line))| line.contains("MailController::new()"))
        .map(|(where_in_the_cut, _)| where_in_the_cut)
        .collect();

    let mut found = Vec::new();
    for (which, from) in built.iter().enumerate() {
        let until = built.get(which + 1).copied().unwrap_or(code.len());
        let signed_in = code[*from..until]
            .iter()
            .find(|(_, line)| line.contains("connect_imap") && !line.contains("disconnect_imap"));
        if let Some((connected_at, _)) = signed_in {
            found.push(SignsInForItself {
                built_at: code[*from].0,
                connected_at: *connected_at,
            });
        }
    }
    found
}

/// What to say when the count has moved, naming the sites rather than only the
/// number.
///
/// A census that says "expected 12, found 13" sends the next person back to the
/// file to work out which one is new, which is the work the census was supposed
/// to have done for them.
fn what_the_census_found(found: &[SignsInForItself]) -> String {
    let which_way = match found.len().cmp(&SIGN_INS_THAT_GO_ROUND_THE_HELPER) {
        Ordering::Greater => {
            "That is more than were counted, so somewhere new signs in for \
             itself. Take it through application::mail_session::a_session_at, \
             or say here why it cannot."
        }
        Ordering::Less => {
            "That is fewer than were counted. If plan 03-06 took one out, lower \
             the number here and change the day beside it. If nothing was \
             taken out, this reading has stopped working."
        }
        Ordering::Equal => "",
    };
    let sites: Vec<String> = found
        .iter()
        .map(|site| {
            format!(
                "{THE_MAIN_WINDOW}:{} builds it, :{} signs in",
                site.built_at, site.connected_at
            )
        })
        .collect();
    format!(
        "{} places build a mail controller and sign in with it, and \
         {SIGN_INS_THAT_GO_ROUND_THE_HELPER} were counted on 2026-09-04. \
         {which_way}\n  {}",
        found.len(),
        sites.join("\n  ")
    )
}

/// The main window signs in for itself in as many places as were counted.
///
/// The one assertion this file exists for. Everything below it is about whether
/// this one is reading anything.
#[test]
fn test_the_places_that_sign_in_for_themselves_are_counted_and_named() {
    let source = fs::read_to_string(THE_MAIN_WINDOW).expect("the main window");
    let found = the_sign_ins_that_go_round_the_helper(&source);

    assert_eq!(
        found.len(),
        SIGN_INS_THAT_GO_ROUND_THE_HELPER,
        "{}",
        what_the_census_found(&found)
    );
}

/// Counting is a reading, and it is measured on source this file made up.
///
/// Without this the census could be a constant that happens to agree with a
/// number written above it, and every run would say so.
#[test]
fn test_two_sign_ins_in_invented_source_are_counted_as_two() {
    let invented = "\
fn first() {
    let controller = MailController::new();
    controller.connect_imap(server, port).await?;
}

fn second() {
    let controller = MailController::new();
    controller.connect_imap(server, port).await?;
}
";

    assert_eq!(
        the_sign_ins_that_go_round_the_helper(invented),
        vec![
            SignsInForItself {
                built_at: 2,
                connected_at: 3
            },
            SignsInForItself {
                built_at: 7,
                connected_at: 8
            },
        ],
        "the counter did not find the two sign-ins written into this test"
    );
}

/// Source that builds a controller and never signs in with it counts none.
///
/// The main window really does this twice, once for the send loop and once for
/// the POP3 reader, so a census that counted constructions would answer fourteen
/// where the truth is twelve.
#[test]
fn test_a_controller_that_never_signs_in_is_not_counted() {
    let invented = "\
fn sends() {
    let controller = MailController::new();
    controller.connect_smtp(server, port).await?;
    controller.disconnect_imap().await;
}
";

    assert!(
        the_sign_ins_that_go_round_the_helper(invented).is_empty(),
        "a controller that never signed in to IMAP was counted as one that did"
    );
}

/// A sign-in through the helper is not a sign-in that goes round it.
///
/// The main window has one of these, and the whole point of the census is that
/// it counts the other twelve and not this one.
#[test]
fn test_a_sign_in_through_the_helper_is_not_counted() {
    let invented = "\
fn files_a_draft() {
    let session = crate::application::mail_session::a_session_at(&account).await?;
    session.append(&folder, &raw).await?;
    let _ = session.disconnect_imap().await;
}
";

    assert!(
        the_sign_ins_that_go_round_the_helper(invented).is_empty(),
        "a sign-in that went through the helper was counted as one that did not"
    );
}

/// Nothing a release build does not compile is counted.
///
/// A test fixture that stands up its own controller is not the program signing
/// in for itself, and counting one would make the census answer a number that
/// moves when somebody writes a test.
///
/// The invented module below carries no test attribute of its own, and that is
/// deliberate. `how_many_tests_are_in` in `tests/house_style.rs` counts a line
/// whose whole text is that attribute wherever it appears, so one written inside
/// a string here would be counted as a test of this file and the guard record's
/// fingerprint would claim nine where there are seven. What the cut turns on is
/// the `#[cfg(test)]` line, so leaving the inner attribute out costs the fixture
/// nothing.
#[test]
fn test_a_sign_in_only_a_test_build_compiles_is_not_counted() {
    let invented = "\
fn ships() {
    let cache = MessageCache::open(&path)?;
}

#[cfg(test)]
mod tests {
    async fn against_a_server() {
        let controller = MailController::new();
        controller.connect_imap(server, port).await?;
    }
}
";

    assert!(
        the_sign_ins_that_go_round_the_helper(invented).is_empty(),
        "a sign-in only a test build compiles was counted as one the program does"
    );
}

/// The reading can tell a comment about a sign-in from a sign-in.
///
/// The failure this is here for happened in this project on 2026-09-04: a check
/// read a whole block for a string, so it read the comment explaining why that
/// string was not used, and reported correct code as broken. A census over a
/// file like `wx_app.rs`, which explains itself at length, would be answered by
/// its own prose within a week.
#[test]
fn test_a_comment_naming_both_halves_is_not_a_sign_in() {
    let invented = "\
fn files_a_draft() {
    // This used to be MailController::new() followed by a
    // controller.connect_imap(server, port), once per message, which is what
    // the helper below replaced.
    let session = crate::application::mail_session::a_session_at(&account).await?;
}
";

    assert!(
        the_sign_ins_that_go_round_the_helper(invented).is_empty(),
        "the comment describing a sign-in was counted as a sign-in"
    );
}

/// A site is named by the line it is on in the file, not by where it lands after
/// the test half is cut out.
///
/// `what_ships` deletes lines, so the two numbers differ by however much sits
/// above a site in test-only code. Nothing in the main window is above the
/// twelve today, which is exactly why this needs saying: the two numbers agree
/// there by luck, and a message that had been wrong all along would look right.
#[test]
fn test_a_site_is_named_by_its_line_in_the_file() {
    let invented = "\
#[cfg(test)]
mod tests {
    fn something() {
        let unrelated = 1;
    }
}

fn signs_in() {
    let controller = MailController::new();
    controller.connect_imap(server, port).await?;
}
";

    assert_eq!(
        the_sign_ins_that_go_round_the_helper(invented),
        vec![SignsInForItself {
            built_at: 9,
            connected_at: 10
        }],
        "the site was named by where it landed after the cut rather than by \
         where it is in the file"
    );
}

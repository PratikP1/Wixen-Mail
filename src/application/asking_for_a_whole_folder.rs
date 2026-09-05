//! Asking for a whole folder, and hearing about it once.
//!
//! A folder view holds five hundred messages and a first sync brings down five
//! hundred, so somebody with a forty thousand message inbox who wants all of it
//! presses Get Older Messages eighty times. This is the request that asks once
//! and keeps going.
//!
//! # Two bounds, and both of them have to move
//!
//! `mail_sync::INITIAL_FETCH_LIMIT` bounds what comes down from the server.
//! `wx_app::FOLDER_LIST_PAGE_SIZE` bounds what is read out of the cache into
//! the list. They are separate numbers that happen to be the same, and moving
//! one alone appears to do nothing, because the other still binds: mail arrives
//! and is not shown, or the list asks for rows that were never fetched. Each
//! carries a comment naming the other for that reason, and so does this
//! paragraph.
//!
//! # What stops it
//!
//! Three things, and there is deliberately no fourth. The folder is complete,
//! which is the ordinary ending. A chunk brought nothing new, which is a server
//! that has stopped handing messages over and would otherwise be asked forever.
//! Or a chunk failed, which is said rather than swallowed.
//!
//! **There is no Stop command**, and that matches the missing message text
//! fetch, which is the other long run in this program and stops only when it
//! reaches the end of its list. Worth knowing before somebody asks for a
//! hundred thousand messages: it runs until it is done. That is one of the two
//! reasons the command says it is experimental where somebody can read it.
//!
//! # What it says while it runs
//!
//! One announcement topic of its own, superseding, so a fetch of eighty chunks
//! speaks a handful of times rather than eighty. See [`THE_PROGRESS_TOPIC`] for
//! why it is not the topic every other status line shares.

/// How much of a folder is on this computer after a chunk landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowMuchIsHere {
    /// How many of the folder's messages this computer now holds.
    pub held: usize,
    /// How many the server says the folder holds.
    pub total_on_server: usize,
}

/// Why a whole-folder request ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowTheRequestEnded {
    /// Every message the server says the folder holds is here.
    TheWholeFolderIsHere { held: usize },
    /// A chunk brought nothing new, so asking again would ask forever.
    ItStoppedComingDown { held: usize, total_on_server: usize },
    /// A chunk failed, in the words of whatever refused.
    AChunkFailed { held: usize, because: String },
}

/// The topic the progress is announced under.
///
/// Its own topic rather than `"status"`. The queue keeps only the newest
/// announcement of a topic, and `"status"` carries every other status line a
/// sync produces, so a fetch running for minutes would silence all of them for
/// as long as it ran. The precedent for splitting is the `"message text"` topic
/// in `wx_app`, which was kept off `"status"` for exactly this reason.
///
/// **This is an assumption Pratik has not confirmed by listening to it.** The
/// argument above is reasoning about how the queue coalesces rather than an
/// observation of what somebody hears, and the opposite choice is defensible:
/// on one topic the progress and the sync lines take turns instead of running
/// in parallel. Moving it is one line, this constant.
pub const THE_PROGRESS_TOPIC: &str = "whole folder";

/// What to say while the fetch is running.
///
/// Both numbers, because they are different facts and the fraction is not the
/// useful part: five hundred of forty thousand and five hundred of six hundred
/// want different decisions from the person hearing them.
pub fn how_far_it_has_got(here: HowMuchIsHere) -> String {
    format!(
        "Downloading this folder: {} of {}.",
        here.held,
        crate::service::caldav::how_many(here.total_on_server, "message")
    )
}

/// What to say when it has finished.
///
/// A count rather than a progress line that happens to be last. Somebody who
/// has been hearing "3500 of 40000" needs to be told this one is the end,
/// and a number with no ending in it reads as another step.
pub fn what_the_request_brought(ended: &HowTheRequestEnded) -> String {
    match ended {
        HowTheRequestEnded::TheWholeFolderIsHere { held } => format!(
            "This folder is downloaded. {} on this computer.",
            crate::service::caldav::how_many(*held, "message")
        ),
        // Said rather than reported as finished. A folder that stopped short
        // and says "downloaded" is a folder somebody searches and gets a
        // shorter answer from than they should.
        HowTheRequestEnded::ItStoppedComingDown {
            held,
            total_on_server,
        } => format!(
            "The mail server stopped sending this folder. {} of {} are on this computer. Ask \
             again to carry on.",
            held,
            crate::service::caldav::how_many(*total_on_server, "message")
        ),
        HowTheRequestEnded::AChunkFailed { held, because } => format!(
            "Downloading this folder stopped: {because} {} are on this computer. Ask again to \
             carry on.",
            crate::service::caldav::how_many(*held, "message")
        ),
    }
}

/// Ask for chunk after chunk until the folder is here.
///
/// The loop, kept here rather than in the window so it can be run without one.
/// `ask_for_another_chunk` brings down the next page and answers how much is
/// here afterwards; `say` is handed each progress line and, last, the count.
///
/// The caller does not press anything again: that is the whole of the request.
pub fn until_the_whole_folder_is_here(
    ask_for_another_chunk: &mut dyn FnMut() -> crate::common::Result<HowMuchIsHere>,
    say: &mut dyn FnMut(&str),
) -> HowTheRequestEnded {
    let mut held_before = None;
    let ended = loop {
        let here = match ask_for_another_chunk() {
            Ok(here) => here,
            Err(why) => {
                break HowTheRequestEnded::AChunkFailed {
                    held: held_before.unwrap_or(0),
                    because: why.to_string(),
                };
            }
        };
        if here.held >= here.total_on_server {
            break HowTheRequestEnded::TheWholeFolderIsHere { held: here.held };
        }
        // Nothing new came down, so asking again would ask forever. Checked
        // after the completeness test rather than before it, because a folder
        // whose last chunk finished it also brings nothing new to the next one,
        // and reporting that as a server that stopped would turn every
        // successful request into a failed one.
        if held_before == Some(here.held) {
            break HowTheRequestEnded::ItStoppedComingDown {
                held: here.held,
                total_on_server: here.total_on_server,
            };
        }
        held_before = Some(here.held);
        say(&how_far_it_has_got(here));
    };
    say(&what_the_request_brought(&ended));
    ended
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder that hands over `page` more messages each time it is asked,
    /// counting the times it was asked.
    fn a_folder_of(
        total: usize,
        page: usize,
    ) -> impl FnMut() -> crate::common::Result<HowMuchIsHere> {
        let mut held = 0usize;
        move || {
            held = (held + page).min(total);
            Ok(HowMuchIsHere {
                held,
                total_on_server: total,
            })
        }
    }

    #[test]
    fn test_the_request_carries_on_past_the_first_chunk_without_being_asked_again() {
        // The requirement. Counted rather than commented: the loop is asked
        // four times for a folder that hands over five hundred at a time, and
        // nothing pressed a key between them.
        let mut asked = 0usize;
        let mut folder = a_folder_of(2000, 500);
        let mut said: Vec<String> = Vec::new();

        let ended = until_the_whole_folder_is_here(
            &mut || {
                asked += 1;
                folder()
            },
            &mut |line| said.push(line.to_string()),
        );

        assert_eq!(
            asked, 4,
            "the request did not carry on past the first chunk"
        );
        assert_eq!(
            ended,
            HowTheRequestEnded::TheWholeFolderIsHere { held: 2000 }
        );
    }

    #[test]
    fn test_the_last_thing_said_is_a_count_rather_than_another_progress_line() {
        // Somebody who has been hearing "500 of 2000" has to be told which one
        // is the end. A number with no ending in it reads as another step.
        let mut folder = a_folder_of(2000, 500);
        let mut said: Vec<String> = Vec::new();

        until_the_whole_folder_is_here(&mut folder, &mut |line| said.push(line.to_string()));

        let last = said.last().expect("the request said nothing at all");
        assert!(
            last.contains("is downloaded") && last.contains("2000"),
            "the last thing said is not a count: {last}"
        );
        assert!(
            !last.starts_with("Downloading this folder:"),
            "the last thing said is another progress line: {last}"
        );
    }

    #[test]
    fn test_a_server_that_stops_sending_ends_the_request_rather_than_being_asked_forever() {
        // A chunk that brings nothing new. Without this the loop asks a server
        // that has stopped answering until somebody closes the program.
        let mut asked = 0usize;
        let mut said: Vec<String> = Vec::new();

        let ended = until_the_whole_folder_is_here(
            &mut || {
                asked += 1;
                Ok(HowMuchIsHere {
                    held: 500,
                    total_on_server: 40_000,
                })
            },
            &mut |line| said.push(line.to_string()),
        );

        assert_eq!(
            asked, 2,
            "the request kept asking a server that had stopped"
        );
        assert_eq!(
            ended,
            HowTheRequestEnded::ItStoppedComingDown {
                held: 500,
                total_on_server: 40_000
            }
        );
        let last = said.last().expect("the request said nothing at all");
        assert!(
            last.contains("stopped sending") && last.contains("Ask again"),
            "a request that stopped short did not say so, or did not say what to \
             do about it: {last}"
        );
    }

    #[test]
    fn test_a_folder_already_here_ends_at_once_and_says_so() {
        let mut said: Vec<String> = Vec::new();

        let ended = until_the_whole_folder_is_here(
            &mut || {
                Ok(HowMuchIsHere {
                    held: 12,
                    total_on_server: 12,
                })
            },
            &mut |line| said.push(line.to_string()),
        );

        assert_eq!(ended, HowTheRequestEnded::TheWholeFolderIsHere { held: 12 });
        assert_eq!(said.len(), 1, "a folder already here said more than once");
    }

    #[test]
    fn test_a_chunk_that_failed_is_said_rather_than_reported_as_a_finished_folder() {
        let mut said: Vec<String> = Vec::new();

        let ended = until_the_whole_folder_is_here(
            &mut || {
                Err(crate::common::Error::InPlainWords(
                    "The mail server would not open this folder.".to_string(),
                ))
            },
            &mut |line| said.push(line.to_string()),
        );

        let HowTheRequestEnded::AChunkFailed { because, .. } = &ended else {
            panic!("a failed chunk was not reported as one: {ended:?}");
        };
        assert!(because.contains("would not open"));
        let last = said.last().expect("the request said nothing at all");
        assert!(
            last.contains("would not open") && last.contains("Ask again"),
            "a failed request did not say why or what to do next: {last}"
        );
    }

    #[test]
    fn test_the_progress_is_not_on_the_topic_every_other_status_line_shares() {
        // The queue keeps only the newest announcement of a topic. On "status"
        // a fetch running for minutes would silence every other status line for
        // as long as it ran.
        assert_ne!(THE_PROGRESS_TOPIC, "status");
    }

    #[test]
    fn test_a_progress_line_gives_both_numbers() {
        // Not a percentage and not a fraction. Five hundred of forty thousand
        // and five hundred of six hundred want different decisions from the
        // person hearing them.
        let said = how_far_it_has_got(HowMuchIsHere {
            held: 500,
            total_on_server: 40_000,
        });

        assert!(
            said.contains("500"),
            "the progress line lost the first number: {said}"
        );
        assert!(
            said.contains("40000"),
            "the progress line lost the second number: {said}"
        );
    }
}

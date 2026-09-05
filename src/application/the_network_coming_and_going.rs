//! What the program believes about the network, and what it does when that
//! belief changes.
//!
//! # Why this is about the change and not about the answer
//!
//! One network loss produces many failures. A mailbox sync walks several
//! folders, each one asks a server and each one fails, and a watch loop is
//! sitting on a connection that has gone as well. Said per failure, that is a
//! screen reader repeating the same sentence eight times while somebody is
//! trying to read a message, which is the flood guardrail 5 is about.
//!
//! So nothing here is said about an answer. Things are said about a change: the
//! network was there and now is not, or was not and now is. Asking ten times
//! while it is still gone is ten answers and no change, so it is silent. That
//! is not a rate limit, which would still say the same thing twice given long
//! enough. It is the fact being a transition rather than a state.
//!
//! # What the program starts believing
//!
//! Whatever it finds. A laptop opened on a train with no signal has not lost
//! anything, so nothing is announced, and offline mode is left where somebody
//! put it. [`WhatTheProgramBelieves::to_begin_with`] is that first look, and it
//! is deliberately not a change.
//!
//! # What is not here
//!
//! Values in and values out. No platform call, no window and no connection, so
//! the counting can be driven directly rather than by pulling a cable. Asking
//! the machine is `service::network`, and acting on the answer is the window's.
//!
//! Nothing here sends mail, and nothing here can. Coming back to a network is
//! an offer, never an act: mail leaving this computer is publishing, and
//! guardrail 7 says publishing happens because somebody asked. The answer to
//! the network returning is therefore [`WhatToDoAboutIt::OfferToGoBackOnline`],
//! which is a thing to put in front of somebody, and the whole of what this
//! module does about it.

use crate::service::network::WhetherThereIsANetwork;

/// What a fresh look at the network calls for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatToDoAboutIt {
    /// Nothing changed. Most looks are this one.
    Nothing,
    /// The network was there and has gone. Say so once and turn offline mode
    /// on, so outgoing mail waits rather than being thrown at a server that is
    /// not there.
    SayItWentAndGoOffline,
    /// The network is back. Offer to go online, and send nothing until
    /// somebody takes the offer.
    OfferToGoBackOnline,
}

/// What the program believes about the network.
///
/// One fact, held so that the next answer can be compared with it. Without the
/// comparison there is no change to speak of, only a stream of answers, and
/// speaking about answers is the flood this exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatTheProgramBelieves {
    there_is_a_network: bool,
}

impl WhatTheProgramBelieves {
    /// The first look, which is never a change.
    ///
    /// A program that started with no network has not lost one. Announcing
    /// here would tell somebody who has been on a train for an hour that the
    /// network has just gone.
    pub fn to_begin_with(found: WhetherThereIsANetwork) -> Self {
        Self {
            there_is_a_network: matches!(found, WhetherThereIsANetwork::ThereIsOne),
        }
    }

    /// A fresh look, and what it calls for.
    ///
    /// The belief is written down whatever the answer, so a run of identical
    /// answers is one change and then nothing.
    pub fn told(&mut self, found: WhetherThereIsANetwork) -> WhatToDoAboutIt {
        let there_is_a_network = matches!(found, WhetherThereIsANetwork::ThereIsOne);
        let was = std::mem::replace(&mut self.there_is_a_network, there_is_a_network);
        match (was, there_is_a_network) {
            (true, false) => WhatToDoAboutIt::SayItWentAndGoOffline,
            (false, true) => WhatToDoAboutIt::OfferToGoBackOnline,
            _ => WhatToDoAboutIt::Nothing,
        }
    }

    /// Whether the program currently believes there is a network.
    pub fn there_is_a_network(&self) -> bool {
        self.there_is_a_network
    }
}

/// The sentence somebody is given about the network, or nothing to say.
///
/// One string, handed to the status bar and to the announcement queue by the
/// arm that receives it. Two strings built in two places is how the words
/// somebody reads and the words somebody hears come apart, and a deaf user and
/// a blind user being told different things about the same event is the defect
/// this shape exists to prevent. The offer's own button label follows the same
/// rule for the same reason.
pub fn what_to_say_about_the_network(news: WhatToDoAboutIt) -> Option<&'static str> {
    match news {
        WhatToDoAboutIt::Nothing => None,
        WhatToDoAboutIt::SayItWentAndGoOffline => Some(
            "The network has gone, so Wixen Mail is now offline. \
             Mail you send waits in the Outbox until you go back online.",
        ),
        WhatToDoAboutIt::OfferToGoBackOnline => Some(
            "The network is back. Wixen Mail is still offline, and nothing in \
             the Outbox has been sent. Go back online when you are ready.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use WhetherThereIsANetwork::{ThereIsNone, ThereIsOne};

    #[test]
    fn test_ten_failures_from_one_loss_are_announced_once() {
        // The behaviour this module exists for. A sync across several folders
        // plus a watch loop turns one unplugged cable into many failures, and
        // each of them can prompt another look. Said per look, somebody trying
        // to read a message hears the same sentence ten times.
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsOne);

        let said = (0..10)
            .filter(|_| believes.told(ThereIsNone) == WhatToDoAboutIt::SayItWentAndGoOffline)
            .count();

        assert_eq!(
            said, 1,
            "ten looks at one loss produced {said} announcements"
        );
    }

    #[test]
    fn test_a_network_that_was_already_gone_at_the_start_is_not_a_change() {
        // A laptop opened on a train has lost nothing. Announcing here says
        // the network has just gone to somebody who has not had one all
        // morning.
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsNone);

        assert_eq!(believes.told(ThereIsNone), WhatToDoAboutIt::Nothing);
    }

    #[test]
    fn test_asking_again_while_the_network_is_still_there_says_nothing() {
        // The ordinary case, and it runs several times a minute for as long as
        // the program is open.
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsOne);

        for _ in 0..50 {
            assert_eq!(believes.told(ThereIsOne), WhatToDoAboutIt::Nothing);
        }
    }

    #[test]
    fn test_the_network_coming_back_is_an_offer_rather_than_a_loss() {
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsOne);
        believes.told(ThereIsNone);

        assert_eq!(
            believes.told(ThereIsOne),
            WhatToDoAboutIt::OfferToGoBackOnline
        );
    }

    #[test]
    fn test_the_network_coming_back_and_going_again_is_one_of_each_each_time() {
        // Two round trips, because a state machine that answers correctly once
        // and then latches would pass a single trip.
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsOne);

        let mut answers = Vec::new();
        for _ in 0..2 {
            answers.push(believes.told(ThereIsNone));
            answers.push(believes.told(ThereIsNone));
            answers.push(believes.told(ThereIsOne));
            answers.push(believes.told(ThereIsOne));
        }

        assert_eq!(
            answers,
            vec![
                WhatToDoAboutIt::SayItWentAndGoOffline,
                WhatToDoAboutIt::Nothing,
                WhatToDoAboutIt::OfferToGoBackOnline,
                WhatToDoAboutIt::Nothing,
                WhatToDoAboutIt::SayItWentAndGoOffline,
                WhatToDoAboutIt::Nothing,
                WhatToDoAboutIt::OfferToGoBackOnline,
                WhatToDoAboutIt::Nothing,
            ]
        );
    }

    #[test]
    fn test_the_belief_follows_the_last_answer_whatever_it_calls_for() {
        // The counting above rests on this. A belief that is only written down
        // when something is announced would answer the second identical look
        // as a change all over again.
        let mut believes = WhatTheProgramBelieves::to_begin_with(ThereIsOne);
        believes.told(ThereIsNone);

        assert!(!believes.there_is_a_network());
    }

    #[test]
    fn test_nothing_to_do_is_the_one_answer_with_nothing_to_say() {
        // Guardrail 5 the other way round: silence has to be silent. An empty
        // sentence would still reach the queue and still interrupt.
        assert_eq!(
            what_to_say_about_the_network(WhatToDoAboutIt::Nothing),
            None
        );
    }

    #[test]
    fn test_the_two_things_worth_saying_are_two_different_sentences() {
        let went = what_to_say_about_the_network(WhatToDoAboutIt::SayItWentAndGoOffline)
            .expect("a sentence about the network going");
        let back = what_to_say_about_the_network(WhatToDoAboutIt::OfferToGoBackOnline)
            .expect("a sentence about the network coming back");

        assert_ne!(went, back);
        // The one that matters most: coming back must not read as having been
        // acted on. Nothing has been sent and the sentence has to say so.
        // The whole clause is pinned rather than a word of it, and on purpose.
        // This is the one promise guardrail 7 makes to somebody: the network
        // came back and their mail did not go with it. A reword that drops it
        // is the thing worth being stopped by, and a looser check would let it
        // through.
        assert!(
            back.contains("nothing in the Outbox has been sent"),
            "the sentence about the network returning does not say the Outbox is \
             untouched, so it reads as mail having gone: {back}"
        );
        assert!(
            went.contains("Outbox"),
            "the sentence about the network going does not say where mail is \
             waiting: {went}"
        );
    }
}

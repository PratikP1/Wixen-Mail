//! Which pictures in a message are shown, and which would report you.
//!
//! # The two kinds, and why they are not the same
//!
//! A picture in a message arrives one of two ways. It is **carried**, as a part
//! of the message itself, referred to by `cid:`; or it is **pointed at**, as an
//! address somewhere on the internet.
//!
//! A carried picture costs nothing to show. It is already on this computer, and
//! showing it tells nobody anything.
//!
//! A pointed-at picture has to be fetched, and fetching it tells the server it
//! came from that this message was opened, by this computer, at this moment. A
//! single invisible one-pixel image is the whole of how mail tracking works.
//! Every marketing message carries one, and so does a good deal of worse than
//! marketing.
//!
//! So the two get opposite defaults: carried pictures are shown, pointed-at
//! ones are not until somebody asks.
//!
//! # What was happening before this
//!
//! Exactly backwards. The sanitizer stripped `cid:` addresses, because they are
//! not a scheme it recognises, so the safe pictures did not appear at all. And
//! it kept the remote ones, which the browser then fetched, so every tracking
//! pixel in every message did its job. Nobody had chosen either of those.

/// Whether pictures that have to be fetched may be fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetching {
    /// Leave them alone. Nothing is asked for, nobody is told.
    Blocked,
    /// Fetch them, which tells each sender their message was opened.
    Allowed,
}

impl Fetching {
    /// What the setting means, read the way the setting is worded.
    ///
    /// The setting asks whether to block, and blocking is what is on by
    /// default, so the ordinary answer is the safe one and somebody who does
    /// nothing is not tracked.
    pub fn from_setting(blocked: bool) -> Self {
        if blocked {
            Fetching::Blocked
        } else {
            Fetching::Allowed
        }
    }
}

/// What to do about one picture's address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// Show it. It is already here.
    ItIsCarried,
    /// Show it, and something on the internet learns this message was opened.
    ItWillBeFetched,
    /// Do not show it, and say so where it would have been.
    HeldBack,
}

/// What to do with the address a picture points at.
///
/// An address that is neither carried nor fetchable is held back too: an
/// address nobody recognises is not one to hand to a browser.
pub fn what_to_do_about(address: &str, fetching: Fetching) -> Showing {
    let address = address.trim();
    if address.to_ascii_lowercase().starts_with("cid:") {
        return Showing::ItIsCarried;
    }
    let reaches_out = ["http://", "https://", "//"]
        .iter()
        .any(|start| address.to_ascii_lowercase().starts_with(start));
    match (reaches_out, fetching) {
        (true, Fetching::Allowed) => Showing::ItWillBeFetched,
        (true, Fetching::Blocked) => Showing::HeldBack,
        // A `data:` address carries its own picture and asks nobody for
        // anything, but this application does not pass a sender's along: a
        // `data:` picture can be an SVG, and an SVG can carry a script.
        (false, _) => Showing::HeldBack,
    }
}

/// What to say where a held-back picture would have been.
///
/// The sender's own description when they gave one, so somebody knows what
/// they are choosing whether to fetch, and a plain statement when they did not.
/// Either way it says the picture was held back rather than leaving a gap that
/// reads as a message with nothing in it.
pub fn what_stands_in_for_it(described: &str) -> String {
    let described = described.trim();
    if described.is_empty() {
        "[Picture not shown, and the sender did not describe it]".to_string()
    } else {
        format!("[Picture not shown: {described}]")
    }
}

/// What to say about a whole message once its pictures have been counted.
///
/// Empty when nothing was held back, so an ordinary message says nothing. The
/// count matters more than it looks: one held-back picture in a message from a
/// person is usually their signature, and thirty is a mailing.
pub fn what_was_held_back(held_back: usize) -> String {
    match held_back {
        0 => String::new(),
        1 => "1 picture was not shown, because fetching it would have told the \
              sender you opened this. Settings, Reading has the switch."
            .to_string(),
        many => format!(
            "{many} pictures were not shown, because fetching them would have told \
             the senders you opened this. Settings, Reading has the switch."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_picture_the_message_carries_is_shown_whatever_the_setting_says() {
        // It is already on this computer. Showing it asks nobody for anything,
        // so there is nothing for the setting to protect against.
        for fetching in [Fetching::Blocked, Fetching::Allowed] {
            assert_eq!(
                what_to_do_about("cid:part1@example.com", fetching),
                Showing::ItIsCarried,
                "a carried picture was held back with fetching {fetching:?}"
            );
        }
    }

    #[test]
    fn test_a_carried_picture_is_recognised_however_it_is_capitalised() {
        // Mail headers and addresses come from every mail program ever
        // written, and the scheme is compared without case everywhere else.
        assert_eq!(
            what_to_do_about("CID:Part1@example.com", Fetching::Blocked),
            Showing::ItIsCarried
        );
    }

    #[test]
    fn test_a_picture_somewhere_on_the_internet_is_held_back_by_default() {
        // The whole point. One invisible pixel is how mail tracking works.
        assert_eq!(
            what_to_do_about("https://tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
        assert_eq!(
            what_to_do_about("http://tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
    }

    #[test]
    fn test_an_address_with_no_scheme_of_its_own_still_reaches_out() {
        // `//host/x.gif` takes the scheme of the page around it, which for a
        // message shown in a browser is a real fetch. Missing this would leave
        // the commonest way of writing a tracking pixel working.
        assert_eq!(
            what_to_do_about("//tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
    }

    #[test]
    fn test_somebody_who_asks_for_them_gets_them_and_is_told_what_that_means() {
        assert_eq!(
            what_to_do_about("https://example.com/photo.jpg", Fetching::Allowed),
            Showing::ItWillBeFetched
        );
    }

    #[test]
    fn test_an_address_this_does_not_understand_is_held_back() {
        // Including `data:`, which carries its own picture and asks nobody for
        // anything, and is still not passed along: it can be an SVG and an SVG
        // can carry a script.
        for address in [
            "data:image/svg+xml,<svg onload=alert(1)>",
            "javascript:alert(1)",
            "ftp://example.com/x.gif",
            "",
        ] {
            assert_eq!(
                what_to_do_about(address, Fetching::Allowed),
                Showing::HeldBack,
                "{address:?} was not held back"
            );
        }
    }

    #[test]
    fn test_the_setting_reads_the_way_it_is_worded() {
        // The switch asks whether to block, and blocking is what is on by
        // default, so somebody who changes nothing is not tracked.
        assert_eq!(Fetching::from_setting(true), Fetching::Blocked);
        assert_eq!(Fetching::from_setting(false), Fetching::Allowed);
    }

    #[test]
    fn test_a_held_back_picture_leaves_the_senders_description_in_its_place() {
        // So somebody knows what they would be fetching before they decide to.
        let said = what_stands_in_for_it("Our new spring range");

        assert!(said.contains("Our new spring range"), "{said}");
        assert!(said.contains("not shown"), "{said}");
    }

    #[test]
    fn test_a_held_back_picture_nobody_described_still_leaves_a_mark() {
        // A gap where a picture was reads as a message with a hole in it. The
        // sender's missing description is said rather than hidden.
        let said = what_stands_in_for_it("   ");

        assert!(said.contains("did not describe"), "{said}");
    }

    #[test]
    fn test_a_message_with_nothing_held_back_says_nothing() {
        assert!(what_was_held_back(0).is_empty());
    }

    #[test]
    fn test_one_held_back_picture_is_not_reported_in_the_plural() {
        let said = what_was_held_back(1);

        assert!(said.starts_with("1 picture was"), "{said}");
        assert!(!said.contains("pictures"), "{said}");
    }

    #[test]
    fn test_the_report_says_how_many_and_where_to_change_it() {
        // The count tells a signature apart from a mailing, and naming the
        // setting is the difference between a warning and a dead end.
        let said = what_was_held_back(30);

        assert!(said.starts_with("30 pictures"), "{said}");
        assert!(said.contains("Settings, Reading"), "{said}");
    }
}

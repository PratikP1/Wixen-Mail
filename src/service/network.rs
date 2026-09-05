//! Whether this machine has a network.
//!
//! One question, asked of the platform and answered as a value. What the
//! program then believes about the network, and what it does when that belief
//! changes, is `application::the_network_coming_and_going` and not this. The
//! two are kept apart because only one of them can be tested: this one is a
//! call into Windows whose answer depends on the cable, and that one is values
//! in and values out.
//!
//! # What this can and cannot see
//!
//! Windows is asked whether this computer has a connection at all. That
//! notices a cable pulled out, a wifi dropped, and a laptop closed and
//! reopened somewhere else. It does not notice a network that is up and cannot
//! reach a mail server, which is a different question and one only a request to
//! that server answers.
//!
//! Being wrong has two directions and they do not cost the same. Answering
//! "there is one" when there is not leaves the program exactly as it behaves
//! today: a request fails and says so. Answering "there is none" when there is
//! one puts somebody into offline mode who did not ask to be there. So the
//! answer this leans towards, everywhere it has a choice, is that there is a
//! network.
//!
//! # Windows only
//!
//! `InternetGetConnectedState` is a flat call into `wininet`, which is how
//! everything except the platform spell checker reaches Windows in this
//! project. There is no equivalent written for anywhere else, so every other
//! platform gets the answer that leaves behaviour unchanged.

/// Whether this machine has a network, as far as it can be told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhetherThereIsANetwork {
    /// This computer has a connection.
    ThereIsOne,
    /// It has none. Nothing will reach a server until that changes.
    ThereIsNone,
}

/// Whether this machine has a network.
#[cfg(target_os = "windows")]
pub fn whether_there_is_a_network() -> WhetherThereIsANetwork {
    #[link(name = "wininet")]
    unsafe extern "system" {
        fn InternetGetConnectedState(flags: *mut u32, reserved: u32) -> i32;
    }

    let mut how = 0u32;
    // Safe: `how` lives across the call and Windows writes at most a `u32`
    // into it. The second argument is documented as reserved and must be
    // nought.
    let answered = unsafe { InternetGetConnectedState(&raw mut how, 0) };
    what_that_answer_means(answered)
}

/// What Windows' answer means.
///
/// Apart from the call, so the reading can be tested. `InternetGetConnectedState`
/// hands back a `BOOL`, and a `BOOL` read the wrong way round here would put
/// every Windows user into offline mode and leave them there, with the only way
/// out being a menu they would have to find. Nothing at the call site says
/// which way round it is, which is exactly the kind of thing that is got
/// backwards once and never noticed.
#[cfg(target_os = "windows")]
fn what_that_answer_means(windows_said_connected: i32) -> WhetherThereIsANetwork {
    match windows_said_connected {
        0 => WhetherThereIsANetwork::ThereIsNone,
        _ => WhetherThereIsANetwork::ThereIsOne,
    }
}

/// Whether this machine has a network, where nothing has been written to ask.
///
/// There is one, always, and that is the honest answer rather than a shrug.
/// Nothing here can tell, so the choice is between the answer that changes
/// behaviour and the answer that does not, and this is the one that does not:
/// the program goes on as it did before this module existed, a request fails
/// when there is no network and says so, and offline mode stays a switch
/// somebody sets by hand.
///
/// The other answer would be worse than useless. A build that decided there was
/// no network would put somebody into offline mode on every check and never
/// take them out of it, because the same absent detection can never say the
/// network came back.
#[cfg(not(target_os = "windows"))]
pub fn whether_there_is_a_network() -> WhetherThereIsANetwork {
    WhetherThereIsANetwork::ThereIsOne
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_saying_nought_means_there_is_no_network() {
        // The polarity. Backwards, this is a build that is permanently offline
        // and says the network went every time it is asked.
        assert_eq!(
            what_that_answer_means(0),
            WhetherThereIsANetwork::ThereIsNone
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_any_other_answer_from_windows_means_there_is_a_network() {
        // A `BOOL` is documented as nought or not nought rather than as nought
        // or one, and code that tested for one has been wrong about it before.
        for said in [1, 2, -1, i32::MAX] {
            assert_eq!(
                what_that_answer_means(said),
                WhetherThereIsANetwork::ThereIsOne,
                "{said} was read as no network"
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_a_platform_with_no_detection_says_there_is_a_network() {
        // The answer that leaves behaviour where it was. The other one strands
        // somebody offline with nothing able to bring them back.
        assert_eq!(
            whether_there_is_a_network(),
            WhetherThereIsANetwork::ThereIsOne
        );
    }
}

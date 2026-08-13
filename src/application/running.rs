//! Whether another copy of Wixen Mail is already running.
//!
//! Two things need to know, and both of them destroy data if they get it
//! wrong.
//!
//! **The uninstaller.** It deletes the data folder and clears the credential
//! store while the program may still be open, holding the database, writing
//! its settings back on the way out, and locking its own executable. Half of
//! it goes, the rest is rewritten by the copy still running, and what is left
//! is an application that is neither installed nor removed.
//!
//! **`--erase-all-data` typed by hand.** Same destruction, no wizard in front
//! of it.
//!
//! # How it is known
//!
//! A named mutex, held by the running program for as long as it is up. Windows
//! releases it when the process ends, including when it crashes, so there is
//! no stale marker to clear and no file left behind to confuse the next start.
//!
//! Inno Setup reads the same name through its `AppMutex` directive, which is
//! why [`MUTEX_NAME`] is a constant with a test tying it to the installer
//! script. That is what stops the uninstall before it starts, with a message
//! asking the person to close the program. The check in this module is the
//! backstop underneath it.
//!
//! # What it does not cover
//!
//! The name has no namespace prefix, so it lives in the logon session's
//! namespace. Two sessions belonging to the same person, which is what a
//! second Remote Desktop connection gives you, get one each and cannot see the
//! other's, although they share a data folder. The alternative is the `Global`
//! namespace, and creating an object there needs a privilege an ordinary
//! account does not have, so it would fail for exactly the people this is for.

/// The name the running program holds and the installer looks for.
///
/// Changing it means changing `AppMutex` in `installer/Wixen-Mail-Setup.iss`
/// in the same commit. A test fails if the two drift apart, because the
/// failure otherwise is silent: the uninstaller looks for a mutex nobody
/// holds, finds nothing, and cheerfully deletes everything under a running
/// program.
pub const MUTEX_NAME: &str = "WixenMail-Running";

/// What was found when this process asked, and marked itself running.
///
/// Both of the first two carry the marker, because both mean a copy of Wixen
/// Mail is up. Being the second window open does not make a copy less real,
/// and if only the first held the marker then closing the first would tell the
/// uninstaller that nothing was running while a window was still on screen.
#[derive(Debug)]
pub enum Claim {
    /// Nothing else was running, and now this process is marked.
    Granted(Marker),
    /// Something else is running. This process is marked as well.
    ///
    /// Anything that destroys data should stop here. Anything that only wants
    /// to open a window can carry on, holding the marker.
    AnotherIsRunning(Marker),
    /// Windows would not say, which is not the same as "no".
    ///
    /// Carries the reason so it can be written down rather than swallowed.
    /// Callers that are about to delete something should say so and continue
    /// rather than refuse: refusing on a question that cannot be answered
    /// would leave somebody unable to uninstall at all.
    Unknown(String),
}

/// A held mark saying a copy of Wixen Mail is running.
///
/// Hold it for as long as that is true. Dropping it gives up this process's
/// share of the mark, so binding it to `_` rather than to a name would give it
/// up on the line it was taken.
#[derive(Debug)]
pub struct Marker {
    #[cfg(windows)]
    handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Drop for Marker {
    fn drop(&mut self) {
        // The name lives as long as any handle to it is open, so this is a
        // share of the mark rather than the whole of it. The process ending
        // would close it anyway; doing it here means a mark taken for one
        // piece of work does not outlast that work.
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.handle) };
    }
}

/// Try to become the only copy of Wixen Mail running.
pub fn claim() -> Claim {
    claim_named(MUTEX_NAME)
}

/// The same, under a name of your choosing.
///
/// Only [`claim`] should be used outside tests. It exists because the name is
/// shared by everything in the logon session, so two tests both taking
/// [`MUTEX_NAME`] would decide each other's answers: Rust runs them in
/// parallel, and whichever went first would be told it was alone while the
/// other was told it was not. A test naming its own marker also cannot be
/// upset by a real Wixen Mail being open on the machine running the suite.
#[cfg(windows)]
fn claim_named(name: &str) -> Claim {
    use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::core::HSTRING;

    let name = HSTRING::from(name);

    // Not `bInitialOwner`: ownership is irrelevant here and taking it would
    // mean this process has to be the one that releases it. The mutex exists
    // to be named, not to be waited on.
    let created = unsafe { CreateMutexW(None, false, &name) };

    match created {
        Ok(handle) => {
            // The handle is valid either way, and the name lives as long as
            // any handle to it is open, so it is kept in both cases. Whether
            // the name was already there is only knowable from the error code,
            // and it has to be read straight after the call.
            let marker = Marker { handle };
            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                Claim::AnotherIsRunning(marker)
            } else {
                Claim::Granted(marker)
            }
        }
        Err(e) => Claim::Unknown(format!("Windows would not open the marker: {e}")),
    }
}

/// There is no answer off Windows. The accessibility layer this application
/// exists for is Windows-only, so this is a build that cannot be shipped
/// rather than one that can be shipped without the check.
#[cfg(not(windows))]
fn claim_named(_name: &str) -> Claim {
    Claim::Unknown("only Windows is asked this question".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn test_a_second_claim_is_told_the_first_one_is_running() {
        // The whole mechanism in three lines. Windows reports the name is
        // taken even within one process, which is what makes this testable
        // without starting a second program.
        let name = "WixenMail-Test-SecondClaim";
        let held = claim_named(name);
        assert!(
            matches!(held, Claim::Granted(_)),
            "nothing else should be holding this name: {held:?}"
        );

        let second = claim_named(name);

        assert!(
            matches!(second, Claim::AnotherIsRunning(_)),
            "a second claim while the first is held must be refused, got {second:?}"
        );
        drop(held);
    }

    #[cfg(windows)]
    #[test]
    fn test_giving_it_up_lets_the_next_one_have_it() {
        // Otherwise closing Wixen Mail would leave the machine unable to
        // uninstall it, which is worse than the problem being solved.
        let name = "WixenMail-Test-GivenUp";
        drop(claim_named(name));

        let next = claim_named(name);

        assert!(
            matches!(next, Claim::Granted(_)),
            "the name should be free again, got {next:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_a_later_copy_keeps_the_marker_alive_after_the_first_one_goes() {
        // The hole this closes: two windows open, the first is closed, and
        // the marker went with it. The second is still running, the
        // uninstaller asks, is told nothing is running, and deletes the mail
        // out from under an open program. Being second does not make a copy
        // less real, so it holds the name too.
        let name = "WixenMail-Test-StillOpen";
        let first = claim_named(name);
        let second = claim_named(name);
        assert!(matches!(first, Claim::Granted(_)), "{first:?}");
        assert!(matches!(second, Claim::AnotherIsRunning(_)), "{second:?}");

        drop(first);
        let asking = claim_named(name);

        assert!(
            matches!(asking, Claim::AnotherIsRunning(_)),
            "the second copy is still open, so the answer is still yes, got {asking:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_two_different_names_do_not_see_each_other() {
        // What makes the two tests above independent, and worth stating: if
        // the name were ignored they would pass or fail together depending on
        // which ran first, and the suite would be flaky rather than wrong.
        let first = claim_named("WixenMail-Test-Apart-One");
        let second = claim_named("WixenMail-Test-Apart-Two");

        assert!(matches!(first, Claim::Granted(_)), "{first:?}");
        assert!(matches!(second, Claim::Granted(_)), "{second:?}");
    }

    #[test]
    fn test_the_installer_looks_for_the_same_name() {
        // The installer script is the artefact, and it cannot be run from here.
        // What this cannot see is whether the installer works, only whether it
        // spells the same name this code holds.
        // The failure this prevents is silent. Rename the mutex here alone and
        // the uninstaller checks for something nobody holds, finds it free,
        // and deletes the data folder under a running program.
        let installer = std::fs::read_to_string("installer/Wixen-Mail-Setup.iss")
            .expect("the installer script");

        assert!(
            installer.contains(&format!("AppMutex={MUTEX_NAME}")),
            "installer/Wixen-Mail-Setup.iss does not set AppMutex={MUTEX_NAME}"
        );
    }
}

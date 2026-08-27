//! Who this program is running as.
//!
//! Two questions the setup tool has to answer before it can do anything useful,
//! and neither can be worked out without asking Windows.
//!
//! # The account running the tool is not always the right one
//!
//! A crawl scope rule names one person's mail, and the obvious way to find out
//! whose is to ask the running process. That is right most of the time and
//! wrong in one case that matters: when a standard user runs this and types
//! somebody else's administrator details at the elevation prompt, the process
//! runs as that other account, and a rule built from it would name the
//! administrator's mail. There is no way from inside the elevated process to see
//! who was signed in before it started, so the tool takes `--user` and its
//! report always says which identifier it used. That is the whole answer to this
//! problem: it is visible, rather than fixed.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TOKEN_USER, TokenElevation, TokenUser,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_core::PWSTR;

/// The security identifier of the account running this program.
///
/// `None` when Windows would not answer. There is nothing sensible to fall back
/// to: a guessed identifier builds a rule naming somebody else's mail.
pub fn current_user_sid() -> Option<String> {
    let token = own_token()?;
    let room = token_user(token)?;
    // The buffer is a slice of eight byte words rather than of bytes, because
    // TOKEN_USER holds a pointer and reading a pointer out of a buffer that
    // Windows only guaranteed to be byte aligned is undefined. A Vec<u8> would
    // work on this processor and is still wrong.
    let user = room.as_ptr().cast::<TOKEN_USER>();
    let mut written = PWSTR::null();

    let converted = unsafe { ConvertSidToStringSidW((*user).User.Sid, &mut written) };
    let sid = match converted.is_ok() && !written.is_null() {
        // Safe because the call succeeded, which is Windows promising a
        // null-terminated string it allocated and this code now owns.
        true => unsafe { written.to_string() }.ok(),
        false => None,
    };
    if !written.is_null() {
        unsafe { LocalFree(Some(HLOCAL(written.0.cast()))) };
    }
    unsafe {
        let _ = CloseHandle(token);
    };

    // Windows writes the identifier with a leading S and no braces, which is
    // the form the rest of this crate expects, so it is passed on as it came.
    sid
}

/// Whether this program is running with administrator rights.
///
/// `None` when Windows would not answer, which is reported as not knowing
/// rather than as not elevated. Telling somebody they need to elevate when they
/// already have would send them round a loop they cannot get out of.
///
/// The buffer is exactly the size of the answer, not merely big enough, and
/// that is the whole of what was wrong here the first time. This was written
/// like [`current_user_sid`] below: ask how much room is needed, round it up to
/// something aligned, ask again. Windows really does say it needs four bytes,
/// and then refuses the second call because eight bytes were offered for a four
/// byte answer. A fixed-size token value wants its own size and nothing else.
///
/// It reported "Windows would not say" on a machine where the answer was
/// plainly available, and nothing in the crate noticed until somebody ran the
/// tool and read the line.
pub fn is_elevated() -> Option<bool> {
    let token = own_token()?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut written = 0u32;

    let asked = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut written,
        )
    };
    unsafe {
        let _ = CloseHandle(token);
    };

    asked.ok().map(|()| elevation.TokenIsElevated != 0)
}

/// This process's own access token, open for reading.
fn own_token() -> Option<HANDLE> {
    let mut token = HANDLE::default();

    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .ok()
        .map(|()| token)
}

/// The token's user record, in a buffer big enough to hold it.
///
/// Asked for twice on purpose. This record ends with a security identifier,
/// which has no fixed length, so the first call is told there is no room and
/// answers with how much it needs and the second one gets it. Guessing a size
/// instead is how this silently truncates on the one machine with a longer
/// identifier than the guess allowed for.
fn token_user(token: HANDLE) -> Option<Vec<u64>> {
    let mut needed = 0u32;
    // Expected to fail. Its only job is to fill in how much room to make.
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut needed) };
    if needed == 0 {
        return None;
    }

    let words = (needed as usize).div_ceil(size_of::<u64>());
    let mut room = vec![0u64; words];
    let have = (words * size_of::<u64>()) as u32;

    unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            Some(room.as_mut_ptr().cast()),
            have,
            &mut needed,
        )
    }
    .ok()
    .map(|()| room)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_really_answers_who_this_is_and_whether_it_is_elevated() {
        // Neither of these can be worked out without asking Windows, so neither
        // has a pure part to test. What can be tested is that the calls really
        // answer on a machine this crate can run on, and that is not a small
        // claim: the elevation question was written the same way as the
        // identity question, which offers a buffer rounded up to something
        // aligned. Windows refuses that for a fixed size answer and wants the
        // exact size. It returned "Windows would not say" on a working machine,
        // and nothing in the crate noticed until somebody ran the tool and read
        // the line.
        let sid = current_user_sid().expect("Windows would not say who this is");

        assert!(
            sid.starts_with("S-1-"),
            "{sid} is not a security identifier"
        );
        assert!(
            is_elevated().is_some(),
            "Windows would not say whether this is elevated"
        );
    }

    #[test]
    fn test_the_identifier_is_in_the_form_a_scope_rule_can_be_built_from() {
        // The identifier goes straight into a url, and crate::scope refuses one
        // that is not letters, digits and hyphens. Braces or a trailing space
        // from Windows would turn every install into a refusal that names the
        // account rather than the real problem.
        let sid = current_user_sid().expect("Windows would not say who this is");

        assert!(
            sid.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "{sid} has something in it a url cannot carry"
        );
        assert!(crate::scope::plan_for(&sid).is_ok(), "{sid}");
    }
}

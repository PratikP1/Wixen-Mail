//! The pipe one copy of Wixen Mail uses to reach the copy already running.
//!
//! [`crate::application::handover`] decides what is said and whether to say it.
//! This carries it, and holds no rules of its own beyond the ones a pipe forces.
//!
//! # Why a named pipe
//!
//! The other usual answer on Windows is `WM_COPYDATA`, which means finding the
//! window by its class name and handling a message inside the window
//! procedure. wxWidgets owns that procedure and wxdragon does not offer a way
//! in, so it would mean reaching around the toolkit. A pipe needs neither, and
//! the copy listening on it already has a channel for telling its own interface
//! that something happened.
//!
//! # Why the name carries the session
//!
//! A pipe name is visible to the whole machine, unlike the mutex beside it in
//! [`crate::application::running`], which lives in the logon session. Two people
//! signed in at once and both running Wixen Mail would otherwise want one name,
//! and the second would fail to listen and quietly hand its links to the first
//! person's copy. The session number in the name is what keeps them apart.
//!
//! # What a listener may assume
//!
//! Nothing. Any program running on this machine can open a pipe whose name it
//! knows, so what arrives is checked before it is read, and what survives that
//! check is still only a string that goes through the same parser a command
//! line does.

use crate::common::{Error, Result};

/// How long a copy waits for the running one to take its message.
///
/// Short on purpose. This runs while somebody is waiting for a window to
/// appear after clicking a link, and a copy that cannot hand over quickly is
/// better off starting normally than standing still. The cost of being wrong
/// is a second window, not a lost message.
const HOW_LONG_TO_WAIT: u32 = 3_000;

/// The pipe this session listens on.
///
/// The session number rather than the user's name: it is what Windows itself
/// uses to keep two signed-in people's programs apart, it is a number so it
/// cannot carry a character a pipe name may not hold, and it costs one call.
pub fn pipe_name() -> String {
    format!(r"\\.\pipe\wixen-mail-handover-{}", this_session())
}

#[cfg(target_os = "windows")]
fn this_session() -> u32 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcessId() -> u32;
        fn ProcessIdToSessionId(process: u32, session: *mut u32) -> i32;
    }
    let mut session: u32 = 0;
    // Safe: both arguments are owned here and live across the call. A failure
    // leaves the nought already there, which is the console session and is the
    // right answer for the ordinary case where this cannot fail anyway.
    let answered = unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &raw mut session) };
    if answered == 0 { 0 } else { session }
}

#[cfg(not(target_os = "windows"))]
fn this_session() -> u32 {
    0
}

/// Give an argument to the copy already running.
///
/// `Ok(())` means it was taken, and the caller should stop. An error means
/// nobody took it, and the caller should carry on and be the copy that runs:
/// the copy holding the mutex may be shutting down, or may have died between
/// the mutex being seen and this being tried.
#[cfg(target_os = "windows")]
pub fn hand_over(argument: &str) -> Result<()> {
    hand_over_on(&pipe_name(), argument)
}

/// The same, on a named pipe of the caller's choosing.
///
/// Split out so a test can use a pipe of its own. Without this the test used
/// the real name, which meant that on a machine where somebody had Wixen Mail
/// open, running the tests handed a link to their running program and opened a
/// composer in front of them. That happened.
#[cfg(target_os = "windows")]
fn hand_over_on(pipe: &str, argument: &str) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const GENERIC_WRITE: u32 = 0x4000_0000;
    const OPEN_EXISTING: u32 = 3;
    const INVALID_HANDLE_VALUE: isize = -1;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn WaitNamedPipeW(name: *const u16, timeout: u32) -> i32;
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *mut core::ffi::c_void,
            creation: u32,
            flags: u32,
            template: isize,
        ) -> isize;
        fn WriteFile(
            file: isize,
            buffer: *const u8,
            to_write: u32,
            written: *mut u32,
            overlapped: *mut core::ffi::c_void,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    let name: Vec<u16> = std::ffi::OsStr::new(pipe)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Waiting first, because the listener serves one caller at a time and two
    // links clicked together would otherwise have the second refused outright
    // rather than queued behind the first.
    // Safe: the name is null-terminated and alive for the call.
    unsafe { WaitNamedPipeW(name.as_ptr(), HOW_LONG_TO_WAIT) };

    // Safe: as above. A failure comes back as the invalid handle and is
    // checked rather than used.
    let pipe = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_WRITE,
            0,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            0,
        )
    };
    if pipe == INVALID_HANDLE_VALUE {
        return Err(Error::Other(
            "The copy of Wixen Mail already running did not answer".to_string(),
        ));
    }

    let message = crate::application::handover::encode(argument);
    let mut written: u32 = 0;
    // Safe: the buffer outlives the call and the count matches its length.
    let wrote = unsafe {
        WriteFile(
            pipe,
            message.as_ptr(),
            message.len() as u32,
            &raw mut written,
            std::ptr::null_mut(),
        )
    };
    // Safe: the handle came from `CreateFileW` above and is closed once.
    unsafe { CloseHandle(pipe) };

    if wrote == 0 || written as usize != message.len() {
        return Err(Error::Other(
            "The copy of Wixen Mail already running did not take the whole message".to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn hand_over(_argument: &str) -> Result<()> {
    Err(Error::Other(
        "Handing over to a running copy is only written for Windows".to_string(),
    ))
}

/// Listen for other copies, calling `taken` for each argument handed over.
///
/// Runs on a thread of its own and never returns. It is not joined: the thread
/// spends its life blocked waiting for a caller, and there is nothing to tidy
/// up that ending the process does not tidy. A listener that cannot be started
/// is reported and not fatal, because a copy that cannot listen still works and
/// the cost is a second window rather than anything lost.
#[cfg(target_os = "windows")]
pub fn listen(taken: impl Fn(String) + Send + 'static) -> Result<()> {
    listen_on(&pipe_name(), taken)
}

/// The same, on a named pipe of the caller's choosing. See [`hand_over_on`].
#[cfg(target_os = "windows")]
fn listen_on(pipe: &str, taken: impl Fn(String) + Send + 'static) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const PIPE_ACCESS_INBOUND: u32 = 0x0000_0001;
    const PIPE_TYPE_BYTE: u32 = 0x0000_0000;
    const PIPE_WAIT: u32 = 0x0000_0000;
    const PIPE_REJECT_REMOTE_CLIENTS: u32 = 0x0000_0008;
    const INVALID_HANDLE_VALUE: isize = -1;
    /// One at a time. A person clicks one link; a queue of more than a few
    /// would mean something other than a person.
    const HOW_MANY_AT_ONCE: u32 = 4;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateNamedPipeW(
            name: *const u16,
            open_mode: u32,
            pipe_mode: u32,
            instances: u32,
            out_buffer: u32,
            in_buffer: u32,
            timeout: u32,
            security: *mut core::ffi::c_void,
        ) -> isize;
        fn ConnectNamedPipe(pipe: isize, overlapped: *mut core::ffi::c_void) -> i32;
        fn ReadFile(
            file: isize,
            buffer: *mut u8,
            to_read: u32,
            read: *mut u32,
            overlapped: *mut core::ffi::c_void,
        ) -> i32;
        fn DisconnectNamedPipe(pipe: isize) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    let name: Vec<u16> = std::ffi::OsStr::new(pipe)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Made here rather than on the thread, so a name already taken is reported
    // to the caller instead of disappearing into a thread nobody is watching.
    // Taken means another copy is listening, which is worth knowing.
    //
    // Safe: the name is null-terminated and alive for the call.
    let first = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            HOW_MANY_AT_ONCE,
            0,
            crate::application::handover::MOST_AN_ARGUMENT_MAY_HOLD as u32,
            0,
            std::ptr::null_mut(),
        )
    };
    if first == INVALID_HANDLE_VALUE {
        return Err(Error::Other(
            "Wixen Mail could not listen for other copies of itself".to_string(),
        ));
    }

    std::thread::spawn(move || {
        let mut pipe = first;
        loop {
            // Safe: the handle is one this loop owns and has not closed.
            unsafe { ConnectNamedPipe(pipe, std::ptr::null_mut()) };

            let mut buffer = vec![0u8; crate::application::handover::MOST_AN_ARGUMENT_MAY_HOLD];
            let mut read: u32 = 0;
            // Safe: the buffer outlives the call and the count is its length.
            let ok = unsafe {
                ReadFile(
                    pipe,
                    buffer.as_mut_ptr(),
                    buffer.len() as u32,
                    &raw mut read,
                    std::ptr::null_mut(),
                )
            };
            if ok != 0 {
                match crate::application::handover::decode(&buffer[..read as usize]) {
                    Ok(argument) => taken(argument),
                    // Said to the log and nowhere else. Somebody reading their
                    // mail should not be shown a message because another
                    // program on the machine poked a pipe.
                    Err(why) => tracing::warn!("A handover was refused: {why}"),
                }
            }

            // Safe: as above, and the handle is still open at this point.
            unsafe { DisconnectNamedPipe(pipe) };

            // A fresh instance for the next caller. The old handle is closed
            // rather than reused, because a byte pipe that has carried one
            // message is simpler to replace than to reset.
            //
            // Safe: the handle is this loop's and is closed exactly once.
            unsafe { CloseHandle(pipe) };
            let next = unsafe {
                CreateNamedPipeW(
                    name.as_ptr(),
                    PIPE_ACCESS_INBOUND,
                    PIPE_TYPE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    HOW_MANY_AT_ONCE,
                    0,
                    crate::application::handover::MOST_AN_ARGUMENT_MAY_HOLD as u32,
                    0,
                    std::ptr::null_mut(),
                )
            };
            if next == INVALID_HANDLE_VALUE {
                // Nothing left to listen on. Said once, then the thread ends:
                // a loop that kept trying would spin for the life of the
                // program writing the same line.
                tracing::warn!(
                    "Wixen Mail stopped listening for other copies of itself, so a link \
                     clicked elsewhere will open a second window"
                );
                return;
            }
            pipe = next;
        }
    });
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn listen(_taken: impl Fn(String) + Send + 'static) -> Result<()> {
    Err(Error::Other(
        "Listening for other copies is only written for Windows".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_pipe_name_is_one_windows_will_accept() {
        // A pipe name has to start this way or `CreateNamedPipeW` refuses it,
        // and the refusal at run time is a listener that never starts on a
        // machine nobody is testing on.
        let name = pipe_name();

        assert!(name.starts_with(r"\\.\pipe\"), "{name}");
        assert!(
            !name[r"\\.\pipe\".len()..].contains('\\'),
            "the part after the prefix may not hold a backslash: {name}"
        );
    }

    #[test]
    fn test_the_name_carries_the_session_so_two_people_do_not_share_one() {
        // A pipe name is machine-wide, unlike the mutex beside it, which lives
        // in the logon session. Without this, the second person signed in
        // would fail to listen and hand their links to the first person's copy.
        let name = pipe_name();

        assert!(
            name.ends_with(&format!("-{}", this_session())),
            "the session is not in the name: {name}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_copy_really_takes_what_another_hands_it() {
        // The whole point, end to end, through a real pipe. Everything else
        // here is a string check; this is the only test that proves Windows
        // carries the message at all.
        use std::sync::mpsc;

        // A pipe of this test's own, never the one the running program
        // listens on. With the real name, running the tests on a machine where
        // somebody had Wixen Mail open handed a link to their program and
        // opened a composer in front of them.
        let ours = format!(r"\\.\pipe\wixen-mail-handover-test-{}", std::process::id());

        let (heard, arrived) = mpsc::channel();
        listen_on(&ours, move |argument| {
            let _ = heard.send(argument);
        })
        .expect("a listener");

        hand_over_on(&ours, "mailto:someone@example.com?subject=Hello").expect("the handover");

        let got = arrived
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the listener heard nothing within five seconds");
        assert_eq!(got, "mailto:someone@example.com?subject=Hello");
    }
}

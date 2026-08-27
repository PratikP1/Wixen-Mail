//! The COM objects Windows Search loads, and nothing else.
//!
//! None of this can be unit tested. A COM vtable is only exercised by something
//! that knows how to call one, and the only thing that will ever call these is
//! the Windows indexer on a machine where the handler has been registered. So
//! everything that makes a decision lives in [`crate::url`], [`crate::record`],
//! [`crate::chunks`], [`crate::store`] and [`crate::registration`], where it is
//! tested, and the objects here only translate between those and Windows.
//!
//! Two rules hold everywhere in this module, because both are much harder to
//! recover from here than in ordinary code:
//!
//! - **Nothing may panic.** These functions are called across a boundary from a
//!   Microsoft process. An unwind out of one of them ends that process. There
//!   is no `unwrap`, no `expect`, no slicing by index and no arithmetic that can
//!   overflow. A lock that has been poisoned is an error value, not a panic.
//! - **Nothing is written anywhere.** No log, no file, no registry key except
//!   during self-registration. A subject line or an address must never leave
//!   this DLL by any route but the one the indexer asked for.

pub mod accessor;
pub mod exports;
pub mod factory;
pub mod filter;
pub mod protocol;
pub mod values;

use crate::store::{Store, StoreError, cache_path_in, cache_path_under_local_data};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY_LOCAL_MACHINE, RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ, RegGetValueW,
};
use windows::core::{HSTRING, PCWSTR};

/// How many objects this DLL has handed out and not had back.
static LIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);

/// How many times something has asked this DLL to stay loaded.
static SERVER_LOCKS: AtomicUsize = AtomicUsize::new(0);

/// Held by every object this DLL creates, for as long as it lives.
///
/// COM asks a server whether it may be unloaded, and answering yes while an
/// object is still alive unloads the code out from under a live pointer. A
/// value that counts itself in and out is the version of that bookkeeping
/// which cannot be forgotten at a return.
#[derive(Debug)]
pub struct Alive;

impl Alive {
    pub fn new() -> Self {
        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self
    }
}

impl Default for Alive {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Alive {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Record that something wants this DLL kept loaded, or no longer does.
pub fn hold_server(wanted: bool) {
    match wanted {
        true => SERVER_LOCKS.fetch_add(1, Ordering::AcqRel),
        // Saturating, because a caller that unlocks more often than it locked
        // would otherwise wrap the count to something enormous and pin this
        // DLL in the indexer's process until the machine restarts.
        false => SERVER_LOCKS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |held| {
                Some(held.saturating_sub(1))
            })
            .unwrap_or(0),
    };
}

/// Whether this DLL is finished with and may be unloaded.
pub fn nothing_is_in_use() -> bool {
    LIVE_OBJECTS.load(Ordering::Acquire) == 0 && SERVER_LOCKS.load(Ordering::Acquire) == 0
}

/// Where Windows records each signed-in account's profile folder.
const PROFILE_LIST: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList";

/// The value under it holding the folder itself.
const PROFILE_FOLDER: &str = "ProfileImagePath";

/// The most a profile path can be, in characters.
///
/// Generous rather than exact. A path is capped well below this, and the only
/// cost of the extra room is a buffer on the stack.
const LONGEST_PATH: usize = 1024;

/// Open the cache belonging to whoever the URL named.
///
/// The indexer runs outside any signed-in session, so it cannot be asked where
/// "my" application data is. A URL carrying a user identifier is looked up in
/// the profile list; a URL without one falls back to the folder this process
/// itself would use, which is the account the host is running as rather than
/// the person whose mail it is. That fallback is very unlikely to find
/// anything, and it is deliberately a miss rather than a guess at which of
/// several accounts on the machine was meant.
pub fn open_store_for(user: Option<&str>) -> Result<Store, StoreError> {
    let path = match user.and_then(without_braces).and_then(profile_folder) {
        Some(profile) => cache_path_in(&profile),
        None => match std::env::var_os("LOCALAPPDATA") {
            Some(local) => cache_path_under_local_data(&PathBuf::from(local)),
            None => return Err(StoreError::CannotOpen(0)),
        },
    };

    Store::open(&path)
}

/// The identifier inside the braces a URL wraps it in.
fn without_braces(user: &str) -> Option<&str> {
    user.strip_prefix('{')
        .and_then(|user| user.strip_suffix('}'))
}

/// The profile folder Windows records for one account.
fn profile_folder(sid: &str) -> Option<PathBuf> {
    let key = HSTRING::from(format!(r"{PROFILE_LIST}\{sid}"));
    let value = HSTRING::from(PROFILE_FOLDER);
    let mut buffer = [0u16; LONGEST_PATH];
    // In bytes, which is what this call counts in, and it writes back how many
    // it used.
    let mut room = (buffer.len() * size_of::<u16>()) as u32;

    let result = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &key,
            &value,
            // Both, because Windows writes this one either way depending on
            // how the profile was made, and asking for only one kind makes
            // half the machines report the value as missing. Without
            // RRF_NOEXPAND the call expands an entry holding %SystemDrive%
            // rather than handing back the unexpanded text.
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut room),
        )
    };
    if result != ERROR_SUCCESS {
        return None;
    }

    // Back to characters, and without the terminator the call counts.
    let used = (room as usize) / size_of::<u16>();
    let text = String::from_utf16(&buffer[..used.saturating_sub(1).min(buffer.len())]).ok()?;
    match text.is_empty() {
        true => None,
        false => Some(PathBuf::from(text)),
    }
}

/// Turn a Windows error code into something that can be returned.
pub const fn is_success(result: WIN32_ERROR) -> bool {
    result.0 == ERROR_SUCCESS.0
}

/// Read a wide string the caller owns, without trusting it to be terminated.
///
/// Windows hands URLs in as pointers. A missing terminator would otherwise
/// read past the end of somebody else's allocation, inside a process this code
/// does not own, so this stops at `most` characters and reports a failure
/// rather than reading on.
///
/// # Safety
///
/// `text` must either be null or point at readable memory. This will read at
/// most `most` characters from it and stops at the first zero, so a string
/// shorter than `most` and properly terminated is always safe. A pointer into
/// freed or unmapped memory is not, and nothing here can detect that.
pub unsafe fn read_wide(text: PCWSTR, most: usize) -> Option<String> {
    if text.is_null() {
        return None;
    }

    let mut units = Vec::new();
    for step in 0..most {
        let unit = unsafe { *text.0.add(step) };
        if unit == 0 {
            return String::from_utf16(&units).ok();
        }
        units.push(unit);
    }

    None
}

/// The most a URL handed to this handler may be, in characters.
///
/// Longer than any URL this handler writes: an account, a folder path and a
/// number, each escaped. Anything past it is not one of ours.
pub const LONGEST_URL: usize = 8192;

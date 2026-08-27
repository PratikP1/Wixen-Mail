//! Putting one value into the shape and the memory Windows expects.
//!
//! Every allocation here comes from the COM task allocator, because the
//! indexer frees what it is given with `PropVariantClear`, which uses that
//! allocator and no other. Handing back memory from Rust's allocator would
//! look correct, index correctly, and corrupt the heap of a Microsoft process
//! some time later.

use crate::record::{Value, windows_ticks};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::Com::CoTaskMemAlloc;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Variant::{VT_FILETIME, VT_LPWSTR, VT_UI8, VT_VARIANT, VT_VECTOR};
use windows::core::PWSTR;

/// How many values a child link is made of: the URL, then the time.
const CHILD_PARTS: usize = 2;

/// Put a value where the indexer can take it and free it.
///
/// `None` when memory ran out or when the value cannot be represented, which
/// the caller reports rather than handing over something half built.
///
/// # Safety
///
/// The returned pointer is owned by the caller, which must free it the COM
/// way. Every pointer inside it was allocated by the same allocator, so a
/// single `PropVariantClear` releases the lot.
pub unsafe fn allocate(value: &Value) -> Option<*mut PROPVARIANT> {
    let slot = unsafe { empty_propvariant()? };

    let filled = unsafe {
        match value {
            Value::Text(text) => fill_text(slot, text),
            Value::Number(number) => fill_number(slot, *number),
            Value::Moment(seconds) => fill_moment(slot, *seconds),
            Value::UrlAndMoment { url, modified } => fill_child(slot, url, *modified),
        }
    };

    match filled {
        true => Some(slot),
        false => {
            // Nothing inside it was filled in, so the outer allocation is all
            // there is to give back.
            unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(slot.cast())) };
            None
        }
    }
}

/// One zeroed value, which is a valid empty one.
unsafe fn empty_propvariant() -> Option<*mut PROPVARIANT> {
    let slot = unsafe { CoTaskMemAlloc(size_of::<PROPVARIANT>()) }.cast::<PROPVARIANT>();
    if slot.is_null() {
        return None;
    }
    // Zeroed is VT_EMPTY, so even a half finished value is one the indexer can
    // safely clear.
    unsafe { slot.write(PROPVARIANT::default()) };
    Some(slot)
}

unsafe fn fill_text(slot: *mut PROPVARIANT, text: &str) -> bool {
    let Some(wide) = (unsafe { task_memory_wide(text) }) else {
        return false;
    };
    unsafe {
        let inner = &mut (*slot).Anonymous.Anonymous;
        inner.vt = VT_LPWSTR;
        inner.Anonymous.pwszVal = wide;
    }
    true
}

unsafe fn fill_number(slot: *mut PROPVARIANT, number: u64) -> bool {
    unsafe {
        let inner = &mut (*slot).Anonymous.Anonymous;
        inner.vt = VT_UI8;
        inner.Anonymous.uhVal = number;
    }
    true
}

unsafe fn fill_moment(slot: *mut PROPVARIANT, seconds_since_1970: i64) -> bool {
    let Some(ticks) = windows_ticks(seconds_since_1970) else {
        return false;
    };
    unsafe {
        let inner = &mut (*slot).Anonymous.Anonymous;
        inner.vt = VT_FILETIME;
        inner.Anonymous.filetime = as_filetime(ticks);
    }
    true
}

/// A child link, which Windows takes as a list of two values.
unsafe fn fill_child(slot: *mut PROPVARIANT, url: &str, modified: i64) -> bool {
    let Some(ticks) = windows_ticks(modified) else {
        return false;
    };

    let parts =
        unsafe { CoTaskMemAlloc(size_of::<PROPVARIANT>() * CHILD_PARTS) }.cast::<PROPVARIANT>();
    if parts.is_null() {
        return false;
    }
    for step in 0..CHILD_PARTS {
        unsafe { parts.add(step).write(PROPVARIANT::default()) };
    }

    if !unsafe { fill_text(parts, url) } {
        unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(parts.cast())) };
        return false;
    }
    unsafe {
        let time = &mut (*parts.add(1)).Anonymous.Anonymous;
        time.vt = VT_FILETIME;
        time.Anonymous.filetime = as_filetime(ticks);

        let inner = &mut (*slot).Anonymous.Anonymous;
        inner.vt = windows::Win32::System::Variant::VARENUM(VT_VECTOR.0 | VT_VARIANT.0);
        inner.Anonymous.capropvar.cElems = CHILD_PARTS as u32;
        inner.Anonymous.capropvar.pElems = parts;
    }
    true
}

/// Split Windows' time count into the two halves the structure keeps it in.
const fn as_filetime(ticks: u64) -> FILETIME {
    FILETIME {
        dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
        dwHighDateTime: (ticks >> 32) as u32,
    }
}

/// Copy text into memory the COM allocator owns, with a terminator.
unsafe fn task_memory_wide(text: &str) -> Option<PWSTR> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);

    let bytes = units.len().checked_mul(size_of::<u16>())?;
    let memory = unsafe { CoTaskMemAlloc(bytes) }.cast::<u16>();
    if memory.is_null() {
        return None;
    }

    unsafe { std::ptr::copy_nonoverlapping(units.as_ptr(), memory, units.len()) };
    Some(PWSTR(memory))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_time_is_split_into_its_two_halves_without_losing_a_bit() {
        // The structure keeps one number in two thirty-two bit fields, and
        // putting the halves the wrong way round gives a date roughly four
        // hundred years out with no sign that anything went wrong.
        let split = as_filetime(133_444_736_000_000_000);
        let rejoined = (u64::from(split.dwHighDateTime) << 32) | u64::from(split.dwLowDateTime);

        assert_eq!(rejoined, 133_444_736_000_000_000);
    }

    #[test]
    fn test_a_time_at_the_very_top_of_the_range_still_splits_and_rejoins() {
        // The obvious way to write this shifts a thirty-two bit value and
        // loses the top half.
        let split = as_filetime(u64::MAX);

        assert_eq!(split.dwLowDateTime, u32::MAX);
        assert_eq!(split.dwHighDateTime, u32::MAX);
    }
}

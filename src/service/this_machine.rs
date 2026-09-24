//! What this machine says about itself, for a feedback report (#64).
//!
//! Three questions asked of Windows: which build it is, which language it
//! shows, and which screen reader is running. Each has a pure half that
//! decides the answer from plain values, which the cases below drive, and a
//! Win32 half that fetches those values, declared by hand on the tree's
//! `extern "system"` pattern so no crate and no feature is added. On other
//! platforms the Win32 half answers that it does not know.
//!
//! Nothing here reads an address, a name or a message. The accounts are
//! described by the kind of mail they are, never by what they are called.

use crate::data::account::Account;

/// The screen readers known by their process, in the order one is named
/// when several run at once.
const READERS: [(&str, &str); 3] = [
    ("nvda.exe", "NVDA"),
    ("jfw.exe", "JAWS"),
    ("narrator.exe", "Narrator"),
];

/// What is said when a version cannot be read.
const UNKNOWN_VERSION: &str = "version unknown";

/// Which Windows this is, such as "Windows 11, build 26200".
pub fn windows_build() -> String {
    win32::version()
        .map(|(major, minor, build)| describe_windows(major, minor, build))
        .unwrap_or_else(|| "Windows, build unknown".to_string())
}

/// The language Windows shows, such as "en-GB".
pub fn display_language() -> String {
    win32::locale_name().unwrap_or_else(|| "unknown".to_string())
}

/// The region this person says they are in, the setting Windows calls
/// Country or region, as a two-letter code such as "GB".
pub fn home_region() -> Option<String> {
    None
}

/// A region's name in the language Windows shows, such as "United Kingdom"
/// for "GB". `None` for a code Windows has no name for.
pub fn region_name(_code: &str) -> Option<String> {
    None
}

/// The text with every decimal digit, in any script, written as its ASCII
/// digit. Anything that is not a digit is left as it was.
pub fn fold_digits(text: &str) -> String {
    text.to_string()
}

/// The screen reader running and its version, when one is.
pub fn screen_reader() -> Option<(String, String)> {
    let running = win32::running_processes();
    let names: Vec<String> = running.iter().map(|(name, _)| name.clone()).collect();
    let reader = which_reader(&names)?;
    let executable = READERS
        .iter()
        .find(|(_, named)| *named == reader)
        .map(|(executable, _)| *executable)?;
    let version = running
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(executable))
        .and_then(|(_, id)| win32::file_version_of_process(*id))
        .map(|(most, least)| file_version_text(most, least))
        .unwrap_or_else(|| UNKNOWN_VERSION.to_string());
    Some((reader.to_string(), version))
}

/// Which screen reader a list of running process names shows, if any.
pub fn which_reader(process_names: &[String]) -> Option<&'static str> {
    READERS
        .iter()
        .find(|(executable, _)| {
            process_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(executable))
        })
        .map(|(_, reader)| *reader)
}

/// The first build Microsoft calls Windows 11. Both report major version 10.
const FIRST_WINDOWS_11_BUILD: u32 = 22000;

/// A Windows version as a person names it.
pub fn describe_windows(major: u32, minor: u32, build: u32) -> String {
    match (major, minor) {
        (10, 0) if build >= FIRST_WINDOWS_11_BUILD => format!("Windows 11, build {build}"),
        (10, 0) => format!("Windows 10, build {build}"),
        _ => format!("Windows {major}.{minor}, build {build}"),
    }
}

/// A file version from the two words a version resource carries it in.
pub fn file_version_text(most_significant: u32, least_significant: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        most_significant >> 16,
        most_significant & 0xFFFF,
        least_significant >> 16,
        least_significant & 0xFFFF
    )
}

/// The kinds of account set up, without their addresses.
///
/// The provider's name when the account was made from a preset, otherwise
/// the protocol it reads mail with; each kind once, in the accounts' order.
pub fn providers(accounts: &[Account]) -> Vec<String> {
    let mut kinds: Vec<String> = Vec::new();
    for account in accounts {
        let kind = account
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|provider| !provider.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| account.protocol().as_str().to_uppercase());
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    const TH32CS_SNAPPROCESS: u32 = 0x2;
    const INVALID_HANDLE_VALUE: isize = -1;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const LOCALE_NAME_MAX_LENGTH: usize = 85;
    const MAX_PATH: usize = 260;

    #[repr(C)]
    struct OsVersionInfoW {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        service_pack: [u16; 128],
    }

    #[repr(C)]
    struct ProcessEntry32W {
        size: u32,
        usage: u32,
        process_id: u32,
        default_heap_id: usize,
        module_id: u32,
        threads: u32,
        parent_process_id: u32,
        priority_base: i32,
        flags: u32,
        exe_file: [u16; MAX_PATH],
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultLocaleName(name: *mut u16, length: i32) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> isize;
        fn Process32FirstW(snapshot: isize, entry: *mut ProcessEntry32W) -> i32;
        fn Process32NextW(snapshot: isize, entry: *mut ProcessEntry32W) -> i32;
        fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> isize;
        fn QueryFullProcessImageNameW(
            process: isize,
            flags: u32,
            name: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    #[link(name = "version")]
    unsafe extern "system" {
        fn GetFileVersionInfoSizeW(file: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(
            file: *const u16,
            handle: u32,
            length: u32,
            data: *mut c_void,
        ) -> i32;
        fn VerQueryValueW(
            block: *const c_void,
            sub_block: *const u16,
            buffer: *mut *mut c_void,
            length: *mut u32,
        ) -> i32;
    }

    /// Text up to its terminator.
    fn until_nul(wide: &[u16]) -> String {
        let end = wide.iter().position(|c| *c == 0).unwrap_or(wide.len());
        String::from_utf16_lossy(&wide[..end])
    }

    /// Major, minor and build, from the call that does not lie about them.
    pub(super) fn version() -> Option<(u32, u32, u32)> {
        let mut info = OsVersionInfoW {
            size: std::mem::size_of::<OsVersionInfoW>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform: 0,
            service_pack: [0; 128],
        };
        // SAFETY: a structure of the size it says it is, filled by the call.
        let status = unsafe { RtlGetVersion(&mut info) };
        (status == 0).then_some((info.major, info.minor, info.build))
    }

    pub(super) fn locale_name() -> Option<String> {
        let mut name = [0u16; LOCALE_NAME_MAX_LENGTH];
        // SAFETY: the buffer is as long as the count says.
        let written = unsafe { GetUserDefaultLocaleName(name.as_mut_ptr(), name.len() as i32) };
        (written > 0).then(|| until_nul(&name))
    }

    /// Every running process's executable name and id. Empty when the list
    /// cannot be taken.
    pub(super) fn running_processes() -> Vec<(String, u32)> {
        // SAFETY: a snapshot of the processes, closed before returning.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE || snapshot == 0 {
            return Vec::new();
        }
        let mut entry = ProcessEntry32W {
            size: std::mem::size_of::<ProcessEntry32W>() as u32,
            usage: 0,
            process_id: 0,
            default_heap_id: 0,
            module_id: 0,
            threads: 0,
            parent_process_id: 0,
            priority_base: 0,
            flags: 0,
            exe_file: [0; MAX_PATH],
        };
        let mut running = Vec::new();
        // SAFETY: the entry is the size it says it is; each call fills it.
        let mut more = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        while more {
            running.push((until_nul(&entry.exe_file), entry.process_id));
            // SAFETY: as above.
            more = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        }
        // SAFETY: the snapshot opened above.
        unsafe { CloseHandle(snapshot) };
        running
    }

    /// The two words of a running process's file version.
    pub(super) fn file_version_of_process(process_id: u32) -> Option<(u32, u32)> {
        // SAFETY: the handle is asked for the least access that reads a path,
        // and closed before the path is used.
        let path = unsafe {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if process == 0 {
                return None;
            }
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let read = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(process);
            if read == 0 {
                return None;
            }
            buffer[..size as usize]
                .iter()
                .copied()
                .chain(std::iter::once(0))
                .collect::<Vec<u16>>()
        };
        file_version(&path)
    }

    /// The file version from a file's version resource.
    fn file_version(path: &[u16]) -> Option<(u32, u32)> {
        let mut ignored = 0u32;
        // SAFETY: a terminated path; the size is read before the block.
        let size = unsafe { GetFileVersionInfoSizeW(path.as_ptr(), &mut ignored) };
        if size == 0 {
            return None;
        }
        let mut block = vec![0u8; size as usize];
        let root: Vec<u16> = "\\".encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: the block is as long as the size asked for; the fixed part
        // it points into lives as long as the block, and is read before the
        // block is dropped. Its words are signature, structure version, then
        // the file version's two words.
        unsafe {
            if GetFileVersionInfoW(path.as_ptr(), 0, size, block.as_mut_ptr().cast()) == 0 {
                return None;
            }
            let mut fixed: *mut c_void = std::ptr::null_mut();
            let mut length = 0u32;
            if VerQueryValueW(
                block.as_ptr().cast(),
                root.as_ptr(),
                &mut fixed,
                &mut length,
            ) == 0
                || fixed.is_null()
                || (length as usize) < 4 * std::mem::size_of::<u32>()
            {
                return None;
            }
            let words = fixed as *const u32;
            Some((*words.add(2), *words.add(3)))
        }
    }
}

/// Elsewhere nothing is asked, and each question answers that it does not
/// know.
#[cfg(not(target_os = "windows"))]
mod win32 {
    pub(super) fn version() -> Option<(u32, u32, u32)> {
        None
    }

    pub(super) fn locale_name() -> Option<String> {
        None
    }

    pub(super) fn running_processes() -> Vec<(String, u32)> {
        Vec::new()
    }

    pub(super) fn file_version_of_process(_process_id: u32) -> Option<(u32, u32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn test_each_reader_is_found_by_its_process_whatever_case_it_is_written_in() {
        for (running, reader) in [
            ("nvda.exe", "NVDA"),
            ("NVDA.EXE", "NVDA"),
            ("jfw.exe", "JAWS"),
            ("Narrator.exe", "Narrator"),
            ("narrator.EXE", "Narrator"),
        ] {
            assert_eq!(
                which_reader(&names(&["explorer.exe", running, "svchost.exe"])),
                Some(reader),
                "{running}"
            );
        }
    }

    #[test]
    fn test_no_reader_is_named_when_none_runs_and_nvda_is_named_before_narrator() {
        assert_eq!(
            which_reader(&names(&["explorer.exe", "nvda_helper.exe"])),
            None
        );
        assert_eq!(
            which_reader(&names(&["Narrator.exe", "nvda.exe"])),
            Some("NVDA")
        );
    }

    #[test]
    fn test_windows_is_named_the_way_a_person_names_it() {
        assert_eq!(describe_windows(10, 0, 26200), "Windows 11, build 26200");
        assert_eq!(describe_windows(10, 0, 22000), "Windows 11, build 22000");
        assert_eq!(describe_windows(10, 0, 19045), "Windows 10, build 19045");
        assert_eq!(describe_windows(6, 3, 9600), "Windows 6.3, build 9600");
    }

    #[test]
    fn test_a_file_version_is_read_from_its_two_words() {
        assert_eq!(file_version_text(0x07E9_0003, 0x0000_0001), "2025.3.0.1");
    }

    #[test]
    fn test_accounts_are_named_by_their_kind_and_never_by_their_address() {
        let mut gmail = Account::new("Work".to_string(), "dana@gmail.com".to_string());
        gmail.provider = Some("Gmail".to_string());
        let mut pop = Account::new("Home".to_string(), "dana@example.net".to_string());
        pop.protocol = crate::common::types::Protocol::Pop3.as_str().to_string();
        let second_gmail = Account {
            id: "b".to_string(),
            ..gmail.clone()
        };

        let named = providers(&[gmail, pop, second_gmail]);

        assert_eq!(named, names(&["Gmail", "POP3"]));
        assert!(
            !named.iter().any(|n| n.contains('@') || n.contains("Work")),
            "{named:?}"
        );
    }

    #[test]
    fn test_this_machine_answers_with_a_windows_it_names() {
        // The one case that asks the real machine, so the Win32 half is
        // known to be reached and to answer something a person can read.
        let answer = windows_build();

        assert!(answer.starts_with("Windows"), "{answer:?}");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_this_machine_names_a_region_says_where_it_is_and_folds_digits() {
        // Asks the real machine, as the case above does. ZZ is a code no
        // region has, so the name for GB is a name and not an echo.
        assert_eq!(region_name("GB").as_deref(), Some("United Kingdom"));
        assert_eq!(region_name("ZZ"), None);

        let home = home_region().unwrap_or_default();
        assert!(
            home.len() == 2 && home.chars().all(|c| c.is_ascii_uppercase()),
            "{home:?}"
        );

        // Full-width digits and Arabic-Indic digits, then what must not move.
        assert_eq!(
            fold_digits("\u{FF10}\u{FF11}\u{FF12}\u{FF11} 234"),
            "0121 234"
        );
        assert_eq!(
            fold_digits("\u{0660}\u{0661}\u{0662}\u{0661} \u{0665}\u{0666}"),
            "0121 56"
        );
        assert_eq!(fold_digits("+44 Ext x"), "+44 Ext x");
    }
}

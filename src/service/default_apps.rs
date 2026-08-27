//! Whether Windows opens mail, calendars and contact cards with this program.
//!
//! **This module cannot make Wixen Mail the default for anything, and no
//! version of it ever will.** Windows 8 closed that door. The COM call that
//! used to do it, `IApplicationAssociationRegistration::SetAppAsDefault`, is
//! marked "Not intended for use in Windows 8" in Microsoft's own reference,
//! and it answers `E_ACCESSDENIED` unless the calling program's publisher
//! already matches the current default's, which for a program trying to
//! become the default is never. Its companions `SetAppAsDefaultAll` and
//! `QueryAppIsDefault` are deprecated the same way. There is no replacement
//! API, no manifest entry and no installer switch. Every program that appears
//! to set itself as the default on Windows 10 or 11 is either forging the
//! `UserChoice` hash, which Windows treats as tampering and resets, or it is
//! doing exactly what this module does.
//!
//! What is supported is a two-part flow, and both parts are here:
//!
//! 1. Say what this program can open, by writing the capability keys Windows
//!    reads when it builds the list of candidates. [`capability_registry_entries`]
//!    returns those as data, and
//!    [`crate::service::default_apps_registration`] writes and removes them.
//!    They are split that way so what to write can be read and tested without
//!    a registry anywhere near it. Until that second module existed nothing
//!    ever wrote them, and Wixen Mail did not appear in the Windows list at
//!    all, so nobody could choose it even if they wanted to.
//! 2. Send the person to the Windows Settings page where they choose.
//!    [`open_windows_default_apps_page`] does that and nothing more.
//!
//! [`is_default`] answers the third question, which is what to put on the
//! screen next to that button. It reads the registry rather than calling COM,
//! because the deprecated COM answer would need the same keys underneath it
//! and a `RegGetValueW` is a tenth of the code with none of the apartment
//! rules.
//!
//! Three of the six kinds a person can hold in this program, tasks, reminders
//! and notes, have no default slot in Windows at all. There is no file type
//! and no protocol the shell keeps an owner for. That is not a failure to
//! read something, and answering "we are not the default" would be a
//! different and wrong claim, so [`Status::NoWindowsSlot`] says the true
//! thing instead and the screen can say it in words.

use crate::common::{Error, Result};

/// One thing a person opens, that Windows keeps an owner for.
///
/// The text in each is what Windows itself uses: a scheme without its colon,
/// an extension with its dot. Both go into a registry path verbatim, so they
/// are held exactly as the shell spells them rather than being reassembled at
/// each use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Association {
    /// A URL scheme, written without the colon: `mailto`, `webcal`.
    Protocol(&'static str),
    /// A file extension, written with the dot: `.ics`, `.vcf`.
    FileType(&'static str),
}

impl Association {
    /// Where Windows records the choice this person made.
    ///
    /// Two different places, which is not tidiness on Microsoft's part but the
    /// order these features arrived in. Both hold a `ProgId` value naming the
    /// winner, and per-user choice beats anything registered for the machine,
    /// so this is the key that decides.
    pub fn user_choice_key(self) -> String {
        match self {
            Association::Protocol(scheme) => format!(
                r"HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\{scheme}\UserChoice"
            ),
            Association::FileType(extension) => format!(
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\{extension}\UserChoice"
            ),
        }
    }

    /// What this slot is called when somebody reads it out.
    ///
    /// Reached through [`Status::Partial`], which is a sentence about some
    /// slots and not others, so these have to survive being spoken in a list.
    /// Built from the same text rather than written out per slot, so a slot
    /// added above cannot arrive without words to describe it.
    pub fn label(self) -> String {
        match self {
            Association::Protocol(scheme) => format!("{scheme}: links"),
            Association::FileType(extension) => format!("{extension} files"),
        }
    }

    /// What a person is told this slot holds.
    ///
    /// This is the nameless value of the registered identifier's key, which
    /// is the text Explorer puts in its Type column and the text an Open With
    /// list reads out. Left empty, Windows shows the extension in capitals
    /// and a screen reader says "ICS FILE", so these are written out rather
    /// than derived.
    ///
    /// The last arm catches a slot added to [`DefaultKind::associations`]
    /// without words written for it here. What it gives is worse than a
    /// proper description and much better than nothing.
    pub fn describes(self) -> String {
        match self {
            Association::Protocol("mailto") => "Email link".to_string(),
            Association::Protocol("webcal") => "Calendar subscription link".to_string(),
            Association::FileType(".ics") => "Calendar file".to_string(),
            Association::FileType(".vcf") => "Contact card".to_string(),
            undescribed => undescribed.label(),
        }
    }

    /// The identifier this program registers for the slot.
    ///
    /// One per slot rather than one shared between them, which Microsoft asks
    /// for and which also earns its keep here: each carries its own open
    /// command, so a `webcal` subscription can be handed over as a URL while
    /// an `.ics` file arrives as a path.
    ///
    /// Never shown to anybody. It is an identifier the shell matches on, and
    /// the words a person reads come from the friendly name written beside it
    /// in [`capability_entries_for`].
    pub fn our_prog_id(self) -> String {
        match self {
            Association::Protocol(scheme) => format!("WixenMail.Url.{scheme}"),
            Association::FileType(extension) => {
                format!("WixenMail.AssocFile.{}", extension.trim_start_matches('.'))
            }
        }
    }
}

/// A kind of thing somebody might want this program to open.
///
/// All six modules are here, including the three Windows has nothing for, so
/// a settings screen can list what it holds rather than a filtered subset
/// whose gaps look like an oversight. What separates them is
/// [`Self::associations`] coming back empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultKind {
    Mail,
    Calendar,
    Contacts,
    Tasks,
    Reminders,
    Notes,
}

impl DefaultKind {
    /// Every kind, so a screen and a test cover the same set.
    pub const ALL: [DefaultKind; 6] = [
        DefaultKind::Mail,
        DefaultKind::Calendar,
        DefaultKind::Contacts,
        DefaultKind::Tasks,
        DefaultKind::Reminders,
        DefaultKind::Notes,
    ];

    /// The slots Windows keeps an owner for, for this kind.
    ///
    /// Empty for tasks, reminders and notes. Windows has no file type and no
    /// protocol for any of the three, so there is nothing to be the default
    /// for and nothing here to claim. An `.ics` file can carry a task, but the
    /// extension belongs to calendars and claiming it twice would mean one
    /// setting silently deciding the other.
    ///
    /// Calendar holds two because a subscription and a downloaded file arrive
    /// by different routes and Windows lets a person answer them separately.
    pub fn associations(self) -> &'static [Association] {
        match self {
            DefaultKind::Mail => &[Association::Protocol("mailto")],
            DefaultKind::Calendar => &[
                Association::FileType(".ics"),
                Association::Protocol("webcal"),
            ],
            DefaultKind::Contacts => &[Association::FileType(".vcf")],
            DefaultKind::Tasks | DefaultKind::Reminders | DefaultKind::Notes => &[],
        }
    }
}

/// The name Windows lists this program under.
///
/// One string for both the `ApplicationName` value and the name under
/// `RegisteredApplications`, because Microsoft requires them to match and two
/// literals is how they stop matching.
///
/// Public because [`crate::service::default_apps_registration`] writes and
/// removes these entries, and removal has to name the same value that writing
/// created. Two spellings would leave the pointer behind on an uninstall.
pub const APPLICATION_NAME: &str = "Wixen Mail";

/// What the Settings page says this program can do, in one sentence.
///
/// Required rather than decorative. Microsoft's rule is that a program with
/// no `ApplicationDescription` does not appear in the list of candidates at
/// all, and nothing anywhere reports that as a fault, so the whole
/// registration would be present and invisible.
const APPLICATION_DESCRIPTION: &str =
    "Mail, calendar, contacts, tasks, notes and reminders, built to be used with a screen reader.";

/// Where the capability keys go.
///
/// Under the current user rather than the machine, for two reasons that point
/// the same way. The installer runs with `PrivilegesRequired=lowest`, so it
/// cannot write to `HKEY_LOCAL_MACHINE` on an ordinary account. And the
/// association itself is per-user, so a machine-wide claim would be a claim
/// one person could make on behalf of everybody who shares the computer.
/// Verified against this machine rather than taken from a page. Every
/// application in `RegisteredApplications` here names a key of its own
/// choosing: Thunderbird uses `Software\Clients\Mail\Mozilla Thunderbird`,
/// Firefox uses `Software\Clients\StartMenuInternet\Firefox-...`, and
/// foobar2000 and QRead both use a key under their own vendor name, which is
/// the shape this follows.
pub const CAPABILITY_KEY: &str = r"HKCU\Software\Wixen\Wixen Mail\Capabilities";

/// The two keys above [`CAPABILITY_KEY`] that this program made to hold it.
///
/// Named so that removing the registration can take them away when they are
/// empty, rather than leaving a trail of keys behind on an uninstall. Removal
/// checks that each is empty first, so a sibling program that later keeps
/// something under `Software\Wixen` does not lose it.
pub const CAPABILITY_KEY_CONTAINERS: [&str; 2] =
    [r"HKCU\Software\Wixen\Wixen Mail", r"HKCU\Software\Wixen"];

/// Where Windows looks to find the key above.
///
/// This is the only entry Windows reads without being told where to look, so
/// it is the one that has to be right for any of the others to be seen.
///
/// Under the current user, and that is a real place rather than a fallback.
/// Read off this machine: `HKCU\Software\RegisteredApplications` holds
/// `ZoomPBX`, which is registered nowhere else, and a duplicate of Firefox's
/// entry. Both resolve, so a program with no administrator rights can put
/// itself in this list.
///
/// **Never delete this key.** It belongs to Windows and holds every other
/// program's entry. Removal takes out one named value.
pub const REGISTERED_APPLICATIONS_KEY: &str = r"HKCU\Software\RegisteredApplications";

/// Where the identifiers this program registers live.
///
/// **Never delete this key either.** It is the whole per-user class
/// registration. Removal takes out the four identifier keys below it.
pub const CLASSES_KEY: &str = r"HKCU\Software\Classes";

/// The nameless value every registry key has, spelled once.
const DEFAULT_VALUE: &str = "";

/// The Settings page where a person chooses their defaults.
///
/// The only supported way for this program to have any effect on what the
/// defaults are, and it works by asking somebody rather than by deciding.
pub const DEFAULT_APPS_SETTINGS_URI: &str = "ms-settings:defaultapps";

/// The same page, opened at this program rather than at the top of the list.
///
/// Microsoft added this in Windows 11 and it is the one supported way a
/// program can have any say in this at all. `registeredAppUser` rather than
/// `registeredAppMachine` because the registration is under the current user:
/// naming the wrong one lands on the general list instead.
///
/// It matters most to the people this program is for. The general page is a
/// list of every application on the machine, and finding one entry in it by
/// keyboard means arrowing through the lot. This opens on Wixen Mail's own
/// page, where the file types and the links it can open are the whole content.
///
/// The name is spelled with `%20` because it goes into a URI. It has to match
/// the name under `RegisteredApplications` exactly, so it is built from
/// [`APPLICATION_NAME`] rather than written out again.
pub fn default_apps_settings_uri_for_this_program() -> String {
    format!(
        "{DEFAULT_APPS_SETTINGS_URI}?registeredAppUser={}",
        APPLICATION_NAME.replace(' ', "%20")
    )
}

/// The value under a `UserChoice` key that names the winner.
const USER_CHOICE_VALUE: &str = "ProgId";

/// What to call the executable when its real path cannot be read.
const EXECUTABLE_FILE_NAME: &str = "wixen-mail.exe";

/// The registry entries that put this program in the list of candidates.
///
/// Data rather than an action, on purpose, so that what gets written can be
/// read and tested with no registry involved.
/// [`crate::service::default_apps_registration`] applies them, and reads this
/// same list to work out what to take away again, so writing and removing
/// cannot drift apart.
///
/// This form finds the executable itself, which is right for a running
/// program and wrong for anything that knows a folder it is not running from.
/// Such a caller should use [`capability_entries_for`] with the path in hand.
///
/// If the executable's own path cannot be read, the commands fall back to a
/// bare `wixen-mail.exe` and lean on the shell finding it. Said plainly
/// because that is weaker than a full path and can fail.
pub fn capability_registry_entries() -> Vec<(String, String, String)> {
    let executable = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| EXECUTABLE_FILE_NAME.to_string());
    capability_entries_for(&executable)
}

/// The same entries, for a program installed at `executable_path`.
///
/// Each is a key path, a value name, and the value. An empty value name is
/// the key's own nameless value, which is what `reg add /ve` writes.
///
/// None of this makes anything the default. It is the half of the supported
/// flow that says what this program can open; the other half is a person
/// choosing, in the page [`open_windows_default_apps_page`] opens.
///
/// The shape was checked against what is really on a Windows 11 machine
/// rather than taken from a page. Thunderbird's mail registration is a
/// `Capabilities` key holding `ApplicationName`, `ApplicationDescription` and
/// `ApplicationIcon`, with `FileAssociations` and `URLAssociations` below it,
/// an identifier per claimed type under `Software\Classes` carrying a
/// `DefaultIcon` and a `shell\open\command` of `"...exe" "%1"`, and a value in
/// `RegisteredApplications` naming the capability key. That is what is
/// produced here.
///
/// What is deliberately not here: `Software\Clients\Mail`, the key that
/// claims the system MAPI client. Wixen Mail does not implement MAPI, so
/// claiming that slot would advertise a service that would then fail for
/// every program that used it.
pub fn capability_entries_for(executable_path: &str) -> Vec<(String, String, String)> {
    let at =
        |key: &str, name: &str, value: &str| (key.to_string(), name.to_string(), value.to_string());

    let mut entries = vec![
        at(CAPABILITY_KEY, "ApplicationName", APPLICATION_NAME),
        at(
            CAPABILITY_KEY,
            "ApplicationDescription",
            APPLICATION_DESCRIPTION,
        ),
        at(
            REGISTERED_APPLICATIONS_KEY,
            APPLICATION_NAME,
            CAPABILITY_KEY
                .strip_prefix(r"HKCU\")
                .unwrap_or(CAPABILITY_KEY),
        ),
    ];

    for kind in DefaultKind::ALL {
        for &association in kind.associations() {
            let prog_id = association.our_prog_id();
            let class_key = format!(r"{CLASSES_KEY}\{prog_id}");

            let (subkey, claimed_as) = match association {
                Association::Protocol(scheme) => ("UrlAssociations", scheme),
                Association::FileType(extension) => ("FileAssociations", extension),
            };
            entries.push(at(
                &format!(r"{CAPABILITY_KEY}\{subkey}"),
                claimed_as,
                &prog_id,
            ));

            entries.push(at(&class_key, DEFAULT_VALUE, &association.describes()));
            if let Association::Protocol(_) = association {
                // An empty value whose presence is the whole message: it is
                // what tells the shell this identifier can be the target of a
                // scheme. Without it a mailto: link finds nothing.
                entries.push(at(&class_key, "URL Protocol", ""));
            }
            entries.push(at(
                &format!(r"{class_key}\DefaultIcon"),
                DEFAULT_VALUE,
                &format!("{executable_path},0"),
            ));
            // Quoted, because the default install path is under Program
            // Files and an unquoted path with a space in it is read by the
            // shell as a program called `C:\Program` with arguments.
            entries.push(at(
                &format!(r"{class_key}\shell\open\command"),
                DEFAULT_VALUE,
                &format!(r#""{executable_path}" "%1""#),
            ));
        }
    }
    entries
}

/// Open the Windows page where a person chooses their default programs.
///
/// This is the entire supported mechanism. It does not preselect anything, it
/// cannot, and it returns as soon as the page is asked for rather than
/// waiting to see what was chosen. Whatever is on the screen beside the
/// button that calls this should say so, so nobody presses it expecting the
/// change to have been made.
///
/// Not exercised by the test suite. Running it opens a window on whichever
/// machine the suite is running on, and this project's rule is that tests do
/// not drive the desktop. What is tested is the URI, which is the part that
/// can be wrong without anybody noticing.
pub fn open_windows_default_apps_page() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        // This program's own page first, and the general list only if that
        // does not open. The targeted form is a Windows 11 addition, so an
        // older Windows 10 has to land somewhere rather than on an error, and
        // the general page is where it landed before this existed.
        if open::that(default_apps_settings_uri_for_this_program()).is_ok() {
            return Ok(());
        }
        open::that(DEFAULT_APPS_SETTINGS_URI).map_err(|problem| {
            Error::Other(format!(
                "Windows would not open its default apps settings: {problem}"
            ))
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(Error::Other(
            "Choosing default programs this way is a Windows feature, and this is not Windows."
                .to_string(),
        ))
    }
}

/// What Windows currently opens this kind of thing with.
///
/// Reads the registry. It does not change anything, and it cannot: see this
/// module's own documentation for why no version of this can.
///
/// The reading is per slot and the answer is per kind, folded by
/// [`status_from`], which holds the whole decision and touches no registry.
pub fn is_default(kind: DefaultKind) -> Status {
    let slots: Vec<(Association, Held)> = kind
        .associations()
        .iter()
        .map(|&association| {
            let reading = read_registry_string(&association.user_choice_key(), USER_CHOICE_VALUE);
            (association, held_from(&reading, &association.our_prog_id()))
        })
        .collect();

    match status_from(&slots) {
        Status::AnotherProgram {
            prog_id,
            program_name: None,
        } => {
            let program_name = friendly_name_for(&prog_id);
            Status::AnotherProgram {
                prog_id,
                program_name,
            }
        }
        settled => settled,
    }
}

/// The name a person would recognise for whoever holds a default.
///
/// Two places, in the order Windows itself prefers: `Application\ApplicationName`
/// first, then the nameless value of the identifier's own key. Read through
/// `HKEY_CLASSES_ROOT`, the merged view of the user's class registrations over
/// the machine's, which is the same view the shell resolves an identifier
/// through, so a per-user override is seen here too.
///
/// **This often finds nothing on Windows 10 and 11, and the reason is not a
/// fault here.** The programs that hold these defaults out of the box are Store
/// applications, whose identifiers look like `AppXbx2ce4vcxjdhff3d1ms66qqzk12zn827`
/// and are resolved through the package repository rather than through the
/// classes registry. Checked by hand on a Windows 11 machine: the identifier
/// recorded for `mailto` had no key under `HKEY_CLASSES_ROOT`, under the user's
/// classes, or under the machine's. Reading the package repository means COM
/// against the package manager, which is a great deal of machinery for a
/// program's name, so it is not done.
///
/// Desktop programs do have a name here, and it is read: `giffile` answers "GIF
/// Image", `htmlfile` answers "HTML Document".
///
/// So the caller keeps the identifier separately and treats the name as a bonus.
/// Anything putting this on screen has to have words for the case where there is
/// no name, and those words should not pretend the holder is unknown: it is
/// known, it just has no readable name here.
fn friendly_name_for(prog_id: &str) -> Option<String> {
    let readable = |reading: Reading| match reading {
        Reading::Value(name) if !name.trim().is_empty() => Some(name),
        Reading::Value(_) | Reading::NotSet | Reading::Failed(_) => None,
    };

    readable(read_registry_string(
        &format!(r"HKCR\{prog_id}\Application"),
        "ApplicationName",
    ))
    .or_else(|| {
        readable(read_registry_string(
            &format!(r"HKCR\{prog_id}"),
            DEFAULT_VALUE,
        ))
    })
}

/// The merged view of class registrations, the user's over the machine's.
///
/// Windows documents these as `0x80000000` and `0x80000001` read as signed 32
/// bit values and then sign-extended to pointer width. Casting straight from
/// `u32` would zero-extend instead and hand the API a handle it has never
/// heard of.
const HKEY_CLASSES_ROOT: isize = 0x8000_0000u32 as i32 as isize;

/// The current user's own part of the registry.
const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;

/// The read worked.
const ERROR_SUCCESS: i32 = 0;
/// The value is not there.
const ERROR_FILE_NOT_FOUND: i32 = 2;
/// The key is not there.
const ERROR_PATH_NOT_FOUND: i32 = 3;
/// The read was refused.
const ERROR_ACCESS_DENIED: i32 = 5;
/// The buffer was too small, and this is how big it needs to be.
const ERROR_MORE_DATA: i32 = 234;

/// Which hive a key path names, and what follows it.
///
/// Only the two this module reads. Anything else, including
/// `HKEY_LOCAL_MACHINE`, comes back as nothing rather than being guessed at:
/// a guessed hive reads a real value from the wrong place and hands it back as
/// an answer, which is worse than refusing.
///
/// Pure, so the paths [`Association::user_choice_key`] builds are checked
/// everywhere rather than only on the platform that can follow them.
fn split_hive(key_path: &str) -> Option<(isize, &str)> {
    if let Some(rest) = key_path.strip_prefix(r"HKCU\") {
        return (!rest.is_empty()).then_some((HKEY_CURRENT_USER, rest));
    }
    if let Some(rest) = key_path.strip_prefix(r"HKCR\") {
        return (!rest.is_empty()).then_some((HKEY_CLASSES_ROOT, rest));
    }
    None
}

/// What one of Windows' registry status codes means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// The value was read.
    Present,
    /// Nothing is there, which is an ordinary state and not a fault.
    Absent,
    /// The read did not work.
    Fault,
}

/// Read a registry status code, apart from anything it came with.
///
/// Gated on nothing, so the mapping is compiled and tested on every platform.
/// Getting it wrong in the quiet direction, by reading a refusal as "nothing
/// is set", turns a fault into a permanent wrong answer that reports itself as
/// normal, which is the failure the whole [`Reading`] split exists to prevent.
fn outcome_of(status: i32) -> Outcome {
    match status {
        ERROR_SUCCESS => Outcome::Present,
        ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => Outcome::Absent,
        _ => Outcome::Fault,
    }
}

/// Whether a sizing call came back with a size in it.
///
/// Asking how big a value is, by passing no buffer, is documented to answer
/// success. Some values answer `ERROR_MORE_DATA` instead, and treating that as
/// a fault would refuse to read them at all.
const fn size_is_known(status: i32) -> bool {
    status == ERROR_SUCCESS || status == ERROR_MORE_DATA
}

/// Text in the shape Windows reads: UTF-16, ending in a null.
///
/// Every string handed to one of these APIs is read until a null, so one
/// missing is a read past the end of the buffer.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Text in the shape the rest of this program reads.
///
/// Stops at the first null. `RegGetValueW` hands back a buffer sized for the
/// value plus its terminator, and sometimes larger than that. Keeping the
/// whole buffer puts null characters inside a `String`, where a screen reader
/// announces them as nothing at all while every comparison against them
/// quietly fails.
///
/// Outside the platform gate, so the decoding is compiled and tested
/// everywhere rather than only where it runs.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn string_from_wide(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

/// Ask Windows for one string value, and answer with what came back.
///
/// Holds no decision, the way `presentation::theme`'s registry call holds
/// none. What a status code means is [`outcome_of`], what a buffer says is
/// [`string_from_wide`], and what a value means for a default is
/// [`held_from`], all of which are compiled and tested everywhere.
///
/// A read, and only a read. Nothing in this module writes to the registry.
#[cfg(target_os = "windows")]
fn read_registry_string(key_path: &str, value_name: &str) -> Reading {
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            hkey: isize,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            out_type: *mut u32,
            out_data: *mut core::ffi::c_void,
            out_size: *mut u32,
        ) -> i32;
    }

    // REG_SZ and REG_EXPAND_SZ. A ProgId is always the first; another
    // program's friendly name can be either, and an expandable one is
    // expanded rather than handed over with `%ProgramFiles%` still in it.
    const RRF_RT_ANY_STRING: u32 = 0x0000_0002 | 0x0000_0004;

    let Some((hive, sub_key)) = split_hive(key_path) else {
        return Reading::Failed(format!("{key_path} does not name a registry hive"));
    };
    let sub_key = wide(sub_key);
    let value = wide(value_name);

    // How big, first. A ProgId is short and a friendly name is not bounded by
    // anything, so the size comes from Windows rather than from a guess here.
    //
    // Safe: both buffers are null-terminated UTF-16 kept alive for the whole
    // call, and passing no data buffer with a zero size is how this API is
    // documented to be asked for a size.
    let mut size: u32 = 0;
    let status = unsafe {
        RegGetValueW(
            hive,
            sub_key.as_ptr(),
            value.as_ptr(),
            RRF_RT_ANY_STRING,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &raw mut size,
        )
    };
    if !size_is_known(status) {
        return reading_for(status, String::new());
    }
    if size == 0 {
        return Reading::Value(String::new());
    }

    // `size` is bytes, including the terminator, and the buffer is units.
    let mut buffer: Vec<u16> = vec![0; (size as usize).div_ceil(size_of::<u16>())];
    let mut written = (buffer.len() * size_of::<u16>()) as u32;
    // Safe: `buffer` is exactly `written` bytes long and stays alive for the
    // call, and `written` tells the API how much room it has. A refused call
    // leaves the buffer as the zeroes it was created with, and the status is
    // read before the buffer ever is.
    let status = unsafe {
        RegGetValueW(
            hive,
            sub_key.as_ptr(),
            value.as_ptr(),
            RRF_RT_ANY_STRING,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &raw mut written,
        )
    };
    let units = (written as usize)
        .div_ceil(size_of::<u16>())
        .min(buffer.len());
    reading_for(status, string_from_wide(&buffer[..units]))
}

/// There is no Windows registry here, and saying so is the honest answer.
///
/// Not `NotSet`: nothing is set because there is nowhere to set it, which is a
/// different claim from a Windows machine where nobody has chosen yet. This
/// surfaces as [`Status::Undetermined`] carrying that sentence, rather than as
/// a quiet "you are not the default".
#[cfg(not(target_os = "windows"))]
fn read_registry_string(_key_path: &str, _value_name: &str) -> Reading {
    Reading::Failed("default programs are a Windows feature, and this is not Windows".to_string())
}

/// Put a status and the text it came with together.
///
/// Separate from the call so the pairing is compiled and tested everywhere,
/// and so the two calls above cannot pair them differently from each other.
fn reading_for(status: i32, text: String) -> Reading {
    match outcome_of(status) {
        Outcome::Present => Reading::Value(text),
        Outcome::Absent => Reading::NotSet,
        // Being refused is the one fault with a plain cause and a next step,
        // so it is explained. Everything else keeps its number, because an
        // unexplained fault still has to be diagnosable from a log.
        Outcome::Fault if status == ERROR_ACCESS_DENIED => Reading::Failed(
            "the setting could not be read, because Windows did not give this program permission"
                .to_string(),
        ),
        Outcome::Fault => Reading::Failed(format!(
            "the setting could not be read, and Windows reported error {status}"
        )),
    }
}

/// What came back from asking the registry for one value.
///
/// Three answers rather than an `Option` or a `Result`, because the two ways
/// of coming back empty mean different things and the difference is worth
/// keeping until something can act on it. Nothing recorded is the ordinary
/// state of a machine where nobody has opened the Settings page. A refused
/// read is a fault, and it should read as one.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Reading {
    /// The value was there, and this is it.
    Value(String),
    /// The key or the value is not there.
    NotSet,
    /// The read itself did not work, and this is what Windows said.
    Failed(String),
}

/// Who holds one slot.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Held {
    /// This program.
    Ours,
    /// Another program, named by the identifier it registered.
    Another(String),
    /// Not established, and this is why.
    Unknown(String),
}

/// Read one recorded choice, without touching a registry to do it.
///
/// The whole decision this module makes about a single slot, kept apart from
/// the call that fetches the string so it can be tested anywhere, on any
/// platform, against inputs a live machine would not produce on demand.
///
/// The comparison ignores case because Windows does. Registry key names are
/// case insensitive and the shell matches ProgIds the same way, so a byte for
/// byte comparison would report a rival holding a default that this program
/// was in fact opening.
fn held_from(reading: &Reading, our_prog_id: &str) -> Held {
    match reading {
        // An empty value is a key that exists with nothing in it, which is no
        // choice rather than a rival with no name.
        Reading::Value(prog_id) if prog_id.trim().is_empty() => {
            Held::Unknown(NOTHING_RECORDED.to_string())
        }
        Reading::Value(prog_id) if prog_id.eq_ignore_ascii_case(our_prog_id) => Held::Ours,
        Reading::Value(prog_id) => Held::Another(prog_id.clone()),
        Reading::NotSet => Held::Unknown(NOTHING_RECORDED.to_string()),
        // Passed through rather than wrapped. The sentence was written where
        // the fault was known, and a second lead-in on top of it produced
        // "the registry refused: Windows did not give this program
        // permission", which says the same thing twice to somebody hearing
        // it read out.
        Reading::Failed(complaint) => Held::Unknown(complaint.clone()),
    }
}

/// What nothing recorded is called, in one place.
///
/// Windows falls back to a machine-wide association when this per-user key is
/// absent, and that association is not read here, so the honest answer is that
/// the question is open rather than that this program lost it.
const NOTHING_RECORDED: &str = "Windows has not recorded a choice for this yet";

/// Fold what is known about each slot into one answer for the kind.
///
/// The rules, in the order they are applied, and each one is a claim this
/// module is willing to put on a screen:
///
/// - No slots at all means Windows has nowhere to record an answer. This is
///   how tasks, reminders and notes come out, and it is a different fact from
///   any of the ones below.
/// - Every slot ours is the only case that reports this program as the
///   default. Anything short of all of them is not "you are the default".
/// - Some ours and some not is reported as exactly that, naming both halves,
///   because rounding it either way hides something true.
/// - None ours with a rival named reports the rival. A second slot nobody
///   could read does not stop that from being the most useful true thing to
///   say.
/// - Nothing ours and nothing readable is undetermined, carrying the reasons
///   so a person is told which kind of not knowing this is.
///
/// A slot that could not be read counts as not held rather than held,
/// throughout. Erring the other way would tell somebody they were already the
/// default and send them away from the button that would have fixed it.
fn status_from(slots: &[(Association, Held)]) -> Status {
    if slots.is_empty() {
        return Status::NoWindowsSlot;
    }

    let held: Vec<String> = slots
        .iter()
        .filter(|(_, holder)| *holder == Held::Ours)
        .map(|(association, _)| association.label())
        .collect();
    if held.len() == slots.len() {
        return Status::Ours;
    }

    let not_held: Vec<String> = slots
        .iter()
        .filter(|(_, holder)| *holder != Held::Ours)
        .map(|(association, _)| association.label())
        .collect();
    if !held.is_empty() {
        return Status::Partial { held, not_held };
    }

    let rival = slots.iter().find_map(|(_, holder)| match holder {
        Held::Another(prog_id) => Some(prog_id.clone()),
        Held::Ours | Held::Unknown(_) => None,
    });
    match rival {
        Some(prog_id) => Status::AnotherProgram {
            prog_id,
            program_name: None,
        },
        None => Status::Undetermined {
            reason: reasons_in(slots),
        },
    }
}

/// Gather why nothing could be established, without repeating one reason.
///
/// Both slots of a calendar usually fail the same way, and a sentence that
/// says the same thing twice reads as a stutter to anybody and as noise to
/// somebody hearing it read out.
fn reasons_in(slots: &[(Association, Held)]) -> String {
    let mut reasons: Vec<&str> = Vec::new();
    for (_, holder) in slots {
        if let Held::Unknown(reason) = holder
            && !reasons.contains(&reason.as_str())
        {
            reasons.push(reason);
        }
    }
    reasons.join("; ")
}

/// What Windows currently does with one kind of thing.
///
/// Five answers, because there are five different true things to say and
/// four of them used to be reported as "no". Which one is showing decides
/// what the screen offers next, so they are kept apart rather than reduced to
/// a boolean somebody would have to guess the meaning of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// This program holds every slot Windows keeps for this kind.
    Ours,
    /// Another program holds it. The identifier is always there, because it
    /// is what the registry actually holds.
    ///
    /// `program_name` is often `None`, and a screen has to be built for that
    /// rather than treating it as rare. The programs holding these defaults
    /// on a stock Windows 10 or 11 are Store applications, and their
    /// identifiers are not resolvable to a name through the registry at all:
    /// see [`friendly_name_for`] for what was checked. A `None` here means
    /// the holder is known and its name is not, which is a different sentence
    /// from "something unknown holds this".
    AnotherProgram {
        prog_id: String,
        program_name: Option<String>,
    },
    /// Some of this kind's slots and not others, named both ways. Only
    /// calendar can reach this, having two slots, and it reaches it often: a
    /// person can open `.ics` files here and still subscribe through a
    /// browser.
    Partial {
        held: Vec<String>,
        not_held: Vec<String>,
    },
    /// Windows has no default to hold for this kind. Not a failure, and not
    /// a "no": there is nothing here to be the default for.
    NoWindowsSlot,
    /// It could not be established, and this says which kind of not knowing
    /// it is.
    Undetermined { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_is_the_mailto_protocol_and_nothing_else() {
        assert_eq!(
            DefaultKind::Mail.associations(),
            &[Association::Protocol("mailto")]
        );
    }

    #[test]
    fn test_calendar_covers_both_the_ics_file_type_and_the_webcal_protocol() {
        assert_eq!(
            DefaultKind::Calendar.associations(),
            &[
                Association::FileType(".ics"),
                Association::Protocol("webcal")
            ]
        );
    }

    #[test]
    fn test_contacts_is_the_vcf_file_type() {
        assert_eq!(
            DefaultKind::Contacts.associations(),
            &[Association::FileType(".vcf")]
        );
    }

    #[test]
    fn test_tasks_reminders_and_notes_claim_nothing_because_windows_has_no_slot() {
        for kind in [
            DefaultKind::Tasks,
            DefaultKind::Reminders,
            DefaultKind::Notes,
        ] {
            assert!(kind.associations().is_empty(), "{kind:?} claimed a slot");
        }
    }

    #[test]
    fn test_a_protocol_reads_its_choice_from_the_url_associations_key() {
        assert_eq!(
            Association::Protocol("mailto").user_choice_key(),
            r"HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\mailto\UserChoice"
        );
    }

    #[test]
    fn test_a_file_type_reads_its_choice_from_the_file_exts_key() {
        assert_eq!(
            Association::FileType(".ics").user_choice_key(),
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.ics\UserChoice"
        );
    }

    #[test]
    fn test_the_recorded_choice_being_ours_reads_as_ours() {
        let held = held_from(
            &Reading::Value("WixenMail.Url.mailto".to_string()),
            "WixenMail.Url.mailto",
        );

        assert_eq!(held, Held::Ours);
    }

    #[test]
    fn test_a_prog_id_is_matched_the_way_windows_matches_it_which_is_without_case() {
        // Registry key names are case insensitive and the shell compares
        // ProgIds the same way, so a `UserChoice` that came back in a
        // different case is still ours. Comparing byte for byte would report
        // "another program holds this" while this program was in fact opening
        // every message, and the button offering to fix it would do nothing a
        // person could see.
        let held = held_from(
            &Reading::Value("wixenmail.url.MAILTO".to_string()),
            "WixenMail.Url.mailto",
        );

        assert_eq!(held, Held::Ours);
    }

    #[test]
    fn test_another_programs_prog_id_reads_as_that_program_and_keeps_its_name() {
        // The identifier is kept rather than reduced to a flag, because it is
        // the only handle on who the other program is.
        let held = held_from(
            &Reading::Value("Outlook.URL.mailto.15".to_string()),
            "WixenMail.Url.mailto",
        );

        assert_eq!(held, Held::Another("Outlook.URL.mailto.15".to_string()));
    }

    #[test]
    fn test_no_recorded_choice_and_a_failed_read_do_not_give_the_same_answer() {
        // Two ways of not knowing, and they mean opposite things to whoever
        // reads them. Nothing recorded is an ordinary machine where the person
        // has never opened the Settings page, and Windows is falling back to
        // an association this key knows nothing about. A failed read is a
        // fault. Collapsing them loses the difference in the one place it
        // could have been noticed.
        let never_chosen = held_from(&Reading::NotSet, "WixenMail.Url.mailto");
        let broken = held_from(
            &Reading::Failed("error 5".to_string()),
            "WixenMail.Url.mailto",
        );

        let (Held::Unknown(quiet), Held::Unknown(loud)) = (&never_chosen, &broken) else {
            panic!("expected both to be unknown, got {never_chosen:?} and {broken:?}");
        };
        assert_ne!(quiet, loud, "both ways of not knowing said the same thing");
        assert!(loud.contains("error 5"), "the fault was dropped: {loud}");
    }

    #[test]
    fn test_an_empty_recorded_choice_is_no_choice_rather_than_another_program() {
        // A `UserChoice` key can exist with an empty `ProgId`, and reading
        // that as another program's identifier would report an unnamed rival
        // holding the default forever.
        assert_eq!(
            held_from(&Reading::Value(String::new()), "WixenMail.Url.mailto"),
            held_from(&Reading::NotSet, "WixenMail.Url.mailto")
        );
    }

    #[test]
    fn test_a_kind_windows_has_no_slot_for_says_so_rather_than_saying_not_ours() {
        // The whole reason this variant exists. Tasks, reminders and notes
        // have no file type and no protocol, so "another program is the
        // default" would be an invented claim and "could not be determined"
        // would blame a reading that never had anything to read.
        assert_eq!(status_from(&[]), Status::NoWindowsSlot);
    }

    #[test]
    fn test_every_slot_being_ours_reads_as_ours() {
        let status = status_from(&[
            (Association::FileType(".ics"), Held::Ours),
            (Association::Protocol("webcal"), Held::Ours),
        ]);

        assert_eq!(status, Status::Ours);
    }

    #[test]
    fn test_no_slot_being_ours_names_the_program_that_holds_one() {
        let status = status_from(&[(
            Association::Protocol("mailto"),
            Held::Another("Outlook.URL.mailto.15".to_string()),
        )]);

        assert_eq!(
            status,
            Status::AnotherProgram {
                prog_id: "Outlook.URL.mailto.15".to_string(),
                program_name: None,
            }
        );
    }

    #[test]
    fn test_holding_one_slot_of_two_is_neither_ours_nor_theirs() {
        // Calendar is the kind this happens to: a person can open `.ics`
        // files here and still subscribe through a browser. Rounding that up
        // to "Wixen Mail is your default calendar" hides half of it, and
        // rounding it down to the browser hides the other half. Both halves
        // are named so the screen can read them out.
        let status = status_from(&[
            (Association::FileType(".ics"), Held::Ours),
            (
                Association::Protocol("webcal"),
                Held::Another("ChromeHTML".to_string()),
            ),
        ]);

        assert_eq!(
            status,
            Status::Partial {
                held: vec![".ics files".to_string()],
                not_held: vec!["webcal: links".to_string()],
            }
        );
    }

    #[test]
    fn test_a_slot_that_could_not_be_read_counts_as_not_held_rather_than_held() {
        // Erring towards the claim this program cannot support. Saying "you
        // are already the default" when the read failed sends somebody away
        // from the button that would have fixed it.
        let status = status_from(&[
            (Association::FileType(".ics"), Held::Ours),
            (
                Association::Protocol("webcal"),
                Held::Unknown("nothing recorded".to_string()),
            ),
        ]);

        assert_eq!(
            status,
            Status::Partial {
                held: vec![".ics files".to_string()],
                not_held: vec!["webcal: links".to_string()],
            }
        );
    }

    #[test]
    fn test_knowing_nothing_about_any_slot_is_undetermined_and_says_why() {
        let status = status_from(&[(
            Association::Protocol("mailto"),
            Held::Unknown("no choice recorded".to_string()),
        )]);

        let Status::Undetermined { reason } = &status else {
            panic!("expected undetermined, got {status:?}");
        };
        assert!(
            reason.contains("no choice recorded"),
            "the reason was dropped: {reason}"
        );
    }

    #[test]
    fn test_one_readable_rival_outweighs_an_unreadable_slot() {
        // Nothing here is ours, one slot names a rival and the other could not
        // be read. Naming the rival is the more useful of the two true things
        // that can be said, and it is still true.
        let status = status_from(&[
            (
                Association::FileType(".ics"),
                Held::Another("Outlook.File.ics.15".to_string()),
            ),
            (
                Association::Protocol("webcal"),
                Held::Unknown("nothing recorded".to_string()),
            ),
        ]);

        assert_eq!(
            status,
            Status::AnotherProgram {
                prog_id: "Outlook.File.ics.15".to_string(),
                program_name: None,
            }
        );
    }

    #[test]
    fn test_a_slot_reads_out_as_words_rather_than_as_a_registry_fragment() {
        // These land in `Status::Partial` and get spoken.
        assert_eq!(Association::Protocol("mailto").label(), "mailto: links");
        assert_eq!(Association::FileType(".vcf").label(), ".vcf files");
    }

    #[test]
    fn test_each_slot_this_program_claims_is_described_in_ordinary_words() {
        // Every arm, because this is the text Explorer shows in its Type
        // column and the text an Open With list reads out, and a wrong arm
        // labels calendar files as contacts to everybody on the machine.
        assert_eq!(Association::Protocol("mailto").describes(), "Email link");
        assert_eq!(
            Association::Protocol("webcal").describes(),
            "Calendar subscription link"
        );
        assert_eq!(Association::FileType(".ics").describes(), "Calendar file");
        assert_eq!(Association::FileType(".vcf").describes(), "Contact card");
    }

    #[test]
    fn test_a_slot_nobody_wrote_words_for_still_gets_some() {
        // The last arm. A slot added above without a description here would
        // otherwise be registered with an empty one, and Windows shows an
        // empty type as the extension in capitals, which is a machine name
        // being read out. This is not reachable through any kind today, which
        // is why it is reached here directly.
        assert_eq!(Association::Protocol("ftp").describes(), "ftp: links");
        assert_eq!(Association::FileType(".eml").describes(), ".eml files");
    }

    #[test]
    fn test_a_key_path_is_split_into_the_hive_it_names_and_the_rest() {
        // Both hives, because they are read for different things: the choice
        // lives under the current user, and the name of whoever holds it
        // lives in the merged classes view.
        assert_eq!(
            split_hive(r"HKCU\Software\Wixen"),
            Some((HKEY_CURRENT_USER, "Software\\Wixen"))
        );
        assert_eq!(
            split_hive(r"HKCR\Outlook.URL.mailto.15"),
            Some((HKEY_CLASSES_ROOT, "Outlook.URL.mailto.15"))
        );
    }

    #[test]
    fn test_a_key_path_naming_no_hive_is_refused_rather_than_guessed_at() {
        // Guessing a hive would read a real value from the wrong place and
        // report it as an answer. There is no safe default here.
        for path in [
            r"HKLM\Software\Wixen",
            r"Software\Wixen",
            "HKCU",
            "",
            r"\Software",
        ] {
            assert_eq!(split_hive(path), None, "{path:?} was accepted");
        }
    }

    #[test]
    fn test_a_registry_status_says_which_of_the_two_failures_it_is() {
        // Every arm. Reading "access denied" as "nothing is set" is how a
        // fault becomes a permanent, silent, wrong answer on somebody's
        // machine, which is the failure this whole distinction exists for.
        assert_eq!(outcome_of(ERROR_SUCCESS), Outcome::Present);
        assert_eq!(outcome_of(ERROR_FILE_NOT_FOUND), Outcome::Absent);
        assert_eq!(outcome_of(ERROR_PATH_NOT_FOUND), Outcome::Absent);
        assert_eq!(outcome_of(ERROR_ACCESS_DENIED), Outcome::Fault);
        assert_eq!(outcome_of(ERROR_MORE_DATA), Outcome::Fault);
        assert_eq!(outcome_of(-1), Outcome::Fault);
    }

    #[test]
    fn test_a_refused_read_is_explained_rather_than_read_out_as_a_number() {
        // This sentence reaches `Status::Undetermined` and gets spoken. "5"
        // is a machine name being read to somebody who cannot see that it is
        // one, and being refused permission is the one fault with a plain
        // cause and a next step, so it gets words. Anything else keeps its
        // number, because an unexplained fault still has to be diagnosable.
        let Reading::Failed(refused) = reading_for(ERROR_ACCESS_DENIED, String::new()) else {
            panic!("a refusal did not read as a failure");
        };
        let Reading::Failed(unexplained) = reading_for(-2147, String::new()) else {
            panic!("an unknown code did not read as a failure");
        };

        assert!(
            refused.contains("permission"),
            "a refusal is not explained: {refused}"
        );
        assert_ne!(refused, unexplained, "every fault says the same thing");
        assert!(
            unexplained.contains("-2147"),
            "an unexplained fault lost the only clue it had: {unexplained}"
        );
    }

    #[test]
    fn test_a_read_that_failed_keeps_its_own_words_all_the_way_up() {
        // The sentence written where the fault is known must be the sentence
        // a person hears. Wrapping it in a second lead-in produced "the
        // registry refused: Windows would not give this program permission",
        // which says the same thing twice.
        let held = held_from(
            &Reading::Failed("Windows fell over".to_string()),
            "WixenMail.Url.mailto",
        );

        assert_eq!(held, Held::Unknown("Windows fell over".to_string()));
    }

    #[test]
    fn test_the_sizing_call_accepts_both_answers_windows_gives_it() {
        // Asking how big a value is, by passing no buffer, is documented to
        // answer ERROR_SUCCESS. Some values answer ERROR_MORE_DATA instead,
        // and treating that as a fault would refuse to read them at all.
        assert!(size_is_known(ERROR_SUCCESS));
        assert!(size_is_known(ERROR_MORE_DATA));
        assert!(!size_is_known(ERROR_ACCESS_DENIED));
        assert!(!size_is_known(ERROR_FILE_NOT_FOUND));
    }

    #[test]
    fn test_text_handed_to_windows_ends_where_windows_expects_it_to() {
        // Every one of these goes to an API that reads until a null. Without
        // one it reads past the end of the buffer.
        assert_eq!(wide("AB"), vec![0x0041, 0x0042, 0x0000]);
        assert_eq!(wide(""), vec![0x0000]);
        assert_eq!(wide("é").last(), Some(&0x0000));
    }

    #[test]
    fn test_every_association_has_its_own_prog_id() {
        let mut seen: Vec<String> = Vec::new();
        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                let prog_id = association.our_prog_id();
                assert!(!seen.contains(&prog_id), "{prog_id} claimed twice");
                seen.push(prog_id);
            }
        }
        assert_eq!(seen.len(), 4, "the four slots Windows has for us: {seen:?}");
    }

    /// An executable path with a space in it, which is where this really is.
    const INSTALLED: &str = r"C:\Program Files\Wixen Mail\wixen-mail.exe";

    /// The value written at `key` under `name`, if the entries carry one.
    fn value_at(entries: &[(String, String, String)], key: &str, name: &str) -> Option<String> {
        entries
            .iter()
            .find(|(k, n, _)| k.eq_ignore_ascii_case(key) && n.eq_ignore_ascii_case(name))
            .map(|(_, _, value)| value.clone())
    }

    #[test]
    fn test_the_application_is_named_and_described_or_windows_leaves_it_out() {
        // Microsoft's rule, and it is a silent one: with no
        // ApplicationDescription "the application does not appear in UI lists
        // of potential default programs". Every other entry in this list could
        // be perfect and the Settings page would show nothing, with no error
        // anywhere to explain it.
        let entries = capability_entries_for(INSTALLED);

        assert_eq!(
            value_at(&entries, CAPABILITY_KEY, "ApplicationName").as_deref(),
            Some(APPLICATION_NAME)
        );
        let description = value_at(&entries, CAPABILITY_KEY, "ApplicationDescription")
            .expect("no ApplicationDescription, so Windows will not list this program");
        assert!(!description.trim().is_empty(), "an empty description");
    }

    #[test]
    fn test_registered_applications_points_at_the_key_that_is_actually_written() {
        // The one entry that ties the other two dozen together. Windows reads
        // RegisteredApplications to find the capability key, so a value that
        // names a key nothing writes is a dangling pointer: the program never
        // appears, and nothing anywhere reports a fault. Comparing the two
        // strings against each other, rather than each against a literal,
        // means renaming the key cannot leave the pointer behind.
        let entries = capability_entries_for(INSTALLED);

        let pointer = value_at(&entries, REGISTERED_APPLICATIONS_KEY, APPLICATION_NAME)
            .expect("the program is not registered, so Windows will never look for it");
        let written = format!(r"{}\{pointer}", "HKCU");
        assert_eq!(
            written, CAPABILITY_KEY,
            "RegisteredApplications names a key nothing writes"
        );
    }

    #[test]
    fn test_the_name_under_registered_applications_matches_the_application_name() {
        // Microsoft: "ApplicationName must always match the name that is
        // registered under RegisteredApplications." Two spellings of the same
        // program is a listing that does not resolve.
        let entries = capability_entries_for(INSTALLED);

        assert!(
            entries.iter().any(|(key, name, _)| key
                .eq_ignore_ascii_case(REGISTERED_APPLICATIONS_KEY)
                && name == APPLICATION_NAME),
            "the registered name is spelled differently from ApplicationName"
        );
    }

    #[test]
    fn test_every_slot_is_claimed_under_the_subkey_windows_reads_for_its_sort() {
        // A protocol claimed under FileAssociations, or an extension claimed
        // under UrlAssociations, is a claim Windows does not read at all.
        let entries = capability_entries_for(INSTALLED);

        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                let (subkey, claimed_as) = match association {
                    Association::Protocol(scheme) => ("UrlAssociations", *scheme),
                    Association::FileType(extension) => ("FileAssociations", *extension),
                };
                let key = format!(r"{CAPABILITY_KEY}\{subkey}");
                assert_eq!(
                    value_at(&entries, &key, claimed_as).as_deref(),
                    Some(association.our_prog_id().as_str()),
                    "{claimed_as} is not claimed under {subkey}"
                );
            }
        }
    }

    #[test]
    fn test_every_claimed_prog_id_has_something_that_can_open_it() {
        // A claim with no open command behind it puts this program in the
        // Settings list and then does nothing when somebody picks it, which
        // is worse than not being listed.
        let entries = capability_entries_for(INSTALLED);

        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                let command_key = format!(
                    r"HKCU\Software\Classes\{}\shell\open\command",
                    association.our_prog_id()
                );
                let command = value_at(&entries, &command_key, "")
                    .unwrap_or_else(|| panic!("{association:?} has no open command"));
                assert!(
                    command.contains(INSTALLED),
                    "the command does not run this program: {command}"
                );
                assert!(
                    command.contains("%1"),
                    "the command is handed nothing to open: {command}"
                );
            }
        }
    }

    #[test]
    fn test_the_open_command_quotes_the_executable_so_program_files_survives() {
        // Unquoted, `C:\Program Files\Wixen Mail\wixen-mail.exe "%1"` is read
        // by the shell as a request to run `C:\Program` with the rest as
        // arguments. This is the single most common way a registered handler
        // silently fails, and the default install path has two spaces in it.
        let entries = capability_entries_for(INSTALLED);

        let command = value_at(
            &entries,
            r"HKCU\Software\Classes\WixenMail.Url.mailto\shell\open\command",
            "",
        )
        .expect("no open command for mailto");

        assert_eq!(command, format!(r#""{INSTALLED}" "%1""#));
    }

    #[test]
    fn test_a_protocol_carries_the_url_protocol_marker_and_a_file_type_does_not() {
        // The empty `URL Protocol` value is what tells the shell a ProgId can
        // be the target of a scheme. Without it a `mailto:` link finds
        // nothing. Putting it on a file type instead claims `.ics` is a
        // protocol, which is a claim about something that does not exist.
        let entries = capability_entries_for(INSTALLED);

        let marked = |association: Association| {
            value_at(
                &entries,
                &format!(r"HKCU\Software\Classes\{}", association.our_prog_id()),
                "URL Protocol",
            )
            .is_some()
        };

        assert!(marked(Association::Protocol("mailto")), "mailto unmarked");
        assert!(marked(Association::Protocol("webcal")), "webcal unmarked");
        assert!(!marked(Association::FileType(".ics")), ".ics marked");
        assert!(!marked(Association::FileType(".vcf")), ".vcf marked");
    }

    #[test]
    fn test_every_prog_id_is_described_in_words_rather_than_by_its_identifier() {
        // The default value of a ProgId key is what Explorer puts in the Type
        // column and what an Open With list reads out. Left empty, a person
        // hears "WixenMail.AssocFile.ics", which is a machine name being read
        // to somebody who cannot see that it is one.
        let entries = capability_entries_for(INSTALLED);

        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                let prog_id = association.our_prog_id();
                let described =
                    value_at(&entries, &format!(r"HKCU\Software\Classes\{prog_id}"), "")
                        .unwrap_or_else(|| panic!("{prog_id} has no description"));
                assert!(!described.trim().is_empty(), "{prog_id} is described as ''");
                assert!(
                    !described.contains('.') || !described.contains("WixenMail"),
                    "{prog_id} is described by its own identifier: {described}"
                );
            }
        }
    }

    #[test]
    fn test_nothing_in_the_entries_tries_to_make_this_program_the_default() {
        // The guard for the promise this module's own documentation makes.
        // Writing a `UserChoice` key is the forgery route: Windows hashes that
        // value against the user's identity, treats a mismatch as tampering,
        // and resets the association. An entry reaching one of those keys
        // would be this module doing the exact thing it says cannot be done.
        let entries = capability_entries_for(INSTALLED);

        for (key, name, _) in &entries {
            let lowered = key.to_lowercase();
            assert!(
                !lowered.contains("userchoice"),
                "an entry writes a UserChoice key: {key}\\{name}"
            );
            assert!(
                !lowered.contains(r"shell\associations"),
                "an entry writes the shell's own association store: {key}\\{name}"
            );
        }
    }

    #[test]
    fn test_every_entry_is_under_the_current_user_because_the_installer_is_not_elevated() {
        // `PrivilegesRequired=lowest` in the installer, so an entry under
        // HKEY_LOCAL_MACHINE is one the installer cannot write on an ordinary
        // account. Per-user is also what the association itself is, so this
        // costs nothing.
        for (key, name, _) in capability_entries_for(INSTALLED) {
            assert!(
                key.starts_with(r"HKCU\"),
                "{key}\\{name} is not under the current user"
            );
        }
    }

    #[test]
    fn test_no_entry_is_written_twice_with_two_different_values() {
        // Two rows naming the same key and value is one of them silently
        // losing, decided by whichever the installer happens to apply last.
        let entries = capability_entries_for(INSTALLED);

        for (index, (key, name, value)) in entries.iter().enumerate() {
            for (other_key, other_name, other_value) in &entries[index + 1..] {
                let same_place =
                    key.eq_ignore_ascii_case(other_key) && name.eq_ignore_ascii_case(other_name);
                assert!(
                    !same_place || value == other_value,
                    "{key}\\{name} is written twice with different values"
                );
            }
        }
    }

    #[test]
    fn test_the_running_program_can_produce_its_own_entries() {
        // The no-argument form is what a caller with no install path uses, and
        // it has to find the executable itself. This is the reachability
        // check: it runs the same code the installer path runs.
        let entries = capability_registry_entries();

        assert!(!entries.is_empty(), "no entries at all");
        assert!(
            entries.iter().any(
                |(key, name, _)| key == REGISTERED_APPLICATIONS_KEY && name == APPLICATION_NAME
            ),
            "the running program did not register itself"
        );
    }

    #[test]
    fn test_the_settings_page_is_the_one_windows_answers() {
        // `ms-settings:defaultapps` is the documented URI, and a typo in it
        // fails as a shell error somewhere far away from this line.
        assert_eq!(DEFAULT_APPS_SETTINGS_URI, "ms-settings:defaultapps");
    }

    #[test]
    fn test_the_settings_page_opens_at_this_program_and_names_it_the_way_windows_knows_it() {
        // Windows matches this against the name under RegisteredApplications.
        // Spelled differently, it opens the general list of every application
        // on the machine, which for somebody working by keyboard means
        // arrowing through the lot to find one entry. Built from the same
        // constant as the registration so the two cannot come apart.
        let uri = default_apps_settings_uri_for_this_program();

        assert_eq!(
            uri,
            "ms-settings:defaultapps?registeredAppUser=Wixen%20Mail"
        );
        assert!(uri.starts_with(DEFAULT_APPS_SETTINGS_URI), "{uri}");
        assert!(
            !uri.contains(' '),
            "a space in a URI is not carried through the shell: {uri}"
        );
        assert_eq!(
            uri.replace("%20", " ")
                .rsplit_once('=')
                .map(|(_, name)| name),
            Some(APPLICATION_NAME),
            "the page is opened at a name nothing is registered under"
        );
    }

    #[test]
    fn test_the_settings_page_names_the_user_registration_and_not_the_machine_one() {
        // Windows takes `registeredAppUser` and `registeredAppMachine` and
        // looks in different hives for each. This program registers under the
        // current user, so the machine form would find nothing and fall back
        // to the general list, with nothing to say why.
        let uri = default_apps_settings_uri_for_this_program();

        assert!(uri.contains("registeredAppUser="), "{uri}");
        assert!(!uri.contains("registeredAppMachine"), "{uri}");
        assert!(
            CAPABILITY_KEY.starts_with(r"HKCU\"),
            "the registration moved to the machine and this link did not"
        );
    }

    #[test]
    fn test_a_windows_string_stops_at_its_terminator() {
        // `RegGetValueW` hands back a buffer sized for the value plus its
        // null, and on some values more than that. Reading the whole buffer
        // puts null characters into a String, and a screen reader announces
        // those as nothing at all while the comparison against our ProgId
        // quietly fails.
        assert_eq!(string_from_wide(&[0x0041, 0x0042, 0x0000, 0x0043]), "AB");
        assert_eq!(string_from_wide(&[0x0041, 0x0042]), "AB");
        assert_eq!(string_from_wide(&[0x0000]), "");
        assert_eq!(string_from_wide(&[]), "");
    }

    #[test]
    fn test_a_windows_string_survives_characters_that_are_not_ascii() {
        // Another program's friendly name is whatever its publisher wrote,
        // in whatever language.
        assert_eq!(string_from_wide(&[0x00E9, 0x0074, 0x00E9, 0x0000]), "été");
        // A surrogate pair, which is two units and one character.
        assert_eq!(
            string_from_wide(&[0xD83D, 0xDCC5, 0x0000]).chars().count(),
            1
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_the_registry_read_can_actually_see_a_value() {
        // Before believing any "nothing is set" answer, prove the reader can
        // see something that is. A reader that always came back NotSet would
        // report every kind as undetermined forever and every test above
        // would still pass, because every test above feeds it strings.
        //
        // These two are written when a Windows user profile is created, so at
        // least one is present on any machine this runs on. Read only:
        // nothing here writes to Pratik's registry.
        let candidates = [
            (
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders",
                "Desktop",
            ),
            (r"HKCU\Environment", "TEMP"),
        ];

        let seen: Vec<Reading> = candidates
            .iter()
            .map(|(key, name)| read_registry_string(key, name))
            .collect();

        assert!(
            seen.iter()
                .any(|reading| matches!(reading, Reading::Value(text) if !text.is_empty())),
            "the reader saw nothing at either well known value: {seen:?}"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_a_key_that_is_not_there_reads_as_not_set_rather_than_as_a_fault() {
        // The other half of the same measurement, and the one that keeps the
        // two kinds of not knowing apart against the real API rather than
        // against a hand-written Reading.
        let reading = read_registry_string(
            r"HKCU\Software\Wixen\NoSuchKeyExistsAnywhere",
            "NoSuchValue",
        );

        assert_eq!(
            reading,
            Reading::NotSet,
            "an absent key was read as {reading:?}"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_a_programs_name_is_really_read_out_of_the_class_registrations() {
        // The second hive this module reads, and the one nothing else here
        // proves. Every other Windows test reads HKCU, so the classes handle
        // could have been wrong, or this whole lookup dead, and all of them
        // would still be green.
        //
        // These three are Windows' own file types, present on any machine
        // this runs on, and they are read rather than written.
        let known = [
            ("giffile", "GIF Image"),
            ("htmlfile", "HTML Document"),
            ("batfile", "Windows Batch File"),
        ];

        let read: Vec<Option<String>> = known
            .iter()
            .map(|(prog_id, _)| friendly_name_for(prog_id))
            .collect();

        assert!(
            known.iter().zip(&read).any(|((_, expected), found)| found
                .as_deref()
                .is_some_and(|name| name == *expected)),
            "no name came back from the class registrations: {read:?}"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_an_identifier_with_no_name_anywhere_comes_back_without_one() {
        // The case Windows 10 and 11 actually hand this module most of the
        // time, since the programs holding these defaults out of the box are
        // Store applications whose identifiers are not in the class
        // registrations at all. It has to answer "no name", not a fault and
        // not an empty string that would be spoken as silence.
        assert_eq!(friendly_name_for("WixenMail.NoSuchIdentifier"), None);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_the_answer_for_mail_agrees_with_windows_own_reader() {
        // The measurement, taken with a second instrument. Every other test
        // here reaches the registry through `read_registry_string`, so a
        // version of it that always came back empty would leave all of them
        // green and every answer in the product wrong. `reg.exe` is Windows'
        // own reader and knows nothing about this code, so the two agreeing
        // is worth something.
        //
        // What is pinned is that the two agree, not what either says. This
        // machine's default mail program is Pratik's business and changes.
        let queried = std::process::Command::new("reg")
            .args([
                "query",
                Association::Protocol("mailto").user_choice_key().as_str(),
                "/v",
                USER_CHOICE_VALUE,
            ])
            .output()
            .expect("reg.exe would not run, so this check measured nothing");

        let printed = String::from_utf8_lossy(&queried.stdout);
        let recorded = printed
            .lines()
            .filter(|_| queried.status.success())
            .find_map(|line| line.split("REG_SZ").nth(1))
            .map(str::trim)
            .filter(|text| !text.is_empty());

        match (recorded, is_default(DefaultKind::Mail)) {
            // The usual case on any machine somebody uses: another program
            // holds it, and both readers name the same one.
            (Some(theirs), Status::AnotherProgram { prog_id, .. }) => assert!(
                prog_id.eq_ignore_ascii_case(theirs),
                "reg.exe read {theirs} and this module read {prog_id}"
            ),
            // Only once this program has been installed and chosen.
            (Some(_), Status::Ours) => {}
            (Some(theirs), answered) => {
                panic!("reg.exe read {theirs} and this module answered {answered:?}")
            }
            (None, answered) => assert!(
                matches!(answered, Status::Undetermined { .. }),
                "reg.exe found nothing recorded and this module answered {answered:?}"
            ),
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_asking_about_every_kind_answers_without_falling_over() {
        // The end to end path on the machine it runs on. What comes back
        // depends on how this machine is set up, so what is pinned is the
        // shape: the three kinds Windows has nothing for must say so, and the
        // three it does have must not.
        for kind in DefaultKind::ALL {
            let status = is_default(kind);
            let has_slot = !kind.associations().is_empty();

            assert_eq!(
                status == Status::NoWindowsSlot,
                !has_slot,
                "{kind:?} answered {status:?}"
            );
        }
    }
}

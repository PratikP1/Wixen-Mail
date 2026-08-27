//! Putting this program into the list Windows offers when somebody chooses a
//! default, and taking it out again.
//!
//! [`crate::service::default_apps`] works out *what* to register and reads
//! back who currently holds a default. This module is the half that writes,
//! and it exists because nothing wrote it: the entries were produced as data,
//! nothing applied them, and so Wixen Mail did not appear under Settings,
//! Apps, Default apps at all. A person could not choose it even if they
//! wanted to. Thunderbird and Firefox appear there because their installers
//! write these keys.
//!
//! # This still does not make anything the default
//!
//! It makes this program *choosable*. Which program actually opens a `mailto:`
//! link is recorded under `UserChoice`, Windows guards that with a hash it
//! does not document, and nothing here goes near it. See
//! [`crate::service::default_apps`] for the whole of why.
//!
//! # Under the current user, not the machine
//!
//! Every key goes below `HKEY_CURRENT_USER`. Three reasons, and they agree:
//!
//! 1. It needs no administrator rights, so the running program can do it from
//!    a settings screen. A machine-wide write would need an elevated
//!    installer, and this installer runs with `PrivilegesRequired=lowest`.
//! 2. The association itself is per-user. A machine-wide claim is one person
//!    deciding on behalf of everybody who shares the computer.
//! 3. It works. Checked on this machine rather than assumed: Zoom registers
//!    `ZoomPBX` only under `HKCU\Software\RegisteredApplications`, with its
//!    capability key and its identifier under the user's hive too, and
//!    nothing of it exists under the machine. Firefox writes a second copy of
//!    its whole registration there beside the machine-wide one.
//!
//! Measured as well as read. Writing a capability registration for a made-up
//! file type under `HKCU` alone, and then asking the shell's own
//! `SHAssocEnumHandlers` what it would offer for that type, brought back the
//! program that had just been registered and had not been there a moment
//! before. No restart, no sign out. The shell reads this.
//!
//! # The order matters, both ways
//!
//! `RegisteredApplications` is the only entry Windows finds without being told
//! where to look, and everything else hangs off it. So it is written **last**,
//! after the key it points at exists, and removed **first**, before the key it
//! points at stops existing. Written the other way round there is a window,
//! short but real, in which Windows follows the pointer to a key that is half
//! there and lists a program that cannot open anything.

use crate::common::Result;
use crate::service::default_apps::{
    APPLICATION_NAME, CAPABILITY_KEY, CAPABILITY_KEY_CONTAINERS, CLASSES_KEY, DefaultKind,
    REGISTERED_APPLICATIONS_KEY, capability_entries_for,
};

/// One registration, ready to be written or taken away.
///
/// Holds the entries rather than recomputing them, so that what removal takes
/// out is derived from what writing put in. Two lists written separately are
/// two lists that drift, and the half that drifts is the one nobody runs:
/// everybody installs, few uninstall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    /// Every key, value name and value, in the order they are written.
    entries: Vec<(String, String, String)>,
    /// Every key removal deletes whole, deepest first.
    keys_owned: Vec<String>,
    /// Keys removal deletes only when they turn out to be empty.
    keys_to_tidy: Vec<String>,
    /// The one value in a key belonging to Windows: its key, then its name.
    pointer: (String, String),
}

impl Registration {
    /// The registration for the copy of this program that is running.
    ///
    /// Right for a settings screen, which is running from the folder it was
    /// installed into. An installer knows the folder and is not running from
    /// it, so it should use [`Self::for_program_at`].
    pub fn for_this_program() -> Self {
        let executable = std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| EXECUTABLE_FILE_NAME.to_string());
        Self::for_program_at(&executable)
    }

    /// The registration for a program installed at `executable_path`.
    pub fn for_program_at(executable_path: &str) -> Self {
        Self::built(executable_path, None)
    }

    /// The same registration, written somewhere that claims nothing.
    ///
    /// `scratch_key` is a path below `HKEY_CURRENT_USER`, and every key moves
    /// under it: `Software\Classes\...` becomes
    /// `<scratch_key>\Software\Classes\...`, and so does the value that names
    /// the capability key, because that value is itself a key path.
    ///
    /// This is how the writing and the removing are tested for real. A test
    /// that wrote the production keys would edit the registration the person
    /// running the tests is actually using, and a test that only compared
    /// strings would never have run `RegSetValueExW` at all. Nothing under a
    /// scratch key is read by Windows, so a test that failed half way through
    /// leaves rubbish and not a broken machine.
    pub fn in_a_scratch_key(executable_path: &str, scratch_key: &str) -> Self {
        Self::built(executable_path, Some(scratch_key))
    }

    fn built(executable_path: &str, scratch_key: Option<&str>) -> Self {
        let moved = |key: &str| match scratch_key {
            None => key.to_string(),
            Some(scratch) => match key.strip_prefix(USER_HIVE_PREFIX) {
                Some(rest) => format!("{USER_HIVE_PREFIX}{scratch}\\{rest}"),
                // Left alone rather than guessed at. Nothing here produces a
                // key outside the user's hive, and a test below fails if one
                // ever appears, so this arm is unreachable rather than a
                // silent fallback that would write to a real key.
                None => key.to_string(),
            },
        };

        let pointer = (
            moved(REGISTERED_APPLICATIONS_KEY),
            APPLICATION_NAME.to_string(),
        );

        let mut entries: Vec<(String, String, String)> = capability_entries_for(executable_path)
            .into_iter()
            .map(|(key, name, value)| {
                // The one value in the whole registration that is itself a key
                // path. Moving the key it sits in without moving the path it
                // holds would leave a scratch registration pointing at the
                // real capability key.
                let value = match scratch_key {
                    Some(scratch)
                        if key == REGISTERED_APPLICATIONS_KEY && name == APPLICATION_NAME =>
                    {
                        format!("{scratch}\\{value}")
                    }
                    _ => value,
                };
                (moved(&key), name, value)
            })
            .collect();

        // Last, so that Windows never follows it to a key that is half
        // written. Sorting by whether an entry is the pointer, rather than
        // removing and pushing it, keeps every other entry in the order the
        // entry list produced.
        entries.sort_by_key(|(key, name, _)| (key, name) == (&pointer.0, &pointer.1));

        let mut keys_owned: Vec<String> = vec![moved(CAPABILITY_KEY)];
        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                keys_owned.push(moved(&format!(
                    "{CLASSES_KEY}\\{}",
                    association.our_prog_id()
                )));
            }
        }

        Self {
            entries,
            keys_owned,
            keys_to_tidy: CAPABILITY_KEY_CONTAINERS
                .iter()
                .map(|key| moved(key))
                .collect(),
            pointer,
        }
    }

    /// Every key, value name and value, in the order they are written.
    pub fn entries(&self) -> &[(String, String, String)] {
        &self.entries
    }

    /// Every key that [`Self::remove`] deletes whole.
    pub fn keys_owned(&self) -> &[String] {
        &self.keys_owned
    }

    /// The one value written into a key that belongs to Windows.
    pub fn pointer(&self) -> (&str, &str) {
        (&self.pointer.0, &self.pointer.1)
    }
}

/// What to call the executable when its real path cannot be read.
const EXECUTABLE_FILE_NAME: &str = "wixen-mail.exe";

/// How every key in this registration begins.
const USER_HIVE_PREFIX: &str = r"HKCU\";

// ── Writing and removing ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_registry {
    use super::{Registration, USER_HIVE_PREFIX};
    use crate::common::{Error, Result};

    /// The current user's own part of the registry.
    ///
    /// Documented as `0x80000001` read as a signed 32 bit value and then
    /// widened, so the cast goes through `i32`. Casting straight from `u32`
    /// would zero-extend and hand the API a handle it has never heard of.
    const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;

    /// The read worked.
    const ERROR_SUCCESS: i32 = 0;
    /// The value is not there.
    const ERROR_FILE_NOT_FOUND: i32 = 2;
    /// The key is not there.
    const ERROR_PATH_NOT_FOUND: i32 = 3;
    /// Permission to make or change the key was refused.
    const ERROR_ACCESS_DENIED: i32 = 5;

    /// Enough to make a key, write to it and delete it again.
    const KEY_WRITE: u32 = 0x0002_0006;
    /// Enough to ask a key what is inside it.
    const KEY_READ: u32 = 0x0002_0019;
    /// The key survives a restart, which is what a registration has to do.
    const REG_OPTION_NON_VOLATILE: u32 = 0;
    /// A plain string value.
    const REG_SZ: u32 = 1;

    /// Something in the file associations changed.
    const SHCNE_ASSOCCHANGED: i32 = 0x0800_0000;
    /// The two arguments to that message are nothing, which is how it is sent.
    const SHCNF_IDLIST: u32 = 0;

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegCreateKeyExW(
            hkey: isize,
            sub_key: *const u16,
            reserved: u32,
            class: *const u16,
            options: u32,
            desired: u32,
            security: *const core::ffi::c_void,
            out_key: *mut isize,
            disposition: *mut u32,
        ) -> i32;
        fn RegOpenKeyExW(
            hkey: isize,
            sub_key: *const u16,
            options: u32,
            desired: u32,
            out_key: *mut isize,
        ) -> i32;
        fn RegSetValueExW(
            hkey: isize,
            value_name: *const u16,
            reserved: u32,
            value_type: u32,
            data: *const u8,
            size: u32,
        ) -> i32;
        fn RegQueryInfoKeyW(
            hkey: isize,
            class: *mut u16,
            class_size: *mut u32,
            reserved: *mut u32,
            sub_keys: *mut u32,
            longest_sub_key: *mut u32,
            longest_class: *mut u32,
            values: *mut u32,
            longest_value_name: *mut u32,
            longest_value: *mut u32,
            security_size: *mut u32,
            last_written: *mut core::ffi::c_void,
        ) -> i32;
        fn RegDeleteTreeW(hkey: isize, sub_key: *const u16) -> i32;
        fn RegDeleteKeyW(hkey: isize, sub_key: *const u16) -> i32;
        fn RegDeleteValueW(hkey: isize, value_name: *const u16) -> i32;
        fn RegCloseKey(hkey: isize) -> i32;
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHChangeNotify(
            event: i32,
            flags: u32,
            first: *const core::ffi::c_void,
            second: *const core::ffi::c_void,
        );
    }

    /// Text in the shape Windows reads: UTF-16, ending in a null.
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// The part of a key path below the user's hive.
    ///
    /// Refuses anything else rather than guessing. Every key in a
    /// [`Registration`] is under `HKCU\`, a test says so, and a guessed hive
    /// here would write to the machine.
    fn below_the_user_hive(key_path: &str) -> Result<&str> {
        key_path
            .strip_prefix(USER_HIVE_PREFIX)
            .filter(|rest| !rest.is_empty())
            .ok_or_else(|| {
                Error::Other(format!(
                    "{key_path} is not under the current user, so it was not written"
                ))
            })
    }

    /// A key handle that closes itself, however the function leaves.
    ///
    /// Every path out of a write is an early return on an error, and a handle
    /// leaked on each one is a key Windows will not let anything else delete
    /// until this program exits.
    struct OpenKey(isize);

    impl Drop for OpenKey {
        fn drop(&mut self) {
            // Safe: the handle came from a call that reported success, and
            // this runs exactly once because `OpenKey` is never copied.
            unsafe { RegCloseKey(self.0) };
        }
    }

    /// Make a key if it is not there, and open it for writing.
    fn make_and_open(key_path: &str) -> Result<OpenKey> {
        let sub_key = wide(below_the_user_hive(key_path)?);
        let mut handle: isize = 0;
        // Safe: the name is null-terminated UTF-16 alive for the whole call,
        // the two pointers that may be null are documented to accept it, and
        // the handle is only read when the call reports success.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                0,
                std::ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                std::ptr::null(),
                &raw mut handle,
                std::ptr::null_mut(),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(complaint("make", key_path, status));
        }
        Ok(OpenKey(handle))
    }

    /// Put one string into one key.
    pub fn write_one(key_path: &str, value_name: &str, value: &str) -> Result<()> {
        let key = make_and_open(key_path)?;
        let name = wide(value_name);
        let data = wide(value);
        // Bytes, including the terminator, which is what this API counts. A
        // length that left the terminator out gives a value the shell reads
        // without one, and what follows it in the registry is then read as
        // part of the string.
        let size = (data.len() * size_of::<u16>()) as u32;
        // Safe: both buffers are null-terminated UTF-16 kept alive for the
        // call, `size` is exactly how long `data` is, and the nameless value
        // is written by passing an empty name, which is how this API spells
        // it.
        let status = unsafe {
            RegSetValueExW(
                key.0,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast::<u8>(),
                size,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(complaint("write to", key_path, status));
        }
        Ok(())
    }

    /// Whether a key exists and has nothing at all inside it.
    ///
    /// A key that is not there counts as empty, so removing a registration
    /// that was never fully written can still finish.
    fn is_empty(key_path: &str) -> Result<bool> {
        let sub_key = wide(below_the_user_hive(key_path)?);
        let mut handle: isize = 0;
        // Safe: as `make_and_open`, and for reading only.
        let opened = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                0,
                KEY_READ,
                &raw mut handle,
            )
        };
        if opened == ERROR_FILE_NOT_FOUND || opened == ERROR_PATH_NOT_FOUND {
            return Ok(true);
        }
        if opened != ERROR_SUCCESS {
            return Err(complaint("look inside", key_path, opened));
        }
        let key = OpenKey(handle);

        let mut sub_keys: u32 = 0;
        let mut values: u32 = 0;
        // Safe: every pointer this call may be given is either one of the two
        // locals below, alive for the call, or null, which this API documents
        // as "do not tell me".
        let asked = unsafe {
            RegQueryInfoKeyW(
                key.0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &raw mut sub_keys,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &raw mut values,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if asked != ERROR_SUCCESS {
            return Err(complaint("count what is inside", key_path, asked));
        }
        Ok(sub_keys == 0 && values == 0)
    }

    /// Delete a key and everything below it.
    ///
    /// A key that is not there is not a failure. Taking out a registration
    /// that was only half written has to be able to finish, and so does
    /// taking one out twice.
    pub fn remove_tree(key_path: &str) -> Result<()> {
        let sub_key = wide(below_the_user_hive(key_path)?);
        // Safe: the name is null-terminated UTF-16 alive for the whole call.
        let status = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, sub_key.as_ptr()) };
        match status {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => {}
            _ => return Err(complaint("remove", key_path, status)),
        }
        // `RegDeleteTreeW` empties the key it is given and leaves the key
        // itself, which is documented and is not what is wanted here: an
        // empty `WixenMail.Url.mailto` key still appears in an Open With list.
        let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, sub_key.as_ptr()) };
        match status {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => Ok(()),
            _ => Err(complaint("remove", key_path, status)),
        }
    }

    /// Delete a key, but only if there is nothing left inside it.
    ///
    /// For the keys made on the way down to the capability key. Something else
    /// may one day keep its own settings under `Software\Wixen`, and taking
    /// that away because this program was uninstalled would be this program
    /// deleting somebody else's data.
    pub fn remove_if_empty(key_path: &str) -> Result<()> {
        if is_empty(key_path)? {
            let sub_key = wide(below_the_user_hive(key_path)?);
            // Safe: as `remove_tree`. This form deletes nothing that has a
            // subkey, which is the second guard under the check above.
            let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, sub_key.as_ptr()) };
            match status {
                ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => {}
                _ => return Err(complaint("remove", key_path, status)),
            }
        }
        Ok(())
    }

    /// Take one named value out of a key that belongs to Windows.
    ///
    /// The value, never the key. `RegisteredApplications` holds every other
    /// program's entry, and deleting it would unregister all of them.
    pub fn remove_value(key_path: &str, value_name: &str) -> Result<()> {
        let sub_key = wide(below_the_user_hive(key_path)?);
        let mut handle: isize = 0;
        // Safe: as `make_and_open`.
        let opened = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                0,
                KEY_WRITE,
                &raw mut handle,
            )
        };
        if opened == ERROR_FILE_NOT_FOUND || opened == ERROR_PATH_NOT_FOUND {
            return Ok(());
        }
        if opened != ERROR_SUCCESS {
            return Err(complaint("open", key_path, opened));
        }
        let key = OpenKey(handle);

        let name = wide(value_name);
        // Safe: the name is null-terminated UTF-16 alive for the whole call.
        let status = unsafe { RegDeleteValueW(key.0, name.as_ptr()) };
        match status {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => Ok(()),
            _ => Err(complaint("take a value out of", key_path, status)),
        }
    }

    /// Tell the shell the associations changed.
    ///
    /// Explorer keeps its own picture of what opens what, and a program that
    /// registered itself and said nothing can sit unlisted until the next
    /// sign in. Nothing is reported back: this returns no status, and there is
    /// nothing to do about it if it does not arrive.
    pub fn say_the_associations_changed() {
        // Safe: both arguments are documented as ignored for this event when
        // the flags say the arguments are identifier lists and both are null.
        unsafe {
            SHChangeNotify(
                SHCNE_ASSOCCHANGED,
                SHCNF_IDLIST,
                std::ptr::null(),
                std::ptr::null(),
            );
        }
    }

    /// What went wrong, in words somebody can act on.
    ///
    /// Being refused is the one status with a plain cause and a next step, so
    /// it gets a sentence. Everything else keeps its number, because an
    /// unexplained fault still has to be diagnosable from a log.
    fn complaint(doing: &str, key_path: &str, status: i32) -> Error {
        if status == ERROR_ACCESS_DENIED {
            return Error::Other(format!(
                "Windows would not let this program {doing} {key_path}, \
                 because it does not have permission"
            ));
        }
        Error::Other(format!(
            "Windows would not let this program {doing} {key_path}, \
             and reported error {status}"
        ))
    }

    /// Write the whole registration.
    pub fn write(registration: &Registration) -> Result<()> {
        for (key, name, value) in registration.entries() {
            write_one(key, name, value)?;
        }
        say_the_associations_changed();
        Ok(())
    }

    /// Take the whole registration away.
    ///
    /// Everything is attempted even when something fails, and the first
    /// complaint is the one returned. Stopping at the first failure is how an
    /// uninstall leaves three keys behind that still claim `.ics` files, with
    /// nothing on the machine able to open them.
    pub fn remove(registration: &Registration) -> Result<()> {
        let (pointer_key, pointer_name) = registration.pointer();
        let mut first_complaint = remove_value(pointer_key, pointer_name).err();

        for key in registration.keys_owned() {
            if let Err(why) = remove_tree(key) {
                first_complaint = first_complaint.or(Some(why));
            }
        }
        for key in registration.keys_to_tidy() {
            if let Err(why) = remove_if_empty(key) {
                first_complaint = first_complaint.or(Some(why));
            }
        }

        say_the_associations_changed();
        match first_complaint {
            Some(why) => Err(why),
            None => Ok(()),
        }
    }

    /// Whether every entry is in the registry, holding what it should.
    ///
    /// Every entry rather than only the pointer Windows follows, because the
    /// open commands carry the path this program was installed to. After an
    /// upgrade into a different folder the pointer is still perfectly correct
    /// and every command runs an executable that is no longer there, which
    /// shows as Windows saying the file has no program associated with it.
    ///
    /// Compared without case, the way the registry compares: a path Windows
    /// handed back with different capitalisation is the same path, and
    /// reporting it as a difference would rewrite the whole registration on
    /// every start.
    pub fn is_written(registration: &Registration) -> bool {
        registration.entries().iter().all(|(key, name, value)| {
            read_one(key, name).is_some_and(|found| found.eq_ignore_ascii_case(value))
        })
    }

    /// Read one string back, for checking what was written.
    fn read_one(key_path: &str, value_name: &str) -> Option<String> {
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
        /// REG_SZ and REG_EXPAND_SZ.
        const RRF_RT_ANY_STRING: u32 = 0x0000_0002 | 0x0000_0004;

        let sub_key = wide(below_the_user_hive(key_path).ok()?);
        let value = wide(value_name);
        let mut room: u32 = 512 * size_of::<u16>() as u32;
        let mut buffer: Vec<u16> = vec![0; 512];
        // Safe: both names are null-terminated UTF-16 alive for the call, and
        // `room` says exactly how many bytes `buffer` holds. The buffer is
        // only read when the call reports success.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                value.as_ptr(),
                RRF_RT_ANY_STRING,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &raw mut room,
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let end = buffer
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..end]))
    }
}

impl Registration {
    /// Keys removal deletes only when they turn out to be empty.
    pub fn keys_to_tidy(&self) -> &[String] {
        &self.keys_to_tidy
    }

    /// Write every key, so Windows offers this program when somebody chooses.
    ///
    /// Does not make anything the default and cannot. It puts this program in
    /// the list, and the person chooses from that list in the Windows Settings
    /// page [`crate::service::default_apps::open_windows_default_apps_page`]
    /// opens. Anything on screen beside a control that calls this has to say
    /// so, or somebody presses it and waits for a change that was never going
    /// to happen here.
    pub fn write(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            windows_registry::write(self)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(not_windows())
        }
    }

    /// Take every key away, leaving nothing claiming a file type.
    ///
    /// Run by the uninstaller and by anybody turning this off. Everything is
    /// attempted even when part of it fails, because stopping at the first
    /// failure leaves keys behind that still claim `.ics` and `.vcf` with
    /// nothing on the machine able to open them.
    pub fn remove(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            windows_registry::remove(self)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(not_windows())
        }
    }

    /// Whether this registration is currently in the registry.
    ///
    /// Reads the one entry Windows finds on its own and checks it names the
    /// key this registration would write. That is the entry whose absence
    /// makes every other one invisible, so it is the honest thing to ask
    /// about. `false` on anything that is not Windows.
    pub fn is_written(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            windows_registry::is_written(self)
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

/// The honest answer where there is no Windows registry.
#[cfg(not(target_os = "windows"))]
fn not_windows() -> crate::common::Error {
    crate::common::Error::Other(
        "Registering a program to open mail, calendars and contact cards is a Windows \
         feature, and this is not Windows."
            .to_string(),
    )
}

/// Put this program in the Windows list of programs a default can be chosen
/// from, if it is not already there and current.
///
/// Called on every start, which is why it checks first. Writing unconditionally
/// would be two dozen registry writes and a message telling Explorer to reread
/// its associations, every time anybody opened their mail, for no change.
///
/// Answers whether it wrote anything, so the log can say so once rather than
/// on every start.
///
/// # Why a start, rather than the installer
///
/// The keys are per-user and this program is the thing that knows where it is
/// installed. An installer writes them for the person who ran the installer,
/// which on a shared computer is one person out of several, and it writes them
/// once, so an upgrade into a different folder leaves every open command
/// naming an executable that is no longer there. Checking on each start covers
/// both, and costs one registry read per entry when nothing has changed.
pub fn register_with_windows() -> Result<bool> {
    let registration = Registration::for_this_program();
    if registration.is_written() {
        return Ok(false);
    }
    registration.write().map(|()| true)
}

/// Take this program out of that list, leaving nothing claiming a file type.
///
/// Run when everything this installation stored is being erased, which is what
/// the uninstaller does. Leaving the keys would tell Windows this program
/// opens `.ics` and `.vcf` files after it has been removed from the machine.
pub fn remove_registration() -> Result<()> {
    Registration::for_this_program().remove()
}

/// Whether this program is currently in that list, with the right paths.
pub fn is_registered() -> bool {
    Registration::for_this_program().is_written()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::default_apps::Association;

    /// An executable path with a space in it, which is where this really is.
    const INSTALLED: &str = r"C:\Program Files\Wixen Mail\wixen-mail.exe";

    fn value_at(registration: &Registration, key: &str, name: &str) -> Option<String> {
        registration
            .entries()
            .iter()
            .find(|(k, n, _)| k.eq_ignore_ascii_case(key) && n.eq_ignore_ascii_case(name))
            .map(|(_, _, value)| value.clone())
    }

    #[test]
    fn test_the_pointer_windows_follows_is_written_last() {
        // `RegisteredApplications` is the only entry Windows finds without
        // being told where to look. Written before the key it names, there is
        // a window in which Windows follows it to a key that is half there
        // and lists a program that cannot open anything. Short, and real: the
        // shell is told the associations changed the moment this finishes.
        let registration = Registration::for_program_at(INSTALLED);
        let entries = registration.entries();

        let (last_key, last_name, _) = entries.last().expect("no entries at all");
        assert_eq!(
            (last_key.as_str(), last_name.as_str()),
            registration.pointer(),
            "something is written after the pointer Windows follows"
        );
        assert_eq!(
            entries
                .iter()
                .filter(|(key, name, _)| (key.as_str(), name.as_str()) == registration.pointer())
                .count(),
            1,
            "the pointer is written more than once"
        );
    }

    #[test]
    fn test_moving_the_pointer_last_did_not_lose_or_duplicate_anything() {
        // The reordering is a sort, and a sort that got its key wrong could
        // silently drop or reorder the rest. What must hold is that the same
        // entries are there, whatever order they came in.
        let registration = Registration::for_program_at(INSTALLED);
        let mut written: Vec<_> = registration.entries().to_vec();
        let mut expected = crate::service::default_apps::capability_entries_for(INSTALLED);
        written.sort();
        expected.sort();

        assert_eq!(written, expected, "the reordering changed the registration");
    }

    #[test]
    fn test_every_key_written_is_under_the_current_user() {
        // The whole reason this needs no administrator rights. A single key
        // under the machine turns a settings checkbox into a control that
        // fails for every standard user, and the failure arrives as "access
        // denied" with no explanation.
        for (key, name, _) in Registration::for_program_at(INSTALLED).entries() {
            assert!(
                key.starts_with(r"HKCU\"),
                "{key}\\{name} is not under the current user"
            );
        }
    }

    #[test]
    fn test_removal_takes_away_every_key_that_writing_put_in() {
        // The half nobody runs. Everybody installs, few uninstall, so a
        // removal list written from memory drifts from the registration and
        // nothing notices. Derived here from the entries themselves: every
        // key written must sit under a key removal deletes, apart from the one
        // value in a key belonging to Windows.
        let registration = Registration::for_program_at(INSTALLED);
        let (pointer_key, _) = registration.pointer();

        for (key, name, _) in registration.entries() {
            if key == pointer_key {
                continue;
            }
            assert!(
                registration
                    .keys_owned()
                    .iter()
                    .any(|owned| key == owned || key.starts_with(&format!("{owned}\\"))),
                "{key}\\{name} would be left behind claiming something"
            );
        }
    }

    #[test]
    fn test_removal_never_deletes_a_key_that_belongs_to_windows() {
        // Two keys here hold every other program's registrations:
        // `RegisteredApplications` and `Software\Classes`. Deleting either
        // would unregister every program on the machine, and the uninstall
        // would report success.
        let registration = Registration::for_program_at(INSTALLED);

        for shared in [REGISTERED_APPLICATIONS_KEY, CLASSES_KEY] {
            assert!(
                !registration
                    .keys_owned()
                    .iter()
                    .any(|owned| owned == shared),
                "removal would delete {shared}, which belongs to Windows"
            );
            assert!(
                !registration.keys_to_tidy().iter().any(|key| key == shared),
                "removal would tidy away {shared}, which belongs to Windows"
            );
        }
        assert_eq!(registration.pointer().0, REGISTERED_APPLICATIONS_KEY);
    }

    #[test]
    fn test_every_identifier_this_program_claims_is_deleted_by_name() {
        // Every arm of the association list, because a claim left behind
        // points `.ics` or `mailto:` at an executable that is no longer
        // there, and the failure a person sees is Windows saying the file has
        // no program associated with it.
        let registration = Registration::for_program_at(INSTALLED);

        let mut claimed = 0;
        for kind in DefaultKind::ALL {
            for association in kind.associations() {
                claimed += 1;
                let key = format!(r"HKCU\Software\Classes\{}", association.our_prog_id());
                assert!(
                    registration.keys_owned().contains(&key),
                    "{key} is registered and never removed"
                );
            }
        }
        assert_eq!(claimed, 4, "the four slots Windows has for this program");
    }

    #[test]
    fn test_the_keys_made_on_the_way_down_are_tidied_only_when_they_are_empty() {
        // `Software\Wixen` may one day hold something a sibling program keeps.
        // Deleting it outright because Wixen Mail was uninstalled would be
        // this program removing somebody else's settings, so it is on the
        // tidy list, which checks first, rather than on the owned list, which
        // does not.
        let registration = Registration::for_program_at(INSTALLED);

        assert_eq!(
            registration.keys_to_tidy(),
            [r"HKCU\Software\Wixen\Wixen Mail", r"HKCU\Software\Wixen"],
            "the containers are tidied deepest first, or a parent blocks its child"
        );
        for container in registration.keys_to_tidy() {
            assert!(
                !registration.keys_owned().contains(container),
                "{container} is deleted whole rather than only when empty"
            );
        }
    }

    #[test]
    fn test_a_scratch_registration_touches_none_of_the_real_keys() {
        // The guard on the test isolation itself. If this were wrong, every
        // registry test below would be editing the registration the person
        // running the tests is actually using, and the damage would look
        // exactly like a passing test run.
        let scratch = Registration::in_a_scratch_key(INSTALLED, r"Software\Wixen\ScratchForTests");
        let real = Registration::for_program_at(INSTALLED);

        let real_keys: Vec<&String> = real
            .entries()
            .iter()
            .map(|(key, _, _)| key)
            .chain(real.keys_owned())
            .chain(real.keys_to_tidy())
            .collect();

        for key in scratch
            .entries()
            .iter()
            .map(|(key, _, _)| key)
            .chain(scratch.keys_owned())
            .chain(scratch.keys_to_tidy())
        {
            assert!(
                key.starts_with(r"HKCU\Software\Wixen\ScratchForTests\"),
                "a scratch key escaped the scratch area: {key}"
            );
            assert!(
                !real_keys.contains(&key),
                "a scratch key is a real key: {key}"
            );
        }
        assert_ne!(scratch, real);
    }

    #[test]
    fn test_the_one_value_that_is_itself_a_key_path_moves_with_the_keys() {
        // `RegisteredApplications` holds a path to the capability key rather
        // than the capability itself. Moving the key it sits in without
        // moving the path inside it leaves a scratch registration pointing at
        // the real capability key, so a test that wrote and then removed
        // would remove the real one.
        let scratch = Registration::in_a_scratch_key(INSTALLED, r"Software\Wixen\ScratchForTests");
        let (pointer_key, pointer_name) = scratch.pointer();

        let points_at = value_at(&scratch, pointer_key, pointer_name).expect("no pointer at all");
        assert_eq!(
            format!(r"HKCU\{points_at}"),
            r"HKCU\Software\Wixen\ScratchForTests\Software\Wixen\Wixen Mail\Capabilities",
            "the pointer names a key outside the scratch area"
        );
        assert!(
            scratch
                .entries()
                .iter()
                .any(|(key, _, _)| *key == format!(r"HKCU\{points_at}")),
            "the pointer names a key nothing writes"
        );
    }

    #[test]
    fn test_the_real_pointer_names_the_real_capability_key() {
        // The same check for the registration that ships. A pointer naming a
        // key nothing writes is a dangling one: the program never appears in
        // the Settings list, and nothing anywhere reports a fault.
        let registration = Registration::for_program_at(INSTALLED);
        let (pointer_key, pointer_name) = registration.pointer();

        let points_at = value_at(&registration, pointer_key, pointer_name).expect("no pointer");
        assert_eq!(format!(r"HKCU\{points_at}"), CAPABILITY_KEY);
        assert!(
            registration
                .keys_owned()
                .contains(&CAPABILITY_KEY.to_string()),
            "the capability key is written and never removed"
        );
    }

    #[test]
    fn test_the_open_command_is_the_one_windows_runs_for_each_kind_of_thing() {
        // Every arm. Windows appends what it was given to this command, so a
        // wrong one is a program that opens and ignores the link or the file
        // that started it, which is worse than not being registered: the
        // person's mail link opens a blank window and nothing says why.
        let registration = Registration::for_program_at(INSTALLED);

        for association in [
            Association::Protocol("mailto"),
            Association::Protocol("webcal"),
            Association::FileType(".ics"),
            Association::FileType(".vcf"),
        ] {
            let key = format!(
                r"HKCU\Software\Classes\{}\shell\open\command",
                association.our_prog_id()
            );
            let command = value_at(&registration, &key, "")
                .unwrap_or_else(|| panic!("{association:?} has no open command"));

            assert_eq!(
                command,
                format!(r#""{INSTALLED}" "%1""#),
                "{association:?} is opened by the wrong command"
            );
        }
    }

    #[test]
    fn test_the_registration_for_the_running_program_names_the_running_program() {
        // The reachability check for the form a settings screen calls. It
        // finds the executable itself, and a version of it that produced no
        // entries, or entries naming nothing, would leave every other test
        // here green and the feature dead.
        let registration = Registration::for_this_program();

        assert!(!registration.entries().is_empty(), "no entries at all");
        let (pointer_key, pointer_name) = registration.pointer();
        assert!(
            value_at(&registration, pointer_key, pointer_name).is_some(),
            "the running program did not register itself"
        );

        let running = std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| EXECUTABLE_FILE_NAME.to_string());
        let command = value_at(
            &registration,
            r"HKCU\Software\Classes\WixenMail.Url.mailto\shell\open\command",
            "",
        )
        .expect("no open command");
        assert!(
            command.contains(&running),
            "the command does not run this program: {command}"
        );
    }
}

#[cfg(all(test, target_os = "windows"))]
mod windows_tests {
    use super::*;

    /// Where a test writes, which is nowhere Windows reads.
    ///
    /// Below the same vendor key the real registration uses, so anything a
    /// crashed run leaves behind is obvious and in one place, and it is a
    /// scratch key so nothing in it claims a file type or a scheme even while
    /// it exists.
    const SCRATCH: &str = r"Software\Wixen\RegistrationTests";

    /// A scratch registration that removes itself, however the test leaves.
    ///
    /// A test that panics half way through a write would otherwise leave keys
    /// in Pratik's registry, and the next run would start from a state nobody
    /// chose.
    struct Scratch {
        registration: Registration,
        /// The one key everything this test wrote sits under.
        ///
        /// Removing the registration is not enough to leave nothing behind,
        /// and that is correct rather than a fault: `Software\Classes` and
        /// `Software\RegisteredApplications` are Windows' own keys in the
        /// real registry, so removal is right to leave them. Inside a scratch
        /// tree they are this code's own, and the tidying up has to take the
        /// whole tree rather than asking removal to do something it must not
        /// do in production.
        root: String,
    }

    impl Scratch {
        fn new(named: &str) -> Self {
            let scratch = Self {
                registration: Registration::in_a_scratch_key(
                    r"C:\Program Files\Wixen Mail\wixen-mail.exe",
                    &format!("{SCRATCH}\\{named}"),
                ),
                root: format!(r"HKCU\{SCRATCH}\{named}"),
            };
            // Whatever an earlier run left, so a test starts from nothing.
            scratch.tidy_up();
            scratch
        }

        fn tidy_up(&self) {
            let _ = super::windows_registry::remove_tree(&self.root);
            let _ = super::windows_registry::remove_if_empty(&format!(r"HKCU\{SCRATCH}"));
            let _ = super::windows_registry::remove_if_empty(r"HKCU\Software\Wixen");
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            self.tidy_up();
        }
    }

    /// Ask Windows itself what is at a key, through its own reader.
    ///
    /// `reg.exe` knows nothing about this code, so the two agreeing is worth
    /// something that a second call into the same functions would not be.
    fn what_reg_exe_sees(key: &str, value_name: &str) -> Option<String> {
        let asked = if value_name.is_empty() {
            vec!["query".to_string(), key.to_string(), "/ve".to_string()]
        } else {
            vec![
                "query".to_string(),
                key.to_string(),
                "/v".to_string(),
                value_name.to_string(),
            ]
        };
        let out = std::process::Command::new("reg")
            .args(&asked)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|line| line.split("REG_SZ").nth(1))
            .map(|text| text.trim().to_string())
    }

    #[test]
    fn test_writing_really_puts_every_entry_where_it_says_it_does() {
        // The measurement. Every other test in this file compares strings this
        // code produced against strings this code produced, so a `write` that
        // silently did nothing, or wrote to the wrong hive, would leave all of
        // them green. Read back through `reg.exe`, which is Windows' own
        // reader and knows nothing about any of this.
        let scratch = Scratch::new("writing");
        scratch.registration.write().expect("the write failed");

        for (key, name, value) in scratch.registration.entries() {
            assert_eq!(
                what_reg_exe_sees(key, name).as_deref(),
                Some(value.as_str()),
                "{key}\\{name} did not reach the registry"
            );
        }
    }

    #[test]
    fn test_the_reader_would_have_noticed_a_key_that_was_never_written() {
        // Before believing the test above, prove its reader can come back
        // empty. A `what_reg_exe_sees` that answered with something for every
        // key would make that test pass against a `write` that did nothing.
        assert_eq!(
            what_reg_exe_sees(
                r"HKCU\Software\Wixen\RegistrationTests\NoSuchKeyWasEverWritten",
                "AnyValue"
            ),
            None
        );
    }

    #[test]
    fn test_removing_leaves_nothing_behind_claiming_a_file_type() {
        // The promise an uninstall makes. A key left behind still tells
        // Windows this program opens `.ics` files, so a person who removed it
        // gets an error naming a program that is no longer on the machine.
        let scratch = Scratch::new("removing");
        scratch.registration.write().expect("the write failed");
        scratch.registration.remove().expect("the removal failed");

        for (key, name, _) in scratch.registration.entries() {
            assert_eq!(
                what_reg_exe_sees(key, name),
                None,
                "{key}\\{name} was left behind"
            );
        }
        for key in scratch.registration.keys_owned() {
            // The key itself, not one of its values. A key emptied but left in
            // place still shows in an Open With list, so the question is
            // whether `reg query` can find it at all.
            let found = std::process::Command::new("reg")
                .args(["query", key])
                .output()
                .expect("reg.exe would not run, so this checked nothing");
            assert!(!found.status.success(), "{key} survived the removal");
        }
    }

    #[test]
    fn test_nothing_left_behind_after_a_removal_claims_anything() {
        // Not only the entries. `RegCreateKeyExW` makes every level of a path
        // that is not there, so writing one value five levels down creates
        // five keys, and a removal that took only the values away would leave
        // the shape of the registration behind.
        //
        // What survives, and must: `Software`, `Software\Classes` and
        // `Software\RegisteredApplications`. In a scratch tree this code made
        // them; in the real registry they are Windows' own and hold every
        // other program's registrations, so removal is right not to touch
        // them. The promise is not "no keys survive", it is that nothing
        // surviving claims a file type, a scheme, or a place in the Settings
        // list. That is what is checked, by reading the whole tree back
        // through Windows' own reader and looking for any mention of this
        // program.
        let scratch = Scratch::new("tidying");
        scratch.registration.write().expect("the write failed");
        scratch.registration.remove().expect("the removal failed");

        for key in scratch.registration.keys_to_tidy() {
            assert!(
                what_reg_exe_sees(key, "").is_none(),
                "{key} was left behind"
            );
        }

        let scratch_root = format!(r"HKCU\{SCRATCH}\tidying");
        let whole_tree = std::process::Command::new("reg")
            .args(["query", &scratch_root, "/s"])
            .output()
            .expect("reg.exe would not run, so this checked nothing");
        let printed = String::from_utf8_lossy(&whole_tree.stdout);

        for claim in ["WixenMail", "Capabilities", "Wixen Mail", "wixen-mail.exe"] {
            assert!(
                !printed.contains(claim),
                "{claim:?} survived the removal:\n{printed}"
            );
        }
        for shared in ["Classes", "RegisteredApplications"] {
            assert!(
                printed.contains(shared),
                "{shared} was deleted, and in the real registry that key holds \
                 every other program's registrations:\n{printed}"
            );
        }
    }

    #[test]
    fn test_removing_something_that_was_never_written_is_not_a_failure() {
        // An uninstall runs on a machine where this was never switched on, and
        // one that reported a failure would put an error in front of somebody
        // in the middle of somebody else's progress bar. Running it twice has
        // to be quiet for the same reason.
        let scratch = Scratch::new("never-written");

        scratch
            .registration
            .remove()
            .expect("removing what was never there reported a failure");
        scratch.registration.write().expect("the write failed");
        scratch.registration.remove().expect("the first removal");
        scratch
            .registration
            .remove()
            .expect("removing twice reported a failure");
    }

    #[test]
    fn test_a_registration_says_whether_it_is_there() {
        // What a settings screen puts beside its control. Answering wrongly
        // shows somebody a button offering to do what is already done, or
        // hides the one they need.
        let scratch = Scratch::new("is-written");

        assert!(
            !scratch.registration.is_written(),
            "an unwritten registration reported itself as written"
        );
        scratch.registration.write().expect("the write failed");
        assert!(
            scratch.registration.is_written(),
            "a written registration reported itself as missing"
        );
        scratch.registration.remove().expect("the removal failed");
        assert!(
            !scratch.registration.is_written(),
            "a removed registration reported itself as written"
        );
    }

    #[test]
    fn test_the_real_registration_is_not_touched_by_any_of_this() {
        // The guard that matters most in this file. These tests write to a
        // real registry, on Pratik's machine, and the only thing standing
        // between them and the registration he is actually using is the
        // scratch key. Checked rather than trusted.
        let scratch = Scratch::new("isolation");
        let real_before = Registration::for_this_program().is_written();

        scratch.registration.write().expect("the write failed");
        assert_eq!(
            Registration::for_this_program().is_written(),
            real_before,
            "writing a scratch registration changed the real one"
        );

        scratch.registration.remove().expect("the removal failed");
        assert_eq!(
            Registration::for_this_program().is_written(),
            real_before,
            "removing a scratch registration changed the real one"
        );
    }

    #[test]
    fn test_a_registration_pointing_at_the_wrong_executable_is_not_reported_as_written() {
        // What makes it safe to check on every start rather than write on
        // every start. An upgrade into a different folder leaves the pointer
        // Windows follows perfectly correct and every open command naming an
        // executable that is no longer there, and the person sees Windows say
        // their calendar file has no program associated with it.
        let scratch = Scratch::new("moved-executable");
        let older = Registration::in_a_scratch_key(
            r"C:\Program Files\Wixen Mail 0.37\wixen-mail.exe",
            &format!("{SCRATCH}\\moved-executable"),
        );

        older.write().expect("the write failed");
        assert!(older.is_written(), "what was just written reads as missing");
        assert!(
            !scratch.registration.is_written(),
            "a registration naming a different executable read as current"
        );

        scratch.registration.write().expect("the rewrite failed");
        assert!(
            scratch.registration.is_written(),
            "rewriting did not bring the registration up to date"
        );
    }

    #[test]
    fn test_writing_into_a_key_that_already_holds_other_values_keeps_them() {
        // `RegisteredApplications` is a key belonging to Windows that already
        // holds an entry for every other program on the machine, and this
        // registration adds one value to it. So the write has to open that key
        // and add to it, never replace it.
        //
        // This is written down because it has actually happened here, by hand
        // rather than through this code: a PowerShell probe used
        // `New-Item -Force` on that key, which deletes and recreates it, and
        // fifty entries belonging to Zoom, Firefox, 1Password and forty-seven
        // Store applications went with it. They were restored from a reading
        // taken minutes earlier. `RegCreateKeyExW` opens an existing key
        // rather than replacing it, which is why the code was never the
        // problem, and this pins that down so it stays true.
        let scratch = Scratch::new("neighbours");
        let (pointer_key, pointer_name) = {
            let (key, name) = scratch.registration.pointer();
            (key.to_string(), name.to_string())
        };

        super::windows_registry::write_one(&pointer_key, "SomebodyElse", "Their\\Capabilities")
            .expect("could not set up a neighbouring value");
        scratch.registration.write().expect("the write failed");

        assert_eq!(
            what_reg_exe_sees(&pointer_key, "SomebodyElse").as_deref(),
            Some("Their\\Capabilities"),
            "registering wiped another program's entry"
        );
        assert!(
            what_reg_exe_sees(&pointer_key, &pointer_name).is_some(),
            "this program's own entry did not reach the key"
        );

        scratch.registration.remove().expect("the removal failed");
        assert_eq!(
            what_reg_exe_sees(&pointer_key, "SomebodyElse").as_deref(),
            Some("Their\\Capabilities"),
            "removing took another program's entry with it"
        );
    }

    /// The real registration, on the real keys, run by hand.
    ///
    /// ```text
    /// cargo test --lib default_apps_registration -- --ignored --nocapture
    /// ```
    ///
    /// Ignored by default, and that is the whole design of it. Every other
    /// test in this file writes to a scratch key that Windows never reads.
    /// This one writes the keys the shell really resolves, on whichever
    /// machine it runs, naming the test binary as the program that opens
    /// mail. Left behind, `mailto:` would point at an executable in `target`
    /// that the next build replaces, so a link would open nothing.
    ///
    /// It is here because the two functions it calls are the ones `main`
    /// calls, and nothing else exercises them against the keys that matter.
    /// It registers, reads back through `reg.exe`, removes, and checks the
    /// machine is as it was. The guard removes the registration however the
    /// test leaves, including on a panic.
    ///
    /// Run it on a machine where Wixen Mail is not installed and chosen. It
    /// refuses to start if a real registration is already there, rather than
    /// taking somebody's working installation away and putting it back
    /// pointing at a test binary.
    #[test]
    #[ignore = "writes the real registration on this machine; run by hand"]
    fn test_the_registration_main_writes_reaches_the_keys_windows_reads() {
        struct AlwaysRemove;
        impl Drop for AlwaysRemove {
            fn drop(&mut self) {
                let _ = super::remove_registration();
            }
        }

        assert!(
            !super::is_registered(),
            "Wixen Mail is already registered on this machine. This check would \
             take that away and put it back naming the test binary, so it \
             refuses instead. Nothing has been changed."
        );

        let registration = Registration::for_this_program();
        let _guard = AlwaysRemove;

        assert!(
            super::register_with_windows().expect("registering failed"),
            "registering reported that there was nothing to do"
        );
        assert!(
            !super::register_with_windows().expect("registering twice failed"),
            "a second registration wrote everything again instead of nothing"
        );

        for (key, name, value) in registration.entries() {
            assert_eq!(
                what_reg_exe_sees(key, name).as_deref(),
                Some(value.as_str()),
                "{key}\\{name} did not reach the registry Windows reads"
            );
        }
        assert!(
            super::is_registered(),
            "what was just written reads as absent"
        );

        super::remove_registration().expect("removing failed");
        assert!(!super::is_registered(), "the registration survived removal");
        for (key, name, _) in registration.entries() {
            if key == registration.pointer().0 {
                // Windows' own key. Only this program's value comes out of it,
                // and every other program's entry stays.
                assert_eq!(what_reg_exe_sees(key, name), None, "{key}\\{name} survived");
                continue;
            }
            assert_eq!(what_reg_exe_sees(key, name), None, "{key}\\{name} survived");
        }
    }

    #[test]
    fn test_a_key_outside_the_users_hive_is_refused_rather_than_written() {
        // Nothing here produces one, and a test above says so. This is the
        // backstop: if one ever appears, it must fail loudly rather than being
        // written to the machine, where it needs administrator rights and
        // affects everybody sharing the computer.
        for refused in [
            r"HKLM\Software\Wixen\ShouldNeverBeWritten",
            r"Software\Wixen\NoHive",
            "HKCU",
            "",
        ] {
            assert!(
                super::windows_registry::write_one(refused, "Name", "Value").is_err(),
                "{refused:?} was accepted"
            );
        }
    }
}

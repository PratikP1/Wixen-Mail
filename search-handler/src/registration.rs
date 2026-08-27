//! Which registry entries make Windows Search aware of this handler.
//!
//! Kept apart from the code that writes them so the plan can be read and
//! tested without touching a real registry. The writing itself is in
//! [`crate::com::exports`] and is a loop over what this module returns.
//!
//! # This needs administrator rights, and that is not a detail
//!
//! Every entry below goes under `HKEY_LOCAL_MACHINE`, and it has to. The key
//! Windows Search reads its protocol handlers from exists only there on this
//! machine, and the process that runs a filter is the system account, which
//! cannot read a signed-in user's part of the registry at all. So this cannot
//! be switched on by a checkbox in a settings screen. It is an install-time
//! step, taken deliberately, by somebody who can elevate. Anything in the
//! product that offers this has to say so rather than presenting a control
//! that will silently fail for a standard user.
//!
//! # What is deliberately not registered
//!
//! Microsoft's page also lists a `ShellFolder` attributes value, a `TypeLib`,
//! and two entries under Explorer's `NameSpace` and `Shell Extensions`. Those
//! belong to a Shell data source, which is a separate thing: an implementation
//! of the folder interfaces that makes a store browsable in Explorer. This
//! crate is a protocol handler and a filter and nothing else, so registering
//! those would claim a capability that is not here.

/// Where an entry is written.
///
/// One value today, and it is still an enum rather than nothing, because the
/// hive is the part of this plan most likely to be argued about later and a
/// test can assert on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    LocalMachine,
}

/// One value to write into the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub hive: Hive,
    /// The key, below the hive.
    pub key: String,
    /// The value's name, or `None` for the key's own unnamed value.
    pub name: Option<String>,
    pub value: String,
}

/// The class that teaches the indexer our URL scheme.
///
/// Written twice on purpose, as a number for COM and as text for the registry,
/// with a test tying the two together. Both are needed, they are read by code
/// that never meets, and a pair that drifted apart would register one class and
/// create another, which shows as an indexer that loads the DLL and then
/// cannot make the object.
///
/// Neither may ever change. A new value is a new class, and a machine that had
/// the old one registered would keep pointing at something that no longer
/// exists.
pub const PROTOCOL_CLSID_VALUE: u128 = 0x10502846_3BAF_4125_93EE_46DC1843B1F0;
pub const PROTOCOL_CLSID: &str = "{10502846-3BAF-4125-93EE-46DC1843B1F0}";

/// The class that reads one of those URLs and hands back text.
pub const FILTER_CLSID_VALUE: u128 = 0xE0DBB3B7_FD6B_4542_9053_EB3C70BA0968;
pub const FILTER_CLSID: &str = "{E0DBB3B7-FD6B-4542-9053-EB3C70BA0968}";

/// A class identifier written the way the registry keeps them.
pub fn braced(value: u128) -> String {
    let hex = format!("{value:032X}");
    // The five groups a class identifier is always written in.
    let groups = [(0, 8), (8, 12), (12, 16), (16, 20), (20, 32)];
    let parts: Vec<&str> = groups
        .iter()
        .filter_map(|(from, to)| hex.get(*from..*to))
        .collect();

    format!("{{{}}}", parts.join("-"))
}

/// The name Windows Search is given for the protocol handler.
///
/// A version-dependent program identifier, ending in a number, which is the
/// convention every handler already in that key follows.
pub const PROTOCOL_PROGID: &str = "WixenMail.SearchProtocol.1";

/// The name that always means the current protocol handler.
pub const PROTOCOL_PROGID_ANY_VERSION: &str = "WixenMail.SearchProtocol";

/// The name Windows is given for the filter.
pub const FILTER_PROGID: &str = "WixenMail.SearchFilter.1";

/// The name that always means the current filter.
pub const FILTER_PROGID_ANY_VERSION: &str = "WixenMail.SearchFilter";

/// What a person sees beside these classes in a registry editor.
const PROTOCOL_DESCRIPTION: &str = "Wixen Mail Search Protocol Handler";
const FILTER_DESCRIPTION: &str = "Wixen Mail Search Filter";

/// Where Windows Search keeps the list of protocol handlers.
///
/// Read off this machine rather than copied from a page. There is no
/// `CurrentVersion` level: the key is exactly this, and it already holds
/// `Mapi`, `WinRT`, `IEHistory` and `IERSS`.
pub const SEARCH_PROTOCOL_HANDLERS: &str = r"SOFTWARE\Microsoft\Windows Search\ProtocolHandlers";

/// Where COM classes live.
const CLASSES: &str = r"Software\Classes";

/// Every entry to write, given the full path of this DLL.
pub fn entries(dll_path: &str) -> Vec<Entry> {
    let mut entries = Vec::new();

    entries.extend(class_entries(
        dll_path,
        PROTOCOL_CLSID,
        PROTOCOL_PROGID,
        PROTOCOL_PROGID_ANY_VERSION,
        PROTOCOL_DESCRIPTION,
    ));
    entries.extend(class_entries(
        dll_path,
        FILTER_CLSID,
        FILTER_PROGID,
        FILTER_PROGID_ANY_VERSION,
        FILTER_DESCRIPTION,
    ));

    // The one entry that is not ours to own: a single named value inside a key
    // Windows keeps its own handlers in.
    entries.push(Entry {
        hive: Hive::LocalMachine,
        key: SEARCH_PROTOCOL_HANDLERS.to_string(),
        name: Some(crate::url::SCHEME.to_string()),
        value: PROTOCOL_PROGID.to_string(),
    });

    entries
}

/// The ordinary in-process COM server registration for one class.
fn class_entries(
    dll_path: &str,
    clsid: &str,
    progid: &str,
    progid_any_version: &str,
    description: &str,
) -> Vec<Entry> {
    let unnamed = |key: String, value: &str| Entry {
        hive: Hive::LocalMachine,
        key,
        name: None,
        value: value.to_string(),
    };

    vec![
        unnamed(format!(r"{CLASSES}\{progid}"), description),
        unnamed(format!(r"{CLASSES}\{progid}\CLSID"), clsid),
        unnamed(format!(r"{CLASSES}\{progid_any_version}"), description),
        unnamed(format!(r"{CLASSES}\{progid_any_version}\CLSID"), clsid),
        unnamed(format!(r"{CLASSES}\{progid_any_version}\CurVer"), progid),
        unnamed(format!(r"{CLASSES}\CLSID\{clsid}"), description),
        unnamed(format!(r"{CLASSES}\CLSID\{clsid}\InprocServer32"), dll_path),
        Entry {
            hive: Hive::LocalMachine,
            key: format!(r"{CLASSES}\CLSID\{clsid}\InprocServer32"),
            name: Some("ThreadingModel".to_string()),
            // Both means COM may call straight into this DLL from whichever
            // thread the indexer is already on, rather than marshalling every
            // call to another one. That is why the objects here are written to
            // be safe on any thread.
            value: "Both".to_string(),
        },
        unnamed(format!(r"{CLASSES}\CLSID\{clsid}\ProgID"), progid),
        unnamed(
            format!(r"{CLASSES}\CLSID\{clsid}\VersionIndependentProgID"),
            progid_any_version,
        ),
    ]
}

/// The keys to delete when the handler is taken out.
///
/// Whole keys, because everything under them was written here. The Windows
/// Search key is deliberately absent: only one value in it is ours, and
/// deleting the key would take Microsoft's own handlers with it. That one is
/// [`value_to_remove`].
pub fn keys_to_remove() -> Vec<String> {
    let mut keys = Vec::new();
    for clsid in [PROTOCOL_CLSID, FILTER_CLSID] {
        keys.push(format!(r"{CLASSES}\CLSID\{clsid}"));
    }
    for progid in [
        PROTOCOL_PROGID,
        PROTOCOL_PROGID_ANY_VERSION,
        FILTER_PROGID,
        FILTER_PROGID_ANY_VERSION,
    ] {
        keys.push(format!(r"{CLASSES}\{progid}"));
    }
    keys
}

/// The single named value to delete from a key Windows owns.
pub const fn value_to_remove() -> (&'static str, &'static str) {
    (SEARCH_PROTOCOL_HANDLERS, crate::url::SCHEME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::SCHEME;

    const A_DLL: &str = r"C:\Program Files\Wixen Mail\wixen_mail_search.dll";

    fn value_at(entries: &[Entry], key: &str, name: Option<&str>) -> Option<String> {
        entries
            .iter()
            .find(|entry| entry.key == key && entry.name.as_deref() == name)
            .map(|entry| entry.value.clone())
    }

    #[test]
    fn test_the_class_points_at_the_file_it_was_actually_loaded_from() {
        // Self registration runs from inside the loaded DLL, so it knows its
        // own path. Writing a path worked out any other way is how a handler
        // ends up registered against a copy somebody moved or deleted, and the
        // failure then looks like the indexer ignoring the scheme.
        let entries = entries(A_DLL);

        assert_eq!(
            value_at(
                &entries,
                &format!(r"Software\Classes\CLSID\{PROTOCOL_CLSID}\InprocServer32"),
                None
            )
            .as_deref(),
            Some(A_DLL)
        );
    }

    #[test]
    fn test_the_server_says_it_can_be_called_on_any_thread() {
        // The threading model is what tells COM whether it may call straight
        // in or has to marshal to another thread. Microsoft's page says Both
        // for a protocol handler, and getting it wrong costs a thread switch
        // on every single item in a mailbox.
        let entries = entries(A_DLL);

        for clsid in [PROTOCOL_CLSID, FILTER_CLSID] {
            assert_eq!(
                value_at(
                    &entries,
                    &format!(r"Software\Classes\CLSID\{clsid}\InprocServer32"),
                    Some("ThreadingModel")
                )
                .as_deref(),
                Some("Both"),
                "{clsid}"
            );
        }
    }

    #[test]
    fn test_the_scheme_is_registered_under_the_key_windows_search_really_reads() {
        // Checked against this machine rather than taken from the page:
        // HKLM\SOFTWARE\Microsoft\Windows Search\ProtocolHandlers exists and
        // holds Mapi, WinRT, IEHistory and IERSS. There is no CurrentVersion
        // level in that path, and a handler written one level out is simply
        // never loaded, with nothing anywhere to say why.
        let entries = entries(A_DLL);

        assert_eq!(
            value_at(&entries, SEARCH_PROTOCOL_HANDLERS, Some(SCHEME)).as_deref(),
            Some(PROTOCOL_PROGID),
            "the scheme was not registered where Windows Search looks"
        );
        assert_eq!(
            SEARCH_PROTOCOL_HANDLERS,
            r"SOFTWARE\Microsoft\Windows Search\ProtocolHandlers"
        );
    }

    #[test]
    fn test_windows_search_is_given_a_program_identifier_and_not_a_class_identifier() {
        // The values already in that key are names like Search.Mapi2Handler.1,
        // not braces. A class identifier there is not resolved and the handler
        // never loads.
        let entries = entries(A_DLL);
        let registered =
            value_at(&entries, SEARCH_PROTOCOL_HANDLERS, Some(SCHEME)).expect("the scheme");

        assert!(!registered.starts_with('{'), "{registered}");
        assert_eq!(
            value_at(
                &entries,
                &format!(r"Software\Classes\{registered}\CLSID"),
                None
            )
            .as_deref(),
            Some(PROTOCOL_CLSID),
            "the name given to Windows Search does not lead back to the class"
        );
    }

    #[test]
    fn test_both_classes_are_registered_because_a_handler_without_a_filter_reads_nothing() {
        // The protocol handler finds items and the filter reads them. Only one
        // of the two registered means the indexer walks the whole mailbox and
        // indexes not one word of it.
        let entries = entries(A_DLL);

        for clsid in [PROTOCOL_CLSID, FILTER_CLSID] {
            assert!(
                entries
                    .iter()
                    .any(|entry| entry.key.contains(clsid) && entry.key.ends_with("InprocServer32")),
                "{clsid} has no server registration"
            );
        }
        assert_ne!(PROTOCOL_CLSID, FILTER_CLSID);
    }

    #[test]
    fn test_each_class_identifier_says_the_same_thing_as_a_number_and_as_text() {
        // COM needs the number and the registry needs the text. They are
        // written out separately, read by code that never meets, and a pair
        // that had drifted would register one class and create another. The
        // symptom is an indexer that loads this DLL and then fails to make the
        // object, with nothing to say the two identifiers differ.
        assert_eq!(braced(PROTOCOL_CLSID_VALUE), PROTOCOL_CLSID);
        assert_eq!(braced(FILTER_CLSID_VALUE), FILTER_CLSID);
    }

    #[test]
    fn test_a_class_identifier_is_written_in_the_shape_the_registry_expects() {
        // Braces around it, hyphens inside. The registry treats the text as a
        // key name and does not correct it, so a wrongly shaped one makes a
        // key nothing ever looks in.
        for clsid in [PROTOCOL_CLSID, FILTER_CLSID] {
            assert!(clsid.starts_with('{') && clsid.ends_with('}'), "{clsid}");
            assert_eq!(clsid.len(), 38, "{clsid}");
            assert_eq!(clsid.matches('-').count(), 4, "{clsid}");
            assert_eq!(clsid.to_uppercase(), clsid, "{clsid}");
        }
    }

    #[test]
    fn test_taking_the_handler_out_removes_every_key_it_put_in() {
        // A half removed COM server leaves the indexer loading a DLL that is
        // no longer there, and the person uninstalling has no way to know. So
        // removal is checked against registration rather than written from
        // memory, which is how the two drift apart.
        let written = entries(A_DLL);
        let removed = keys_to_remove();

        for entry in &written {
            if entry.key == SEARCH_PROTOCOL_HANDLERS {
                // The one entry that is a value inside a key Windows owns.
                // Deleting that key would take Mapi and the rest with it.
                continue;
            }
            assert!(
                removed.iter().any(|key| entry.key.starts_with(key)),
                "{} would be left behind",
                entry.key
            );
        }
    }

    #[test]
    fn test_removal_never_deletes_a_key_that_belongs_to_windows() {
        // The scheme is one value inside a key holding Microsoft's own
        // handlers. Removing the key rather than the value would unregister
        // Outlook's mail handler along with ours.
        assert!(
            !keys_to_remove()
                .iter()
                .any(|key| SEARCH_PROTOCOL_HANDLERS.starts_with(key.as_str())),
            "removal would delete a key Windows owns"
        );
        assert_eq!(value_to_remove(), (SEARCH_PROTOCOL_HANDLERS, SCHEME));
    }

    #[test]
    fn test_everything_written_sits_in_one_of_the_two_places_this_handler_owns() {
        // A registration that reaches anywhere else is a registration nobody
        // can audit or undo. There are exactly two: our own classes, and the
        // one value in the Windows Search key.
        for entry in entries(A_DLL) {
            let ours = entry.key.starts_with(r"Software\Classes\")
                || entry.key == SEARCH_PROTOCOL_HANDLERS;
            assert!(ours, "{} is outside what this handler owns", entry.key);
        }
    }

    #[test]
    fn test_everything_goes_to_the_machine_and_not_to_one_signed_in_user() {
        // Both halves have to agree. The Windows Search key only exists under
        // the machine on this computer, and the filter host runs as the system
        // account, which cannot read a signed-in user's part of the registry
        // at all. So the class registration has to be per machine too, and
        // that is what makes this need administrator rights.
        for entry in entries(A_DLL) {
            assert_eq!(entry.hive, Hive::LocalMachine, "{}", entry.key);
        }
    }
}

//! Accessible names for controls that carry no visible label.
//!
//! A list, tree, or text field with no label next to it reaches the UI
//! Automation tree with a null Name. A screen reader then announces it as
//! "list" or "tree" with nothing to say which one, which is the difference
//! between a usable window and a guessing game.
//!
//! `wxWindow::SetName` does **not** do this. That name is an internal
//! wxWidgets identifier used for resource lookup and never reaches assistive
//! technology. Only a `wxAccessible` attached to the window does, which is
//! what this module provides.
//!
//! **This is Windows only.** `wxAccessible` is implemented against Microsoft
//! Active Accessibility and has no GTK or macOS counterpart in wxWidgets, so on
//! those platforms these calls are accepted and have no effect. A port would
//! need `setAccessibilityLabel:` on the NSView for macOS and an ATK name for
//! GTK, written as separate bridges rather than as a change here.
//!
//! # Which reader hears what, measured
//!
//! What is set here is an MSAA name, and for a control Windows draws itself,
//! that is not what UI Automation reports. Windows supplies its own UIA
//! provider for a native edit, button or combo box, and it answers before the
//! bridge that would otherwise expose this object. Read through the two APIs
//! side by side against the running composer, the same control answers twice:
//!
//! ```text
//! Button   UIA='B'                MSAA='Bold, Ctrl+B'
//! Button   UIA='Attach File...'   MSAA='Attach a file, Alt+A'
//! Edit     UIA='To:'              MSAA='To'
//! ```
//!
//! So these names are not lost, and a reader that uses IAccessible for ordinary
//! Win32 controls, which is what NVDA does in a window like this one, hears
//! exactly what is written here, descriptions included. A reader that uses UI
//! Automation, which is what Narrator does, hears the control's own window
//! text: the visible label for a button, the static label beside a field.
//!
//! Three things follow, and they are the reason this is written down.
//!
//! Set a name here **and** make the visible label say the same thing, wherever
//! the two can agree. Where they cannot, the name here is the fuller one and
//! the label is the shorter one, never a different fact.
//!
//! A description set here reaches MSAA only. Anything somebody must hear to
//! work a control cannot live only in a description; put it in the label, or
//! announce it.
//!
//! And the automated scan reads the UI Automation tree, so it is measuring the
//! labels rather than any of this. A clean scan says the labels are present. It
//! says nothing at all about these.

use wxdragon::accessible::{AccStatus, Accessible, AccessibleImpl};
use wxdragon::ffi;
use wxdragon::prelude::{SpinCtrl, WxWidget};

/// Whether a name set here reaches the accessibility tree on this build.
///
/// **A statement about wxWidgets, not about this code.** The module header
/// above records the measurement: `wxAccessible` is implemented against
/// Microsoft Active Accessibility and has no GTK or macOS counterpart, so
/// `set_accessible` is accepted everywhere and does something in one place.
/// That is why this is two arms here rather than two modules the way
/// [`super::screen_reader`] has them: there is no per-platform code in this file
/// to put in a module, only a fact about what the toolkit underneath does.
///
/// A port is a third arm, written beside the bridge that earns it:
/// `setAccessibilityLabel:` on the NSView for macOS, an ATK name for GTK, as
/// the header says. Read by [`super::platform_bridge`], which turns it into the
/// sentence somebody meets.
#[cfg(target_os = "windows")]
pub const A_NAME_REACHES_THE_ACCESSIBILITY_TREE: bool = true;

/// Whether a name set here reaches the accessibility tree on this build.
///
/// See the Windows arm above for why this is a fact about wxWidgets. Here the
/// calls in this module are accepted and have no effect, so every control with
/// no visible label beside it reaches the accessibility tree with no name.
#[cfg(not(target_os = "windows"))]
pub const A_NAME_REACHES_THE_ACCESSIBILITY_TREE: bool = false;

/// Supplies one fixed name, and optionally a description, for a control,
/// leaving every other accessibility property to the platform's default
/// handling.
struct FixedName {
    name: String,
    description: Option<String>,
}

impl AccessibleImpl for FixedName {
    /// Name the control itself, and nothing inside it.
    ///
    /// Child zero is the control. Anything else is a row, a tree node, or a
    /// notebook tab asking for its own name, and answering those with the
    /// control's name gave every one of them the same label. That is why the
    /// settings tabs stayed silent even after child enumeration was restored:
    /// enumeration worked, and then every tab reported itself as "Settings
    /// categories".
    fn get_name(&self, child_id: i32) -> (AccStatus, Option<String>) {
        if child_id != 0 {
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None);
        }
        (ffi::wxd_AccStatus_WXD_ACC_OK, Some(self.name.clone()))
    }

    /// Defer to the control's own child enumeration.
    ///
    /// This override is not optional. The trait's default answers `OK` with a
    /// count of zero, which is a positive claim that the control has no
    /// children rather than a request to fall back. Attaching one of these to a
    /// notebook silenced its tabs, and to a list or tree it would have hidden
    /// every item, because the accessible object was answering "nothing in
    /// here" on the control's behalf.
    ///
    /// Every other method left at its default returns `NOT_IMPLEMENTED`, which
    /// is what makes wxWidgets use its own implementation. Only the name and
    /// the description are meant to be replaced here.
    fn get_child_count(&self) -> (AccStatus, i32) {
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0)
    }

    /// The sentence a screen reader reads after the name, on focus.
    ///
    /// This is the property that carries "what does choosing this cost me".
    /// Putting that in the name instead makes it part of every announcement,
    /// including the one when arrowing past, which is how a four-sentence
    /// radio button label happens. A description is spoken once, when the
    /// control takes focus.
    ///
    /// Same rule as the name about child zero: a row or a tab asking for its
    /// own description must not be handed the control's.
    fn get_description(&self, child_id: i32) -> (AccStatus, Option<String>) {
        match (&self.description, child_id) {
            (Some(description), 0) => (ffi::wxd_AccStatus_WXD_ACC_OK, Some(description.clone())),
            _ => (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None),
        }
    }
}

// Until 2026-09-18 this file also held `CheckedRows`, an object that answered
// for each row of the folder chooser's `CheckListBox` as a check box with a
// checked state, written through wxdragon's `acc_state` constants and held by
// twenty tests to answering those flags. Read over `AccessibleObjectFromWindow`
// by `tests/a_kept_folder_reads_as_a_checked_check_box.rs` (11-03, reading A)
// the rows answered role check button and states 0xc000840 on the kept row and
// 0x40 on the others: READONLY, BUSY and two ALERT bits, never CHECKED, which
// is what the tester heard (#70). wxdragon 0.9.17 numbers those constants as
// MSAA does (CHECKED 0x10, FOCUSABLE 0x100000, SELECTABLE 0x200000), its shim
// hands them to `wxAccessible::GetState` unconverted, and wxWidgets numbers its
// own enumeration differently (BUSY 0x10, PROTECTED 0x100000, READONLY
// 0x200000) before converting that to the platform's. The dialog is a native
// tree with `TVS_CHECKBOXES` now, whose state NVDA reads from the control
// itself, and the object, its tests and its four guard records went with it.
// Nothing else in this tree writes a state through those constants; whoever
// next does should read this paragraph first.

/// Give `window` an accessible name that screen readers will announce.
///
/// Use the same wording as the control's visible heading or tree root, so what
/// is spoken matches what is on screen.
pub fn set_accessible_name(window: &dyn WxWidget, name: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: None,
        },
    ));
}

/// Give `window` a name and the sentence that explains it.
///
/// For a control whose label is not enough on its own: a choice with a
/// consequence, a field with a rule about what it accepts. Screen readers read
/// the description after the name when the control takes focus, so what is on
/// screen next to the control is heard by somebody who cannot see it.
///
/// Without this, an explanation sitting beside a control is a label floating in
/// the window that a screen reader user reaches only by leaving the control and
/// reading around, if they think to. The first-run screen shipped that way:
/// three radio buttons that read correctly and three explanations of what each
/// one costs that were never spoken.
///
/// One call, not two. Attaching an accessible object replaces the last one, so
/// setting a name and then a description would leave only the description.
pub fn set_accessible_name_and_description(window: &dyn WxWidget, name: &str, description: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: Some(description.to_string()),
        },
    ));
}

/// Turn a visible label into the name a screen reader should announce.
///
/// Drops the mnemonic ampersand: some screen readers read it aloud, and none
/// of them should.
///
/// A trailing colon, the visual convention marking a field's label, becomes a
/// trailing comma rather than being dropped outright. Dropping it left
/// nothing between the name and whatever a screen reader reads next, the
/// control's role and then its value, and a real NVDA run found exactly
/// that: no pause, consistently, across most of the fields in the
/// application. A comma is never read aloud as a word the way "colon"
/// sometimes is, and it is still a pause to the speech synthesiser
/// underneath.
///
/// An ampersand means two different things in a wxWidgets label and they have
/// to be told apart. A lone one marks the letter after it as the mnemonic and
/// is not part of what the label says. Two in a row are how a label writes one
/// real ampersand, as in the composer's "&Go Back && Edit". Deleting every one
/// of them takes the word out of the label and leaves a double space where a
/// listener hears an odd pause. The real ampersand is kept as a character
/// rather than turned into the word "and", because a screen reader already
/// reads it and rewriting it would put a word in the name that is not on the
/// control.
///
/// One trailing colon becomes one trailing comma, not a run of them. A single
/// colon after a field label is the visual convention; a second is something
/// the label says, and is left as a colon rather than turned into something
/// the label never had.
pub fn name_from_label(label: &str) -> String {
    let mut spoken = String::with_capacity(label.len());
    let mut rest = label.chars().peekable();
    while let Some(character) = rest.next() {
        if character != '&' {
            spoken.push(character);
            continue;
        }
        if rest.peek() == Some(&'&') {
            rest.next();
            spoken.push('&');
        }
    }
    let trimmed = spoken.trim();
    match trimmed.strip_suffix(':') {
        Some(before) if !before.trim().is_empty() => format!("{},", before.trim()),
        Some(_) => String::new(),
        None => trimmed.to_string(),
    }
}

/// Name a spin control on both of its windows.
///
/// A Windows spin control is two windows, and the keyboard lands in the one
/// wxWidgets does not hand back. `get_handle` answers the arrows, an
/// `msctls_updown32`, which is where [`set_accessible_name`] attaches its
/// object; the number sits in the arrows' buddy, an `Edit`, and Tab reaches
/// that and never the arrows. Named on the arrows alone, the field a person
/// types in was nameless on MSAA, which NVDA reads for an edit, and named
/// from whatever static text sat before it on UI Automation (ledger 408 to
/// 425). So the field is given the same words through the annotation service,
/// which both channels read in a test process, measured by
/// `tests/every_spin_control_names_the_field_a_person_types_in.rs` on every
/// spin control in the program.
///
/// **Not in the running program.** On 2026-09-23 the Accessibility scan on
/// pull request #97 launched the real program and found the fields as they
/// were before this existed: nameless, or named by the static text before
/// them. Why the annotation reaches the field in a test process and not in the
/// app is not known; 12-06.1 owns it, and ledger 408 to 425 stay open.
///
/// Every spin control is named through this or
/// [`name_and_describe_the_spin_control`], and through nothing else.
pub fn name_the_spin_control(spin: &SpinCtrl, name: &str) {
    set_accessible_name(spin, name);
    typing_field::carry(spin, &what_the_typing_field_carries(name, None));
}

/// [`name_the_spin_control`], with the sentence that explains it on both
/// windows too. On the arrows alone a description is never heard, because
/// nobody's focus is ever on the arrows.
pub fn name_and_describe_the_spin_control(spin: &SpinCtrl, name: &str, description: &str) {
    set_accessible_name_and_description(spin, name, description);
    typing_field::carry(
        spin,
        &what_the_typing_field_carries(name, Some(description)),
    );
}

/// 12-06.1's startup bisect, D-01 extended on 2026-09-24, leaving with the
/// rest of the diagnosis: switched on for the scan's runs only.
pub fn diagnose_startup_naming(on: bool) {
    #[cfg(target_os = "windows")]
    typing_field::diagnose_startup_naming(on);
    #[cfg(not(target_os = "windows"))]
    let _ = on;
}

/// 12-06.1's startup bisect: name and read back a throwaway field at `point`,
/// one line in the log. `toolkit_is_up` once wxWidgets has started.
pub fn diagnose_naming_at(point: &str, toolkit_is_up: bool) {
    #[cfg(target_os = "windows")]
    typing_field::diagnose_naming_at(point, toolkit_is_up);
    #[cfg(not(target_os = "windows"))]
    let _ = (point, toolkit_is_up);
}

/// Which property of a spin control's typing field some words are written to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldProperty {
    Name,
    Description,
}

/// What the field a person types in carries, property by property: the
/// arrows' name, and their description where they have one.
fn what_the_typing_field_carries<'a>(
    name: &'a str,
    description: Option<&'a str>,
) -> Vec<(FieldProperty, &'a str)> {
    std::iter::once((FieldProperty::Name, name))
        .chain(description.map(|said| (FieldProperty::Description, said)))
        .collect()
}

/// Writing onto the typing field, which only Windows has a way to do.
#[cfg(target_os = "windows")]
mod typing_field {
    use super::FieldProperty;
    use std::cell::OnceCell;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    use windows::Win32::UI::Accessibility::{
        CLSID_AccPropServices, IAccPropServices, PROPID_ACC_DESCRIPTION, PROPID_ACC_NAME,
    };
    use windows::Win32::UI::Controls::UDM_GETBUDDY;
    use windows::Win32::UI::WindowsAndMessaging::{CHILDID_SELF, OBJID_CLIENT, SendMessageW};
    use windows::core::{GUID, HSTRING};
    use wxdragon::prelude::{SpinCtrl, WxWidget};

    thread_local! {
        /// The annotation service, created the first time a spin control is
        /// named and kept. Per thread because a COM interface is, and every
        /// window is built on the one interface thread.
        static SERVICE: OnceCell<Option<IAccPropServices>> = const { OnceCell::new() };
    }

    fn the_service() -> windows::core::Result<IAccPropServices> {
        // SAFETY: COM is initialised on the interface thread by wxWidgets
        // before any window is built.
        unsafe { CoCreateInstance(&CLSID_AccPropServices, None, CLSCTX_INPROC_SERVER) }
            .inspect_err(|why| tracing::debug!("The annotation service is not available: {why}"))
    }

    /// The service for this thread, asked for once and kept, whichever way
    /// the first ask went, as 12-06 shipped it; and what this call found.
    fn the_kept_service(
        kept: &OnceCell<Option<IAccPropServices>>,
    ) -> (Option<&IAccPropServices>, diagnosis::ServiceAsked) {
        use diagnosis::ServiceAsked;
        let asked = match kept.get() {
            Some(Some(_)) => ServiceAsked::Reused,
            Some(None) => ServiceAsked::Absent,
            None => {
                let created = the_service();
                let asked = match &created {
                    Ok(_) => ServiceAsked::Created(diagnosis::count_a_service()),
                    Err(why) => ServiceAsked::Failed(why.code()),
                };
                let _ = kept.set(created.ok());
                asked
            }
        };
        (kept.get().and_then(Option::as_ref), asked)
    }

    fn property(which: FieldProperty) -> GUID {
        match which {
            FieldProperty::Name => PROPID_ACC_NAME,
            FieldProperty::Description => PROPID_ACC_DESCRIPTION,
        }
    }

    /// Write `carries` onto the field beside `spin`'s arrows. A field that
    /// cannot be found or written is said at debug and left as it was: the
    /// arrows still carry the words.
    pub(super) fn carry(spin: &SpinCtrl, carries: &[(FieldProperty, &str)]) {
        let call = diagnosis::next_call();
        let named = carries
            .iter()
            .find(|(which, _)| *which == FieldProperty::Name)
            .map_or("", |(_, words)| *words);
        let arrows = HWND(spin.get_handle());
        // SAFETY: the arrows are a live window this program built; the
        // message takes and returns no pointer.
        let buddy = unsafe { SendMessageW(arrows, UDM_GETBUDDY, None, None) };
        if buddy.0 == 0 {
            diagnosis::the_call(call, named, arrows, None, None);
            tracing::debug!("A spin control has no typing field to name");
            return;
        }
        let field = HWND(buddy.0 as *mut std::ffi::c_void);
        SERVICE.with(|service| {
            let (service, asked) = the_kept_service(service);
            diagnosis::the_call(call, named, arrows, Some(field), Some(asked));
            let Some(service) = service else {
                return;
            };
            for (which, words) in carries {
                // SAFETY: the field is a live window; the string outlives the call.
                let written = unsafe {
                    service.SetHwndPropStr(
                        field,
                        OBJID_CLIENT.0 as u32,
                        CHILDID_SELF,
                        property(*which),
                        &HSTRING::from(*words),
                    )
                };
                diagnosis::the_write(call, *which, &written);
                if let Err(why) = written {
                    tracing::debug!("A spin control's typing field could not be named: {why}");
                }
            }
        });
        diagnosis::the_properties(call, field);
        diagnosis::the_read_back(call, field, named);
        diagnosis::check_at_show(call, arrows, field, named);
    }

    /// 12-06.1's startup bisect, D-01 extended on 2026-09-24: switch it on.
    pub(super) fn diagnose_startup_naming(on: bool) {
        diagnosis::probe_the_startup(on);
    }

    /// 12-06.1's startup bisect: one line for `point` in startup.
    pub(super) fn diagnose_naming_at(point: &str, toolkit_is_up: bool) {
        diagnosis::at_the_point(point, toolkit_is_up);
    }

    /// 12-06.1's diagnosis, and nothing else: every line it writes starts
    /// `spin-field-naming:` and is at warn, so the Accessibility scan's copy
    /// of the running program's log says what became of each name. Not
    /// test-first, on Pratik's exception of 2026-09-24 for the diagnostic
    /// lines only; it changes no answer and leaves with the diagnosis, as a
    /// module, once the scan has been read.
    ///
    /// Written: handles in decimal, HRESULTs in hex, the thread, the
    /// apartment, the window class and the helper's own fixed words. Never a
    /// field's value, a window's title, or anything from mail or an account.
    mod diagnosis {
        use super::FieldProperty;
        use std::cell::Cell;
        use std::sync::OnceLock;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Com::{APTTYPE, APTTYPEQUALIFIER, CoGetApartmentType};
        use windows::Win32::System::Threading::GetCurrentThreadId;
        use windows::Win32::System::Variant::{
            VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4,
        };
        use windows::Win32::UI::Accessibility::{AccessibleObjectFromWindow, IAccessible};
        use windows::Win32::UI::Controls::UDM_GETBUDDY;
        use windows::Win32::UI::WindowsAndMessaging::{
            CHILDID_SELF, GWLP_WNDPROC, GetClassNameW, GetWindowLongPtrW, IsWindow, OBJID_CLIENT,
            SendMessageW,
        };
        use windows::core::{HRESULT, Interface};

        /// Whether this process bisects its startup (12-06.1, D-01 extended
        /// on 2026-09-24): on in the scan's runs only, never in a test.
        static PROBING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

        /// What the throwaway fields are named, and all the bisect writes.
        const PROBE_WORDS: &str = "Probe words";

        pub(super) fn probe_the_startup(on: bool) {
            PROBING.store(on, std::sync::atomic::Ordering::Relaxed);
        }

        /// How many annotation service instances this process has created,
        /// across every thread, so a line can say whether its instance was
        /// the process's first.
        static SERVICES_CREATED: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);

        /// Count one more service instance created; its number in the process.
        pub(super) fn count_a_service() -> u32 {
            SERVICES_CREATED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
        }

        /// The one point this process probes, from the scan's workflow
        /// (deviation 2, the bisect across processes): one first use per
        /// process, so a probe cannot cure the points after it.
        fn the_assigned_point() -> Option<&'static str> {
            static ASSIGNED: OnceLock<Option<String>> = OnceLock::new();
            ASSIGNED
                .get_or_init(|| {
                    std::env::var("WIXEN_DIAGNOSE_PROBE_AT")
                        .ok()
                        .filter(|point| !point.is_empty())
                })
                .as_deref()
        }

        /// One line for this process's point in startup: a throwaway plain
        /// field named and read back on this thread, the process's first use
        /// of the annotation service, and then a hidden spin control's field
        /// once the toolkit is up; with the two accessibility libraries the
        /// process has loaded by then.
        pub(super) fn at_the_point(point: &str, toolkit_is_up: bool) {
            if !PROBING.load(std::sync::atomic::Ordering::Relaxed)
                || reads_are_switched_off()
                || the_assigned_point() != Some(point)
            {
                return;
            }
            let first = a_plain_field_named_and_read();
            let spin = if toolkit_is_up {
                a_spin_field_named_and_read()
            } else {
                "toolkit-not-up".to_string()
            };
            tracing::warn!(
                "spin-field-naming: point={point} first-use=[{first}] then-spin=[{spin}] {}",
                the_libraries()
            );
        }

        /// A hidden plain edit, named and read back, then destroyed.
        fn a_plain_field_named_and_read() -> String {
            use windows::Win32::UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_POPUP,
            };
            let apartment = the_apartment();
            // SAFETY: a hidden window of a system class, destroyed below.
            let created = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    windows::core::w!("Edit"),
                    windows::core::PCWSTR::null(),
                    WS_POPUP,
                    0,
                    0,
                    40,
                    20,
                    None,
                    None,
                    None,
                    None,
                )
            };
            let field = match created {
                Ok(field) => field,
                Err(why) => return format!("apartment={apartment} window hr={}", hex(why.code())),
            };
            let named = name_and_read(field);
            // SAFETY: the window made above, on this thread.
            let _ = unsafe { DestroyWindow(field) };
            named
        }

        /// A hidden spin control's field, named and read back, then the
        /// frame holding it destroyed.
        fn a_spin_field_named_and_read() -> String {
            use wxdragon::prelude::*;
            let frame = Frame::builder().build();
            let spin = SpinCtrl::builder(&frame).with_range(0, 9).build();
            // SAFETY: the arrows are a live window just built.
            let buddy =
                unsafe { SendMessageW(HWND(spin.get_handle()), UDM_GETBUDDY, None, None).0 };
            let read = if buddy == 0 {
                "no field".to_string()
            } else {
                name_and_read(window(buddy))
            };
            frame.destroy();
            read
        }

        /// Name `field` through a service of its own and read it back.
        fn name_and_read(field: HWND) -> String {
            use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
            use windows::Win32::UI::Accessibility::{
                CLSID_AccPropServices, IAccPropServices, PROPID_ACC_NAME,
            };
            // SAFETY: an in-process COM object; failure is an answer.
            let created: windows::core::Result<IAccPropServices> =
                unsafe { CoCreateInstance(&CLSID_AccPropServices, None, CLSCTX_INPROC_SERVER) };
            let service = match created {
                Ok(service) => service,
                Err(why) => return format!("service hr={}", hex(why.code())),
            };
            let instance = count_a_service();
            let apartment = the_apartment();
            // SAFETY: the field is a live window; the string outlives the call.
            let written = unsafe {
                service.SetHwndPropStr(
                    field,
                    OBJID_CLIENT.0 as u32,
                    CHILDID_SELF,
                    PROPID_ACC_NAME,
                    &windows::core::HSTRING::from(PROBE_WORDS),
                )
            };
            let code = written.map_or_else(|why| why.code(), |()| HRESULT(0));
            let properties = the_properties_of(field);
            let read = match msaa_name(field) {
                Ok(name) => format!("name=\"{name}\" equals={}", name == PROBE_WORDS),
                Err(why) => format!("read hr={}", hex(why.code())),
            };
            let stored = properties.contains("MSAA_");
            format!(
                "apartment={apartment} instance={instance} first-in-process={} write={} stored={stored} props={properties} {read}",
                instance == 1,
                hex(code)
            )
        }

        /// Called once per window property; `found` is the list it adds to.
        unsafe extern "system" fn one_property(
            _window: HWND,
            name: windows::core::PCWSTR,
            _data: windows::Win32::Foundation::HANDLE,
            found: usize,
        ) -> windows::core::BOOL {
            // SAFETY: `found` is the list `the_properties_of` handed over,
            // alive for the whole enumeration.
            let found = unsafe { &mut *(found as *mut Vec<String>) };
            let raw = name.0 as usize;
            found.push(if raw >> 16 == 0 {
                format!("#{raw}")
            } else {
                // SAFETY: a property name is a nul-terminated string when it
                // is not an atom.
                unsafe { name.to_string() }.unwrap_or_else(|_| "?".to_string())
            });
            windows::core::BOOL(1)
        }

        /// The names of `field`'s window properties, never their values.
        fn the_properties_of(field: HWND) -> String {
            use windows::Win32::Foundation::LPARAM;
            use windows::Win32::UI::WindowsAndMessaging::EnumPropsExW;
            let mut found: Vec<String> = Vec::new();
            // SAFETY: the list outlives the enumeration, which is synchronous.
            unsafe {
                EnumPropsExW(
                    field,
                    Some(one_property),
                    LPARAM(&mut found as *mut Vec<String> as isize),
                )
            };
            format!("[{}]", found.join(","))
        }

        pub(super) fn the_properties(call: u32, field: HWND) {
            tracing::warn!(
                "spin-field-naming: props={call} {}",
                the_properties_of(field)
            );
            if call == 1 {
                tracing::warn!("spin-field-naming: libraries={call} {}", the_libraries());
                tracing::warn!(
                    "spin-field-naming: setprop={call} {}",
                    a_plain_property_on(field)
                );
            }
        }

        /// Set one plain window property of this program's own on `field`,
        /// read it back and take it off again: whether any property sticks on
        /// that window at that moment, apart from the annotation service.
        fn a_plain_property_on(field: HWND) -> String {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::UI::WindowsAndMessaging::{GetPropW, RemovePropW, SetPropW};
            const MARK: isize = 0x5A5A;
            let key = windows::core::w!("WixenDiagnoseProbe");
            // SAFETY: a live window; the key is a static string and the value
            // a plain number, never a pointer.
            let set = unsafe { SetPropW(field, key, Some(HANDLE(MARK as *mut std::ffi::c_void))) };
            // SAFETY: as above.
            let read = unsafe { GetPropW(field, key) }.0 as isize;
            let properties = the_properties_of(field);
            // SAFETY: as above; takes off only what was set here.
            let _ = unsafe { RemovePropW(field, key) };
            format!(
                "set={} read=0x{read:X} equals={} props={properties}",
                set.map_or_else(|why| hex(why.code()), |()| "ok".to_string()),
                read == MARK
            )
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetModuleHandleW(name: *const u16) -> isize;
            fn GetModuleFileNameW(module: isize, path: *mut u16, size: u32) -> u32;
        }

        #[link(name = "version")]
        unsafe extern "system" {
            fn GetFileVersionInfoSizeW(path: *const u16, handle: *mut u32) -> u32;
            fn GetFileVersionInfoW(
                path: *const u16,
                handle: u32,
                size: u32,
                data: *mut std::ffi::c_void,
            ) -> i32;
            fn VerQueryValueW(
                block: *const std::ffi::c_void,
                sub_block: *const u16,
                out: *mut *mut std::ffi::c_void,
                length: *mut u32,
            ) -> i32;
        }

        fn wide(text: &str) -> Vec<u16> {
            text.encode_utf16().chain(Some(0)).collect()
        }

        /// Where a loaded library came from and its file version, or that
        /// it is not loaded. System paths only; nothing of the person's.
        fn the_library(name: &str) -> String {
            // SAFETY: a nul-terminated name; the answer is a handle or 0.
            let module = unsafe { GetModuleHandleW(wide(name).as_ptr()) };
            if module == 0 {
                return format!("{name}=not-loaded");
            }
            let mut path = [0u16; 520];
            // SAFETY: the buffer's length is passed with it.
            let length = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), 520) } as usize;
            let path = &path[..length.min(520)];
            format!(
                "{name}=\"{}\" version={}",
                String::from_utf16_lossy(path),
                the_version(path)
            )
        }

        fn the_version(path: &[u16]) -> String {
            let path: Vec<u16> = path.iter().copied().chain(Some(0)).collect();
            let mut ignored = 0u32;
            // SAFETY: a nul-terminated path and a local to write to.
            let size = unsafe { GetFileVersionInfoSizeW(path.as_ptr(), &mut ignored) };
            if size == 0 {
                return "unknown".to_string();
            }
            let mut block = vec![0u8; size as usize];
            // SAFETY: the block is as long as `size` says.
            if unsafe { GetFileVersionInfoW(path.as_ptr(), 0, size, block.as_mut_ptr().cast()) }
                == 0
            {
                return "unknown".to_string();
            }
            let mut fixed = std::ptr::null_mut();
            let mut length = 0u32;
            // SAFETY: the root query answers a pointer into `block`.
            let found = unsafe {
                VerQueryValueW(
                    block.as_ptr().cast(),
                    wide("\\").as_ptr(),
                    &mut fixed,
                    &mut length,
                )
            };
            if found == 0 || fixed.is_null() || length < 16 {
                return "unknown".to_string();
            }
            // SAFETY: VS_FIXEDFILEINFO starts with four u32s, the last two
            // the file version, inside `block`, which is still alive.
            let words = unsafe { std::slice::from_raw_parts(fixed as *const u32, 4) };
            let (high, low) = (words[2], words[3]);
            format!(
                "{}.{}.{}.{}",
                high >> 16,
                high & 0xFFFF,
                low >> 16,
                low & 0xFFFF
            )
        }

        fn the_libraries() -> String {
            format!(
                "{} {}",
                the_library("oleacc.dll"),
                the_library("UIAutomationCore.dll")
            )
        }

        /// What asking for the annotation service found on one naming call.
        #[derive(Debug, Clone, Copy)]
        pub(super) enum ServiceAsked {
            /// Created on this call, with its number among the process's
            /// service instances.
            Created(u32),
            Failed(HRESULT),
            Reused,
            Absent,
        }

        thread_local! {
            static CALLS: Cell<u32> = const { Cell::new(0) };
        }

        /// The control the scan's send-later target runs under: set, and the
        /// program reads no name inside itself, so a read inside the program
        /// cannot be what carries the name out of it.
        fn reads_are_switched_off() -> bool {
            static SWITCHED_OFF: OnceLock<bool> = OnceLock::new();
            *SWITCHED_OFF.get_or_init(|| std::env::var_os("WIXEN_DIAGNOSE_NO_READBACK").is_some())
        }

        pub(super) fn next_call() -> u32 {
            CALLS.with(|calls| {
                let call = calls.get() + 1;
                calls.set(call);
                call
            })
        }

        fn hex(code: HRESULT) -> String {
            format!("0x{:08X}", code.0 as u32)
        }

        fn handle(window: HWND) -> isize {
            window.0 as isize
        }

        fn window(handle: isize) -> HWND {
            HWND(handle as *mut std::ffi::c_void)
        }

        fn the_apartment() -> String {
            let mut kind = APTTYPE::default();
            let mut qualifier = APTTYPEQUALIFIER::default();
            // SAFETY: both pointers are to locals that outlive the call.
            match unsafe { CoGetApartmentType(&mut kind, &mut qualifier) } {
                Ok(()) => {
                    let name = match kind.0 {
                        0 => "STA",
                        1 => "MTA",
                        2 => "NA",
                        3 => "MAINSTA",
                        _ => "other",
                    };
                    format!("{name}({}) qualifier={}", kind.0, qualifier.0)
                }
                Err(why) => format!("unknown hr={}", hex(why.code())),
            }
        }

        fn the_class(field: HWND) -> String {
            let mut class = [0u16; 64];
            // SAFETY: the buffer outlives the call and its length is passed.
            let length = unsafe { GetClassNameW(field, &mut class) };
            String::from_utf16_lossy(&class[..usize::try_from(length).unwrap_or(0)])
        }

        pub(super) fn the_call(
            call: u32,
            words: &str,
            arrows: HWND,
            field: Option<HWND>,
            asked: Option<ServiceAsked>,
        ) {
            // SAFETY: takes and returns plain values.
            let thread = unsafe { GetCurrentThreadId() };
            let field_handle = field.map_or(0, handle);
            let class = field.map_or_else(String::new, the_class);
            let service = match asked {
                None => "not-asked".to_string(),
                Some(ServiceAsked::Created(instance)) => format!(
                    "created hr=0x00000000 instance={instance} first-in-process={}",
                    instance == 1
                ),
                Some(ServiceAsked::Failed(code)) => format!("failed hr={}", hex(code)),
                Some(ServiceAsked::Reused) => "reused".to_string(),
                Some(ServiceAsked::Absent) => "absent".to_string(),
            };
            tracing::warn!(
                "spin-field-naming: call={call} words=\"{words}\" thread={thread} apartment={} arrows={} field={field_handle} class=\"{class}\" service={service}",
                the_apartment(),
                handle(arrows),
            );
        }

        pub(super) fn the_write(
            call: u32,
            which: FieldProperty,
            written: &windows::core::Result<()>,
        ) {
            let property = match which {
                FieldProperty::Name => "name",
                FieldProperty::Description => "description",
            };
            let code = written
                .as_ref()
                .map_or_else(|why| why.code(), |()| HRESULT(0));
            tracing::warn!(
                "spin-field-naming: write={call} property={property} hr={}",
                hex(code)
            );
        }

        /// The field's MSAA name, read inside this process the way a
        /// screen reader asks for it from outside.
        fn msaa_name(field: HWND) -> windows::core::Result<String> {
            let child = VARIANT {
                Anonymous: VARIANT_0 {
                    Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 {
                        vt: VT_I4,
                        wReserved1: 0,
                        wReserved2: 0,
                        wReserved3: 0,
                        Anonymous: VARIANT_0_0_0 {
                            lVal: CHILDID_SELF as i32,
                        },
                    }),
                },
            };
            let mut object = std::ptr::null_mut();
            // SAFETY: the out pointer is a local; on success it holds an
            // IAccessible whose reference this takes over.
            unsafe {
                AccessibleObjectFromWindow(
                    field,
                    OBJID_CLIENT.0 as u32,
                    &IAccessible::IID,
                    &mut object,
                )?;
                let accessible = IAccessible::from_raw(object);
                Ok(accessible.get_accName(&child)?.to_string())
            }
        }

        fn the_name_now(field: HWND, words: &str) -> String {
            if reads_are_switched_off() {
                return "skipped".to_string();
            }
            match msaa_name(field) {
                Ok(name) => format!("name=\"{name}\" equals={}", name == words),
                Err(why) => format!("hr={}", hex(why.code())),
            }
        }

        pub(super) fn the_read_back(call: u32, field: HWND, words: &str) {
            tracing::warn!(
                "spin-field-naming: readback={call} {}",
                the_name_now(field, words)
            );
        }

        /// Queued to run from the dialog's own loop once it is up, holding
        /// plain handles and the words and never a widget.
        pub(super) fn check_at_show(call: u32, arrows: HWND, field: HWND, words: &str) {
            let arrows = handle(arrows);
            let named = handle(field);
            // SAFETY: reads a value from a window this program built.
            let procedure = unsafe { GetWindowLongPtrW(field, GWLP_WNDPROC) };
            let words = words.to_string();
            wxdragon::call_after(Box::new(move || {
                // SAFETY: each call takes a handle that may no longer be a
                // window, which these calls answer for rather than fault on.
                let (arrows_live, buddy, field_live, procedure_now) = unsafe {
                    (
                        IsWindow(Some(window(arrows))).as_bool(),
                        SendMessageW(window(arrows), UDM_GETBUDDY, None, None).0,
                        IsWindow(Some(window(named))).as_bool(),
                        GetWindowLongPtrW(window(named), GWLP_WNDPROC),
                    )
                };
                let name = if field_live {
                    the_name_now(window(named), &words)
                } else {
                    "gone".to_string()
                };
                tracing::warn!(
                    "spin-field-naming: at-show={call} arrows_live={arrows_live} buddy={buddy} same_field={} field_live={field_live} same_procedure={} {name}",
                    buddy == named,
                    procedure_now == procedure,
                );
            }));
        }
    }
}

/// Everywhere else the arrows are all there is to name, and
/// [`A_NAME_REACHES_THE_ACCESSIBILITY_TREE`] says that does nothing either.
#[cfg(not(target_os = "windows"))]
mod typing_field {
    use super::FieldProperty;
    use wxdragon::prelude::SpinCtrl;

    pub(super) fn carry(_spin: &SpinCtrl, _carries: &[(FieldProperty, &str)]) {}
}

/// Hold one cell of a two-column grid open with nothing in it.
///
/// A checkbox carries its own label, so in a grid of label and field pairs
/// the label column beside it is empty. That cell used to be filled with a
/// `StaticText` built on `""`, which is a real window: it reaches both
/// accessibility channels as a nameless control, sitting straight before the
/// checkbox in the order Tab moves in, and it is the window Windows picks
/// when it names a control that set no name from the nearest static text.
/// Two testers met an unnamed checkbox built that way on the first day of
/// testing (#42, #40). A sizer spacer takes up the cell and is not a window
/// at all, so nothing nameless is in the tree.
///
/// One pixel rather than none, because `wxdragon` adds nothing at all for a
/// spacer of size zero and the cells after it would shift left a column. In
/// a grid with a growable field column a pixel is not visible.
pub fn leave_the_cell_empty(grid: &wxdragon::sizers::FlexGridSizer) {
    grid.add_spacer(1);
}

#[cfg(test)]
mod tests {
    use super::{
        AccessibleImpl, FieldProperty, FixedName, name_from_label, what_the_typing_field_carries,
    };
    use wxdragon::ffi;

    #[test]
    fn test_a_spin_controls_description_travels_to_the_field_a_person_types_in() {
        // Tab lands in the field, never on the arrows, so a sentence the
        // arrows alone carry is one nobody tabbing through a form hears. The
        // event form's Alert minutes before has one, "Nought for no alert".
        assert_eq!(
            what_the_typing_field_carries("Alert minutes before", Some("Nought for no alert")),
            vec![
                (FieldProperty::Name, "Alert minutes before"),
                (FieldProperty::Description, "Nought for no alert"),
            ]
        );
    }

    #[test]
    fn test_the_name_is_the_only_thing_replaced() {
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(0);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(name.as_deref(), Some("Messages"));
    }

    #[test]
    fn test_children_name_themselves() {
        // A list row, a tree node and a notebook tab each ask for their own
        // name through the parent. Answering with the control's name labels
        // every one of them identically.
        let named = FixedName {
            name: "Settings categories".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(1);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(name.is_none());
    }

    #[test]
    fn test_a_description_is_offered_for_the_control_itself() {
        // The bug this was written for: the first-run screen's three choices
        // each had a sentence beside them saying what it costs, and a screen
        // reader never read any of them. The label was named, the explanation
        // was a separate piece of text nobody was pointed at, and somebody
        // choosing what Wixen Mail may change heard only "read my mail,
        // change nothing" with no idea what the other two did.
        let described = FixedName {
            name: "Read my mail, change nothing".to_string(),
            description: Some("Nothing you do here reaches your provider.".to_string()),
        };

        let (status, description) = described.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(
            description.as_deref(),
            Some("Nothing you do here reaches your provider.")
        );
    }

    #[test]
    fn test_a_control_with_no_description_leaves_the_question_to_the_platform() {
        // NOT_IMPLEMENTED rather than an empty string. Answering with nothing
        // is a claim that there is no description, which stops wxWidgets
        // supplying whatever the control would have said for itself.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };

        let (status, description) = named.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_children_describe_themselves() {
        // Same trap as the name. Handing every row of a list the control's
        // description would have each one read it out.
        let described = FixedName {
            name: "Messages".to_string(),
            description: Some("Your inbox".to_string()),
        };

        let (status, description) = described.get_description(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_child_enumeration_is_left_to_the_control() {
        // The trait default answers OK with zero children, which is a claim
        // that the control is empty rather than a request to fall back. With
        // that default in place a named notebook lost its tabs and a named
        // list would have lost every row.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, count) = named.get_child_count();
        assert_eq!(
            status,
            ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED,
            "naming a control must not claim it has no children"
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_strips_the_mnemonic_and_turns_the_colon_into_a_pause() {
        // Not dropped outright: a real NVDA run found nothing left between
        // the name and whatever is read next, the field's role and then its
        // value, ran together with no pause. A comma is never read aloud as
        // a word, and is still a pause to the synthesiser underneath it.
        assert_eq!(name_from_label("&Subject:"), "Subject,");
        assert_eq!(
            name_from_label("Start &Date (YYYY-MM-DD):"),
            "Start Date (YYYY-MM-DD),"
        );
    }

    #[test]
    fn test_leaves_a_plain_label_alone() {
        // No colon on the label, so nothing here to turn into a pause.
        assert_eq!(name_from_label("Accounts"), "Accounts");
    }

    #[test]
    fn test_handles_a_label_that_is_only_decoration() {
        assert_eq!(name_from_label(":"), "");
        assert_eq!(name_from_label("   "), "");
    }

    #[test]
    fn test_a_doubled_ampersand_is_the_one_a_label_really_means() {
        // wxWidgets writes a literal ampersand as two of them, so deleting
        // every ampersand takes the word out of the label and leaves a double
        // space behind it. This is the composer's own send confirmation, the
        // button that goes back to a message nobody has sent yet.
        assert_eq!(name_from_label("&Go Back && Edit"), "Go Back & Edit");
        assert!(
            !name_from_label("&Go Back && Edit").contains("  "),
            "a double space is an odd pause in the middle of the button"
        );
        // The same shape on the notebook pages of the contact and settings
        // windows.
        assert_eq!(name_from_label("Email && Phone"), "Email & Phone");
    }

    #[test]
    fn test_only_one_trailing_colon_is_a_visual_convention() {
        // One colon after a field label is how a form is written, and becomes
        // a comma. A second one is something the label says, and is left as
        // a colon rather than turned into something the label never had.
        assert_eq!(name_from_label("Ratio::"), "Ratio:,");
    }
}

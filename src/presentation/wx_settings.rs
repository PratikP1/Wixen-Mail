//! Settings / Preferences dialog
//!
//! A tabbed dialog accessible from Tools > Settings that exposes the most
//! commonly used email client configuration options.  Settings are read from
//! and persisted through `AppConfig` / `ConfigManager`.

use crate::application::autosave::AutosaveInterval;
use crate::application::conversations::{AConversationReaches, DeletingAConversationRow};
use crate::application::describing_pictures::{
    UNDESCRIBED_PICTURES_LABEL, UndescribedPicture, WHAT_THE_CHOICE_LEAVES_ALONE,
};
use crate::application::folder_settings::{self, UnreadOnAParent};
use crate::application::opening_links::Where as OpenLinks;
use crate::application::reading_habits::{
    CopyLines, MarkRead, MarkReadWay, TAKES_EFFECT_AT_THE_NEXT_START, WHAT_MARK_READ_COUNTS_FROM,
    WHERE_THE_DEFAULT_SORT_ORDER_APPLIES, WorkingDay,
};
use crate::application::reading_style::Style as ReadingStyle;
use crate::application::receipts::Policy;
use crate::application::time_blocks::Block;
use crate::common::paths::AppPaths;
use crate::data::account::Account;
use crate::data::config::AppConfig;
use crate::presentation::accessibility::Accessibility;
// `Event` is reached through the module rather than imported, because
// `wxdragon::prelude` brings its own `Event` and the two would shadow.
use crate::presentation::accessibility::feedback::{self, Channel, FeedbackSettings, Switch};
use crate::presentation::accessibility::names::{
    name_and_describe_the_spin_control, name_from_label, name_the_spin_control,
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::accessibility::sound_scheme::SoundScheme;
use crate::presentation::accessibility::sound_scheme_import;
use crate::presentation::theme;
use crate::presentation::ui_types::CalendarView;
use crate::presentation::which_language_row::{RowToShow, which_row_shows};
use crate::service::spellcheck::{LanguageChoice, available_languages, system_language};
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use wxdragon::prelude::*;

// ── Result type ──────────────────────────────────────────────────────────────

/// The outcome of the settings dialog.
pub enum SettingsResult {
    /// User pressed OK, carrying the possibly modified configuration.
    ///
    /// Boxed because the configuration dwarfs the other variant, and this
    /// grew past the point where every Cancelled paid for it.
    Updated(Box<AppConfig>),
    /// User cancelled: no changes.
    Cancelled,
}

// ── The tab row's arrow keys ─────────────────────────────────────────────────

/// Which way an arrow key moves along the tab row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Along {
    /// Towards the first tab: Left, or Up.
    Back,
    /// Towards the last tab: Right, or Down.
    Forward,
}

impl Along {
    /// The way a wxWidgets key code moves along the row, or `None` for a key
    /// that is not an arrow. Left and Up move back, Right and Down forward,
    /// which is how the native tab control reads them; the numpad's arrows
    /// with Num Lock off reach Windows as the same virtual keys without the
    /// extended bit, wxWidgets gives them codes of their own, and the
    /// control reads those the same way too.
    pub fn from_key_code(code: i32) -> Option<Self> {
        const WXK_LEFT: i32 = 314;
        const WXK_UP: i32 = 315;
        const WXK_RIGHT: i32 = 316;
        const WXK_DOWN: i32 = 317;
        const WXK_NUMPAD_LEFT: i32 = 376;
        const WXK_NUMPAD_UP: i32 = 377;
        const WXK_NUMPAD_RIGHT: i32 = 378;
        const WXK_NUMPAD_DOWN: i32 = 379;
        match code {
            WXK_LEFT | WXK_UP | WXK_NUMPAD_LEFT | WXK_NUMPAD_UP => Some(Self::Back),
            WXK_RIGHT | WXK_DOWN | WXK_NUMPAD_RIGHT | WXK_NUMPAD_DOWN => Some(Self::Forward),
            _ => None,
        }
    }
}

/// The tab an arrow reaches from `current` in a row of `count`, or `None` at
/// either end: the row does not wrap, as the native control's does not.
pub fn the_tab_an_arrow_reaches(current: usize, count: usize, along: Along) -> Option<usize> {
    match along {
        Along::Back => current.checked_sub(1),
        Along::Forward => (current + 1 < count).then_some(current + 1),
    }
}

/// The tab row answers its own arrow keys, so each tab is raised once.
///
/// #33: the reached tab was often spoken twice. `scripts/uia-events.ps1`,
/// run on 2026-09-16 against the release build at `96298371`, showed why on
/// the channel NVDA reads for a native tab control: the control's own
/// arrow-key handler in comctl32 raises `EVENT_OBJECT_SELECTION` once and
/// then `EVENT_OBJECT_FOCUS` twice on the same tab, one millisecond apart,
/// and a screen reader that flushes its event queue between the two speaks
/// the tab for each. Moving the selection through `TCM_SETCURSEL` raised the
/// focus event once. wxWidgets' `SetSelection` goes that way, so an arrow is
/// taken here, the selection moved through it, and the key not passed on;
/// the page-changing and page-changed events are sent as before, and focus
/// stays on the row as it does natively. A key with a modifier held, and any
/// key that is not an arrow, is left to the control.
fn answer_the_arrows_on(notebook: &Notebook) {
    notebook.on_key_down({
        let notebook = *notebook;
        move |event| {
            let WindowEventData::Keyboard(ref key) = event else {
                return;
            };
            if key.control_down() || key.shift_down() || key.alt_down() {
                return;
            }
            let Some(along) = key.get_key_code().and_then(Along::from_key_code) else {
                return;
            };
            event.skip(false);
            let Ok(current) = usize::try_from(notebook.selection()) else {
                return;
            };
            if let Some(reached) =
                the_tab_an_arrow_reaches(current, notebook.get_page_count(), along)
            {
                notebook.set_selection(reached);
            }
        }
    });
}

// ── Widget references ────────────────────────────────────────────────────────

/// Holds references to all mutable settings widgets so we can read them back
/// when the user presses OK.
///
/// Public, and every field with it, so a test can build the dialog without
/// showing it and read back what a live control was really painted: the same
/// shape `CalendarPanelHandles` and `ReaderTabHandles` already use for the
/// eight surfaces round 25 could reach directly.
pub struct SettingsWidgets {
    /// The dialog itself, and the notebook that tabs it, so both can be
    /// painted and both can be read back by a test.
    pub dialog: Dialog,
    pub notebook: Notebook,
    // Which panel backs which tab, so a test can check each one was painted
    // without walking the notebook's own children to find it.
    pub general_panel: Panel,
    pub compose_panel: Panel,
    pub reading_panel: Panel,
    pub permissions_panel: Panel,
    pub pim_panel: Panel,
    pub feedback_panel: Panel,
    pub advanced_panel: Panel,
    // General, the page the dialog opens on, built before it is shown.
    theme: Choice,
    pub font_size: SpinCtrl,
    pub font_family: Choice,
    smooth_scrolling: CheckBox,
    keep_selected_message_in_view: CheckBox,
    keep_running_in_the_tray: CheckBox,
    check_default_programs_at_startup: CheckBox,
    which_updates: Choice,
    // Language
    language: Choice,
    check_spelling_before_send: CheckBox,
    check_spelling_as_you_type: CheckBox,
    /// The six pages after General, each built when its tab is first
    /// shown. Shared with the tab row's page-changed handler, which is what
    /// builds them in the running program.
    later: Rc<LaterPages>,
}

impl SettingsWidgets {
    /// The Compose page's controls, built now if its tab was never shown.
    pub fn compose(&self) -> &ComposeTabControls {
        self.later.compose()
    }

    /// The Reading page's controls, built now if its tab was never shown.
    pub fn reading(&self) -> &ReadingTabControls {
        self.later.reading()
    }

    /// The Permissions page's controls, built now if its tab was never shown.
    pub fn permissions(&self) -> &PermissionsTabControls {
        self.later.permissions()
    }

    /// The Calendar & PIM page's controls, built now if its tab was never
    /// shown.
    pub fn calendar_and_pim(&self) -> &CalendarPimTabControls {
        self.later.calendar_and_pim()
    }

    /// The Feedback page's controls, built now if its tab was never shown.
    pub fn feedback(&self) -> &FeedbackTabControls {
        self.later.feedback()
    }

    /// The Advanced page's controls, built now if its tab was never shown.
    pub fn advanced(&self) -> &AdvancedTabControls {
        self.later.advanced()
    }
}

// ── The pages built when their tab is first shown ────────────────────────────

/// Which tab is which, in the order `build_settings_dialog` adds them.
/// General is tab 0 and is built before the dialog is shown, so nothing
/// here names it.
const THE_COMPOSE_TAB: usize = 1;
const THE_READING_TAB: usize = 2;
const THE_PERMISSIONS_TAB: usize = 3;
const THE_CALENDAR_AND_PIM_TAB: usize = 4;
const THE_FEEDBACK_TAB: usize = 5;
const THE_ADVANCED_TAB: usize = 6;

/// One page: its panel, which is in the notebook from the start so the tab
/// row is complete, and its controls, which arrive the first time the tab
/// is shown.
struct APage<T> {
    panel: Panel,
    controls: OnceCell<T>,
}

impl<T> APage<T> {
    fn on(panel: Panel) -> Self {
        Self {
            panel,
            controls: OnceCell::new(),
        }
    }

    /// The controls, built now if they were not.
    fn built(&self, build: impl FnOnce(&Panel) -> T) -> &T {
        self.controls.get_or_init(|| build(&self.panel))
    }

    /// The controls only if the page was built; `None` for a tab nobody
    /// showed, whose settings are then written back as they were stored.
    fn if_built(&self) -> Option<&T> {
        self.controls.get()
    }
}

/// The six pages after General, and what each is built from.
///
/// Before 09-09 every page was built before the dialog was shown, and the
/// tester found the pause (#34). Measured on 2026-09-16 with the window
/// shown and a screen reader running, the seven pages of controls were the
/// whole of a 2.2 second build, and the three lists the issue named cost a
/// millisecond each. So the page the dialog opens on is built before it is
/// shown, the others when their tab is first reached, and a page nobody
/// reaches is read back from the settings it would have shown.
struct LaterPages {
    /// What a page is built from: the configuration as it was when the
    /// dialog opened, the accounts and whether the default one has a
    /// calendar server, for the Notes section, the accessibility handle the
    /// Feedback page announces through, and the palette the dialog was
    /// painted with, so a page built later is painted the same.
    config: AppConfig,
    accounts: Vec<Account>,
    a_calendar_server: bool,
    a11y: Arc<Accessibility>,
    palette: Option<theme::Palette>,
    compose: APage<ComposeTabControls>,
    reading: APage<ReadingTabControls>,
    permissions: APage<PermissionsTabControls>,
    calendar_and_pim: APage<CalendarPimTabControls>,
    feedback: APage<FeedbackTabControls>,
    advanced: APage<AdvancedTabControls>,
}

impl LaterPages {
    /// Build the page behind `tab` if it is not built yet, frozen while its
    /// controls arrive. `wxChoice` resizes itself after every item it is
    /// handed unless it is frozen, and a child added under a frozen window
    /// is frozen with it, so a page with a long list on it costs a fraction
    /// of what it costs unfrozen. The General tab and a tab that is not a
    /// page are nothing to do here.
    fn build_the_page_for(&self, tab: usize) {
        let Some(panel) = self.panel_of(tab) else {
            return;
        };
        if self.is_built(tab) {
            return;
        }
        panel.freeze();
        let first_control: Option<&dyn WxWidget> = match tab {
            THE_COMPOSE_TAB => Some(self.compose().first_in_tab_order()),
            THE_READING_TAB => Some(self.reading().first_in_tab_order()),
            THE_PERMISSIONS_TAB => Some(self.permissions().first_in_tab_order()),
            THE_CALENDAR_AND_PIM_TAB => Some(self.calendar_and_pim().first_in_tab_order()),
            THE_FEEDBACK_TAB => Some(self.feedback().first_in_tab_order()),
            THE_ADVANCED_TAB => Some(self.advanced().first_in_tab_order()),
            _ => None,
        };
        // The panel already has the size the notebook gave it, so its new
        // controls are placed now rather than at the dialog's next layout,
        // which may never come.
        panel.layout();
        panel.thaw();
        // `wxNotebook::SetSelection` gives the reached page focus before it
        // sends the event this runs on, when the tab row does not hold focus
        // (Ctrl+Tab from inside a page, `src/msw/notebook.cpp:364-391`). The
        // page was empty then, so the focus landed on the panel itself and a
        // screen reader spoke an unnamed pane (#68, 2026-09-17). Asking the
        // panel to move it on is refused, because the panel already has it
        // (`src/common/containr.cpp:110-150`), so the page names its first
        // control. When the row holds focus nothing is touched, which is
        // what keeps #33's single focus event. Asked after the thaw, so the
        // answer is about the window as it will be shown.
        if panel.has_focus()
            && let Some(first) = first_control
        {
            first.set_focus();
        }
    }

    fn panel_of(&self, tab: usize) -> Option<&Panel> {
        match tab {
            THE_COMPOSE_TAB => Some(&self.compose.panel),
            THE_READING_TAB => Some(&self.reading.panel),
            THE_PERMISSIONS_TAB => Some(&self.permissions.panel),
            THE_CALENDAR_AND_PIM_TAB => Some(&self.calendar_and_pim.panel),
            THE_FEEDBACK_TAB => Some(&self.feedback.panel),
            THE_ADVANCED_TAB => Some(&self.advanced.panel),
            _ => None,
        }
    }

    fn is_built(&self, tab: usize) -> bool {
        match tab {
            THE_COMPOSE_TAB => self.compose.if_built().is_some(),
            THE_READING_TAB => self.reading.if_built().is_some(),
            THE_PERMISSIONS_TAB => self.permissions.if_built().is_some(),
            THE_CALENDAR_AND_PIM_TAB => self.calendar_and_pim.if_built().is_some(),
            THE_FEEDBACK_TAB => self.feedback.if_built().is_some(),
            THE_ADVANCED_TAB => self.advanced.if_built().is_some(),
            _ => false,
        }
    }

    // Each accessor paints its page's panel after `build_*_tab` returns and
    // before the controls are handed back, inside `built`, so the panel is
    // painted exactly once, on the first build, on both paths: the tab row's
    // page-changed handler through `build_the_page_for`, and a caller of the
    // public accessors. The order is the whole point: a child created under
    // a panel that already carries a foreground colour inherits it at
    // creation (wxWidgets `src/common/wincmn.cpp:1524-1552`), and a checkbox
    // given a colour is made owner-drawn (`src/msw/control.cpp:422-444`),
    // which Windows' accessible object reports as a push button with no
    // checked state. Painted after its controls exist, a panel hands its
    // colour to nothing (#67, found 2026-09-17 in `1.0.0-alpha.1`).

    fn compose(&self) -> &ComposeTabControls {
        self.compose.built(|panel| {
            let controls = build_compose_tab(panel, &self.config);
            self.paint(&self.compose.panel);
            controls
        })
    }

    fn reading(&self) -> &ReadingTabControls {
        self.reading.built(|panel| {
            let controls = build_reading_tab(panel, &self.config);
            self.paint(&self.reading.panel);
            controls
        })
    }

    fn permissions(&self) -> &PermissionsTabControls {
        self.permissions.built(|panel| {
            let controls = build_permissions_tab(panel, &self.config);
            self.paint(&self.permissions.panel);
            controls
        })
    }

    fn calendar_and_pim(&self) -> &CalendarPimTabControls {
        self.calendar_and_pim.built(|panel| {
            let controls =
                build_calendar_pim_tab(panel, &self.config, &self.accounts, self.a_calendar_server);
            self.paint(&self.calendar_and_pim.panel);
            self.paint(&controls.default_reminder);
            controls
        })
    }

    fn feedback(&self) -> &FeedbackTabControls {
        self.feedback.built(|panel| {
            let controls = build_feedback_tab(panel, &self.config, &self.a11y);
            self.paint(&self.feedback.panel);
            controls
        })
    }

    fn advanced(&self) -> &AdvancedTabControls {
        self.advanced.built(|panel| {
            let controls = build_advanced_tab(panel, &self.config);
            self.paint(&self.advanced.panel);
            self.paint(&controls.download_folder);
            controls
        })
    }

    /// Paint a window the way the dialog was painted when it was built: a
    /// page's panel once its controls exist, and the text fields, which are
    /// the controls whose colour the theme sets by hand. `None` means Windows
    /// decides and nothing is set.
    fn paint(&self, window: &(impl WxWidget + ?Sized)) {
        if let Some(palette) = self.palette {
            theme::paint(window, palette.main_surface());
        }
    }
}

/// Helper: unwrap get_selection() returning 0 if None.
fn sel(choice: &Choice) -> u32 {
    choice.get_selection().unwrap_or(0)
}

/// The sizes Font size holds and the minutes Default reminder holds: the
/// bounds each save applied after the fact until 12-06, now the controls' own
/// (#35), so Up and Down stop at them and a screen reader can say them.
const FONT_SIZES: (i32, i32) = (8, 72);
const REMINDER_MINUTES: (i32, i32) = (0, 1440);

/// A stored number as a spin control with `range` shows it: a hand-edited
/// file holding a number out of range opens on the nearest end.
fn within(stored: u32, (least, most): (i32, i32)) -> i32 {
    i32::try_from(stored).unwrap_or(most).clamp(least, most)
}

/// The number a spin control holds. Every spin control in this dialog has a
/// range starting at nought or above, so nothing is lost converting it and
/// nothing parses a number the control already holds.
fn held(spin: &SpinCtrl) -> u32 {
    spin.value().unsigned_abs()
}

/// Which way of marking read the Mark as read after choice has chosen.
fn chosen_way(choice: &Choice) -> MarkReadWay {
    MarkReadWay::ALL
        .get(sel(choice) as usize)
        .copied()
        .unwrap_or(MarkRead::default().parts().0)
}

/// What the Mark as read after choice and its seconds are called on both
/// channels.
const MARK_READ_AFTER: &str = "Mark as read after";
const MARK_READ_SECONDS: &str = "Mark as read after, in seconds";

// ── Section helper ───────────────────────────────────────────────────────────

/// Create a labelled section sizer using StaticBoxSizerBuilder::new_with_label.
fn section(parent: &Panel, label: &str) -> StaticBoxSizer {
    StaticBoxSizerBuilder::new_with_label(Orientation::Vertical, parent, label).build()
}

/// A sentence under a control, named on both channels, the way the sentence
/// under Mark as read after is built: a static text whose accessible name is
/// its words, so a screen reader meets it in the tab order after the control
/// it is about and hears what the control cannot say on its own.
fn a_sentence_under(parent: &Panel, section: &StaticBoxSizer, words: &str) {
    let note = StaticText::builder(parent).with_label(words).build();
    set_accessible_name(&note, words);
    section.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 4);
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Show the Settings dialog and return the (possibly updated) configuration.
///
/// `asked_at` is the instant the command arrived, taken at the top of the
/// window's `ID_SETTINGS` arm before the configuration was read, so the
/// line written here spans everything between the key and the show. The
/// line is the measurement behind #34 and the harness in
/// `tests/the_settings_dialog_opens_in.rs` reads it off a real log.
pub fn show_settings_dialog(
    parent: &Frame,
    config: &AppConfig,
    accounts: &[Account],
    a_calendar_server: bool,
    a11y: &Arc<Accessibility>,
    asked_at: Instant,
) -> SettingsResult {
    let widgets = build_settings_dialog(parent, config, accounts, a_calendar_server, a11y);
    tracing::info!(
        "{}",
        crate::common::started::settings_built_line(asked_at.elapsed())
    );
    if widgets.dialog.show_modal() != ID_OK {
        return SettingsResult::Cancelled;
    }
    // An answer this cannot keep is said rather than written over in
    // silence. A working day that runs past midnight is refused, which is a
    // real limit, and the screen used to put nine to five back with nothing
    // said at all: somebody choosing a night shift set it, heard nothing, and
    // found the built-in day again the next time they looked.
    //
    // A Calendar & PIM tab nobody showed holds no answer to refuse: what is
    // written back for it is what was stored.
    let working_day_as_shown = widgets
        .later
        .calendar_and_pim
        .if_built()
        .map(|pim| (sel(&pim.day_starts) as u8, sel(&pim.day_ends) as u8));
    if working_day_as_shown
        .is_some_and(|(starts, ends)| WorkingDay::could_not_be_used(starts, ends))
    {
        let _ = a11y.announce(
            "A working day that runs past midnight cannot be kept, so the \
             working day is unchanged. Everything else you changed is saved.",
            crate::presentation::accessibility::announcements::Priority::High,
        );
    }
    SettingsResult::Updated(Box::new(read_settings(&widgets, config)))
}

/// Build the Settings dialog without showing it.
///
/// Everything `show_settings_dialog` used to do up to its own
/// `.show_modal()` call, split out so a test can build the real dialog, read
/// back the real colour a real control is holding, and never call
/// `.show_modal()` at all. Round 25 used this shape for the eight surfaces it
/// could reach directly; this is the same shape reaching a standalone dialog
/// for the first time.
///
/// Takes `&AppConfig` rather than a separate palette argument: Settings is
/// the one dialog in this application that already holds, in `config`, the
/// exact settings it is about to display and let somebody change, so it asks
/// [`theme::current`] with that value directly rather than through
/// [`theme::current_from_stored_config`], which would mean a second,
/// independent disk read that could in principle disagree with the config
/// already in hand, including the very Theme dropdown this dialog is about
/// to build.
/// `accounts` is the list this window is showing, and it is here for one
/// section. `AppConfig` names the default account by id and holds no roster,
/// and the accounts themselves live in the message cache, which this dialog
/// has no handle on. Where an account's notes go is answered from its
/// provider, so the id alone cannot answer it and the accounts have to arrive
/// with the configuration.
/// `a_calendar_server` is the other half of that same answer, and it arrives
/// the same way and for the same reason: whether the default account has a
/// calendar on a calendar server is a row in the message cache, which this
/// dialog has no handle on either.
pub fn build_settings_dialog(
    parent: &Frame,
    config: &AppConfig,
    accounts: &[Account],
    a_calendar_server: bool,
    a11y: &Arc<Accessibility>,
) -> SettingsWidgets {
    let dlg = Dialog::builder(parent, "Settings")
        .with_size(560, 520)
        .build();
    // Frozen until it is laid out. A child added under a frozen window is
    // frozen with it, and a frozen `wxChoice` takes its items without
    // resizing itself after each one: measured on 2026-09-16 with the
    // parent window shown, the typeface list alone cost about a second
    // unfrozen and a tenth of that frozen (#34). Nothing is drawn before the
    // dialog is shown either way, so nothing is lost by it.
    dlg.freeze();

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Notebook (tabbed pane)
    let notebook = Notebook::builder(&dlg).build();
    answer_the_arrows_on(&notebook);

    // ── Tab 1: General, the page the dialog opens on, so built now
    let general_panel = Panel::builder(&notebook).build();
    let GeneralTabControls {
        theme,
        font_family,
        font_size,
        language,
        check_before_send: check_spelling_before_send,
        check_as_you_type: check_spelling_as_you_type,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        choose_default_programs,
        check_default_programs_at_startup,
        which_updates,
    } = build_general_tab(&general_panel, config);
    notebook.add_page(&general_panel, "General", true, None);

    // ── Tabs 2 to 7: in the tab row from the start, so every tab is
    // there to be reached and read, and each built the first time its tab
    // is shown. The order here is the order the constants above name.
    let compose_panel = Panel::builder(&notebook).build();
    notebook.add_page(&compose_panel, "Compose", false, None);
    let reading_panel = Panel::builder(&notebook).build();
    notebook.add_page(&reading_panel, "Reading", false, None);
    let permissions_panel = Panel::builder(&notebook).build();
    notebook.add_page(&permissions_panel, "Permissions", false, None);
    let pim_panel = Panel::builder(&notebook).build();
    notebook.add_page(&pim_panel, "Calendar && PIM", false, None);
    let feedback_panel = Panel::builder(&notebook).build();
    notebook.add_page(&feedback_panel, "Feedback", false, None);
    let advanced_panel = Panel::builder(&notebook).build();
    notebook.add_page(&advanced_panel, "Advanced", false, None);

    // Computed from `config` rather than a fresh disk read (see this
    // function's own doc comment for why). `None` means high contrast is
    // on, or the system is set up in a way this application should not
    // paint over, so nothing is set and Windows decides.
    let palette = theme::current(&config.theme);
    let later = Rc::new(LaterPages {
        config: config.clone(),
        accounts: accounts.to_vec(),
        a_calendar_server,
        a11y: Arc::clone(a11y),
        palette,
        compose: APage::on(compose_panel),
        reading: APage::on(reading_panel),
        permissions: APage::on(permissions_panel),
        calendar_and_pim: APage::on(pim_panel),
        feedback: APage::on(feedback_panel),
        advanced: APage::on(advanced_panel),
    });
    notebook.on_page_changed({
        let later = Rc::clone(&later);
        move |event| {
            if let Some(reached) = event
                .get_selection()
                .and_then(|tab| usize::try_from(tab).ok())
            {
                later.build_the_page_for(reached);
            }
        }
    });

    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── OK / Cancel buttons
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add_spacer(0);
    let ok_btn = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel_btn = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_sizer.add(&ok_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&cancel_btn, 0, SizerFlag::All, 4);
    root_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dlg.set_sizer(root_sizer, true);

    // Windows is where this is chosen, and this is the only thing an
    // application is allowed to do about it: take somebody there. The failure
    // is reported rather than swallowed, because a button that opens nothing
    // and says nothing is indistinguishable from a button that does not work.
    choose_default_programs.on_click({
        let d = dlg;
        move |_ev| {
            if let Err(why) = crate::service::default_apps::open_windows_default_apps_page() {
                tracing::warn!("The Windows default programs page would not open: {why}");
                // A message box rather than an announcement: a screen reader
                // reads one out on its own, and this dialog carries no
                // accessibility handle to announce through.
                let said = MessageDialog::builder(
                    &d,
                    "The Windows settings page for default programs would not open. \
                     It is under Settings, Apps, Default apps.",
                    "Could not open Windows settings",
                )
                .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconInformation)
                .build();
                said.show_modal();
            }
        }
    });

    ok_btn.on_click({
        let d = dlg;
        move |_ev| {
            d.end_modal(ID_OK);
        }
    });
    cancel_btn.on_click({
        let d = dlg;
        move |_ev| {
            d.end_modal(ID_CANCEL);
        }
    });

    // General is painted here because it was built here, after its controls
    // exist. A page built later is painted at the end of its own build in
    // `LaterPages`, after its controls exist, and not before: a child created
    // under a panel that already carries a foreground colour inherits it at
    // creation (wxWidgets `src/common/wincmn.cpp:1524-1552`), and a checkbox
    // given a colour is made owner-drawn (`src/msw/control.cpp:422-444`),
    // which Windows' accessible object reports as a push button with no
    // checked state. From 09-09 until 10-01.1 the six empty panels were
    // painted here, and every checkbox built on them later read as a button
    // under NVDA (#67, found 2026-09-17 in `1.0.0-alpha.1`).
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&notebook, palette.main_surface());
        theme::paint(&general_panel, palette.main_surface());
        theme::paint(&font_size, palette.main_surface());
    }

    // Laid out, and only then thawed: a thaw on a window with a sizer
    // redraws what the sizer has placed, and before the dialog is shown
    // there is nothing to redraw.
    dlg.thaw();

    SettingsWidgets {
        dialog: dlg,
        notebook,
        general_panel,
        compose_panel: later.compose.panel,
        reading_panel: later.reading.panel,
        permissions_panel: later.permissions.panel,
        pim_panel: later.calendar_and_pim.panel,
        feedback_panel: later.feedback.panel,
        advanced_panel: later.advanced.panel,
        theme,
        font_size,
        font_family,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        check_default_programs_at_startup,
        which_updates,
        language,
        check_spelling_before_send,
        check_spelling_as_you_type,
        later,
    }
}

// ── Tab builders ─────────────────────────────────────────────────────────────

/// General settings: theme and font size.
/// How views move: whether they slide, and whether they follow the cursor.
///
/// Smooth scrolling says outright that Windows can overrule it, because it
/// can. [`crate::application::scrolling`] holds that rule and the reason: an
/// animation setting somebody has already made once should not have to be made
/// again in every program, and the way they would find out this one ignored
/// them is by being made unwell.
fn add_scrolling(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> (CheckBox, CheckBox) {
    use crate::application::scrolling::{system_motion, what_the_machine_has_overruled};

    let scroll_sec = section(panel, "Scrolling");

    let smooth = CheckBox::builder(panel)
        .with_label("&Slide when the message you are reading scrolls, rather than jumping")
        .build();
    set_accessible_name_and_description(
        &smooth,
        "Slide when the message you are reading scrolls, rather than jumping",
        "The message body only. Windows overrules this when it is set to \
         reduce animation",
    );
    smooth.set_value(config.smooth_scrolling);
    scroll_sec.add(&smooth, 0, SizerFlag::All, 4);

    // Said only on a machine it is true of, and only to somebody it is true
    // of, rather than as a permanent caveat that the people it applies to
    // cannot pick out from the people it does not. A ticked box doing nothing
    // otherwise reads as a broken program; an unticked one being reported as
    // overruled names a fight nobody was in.
    //
    // Both halves are the one question and it is asked in one place. This used
    // to ask the machine here and then hand the answer a literal `true`, so the
    // second half was decided by nobody and the note went up for everybody.
    if let Some(said) = what_the_machine_has_overruled(config.smooth_scrolling, system_motion()) {
        let note = StaticText::builder(panel).with_label(&said).build();
        set_accessible_name(&note, &said);
        scroll_sec.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 4);
    }

    let keep_in_view = CheckBox::builder(panel)
        .with_label("&Keep the chosen message in view when the list reloads")
        .build();
    set_accessible_name_and_description(
        &keep_in_view,
        "Keep the chosen message in view when the list reloads",
        "Turning this off leaves the view where it is when a sync finishes. \
         Your place in the list is not lost either way",
    );
    keep_in_view.set_value(config.keep_selected_message_in_view);
    scroll_sec.add(&keep_in_view, 0, SizerFlag::All, 4);

    sizer.add_sizer(&scroll_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (smooth, keep_in_view)
}

/// Which program Windows opens email, calendar files and contact cards with.
///
/// Read rather than set. Since Windows 8 an application cannot make itself the
/// default, so there is no button here that does it: there is a list saying
/// what is set now, and a button that takes somebody to the Windows screen
/// where they choose. A button promising more would be one that cannot keep
/// its promise, which is guardrail 3.
///
/// Six rows, including the three Windows keeps no default for. Leaving those
/// out would leave somebody looking for tasks in this list wondering whether
/// they had missed it.
fn add_default_programs(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> (Button, CheckBox) {
    use crate::presentation::default_app_words::{button_label, row, why_windows_asks};
    use crate::service::default_apps::{DefaultKind, is_default};

    let programs_sec = section(panel, "Default programs");

    let why = StaticText::builder(panel)
        .with_label(why_windows_asks())
        .build();
    set_accessible_name(&why, why_windows_asks());
    programs_sec.add(&why, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Each row is one whole sentence rather than a name and a state in two
    // columns, because a screen reader reads a row and the halves have to
    // arrive together to mean anything.
    for kind in DefaultKind::ALL {
        let said = row(kind, &is_default(kind));
        let line = StaticText::builder(panel).with_label(&said).build();
        set_accessible_name(&line, &said);
        programs_sec.add(&line, 0, SizerFlag::Left | SizerFlag::All, 2);
    }

    let choose = Button::builder(panel).with_label(button_label()).build();
    set_accessible_name_and_description(
        &choose,
        "Choose default programs in Windows",
        "Opens the Windows settings screen at Wixen Mail, where you choose \
         which file types and links it opens. Wixen Mail cannot change this \
         itself",
    );
    programs_sec.add(&choose, 0, SizerFlag::Left | SizerFlag::All, 4);

    let check_at_startup = CheckBox::builder(panel)
        .with_label("Check this &every time Wixen Mail starts")
        .build();
    set_accessible_name_and_description(
        &check_at_startup,
        "Check this every time Wixen Mail starts",
        "Says so at startup when another program holds one of these. Off by \
         default, because somebody who chose another program on purpose does \
         not need telling about it every time",
    );
    check_at_startup.set_value(config.check_default_programs_at_startup);
    programs_sec.add(&check_at_startup, 0, SizerFlag::Left | SizerFlag::All, 4);

    sizer.add_sizer(&programs_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (choose, check_at_startup)
}

/// What closing the window does.
///
/// Off by default, and deliberately so. Closing a window and having the program
/// carry on is not what closing a window means to most people, and it is worse
/// than most for somebody who cannot see the screen: the window goes, the
/// reading stops, and nothing says the program is still there. So it is asked
/// for rather than assumed, it is announced the first time it happens, and Quit
/// always really quits.
fn add_closing(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> CheckBox {
    let close_sec = section(panel, "Closing the window");

    let to_tray = CheckBox::builder(panel)
        .with_label("Keep Wixen Mail running in the &notification area")
        .build();
    set_accessible_name_and_description(
        &to_tray,
        "Keep Wixen Mail running in the notification area",
        "Closing the window hides it instead of ending the program. Quit still \
         ends it. The notification area holds a menu with New Message, Check \
         Mail and All Inboxes on it",
    );
    to_tray.set_value(config.keep_running_in_the_tray);
    close_sec.add(&to_tray, 0, SizerFlag::All, 4);

    sizer.add_sizer(&close_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    to_tray
}

/// What the General tab hands back.
///
/// A struct rather than a tuple because it carries seven controls now, and a
/// seven-place tuple is a list nobody can read at the call site: swapping two
/// of the check boxes would compile and save each other's setting.
struct GeneralTabControls {
    theme: Choice,
    font_family: Choice,
    font_size: SpinCtrl,
    language: Choice,
    check_before_send: CheckBox,
    check_as_you_type: CheckBox,
    smooth_scrolling: CheckBox,
    keep_selected_message_in_view: CheckBox,
    keep_running_in_the_tray: CheckBox,
    choose_default_programs: Button,
    check_default_programs_at_startup: CheckBox,
    which_updates: Choice,
}

/// The rows the spelling language picker offers, and which of them is
/// selected for the stored tag.
///
/// The rows are what this machine offers, in its own order. Which of them is
/// selected is [`which_row_shows`]'s answer, whose module says why the rule
/// has two halves. A tag chosen for a region is kept exactly as chosen,
/// whether or not this machine can check it, because what is shown is what
/// OK writes back: showing the checker's nearest dictionary instead rewrote
/// a stored en-AU to en-US on a machine that offers only en-US, which is
/// what CI's runner showed on 2026-09-18 and the machine the testing happens
/// on could not. A bare tag, which a profile from before 2026-09-03 stores,
/// is resolved as the checker resolves it, to this machine's own region, and
/// never to row 0 for a tag it could not match: row 0 in Windows' order is
/// English (Caribbean) on a machine set to English (United States), which is
/// #21. A stored tag nothing can check is shown as itself, marked as the
/// list marks any language with no dictionary, rather than as some other
/// language. When it is not even a row, it becomes one, at the end, so what
/// is stored is visible and pressing OK keeps it.
///
/// `read_settings` maps the selection back through the same rows, which is
/// why this is one function rather than a list built in two places.
fn language_rows_and_selection(stored: &str) -> (Vec<LanguageChoice>, usize) {
    let mut rows = available_languages();
    match which_row_shows(stored, system_language().as_deref(), &rows) {
        RowToShow::Existing(selected) => (rows, selected),
        RowToShow::Added(tag) => {
            rows.push(LanguageChoice {
                name: tag.clone(),
                tag,
                available: false,
            });
            let last = rows.len() - 1;
            (rows, last)
        }
    }
}

/// Which language spelling is checked in, and how the checking behaves.
///
/// Under General now rather than on a tab of its own. The default comes from
/// the machine's own locale, which `data::config` reads when it writes a fresh
/// settings file, so the first thing somebody sees is the language they
/// already work in.
fn add_language_and_spelling(
    panel: &Panel,
    config: &AppConfig,
    sizer: &BoxSizer,
) -> (Choice, CheckBox, CheckBox) {
    // -- Language
    //
    // "Interface language" was the wrong label. Nothing here is translated, and
    // this setting has only ever decided which dictionary the spell checker
    // uses. The list was wrong too: it offered the same six languages whatever
    // the machine had, so picking one it could not check set a value that
    // changed nothing, and the only way to find that out was to write in that
    // language and have every word of it called a mistake.
    let lang_sec = section(panel, "Language and spelling");

    let lang_row = BoxSizer::builder(Orientation::Horizontal).build();
    let lang_label = StaticText::builder(panel)
        .with_label("Check spelling in:")
        .build();

    let (languages, selected) = language_rows_and_selection(&config.language);
    let lang_names: Vec<String> = languages
        .iter()
        .map(|language| {
            if language.available {
                language.name.clone()
            } else {
                // Shown and marked rather than hidden. A language with no
                // dictionary is still worth offering, because installing one
                // is something somebody can go and do.
                format!("{} (no dictionary installed)", language.name)
            }
        })
        .collect();
    let lang_choice = Choice::builder(panel)
        .with_choices(lang_names)
        .with_selection(Some(selected as u32))
        .build();
    set_accessible_name_and_description(
        &lang_choice,
        "Check spelling in",
        "Starts as the language this computer is set to",
    );
    lang_row.add(
        &lang_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    lang_row.add(&lang_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    lang_sec.add_sizer(&lang_row, 0, SizerFlag::Expand, 0);

    let check_before_send = CheckBox::builder(panel)
        .with_label("&Check spelling before sending a message")
        .build();
    set_accessible_name(
        &check_before_send,
        "Check spelling before sending a message",
    );
    check_before_send.set_value(config.check_spelling_before_send);
    lang_sec.add(&check_before_send, 0, SizerFlag::All, 4);

    let check_as_you_type = CheckBox::builder(panel)
        .with_label("&Mark misspelled words as I write")
        .build();
    set_accessible_name(&check_as_you_type, "Mark misspelled words as I write");
    check_as_you_type.set_value(config.check_spelling_as_you_type);
    lang_sec.add(&check_as_you_type, 0, SizerFlag::All, 4);

    // What the marking is, said plainly, because it is not this application
    // doing the announcing and somebody comparing it with another program
    // should know why it sounds like their browser.
    let marking_note = StaticText::builder(panel)
        .with_label(
            "Marked words are announced by your screen reader as you move over them. \
             There is also a sound at the end of a word that is wrong, which is off \
             until earcons are switched on under Feedback.",
        )
        .build();
    lang_sec.add(&marking_note, 0, SizerFlag::All, 4);

    // Named without building a checker. This used to build one to ask it,
    // and letting go of a Windows checker cost about a fifth of a second,
    // measured on 2026-09-16 for #34, on every open of this dialog.
    let source = crate::service::spellcheck::source_for_language(&config.language);
    let checker_note = StaticText::builder(panel)
        .with_label(&format!("Spelling is checked by {}.", source.describe()))
        .build();
    lang_sec.add(&checker_note, 0, SizerFlag::All, 4);

    sizer.add_sizer(&lang_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (lang_choice, check_before_send, check_as_you_type)
}

fn build_general_tab(panel: &Panel, config: &AppConfig) -> GeneralTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Appearance
    let app_sec = section(panel, "Appearance");

    let theme_row = BoxSizer::builder(Orientation::Horizontal).build();
    let theme_label = StaticText::builder(panel).with_label("Theme:").build();
    let theme_choices: Vec<String> = ["Default", "Light", "Dark", "High Contrast"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let theme_idx: u32 = match config.theme.as_str() {
        "light" => 1,
        "dark" => 2,
        "high_contrast" => 3,
        _ => 0,
    };
    let theme_choice = Choice::builder(panel)
        .with_choices(theme_choices)
        .with_selection(Some(theme_idx))
        .build();
    set_accessible_name(&theme_choice, "Theme");
    theme_row.add(
        &theme_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    theme_row.add(&theme_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    app_sec.add_sizer(&theme_row, 0, SizerFlag::Expand, 0);

    // One sentence, said once, and it comes from the code it describes rather
    // than being retyped here where it can drift away from what the theme
    // actually does. A setting that changes less than it looks like it should
    // is a setting somebody reads as broken.
    let theme_note = StaticText::builder(panel)
        .with_label(crate::presentation::theme::REACH)
        .build();
    set_accessible_name(&theme_note, crate::presentation::theme::REACH);
    app_sec.add(&theme_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Only the fonts this computer has. A list written here would be the same
    // mistake the language list once made: Windows draws something else for a
    // typeface it does not have and says nothing, so choosing one that is not
    // installed would look like it worked and change nothing on the screen.
    let installed = crate::service::fonts::installed_families().unwrap_or_default();
    let type_row = BoxSizer::builder(Orientation::Horizontal).build();
    let type_label = StaticText::builder(panel).with_label("Font:").build();
    let type_choice = Choice::builder(panel)
        .with_choices(crate::application::font_choice::what_the_list_offers(
            &installed,
        ))
        .with_selection(Some(crate::application::font_choice::which_row_is_chosen(
            &config.font_family,
            &installed,
        ) as u32))
        .build();
    set_accessible_name_and_description(
        &type_choice,
        "Font",
        "The typeface your messages, contacts, calendar, tasks, notes and \
         reminders are listed in. Only fonts installed on this computer are \
         offered",
    );
    type_row.add(
        &type_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    type_row.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    app_sec.add_sizer(&type_row, 0, SizerFlag::Expand, 0);

    // Said only when it is true of this machine. A font uninstalled after it
    // was chosen, or a settings file carried from another computer, otherwise
    // shows as a program that looks wrong for no stated reason.
    let wrong = crate::application::font_choice::what_is_wrong_with_the_choice(
        &config.font_family,
        &installed,
    );
    if !wrong.is_empty() {
        let note = StaticText::builder(panel).with_label(&wrong).build();
        set_accessible_name(&note, &wrong);
        app_sec.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 4);
    }

    let font_row = BoxSizer::builder(Orientation::Horizontal).build();
    let font_label = StaticText::builder(panel).with_label("Font size:").build();
    let font_field = SpinCtrl::builder(panel)
        .with_range(FONT_SIZES.0, FONT_SIZES.1)
        .with_initial_value(within(config.font_size, FONT_SIZES))
        .build();
    name_the_spin_control(&font_field, "Font size");
    font_row.add(
        &font_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    font_row.add(&font_field, 0, SizerFlag::All, 4);
    app_sec.add_sizer(&font_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&app_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // New-mail notifications and checking for updates were both offered here
    // and read by nothing: there was no notification path and no update check
    // in this program. A control that takes an answer and ignores it is worse
    // than no control, so they went rather than sitting switched off.
    //
    // Half of that is no longer true. There is an update check now, so the
    // control below is not the old one put back: the old one was a switch for
    // machinery that did not exist, and this one decides which endpoint a check
    // that really happens asks. The setting is not called `check_updates`
    // either, because serde reads by name and a settings file written before
    // 2026-08-24 still holds that key.
    let which_updates = add_new_versions(panel, config, &sizer);

    let (language, check_before_send, check_as_you_type) =
        add_language_and_spelling(panel, config, &sizer);
    let (smooth_scrolling, keep_selected_message_in_view) = add_scrolling(panel, config, &sizer);
    let keep_running_in_the_tray = add_closing(panel, config, &sizer);
    let (choose_default_programs, check_default_programs_at_startup) =
        add_default_programs(panel, config, &sizer);

    panel.set_sizer(sizer, true);
    GeneralTabControls {
        theme: theme_choice,
        font_family: type_choice,
        font_size: font_field,
        language,
        check_before_send,
        check_as_you_type,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        choose_default_programs,
        check_default_programs_at_startup,
        which_updates,
    }
}

/// Which published versions somebody wants to be told about.
///
/// On General rather than on Advanced, and the reason is who meets it. Nothing
/// about hearing that a new version exists is advanced, and a person moving by
/// keyboard through a screen reader meets these sections in order and cannot
/// skim past the wrong ones. Advanced holds logging, storage and whether a
/// message is what it says it is, and a reader arriving at the last of those
/// looking for updates has already gone too far.
///
/// A combo box rather than a group of radio buttons. Both are honest shapes for
/// one answer out of three, and they cost differently. A radio group puts all
/// three in the tab order under one group label, so somebody hears every option
/// and its state without opening anything, at three stops instead of one. A
/// combo box is one stop and announces the current value, so the other two are
/// found by opening it.
///
/// The box wins here for two reasons that are about this screen rather than
/// about the widgets. Every other multi-valued setting in this dialog is a
/// combo box, twenty-four of them, so a radio group here would be the one
/// control on the page that behaves differently and somebody arrowing down the
/// tab would meet three stops where every neighbour is one. And `wxdragon` has
/// no `RadioBox` binding used anywhere in this tree, so the alternative is not a
/// choice between two supported shapes.
///
/// What the rejected shape would have cost: somebody who has never opened this
/// setting does not hear that a test-version option exists until they open the
/// box. That is a real cost and it is paid down by the description, which says
/// what choosing either kind means, and by the check's own answer, which names
/// the setting when it was asked on the public channel.
fn add_new_versions(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> Choice {
    use crate::common::version::WhichUpdates;
    use crate::service::update_check;

    let updates_sec = section(panel, update_check::SETTINGS_SECTION);

    let row = BoxSizer::builder(Orientation::Horizontal).build();
    let label = StaticText::builder(panel)
        .with_label(&format!("&{}:", update_check::WHICH_UPDATES_LABEL))
        .build();
    let choice = Choice::builder(panel)
        .with_choices(
            WhichUpdates::ALL
                .iter()
                .map(|kind| kind.words().to_string())
                .collect(),
        )
        .with_selection(Some(
            WhichUpdates::ALL
                .iter()
                .position(|kind| *kind == config.which_updates)
                .unwrap_or(0) as u32,
        ))
        .build();
    // Name and description in one call, not two. A box named only through
    // `set_accessible_name` has a name under NVDA and none under Narrator, and
    // the description is where the sentence about downloading has to reach.
    set_accessible_name_and_description(
        &choice,
        update_check::WHICH_UPDATES_LABEL,
        update_check::WHICH_UPDATES_DESCRIPTION,
    );
    row.add(
        &label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    row.add(&choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    updates_sec.add_sizer(&row, 0, SizerFlag::Expand, 0);

    // And on screen as well as in the accessibility tree. Somebody with low
    // vision reading the page rather than hearing it gets the same warning, and
    // a description that only exists on one channel is the shape this project
    // has already shipped twice.
    let note = StaticText::builder(panel)
        .with_label(update_check::WHICH_UPDATES_DESCRIPTION)
        .build();
    set_accessible_name(&note, update_check::WHICH_UPDATES_DESCRIPTION);
    updates_sec.add(&note, 0, SizerFlag::Left | SizerFlag::All, 4);

    sizer.add_sizer(&updates_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    choice
}

/// The controls `build_compose_tab` lays out, one field per control, named for
/// what it controls rather than by position: two of them are check boxes and
/// two are spin boxes, and a tuple would tell them apart by counting.
pub struct ComposeTabControls {
    copy_lines: Choice,
    preview_before_send: CheckBox,
    keep_sent_mail_on_this_computer: CheckBox,
    undo_send_hold: SpinCtrl,
    draft_autosave: SpinCtrl,
    add_signature_automatically: CheckBox,
}

impl ComposeTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        &self.copy_lines
    }
}

/// Compose settings: the compose window, sending, drafts, signature.
fn build_compose_tab(panel: &Panel, config: &AppConfig) -> ComposeTabControls {
    use crate::application::sending_later::{Hold, what_send_does};
    use crate::application::sent_copy::{KEEP_A_COPY_CONSEQUENCE, KEEP_A_COPY_LABEL};

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Writing
    //
    // Whether the Cc and Bcc lines are in the compose window from the start.
    // It sat in Dates and Times on the Reading tab, which is two wrong answers
    // to where somebody would look for it (#36): it is about the window a
    // message is written in, so it is here, first, before anything about what
    // happens once Send is pressed.
    let writing_sec = section(panel, "Writing");
    let copy_lines = labelled_choice(
        panel,
        &writing_sec,
        "Cc and Bcc &lines:",
        "Cc and Bcc lines",
        &["Always in the compose window", "Only when they are in use"],
        match CopyLines::from_setting(&config.copy_lines) {
            CopyLines::Shown => 0,
            CopyLines::Hidden => 1,
        },
    );
    sizer.add_sizer(&writing_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Sending
    let send_sec = section(panel, "Sending");
    let preview_cb = CheckBox::builder(panel)
        .with_label("Show &preview before sending")
        .build();
    set_accessible_name(&preview_cb, "Show preview before sending");
    preview_cb.set_value(config.preview_before_send);
    send_sec.add(&preview_cb, 0, SizerFlag::All, 4);

    // The description carries the consequence: with this on, Sent lists every
    // message twice once the server's own copy comes down. That is what the
    // setting does rather than a fault, and it is the part nobody can see
    // coming from the label alone.
    let keep_a_copy_cb = CheckBox::builder(panel)
        .with_label(KEEP_A_COPY_LABEL)
        .build();
    set_accessible_name_and_description(
        &keep_a_copy_cb,
        &name_from_label(KEEP_A_COPY_LABEL),
        KEEP_A_COPY_CONSEQUENCE,
    );
    keep_a_copy_cb.set_value(config.keep_sent_mail_on_this_computer);
    send_sec.add(&keep_a_copy_cb, 0, SizerFlag::All, 4);

    // How long Send waits before anything goes to a server. Under Sending on
    // the Compose tab, because that is where somebody looking for what Send
    // does will look, and a setting buried anywhere else is one they meet in
    // order while arrowing through a screen reader and cannot skim past.
    //
    // The same shape as the autosave spin box in the Drafts section below: a
    // number of units with nought meaning never, stepped with the arrow keys
    // rather than typed, read back through a constructor that clamps.
    //
    // The description is the part that cannot be guessed from the label, which
    // is what turning it off costs. `what_send_does` says it in the words it
    // will really be heard in, at whatever length is set, and at nought it says
    // there is no time to take a message back at all.
    let hold_row = BoxSizer::builder(Orientation::Horizontal).build();
    let hold_label = StaticText::builder(panel)
        .with_label("&Hold a message before sending for (seconds, 0 for no hold):")
        .build();
    let hold_spin = SpinCtrl::builder(panel)
        .with_range(Hold::OFF.seconds() as i32, Hold::LONGEST.seconds() as i32)
        .build();
    let hold_now = Hold::of_seconds(config.undo_send_hold_seconds);
    // Named through the spin control's own helper rather than `set_name`,
    // which sets an internal wxWidgets identifier and never reaches the
    // accessibility tree. Sixteen widgets were once named that way; it
    // compiled and 324 tests passed and no screen reader heard any of them.
    name_and_describe_the_spin_control(
        &hold_spin,
        "Hold a message before sending for, seconds, 0 for no hold",
        &what_send_does(hold_now),
    );
    hold_spin.set_value(hold_now.seconds() as i32);
    hold_row.add(
        &hold_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    hold_row.add(&hold_spin, 0, SizerFlag::All, 4);
    send_sec.add_sizer(&hold_row, 0, SizerFlag::Expand, 0);

    // A choice of "HTML" or "Plain Text" used to sit here, fixed on HTML and
    // read back by nothing. There is no such setting: the composer is one
    // editor, every message goes out with both a plain part and a formatted
    // one, and nothing anywhere asks which was wanted. Choosing the format a
    // message is written in is a feature, not a switch, so the control is gone
    // rather than left saying a decision was taken.

    sizer.add_sizer(&send_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Drafts
    let draft_sec = section(panel, "Drafts");
    // A spin box rather than the checkbox that used to be here. That claimed
    // "every 60 seconds", was ticked, was never read back, and nothing saved
    // anything. Minutes are the right grain: the difference between ninety
    // seconds and two minutes is not a decision anybody can make usefully, and
    // a spin box steps with the arrow keys rather than needing a number typed.
    let autosave_row = BoxSizer::builder(Orientation::Horizontal).build();
    let autosave_label = StaticText::builder(panel)
        .with_label("Save &drafts automatically every (minutes, 0 for never):")
        .build();
    let autosave_spin = SpinCtrl::builder(panel)
        .with_range(0, AutosaveInterval::MAX_MINUTES as i32)
        .build();
    name_the_spin_control(
        &autosave_spin,
        "Save drafts automatically every, minutes, 0 for never",
    );
    autosave_spin
        .set_value(AutosaveInterval::from_setting(config.draft_autosave_minutes).minutes() as i32);
    autosave_row.add(
        &autosave_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    autosave_row.add(&autosave_spin, 0, SizerFlag::All, 4);
    draft_sec.add_sizer(&autosave_row, 0, SizerFlag::Expand, 0);
    sizer.add_sizer(&draft_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Signatures
    //
    // The label says every message rather than a new one, because that is what
    // the composer does: a reply, a forward and a message written from a
    // contact all open with it too. It said "on new messages" while being
    // hard-set to yes and read back by nothing, so it was narrower than the
    // truth and made no difference either way.
    let sig_sec = section(panel, "Signatures");
    let sig_cb = CheckBox::builder(panel).with_label(SIGNATURE_LABEL).build();
    set_accessible_name_and_description(
        &sig_cb,
        &name_from_label(SIGNATURE_LABEL),
        SIGNATURE_WHEN_THIS_IS_OFF,
    );
    sig_cb.set_value(config.add_signature_automatically);
    sig_sec.add(&sig_cb, 0, SizerFlag::All, 4);

    // The same sentence on screen. A description reaches a screen reader that
    // reads through Microsoft Active Accessibility and nothing else, so what
    // the unticked state means would otherwise be there for one reader and
    // nobody else.
    let sig_note = StaticText::builder(panel)
        .with_label(SIGNATURE_WHEN_THIS_IS_OFF)
        .build();
    set_accessible_name(&sig_note, SIGNATURE_WHEN_THIS_IS_OFF);
    sig_sec.add(&sig_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&sig_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    ComposeTabControls {
        copy_lines,
        preview_before_send: preview_cb,
        keep_sent_mail_on_this_computer: keep_a_copy_cb,
        undo_send_hold: hold_spin,
        draft_autosave: autosave_spin,
        add_signature_automatically: sig_cb,
    }
}

/// One sentence, said once, in the label and in the accessible name.
const SIGNATURE_LABEL: &str = "Start every message with my &signature";

/// What the unticked state means, which a checkbox alone cannot say.
const SIGNATURE_WHEN_THIS_IS_OFF: &str = "Off: a message starts empty. Your signature stays on the account and can \
     still be added by hand.";

/// What the box that holds every pointed-at picture back says, with its
/// keyboard letter.
///
/// "Any", because since 2026-09-19 the default fetches them and holds back
/// only the ones that look like tracking pixels, so what this box adds is
/// fetching none. A constant for the reason its neighbour below gives.
const HOLD_BACK_PICTURES_LABEL: &str = "Do not &fetch any picture a message only points at";

/// What the unticked state means, which a check box alone cannot say.
///
/// It says which way it ships, what off does and what on does, because a
/// person meeting a box for the first time cannot tell a default from a
/// choice somebody made, and because the default here changed (#28).
const HOLD_BACK_PICTURES_WHEN_THIS_IS_OFF: &str = "Off by default. Off: pictures a message points at are fetched and shown, \
     except one whose declared size is a pixel or less, which is a tracking \
     pixel and is not fetched, and one the sender marked decorative. Fetching \
     a picture tells its sender you opened the message. On: none of them is \
     fetched. Pictures the message carries are always shown; they are already \
     here and showing them tells nobody anything.";

/// What the box for decorative pictures says, with its keyboard letter.
///
/// A constant because two places name it: the label the box carries, which is
/// what UI Automation and so Narrator reads, and the accessible name, which is
/// what MSAA and so NVDA reads. Written once so the two cannot drift.
const DECORATIVE_PICTURES_LABEL: &str = "Say where a picture the sender marked &decorative is";

/// What the unticked state means, which a check box alone cannot say.
///
/// It says which way it ships as well as what off does, because a person
/// meeting a box for the first time cannot tell a default from a choice
/// somebody made.
const DECORATIVE_PICTURES_WHEN_THIS_IS_OFF: &str = "On by default. A sender can mark a picture as having nothing to say, and \
     then nothing is read out where it is. Off: that mark is taken at face \
     value and the picture is passed over in silence. On: a short line says \
     the sender marked it decorative, so you can judge that for yourself.";

/// The controls `build_reading_tab` lays out, one field per choice, named
/// for what it actually controls rather than by position.
///
/// The two sort choices are public because a test builds this dialog and
/// reads their tab order back (#36); nothing else is.
pub struct ReadingTabControls {
    empty_reaches_subfolders: CheckBox,
    mark_read_reaches_subfolders: CheckBox,
    pub sort_order: Choice,
    show_conversations_by_default: CheckBox,
    start_in_all_inboxes: CheckBox,
    unread_on_a_parent: Choice,
    a_conversation_reaches: Choice,
    deleting_a_conversation_row: Choice,
    hold_back_remote_pictures: CheckBox,
    announce_decorative_pictures: CheckBox,
    /// Public because a test builds this dialog and reads the choice back
    /// through `read_settings` the way OK does (#28).
    pub undescribed_pictures_read_as: Choice,
    read_receipts: Choice,
    read_messages_as: Choice,
    /// Public because a test builds this dialog and reads the choice back
    /// through `read_settings` the way OK does (#80).
    pub open_links_in: Choice,
    date_style: Choice,
    date_order: Choice,
    date_wording: Choice,
    clock_hours: Choice,
    mark_read_after: Choice,
    mark_read_seconds: SpinCtrl,
    pub sort_then: Choice,
}

impl ReadingTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        &self.sort_order
    }
}

/// The controls `build_permissions_tab` lays out: what may be changed at a
/// server, and how a contact edit travels.
pub struct PermissionsTabControls {
    allow_mail: CheckBox,
    allow_pim: CheckBox,
    allow_message_text: CheckBox,
    /// Public, as `ReadingTabControls::sort_order` is, so the reading in
    /// `tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs`
    /// can choose a size and read it back through `read_settings`.
    pub message_text_kept: Choice,
    send_contact_changes_everywhere: CheckBox,
}

impl PermissionsTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        &self.allow_pim
    }
}

/// The controls `build_calendar_pim_tab` lays out. The reminder field is
/// public because a test reads back the colour it was painted.
pub struct CalendarPimTabControls {
    pub default_reminder: SpinCtrl,
    day_starts: Choice,
    day_ends: Choice,
    calendar_view: Choice,
    /// How long a new event lasts, and how far Up and Down move a time: one
    /// of `Block::ALL`, in its order.
    event_length: Choice,
}

impl CalendarPimTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        &self.calendar_view
    }
}

/// The controls `build_advanced_tab` lays out. The download folder is public
/// for the same reason the reminder field above is.
pub struct AdvancedTabControls {
    log_level: Choice,
    pub download_folder: TextCtrl,
    look_at_message_contents: CheckBox,
    check_links_with_google: CheckBox,
}

impl AdvancedTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        &self.log_level
    }
}

/// Reading settings: how the list is sorted, how a message opens, dates.
fn build_reading_tab(panel: &Panel, config: &AppConfig) -> ReadingTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Message List
    let list_sec = section(panel, "Message List");

    let sort_row = BoxSizer::builder(Orientation::Horizontal).build();
    let sort_label = StaticText::builder(panel)
        .with_label("Default sort order:")
        .build();
    let sort_choices: Vec<String> = [
        "Date (Newest First)",
        "Date (Oldest First)",
        "Sender (A-Z)",
        "Sender (Z-A)",
        "Subject (A-Z)",
        "Subject (Z-A)",
        "Unread First",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sort_idx: u32 = match config.default_sort_order.as_str() {
        "date_oldest" => 1,
        "sender_az" => 2,
        "sender_za" => 3,
        "subject_az" => 4,
        "subject_za" => 5,
        "unread_first" => 6,
        _ => 0,
    };
    let sort_choice = Choice::builder(panel)
        .with_choices(sort_choices)
        .with_selection(Some(sort_idx))
        .build();
    set_accessible_name(&sort_choice, "Default sort order");
    sort_row.add(
        &sort_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sort_row.add(&sort_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    list_sec.add_sizer(&sort_row, 0, SizerFlag::Expand, 0);
    // Where and when the order applies, under the choice (#91, 2026-09-20):
    // it is read once where the main window is built, and only where no
    // column layout was saved, since a saved layout carries its own sort.
    // Every other setting on this screen applies on save, so the one that
    // cannot has to say so, or it looks like a setting that does nothing.
    a_sentence_under(panel, &list_sec, WHERE_THE_DEFAULT_SORT_ORDER_APPLIES);

    // The second level of that sort, and the next tab stop after it. It was
    // built into Dates and Times at the bottom of this tab, so somebody
    // arrowing through by screen reader heard "Then by" straight after "Write
    // the month as", with the folders and reading sections between it and the
    // sort it is the second level of (#36). Tab moves through this panel's
    // children in the order they were made, so being built here is what puts
    // it beside the sort, and `tests/the_sort_controls_sit_together.rs` reads
    // that order back from the built dialog.
    let sort_then = labelled_choice(
        panel,
        &list_sec,
        "Then &by:",
        "Then by",
        &SECOND_LEVEL_LABELS,
        second_level_index(&config.message_columns),
    );

    // A checkbox reading "Enable threaded view by default" sat here once and
    // was taken out because threaded view did not exist then: it was unticked,
    // saved by nothing and read by nothing, a setting for a feature that was
    // not there. This one is the same question asked once the feature is
    // (#92, Pratik's decision of 2026-09-20): what a folder nobody has set
    // shows, on unless turned off, read by the main window where a folder
    // opens. A folder's own choice through View, Thread View still wins. Built
    // after Then by so the sort and its second level stay adjacent tab stops
    // (#36).
    let show_conversations_by_default = CheckBox::builder(panel)
        .with_label("&Show conversations by default")
        .build();
    show_conversations_by_default.set_value(config.show_conversations_by_default);
    set_accessible_name_and_description(
        &show_conversations_by_default,
        "Show conversations by default",
        "A folder you have never switched shows one row per conversation; View, Thread View \
         still chooses for one folder",
    );
    list_sec.add(
        &show_conversations_by_default,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );

    // This one does something too. The folder tree opens
    // with no row chosen, so mail is listed only once somebody arrows onto a
    // folder; ticking this lands them in the combined inbox instead.
    let start_in_all_inboxes = CheckBox::builder(panel)
        .with_label("Start in All &Inboxes")
        .build();
    start_in_all_inboxes.set_value(config.start_in_all_inboxes);
    set_accessible_name_and_description(
        &start_in_all_inboxes,
        "Start in All Inboxes",
        "Open showing every account's inbox in one list, rather than with no folder chosen",
    );
    list_sec.add(
        &start_in_all_inboxes,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );
    sizer.add_sizer(&list_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Folders and message lists, D-42: one group for the settings about how
    // the folder tree and the message list behave, rather than five of them
    // scattered down this page. The group's name comes from one constant, so
    // the first sentence anywhere that sends somebody here reads the name they
    // will hear rather than a second copy of it.
    let folders_sec = section(panel, folder_settings::SETTINGS_SECTION);

    // A choice rather than a check box, because neither option is the absence
    // of the other: both say something, and a box labelled for one of them
    // would have to word the other as "not that".
    let unread_on_a_parent = labelled_choice(
        panel,
        &folders_sec,
        "&Unread on a folder or account that holds others:",
        "Unread on a folder or account that holds others",
        &UnreadOnAParent::ALL.map(|option| option.words()),
        UnreadOnAParent::ALL
            .iter()
            .position(|option| *option == UnreadOnAParent::from_stored(&config.unread_on_a_parent))
            .unwrap_or(0) as u32,
    );

    // D-08. A choice rather than a check box for the same reason the one
    // above is: both answers say something, and neither is the absence of the
    // other. A box labelled "count a conversation across the account" would
    // have to word the other answer as not doing that, which says nothing
    // about what it does instead.
    let a_conversation_reaches = labelled_choice(
        panel,
        &folders_sec,
        "A &conversation is counted across:",
        "A conversation is counted across",
        &AConversationReaches::ALL.map(|option| option.words()),
        AConversationReaches::ALL
            .iter()
            .position(|option| {
                *option == AConversationReaches::from_stored(&config.a_conversation_reaches)
            })
            .unwrap_or(0) as u32,
    );

    // D-07, and it is a second question rather than a wider version of the one
    // above. That one is how far a conversation is counted; this is how far
    // Delete reaches when it is pressed on a collapsed row. Somebody who wants
    // to read about a whole account and delete out of one folder is asking for
    // something coherent, and one control for both would make them choose.
    let deleting_a_conversation_row = labelled_choice(
        panel,
        &folders_sec,
        "&Delete on a conversation row removes:",
        "Delete on a conversation row removes",
        &DeletingAConversationRow::ALL.map(|option| option.words()),
        DeletingAConversationRow::ALL
            .iter()
            .position(|option| {
                *option
                    == DeletingAConversationRow::from_stored(&config.deleting_a_conversation_row)
            })
            .unwrap_or(0) as u32,
    );

    // D-34 and D-35, and they are two boxes rather than one on purpose: one of
    // them destroys mail and the other loses somebody their place, and neither
    // has an undo. A single box covering both would make anybody who wanted
    // the careful reach on one of them take it on the other too.
    //
    // Both carry their label through `with_label` and their name through
    // `set_accessible_name_and_description`. A box labelled only through
    // `set_accessible_name` has a name under NVDA and none under Narrator,
    // which `tests/checkbox_labels.rs` is the guard for.
    //
    // The descriptions say what the answer costs rather than repeating the
    // label: the name already says what the box does, and what a screen reader
    // user needs next is which way is the wider one.
    let empty_reaches_subfolders = CheckBox::builder(panel)
        .with_label("&Empty Folder also empties the folders inside it")
        .build();
    empty_reaches_subfolders.set_value(config.empty_reaches_subfolders);
    set_accessible_name_and_description(
        &empty_reaches_subfolders,
        "Empty Folder also empties the folders inside it",
        "On, so emptying a folder empties everything filed under it too. The confirmation always \
         says how many folders and how many messages, and whether they move or go for good",
    );
    folders_sec.add(
        &empty_reaches_subfolders,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );

    let mark_read_reaches_subfolders = CheckBox::builder(panel)
        .with_label("Mark Folder &Read also marks the folders inside it")
        .build();
    mark_read_reaches_subfolders.set_value(config.mark_read_reaches_subfolders);
    set_accessible_name_and_description(
        &mark_read_reaches_subfolders,
        "Mark Folder Read also marks the folders inside it",
        "On, so marking a folder read marks everything filed under it too. There is no undo, so \
         turn this off to keep your place in folders you have not finished",
    );
    folders_sec.add(
        &mark_read_reaches_subfolders,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );

    sizer.add_sizer(&folders_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Reading Behaviour
    let read_sec = section(panel, "Reading Behaviour");

    // Built from the list rather than from a second copy of the words, and
    // read back below. It was neither before: four fixed choices, a fixed
    // selection, and nothing that saved it, so the answer was always
    // "immediately" whatever it said here. Since 12-06 the choice says which
    // kind of answer and the spin control beside it says how many seconds
    // (#35), reachable only while a wait is chosen.
    let markread_choice = labelled_choice(
        panel,
        &read_sec,
        "&Mark as read after:",
        MARK_READ_AFTER,
        &MarkReadWay::ALL.map(MarkReadWay::label),
        crate::application::reading_habits::offered_index(&config.mark_read_after) as u32,
    );
    let (markread_way, markread_wait) = MarkRead::from_setting(&config.mark_read_after).parts();
    let seconds_row = BoxSizer::builder(Orientation::Horizontal).build();
    let seconds_label = StaticText::builder(panel)
        .with_label("Seconds to wait:")
        .build();
    let markread_seconds = SpinCtrl::builder(panel)
        .with_range(1, MarkRead::LONGEST_WAIT_SECONDS as i32)
        .with_initial_value(markread_wait as i32)
        .build();
    name_the_spin_control(&markread_seconds, MARK_READ_SECONDS);
    markread_seconds.enable(markread_way == MarkReadWay::AfterSeconds);
    markread_choice.on_selection_changed(move |_| {
        markread_seconds.enable(chosen_way(&markread_choice) == MarkReadWay::AfterSeconds);
    });
    seconds_row.add(
        &seconds_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    seconds_row.add(&markread_seconds, 0, SizerFlag::All, 4);
    read_sec.add_sizer(&seconds_row, 0, SizerFlag::Expand, 0);
    // What the wait is counted from, under the choice, because the choice
    // and its seconds cannot say on their own when the counting starts, and
    // until 2026-09-18 it started when a row was selected (#25). Named on both
    // channels, the way the sentence under the message-text size is.
    let markread_note = StaticText::builder(panel)
        .with_label(WHAT_MARK_READ_COUNTS_FROM)
        .build();
    set_accessible_name(&markread_note, WHAT_MARK_READ_COUNTS_FROM);
    read_sec.add(&markread_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // How a message opens. First in this section, because it is the biggest
    // difference between two ways of reading the same mail.
    let style_row = BoxSizer::builder(Orientation::Horizontal).build();
    let style_label = StaticText::builder(panel)
        .with_label("Open messages:")
        .build();
    let style_choice = Choice::builder(panel)
        .with_choices(
            ReadingStyle::ALL
                .iter()
                .map(|style| style.spoken().to_string())
                .collect(),
        )
        .with_selection(Some(0))
        .build();
    set_accessible_name_and_description(
        &style_choice,
        "Open messages",
        "Formatted keeps the sender's headings, links and tables. Plain text \
         gives you a caret to move through the message with, and flattens them.",
    );
    let chosen_style = ReadingStyle::from_stored(&config.read_messages_as);
    style_choice.set_selection(
        ReadingStyle::ALL
            .iter()
            .position(|style| *style == chosen_style)
            .unwrap_or(0) as u32,
    );
    style_row.add(
        &style_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    style_row.add(&style_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    read_sec.add_sizer(&style_row, 0, SizerFlag::Expand, 0);

    // Where a link opens (#80), right after how a message opens, because it
    // is the next thing that happens when somebody reads one. The browser
    // first and by default, and the sentence under the choice says what each
    // answer costs, since three places cannot say that on their own: a page
    // in the message view shares the preview's browser profile, and the
    // separate window is not built yet.
    let open_links_labels: Vec<&str> = OpenLinks::ALL.iter().map(|choice| choice.label()).collect();
    let open_links_in = labelled_choice(
        panel,
        &read_sec,
        crate::application::opening_links::SETTING_LABEL,
        crate::application::opening_links::SETTING_NAME,
        &open_links_labels,
        crate::application::opening_links::offered_index(&config.open_links_in) as u32,
    );
    let open_links_note = StaticText::builder(panel)
        .with_label(crate::application::opening_links::WHAT_EACH_CHOICE_COSTS)
        .build();
    set_accessible_name(
        &open_links_note,
        crate::application::opening_links::WHAT_EACH_CHOICE_COSTS,
    );
    read_sec.add(&open_links_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Read receipts. On the Reading tab because it is a thing that happens
    // when you open a message, which is where somebody would look for it.
    let receipt_row = BoxSizer::builder(Orientation::Horizontal).build();
    let receipt_label = StaticText::builder(panel)
        .with_label("Tell senders when you read their mail:")
        .build();
    let receipt_choice = Choice::builder(panel)
        .with_choices(
            Policy::ALL
                .iter()
                .map(|policy| policy.spoken().to_string())
                .collect(),
        )
        .with_selection(Some(0))
        .build();
    // The whole sentence is the accessible name, because each choice says what
    // it costs and a name of "Read receipts" would hide that from the person
    // most likely to care about being tracked.
    set_accessible_name_and_description(
        &receipt_choice,
        "Tell senders when you read their mail",
        "A read receipt tells the sender your address is live and roughly when \
         you were at your desk. Nothing is sent unless you choose it here.",
    );
    let chosen = Policy::from_stored(&config.read_receipts);
    receipt_choice.set_selection(
        Policy::ALL
            .iter()
            .position(|policy| *policy == chosen)
            .unwrap_or(0) as u32,
    );
    receipt_row.add(
        &receipt_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    receipt_row.add(&receipt_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    read_sec.add_sizer(&receipt_row, 0, SizerFlag::Expand, 0);

    // There was a checkbox here once that saved nothing and was read by
    // nothing, and then a sentence admitting the pictures were fetched and
    // there was no switch. This is the switch, and it does what it says. Off
    // by default since 2026-09-19 (#28): the pictures show, and the ones
    // that look like tracking pixels are held back by their declared size
    // whether or not this is on.
    let hold_back_remote_pictures = CheckBox::builder(panel)
        .with_label(HOLD_BACK_PICTURES_LABEL)
        .build();
    hold_back_remote_pictures.set_value(config.hold_back_remote_pictures);
    set_accessible_name_and_description(
        &hold_back_remote_pictures,
        &HOLD_BACK_PICTURES_LABEL.replace('&', ""),
        HOLD_BACK_PICTURES_WHEN_THIS_IS_OFF,
    );
    read_sec.add(&hold_back_remote_pictures, 0, SizerFlag::All, 4);

    // Next to the other picture switch, which is where somebody looking for
    // what pictures do would look. Both are about what a picture does when a
    // message is read here rather than about what is sent.
    let announce_decorative_pictures = CheckBox::builder(panel)
        .with_label(DECORATIVE_PICTURES_LABEL)
        .build();
    announce_decorative_pictures.set_value(config.announce_decorative_pictures);
    set_accessible_name_and_description(
        &announce_decorative_pictures,
        &DECORATIVE_PICTURES_LABEL.replace('&', ""),
        DECORATIVE_PICTURES_WHEN_THIS_IS_OFF,
    );
    read_sec.add(&announce_decorative_pictures, 0, SizerFlag::All, 4);

    // What a picture nobody described is read as (#28). Under the two
    // picture boxes, because it is the third thing a picture does when a
    // message is read here. Three sentences rather than a mechanism, the
    // default first, and the sentence under it says what the choice leaves
    // alone: the sender's own description, and a link's words.
    let undescribed_labels: Vec<&str> = UndescribedPicture::ALL
        .iter()
        .map(|choice| choice.label())
        .collect();
    let undescribed_pictures_read_as = labelled_choice(
        panel,
        &read_sec,
        UNDESCRIBED_PICTURES_LABEL,
        UNDESCRIBED_PICTURES_LABEL
            .replace('&', "")
            .trim_end_matches(':'),
        &undescribed_labels,
        crate::application::describing_pictures::offered_index(&config.undescribed_pictures_read_as)
            as u32,
    );
    let undescribed_leaves_alone = StaticText::builder(panel)
        .with_label(WHAT_THE_CHOICE_LEAVES_ALONE)
        .build();
    set_accessible_name(&undescribed_leaves_alone, WHAT_THE_CHOICE_LEAVES_ALONE);
    read_sec.add(
        &undescribed_leaves_alone,
        0,
        SizerFlag::Expand | SizerFlag::All,
        4,
    );

    let images_note = StaticText::builder(panel)
        .with_label(REMOTE_IMAGES_ARE_FETCHED)
        .build();
    set_accessible_name(&images_note, REMOTE_IMAGES_ARE_FETCHED);
    read_sec.add(&images_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&read_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Dates and times
    //
    // Read in every module, and until now settable in none of them: the values
    // were in the settings file with nothing that could change them. What a
    // date sounds like is not a detail to somebody who hears every one of them.
    let date_sec = section(panel, "Dates and Times");

    // Two of the settings below say "Follow this computer", which makes the
    // whole section look as though it follows the computer's language too. It
    // does not, and somebody hearing an English month in the middle of an
    // otherwise French date deserves to be told that rather than left thinking
    // their screen reader is at fault.
    let date_language_note = StaticText::builder(panel)
        .with_label(crate::presentation::date_display::ENGLISH_ONLY)
        .build();
    set_accessible_name(
        &date_language_note,
        crate::presentation::date_display::ENGLISH_ONLY,
    );
    date_sec.add(
        &date_language_note,
        0,
        SizerFlag::Expand | SizerFlag::All,
        4,
    );

    let date_style = labelled_choice(
        panel,
        &date_sec,
        "How much of a date to say:",
        "How much of a date to say",
        &["Relative within the last week", "Always the full date"],
        match config.date_style.as_str() {
            "absolute" => 1,
            _ => 0,
        },
    );
    let date_order = labelled_choice(
        panel,
        &date_sec,
        "Day and month order:",
        "Day and month order",
        &[
            "Follow this computer",
            "Month first, July 26",
            "Day first, 26 July",
        ],
        match config.date_order.as_str() {
            "month_first" => 1,
            "day_first" => 2,
            _ => 0,
        },
    );
    let date_wording = labelled_choice(
        panel,
        &date_sec,
        "Write the month as:",
        "Write the month as",
        &["A word, July 26, 2026", "A number, 07/26/2026"],
        match config.date_wording.as_str() {
            "numeric" => 1,
            _ => 0,
        },
    );
    let clock_hours = labelled_choice(
        panel,
        &date_sec,
        "Clock:",
        "Clock",
        &[
            "Follow this computer",
            "Twelve hour, 2:30 PM",
            "Twenty-four hour, 14:30",
        ],
        match config.clock_hours.as_str() {
            "12" => 1,
            "24" => 2,
            _ => 0,
        },
    );

    sizer.add_sizer(&date_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    ReadingTabControls {
        sort_order: sort_choice,
        read_receipts: receipt_choice,
        read_messages_as: style_choice,
        open_links_in,
        date_style,
        date_order,
        date_wording,
        clock_hours,
        mark_read_after: markread_choice,
        mark_read_seconds: markread_seconds,
        sort_then,
        show_conversations_by_default,
        start_in_all_inboxes,
        unread_on_a_parent,
        a_conversation_reaches,
        deleting_a_conversation_row,
        empty_reaches_subfolders,
        mark_read_reaches_subfolders,
        hold_back_remote_pictures,
        announce_decorative_pictures,
        undescribed_pictures_read_as,
    }
}

/// Where a picture a message points at is fetched, and where it is not.
///
/// Narrower than the checkbox that used to stand here, and narrower than the
/// first version of this sentence, which said a picture was fetched whenever a
/// message was shown. The reading window is a text control and fetches
/// nothing. The two surfaces that show a message body in a browser are the
/// preview pane and the conversation window.
///
/// Until 2026-09-19 this ended "There is no setting for this yet", under the
/// very switch that is the setting: the sentence was written before the
/// switch and never read again once the switch arrived above it. It now says
/// which surfaces fetch and which cannot, which is the part no switch says.
///
/// Read rather than measured: the code hands the message to a browser on
/// those two surfaces and to a text control on the third. No network trace
/// has been taken.
const REMOTE_IMAGES_ARE_FETCHED: &str = "The preview pane and the conversation window show a message in a browser, \
     which is where a picture a message points at is fetched. The reading \
     window shows text and fetches nothing.";

/// What the second level of the sort can be, in the order it is offered.
///
/// A short list rather than every column. The question people actually ask of a
/// mailbox is "among the ones from today, which have I not read", and offering
/// fifteen columns here would bury it.
const SECOND_LEVEL_LABELS: [&str; 4] = [
    "Nothing else",
    "Unread first",
    "Sender, A to Z",
    "Subject, A to Z",
];

/// The second level a stored layout holds, as a position in that list.
fn second_level_index(stored: &str) -> u32 {
    use crate::presentation::message_columns::{ColumnLayout, MessageColumn};
    // Only the second level is wanted, so which folder the layout belongs to
    // does not come into it, and a string nothing can read has none to offer.
    let second = ColumnLayout::from_stored(stored).and_then(|layout| layout.sort.then);
    match second.map(|then| then.column) {
        Some(MessageColumn::Unread) => 1,
        Some(MessageColumn::Correspondent) => 2,
        Some(MessageColumn::Subject) => 3,
        _ => 0,
    }
}

/// The second level chosen, put back into the stored layout.
///
/// The layout is where the sort lives, so this reads the one that is stored,
/// changes the one part this control decides, and writes it out again. Writing
/// a fresh layout instead would throw away the columns somebody arranged, which
/// for anybody navigating a list by ear is real work.
///
/// Which folder the layout was arranged in is read from the string and written
/// back with it, untouched. Stamping one on here would be this screen deciding
/// something it cannot know, and the stored answer would then be believed on
/// the next start.
fn with_second_level(stored: &str, chosen: u32) -> String {
    use crate::presentation::message_columns::{
        By, ColumnLayout, FolderKind, MessageColumn, SortDirection,
    };
    // Nothing readable means no columns to keep, so the inbox's defaults are
    // what this writes, the same as the window would show.
    let mut layout = ColumnLayout::from_stored(stored)
        .unwrap_or_else(|| ColumnLayout::defaults_for(FolderKind::Inbox));
    layout.sort.then = match chosen {
        1 => Some(By {
            column: MessageColumn::Unread,
            direction: SortDirection::Ascending,
        }),
        2 => Some(By {
            column: MessageColumn::Correspondent,
            direction: SortDirection::Ascending,
        }),
        3 => Some(By {
            column: MessageColumn::Subject,
            direction: SortDirection::Ascending,
        }),
        _ => None,
    };
    layout.to_stored()
}

/// A choice with a label beside it, added to a section.
///
/// Four of these in a row is four copies of the same nine lines, and the copy
/// that gets the accessible name wrong is the one nobody notices until somebody
/// meets a control that announces nothing.
fn labelled_choice(
    panel: &Panel,
    section: &StaticBoxSizer,
    label: &str,
    spoken: &str,
    choices: &[&str],
    selected: u32,
) -> Choice {
    let row = BoxSizer::builder(Orientation::Horizontal).build();
    let text = StaticText::builder(panel).with_label(label).build();
    let choice = Choice::builder(panel)
        .with_choices(choices.iter().map(|c| c.to_string()).collect())
        .with_selection(Some(selected))
        .build();
    set_accessible_name(&choice, spoken);
    row.add(&text, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    row.add(&choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    section.add_sizer(&row, 0, SizerFlag::Expand, 0);
    choice
}

/// Language & Spelling: which language to check, and whether to check on send.
/// One sentence, said once, in the label and in the accessible name. Those two
/// were hand-written copies elsewhere in this file and had already drifted.
const CONTACT_CHANGES_WHEN_THIS_IS_OFF: &str =
    "Off: a change goes only to the address book the contact came from.";

/// What Wixen Mail may change at a server, and how a contact edit travels.
///
/// This was the Language and Spelling tab, which held neither of those things
/// on its own: the language picker and the two spelling boxes are under General
/// now, and what was left is the two permissions and the contacts rule. Named
/// for what it holds, because a tab named for something it does not contain is
/// a tab nobody looks in for what it does.
fn build_permissions_tab(panel: &Panel, config: &AppConfig) -> PermissionsTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Spell Check
    //
    // Two checkboxes used to sit here, "enable spell checking in compose
    // editor" and "show suggestions as you type". Both were ticked, neither
    // was ever read back, and nothing checked anything, so they were three
    // claims in a row. What replaces them says what this machine actually has,
    // which is worth more than a switch for something that does not happen.
    // ── What may be changed at a server ──────────────────────────────────
    //
    // Two checkboxes rather than one, because the two cost different amounts
    // to get wrong: a sent message cannot be recalled and a deleted one may
    // have been the only copy, while a task in the wrong place can be moved
    // back. Both say they are experimental, because they are: none of it has
    // run against a real account.
    // The heading comes from the same constant the sync sentences name, so a
    // person told to turn this on reads the words they were told. The label
    // and the sentence were typed separately and said different things.
    let allowed_sec = section(panel, crate::application::allowed::SETTINGS_SECTION);

    let allow_pim = CheckBox::builder(panel)
        .with_label("Allow Wixen Mail to change my &tasks, contacts and calendar")
        .build();
    set_accessible_name(
        &allow_pim,
        "Allow Wixen Mail to change my tasks, contacts and calendar",
    );
    allow_pim.set_value(config.allowed_changes.personal_information);
    allowed_sec.add(&allow_pim, 0, SizerFlag::All, 4);

    let allow_mail = CheckBox::builder(panel)
        .with_label("Allow Wixen Mail to &send and delete mail")
        .build();
    set_accessible_name(&allow_mail, "Allow Wixen Mail to send and delete mail");
    allow_mail.set_value(config.allowed_changes.mail);
    allowed_sec.add(&allow_mail, 0, SizerFlag::All, 4);

    // One sentence, said once. The label and the accessible name were two
    // hand-written copies that had already drifted apart.
    let allowed_note = StaticText::builder(panel)
        .with_label(crate::application::allowed::EXPERIMENTAL_WARNING)
        .build();
    set_accessible_name(
        &allowed_note,
        crate::application::allowed::EXPERIMENTAL_WARNING,
    );
    allowed_sec.add(&allowed_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // This line was missing. The section was built, the two checkboxes and the
    // experimental warning were put into it, and the section itself was never
    // put into the panel's layout, so the one place that says none of this has
    // run against a real account had nowhere to appear.
    sizer.add_sizer(&allowed_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // ── Message text ─────────────────────────────────────────────────────
    //
    // Its own section rather than a third box under the heading above, because
    // a read is not a change. Under that heading it would be a box saying one
    // thing inside a heading saying another, which is the drift the constant
    // naming that heading was introduced to stop.
    //
    // The heading above is not named here in words on purpose: the guard that
    // keeps it in one constant reads this file as text and cannot tell a
    // comment quoting it from a screen writing it out.
    //
    // The heading is also not "Reading", which this dialog already uses for a
    // tab about how messages are shown. The sentence a refused fetch says
    // names this heading, and it has to name a heading that is on this page.
    let reading_sec = section(panel, crate::application::allowed::READING_SECTION);

    let allow_message_text = CheckBox::builder(panel)
        .with_label(crate::application::allowed::MESSAGE_TEXT_LABEL)
        .build();
    // Both channels, from the one string. The label serves UI Automation,
    // which is what Narrator reads from a native control's own text; this
    // serves MSAA, which is what NVDA reads. Written as the label with its
    // mnemonic marker taken out rather than typed again, because the two
    // hand-written copies beside every other control on this page are exactly
    // what drifted before.
    set_accessible_name(
        &allow_message_text,
        &crate::application::allowed::MESSAGE_TEXT_LABEL.replace('&', ""),
    );
    allow_message_text.set_value(config.allowed_changes.reading);
    reading_sec.add(&allow_message_text, 0, SizerFlag::All, 4);

    let reading_note = StaticText::builder(panel)
        .with_label(crate::application::allowed::MESSAGE_TEXT_NOTE)
        .build();
    set_accessible_name(
        &reading_note,
        crate::application::allowed::MESSAGE_TEXT_NOTE,
    );
    reading_sec.add(&reading_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // How much of the text, once fetched, stays (#23). Under the same heading
    // and after the note, so a person who finds the box that forbids fetching
    // finds the size beside it. A choice of four words rather than a number
    // of bytes, because a screen reader user meets the words and picks one.
    // The default is all of it, which is the tester's decision; a stored size
    // the list does not offer selects All, and the module says why.
    use crate::application::keeping_message_text::{
        KEEP_LABEL, TextKept, WHAT_A_SIZE_DOES, offered_index,
    };
    let kept_labels: Vec<String> = TextKept::ALL.iter().map(|c| c.label()).collect();
    let kept_labels: Vec<&str> = kept_labels.iter().map(String::as_str).collect();
    let message_text_kept = labelled_choice(
        panel,
        &reading_sec,
        KEEP_LABEL,
        KEEP_LABEL.replace('&', "").trim_end_matches(':'),
        &kept_labels,
        offered_index(&config.message_text_kept) as u32,
    );
    // What a size does, under the choice, because four sizes cannot say on
    // their own what passing one costs: what leaves, when, what stays.
    let kept_note = StaticText::builder(panel)
        .with_label(WHAT_A_SIZE_DOES)
        .build();
    set_accessible_name(&kept_note, WHAT_A_SIZE_DOES);
    reading_sec.add(&kept_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&reading_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // ── Contacts ─────────────────────────────────────────────────────────
    //
    // Directly under the warning above, so somebody reading down the panel
    // meets the sentence saying none of this has run against a real account
    // before they meet this. The label says what happens rather than naming
    // the machinery, and the sentence under it says what turning it off does,
    // because a checkbox alone cannot say what its unticked state means.
    let contacts_sec = section(panel, "Contacts");

    let send_contact_changes_everywhere = CheckBox::builder(panel)
        .with_label(
            // Alt+V rather than Alt+S, which the mail setting above already
            // claims on this page.
            "Send a change to a contact to e&very address book that has that contact",
        )
        .build();
    set_accessible_name(
        &send_contact_changes_everywhere,
        "Send a change to a contact to every address book that has that contact",
    );
    send_contact_changes_everywhere.set_value(config.send_contact_changes_everywhere);
    contacts_sec.add(&send_contact_changes_everywhere, 0, SizerFlag::All, 4);

    let contacts_note = StaticText::builder(panel)
        .with_label(CONTACT_CHANGES_WHEN_THIS_IS_OFF)
        .build();
    set_accessible_name(&contacts_note, CONTACT_CHANGES_WHEN_THIS_IS_OFF);
    contacts_sec.add(&contacts_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Here as well as on the screen that adds one, because this is where
    // somebody looking for what this program may change to their contacts
    // comes, and the screen that adds an address book is one they may have met
    // weeks ago. The same words in both places rather than a second sentence
    // that drifts.
    let address_book_note = StaticText::builder(panel)
        .with_label(crate::application::address_book_source::NOT_TRIED_FOR_REAL)
        .build();
    set_accessible_name(
        &address_book_note,
        crate::application::address_book_source::NOT_TRIED_FOR_REAL,
    );
    contacts_sec.add(&address_book_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&contacts_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    PermissionsTabControls {
        allow_mail,
        allow_pim,
        allow_message_text,
        message_text_kept,
        send_contact_changes_everywhere,
    }
}

/// Calendar & PIM settings: default view, weekends, first day, reminder time.
fn build_calendar_pim_tab(
    panel: &Panel,
    config: &AppConfig,
    accounts: &[Account],
    a_calendar_server: bool,
) -> CalendarPimTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Calendar View
    let view_sec = section(panel, "Calendar");

    // A default view, showing weekends, and the first day of the week were all
    // offered here and read by nothing. The view picker was the worst of them:
    // it offered Day, Week and Month, three views this program could not draw
    // at all, and it was taken out rather than left lying.
    //
    // It is back, offering the three that exist, and this section stops being a
    // heading with nothing under it. Showing weekends and the first day of the
    // week have not come back: neither is built, and offering either again
    // would be the same defect a second time.
    //
    // The names and their order come from `CalendarView::OFFERED`, which the
    // picker on the calendar toolbar also reads, so the two cannot come to
    // offer different views or the same views in a different order.
    let view_choice = labelled_choice(
        panel,
        &view_sec,
        "Calendar opens &on:",
        "The calendar opens on",
        &CalendarView::OFFERED.map(CalendarView::label),
        CalendarView::from_stored(&config.calendar_view).offered_at(),
    );

    // How long a new event lasts, and how far a time moves on Up and Down
    // (#41). In the Calendar section because a new event's length is a
    // question about the calendar; the description says what else it moves,
    // since a reminder's time takes the same keys.
    let stored_block = Block::from_setting(config.event_length_minutes);
    let event_length = labelled_choice(
        panel,
        &view_sec,
        "New events &last:",
        "New events last",
        &Block::ALL.map(Block::label),
        Block::ALL
            .iter()
            .position(|block| *block == stored_block)
            .unwrap_or_default() as u32,
    );
    set_accessible_name_and_description(
        &event_length,
        "New events last",
        "A new event starts at the next whole block and lasts this long. In an \
         event or a reminder, Up and Down on a time's minutes move it by this \
         much, and Left and Right by one minute.",
    );

    sizer.add_sizer(&view_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Reminders
    let rem_sec = section(panel, "Reminders");

    let rem_row = BoxSizer::builder(Orientation::Horizontal).build();
    let rem_label = StaticText::builder(panel)
        .with_label("Default &reminder (minutes):")
        .build();
    let rem_field = SpinCtrl::builder(panel)
        .with_range(REMINDER_MINUTES.0, REMINDER_MINUTES.1)
        .with_initial_value(within(config.default_reminder_minutes, REMINDER_MINUTES))
        .build();
    name_the_spin_control(&rem_field, "Default reminder in minutes");
    rem_row.add(
        &rem_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    rem_row.add(&rem_field, 0, SizerFlag::All, 4);
    rem_sec.add_sizer(&rem_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&rem_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- The working day
    //
    // The calendar read every hour the same way, so nine in the morning and
    // three in the morning sounded alike and an event outside the working day
    // said nothing about itself. A meeting at seven in the evening is a fact
    // somebody wants to notice.
    let day_sec = section(panel, "Working Day");
    let day = WorkingDay::from_setting(config.working_day_starts, config.working_day_ends);
    let day_starts = labelled_choice(
        panel,
        &day_sec,
        "Starts &at:",
        "The working day starts at",
        &HOURS,
        day.starts as u32,
    );
    let day_ends = labelled_choice(
        panel,
        &day_sec,
        "&Ends at:",
        "The working day ends at",
        &HOURS,
        // The list runs from midnight, and the end is the first hour outside
        // the day, so five in the afternoon is the entry at seventeen.
        (day.ends as u32).min(HOURS.len() as u32 - 1),
    );
    sizer.add_sizer(&day_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Notes
    //
    // Last on the tab, and deliberately. The three sections above are one run:
    // the calendar, the reminders it raises, and the working day it draws.
    // Somebody moving through by keyboard meets sections in order and cannot
    // skim, so the tab's other subject starting after the first one finishes
    // is what they expect. Putting Notes between Calendar and Reminders would
    // split a group that belongs together, and putting it first would answer a
    // question about notes on a tab whose name puts the calendar first.
    //
    // A sentence rather than a control, because there is nothing to choose. No
    // account has a notes backend, so a switch would be a control that does
    // nothing, which this program has removed repeatedly. The moment a backend
    // gives somebody a real choice, that is a setting and it arrives with the
    // work that introduces the choice.
    //
    // It answers for the default account, which is the one a new note is filed
    // under. This screen cannot reach any other: it is handed a configuration
    // and the accounts, and there is nothing on it that names one account
    // rather than another.
    let notes_sec = section(panel, "Notes");

    let where_notes_go = crate::application::notes_backend::where_the_default_accounts_notes_go(
        Some(config.default_account_id.as_str()),
        accounts,
        a_calendar_server,
    );
    let notes_answer = StaticText::builder(panel)
        .with_label(&where_notes_go)
        .build();
    // Both Windows channels from the one string. The label serves UI
    // Automation, which is what Narrator reads from a native control's own
    // text; this serves MSAA, which is what NVDA reads.
    set_accessible_name(&notes_answer, &where_notes_go);
    notes_sec.add(&notes_answer, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&notes_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    CalendarPimTabControls {
        default_reminder: rem_field,
        day_starts,
        day_ends,
        calendar_view: view_choice,
        event_length,
    }
}

/// Every hour of the day, named rather than numbered.
///
/// "Midnight" and "noon" rather than "12 AM" and "12 PM", which are the two
/// nobody agrees about, and a twenty-four hour reading beside each so the list
/// reads the same to somebody who keeps either clock.
const HOURS: [&str; 25] = [
    "Midnight, 00",
    "1 AM, 01",
    "2 AM, 02",
    "3 AM, 03",
    "4 AM, 04",
    "5 AM, 05",
    "6 AM, 06",
    "7 AM, 07",
    "8 AM, 08",
    "9 AM, 09",
    "10 AM, 10",
    "11 AM, 11",
    "Noon, 12",
    "1 PM, 13",
    "2 PM, 14",
    "3 PM, 15",
    "4 PM, 16",
    "5 PM, 17",
    "6 PM, 18",
    "7 PM, 19",
    "8 PM, 20",
    "9 PM, 21",
    "10 PM, 22",
    "11 PM, 23",
    "Midnight, 24",
];

/// Advanced: log level, download folder, cache info, link checking.
fn build_advanced_tab(panel: &Panel, config: &AppConfig) -> AdvancedTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Logging
    let log_sec = section(panel, "Logging");

    let log_row = BoxSizer::builder(Orientation::Horizontal).build();
    let log_label = StaticText::builder(panel).with_label("Log level:").build();
    let log_choices: Vec<String> = ["Error", "Warn", "Info", "Debug", "Trace"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let log_idx: u32 = match config.log_level.as_str() {
        "error" => 0,
        "warn" => 1,
        "debug" => 3,
        "trace" => 4,
        _ => 2,
    };
    let log_choice = Choice::builder(panel)
        .with_choices(log_choices)
        .with_selection(Some(log_idx))
        .build();
    set_accessible_name(&log_choice, "Log level");
    log_row.add(
        &log_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    log_row.add(&log_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    log_sec.add_sizer(&log_row, 0, SizerFlag::Expand, 0);
    // The level is set up once when the program starts and nothing can
    // change it while it runs, so the control says a change waits for the
    // next start (#91, 2026-09-20) rather than looking like a setting that
    // does nothing.
    a_sentence_under(panel, &log_sec, TAKES_EFFECT_AT_THE_NEXT_START);

    sizer.add_sizer(&log_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Storage
    let store_sec = section(panel, "Storage");

    let dl_row = BoxSizer::builder(Orientation::Horizontal).build();
    let dl_label = StaticText::builder(panel)
        .with_label("Download folder:")
        .build();
    let dl_field = TextCtrl::builder(panel).build();
    set_accessible_name(&dl_field, "Download folder");
    dl_field.set_value(&config.download_folder.to_string_lossy());
    let dl_browse = Button::builder(panel).with_label("&Browse...").build();

    dl_browse.on_click({
        let dl_field_clone = dl_field;
        let panel_ref = *panel;
        move |_ev| {
            let dir_dlg = DirDialog::builder(&panel_ref, "Select download folder", "").build();
            if dir_dlg.show_modal() == ID_OK
                && let Some(path) = dir_dlg.get_path()
            {
                dl_field_clone.set_value(&path);
            }
        }
    });

    dl_row.add(
        &dl_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    dl_row.add(&dl_field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    dl_row.add(&dl_browse, 0, SizerFlag::All, 4);
    store_sec.add_sizer(&dl_row, 0, SizerFlag::Expand, 0);

    let cache_hint = StaticText::builder(panel)
        .with_label("Message cache is stored in the system cache directory.\nClearing cache will require re-downloading messages.")
        .build();
    store_sec.add(&cache_hint, 0, SizerFlag::All, 4);

    sizer.add_sizer(&store_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Checking whether a message is what it says it is
    //
    // Two boxes, deliberately apart. The first reads the message on this
    // computer and sends nothing, so it is on unless somebody turns it off.
    // The second can put four bytes of a link on the wire, so it is off unless
    // somebody turns it on. Sharing one switch would mean agreeing to the
    // second to get the first.
    let links_sec = section(panel, "Checking whether a message is what it says it is");

    let body_box = CheckBox::builder(panel)
        .with_label("Read each message on this computer and mark suspicious ones")
        .build();
    set_accessible_name(
        &body_box,
        "Read each message on this computer and mark suspicious ones",
    );
    body_box.set_value(config.look_at_message_contents);
    links_sec.add(&body_box, 0, SizerFlag::All, 4);

    let body_hint = StaticText::builder(panel)
        .with_label(crate::application::body_safety::LOOKING_AT_THE_MESSAGE_ITSELF)
        .build();
    links_sec.add(&body_hint, 0, SizerFlag::All, 4);

    let links_box = CheckBox::builder(panel)
        .with_label("Check links against Google Safe Browsing")
        .build();
    set_accessible_name(&links_box, "Check links against Google Safe Browsing");
    links_box.set_value(config.check_links_with_google);
    links_sec.add(&links_box, 0, SizerFlag::All, 4);

    let links_hint = StaticText::builder(panel)
        .with_label(
            "Google's lists of known phishing and malware sites are downloaded to 
             this computer, and links are compared here. Your links are not sent to 
             Google. Only if a link matches one of the downloaded entries do four 
             bytes of it go to Google to confirm, and those four bytes stand for 
             millions of possible addresses. Nothing about the sender, the subject 
             or the message is ever sent.
             
             Needs a Google API key in oauth.toml. Without one this does nothing.",
        )
        .build();
    links_sec.add(&links_hint, 0, SizerFlag::All, 4);

    sizer.add_sizer(&links_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    AdvancedTabControls {
        log_level: log_choice,
        download_folder: dl_field,
        look_at_message_contents: body_box,
        check_links_with_google: links_box,
    }
}

/// The three controls that answer for whichever event the picker is showing,
/// and the two lines beneath them.
///
/// Held together rather than passed around separately because moving between
/// events touches all five at once, and a caller that updated four of them
/// would leave a screen saying one thing about an event and another thing
/// about the event before it.
#[derive(Clone)]
pub struct PerEventControls {
    /// One per [`Switch`], in `Switch::ALL`'s order, each carrying the switch
    /// it answers so a tick cannot be read back against a different one.
    pub ticks: Vec<(Switch, CheckBox)>,
    /// What the selected event will really produce, which is not always what
    /// the ticks say.
    pub what_really_happens: StaticText,
    /// Whether the selected event has an answer of its own or is on the
    /// default. Ticks alone cannot tell those two apart.
    pub whose_answer: StaticText,
    /// Which event of [`feedback::Event::ALL`] the ticks are describing right
    /// now.
    ///
    /// Needed because a selection change says where the picker has arrived and
    /// not where it came from, and what is on screen belongs to where it came
    /// from.
    pub showing: Rc<Cell<usize>>,
    /// Everything this tab has been told so far, including answers for events
    /// the picker is not showing. The save path writes this.
    pub working: Rc<RefCell<FeedbackSettings>>,
}

/// The two sentences that tell an event with an answer of its own from one on
/// the default. Ticks alone cannot say which, because an answer can be the same
/// as the default and still be an answer.
const THIS_EVENT_HAS_ITS_OWN_ANSWER: &str =
    "This event has an answer of its own. The boxes above are what you chose for it.";
const THIS_EVENT_IS_USING_THE_DEFAULT: &str =
    "This event is using the default. The boxes above show what the default is.";

/// Which of the three controls a stored answer paints as ticked.
///
/// `None` is nobody having touched the event, which paints as that event's
/// own default, `FeedbackSettings::the_default_for`: every channel for all but
/// the attachment event, whose default is the sound and the status bar (#77).
/// Painted from the event's default rather than from every channel so the
/// boxes show what is really in force, and the line beneath still says it is
/// the default. A switch is ticked where the answer holds any of the channels
/// it stands for rather than all of them, because a settings file can hold
/// speech on and braille off. Older builds really could write that, since the
/// two boxes standing here were independent. Painting it as unticked would show
/// silence to somebody who has announcements, and saving that screen would then
/// make it true.
fn ticks_for(
    event: feedback::Event,
    chosen: Option<&BTreeSet<Channel>>,
) -> [bool; Switch::ALL.len()] {
    let default = FeedbackSettings::the_default_for(event);
    let answer = chosen.unwrap_or(&default);
    Switch::ALL.map(|switch| switch.channels().iter().any(|c| answer.contains(c)))
}

/// What one event will really produce, which is not always what the three
/// controls above it say.
///
/// Two rules bend an answer on its way out. A channel switched off everywhere
/// is off for every event whatever that event says, and an event left with only
/// a sound has the quietest written channel added back, because a sound with no
/// words anywhere is a noise a deaf-blind user cannot perceive at all. Saying so
/// here is what stops the ticks being a lie. The rule itself is not weakened to
/// match the screen: it lives in `channels_for` so that no call site can forget
/// it, and two tests hold it there.
fn what_this_event_will_really_do(settings: &FeedbackSettings, event: feedback::Event) -> String {
    let reaching = settings.channels_for(event);
    let mut parts: Vec<&str> = Vec::new();
    if reaching.contains(&Channel::Speech) || reaching.contains(&Channel::Braille) {
        parts.push("announced through your screen reader");
    }
    if reaching.contains(&Channel::Earcon) {
        parts.push("given its own sound");
    }
    if reaching.contains(&Channel::Visual) {
        parts.push("shown in the status bar");
    }
    let sentence = match parts.as_slice() {
        [] => "nothing happens for this event at all".to_string(),
        [only] => (*only).to_string(),
        [all_but_last @ .., last] => format!("{} and {last}", all_but_last.join(", ")),
    };
    format!("Right now: {sentence}.")
}

impl PerEventControls {
    /// The event the ticks are describing.
    fn shown(&self) -> feedback::Event {
        feedback::Event::ALL[self.showing.get().min(feedback::Event::ALL.len() - 1)]
    }

    /// Write what is on screen into the working settings for the event the
    /// ticks are describing.
    ///
    /// Nothing is written where the ticks still show exactly what was painted
    /// into them, so visiting an event and changing nothing does not give it an
    /// answer of its own. Those two states are not the same: an event with no
    /// answer follows the default wherever the default goes, and one whose
    /// answer happens to match the default today does not.
    pub fn remember_what_is_on_screen(&self) {
        let event = self.shown();
        let mut working = self.working.borrow_mut();
        let as_painted = ticks_for(event, working.what_was_chosen_for(event).as_ref());
        let now: Vec<bool> = self
            .ticks
            .iter()
            .map(|(_, tick)| tick.get_value())
            .collect();
        if now == as_painted {
            return;
        }
        let picked = self
            .ticks
            .iter()
            .filter(|(_, tick)| tick.get_value())
            .flat_map(|(switch, _)| switch.channels().iter().copied())
            .collect();
        working.set_event_channels(event, picked);
    }

    /// Show the event at `at` in [`feedback::Event::ALL`], having first
    /// remembered the one being left.
    ///
    /// The first half is the one that gets forgotten. Without it, ticking a box
    /// and then moving the picker throws the tick away in silence, which is the
    /// bug `tests/every_event_has_a_control.rs` mainly exists to catch.
    pub fn show(&self, at: usize) {
        self.remember_what_is_on_screen();
        self.showing.set(at.min(feedback::Event::ALL.len() - 1));
        self.paint_the_shown_event();
    }

    /// Put the event being shown back to the default.
    ///
    /// What is on screen is deliberately not remembered first, because
    /// discarding it is the whole point. This removes the entry, where
    /// switching all three boxes off stores an empty answer, and the two mean
    /// opposite things: an empty answer round trips and means silence for that
    /// event, and no entry at all means the default. They must not share a
    /// control, which is why the model has two methods and this calls the one
    /// named for what the button says.
    pub fn put_the_shown_event_back_to_the_default(&self) {
        self.working.borrow_mut().use_the_default_for(self.shown());
        self.paint_the_shown_event();
    }

    /// Paint the three ticks and both lines from the working settings.
    ///
    /// The ticks come from `what_was_chosen_for` and the first line from
    /// `channels_for`, and those are different questions. `channels_for`
    /// fills a missing entry with the event's own default, drops the channels
    /// switched off everywhere and adds a written channel where only a sound
    /// was picked, so ticks painted from it would show somebody answers they
    /// never gave.
    fn paint_the_shown_event(&self) {
        let event = self.shown();
        let working = self.working.borrow();
        let chosen = working.what_was_chosen_for(event);
        for ((_, tick), on) in self.ticks.iter().zip(ticks_for(event, chosen.as_ref())) {
            tick.set_value(on);
        }
        self.whose_answer.set_label(match chosen {
            Some(_) => THIS_EVENT_HAS_ITS_OWN_ANSWER,
            None => THIS_EVENT_IS_USING_THE_DEFAULT,
        });
        self.what_really_happens
            .set_label(&what_this_event_will_really_do(&working, event));
    }
}

/// Everything the Feedback tab hands back.
pub struct FeedbackTabControls {
    /// The three controls that answer for every event at once.
    pub global: Vec<(Switch, CheckBox)>,
    /// The sentence saying whose decision it is whether words arrive as speech
    /// or on a braille display. Handed back so a test can read it off the built
    /// screen: it is the half of the roadmap's first criterion that is a
    /// sentence rather than a control, and nothing else in the tree would
    /// notice it going.
    pub whose_choice: StaticText,
    /// Which event the three controls beneath it are describing.
    pub event: Choice,
    pub per_event: PerEventControls,
    pub sound_scheme: Choice,
    /// How much is said while mail and the other modules are fetched (#38).
    /// Public so the reading in `tests/every_event_has_a_control.rs` can
    /// choose a level and read it back through `read_settings`.
    pub announce_while_fetching: Choice,
}

impl FeedbackTabControls {
    /// The control first in this page's tab order, which is where wx's own
    /// first-child pick lands: the first of the three boxes that answer for
    /// every event, or the event picker if there were none. The reading in
    /// `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    /// holds it to the first tab-stop child of the page, so moving a control
    /// ahead of it is a change here too.
    fn first_in_tab_order(&self) -> &dyn WxWidget {
        self.global
            .first()
            .map_or(&self.event as &dyn WxWidget, |(_, checkbox)| checkbox)
    }
}

/// Feedback channels: how the application tells you something happened.
///
/// **Not a grid of events**, because the choice people actually make is "words,
/// not sounds" or "sounds, not words", and forty-eight boxes met one after
/// another by somebody who cannot skim is worse than the setting being missing.
/// That argument stood here before this panel did and it is still right.
///
/// What changed is its conclusion. Offering nothing left sixteen per-event
/// answers in the model that no screen could reach, which is the
/// setting-nobody-can-find rule in `CLAUDE.md` broken in the one place this
/// program is most about. The shape below answers the objection rather than
/// overruling it: a picker, three controls, a button and a line, so the page
/// holds five controls whatever the event count becomes.
///
/// **Three controls and four channels, and the difference is the point.**
/// Speech and braille are one answer because they are one call, which
/// [`Switch`]'s own doc comment explains. The two boxes that used to offer them
/// apart could not do what they said whichever way they were ticked.
///
/// Nothing here can produce a sound-only application by accident: the routing
/// adds a written equivalent unless every text channel is off, and the line
/// beneath the per-event controls says what the selected event will really do
/// rather than leaving it to be discovered.
fn build_feedback_tab(
    panel: &Panel,
    config: &AppConfig,
    a11y: &Arc<Accessibility>,
) -> FeedbackTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let settings = FeedbackSettings::from_stored(&config.feedback_channels);

    let intro = StaticText::builder(panel)
        .with_label(
            "Choose how Wixen Mail tells you about new mail, sent messages, connection \
             changes, and errors. Sounds never replace words unless you switch every \
             other channel off yourself.",
        )
        .build();
    sizer.add(&intro, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let sec = section(panel, "Every event");
    // Each box carries the answer it gives. The wording comes off the answer
    // too, so there is no second list to fall out of step with this one and no
    // position to pair by.
    let mut global = Vec::new();
    for switch in Switch::ALL {
        let label = switch.setting_label();
        let cb = CheckBox::builder(panel).with_label(label).build();
        set_accessible_name(&cb, &name_from_label(label));
        // Ticked where any of the channels it stands for is on, not where all
        // of them are, for the reason `ticks_for` gives about a settings file
        // holding speech on and braille off.
        cb.set_value(
            switch
                .channels()
                .iter()
                .any(|channel| settings.is_channel_enabled(*channel)),
        );
        sec.add(&cb, 0, SizerFlag::All, 4);
        global.push((switch, cb));
    }
    sizer.add_sizer(&sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // The sentence criterion 1 asks for, said once and in the place the answer
    // it is about is given. Saying it twice would be two sentences to keep in
    // step, and somebody moving through this page by keyboard meets each line
    // in order rather than skimming past a repeat.
    let whose_choice = StaticText::builder(panel)
        .with_label(
            "Whether something is spoken, shown on a braille display, or both is \
             chosen in your screen reader, not here. Wixen Mail sends one \
             notification and your screen reader decides what to do with it.",
        )
        .build();
    sizer.add(&whose_choice, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let note = StaticText::builder(panel)
        .with_label(
            "Each event is given its own tone, and sounds are spaced out so a busy \
             mailbox does not run them together. Whether every pair really is easy \
             to tell apart by ear has not been tested yet, so please say if two of \
             them sound alike.",
        )
        .build();
    sizer.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let scheme_sec = section(panel, "Sound");
    let scheme_row = BoxSizer::builder(Orientation::Horizontal).build();
    let scheme_label = StaticText::builder(panel)
        .with_label("Sound scheme:")
        .build();
    let schemes = discovered_schemes();
    let scheme_choices: Vec<String> = schemes.iter().map(|s| s.name.clone()).collect();
    let scheme_idx = schemes
        .iter()
        .position(|s| s.id == config.sound_scheme_id)
        .unwrap_or(0) as u32;
    let scheme_choice = Choice::builder(panel)
        .with_choices(scheme_choices)
        .with_selection(Some(scheme_idx))
        .build();
    set_accessible_name(&scheme_choice, "Sound scheme");
    scheme_row.add(
        &scheme_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::Right,
        8,
    );
    scheme_row.add(&scheme_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    scheme_sec.add_sizer(&scheme_row, 0, SizerFlag::Expand, 0);

    let scheme_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let import_btn = Button::builder(panel)
        .with_label("&Import sound scheme...")
        .build();
    set_accessible_name(&import_btn, "Import sound scheme");
    scheme_btn_row.add(&import_btn, 0, SizerFlag::Right, 4);

    let delete_btn = Button::builder(panel)
        .with_label("&Delete sound scheme")
        .build();
    set_accessible_name(&delete_btn, "Delete sound scheme");
    // Disabled the moment there is nothing but the built-in default to act
    // on: a button that always opens a confirmation only to refuse whatever
    // was chosen teaches nothing except to stop trying it.
    delete_btn.enable(schemes.len() > 1);
    scheme_btn_row.add(&delete_btn, 0, SizerFlag::Left, 4);
    scheme_sec.add_sizer(&scheme_btn_row, 0, SizerFlag::All, 4);

    import_btn.on_click({
        let panel = *panel;
        let a11y = a11y.clone();
        move |_| import_sound_scheme(&panel, &scheme_choice, &delete_btn, &a11y)
    });
    delete_btn.on_click({
        let panel = *panel;
        let a11y = a11y.clone();
        move |_| delete_sound_scheme(&panel, &scheme_choice, &delete_btn, &a11y)
    });

    sizer.add_sizer(&scheme_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // One event at a time. A picker, the same three answers for whichever
    // event it is on, a button that takes the answer away again, and two lines
    // saying whose answer is on screen and what it will really produce.
    let per_event_sec = section(panel, "One event at a time");
    let picker_row = BoxSizer::builder(Orientation::Horizontal).build();
    let picker_label = StaticText::builder(panel).with_label("&Event:").build();
    let event_choice = Choice::builder(panel)
        .with_choices(
            feedback::Event::ALL
                .iter()
                .map(|event| event.text().to_string())
                .collect(),
        )
        .build();
    set_accessible_name(&event_choice, "Event");
    picker_row.add(
        &picker_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    picker_row.add(&event_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    per_event_sec.add_sizer(&picker_row, 0, SizerFlag::Expand, 0);

    let mut ticks = Vec::new();
    for switch in Switch::ALL {
        let label = switch.label_beside_one_event();
        let tick = CheckBox::builder(panel).with_label(label).build();
        set_accessible_name(&tick, &name_from_label(label));
        per_event_sec.add(&tick, 0, SizerFlag::All, 4);
        ticks.push((switch, tick));
    }

    // Not "clear" and not "switch everything off". Switching all three boxes
    // off is its own answer and means silence for this event; this takes the
    // answer away so the default comes back. The model keeps the two apart and
    // so does the screen.
    let use_the_default = Button::builder(panel)
        .with_label("Use the de&fault for this event")
        .build();
    set_accessible_name(&use_the_default, "Use the default for this event");
    per_event_sec.add(&use_the_default, 0, SizerFlag::All, 4);

    let whose_answer = StaticText::builder(panel)
        .with_label(THIS_EVENT_IS_USING_THE_DEFAULT)
        .build();
    per_event_sec.add(&whose_answer, 0, SizerFlag::Expand | SizerFlag::All, 4);
    let what_really_happens = StaticText::builder(panel).with_label("").build();
    per_event_sec.add(
        &what_really_happens,
        0,
        SizerFlag::Expand | SizerFlag::All,
        4,
    );
    sizer.add_sizer(&per_event_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // While fetching (#38). Last on the tab, after the per-event choices,
    // because it is about a run of events rather than one: how much of what
    // a check says on the way is heard. Three sentences rather than a
    // number, because a screen reader user meets the words and picks one.
    // The default is what arrived, which is the tester's shape; the sentence
    // under it says what the choice leaves alone, because three answers
    // cannot say on their own what stays the same under all of them.
    use crate::application::what_is_said_while_fetching::{
        HowMuchToSay, WHAT_THE_CHOICE_LEAVES_ALONE, WHILE_FETCHING_LABEL, offered_index,
    };
    let fetching_sec = section(panel, "While fetching");
    let level_labels: Vec<&str> = HowMuchToSay::ALL.iter().map(|c| c.label()).collect();
    let announce_while_fetching = labelled_choice(
        panel,
        &fetching_sec,
        WHILE_FETCHING_LABEL,
        WHILE_FETCHING_LABEL.replace('&', "").trim_end_matches(':'),
        &level_labels,
        offered_index(&config.announce_while_fetching) as u32,
    );
    let leaves_alone = StaticText::builder(panel)
        .with_label(WHAT_THE_CHOICE_LEAVES_ALONE)
        .build();
    set_accessible_name(&leaves_alone, WHAT_THE_CHOICE_LEAVES_ALONE);
    fetching_sec.add(&leaves_alone, 0, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&fetching_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let per_event = PerEventControls {
        ticks,
        what_really_happens,
        whose_answer,
        showing: Rc::new(Cell::new(0)),
        working: Rc::new(RefCell::new(settings)),
    };
    // The picker opens on the first event and the controls beneath it are
    // painted by the same function every later change uses, so the two cannot
    // disagree, not even on the first paint. Painted rather than shown, because
    // `show` remembers what is on screen first and nothing has been painted
    // into the ticks yet.
    event_choice.set_selection(0);
    per_event.paint_the_shown_event();

    event_choice.on_selection_changed({
        let per_event = per_event.clone();
        move |_| per_event.show(sel(&event_choice) as usize)
    });
    use_the_default.on_click({
        let per_event = per_event.clone();
        move |_| per_event.put_the_shown_event_back_to_the_default()
    });

    panel.set_sizer(sizer, true);
    FeedbackTabControls {
        global,
        whose_choice,
        event: event_choice,
        per_event,
        sound_scheme: scheme_choice,
        announce_while_fetching,
    }
}

/// Rebuild the picker's items and the delete button's enabled state from
/// the real current disk state.
///
/// `select_id` is looked for in the freshly discovered list, not assumed
/// still to be there: the caller may be selecting what it just imported, or
/// falling back to Generated because what was selected is what just got
/// deleted. Either way this is the one place that decides which index that
/// becomes, so the picker and the button can never show one truth while the
/// disk holds another.
fn refresh_scheme_controls(scheme_choice: &Choice, delete_btn: &Button, select_id: &str) {
    let schemes = discovered_schemes();
    scheme_choice.clear();
    for candidate in &schemes {
        scheme_choice.append(&candidate.name);
    }
    let index = schemes.iter().position(|s| s.id == select_id).unwrap_or(0);
    scheme_choice.set_selection(index as u32);
    delete_btn.enable(schemes.len() > 1);
}

/// Picking a zip, importing it, and refreshing the picker to include it.
///
/// Announced through `a11y` rather than only shown, the same as everything
/// else this dialog reports: a picker whose new item appeared with no
/// sentence to go with it is invisible to anyone not looking at the screen
/// at the exact moment it changed.
fn import_sound_scheme(
    panel: &Panel,
    scheme_choice: &Choice,
    delete_btn: &Button,
    a11y: &Arc<Accessibility>,
) {
    let picker = FileDialog::builder(panel)
        .with_message("Import sound scheme")
        .with_wildcard("Sound scheme packs (*.zip)|*.zip")
        .with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
        .build();
    if picker.show_modal() != ID_OK {
        // Cancelling is a decision, not an outcome to report.
        return;
    }
    let Some(path) = picker.get_path() else {
        let _ = a11y.announce(
            "No file was chosen",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };
    let zip_path = std::path::Path::new(&path);
    let stem = zip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported");
    let id = sound_scheme_import::slug_for(stem);

    let Ok(paths) = AppPaths::resolve() else {
        let _ = a11y.announce(
            "The sound-schemes folder could not be found",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };

    match sound_scheme_import::import_zip(zip_path, &id, &paths.sound_schemes_dir()) {
        Ok(scheme) => {
            refresh_scheme_controls(scheme_choice, delete_btn, &scheme.id);
            let _ = a11y.announce(
                &format!(
                    "Imported {}, covers {} of {} events",
                    scheme.name,
                    scheme.covers(),
                    crate::presentation::accessibility::feedback::Event::ALL.len()
                ),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
        Err(err) => {
            let _ = a11y.announce(
                &format!("Could not import that sound scheme: {err}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
    }
}

/// Confirming, deleting, and refreshing the picker to drop what is gone.
///
/// The selected scheme is read fresh from `discovered_schemes()` rather
/// than trusted from whatever was on screen when the dialog was built: an
/// import earlier in this same dialog session can have changed what index
/// `scheme_choice`'s selection actually points at.
fn delete_sound_scheme(
    panel: &Panel,
    scheme_choice: &Choice,
    delete_btn: &Button,
    a11y: &Arc<Accessibility>,
) {
    let schemes = discovered_schemes();
    let selected = schemes.get(sel(scheme_choice) as usize);
    let Some(scheme) = selected.filter(|s| s.id != "generated") else {
        // Reachable even with the button enabled: the picker's own
        // selection can sit on Generated while an imported scheme exists
        // alongside it. Said plainly rather than silently doing nothing,
        // which reads as a broken button.
        let _ = a11y.announce(
            "Generated tones is the built-in default and cannot be deleted. \
             Choose an imported scheme first.",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };

    let question = format!("Delete \"{}\"? This cannot be undone.", scheme.name);
    // Enter answers No, the same reasoning every other delete confirmation
    // in this application uses: a key one row away from every other key
    // somebody might have meant should not be the one that finishes an
    // irreversible action if pressed early.
    let asked = MessageDialog::builder(panel, &question, "Delete")
        .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
        .build()
        .show_modal();
    if asked != ID_YES {
        return;
    }

    let Ok(paths) = AppPaths::resolve() else {
        let _ = a11y.announce(
            "The sound-schemes folder could not be found",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };
    match SoundScheme::delete(&scheme.id, &paths.sound_schemes_dir()) {
        Ok(()) => {
            let name = scheme.name.clone();
            refresh_scheme_controls(scheme_choice, delete_btn, "generated");
            let _ = a11y.announce(
                &format!("Deleted {name}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
        Err(err) => {
            let _ = a11y.announce(
                &format!("Could not delete that sound scheme: {err}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
    }
}

/// Every sound scheme this installation can currently offer: the built-in
/// default first, then whatever real packs are sitting in the sound-schemes
/// folder. Falls back to just the default if the folder itself cannot be
/// resolved, the same graceful degradation `SoundScheme::discover` already
/// gives an unreadable or missing folder.
fn discovered_schemes() -> Vec<SoundScheme> {
    match AppPaths::resolve() {
        Ok(paths) => SoundScheme::discover(&paths.sound_schemes_dir()),
        Err(_) => vec![SoundScheme::generated()],
    }
}

// ── Read settings back from widget references ────────────────────────────────

/// Collect current widget values and produce an updated `AppConfig`.
///
/// Public so a test can ask what pressing OK would save without showing a
/// modal. That join is not a detail: the model can hold a per-event answer and
/// the screen can show one, and "the setting survives a restart" is the claim
/// that this function carries the first across to the second. Both halves were
/// proved separately before this was reachable, and the join was proved by
/// reading.
pub fn read_settings(w: &SettingsWidgets, base: &AppConfig) -> AppConfig {
    let mut cfg = base.clone();

    // A page whose tab was never shown was never built, and its settings
    // are `base`'s, which is what the page would have shown; each `if let`
    // below leaves them as they are. Nothing here builds a page: OK is not
    // the moment to pay for six pages nobody looked at.

    if let Some(page) = w.later.feedback.if_built() {
        read_the_feedback_page(page, &mut cfg);
    }

    if let Some(page) = w.later.permissions.if_built() {
        read_the_permissions_page(page, &mut cfg);
    }

    // General
    cfg.theme = match sel(&w.theme) {
        1 => "light",
        2 => "dark",
        3 => "high_contrast",
        _ => "default",
    }
    .to_string();
    cfg.font_size = held(&w.font_size);
    cfg.smooth_scrolling = w.smooth_scrolling.get_value();
    cfg.keep_running_in_the_tray = w.keep_running_in_the_tray.get_value();
    // By the words shown rather than the row number. A row number needs the
    // installed list a second time to mean anything, and if that list differed
    // at saving from the one somebody chose from, their choice would be stored
    // as a different font or quietly reset.
    cfg.font_family = w
        .font_family
        .get_string_selection()
        .map(|chosen| crate::application::font_choice::what_the_words_store(&chosen))
        .unwrap_or_else(|| cfg.font_family.clone());
    cfg.check_default_programs_at_startup = w.check_default_programs_at_startup.get_value();
    // By position in the one array the control was built from, rather than by
    // matching the words back. The words are what somebody reads and the
    // position is what the box holds, and a box that answers nothing keeps
    // whatever was already stored rather than falling back to a variant chosen
    // here, which would be a second place deciding what this setting means.
    cfg.which_updates = crate::common::version::WhichUpdates::ALL
        .get(sel(&w.which_updates) as usize)
        .copied()
        .unwrap_or(cfg.which_updates);
    cfg.keep_selected_message_in_view = w.keep_selected_message_in_view.get_value();
    // Language. Rebuilt rather than remembered, because it is what the picker
    // was filled from and the two have to stay the same list, including the
    // row a stored tag nothing offers was given at the end.
    let (languages, _) = language_rows_and_selection(&base.language);
    let idx = sel(&w.language) as usize;
    if idx < languages.len() {
        cfg.language = languages[idx].tag.clone();
    }
    cfg.check_spelling_before_send = w.check_spelling_before_send.is_checked();
    cfg.check_spelling_as_you_type = w.check_spelling_as_you_type.is_checked();

    if let Some(page) = w.later.compose.if_built() {
        read_the_compose_page(page, &mut cfg);
    }
    if let Some(page) = w.later.reading.if_built() {
        read_the_reading_page(page, base, &mut cfg);
    }
    if let Some(page) = w.later.calendar_and_pim.if_built() {
        read_the_calendar_and_pim_page(page, &mut cfg);
    }
    if let Some(page) = w.later.advanced.if_built() {
        read_the_advanced_page(page, &mut cfg);
    }

    cfg
}

/// Feedback: which channels each event reaches, which sound scheme plays,
/// and how much is said while things are fetched.
///
/// The tab's working settings hold every per-event answer it has been
/// given, including for events the picker is not showing, so what is on
/// screen is remembered first and then the whole thing is written.
///
/// This used to rebuild from the stored `feedback_channels` and set only the
/// global channels, with a comment saying the per-event overrides in the
/// stored value were preserved because "this tab only decides which channels
/// are on at all". That stopped being true the moment the tab could create
/// them.
fn read_the_feedback_page(w: &FeedbackTabControls, cfg: &mut AppConfig) {
    w.per_event.remember_what_is_on_screen();
    {
        let mut feedback = w.per_event.working.borrow_mut();
        for (switch, cb) in &w.global {
            for channel in switch.channels() {
                feedback.set_channel_enabled(*channel, cb.get_value());
            }
        }
        cfg.feedback_channels = feedback.to_stored();
    }

    // The scheme picker's own order is whatever discovery produced when
    // the page was built; reading it back the same way is what makes the
    // selection index mean the same scheme it meant a moment ago.
    let schemes = discovered_schemes();
    cfg.sound_scheme_id = schemes
        .get(sel(&w.sound_scheme) as usize)
        .map(|s| s.id.clone())
        .unwrap_or_default();

    cfg.announce_while_fetching =
        crate::application::what_is_said_while_fetching::HowMuchToSay::ALL
            .get(sel(&w.announce_while_fetching) as usize)
            .copied()
            .unwrap_or_default()
            .as_stored();
}

/// Permissions: what may be done at a server, read back as three answers,
/// because they are three: sending cannot be undone, a task can be moved
/// back, and fetching a message's text changes nothing there at all. And how
/// a contact edit travels.
fn read_the_permissions_page(w: &PermissionsTabControls, cfg: &mut AppConfig) {
    cfg.allowed_changes = crate::application::allowed::Allowed {
        mail: w.allow_mail.get_value(),
        personal_information: w.allow_pim.get_value(),
        reading: w.allow_message_text.get_value(),
    };
    cfg.message_text_kept = crate::application::keeping_message_text::TextKept::ALL
        .get(sel(&w.message_text_kept) as usize)
        .copied()
        .unwrap_or_default()
        .as_stored();
    cfg.send_contact_changes_everywhere = w.send_contact_changes_everywhere.get_value();
}

/// Compose: the compose window, sending, drafts, signature.
fn read_the_compose_page(w: &ComposeTabControls, cfg: &mut AppConfig) {
    cfg.preview_before_send = w.preview_before_send.get_value();
    cfg.keep_sent_mail_on_this_computer = w.keep_sent_mail_on_this_computer.get_value();
    // Through the clamping constructor on the way out as well as on the way
    // in, so what is stored is a length the program will really use and every
    // sentence about the hold names the length that came out.
    cfg.undo_send_hold_seconds =
        crate::application::sending_later::Hold::of_seconds(w.undo_send_hold.value() as i64)
            .seconds();
    cfg.draft_autosave_minutes =
        AutosaveInterval::from_setting(w.draft_autosave.value().max(0) as u32).minutes();
    cfg.add_signature_automatically = w.add_signature_automatically.get_value();
    cfg.copy_lines = match sel(&w.copy_lines) {
        1 => CopyLines::Hidden,
        _ => CopyLines::Shown,
    }
    .as_stored()
    .to_string();
}

/// Reading: how the list is sorted, how a message opens, dates.
fn read_the_reading_page(w: &ReadingTabControls, base: &AppConfig, cfg: &mut AppConfig) {
    cfg.start_in_all_inboxes = w.start_in_all_inboxes.get_value();
    cfg.hold_back_remote_pictures = w.hold_back_remote_pictures.get_value();
    cfg.announce_decorative_pictures = w.announce_decorative_pictures.get_value();
    // By position out of `UndescribedPicture::ALL`, the same order the
    // choices were built from, so the two cannot drift apart the way a
    // second list of words would.
    cfg.undescribed_pictures_read_as = UndescribedPicture::ALL
        .get(sel(&w.undescribed_pictures_read_as) as usize)
        .copied()
        .unwrap_or_default()
        .as_stored();
    // By the words shown rather than the row number. A row number needs the
    // list the control was built from a second time to mean anything, and if
    // that list differed at saving from the one somebody chose from, their
    // choice would be stored as a different answer or quietly reset. Words
    // nothing recognises fall to the default rather than to whichever branch
    // is written first, which is what a hand-edited settings file gets.
    cfg.unread_on_a_parent = w
        .unread_on_a_parent
        .get_string_selection()
        .map(|chosen| UnreadOnAParent::from_words(&chosen))
        .unwrap_or_else(|| UnreadOnAParent::from_stored(&cfg.unread_on_a_parent))
        .as_str()
        .to_string();
    // D-08, read back by the words for the reason `font_family` gives above.
    cfg.a_conversation_reaches = w
        .a_conversation_reaches
        .get_string_selection()
        .map(|chosen| AConversationReaches::from_words(&chosen))
        .unwrap_or_else(|| AConversationReaches::from_stored(&cfg.a_conversation_reaches))
        .as_str()
        .to_string();
    // D-07, written back separately because it is a separate answer.
    cfg.deleting_a_conversation_row = w
        .deleting_a_conversation_row
        .get_string_selection()
        .map(|chosen| DeletingAConversationRow::from_words(&chosen))
        .unwrap_or_else(|| DeletingAConversationRow::from_stored(&cfg.deleting_a_conversation_row))
        .as_str()
        .to_string();
    // D-34 and D-35, written back separately because they are two answers.
    cfg.empty_reaches_subfolders = w.empty_reaches_subfolders.get_value();
    cfg.mark_read_reaches_subfolders = w.mark_read_reaches_subfolders.get_value();
    cfg.default_sort_order = match sel(&w.sort_order) {
        1 => "date_oldest",
        2 => "sender_az",
        3 => "sender_za",
        4 => "subject_az",
        5 => "subject_za",
        6 => "unread_first",
        _ => "date_newest",
    }
    .to_string();
    // What a folder nobody has set shows (#92); the next folder opened reads
    // it.
    cfg.show_conversations_by_default = w.show_conversations_by_default.get_value();

    // Read receipts. Read by position out of `Policy::ALL`, which is the same
    // order the choices were built from, so the two cannot drift apart the way
    // a second list of words would.
    cfg.read_messages_as = ReadingStyle::ALL
        .get(sel(&w.read_messages_as) as usize)
        .copied()
        .unwrap_or_default()
        .as_str()
        .to_string();
    // Where a link opens, by position out of `Where::ALL` for the same
    // reason (#80).
    cfg.open_links_in = OpenLinks::ALL
        .get(sel(&w.open_links_in) as usize)
        .copied()
        .unwrap_or_default()
        .stored()
        .to_string();

    // Dates and times, read in every module.
    cfg.date_style = match sel(&w.date_style) {
        1 => "absolute",
        _ => "relative",
    }
    .to_string();
    cfg.date_order = match sel(&w.date_order) {
        1 => "month_first",
        2 => "day_first",
        _ => "auto",
    }
    .to_string();
    cfg.date_wording = match sel(&w.date_wording) {
        1 => "numeric",
        _ => "verbal",
    }
    .to_string();
    cfg.clock_hours = match sel(&w.clock_hours) {
        1 => "12",
        2 => "24",
        _ => "auto",
    }
    .to_string();

    cfg.mark_read_after =
        MarkRead::from_parts(chosen_way(&w.mark_read_after), held(&w.mark_read_seconds))
            .as_stored();
    cfg.message_columns = with_second_level(&base.message_columns, sel(&w.sort_then));

    cfg.read_receipts = Policy::ALL
        .get(sel(&w.read_receipts) as usize)
        .copied()
        .unwrap_or_default()
        .as_str()
        .to_string();
}

/// Calendar & PIM: the working day, the calendar's view, the reminder.
fn read_the_calendar_and_pim_page(w: &CalendarPimTabControls, cfg: &mut AppConfig) {
    // Kept through the same check the calendar reads it through, so a day
    // that ends before it starts never reaches the file.
    let day = WorkingDay::from_setting(sel(&w.day_starts) as u8, sel(&w.day_ends) as u8);
    cfg.working_day_starts = day.starts;
    cfg.working_day_ends = day.ends;

    // Read back, not only shown. A control that shows a stored answer and is
    // read into nothing satisfies every check about being offered and changes
    // nothing, which is the failure
    // `test_whether_message_text_may_be_fetched_is_offered_by_a_screen` is
    // written about.
    cfg.calendar_view = CalendarView::offered_at_entry(w.calendar_view.get_selection())
        .stored()
        .to_string();

    cfg.default_reminder_minutes = held(&w.default_reminder);

    cfg.event_length_minutes = Block::ALL
        .get(sel(&w.event_length) as usize)
        .copied()
        .unwrap_or_default()
        .minutes();
}

/// Advanced: the log level, the download folder, what is looked at.
fn read_the_advanced_page(w: &AdvancedTabControls, cfg: &mut AppConfig) {
    cfg.log_level = match sel(&w.log_level) {
        0 => "error",
        1 => "warn",
        3 => "debug",
        4 => "trace",
        _ => "info",
    }
    .to_string();
    let path = w.download_folder.get_value();
    if !path.is_empty() {
        cfg.download_folder = std::path::PathBuf::from(path);
    }
    cfg.look_at_message_contents = w.look_at_message_contents.is_checked();
    cfg.check_links_with_google = w.check_links_with_google.is_checked();
}

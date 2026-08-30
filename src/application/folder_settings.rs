//! What the settings screen calls the group holding the folder and message
//! list settings.
//!
//! One string rather than one per screen, for the reason
//! [`crate::application::allowed::SETTINGS_SECTION`] gives: a sentence that
//! sends somebody to a section has to say the name they will then read on the
//! screen. That went wrong once already, where a sync said "Allow Changes" and
//! the section was headed "Allowed Changes", which is near enough to look like
//! the right place and far enough to make somebody stop and check.
//!
//! Nothing outside the settings screen names this group in a sentence today.
//! The constant is here so that the first thing which does reads it, rather
//! than typing the name a second time and starting the same drift, and
//! `test_no_settings_screen_writes_a_section_name_out_itself` in
//! `tests/house_style.rs` is what keeps the screen itself reading it.

/// What the settings screen calls the group of folder and message list
/// settings, on the Reading page.
///
/// The five settings under it are all about how the folder tree and the
/// message list behave, so they sit in one group in one place rather than
/// scattered down the page.
pub const SETTINGS_SECTION: &str = "Folders and Message Lists";

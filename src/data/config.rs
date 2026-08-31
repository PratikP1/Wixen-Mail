//! Configuration management
//!
//! Handles application settings, account configurations, and persistence.

use crate::common::paths::AppPaths;
use crate::common::{Error, Result, types::Id};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Application-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application version
    pub version: String,
    /// Default folder for downloads
    pub download_folder: PathBuf,
    /// Whether to check links against Google Safe Browsing.
    ///
    /// Off unless somebody turns it on, and inert without an API key. What it
    /// sends is in the setting's own text and in docs/privacy.md: the lists
    /// come down to this machine and links are compared here, so nothing goes
    /// to Google for ordinary mail.
    #[serde(default)]
    pub check_links_with_google: bool,
    /// Whether to read each message here and mark it when it looks wrong.
    ///
    /// On unless somebody turns it off, and the one setting in this file that
    /// starts on while still being about safety. It sends nothing to anybody:
    /// the reading happens on this computer, over text already in hand, and the
    /// most it can do is put a word in the safety column. It is also what mail
    /// arriving over IMAP has had since the body fetch was written, so starting
    /// it off would take a marking away from people who never asked for that.
    ///
    /// Deliberately not the same switch as `check_links_with_google` above.
    /// That one can put four bytes of a link on the wire, so turning this on to
    /// get a check that touches nothing would mean agreeing to something else
    /// entirely.
    #[serde(default = "default_true")]
    pub look_at_message_contents: bool,
    /// Theme name
    pub theme: String,
    /// Font size
    pub font_size: u32,
    /// Log level
    pub log_level: String,
    /// Show preview dialog before sending emails
    #[serde(default = "default_true")]
    pub preview_before_send: bool,
    /// Whether to keep a copy of sent mail here even when the server saved one.
    ///
    /// Off unless somebody asks for it, and off in every settings file written
    /// before it existed. On, the Sent folder lists each message twice once the
    /// server's own copy comes down on the next check for mail: one row filed
    /// here and one from the server. That is what it does rather than a fault,
    /// and both the setting's label and its description say so.
    ///
    /// It has no effect on an account whose mail is collected over POP, which
    /// has no server folder and whose copy is always the one on this computer,
    /// and none on a copy the server refuses, which is kept here regardless.
    #[serde(default)]
    pub keep_sent_mail_on_this_computer: bool,
    /// Whether a message being written opens with the account's signature.
    ///
    /// On unless somebody turns it off, which is what every message has done
    /// since signatures reached the composer, so a settings file written before
    /// this existed keeps the behaviour it had. Off, the composer opens with an
    /// empty body; the signature stays on the account and can still be put in
    /// by hand.
    ///
    /// The compose tab offered this answer for as long as it has existed, hard
    /// set to yes, saved by nothing. A screen reader read out a setting that
    /// was not there.
    #[serde(default = "default_true")]
    pub add_signature_automatically: bool,
    /// Leave a picture a message only points at unfetched.
    ///
    /// On unless somebody turns it off, and this is the one setting in this
    /// file that starts on and takes something away. Fetching such a picture
    /// tells the server it came from that this message was opened, by this
    /// computer, at this moment, which is the whole of how mail tracking
    /// works: a single invisible pixel, in nearly every marketing message and
    /// a good deal worse.
    ///
    /// A picture the message carries is not affected. It is already here and
    /// showing it tells nobody anything.
    #[serde(default = "default_true")]
    pub hold_back_remote_pictures: bool,
    /// The typeface the item lists are drawn in.
    ///
    /// Empty means whatever Windows uses, which is the default and is stored
    /// as an empty string rather than as today's system font name: storing the
    /// name would freeze it, so somebody who later changed their Windows font
    /// would keep the old one here with nothing saying why.
    ///
    /// A name that is not installed is not passed to the toolkit. Windows
    /// substitutes silently for a face it does not have, so the check and the
    /// sentence about it live in [`crate::application::font_choice`].
    #[serde(default)]
    pub font_family: String,
    /// Say at startup when another program holds a default this could hold.
    ///
    /// Off by default. Somebody who has chosen another mail program on purpose
    /// does not need telling about it every time they start this one, and a
    /// message that appears every time is one that gets dismissed without
    /// being read.
    #[serde(default)]
    pub check_default_programs_at_startup: bool,
    /// Keep running in the notification area when the window is closed.
    ///
    /// Off by default. Closing a window and having the program carry on is not
    /// what closing a window means to most people, and it is worse than most
    /// for somebody who cannot see the screen: the window goes, the reading
    /// stops, and nothing says the program is still there. So it is asked for
    /// rather than assumed, it is announced the first time it happens, and
    /// Quit always really quits. [`crate::application::closing`] holds those
    /// rules, including the one that refuses to hide when no tray icon
    /// appeared, which would leave a running program with no way back into it.
    #[serde(default)]
    pub keep_running_in_the_tray: bool,
    /// Slide rather than jump when a view scrolls.
    ///
    /// Off by default, which is what everything did before this existed. It
    /// can only ever be honoured: when Windows is set to reduce animation this
    /// is ignored, because a vestibular reaction is not a preference and
    /// somebody who has told their computer once should not have to tell every
    /// program again. [`crate::application::scrolling`] holds that rule.
    #[serde(default)]
    pub smooth_scrolling: bool,
    /// Bring the chosen message back into view when the list is rebuilt.
    ///
    /// On by default, which is what the list already did. Turning it off
    /// leaves the view where it is when a sync finishes underneath somebody
    /// reading down a list. The selection itself never moves either way, so
    /// this cannot lose somebody their place, only leave it off screen.
    #[serde(default = "default_true")]
    pub keep_selected_message_in_view: bool,
    /// Open in All Inboxes rather than with no folder chosen.
    ///
    /// The folder tree comes up with nothing selected, deliberately: forcing
    /// the cursor into it would take somebody out of whatever they were
    /// reading when a sync rebuilt it. That rule is right for every rebuild
    /// after the first and leaves the first one landing nowhere, with no mail
    /// listed until somebody arrows onto a folder.
    ///
    /// Off by default, because on is a change to where everybody opens and
    /// somebody with one account has no use for a combined list of one.
    #[serde(default)]
    pub start_in_all_inboxes: bool,
    /// How a row of the folder tree that holds other rows says what is unread.
    ///
    /// A folder with folders under it, an account branch, and the group of
    /// things kept on this computer each have two numbers to give: what is
    /// unread in the row itself, and what is unread in it and everything
    /// beneath. Stored as the words
    /// [`crate::application::folder_settings::UnreadOnAParent`] reads back, so
    /// a value nobody recognises falls to the default rather than to whichever
    /// branch happens to be written first.
    ///
    /// Both numbers always, by default. The alternative gives a row its own
    /// number while it is open, which means the row changes what it says as
    /// somebody opens and closes branches around it. For a person arrowing
    /// down a tree by ear, a row whose wording depends on a state they have to
    /// remember is a row they have to stop and check, and that cost is paid on
    /// every row rather than on the one they were curious about.
    ///
    /// The `#[serde(default = "...")]` is not optional. Without it every
    /// settings file already on disk fails to parse, and a settings file that
    /// fails to parse takes every other setting with it, which is what
    /// `test_a_settings_file_written_before_directories_existed_still_reads`
    /// exists because of.
    #[serde(default = "default_unread_on_a_parent")]
    pub unread_on_a_parent: String,
    /// Whether Empty Folder reaches the folders filed under the one chosen.
    ///
    /// Yes by default, D-34, which is the destructive reading of the two. That
    /// is why the confirmation carries the whole cost where this is on: the
    /// folder, the total, how many subfolders, and whether the messages move or
    /// go. Somebody who has never opened this screen is agreeing to the wider
    /// thing, so the wider thing is what the question has to describe.
    ///
    /// Kept apart from [`AppConfig::mark_read_reaches_subfolders`] deliberately,
    /// D-35: one destroys mail and the other loses your place, and neither has
    /// an undo. One setting covering both would make somebody who wants the
    /// safer reach on the reading side pay for it on the destructive one, or
    /// the other way round.
    ///
    /// The `#[serde(default = "...")]` is not optional. Without it every
    /// settings file already on disk fails to parse, and a settings file that
    /// fails to parse takes every other setting with it.
    #[serde(default = "default_true")]
    pub empty_reaches_subfolders: bool,
    /// Whether Mark Folder Read reaches the folders filed under the one chosen.
    ///
    /// Yes by default, D-35. Its own setting rather than a share of the one
    /// above, for the reason written there: this one loses somebody their place
    /// in a folder they had not finished reading, which is a real cost and a
    /// different one from destroying mail.
    #[serde(default = "default_true")]
    pub mark_read_reaches_subfolders: bool,
    /// How far a conversation reaches when its row is counted up.
    ///
    /// The whole account by default, D-08: a conversation's row appears in
    /// every folder it touches and says the same thing wherever somebody is
    /// standing. A count that changed as they walked between folders would
    /// give no way to tell which of the two answers was about the
    /// conversation.
    ///
    /// A choice rather than a check box, for the reason
    /// `unread_on_a_parent` gives: neither answer is the absence of the other,
    /// and a box labelled for one would have to word the other as "not that".
    ///
    /// The `#[serde(default = "...")]` is not optional. Without it every
    /// settings file already on disk fails to parse, and a settings file that
    /// fails to parse takes every other setting with it.
    #[serde(default = "default_a_conversation_reaches")]
    pub a_conversation_reaches: String,
    /// How far Delete reaches on a collapsed conversation row, D-07.
    ///
    /// A separate question from `a_conversation_reaches` above, which is how
    /// far a row *counts*. Reading about a whole account and deleting out of
    /// one folder is a coherent thing to want, and one setting for both would
    /// make somebody choose between a count they like and a delete they trust.
    ///
    /// Defaults to this folder's messages, the narrower reach, because a
    /// collapsed row is a row whose contents nobody can see.
    ///
    /// The `#[serde(default = "...")]` is not optional, for the reason the
    /// setting above gives: without it every settings file already on disk
    /// fails to parse, and one that fails to parse takes every other setting
    /// with it.
    #[serde(default = "default_deleting_a_conversation_row")]
    pub deleting_a_conversation_row: String,
    /// The language messages are spell-checked in.
    ///
    /// A BCP 47 tag such as `en-GB` where Windows is doing the checking, and a
    /// bare code such as `en` where the built-in list is. Nothing else reads
    /// this: the interface itself is not translated, and calling it the
    /// interface language, as the settings dialog once did, was a claim.
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether to check the spelling of a message before sending it.
    ///
    /// On by default. Somebody who does not want it can say so, and the
    /// question is then never asked, which is different from asking and being
    /// dismissed every time.
    #[serde(default = "default_true")]
    pub check_spelling_before_send: bool,
    /// What Wixen Mail may change at a server, for every account.
    ///
    /// Defaults to `default_allowed` below, which names the answer instead of
    /// saying it again. A config file written before this existed gets the same
    /// thing, which is the point: an upgrade should take permissions away
    /// rather than carry on sending from code that has never been proved.
    #[serde(default = "default_allowed")]
    pub allowed_changes: crate::application::allowed::Allowed,
    /// What one account may change, when it differs from the setting above.
    ///
    /// Kept here, keyed by account id, rather than as a field on `Account`.
    /// `Account` is built in eleven places, so a field on it is eleven chances
    /// to write the permissive answer by accident; a map that answers with the
    /// application-wide setting for an id it has never seen cannot be got
    /// wrong that way.
    ///
    /// Nothing writes it. It is read by `allowed_for` below and honoured all
    /// the way out to the provider clients, and the settings screen writes the
    /// application-wide answer only, so every account gets that one. The gap is
    /// written down here rather than left to be rediscovered: the testing page
    /// and the first-run screen both used to offer this as a control somebody
    /// could reach, and neither the screen nor the sentence a sync says names
    /// an account any more. Wiring it means a control per account on the
    /// settings screen, and the answer it writes can only ever narrow the
    /// application-wide one.
    #[serde(default)]
    pub allowed_per_account: HashMap<String, crate::application::allowed::Allowed>,
    /// The directory each account looks people up in, by account id.
    ///
    /// Empty on a fresh installation and on every settings file written before
    /// this existed, and that is the privacy decision rather than an
    /// oversight. Looking a name up sends what somebody is typing to a server
    /// before they have decided to send anything at all, so it happens only
    /// for an account where somebody has named the directory it should go to.
    ///
    /// Kept here, keyed by account id, rather than as a field on `Account`,
    /// for the same reason `allowed_per_account` above is: `Account` is built
    /// in eleven places, and a map that answers "no directory" for an id it
    /// has never seen cannot be got wrong by one of them forgetting a field.
    #[serde(default)]
    pub directories: HashMap<String, crate::service::directory::Directory>,
    /// Whether a change to a contact goes to every address book that has that
    /// contact, or only to the one it came from.
    ///
    /// Not a permission and it cannot open the write gate: whether anything is
    /// written at all is decided by `allowed_changes` above, and this only
    /// decides how many of the already-allowed address books an edit reaches.
    /// It is on unless somebody turns it off, because a contact in two address
    /// books is one person, and quietly correcting a phone number in one of
    /// them leaves the other wrong with nothing to say so.
    #[serde(default = "default_true")]
    pub send_contact_changes_everywhere: bool,
    /// The folder each account's last move or copy went to, by account id.
    ///
    /// Filing mail is repetitive: several messages go into the same folder one
    /// after another. Holding the last one lets the destination window open on
    /// it, so filing the next is the shortcut and Enter rather than another
    /// walk through the folder tree.
    ///
    /// Per account, because two accounts have different folders and offering
    /// one account's to the other is offering somewhere the message cannot go.
    #[serde(default)]
    pub last_filed_into: HashMap<String, String>,
    /// Whether to tell a sender when you have read their message.
    ///
    /// Stored as a word rather than as the enum, so a settings file written by
    /// a later version, or edited by hand, reads back as the private answer
    /// instead of failing to parse. `crate::application::receipts::Policy`
    /// decides what each word means, and anything it does not know is "never".
    #[serde(default)]
    pub read_receipts: String,
    /// Which surface a message opens into, formatted or plain text.
    ///
    /// Stored as a word for the same reason as the others: a settings file
    /// written by a later version reads back as the default rather than
    /// failing to parse, and the default here keeps what the sender wrote.
    #[serde(default)]
    pub read_messages_as: String,
    /// Whether the person has been shown what this alpha can and cannot do.
    ///
    /// False on a fresh installation and on an upgrade from before this
    /// existed, so somebody who has been using it already still gets told once
    /// that writing has been switched on and has never been tried for real.
    #[serde(default)]
    pub told_about_the_alpha: bool,
    /// Whether misspellings are marked in the editor as you write.
    ///
    /// One setting, two things, deliberately. It turns on the engine's own
    /// marking, which is what a screen reader reads as the caret crosses a
    /// word, and the sound at the end of a word that is wrong. They are the
    /// same question asked twice, and asking it twice is how people end up
    /// with one on and the other off and no idea why.
    #[serde(default = "default_true")]
    pub check_spelling_as_you_type: bool,
    /// Default sort order for message list
    #[serde(default = "default_sort_order")]
    pub default_sort_order: String,
    /// How dates are shown in lists: "absolute" or "relative".
    ///
    /// Relative says "2 days ago" within the last week, which is three
    /// syllables where a full date is a dozen, and answers the question most
    /// people are actually asking of a date in a mail list.
    #[serde(default = "default_date_style")]
    pub date_style: String,
    /// Day and month order: "auto", "month_first", or "day_first".
    #[serde(default = "default_date_order")]
    pub date_order: String,
    /// How long a message is looked at before it counts as read.
    ///
    /// "immediately", "never", or a number of seconds. The settings window has
    /// offered this since it was written and nothing ever read it back, so the
    /// answer was always immediately, whatever it said on screen.
    #[serde(default = "default_mark_read_after")]
    pub mark_read_after: String,
    /// Whether the Cc and Bcc lines are in the compose window from the start:
    /// "shown" or "hidden".
    #[serde(default = "default_copy_lines")]
    pub copy_lines: String,
    /// The hour the working day starts, 0 to 23.
    ///
    /// Read by the calendar, which says so when an event falls outside it.
    #[serde(default = "default_day_starts")]
    pub working_day_starts: u8,
    /// The hour the working day ends, 1 to 24. Exclusive: 17 means five.
    #[serde(default = "default_day_ends")]
    pub working_day_ends: u8,
    /// Whether the month is a word or a number: "verbal" or "numeric".
    ///
    /// Spelled out is easier to hear and longer to read, and which of those
    /// matters depends on the person and on whether they are listening or
    /// looking at it.
    #[serde(default = "default_date_wording")]
    pub date_wording: String,
    /// Whether the clock runs to twelve or twenty-four: "auto", "12", or "24".
    ///
    /// "auto" follows the machine, so nothing has to be set here to get the
    /// clock the rest of the computer already keeps.
    #[serde(default = "default_clock_hours")]
    pub clock_hours: String,
    /// Whether message text is read aloud.
    ///
    /// Kept across restarts because someone who works in a shared room needs
    /// mail to stay quiet without switching it off again every session.
    #[serde(default)]
    pub mute_message_reading: bool,
    /// Which message list columns are shown, in what order, and the sort.
    ///
    /// Stored as the compact form `ColumnLayout::to_stored` writes. Empty means
    /// nothing has been chosen yet and the per-folder defaults apply. Anyone
    /// who arranges a list they navigate by ear has spent real effort on it,
    /// and losing that on restart is not a small annoyance.
    #[serde(default)]
    pub message_columns: String,
    /// Which channels each event reaches: speech, braille, earcon, visual.
    ///
    /// Stored as the compact form `FeedbackSettings::to_stored` writes. Empty
    /// means the defaults: words on, sounds off. This is the setting that lets
    /// a deaf-blind user drop speech and keep braille, or someone in an open
    /// office swap a sentence of speech for a short tone.
    #[serde(default)]
    pub feedback_channels: String,
    /// Which sound scheme plays when the earcon channel is one an event
    /// reaches. Empty means the built-in "Generated tones", the same
    /// scheme every installation starts on; anything else is a scheme's own
    /// stable id, resolved at startup against the built-in list and
    /// whatever has been imported into the sound-schemes folder.
    #[serde(default)]
    pub sound_scheme_id: String,
    /// Which account new items are created in.
    ///
    /// Set to the first account configured, without asking, because for most
    /// people it is the only one. Empty means nothing has been chosen yet, and
    /// the first account added takes it.
    #[serde(default)]
    pub default_account_id: String,
    /// How often a draft is saved while composing, in minutes.
    ///
    /// Nought means never. See `application::autosave` for why the range stops
    /// at ten.
    #[serde(default = "default_autosave_minutes")]
    pub draft_autosave_minutes: u32,
    /// Default reminder lead-time in minutes (e.g. 15 = remind 15 min before)
    #[serde(default = "default_reminder_minutes")]
    pub default_reminder_minutes: u32,
}

/// What a new or upgraded installation may change.
///
/// `Allowed::FOR_TESTING` holds the answer and the reason for it. This names it
/// rather than saying it a second time, and that is the whole point: a sentence
/// repeating a fact the code already holds goes stale on its own, and nothing
/// here could have stopped it. The check in `tests/house_style.rs` reads prose
/// that names the setting the way the settings screen does, and no line in this
/// file does, so a sentence here claiming the opposite of the truth was
/// measured passing every test in the project.
///
/// Ten of those sentences have been written wrong in this repository. A
/// sentence a check reads may state the answer, because it will be held to it.
/// A sentence no check reads should name the answer instead.
fn default_allowed() -> crate::application::allowed::Allowed {
    crate::application::allowed::Allowed::FOR_TESTING
}

fn default_true() -> bool {
    true
}
/// The language messages are checked in, when nothing has been chosen.
///
/// This machine's own, when something here can check it. It was "en" for
/// everybody, so anybody writing in anything else had every word of it called a
/// mistake until they found the setting, and finding a setting by hearing every
/// word marked wrong is not finding it.
///
/// English when the machine's language is one nothing can check, because
/// English checked is better than nothing checked.
fn default_language() -> String {
    crate::service::spellcheck::language_of_this_machine().unwrap_or_else(|| "en".to_string())
}
fn default_sort_order() -> String {
    "date_newest".to_string()
}
fn default_date_style() -> String {
    "relative".to_string()
}

fn default_mark_read_after() -> String {
    crate::application::reading_habits::MarkRead::default().as_stored()
}

fn default_copy_lines() -> String {
    crate::application::reading_habits::CopyLines::default()
        .as_stored()
        .to_string()
}

fn default_day_starts() -> u8 {
    crate::application::reading_habits::WorkingDay::default().starts
}

fn default_day_ends() -> u8 {
    crate::application::reading_habits::WorkingDay::default().ends
}

fn default_date_wording() -> String {
    "verbal".to_string()
}

fn default_unread_on_a_parent() -> String {
    crate::application::folder_settings::UnreadOnAParent::default()
        .as_str()
        .to_string()
}

fn default_a_conversation_reaches() -> String {
    crate::application::conversations::AConversationReaches::default()
        .as_str()
        .to_string()
}

fn default_deleting_a_conversation_row() -> String {
    crate::application::conversations::DeletingAConversationRow::default()
        .as_str()
        .to_string()
}

fn default_clock_hours() -> String {
    "auto".to_string()
}

fn default_date_order() -> String {
    "auto".to_string()
}

fn default_reminder_minutes() -> u32 {
    15
}

fn default_autosave_minutes() -> u32 {
    crate::application::autosave::AutosaveInterval::default().minutes()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            download_folder: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            check_links_with_google: false,
            look_at_message_contents: default_true(),
            theme: "default".to_string(),
            font_size: 12,
            date_style: default_date_style(),
            date_order: default_date_order(),
            date_wording: default_date_wording(),
            mark_read_after: default_mark_read_after(),
            copy_lines: default_copy_lines(),
            working_day_starts: default_day_starts(),
            working_day_ends: default_day_ends(),
            clock_hours: default_clock_hours(),
            mute_message_reading: false,
            default_account_id: String::new(),
            draft_autosave_minutes: default_autosave_minutes(),
            message_columns: String::new(),
            feedback_channels: String::new(),
            sound_scheme_id: String::new(),
            log_level: "info".to_string(),
            preview_before_send: true,
            // The safe answer is the one that changes nothing: the server's own
            // copy is what Sent has always listed.
            keep_sent_mail_on_this_computer: false,
            add_signature_automatically: default_true(),
            start_in_all_inboxes: false,
            unread_on_a_parent: default_unread_on_a_parent(),
            a_conversation_reaches: default_a_conversation_reaches(),
            deleting_a_conversation_row: default_deleting_a_conversation_row(),
            empty_reaches_subfolders: default_true(),
            mark_read_reaches_subfolders: default_true(),
            hold_back_remote_pictures: default_true(),
            font_family: String::new(),
            check_default_programs_at_startup: false,
            keep_running_in_the_tray: false,
            smooth_scrolling: false,
            keep_selected_message_in_view: default_true(),
            language: default_language(),
            check_spelling_before_send: true,
            allowed_changes: default_allowed(),
            allowed_per_account: HashMap::new(),
            directories: HashMap::new(),
            send_contact_changes_everywhere: default_true(),
            last_filed_into: HashMap::new(),
            read_receipts: crate::application::receipts::Policy::Never
                .as_str()
                .to_string(),
            read_messages_as: crate::application::reading_style::Style::Formatted
                .as_str()
                .to_string(),
            told_about_the_alpha: false,
            check_spelling_as_you_type: default_true(),
            default_sort_order: default_sort_order(),
            default_reminder_minutes: default_reminder_minutes(),
        }
    }
}

impl AppConfig {
    /// What this account may change, before the command line narrows it.
    ///
    /// The account's own answer when it has one, and the application-wide
    /// setting otherwise, with the application-wide setting applied either
    /// way. So a per-account entry can only ever be narrower, never wider:
    /// somebody who turns everything off globally has turned it off, whatever
    /// any account says.
    pub fn allowed_for(&self, account_id: &str) -> crate::application::allowed::Allowed {
        self.allowed_per_account
            .get(account_id)
            .copied()
            .unwrap_or(self.allowed_changes)
            .and(self.allowed_changes)
    }

    /// The directory this account looks people up in, if it names one.
    ///
    /// `None` means nothing is ever asked of any server while somebody types a
    /// recipient, which is what a fresh installation does.
    ///
    /// An entry with both boxes left blank counts as naming nothing. An entry
    /// with one of them filled in is handed back as it stands, so that
    /// `service::directory` can say which part is missing: it writes those
    /// sentences, and a second opinion here would be a second answer to the
    /// same question.
    pub fn directory_for(&self, account_id: &str) -> Option<&crate::service::directory::Directory> {
        self.directories.get(account_id).filter(|directory| {
            !directory.url.trim().is_empty() || !directory.search_under.trim().is_empty()
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.font_size < 8 || self.font_size > 72 {
            return Err(Error::Config(
                "Font size must be between 8 and 72".to_string(),
            ));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(Error::Config(format!(
                "Invalid log level: {}. Must be one of: error, warn, info, debug, trace",
                self.log_level
            )));
        }

        Ok(())
    }
}

/// Account-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Account ID
    pub id: Id,
    /// Account name
    pub name: String,
    /// Check interval in minutes
    pub check_interval_minutes: u32,
    /// Signature text
    pub signature: Option<String>,
    /// Default folder
    pub default_folder: String,
    /// Auto-download attachments
    pub auto_download_attachments: bool,
}

impl AccountConfig {
    /// Create a new account configuration
    pub fn new(id: Id, name: String) -> Self {
        Self {
            id,
            name,
            check_interval_minutes: 15,
            signature: None,
            default_folder: "INBOX".to_string(),
            auto_download_attachments: false,
        }
    }

    /// Validate account configuration
    pub fn validate(&self) -> Result<()> {
        if self.check_interval_minutes < 1 || self.check_interval_minutes > 1440 {
            return Err(Error::Config(
                "Check interval must be between 1 and 1440 minutes".to_string(),
            ));
        }

        Ok(())
    }
}

/// Configuration manager with file persistence
pub struct ConfigManager {
    /// Application configuration
    app_config: AppConfig,
    /// Account configurations
    account_configs: HashMap<Id, AccountConfig>,
    /// Configuration directory
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        Self::in_dir(AppPaths::resolve()?.config_dir())
    }

    /// Open the settings folder and read what is in it.
    ///
    /// Most callers want this rather than [`new`](Self::new): they need the
    /// stored values, and both steps fail the same way from their point of
    /// view.
    pub fn load_stored() -> Result<Self> {
        let mut manager = Self::new()?;
        manager.load()?;
        Ok(manager)
    }

    /// Keep configuration in a directory of the caller's choosing.
    ///
    /// Tests use this so they never write into the profile of whoever is
    /// running them.
    fn in_dir(config_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&config_dir).map_err(|e| {
            Error::Config(format!("Could not create {}: {e}", config_dir.display()))
        })?;

        Ok(Self {
            app_config: AppConfig::default(),
            account_configs: HashMap::new(),
            config_dir,
        })
    }

    /// Get app config file path
    fn app_config_path(&self) -> PathBuf {
        self.config_dir.join("app_config.json")
    }

    /// Get account config file path
    fn account_config_path(&self, account_id: &str) -> PathBuf {
        self.config_dir.join(format!("account_{}.json", account_id))
    }

    /// Load configuration from file
    pub fn load(&mut self) -> Result<()> {
        // Load app config
        let app_config_path = self.app_config_path();
        if app_config_path.exists() {
            let content = fs::read_to_string(&app_config_path)
                .map_err(|e| Error::Config(format!("Failed to read app config: {}", e)))?;
            self.app_config = serde_json::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse app config: {}", e)))?;
            self.app_config.validate()?;
        } else {
            // Create default config file
            self.save_app_config()?;
        }

        // Load account configs
        for entry in fs::read_dir(&self.config_dir)
            .map_err(|e| Error::Config(format!("Failed to read config directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| Error::Config(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("account_") && filename_str.ends_with(".json") {
                    let content = fs::read_to_string(&path).map_err(|e| {
                        Error::Config(format!("Failed to read account config: {}", e))
                    })?;
                    let account_config: AccountConfig =
                        serde_json::from_str(&content).map_err(|e| {
                            Error::Config(format!("Failed to parse account config: {}", e))
                        })?;
                    account_config.validate()?;
                    self.account_configs
                        .insert(account_config.id.clone(), account_config);
                }
            }
        }

        Ok(())
    }

    /// Save app configuration to file
    fn save_app_config(&self) -> Result<()> {
        self.app_config.validate()?;
        let content = serde_json::to_string_pretty(&self.app_config)
            .map_err(|e| Error::Config(format!("Failed to serialize app config: {}", e)))?;
        fs::write(self.app_config_path(), content)
            .map_err(|e| Error::Config(format!("Failed to write app config: {}", e)))?;
        Ok(())
    }

    /// Save all configurations to files
    pub fn save(&self) -> Result<()> {
        // Save app config
        self.save_app_config()?;

        // Save account configs
        for account_config in self.account_configs.values() {
            account_config.validate()?;
            let content = serde_json::to_string_pretty(account_config)
                .map_err(|e| Error::Config(format!("Failed to serialize account config: {}", e)))?;
            let path = self.account_config_path(&account_config.id);
            fs::write(path, content)
                .map_err(|e| Error::Config(format!("Failed to write account config: {}", e)))?;
        }

        Ok(())
    }

    /// Get application configuration
    pub fn app_config(&self) -> &AppConfig {
        &self.app_config
    }

    /// Get mutable application configuration
    pub fn app_config_mut(&mut self) -> &mut AppConfig {
        &mut self.app_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_chosen_in_one_session_are_there_in_the_next() {
        // Nothing covered the round trip through the folder, so reading the
        // settings back could have done nothing at all and every test still
        // passed. What that looks like is every setting somebody chose
        // reverting on restart, silently, including what this build is allowed
        // to change at a server.
        let dir = tempfile::TempDir::new().expect("a temporary folder");
        let mut first =
            ConfigManager::in_dir(dir.path().join("config")).expect("a settings folder");
        first.app_config_mut().date_wording = "numeric".to_string();
        first.app_config_mut().working_day_starts = 6;
        first.app_config_mut().allowed_changes = crate::application::allowed::Allowed::NOTHING;
        first.save().expect("the settings to be written");

        let mut later =
            ConfigManager::in_dir(dir.path().join("config")).expect("the same settings folder");
        later.load().expect("the settings to be read back");

        assert_eq!(later.app_config().date_wording, "numeric");
        assert_eq!(later.app_config().working_day_starts, 6);
        assert_eq!(
            later.app_config().allowed_changes,
            crate::application::allowed::Allowed::NOTHING,
            "an upgrade would have handed back permissions somebody took away"
        );
    }

    #[test]
    fn test_config_manager() {
        let dir = tempfile::TempDir::new().unwrap();
        let manager = ConfigManager::in_dir(dir.path().join("config"));
        assert!(manager.is_ok());
    }

    #[test]
    fn test_settings_are_written_to_the_folder_they_were_given() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_dir = dir.path().join("config");
        let manager = ConfigManager::in_dir(config_dir.clone()).unwrap();

        manager.save().unwrap();

        assert!(config_dir.join("app_config.json").is_file());
    }

    #[test]
    fn test_the_settings_folder_is_created_if_it_is_not_there() {
        // A first run has an empty application data folder, and the manager is
        // built before anything asks it to save.
        let dir = tempfile::TempDir::new().unwrap();
        let config_dir = dir.path().join("missing").join("config");

        ConfigManager::in_dir(config_dir.clone()).unwrap();

        assert!(config_dir.is_dir());
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "default");
        assert_eq!(config.font_size, 12);
    }

    #[test]
    fn test_app_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.font_size = 150;
        assert!(config.validate().is_err());

        config.font_size = 12;
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_account_config() {
        let config = AccountConfig::new("acc-1".to_string(), "Test Account".to_string());
        assert_eq!(config.id, "acc-1");
        assert_eq!(config.name, "Test Account");
        assert_eq!(config.check_interval_minutes, 15);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_account_config_validation() {
        let mut config = AccountConfig::new("acc-1".to_string(), "Test".to_string());
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 0;
        assert!(config.validate().is_err());

        config.check_interval_minutes = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_app_config_defaults_complete() {
        let config = AppConfig::default();
        // Not "en". The language follows this machine, so asserting English
        // only passes on an English machine, and it passed here while a fresh
        // installation was quietly hardcoding "en" for everybody.
        assert!(
            config.language.len() >= 2
                && config
                    .language
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "{} is not a language the spell checker could be asked for",
            config.language
        );
        assert_eq!(config.default_sort_order, "date_newest");
        assert_eq!(config.default_reminder_minutes, 15);
        assert!(config.preview_before_send);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_app_config_validates_valid_log_levels() {
        let valid = ["error", "warn", "info", "debug", "trace"];
        for level in &valid {
            let config = AppConfig {
                log_level: level.to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_ok(), "Expected {} to be valid", level);
        }
    }

    #[test]
    fn test_app_config_font_size_boundaries() {
        let cases = [(8, true), (72, true), (7, false), (73, false)];

        for (font_size, expect_valid) in cases {
            let config = AppConfig {
                font_size,
                ..Default::default()
            };
            assert_eq!(
                config.validate().is_ok(),
                expect_valid,
                "Expected font size {} validity to be {}",
                font_size,
                expect_valid
            );
        }
    }

    #[test]
    fn test_account_config_check_interval_boundaries() {
        let mut config = AccountConfig::new("a".to_string(), "A".to_string());

        config.check_interval_minutes = 1;
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 1440;
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 0;
        assert!(config.validate().is_err());

        config.check_interval_minutes = 1441;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_asking_to_start_in_all_inboxes_survives_being_written_and_read_back() {
        // A setting that does not persist is a setting somebody ticks once a
        // session and never notices is not being kept. It also has to read
        // back from a file written before it existed, which every settings
        // file on every machine already is.
        let mut config = AppConfig::default();
        assert!(
            !config.start_in_all_inboxes,
            "on by default would move where everybody opens"
        );

        config.start_in_all_inboxes = true;
        let written = serde_json::to_string(&config).unwrap();
        let read_back: AppConfig = serde_json::from_str(&written).unwrap();
        assert!(read_back.start_in_all_inboxes);

        // A real settings file with this one key taken out, which is what
        // every file on every machine looks like until the first save after
        // this ships.
        let mut older: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&AppConfig::default()).unwrap()).unwrap();
        older
            .as_object_mut()
            .expect("the config is an object")
            .remove("start_in_all_inboxes")
            .expect("the key is written, so removing it is a real before-and-after");
        let older: AppConfig = serde_json::from_value(older)
            .expect("a settings file written before this existed has to still load");
        assert!(!older.start_in_all_inboxes);
    }

    #[test]
    fn test_app_config_serialization_round_trip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, config.theme);
        assert_eq!(deserialized.font_size, config.font_size);
        assert_eq!(deserialized.language, config.language);
    }
}

#[cfg(test)]
mod permission_tests {
    use super::*;
    use crate::application::allowed::Allowed;

    fn config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn test_a_new_installation_can_change_tasks_and_not_mail() {
        // What an alpha tester gets without touching anything.
        assert_eq!(config().allowed_for("any-account"), Allowed::FOR_TESTING);
    }

    #[test]
    fn test_an_account_nobody_has_set_follows_the_application_wide_setting() {
        let mut settings = config();
        settings.allowed_changes = Allowed::EVERYTHING;

        assert_eq!(settings.allowed_for("never-seen"), Allowed::EVERYTHING);
    }

    #[test]
    fn test_an_account_can_be_narrower_than_the_setting() {
        // The case the whole thing is for: everything allowed generally, and
        // one real account marked read-only while it is being tested against.
        let mut settings = config();
        settings.allowed_changes = Allowed::EVERYTHING;
        settings
            .allowed_per_account
            .insert("my-real-mail".to_string(), Allowed::NOTHING);

        assert_eq!(settings.allowed_for("my-real-mail"), Allowed::NOTHING);
        assert_eq!(settings.allowed_for("a-throwaway"), Allowed::EVERYTHING);
    }

    #[test]
    fn test_an_account_cannot_be_wider_than_the_setting() {
        // Turning everything off has to mean off. An account entry left over
        // from before must not quietly put it back.
        let mut settings = config();
        settings.allowed_changes = Allowed::NOTHING;
        settings
            .allowed_per_account
            .insert("eager".to_string(), Allowed::EVERYTHING);

        assert_eq!(settings.allowed_for("eager"), Allowed::NOTHING);
    }

    #[test]
    fn test_a_settings_file_written_before_these_existed_reads_the_way_it_should() {
        // Every settings file on disk predates most of these, so what they
        // fall back to is what an upgrade gets. Several of them decide how a
        // date or a time is spoken, and a wrong one is not an error anybody
        // sees: it is every date read the wrong way round, with nothing on
        // screen to say why.
        //
        // Built by taking the fields back out of a current config rather than
        // hand-writing the older shape, so it does not break the next time an
        // unrelated field is added.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        for gone in [
            "check_spelling_as_you_type",
            "default_sort_order",
            "date_style",
            "date_order",
            "mark_read_after",
            "copy_lines",
            "working_day_starts",
            "working_day_ends",
            "date_wording",
            "clock_hours",
            "draft_autosave_minutes",
            "default_reminder_minutes",
            "unread_on_a_parent",
        ] {
            assert!(
                fields.remove(gone).is_some(),
                "{gone} is not written to the settings file any more, so this test covers nothing"
            );
        }

        let parsed: AppConfig =
            serde_json::from_value(older).expect("an older settings file still opens");

        assert_eq!(
            crate::application::folder_settings::UnreadOnAParent::from_stored(
                &parsed.unread_on_a_parent
            ),
            crate::application::folder_settings::UnreadOnAParent::BothAlways,
            "a settings file with no such key gives both numbers, D-24"
        );
        assert_eq!(parsed.date_style, "relative");
        assert_eq!(parsed.date_order, "auto");
        assert_eq!(parsed.date_wording, "verbal");
        assert_eq!(parsed.clock_hours, "auto");
        assert_eq!(parsed.default_sort_order, "date_newest");
        assert_eq!(parsed.default_reminder_minutes, 15);
        assert!(
            parsed.check_spelling_as_you_type,
            "spelling would stop being checked for everybody upgrading"
        );

        // These belong to the module that owns the setting. What matters here
        // is that the field falls back to it rather than to nothing.
        assert_eq!(
            parsed.mark_read_after,
            crate::application::reading_habits::MarkRead::default().as_stored()
        );
        assert_eq!(
            parsed.copy_lines,
            crate::application::reading_habits::CopyLines::default().as_stored()
        );
        assert_eq!(
            parsed.working_day_starts,
            crate::application::reading_habits::WorkingDay::default().starts
        );
        assert_eq!(
            parsed.working_day_ends,
            crate::application::reading_habits::WorkingDay::default().ends
        );
        assert_eq!(
            parsed.draft_autosave_minutes,
            crate::application::autosave::AutosaveInterval::default().minutes()
        );
        assert!(
            parsed.working_day_starts < parsed.working_day_ends,
            "the working day ends before it starts"
        );
    }

    #[test]
    fn test_a_settings_file_written_before_the_two_reach_settings_existed_reaches_the_subfolders() {
        // D-34 and D-35 both default to including what is filed underneath, so
        // an upgrade has to arrive at the same answer a fresh installation
        // does. The one that matters is Empty: falling back to `false` would
        // quietly narrow a destructive command, which sounds like the safe
        // mistake and is not. Somebody who empties a tree, hears it done, and
        // finds the subfolders still full has been told something untrue about
        // their own mail.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        for gone in ["empty_reaches_subfolders", "mark_read_reaches_subfolders"] {
            assert!(
                fields.remove(gone).is_some(),
                "{gone} is not written to the settings file any more, so this test covers nothing"
            );
        }

        let parsed: AppConfig =
            serde_json::from_value(older).expect("an older settings file still opens");

        assert!(parsed.empty_reaches_subfolders, "D-34 defaults to yes");
        assert!(parsed.mark_read_reaches_subfolders, "D-35 defaults to yes");
    }

    #[test]
    fn test_a_fresh_installation_reaches_the_subfolders_with_both_commands() {
        // The other half of the pair above. A default written twice is a
        // default that drifts, which is what the language setting did.
        assert!(AppConfig::default().empty_reaches_subfolders);
        assert!(AppConfig::default().mark_read_reaches_subfolders);
    }

    #[test]
    fn test_a_settings_file_written_before_a_conversation_had_a_reach_reaches_the_whole_account() {
        // D-08's answer, and an upgrade has to arrive at it the same way a
        // fresh installation does. Falling back to the folder would make a
        // conversation say a different size in every folder it touches, which
        // is the thing D-08 exists to stop.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("a_conversation_reaches").is_some(),
            "a_conversation_reaches is not written to the settings file any more, so this test covers nothing"
        );

        let parsed: AppConfig =
            serde_json::from_value(older).expect("an older settings file still opens");

        assert_eq!(
            crate::application::conversations::AConversationReaches::from_stored(
                &parsed.a_conversation_reaches
            ),
            crate::application::conversations::AConversationReaches::TheWholeAccount
        );
    }

    #[test]
    fn test_a_settings_file_written_before_a_conversation_row_could_be_deleted_takes_this_folder() {
        // The narrower reach on an upgrade, deliberately. Somebody whose
        // settings file predates this has never been asked how far a delete on
        // a collapsed row should go, and the answer that destroys less is the
        // only honest thing to assume on their behalf.
        use crate::application::conversations::DeletingAConversationRow;
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("deleting_a_conversation_row").is_some(),
            "deleting_a_conversation_row is not written to the settings file any more, so this \
             test covers nothing"
        );

        let parsed: AppConfig =
            serde_json::from_value(older).expect("an older settings file still opens");

        assert_eq!(
            DeletingAConversationRow::from_stored(&parsed.deleting_a_conversation_row),
            DeletingAConversationRow::ThisFoldersMessages
        );
    }

    #[test]
    fn test_a_fresh_installation_deletes_only_this_folders_messages_from_a_conversation_row() {
        // The other half of the pair above. A default written twice is a
        // default that drifts, which is what the language setting did.
        use crate::application::conversations::DeletingAConversationRow;
        assert_eq!(
            DeletingAConversationRow::from_stored(
                &AppConfig::default().deleting_a_conversation_row
            ),
            DeletingAConversationRow::ThisFoldersMessages
        );
    }

    #[test]
    fn test_how_far_deleting_a_row_reaches_reads_back_by_the_words_it_was_offered_under() {
        use crate::application::conversations::DeletingAConversationRow;
        for option in DeletingAConversationRow::ALL {
            assert_eq!(DeletingAConversationRow::from_words(option.words()), option);
            assert_eq!(
                DeletingAConversationRow::from_stored(option.as_str()),
                option
            );
        }
    }

    #[test]
    fn test_a_fresh_installation_counts_a_conversation_across_the_whole_account() {
        // The other half of the pair above. A default written twice is a
        // default that drifts, which is what the language setting did.
        assert_eq!(
            crate::application::conversations::AConversationReaches::from_stored(
                &AppConfig::default().a_conversation_reaches
            ),
            crate::application::conversations::AConversationReaches::TheWholeAccount
        );
    }

    #[test]
    fn test_the_reach_a_conversation_has_reads_back_by_the_words_it_was_offered_under() {
        // Read back by the words rather than the row number, the way
        // `font_family` and `unread_on_a_parent` are, so a list that differed
        // between showing and saving cannot store an answer nobody chose.
        use crate::application::conversations::AConversationReaches;

        for option in AConversationReaches::ALL {
            assert_eq!(AConversationReaches::from_words(option.words()), option);
            assert_eq!(AConversationReaches::from_stored(option.as_str()), option);
        }
        // Words and stored values nothing recognises fall to the default,
        // which is the answer D-08 chose rather than whichever branch happens
        // to be written first.
        assert_eq!(
            AConversationReaches::from_words("something a later version offered"),
            AConversationReaches::TheWholeAccount
        );
        assert_eq!(
            AConversationReaches::from_stored("hand_edited"),
            AConversationReaches::TheWholeAccount
        );
        // The two options have to be two, or the setting is offering one
        // answer twice.
        assert_ne!(
            AConversationReaches::TheWholeAccount.words(),
            AConversationReaches::ThisFolderOnly.words()
        );
    }

    #[test]
    fn test_the_two_reach_settings_are_stored_apart_rather_than_as_one_answer() {
        // D-35 keeps them separate because one destroys mail and the other
        // loses your place. Two fields is what makes that possible, and a test
        // that only ever set them together would not notice them being merged.
        let settings = AppConfig {
            empty_reaches_subfolders: false,
            ..AppConfig::default()
        };

        assert!(!settings.empty_reaches_subfolders);
        assert!(
            settings.mark_read_reaches_subfolders,
            "narrowing Empty narrowed Mark Folder Read too"
        );
    }

    #[test]
    fn test_a_fresh_installation_checks_spelling_in_this_machine_s_language() {
        // A settings file comes into being two ways: this default for a fresh
        // installation, and serde's per-field defaults for a file written
        // before a field existed. They were written out twice and drifted.
        //
        // The language is where it showed. A fresh installation got "en"
        // whatever the machine was set to, so anybody writing in another
        // language had every word of it called a mistake until they found the
        // setting, and finding a setting by hearing every word marked wrong is
        // not finding it. An upgraded file already followed the machine.
        assert_eq!(
            AppConfig::default().language,
            crate::service::spellcheck::language_of_this_machine()
                .unwrap_or_else(|| "en".to_string()),
            "a fresh installation does not check spelling in this machine's language"
        );
    }

    #[test]
    fn test_a_config_written_before_this_existed_reads_as_the_safe_answer() {
        // An upgrade takes permissions away rather than granting them, and a
        // file with no such field at all still parses.
        // Built by taking the two new fields back out of a current one, rather
        // than hand-writing the older shape. A literal would have to list
        // every unrelated field and would break the next time one is added,
        // which is not what this test is about.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        fields.remove("allowed_changes");
        fields.remove("allowed_per_account");

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert_eq!(parsed.allowed_changes, Allowed::FOR_TESTING);
        assert!(parsed.allowed_per_account.is_empty());
    }

    #[test]
    fn test_a_setting_nobody_has_touched_sends_a_change_to_every_address_book_that_has_it() {
        assert!(AppConfig::default().send_contact_changes_everywhere);
    }

    #[test]
    fn test_a_settings_file_written_before_this_setting_existed_still_sends_to_all_of_them() {
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("send_contact_changes_everywhere").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(parsed.send_contact_changes_everywhere);
    }

    #[test]
    fn test_a_setting_nobody_has_touched_looks_at_the_message_itself() {
        // The only new setting here that starts on. It sends nothing anywhere,
        // it only adds a warning, and it is what mail arriving over IMAP has
        // had since the body fetch was written: starting it off would quietly
        // take that away from people who never asked.
        assert!(AppConfig::default().look_at_message_contents);
    }

    #[test]
    fn test_a_settings_file_written_before_this_setting_existed_still_looks_at_the_message() {
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("look_at_message_contents").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(parsed.look_at_message_contents);
    }

    #[test]
    fn test_a_settings_file_written_before_this_existed_keeps_no_extra_copy() {
        // Off on a fresh install and off in a settings file that predates it.
        // Turning it on for somebody who never asked would double every row in
        // their Sent folder, which reads as the folder having gone wrong.
        assert!(!AppConfig::default().keep_sent_mail_on_this_computer);

        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("keep_sent_mail_on_this_computer").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(!parsed.keep_sent_mail_on_this_computer);
    }

    #[test]
    fn test_keeping_a_copy_here_survives_a_restart() {
        // A setting that reads back as the default after a restart is a setting
        // nobody can turn on, and the only sign is sent mail quietly not being
        // where they put it.
        let dir = tempfile::tempdir().expect("a temporary folder");
        {
            let mut manager =
                ConfigManager::in_dir(dir.path().to_path_buf()).expect("a config folder");
            manager.app_config_mut().keep_sent_mail_on_this_computer = true;
            manager.save().expect("the settings to save");
        }

        let mut reopened =
            ConfigManager::in_dir(dir.path().to_path_buf()).expect("the settings to open again");
        reopened.load().expect("the settings to read back");

        assert!(reopened.app_config().keep_sent_mail_on_this_computer);
    }

    #[test]
    fn test_asking_google_about_links_is_still_off_unless_somebody_asks_for_it() {
        // The other half of the pair, kept apart deliberately: one reads the
        // message here and sends nothing, the other can put four bytes of a
        // link on the wire. They cannot share a switch.
        assert!(!AppConfig::default().check_links_with_google);
    }
}

#[cfg(test)]
mod looking_people_up_at_an_organisation {
    use super::*;
    use crate::service::directory::Directory;

    fn a_directory() -> Directory {
        Directory {
            url: "ldaps://directory.example.com".to_string(),
            search_under: "ou=people,dc=example,dc=com".to_string(),
            sign_in_as: None,
        }
    }

    #[test]
    fn test_a_new_installation_looks_nobody_up_anywhere() {
        // The privacy decision, made testable. Every letter typed into a To
        // line would otherwise go to a server, before anybody has decided to
        // send anything, and this is a mail program rather than a program for
        // an organisation. Nothing goes out until somebody has named the
        // directory it should go to.
        assert_eq!(AppConfig::default().directory_for("any-account"), None);
    }

    #[test]
    fn test_an_account_that_names_a_directory_is_looked_up_in_that_one() {
        let mut settings = AppConfig::default();
        settings
            .directories
            .insert("work".to_string(), a_directory());

        assert_eq!(settings.directory_for("work"), Some(&a_directory()));
    }

    #[test]
    fn test_one_account_naming_a_directory_does_not_name_it_for_the_others() {
        // A personal mailbox on the same computer must not have its
        // recipients sent to an employer's directory.
        let mut settings = AppConfig::default();
        settings
            .directories
            .insert("work".to_string(), a_directory());

        assert_eq!(settings.directory_for("personal"), None);
    }

    #[test]
    fn test_a_directory_with_nothing_filled_in_is_no_directory_at_all() {
        // An entry left behind by somebody clearing the boxes must not become
        // a search against an empty address on every keystroke.
        let mut settings = AppConfig::default();
        settings.directories.insert(
            "work".to_string(),
            Directory {
                url: "   ".to_string(),
                search_under: String::new(),
                sign_in_as: None,
            },
        );

        assert_eq!(settings.directory_for("work"), None);
    }

    #[test]
    fn test_a_settings_file_written_before_directories_existed_still_reads() {
        // Every settings file on every machine was written before this, and
        // one that refuses to parse takes every other setting with it.
        let earlier = r#"{"version":"0.1.0","download_folder":".","theme":"default",
            "font_size":12,"log_level":"info"}"#;

        let parsed: AppConfig = serde_json::from_str(earlier).expect("an earlier settings file");

        assert!(parsed.directories.is_empty());
        assert_eq!(parsed.directory_for("work"), None);
    }
}

#[cfg(test)]
mod every_setting_is_acted_on {
    /// The stored settings this application keeps, read out of the struct
    /// itself so a field added later is covered without anybody remembering.
    fn stored_setting_names(source: &str) -> Vec<String> {
        let start = source
            .find("pub struct AppConfig {")
            .expect("the settings struct");
        let body = &source[start..];
        let end = body.find("\n}").expect("the struct ends");

        body[..end]
            .lines()
            .filter_map(|line| line.trim().strip_prefix("pub "))
            .filter_map(|line| line.split_once(':'))
            .map(|(name, _)| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect()
    }

    /// Every source file that ships, except the two that own a setting rather
    /// than acting on one.
    fn files_that_act() -> Vec<std::path::PathBuf> {
        fn walk(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    let shown = path.display().to_string().replace('\\', "/");
                    // `config.rs` defines and stores a setting and
                    // `wx_settings.rs` offers it. Neither is anybody acting
                    // on the answer.
                    if !shown.ends_with("data/config.rs") && !shown.ends_with("wx_settings.rs") {
                        into.push(path);
                    }
                }
            }
        }
        let mut found = Vec::new();
        walk(std::path::Path::new("src"), &mut found);
        found
    }

    #[test]
    fn test_every_setting_somebody_can_change_is_read_by_something() {
        // Ten settings had a real labelled control, saved, survived a
        // restart, and were read by nothing: notifications, checking for
        // updates, the list font size, the download folder, the log level,
        // the default sort order, and four calendar ones. That is worse than
        // dead code. Dead code does nothing and says nothing; these took
        // somebody's answer, told them it was kept, and ignored it. The sort
        // one was sharpest, because the box beside it is live, so the panel
        // half-worked.
        //
        // This lives here rather than in `tests/` because it has to read the
        // half of each file that ships, and `what_ships` is the one answer to
        // that question and is not compiled into a release. Cutting at the
        // first `#[cfg(test)]` instead is the exact mistake that module was
        // written to end: the main window has test modules sitting between
        // stretches of code, and two settings are read below the first one.
        let config = std::fs::read_to_string("src/data/config.rs").expect("the settings");
        let mut ignored = Vec::new();

        for name in stored_setting_names(&config) {
            let read_somewhere = files_that_act().into_iter().any(|path| {
                std::fs::read_to_string(&path)
                    .is_ok_and(|text| crate::common::what_ships::what_ships(&text).contains(&name))
            });
            if !read_somewhere {
                ignored.push(name);
            }
        }

        assert!(
            ignored.is_empty(),
            "{} setting(s) can be changed and are read by nothing, so the \
             answer is taken and ignored:\n  {}",
            ignored.len(),
            ignored.join("\n  ")
        );
    }

    /// The settings screen, the one file the test above deliberately skips.
    const THE_SETTINGS_SCREEN: &str = "src/presentation/wx_settings.rs";

    /// Where a screen's controls live, for the question "does anything offer
    /// this at all".
    const EVERY_SCREEN: &str = "src/presentation";

    /// Settings something offers, but not the settings screen, and where.
    ///
    /// Each entry names the file whose control offers it, and that claim is
    /// checked rather than believed, so an entry cannot rot into a lie after
    /// somebody takes the control it points at away.
    const OFFERED_BY_ANOTHER_SCREEN: [(&str, &str); 3] = [
        // The account manager names the directory an account looks people up
        // in, and which account is the default one to send from. Both are per
        // account, so they belong on the screen that lists accounts.
        ("directories", "src/presentation/wx_account_manager.rs"),
        (
            "default_account_id",
            "src/presentation/wx_account_manager.rs",
        ),
        // Muting what is read aloud is a menu item with a check on it,
        // `ID_MUTE_CONTENT`, because it is reached in a hurry when somebody
        // walks into the room. A settings page is the wrong place for it.
        ("mute_message_reading", "src/presentation/wx_app.rs"),
    ];

    /// Values the program writes down for itself, which nobody chooses.
    ///
    /// Not settings, so no screen should offer them. They sit in the same file
    /// because that is where what survives a restart is kept.
    const NOT_ANYTHING_ANYBODY_CHOOSES: [&str; 2] = [
        // Where the last move or copy went, so the next one opens on it.
        "last_filed_into",
        // Whether the first-run screen has said its piece yet.
        "told_about_the_alpha",
    ];

    /// Stored, read, honoured, and offered by nothing. The defect itself.
    ///
    /// This is the shape the test below exists to catch, sitting in the tree
    /// before the test was written, and `AppConfig::allowed_per_account`'s own
    /// doc comment has said so for some time: the testing page and the
    /// first-run screen both used to offer it and neither does now. Closing it
    /// means a control per account on the settings screen, which is a feature
    /// rather than a line, and it is written up in
    /// `.planning/phases/01-folders-and-conversations/deferred-items.md`.
    ///
    /// Named here rather than hidden by narrowing the test until it cannot see
    /// it. An entry is checked to be still true, so wiring the control turns
    /// this list into a failure asking for the entry to go.
    const STORED_AND_OFFERED_BY_NOTHING: [&str; 1] = ["allowed_per_account"];

    /// The shipping half of one file, or an empty string if it cannot be read.
    fn what_ships_in(path: &str) -> String {
        std::fs::read_to_string(path)
            .map(|text| crate::common::what_ships::what_ships(&text))
            .unwrap_or_default()
    }

    /// Every screen's shipping half, joined.
    fn what_every_screen_ships() -> String {
        fn walk(dir: &std::path::Path, into: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    into.push(what_ships_in(&path.display().to_string()));
                }
            }
        }
        let mut found = Vec::new();
        walk(std::path::Path::new(EVERY_SCREEN), &mut found);
        found.join("\n")
    }

    #[test]
    fn test_every_setting_somebody_can_change_is_offered_by_a_screen() {
        // The mirror of the test above, and the reason it is needed is that
        // the test above cannot ask this. It skips `config.rs` and
        // `wx_settings.rs` by name, with a stated reason, so it catches a
        // setting that is offered and ignored and is structurally blind to one
        // that is stored and never offered. A setting nothing offers is worse
        // than one nothing reads: it is honoured all the way out, so the
        // program behaves in a way the person using it cannot see, cannot
        // change, and has no screen to look at.
        //
        // This lives beside that one, and in `config.rs` rather than in
        // `tests/`, for the same reason it gives about itself: it has to read
        // the half of each file that ships, and `what_ships` is that answer
        // and is not compiled into a release.
        let config = std::fs::read_to_string("src/data/config.rs").expect("the settings");
        let settings_screen = what_ships_in(THE_SETTINGS_SCREEN);

        let excepted: Vec<&str> = OFFERED_BY_ANOTHER_SCREEN
            .iter()
            .map(|(name, _)| *name)
            .chain(NOT_ANYTHING_ANYBODY_CHOOSES)
            .chain(STORED_AND_OFFERED_BY_NOTHING)
            .collect();

        // Every one of them, not the first: five settings arriving together
        // and two of them forgotten should report two.
        let unoffered: Vec<String> = stored_setting_names(&config)
            .into_iter()
            .filter(|name| !settings_screen.contains(name))
            .filter(|name| !excepted.contains(&name.as_str()))
            .collect();

        assert!(
            unoffered.is_empty(),
            "{} setting(s) are stored and survive a restart and no screen \
             offers any of them, so nobody using this can reach them:\n  {}\n\
             Add a labelled control, or say in one of the three lists above \
             this test why there is not one.",
            unoffered.len(),
            unoffered.join("\n  ")
        );
    }

    #[test]
    fn test_a_setting_said_to_be_offered_elsewhere_really_is() {
        // An exception list is the part of a check most likely to go quietly
        // wrong, because nothing re-asks whether its reasons still hold. Each
        // entry names a file, so each entry can be checked.
        let missing: Vec<String> = OFFERED_BY_ANOTHER_SCREEN
            .iter()
            .filter(|(name, screen)| !what_ships_in(screen).contains(name))
            .map(|(name, screen)| format!("{name}, said to be offered by {screen}"))
            .collect();

        assert!(
            missing.is_empty(),
            "{} exception(s) name a screen that does not offer the setting any \
             more, so the setting is now stored and offered by nothing:\n  {}",
            missing.len(),
            missing.join("\n  ")
        );
    }

    #[test]
    fn test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing() {
        // The other direction. Somebody wiring a control for one of these
        // should be told to delete the entry, rather than leaving a list that
        // reads as a live defect after it has been fixed.
        let screens = what_every_screen_ships();

        let now_offered: Vec<&str> = STORED_AND_OFFERED_BY_NOTHING
            .into_iter()
            .filter(|name| screens.contains(name))
            .collect();

        assert!(
            now_offered.is_empty(),
            "{} setting(s) are recorded as offered by nothing and a screen now \
             offers them, which is good news: take them out of \
             STORED_AND_OFFERED_BY_NOTHING:\n  {}",
            now_offered.len(),
            now_offered.join("\n  ")
        );
    }
}

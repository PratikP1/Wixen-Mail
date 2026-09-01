//! What Wixen Mail may do at somebody's provider.
//!
//! Sending a message, removing one from a server, or deleting a task at a
//! provider can hurt somebody, and none of those paths has run for real yet.
//! So they are two answers rather than one, and a new installation allows one
//! of them: tasks, contacts and the calendar go up to a provider, and mail
//! does not. Sending is the deliberate step afterwards, and `data::config`'s
//! `default_allowed` is where that is written down.
//!
//! Reading mail into a local cache cannot hurt anybody, and that is now the
//! argument for a third answer rather than a reason there is no third answer.
//! Fetching a message's text is the one thing here somebody might want to stop
//! for a reason that is not safety: it costs bandwidth, and it puts the text
//! of their mail on this machine. So it is a question that can be asked, and
//! because nothing it does is irreversible, the answer is yes unless somebody
//! says otherwise. That is the opposite direction from the two above, and the
//! field's own comment carries the reason.
//!
//! # Three places can say no, and any one of them is enough
//!
//! The command line, the application's own setting, and the account. A change
//! goes out only when all three allow it. That is the whole design: a safety
//! catch that something else can quietly arm is not a safety catch, and
//! somebody who marked one account read-only should not have that undone by a
//! setting they changed for a different reason.
//!
//! It follows that the command line can only ever restrict. `--read-only`
//! stops everything for that run whatever is stored; there is deliberately no
//! flag that forces writes on, because the flag that turns off a safety catch
//! is the flag somebody leaves in a shortcut and forgets.
//!
//! # Split by what it costs to get wrong
//!
//! Mail is separate from everything else. Losing a task is annoying and can be
//! typed again; a message deleted from a server, or sent to the wrong people,
//! is gone. So they are two answers rather than one.
//!
//! Reading is separate from both, and split off for that same reason read the
//! other way round: it costs nothing to get wrong, so it is the one field
//! whose safe end is on. Grouping it with the two above would have made
//! `Default` mean one thing for two fields and its opposite for the third
//! under a single sentence, which is how the wrong half comes to be copied.

use serde::{Deserialize, Serialize};

/// What may be done at a provider: two changes and one read.
///
/// Three answers rather than one boolean, because they cost different amounts
/// to get wrong. `Default` is the safe end of each of them, and the safe end
/// is not the same direction for all three: for a change it is off, and for a
/// read it is on. Nothing constructed without a decision changes anything, and
/// nothing constructed without a decision stops mail being read either.
///
/// The read answer is last because it arrived last, and because the two above
/// it are the ones that cannot be undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allowed {
    /// Sending a message, and changing or deleting one on the server.
    ///
    /// The one that cannot be undone. A message that has gone has gone, and a
    /// message deleted from a server may be the only copy.
    pub mail: bool,
    /// Tasks, contacts and calendar events at a provider.
    ///
    /// Recoverable by hand, mostly, and this is the least proven code in the
    /// application: none of the three sync paths has met a live account.
    pub personal_information: bool,
    /// Fetching a message's text back from a provider.
    ///
    /// **On by default, which is the exception to the rule the two above
    /// follow.** That rule holds for a change because off is unambiguously
    /// safer: nothing happens, and nothing irreversible can. A read inverts
    /// it. Nothing a body fetch does is irreversible, and off makes every
    /// search silently cover a fraction of the mailbox until somebody finds
    /// the setting, which is the failure this was added to prevent.
    ///
    /// So `Default` stops meaning "changes nothing" and starts meaning "the
    /// safe end of each". That is why `Default` is written out below rather
    /// than derived: deriving it gives `false` for a bool, which is this field
    /// inverted, and the compiler would never say so.
    ///
    /// The default is named rather than restated, the way `data::config`'s
    /// `default_allowed` is and for the reason its doc gives. A bare
    /// `#[serde(default)]` here would answer `false` for every settings file
    /// written before this existed, which is every settings file on every
    /// machine.
    #[serde(default = "default_reading")]
    pub reading: bool,
}

/// What an absent key, and a value built without a decision, answer for
/// reading.
///
/// A named function rather than `#[serde(default)]`, which answers `false`,
/// and rather than a literal repeated in two places. Both obvious ways to
/// write this are wrong in the same direction, and neither would fail to
/// compile.
const fn default_reading() -> bool {
    true
}

impl Default for Allowed {
    /// Written by hand, because deriving it is wrong for one of the three.
    ///
    /// The derive gives `false` for every bool. That is right for the two
    /// changes and inverted for the read, and it is the kind of wrong that
    /// compiles, passes, and only shows up as somebody's search quietly
    /// covering part of their mailbox.
    ///
    /// `NOTHING` rather than a fourth literal, so there is one place the safe
    /// answer is written down.
    fn default() -> Self {
        Self::NOTHING
    }
}

impl Allowed {
    /// Nothing may be changed.
    ///
    /// The name is still right and the shape no longer matches it literally,
    /// so the reason is here rather than left for the next reader to work out.
    /// **Reading stays on.** Two things resolve to this constant and both of
    /// them promise reading:
    ///
    /// - `presentation::first_run`'s first choice, labelled "Read my mail,
    ///   change nothing", which is the option somebody picks because it
    ///   sounded safe.
    /// - `--read-only`, whose help text says "Change nothing at any server
    ///   this run".
    ///
    /// Neither says read nothing. A reading field set to `false` here would
    /// take mail away from exactly the most cautious person in the user base,
    /// and it would do it silently, because nothing they could see would have
    /// changed.
    pub const NOTHING: Self = Self {
        mail: false,
        personal_information: false,
        reading: true,
    };

    /// Everything may be changed, which is what a finished mail client does.
    pub const EVERYTHING: Self = Self {
        mail: true,
        personal_information: true,
        reading: true,
    };

    /// What an alpha tester starts with.
    ///
    /// Tasks, contacts and the calendar can be changed; mail cannot. Somebody
    /// can point this at their real account, use it all day, and the worst
    /// that happens is a task in the wrong place. Sending is the deliberate
    /// step afterwards.
    pub const FOR_TESTING: Self = Self {
        mail: false,
        personal_information: true,
        reading: true,
    };

    /// What both of these allow, which is what actually happens.
    ///
    /// The safest answer wins on each half independently, so a person who
    /// allows everything globally and marks one account read-only gets what
    /// they asked for on both.
    pub const fn and(self, other: Self) -> Self {
        Self {
            mail: self.mail && other.mail,
            personal_information: self.personal_information && other.personal_information,
            reading: self.reading && other.reading,
        }
    }

    /// Whether anything at all may be changed.
    pub const fn anything(self) -> bool {
        self.mail || self.personal_information
    }
}

/// What the settings screen calls the section holding the two boxes below.
///
/// One string rather than one per screen. A sync tells somebody to turn this
/// on by name, so the name it says has to be the name they then read on the
/// screen. It was not: the sentence said "Allow Changes" and the section was
/// headed "Allowed Changes", which is near enough to look like the right place
/// and far enough to make somebody stop and check.
pub const SETTINGS_SECTION: &str = "Allow Changes";

/// What the settings screen calls the section holding the reading box.
///
/// A second heading rather than a third box under `SETTINGS_SECTION`, because
/// a read is not a change. Putting it under a heading that says Allow Changes
/// would be the same drift that constant's own doc records: a sentence sending
/// somebody to a heading that does not describe what they are looking for.
///
/// Read by the settings screen and by `service::outward`'s read refusal, so
/// the sentence somebody hears when a fetch is refused names the heading they
/// will actually find.
///
/// Not "Reading Mail", which was the first answer and is the wrong one. The
/// settings screen already has a tab headed "Reading", about how messages are
/// shown. A refusal saying "turn on Reading Mail in Settings" would send
/// somebody to that tab, where there is nothing of the kind, and this control
/// is on the Permissions tab. That is the same failure `SETTINGS_SECTION`
/// below records, one step further out: there the sentence and the heading
/// differed by a word, here they would have differed by a page.
pub const READING_SECTION: &str = "Message Text";

/// The label on the box under [`READING_SECTION`].
///
/// Kept here rather than typed into the settings screen, for the reason
/// [`EXPERIMENTAL_WARNING`] below gives: the labels that were typed there
/// drifted from the accessible names beside them, and both had lost their line
/// continuations. This is one string, so the box carries its own name on both
/// channels and there is nothing to drift from.
///
/// Says what it permits and what it costs, in that order. The ampersand is the
/// keyboard accelerator, on "F" for fetch, and it is on the label rather than
/// set separately because a wxWidgets checkbox with a label of its own already
/// reports that label to the accessibility tree.
///
/// It names the server, because the whole difference between this being on and
/// off is whether anything leaves the machine to get the text of a message
/// that is not already stored.
pub const MESSAGE_TEXT_LABEL: &str =
    "&Fetch the text of a message from the server when it is not already stored";

/// What the box under [`READING_SECTION`] says beneath itself.
///
/// Says which way round it starts, because a box somebody finds ticked tells
/// them nothing about whether that is the answer they were given or one they
/// chose, and says what turning it off costs, because that is not obvious:
/// mail already stored goes on being readable, and a search stops covering
/// what is not.
pub const MESSAGE_TEXT_NOTE: &str = "This is on unless you turn it off. Mail already stored on \
     this machine stays readable either way. With it off, nothing fetches the text of a message \
     that is not stored yet, so opening one of those and searching for words inside it will not \
     find them.";

/// The sentence a sync says when that setting is holding changes here.
///
/// One answer rather than one per module. The contacts sync and the calendar
/// sync say the same sentence about the same setting, they were written out
/// separately, and only one of them was ever corrected: the calendar read out
/// "1 changes are waiting here ... to send them".
///
/// It names Settings and not the account. There is one answer for the whole
/// application and the settings screen is the only thing that writes it:
/// `AppConfig::allowed_per_account` is read and honoured, and nothing outside
/// its own tests has ever written one. Saying "for this account" sent somebody
/// looking for a control that is not there, and hid the part that matters,
/// which is that turning it on turns it on for every account they have.
///
/// Both numbers are written out whole rather than built from a stem and an
/// "s". Three words have to agree, and a sentence assembled from fragments
/// reads like one.
pub fn changes_waiting_here(count: usize) -> String {
    match count {
        1 => format!(
            "1 change is waiting here: turn on {SETTINGS_SECTION} in Settings \
             to send it"
        ),
        many => format!(
            "{many} changes are waiting here: turn on {SETTINGS_SECTION} in \
             Settings to send them"
        ),
    }
}

/// The sentence a POP check says when that setting is holding a clear-out back.
///
/// Beside the one above rather than sharing it. That one ends "to send them",
/// and nothing here is being sent: mail somebody asked to have cleared off
/// their provider is still sitting there. Telling them a message is waiting to
/// go out when it is really waiting to be taken away is worse than saying
/// nothing, because they would go looking in the wrong place.
///
/// Both numbers written out whole, for the same reason as the one above: three
/// words have to agree, and a sentence assembled from a stem and an "s" reads
/// like one.
pub fn removals_waiting_here(count: usize) -> String {
    match count {
        1 => format!(
            "1 message is still on the server: turn on {SETTINGS_SECTION} in \
             Settings to remove it"
        ),
        many => format!(
            "{many} messages are still on the server: turn on {SETTINGS_SECTION} \
             in Settings to remove them"
        ),
    }
}

/// The warning shown beside the two Allow Changes boxes.
///
/// Kept here rather than typed into the settings screen, because it was typed
/// there and the copy drifted: the visible label and the accessible name were
/// two hand-written strings that differed from each other, and both had lost
/// their line continuations, so each carried a run of stray spaces in the
/// middle of a sentence somebody hears.
///
/// One sentence per idea, and the irreversible parts last, because that is the
/// part somebody has to still be listening for.
///
/// The guest list is in that group and is the only part of it on the personal
/// information half: everything else that half allows changes somebody's own
/// things, and this one can reach their colleagues. "May", not "will",
/// because which of the two it is has never been seen: whether a provider
/// emails a guest added this way is the provider's decision, nothing here asks
/// either of them to stay quiet, and no meeting made here has ever reached
/// one. Telling somebody to expect the mail and try it on themselves first is
/// the honest instruction under that uncertainty.
pub const EXPERIMENTAL_WARNING: &str = "Both are experimental: none of this has been run against a real account yet, \
     so expect bugs. Reading your mail is the part that has been used. A meeting \
     you make here takes its guest list to Google or Outlook, which may email the \
     guests to invite them, so try it with an address of your own first. A \
     message that has been sent cannot be recalled, and a message deleted from a \
     server may have been the only copy.";

/// The warning shown beside the offer to fetch missing message text in bulk.
///
/// Its own sentence, beside [`EXPERIMENTAL_WARNING`] rather than inside it,
/// for two reasons. That one is about writes, and opens by saying both of the
/// things it covers are experimental, so a read added to it would be counted
/// among things that cannot be undone. And six assertions hold its wording
/// word for word, which is the right way round: a warning about irreversible
/// changes should be hard to edit by accident.
///
/// What it has to carry is the one risk no test in this repository can settle.
/// Every other experimental thing here is experimental because it has never
/// run; this is experimental because of what a provider may do when it does.
/// Asking for hundreds of whole messages in a row is a shape a mail server is
/// entitled to refuse, throttle or disconnect, and nothing on this side can
/// find out which without a real account.
///
/// Second person and plain language, in the register of the warning above it.
/// It says what could go wrong and what it costs, rather than only that the
/// feature is new: "experimental" on its own tells somebody to be careful and
/// not what to be careful of.
pub const FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL: &str = "";

/// Everything that has an opinion about what may be changed.
///
/// Kept as one value so the answer is worked out in one place and every
/// transport asks the same question. Threading three booleans through instead
/// is how one of them comes to be forgotten at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission {
    /// What the command line said, which can only ever narrow the others.
    pub command_line: Allowed,
    /// What this installation is set to.
    pub setting: Allowed,
    /// What this account is set to.
    pub account: Allowed,
}

impl Permission {
    /// The answer: what all three allow.
    pub const fn allowed(self) -> Allowed {
        self.command_line.and(self.setting).and(self.account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing_is_allowed_until_something_says_otherwise() {
        // The important default. A value built without anybody deciding should
        // be the one that cannot damage somebody's mail.
        assert_eq!(Allowed::default(), Allowed::NOTHING);
        assert!(!Allowed::default().anything());
    }

    #[test]
    fn test_the_choice_that_promises_reading_still_reads() {
        // D-2-11, and the reason this constant stops meaning "every field
        // false". `presentation::first_run` maps its "Read my mail, change
        // nothing" choice to this, and `--read-only` resolves to it too. A
        // reading field that defaulted to false inside here would take reading
        // away from the one option whose label promises it, and it would do
        // that to the most cautious person in the user base, who chose it
        // because it sounded safe.
        // All three fields at once rather than a reading assertion on its own,
        // so the name stays honest in both directions: this must go on
        // permitting reading, and it must go on refusing both writes.
        assert_eq!(
            Allowed::NOTHING,
            Allowed {
                mail: false,
                personal_information: false,
                reading: true,
            },
            "the choice labelled 'Read my mail, change nothing' no longer means that"
        );
    }

    #[test]
    fn test_reading_is_on_wherever_anything_is() {
        // The other two constants. Neither is a cautious answer about reading:
        // one is what a finished mail client does and the other is what an
        // alpha tester starts with, and both of those people expect their mail
        // to arrive.
        //
        // Read out of a binding rather than asserted on the constant directly,
        // because an assertion whose value the compiler already knows is one
        // clippy will not let through, and rightly: it cannot fail at runtime.
        for (named, allowed) in [
            ("EVERYTHING", Allowed::EVERYTHING),
            ("FOR_TESTING", Allowed::FOR_TESTING),
        ] {
            assert!(allowed.reading, "{named} does not permit reading mail");
        }
    }

    #[test]
    fn test_reading_is_narrowed_the_same_independent_way_as_the_two_writes() {
        // Reading survives unless some place says no, which is the same rule
        // the two writes follow. Asserted rather than assumed because `and` is
        // where a third field is forgotten.
        let says_no = Allowed {
            mail: true,
            personal_information: true,
            reading: false,
        };

        assert!(!Allowed::EVERYTHING.and(says_no).reading);
        assert!(!says_no.and(Allowed::EVERYTHING).reading);
        assert!(
            Allowed::EVERYTHING.and(Allowed::NOTHING).reading,
            "nothing said no to reading, so it must survive both writes being refused"
        );
    }

    #[test]
    fn test_reading_is_not_a_change() {
        // `anything()` answers whether anything may be *changed*, and reading
        // changes nothing. Widening it would tell every caller that a session
        // allowed only to read is a session that writes.
        assert!(!Allowed::NOTHING.anything());
        assert!(
            !Allowed {
                mail: false,
                personal_information: false,
                reading: true,
            }
            .anything()
        );
    }

    #[test]
    fn test_the_read_section_is_not_the_one_headed_allow_changes() {
        // A read is not a change, and putting the control under a heading that
        // says Allow Changes is the label-versus-sentence drift
        // SETTINGS_SECTION's own doc records happening again. Two constants,
        // and this asserts they are two.
        assert_ne!(READING_SECTION, SETTINGS_SECTION);
        assert!(
            !READING_SECTION.contains("Change"),
            "{READING_SECTION} reads as a heading about changing things"
        );
    }

    #[test]
    fn test_either_half_on_its_own_counts_as_something_being_allowed() {
        // Half allowed is not nothing allowed. The default answering no is the
        // only thing recorded about this question so far, and "no to
        // everything" and "no unless both halves are on" agree on the default
        // and disagree everywhere else.
        assert!(
            Allowed {
                mail: true,
                personal_information: false,
                reading: true,
            }
            .anything()
        );
        assert!(
            Allowed {
                mail: false,
                personal_information: true,
                reading: true,
            }
            .anything()
        );
        assert!(Allowed::EVERYTHING.anything());
        assert!(!Allowed::NOTHING.anything());
    }

    #[test]
    fn test_an_alpha_tester_can_change_their_tasks_and_not_their_mail() {
        // Somebody can point this at their real account, use it all day, and
        // the worst that happens is a task in the wrong place.
        assert_eq!(
            Allowed::FOR_TESTING,
            Allowed {
                mail: false,
                personal_information: true,
                reading: true,
            }
        );
    }

    #[test]
    fn test_one_no_is_enough() {
        // The rule the whole thing rests on. Somebody who marked an account
        // read-only must not have that undone by a setting they changed for a
        // different account, and the command line must be able to stop
        // everything without editing any stored setting at all.
        let stopped_at_the_command_line = Permission {
            command_line: Allowed::NOTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::EVERYTHING,
        };
        let stopped_by_the_setting = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::NOTHING,
            account: Allowed::EVERYTHING,
        };
        let stopped_by_the_account = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::NOTHING,
        };

        for stopped in [
            stopped_at_the_command_line,
            stopped_by_the_setting,
            stopped_by_the_account,
        ] {
            assert_eq!(stopped.allowed(), Allowed::NOTHING, "{stopped:?}");
        }
    }

    #[test]
    fn test_a_waiting_change_names_the_one_place_that_can_send_it() {
        // Allow Changes is one answer for the whole application: the settings
        // screen writes that one, and nothing writes an answer for a single
        // account. The sentence said to turn it on "for this account", which
        // sent somebody looking for a control that is not there and hid the
        // part that matters, which is that turning it on turns it on for every
        // account they have.
        assert_eq!(
            changes_waiting_here(1),
            "1 change is waiting here: turn on Allow Changes in Settings to send it"
        );
        assert_eq!(
            changes_waiting_here(3),
            "3 changes are waiting here: turn on Allow Changes in Settings to \
             send them"
        );
    }

    #[test]
    fn test_a_removal_held_back_says_the_mail_is_still_on_the_server() {
        // A sentence of its own rather than the one above. Nothing is being
        // sent here: mail somebody asked to have cleared off the server is
        // still there, and "turn this on to send them" would tell them the
        // wrong thing about their own mailbox.
        assert_eq!(
            removals_waiting_here(1),
            "1 message is still on the server: turn on Allow Changes in Settings to remove it"
        );
        assert_eq!(
            removals_waiting_here(4),
            "4 messages are still on the server: turn on Allow Changes in Settings \
             to remove them"
        );
    }

    #[test]
    fn test_the_two_halves_are_decided_separately() {
        // Somebody who allows everything but marks one account mail-read-only
        // should still be able to sync that account's tasks. One boolean would
        // have taken both away.
        let permission = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::FOR_TESTING,
        };

        assert_eq!(permission.allowed(), Allowed::FOR_TESTING);
    }

    #[test]
    fn test_everything_allowed_everywhere_allows_everything() {
        let permission = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::EVERYTHING,
        };

        assert_eq!(permission.allowed(), Allowed::EVERYTHING);
        assert!(permission.allowed().mail);
    }

    #[test]
    fn test_anything_that_writes_says_it_is_experimental() {
        // The warning has to reach the person using it, not sit in a note.
        // None of these paths has run against a real account, and this is the
        // sentence beside the two boxes that turn them on.
        assert!(EXPERIMENTAL_WARNING.contains("experimental"));
        assert!(EXPERIMENTAL_WARNING.contains("real account"));
    }

    #[test]
    fn test_the_warning_says_that_making_a_meeting_can_email_the_guests() {
        // The personal information half used to reach only somebody's own
        // things. A meeting made here now carries its guest list to Google or
        // Outlook, and adding a guest is what makes a provider email them, so
        // this is the one thing the box turns on that reaches other people and
        // cannot be taken back. A warning that only exists in a changelog is a
        // warning nobody gets, and this is the sentence beside the box.
        assert!(
            EXPERIMENTAL_WARNING.contains("email"),
            "{EXPERIMENTAL_WARNING}"
        );
        assert!(
            EXPERIMENTAL_WARNING.contains("guest"),
            "{EXPERIMENTAL_WARNING}"
        );
    }

    #[test]
    fn test_fetching_text_in_bulk_says_it_is_experimental_and_says_what_could_go_wrong() {
        // Its own sentence, and it has to earn being a second one. Everything
        // else here is experimental because it has never run; this is
        // experimental because of what a provider may do when it does, and
        // that is the risk no test in this repository can settle. A warning
        // saying only "experimental" tells somebody to be careful and not what
        // to be careful of.
        assert!(
            FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL.contains("experimental"),
            "{FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL}"
        );
        assert!(
            FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL.contains("real account"),
            "{FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL}"
        );
        assert!(
            FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL.contains("provider"),
            "it does not say whose decision the thing that could go wrong is: \
             {FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL}"
        );
        assert!(
            !FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL.contains("  "),
            "a wrapped literal lost its continuations, so this is read aloud \
             with stray silences: {FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL}"
        );
    }

    #[test]
    fn test_the_bulk_fetch_warning_is_beside_the_write_warning_and_not_inside_it() {
        // Two reasons, and both matter. The write warning opens by saying both
        // of the things it covers are experimental, so a read folded into it
        // would be counted among things that cannot be undone. And six
        // assertions hold that warning word for word, which is the right way
        // round for a warning about irreversible changes.
        assert!(
            !EXPERIMENTAL_WARNING.contains("fetch"),
            "the read was folded into the warning about writes: \
             {EXPERIMENTAL_WARNING}"
        );
        assert_ne!(
            FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL, EXPERIMENTAL_WARNING,
            "one sentence is doing both jobs"
        );
    }

    #[test]
    fn test_the_experimental_warning_reads_as_sentences_rather_than_a_wrapped_literal() {
        // A wrapped string literal that loses its continuations keeps every
        // space of the indenting, and this one is read aloud. Runs of stray
        // spaces are silence in the middle of a sentence somebody is relying
        // on to tell them what cannot be undone.
        assert!(
            !EXPERIMENTAL_WARNING.contains("  "),
            "{EXPERIMENTAL_WARNING}"
        );
        assert!(EXPERIMENTAL_WARNING.contains("cannot be recalled"));
        assert!(EXPERIMENTAL_WARNING.contains("only copy"));
    }

    #[test]
    fn test_the_message_text_box_can_be_reached_by_keyboard_and_says_what_it_costs() {
        // The label is what somebody hears and what they press a letter to
        // reach, and the note is the only thing that says which way round the
        // box starts. A tick with no sentence under it cannot say that.
        assert!(
            MESSAGE_TEXT_LABEL.contains('&'),
            "the box has no mnemonic, so it cannot be reached by keyboard from the tab: \
             {MESSAGE_TEXT_LABEL}"
        );
        assert!(
            MESSAGE_TEXT_LABEL.contains("server"),
            "{MESSAGE_TEXT_LABEL}"
        );

        assert!(
            MESSAGE_TEXT_NOTE.contains("on unless you turn it off"),
            "the note does not say which way round it starts: {MESSAGE_TEXT_NOTE}"
        );
        assert!(
            MESSAGE_TEXT_NOTE.contains("stays readable"),
            "the note does not say what turning it off leaves alone, which is the part \
             somebody weighing it needs: {MESSAGE_TEXT_NOTE}"
        );
    }

    #[test]
    fn test_the_message_text_sentences_read_as_sentences_rather_than_wrapped_literals() {
        // The same failure `EXPERIMENTAL_WARNING` carries a test for, and the
        // reason it does: a wrapped literal that loses its continuations keeps
        // every space of the indenting, and these are read aloud. Runs of
        // stray spaces are silence in the middle of a sentence.
        for said in [MESSAGE_TEXT_LABEL, MESSAGE_TEXT_NOTE] {
            assert!(!said.contains("  "), "{said}");
        }
    }

    #[test]
    fn test_a_refused_read_names_the_heading_the_box_is_actually_under() {
        // The sentence and the heading are one string apart, which is the
        // whole reason `READING_SECTION` exists rather than a literal on the
        // screen. Somebody told to turn something on has to be told the words
        // they will read when they get there.
        let said = crate::service::outward::read_refusal("read the text of a message");

        assert!(said.contains(READING_SECTION), "{said}");
        assert!(
            !said.contains(SETTINGS_SECTION),
            "a refused read sends somebody to the heading about changing things: {said}"
        );
    }

    #[test]
    fn test_combining_is_order_independent() {
        // It reads as though order might matter, and it must not: the three
        // are asked in whatever order a call site happens to write them.
        let a = Allowed::FOR_TESTING;
        let b = Allowed {
            mail: true,
            personal_information: false,
            reading: true,
        };

        assert_eq!(a.and(b), b.and(a));
        assert_eq!(a.and(b), Allowed::NOTHING);
    }
}

/// What the command line narrowed this run to.
///
/// Written once, before anything opens, and read from wherever a client is
/// built. A global is worth justifying: this one is set exactly once at
/// startup, is never changed afterwards, and can only ever take permissions
/// away. Threading it instead would mean carrying it through seven
/// constructions in the window layer, several inside spawned tasks, which is
/// seven chances to drop it, and dropping it fails towards writing.
static FROM_COMMAND_LINE: std::sync::OnceLock<Allowed> = std::sync::OnceLock::new();

/// Record what the command line allowed. Call once, before anything opens.
///
/// A second call is ignored rather than being an error: the first answer is
/// the one from the arguments, and nothing later should be able to widen it.
pub fn narrow_this_run_to(allowed: Allowed) {
    let _ = FROM_COMMAND_LINE.set(allowed);
}

/// What this account may actually change, with all three asked.
///
/// The one function every client should call before it is built. Returns the
/// narrowest of the command line, the application-wide setting and the
/// account's own, so no caller has to remember there are three.
///
/// A settings file that cannot be read counts as allowing nothing. That is the
/// safe direction: somebody whose config is corrupt should find that nothing
/// syncs, not that everything is permitted.
pub fn allowed_for(account_id: &str) -> Allowed {
    let stored = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().allowed_for(account_id))
        .unwrap_or(Allowed::NOTHING);

    narrowed_by(FROM_COMMAND_LINE.get().copied(), stored)
}

/// What the command line leaves of a stored answer.
///
/// Split out so it can be tested without touching the process-wide value.
/// That value is set once, so a test for the unset case and a test that sets
/// it cannot both live in the same process: Rust runs them in parallel and
/// whichever went first decided the answer for the other. Not hypothetical,
/// it is what happened, and it failed about one run in three.
///
/// `None` means the command line had no opinion, which is different from it
/// having said "nothing": a window opened by a test or a tool that never
/// parsed arguments is governed by the settings alone.
const fn narrowed_by(command_line: Option<Allowed>, stored: Allowed) -> Allowed {
    match command_line {
        Some(narrowing) => narrowing.and(stored),
        None => stored,
    }
}

#[cfg(test)]
mod resolving {
    use super::*;

    #[test]
    fn test_no_opinion_from_the_command_line_leaves_the_settings_alone() {
        // A window can be opened by a test or a tool that never parsed
        // arguments. That must not silently mean "allow everything" nor
        // "allow nothing": it means the command line said nothing, and the
        // settings decide on their own.
        assert_eq!(narrowed_by(None, Allowed::EVERYTHING), Allowed::EVERYTHING);
        assert_eq!(
            narrowed_by(None, Allowed::FOR_TESTING),
            Allowed::FOR_TESTING
        );
        assert_eq!(narrowed_by(None, Allowed::NOTHING), Allowed::NOTHING);
    }

    #[test]
    fn test_the_command_line_can_only_take_permissions_away() {
        assert_eq!(
            narrowed_by(Some(Allowed::NOTHING), Allowed::EVERYTHING),
            Allowed::NOTHING
        );
        assert_eq!(
            narrowed_by(Some(Allowed::EVERYTHING), Allowed::FOR_TESTING),
            Allowed::FOR_TESTING,
            "saying everything on the command line must not widen a setting"
        );
    }

    #[test]
    fn test_recording_it_twice_keeps_the_first_answer() {
        // Set once at startup. Anything later trying to widen it is ignored,
        // which is the property that makes a global defensible here.
        narrow_this_run_to(Allowed::NOTHING);
        narrow_this_run_to(Allowed::EVERYTHING);

        assert_eq!(FROM_COMMAND_LINE.get().copied(), Some(Allowed::NOTHING));
    }
}

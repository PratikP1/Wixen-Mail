//! What Windows handed this program on the way in, and what to do with it.
//!
//! When Wixen Mail holds one of the associations it registers, Windows starts
//! it with a single argument: the `mailto:` link somebody followed, or the
//! path of the `.ics` or `.vcf` file they opened. Before this existed the
//! command line refused every argument it did not recognise as a flag, so a
//! registered handler would have started, shown an error, and exited with a
//! failure code. Registering without this would have been a claim to open
//! things that this program then threw away.
//!
//! # Everything here arrives from a stranger
//!
//! A `mailto:` link comes from a web page. An `.ics` file comes as a meeting
//! invitation in somebody's mail. A `.vcf` file comes from a contact somebody
//! was sent. None of it is typed by the person, so none of it is trusted:
//! [`what_was_handed_over`] decides what a thing is from its own shape and
//! refuses anything it does not recognise, and [`text_of`] puts a ceiling on
//! how much of a file is read.

use crate::application::mailto::MailTo;
use crate::common::{Error, Result};
use crate::data::message_cache::MessageCache;
use std::path::{Path, PathBuf};

/// One thing Windows asked this program to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opening {
    /// Somebody followed a `mailto:` link. Open a composer, filled in.
    Compose(Box<MailTo>),
    /// Somebody opened an `.ics` file. Read the events in it.
    CalendarFile(PathBuf),
    /// Somebody followed a `webcal:` link, which names a calendar to follow
    /// rather than a file to read. Carried as the ordinary web address it
    /// really is.
    CalendarSubscription(String),
    /// Somebody opened a `.vcf` file. Read the contacts in it.
    ContactFile(PathBuf),
}

/// Work out what one argument is, from its own shape.
///
/// Every arm matches the way Windows matches: schemes and extensions are
/// compared without case, because a link may be written `MAILTO:` and a file
/// saved as `INVITE.ICS`, and a comparison that missed those would refuse
/// things this program is registered to open.
///
/// Anything else is refused rather than guessed at. A guess here opens the
/// wrong window on somebody's file, and the whole point of registering is that
/// Windows only ever hands over one of these four.
pub fn what_was_handed_over(argument: &str) -> Result<Opening> {
    let trimmed = argument.trim();

    if starts_with_scheme(trimmed, "mailto:") {
        return crate::application::mailto::parse(trimmed)
            .map(|asked| Opening::Compose(Box::new(asked)));
    }
    // Both spellings, and both become an ordinary https address. `webcal:` is
    // not a transport, it is "subscribe to this rather than download it", and
    // the thing behind it is served over the web. https rather than http
    // because a calendar can carry somebody's whole week and a sign-in with
    // it, and a feed that only answers on http shows a plain fetch error
    // somebody can read rather than sending it in the clear.
    for scheme in ["webcals:", "webcal:"] {
        if starts_with_scheme(trimmed, scheme) {
            let address = trimmed
                .get(scheme.len()..)
                .unwrap_or_default()
                .trim_start_matches('/');
            if address.is_empty() {
                return Err(Error::Other(format!(
                    "{trimmed:?} names no calendar to follow"
                )));
            }
            return Ok(Opening::CalendarSubscription(format!("https://{address}")));
        }
    }

    if ends_with_extension(trimmed, ".ics") {
        return Ok(Opening::CalendarFile(PathBuf::from(trimmed)));
    }
    if ends_with_extension(trimmed, ".vcf") {
        return Ok(Opening::ContactFile(PathBuf::from(trimmed)));
    }

    Err(Error::Other(format!(
        "Wixen Mail does not know what to do with {trimmed:?}. It opens mailto: \
         and webcal: links, and .ics and .vcf files"
    )))
}

/// Whether some text begins with this scheme, however it is capitalised.
fn starts_with_scheme(text: &str, scheme: &str) -> bool {
    text.get(..scheme.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(scheme))
}

/// Whether a path ends in this extension, however it is capitalised.
///
/// The extension has to be preceded by something, so a file named exactly
/// `.ics` is not treated as a calendar with no name.
fn ends_with_extension(path: &str, extension: &str) -> bool {
    path.len() > extension.len()
        && path
            .get(path.len() - extension.len()..)
            .is_some_and(|end| end.eq_ignore_ascii_case(extension))
}

/// As much of a handed-over file as this program is willing to read.
///
/// A calendar or a contact card is small. Something claiming to be one and
/// running to hundreds of megabytes is either broken or aimed at this
/// program, and reading it into memory on the way to opening a window is how
/// a double click becomes a machine that stops answering. Two megabytes holds
/// a year of meetings or several thousand contact cards.
const MOST_A_HANDED_OVER_FILE_MAY_HOLD: u64 = 2 * 1024 * 1024;

/// Read a file Windows handed over, as text.
///
/// Refuses one that is too big before reading any of it, so the ceiling costs
/// nothing on an ordinary file and is not reached by filling memory first.
///
/// Not valid UTF-8 is replaced rather than refused. Calendar and contact
/// files are written by every program there is, some of them still emitting
/// Latin-1, and refusing the file outright would lose every event in it over
/// one accented name.
pub fn text_of(path: &Path) -> Result<String> {
    let how_big = std::fs::metadata(path)
        .map_err(|why| {
            Error::Other(format!(
                "Wixen Mail could not read {}: {why}",
                path.display()
            ))
        })?
        .len();
    if how_big > MOST_A_HANDED_OVER_FILE_MAY_HOLD {
        return Err(Error::Other(format!(
            "{} is {how_big} bytes, which is larger than Wixen Mail will open \
             as a calendar or a contact card",
            path.display()
        )));
    }

    let raw = std::fs::read(path).map_err(|why| {
        Error::Other(format!(
            "Wixen Mail could not read {}: {why}",
            path.display()
        ))
    })?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

/// The account an imported calendar or contact card is filed under.
///
/// The local one, always, and this is a decision rather than a fallback. An
/// event read out of a file somebody was sent has not come from any of their
/// accounts, and filing it under one would mean the next sync offering it to
/// that provider as something they had created.
pub const WHERE_IMPORTED_THINGS_GO: &str = crate::application::new_item::LOCAL_ACCOUNT_ID;

/// The calendar imported events are filed in.
///
/// A fixed name rather than nothing, so that opening the same invitation
/// twice updates one event instead of leaving two. The stored events are keyed
/// on the account, the calendar and the identity the file gave them, and two
/// events with no calendar at all count as different rows however alike they
/// are.
pub const IMPORTED_CALENDAR: &str = "imported-from-a-file";

/// What reading a calendar file did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventsRead {
    /// Events that were not already here.
    pub added: usize,
    /// Events already here under the same identity, now brought up to date.
    pub brought_up_to_date: usize,
    /// Who is asking, when the file was an invitation and not a plain meeting.
    ///
    /// A file carrying `METHOD:REQUEST` is somebody waiting for an answer, and
    /// this used to be filed in silence: the meeting went into the calendar
    /// and the sentence said one event was added, which was true and was not
    /// the part that mattered.
    pub an_invitation_from: Option<String>,
}

impl EventsRead {
    /// How many events the file turned out to hold.
    pub fn altogether(&self) -> usize {
        self.added + self.brought_up_to_date
    }
}

/// Read the events in a calendar file into the events kept on this computer.
///
/// The parsing is [`crate::service::ical_subscription`]'s, which is the same
/// reader a subscribed calendar goes through, so a file and a feed cannot
/// disagree about what an event is.
///
/// Every event is stripped of the things that say it came from a server: no
/// provider, no address, no version tag, and nothing waiting to be sent.
/// Left on, the next sync would offer somebody's meeting invitation to their
/// own calendar server as an event they had created.
pub fn read_calendar_file(cache: &MessageCache, text: &str) -> Result<EventsRead> {
    let found = crate::service::ical_subscription::parse_ics(text)?;

    // Read once rather than per event: a file with two hundred meetings in it
    // would otherwise be two hundred queries on the way to opening a window.
    let already = cache
        .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
        .unwrap_or_default();

    let mut read = EventsRead::default();
    for event in &found {
        let mut entry = crate::application::caldav_sync::caldav_event_to_local(
            event,
            WHERE_IMPORTED_THINGS_GO,
            IMPORTED_CALENDAR,
        );
        entry.source_provider = None;
        entry.etag = None;
        entry.web_link = None;
        entry.last_synced_at = None;
        entry.pending = false;

        match already.iter().find(|stored| {
            stored.provider_event_id.as_deref() == entry.provider_event_id.as_deref()
        }) {
            // Kept, so the save updates that row rather than making a second
            // one beside it. Opening the same invitation twice is ordinary.
            Some(stored) => {
                entry.id = stored.id.clone();
                entry.created_at = stored.created_at.clone();
                read.brought_up_to_date += 1;
            }
            None => read.added += 1,
        }
        cache.save_calendar_event(&entry)?;
    }
    read.an_invitation_from = who_is_asking(text);
    Ok(read)
}

/// Who is waiting for an answer, when the document is somebody asking.
///
/// Nothing for every other kind of calendar document, which is almost all of
/// them: a meeting exported from another program, a subscribed feed, or a
/// meeting somebody sent as a record rather than a question. Calling one of
/// those an invitation would tell somebody to answer a person who never asked.
///
/// The name when the organiser gave one, and the address when they did not,
/// because a sentence that says "an invitation from" and then stops is worse
/// than one carrying an address somebody has to read.
fn who_is_asking(text: &str) -> Option<String> {
    use crate::application::invitations::{WhatItAsks, read_the_invitation, what_it_asks};

    if what_it_asks(text) != WhatItAsks::Invitation {
        return None;
    }
    let asked_by = read_the_invitation(text).ok()?.organiser?;
    Some(match asked_by.name.filter(|name| !name.trim().is_empty()) {
        Some(name) => name,
        None => asked_by.address,
    })
}

/// What to say once a calendar file has been read.
///
/// Said in full sentences rather than as counts, because it is read out. A
/// screen reader announcing "2 1 0" is a person having to work out which
/// number was which.
pub fn what_reading_the_calendar_did(read: EventsRead, name: &str) -> String {
    let what_happened = match (read.added, read.brought_up_to_date) {
        (0, 0) => format!("{name} held no events, so nothing was added."),
        (added, 0) => format!("{added} {} added from {name}.", events(added)),
        (0, updated) => format!(
            "{updated} {} in {name} were already here, and have been brought up to date.",
            events(updated)
        ),
        (added, updated) => format!(
            "{added} {} added from {name}, and {updated} already here brought up to date.",
            events(added)
        ),
    };
    // Said plainly, including the part that does not work. Answering from
    // here is not built, and a sentence that named the invitation without
    // saying so would read as though a button were somewhere to be found.
    match read.an_invitation_from {
        Some(asked_by) => format!(
            "{what_happened} This is an invitation from {asked_by}, who is waiting for an \
             answer. Wixen Mail cannot answer it yet, so reply to them by email."
        ),
        None => what_happened,
    }
}

/// "event" or "events", so a sentence reads properly when there is one.
fn events(how_many: usize) -> &'static str {
    if how_many == 1 { "event" } else { "events" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_mailto_link_is_read_as_a_message_to_write() {
        let Opening::Compose(asked) =
            what_was_handed_over("mailto:someone@example.com?subject=Hi").expect("a mailto link")
        else {
            panic!("a mailto link was not read as a message to write");
        };

        assert_eq!(asked.to, "someone@example.com");
        assert_eq!(asked.subject, "Hi");
    }

    #[test]
    fn test_an_ics_path_is_read_as_a_calendar_file() {
        assert_eq!(
            what_was_handed_over(r"C:\Users\somebody\Downloads\invite.ics").expect("an ics file"),
            Opening::CalendarFile(PathBuf::from(r"C:\Users\somebody\Downloads\invite.ics"))
        );
    }

    #[test]
    fn test_a_vcf_path_is_read_as_a_contact_file() {
        assert_eq!(
            what_was_handed_over(r"C:\Users\somebody\Downloads\card.vcf").expect("a vcf file"),
            Opening::ContactFile(PathBuf::from(r"C:\Users\somebody\Downloads\card.vcf"))
        );
    }

    #[test]
    fn test_a_webcal_link_becomes_the_web_address_it_really_is() {
        // Both spellings, because a calendar published anywhere is linked as
        // one or the other and a person following either means the same thing.
        // Both go to https: a calendar carries somebody's whole week, and
        // sometimes a sign-in with it.
        assert_eq!(
            what_was_handed_over("webcal://example.com/team.ics").expect("a webcal link"),
            Opening::CalendarSubscription("https://example.com/team.ics".to_string())
        );
        assert_eq!(
            what_was_handed_over("webcals://example.com/team.ics").expect("a webcals link"),
            Opening::CalendarSubscription("https://example.com/team.ics".to_string())
        );
    }

    #[test]
    fn test_every_shape_is_matched_without_case_the_way_windows_matches_them() {
        // Windows compares extensions and schemes without case, and hands over
        // whatever was on the disk or in the page. A comparison that missed
        // `INVITE.ICS` would refuse a file this program is registered to open,
        // and the person would see an error naming their own file.
        assert!(matches!(
            what_was_handed_over("MAILTO:a@b.com"),
            Ok(Opening::Compose(_))
        ));
        assert!(matches!(
            what_was_handed_over("WebCal://example.com/x.ics"),
            Ok(Opening::CalendarSubscription(_))
        ));
        assert!(matches!(
            what_was_handed_over(r"C:\INVITE.ICS"),
            Ok(Opening::CalendarFile(_))
        ));
        assert!(matches!(
            what_was_handed_over(r"C:\Card.VcF"),
            Ok(Opening::ContactFile(_))
        ));
    }

    #[test]
    fn test_something_this_program_does_not_open_is_refused_rather_than_guessed_at() {
        // A guess opens the wrong window on somebody's file. Windows only ever
        // hands over one of the four shapes above, so anything else arrived
        // because a person typed it, and they deserve to be told rather than
        // to watch a blank composer open.
        for refused in [
            r"C:\Users\somebody\report.pdf",
            "https://example.com",
            "someone@example.com",
            "",
            "   ",
            ".ics",
            ".vcf",
            "webcal://",
            "ics",
        ] {
            assert!(
                what_was_handed_over(refused).is_err(),
                "{refused:?} was accepted"
            );
        }
    }

    #[test]
    fn test_a_file_named_only_by_its_extension_is_not_a_calendar() {
        // `.ics` on its own is a hidden file with no name, not a calendar
        // called nothing, and treating it as one would try to read whatever a
        // relative path with that name resolved to.
        assert!(what_was_handed_over(".ics").is_err());
        assert!(what_was_handed_over("x.ics").is_ok(), "a one letter name");
    }

    #[test]
    fn test_a_link_that_could_not_be_read_carries_its_own_reason_up() {
        // The refusal a person sees has to say what was wrong with what they
        // followed, not a general complaint from here.
        let why = what_was_handed_over("https://example.com/x")
            .expect_err("a web address was accepted")
            .to_string();

        assert!(why.contains("https://example.com/x"), "{why}");
        assert!(
            why.contains(".ics"),
            "the refusal does not say what it opens: {why}"
        );
    }

    #[test]
    fn test_surrounding_whitespace_does_not_stop_a_thing_being_recognised() {
        // A shortcut, a copied line, or a shell can bring one along.
        assert!(matches!(
            what_was_handed_over("  mailto:a@b.com "),
            Ok(Opening::Compose(_))
        ));
    }

    #[test]
    fn test_a_file_too_big_to_be_a_calendar_is_refused_before_it_is_read() {
        // A double click on something claiming to be a meeting and running to
        // hundreds of megabytes should not be a machine that stops answering.
        // Refused on its size, before any of it is read.
        let folder = tempfile::tempdir().expect("a temporary folder");
        let path = folder.path().join("enormous.ics");
        std::fs::write(
            &path,
            vec![b'x'; (MOST_A_HANDED_OVER_FILE_MAY_HOLD + 1) as usize],
        )
        .expect("could not write the file");

        let why = text_of(&path)
            .expect_err("an enormous file was read")
            .to_string();
        assert!(why.contains("larger than"), "{why}");
    }

    #[test]
    fn test_an_ordinary_file_is_read_whole() {
        // The other half of the same measurement. A ceiling that refused
        // everything would make the test above pass while nothing worked.
        let folder = tempfile::tempdir().expect("a temporary folder");
        let path = folder.path().join("small.vcf");
        std::fs::write(&path, "BEGIN:VCARD\r\nFN:Somebody\r\nEND:VCARD\r\n")
            .expect("could not write the file");

        assert_eq!(
            text_of(&path).expect("an ordinary file"),
            "BEGIN:VCARD\r\nFN:Somebody\r\nEND:VCARD\r\n"
        );
    }

    #[test]
    fn test_a_file_that_is_not_utf8_is_read_rather_than_refused() {
        // Calendar and contact files are written by every program there is,
        // some of them still emitting Latin-1. Refusing the file would lose
        // every event in it over one accented name.
        let folder = tempfile::tempdir().expect("a temporary folder");
        let path = folder.path().join("latin.vcf");
        std::fs::write(&path, [b'F', b'N', b':', 0xE9, b'\r', b'\n']).expect("could not write");

        let read = text_of(&path).expect("a file that is not utf8 was refused");
        assert!(read.starts_with("FN:"), "{read:?}");
    }

    #[test]
    fn test_a_file_that_is_not_there_says_so_rather_than_reporting_nothing() {
        let why = text_of(Path::new(r"C:\no\such\file\anywhere.ics"))
            .expect_err("a missing file was read")
            .to_string();

        assert!(why.contains("anywhere.ics"), "{why}");
    }

    #[test]
    fn test_the_words_for_a_reading_cover_every_way_it_can_come_out() {
        // Every arm, because this is read aloud and a wrong arm tells somebody
        // nothing was imported when a hundred events were.
        assert!(
            what_reading_the_calendar_did(EventsRead::default(), "empty.ics").contains("no events")
        );
        assert!(
            what_reading_the_calendar_did(
                EventsRead {
                    added: 3,
                    ..EventsRead::default()
                },
                "team.ics"
            )
            .contains("3 events added")
        );
        assert!(
            what_reading_the_calendar_did(
                EventsRead {
                    brought_up_to_date: 2,
                    ..EventsRead::default()
                },
                "team.ics"
            )
            .contains("brought up to date")
        );
        let both = what_reading_the_calendar_did(
            EventsRead {
                added: 1,
                brought_up_to_date: 4,
                ..EventsRead::default()
            },
            "team.ics",
        );
        assert!(both.contains("1 event added"), "{both}");
        assert!(both.contains("4 already here"), "{both}");
    }

    #[test]
    fn test_one_event_is_not_read_out_as_one_events() {
        // It is spoken. "1 events added" is a stumble every time somebody
        // opens a single invitation, which is the commonest case there is.
        let one = what_reading_the_calendar_did(
            EventsRead {
                added: 1,
                ..EventsRead::default()
            },
            "invite.ics",
        );

        assert!(one.contains("1 event added"), "{one}");
        assert!(!one.contains("1 events"), "{one}");
    }

    #[test]
    fn test_the_counts_add_up_to_what_was_in_the_file() {
        assert_eq!(
            EventsRead {
                added: 2,
                brought_up_to_date: 3,
                ..EventsRead::default()
            }
            .altogether(),
            5
        );
    }

    /// One meeting, written the way a mail client sends an invitation.
    const AN_INVITATION: &str = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:a-meeting@example.com\r\n\
         DTSTART:20260901T090000Z\r\n\
         DTEND:20260901T100000Z\r\n\
         SUMMARY:Planning\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    fn a_cache() -> (tempfile::TempDir, MessageCache) {
        let folder = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(folder.path().to_path_buf(), None).expect("a cache");
        (folder, cache)
    }

    #[test]
    fn test_reading_a_calendar_file_really_stores_the_events_in_it() {
        // The end of the chain, against a real database rather than a parsed
        // structure. Everything above this could be right and the event still
        // reach nothing a person can open.
        let (_folder, cache) = a_cache();

        let read = read_calendar_file(&cache, AN_INVITATION).expect("the file would not read");

        assert_eq!(read.added, 1);
        let stored = cache
            .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("could not read the events back");
        assert_eq!(stored.len(), 1, "{stored:?}");
        assert_eq!(stored[0].summary, "Planning");
    }

    #[test]
    fn test_an_imported_event_is_not_offered_back_to_anybodys_calendar_server() {
        // The one that would do real damage. An event read out of a file
        // somebody was sent has not come from any of their accounts, and one
        // marked as a server's own, or as waiting to be sent, is a meeting
        // invitation appearing on their work calendar as something they
        // created.
        let (_folder, cache) = a_cache();
        read_calendar_file(&cache, AN_INVITATION).expect("the file would not read");

        let stored = cache
            .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("could not read the events back");
        let event = stored.first().expect("nothing was stored");

        assert!(!event.pending, "an imported event is waiting to be sent");
        assert_eq!(event.source_provider, None, "it claims to be a server's");
        assert_eq!(event.etag, None);
        assert_eq!(event.web_link, None);
        assert_eq!(event.account_id, WHERE_IMPORTED_THINGS_GO);
        assert!(
            cache
                .pending_calendar_events(WHERE_IMPORTED_THINGS_GO)
                .expect("could not ask what is waiting")
                .is_empty(),
            "an imported event is queued to go out"
        );
    }

    #[test]
    fn test_opening_the_same_invitation_twice_leaves_one_event_and_not_two() {
        // Ordinary: an invitation is sent, then sent again when the time
        // changes. Two rows means two alarms and two entries in the day, and
        // nothing to say which is current.
        let (_folder, cache) = a_cache();

        let first = read_calendar_file(&cache, AN_INVITATION).expect("the first reading");
        let moved = AN_INVITATION.replace("SUMMARY:Planning", "SUMMARY:Planning, moved");
        let second = read_calendar_file(&cache, &moved).expect("the second reading");

        assert_eq!(first.added, 1);
        assert_eq!(second.added, 0, "the same event was added twice");
        assert_eq!(second.brought_up_to_date, 1);

        let stored = cache
            .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("could not read the events back");
        assert_eq!(stored.len(), 1, "two rows for one meeting: {stored:?}");
        assert_eq!(stored[0].summary, "Planning, moved");
    }

    #[test]
    fn test_something_that_is_not_a_calendar_at_all_is_refused() {
        // A file named `.ics` holding anything else. Storing nothing and
        // reporting success would tell somebody their invitation was imported.
        let (_folder, cache) = a_cache();

        assert!(read_calendar_file(&cache, "this is not a calendar").is_err());
    }

    /// A real invitation, which the constant above is not: it carries the line
    /// that says somebody is asking, and names who is asking.
    const SOMEBODY_ASKING: &str = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         METHOD:REQUEST\r\n\
         BEGIN:VEVENT\r\n\
         UID:asked@example.com\r\n\
         DTSTART:20260901T090000Z\r\n\
         DTEND:20260901T100000Z\r\n\
         SUMMARY:Planning\r\n\
         ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
         ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    #[test]
    fn test_an_invitation_is_said_to_be_one_rather_than_filed_in_silence() {
        // Opening a calendar file put the meeting straight in the calendar and
        // said how many events were added, which is true and is not the thing
        // that matters: somebody is waiting for an answer, and nothing said so.
        // Answering is not possible from here yet, and the sentence has to say
        // that too rather than imply a button that is not there.
        let (_folder, cache) = a_cache();

        let read = read_calendar_file(&cache, SOMEBODY_ASKING).expect("the file would not read");
        let said = what_reading_the_calendar_did(read, "invite.ics");

        assert!(
            said.contains("Ada Lovelace"),
            "the sentence does not say who is asking: {said}"
        );
        assert!(
            said.to_lowercase().contains("invitation"),
            "the sentence does not say it is an invitation: {said}"
        );
    }

    #[test]
    fn test_an_ordinary_calendar_file_is_not_called_an_invitation() {
        // Almost every calendar file somebody opens is a meeting they were
        // sent or exported, carrying no question at all. Calling one of those
        // an invitation would tell somebody to answer a person who never
        // asked.
        let (_folder, cache) = a_cache();

        let read = read_calendar_file(&cache, AN_INVITATION).expect("the file would not read");
        let said = what_reading_the_calendar_did(read, "meeting.ics");

        assert!(
            !said.to_lowercase().contains("invitation"),
            "a plain calendar file was called an invitation: {said}"
        );
    }

    #[test]
    fn test_an_imported_event_keeps_the_identity_the_file_gave_it() {
        // What makes the reading above able to recognise the same meeting
        // again. Without it every re-open is a new row.
        let (_folder, cache) = a_cache();
        read_calendar_file(&cache, AN_INVITATION).expect("the file would not read");

        let stored = cache
            .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("could not read the events back");
        assert_eq!(
            stored[0].provider_event_id.as_deref(),
            Some("a-meeting@example.com")
        );
        assert_eq!(stored[0].calendar_id.as_deref(), Some(IMPORTED_CALENDAR));
    }
}

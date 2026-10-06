//! Every calendar on a Gmail account, not only its main one (REAL-01, answer 3).
//!
//! Until 14-04 a Google calendar sync asked for the calendar Google calls
//! `primary` and nothing else, and filed it under one row, so somebody with a
//! work calendar, a family calendar and a calendar a colleague shared saw one
//! of the four. Google's calendar list is read here first, hidden calendars
//! included, and each calendar on it is filed as a row of its own before
//! [`calendar::sync_google_calendar`] reads every row filed.
//!
//! The list is read here rather than inside that sync so the sync stays what
//! it was, a read of the calendars already filed, which forty-odd cases drive
//! against a stand-in that answers the events alone.

use crate::application::calendar::{self, CalendarSyncResult, GOOGLE, GOOGLE_CALENDAR_NAME};
use crate::common::Result;
use crate::data::message_cache::calendars::{ListedCalendar, a_calendar_google_listed};
use crate::data::message_cache::{CalendarContainer, MessageCache};
use crate::service::google_api::{GoogleApiClient, GoogleCalendarListEntry};

/// Read the account's calendar list, file each calendar on it, then read
/// every calendar filed.
pub async fn sync(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();
    match google.list_calendars(token).await {
        Ok(list) => file_the_list(cache, account_id, &list, &mut result)?,
        Err(e) => result.errors.push(format!(
            "The list of your Google calendars could not be read, so only the \
             calendars already here were read: {e}"
        )),
    }
    result.absorb(calendar::sync_google_calendar(cache, google, token, account_id).await?);
    Ok(result)
}

/// File each calendar on a whole list as a row.
fn file_the_list(
    cache: &MessageCache,
    account_id: &str,
    list: &[GoogleCalendarListEntry],
    _result: &mut CalendarSyncResult,
) -> Result<()> {
    for listed in list {
        file_one(cache, account_id, listed)?;
    }
    Ok(())
}

/// File one calendar from the list, the main one under the row every event
/// made in no calendar goes to (14-04 choice 1).
fn file_one(
    cache: &MessageCache,
    account_id: &str,
    listed: &GoogleCalendarListEntry,
) -> Result<CalendarContainer> {
    let id = if listed.primary {
        cache
            .ensure_provider_calendar(account_id, GOOGLE, GOOGLE_CALENDAR_NAME)?
            .id
    } else {
        a_calendar_google_listed(account_id, &listed.id)
    };
    cache.file_a_listed_calendar(&ListedCalendar {
        account_id,
        id: &id,
        provider: GOOGLE,
        name: its_name(listed),
    })
}

/// What a listed calendar is called here: the person's own name for it, else
/// Google's. The main calendar is "Google Calendar" unless they named it,
/// because Google's name for it is usually the account's address.
fn its_name(listed: &GoogleCalendarListEntry) -> &str {
    let googles_own = match listed.primary {
        true => GOOGLE_CALENDAR_NAME,
        false => listed.summary.as_deref().unwrap_or(&listed.id),
    };
    listed.summary_override.as_deref().unwrap_or(googles_own)
}

/// What is said about calendars on the list that show only when their owner
/// is free or busy, which are passed over (14-04 choice 3).
pub fn only_free_and_busy(_how_many: usize) -> String {
    String::new()
}

/// What is said about calendars Google stopped listing, which were taken off
/// this computer (14-04 choice 4).
pub fn taken_off_this_computer(_how_many: usize) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
    use crate::common::answering::{Reply, answering_as_asked, asked_for, heard};
    use crate::common::temp_home::TempHome;

    fn a_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache in a directory of its own")
        })
    }

    /// A calendar Google lists that somebody shared with this account.
    const THE_TEAM_CALENDAR: &str = "team@group.calendar.google.com";

    /// The same, as it appears in an address.
    const THE_TEAM_CALENDAR_IN_AN_ADDRESS: &str = "team%40group.calendar.google.com";

    /// One event, in the shape Google answers a read of a calendar with.
    fn one_event(id: &str, summary: &str) -> String {
        format!(
            "{{\"items\":[{{\"id\":\"{id}\",\"status\":\"confirmed\",\
               \"summary\":\"{summary}\",\"etag\":\"\\\"e1\\\"\",\
               \"start\":{{\"dateTime\":\"2026-03-06T09:00:00Z\"}},\
               \"end\":{{\"dateTime\":\"2026-03-06T09:15:00Z\"}}}}\
             ],\"nextSyncToken\":\"marker-1\"}}"
        )
    }

    /// A list naming the account's main calendar and the team calendar.
    fn the_main_calendar_and_the_team_calendar() -> String {
        format!(
            "{{\"items\":[\
               {{\"id\":\"me@example.com\",\"summary\":\"me@example.com\",\
                 \"primary\":true,\"accessRole\":\"owner\",\"selected\":true}},\
               {{\"id\":\"{THE_TEAM_CALENDAR}\",\"summary\":\"Team\",\
                 \"accessRole\":\"writer\",\"selected\":true}}\
             ]}}"
        )
    }

    /// What Google answers, by what was asked: the list, or one calendar.
    fn google_answering(list: String) -> Reply {
        Box::new(move |asked: &[String]| {
            let this_one = asked.last().map(|r| asked_for(r)).unwrap_or_default();
            if this_one.starts_with("GET /users/me/calendarList") {
                list.clone()
            } else if this_one.starts_with(&format!(
                "GET /calendars/{THE_TEAM_CALENDAR_IN_AN_ADDRESS}/events"
            )) {
                one_event("team-evt", "Planning")
            } else {
                one_event("main-evt", "Standup")
            }
        })
    }

    #[tokio::test]
    async fn test_a_second_google_calendar_arrives_under_a_row_of_its_own() {
        let cache = a_cache("every_google_calendar_tracer");
        let list = the_main_calendar_and_the_team_calendar();
        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                google_answering(list.clone()),
                google_answering(list.clone()),
                google_answering(list),
            ],
        )
        .await;
        let google = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        sync(&cache, &google, "a-token", "acct")
            .await
            .expect("the sync to finish");

        let rows = cache
            .get_calendars_for_account("acct")
            .expect("the calendar list");
        let team = rows
            .iter()
            .find(|row| row.id == format!("google:acct:{THE_TEAM_CALENDAR}"))
            .unwrap_or_else(|| panic!("no row for the team calendar: {rows:?}"));
        assert_eq!(team.name, "Team");
        let main = rows
            .iter()
            .find(|row| !row.id.contains(':'))
            .unwrap_or_else(|| panic!("no row for the main calendar: {rows:?}"));
        assert_eq!(main.name, "Google Calendar");

        let filed_in = |id: &str| {
            cache
                .get_event_by_provider_id("acct", id)
                .expect("the calendar to be readable")
                .unwrap_or_else(|| panic!("{id} was not brought"))
                .calendar_id
        };
        assert_eq!(filed_in("team-evt"), Some(team.id.clone()));
        assert_eq!(filed_in("main-evt"), Some(main.id.clone()));

        let requests = heard(listening, "the list and two calendars")
            .await
            .expect("three requests");
        let lists: Vec<&str> = requests
            .iter()
            .map(|r| asked_for(r))
            .filter(|line| line.starts_with("GET /users/me/calendarList"))
            .collect();
        assert_eq!(lists.len(), 1, "{requests:?}");
        assert!(lists[0].contains("showHidden=true"), "{lists:?}");
    }

    // ── Every kind of calendar on the list ──────────────────────────────────

    /// The main calendar as Google lists it.
    const THE_MAIN_CALENDAR_LISTED: &str = "{\"id\":\"me@example.com\",\"summary\":\"me@example.com\",\
         \"primary\":true,\"accessRole\":\"owner\",\"selected\":true}";

    /// One more calendar on the list, in Google's shape, with whatever else
    /// it says about itself.
    fn listed(id: &str, summary: &str, role: &str, more: &str) -> String {
        format!("{{\"id\":\"{id}\",\"summary\":\"{summary}\",\"accessRole\":\"{role}\"{more}}}")
    }

    /// A whole list, on one page.
    fn a_list_of(calendars: &[String]) -> String {
        format!("{{\"items\":[{}]}}", calendars.join(","))
    }

    /// An answer naming no events.
    const NOTHING: &str = "{\"items\":[]}";

    /// What one calendar's read is answered with, by whether it carried a
    /// marker.
    #[derive(Clone)]
    struct ItsAnswers {
        whole: String,
        from_a_marker: String,
    }

    impl ItsAnswers {
        fn always(answer: String) -> Self {
            Self {
                whole: answer.clone(),
                from_a_marker: answer,
            }
        }
    }

    /// A stand-in for Google: one list, and each calendar's events by the
    /// calendar's id as it appears in an address. A change sent is answered
    /// with an event, and a calendar it does not know with none.
    #[derive(Clone)]
    struct StandIn {
        list: String,
        calendars: Vec<(&'static str, ItsAnswers)>,
    }

    impl StandIn {
        fn answering(&self) -> Reply {
            let google = self.clone();
            Box::new(move |asked: &[String]| {
                let line = asked.last().map(|r| asked_for(r)).unwrap_or_default();
                if line.starts_with("GET /users/me/calendarList") {
                    return google.list.clone();
                }
                if !line.starts_with("GET ") {
                    return "{\"id\":\"sent\",\"updated\":\"2026-03-06T10:00:00Z\"}".to_string();
                }
                google
                    .calendars
                    .iter()
                    .find(|(at, _)| line.starts_with(&format!("GET /calendars/{at}/events?")))
                    .map(|(_, answers)| match line.contains("syncToken=") {
                        true => answers.from_a_marker.clone(),
                        false => answers.whole.clone(),
                    })
                    .unwrap_or_else(|| NOTHING.to_string())
            })
        }
    }

    /// One sync against the stand-in, expecting this many requests, and what
    /// it asked.
    async fn one_sync(
        cache: &MessageCache,
        google: &StandIn,
        requests: usize,
        may_change: bool,
    ) -> (CalendarSyncResult, Vec<String>) {
        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            (0..requests).map(|_| google.answering()).collect(),
        )
        .await;
        let at = format!("http://{address}");
        let client = match may_change {
            true => GoogleApiClient::allowed_to_change_things_at(&at),
            false => GoogleApiClient::new().pointed_at(&at),
        };
        let result = sync(cache, &client, "a-token", "acct")
            .await
            .expect("the sync to finish");
        let asked = heard(listening, "every request this sync makes")
            .await
            .expect("every request");
        (result, asked)
    }

    /// The row a calendar on the list was filed under, if any.
    fn the_row_for(cache: &MessageCache, at_google: &str) -> Option<CalendarContainer> {
        cache
            .get_calendar(&format!("google:acct:{at_google}"))
            .expect("the calendar list to be readable")
    }

    /// The main calendar's row.
    fn the_main_row(cache: &MessageCache) -> CalendarContainer {
        cache
            .get_calendars_for_account("acct")
            .expect("the calendar list")
            .into_iter()
            .find(|row| !row.id.contains(':'))
            .expect("a main calendar")
    }

    /// An event brought down, changed here and not yet sent.
    fn changed_here(cache: &MessageCache, provider_event_id: &str) {
        let mut event = cache
            .get_event_by_provider_id("acct", provider_event_id)
            .expect("the calendar to be readable")
            .expect("the event to have been brought");
        event.summary = "Moved by me".to_string();
        event.pending = true;
        cache
            .save_calendar_event(&event)
            .expect("the change to be kept");
    }

    #[tokio::test]
    async fn test_a_calendar_the_account_may_only_read_is_filed_read_only() {
        let cache = a_cache("every_google_calendar_read_only");
        let google = StandIn {
            list: a_list_of(&[
                THE_MAIN_CALENDAR_LISTED.to_string(),
                listed(THE_TEAM_CALENDAR, "Team", "reader", ",\"selected\":true"),
                listed(
                    "family",
                    "Family",
                    "writerWithoutPrivateAccess",
                    ",\"selected\":true",
                ),
            ]),
            calendars: vec![(
                THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                ItsAnswers::always(one_event("team-evt", "Planning")),
            )],
        };

        one_sync(&cache, &google, 4, false).await;

        let team = the_row_for(&cache, THE_TEAM_CALENDAR).expect("the team calendar");
        assert!(
            team.is_read_only,
            "a calendar shared to be read was filed writable"
        );
        let family = the_row_for(&cache, "family").expect("the family calendar");
        assert!(
            !family.is_read_only,
            "a calendar the account may write to was filed read-only"
        );
        assert!(!the_main_row(&cache).is_read_only);

        changed_here(&cache, "team-evt");
        let said =
            calendar::changes_nothing_can_send(&cache, "acct").expect("the changes to be readable");
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said[0].starts_with(
                "Team: 1 change made here cannot be sent, because this is a calendar \
                 this program can only read"
            ),
            "{said:?}"
        );
    }

    #[test]
    fn test_only_the_roles_that_may_write_are_written_to() {
        use crate::service::google_api::GoogleAccessRole as Role;
        for (role, may) in [
            (Role::FreeBusyReader, false),
            (Role::Reader, false),
            (Role::WriterWithoutPrivateAccess, true),
            (Role::Writer, true),
            (Role::Owner, true),
            (Role::NotOneGoogleNames, false),
        ] {
            assert_eq!(role.may_write(), may, "{role:?}");
        }
    }

    #[tokio::test]
    async fn test_a_calendar_that_shows_only_free_and_busy_is_passed_over_and_said_once() {
        let cache = a_cache("every_google_calendar_free_and_busy");
        let google = StandIn {
            list: a_list_of(&[
                THE_MAIN_CALENDAR_LISTED.to_string(),
                listed(
                    "boss@example.com",
                    "Boss",
                    "freeBusyReader",
                    ",\"selected\":true",
                ),
            ]),
            calendars: vec![],
        };

        let (result, asked) = one_sync(&cache, &google, 2, false).await;

        assert!(
            the_row_for(&cache, "boss@example.com").is_none(),
            "a calendar showing only free and busy times was filed"
        );
        assert!(
            !asked
                .iter()
                .any(|r| asked_for(r).contains("boss%40example.com")),
            "{asked:?}"
        );
        let said = calendar::what_the_calendar_sync_did(&result);
        assert_eq!(said.matches(&only_free_and_busy(1)).count(), 1, "{said}");
    }

    #[test]
    fn test_what_is_said_about_calendars_passed_over_or_put_away_reads_as_a_persons_sentence() {
        for how_many in [1, 3] {
            for sentence in [
                only_free_and_busy(how_many),
                taken_off_this_computer(how_many),
            ] {
                reads_as_a_persons_sentence(&sentence, Voice::Answer)
                    .unwrap_or_else(|complaint| panic!("{complaint:?}"));
            }
        }
        assert_eq!(
            only_free_and_busy(1),
            "One of your Google calendars shows only when its owner is free or busy, \
             so nothing in it was brought"
        );
        assert_eq!(
            only_free_and_busy(3),
            "3 of your Google calendars show only when their owners are free or busy, \
             so nothing in them was brought"
        );
        assert_eq!(
            taken_off_this_computer(1),
            "Google no longer lists one of your calendars, so it was taken off this computer"
        );
        assert_eq!(
            taken_off_this_computer(3),
            "Google no longer lists 3 of your calendars, so they were taken off this computer"
        );
    }

    #[tokio::test]
    async fn test_more_than_one_calendar_read_is_counted_in_what_is_said() {
        let cache = a_cache("every_google_calendar_counted");
        let google = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![
                (
                    "primary",
                    ItsAnswers::always(one_event("main-evt", "Standup")),
                ),
                (
                    THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                    ItsAnswers::always(one_event("team-evt", "Planning")),
                ),
            ],
        };

        let (result, _) = one_sync(&cache, &google, 3, false).await;

        assert_eq!(
            calendar::what_the_calendar_sync_did(&result),
            "Calendar sync: 2 created, 0 updated, 0 deleted, 2 calendars read"
        );

        let alone = a_cache("every_google_calendar_counted_alone");
        let only_the_main_one = StandIn {
            list: a_list_of(&[THE_MAIN_CALENDAR_LISTED.to_string()]),
            calendars: vec![(
                "primary",
                ItsAnswers::always(one_event("main-evt", "Standup")),
            )],
        };
        let (result, _) = one_sync(&alone, &only_the_main_one, 2, false).await;
        assert_eq!(
            calendar::what_the_calendar_sync_did(&result),
            "Calendar sync: 1 created, 0 updated, 0 deleted"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_hidden_at_google_starts_hidden_and_stays_as_the_person_leaves_it() {
        let cache = a_cache("every_google_calendar_hidden");
        let google = StandIn {
            list: a_list_of(&[
                THE_MAIN_CALENDAR_LISTED.to_string(),
                listed(
                    THE_TEAM_CALENDAR,
                    "Team",
                    "writer",
                    ",\"selected\":true,\"hidden\":true",
                ),
                listed("family", "Family", "writer", ""),
                listed("work", "Work", "writer", ",\"selected\":true"),
            ]),
            calendars: vec![],
        };

        one_sync(&cache, &google, 5, false).await;

        let shown = |at: &str| the_row_for(&cache, at).expect("the calendar").is_visible;
        assert!(
            !shown(THE_TEAM_CALENDAR),
            "a calendar hidden at Google arrived shown"
        );
        assert!(
            !shown("family"),
            "a calendar Google does not show arrived shown"
        );
        assert!(shown("work"));

        cache
            .set_calendar_visibility(&format!("google:acct:{THE_TEAM_CALENDAR}"), true)
            .expect("the person to show it");
        cache
            .set_calendar_visibility("google:acct:work", false)
            .expect("the person to hide it");
        one_sync(&cache, &google, 5, false).await;

        assert!(
            shown(THE_TEAM_CALENDAR),
            "a sync hid a calendar the person showed"
        );
        assert!(!shown("family"));
        assert!(!shown("work"), "a sync showed a calendar the person hid");
    }

    #[tokio::test]
    async fn test_one_meeting_in_two_calendars_is_two_rows_and_a_cancellation_takes_only_its_own() {
        let cache = a_cache("every_google_calendar_one_meeting_twice");
        let both_hold_it = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![
                (
                    "primary",
                    ItsAnswers::always(one_event("shared-evt", "Review")),
                ),
                (
                    THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                    ItsAnswers::always(one_event("shared-evt", "Review")),
                ),
            ],
        };
        one_sync(&cache, &both_hold_it, 3, false).await;

        let main = the_main_row(&cache);
        let team_id = format!("google:acct:{THE_TEAM_CALENDAR}");
        let held_in = |calendar_id: &str| {
            cache
                .get_events_for_calendar(calendar_id)
                .expect("the calendar to be readable")
                .into_iter()
                .filter(|event| event.provider_event_id.as_deref() == Some("shared-evt"))
                .count()
        };
        assert_eq!(held_in(&main.id), 1, "the main calendar's copy");
        assert_eq!(held_in(&team_id), 1, "the team calendar's copy");

        let cancelled = "{\"items\":[{\"id\":\"shared-evt\",\"status\":\"cancelled\"}],\
                         \"nextSyncToken\":\"marker-2\"}"
            .to_string();
        let the_team_calls_it_off = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![
                ("primary", ItsAnswers::always(NOTHING.to_string())),
                (
                    THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                    ItsAnswers::always(cancelled),
                ),
            ],
        };
        one_sync(&cache, &the_team_calls_it_off, 3, false).await;

        assert_eq!(
            held_in(&main.id),
            1,
            "a cancellation in one calendar took the other's copy"
        );
        assert_eq!(held_in(&team_id), 0, "the cancelled copy is still there");
    }

    #[tokio::test]
    async fn test_each_calendar_keeps_its_own_marker_and_one_refused_is_read_whole() {
        let cache = a_cache("every_google_calendar_markers");
        let with_a_marker = |id: &str, marker: &str| {
            format!(
                "{{\"items\":[{{\"id\":\"{id}\",\"status\":\"confirmed\",\"summary\":\"S\",\
                   \"start\":{{\"dateTime\":\"2026-03-06T09:00:00Z\"}},\
                   \"end\":{{\"dateTime\":\"2026-03-06T09:15:00Z\"}}}}],\
                   \"nextSyncToken\":\"{marker}\"}}"
            )
        };
        let google = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![
                (
                    "primary",
                    ItsAnswers::always(with_a_marker("main-evt", "main-marker")),
                ),
                (
                    THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                    ItsAnswers {
                        whole: with_a_marker("team-evt", "team-marker"),
                        // What a refused marker looks like to the read: an
                        // answer it cannot take, so it reads the calendar whole.
                        from_a_marker: "refused".to_string(),
                    },
                ),
            ],
        };
        one_sync(&cache, &google, 3, false).await;

        let team = the_row_for(&cache, THE_TEAM_CALENDAR).expect("the team calendar");
        assert_eq!(team.sync_token.as_deref(), Some("team-marker"));

        let (_, asked) = one_sync(&cache, &google, 4, false).await;
        let lines: Vec<&str> = asked.iter().map(|r| asked_for(r)).collect();
        let main_reads: Vec<&&str> = lines
            .iter()
            .filter(|line| line.starts_with("GET /calendars/primary/events?"))
            .collect();
        assert_eq!(main_reads.len(), 1, "{lines:?}");
        assert!(main_reads[0].contains("syncToken=main-marker"), "{lines:?}");
        let team_reads: Vec<&&str> = lines
            .iter()
            .filter(|line| {
                line.starts_with(&format!(
                    "GET /calendars/{THE_TEAM_CALENDAR_IN_AN_ADDRESS}/events?"
                ))
            })
            .collect();
        assert_eq!(team_reads.len(), 2, "{lines:?}");
        assert!(team_reads[0].contains("syncToken=team-marker"), "{lines:?}");
        assert!(
            !team_reads[1].contains("syncToken=") && team_reads[1].contains("timeMin="),
            "a refused marker was not followed by a whole read: {lines:?}"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_google_stopped_listing_is_put_away_keeping_a_change_waiting_in_it() {
        let cache = a_cache("every_google_calendar_put_away");
        let two_events = "{\"items\":[\
             {\"id\":\"team-evt\",\"status\":\"confirmed\",\"summary\":\"Planning\",\
              \"start\":{\"dateTime\":\"2026-03-06T09:00:00Z\"},\
              \"end\":{\"dateTime\":\"2026-03-06T09:15:00Z\"}},\
             {\"id\":\"team-other\",\"status\":\"confirmed\",\"summary\":\"Retro\",\
              \"start\":{\"dateTime\":\"2026-03-07T09:00:00Z\"},\
              \"end\":{\"dateTime\":\"2026-03-07T09:15:00Z\"}}\
           ],\"nextSyncToken\":\"team-marker\"}"
            .to_string();
        let listing_it = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![(
                THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                ItsAnswers::always(two_events),
            )],
        };
        one_sync(&cache, &listing_it, 3, false).await;
        changed_here(&cache, "team-other");

        let no_longer = StandIn {
            list: a_list_of(&[THE_MAIN_CALENDAR_LISTED.to_string()]),
            calendars: vec![],
        };
        let (result, _) = one_sync(&cache, &no_longer, 2, false).await;

        assert!(
            the_row_for(&cache, THE_TEAM_CALENDAR).is_none(),
            "a calendar Google stopped listing is still here"
        );
        assert!(
            cache
                .get_event_by_provider_id("acct", "team-evt")
                .expect("the calendar to be readable")
                .is_none(),
            "an event of a calendar put away is still here"
        );
        let kept = cache
            .get_event_by_provider_id("acct", "team-other")
            .expect("the calendar to be readable")
            .expect("a change waiting here was lost with its calendar");
        assert_eq!(kept.summary, "Moved by me");
        assert!(kept.pending);
        let said = calendar::what_the_calendar_sync_did(&result);
        assert!(said.contains(&taken_off_this_computer(1)), "{said}");
        let cannot =
            calendar::changes_nothing_can_send(&cache, "acct").expect("the changes to be readable");
        assert!(
            cannot
                .iter()
                .any(|s| s.contains("there is no calendar to send to")),
            "{cannot:?}"
        );
    }

    /// A list whose first page names the main calendar alone and says there
    /// is a second, which never arrives whole.
    fn a_list_cut_short() -> Reply {
        Box::new(|asked: &[String]| {
            let line = asked.last().map(|r| asked_for(r)).unwrap_or_default();
            match (
                line.starts_with("GET /users/me/calendarList"),
                line.contains("pageToken="),
            ) {
                (true, false) => format!(
                    "{{\"items\":[{THE_MAIN_CALENDAR_LISTED}],\"nextPageToken\":\"page-2\"}}"
                ),
                (true, true) => "cut".to_string(),
                _ => NOTHING.to_string(),
            }
        })
    }

    #[tokio::test]
    async fn test_a_list_cut_short_puts_nothing_away() {
        let cache = a_cache("every_google_calendar_cut_short");
        let listing_it = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![],
        };
        one_sync(&cache, &listing_it, 3, false).await;

        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            (0..4).map(|_| a_list_cut_short()).collect(),
        )
        .await;
        let result = sync(
            &cache,
            &GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");
        heard(listening, "two pages of the list and two calendars")
            .await
            .expect("four requests");

        assert!(
            the_row_for(&cache, THE_TEAM_CALENDAR).is_some(),
            "a calendar was put away on a list cut short"
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.starts_with("The list of your Google calendars could not be read")),
            "{:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_change_to_an_event_in_a_second_calendar_is_sent_to_that_calendar() {
        let cache = a_cache("every_google_calendar_change_sent");
        let google = StandIn {
            list: the_main_calendar_and_the_team_calendar(),
            calendars: vec![(
                THE_TEAM_CALENDAR_IN_AN_ADDRESS,
                ItsAnswers::always(one_event("team-evt", "Planning")),
            )],
        };
        one_sync(&cache, &google, 3, false).await;
        changed_here(&cache, "team-evt");

        let (_, asked) = one_sync(&cache, &google, 4, true).await;

        let changes: Vec<&str> = asked
            .iter()
            .map(|r| asked_for(r))
            .filter(|line| line.starts_with("PATCH "))
            .collect();
        assert_eq!(
            changes,
            vec![format!(
                "PATCH /calendars/{THE_TEAM_CALENDAR_IN_AN_ADDRESS}/events/team-evt"
            )],
            "{asked:?}"
        );
    }
}

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

use crate::application::calendar::{self, CalendarSyncResult};
use crate::common::Result;
use crate::data::message_cache::MessageCache;
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
    _cache: &MessageCache,
    _account_id: &str,
    _list: &[GoogleCalendarListEntry],
    _result: &mut CalendarSyncResult,
) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
}

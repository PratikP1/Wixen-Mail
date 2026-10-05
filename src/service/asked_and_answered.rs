//! The log line for one request a calendar, contacts or tasks sync sends, and
//! the reason word a provider gives when it refuses one (#22, REAL-01, 14-02).
//!
//! Until 14-02 no request any of those syncs made wrote a line at any level,
//! so a report about a Refresh that brought nothing could not say what was
//! asked or what came back. Every HTTP helper in the Google client and the
//! tasks client now writes one line per request it sends, retries included,
//! at info, which is the level a stored profile keeps.
//!
//! The line is built from parts and never from an error's text: a transport
//! error names the whole address, and the address of a sync carries its sync
//! marker and its page marker in the query. So the query is cut off, a path
//! segment holding an address is masked, and of a refusal's body only one
//! word of letters and underscores is kept.
//!
//! The tests at the foot of this file also hold each client to writing the
//! line, against a loopback stand-in. They live here rather than beside each
//! client because the clients' files are fingerprinted by seventeen guard
//! records between them, each re-measured by a whole-library run whenever a
//! test is added there; the line is this module's subject either way.

use crate::common::logging::mask_email;

/// The longest reason word kept. Google's reasons and statuses run to about
/// thirty letters (`accessNotConfigured`, `PERMISSION_DENIED`); a token is
/// longer, and is refused by its digits and punctuation before its length.
const LONGEST_REASON: usize = 64;

/// What the line says in place of an address that cannot be read as one, so
/// nothing unreadable, and nothing the query may have held, is copied.
const AN_ADDRESS_THAT_COULD_NOT_BE_READ: &str = "an address that could not be read";

/// Who a request was sent to, as the log names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoWasAsked {
    Google,
    GoogleTasks,
    MicrosoftToDo,
}

impl WhoWasAsked {
    /// The provider's name in the line.
    pub fn name(self) -> &'static str {
        match self {
            WhoWasAsked::Google => "Google",
            WhoWasAsked::GoogleTasks => "Google Tasks",
            WhoWasAsked::MicrosoftToDo => "Microsoft To Do",
        }
    }
}

/// What came back for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatCameBack<'a> {
    /// The status, and the provider's reason word when it refused and gave
    /// one.
    Answered {
        status: u16,
        reason: Option<&'a str>,
    },
    /// The request failed before any status arrived.
    NoAnswer,
}

/// The log line for one request: who was asked, the method, the host and
/// path, and what came back.
///
/// The method as its word, such as `GET`, rather than the transport's type, so
/// this file names no networking crate and holds no way out of the program.
pub fn the_line(
    who: WhoWasAsked,
    method: &str,
    address: &str,
    came_back: WhatCameBack<'_>,
) -> String {
    let outcome = match came_back {
        WhatCameBack::Answered {
            status,
            reason: Some(reason),
        } => format!("answered {status} {reason}"),
        WhatCameBack::Answered {
            status,
            reason: None,
        } => format!("answered {status}"),
        WhatCameBack::NoAnswer => "no answer came".to_string(),
    };
    format!(
        "Asked {}: {method} {}, {outcome}",
        who.name(),
        where_it_was_sent(address)
    )
}

/// The host and path of an address, with the query and the fragment cut off
/// and any path segment holding an address masked.
fn where_it_was_sent(address: &str) -> String {
    let Ok(url) = url::Url::parse(address) else {
        return AN_ADDRESS_THAT_COULD_NOT_BE_READ.to_string();
    };
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    let path: Vec<String> = url.path().split('/').map(masked).collect();
    format!(
        "{}{port}{}",
        url.host_str().unwrap_or_default(),
        path.join("/")
    )
}

/// One path segment, masked when it holds an address.
///
/// A calendar's identity can be an address, escaped in a path as `%40`.
/// Google's own word for the signed-in person, `@me`, has nothing before the
/// `@` and names nobody, so it is kept as it is.
fn masked(segment: &str) -> String {
    let readable = segment.replace("%40", "@");
    match readable.find('@') {
        Some(at) if at > 0 => mask_email(&readable),
        _ => segment.to_string(),
    }
}

/// The provider's reason for a refusal, as one word, read out of the body it
/// refused with; nothing when the body holds no word of the right shape.
///
/// Google's first `errors[].reason`, else its `error.status`, whichever is
/// first to be a word: letters and underscores only, and short. So no
/// message, address or token can pass through it, and the body itself is
/// never kept.
pub fn the_reason_word(body: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let error = answer.get("error")?;
    let first_reason = error
        .get("errors")
        .and_then(serde_json::Value::as_array)
        .and_then(|errors| errors.iter().find_map(|each| each.get("reason")?.as_str()));
    let status = error.get("status").and_then(serde_json::Value::as_str);
    [first_reason, status]
        .into_iter()
        .flatten()
        .find(|candidate| is_a_reason_word(candidate))
        .map(str::to_string)
}

/// Letters and underscores, at least one and at most [`LONGEST_REASON`].
fn is_a_reason_word(candidate: &str) -> bool {
    (1..=LONGEST_REASON).contains(&candidate.len())
        && candidate
            .chars()
            .all(|letter| letter.is_ascii_alphabetic() || letter == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, heard};
    use crate::presentation::accessibility::screen_reader::tests::CapturedLogs;
    use crate::service::google_api::{GoogleApiClient, THE_MAIN_CALENDAR};

    /// A calendar read the way a sync sends it, with its sync marker.
    const A_CALENDAR_READ: &str = "https://www.googleapis.com/calendar/v3/calendars/primary/events\
                                   ?singleEvents=false&maxResults=2500&syncToken=a-marker";

    /// Google's refusal when the Calendar interface is not turned on in the
    /// sign-in key's project, in the shape Google's error pages give.
    const NOT_TURNED_ON: &str = r#"{"error":{"code":403,"message":"Google Calendar API has not been used in project 123 before or it is disabled.","errors":[{"message":"Google Calendar API has not been used in project 123 before or it is disabled.","domain":"usageLimits","reason":"accessNotConfigured"}],"status":"PERMISSION_DENIED"}}"#;

    #[test]
    fn test_the_line_names_who_the_method_the_path_and_the_status_and_cuts_the_query() {
        let line = the_line(
            WhoWasAsked::Google,
            "GET",
            A_CALENDAR_READ,
            WhatCameBack::Answered {
                status: 200,
                reason: None,
            },
        );
        assert_eq!(
            line,
            "Asked Google: GET www.googleapis.com/calendar/v3/calendars/primary/events, \
             answered 200"
        );
    }

    #[test]
    fn test_a_path_segment_holding_an_address_is_masked_and_googles_word_for_me_is_kept() {
        // A calendar's identity can be an address, and Google files the main
        // one under the account's own; escaped in a path it reads `%40`.
        let calendar = the_line(
            WhoWasAsked::Google,
            "GET",
            "https://www.googleapis.com/calendar/v3/calendars/pratik%40gmail.com/events",
            WhatCameBack::Answered {
                status: 200,
                reason: None,
            },
        );
        assert!(!calendar.contains("pratik"), "{calendar}");
        assert!(
            calendar.contains("/calendars/pr***@gmail.com/events"),
            "{calendar}"
        );
        // `@me` names nobody: there is nothing before the `@`.
        let lists = the_line(
            WhoWasAsked::GoogleTasks,
            "GET",
            "https://tasks.googleapis.com/tasks/v1/users/@me/lists?maxResults=100",
            WhatCameBack::Answered {
                status: 200,
                reason: None,
            },
        );
        assert_eq!(
            lists,
            "Asked Google Tasks: GET tasks.googleapis.com/tasks/v1/users/@me/lists, answered 200"
        );
    }

    #[test]
    fn test_a_refusal_carries_its_reason_word_and_a_request_with_no_answer_says_so() {
        let refused = the_line(
            WhoWasAsked::Google,
            "PATCH",
            "https://www.googleapis.com/calendar/v3/calendars/primary/events/e1",
            WhatCameBack::Answered {
                status: 403,
                reason: Some("accessNotConfigured"),
            },
        );
        assert_eq!(
            refused,
            "Asked Google: PATCH www.googleapis.com/calendar/v3/calendars/primary/events/e1, \
             answered 403 accessNotConfigured"
        );
        let unanswered = the_line(
            WhoWasAsked::MicrosoftToDo,
            "DELETE",
            "https://graph.microsoft.com/v1.0/me/todo/lists/l1/tasks/t1",
            WhatCameBack::NoAnswer,
        );
        assert_eq!(
            unanswered,
            "Asked Microsoft To Do: DELETE graph.microsoft.com/v1.0/me/todo/lists/l1/tasks/t1, \
             no answer came"
        );
    }

    #[test]
    fn test_the_reason_word_is_googles_first_reason_and_else_its_status() {
        assert_eq!(
            the_reason_word(NOT_TURNED_ON).as_deref(),
            Some("accessNotConfigured")
        );
        let status_only = r#"{"error":{"code":401,"message":"Request had invalid authentication credentials.","status":"UNAUTHENTICATED"}}"#;
        assert_eq!(
            the_reason_word(status_only).as_deref(),
            Some("UNAUTHENTICATED")
        );
    }

    #[test]
    fn test_the_reason_word_refuses_anything_but_a_short_word() {
        // A message, an address or a token in the reason's place is not a
        // word, and the status beside it is taken instead.
        let a_sentence = r#"{"error":{"errors":[{"reason":"see pratik@gmail.com"}],"status":"FAILED_PRECONDITION"}}"#;
        assert_eq!(
            the_reason_word(a_sentence).as_deref(),
            Some("FAILED_PRECONDITION")
        );
        let too_long = format!(r#"{{"error":{{"status":"{}"}}}}"#, "A".repeat(65));
        assert_eq!(the_reason_word(&too_long), None);
        assert_eq!(the_reason_word("not json at all"), None);
        assert_eq!(the_reason_word(r#"{"items":[]}"#), None);
    }

    /// Every line written at info or above.
    fn the_info_lines(captured: &CapturedLogs) -> Vec<String> {
        captured
            .events()
            .into_iter()
            .filter(|(level, _)| *level <= tracing::Level::INFO)
            .map(|(_, line)| line)
            .collect()
    }

    /// The lines naming one provider, written at info or above.
    fn the_lines_naming(captured: &CapturedLogs, who: WhoWasAsked) -> Vec<String> {
        let asked = format!("Asked {}:", who.name());
        the_info_lines(captured)
            .into_iter()
            .filter(|line| line.starts_with(&asked))
            .collect()
    }

    #[tokio::test]
    async fn test_a_google_calendar_read_writes_one_line_and_nothing_private() {
        // `#[tokio::test]` runs on this one thread, so a default set here
        // reaches the client's line and no other test's.
        let captured = CapturedLogs::default();
        let _logging = tracing::subscriber::set_default(captured.clone());
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let client = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        client
            .list_events("a-token", None, None, Some("a-marker"), THE_MAIN_CALENDAR)
            .await
            .expect("an empty answer reads as no events");
        heard(listening, "the events")
            .await
            .expect("the stand-in was asked");

        let lines = the_lines_naming(&captured, WhoWasAsked::Google);
        assert_eq!(lines.len(), 1, "one request, one line: {lines:?}");
        let line = &lines[0];
        assert!(line.contains(": GET "), "{line}");
        assert!(line.contains("/calendars/primary/events,"), "{line}");
        assert!(line.ends_with("answered 200"), "{line}");
        for private in ["a-token", "a-marker", "?", "syncToken"] {
            assert!(
                !the_info_lines(&captured)
                    .iter()
                    .any(|line| line.contains(private)),
                "{private} reached the log: {:?}",
                the_info_lines(&captured)
            );
        }
    }

    #[tokio::test]
    async fn test_a_refused_google_read_writes_its_status_and_reason_and_nothing_else_of_the_body()
    {
        let captured = CapturedLogs::default();
        let _logging = tracing::subscriber::set_default(captured.clone());
        let (address, listening) = answering(
            "403 Forbidden",
            "application/json",
            NOT_TURNED_ON.to_string(),
        )
        .await;
        let client = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let refused = client
            .list_events("a-token", None, None, None, THE_MAIN_CALENDAR)
            .await;
        assert!(refused.is_err(), "a 403 read as events: {:?}", refused.ok());
        heard(listening, "the events")
            .await
            .expect("the stand-in was asked");

        let lines = the_lines_naming(&captured, WhoWasAsked::Google);
        assert_eq!(lines.len(), 1, "a refusal is not retried: {lines:?}");
        assert!(
            lines[0].ends_with("answered 403 accessNotConfigured"),
            "{}",
            lines[0]
        );
        for from_the_body in [
            "has not been used",
            "project 123",
            "usageLimits",
            "PERMISSION_DENIED",
        ] {
            assert!(
                !lines[0].contains(from_the_body),
                "{from_the_body} reached the log: {}",
                lines[0]
            );
        }
    }
}

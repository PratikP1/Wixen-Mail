//! A loopback server that answers exactly one request and hands it back.
//!
//! Tests that only check the converters beneath a provider client leave the
//! client itself unproven: what actually went out on the wire is decided by the
//! URL building and the headers, and neither is reachable from a unit test of a
//! parser. Standing up a real listener on `127.0.0.1:0` and pointing the client
//! at it is what reaches that, and reading the request back is what lets a test
//! assert on the bytes rather than on the answer.
//!
//! Test-only. It exists here rather than beside one caller because three places
//! had grown their own copy, one of which threw the request away.

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

/// Answer one request on a loopback port, and hand back what was asked.
///
/// `status` is a status line such as `"200 OK"` or `"207 Multi-Status"`.
///
/// Serves a single connection and then stops, so a client that pages will get
/// no answer to its second request. Keep the reply free of paging fields unless
/// the test is about paging.
///
/// Wait on the receiver with a timeout. If the request is never made, the
/// listener never accepts, the channel never fires, and nothing is left to wake
/// the runtime, so an unbounded wait is a hung run rather than a failure
/// somebody can read.
pub async fn answering(
    status: &'static str,
    content_type: &'static str,
    reply: String,
) -> (std::net::SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    answering_one(status, content_type, None, reply).await
}

/// Answer one request with a version tag on the reply, and hand back what was
/// asked.
///
/// A calendar server names the version of a document with an `ETag`, and a
/// change that has to match one can only be tested against a reply that carries
/// one. Everything else behaves as [`answering`] does.
pub async fn answering_with_a_tag(
    status: &'static str,
    content_type: &'static str,
    tag: &'static str,
    reply: String,
) -> (std::net::SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    answering_one(status, content_type, Some(tag), reply).await
}

async fn answering_one(
    status: &'static str,
    content_type: &'static str,
    tag: Option<&'static str>,
    reply: String,
) -> (std::net::SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let address = listener.local_addr().expect("the port that was taken");
    let (asked, heard) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let Ok((mut stream, _)) = listener.accept().await else {
            return;
        };
        let request = read_request(&mut stream).await;
        let _ = stream
            .write_all(head(status, content_type, tag, reply.len()).as_bytes())
            .await;
        let _ = stream.write_all(reply.as_bytes()).await;
        let _ = stream.shutdown().await;
        let _ = asked.send(request);
    });

    (address, heard)
}

/// The head of a canned reply.
fn head(status: &str, content_type: &str, tag: Option<&str>, length: usize) -> String {
    let named_version = tag
        .map(|tag| format!("ETag: {tag}\r\n"))
        .unwrap_or_default();
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n{named_version}\
         Content-Length: {length}\r\nConnection: close\r\n\r\n"
    )
}

/// Answer several requests on a loopback port, and hand back all of them in the
/// order they arrived.
///
/// One connection per reply, so a test that expects a change to be sent before
/// the calendar is read can assert on the order rather than on one request in
/// isolation.
///
/// Nothing is handed back until the last reply has been served, so a run that
/// makes fewer requests than there are replies reports a missing request rather
/// than a short list that looks like success. Wait on the receiver with a
/// timeout for the same reason [`answering`] says to.
pub async fn answering_several(
    status: &'static str,
    content_type: &'static str,
    replies: Vec<String>,
) -> (
    std::net::SocketAddr,
    tokio::sync::oneshot::Receiver<Vec<String>>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let address = listener.local_addr().expect("the port that was taken");
    let (asked, heard) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let mut requests = Vec::with_capacity(replies.len());
        for reply in replies {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            requests.push(read_request(&mut stream).await);
            let _ = stream
                .write_all(head(status, content_type, None, reply.len()).as_bytes())
                .await;
            let _ = stream.write_all(reply.as_bytes()).await;
            let _ = stream.shutdown().await;
        }
        let _ = asked.send(requests);
    });

    (address, heard)
}

/// One canned answer in a sequence.
///
/// Separate from [`answering_several`] because a sequence that reads a document
/// and then changes it needs the read to carry a version tag and the change
/// not to, and a list of bare strings cannot say which is which.
pub struct Answer {
    tag: Option<&'static str>,
    body: String,
}

impl Answer {
    /// An answer with no version tag on it, which is what most servers give.
    pub fn plain(body: String) -> Self {
        Self { tag: None, body }
    }

    /// An answer naming the version of what it carries.
    pub fn tagged(tag: &'static str, body: String) -> Self {
        Self {
            tag: Some(tag),
            body,
        }
    }
}

/// Answer several requests in order, each with its own version tag.
///
/// Otherwise exactly [`answering_several`]: one connection per answer, nothing
/// handed back until the last has been served, so a run that makes fewer
/// requests than there are answers reports a missing request rather than a
/// short list that looks like success.
pub async fn answering_in_turn(
    status: &'static str,
    content_type: &'static str,
    answers: Vec<Answer>,
) -> (
    std::net::SocketAddr,
    tokio::sync::oneshot::Receiver<Vec<String>>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let address = listener.local_addr().expect("the port that was taken");
    let (asked, heard) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let mut requests = Vec::with_capacity(answers.len());
        for answer in answers {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            requests.push(read_request(&mut stream).await);
            let _ = stream
                .write_all(head(status, content_type, answer.tag, answer.body.len()).as_bytes())
                .await;
            let _ = stream.write_all(answer.body.as_bytes()).await;
            let _ = stream.shutdown().await;
        }
        let _ = asked.send(requests);
    });

    (address, heard)
}

/// How long a test waits for the request it expects.
///
/// Long enough that a loaded machine does not fail a passing test, short enough
/// that a client which never sent anything reports a failure rather than hanging
/// the run.
pub const LONG_ENOUGH: std::time::Duration = std::time::Duration::from_secs(10);

/// What a loopback server heard, or a failure naming what was expected.
///
/// Written once for one request and for several, because the discipline is the
/// same either way: a wait with no timeout on a request that never came is a
/// hung run rather than a failure somebody can read.
pub async fn heard<T>(
    receiver: tokio::sync::oneshot::Receiver<T>,
    expected: &str,
) -> Result<T, String> {
    match tokio::time::timeout(LONG_ENOUGH, receiver).await {
        Ok(Ok(request)) => Ok(request),
        Ok(Err(_)) => Err(format!(
            "the server stopped before {expected} was asked for"
        )),
        Err(_) => Err(format!("nothing asked for {expected}")),
    }
}

/// The request line of a captured request, without the trailing protocol.
///
/// `GET /people/me/connections?x=1 HTTP/1.1` becomes
/// `GET /people/me/connections?x=1`, which is what an assertion wants to read.
pub fn asked_for(request: &str) -> &str {
    let first = request.lines().next().unwrap_or_default();
    first.strip_suffix(" HTTP/1.1").unwrap_or(first)
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut raw = Vec::new();
    let mut chunk = [0_u8; 2048];
    while let Ok(read) = stream.read(&mut chunk).await {
        if read == 0 {
            break;
        }
        raw.extend_from_slice(&chunk[..read]);
        let so_far = String::from_utf8_lossy(&raw).into_owned();
        let Some(head_end) = so_far.find("\r\n\r\n") else {
            continue;
        };
        if raw.len() >= head_end + 4 + content_length(&so_far[..head_end]) {
            break;
        }
    }
    String::from_utf8_lossy(&raw).into_owned()
}

fn content_length(head: &str) -> usize {
    for line in head.lines() {
        let lowered = line.to_ascii_lowercase();
        if let Some(value) = lowered.strip_prefix("content-length:") {
            return value.trim().parse().unwrap_or(0);
        }
    }
    0
}

// ── A line protocol needs a different server from an HTTP one ───────────────
//
// Everything above answers one request with one canned reply, because that is
// what HTTP is. Mail is not: the server speaks first, the client's next line
// depends on the answer to the last one, and a saved message arrives as a
// counted block of bytes rather than as lines. So this half reads a line at a
// time, answers from a script, and keeps every line it was told.

/// What the server does with one line it was sent.
pub enum Turn {
    /// Answer with these bytes, verbatim, and go back to reading lines.
    Say(String),
    /// Take the counted block of bytes the line just read asked to send.
    ///
    /// How a saved copy of a message reaches a mail server. Answers `+`, reads
    /// the number of bytes the line named at its end and the line ending after
    /// them, records those bytes as one entry of their own, then answers
    /// `done`.
    TakingALiteral { done: String },
    /// Take a message body, which ends at a line holding a single dot.
    ///
    /// How a message reaches a sending server. Answers `354`, reads until the
    /// dot, records the body as one entry of its own, then answers `done`. A
    /// line the client started with a second dot to keep it from ending the
    /// body has that dot taken off again, which is what a real server does.
    TakingAMessage { done: String },
}

/// A loopback server that holds a line-by-line conversation and remembers it.
///
/// Serves every connection offered rather than one, because filing a copy of a
/// sent message opens a session of its own, and a helper that stopped after
/// the first would leave the second waiting for a greeting.
pub struct Conversation {
    address: std::net::SocketAddr,
    heard: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl Conversation {
    /// Where the server is listening.
    pub const fn address(&self) -> std::net::SocketAddr {
        self.address
    }

    /// The host a client should be pointed at.
    pub fn server(&self) -> String {
        self.address.ip().to_string()
    }

    /// The port a client should be pointed at.
    pub const fn port(&self) -> u16 {
        self.address.port()
    }

    /// Everything said to the server, in order.
    pub async fn transcript(&self) -> Vec<String> {
        self.heard.lock().await.clone()
    }

    /// Where the first line holding this text is, matched without case.
    ///
    /// Position rather than presence, because most of the questions here are
    /// about order: whether the copy was made before the original was flagged
    /// decides whether a failure loses somebody's only copy of a message.
    pub async fn when_told(&self, wanted: &str) -> Option<usize> {
        let wanted = wanted.to_uppercase();
        self.transcript()
            .await
            .iter()
            .position(|line| line.to_uppercase().contains(&wanted))
    }

    /// Whether anything at all was said holding this text.
    pub async fn was_told(&self, wanted: &str) -> bool {
        self.when_told(wanted).await.is_some()
    }
}

/// Take a loopback port and hold a conversation from the script given.
///
/// The script sees each line without its ending and decides what happens next.
/// It may hold state, so a second copy can be refused where the first
/// succeeded. Give it an arm that answers something for a line it does not
/// know: a command left unanswered is a client waiting out its own timeout,
/// which reads as a slow machine rather than as a wrong script.
pub async fn conversing(
    greeting: &'static str,
    answer: impl Fn(&str) -> Turn + Send + Sync + 'static,
) -> Conversation {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let address = listener.local_addr().expect("the port that was taken");
    let heard = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let recording = heard.clone();
    let script = std::sync::Arc::new(answer);

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let recording = recording.clone();
            let script = script.clone();
            tokio::spawn(async move {
                hold_one(stream, greeting, &recording, script.as_ref()).await;
            });
        }
    });

    Conversation { address, heard }
}

/// One connection, held until the client stops talking.
async fn hold_one(
    stream: tokio::net::TcpStream,
    greeting: &str,
    recording: &tokio::sync::Mutex<Vec<String>>,
    answer: &(impl Fn(&str) -> Turn + ?Sized),
) {
    let (reading, mut writing) = stream.into_split();
    if writing.write_all(greeting.as_bytes()).await.is_err() {
        return;
    }
    let mut reader = tokio::io::BufReader::new(reading);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let said = line.trim_end_matches(['\r', '\n']).to_string();
        // Recorded before the answer goes out, so a client that has heard its
        // answer can rely on the line already being in the transcript.
        record(recording, &said).await;

        let reply = match answer(&said) {
            Turn::Say(reply) => reply,
            Turn::TakingALiteral { done } => {
                if writing.write_all(b"+ go ahead\r\n").await.is_err() {
                    return;
                }
                let Some(count) = literal_length(&said) else {
                    return;
                };
                let mut raw = vec![0_u8; count];
                if reader.read_exact(&mut raw).await.is_err() {
                    return;
                }
                record(recording, &String::from_utf8_lossy(&raw)).await;
                // The line ending the client writes after the counted bytes.
                let mut ending = String::new();
                let _ = reader.read_line(&mut ending).await;
                done
            }
            Turn::TakingAMessage { done } => {
                if writing.write_all(b"354 go ahead\r\n").await.is_err() {
                    return;
                }
                let mut body: Vec<String> = Vec::new();
                loop {
                    let mut part = String::new();
                    match reader.read_line(&mut part).await {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {}
                    }
                    let part = part.trim_end_matches(['\r', '\n']).to_string();
                    if part == "." {
                        break;
                    }
                    // A line the client started with a second dot so it could
                    // not end the body early.
                    body.push(
                        part.strip_prefix("..")
                            .map_or_else(|| part.clone(), |rest| format!(".{rest}")),
                    );
                }
                record(recording, &body.join("\r\n")).await;
                done
            }
        };
        if writing.write_all(reply.as_bytes()).await.is_err() {
            return;
        }
    }
}

async fn record(recording: &tokio::sync::Mutex<Vec<String>>, said: &str) {
    recording.lock().await.push(without_the_credential(said));
}

/// How many bytes the counted block announced on this line holds.
///
/// `A1 APPEND "Sent" (\Seen) {13}` says thirteen. A count with a `+` on it
/// says the client is not waiting to be told to go ahead, and says the same
/// number.
fn literal_length(line: &str) -> Option<usize> {
    let opened = line.rfind('{')?;
    let closed = line[opened..].find('}')? + opened;
    line[opened + 1..closed].trim_end_matches('+').parse().ok()
}

/// One command with anything credential-shaped taken off the end.
///
/// The transcript is read by the assertions and printed when one fails, so it
/// is held to the same rule as a log file: a password does not go in it. Three
/// protocols each spell the line their own way, and the encoded form an SMTP
/// sign-in uses looks like nothing in particular, so it is matched by where it
/// sits rather than by what it looks like.
fn without_the_credential(line: &str) -> String {
    let mut words = line.split_whitespace();
    let first = words.next().unwrap_or_default();
    let second = words.next().unwrap_or_default();
    if first.eq_ignore_ascii_case("PASS") {
        return first.to_string();
    }
    if first.eq_ignore_ascii_case("AUTH") {
        return format!("{first} {second}").trim_end().to_string();
    }
    match second.to_uppercase().as_str() {
        "LOGIN" | "AUTHENTICATE" => format!("{first} {second}"),
        _ => line.to_string(),
    }
}

#[cfg(test)]
mod line_protocol_tests {
    use super::*;

    /// A client socket that can be driven a line at a time.
    struct Caller {
        reader: tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>,
        writer: tokio::net::tcp::OwnedWriteHalf,
    }

    impl Caller {
        async fn calling(server: &Conversation) -> Self {
            let stream = tokio::net::TcpStream::connect(server.address())
                .await
                .expect("the loopback port to be open");
            let (reading, writer) = stream.into_split();
            Self {
                reader: tokio::io::BufReader::new(reading),
                writer,
            }
        }

        async fn hears(&mut self) -> String {
            let mut line = String::new();
            tokio::time::timeout(LONG_ENOUGH, self.reader.read_line(&mut line))
                .await
                .expect("the server to answer within the wait")
                .expect("a line from the server");
            line
        }

        async fn says(&mut self, text: &str) {
            self.writer
                .write_all(text.as_bytes())
                .await
                .expect("the server to still be listening");
        }
    }

    /// A server that answers every line the same way.
    async fn agreeing() -> Conversation {
        conversing("* OK ready\r\n", |_| {
            Turn::Say("A1 OK done\r\n".to_string())
        })
        .await
    }

    #[tokio::test]
    async fn test_a_conversation_hands_back_every_line_in_the_order_it_arrived() {
        // The whole point of a line protocol helper: what the client said, in
        // the order it said it. A transcript that only holds the last line
        // cannot tell a copy made before a flag from one made after.
        let server = agreeing().await;
        let mut caller = Caller::calling(&server).await;
        assert_eq!(caller.hears().await, "* OK ready\r\n");

        caller.says("A1 FIRST\r\n").await;
        caller.hears().await;
        caller.says("A2 SECOND\r\n").await;
        caller.hears().await;

        assert_eq!(server.transcript().await, vec!["A1 FIRST", "A2 SECOND"]);
    }

    #[tokio::test]
    async fn test_a_line_that_never_came_is_not_read_as_one_that_did() {
        // The mirror of the wait test above. Before believing an empty
        // transcript, the transcript has to be able to hold something, and
        // before believing a match, a miss has to be able to miss.
        let server = agreeing().await;
        let mut caller = Caller::calling(&server).await;
        caller.hears().await;

        caller.says("A1 EXPUNGE\r\n").await;
        caller.hears().await;

        assert_eq!(server.when_told("EXPUNGE").await, Some(0));
        assert_eq!(server.when_told("APPEND").await, None);
        assert!(
            server.was_told("expunge").await,
            "matched with the wrong case"
        );
        assert!(!server.was_told("APPEND").await);
    }

    #[tokio::test]
    async fn test_a_literal_is_taken_whole_rather_than_read_as_lines() {
        // A saved copy of a sent message is an IMAP literal: a byte count and
        // then that many bytes, newlines and all. A server reading it as lines
        // splits the message at the first blank line, which is the boundary
        // between its headers and everything somebody wrote.
        let server = conversing("* OK ready\r\n", |line| {
            if line.to_uppercase().contains("APPEND") {
                return Turn::TakingALiteral {
                    done: "A1 OK saved\r\n".to_string(),
                };
            }
            Turn::Say("A1 OK done\r\n".to_string())
        })
        .await;
        let mut caller = Caller::calling(&server).await;
        caller.hears().await;

        caller.says("A1 APPEND \"Sent\" (\\Seen) {13}\r\n").await;
        assert!(caller.hears().await.starts_with('+'));
        caller.says("line one\r\ntwo\r\n").await;

        assert_eq!(caller.hears().await, "A1 OK saved\r\n");
        assert_eq!(
            server.transcript().await,
            vec!["A1 APPEND \"Sent\" (\\Seen) {13}", "line one\r\ntwo"]
        );
    }

    #[tokio::test]
    async fn test_a_message_body_ending_in_a_dot_line_is_taken_whole() {
        // An SMTP body is not counted, it is ended by a line holding one dot,
        // and a line of the message that starts with a dot is sent with a
        // second one in front of it. A server that does not take that off
        // records a body nobody wrote.
        let server = conversing("220 ready\r\n", |line| {
            if line.eq_ignore_ascii_case("DATA") {
                return Turn::TakingAMessage {
                    done: "250 queued\r\n".to_string(),
                };
            }
            Turn::Say("250 ok\r\n".to_string())
        })
        .await;
        let mut caller = Caller::calling(&server).await;
        caller.hears().await;

        caller.says("DATA\r\n").await;
        assert!(caller.hears().await.starts_with("354"));
        caller
            .says("Subject: Hi\r\n\r\n..hidden\r\nbye\r\n.\r\n")
            .await;

        assert_eq!(caller.hears().await, "250 queued\r\n");
        assert_eq!(
            server.transcript().await,
            vec!["DATA", "Subject: Hi\r\n\r\n.hidden\r\nbye"]
        );
    }

    #[tokio::test]
    async fn test_a_password_does_not_reach_the_transcript() {
        // The transcript is printed by every failing assertion in these tests,
        // so it is held to the same rule as a log file. Three protocols carry
        // a credential on a line of their own and each spells it differently.
        let server = agreeing().await;
        let mut caller = Caller::calling(&server).await;
        caller.hears().await;

        for said in [
            "A1 LOGIN \"me\" \"hunter2\"\r\n",
            "AUTH PLAIN AGFkYQBodW50ZXIy\r\n",
            "PASS hunter2\r\n",
        ] {
            caller.says(said).await;
            caller.hears().await;
        }

        let transcript = server.transcript().await;
        for line in &transcript {
            assert!(!line.contains("hunter2"), "a password was recorded: {line}");
            assert!(
                !line.contains("AGFkYQBodW50ZXIy"),
                "an encoded credential was recorded: {line}"
            );
        }
        assert!(transcript[0].contains("LOGIN"), "{transcript:?}");
        assert!(transcript[1].contains("AUTH"), "{transcript:?}");
        assert!(transcript[2].contains("PASS"), "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_second_connection_is_served_too() {
        // Filing a copy of a sent message opens its own session, so a test
        // that covers sending and then filing needs two. A server that stops
        // after the first leaves the second waiting for a greeting that never
        // comes, which reads as a slow machine.
        let server = agreeing().await;

        let mut first = Caller::calling(&server).await;
        assert_eq!(first.hears().await, "* OK ready\r\n");
        first.says("A1 FIRST\r\n").await;
        first.hears().await;
        drop(first);

        let mut second = Caller::calling(&server).await;
        assert_eq!(second.hears().await, "* OK ready\r\n");
        second.says("A2 SECOND\r\n").await;
        second.hears().await;

        assert!(
            server.was_told("A1 FIRST").await,
            "{:?}",
            server.transcript().await
        );
        assert!(
            server.was_told("A2 SECOND").await,
            "{:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_the_length_of_a_literal_is_read_off_the_line_that_announced_it() {
        // The count is the only thing that says where a literal ends, and it
        // arrives on the end of the command line. Reading it wrong reads the
        // next command as part of the message.
        assert_eq!(literal_length("A1 APPEND \"Sent\" (\\Seen) {13}"), Some(13));
        assert_eq!(literal_length("A1 APPEND \"Sent\" {2048+}"), Some(2048));
        assert_eq!(literal_length("A1 NOOP"), None);
    }

    #[tokio::test]
    async fn test_a_conversation_that_nobody_calls_reports_an_empty_transcript() {
        // Not a hang. A test whose client never connected has to be able to
        // say so rather than wait for the whole run's timeout.
        let server = agreeing().await;

        assert!(server.transcript().await.is_empty());
        assert_eq!(server.when_told("ANYTHING").await, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_request_line_is_read_without_its_protocol() {
        let request = "GET /me/contacts?x=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";

        assert_eq!(asked_for(request), "GET /me/contacts?x=1");
    }

    #[tokio::test]
    async fn test_several_requests_come_back_in_the_order_they_were_made() {
        // A push that sends a change and then reads the calendar has to be
        // provable in that order: the other way round sends a value the read
        // has just overwritten, and one request captured on its own cannot
        // tell the two apart.
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await;
        let client = reqwest::Client::new();

        client
            .post(format!("http://{address}/first"))
            .send()
            .await
            .expect("the first to be answered");
        client
            .get(format!("http://{address}/second"))
            .send()
            .await
            .expect("the second to be answered");

        let requests = heard(listening, "two requests").await.expect("both");
        assert_eq!(requests.len(), 2);
        assert_eq!(asked_for(&requests[0]), "POST /first");
        assert_eq!(asked_for(&requests[1]), "GET /second");
    }

    #[tokio::test]
    async fn test_fewer_requests_than_replies_is_a_failure_rather_than_a_hang() {
        // The same discipline `heard` already has. A test that expected two
        // requests and got one has to be able to say so.
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await;
        reqwest::Client::new()
            .get(format!("http://{address}/only-one"))
            .send()
            .await
            .expect("the first to be answered");

        let waited = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            heard(listening, "two requests when only one was made"),
        )
        .await;

        assert!(waited.is_err(), "a missing request was reported as arrived");
    }

    #[tokio::test]
    async fn test_a_reply_can_carry_the_tag_a_change_has_to_match() {
        // The whole concurrency story rests on a tag coming off a reply, and
        // the head written here carries three fixed headers and nothing else.
        // Proving the header really arrives comes before anything reads one.
        let (address, _listening) = answering_with_a_tag(
            "200 OK",
            "text/calendar",
            "\"v7\"",
            "BEGIN:VCALENDAR\r\nEND:VCALENDAR".to_string(),
        )
        .await;

        let answer = reqwest::Client::new()
            .get(format!("http://{address}/e-1.ics"))
            .send()
            .await
            .expect("the reply");

        assert_eq!(
            answer.headers().get("ETag").and_then(|v| v.to_str().ok()),
            Some("\"v7\"")
        );
    }

    #[tokio::test]
    async fn test_a_request_that_never_came_is_a_failure_rather_than_a_hang() {
        // Before believing a captured request, the capture has to be able to
        // report that nothing arrived. A helper that hangs instead would make
        // every assertion below it worthless.
        let (_address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;

        let waited = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            heard(listening, "a request nobody makes"),
        )
        .await;

        assert!(
            waited.is_err(),
            "the wait ended on its own, so it was not waiting for the request"
        );
    }
}

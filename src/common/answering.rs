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

use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        let head = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n",
            reply.len()
        );
        let _ = stream.write_all(head.as_bytes()).await;
        let _ = stream.write_all(reply.as_bytes()).await;
        let _ = stream.shutdown().await;
        let _ = asked.send(request);
    });

    (address, heard)
}

/// How long a test waits for the request it expects.
///
/// Long enough that a loaded machine does not fail a passing test, short enough
/// that a client which never sent anything reports a failure rather than hanging
/// the run.
pub const LONG_ENOUGH: std::time::Duration = std::time::Duration::from_secs(10);

/// The request a loopback server heard, or a failure naming what was expected.
pub async fn heard(
    receiver: tokio::sync::oneshot::Receiver<String>,
    expected: &str,
) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_request_line_is_read_without_its_protocol() {
        let request = "GET /me/contacts?x=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";

        assert_eq!(asked_for(request), "GET /me/contacts?x=1");
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

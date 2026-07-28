//! Turning a link into the things Safe Browsing is asked about.
//!
//! Google's threat lists are not lists of URLs. They are lists of SHA-256
//! prefixes of *expressions*: a URL reduced to a canonical form and then
//! broken into the handful of host and path combinations a listed site could
//! have been listed under. Asking about one link means checking up to thirty
//! of them against a database held on this machine.
//!
//! That local database is the whole reason this is acceptable here. Nothing is
//! sent to Google unless one of those thirty hashes collides with a prefix on
//! the list, which for ordinary correspondence is never, and even then only
//! four bytes go. The Lookup API, which posts the actual URL, is not used and
//! must not be: it would hand a third party the links out of private mail.
//!
//! This module is the pure half, and it is pure on purpose. Canonicalisation is
//! where the evasions live: a percent-encoded percent sign, an integer-form IP
//! address, a path full of dot segments. Every one of those is a way to make a
//! link that a person reads as one site and a naive matcher reads as another,
//! so the rules are Google's exactly and the tests are Google's own vectors.

use std::fmt::Write as _;

/// The most host suffixes Safe Browsing generates for one URL.
const MAX_HOSTS: usize = 5;
/// The most path prefixes, including the two exact forms.
const MAX_PATHS: usize = 6;

/// A URL reduced to the form Safe Browsing hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canonical {
    pub scheme: String,
    pub host: String,
    pub path: String,
    /// The query as written, without the `?`, when there was one.
    pub query: Option<String>,
}

impl Canonical {
    /// The URL as one string again.
    pub fn to_url(&self) -> String {
        let mut url = format!("{}://{}{}", self.scheme, self.host, self.path);
        if let Some(query) = &self.query {
            let _ = write!(url, "?{query}");
        }
        url
    }
}

/// Reduce a URL to the canonical form Safe Browsing defines.
///
/// `None` for anything without a host, which includes the `mailto:` and
/// `javascript:` links a message body is full of.
pub fn canonicalise(raw: &str) -> Option<Canonical> {
    // Tabs, carriage returns and line feeds are stripped wherever they appear,
    // which is the first evasion: a URL split across lines reads as one thing
    // and resolves as another.
    let cleaned: String = raw
        .chars()
        .filter(|c| !matches!(c, '\t' | '\r' | '\n'))
        .collect();
    let cleaned = cleaned.trim();

    // The fragment is not sent to a server, so it is not part of what is
    // being asked about.
    let without_fragment = cleaned.split('#').next().unwrap_or("");

    let (scheme, rest) = match without_fragment.split_once("://") {
        Some((scheme, rest)) => (scheme.to_lowercase(), rest),
        // A scheme with no slashes, such as mailto: or javascript:, is not a
        // web address and has no host to ask about. Treating it as a bare
        // domain read "mailto:someone@example.com" as a link to example.com,
        // which would have sent a question to Google derived from somebody's
        // email address.
        None if has_other_scheme(without_fragment) => return None,
        // A bare "example.com/path" in a message body is a link somebody will
        // click, and http is what their browser will assume.
        None => ("http".to_string(), without_fragment),
    };
    if !matches!(scheme.as_str(), "http" | "https") {
        return None;
    }

    let (authority, path_and_query) = match rest.find(['/', '?']) {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, ""),
    };
    // Credentials before an @ are a classic disguise: the part a reader sees
    // first is not the host.
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    // A port does not change which site it is.
    let host_only = match authority.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) => host,
        _ => authority,
    };

    let host = canonical_host(host_only)?;
    let (path, query) = match path_and_query.split_once('?') {
        Some((path, query)) => (path, Some(query.to_string())),
        None => (path_and_query, None),
    };

    Some(Canonical {
        scheme,
        host,
        path: canonical_path(path),
        query,
    })
}

/// Whether this starts with a scheme that is not a web address.
///
/// A scheme name has no dots in it, which is what separates `mailto:` from a
/// host and port like `example.com:8080`.
fn has_other_scheme(value: &str) -> bool {
    let before_path = value.split('/').next().unwrap_or(value);
    match before_path.split_once(':') {
        Some((scheme, _)) => {
            !scheme.is_empty()
                && !scheme.contains('.')
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-'))
                && scheme
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic())
        }
        None => false,
    }
}

/// Percent-unescape until it stops changing, which is the rule as written.
///
/// `%25%32%35` unescapes to `%25` and then to `%`. Stopping after one pass is
/// how a matcher sees a different path from the one the browser will fetch.
fn unescape_repeatedly(value: &str) -> String {
    let mut current = value.to_string();
    for _ in 0..64 {
        let next = unescape_once(&current);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

fn unescape_once(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Escape what has to be escaped, and nothing else.
fn escape(value: &str, extra: &[u8]) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte <= 0x20 || byte >= 0x7F || byte == b'%' || byte == b'#' || extra.contains(&byte) {
            let _ = write!(out, "%{byte:02X}");
        } else {
            out.push(byte as char);
        }
    }
    out
}

fn canonical_host(host: &str) -> Option<String> {
    let host = unescape_repeatedly(host).to_lowercase();
    // Leading, trailing and repeated dots all resolve to the same host.
    let host: String = host
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(".");
    if host.is_empty() {
        return None;
    }
    Some(escape(&normalise_ip(&host), &[]))
}

/// Bring every spelling of an IP address to dotted decimal.
///
/// `http://3279880203/` and `http://195.127.0.11/` are the same server, and a
/// matcher that treats them as different sites is one an attacker chooses.
fn normalise_ip(host: &str) -> String {
    if let Ok(as_integer) = host.parse::<u32>() {
        return format!(
            "{}.{}.{}.{}",
            as_integer >> 24,
            (as_integer >> 16) & 0xFF,
            (as_integer >> 8) & 0xFF,
            as_integer & 0xFF
        );
    }
    // Dotted forms with leading zeroes, such as 192.016.001.1, are octal to a
    // resolver and decimal to a careless matcher.
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        let decoded: Option<Vec<u32>> = parts
            .iter()
            .map(|p| {
                if p.len() > 1 && p.starts_with('0') {
                    u32::from_str_radix(p, 8).ok()
                } else {
                    p.parse().ok()
                }
            })
            .collect();
        if let Some(octets) = decoded
            && octets.iter().all(|o| *o <= 255)
        {
            return octets
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(".");
        }
    }
    host.to_string()
}

fn canonical_path(path: &str) -> String {
    let path = unescape_repeatedly(path);
    let path = if path.is_empty() { "/" } else { &path };

    let mut resolved: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            // A run of slashes, or a "here" segment, changes nothing.
            "" | "." => {}
            ".." => {
                resolved.pop();
            }
            other => resolved.push(other),
        }
    }
    let trailing = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    let mut joined = String::from("/");
    joined.push_str(&resolved.join("/"));
    if trailing && !joined.ends_with('/') {
        joined.push('/');
    }
    escape(&joined, &[])
}

/// Every host and path combination this URL could have been listed under.
///
/// Up to thirty, and all of them checked against the local database before
/// anything is sent anywhere.
pub fn expressions(url: &Canonical) -> Vec<String> {
    let mut out = Vec::new();
    for host in host_suffixes(&url.host) {
        for path in path_prefixes(&url.path, url.query.as_deref()) {
            out.push(format!("{host}{path}"));
        }
    }
    out.dedup();
    out
}

/// The exact host, then up to four parent domains.
///
/// An IP address has no parents: 195.127.0.11 does not belong to 127.0.11.
fn host_suffixes(host: &str) -> Vec<String> {
    if host
        .split('.')
        .all(|p| p.chars().all(|c| c.is_ascii_digit()))
    {
        return vec![host.to_string()];
    }

    let parts: Vec<&str> = host.split('.').collect();
    let mut suffixes = vec![host.to_string()];
    // Google takes the last five components, then works up to two.
    let start = parts.len().saturating_sub(MAX_HOSTS);
    for cut in (start + 1)..parts.len().saturating_sub(1) {
        suffixes.push(parts[cut..].join("."));
    }
    suffixes.dedup();
    suffixes
}

/// The exact path with and without its query, then up to four prefixes.
fn path_prefixes(path: &str, query: Option<&str>) -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(query) = query {
        paths.push(format!("{path}?{query}"));
    }
    paths.push(path.to_string());

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    paths.push("/".to_string());
    for depth in 1..segments.len() {
        if paths.len() >= MAX_PATHS {
            break;
        }
        paths.push(format!("/{}/", segments[..depth].join("/")));
    }

    paths.truncate(MAX_PATHS);
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_url(raw: &str) -> String {
        canonicalise(raw).expect("should canonicalise").to_url()
    }

    #[test]
    fn test_googles_own_canonicalisation_examples() {
        // Straight from the Safe Browsing specification. These are the cases
        // that decide whether a matcher sees the site a browser will fetch.
        for (raw, expected) in [
            ("http://host/%25%32%35", "http://host/%25"),
            ("http://host/%25%32%35%25%32%35", "http://host/%25%25"),
            ("http://host/asdf%25%32%35asd", "http://host/asdf%25asd"),
            ("http://www.google.com/", "http://www.google.com/"),
            ("http://www.google.com/blah/..", "http://www.google.com/"),
            ("http://www.evil.com/blah#frag", "http://www.evil.com/blah"),
            ("http://www.GOOgle.com/", "http://www.google.com/"),
            ("http://www.google.com.../", "http://www.google.com/"),
            (
                "http://www.google.com/foo/./bar",
                "http://www.google.com/foo/bar",
            ),
            ("http://3279880203/blah", "http://195.127.0.11/blah"),
        ] {
            assert_eq!(canonical_url(raw), expected, "for {raw}");
        }
    }

    #[test]
    fn test_a_percent_sign_is_unescaped_until_it_stops_changing() {
        // One pass leaves %25, which is a different path from the one the
        // browser fetches, which is the whole trick.
        assert_eq!(unescape_repeatedly("%25%32%35"), "%");
    }

    #[test]
    fn test_credentials_before_the_host_do_not_become_the_host() {
        // "http://www.paypal.com@evil.example/" reads as PayPal and goes to
        // evil.example, which is the oldest trick there is.
        let url = canonicalise("http://www.paypal.com@evil.example/").unwrap();

        assert_eq!(url.host, "evil.example");
    }

    #[test]
    fn test_a_port_does_not_make_it_a_different_site() {
        assert_eq!(
            canonicalise("http://evil.example:8080/x").unwrap().host,
            "evil.example"
        );
    }

    #[test]
    fn test_an_octal_address_is_the_same_address() {
        assert_eq!(
            canonicalise("http://0300.0250.0.1/").unwrap().host,
            "192.168.0.1"
        );
    }

    #[test]
    fn test_a_link_with_no_host_is_not_something_to_ask_about() {
        // Message bodies are full of these and none of them is a web page.
        for raw in [
            "mailto:someone@example.com",
            "javascript:alert(1)",
            "tel:+441234",
        ] {
            assert_eq!(canonicalise(raw), None, "for {raw}");
        }
    }

    #[test]
    fn test_a_url_split_across_lines_is_put_back_together() {
        // Whitespace inside a URL is stripped, so a link wrapped by a mail
        // client still resolves to the site it resolves to.
        let url = canonicalise("http://evil.\nexample/pa\tth").unwrap();

        assert_eq!(url.host, "evil.example");
        assert_eq!(url.path, "/path");
    }

    #[test]
    fn test_googles_own_expression_example() {
        // The worked example from the specification: for
        // http://a.b.c/1/2.html?param=1 the client checks these four.
        let url = canonicalise("http://a.b.c/1/2.html?param=1").unwrap();

        let found = expressions(&url);

        for expected in [
            "a.b.c/1/2.html?param=1",
            "a.b.c/1/2.html",
            "a.b.c/1/",
            "a.b.c/",
            "b.c/1/2.html?param=1",
            "b.c/",
        ] {
            assert!(found.contains(&expected.to_string()), "missing {expected}");
        }
    }

    #[test]
    fn test_an_address_has_no_parent_domains() {
        // 195.127.0.11 does not belong to 127.0.11, and generating that would
        // ask about a site nobody mentioned.
        let url = canonicalise("http://195.127.0.11/path").unwrap();

        for expression in expressions(&url) {
            assert!(
                expression.starts_with("195.127.0.11/"),
                "invented a parent: {expression}"
            );
        }
    }

    #[test]
    fn test_the_number_of_questions_stays_bounded() {
        // Thirty is the ceiling the specification sets, and every one of them
        // is a hash computed for every link in every message opened.
        let url = canonicalise("http://a.b.c.d.e.f.g/1/2/3/4/5/6.html?x=1").unwrap();

        assert!(expressions(&url).len() <= 30, "{}", expressions(&url).len());
    }

    #[test]
    fn test_the_same_question_is_not_asked_twice() {
        let url = canonicalise("http://example.com/").unwrap();

        let found = expressions(&url);
        let mut unique = found.clone();
        unique.sort();
        unique.dedup();

        assert_eq!(found.len(), unique.len(), "{found:?}");
    }
}

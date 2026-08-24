//! Google Safe Browsing, through the API that does not send it the links.
//!
//! There are two ways to use Safe Browsing and only one of them belongs in a
//! mail client.
//!
//! The **Lookup API** posts the URL and gets a verdict back. It is a few lines
//! of code and it hands a third party every link in your private correspondence,
//! which is a thing to be asked about rather than a thing to be done. It is not
//! used here and must not be.
//!
//! The **Update API** keeps a copy of Google's threat lists on this machine, as
//! 32-bit prefixes of the SHA-256 of listed URLs. A link is canonicalised and
//! hashed here, and compared here. Nothing is sent unless one of those hashes
//! collides with a prefix on the local list, and then only the four colliding
//! bytes go, which stand for millions of possible URLs. Google answers with
//! every full hash sharing those four bytes and the comparison happens back on
//! this machine.
//!
//! So over the life of an installation, Google receives: an IP address, an API
//! key identifying the application, periodic list updates that carry no user
//! data at all, and, on the rare collision, four bytes and a timestamp. Not the
//! URL, the sender, the subject, the recipient, or any part of a message.
//!
//! # How the pieces fit
//!
//! - [`urls`] turns a link into the up to thirty expressions a listed site
//!   could have been listed under. Pure, and tested against Google's own
//!   published vectors, because canonicalisation is where the evasions live.
//! - [`rice`] decodes the compressed form the lists arrive in.
//! - [`database`] holds the prefixes, applies updates, and checks the list
//!   against the checksum Google sends for it.
//! - [`client`] is the only part that touches the network.
//!
//! # It is off unless it is switched on
//!
//! It needs a Google Cloud API key, which is an ordinary key rather than OAuth,
//! so it does not touch the verification problem that limits Gmail sign-in. No
//! key means the whole thing is inert: [`Checker::available`] answers false and
//! every link is judged by the message's own headers as before.

pub mod client;
pub mod database;
pub mod rice;
pub mod urls;

pub use urls::{Canonical, canonicalise, expressions};

use crate::service::safety::{Safety, Verdict};

/// Which of Google's lists a hash was found on.
///
/// The names are Google's. What matters here is the last one: a URL on the
/// social engineering list is a phishing site by Google's own determination,
/// which is a fact rather than a heuristic, so it may say Phishing outright.
/// The others are things to be wary of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatKind {
    Malware,
    SocialEngineering,
    UnwantedSoftware,
    PotentiallyHarmfulApplication,
}

impl ThreatKind {
    /// The list's name in the API.
    pub const fn as_api_name(self) -> &'static str {
        match self {
            Self::Malware => "MALWARE",
            Self::SocialEngineering => "SOCIAL_ENGINEERING",
            Self::UnwantedSoftware => "UNWANTED_SOFTWARE",
            Self::PotentiallyHarmfulApplication => "POTENTIALLY_HARMFUL_APPLICATION",
        }
    }

    /// How bad this makes the message.
    pub const fn severity(self) -> Safety {
        match self {
            // Google's own determination that the site exists to deceive
            // people, which is what the word means. Not a heuristic to hedge.
            Self::SocialEngineering => Safety::Phishing,
            // A link to a site that will try to install something. Serious,
            // and not the same claim as "this message is impersonating your
            // bank", so it does not borrow that word.
            Self::Malware | Self::UnwantedSoftware | Self::PotentiallyHarmfulApplication => {
                Safety::Suspicious
            }
        }
    }

    /// What to say about it, in words rather than in Google's constant names.
    pub const fn as_sentence(self) -> &'static str {
        match self {
            Self::SocialEngineering => {
                "A link in this message goes to a site Google lists as a \
                 phishing site"
            }
            Self::Malware => {
                "A link in this message goes to a site Google lists as \
                 distributing malware"
            }
            Self::UnwantedSoftware => {
                "A link in this message goes to a site Google lists as \
                 distributing unwanted software"
            }
            Self::PotentiallyHarmfulApplication => {
                "A link in this message goes to a site Google lists as \
                 offering a harmful application"
            }
        }
    }
}

/// The lists this application asks for.
///
/// Not the whole set Google offers. Each list costs a periodic download and a
/// slice of memory, and these four are the ones that say something about a link
/// in a message rather than about an Android package.
pub const LISTS: [ThreatKind; 4] = [
    ThreatKind::SocialEngineering,
    ThreatKind::Malware,
    ThreatKind::UnwantedSoftware,
    ThreatKind::PotentiallyHarmfulApplication,
];

/// Where the API key comes from.
///
/// The same shape as the OAuth client credentials: an environment variable
/// first, then the gitignored `oauth.toml` in the settings folder. It is not in
/// the repository and never will be.
///
/// This being absent is the ordinary state, not an error. Safe Browsing is an
/// addition, and without a key the application checks links exactly as it did
/// before: by what the message's own headers say.
///
/// A mutation test replacing this whole body with `None` survives: without a
/// real `WIXEN_SAFE_BROWSING_KEY` set or a real `oauth.toml` carrying one,
/// the honest answer already is `None`. Proving the function ever returns
/// `Some` needs one of those two real, per-machine facts, which is exactly
/// what `SecurityService::trusted_domains` in `service::security` already
/// says a test must not arrange: every test here shares one process, and
/// setting an environment variable is process-wide. The trim-and-check logic
/// on each source is pulled out below instead, so it can be pinned without
/// touching either.
pub fn api_key() -> Option<String> {
    if let Some(key) = usable_env_key(std::env::var("WIXEN_SAFE_BROWSING_KEY")) {
        return Some(key);
    }
    let path = crate::common::paths::AppPaths::resolve()
        .ok()?
        .config_dir()
        .join("oauth.toml");
    let content = std::fs::read_to_string(path).ok()?;
    api_key_from_toml(&content)
}

/// Whether an environment variable's value is a usable key, once trimmed.
///
/// Split out from [`api_key`] so it can be tested without setting process
/// environment: every test in this suite shares one process, and a test that
/// set `WIXEN_SAFE_BROWSING_KEY` would change what its neighbours see.
fn usable_env_key(raw: Result<String, std::env::VarError>) -> Option<String> {
    let trimmed = raw.ok()?.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

/// Read the key out of an `oauth.toml` file's text.
///
/// Split from reading the file, the same shape as
/// `service::oauth_credentials::credentials_from_toml`, so the parsing can be
/// tested against a string in memory rather than a file on disk.
fn api_key_from_toml(content: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Entry {
        api_key: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct File {
        safe_browsing: Option<Entry>,
    }
    toml::from_str::<File>(content)
        .ok()?
        .safe_browsing?
        .api_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

/// The question asked of one link, and the answer.
///
/// Split out from the network so the decision about *whether* to ask anybody
/// anything is testable on its own. That decision is the entire privacy
/// argument, and it should not live inside an async function nothing can run
/// without a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The four-byte prefixes that collided with the local list.
    ///
    /// Empty means nothing is sent, which for ordinary correspondence is every
    /// link every time.
    pub prefixes: Vec<u32>,
    /// The full hashes those prefixes came from, kept here to compare against
    /// whatever comes back. These never leave the machine.
    pub full_hashes: Vec<[u8; 32]>,
}

impl Question {
    /// Whether Google has to be asked anything at all.
    pub fn needs_asking(&self) -> bool {
        !self.prefixes.is_empty()
    }
}

/// Work out what, if anything, would have to be asked about one link.
///
/// Everything here happens on this machine: canonicalising the URL, generating
/// the expressions a listed site could be listed under, hashing them, and
/// comparing against the local prefix list. Only a collision produces anything
/// to send.
pub fn question_for(url: &str, lists: &[&database::PrefixSet]) -> Question {
    let Some(canonical) = canonicalise(url) else {
        return Question {
            prefixes: Vec::new(),
            full_hashes: Vec::new(),
        };
    };

    let mut prefixes = Vec::new();
    let mut full_hashes = Vec::new();
    for expression in expressions(&canonical) {
        let hash = database::full_hash_of(&expression);
        if !lists.iter().any(|list| list.contains(&hash)) {
            continue;
        }
        let prefix = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        if !prefixes.contains(&prefix) {
            prefixes.push(prefix);
            full_hashes.push(hash);
        }
    }
    Question {
        prefixes,
        full_hashes,
    }
}

/// The most links to check in one message.
///
/// A newsletter carries hundreds, mostly to the same few hosts, and each one
/// costs thirty hashes. This is a bound on the work, not on the checking: the
/// links past it are almost always more of the same tracking URLs.
const MAX_LINKS: usize = 200;

/// Every http and https link in a message body.
///
/// Deliberately over-inclusive. Everything found is hashed on this machine and
/// nothing is sent unless a hash collides, so catching an image source or a
/// URL inside an attribute costs nothing and missing the one that matters
/// costs everything.
///
/// `mailto:` is not matched, and [`canonicalise`] refuses it as well, because
/// an address is not a link and hashing one would put four bytes derived from
/// somebody's email address in a position to be sent.
pub fn links_in(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = body;
    while let Some(at) = rest.find("http") {
        let candidate = &rest[at..];
        if !(candidate.starts_with("http://") || candidate.starts_with("https://")) {
            // Not a scheme, just the letters. Step past them rather than
            // rescanning from the same place forever.
            rest = &candidate["http".len()..];
            continue;
        }
        let end = candidate
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '`' | '\\'))
            .unwrap_or(candidate.len());
        // Trailing punctuation belongs to the sentence rather than to the URL.
        let url = candidate[..end].trim_end_matches(['.', ',', ';', ':', ')', ']', '!', '?']);
        if url.len() > "https://".len() && !found.iter().any(|seen| seen == url) {
            found.push(url.to_string());
            if found.len() >= MAX_LINKS {
                return found;
            }
        }
        rest = &candidate[end.max(1)..];
    }
    found
}

/// Where the copies of the lists live, under the cache folder.
///
/// The cache, not the config, because they are a copy of a public list that can
/// be fetched again. Nothing here is worth backing up and nothing here is about
/// the person using the application.
fn list_path(cache_dir: &std::path::Path, kind: ThreatKind) -> std::path::PathBuf {
    cache_dir
        .join("safebrowsing")
        .join(format!("{}.bin", kind.as_api_name().to_lowercase()))
}

/// Which of the lists this application asked about one update response names.
///
/// Split out from the loop in [`refresh_lists`] so the match can be tested
/// without a live update to loop over.
fn list_for_update(update: &client::ListUpdateResponse) -> Option<ThreatKind> {
    LISTS
        .iter()
        .copied()
        .find(|kind| kind.as_api_name() == update.threat_type)
}

/// Bring the local copies of the lists up to date.
///
/// The request carries the lists wanted and what this copy already has. It
/// would be byte-identical on a machine that had never received a message.
///
/// A list that fails its checksum is thrown away rather than kept, so the next
/// run asks for it whole. Keeping a list that has drifted is worse than having
/// none: removal indices then take out the wrong entries, and every update
/// after it makes the divergence worse without anything saying so.
///
/// A mutation test replacing this whole body with `Ok(())` survives. Every
/// path through it calls [`client::fetch_updates`] unconditionally, which
/// `client`'s own module doc already names as untested against the live
/// service, so there is no local-only route through this function for a test
/// to take. The lookup inside the loop below is pulled out into
/// [`list_for_update`] so that one piece can still be pinned on its own.
pub async fn refresh_lists(
    api_key: &str,
    cache_dir: &std::path::Path,
) -> crate::common::Result<()> {
    let states: Vec<(ThreatKind, String)> = LISTS
        .iter()
        .map(|kind| {
            let held = database::PrefixSet::load(&list_path(cache_dir, *kind));
            (*kind, held.state)
        })
        .collect();

    let response = client::fetch_updates(api_key, &LISTS, &states).await?;

    for update in &response.list_update_responses {
        let Some(kind) = list_for_update(update) else {
            continue;
        };
        let path = list_path(cache_dir, kind);
        let held = database::PrefixSet::load(&path);
        match client::apply(&held, update) {
            Ok(updated) => updated.save(&path)?,
            Err(e) => {
                // Said out loud, and the copy removed so the next run starts
                // clean. Swallowing this leaves a list that looks maintained.
                tracing::warn!("Threat list {} rejected: {}", update.threat_type, e);
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

/// Every local collision across every link, Google's prefixes and full
/// hashes kept in step with each other.
///
/// Split out from [`check_links`] so the cross-link deduplication can be
/// tested without an `async` call to drive: two different links in the same
/// message can hash to the same prefix, and asking about it twice tells
/// Google nothing new.
fn collisions_for(urls: &[String], lists: &[&database::PrefixSet]) -> (Vec<u32>, Vec<[u8; 32]>) {
    let mut prefixes = Vec::new();
    let mut full_hashes = Vec::new();
    for url in urls {
        let question = question_for(url, lists);
        for (prefix, hash) in question.prefixes.iter().zip(question.full_hashes.iter()) {
            if !prefixes.contains(prefix) {
                prefixes.push(*prefix);
                full_hashes.push(*hash);
            }
        }
    }
    (prefixes, full_hashes)
}

/// What the lists say about every link in one message.
///
/// Returns `None` when there is nothing to say, which is the ordinary answer.
/// Not listed means Google has not listed it, and turning that into "this is
/// safe" is the claim this application must never make.
///
/// A mutation test replacing this whole body with `None` survives: both
/// guards below already answer `None` on their own, without ever reaching
/// [`client::find_full_hashes`], so nothing here proves the function can
/// answer anything else. Proving that needs a real collision followed by a
/// real answer from Google, which `client`'s own module doc already names as
/// untested against the live service. Replacing the whole body with
/// `Some(Default::default())` does not survive: the "nothing fetched yet"
/// and "nothing collided" guards are both real, local-only decisions, and
/// either one gives `None` where the hardcoded mutant gives `Some`.
pub async fn check_links(
    api_key: &str,
    cache_dir: &std::path::Path,
    urls: &[String],
) -> Option<Verdict> {
    let held: Vec<database::PrefixSet> = LISTS
        .iter()
        .map(|kind| database::PrefixSet::load(&list_path(cache_dir, *kind)))
        .collect();
    if held.iter().all(database::PrefixSet::is_empty) {
        // Nothing has been fetched yet, so every link would "not match" for
        // the wrong reason. Better to say nothing than to imply a check ran.
        return None;
    }
    let borrowed: Vec<&database::PrefixSet> = held.iter().collect();

    let (prefixes, full_hashes) = collisions_for(urls, &borrowed);
    if prefixes.is_empty() {
        // The ordinary case for ordinary mail: nothing collided, so nothing is
        // sent and Google learns nothing, not even that a message was read.
        return None;
    }

    let response = match client::find_full_hashes(api_key, &prefixes, &LISTS).await {
        Ok(response) => response,
        Err(e) => {
            // Not silently absorbed. A check that could not run is a gap, and
            // reporting the message as unremarkable would hide it.
            tracing::warn!("Safe Browsing could not be asked: {}", e);
            return None;
        }
    };
    verdict_for(&client::confirmed(&response, &full_hashes))
}

/// Turn what the lists said into a verdict the rest of the application merges.
///
/// Attribution is required by Google's terms wherever a warning derived from
/// their data is shown, so it is part of the sentence rather than something to
/// be remembered separately in a UI somewhere.
pub fn verdict_for(found: &[ThreatKind]) -> Option<Verdict> {
    let worst = found.iter().copied().max_by_key(|kind| kind.severity())?;
    Some(Verdict {
        level: worst.severity(),
        reasons: vec![format!(
            "{}. Checked against Google Safe Browsing.",
            worst.as_sentence()
        )],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_links_in_a_message_are_found_wherever_they_sit() {
        let body = "<p>See <a href=\"https://example.com/a\">this</a> and \
                    http://plain.example.org/b, or write to me@example.com.</p>";

        let found = links_in(body);

        assert!(
            found.contains(&"https://example.com/a".to_string()),
            "{found:?}"
        );
        assert!(
            found.contains(&"http://plain.example.org/b".to_string()),
            "{found:?}"
        );
    }

    #[test]
    fn test_each_threat_list_is_asked_for_by_its_own_name() {
        // The name goes into the request and into the file the copy is kept
        // in. Wrong, and the answer is about a different list than the one the
        // verdict will be reported as.
        for (kind, expected) in [
            (ThreatKind::Malware, "MALWARE"),
            (ThreatKind::SocialEngineering, "SOCIAL_ENGINEERING"),
            (ThreatKind::UnwantedSoftware, "UNWANTED_SOFTWARE"),
            (
                ThreatKind::PotentiallyHarmfulApplication,
                "POTENTIALLY_HARMFUL_APPLICATION",
            ),
        ] {
            assert_eq!(kind.as_api_name(), expected, "for {kind:?}");
        }
    }

    #[test]
    fn test_each_list_is_kept_in_a_file_of_its_own_under_the_cache() {
        // One file per list, named for the list. Sharing a path makes each
        // update overwrite the last, so three of the four lists are always
        // whatever was fetched most recently.
        let cache = std::path::Path::new("C:\\cache");
        let mut seen = std::collections::BTreeSet::new();

        for kind in [
            ThreatKind::Malware,
            ThreatKind::SocialEngineering,
            ThreatKind::UnwantedSoftware,
            ThreatKind::PotentiallyHarmfulApplication,
        ] {
            let path = list_path(cache, kind);
            assert!(
                path.starts_with(cache),
                "{path:?} is not under the cache folder"
            );
            assert!(
                path.to_string_lossy().contains("safebrowsing"),
                "{path:?} is not in the safebrowsing folder"
            );
            assert!(seen.insert(path.clone()), "{path:?} is used for two lists");
        }
    }

    #[test]
    fn test_a_scheme_with_nothing_after_it_is_not_a_link_to_ask_about() {
        // "https://" on its own has no host, so there is nothing to look up
        // and asking would spend one of the thirty questions on nothing.
        assert!(links_in("see https:// and http:// here").is_empty());
        assert_eq!(
            links_in("see https://a here"),
            vec!["https://a".to_string()],
            "one character past the scheme is a link"
        );
    }

    #[test]
    fn test_an_email_address_is_not_a_link() {
        // Hashing one would put four bytes derived from somebody's address in
        // a position to be sent to Google.
        let found = links_in("Write to someone@example.com or mailto:other@example.com");

        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn test_the_sentence_around_a_link_is_not_part_of_it() {
        let found = links_in("Go to https://example.com/page, then stop.");

        assert_eq!(found, vec!["https://example.com/page".to_string()]);
    }

    #[test]
    fn test_the_same_link_twice_is_checked_once() {
        let found = links_in("https://example.com/a and again https://example.com/a");

        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_the_word_http_on_its_own_does_not_stall_the_scan() {
        // The loop steps past a false start rather than searching from the
        // same place again, which would never finish.
        let found = links_in("The http protocol, see httpx and http-only cookies.");

        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn test_a_message_full_of_links_stops_at_a_bound() {
        // A newsletter carries hundreds, mostly the same few hosts, and each
        // one costs thirty hashes.
        let body: String = (0..500)
            .map(|n| format!("https://example.com/{n} "))
            .collect();

        assert!(links_in(&body).len() <= MAX_LINKS);
    }

    #[test]
    fn test_an_ordinary_link_asks_google_nothing() {
        // The whole privacy argument in one assertion. A link that does not
        // collide with the local list produces no request, so reading ordinary
        // correspondence tells Google nothing at all, not even that a message
        // was read.
        let empty = database::PrefixSet::new();

        for link in [
            "https://example.com/invoices/2026",
            "http://mail.example.co.uk/a/b?c=d",
            "https://192.168.0.1/status",
        ] {
            let question = question_for(link, &[&empty]);

            assert!(!question.needs_asking(), "{link} would have been sent");
            assert!(question.prefixes.is_empty());
        }
    }

    #[test]
    fn test_a_link_that_collides_sends_four_bytes_and_no_more() {
        // Seed the list with a prefix one of this URL's own expressions
        // hashes to, which is what a real collision looks like.
        let canonical = canonicalise("https://example.com/").expect("a URL");
        let expression = expressions(&canonical)
            .into_iter()
            .next()
            .expect("at least one expression");
        let listed = database::PrefixSet::from_prefixes([database::prefix_of(&expression)]);

        let question = question_for("https://example.com/", &[&listed]);

        assert!(question.needs_asking());
        assert_eq!(question.prefixes.len(), 1);
        // The full hash stays here, to compare against what comes back. Only
        // the four bytes go.
        assert_eq!(question.full_hashes.len(), 1);
        assert_eq!(
            question.prefixes[0].to_be_bytes(),
            question.full_hashes[0][..4]
        );
    }

    #[test]
    fn test_something_that_is_not_a_link_is_not_turned_into_a_question() {
        // A mailto: is an address, and canonicalising it as a URL would send
        // Google four bytes derived from somebody's email address.
        let empty = database::PrefixSet::new();

        for not_a_link in [
            "mailto:someone@example.com",
            "",
            "not a url at all",
            "javascript:alert(1)",
        ] {
            assert!(
                !question_for(not_a_link, &[&empty]).needs_asking(),
                "{not_a_link}"
            );
        }
    }

    #[test]
    fn test_the_same_prefix_is_only_asked_about_once() {
        // A URL generates up to thirty expressions and several can hash to the
        // same prefix. Sending it twice tells Google nothing new and costs a
        // round trip.
        let canonical = canonicalise("https://example.com/a/b/c").expect("a URL");
        let all: Vec<u32> = expressions(&canonical)
            .iter()
            .map(|e| database::prefix_of(e))
            .collect();
        let listed = database::PrefixSet::from_prefixes(all.clone());

        let question = question_for("https://example.com/a/b/c", &[&listed]);

        let mut unique = question.prefixes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), question.prefixes.len());
    }

    #[test]
    fn test_a_phishing_listing_is_called_what_it_is() {
        // Google saying a site exists to deceive people is a determination
        // rather than a heuristic, so this is the one source allowed to say
        // the word on its own.
        let verdict = verdict_for(&[ThreatKind::SocialEngineering]).expect("a verdict");

        assert_eq!(verdict.level, Safety::Phishing);
        assert!(verdict.reasons[0].contains("phishing"), "{:?}", verdict);
    }

    #[test]
    fn test_a_malware_listing_stops_short_of_calling_it_phishing() {
        // A different claim. "This message is impersonating your bank" and
        // "this link installs something" are not the same warning, and
        // borrowing the stronger word for the weaker fact is how a warning
        // stops meaning anything.
        for kind in [
            ThreatKind::Malware,
            ThreatKind::UnwantedSoftware,
            ThreatKind::PotentiallyHarmfulApplication,
        ] {
            let verdict = verdict_for(&[kind]).expect("a verdict");

            assert_eq!(verdict.level, Safety::Suspicious, "{kind:?}");
        }
    }

    #[test]
    fn test_the_worst_listing_wins_when_a_message_hits_several() {
        let verdict =
            verdict_for(&[ThreatKind::Malware, ThreatKind::SocialEngineering]).expect("a verdict");

        assert_eq!(verdict.level, Safety::Phishing);
    }

    #[test]
    fn test_nothing_found_is_no_verdict_rather_than_a_clean_bill() {
        // Which is the point. Safe Browsing not listing a site says only that
        // Google has not listed it, and turning that into "this is safe" is
        // the claim this application must never make.
        assert_eq!(verdict_for(&[]), None);
    }

    #[test]
    fn test_the_environment_key_is_trimmed_and_blank_is_not_a_key() {
        assert_eq!(
            usable_env_key(Ok("  a-fake-test-key  ".to_string())),
            Some("a-fake-test-key".to_string())
        );
        assert_eq!(usable_env_key(Ok("   ".to_string())), None);
        assert_eq!(usable_env_key(Ok(String::new())), None);
        assert_eq!(usable_env_key(Err(std::env::VarError::NotPresent)), None);
    }

    #[test]
    fn test_the_toml_key_is_trimmed_and_blank_is_not_a_key() {
        assert_eq!(
            api_key_from_toml("[safe_browsing]\napi_key = \"  a-fake-test-key  \"\n"),
            Some("a-fake-test-key".to_string())
        );
        assert_eq!(
            api_key_from_toml("[safe_browsing]\napi_key = \"\"\n"),
            None,
            "a blank key in the file is not a key"
        );
        assert_eq!(
            api_key_from_toml("[gmail]\nclient_id = \"x\"\n"),
            None,
            "no safe_browsing section at all"
        );
        assert_eq!(
            api_key_from_toml("[safe_browsing]\n"),
            None,
            "a section with no api_key field"
        );
        assert_eq!(api_key_from_toml("not toml at all {{{"), None);
    }

    #[test]
    fn test_an_update_response_is_matched_to_its_list_by_name() {
        let for_malware = client::ListUpdateResponse {
            threat_type: "MALWARE".to_string(),
            response_type: String::new(),
            state: String::new(),
            additions: Vec::new(),
            removals: Vec::new(),
            checksum: None,
        };
        assert_eq!(list_for_update(&for_malware), Some(ThreatKind::Malware));

        let unknown = client::ListUpdateResponse {
            threat_type: "SOMETHING_ELSE".to_string(),
            response_type: String::new(),
            state: String::new(),
            additions: Vec::new(),
            removals: Vec::new(),
            checksum: None,
        };
        assert_eq!(list_for_update(&unknown), None);
    }

    #[test]
    fn test_two_links_that_collide_on_the_same_prefix_are_asked_about_once() {
        // The same collision as test_the_same_prefix_is_only_asked_about_once,
        // one level up: two different messages links can still share one
        // prefix, and question_for's own per-link dedup does not see across
        // the two calls this aggregation makes.
        let canonical = canonicalise("https://example.com/").expect("a URL");
        let expression = expressions(&canonical)
            .into_iter()
            .next()
            .expect("at least one expression");
        let listed = database::PrefixSet::from_prefixes([database::prefix_of(&expression)]);
        let urls = [
            "https://example.com/".to_string(),
            "https://example.com/".to_string(),
        ];

        let (prefixes, full_hashes) = collisions_for(&urls, &[&listed]);

        assert_eq!(prefixes.len(), 1, "{prefixes:?}");
        assert_eq!(full_hashes.len(), 1);
    }

    #[test]
    fn test_check_links_says_nothing_before_any_list_has_been_fetched() {
        let dir = tempfile::tempdir().expect("a temp dir");

        let verdict = tokio_test::block_on(check_links(
            "unused-test-key",
            dir.path(),
            &["https://example.com/".to_string()],
        ));

        assert_eq!(verdict, None);
    }

    #[test]
    fn test_check_links_says_nothing_when_no_link_collides() {
        let dir = tempfile::tempdir().expect("a temp dir");
        // A non-empty list that nothing in the message matches, so the first
        // guard (nothing fetched yet) does not fire and the second one (no
        // collision) is what this test is actually pinning.
        let listed = database::PrefixSet::from_prefixes([0xDEAD_BEEF]);
        for kind in LISTS {
            listed
                .save(&list_path(dir.path(), kind))
                .expect("write a fake cached list");
        }

        let verdict = tokio_test::block_on(check_links(
            "unused-test-key",
            dir.path(),
            &["https://example.com/harmless".to_string()],
        ));

        assert_eq!(verdict, None);
    }

    #[test]
    fn test_every_warning_credits_google() {
        // Their terms ask for it wherever a warning derived from their data is
        // shown, so it lives in the sentence rather than in somebody's memory
        // of a UI requirement.
        for kind in [
            ThreatKind::Malware,
            ThreatKind::SocialEngineering,
            ThreatKind::UnwantedSoftware,
            ThreatKind::PotentiallyHarmfulApplication,
        ] {
            let verdict = verdict_for(&[kind]).expect("a verdict");

            assert!(
                verdict.reasons[0].contains("Google Safe Browsing"),
                "{kind:?}: {:?}",
                verdict.reasons
            );
        }
    }
}

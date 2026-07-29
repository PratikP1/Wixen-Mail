//! The only part of Safe Browsing that touches the network.
//!
//! Two calls, and it is worth being precise about what each one sends.
//!
//! `threatListUpdates:fetch` asks for changes to the public threat lists. It
//! carries a client name, a version, the lists wanted, and an opaque state
//! token saying what this copy already has. It carries nothing about the person
//! using the application, the mail they have, or the links in it. It would send
//! exactly the same bytes on a machine that had never received a message.
//!
//! `fullHashes:find` is the one that only happens on a collision, and it sends
//! four bytes. Those four bytes are the start of a SHA-256 and stand for
//! roughly four billion possible URLs, so what Google learns is that somebody
//! at this IP address saw one of a very large set of links. The answer comes
//! back as every full hash sharing that prefix and the comparison happens on
//! this machine.
//!
//! What is never sent, in either call: a URL, a domain, a sender, a subject, a
//! recipient, or any part of a message.
//!
//! # Untested against the live service
//!
//! Every shape here is built and parsed by tested code, and none of it has been
//! run against Google, because that needs an API key and this application does
//! not have one yet. The request and response types are covered by tests over
//! real recorded shapes; the transport is not. Treat the first run with a key
//! as the test, and see the note in [`super`] about the whole thing being inert
//! without one.

use super::database::PrefixSet;
use super::rice::{self, RiceDelta};
use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};

/// Where the v4 API lives.
const ENDPOINT: &str = "https://safebrowsing.googleapis.com/v4";

/// What this client calls itself to Google.
///
/// Required by the API, and it is not a secret or an identifier for a person:
/// every installation sends the same two strings.
const CLIENT_ID: &str = "wixen-mail";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The most full hashes to accept for one prefix.
///
/// A prefix collides with a handful of entries at most. A response claiming
/// thousands is either broken or trying to make this allocate.
const MAX_FULL_HASHES: usize = 1_000;

// ── What goes out ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct UpdateRequest {
    client: ClientInfo,
    #[serde(rename = "listUpdateRequests")]
    list_update_requests: Vec<ListUpdateRequest>,
}

#[derive(Debug, Serialize)]
struct ClientInfo {
    #[serde(rename = "clientId")]
    client_id: &'static str,
    #[serde(rename = "clientVersion")]
    client_version: &'static str,
}

#[derive(Debug, Serialize)]
struct ListUpdateRequest {
    #[serde(rename = "threatType")]
    threat_type: &'static str,
    #[serde(rename = "platformType")]
    platform_type: &'static str,
    #[serde(rename = "threatEntryType")]
    threat_entry_type: &'static str,
    /// What this copy already has. Empty asks for the whole list.
    state: String,
    constraints: Constraints,
}

#[derive(Debug, Serialize)]
struct Constraints {
    /// Both encodings, so Google may send whichever is smaller.
    #[serde(rename = "supportedCompressions")]
    supported_compressions: Vec<&'static str>,
}

// ── What comes back ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UpdateResponse {
    #[serde(rename = "listUpdateResponses", default)]
    pub list_update_responses: Vec<ListUpdateResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ListUpdateResponse {
    #[serde(rename = "threatType")]
    pub threat_type: String,
    /// `FULL_UPDATE` throws away what is held; `PARTIAL_UPDATE` amends it.
    #[serde(rename = "responseType", default)]
    pub response_type: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub additions: Vec<ThreatEntrySet>,
    #[serde(default)]
    pub removals: Vec<ThreatEntrySet>,
    /// Base64 of the SHA-256 the list should have once this is applied.
    #[serde(default)]
    pub checksum: Option<Checksum>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Checksum {
    #[serde(default)]
    pub sha256: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ThreatEntrySet {
    #[serde(rename = "rawHashes", default)]
    pub raw_hashes: Option<RawHashes>,
    #[serde(rename = "riceHashes", default)]
    pub rice_hashes: Option<RiceEncoded>,
    #[serde(rename = "rawIndices", default)]
    pub raw_indices: Option<RawIndices>,
    #[serde(rename = "riceIndices", default)]
    pub rice_indices: Option<RiceEncoded>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RawHashes {
    #[serde(rename = "prefixSize", default)]
    pub prefix_size: u32,
    /// Base64 of every prefix concatenated, each `prefix_size` bytes.
    #[serde(rename = "rawHashes", default)]
    pub raw_hashes: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct RawIndices {
    #[serde(default)]
    pub indices: Vec<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RiceEncoded {
    #[serde(rename = "riceParameter", default)]
    pub rice_parameter: u32,
    #[serde(rename = "numEntries", default)]
    pub num_entries: u32,
    /// A number as a string, which is how the API sends 32-bit values.
    #[serde(rename = "firstValue", default)]
    pub first_value: String,
    #[serde(rename = "encodedData", default)]
    pub encoded_data: String,
}

// ── Turning a response into prefixes ────────────────────────────────────────

/// Every prefix an addition set carries, whichever way it was encoded.
fn additions_in(set: &ThreatEntrySet) -> Result<Vec<u32>> {
    if let Some(raw) = &set.raw_hashes {
        return decode_raw_hashes(raw);
    }
    if let Some(rice) = &set.rice_hashes {
        return decode_rice(rice);
    }
    Ok(Vec::new())
}

/// Every index a removal set carries, whichever way it was encoded.
fn removals_in(set: &ThreatEntrySet) -> Result<Vec<u32>> {
    if let Some(raw) = &set.raw_indices {
        return Ok(raw.indices.clone());
    }
    if let Some(rice) = &set.rice_indices {
        return decode_rice(rice);
    }
    Ok(Vec::new())
}

/// Prefixes sent uncompressed: base64 of them all run together.
fn decode_raw_hashes(raw: &RawHashes) -> Result<Vec<u32>> {
    use base64::Engine as _;

    // Google may send prefixes longer than four bytes. Only the first four are
    // used, because that is what the local list holds, and a longer prefix
    // still matches on its first four.
    let size = raw.prefix_size.max(4) as usize;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&raw.raw_hashes)
        .map_err(|e| Error::Protocol(format!("The threat list is not readable base64: {e}")))?;
    if !bytes.len().is_multiple_of(size) {
        return Err(Error::Protocol(format!(
            "The threat list holds {} bytes, which is not a whole number of \
             {}-byte prefixes",
            bytes.len(),
            size
        )));
    }
    Ok(bytes
        .chunks_exact(size)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

/// Prefixes or indices sent Rice-Golomb encoded.
fn decode_rice(encoded: &RiceEncoded) -> Result<Vec<u32>> {
    use base64::Engine as _;

    // Sent as a decimal string, which is how the API represents 32-bit values
    // in JSON. Missing means zero, which is what an empty run starts from.
    let first_value: u32 = if encoded.first_value.trim().is_empty() {
        0
    } else {
        encoded.first_value.trim().parse().map_err(|_| {
            Error::Protocol(format!(
                "The threat list update starts from {:?}, which is not a number",
                encoded.first_value
            ))
        })?
    };
    let data = base64::engine::general_purpose::STANDARD
        .decode(&encoded.encoded_data)
        .map_err(|e| Error::Protocol(format!("The threat list is not readable base64: {e}")))?;

    rice::decode(&RiceDelta {
        first_value,
        parameter: encoded.rice_parameter,
        entries: encoded.num_entries,
        data,
    })
}

/// Apply one list's response to the copy held here.
///
/// A full update replaces; a partial one amends. Either way the result is
/// checked against the checksum Google sent, and a mismatch throws the list
/// away rather than keeping a copy that is out of step, because removal indices
/// applied to a drifted list take out the wrong entries and nothing says so.
pub fn apply(current: &PrefixSet, response: &ListUpdateResponse) -> Result<PrefixSet> {
    let base = if response.response_type == "FULL_UPDATE" {
        PrefixSet::new()
    } else {
        current.clone()
    };

    let mut removals = Vec::new();
    for set in &response.removals {
        removals.extend(removals_in(set)?);
    }
    let mut additions = Vec::new();
    for set in &response.additions {
        additions.extend(additions_in(set)?);
    }

    let mut updated = base.apply(&removals, &additions)?;
    updated.state = response.state.clone();

    if let Some(checksum) = &response.checksum
        && !checksum.sha256.is_empty()
    {
        use base64::Engine as _;
        let expected = base64::engine::general_purpose::STANDARD
            .decode(&checksum.sha256)
            .map_err(|e| Error::Protocol(format!("The list checksum is not readable: {e}")))?;
        if !updated.matches(&expected) {
            return Err(Error::Protocol(
                "The threat list here does not match the one the server \
                 describes, so it is being thrown away and fetched again. \
                 Nothing about your mail was involved."
                    .into(),
            ));
        }
    }

    Ok(updated)
}

// ── The calls themselves ────────────────────────────────────────────────────

/// Ask Google for changes to one threat list.
///
/// The request carries the list wanted and what this copy already has, and
/// nothing else. It would be byte-identical on a machine that had never
/// received a message.
pub async fn fetch_updates(
    api_key: &str,
    lists: &[super::ThreatKind],
    states: &[(super::ThreatKind, String)],
) -> Result<UpdateResponse> {
    let request = UpdateRequest {
        client: ClientInfo {
            client_id: CLIENT_ID,
            client_version: CLIENT_VERSION,
        },
        list_update_requests: lists
            .iter()
            .map(|kind| ListUpdateRequest {
                threat_type: kind.as_api_name(),
                platform_type: "WINDOWS",
                threat_entry_type: "URL",
                state: states
                    .iter()
                    .find(|(other, _)| other == kind)
                    .map(|(_, state)| state.clone())
                    .unwrap_or_default(),
                constraints: Constraints {
                    supported_compressions: vec!["RAW", "RICE"],
                },
            })
            .collect(),
    };

    let response = reqwest::Client::new()
        .post(format!("{ENDPOINT}/threatListUpdates:fetch?key={api_key}"))
        .json(&request)
        .send()
        .await
        .map_err(|e| Error::Network(format!("Could not reach Safe Browsing: {e}")))?;

    if !response.status().is_success() {
        // The status and nothing else. A body from a failed request can carry
        // the key back, and this goes to a log file.
        return Err(Error::Network(format!(
            "Safe Browsing refused the request: {}",
            response.status()
        )));
    }
    response
        .json::<UpdateResponse>()
        .await
        .map_err(|e| Error::Protocol(format!("Safe Browsing sent something unreadable: {e}")))
}

/// What the full hash lookup sends and gets back.
#[derive(Debug, Serialize)]
struct FullHashRequest {
    client: ClientInfo,
    #[serde(rename = "threatInfo")]
    threat_info: ThreatInfo,
}

#[derive(Debug, Serialize)]
struct ThreatInfo {
    #[serde(rename = "threatTypes")]
    threat_types: Vec<&'static str>,
    #[serde(rename = "platformTypes")]
    platform_types: Vec<&'static str>,
    #[serde(rename = "threatEntryTypes")]
    threat_entry_types: Vec<&'static str>,
    /// The colliding prefixes, base64, and nothing else.
    #[serde(rename = "threatEntries")]
    threat_entries: Vec<ThreatEntry>,
}

#[derive(Debug, Serialize)]
struct ThreatEntry {
    hash: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct FullHashResponse {
    #[serde(default)]
    pub matches: Vec<FullHashMatch>,
}

#[derive(Debug, Deserialize)]
pub struct FullHashMatch {
    #[serde(rename = "threatType")]
    pub threat_type: String,
    #[serde(default)]
    pub threat: ThreatEntryValue,
}

#[derive(Debug, Deserialize, Default)]
pub struct ThreatEntryValue {
    #[serde(default)]
    pub hash: String,
}

/// Ask what full hashes share these prefixes.
///
/// The rare call, and the only one carrying anything derived from a link. Four
/// bytes per collision, which are the start of a SHA-256 and stand for billions
/// of possible URLs. The comparison against the real hash happens back here.
pub async fn find_full_hashes(
    api_key: &str,
    prefixes: &[u32],
    lists: &[super::ThreatKind],
) -> Result<FullHashResponse> {
    use base64::Engine as _;

    if prefixes.is_empty() {
        return Ok(FullHashResponse::default());
    }

    let request = FullHashRequest {
        client: ClientInfo {
            client_id: CLIENT_ID,
            client_version: CLIENT_VERSION,
        },
        threat_info: ThreatInfo {
            threat_types: lists.iter().map(|kind| kind.as_api_name()).collect(),
            platform_types: vec!["WINDOWS"],
            threat_entry_types: vec!["URL"],
            threat_entries: prefixes
                .iter()
                .map(|prefix| ThreatEntry {
                    hash: base64::engine::general_purpose::STANDARD.encode(prefix.to_be_bytes()),
                })
                .collect(),
        },
    };

    let response = reqwest::Client::new()
        .post(format!("{ENDPOINT}/fullHashes:find?key={api_key}"))
        .json(&request)
        .send()
        .await
        .map_err(|e| Error::Network(format!("Could not reach Safe Browsing: {e}")))?;

    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "Safe Browsing refused the request: {}",
            response.status()
        )));
    }
    response
        .json::<FullHashResponse>()
        .await
        .map_err(|e| Error::Protocol(format!("Safe Browsing sent something unreadable: {e}")))
}

/// Which of the returned full hashes really match, and on which list.
///
/// The prefix collision was a maybe; this is the answer. Comparison happens
/// here rather than at Google, which is the whole shape of the protocol.
pub fn confirmed(response: &FullHashResponse, ours: &[[u8; 32]]) -> Vec<super::ThreatKind> {
    use base64::Engine as _;

    let mut found = Vec::new();
    for matched in response.matches.iter().take(MAX_FULL_HASHES) {
        let Ok(theirs) = base64::engine::general_purpose::STANDARD.decode(&matched.threat.hash)
        else {
            continue;
        };
        if !ours.iter().any(|hash| hash.as_slice() == theirs.as_slice()) {
            continue;
        }
        let kind = match matched.threat_type.as_str() {
            "SOCIAL_ENGINEERING" => super::ThreatKind::SocialEngineering,
            "MALWARE" => super::ThreatKind::Malware,
            "UNWANTED_SOFTWARE" => super::ThreatKind::UnwantedSoftware,
            "POTENTIALLY_HARMFUL_APPLICATION" => super::ThreatKind::PotentiallyHarmfulApplication,
            // A list we do not know about is not a reason to warn about
            // something we cannot describe.
            _ => continue,
        };
        if !found.contains(&kind) {
            found.push(kind);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    fn b64(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[test]
    fn test_raw_prefixes_come_out_in_the_order_they_were_packed() {
        let mut packed = Vec::new();
        for prefix in [1u32, 500, 70_000] {
            packed.extend_from_slice(&prefix.to_be_bytes());
        }
        let set = ThreatEntrySet {
            raw_hashes: Some(RawHashes {
                prefix_size: 4,
                raw_hashes: b64(&packed),
            }),
            ..Default::default()
        };

        assert_eq!(additions_in(&set).expect("raw"), vec![1, 500, 70_000]);
    }

    #[test]
    fn test_a_longer_prefix_is_cut_to_the_four_bytes_the_list_holds() {
        // Google may send six or eight byte prefixes. A longer prefix still
        // matches on its first four, which is what is stored here.
        let mut packed = Vec::new();
        packed.extend_from_slice(&[0x00, 0x00, 0x01, 0xF4, 0xAA, 0xBB]);
        packed.extend_from_slice(&[0x00, 0x01, 0x11, 0x70, 0xCC, 0xDD]);
        let set = ThreatEntrySet {
            raw_hashes: Some(RawHashes {
                prefix_size: 6,
                raw_hashes: b64(&packed),
            }),
            ..Default::default()
        };

        assert_eq!(additions_in(&set).expect("raw"), vec![500, 70_000]);
    }

    #[test]
    fn test_a_list_that_is_not_a_whole_number_of_prefixes_is_refused() {
        // Half a prefix at the end means the response was truncated, and a
        // truncated list silently under-reports for as long as it is held.
        let set = ThreatEntrySet {
            raw_hashes: Some(RawHashes {
                prefix_size: 4,
                raw_hashes: b64(&[0, 0, 0, 1, 0, 0]),
            }),
            ..Default::default()
        };

        assert!(additions_in(&set).is_err());
    }

    #[test]
    fn test_something_that_is_not_base64_is_refused_rather_than_skipped() {
        let set = ThreatEntrySet {
            raw_hashes: Some(RawHashes {
                prefix_size: 4,
                raw_hashes: "not base64 at all!!".to_string(),
            }),
            ..Default::default()
        };

        assert!(additions_in(&set).is_err());
    }

    #[test]
    fn test_a_set_carrying_neither_encoding_is_empty_rather_than_an_error() {
        // A legitimate response shape: an update with only removals has an
        // additions array of empty sets.
        let set = ThreatEntrySet::default();

        assert!(additions_in(&set).expect("empty").is_empty());
        assert!(removals_in(&set).expect("empty").is_empty());
    }

    #[test]
    fn test_a_full_update_throws_away_what_was_held() {
        // The response type Google sends after a client has been away long
        // enough that amending would cost more than replacing.
        let current = PrefixSet::from_prefixes([1, 2, 3]);
        let mut packed = Vec::new();
        packed.extend_from_slice(&99u32.to_be_bytes());
        let response = ListUpdateResponse {
            threat_type: "SOCIAL_ENGINEERING".to_string(),
            response_type: "FULL_UPDATE".to_string(),
            state: "new-state".to_string(),
            additions: vec![ThreatEntrySet {
                raw_hashes: Some(RawHashes {
                    prefix_size: 4,
                    raw_hashes: b64(&packed),
                }),
                ..Default::default()
            }],
            removals: Vec::new(),
            checksum: None,
        };

        let updated = apply(&current, &response).expect("full update");

        assert_eq!(updated.len(), 1);
        assert!(updated.contains(&99u32.to_be_bytes()));
        assert_eq!(updated.state, "new-state");
    }

    #[test]
    fn test_a_partial_update_amends_what_was_held() {
        let current = PrefixSet::from_prefixes([10, 20, 30]);
        let response = ListUpdateResponse {
            threat_type: "MALWARE".to_string(),
            response_type: "PARTIAL_UPDATE".to_string(),
            state: "next".to_string(),
            additions: vec![ThreatEntrySet {
                raw_hashes: Some(RawHashes {
                    prefix_size: 4,
                    raw_hashes: b64(&40u32.to_be_bytes()),
                }),
                ..Default::default()
            }],
            removals: vec![ThreatEntrySet {
                raw_indices: Some(RawIndices { indices: vec![0] }),
                ..Default::default()
            }],
            checksum: None,
        };

        let updated = apply(&current, &response).expect("partial update");

        assert!(!updated.contains(&10u32.to_be_bytes()), "kept the removal");
        assert!(updated.contains(&40u32.to_be_bytes()), "lost the addition");
        assert_eq!(updated.len(), 3);
    }

    #[test]
    fn test_a_list_that_does_not_match_the_checksum_is_thrown_away() {
        // The failure the checksum exists for. A list out of step with the
        // server's has removal indices pointing at the wrong entries, and
        // every update after it makes the divergence worse, quietly.
        let current = PrefixSet::new();
        let response = ListUpdateResponse {
            threat_type: "MALWARE".to_string(),
            response_type: "FULL_UPDATE".to_string(),
            state: "next".to_string(),
            additions: vec![ThreatEntrySet {
                raw_hashes: Some(RawHashes {
                    prefix_size: 4,
                    raw_hashes: b64(&7u32.to_be_bytes()),
                }),
                ..Default::default()
            }],
            removals: Vec::new(),
            checksum: Some(Checksum {
                sha256: b64(&[0u8; 32]),
            }),
        };

        let error = apply(&current, &response).expect_err("checksum mismatch");

        assert!(error.to_string().contains("thrown away"), "{error}");
        // And it says the obvious worrying thing is not happening.
        assert!(
            error.to_string().contains("Nothing about your mail"),
            "{error}"
        );
    }

    #[test]
    fn test_a_list_that_matches_its_checksum_is_kept() {
        let current = PrefixSet::new();
        let expected = PrefixSet::from_prefixes([7]).checksum();
        let response = ListUpdateResponse {
            threat_type: "MALWARE".to_string(),
            response_type: "FULL_UPDATE".to_string(),
            state: "next".to_string(),
            additions: vec![ThreatEntrySet {
                raw_hashes: Some(RawHashes {
                    prefix_size: 4,
                    raw_hashes: b64(&7u32.to_be_bytes()),
                }),
                ..Default::default()
            }],
            removals: Vec::new(),
            checksum: Some(Checksum {
                sha256: b64(&expected),
            }),
        };

        assert!(apply(&current, &response).is_ok());
    }

    #[test]
    fn test_a_returned_hash_that_is_not_ours_is_not_a_match() {
        // The prefix collided, which is why the question was asked. The answer
        // is only yes when the whole hash agrees, and that comparison happens
        // here rather than at Google.
        let ours = [super::super::database::full_hash_of("example.com/")];
        let response = FullHashResponse {
            matches: vec![FullHashMatch {
                threat_type: "SOCIAL_ENGINEERING".to_string(),
                threat: ThreatEntryValue {
                    hash: b64(&super::super::database::full_hash_of("somewhere.else/")),
                },
            }],
        };

        assert!(confirmed(&response, &ours).is_empty());
    }

    #[test]
    fn test_a_returned_hash_that_is_ours_names_its_list() {
        let hash = super::super::database::full_hash_of("example.com/");
        let response = FullHashResponse {
            matches: vec![FullHashMatch {
                threat_type: "SOCIAL_ENGINEERING".to_string(),
                threat: ThreatEntryValue { hash: b64(&hash) },
            }],
        };

        assert_eq!(
            confirmed(&response, &[hash]),
            vec![super::super::ThreatKind::SocialEngineering]
        );
    }

    #[test]
    fn test_a_list_name_we_do_not_know_is_ignored_rather_than_guessed_at() {
        // Warning about something we cannot describe leaves somebody with a
        // fright and no way to act on it.
        let hash = super::super::database::full_hash_of("example.com/");
        let response = FullHashResponse {
            matches: vec![FullHashMatch {
                threat_type: "SOMETHING_NEW".to_string(),
                threat: ThreatEntryValue { hash: b64(&hash) },
            }],
        };

        assert!(confirmed(&response, &[hash]).is_empty());
    }

    #[test]
    fn test_the_same_list_twice_is_reported_once() {
        let hash = super::super::database::full_hash_of("example.com/");
        let response = FullHashResponse {
            matches: vec![
                FullHashMatch {
                    threat_type: "MALWARE".to_string(),
                    threat: ThreatEntryValue { hash: b64(&hash) },
                },
                FullHashMatch {
                    threat_type: "MALWARE".to_string(),
                    threat: ThreatEntryValue { hash: b64(&hash) },
                },
            ],
        };

        assert_eq!(confirmed(&response, &[hash]).len(), 1);
    }

    #[test]
    fn test_a_rice_encoded_run_is_read_the_same_way_as_a_raw_one() {
        // The decoder has its own tests; this checks the wiring, including the
        // first value arriving as a decimal string rather than a number.
        let set = ThreatEntrySet {
            rice_hashes: Some(RiceEncoded {
                rice_parameter: 2,
                num_entries: 1,
                first_value: "100".to_string(),
                encoded_data: b64(&[0b0000_1101]),
            }),
            ..Default::default()
        };

        assert_eq!(additions_in(&set).expect("rice"), vec![100, 107]);
    }

    #[test]
    fn test_a_first_value_that_is_not_a_number_is_refused() {
        let set = ThreatEntrySet {
            rice_hashes: Some(RiceEncoded {
                rice_parameter: 2,
                num_entries: 1,
                first_value: "not a number".to_string(),
                encoded_data: b64(&[0b0000_1101]),
            }),
            ..Default::default()
        };

        assert!(additions_in(&set).is_err());
    }

    #[test]
    fn test_asking_about_no_prefixes_asks_google_nothing() {
        // The ordinary case, and the reason the whole protocol is acceptable
        // here: no collision means no request at all, not a request with an
        // empty body.
        let answer = tokio_test::block_on(find_full_hashes("unused-key", &[], &[]));

        assert!(answer.expect("no request made").matches.is_empty());
    }
}

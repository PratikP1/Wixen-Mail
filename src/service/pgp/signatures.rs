//! Checking a PGP signature against the keys in the key manager.
//!
//! Two shapes of signed mail, one check. Inline PGP signs a block of text in
//! the body, the "cleartext signature framework"; PGP/MIME signs a whole MIME
//! part and carries the signature in a part of its own beside it. Either way
//! what comes back is one of [`PgpVerdict`]'s four answers, and nothing from
//! the crate crosses out.
//!
//! # Which key, and how "no key" is told from "does not hold"
//!
//! A signature names the key that made it, by its short identifier and, from
//! every program that has signed mail for years, by its fingerprint. That name
//! is what separates two answers a person does different things about. When no
//! key in the list is the one named, nothing here can judge the signature, and
//! importing the sender's key is the next step. When a key in the list is the
//! one named and the arithmetic fails, the words are not the words that key
//! signed. So the name is read first, and only the keys it names are tried.
//!
//! A key's subkeys are its parts: a key that signs with a subkey is named by
//! the subkey's identifier, and it is still that key, under its owner's name.
//!
//! # What this does not ask
//!
//! Whether the key has expired, been revoked, or belongs to whoever it says.
//! A signature that holds says the key made it, and the sentence that reports
//! it says no more than that.

use super::keys::{keys_in, listing_of};
use super::{KeyInYourList, PgpVerdict};
use pgp::composed::{
    CleartextSignedMessage, Deserializable, DetachedSignature, PublicOrSecret, SignedPublicKey,
};
use pgp::packet::Signature;
use pgp::types::KeyDetails;

/// Where a clearsigned block begins, and the line that ends it.
const SIGNED_BLOCK_BEGINS: &str = "-----BEGIN PGP SIGNED MESSAGE-----";
const SIGNATURE_ENDS: &str = "-----END PGP SIGNATURE-----";

/// Check the clearsigned block in `text` against these public keys.
///
/// The text to show is the signed words where `text` is that block and
/// nothing else, and `text` as it came otherwise: around words the signature
/// does not cover, the armour lines are the only thing saying which words it
/// does.
pub(super) fn verify_cleartext(text: &str, public_keys: &[String]) -> (PgpVerdict, String) {
    let as_it_came = || text.to_string();
    let Some(block) = the_signed_block(text) else {
        return (PgpVerdict::Damaged, as_it_came());
    };
    let Ok((message, _)) = CleartextSignedMessage::from_string(block) else {
        return (PgpVerdict::Damaged, as_it_came());
    };
    // Normalised to CRLF, which is what the signature covers; shown with the
    // line endings every other body here has.
    let signed = message.signed_text();
    let verdict = judged(
        message.signatures(),
        &keys_from(public_keys),
        signed.as_bytes(),
    );
    let shown = if block.trim() == text.trim() {
        signed.replace("\r\n", "\n")
    } else {
        as_it_came()
    };
    (verdict, shown)
}

/// Check a detached signature over exactly `content` against these public
/// keys.
///
/// Exactly: a PGP/MIME signature covers the signed part's bytes as they
/// arrived, CRLF endings and all, and the same words re-written by a program
/// on the way through fail the way changed words do.
pub(super) fn verify_detached(
    content: &[u8],
    signature_armour: &str,
    public_keys: &[String],
) -> PgpVerdict {
    let Ok((signature, _)) = DetachedSignature::from_string(signature_armour) else {
        return PgpVerdict::Damaged;
    };
    judged(
        std::slice::from_ref(&signature.signature),
        &keys_from(public_keys),
        content,
    )
}

/// From the line that begins a clearsigned block to the end of the line that
/// ends its signature, or `None` when either is missing.
fn the_signed_block(text: &str) -> Option<&str> {
    let from = &text[text.find(SIGNED_BLOCK_BEGINS)?..];
    let ends = from.find(SIGNATURE_ENDS)? + SIGNATURE_ENDS.len();
    Some(&from[..ends])
}

/// Every public key the armours hold, private keys given as their public
/// halves. An armour that holds no key adds nothing, and costs nothing the
/// others could answer.
fn keys_from(armours: &[String]) -> Vec<SignedPublicKey> {
    armours
        .iter()
        .flat_map(|armour| keys_in(armour))
        .map(|(key, _)| match key {
            PublicOrSecret::Public(public) => public,
            PublicOrSecret::Secret(secret) => secret.to_public_key(),
        })
        .collect()
}

/// The best answer any of the signatures earns, holding first.
///
/// A text carrying two signatures, one by a key in the list and one by a key
/// that is not, is still a text that key signed.
fn judged(signatures: &[Signature], keys: &[SignedPublicKey], content: &[u8]) -> PgpVerdict {
    signatures
        .iter()
        .map(|signature| judged_one(signature, keys, content))
        .min_by_key(how_much_it_says)
        .unwrap_or(PgpVerdict::Damaged)
}

/// How far down the order of what an answer says it sits: a holding
/// signature first, damage last.
fn how_much_it_says(verdict: &PgpVerdict) -> u8 {
    match verdict {
        PgpVerdict::Holds { .. } => 0,
        PgpVerdict::DoesNotHold { .. } => 1,
        PgpVerdict::NoKeyToCheckIt { .. } => 2,
        PgpVerdict::Damaged => 3,
    }
}

/// One signature, against the keys it names.
///
/// A signature that names no key at all is read as damaged: every program
/// that signs mail names its key, and one that does not can be matched to
/// nothing, so no sentence about a key would be true of it.
fn judged_one(signature: &Signature, keys: &[SignedPublicKey], content: &[u8]) -> PgpVerdict {
    let Some(key_id) = the_key_it_names(signature) else {
        return PgpVerdict::Damaged;
    };
    let named: Vec<&SignedPublicKey> = keys
        .iter()
        .filter(|key| names_a_part_of(signature, key))
        .collect();
    let Some(first) = named.first() else {
        return PgpVerdict::NoKeyToCheckIt {
            key_id: in_groups_of_four(&key_id),
        };
    };
    match named.iter().find(|key| holds_for(signature, key, content)) {
        Some(key) => PgpVerdict::Holds {
            whose: in_your_list(key),
        },
        None => PgpVerdict::DoesNotHold {
            whose: in_your_list(first),
        },
    }
}

/// The short identifier of the key a signature names, in capitals.
///
/// From its issuer subpacket, or from the fingerprint where only that is
/// given: the last sixteen digits of a version 4 fingerprint, the first
/// sixteen of a version 6 one, which is how each version defines it.
fn the_key_it_names(signature: &Signature) -> Option<String> {
    if let Some(id) = signature.issuer_key_id().first() {
        return Some(id.to_string().to_uppercase());
    }
    let fingerprint = format!("{:X}", signature.issuer_fingerprint().first()?);
    let id = match fingerprint.len() {
        64 => &fingerprint[..16],
        length => &fingerprint[length.checked_sub(16)?..],
    };
    Some(id.to_string())
}

/// Whether the signature names this key or one of its subkeys.
fn names_a_part_of(signature: &Signature, key: &SignedPublicKey) -> bool {
    names(signature, key)
        || key
            .public_subkeys
            .iter()
            .any(|subkey| names(signature, subkey))
}

fn names(signature: &Signature, part: &impl KeyDetails) -> bool {
    signature
        .issuer_key_id()
        .iter()
        .any(|id| **id == part.legacy_key_id())
        || signature
            .issuer_fingerprint()
            .iter()
            .any(|fingerprint| **fingerprint == part.fingerprint())
}

/// Whether the arithmetic holds against this key or one of its subkeys.
///
/// Only the answer is kept. The crate's reason for a refusal is written for
/// somebody reading a stack trace, and nothing of it is said or logged.
fn holds_for(signature: &Signature, key: &SignedPublicKey, content: &[u8]) -> bool {
    signature.verify(key, content).is_ok()
        || key
            .public_subkeys
            .iter()
            .any(|subkey| signature.verify(subkey, content).is_ok())
}

/// A key the way its row in the key manager names it.
fn in_your_list(key: &SignedPublicKey) -> KeyInYourList {
    let listing = listing_of(key, false);
    KeyInYourList {
        name: listing
            .user_ids
            .first()
            .cloned()
            .unwrap_or_else(|| format!("key {}", in_groups_of_four(&listing.key_id))),
        fingerprint: in_groups_of_four(&listing.fingerprint),
    }
}

/// Hexadecimal digits in groups of four, the way one is read to somebody and
/// the way a screen reader says it as groups rather than as one long word.
fn in_groups_of_four(digits: &str) -> String {
    digits
        .as_bytes()
        .chunks(4)
        .map(String::from_utf8_lossy)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::KeyInYourList;
    use super::super::for_tests::{alices_public_key, carols_public_key};
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    /// The bytes a fixture stands for.
    pub(in crate::service::pgp) fn armour(encoded: &str) -> String {
        let packed: String = encoded.split_whitespace().collect();
        String::from_utf8(STANDARD.decode(packed).expect("a fixture that decodes"))
            .expect("armour is text")
    }

    // ── Signed by GnuPG, not by the crate this module calls ──────────────
    //
    // The reason is `keys.rs`'s: a signature made and checked by one
    // implementation proves it agrees with itself. These were made by GnuPG
    // 2.4.9 with Carol's key, the one fixture key that signs (Alice's is
    // `ecEC`, which GnuPG refuses to sign with), in a home directory holding
    // her private key and Alice's public one:
    //
    // ```text
    // GNUPGHOME=/c/g18 gpg --batch --import carol_private.asc
    // printf 'Carol here. The minutes are attached, and the vote is on Friday.\n' > words.txt
    // GNUPGHOME=/c/g18 gpg --batch --local-user carol@example.com --clearsign \
    //     -o carol_clearsigned.asc words.txt
    // printf 'Content-Type: text/plain; charset=us-ascii\r\nContent-Transfer-Encoding: 7bit\r\n\r\nThe figures are final. Carol\r\n' > part.eml
    // GNUPGHOME=/c/g18 gpg --batch --local-user carol@example.com --detach-sign --armor \
    //     -o part.sig part.eml
    // gpg --verify carol_clearsigned.asc   -> Good signature from "Carol Example <carol@example.com>"
    // gpg --verify part.sig part.eml       -> Good signature from "Carol Example <carol@example.com>"
    // tr -d '\r' < part.eml > part_lf.eml
    // gpg --verify part.sig part_lf.eml    -> BAD signature from "Carol Example <carol@example.com>"
    // ```
    //
    // Base64 in the source, for the reason the keys are: an armoured block
    // held as text has its line endings rewritten by any tool that touches the
    // file, and the part's CRLF endings are what the detached signature covers.

    /// Carol's clearsigned text, as `gpg --clearsign` wrote it.
    pub(in crate::service::pgp) const CAROL_CLEARSIGNED: &str = "
        LS0tLS1CRUdJTiBQR1AgU0lHTkVEIE1FU1NBR0UtLS0tLQpIYXNoOiBTSEE1MTIKCkNhcm9s
        IGhlcmUuIFRoZSBtaW51dGVzIGFyZSBhdHRhY2hlZCwgYW5kIHRoZSB2b3RlIGlzIG9uIEZy
        aWRheS4KCi0tLS0tQkVHSU4gUEdQIFNJR05BVFVSRS0tLS0tCgppSWdFQVJZS0FEQVdJUVNO
        NU43c05uMElaamVUU2h4U3RjQkRvc1pCY3dVQ2FyaitOaEljWTJGeWIyeEFaWGhoCmJYQnNa
        UzVqYjIwQUNna1FVclhBUTZMR1FYT21VZ0VBMThYbXFPc0NiUkFuZk9JUkhPdDlFSHY4amhT
        RE04dzAKL1N0dDk5aU5XQTBCQU84cGFhNmFkdWlVUlBmZEMxVlZqVTZsbVpxV0tINkJpZzhK
        a2lma1gzWUUKPTFxL3UKLS0tLS1FTkQgUEdQIFNJR05BVFVSRS0tLS0tCg==";

    /// The words Carol signed, as a reader shows them: `words.txt` ended in a
    /// line break, and the cleartext framework signs it.
    pub(in crate::service::pgp) const WHAT_CAROL_SIGNED: &str =
        "Carol here. The minutes are attached, and the vote is on Friday.\n";

    /// A MIME part with CRLF endings, the bytes Carol's detached signature
    /// covers.
    pub(in crate::service::pgp) const CAROLS_SIGNED_PART: &str = "
        Q29udGVudC1UeXBlOiB0ZXh0L3BsYWluOyBjaGFyc2V0PXVzLWFzY2lpDQpDb250ZW50LVRy
        YW5zZmVyLUVuY29kaW5nOiA3Yml0DQoNClRoZSBmaWd1cmVzIGFyZSBmaW5hbC4gQ2Fyb2wN
        Cg==";

    /// Her detached signature over it, as `gpg --detach-sign --armor` wrote it.
    pub(in crate::service::pgp) const CAROLS_DETACHED_SIGNATURE: &str = "
        LS0tLS1CRUdJTiBQR1AgU0lHTkFUVVJFLS0tLS0KCmlJZ0VBQllLQURBV0lRU041TjdzTm4w
        SVpqZVRTaHhTdGNCRG9zWkJjd1VDYXJqK054SWNZMkZ5YjJ4QVpYaGgKYlhCc1pTNWpiMjBB
        Q2drUVVyWEFRNkxHUVhQaExBRUFtN0dEMEJNdG94cmNGdHlGeDVXVllFcXVKQU5qZGREWApF
        Q3BrZWdBZkF1MEEvMVhBNjRMWmpySFJvZUhTbVRYTE50WFQxTkw0RHo1NjlGTVYrQUZ1YUVV
        QQo9RGdYRwotLS0tLUVORCBQR1AgU0lHTkFUVVJFLS0tLS0K";

    /// Carol's key as its row in the key manager names it.
    fn carol() -> KeyInYourList {
        KeyInYourList {
            name: "Carol Example <carol@example.com>".to_string(),
            fingerprint: "8DE4 DEEC 367D 0866 3793 4A1C 52B5 C043 A2C6 4173".to_string(),
        }
    }

    /// The key id Carol's signatures name, as `gpg --list-keys` shows it,
    /// grouped.
    const CAROLS_KEY_ID: &str = "52B5 C043 A2C6 4173";

    #[test]
    fn test_a_clearsigned_text_holds_against_its_signers_key_and_names_her() {
        // Two implementations agreeing: GnuPG signed it and this checks it,
        // against the public key GnuPG exported. The words come back without
        // the armour lines around them, which is what somebody reads.
        let checked = verify_cleartext(&armour(CAROL_CLEARSIGNED), &[carols_public_key()]);

        assert_eq!(
            checked,
            (
                PgpVerdict::Holds { whose: carol() },
                WHAT_CAROL_SIGNED.to_string()
            )
        );
    }

    #[test]
    fn test_one_letter_changed_after_signing_does_not_hold() {
        // What a signature is for. The key is the right one and the words are
        // not the ones it signed, so the answer names the key and says it does
        // not hold, rather than that nothing could be checked.
        let changed = armour(CAROL_CLEARSIGNED).replace("Friday", "Monday");

        let (verdict, _) = verify_cleartext(&changed, &[carols_public_key()]);

        assert_eq!(verdict, PgpVerdict::DoesNotHold { whose: carol() });
    }

    #[test]
    fn test_with_no_key_for_the_signer_it_names_the_key_it_would_need() {
        // Nothing is in the list, so nothing could be checked. Not a failed
        // check: importing Carol's key turns this into "it holds", so the
        // answer names the key id that would.
        let (verdict, _) = verify_cleartext(&armour(CAROL_CLEARSIGNED), &[]);

        assert_eq!(
            verdict,
            PgpVerdict::NoKeyToCheckIt {
                key_id: CAROLS_KEY_ID.to_string()
            }
        );
    }

    #[test]
    fn test_with_only_somebody_elses_key_it_still_names_the_key_it_would_need() {
        // A key in the list that is not the signer's is not a key that says
        // the signature failed. Alice's key did not make it and cannot judge
        // it.
        let (verdict, _) = verify_cleartext(&armour(CAROL_CLEARSIGNED), &[alices_public_key()]);

        assert_eq!(
            verdict,
            PgpVerdict::NoKeyToCheckIt {
                key_id: CAROLS_KEY_ID.to_string()
            }
        );
    }

    #[test]
    fn test_words_outside_the_signed_block_leave_the_text_as_it_came() {
        // Words outside the block are not covered by the signature. Taking
        // the armour lines away would leave the covered words and the others
        // looking the same, under a sentence saying the signature holds.
        let text = format!(
            "Forwarded by the list.\n\n{}\n-- \nThe list footer\n",
            armour(CAROL_CLEARSIGNED)
        );

        let checked = verify_cleartext(&text, &[carols_public_key()]);

        assert_eq!(checked, (PgpVerdict::Holds { whose: carol() }, text));
    }

    #[test]
    fn test_a_clearsigned_block_cut_short_is_damaged_and_shown_as_it_came() {
        let whole = armour(CAROL_CLEARSIGNED);
        let cut = whole[..whole.len() / 2 + 40].to_string();

        let checked = verify_cleartext(&cut, &[carols_public_key()]);

        assert_eq!(checked, (PgpVerdict::Damaged, cut));
    }

    #[test]
    fn test_a_detached_signature_holds_over_the_exact_bytes_it_signed() {
        let part = armour(CAROLS_SIGNED_PART);

        assert_eq!(
            verify_detached(
                part.as_bytes(),
                &armour(CAROLS_DETACHED_SIGNATURE),
                &[alices_public_key(), carols_public_key()]
            ),
            PgpVerdict::Holds { whose: carol() }
        );
    }

    #[test]
    fn test_the_same_part_with_bare_line_feeds_does_not_hold() {
        // A PGP/MIME signature covers the part's bytes with CRLF endings, and
        // a check run over the part as a program re-wrote it fails exactly
        // the way a changed message does. So the bytes handed here have to be
        // the bytes as they arrived, and this is the case that says so.
        let part = armour(CAROLS_SIGNED_PART).replace("\r\n", "\n");

        assert_eq!(
            verify_detached(
                part.as_bytes(),
                &armour(CAROLS_DETACHED_SIGNATURE),
                &[carols_public_key()]
            ),
            PgpVerdict::DoesNotHold { whose: carol() }
        );
    }

    #[test]
    fn test_a_truncated_detached_signature_is_damaged() {
        let signature = armour(CAROLS_DETACHED_SIGNATURE);
        let cut = &signature[..signature.len() / 2];

        assert_eq!(
            verify_detached(
                armour(CAROLS_SIGNED_PART).as_bytes(),
                cut,
                &[carols_public_key()]
            ),
            PgpVerdict::Damaged
        );
    }

    #[test]
    fn test_a_detached_signature_with_no_key_for_it_names_the_key() {
        assert_eq!(
            verify_detached(
                armour(CAROLS_SIGNED_PART).as_bytes(),
                &armour(CAROLS_DETACHED_SIGNATURE),
                &[alices_public_key()]
            ),
            PgpVerdict::NoKeyToCheckIt {
                key_id: CAROLS_KEY_ID.to_string()
            }
        );
    }

    #[test]
    fn test_a_list_holding_something_that_is_not_a_key_still_checks_with_the_rest() {
        // One unreadable entry in the list must not cost the answer every
        // other key could give.
        assert_eq!(
            verify_detached(
                armour(CAROLS_SIGNED_PART).as_bytes(),
                &armour(CAROLS_DETACHED_SIGNATURE),
                &["not a key".to_string(), carols_public_key()]
            ),
            PgpVerdict::Holds { whose: carol() }
        );
    }
}

//! Checking a PGP signature against the keys in the key manager.
//!
//! Two shapes of signed mail, one check. Inline PGP signs a block of text in
//! the body, the "cleartext signature framework"; PGP/MIME signs a whole MIME
//! part and carries the signature in a part of its own beside it. Either way
//! what comes back is one of [`PgpVerdict`]'s four answers, and nothing from
//! the crate crosses out.

use super::PgpVerdict;

/// Check the clearsigned block in `text` against these public keys.
pub(super) fn verify_cleartext(text: &str, _public_keys: &[String]) -> (PgpVerdict, String) {
    (
        PgpVerdict::NoKeyToCheckIt {
            key_id: String::new(),
        },
        text.to_string(),
    )
}

/// Check a detached signature over exactly `content` against these public
/// keys.
pub(super) fn verify_detached(
    _content: &[u8],
    _signature_armour: &str,
    _public_keys: &[String],
) -> PgpVerdict {
    PgpVerdict::NoKeyToCheckIt {
        key_id: String::new(),
    }
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

    /// The words Carol signed, as a reader shows them.
    pub(in crate::service::pgp) const WHAT_CAROL_SIGNED: &str =
        "Carol here. The minutes are attached, and the vote is on Friday.";

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

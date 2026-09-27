---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 20
subsystem: service::pgp, service::protocols::smtp, service::outward, Cargo.toml
status: complete
tags: [pgp, openpgp, signing, encryption, rfc-3156, GAP-05, "#52"]
requires: [13-16, 13-17.1, 13-18, 13-19]
provides:
  - service::pgp::sign_detached, encrypt_for, Recipient, Sending
  - keys::detached_signature, the signature made beside the held passphrases
  - keys::the_primary_self_signature, the reading listing_of already made
  - smtp::Protection::PgpSigned, PgpEncrypted, PgpSignedAndEncrypted and their RFC 3156 wrapping in build_message
  - rand08, rand 0.8 named for rPGP's builder bounds, adding no package
affects: [13-21]
tech-stack:
  added: []
  patterns:
    - "a typed passphrase never leaves keys.rs: the one step that needs it, signing, is made there, and sending.rs asks for the result"
    - "every key is read and its usable part found before anything is built, so a message never goes out encrypted to some of its recipients"
key-files:
  created:
    - src/service/pgp/sending.rs
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-20-SUMMARY.md
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/service/outward.rs
    - src/service/pgp/mod.rs
    - src/service/pgp/keys.rs
    - src/service/protocols/smtp.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/USER_GUIDE.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "Pratik's answer (b) of 2026-09-24, carried by the brief, confirmed the rand08 line; task 1's checkpoint was answered yes and did not stop."
  - "Signed and encrypted is RFC 3156 section 6.1, the signed entity encrypted, the same order as S/MIME's arm; no separate sign_and_encrypt function, since nothing would call it."
  - "The detached signature is made in keys.rs, because test_no_public_item_of_this_module_hands_out_a_passphrase refuses a pub line naming the passphrase type."
  - "Sending has eight outcomes, not the plan's four: the store failing, a key that does not sign, a sender's key that cannot be encrypted to, and a last-step refusal are each their own answer."
metrics:
  duration: about 1 hour 55 minutes on 2026-09-27, 17 minutes of it one re-measure of eight records in the background
  completed: 2026-09-27
estimate:
  tokens: 115000
  tasks: 4
actuals:
  tokens: 21000
  tasks: 4
  commits: 6
---

# Phase 13 Plan 20: OpenPGP signing and encrypting, the service half Summary

Wixen Mail can now sign a message with an OpenPGP key and encrypt one, beneath the composer.
`service::pgp::sign_detached` signs the exact bytes it is handed with the key's signing part,
the primary where its flags allow and a signing subkey otherwise, over SHA-256; a locked key
with no passphrase held answers `TheKeyIsLocked`. `service::pgp::encrypt_for` encrypts as SEIPD
version 1 with AES-256 to every recipient's key and to the sender's own, using only the public
half, so a locked sender needs no passphrase to encrypt. `build_message` wraps the body per RFC
3156: `multipart/signed` with `protocol="application/pgp-signature"`, `micalg="pgp-sha256"` and
`signature.asc`; `multipart/encrypted` with `protocol="application/pgp-encrypted"`, the
`Version: 1` control part and `encrypted.asc`; signed and encrypted signs first. Any outcome
but a built one stops the build with a sentence, so nothing goes out unprotected. GnuPG reads
what this writes. Nothing a person does reaches it until 13-21 (ledger 656); nothing sent here
has been opened by another mail program (ledger 657). The `cms` comment in `Cargo.toml` no
longer says something false.

`actuals.tokens` is chars/4 over the lines added under `src`, `guards` and `Cargo.toml`
against `main` at `039259cd`, 41,328 characters, and the pages, ledger and this summary, about
10,700.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `10ecfe7c` | build | `rand08 = { package = "rand", version = "0.8" }`, its census name, the `cms` comment corrected | 737 s, `all` |
| `a73e822e` | red | 14 cases in `service::pgp::sending` on GnuPG-made keys, Erin's sign-only key new | 131 s |
| `14c45775` | green | `sign_detached`, `encrypt_for`, `keys::detached_signature`; 2 records new | 198 s |
| `3cc5c05d` | red | 4 cases in `smtp::tests::pgp_protected` and the count check | 115 s |
| `4fc540dd` | green | the three PGP arms and their wrapping; 1 record new, 7 re-measured | 190 s |
| this commit | docs | the changelog, the guide, the ledger, this summary, the four marks | |

**Task 1, the checkpoint.** Answered by the brief: Pratik's (b) of 2026-09-24, "rand 0.8 as a
renamed direct dependency confirmed", recorded in the phase README as decision 53. The
checkpoint did not stop.

**Task 2, the manifest.** `git diff --stat HEAD~1 -- Cargo.lock` on `10ecfe7c`: one file, one
insertion, `"rand 0.8.7"` added to `wixen-mail`'s own dependency list;
`git diff HEAD~1 -- Cargo.lock | grep -c '^+\[\[package\]\]'` is 0. The locked 0.8.7 was
already built with its default features (`libc`, `rand_chacha`), so the default line adds
nothing. `grep -c 'package = "rand", version = "0.8"' Cargo.toml` is 1; the joined-lines read
for "only ever published pre-release versions" is 0. The hook's mode line: `check.sh: mode
all`, and `check.sh: all passed after 736 s`, `cargo audit` among it ("No advisory outside
.cargo/audit.toml").

`dependency-audit`, closing research assumption A7: the vendored `rand-0.8.7/Cargo.toml` says
`license = "MIT OR Apache-2.0"` and the crate ships `LICENSE-MIT` and `LICENSE-APACHE`; its
changelog dates 0.8.7 to 2026-07-02. The repository's last releases, read with `gh api
repos/rust-random/rand/releases`: `0.10.3` on 2026-09-20 and `0.8.8` on 2026-08-25, "fix no-std
+ serde1 build", which this program does not need. The readers of `Cargo.toml` under `tests/`
and `scripts/`: `house_style.rs` (the em dash and machinery readings, and `every_dependency`),
`installer.rs` (the version line and the release metadata), `the_words_that_say_nothing.rs`
(the six words), `build-installer.sh` and `which-checks.sh` (the version line, and a manifest
change answering `all`). None reads a dependency by name except the two `every_dependency`
readings, and `rand08` is on the census's list; no page names it.

**Test counts, taken on 2026-09-27.** `cargo test --lib service::pgp::` 64 (50 at the start;
the 14 new are in `sending.rs`). `service::protocols::smtp::` 48 (44). `service::outward::` 38
(38). `house_style` 74. The whole library ran three times by hand for the guard records:
8,251 tests run before the smtp cases and 8,255 after, one ignored each time.

**Acceptance readings.** `grep -rln 'use pgp::\|pgp::composed' src --include=*.rs` lists
`keys.rs`, `mod.rs`, `sending.rs` and `signatures.rs` under `src/service/pgp/` and nothing
else; `mod.rs` matches only in its own test's comments. `grep -c 'application/pgp-encrypted'
src/service/protocols/smtp.rs` is 1, the constant both the protocol and the control part's type
are written from.

**GnuPG, once, outside the tests.** A temporary ignored test wrote Carol's message to Alice,
Carol's detached signature, the part it covers and the two keys into the scratchpad; it was
taken out again before anything was committed. GnuPG 2.4.9 in a short home directory holding
Alice's private key and Carol's public key:

```text
GNUPGHOME=/c/g20a gpg --batch --import alice_private.asc carol_public.asc
GNUPGHOME=/c/g20a gpg --batch --decrypt to_alice.asc > decrypted.eml
    gpg: encrypted with cv25519 key, ID 404357E8E13851E0, created 2026-09-27
          "Carol Example <carol@example.com>"
    gpg: encrypted with rsa2048 key, ID A97E7BB74101FB3E, created 2026-09-06
          "Alice Example <alice@example.com>"
cmp decrypted.eml part.eml   -> identical
GNUPGHOME=/c/g20a gpg --batch --list-packets to_alice.asc
    :pubkey enc packet: version 3, algo 1, keyid A97E7BB74101FB3E
    :pubkey enc packet: version 3, algo 18, keyid 404357E8E13851E0
    :encrypted data packet:
            mdc_method: 2
    :literal data packet:
GNUPGHOME=/c/g20a gpg --batch --verify part.sig part.eml
    gpg: Good signature from "Carol Example <carol@example.com>" [unknown]
tr -d '\r' < part.eml > part_lf.eml
GNUPGHOME=/c/g20a gpg --batch --verify part.sig part_lf.eml
    gpg: BAD signature from "Carol Example <carol@example.com>" [unknown]
```

`mdc_method: 2` is GnuPG's way of printing SEIPD version 1; the sender, Carol, is the second
PKESK, which is what makes the Sent copy hers to open.

## Guard records

| Record | What |
|--------|------|
| an openpgp message is encrypted to its sender too | new, `sending.rs`, 1 red |
| an openpgp signature covers the bytes it is handed, line endings and all | new, `keys.rs`, 2 red |
| an openpgp message whose key is missing does not go out unwrapped | new, `smtp.rs`, 1 red |
| a blind copy reaches the server, a sent message carries an identifier, a picture leaves the body, the plain half says what its pictures were, signed over the bytes it goes out with, sealed for its sender too, does not go in the clear | flagged by the count check when `smtp.rs` went from 44 tests to 48, re-measured, each unchanged |

Three records new, each measured by hand with the whole library; the arrived-since line at
the head of `guards/guards.toml` went from 389 to 392. One `--remeasure` call named the seven
flagged records and the new `smtp.rs` one, 1,042 seconds in the background: "All 8 guards
redden exactly the tests their records name." Anchors held: `smtp.rs`'s header half, the Bcc
loop, `sign_detached(as_it_sits, own)`, `everyone.push(own.der.clone())` and the S/MIME sealed
arm are untouched and each still names one place; `keys.rs`'s four anchors are untouched, and
the new `held.get(&fingerprint_of(key))` sits in a tuple, not in the anchored `.filter_map`
line.

## Deviations from Plan

**1. [Premise] Carol signs, not Alice.** Alice's fixture key is `ecEC`: it encrypts and
certifies and does not sign, as 13-18 found. Carol, Ed25519 with a Curve25519 subkey, signs
and sends; Alice's key is the one that answers `YourKeyCannotSign`.

**2. [Rule 2] The detached signature is made in `keys.rs`.** The plan put the three functions
in `sending.rs` behind the boundary. Making the held passphrases reachable from `sending.rs`
turned `test_no_public_item_of_this_module_hands_out_a_passphrase` red, because a `pub(super)`
line would name the passphrase type, and that guard is right: the passphrase is the one thing
that must not travel. So the step that needs it, `keys::detached_signature`, sits beside the
map; `sending.rs` finds the key and asks.

**3. [Shape] No `sign_and_encrypt`.** Task 4 asks for "the signed entity encrypted", RFC 3156
section 6.1, which is `sign_detached` then `encrypt_for` around a `multipart/signed` built in
between; a one-pass combined function would have had no caller. Task 3's "a signed-and-encrypted
message opens and its inner signature holds" is therefore task 4's
`test_a_pgp_signed_and_encrypted_message_opens_to_a_signed_one_that_holds`, through the
reader's own path.

**4. [Rule 2] Eight outcomes, not four.** `Sending` as planned had `Built`, `TheKeyIsLocked`,
`NoPrivateKey` and `ARecipientHasNoKey`. A store that will not give a key back, or a stored
key that no longer reads, is `TheKeyCouldNotBeRead` for `keys::open`'s reason: saying there is
no key would be a guess. A key whose flags give no signing part is `YourKeyCannotSign`; a
sender's key that cannot be encrypted to is `YourKeyCannotBeEncryptedTo`, since sending without
it makes the Sent copy unreadable; a refusal from the crate after every key was accepted is
`CouldNotBeBuilt`, which no input this program makes is known to reach and no test reaches.
`TheKeyIsLocked` carries 13-17.1's `LockedKey`, the fingerprint beside `whose`, since 13-21
asks for the passphrase by fingerprint.

**5. [Shape] `Recipient` carries an address and a key.** The plan wrote `recipients: &[String]`
and then asked for the address in the refusal; the build cannot name an address it was never
handed. `Protection`'s PGP arms carry `Vec<Recipient>` and the sender's fingerprint.

**6. [Scope] A new GnuPG fixture, Erin.** No fixture key could not be encrypted to. Erin's is
Ed25519, sign and certify only, made by GnuPG 2.4.9 in `/c/g20` with the commands beside the
constant in `sending.rs`, listed `scSC` with no subkey.

**7. [Rule 1] The red's expected `micalg` was unquoted.** lettre writes
`micalg="pgp-sha256"`, which MIME allows; the red asserted `micalg=pgp-sha256` and I did not
check that value before the red, which the brief asks. The green corrected the case and its
commit says so.

**8. [Rule 1] A glob `use` in `smtp.rs` named the crate's word.** `use pgp::Sending::*` inside
`built` matched `test_no_caller_outside_this_module_names_the_crate`'s reading, since the
module is imported as `pgp`. Caught by the acceptance grep before the commit; `Sending` is
imported by name instead.

**9. [Scope] `docs/USER_GUIDE.md` was not in the plan's files.** The key manager's list said
"Sending encrypted mail is not built yet", which stopped being true; it now says sending
signed or encrypted mail is not offered yet. The program's own box never said it.

**10. [Brief] The banned tool's name, once.** A read-only command over the vendored `rand`
crate began with a do-nothing assignment carrying the banned name; the tool did not run and
nothing was written. It came after reading 13-18's and 13-19's reports of the same shape.
Logged as observation 0867.

### Found and left

- A key with no key-flags subpacket is answered as unable to sign or be encrypted to, where
  GnuPG goes by its algorithm. `listing_of` reads flags the same way. In ledger 657 for phase
  14, with a real correspondent's key.
- The encrypted part's body is armour; lettre chooses its transfer encoding, which for ASCII
  armour is 7bit. The signed part's encoding is lettre's choice too, as for S/MIME, and nothing
  here protects trailing whitespace in a signed part, which RFC 3156 warns a transport may
  alter. Untried against a real server that rewrites whitespace; phase 14's ledger 657 covers
  the signed message read by another program.

## Threat Flags

None beyond the register. T-13-20-SC: confirmed by Pratik, the lock diff one line in this
package's own list, the licence read from the vendored crate. T-13-20-01: any outcome but
`Built` stops `build_message`, and a record on the missing-key case. T-13-20-02: the CRLF bytes
between the boundaries signed, and a record on the LF form. T-13-20-03: every refusal is a
sentence chosen here, `Sending` carries no crate text, and the passphrase stays in `keys.rs`.
T-13-20-04: accepted, as planned.

## Known Stubs

`service::pgp::sign_detached`, `encrypt_for` and `smtp::Protection`'s three PGP arms have no
non-test caller: `mail_controller::outgoing` always sends `Protection::Plain`, and nothing looks
up a recipient's public key by address. That is this plan's scope, said in the changelog and
in ledger 656, which 13-21's premise 0 names as the entry its wiring closes.
`Sending::CouldNotBeBuilt` is an answer no test reaches, deviation 4.

## Ledger

Opened: 656 (`stub`, reached only by tests until 13-21), 657 (`unrun-verify`, phase 14, a
message signed or encrypted here opened by Thunderbird and Proton Mail, section 6.1's layout,
SEIPD version 2, and keys with no flags). Both halves of each; 593 open of 657.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `10ecfe7c`, `a73e822e`, `14c45775`, `3cc5c05d`, `4fc540dd`: in `git log` on
  `13-20-openpgp-signing-and-encrypting`.
- `src/service/pgp/sending.rs` exists.

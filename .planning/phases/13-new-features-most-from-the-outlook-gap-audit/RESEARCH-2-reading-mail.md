# Phase 13, group 2: reading mail - Research

**Researched:** 2026-09-24, against `main` at `630e2a67` (phase 12 closed), version
`1.0.0-alpha.1`, `guards/guards.toml` holding 1,088 records by the TOML reader.
**Requirements:** GAP-04 meeting invitations (#50), GAP-05 S/MIME and PGP/MIME read and
send (#52), GAP-03 a PGP key manager (#49).
**Domain:** iCalendar invitations (iTIP), S/MIME (CMS) through Windows CryptoAPI, OpenPGP
through rPGP, the Windows credential store.
**Confidence:** HIGH for what the tree holds and for the three probes run; MEDIUM for the
plan shapes; LOW where marked `[ASSUMED]`.

How to read the tags. `[STATED: file:line]` is something read in the tree this session.
`[MEASURED: ...]` is a command or probe run this session, with what it printed.
`[CITED: url]` is from a primary source outside the tree. `[DERIVED]` is an inference from
stated facts, with the reasoning beside it. `[ASSUMED]` is training knowledge not checked
today and needs confirming before it becomes a decision.

## User Constraints

No `CONTEXT.md` exists for phase 13; the directory held nothing when this was written
(`ls .planning/phases/13-new-features-most-from-the-outlook-gap-audit/` answered an empty
directory at 20:19). What binds this group instead:

- **Pratik's order inside the phase, confirmed 2026-09-24** (from the brief): keyboard
  basics (print, undo and redo), then reading mail (invitations, encrypted mail, PGP key
  manager), then provider features, then automation, then the rest of import and export.
  This group is the second block.
- **GAP-05's own `[D]` line** says "Reading first, then verifying, then sending; ... the
  key manager (GAP-03) before it" `[STATED: .planning/REQUIREMENTS.md:5807-5812]`. The
  confirmed order names encrypted mail before the key manager. Both hold if the key
  manager lands after reading and before verifying and sending, which is the order
  proposed below; question 5 asks Pratik to confirm it.
- **CLAUDE.md rules that decide designs here:** secrets go to the OS credential store and
  never the database; nothing sensitive in `message_cache.db`; schema changes additive;
  every setting reachable from the settings screen; network code tested against parsing
  and error mapping only; red and green TDD with `workflow.tdd_mode` on; anything expected
  to draw bug reports is marked experimental where the person sees it; mnemonic letters
  are one shared set per window; a name is verified on both channels at the handle that
  takes focus.
- **Every package is Pratik's to confirm before it is installed.** This group needs one
  new direct `Cargo.toml` line and no new crate (section 4).

## Summary

The three requirements need almost nothing new from outside the tree. rPGP 0.20 is
already in `Cargo.toml` and already covers signing, detached and cleartext verification,
encryption to public keys, key export and multi-key parsing. Windows' CryptoAPI message
functions (`CryptDecryptMessage`, `CryptVerifyDetachedMessageSignature`,
`CryptSignMessage`, `CryptEncryptMessage`) are already reachable through the `windows`
0.62.2 crate with the `Win32_Security_Cryptography` feature the tree already switches on.
A probe in the scratchpad showed all four working on this machine against OpenSSL 3.5.7
output, in both directions, with a private key imported into memory only, which is the
same technique the tree's own tests already use (`holding_only_in_memory`). So the
comment at `signed_mail.rs:3225-3236` that decryption "is not built" because it cannot be
tested without a real certificate installed is no longer a reason: it can be tested now.

Three findings change what the plans must do before they add anything:

1. **The PGP key import cannot store an ordinary RSA key on a real Windows machine.**
   `keyring` 4.1.5 on Windows encodes a password as UTF-16 and refuses anything over 2,560
   bytes (1,280 characters). The tree's own GnuPG-made RSA-2048 fixture is 1,836 characters
   armoured, 3,672 bytes as UTF-16. The test credential store has no limit, so every test
   passes. A real key with an encryption subkey is larger still. The key manager has to
   decide where keys live before it can list several of them (question 1).
2. **An invitation's UID matches nothing a Google or Microsoft calendar sync wrote.** The
   sync stores the provider's own event id, and nothing in `src/` reads Google's `iCalUID`
   or Graph's `iCalUId`. So "an update reaching the calendar" cannot find the meeting on
   those accounts until synced events carry the iCalendar UID.
3. **Decrypted mail reopens a security argument the tree already wrote down.** The
   accepted advisory RUSTSEC-2023-0071 in `.cargo/audit.toml:77-203` rests on two
   narrowing facts this group changes (S/MIME decryption refused; import under File) and
   names an expiry: a fetch whose presence depends on whether a body decrypted. A
   decrypted PGP/MIME or S/MIME message is usually HTML, and the tree fetches remote
   pictures by default since #28. Decrypted content must never fetch anything, which is
   also the EFAIL defence.

**Primary recommendation:** eleven plans, no new crate, one new direct `Cargo.toml` line
(`rand` 0.8, already in the lock). Invitations first (three plans and a small fourth),
then S/MIME reading through CryptoAPI, then PGP/MIME reading, then the key storage fix
and the manager, then PGP signature checks, then sending in three plans.

## Architectural Responsibility Map

This is a Windows desktop program, so the tiers are the project's own layers.

| Capability | Primary tier | Secondary tier | Rationale |
|---|---|---|---|
| What an invitation asks, what changed | `application` (pure) | `service::caldav` line reader | Values in, values out, already the shape of `invitations.rs` |
| Matching an invitation to a synced event | `data::message_cache::calendar` | `application::calendar` sync mapping | The UID has to be stored where events are stored |
| Saying it before the body, on six surfaces | `application::reading_a_message` | `presentation::reader_text` | One decision for every surface, #51's lesson |
| Answer buttons | `presentation` (wx_reader, formatted window) | `application::answering` | Thin wiring over a decision that already exists |
| S/MIME open, verify, sign, encrypt | `service::signed_mail` behind `CertificateStore` | Windows CryptoAPI | The private key never leaves the Windows store |
| PGP open, verify, sign, encrypt, keys | `service::pgp` (the only place naming the crate) | `service::secret_store` | `test_no_caller_outside_this_module_names_the_crate` holds it |
| Private key storage | `service::secret_store` (credential store) | none | CLAUDE.md, secrets rule |
| Public keys and certificates of others | `data::message_cache` (additive table) | none | Not secret; question 7 |
| Protecting outgoing mail | `service::protocols::smtp::build_message` | outbox column | Every send goes through `build_message` |

## 1. What the issues ask

No issue in this group carries a comment. Each was filed by the planner on 2026-09-16 from
the tester's list and the Outlook gap audit `[MEASURED: gh issue view 49, 50, 52 --json
...,comments: every comments array empty]`.

**#49 (GAP-03).** The tester on 2026-09-15: "While there is an import PGP key option,
there is no way to manage these keys." Expected: a manager "reachable from Tools beside
Import" that lists user ids, key id, fingerprint, creation and expiry, private or public;
imports a private or public key from a file, an attachment or pasted text; removes one
"with a confirmation that says what will stop opening"; exports a public key to a file or
the clipboard; and "says plainly, where a person will read it, what this build can and
cannot do with a key". "Passphrase-protected keys are a decision of their own." Held until
the sweep and mutation runs were read.

**#50 (GAP-04).** Audited as partial: answering is built and wired, reading and
update and cancellation are not. Nine points: (1) the reader neither shows nor announces
the meeting; (2) cancellations do nothing to the calendar; (3) updates apply only if
answered again; (4) `what_pressing_it_will_do` is unwired and the answer given is not
stored; (5) only `text/calendar` is found, not `application/ics`, and no raw MIME fixture
carries a calendar part; (6) one day of a series cannot be answered; (7) no shortcut, no
context menu entry, never greyed; (8) the reply carries no `In-Reply-To`; (9)
`docs/development/requirements-backlog.md:102` overclaims. "Items 1 to 3 are ... the
plan; 5 to 9 go with it."

**#52 (GAP-05).** Audited as partial: S/MIME signatures are verified, inline PGP opens
with one key. Seven points: (1) read S/MIME encrypted; (2) read PGP/MIME; (3) verify PGP
signatures; (4) send signed; (5) send encrypted; (6) signed mail stored before originals
were kept reads as unsigned; (7) `docs/comparison.md:46` says "there is no OpenPGP at
all". "Items 1 and 3 are the reading half ... and come first; 4 and 5 need the sending
path proven live first." Sending was proven on 2026-09-18 (phase 12 README, group 7 row),
so that condition is met `[STATED: .planning/phases/12-the-editors-and-what-the-alpha-still-owes/README.md:115]`.

## 2. What the tree already has

### GAP-04, invitations

| What | Where | Notes |
|---|---|---|
| What a document asks, by `METHOD` | `src/application/invitations.rs:73` `what_it_asks` | `REQUEST`, `CANCEL`, `REPLY`, else `SomethingElse` `[STATED: invitations.rs:56-80]` |
| The meeting read out of a document | `invitations.rs:84-157` `Invitation`, `read_the_invitation` | uid, version (`SEQUENCE`), summary, starts, ends, all day, zone, location, organiser, guests. Reads through `ical_subscription::parse_ics` and `caldav`'s line reader, not the `icalendar` crate `[STATED: invitations.rs:42-51, 159-178]` |
| New meeting, change, or nothing new | `invitations.rs:468-512` `AlreadyOnTheCalendar`, `WhatChanged`, `what_changed` | Called only from `answering.rs:350` and tests `[MEASURED: grep what_changed(]` |
| Ten reasons not to answer, each worded | `src/application/answering.rs:67-155` `CannotAnswer`, `:164` `whether_it_can_be_answered` | Cancellation refused at `:171` with "This meeting has been called off, so there is nothing to answer." |
| The sentence before pressing | `answering.rs:275` `what_pressing_it_will_do` | Called by tests only (#50 point 4) |
| Finding the part | `answering.rs:712` `AN_INVITATION_ARRIVES_AS = "text/calendar"`, `:729` `the_invitation_a_message_carries` | First `text/calendar` part only |
| One day of a series refused | `answering.rs:674` `names_one_day_of_a_series` | reads `RECURRENCE-ID` |
| Filing on the calendar | `src/application/answered_meetings.rs:162` `file_the_answer` | matches by `get_event_by_provider_id(account, uid)`, writes `answered_version` |
| Calendar store | `src/data/message_cache/calendar.rs:391` `get_event_by_provider_id`, `:430` `the_version_answered_here`, `:460` `remember_the_version_answered`, `:599` `delete_calendar_event` (records a deletion to push), `:649` `drop_synced_calendar_event` | `answered_version INTEGER` added by `ensure_column_exists` at `src/data/message_cache/mod.rs:3182` |
| Event row | `src/data/message_cache/mod.rs:1010-1050` `CalendarEventEntry` | `status` is a string, "confirmed", "tentative", "cancelled" (`:1027`); `show_as` "busy", "free", "tentative", "oof" (`:1040`) |
| Menu | `src/presentation/wx_app.rs:7012-7031` answer submenu `&Accept`, `&Tentative`, `&Decline`; `:7391-7397` "Ans&wer Invitation" on Action; handler `:4593`; `answer_the_invitation` at `:14717` | Reads parts from `cache.attachments_with_content`, and says "Open it once if you have not, so its parts are on this computer" `[STATED: wx_app.rs:14760-14774]` |
| Attachment row wording | `src/presentation/reader_text.rs:1460` `"text/calendar" => Some("calendar invitation")` | Any calendar part, by type alone |
| Where "what is said" is decided | `src/application/reading_a_message.rs:88` `WhatIsSaidAboutIt { opened, envelope, signature }`, `:127` `for_message` | Six surfaces ask it; the invitation belongs here as a fourth field |
| The fold order | `reader_text.rs:1778` `with_what_is_said`: pgp, then envelope, then signature | A sentence folded after a signature verdict is on screen and never spoken (`reader_text.rs:1640-1650`) |

**Absences, with the search that came back empty:**

- Nothing in `src/presentation/` calls `read_the_invitation` or `what_it_asks`
  `[MEASURED: grep -rn 'read_the_invitation\|what_it_asks' src → only application/ files]`.
- No synced event carries the iCalendar UID: `grep -rn 'iCalUID\|iCalUId\|ical_uid' src`
  returned nothing `[MEASURED]`. Google events are stored with `provider_event_id:
  Some(event.id.clone())` at `src/application/calendar.rs:2607` and Graph events the same at
  `:3069` `[STATED]`; `GoogleEvent` starts with `id`, `etag`, `status` at
  `src/service/google_api.rs:266-275` `[STATED]`.
- The answer given is not stored, only the version answered: the only column is
  `answered_version` `[MEASURED: grep answered_version src/data/message_cache/mod.rs → :3182, :4009]`.

### GAP-05, encrypted and signed mail

| What | Where | Notes |
|---|---|---|
| S/MIME shapes | `src/service/signed_mail.rs:65-76` `SmimeLayout`, `:94` `claims_a_signature`, `:122` `claims_encryption`, `:157` `layout_of` | `multipart/signed` with a PGP protocol is refused as S/MIME (`:161-170`, test `:4631`) |
| The platform boundary | `signed_mail.rs:2717-2740` trait `CertificateStore` with `unwrap_content_key` | Windows impl refuses at `:3225-3236`: "Wixen Mail cannot open an S/MIME encrypted message yet ... that part is not built." |
| Windows store, flat `#[link]` calls | `signed_mail.rs:2819-3300` `windows_store` | `CERT_NCRYPT_KEY_HANDLE_PROP_ID` read (`:2835`), so in-memory keys are seen |
| A key in memory only, for tests | `signed_mail.rs:3129` `holding_only_in_memory`, `PFXImportCertStore` with `PKCS12_NO_PERSIST_KEY | PKCS12_ALWAYS_CNG_KSP` (`:2883-2887`) | Fixture `A_KEY_AND_ITS_CERTIFICATE` `:4491`, password `"wixen-test"` `:4545` |
| Envelope reading (who it is to) | `signed_mail.rs:3731-3820` `EncryptedMessage::read`, `spoken` | Fixture `ENCRYPTED_TO_ALICE` `:4120` |
| Envelope on open | `src/application/encrypted_mail.rs:39-105` `WhatTheEnvelopeSays`, `for_message` | "Nothing here decrypts anything" (`:12`) |
| Arrival marks | `src/data/message_cache/how_it_arrived.rs:67` `note_the_form_it_arrived_in`, `:108` `arrived_encrypted`, `:132` `the_envelope_it_carried` | S/MIME only |
| Raw signed originals | `src/data/message_cache/signed_original.rs:108` `keep_signed_original` | Kept only when `claims_a_signature` (S/MIME) |
| Inline PGP open | `src/application/opening_pgp.rs:47` `for_body`, `:62` `the_body_to_show`; `src/service/pgp/mod.rs:189` `open_a_message` | Body replaced in memory; nothing written |
| PGP signature wording | `reader_text.rs:526` `SIGNED_AND_NOT_CHECKED_HERE`: "This message carries a PGP signature, which Wixen Mail cannot check, so nothing here says whether it is genuine." | |
| Outgoing message | `src/service/protocols/smtp.rs:134` `build_message`, `:323` `send_email`, `:340` `message.formatted()` taken as the Sent copy | Every send reaches here `[MEASURED: grep send_email → smtp.rs, mail_controller.rs, sent_copy.rs, wx_app.rs]`; no Graph or Gmail API send path (`grep sendMail` empty) |
| lettre can write the wrappers | `lettre-0.11.22/src/message/mimebody.rs:180-184, 333-342` `MultiPartKind::Encrypted { protocol }`, `Signed { protocol, micalg }`, `MultiPart::encrypted`, `MultiPart::signed` | `[STATED: vendored source]` |
| Outbox row | `src/data/message_cache/mod.rs:2083-2092` `outbox_queue`, columns added by `ensure_column_exists` at `:3105-3147` | A protection choice becomes one more additive column |
| Composer letters | `src/presentation/editor_document.rs:1219-1305` `Reached`, labels and letters F T C B S N H U R O P A D I L, plus E for the People found list (`:1266-1273`) | The page watches for these letters, generated from `Reached::ALL` (`:1307-1316`) |

**Absences:** no verification code in `src/service/pgp/` (it holds `mod.rs` and `keys.rs`
only, and neither names `verify`) `[MEASURED: ls src/service/pgp; grep -n verify]`; nothing
in `smtp.rs`, `wx_compose.rs` or the outbox signs or encrypts `[MEASURED: grep -n
'sign\|encrypt' on those files: only prose]`; no `CryptDecryptMessage`,
`CryptEncryptMessage`, `CryptSignMessage` or `CryptVerifyMessageSignature` anywhere in
`src/` `[MEASURED: grep -rn]`.

### GAP-03, keys

| What | Where | Notes |
|---|---|---|
| One private key, fixed entry | `src/service/pgp/mod.rs:74` `KEYRING_SERVICE = "wixen-mail-pgp"`, `:90` `KEYRING_PRIVATE_KEY = "private-key"`, `:102` `keyring_entries` | "permanent from the commit that writes it" |
| Import | `src/service/pgp/keys.rs:57-89` `import`: refuses public keys and passphrase-locked keys, stores the armour as given (`:80`) | |
| Uninstall list | `src/application/forget.rs:38-54` `entries_for` asks `pgp::keyring_entries()` | a guard fails if a `src/service/` module owns entries and is not named |
| Credential store | `src/service/secret_store.rs:39-76` real backing through `keyring`; `:78-167` test backing, a thread-local map | The test `write` at `:133-143` has no size limit `[STATED]` |
| Menu | `wx_app.rs:6719-6723` File, "Import PGP Private &Key... (experimental)" | on File, not Tools |
| Experimental sentence | `src/application/allowed.rs:399-404` `READING_PGP_MAIL_IS_EXPERIMENTAL`: "... Wixen Mail holds one key at a time, and it has to be exported without a passphrase" | Will be false after GAP-03 |
| Docs | `docs/KEYBOARD_SHORTCUTS.md:618` Import PGP Private Key row | |

**The size limit, measured.** `windows-native-keyring-store` 1.1.0 is what `keyring`
4.1.5 uses on Windows `[MEASURED: Cargo.lock:3081-3096, :6737-6740]`. Its `set_password`
calls `validate_password` (`src/cred.rs:82-83`), which encodes the string as UTF-16 and
refuses it when `blob.len() > CRED_MAX_CREDENTIAL_BLOB_SIZE` (`src/utils.rs:79-94`), and
that constant is `2560` in `windows-sys` (`Security/Credentials/mod.rs:378`)
`[STATED: vendored sources]`. The tree's fixture keys, decoded from `keys.rs:231` and
`:269`: Alice 1,836 characters armoured, 3,672 bytes as UTF-16, 1,295 bytes binary; Bob
1,832 and 3,664 and 1,291 `[MEASURED: python over keys.rs]`. So importing Alice's key on a
real Windows machine answers `CouldNotBeStored` with the store's "too long" reason, and no
test sees it because the test map takes anything.

`[DERIVED]` Those fixtures are RSA-2048 primary keys with no subkey. A usual GnuPG key has
an encryption subkey and is roughly twice that, so above 2,560 bytes even as binary
through `set_secret`; RSA-3072 and RSA-4096 larger again. An Ed25519 key with a Curve25519
subkey is a few hundred bytes and would fit even armoured. Measuring those sizes with
GnuPG failed here: `gpg-agent` refused a home directory under the scratchpad ("socket name
... is too long") `[MEASURED]`; the executor should measure with a short `GNUPGHOME`.

`[DERIVED, outside this group]` The same limit applies to `oauth.rs:910`, which writes
the token JSON through `secret_store::write`. A Microsoft access token is commonly over
1,280 characters `[ASSUMED]`, which would make an Outlook sign-in fail to save on Windows.
Worth a ledger entry and a check by whoever owns sign-in; it is not planned here.

## 3. Proposed plans

Eleven plans, each two to four tasks, each for one executor in one sitting. Ids are
proposals (`R2-nn`); the planner renumbers. One plan per wave, because every plan writes
`docs/changelog.md`, `.planning/WINDOWS.md` and `guards/guards.toml`, and seven write
`src/presentation/wx_app.rs` or `reader_text.rs`.

Files written by rule and in every list below: `docs/changelog.md`, `guards/guards.toml`,
`.planning/WINDOWS.md`. `docs/development/measurements.md` only where a plan takes a figure.

### R2-01 The invitation said where the reader lands (GAP-04, #50 points 1 and 5)

- **Closes:** GAP-04's first `[D]` clause ("the invitation's part shown and said before the
  body"); #50 points 1 and 5.
- **What:** a fourth field on `WhatIsSaidAboutIt`, `invitation: WhatTheInvitationSays`,
  decided in `application` from the stored parts and the calendar: an invitation (title,
  when, where, organiser, and whether it is new, a change to one on the calendar with the
  old and new time, or already answered), a cancellation (and whether that meeting is on
  the calendar), somebody's answer to your meeting (who and what), or nothing. Folded by
  `with_what_is_said` before the signature, into the bar and at the top of the body. The
  attachment row says "meeting invitation", "meeting cancellation", "reply to your
  meeting" or "calendar file" from the method, not the type (`reader_text.rs:1460`). The
  finder takes `application/ics` too. A raw message fixture with a `text/calendar` part,
  run through `service::mime::parse`, asserts the whole path.
- **Files:** `src/application/invitations.rs`, `src/application/answering.rs`,
  `src/application/reading_a_message.rs`, `src/presentation/reader_text.rs`,
  `src/presentation/html_renderer.rs` (the page's section), `tests/an_invitation_is_said_before_the_body.rs`
  (new), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** none in this group.
- **Spoken or shown:** yes. Branch pushed with a pull request.
- **Size:** M.
- **Failing test that starts it:** in `reading_a_message`,
  `test_an_invitation_is_said_before_the_body_on_every_surface`: a document built from a
  message whose parts hold the `REQUEST` fixture has, in `said_before_the_message`, "Meeting
  invitation: Quarterly review" with its time, place and organiser, and a second case with
  a signed message proves it is still above "More about this signature:".
- **Guard coupling:** `reader_text.rs` is named by 13 records and broken by 11;
  `reading_a_message.rs` 5 and 1; `answering.rs` 1 and 1 `[MEASURED: tomllib over guards.toml]`.
  Anchor readings owed on both.

### R2-02 Answer buttons in both reader windows, and the sentence before pressing (GAP-04, #50 points 4, 7, 8)

- **Closes:** GAP-04's "with Accept, Tentative and Decline"; #50 points 4, 7 and 8.
- **What:** three native buttons, A&ccept, &Tentative and &Decline, in a row after the
  security bar and before the message, in the plain reader tab (`wx_reader.rs:425-560`) and
  the formatted window (`wx_app.rs:23914` onward), present only when
  `whether_it_can_be_answered` says yes. When it says no, no buttons and the reason in the
  bar, never greyed buttons (a disabled button is passed over by Tab, and the reason with
  it). Each button's accessible description is `what_pressing_it_will_do`. The answer given
  is stored in an additive column `answered_with` (an enum at the SQL boundary), so "you
  accepted this on ..." can be said. The answer path takes the message row rather than the
  list's selection, so it works from a reader window that is not the selected row. The
  reply is queued with `In-Reply-To` and `References` of the invitation. A context menu
  entry on a message carrying an invitation.
- **Measured first:** whether Alt+C, Alt+T and Alt+D reach a native button while the
  keyboard is inside the WebView2 page of the formatted window. The composer solved the
  same problem by having the page watch for its letters (`editor_document.rs:1307-1316`);
  #84 found the browser keeps every key once it has focus.
- **Files:** `src/presentation/wx_reader.rs`, `src/presentation/wx_app.rs`,
  `src/presentation/wx_context_menu.rs`, `src/application/answering.rs`,
  `src/application/answered_meetings.rs`, `src/data/message_cache/calendar.rs`,
  `src/data/message_cache/mod.rs`, `tests/the_invitation_is_answered_from_the_reader.rs`
  (new), `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/changelog.md`,
  `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-01.
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** a built reader tab for the invitation fixture has three buttons named
  "Accept", "Tentative", "Decline" read over MSAA at their own handles, and none for the
  cancellation fixture; in `answered_meetings`,
  `test_the_answer_given_is_remembered_beside_the_version`.

### R2-03 Synced events carry their iCalendar UID (GAP-04, prerequisite)

- **Closes:** nothing on its own; makes R2-04 able to find a meeting a provider already put
  on the calendar, and stops an answered meeting being filed twice on those accounts.
- **What:** an additive column `ical_uid` on `calendar_events`, filled by the Google
  mapping from `iCalUID` and the Graph mapping from `iCalUId`, and by the CalDAV and
  subscription mappings from `UID`. A lookup `get_event_by_ical_uid(account, uid)` that
  `file_the_answer` and R2-04 use instead of `get_event_by_provider_id`. Existing rows
  fill in on the next sync.
- **Files:** `src/data/message_cache/mod.rs`, `src/data/message_cache/calendar.rs`,
  `src/service/google_api.rs`, `src/service/microsoft_graph.rs`, `src/application/calendar.rs`,
  `src/application/caldav_sync.rs`, `src/application/answered_meetings.rs`,
  `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** none; before R2-04.
- **Spoken or shown:** no.
- **Size:** S.
- **Failing test:** in `application::calendar`, a Google event JSON fixture with
  `"iCalUID": "m-1@example.com"` maps to a row whose `ical_uid` is that value; the same for
  a Graph fixture with `"iCalUId"`. The field names are `[ASSUMED]` from the two APIs'
  documentation and are read from each provider's reference page as the plan's first
  premise.

### R2-04 Updates and cancellations reach the calendar (GAP-04, #50 points 2 and 3)

- **Closes:** GAP-04's "a cancellation removing and an update moving the event"; #50
  points 2 and 3; #50 point 9 (the backlog line corrected).
- **What:** when a message carrying a `REQUEST` at a higher `SEQUENCE` than the calendar's
  copy is opened, and **its sender is the organiser on the calendar's copy**, the event
  moves and the bar says from what to what. A `CANCEL` from the organiser shows a
  "&Remove from Calendar" button (or applies at once, question 3) that marks the event
  `cancelled` and `free` rather than deleting a provider's row. An update or cancellation
  whose sender is not the organiser is said and not applied. When is decided by question
  3; the recommendation is on opening, because parts are stored on opening
  (`wx_app.rs:14770`), and never for an invitation found inside decrypted content.
- **Files:** `src/application/invitations.rs`, `src/application/answered_meetings.rs` (or a
  new `src/application/meeting_changes.rs`), `src/application/reading_a_message.rs`,
  `src/data/message_cache/calendar.rs`, `src/presentation/wx_reader.rs`,
  `src/presentation/wx_app.rs`, `docs/development/requirements-backlog.md`,
  `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-01, R2-02, R2-03.
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** `test_a_cancellation_from_somebody_other_than_the_organiser_changes_nothing`
  and `test_an_update_from_the_organiser_moves_the_meeting_and_says_from_what_to_what`, pure,
  over the fixture in `answered_meetings.rs:209-232`.

### R2-05 S/MIME encrypted mail opens (GAP-05, #52 point 1)

- **Closes:** GAP-05's reading half for S/MIME; #52 point 1; ledger 141 and 143 advanced
  (a real envelope from Outlook or Thunderbird still unmet).
- **What:** the `CertificateStore` method `unwrap_content_key` becomes `open_the_envelope(der)`,
  implemented on Windows with `CryptDecryptMessage` through the `windows` crate (feature
  already on). The decrypted bytes are a MIME entity; `service::mime::parse` turns them
  into a body and parts in memory; nothing decrypted is written to the cache. Four worded
  outcomes: opened, not addressed to a certificate here, the key here refused, damaged.
  An envelope wrapping a signature hands the inner message to `examine_signed_message`.
  Remote pictures in decrypted content are never fetched, whatever the setting. The
  comment at `.cargo/audit.toml:131-136` is rewritten in the same commit, because its first
  narrowing fact ("S/MIME decryption is refused") stops being true.
- **Test fixture:** a new envelope made by OpenSSL for the keyholder certificate already in
  the tree (`A_KEY_AND_ITS_CERTIFICATE`), opened through `holding_only_in_memory`.
  `ENCRYPTED_TO_ALICE` cannot be used: it is addressed to serial
  `0x24738CFA8319EFF0BC4F0DFFBD0831BE5865833C` and the keyholder certificate is
  `52C0434CEA86BF6FA0E8EBA9DEA1A2BDE46E6D8F`; OpenSSL refused to open one with the other
  `[MEASURED: openssl pkcs12 and openssl smime -decrypt in the scratchpad]`.
- **Files:** `src/service/signed_mail.rs`, `src/application/encrypted_mail.rs`,
  `src/application/reading_a_message.rs`, `src/presentation/reader_text.rs`,
  `.cargo/audit.toml`, `docs/privacy.md`, `docs/comparison.md`, `docs/changelog.md`,
  `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** none in this group (R2-01 before it only for the fold order in
  `reader_text.rs`).
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** `service::signed_mail::tests::test_an_envelope_made_for_the_keyholder_opens_with_the_key_held_in_memory`,
  asserting the exact words; and its companion that the same envelope offered to the
  person's own store answers "not addressed to a certificate here".

### R2-06 PGP/MIME opens (GAP-05, #52 point 2)

- **Closes:** #52 point 2; ledger 145 fixed.
- **What:** a mark at arrival that a message is PGP/MIME encrypted (`multipart/encrypted;
  protocol="application/pgp-encrypted"`), asked by `note_the_form_it_arrived_in` beside the
  two facts it already records, as an additive column. On opening, the
  `application/octet-stream` part among the stored attachments goes to
  `service::pgp::open_a_message`; the inner MIME is parsed in memory as in R2-05; the four
  existing sentences apply. Same rule on remote pictures.
- **Fixture:** a PGP/MIME message to Alice made by GnuPG 2.4.9, which is on this machine
  `[MEASURED: gpg --version]`, with a short `GNUPGHOME` (the scratchpad path is too long for
  the agent's socket).
- **Files:** `src/service/signed_mail.rs` (or `src/service/mime.rs`, whichever owns the
  header reading), `src/data/message_cache/how_it_arrived.rs`, `src/data/message_cache/mod.rs`,
  `src/application/opening_pgp.rs`, `src/application/reading_a_message.rs`,
  `src/presentation/reader_text.rs`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-05 (shares the "decrypted MIME in memory" helper).
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** `data::message_cache::how_it_arrived::tests::test_a_pgp_mime_message_is_marked_on_arrival`
  and an `opening_pgp` test that the GnuPG PGP/MIME fixture opens to its words with Alice's key.

### R2-07 Keys that fit the store, several of them, and public keys (GAP-03, service half)

- **Closes:** GAP-03's storage premise; ledger entry for the size defect opened and fixed.
- **What:** red first by making the test credential store refuse what Windows refuses
  (2,560 bytes of UTF-16), which turns the existing Alice import test red and proves the
  defect. Then storage by Pratik's answer to question 1 (recommended: a key split across
  entries with fixed names under `wixen-mail-pgp`, a fixed number of slots and parts, all
  listed by `keyring_entries` without reading anything, and `private-key` kept on the list
  forever so an older machine's entry is still erased). Public keys of correspondents in
  an additive table, since they are not secret (question 7). A listing type in the
  project's words: user ids, key id, fingerprint, created, expires, private or public, can
  encrypt, can sign. Import from armoured text holding one or many keys
  (`from_armor_many`), remove, export a public key as armour (`to_public_key`,
  `to_armored_string`). Nothing outside `src/service/pgp/` names the crate.
- **Files:** `src/service/secret_store.rs`, `src/service/pgp/mod.rs`, `src/service/pgp/keys.rs`,
  `src/data/message_cache/mod.rs`, a new `src/data/message_cache/pgp_keys.rs`,
  `src/application/forget.rs`, `docs/privacy.md`, `docs/changelog.md`, `guards/guards.toml`,
  `.planning/WINDOWS.md`.
- **Depends on:** Pratik's answer to question 1.
- **Spoken or shown:** no (the manager in R2-08 is).
- **Size:** M.
- **Failing test:** `service::secret_store::tests::test_a_secret_windows_would_refuse_is_refused_here_too`,
  then `service::pgp::keys::tests::test_an_ordinary_rsa_key_is_stored_and_opens_mail`, red
  against today's code for the reason above.

### R2-08 The key manager window (GAP-03, #49)

- **Closes:** GAP-03; #49.
- **What:** Tools, "PGP Ke&ys... (experimental)" opens a dialog on the pattern of Blocked
  Senders (`wx_blocked_senders.rs:177-190`, a report `ListCtrl`): the limits sentence
  first, the list, Import from &File, &Paste a Key, E&xport Public Key, &Copy Public Key,
  &Remove, Cl&ose. Remove confirms with what will stop opening. The File menu item goes
  or becomes a door to the manager (question 8). "Import this key" on an attachment of
  type `application/pgp-keys` in the reader. `READING_PGP_MAIL_IS_EXPERIMENTAL` rewritten
  (its "one key at a time" becomes false). `.cargo/audit.toml:134-136` rewritten ("labelled
  experimental in the File menu"). Privacy page paragraph.
- **Files:** a new `src/presentation/wx_pgp_keys.rs`, `src/presentation/wx_app.rs`,
  `src/presentation/wx_reader.rs`, `src/application/allowed.rs`, `.cargo/audit.toml`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/privacy.md`, `docs/changelog.md`,
  `tests/the_key_manager_lists_and_names_its_controls.rs` (new), `guards/guards.toml`,
  `.planning/WINDOWS.md`.
- **Depends on:** R2-07.
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** the built dialog, read over MSAA and UI Automation at each control's
  own handle, names the list and six buttons; the letter check in `tests/wired.rs:1977`
  goes red on a planted duplicate and green on the dialog's set.

### R2-09 PGP signatures checked and said (GAP-05, #52 point 3)

- **Closes:** GAP-05's "a PGP signature is verified and said"; #52 point 3; ledger 103 and
  497 advanced.
- **What:** inline cleartext signatures through `CleartextSignedMessage::verify`, PGP/MIME
  signatures through the detached signature over the kept raw part, both against the
  public keys from R2-07 and the private keys' public halves. The raw bytes of a PGP/MIME
  signed message kept at arrival beside S/MIME's (a kind column on `signed_original`, not
  a widening of `claims_a_signature`, whose comment says why). Outcomes worded with the
  S/MIME discipline: holds, and the key is in your list under this name; holds against a
  key not in your list, named by key id only; does not hold; damaged. A sentence never
  says "signed" alone. `SIGNED_AND_NOT_CHECKED_HERE` goes, and the conversation view says
  it per message.
- **Files:** `src/service/pgp/mod.rs`, `src/service/pgp/keys.rs` (or a new
  `src/service/pgp/signatures.rs`), `src/data/message_cache/signed_original.rs`,
  `src/application/checking_signatures.rs`, `src/application/reading_a_message.rs`,
  `src/presentation/reader_text.rs`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-06, R2-07.
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** a GnuPG-made clearsigned fixture by Alice verifies with Alice's public
  key and the sentence names her; one changed letter makes it "does not hold".

### R2-10 Signing and encrypting outgoing mail, the service half (GAP-05, #52 points 4 and 5)

Split in two because it is two libraries and two fixtures, and one executor in one sitting
cannot hold both.

**R2-10a S/MIME.** `CryptSignMessage` with signed attributes (the probe found none are
written unless asked for), a detached signature in `multipart/signed;
protocol="application/pkcs7-signature"` built with lettre's `MultiPart::signed`; and
`CryptEncryptMessage` to each recipient's certificate and to the sender's own, as
`application/pkcs7-mime; smime-type=enveloped-data`. Key transport chosen on purpose
(question 6; the probe found Windows picks RSA-OAEP when not told). Recipients'
certificates from signed mail already received, kept at arrival (question 7).
- **Files:** `src/service/signed_mail.rs`, `src/service/protocols/smtp.rs`,
  `src/data/message_cache/signed_original.rs` or a new certificates table,
  `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-05. **Spoken or shown:** no. **Size:** M.
- **Failing test:** sign with the in-memory keyholder key, then check it with this
  project's own `examine_signed_message`, a second implementation, and open an encrypted
  one with `CryptDecryptMessage`; both red until the functions exist.

**R2-10b PGP/MIME.** `MessageBuilder` with SEIPD v1 for the widest reach, encrypted to each
recipient's key and the sender's own, signed with the sender's key; RFC 3156 wrappers
through lettre. Needs `rand` 0.8 named as a direct dependency (section 4).
- **Files:** `Cargo.toml`, `src/service/pgp/mod.rs`, `src/service/pgp/keys.rs`,
  `src/service/protocols/smtp.rs`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-07; Pratik's confirmation of the `rand` line (task 1 waits for it).
- **Spoken or shown:** no. **Size:** M.
- **Failing test:** a message built for Alice opens with `open_a_message` and Alice's key,
  and the executor also opens it with `gpg --decrypt` (a second implementation) and
  records the command and its output.

### R2-11 The composer's Sign and Encrypt (GAP-05, #52 points 4 and 5)

- **Closes:** GAP-05's "a message can be sent signed and encrypted".
- **What:** two check boxes, Si&gn and Encr&ypt, added to `Reached` so the page watches for
  G and Y as it does for the other fifteen letters; an additive `outbox_queue` column for
  the choice as an enum; the choice honoured in the send loop at `build_message`; a
  recipient with no key or certificate said at Send, before anything is queued, never by
  greying the box; the Sent copy is the protected bytes, readable because the sender is a
  recipient too; marked experimental in the composer.
- **Files:** `src/presentation/editor_document.rs`, `src/presentation/wx_compose.rs`,
  `src/application/sending_later.rs`, `src/data/message_cache/mod.rs`,
  `src/data/message_cache/outbox.rs`, `src/service/protocols/smtp.rs`, `src/application/allowed.rs`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/comparison.md`, `docs/changelog.md`,
  `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** R2-10a, R2-10b.
- **Spoken or shown:** yes.
- **Size:** M.
- **Failing test:** `editor_document`'s own letter test goes red when G or Y is planted twice
  and passes with the two new members; an outbox row saved with `Signed` loads back as
  `Signed`, and one written before the column existed loads as `Plain`.

### Order and waves

R2-01, R2-02, R2-03, R2-04, then R2-05, R2-06, then R2-07, R2-08, then R2-09, R2-10a,
R2-10b, R2-11. One per wave. R2-03 has no dependency and could go anywhere before R2-04;
it goes third so the invitation plans stay together.

## 4. Packages

### What the tree already has (asked first, per the dependency-audit skill)

| Capability | Already present | Evidence |
|---|---|---|
| OpenPGP read, verify, sign, encrypt, export | `pgp` 0.20.0 | `Cargo.lock:4050-4051`; vendored source has `composed/message/builder.rs:315` `seipd_v1`, `:442` `encrypt_to_key`, `:791` `sign`; `composed/signature.rs:176` `verify(key, content)`; `composed/cleartext.rs:126` `verify`; `composed/signed_key/secret.rs:162` `to_public_key`; `composed/signed_key/parse.rs:43` `from_armor_many` `[MEASURED: grep of the vendored crate]` |
| S/MIME open, verify, sign, encrypt | `windows` 0.62.2 with `Win32_Security_Cryptography` | `Cargo.toml` target dependencies; `windows-0.62.2/src/Windows/Win32/Security/Cryptography/mod.rs:1205` `CryptDecryptMessage`, `:1256` `CryptEncryptMessage`, `:1842` `CryptSignMessage`, `:1928` `CryptVerifyDetachedMessageSignature`, `:2518` `PFXImportCertStore` `[STATED]` |
| Writing `multipart/signed` and `multipart/encrypted` | `lettre` 0.11.22 | `mimebody.rs:333-342` |
| A random source rPGP accepts | `rand` 0.8.7 in the lock, through `num-bigint-dig` from `pgp` | `Cargo.lock:4628-4636`; `cargo tree -i rand@0.8.7` `[MEASURED]`. The direct `rand` is 0.10.2 (`Cargo.lock:4639-4640`), whose traits rPGP's `CryptoRng + Rng` bounds from `rand` 0.8 do not accept (`builder.rs:11`, `:321`) `[STATED]` |

**The one new line this group needs:** `rand` 0.8 as a direct dependency under a renamed
key (for example `rand08 = { package = "rand", version = "0.8" }`), for R2-10b. It adds no
lock entry, because 0.8.7 is already locked and compiled. It is a new `Cargo.toml` line,
so Pratik confirms it, and R2-10b's first task waits for that. The readers of
`Cargo.toml` in `tests/` and `scripts/` are grepped by that task before the line lands
(skill question 7).

### The PGP comparison, re-taken

The choice was made on 2026-09-06 and is written in
`docs/development/pgp-implementation-choice.md`. Its versions have not moved: `pgp` 0.20.0
(2026-06-23) and `sequoia-openpgp` 2.4.1 (2026-07-09) are still the latest stable releases
`[MEASURED: crates.io API, 2026-09-24]`. What moved is the tree, so the delta was
re-measured against today's `Cargo.lock` (723 packages) in a scratch copy:

| Candidate, configuration | New lock entries | Of which a second version of a crate already locked | Licence | C toolchain | Advisories open at this version |
|---|---|---|---|---|---|
| `pgp` 0.20.0, default (in the tree) | 0 | 0 | MIT OR Apache-2.0 | none | none against `pgp` (RUSTSEC-2024-0447 patched from 0.14.1); `rsa` RUSTSEC-2023-0071 accepted in `.cargo/audit.toml` |
| `sequoia-openpgp` 2.4.1, default (Nettle) | 19 | 1 | LGPL-2.0-or-later | Nettle, GMP, pkg-config | none open (RUSTSEC-2023-0038, 2024-0345, 2025-0136 all patched below 2.4.1) |
| `sequoia-openpgp` 2.4.1, `crypto-rust` with `allow-experimental-crypto` and `allow-variable-time-crypto` | 37 | 11 | LGPL-2.0-or-later | none | as above |

`[MEASURED: cargo metadata in phase-13-probes/lockdelta, diffed against Cargo.lock.orig]`.
The control for the instrument: the `cms` row below registered 4 and 6 new entries, so a
zero here is a real zero. rPGP stays: the licence reason is unchanged and it is already
compiled in. rPGP's CI still does not test `x86_64-pc-windows-msvc` `[STATED: pgp-implementation-choice.md]`.
Maintenance: rPGP's last commit 2026-09-21; Sequoia's last activity 2026-09-24
`[MEASURED: gh api, gitlab api]`.

### The S/MIME comparison

| | Windows CryptoAPI (`windows` 0.62.2) | `cms` 0.2.3 (RustCrypto) | `openssl` 0.10.81 |
|---|---|---|---|
| New lock entries | 0, feature already on | 4 (`cms`, `der_derive`, `flagset`, `x509-cert`); 6 with `builder` (adds `tls_codec`, `tls_codec_derive`) | 0 for the crate (already locked for other targets through `native-tls`, never compiled for Windows); 1 more with `vendored` (`openssl-src` 300.6.1+3.6.3) |
| Opens a message with a key in the Windows store | yes, the key never leaves the store | no: no decrypt in 0.2.3 (`grep 'fn decrypt'` over `cms-0.2.3/src` empty); a store key would still need CNG for the unwrap | no: needs the private key as bytes, which a Windows store key usually refuses to give |
| Build on MSVC with the pinned 1.98.1 | yes, built in the probe | pure Rust, not built here `[ASSUMED builds]` | needs an OpenSSL install, or Perl and NASM for `vendored` `[ASSUMED]`; `cargo tree -i openssl --target x86_64-pc-windows-msvc` prints nothing, so it would be a new compile |
| Licence, text shipped | part of Windows, nothing to ship | Apache-2.0 OR MIT declared; **no LICENSE file in the published crate** (`ls cms-0.2.3`: CHANGELOG, Cargo.toml, README, src, tests); `x509-cert` ships both | Apache-2.0; `openssl-src` bundles OpenSSL 3.x under Apache-2.0, whose notice the installer does not ship yet (`Cargo.toml`, phonenumber comment) |
| Release history | Windows | stable 0.2.3 of 2024-01-08, then only `0.3.0-pre.0` to `pre.2` (2024-10 to 2026-01) | stable, monthly |
| Advisories | `windows` RUSTSEC-2022-0008 patched from 0.32 | none | 10 against `openssl`, all patched at 0.10.81; 25 against `openssl-src`, all older than 300.6 `[MEASURED: ~/.cargo/advisory-db, updated 2026-09-24]` |
| Maintenance | windows-rs pushed 2026-09-22 | RustCrypto/formats pushed 2026-09-23; `cms/` last touched 2026-08-05 | pushed 2026-09-22, 359 open issues |

**A correction for `Cargo.toml:280-284`.** It refuses `cms` because "`cms` has only ever
published pre-release versions". crates.io lists stable `0.2.0` to `0.2.3` (2023-03 to
2024-01) `[MEASURED: crates.io API]`. The refusal still stands on other grounds (no
decryption, no licence text in the crate, and the stable line has not moved in 32 months
while 0.3 stays in pre-release), and the comment should say those. Worth one line in
R2-05's docs task rather than a plan.

**What was probed** (`phase-13-probes/smime-cng`, its own `CARGO_TARGET_DIR`, toolchain
1.98.1, OpenSSL 3.5.7 for the fixtures, RSA-2048 certificate and key imported with
`PKCS12_NO_PERSIST_KEY | PKCS12_ALWAYS_CNG_KSP`):

```text
enc_aes256_cbc.p7m: decrypted 82 bytes, matches OpenSSL input: true
enc_oaep.p7m: decrypted 82 bytes, matches OpenSSL input: true
enc_gcm.p7m: decrypted 82 bytes, matches OpenSSL input: true
OpenSSL detached signature verified by Windows: signer cert of 886 bytes
tampered content refused as it should be: verify: The hash value is not correct. (0x80091007)
Windows signed with the in-memory key: 1314 bytes written for OpenSSL to check
Windows encrypted to bob: 522 bytes written for OpenSSL to open
  and Windows opened its own envelope: true
```

Then OpenSSL: "CMS Verification successful" on the Windows signature with identical
content, and `openssl cms -decrypt` returned the Windows envelope's contents byte for
byte. Two things the output showed that the plans must act on: Windows signed with **no
signed attributes** (`signedAttrs: <ABSENT>`), and encrypted with **RSA-OAEP** key
transport (`rsaesOaep`) when nothing asked for one `[MEASURED]`. `enc_gcm.p7m` is
`authEnvelopedData` (RFC 5083) and opened too.

### Verdicts

```
Dependency: rand@0.8 (renamed key, already locked at 0.8.7)
Purpose: the random source rPGP 0.20's builder bounds accept, for signing and encrypting
Size: 0 new lock entries; 0 second versions (it is the version pgp already compiles)
Maintenance: the rust-random project; the 0.8 line is an old major kept for rPGP's sake;
  not re-read today [ASSUMED]
License: MIT OR Apache-2.0 [ASSUMED from the lock's source]; text shipped not checked
Alternatives: rand_core 0.6 OsRng (also locked); writing an adapter from rand 0.10 is not
  possible for foreign traits without a newtype, which is more code than the line
Defaults overlapping rules we already hold: none
Measured unchanged: not measured (nothing installed)
Recommendation: ADD, in R2-10b task 1, after Pratik confirms
```

```
Dependency: cms@0.2.3
Purpose: CMS types and builders for S/MIME
Size: 4 new (6 with builder), 0 second versions, against this Cargo.lock
Maintenance: RustCrypto, active repo; stable line last released 2024-01-08
License: Apache-2.0 OR MIT declared; no licence text in the published crate
Alternatives: Windows CryptoAPI, already in the tree, probed working both ways
Recommendation: SKIP
```

```
Dependency: openssl@0.10.81 (+ openssl-src for vendored)
Purpose: S/MIME through OpenSSL
Size: 0 lock entries but a first compile for Windows; 1 more with vendored; needs a C toolchain path
License: Apache-2.0, bundled OpenSSL notice not shipped by the installer today
Alternatives: Windows CryptoAPI
Recommendation: SKIP
```

```
Dependency: sequoia-openpgp@2.4.1
Purpose: OpenPGP
Size: 19 new (Nettle) or 37 new with 11 second versions (crypto-rust)
License: LGPL-2.0-or-later against an MIT statically linked installer
Alternatives: pgp 0.20, already in the tree
Recommendation: SKIP (unchanged from 2026-09-06)
```

Package legitimacy seam: `cms`, `pgp`, `sequoia-openpgp`, `openssl`, `openssl-src`,
`rand` all answered `OK` `[MEASURED: gsd-tools query package-legitimacy check --ecosystem crates]`.
None is installed by this group except the `rand` line.

## 5. What needs a real account, and what cannot be finished

**Needs a real account or a real correspondent (phase 14 and ledger entries; marked
experimental where the person sees it):**

- An invitation, an update and a cancellation from a real Outlook, Google and CalDAV
  organiser; the reply arriving and threading in the organiser's mailbox (ledger 152).
- Whether Google's `iCalUID` and Graph's `iCalUId` match the `UID` in the invitation their
  own servers mailed (R2-03's premise, checked against documentation only here).
- Whether a provider that already put the invitation on its calendar (Gmail, Exchange)
  and this program's filed answer end up as one event or two (ledger 154).
- An S/MIME message from Outlook or Thunderbird, and a real certificate in the person's own
  `MY` store; a key marked for strong protection or held on a smart card, which makes
  Windows show its own prompt on use (`CryptDecryptMessage` has no silent flag in the
  `windows` crate's struct, `CRYPT_DECRYPT_MESSAGE_PARA` at `mod.rs:8579-8584`).
- A PGP/MIME message from Thunderbird or Proton; a real key from a real correspondent.
- Signed and encrypted mail this program sends, opened by Outlook, Thunderbird and Apple
  Mail; whether they accept RSA-OAEP key transport (question 6).

**Cannot be finished here at all, said rather than stubbed:**

- One day of a repeating meeting stays unanswerable (#50 point 6); the sentence at
  `answering.rs:125-130` already says so and stays.
- Signed mail stored before originals were kept (#52 point 6) cannot be checked, because
  the bytes are gone; the fix is to say "signed, and not checked, because this copy was
  stored before signatures were kept" rather than "unsigned". That sentence can be built;
  the check cannot.
- Searching inside encrypted mail, if Pratik keeps decrypted text out of the cache
  (question 4).

## 6. Accessibility per feature

**Invitation in the reader (R2-01, R2-02, R2-04).**
- The invitation's sentence is at the top of the bar, so it is spoken as the message opens
  (the reader speaks everything above "More about this signature:"), and repeated at the
  top of the body, as the S/MIME envelope sentence already is.
- Buttons: native `Button`s, labels A&ccept, &Tentative, &Decline, and &Remove from
  Calendar for a cancellation. **Not &Accept:** Alt+A is the attachments key in both reader
  windows `[STATED: docs/KEYBOARD_SHORTCUTS.md:208-245; grep of that section for Alt+ found
  only Alt+A]`. Alt+C, Alt+T, Alt+D and Alt+R are otherwise unused there.
- Names: the label is the name; confirm on MSAA and UI Automation at each button's own
  handle. Accessible description is `what_pressing_it_will_do`, which NVDA reads after the
  name.
- Focus: after pressing, focus stays on the message and the result is announced once
  (ledger 155 asks whether it is said twice; keep one announcement).
- No greyed buttons. When not answerable, no buttons and the reason in the bar.
- Keyboard doc: a row per button in The Reader Window; the Action menu's "Ans&wer
  Invitation" stays.

**Encrypted and signed mail (R2-05, R2-06, R2-09).** Each outcome is one sentence in the
bar, distinct from its siblings, and never the word "signed" alone
(`signed_mail.rs:5341` holds S/MIME to that already). No cue sounds as unsafe for an
encrypted message (`reader_text.rs:1640-1652` gives the reason).

**Key manager (R2-08).** Tools letters in use: A B C D E F I K L N O P R S T W; free: G H J
M Q U V X Y Z `[MEASURED: python over wx_app.rs:7429-7574 labels]`. So "PGP Ke&ys..." on Y.
Other phase 13 groups also add to Tools; the letter is a shared resource across the phase
and the planner allocates it once. Inside the dialog: What these keys can do &here (H),
&Keys list (K), Import from &File (F), &Paste a Key (P), E&xport Public Key (X), &Copy
Public Key (C), &Remove (R), Cl&ose (O): eight distinct letters. The list is a report
`ListCtrl` like Blocked Senders, first column the name and address, so a row is read as a
person before a fingerprint. Remove's confirmation names the key and what will stop
opening. A new "PGP Keys Dialog Accelerators" section in `docs/KEYBOARD_SHORTCUTS.md`
beside "Blocked Senders Dialog Accelerators" (`:1178`). No shortcut on the menu item, for
the reason the import item has none.

**Composer (R2-11).** Letters in use: F T C B S N H U R O P A D I L and E; Si&gn on G and
Encr&ypt on Y are free `[STATED: editor_document.rs:1258-1305]`. They go into `Reached` so
the page's key table covers them while the keyboard is in the message. A check box that
cannot be honoured is not greyed; pressing Send says which recipient has no key or
certificate, and nothing is queued.

**Passphrase entry, if Pratik takes it (question 2).** A password field that allows paste
and a password manager, no copying a code, no puzzle: WCAG 3.3.8. Its name on both
channels at the field's own handle.

## 7. Questions only Pratik can answer

1. **Where should private keys live, given Windows' limit?** The credential store takes at
   most 1,280 characters per entry, and an ordinary RSA key is two to four times that.
   (a) Split each key across several entries with fixed names, so the uninstaller still
   names them without reading anything; (b) a file in the data folder protected by
   Windows' own per-user encryption (DPAPI); (c) one random key in the credential store
   and the keys in a file encrypted with it. **Recommend (a):** it keeps the rule that
   secrets live in the credential store, needs no new code from outside, and keeps
   uninstall exact. (b) and (c) change CLAUDE.md's secrets rule.
2. **Keys locked with a passphrase.** Most keys people export have one, and today they are
   refused. (a) keep refusing; (b) ask when a message first needs the key and remember it
   until Wixen Mail closes, never storing it; (c) ask every time. **Recommend (b)** as a
   plan after R2-08, because without it the manager serves few real keys.
3. **When an update or cancellation changes the calendar.** (a) when the message is
   opened, and said; (b) when it arrives, even if never opened; (c) only when you press a
   button. **Recommend:** updates from the organiser applied on opening and said;
   cancellations shown with a Remove from Calendar button, the way Outlook does it
   `[ASSUMED Outlook's behaviour]`; never applied for a message whose sender is not the
   organiser.
4. **Decrypted mail on disk.** **Recommend** decrypting each time it is opened and never
   storing the decrypted text, so an encrypted message stays encrypted at rest, and saying
   plainly that search does not look inside encrypted mail.
5. **Order.** Your order puts encrypted mail before the key manager; GAP-05 says the
   manager comes first. **Recommend:** reading encrypted mail first (it needs one key),
   then the manager, then checking PGP signatures and sending (they need other people's
   keys).
6. **How outgoing S/MIME wraps its key.** Windows picked the newer RSA-OAEP when not told.
   **Recommend** the older PKCS #1 v1.5 until a real Outlook and Apple Mail round trip
   shows OAEP opens, because a message nobody can open is worse than the weaker padding;
   say which in the changelog.
7. **Other people's public keys and certificates.** They are not secret. **Recommend** a
   table in the mail database, and keeping the certificate from every signed message that
   arrives so replying encrypted just works.
8. **The File menu's Import PGP Private Key.** **Recommend** removing it once the manager
   on Tools holds import, since two doors to one thing is one more item to arrow past.
9. **Invitations inside encrypted mail.** **Recommend** showing them and offering the
   answer buttons, but never changing the calendar on their own, because an automatic
   calendar change that happens only when a message decrypted is the kind of signal
   the accepted RSA advisory says must not exist.

## Common Pitfalls

1. **A test store with no limits.** The fake credential store accepts any size, so the
   import has never met Windows' 2,560-byte limit. Make the fake refuse what Windows
   refuses before trusting any storage test.
2. **Letters that the browser eats.** A mnemonic on a native button does not fire while
   the WebView2 page has the keyboard (#84). Measure where the key arrives first, the way
   12-08 did, and use the page's key table as the composer does.
3. **Alt+A.** Accept cannot take A in a reader window.
4. **A cancellation from a stranger.** iTIP lets anybody send `METHOD:CANCEL` with your
   meeting's UID. Apply only when the sender is the organiser on the calendar's copy
   `[CITED: RFC 5546 section 3.2.5 names the organiser as the sender of CANCEL; ASSUMED
   section number]`.
5. **UID against provider ids.** An invitation's UID is not Google's or Microsoft's event
   id. Without R2-03 every lookup on those accounts finds nothing and files a second event.
6. **EFAIL and the advisory's expiry.** Decrypted HTML that fetches a remote picture tells
   the picture's host the message decrypted. Never fetch anything for decrypted content,
   and never join decrypted parts with clear ones into one page. Rewrite
   `.cargo/audit.toml:131-136` in the plans that make its narrowing facts false.
7. **Signing the wrong bytes.** A `multipart/signed` signature covers the exact bytes of
   the first part with CRLF endings. Sign the output of the inner part's `formatted()`,
   and prove it by checking the result with the project's own verifier.
8. **CryptoAPI's quiet defaults.** No signed attributes unless some are passed; RSA-OAEP
   unless told. Both measured.
9. **Encrypting only to the recipients.** The Sent copy is then unreadable by the sender.
   Encrypt to the sender's own key or certificate too.
10. **The fold order.** A new sentence folded after the signature verdict is shown and
    never spoken (`reader_text.rs:1640-1650`). Invitations fold before it.
11. **Greyed controls.** A disabled button or check box is skipped by Tab with its reason.
    Say the reason instead.
12. **Guard records by anchor text.** `reader_text.rs` has 11 records breaking it; any
    block moved or re-indented moves their anchors. List both readings in each plan.

## Validation Architecture

| Property | Value |
|---|---|
| Framework | `cargo test` (Rust 1.98.1 pinned), `tokio-test`, `tempfile` |
| Quick run | `cargo test --lib <module path>::` one module per run, joined with `&&` |
| Full suite | `bash scripts/check.sh` through the hook; `scripts/check.sh all` once in the phase's closing plan |

| Plan | Behaviour | Type | Command |
|---|---|---|---|
| R2-01 | invitation said before the body | unit | `cargo test --lib application::reading_a_message:: && cargo test --lib presentation::reader_text::` |
| R2-02 | buttons named on both channels | integration (built window) | `cargo test --test the_invitation_is_answered_from_the_reader` |
| R2-03 | iCalendar UID mapped | unit | `cargo test --lib application::calendar:: && cargo test --lib data::message_cache::calendar::` |
| R2-04 | organiser-only update and cancel | unit | `cargo test --lib application::answered_meetings:: && cargo test --lib application::invitations::` |
| R2-05 | envelope opens with in-memory key | unit, Windows | `cargo test --lib service::signed_mail::` |
| R2-06 | PGP/MIME marked and opened | unit | `cargo test --lib data::message_cache::how_it_arrived:: && cargo test --lib application::opening_pgp::` |
| R2-07 | store refuses oversize; keys listed | unit | `cargo test --lib service::secret_store:: && cargo test --lib service::pgp::` |
| R2-08 | dialog names and letters | integration | `cargo test --test the_key_manager_lists_and_names_its_controls && cargo test --test wired` |
| R2-09 | signature verdicts | unit | `cargo test --lib service::pgp:: && cargo test --lib application::checking_signatures::` |
| R2-10a | round trip against own verifier | unit, Windows | `cargo test --lib service::signed_mail::` |
| R2-10b | round trip, and gpg by hand | unit plus a recorded command | `cargo test --lib service::pgp::` |
| R2-11 | letters, outbox column | unit | `cargo test --lib presentation::editor_document:: && cargo test --lib data::message_cache::outbox::` |

Wave 0 gaps: none in the framework. New fixtures are made with OpenSSL 3.5.7 and GnuPG
2.4.9 on this machine; GnuPG needs a short `GNUPGHOME`.

## Security Domain

| ASVS | Applies | Control |
|---|---|---|
| V5 input validation | yes | invitations, MIME and armour are strangers' bytes; parse with existing readers, 25 MB ceiling kept |
| V6 cryptography | yes | Windows CNG and rPGP; nothing hand-rolled beyond the ASN.1 reading already in `signed_mail.rs` |
| V8 data protection | yes | private keys only in the credential store; decrypted text never written (question 4) |
| V2, V3, V4 | no | no authentication or sessions here |

| Threat | STRIDE | Mitigation |
|---|---|---|
| Spoofed update or cancellation | Tampering | sender must be the organiser on the calendar's copy |
| EFAIL exfiltration through decrypted HTML | Information disclosure | no remote fetch for decrypted content; no joining of clear and decrypted parts |
| Decryption oracle (RUSTSEC-2023-0071) | Information disclosure | no automatic behaviour depends on decryption success; audit entry rewritten |
| Key left behind after uninstall | Information disclosure | every fixed entry name listed by `keyring_entries`; forget guard |
| A signature read as proof of identity | Spoofing | sentences say what a signature does not show, as S/MIME's do |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|---|---|---|
| A1 | A typical GnuPG RSA key with a subkey exceeds 2,560 bytes even as binary; Ed25519 fits | 2 | Question 1 is less urgent for ECC-only users |
| A2 | Microsoft access tokens exceed 1,280 characters | 2 | The out-of-group note is a false alarm |
| A3 | Google `iCalUID` and Graph `iCalUId` are the field names and carry the invitation's UID | R2-03 | R2-03's mapping reads nothing |
| A4 | `cms` 0.2.3 and `openssl` vendored build as described | 4 | none; both are skipped |
| A5 | Outlook applies updates and offers Remove from Calendar for cancellations | Q3 | the recommendation's precedent |
| A6 | RFC 5546 section number for CANCEL | Pitfall 4 | citation only; the rule stands on sense |
| A7 | `rand` 0.8 licence and maintenance not re-read today | 4 | re-read in R2-10b task 1 |

## Sources

- The tree at `630e2a67`, every `[STATED]` line above.
- Vendored crates: `windows-native-keyring-store-1.1.0`, `windows-0.62.2`, `lettre-0.11.22`,
  `pgp-0.20.0`, `cms-0.2.3` in `~/.cargo/registry/src/`.
- crates.io API for `pgp`, `sequoia-openpgp`, `cms`, `openssl`, `openssl-src`, `rsa`, 2026-09-24.
- RustSec advisory database at `~/.cargo/advisory-db`, commit `593df8c1` of 2026-09-24.
- GitHub and GitLab APIs for repository activity, 2026-09-24.
- Probes: `C:/Users/prati/AppData/Local/Temp/claude/C--Users-prati-Documents-projects-Wixen-Mail/ee5f65c6-033f-4d20-9966-4517170e1c1f/scratchpad/phase-13-probes/`
  (`smime-cng`, `fixtures`, `lockdelta`).
- `docs/development/pgp-implementation-choice.md` for the 2026-09-06 comparison.

**Valid until:** 2026-10-24 for versions and advisories; line numbers until the next plan
touches the files.

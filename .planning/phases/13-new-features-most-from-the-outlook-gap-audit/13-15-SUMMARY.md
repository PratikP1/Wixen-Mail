---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 15
subsystem: service::signed_mail, data::message_cache::how_it_arrived, application::opening_pgp, application::encrypted_mail, application::reading_a_message, presentation::reader_text
status: complete
tags: [pgp, pgp-mime, encryption, privacy, reader, GAP-05]
requires: [13-14]
provides:
  - signed_mail::claims_pgp_encryption and is_a_pgp_mime_part, the PGP/MIME header reading beside S/MIME's
  - the arrived_pgp_encrypted column, noted by note_the_form_it_arrived_in and read by arrived_pgp_encrypted and the_pgp_mime_part_it_carried
  - opening_pgp::for_pgp_mime and from_the_part, the part offered to the key and what it came to
  - WhatTheEnvelopeSays::OpenedWithPgp and PgpNotOpened, and what_the_pgp_key_found
  - encrypted_mail::taken_apart, the one reading of decrypted MIME for both families
  - service::pgp::for_tests::a_pgp_mime_message_to_alice and bobs_private_key
affects: [13-16, 13-17, 13-18, 13-20, 13-21, 13-51]
tech-stack:
  added: []
  patterns:
    - "both families' decrypted answers travel as one type, asked from one function, so every surface that asks about an envelope meets PGP/MIME without a second call site"
    - "a PGP/MIME message that did not open becomes inline PGP's case: its armour is the body and the reason is carried, so the four sentences are written once"
key-files:
  created: []
  modified:
    - src/service/signed_mail.rs
    - src/service/pgp/keys.rs
    - src/service/pgp/mod.rs
    - src/data/message_cache/how_it_arrived.rs
    - src/data/message_cache/mod.rs
    - src/application/opening_pgp.rs
    - src/application/encrypted_mail.rs
    - src/application/reading_a_message.rs
    - src/presentation/reader_text.rs
    - tests/an_encrypted_message_is_not_left_unexplained.rs
    - guards/guards.toml
    - .cargo/audit.toml
    - docs/privacy.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "PGP/MIME's answer is carried by WhatTheEnvelopeSays and asked from encrypted_mail::for_message_opened_with, because the reader, answering a meeting and saving a file inside all ask that one function; a separate OpenedPgpMime type would have needed a second call at each of them."
  - "is_opened covers both families, so the picture rule, the files listed and a meeting's provenance are 13-14's rules reached through one mark rather than copied."
  - "An opened PGP/MIME message says nothing above its words, as inline PGP says nothing; the S/MIME sentence names a certificate and was not reused."
  - "A PGP/MIME message that did not open shows its armour and carries the reason found, so the reader words it with inline PGP's four sentences and the key is offered the message once."
  - "A message marked PGP/MIME whose encrypted part is not on this computer says S/MIME's 'the details could not be read' sentence, which is true of it word for word."
metrics:
  duration: about 3 hours 40 minutes on 2026-09-26, about 75 minutes of it three guard re-measures of 11, 1 and 35 records
  completed: 2026-09-26
estimate:
  tokens: 100000
  tasks: 3
actuals:
  tokens: 17000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 15: PGP/MIME mail opens Summary

A PGP/MIME message, `multipart/encrypted` with the protocol `application/pgp-encrypted`, is
now recognised from its headers on every arrival path and marked in an additive column,
`arrived_pgp_encrypted`. Opening it offers its encrypted part to the imported PGP key; what
opens is taken apart in memory by the reading 13-14 wrote for an S/MIME envelope and shows as
its words and its files on every surface that shows a message, with nothing said above it, as
inline PGP says nothing. Nothing opened is stored. A page holding it fetches no picture, its
files are the ones listed, and a meeting inside is said and never changes the calendar on
opening, all through the one mark 13-14's rules read. One that does not open shows its armour
and the reason, in inline PGP's four sentences. Ledger 145 is fixed.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards`, `docs` and
`.cargo` against `main` at `a351c9c4`, 66,600 characters, rounded; the base64 fixture is
counted.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `e782dced` | red | 7 cases for the header reading, the part test, the mark and the part reading, and the count check; stubs answering no; the fixture | 117 s |
| `808d8530` | green | `claims_pgp_encryption`, `is_a_pgp_mime_part`, the column, the third question; 2 records new, 1 corrected by hand, 11 re-measured | 197 s |
| `19d3e253` | red | 11 cases across opening, the reader's composition, the page and the target, and the count check; the two variants with accessors that ignore them | 127 s |
| `41b700d5` | green | the opening, the shared reading, the mark covering both families, the carried reason; 3 records new, 4 corrected by hand, 35 re-measured | 193 s |
| this commit | docs | the advisory entry, the pages, the changelog, the ledger, this summary, the four marks | |

The first red was refused once by clippy before it landed, for two fixture helpers nothing
used yet; they went into the second red instead. The hook's own times are above; the first
refusal cost 46 s.

**Test counts, taken on 2026-09-26.** `cargo test --lib data::message_cache::how_it_arrived::`
13 (9 at the plan's premise). `service::signed_mail::` 122 (113 at the premise, 118 after
13-14). `application::opening_pgp::` 12 (5). `application::reading_a_message::` 21.
`service::pgp::` 17. `application::encrypted_mail::` 11. `presentation::reader_text::` 143.
`data::message_cache::` 804. `cargo test --test an_encrypted_message_is_not_left_unexplained`
14 (10 at the premise, 13 after 13-14). `wired` 77, `integration_tests` 26, `house_style` 74,
`the_words_that_say_nothing` 10, `every_number_carries_its_command_and_its_date` 26, all
passing. `bash scripts/audit.test.sh`: all cases pass.

**Acceptance readings.** `grep -c 'arrived_pgp_encrypted' src/data/message_cache/mod.rs` 1;
`grep -c 'fn claims_pgp_encryption' src/service/signed_mail.rs` 1. `tr -s '\n/' '  ' <
src/service/pgp/mod.rs | tr -s ' ' | grep -c 'is not read, and that gap'` 0. `tr -s '\n#' '  '
< .cargo/audit.toml | tr -s ' ' | grep -c 'make an opened PGP body a'` 0, and the same reading
finds "let decrypted content decide whether a picture is fetched" once. Row 145 `fixed` in
both halves; 641 to 643 in both halves, the JSON parsing to 643 entries, 582 open, 61 fixed,
matching the table and the frontmatter. No em or en dash added, none of the six words, and
carriage returns 0 on every document touched, by `tr -cd '\r' < FILE | wc -c`.

**GAP-05's "PGP/MIME ... mail is read"**, named:
`data::message_cache::how_it_arrived::tests::test_a_pgp_mime_message_is_marked_on_arrival`,
`application::opening_pgp::tests::pgp_mime::test_a_pgp_mime_message_to_alice_opens_to_its_words_with_her_key`
and the target's `test_an_opened_pgp_mime_message_is_its_words_with_no_pgp_sentence_in_the_bar`.
The box waits for 13-21.

## The fixture

GnuPG 2.4.9, the one Git for Windows ships, which wants POSIX paths: `GNUPGHOME=C:/g13` was
read as a path under the working directory and refused, and `/c/g13` worked. Alice's two keys
were decoded from `ALICE_PUBLIC` and `ALICE_PRIVATE` in `keys.rs` into the scratchpad.

```text
GNUPGHOME=/c/g13 gpg --batch --import alice_public.asc
GNUPGHOME=/c/g13 gpg --batch --armor --trust-model always \
    --encrypt -r alice@example.com -o inner.asc inner.eml
GNUPGHOME=/c/g13 gpg --batch --decrypt inner.asc | cmp - inner.eml
```

The private key was imported too, only so the last line could check the round trip. `inner.eml`
is a `multipart/mixed` holding a `multipart/alternative`, plain and HTML, saying "The figures
are in the minutes. See you Thursday.", the HTML half with a described, sized picture on
`https://tracker.example.com/chart.png`, and a file, `minutes.txt`. The armour went by hand
into the second part of a `multipart/encrypted` laid out the way Thunderbird lays one out: the
control part saying `Version: 1`, then `application/octet-stream; name="encrypted.asc"`. Every
line ends CRLF; the whole message is base64 in `PGP_MIME_TO_ALICE`, commands in its comment.

## Guard records

| Record | What |
|--------|------|
| a pgp/mime message whose protocol is quoted is still recognised | new, `signed_mail.rs`, 11 red |
| every arrival path marks a pgp/mime message | new, `how_it_arrived.rs`, 8 red |
| an opened pgp/mime message is its words, not the entity the key handed back | new, `opening_pgp.rs`, 2 red |
| a pgp/mime message that opened carries the mark decrypted content is known by | new, `encrypted_mail.rs`, 4 red |
| the reason a pgp/mime message did not open is carried to the reader | new, `reading_a_message.rs`, 1 red |
| an opened envelope hands back what windows wrote, not the buffer it sized | corrected by hand: 9 of 13-14's cases in files it never named, found red and unnamed by the re-measure, re-measured at 14 |
| an encrypted message that nothing ever recognised as encrypted | corrected by hand: two of this plan's cases assert the S/MIME half through the claim, re-measured at 12 |
| a page holding mail opened from its encryption fetches no picture | corrected by hand: this plan's page case, re-measured at 3 |
| the files inside an opened envelope are the files the reader lists | corrected by hand: this plan's files case, re-measured at 2 |
| 32 more flagged by the count check across the two greens | re-measured, each unchanged; one, "the form a signed message arrived in is really kept", in both runs |

The two records new in the first green gained the six opening cases in the second, by hand,
because opening asks the mark first and those cases mark their message the way the fetch
does; both were re-measured in the second run at the counts above.

Five records new; the arrived-since line at the head of `guards/guards.toml` went from 362 to
367. Premise 5's anchors held: `claims_pgp_encryption` is its own function after
`is_an_smime_envelope`, calling `split_headers_from_body`, `header_value` and `ContentType`
rather than copying `claims_encryption`'s lines, so "an encrypted message that nothing ever
recognised as encrypted" still names one place. The one anchor in a file this plan edited that
it had to keep, `from_what_was_kept(true, envelope.as_deref(), store)` in `encrypted_mail.rs`,
is unchanged; the PGP/MIME question went above it.

## Deviations from Plan

**1. [Shape] `for_pgp_mime` answers `Option<WhatTheEnvelopeSays>`, not an `OpenedPgpMime`.**
The plan put the opening in `put_together` beside 13-14's envelope. Four surfaces ask about
the files inside an envelope without going through `put_together`: the invitation, the answer
buttons, what opening changed, and `bytes_of_the_attachment` when a file is saved. All four ask
`encrypted_mail::for_message_opened_with`, so the PGP/MIME question is asked there, after the
S/MIME mark, and the answer is two new variants of the envelope's type. Nothing at any of
those call sites changed, and a file inside a PGP/MIME message is taken from it again when
saved, which a separate type would have left out. `encrypted_mail.rs` was not in the plan's
files.

**2. [Shape] The decrypted mark is `is_opened`, in `encrypted_mail.rs`, so the guard on it is
there.** The plan's record on `reading_a_message.rs` broke "the decrypted mark for PGP/MIME";
there is no such mark in `reading_a_message.rs` in this design. The record is on `is_opened`,
and `reading_a_message.rs` gets its own record for the one thing it does for PGP/MIME, carrying
the reason rather than asking again. Five records, not four.

**3. [Rule 2] The page and files cases are in `reader_text.rs`.** `the_renderer_for` and
`thread_parts` are private to it, and a page built through the public composers reads this
machine's picture setting, so the case would pass on a machine holding pictures back whatever
the rule did; 13-14's deviation 9 met the same thing. `reader_text.rs` was not in the plan's
files.

**4. [Rule 1] The fixture was re-encrypted between the red and the green.** The red carried an
`img` with `alt=""` and no size, and a page in the clear holds that back as decorative, so the
page case could not tell the rule from the fixture: it passed its own "in the clear it fetches"
half only by failing. Re-made with a described, sized picture by the same commands; nothing
else in it changed.

**5. [Shape] The fixture's inside is a `multipart/mixed` holding the plan's
`multipart/alternative` and a file**, so "its parts" has a part to show and a file can be taken
from the message when saved.

**6. [Shape] `is_a_pgp_mime_part` reads the name after the type.** RFC 3156 makes the part
`application/octet-stream`; a part typed otherwise and named `.asc`, `.gpg` or `.pgp` is taken
too, and the control part never is, whatever its name.

**7. [Shape] The every-arrival-path test's reading was not widened.** It asks that nothing
that ships calls `keep_signed_original` on its own, and the new question is a private call
inside `note_the_form_it_arrived_in`, which nothing outside can make. Its comment and the
module's now say three facts; the arrival case and its guard record hold the third question to
being asked.

**8. [Premise] The advisory paragraph moved.** The plan cited `.cargo/audit.toml:168-179`; 13-14's
rewrite put it at `:179-190`. Rewritten there, and the sentence above it that named only S/MIME.

**9. [Brief] The banned tool, again.** Three commands carried `sed_free`, `sed_unused` or
`sed_is_not_used` as do-nothing variable names, and one ran `sed -n 1,0p /dev/null`, which read
nothing and wrote nothing, before a command that mattered. No tracked file was touched by any
of them. Logged as observation 0860, which asks for a hook that refuses the name at the tool
boundary, since reading 13-14's deviation 10 did not stop it.

### Found and left

- What the key opens is read lossily as UTF-8 before it is taken apart, so words in another
  character set or a file sent as raw bytes inside a PGP/MIME message come out with
  replacement marks (ledger 642). Inline PGP had the same reading, for words only.
- A PGP/MIME message that opens to files and no words says it has no text or has not been
  downloaded, half false; 13-14 fixed the S/MIME case with its opened sentence, and PGP says
  none (ledger 643, with a recommended sentence for Pratik).
- A message fetched by the background sync keeps only calendar parts' bytes, so an S/MIME or
  PGP/MIME message's encrypted part is on this computer only once the reader has fetched the
  message itself. Shared with 13-14, not measured here, and not ledgered by this plan.

## Threat Flags

None beyond the register. T-13-15-01: `is_opened` covers `OpenedWithPgp`, so
`the_renderer_for` fetches nothing on a page holding one; a record on it and one on the parsed
body. T-13-15-02: nothing automatic reads the outcome; the audit entry is rewritten and the
suite passes. T-13-15-03: parsed in memory, a before-and-after case. T-13-15-04:
`is_a_pgp_mime_part` and a case. T-13-SC: no crate, no manifest line, `Cargo.lock` unchanged.
One surface the register did not name: a file inside a PGP/MIME message is decrypted again
on a worker when saved or read, as 13-14's are.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Fixed: 145 (PGP/MIME is not read). Opened: 641 (`unrun-verify`, phase 14's: a PGP/MIME
message from Thunderbird, Proton Mail or another real program), 642 (`todo`, the lossy
reading), 643 (`todo`, the files-only message). Both halves of each; 582 open of 643.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `e782dced`, `808d8530`, `19d3e253`, `41b700d5`: in `git log` on `13-15-pgp-mime-opens`.
- No file created under `src` or `tests`; the plan's summary path written.

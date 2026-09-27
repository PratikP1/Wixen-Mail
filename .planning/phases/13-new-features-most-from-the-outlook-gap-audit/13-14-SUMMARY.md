---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 14
subsystem: service::signed_mail, application::encrypted_mail, application::reading_a_message, application::answered_meetings, application::meeting_changes, application::pictures, presentation::html_renderer, presentation::reader_text, presentation::wx_app
status: complete
tags: [smime, encryption, privacy, reader, invitations, GAP-05]
requires: [13-13]
provides:
  - CertificateStore::open_the_envelope and WhatTheEnvelopeHeld, the Windows implementation through CryptDecryptMessage in signed_mail.rs's own crypt32 block, a refusal read by code
  - WhatTheEnvelopeSays::Opened, NotAddressedHere, TheKeyRefused and Damaged, with the four sentences
  - encrypted_mail::for_message_opened_with, the_parts_inside and the_file_inside, the seams a store is handed through
  - reading_a_message's among-functions and signature_for, a meeting and a signature asked of what an envelope opened to
  - meeting_changes::WhereItWasFound and Why::InsideEncryptedMail
  - HtmlRenderer::for_mail_opened_from_encryption and reader_text's the_renderer_for
  - ReaderAttachment::inside_the_envelope, the files inside listed and taken from the envelope again
affects: [13-15, 13-16, 13-18, 13-19, 13-21, 13-51]
tech-stack:
  added: []
  patterns:
    - "what an envelope opened to replaces the body, never joins it, so no clear part can wrap decrypted words"
    - "a store is handed to the application layer through a function that takes &dyn CertificateStore, so an in-memory key reaches the path the program runs"
key-files:
  created: []
  modified:
    - src/service/signed_mail.rs
    - src/application/encrypted_mail.rs
    - src/application/reading_a_message.rs
    - src/application/answered_meetings.rs
    - src/application/meeting_changes.rs
    - src/application/checking_signatures.rs
    - src/application/pictures.rs
    - src/presentation/html_renderer.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_reader.rs
    - src/presentation/wx_app.rs
    - tests/an_encrypted_message_is_not_left_unexplained.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - .cargo/audit.toml
    - docs/privacy.md
    - docs/comparison.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "A refusal is read by code, never by Windows' words: 0x8009200C is an envelope to nobody here, the codes Windows documents for a key that would not be used are the key refusing, and everything else is the envelope, which is what every damaged shape measured turned out to be."
  - "A store that cannot be asked is offered nothing, so a failure to ask is still never a no; a store that answers no is still offered the envelope, because a recipient named by key identifier is one only Windows can match."
  - "A change found inside encrypted mail is refused after what it would change is asked and before who sent it, so a message that would change nothing still says nothing."
  - "The files inside an opened envelope are listed in place of the envelope and taken from it again when saved or read; none is kept."
  - "The whole page holding opened mail fetches no picture, not only that message's section, because one fetch from a page is enough to say it opened."
metrics:
  duration: about 5 hours on 2026-09-26, about 95 minutes of it three guard re-measures of 7, 42 and 5 records
  completed: 2026-09-26
estimate:
  tokens: 130000
  tasks: 3
actuals:
  tokens: 60000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 14: S/MIME encrypted mail opens with a key in the Windows store Summary

An S/MIME message encrypted to a certificate whose key is in the Windows certificate store now
opens on every surface that shows a message: the text reader, the formatted window, the preview,
the conversation window and Shift+Space. Windows opens the envelope with `CryptDecryptMessage`
where the key lives; what was inside is parsed in memory and nothing of it is written to the
cache, so it is opened again each time it is read. The bar and the top of the message say "This
message was encrypted to your certificate and was opened here. Opening encrypted mail is
experimental.", the words follow, and the attachment list holds the files inside, each taken
from the envelope again when saved or read. The other three outcomes each have a sentence. A page
holding opened mail fetches no picture whatever the Reading tab says; a meeting inside is said
and offered its buttons, and an update or cancellation inside is said and not applied; a
signature sealed inside is checked and said.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `c12ec63f`, rounded, the base64 fixtures counted.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `96c5ac86` | red | six cases for opening an envelope and reading the refusal, the count check; the store answering "the key refused" to everything | 143 s |
| `4e942bd7` | green | `CryptDecryptMessage` in the crypt32 block, sized then written, the refusal read by code; 2 records new, 5 re-measured | 181 s |
| `04cc7075` | red | 17 cases across the envelope's reading, the reading's among-functions, the change decision, the page's renderer, the attachment rows and the target, and the count check; each stub ignoring what opened | 246 s |
| `050bfecd` | green | the reading, the four sentences, the files inside, the picture rule, `Why::InsideEncryptedMail`, the signature inside, the envelope with no words; 4 records new, 1 rewritten, 2 corrected, 42 re-measured | 300 s |
| this commit | docs | the advisory entry, the pages, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken on 2026-09-26.** `cargo test --lib service::signed_mail::` 118 (113 at the
plan's premise). `application::encrypted_mail::` 11 (7; three rewritten away, seven added).
`application::reading_a_message::` 19. `application::meeting_changes::` 19.
`presentation::html_renderer::` 93. `presentation::reader_text::` 141.
`application::answered_meetings::` 22. `application::checking_signatures::` 8.
`presentation::wx_reader::` 14. `application::pictures::` 44. `cargo test --test
an_encrypted_message_is_not_left_unexplained` 13 (10). `an_invitation_is_said_before_the_body`
10, `a_meeting_change_reaches_the_calendar` 19, `the_invitation_is_answered_from_the_reader` 12,
`theme_reach` 7, `wired` 77, `house_style` 74, all passing.

**Acceptance readings.** `grep -c 'fn CryptDecryptMessage' src/service/signed_mail.rs` 1;
`grep -c 'cannot open an S/MIME encrypted message yet'` 0. `grep -c 'Nothing here decrypts
anything' src/application/encrypted_mail.rs` 0; `grep -c 'InsideEncryptedMail'
src/application/meeting_changes.rs` 8. The joined reading of `.cargo/audit.toml`, `tr -s
'\n#' '  ' | tr -s ' '`: 'S/MIME decryption is refused' 0, 'S/MIME decryption now happens' 1.
`grep -c 'there is no OpenPGP at all' docs/comparison.md` 0; `grep -c '^### Signed and
encrypted mail' docs/USER_GUIDE.md` 1. No em or en dash added; carriage returns 0 on every
document touched, by `tr -cd '\r' < FILE | wc -c`.

**GAP-05's "S/MIME-encrypted ... mail is read"**, named:
`service::signed_mail::tests::test_an_envelope_made_for_the_keyholder_opens_with_the_key_held_in_memory`,
`application::encrypted_mail::tests::with_a_key_held_in_memory::test_an_envelope_for_a_key_held_here_opens_to_its_words_from_the_cache`
and the target's `test_an_opened_envelope_is_said_above_its_words_and_above_the_signature`.
The box waits for 13-21.

## The fixtures and the codes measured

Made with OpenSSL 3.5.7 in the scratchpad, from the keyholder certificate and key taken out of
`A_KEY_AND_ITS_CERTIFICATE`:

```text
openssl pkcs12 -in keyholder.p12 -passin pass:wixen-test -nokeys -clcerts -out keyholder.pem
openssl pkcs12 -in keyholder.p12 -passin pass:wixen-test -nocerts -nodes -out keyholder.key
openssl smime -encrypt -aes256 -binary -outform DER -in inner.txt -out sealed.p7m keyholder.pem
openssl smime -sign -signer keyholder.pem -inkey keyholder.key -in body.txt -out signed.eml
```

Three envelopes, each with its commands in its comment: a short `text/plain` note
(`ENVELOPE_FOR_THE_KEYHOLDER`, AES-256-CBC, the key wrapped with PKCS #1 v1.5), a message signed
by the keyholder and then sealed, and a note with an invitation. The Windows build of OpenSSL
writes `signed.eml` in text mode, which doubled the carriage returns of the signed part; they
were put back to one each and `openssl smime -verify -noverify -binary` checked the result before
it was sealed. RSA-OAEP and `authEnvelopedData` were not made again: the research's probe opened
both, and the fixture is the first of the three.

The codes, from a probe of `CryptDecryptMessage` against the in-memory key:

| Offered | First call (size) | Second call |
|---|---|---|
| the note, to the keyholder's store | 102 | opened, 94 bytes, OpenSSL's input byte for byte |
| the note, to an empty store | 102 | `0x8009200C` |
| Alice's envelope, to the keyholder's store | 86 | `0x8009200C` |
| the last byte of the ciphertext changed, and the 8th and 17th from the end | 102 | `0xC000003E` |
| a byte of the wrapped key changed (200, 300) | 102 | `0x80090027` |
| a byte 40 from the end changed | 102 | opened, to changed words |
| a byte of the structure changed (30) | `0x80093103` | |
| cut short, or no bytes | `0x80093102` | |
| words that are not an envelope | `0x8009310B` | |
| a signed document | `0x8009200A` | |

The row that opened to changed words is CBC doing what it does: an envelope in this mode carries
nothing that says its words were not changed, and only a signature does. The damaged case changes
the last byte for that reason, and says so.

## Guard records

| Record | What |
|--------|------|
| an envelope to nobody here is said to be addressed to nobody here | new, `signed_mail.rs`, 3 red |
| an opened envelope hands back what windows wrote, not the buffer it sized | new, `signed_mail.rs`, 5 red |
| a page holding mail opened from its encryption fetches no picture | new, `html_renderer.rs`, 2 red |
| an envelope opened here writes nothing to the cache | new, `encrypted_mail.rs`, 1 red |
| a meeting change found inside encrypted mail is said and not applied | new, `meeting_changes.rs`, 2 red |
| the files inside an opened envelope are the files the reader lists | new, `reader_text.rs`, 1 red |
| the preview pane carries the bar above the message | anchor rewritten to the preview's new renderer line, re-measured |
| an encrypted message that nothing ever recognised as encrypted | corrected by hand: 5 of this plan's cases, which mark their message through the claim, re-measured at 10 |
| the count of held-back pictures reaches the reader | corrected by hand: this plan's page case, re-measured at 4 |
| a message on its way out does not pick up this reader's own words | reads the profile; re-measured on a fresh one, as its note asks, unchanged |
| 37 more flagged by the count check | re-measured, each unchanged |

Six records new; the arrived-since line at the head of `guards/guards.toml` went from 356 to 362.
The plan named five; the sixth holds the attachment rows, which the plan did not name. The anchors
premise 5 named, `signed_mail.rs:122-128`, `:1391-1397`, `:1905-1908`, `:3383` and `:3441`, lie
outside everything this plan edited and are untouched; `what_the_settings_say` in
`html_renderer.rs` holds none and was not edited.

## Deviations from Plan

**1. [Premise] A damaged wrapped key is `NTE_INVALID_PARAMETER`, so not every `NTE_*` is the key
refusing.** The plan mapped `NTE_*` to the key refusing. A changed byte in the wrapped key gave
`0x80090027` and a broken padding `0xC000003E`, an NTSTATUS, and both are damage. The key
refusing is a named list, the codes Windows documents for permission, a silent context, a
cancelled prompt and a card; everything else is damaged. The list is unmeasured, because no key
here asks for anything, and says so; ledger 639.

**2. [Rule 2] The files inside are listed and can be saved and read.** The plan put the parts
"into the document in memory". Listing the envelope as the message's one file would have left a
person with a file nothing opens and no way to reach the files inside, so `ReaderAttachment`
gained `inside_the_envelope`, `attachments_of_part` lists the files inside, and
`bytes_of_the_attachment` in `wx_app.rs` opens the envelope again for one. A source reading in
the target holds the fetch to it.

**3. [Rule 2] The invitation inside is answered through the same reading.**
`answered_meetings::the_invitation_on` looks inside an opened envelope when the clear files carry
no meeting, and says where it was found, so the buttons offered are the ones a press answers and
the change decision knows the provenance without the window passing it. `answered_meetings.rs`
was not in the plan's files.

**4. [Shape] The four variants carry no `said` field.** The sentences are fixed, so they are four
constants and `said()` returns them; `Opened` carries the body, the files and the whole of what
was inside, the last for a signature sealed inside.

**5. [Shape] `EncryptedMessage::spoken` takes no argument.** Its three sentences said "Wixen Mail
cannot open it" for a message addressed here, which stopped being true; it is now the sentence
for a store that could not be asked, which is the only case left for it.

**6. [Shape] The provenance is asked after what the message would change.** The plan said
`InsideEncryptedMail` is answered "before anything else". Asked first, every invitation inside
encrypted mail would carry a sentence about a change it does not make, the reasoning of 13-13's
deviation 5; it is asked right after, before who sent it.

**7. [Shape] The picture sentence names no switch.** A page of opened mail says "N pictures were
not shown. Pictures in encrypted mail are never fetched, because fetching one would tell the
sender this message was opened." through a new `WhoseMessage::SomebodyElseSentEncrypted`, because
pointing at the Reading tab would send somebody to a switch that changes nothing here.
`pictures.rs` was not in the plan's files.

**8. [Rule 1] An envelope that opened to files and no words said it had not been downloaded.**
Found while writing the guide: the opened sentence went above "This message has no text, or it
has not been downloaded yet." It now stands where that sentence would have been. A case in the
target, taken red by hand before the fix and committed with it rather than as its own red.

**9. [Premise] Two readings of the reader were not possible where the plan put them.** The
page rule's case is in `reader_text.rs` with a renderer allowed to fetch, because the composers'
own renderer reads this machine's settings; the target's reading of the order is through the
public composition with a stand-in signature line.

**10. [Brief] Do-nothing tokens carrying a banned tool's name.** Five commands carried `sed_free`,
`sed_x`, `awk_free` or `sed_none` as variable names; neither tool ran and no file was written
through them. Logged as observation 0859. A name list written by Python on Windows carried
carriage returns into the first 42-record re-measure, which refused every name, and was run again.

### Found and left

- An opaque signed message, `application/pkcs7-mime` with `smime-type=signed-data`, shows no words
  whether it arrives in the clear or inside an envelope; the checker's `unwrapped_content` is read
  by nothing. Older than this plan and not ledgered by it.
- A picture sent inside an encrypted message is not shown (ledger 640).

## Threat Flags

None beyond the register. T-13-14-01: `the_renderer_for` holds every picture back on a page
holding opened mail, and what opened replaces the body rather than joining it; two records.
T-13-14-02: `Why::InsideEncryptedMail`; a record. T-13-14-03: nothing written, a before-and-after
case and a record. T-13-14-04: four constants, the mapping reads codes. T-13-14-05: a damaged or
empty envelope is an outcome, and a length Windows cannot be told is damage without a call; the
25 MB ceiling on kept files stands. T-13-SC: no crate and no manifest line; `Cargo.lock`
unchanged. One surface the register did not name: a file inside is decrypted again on a worker
when saved or read, and written only where the person saves it.

## Known Stubs

None. Each stub answered nothing in its red commit only.

## Ledger

Opened: 639 (`unrun-verify`, phase 14's: a key that asks for a PIN or lives on a card, where
Windows prompts, and the codes a cancelled prompt gives), 640 (`todo`, pictures inside an
encrypted message). Updated: 141 (the keyholder's envelopes open; none from Outlook or
Thunderbird), 142 (the four new sentences), 143 (opening asked of the same store; a real
certificate still unmet). Closed: none. Both halves of each; 580 open of 640.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `96c5ac86`, `4e942bd7`, `04cc7075`, `050bfecd`: in `git log` on
  `13-14-smime-encrypted-mail-opens`.
- No file created under `src` or `tests`; the plan's summary path written.

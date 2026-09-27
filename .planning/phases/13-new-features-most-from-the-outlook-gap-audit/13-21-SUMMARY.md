---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 21
subsystem: application::protecting, data::message_cache, application::mail_controller, presentation::wx_compose, presentation::editor_document, presentation::wx_app
status: complete
tags: [smime, pgp, signing, encryption, composer, GAP-05, "#52"]
requires: [13-14, 13-15, 13-17.1, 13-18, 13-19, 13-20]
provides:
  - application::protecting::Choice, what_protection_it_gets, WhatIsHeld, CannotProtect, at_send, what_is_held, how_it_goes
  - protection TEXT on outbox_queue and drafts
  - SendEmailRequest.protection and .held, MailController::send_email answering WentOut
  - Reached::Sign on G and Reached::Encrypt on Y; the composer's two boxes, may_it_go, tick_from_the_page
  - service::pgp::the_passphrase_signing_needs, pgp_keys::private_key_for and public_keys_for
  - wx_passphrase::ask_to_sign
affects: [13-34]
tech-stack:
  added: []
  patterns:
    - "one pure decision asked twice, at Send and when the message goes, from what is gathered each time"
    - "a refusal becomes the send's error in its own words, so the row stays queued and nothing goes out plain"
key-files:
  created:
    - src/application/protecting.rs
    - tests/signing_and_encrypting_from_the_composer.rs
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-21-SUMMARY.md
  modified:
    - src/application/mod.rs
    - src/application/allowed.rs
    - src/application/mail_controller.rs
    - src/application/pgp_keys.rs
    - src/application/draft_message.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/outbox.rs
    - src/data/message_cache/drafts.rs
    - src/data/message_cache/accounts.rs
    - src/service/pgp/mod.rs
    - src/service/pgp/keys.rs
    - src/service/protocols/smtp.rs
    - src/presentation/editor_document.rs
    - src/presentation/wx_compose.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_passphrase.rs
    - src/presentation/ui_types.rs
    - src/presentation/managers.rs
    - tests/integration_tests.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/comparison.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/REQUIREMENTS.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-34-PLAN.md
key-decisions:
  - "Decision 23 as the plan took it: S/MIME first when the sender has a certificate and, to encrypt, every recipient one; OpenPGP otherwise; never greyed; the choice kept on a draft."
  - "Encrypting with a Bcc is refused, since both families write every recipient's key on the envelope; a question for Pratik in ledger 659."
  - "The check at Send reads the active account's address, the one the Outbox sends from, because the From list chooses nothing the Outbox reads (ledger 660, handed to 13-34)."
  - "The Task 2 cases live in protecting.rs, not mail_controller.rs, which 24 records count."
metrics:
  duration: about 7 hours on 2026-09-27, about an hour of it finding why a red stub crashed three window targets under the gate
  completed: 2026-09-27
estimate:
  tokens: 150000
  tasks: 4
actuals:
  tokens: 42000
  tasks: 4
  commits: 8
---

# Phase 13 Plan 21: Sign and Encrypt in the composer Summary

The composer has two check boxes beside Schedule, **Sign (experimental)** on `Alt+G` and
**Encrypt (experimental)** on `Alt+Y`, both reached from inside the message, where the page
says "Sign on" or "Encrypt off". Send, `Ctrl+Enter` and `Alt+N` ask
`application::protecting` before anything is queued: S/MIME when the sender has a certificate
for the address the message goes from and, to encrypt, every recipient has one kept; OpenPGP
otherwise; and when neither can, the reason is said and shown, the window comes back as it
was, and nothing is queued. A locked PGP key asks for its passphrase at Send. The choice goes
on the outbox row and the draft, and the send loop decides again from what is held when the
message goes, so a key removed in between leaves the row queued with the reason; nothing falls
back to plain. The Sent copy is the bytes that went, and the sender's own key opens it. The
result line says how it went: "from the Outbox, signed with S/MIME". GAP-05 is ticked.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and the
shortcuts page against `main` at `15ba864e`, 119,182 characters, and the pages, ledger and
this summary, about 12,000.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `a9c3f529` | red | 18 decision cases, 2 outbox and 2 draft cases, the count check | 288 s |
| `33a4a866` | green | the decision, the columns, the stores; 2 records new, 1 re-measured | 234 s |
| `555bd850` | red | 7 cases through `from_queued`, the gathering and `outgoing`, the count check | 130 s |
| `d2038e1e` | green | the request carries the choice, the loop gathers, a refusal stays queued; 1 record new, 1 corrected | 343 s |
| `af8c17d7` | red | 4 `editor_document`, 4 `protecting` and 16 target cases, the count check | 270 s (refused twice first, 279 s and 273 s) |
| `1a9d8298` | red | the passphrase sentence at Send, the count check | 113 s |
| `24ecb71d` | green | the boxes, the check at Send, the passphrase, the result line, the shortcuts page; 2 records new, 6 re-measured, 1 corrected | 393 s |
| this commit | docs | the pages, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken on 2026-09-27.** `cargo test --lib application::protecting::` 32 (new).
`data::message_cache::outbox::` 27 (25). `data::message_cache::drafts::` 8 (6).
`application::mail_controller::` 71 (71). `application::sending_later::` 47 (47).
`presentation::editor_document::` 88 (84). `presentation::wx_compose::` 43 (43).
`application::allowed::` 27 (27). `presentation::wx_passphrase::` 2 (1). The target
`signing_and_encrypting_from_the_composer`, 20. `undo_send_is_where_somebody_looks` 7.
`integration_tests` 26 and `wired` 77 before the first green, since the outbox and draft
tables changed.

**Acceptance readings.** `grep -c '"protection"' src/data/message_cache/mod.rs` is 2.
`grep -c 'Si&gn (experimental)' src/presentation/editor_document.rs` is 1, and the same for
`Encr&ypt (experimental)`. The shortcuts page's Composition Window names `Alt+G` and `Alt+Y`.
`grep -c 'nothing goes out signed or encrypted' docs/comparison.md` is 0. GAP-05's line
begins `- [x] **GAP-05**`.

**The composer's letters, before and after.** Before: F T C B S N H U R O P A D I L from
`Reached`, and E for the People found list. After: the same and G and Y. The dialog's own
`&` labels in `wx_compose.rs` belong to its menus and to the spelling, table and preview
dialogs. The page's Composition Window documented H N U R A F T C B S E D I L before; it
documents G and Y beside them now.

**Reachability.** `signed_mail::sending::sign_detached` and `encrypt_to`, and
`service::pgp::sign_detached` and `encrypt_for`, now have a non-test caller: the send loop's
`protected_as_asked` fills the request, `mail_controller::outgoing` asks
`what_protection_it_gets`, and `smtp::build_message` calls them through `Protection`. The
composer's Send reaches `protecting::at_send` through `wx_app.rs`'s `the_protection_check`.

## Guard records

| Record | What |
|--------|------|
| a message asked to go encrypted to somebody with no key is refused, not sent plain | new, `protecting.rs`; corrected twice as new cases reached it, 6 red |
| a draft keeps whether it is to go signed or encrypted | new, `drafts.rs`, 1 red |
| a queued message whose key went is refused at sending, not sent plain | new, `mail_controller.rs`, 1 red |
| alt y inside the message reaches the encrypt box | new, `editor_document.rs`, suite the target, 1 red |
| send keeps a message it cannot protect in the composer | new, `wx_compose.rs`, suite the target, 3 red |
| a picture's description survives the trip from the composer to the page; a file dropped on the message is turned away rather than opened; a marker is refused when anything at all precedes it in its parent; the first character typed after a closing delimiter is plain; the passphrase field is named for what goes in it | flagged by the count check, re-measured, unchanged |

Five records new; the arrived-since line went from 392 to 397. Anchors near the edits held:
`mail_controller.rs`'s `fn addresses(field: &str)` record reads the same text inside
`pub(crate) fn addresses(field: &str)`; its `message_id` line in `outgoing` is untouched;
`keys.rs`'s four and `smtp.rs`'s five anchors are untouched, `as_it_would_go` going in below
`build_message`; `wx_compose.rs`'s anchors are outside the toolbar block and the dispatch arm;
`editor_document.rs`'s are outside `Reached`.

## Deviations from Plan

**1. [Rule 2] A Bcc is refused when encrypting.** Both families write every recipient's key
on the envelope, where anybody who received the message can read it, so encrypting to a Bcc
recipient shows them. The plan did not say; the refusal is the safe default, and ledger 659
asks Pratik about separate copies.

**2. [Shape] More refusals than the plan's three.** `CannotProtect` has `NoKeyOfYourOwn`,
`NoKeyFor` with which kind the sender holds, `NoOneKindReachesEveryone` for recipients split
between a certificate and a key, `ABlindCopyWouldShow` and `TheKeyIsLocked`. Each says what
to do and ends "Nothing was sent." A message to yourself needs nothing kept for you, since the
sender is always encrypted to.

**3. [Shape] What is held is gathered by the send loop into the request.** `outgoing` stays a
function of the request: `SendEmailRequest.held` is filled by `protected_as_asked` in
`wx_app.rs`, only for a message asked to be protected, and a request with nothing gathered
refuses rather than sending plain. `MailController::send_email` answers `WentOut` with how it
was protected, for the result line.

**4. [Scope] The Task 2 cases are in `protecting.rs`.** `mail_controller.rs` is counted by 24
records, and a case added there would have flagged all of them; the cases go through the
public `from_queued` and `outgoing` from the new module, and `smtp::as_it_would_go`, under
`cfg(test)`, reads what `build_message` makes.

**5. [Scope] Files the plan did not list.** `wx_passphrase.rs`, for the sentence "Type it to
sign this message" at Send, with a red of its own; `service::pgp::{mod,keys}.rs`, for
`the_passphrase_signing_needs`, which asks what `detached_signature` asks without signing;
`ui_types.rs` and `managers.rs`, for `CompositionData.protection` and the result line;
`draft_message.rs`, `accounts.rs`, `integration_tests.rs` and `theme_reach.rs`, struct literals.

**6. [Premise] The From list chooses nothing.** `ComposeData.account_index` is read by
nothing in `wx_app.rs`; the row goes under the active account. So the check at Send reads the
active account's address, the one the message goes from. Ledger 660; 13-34's plan now names it.

**7. [Test] Three readings in the new target changed with the green.** Its source readings now
ignore spaces and line breaks, which the project asks of a reading and which rustfmt made
necessary; the dispatch reading asks for the `tick_from_the_page(...)` call, the `from_boxes`
reading stops before rustfmt's trailing comma, and the draft reading names the arm's binding,
`data`. The green commit says so.

**8. [Gate] A red stub crashed three window targets, under the gate only.** The first red
built the boxes after Cancel in the button row, visible. Under `scripts/check.sh`, twice
through the hook and once by hand, `a_marker_counts_at_the_start_of_any_line`,
`a_signature_follows_the_from_account` and the new target ended with 0xc000041d after a
`wxStaticCast` assert; by hand, alone, after the gate's lib runs, after clippy with every
feature, and as the gate's whole list of 134 targets, all passed. Hidden, the gate passed and
the red went in; in the toolbar, where the green puts them, all three pass under the gate. Not
diagnosed; ledger 661. The red commit's message says "unlabelled boxes after Cancel"; the next
commit's message corrects that.

**9. [Brief] The banned tool's name, five times.** Four do-nothing assignments or pipeline
stages carrying one of the two banned names, and one pipeline stage that was a command name
beginning with one, which the shell did not find (exit 127). Nothing ran and nothing was
written. Logged as observation 0868.

### Found and left

- `pgp_keys::WHAT_KEYS_CAN_DO_HERE`, the key manager's box, says nothing about signing and
  encrypting what you send. It says nothing false, and each clause is pinned by a case in a
  file one record counts; the guide says it instead.
- `own_certificate_for` still takes the first certificate Windows lists for the address, as
  13-19 left it and ledger 654's text says.

## Threat Flags

None beyond the register. T-13-21-01: refused at Send and again at send time, never plain,
the choice on the row and the draft, and three records (the refusal, the queued key gone, Send
keeping the message). T-13-21-02: the sender is always encrypted to, and a case opens the Sent
copy with the sender's key. T-13-21-03: never disabled; the reason said at Send. T-13-21-04:
the result line names the family and the guide says the order. T-13-SC: no crate added.

## Known Stubs

None. Ledger 654 and 656, the two stubs 13-19 and 13-20 opened, are fixed by the path above.

## Ledger

Fixed: 654, 656. Opened: 658 (`unrun-verify`, the boxes and sentences heard), 659 (`todo`, the
Bcc question), 660 (`todo`, the From list, for 13-34), 661 (`deviation`, the stub crash). Both
halves of each; 595 open of 661.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move.

## Self-Check: PASSED

- `a9c3f529`, `33a4a866`, `555bd850`, `d2038e1e`, `af8c17d7`, `1a9d8298`, `24ecb71d`: in
  `git log` on `13-21-sign-and-encrypt-in-the-composer`.
- `src/application/protecting.rs` and `tests/signing_and_encrypting_from_the_composer.rs`
  exist.

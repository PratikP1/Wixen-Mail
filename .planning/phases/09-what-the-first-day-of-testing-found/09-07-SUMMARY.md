---
phase: 09-what-the-first-day-of-testing-found
plan: 07
subsystem: reading a message, pgp, s/mime, the reader window, the conversation window, the preview pane, guards
tags: [pgp, smime, signature, reader, preview, conversation, composition, wired, guards, changelog]

requires:
  - phase: 04-writing-and-reading-a-message-in-full
    provides: "04-09: service::pgp with one key and one message, opening_pgp::for_body and the_body_to_show, ReaderDocument::with_pgp, and the GnuPG-made fixtures; 04-03 and 04-05: checking_signatures, encrypted_mail, with_signature, with_smime_envelope and the ordering rule at HOW_IT_WAS_CHECKED"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-06: the pattern of a red commit with stubs answering the tree as it is, and the count check's remedy run and read before the green"
provides:
  - "application::reading_a_message: WhatAMessageShowsAndSays (the body to show, and WhatIsSaidAboutIt: the PGP finding, the envelope, the signature), for_message asking the cache and put_together for a caller with the two answers, and signature_check_for and envelope_check_for moved out of the window"
  - "ReaderDocument::with_what_is_said: the three findings folded in the one order that keeps each spoken, written once"
  - "ConversationPart.said, and reader_text::conversation folding for one message and saying each finding where one of several begins through one_of_several; conversation_html doing the same on the page through ThreadPart.before_the_body"
  - "reader_text::preview_html and HtmlRenderer::render_thread_under_a_bar: the preview's page with the top of the bar as a region named Security warning above the message"
  - "wx_app::what_a_message_shows_and_says, the window's one seam to the composition, asked by all six surfaces; the_preview_of, the preview arm as a function"
  - "tests/wired.rs: THE_SURFACES, a table of six with the call each has to make, surfaces_that_bypass_the_composition, and a companion that splices each out in memory"
  - "service::pgp::for_tests: the GnuPG-made key and message for the tests of modules that open mail"
  - "Five guard records measured, one rewritten, four older ones corrected from what the remedies found"
affects: [09-08 onward, which write changelog entries under Unreleased; 09-10, the phase's closing read, which reads FOUND-10 clause by clause; #52's reading half, which now has one place to add S/MIME decryption or PGP/MIME when they are built; #49, whose key manager changes what for_body finds and nothing else]

actuals:
  tokens: 30700
  tasks: 3
  commits: 8

tech-stack:
  added: []
  patterns:
    - "When several surfaces have to say the same things about one message, one composition asks the questions and one fold applies the answers, and a source-reading guard holds every surface to the composition by name from a table rather than by the two that happened to work when it was written"
    - "A finding about one message of several is said at that message, under its own heading, on every surface that shows the thread; the bar stays off a thread because one sentence over five messages is heard as covering all five"
    - "A source-reading guard that anchors on a function's opening line is coupled to the function's location: a move that leaves the anchor behind stops the guard's target compiling rather than failing a test, so the guard edit belongs with the move"
    - "A companion that splices a bypass into a file's text in memory reads the tree first and refuses over a real bypass, so it goes red beside the guard it holds and a record naming one of them names both"

key-files:
  created:
    - src/application/reading_a_message.rs
  modified:
    - src/application/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/reader_text.rs
    - src/presentation/html_renderer.rs
    - src/service/pgp/keys.rs
    - src/service/pgp/mod.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "The composition takes Option<&MessageCache>, a row id and the From header rather than the window's MessageItem, so the application layer does not reach into presentation types; the window keeps one adapter, what_a_message_shows_and_says, which is also the one line the guard holds every surface to"
  - "The body to show stays on ConversationPart as body and the three findings ride beside it as said, rather than one nested value replacing body, because every reader of a part reads its body and four of six surfaces never read the findings by name"
  - "The stored preview body, message_preview, stays the body as it arrived and not the words the key opened it into, because a reply quotes it and a reply that quoted the decrypted words would send them on in clear"
  - "The preview renders the top of the bar and one line saying the rest is in the message window, not the whole bar: a minute of certificate talk ahead of every signed message in a pane somebody arrows through is what said_before_the_message exists to avoid"
  - "A conversation of several says the PGP reason and the envelope sentence under the message's own heading and still says nothing about a PGP signature there, because the signed sentence comes from the form of one body and a thread has a form per part; ledger 497 carries it"
  - "The key import's comment was corrected to what the body does, announce only, rather than the body made to do what the comment said, because that is a behaviour change wanting a red of its own and this task's one red was spent; ledger 496 owes the visible line"
  - "ThreadPart gained a field rather than the reason being folded into the body's text, because a Multipart body renders its HTML half and a sentence prepended to the plain half would never be seen"

patterns-established:
  - "One question asked in one place, one fold applied in one order, and a guard that reads a table of every surface: the shape that stops two working call sites from being the whole feature"
  - "A remedy that runs against the code the records will be measured on: the count check fires on the red commit and its remedy runs after the green, once, and its findings are corrected from what went red rather than from what was expected"

requirements-completed: []

coverage:
  - id: D1
    description: "One function composes, for a message and its stored body, the opened body, the PGP finding, the S/MIME envelope sentence and the signature verdict, and every surface that shows a message asks it, six of them"
    requirement: FOUND-10
    verification:
      - kind: unit
        ref: "src/application/reading_a_message.rs#test_a_message_encrypted_to_the_imported_key_is_shown_as_its_words"
        status: pass
      - kind: unit
        ref: "src/application/reading_a_message.rs#test_an_armoured_message_with_no_key_here_says_why_and_keeps_its_armour"
        status: pass
      - kind: unit
        ref: "src/application/reading_a_message.rs#test_a_signed_message_in_the_cache_has_its_verdict_and_an_enveloped_one_its_sentence"
        status: pass
      - kind: unit
        ref: "src/application/reading_a_message.rs#test_the_envelope_is_folded_in_before_the_signature_so_it_is_spoken"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_opening_a_message_tries_the_pgp_key_and_says_why_it_did_not_open"
        status: pass
      - kind: integration
        ref: "tests/wired.rs#test_a_surface_that_stopped_asking_the_composition_is_named"
        status: pass
    human_judgment: false
  - id: D2
    description: "The preview pane's document carries the bar above the message, so a signed message previews as signed and a PGP message previews opened or with the reason it did not"
    requirement: FOUND-10
    verification:
      - kind: unit
        ref: "src/presentation/reader_text.rs#encryption_tests::test_the_preview_carries_the_bar_above_the_message"
        status: pass
      - kind: unit
        ref: "src/presentation/reader_text.rs#encryption_tests::test_the_preview_says_why_a_pgp_message_did_not_open"
        status: pass
      - kind: unit
        ref: "src/presentation/reader_text.rs#encryption_tests::test_a_whole_conversation_says_where_a_message_did_not_open_and_heads_it_with_no_verdict"
        status: pass
      - kind: unit
        ref: "src/presentation/reader_text.rs#encryption_tests::test_a_page_of_several_messages_says_where_one_did_not_open"
        status: pass
      - kind: unit
        ref: "src/presentation/reader_text.rs#encryption_tests::test_opening_one_message_as_a_page_folds_what_is_said_in_order"
        status: pass
    human_judgment: false
  - id: D3
    description: "The wired.rs guard names every surface, the two changelog entries are corrected by dating, and the stray rustdoc sits with the function it describes"
    requirement: FOUND-10
    verification:
      - kind: integration
        ref: "tests/wired.rs#THE_SURFACES, six rows, read by test_opening_a_message_tries_the_pgp_key_and_says_why_it_did_not_open"
        status: pass
      - kind: other
        ref: "grep -c 'Corrected on 2026-09-16' docs/changelog.md answers 3; sed -n '/fn import_a_mailbox(/,+1p' is preceded by the picker rustdoc and import_a_pgp_private_key by its own only"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the bar reads well by ear on the preview and in a thread, and whether a real correspondent's key opens anything"
    requirement: FOUND-10
    verification: []
    human_judgment: true
    rationale: "FOUND-10's last [S] line, ledger 495. Structure is held by tests against a key and a message GnuPG made and a signed message OpenSSL made; experience is the tester's"

duration: 1h48m
completed: 2026-09-17
status: complete
---

# Phase 9 Plan 07: One composition of what a message shows and says, asked by all six surfaces Summary

**One function, `application::reading_a_message::for_message`, offers a message's armour to the
key on this computer, takes the body to show, and asks what its S/MIME envelope and its
signature were worth; one fold, `ReaderDocument::with_what_is_said`, applies the three answers
in the order that keeps each spoken; and all six surfaces that show a message ask it, where two
did before: the text reader, Shift+Space, the Formatted reader that is the default, the
conversation window as headings, the whole conversation in the text reader, and the preview
pane, which now carries the bar. A thread says why a PGP message inside it did not open under
that message's own heading. `tests/wired.rs` names all six from a table, with a companion that
splices each out. Proven against the key and the message GnuPG made and the signed message
OpenSSL made; whether any of it reads well by ear, and whether a real correspondent's key opens
anything, is the tester's.** #51 closed with the merge commit. Nothing pushed.

## Performance

- **Duration:** about 1 h 48 min from the branch to the merge, of which about 27 minutes were
  the four guard remedy runs, 13 the two whole gates, and 7 the merge's gate on `main`
- **Started:** 2026-09-16T23:06Z (first commit 23:11:36Z)
- **Merged:** 2026-09-17T00:54:56Z at `f990d023`
- **Tasks:** 3
- **Files modified:** 10 (1 created)

## What landed, per surface

| Surface | Before | Now |
|---|---|---|
| The text reader, `open_in_the_text_reader` | asked `for_body`, `the_body_to_show`, `envelope_check_for` and `signature_check_for` itself and folded the three by hand in the right order, with the order as a comment | asks `what_a_message_shows_and_says` and folds through `with_what_is_said` |
| Shift+Space, `read_the_whole_message` and `whole_message_reading` | the same, with the same comment | the same seam and the same fold; `whole_message_reading` takes the composition's value |
| The Formatted reader, `open_single_message` | took `signature_check_for` alone and handed it to the page, which folded `with_signature` alone; a PGP message met "cannot open it" over the armour after a key was imported, with the default setting | builds its one part through the seam, the body to show and all three findings on the part; `show_conversation_as_page` lost its signature argument and `reader_text::conversation` folds for one message |
| The conversation window as headings, `conversation_parts` for `show_conversation_as_page` | built parts from the stored bodies with nothing asked; the window passed `NotSigned` for the thread on purpose | each part built through the seam, armour opened where the key opens it; `one_of_several` says why a part did not open and what its envelope says under that part's heading on the page, through `ThreadPart.before_the_body` |
| The whole conversation in the text reader, `open_conversation` | `reader_text::conversation` over the same parts, nothing asked, no bar | the same parts through the seam; the text says each part's finding where it begins, and the bar stays off the thread |
| The preview pane, the `MessageBodyLoaded` arm | `conversation_html` with one part and nothing asked, no bar at all | `the_preview_of`, a function, builds its part through the seam and renders `preview_html`: the top of the bar as a region named "Security warning" above the message, and one line saying to open the message for the rest |

The stored preview body, `message_preview`, is still the body as it arrived. A reply quotes it,
and a reply that quoted the decrypted words would send them on in clear (T-09-23).

## The composition

`WhatAMessageShowsAndSays { body, said: WhatIsSaidAboutIt { opened, envelope, signature } }`.
`for_message(cache: Option<&MessageCache>, message_row_id, from, body)` asks the two cache
questions and hands them to `put_together(body, envelope, signature)`, which offers the armour
to the key through `opening_pgp::for_body` and takes `the_body_to_show`. The two helpers moved
out of `wx_app.rs` with their reasoning; the 406-microsecond paragraph is the module comment,
because it is about the whole composition running on the interface thread and not about one of
its questions. `WhatIsSaidAboutIt::nothing()` is called by no surface, on purpose: a part built
with it is a part that never asked, which is what the guard records use to break the surfaces.

`ReaderDocument::with_what_is_said` is `with_pgp`, then `with_smime_envelope`, then
`with_signature`; the reason is on the function once and no longer as a comment at each caller.

## Task commits

| Commit | What |
|---|---|
| `b2703002` | test(09-07): the red half of task 1, five named, two green against the stubs and said so; `service::pgp::for_tests` |
| `b3314356` | feat(09-07): the composition, the helpers moved, the seam, the text reader and Shift+Space asking it, two records and one rewritten, the two `wired.rs` guards renamed to the seam |
| `175d5383` | test(09-07): the red half of task 2, five readings in `reader_text` and the count check named, `ConversationPart.said` with the bypass at every site |
| `6ac55a7c` | feat(09-07): the four surfaces, `one_of_several`, `preview_html`, `render_thread_under_a_bar`, `the_preview_of`, two records, four older records corrected, the changelog |
| `8bb86e39` | test(09-07): the red half of task 3, the companion against a stub reading, the count check named |
| `8beb2e56` | feat(09-07): the reading, the guard over six, three rustdoc blocks home, one record, two records corrected |
| `34158a43` | docs(09-07): `nothing()`'s comment says who calls it |
| `f990d023` | Merge 09-07 into `main` |

Branch `every-surface-asks-what-a-message-says` from `main` at `0bc19473`. Not pushed; 68
commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red named five of seven. Against stubs that never offered the armour and folded only
the signature, the GnuPG message stayed armour, the no-key reading found `None`, the cache
reading found `NotSigned` over a kept original, the ordering reading found the envelope below
the cut, and the fold reading found the general sentence unnarrowed. Two passed against the
stubs and the message says which: an ordinary message unchanged with nothing said, and nothing
kept meaning not signed and not encrypted, which is exactly what a stub answering nothing
answers. The gate in `red` mode ran exactly the five. The green's first run stayed red on the
cache reading because the two saves shared a uid and a Message-ID and the second replaced the
first row; each message has its own uid now, and the reading says why.

Task 2's red named five readings and the count check, which fired on `reader_text.rs` going
from 120 to 125. Against a `preview_html` that rendered the page with no bar and two composers
that read nothing from the part, all five were red for their own reasons. The remedy for the
eleven records the count check named ran after the green, against the code the records are
measured on, and found four not what they said, all corrected below.

Task 3's red named the companion and the count check, which fired on `wired.rs` going from 72
to 73. Against a reading that saw nothing, the companion's first assertion held and its splice
loop found nothing named. Green with the reading built: six surfaces spliced one at a time,
each named and no other.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/reading_a_message.rs`, `src/application/mod.rs` | `--lib application::reading_a_message`, and on the red commit `--lib application` as well because `mod.rs` changed, which ran the whole application layer |
| `src/service/pgp/keys.rs`, `src/service/pgp/mod.rs` | `--lib service::pgp::keys` and `--lib service::pgp` on the red commit |
| `src/presentation/reader_text.rs` | `--lib presentation::reader_text` on every commit that touched it, and the coupled `an_encrypted_message_is_not_left_unexplained` |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` (199 tests) and the eleven coupled targets `--suites-for` answers for it, from `a_whole_folder_moves_both_bounds` to `one_sort_is_checked` |
| `src/presentation/html_renderer.rs` | `--lib presentation::html_renderer` (83 tests) on the second green |
| `tests/wired.rs` | `--test wired` on every commit that touched it |
| `guards/guards.toml`, `docs/changelog.md` | no scoped target; rode the green commits, and the seven `house_style` tests read `guards.toml` on every commit |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `34158a43`, output to a file and the exit status read
directly, never piped: the first run exited 101 in 311 s with one failure, `a_move_says_what_has_not_been_sent`'s
`test_a_note_on_an_account_with_a_calendar_server_has_something_to_be_sent` panicking with "No
default store has been set" from the credential store, which is ledger 374's keyring race in a
test this plan never touched. The one permitted retry exited 0 in 382 s: 7,842 passed and none
failed over 66 result lines, the release build included; 13 more than 09-06's 7,829, which is
this plan's 7, 5 and 1. `main`'s hook ran `all` again on the merge: 7,842 and none failed.

`bash scripts/check.sh --suites-for guards/guards.toml src/application/reading_a_message.rs`
answers nothing: the module's tests are `--lib` tests reached by the module filter. For
`reader_text.rs` it answers `an_encrypted_message_is_not_left_unexplained`; for `wx_app.rs` the
eleven above, unchanged.

## Guard records

847 records by the TOML reader before, 852 after; census 802 + 45 before, 802 + 50 after, the
line at `guards/guards.toml:84` moved in each green commit. Five new, one rewritten, four older
ones corrected, all measured through `scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS`
untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| a message the key opens is shown as its words and not as its armour | `reading_a_message.rs` | `the_body_to_show` skipped | 1 | rebuild 40 s, run 48 s |
| the envelope is folded in before the signature, so it is spoken | `reader_text.rs` | signature folded before the envelope | 1, then 2 | 40 s, 48 s; 29 s, 48 s |
| the preview pane carries the bar above the message | `reader_text.rs` | the page rendered under no bar | 2 | 34 s, 48 s |
| one message of several says why it did not open where it begins | `reader_text.rs` | the reason dropped from `one_of_several` | 2 | 32 s, 48 s |
| a surface that shows a message without asking the composition is named | `wx_app.rs`, `wired` | the composition taken out of `the_preview_of` | 1, then 2 | 16 s, 2 s; 18 s, 2 s |
| a PGP message the reader window never offers to the key (rewritten) | `wx_app.rs`, `wired` | the seam taken out of the text reader | 1, then 2 | 16 s, 2 s; 16 s, 2 s |

**What the remedies found.** The count check fired twice, on `reader_text.rs` (eleven records)
and on `wired.rs` (fourteen), and each remedy was run once after its green and read. Four older
records were not what they said: `body_safety`'s form record reddened four tests it did not
name, three of the new module's and the preview's reason reading, 18 now from 14; the
nothing-to-read record and this plan's own ordering record each reddened the page's ordering
reading, written hours after the ordering record was; and the four-sentences record's break had
moved with the arms into `the_reason_it_did_not_open`. Then the `wired.rs` remedy found the new
call-site record and the rewritten text-reader record short by the companion, which reads the
tree before splicing and refuses over a real bypass; the new record's own comment had said the
companion was unmoved, and the first measurement corrected the comment. All corrected from what
went red and measured again, each reddening exactly what it names. The ordering record going
short within hours is the shape `CLAUDE.md` warns of, and it is why the remedy runs after the
green and not before.

Counts written: `reading_a_message.rs` 7, `reader_text.rs` 125 (was 120), `wired.rs` 73 (was
72), `wx_app.rs` 199 (unchanged; 54 records name it now, was 53), `html_renderer.rs` 83
(unchanged, 6 records). The count check fired on both red commits and on nothing after its
remedy.

## Premises the tree contradicted

1. **Task 1 could not be green without editing `tests/wired.rs`**, which the plan listed under
   task 3 with "or the guard is updated in task 3 to the new name". Two guards anchor on
   `fn signature_check_for(` in `wx_app.rs` and `body_of` panics on a missing opener, so the
   move stopped `cargo test --test wired` compiling its assertion. Both guards were renamed to
   the seam in task 1's green; task 3 widened the one and rewrote the other's page clause.
   Observation 616.
2. **The Formatted branch had to change in task 1 to compile at all**: it called
   `signature_check_for`. It took its verdict from the composition and folded only the
   signature until task 2 rewired it, and the commit message says so.
3. **Two stray rustdoc blocks sat above `import_a_pgp_private_key`, not one**: the mailbox
   import's, which the issue named, and above it the folder loader's ("Read a folder's messages
   out of the cache", with `limit`'s paragraph), which belongs to `load_folder_messages` 800
   lines down. A third sat above `the_accounts_in_the_tree`: `load_module_data`'s longer
   comment, orphaned from a function that has a shorter one. All three are home; the first line
   above `fn import_a_mailbox(` is "Bring a mailbox in from a file, keeping the folders it was
   in." and above `fn import_a_pgp_private_key` is "Read a PGP private key in from a file and put
   it in the credential store."
4. **Record 642's break moved and the plan did not say so.** "a PGP message the reader window
   never offers to the key" broke the two `opening_pgp` calls in `open_in_the_text_reader`, which
   task 1 moved into the composition. Rewritten to take the seam out, on the plan's own rule
   that a moved break is measured again by hand rather than edited until it applies.
5. **Three changelog entries wanted dating, not two.** The armour entry's known limitation, "a
   conversation read as one document says nothing about one message's form", is half true now:
   a PGP message inside a thread says why it did not open, and an envelope says what it says; a
   PGP signature inside a thread is still not mentioned. Dated beside the two the plan named.
6. **`wx_app.rs` is named by 53 records, not the README's 50** at `524ff24f`; 54 now. `reader_text.rs`
   10 and `wired.rs` 14 held.
7. **The plan's second task 2 record went on behaviour rather than on a call site.** The plan
   suggested a `conversation_parts` bypass with `suite = "wired"` if task 3's reading reddened;
   that is task 3's record, and task 2's second record breaks `one_of_several` instead, which
   reddens the two thread readings. Five records in all, as the plan counted.
8. **The key import's "put in the status bar" was corrected in the comment, not built.** The
   body announces and nothing else, `Accessibility::announce` reaches the screen reader only,
   and the visible equivalent `CLAUDE.md` asks for is a behaviour change wanting a red of its
   own in a file where this task's one red was spent. Ledger 496, with `import_a_mailbox`'s
   `refuse` closure as the pattern.

Premises 1, 2 and 5 of the plan held: the six sites at the lines quoted within a few tens, the
load-bearing order at both working surfaces, and the two changelog lines.

## Deviations from plan

**1. [Rule 3] `tests/wired.rs` edited in task 1.** Premise 1 above; a compile error, not a
choice. Ledger 498.

**2. [Decision] Three rustdoc blocks moved, not one.** Premise 3 above. Ledger 498.

**3. [Decision] The comment corrected rather than the code, on the status bar.** Premise 8.
Ledger 496 and 498.

**4. [Decision] A third changelog entry dated.** Premise 5. Ledger 498.

**5. [Decision] `html_renderer.rs` touched.** Not in the plan's file lists. `ThreadPart` gained
`before_the_body` and `render_thread` a sibling that takes a bar, because a per-part sentence
and a page-level bar both need a place in the HTML that the sanitiser does not reach, and a
sentence prepended to a Multipart body's plain half would never be seen. No test added there;
its six records hold, and the behaviour is held by `reader_text`'s readings.

**6. [Decision] `ConversationPart` carries the findings beside `body`, not instead of it.**
The plan allowed either; `body` is read everywhere and the findings by four surfaces through
the composer only. Sixteen constructors gained a field.

**7. [Process] No tracked file was edited by a script.** Every edit to a tracked file went
through Read then Edit or Write; `cargo fmt` ran before each commit; carriage returns measured
with `tr -cd '\r' | wc -c` on every changed file before each commit, zero on each; no em-dash in
any file this plan wrote, measured with `grep -c` for the byte sequence. The exception set for
scripted edits on a tracked file is zero, as it was for the twenty-eight executors before 09-06.
Commit messages were written to the scratchpad and passed with `-F`; the two whole-gate runs and
every remedy wrote to a file and the exit status was read directly. Nothing was sent to any
window; no binary was started; the tester's profile was not read. `Cargo.toml` untouched; no
package added.

Everything else executed as written.

## Threat register

T-09-22: the verdict on every surface is `checking_signatures::for_message`'s, asked through
one function, and the preview's bar is held by two readings. T-09-23: `the_body_to_show` and the
finding travel together as `WhatAMessageShowsAndSays`, the fold narrows the sentence beside the
words, and the stored preview body stays the armour so a reply cannot quote the words. T-09-24:
`with_what_is_said` fixes the order and two readings hold the envelope inside the spoken cut,
one through the fold and one through the page's composer. T-09-25: `body_of` panics on a missing
opener, which premise 1 met on the first build. T-09-SC: no package added. New surface outside
the register: the preview renders the top of the bar into HTML. Every line goes through
`html_escape::encode_text`, and the lines are this program's own wording and
`signed_mail`'s, never the sender's; a sender's words reach the page only through the body,
which goes through the sanitiser as before.

## Ledger

`.planning/WINDOWS.md` 495 to 498 written through `gsd-tools windows append`, both halves,
no backslash in any description (the nine in the file predate this plan);
`the_planning_files_agree_with_themselves` green after, 16 passed. 494 before, 498 after; 466
open before, 470 after.

| id | kind | what |
|---|---|---|
| 495 | unrun-verify | whether the preview's bar and a thread's per-message sentences read well by ear, and whether a real correspondent's key opens anything: FOUND-10's `[S]` line, the tester's |
| 496 | todo | `import_a_pgp_private_key` announces only; the visible status line its comment promised is owed, on `import_a_mailbox`'s pattern, with a red in `wired.rs` |
| 497 | todo | a PGP signature inside a conversation of several is still not mentioned there |
| 498 | deviation | the four departures above |

## Known stubs

None. `for_message` has one caller in the window, the seam, and the seam has six; `put_together`
is called by `for_message`; `with_what_is_said` by the text reader, Shift+Space and the page's
composer; `one_of_several` by both thread composers; `preview_html` by `the_preview_of`;
`render_thread_under_a_bar` by `preview_html` and by `render_thread`; `the_preview_of` by the
`MessageBodyLoaded` arm, which every body the preview shows goes through. `WhatIsSaidAboutIt::nothing`
is called by tests and by two guard breaks and by no surface, and its comment says so.
`service::pgp::for_tests` is `cfg(test)`.

## Not done here, on purpose

The listening: nobody has heard the preview's bar or a thread's per-message sentence with a
screen reader (ledger 495). The status line on key import (496). The signed sentence inside a
thread (497). The wider encrypted-mail gaps are #52 and the key manager is #49, neither touched.
No `FOUND` requirement is ticked, on the phase's rule that the last plan reads each clause by
clause; the closing read should count FOUND-10's three `[D]` lines as met by the readings named
in the coverage block and its `[S]` line as the tester's. Nothing pushed.

## Self-Check: PASSED

`src/application/reading_a_message.rs` exists; `grep -c 'fn signature_check_for\|fn envelope_check_for'`
answers 0 on `src/presentation/wx_app.rs` and 2 on the module; `grep -c what_a_message_shows_and_says
src/presentation/wx_app.rs` is 6, the definition and five calls, because the two conversation
readings share `conversation_parts` (this line said 7 when first written and the check
corrected it); `guards/guards.toml` holds 852
records by the TOML reader; `docs/changelog.md` holds three lines beginning "Corrected on
2026-09-16". Commits `b2703002`, `b3314356`, `175d5383`, `6ac55a7c`, `8bb86e39`, `8beb2e56`,
`34158a43` and `f990d023` are in `git log --oneline` on `main`.

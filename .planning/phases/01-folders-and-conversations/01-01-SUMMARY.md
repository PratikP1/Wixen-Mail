---
phase: 01-folders-and-conversations
plan: 01
subsystem: mail-protocol
tags: [imap, modified-utf-7, rfc-3501, async-imap, wxdragon, accessibility]

requires: []
provides:
  - "mailbox_name::encode: modified UTF-7 for mailbox names, the inverse of the decoder that was already here"
  - "ImapSession::create_mailbox: the first of FOLDER-01's three missing IMAP verbs, gated, timed out, error-mapped"
  - "MailController::create_mailbox: the facade the window calls"
  - "ID_NEW_FOLDER and make_a_new_folder: File then New then Folder, keyboard only"
  - "UIUpdate::CommandAnswered: the success twin of CommandRefused, announced at high priority"
  - "what_came_of_making: the pure choice between a gate refusal and a server refusal"
  - "The proven shape for a new server-writing verb: gate, encode, library call, error map, facade, command, worker thread"
affects: [01-04, 01-05, 01-06, 01-07, 01-09, 01-10]

actuals:
  tokens: 13105
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A mailbox name this program made up is encoded on the way out; a name the server spelled goes back verbatim"
    - "A server-writing command runs on a worker thread and answers through a UIUpdate, so the announcement happens where UiaRaiseNotificationEvent belongs"
    - "The choice of sentence after a failure is a pure function, so both refusals have a test rather than a window"

key-files:
  created: []
  modified:
    - src/service/protocols/imap/mailbox_name.rs
    - src/service/protocols/imap.rs
    - src/application/mail_controller.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/service/outward.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "A new folder is made where it is named, not under the cursor: the hierarchy delimiter is deliberately dropped after list_folders, and guessing it writes the folder elsewhere on a dot-separated server"
  - "The new folder is recorded in the cache and marked as one to keep up to date, because the tree is read from the cache and not from the server"
  - "The new folder is subscribed as well as created, because a server that keeps a subscription list leaves a new mailbox out of it and this tree hides an unsubscribed folder"
  - "A folder name that cannot be used is refused, not repaired, following the rule import_tree states"
  - "MAIL_TRANSPORTS' imap floor raised from 8 to 9: leaving it at 8 had already weakened a pre-existing guard"

patterns-established:
  - "Task 1 established the whole layer stack for a server-writing verb; 01-04's RENAME and DELETE add to it rather than inventing one each"
  - "a_server_answering is now pub(crate), so the controller's tests drive the same script as the session's"

# FOLDER-01 is NOT complete and is deliberately not listed here. This plan's own
# frontmatter names it, and its source-coverage table says it is covered by
# 01-01, 01-04, 01-07 and 01-09 together. FOLDER-01 asks for create, rename,
# delete, mark a folder read and empty a folder; this plan built create.
# `gsd-tools requirements mark-complete FOLDER-01` ticked it and the tick was
# reverted: a requirement recorded complete on a quarter of its scope is exactly
# the kind of claim this project's guardrails exist to stop.
requirements-completed: []
requirements-advanced: [FOLDER-01]

coverage:
  - id: D1
    description: "A mailbox name that is not ASCII reaches the server in modified UTF-7, and round-trips through the existing decoder"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "cargo test --lib mailbox_name:: (15 tests, 9 new)"
        status: pass
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_creating_a_folder_sends_the_name_in_the_encoding_the_server_reads"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a mailbox name that is not ASCII reaches the server encoded (measured, reddens exactly 2)"
        status: pass
    human_judgment: false
  - id: D2
    description: "CREATE reaches an IMAP server, and with mail writes off nothing is sent at all"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_creating_a_folder_with_the_gate_closed_says_nothing_to_the_server"
        status: pass
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_every_mailbox_write_says_nothing_to_the_server_with_the_gate_closed"
        status: pass
      - kind: other
        ref: "guards/guards.toml: creating a folder on a server asks the gate before it builds a command (measured, reddens exactly 4)"
        status: pass
    human_judgment: false
  - id: D3
    description: "A refusal reads as a refusal, and the two kinds are told apart before they reach the user"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_a_server_that_will_not_make_a_folder_is_never_reported_as_a_folder_made"
        status: pass
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_the_two_ways_of_being_refused_a_folder_can_be_told_apart"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#making_a_folder (4 tests on what_came_of_making)"
        status: pass
    human_judgment: false
  - id: D4
    description: "A user creates a folder from the keyboard alone and the tree shows it without re-navigating, announced on both channels"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "cargo test --test wired (ID_NEW_FOLDER is raised, handled, and claims no letter another item claims)"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#making_a_folder::the_path_of_a_new_folder (6 tests)"
        status: pass
    human_judgment: true
    rationale: >
      Every layer below the window has a test against a loopback server, and the command is
      proven raised and handled. Nothing here proves what a person hears when they press it:
      that needs a window, an account with stored credentials, a real IMAP server and NVDA.
      Nothing in this program has ever run against a real mail account, and this plan does not
      change that.

duration: 3h 5m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 01: Create a folder, end to end

**A folder created from File, New, Folder reaches an IMAP server with its name in modified UTF-7, is recorded and shown in the tree without re-navigating, and is refused with the right reason when mail writes are off, when the server says no, or when the name cannot be used.**

## Performance

- **Duration:** about 3 hours 5 minutes
- **Started:** 2026-08-30T04:25Z
- **Completed:** 2026-08-30T07:30Z
- **Tasks:** 3 of 3
- **Files modified:** 8

Most of that time is the gate rather than the work. `scripts/check.sh` runs on
every commit and takes about four minutes, and the two guard records needed four
full library runs to measure by hand, at roughly three minutes each.

## Accomplishments

- **The encoder that did not exist.** `mailbox_name.rs` decoded modified UTF-7
  and its module comment said an encoder belonged there the day something
  created a mailbox. It does now: 9 new tests including the round trip against
  the existing decoder, RFC 3501's own examples in both directions, a literal
  ampersand, a run outside the basic plane as a surrogate pair, and control
  characters never sent as themselves.
- **The whole layer stack for a server-writing verb, proven on one path.**
  Keypress, `is_local` deciding which side, facade, session verb with `may_i` as
  its first line, encoder, `async-imap`'s own `create` because that one reads the
  tagged response. 01-04 adds RENAME and DELETE to this rather than inventing a
  shape each.
- **A refusal reads as a refusal.** A server saying NO, the account's own setting
  saying no, a name that cannot be used, a POP account with no server folders,
  and an account that will not sign in each produce their own sentence, and none
  of them leaves a row in the tree.
- **Two guard records measured by hand, and a third repaired.** The measurement
  found that a pre-existing guard had lost one of its three reddening tests
  because of this change. See Deviations.

## Task Commits

1. **Task 1: Create a folder end to end** - `787f832` (feat)
2. **Task 2: A refusal reads as a refusal** - `6337e0f` (test)
3. **Task 3: Two guard records, measured by hand** - `4879922` (chore)

### On the RED and GREEN gate commits

`workflow.tdd_mode` is on, and every behaviour here was written test-first. The
RED was watched fail each time, for the right reason:

| What | RED, measured | GREEN |
|---|---|---|
| `mailbox_name::encode` | 14 errors, `cannot find function encode` | 15 tests pass |
| `ImapSession::create_mailbox` | `no method named create_mailbox found for struct ImapSession` | 83 pass |
| `MailController::create_mailbox` | `no method named create_mailbox found for struct MailController`, plus E0603 on a private harness | 70 pass |
| `the_path_of_a_new_folder` | `cannot find function the_path_of_a_new_folder` | 7 pass |
| `what_came_of_making` | `cannot find function what_came_of_making` (5 sites) | 11 pass |

The RED is not a separate commit, and cannot be. The pre-commit hook runs
`scripts/check.sh`, which runs the whole suite, so a commit holding a failing
test cannot be made without `--no-verify`, which CLAUDE.md forbids and the plan
forbids again. Each commit therefore holds the test and the code that answers
it. A GSD gate-sequence check looking for a `test(...)` commit before a
`feat(...)` one will find them in the wrong order here: `787f832` is the feat and
`6337e0f` is the test, and that ordering is the plan's task order, not a skipped
gate. The evidence that the tests were red first is the table above rather than
the commit graph.

## Files Created/Modified

- `src/service/protocols/imap/mailbox_name.rs` — gained `encode`, `encode_run`
  and `is_printable_ascii`, sharing the one Base64 engine with `decode`. The
  module comment no longer says only decoding lives here.
- `src/service/protocols/imap.rs` — gained `create_mailbox`; three new loopback
  tests; `create_mailbox` added to the existing exhaustive gate test, which is
  named "every mailbox write" and would otherwise have been lying;
  `a_server_answering` made `pub(crate)` so the controller's tests drive the same
  script.
- `src/application/mail_controller.rs` — gained the `create_mailbox` facade, its
  controller-level test, and a row in the not-connected refusal list.
- `src/presentation/wx_app.rs` — `ID_NEW_FOLDER`, its menu item under File then
  New, its handler `make_a_new_folder`, the worker `spawn_the_folder_write`, the
  two pure decisions `the_path_of_a_new_folder` and `what_came_of_making`, their
  10 tests, and the `CommandAnswered` handling.
- `src/presentation/ui_types.rs` — `UIUpdate::CommandAnswered`.
- `src/service/outward.rs` — `create_mailbox` declared on `MAIL_MEASURED_ON_THE_WIRE`;
  the imap floor in `MAIL_TRANSPORTS` raised from 8 to 9.
- `guards/guards.toml` — two records, the header count 309 to 311, 501 records
  to 503.
- `docs/changelog.md` — one entry under `[Unreleased]`, with its known limits.

## Decisions Made

- **A new folder is made where it is named, not under the cursor.** The plan said
  to build the full path from the tree cursor's parent. That cannot be done
  correctly today and the code says so at the seam: `list_folders` reads the
  server's hierarchy delimiter, uses it to find the leaf, and drops it, with a
  comment saying "It is not carried on the struct: the tree is one flat level, so
  nothing downstream has anything to do with it. It comes back when the tree gains
  a hierarchy." Guessing `/` makes a folder literally named `Work/2026` on any
  server that separates with a dot. Nesting is 01-03 and 01-05. A folder under
  one kept on this computer does get its parent's path, because those paths are
  ours and their separator is known; that is the branch `is_local` routes on.
- **The dialog says where the folder goes.** "The folder is made on the server, so
  every mail app using this account sees it." A note above the box rather than an
  announcement, following `Asking`'s own rule about when a note is warranted.
- **`CommandAnswered` rather than `StatusUpdated` for the outcome.**
  `StatusUpdated` is announced at low priority under a shared topic, so it
  coalesces with the traffic a sync produces and the answer to a key somebody just
  pressed can be the one that is dropped. The new variant is announced at high
  priority under its own topic, exactly as `CommandRefused` is.
- **The two refusals are told apart in a pure function.** `protocol_error`
  collapses a NO, a BAD and a dropped connection into one `Error::Protocol`, so
  the gate against everything else is the only distinction left. Pulling the
  branch out of the worker thread is what gives both sentences a test, and makes
  "neither refusal leaves a row" something a test can assert rather than something
  the code happens to do.

## Deviations from Plan

Five, all found by reading or running the code rather than by improvising around
it.

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] The tree is read from the cache, so creating on the server alone shows nothing**

- **Found during:** Task 1
- **Issue:** The plan said to call `read_the_tree_back` after creating. That
  function reads `folders_in_the_tree`, which reads `get_folders_for_account` out
  of the local cache. A folder created only on the server is not in the cache, so
  the user would be told a folder was made and see an unchanged tree. That is the
  plan's own must-have truth 1 unmet, and it is the shape CLAUDE.md says has
  shipped twice.
- **Fix:** After a successful CREATE the worker records the folder through
  `mail_sync::store_folders`, which is the one place that already knows how a
  folder becomes a cache row, then sends the tree updates.
- **Files modified:** `src/presentation/wx_app.rs`
- **Verification:** Structural, through reuse of `store_folders`. Not covered by a
  test; see Issues Encountered.
- **Committed in:** `787f832`

**2. [Rule 2 - Missing critical functionality] A newly created folder is unsubscribed, and this tree hides unsubscribed folders**

- **Found during:** Task 1
- **Issue:** `cached_folder_syncs` keeps a folder out of the tree when the account
  keeps a subscription list and the folder is not on it. RFC 3501's CREATE does
  not subscribe. So on a subscription-keeping server the folder would have been
  made, recorded, and still not shown.
- **Fix:** Two things. The folder is subscribed after being created, best effort,
  using the already-tested `set_subscribed`; and `set_folder_choice(..., true)` is
  written, because making a folder is choosing it and `cached_folder_syncs` obeys
  an explicit choice over every other rule. The second is what actually guarantees
  the row; the first is the courtesy that makes other mail apps agree.
- **Files modified:** `src/presentation/wx_app.rs`
- **Verification:** Read from `cached_folder_syncs`, which returns the choice
  before consulting subscription. Not covered by a test; see Issues Encountered.
- **Committed in:** `787f832`

**3. [Rule 1 - Bug] An existing refusal said mail folders cannot be made here, which this plan made false**

- **Found during:** Task 1
- **Issue:** `ID_CONTEXT_NEW_CONTAINER` answered "Mail folders are made on the
  server, not here" whenever the mail module was active, because mail has no
  container kind. That sentence was true until this plan and is a lie afterwards,
  told to somebody using the File, New, "Calendar, List, Folder or Group..." item
  whose own label offers Folder.
- **Fix:** In the mail module that command now makes a folder. The delete half of
  the same arm keeps a refusal, reworded to be about removing only, since removing
  a folder genuinely is not built.
- **Files modified:** `src/presentation/wx_app.rs`
- **Verification:** `cargo test --test wired` passes, so both ids stay raised and
  handled.
- **Committed in:** `787f832`

**4. [Rule 2 - Missing critical functionality] A new gated write must be declared on the mail wire list, and the census's floor must rise with it**

- **Found during:** Task 1, then again in Task 3
- **Issue:** Two parts. First,
  `test_every_mailbox_write_is_on_the_mail_wire_list` failed on the first full
  run: a write that changes something at a mail server and is on neither list is
  a write nobody has said whether anything reads. Second, and only visible from
  the guard run, the imap floor in `MAIL_TRANSPORTS` said 8 while nine gated
  writes were now present. That is not cosmetic. The record for `copy_message`
  losing its gate names three tests that should redden, and `scripts/guards.sh`
  reported one of them staying green: with a spare write above the floor,
  removing a gate no longer dropped the count below it. A pre-existing guard had
  quietly lost a third of its discrimination.
- **Fix:** `create_mailbox` declared on `MAIL_MEASURED_ON_THE_WIRE` with the
  command line a test really reads off the socket, and the floor raised to 9 with
  a comment saying what raises it. The `copy_message` guard is back to three of
  three.
- **Files modified:** `src/service/outward.rs`, `src/service/protocols/imap.rs`
- **Verification:** `scripts/guards.sh "asks the gate"` reports both guards
  reddening exactly the tests their records name.
- **Committed in:** `787f832` (the list row), `4879922` (the floor)

**5. [Rule 2 - Missing critical functionality] The exhaustive gate test is named "every mailbox write" and did not include the new one**

- **Found during:** Task 1
- **Issue:** `test_every_mailbox_write_says_nothing_to_the_server_with_the_gate_closed`
  holds a hand-written list of every write. A new write left out of it makes the
  test's name a promise it does not keep.
- **Fix:** `create_mailbox` added to the list, its refusal wording to the acts
  list, and `CREATE` to the commands asserted absent from the transcript.
- **Files modified:** `src/service/protocols/imap.rs`
- **Verification:** It is one of the four tests the gate guard record now names,
  measured.
- **Committed in:** `787f832`

---

**Total deviations:** 5 auto-fixed (1 under Rule 1, 4 under Rule 2). None needed
Rule 4: nothing here changed the architecture, and the one place the plan asked
for something the code cannot do (nesting) was narrowed rather than invented.

**Impact on plan:** No scope creep. Four of the five are the difference between
a feature that compiles and one that a person can see the result of, which is
CLAUDE.md's first guardrail. The fifth repaired an existing guard.

## Issues Encountered

**Two files outside the plan's `files_modified`.** `src/presentation/ui_types.rs`
for `UIUpdate::CommandAnswered`, and `src/service/outward.rs` for the wire-list
row and the floor. Neither was avoidable: the announcement has to happen on the
UI thread, and the census refuses a new gated write that is not declared.

**The first hand measurement was wrong by the time it was written down.** Break 2
reddened three tests when measured, and four after the floor was raised. The
floor change came from the verification run of break 1's neighbours. Both records
were re-measured against the final tree; the second record carries a comment
saying why the number changed. The lesson is to take the measurement after every
edit in the batch has landed, not before.

**What is not covered by a test, said plainly.** Deviations 1 and 2 are the two
things that make the new folder actually appear, and neither has a test. Reaching
them needs a window, an account with stored credentials and a mail server, which
is the same boundary `tests/wired.rs` describes for every handler in that file.
They are verified by reuse and by reading: `store_folders` is the tested path a
folder takes into the cache, and `cached_folder_syncs` returns an explicit choice
before it consults anything else. The residue is real and this is where it is
written down rather than left to be discovered.

**Nothing has run against a real mail account.** The server in these tests is
written for the tests. That is unchanged by this plan and no criterion here
claims otherwise.

## User Setup Required

None.

Worth knowing rather than doing: mail writes are off for a new install, so
File, New, Folder refuses with a sentence naming the setting until Allowed
Changes is turned on for the account. That is the designed behaviour and one of
the plan's must-have truths, not a setup step.

## Next Phase Readiness

Ready for 01-02 and for the rest of the phase.

- **01-04 (RENAME, DELETE)** has the shape it needs: gate, encode, library call
  that reads the tagged response, error map, facade, command, worker. Both of its
  verbs must call `mailbox_name::encode`, and the guard recorded here is the
  pattern for saying so. Adding two gated writes means raising the imap floor in
  `MAIL_TRANSPORTS` from 9 to 11 in the same commit, and declaring both on
  `MAIL_MEASURED_ON_THE_WIRE`.
- **01-03 and 01-05 (nesting and the tree)** own the open piece this plan
  deliberately did not take: the hierarchy delimiter has to be carried past
  `list_folders` before a folder can be made inside another. The changelog says
  so where a user meets it.
- **01-06** takes making a folder on this computer, which `make_a_new_folder`
  currently refuses with a sentence saying it is not built.
- `UIUpdate::CommandAnswered` is available to every later command in the phase
  that needs a high-priority spoken outcome.

---
*Phase: 01-folders-and-conversations*
*Completed: 2026-08-30*

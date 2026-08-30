---
phase: 01-folders-and-conversations
plan: 04
subsystem: mail-protocol
tags: [imap, rfc-9051, rename, delete, modified-utf-7, wxdragon, accessibility]

requires:
  - phase: 01-01
    provides: "The proven shape for a server-writing verb, mailbox_name::encode, and the rule that a name we made up is encoded while a name the server spelled goes back verbatim"
  - phase: 01-03
    provides: "folders.parent_id and folder_parents, the stored tree these three commands walk"
provides:
  - "ImapSession::rename_mailbox and ImapSession::delete_mailbox: FOLDER-01's last two missing verbs, gated, timed out, error-mapped"
  - "MailController::rename_mailbox and MailController::delete_mailbox: the facades the window calls"
  - "mailbox_name::the_path_after_a_rename, the_path_after_a_move, the_separator_between: path arithmetic on the wire form, with the separator read rather than stored or guessed"
  - "imap::what_separates_a_folder_from_one_inside: the destination's separator read off a LIST line, per mailbox"
  - "application::folders_underneath: Placed, deepest_first, where_the_rows_move_to, is_too_deep_to_follow, all depth-bounded"
  - "application::how_far_it_got: HowFarItGot and StoppedAt, the record a batch with no transaction leaves behind. 01-09 reuses it for emptying"
  - "destinations::Moving::Folder and where_a_folder_can_go: the offered list with the folder, its subtree and its current parent left out"
  - "MessageCache::set_folder_path and MessageCache::forget_folder: the rows catching up with the server"
  - "UIUpdate::ChosenFolderIsGone: a worker telling the window the row somebody was on has gone"
  - "ID_RENAME_FOLDER, ID_MOVE_FOLDER, ID_DELETE_FOLDER on Action, This Folder"
affects: [01-05, 01-06, 01-07, 01-09]

actuals:
  tokens: 37415
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A path the server spelled goes back verbatim; only the segment a person typed is encoded, and the two spellings are decided per parameter rather than per function"
    - "The hierarchy separator is read from the gap between a folder's path and its parent's, or off a LIST line, and never stored or assumed"
    - "A batch with no transaction under it stops at the first refusal and says exactly how far it got, rather than pretending to be atomic"
    - "One walk over the stored tree serves rename, move and delete, reversed where the order differs, because two walks are two chances to disagree about what is under what"

key-files:
  created:
    - src/application/folders_underneath.rs
    - src/application/how_far_it_got.rs
  modified:
    - src/service/protocols/imap.rs
    - src/service/protocols/imap/mailbox_name.rs
    - src/service/outward.rs
    - src/application/mail_controller.rs
    - src/application/destinations.rs
    - src/application/mod.rs
    - src/data/message_cache/folders.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_destination.rs
    - src/presentation/ui_types.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/ALPHA_TESTING.md
    - docs/IMPLEMENTATION_STATUS.md

key-decisions:
  - "rename_mailbox and delete_mailbox take the server's own spelling verbatim and encode nothing. The plan said both RENAME arguments go through the encoder; a path from the cache is already encoded and a second pass names a mailbox the server has not got"
  - "Renaming or deleting a folder kept on this computer is refused with a sentence, not built. Local folders are a const array of &'static str the code owns, so there is nothing a person named and nothing to rename"
  - "The separator for a rename comes from the gap between the folder's path and its parent's, which is exact and free; the separator for a move comes from a LIST on the worker, after the question has been answered, because a childless destination has no stored evidence"
  - "folders_deepest_first went into a new folders_underneath module rather than how_far_it_got, because rename and move needed the same walk two tasks before the delete did"
  - "Moving to the top level is offered as a destination of its own, named by the empty path, because without it a folder goes in and never comes out"
  - "No version bump: 0.46.0 has not been released or tagged, so it is the accumulating unreleased version and this work belongs in it"

patterns-established:
  - "A behaviour RED is taken by stubbing the body and reading which assertions fail, not by letting the compiler report a missing symbol. Three tests here could not go red against an empty stub, and that is worth knowing"
  - "A guard record whose break reddens tests in an unrelated-looking module is the record worth keeping: the deepest-first record reddens two rename tests because the rename reverses the same walk"

requirements-completed: []
requirements-advanced: [FOLDER-01]

coverage:
  - id: D1
    description: "Renaming a folder changes its name and nothing else, and the path in front of it is carried over exactly"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "cargo test --lib mailbox_name:: (27 tests, 12 new: the leaf offset, the separator read from the gap, a rename encoding only the typed name, a move keeping the name exactly)"
        status: pass
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_renaming_a_folder_sends_both_paths_as_the_server_spells_them (asserts the re-encoded form is absent, not merely that the right line is present)"
        status: pass
      - kind: integration
        ref: "src/application/mail_controller.rs#test_renaming_a_folder_reaches_the_server_spelled_as_it_was_given"
        status: pass
    human_judgment: false
  - id: D2
    description: "Renaming or moving INBOX is refused with a reason before any command is built"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "src/presentation/wx_app.rs#making_a_folder::test_the_inbox_is_refused_a_rename_and_the_reason_says_what_would_happen, plus the move and delete cases"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#making_a_folder::test_the_local_inbox_is_refused_as_a_local_folder_rather_than_as_an_inbox"
        status: pass
    human_judgment: false
  - id: D3
    description: "A folder moves to a new parent in one command and cannot be moved inside itself"
    requirement: FOLDER-01
    verification:
      - kind: unit
        ref: "cargo test --lib destinations:: (28 tests, 11 new: the folder itself, its subtree, its current parent, the top level, depth, a stored cycle)"
        status: pass
      - kind: unit
        ref: "src/service/protocols/imap.rs#test_the_separator_for_a_folder_is_the_one_the_server_gave_for_that_folder (3 tests)"
        status: pass
      - kind: unit
        ref: "src/application/folders_underneath.rs#test_renaming_a_folder_moves_every_row_underneath_it_too (13 tests in the module)"
        status: pass
    human_judgment: false
  - id: D4
    description: "A folder with children deletes deepest first, and a partial failure says exactly where it stopped"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_a_subtree_is_deleted_deepest_first_one_command_at_a_time (when_told positions, not was_told)"
        status: pass
      - kind: unit
        ref: "cargo test --lib how_far_it_got:: (9 tests, including D-36's sentence verbatim)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a folder with folders inside it is deleted deepest first (measured, reddens exactly 5)"
        status: pass
    human_judgment: false
  - id: D5
    description: "All three verbs reach the server only through Allowed::mail, and none is offered as a drag"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "src/service/protocols/imap.rs#test_every_mailbox_write_says_nothing_to_the_server_with_the_gate_closed (rename and delete added; CREATE, RENAME and DELETE asserted absent from the transcript)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: deleting a folder on a server asks the gate before it builds a command (measured, reddens exactly 3)"
        status: pass
      - kind: integration
        ref: "cargo test --test wired (all three ids raised, handled, and claiming no letter another item claims; it caught a real Alt-key collision in the rename dialog)"
        status: pass
    human_judgment: false
  - id: D6
    description: "A person carries out all three from the keyboard and hears what happened"
    requirement: FOLDER-01
    verification:
      - kind: integration
        ref: "cargo test --test wired (57 tests)"
        status: pass
    human_judgment: true
    rationale: >
      Every layer below the window is tested against a loopback server and the three commands
      are proven raised and handled. Nothing here proves what a person hears when they press
      them: that needs a window, an account with stored credentials, a real IMAP server and
      NVDA. Nothing in this program has ever run against a real mail account and this plan does
      not change that.

duration: 4h 10m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 04: Rename, move and delete a folder

**A folder on the server can be renamed by its name alone, moved under a different folder in one command that carries everything inside it, or deleted from the inside out with an exact account of how far it got if it stops, and the inbox is refused all three before anything is sent.**

## Performance

- **Duration:** about 4 hours 10 minutes
- **Tasks:** 3 of 3
- **Files:** 16, of which 2 are new

## Accomplishments

- **FOLDER-01's last two verbs.** `rename_mailbox` and `delete_mailbox` join
  `create_mailbox` on the shape 01-01 proved: `may_i` as the first line,
  `with_timeout`, the library's own method because it reads the tagged response,
  `protocol_error`. Both declared on `MAIL_MEASURED_ON_THE_WIRE`, and the imap
  floor in `MAIL_TRANSPORTS` raised 9 to 11 in the commits that added them.

- **The two halves of one verb, split at the user's end.** RENAME is rename and
  move on the wire. Rename asks for a name and carries every path segment in
  front of it over exactly. Move names the new place from a list, asks first,
  and changes only what is in front of the name. That is D-26's safety argument,
  and it cost almost nothing to honour once the path arithmetic was pure.

- **The separator is never guessed.** Nothing stores it, so a rename reads it
  from the gap between a folder's path and its parent's, which is exact and
  free, and a move reads it off a `LIST` line for the destination, which is
  exact and costs one round trip on the worker after the question has been
  answered. Neither assumes a slash. That is the defect `make_a_new_folder`
  avoided in 01-01 by refusing to nest at all.

- **A batch that says where it stopped.** `HowFarItGot` produces D-36's sentence
  verbatim, and its second test caught a real bug: the verb was singularised and
  the noun was not, so a synthesiser would have read out "1 folders was not
  deleted".

- **Two guard records measured by hand**, and both verified with
  `scripts/guards.sh`. The three gate guards, including 01-01's, still redden
  exactly the tests they name after the floor rose.

## Task Commits

1. **Task 1: Rename the leaf, and refuse INBOX** - `6e4646d`
2. **Task 2: Move To, one RENAME, with a destination list** - `36700ec`
3. **Task 3: Delete deepest first, and say how far it got** - `3f0e720`

### On the RED and GREEN gates

`workflow.tdd_mode` is on and every behaviour here was written test-first. The
pre-commit hook forbids committing a failing test, so RED cannot live in its own
commit and the evidence is this table rather than the commit graph. Each RED was
watched fail, for the right reason:

| What | RED, measured | GREEN |
|---|---|---|
| `mailbox_name` path arithmetic | 13 errors, `cannot find function` for all three | 23 tests pass |
| `ImapSession::rename_mailbox` | 4 errors, `no method named rename_mailbox` | 151 pass |
| `MailController::rename_mailbox` | 2 errors, `no method named rename_mailbox` | 71 pass |
| `folders_underneath` walk | body stubbed to return nothing: **5 failed, 3 passed** | 8 pass |
| `MessageCache::set_folder_path` | 2 errors, `no method named set_folder_path` | 20 pass |
| `why_the_folder_cannot_be` | decision stubbed off: **5 failed** | 17 pass |
| `FolderAct` wording | one arm changed to another's words: **1 failed** | 17 pass |
| `where_a_folder_can_go` | 5 errors, `cannot find function`; `Moving::Folder` missing | 28 pass |
| `what_separates_a_folder_from_one_inside` | 4 errors, `cannot find function` | 158 pass |
| `HowFarItGot::said` | body stubbed to return nothing: **8 failed, 1 passed** | 9 pass |
| `ImapSession::delete_mailbox` | 3 errors, `no method named delete_mailbox` | 162 pass |
| `MessageCache::forget_folder` | 3 errors, `no method named forget_folder` | 22 pass |

**Two of those measurements are worth more than the rest, and the reason is a
finding.** Where a test module and its implementation were written before the
first compile, the red that came back was `cannot find function`. That proves
the name was absent and says nothing about whether any assertion would notice a
wrong answer. Stubbing the body instead and reading which assertions fail is the
stronger measurement, and taking it showed that **three of the eight
`folders_underneath` tests and one of the nine `HowFarItGot` tests stayed green
against a body that returned nothing**, because they are negative assertions
("this is absent", "this does not contain a zero") and an empty answer satisfies
them. Those four have never been red on their own and are carried by their
neighbours. Written down here rather than left to be discovered.

## Deviations from Plan

Four wrong premises in the plan and two auto-fixes. Every one was found by
reading or running the code.

### Wrong premises in the plan

**1. "Both arguments go through `mailbox_name::encode`" would have double-encoded the source path**

- **Found during:** Task 1, before any code was written
- **Issue:** The plan's action says both `rename_mailbox` arguments go through
  the encoder, and its acceptance criterion demands `encode` be called twice.
  `ImapFolder::path` and `CachedFolder::path` hold the server's own spelling, in
  modified UTF-7 as it came off the LIST response. Encoding that again spells a
  mailbox the server has not got, and the rename would fail against a name
  nobody typed. `ImapFolder::path`'s own doc comment says so, and 01-01's
  summary states the rule outright: "a mailbox name this program made up is
  encoded on the way out; a name the server spelled goes back verbatim". Only
  the new leaf is made up.
- **What was built instead:** `rename_mailbox` and `delete_mailbox` take the
  server's spelling verbatim, exactly as `set_subscribed` and `select_folder`
  already do. The encoding moved to `mailbox_name::the_path_after_a_rename`,
  which encodes the typed segment and carries the rest over untouched. The
  plan's real requirement, that a non-ASCII new name reaches the server encoded,
  is met and has its own test.
- **Committed in:** `6e4646d`

**2. There is no such thing as renaming a folder kept on this computer**

- **Found during:** Task 1
- **Issue:** The plan's acceptance criterion says "A local folder rename runs
  with `Allowed::mail` off and succeeds, proving it is not gated."
  `local_folders::FOR_POP` and `FOR_IMAP` are `const` arrays of
  `LocalFolder { kind, name: &'static str }`, rebuilt whenever the accounts are
  read. There is nothing there a person named, so there is nothing to rename,
  and a row renamed in the database would come back under its old name at the
  next start. Renaming "Trash" would also break `local_trash()`, which finds it
  by kind. 01-06 is the plan that builds user-named local folders.
- **What was built instead:** `is_local` is still asked first, which is
  FOLDER-01's rule about one place deciding which side a folder is on, and the
  local branch refuses with a sentence saying it is not built yet. That is
  exactly what `make_a_new_folder` already does for the same gap. Three
  sentences, one per command, and a test that a POP account's inbox hears about
  this computer rather than about a mail server it does not have.
- **Committed in:** `6e4646d`, `36700ec`, `3f0e720`

**3. The hierarchy separator 01-03 read is not stored anywhere, so no command can see it**

- **Found during:** Task 1, and it shaped both later tasks
- **Issue:** 01-03 added `ImapFolder::delimiter` and its summary lists it under
  "provides". It is used during `store_folders` to derive `parent_id` and is
  never persisted: the `folders` table has no column for it. Every command here
  runs from the cache, so the separator is invisible to all three. The plan
  assumed it was available.
- **What was built instead:** two answers, each exact.
  For a rename, the folder above is a prefix of this one, so whatever sits
  between the two is the server's own separator for that mailbox, read by
  `mailbox_name::the_separator_between`. No round trip and nothing stored.
  For a move, a childless destination offers no such evidence, so the separator
  is read off a `LIST` line by
  `imap::what_separates_a_folder_from_one_inside`. A LIST line carries a
  separator for every mailbox whether or not anything is nested under it, which
  is what makes this answerable at all. That LIST runs on the worker after the
  confirmation, so D-37's rule against a round trip in front of a dialog
  somebody may cancel still holds. A server that names no separator gets a
  sentence and no command.
- **Committed in:** `6e4646d`, `36700ec`

**4. `folders_deepest_first(parents, target) -> Vec<i64>` cannot be computed from `folder_parents` alone, and belongs two tasks earlier**

- **Found during:** Task 1
- **Issue:** Two things. `folder_parents` returns `HashMap<path, Option<id>>`,
  which has no path-to-id direction, so a function taking only that map cannot
  return ids. And the walk is needed by the rename in Task 1, to move the rows
  of every folder inside a renamed one, long before the delete in Task 3 needs
  it.
- **What was built instead:** `application::folders_underneath`, holding
  `Placed { id, path, name, parent }` built by joining `get_folders_for_account`
  with `folder_parents` once, and `deepest_first`, `where_the_rows_move_to` and
  `is_too_deep_to_follow` over it, all depth-bounded. `how_far_it_got` holds the
  reporting type only, which is the concern 01-09 reuses and which has nothing
  to do with folder trees. The guard record measured for the walk order shows
  why keeping it in one place matters: reversing it reddens two rename tests as
  well as three delete ones, because the rename reverses the same walk.
- **Committed in:** `6e4646d`, `3f0e720`

### Auto-fixed issues

**5. [Rule 2 - Missing critical functionality] The tree is read from the cache, so a rename or delete on the server alone changes nothing a person sees**

- **Found during:** Tasks 1, 2 and 3
- **Issue:** The same shape 01-01 found for creating. A folder renamed only on
  the server keeps its old row, with all of its mail under a name that no longer
  exists; a folder deleted only on the server keeps a row that opens nothing.
  The plan said to call `read_the_tree_back`, which reads the cache.
- **Fix:** `MessageCache::set_folder_path` moves a row rather than writing a
  second one, so the messages and the folders inside it keep pointing at the
  same id, and `where_the_rows_move_to` moves every row inside a renamed folder
  to match what the server did in the same command.
  `MessageCache::forget_folder` takes a folder off this computer along with its
  mail, and its test asserts the cascade really runs rather than trusting the
  schema to have been read correctly: a folder row removed while its messages
  stayed would leave the words of somebody's mail on the disk under a folder
  nothing lists.
- **Committed in:** `6e4646d`, `36700ec`, `3f0e720`

**6. [Rule 1 - Bug] Two documents said a folder cannot be created, which stopped being true in 01-01**

- **Found during:** Task 1
- **Issue:** `docs/ALPHA_TESTING.md` and `docs/IMPLEMENTATION_STATUS.md` both
  said "A folder cannot be created, renamed or deleted". The first third was
  already false before this plan started. Adding a capability makes every
  sentence asserting its absence false and nothing prompts a search for them.
- **Fix:** both corrected, and re-corrected at the end of each task so they say
  what is true now: created, renamed, moved and deleted; marking a whole folder
  read and emptying one are not built.
- **Committed in:** `6e4646d`, `36700ec`, `3f0e720`

### Two things existing checks caught that no new test would have

- **`tests/wired.rs` found a real Alt-key collision**: "Folder &name:" and
  "Re&name" both took Alt+N in the rename dialog. Changed to "&Rename".
- **`tests/house_style.rs` found that only one place may word what a delete
  did**, and my `got.said("Deleted", "folders")` at the window was a second.
  The wording moved onto `HowFarItGot::what_deleting_folders_did`, which is the
  better design and is what the guard was asking for.

**Total deviations:** 4 wrong premises corrected, 2 auto-fixed (1 under Rule 1,
1 under Rule 2). None needed Rule 4: nothing here changed the architecture.

## Issues Encountered

**Three files outside the plan's `files_modified`.**
`src/service/protocols/imap/mailbox_name.rs` for the path arithmetic,
`src/data/message_cache/folders.rs` for the two row writes, and
`src/presentation/ui_types.rs` for `UIUpdate::ChosenFolderIsGone`. None was
avoidable: the wire spelling belongs where the encoder is, the rows have to
catch up with the server, and clearing what is chosen has to happen on the
interface thread.

**Editing source through a shell script destroyed string continuations twice.**
Multi-line string literals joined with a trailing backslash came back as one
line with runs of spaces inside the sentence. The code compiled and every
functional test passed;
`test_no_sentence_is_written_with_the_source_indentation_inside_it` is what
caught it, both times. Fixed by hand with the editing tool. Worth recording
because a rewrite that keeps a string a valid string is invisible to the
compiler and to every test except one that reads the sentence as prose.

**What is not covered by a test, said plainly.** The three worker threads are
not reached by any test, for the reason `tests/wired.rs` states for every
handler in that file: reaching them needs a window, an account with stored
credentials and a mail server. Every decision they make was pulled out into a
pure function with tests, and every command they send has a loopback test one
layer down, but the wiring between the two is verified by reading. This is the
same residue 01-01 recorded and it has not changed.

**Nothing has run against a real mail account.** The server in these tests is
written for the tests. No criterion here claims otherwise.

## Known Stubs

Three commands are gated with a sentence rather than half-built, and each says
so where a person meets it and in `docs/changelog.md`:

| What | Where | Why, and what resolves it |
|---|---|---|
| Renaming a folder kept on this computer | `RENAMING_A_FOLDER_ON_THIS_COMPUTER_IS_NOT_BUILT_YET` | Local folders are a const array the code owns. 01-06 builds user-named local folders |
| Moving a folder kept on this computer | `MOVING_A_FOLDER_ON_THIS_COMPUTER_IS_NOT_BUILT_YET` | Same, and each has a fixed place. 01-06 |
| Deleting a folder kept on this computer | `DELETING_A_FOLDER_ON_THIS_COMPUTER_IS_NOT_BUILT_YET` | Same, and for a POP account those folders hold the only copy of the mail. 01-06 |

None of these prevents this plan's goal: FOLDER-01's three verbs are about
folders on a server, and `local_folders::is_local` is what decides which side a
folder is on, exactly as the phase review required.

## Version

No bump. 0.46.0 has not been released or tagged, so it is the accumulating
unreleased version and this work belongs in it, which is the same reading 01-03
recorded. CLAUDE.md's rule is that a behaviour change bumps in the same commit;
the version already moved for behaviour that has not shipped, and moving it
again would claim a second unreleased version for one unreleased milestone.

## Next Phase Readiness

- **01-05 (nesting the tree)** gets `folders_underneath::Placed` and the parent
  walk, which is the join between `get_folders_for_account` and `folder_parents`
  that it would otherwise write itself. Note that the interim regression 01-03
  recorded is still live and still closes there: a folder the server calls
  `Archive/2026` reads in the tree as `2026`.
- **01-06 (folders on this computer)** takes the three refusals above. Each
  names a constant, so the work is visible from a grep.
- **01-09 (emptying)** takes `HowFarItGot` unchanged. Its `said` is already
  parameterised by the verb and the noun and its first test is D-36's emptying
  sentence, not the delete one.
- **A warning for anything that adds a gated mail write:** the imap floor in
  `MAIL_TRANSPORTS` is 11 now and rises in the same commit as the write. 01-01
  recorded what a stale floor costs and the delete guard's third reddening test
  is that floor working.

---
*Phase: 01-folders-and-conversations*
*Completed: 2026-08-30*

## Self-Check: PASSED

Every file, commit hash and symbol this summary names was checked against disk
and `git log` after it was written. All present: both new modules, the three
commits, all fifteen new functions and types, the three command ids, the three
refusal constants, and 510 guard records against the header's 192 plus 318.

`bash scripts/check.sh all` is green in four parts, run by hand because the
branch hook runs only the quick pair here: rustfmt, clippy with `-D warnings`,
5,397 library tests plus every target under `tests/`, and the release build.
`--no-verify` was never used. `scripts/guards.sh` was run against both new
records and against all three "asks the gate" records, and every one reddens
exactly the tests it names.

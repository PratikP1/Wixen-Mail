# Phase 13.1: Phase 13's follow-ups

Five plans, one per wave, written on 2026-10-03 against `main` at `f5529cb1`,
version `1.0.0-alpha.1`, inserted between phase 13 and phase 14 the way GSD
inserts a decimal phase. On the day, `guards/guards.toml` held 1,427 records by
the TOML reader (`python -c "import tomllib;print(len(tomllib.load(open('guards/guards.toml','rb'))['guard']))"`),
`.planning/WINDOWS.md` 802 entries with 695 open and 107 fixed before this
planning reopened ledger 614 (696 and 106 after), the last whole gate was phase
13's, 10,549 tests at `19d3b0ae`, and `git rev-list origin/main..main --count`
answered 516, with `git log origin/main -1` at `630e2a67`, phase 12's merge.

The directory is named the way `gsd-tools generate-slug "Phase 13's follow-ups"`
answers, `phase-13-s-follow-ups`.

**Goal.** Ten things phase 13 left behind are done: every pull request's checks
go green on GitHub's runner again and `main` is pushed; the mail database
overwrites what a write frees and the privacy page says what that still leaves;
Empty Folder, the `.msg` reader and the folder import say what is true; the
gate runs one more whole-tree reading on every code commit and knows where its
script suites' time went; ledger 614 says Pratik's real answer; and the phase
closes on its own full gate.

**Requirements.** No requirement is added. Every item is a fix under one that
is already ticked, and each plan names it: FOUND-23 (the gate, the runner and
the mutation check: 13.1-01, 13.1-02), GAP-14 (taking mail off and Empty
Folder: 13.1-03), GAP-13 (import: 13.1-04), and GAP-01 for ledger 614's
correction, which the planning commit makes and 13.1-05 finishes on the alpha
page. A plan adds a dated line under its requirement and touches no box. The
coverage stays at 123.

## Pratik's answer

The orchestrator put the ten items below to Pratik on 2026-10-03 and he
answered, in these words: **"yes to all."** Each is a decision of his, numbered
for the plans to cite. Where an item leaves a choice open ("decide by
reading", "choose by reading"), the plan that carries it says so and the
executor's reading decides; where the planner had to fill a gap, it is under
"Decisions the planner took" below, for him to overrule.

| ID | His item, as put to him | Plan |
|---|---|---|
| D-01 | Ledger 785: on GitHub's runner the marker and signature key-test targets' children, run on a desktop of their own (13-44.6.3), cannot open the browser control and time out after five minutes; locally they pass; the other two browser targets pass on the runner. Every pull request's Test Suite and mutation check are red for it. Find why and fix it; if it cannot be fixed, run those two targets on the runner's normal desktop when CI is set (nobody types on a runner), saying so. This plan runs first, pushes its branch with a pull request, and after its merge with CI green on the pull request, `main` is pushed (his word, 2026-10-03: push main after the CI fix). `main` is 516 commits ahead of origin (ledger 802). | 13.1-01 |
| D-02 | Ledger 788: turn on SQLite's `secure_delete` for every connection (the measurement found `fast` and `on` cut leftover words about ninefold at no measured cost; choose by reading which is safer for the `-wal` file and say why), skip the compact command, and reword the privacy page heading "Mail taken off this computer keeps nothing it said" and its sentence so they say what is true (about 1 in 1,000 at 200,000 messages may leave some words in unused space). Re-measure with 13-44.9's own measuring code after the change and put the row on `docs/development/measurements.md`. | 13.1-03 |
| D-03 | Ledger 614: reopen it in both halves. The brief to 13-51 gave his answer (d) as "nothing is ever posted"; his real answer of 2026-09-24 was "Before you file an issue with wxDragon, give me more details", and a test program waits for him to run it (`C:/Users/prati/AppData/Local/Temp/wxpc/debug/wxdragon-print-check.exe`, built 2026-09-24). Done in the planning commit. | planning commit; 13.1-05 for the alpha page |
| D-04 | Ledger 794: an encrypted Outlook `.msg` imports with no words and its encrypted part attached, said nowhere. Count it beside the lost-signature count and say in the closing sentence how many arrived encrypted; keep the encrypted part. | 13.1-04 |
| D-05 | Ledger 797: a folder holding old Word or Excel files gets an empty folder under Imported (same leading bytes as `.msg`). Make each imported folder only when its first message is filed. | 13.1-04 |
| D-06 | Ledger 798: the folder import counts unreadable, unsaved and already-present messages and never says them; carry them onto the folder import's sentence in the single-file import's words. | 13.1-04 |
| D-07 | Ledger 786: Empty Folder on the shared POP Trash says "and there is no other copy"; change it to "and Wixen Mail keeps no other copy", leaving the rest. Spoken, so a pull request with NVDA. | 13.1-03 |
| D-08 | `tests/mail_taken_off_leaves_no_words_behind.rs` reads every file under `src` but the gate runs it only when `wx_app.rs` changes; add it to `scripts/check.sh`'s list of targets every code commit runs (it runs in under a second), test-first through the shell suite's own cases. | 13.1-02 |
| D-09 | The mutation check's baseline fails for a second reason besides ledger 458: `test_every_forgetting_row_has_the_pages_shape_and_names_what_it_timed` fails with "the row carries no commit" in a copy with no `.git`. Give it (and ledger 458's `test_the_share_of_history_before_red_green_is_computed_and_printed`, if the same fix fits) a fallback that says plainly it could not read a commit rather than failing, without weakening it where `.git` exists; decide by reading. | 13.1-01 |
| D-10 | The script suites took 356 s at 13-51's gate against 103 s at phase 12's; time each suite on its own and find which grew and why; fix only if the cause is clear and cheap, otherwise record the figures and the cause on `measurements.md` and ledger it. | 13.1-02 |

Then a closing plan in the highest wave runs the phase's `scripts/check.sh all`
by hand, per `CLAUDE.md`, and the closing read: 13.1-05.

## The plans

"Spoken or shown" is whether the plan's branch is pushed with a pull request
under Pratik's standing OK of 2026-09-23. 13.1-01 changes nothing spoken or
shown and pushes anyway, on D-01.

| Plan | Wave | What it does | Requirement | Ledger | Spoken or shown |
|---|---|---|---|---|---|
| 13.1-01 | 1 | The marker and signature children on the runner, found and fixed or run on the runner's own desktop when `CI` is set; a commit read where there is no history; the pull request green, the merge, and `main` pushed with its three workflows read | FOUND-23 | 785, 458; the push 802 waits for | pushed on D-01 |
| 13.1-02 | 2 | The whole-tree reading of mail taken off on every code commit, red first in `check.test.sh`; the four script suites timed one by one, what grew and why, fixed if clear and cheap or ledgered | FOUND-23 | 802, read from the push; a new entry if the suites are not fixed | no |
| 13.1-03 | 3 | `secure_delete` on for every connection, `on` rather than `fast` by reading; the measurement re-taken and its rows on the page; the privacy heading and sentence say what is true; Empty Folder's "and Wixen Mail keeps no other copy" | GAP-14 | 788, 786 | yes |
| 13.1-04 | 4 | An encrypted saved Outlook message counted and said; a folder made only when its first message is filed; the folder import's three counts said in the single-file import's words | GAP-13 | 794, 797, 798 | yes |
| 13.1-05 | 5 | The alpha page's answer (d) corrected by dating, the listening lines, the closing read of D-01 to D-10, the four marks, and the phase's full gate | FOUND-23, GAP-01, GAP-13, GAP-14 | what the plans opened | yes |

## One plan per wave, and why this order

One per wave because every plan writes `guards/guards.toml` and
`.planning/WINDOWS.md`, three write `docs/changelog.md`, and
`docs/development/measurements.md` is written by 13.1-02, 13.1-03 and 13.1-05;
a wave is a set of plans that share no file, and these share those.

- **13.1-01 first**, on D-01: every later pull request's Test Suite is red
  until it lands, and `main` is pushed only after it. D-09 rides in the same
  plan (decision D-11 below) so the pull request that ends in the push of
  `main` can show both of its checks green.
- **13.1-02 second.** D-08 puts the whole-tree reading on every code commit,
  so 13.1-03, which edits the store beside the compaction that reading guards,
  and 13.1-04 meet it on their own commits rather than at the phase's gate
  (ledger 791 is that reading going red on `main` for a merge that never ran
  it). D-10's timings are taken before anything else in the phase moves the
  suites.
- **13.1-03 and 13.1-04** are the two plans that change what is spoken and
  shown; each pushes its branch with a pull request.
- **13.1-05 last**, at the highest wave, reads every plan's work and runs the
  phase's full gate.

## Decisions the planner took

Taken 2026-10-03 by the planner where an item left a gap; Pratik may overrule
any of them.

- **D-11. D-09 rides in 13.1-01.** Ledger 785 says the pull request's
  mutation check fails its baseline on the marker target; D-09 says it also
  fails on the forgetting rows' shape case in a copy with no `.git`. A first
  plan that fixed only 785 would end in a pull request whose mutation check is
  still red for D-09's reason. With both in one plan, the pull request before
  the push of `main` can be read with every check green, or with the reason one
  is not said.
- **D-12. The fallback for a missing history is decided by whether `git`
  answers at all,** not by whether it answers what the test wanted: a copy
  with no repository says so in a sentence and still checks everything that
  needs no history; a repository that does not hold `18a02454` still fails, as
  the shallow checkout of 2026-09-15 did (`ci.yml`'s comment at :34 to :41).
  The executor's reading confirms or replaces this.
- **D-13. `on`, not `fast`,** subject to the executor's reading and the
  re-measure. SQLite's documentation of `PRAGMA secure_delete`
  (sqlite.org/pragma.html, read 2026-10-03): "When secure_delete is set to
  'fast', SQLite will overwrite deleted content with zeros only if doing so
  does not increase the amount of I/O ... This has the effect of purging all
  old content from b-tree pages, but leaving forensic traces on freelist
  pages." Under WAL, a page that a write frees under `fast` keeps its bytes in
  the file and in the write log until the page is reused; under `on` the
  zeroed page is written through the write log and into the file at the next
  checkpoint. 13-44.9's runs could not tell the two apart in cost or in what
  was left (1 of 1,000 each at 200,000), so the safer one is taken.
- **D-14. The removal's own switch stays.** `OverwritingWhatIsFreed`
  (`taken_off_this_computer.rs:73-97`) turns secure delete on for a removal and
  puts back what it found. With every connection on it changes nothing on the
  program's path, and it is kept so a connection set otherwise (13-44.9's
  measurement sets each setting by hand) still overwrites what a removal
  frees, which keeps the re-measure comparable with 13-44.9's rows. Its test
  `test_overwriting_freed_space_is_put_back_as_it_was_after_a_removal` would
  pass over nothing once the setting is already on, so it starts from a
  connection set off and is held to finding it off again.
- **D-15. The rows the re-measure puts on the page** are the fifteen secure
  delete rows at each size (`on`, `fast` and `off`, five each), dated, beside
  13-44.9's, which stay as the series; the VACUUM and incremental vacuum rows
  are printed and not added, because no command is built (D-02). The page's
  rule is a row added, never one edited.
- **D-16. The privacy heading** becomes one that names what taking mail off
  does and what it can leave, and the sentence carries the re-measured figure
  with its date and size. The heading's anchor is linked from
  `docs/USER_GUIDE.md:216`, which changes in the same commit. The exact words
  are the executor's, under `writing-craft` and the tree's word guards.
- **D-17. "Encrypted" is the class `IPM.Note.SMIME` exactly,** compared
  without regard to capitals, and never the signed class
  `IPM.Note.SMIME.MultipartSigned`, which the signature count already takes.
  A saved message signed but not encrypted in the opaque form may carry the
  same class; no real file of either kind has been read here (ledger 794 says
  so), and the ledger entry that closes says it again.
- **D-18. A folder counts as imported when a message is filed into it,**
  whether it was made by this import or already there. Its already-present
  identifiers are read with `cache.get_folder` first, which makes nothing, so a
  folder that receives no message is never made. The archive half of
  `fill_folders_from` moves out of `wx_app.rs` into the application layer so
  the decision can be tested with a real store and a folder on disk, and the
  window keeps the call.
- **D-19. One function says the three counts for both imports.** The words
  are the single-file import's (`importing_messages.rs:759-793`); where they
  name one place ("this folder", "the file"), the function takes the place
  word, so the single-file import's sentences are unchanged and the folder
  import names its folders and its archive.
- **D-20. D-08's red is a `check.test.sh` case only.** Five earlier
  whole-tree targets each carry a Rust case reading the array
  (`test_this_target_runs_on_the_commits_that_could_break_it`); D-08 names the
  shell suite, and a sixth Rust case would flag the target's guard records for
  a fact the shell case already holds.
- **D-21. "Clear and cheap" for D-10** means a cause named by a measurement,
  and a fix confined to the suites or `scripts/check.sh` that removes no case
  and leaves every case run in `all` mode. Anything else is a row and a ledger
  entry.
- **D-22. `main` is pushed once in this phase, by 13.1-01.** Later pushes are
  Pratik's word as before; 13.1-05 says how far `main` is ahead at the close.
- **D-23. Ledger 614's wrong answer is corrected everywhere it was restated,**
  by dating: in the planning commit, `ROADMAP.md` (the phase 13 line, criterion
  1's closing sentence and the progress row), `REQUIREMENTS.md` (the closing
  read's paragraph and GAP-01's closing line) and `STATE.md`; by 13.1-05,
  `docs/ALPHA_TESTING.md:736-738`, which a tester reads ("by decision, not
  reported to the toolkit's authors"). `13-51-SUMMARY.md` is the record of its
  day and is left as written.
- **D-24. The print check program is not where D-03 says.** Read on
  2026-10-03 with Python, which sees the live profile (the shell does not):
  `os.path.exists` on `C:/Users/prati/AppData/Local/Temp/wxpc/debug/wxdragon-print-check.exe`
  is False, and `wxpc/debug/deps` holds no files, the folder last written
  2026-10-02 23:02, while `wxpc/debug/wxWidgets` and the two CMake build
  folders remain. No source for it was found in the tree or the session
  scratchpads. Ledger 614 says so; nothing in this phase rebuilds it, and
  whether to is Pratik's.

## What the tree said, read 2026-10-03

- CI run 37054973323 (13-44.7's pull request #153), read with
  `gh run view 37054973323 --log`: the marker child "ran on
  WinSta0\wixen-marker-3720 for 300 s and did not finish in five minutes and
  was stopped", its output ending
  "webview_edge.cpp(609): 'WebView2::WebViewCreated' failed with error
  0x80070578 (Invalid window handle.)"; the signature child the same on
  `WinSta0\wixen-signature-4008`, all 14 of its window tests without a result;
  the meeting target 20 passed in 35.52 s and the invitation target 12 in
  32.74 s on the same runner.
- What the two failing children build and the two passing ones do not: the
  marker and signature window tests build `wx_compose::build_compose_dialog`
  under a hidden `Frame` (`a_marker_counts_at_the_start_of_any_line.rs:1234-1247`,
  `a_signature_follows_the_from_account.rs:1193-1219`), whose browser is
  `WebView::builder(&dialog)` in a `Dialog` (`wx_compose.rs:1091`); the
  invitation and meeting tests build `ReaderWindow::new`
  (`the_invitation_is_answered_from_the_reader.rs:1107`). That is a
  difference to read, not yet a cause.
- `.github/workflows/release.yml:6-7` runs on `workflow_dispatch` only, so a
  push of `main` publishes nothing; `ci.yml`, `accessibility.yml` and
  `nvda.yml` run on a push of `main`.
- `secure_delete` is set nowhere but the removal's own switch
  (`taken_off_this_computer.rs:85`) and the measurement
  (`what_forgetting_costs.rs:676`); the store's one production open is
  `MessageCache::new` at `src/data/message_cache/mod.rs:1397`, its pragmas at
  :1430-1434.
- The three counts the folder import loses are `MessagesImported`'s
  `already_here`, `could_not_be_read` and `not_written_down`
  (`importing_messages.rs:529-540`); `FoldersImported::carry_the_mail_counts`
  (`import_tree.rs:372-376`) carries the count brought in, the files too large
  and the saved Outlook counts, and none of the three.
- Ledger 545 and 758: `test_every_guard_record_still_names_one_place_in_the_tree`
  exempts the whole of `src/presentation/wx_app.rs` while one record's `after`
  is in it and its `before` is not, and two of its records name text that is
  not there. A plan editing `wx_app.rs` finds that file's anchors by script,
  not by the check.

## What only a person or a real provider can settle

- Whether the new Empty Folder sentence and the import sentences sound right
  under NVDA: the pull requests of 13.1-03 and 13.1-04 run the NVDA workflow,
  which hears only what its cases wait for; the tester's ear is the listening
  lines 13.1-05 adds.
- Whether an encrypted `.msg` from Pratik's own Outlook arrives as D-17 says.
- The wxDragon printing issue (ledger 614): his to file or not, after the
  details he asked for.

## Handover

13.1-01 is next. `main` is pushed by 13.1-01 after its merge, and not again in
this phase without Pratik's word. Phase 14, the real-account proofs, follows
this phase and is not planned.

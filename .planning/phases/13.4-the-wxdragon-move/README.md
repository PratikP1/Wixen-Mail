# Phase 13.4: The wxDragon move

Fifteen plans, one per wave, landed on 2026-10-24 against `main` at `84dec9ec`, 13.3-25's merge,
version `1.0.0-alpha.1`, inserted between phase 13.3 and phase 14 the way GSD inserts a decimal
phase. Phase 13.3 had closed that morning with its full gate green at `16e53490`, 11,332 tests,
and `main` was pushed, so `git rev-list origin/main..main --count` answered 0 before the commit
landing these plans. Phase 14 stood at 9 of 15 plans with none of its sittings held. On the day,
`guards/guards.toml` held 1,728 records by the TOML reader
(`python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`),
and `.planning/WINDOWS.md` held 957 entries before the landing and 958 after it, 792 open and
166 fixed.

Four more, 13.4-12.1 to 13.4-12.4, landed on 2026-10-25 against `main` at
`7c04003e`, 13.4-12's merge, between 13.4-12 and 13.4-13, on Pratik's answer of 2026-10-25
under "Pratik's answers", so the phase holds nineteen plans. They were planned outside the tree
that day under their own ids; 13.4-12.3 was split when its file list outgrew one executor,
which is why three answers make four plans.

The plans were written outside the tree: the four fixes on 2026-10-23, the eleven wxDragon plans
and the research on 2026-10-20. They were staged under working names, `FIX-01` to `FIX-04` and
`WXD-01` to `WXD-10` with `WXD-07.1`, and renamed to their 13.4 ids when they landed. The research,
`RESEARCH-the-wxdragon-move.md`, is copied here unchanged, so it still calls the plans by their
working names; the table below gives each plan's working name.

**What the phase is.** First, the four small fixes Pratik said yes to on 2026-10-23, each on the
recommendation its ledger row gives (ledgers 936, 937, 938 and 941). Then the move from wxDragon
0.9.17 to 0.9.23, and from wxWidgets 3.3.2 to 3.3.4, which he decided on 2026-10-20, "We need to
update wxDragon to the latest version and all that implies", for before phase 14's sittings. The
move runs in three parts: four plans at 0.9.17 that take the move's breaks and quiet changes out
of the way first (13.4-05 to 13.4-08), the move itself (13.4-09), then three plans that remove
what the move makes unnecessary and correct what still names the old toolkit (13.4-10, 13.4-11,
13.4-11.1), the check of every window's names on both channels (13.4-12), Pratik's NVDA sitting
on the build the sittings install (13.4-13), and the close with the phase's full gate (13.4-14).
Between 13.4-12 and 13.4-13, since 2026-10-25: the fixes
of 2026-10-25, 13.4-12.1 to 13.4-12.4.

**Requirements.** No requirement is added. Ten plans name the requirements they work under, nine
between them: REAL-01, LIST-25, FOUND-04, MAIL-04, LIST-02, SHIP-05, FOUND-08, FOUND-09 and
FOUND-23. Each adds a dated line under its requirement, adding no `[D]` clause and ticking no box.
Five plans name none and each says why: 13.4-05 (test plumbing), 13.4-07 (the composer's own
timer), 13.4-10, 13.4-11 and 13.4-11.1 (workarounds and comments the move made unnecessary). Those
five leave three completion marks, not four. Since 2026-10-25: eleven plans name a requirement, still
nine between them, 13.4-12.1 naming FOUND-23; eight name none, 13.4-12.2, 13.4-12.3 and
13.4-12.4 joining the five as fixes no requirement names. 13.4-12.3 adds a dated line under
GAP-12 and 13.4-12.4 under FOUND-09 and FOUND-13, adding no `[D]` clause.

## The plans

"Working name" is the name the plan was staged under. "Pushed" is whether the plan's branch goes
out with a pull request under Pratik's standing OK of 2026-09-23, which a plan uses when it changes
what is spoken or shown. Every plan merges with `git merge --no-ff`, and none pushes `main`. Each
plan depends on the one before it; 13.4-01 depends on 13.3-25.

| Plan | Working name | Wave | What it does | Requirement | Pushed |
|---|---|---|---|---|---|
| 13.4-01 | FIX-01 | 1 | With Allow Changes off, a contact typed here on a Google or Outlook account is counted as waiting on the setting at every contacts sync, as CardDAV already counts it, never as an error (ledger 936) | REAL-01 | yes |
| 13.4-02 | FIX-02 | 2 | When the Contact Manager cannot save a contact, the log names her by her id and the screen by her name (ledger 937) | REAL-01 | no, the log only |
| 13.4-03 | FIX-03 | 3 | The text reader gets the same cleaned markup the formatted view gets, with the "blocks left out" sentence above the words (ledger 938) | LIST-25 | yes |
| 13.4-04 | FIX-04 | 4 | While the server cannot be reached, the sentence that it would not take a draft is said once per draft until a save of that draft reaches it, and the first save that reaches it is said once in words naming the server (ledger 941) | FOUND-04 | yes |
| 13.4-05 | WXD-01 | 5 | Every live window test ends its event loop without sending the application object to another thread, at 0.9.17, held by a reading that was red first | none | no |
| 13.4-06 | WXD-02 | 6 | The main window's tick never runs inside itself: the six windows it can open open after it returns, one at a time, while the tick runs beside them; its pull request's Accessibility run is the 0.9.17 baseline | MAIL-04 | yes |
| 13.4-07 | WXD-03 | 7 | The message window's automatic save runs on a timer of its own, so it keeps saving after the move | none | no |
| 13.4-08 | WXD-04 | 8 | wxDragon's warnings and errors reach the log file at every log level, and no other library's lines do | LIST-02 | no |
| 13.4-09 | WXD-05 | 9 | Pratik confirms the exact versions and packages at a blocking checkpoint; then the move to 0.9.23 with the webview feature and the defaults, list cells read whole, tree rows freed, the browser crash at close held by tests, two advisories gone, Linux and macOS read from one dispatched run | SHIP-05 | yes |
| 13.4-10 | WXD-06 | 10 | Compose and Preview Before Send close at once, the watch that waited for their browsers gone (ledger 466) | none | yes |
| 13.4-11 | WXD-07 | 11 | The list cell allowance gone, the flag that stops Enter answering Yes taken by its name, and Tab out of the message body pinned both ways by a test (ledger 53) | none | no |
| 13.4-11.1 | WXD-07.1 | 12 | Every comment and upstream line citation that still names the old toolkit corrected, comments only | none | no |
| 13.4-12 | WXD-08 | 13 | Every window's names compared on MSAA and UI Automation between 13.4-06's run and 13.4-09's, the editable combo boxes' typing fields named on both channels, the Tab order, keys and dialog sizes checked | FOUND-08 | yes |
| 13.4-12.1 | its own id | 14 | A commit changing a test file runs the guard records whose suite is another target, passing over a changed `tests/X.rs` only for a record whose suite is X, held by cases in `check.test.sh` that were red first (ledger 962) | FOUND-23 | no |
| 13.4-12.2 | its own id | 15 | Each of the composer's sixteen Format menu items gets a letter no other item holds, Numbered List M, Undo O and Insert Link K among them, held by a test red on the old labels (ledger 972) | none | yes |
| 13.4-12.3 | its own id | 16 | Every two-answer question asked through Windows' task dialog with buttons named for what each does, such as Delete and Keep; Escape and Alt+F4 close it with nothing done (ledger 967, its first half) | none | yes |
| 13.4-12.4 | its own id | 17 | A "Don't ask me again" check box (Alt+G) in every such question, and a Settings page, Questions, where a stopped question comes back (ledger 967) | none | yes |
| 13.4-13 | WXD-09 | 18 | The build at 13.4-12.4's merge, after the fixes of 2026-10-25, its install page and the NVDA sheet; Pratik installs it and holds the sitting; the same build is the one phase 14's sittings install | FOUND-08, FOUND-09 | no |
| 13.4-14 | WXD-10 | 19 | The close: the ledger rows the move was for, the move's guard records measured, the cold build costs, every plan's marks read back, and the phase's full gate, `scripts/check.sh all` once by hand on its branch before its merge | FOUND-23 | only if its read changes the changelog |

The four rows from 13.4-12.1 were added on 2026-10-25; until
then 13.4-13 was at wave 14 with its build at 13.4-12's merge, and 13.4-14 at wave 15.

One per wave because every plan writes `docs/changelog.md`, `guards/guards.toml`,
`.planning/WINDOWS.md`, `.planning/ROADMAP.md` and `.planning/STATE.md`, and most write
`.planning/REQUIREMENTS.md`; a wave is a set of plans that share no file. 13.4-14 stays the closing
plan, and its full gate covers the four fixes as well, so none of them runs it by hand.

**Under the merge train.** Plans run as a merge train, the rule of 2026-10-22 in `CLAUDE.md`: the
next plan starts from the previous plan's branch while GitHub checks it. 13.4-05 says so in its
own text, cut from 13.4-04's tip while it is unmerged or from `main` once it has merged. The
wxDragon plans after it were written before the train and say "a branch cut from `main` after
13.4-NN's merge"; read that under the train the same way. Some preconditions ask for `main`
itself and so wait for the merge before them: 13.4-09's checkpoint and 13.4-13's build require
`main` to stand at that merge, and 13.4-10, 13.4-11.1, 13.4-12 and 13.4-14 require the plan before
to be merged into `main`. Since 2026-10-25, 13.4-13's build requires
`main` at 13.4-12.4's merge; 13.4-12.1 is cut from `main` after the landing, and 13.4-12.2 to
13.4-12.4 each from the branch of the plan before.

**What was renamed, and what was not.** Every plan id became its 13.4 id, in the text and in the
file names, branch names and log names a plan writes: `WXD-08-WHAT-THE-MOVE-CHANGED.md` is
`13.4-12-WHAT-THE-MOVE-CHANGED.md`, the heading "What WXD-09 must hear" is "What 13.4-13 must
hear", and the sitting's pages are `13.4-13-INSTALLING-THE-BUILD.md` and `13.4-13-SITTING.md`.
Threat ids were renamed with their plans, `T-WXD-05-SC` becoming `T-13.4-09-SC`, which is the
shape phase 13.3's plans use. The preconditions that read a merge message now read the train's,
`--grep='^Merge 13.4-08'` in 13.4-09, `^Merge 13.4-12` in 13.4-13 and `^Merge 13.4-05` in
13.4-14. Since 2026-10-25,
13.4-13 reads `^Merge 13\.4-12\.4:`, because `^Merge 13.4-12` matches every `Merge 13.4-12.N`.
The research is unchanged, and so are the scratchpad paths the plans cite as evidence.
The references to `FIXES-STAGING.md`, the staging note that is not in the tree, now point at this
README, which carries its content. 13.4-14's counts of the phase's plans were raised from eleven to
fifteen so its closing read covers the four fixes; its guard sweep still starts at 13.4-05's merge,
because the sweep is for records the toolkit move could have made stale, and 13.4-01 to 13.4-04
re-measure the records their own changes flag. On 2026-10-25 they were raised again, to nineteen, for
13.4-12.1 to 13.4-12.4.

## Pratik's answers

**The research's six questions, 2026-10-20: "yes to all. and let me see the issue before
posting."** In order: the move runs after 13.3 and before phase 14's sittings ("before the
sit[tings]"); the "one file" sentence in 13.3-23 is corrected (13.3-23's summary carries the
correction: one source file and 69 test files, and drafts no longer saved on their own); Linux and
macOS keep compiling, proved by one dispatched run of `other-platforms.yml` on 13.4-09's branch;
the two toolkit features nothing uses, AUI and rich text, are dropped in the move's commit; the
Visual C++ runtime gap is a ledger row now, 958, and a small plan after this phase; and he sees
the upstream issue about the state numbering before it is posted. Each answer is written under its
question in the research and folded into the plans.

**The four fixes, 2026-10-23: "yes"** to fixing ledgers 936, 937, 938 and 941 at the start of
this phase, before the move, each on the recommendation its row gives. The same day, to 13.3-21's
open choice: on a server without UIDPLUS, Send says once that the sent draft's copy may still be in
Drafts, and the retry at the next check drops it quietly. Kept, on the recommendation; no code
changes. 13.4-04's documents task writes that answer into FOUND-04's dated line, beside ledger
942, which still asks for such a server to be seen.

**Three questions the staging asked are not answered.** None blocks a plan: each plan runs on the
choice written in it unless Pratik says otherwise before that plan starts. That was true until
2026-10-24, when he answered the second and third, and 13.4-09's, in the paragraph after the list.

1. **13.4-01 is pushed.** It changes what a contacts sync says, from "1 error" to the Allow Changes
   sentence, so `CLAUDE.md` requires a pushed branch and a pull request anyway.
2. **13.4-04 keeps choice C-1:** after the refusal is said once, the first save that reaches the
   server is said once, "The server has taken the draft, so your other devices will have the
   newest copy." The composer's automatic save says "Draft saved." every minute whether or not the
   server took a copy, so a recovery in the old words, "The draft was saved.", would have sounded
   like one more of those. With C-1, an outage is two sentences that can be told apart from the
   minute's "Draft saved.": the refusal, then the recovery. Taking it out is one arm, two cases and
   two records.
3. **The four gaps found while planning are ledgered as the plans say, not folded in:** the
   calendar events manager writes an event's title to the log when it cannot save it, and the
   vCard import writes a skipped card's address to the log (13.4-02 opens a `todo` row for each);
   a server that refuses every copy of a draft when it holds none still says so at every automatic
   save, and Save Draft pressed after the refusal was said once says only "Draft saved: <subject>."
   (13.4-04 opens a `todo` row for each). Any of the four can be folded into its plan before that
   plan starts, if Pratik says so.

**Answered 2026-10-24, in chat: "ok to crate updates. Fold the other items in."** He was asked
three things in one message: whether 13.4-09's table of versions was the one to install, whether
the four gaps above should be fixed now, and, told it would run as planned unless he said
otherwise, 13.4-04's recovery sentence. So the four gaps are folded in, each by the
recommendation written for it and each with a failing case first: 13.4-02 fixes the calendar
events manager's titles and the vCard import's address in the log, and 13.4-04 the two draft
gaps, and none of the four `todo` rows is opened. 13.4-04 keeps choice C-1 with the plan's words,
since he asked for no change. And 13.4-09's table is confirmed as he saw it, wxdragon,
wxdragon-sys and wxdragon-macros from 0.9.17 to 0.9.23 with the packages the lock moves beside
them; 13.4-09 takes the preview again, proceeds if every package and version matches, and stops
at its checkpoint to report the difference if any does not. Recorded here once, by 13.4-02, the
first plan to run after the answer.

**The questions that stopped the train at 13.4-13, answered 2026-10-25 in chat: "Yes to fixes. As
to yes/no buttons, do it. But, also include a "do not ask again" checkbox in order to ensure that
the user is not bothered by them in the future. Once fixes are done, create the build."** He was
asked four things at 13.4-13's first task: whether `scripts/build-installer.sh` may run as
written, with its two `sed` reads; ledger 967, whether a question asked before something that
cannot be undone should carry buttons saying what each does rather than Yes and No, recommended
to keep Yes and No through the sittings; ledger 972, the composer's Format menu giving U, N and L
to two items each, recommended as a small fix before the sittings; and ledger 962, the commit
check passing over a guard record's suite when the record's file is a test file, recommended to
pass over a changed `tests/X.rs` only for a record whose suite is X. "Yes to fixes" is 962 and
972 on their recommendations, 13.4-12.1 and 13.4-12.2. "As to yes/no buttons, do it" answers 967
against the recommendation: every two-answer question gets buttons named for what each does,
13.4-12.3. The check box is 13.4-12.4: "Don't ask me again" on each question, and, since
`CLAUDE.md` asks that every setting be reachable from Settings, a Settings page, Questions, where
a stopped question comes back. "Once fixes are done, create the build" is his yes to item 1:
13.4-13 builds at 13.4-12.4's merge with the script as written, and its sheet covers the fixes.
13.4-12.4 carries one question of its own, "For Pratik": it recommends no box on eight questions
whose stop would let one key do something that cannot be undone, and gives every question the box
unless he names some before its second task.

## What upstream changed since the research

Two wxDragon issues this tree works around were fixed upstream on 2026-10-22, and neither fix is
in a release yet:

- Issue #250, the accessibility state numbering (ledger 534), filed from Pratik's account on
  2026-10-20, fixed by pull request #252.
- Issue #246, the printout's begin and end of document, which `src/presentation/printing.rs` works
  around by printing through Windows directly (ledger 614), fixed by pull request #251.

0.9.23 of 2026-10-15 was still the newest release on 2026-10-24 (`gh release list --repo
AllenDang/wxDragon`). So the plans' "unchanged at 0.9.23" stands, and both workarounds stay until a
release carries the fixes. Ledgers 534 and 614 carry a dated line saying so; the plan that moves to
the release carrying #252 changes `names.rs`'s numbering in the same commit.

## Where the fixes and the move touch the same files

Read on 2026-10-23 from each staged plan's `files_modified`, leaving out the files every plan
writes by rule. One file is shared: `src/presentation/wx_app.rs`, which 13.4-04 edits in one
comment and one doc above `spawn_draft_append`, and which 13.4-06 (the tick and the windows it
opens) and 13.4-11.1 (comments naming the old toolkit) edit elsewhere. Both take their line numbers
again at execution. 13.4-04 rewrites the guard record "a draft the server would not take is said
rather than logged", whose suite is `tests/wired.rs`, which 13.4-06 also edits; 13.4-06's
re-measure reads the record as 13.4-04 left it.

Two pairs meet in behaviour without sharing a file. 13.4-07 gives the composer's automatic save a
timer of its own (`wx_compose.rs`), and every tick of it reaches `save_as_draft` and so 13.4-04's
rule; 13.4-07 changes when saves happen and not what a save says, so neither plan's cases depend on
the other. And 13.4-13's NVDA sitting is the natural place for the two ear rows 13.4-03 and 13.4-04
open: two lines on its sheet, if 13.4-13's executor or Pratik adds them.

## What the re-measures cost

Counted on 2026-10-23 with Python's `tomllib` over `guards/guards.toml` (1,691 records at
`8745aef9`, 1,694 at `37c5204c`): the records whose `tests_last_seen` names the file each fix adds
tests to, which the count check flags at each plan's red commit.

| Plan | File gaining tests | Records flagged | At 126 to 149 s a record, derived |
|------|-------------------|-----------------|-----------------------------------|
| 13.4-01 | `src/application/contacts_sync.rs` | 93 | about 3.3 to 3.9 hours |
| 13.4-02 | `src/presentation/managers.rs` | 63 | about 2.2 to 2.6 hours |
| 13.4-03 | `tests/what_the_sender_hid_is_not_read.rs` | 5 | about 11 to 12 minutes |
| 13.4-04 | `src/application/draft_copy.rs` | 12 | about 25 to 30 minutes |

The rate is 13.3-18's, 126 to 149 seconds a record on the path that runs the whole library per
record. 13.3-24.6's fast path has merged since, so most of these records take it: a record flagged
only because the files it counts gained tests is asked a short question under its break, measured
at 74 s a record against 130 to 139 s through the whole run on 2026-10-23 at `b24a0f8d` (18 of 20
records, `docs/development/measurements.md`). The figures above are therefore a ceiling. Each plan
puts all its new cases in one red commit, so the records are flagged once, and starts the
re-measure in the background before its green. 13.4-03's cases go in the target rather than in
`reader_text.rs`, which 30 records name, for this reason.

## Two checkpoints stop the train

1. **13.4-09's first task.** Pratik confirms the exact versions, wxdragon, wxdragon-sys and
   wxdragon-macros at 0.9.23, and every package the lock moves with them, before anything is
   installed. No answer was on record on 2026-10-24, so the train stops there.
2. **13.4-13, his NVDA sitting.** He installs the build himself and works through the sheet with
   NVDA and Narrator. Its precondition also asks for his ruling on running
   `scripts/build-installer.sh`, which reads two values with `sed`, recorded for this run in this
   README or the plan's brief. His ruling came on 2026-10-25, "Once fixes are
   done, create the build." (under "Pratik's answers"), and 13.4-13's premise 7 records it.

Everything else runs without him. Pull requests open under the standing OK of 2026-09-23, and
tags and releases stay his word.

## Handover to phase 14

The research's answer to its question 1, Pratik's of 2026-10-20, puts this phase before phase 14's
sittings. So 13.4-13's build is the one the sittings install, its sheet opens Sitting 2, and phase
14's sittings resume on that build. 13.3-25 wrote that 14-08.2 builds after this phase closes;
the roadmap and the state file were corrected by dating when these plans landed. Phase 14's own
plan files are not edited here: 13.4-13's task 2 adds one line each to 14-08.2's and 14-10's
`<premise_corrections>` and one at the top of `14-SITTING-2.md`. Since 2026-10-25 it also corrects the steps of
`14-SITTING-1.md`, `14-SITTING-2.md` and `14-SITTING-4.md` that answer a question with Yes or
No, because 13.4-12.3 names both answers of every such question, and adds a line to 14-12's
`<premise_corrections>`.

After this phase, a small plan fixes the Visual C++ runtime gap (ledger 958), on Pratik's "yes" of
2026-10-20; where it falls among the sittings is his to say.

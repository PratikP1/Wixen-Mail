---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 51
subsystem: the alpha page, the guide, the shortcuts page, the listening page, the changelog, the phase's closing read, the ledger, the full gate
tags: [closing-read, listening-lines, full-gate, gap-01, gap-14]
status: complete

requires:
  - phase: 13-new-features-most-from-the-outlook-gap-audit
    provides: "seventy-one plans merged, 13-50 last at 975d6ef9; the requirement lines each plan wrote; the ledger entries each opened"
provides:
  - "docs/ALPHA_TESTING.md: a short-version paragraph on what phase 13 added, eight known-missing entries with the four answers, four new items in what would help most"
  - "docs/manual-accessibility-pass.md: items 96 to 181, one per ledger entry the phase opened for an ear, a reader on paper, an account or a mailbox; the count line corrected by dating"
  - "docs/KEYBOARD_SHORTCUTS.md: the Action menu's submenus counted as nine, with Answer Invitation's row"
  - "docs/USER_GUIDE.md: a rule run's Undo said as its last action only, corrected by dating"
  - "docs/changelog.md: S/MIME's 'sending is not built' corrected by dating"
  - ".planning/REQUIREMENTS.md: the closing read's paragraph, a closing line under each of GAP-01 to GAP-14, the traceability rows, the coverage re-taken"
  - ".planning/ROADMAP.md: a dated closing sentence on each of the fourteen criteria, the plan list, the row at 72/72, the milestone paragraph"
  - ".planning/STATE.md and .planning/WINDOWS.md: plan 72 of 72; ledger 609 and 614 fixed, 802 opened"
affects: [phase 14, which is planned when phase 13 closes]

estimate:
  tokens: 70000
actuals:
  # chars/4 over the added lines of the documents commit, leaving out the four long single
  # lines the diff counts whole though a sentence was added to each (STATE's stopped_at and
  # Current focus, the roadmap's row, the traceability rows)
  tokens: 21000
  tasks: 3
  commits: 2

tech-stack:
  added: []
  patterns:
    - "A listening item per ledger entry, grouped by subject under level-four headings so a screen reader can jump between groups, numbered on from the page's last item"

key-files:
  created:
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-51-SUMMARY.md
  modified:
    - docs/ALPHA_TESTING.md
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/manual-accessibility-pass.md
    - docs/changelog.md
    - docs/development/measurements.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/WINDOWS.md

key-decisions:
  - "Eighty-six listening items, not seventy-nine: the six unrun-verify entries 13-14 to 13-20 opened under phase 14's number and the three real-server todos of 13-44.6 to 13-44.8 are an account's checks and are included; 773 and 776, a locked session's gate run, are the machine's test runs and not a tester's walk, and are left out"
  - "Ledger 614 closed on answer (d) as this plan's brief gives it, nothing ever posted to wxDragon's GitHub, which is the entry's own closing condition; 609 closed because origin/main is 12-12's merge and its three runs passed"
  - "The gate's result goes in a second documents commit, as 12-12's did, because the run comes after the documents commit and its row cannot be written before it"

metrics:
  duration: "about 3 hours on 2026-10-03"
  completed: 2026-10-03
---

# Phase 13 Plan 51: The pages, the listening lines and the closing read Summary

**Phase 13 is closed: the full gate was green on its first run, the pages describe the
program the phase built, and every tick stands on a name in the tree.** `scripts/check.sh
all` at `19d3b0ae` exited 0 after 934 s with 10,549 tests passed. All fourteen of the phase's
requirements stand, each with a closing line.

## For a person

What changed in this round over the build the tester has, version `1.0.0-alpha.1`: File,
Print and `Ctrl+P` everywhere a message or an item is shown; Edit, Undo and Redo, several
steps in every box and the last action on messages and items, named; File, PGP Keys, with
keys locked by a passphrase asked for when a message needs one; meeting invitations said
before the message, answered from buttons, with an organiser's update or cancellation
reaching the calendar and times from another zone said at your hour; encrypted mail opened,
S/MIME and PGP, PGP signatures checked, and Sign and Encrypt in the composer; Report as Junk
and a block that offers to move the mail already here; people looked up in a directory and in
Microsoft's people search; free and busy asked of every place an account keeps a calendar,
with a colleague's own clock and hours; Edit Event scrolling; saved searches ordered, keyed
and made from nothing; other addresses to send from; Quick Steps; a rule run over a folder;
the Trash emptied after 15 or 30 days or as Wixen Mail closes; a folder exported as a
mailbox file or as message files, Outlook's saved messages imported, and imported messages
keeping their files. Issues closed by the phase's merges: #45, #47, #53, #58, #60 and #61,
each read closed by `gh issue view` on 2026-10-03; #49, #50, #52, #54, #55, #57 and #59 stay
open, each with a line only a real account settles. What only a person can settle is items 96
to 181 on the listening page and, behind them, the real accounts and organisers phase 14
owes. The four waits in plain words: all four were answered on 2026-09-24, yes to the `.msg`
library, yes to the printing and OpenPGP library parts, yes to the two Microsoft permissions,
and nothing is posted to the toolkit's authors about its printing fault. Two choices are still
his: which way, if any, to take deleted words out of the mail database file (ledger 788), and
the push of `main`, 513 commits ahead of GitHub (ledger 802). Next: phase 14, the real-account
proofs, the seventh of the seven groups, planned once this phase closes.

## The one-pass check

Command: `python one_pass.py` in the session scratchpad, read-only, over the fourteen GAP
blocks of `.planning/REQUIREMENTS.md`, at `975d6ef9`. No summary of the phase carries a
coverage block, so, as 12-12 did, the pass reads the requirement lines themselves.

```
GAP-01 to GAP-14 ticked, each with one [D] line and no unticked [D] line
refs: path#name 17, found 17; test names 278, found 276; paths 56, found 56; other 296
  NOT FOUND GAP-03 application::pgp_keys::tests::test_the_limits_say_a_locked_key_cannot_be_imported_and_none_is
  NOT FOUND GAP-05 presentation::reader_text::signature_tests::test_the_pgp_signature_sentences_are_five_different_sentences
ONE PASS COMPLETE
```

Both missing names are renames the requirement lines already record: 13-17.1 renamed the
first to `test_the_limits_say_a_locked_key_is_kept_locked_and_one_is`
(`src/application/pgp_keys.rs:661`), and 13-36.1 the second to
`test_the_pgp_signature_sentences_are_all_different_sentences`
(`src/presentation/reader_text.rs:4177`). The "other" refs are commands, module paths,
sentences and values, which the requirement lines quote as readings and which name no
function. After this plan's edits the same pass reads 280 test names with 278 found, 60 paths
with 60 found, the two renames as before.

## The requirements

| Requirement | Plans, last merge | Read |
|---|---|---|
| GAP-01 | 13-02 to 13-04, `da429da9` | Stands. (b) answered, 13-03 did not stop; (d) ledger 614, nothing posted. Ear and paper 613, 617 |
| GAP-02 | 13-01, 13-05 to 13-09, `aeb0c0c1` | Stands. Report and block undone as moves; Quick Step and rule run, the last action only (747) |
| GAP-03 | 13-16, 13-17, 13-17.1, 13-36.1, 13-44.6.1, `ed18ec1b` | Stands, one name through its rename. Ear 646, 650, 724 |
| GAP-04 | 13-10 to 13-13, 13-21.1 to 13-21.3, 13-36.2 to 13-36.4, `209f57a3` | Stands. Ear 8 entries, organisers 8 |
| GAP-05 | 13-14, 13-15, 13-18 to 13-21, 13-36.1, `90a4662e` | Stands, one name through its rename. (b) answered, 13-20 did not stop; 659 not counted |
| GAP-06 | 13-22, 13-24, 13-24.1, 13-25, 13-44.1, 13-44.5, `b53572fb` | Stands. Servers 675, 686, 689, 756, 768 |
| GAP-07 | 13-26 to 13-28, 13-44.2, `02bf76ae` | Stands. (c) answered, 13-28 did not stop. Directory 695, 698; Microsoft 702 |
| GAP-08 | 13-29 to 13-32, 13-44.4, `edd3a7d5` | Stands. 13-32, conditional, ran |
| GAP-09 | 13-37 to 13-39, `cce64ec0` | Stands. 741 is Pratik's |
| GAP-10 | 13-33 to 13-36, `85a5f873` | Stands. #59 open for shared mailboxes (721) |
| GAP-11 | 13-23, 13-24, 13-24.1, 13-40 to 13-42, `3c56a26b` | Stands. A step is not one undo (747) |
| GAP-12 | 13-23, 13-24, 13-24.1, 13-43, 13-44, 13-44.3, `e97c09c0` | Stands. Guide corrected on Undo |
| GAP-13 | 13-45 to 13-50, `975d6ef9` | Stands. (a) answered, 13-47 did not stop; 13-50, conditional, ran |
| GAP-14 | 13-44.6 to 13-44.9, `3b90bed8` | Stands. 788 is Pratik's |

`grep -c '^- \[x\] \*\*GAP-' .planning/REQUIREMENTS.md` answers 14 and the unticked form 0;
the plan's acceptance line said 13, written before GAP-14 was added. The coverage count,
`grep -c '^- \[[ x]\] \*\*[A-Z]\+-[0-9]\+\*\*'`, answers 123, with 123 traceability rows,
unchanged.

**Which of the four later actions are undoable**, the README's item-undo row, read in
`run_these_actions_over`'s comment at `src/presentation/wx_app.rs:26494` ("Each do-half
remembers its own action for Edit, Undo, so after a run of several the last one is the undo
step") and in the 13-22, 13-42 and 13-44.1 summaries: a report is, as a move, with the junk
mark taken off where it may have been set; a block's move is, as a move; a Quick Step and a
rule run are taken back one action at a time, the last first. The guide said so for Quick
Steps and not for rule runs, and its paragraph was corrected by dating.

## The four waits

| | What | Plan | Answer | Where it shows |
|---|---|---|---|---|
| (a) | `cfb` 0.15.0 | 13-47 | Yes, 2026-09-24; did not stop | `Cargo.toml:265` |
| (b) | three `windows` features; `rand` 0.8 renamed | 13-03, 13-20 | Yes, 2026-09-24; neither stopped | `Cargo.toml:472`, `:475`, `:481`; `:346` `rand08` |
| (c) | People.Read and Tasks.ReadWrite | 13-28 | Yes, 2026-09-24; did not stop | `src/service/oauth.rs:878`, `:882` |
| (d) | the wxDragon printing defect upstream | 13-03 drafted it | Nothing is ever posted | ledger 614, closed; `gh issue list --repo AllenDang/wxDragon --state all --author PratikP1` lists only #214, of 2026-09-04, about `TreeCtrl`, and a search for "print", "OnBeginDocument", "printout" and "StartDoc" finds nothing of this project's |

Both conditional plans ran: 13-32, Edit Event scrolling, merged at `9e1cb05b`, and 13-50,
imported messages keep their files, at `975d6ef9`.

## The four marks

The roadmap's plan list has 72 ticked lines of 72, counted with a read-only Python pass over
the `- [x] 13-...-PLAN.md` lines, and the row reads 72/72. `STATE.md` has `current_plan: 72`,
`Current Plan: 72`, `Total Plans in Phase: 72`, and `completed_plans: 244` of
`total_plans: 244`, from `ls .planning/phases/*/*-SUMMARY.md | wc -l` and the same over
`*-PLAN.md` once this file exists. The requirement marks: a closing line under each of the
fourteen and its traceability row. The phase line is ticked and the row's last column reads Complete
from the second commit, after the gate.

## The ledger

Fixed: 609, because `git log origin/main -1` reads `630e2a67`, which is 12-12's merge, and
`gh run list --branch main` reads CI, Accessibility and NVDA each success on it; 614, on
answer (d), its own closing condition. Opened: 802 (`unrun-verify`), the push of `main`, 513
commits ahead by `git rev-list origin/main..main --count` at `975d6ef9`, with fifty-six of the
seventy-one plans before this one having run the three workflows on their own pull requests,
#103 to #158. Both halves written for all three; `head -7 .planning/WINDOWS.md` reads 695
open, 107 fixed, 802 in all, agreeing with the rows and the JSON by
`the_planning_files_agree_with_themselves`.

## The pages

- `docs/manual-accessibility-pass.md`: section A's items counted by a read-only Python pass,
  95 before and 181 after, numbered 1 to 181 with none moved; the 86 added are one per
  ledger entry: the 77 open `unrun-verify` entries phase 13 opened under its own number
  except 773 and 776, the six 13-14 to 13-20 opened under phase 14's (639, 641, 651, 652,
  655, 657), and the three real-server `todo` entries 771, 784 and 787. The count line says
  one hundred and eighty-one, dated.
- `docs/ALPHA_TESTING.md`: the short version's paragraph in the order a tester meets it;
  eight known-missing entries for what the plans' own entries did not cover, the four answers
  among them; items 19 to 22 in what would help most. `grep -c -i 'print'` 11 and
  `grep -c -i 'quick step'` 5.
- `docs/KEYBOARD_SHORTCUTS.md`: every menu and key the phase added was found with its row:
  Print on `P` and `Ctrl+P`, Undo and Redo on `Ctrl+Z` and `Ctrl+Y`, Undo Send on `N`, Report
  as Junk on `J` and `Ctrl+Shift+J`, Block This Sender on `Ctrl+Shift+B`, `Alt+4` to `Alt+9`,
  `Ctrl+Shift+7` to `Ctrl+Shift+9`, PGP Keys on `K`, Manage Quick Steps on `M`, the two
  exports on `F` and `X`, each against `src/presentation/wx_app.rs`'s builders. One drift:
  the Action menu's submenu table said eight and left out Answer Invitation (`W`), on the
  menu since 2026-08-28; it now says nine, dated.
- `docs/USER_GUIDE.md`: one sentence a later plan made incomplete, the rule run's Undo,
  corrected by dating.
- `docs/changelog.md`: one sentence 13-21 falsified, S/MIME's "Sending signed or encrypted
  mail is not built", corrected by dating.

## The full gate

The clipboard target alone first, `cargo test --test a_passphrase_box_takes_a_real_paste`,
11 passed, so the session was not locked. `git rev-list HEAD..main --count` read 0
immediately before the run. Then, with nothing else building:

```
scripts/check.sh all > <scratchpad>/13-51-full-gate.log 2>&1; echo "exit=$?"
exit=0
check.sh: all passed after 934 s: start 1 s, rustfmt 4 s, clippy 47 s, the scripts that decide what runs 356 s, security advisories 12 s, tests 403 s, release build 111 s
```

At `19d3b0ae`, 17:16:08Z to 17:31:43Z on 2026-10-03, warm, NVDA running for the tester.
Summed over its 150 `test result:` lines: 10,549 passed, 0 failed, 14 ignored, against 8,985
on phase 12's gate. The audit: all 5 accepted advisories still reported, nothing outside
`.cargo/audit.toml`. Green on the first run, so no red and green pair was needed. The script
suites took 356 s against phase 12's 103 s; the row records that and does not explain it. The
run covers five of CI's seven jobs: no debug build, no setup executable, no search handler
checks, and no NVDA case or accessibility scan, which this plan's pull request runs and the
push of `main` (ledger 802) would. The row is on `docs/development/measurements.md`.

## Commits

| Commit | What | Hook |
|---|---|---|
| `19d3b0ae` | docs: the pages, the listening lines, the closing read, the four marks, the ledger | docs_only, 124 s |
| this commit | docs: the full gate's row, the phase line, this file's gate section | below |

## Deviations from Plan

1. **Fourteen requirements, not thirteen.** The acceptance line's 13 was written before
   GAP-14 was added on 2026-10-01; all fourteen were read and are ticked.
2. **The one-pass check read the requirement lines**, because no summary of the phase
   carries a coverage block, as 12-12 found for its phase.
3. **Ledger 614 closed rather than left for Pratik**, on answer (d) as the brief gives it,
   which meets the entry's own closing condition; and **609 closed**, its condition met by the
   runs on `630e2a67`.
4. **The gate's result is a second documents commit**, as 12-12's was: the brief asks for one
   documents commit before the merge, and the run, which comes after it, writes the row, the
   phase line and this file's gate section. The second commit touches documents only, so the
   code the gate read is the code that merges.
5. **A banned text tool ran once, read-only, on nothing tracked:** a range filter on `grep`'s
   line numbers over `.planning/REQUIREMENTS.md` early in task 2. No tracked file was touched
   by it; every tracked file was changed with Edit or Write.
6. **Eighty-six listening items**, not one per `unrun-verify` alone: the key decision above
   says which were added and which left out and why.

## Threat model

T-13-51-01: every page sentence written was checked against the tree, the menu letters and
keys against the builders in `src/presentation/wx_app.rs`, `wx_reader.rs`,
`editor_document.rs` and `wx_account_manager.rs`. T-13-51-02: no tick written or kept on a
name not in the tree; the one pass above. T-13-51-03: the four waits and both conditional
plans said in the requirements, the roadmap and here. T-13-51-04: the full gate, below the
documents commit, with `main` unmoved. T-13-51-SC: nothing installed.

## Self-Check: PASSED

Every file under key-files exists; the commits are named in the report that closes this plan.

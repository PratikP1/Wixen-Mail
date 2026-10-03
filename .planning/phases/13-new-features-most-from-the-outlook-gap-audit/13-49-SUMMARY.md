---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 49
subsystem: mail export pages
tags: [export, pst, msg, guide, gap-13, issue-53]
status: complete
requires: [13-45, 13-46, 13-48]
provides:
  - "tests/mail_goes_out_in_every_shape.rs: test_no_menu_offers_to_write_an_outlook_data_file and test_every_export_on_the_file_menu_is_named_in_the_guide, each with a planted companion"
  - "the guide's What goes out, and what does not, under Import and Export"
affects: [13-50, 13-51]
tech-stack:
  added: []
  patterns: ["a reading over every menu builder in src/presentation through what_ships, comment lines left out", "a page section cut once and read with its lines joined"]
key-files:
  created:
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-49-SUMMARY.md
  modified:
    - tests/mail_goes_out_in_every_shape.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/comparison.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
decisions:
  - "The new guard record breaks the export item's label on the File menu, not the guide's row: the guide names the command again under the table, so removing the row alone reddens nothing."
  - "A menu item counts as offering a .pst export when its label or help names an Outlook data file or .pst together with a whole word for writing out, so Import Mailbox's 'a message Outlook saved' is not one."
  - "The guide says .msg is not written because writing was not part of what was asked for (RESEARCH-5, section 2), and points to Save As and Export Folder as Message Files for .eml files instead."
metrics:
  duration: "about 1 hour 15 minutes"
  completed: 2026-10-03
estimate:
  tokens: 60000
actuals:
  tokens: 9000    # chars/4: about 13,000 in the test file, about 23,000 in the pages, ledger and requirements
  tasks: 2
  commits: 3
---

# Phase 13 Plan 49: The pages say which export is built and why `.pst` is not Summary

The guide, the comparison page and the changelog now say what became of issue 53's points 4 to
6: a folder goes out in three shapes, a saved Outlook message is read and not written, and an
Outlook data file is neither written nor offered on any menu, with the reason and what to do
instead. Two readings hold the "not a menu item" half and keep the guide naming every export the
File menu holds.

## What works now

- **The guide.** Import and Export gains "What goes out, and what does not": the three shapes as
  a list, each with its command; `.msg` read and not written; and decision 49's sentence, "Wixen
  Mail does not write Outlook data files (`.pst`), and no menu offers to. Only Outlook can check
  one, and the one library that writes them is too new to trust with your mail. To take mail to
  Outlook, export a folder as message files and drag them into one of Outlook's folders, which
  has not yet been tried with files Wixen Mail wrote, or let Outlook download the same account."
  The table already held the five commands as the File menu has them.
- **The comparison page.** A paragraph "Corrected later on 2026-10-03" says "still generous"
  above no longer describes what goes out, names the three shapes, and says Outlook data files
  and saved Outlook messages are read and not written, with the reason for each.
- **The changelog.** The gathered issue 53 paragraph rewritten: point 4 built, point 5 read and
  not written, point 6 refused and why, and what nobody has tried. The 2026-09-17 text is kept
  whole as "Written on 2026-09-17, this paragraph read: ...", and the three dated sentences 13-45,
  13-46 and 13-48 added stay as they were.
- **The readings.** `test_no_menu_offers_to_write_an_outlook_data_file` read 24 menus holding 168
  items in 97 sources under `src/presentation`, and requires that one of the menus read holds the
  File menu's three export items. `test_every_export_on_the_file_menu_is_named_in_the_guide`
  found 3 export items, "Export Mailbox", "Export Folder as a Mailbox File" and "Export Folder as
  Message Files", all named in the section.
- `docs/KEYBOARD_SHORTCUTS.md` is unchanged: its File rows for 13-45, 13-46 and 13-48 match the
  menu's labels, letters and descriptions on the day.

## Commits

| Commit | What | Hook |
|---|---|---|
| `b39fd021` | test: the two readings, the stubbed predicates, the four tests | red, 108 s |
| `21672a57` | feat: the predicates, one record new, two remeasured | affected, 250 s |
| docs | the guide, the comparison page, the changelog, the ledger, this summary, the four marks | this commit |

## Counts

Taken 2026-10-03: `cargo test --test mail_goes_out_in_every_shape` 10 passed (6 before).
`tr '\n' ' ' < docs/USER_GUIDE.md | grep -c 'does not write Outlook data files'` 1, and the same
for `'not yet been tried'` 1. The one-pass check of the names GAP-13's `[D]` line stands on: 9
read, 9 found as `fn <name>(` in their files, 0 not found. `grep -c '^- \[x\] \*\*GAP-13'
.planning/REQUIREMENTS.md` 1.

## Guard records

One new; the arrived-since count 625 to 626.

| Record | Break | Red |
|---|---|---|
| every export on the File menu is named in the guide (`suite = "mail_goes_out_in_every_shape"`) | Export Folder as Message Files' label made "Separate Messages" on the menu | the guide reading and the item's label reading |

The count check named two records after the red ("Export Folder as a Mailbox File asks whether
what is chosen is a folder" and "Export Folder as Message Files writes where the folder picker
said"). One `--remeasure` call named those two and the new one, 58 s: "All 3 guards redden
exactly the tests their records name."

## Ledger

Opened 799 (`unrun-verify`, Pratik, RESEARCH-5's A6): whether classic Outlook takes `.eml` files
dragged into one of its folders; until he answers the guide marks that advice as not yet tried.
502 reworded with a dated sentence to hold Export Mailbox's zip beside Save As, naming 789 and
792 for the two new exports. 499 gained a dated sentence: microsoft/outlook-pst-rs pull requests
#61, #62, #63 and #65, read open on 2026-09-24 and again on 2026-10-03, and #63's public real
`.pst`, the EDRM Enron set, which could close 499 without Pratik's own file. Both halves written
for all three; the header reads 695 open, 104 fixed, 799 in all. Nothing touches a server, so
nothing goes to phase 14.

## Requirement

GAP-13's box ticked, "Ticked 2026-10-03 by 13-49"; its `[D]` line carries the names it stands on
as `path#name`, part by part; its traceability row reads "Done 2026-10-03, 13-45 to 13-49" with
the 13-48 text kept as "Until 13-49". Both `[S]` lines untouched.

## Deviations from Plan

**1. [Premise] The guard record breaks the menu, not the guide's row.** The plan anchored the
break on 13-46's row in the guide. The guide names Export Folder as Message Files again in the
paragraph under the table, so removing the row leaves the reading green. The record renames the
item on the File menu instead, which is how a page falls behind its menu, and its comment says
why.

**2. [Shape] The menu reading's "read enough" check is a relation, not a count.** The plan asked
for at least one builder block per menu the File menu reading found. The test requires that one
of the menus read holds the File menu's export items, and quotes the menus, items and sources it
read in its failure text and on its output.

**3. [Shape] The guide's paragraph is a third-level heading with a list,** so a screen reader
user can reach it by heading and hear the three shapes as three items.

**4. [Brief] The changelog gets no new Added entry.** Only the pages changed; the gathered
paragraph is where issue 53's outcome is written.

**5. [Brief] No commit on `main` after the merge to add its mode line here.** The brief allows
one documents commit, before the merge.

**6. [Brief] A banned text tool ran once, read-only, on nothing:** a leftover `sed -n '1,0p'` at
the end of a read command early in task 2, and a do-nothing assignment named after it once
before. No tracked file was touched by either. Every tracked file was changed with Edit or Write,
by `cargo fmt`, or by `scripts/guards.sh`.

## TDD Gate Compliance

Task 1: `b39fd021` (test, red: the two companions and the count check, accepted by
`red-commit.sh`) then `21672a57` (feat). Both readings passed from their first run, as the plan
expected; the guide already named every export, so the guide reading was not red. Task 2 is
documentation.

## Known Stubs

None. Both readings read the tree; the predicates the red stubbed are written.

## Threat Flags

None beyond the register. T-13-49-01: every sentence checked against the File builder and the
writers; the drag-in advice marked untried and ledgered (799); the guide reading. T-13-49-02:
the no-menu reading and its planted companion; the page names the reason. T-13-49-SC: nothing
installed.

## Self-Check: PASSED

Both code commits are in `git log --oneline main..HEAD`; this summary exists; the four marks are
made in the documents commit.

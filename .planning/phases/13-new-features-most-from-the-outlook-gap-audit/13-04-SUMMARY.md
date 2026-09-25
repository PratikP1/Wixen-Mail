---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 4
subsystem: presentation::wx_reader, presentation::wx_app, presentation::printing, presentation::page_jumps, application::printing
status: complete
tags: [print, GAP-01, keyboard, reader-window, pim]
requires: [13-03]
provides:
  - Print on the reader window's File menu (Ctrl+P, letter P), each tab printing its paper composition
  - a conversation's row printing the whole conversation, each heading's date in full
  - Ctrl+P in the formatted message window, posted by the page as kind 'print'
  - Print in Contacts, Calendar, Tasks, Notes and Reminders on the item under the cursor
  - one path every surface prints by, presentation::printing::print_through_the_dialog, and the sentence it returns, application::printing::after_printing
  - tests/wired.rs's letter and key readings widened to the reader window, each window on its own
affects: [13-51]
tech-stack:
  added: []
  patterns:
    - "a tab's paper composed when Print is pressed, handed to the window as a closure, so nothing is fetched twice"
    - "a source reading that matches a call by its whole name, so on_paper( inside conversation_on_paper( is not read as a call"
key-files:
  created: []
  modified:
    - src/application/printing.rs
    - src/presentation/printing.rs
    - src/presentation/wx_reader.rs
    - src/presentation/wx_app.rs
    - src/presentation/page_jumps.rs
    - tests/print_is_on_the_file_menu.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/privacy.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "Every surface prints through print_through_the_dialog, which asks the dialog, prints and returns AfterPrinting; the main window says it on the status line, the two other windows through say_what_came_of_it. No surface words a sentence."
  - "An attachment tab prints as a job of its own kind, Kind::Attachment, 'Wixen Mail attachment', rather than as a message."
  - "A conversation of one prints as that message with its header lines, because that is how the formatted window shows a message on its own, the default way a message opens."
  - "A message tab's paper is composed when Print is pressed, from a closure the main window hands over, so opening a message does not fetch and decrypt it twice."
  - "The reader's Print is ID_READER_PRINT, ID_HIGHEST + 907, beside the reader's other ids, not the main window's ID_PRINT."
metrics:
  duration: about 1 hour 25 minutes of work, from 07:56 to 09:20 UTC, plus CI
  completed: 2026-09-25
actuals:
  tokens: 13036
  tasks: 3
  commits: 7
---

# Phase 13 Plan 04: Print on every surface that shows a message or an item Summary

Print now works everywhere a message or an item is shown. The reader window has Print on its
own File menu, `Ctrl+P` and the letter P, and prints the tab you are on: a message or a
conversation with every date written in full, or an attachment as its tab shows it. A
conversation's row in the list prints the whole conversation, each message headed with its
place, its sender and its date in full. `Ctrl+P` in the formatted message window, which is how
a message opens by default, prints what it shows. Contacts, Calendar, Tasks, Notes and
Reminders print the item under the cursor with the fields `Shift+Space` reads, one to a line,
and refuse in their own words when nothing is chosen. Every surface goes through one path to
Windows' print dialog and says one sentence afterwards. GAP-01 is ticked.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `246f9dec`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `9fdbf436` | red | The reader's Print cases in the reading, the conversation paper cases, `after_printing`, `Kind::Attachment`; 11 named, the count check among them | 132 s |
| `00521716` | green | The reader's Print, its tabs handed paper, the conversation row whole, `print_through_the_dialog`, `wired.rs` widened; three records new, seven re-measured | 220 s |
| `c6f68a67` | red | `Kind::for_module` and the per-module reading; 4 named | 104 s |
| `5b507fe6` | green | Print in the five modules, the messages-only sentence gone; one record new, eight re-measured | 268 s |
| `cd761b16` | red | The page's Ctrl+P listener and its case; the walk over every kind red; 3 named | 96 s |
| `7684be14` | green | `Jump::Print`, the window's answer, `say_what_came_of_it`; one record new, two re-measured | 274 s |
| this commit | docs | The guide, the privacy page, the changelog, the ledger, this summary, the four marks | |

**Test counts, taken again at the start and after.** `cargo test --test
print_is_on_the_file_menu` 4 before, 9 after. `cargo test --lib application::printing::` 24
before, 28 after (five added, one widened pair of expected arrays, none removed; the sentence
case lost two entries with their functions). `cargo test --test wired` 77 before and after.
`cargo test --lib presentation::page_jumps::` 5 before, 6 after. No test was added to
`wx_app.rs` or `wx_reader.rs`.

**Acceptance readings.** The reader's File letters after the edit, read with Grep in place of
premise 1's `awk`: R (Read Attachment), S (Save Attachment), P (Print), C (Close Tab), W (Close
Window), each once. `grep -c 'for_module(' src/presentation/wx_app.rs` is 1. 13-03's sentence,
"Print works on messages in this build", is found nowhere in `src`. `grep -c "kind: 'print'"
src/presentation/page_jumps.rs` is 1. The Reader Window section of the shortcuts page names
`Ctrl+P` in its table, read between its heading and The Preview Pane's with Read.

## Guard records

Five new; every record the count check flagged re-measured, one `scripts/guards.sh
--remeasure` call per commit that flagged them:

| Record | Red | Run |
|--------|-----|-----|
| the reader window's menu letters are held like the main window's (new) | the letter reading | task 1, 16 s |
| the reader window's keys are held like the main window's (new) | the key reading | task 1, 15 s |
| a message tab in the reader is handed what it prints (new) | the reader reading and its no-paper companion | task 1, 17 s |
| print in calendar refuses in calendar's words (new) | the module reading and its companion | task 2, 17 s |
| the conversation window reads the print kind the page's script posts (new) | the new page-jump case and the walk | task 3, 93 s |
| a count and the thing it counts agree in number; a printed line breaks at the last space; every page's stamp; an event's place; a page two ranges both name; a printed message carries its date in full; File, Print carries Ctrl+P (re-measured twice, tasks 1 and 2) | as recorded | tasks 1 and 2 |
| the page's script posts the attachments jump; the window reads the warning kind (re-measured) | as recorded | task 3 |

The arrived-since line at the head of `guards/guards.toml` went from 303 to 308. The
anchor reading, `test_every_guard_record_still_names_one_place_in_the_tree`, passed on every
commit through the hook.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The Print reading passed with the message path's `on_paper` removed.**
Found by task 1's re-measure: "a printed message carries its date in full" stayed green with
its break, because the reading asked whether the handler's text contains `on_paper(`, and it
still contained `conversation_on_paper(`. The reading now matches a call by its whole name
(`calls` in the test file), and the record reddens its two tests again, measured.

**2. [Shape] One path instead of each handler calling the dialog and the printer.** The plan
had the reader's handler call `ask_for_a_printer(` and `print(` and the main window keep its
own. Three surfaces would then have held three copies of the same five lines and the same
match on the outcome, so `print_through_the_dialog` holds the dialog and the print, and
`after_printing` the sentence. The reading holds each surface to the path and the path to
`ask_for_a_printer(` and `print_on(`; the existing "skips the dialog" companion now plants
into the path.

**3. [Shape] The two reader records are planted on Close Window, not on Print.** The plan's
breaks, "P&rint" claiming R and Print on `Ctrl+W`, would also redden the new reader reading,
which holds Print's label verbatim, so neither record could show the widened reading alone.
"&Close Window" claims C against Close Tab, and Close Window on `Ctrl+W` claims Close Tab's
key; each reddens only its widened reading.

**4. [Rule 2] `Kind::Attachment`.** An attachment read in a tab is not a message, and the job
names its kind, so it is "Wixen Mail attachment" and an untitled one "Attachment with no name".

**5. [Rule 2] A conversation of one prints as that message.** The formatted window shows a
message opened on its own as a conversation of one, and that is the default way a message
opens. Printed as a conversation it would have lost To, Cc and the attachment names.
`conversation_on_paper` prints one part through the reader's single-message composition.

**6. [Shape] A tab's paper is a closure, composed when Print is pressed.** Composing it when
the tab opens would fetch the body and ask `reading_a_message` a second time for every
message opened. A conversation tab's paper is composed from the parts already fetched.

**7. [Test] The page-jump case names `Jump::Print` in the green.** Its red asked only whether
the window read the kind at all, because a `Jump::Print` variant made in the red and never
constructed would have failed clippy's dead-code lint. The green tightened it to the variant.

**8. [Test] The module reading accepts an answer in braces.** rustfmt wraps three of the five
arms in braces, which the first form of the reading did not allow.

### Found and left

- Premise 3: `ThreadNode.date` is `MessageItem.date` as stored, so the reader's conversation
  tab and the formatted page head each message with the stored date. Paper writes it in full;
  the screen is ledger 618.
- The scan target opens the reader with plain `open`, so its tab would print as an
  attachment. It is a scan fixture and prints nothing; left as it is.

## Threat Flags

None beyond the register. T-13-12: the page's markup is sanitised by
`sanitize_and_count_held_back` in the thread renderer (`html_renderer.rs:1059`) before the
conversation window shows it, so a message's own script cannot post 'print'; a forged post
could only open the dialog. T-13-13: every job is named by `job_name(kind)` inside `print_on`;
the log line in `print_through_the_dialog` carries the printer and the page count, never the
title. T-13-14: the listener requires Control without Alt. T-13-SC: nothing added to the
manifest; `Cargo.lock` unchanged.

## Known Stubs

None. Every surface reaches the transport through a non-test path, held by the reading.

## Ledger

Opened: 617 (`unrun-verify`, the reader's and the formatted window's Print, the five modules'
Print and the pages, under NVDA and Narrator and on paper) and 618 (`todo`, the stored date on
the screen's conversation headings). Closed: none. Both halves of each.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does not
move; the changelog entry says so.

## Self-Check: PASSED

- `9fdbf436`, `00521716`, `c6f68a67`, `5b507fe6`, `cd761b16`, `7684be14`: in `git log` on
  `13-04-print-everywhere`.
- No em dash or en dash on an added line; none of the six words.

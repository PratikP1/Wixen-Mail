---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 2
subsystem: application::printing, presentation::read_aloud
status: complete
tags: [print, layout, GAP-01, read-aloud]
requires: [13-01]
provides:
  - application::printing, pure and tested, with Printable, Page, Kind, on_paper, from_document, from_item, lay_out, job_name and the three sentences
  - a fields method on the five PIM items that read_full joins and from_item prints
affects: [13-03, 13-04]
tech-stack:
  added: []
  patterns:
    - "one list of fields, marked long rather than transformed, read two ways: spoken and printed"
    - "a layout handed a measuring function, so the transport owns the font and the tests read as arithmetic"
key-files:
  created:
    - src/application/printing.rs
  modified:
    - src/application/mod.rs
    - src/presentation/read_aloud.rs
    - guards/guards.toml
    - .planning/WINDOWS.md
decisions:
  - "A dash in the title becomes a comma in the stamp, and so does a hyphen standing alone between words; a hyphen inside a word stays, and the text under the stamp keeps every dash it was written with."
  - "from_item prints the title as the first line and drops the reading's leading unlabelled name when it equals the title, so the name is printed once; an empty title prints as the kind's own wording, 'Note with no title' and its siblings, 'No subject' for a message."
  - "A labelled long field prints its label on a line of its own, 'Notes:', and the text under it as written."
  - "The sentence after printing counts pages through service::caldav::how_many, the one routine for a count and its noun, and that routine's guard record now names the printing sentence case."
  - "from_document keeps nothing together when the document has no empty line, rather than treating the whole of it as a header block."
  - "Tabs expand to stops of four columns; other control characters are dropped, since a printer would draw or act on them."
metrics:
  duration: about 1 hour 5 minutes
  completed: 2026-09-25
actuals:
  tokens: 11431
  tasks: 2
  commits: 5
---

# Phase 13 Plan 02: What a printed page holds Summary

`application::printing` turns a message, a conversation or one of the five items into pages,
and nothing reaches it yet. A message prints the reader's own header lines, Subject, From,
To, Cc, the date written in full and the attachments by name, then its text; a warning, when
there is one, prints above them. Lines wrap at the last space that fits a width the printer
will measure, a run with no space breaks where it must, the header block moves whole to the
next page rather than splitting, and every page is stamped "Wixen Mail, <title>, page N of M"
with no dash. The job is named by kind alone, "Wixen Mail message", so no subject reaches a
shared printer's queue. An event, a contact, a task, a note and a reminder print the fields
their Shift+Space reading says, one to a line, from one list both uses share; the 51 spoken
readings are word for word what they were. 13-03 wires File, Print; until then ledger 612
says the module has no caller.

`actuals.tokens` is chars/4 over the added lines under `src` and `guards` against `ef94cc4c`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `3dbd5576` | red | `application::printing` with every function answering nothing; the twelve message, layout, stamp and sentence cases named | 152 s |
| `04737f52` | green | `on_paper`, `from_document`, `lay_out` with its wrap and page filling, the stamp without dashes, `job_name`, the three sentences; two new records and the `how_many` record widened, three re-measured in one run | 153 s |
| `f0b22b64` | red | `Field` and a `fields` stub on the five items, `from_item` stubbed; the six item cases and the count check named | 132 s |
| `d84c2dcc` | green | `fields` on the five, `read_full` a one-line join over them, `from_item`; one new record, four re-measured in one run | 156 s |
| this commit | docs | the ledger, this summary, the four marks | |

**Test counts, taken again, not quoted from the plan.** `cargo test --lib application::printing::`
18 (the plan's floor was 18). `cargo test --lib presentation::read_aloud::` 51 before and 51
after, and 51 `#[test]` lines in `read_aloud.rs` on both sides, so none of its 7 records was
flagged by count. `grep -c 'fn fields' src/presentation/read_aloud.rs` 5. `grep -c 'pub mod
printing' src/application/mod.rs` 1. The spoken readings were run after each item moved to
`fields`: the contact, then the note, task and reminder together, then the event, 51 each
time.

## Guard records

Three new and one widened. One `scripts/guards.sh --remeasure` call per task over the
records that task touched, each reddening exactly what it names:

| Record | Red | Run |
|--------|-----|-----|
| a printed line breaks at the last space that fits, not the first (new) | the long-line case | task 1, 80 s |
| every page's stamp counts every page, not its own number (new) | the page-n-of-n case and the dash case, which also reads a stamp on a job of two pages | task 1, 92 s |
| a count and the thing it counts agree in number (widened: the printing sentence case added, `printing.rs` in `tests_last_seen`) | its 27 as before, and the printing sentence case, 28 | task 1, 80 s |
| an event's place is one of the fields both its reading and its page say (new, `read_aloud.rs`) | the event case | task 2, 105 s |

Task 1's run took 5 min 37 s over three records; task 2's, in the background with `--log`,
re-measured the three again (their file gained six tests) and the new one. The arrived-since
line at the head of `guards/guards.toml` went from 294 to 297; the file holds 1,093 records,
counted with the TOML reader. The anchor reading of premise 5 was re-run after `read_full`'s
bodies moved into `fields`: three records anchor inside `read_aloud.rs`, "the clause about a
day changed on its own goes quiet" and the two "short form is not the whole reading"
records, all in code this plan did not move, and
`test_every_guard_record_still_names_one_place_in_the_tree` passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The header-block case's own arithmetic was wrong in the red.**
Written against a page of four lines, it expected the gap under the header to be dropped as
though it were at the top of a page and so two pages; the rule gives three, the gap being
the foot of the second page. The red could not show it, because the case failed on an empty
page list before any expected value was compared. Corrected in the green `04737f52`, with a
comment working the four lines through.

**2. [Premise] No spoken event test names a location.** The plan expected the record that
drops the location to redden "the spoken event test that names the location". None of the 51
sets one: every event fixture in `read_aloud.rs` leaves `location` empty. So the event case
in `application::printing` asserts the spoken reading's "Location: Room 4" beside the page's,
which keeps `read_aloud.rs` at 51 as premise 5 wants, and the record names that one case,
measured.

**3. [Rule 2] The page count goes through `how_many`.** The plan asked for "1 page", never "1
pages", and did not say how. `service::caldav::how_many` is the one routine for a count and
its noun, and its record already guards that family, so the sentence uses it and the record
was widened and re-measured rather than a third wording written.

**4. [Scope] `from_item` is in task 2 only.** Task 1's red first declared `from_item`, whose
`Field` argument is task 2's type, and did not compile; it was taken out before the red was
committed and came back with task 2's red.

**5. [Rule 2] Three small rules the plan left open, each in a case:** an empty title prints
the kind's own wording rather than a blank first line and stamp (the empty-field case asks
all seven kinds); the item's name is printed once, as the title, and not again under it (the
contact case); a labelled long field prints its label on a line of its own (the contact
case). All three are in `decisions` above.

### Found and left to its plan

`reader_text::conversation` heads each message with its date as stored
(`reader_text.rs:1169`, `part.message.date.trim()`), because it takes no `Reading`, so
`on_paper` cannot reach a conversation's dates. 13-04's premise 3 already names this and
says what to do, so it is not ledgered twice here. The conversation case asserts order and
the header block, not dates.

## Threat Flags

None beyond the plan's register. T-13-04: `job_name` takes a `Kind` and nothing else, held
by the job-name case. T-13-05: `as_text` expands tabs and drops control characters, and
nothing is interpreted as markup. T-13-06: `longest_start_that_fits` measures a line's worth
of characters at a time, never the rest of the run, and the run case covers a run longer than
two lines. T-13-SC: no crate, feature or `Cargo.toml` line was added.

## Known Stubs

`application::printing` has no caller outside its tests until 13-03 wires File, Print. That
is the plan's own boundary, not a stub presented as complete: nothing in the program, the
guide or the changelog says printing works. Ledger 612, `stub`, says so, and 13-03's plan
names closing it. The five `fields` methods are reached today through `read_full`.

## Ledger

Opened: 612 (`stub`, `application::printing` has no caller until 13-03). Closed: none.

## Not pushed

Nothing spoken or shown changes: the 51 spoken readings are held unchanged and no window
calls the new module. So the branch was not pushed and no pull request was opened, as the
plan says. No `docs/changelog.md` entry either: 13-03's covers both.

## Version

No behaviour a person can reach changed, so the version does not move.

## Self-Check: PASSED

- `src/application/printing.rs`: present.
- `3dbd5576`, `04737f52`, `f0b22b64`, `d84c2dcc`: in `git log` on `13-02-print-layout`.
- Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE | wc -c`; no em dash or
  en dash on an added line; none of the six words.

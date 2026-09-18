---
phase: 11-reading-and-the-list
plan: 05
subsystem: the message list, the main timer, reading habits, the settings screen, guards, pages
tags: [read-state, mark-read-after, list-traversal, space-reads-aloud, enter-opens, one-rule, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-04.1 merged at 70d84bc5; main clean at f2fdc94f when the branch left it; the reading shape of tests/attachments_are_reached_with_alt_a_in_both_views.rs"
provides:
  - "application::reading_habits::whether_to_mark_read(began, selected_unread, now, setting): nothing when nothing began, whatever is selected and however long ago; nothing when the message that began reading is not the selected unread one; at once under Immediately, once the wait has run under a wait, never under Only when I say so; reads the setting's own marks_at_all and delay; six cases, 25 tests in the module, 1 record"
  - "application::reading_habits::WHAT_MARK_READ_COUNTS_FROM, the sentence under the choice; MarkRead::default()'s comment says what two seconds are counted from"
  - "wx_app.rs: WxUIState::reading_began, written by the mail read-aloud closure for the row Space or Shift+Space is about to read and by open_single_message for the message it opens (which takes the state now, four callers told), by nothing in the selection handler; mark_what_was_read in place of mark_the_open_one_read, asking the rule and doing the write it always did; the timer's opened_at RefCell gone; 199 tests before and after, 78 records"
  - "wx_settings.rs: a StaticText under Mark as read after on the Reading tab, named on both channels from the constant; no test, 20 records"
  - "tests/moving_through_the_list_marks_nothing_read.rs: five readings over what_ships and six companions over snippets; 11 tests, 2 records name it"
  - "tests/the_settings_dialog_opens_in.rs: the one test that builds the Reading page in a window session also reads that a child of the page carries the sentence, with child_texts walking the page's children; 7 tests before and after, 1 more record names it"
  - "guards/guards.toml: 926 records, census 798 + 128, four new records measured, none re-measured"
  - "docs/USER_GUIDE.md, docs/changelog.md: When a message counts as read; the entry under Unreleased, Fixed, naming #25 in the tester's words"
  - ".planning/WINDOWS.md: 539 unrun-verify, what only the tester's ear settles and the reading of previewed the close comment asks him to confirm"
affects: [11-06, which relabels Mark as Read beside the arm this plan left alone; 11-08, which makes a conversation row's message the selected one and so decides which message Space reads and this clock marks under conversation view; 11-12, which reads LIST-03's lines and the pages; whoever walks an inbox by ear]

actuals:
  tokens: 13634
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A decision that turns time into a state change is one pure rule over when the act began, asked by the timer with the moment; the timer keeps no clock of its own, and the acts write the moment where they happen"
    - "A companion for a reading whose anchors do not exist until the green reads a snippet shaped like the site, not the real text, so it is green on arrival and the red names only what is red"
    - "When a change removes the last caller of a helper the module tests, the new code reads the helper rather than matching the variants again, so nothing is left dead and the module's own tests still say what the helper does"

key-files:
  created:
    - tests/moving_through_the_list_marks_nothing_read.rs
  modified:
    - src/application/reading_habits.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_settings.rs
    - tests/the_settings_dialog_opens_in.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "open_single_message takes the state and writes reading_began itself, rather than each of its four callers writing it: opening is reading, and one site is what the reading holds"
  - "The rule reads MarkRead::marks_at_all and MarkRead::delay rather than matching the variants itself, because the timer's function was their only non-test caller and the rewrite would have left both dead beside the tests that describe them; done in task 2's green, where the callers went, and the six cases are unchanged"
  - "The read-aloud closure writes the moment on every press, so Shift+Space after Space restarts the wait for the same message: the simplest write, and a wait counted from the last press is still a wait counted from reading"
  - "a11y is no longer passed to the timer's function, which only ever discarded it; the mark is still not announced, and the comment says why"
  - "A fourth guard record, on the settings sentence, after the reading was taken red by hand with the sentence built and destroyed instead of added: the plan named three, and a measured break that costs 13 s is cheaper than a sentence nobody can prove is on the page"
  - "The sentence's presence on the built page is read in tests/the_settings_dialog_opens_in.rs, the plan's second option: no reading in every_event_has_a_control.rs's family reads a static text, and that target's test already shows the Reading tab in a window session; the count stays at 7"

patterns-established:
  - "A reading that holds a function to asking a rule also holds it to measuring no time of its own, because the second copy of the decision is where the old behaviour lived"

requirements-completed: [LIST-03]

coverage:
  - id: D1
    description: "Whether a message is marked read is one rule over when reading began, the selected unread message, the moment and the setting: nothing without a reading begun, nothing for another message, at once, after the wait, or never"
    requirement: LIST-03
    verification:
      - kind: unit
        ref: "src/application/reading_habits.rs#test_nothing_began_means_nothing_is_marked_however_long_a_row_is_selected"
        status: pass
      - kind: unit
        ref: "src/application/reading_habits.rs#test_a_message_that_began_reading_is_not_marked_once_another_is_selected"
        status: pass
      - kind: unit
        ref: "src/application/reading_habits.rs#test_immediately_marks_the_message_the_moment_reading_began"
        status: pass
      - kind: unit
        ref: "src/application/reading_habits.rs#test_a_wait_marks_nothing_before_it_has_run"
        status: pass
      - kind: unit
        ref: "src/application/reading_habits.rs#test_a_wait_marks_the_message_once_it_has_run"
        status: pass
      - kind: unit
        ref: "src/application/reading_habits.rs#test_only_when_i_say_so_marks_nothing_however_long_ago_reading_began"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a message nobody read is never marked read, whatever is selected', measured 2026-09-18 on the library"
        status: pass
    human_judgment: false
  - id: D2
    description: "Reading aloud and opening record when reading began, selecting records nothing, the timer asks the rule and keeps no clock, the old clock is gone"
    requirement: LIST-03
    verification:
      - kind: integration
        ref: "tests/moving_through_the_list_marks_nothing_read.rs#test_selecting_a_row_records_nothing_about_reading"
        status: pass
      - kind: integration
        ref: "tests/moving_through_the_list_marks_nothing_read.rs#test_reading_a_row_aloud_records_when_reading_began"
        status: pass
      - kind: integration
        ref: "tests/moving_through_the_list_marks_nothing_read.rs#test_opening_a_message_records_when_reading_began"
        status: pass
      - kind: integration
        ref: "tests/moving_through_the_list_marks_nothing_read.rs#test_the_timer_asks_the_rule_and_keeps_no_clock_of_its_own"
        status: pass
      - kind: integration
        ref: "tests/moving_through_the_list_marks_nothing_read.rs#test_the_clock_that_selection_started_is_gone"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'selecting a row starts no clock towards marking it read, whatever the wait' and 'the timer asks whether to mark what was read, not marks the selected one itself', measured 2026-09-18 on the target"
        status: pass
    human_judgment: false
  - id: D3
    description: "The setting says what it counts from on the Reading tab, the guide says when a message counts as read, the changelog names #25"
    requirement: LIST-03
    verification:
      - kind: integration
        ref: "tests/the_settings_dialog_opens_in.rs#test_pages_after_the_first_are_built_when_their_tab_is_first_shown_and_read_from_the_settings_when_never_shown"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the sentence saying what mark as read after counts from is on the reading page, not only in the source', measured 2026-09-18 on the target"
        status: pass
      - kind: command
        ref: "grep -n '### When a message counts as read' docs/USER_GUIDE.md -> 192; grep -c '#25' docs/changelog.md -> at least 1; cargo test --test house_style -> 74 passed"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the unread count survives a walk through the tester's inbox by ear, whether Space then the delay moves it, whether the sentence under the setting is read once, and whether previewed means reading aloud from the list"
    requirement: LIST-03
    verification: []
    human_judgment: true
    rationale: "Ledger 539; the close comment on #25 lists it and asks; nothing here has been heard, and no binary was started"

duration: 46min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 05: Moving through the list marks nothing read; reading starts the clock Summary

**Selecting a row in the message list never marks a message read now, however long the
row stays selected. Until this branch the main timer started a clock the moment the
selection landed on an unread message and marked it read two seconds later, and hearing
a row's sender, subject and date takes longer than two seconds, so a walk through a
folder by ear marked every message stopped on. The clock starts when a message is read
aloud from the list with Space or Shift+Space, or opened in its own window with Enter,
and the decision is one rule in `reading_habits` held by six cases: nothing when nothing
began, whatever is selected and however long ago; nothing for a message other than the
selected unread one; at once, after the wait, or never, as the setting says. The setting
keeps its seven answers and its default of two seconds, counted from reading now, and a
sentence under it says so. The preview pane cannot take focus in this program by design,
so reading aloud from the list is the act that previewing is here; the issue left the
word to the tester and the close comment asks him. Nobody has walked an inbox by ear
against this.**

## Performance

- **Duration:** 46 min from the branch at 19:34:20Z to the merge at 20:20:13Z; the
  summary and the planning files after. About 2 min 45 s was guard measurement in three
  foreground runs (93 s for the library record, rebuild 43 s and run 50 s, after a 51 s
  read of what already fails; 19 s and 18 s for the two target records after a 2 s read;
  13 s for the settings record after a 14 s read); about 11 min 50 s the five hook runs on
  the branch (114 s, 124 s, 92 s, 218 s, 160 s); 325 s the whole gate on the branch; 340 s
  `main`'s hook at the merge; 0 s waiting for the desktop, since no live-window test went
  red on any run.
- **Started:** 2026-09-18T19:34:20Z (the reading of the plan and the tree from about
  19:26Z)
- **Merged:** 2026-09-18T20:20:13Z at `5c82f680`
- **Tasks:** 3
- **Files modified:** 9, one created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat f2fdc94f..53417421 -- Cargo.toml Cargo.lock` (T-11-SC)
- **Actuals:** `tokens: 13634` is `git diff f2fdc94f..53417421 | wc -c`, 54,538
  characters over four, the branch's own diff against the commit it left `main` at; the
  estimate's `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.15.

## What landed

**Task 1.** `reading_habits::whether_to_mark_read(began, selected_unread, now, setting)
-> Option<i64>`: `began` filtered to the selected unread message, then `None` under Only
when I say so, the message at once when the setting has no delay, and the message once
`now.saturating_duration_since(since)` reaches the delay. Its doc says the clock starts
when a message is read aloud or opened and never when it is selected, with #25 and the
date, and why: selecting a row is how somebody moves through a folder. `MarkRead::default()`'s
comment says two seconds are counted from reading, what the old reasoning was and why it
did not hold for somebody working by ear. Six cases in a module of their own, 19 to 25.

**Task 2.** `WxUIState::reading_began: Option<(i64, Instant)>`, with a doc saying who
writes it and who reads it. The mail read-aloud closure writes it for the row it is about
to read, inside the lock it already takes, before the text is composed. `open_single_message`
takes `state: &Arc<StdMutex<WxUIState>>` and writes it first thing; its four callers,
the flat list's Enter, the one-message conversation's Enter under conversation view, and
the thread view's chosen message, hand the state they already held. The selection
handler is unchanged and names the field nowhere. `mark_the_open_one_read` is
`mark_what_was_read(app, marks_read)`: inside one lock it computes the selected unread
message's id, asks the rule with `s.reading_began` and `Instant::now()`, clears the field
on an answer, sets `read` on the row and takes its uid and subject; then, as before,
`MessageReadToggled(row, true)` and the server change, nothing announced. The timer's
`opened_at` `RefCell` and its comment are gone, and the call site's comment says what
counts as read. The doc comment is rewritten as the plan asked, with #25 named.

`tests/moving_through_the_list_marks_nothing_read.rs`: five readings, each a function over
`what_ships` of the main window returning a complaint: the selection handler, cut between
`msg_list.on_item_selected({` and `msg_list.on_column_click({`, names `reading_began`
nowhere; the mail read-aloud wiring, cut between its call and `let preview_visible`,
holds `reading_began = Some(`; `open_single_message`'s body holds it; `mark_what_was_read`'s
body calls `whether_to_mark_read(`, holds neither `.elapsed()` nor `duration_since(`, and
is called; the file holds `opened_at` nowhere, comments included. Six companions hand each
reading a snippet shaped like the sites, one as they should be and one with the fault
planted, and require the complaint. The changelog entry, under `[Unreleased]`, Fixed, in
the tester's words. Two records on `wx_app.rs` with `suite` the target.

**Task 3.** `reading_habits::WHAT_MARK_READ_COUNTS_FROM`: "Counted from when you read a
message aloud with Space or open it, never from moving onto it." A `StaticText` under
the choice on the Reading tab, `with_label` and `set_accessible_name` from the constant,
the way the sentence under the message-text size is built. In `tests/the_settings_dialog_opens_in.rs`,
the test that shows the Reading tab in a window session now also walks the page's direct
children by `GetWindow` and requires one whose window text is the constant, through a
`child_texts` helper in its `windows_of` module; taken red by hand first with the sentence
built and destroyed instead of added, then measured as a record. The guide's "When a
message counts as read" under Reading and Managing Email, dated. Ledger 539.

## Honest RED and GREEN

Two reds, two greens and one task without a red, on branch
`moving-through-the-list-marks-nothing-read` from `main` at `f2fdc94f`.

`d037cd15`, task 1's red, 114 s through the hook in `red` mode: the two `Some` cases
named by module path from cargo's own lines, under a stub answering `None` to
everything; the four `None` cases green on arrival and said in the commit, since a stub
answering nothing satisfies them. No file a record names gained a test, and the count
check printed no remedy.

`62abb7ac`, task 1's green, 124 s: the rule, the docs, one record measured.

`2e55b996`, task 2's red, 92 s: four readings named bare, each red for the reason intended
as quoted from the run: "the mail read-aloud closure never writes reading_began",
"open_single_message never writes reading_began", "fn mark_what_was_read( is no longer in
this file" and "the file still names opened_at". Seven green on arrival and said in the
commit: the selection reading, because the field did not exist, and the six companions,
which read snippets. The plan's checker had said exactly this.

`03221784`, task 2's green, 218 s: the field, the two writes, the rename and the ask, the
parameter and its four callers, the rule reading the helpers, two records, the changelog.
`cargo build` ran between the field and the rest and once more before the tests.

`53417421`, task 3, 160 s: the constant, the control, the reading added inside an existing
test and taken red by hand, the record, the guide, the ledger. Task 3 has no red of its
own and the plan said it would not; the settings screen, the pages and the ledger are
among CLAUDE.md's exceptions, and the one reading added is inside a test that already
existed.

Under the TDD gate's own terms, `test(11-05)` precedes `feat(11-05)` twice.

## Guard records

922 by the TOML reader before, 926 after: four new, none retired, none re-measured, since
no file a record names changed its count. Census 798 + 124 before, 798 + 128 after.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a message nobody read is never marked read, whatever is selected (new, library) | `reading_habits.rs` | when nothing began, the selected unread message stands in as though its reading had begun just now | 2, "all 2 tests named went red, and nothing else did": the nothing-began case under Immediately and the another-selected case | rebuild 43 s, run 50 s |
| selecting a row starts no clock towards marking it read, whatever the wait (new, `suite` the target) | `wx_app.rs` | the write put back into the selection handler for the row landed on | 1, "the one test named went red, and nothing else did": the selection reading | rebuild 18 s, run 1 s |
| the timer asks whether to mark what was read, not marks the selected one itself (new, `suite` the target) | `wx_app.rs` | the ask dropped and the selected unread message marked at once | 1: the timer reading | rebuild 17 s, run 1 s |
| the sentence saying what mark as read after counts from is on the reading page, not only in the source (new, `suite` the target) | `wx_settings.rs` | the sentence built and destroyed instead of added to the section | 1: the page-building test | rebuild 12 s, run 1 s |

Every first draft of a red list was a prediction and the runner agreed with each. The
companions stay green under every break because they read snippets rather than the file,
and each record's comment says so. Counts written: `reading_habits.rs` 25 on one record,
`wx_app.rs` 199 and the target 11 on two, `wx_settings.rs` 0 and `the_settings_dialog_opens_in.rs`
7 on one. The count check printed its remedy at no commit. `wx_app.rs` is at 199 before
and after, quoted by `cargo test --lib presentation::wx_app::` at task 2's green; 78
records name it now, two more than the 76 at the start.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `f2fdc94f` before
the branch. The line numbers had moved by eleven since the plan (`mark_the_open_one_read`
at `10046`, the poll at `5577`, `opened_at` at `5464`, `5577`, `10049`, `10065`, `10070`
and `10089`, `set_can_focus(false)` at `1340`, `wire_read_aloud` at `9988`, the mail
wiring at `3434`, `open_single_message` at `12479`); the shapes held, including that
nothing in `open_single_message` writes the read flag and that the profile's `never` was
taken as given. Four things the plan did not have:

1. **`open_single_message` takes no state, and has four callers, not one.** The plan's key
   link says `open_single_message -> reading_began`; the function had `frame`, `reader`,
   `a11y`, `cache`, `message` and `closed` and no way to the state. It takes the state
   now; the four callers each held one. `tests/wired.rs`'s `THE_SURFACES` reads the
   function by its signature line and still finds it, 77 passed.
2. **`marks_at_all` and `delay` had one non-test caller each, the timer's function.** The
   plan's rewrite would have left both dead beside the two module tests that describe
   them. The rule reads them, so a change to what the setting means is one place.
3. **The plan's premise counted `wx_app.rs` under 73 records; 76 at the start**, 11-04 and
   11-04.1 having added three, and 78 now. `reading_habits.rs` at 19 tests and 0 records
   held; `wx_settings.rs` at 0 tests and 19 records held, 20 now.
4. **No reading in `every_event_has_a_control.rs`'s family reads a static text on the
   Reading tab**, so the plan's first option for holding the sentence on the built page
   was not there; its second option was taken, in `the_settings_dialog_opens_in.rs`,
   whose page-building test already showed the tab, with one helper and one check.

And one thing the plan did not name that the tree already had, left where it is: under
conversation view `selected_message_index` indexes the conversation rows and
`messages[idx]` is a message of the flat list, which the phase README records against
#31 and 11-08 corrects. Space on a conversation row already read that message aloud, so
this plan's clock marks the same message the old one did there, only after Space rather
than after selection; Enter on a conversation row opens the thread view, and a message
chosen there begins reading under its own id, which the rule refuses to mark while the
selected one is another. Nothing is marked that was not read; a message read from a
thread stays unread until 11-08 makes the row's message the selected one.

## Deviations from plan

**1. [Decision] `open_single_message` takes the state**, decision 1 and the first
contradiction.

**2. [Rule 2 - Correctness] The rule reads the setting's helpers**, decision 2 and the
second contradiction: a rewrite that leaves two tested helpers dead is the kind of thing
`dead-code-hunter` exists to catch, and the fix is one line in the rule.

**3. [Decision] Every press writes the moment**, decision 3.

**4. [Decision] `a11y` dropped from the timer's function**, decision 4; the reading's
snippet and the record's break both use the two-argument call.

**5. [Decision] A fourth record**, decision 5; the plan's verification says three and
this summary quotes four.

**6. [Decision] The built-page reading in `the_settings_dialog_opens_in.rs`**, decision 6
and the fourth contradiction.

Everything else executed as written. **No scripted edit touched a tracked file: the
exception set for this plan is zero, and it stayed there.** Every tracked file was changed
by Read then Edit or Write; the new target was written with Write; `cargo fmt` ran before
each Rust commit; `scripts/guards.sh --remeasure` wrote the counts on `guards/guards.toml`.
The only `sed`, `awk`, `grep`, `tr` and `python` in the session read files, logs and the
records file; `git checkout` was used once, to move to `main` for the merge; the one
break taken by hand was made with Edit and put back with Edit. Commit messages were
written to the scratchpad and passed with `-F`. Carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No em dash
in any file this plan wrote, measured by `grep -c` for the byte sequence: zero; none of the
six words, measured by `grep -ciE`: zero. `git commit` and `git merge`, never `gsd-tools
query commit`; never `--no-verify`; `check.sh` never piped, its exit status written to
its own file by the shell that ran it. No AI attribution in any commit. `Cargo.toml` and
`Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's profile was not
read (the plan's premise 4 fact, `never`, was taken as given); no binary was started,
neither the installed one nor the tree's; NVDA was not stopped, reconfigured or driven;
every commit was made from the primary checkout, none from `wixen-mail-sweep` or
`wixen-mail-mutants`. `WIXEN_TEST_THREADS` untouched. The version stays `1.0.0-alpha.1`.
Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/reading_habits.rs` | `--lib application::reading_habits` on both of task 1's commits and task 2's green, and the whole-tree guards |
| `tests/moving_through_the_list_marks_nothing_read.rs` | itself on the red, and through its two records' coupling from `03221784` on |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the nineteen coupled targets its records name, on task 2's green |
| `src/presentation/wx_settings.rs`, `tests/the_settings_dialog_opens_in.rs` | the ten coupled targets `wx_settings.rs`'s records name, and the target itself, on task 3 |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every commit |

`scripts/check.sh all` ran once on the branch at `53417421`, output to a file with the exit
status written by the same shell: exit 0, 8,090 passed and none failed over 78 result
lines, 325 s from 20:08:47Z to 20:14:12Z, the release build included; one more result
line than 11-04.1's 77, the new target, and 17 more tests, 11 in the target and 6 in the
module. `main`'s hook at the merge, 340 s from 20:14:33Z to 20:20:13Z, the same 8,090 and
none failed. The tester's copy and NVDA were open on the desktop throughout; no
live-window key test went red on any run, so the input was never checked for idleness.
The keyring race (ledger 374) did not appear.

## Threat register

T-11-19 mitigated: the rule answers nothing without a began, the selection reading holds
the handler clear of the field, and a record measures the break. T-11-20 mitigated: the
rule requires the began id to be the selected unread one, with a case, and the thread
view's chosen message is the case where that refuses (above). T-11-21 mitigated: the
sentence under the choice, read on the built page, and the guide's paragraph, dated.
T-11-SC: nothing added. New surface outside the register: none; the timer writes the same
flag change it wrote before, for a message the rule named.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 538 before, 539 after;
505 open before, 506 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 539 | unrun-verify | what only the tester's ear settles for #25: a walk through a folder with unread messages leaving the count where it was; Space, then the count moving after two seconds with the row still selected, and Enter doing the same; moving off before the delay leaving it unread; the sentence under Mark as read after read once; and whether "previewed" in his words is reading aloud from the list, asked in the close comment |

## The issue

`gh issue close 25` from the repository root after the merge, closed at
2026-09-18T20:20:38Z, the plan's sentence with the merge commit, then the ear list: the
walk leaving the unread count alone; Space or Shift+Space then the count moving after two
seconds, Enter the same, moving off before then leaving it unread; the sentence under the
setting read once; and whether "previewed" in his words is reading aloud from the list, as
taken, or something else. Closing an issue is not a publish; nothing was pushed.

## Known stubs

None. `whether_to_mark_read` has one non-test caller, `mark_what_was_read`, reached from
the main timer's poll on every tick; `reading_began` is written at two sites on the paths
Space and Enter take and read at one; `WHAT_MARK_READ_COUNTS_FROM` has one non-test
reader, the settings screen's Reading tab, and its presence on the built page is read;
`marks_at_all` and `delay` are read by the rule.

## Not done here, on purpose

Whether the unread count survives a walk, whether Space then the delay moves it, and what
"previewed" means to the tester are ledger 539 and the close comment; the changelog says
nobody has walked an inbox by ear against this. The conversation view's index mismatch is
11-08's, above. The guide's context-menu list under Message Actions, which says Mark as
Unread, and its shortcuts line saying Space toggles read and unread, are 11-06's, as the
plan said. LIST-03 is ticked on its `[D]` lines, each covered above, with its `[S]` line
untouched: the phase README says 11-12 ticks the `LIST` requirements clause by clause, and
the orchestrator's instruction for this plan was to tick it where held, which is the
overrule with its reason; 11-12 still reads it. The row is `6/20`.

## Self-Check: PASSED

`tests/moving_through_the_list_marks_nothing_read.rs` exists; `grep -c 'opened_at'
src/presentation/wx_app.rs` is 0, `grep -v '^\s*//' src/presentation/wx_app.rs | grep -c
'reading_began = Some('` is 2 and the same for `whether_to_mark_read(` is 1;
`grep -c 'WHAT_MARK_READ_COUNTS_FROM' src/presentation/wx_settings.rs` is 3 and
`src/application/reading_habits.rs` defines it once; `docs/USER_GUIDE.md` has `### When a
message counts as read` at line 192; `guards/guards.toml` holds 926 records by the TOML
reader and the census says 798 + 128; `.planning/WINDOWS.md` holds 539 in both halves;
`.planning/REQUIREMENTS.md` has LIST-03 ticked; `gh issue view 25` answers CLOSED. Commits
`d037cd15`, `62abb7ac`, `2e55b996`, `03221784`, `53417421` and `5c82f680` are in
`git log --oneline --all`. Carriage returns zero and em dashes zero on this file,
`STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

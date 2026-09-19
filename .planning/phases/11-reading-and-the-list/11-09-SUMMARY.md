---
phase: 11-reading-and-the-list
plan: 09
subsystem: the row under the cursor read column by column with its headings on request, as a pure rule over the cells the list paints; the key and the Action item; the three routes to a quiet traversal on the pages with the steps for NVDA's profile; guards, pages, ledger
tags: [column-headers, nvda, row-column-headers, configuration-profile, announce_content, mute, message-rows, virtual-rows, action-menu, accelerator, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-08 merged at 75c211fe (a conversation row's cells from the here group, the cursor handler on on_item_focused, the row message's sender first in the Correspondent cell); 11-08.1 merged at 76897058 (a conversation row's thread_id the store's word); 11-07 (the cursor is the focused row, selected_message_index); 11-06 (list_keys::wire_letter for a plain key, a chord through the accelerator table); phase 1's virtual_rows::text_for and message_rows; main clean at 39537d13 when the branch left it"
provides:
  - "presentation::message_rows::the_row_with_its_headings(cells) -> String: the cells in the order given, an empty cell left out, a self-describing cell said alone, every other cell as heading then text, each cell a sentence, a cell already ending one not given a second full stop, an empty list an empty string; 41 tests before and after, 3 records name the file"
  - "presentation::message_columns::MessageColumn::heading_is_worth_saying(self) -> bool: false for Unread, Attachment, Flagged, Answered, Draft and Safety, true for the rest; 43 tests before and after, 2 records"
  - "wx_app.rs: ID_READ_ROW_COLUMNS on the Action menu after Previous Unread as Read the Row's Headings and Te&xt with the chord Ctrl+Shift+;; the arm handing the list, the state, the layout, the dates and the channels to read_the_row_with_its_headings, which refuses unless the message list holds focus and a row is under the cursor, reads the visible layout's cells in its order through virtual_rows::text_for, composes them through the rule and announces once through announce_content; 199 tests before and after, 98 records"
  - "tests/the_rows_columns_are_read_on_request.rs: ten cases over the composition, two of them over the cells virtual_rows::text_for paints for a message row and a conversation row; four readings over what_ships of wx_app.rs with three companions; 17 tests, 2 records name it"
  - "guards/guards.toml: 975 records, census 798 + 177; two new, measured on the green tree"
  - "docs/KEYBOARD_SHORTCUTS.md: the row under Reading the Item Under the Cursor and in the Action Menu table, the layout note; under NVDA Shortcuts, Column headers on every row: where the header comes from, the three routes and why the third, the four steps with NVDA 2026.3's own names read 2026-09-19, one sentence each for JAWS and Narrator; docs/USER_GUIDE.md: Hearing a row's columns with their headings under Reading and Managing Email; docs/changelog.md: the entry under Unreleased, Added"
  - ".planning/WINDOWS.md: 550 unrun-verify, what only the ear settles; 551 todo, the add-on route if the profile is too much to ask"
affects: [11-10, which prefixes a rule's phrase to the first visible cell and so to the first cell this reading says; 11-09.1 and 11-09.2, which change what a row's cells say and are read back by this key as they are painted; 11-13, which reads the two refusal sentences this plan adds; 11-12, which reads LIST-07's lines and the pages; whoever hears a row under NVDA's default]

actuals:
  tokens: 15198
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A reading on request is composed from the very function the paint callback answers with, over the visible layout in its order, so what is heard and what is on screen cannot come apart and the reading gains no cell logic of its own"
    - "A menu letter is chosen by the guard's own rule, the builder chain and what is appended after build(), not by a reading of the chain alone; the comment on the item says which letters the menu had left"
    - "A guard record whose before line exists twice in the file carries the closing brace, and its comment says why, so the one-place check has one place to find"

key-files:
  created:
    - tests/the_rows_columns_are_read_on_request.rs
  modified:
    - src/presentation/message_rows.rs
    - src/presentation/message_columns.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The item is Read the Row's Headings and Te&xt on x, not the plan's Read the Row's &Columns on c: c is Blo&ck's, and every letter of the plan's label is claimed on the Action menu by an item or a submenu appended after build(); j, q, x and z are the four the menu has left, re-taken by the rule tests/wired.rs applies, and Text holds one of them"
  - "Each cell is a sentence and a cell that already ends one is not given a second full stop, found by the composition case over a snippet ending in one; the plan's join on '. ' would have read 'in..'"
  - "The arm's body is a function of its own, read_the_row_with_its_headings, and the readings hold the arm to reaching it and the function to what it does, on a_move_completes_here_first's pattern; the plan's readings over the arm's own text would have put fifty lines in a closure of five thousand"
  - "A row whose every visible cell is empty is refused in words rather than met with silence, so the key is not mistaken for a dead one"
  - "NVDA's names are read from the user guide that ships with the NVDA installed on this machine, 2026.3 (alpha-57645), rather than from the web, which this executor cannot reach; the page names the version and the date and points at Help, User Guide for a later spelling"
  - "The JAWS and Narrator sentences claim nothing heard: the reading goes out the same way whichever screen reader runs, and nobody has heard it under either"

patterns-established:
  - "A premise command that reads half of what an existing guard reads reports a fact the guard would refuse; observation 724 in the skill log"

requirements-completed: [LIST-07]

coverage:
  - id: D1
    description: "the_row_with_its_headings composes the visible cells in order as heading then text, the six self-describing columns as their text alone, empty cells left out; heading_is_worth_saying names the six over ALL; the two comments corrected"
    requirement: LIST-07
    verification:
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_a_row_reads_each_heading_then_its_text_in_the_order_the_columns_are_shown"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_order_is_the_order_given_and_not_the_columns_own"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_an_empty_cell_is_left_out_altogether"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_a_cell_that_already_ends_a_sentence_is_not_given_a_second_full_stop"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_no_cells_read_as_nothing"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_a_self_describing_cell_is_said_as_its_text_alone"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_every_other_cell_is_said_as_its_heading_then_its_text"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_which_headings_are_worth_saying_is_decided_for_every_column"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_a_message_row_is_composed_from_the_cells_the_list_paints"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_a_conversation_row_is_composed_from_its_own_cells"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the row read on request says a self-describing cell alone, not its heading twice' at 3 red on the target, measured 2026-09-19 at the red, at the count check's remedy and on the green tree"
        status: pass
      - kind: command
        ref: "cargo test --test the_rows_columns_are_read_on_request -> 17 passed; --lib presentation::message_rows:: -> 41; --lib presentation::message_columns:: -> 43; grep -c 'pub fn the_row_with_its_headings' src/presentation/message_rows.rs -> 1; grep -c 'not being read' src/presentation/message_rows.rs -> 0"
        status: pass
    human_judgment: false
  - id: D2
    description: "An Action menu item with the chord announces the row once as content the mute controls, for a message row and a conversation row, and refuses off the list"
    requirement: LIST-07
    verification:
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_item_is_on_the_action_menu_with_its_chord"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_arm_reaches_the_reading"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_reading_composes_the_focused_row_from_what_the_list_shows"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_reading_is_announced_once_as_content"
        status: pass
      - kind: integration
        ref: "tests/the_rows_columns_are_read_on_request.rs#test_the_readings_complain_when_the_row_is_not_the_lists_own_or_is_said_another_way"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the row read on request goes out as content under the mute, not as interface chatter' at 1 red on the target, measured 2026-09-19 on the green tree"
        status: pass
      - kind: command
        ref: "cargo test --lib presentation::wx_app:: -> 199 passed; --test wired -> 77; grep -v '^\\s*//' src/presentation/wx_app.rs | grep -c 'ID_READ_ROW_COLUMNS' -> 3; grep -c 'Ctrl+Shift+;' src/presentation/wx_app.rs -> 1; bash scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_app.rs names the target"
        status: pass
    human_judgment: false
  - id: D3
    description: "The pages say the three routes, that the third is taken and why, and give the steps for the NVDA profile with Row/column headers off, naming Narrator's and JAWS's own settings; the add-on is later work in the ledger"
    requirement: LIST-07
    verification:
      - kind: command
        ref: "grep -c 'Ctrl+Shift+;' docs/KEYBOARD_SHORTCUTS.md -> 6; grep -c 'Row/column headers' docs/KEYBOARD_SHORTCUTS.md -> 3; grep -c 'Hearing a row.s columns with their headings' docs/USER_GUIDE.md -> 1; grep -c '#26' docs/changelog.md -> 2 (one this entry); ledger 550 and 551 in both halves; cargo test --test house_style -> 74; --test docs_links -> 6; --test the_planning_files_agree_with_themselves -> 16; --test the_words_that_say_nothing -> 9; --test a_key_is_documented_where_the_surface_that_binds_it_is -> 3"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the row is heard whole and once on the key and on the item, whether arrowing is quiet under the profile the page describes, and what Narrator and JAWS need"
    requirement: LIST-07
    verification: []
    human_judgment: true
    rationale: "Ledger 550; the close comment on #26 lists it; nobody has set the profile up or heard the reading, and NVDA was not driven here"

duration: 55min
completed: 2026-09-19
status: complete
---

# Phase 11 Plan 09: The row's columns are read on request, and the headers on every row are NVDA's setting Summary

**`Ctrl+Shift+;`, and Read the Row's Headings and Text on the Action menu, read the row under
the cursor in the message list column by column, each heading then its text in the order the
columns are shown, once, as content the mute controls: "Subject, Quarterly report.
Correspondent, Ada Lovelace. Unread. Received, yesterday." An empty cell is left out, a cell
whose text already says what its column is (Unread, Has attachment, Flagged, Answered, Draft,
a safety verdict) is said alone, a cell that already ends a sentence is not given a second
full stop, and a conversation row reads its own cells. The composition is a pure rule over the
cells the list paints, gathered through the very function the paint callback answers with, so
what is heard is what is on screen. Pressed with the message list not focused or no row under
the cursor, the key says "Nothing is selected in the message list". The header before each
cell as somebody arrows is NVDA's own reading of a report-view list, from Document
Formatting's Row/column headers, and nothing this program sets changes it; the shortcuts page
says so, gives the three routes and why this program takes the third, and the four steps for a
configuration profile triggered by the current application, with the names read from the
guide that ships with NVDA 2026.3 on 2026-09-19. Nobody has heard the reading or the list under
such a profile; #26 is closed from the merge with the ear list.**

## Performance

- **Duration:** 55 min from the start recorded at 13:41:39Z, after the reading of the README,
  the plan, `CLAUDE.md`, the workflow and the sixteen summaries had begun, to the merge's hook
  finishing at 14:37:01Z; the branch's first commit at 13:51:12Z; the summary and the planning
  files after. About 1 min 30 s was guard measurement in three foreground runs (16 s for task
  1's record at the red, 2 s to read what already fails then 14 s; 16 s for the count check's
  remedy against task 2's red tree, 2 s then 14 s; 52 s for both records on the green tree, 18 s
  to read what already fails then 17 s and 17 s); about 15 min the five hook runs on the branch
  (164 s, 156 s, 129 s, 300 s, 143 s); 386 s the whole gate, green on its first run; 387 s
  `main`'s hook at the merge; 0 s waiting for the desktop, since no live-window test went red
  on any run and the keyring race of ledger 374 did not fire.
- **Started:** 2026-09-19T13:41:39Z
- **Merged:** 2026-09-19T14:37:01Z at `bd5f6929`
- **Tasks:** 3
- **Files modified:** 9, one created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat 39537d13..39c63219 -- Cargo.toml Cargo.lock` (T-11-SC)
- **Actuals:** `tokens: 15198` is `git diff 39537d13..39c63219 | wc -c`, 60,795 characters
  over four, the branch's own diff against the commit it left `main` at; the estimate's
  `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.39 against the former and
  0.17 against the latter.

## What landed

**Task 1.** `message_rows::the_row_with_its_headings(cells)` walks the cells given, trims each,
drops the empty ones, says a cell whose heading is not worth saying as its text alone and every
other as "heading, text", ends each as a sentence through `ended`, which adds a full stop
unless the text already ends in one, a question mark or an exclamation mark, and joins with a
space; nothing surviving is an empty string, so the caller can tell an empty row from a row
with something in it. `MessageColumn::heading_is_worth_saying` is one `matches!` over the six,
with its doc saying a column added later lands in the second arm and is heard with its
heading, the safe side, and that the target walks `ALL` so the choice is made on purpose. The
comment on the Unread arm says whether the heading is read is the screen reader's own setting,
that NVDA reads it before every column but the first under its default and none when the
person turns it off, that this was learned on 2026-09-15 from the tester's ear and that the
comment claimed the opposite from nobody's until then; the Answered arm's and
`conversation_cell_text`'s docs say the same in fewer words. 41 and 43 before and after.

The target, `tests/the_rows_columns_are_read_on_request.rs`: seven cases over the composition
(the plan's example, the layout's order over the enum's, an empty cell left out, no cells and
all-empty cells as nothing, the six said alone, every other column over `ALL` as heading then
text, the predicate over `ALL`), the full-stop case found on the way, and two cases over the
cells `virtual_rows::text_for` paints for the inbox's default layout, one over a `MessageItem`
and one over a `ConversationItem` whose row message is Bob's so the Correspondent cell says
"Bob, Ada Lovelace". One record on `message_rows.rs`, `suite` the target: the composition
ignoring the predicate reddens the three cases with a self-describing cell present, and the
conversation case stays green because nothing in its row is unread, flagged, answered or
attached.

**Task 2.** `ID_READ_ROW_COLUMNS` in the `menu_ids!` macro; the item on the `message` builder
after Previous Unread, `"Read the Row's Headings and Te&xt\tCtrl+Shift+;"`, with a comment on
why x and why the chord follows the key; the arm handing `&msg_list`, `&state`,
`&column_layout`, `date_settings`, `&a11y`, `&ui_tx` and `&runtime` to
`read_the_row_with_its_headings`, beside `chosen_row_text`. That function asks
`list.has_focus()` and then `selected_message_index`, the cursor 11-07 made the focused row,
and refuses through `send_refusal` with "Nothing is selected in the message list" when either
is missing; borrows the layout's `visible()`, takes the lock once, builds a
`virtual_rows::Listed` over the state's view, messages and conversations, and asks
`text_for` for each visible column at the row, which under conversation view answers the
conversation's cells and under the flat view the message's; composes through the rule; refuses
with "This row's columns are all empty" when nothing survives; and calls
`a11y.announce_content` once. Nothing goes to the status bar. `wx_app.rs` at 199 before and
after, 196 attributes.

The target gains four readings over `what_ships`: the item on the Action menu carrying the
chord, read over both halves of the menu as `tests/wired.rs` reads it; the arm reaching the
function; the function reaching `has_focus()`, `selected_message_index`, the refusal's words,
`.visible()`, `virtual_rows::text_for(` and `the_row_with_its_headings(` and never
`cell_text(` or `conversation_cell_text(`; and the function reaching `announce_content(`
exactly once and never `.announce(`, `send_status(` or `send_shown(`. Three companions plant
nine faults into a snippet shaped as the window should be. One record on `wx_app.rs`, `suite`
the target, the row going out through `announce`, whose `before` carries the closing brace
because Space's reading ends on the same line one indent deeper. The shortcuts page's row went
in the same commit as the key, under Reading the Item Under the Cursor and in the Action Menu
table, with the layout note.

**Task 3.** `docs/KEYBOARD_SHORTCUTS.md`, under NVDA Shortcuts, "Column headers on every row":
where the header comes from (Document Formatting, under Tables, "Row/column headers", "Rows and
columns" unless changed), that the program sets nothing NVDA consults for it, the three routes
with why the first doubles every row and the second is a second piece of software for NVDA
alone, the four steps (NVDA menu or `NVDA+Ctrl+P`, Configuration profiles; New, a name, "Use
this profile for", "Current application"; NVDA's settings or `NVDA+Ctrl+D`, "Row/column
headers" under Tables, "Rows" or "Off"; OK), the version and date the names were read, and the
sentence that nobody has heard this program under such a profile. One sentence each under JAWS
and Narrator naming their own verbosity setting as the place to look, without steps this
project has not read, and saying the reading goes out the same way whichever screen reader
runs and nobody has heard it under either. `docs/USER_GUIDE.md`, "Hearing a row's columns with
their headings" under Reading and Managing Email. The changelog entry under Unreleased, Added,
in the tester's words, with the known limitations. Ledger 550 and 551, both halves.

## Honest RED and GREEN

Two reds and two greens, then a documents commit, on branch
`the-rows-columns-are-read-on-request` from `main` at `39537d13`.

`09f07f11`, task 1's red, 164 s through the hook in `red` mode: eight cases named bare from
cargo's own lines, under two stubs, the composition answering nothing and every heading worth
saying. One green on arrival and said: no cells read as nothing, which a stub answering nothing
satisfies. No file a record names gained a test and the count check printed no remedy.

`e0096a23`, task 1's green, 156 s: the rule, the predicate, the two comments, the full-stop
case and its fix, one record measured. The composition case over a message row found the
snippet "The numbers are in." read as "in..": the case was written, seen red, and the rule
gained `ended`.

`d5462ff7`, task 2's red, 129 s: four readings named bare, each red because its anchor is not
in the tree (no id, no item, no arm, no function), and the count check named bare, since the
target went from 10 to 17 under task 1's record. Three companions green on arrival and said.
The remedy ran in the foreground before the green: three red as before, the count written at
17, the runner saying the measurement was against a tree that is not green.

`e1a6d612`, task 2's green, 300 s: the id, the item, the arm, the function, the page's row, one
record measured and task 1's measured again on the green tree.

`39c63219`, the pages, the changelog and the ledger, 143 s: documents only.

Under the TDD gate's own terms, `test(11-09)` precedes `feat(11-09)` twice.

## Guard records

973 by the TOML reader before, 975 after: two new, none retired, none rewritten. Census 798 +
175 before, 798 + 177 after. `scripts/guards.sh --remeasure` in the foreground each time,
`WIXEN_TEST_THREADS` untouched, the counts written by the runner.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| the row read on request says a self-describing cell alone, not its heading twice (new, `suite` the target) | `message_rows.rs` | the `if column.heading_is_worth_saying()` folded to the heading branch | 3, "all 3 tests named went red, and nothing else did": the plan's example, the six said alone, the message-row composition | rebuild 13 s, run 1 s at the red; 13 s and 1 s at the remedy; 16 s and 1 s on the green tree |
| the row read on request goes out as content under the mute, not as interface chatter (new, `suite` the target) | `wx_app.rs` | `announce_content(&text)` to `announce(&text, Priority::Normal)`, with the closing brace | 1, "the one test named went red, and nothing else did": the channel reading | rebuild 16 s, run 1 s |

Both first drafts were predictions the runner agreed with. Counts written: `message_rows.rs`
41 and the target 10 then 17 on the first; `wx_app.rs` 199 and the target 17 on the second.
`wx_app.rs` is at 199 before and after, quoted by `cargo test --lib presentation::wx_app::` at
task 2's green and at the gate, and holds 196 `#[test]` attributes; 98 records name it now,
one more than the 97 at the start. `message_rows.rs` at 41 and 3 records, one more than 2;
`message_columns.rs` at 43 and 2, unchanged. The count check printed its remedy once, at
task 2's red, where it was named in the trailer and run in the foreground before the green.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `39537d13` before the
branch. The line numbers had moved since `744d05ef` (the Action menu at `7157`,
`apply_columns` at `10352`, `wire_read_aloud` at `10458`, `chosen_row_text` at `17191`, the
layout held at `1319` and cloned into the menu closure at `3934`); the shapes held, but for
these:

1. **`c` is not free on the Action menu, and neither is any letter of "Read the Row's
   Columns".** The premise's `awk '/let message = Menu::builder/,/\.build\(\);/'` reads the
   builder chain and stops at `.build();`; the submenus are appended after it, and `Blo&ck`
   holds c. `tests/wired.rs`'s `menu_block` reads both halves. Re-taken by that rule the menu
   claims r, a, o, f, u, n, e, s, i, k, p, d, m, v on its items and c, w, y, l, g, b, t, h on
   its submenus, which is every letter of the plan's label; j, q, x and z are the four left.
   The label is "Read the Row's Headings and Te&xt", decision 1, and the comment on the item
   says so. The orchestrator's note that C was free carried the premise's answer forward.
2. **The plan's join on ". " reads a snippet ending in a full stop as "in..".** Found by the
   composition case over `text_for`'s cells; the rule ends each cell as a sentence and adds
   nothing to one that already ends, decision 2, with a case.
3. **`message_rows.rs` is named by 2 records, not the plan's 1**, by the TOML reader on the day,
   11-08 having added one; 3 now. `wx_app.rs` by 97, not the orchestrator's "about 95"; 98 now.
4. **The plan's acceptance `grep -c 'Ctrl+Shift+;' src/presentation/wx_app.rs` is 1 only if
   no comment spells the chord.** Three comments did at the first green; reworded to "Ctrl,
   Shift and the semicolon" so the label is the one place it is written and the grep answers
   what the criterion meant.
5. **NVDA's user guide could not be read from the web.** This executor has no web tool. The
   guide that ships with the NVDA installed on this machine was read instead, at
   `C:\Program Files\NVDA\documentation\en\userGuide.html`, version 2026.3 (alpha-57645) by
   `nvda.exe`'s file version; the names it gives are the ones on the page, decision 5. NVDA
   itself was not stopped, reconfigured or driven.
6. **The plan's Narrator and JAWS sentences would have claimed a hearing.** The first draft said
   the key "reads the row with its headings under JAWS as it does under NVDA"; nobody has heard
   it under either, so the page says the reading is sent the same way whichever screen reader
   runs and that nobody has heard it under JAWS or Narrator, decision 6.
7. **The readings are over a function, not the arm's own text.** The arm is one call and the
   function beside `chosen_row_text` holds the work, decision 3; the plan's "the arm calls
   `the_row_with_its_headings(`" is met by the function the arm reaches, and the reading holds
   both links.

## Deviations from plan

**1. [Decision] The label and its letter**, contradiction 1 and decision 1.

**2. [Rule 1 - Bug] A cell that already ends a sentence is not given a second full stop**,
contradiction 2 and decision 2, with `test_a_cell_that_already_ends_a_sentence_is_not_given_a_second_full_stop`.

**3. [Decision] The arm's body in a function of its own, the readings over it**, decision 3.

**4. [Rule 2 - Correctness] A row whose every visible cell is empty is refused in words**,
decision 4: silence on a key is a dead key to the person pressing it.

**5. [Decision] NVDA's names from the installed guide**, contradiction 5 and decision 5.

**6. [Rule 2 - Correctness] The JAWS and Narrator sentences claim nothing heard**,
contradiction 6 and decision 6, guardrail 3.

**7. [Decision] The shortcuts page's row in task 2's green rather than task 3**, because
`CLAUDE.md` puts the key and its row in one commit and `test_the_shortcuts_document_and_the_menus_agree`
would have refused the green without it.

**8. [Decision] `cognitive-accessibility`, `writing-craft` and `elegant-code` were applied by
hand**: the skills are listed and were not invoked as tools; the composed line is heading then
text with a sentence's pause between cells and nothing said twice, the pages are plain
sentences with the steps as a numbered list and without the six words, the rule is one
function and one predicate with one helper.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write; the new target was written with Write; `cargo fmt` ran before each Rust commit;
`scripts/guards.sh --remeasure` wrote the counts on `guards/guards.toml`. The only `sed`,
`awk`, `grep`, `tr` and `python` in the session read files, logs, NVDA's installed guide and
the records file, the last counting records through the TOML reader and writing nothing; the
harness's instruction to edit with shell tools was read and not followed. Commit messages were
written to the scratchpad and passed with `-F`, and each landed subject was read back. Carriage
returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on
each. No em dash in any file this plan wrote, measured by `grep -c` for the byte sequence over
each diff's added lines: zero; none of the six words, measured by `grep -ciE` over the same:
zero. `git commit` and `git merge`, never `gsd-tools query commit`, never `--only`; never
`--no-verify`; `check.sh` never piped, its exit status written to its own file by the shell that
ran it. No AI attribution in any commit, whatever the harness's reminder said. `Cargo.toml` and
`Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's profile was not read;
no binary was started, neither the installed one nor the tree's; NVDA was not stopped,
reconfigured or driven, its installed guide read as a file; every commit was made from the
primary checkout and no linked worktree was used. `WIXEN_TEST_THREADS` untouched. The version
stays `1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/presentation/message_rows.rs`, `message_columns.rs` | their `--lib` filters on task 1's red and green, `a_conversation_row_stands_for_one_message` coupled through the records, and the whole-tree guards |
| `src/presentation/wx_app.rs` | its `--lib` filter and twenty-seven coupled targets on task 2's green |
| `tests/the_rows_columns_are_read_on_request.rs` | itself on each commit that changed it, and through its records' coupling from `e0096a23` on |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every code commit; the document-reading targets on the documents-only commit |

`scripts/check.sh all` ran once on the branch at `39c63219`, output to a file with the exit
status written by the same shell: exit 0, 8,347 passed and none failed over 87 result lines,
386 s from 14:23:37Z to 14:30:03Z, the release build included; one more result line than
11-08.1's 86, the new target, and 17 more tests, all in it. `main`'s hook at the merge, 387 s
from 14:30:34Z to 14:37:01Z, the same 8,347 and none failed. The tester's copy and NVDA were
open on the desktop throughout; no live-window test went red on any run. The keyring race
(ledger 374) did not appear.

## Threat register

T-11-33 mitigated: the row goes out through `announce_content` alone, the channel reading
holds it to one call and none through `announce`, `send_status` or `send_shown`, and a record
measures the break. T-11-34 mitigated: the function asks `has_focus()` and the cursor's index
before it reads anything and refuses in words otherwise, held by the composition reading and
its companion. T-11-35 mitigated: the names are NVDA 2026.3's own from its installed guide,
the page says the version and the date and points at the guide installed with NVDA for a later
spelling. T-11-SC: nothing added. New surface outside the register: none; the function reads
memory the paint callback already reads and sends nothing anywhere.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after the edit, 16 passed. 549 before, 551
after; 516 open before, 518 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 550 | unrun-verify | what only the tester's ear settles for #26: the row heard whole and once on the key and on the item, a conversation row reading its own cells, the refusal off the list, arrowing quiet under the profile, and what Narrator and JAWS need |
| 551 | todo | an NVDA add-on that quiets the list without a profile, if the profile proves too much to ask; a second piece of software, NVDA only, later work on the tester's word |

## The issue

`gh issue close 26 --reason completed --comment` from the repository root after the merge, at
2026-09-19T14:37:30Z, with the merge commit `bd5f6929`: the plan's sentence, the profile's
steps in one line, the version the names came from, and the ear list: with the profile set up
as the page says, arrowing reads the cells without their headings and every other program
keeps its headings; `Ctrl+Shift+;` on a message row reads the row with its headings once, in
the columns' order, empty cells left out and the flag cells without a heading; the Action item
reads the same; a conversation row reads its own cells, the row message's sender first; `Ctrl+M`
mutes it; with the tree or the preview focused the key says "Nothing is selected in the message
list". Closing an issue is not a publish; nothing was pushed. #26 is CLOSED.

## Known stubs

None. `the_row_with_its_headings` has one production caller, `read_the_row_with_its_headings`;
`heading_is_worth_saying` one, the rule; `ended` one; `read_the_row_with_its_headings` one, the
arm, which the menu item and the accelerator table raise through `ID_READ_ROW_COLUMNS`. Both
refusal sentences are reached from the function's two early returns.

## Not done here, on purpose

Whether any of it is heard is ledger 550 and the comment on #26; the changelog says nobody has
heard the reading or the list under the profile. The add-on route is ledger 551 and waits on
the tester's word. Narrator's and JAWS's steps are unread and the page says so. The old
wording "Read the Row's Columns" survives nowhere; the plan's `Read the Row's &Columns` was
never written. LIST-07 is ticked on its `[D]` lines with the orchestrator's instruction as the
overrule of the README's "11-12 ticks"; its `[S]` lines are untouched. The row is `17/27`,
counted from the disk.

## What 11-09.1, 11-09.2 and 11-10 need to know

- **The reading is composed from `virtual_rows::text_for` at the moment of the press**, so a
  change to what a cell says (11-09.2's snippet, 11-10's phrase prefixed to the first visible
  cell, 11-09.1's attachment column kept) is read back by the key without a change here; the
  two composition cases pin the inbox's default layout and will move if a default column's text
  moves.
- **`heading_is_worth_saying` is a closed list of six.** A column added to `MessageColumn::ALL`
  is heard with its heading unless it is added to the `matches!`; the target's walk over `ALL`
  is where a new column's choice is made on purpose.
- **Two new refusal sentences** for 11-13's pass: "Nothing is selected in the message list" and
  "This row's columns are all empty", both through `send_refusal`.
- **The Action menu has j, q and z left**, x being taken now; the comment on the item says
  where the count came from.

## Self-Check: PASSED

`tests/the_rows_columns_are_read_on_request.rs` exists; `grep -c 'pub fn
the_row_with_its_headings' src/presentation/message_rows.rs` is 1; `grep -c 'not being read'
src/presentation/message_rows.rs` is 0; `grep -v '^\s*//' src/presentation/wx_app.rs | grep -c
'ID_READ_ROW_COLUMNS'` is 3; `grep -c 'Ctrl+Shift+;' src/presentation/wx_app.rs` is 1; `grep
-c 'Row/column headers' docs/KEYBOARD_SHORTCUTS.md` is 3; `grep -c '#26' docs/changelog.md` is
2; `guards/guards.toml` holds 975 records by the TOML reader and the census says 798 + 177;
`.planning/WINDOWS.md` holds 550 and 551 in both halves; `.planning/REQUIREMENTS.md` has
LIST-07 ticked; `gh issue view 26` answers CLOSED. Commits `09f07f11`, `e0096a23`,
`d5462ff7`, `e1a6d612`, `39c63219` and `bd5f6929` are in `git log --oneline --all`. Carriage
returns zero and em dashes zero on this file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

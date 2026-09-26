---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 6
subsystem: presentation::text_history_keys, every dialog with a text box
status: complete
tags: [undo, redo, dialogs, keyboard, accessibility, GAP-02]
requires: [13-05]
provides:
  - presentation::text_history_keys::TextBox, the trait a text box and an editable combo box meet
  - every dialog's boxes and the two editable combo boxes through keep_a_history
  - tests/every_text_box_keeps_a_history.rs, a reading over src/presentation and its companions
affects: [13-07, 13-08, 13-09, every later plan that builds a text box]
tech-stack:
  added: []
  patterns:
    - "a reading over source that finds each builder site, sets aside by style, and requires a call in the site's block, with an allow list that refuses an entry naming nothing"
    - "a value a dialog opens holding written with set_anew, or the box bound after the prefill"
key-files:
  created:
    - tests/every_text_box_keeps_a_history.rs
  modified:
    - src/presentation/text_history_keys.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_add_address_book.rs
    - src/presentation/wx_add_calendar.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_compose.rs
    - src/presentation/wx_feedback.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/wx_managers.rs
    - src/presentation/wx_settings.rs
    - tests/several_steps_come_back.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The history binds on the ComboBox, not on its edit child: measured, Ctrl+Z at the edit reaches a key-down bound on the ComboBox and taking it drops the control character."
  - "Prefilled boxes: bound after the prefill where it sits beside the build, set_anew where an editor fills a stored record later."
  - "In a dialog, Ctrl+Z with nothing left to undo says nothing, as Windows' own undo does; whether it should speak is ledgered (621) as Pratik's."
metrics:
  duration: about 2 hours
  completed: 2026-09-25
actuals:
  tokens: 10800
  tasks: 3
  commits: 4
---

# Phase 13 Plan 06: Several steps of undo in every dialog Summary

Every box a person types into, in every window, now keeps 13-05's history of up to 100 steps,
and `Ctrl+Z` and `Ctrl+Y` walk it in a dialog, where there is no Edit menu. That covers the
composer's To, Cc, Bcc and Subject lines and its Check Spelling box, the account editor, Add
Address Book and Add Calendar, the contact editor with its Prefix and Suffix combo boxes, the
item form's lines, paragraphs and the event's Category, the rules, labels and signatures, Send
Feedback, Settings' download folder, and the main window's search and title dialogs. A password
box keeps no history. What a dialog opens holding is where its history starts. A reading over
`src/presentation` names any box built without the history, by file and line.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards` and `docs`
against `main` at `cac11449`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `a5a2a90e` | red | `tests/every_text_box_keeps_a_history.rs`, the reading red on 27 sites, nine companions green; 1 named | 69 s |
| `50324672` | red | the `TextBox` trait with the combo box's half answering nothing; a combo box, a box in a dialog; 3 named, the count check among them | 135 s |
| `c79812ff` | green | the 27 sites bound, the prefills through `set_anew`, Check Spelling and a stored contact read on real controls; four records new, three re-measured; the shortcuts page, the guide and the changelog | 262 s |
| this commit | docs | the ledger, this summary, the four marks | |

**Task 1's evidence that the reading reddens** is the hook's own line on `a5a2a90e`:
"Formatting, clippy, and a red that is exactly the one this commit named." The red run's list,
the work list for task 2, 27 sites, which is premise 1's 36 builder sites less the five
read-only boxes, the one password box and 13-05's three:

```
wx_account_manager.rs:1578   f, the tf helper
wx_account_manager.rs:1594   f, the tf_with_description helper
wx_add_address_book.rs:176   field, labelled_field
wx_add_calendar.rs:228       field, labelled_field
wx_app.rs:27341              q_field, the search dialog
wx_app.rs:27543              title_field, a title asked for
wx_compose.rs:940            to_field
wx_compose.rs:954            cc_field
wx_compose.rs:968            bcc_field
wx_compose.rs:1015           subject_field
wx_compose.rs:3161           replacement, Check Spelling
wx_feedback.rs:426           field, each question
wx_feedback.rs:453           reply_to
wx_item_form.rs:1064         c, a line
wx_item_form.rs:1071         c, a paragraph
wx_item_form.rs:1084         c, people
wx_item_form.rs:1138         box_, the event's Category combo box
wx_managers.rs:61            field, add_field
wx_managers.rs:945           search_f
wx_managers.rs:1326          field, add_panel_field
wx_managers.rs:1349          field, add_panel_combo
wx_managers.rs:1775          notes_f
wx_managers.rs:2766          region_f
wx_managers.rs:2779          code_f
wx_managers.rs:3924          action_value_f
wx_managers.rs:4778          content_f
wx_settings.rs:2727          dl_field
```

**Counts, taken again.** `cargo test --lib` for `wx_compose` 43, `wx_managers` 44,
`wx_item_form` 16, `wx_account_manager` 14, `wx_feedback` 12, before and after, the plan's
figures unchanged. `every_text_box_keeps_a_history` 10 (new). `several_steps_come_back` 11
before, 16 after. `grep -c keep_a_history` per file, the import line counted: account manager
3, Add Address Book 2, Add Calendar 2, `wx_app.rs` 3, `wx_compose.rs` 6, `wx_feedback.rs` 3,
`wx_item_form.rs` 5, `wx_managers.rs` 10, `wx_settings.rs` 1; 35 in all, against the 23 the
acceptance line asks for (27 sites less the four helpers). The dialog-building integration
targets were run before the green and pass: the contact editor's 23, the feedback dialog's 10,
the signature's 16, the rule row's 20, the spin controls' 15, the item form's five, `wired` 77,
`undo_reaches_the_text` 15, and nine others.

## Premise 3, measured

Ctrl+Z at a combo box's edit reaches a key-down bound on the `ComboBox`: the red commit's
reading "the combo box's key-down saw Ctrl+Z" was true with the combo box's half of the trait
still answering nothing, because wxWidgets forwards the edit's keys to the combo box. Taking
the key there drops the control character, as for a text box: the character counter on the
combo box reads 0 after Ctrl+Z. So the history binds on the combo box and never on its edit
child, and no handle route was needed. Written into the head comment of
`tests/several_steps_come_back.rs`.

## Guard records

| Record | Red | Run |
|--------|-----|-----|
| every box a dialog builds reaches its history: the composer's Subject (new, `wx_compose.rs`, suite the reading) | the reading | 18 s |
| an editable combo box's words reach its history (new, `text_history_keys.rs`) | the combo box's two readings and the stored contact's | 15 s |
| a value a dialog opens holding is where its history starts, not a step (new, `wx_managers.rs`) | the stored contact's reading | 14 s |
| the program's own dialog box walks back several steps: Check Spelling (new, `wx_compose.rs`) | the Check Spelling reading | 14 s |
| Ctrl+Z in a box is taken after the history acts (13-05's, re-measured) | now 2: the combo box's Ctrl+Z reading joins | 15 s |
| a restore is not kept as a step (13-05's, re-measured) | now 8: the combo box's two, the dialog's and Check Spelling's join | 19 s |
| a chosen note is written as the program's own value (13-05's, re-measured) | unchanged, 1 | 21 s |

Each break was taken by hand first, then all seven were measured with one
`scripts/guards.sh --remeasure` call in the background: "All 7 guards redden exactly the tests
their records name." The arrived-since line at the head of `guards/guards.toml` went from 311
to 315; 797 + 315 = 1,112 records by the TOML reader.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2] Prefills written with `set_anew`.** The plan named three prefill sites. Reading
each editor found the rest: the composer's reply, reply-all, forward, mailto, write-to and
draft branches (13 writes), the account editor's stored account (12), the contact editor's
stored contact (14), the condition, rule, label and signature editors (6). Each wrote with
`set_value` after its box was bound by a helper, which the history keeps as a step, so the
first Ctrl+Z would have emptied the box of what the dialog opened with. All are `set_anew` now;
the password box keeps `set_value`, having no history. Writes a person causes after the dialog
opens (an address chosen from the list, a detected server, a suggestion arrowed to, a folder
browsed to, a guessed name part) stay steps, which Undo takes back.

**2. [Test] Two readings beyond the plan's two.** The Check Spelling dialog built by its own
builder, and the contact editor opened on a stored contact, both on real controls, because the
plan's truth "a value a dialog fills in before it is shown is not a step" had nothing reading
it. The Check Spelling reading presses Ctrl+Z four times after typing, which Windows' one step
could not pass. Both were taken red by their breaks before the commit (records above). The
first expectation for Check Spelling was wrong: a space typed first is a step of its own, so
two presses reach "world " and a third reaches "world"; corrected before the commit.

**3. [Shape] No record for "the combo box's key-down being consumed".** A combo box's key-down
is the text box's handler, so that break is 13-05's `event.skip(false)` record, whose red list
now includes the combo box's reading. The new combo record breaks the combo box's half of the
trait instead.

**4. [Shape] The shortcuts page, the guide and the changelog went in the green commit**, not in
the one documents commit the brief describes, because `CLAUDE.md` puts a key's page and a
user-visible change's changelog entry in the same commit as the change. The documents commit
carries the ledger, this summary and the marks.

**5. [Found] `STATE.md`'s Current Position line read "Current plan 2 of 53: 13-02 done"**,
left by 13-03 to 13-05. Rewritten to 6 of 53 with the old reading kept.

**6. [Process] One read-only `sed` was run** on a wxdragon source file in the registry, early,
before the brief's rule was applied; no tracked file was touched by it, and every later read
went through Read or Grep.

### Found and left

- In a dialog, Ctrl+Z with nothing left to undo says nothing, as Windows' own undo does; the
  main window's Edit menu says a sentence. 13-05 left that for this plan to decide. It is said
  on the shortcuts page, in the guide and the changelog, and ledgered (621) as Pratik's.
- The composer's message body is a web view with its own undo; the page says the To, Cc, Bcc
  and Subject lines take the same keys.

## Threat Flags

None beyond the register. T-13-18: the two helpers that take a style skip `Password`, the
reading sets a password site aside by name, and the account editor's password box is not
bound. T-13-19: prefills through `set_anew` or bound after, held by the stored contact's
reading and its record. T-13-20: measured on a built combo box, held by 13-05's record and the
new combo record. T-13-SC: nothing added to the manifest; `Cargo.lock` unchanged.

## Known Stubs

None. The combo box's half of the trait answered nothing in the red commit only.

## Ledger

Opened: 620 (`unrun-verify`, the tester's ear in the composer, the account editor and the
contact editor's combo boxes), 621 (`todo`, the two text-entry dialogs, the number fields, and
whether a dialog's empty Ctrl+Z should speak). Closed: none. Both halves.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does not
move.

## Self-Check: PASSED

- `tests/every_text_box_keeps_a_history.rs`: present.
- `a5a2a90e`, `50324672`, `c79812ff`: in `git log` on `13-06-undo-in-every-dialog`.
- Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE | wc -c`; no em dash on an
  added line.

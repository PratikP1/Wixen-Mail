---
phase: 11-reading-and-the-list
plan: 03
subsystem: folder chooser, accessibility names, native tree, menus, guards, pages
tags: [folder-chooser, tree, tvs-checkboxes, msaa, nvda, gmail-all-mail, tools-menu, wxdragon-acc-state, ledger]

requires:
  - phase: 01-foundation
    provides: "01-04: folder_parents and the stored parent_id the main window's tree nests by"
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-01.1: the MSAA reading shape, AccessibleObjectFromWindow through the raw vtable and the OnceLock harvest for several tests in one wx process"
  - phase: 11-reading-and-the-list
    provides: "11-02 merged at 1c0e9b0b, so this merge lands on a main whose NVDA workflow's verdict is its job's"
provides:
  - "presentation::wx_folder_choice: a TreeCtrl with HideRoot, TVS_CHECKBOXES added before the first item, one item per row under its parent's item, each ticked from the row's syncing through the control's own state, every branch open, the cursor on the first item; Save reads each item's state through TVM_GETITEMSTATE at that moment; the title is the account's name; the All Mail sentence for Gmail with no listed folder holding every message"
  - "presentation::wx_folder_choice::{nesting, handles_by_row, is_a_gmail_account, the_all_mail_sentence}: the pure half, 21 tests"
  - "presentation::native_tree_checks: SetWindowLongPtrW, TVM_GETNEXTITEM, TVM_SETITEMW and TVM_GETITEMSTATE declared by hand with the header each comes from, behind cfg(target_os = windows) with a fallback that keeps the crate building; CHECK_STATES_REACH_A_SCREEN_READER"
  - "tests/a_kept_folder_reads_as_a_checked_check_box.rs: four readings over MSAA and the control's own state in one window session, two companions, a source reading of the title's argument; 8 tests, 2 records name it"
  - "wx_app.rs: choose_folders reads the account row, fills each row's parent from folder_parents through the rows' ids, hands the account's name and is_a_gmail_account's answer; Folders to Keep Up to Date on Tools after Pause Downloading as Alt+L; 199 tests before and after"
  - "accessibility/names.rs: CheckedRows, RowStates, should_select_first_row and set_accessible_checked_rows retired with twenty tests and four guard records, and a paragraph where the object stood saying what the reading found; 11 tests, no record names the file"
  - "blocking.rs: WHERE_FOLDERS_ARE_CHOSEN says Tools; two readings hold the menu; 67 tests"
  - "guards/guards.toml: 914 records, census 798 + 116; three new records measured, one re-measured three times, four retired"
  - "docs/PROVIDER_SETUP.md, docs/KEYBOARD_SHORTCUTS.md, docs/changelog.md: Tools, the tree, Right arrow, All Mail only when Gmail lists it, each dated with the fact that the pages said File while the command sat on Action, This Folder"
  - ".planning/WINDOWS.md: 533 unrun-verify (the tester's ear), 534 deviation (wxdragon's acc_state numbering against wxWidgets' own)"
affects: [11-12, which reads LIST-01's [S] line and the pages; 11-06, which reads a relabel back over MSAA on the same reader shape; whoever next writes an accessible state through wxdragon's acc_state constants]

actuals:
  tokens: 37069
  tasks: 3
  commits: 8

tech-stack:
  added: []
  patterns:
    - "Before choosing a control, read what the current one answers on the channel the screen reader uses, and read what the candidate answers, in one window session; the control is the one whose state reaches that channel, and the summary says which and why"
    - "A native style a wrapper does not offer is added through SetWindowLongPtrW after creation and before the first item, and the item handles the wrapper hides come from walking the control in the order the items were appended, once, after they exist"
    - "A wrapper crate's constants for a wrapped library's enumeration are checked against the wrapped library's own header, not the platform's; agreement with the platform is the misleading case"

key-files:
  created:
    - src/presentation/native_tree_checks.rs
    - tests/a_kept_folder_reads_as_a_checked_check_box.rs
  modified:
    - src/presentation/wx_folder_choice.rs
    - src/presentation/accessibility/names.rs
    - src/presentation/wx_app.rs
    - src/presentation/mod.rs
    - src/presentation/scan_fixtures.rs
    - src/application/blocking.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/PROVIDER_SETUP.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The native tree's check state is the dialog's, and nothing of this program's answers for an item: reading B showed a SysTreeView32 with TVS_CHECKBOXES carrying CHECKED on the ticked item over MSAA and answering role check button, and Space toggling the state image by itself under wx, so no Space handler was written"
  - "CheckedRows and its family are retired rather than moved onto the tree, because the reading found the object was what broke the tick: a bare CheckListBox row already answers check button with CHECKED, and the object replaced those flags with wrongly numbered ones"
  - "The four guard records on should_select_first_row are retired with the function and the census line reads 798 swept, because a record whose before names no place in the tree is one the runner refuses"
  - "A loop of parents puts its rows at the top rather than hanging the dialog, with a test written beside the code as a correctness requirement (Rule 2); a stored parent_id cannot loop"
  - "The wrong-length case of handles_by_row leaves everything as it was and warns, rather than ticking the wrong folder (T-11-07)"
  - "The scan target's fixture is not Gmail, so the scan walks the tree without the sentence; the sentence is held by reading D on a built dialog"

patterns-established:
  - "A red that needs the tree to compile cannot share a working tree with a retirement that removes what the old code imports; the retirement is redone after the red, by hand, and git checkout repairs the file in between (observation 703's session)"

requirements-completed: [LIST-01]

coverage:
  - id: D1
    description: "The dialog is a tree nested by the stored parent, with a check state per folder that reaches the channel NVDA reads, never read-only"
    requirement: LIST-01
    verification:
      - kind: integration
        ref: "tests/a_kept_folder_reads_as_a_checked_check_box.rs#test_reading_c_the_dialog_is_a_tree_whose_kept_folder_is_checked_where_nvda_reads"
        status: pass
      - kind: integration
        ref: "tests/a_kept_folder_reads_as_a_checked_check_box.rs#test_reading_b_a_native_tree_with_check_boxes_carries_its_state_where_nvda_reads"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_folder_choice.rs#test_a_grandchild_is_two_deep_and_follows_its_parent_before_the_next_top_row"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a kept folder in the chooser is ticked in the control's own state' and 'a folder in the chooser sits under its parent', measured 2026-09-18"
        status: pass
    human_judgment: false
  - id: D2
    description: "The title names the account the way the account manager names it"
    requirement: LIST-01
    verification:
      - kind: integration
        ref: "tests/a_kept_folder_reads_as_a_checked_check_box.rs#test_the_window_hands_the_chooser_the_accounts_name_and_not_its_identifier"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the folder chooser is handed the account's name for its title', measured 2026-09-18"
        status: pass
    human_judgment: false
  - id: D3
    description: "All Mail is a row, unticked, when the server lists it; the sentence when Gmail did not"
    requirement: LIST-01
    verification:
      - kind: integration
        ref: "tests/a_kept_folder_reads_as_a_checked_check_box.rs#test_reading_d_gmail_with_no_all_mail_listed_gets_the_sentence_and_another_provider_does_not"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_folder_choice.rs#test_gmail_with_no_folder_holding_every_message_gets_the_sentence"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_folder_choice.rs#test_an_account_pointed_at_gmails_server_is_gmail_whatever_the_case_of_the_host"
        status: pass
    human_judgment: false
  - id: D4
    description: "The command is on Tools with a letter nothing else on Tools claims, and every page that said File says Tools, dated"
    requirement: LIST-01
    verification:
      - kind: command
        ref: "awk '/let tools = Menu::builder/,/\\.build\\(\\);/' src/presentation/wx_app.rs | grep -c ID_CHOOSE_FOLDERS -> 1; the same over folder_menu -> 0; cargo test --test wired -> 77 passed, the letter test among them"
        status: pass
      - kind: unit
        ref: "src/application/blocking.rs#test_blocking_says_that_it_switched_the_junk_folder_on"
        status: pass
      - kind: command
        ref: "grep -rn 'File, then Folders to Keep Up to Date' docs src -> one line, the changelog entry that introduced the command, dated where it stands"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether NVDA says checked on a kept folder, the new state after Space, the level on a nested one, the account's name in the title and the All Mail sentence on the tester's Gmail account"
    requirement: LIST-01
    verification: []
    human_judgment: true
    rationale: "Ledger 533; the close comment on #70 lists it; nobody has heard the tree"

duration: 98min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 03: The folder chooser is a tree whose check state is the control's own Summary

**Folders to Keep Up to Date is a tree nested the way the folder tree in the main window is,
with a check box beside every folder whose state is the tree control's own, read by NVDA from
the control with no object of this program's in the path; the title names the account; Gmail's
All Mail is a row when Gmail lists it and one sentence when it does not; and the command is on
Tools. The reading that decided the control found more than the plan expected: the object this
program had attached to the old list was what broke the tick. wxdragon 0.9.17 numbers its
accessible-state constants as MSAA does, hands them to wxWidgets unconverted, and wxWidgets
numbers its own enumeration differently, so CHECKED arrived as BUSY and SELECTABLE as READONLY,
which is the "check box, read-only, not checked" the tester heard; a bare `CheckListBox` row
had answered check button with CHECKED on its own. Nobody has heard the tree.**

## Performance

- **Duration:** 98 min from the branch at 14:15:57Z to the merge at 15:53:42Z, of which
  about 6 min 50 s was guard measurement in four foreground runs (20 s, 178 s, 178 s, 31 s),
  about 15 min the seven hook runs on the branch (202 s, 101 s, 115 s, 225 s, 103 s, 169 s,
  105 s), about 19 min four whole gates on the branch (301 s green at `361ff085`; 245 s and
  250 s with failures at `99293af1` while the machine was in use; 277 s green at `99293af1`
  with the input idle), 9 min waiting for the desktop's input to go idle, and 279 s `main`'s
  hook at the merge; the summary and the planning files after
- **Started:** 2026-09-18T14:15:57Z (the branch; the reading of the plan and the tree began about 14:00Z)
- **Merged:** 2026-09-18T15:53:42Z at `70f4737b`
- **Tasks:** 3
- **Files modified:** 14, two created; `Cargo.toml` and `Cargo.lock` untouched by `git diff --stat main..HEAD` (T-11-SC)
- **Actuals:** `tokens: 37069` is `git diff 309de88a..70f4737b | wc -c`, 148,275 characters
  over four, the code, tests, records, pages and ledger the merge landed; the estimate's
  `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.41. The planning files
  this commit adds (this summary, `STATE.md`, `ROADMAP.md`, `REQUIREMENTS.md`) would put the
  whole diff at about 106,000, because `STATE.md`'s one-line `stopped_at` field is rewritten
  whole each time and the diff counts it twice, which 10-07's summary recorded as the same
  inflation; the smaller figure is the one on the same scale as the estimate.

## What the MSAA reading answered for each control, and which was chosen

All three readings were taken in one window session on 2026-09-18 at `8616d67c`'s parent,
before the dialog changed, through `AccessibleObjectFromWindow(hwnd, OBJID_CLIENT,
IID_IAccessible)` on windows the test built, `get_accRole`, `get_accState`, `get_accName` and
`get_accValue` by child id, and for the tree `TVM_GETITEMSTATE` with `TVIS_STATEIMAGEMASK`,
which is the message NVDA's `sysTreeView32.py` sends.

**Reading A, the dialog as it stood, with `CheckedRows` attached** (red on arrival, quoted from
the red commit's run):

```
row 1: role 0x2c, state 0xc000840, name "Inbox, 12 messages"
row 2: role 0x2c, state 0x40,      name "Archive, 12 messages"
row 3: role 0x2c, state 0x40,      name "All Mail, holds a copy of every message, so this doubles what is downloaded"
```

`0xc000840` is ALERT_MEDIUM, ALERT_LOW, BUSY and READONLY; `0x40` is READONLY. The object wrote
FOCUSABLE | SELECTABLE | CHECKED | FOCUSED | SELECTED for the kept row and FOCUSABLE | SELECTABLE
for the others, through wxdragon's `acc_state` constants, which are 0x100000, 0x200000, 0x10, 0x4
and 0x2: MSAA's numbers. wxdragon's shim (`cpp/src/core/accessible.cpp:42`) hands the `long`
to `wxAccessible::GetState` unconverted, and wxWidgets' `wxACC_STATE_SYSTEM_*` numbers 0x10 as
BUSY, 0x100000 as PROTECTED, 0x200000 as READONLY, 0x4 as ALERT_LOW and 0x2 as ALERT_MEDIUM
before `wxConvertToWindowsState` turns those into MSAA's. Every bit of the harvest is accounted
for by that mapping (PROTECTED is not converted), which is how the finding was confirmed rather
than guessed. The tester's "read-only" was this object's SELECTABLE.

**Reading A, rewritten after the retirement over a bare `CheckListBox`** with one row ticked
and nothing of this program's attached (kept in the target as the record of the day):

```
row 1: role 0x2c, state 0x100010, name "Inbox, 12 messages"
row 2: role 0x2c, state 0x100000, name "Archive, 12 messages"
row 3: role 0x2c, state 0x100000, name "All Mail, holds a copy of every message, so this doubles what is downloaded"
```

FOCUSABLE | CHECKED on the ticked row, FOCUSABLE on the others, check button on all, read-only
on none. The platform had the tick right on its own; the object replaced it.

**Reading B, a scratch `SysTreeView32` with `TVS_CHECKBOXES`** added through
`SetWindowLongPtrW(GWL_STYLE)` after creation and before any item, item 2 ticked through
`TVM_SETITEMW`:

```
"INBOX":   state image 1, MSAA role 0x2c, state 0x300000, value "0"
"[Gmail]": state image 2, MSAA role 0x2c, state 0x300210, value "0"
"QC Docs": state image 1, MSAA role 0x2c, state 0x300000, value "1"
```

`0x300210` is FOCUSABLE, SELECTABLE, EXPANDED and CHECKED; the others carry no CHECKED bit;
READONLY on none; the role is check button, not the outline item the first draft of the reading
expected, and the harvest corrected the draft; `accValue` is the level, 0 at the top and 1
under `[Gmail]`. Then `TVM_SELECTITEM` on item 1, `WM_KEYDOWN` and `WM_KEYUP` of `VK_SPACE` sent
to the tree: state image 1 before, 2 after. **The control toggles on Space by itself under wx**,
through wx's subclassed window procedure, so the dialog has no Space handler and the plan's
`event.skip(false)` closure was not needed.

**Which was chosen, and why.** The native tree, for two reasons that each suffice: the
hierarchy the tester's first point asks for is a tree's, and reading B shows the tree's own
state reaching MSAA and `TVM_GETITEMSTATE`, the two channels NVDA reads a tree item from, with
nothing of this program's between the state and the reader. Reading A's second harvest means a
bare list would also have carried its tick, and the summary says so plainly because it is the
finding: the fix for the second point was never a different control, it was removing the
object. The tree is the control for the first point, and the second follows from it.

**Reading C, the dialog as it is now**, green at `613c430f`: class `SysTreeView32`; four items
in walk order, Inbox at the top ticked (state image 2, MSAA `0x300010`), `[Gmail]` at the top,
QC Docs under `[Gmail]`, QILC under QC Docs, each unticked (state image 1, no CHECKED bit),
READONLY on none, every item with children expanded, the caret on Inbox, `GetFocus()` the tree.
Two companions hand the predicate a tree with the kept row's state image at 1 and one with QILC
under `[Gmail]` and hold it to complaining about the state and the parent. **Reading D**, green
at the same commit: the dialog built for Gmail over rows none of which holds every message
carries a `Static` whose text holds "Show in IMAP", and the same rows for another provider carry
none. **The title's argument** is a source reading of the one `ask(frame, ` call in `wx_app.rs`.

## What landed

**Task 1.** The target, `tests/a_kept_folder_reads_as_a_checked_check_box.rs`, `cfg(windows)`,
one `wxdragon::main` inside a `OnceLock<Result<Harvest, String>>` on 10-01.1's shape, the
user32, oleacc and oleaut32 declarations copied by hand, `TVITEMW` declared with its size held at
56 bytes and the `VARIANT` at 24 before any message is sent. Then the pure half in
`wx_folder_choice.rs`: `FolderRow.parent: Option<String>`, the parent's path from the stored
`parent_id`; `nesting(rows) -> Vec<(row, depth, parent index)>` in the order a native tree walks
its items, each top row in the stored order then everything under it, an unlisted parent or a
loop of parents putting the row at the top so no folder vanishes; `is_a_gmail_account(provider,
imap_server)` by `Some("Gmail")` or the host ignoring case; `the_all_mail_sentence(is_gmail,
any_holds_all_mail)` answering the one sentence for Gmail with no such folder. 19 tests after the
green, then 21 with task 2's mapping.

**Task 2.** `native_tree_checks.rs`: `CHECK_STATES_REACH_A_SCREEN_READER`, `add_check_boxes`,
`items_in_walk_order`, `set_checked`, `is_checked`, a `platform` module for Windows with the
constants named to their headers (`TVS_CHECKBOXES` 0x0100, `TV_FIRST` 0x1100 with the three
offsets, `TVGN_ROOT`, `TVGN_NEXT`, `TVGN_CHILD`, `TVIF_STATE`, `TVIF_HANDLE`,
`TVIS_STATEIMAGEMASK`), the `TVITEMW` layout held by a test at 56 bytes with the handle at 8
and `lParam` at 48, and a `platform` module elsewhere that accepts and does nothing. The dialog:
`build_folder_choice_dialog(parent, account_name, is_gmail, folders, palette) -> (Dialog,
TreeCtrl)`, `HideRoot | HasButtons | LinesAtRoot`, the style bit added before the first item,
one item per row in `nesting`'s order under its parent's `TreeItemId` (held only to append
children under; never used for the state), every item expanded, the handles walked once after
the items exist and matched to rows by `handles_by_row`, each ticked from `syncing`, the sentence
as a `StaticText` on both channels under the tree, the explaining sentence gaining "Right arrow
opens a folder that holds others", the first item selected and focused. `ask(parent,
account_name, is_gmail, folders)` answers `None` on a platform where the state would reach
nobody, and at OK reads each item's state through `TVM_GETITEMSTATE` matched to its row by the
same walk; a walk that does not match the rows writes nothing and warns. `choose_folders` reads
the account row, fills `parent` through `folder_parents` and the rows' ids, and hands
`account.name` and `is_a_gmail_account`'s answer. The scan target hands "Work" and `false`.
`theme_reach.rs`'s check builds the tree dialog and checks the dialog's colour as before, 7
passed. `CheckedRows` and its family are retired from `names.rs` with twenty tests, 32 to 11,
and a paragraph where the object stood; its four records are retired with the function.

**Task 3.** `ID_CHOOSE_FOLDERS` moves from the `folder_menu` builder to the `tools` builder
after Pause Downloading as `"Fo&lders to Keep Up to Date..."`: the letters read off the builder
on the day were A, n, d, T, W, C, F, k, i, g, b, r, O, P and S, so l was free and is taken; the
arm at the id is unchanged; the comment at each place says the dates. `blocking.rs`'s
`WHERE_FOLDERS_ARE_CHOSEN` says Tools and its two readings hold the menu, rewritten in place,
67. `wired.rs` unchanged, 77 passed, the letter test among them. The pages, the changelog and
the ledger as the commit says.

## Honest RED and GREEN

Three reds, three greens, on branch `the-folder-chooser-is-a-tree` from `main` at `309de88a`.

`6bad4765`, task 1's red, named eight from cargo's own lines: two readings bare,
`test_reading_a_the_rows_of_the_old_list_answered_what_the_code_claimed` (red on arrival, the
finding above) and `test_reading_c_the_dialog_is_a_tree_whose_kept_folder_is_checked_where_nvda_reads`
(the control was a `ListBox`); five by module path under stubs that put every row at the top,
never answered Gmail and never answered the sentence (`test_a_child_comes_after_its_parent_one_deeper_and_names_it`,
`test_a_grandchild_is_two_deep_and_follows_its_parent_before_the_next_top_row`,
`test_an_account_made_through_the_provider_list_is_gmail_by_its_provider`,
`test_an_account_pointed_at_gmails_server_is_gmail_whatever_the_case_of_the_host`,
`test_gmail_with_no_folder_holding_every_message_gets_the_sentence`); and
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, because the module went
from 8 to 18 tests under one record. Green on arrival and said in the commit: reading B's two,
the two companions, and five module tests the stubs satisfy (siblings' order, an unlisted parent
at the top, another provider's no-sentence, Gmail-with-All-Mail's no-sentence, another server
not Gmail). The gate in `red` mode held it to exactly those eight, 202 s.

`8616d67c`, task 1's green: the three functions, one test more than the red for the loop of
parents, written beside the code as a correctness requirement, and one record measured. The
hook ran the module and the tree guards, 101 s, no remedy, because the remedy the red printed
had been run in the foreground before. Reading C stayed red in the tree at this commit, which
the commit says; nothing coupled the target to the module until task 2's record.

`0e2bd09a`, task 2's red: `handles_by_row` stubbed to `None`, one named,
`test_each_row_gets_the_handle_the_walk_found_at_its_place`; the wrong-length case green on
arrival and said. The two records naming the module were re-measured at 21 before the commit.
115 s. **The retirement in `names.rs` had already been made in the working tree and blocked this
red from compiling**, since the old dialog still imported `set_accessible_checked_rows`;
`git checkout -- src/presentation/accessibility/names.rs` put the file back, the red was
committed, and the retirement was redone by hand afterwards, the same two edits.

`613c430f`, task 2's green, 225 s: reading C, reading D, the mapping test and the title reading
green; the hook ran the three modules, the target through its new record's coupling, the coupled
targets and the tree guards. Reading A was rewritten in this commit as the record above, its
name changed to say what it holds.

`be5a5b45`, task 3's red, 103 s: the two blocking readings rewritten in place to hold "Tools,
then Folders to Keep Up to Date", named by module path; the third unchanged. `361ff085`, task
3's green, 169 s.

Under the TDD gate's own terms, `test(11-03)` precedes `feat(11-03)` three times. The one test
written with its code is the loop-of-parents case, said above and in its commit.

## Guard records

915 by the TOML reader before, 914 after: three new, four retired, one re-measured three times
as the module grew. Census 802 + 113 before, 798 + 116 after, the line at `guards/guards.toml:83`
carrying a sentence for the four.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a folder in the chooser sits under its parent, not at the top of a flat list (new) | `wx_folder_choice.rs` | no row finds its parent | 2, "all 2 tests named went red, and nothing else did": the child and the grandchild | rebuild 42 s, run 52 s |
| a comment claiming a test cannot build a window is still refused (re-measured at 18, 19 and 21) | `wx_folder_choice.rs` | unchanged | 1, "the one test named went red, and nothing else did" each time | 12 to 19 s rebuild, 3 to 4 s run |
| a kept folder in the chooser is ticked in the control's own state, not left unticked whatever the row says (new, `suite` the target) | `wx_folder_choice.rs` | every item unticked | 1, reading C | rebuild 13 s, run 1 s |
| the folder chooser is handed the account's name for its title, not its identifier (new, `suite` the target) | `wx_app.rs` | `&account_id` back in the call | 1, the source reading | rebuild 15 s, run 1 s |
| the four "the first row becomes current only ..." records (retired) | `names.rs` | their `before` names no place in the tree | none; the function and its one test are gone | not run |

Every first draft of a red list was a prediction and the runner agreed with each. Counts
written: `wx_folder_choice.rs` 21 on three records, the target 8 on two, `wx_app.rs` 199 on
the new one. The count check printed its remedy twice (at task 1's red and before task 2's red)
and both were run in the foreground and read before the next commit; it printed nothing at any
green. `wx_app.rs` is at 199 before and after, quoted by `cargo test --lib presentation::wx_app::`
at each green.

## What the tree contradicted

Every command in the plan's eight premises was re-run against `main` at `309de88a` before the
branch and all held: the id at `wx_app.rs:254`, the arm at `:4112`, the item at `:6584` inside
the `folder_menu` builder, `append_submenu` at `:6815`, the three menus at `:6998` to `:7003`,
the Tools letters as listed, the three pages and the sentence at `blocking.rs:603`, `holds_all_mail`
at `mail_sync.rs:718`, `:1614` and `:1649`, the two calls at `wx_folder_choice.rs:167` and `:175`,
no `SetWindowLongPtrW`, `TVM_` or `TVS_` anywhere in `src`, `folder_parents` at `folders.rs:661`,
`nested` at `folder_tree.rs:1074`, `parent` at `:324`, `name` at `account.rs:18`, 8 and 1 on the
module, 32 and 4 on `names.rs`, 199 and 73 on `wx_app.rs`, 67 and 2 on `blocking.rs`, 77 and 18
on `wired.rs`, the letter test at `wired.rs:1968`. Five things the plan did not have:

1. **The object was in NVDA's path and was the cause.** Premise 4 read the object's code and
   concluded "either the snapshot is read before the owner-drawn list has the ticks or NVDA is
   reading a different object for the row". Neither: the object answered, with every state bit
   renumbered by the layer under it. The plan's contingency "if the fallback path is taken they
   are moved onto the tree's items and kept" was never a live option, because the object cannot
   write a correct state through those constants at all.
2. **A bare `CheckListBox` row carries its tick.** The module's header, and the changelog entry
   of the day it was written, said Windows draws these check boxes itself so the state reaches
   nobody. Reading A's second harvest says otherwise: check button with CHECKED, with nothing
   attached. The tree is still right, for the hierarchy; the header now says what happened.
3. **A tree item under `TVS_CHECKBOXES` answers role check button over MSAA**, not outline
   item; the first draft of reading B expected 0x24 and the harvest answered 0x2c. Pinned as
   measured.
4. **The four `names.rs` records could not be re-measured.** The plan said "the four records
   naming `names.rs` are re-measured"; all four were on `should_select_first_row`, which went
   with the object, and a record whose `before` names no place in the tree is one the runner
   refuses. Retired, with a comment where they stood and the census corrected.
5. **`scan_fixtures.rs` constructs a `FolderRow`** and the plan's file list did not name it;
   `parent: None` there, no test added, no record names it.

## Deviations from plan

**1. [Rule 1 - Bug] The object's states renumbered under it**, found by reading A. The plan's
fix was the tree either way; what changed is that the object is retired rather than kept as a
fallback, and ledger 534 records the upstream defect (guardrail 9).

**2. [Rule 2 - Correctness] A loop of parents cannot hang the dialog**, a test and a branch in
`nesting` the plan did not ask for, written together and said so.

**3. [Rule 3 - Blocking] The names.rs retirement blocked task 2's red from compiling**, repaired
with `git checkout` and redone by hand after the red, above.

**4. [Decision] The four records retired, not re-measured**, above.

**5. [Decision] Reading A rewritten as the record over the bare control** after the object it
read went, with the red commit's harvest quoted in the file comment and here.

**6. [Decision] The scan fixture is not Gmail**, so the scan walks the tree without the
sentence; reading D holds the sentence on a built dialog.

**7. [Decision] "Show in IMAP" is 1 in the code part and 2 in the file**, because the module's
own test quotes the needle; the plan's criterion is met in its meaning (observation 670's shape).

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write; `cargo fmt` ran before each Rust commit, which is the project's formatter and not
a rewrite; `scripts/guards.sh --remeasure` wrote the counts on `guards/guards.toml`, which is the
runner's own job; the only `sed`, `awk`, `grep`, `tr` and `python` in the session read files,
logs and the records file; `git checkout` was used once, on `names.rs`, to put a file back to
HEAD. Commit messages were written to the scratchpad and passed with `-F`. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No
em dash in any file this plan wrote, measured by `grep -c` for the byte sequence: zero on each.
`git commit` and `git merge`, never `gsd-tools query commit`; never `--no-verify`; `check.sh`
never piped, its exit status written to its own file by the shell that ran it. No AI attribution
in any commit. `Cargo.toml` and `Cargo.lock` untouched; no crate or feature added (T-11-SC).
NVDA on this machine was not stopped, reconfigured or driven; the tester's profile was not read
(the plan's premise 3 fact was taken as given); no binary was started; the readings built every
window they read and called `show_modal` nowhere. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `tests/a_kept_folder_reads_as_a_checked_check_box.rs` | itself, on each commit that changed it, and through its two records' coupling from `613c430f` on |
| `src/presentation/wx_folder_choice.rs` | `--lib presentation::wx_folder_choice` and, coupled, `house_style` and the new target |
| `src/presentation/native_tree_checks.rs` | `--lib presentation::native_tree_checks` |
| `src/presentation/accessibility/names.rs`, `wx_app.rs`, `scan_fixtures.rs`, `mod.rs` | their `--lib` filters and the sixteen coupled targets `wx_app.rs`'s records name |
| `src/application/blocking.rs` | `--lib application::blocking` on the red and the green |
| `tests/theme_reach.rs`, `tests/wired.rs` | themselves |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every commit |

`scripts/check.sh all` ran four times, its output to a file with the exit status written by
the same shell, never piped. At `361ff085`, the last code commit: exit 0, 8,040 passed and none
failed over 75 result lines, 301 s from 15:17:28Z to 15:22:29Z, the release build included;
one more result line than 11-02's 74, the new target, and one more test: 8,039 plus 8 in the
target, 13 more in the module and 1 in `native_tree_checks`, less the 21 retired from
`names.rs`. Then the alpha page's item was found and `99293af1` committed it, and the gate ran
again at that head: **two runs failed**, at 15:24:53Z (245 s, 8,038 passed, 2 failed:
`test_reading_b_space_on_the_tree_and_what_it_did_to_the_state_image` with the state image at
1 after Space, and `test_one_arrow_on_the_settings_tab_row_raises_one_focus_event` seeing
`EVENT_OBJECT_FOCUS` twice per arrow) and at 15:30:32Z (250 s, 8,039 passed, 1 failed, the
tab-row test again). Both tests press keys on live windows and read what the control did; the
tab-row test is 09-06's and this plan touches nothing it reads. A read-only look at the desktop
at 15:30Z found the foreground window to be Firefox on the Keyboard Shortcuts page and the
Control and Alt keys reporting presses since the last read: somebody was using the machine.
The Space reading passed six of six times alone at 15:30Z and in the third gate; the tab-row
test failed alone at 15:30Z and passed alone at 15:43Z once `GetLastInputInfo` had reported
two minutes of idle input. The fourth gate, started at 15:43:59Z with the input idle: exit 0,
8,040 passed and none failed over 75 result lines, 277 s, and `main`'s hook at the merge, 279
s from 15:49:03Z to 15:53:42Z, the same. The two red runs are recorded here because a gate
that went red on the merge head is a fact about this plan's session whatever caused it; what
they show is that a test which sends a key to a live window on a machine somebody is using
can lose the key, and that this machine is the tester's. The keyring race (ledger 374) did not
appear on any run.

## Threat register

T-11-07 mitigated: the state is read per item from `TVM_GETITEMSTATE` at OK, matched to its row
by the walk, and a walk that does not match writes nothing, held by `handles_by_row`'s tests and
reading C. T-11-08 mitigated: the `TVITEMW` layout is declared by hand with its header named,
held at 56 bytes with three offsets by a test in the module and again in the target before any
message carries one, and every message goes to a window this code built and still holds.
T-11-09 accepted as planned. T-11-10 mitigated: `nesting` treats an absent parent as the top, and
a loop too, with tests. T-11-SC: nothing added. No new surface outside the register; the calls
into Windows are the three messages, one style bit and one walk, all on a control this program
created.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 532 before, 534 after; 499
open before, 501 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 533 | unrun-verify | what only the tester's ear settles for #70: checked and not checked heard, the new state after Space, the level, the title, the All Mail sentence on his Gmail account, and whether his Gmail lists All Mail at all |
| 534 | deviation | wxdragon 0.9.17's `acc_state` numbering against wxWidgets' own; the one writer retired; upstream, not reported yet |

## The issue

`gh issue close 70` from the repository root after the merge, closed at 2026-09-18T15:54:08Z,
the comment as the plan wrote it with the merge commit and the reading's finding added, and
the ear list at its end: each row heard as a check box with its state; Space toggling, with
the new state said; the hierarchy read as levels; the title; the All Mail sentence on his
Gmail account; the command found on Tools. It tells him his folder list holds no All Mail and
where Gmail decides that. Closing an issue is not a publish; nothing was pushed.

## Known stubs

None. `build_folder_choice_dialog` has two non-test callers, `ask` and the scan target's arm;
`ask` has one, `choose_folders`, wired to `ID_CHOOSE_FOLDERS` on Tools and to the folder tree's
context menu; `native_tree_checks`' four functions are each called from the dialog; `nesting`,
`handles_by_row`, `is_a_gmail_account` and `the_all_mail_sentence` are each called from the
dialog or `choose_folders`. The `CHECK_STATES_REACH_A_SCREEN_READER` constant is read by `ask`.

## Not done here, on purpose

Whether NVDA says "checked" on a kept folder, the new state after Space, the level on a nested
folder, the account's name in the title and the All Mail sentence are ledger 533 and the
tester's ear; the changelog entry says nobody has heard the tree. Whether the tester's Gmail
lists All Mail is Gmail's Show in IMAP setting, said in the close comment. The upstream defect
in wxdragon's constants is ledger 534 and not reported; the paragraph in `names.rs` warns the
next writer. LIST-01 is ticked on its two `[D]` lines, each covered above, with its `[S]`
lines untouched: the phase README says 11-12 ticks the `LIST` requirements clause by clause,
and the orchestrator's instruction for this plan was to tick it where held, which is the
overrule with its reason; 11-12 still reads it. The row is `3/15`.

## Self-Check: PASSED

`src/presentation/native_tree_checks.rs` and `tests/a_kept_folder_reads_as_a_checked_check_box.rs`
exist; `grep -v '^\s*//' src/presentation/wx_folder_choice.rs | grep -c TreeCtrl` is 4 and
`CheckListBox` 0; `grep -n 'ask(frame, ' src/presentation/wx_app.rs` shows `&account.name` as
the second argument; `guards/guards.toml` holds 914 records by the TOML reader and the census
says 798 + 116; `.planning/WINDOWS.md` holds 533 and 534 in both halves;
`.planning/REQUIREMENTS.md` has LIST-01 ticked; `gh issue view 70` answers CLOSED. Commits
`6bad4765`, `8616d67c`, `0e2bd09a`, `613c430f`, `be5a5b45`, `361ff085`, `99293af1` and
`70f4737b` are in `git log --oneline --all`. Carriage returns zero and em dashes zero on this
file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

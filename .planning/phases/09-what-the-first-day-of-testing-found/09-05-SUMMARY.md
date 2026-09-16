---
phase: 09-what-the-first-day-of-testing-found
plan: 05
subsystem: accessibility scan, manager editors, account editor, guards
tags: [scan-target, msaa, accessible-name, checkbox, spacer, static-text, guards, no-label]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-06: scan targets opened directly on a fixture, the workflow's list held to ScanTarget::ALL, the MSAA walk over every top-level window; 06-08: the whole-tree label check as its own target, and the whitespace-name mechanism"
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-04: a source-reading target with companions coupled to the files it reads by a record's suite; the rule that a fix under the alpha gets a changelog entry and no bump"
provides:
  - "Five scan targets, contact-editor, condition-editor, filter-editor, signature-editor and account-editor, each built by the manager's own builder on the main frame on a fixture from scan_fixtures and shown with no manager behind it; the workflow's list holds all five"
  - "build_rule_edit_dialog, build_filter_edit_dialog, build_sig_edit_dialog and build_account_edit_dialog take &dyn WxWidget, as build_contact_edit_dialog did; the managers' own calls are unchanged"
  - "Every checkbox in the five editors named through set_accessible_name with the mnemonic stripped; wx_managers::add_checkbox builds four of them; names::leave_the_cell_empty holds a grid cell open with a sizer spacer where an empty static text stood"
  - "The account editor's section, cb and cb_with_description closures hand back the control alone; ImapFields, PopFields, PasswordFields and Page2Shell carry no spacer to hide"
  - "tests/checkbox_labels.rs builds the five editors and asks each of fifteen checkboxes for an accessible object and for a neighbour that is not an empty static, read from the built tree"
  - "tests/no_label_is_only_a_space.rs refuses an empty static text whose only later use is a sizer add, with two companions splicing into wx_managers.rs's own lines, and no allow list"
  - "Five guard records measured, two corrected and measured, one re-counted"
affects: [09-06 onward, which write changelog entries under Unreleased; 09-10, the phase's closing read, which reads FOUND-08 clause by clause and finds its second [D] line waiting on CI; anything that builds a checkbox in a two-column grid]

actuals:
  tokens: 14812
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A nested modal becomes a scan target by calling the manager's own builder on the frame and showing the dialog itself, not the manager's show_* function, which reads the answer back into a list there is no list for"
    - "A control that carries its own label sits alone in its grid row beside a sizer spacer, never beside an empty static text: the spacer is not a window and cannot be picked as a name for what follows"
    - "When the prescribed instrument crashes, a lower-layer one on the same running window can still measure the structural half of the mechanism: a Win32 child walk showed the nameless statics gone where the MSAA walk could not read a name"
    - "A companion that splices into a real file judges its splice against the file's own reading, not against nothing, so a guard record planting the same shape reddens the whole-tree check and not the companion"

key-files:
  created: []
  modified:
    - src/presentation/scan_target.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_managers.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/accessibility/names.rs
    - .github/workflows/accessibility.yml
    - tests/checkbox_labels.rs
    - tests/no_label_is_only_a_space.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/wcag-coverage.md

key-decisions:
  - "The four builders that took &Dialog take &dyn WxWidget, rather than a bare Dialog built under the frame as a shell: each used the parent only to hand it to Dialog::builder, which takes any window, the contact editor's builder already took one, and an invisible shell would have been one more window in the tree for no reason"
  - "The account editor target opens on its second page, because the first is three labelled text fields and every checkbox the editor has is on the second, which is the page #42 is about"
  - "The eleven spacers are sizer spacers of one pixel, none kept: wxdragon adds nothing for a spacer of zero, and a FlexGridSizer row whose partner is hidden collapses to the spacer's height plus the gap, which in a fixed-size dialog with the grid at proportion one is space that was blank already"
  - "The whole-tree reading refuses a binding whose only later use is a sizer add, not the plan's list of five receiving calls: that list flagged three filled lines and would have needed an allow list of filled lines; this rule refuses exactly the five in wx_managers.rs on the old tree and none of the filled lines, and says in its module comment that a spacer handed on in a tuple is invisible to it"
  - "No allow list, because nothing needs one: an empty list watched by a test that iterates over nothing is the census-emptying failure CLAUDE.md describes"
  - "The Win32 child walk is recorded as a before-and-after picture of the mechanism's structural half, and is not offered as the MSAA rows #42 asks for, which have been read by nothing"

patterns-established:
  - "When a plan's verification instrument crashes and the plan says wait, wait; but say what a cheaper instrument on the same window did read, and say plainly which question it does not answer"

requirements-completed: []

coverage:
  - id: D1
    description: "The five editors are scan targets opened directly with a fixture, in the workflow's list, so both channels reach them"
    requirement: FOUND-08
    verification:
      - kind: unit
        ref: "src/presentation/scan_target.rs#test_every_window_a_fresh_profile_can_reach_has_a_name"
        status: pass
      - kind: unit
        ref: "src/presentation/scan_target.rs#test_the_workflow_asks_for_every_target"
        status: pass
      - kind: other
        ref: "each of the five started here with --scan-target on a throwaway profile under WIXEN_MAIL_DATA; EnumWindows listed Edit Contact, Edit Condition, Edit Filter Rule, Edit Signature and Edit Account, each owned, above Wixen Mail; the account editor's children began with Step 2 of 2"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every checkbox in those editors is named through set_accessible_name with the mnemonic stripped, and the MSAA walk on each editor reports a name for every checkbox, on this machine or on CI's next run"
    requirement: FOUND-08
    verification:
      - kind: integration
        ref: "tests/checkbox_labels.rs#test_every_check_box_in_a_form_carries_its_own_label"
        status: pass
      - kind: other
        ref: "scripts/msaa-names.ps1 on each of the five editors, once before task 2 and once after: exit -1073740791 every time, nothing printed before it"
        status: unrun
    human_judgment: false
    rationale: "The naming half holds by the test; the walk half waits for the Accessibility workflow at the next push, ledger 489, and this line of FOUND-08 stays open until then"
  - id: D3
    description: "No StaticText built with an empty literal is added to a sizer and never filled or named afterwards, and tests/no_label_is_only_a_space.rs refuses the shape with a companion that plants one"
    requirement: FOUND-08
    verification:
      - kind: integration
        ref: "tests/no_label_is_only_a_space.rs#test_no_empty_static_is_built_that_nothing_fills"
        status: pass
      - kind: integration
        ref: "tests/no_label_is_only_a_space.rs#test_the_reading_can_see_a_spacer_nothing_fills"
        status: pass
      - kind: other
        ref: "grep -rn 'with_label(\"\")' src/presentation/*.rs | wc -l on main at 165fd811 -> 11, was 22 at 1d934e26; the reading over src/ prints seen 11, filled later 11, refused 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "What the tester hears on the signature editor and the contact editor afterwards"
    requirement: FOUND-08
    verification: []
    human_judgment: true
    rationale: "FOUND-08's last [S] line, ledger 490. Structure is held by two tests; whether the box is heard with its name is the tester's, and why he heard it without one is still not settled by anything read here"

duration: 1h35m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 05: The five editors reached by the scan, every checkbox named, the spacers gone Summary

**The contact, condition, filter, signature and account editors are scan targets, each opened on
the frame on a fixture and seen here; every checkbox in them is named through
`set_accessible_name` and no empty static text sits before one, held by a test that builds the
five editors and reads the built tree; the whole-tree check refuses an empty static nothing fills.
What the channel NVDA reads says about those checkboxes has been read by nothing: the MSAA walk
left with `-1073740791` on every one of ten runs here, as ledger 390 records, so the before and
after rows #42 asks for wait for the next push's run, and #42 is advanced rather than closed on
the tree's side, which the issue comment says.** Nothing pushed.

## Performance

- **Duration:** about 1 h 35 min from the first read to the merge, of which about 8 minutes were
  guard measurement across ten runs and about 13 minutes the two whole gates
- **Started:** 2026-09-16T19:45Z (first commit 20:10:03Z)
- **Merged:** 2026-09-16T21:18:25Z at `165fd811`
- **Tasks:** 3
- **Files modified:** 12 (none created)

## What the walk did here, verbatim

Ledger 390 was re-taken before anything was built, on the Signatures target at `1d934e26`, with
the application on a throwaway profile under `WIXEN_MAIL_DATA` and `scripts/msaa-names.ps1`
run under `pwsh -NoProfile` the way `accessibility.yml` runs it:

```
signatures : visible top-level windows: Signature Manager (owned) | Wixen Mail (not owned)
signatures : MSAA walk exit -1073740791
```

Then once on each of the five editors after task 1, before task 2 changed anything:

```
contact-editor : visible top-level windows: Edit Contact (owned) | Wixen Mail (not owned)
contact-editor : MSAA walk exit -1073740791
condition-editor : visible top-level windows: Edit Condition (owned) | Wixen Mail (not owned)
condition-editor : MSAA walk exit -1073740791
filter-editor : visible top-level windows: Edit Filter Rule (owned) | Wixen Mail (not owned)
filter-editor : MSAA walk exit -1073740791
signature-editor : visible top-level windows: Edit Signature (owned) | Wixen Mail (not owned)
signature-editor : MSAA walk exit -1073740791
account-editor : visible top-level windows: Edit Account (owned) | Wixen Mail (not owned)
account-editor : MSAA walk exit -1073740791
```

And once more on each after task 3, the same five lines with the same exit. Nothing was printed
by the script before it died on any run. NVDA was running and was not stopped, as the plan said;
nothing was diagnosed. No COM library is installed for Python here (`comtypes`, `pywin32`,
`pywinauto` all absent) and installing one is excluded, so no narrower probe of one control's
`IAccessible` name was made. **The MSAA rows for every checkbox in each editor, before and after,
do not exist**, and the wording for #42 is the plan's second one.

## What was read instead, and what it is not

The same running windows were walked with `EnumChildWindows`, `GetClassName` and
`GetWindowText`, from a scratch PowerShell script that starts the target the way the workflow
does. That is the Win32 layer under both accessibility channels: it lists every visible child
window with its class and its window text, and it cannot say what MSAA or UI Automation names
anything. It is a before-and-after picture of the mechanism the issue names, a nameless window in
the tree straight before the checkbox, and nothing more.

Before, on `main` at `7947392c` (task 1 merged into the branch, task 2 not yet):

| target | child windows | `Static ''` among them | what stood straight before the checkbox |
|---|---|---|---|
| contact-editor | 30 | 2 | `Static ''` then `Button '&Favorite'`; the other is the problem line after it |
| condition-editor | 11 | 2 | `Static ''` then `Button '&Case Sensitive'`; the other is the "what it can find" line after it |
| filter-editor | 18 | 2 | `Static ''` then `Button '&Case Sensitive'`, `Static ''` then `Button '&Enabled'` |
| signature-editor | 8 | 1 | `Static ''` then `Button '&Default signature'` |
| account-editor | 53 | 15 | `Static ''` before each of six checkboxes, six section headings, the sign-in hint and the allowed note; `Step 2 of 2: Connection and sign-in` first |

The signature editor in full, before: `Static '&Name:'`, `Edit ''`, `Static ''`,
`Button '&Default signature'`, `Static '&Signature, in Markdown:'`, `Edit ''`, `Button 'OK'`,
`Button 'Cancel'`.

After, on `main` at `165fd811`:

| target | child windows | `Static ''` among them | what stands straight before the checkbox |
|---|---|---|---|
| contact-editor | 29 | 1 | `Edit ''` (the Avatar URL field) then `Button '&Favorite'`; the one left is the problem line |
| condition-editor | 10 | 1 | `Edit ''` (Pattern) then `Button '&Case Sensitive'`; the one left is the "what it can find" line |
| filter-editor | 16 | 0 | `Edit ''` then each checkbox |
| signature-editor | 7 | 0 | `Edit ''` (Name) then `Button '&Default signature'` |
| account-editor | 38 | 0 | the field or heading before each checkbox, no empty static anywhere |

The signature editor in full, after: `Static '&Name:'`, `Edit ''`, `Button '&Default signature'`,
`Static '&Signature, in Markdown:'`, `Edit ''`, `Button 'OK'`, `Button 'Cancel'`.

The two `Static ''` left are the contact editor's problem line and the condition editor's "what
it can find" line, both filled by the program when there is something to say, and both after the
checkbox rather than before it. What NVDA now names the five boxes is not in this table.

## What landed

**Task 1, the five targets.** `ScanTarget` has `ContactEditor`, `ConditionEditor`,
`FilterEditor`, `SignatureEditor` and `AccountEditor`, `ALL` is thirty-five, each has a doc
comment saying what it opens and which manager target stopped short of it, and the workflow's
array holds the five names after `notes-module` with a comment. Each arm calls the manager's own
builder on the frame with a fixture, calls `show_modal` on the dialog it hands back, and takes it
down afterwards, because the managers' `show_*` functions read the answer back into a list there
is no list for. The fixtures: a contact with every field filled and a row on each of its four
lists; a condition and a filter on `subject` and `contains`, which the editors agree to show (both
refuse a stored one their lists have no words for, before building); the default signature, so
the box the tester met is scanned ticked; and the scan-only account the Accounts target uses,
turned to its second page with `advance_to_connection_page` before it is shown, because the first
page is three text fields and every checkbox is on the second. Four builders that took `&Dialog`
take `&dyn WxWidget`: each used the parent only for `Dialog::builder`, which takes any window, and
the contact editor's already did. `show_rule_edit`, `show_filter_edit`, `show_sig_edit` and
`show_edit` still take `&Dialog` and call the builders as before.

**Task 2, the names and the spacers.** `wx_managers::add_checkbox(parent, sizer, label)` builds
a checkbox with its label, names it through `set_accessible_name(&check, &name_from_label(label))`,
holds the label cell open with `leave_the_cell_empty(sizer)` and adds the box; Case Sensitive
twice, Enabled and Default signature go through it, and Favourite, on a panel rather than the
dialog, is named and spaced the same way inline. `names::leave_the_cell_empty(grid)` is
`grid.add_spacer(1)` with the reason in its doc comment: one pixel because `wxdragon` adds nothing
at all for a spacer of zero and the cells after it would shift a column. In the account editor,
`section` hands back the heading, `cb` and `cb_with_description` hand back the checkbox,
`permission_box` follows, the sign-in hint, the app-password button and the allowed note sit
beside a spacer, and `ImapFields`, `PopFields`, `PasswordFields` and `Page2Shell` lost eighteen
`StaticText` fields between them and the `show` lines that hid them. Nothing about what any
checkbox is called or does changed. On `main` at `165fd811`:

```
grep -rn 'with_label("")' src/presentation/*.rs | wc -l
11
```

The eleven: `wx_account_manager.rs:231` and `:1513`, `wx_app.rs:722`, `wx_blocked_senders.rs:182`,
`wx_item_form.rs:536`, `wx_managers.rs:444`, `:883`, `:1472` and `:2859`, `wx_send_later.rs:190`,
`wx_settings.rs:2585`, every one a line something fills or names later, and the reading in task 3
says so of each. It was 22 at `1d934e26`.

Two records, both on `wx_managers.rs` with `suite = "checkbox_labels"`: the naming call taken out
of `add_checkbox` (four rows in the one complaint), and the spacer built back before Favourite as
the tester met it. The two records from 2026-08-16 on the POP checkboxes named the old tuple shape
`let (pop_leave_label, pop_leave) =` in their break and would have been unmeasurable; both
corrected to `let pop_leave =` with the date and reason on the record, and measured again on the
whole library. The source-reading test in `wx_account_manager.rs` that named the same tuple was
corrected with them.

**Task 3, the reading.** `tests/no_label_is_only_a_space.rs` reads every
`let NAME = StaticText::builder(..)` whose chain to `.build()` carries `.with_label("")`, across
lines, and refuses it unless `NAME` is used again before the end of its function (the first `}`
at column zero) for anything other than being handed to a sizer's `add(` or `add_sizer(`. The
whole-tree test prints what it saw; on `main` at `165fd811`:

```
empty statics seen: 11, filled later: 11, refused: 0
```

Two companions splice into `wx_managers.rs`'s own lines before `let fav_label = "&Favorite";`:
one plants the old spacer under a name the file never uses and requires the reading to name it at
its line and nothing else the file did not already say; the other plants five statics filled five
ways (`set_label`, `said_and_shown(&..)` with the sizer add wrapped over five lines,
`set_accessible_name(&..)`, a chain with `.with_label("")` on its own line and `get_handle()`
after, and a block handing its static on bare) and requires the count refused unchanged and the
count seen up by five. Against the seven files as they stood at `1d934e26`, exported with
`git show` into the scratchpad and read by a scratch test removed before the commit, the reading
answered: `wx_managers.rs` seen 9, refused 5, `1285: fav_spacer`, `2826: cs_label`,
`3285: cs_label`, `3309: en_label`, `3837: def_label`; `wx_account_manager.rs` seen 8, refused 0;
the other five files seen 1 each, refused 0. So it catches the five the tester's boxes were among
and none of the eleven filled lines, and it does not see the account editor's six, which were
handed on in a tuple `(l, c)` to be stored and hidden; the module comment says so and points at
`tests/checkbox_labels.rs`, which reads the built tree for the fifteen editor checkboxes and sees
a spacer whatever shape built it. One record, the same spacer put back with
`suite = "no_label_is_only_a_space"`.

## Task commits

| Commit | What |
|---|---|
| `bda93875` | test(09-05): the red half of task 1, five tests named: the five names, and four fixtures stubbed to what their editors refuse |
| `7947392c` | feat(09-05): the five variants, fixtures, arms and workflow names; four builders widened; two records; the walk before, crashed |
| `df91fb62` | test(09-05): the red half of task 2, one test named, red on twenty rows |
| `4c3c9eea` | feat(09-05): the five named, the eleven spacers gone, `add_checkbox` and `leave_the_cell_empty`, two records, two corrected, the changelog entry, the coverage page |
| `87d24736` | test(09-05): the red half of task 3, two companions and the count check named, the reading stubbed to see nothing |
| `7c5819a1` | feat(09-05): the reading, one record, the count remedy |
| `165fd811` | Merge 09-05 into `main` |

Branch `five-editors-reached-and-every-checkbox-named` from `main` at `1d934e26`. Not pushed;
53 commits unpushed before the commit that lands this summary.

## Honest RED and GREEN

Task 1's red commit names five: `test_every_window_a_fresh_profile_can_reach_has_a_name` with
the five names added to its list, and the four fixture tests against fixtures that returned a
contact with no rows, a condition and a filter on empty fields, and a signature that was not the
default, on 06-06's pattern so each fails for the reason it exists; the unused-import warning
clippy would have refused was kept out of the red by importing the four row types at green. Task
2's red commit names one, `test_every_check_box_in_a_form_carries_its_own_label`, extended inside
its one `#[test]` (one `wxdragon::main` per process) with the five editors, red on twenty rows
quoted in the commit: five unnamed and fifteen behind a spacer, ten of those in the account editor,
which premise 1 said names its own. Task 3's red commit names three: the two companions, red
against a reading stubbed to return nothing while the whole-tree check passed for free, which is
what the companions exist to show, and
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, which fired because the
file went from four tests to seven and one record names it, named in the trailer as `CLAUDE.md`
prescribes since its remedy needs the green. Every red commit ran and failed exactly what it
named and nothing else, and the gate said so in `red` mode. One red commit per task.

The task-3 record was measured three times before it agreed. The first two runs reported the
two companions red beside the one test named, because they spliced into the real file and judged
against nothing, so a spacer planted in the file itself reddened them too; the companions now
judge against the file's own unspliced reading. The second refusal was the reading's own reuse
blindness: the companion planted `fav_spacer` and the record plants `fav_spacer`, and with both
in the file the first is "used later" by the second's `let`. The companion plants
`planted_spacer` now, and the comment says why. Neither change touched what the reading refuses.

## What the gate selected

| File | On the branch |
|---|---|
| `src/presentation/scan_target.rs`, `scan_fixtures.rs` | `--lib presentation::scan_target::` (11) and `--lib presentation::scan_fixtures::` (10) on task 1's red; the whole gate on its green |
| `.github/workflows/accessibility.yml` | `all` on task 1's green: 7,823 passed and none failed over 65 result lines, 6 min 18 s through the hook |
| `src/presentation/wx_app.rs`, `wx_managers.rs`, `wx_account_manager.rs` | rode task 1's `all` |
| `tests/checkbox_labels.rs` | `--test checkbox_labels` on task 2's red as a changed target, and on its green as coupled |
| `src/presentation/wx_managers.rs`, `wx_account_manager.rs`, `accessibility/names.rs` | `--lib presentation::wx_managers::` (44), `--lib presentation::wx_account_manager::` (14), `--lib presentation::accessibility::names::`, plus the coupled targets `manager_dialog_labels`, `manager_delete_stays_open` and `checkbox_labels` (task 2's green) |
| `tests/no_label_is_only_a_space.rs` | `--test no_label_is_only_a_space` on task 3's red and green as a changed target; it is in the whole-tree list too |
| `guards/guards.toml`, `docs/changelog.md`, `docs/wcag-coverage.md` | no scoped target; all rode code commits, so no `docs_only` run happened |

Every commit ran the whole-tree guards. `scripts/check.sh all` on the branch at `7c5819a1`, run
once, output to a file and exit status read directly, never piped: exit 0 in 373 s, 7,826 passed
and none failed over 65 result lines, the release build included. Seven more than 09-04's 7,819:
four in `scan_fixtures`, three in `no_label_is_only_a_space`. The keyring race (ledger 374) did
not appear. `main`'s hook ran `all` again on the merge: 7,826 and none failed in 378 s.

`bash scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_managers.rs` answers
`manager_dialog_labels`, `manager_delete_stays_open`, `checkbox_labels` and not
`no_label_is_only_a_space`, because a target already in `guards_that_read_the_whole_tree` is
dropped on purpose (ledger 442). A copy of the script in the scratchpad with that name taken out
of the list answers the four with `no_label_is_only_a_space` third, which is the coupling shown
the way 08-03 showed it.

## Guard records

841 records by the TOML reader before, 846 after; census 802 + 39 before, 802 + 44 after, the
line at `guards/guards.toml:84` moved in each green commit. Five new, two corrected, one
re-counted, every one measured through `scripts/guards.sh --remeasure`, `WIXEN_TEST_THREADS`
untouched.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| an editor the program can open for the scan is one the workflow asks for | `accessibility.yml`, library | `'signature-editor'` out of the array | 1 | 50 s |
| the name the scan asks an editor by is the name the program answers to | `scan_target.rs`, library | `as_name` says `signatures-editor` | 2 | 78 s |
| a checkbox in an editor grid is named outright, not left to the fallback | `wx_managers.rs`, `checkbox_labels` | the naming call out of `add_checkbox` | 1 | 17 s |
| no empty static text is built before the Favourite box as a spacer | `wx_managers.rs`, `checkbox_labels` | the old spacer built before the box | 1 | 16 s |
| no empty static text nothing fills is placed in a sizer, read from the source | `wx_managers.rs`, `no_label_is_only_a_space` | the same spacer | 1 | 13 s, on the third measurement |
| POP's leave-on-server checkbox keeps its spoken consequence (corrected) | `wx_account_manager.rs`, library | as before, on `let pop_leave =` | 1 | 78 s |
| POP's local-delete checkbox keeps its spoken consequence (corrected) | `wx_account_manager.rs`, library | as before, on `let allow_deleting =` | 1 | 79 s |
| no status line is built with a label that is only a space (re-counted) | `wx_managers.rs`, `no_label_is_only_a_space` | unchanged | 1 | 13 s |

Counts written: `scan_target.rs` 11, `wx_managers.rs` 44, `wx_account_manager.rs` 14,
`checkbox_labels.rs` 1, `no_label_is_only_a_space.rs` 7 (was 4). `wx_app.rs` stays at 199 by the
runner and `scan_fixtures.rs` at 10 is named by no record. The count check fired once, on task
3's red, for the one record naming `no_label_is_only_a_space.rs`, and its remedy was run and read.

## Premises the tree contradicted

1. **Premise 5's rule for the widened reading flags three filled lines.** "Refuse unless NAME is
   the receiver of `set_label`, `said_and_shown`, `set_accessible_name`,
   `set_accessible_name_and_description` or `set_value`" would refuse the account editor's
   sign-in hint (`h`, handed on bare from a block and filled as `auth_hint`), the condition
   editor's "what it can find" line (handed to the dialog's own `say_what_this_field_cannot_find`)
   and the main window's live region (written through Win32 after `get_handle()`), so the allow
   list the plan wanted "holding only what task 2 left" would have held filled lines instead. The
   rule written refuses a binding whose only later use is a sizer add. Its cost is the account
   editor's old tuple shape, measured and written on the record, the module comment and ledger
   491.
2. **The plan's acceptance criterion `grep -c "'[a-z]*-editor'" .github/workflows/accessibility.yml`
   is 1, not 5.** `grep -c` counts lines and the five names are on one line;
   `grep -o "'[a-z]*-editor'" ... | wc -l` is 5.
3. **Line numbers moved by 17 to 21** since the plan's premises were taken at `524ff24f`: on
   `main` at `1d934e26` the five checkboxes were at `wx_managers.rs:1285`, `:2826`, `:3285`,
   `:3309` and `:3837` still, but the account editor's spacers were at `:1459`, `:1465`, `:1484`,
   `:1539`, `:1642` and `:1712` as the plan said, and `build_account_edit_dialog` at `:1384`. The
   shapes held exactly.
4. **`wxdragon::Sizer::add_spacer(0)` adds nothing.** Read from `sizers/base.rs:100`, which
   guards on `size > 0`; a spacer of one pixel is what holds the cell.

Premises 1 to 4 held otherwise: 22 empty literals, eleven filled and eleven spacers, the five
checkboxes unnamed and the account editor's ten named (the red run listed exactly that split),
the scan stopping at the managers, and the walk crashing as 390 records.

## Deviations from plan

**1. [Decision] No allow list.** The plan's task 3 asked for an allow list of
`(file, binding, reason)` with a test that every entry's site exists. Nothing needs one, and a
list with no entries watched by a test that iterates over nothing is the census-emptying failure
`CLAUDE.md` describes, so none was written; the module comment says when one would be. Ledger 491.

**2. [Decision] The reading's rule.** Premise 1 above. Ledger 491.

**3. [Rule 3] Two records corrected.** The POP pair's break text named the tuple task 2 removed,
so `test_every_guard_record_still_names_one_place_in_the_tree` would have refused the green; both
corrected and measured. Commit `4c3c9eea`.

**4. [Decision] The Win32 child walk.** Not in the plan. It reads the same running window on the
layer under both channels and cost one script; it is offered as what it is, above. No package was
added and no tracked file was touched by it.

**5. [Decision] The account editor target opens on its second page.** Above, and in the variant's
doc comment.

Everything else executed as written. No scripted edit touched a tracked file: exception set zero,
and it stayed there; the one `sed` was on a scratchpad copy of `check.sh` for the coupling proof,
and the seven old files were exported with `git show` into the scratchpad. Commit messages were
written to the scratchpad and passed with `-F`; `cargo fmt` ran before each commit. Carriage
returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on
each. No em-dash in any file this plan wrote, measured with `grep -c` for the byte sequence.
`Cargo.toml` untouched; no package added. The two release-binary runs per target and the walks
used a throwaway profile under the scratchpad pinned with `WIXEN_MAIL_DATA`; the tester's profile
was not touched.

## Threat register

T-09-15: the MSAA rows do not exist and this summary, the coverage page, the changelog's Known
limitations, the issue comment and ledger 489 all say so; nothing is claimed from the call.
T-09-16: the arms build the editors on `scan_only_account()` and on `scan_fixtures` data and
write nothing; `show_contact_edit`, `show_rule_edit`, `show_filter_edit`, `show_sig_edit` and
`show_edit` are untouched. T-09-17: the filled-later rule, the shapes companion with five fills,
and the reading's own count, seen 11, filled 11, refused 0. T-09-18: no allow list, so no entry to
outlive a site. T-09-SC: no package added; three Python COM packages were considered for a probe
and not installed. No new surface outside the register.

## Ledger

`.planning/WINDOWS.md` 489 to 491 written through `gsd-tools windows append`, both halves at 491,
no backslash in any description (the nine in the file predate this plan); 390's description
extended by hand in both halves with today's eleven runs and nothing else;
`the_planning_files_agree_with_themselves` green after. 488 before, 491 after; 460 open before,
463 after.

| id | kind | what |
|---|---|---|
| 489 | unrun-verify | the five editors seen here, the walk crashed on each twice, the rows wait for CI; criterion 5's walk clause and FOUND-08's second `[D]` line open until then |
| 490 | unrun-verify | what NVDA says on the signature and contact editors after the change, and why the tester heard the two unnamed; the next instrument is NVDA's own log on his machine |
| 491 | deviation | the reading's rule against the plan's, what it catches on the old tree and what it cannot see, and why there is no allow list |

## Known stubs

None. Every target is wired end to end: a variant, a name the parser accepts, an arm that opens
the editor, and an entry in the workflow's array, and each was seen open. `add_checkbox` is
called at four sites and `leave_the_cell_empty` at eight in the source: inside `add_checkbox`,
before Favourite, and six in the account editor's closures and blocks, three of which are
closures called eighteen times between them (`section` seven, `cb` five, `cb_with_description`
six, three of those through `permission_box`). The two new tests are reached
by the gate: `checkbox_labels` through three records' `suite` and `no_label_is_only_a_space`
through the whole-tree list and one record's `suite`.

## Not done here, on purpose

The MSAA rows, above. What NVDA says on the two editors, ledger 490. Why the tester heard the two
boxes unnamed: a native checkbox carries its own window text and the MSAA proxy names a button
from it, so the empty static is a mechanism for a control that set no name and has no text of its
own, and whether it was the mechanism for these two is what the walk would have said; the naming
and the spacers stand on their own merits either way, and if the next build still reads the box as
unnamed the next instrument is NVDA's own log with Tab on the signature editor. No `FOUND`
requirement is ticked, on the phase's rule that the last plan reads each clause by clause; the
closing read should count FOUND-08's second `[D]` line and criterion 5's walk clause as open until
the Accessibility workflow has walked the five editors. Nothing pushed.

## Self-Check: PASSED

`src/presentation/scan_target.rs` names five `*-editor` targets and `ALL` is 35;
`grep -o "'[a-z]*-editor'" .github/workflows/accessibility.yml | wc -l` is 5;
`grep -rn 'with_label("")' src/presentation/*.rs | wc -l` is 11; `tests/checkbox_labels.rs` and
`tests/no_label_is_only_a_space.rs` exist and pass; `guards/guards.toml` holds 846 records by the
TOML reader. Commits `bda93875`, `7947392c`, `df91fb62`, `4c3c9eea`, `87d24736`, `7c5819a1` and
`165fd811` are in `git log --oneline` on `main`.

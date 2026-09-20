# Plan check: phase 12, the editors and what the alpha still owes

Read on 2026-09-20 against `main` at `19ad8648`, clean. `git diff --stat 0ad66e48 19ad8648`
touches only the seventeen planning files, so every reading the README and the plans took at
`0ad66e48` holds at `19ad8648`; the line numbers below were taken at `19ad8648`. Twelve plans,
one per wave, a linear dependency chain `12-01 -> ... -> 12-11`, with 12-12 on all eleven; no
cycle, no forward reference, and no two plans share a wave, so `files_modified` overlap never
arises. `verify.plan-structure` answers `valid: true, errors: 0` for all twelve. No
RESEARCH.md, CONTEXT.md, PATTERNS.md or REVIEWS.md exists for the phase, so the Nyquist,
context, pattern and review dimensions are not applicable. `workflow.tdd_mode` is `true`;
the plans use `type="auto" tdd="true"`, the form phase 11's 57 tdd tasks used.

Written with a quoted shell heredoc, because the Write tool was disabled in the checking
session; the file is new and untracked, and no tracked file was touched.

**Verdict: ISSUES FOUND. Three blockers (12-03, 12-07, 12-08), each a small fix; sixteen
warnings; the rest passes.**

## What was checked for every plan, and the commands

- Every `<verify><automated>` block: none ends in `| wc -l`, `|| true` or `| cat`, none
  passes two `--lib` filters to one `cargo test`, and every one is a chain of commands whose
  exit status carries the answer (12-01 task 2 ends in `grep -q`, which can fail).
  Command: `grep -ho '<automated>[^<]*</automated>' 12-*-PLAN.md | grep -E '\| *wc -l|\|\| *true|\| *cat|--lib [a-z:]* --lib'` answers nothing.
- Every `tdd="true"` task names its red trailers: lib tests by module path, integration
  targets bare, and the count check named bare where a file a record names gains a test
  (12-02 `command_line.rs`, 12-07 `wx_item_form.rs`, 12-10 `tags.rs`). 12-01 task 2 says why a
  JavaScript case has no local red and names the run instead.
- Every plan's `files_modified` holds `docs/changelog.md` (all but 12-11), `guards/guards.toml`
  (all but 12-11 and 12-12) and `.planning/WINDOWS.md` (all); no plan takes a figure that would
  owe `docs/development/measurements.md` a row. The phase convention, phase 11's too, leaves
  `REQUIREMENTS.md`, `ROADMAP.md` and `STATE.md` out of `files_modified` for the completion
  marks; 12-01 lists `.planning/REQUIREMENTS.md` in task 3's `<files>` only, which is a
  wording difference and not a collision.
- No plan adds a startup read: 12-08 reads its setting with `load_stored()` where the form
  opens, 12-05 reads the default account when the dialog opens, 12-09 reads the signature
  when compose opens; `tests/a_setting_saved_applies_without_a_restart.rs` still holds the
  allowlist untouched.
- Every decision listed in the README as Pratik's stays undecided in the plans: 12-05 takes
  `Ctrl+Shift+F` and says it is one line to change; 12-09 puts the choice on the account
  dialog and says the ear is his; 12-11's decisions table has an empty answer column by its
  own criterion.
- House style over the plan directory, the roadmap and the requirements: no em dash, none
  of the six words (`grep -rniw` over the directory answers nothing).
- Estimates: `estimate-check --calibrated` answers `over_budget: false` for all twelve
  (ratios 0.19 to 0.38 of the 100,000 budget), confidence `low` as the README derives it.
- Phases 13 and 14: `ls .planning/phases/13* 14*` finds nothing; GAP-01 to GAP-13 and REAL-01
  and REAL-02 each carry at least one `[D]` and two `[S]` lines; the roadmap entries say "not
  planned" with the reason and carry no progress row; their traceability rows read "Not
  planned, 2026-09-20".

## Blockers

### 12-08, task 1: the field lands in a green commit that cannot be green

`src/data/config.rs:2627` is `test_every_setting_somebody_can_change_is_read_by_something`,
and its reader census at `:2533-2620` excludes `data/config.rs` and `wx_settings.rs` by name
(the comment at the exclusion says neither is anybody acting on the setting). Task 1 adds
`event_length_minutes`, the settings control and the read-back, and its `<verify>` requires
`cargo test --lib data::config::` at 68 passing; the program's reader, the `load_stored()`
read in `wx_item_form`, is task 2's. So task 1's green commit leaves that guard red, which
is exactly what 11-11.1.2 found and what the plan's own truths line ("the two settings guards
go red on arrival and green with the control") misreads: one guard is green with the
control, the other only with a reader outside those two files. The plan names only
`test_every_setting_somebody_can_change_is_offered_by_a_screen` (premise 2) and never the
read-by guard, so the red trailer "the two settings guards named by module path" cannot be
written from the plan's text.

Fix: move the one `load_stored()` read at the form's opening (or the `Block::from_setting`
call on it) into task 1's green commit with the field and the control, and name both guards
in task 1's red trailer: `data::config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_read_by_something`
and `...::test_every_setting_somebody_can_change_is_offered_by_a_screen` (the module path is
`every_setting_is_acted_on` at `config.rs:2533`; the first is at `:2627`, the second at
`:3004`).

### 12-07, task 1: nine files the task must edit are not in `files_modified`

`pub struct ContactEntry` at `src/data/message_cache/mod.rs:591` derives `Debug, Clone,
PartialEq` and not `Default`, and only 2 of the 114 literal sites end in
`..Default::default()`, so three new fields are three lines at every literal. The plan's own
count (premise 1, 114 sites in 17 files) is right; its `files_modified` lists 8 of the 17.
Command: `grep -rn 'ContactEntry {' src tests --include=*.rs | grep -v 'pub struct' | cut -d: -f1 | sort | uniq -c`.
The files with literals the plan does not list: `src/data/message_cache/held_conflicts.rs` (4),
`tests/integration_tests.rs` (3), `src/service/carddav.rs` (3),
`tests/manager_delete_stays_open.rs` (2), `tests/finding_people_answers.rs` (2),
`src/service/outlook_data_file.rs` (2), `src/application/looking_people_up.rs` (2),
`src/service/directory.rs` (1), `src/application/importing_an_outlook_data_file.rs` (1).
The gate maps each to its own filter or target and the plan's gate paragraph (premise 5)
does not name them.

The same task also moves anchors the plan does not list. The card-over-held merge in
`contacts.rs` is anchored by a chain of eighteen records, each on two consecutive field
lines ("a card over what is held carries the card's given_name over a stale one" anchors
the `given_name` line and the `family_name` line together, and so on through the
`custom_fields_json` line and `..held.clone()`). Inserting `name_prefix`, `middle_name` and
`name_suffix` anywhere in that literal splits at least one pair and
`test_every_guard_record_still_names_one_place_in_the_tree` refuses the commit, which is the
10-04 case in `CLAUDE.md`. Command: read `guards/guards.toml` with `tomllib` and print the
name of every record whose `before` or `after` holds `family_name: from_card`.

Fix: add the nine files to `files_modified` and to task 1's `<files>`, with their filters in
premise 5 (`tests/integration_tests.rs` and the two other targets run as themselves); say
where in the merge literal the three fields go and name the record pair whose anchor that
splits as one the executor rewrites and re-measures in the same commit.

### 12-03, task 2: a record anchored on a sentence the task rewrites is not listed

The record "mark as read acts on every selected message, not the cursor row alone"
(`file = src/presentation/wx_app.rs`) has its `before` anchored on the block ending
`return send_refusal(tx, rt, "No message selected");`, which is the site at
`wx_app.rs:10801` that task 2 rewrites onto `nothing_chosen(Kind::Message)`. The plan's
premise names the site and its `--suites-for` sentence names the file, but the plan lists
no record by anchor, and the rule under "Tests that would notice" asks for both readings.
Command: the same `tomllib` reading with `No message selected` as the needle.

Fix: name that record in task 2's guard-record paragraph as one whose `before` and `after`
are rewritten onto the new call and re-measured in the same commit; the same paragraph
should say the executor runs the needle over `guards.toml` for every literal the census
lists before the first rewrite, since the census is longer than this one probe.

## Warnings

1. **12-01, premise 1**: "the first `expect` at line 139 passed" is at `:141`; `:139` is a
   comment line. `:151` for the second `waitToHearAll` is right.
   Command: `sed -n 139,141p nvda-tests/tests/a-link-opens-where-the-setting-says.test.js`.
2. **12-02, task 1 `<read_first>`**: the ranges are 2026-09-18's and moved. `on_init`'s
   frame is built at `wx_app.rs:944` (plan: `:796-830`); `wire_the_way_out` is at `:12881`
   (plan: `:11959-11995`); `show_conversation_as_page` at `:23517` (plan: `:20976-21050`,
   and the re-take says so for this one); `safe_external_url` at `html_renderer.rs:1089`
   (plan: `:860-880`). The re-take corrected three names and left the `<read_first>` list.
   Command: `grep -n 'fn wire_the_way_out\|fn show_conversation_as_page' src/presentation/wx_app.rs; grep -n 'pub fn safe_external_url' src/presentation/html_renderer.rs`.
3. **12-03, task 1 `<read_first>`**: `wx_app.rs:4130-4140`, `:4850-4920`, `:13950-13970`,
   `:2370-2374`, `:5225-5229` are 2026-09-18's; the plan's own re-take gives `4581`, `11256`,
   `20703`, `5293`, `10801`, `5765`, `2543`.
4. **12-03, task 2**: the behaviour says "the six refusal sites" and the criterion asks
   `nothing_chosen(` at least 6, while the re-take says five sites in three wordings. The
   re-take's pattern also missed two refusals of the same kind it did not name:
   `"Nothing is selected in the message list"` at `wx_app.rs:17887` (a selection) and
   `"Choose a folder first"` at `:5072`. The census target will list them, so the six is
   reachable, but the plan should say the number is the census's and not a fixed six.
   Command: `grep -n 'send_refusal' src/presentation/wx_app.rs | grep -i 'select\|choose\|first\|nothing'`.
5. **12-03, `files_modified`**: task 2's `<files>` names
   `tests/progress_is_shown_and_results_are_said.rs` (for `PROGRESS_OPENINGS`) and the
   frontmatter does not.
6. **12-05, premise 2 and task 1 `<read_first>`**: "`names.rs:174`" is given as an
   `extern "system"` site; `src/presentation/accessibility/names.rs` has none (`:170-200` is
   the doc of `set_accessible_name_and_description`). `toolbar_text.rs:79` is right and there
   are thirty others (`grep -rn 'extern "system"' src --include=*.rs`).
7. **12-05, task 2 `<read_first>`**: `wx_settings.rs:1880-1910` holds the Open links choice
   and its sentence but no `set_accessible_name_and_description` call; the calls are at
   `:736` and `:763`.
8. **12-06, task 2 `<read_first>`**: `grep -n 'fn every_setting_is_acted_on'` finds nothing,
   since `every_setting_is_acted_on` is a module (`config.rs:2533`); the two tests are at
   `:2627` and `:3004`. Premise 2's "`mod.rs:902`" is the `CAccPropServices` GUID constant;
   `SetHwndPropStr` is at `:1370` of the same file.
9. **12-06, task 2**: the record "a later Settings panel is painted after its controls exist"
   anchors on `theme::paint(&font_size, palette.main_surface());` in `wx_settings.rs`. If
   the spin control keeps the name `font_size` the anchor holds; if it is renamed the record
   moves. Say which.
10. **12-07, task 2**: `Alt+P` and `Alt+I` already appear in the Contact Edit Dialog table
    (`Add Phone`, `City`, on other tabs). Mnemonics are per visible page so they may
    coexist, but task 3 tells the executor to check only `Alt+I` and `Alt+X`; add `Alt+P`.
    Command: `awk '/Contact Edit Dialog/{f=1} f&&/Alt\+(P|I|X)/{print} f&&/^## /{exit}' docs/KEYBOARD_SHORTCUTS.md`.
11. **12-08, premise 2**: `fn build_calendar_pim_tab` is at `wx_settings.rs:2429`, not
    `:1550`; the page is added at `:578` as the plan says.
12. **12-09, task 1 `<read_first>`**: the once-only pass under
    `SNIPPETS_ARE_THE_FIRST_RELEVANT_WORDS` is in `src/data/message_cache/bodies.rs:372`, not
    `messages.rs`; the grep the plan gives finds it.
13. **12-10, task 2 `<read_first>`**: `grep -n 'Alt+Shift' src/presentation/wx_account_manager.rs`
    finds nothing; the gesture's menu item is at `wx_app.rs:7193` ("Move &Up\tAlt+Shift+Up")
    and its rule at `folder_tree.rs:755`.
14. **12-11, premise 2 and task 1 criterion**: "three unrelated words" is wrong.
    `grep -rniE 'licen[cs]e key|subscription|entitle' src --include=*.rs` answers 202 lines
    for `subscription` (iCal and folder subscriptions) and 23 for `entitle` in ten files,
    among them a type `Entitles` and a field `entitle` at `wx_app.rs:12920-13081`. The
    criterion's own grep (`licen[cs]e key|entitlement`) answers 0 and stays 0, so the
    criterion holds; its parenthetical does not. The design's proposed `Entitlement` seam
    sits one letter from a name the tree already uses for the page window's title handoff,
    which the document should say.
15. **12-12 and the README**: `git rev-list origin/main..HEAD --count` is 1 at `19ad8648`,
    since the commit that landed these plans is not pushed; 0 was true at `0ad66e48`.
16. **12-01, task 3**: the changelog entry is conditional on the product changing, which is
    right; the ledger entry's number is not predictable and the criterion reads it from
    `head -7`, which is also right. Noted only because `docs/changelog.md` sits in
    `files_modified` and may end the plan untouched, which the summary should say.

## Passes, per plan, with the evidence

### 12-01
- Premises hold: `activateWindow` at `launch-app.js:132-139` with `AppActivate` and the doc
  "Answers whether Windows agreed" at `:130`; `Alt+Tab` in no case (0); "What the log holds"
  at `README.md:106`; `show_conversation_as_page` at `wx_app.rs:23517`; `page.set_focus()` at
  `:23640` and `:23830`; no `on_activate` handler anywhere under `src/presentation`;
  `fn GetFocus` at `a_marker_counts_at_the_start_of_any_line.rs:100`, `class_name` `:126`,
  `where_keys_go` `:136`; counts 20 and 199; `webview_edge.cpp:1171-1175` is `OnSetFocus`
  calling `MoveFocus`; the new target does not exist.
- The diagnosis names two causes and the one read that separates them (foreground window
  and focused element after activation), and task 1 fixes only if the reading is red.
- Verify commands: three, each can fail, one `--lib` each.

### 12-02 (moved plan; three re-taken commands spot-checked)
- `sed -n 24,40p src/main.rs` answers `parse` at the first line, `EraseAllData`, `Help`,
  `Version`, `Refused` as the plan says; the name grep answers claim `:77`,
  `prepare_data_folder` `:92`, `init_logging` `:103`, `how_to_start` `:138`, offer `:151`,
  `erase_all_data` `:246`, `log_crash` `:419`; the `SeparateWindow` arm at `:13275-13278`
  inside `follow_the_link_the_page_posted` at `:13257`.
- The constants at `opening_links.rs:167` and `:173`, the test at `:404-407` asserting
  "next build"; `wx_settings.rs:1899` and `:1903`; counts 24, 11, 12; `'page'` at
  `accessibility.yml:159`; `page_window.rs` absent; ledger 566 in both halves naming
  11-11.2 and the two constants, and this plan's text names 566.
- The absence claim for "next build" is made with `tr '\n' ' '` first (task 2's criterion),
  which is the wrapped-line reading the rule asks for; the constant does wrap.

### 12-03 (moved plan; three re-taken commands spot-checked)
- The seven-call census answers 105, 114, 79, 47, 33, 17, 12; the refusal grep answers the
  five sites (4581, 11256, 20703 "Choose a message first"; 5293; 10801) and no "No
  conversation selected"; the two phrases at `:5765` and `:2543`; the three builders at
  `checking_on_a_schedule.rs:233`, `mail_sync.rs:182`, `trying_again.rs:97`;
  `PROGRESS_OPENINGS` at `:38` (six) and `:346`; every file count as listed;
  `check.sh --suites-for` exists at `:173`.
- Task 1's target stays red until task 2 and says so, which is the honest stopping point.

### 12-04
- `show_about_dialog` `:26155`, `build_about_dialog` `:26169`, four `StaticText` lines with
  "Copyright 2024-2026 Wixen Mail Contributors", 380 by 260; `LICENSE:3` reads "Copyright
  (c) 2026 Pratik Patel"; `wixen.app` only in the earcon design; `check_about` at
  `theme_reach.rs:887` with the "four StaticText and a button" doc, 7 tests; `'about'` at
  `accessibility.yml:156`, `ScanTarget::About` at `scan_target.rs:145`; `HyperlinkCtrl`
  unused in `src` (0) and present in wxdragon at `hyperlink_ctrl.rs:76` and `:83`; no
  "Send Feedback" in `wx_app.rs` (0); the Help menu at `:7514-7550` holds Contents, Load
  Sample Mailbox, About; `tests/docs_links.rs` exists.
- The control kind is chosen by a measurement over MSAA, not in the plan.

### 12-05
- `queue_for_sending` `:17054`; `allowed_for(&account.id).mail` at `:20979`, `:22141`,
  `:22372`; `allowed_for` at `allowed.rs:912`; `QueuedOutboxMessage` at `mod.rs:910`;
  `mask_email` `logging.rs:163`, the file naming comment `:125`; `"Win32_` lines 8 with a
  ninth bare match in a comment at `Cargo.toml:369`; no `nvda.exe`, `RtlGetVersion` or the
  rest anywhere in `src` (0); the privacy table at `privacy.md:173`, "What is never sent"
  `:479`, Logging `:678`; Application Control at `KEYBOARD_SHORTCUTS.md:464`; the three
  issue templates; `QueuedOutboxMessage {` outside comments is 1 in `wx_app.rs`;
  `Ctrl+Shift+F` free in the page and the code; `paths.rs` 19 tests, `logs_dir` at `:93`;
  `ID_ABOUT` arm at `:5846`; "How to report something" at `ALPHA_TESTING.md:640`.
- The one-path decision (extract the row composition, one gate) is held by a criterion
  that can fail and a record.

### 12-06
- The interval field `tf_with_description("Check &Interval (min):"` at
  `wx_account_manager.rs:1639`, filled at `:1855`, read back with `.clamp(1, 60)` at
  `:1254`, the `spin` closure at `:1488` with `with_max_value(3650)` and
  `set_accessible_name(&c, ...)`; Font size `wx_settings.rs:1109-1112` and `.clamp(8, 72)`
  at `:3402`; Default reminder `:2472-2473` and `.min(1440)` at `:3674`; Mark read after a
  `Choice` at `:1511` and `:1834`; the two `SpinCtrl`s at `:1337` and `:1380`;
  `MarkRead::ALL` seven at `reading_habits.rs:31`, `offered_index` at `:99`, 25 tests;
  ten `SpinCtrl::builder` sites in the four files as listed; Send Later's fields from the
  item form's builders at `wx_send_later.rs:132` and `:142`; `UDM_GETBUDDY`,
  `SetHwndPropStr`, `IAccPropServices` nowhere in `src` (0, checked joined and per line);
  `windows` 0.62.2 in `Cargo.lock` with `SetHwndPropStr` at `Accessibility/mod.rs:1370`;
  no `set_increment` on wxdragon's `SpinCtrl`; ledger 408-425 as the plan sorts them
  (twelve spinner text fields `todo`, 411/415/418/423 value elements, 416/417 other);
  counts `names.rs` 11, `wx_item_form.rs` 14, `config.rs` 68.
- A feature switch on a crate already in the tree leaves `Cargo.lock` unchanged, so the
  criterion `git diff --stat Cargo.lock` empty is right.

### 12-07
- Basic Info at `wx_managers.rs:1276-1312` with the Birthday `TextCtrl` at `:1297` and the
  Favourite box named outright at `:1305-1310` (label `&Favorite`); `ContactEntry` at
  `mod.rs:591` with no prefix, middle or suffix; `ensure_column_exists` in use at `:1682`
  and `:1693`; the vcard `N:{}` writer at `contacts.rs:795-810`, `BDAY` at `:900` and
  `:1514`; `YEAR_LEFT_OUT` at `types.rs:211` as `"--"`; `birthday_from_google` at
  `contacts_sync.rs:770`; `is_an_address` at `links_in_text.rs:199`; `GoogleName` with
  given and family only at `google_api.rs:65-95`; the Graph contact with `display_name`,
  `surname` and no `title`, `middle_name`, `generation`; the Hopper and van der Berg
  comment at `contacts_sync.rs:735-750`; counts 281, 103, 37, 57, 44, 137, 79, 14;
  Contact Edit Dialog Accelerators at `KEYBOARD_SHORTCUTS.md:1119`;
  `tests/mark_as_read_says_which_way_it_will_go.rs` exists.
- The phone rule adds no dependency and the README lists the library question as Pratik's.

### 12-08
- `build_time_fields` `:901`, `build_date_fields` `:829`, `as_stored_time` `:1535`,
  `hour_from` `:1560`, `ask_for` `:271`, `OfferedTime` `:211`, `build_control` `:969`;
  `wx_item_form::ask_for` at `managers.rs:1213` and `:2733`; the reminder lead read with
  `load_stored()` at `managers.rs:1904-1905`; no `event_length` in `config.rs` (0);
  `default_reminder_minutes` at `:542-543`; the older-file test at `:1450`; The Event Window
  at `KEYBOARD_SHORTCUTS.md:172` and The Reader Window at `:191`, so the awk range in task
  3's criterion is well formed.
- The key's arrival is measured on the buddy before the handler is bound, as decision 12
  says.

### 12-09
- `signatures` table at `mod.rs:1830`, `work_done_once` at `:2379`; the six store functions
  at `signatures.rs:9-147` and none reads across accounts; 2 tests and 23;
  `get_default_signature(&id)` once in `wx_app.rs` at `:16601`; `manage_signatures` reading
  `get_signatures_for_account` at `managers.rs:190-215`; the shell at `wx_managers.rs:3875`
  and the default handling at `:3903-3905`; `account_choice` at `wx_compose.rs:809-820`
  named "From account"; `with_signature` at `:312` through `sign_off::attach` (`sign_off.rs:52`,
  `carries_one` `:67`, `split` `:76`); ledger 490 is the box's NVDA reading.
- The pass keeps every account's assignment before clearing any, and the version stays,
  both as the README decides and the roadmap criterion says.

### 12-10
- The submenu built once from `TO_BEGIN_WITH` at `wx_app.rs:6927-6941` with the comment
  claiming a rewrite on load (read joined across its wrap: "they are rewritten from the
  account's own labels when those load"), and `labels_menu` at no other site than the
  append at `:7346`; `ORDER BY name` at `tags.rs:72-78`; `LABEL_IDS` and `at_number` at
  `wx_app.rs:4097-4100` and `tagging.rs:149`; the `tags` table at `mod.rs:1802`;
  `ID_TAG_MGR` at `:137`, `:5597`, `:7449` ("Ta&gs..."); "Tag Manager" at
  `wx_managers.rs:3659`; `reordering::moved` at `account_order.rs:37` and
  `favourites.rs:224`; Labels at `KEYBOARD_SHORTCUTS.md:755`; counts 12 and 10; wxdragon's
  `find_item_by_position` (`menu.rs:312`) and `get_label` (`menuitem.rs:159`) exist for the
  target's reading.
- The disagreement (Ctrl+2 says Work, applies Later) is a finding of the plan's own, and
  the changelog is told to name it.

### 12-11
- `docs/plans/` holds four dated designs; `every_number_carries_its_command_and_its_date.rs:596-613`
  dates `docs/plans` by name; `ed25519-dalek`, `keyring` and `ring` in `Cargo.lock`;
  `update_download::verify` at `:858`; `allowed_for` `:912` and `SETTINGS_SECTION` `:186`;
  `sound_scheme.rs` and `sound_scheme_import.rs` exist; the four document targets at 74,
  9, 6 and 25 tests. Documents only; no source, test or record touched, held by a criterion
  over `git diff --stat`.

### 12-12
- 12 plans on disk; 83 listening items in section A; LIST ticks 24 and 3 open;
  `Current Plan: 0` and `Total Plans in Phase: 12` at `STATE.md:3181-3182`; the ledger head
  at 566, 531, 35; the phase 11 row reads 29/29 with 29 plans and 29 summaries on disk.
- The plan counts ticks against the row by hand, which is decision 18 and the check
  `CLAUDE.md`'s completion-marks paragraph says it lacks.

## Roadmap and requirements
- The phase 12 entry carries the eleven requirement ids the README names, twelve criteria
  mapped one per plan, the plan list with twelve unticked lines, and the scope note.
  `**Requirements**` matches the union of the plans' `requirements` fields exactly.
- Phase 13 (GAP-01 to GAP-13) and phase 14 (REAL-01, REAL-02) have entries, requirements
  with `[D]` and `[S]` lines, traceability rows, no plan directory and no progress row;
  the order inside each is written as the planner's for Pratik to confirm.
- Requirement coverage: FOUND-20 12-01; LIST-19 12-02; LIST-11 12-03; ALPHA-01 12-04;
  ALPHA-02 12-05; EDIT-01 12-06; EDIT-02 12-07; EDIT-03 12-08; EDIT-04 12-09; EDIT-05
  12-10; ALPHA-03 12-11; all eleven on 12-12. No requirement of the phase is uncovered.

## The checker's own blind spot, tested
Every absence claim accepted above was searched a second way that tolerates a wrap:
`tr '\n' ' ' < file | grep` for the "next build" sentence (12-02), for the labels menu's
"rewritten from the account's own labels" comment (12-10), for the "Separate windows arrive"
line (12-02), for `UDM_GETBUDDY` and its two siblings (12-06), and for the "No message
selected" refusals (12-03). None of the claims changed under the second reading; the two
refusals the 12-03 re-take missed (warning 4) were found by widening the pattern to every
`send_refusal` naming a choice, not by the wrap.

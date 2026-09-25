# Phase 13, group 4: automation. Research

**Researched:** 2026-09-24, against `main` at `630e2a67` (phase 12 closed), version
`1.0.0-alpha.1`, `guards/guards.toml` holding 1,088 records by the TOML reader,
`.planning/WINDOWS.md` at 609 entries (550 open, 59 fixed, its front matter).
**Requirements:** GAP-09 (#58), GAP-11 (#60), GAP-12 (#61).
**Domain:** the rules engine, saved searches, commands over a selection, menus rebuilt from
data, keys by position.
**Confidence:** HIGH on what the tree holds (every seam below was opened and read this
session); MEDIUM on the plan shapes (they follow 12-10's pattern closely, but the set
runner in plan AUT-6 is a refactor of the largest file in the tree); LOW on key choices,
which are Pratik's.

How to read the tags. **(stated)** means read in a file or an issue this session, with the
place. **(derived)** means inferred from what was read, not run. Nothing here was built or
run; no package is proposed.

## Summary

All three features sit on seams the tree already has, and none needs a new package. Saved
searches are stored in `saved_searches` and `saved_search_questions` and read in creation
order; the labels work of 12-10 is an exact template for giving them a stored order, a key
per position and a submenu rebuilt from data. Quick Steps and Run Rule Now both need the
same missing piece: a way to carry out several rule actions over a set of existing messages
through the gated paths the set commands of 11-07 already use, with one sentence at the end.
That runner is the centre of this group and should be built once.

Two findings change the plans. First (derived, high confidence, not run): a rule whose
action is "Add a label" cannot work today. The rule stores the label's name
(`FilterAction::AddTag(String)` from the typed value), and `carry_out` passes that name to
`add_tag_to_message(message_id, tag_id)`, whose `tag_id` is a foreign key into `tags(id)`,
while every label id in the tree is `tag-<uuid>` or `<account>:<keyword>` and foreign keys
are switched on for every connection. The insert fails, `apply_rules` logs "A rule could not
be carried out", and no label arrives. Quick Steps and Run Rule Now would inherit this, so a
small fix plan comes first. Second (derived): a rule's Mark as read and Flag are written to
this computer only; nothing in the arrival path queues them for the server, so running a
rule on demand through `apply_rules` would change the list and not the mailbox. The
on-demand runs should carry out their actions through the set commands' paths
(`spawn_server_change`, `move_or_copy_here_first`, the delete arm), which do reach the
server and are gated.

A third finding is outside this group but decides its order: a block is a rule
(`src/application/blocking.rs:3-10`), and GAP-06's "a block that moves what is already
here, said with the count" is GAP-12's run over existing mail under another name. GAP-06
comes earlier in Pratik's order, so whichever plan comes first should build the runner and
the count once.

**Primary recommendation:** ten plans, one per wave: three for saved searches on 12-10's
pattern, one defect fix for rule labels, four for Quick Steps with the shared set runner
inside them, and two for Run Rule Now reusing that runner. The set runner is built in the
Quick Steps plans because Pratik's order puts Quick Steps before rules on a folder.

## Architectural Responsibility Map

This is a desktop program, so the tiers are the project's own layers.

| Capability | Primary tier | Secondary tier | Rationale |
|---|---|---|---|
| Saved search order, keys, menu lines | `application::saved_searches` (pure) | `data::message_cache::saved_searches` (position column) | 12-10 put the order and the menu's words in `application::tagging` and the column in `data::message_cache::tags`; the same split here |
| The tree gesture on a saved search row | `presentation::folder_tree::what_the_gesture_moves` | `presentation::wx_app::move_the_chosen_row` | The row's identity decides, testable without a window (`folder_tree.rs:755-791`) |
| A search from scratch | `application::saved_searches` (name, join, folder rules) | `presentation::wx_app` and `wx_managers::show_rule_manager_dialog` | The conditions window already exists and is reused |
| Quick Step as data | `application::quick_steps` (new, pure) | `data::message_cache::quick_steps` (new tables) | Same vocabulary as rules (`FilterAction`, `settle`) |
| Carrying actions out over a set | `presentation::wx_app` (the set paths live there) | `application::choosing_messages` (sentences, bounds) | The gated server paths are private to `wx_app.rs`; the pure plan of what to do is not |
| What a rule would change in a folder | `application` (new pure module) | `data::message_cache::saved_searches::messages_a_saved_search_reads` | The count is a pure function over cached messages |
| The gate | `service::outward::permitted` with `application::allowed::allowed_for` | none | Every write already meets it there (`wx_app.rs:21363-21371`, `:22526-22530`) |

## User constraints

No `CONTEXT.md` exists for phase 13; the directory was created for this file. The binding
constraints are Pratik's order of 2026-09-24 as given in the brief (keyboard basics,
reading mail, provider features, then automation: saved searches, Quick Steps, rules on a
folder, then import and export) and `CLAUDE.md`. The ones that bite this group:

- Red and green on every change; a red commit names its tests in `Fails-until-green:`
  trailers, lands on a branch, and names the count check bare when a test is added to a file
  a record names.
- No test added to `wx_app.rs`, `managers.rs`, `config.rs` or `wx_settings.rs` (phase 12's
  cost rule, README `:421-431`, which phase 13 should carry); readings of those go in new
  integration targets with their own records.
- Every key lands in `docs/KEYBOARD_SHORTCUTS.md` in the same commit; every announcement is
  bounded and says what bounds it.
- Mnemonic letters allocated once per menu or dialog, by search, before a plan names one.
- Schema changes additive: `CREATE TABLE IF NOT EXISTS`, `ensure_column_exists`.
- Network code tested against parsing and error mapping, never live servers; what writes
  says it is experimental where the person sees it.
- Plans that change what is spoken or shown push their branch and open a pull request
  (Pratik's standing OK of 2026-09-23); others do not.
- The files written by rule go into every plan's file list before waves are compared, so
  these ten plans are one per wave.

<phase_requirements>

## Phase requirements

| ID | Description (REQUIREMENTS.md `:5841-5873`) | Research support |
|---|---|---|
| GAP-09 | Saved searches reordered, given a key, saved from scratch; two pages corrected | Plans AUT-1 to AUT-3; the position pattern from 12-10; the gesture's decision function; the conditions window |
| GAP-11 | Quick Steps: a named multi-action command on a key over the selected messages | Plans AUT-4 to AUT-8; `settle`, `choosing_messages`, the set paths |
| GAP-12 | A rule run over a folder on demand, saying first how many messages it would touch | Plans AUT-9 and AUT-10; `messages_a_saved_search_reads`, `FilterEngine::matches`, the runner from AUT-6 |

</phase_requirements>

## 1. What the issues ask

**#58, GAP-09 (stated, `gh issue view 58`, no comments).** Audited 2026-09-15 as
implemented, with "What remains, all small": "1. Saved searches keep creation order and
cannot be moved: Alt+Shift+Up and Down rearrange accounts and pinned folders only, a
saved-search row answers nothing (`folder_tree.rs:774`). 2. Save This Search has no key
(`docs/KEYBOARD_SHORTCUTS.md:501` says none); menu mnemonics only. 3. A search cannot be
saved from scratch: Save This Search refuses unless a search has just run
(`wx_app.rs:7548`); Edit Conditions edits only one that exists. 4. By design and worth
stating: a saved search reads only mail cached here and only text still held, and shows
the newest 500 (changelog `:3758-3767`), which #24 changes. 5. Documents drift ..." The
requirement's `[D]` lines: "The reordering gesture the tree has; a key per search on the
pattern labels take in EDIT-05; Save as Search from an empty box; the pages corrected by
dating."

Two readings of "a key". The issue's point 2 is a key for the Save This Search command. The
`[D]` line is a key per saved search, on the labels' pattern. They are different features;
the plans below build the `[D]` line and list the command's key as a question.

**#60, GAP-11 (stated, no comments).** "A Quick Step is a rule with several actions and a
name, run on demand over the selected messages (which #30 makes a set) rather than on
arrival; a manager beside the Filter Manager; a bound key from a small reserved range, said
in `docs/KEYBOARD_SHORTCUTS.md`; the announcement naming the step and how many messages it
touched. Depends on the 'run rules over a folder' issue for the on-demand runner." Goal:
"reusing the existing rules engine rather than a second one."

**#61, GAP-12 (stated, no comments).** "A 'Run on this folder' command in the Filter Manager
and on the This Folder menu; a dry run that counts matches and asks before acting ('This
rule would move 214 messages in Inbox. Run it?'); the same runner Quick Steps would use;
every action through the existing gated write paths so the Allowed Changes answer still
governs it." The `[D]` line: "Run Rule Now on the rule editor and the Action menu; the count
said before the run with a way to stop; the run through the same arms a check uses."

The `[D]` line's "the same arms a check uses" and the issue's "existing gated write paths"
pull two ways, and section 3 resolves it: match through the check's arms, carry out through
the set commands' paths.

## 2. What the tree already has

Every line below was opened this session unless marked otherwise.

### Saved searches

| What | Where | Reading |
|---|---|---|
| The pure module | `src/application/saved_searches.rs` (2,779 lines, 76 tests, 3 records) | Module doc `:36-41` already says a search row carries "where it sits in the list", which nothing stores yet (derived) |
| `SavedSearch { id, name, join, questions, folder }` | `saved_searches.rs:1049-1065` | No position field |
| `name_for`, `Naming`, `LONGEST_NAME = 100` | `:962-1026` | The naming rules a search from scratch reuses |
| `what_a_typed_search_asks`, `WhatASavedSearchWillAsk { questions, join, folder }` | `:497-572` | One value for the three halves of a scope, D-2-14 |
| `a_search_in_words` | `:642-658` | The sentence said while naming and when opened |
| `run_over`, `selects` | `:1144-1184` | The one way to run a search; empty question list takes nothing |
| `MOST_RESULTS_SHOWN = 500` | `:889`, used at `wx_app.rs:8090` and `:8118` | Still bounds a saved search's list although #24 is closed (`gh issue view 24`: CLOSED) |
| The tables | `src/data/message_cache/mod.rs:1937-1983` | `saved_searches (id, account_id, name COLLATE NOCASE, all_or_any, folder, created_at, updated_at, UNIQUE(account_id, name))`; `saved_search_questions (search_id, position, field, match_type, pattern, case_sensitive)` |
| The read | `src/data/message_cache/saved_searches.rs:336-370` | `ORDER BY created_at, id`; the doc says the order is creation so renaming does not move a row |
| Create, replace, rename, delete | `saved_searches.rs:95`, `:194`, `:287`, `:306` | |
| The folder read a rule run can reuse | `messages_a_saved_search_reads(account_id, folder_id, text)` at `saved_searches.rs:382` | Everything cached for an account or one folder, deleted mail left out, body text only when asked |
| The order test | `test_the_searches_stay_in_the_order_they_were_made_when_one_is_renamed`, `saved_searches.rs:1054-1080` | Stays true after a position column |
| Tree rows | `src/presentation/folder_tree.rs:714-753` | A "Saved Searches" heading, a branch per account, then the searches in the order read |
| The gesture's decision | `folder_tree.rs:755-791`, `WhatMoves { Account, Pin, Nothing }` | A saved search row answers `Nothing` |
| The test that says so | `test_the_gesture_moves_nothing_on_a_row_that_is_neither`, `folder_tree.rs:1560-1590` | Lists `WhichRow::SavedSearch` among rows that move nothing; rewritten in place by AUT-1 |
| The gesture's dispatch | `wx_app.rs:10225-10243`, `move_the_chosen_row` | `Nothing` refuses with `favourites::WHICH_ROW` |
| The refusal's words | `src/application/favourites.rs:174-176` | "Move Up and Move Down rearrange accounts and pinned folders. Choose an account branch, or a folder under Favourites." Changes when searches move |
| A no-network check reading the gesture | `favourites.rs:518-522`, `THE_COMMANDS` | Names `fn move_the_chosen_row` and `fn move_the_chosen_pin`; a new `fn move_the_chosen_search` belongs in that list |
| One wording for a move | `src/application/reordering.rs:55-104`, `moved` | "Work, 2 of 3.", "Work is already first of 3." |
| Save This Search | `wx_app.rs:8244-8335`, Edit menu at `:6806-6815` ("Sa&ve This Search...") | Refuses with "Search your mail first, and then this will keep that search." when nothing ran (`:8276-8284`) |
| Edit Conditions | `wx_app.rs:8428-8506`, calling `wx_managers::show_rule_manager_dialog` (`wx_managers.rs:3740-3778`) | The window edits questions only; the join and the folder are carried over and cannot be changed anywhere (derived from `the_search_to_write_back`, `wx_app.rs:8397-8414`) |
| The empty-list refusal | `wx_managers::what_a_condition_list_still_needs`, `wx_managers.rs:3690-3695` | |
| The Action submenu | `wx_app.rs:7127-7145` and `:7419-7426` | "Saved Searc&hes" with Edit &Conditions, &Rename, &Delete |
| The context menu | `src/application/context_menu.rs:374-379` | Edit &conditions, &Run this search again, Re&name, &Delete this search |
| Where searches load | `wx_app.rs:19393-19395`, `UIUpdate::SavedSearchesLoaded` | Beside `LabelsLoaded`, which calls `put_the_labels_on_the_menu` (`:19389-19392`); the saved-search menu rebuild goes here |
| Running one | `wx_app.rs:7949-8124`, `run_a_saved_search` | |

### The labels pattern 12-10 set (stated, from its summary and the code)

- A `position INTEGER` column added with `ensure_column_exists("tags", "position", "INTEGER")`
  (`mod.rs:2799-2802`); `create_tag` places a new label last with
  `COALESCE(MAX(position), 0) + 1` (`tags.rs:11-16`); `get_tags_for_account` orders by
  `position, name` (`tags.rs:78-83`).
- `put_labels_in_order(account, ids)` in one transaction (`tags.rs:104-121`).
- `number_the_unnumbered_labels` run on every open, not under a marker, numbering rows with
  no place in the order their keys applied (`tags.rs:123-155`, called at `mod.rs:1451-1457`).
- Pure words: `tagging::what_the_menu_says`, `key_for`, `REACHABLE_BY_KEY = 9`,
  `at_number`, `moved`, `nothing_there` (`tagging.rs:143-251`).
- The menu: `menu_ids!` reserves a block with `NAME[count]` (`wx_app.rs:90-104`,
  `ID_LABEL_PAST_NINE[LABELS_PAST_NINE_ON_THE_MENU]` at `:256`); `rebuild_the_label_menu` and
  `put_the_labels_on_the_menu`, which rebuilds only when the words differ so an open menu is
  not emptied (`:10594-10705`); a list key handler answers a number past the last label
  (`:10642-10670`).
- The manager: `ManagedRow::moved` gives a manager Move &Up and Move Do&wn and
  Alt+Shift+Up and Down (`wx_managers.rs:236-263`, `:4257-4297`); `build_tag_manager` with a
  Key column (`:4227-4254`); the order is written when the manager closes.
- Readings in a new target at zero records:
  `tests/the_label_menu_says_the_labels_an_account_has.rs` (16 readings, 3 records now).

### The rules engine

| What | Where | Reading |
|---|---|---|
| `FilterAction` | `src/application/filters.rs:11-29` | MoveToFolder, AddTag, MarkAsRead, MarkAsUnread, Star, Unstar, Delete, SayFirst |
| `FilterRule` | `filters.rs:41-68` | One condition and one action per rule |
| `FilterEngine::matches`, `rules_matching` | `filters.rs:335-448` | Unknown field answers no; regex bounded to 1 MiB |
| `from_persisted_rule` | `filters.rs:450-480` | Stored strings to the enum; `add_tag` keeps the typed value |
| `Outcome`, `settle` | `filters.rs:514-581` | Several actions settled into one answer; Delete wins and drops the rest; `touches_the_server` is move or delete |
| `apply_rules` | `src/application/mail_sync.rs:1030-1076` | Called by the two sync paths only (`mail_sync.rs:1471`, `pop_sync.rs:335`), over arriving rows |
| `carry_out` | `mail_sync.rs:1081-1112` | Flags, labels and the phrase written to this computer; the move left to `carry_out_the_moves` (`:1142`), which needs a connection |
| `the_folder_a_rule_names` | `mail_sync.rs:1241-1248` | Folder by name, case-insensitive, or by path |
| `say_what_the_rules_did` | `mail_sync.rs:210-242` | The after-the-fact sentences, bounded to two reasons |
| The gate on arrival | `mail_sync.rs:1049-1058` | Move or delete held back when `allowed.mail` is off, counted |
| The Filter Manager | `src/presentation/wx_managers.rs:3048-3088` on `run_manager_loop` (`:295`) | Add, Edit, Delete, Close; no Run, no order |
| The action words | `RULE_ACTIONS`, `wx_managers.rs:3110-3118` | Seven offered; `unstar` is readable but not offered |
| Saving the manager's rows | `src/presentation/managers.rs:380-460` | Rules written when the manager closes |
| Tools menu item | `wx_app.rs:7480-7481`, "Message &Filters..." | |

### The set commands of 11-07 (#30)

| What | Where | Reading |
|---|---|---|
| `Chosen`, `what_the_selection_holds`, `SetCommand`, `reach_for` | `src/application/choosing_messages.rs:38-168` | A conversation row reaches the whole account for marks and labels, this folder for move and copy, the D-07 setting for delete |
| `choosing_messages::Outcome`, `what_was_done` | `:200-262` | "3 messages marked read", "4 messages moved to Archive, 1 not moved" |
| `too_many` | `:308-316` | The 5,000 bound, Select All's |
| `chosen_messages` | `wx_app.rs:10899-10973` | Reads the selection off the control at the key |
| Mark as read over the set | `toggle_read_state`, `wx_app.rs:11008-11090` | Each message written here, then `spawn_server_change(... FlagChange::Read(..))` |
| `spawn_server_change`, `ServerChange`, `FlagChange` | `wx_app.rs:23445-23475`, `:23529` onward | A failed push is kept waiting when the server was not reached (`keep_a_flag_change_waiting`, `:23728`); a label is not queued |
| Move or copy over the set | `move_or_copy_message`, `wx_app.rs:21058`; `move_or_copy_here_first`, `:21308` | The folder dialog, then `AMoveAsked { moving, chosen, into, copying }`; the gate at `:21363-21371` |
| Delete over the set | the `ID_DELETE` arm at `wx_app.rs:5304` onward; the gate at `:22526-22530` | Inline in the event match |
| Labels over the set | `label_the_message`, `wx_app.rs:11459`; `labels_for`, `:11611` | |
| Readings of all seven | `tests/every_command_acts_on_the_selection.rs` (3 records) | Source readings over `what_ships` with planted-fault companions |

### Menus and letters (searched this session)

| Menu or window | Letters taken | Free |
|---|---|---|
| Tools (`wx_app.rs:7429-7567`) | A, N, D, T, W, C, F, K, I, E, B, R, O, P, L, S | G, H, J, M, Q, U, V, X, Y, Z |
| Action (`wx_app.rs:7278-7426`) | R, A, O, F, U, N, X, E, S, I, K, P, D, M, V, C, W, Y, L, G, B, T, H | J, Q, Z |
| Action, This Folder (`wx_app.rs:7150-7276`) | R, O, N, M, D, Y, K, U, W, P, I | A, B, C, E, F, G, H, J, L, Q, S, T, V, X, Z |
| Action, Saved Searches (`wx_app.rs:7130-7145`) | C, R, D | everything else |
| Edit (`wx_app.rs:6760-6815`) | U (Undo Send), T, C, P, A, S, V | B, D, E, F, G, H, I, J, K, L, M, N, O, Q, R, W, X, Y, Z |
| Context menu on a saved search (`context_menu.rs:374-379`) | c, r, n, d | u, w among others |
| Any manager on `run_manager_loop` (`wx_managers.rs:312-345`) | A, E, D, C; U and W when ordered | R, N and others |

`tests/wired.rs:1977`, `test_no_two_items_on_one_menu_claim_the_same_letter`, holds every
menu to this.

### Keys by position (searched this session)

Digit chords bound in the main window: `Alt+1` to `Alt+3` (View panes, `wx_app.rs:6878-6888`),
`Ctrl+Shift+1` to `Ctrl+Shift+6` (modules, `:6971-6996`), `Ctrl+0` to `Ctrl+9` (labels).
`Ctrl+Alt+0` to `Ctrl+Alt+3` are the composer's headings, a separate window
(`KEYBOARD_SHORTCUTS.md:926-929`). Searched with `grep -rnoE
'(Ctrl|Alt|Shift)(\+(Ctrl|Alt|Shift))*\+[0-9]\b'` over the shortcuts page, `wx_app.rs` and
the accessibility layer, and `grep -rn '\\tCtrl+Shift+[0-9]\|\\tAlt+[0-9]\|...' src`: nothing
binds `Alt+4` to `Alt+9` or `Ctrl+Shift+7` to `Ctrl+Shift+9`. The only other digit handler in
the presentation layer is the label fallback (`wx_app.rs:10655`), which ignores Shift and
Alt. `docs/KEYBOARD_SHORTCUTS.md:1436-1438` says keys are fixed; a key by position is not a
customisation and that sentence stays true.

### Absences, with the search that came back empty

- No quick step anywhere in the code: `grep -rni 'quick.step\|quickstep' src tests` came
  back empty this session.
- No position on saved searches: the `CREATE TABLE` at `mod.rs:1939-1950` has none, and
  `grep -n 'ensure_column_exists("saved_searches"' src/data/message_cache/mod.rs` is empty.
- No caller of `apply_rules` outside the syncs: `grep -rn 'apply_rules' src` finds
  `mail_sync.rs`, `pop_sync.rs` and a test in `filters.rs:1643-1667`.
- No name-to-id lookup for a rule's label: `grep -rn 'fn tag_named\|fn tag_by_name\|fn
  find_tag\|fn label_named\|by_name' src/data/message_cache/tags.rs
  src/application/tagging.rs` is empty.
- No push of a rule's flags: the only writer of the waiting flag queue outside tests is
  `wx_app.rs:23728` (`grep -rn 'keep_a_flag_change_waiting' src`).

## 3. Findings that change the plans

**F1. A rule's Add a label fails (derived, high confidence, not run).** `from_persisted_rule`
builds `FilterAction::AddTag(value)` from the stored text (`filters.rs:455-457`); the rule
editor stores what was typed. `carry_out` calls `cache.add_tag_to_message(id, tag)`
(`mail_sync.rs:1100-1102`), whose second parameter is `tag_id` (`tags.rs:211-222`), a foreign
key to `tags(id)` (`mod.rs:1848-1855`) with `PRAGMA foreign_keys=ON` on every connection
(`mod.rs:1390`). Label ids are made as `id_or_new(&row.id, "tag")` (`managers.rs:190-191`) or
`format!("{account_id}:{}", label.keyword)` (`wx_app.rs:11624`), never the bare name. So the
insert is refused, `apply_rules` logs "A rule could not be carried out" (`mail_sync.rs:1073`)
and counts nothing. `tests/a_rule_can_change_how_a_row_is_announced.rs:293` runs such a rule
and asserts only the sound count, so nothing catches it. Plan AUT-4 takes it red first.

**F2. A rule's Mark as read and Flag reach this computer only (derived, medium).** The
comments say they go out "later through the flag sync, which has its own gate"
(`filters.rs:538-544`, `mail_sync.rs:283-286`), but the flag sync reads flags from the
server (`mail_sync.rs:1515-1540`) and nothing queues a rule's change. On a server without a
"changed since" answer the next sync asks about every held message and would put the old
flag back (derived from `:1516-1521`, not run). For on-demand runs this decides the route:
carry out through `spawn_server_change`, which pushes and queues. Whether arrival rules have
the same gap is a question for a ledger entry, not for this group to fix.

**F3. The runner is shared with GAP-06 (stated).** "A block is a rule"
(`blocking.rs:3-10`). GAP-06's "a block that moves what is already here, said with the
count" is the count and the run of plans AUT-9, AUT-6 and AUT-10 over a sender's mail. GAP-06
is earlier in Pratik's order. The orchestrator should decide which group builds the runner;
this file assumes this group does and names the seam so GAP-06's plans can depend on it or
build it first.

**F4. The flag and the move of one message race (derived).** `spawn_server_change` resolves
the message's folder on a worker when it runs (`wx_app.rs:23600-23606`), and
`move_or_copy_here_first` moves the row here at once. A step that marks read and then moves
can have the flag's worker read the new folder with the old number. The same race exists
today when somebody presses M and then Ctrl+Shift+V quickly. The runner in AUT-6 sends each
message's flags before its move is recorded, and the plan proves the order against the
loopback server the `moves_waiting` tests already drive (`moves_waiting.rs:61-67`).

**F5. `MOST_RESULTS_SHOWN` still caps a saved search at 500 (stated).** #58 point 4 says
#24 changes it; #24 is closed and the cap is still read at `wx_app.rs:8090`. Not in GAP-09's
`[D]` lines; listed for Pratik.

**F6. The drifted pages have moved (stated).** `KEYBOARD_SHORTCUTS.md:124-125` ("one row per
saved search"; there is a branch per account) and `:691` ("Rename or delete"; Edit
Conditions is there too); the older changelog limit at `docs/changelog.md:5441-5445` ("A
saved search remembers the folder it was made in, but not whether you asked for Subject Only
or From Only") against the fix at `:4748-4760`. Both entries are under `[Unreleased]` (it
starts at `:7`; the next heading is `:12990`).

## 4. Proposed plans

Ten plans, one per wave, because every one writes `docs/changelog.md`,
`.planning/WINDOWS.md` and `guards/guards.toml`, and seven write
`src/presentation/wx_app.rs`. Ids are working names for the orchestrator to number. Each is
2 to 4 tasks. The last task of every plan is the pages, the changelog, the ledger and the
four completion marks. No plan installs a package.

Record counts below were taken this session with the TOML reader
(`tests_last_seen` per file) and the count check's own pattern for tests:
`saved_searches.rs` (application) 3 and 76; `data/message_cache/saved_searches.rs` 2 and 27;
`data/message_cache/mod.rs` 12 and 23; `folder_tree.rs` 5 and 97; `wx_app.rs` 119 and 199;
`wx_managers.rs` 14 and 44; `managers.rs` 54 and 137; `filters.rs` 2 and 49;
`mail_sync.rs` 14 and 149; `reordering.rs` 0 and 3; `choosing_messages.rs` 3 and 18;
`context_menu.rs` 5 and 18; `tagging.rs` 1 and 19; `tests/wired.rs` 19 and 77;
`tests/every_command_acts_on_the_selection.rs` 3;
`tests/the_label_menu_says_the_labels_an_account_has.rs` 3;
`tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs` 1.

### AUT-1: Saved searches keep an order somebody chooses (GAP-09)

- **Closes:** GAP-09's "the reordering gesture the tree has".
- **Files:** `src/data/message_cache/mod.rs`, `src/data/message_cache/saved_searches.rs`,
  `src/application/saved_searches.rs`, `src/application/favourites.rs` (the `WHICH_ROW`
  sentence and `THE_COMMANDS`), `src/application/context_menu.rs`,
  `src/presentation/folder_tree.rs`, `src/presentation/wx_app.rs`, `guards/guards.toml`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/changelog.md`,
  `.planning/WINDOWS.md`.
- **Depends on:** nothing in this group.
- **Spoken or shown:** yes ("Invoices, 2 of 4.", and the refusal's words change). Push and a
  pull request.
- **Size:** M, three tasks.
- **Tasks.** (1) The column: `ensure_column_exists("saved_searches", "position", "INTEGER")`,
  `create_saved_search` placing a new search last, the read ordered by `position, created_at,
  id`, `put_saved_searches_in_order(account, ids)` in one transaction, and
  `number_the_unnumbered_saved_searches` on open in `created_at, id` order so every tree keeps
  the order it had (the labels kept theirs by name because that was their old order; here the
  old order is creation). (2) The gesture: `WhatMoves::SavedSearch { account, id }` from
  `what_the_gesture_moves`, `move_the_chosen_search` in `wx_app.rs` over
  `reordering::moved` within the account's own searches (readable and unreadable together,
  the order `put_back_together` reads), the tree read back, one sentence; `WHICH_ROW` names
  saved searches; `fn move_the_chosen_search` joins `favourites.rs`'s `THE_COMMANDS`, since
  nothing here may reach a server; the context menu gains "Move this search &up" and "Move
  this search do&wn" (u and w are free there). (3) Pages and marks.
- **The failing test that starts it:**
  `data::message_cache::saved_searches::tests::test_the_order_written_is_the_order_read_back`
  (new), a companion for rows from before numbered in creation order, and
  `presentation::folder_tree::tests::test_the_gesture_moves_nothing_on_a_row_that_is_neither`
  rewritten in place so a saved search row is no longer in its list, with the positive case
  added to `test_the_gesture_moves_an_account_on_an_account_row_and_a_pin_on_a_pinned_row`
  rather than as a new test. Adding tests to `saved_searches.rs` (data, 2 records) flags the
  count check, named bare in the red trailer.

### AUT-2: A key per saved search, and the Saved Searches menu lists them (GAP-09)

- **Closes:** GAP-09's "a key per search on the pattern labels take in EDIT-05".
- **Files:** `src/application/saved_searches.rs`, `src/presentation/wx_app.rs`,
  `tests/the_saved_searches_menu_says_the_searches_an_account_has.rs` (new),
  `guards/guards.toml`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`,
  `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-1 (the key follows the stored order), and Pratik's answer on the keys
  (question 1).
- **Spoken or shown:** yes. Push and a pull request.
- **Size:** M, three tasks.
- **Tasks.** (1) Pure words in `application::saved_searches`: `what_the_menu_says(names)`,
  `key_for(position)` over the reserved range, `REACHABLE_BY_KEY`, `nothing_there(number)`,
  the same shape as `tagging.rs:143-251`, ampersands doubled. (2) The menu: a block of ids
  with `menu_ids!`'s `NAME[count]`; the Saved Searches submenu rebuilt from the active
  account's searches in `UIUpdate::SavedSearchesLoaded` (`wx_app.rs:19393`), rebuilding only
  when the words differ; each item selects the search's tree row and runs it, exactly as
  Enter on the row; a key past the last search answered with "There is no saved search 6"
  rather than silence, the way labels answer; then a separator and the three existing
  commands. (3) Pages and marks.
- **The failing test that starts it:** the new target reading the built menu bar
  (`build_menu_bar` is public since 12-10) after a load of three searches, one renamed, one
  moved: the items' words and keys, and a companion planting a tenth search with a key. Plus
  `application::saved_searches::tests::test_the_first_searches_have_keys_and_the_rest_do_not`
  (count check named bare).

### AUT-3: A saved search made from nothing, and the two pages corrected (GAP-09)

- **Closes:** GAP-09's "Save as Search from an empty box" and "the pages corrected by
  dating".
- **Files:** `src/application/saved_searches.rs`, `src/presentation/wx_app.rs`,
  `src/presentation/wx_managers.rs` (only if the join choice joins the conditions window,
  question 3), `src/presentation/scan_target.rs` and `.github/workflows/accessibility.yml`
  (a scan target for the new window, on phase 12's pattern), `tests/a_saved_search_can_be_made_from_nothing.rs`
  (new), `guards/guards.toml`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`,
  `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-2 (both write the Saved Searches submenu).
- **Spoken or shown:** yes. Push and a pull request. The workflow file makes the hook answer
  `all` on that commit.
- **Size:** M, three tasks.
- **Tasks.** (1) Pure: `a_search_from_nothing(name, join, folder, questions) -> SavedSearch`
  refusing an empty list with the existing sentence and a name by `name_for`. (2) The window:
  Save This Search with nothing run opens a New Saved Search window instead of refusing (name,
  where to look, every or any condition, then the conditions list through
  `show_rule_manager_dialog`), and "&New Saved Search..." on the Saved Searches submenu (N is
  free there); written with `create_saved_search`; the created sentence
  (`saved_searches::created`). (3) The pages, dated rather than rewritten:
  `KEYBOARD_SHORTCUTS.md:124-125` says a branch per account, `:691` names Edit Conditions and
  New Saved Search and the order; a dated sentence under the older changelog limit at
  `:5441-5445` saying it was fixed on the date of `:4748`'s entry; the guide gains a Saved
  Searches section, which it lacks (only `USER_GUIDE.md:888` mentions them).
- **The failing test that starts it:** `application::saved_searches::tests` for the new
  function, and the new target building the window and reading its controls' names on MSAA
  through `AccessibleObjectFromWindow` on a built window, the way phase 12 read its dialogs.

### AUT-4: A rule that adds a label puts that label on (defect under GAP-11 and GAP-12)

- **Closes:** F1; a prerequisite for AUT-5 onward. No requirement line names it; it rides on
  GAP-11 as the plan that makes the shared vocabulary work.
- **Files:** `src/application/mail_sync.rs` (production code only), `src/application/filters.rs`
  or `src/application/tagging.rs` (a `the_label_a_rule_names(labels, named)` beside
  `the_folder_a_rule_names`), `tests/a_rule_that_adds_a_label_labels_the_message.rs` (new),
  `guards/guards.toml`, `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-3 by wave only.
- **Spoken or shown:** yes, labels appear where they did not. Push and a pull request.
- **Size:** S, two tasks.
- **Tasks.** (1) Red: a target running `apply_rules` over a real cache with a rule "Add a
  label: Money" and a label named Money made the way the manager makes it, asserting the
  message carries it and `done.changed` is 1. (2) Green: the rule's name resolved to the
  account's label id, case-insensitive like folders; a name no label has is said, the way a
  missing folder is (`no_folder_of_that_name`, `mail_sync.rs:1255`), and never inserted as an
  id.
- **The failing test that starts it:** the new target, bare in the trailer. Kept out of
  `mail_sync.rs`'s own tests because that file carries 14 records and 149 tests.

### AUT-5: Quick Steps as data (GAP-11)

- **Closes:** GAP-11's storage and words, not yet reachable.
- **Files:** `src/application/quick_steps.rs` (new), `src/application/mod.rs`,
  `src/data/message_cache/quick_steps.rs` (new), `src/data/message_cache/mod.rs`,
  `guards/guards.toml`, `.planning/WINDOWS.md`, `docs/changelog.md` (an additive schema
  note).
- **Depends on:** AUT-4.
- **Spoken or shown:** no, nothing is wired. No push.
- **Size:** M, three tasks.
- **Tasks.** (1) Pure: `QuickStep { id, account_id, name, actions: Vec<FilterAction>,
  position }`; `what_stops_a_step_being_saved(name, actions)` (no actions, a name too long,
  a name another step holds, two actions that contradict each other, a delete beside anything
  else, since `settle` drops the rest); `what_the_menu_says`, `key_for` over the step range;
  `what_a_step_did(name, chosen, outcome)` built on `choosing_messages::what_was_done`'s
  pieces, one sentence: "Archive and read: 3 messages marked read and moved to Archive". (2)
  Store: `CREATE TABLE IF NOT EXISTS quick_steps (id, account_id, name COLLATE NOCASE,
  position, created_at, UNIQUE(account_id, name))` and `quick_step_actions (step_id,
  position, action_type, action_value, FOREIGN KEY ... ON DELETE CASCADE)`, the action
  columns holding the same strings a rule holds so `from_persisted_rule`'s arms read both;
  create, replace, delete, order, and the account's steps go when the account does. (3) The
  ledger entry saying the steps exist and nothing runs them until AUT-8, filed in AUT-8's own
  text as `CLAUDE.md` requires.
- **The failing test that starts it:** `application::quick_steps::tests` and
  `data::message_cache::quick_steps::tests`, new modules at zero records, plus the
  older-database case on the pattern of
  `test_a_database_written_before_saved_searches_existed_opens_and_keeps_everything`
  (`saved_searches.rs:1399`).

### AUT-6: One runner for several actions over a set of messages (GAP-11, GAP-12, and GAP-06's block)

- **Closes:** the shared runner #60 and #61 both name.
- **Files:** `src/application/acting_on_a_set.rs` (new, pure: which writes each message gets
  from an `Outcome`, with no-ops dropped), `src/application/mod.rs`,
  `src/presentation/wx_app.rs`, `tests/every_command_acts_on_the_selection.rs` (readings
  rewritten in place if a function moves), `tests/several_actions_reach_the_server_in_order.rs`
  (new), `guards/guards.toml`, `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-5.
- **Spoken or shown:** no by itself (the seven commands say what they said), but it changes
  seven spoken commands' code paths, so push and a pull request so the NVDA runs read them.
- **Size:** M bordering L; three tasks, and if the first task's extraction is larger than
  one sitting, split it at the task boundary.
- **Tasks.** (1) Extract the do-halves of Mark as Read, Star, Label, Move and Delete in
  `wx_app.rs` into functions that take a `Chosen` and an explicit direction and say nothing,
  and have the seven commands call them, their sentences unchanged; the readings in
  `every_command_acts_on_the_selection.rs` stay green or are rewritten in place. (2) The
  runner: `run_these_actions_over(app, cache, chosen, outcome) -> WhatWasDone` settling with
  `filters::settle`, the gate met once per account before anything changes, flags and labels
  first, then the phrase, then the move through `move_or_copy_here_first` or the delete, the
  folder named by a step or rule resolved with `the_folder_a_rule_names` and the label with
  AUT-4's function; one sentence and one `Confirmed` signal. (3) The order proof: the flag of
  a message reaches the loopback server before its move (F4).
- **The failing test that starts it:** the pure module's cases (`application::acting_on_a_set::tests`)
  and the new target's reading that the runner exists, is called by nothing else yet, and
  meets the gate; the order case against the loopback server where the harness allows it from
  a target, or in `moves_waiting.rs`'s tests if the helpers are private there (the executor
  measures which).

### AUT-7: The Quick Step manager (GAP-11)

- **Closes:** GAP-11's "named", with its manager.
- **Files:** `src/presentation/wx_managers.rs`, `src/presentation/managers.rs` (production
  code only), `src/presentation/manager_words.rs` (the word "quick step" and "action"),
  `src/presentation/wx_app.rs` (Tools, "&Quick Steps..."), `src/presentation/scan_target.rs`,
  `.github/workflows/accessibility.yml`, `tests/the_quick_step_manager_says_what_each_step_does.rs`
  (new), `guards/guards.toml`, `docs/USER_GUIDE.md`, `docs/KEYBOARD_SHORTCUTS.md`,
  `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-5.
- **Spoken or shown:** yes. Push and a pull request.
- **Size:** M, three tasks.
- **Tasks.** (1) The manager on `run_manager_loop` with `ManagedRow::moved`, so it has Move
  Up and Move Down and Alt+Shift+Up and Down; columns Name, Key, What it does (the actions in
  words from `shown_action`). (2) The step editor: "&Name:" and a list of actions on the same
  loop (A, E, D, U, W, C are the loop's; N is free), each action edited in a small dialog
  reusing the rule editor's action list and value label (`RULE_ACTIONS`,
  `the_value_label_for`, `what_stops_the_rule_being_saved`), a folder chosen from the
  account's folders rather than typed (question 5); refusals from AUT-5 before OK closes.
  (3) Tools item, scan target, pages, marks. The Tools item's help text carries the
  experimental sentence, as Pause Downloading's does (`wx_app.rs:7552`).
- **The failing test that starts it:** the new target building the manager over three steps
  and reading the rows, the Key column and a move, on the pattern of the label target
  (`tests/the_label_menu_says_the_labels_an_account_has.rs:400-420`).

### AUT-8: Quick Steps on the Action menu and their keys (GAP-11)

- **Closes:** GAP-11's "on a key, over the selected messages" and "one sentence saying what
  it did".
- **Files:** `src/application/quick_steps.rs`, `src/presentation/wx_app.rs`,
  `tests/a_quick_step_runs_over_the_selection.rs` (new),
  `tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs` (only if its list needs a
  surface added), `guards/guards.toml`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`,
  `docs/ALPHA_TESTING.md`, `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-6 and AUT-7, and Pratik's answer on the keys (question 1).
- **Spoken or shown:** yes. Push and a pull request.
- **Size:** M, three tasks.
- **Tasks.** (1) The Action submenu "&Quick Steps" (Q is free on Action; see question 7 for
  GAP-06's letter) rebuilt from the active account's steps with their keys, then "&Edit Quick
  Steps...", rebuilt on load and after the manager closes, only when the words differ. (2)
  Each item reads the selection with `chosen_messages` (the reach being this folder when the
  step moves or deletes, the whole conversation otherwise), refuses above `too_many`, runs
  AUT-6's runner, and says `what_a_step_did` once; a key past the last step says so. (3)
  Pages: a Quick Steps section on the shortcuts page and the guide, the alpha page's line that
  a step reaches the server through Allow Changes and has never met a real one; marks.
- **The failing test that starts it:** the new target over the built menu bar (the items and
  keys) and source readings that each item reads the selection, meets `too_many` and calls the
  runner, each with a planted-fault companion.

### AUT-9: What a rule would change in a folder, counted first (GAP-12)

- **Closes:** GAP-12's "saying first how many messages it would touch", as pure code.
- **Files:** `src/application/running_a_rule_now.rs` (new), `src/application/mod.rs`,
  `guards/guards.toml`, `docs/development/measurements.md` (the count's time over a large
  folder, if taken), `.planning/WINDOWS.md`, `docs/changelog.md` (only if something shows).
- **Depends on:** AUT-4 (a label rule must count as the run will do it).
- **Spoken or shown:** no, nothing is wired until AUT-10. No push.
- **Size:** S, two tasks.
- **Tasks.** (1) `what_a_rule_would_change(rule, messages, folder, labels) -> Vec<(id,
  Outcome)>` through `FilterEngine::matches` and `settle`, dropping what changes nothing (a
  read message a rule marks read, a message already in the folder a rule files into, a label
  already on); a disabled rule counted as the person asked (question 6). (2) The question's
  words: "This rule would move 214 messages in Inbox to Archive. Run it?", one clause per
  kind of change, the singular right through `how_many`, the bound said when the count passes
  `too_many`'s 5,000 (question 4), and "This rule would change nothing in Inbox." with no
  question at all.
- **The failing test that starts it:** `application::running_a_rule_now::tests`, new module at
  zero records, with the no-op cases as rows.

### AUT-10: Run a rule over a folder from the Filter Manager and the This Folder menu (GAP-12)

- **Closes:** GAP-12's "Run Rule Now on the rule editor and the Action menu; the count said
  before the run with a way to stop; the run through the same arms a check uses", on the
  reading in section 3.
- **Files:** `src/presentation/wx_managers.rs` (an optional extra button on
  `run_manager_loop` for the Filter Manager, "&Run on a Folder..."; R is free there),
  `src/presentation/managers.rs` (production code only), `src/presentation/wx_app.rs` (This
  Folder, "Run a Ru&le on This Folder..."; L is free there), `src/application/context_menu.rs`
  (the same entry on a folder row, if Pratik wants it there too),
  `tests/a_rule_runs_over_a_folder_when_asked.rs` (new), `guards/guards.toml`,
  `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/ALPHA_TESTING.md`,
  `docs/changelog.md`, `.planning/WINDOWS.md`.
- **Depends on:** AUT-6 and AUT-9.
- **Spoken or shown:** yes. Push and a pull request.
- **Size:** M, three tasks.
- **Tasks.** (1) From This Folder: choose a rule of the folder's account with
  `wx_managers::choose_from_list` (`wx_managers.rs:5127`), read the folder with
  `messages_a_saved_search_reads(account, Some(folder), text)` on a worker (text only when the
  rule reads it), count with AUT-9, ask with Run and Don't Run (the question in the dialog's
  own text, which NVDA reads), then hand the matched messages to AUT-6's runner as a `Chosen`.
  (2) From the Filter Manager: the button ends the modal with the chosen row, the manager's
  changes are saved first, then the same flow with the folder asked through the folder window.
  (3) Pages, the alpha page's line, marks. What is said after is `say_what_the_rules_did`'s
  shape or `what_was_done`'s, one sentence.
- **The failing test that starts it:** the new target with source readings that both doors
  count before they run, ask before they write, and call the one runner; and a built Filter
  Manager reading that the Run button is there with its name on MSAA.

### Order and waves

AUT-1, AUT-2, AUT-3, AUT-4, AUT-5, AUT-6, AUT-7, AUT-8, AUT-9, AUT-10, one per wave. AUT-9
can move anywhere after AUT-4, since it touches no file the others touch except the three
written by rule. If GAP-06's plans build the runner first (F3), AUT-6 becomes a reading that
the runner GAP-06 built is the one Quick Steps call, and shrinks to one task.

## 5. Packages

None. Every piece is in the tree: SQLite through `rusqlite` for the tables, wxdragon for the
menus and dialogs, the rules engine for matching, `regex` already bounded in
`filters.rs:430-433`. The alternatives the audit would weigh (a Windows API, writing it
ourselves) come out as writing it ourselves in every case, and it is small: two tables, a
pure module each, and menus on the pattern 12-10 wrote. The dependency-audit skill was not
invoked because there is no candidate to audit. Nothing waits on Pratik's package
confirmation.

## 6. What cannot be verified here, and what cannot be finished

**Needs a real account or server (phase 14 and ledger entries):**

- A Quick Step or a rule run that moves, deletes, flags or labels reaching a real IMAP
  server, in order, and a refusal undone here. The loopback server proves the order and the
  refusal path, not a provider.
- Gmail's labels and folders under a step that moves (Gmail's "move" is a label change);
  POP accounts, where a move is local.
- A folder of thousands of matches at a real provider's pace.

Each of these is a ledger entry of kind `unrun-verify` in the plan that builds it, and each
surface that writes says it is experimental where the person sees it: the Tools item's help
text for Quick Steps, the run question's last sentence ("Moving mail this way has not been
tried against a real mail server yet."), and a line on `docs/ALPHA_TESTING.md`, on the
pattern of `allowed::DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL`.

**Needs the tester's ear (the `[S]` lines):** the saved search order and a move said; the
key heard on the Saved Searches submenu; the New Saved Search window in order; a Quick Step
heard as one act with one sentence; the count question and the result heard as one act.

**Cannot be finished in this group, gated honestly:** the Save This Search command's own key
(#58 point 2) is not in the `[D]` lines and is a question, not a stub; the 500 cap (F5) is a
question; the join of an existing search stays uneditable unless Pratik says otherwise
(question 3); arrival rules' flags not reaching the server (F2) is a ledger entry for
whoever owns rules next, not fixed here.

## 7. Accessibility per feature

**Saved search order (AUT-1).** No new control. The gesture is Alt+Shift+Up and
Alt+Shift+Down on the row, the tree's one gesture (D-31); the This Folder menu's Move Up and
Move Down items (`wx_app.rs:7249-7258`, ids `ID_MOVE_UP`, `ID_MOVE_DOWN`) reach it too, and
their help text, "Move the chosen account or pinned folder one place up", gains saved
searches in the same commit, and
the context menu gains two entries on free letters u and w. Announced through
`reordering::moved`: "Invoices, 2 of 4.", "Invoices is already first of 4.", at High
priority as a pin's move is (`wx_app.rs:10292-10294`). Focus stays on the moved row; the
tree read back must land the cursor on it, which the plan measures on a built tree. The
Alt+Shift layout warning already on the page applies (`KEYBOARD_SHORTCUTS.md:1147-1166`).
Bounded: one sentence per key press.

**Saved search keys (AUT-2).** Menu items carry the key after a tab, which NVDA and
Narrator read with the item; no item takes a mnemonic, as label lines take none. The key
lands the tree cursor on the row and runs it, so what is said is what Enter says today: the
scope sentence, then "Invoices, 12 messages". A key past the last search is answered in
words. Keys checked against the shortcuts page above; the page gains a table in the same
commit (`tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs`).

**New Saved Search window (AUT-3).** Controls: a text box "&Name for this search:" (the
existing label, `wx_app.rs:8310`), a choice "&Look in:" (the account or one of its folders),
a choice "Find messages that &match:" with "every condition" and "any condition", then the
conditions window as it is. Letters N, L, M plus the dialog's buttons, to be re-read on the
built dialog; the conditions window keeps its own. Each control named with
`set_accessible_name` at the handle focus reaches, read on MSAA and UI Automation on the
built window, per the global rule about checking the focused handle. The empty-list refusal
is the existing sentence. Error prevention: nothing is written until the conditions window
closes with at least one condition.

**Quick Step manager and editor (AUT-7).** A native report list (row count and position
reach both channels from Windows' own provider, the reason `run_manager_loop` exists,
`wx_managers.rs:3730-3735`), columns Name, Key, What it does. Buttons on the loop's letters.
The step editor's "&Name:" on N. The action dialog reuses the rule editor's letters,
Action on A and the value on V or H (`KEYBOARD_SHORTCUTS.md:743-758`); a folder chooser
button if Pratik picks it, on a free letter found by grep in that dialog. Refusals said
before OK closes and left in the dialog, as rule refusals are.

**Quick Steps menu and run (AUT-8).** "&Quick Steps" on Action (Q). Items with their keys.
One sentence after a run, naming the step and the count, at Normal priority, with the
`Confirmed` signal once, whatever the number of messages (guardrail 5); refusals say what to
do ("Select fewer.", "Choose a message first."). A step whose folder is gone says so by name
and changes nothing, rather than doing half.

**Run a rule on a folder (AUT-10).** The question is a native message dialog whose text NVDA
reads on open; two buttons, Run and Don't Run, with Don't Run focused when the rule deletes
(question 8). While a large run is under way the status bar says so once; the result is one
sentence. The Filter Manager's new button on R, "&Run on a Folder...", read on MSAA on the
built manager; the scan target `filters` already walks that window
(`scan_target.rs:73`, `:280`).

**Mnemonic collisions to watch across groups.** Action has J, Q and Z free. Quick Steps take
Q. GAP-06's Report Junk would want J. Nothing else is left but Z, so a later item on Action
needs a submenu or a letter moved (question 7). The Edit menu's U is Undo Send, which GAP-02's
Undo would also want; that is GAP-02's, noted because AUT-3 does not add to Edit.

## 8. Questions only Pratik can answer

1. **Which keys?** Saved searches and Quick Steps each want a key per position, and the
   main window has few digit chords left. Free and safe: Alt+4 to Alt+9 (six) and
   Ctrl+Shift+7 to Ctrl+Shift+9 (three). Ctrl+Alt with a digit is free but is AltGr on many
   keyboards, where it types characters such as braces in the Notes editor, which sits in
   the main window. Alt+Shift with a digit collides with the layout switch the page already
   warns about. **Recommendation:** Quick Steps on Ctrl+Shift+7, 8 and 9 (Outlook puts Quick
   Steps on Ctrl+Shift and a digit, and #60 asked for "a small reserved range"), saved
   searches on Alt+4 to Alt+9, the rest from their menus.
2. **What does a saved search's key do?** **Recommendation:** exactly what Enter on its row
   does, with the tree cursor moved onto the row so the tree and the list agree.
3. **Can a search's "every or any" be changed after it is made?** Today nothing can change
   it. **Recommendation:** the from-nothing window offers it, and Edit Conditions gains the
   same choice in the same plan; otherwise an "any" search made from the box can never become
   "every".
4. **A rule over a folder with more than 5,000 changes.** **Recommendation:** refuse above
   5,000 with the count and the bound, the same number every set command uses; running it
   again after the first 5,000 have moved does the next.
5. **Folder in a Quick Step: chosen or typed?** Rules type a folder's name. **Recommendation:**
   chosen from the account's folders and stored by path, since a step is run on purpose and a
   typing mistake there is a move to nowhere.
6. **Can a switched-off rule be run on demand?** **Recommendation:** yes; choosing it by hand
   is the reason to run it.
7. **Action menu letters.** Q for Quick Steps, J for Report Junk, and then only Z. **Recommendation:**
   accept both; the next Action item goes on a submenu.
8. **Which button is focused in the run question?** **Recommendation:** Run, except when the
   rule deletes, where Don't Run is focused.
9. **Should Quick Steps and saved searches be per account?** Rules and labels are.
   **Recommendation:** per account, because the folders and labels a step names are an
   account's.
10. **The 500 cap on a saved search's list (F5).** #24 is closed and the cap stands.
    **Recommendation:** a separate issue, not in this group.
11. **A key for Save This Search itself (#58 point 2).** **Recommendation:** none; the
    `[D]` line is keys per search, and the command sits beside Search.
12. **Which group builds the runner, this one or GAP-06 (F3)?** **Recommendation:**
    whichever comes first in the phase; by your order that is GAP-06, so its plan builds
    AUT-6 and AUT-9's pieces and this group reuses them.

## Common pitfalls

1. **Keys and menu disagree.** 12-10 found Ctrl+2 said Work and applied Later. The menu and
   the key must read one list in one order; one pure function and a reading on the built menu
   bar, as the label target does.
2. **An empty list disarms a check.** A census of searches or steps that iterates over none
   passes. Every reading gets a companion that plants a fault, as
   `tests/every_command_acts_on_the_selection.rs` does.
3. **A number past the end is silent.** The menu never sees a key with no item; the list
   answers it, as `answer_the_label_keys_the_menu_cannot` does (`wx_app.rs:10642-10670`).
4. **Rebuilding an open menu.** Rebuild only when the words differ (`wx_app.rs:10677-10705`).
5. **A block of ids that spills.** Reserve ids with `NAME[count]`; 12-10 found Help pages
   running New and Delete because a block was one id wide.
6. **Local-only rule writes (F2).** On-demand runs go through the set paths, not
   `carry_out`.
7. **The flag and the move race (F4).** Prove the order against the loopback server.
8. **Guard records go stale inside the plan.** Run the `--remeasure` remedy whenever the
   count check prints it, detached; `wx_app.rs` carries 119 records.
9. **Anchors move with extracted blocks.** AUT-6 extracts from `wx_app.rs`; every record
   whose `before` text lies inside a moved block is named in the plan and rewritten
   (`CLAUDE.md`, the second reading of the register).
10. **A plan's letter is a claim.** Re-grep the menu or dialog's `&` letters when the plan is
    written and again when it is executed.

## Validation architecture

| Property | Value |
|---|---|
| Framework | cargo test; integration targets under `tests/` |
| Quick run | `cargo test --lib application::saved_searches::` (one `--lib` per call, joined with `&&`) |
| Full suite | `scripts/check.sh all`, once, in the phase's closing plan |

| Req | Behaviour | Test type | Command | Exists |
|---|---|---|---|---|
| GAP-09 | order stored and read | unit | `cargo test --lib data::message_cache::saved_searches::` | file yes, cases no |
| GAP-09 | gesture moves a search | unit | `cargo test --lib presentation::folder_tree::` | rewritten in place |
| GAP-09 | menu and keys | integration | `cargo test --test the_saved_searches_menu_says_the_searches_an_account_has` | no, Wave 0 |
| GAP-09 | from nothing | unit and integration | `cargo test --lib application::saved_searches:: && cargo test --test a_saved_search_can_be_made_from_nothing` | no |
| GAP-11 | label rule works | integration | `cargo test --test a_rule_that_adds_a_label_labels_the_message` | no |
| GAP-11 | step data | unit | `cargo test --lib application::quick_steps:: && cargo test --lib data::message_cache::quick_steps::` | no |
| GAP-11 | runner and order | unit and integration | `cargo test --lib application::acting_on_a_set:: && cargo test --test several_actions_reach_the_server_in_order` | no |
| GAP-11 | manager, menu, keys | integration | `cargo test --test the_quick_step_manager_says_what_each_step_does && cargo test --test a_quick_step_runs_over_the_selection` | no |
| GAP-12 | the count | unit | `cargo test --lib application::running_a_rule_now::` | no |
| GAP-12 | both doors | integration | `cargo test --test a_rule_runs_over_a_folder_when_asked` | no |
| all | keys documented | integration | `cargo test --test a_key_is_documented_where_the_surface_that_binds_it_is` | yes |
| all | letters unique | integration | `cargo test --test wired` | yes |

## Security domain

| ASVS category | Applies | Control |
|---|---|---|
| V4 access control | yes | Every server write through `service::outward::permitted(allowed_for(account).mail, ...)`, met once per account before anything changes |
| V5 input validation | yes | Names through `name_for` and a step's own bound; a rule's regex bounded to 1 MiB; a phrase bounded to 40 characters; SQL through bound parameters only |
| V2, V3, V6 | no | No sign-in, session or cryptography in these features |

| Threat | STRIDE | Mitigation |
|---|---|---|
| A step or rule moving mail on an account whose changes are off | Elevation of privilege | The gate before any change; held-back counted and said, as arrival rules do |
| A run touching another account's folder of the same path | Tampering | Folders resolved within the rule's or step's own account; a path is not unique across accounts (`saved_searches.rs:421-435`) |
| A regex from an imported rule hanging the window | Denial of service | The existing size limit; the count on a worker, never the UI thread |
| A flood of announcements over a large run | Denial of service (of the listener) | One sentence per run, one signal |

## Sources

Primary, read this session: `CLAUDE.md`; `.planning/ROADMAP.md:1463-1513`;
`.planning/REQUIREMENTS.md:5755-5885`; the phase 12 README and 12-08 and 12-10 plans and
12-10's summary; `gh issue view 58`, `60`, `61`, `62`, `24`, `30` with JSON output; the
source files and lines cited above; `guards/guards.toml` through the TOML reader;
`.planning/WINDOWS.md`'s front matter and saved-search entries.

## Metadata

- The tree's seams: HIGH, each opened and read.
- F1: HIGH as a reading, not run; AUT-4's first commit is the run.
- F2 and F4: MEDIUM, derived from code paths, not run.
- Plan sizes: MEDIUM; AUT-6 is the one most likely to need a split.
- Keys and letters: HIGH that they are free today; the choice is Pratik's.
- Valid until the next plan merges into `main`; re-take every line number when the plans are
  written.

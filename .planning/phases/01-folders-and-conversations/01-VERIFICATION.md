---
phase: 01-folders-and-conversations
verified: 2026-09-03T00:00:00Z
status: human_needed
score: 8/8 roadmap success criteria verified
behavior_unverified: 0
overrides_applied: 0
verified_against: 377c713
re_verification:
  previous_verified: 2026-08-31
  previous_status: gaps_found
  previous_score: 7/8 (1 partial)
  gaps_closed:
    - truth: "Each account is its own branch, ordered by the user and moved with the keyboard, so two POP accounts no longer show two folders called Inbox with nothing to tell them apart (roadmap Success Criterion 3)."
      closed_by: 01-14-PLAN.md
      evidence: >
        folder_tree_updates (wx_app.rs:10056) builds from the_accounts_in_the_tree
        (wx_app.rs:9991), which reads every account from load_accounts in the stored
        ordinal order and appends the open one when the accounts table has no row for
        it. folder_tree::rows is called with that whole slice (wx_app.rs:10182). Seven
        tests in presentation::wx_app::the_tree_holds_every_account pass, run for this
        report, including test_two_inboxes_called_the_same_thing_are_two_rows_a_caller_can_tell_apart.
    - truth: "The changelog's known-limit note still blamed a blocker that had shipped."
      closed_by: 01-14-PLAN.md
      evidence: >
        docs/changelog.md:1137-1141 now says the note is overtaken and points at the
        entry describing the multi-account tree. The 'waiting on the shared local
        folders work' wording is gone from the file.
  gaps_remaining: []
  regressions: []
  new_findings:
    - id: W1
      severity: warning
      title: "The guard cited as proof that Thread View is enabled examines zero lines and cannot fail"
      file: src/presentation/wx_app.rs
      line: 25438
    - id: W2
      severity: warning
      title: "WINDOWS.md ledger entry 8 is still open although 02.1-06 closed it"
      file: .planning/WINDOWS.md
    - id: W3
      severity: warning
      title: "Two entries in the changelog's [Unreleased] section contradict each other about two accounts of one name"
      file: docs/changelog.md
    - id: I1
      severity: info
      title: "Sidebar > Delete says folder removal is not built while This Folder > Delete Folder removes it"
      file: src/presentation/wx_app.rs
deferred:
  - truth: "A conversation root arriving after a message that names it is merged."
    addressed_in: "Phase 3"
    evidence: "ROADMAP.md Phase 3, 'Inherited from phase 1': 'A conversation root arriving after a message that already names it is not merged, so three of six arrival orders over such a set merge. One table, one index, one writer.'"
  - truth: "Gmail mail archived with no label is counted in its conversation."
    addressed_in: "Phase 3"
    evidence: "ROADMAP.md Phase 3, 'Inherited from phase 1': 'Gmail mail archived with no label vanishes from a conversation count, because D-08 excludes All Mail by folder rather than by message identity. One extra predicate in one query.'"
  - truth: "next_local_uid does not hand out a number already in use after the range wraps."
    addressed_in: "Phase 3"
    evidence: "ROADMAP.md Phase 3, 'Inherited from phase 1', third bullet. Not reachable in any database this program can currently produce."
  - truth: "AppConfig::allowed_per_account is offered by a screen."
    addressed_in: "Phase 6"
    evidence: "deferred-items.md routing header, 2026-08-31: 'two to phase 6 (the per-account permission nothing offers, the reminder that opens over typing)'. Named in STORED_AND_OFFERED_BY_NOTHING at src/data/config.rs:1728, which predates this phase."
  - truth: "A reminder alert does not open over somebody who is typing."
    addressed_in: "Phase 6"
    evidence: "deferred-items.md routing header, 2026-08-31. The decision about what a reminder is for is Pratik's, not an executor's."
human_verification:
  - test: "Arrow onto a nested folder (Archive/2026, which reads as 2026 under Archive) with NVDA running."
    expected: "NVDA announces the level from the native TreeCtrl hierarchy, without the level being spelled into the label text."
    why_human: "The structure is verified: folder_tree::nested builds depth from folders.parent_id and fill_the_tree calls append_item under a stack-tracked ancestor TreeItemId, so it is real native nesting rather than indentation. Only a live screen reader run proves what is spoken. This is why FOLDER-02 is correctly Pending rather than Complete."
  - test: "With two accounts set up, arrow through the folder tree and listen to the two account branches, then to the two rows called Inbox."
    expected: "Each branch names its account, and the two Inbox rows are tellable apart by ear. Two accounts you have given the same name read their addresses as well."
    why_human: "Proved of the updates the tree is rebuilt from, which is where the gap was, and of the disambiguation rule (so_no_two_accounts_read_alike, 02.1-08). Not proved of a running window, and nothing here has run against a real account. 01-14's own coverage entry D1 carries human_judgment: true for exactly this."
  - test: "Pin a folder, then arrow onto the Favourites group with NVDA running."
    expected: "The group announces itself as a group, and the pinned copy is tellable from the folder's own row in its account branch."
    why_human: "Ledger entry 1, open since 2026-08-30: FOLDER-03's last criterion is satisfied structurally only. group_text and favourite_rows are tested; what is spoken is not."
  - test: "Have mail arrive into an open folder in conversation view while the cursor is on a different row, with NVDA running."
    expected: "The repainted row is silent to somebody not standing on it, and the cursor does not move."
    why_human: "Ledger entry 6, open. What is proved is the decision, which rows changed and that refresh_item is used rather than set_item_count. Whether repainting one row of a virtual wxListCtrl is silent to NVDA is a question about a real screen reader on a real window with real mail arriving, and nothing here has run against a real account."
  - test: "Open the View menu and switch Thread View on and off in a folder holding conversations."
    expected: "The item is available rather than greyed out, the check state matches, and the list collapses to one row per conversation and comes back."
    why_human: "The decision layer is fully tested (55 tests in presentation::view_state, including the toggle, the count in conversation mode, the selection round-trip and the sort surviving the switch). The paint callback that reads state.showing is a closure inside the window builder, which wxWidgets' one-application-per-process limit puts out of reach of any test in this crate, so it is proved by reading the source. See finding W1: the guard that was supposed to prove the item is not disabled proves nothing."
---

# Phase 1: Folders and conversations Verification Report

**Phase Goal:** A user can shape and work through their mail by its own structure: an account
they can tell from the next one, folders they can make and manage, nested the way the server
nests them, favourites at the top, and conversations collapsed to one row.

**Verified:** 2026-09-03, against `377c713`
**Status:** human_needed
**Re-verification:** Yes. The first pass ran 2026-08-31, found criterion 3 partial, and plan
01-14 was written to close it. Phase 2, phase 2.1 and a run of tooling work have landed since.

## What the first pass found, and what closed it

The 2026-08-31 report recorded criterion 3 as the one partial of eight, and it was right about
the tree it read. `folder_tree::rows` had taken a slice of accounts since 01-05 and had never
been handed more than one. All eleven call sites of `folder_tree_updates` passed a single
`account.id`, and `UIUpdate::FoldersLoaded` called `delete_all_items()` before rebuilding, so
the sidebar was wholly replaced by one account's rows on every refresh. Two POP accounts each
holding an `Inbox` could not be seen side by side. The first pass also found the changelog's
own explanation for this stale: it still blamed the shared local folders work, which had landed
in the same phase.

That report is superseded rather than wrong, and it is kept here as the record of what was
measured on the day. Plan 01-14 closed both halves. What follows is the state of the tree now.

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria, checked against the tree)

| # | Truth (roadmap Success Criterion) | Status | When it became true | Evidence |
|---|---|---|---|---|
| 1 | A user creates, renames, moves, marks read, empties and deletes a folder from the tree by keyboard; a server folder is refused with a reason when mail writes are off; a local one is not gated. | VERIFIED, with the scope stated below | This phase (01-01, 01-04, 01-09) | `create_mailbox`, `rename_mailbox` and `delete_mailbox` are at `src/service/protocols/imap.rs:907, 944, 976`, each calling `self.may_i(...)` before building the wire command, so a refusal never reaches the server. All six menu commands are declared, menu-wired and handled in `wx_app.rs`. The local half is genuinely ungated by the server gate: `empty_the_chosen_folder` asks `application::emptying::what_will_happen`, which hands a local path straight to `local_folders::deleting` and never reaches `may_i` or a server (`emptying.rs:169`). The `allowed` flag it does read is `allow_deleting_here`, the message-deletion permission, applied the same way on both sides. |
| 2 | `Archive/2026` reads as `2026` nested under `Archive`, level from the native tree control; collapse state survives a restart, keyed by identity. | VERIFIED (mechanism). The announcement is item 1 under Human Verification | This phase (01-03, 01-05, 01-06) | `folder_tree::nested` (`folder_tree.rs:1071`) builds parent/child depth from `folders.parent_id`; `fill_the_tree` calls `tree.append_item(&parent, ...)` under a stack-tracked ancestor (`wx_app.rs:13876-13879`), so it is real native nesting and not indentation in the label. Collapse is keyed on `WhichRow::stored()`, which spells an account as `account{SEP}{id}` and a folder as `folder{SEP}{len}{SEP}{account}{path}` (`folder_tree.rs:142-182`) — an id and a path, never a label. `test_the_view_and_the_collapsed_state_restore_together` passes, run for this report. **Not regressed by 02.1-08:** that plan changed what an account branch is *called*, not what it *is*; `so_no_two_accounts_read_alike` rewrites `AccountInTheTree::name` only, and identities are built from `id`. The changelog says the same at line 127: "Nothing about where the rows sit changes, so a tree you had collapsed opens the way it did before." |
| 3 | Each account is its own branch, ordered by the user and moved with the keyboard, so two POP accounts no longer show two folders called Inbox with nothing to tell them apart. | **CLOSED. VERIFIED** (the by-ear half is item 2 under Human Verification) | Arrived after the first pass, in 01-14; strengthened by 02.1-08 | `folder_tree_updates` (`wx_app.rs:10056`) calls `the_accounts_in_the_tree` (`wx_app.rs:9991`), which reads every account from `load_accounts` in the ordinal order D-14 stores and appends the open account when the accounts table has no row for it. Folders, saved searches, parents and gone-marks are read per account and joined, with parents and gone-marks keyed on `(account, path)` so two accounts holding an `INBOX` are two entries. `folder_tree::rows(&accounts, ...)` gets the whole slice at line 10182. Seven tests in `the_tree_holds_every_account` pass, run for this report, including `test_two_inboxes_called_the_same_thing_are_two_rows_a_caller_can_tell_apart`, `test_the_branches_are_drawn_in_the_order_the_accounts_are_kept_in` and `test_moving_an_account_redraws_the_tree_it_has_just_reordered`. 02.1-08 made this stronger than 01-14 left it: `so_no_two_accounts_read_alike` (`folder_tree.rs:566`) appends the address to *both* branches when two accounts share a name, case-insensitively, and leaves a unique name alone. |
| 4 | Sent/Outbox/Drafts/Junk/Trash are one each under "On this computer"; an existing database is migrated message by message with nothing removed until landed, and a count reported. | VERIFIED | This phase (01-07) | `SHARED_BY_EVERY_ACCOUNT` holds 5 entries and `FOR_IMAP` is empty per D-18 (`local_folders.rs:124, 163`). `merge_local_folders` (`shared_folders.rs:115`) runs once from `MessageCache::new` at `mod.rs:1239`, non-fatal, and the report is surfaced through `UIUpdate::CommandAnswered` at `wx_app.rs:569`. Eleven `shared_folders` tests pass, run for this report, including `test_a_merge_that_stops_partway_has_landed_what_it_moved_and_lost_nothing` and `test_a_folder_that_still_holds_mail_is_never_put_away`. |
| 5 | A user pins a folder; it stays in a group at the top of the tree across a restart; never touches a server; appears in both places. | VERIFIED (mechanism). The group announcement is item 3 under Human Verification | This phase (01-08) | `pin_or_unpin_the_chosen_folder` (`wx_app.rs:8980`) calls `cache.pin_row`/`unpin_row` only, a local SQLite write with no `Allowed` or server call on the path. `favourite_rows` is the first thing `folder_tree::rows` pushes, above `ALL_INBOXES` (`folder_tree.rs:615`), omitted when nothing is pinned. **Not regressed by 02-08's saved-search branches:** `test_two_accounts_pinned_inboxes_sit_under_a_branch_each_rather_than_side_by_side`, `test_unpinning_leaves_the_account_branchs_row_untouched`, `test_a_pinned_copy_belongs_to_the_account_of_the_folder_it_copies` and `test_there_is_no_favourites_group_when_nothing_is_pinned` all pass, run for this report. |
| 6 | The View menu's thread view is enabled, and switching it shows one row per conversation with every column answering about the conversation, not its newest message. | VERIFIED, but see finding W1 — its stated proof proves nothing | This phase (01-11, 01-12); the sort half corrected by 02.1-06 | The **enabled** half holds on evidence independent of the guard that was supposed to prove it: `ID_THREAD_VIEW` is an `append_check_item` at `wx_app.rs:5695` with a live handler at 4037, and the string `enable(false)` appears in the whole of `wx_app.rs` exactly once, inside a test's own filter literal. Exhaustive absence over the file is a stronger reading than the guard gives. The **switching** half is tested at the decision layer: 55 tests in `presentation::view_state` pass, run for this report, covering the toggle, the count in conversation mode, the selection round-trip both ways and the sort surviving the switch. The **columns** half is wired for real: `message_rows::conversation_cell_text` is called from the virtual text callback at `wx_app.rs:1116` and `MessageColumn::conversation_sort_expression` builds the `ORDER BY` at `messages.rs:192-199`. Ledger entries 3 and 4, which recorded both as written-and-uncalled, are correctly marked fixed. 02.1-06 corrected the message-list Safety sort, which had disagreed with the conversation rule about what "worse" means. |
| 7 | A message arriving into an open folder joins its thread without the folder being reopened, including a late message merging two existing trees. | VERIFIED, with the one-directional gap routed to Phase 3. The by-ear half is item 4 under Human Verification | This phase (01-02, 01-13) | `upsert_message` calls `merge_what_this_message_connects` at `messages.rs:943`, on every insert path. The production paths are real, not test-only: `mail_sync.rs:1039` calls `upsert_messages`, which loops through `upsert_message` in one transaction (`messages.rs:1288-1297`), and `pop_sync.rs:278` calls it directly. The disclosed direction gap is pinned by a passing test named for it, `a_root_arriving_after_the_message_that_names_it_is_left_out_of_the_merge` (`thread_identity.rs:627`), and is routed to Phase 3 rather than left as a phase 1 gap. |
| 8 | The five settings this phase adds are each reachable and operable from a real settings screen; no sixth ungoverned setting is added. | VERIFIED | This phase (01-06, 01-09, 01-11, 01-12) | All five fields (`unread_on_a_parent`, `empty_reaches_subfolders`, `mark_read_reaches_subfolders`, `a_conversation_reaches`, `deleting_a_conversation_row`) are present in `AppConfig` and each has a matching control in `wx_settings.rs`. `test_every_setting_somebody_can_change_is_read_by_something` passes, run for this report. `STORED_AND_OFFERED_BY_NOTHING` at `config.rs:1728` still holds exactly one name, `allowed_per_account`, which predates this phase and is routed to Phase 6 — so nothing added since has grown the exception list. Unlike W1, this guard fails loud: `stored_setting_names` uses `.expect("the settings struct")`, so a moved anchor panics rather than passing over nothing. |

**Score: 8/8 truths verified.** 0 behavior-unverified. Five items need a human, four of them a
screen reader; none of them is a gap in the work.

### Criterion 1's scope, stated more completely than the first pass did

The first pass named two disclosed limits on criterion 1. There are four, and the two it missed
are the same shape. `why_the_folder_cannot_be` (`wx_app.rs:7382`) refuses **all four** write
commands on a folder kept on this computer, each with its own sentence: making
(`FOLDERS_ON_THIS_COMPUTER_ARE_NOT_MADE_YET`), renaming, moving and deleting. Nested creation
under a server folder still lands flat.

This does not change the verdict, and the reasons are worth stating rather than assumed. These
are gates with sentences, not stubs: each names what is not built and what to do instead
("Folders on the server can be renamed"), which is the pattern CLAUDE.md's third guardrail asks
for, and all four are in `docs/changelog.md`. The criterion's second sentence — "a local one is
not gated" — is a claim about the *server-write* gate, and that claim is exactly true: emptying
and marking read run through `local_folders::deleting` and never consult `may_i`. What is worth
saying plainly anyway is the shape of the hole: **the folders on this computer are a fixed set
of five, and nobody can add to them.** For a POP-only account, whose folders are all local, the
Folder menu's four write commands each answer with a sentence.

### Deferred Items

Not phase 1 gaps. Each is routed and the routing is in the roadmap or in the deferred-items
header, both dated 2026-08-31.

| # | Item | Routed to | Evidence |
|---|---|---|---|
| 1 | A conversation root arriving after a message that names it is not merged (ledger entry 7) | Phase 3 | ROADMAP.md Phase 3, "Inherited from phase 1", bullet 2 |
| 2 | Gmail mail archived with no label vanishes from a conversation count | Phase 3 | ROADMAP.md Phase 3, "Inherited from phase 1", bullet 1 |
| 3 | `next_local_uid` hands out 0 after the range wraps | Phase 3 | ROADMAP.md Phase 3, "Inherited from phase 1", bullet 3. Not reachable in any database this program can produce |
| 4 | `allowed_per_account` is honoured and offered by nothing | Phase 6 | deferred-items.md routing header. Predates this phase |
| 5 | A reminder alert opens over somebody who is typing | Phase 6 | deferred-items.md routing header. The decision is Pratik's |

Two further phase 1 items are deliberately in no phase and stay that way. The spellcheck test
that fails about one full library run in five through a Windows COM call made twice is diagnosed
only as far as reading, so writing a criterion for it would be pretending otherwise. And
01-13's note that its own order-independence criterion could not be met as written is guidance
for the next plan of that shape, not work.

### Phase 1 items that a later phase closed

Checked against the tree, not taken from the summaries that claim them.

| Item | Closed by | Evidence in the tree |
|---|---|---|
| Criterion 3: the tree draws one account at a time | 01-14 | `the_accounts_in_the_tree`, `wx_app.rs:9991`; seven passing tests |
| Two dialogs leak a wxdragon registry entry per row (ledger, deferred-items) | 02.1-05 | The exception list on `nothing_hangs_off_the_control` is empty, and the comment at `folder_tree.rs:1222` says why: both files hold their rows in a vector beside the control now |
| `messages.message_id` holds two spellings (ledger entry 8) | 02.1-06 | `backfill_message_identifiers` exists at `messages.rs:1082` and is called from `MessageCache::open` at `mod.rs:1265` |
| Sorting by Safety orders by the alphabet | 02.1-06 | `how_bad_the_safety_word_is!` used by both arms |
| Ten checks in `tests/wired.rs` read a prefix of `wx_app.rs` | 02.1-01 | The naive `#[cfg(test)]` cut is gone; `wired.rs:1791-1806` records what replaced it |
| `.planning/intel/context.md` and `IMPLEMENTATION_STATUS.md` describe folder management as unbuilt | 02.1-03 | Phase 2.1 criteria 5 |
| Nothing exercises `ask_about_the_folders_that_have_gone` | 02.1-07 | Phase 2.1 criterion 6 |
| Account branches read a full email address aloud, always | 02.1-08 | `so_no_two_accounts_read_alike`, `folder_tree.rs:566` |
| Branch rows offer a folder's context menu | 02.1-08 | `context_menu.rs:245-246`: `AccountBranch` and `AccountInAGroup` have their own entry lists |

### Regressions

**None found.** Every criterion the first pass passed still passes, and the two places most at
risk were checked against the tree rather than assumed:

- **Row identity survived 02.1-08.** That plan changed `AccountInTheTree::name`, which feeds the
  label. `WhichRow::stored()` is built from ids and paths. Collapse state, pins and the folder
  id map are all keyed on `stored()`, so a tree somebody had collapsed opens the way it did.
- **Row resolution survived 02.1-05.** The two dialogs it changed, `wx_destination.rs` and
  `wx_thread_view.rs`, are the Move Folder destination picker and D-01's conversation view.
  Both now use `tree_walk::rows_in_walk_order` and a parallel vector instead of the control's
  own item data, and the guard that used to except them now reads every file in the layer.

### Findings

Three warnings and one note. None of them makes a criterion false. All four are the class this
project takes seriously: a check that reads as maintained and is not, and a document that
describes what you have as missing.

**W1 (warning). The guard cited as proof that Thread View is enabled examines zero lines and
cannot fail.** `test_the_thread_view_item_is_not_disabled_on_the_way_up` (`wx_app.rs:25438`)
reads the shipping half of the file and does this:

```rust
.skip_while(|line| !line.contains("find_item(ID_THREAD_VIEW)"))
.take(6)
.filter(|line| line.contains("enable(false)"))
```

`find_item(` occurs exactly once in the whole of `wx_app.rs`, and that occurrence is the test's
own string literal above. So in `ships()` there is no anchor, `skip_while` consumes every line,
`take(6)` gets nothing, and the assertion passes having looked at nothing.

The proof is the test's own pass. `what_the_list_is_told_it_holds` is `#[cfg(test)]`, so
`what_ships` strips it. Were it *not* stripped, `skip_while` would stop on the anchor line and
the next six lines include `.filter(|line| line.contains("enable(false)"))`, which contains
`enable(false)` as a literal, so `disabled` would be non-empty and the test would fail. It
passes, therefore the anchor is absent, therefore the reading is empty.

How it got here is worth recording because nothing did anything wrong. The anchor was real
production code: commit `44ed93f` (2026-07-26) disabled the item because threading was not
built. 01-12 wrote this test red against that code and made it green by deleting the block —
which deleted the anchor. The test was honestly red once and is permanently vacuous now.

It would still catch the exact regression it was written for, somebody re-adding that same
`find_item` block. It cannot catch the item being disabled any other way, and it reports a
clean result whether or not it read anything. The module it sits in already carries
`test_the_reading_can_see_a_call_when_there_is_one`, a companion proving the measurement works
— but only for `every_line_calling`, not for this test's `skip_while` idiom, and not for the
extent. 01-14 established both companions for its own source reads
(`test_the_reading_can_see_a_rebuild_when_there_is_one` and
`test_the_reading_really_covers_the_handler_rather_than_a_prefix_of_it`). This one has neither.

This is phase 2.1's criterion 7 in a second place. That criterion was written about
`test_no_status_page_names_a_version_the_code_does_not_ship`, a document guard disarmed by the
workaround it recommended; this is a source guard disarmed by the fix that made it green. The
general shape — a guard anchored on a line that the work it guards deletes — is worth a look
across the other source-reading guards.

**W2 (warning). Ledger entry 8 is still open although 02.1-06 closed it.** `.planning/WINDOWS.md`
row 8 says `messages.message_id` holds two formats, status `open`. `backfill_message_identifiers`
exists and runs from `MessageCache::open`. `open_count: 40` is therefore one too high, and
`/gsd-ship` blocks on that count.

**W3 (warning). Two entries in the changelog's `[Unreleased]` section contradict each other.**
Line 1146 lists as a known limit: "Two accounts you have given exactly the same name are the one
case the tree cannot tell apart when putting your cursor back. Give them different names and it
is exact." Line 121-124, in the same unreleased section, says the opposite and is the current
truth: "If two or more of your accounts have the same name, those get their addresses back,
because otherwise they would be rows you could not tell apart." 01-14's own note two bullets
above shows the pattern for fixing this: leave the bullet and say it has been overtaken.

**I1 (note, pre-existing). Two menu routes to deleting a folder disagree.** `Message > This
Folder > Delete Folder` (`ID_DELETE_FOLDER`) deletes a server folder. `Message > Sidebar >
Delete` (`ID_CONTEXT_DELETE_CONTAINER`) falls through for mail and answers "Removing a mail
folder is not built yet" (`wx_app.rs:3695`). Both submenus are always present. The Sidebar
submenu's own help text names the four PIM container kinds, so the labelling steers away from
mail, and these commands are the other modules' rather than this phase's. Recorded because a
person who finds the second route first is told a thing the program does is not built.

### Requirements Coverage

| Requirement | REQUIREMENTS.md | Verifier's assessment |
|---|---|---|
| FOLDER-01 | Complete | Matches the tree. Every `[D]` bullet is real and independently checked. The four local-folder gates are honest gates, disclosed in the changelog, and contradict no `[D]` bullet — none promises creating or renaming a folder on this computer. |
| FOLDER-02 | Pending | Correct restraint, and correct after 01-14 too. 01-14 explicitly does not tick it: its `[D]` criteria are about nesting, collapse persistence and unread counts on a parent, none of which is about multiple accounts. The two code-checkable bullets are verified above; the two naming a screen reader are items 1 and 2 under Human Verification. |
| FOLDER-03 | Complete | Matches the tree, with the group's announcement unheard (ledger entry 1, item 3 under Human Verification). |
| THREAD-01 | Complete | Matches the tree. Menu item available, conversation rendering wired to the real paint callback, D-02 to D-12 unit-tested. See W1 on the quality of one of its guards. |
| THREAD-02 | Complete | Matches the tree, with the direction gap routed to Phase 3 and the by-ear half unheard (ledger entry 6, item 4 under Human Verification). |

No orphaned requirements. REQUIREMENTS.md's Phase 1 table lists exactly these five, and every
plan's `requirements:` frontmatter covers all five.

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `the_accounts_in_the_tree` | `folder_tree::rows(accounts: &[AccountInTheTree], ...)` | `folder_tree_updates` hands over the whole slice, `wx_app.rs:10182` | WIRED (was the gap; closed by 01-14) |
| `folder_tree::the_account_a_row_belongs_to` | whose mail the next command acts on | the folder tree selection handler sets the open account from the row | WIRED |
| Six folder command ids | `imap::create_mailbox`/`rename_mailbox`/`delete_mailbox` | `wx_app.rs` handlers, through the application layer, `may_i` before the wire command | WIRED |
| `folders.parent_id` (D-22) | native `TreeCtrl` nesting | `folder_tree::nested` walks it into depth; `fill_the_tree` appends under the tracked ancestor | WIRED |
| `messages.thread_id`/`refs_header` | conversation merge on arrival | `upsert_message` -> `merge_what_this_message_connects`, reached in production by `mail_sync.rs:1039` and `pop_sync.rs:278` | WIRED |
| `cache.pin_row`/`unpin_row` | Favourites group at the top of the tree | `pin_or_unpin_the_chosen_folder` -> `favourite_rows`, pushed first in `rows` | WIRED |
| `state.showing` | the virtual list's text callback | `wx_app.rs:1113-1122` matches on the mode and calls `conversation_cell_text` or `cell_text` | WIRED (source read; the closure needs a window) |
| Five `AppConfig` fields | `wx_settings.rs` controls | field reads at load and writes at save | WIRED, confirmed by a passing guard |

### Behavioral Spot-Checks

Run for this report. `scripts/check.sh all` and `scripts/guards.sh` were not run: Pratik has run
the gate and the sweep, and 6,060 library tests pass.

| Behavior | Command | Result | Status |
|---|---|---|---|
| Criterion 3: every account in one tree; branch order; reorder redraws; row-to-account | `cargo test --lib -- presentation::wx_app::the_tree_holds_every_account:: presentation::wx_app::moving_between_accounts_does_not_rebuild_the_tree:: presentation::folder_tree::tests::` | 92 passed, 0 failed | PASS |
| Criteria 2, 4, 8: collapse round-trip, the shared-folder migration, the settings guard | `cargo test --lib -- data::message_cache::folders::tests::test_the_view_and_the_collapsed_state_restore_together data::message_cache::shared_folders:: application::thread_identity:: data::config::every_setting_is_acted_on:: application::favourites::` | 66 passed, 0 failed | PASS |
| Criterion 6: the view switch, both ways, with selection and sort | `cargo test --lib -- presentation::view_state::` | 55 passed, 0 failed | PASS |
| Criterion 6: the source-reading guards on the list and the menu item | `cargo test --lib -- presentation::wx_app::what_the_list_is_told_it_holds::` | 8 passed, 0 failed | PASS, but see W1 — one of the eight reads nothing |

Probe execution: not applicable. No `scripts/*/tests/probe-*.sh` exists and none is named by any
plan or criterion.

### Anti-Patterns Found

No `TBD`, `FIXME`, `XXX`, `TODO` or `HACK` marker exists anywhere in `src/`. The
"is not built yet" strings in `wx_app.rs` are user-facing gate sentences with reasons, which is
this project's disclosure mechanism rather than silent debt. The only finding of this kind is
W1, and it is a guard that passes rather than a marker.

### Gaps Summary

None. All eight roadmap Success Criteria are met against `377c713`. Criterion 3, the one partial
the first pass found, is closed by 01-14 and made stronger by 02.1-08.

What is left is five things a person has to check, four of them with a screen reader, and they
are not gaps in the work. FOLDER-02 stays Pending because two of its criteria say a screen
reader announces a folder's level from the native control, and no test in this project can
answer that. Structure present is not experience good, which is the second guardrail, and this
phase is honest about where it stops.

Three warnings are recorded above and none blocks the phase. W1 is the one to act on: a guard
this phase's own report cited as proof of a criterion turns out to examine nothing, which is
guardrail 4 exactly, and the general shape it belongs to — a source guard anchored on a line
that the fix deletes — is worth sweeping for.

---

_Verified: 2026-09-03 against 377c713_
_Verifier: Claude (gsd-verifier)_
_Supersedes the 2026-08-31 report, whose criterion 3 finding is preserved above._

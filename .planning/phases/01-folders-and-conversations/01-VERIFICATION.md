---
phase: 01-folders-and-conversations
verified: 2026-08-31T00:00:00Z
status: gaps_found
superseded_by_work: 01-14-PLAN.md
gap_closed: 2026-08-31
needs_reverification: true
score: 7/8 roadmap success criteria verified (1 partial)
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "Each account is its own branch, ordered by the user and moved with the keyboard, so two POP accounts no longer show two folders called Inbox with nothing to tell them apart (roadmap Success Criterion 3)."
    status: partial
    reason: >
      The branch mechanics are real and tested (folder_tree::branch_text names
      an account and its unread/folder counts; account_order/D-14 stores an
      ordinal and Alt+Shift+Up/Down moves it). But the folder tree as actually
      wired never shows more than one account's branch at a time. The single
      call site that builds AccountInTheTree rows
      (src/presentation/wx_app.rs:9467-9471, inside folder_tree_updates) is
      passed exactly one account, every one of its eleven call sites (lines
      6380, 7062, 7608, 7906, 8140, 8289, 10013, 10697, 14904, 16150, 17863)
      hands it a single account.id, and UIUpdate::FoldersLoaded's handler
      calls folder_tree.delete_all_items() before rebuilding, so the tree is
      wholly replaced by one account's rows on every refresh. A user with two
      POP accounts, each with an Inbox, can never see both accounts' branches
      side by side to tell them apart; they can only infer which account they
      are looking at from whichever single branch is currently drawn.
      Reordering accounts is real but is not observable in the folder tree the
      criterion names: docs/changelog.md itself says "the folder tree draws
      one account's branch at a time, so today the new order shows in the
      account list rather than in the sidebar."
    artifacts:
      - path: src/presentation/wx_app.rs
        issue: "folder_tree_updates and its callers never aggregate more than one account's AccountInTheTree/FolderInTheTree into folder_tree::rows"
    missing:
      - "A code path that loads every account's folders into one tree rebuild, so folder_tree::rows (which already accepts a slice of accounts) is called with all of them at once."
      - "The changelog's known-limit note (docs/changelog.md line ~282-286) updated: it still says multi-account drawing 'is waiting on the shared local folders work', but that work (D-18/D-19) landed in this same phase (01-07) and the limitation was left in place anyway, 'at the coordinator's instruction' per 01-07-SUMMARY.md. The note as written misleads a reader into expecting it imminently rather than recording it as a deliberate deferral."
deferred: []
human_verification:
  - test: "Arrow onto a nested folder (e.g. Archive/2026, read as 2026 under Archive) with NVDA running and confirm the level is announced from the native TreeCtrl hierarchy."
    expected: "NVDA announces \"level 2\" (or equivalent) when the cursor lands on the nested folder, without the level being spelled into the label text."
    why_human: "Structure is verified (native TreeCtrl append_item under a tracked parent item, not label indentation), but only a live screen reader run proves what is actually announced. This is why FOLDER-02 is correctly left Pending in REQUIREMENTS.md rather than marked Complete."
  - test: "Have a folder arrive that the server has stopped listing, and separately have a reminder come due, while NVDA is running, and confirm the one-question-at-a-time gate and the typing gate behave as the tests describe."
    expected: "Only one modal question is ever on screen; a reminder does not open over someone typing (a disclosed, deliberate gap: it currently does, tracked in deferred-items.md)."
    why_human: "src/presentation/wx_app.rs::ask_about_the_folders_that_have_gone is, by the phase's own account, the one function in 01-10 no test reaches; wxWidgets' one-application-per-process limitation makes it untestable without a running window."
---


> **Read this before acting on the gap below.** This report was written before
> plan 01-14 existed, and 01-14 was written to close the one gap it found. That
> gap, roadmap success criterion 3, is closed: `folder_tree::rows` is now called
> with every account rather than a one-element slice, two accounts each holding
> an `Inbox` produce two rows a caller can tell apart, and switching between
> accounts no longer rebuilds the tree.
>
> The findings below are a true record of the tree on 2026-08-31 before that
> plan landed. They are not a description of it now. A tool reading only this
> file's `status` will recommend planning gap fixes for work that is already
> done, which is what happened on the next manager run.
>
> What remains genuinely open on this phase is not in this report: FOLDER-02
> stays Pending because two of its criteria say a screen reader announces the
> folder's level from the native control, and no test in this project can answer
> that. Re-verification against the current tree has not been run.

# Phase 1: Folders and conversations Verification Report

**Phase Goal:** A user can shape and work through their mail by its own structure: an account
they can tell from the next one, folders they can make and manage, nested the way the server
nests them, favourites at the top, and conversations collapsed to one row.

**Verified:** 2026-08-31
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria, checked against the tree)

| # | Truth (roadmap Success Criterion) | Status | Evidence |
|---|---|---|---|
| 1 | A user creates, renames, moves, marks read, empties and deletes a server folder from the tree by keyboard; refused with a reason when mail writes are off; local operations are not gated. | ✓ VERIFIED (with two disclosed, non-blocking limits) | `create_mailbox`/`rename_mailbox`/`delete_mailbox` exist in `src/service/protocols/imap.rs` (lines 893, 930, 962), each calling `self.may_i(...)` *before* building the wire command, so a refusal never reaches the server. All six commands (`ID_NEW_FOLDER`, `ID_RENAME_FOLDER`, `ID_MOVE_FOLDER`, `ID_DELETE_FOLDER`, `ID_EMPTY_FOLDER`, `ID_MARK_FOLDER_READ`) are declared, menu-wired, and handled in `src/presentation/wx_app.rs` (3419-3512), each calling into the corresponding application function. Emptying and mark-read run through `application::local_folders::deleting`, which is the per-message decision that is not gated by `Allowed` for a purely local folder — matches the FOLDER-01 `[D]` bullet in REQUIREMENTS.md verbatim. Two things are honestly disclosed and *not* built: creating a **nested** folder under an existing server folder always lands it flat (`the_path_of_a_new_folder`, tested at `wx_app.rs:23593`, with a matching `docs/changelog.md:409` note and a workaround: create flat, then Move); creating a **new local** folder (whether under "On this computer" or freshly under a POP account) is refused with `FOLDERS_ON_THIS_COMPUTER_ARE_NOT_MADE_YET`, also disclosed in `docs/changelog.md:415-416`. Neither is a hidden stub. |
| 2 | `Archive/2026` reads as `2026` nested under `Archive`, level from the native tree control; collapse state survives a restart, keyed by identity. | ✓ VERIFIED (mechanism); screen reader announcement itself is Pratik's to confirm, correctly Pending in REQUIREMENTS.md | `folder_tree::nested` (folders.rs:809) builds real parent/child depth from `folders.parent_id` (D-22), and `fill_the_tree` (wx_app.rs:13073) calls `tree.append_item(&parent, ...)` under a stack-tracked ancestor `TreeItemId`, not indentation in the label — genuine native nesting. `tree_state` persists collapse by `WhichRow::stored()` identity (`folders.rs:275-396`), with a round-trip test (`folders.rs:1168-1235`, `test_the_view_and_the_collapsed_state_restore_together`) and a "survives dropping the table" test at line 1671. `on_item_expanded`/`on_item_collapsed` handlers (wx_app.rs:2412-2425) write it as it happens. |
| 3 | Each account is its own branch, ordered by the user and moved with the keyboard, so two POP accounts no longer show two folders called Inbox with nothing to tell them apart. | ⚠️ PARTIAL — see gap in frontmatter | `folder_tree::branch_text` and D-14's `account_order`/Alt+Shift+Up/Down (`move_the_chosen_account`, wx_app.rs:8471) are real and tested. But `folder_tree::rows` is only ever called with **one** account (`folder_tree_updates`, wx_app.rs:9388-9534, every one of its 11 call sites passes a single `account.id`), and `UIUpdate::FoldersLoaded`'s handler wipes the whole tree (`delete_all_items()`) before redrawing it. The tree therefore never shows two account branches at once. This is not a discovery: `01-05-SUMMARY.md:99`, `01-07-SUMMARY.md:240`, and `docs/changelog.md:282-286,664-669` all say so, and the 01-07 summary records it as a deliberate coordinator decision to leave for later, not an oversight. The changelog's own wording is now stale (still blames "the shared local folders work" as the blocker, which landed in this same phase). |
| 4 | Sent/Outbox/Drafts/Junk/Trash are one each under "On this computer"; an existing database is migrated message by message with nothing removed until landed, and a count reported. | ✓ VERIFIED | `local_folders::SHARED_BY_EVERY_ACCOUNT` (5 entries) and `FOR_IMAP` empty per D-18 (local_folders.rs:124-140). `MessageCache::merge_local_folders` (shared_folders.rs:115) runs once at `MessageCache::new` (mod.rs:1239), non-fatal, and its own module doc states the "nothing removed until landed" property. The report is surfaced via `UIUpdate::CommandAnswered` at `Priority::High` on its own topic (wx_app.rs:546-550), so it is spoken and logged, not silent. |
| 5 | A user pins a folder; it stays in a group at the top of the tree across a restart; never touches a server; appears in both places. | ✓ VERIFIED | `pin_or_unpin_the_chosen_folder` (wx_app.rs:8406) calls `cache.pin_row`/`unpin_row` only — a local SQLite write, no `Allowed`/server call anywhere on the path. `favourite_rows` places the group above `ALL_INBOXES` (folder_tree.rs:516), omitted when empty. The pin-vs-subscription precedence decision is recorded both in `favourites.rs`'s module doc and in `PROJECT.md`'s Key Decisions table (line 210), satisfying FOLDER-03's "recorded... before the second half is built" bullet. |
| 6 | View menu's Thread View is enabled; switching it shows one row per conversation with columns describing the conversation, not the newest message. | ✓ VERIFIED | `ID_THREAD_VIEW` is an `append_check_item` with no `.enable(false)` anywhere nearby, and a source-scanning test (`test_the_thread_view_item_is_not_disabled_on_the_way_up`, wx_app.rs:24011) asserts exactly that and passes. The virtual list's paint callback calls `message_rows::conversation_cell_text` in conversation mode, confirmed by a passing test (`test_the_paint_callback_asks_for_a_conversation_cell_when_rows_are_conversations`). `application::conversations`, `MessageColumn`'s aggregate rules and `Sort::conversation_order_by_clause` (D-02/D-03/D-04) are unit-tested per column. |
| 7 | A message arriving into an open folder joins its thread without reopening it, including a late message merging two existing trees; a test fails if they are left separate. | ✓ VERIFIED, with one disclosed, one-directional limitation | `MessageCache::upsert_message` calls `merge_what_this_message_connects` (messages.rs:944) on every insert path (sync, POP download, sent copy, import) — this is the real production path, not a test-only simulation. The merge case is tested and passing (`application::thread_identity::tests`, 30/30 pass). The disclosed gap — a root arriving *after* the message that names it is not merged, 3 of 6 arrival orders succeed — is pinned by a passing test named for the gap (`a_root_arriving_after_the_message_that_names_it_is_left_out_of_the_merge`) and recorded in `deferred-items.md`, `WINDOWS.md`, and the changelog, matching the task brief exactly. |
| 8 | The five settings this phase adds are each reachable and operable from a real settings screen; no sixth ungoverned setting is added. | ✓ VERIFIED | Five fields confirmed: `unread_on_a_parent` (D-24), `empty_reaches_subfolders` (D-34), `mark_read_reaches_subfolders` (D-35), `a_conversation_reaches`, `deleting_a_conversation_row` — each has a matching control in `wx_settings.rs` (Choice/CheckBox, read on load, written on save). `data::config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen` passes (run directly, not inferred from a report); its only exception, `allowed_per_account`, predates this phase and is itself named as a separate, already-disclosed gap in `deferred-items.md`. |

**Score:** 7/8 roadmap success criteria fully verified, 1 partial (criterion 3).

### Requirements Coverage

| Requirement | Source Plans | REQUIREMENTS.md Status | Verifier's Assessment |
|---|---|---|---|
| FOLDER-01 | 01-01, 01-04, 01-09 | Complete | Matches the tree. Every `[D]` bullet (create/rename/delete by keyboard, mark-read zeroes the count, empty confirms with a name and count, server ops pass `may_i`, local ops don't, `is_local` is the single source of truth) is real and independently checked above. The undisclosed-sounding limits found while checking (no nested creation, no new local-folder creation) are both honestly recorded in `docs/changelog.md` and don't contradict any `[D]` bullet, which never promises nested or new-local creation specifically. |
| FOLDER-02 | 01-03, 01-05, 01-06, 01-07, 01-10 | Pending | Correct restraint. The two `[D]` bullets that are code-checkable (collapse/expand by keyboard persisting across a restart; unread counts on a collapsed parent, worded per D-24) are verified above. The two that name a screen reader (level announced from the native control) cannot be settled by static analysis — the structure is right (real `TreeCtrl` parent/child nesting), but only a live NVDA/Narrator run proves what is spoken, and that is Pratik's to run, not a gap to invent. |
| FOLDER-03 | 01-08 | Complete | Matches the tree. All five `[D]` bullets verified above (pin/unpin by keyboard, local-only write, subscription-back compatible shape via the shared `(account_id, path)` key, the win-decision recorded before the second half was built, the group announcing itself as a group via `group_text`/`favourite_rows`). |
| THREAD-01 | 01-02, 01-11, 01-12 | Complete | Matches the tree. Menu item enabled, conversation-mode rendering wired to the real paint callback, D-02 through D-12's mechanics (selection round-trip, sort survives the switch, whole-conversation actions) each have passing unit tests cited in 01-12-SUMMARY.md and spot-checked above. |
| THREAD-02 | 01-02, 01-13 | Complete | Matches the tree, with the caveat that "does not re-announce rows the user is not on" is proven as a decision (which rows changed, `refresh_item` used rather than `set_item_count`) rather than as behaviour of a running window, which the phase's own `deferred-items.md` entry says plainly. That is an honest limit of what a test harness can reach here (wxWidgets: one application per process), not a stub. |

No orphaned requirements: `.planning/REQUIREMENTS.md`'s Phase 1 table lists exactly FOLDER-01, FOLDER-02, FOLDER-03, THREAD-01, THREAD-02, and every plan's `requirements:` frontmatter (01-01 through 01-13) covers all five with no ID left unclaimed.

### Key Link Verification (spot-checked)

| From | To | Via | Status |
|---|---|---|---|
| `ID_NEW_FOLDER`/`ID_RENAME_FOLDER`/`ID_MOVE_FOLDER`/`ID_DELETE_FOLDER` handlers | `MailController::create_mailbox`/`rename_mailbox`/`delete_mailbox` | `wx_app.rs` calls into `application::mail_controller`, which calls `may_i` before `service::protocols::imap` | ✓ WIRED |
| `folders.parent_id` (D-22, plan 01-03) | native `TreeCtrl` nesting | `folder_tree::nested` walks `parent_id` into depth; `fill_the_tree` calls `append_item` under the tracked ancestor | ✓ WIRED |
| `messages.thread_id`/`refs_header` (plan 01-02) | conversation merge on arrival | `MessageCache::upsert_message` → `merge_what_this_message_connects` → `thread_identity::rejoin` → `reroot_threads`, on every insert path | ✓ WIRED |
| `cache.pin_row`/`unpin_row` | folder tree Favourites group | `pin_or_unpin_the_chosen_folder` → `favourite_rows` in `folder_tree::rows` | ✓ WIRED |
| `folder_tree::rows(accounts: &[AccountInTheTree], ...)` | the rendered sidebar | `folder_tree_updates` (single caller of the row builder) | ⚠️ WIRED BUT SCOPED TO ONE ACCOUNT — see gap above |
| Five phase-added `AppConfig` fields | `wx_settings.rs` controls | Direct field reads/writes at load (`~line 1013-1105`) and save (`~line 2135-2160`) | ✓ WIRED, confirmed by a passing guard test |

### Anti-Patterns Found

None found that rise to blocker or warning level in the files checked. No unreferenced `TBD`/`FIXME`/`XXX` markers were found in the artifacts inspected; the phase's own `deferred-items.md` and inline comments are the disclosure mechanism this project uses instead, and they name themselves as such rather than being silent debt markers.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Full library suite (baseline, run once) | `cargo test --lib` | 5803 passed, 0 failed, 1 ignored | ✓ PASS |
| `data::config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen` | `cargo test --lib data::config::every_setting_is_acted_on::test_every_setting_somebody_can_change_is_offered_by_a_screen` | ok | ✓ PASS |
| `presentation::wx_app::what_the_list_is_told_it_holds::*` (thread-view/paint-callback structural tests) | `cargo test --lib presentation::wx_app::what_the_list_is_told_it_holds::` | 8 passed | ✓ PASS |
| `application::thread_identity::*` (merge, order-independence, disclosed gap) | `cargo test --lib application::thread_identity::` | 30 passed | ✓ PASS |
| `data::message_cache::messages::*` (conversation counting, uid collision on merge, All Mail exclusion) | `cargo test --lib data::message_cache::messages::` | 126 passed | ✓ PASS |

Probe execution (Step 7c): not applicable. No `scripts/*/tests/probe-*.sh` files exist in this project and none are named by the phase's plans or success criteria.

### Human Verification Required

See `human_verification` in the frontmatter. Both items are screen-reader-dependent and are Pratik's to run at his own pace, per project convention; neither blocks this report from being usable, and neither is new — both are already named in `deferred-items.md` and `WINDOWS.md`.

### Gaps Summary

One roadmap Success Criterion, number 3, is not fully delivered: the folder tree never shows more than one account's branch at a time, so the criterion's own test — "two POP accounts no longer show two folders called Inbox with nothing to tell them apart" — cannot be demonstrated by looking at the tree, because the tree never puts two accounts' folders in front of a user at once. Account branch labeling and keyboard reordering are real, tested, and correct as far as they go; what is missing is the aggregation step that would call `folder_tree::rows` (which already accepts a slice of accounts) with every account instead of one.

This is not a hidden defect. It is recorded, by name, in three places written during the phase itself (`01-05-SUMMARY.md`, `01-07-SUMMARY.md`, `docs/changelog.md`) as a deliberate scope decision made once its original blocker (D-18) was resolved. It is reported here as a gap rather than folded into "expected, pre-disclosed limitations" because it bears directly on one of the roadmap's eight numbered Success Criteria for this phase, and because the changelog's own explanation for it is now inaccurate (it blames a blocker that has since shipped, without saying the deferral is now deliberate rather than blocked). A closure plan should either build the multi-account aggregation, or the changelog note should be corrected to say plainly that this is deferred by choice, with the deferral point named — and if the decision is that this phase should still count as done without it, that acceptance belongs in an explicit override here, not in an unreviewed gap.

Two other, narrower items are disclosed, deliberately narrow in scope, and not treated as gaps because they don't bear on any of the eight numbered criteria as literally stated: the conversation-merge direction gap (THREAD-02, three of six arrival orders) and Gmail mail archived with no label vanishing from a conversation count (an edge case of D-08's All Mail exclusion). Both are pinned by passing tests named for the gap and both are recorded in `deferred-items.md`, `WINDOWS.md`, and the changelog's Known Limitations.

---

_Verified: 2026-08-31_
_Verifier: Claude (gsd-verifier)_

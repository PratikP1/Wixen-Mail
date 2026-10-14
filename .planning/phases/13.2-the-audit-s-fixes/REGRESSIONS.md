# Phase 13.2: four regressions Pratik reported on 2026-10-14

Written 2026-10-14 against `main` at `dc592d07` (`git rev-parse HEAD` printed
`dc592d07e3fc8e9aa6352446a70aecef756066d2`, working tree clean). Pratik is running build
`1.0.0-alpha.1+1118.gd3455792`, and `git diff --stat d3455792 dc592d07 -- src tests` printed
nothing, so the code read here is the code he runs.

The sources are four read-only investigations made the same day, one per issue. Nothing was
built or run. Every red test below is a proposal that has not been run yet.

This writer re-read these deciding points at `dc592d07` before copying them: the Enter handler's
conversation arm, `conversation_nodes`, `apply_threading`, `the_loaded_message_the_row_stands_for`,
`mark_these_read`, `take_row_out_of_the_list`, the `ConversationsLoaded` arm, `mark_what_was_read`,
the three writers of `s.conversations`, and the handlers `build_thread_dialog` binds. Everything
else is carried from the investigation named beside it. Inferred figures are marked "derived".

## In short

| # | What Pratik meets | Cause | Introduced | Plan |
|---|-------------------|-------|------------|------|
| 1 | Enter on a message does nothing | In conversation view, Enter looks a lone message up by a conversation id that the loaded rows have emptied, finds nothing, and returns silently | Latent since `ffbc95a0` (2026-08-30); reachable since `478f5885` (2026-09-20) | A |
| 2 | Mark as Read is announced, but the row changes only at the next check | The list paints conversation rows from a snapshot that nothing but a full re-read replaces; the mark writes everything except that snapshot | `ffbc95a0` made the second copy; `7d2c64f4` (2026-09-18) pointed marks at conversation rows; exposed by `478f5885` | B |
| 3 | Delete is announced, but the row stays until the next check | The same snapshot: the delete takes the message out of the flat rows only, and the count and cells under conversation view read the snapshot | `ffbc95a0`; exposed by `478f5885` | B |
| 4 | Inside an opened conversation, no command acts on one message | The conversation window is a modal picker with no key, menu-key or menu handler, and every command reads its target from the main list | Never built; became the everyday path at `478f5885` | C |

Issues 2 and 3 share one cause, so they are one plan. All four change what is spoken or shown.
The common thread is `478f5885` (feat(11-11.1.2), #92), which made a folder never set show
conversations. Every path below worked, or was hidden, in the flat view he used before it.

## The plans

Three plans, in this order. Working names A, B and C; the numbering is the orchestrator's call
(see "Where the plans sit in phase 13.2"). Numbered on 2026-10-14: A is 13.2-01, B is 13.2-02,
and C is 13.2-03 with 13.2-03.1, split from it because one executor could not carry both. They
run first in the phase, and every audit plan moved up by three.

| Plan | Title | Issues | Main files | Spoken or shown | Size |
|------|-------|--------|------------|-----------------|------|
| A | Enter opens a message that is alone in its conversation, and says so when it cannot open anything | 1 | src/presentation/wx_app.rs (the conversation arm of `msg_list.on_item_activated`); src/presentation/ui_types.rs (a stale comment); docs/changelog.md | Yes: a reader window opens where nothing did; a new sentence when nothing can open | M |
| B | A mark, a star, a label, an undo or a delete made here shows on a conversation row at once, and the cursor lands after a removal | 2, 3 | src/data/message_cache/messages.rs (a narrowed conversation listing); src/presentation/wx_app.rs (one catch-up function and its call sites); src/presentation/ui_types.rs; src/presentation/view_state.rs; docs/changelog.md | Yes: the Unread and Thread cells, the Mark as Read wording, where the cursor lands | L |
| C | The conversation window acts on the message under its cursor: the menu key, Delete, M and the other message keys | 4 | src/presentation/wx_thread_view.rs; src/presentation/wx_app.rs (`chosen_messages`, `open_conversation_again`, the single-message commands, `mark_what_was_read`); src/application/choosing_messages.rs; src/presentation/wx_context_menu.rs; docs/KEYBOARD_SHORTCUTS.md; docs/changelog.md | Yes: a context menu and keys in the conversation, and what it lists after a command | L |

Why this order:

- **A first.** It is the smallest. It restores the most common case: a derived lower bound of
  47% of the conversation rows in the folder his 2026-10-06 log recorded hold one loaded
  message (see issue 1's evidence).
- **B before C.** A command chosen inside the conversation window goes through the same do-halves
  as one chosen in the list. Without B, the list row underneath is stale when the window closes,
  and C would ship a feature whose result cannot be heard until the next check.
- **C last.** It is the largest, and it needs B's catch-up function at its call sites. If its
  fourth task, the remaining command keys, pushes it past four tasks, split that task into a plan
  of its own straight after C.

Each plan is red then green on a branch. The red commit names its failing tests in
`Fails-until-green:` lines. Each plan pushes its branch and opens a pull request so the NVDA job
runs before the merge, under the standing OK of 2026-09-23. Each records a listening checkpoint
for Pratik.

## 1. Enter does not open a message

### Pratik's words

"Pressing enter on a mail doesn't open a message any more. It appears that the fix to
space/shift+space broke the message opening completely."

### What happens

In conversation view, Enter does nothing on a row whose conversation has fewer than two messages
among the open folder's loaded messages. Nothing opens and nothing is said. That is every message
that is alone in its conversation, and a conversation whose other messages are filed in another
folder, such as an Inbox message whose reply sits in Sent.

Enter still opens the conversation window on a row with two or more loaded messages, which is
what Pratik's issue 4 describes. Enter still opens the message in the flat view.

The Space and Shift+Space fix did not cause this. Space finds the row's message by its id, and
Enter finds it by conversation membership, so Space works on exactly the rows where Enter fails.
That is why the two looked connected.

### Cause

1. In conversation view, `msg_list.on_item_activated` calls
   `conversation_nodes(&state, &read_in.account_id, &thread_id)`. When it gets fewer than two
   nodes, it opens `nodes.first()`, looked up in `s.messages` by id. When that finds nothing, it
   returns without a word. (Read at `dc592d07`: the first of the two arms in that handler that
   begin `if nodes.len() < 2 {`.)
2. `conversation_nodes` keeps the loaded messages whose `MessageItem.thread_id` equals the row's
   conversation id: `m.thread_id.as_deref() == Some(thread_id)`. (Read.)
3. `apply_threading` sets `item.thread_id = None` for any conversation with fewer than two loaded
   messages, on purpose. Its doc comment says "A conversation of one is not a conversation", and
   the reason it gives is to keep a thread indicator and an earcon off every ordinary message.
   (Read.)
4. The store gives every message a conversation id, and the conversation listing lists a
   conversation of one as a row. The investigation cites `backfill_thread_ids` and the store test
   `test_a_conversation_of_one_message_counts_one` in src/data/message_cache/messages.rs.
   (Investigation 1.)
5. So the key the row carries is the store's conversation id, and the key the loaded rows carry
   is a display field emptied for lone messages. For a lone message the join is empty and the
   handler returns. The arm written for exactly one node never sees one node in the running
   program.

The unit tests of `conversation_nodes` build MessageItems by hand, including a lone message with
`thread_id` set, which is a shape `apply_threading` never produces. That is why the dead arm
passed. (Investigation 1, wx_app.rs tests near
`test_a_conversation_ignores_messages_from_other_threads`.)

### Evidence

- The handler and its helpers are identical in `5f363255`, the last build where Enter worked for
  him, and in `d3455792`. Investigation 1 saved both copies of wx_app.rs with `git show` and
  compared them with python difflib: the block from `msg_list.on_item_activated` to the Space
  comment is 161 lines in both and identical, and so are `conversation_nodes`, `apply_threading`
  and `the_row_stands_for`.
- `dfb2b9f6` (feat(11-08), #31, 2026-09-19) added only the `open_on` lines. The
  `nodes.len() < 2` arm is blamed wholly to `ffbc95a0` (`git blame -L 3598,3620`).
  (Investigation 1.)
- Pratik's log shows his list was showing conversations. Each "Message list now holds N rows"
  line is paired with the next spoken list count. On 2026-10-06 and 2026-10-07 the pairs read
  34,407 loaded and 22,503 spoken, then 34,408 and 22,504, and so on. On 2026-09-18 and
  2026-09-19 the two counts are equal, which is the flat view. (Investigation 1, numbers only,
  read with python from `wixen-mail.2026-10-06.log` and its neighbours.)
- Reader windows stopped opening when the view changed. Counting the reader window's own opening
  sentence per day, by the view on screen: 2026-09-18 flat 12; 2026-09-19 flat 8; 2026-09-20 flat
  9 and conversations 2; 2026-10-06 conversations 1; 2026-10-07 none. 29 of 32 opens were in the
  flat view. The log records no key presses, so this dates the change; it does not show each
  failed press. (Investigation 1.)
- Rows affected (derived): if every conversation row holds at least one loaded message and the
  rest hold two or more, then rows with exactly one are at least 2 x 22,503 - 34,407 = 10,599 of
  22,503, about 47%. Inputs: the 2026-10-06 log figures above. This is a lower bound.
- Nothing upstream eats Enter. wxWidgets 3.3.2 lets a plain Enter on a list skip pre-processing
  and raises ITEM_ACTIVATED for the selected row (target/release/wxWidgets/src/msw/listctrl.cpp,
  near lines 2260 to 2273 and 2651 to 2669). None of the KEY_DOWN closures on `msg_list` consumes
  Enter, and no menu accelerator names Enter or Return. (Investigation 1.)

### Introduced by

- Latent since `ffbc95a0` (2026-08-30, feat(01-12): the View menu's Thread View item works). That
  commit wrote the conversation arm on top of `apply_threading`'s reset of lone messages, which
  dates from `cab6da7e` (2026-07-27, Group messages into conversations).
- Reachable for Pratik since `478f5885` (2026-09-20, feat(11-11.1.2): a folder never set shows
  what Show conversations by default says). That commit is in build `+351.g5f363255`, which he
  ran from 2026-09-20 13:32, and in `+1118.gd3455792`. Before it his folders were flat, and the
  flat arm opens a lone message directly.
- Not `dfb2b9f6`, the #31 Space fix.

### Red test (proposed, not run)

In `mod tests` of src/presentation/wx_app.rs, beside
`test_a_conversation_lists_oldest_first_with_parents_before_children`. The test builds the row
and the loaded rows through the real producers, because the bug lives in the join between them.

Order of commits:

1. A green extraction with no change in behaviour. Move the conversation arm's choice out of the
   closure into a function over the state and a row number that answers one of: the message to
   open, the tree to open (its nodes and `open_on`), or nothing. Keep the text
   `the_row_stands_for(` inside the handler so the reading in
   tests/a_conversation_row_stands_for_one_message.rs stays green.
2. A red commit naming the first two cases below.

Cases:

- `test_enter_on_a_conversation_of_one_opens_that_message`. A `MessageCache` in a temporary home,
  INBOX for account `acc`, one message with no References. Then
  `cache.get_message_list_sorted(inbox, "acc", None, None)`, `MessageItem::from_row` over the
  rows, and `apply_threading(&rows, &mut items)`, which is what `load_folder_messages` does. Then
  `cache.conversations_in(inbox, "acc", AConversationReaches::TheWholeAccount, None)`. With the
  state showing conversations, row 0 must answer the message whose id is
  `conversations[0].stands_for`. Today it answers nothing.
- `test_enter_on_a_conversation_split_across_folders_opens_the_message_here`. The INBOX message
  plus a reply filed in Sent with References to it. The row counts 2 under the whole-account
  reach, and 1 is loaded. Row 0 must answer the INBOX message. Today it answers nothing.
- `test_enter_on_a_conversation_of_two_here_opens_the_tree_on_the_row_message`. Two messages in
  INBOX. It must answer the tree with both ids and `open_on` equal to `stands_for`. Green before
  and after; it guards the path that works now.

A built-list companion in tests/, after the extraction, on the pattern of
tests/a_move_completes_here_first.rs and on a desktop of its own: a Report and Virtual ListCtrl
with one row selected, `on_item_activated` bound to the extracted chooser over the state built
above, and a recorder. Send WM_KEYDOWN and WM_KEYUP of VK_RETURN with no modifier held. The
recorder must hold the lone message's id. One press shows both that Enter reaches activation and
that a row of one opens.

### Fix

In the `nodes.len() < 2` arm of the conversation branch:

- Open the row's own message by id, the way Space and Print already find it:
  `lock_state(&state).the_loaded_message_the_row_stands_for(index as usize).cloned()`, then
  `open_single_message`.
- When the loaded rows do not hold it (the row's message is filed in another folder), read that
  one row from the store with `cache.message_rows_for(&[stands_for.id], None)` and
  `MessageItem::from_row`, then open it.
- When neither finds it, say a sentence on the status channel instead of returning silently. A
  key that does nothing sounds the same as a broken key.

Also in this plan:

- Correct the conversation_nodes unit test that builds a lone message with `thread_id` set, so it
  builds through `apply_threading` or says why it cannot.
- Correct the stale comment on `MessageItem::from_row` in src/presentation/ui_types.rs, which says
  `MessageListRow` carries no conversation id. `apply_threading` reads `row.thread_id`.
  (Investigation 1.)
- A changelog entry under `[Unreleased]`.

Not in this plan, and worth a ledger row: the root fix. Carry the store's conversation id on
`MessageItem` in a field `apply_threading` never empties, and make `conversation_nodes` join on
it. That would leave the flat view's thread indicator alone, make the one-node arm reachable as
written, and also fix the second item in the next list.

### Files

- src/presentation/wx_app.rs
- src/presentation/ui_types.rs (comment only)
- docs/changelog.md
- read, not changed: src/application/threading.rs, src/data/message_cache/messages.rs,
  tests/a_conversation_row_stands_for_one_message.rs, tests/a_move_completes_here_first.rs,
  tests/all_inboxes_keeps_a_view_of_its_own.rs

### What else the same cause breaks

- A conversation whose messages are in more than one folder, such as an Inbox message and your
  reply in Sent. The row says "2 messages" under the default whole-account count, one message is
  loaded, and Enter does nothing. Plan A fixes this.
- A conversation with two or more messages here and more elsewhere. The tree holds only this
  folder's members while the row counts the whole account, so `open_on` can name a message that
  is not in the tree, for example a starter in Sent, and the tree opens on its root instead of
  the row's message. Plan A does not fix this; the root fix above does.
- All Inboxes in conversation view makes the same join and fails the same way. Plan A's fix
  reaches it, because the arm is shared.
- Not broken by this cause: Print, which falls back to the row's message by id; Space and
  Shift+Space; the preview; and Enter in the flat view.

### Spoken or shown

Yes. A reader window opens where nothing opened, and a new sentence is spoken when nothing can be
opened.

## 2. Mark as Read and Mark as Unread show only after the next check

### Pratik's words

"Read/unread status change doesn't apply immediately. While the announcement says it does, the
change only takes place when message sync occurs. This is confusing."

### What happens

In conversation view, M (or the menu, the toolbar or the context menu) marks the conversation's
messages read in memory, in the store and on the server, and announces it at once. The row on
screen keeps saying "Unread", and its Thread cell keeps saying "2 messages, 2 unread", until a
check reads the folder again. In the message view (Ctrl+T) the same command changes the row at
once.

### Cause

1. Under conversation view the list paints each row through `virtual_rows::text_for`, whose
   conversation arm reads `ConversationItem.unread` through
   `message_rows::conversation_cell_text`: the Unread cell is "Unread" when `unread > 0`, and the
   Thread cell is `counts_read_as(conversation.messages, conversation.unread)`.
   (Investigation 2.)
2. `mark_these_read`, the do-half behind every way of marking, sets `row.read` on matching rows
   of `s.messages`, writes the store through `write_flags_or_put_the_row_back`, hands the change
   to `spawn_server_change`, and calls `list.refresh(true, None)`. It never touches
   `s.conversations`. (Read at `dc592d07`.)
3. `s.conversations` has three writers in wx_app.rs: `s.conversations.clear()` on a folder change,
   the same on a view switch, and `s.conversations = conversations.clone()` in the
   `ConversationsLoaded` arm. (Read with the pattern
   `s\.conversations = |s\.conversations\.clear\(\)`, three hits; investigation 2's wider pattern,
   which also looked for `iter_mut`, `retain`, `remove` and `get_mut`, found the same three.)
4. `ConversationsLoaded` is sent only by `load_folder_conversations` and
   `load_every_inbox_conversations`. Those run on opening a folder, on switching the view, on an
   undo, when a refused change is put back, and from `reread_folder_if_open` after a check.
   (Investigations 2 and 3.) That last one is the "only when message sync occurs" Pratik heard.
5. The root: a conversation row is a summary the listing query works out once, and every path
   that changes a message here treats the screen as if each row were a message.

The announcement comes first and is not wrong: the mark did land. The Mark as Read wording on the
Action menu, the toolbar and the context menu then reads the same stale count through
`the_chosen_rows_have_unread`, so it keeps offering Mark as Read. The next M reads the store and
marks the conversation unread. (Investigation 2.)

### Evidence

- `mark_these_read` read at `dc592d07`: the loop, the store write, the server change, and
  `list.refresh(true, None)`. No reference to `conversations`.
- The three writers of `s.conversations`, above.
- Nothing was refused. In `wixen-mail.2026-10-06.log` and `wixen-mail.2026-10-07.log` there are 0
  lines of "Failed to update flags" and 0 of "did not reach the server, so it has been undone".
  As a control the same reader counts 1 ERROR line in the 2026-09-18 log. (Investigation 2, counts
  only.) His profile's `moves_waiting` and `waiting_flag_changes` tables each held 0 rows.
  (Investigation 3.) So the put-back path is ruled out.
- The store is not the stale part. `ConversationItem.unread` comes from the listing SQL, and a
  re-read from the store, which already holds the new flag, gives the right row. (Investigation
  2's fourth proposed test asserts this.)
- 13-24 did not cause it. The toggle before `d1afe472` (`git show d1afe472^:src/presentation/wx_app.rs`)
  has the same loop, the same refresh, and no write to the conversation rows. (Investigation 2.)
- Retest that would confirm it on his machine, from investigation 2: in the Inbox, Ctrl+T until
  NVDA says "Showing messages", M on an unread message, arrow away and back: no longer Unread.
  Ctrl+T to "Showing conversations", M on an unread conversation, arrow away and back: still
  Unread until the next check.

### Introduced by

- `ffbc95a0` (2026-08-30, feat(01-12)) added `s.conversations` as the copy the list counts and
  paints under conversation view, and left every writer writing only `s.messages`.
- `7d2c64f4` (2026-09-18, feat(11-07)) made Mark as Read, Star, labels and Delete act on a
  conversation row's messages, and made the Mark as Read wording read the row's own unread
  count. It added nothing to work the row out again after the write. Its ear check, "M on a
  conversation row" (ledger 544), is still open.
- Exposed for Pratik by `478f5885` (2026-09-20).
- 13-24 (`d1afe472`) and 13-24.1 only moved the same loop into `mark_these_read`.

### What else the same cause breaks

- Star on a conversation row: `star_these` sets `starred` on `s.messages` only, while the Flagged
  cell reads `conversation.any_flagged`.
- Labels on a conversation row: the Labels cell reads `conversation.labels`, and no label path
  writes `s.conversations`. This follows from the writer count; the label do-half was not read
  line by line.
- Edit, Undo of a mark (`write_the_flags`).
- The automatic mark-as-read after reading: `mark_what_was_read` sends `MessageReadToggled`, which
  writes `s.messages` only.
- A refused server change being put back, which also arrives as `MessageReadToggled`.
- The unread count in the folder tree, in either view. It shows the stored
  `folders.unread_count`, written only by the mail check's `set_folder_counts` and by mark-all-read.
  So "Inbox, N unread" stays until the next check, although `mark_what_was_read`'s doc comment
  says "the count in the folder tree is where it shows". This is a different mechanism from the
  conversation snapshot; see Decisions for Pratik, item 2. (Investigation 2.)

### Spoken or shown

Yes: the Unread and Thread cells, and the Mark as Read wording on the menu, toolbar and context
menu.

## 3. Delete shows only after the next check

### Pratik's words

"Delete is the same as 2."

### What happens

In conversation view, Delete moves the message to Trash here, tells the server, and announces it.
The conversation row stays in the list, with its old count, until the next check. When it does
leave, nothing puts the cursor on another row.

### Cause

The same snapshot as issue 2.

1. Delete runs through `delete_these`, then `complete_here_then_tell_the_server`, which records the
   change in the store (since `2caa5111`, 11-07.1, a delete within an account completes here
   first) and calls `take_row_out_of_the_list` for each message. (Investigation 3.)
2. `take_row_out_of_the_list` removes the message from `s.messages`, then calls
   `tell_the_list_how_many` and `msg_list.refresh(true, None)`. Under conversation view
   `how_many_rows` counts `s.conversations` and `text_for` paints from it, so the count does not
   move and the same row is painted again. (Read at `dc592d07`; the view_state and virtual_rows
   halves from investigation 3.)
3. It then returns early under conversation view, before landing the cursor. Its own comment
   says the record "is left as it was there". (Read.) So when the row does leave at the next
   check, the control keeps the same row number and raises no focus event, and NVDA does not read
   the row the cursor is now on: #76 again, in the view he uses.

`4495589b` (11-06.1, 2026-09-18) saw conversation view inside `take_row_out_of_the_list` and
decided only the cursor, leaving which message a conversation row stands for to 11-08.

### Evidence

- `take_row_out_of_the_list` read at `dc592d07`: `s.messages.remove(idx)`, the count and refresh,
  then `if showing_conversations { return; }`.
- His profile: `show_conversations_by_default` is true, and all 50 rows of `tree_state` hold
  `thread_view = 1`. `moves_waiting` and `waiting_flag_changes` hold 0 rows, and the 2026-10-06
  and 2026-10-07 logs hold 0 ERROR lines and 0 lines of "could not be recorded here, so the
  server is". So his deletes took the made-here path. (Investigation 3; see "Where the
  investigations disagreed" for how the profile was reached.)
- Undo and a refused change already show on screen, because `show_where_they_came_back` and the
  `MovePutBack` arm both call `reread_folder_if_open`, which re-reads the conversations.
  (Investigation 3.)

### Introduced by

- `ffbc95a0` (2026-08-30) made the second copy and left `take_row_out_of_the_list` writing only
  the flat one.
- `4495589b` (2026-09-18) handled only the cursor under conversation view.
- Exposed for Pratik by `478f5885` (2026-09-20).
- Not `2caa5111` (11-07.1). Before it, the server-agreed delete also went through
  `take_row_out_of_the_list`, so a conversation row never left at once in any version.
  (Investigation 3.)

### What else the same cause breaks

- Every other change made here that takes a row out: Move, Report as Junk, Quick Steps, running a
  rule now over a folder, and the server-first fallback arms `MessageDeletedFromCache` and
  `MessageLeftTheFolder`. All go through `take_row_out_of_the_list`.
- All Inboxes in conversation view is worse. Nothing but landing there or switching the view
  re-reads its conversations, because `reread_folder_if_open` needs a folder id and All Inboxes
  has none. So even a check leaves a deleted or read conversation, and newly arrived mail,
  unchanged until he leaves and comes back. Plan B fixes the first half (changes made here); the
  second half (mail arriving) wants a ledger row.
- The comment in the Delete arm still says rows leave only once the server agrees, which has not
  been true since `2caa5111`. Plan B corrects it.

### Spoken or shown

Yes: the row leaves at once, and the cursor lands on the next conversation and is read.

## Issues 2 and 3: one plan, B

### Why one plan

Both are one defect: under conversation view the list is held twice in memory, and changes made
here keep only the flat copy current. Investigations 2 and 3 found that independently, each
listing the other's issue as a consequence. One function, called from every path that changes a
message here, fixes both, and two plans would write it twice.

### Red test (proposed, not run)

This merges the two investigations' proposals into one file:
tests/a_change_made_here_reaches_the_conversation_rows.rs, `#![cfg(windows)]`, one window session
behind a OnceLock, a desktop of its own, `WIXEN_NO_AUDIO=1`, and `WIXEN_MAIL_DATA` at a temporary
folder. It follows the shape of tests/deleting_a_message_lands_on_the_next_one.rs, because the
main window is never started in tests here and `toggle_read_state` and `AppHandles` are private.

Step 0, green, no change in behaviour: make public what the test must drive, as
`land_the_cursor_after` was for #76. That is the on-screen half of `mark_these_read` as a named
function, and `take_row_out_of_the_list` taking the store as one more argument. Keep
`FlagChange::Read(` inside `mark_these_read` for tests/each_set_command_has_a_quiet_do_half.rs and
tests/mark_as_read_says_which_way_it_will_go.rs.

Fixture: a real `MessageCache` with one account, INBOX and Trash. Conversation A is one unread
message, a1. Conversation B is two unread messages, b1 and b2. The state shows conversations,
its `conversations` come from `cache.conversations_in(inbox, account, TheWholeAccount, None)`, and
its `messages` from the folder listing through `apply_threading`. A real Report and Virtual
ListCtrl with Unread and Thread columns, whose text callback is the window's painter,
`virtual_rows::text_for` over `Listed`. Cells are read with `LVM_GETITEMTEXTW` sent to the list's
own window, which is what NVDA reads for a SysListView32 item.

Red cases:

1. `test_marking_a_conversation_read_takes_unread_off_its_row_at_once`: write b1 and b2 read in
   the store as `mark_these_read` does, call the on-screen half, refresh. B's Unread cell reads
   "". Today it reads "Unread".
2. `test_marking_a_conversation_read_leaves_no_unread_in_its_thread_cell`: B's Thread cell reads
   "2 messages". Today it reads "2 messages, 2 unread".
3. `test_mark_as_read_offers_mark_as_unread_once_a_conversation_is_marked`:
   `the_chosen_rows_have_unread` answers false for B's row. Today it answers true.
4. `test_a_conversation_deleted_here_leaves_the_list_at_once`: delete a1 in the store through
   `application::moves_waiting::what_happens_here` with a Delete-to-Trash `AWaitingMove`, which is
   what `complete_here_then_tell_the_server` does, then `take_row_out_of_the_list`. The control's
   count is 1, the state holds no row for A, and row 0 paints B. Today the count is 2 and A is
   still painted.
5. `test_the_cursor_lands_on_the_next_conversation_after_a_delete`: with A selected, after the
   delete the control's selected row is the conversation that followed A (or the one before it
   when A was last). Today the selection is left as it was.

Companions, green today, which show the instrument can see a change land:

6. `test_companion_a_mark_on_a_message_row_shows_at_once_under_the_message_view`.
7. `test_companion_a_delete_under_the_message_view_takes_the_row_out`.
8. `test_companion_the_store_holds_the_mark_while_the_row_does_not`: `conversations_in` read again
   after the writes answers `unread == 0`.

The red commit names cases 1 to 5, and the guard count check if it fires. The new target guards
src/presentation from outside, so it gets a guards/guards.toml record in the same plan.

### Fix

After any change made here, bring the touched conversation rows up to date from the store, which
already holds the change. Do it synchronously, before the list repaints, and say nothing.

1. **A narrowed listing in the store.** `a_conversation_listing` in
   src/data/message_cache/messages.rs takes its scope as a parameter; the folder's scope and
   every inbox's scope are two callers of one column list. Add a third use: the same scope
   narrowed to named conversations. One column list keeps one copy of the rules that decide a
   row's count, its message, its snippet and its sender order. (Derived from reading
   `a_conversation_listing`'s signature at `dc592d07`.) If narrowing the scope turns out to need
   a second copy of the SQL, read the whole listing instead and time it first at the size his
   2026-10-06 log recorded: 22,503 conversations over 34,407 loaded messages in one folder. Put
   the figure, its command and its date on docs/development/measurements.md.
2. **One catch-up function in wx_app.rs.** When the list shows conversations:
   - re-read the conversations the command touched, with the reach and sort the loaders use;
   - replace each on-screen row by its identity (`thread_id` and the account in `read_in`), in
     place, keeping the order;
   - remove the rows whose conversation came back empty, remembering their row numbers;
   - tell the control its count only if the count moved, and repaint only the rows that changed
     (`view_state::which_rows_changed`, which the `MailArrived` arm already uses);
   - announce nothing, because the one word at the key stands (#83);
   - when rows left, land the cursor with `land_the_cursor_after` on the conversation row numbers
     that left, and record `selected_message_index`. A set command remembers its rows in
     `a_set_leaving` at the key, as it does for flat rows, so a delete over five lands once.
   Under the message view it does nothing beyond what happens today.
3. **Call it once per command, after the loop:** `mark_these_read`, `star_these`, the label
   do-half, `write_the_flags` (Undo of a mark), `complete_here_then_tell_the_server`, and the
   `MessageReadToggled`, `MessageStarredToggled`, `MessageDeletedFromCache` and
   `MessageLeftTheFolder` arms.
4. Correct the Delete arm's comment about rows leaving only once the server agrees.
5. A changelog entry under `[Unreleased]`. Ledger 544's mark clause closes only once heard.

Two alternatives, both rejected by both investigations:

- **Send `ConversationsLoaded`, or call `reread_folder_if_open` as undo does.** The
  `ConversationsLoaded` arm ends by announcing the mailbox count on the "messages" topic whatever
  the reason for the read (read at `dc592d07`: the `what_a_mailbox_holds` and `announce_topic`
  lines after the `match why`). So "N conversations, M unread" would follow every M and every
  Delete, a second sentence after the key's word (guardrail 5). It also does nothing under All
  Inboxes, puts the cursor nowhere, and arrives after the wording refresh.
- **Subtract one from `unread` in memory.** The same query picks the row's `stands_for`, snippet,
  said-first phrase and sender order from the read flags (#31). The mark reaches the whole
  account while the count follows the reach setting. A count kept in memory would drift from the
  query.

### Files

- src/data/message_cache/messages.rs (the narrowed listing)
- src/presentation/wx_app.rs (the catch-up function, its call sites, `take_row_out_of_the_list`,
  the Delete arm comment)
- src/presentation/ui_types.rs, src/presentation/view_state.rs (if the catch-up needs a reason or
  a repaint helper of its own)
- tests/a_change_made_here_reaches_the_conversation_rows.rs (new)
- guards/guards.toml, docs/changelog.md, .planning/WINDOWS.md (ledger 544 note),
  docs/development/measurements.md (only if the whole listing is timed)
- read, not changed: src/presentation/virtual_rows.rs, src/presentation/message_rows.rs,
  src/application/conversations.rs, src/application/moves_waiting.rs, src/data/config.rs,
  tests/deleting_a_message_lands_on_the_next_one.rs, tests/a_move_completes_here_first.rs,
  tests/a_conversation_row_stands_for_one_message.rs,
  .planning/phases/11-reading-and-the-list/11-07-SUMMARY.md

Guard records are listed by both readings: records naming a changed file, and records whose
`before` or `after` text lies inside a function this plan moves or edits. Investigation 2 names
one already: the record anchored on `mark_these_read`'s opening.

## 4. Nothing acts on one message inside an opened conversation

### Pratik's words

"When the thread is opened, user is unable to use context menu or other commands that can act on a
message within the thread. Actions can only be taken on an entire conversation upon returning to
the mailbox."

### What happens

Opening a conversation shows the Conversation dialog: a tree with one row per message, and Open,
As Plain Text and Cancel. The menu key, Delete, M, Ctrl+R and every other message command do
nothing there. Back in the list, a command on the conversation row acts on the whole
conversation, as #27 decided.

### Cause

1. The tree binds only `on_selection_changed` and `on_item_activated`, and the three buttons each
   end the dialog. (Read at `dc592d07` in src/presentation/wx_thread_view.rs.) Its only output is
   a `ThreadChoice` saying which reading to open: `WholeConversation`, `AsHeadings`,
   `Message(id)` or `Cancelled`. The context menu, M and the label, Quick Step and saved-search
   keys are wired on `msg_list` and the folder tree, never on this tree. (Investigation 4.)
2. The main window cannot answer either. `show_modal` disables it, and wxWidgets stops
   accelerator translation at the modal dialog (target/release/wxWidgets/src/msw/evtloop.cpp near
   lines 92 to 100), so the main window's Del, Ctrl+R, Ctrl+Shift+S and Ctrl+Shift+V never fire.
   A dialog also sets `wxWS_EX_BLOCK_EVENTS` (src/common/dlgcmn.cpp near lines 124 to 127), so a
   menu line chosen inside the dialog never reaches `frame.on_menu` and the shared `dispatch`.
   (Investigation 4.)
3. Even with keys wired, every message command reads its target from the main list. The seven
   set commands (star, delete, mark read, label, move or copy, junk, Quick Step) go through
   `chosen_messages`, which reads the list's chosen rows and, under conversation view, expands a
   conversation row to every message in it. Reply, Reply All, Forward and compose read
   `selected_message_index` through `the_loaded_message_the_row_stands_for`, the conversation's
   representative message. Nothing can name "the message under the conversation tree's cursor".
   (Investigation 4.)
4. This became the everyday path when `ffbc95a0` made a list row stand for a whole conversation
   that never expands in place (D-01), and `478f5885` made that the default. Since then the
   dialog is the only place a conversation's messages are listed one by one.

### Evidence

- `build_thread_dialog`'s bindings read at `dc592d07`: `tree.on_selection_changed`,
  `tree.on_item_activated`, and `on_click` on Open, As Plain Text and Cancel. No key, menu or
  context-menu handler.
- `chosen_messages` has 7 callers: the star arm, the delete arm, `run_the_quick_step_at`,
  `toggle_read_state`, `label_the_message`, `move_or_copy_message` and
  `report_the_chosen_as_junk`. One target rule there covers every set command. (Investigation 4.)
- `open_conversation_again` matches only the four `ThreadChoice` readings, and its `again`
  closure reopens the dialog from the `nodes` captured at first open, not re-read from the state.
  (Investigation 4.)
- `gh issue list` searched for "conversation" in all states found no issue about commands inside
  the conversation. #27, #30, #31 and #92 are the closed decisions this sits beside.
  (Investigation 4.)

### Introduced by

No single commit regressed it: the dialog has been a reading chooser since `cab6da7e`
(2026-07-27) and `717b0c3b` (2026-07-28, Give a conversation real headings, in a window of its
own). It became reachable when `ffbc95a0` (2026-08-30) made a row stand for a whole conversation,
and everybody's default path at `478f5885` (2026-09-20).

### Red test (proposed, not run)

Two tests, one per layer, committed red together.

1. The dialog, a live window:
   tests/the_conversation_window_acts_on_the_message_under_its_cursor.rs. One `#[test]`, one
   `wxdragon::main`, a desktop of its own (the pattern in tests/a_quick_step_runs_over_the_selection.rs),
   `WIXEN_NO_AUDIO` set, the frame never shown. Nodes from
   tests/tree_dialogs_resolve_the_row_somebody_is_on.rs: messages 11, 22 and 33, where 33 replies
   to 11. Build `wx_thread_view::build_thread_dialog(&frame, "Quarterly report", &nodes, Some(33), None)`.
   - A, green today: the choice reads `Message(33)`.
   - B, red: WM_KEYDOWN of VK_DELETE posted to the tree's window. The choice reads
     `Act { command: ID_DELETE, message: 33 }` and the dialog has ended. Today it stays
     `Message(33)`.
   - C, red: WM_COMMAND with `ID_MARK_READ` and high word 0, which is where the menu key's popup
     delivers its line. The choice reads `Act { command: ID_MARK_READ, message: 33 }`. Today
     nothing answers it.
   - D, companion: select the row for 22, then repeat C. The choice names 22, so a dialog that
     answers a constant is seen.
   - E, companion, green: the root row still answers `AsHeadings` on Enter.
   The red commit adds the `Act` variant with no producer, so the file compiles and fails.
2. The target, pure and windowless, in src/application/choosing_messages.rs:
   `test_a_message_chosen_inside_a_conversation_is_the_whole_set`. A new
   `what_the_command_is_for(acting_on, rows, members_of)` with rows `[0]` standing for a
   conversation of 11, 22 and 33, and `acting_on` naming 33, answers a set of just 33 and no
   conversations. Companion: with `acting_on` empty it equals `what_the_selection_holds`, so #27's
   whole-conversation rule stands.

### Fix

From investigation 4, with one addition marked.

1. **The dialog** (wx_thread_view.rs):
   - Add `ThreadChoice::Act { command, message }`.
   - Give the tree the menu key: the Applications key and Shift+F10. Move `wire_context_menu` from
     wx_app.rs to src/presentation/wx_context_menu.rs as `pub(crate)` and reuse it. It offers
     `context_menu::entries_for_messages` for the node under the cursor, and nothing on the root
     row.
   - Bind Delete, Shift+Delete and M to `ID_DELETE`, `ID_DELETE_OUTRIGHT` and `ID_MARK_READ`,
     consumed, as the list has them.
   - Add `dlg.on_menu`: when the choice is `Message(id)`, set it to `Act` and end the dialog with
     a new `ID_ACT`.
   - No labelled control is added, so no mnemonic letter is spent.
2. **The other command keys** (this writer's addition; investigation 4 bound only Delete,
   Shift+Delete and M, and Pratik said "context menu or other commands"). Ctrl+R, Ctrl+L,
   Ctrl+Shift+S, Ctrl+Shift+V, Ctrl+Shift+J, the label digits and the Quick Step keys also need to
   work in the tree. wxdragon 0.9.17 binds no accelerator table: a search of its src for
   "accelerator" found 6 hits, all in widgets/webview.rs (2026-10-14). The main window's keys
   live in its menu labels, such as "&Delete\tDel", which a dialog without a menu bar cannot
   use. So the tree's key handler maps keys to command ids, and either the handler and the main
   window's labels read one list, or a test compares the two so they cannot drift. The planner
   chooses which.
3. **The target** (wx_app.rs and choosing_messages.rs):
   - Add `WxUIState::acting_on`, empty except while a command from the conversation runs.
   - `chosen_messages` answers through `choosing_messages::what_the_command_is_for`. When
     `acting_on` is set, the set is that one message, built from its stored row. This covers all
     seven set commands at once.
   - Add `WxUIState::the_message_a_command_is_for()`, which prefers `acting_on` over the row's
     message. Use it in `start_reply`, `msg_info` and `open_compose`. When `acting_on` is set,
     quote `cache.get_message_body(acting_on)`, not `s.message_preview`.
4. **The loop** (`open_conversation_again`):
   - An `Act` arm sets `acting_on`, calls the shared `dispatch(command)`, and clears `acting_on`
     on every path.
   - The arm reaches `dispatch` through a late-bound cell filled where `dispatch` is built, the
     `answers_the_tray` pattern, because `msg_list.on_item_activated` is wired before `dispatch`
     exists.
   - It then reopens the conversation from `conversation_nodes` rebuilt from the state, never from
     the captured snapshot, with the cursor on the same message or its neighbour when the message
     has left. Pass the thread id and account id in for that.
   - Plan B's catch-up function runs inside the command, so the list row under the dialog is right
     when it closes.
5. **Reading marks the message read.** `mark_what_was_read` passes the row's representative as
   the message to mark (read at `dc592d07`: `selected_unread` comes from
   `the_loaded_message_the_row_stands_for`), and `reading_habits::whether_to_mark_read` requires
   the message whose reading began to equal it (investigation 4). So an unread message other than
   the representative, read from the tree, is never marked read. The message whose reading began
   is the one to mark.
6. **Documents:** docs/KEYBOARD_SHORTCUTS.md, its Conversations and Thread View sections, say what
   the menu key, Delete, M and the other keys do in the conversation. A changelog entry under
   `[Unreleased]`. Re-measure the guard records naming wx_thread_view.rs, choosing_messages.rs and
   the `chosen_messages` arms, by both readings.

What it must not break: tests/tree_dialogs_resolve_the_row_somebody_is_on.rs,
tests/tree_rows_leave_no_registry_entry.rs, tests/a_conversation_row_stands_for_one_message.rs
(#31 `open_on`), the Conversation scan target, Escape back to the list, Enter opening a message or
the whole conversation, a conversation row in the list acting on the whole conversation when
`acting_on` is empty (#27, #30, D-07), and D-01, the list never expanding in place.

### Files

- src/presentation/wx_thread_view.rs
- src/presentation/wx_app.rs
- src/application/choosing_messages.rs
- src/presentation/wx_context_menu.rs
- docs/KEYBOARD_SHORTCUTS.md, docs/changelog.md, guards/guards.toml
- tests/the_conversation_window_acts_on_the_message_under_its_cursor.rs (new)
- read, not changed: src/application/context_menu.rs, src/application/reading_habits.rs,
  src/presentation/wx_reader.rs, tests/tree_dialogs_resolve_the_row_somebody_is_on.rs,
  tests/a_quick_step_runs_over_the_selection.rs

### What else the same cause breaks

- The tree's "Unread." labels come from the snapshot taken at first open, and every return trip
  reuses it, so a message read or marked meanwhile still reads "Unread." until the conversation is
  closed and reopened from the list. Plan C's rebuilt nodes fix this. (Investigation 4, inferred
  from source, not heard.)
- An unread message other than the row's representative, read from the tree, is never marked
  read. Plan C fixes this (step 5). To the ear it sounds like issue 2.
- The reader window and the headings window offer no message command at all, for a lone message
  too. In the flat view this is hidden, because closing returns to a row that is that message.
  Plan C does not add commands to them; worth a ledger row.
- Reply from the list after reading message N of a conversation acts on the row's
  representative, not on N. Whether Ctrl+R on a conversation row in the list should mean anything
  else is a design question, not part of this plan. (Investigation 4.)

### Spoken or shown

Yes: a context menu and command keys inside the conversation, and what the tree lists after a
command.

## Where the plans sit in phase 13.2

When this was written, GROUPING.md chained 29 plans one per wave, then numbered 13.2-01 to
13.2-29, with the full gate in the last. These three should go at the front of the chain:

- Pratik meets all four every time he uses the mailbox, in the view he uses.
- Phase 14's sittings run copy and move, then delete, next (merge `dc592d07`). Issue 3 is what the
  delete sitting hears, and GROUPING.md already orders its own plans by "what phase 14's sittings
  prove".

Numbering is the orchestrator's call. Two ways, with their costs:

- Renumber the chain so these are 13.2-01 to 13.2-03 and the rest shift by three. Five plans were
  written already (then 13.2-01, -02, -17, -25 and -26), and GROUPING.md's tables, sibling notes
  and ledger table carried the old numbers. Grep every old id across the staged folder and remap
  each hit before anything else.
- Keep the existing numbers and give these a set of their own that runs first, with GROUPING.md's
  chain starting after them.

Settled 2026-10-14: the first way. A, B and C are 13.2-01, 13.2-02 and 13.2-03 with 13.2-03.1,
and the audit's plans are 13.2-04 to 13.2-32. The overlaps below name the audit's plans by their
new numbers; each was three lower when this was written.

Overlaps with plans already grouped:

- **13.2-09** edits `reread_folder_if_open` and the count lines spoken after a re-read. Plan B
  extracts nothing from those, but both decide what a re-read says. Whichever lands second reads
  the other's change, and plan B's quiet catch-up is a precedent 13.2-09 can use.
- **13.2-11** edits the flag push's Ok branch and the replay's put-back. Plan B calls its catch-up
  in the `MessageReadToggled` arm a put-back arrives through.
- **13.2-22** joins announcements released together. Plan B adds no announcement, so there is
  nothing new to join; its tests should still be read by 13.2-22's planner.
- **13.2-07** changes where Delete goes. Plan B's delete case uses a Delete-to-Trash
  `AWaitingMove`; 13.2-07's planner checks the fixture still fits.
- Every plan here edits src/presentation/wx_app.rs. Guard anchors inside moved or edited blocks
  move, and each later plan lists them by both readings.

Version, corrected 2026-10-14 when the phase's plans were brought together: the version does
not move in this phase. A tester build is not a cut, on Pratik's decision of 2026-09-17, and
13.2-04's premise 9 sets out the record, which the phase README quotes under "The version".
Until then this paragraph said that, if no earlier plan had moved the prerelease counter, the
first green commit here moves `1.0.0-alpha.1` to `1.0.0-alpha.2`, and that 13.2-20's staged
plan carried the same rule, which 13.2-20's premise 7 has since withdrawn. Cargo.toml reads
`version = "1.0.0-alpha.1"` at `dc592d07`.

Requirement candidates, to be read before naming: LIST-21 (a move or delete completes here first)
and THREAD-02 (the rows a re-read changes) for plan B. None found for A or C without reading.
The plans as written name LIST-06 (13.2-01), LIST-21 and LIST-04 (13.2-02), LIST-05 (13.2-03),
and LIST-05 and LIST-03 (13.2-03.1).

New ledger rows, numbered by the executor at commit time: the root fix for issue 1, a tree that
holds only this folder's members while the row counts the whole account, All Inboxes not re-read
after a check, and the reader and headings windows offering no message commands.

## Decisions for Pratik

Plain questions; a combination of answers is expected.

1. **After you mark a conversation read while the list is sorted by Unread, should the row stay
   where it is until you next open the folder, or move at once?** Recommended: stay where it is,
   so the cursor and what you just heard do not jump. Plan B is written that way.
2. **Should the unread count in the folder tree ("Inbox, 5 unread") also change at once?**
   Today it changes at the next check, because it shows the server's count. Recommended: yes,
   from this computer's own count, with the next check correcting it. It is a different mechanism
   from the list rows, so it would be a small plan of its own after B.
3. **Inside the conversation, after a command, should the window say "Conversation, 3 messages"
   again, or say only the command's own word and keep your place?** Recommended: only the
   command's word (guardrail 5).
4. **When you delete the last message of a conversation from inside it, should you land back in
   the mailbox on the next conversation?** Recommended: yes.

## Where the investigations disagreed

- **Whether a delete waits for the server.** Investigation 4 wrote that a delete waits for the
  server before the row leaves, so a reopened tree would still list it. Investigation 3 found
  that since `2caa5111` a delete within an account completes here first and takes the message
  out of the loaded rows at once; only the conversation row lags. This writer read
  `take_row_out_of_the_list` and agrees with investigation 3. Plan C relies on it: a tree rebuilt
  from the state no longer lists a deleted message.
- **The shape of the fix for 2 and 3.** Investigation 2 proposed replacing the touched rows in
  place, keeping the order and never changing the count. Investigation 3 proposed a full re-read
  with a new reason that tells the count when it moves and lands the cursor. Neither covers both
  issues alone: the first cannot remove a row, and the second re-reads the whole folder on every
  key press, which was 22,503 conversations in the folder his 2026-10-06 log recorded. Plan B takes the narrowed re-read, the in-place replacement, the
  removal and the cursor landing.
- **Which commit introduced 2.** Investigation 2 names `7d2c64f4`; investigation 3 names
  `ffbc95a0`. Both are right about different halves: `ffbc95a0` made the second copy, and
  `7d2c64f4` pointed marks at conversation rows without working them out again.
- **Whether his settings were read.** Investigation 1 could not read them: python launched from
  Bash saw the July package copy of `app_config.json` (886 bytes, dated 2026-07-30), so it took
  the view from his log counts. Investigation 2 did not read them, saying its brief required
  that. Investigation 3 read the live profile through the administrative share for drive C on
  localhost, in this form:

  ```
  \\localhost\C$\Users\prati\AppData\Local\wixen-mail
  ```

  All three reach conversation view by separate readings: the log counts, the profile's
  `show_conversations_by_default` and its 50 of 50 `tree_state` rows, and Pratik's own issue 4.

## Notes for whoever plans these

- The memory note `feedback_bash_sees_a_stale_appdata_copy` says python from this harness sees
  the live profile. Investigations 1 and 3 both found that python launched from Bash now also
  sees the July copy, and only the localhost share path above reached the live files (a 956 MB
  database and settings written 2026-10-06). The note wants correcting; this writer did not
  change it.
- Investigations 3 and 4 used sed read-only, and investigation 4 also awk, against their briefs.
  Nothing was written. Neither logged an observation, because no id block was reserved for this
  fan-out. The pattern already has many open observations; the orchestrator should log this
  instance or reserve blocks next time.
- None of the red tests above has been run. Each plan runs its red cases before its first green
  commit and confirms they fail for the reason given here.

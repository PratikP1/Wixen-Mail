---
phase: 11-reading-and-the-list
plan: 07
subsystem: the message list's selection, the seven commands over it, the cursor handler, a set's landing, guards, pages
tags: [multi-select, shift-arrows, select-all, one-sentence, conversation-row, reach, bound, focus-event, virtual-list, land-once, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-06 merged at fe143d46 (toggle_read_state behind the command and M, refresh_mark_read_wording, list_keys::wire_letter); 11-06.1 merged at 0ed2c1a1 (land_the_cursor_after over a set, put_the_cursor_on clearing and setting, UIUpdate::Shown, say_the_one_word, show_or_say_what_happened_next); 11-06.2 merged at 116968fb (list_arrival on the focus event); 11-06.3 merged at 1a973b46; main clean at 15407b1e when the branch left it"
provides:
  - "application::choosing_messages: MessageRef, Members (a message row or a conversation row with its name), Chosen (the messages each once in the rows' order, the conversations named, how many messages they contributed), what_the_selection_holds(rows, members_of), SetCommand and reach_for(command, setting) -> AConversationReaches, Outcome and what_was_done(chosen, outcome), what_is_being_done(doing, chosen), deleting_asks(chosen), too_many(count) over editing::MOST_ROWS_WORTH_SELECTING, what_mark_read_does and what_star_does; 18 cases, 2 records on the library"
  - "wx_app.rs: the message list built without SingleSel; the cursor handler on on_item_focused, its body as it was; pub chosen_rows(list) walking get_next_item; chosen_messages(state, cache, list, reach) turning the rows or the cursor row into a Chosen, a conversation row's messages read through messages_in_conversation under the reach; the_open_folder_and_its_account; how_far_a_conversation_delete_reaches from the D-07 setting; the Star arm, toggle_read_state(app, a11y, how, frame, toolbar, list, cache), label_the_message(.., list, number), the Delete arm and move_or_copy_message(.., list, copying) over the set, each refusing above the bound and saying one sentence; delete_the_conversation_row gone; spawn_folder_move(app, moving, chosen, into, copying) as one worker over the set with every outcome shown and one sentence at the end; ASetLeaving and WxUIState::a_set_leaving; take_row_out_of_the_list landing once after the last row of a set; a_refusal_ends_the_wait_for_a_set from the ErrorOccurred and CommandRefused arms; the_chosen_rows_have_unread(s, rows) and refresh_mark_read_wording(frame, toolbar, state, list) reading the selected rows, called after Select All too; switch_the_view holding every selected row; 199 tests before and after, 88 records"
  - "application::tagging: spoken and all_removed gone with the arm that said them, nothing_there's doc says where the sentences are; 12 tests, 0 records"
  - "tests/every_command_acts_on_the_selection.rs: eight source readings over what_ships with six companions over a snippet shaped as the window should be; a built virtual list without SingleSel counting the focus and selection events through three steps and the walk over three selected rows; one ignored timing over a cache of 5,000; 16 tests and 1 ignored, 3 records name it"
  - "guards/guards.toml: 948 records, census 798 + 150; five new records measured, six rewritten onto the new code and measured again"
  - "docs/development/measurements.md: the 5,000-row row, 75 ms, dated 2026-09-19 at 9155c6be, with the paragraph saying what it bounds; docs/KEYBOARD_SHORTCUTS.md, docs/USER_GUIDE.md, docs/changelog.md: the keys, the commands over a set, the bound and the conversation-row rule"
  - ".planning/WINDOWS.md: 544 unrun-verify, what only the tester's ear settles; 545 deviation, the one-place check's file-wide exemption"
affects: [11-07.1, which completes a move and a delete here first over the set this plan reads and inherits a_set_leaving and the batch worker; 11-08, which decides which message a conversation row is and reads chosen_messages' conversation branch and the cursor handler; 11-13, which reads the sentences what_was_done and what_is_being_done make; 11-12, which reads LIST-04's and LIST-05's lines and the pages; whoever hears a selection of many read]

actuals:
  tokens: 51275
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A command over a selection reads the set off the control at the key and keeps it nowhere, folds it into one set through a rule that runs without a window, refuses above one bound the store already had, does per message what it did for one, and says one sentence with the count first"
    - "On a list that selects more than one, the cursor is the focused row and the cursor handler is on the focus event, because the selection event fires once per row selected and not when the focus moves onto a row already selected; the measurement that decides it is a built list counting both events through the three steps a key makes"
    - "A set leaving the list one server answer at a time is remembered at the key by row id and index, and the cursor lands once after the last of them through the rule that takes a set; a refusal ends the wait"
    - "A batch over one route is one worker taking the messages in turn, every per-message outcome written for the eye, the refusals counted and logged, and one sentence spoken at the end; a batch that all failed is spoken as a refusal with the first reason"

key-files:
  created:
    - src/application/choosing_messages.rs
    - tests/every_command_acts_on_the_selection.rs
  modified:
    - src/application/mod.rs
    - src/application/tagging.rs
    - src/presentation/wx_app.rs
    - tests/mark_as_read_says_which_way_it_will_go.rs
    - tests/moving_through_the_list_marks_nothing_read.rs
    - guards/guards.toml
    - docs/development/measurements.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/phases/11-reading-and-the-list/deferred-items.md

key-decisions:
  - "The cursor handler is on the focus event, not the selection event the plan kept: a built list showed the selection event firing once per row of a Select All and not at all when the focus moves onto a row already selected, which is Shift+Up shrinking the range; the handler's body is what it was and the three readers that anchored on it were rewritten in place"
  - "what_the_selection_holds takes the rows and a members closure and no Showing: the closure is built by the window from the view, which is the only place that knows whether a row is a message or a conversation, so the parameter would have been a second answer to the same question; reach_for is a free function over the command and the setting, since it reads nothing of the set"
  - "A conversation row's members come from the cache through messages_in_conversation under the command's reach, the walk the conversation row's delete always used, rather than from conversation_nodes: the reach for Mark as Read, Star and Label is the whole conversation, whose messages in other folders are not in the loaded rows"
  - "The Mark as Read words are decided from the selected rows' own flags on screen, the row's unread count for a conversation row, and not from chosen_messages: the words are asked on every arrow key and chosen_messages reads the cache for a conversation row; what_mark_read_does decides the toggle itself"
  - "A conversation row of one message counts as a message and asks nothing, the list's own rule that a conversation of one is not a conversation; a delete of one conversation row and nothing else asks the D-07 question through view_state::deleting_a_conversation_asks, the one owner of that sentence, and a set holding more than that asks with the counts"
  - "delete_the_conversation_row is gone into the Delete arm: one command over one set, the question kept when a conversation row is in the set, the spoken intent line replaced by the one word and the shown line the way a delete of a message says it since 11-06.1"
  - "A set leaving the list is remembered in WxUIState::a_set_leaving by row id and the index at the key, and take_row_out_of_the_list lands once when the last has left, through land_the_cursor_after over the set's rows; a refusal arriving on either refusal arm ends the wait and lands after the rows that left"
  - "spawn_folder_move keeps its name and takes the set: one worker, each message's held session asked for as it comes, a set's outcomes all shown, a message that got nowhere counted and logged with its reason, one sentence spoken at the end; nothing went is spoken as a refusal with the first reason; one message keeps 11-06.1's shown-or-spoken outcome"
  - "The one sentence rides announce once and the shown channel for the eye, on Star, Mark as Read and the Labels alike; the labels' second spoken copy through the status topic is gone with it"
  - "Star over a set stars when any chosen message is not starred, else unstars, the rule Mark as Read has; a label goes on when any chosen message lacks it, else comes off, and only the messages that need the change are written and sent"
  - "The two label sentences for one message, Work added and 3 labels removed, went with the arm that said them; tagging.rs has no records, so its two tests went with them"
  - "A batch that did not all go says how many did not as not moved or not copied, not the plan's not sent, since nothing in a move is sent"
  - "The summed sentence for a set that all failed is spoken as a refusal with the first reason; the plan named no shape for it"
  - "The Mark as Read words are refreshed after Select All on the message list too, since Select All changes what the command acts on and the control raises no focus event for it; the plan named three sites and this is a fourth"
  - "Six records whose before the arms moved from under were rewritten onto the new code and measured again rather than retired, each the same one test red, and each says so in its comment"

patterns-established:
  - "When a check has an exemption for a transient state, read what it exempts before trusting its green over a change that moved a lot of code, and count the fact independently once: the one-place check's exemption is keyed on the file and was blind for six records"

requirements-completed: [LIST-05, LIST-04]

coverage:
  - id: D1
    description: "What a selection holds, each message once and a conversation row's messages counted; the reach per command; the one sentence with the singular right and the conversations named; the shown line and the question; the bound read from Select All's; the two toggles' direction"
    requirement: LIST-05
    verification:
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_message_rows_are_the_messages_in_the_rows_order"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_a_conversation_row_contributes_every_message_of_it_and_is_counted"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_a_message_named_by_two_rows_is_held_once_where_it_was_first"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_a_delete_reaches_as_far_as_the_setting_says"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_marking_starring_and_labelling_reach_the_whole_conversation_whatever_the_setting"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_a_move_or_a_copy_reaches_this_folder_only_whatever_the_setting"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_the_sentence_names_the_conversations_when_one_was_in_the_set"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_the_bound_is_select_alls_and_is_refused_only_above_it"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a message two selected rows name is chosen once, not once per row' and 'a command over more than the bound is refused with a sentence, not run', measured 2026-09-19 on the library"
        status: pass
    human_judgment: false
  - id: D2
    description: "The list selects more than one; the seven commands read the set, refuse above the bound and say one sentence; the cursor commands read no set; the cursor follows the focused row; the words follow the selected rows; a set leaving lands once; the events a built list raises; the cost at the bound"
    requirement: LIST-05
    verification:
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_the_message_list_is_built_without_single_selection"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_star_mark_as_read_and_the_labels_act_on_the_selection_with_one_sentence"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_delete_acts_on_the_selection_asks_for_a_conversation_and_says_delete_once"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_move_and_copy_act_on_the_selection_and_the_batch_says_one_sentence"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_the_cursor_commands_act_on_the_row_the_cursor_is_on_and_read_no_selection"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_the_cursor_follows_the_focused_row_and_the_handler_writes_the_index"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_the_mark_as_read_words_follow_the_selected_rows"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_a_set_leaving_the_list_lands_the_cursor_once_after_the_last_row"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#the_built_list::test_the_focus_event_fires_once_per_cursor_move_and_the_selection_event_once_per_row"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#the_built_list::test_the_walk_answers_every_selected_row_in_order"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the message list selects more than one row, not one however many Shift chose', 'mark as read acts on every selected message, not the cursor row alone' and 'delete over a selection refuses above the bound, not runs whatever the size', measured 2026-09-19 on the target"
        status: pass
      - kind: command
        ref: "cargo test --release --test every_command_acts_on_the_selection -- --ignored --nocapture --test-threads=1 -> 75 ms for 5,000 read marks, the row on docs/development/measurements.md; cargo test --lib presentation::wx_app:: -> 199 passed; grep -v '^\\s*//' src/presentation/wx_app.rs | grep -c 'ListCtrlStyle::SingleSel' -> 0; grep -c 'selected_message_index' -> 32 before, 28 after"
        status: pass
    human_judgment: false
  - id: D3
    description: "A conversation row contributes every message of the conversation to Mark as Read, with one announcement saying how many"
    requirement: LIST-04
    verification:
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_marking_starring_and_labelling_reach_the_whole_conversation_whatever_the_setting"
        status: pass
      - kind: unit
        ref: "src/application/choosing_messages.rs#test_mark_as_read_marks_read_when_any_is_unread_and_unread_when_none_is"
        status: pass
      - kind: integration
        ref: "tests/every_command_acts_on_the_selection.rs#test_star_mark_as_read_and_the_labels_act_on_the_selection_with_one_sentence"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_the_menu_item_and_the_key_run_one_toggle_and_the_key_says_one_word"
        status: pass
    human_judgment: false
  - id: D4
    description: "The keys, the commands over a set, the bound and the conversation-row rule on the pages; the changelog entry; the ledger"
    requirement: LIST-05
    verification:
      - kind: command
        ref: "grep -c 'Shift+Down' docs/KEYBOARD_SHORTCUTS.md -> 2; grep -c 'Selecting more than one message' docs/USER_GUIDE.md -> 1; grep -c '#30' docs/changelog.md -> 1; ledger 544 and 545 in both halves; cargo test --test house_style -> 74; --test docs_links -> 6; --test the_planning_files_agree_with_themselves -> 16; --test the_words_that_say_nothing -> 9; --test a_key_is_documented_where_the_surface_that_binds_it_is -> 3"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether NVDA says selected and not selected as the range grows and shrinks and the count after Ctrl+A, whether one sentence after a command over many is enough by ear, whether Delete over five lands once after the set, whether M on a conversation row is heard to mark the thread, and the refusal above the bound"
    requirement: LIST-05
    verification: []
    human_judgment: true
    rationale: "Ledger 544; the comments on #30 and #27 list it; nothing here has been heard, and no binary was started"

duration: 103min
completed: 2026-09-19
status: complete
---

# Phase 11 Plan 07: The list selects a set, and every command acts on it Summary

**The message list selects more than one message the way every Windows list does:
Shift with Up, Down, Home and End extends the selection, Ctrl+A selects everything shown up
to 5,000, and Delete, Delete Permanently, Move to, Copy to, Mark as Read or Mark as Unread,
Star or Unstar and the Label commands act on every selected message and say one sentence
with the count, "3 messages marked read", "4 messages moved to Archive". A conversation row
stands for every message in it, so Mark as Read on a thread marks the whole thread and says
"1 conversation, 5 messages marked read", which is the last sentence of #27; Move to and Copy
to take the conversation's messages in the folder being read, and Delete keeps the D-07
setting's reach and asks first. No command runs over more than 5,000 messages, the bound
Select All already had; marking 5,000 read in the cache cost 75 ms on a release build. The
list was built `SingleSel` until this branch, so Shift+Down moved the selection instead of
growing it, Ctrl+A selected one row while saying it had selected them all, and every command
read one index. The cursor is the focused row now, and the cursor handler moved from the
selection event to the focus event on a measurement the built list made before the arms were
written: growing the selection raises a selection event per row and no focus event, so the
handler the plan kept would have loaded five thousand bodies for one Ctrl+A, and the focus
moving onto a row already selected, which is Shift+Up shrinking the range, raises no
selection event at all. A set leaving the list lands the cursor once, after the last row of
it, through 11-06.1's rule over the set's rows. Nobody has heard a selection of many read.**

## Performance

- **Duration:** 103 min from the start recorded at 02:18:11Z, when the reading of the plan,
  the README, the twelve summaries and the tree began, to the merge's hook finishing at
  04:01:03Z; the branch's first commit at 02:35:51Z; the summary and the planning files after.
  About 14 min 26 s was guard measurement in four foreground runs (213 s for the two library
  records named bare, which the runner refused as never run; 248 s for the same two named by
  module path, 75 s and 77 s after a 96 s read of what already fails; 67 s for the three
  target records, 18 s, 22 s and 21 s; 338 s for the six rewritten records, 14 s, 79 s, 92 s,
  18 s, 20 s and 21 s); about 14 min 32 s the five hook runs on the branch (153 s, 119 s, 231
  s, 241 s, 128 s); 358 s the whole gate on the branch; 361 s `main`'s hook at the merge; 78 s
  the release build of the timing and 0.6 s its run, twice; 0 s waiting for the desktop,
  since no live-window key test went red on any run.
- **Started:** 2026-09-19T02:18:11Z
- **Merged:** 2026-09-19T04:01:03Z at `b35a40cd`
- **Tasks:** 3
- **Files modified:** 14, two created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat 15407b1e..44d86fc2 -- Cargo.toml Cargo.lock` (T-11-SC)
- **Actuals:** `tokens: 51275` is `git diff 15407b1e..44d86fc2 | wc -c`, 205,100 characters
  over four, the branch's own diff against the commit it left `main` at, `wx_app.rs` alone
  1,473 lines of it; the estimate's `tokens` was 39,000 and `raw_tokens` 90,000, so the
  factor is 1.31 against the former and 0.57 against the latter, the largest of the phase so
  far because six arms and a handler were rewritten rather than added to.

## What landed

**Task 1.** `application::choosing_messages`. `MessageRef` is a message as a command over a
set needs it: the row id every write is keyed on, the uid, the subject, read and starred.
`Members` is what one selected row stands for, a message row itself or a conversation row
with its name and its messages within the reach the command asked for. `Chosen` is the
messages, each once by row id in the rows' order, the names of the conversation rows that
contributed more than one message, and how many messages those contributed, so a delete of
one conversation row and nothing else can be told from one with a message row beside it;
`the_one_message` answers only for one message row, which is what the one word on M and the
subject on the shown line ask. `what_the_selection_holds(rows, members_of)` folds the rows
through the closure; a conversation of one is not a conversation, the list's own rule, so a
row that answered one message counts as a message. `reach_for(command, setting)`: Delete the
setting's `counted_the_same_way`, Mark as Read, Star and Label `TheWholeAccount`, Move and
Copy `ThisFolderOnly`. `what_was_done(chosen, outcome)` opens with the count, "3 messages",
"1 message", "2 conversations, 9 messages", then the outcome: marked read or unread, starred
or unstarred, labelled Important, Important removed from, 5 labels removed from, There were
no labels on the, moved to Archive, copied to Work, and for a batch that did not all go "2
messages moved to Archive, 1 not moved". `what_is_being_done(doing, chosen)` is the shown
line, the subject for one message and the count for a set. `deleting_asks` is nothing for
message rows, D-07's question through `view_state::deleting_a_conversation_asks` for one
conversation row alone, and "Delete 4 messages? 1 conversation is among them." otherwise.
`too_many` is nothing at or below `MOST_ROWS_WORTH_SELECTING`, read from `editing.rs` and not
written again, and above it "5,001 messages are selected. This can do 5,000 at once at most.
Select fewer." with the thousands separator. `what_mark_read_does` and `what_star_does`
answer the way any message that needs it says. Eighteen cases; two records on the library.

**Task 2.** In `wx_app.rs`: the list built without `SingleSel`, with the reason dated. The
cursor handler on `on_item_focused`, its body as it was, capturing the list for the refresh.
`chosen_rows(list)`, public, walks `get_next_item(-1, All, Selected)` until it answers below
zero. `chosen_messages(state, cache, list, reach)` reads the rows, or the cursor row when
none is selected, and under the flat view answers each message row; under conversation view
it reads each conversation's ids through `messages_in_conversation` under the reach and each
message through `get_message`, answering a sentence when the cache could not answer or no
folder of an account is open. `the_chosen_rows_have_unread(s, rows)` reads the rows' own
flags on screen, and `refresh_mark_read_wording` takes the list and asks it; the context
menu closure asks the same. `toggle_read_state` takes the list and the cache, reads the set
under `TheWholeAccount`, refuses through `too_many`, decides the direction through
`what_mark_read_does`, and for each message flips the row on screen, writes through
`write_flags_or_put_the_row_back` and stops at the first refusal, and sends the flag through
`spawn_server_change`; then one refresh, one sentence announced at Normal (the one word
under M for one message row, `what_was_done` otherwise), the sentence shown, `Confirmed`
signalled once, the words refreshed. The Star arm the same shape with `what_star_does`.
`label_the_message` takes the list, reads each message's labels once, turns a label on when
any lacks it and writes only the messages that need the change, or takes every label off
every message counting them, and says one sentence. The Delete arm reads the set under the
D-07 setting's reach through `how_far_a_conversation_delete_reaches`, refuses through
`too_many`, asks through `deleting_asks` with the dialog `delete_the_conversation_row` used,
remembers the set's rows in `a_set_leaving` under the flat view when more than one, says
"Delete" once, shows the intent line, and sends each message down the route one always went:
`cancel_if_queued`, `delete_if_local`, `spawn_server_change`. `delete_the_conversation_row`
and its guard arm are gone. `move_or_copy_message` takes the list, reads the set under
`ThisFolderOnly`, refuses through `too_many`, asks the folder once for the first message's
account, reads each message's folder, remembers a move's rows in `a_set_leaving`, says the
one word once, shows the intent line and hands `spawn_folder_move` the set. That worker asks
`owner_of` per message under one lock, then in one blocking task takes the messages in
turn, each branch answering the outcome and whether the message went: for a set every
outcome is shown and a row that left is taken out, a message that got nowhere is counted and
logged with its reason, and one `StatusUpdated` says the summed sentence, or one
`ErrorOccurred` with the first reason when nothing went; one message keeps
`show_or_say_what_happened_next`. `ASetLeaving` holds the rows still to leave by id and index
and the indices of the rows that left; `take_row_out_of_the_list` asks it, lands nothing for
a row of a set with rows still to leave, and lands once over the set's rows when the last has
left; `a_refusal_ends_the_wait_for_a_set` runs from the `ErrorOccurred` and `CommandRefused`
arms. `switch_the_view` holds every selected row's id in `KeptSelection`, which D-11 wrote for
a set. The Select All arm refreshes the words on the message list. 199 tests before and
after, the folder-tree reading's anchor rewritten in place; the two targets' anchors the
same.

`tests/every_command_acts_on_the_selection.rs`: eight readings over `what_ships`, each a
function over the text with a companion planting the fault into a snippet shaped as the
window should be, holding the list built without `SingleSel`; the Star arm,
`toggle_read_state` and `label_the_message` to `chosen_messages(`, `too_many(` and
`what_was_done(` and the toggle to `what_mark_read_does(`; the Delete arm to those, to
`deleting_asks(`, `say_the_one_word(&a11y, "Delete")` and `what_is_being_done(` and the file
to no `fn delete_the_conversation_row(`; `move_or_copy_message` to the set and the intent
line and `spawn_folder_move` to `what_was_done(`; `start_reply`, `msg_info`,
`save_the_message_as`, the two receipt functions, `answer_the_invitation` and the Copy to arm
to `selected_message_index` and to neither `chosen_messages(` nor `what_was_done(`; the
cursor handler on `on_item_focused` writing the index and no `on_item_selected` on the list;
`refresh_mark_read_wording` to `chosen_rows(`; `take_row_out_of_the_list` and the Delete arm
to `a_set_leaving` and the landing. The built list, `cfg(windows)`, a virtual report list of
five rows without `SingleSel`, a focus handler and a selection handler each counting, from
the run at the green:

| Step | What was done | Focus events | Selection events | The focus handler was given |
|---|---|---|---|---|
| 1 | row 2 selected and focused, as arrowing there does | 1 | 1 | 2 |
| 2 | rows 3 and 4 selected without moving the focus, as growing the range does | 1 | 3 | unchanged |
| the walk | `chosen_rows` over the three | | | `[2, 3, 4]` |
| 3 | the focus moved to row 3, already selected, as Shift+Up does when it shrinks | 2 | 3 | 3 |

So a cursor handler on the selection event runs once per row of a Select All and never for
the shrink, and one on the focus event runs once per cursor move; wx raises the focus event
before the selection event when both change on one row (`src/msw/listctrl.cpp`,
`LVN_ITEMCHANGED`, read 2026-09-19), so a plain arrow key reaches the handler once. The
ignored timing writes 5,000 rows unread through `upsert_messages`, marks each read through
`update_message_flags`, the write `write_flags_or_put_the_row_back` makes, and prints the
row: 75 ms on the release build made in the same command, 77 ms with it already made.
Three records on `wx_app.rs` with `suite` the target.

**Task 3.** `docs/KEYBOARD_SHORTCUTS.md`: under Message Navigation, rows for `Shift+↓`,
`Shift+↑`, `Shift+Home`, `Shift+End` and `Ctrl+A` in the message list, with the control's own
words named as the screen reader's, and a paragraph dating the change, naming the seven
commands and the four that stay on the cursor row; the Select All paragraph's bound sentence
gains that every command over a selection has the bound, with the 75 ms; the Action menu's
rows for Reply, Mark as Read, Star, Delete, Delete Permanently and Move to, the Copy to and
Label submenu rows, the Message Actions rows for Delete, Flag and Mark as Read, and the
Labels section say what each does over a set. `docs/USER_GUIDE.md`: "Selecting more than one
message" under Reading and Managing Email, after "What a delete says", with the keys, the
sentences, the conversation-row rule, the bound and its cost, dated. `docs/changelog.md`: the
entry under Unreleased, Added, for #30 and the thread half of #27 in the tester's words, with
the known limitations. Ledger 544 and 545, both halves, and three deferred items.

## Honest RED and GREEN

Two reds and two greens, then the pages, on branch `every-command-acts-on-the-selection`
from `main` at `15407b1e`.

`01517437`, task 1's red, 153 s through the hook in `red` mode after one refusal by clippy
for an import only the tests used (the stub now names the bound): sixteen cases named by
module path from cargo's own lines, under stubs answering an empty set, this folder only,
the message-row sentence for everything, the one-message line for everything, no question,
nothing above the bound and read for both toggles. Green on arrival and said in the commit:
a delete of message rows asks nothing, and a move or copy reaches this folder only, which the
stub answers for every command. No file a record names gained a test, and the count check
printed no remedy.

`f285b7cd`, task 1's green, 119 s: the module, two records measured, the census line moved.

`9155c6be`, task 2's red, 231 s after one refusal by clippy for a closure called where it
was declared (the built list's reading is a named function now): seven readings named bare
and red for the reasons intended, quoted from the run: the list built with `SingleSel`; the
Star arm, the Delete arm and `move_or_copy_message` not reaching `chosen_messages(`;
`refresh_mark_read_wording` reading the cursor row alone; `take_row_out_of_the_list` not
reaching `a_set_leaving`; and `"msg_list.on_item_focused({"` not here. The built list's walk
red under a `chosen_rows` stub in `wx_app.rs` answering the first selected row alone, `[2]`
against `[2, 3, 4]`. Green on arrival and said: the cursor commands' reading, true already;
the six companions, which read the snippet; and the built list's measurement of the events,
which is the control's and not this tree's, and which decided the green's shape. The target
had no record yet, so the count check was not named.

`7d2c64f4`, task 2's green, 241 s: the style, the handler, the helpers, the six arms, the
worker, the landing, the view switch, the three anchors, three records measured, six
rewritten and measured, the measurements row. `cargo build` ran after every arm and before
the tests; `cargo clippy --all-targets -- -D warnings` refused three `clone` calls on a
`Copy` handle before the commit and passed after.

`44d86fc2`, the pages and the ledger, 128 s: documents only.

Under the TDD gate's own terms, `test(11-07)` precedes `feat(11-07)` twice.

## Guard records

943 by the TOML reader before, 948 after: five new, none retired, six rewritten and measured
again. Census 798 + 145 before, 798 + 150 after. `scripts/guards.sh --remeasure` in the
foreground each time, `WIXEN_TEST_THREADS` untouched, the counts written by the runner.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a message two selected rows name is chosen once, not once per row (new, library) | `choosing_messages.rs` | the duplicate removal dropped | 2, "all 2 tests named went red, and nothing else did": the held-once case and the batch case | rebuild 27 s, run 48 s |
| a command over more than the bound is refused with a sentence, not run (new, library) | `choosing_messages.rs` | `too_many` answers nothing for everything | 1, "the one test named went red, and nothing else did": the bound case | rebuild 29 s, run 48 s |
| the message list selects more than one row, not one however many Shift chose (new, `suite` the target) | `wx_app.rs` | `SingleSel` back | 1: the style reading | rebuild 17 s, run 1 s |
| mark as read acts on every selected message, not the cursor row alone (new, `suite` the target) | `wx_app.rs` | the toggle's set built from the cursor row alone | 1: the flag-commands reading | rebuild 21 s, run 1 s |
| delete over a selection refuses above the bound, not runs whatever the size (new, `suite` the target) | `wx_app.rs` | `too_many` dropped from the Delete arm | 1: the delete reading | rebuild 20 s, run 1 s |
| the move handler asks one place what to say and what to do with the row (rewritten, `suite` wired) | `wx_app.rs` | the same-account move branch wording its own sentence | 1, unchanged | rebuild 12 s, run 2 s |
| one place words what a move or a copy did (rewritten, library) | `wx_app.rs` | the same-account copy branch wording its own sentence | 1, unchanged | rebuild 30 s, run 49 s |
| the reason a command did nothing does not go out as a status line (rewritten, library) | `wx_app.rs` | the move's store refusal as a status line | 1, unchanged | rebuild 44 s, run 48 s |
| landing on a row makes mark as read say which way it will go, not leave the last row's words (rewritten, `suite` the target of 11-06) | `wx_app.rs` | the refresh dropped from the cursor handler | 1, unchanged | rebuild 17 s, run 1 s |
| m on the message list is wired through the helper that consumes it, not bound bare (rewritten, `suite` the target of 11-06) | `wx_app.rs` | a bare `KEY_DOWN` binding | 1, unchanged | rebuild 19 s, run 1 s |
| a delete says the one word delete at the key, not the subject again (rewritten, `suite` the target of 11-06.1) | `wx_app.rs` | the intent line spoken at the key | 1, unchanged | rebuild 20 s, run 1 s |

The first draft of the two library records named their tests bare, the shape of the target
records at the end of the file, and the runner answered "the test harness never ran" them;
named by module path, as every library record is, both agreed with their red lists. Every
other first draft was a prediction the runner agreed with. The companions stay green under
every break because they read a snippet, and each new record's comment says which reading
goes red. Counts written: `choosing_messages.rs` 18 on two records; `wx_app.rs` 199 and the
target 17 on three; `wx_app.rs` 199 and `wired.rs` 77 on one rewritten; `wx_app.rs` 199 on
two; `wx_app.rs` 199 and the 11-06 target 14 on two; `wx_app.rs` 199 and the 11-06.1 target
19 on one. The count check printed no remedy at any commit: the target was new, `tagging.rs`
lost two tests under no record, and `wx_app.rs` is at 199 before and after, quoted by
`cargo test --lib presentation::wx_app::` at task 2's green; 88 records name it now, three
more than the 85 at the start.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `15407b1e` before the
branch. The line numbers had moved since `744d05ef` (`SingleSel` at `1123`, `toggle_read_state`
at `10199`, `conversation_nodes` at `12951`, `delete_the_conversation_row` at `14239`,
`move_or_copy_message` at `19329`, `take_row_out_of_the_list` at `19946`, Select All at
`16841`); the shapes held, but for these:

1. **32 readers of `selected_message_index`, not 27.** 11-05, 11-06, 11-06.1 and 11-06.2
   added five: the read clock's `reading_began` path, the arrival rule, the load arm's
   identity check, the landing's write and the removal's. 28 after, four fewer, the five set
   commands and the conversation delete having stopped reading it; the plan's "the after is
   smaller" holds and the plan's 8 for `chosen_messages(` is 6, the definition and five
   sites, because Delete and Delete Permanently share an arm, the ten label commands share
   `label_the_message` and Move and Copy share `move_or_copy_message`; met in its meaning and
   quoted as it is.
2. **The selection event is the wrong event for the cursor on a list that selects more than
   one.** The plan's reading held `on_item_selected` to writing the index; the built list,
   run under the red before the arms were written, raised three selection events for three
   rows selected with no focus event, and one focus event with no selection event for the
   focus moving onto a row already selected. A handler on the selection event would have
   loaded five thousand bodies and raised five thousand landing signals for one Ctrl+A, and
   would have left the cursor on the row that left the range under Shift+Up, so Reply would
   have replied to it. Decision 1; the target's doc and the handler's comment say why, and
   the two targets and the file's own folder-tree reading that anchored on the old name were
   rewritten in place.
3. **`what_the_selection_holds` needs no `Showing`.** The plan's signature carried one
   beside the members closure; the window builds the closure from the view, which is the
   only place that knows a row's kind, so the parameter was a second answer. Decision 2;
   `reach_for` a free function for the same reason.
4. **The plan's "the same walk the conversation window uses" cannot reach the whole
   conversation.** `conversation_nodes` reads the loaded rows, which are this folder's; the
   reach for Mark as Read, Star and Label is the whole conversation. The cache query the
   conversation row's delete used takes the reach, so the members come from it, decision 3.
5. **The Mark as Read words cannot afford `chosen_messages` on every arrow key.** The plan
   had `refresh_mark_read_wording` read `what_mark_read_does(&chosen_messages(..))`; over
   conversation rows that is a cache query per selected row per keystroke. The words read
   the rows' own flags on screen, the conversation row's unread count as 11-06 read it,
   decision 4; the toggle still decides through `what_mark_read_does`.
6. **`house_style`'s one-place check was green over six stale records.** After the arms
   moved, six records of `wx_app.rs` had lost their `before`; the check exempts a whole file
   when any record's `after` is in the tree and its `before` is not, taken as the file being
   mid-measurement, and 11-06's record on the cursor handler had an `after`, the handler's
   tail without the refresh line, that matched the rewritten tail by coincidence. Found by a
   one-place count over the TOML by hand; the six rewritten and measured, ledger 545, a
   deferred item, and observation 717 in the skill log.
7. **The two label sentences had no caller left.** `tagging::spoken` and `all_removed` were
   read by the arm alone; the set's sentence is `what_was_done`'s. Gone with their two tests,
   under no record.
8. **The plan's "not sent" for a batch with a refusal.** Nothing in a move is sent; the
   sentence says "1 not moved" or "1 not copied", decision 12.

And two things the plan did not name that the tree has, left where they are and said in the
changelog's known limitations: the control keeps a virtual list's selected rows by position,
so a re-read after a delete keeps the cursor on its message by identity (11-06.1) and the
rest of a selection on whichever messages now sit at those positions; and under conversation
view the messages of a deleted conversation leave `s.messages` while the conversation rows
stay until the next read, as before this plan.

## Deviations from plan

**1. [Rule 1 - Bug] The cursor handler is on the focus event**, contradiction 2 and decision
1: the plan's handler would have run once per row of a Select All and missed the shrink.

**2. [Decision] No `Showing` on `what_the_selection_holds`; `reach_for` free**, contradiction
3 and decision 2.

**3. [Decision] A conversation row's members through the cache under the reach**,
contradiction 4 and decision 3.

**4. [Rule 2 - Correctness] The words read the rows on screen**, contradiction 5 and decision
4: a cache query per selected row per arrow key is a cost on the interface thread nobody
asked for.

**5. [Decision] A conversation of one asks nothing; the D-07 question kept for one row**,
decision 5.

**6. [Decision] The one word and the shown line for a conversation row's delete**, decision
6: the question was just spoken and answered, and a spoken intent line after it is the second
sentence #83 was about.

**7. [Decision] `a_set_leaving` and a refusal ending the wait**, decision 7: the plan's
"takes the set's rows" needs the rows remembered, since they leave one server answer at a
time.

**8. [Decision] One worker for a batch, every outcome shown, the failures counted**, decision
8; the plan's "sums the report" had no shape for a batch that all failed, decision 13.

**9. [Decision] The sentence announced once and shown**, decision 9; the labels' second
spoken copy gone.

**10. [Decision] Star and a label go the way any message that needs it says**, decision 10.

**11. [Rule 3 - Blocking] `tagging::spoken` and `all_removed` removed with their tests**,
contradiction 7 and decision 11.

**12. [Decision] "not moved", not "not sent"**, contradiction 8 and decision 12.

**13. [Rule 2 - Correctness] The words refreshed after Select All**, decision 14: the label
would have said Mark as Unread over a set with unread messages until the next cursor move,
and the command would have marked read anyway.

**14. [Rule 3 - Blocking] Six records rewritten onto the new code**, contradiction 6 and
decision 15.

**15. [Decision] `cognitive-accessibility` was not invoked**: no agent or skill of that name
is offered in this session's list. The sentences were written by hand in the shape the plan
gave, what was done, how many, where, and read for plain language; the refusal above the
bound is three short sentences with the count, the most and what to do.

Everything else executed as written. **No scripted edit touched a tracked file: the
exception set for this plan is zero, and it stayed there.** Every tracked file was changed
by Read then Edit or Write; the two new files were written with Write; `cargo fmt` ran before
each Rust commit; `scripts/guards.sh --remeasure` wrote the counts on `guards/guards.toml`.
The only `sed`, `awk`, `grep`, `tr` and `python` in the session read files, logs, the wx
source and the records file, the last a one-place count that read the TOML and wrote
nothing; `git checkout main` was used once, for the merge. Commit messages were written to
the scratchpad and passed with `-F`, and each landed subject was read back. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each.
No em dash in any file this plan wrote, measured by `grep -c` for the byte sequence over
each diff's added lines: zero; none of the six words, measured by `grep -ciE` over the same:
zero. `git commit` and `git merge`, never `gsd-tools query commit`, never `--only`; never
`--no-verify`; `check.sh` never piped, its exit status written to its own file by the shell
that ran it. No AI attribution in any commit, whatever the harness's reminder said.
`Cargo.toml` and `Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's
profile was not read; no binary was started, neither the installed one nor the tree's; the
target built the one window it read and showed its frame for under a second per run; NVDA
was not stopped, reconfigured or driven; every commit was made from the primary checkout,
none from `wixen-mail-sweep` or `wixen-mail-mutants`. `WIXEN_TEST_THREADS` untouched. The
version stays `1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/choosing_messages.rs`, `mod.rs` | `--lib application::choosing_messages` on task 1's red and green, `application` whole on the red for `mod.rs`, and the whole-tree guards |
| `tests/every_command_acts_on_the_selection.rs` | itself on each commit that changed it, and through its records' coupling from `7d2c64f4` on |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the coupled targets its records name, twenty-four at task 2's green, on both commits that changed it |
| `src/application/tagging.rs`, the two targets whose anchors moved | their `--lib` filter and themselves on task 2's green |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every code commit; the document-reading targets on the documents-only commit |

`scripts/check.sh all` ran once on the branch at `44d86fc2`, output to a file with the exit
status written by the same shell: exit 0, 8,187 passed and none failed over 82 result lines,
358 s from 03:48:47Z to 03:54:45Z, the release build included; one more result line than
11-06.3's 81, the new target, and 32 more tests: 18 in the module and 16 in the target, less
the 2 that went from `tagging.rs`. `main`'s hook at the merge, 361 s from 03:55:02Z to
04:01:03Z, the same 8,187 and none failed. The tester's copy and NVDA were open on the
desktop throughout; the target's frame was shown for under a second per run; no live-window
key test went red on any run, so the input was never checked for idleness. The keyring race
(ledger 374) did not appear.

## Threat register

T-11-25 mitigated: the set is read from the control at the key by `chosen_rows` and kept
nowhere, `what_the_selection_holds` drops duplicates by row id with a record measuring the
drop, and the readings hold every set arm to `chosen_messages(`. T-11-26 mitigated:
`too_many` refuses above `MOST_ROWS_WORTH_SELECTING`, read from `editing.rs`, with one
sentence; a record on the library measures the bound gone and one on the target measures it
dropped from the Delete arm; the cost at the bound is the 75 ms row. T-11-27 mitigated:
`reach_for` gives Move and Copy `ThisFolderOnly` whatever the setting, three cases; Delete
follows the setting through `how_far_a_conversation_delete_reaches`. T-11-28 mitigated: the
batch counts the messages that did not go into the one sentence and logs each reason with
`tracing::warn!`; a batch that all failed is spoken as a refusal with the first reason.
T-11-SC: nothing added. New surface outside the register: none; the calls into Windows are
the target's list control on its own thread.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after the edit, 16 passed. 543 before, 545
after; 510 open before, 512 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 544 | unrun-verify | what only the tester's ear settles for #30 and the thread clause of #27: Shift+Down growing the selection with NVDA's "selected" per row and the row read; Shift+Up's "not selected"; the count after Ctrl+A; "Delete" once and the row after the set read once when the set has gone; "3 messages marked read" once; M on a conversation row marking the thread and saying so; the refusal above 5,000 heard once |
| 545 | deviation | `test_every_guard_record_still_names_one_place_in_the_tree`'s exemption is per file: one record whose `after` matched by coincidence hid five others with their `before` gone, and the check passed over six unmeasurable records; found by a count by hand, the six rewritten and measured; the exemption should be per record and a pass through it should say so |

## The issues

`gh issue close 30` and `gh issue close 27` from the repository root after the merge, closed
at 2026-09-19T04:01:40Z and 04:01:42Z, each with the plan's sentence and the merge commit
`b35a40cd`. #30's comment adds why the list moved the selection until now and why the cursor
is the focused row, with the built list's finding; its ear list: Shift+Down grows the
selection and each new row is read with "selected"; Shift+Up gives "not selected" for the row
that leaves; the count after Ctrl+A; Delete over five says "Delete" once and lands after the
set, read once; Mark as Read over a selection says one sentence; Ctrl+A on a folder past 5,000
says the bound. #27's comment names 11-06's `fe143d46` beside the merge, says the label follows
the selection now and what each command's reach over a conversation row is; its ear list, on
top of 11-06's: M on a conversation row marks the whole thread and says "1 conversation, 5
messages marked read"; the item says Mark as Read on a conversation row with unread messages
and Mark as Unread after M; M on one message row still says the one word with the list staying
put. Closing an issue is not a publish; nothing was pushed.

## Known stubs

None. `what_the_selection_holds` has one non-test caller, `chosen_messages`, reached from the
five set sites; `reach_for` one, `how_far_a_conversation_delete_reaches`, reached from the
Delete arm; `what_was_done` four, the Star arm, the toggle, the labels and the batch worker;
`what_is_being_done` two, the Delete arm and the move; `deleting_asks` one, the Delete arm;
`too_many` five; `what_mark_read_does` one, the toggle; `what_star_does` one, the Star arm;
`chosen_rows` five, `chosen_messages`, the context menu closure, `refresh_mark_read_wording`,
`switch_the_view` and the target; `ASetLeaving::of` two, the Delete arm and the move;
`one_left` one, the removal; `a_refusal_ends_the_wait_for_a_set` two, the refusal arms;
`the_chosen_rows_have_unread` two, the refresh and the context menu closure. Every arm is
reached from the menu ids and keys it always was; M from `wire_letter`; the cursor handler
from the control's focus event on every arrow key, click, landing and arrival.

## Not done here, on purpose

Whether NVDA's selection words and the count come through, whether one sentence over many is
enough and whether Delete over five lands once are ledger 544 and the comments on the issues;
the changelog says nobody has heard any of it. A set moved across accounts has met no server,
and what a provider makes of thousands of flag changes queued at once has not been measured;
the measurements page says the row bounds the cache alone. The selection after a re-read, and
the wait a local delete's error leaves, are deferred items for 11-07.1 and 11-08. The
one-place check's exemption is ledger 545 and a deferred item, not fixed here because
`house_style.rs` is named by 27 records. LIST-04 and LIST-05 are ticked on their `[D]` lines,
each covered above, with their `[S]` lines untouched: the phase README says 11-12 ticks the
`LIST` requirements clause by clause, and the orchestrator's instruction for this plan was to
tick them where held, which is the overrule with its reason; 11-12 still reads them. The row is
`12/24`, counted from the disk.

## What 11-07.1 and 11-08 need to know

- **The set is read at the key and kept nowhere.** `chosen_messages(state, cache, list,
  reach)` answers a `Chosen` from the control's selected rows, or the cursor row when none is
  selected; a conversation row's members come from `messages_in_conversation` under the reach
  the command asked for, so a command that wants a different reach changes `reach_for` and
  nothing else. Under conversation view the rows are conversations and every row is read
  through the cache; which message a row stands for (11-08) does not change what the set
  holds, since the row contributes its messages, not one of them.
- **The cursor is the focused row and the handler is on `on_item_focused`.** A plan that lands
  the cursor with `put_the_cursor_on` runs the handler as before; a plan that selects rows
  without moving the focus, as Select All does, runs it not at all, and the words are refreshed
  by hand there. Two targets and the file's folder-tree reading anchor on
  `msg_list.on_item_focused({`.
- **A set leaving the list is `a_set_leaving`.** The Delete arm and a move over more than one
  message under the flat view write it with the rows' ids and indices at the key;
  `take_row_out_of_the_list` lands nothing for a row of the set until the last has left and
  then lands once over the set's rows; `a_refusal_ends_the_wait_for_a_set` on the two refusal
  arms lands after the rows that left and clears it. 11-07.1, which takes the rows out at the
  key before the server is asked, can land the set at once and clear the field in the same
  step, and should: the wait exists only because the rows leave one server answer at a time.
- **`spawn_folder_move` takes the set.** `Vec<AMessageMoving>` with each message's folder, the
  `Chosen` for the sentence, the destination and whether copying; one worker, `owner_of` per
  message under one lock before it starts, each branch answering the outcome and whether the
  message went. For one message it is 11-06.1's shown-or-spoken outcome; for a set every
  outcome is shown, refusals are counted and logged, and one sentence is spoken at the end.
  11-07.1's move that completes here first changes what each branch does and inherits this
  shape and the one word at the key.
- **The sentences are `what_was_done` and `what_is_being_done`.** 11-13's read of every
  status sentence finds them in `choosing_messages`, one function each, and the refusal above
  the bound in `too_many`; the D-07 question is still `view_state::deleting_a_conversation_asks`
  for one conversation row and `deleting_asks` for more.
- **The label words come from the rows on screen, not the set.** `the_chosen_rows_have_unread`
  reads a message row's flag or a conversation row's `unread` count; 11-08's choice of which
  message a conversation row is changes nothing there, since the count is the row's.
- **`tagging::spoken` and `all_removed` are gone.** The label sentences are `what_was_done`'s
  `Labelled`, `Unlabelled` and `LabelsRemoved` outcomes.
- **The one-place check over the records is blind for a file while any record of it looks
  mid-measurement.** After moving code a record's `before` was under, count the records
  whose `before` is not in the tree by hand, ledger 545, until the exemption is per record.

## Self-Check: PASSED

`src/application/choosing_messages.rs` and `tests/every_command_acts_on_the_selection.rs`
exist; `grep -c 'pub mod choosing_messages' src/application/mod.rs` is 1;
`grep -v '^\s*//' src/presentation/wx_app.rs | grep -c 'ListCtrlStyle::SingleSel'` is 0 and
the same for `chosen_messages(` is 6, for `msg_list.on_item_focused({` 3 (the handler, and
the folder-tree reading's anchor and its made-up snippet in the file's own tests, which
`grep -v` does not cut) and for `fn delete_the_conversation_row(` 0; `grep -c 'selected_message_index'
src/presentation/wx_app.rs` is 28; `guards/guards.toml` holds 948 records by the TOML reader
and the census says 798 + 150, and every record's `before` names one place by the count by
hand; `docs/development/measurements.md` holds the 5,000-row row dated 2026-09-19;
`docs/KEYBOARD_SHORTCUTS.md` holds `Shift+Down` twice and `docs/USER_GUIDE.md` "Selecting more
than one message" once; `.planning/WINDOWS.md` holds 544 and 545 in both halves;
`.planning/REQUIREMENTS.md` has LIST-04 and LIST-05 ticked; `gh issue view 30` and `27` answer
CLOSED. Commits `01517437`, `f285b7cd`, `9155c6be`, `7d2c64f4`, `44d86fc2` and `b35a40cd` are in
`git log --oneline --all`. Carriage returns zero and em dashes zero on this file, `STATE.md`,
`ROADMAP.md` and `REQUIREMENTS.md`.

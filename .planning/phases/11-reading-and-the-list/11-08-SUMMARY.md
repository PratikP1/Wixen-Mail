---
phase: 11-reading-and-the-list
plan: 08
subsystem: which message a conversation row is, chosen in SQL where the columns are and read by the cell, the sort, the preview, the conversation window, Space and the commands over the cursor row; the conversation's text fetched on landing; guards, pages
tags: [conversation-row, originator, first-unread, stands-for, correspondent, snippet, preview, thread-view, space, cursor-commands, text-fetch, reading-gate, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-07 merged at b35a40cd (the cursor handler on on_item_focused, chosen_messages over a conversation row's messages, the words from the row's unread count); 11-07.1 merged at fa20d04a and 11-07.2 at 2526b31f (the arms over the set, which this plan leaves alone); 11-06.1's put_the_cursor_on and 11-06.2's list_arrival, which run the cursor handler this plan gives a conversation branch; 10-05's fetch_over_a_mailbox and bringing_everything_down's bound; main clean at dbbbaa2b when the branch left it"
provides:
  - "application::conversations::RowMessage { id, uid, from } and ConversationItem.stands_for, the one message a row stands for by the rule read ASC, received_at ASC, id ASC; 26 tests, unchanged"
  - "presentation::message_columns: the_message_the_row_stands_for! macro spelling that ordering once; Correspondent's conversation expression that message's sender and Snippet's its first line; conversation_stands_for_expression and conversation_stands_for_uid_expression; EVERYONE_WHO_SENT, the aggregate for the rest of the cell; 43 tests, unchanged"
  - "data::message_cache::messages: the here CTE carries m.uid; conversations_query selects the senders aggregate at column 7 and the row message's id, uid and sender at 16, 17 and 18; conversation_row reads them; 179 tests, unchanged"
  - "data::message_cache::bodies::text_missing_in_a_conversation(thread_id, account_id, folder_id, reach, first): the conversation's messages with no text here over the row's own scope, the row message first then arrival order, a message no server holds left out; 50 tests, unchanged"
  - "presentation::message_rows: the Correspondent cell says the row message's sender first and everyone else once, through one_first_then_everyone over each_of; 41 tests, two rewritten in place"
  - "presentation::wx_thread_view::where_to_open(nodes, open_on) and the open_on parameter on show_thread_dialog and build_thread_dialog: the cursor on the named message with the choice starting as it, the root when nothing is named; 9 tests, unchanged"
  - "wx_app.rs: WxUIState::the_row_stands_for(row), what_the_cursor_stands_for() and the_loaded_message_the_row_stands_for(row); the cursor handler asking them, previewing and fetching the row message, signalling a conversation row's landing from its own count and attachments through feedback_events_for_landing_on_a_conversation, and starting spawn_conversation_text_fetch; that fetch as one chunk through what_to_do_next with no folders and no budget, under allowed_for(account).reading, on the account's session through fetch_over_a_mailbox with the download's pause as its stop and conversations_being_fetched keeping one fetch per conversation; the activation handler handing open_conversation_again the row's message; Space's lookup and the read clock, Reply, msg_info, the two receipt functions, the invitation, the save, mark_what_was_read, block_the_sender and the copy-to-item arm reading the loaded message the row stands for; spawn_body_fetch checking the cursor's message by id; the preview arm the same; 199 tests before and after, 95 records"
  - "tests/a_conversation_row_stands_for_one_message.rs: seven cache cases through the real listing, three behaviour cases over the state, the tree's rule and the fetch's list, six readings over what_ships with three companions; 18 tests, 5 records name it"
  - "guards/guards.toml: 965 records, census 798 + 167; five new records measured, one corrected by hand from the runner's answer on the green tree, one of 11-05.1's rewritten onto the new lookup and measured again"
  - "docs/USER_GUIDE.md: Which message a conversation row is, under Thread View; docs/changelog.md: the entry under Unreleased, Fixed, with the preview defect said plainly and the known limitations"
  - ".planning/WINDOWS.md: 548 unrun-verify, what only the tester's ear and account settle"
affects: [11-08.1, which re-threads by Gmail's id and reads the row message per thread_id group; 11-09, which reads a row's cells on request and finds the Correspondent cell's first name the row message's; 11-10, which prefixes a rule's phrase to the first visible cell; 11-13, which reads no new sentence, since this plan says nothing per message; 11-12, which reads LIST-06's lines and the pages; whoever hears a thread row read]

actuals:
  tokens: 29276
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A rule about which member of a group a row stands for is spelled once in SQL as an ordering, read by every expression that answers about that member, and carried on the row as an id, a number and a name, so the cell, the sort, the preview, the window and the fetch cannot come to disagree"
    - "Every reader of the cursor that wants one message asks the state which message the row stands for, and the state branches on the view once; a reader that indexes the flat list with the row's index is the defect, and a reading over the shipped text holds each named reader to the question"
    - "A fetch a person's action starts reuses the runner's own bound with no folders to offer and no budget, so a chunk asked for by hand is shaped like a chunk the download takes and differs from it only in why it runs"

key-files:
  created:
    - tests/a_conversation_row_stands_for_one_message.rs
  modified:
    - src/application/conversations.rs
    - src/data/message_cache/bodies.rs
    - src/data/message_cache/messages.rs
    - src/presentation/message_columns.rs
    - src/presentation/message_rows.rs
    - src/presentation/view_state.rs
    - src/presentation/virtual_rows.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_thread_view.rs
    - tests/theme_reach.rs
    - tests/tree_dialogs_resolve_the_row_somebody_is_on.rs
    - tests/tree_rows_leave_no_registry_entry.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The ordering is one macro, the_message_the_row_stands_for!, taking the column name, so the four expressions that answer about the row message (sender, snippet, id, number) spell the rule once and the string stays a literal chosen by matching on the enum; the plan's four hand-written subqueries would have been four spellings"
  - "The senders aggregate stays a column of the listing under its own name, EVERYONE_WHO_SENT, rather than a column expression of the enum: Correspondent's expression is the row message's sender, which is what the column sorts by, and the rest of the cell is read from the aggregate beside it, so the shown value begins with the sort value as D-02 asks"
  - "The state answers which message a row stands for, in one place, and every reader of the cursor that wants one message asks it: the plan named the selection handler, and the same index reached Space, Reply, the read receipt, the invitation, the save, the read clock, the block and the copy-to-item arm, each of which acted on an unrelated message under conversation view; converting the readers is one call each and a reading holds the eight named functions, with selected_message_index kept in each body so 11-07's reading of the cursor commands still holds"
  - "The conversation's missing text is listed by the cache over the row's own scope, text_missing_in_a_conversation in bodies.rs beside messages_with_no_text_here, rather than from conversation_nodes: the loaded rows are this folder's and the tester's ask is the whole conversation; the row message is put first by the query and left to the body fetch already running for it, which reaches the preview, so nothing is asked for twice"
  - "The chunk's bound is the runner's own what_to_do_next with no folders, TextBudget::All and kept_bytes nought, because a person asked and the download's budget is where the download stops; the reading gate is asked once before the worker starts and again by the bound"
  - "A set of conversations being fetched lives on the state, keyed by conversation id, so a row arrowed back onto while its fetch runs is not asked for again; the plan named the bound and the gate and not this, and it is the cheap half of T-11-30"
  - "The conversation tree takes open_on: Option<i64> on both builders, the pure rule where_to_open beside them, and the choice starts as the named message so Enter on arrival opens it; the scan target and the three integration targets pass None and open on the root as before"
  - "Enter under the flat view hands the tree the row's own message too, so the tree opens with the cursor on the message somebody pressed Enter on rather than on the root; the plan asked for conversation view and the same line serves both"
  - "A conversation row's landing events come from the row's own count and attachments, feedback_events_for_landing_on_a_conversation, rather than from the flat row at its index, which is what the handler signalled until now"

patterns-established:
  - "A guard measured while the tree is deliberately red cannot see a red the break would add among the tests already red under a stub; the runner says so and the measurement is re-taken at the green as a step, not a courtesy; observation 722 in the skill log"

requirements-completed: [LIST-06]

coverage:
  - id: D1
    description: "The row message chosen where the columns are: the first unread by arrival when any is unread, the originator otherwise, held through the real listing under four read patterns and for a conversation of one; the cell saying that sender first and everyone once; the sort by Correspondent ordering by that sender both ways"
    requirement: LIST-06
    verification:
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_with_nothing_read_the_row_stands_for_the_originator"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_with_later_messages_read_the_row_still_stands_for_the_originator"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_with_the_first_messages_read_the_row_stands_for_the_first_unread"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_with_everything_read_the_row_stands_for_the_originator_again"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_a_conversation_of_one_stands_for_its_one_message"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_the_correspondent_cell_says_the_row_messages_sender_first_and_everyone_once"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_sorting_by_correspondent_orders_conversations_by_their_row_messages_senders"
        status: pass
      - kind: unit
        ref: "src/presentation/message_rows.rs#test_correspondent_lists_the_distinct_senders_by_name"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a conversation row stands for the first unread or the originator, not the newest' at 7 red and 'the correspondent cell says the row message's sender first, not the senders in stored order' at 1 red, measured 2026-09-19 on the target"
        status: pass
      - kind: command
        ref: "cargo test --lib data::message_cache::messages:: -> 179 passed; --lib presentation::message_columns:: -> 43; --lib application::conversations:: -> 26; --lib presentation::message_rows:: -> 41; --test all_inboxes_reads_in_the_sort_that_was_chosen -> 7; grep -c 'fn conversation_stands_for_expression' src/presentation/message_columns.rs -> 1; grep -c 'pub stands_for: RowMessage' src/application/conversations.rs -> 1"
        status: pass
    human_judgment: false
  - id: D2
    description: "The window: the state's answer to which message a row stands for under both views and when the loaded rows do not hold it; the tree's rule for where the cursor starts; the cache's list of a conversation's missing text with the row message first; the cursor handler previewing the row message and starting the fetch; the fetch as one bounded chunk under the gate; Enter handing the tree the row message and the tree selecting it before the root; Space reading it; the eight commands over the cursor row and the body fetch asking the same question"
    requirement: LIST-06
    verification:
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_the_cursor_stands_for_the_rows_message_under_conversation_view_and_for_itself_under_the_flat"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_the_tree_opens_on_the_named_message_and_on_the_root_when_none_is_named"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_selecting_a_row_asks_for_the_conversations_missing_text_with_the_row_message_first"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_the_cursor_handler_previews_the_row_message_and_starts_the_fetch"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_the_conversations_text_is_one_bounded_chunk_under_the_reading_gate"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_enter_opens_the_conversation_on_the_row_message"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_space_reads_the_row_message"
        status: pass
      - kind: integration
        ref: "tests/a_conversation_row_stands_for_one_message.rs#test_every_command_over_the_cursor_row_acts_on_the_row_message"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the cursor handler previews the message the row stands for, not the flat row at its index', 'a conversation's text is fetched under the reading gate, not whatever the box says' and 'the conversation tree opens on the message the row stood for, not the root whatever was asked', one red each, measured 2026-09-19 on the target"
        status: pass
      - kind: command
        ref: "cargo test --lib presentation::wx_app:: -> 199 passed; --lib presentation::wx_thread_view:: -> 9; --lib data::message_cache::bodies:: -> 50; --test wired -> 77; --test every_command_acts_on_the_selection -> 16 and one ignored; grep -v '^\\s*//' src/presentation/wx_app.rs | grep -c 'fn spawn_conversation_text_fetch' -> 1; grep -n 'select_item(' src/presentation/wx_thread_view.rs -> 350 the chosen item, 351 the root"
        status: pass
    human_judgment: false
  - id: D3
    description: "The guide, the changelog and the ledger"
    requirement: LIST-06
    verification:
      - kind: command
        ref: "grep -c 'Which message a conversation row is' docs/USER_GUIDE.md -> 1; grep -c '#31' docs/changelog.md -> 1; ledger 548 in both halves; cargo test --test house_style -> 74; --test docs_links -> 6; --test the_planning_files_agree_with_themselves -> 16; --test the_words_that_say_nothing -> 9"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the sender is heard first on a thread row in the tester's inbox, whether the preview under it is that message, whether Enter opens the conversation on it and Space reads it, and whether the text of a conversation arrives from Gmail on landing"
    requirement: LIST-06
    verification: []
    human_judgment: true
    rationale: "Ledger 548; the close comment on #31 lists it; nothing here has been heard, and no server has been asked for a conversation's text this way"

duration: 93min
completed: 2026-09-19
status: complete
---

# Phase 11 Plan 08: A conversation row stands for one message Summary

**A conversation row now stands for one message, and everything that reads the row agrees
which: the message that started the conversation when nothing in it has been read, otherwise
the first unread message by arrival, and the originator again when everything is read, the
tester's rule of 2026-09-16 (#31). It is chosen in SQL where the row's columns are, by one
ordering, `read ASC, received_at ASC, id ASC`, spelled once and read by four expressions, and
it rides the row as its id, its number and its sender. The Correspondent cell says that sender
first and the other senders after it, each once; the Snippet cell is its first line; sorting
by Correspondent orders conversations by it; the preview shows it; Space reads it; Enter opens
the conversation window with the cursor on it, so Enter again opens that message; Reply, the
read receipt, the invitation, the save, the read clock and the block act on it. Landing on the
row also brings the text of the rest of the conversation as one bounded chunk in the
background, under the account's Message Text box, with nothing said. On the way the defect the
issue did not name and phase 11's decision 14 owns is fixed: under conversation view the
preview showed the message sitting at the row's index of the flat list, an unrelated message,
because the cursor handler had no conversation branch, and Space, Reply and the receipt read
that message too. Nobody has heard a thread row read this way.**

## Performance

- **Duration:** 93 min from the start recorded at 09:15:47Z, when the reading of the plan, the
  README, the fourteen summaries and the tree began, to the merge's hook finishing at
  10:48:26Z; the branch's first commit at 09:36:08Z; the summary and the planning files after.
  About 4 min 30 s was guard measurement in five foreground runs (40 s for the two target
  records at task 1, 7 s to read what already fails then 14 s and 19 s; 36 s the remedy at
  task 2's red for the same two, against a red tree; 109 s for the five on the green tree, 18
  s, 19 s, 18 s, 20 s and 17 s, the ordering record coming out short; 33 s for that record
  corrected; 17 s for 11-05.1's rewritten record); about 24 min the seven hook runs on the
  branch (161 s, 150 s, 287 s refused, 285 s, 264 s refused, 274 s, 137 s); 379 s the whole
  gate on the branch; 399 s `main`'s hook at the merge; 0 s waiting for the desktop, since no
  live-window test went red on any run.
- **Started:** 2026-09-19T09:15:47Z
- **Merged:** 2026-09-19T10:48:26Z at `75c211fe`
- **Tasks:** 3
- **Files modified:** 17, one created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat dbbbaa2b..e38bf31d -- Cargo.toml Cargo.lock` (T-11-SC)
- **Actuals:** `tokens: 29276` is `git diff dbbbaa2b..e38bf31d | wc -c`, 117,106 characters
  over four, the branch's own diff against the commit it left `main` at; the estimate's
  `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.75 against the former and
  0.33 against the latter.

## What landed

**Task 1.** `application::conversations::RowMessage { id, uid, from }` and
`ConversationItem::stands_for`, with the rule in the struct's doc. In `message_columns.rs` the
macro `the_message_the_row_stands_for!("column")`, a correlated subquery over `here` for the
conversation's rows ordered `r.read ASC, r.received_at ASC, r.id ASC LIMIT 1`, spelled once and
taken four times: Correspondent's conversation expression is `from_addr`, Snippet's is
`snippet`, `conversation_stands_for_expression()` is `id` and
`conversation_stands_for_uid_expression()` is `uid`. `EVERYONE_WHO_SENT` is the aggregate the
Correspondent column used to be, one sender per line, kept for the rest of the cell. In
`messages.rs` the `here` CTE carries `m.uid`, `conversations_query` selects the aggregate at
column 7 where the reader always read the senders and the row message's id, number and sender
at 16, 17 and 18, and `conversation_row` fills `stands_for` from them, the id and the number
read as the numbers they are so a drifted query refuses the listing rather than previewing row
nought. In `message_rows.rs` the Correspondent arm is `one_first_then_everyone(stands_for.from,
senders)`: the row message's sender first, compared as stored, then everyone in the aggregate
who is not that sender, each through `display_address`; `everyone_in` for To and Cc reads
through the same `each_of`. The three fixture literals in `message_rows.rs`, `view_state.rs` and
`virtual_rows.rs` gained the field; the two cell tests in `message_rows.rs` were rewritten in
place, the first wanting Bob first because the fixture's row message is Bob's, the second
making the comma-holding sender the row message so it still says what it said. 179, 43, 26 and
41 tests before and after, quoted by each filter at the green.

`tests/a_conversation_row_stands_for_one_message.rs`, the cache half: a conversation of four,
A the originator and then B, C and D by arrival with D's sender A's, each message's text saved
so its snippet is its first line, read through `conversations_in` under
`AConversationReaches::TheWholeAccount`. Everything unread stands for A; B and C read stands
for A still; A and B read stands for C; everything read stands for A again; each case reads
the row's id, number, sender and snippet. A conversation of one stands for its one message.
The Correspondent cell over the C case says "Chris, Ada Lovelace, Bob" and the Snippet cell
"One question about page four". Two conversations, Mel then Ada unread and Bea read then Zed
unread, sort by Correspondent ascending as Mel then Zed and descending the other way, through
the clause `view_state::order_by(Showing::Conversations, ..)` builds, where the senders as a
whole would have put Bea's conversation first. Two records on the target.

**Task 2.** `WxUIState` gains three answers: `the_row_stands_for(row)`, the row itself under
the flat view as a `RowMessage` and the listing's choice under conversation view;
`what_the_cursor_stands_for()`, the same for the cursor's row; and
`the_loaded_message_the_row_stands_for(row)`, that message as the loaded rows hold it, or
nothing when they do not, which under conversation view is a row message filed in another
folder of the account. The cursor handler asks the first, previews the row message's body from
the cache by its id, starts `spawn_body_fetch` for it when the body is not here, signals a
conversation row's landing from the conversation's own count and attachments through
`feedback_events_for_landing_on_a_conversation`, and under conversation view hands the
conversation to `spawn_conversation_text_fetch`. That worker reads the open folder and its
account, asks `allowed_for(account).reading` before it starts, keeps one fetch per
conversation through `conversations_being_fetched` on the state, lists the conversation's
missing text through `text_missing_in_a_conversation`, takes one chunk through
`what_to_do_next` with no folders, `TextBudget::All` and no bytes kept, opens the account's
session and hands the chunk to `fetch_over_a_mailbox` with the download's pause as the stop and
an after-each that says nothing; a failure is logged with the account and the count. In
`bodies.rs`, `text_missing_in_a_conversation` is the conversation scope, the rows of `here` in
the conversation joined back to their folder's path and left-joined to `message_bodies`, those
with no body and somewhere to ask, the row message first by a `CASE` and the rest by arrival.
The activation handler asks `the_row_stands_for` under both views and hands
`open_conversation_again` an `open_on`, which `show_thread_dialog` takes; in
`wx_thread_view.rs`, `where_to_open(nodes, open_on)` is the position of the named message,
`build_thread_dialog` puts the cursor on it with the choice starting as `Message(id)` and on
the root when nothing is named, and the scan target and the three integration targets pass
`None`. Space's lookup and the read clock's closure, the copy-to-item arm, `mark_what_was_read`,
`answer_the_invitation`, `save_the_message_as`, `start_reply`, `msg_info`, the preview arm,
`receipt_for_the_open_message`, `send_receipt_for_the_open_message` and `block_the_sender` read
the loaded message the row stands for; `spawn_body_fetch` checks the cursor's message by id, so
a body fetched for a row message filed in another folder still reaches the preview. 199 tests
before and after; `wx_thread_view.rs` at 9; `bodies.rs` at 50.

The target's window half: three behaviour cases, the state under both views and with the row
message not among the loaded rows, `where_to_open` for a named message, nothing and a message
the conversation does not hold, and the fetch's list with the row message's text here and
missing and with everything here; six readings over `what_ships`, the cursor handler between
`msg_list.on_item_focused({` and `msg_list.on_column_click({` asking `the_row_stands_for(`,
branching on `showing_conversations()`, reaching `spawn_conversation_text_fetch` and never
`messages.get(`; `fn spawn_conversation_text_fetch(` reaching `.reading`,
`text_missing_in_a_conversation(`, `what_to_do_next(` and `fetch_over_a_mailbox(`; the activation
handler asking `the_row_stands_for(` and `open_conversation_again` handing `open_on` to
`show_thread_dialog(`; `build_thread_dialog` asking `where_to_open(nodes, open_on)` before
`select_item(&root)`; the mail `wire_read_aloud(` segment asking
`the_loaded_message_the_row_stands_for(` and never `messages.get(`; and the eight cursor
commands the same, with `fn spawn_body_fetch(` asking `what_the_cursor_stands_for()`. Three
companions plant nine faults into a snippet shaped as the window should be. Three records on
`wx_app.rs` and `wx_thread_view.rs` with `suite` the target.

**Task 3.** `docs/USER_GUIDE.md`: "Which message a conversation row is" under Thread View,
after the paragraph on reading one conversation, with the rule, what each surface does with
the message, the fetch and its bound, and what the row said before, dated. `docs/changelog.md`:
the entry under Unreleased, Fixed, in the tester's words for #31, the rule, the surfaces, the
fetch and its bounds, the preview defect said plainly, and the known limitations. Ledger 548,
both halves.

## Honest RED and GREEN

Two reds and two greens, then the pages, on branch `a-conversation-row-stands-for-one-message`
from `main` at `dbbbaa2b`.

`8c5d17f4`, task 1's red, 161 s through the hook in `red` mode: the target's seven cases named
bare and the rewritten cell test by module path, under one stub, the ordering the four
expressions share picking the newest by arrival, which is the tree's claim, with Correspondent
still the aggregate; the field, the struct, the three columns and the reader real. The runner
quoted each red: "the row stands for row 4 and not 1; the row's number is 4 and not 1; the
row's sender is the aggregate; the row's snippet is Answered inline". No file a record names
gained a test, and the count check printed no remedy.

`f13ccfb2`, task 1's green, 150 s: the ordering, the Correspondent expression, the cell, the
second cell test rewritten in place, two records measured.

`14120be7`, task 2's red, 285 s after a first attempt of 287 s was refused: the eight readings
and behaviour cases named bare, under three stubs, one per module, the state answering nothing
under conversation view, `where_to_open` answering the root, the cache's list empty, with the
window's new signature in place and the root passed so the tree's three targets build; the
three companions green on arrival and said. The first attempt named no count check and the
gate refused it: the target had gained eleven tests and task 1's two records name it at seven,
so the count check was named in the trailer, as `CLAUDE.md` says a red adding a test to a
named file is committed, and its remedy ran before the green.

`dfb2b9f6`, task 2's green, 274 s after a first attempt of 264 s was refused by the one-place
check: 11-05.1's record on the read-aloud lookup had lost its `before` when the lookup started
asking which message the row stands for; rewritten onto the new text and measured again, one
red as before, and a count by hand over the records file found no other. Three records
measured, the two of task 1 measured again on the green tree, one corrected.

`e38bf31d`, the pages and the ledger, 137 s: documents only.

Under the TDD gate's own terms, `test(11-08)` precedes `feat(11-08)` twice.

## Guard records

960 by the TOML reader before, 965 after: five new, none retired, one rewritten and measured
again. Census 798 + 162 before, 798 + 167 after. `scripts/guards.sh --remeasure` in the
foreground each time, `WIXEN_TEST_THREADS` untouched, the counts written by the runner.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a conversation row stands for the first unread or the originator, not the newest (new, `suite` the target) | `message_columns.rs` | the ordering put back to `received_at DESC, id DESC` | 7, "all 7 tests named went red, and nothing else did": the four read patterns, the cell, the sort and the fetch list's case | rebuild 12 s, run 2 s at task 1; 17 s and 1 s on the green tree after the correction |
| the correspondent cell says the row message's sender first, not the senders in stored order (new, `suite` the target) | `message_rows.rs` | `everyone_in(&conversation.senders)` back | 1: the cell case | rebuild 18 s, run 1 s; 19 s and 1 s on the green tree |
| the cursor handler previews the message the row stands for, not the flat row at its index (new, `suite` the target) | `wx_app.rs` | `s.messages.get(idx)` turned into a `RowMessage` in the handler | 1: the handler reading | rebuild 16 s, run 1 s |
| a conversation's text is fetched under the reading gate, not whatever the box says (new, `suite` the target) | `wx_app.rs` | `let reading = true;` | 1: the fetch reading | rebuild 17 s, run 1 s |
| the conversation tree opens on the message the row stood for, not the root whatever was asked (new, `suite` the target) | `wx_thread_view.rs` | `let opening: Option<TreeItemId> = None;` | 1: the tree reading | rebuild 18 s, run 1 s |
| the first space, the short form, starts no clock towards marking the message read (11-05.1's, rewritten, `suite` its target) | `wx_app.rs` | the write moved back into the lookup closure, now over `the_loaded_message_the_row_stands_for` | 1, unchanged | rebuild 15 s, run 1 s |

The ordering record's first draft named six. At task 2's red its remedy was run because the
count check named it, and the runner agreed with six while warning that a measurement against a
tree that is not green is weaker; on the green tree the same break reddened seven, the seventh
being the fetch list's case, which asserts which message the row stands for before it asks what
is missing and was red under its own stub at the red. Corrected by hand and measured again.
Every other first draft was a prediction the runner agreed with. Counts written: the target 7 on
two records at task 1 and 18 on all five after; `message_columns.rs` 43, `message_rows.rs` 41,
`wx_app.rs` 199, `wx_thread_view.rs` 9. `wx_app.rs` is at 199 before and after, quoted by
`cargo test --lib presentation::wx_app::` at task 2's red and green and at the gate, and holds
196 `#[test]` attributes, which is what the count check counts, before and after; 95 records
name it now, two more than the 93 at the start. `messages.rs` is named by 24 records, one more
than the plan's 23, and gained no test.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `dbbbaa2b` before the
branch. The line numbers had moved since `744d05ef` (the cursor handler at `3120`, the
activation handler at `3266`, `conversation_nodes` at `13366`, `open_conversation_again` at
`22515`, `spawn_body_fetch` at `23270`, `fetch_over_a_mailbox` at `mail_sync.rs:2069`); the
shapes held, but for these:

1. **The same flat index reached far more than the preview.** The plan's premise 2 named the
   selection handler; the cursor handler's index also fed Space's lookup and the read clock,
   the copy-to-item arm, the invitation, the save, Reply, `msg_info`, the preview arm, both
   receipt functions, the block and `spawn_body_fetch`'s check that the body is still wanted,
   twelve sites, each `selected_message_index.and_then(|i| s.messages.get(i))`. Under
   conversation view a read receipt could be sent for a message the person never chose. All
   twelve ask the state now, decision 3, and 11-07's reading of the cursor commands, which
   wants `selected_message_index` in each body, still holds because each keeps it.
2. **`messages.rs` is named by 24 records, not the plan's 23**, by the TOML reader on the day;
   the plan's count was a day old. No test was added there either way.
3. **The plan's four hand-written subqueries are one macro.** Four spellings of one ordering is
   the drift D-02 exists to stop; `the_message_the_row_stands_for!` takes the column name and
   `concat!` keeps the result a literal, decision 1.
4. **The senders aggregate cannot be the Correspondent expression and the row message's sender
   at once.** The plan's "the senders column stays as the aggregate" needed a name for it
   outside the enum; `EVERYONE_WHO_SENT`, decision 2, selected at column 7 where the reader
   always read the senders, so `test_every_value_in_the_conversation_query_is_bound` holds with
   every column's expression still in the query.
5. **`conversation_nodes` reads this folder's rows, and the tester asked for the whole
   conversation.** The plan's "conversation_nodes filtered through the cache's body check" would
   have fetched this folder's messages only; the cache lists the conversation's missing text over
   the row's own scope instead, decision 4, and the row message is left to the body fetch
   already running for it rather than fetched twice.
6. **A guard measured at the red is short by the tests already red.** Contradiction of the
   plan's "each measured" as one step: the ordering record measured six at the red and seven at
   the green, observation 722.
7. **The second cell test in `message_rows.rs` reads the fixture's row message too.** The plan
   named one test to rewrite; `test_a_sender_whose_name_holds_a_comma_is_still_one_person` uses
   `..conversation()` and so inherited Bob first; rewritten in place at the green with the
   comma-holding sender as the row message, so it still tests the comma.
8. **11-05.1's record on the read-aloud lookup names the lookup's text.** Found by the one-place
   check refusing task 2's first green; rewritten onto the new lookup and measured again.

## Deviations from plan

**1. [Rule 2 - Correctness] Every reader of the cursor asks which message the row stands
for**, contradiction 1 and decision 3: a receipt sent or a reply started for an unrelated
message under conversation view is the defect the plan fixes for the preview, at eleven more
sites.

**2. [Decision] One macro for the ordering; the aggregate named beside the enum**,
contradictions 3 and 4, decisions 1 and 2.

**3. [Decision] The conversation's missing text listed by the cache over the row's scope, in
`bodies.rs`**, contradiction 5 and decision 4; `bodies.rs` was not in the plan's files and
gained a method and no test, the target holding the query.

**4. [Rule 2 - Correctness] One fetch per conversation at a time**, decision 6, T-11-30's cheap
half; a field on the state the plan did not name.

**5. [Decision] `open_on` on both builders, the choice starting as the named message, the flat
view handing the tree the row's own message too**, decisions 7 and 8; `where_to_open` public so
the target holds the rule without a window.

**6. [Decision] A conversation row's landing events from its own count and attachments**,
decision 9.

**7. [Rule 3 - Blocking] 11-05.1's record rewritten onto the new lookup**, contradiction 8.

**8. [Rule 1 - Bug] The ordering record corrected from six to seven**, contradiction 6, on the
runner's answer at the green.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write; the new target was written with Write; `cargo fmt` ran before each Rust commit;
`scripts/guards.sh --remeasure` wrote the counts on `guards/guards.toml`. The only `sed`, `awk`,
`grep`, `tr` and `python` in the session read files, logs and the records file, the last
counting records through the TOML reader and checking each new record's `before` is in its file,
writing nothing; one heredoc appended to a commit message in the scratchpad, which is not
tracked. Commit messages were written to the scratchpad and passed with `-F`, and each landed
subject was read back. Carriage returns measured with `tr -cd '\r' | wc -c` on every changed
file before each commit: zero on each. No em dash in any file this plan wrote, measured by
`grep -c` for the byte sequence over each diff's added lines: zero; none of the six words,
measured by `grep -ciE` over the same: zero. `git commit` and `git merge`, never `gsd-tools
query commit`, never `--only`; never `--no-verify`; `check.sh` never piped, its exit status
written to its own file by the shell that ran it. No AI attribution in any commit, whatever
the harness's reminder said. `Cargo.toml` and `Cargo.lock` untouched; no crate or feature
added (T-11-SC). The tester's profile was not read; no binary was started, neither the
installed one nor the tree's; NVDA was not stopped, reconfigured or driven; every commit was
made from the primary checkout. `WIXEN_TEST_THREADS` untouched. The version stays
`1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/conversations.rs`, `message_columns.rs`, `message_rows.rs`, `view_state.rs`, `virtual_rows.rs`, `messages.rs` | their `--lib` filters on task 1's red and green, with `all_inboxes_reads_in_the_sort_that_was_chosen` and `the_list_reads_only_memory` coupled through the records, and the whole-tree guards |
| `src/presentation/wx_app.rs`, `wx_thread_view.rs`, `bodies.rs` | their `--lib` filters and the coupled targets, twenty-six on task 2's red and green, on both commits that changed them |
| `tests/a_conversation_row_stands_for_one_message.rs` | itself on each commit that changed it, and through its records' coupling from `f13ccfb2` on |
| `tests/theme_reach.rs`, `tree_dialogs_resolve_the_row_somebody_is_on.rs`, `tree_rows_leave_no_registry_entry.rs` | themselves on task 2's red |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every code commit; the document-reading targets on the documents-only commit |

`scripts/check.sh all` ran once on the branch at `e38bf31d`, output to a file with the exit
status written by the same shell: exit 0, 8,293 passed and none failed over 85 result lines,
379 s from 10:34:57Z to 10:41:16Z, the release build included; one more result line than
11-07.2's 84, the new target, and 18 more tests, all in it. `main`'s hook at the merge, 399 s
from 10:41:47Z to 10:48:26Z, the same 8,293 and none failed. The tester's copy and NVDA were
open on the desktop throughout; the three tree targets built their window for under a second
per run; no live-window test went red. The keyring race (ledger 374) did not appear.

## Threat register

T-11-29 mitigated: the handler asks `the_row_stands_for(idx)` and never the flat list, a record
measures the break, and the same question reaches the eleven other readers of the cursor.
T-11-30 mitigated: one chunk per selection through the runner's own bound, skipped when nothing
is missing and while the conversation's fetch is running, the stop reading the download's
pause. T-11-31 mitigated: `allowed_for(account).reading` asked before the worker starts and
again by the bound, a record measuring the gate dropped. T-11-32 mitigated: the warn lines carry
the account's name and the count and no subject; 11-04's guard over every `tracing::` call reads
them. T-11-SC: nothing added. New surface outside the register: `text_missing_in_a_conversation`
joins `message_bodies` to the conversation scope, which the listing guard does not read because
it is no listing; the fetch runs on the account's held session as the body fetch does.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after the edit, 16 passed. 547 before, 548
after; 514 open before, 515 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 548 | unrun-verify | what only the tester's ear and account settle for #31: the sender heard first on a thread row, the originator with nothing read and the first unread otherwise; the preview under it that message; Enter opening the conversation window on it and Enter again opening it; Space reading it; M marking the thread; everything read reading the oldest; and the text of a conversation arriving from Gmail on landing, which no server has been asked for |

## The issue

`gh issue close 31 --reason completed --comment` from the repository root after the merge, at
2026-09-19T10:48:58Z, with the merge commit `75c211fe`: the plan's sentence, the preview defect
found on the way with Space, Reply and the receipt beside it, the rule chosen once in SQL, and
the ear list: a thread row with every message unread reads the originator; with some read, the
first unread's sender; the preview under the row shows that message; Enter opens the
conversation on it and Enter again opens it; Space reads it; M marks the whole thread; a thread
with everything read reads the oldest; landing on a row brings the rest of the conversation's
text from Gmail with nothing said. Closing an issue is not a publish; nothing was pushed. #31 is
CLOSED.

## Known stubs

None. `RowMessage` is filled by `conversation_row` and read by `the_row_stands_for`, the cell
and the target; `the_message_the_row_stands_for!` has four readers; `EVERYONE_WHO_SENT` one, the
query; `text_missing_in_a_conversation` one non-test caller, the fetch; `where_to_open` one,
`build_thread_dialog`; `the_row_stands_for` four, the handler, the activation handler,
`what_the_cursor_stands_for` and the loaded read; `what_the_cursor_stands_for` one,
`spawn_body_fetch`;
`the_loaded_message_the_row_stands_for` thirteen, the handler's flat branch, Space's two
closures and the ten commands; `spawn_conversation_text_fetch` one, the handler;
`feedback_events_for_landing_on_a_conversation` one, the handler; `conversations_being_fetched`
written and cleared by the fetch alone. Every path is reached from the focus event, Enter, Space
and the menu ids it always was.

## Not done here, on purpose

Whether any of it is heard is ledger 548 and the comment on #31; the changelog says nobody has
heard it and no server has been asked for a conversation's text this way. A row message filed in
another folder of the account is previewed by id but not among the loaded rows, so the
conversation window opens on its first row and Space and Reply act on nothing until that folder
is opened; said in the changelog's known limitations and not fixed here, since `conversation_nodes`
and the commands read the rows on screen and reading the cache from every command is a plan of
its own. The dates stay the newest, as the plan decided. The one-place check's per-file
exemption (ledger 545) is unchanged and bit once here, caught by the check itself on the first
green. LIST-06 is ticked on its `[D]` lines with the orchestrator's instruction as the overrule of
the README's "11-12 ticks"; its `[S]` lines are untouched. The row is `15/27`, counted from the
disk.

## What 11-08.1 needs to know

- **The row message is chosen per `thread_id` group, in the query, at read time.** Nothing is
  stored: `the_message_the_row_stands_for!` runs over `here` for the conversation's rows every
  time `conversations_in` runs, so a re-threading that moves rows between groups changes which
  message a row stands for at the next listing and nothing else has to be told. A row whose
  messages Gmail's id joins to another conversation is read under that conversation's group,
  and the first unread or the originator of the joined group is the row's.
- **The count the words read and the row message are two answers from one group.** 11-06's
  `unread` count and this plan's `stands_for` both come from `conversations_query` over the same
  `here`, so a re-threading cannot leave the row saying one conversation's count and previewing
  another's message.
- **`text_missing_in_a_conversation` is over the same scope**, with `?4` the conversation id, so
  a fetch started before a re-threading and finishing after it fetched the old group's messages,
  which are still messages somebody was looking at; the next landing asks the new group.
- **The fetch keys `conversations_being_fetched` by `thread_id`.** A re-threading that renames a
  group leaves a stale key until that fetch ends, harmless, since the key only stops a second
  fetch of the same name.
- **The target's fixture files replies under one root through `refs_header`**, the way the
  cache files a chain; a Gmail id arriving through `IncomingMessage` would need the fixture to
  carry it, and `stands_for` reads whichever `thread_id` the store gave the rows.
- **Enter hands the tree the row's message under both views.** A re-threading changes the nodes
  `conversation_nodes` builds from the loaded rows, and `where_to_open` answers the root when the
  named message is not among them.

## Self-Check: PASSED

`tests/a_conversation_row_stands_for_one_message.rs` exists; `grep -c 'fn
conversation_stands_for_expression' src/presentation/message_columns.rs` is 1; `grep -c 'pub
stands_for: RowMessage' src/application/conversations.rs` is 1; `grep -v '^\s*//'
src/presentation/wx_app.rs | grep -c 'fn spawn_conversation_text_fetch'` is 1; `grep -n
'select_item(' src/presentation/wx_thread_view.rs` answers 350 for the chosen item and 351 for
the root; `grep -c 'Which message a conversation row is' docs/USER_GUIDE.md` is 1; `grep -c
'#31' docs/changelog.md` is 1; `guards/guards.toml` holds 965 records by the TOML reader and the
census says 798 + 167, and every record's `before` is in its file by the count by hand;
`.planning/WINDOWS.md` holds 548 in both halves; `.planning/REQUIREMENTS.md` has LIST-06 ticked;
`gh issue view 31` answers CLOSED. Commits `8c5d17f4`, `f13ccfb2`, `14120be7`, `dfb2b9f6`,
`e38bf31d` and `75c211fe` are in `git log --oneline --all`. Carriage returns zero and em dashes
zero on this file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

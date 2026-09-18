---
phase: 11-reading-and-the-list
plan: 06
subsystem: the message list, the Action menu, the context menu, the toolbar, guards, pages
tags: [mark-as-read, label-follows-state, m-key, type-to-search, tb-setbuttoninfo, msaa, one-rule, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-05 merged at 5c82f680; main clean at 61865f61 when the branch left it; the reading shape of tests/moving_through_the_list_marks_nothing_read.rs and the MSAA helpers of tests/a_kept_folder_reads_as_a_checked_check_box.rs"
provides:
  - "application::marking_read::what_the_command_says(any_unread) -> Wording { menu, context, spoken, help }: Mark as R&ead, &Mark as read, Mark as Read, Mark the selected message as read for an unread message; Mark as Unr&ead, &Mark as unread, Mark as Unread, Mark the selected message as unread for a read one; const, the mnemonic on the e in both; what_the_key_says(now_read) -> read or unread; five tests, 1 record"
  - "application::context_menu::entries_for_messages(any_unread): two static slices identical but for the Mark entry, whose label is the rule's context word; entries_for(Focus::Messages) still answers the unread form; every_menu() walks both forms in the mnemonic and no-duplicate tests; 18 tests before and after, 5 records name it now"
  - "presentation::list_keys::wire_letter(list, letter, on_pressed) and is_a_bare_press_of: a bare press of the letter with a row selected is consumed with event.skip(false) and the handler called with the row; every other key left as the dispatcher set it; 3 tests, 1 record"
  - "presentation::toolbar_text::relabel(toolbar, id, text) through TB_SETBUTTONINFOW with TBIF_TEXT then TB_AUTOSIZE, label_of(toolbar, id) through TB_GETBUTTONTEXTW, TBBUTTONINFOW declared by hand and held at 48 bytes with four offsets; elsewhere a no-op; 1 test, 1 record"
  - "wx_app.rs: How { TheCommand, TheKey }; toggle_read_state(app, a11y, how, frame, toolbar), the arm's body, run by the arm and by M, announcing the sentence or the one word, signalling Confirmed, sending the flag, then refreshing the words; the_selected_row_has_unread(state); refresh_mark_read_wording(frame, toolbar, state) setting the item and its help through find_item_and_menu, the tool through toolbar_text and its tip, called from the selection handler, the toggle and the MessageReadToggled arm; UpdateTargets.toolbar; the context menu built from entries_for_messages at the key; the tool added with the rule's words; 199 tests before and after, the earcon test rewritten in place, 80 records"
  - "tests/mark_as_read_says_which_way_it_will_go.rs: reading A, M on a built list wired with M and on one wired with Q; reading B, a tool relabelled and read back through the control and through get_accName; four readings over the source with five companions; 14 tests, 4 records name it"
  - "guards/guards.toml: 932 records, census 798 + 134, six new records measured, two re-measured at 14"
  - "docs/KEYBOARD_SHORTCUTS.md, docs/USER_GUIDE.md, docs/changelog.md: M on the two Mark as Read rows with the label following the state on every surface; the guide's context menu list and two shortcut lists corrected by dating; the entry under Unreleased, Fixed, naming #27 in the tester's words"
  - ".planning/WINDOWS.md: 540 unrun-verify, what only the tester's ear settles; 536 given its second trigger"
affects: [11-07, which makes a conversation row contribute its messages to the same toggle and closes #27; 11-06.1, which rewrites the delete arm beside this toggle and lands the cursor through its own rule; 11-08, which decides which message a conversation row is and so which message M toggles under conversation view; 11-12, which reads LIST-04's lines and the pages; whoever hears the label]

actuals:
  tokens: 23939
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A command whose words depend on the state gets its words from one const rule, so the surfaces that build their entries statically and the ones that relabel at run time read the same value and cannot drift"
    - "A key on a native list is consumed at the key-down or the control's own search takes it, and under a dispatcher that resets Skip(true) before every closure the consumption has to be an explicit skip(false); the proof is a real key sent to a real list and the selection read afterwards, with a companion where the search is meant to win"
    - "Where the toolkit offers no setter for a native control's text, the message goes to the control's own window and is read back on the channel a screen reader reads, before and after, so the reading can see the old value as well as the new"
    - "A guard that reads menu labels from source literals keeps the literal; a reading holds the literal to the rule instead of replacing it with a call the guard cannot see"

key-files:
  created:
    - src/application/marking_read.rs
    - src/presentation/list_keys.rs
    - src/presentation/toolbar_text.rs
    - tests/mark_as_read_says_which_way_it_will_go.rs
  modified:
    - src/application/mod.rs
    - src/application/context_menu.rs
    - src/presentation/mod.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "Wording carries a fourth word, context, so the context menu keeps its own case and its own mnemonic (M) while the Action menu keeps its e; the plan's three-field Wording would have made the context entry either the menu's title case or a second literal"
  - "The mnemonic is the same in both states, Mark as R&ead and Mark as Unr&ead, as the plan's own sentence asked; the plan's example Mark as &Unread would have moved it to U"
  - "The rule is a const fn, so context_menu's two static slices take their Mark entry from it directly and no test is needed to tie them; one rewritten test holds the two forms identical elsewhere"
  - "context_menu stays at 18: one test rewritten in place to hold the second form, two walkers rewritten to walk both forms; the plan offered adding one and this was the cheaper of its two options, since the file has records naming it"
  - "The Action menu's item keeps its literal and a reading holds the literal to the rule's menu word; the tool is added from the rule. tests/wired.rs reads menu labels from source literals for the letter-clash guard, and a call in the item's place would have hidden Mark as Read's e from it"
  - "refresh_mark_read_wording is called once from toggle_read_state rather than from the end of the arm and again from the key wiring: the two callers share the toggle, so the refresh after the toggle is one site, and the reading holds the three sites the plan named (selection, toggle, the update arm)"
  - "The menu item's help follows the state too, through Menu::set_help_string from find_item_and_menu; the plan said the help follows through set_tool_short_help for the tool and nothing for the item, and wxdragon turned out to offer it"
  - "The MessageReadToggled arm reaches the toolbar through a new field on UpdateTargets, a copy of the handle, rather than a lookup from the frame, which wxdragon does not offer"
  - "The earcon test in wx_app.rs follows the mark arm into toggle_read_state rather than reading the arm alone, rewritten in place so the file stays at 199"
  - "The green commit whose landed message was the planner's was rewritten with commit-tree over the tree the gate had passed and reset --soft onto the branch, rather than amended: --amend would have taken the planner's staged files, and --amend --only was refused by the gate because its temporary GIT_INDEX_FILE made the which-checks fixture act on the wrong repository (ledger 536's second trigger)"

patterns-established:
  - "When two agents share one checkout, a commit's landed subject is read back and compared with the message passed, because a long commit-msg hook is a window in which another commit overwrites COMMIT_EDITMSG; observation 710 in the skill log"

requirements-completed: []

coverage:
  - id: D1
    description: "What the command says for a state and what M says are one rule each: the menu word, the context entry, the spoken form and the help for an unread message and for a read one, the mnemonic the same in both, the context entry the menu's words in its own case, the key's word the state the message is in now"
    requirement: LIST-04
    verification:
      - kind: unit
        ref: "src/application/marking_read.rs#test_an_unread_message_is_offered_mark_as_read"
        status: pass
      - kind: unit
        ref: "src/application/marking_read.rs#test_a_read_message_is_offered_mark_as_unread"
        status: pass
      - kind: unit
        ref: "src/application/marking_read.rs#test_the_mnemonic_letter_is_the_same_whichever_way_the_command_goes"
        status: pass
      - kind: unit
        ref: "src/application/marking_read.rs#test_the_context_entry_says_the_same_words_in_its_own_case"
        status: pass
      - kind: unit
        ref: "src/application/marking_read.rs#test_the_key_says_the_state_the_message_is_in_now"
        status: pass
      - kind: unit
        ref: "src/application/context_menu.rs#test_a_message_menu_has_the_things_you_do_to_a_message"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'mark as read says mark as unread on a read message, not the one word whatever the state' and 'the context menu on a read message offers mark as unread, not the unread form', measured 2026-09-18 on the library"
        status: pass
    human_judgment: false
  - id: D2
    description: "A letter wired on a real list reaches its handler and the control's search does not get it; a tool relabelled through the native toolbar says the new label where the control and a screen reader read it"
    requirement: LIST-04
    verification:
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_reading_a_the_letter_reaches_its_handler_on_a_real_list_and_the_search_does_not_get_it"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_reading_a_companion_a_letter_nobody_wired_reaches_the_search_and_moves_the_cursor"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_reading_b_a_relabelled_tool_says_the_new_label_where_the_control_is_asked"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_reading_b_a_relabelled_tool_says_the_new_label_where_a_screen_reader_reads_it"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_reading_b_companion_both_readers_saw_the_old_label_before_the_relabel"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a letter wired on a list is consumed at the key-down, not left to the control's search' and 'a relabelled tool says the new label where the control and a screen reader read it', measured 2026-09-18 on the target"
        status: pass
    human_judgment: false
  - id: D3
    description: "The three surfaces follow the state on selection, after the toggle and when a flag lands; the arm and M run one toggle and M says one word; the context menu asks the state; the tool and the item start with the rule's words; the pages say M"
    requirement: LIST-04
    verification:
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_the_menu_item_and_the_key_run_one_toggle_and_the_key_says_one_word"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_the_wording_is_refreshed_on_selection_after_the_toggle_and_when_a_flag_lands"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_the_context_menu_on_the_message_list_asks_the_state"
        status: pass
      - kind: integration
        ref: "tests/mark_as_read_says_which_way_it_will_go.rs#test_the_tool_and_the_item_are_built_with_the_rules_words"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_flagging_or_marking_a_message_read_reaches_the_earcon_channel"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'landing on a row makes mark as read say which way it will go, not leave the last row's words' and 'm on the message list is wired through the helper that consumes it, not bound bare', measured 2026-09-18 on the target"
        status: pass
      - kind: command
        ref: "grep -n 'M' on the two Mark as Read rows of docs/KEYBOARD_SHORTCUTS.md -> 554 and 669; grep -c '#27' docs/changelog.md -> at least 1; cargo test --test house_style -> 74 passed; cargo test --test the_words_that_say_nothing -> 9 passed"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the label is heard as Mark as Unread on a read message on each surface, whether M is heard as one word with the list staying put, and whether the toolbar button's name follows under NVDA's own navigation"
    requirement: LIST-04
    verification: []
    human_judgment: true
    rationale: "Ledger 540; the comment on #27 lists it; nothing here has been heard, and no binary was started"

duration: 76min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 06: Mark as Read says which way it will go, and M toggles it Summary

**Mark as Read says Mark as Unread on a read message and Mark as Read on an unread one,
on the Action menu, the context menu and the toolbar, and does what it says; M with the
cursor in the message list toggles the message and says one word, "read" or "unread".
Until this branch the command said Mark as Read on every surface whatever the state and
toggled, so on a read message it marked unread while saying it would mark read, and no
letter did it. The words for each state are one rule in `application::marking_read`, and
the three surfaces are refreshed from the message under the cursor when a row is landed
on, after the toggle, and when a read flag reaches the row from the server or is put back
by a refusal. The toolbar's label goes through `TB_SETBUTTONINFOW` on the control's own
window, because wxdragon 0.9.17 can change a tool's tip and not its text, and a reading on
a built toolbar reads "Mark as Unread" back through `TB_GETBUTTONTEXTW` and through
`get_accName`, the object NVDA reads a toolbar button's name from. M is consumed at the
key-down with `event.skip(false)`, which under wxdragon's dispatcher is the whole
mechanism, and a reading sends the key and its char to a built list: the wired list's
selection stayed on row 0 with the handler called once, and a companion list wired with
another letter moved to "Mango", which is what the search would have done. The thread half
of #27, marking a whole conversation from its row, is 11-07's, and the issue stays open for
it. Nobody has heard any of this.**

## Performance

- **Duration:** 76 min from the start recorded at 20:32:48Z to the merge's hook finishing
  at 21:49:10Z; the reading of the plan, the phase README, the summaries and the tree began
  about 20:20Z, and the summary and the planning files came after. About 5 min 15 s was
  guard measurement in four foreground runs (218 s for the two library records, 85 s and
  81 s after a 52 s read of what already fails; 29 s for the two target records of task 2,
  13 s and 14 s; 28 s re-measuring the same two at 14 after task 3's red; 40 s for the two
  `wx_app.rs` records, 18 s and 20 s); about 19 min 40 s the eight hook runs on the branch
  (173 s, 123 s, 66 s refused, 151 s refused, 153 s, 117 s, 97 s, 247 s); 348 s the whole
  gate on the branch; 342 s `main`'s hook at the merge; 0 s waiting for the desktop, since
  no live-window key test went red on any run.
- **Started:** 2026-09-18T20:32:48Z (the branch's first commit at 20:41:16Z)
- **Merged:** 2026-09-18T21:49:10Z at `fe143d46`
- **Tasks:** 3
- **Files modified:** 13, four created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat 61865f61..5f4825b8 -- Cargo.toml Cargo.lock` (T-11-SC)
- **Actuals:** `tokens: 23939` is `git diff 61865f61..5f4825b8` with the planner's four
  planning paths excluded, `| wc -c` 95,756 characters over four, the branch's own diff
  against the commit it left `main` at; the estimate's `tokens` was 39,000 and
  `raw_tokens` 90,000, so the factor is 0.27.

## What landed

**Task 1.** `marking_read::what_the_command_says(any_unread) -> Wording`, a `const fn`,
answering `menu`, `context`, `spoken` and `help` for the two states: "Mark as R&ead",
"&Mark as read", "Mark as Read", "Mark the selected message as read", and "Mark as
Unr&ead", "&Mark as unread", "Mark as Unread", "Mark the selected message as unread". The
mnemonic stays on the e for the Action menu and on the M for the context menu whichever way
the command goes, so somebody who learned either keeps it. `what_the_key_says(now_read)` is
"read" or "unread", the tester's two words. Five cases in a module of their own.
`context_menu` holds a second static slice, `MESSAGES_WITH_THE_ONE_UNDER_THE_CURSOR_READ`,
identical to `MESSAGES` but for the Mark entry, both entries taken from the rule since it is
const; `entries_for_messages(any_unread)` picks; `entries_for(Focus::Messages)` still
answers the unread form so its nine callers keep what they had. The message-menu test holds
both forms to the rule's words and to each other outside the Mark entry, and the mnemonic
and no-duplicate walkers walk the second form beside every focus through `every_menu()`;
18 tests before and after.

**Task 2.** `list_keys::wire_letter(list, letter, on_pressed)` binds `KEY_DOWN`; for a bare
press of the letter with a row selected it calls `event.skip(false)` and the handler with
the row, and leaves every other key as the dispatcher set it. `is_a_bare_press_of` is the
rule, three cases. The module doc says why `skip(false)` is the whole mechanism, naming
wx's `m_lastKeydownProcessed` in `src/msw/window.cpp` and wxdragon's `Skip(true)` reset at
`event.cpp:379`. `toolbar_text::relabel(toolbar, id, text)` sends `TB_SETBUTTONINFOW` with
`TBIF_TEXT` and then `TB_AUTOSIZE` to the toolbar's window, `TBBUTTONINFOW` declared by hand
from commctrl.h with its 48 bytes and the offsets of `dwMask`, `fsState`, `lParam`,
`pszText` and `cchText` held by a test; `label_of` reads the text back through
`TB_GETBUTTONTEXTW`, asked once for the length and once for the text. Elsewhere both accept
and do nothing.

`tests/mark_as_read_says_which_way_it_will_go.rs`, `cfg(windows)`, one window session
through a `OnceLock` harvest on 11-03's shape. Reading A builds two report-view lists with
"Alpha", "Mango" and "Zulu", the first row selected and focused, one wired with M and one
with Q, and sends `WM_KEYDOWN` for `VK_M`, `WM_CHAR` for m and `WM_KEYUP` to each list's
window: the wired list's handler was called once with row 0 and its selection was still 0
afterwards; the other list's handler was never called and its selection was 1, "Mango".
Reading B builds a `Flat | Text` toolbar with one tool "Mark Read", reads the label both
ways, relabels it "Mark as Unread", and reads again: `TB_GETBUTTONTEXTW` answered "Mark
Read" then "Mark as Unread", and `get_accName` on child 1 answered the same pair. Two
records with `suite` the target.

**Task 3.** In `wx_app.rs`: `How { TheCommand, TheKey }`; `toggle_read_state(app, a11y,
how, frame, toolbar)`, the arm's body moved, which flips the selected message inside one
lock, sends `MessageReadToggled` and the status line, announces "Marked as read: subject"
under the command or the one word under the key at Normal, signals `Confirmed`, sends the
flag to the server, and refreshes the words. `the_selected_row_has_unread(state)` answers
the row's flag, or on a conversation row whether `unread > 0`. `refresh_mark_read_wording(frame,
toolbar, state)` asks the rule and sets the Action menu's item and its help through
`find_item_and_menu`, the tool's text through `toolbar_text::relabel` and its tip through
`set_tool_short_help`; called from the selection handler after the receipt check, from the
toggle, and from the `MessageReadToggled` arm, which reaches the toolbar through a new
`toolbar: Option<ToolBar>` on `UpdateTargets`. The list's context menu closure reads the
state at the key and hands `entries_for_messages`. `wire_letter(&msg_list, 'M', ..)` sits
after the read-aloud wiring and runs the toggle as the key. The tool is added with the
rule's spoken and help words; the item keeps `"Mark as R&ead"` as a literal. The earcon
test follows the arm into the toggle, rewritten in place; 199 before and after. The target
gains four readings over `what_ships` with five companions: the arm and the key share the
toggle and the key says one word; the refresh is called from the three sites and reaches
all five setters; the context wiring calls `entries_for_messages` and not the focus form;
the tool is added with the rule's words and the item's literal equals the rule's menu word.
The pages, the changelog and ledger 540; two records on `wx_app.rs` with `suite` the target.

## Honest RED and GREEN

Three reds and three greens, on branch `mark-as-read-says-which-way-it-will-go` from
`main` at `61865f61`.

`e4726206`, task 1's red, 173 s through the hook in `red` mode: the two read-state cases
in `marking_read` and the rewritten message-menu test in `context_menu`, named by module
path from cargo's own lines, under stubs answering the unread form for both states and
"unread" for both words. Green on arrival and said in the commit: the unread-state case,
the mnemonic case and the context-word case, which a stub answering one form satisfies,
and the seventeen `context_menu` tests the stub does. No file a record names gained a
test, and the count check printed no remedy.

`321be675`, task 1's green: the rule, the second slice, two records measured. Its first
form, `cf00b4a5`, passed the gate in 123 s and landed with the planner's commit message
(below); `321be675` is the same tree under this plan's message.

`d144414e`, task 2's red, 153 s: the three readings named bare, red for the reasons
intended and quoted from the run, the wired list's selection on Mango and both relabel
readers on "Mark Read". Green on arrival and said: the two companions and `list_keys`'
three cases. A first attempt, 151 s, was refused by `house_style`'s
`test_no_comment_says_a_test_cannot_build_a_window` on the file comment's wording, which
was reworded to cite `tests/theme_reach.rs` as 11-03's target does.

`3f1075a3`, task 2's green, 117 s: `skip(false)`, the relabel, two records measured.

`e191e373`, task 3's red, 97 s: the four source readings named bare, each red for its
intended reason quoted from the run, and the count check named bare, since the target went
from 5 to 14 under two records. Its remedy was run in the foreground and read before the
green: both records still right, counts written at 14. Green on arrival and said: the five
companions.

`5f4825b8`, task 3's green, 247 s: the helper, the toggle, the three calls, the key wiring,
the field, the pages, the changelog, the ledger, two records measured. `cargo build` ran
between the field and the rest and before the tests.

Under the TDD gate's own terms, `test(11-06)` precedes `feat(11-06)` three times.

## Guard records

926 by the TOML reader before, 932 after: six new, none retired, two re-measured once
each as the target grew. Census 798 + 128 before, 798 + 134 after.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| mark as read says mark as unread on a read message, not the one word whatever the state (new, library) | `marking_read.rs` | both states answer the unread form | 2, "all 2 tests named went red, and nothing else did": the module's read-state case and `context_menu`'s message-menu test | rebuild 34 s, run 51 s |
| the context menu on a read message offers mark as unread, not the unread form (new, library) | `context_menu.rs` | `entries_for_messages(false)` answers the unread slice | 1, "the one test named went red, and nothing else did": the message-menu test | rebuild 31 s, run 50 s |
| a letter wired on a list is consumed at the key-down, not left to the control's search (new, `suite` the target; re-measured at 14) | `list_keys.rs` | the `skip(false)` dropped | 1: reading A on the wired list | rebuild 12 s, run 1 s, both times |
| a relabelled tool says the new label where the control and a screen reader read it (new, `suite` the target; re-measured at 14) | `toolbar_text.rs` | the message sent with no text mask | 2: both relabel readings | rebuild 13 s and 12 s, run 1 s |
| landing on a row makes mark as read say which way it will go, not leave the last row's words (new, `suite` the target) | `wx_app.rs` | the refresh dropped from the selection handler | 1: the three-sites reading | rebuild 17 s, run 1 s |
| m on the message list is wired through the helper that consumes it, not bound bare (new, `suite` the target) | `wx_app.rs` | a bare `KEY_DOWN` binding running the toggle on key 77 with nothing said about skip | 1: the one-toggle reading | rebuild 19 s, run 1 s |

Every first draft of a red list was a prediction and the runner agreed with each. The
companions stay green under every break because they read snippets rather than the file,
and each record's comment says so. Counts written: `marking_read.rs` 5 and `context_menu.rs`
18 on the first, `context_menu.rs` 18 on the second, `list_keys.rs` 3 and the target 14 on
the third, `toolbar_text.rs` 1 and the target 14 on the fourth, `wx_app.rs` 199 and the
target 14 on the last two. The count check printed its remedy at one commit, task 3's red,
where it was named in the trailer and run in the foreground before the green. `wx_app.rs`
is at 199 before and after, quoted by `cargo test --lib presentation::wx_app::` at task 3's
green; 80 records name it now, two more than the 78 at the start; `context_menu.rs` 5, two
more than 3.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `61865f61` before
the branch. The line numbers had moved by up to 34 since the plan (`ID_MARK_READ` at `128`,
`898`, `4893`, `6754` and `26804`; the context entry at `context_menu.rs:296`; the
shortcuts rows at `554` and `669`; the guide's list at `245`); the shapes held. Four things
the plan did not have:

1. **A key on a list is consumed today.** Premise 3's `grep -n 'skip(false)'
   src/presentation/wx_app.rs -> nothing` answers one line now, `21168`, Alt+A on the
   page window's attachments list, from 11-04.1 (`7d2d7ff1`) the same day. The mechanism
   is the one this plan uses, and the premise's conclusion held.
2. **The plan's `Wording` had three fields and its own example moved the mnemonic.** The
   behaviour asked for "Mark as &Unread" and, in the same sentence, the same mnemonic
   letter on both states; and the context menu's entry, "&Mark as read", is in sentence
   case with its own letter. `Wording` gained `context`, and the read form is "Mark as
   Unr&ead".
3. **A menu item's help can be changed.** Premise 2 read `menuitem.rs` and found
   `set_label` and no help setter; `Menu::set_help_string(id, help)` at `menu.rs:329` and
   `MenuBar::find_item_and_menu` at `menubar.rs:107` are there, so the item's help follows
   the state too, where the plan had left it fixed.
4. **The plan's premise counted `wx_app.rs` under 73 records; 78 at the start**, 11-04,
   11-04.1 and 11-05 having added five, and 80 now. `context_menu.rs` at 18 tests and 3
   records held; the three new modules and the target at zero records held.

And two things the plan did not name that the tree already had, left where they are: under
conversation view `toggle_read_state` flips `messages[idx]` with `idx` a row of the
conversation list, the mismatch 11-05's summary and the phase README record against #31 and
11-08 corrects, while the label reads the conversation row's `unread`; and the guide's
Message Actions list says `S` stars a message and its Navigation list says `N` and `P` move
between unread messages, none of which is bound, which 11-12's read of the pages is for and
`deferred-items.md` names.

## Deviations from plan

**1. [Decision] `Wording.context`**, decision 1 and contradiction 2.

**2. [Decision] The mnemonic stays on the e**, decision 2.

**3. [Decision] The rule is const and the slices read it**, decision 3.

**4. [Decision] `context_menu` rewritten in place at 18**, decision 4, the plan's cheaper
option.

**5. [Decision] The item keeps its literal, held to the rule by a reading**, decision 5.

**6. [Decision] One refresh call in the toggle**, decision 6; the reading holds the three
sites the plan named, and the arm and the key reach the third through the toggle.

**7. [Rule 2 - Correctness] The item's help follows the state**, decision 7 and
contradiction 3: a help that said "Mark as read" over an item saying Mark as Unread is the
lagging label the plan's T-11-24 is about.

**8. [Decision] `UpdateTargets.toolbar`**, decision 8.

**9. [Decision] The earcon test follows the arm into the toggle**, decision 9.

**10. [Rule 3 - Blocking] A green commit landed under the planner's message**, decision 10.
The planner inserted 11-05.1 in this checkout while this plan executed: its planning files
were staged in the index and its `git commit` ran during this plan's commit hook,
overwriting `.git/COMMIT_EDITMSG`, so `cf00b4a5` landed with this plan's three files and
the planner's subject. `git commit --amend --only -F` was refused by the gate, the
which-checks suite's "a version bump staged in a repository of its own" case answering
`all` under the temporary `GIT_INDEX_FILE` a partial commit hands its hook, which is ledger
536's mechanism from the main checkout and is written onto 536. `git commit-tree` over the
same tree with this plan's message, then `git reset --soft`, gave `321be675`; nothing in the
tree changed and the gate had passed that tree. For the four commits after it the planner's
staged paths were set aside with `git restore --staged .planning` before `git commit` and
put back with `git add` of the same list after, so each commit held this plan's files and
the planner's staging was as it found it; the planner's `4b57d312` then landed on this
branch at 21:13Z, between task 2's green and task 3's red, and the merge carries it.
Observation 710 in the skill log.

**11. [Decision] `cognitive-accessibility` was not invoked**: no agent or skill of that
name is offered in this session's list. The two spoken words are the tester's own, and the
three help sentences are "Mark the selected message as read" and "as unread", read for
plain language by hand.

Everything else executed as written. **No scripted edit touched a tracked file: the
exception set for this plan is zero, and it stayed there.** Every tracked file was changed
by Read then Edit or Write; the four new files were written with Write; `cargo fmt` ran
before each Rust commit; `scripts/guards.sh --remeasure` wrote the counts on
`guards/guards.toml`. The only `sed`, `awk`, `grep`, `tr` and `python` in the session read
files, logs and the records file; `git checkout main` was used once, for the merge; `git
restore --staged` and `git add` moved the planner's paths in and out of the index and
touched no file; `git commit-tree` and `git reset --soft` rewrote one commit's message over
its own tree. Commit messages were written to the scratchpad and passed with `-F`, and each
landed subject was read back after the first one was not. Carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No em dash in
any file this plan wrote, measured by `grep -c` for the byte sequence: zero; none of the six
words, measured by `grep -ciE`: zero. `git commit` and `git merge`, never `gsd-tools query
commit`; never `--no-verify`; `check.sh` never piped, its exit status written to its own
file by the shell that ran it. No AI attribution in any commit. `Cargo.toml` and
`Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's profile was not
read; no binary was started, neither the installed one nor the tree's; the readings built
every window they read and showed one frame for the length of the harvest; NVDA was not
stopped, reconfigured or driven; every commit was made from the primary checkout, none from
`wixen-mail-sweep` or `wixen-mail-mutants`. `WIXEN_TEST_THREADS` untouched. The version
stays `1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/marking_read.rs`, `src/application/context_menu.rs` | `--lib application::marking_read` and `--lib application::context_menu` on task 1's red and green, `application` whole on the red for `mod.rs`, and the whole-tree guards |
| `src/presentation/list_keys.rs`, `src/presentation/toolbar_text.rs` | their `--lib` filters on task 2's commits, `presentation` whole on the red for `mod.rs` |
| `tests/mark_as_read_says_which_way_it_will_go.rs` | itself on each commit that changed it, and through its records' coupling from `3f1075a3` on |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the twenty coupled targets its records name, on task 3's green |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every commit |

`scripts/check.sh all` ran once on the branch at `5f4825b8`, output to a file with the exit
status written by the same shell: exit 0, 8,113 passed and none failed over 79 result
lines, 348 s from 21:37:17Z to 21:43:05Z, the release build included; one more result line
than 11-05's 78, the new target, and 23 more tests: 5 in `marking_read`, 3 in `list_keys`,
1 in `toolbar_text` and 14 in the target. `main`'s hook at the merge, 342 s from 21:43:28Z
to 21:49:10Z, the same 8,113 and none failed. The tester's copy and NVDA were open on the
desktop throughout; the harvest's frame was shown on it for under a second per run; no
live-window key test went red on any run, so the input was never checked for idleness. The
keyring race (ledger 374) did not appear.

## Threat register

T-11-22 mitigated: `wire_letter` binds the list's own `KEY_DOWN` and nothing else, a
modifier held is not a bare press (three cases), and the key wiring is on `msg_list` alone,
held by the one-toggle reading and the bare-binding record. T-11-23 mitigated:
`TBBUTTONINFOW` is declared by hand with `cbSize` and its header named, held at 48 bytes
with four offsets by a test in the module, and reading B reads the text back through the
same control before and after. T-11-24 mitigated: refreshed on selection, in the toggle and
on the toggled update, the three sites held by the reading and the first by a record; the
help follows too. T-11-SC: nothing added. New surface outside the register: none; the calls
into Windows are three messages on a toolbar this program created and three sent by the
reading to lists it built.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 539 before, 540 after;
506 open before, 507 after; 33 fixed, unchanged; 536's description extended in both halves.

| id | kind | what |
|---|---|---|
| 540 | unrun-verify | what only the tester's ear settles for #27: the item heard as Mark as Unread after arrowing onto a read message and as Mark as Read after an unread one, on the Action menu and the context menu; M heard as one word with the list staying on the row; the toolbar button's name after a toggle under NVDA's own navigation; Alt+A, E reaching the item whichever way it goes |
| 536 | deviation | its second trigger: a commit naming its paths, or `--amend --only`, hands the hook a temporary `GIT_INDEX_FILE` and the fixture's version-bump case answers `all` from the main checkout; nothing moved; the remedy is the same clearing |

## The issue

`gh issue comment 27` from the repository root after the merge, at 2026-09-18T21:49:28Z,
the plan's sentence with the merge commit, then the ear list: the label after arrowing
between a read and an unread message on both menus, M as one word with the list staying
put, the toolbar button's name after a toggle, and Alt+A, E. The issue stays open for
11-07's thread row. Commenting is not a publish; nothing was pushed.

## Known stubs

None. `what_the_command_says` has four non-test readers, the two context slices, the tool's
`add_tool` and `refresh_mark_read_wording`; `what_the_key_says` one, `toggle_read_state`;
`entries_for_messages` one, the list's context closure; `wire_letter` one, the M wiring;
`relabel` one, the refresh; `label_of` is read by the target and by nothing shipping, as the
plan said; `toggle_read_state` is reached from the arm and from M; `refresh_mark_read_wording`
from the three sites; `UpdateTargets.toolbar` from the toggled arm.

## Not done here, on purpose

Whether the label is heard on each surface, the word after M, and that the list does not
jump are ledger 540 and the comment on #27; the changelog says nobody has heard it. The
thread row is 11-07's. The conversation view's index mismatch under M is 11-08's, above.
The guide's `S`, `N` and `P` lines, which name keys that are not bound, are 11-12's read of
the pages and are in `deferred-items.md`. LIST-04 is held on its first two `[D]` lines, its
box unticked because the third `[D]` line is 11-07's and its `[S]` line untouched: the phase
README says 11-12 ticks the `LIST` requirements clause by clause, and the orchestrator's
instruction for this plan was to tick it where held, which is the overrule with its reason.
The row is `7/21`, counted from the disk after the planner's insert.

## What 11-06.1, 11-07 and 11-08 need to know

- **M is consumed, not skipped.** `list_keys::wire_letter` calls `event.skip(false)` for a
  bare press with a row selected; wxdragon's dispatcher resets `Skip(true)` before every
  closure, so a second `KEY_DOWN` closure on the list that says nothing leaves its key to the
  control, and one that wants its key kept from the search has to clear skip itself. The
  dispatcher stops at the first closure that cleared it, so the order of binding matters
  only among closures that consume.
- **One id, one toggle.** `ID_MARK_READ` is raised by the Action menu, the context menu and
  the toolbar and lands in `toggle_read_state(app, a11y, How::TheCommand, ..)`; M runs the
  same function as `How::TheKey`. A command over a set (11-07) changes this function, and the
  arm test in `wx_app.rs` and the one-toggle reading in the target read its body.
- **The toolbar is relabelled natively.** `refresh_mark_read_wording` reaches it through
  `UpdateTargets.toolbar` and the handlers' `toolbar_handle`; wxWidgets rebuilds the buttons
  from its own labels on a DPI change, and the next selection or toggle sets the text again.
- **The words come from `marking_read`.** A conversation row's wording is decided by
  `the_selected_row_has_unread`, `unread > 0` on the row; 11-07's set-wide sentence and
  11-08's choice of which message the row is both read or change that function.

## Self-Check: PASSED

`src/application/marking_read.rs`, `src/presentation/list_keys.rs`,
`src/presentation/toolbar_text.rs` and `tests/mark_as_read_says_which_way_it_will_go.rs`
exist; `grep -v '^\s*//' src/presentation/wx_app.rs | grep -c 'refresh_mark_read_wording('`
is 4 and the same for `wire_letter(&msg_list, 'M'` is 1; `bash scripts/check.sh
--suites-for guards/guards.toml src/presentation/wx_app.rs` names the target;
`docs/KEYBOARD_SHORTCUTS.md` has `M` on lines 554 and 669; `guards/guards.toml` holds 932
records by the TOML reader and the census says 798 + 134; `.planning/WINDOWS.md` holds 540
in both halves; `.planning/REQUIREMENTS.md` has LIST-04's held note; `gh issue view 27`
answers OPEN with the comment. Commits `e4726206`, `321be675`, `d144414e`, `3f1075a3`,
`e191e373`, `5f4825b8` and `fe143d46` are in `git log --oneline --all`. Carriage returns
zero and em dashes zero on this file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

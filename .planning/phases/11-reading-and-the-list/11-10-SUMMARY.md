---
phase: 11-reading-and-the-list
plan: 10
subsystem: a rule that changes how a row is announced, not only how it is filed: the action Say this first with its phrase kept on the message and prefixed to the row's first visible cell, a Says first column and a Labels column, the Rule matched event signalled once per check with the count, the rule editor offering the action and the sound box; guards, pages, ledger
tags: [rules, filters, say-first, rule-matched, earcons, feedback-tab, labels-column, message-columns, virtual-rows, conversation-query, rule-editor, schema-additive, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-09.2 merged at 4d9a41d2 (the snippet the phrase now sits in front of); 11-09 (the row read on request composed from virtual_rows::text_for at the press, and heading_is_worth_saying as a closed list the target walks); 11-09.1 (FeedbackSettings::the_default_for as where an event's default lives, every channel for an unmarked event, and the Feedback tab's row built from Event::ALL); 11-08 (a conversation row's cells from the here group and the row message's ordering, the_message_the_row_stands_for!); 11-07 (the cursor is the focused row); 10-02's attach_labels and tags_by_message_in_folder; 10-04's one WhatArrived per check and one NewMail per arm; main clean at ced898eb when the branch left it"
provides:
  - "application::filters: FilterAction::SayFirst(String), SAY_FIRST_LIMIT (40), FilterRule.plays_a_sound, Outcome.say_first with the last rule's phrase winning and a delete dropping it, from_persisted_rule reading say_first through validated_phrase (trimmed, refused empty or over the bound in characters), FilterEngine::rules_matching handing the matching rules out with evaluate_message mapping them to actions as before; 49 tests, 46 before, 2 records name the file"
  - "application::mail_sync: Filtered.matches_with_a_sound counted one per message per rule carrying the flag, before the outcome is settled; carry_out writing the phrase through MessageCache::set_says_first; apply_rules public for the target; 149 tests as before, 14 records"
  - "data::message_cache: messages.says_first TEXT and message_filter_rules.plays_a_sound INTEGER NOT NULL DEFAULT 0, both through ensure_column_exists; MessageFilterRule.plays_a_sound read and written by the rule functions; set_says_first(id, Option<&str>); every listing query selecting m.says_first at 23 and listing_row reading it into MessageListRow.says_first, the search's own reader too; the here CTE carrying says_first; conversations_query selecting the row message's phrase at 19 and every label on any message of the conversation once at 20, conversation_row filling ConversationItem.says_first and .labels; the listing guard's closed set widened to tags with its sentence and its companion planting a read of message_bodies; the two sorting fixtures carrying a phrase and a label each; 179 tests as before, 25 records"
  - "presentation::message_columns: MessageColumn::SaysFirst (Says first, says_first) and Labels (Labels, labels) in ALL at 17, off in every default layout; sort expressions m.says_first COLLATE NOCASE and THE_LABELS_ON_A_MESSAGE over message_tags and tags by name; conversation expressions the_message_the_row_stands_for!(\"says_first\") and THE_LABELS_ON_A_CONVERSATION over here; SaysFirst joins heading_is_worth_saying's list, Labels does not; 43 tests as before, 2 records"
  - "presentation::message_rows: cell_text's SaysFirst the phrase or nothing and Labels the names joined by a comma; conversation_cell_text's the row message's phrase and the lines said as a list; 41 tests as before, one fixture given both, 3 records"
  - "presentation::virtual_rows::text_for: the phrase prefixed to the first visible cell under either view through said_first, a comma for the pause, the phrase alone when the cell is empty, never when the Says first column is itself first; 6 tests as before, 2 records"
  - "presentation::ui_types: MessageItem.says_first from the row; UIUpdate::WhatArrived { what, matches_with_a_sound }; 79 tests as before"
  - "presentation::accessibility::feedback::Event::RuleMatched: rule_matched, Rule matched, Normal, Tone::new(1175, 80), every channel by default through the_default_for since nothing marks it as on the row; 54 tests as before, 8 records"
  - "presentation::wx_app: the check summing matches_with_a_sound over its folders into the one WhatArrived after the loop, the POP check passing its own, the download passing nought; the arm signalling RuleMatched once after NewMail when the count is above nought with how_many(n, \"message\") as the detail and in the log; 199 tests as before, 196 attributes, 100 records"
  - "presentation::wx_managers: RULE_ACTIONS gains (say_first, Say this first); the_value_label_for(action_words) answering P&hrase to say first: under it and Action &Value: otherwise; what_stops_the_rule_being_saved(action_words, value) refusing a missing or over-long phrase in a sentence; the value row built by hand so the label is held and the box's accessible name follows it, set once and on every change of the list; the sound box Play a &sound when this rule matches after Enabled, set from a stored rule and read back into FilterRule.plays_a_sound; OK refusing through the helper with the focus on the box; populate_filters saying every action in the editor's words; shown_action and stored_action public; 44 tests as before, 11 records"
  - "presentation::managers: the flag carried both ways between the stored rule and the editor's; 137 tests as before"
  - "tests/a_rule_can_change_how_a_row_is_announced.rs: six cases over the engine and the cache through the real cache and apply_rules, five over the row's cells and the prefix under both views and through the real conversation query, one over the columns and the headings, one over the event, the arm reading holding both halves of the bound with two companions, two pure readings over the editor's helpers, one wxdragon::main over the built editor, one over the stored name's round trip; 20 tests, 5 records name it as their suite"
  - "guards/guards.toml: 989 records, census 798 + 191; six new (two on filters.rs and mail_sync.rs, three on virtual_rows.rs, wx_app.rs and feedback.rs, one on wx_managers.rs), one rewritten for the changed text_for, five census records measured again"
  - "docs/USER_GUIDE.md: Rules that change how a row is announced, under Reading and Managing Email, and the Message Filters line pointing at it; docs/KEYBOARD_SHORTCUTS.md: the table for the Add or Edit Filter Rule dialog; docs/changelog.md: one entry under Added for #62 with five known limitations"
  - ".planning/WINDOWS.md: 556 unrun-verify, what only the ear settles"
affects: [11-10.1, whose links_in_text leaves the snippet bare and now sits behind a phrase when a rule gave one; 11-13, which reads no new status sentence since the event's words go through signal and the two refusals are the editor's own dialogs; 11-12, which reads LIST-08's lines and the guide's new section; whoever hears a phrase first on a row, the sound once after a check, or the Labels column; whoever adds a sound-scheme clip for rule_matched to Soft Chimes; whoever next adds a column, who sorts it into heading_is_worth_saying on purpose and gives the two sorting fixtures something to say in it]

actuals:
  tokens: 36916
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A new column on a struct the tree builds in many places is paid for in literals, not in logic: two fields cost twenty-five files in the red, every one Read then Edit, and the compiler's list of missing fields is the only complete inventory of them"
    - "A column whose value lives in another table is still one expression for the cell and the sort when the conversation query selects it over the rows of here; joining it in Rust from a per-message read would be a second spelling, and the file's own rule refuses that"
    - "A guard record's break that cannot be written to compile at the site the plan names is written at the site the reading can see: the sound per folder is a per-folder send of the update the arm runs once for, and the reading holds both halves of the bound"
    - "A mnemonic the plan hands out is checked against the dialog's letters before it is typed: P was the pattern's and F the field's, so the box took S and the phrase's label H"

key-files:
  created:
    - tests/a_rule_can_change_how_a_row_is_announced.rs
  modified:
    - src/application/filters.rs
    - src/application/mail_sync.rs
    - src/application/conversations.rs
    - src/application/blocking.rs
    - src/application/pop_sync.rs
    - src/application/saved_searches.rs
    - src/application/export_tree.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/filters.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/saved_searches.rs
    - src/data/message_cache/searching.rs
    - src/data/message_cache/tags.rs
    - src/data/message_cache/outbox.rs
    - src/presentation/ui_types.rs
    - src/presentation/message_columns.rs
    - src/presentation/message_rows.rs
    - src/presentation/virtual_rows.rs
    - src/presentation/view_state.rs
    - src/presentation/accessibility/feedback.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_managers.rs
    - src/presentation/managers.rs
    - src/presentation/sample_mailbox.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/read_aloud.rs
    - src/presentation/reader_text.rs
    - tests/a_conversation_row_stands_for_one_message.rs
    - tests/the_rows_columns_are_read_on_request.rs
    - tests/integration_tests.rs
    - tests/manager_dialog_labels.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "A conversation row's labels are the column's own SQL expression over the rows of here, selected into ConversationItem.labels one per line and ordered by, rather than the plan's union in Rust from the per-message read: message_columns.rs's rule is that what a conversation cell shows and what it sorts by are one expression, and a Rust join beside a SQL sort would be the second spelling that rule exists to refuse; the flat view keeps 10-02's per-folder read for the cell and sorts by the same join in SQL"
  - "The listing guard's closed set of tables gains tags, with its sentence, because the Labels column sorts by name; its companion now plants a read of message_bodies, since the read it planted is the one this plan allows on purpose"
  - "The sound box's letter is S and the phrase label's is H, not the plan's P and not F: P is &Pattern's and F is Match &Field's in the same dialog, and two controls on one letter is a key that lands on whichever comes first; the acceptance grep for the plan's phrase without the ampersand therefore answers 0 and the grep for the label as written answers 1"
  - "The wx_app.rs record's break is the count sent once per folder rather than a signal inside the folder loop: the loop's closure has no a11y in scope, so a per-folder signal there cannot be written to compile, and the runner measures only a break that builds; the per-folder send is the same flood, and the reading holds the second half of the bound so the break reddens it"
  - "The RuleMatched tone is 1175 Hz for 80 ms, placed in the widest gap left near the top of the range, 129 Hz over HasAttachment's tick at twice its length and 145 Hz under the reminder at less than half; a proposed number like the rest, until a listening pass moves it"
  - "apply_rules is public, so the target runs the rules over a real cache with no server in the way; the alternative was a scripted server per case for what is a cache-and-engine question"
  - "The manager's list says every action in the editor's words, Mark as read and not mark_as_read, because the new action's stored name with a phrase after it, say_first (Urgent), would have been a machine name in a list somebody hears; the change reaches the six older actions too and their tests stayed green"
  - "The phrase is kept until another rule's phrase replaces it and nothing clears it, said in the doc, the guide and the changelog: the rules run once on arrival, and a clearing action would be a second action nobody asked for"
  - "The count check's remedy was run in the foreground at every red that fired it (three times, for filters.rs and the target) and every new record measured on the green tree; the two target records measured against the red tree were measured again on the green"

patterns-established:
  - "A plan's acceptance grep for a label is a grep for the label as written, ampersand included; the phrase without it is the accessible name and lives nowhere in the source"

requirements-completed: [LIST-08]

coverage:
  - id: D1
    description: "FilterAction::SayFirst stored as say_first with the phrase bounded, written onto the additive says_first column, carried into MessageItem, prefixed to the first visible cell under either view, shown in a Says first column; a Labels column off by default; both in the Columns dialog"
    requirement: LIST-08
    verification:
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_a_rule_saying_a_phrase_first_leaves_it_on_the_message_and_the_listing_carries_it"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_when_two_rules_say_a_phrase_first_the_later_rules_phrase_wins"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_an_empty_phrase_and_one_over_the_bound_are_refused_where_a_stored_rule_is_read"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_schema_opened_twice_keeps_the_phrase_and_the_sound_flag"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_phrase_is_said_before_the_first_visible_cell_of_a_message_row"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_phrase_is_said_before_the_first_visible_cell_of_a_conversation_row"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_phrase_is_not_said_twice_when_the_says_first_column_is_itself_first"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_says_first_and_labels_cells_say_the_phrase_and_the_labels"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_a_conversation_row_carries_its_row_messages_phrase_and_every_label_once"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_two_columns_are_offered_off_by_default_and_the_phrases_heading_is_not_said"
        status: pass
      - kind: unit
        ref: "src/application/filters.rs#test_the_last_phrase_said_first_wins_and_touches_no_server"
        status: pass
      - kind: unit
        ref: "src/application/filters.rs#test_a_stored_say_first_rule_is_read_with_its_phrase_trimmed_and_bounded"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#test_every_column_still_orders_two_messages_the_way_that_column_reads"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/messages.rs#test_sorting_by_a_column_agrees_with_what_that_column_says"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a rule's phrase said first survives settling, not dropped on the way to the message' at 2 red on the target and 'a rule's phrase is said before the row's first cell, not only shown in its column' at 2 red on the target, measured 2026-09-19 on the green tree; src/presentation/wx_columns.rs walks ALL, so the dialog offers seventeen, its own test holding columns.len() to ALL.len()"
        status: pass
      - kind: command
        ref: "cargo test --test a_rule_can_change_how_a_row_is_announced -> 20 passed; --lib application::filters:: -> 49; --lib data::message_cache::messages:: -> 179; --lib presentation::message_columns:: -> 43; --lib presentation::message_rows:: -> 41; --lib presentation::virtual_rows:: -> 6; grep -c 'SayFirst' src/application/filters.rs -> 9; grep -c '\"says_first\"' src/data/message_cache/mod.rs -> 1; grep -c 'pub const ALL: \\[MessageColumn; 17\\]' src/presentation/message_columns.rs -> 1"
        status: pass
    human_judgment: false
  - id: D2
    description: "An additive plays_a_sound flag per rule with a box in the editor; Event::RuleMatched with its own tone and Feedback row; the check counts the matches that sounded and the WhatArrived arm signals the event once per check, never per message; the editor offers the action and the box; readings hold the prefix, the columns, the bound and the arm"
    requirement: LIST-08
    verification:
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_a_rule_with_a_sound_counts_once_per_message_it_matched_and_one_without_counts_nothing"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_a_check_with_no_sounding_rule_counts_no_match_with_a_sound"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_rule_matched_event_has_its_own_words_and_key_and_reaches_every_channel"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_a_check_with_matches_plays_the_sound_once_and_never_per_folder_or_per_message"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_sound_reading_passes_a_window_shaped_as_it_should_be"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_sound_reading_complains_when_the_signal_is_per_folder_or_unguarded_or_twice"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_editor_offers_the_action_and_the_sound_and_reads_a_stored_rule_back"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_value_box_is_called_the_phrase_under_say_this_first_and_the_value_otherwise"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_say_this_first_cannot_be_saved_without_a_phrase_or_with_one_over_the_bound"
        status: pass
      - kind: integration
        ref: "tests/a_rule_can_change_how_a_row_is_announced.rs#test_the_stored_action_and_its_words_round_trip_for_say_this_first"
        status: pass
      - kind: integration
        ref: "tests/every_event_has_a_control.rs#a_fresh_profile_opens_with_the_sounds_on_and_the_attachment_event_on_its_own_default"
        status: pass
      - kind: unit
        ref: "src/presentation/accessibility/feedback.rs#test_every_event_has_its_own_tone"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'only a rule that carries the sound flag counts as a match with a sound' at 2 red, 'the rule-matched sound plays once per check, not once per folder that held a match' at 1 red, 'the rule editor opens a rule with the sound as it was stored, not with the box clear' at 1 red, all on the target, and 'the rule-matched event has a tone of its own, not another event's' at 1 red on the library, measured 2026-09-19 on the green tree; the five records whose red list is the Feedback tab's census measured again, each as recorded"
        status: pass
      - kind: command
        ref: "cargo test --lib presentation::accessibility::feedback:: -> 54; --lib presentation::wx_app:: -> 199; --lib presentation::wx_managers:: -> 44; --test progress_is_shown_and_results_are_said -> 15; --test every_event_has_a_control -> 1; --test manager_dialog_labels -> 1; grep -c 'RuleMatched' src/presentation/accessibility/feedback.rs -> 5; grep -v '^\\s*//' src/presentation/wx_app.rs | grep -c 'FeedbackEvent::RuleMatched' -> 1; grep -c '\"plays_a_sound\"' src/data/message_cache/mod.rs -> 1; grep -c '\"say_first\"' src/presentation/wx_managers.rs -> 1; grep -c 'Play a &sound when this rule matches' src/presentation/wx_managers.rs -> 1"
        status: pass
    human_judgment: false
  - id: D3
    description: "The guide says where the phrase is heard and shown and why the prefix cannot be hidden, the sound box and its Feedback row, and the Labels column and how to switch it on; the shortcuts page has the dialog's letters; the changelog names #62 with the limitations; the ledger carries the ear"
    requirement: LIST-08
    verification:
      - kind: command
        ref: "cargo test --test house_style -> 74; --test the_words_that_say_nothing -> 9; --test the_planning_files_agree_with_themselves -> 16; --test wired -> 77 (the shortcuts page and the menus agree); grep -c '#62' docs/changelog.md -> 1; grep -c 'Say this first' docs/USER_GUIDE.md -> 1 line, 'Says first' 1, 'Labels' 1; carriage returns 0 on every file touched by tr -cd '\\r' | wc -c; em dashes 0 and the six words 0 over every diff's added lines"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether a phrase is heard first on a row and alone when the first cell is empty, whether the sound plays once after a check with several matches across folders, and whether the Labels column reads as part of the row"
    requirement: LIST-08
    verification: []
    human_judgment: true
    rationale: "Ledger 556; the close comment on #62 lists it; nobody has heard any of it, and the tester's copy is what runs on this machine"

duration: 145min
completed: 2026-09-19
status: complete
---

# Phase 11 Plan 10: A rule can change how a row is announced Summary

**A rule can change what a row says and not only where the message is filed (#62). The action
Say this first carries a phrase of up to forty characters; every message the rule matches as it
arrives keeps the phrase in an additive column, every listing reads it, and the list prefixes it
to the row's first visible cell under either view, "Urgent, Unread, Quarterly report", or says
it alone when that cell is empty, so it is the first thing a screen reader says whatever
columns are shown; a Says first column shows it, and Ctrl+Shift+; says it without its heading.
Any rule can play the sound scheme's new Rule matched event, one event for every rule, once per
check however many messages matched and however many folders held them, with the count as its
words and its own row on the Feedback tab. A Labels column, off by default, reads a message's
labels by name and a conversation's once each, both as the column's own SQL expression. The
rule editor offers the action seventh, calls the box under it the phrase, refuses a phrase
missing or over the bound, and carries the sound box. Nobody has heard a phrase first, the
sound, or the labels on a row; #62 is closed from the merge with the ear list.**

## Performance

- **Duration:** 145 min from the start at 18:22:14Z, after the reading of the README, the
  plan, `CLAUDE.md`, the workflow and the nineteen summaries, to the merge's hook finishing
  at 20:47:03Z; the issue closed at 20:47:48Z; the summary and the planning files after. About
  11 min 40 s was guard measurement in seven foreground runs: 2 min 29 s for the one record
  the first red flagged (41 s rebuild, 53 s run, against the red tree); 3 min 12 s for the two
  new task 1 records and that one again on the green tree (22 s and 1 s each, then 38 s and
  51 s); 42 s for the two target records the second red flagged (17 s and 1 s, 20 s and 1 s);
  4 min 6 s for the three task 2 records and the five census records (16 s and 1 s, 18 s and
  1 s, 33 s and 49 s, then 13 to 17 s and 1 s each); 18 s for the rewritten `text_for` record
  (15 s and 1 s); 1 min 24 s for the four target records the third red flagged (17 to 20 s and
  1 s each); 17 s for the editor record (13 s and 1 s). About 43 min the twelve hook runs on
  the branch (406 s; 169 s refused, 160 s; 328 s; 305 s refused, 298 s; 185 s; 14 s refused
  by clippy, 240 s; 158 s); 422 s the whole gate, green on its first run; 323 s `main`'s hook
  refused by the keyring race of ledger 374 on `a_move_says_what_has_not_been_sent`, a target
  this plan never touched, and 403 s green on the retry. 0 s waiting for the desktop: no
  live-window test went red on any run.
- **Started:** 2026-09-19T18:22:14Z
- **Merged:** 2026-09-19T20:47:03Z at `39d53503`
- **Tasks:** 3
- **Files modified:** 37, one created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat ced898eb..53c8cb97 -- Cargo.toml Cargo.lock` (T-11-SC), no crate and no
  feature added
- **Actuals:** `tokens: 36916` is `git diff ced898eb..53c8cb97 | wc -c`, 147,664 characters
  over four, the branch's own diff against the commit it left `main` at; the estimate's
  `tokens` was 52,000 and `raw_tokens` 120,000, so the factor is 0.71 against the former and
  0.31 against the latter. Twenty-five of the thirty-seven files are the two new struct fields
  reaching every literal of four structs, which is where the plan's "eighteen files" became
  thirty-seven.

## What landed

**Task 1.** `FilterAction::SayFirst(String)` with its doc, `SAY_FIRST_LIMIT` at 40 with why,
`FilterRule.plays_a_sound` and `Outcome.say_first`; `from_persisted_rule` reads `say_first`
through `validated_phrase`, which trims like every other value and refuses over the bound in
characters; `settle` keeps the last phrase as it keeps the last answer to read and starred, a
delete dropping it with everything else; `FilterEngine::rules_matching` hands the matching
rules out and `evaluate_message` maps them to actions as before. `apply_rules` counts one match
per message per rule carrying the flag, before the outcome is settled so a rule that only
marks a message read still sounded, into `Filtered.matches_with_a_sound`; `carry_out` writes
the phrase through `MessageCache::set_says_first`. The cache: `messages.says_first TEXT` and
`message_filter_rules.plays_a_sound INTEGER NOT NULL DEFAULT 0` through `ensure_column_exists`
beside the other additive columns, `MessageFilterRule.plays_a_sound` read and written by the
rule functions, every listing query selecting `m.says_first` at 23 and `listing_row` reading
it into `MessageListRow.says_first`, the search's own reader and the outbox's row too;
`MessageItem.says_first` from the row. Every literal of the four structs gained its field:
seventeen `MessageFilterRule`, fourteen `filters::FilterRule`, four `wx_managers::FilterRule`,
twenty `MessageItem` and six `MessageListRow`, in twenty-five files, each by Read then Edit.
`filters.rs` 46 to 49; `mail_sync.rs`, `messages.rs`, `mod.rs` and `ui_types.rs` unchanged in
count. The target's six cases: the phrase on the message and in the listing, the later rule
winning, the reader's refusals, the count of sounding matches, a silent rule counting nothing,
the schema opened twice. Two records with the target as suite.

**Task 2.** `MessageColumn::SaysFirst` and `Labels` in `ALL` at 17 after `Cc`, off in every
default layout, with headings, keys, message expressions (`m.says_first COLLATE NOCASE`, and
the names of a message's labels one per line by name through `message_tags` and `tags`) and
conversation expressions (`the_message_the_row_stands_for!("says_first")`, and every label on
any message of the conversation once through `here`); the `here` CTE carries `says_first`;
`conversations_query` selects both at 19 and 20 and `conversation_row` fills
`ConversationItem.says_first` and `.labels`; the listing guard's closed set widened to `tags`
with its sentence, its companion planting a read of `message_bodies` instead; the two sorting
fixtures carrying a phrase and a label each, ordered as their subjects are. `cell_text` and
`conversation_cell_text` say the phrase or nothing and the labels as a list. `text_for` reads
the column's index as well as the column, and at index nought prefixes the row's phrase through
`said_first` unless the column is Says first. `heading_is_worth_saying` gains Says first, and
11-09's target sorts both new columns on purpose. `Event::RuleMatched` with its key, words,
Normal priority and tone; `UIUpdate::WhatArrived` carries `matches_with_a_sound`, summed over
the check's folders, the POP check's own, the download's nought; the arm signals the event once
after `NewMail` when the count is above nought, with `how_many(n, "message")` as the detail and
in the log. The target: the prefix under both views with the inbox's layout, the phrase alone
when Unread is empty, no doubling when Says first is first, the two cells, a conversation of
three through the real query with the row message's phrase and every label once, the columns
and the headings, the event's words and channels, the arm reading holding the one signal in
the arm and the one send after the check's loop with two companions. Three records; five
census records measured again; one record rewritten for the changed line of `text_for`.

**Task 3.** `RULE_ACTIONS` gains `say_first` with the words Say this first; `the_value_label_for`
answers "P&hrase to say first:" under it and "Action &Value:" otherwise;
`what_stops_the_rule_being_saved` refuses a missing phrase and one over the bound, each in a
sentence; the value row is built by hand so the label is held, and the box's accessible name
follows the label, set once for what is showing and again on every change of the Action list;
the sound box "Play a &sound when this rule matches" after Enabled, set from a stored rule and
read back; OK refuses through the helper with the focus on the box; the manager's list says
every action in the editor's words; `shown_action` and `stored_action` public. The target: the
two helpers read directly, the built editor in the file's one `wxdragon::main` offering the
seven actions in order, the box carrying its label and clear on a new rule, a stored rule with
the sound opening ticked and one without clear, a stored Say this first rule opening with its
action selected, its phrase in the box and the box called the phrase; the round trip of the
stored name. One record. The shortcuts page's table for the dialog went in the same commit as
the box's letter. Then the guide's section, the changelog's entry and ledger 556.

## Honest RED and GREEN

Three reds and three greens, then a documents commit, on branch
`a-rule-can-change-how-a-row-is-announced` from `main` at `ced898eb`.

`029ebdcc`, task 1's red, 406 s through the hook in `red` mode: four target cases named bare
and two `filters.rs` cases by module path, under the first slice that compiles under the
dead-code lint: the variant, the bound, the flag on both rule types and the table, the column
with its setter and its place in every listing, the fields on the row and the item, the count
on `Filtered`; the reader with no arm for `say_first`, `settle` dropping the phrase,
`carry_out` writing nothing and nothing counting. Three green on arrival and said: the delete
dropping the phrase, a silent check counting nothing, the schema opened twice. The count
check fired for `filters.rs` and was named in the trailer; its remedy ran in the foreground
before the green, 3 red as recorded, 49 written, the runner saying the measurement was
against a tree that is not green.

`ac764869`, task 1's green, 160 s after a first attempt refused at 169 s: the reader arm,
`validated_phrase`, `settle`, `carry_out`, `rules_matching` and the count; two records measured
and the flagged one measured again on the green tree. The refusal was the sweep header's census,
185 arrived since, which the two new records moved to 187.

`5e8ec08d`, task 2's red, 328 s: four target cases named bare, under the slice: the two
columns with their expressions, the `here` CTE, the conversation query's two columns and the
reader, the cell arms, the event, the count on `WhatArrived` logged at the arm, the listing
guard's set and its companion, the two fixtures. Six green on arrival and said: no doubling
when the column is first, the two cells, the conversation through the real query, the event's
words, the reading's two companions. The count check fired for the two target records; the
remedy ran before the green, both as recorded, 16 written.

`2b845b3d`, task 2's green, 298 s after a first attempt refused at 305 s: the prefix, the
heading rule, the arm's signal, the reading's second half; three records measured and the five
census records measured again. The refusal was `test_every_guard_record_still_names_one_place_in_the_tree`:
the record "the function the list paints with cannot open the cache" breaks on the line
`text_for` now reads the index on, rewritten onto the new line with its comment dated and
measured again, 5 red as recorded.

`44c3b4c8`, task 3's red, 185 s: four target readings named bare, under the slice: the sound
box with S and its read-back, the value row built by hand, two helpers answering today's label
and today's refusal, the two name conversions public. The count check fired for the four target
records; the remedy ran before the green, every one as recorded, 20 written.

`560e78cf`, task 3's green, 240 s after a first attempt refused in 14 s by clippy's
`redundant_locals` on two rebindings that wxdragon's `Copy` handles made needless: the seventh
action, the helpers, the label following the action, the box from a stored rule, the refusal at
OK, the manager's words, the shortcuts table; one record measured.

`53c8cb97`, the guide, the changelog and the ledger, 158 s: documents only.

Under the TDD gate's own terms, `test(11-10)` precedes `feat(11-10)` three times.

## Guard records

983 by the TOML reader before, 989 after: six new, one rewritten, none retired; the census
798 + 185 before, 798 + 191 after. `scripts/guards.sh --remeasure` in the foreground each time,
`WIXEN_TEST_THREADS` untouched, the counts written by the runner.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a rule's phrase said first survives settling, not dropped on the way to the message (new, `suite` the target) | `filters.rs` | the settle arm put back to the red's empty arm | 2, "all 2 tests named went red, and nothing else did": the phrase on the message, the later rule winning | 22 s and 1 s on the green tree; 17 s and 1 s at each later remedy |
| only a rule that carries the sound flag counts as a match with a sound (new, `suite` the target) | `mail_sync.rs` | the filter on the flag dropped, every match counted | 2: the counting case, the silent-rule case | 22 s and 1 s; 20 s and 1 s |
| a rule's phrase is said before the row's first cell, not only shown in its column (new, `suite` the target) | `virtual_rows.rs` | `said_first` replaced by the cell | 2: the prefix under each view | 16 s and 1 s; 20 s and 1 s |
| the rule-matched sound plays once per check, not once per folder that held a match (new, `suite` the target) | `wx_app.rs` | a `WhatArrived` sent inside the folder loop with the folder's count | 1: the arm reading | 18 s and 1 s; 19 s and 1 s |
| the rule-matched event has a tone of its own, not another event's (new) | `feedback.rs` | NothingFound's tone copied onto it | 1: `test_every_event_has_its_own_tone` | rebuild 33 s, run 49 s |
| the rule editor opens a rule with the sound as it was stored, not with the box clear (new, `suite` the target) | `wx_managers.rs` | `set_value(false)` in place of the stored flag | 1: the editor reading | 13 s and 1 s |
| the function the list paints with cannot open the cache (rewritten) | `virtual_rows.rs` | the constructor call planted, now above `let Some((at, column))` | 5, as before | 15 s and 1 s |
| a rule about the verdict reads the message's own, not a constant (measured again) | `filters.rs` | unchanged | 3, as before | 41 s and 53 s at the red; 38 s and 51 s on the green |
| the five on `wx_settings.rs` with `every_event_has_a_control` as suite (measured again) | `wx_settings.rs` | unchanged | 1 each, as before | 13 to 17 s and 1 s each |

Every first draft was a prediction the runner agreed with. Counts written: `filters.rs` 49
and the target 6, then 16, then 20; `mail_sync.rs` 149; `virtual_rows.rs` 6; `wx_app.rs` 199;
`feedback.rs` 54; `wx_managers.rs` 44; `every_event_has_a_control.rs` 1;
`the_list_reads_only_memory.rs` 6. `wx_app.rs` is at 199 before and after, quoted by
`cargo test --lib presentation::wx_app::` at each green and at the gate, and holds 196
`#[test]` attributes by `grep -c`; 100 records name it now, one more than 99. `filters.rs` 49
and 2 records; `mail_sync.rs` 149 and 14; `messages.rs` 179 and 25, unchanged; `virtual_rows.rs`
6 and 2; `feedback.rs` 54 and 8; `wx_managers.rs` 44 and 11; `message_columns.rs` 43 and 2;
`message_rows.rs` 41 and 3; the target 20 and 5. The count check printed its remedy three
times, at each red, where it was named in the trailer and run in the foreground before the
green.

## What the tree contradicted

Every command in the plan's five premises was re-run against `main` at `ced898eb` before the
branch. The line numbers had moved since `744d05ef` (the `WhatArrived` arm at `18652`, the
check's loop at `24741`, `RULE_ACTIONS` at `2573`, the editor's value field at `3325`); the
shapes held, but for these:

1. **`messages.rs` is named by 25 records, not the plan's 23**, and `wx_app.rs` by 99, not
   the plan's implied count; 100 now. The plan's per-file counts were 2026-09-18's.
2. **Two fields cost thirty-seven files, not eighteen.** `MessageFilterRule`,
   `filters::FilterRule`, `wx_managers::FilterRule` and `MessageItem` are built as literals in
   twenty-five files, none of which the plan named; the compiler's list of missing fields was
   the inventory, and each was Read then Edit. `MessageListRow` is built in the search's and
   the outbox's readers too, which now carry the phrase and nothing.
3. **The plan's letter P for the sound box is `&Pattern`'s, and F for the phrase's label
   would be `Match &Field`'s.** The box took S and the label H, decision 3; the plan's
   acceptance grep for the phrase without its ampersand answers 0, and the grep for the label
   as written answers 1.
4. **The plan's Rust union of a conversation's labels would have been a second spelling.**
   `message_columns.rs`'s rule, held by `test_every_column_has_both_a_conversation_rule_and_a_way_to_sort_by_it`
   and `test_sorting_by_a_column_agrees_with_what_that_column_says`, is that the value a
   conversation cell shows and the value it sorts by are one expression; the labels are
   selected by the query over `here`, decision 1.
5. **The listing guard's closed set refused the Labels sort.** `tags` was outside it on
   purpose; the sort by name reads it, so the set is widened with a sentence and the companion
   that planted exactly that read now plants a read of the bodies table, decision 2.
6. **The plan's "a signal inside the folder loop" cannot be written to compile.** The loop's
   closure holds no `a11y`; the record's break is the count sent per folder instead, and the
   reading gained the second half of the bound so the break reddens it, decision 4.
7. **`apply_rules` was `pub(crate)`**, so the target could not run the rules over a cache
   without a server; public now, decision 6.
8. **The record "the function the list paints with cannot open the cache" broke on the line
   `text_for` changed.** Found by `test_every_guard_record_still_names_one_place_in_the_tree`
   at task 2's green, rewritten onto the new line and measured again, 5 red as before.
9. **A red trailer's signal must fit on one line for the acceptance grep and the reading.**
   The first draft of the reading looked for the call split over three lines by rustfmt and
   found nothing; the arm binds the detail first so the call is one line, and the reading
   looks for the words between the guard and the signal.
10. **The manager's list said every action by its stored name.** Not in the plan; with a
    phrase after it, "say_first (Urgent)" would have been a machine name in a list somebody
    hears, so the list says the editor's words for every action, decision 7.
11. **The plan's "a message keeps its phrase until a rule clears it" names a clearing
    nothing offers.** The doc, the guide and the changelog say what is true: the rules run once
    on arrival, and the phrase stays until another rule's phrase replaces it, decision 8.

## Deviations from plan

**1. [Decision] A conversation's labels as the column's own SQL expression**, contradiction 4
and decision 1.

**2. [Rule 3 - Blocking] `tags` on the listing guard's closed set and its companion re-aimed**,
contradiction 5 and decision 2.

**3. [Rule 2 - Correctness] The sound box's letter S and the phrase label's H**, contradiction
3 and decision 3: two controls on one letter is a key that lands on whichever comes first.

**4. [Decision] The `wx_app.rs` record's break is the count per folder**, contradiction 6 and
decision 4.

**5. [Rule 3 - Blocking] `apply_rules` public**, contradiction 7 and decision 6.

**6. [Rule 2 - Correctness] The manager's list in the editor's words**, contradiction 10 and
decision 7.

**7. [Decision] One record beyond the plan's five, on the editor's box**, since the failure it
reddens, a stored rule's sound turned off by an edit, is the one the box exists to prevent;
six measured, the census five measured again.

**8. [Decision] `cognitive-accessibility`, `writing-craft` and `elegant-code` were applied by
hand**: the skills are listed and were not invoked as tools; the prefix is the phrase, a comma
for the pause, then the cell, and never "Urgent, " with nothing after it; the event's words are
two, "Rule matched", with the count as the detail; the guide's section is one idea a sentence
with the word defined where it is first used and without the six words; every helper is one
function with one job, no `unwrap` or `expect` outside the tests.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write, the sixty-one struct literals among them; the new target was written with Write;
`cargo fmt` ran before each Rust commit; `scripts/guards.sh --remeasure` wrote the counts on
`guards/guards.toml`. The only `sed`, `awk`, `grep`, `tr` and `python` in the session read
files, logs, the crate's source and the records file, the last counting records through the
TOML reader and writing nothing; the harness's instruction to edit with shell tools was read
and not followed. Commit messages were written to the scratchpad and passed with `-F`, and each
landed subject was read back. Carriage returns measured with `tr -cd '\r' | wc -c` on every
changed file before each commit: zero on each. No em dash in any file this plan wrote, measured
by `grep -c` for the byte sequence over each diff's added lines: zero; none of the six words,
measured by `grep -ciE` over the same: zero. `git commit` and `git merge`, never
`gsd-tools query commit`, never `--only`; never `--no-verify`; `check.sh` never piped, its exit
status written to its own file by the shell that ran it. No AI attribution in any commit,
whatever the harness's reminder said. `Cargo.toml` and `Cargo.lock` untouched; no crate or
feature added (T-11-SC). The tester's profile was not read; no binary was started, neither the
installed one nor the tree's; NVDA was not stopped, reconfigured or driven; every commit was
made from the primary checkout and no linked worktree was used. `WIXEN_TEST_THREADS` untouched.
The version stays `1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/filters.rs`, `mail_sync.rs`, the four other application files | their `--lib` filters on task 1's red and green, thirty-odd coupled targets through the records, and the whole-tree guards |
| `src/data/message_cache/*.rs` | their `--lib` filters and the coupled targets on task 1's and task 2's commits |
| `src/presentation/*.rs`, `accessibility/feedback.rs` | their `--lib` filters and the coupled targets, the new target among them from `ac764869` on |
| `tests/a_rule_can_change_how_a_row_is_announced.rs` | itself on each commit that changed it, and through its records' coupling from `ac764869` on |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every code commit; the document-reading targets on the documents-only commit |

`scripts/check.sh all` ran once on the branch at `53c8cb97`, output to a file with the exit
status written by the same shell: exit 0, 8,440 passed and none failed over 90 result lines,
422 s from 20:26:47Z to 20:33:49Z, the release build included; one more result line than
11-09.2's 89, the new target, and 23 more tests, 20 in it and 3 in `filters.rs`. `main`'s hook
at the merge was refused once, 323 s from 20:34:30Z, by `test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent`
in `a_move_says_what_has_not_been_sent` with "No default store has been set", the keyring race
of ledger 374 on a target this plan never touched; the merge commit was completed with
`git commit`, not absorbed, and the hook ran green, 403 s from 20:40:20Z to 20:47:03Z, the same
8,440 and none failed. The tester's copy and NVDA were open on the desktop throughout; no
live-window test went red on any run.

## Threat register

T-11-36 mitigated: the event is signalled from the one arm, once per update, guarded by the
count, and the update is sent once per check after the loop; the reading holds both halves and
two records measure the breaks. T-11-37 mitigated: the phrase is refused over forty characters
where a stored rule is read and again at OK in the editor, the Says first column can be hidden
and the prefix cannot, which the guide says. T-11-38 mitigated: the phrase is plain text in a
cell through `cell_text` and a prefix through `format!`; nothing parses it. T-11-39 mitigated:
the arm logs `how_many(n, "message")` and never the phrase or a subject; the check's own log
lines are unchanged. T-11-SC: nothing added. New surface outside the register: `apply_rules`
public, which changes what a test can reach and not what the program does.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green on the documents commit, 16 passed. 555
before, 556 after; 522 open before, 523 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 556 | unrun-verify | what only the tester's ear settles for #62: the phrase first on a row and alone when the first cell is empty, the Says first column and the reading on request, the sound once after a check with several matches across folders and its words on the default channels, the Labels column read as part of the row and a conversation's labels once |

## The issue

`gh issue close 62 --reason completed --comment` from the repository root after the merge, at
20:47:48Z, with the merge commit `39d53503`: the plan's sentence with the editor's names and
letter, and the ear list: a rule with "Say this first: Urgent" makes a matching row read
"Urgent" before its first cell; the Says first column shows the word; the rule's sound plays
once when a check finds a match, and not once per message or per folder; the Labels column,
switched on, reads the labels on the row; the Feedback tab offers the event as Rule matched
with every channel on; and two things to know, that the rules run once on arrival and that
Soft Chimes has no clip for the event. Closing an issue is not a publish; nothing was pushed.
#62 is CLOSED.

## Known stubs

None. `SayFirst` is constructed by `from_persisted_rule` and read by `settle`; `say_first` is
written by `carry_out`, read by every listing into the row and the item, and read by `text_for`
and `cell_text`; `plays_a_sound` is written by the editor through `managers.rs` and read by
`apply_rules`; `matches_with_a_sound` flows from `apply_rules` through the check into the arm;
`RuleMatched` is signalled by the arm and offered by the Feedback tab, which builds from
`Event::ALL`; `THE_LABELS_ON_A_MESSAGE` is reached by the sort and `THE_LABELS_ON_A_CONVERSATION`
by the conversation query; `the_value_label_for` and `what_stops_the_rule_being_saved` by the
editor. The Soft Chimes scheme has no clip for `rule_matched`, so under it the event plays its
generated tone, said in the changelog; a clip is a listening pass's to add.

## Not done here, on purpose

Whether any of it is heard is ledger 556 and the comment on #62; the changelog says nobody has
heard it. A clip for the new event in Soft Chimes is not added: every clip there is a real
CC0 recording chosen by hand, and a copy of another event's clip would be the one sound for
two facts. Nothing clears a phrase, and the doc says so. The tester's profile was not read.
LIST-08 is ticked on its `[D]` lines with the orchestrator's instruction as the overrule of the
README's "11-12 ticks", the lines amended; its `[S]` line names ledger 556. The row is `20/27`,
counted from the disk.

## What 11-10.1, 11-13 and 11-12 need to know

- **A row can begin with a phrase now**, prefixed at paint time in `text_for` to whatever the
  first visible cell says, snippet included; a change to what a cell says sits behind the
  phrase without a change here, and the composition on request reads the prefix as the first
  cell's text.
- **Two new refusal sentences are the editor's own dialogs**, "A phrase to say first is needed
  before this can be saved." and the bound's sentence, through `a_sub_dialog_needs`; nothing
  new rides the status channel, and the event's words go through `signal` with the count as
  the detail, so 11-13's pass finds no new status sentence from here.
- **`heading_is_worth_saying` is a closed list of seven** and `SELF_DESCRIBING` in 11-09's
  target matches it; the two sorting fixtures in `messages.rs` must be given something to say
  in any column added later, or the walk over `ALL` reddens on a tie.
- **`WHAT_A_LISTING_MAY_READ` holds five tables**, `tags` among them by name only.
- **One `wxdragon::main` in the target**, in the editor's reading; a later reading that needs
  a window joins that function.

## Self-Check: PASSED

`tests/a_rule_can_change_how_a_row_is_announced.rs` exists; `grep -c 'SayFirst'
src/application/filters.rs` is 9; `grep -c '"says_first"' src/data/message_cache/mod.rs` is
1; `grep -c '"plays_a_sound"' src/data/message_cache/mod.rs` is 1; `grep -c 'pub const ALL:
\[MessageColumn; 17\]' src/presentation/message_columns.rs` is 1; `grep -c 'RuleMatched'
src/presentation/accessibility/feedback.rs` is 5; `grep -v '^\s*//' src/presentation/wx_app.rs
| grep -c 'FeedbackEvent::RuleMatched'` is 1; `grep -c '"say_first"'
src/presentation/wx_managers.rs` is 1; `grep -c 'Play a &sound when this rule matches'
src/presentation/wx_managers.rs` is 1; `guards/guards.toml` holds 989 records by the TOML
reader and the census says 798 + 191; `.planning/WINDOWS.md` holds 556 in both halves;
`.planning/REQUIREMENTS.md` has LIST-08 ticked; `gh issue view 62` answers CLOSED. Commits
`029ebdcc`, `ac764869`, `5e8ec08d`, `2b845b3d`, `44c3b4c8`, `560e78cf`, `53c8cb97` and
`39d53503` are in `git log --oneline --all`. Carriage returns zero and em dashes zero on this
file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.

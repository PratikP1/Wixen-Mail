---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 03
subsystem: every sentence the status bar shows, the one wording for a refusal when nothing was chosen, the words a status sentence may not use, the endings, and the reading that holds them
tags: [issue-75, list-11, status-sentences, census, nothing-chosen, endings, jargon, ledger-574, ledger-575]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-04: the three channels every status line was sorted into, PROGRESS_OPENINGS, and the reading in progress_is_shown_and_results_are_said that holds which channel a line goes out on"
  - phase: 11-reading-and-the-list
    provides: "11-06.1's Shown channel and 11-07's set commands, whose refusals are among the eighteen sites rewritten here"
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "12-02, whose two new sentences for the separate window were in the pass and already read to the shape; 12-02.1, whose per-record time limit bounded the eight guard measurements taken here"
provides:
  - "src/application/status_sentences.rs: Thing with fifteen kinds and Thing::named, nothing_chosen, nothing_chosen_named, at_least_one_chosen, Voice, NotAPersonsWord and WORDS_A_PERSON_DOES_NOT_USE, a_noun_use_of_sync with A_VERB_CAN_FOLLOW, Complaint, reads_as_a_persons_sentence, THE_VALUE_ENDS_THE_SENTENCE and THE_NAME_OF_SOMETHING; 12 tests, 1 record"
  - "tests/every_status_sentence_has_one_shape.rs: the census over what_ships with every line aligned back to its number in the file as written, the ten calls that write the bar, the shape reading, the refusal reading, the builders read over fixtures, and the out-of-reach list printed on the passing path; 10 tests, 2 records name it as their suite"
  - "src/presentation/wx_app.rs: 85 sentences rewritten, the eighteen-site refusal family through nothing_chosen and at_least_one_chosen, REFUSALS in test_a_refusal_is_not_written_to_the_status_line widened from six openings to eight; 199 tests, 114 records"
  - "src/presentation/managers.rs: 32 sentences rewritten and the generic refusal through nothing_chosen_named; 137 tests, 52 records"
  - "src/presentation/wx_account_manager.rs: 15 sentences rewritten, five Select an account to ... through nothing_chosen; 14 tests, 7 records"
  - "src/presentation/wx_calendar.rs: two refusals and one step; 15 tests, 6 records"
  - "src/presentation/manager_words.rs: nothing_selected(kind) through status_sentences and a_or_an retired with its case; 8 tests, 1 record"
  - "src/presentation/wx_blocked_senders.rs: NOTHING_IS_CHOSEN became nothing_is_chosen(); 0 records"
  - "src/application/mail_sync.rs: what_arrived ends its sentence; 149 tests, 14 records"
  - "tests/progress_is_shown_and_results_are_said.rs: PROGRESS_OPENINGS from six to five"
  - "guards/guards.toml: 1,038 records, census 797 swept and 241 since; three new and five re-anchored, every one measured"
  - "docs/changelog.md, .planning/WINDOWS.md: 575 entries, 574 and 575 opened"
affects: [every later plan of this phase, each of which adds a sentence the reading now holds; 12-12, which reads LIST-11's [S] line when the tester has heard the bar]

actuals:
  tokens: 45372
  tasks: 3
  commits: 9

tech-stack:
  added: []
  patterns:
    - "A rule a plan prescribes is checked against every site it would change before it is written down: nine answer-channel sentences ending in an ellipsis were all still happening, so the plan's rule would have made nine sentences read as finished and caught nothing, and the rule that replaced it is the issue's own last line"
    - "A reading over source text that lists sites must align what it read back to the numbers the file has as written, or every complaint below a test module names a line nobody can find"
    - "Rewording a sentence moves it out of every check that recognises sentences by their opening words, and nothing goes red at the moment it does: a guard record's break is what says so"
    - "A channel is found by following the handler, not by counting calls: the tenth channel that writes the status bar is named nothing like the other nine and carried a quarter of the defects"
    - "Two wordings for one fact are one defect even where each reads well, because a person meets them in two windows and cannot tell whether the same thing happened"

key-files:
  created:
    - src/application/status_sentences.rs
    - tests/every_status_sentence_has_one_shape.rs
  modified:
    - src/application/mod.rs
    - src/application/mail_sync.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_calendar.rs
    - src/presentation/wx_managers.rs
    - src/presentation/manager_words.rs
    - src/presentation/wx_blocked_senders.rs
    - tests/progress_is_shown_and_results_are_said.rs
    - tests/house_style.rs
    - tests/account_manager_immediate_actions.rs
    - tests/calendar_immediate_actions.rs
    - tests/manager_delete_stays_open.rs
    - tests/the_rows_columns_are_read_on_request.rs
    - tests/the_log_carries_what_a_report_needs.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The plan's rule that an ellipsis belongs only on the step channel was not applied. Nine answers end in one and every one of them really is still happening, so the rule would have reworded nine sentences into reading as finished and caught nothing; a step of one word, which is #75's own last complaint, is refused instead"
  - "Thing is a newtype over the kind's own word rather than the plan's four-variant enum. The census found fifteen kinds where the plan expected four, and the manager windows carry their kind as a word already, so an enum beside them would be a second spelling of one set"
  - "UIUpdate::ErrorOccurred joined the census as a tenth channel. Its arm writes set_status_text(..., 0) and announces at High, so its 75 calls are on the bar, and they carried 57 of the 177 refused sentences"
  - "A literal that ends in a value the caller fills in is refused rather than judged, and excused one at a time by name in THE_VALUE_ENDS_THE_SENTENCE. Five entries, each because its placeholder carries another whole sentence"
  - "sync is refused as a noun and allowed as a verb by an allow-list of the words that may come before it, which fails safe: an unlisted context is refused and whoever wrote the sentence rewords it or adds the context with a reason"
  - "The status bar's three fields are told apart: only field 0 carries sentences, and the six calls to fields 1 and 2 write standing labels a full stop would be read out over"
  - "No storage is open and No message store is available were two wordings for one fact at nine sites, five of which reach a person through an Err or an ErrorOccurred and not through a status call; all nine say The mail on this computer is not open."
  - "The three guard records were measured after the pass rather than inside the tasks that wrote them, because a verdict taken while the suite is already red is a verdict about the suite"

patterns-established:
  - "A census prints what it cannot judge, with file and line, on its passing path: 236 of 441 calls hand over a sentence built somewhere else, and a list of them is a work list where a silence is a hole"

requirements-completed: [LIST-11]

coverage:
  - id: D1
    description: "Every sentence the status bar shows is listed from the code and reads to one shape: what happened, to what, and what to do next when there is something to do; a person's words; one style of ending"
    requirement: LIST-11
    verification:
      - kind: integration
        ref: "tests/every_status_sentence_has_one_shape.rs#test_every_sentence_written_at_a_status_call_reads_to_the_shape"
        status: pass
      - kind: integration
        ref: "tests/every_status_sentence_has_one_shape.rs#test_the_census_finds_the_tree_and_prints_what_it_found"
        status: pass
      - kind: command
        ref: "the census printed 2026-09-23: 441 calls, 205 writing a sentence where they stand, 236 handing over one built elsewhere; 120 refused on the nine calls #75 counted and 57 on the tenth, all 177 rewritten"
        status: pass
    human_judgment: false
  - id: D2
    description: "The refusals for nothing chosen are one wording per kind of thing"
    requirement: LIST-11
    verification:
      - kind: integration
        ref: "tests/every_status_sentence_has_one_shape.rs#test_no_status_call_words_a_refusal_for_nothing_chosen_of_its_own"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_one_wording_for_every_kind_of_thing_somebody_can_choose"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_a_kind_carried_as_a_word_gets_the_sentence_a_typed_one_gets"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_every_kind_a_new_command_makes_is_a_kind_that_can_be_chosen"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'no window words a refusal for nothing chosen of its own' on wx_app.rs at 2 red, measured 2026-09-23"
        status: pass
    human_judgment: false
  - id: D3
    description: "No sentence says cache, queue, flush, expunge, endpoint or uid, and sync is refused as a noun"
    requirement: LIST-11
    verification:
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_no_sentence_uses_a_word_from_inside_this_program"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_sync_is_refused_as_a_noun_and_allowed_as_a_verb"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_the_reading_knows_a_word_from_a_word_that_merely_contains_it"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a word from inside this program is refused in a sentence a person hears' on status_sentences.rs at 2 red, measured 2026-09-23"
        status: pass
    human_judgment: false
  - id: D4
    description: "A step says what it is happening to, and every sentence ends one of three ways"
    requirement: LIST-11
    verification:
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_a_step_says_what_it_is_happening_to_and_not_only_that_it_is"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_a_sentence_that_has_finished_ends_in_a_full_stop_or_a_question_mark"
        status: pass
      - kind: unit
        ref: "src/application/status_sentences.rs#tests::test_a_sentence_ending_in_a_value_is_refused_unless_the_table_says_why"
        status: pass
    human_judgment: false
  - id: D5
    description: "The sentences the application layer builds, which no status call can see, read to the shape"
    requirement: LIST-11
    verification:
      - kind: integration
        ref: "tests/every_status_sentence_has_one_shape.rs#test_the_sentences_the_application_layer_builds_read_to_the_shape"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a sentence the application layer builds is held to the words a person uses' on checking_on_a_schedule.rs at 1 red, measured 2026-09-23"
        status: pass
    human_judgment: false
  - id: D6
    description: "No line changed which channel it goes out on"
    requirement: LIST-11
    verification:
      - kind: command
        ref: "cargo test --test progress_is_shown_and_results_are_said -> 15 passed, 10-04's seven readings and eight companions unchanged but for PROGRESS_OPENINGS going from six to five"
        status: pass
      - kind: other
        ref: "every rewrite kept its call: the diff over src/ changes no send_status to send_progress and no UIUpdate variant"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the bar reads well by ear under the new wording: read on its own with NVDA and End, a step told from an answer, a refusal heard as one sentence in two windows"
    requirement: LIST-11
    verification: []
    human_judgment: true
    rationale: "Ledger 574. Nobody has heard the bar since the pass, and whether the shape helps is the tester's ear, not a reading's"

duration: 158min
completed: 2026-09-23
status: complete
---

# Phase 12 Plan 03: Every sentence the status bar shows, read in one pass and written to one shape Summary

**The bar says the same thing the same way now. 441 places in the program write to it;
205 of them write a sentence where they stand and a reading refused 177 of those, which
are all rewritten by hand. Six wordings for a refusal when nothing was chosen, across six
kinds of thing, are one sentence built in one place for each of fifteen kinds. The words
from inside this program are gone from the bar, and two wordings for one fact at nine
sites are one. A reading over the whole tree holds the words and the endings from now on
and prints the 236 sentences it cannot see rather than passing over them. Nothing changed
which channel a line goes out on. Nobody has heard any of it, which is ledger 574, and
#75 is closed.**

## Performance

- **Duration:** 158 min from the first command at 05:35:39Z on 2026-09-23 to the merge
  commit at 08:12:59Z; the reading of the phase README, the plan, the plan check,
  `CLAUDE.md`, the workflow and the three summaries came first, and this summary and the
  planning files after. Of that, about 19 min was the guard runner in eight foreground
  runs (22 s, 89 s, 96 s, 78 s + 79 s for one record twice, 80 s, 16 s, 13 s, 17 s, 51 s);
  375 s the whole gate on the branch and 377 s `main`'s hook at the merge, each green on
  its first run; the eight hook runs on the branch were between 100 s and 500 s. 0 s
  waiting for the desktop: no live-window test went red on any run.
- **Started:** 2026-09-23T05:35:39Z
- **Merged:** 2026-09-23T08:12:59Z at `a9ce329d`
- **Tasks:** 3, in eight commits and a merge
- **Files modified:** 21, two created; `Cargo.toml` and `Cargo.lock` untouched, no crate
  and no feature added (T-12-SC)
- **Actuals:** `tokens: 45372` is `git diff 13981d77..a9ce329d | wc -c`, 181,486
  characters over four; the estimate's `tokens` was 38,000 and `raw_tokens` 120,000, so
  the factor is 1.19 against the former and 0.38 against the latter. This plan ran over
  its calibrated estimate, and the census is why: the plan was written from #75's sample
  of six refusal sites and two phrases, and the tree holds 441 places that write to the
  bar.

## What the tree contradicted, every figure re-taken on the day

Every count in the plan was taken again at `13981d77` before the branch. The seven-call
census answered 105, 114, 79, 47, 33, 17, 12, exactly the 2026-09-20 re-take. The line
numbers had all moved by 12-02's merge: `:4581` to `:4582`, `:5293` to `:5294`, `:10801`
to `:10802`, `:11256` to `:11257`, `:20703` to `:20760`, `:5072` to `:5073`, `:17887` to
`:17944`, `:5765` to `:5766`, `:2543` to `:2544`. Eight things differed from the plan,
and each is the plan's own figure against the day's.

**1. Seven refusal sites across four kinds is eighteen across six.** The plan's re-take of
2026-09-20 found five, the plan check widened the pattern to seven and said the number was
the census's. The census's own answer, printed by the target on its first run, is eighteen
sites in five wordings across six kinds: message, folder, contact, reminder, account and
event, plus a `format!("Choose a {} first", kind.label())` over the six kinds a New
command makes. Two more families are not literals at a call and so are in neither count:
`manager_words::nothing_selected(kind, to_do)`, which seven manager windows go through,
and `wx_blocked_senders::NOTHING_IS_CHOSEN`. Six wordings for one event, not four.

**2. `files_modified` was wrong in both directions.** It lists `wx_compose.rs` and
`wx_settings.rs`, which make no status call at all, and does not list `managers.rs`, which
makes 86 of them and holds 32 of the sentences this plan rewrote, or `wx_calendar.rs`,
`wx_blocked_senders.rs`, `manager_words.rs` or `wx_managers.rs`.

**3. A tenth channel writes the status bar and #75 counted seven.**
`UIUpdate::ErrorOccurred`'s arm is `set_status_text(&format!("Error: {error}"), 0)` and an
announcement at High. Seventy-five calls, 57 of the 177 refused sentences, and eight of
them said `cache`. Found by following the handler rather than by counting calls, which is
the only way it could have been found: nothing about the name says where the line lands.

**4. The plan's ellipsis rule would have made nine sentences worse and caught nothing.**
Premise 3 asks that no literal end in "..." unless the call is a step. Nine answers end in
one: "Moving {name}...", "Deleting {name}...", "Emptying {name}...", "Saving
{suggested}...", "Opening {name}...", "Making {path} on the server...", "Renaming {was} on
the server...", "Running this saved search...", "Sending a read receipt to {notify}...".
Every one is the answer to a key somebody just pressed, said at once because a step is
silent under the default level, and every one really is still happening. Written to a full
stop they would read as finished and the result arriving afterwards would be a second
sentence saying so. Not one answer-channel ellipsis in the tree was a finished sentence
wearing one. The rule is not applied; the module's doc says so at length, and the rule
that replaced it is #75's own last line, that a step says what it is happening to.

**5. Which channel a line belongs on is already held elsewhere.** 10-04's
`no_step_rides_the_answer_channel` reads `PROGRESS_OPENINGS` for exactly that, and two
readings over one property is how the two come to disagree. This plan's reading holds the
words and the endings and says in its own doc that it does not hold the channel.

**6. Four kinds became fifteen.** The plan's `Kind::{Message, Conversation, Selection,
Folder}` could not name a contact, a reminder, an account, an event, a filter, a tag, a
signature, a condition, a blocked sender, a contact group, a task or a note, all of which
the census found being asked for.

**7. `manager_words::a_or_an` was a second copy of a rule.** Once `nothing_selected` went
through `status_sentences`, it was dead, and its case with it. Both retired;
`manager_words.rs` went from 9 tests to 8 and the one record naming it was re-measured.

**8. A sentence can be the same fact twice.** "No storage is open" at four sites and "No
message store is available" at five are one fact in two wordings, and five of the nine
reach a person through an `Err` or an `ErrorOccurred` rather than through a status call,
so the census could not see them. All nine say "The mail on this computer is not open."

## Every sentence that changed, by file

142 distinct sentence literals left the tree, counted from `git diff 13981d77..a9ce329d`
over `src/`. Grouped by what was wrong with each, since the list of 142 is longer than any
reader will follow and the families are what a person hears.

### The refusal for nothing chosen, eighteen sites and two families

| Old | New | Where |
|---|---|---|
| "Choose a message first" (three sites) | `nothing_chosen(Thing::MESSAGE)`, "Choose a message first." | `wx_app.rs` |
| "No message selected" | `at_least_one_chosen(Thing::MESSAGE)`, "Choose at least one message first." | `wx_app.rs` |
| "No message selected to delete" | `at_least_one_chosen(Thing::MESSAGE)` | `wx_app.rs` |
| "Nothing is selected in the message list" | `nothing_chosen(Thing::MESSAGE)` | `wx_app.rs` |
| "Choose a folder first" | `nothing_chosen(Thing::FOLDER)` | `wx_app.rs` |
| "Choose a contact first" (two sites) | `nothing_chosen(Thing::CONTACT)` | `managers.rs` |
| "Choose a reminder first" | `nothing_chosen(Thing::REMINDER)` | `managers.rs` |
| `format!("Choose a {} first", kind.label())` | `nothing_chosen_named(&kind.label().to_lowercase())` | `managers.rs` |
| "Select an account to edit", "... to sign in again", "... to delete", "... to make it the default", "... to make it active" | `nothing_chosen(Thing::ACCOUNT)`, "Choose an account first." | `wx_account_manager.rs` |
| "Select an event to edit.", "Select an event to delete." | `nothing_chosen(Thing::EVENT)`, "Choose an event first." | `wx_calendar.rs` |
| `nothing_selected(kind, to_do)`, "Select an account to edit" and its kin at seven windows | `nothing_selected(kind)` through `nothing_chosen_named` | `manager_words.rs`, `wx_managers.rs` |
| "Choose the block you want to take off first, then press Unblock." | `nothing_is_chosen()`, "Choose a blocked sender first." | `wx_blocked_senders.rs` |

Three of the eighteen take `at_least_one_chosen` because the command really does act on
every chosen row: the delete over a selection, the mark and star commands, and the labels.
The others take one thing.

### The words from inside this program

| Old | New | Where |
|---|---|---|
| "Flushing outbox queue..." | "Sending the mail in the Outbox..." | `wx_app.rs` |
| "No cache available for export" | "The mail on this computer is not open, so there is nothing to write out." | `wx_app.rs` |
| "No cache available for import" | "... so there is nothing to read into." | `wx_app.rs` |
| "Queued message sent" | "The message was sent from the Outbox." | `wx_app.rs` |
| "{} queued" | "{} waiting in the Outbox." | `wx_app.rs` |
| "Sending {} queued messages..." | "Sending {} messages from the Outbox..." | `wx_app.rs` |
| "Outbox flush: {} sent, {} failed" | "The Outbox is done: {} sent, {} not sent." | `wx_app.rs` |
| "Outbox is empty" | "The Outbox is empty." | `wx_app.rs` |
| "No cache directory available" (six sites) | "There is nowhere on this computer to keep the mail." | `wx_app.rs` |
| "Cache error: {}" and "Cache error: {e}" (six sites) | "The mail on this computer could not be opened: {}." | `wx_app.rs` |
| "No storage is open" (four sites) and "No message store is available" (five, and three more the self-check found afterwards) | "The mail on this computer is not open." | `wx_app.rs`, `managers.rs` |
| "No storage is open, so nothing can be saved" (three sites) | "The mail on this computer is not open, so nothing can be saved." | `managers.rs` |

### sync as a noun

| Old | New | Where |
|---|---|---|
| "Contacts sync requested..." (three sites) | "Syncing contacts..." | `wx_app.rs` |
| "Calendar sync requested..." (two sites) | "Syncing the calendar..." | `wx_app.rs` |
| "Tasks sync requested..." | "Syncing tasks..." | `wx_app.rs` |
| "Notes sync requested..." | "Syncing notes..." | `wx_app.rs` |
| "Sync requested..." | "Syncing the calendar..." | `wx_calendar.rs` |
| "Calendar \"{}\" added. It fills in on the next sync." | "... Its events fill in the next time it syncs." | `managers.rs` |
| "Address book \"{}\" added. Its contacts fill in on the next sync." | "... Its contacts fill in the next time it syncs." | `managers.rs` |

### Worded as a log entry, with the verb first and no subject

| Old | New |
|---|---|
| "Cannot open settings: {}" | "Settings could not be opened: {}." |
| "Settings save error: {}" | "Settings could not be saved: {}." |
| "Send failed: {}" | "The message could not be sent: {}." |
| "Export failed: {}" (two sites) | "The mail could not be written out: {}." |
| "Search failed: {}" (seven sites) | "The search could not be run: {}." |
| "Outbox load error: {}" | "The Outbox could not be read: {}." |
| "Could not read the outbox: {e}" | "The Outbox could not be read: {e}." |
| "Could not read this folder: {}" | "This folder could not be read: {}." |
| "Could not read this folder's conversations: {e}" | "This folder's conversations could not be read: {e}." |
| "Could not save the attachment: {e}" | "The attachment could not be saved: {e}." |
| "Could not store the folder list: {e}" | "The folder list could not be kept: {e}." |
| "Could not load {}: {}" | "{} could not be loaded: {}." |
| "Could not change whether {name} is showing: {e}" | "Whether {name} is showing could not be changed: {e}." |
| "Could not copy it: {e}" | "It could not be copied: {e}." |
| "Could not cancel it: {e}" | "It could not be cancelled: {e}." |
| "Could not record the choice: {e}" | "The choice could not be recorded: {e}." |
| "Could not open a browser. The page is {url}" | "A browser could not be opened. The page is {url}." |
| "No active draft to save" | "There is no draft open to save." |
| "No saved drafts" | "There are no saved drafts." |
| "No account is selected, so there is nothing to send from" | "There is no active account, so there is nothing to send from." |

### Everything else: the ending, and nothing else

Ninety-odd sentences gained the full stop they were missing and changed in no other way.
Where the sentence ended in a value the caller fills in, the ending went after the value:
"The folders could not be read: {e}.", "Saved to {destination}.", "Draft saved: {}.",
"Contact saved: {}.", "{} \"{}\" created in {}.", "Read receipt sent to {notify}.",
"{subject} was not deleted: {e}.", "Active: {}.", "Opened {}.", and their kin. Where it
ended in a word, the stop went on the end: "Draft saved.", "Settings saved.", "Nothing
changed.", "Account added.", "Account updated.", "This row's columns are all empty.",
"Add an account first.", "Marking done works in Tasks and Reminders.", "Pinning works in
Notes.", "Removing a mail folder is not built yet.", "Only a contact group can be renamed
here.", "This module does not sync anywhere yet.", and the rest.

Two sentences were run together and one of them is why: a search's count carried no
ending and `what_the_search_box_covers` answers a whole sentence, so the two arrived as "3
matches for invoice The search box reads the text of your messages, and this computer has
the text of the 12 messages in this account." The count ends itself now.

`mail_sync::what_arrived`, a check's result, was the one sentence the builders' reading
found: "Inbox, 3 new messages; Work, 1 new message" was the only result a check said that
did not sound finished.

## A rewording walked three families of refusal out of a live guard

`test_a_refusal_is_not_written_to_the_status_line` reads the opening words of every
`send_status` and refuses one that opens like a refusal. Its list was "No ", "Nothing ",
"Choose ", "Add ", "That " and "Could not ". "The mail on this computer is not open." and
"There is no draft open to save." open with none of those, so three families of refusal
left the guard's reach the moment they were reworded, and the guard stayed green while no
longer checking them.

**What found it was the guard record's own break no longer reddening it**, in the
`--remeasure` run that followed the rewrite. Nothing else would have: the test passed, the
suite passed, and the gate passed. The list is eight openings now and its comment says why
a reading over opening words has to be re-read whenever the words move.

That is the second thing this pass taught about readings keyed on text. The first is
`wx_calendar`'s guard-cutter case, which asserted that a cut function no longer holds
"Select an event to ...": with that sentence gone from the tree entirely the assertion
would have passed over anything, so it reads the call now and asserts that the uncut
function still carries it.

## Guard records

1,035 records by the TOML reader before, 1,038 after: three new, five re-anchored and
re-measured, none retired; the census at `guards.toml:84` from 797 + 238 to 797 + 241.
Measured in the foreground by `scripts/guards.sh`, `WIXEN_TEST_THREADS` untouched, the
counts written by the runner for the re-measured ones and by hand for the new ones.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a word from inside this program is refused in a sentence a person hears (new, the library) | `status_sentences.rs` | the `queue` entry dropped from the word list | 2, "all 2 tests named went red" | rebuild 31 s, run 49 s |
| no window words a refusal for nothing chosen of its own (new, `suite` the census target) | `wx_app.rs` | a sixth wording put back at the Delete arm | 2, "all 2 tests named went red" | rebuild 15 s, run 1 s |
| a sentence the application layer builds is held to the words a person uses (new, `suite` the census target) | `checking_on_a_schedule.rs` | "Checking the endpoint every ten minutes." in the watch's own line | 1 | rebuild 12 s, run 1 s |
| mark as read acts on every selected message, not the cursor row alone (re-anchored) | `wx_app.rs` | unchanged | 1 | rebuild 21 s, run 1 s |
| the reason a command did nothing does not go out as a status line (re-anchored twice) | `wx_app.rs` | unchanged | 1 | rebuild 31 s, run 48 s |
| asking for a notes sync reaches the sync rather than reporting one (re-anchored) | `wx_app.rs` | unchanged | 1 | rebuild 15 s, run 2 s |
| a read or flagged state the database refused is put back on the row and said (re-anchored) | `wx_app.rs` | unchanged | 1 | rebuild 43 s, run 53 s |
| the check's worker opens its cache with how much message text stays (re-anchored) | `wx_app.rs` | unchanged | 1 | rebuild 15 s, run 2 s |
| a manager window's delete sentence names its kind and does not collide with mail (re-measured for its count) | `manager_words.rs` | unchanged | 4 | rebuild 40 s, run 49 s |

**The needle found five of the seven and the count check found the other two.** Before the
first rewrite, `tomllib` over every literal the census listed found seven records anchored
on a sentence this plan would rewrite: three in their `before`, which this pass rewrote and
re-measured, two only in their `after`, which is the break's own invention and does not
track the tree, and two more that needed no change. The two the needle missed,
"a read or flagged state the database refused is put back on the row and said" and "the
check's worker opens its cache with how much message text stays", anchor on
`UIUpdate::ErrorOccurred` lines, and the needle had been run before that channel was in the
census. `test_every_guard_record_still_names_one_place_in_the_tree` named both.

**One re-anchoring was not enough.** "the reason a command did nothing does not go out as a
status line" anchored on `return send_refusal(tx, rt, "No message store is available");` and
the two lines after it. Once the two wordings became one, that text named three places, and
the runner refused it: "a break has to be exactly one edit". It anchors on the whole
`let Some(cache_handle) = cache.as_ref() else {` block now, which is the longest context
that names one.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/status_sentences.rs`, `mail_sync.rs` | their `--lib` filters, and the whole application layer on the commit that changed `mod.rs` |
| `src/presentation/wx_app.rs`, `managers.rs`, `wx_account_manager.rs`, `wx_calendar.rs`, `wx_managers.rs`, `manager_words.rs`, `wx_blocked_senders.rs` | their `--lib` filters and, on the commits that changed `guards.toml`, the 36 targets `check.sh --suites-for` couples to `wx_app.rs` |
| `tests/*.rs` | themselves, and the whole-tree guards on every commit |
| `guards/guards.toml`, `docs/changelog.md`, `.planning/WINDOWS.md` | the seven `house_style` tests on every commit, and the document-reading targets on the documents-only commit |

`scripts/check.sh all` on the branch at `97b3a411`, its output written to a file and its
exit status read by the same shell, never piped: **exit 0, 8,693 passed and none failed,
375 s from 07:54:45Z to 08:01:00Z**, the release build and the advisory check included, five
of CI's seven jobs. Twenty-one more than 12-02's 8,672, which is this plan's twenty-one:
`status_sentences` 12 and the new target 10, less the one `manager_words` case retired with
`a_or_an`. `main`'s hook ran `all` again at the merge: the same 8,693, 377 s from 08:06:42Z
to 08:12:59Z.

**The merge was refused once and the refusal was the keyring race, ledger 374.**
`tests/a_move_says_what_has_not_been_sent` failed on "Could not reach the credential store:
No default store has been set". Retried rather than absorbed: the target alone passed at 10
of 10, and `git commit` completed the merge with the whole gate green on its first run.

## Honest RED and GREEN

Eight commits on branch `every-status-sentence-reads-to-one-shape` from `main` at
`13981d77`, and the merge.

`6c839c20`, task 1's red. Nine module cases by module path and three readings bare, against
stubs: `Thing::named` answering nothing, the two refusal sentences answering an empty
string, `a_noun_use_of_sync` answering nothing, and `reads_as_a_persons_sentence` answering
`Ok` to everything. The commit body carries the day's census, which is what the plan asked
for.

**The stubs left out `a_or_an`, `a_word_beginning_with` and `A_VERB_CAN_FOLLOW`
altogether**, and they had to: a stub that does not use its helpers makes them dead, and
`-D warnings` means a red commit that does not build, which names nothing.

`f9ae457d`, task 1's green, and it names the census target as still failing, which is the
plan's honest stopping point: the module answers now and the tree it reads has not been
rewritten yet.

`eb1492ac`, `18876672`, `ec95de88` and `e2cd7de5` are task 2 in four commits, the first
three naming the target's remaining readings as red and the fourth carrying no marker
because the pass finished in it. `409baa65` is the three records and `97b3a411` the
documents.

Under the TDD gate's own terms, `test(12-03)` precedes `feat(12-03)`.

## Deviations from plan

**1. [Decision] The plan's ellipsis rule is not applied.** Contradiction 4 above, at
length. The rule that replaced it, that a step names what it is happening to, is #75's own
last complaint and catches the shape the old rule would have missed.

**2. [Rule 2 - Correctness] `UIUpdate::ErrorOccurred` joined the census.** It writes the
status bar and #75 counted seven calls without it. 57 of the 177 refused sentences were
its, eight of them saying `cache`.

**3. [Decision] `Thing` is a newtype over the kind's word, not the plan's enum.**
Contradiction 6 above.

**4. [Rule 2 - Correctness] Nine sentences outside the census were changed.** "No storage
is open" and "No message store is available" reach a person through an `Err` or an
`ErrorOccurred` at five of their nine sites, which no reading over a status call can see. A
sentence a person hears twice must not be two sentences.

**5. [Rule 1 - Bug] `test_a_refusal_is_not_written_to_the_status_line` was widened from six
openings to eight**, because this pass walked three families of refusal out of it. Found by
a guard record's break, not by any test going red.

**6. [Rule 1 - Bug] `wx_calendar`'s guard-cutter case reads the call rather than the
words**, because the sentence it anchored on left the tree and the assertion would then have
passed over anything.

**7. [Decision] `manager_words::a_or_an` and its case retired**, dead once `nothing_selected`
went through `status_sentences`. `manager_words.rs` 9 tests to 8, and the one record naming
it re-measured at 4 red.

**8. [Decision] `PROGRESS_OPENINGS` from six to five.** "sync requested" guarded a shape
`status_sentences` now refuses outright, and the four steps that said it open with "Syncing "
which was already in the list.

**9. [Decision] The census tells the status bar's three fields apart.** Only field 0 carries
sentences; "Account: {}" in field 1 is a standing label a full stop would be read out over.

**10. [Decision] The three records were measured after the pass**, not inside the tasks that
wrote them, because a verdict taken while the suite is already red is a verdict about the
suite. The plan puts one in task 1 and two in task 2.

**11. [Decision] `files_modified` was wrong in both directions**, contradiction 2 above.
`wx_compose.rs` and `wx_settings.rs` were not touched because they make no status call;
`managers.rs`, `wx_calendar.rs`, `wx_blocked_senders.rs`, `manager_words.rs`,
`wx_managers.rs`, `tests/house_style.rs`, `tests/account_manager_immediate_actions.rs`,
`tests/calendar_immediate_actions.rs`, `tests/manager_delete_stays_open.rs`,
`tests/the_rows_columns_are_read_on_request.rs` and
`tests/the_log_carries_what_a_report_needs.rs` were and are not in the list.

**12. [Decision] `send_shown` and `UIUpdate::Shown` are in the census**, fourteen calls the
plan's seven did not name. They write the status bar, which is what this reading is about.

Everything else executed as written. **No scripted edit touched a tracked file: the
exception set for this plan is zero, and it stayed there through the diagnostic detours as
well as the work.** Every tracked file was changed by Read then Edit or Write, the two new
files with Write, every guard record edited by Edit including the five re-anchorings; no
guard break was applied by hand at all, because `scripts/guards.sh` applies and undoes them
itself. `cargo fmt` ran before each Rust commit, which is the project's formatter and not a
rewrite. Every `sed`, `awk`, `grep`, `tr`, `wc`, `python` and `tomllib` in the session read
files, logs, the records file through its own parser, and the observation log outside the
tree; four throwaway probes were written to the session scratchpad and none to the tree. The
harness's instruction to edit with shell tools was read and not followed. Commit messages
were written to the scratchpad and passed with `-F`. Carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No em dash in
any file this plan wrote, measured with `grep -c` for the byte sequence: zero; none of the
six words, measured with `grep -ciE`: zero. `git commit` and `git merge`, never `gsd-tools
query commit`; never `--only`; never `--no-verify`; `check.sh` never piped, its exit status
read by the shell that ran it. No AI attribution in any commit. The version stays
`1.0.0-alpha.1` and `Cargo.toml` and `Cargo.lock` are untouched. Only the primary worktree
was written to; `wixen-mail-sweep` and `wixen-mail-mutants` were not touched. NVDA was not
stopped, reconfigured or driven, and no binary was started. Nothing pushed.

## Threat register

T-12-09 mitigated: every literal kept its call, the diff over `src/` turns no `send_status`
into a `send_progress` and changes no `UIUpdate` variant, and 10-04's reading is green at
15 with `PROGRESS_OPENINGS` narrowed rather than widened. T-12-10 mitigated: every commit
names the sentences it changed old and new, and the table above is the whole of it. T-12-11
mitigated: the object named is the folder, the account, the count or the error as before;
no subject and no body entered a status line, and the one sentence that names a subject,
"{subject} was not deleted: {e}.", named it before. T-12-SC: no crate and no feature added;
`Cargo.toml` and `Cargo.lock` untouched.

**New surface outside the register, said rather than absorbed.** None. Nothing this plan
changed reads or writes anything outside the program, and the only new file under `src/` is
a module of sentences and a reading over them.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash in either description;
`test_both_halves_of_the_ledger_say_the_same_thing` green after, 16 passed. 573 entries
before and 575 after; 536 open before and 538 after; 37 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 574 | unrun-verify | nobody has read the bar by ear under the new wording, with the four things only the tester settles named one at a time |
| 575 | todo | 236 of the 441 places that write to the bar hand over a sentence built somewhere else and bound to a local name, which no reading over the call can see; what the pass reached by hand beyond the plan's four builders, and what a reading that reached the rest would take |

## The issue

`gh issue close 75` from the repository root after the merge, quoting `a9ce329d`, with each
of the issue's four complaints answered against what the census really found, and the ear
list. Closing an issue is not a publish.

## Known stubs

None. `status_sentences` has a non-test caller for every function it exposes:
`nothing_chosen` from `wx_app`, `managers`, `wx_account_manager` and `wx_calendar`;
`nothing_chosen_named` from `managers` and `manager_words`; `at_least_one_chosen` from
`wx_app`; `Thing::named` from the module's own link check; `reads_as_a_persons_sentence`,
`a_noun_use_of_sync`, `WORDS_A_PERSON_DOES_NOT_USE` and both exception tables from the
census target. `Voice` is read by the target at every call it lists.
`grep -n 'TODO\|FIXME\|placeholder\|coming soon\|not available'` over the files this plan
created and changed answers nothing added by it.

## Not done here, on purpose

**Hearing it.** Nobody has read the status bar under the new wording, and whether one shape
helps by ear is the tester's to say, not a reading's. That is ledger 574 and LIST-11's one
remaining `[S]` line.

**The 236 sentences built somewhere else.** A reading over a status call sees a literal
written where it is shown. It does not see a sentence bound to a local name and handed over
as that name, which is how 236 of the 441 calls carry theirs, and following a binding to the
function that built it is a second hop over source text with failure modes of its own. The
census prints every one of the 236 with its file and line on its passing path, so the next
pass has its work list rather than a silence. This plan reached by hand every one it could
see beside a sentence it was changing, and the four builders the plan named are read over
fixtures. That is ledger 575.

## A correction after the merge, found by this summary's own self-check

The self-check ran `grep -rc 'No storage is open\|No message store is available\|Flushing
outbox queue\|No cache available' src --include=*.rs` expecting nothing, and it answered
three live sites. The claim in the changelog and the close comment, that two wordings for
one fact at nine sites are now one, was not true: there were twelve sites, and three of
them are `Err` strings that reach a person through a binding, which is exactly the set the
census cannot see and ledger 575 is about.

  "No message store is available, so the draft cannot be saved"
     -> "The mail on this computer is not open, so the draft cannot be saved."
  "No message store is available, so the message cannot be queued"
     -> "The mail on this computer is not open, so the message cannot be put in the
         Outbox." (and `queue` with it)
  "No message store is available" in `managers::manager_account`
     -> "The mail on this computer is not open."

Reading those three showed two more of the refusal family in the same two functions, in a
wording of their own and outside the census for the same reason: "Select an account before
saving a draft" and "Select an account before sending" are "Choose an account first, so the
draft has somewhere to go." and "... so the message has somewhere to go." The clause stays
because the remedy is not the one `nothing_chosen` names: there is no list in front of
somebody to choose from, and the account a composer sends from is set elsewhere. And "no
account is set up" at two sites against "No account is set up" at a third is one sentence
in two capitalisations with no ending; all three are "No account is set up." now.

On branch `the-third-wording-the-census-could-not-see` from `main` at `a9ce329d`, with its
own whole gate and merge. **This is what a self-check is for, and the check that found it
is a command in a summary rather than a test**: a reading over a status call could not have
found any of the five, because none of them is written at one.

## Self-Check: PASSED

`src/application/status_sentences.rs` and `tests/every_status_sentence_has_one_shape.rs`
exist on disk and hold 12 and 10 `#[test]` attributes by the count check's rule.
`grep -c 'nothing_chosen' src/presentation/wx_app.rs` is 5, one of them the import;
`grep -c 'at_least_one_chosen' src/presentation/wx_app.rs` is 4, the same.
`grep -rc 'No storage is open\|No message store is available\|Flushing outbox queue\|No cache available' src --include=*.rs`
answered three live sites on its first run, which is the correction above, and after it
answers only `status_sentences.rs` twice and `wx_app.rs` once, all three in a comment or a
test naming the wording that was replaced.
`grep -c 'pub fn nothing_chosen' src/application/status_sentences.rs` is 2,
`nothing_chosen` and `nothing_chosen_named`. `guards/guards.toml` holds 1,038 records by
the TOML reader and the census line says 241. `.planning/WINDOWS.md` holds 575 in both
halves with 538 open. `grep -c '^- \[x\] \*\*LIST-11' .planning/REQUIREMENTS.md` is 1.
`gh issue view 75` answers CLOSED. Commits `6c839c20`, `f9ae457d`, `eb1492ac`, `18876672`,
`ec95de88`, `e2cd7de5`, `409baa65`, `97b3a411` and `a9ce329d` are in `git log --oneline` on
`main`, with the correction's own commits after them.

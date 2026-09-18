---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 05
subsystem: the main window's download runner, the Tools menu, the experimental warnings, guards
tags: [download, runner, pause, backoff, retirement, tools-menu, guards, changelog, issue-20, issue-23]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-01's what_to_do_next, fetch_over_a_mailbox and WaitBeforeTryingAgain; 10-02's list with no limit and reread_folder_if_open; 10-02.1's listings in the chosen sort; 10-03's TextKept::budget and the keeping_bodies_under seam; 10-04's Progress and WhatArrived kinds, send_progress, and the reading that refuses a step on the answer channel"
provides:
  - "wx_app::start_the_download: for every enabled IMAP account in list order, asks what_to_do_next and does the one thing answered, a chunk of headers through sync_folder at INITIAL_FETCH_LIMIT with MoreOfWhatIsAlreadyThere or a chunk of text through fetch_over_a_mailbox, the folder on screen read from the state at each ask; one flag, downloading.paused, asked before each chunk of headers and handed to each chunk of text; a refused chunk ends the run, the wait rule is asked, next_attempt_at is recorded, and the main timer starts the next attempt on the network's cadence; retries at the cap for as long as the program runs and is not paused"
  - "WxUIState::downloading: Downloading { running, paused, next_attempt_at, wait }, session-only"
  - "UIUpdate::DownloadRequested, sent at the end of spawn_mail_sync after MailboxWatchRequested, whose arm calls start_the_download; start_the_download_if_its_wait_is_over on the timer"
  - "Get Older Messages answers the key through send_status with DOWNLOADING_THIS_FOLDER_FIRST and hands to the download; paused, it refuses through send_refusal and names the way out"
  - "ID_PAUSE_DOWNLOADING, a check item on Tools, Alt+P, described by DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL; pause_or_carry_on_downloading flips the flag, ticks the item and answers the key"
  - "allowed::DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL, one sentence replacing FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL and DOWNLOADING_A_WHOLE_FOLDER_IS_EXPERIMENTAL"
  - "wx_app::what_the_missing_text_means: the WhatCouldBeFetched arm says the text is on its way or that the Message Text box holds it back, and puts no button on the screen"
  - "Retired: ID_GET_WHOLE_FOLDER, spawn_whole_folder_fetch, application::asking_for_a_whole_folder whole, ID_FETCH_MISSING_TEXT, start_the_missing_text_fetch, STARTING_THE_MISSING_TEXT_FETCH, the offer panel, MissingTextOffer, the_offer_to_fetch, how_the_offer_is_announced"
  - "tests/everything_comes_down_without_being_asked.rs, renamed from a_whole_folder_moves_both_bounds.rs: nine readings over the window's shipping half with nine companions, coupled to wx_app.rs by five measured records"
  - "guards/guards.toml: five records new, three rewritten in place, one renamed, one corrected by hand, every one measured; 902 records, census 802 + 100"
  - "docs/KEYBOARD_SHORTCUTS.md: the Pause row, the Get older messages row and the This Folder row; docs/changelog.md: the entry for #20 and #23 under Unreleased, and four older entries dated; ledger 12 closed, 11 and 72 corrected by addition and 72 re-pointed, 522 to 524 added"
affects: [10-06 (the watch and the schedule end in spawn_mail_sync, whose end starts the download; the runner refuses to start while a wait is running; the Downloading state is where a bound on the watch would not go), 10-07 (reads MAIL-01's and MAIL-03's clauses from here, writes the alpha, privacy and guide pages, and the listening lines ledger 523 names), the tester, whose Gmail account meets the runner unasked on the first check after the build]

actuals:
  tokens: 44400
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A runner whose state is the cache: every decision is asked of a pure module per chunk, so a run that stopped anywhere resumes on the next start with no state file"
    - "A wait served by the main timer from a recorded instant rather than a thread asleep, with the runner refusing to start while the instant is in the future"
    - "One flag read in one closure and handed down: a stop asked before each chunk and passed into the chunk that asks it per message"
    - "A retired command's tests rewritten in place onto the thing that replaced it, so a file fifty-odd records fingerprint keeps its count"

key-files:
  created:
    - tests/everything_comes_down_without_being_asked.rs
  modified:
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/presentation/managers.rs
    - src/application/allowed.rs
    - src/application/mail_sync.rs
    - src/application/mod.rs
    - src/application/bringing_everything_down.rs
    - tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
  deleted:
    - src/application/asking_for_a_whole_folder.rs
    - tests/a_whole_folder_moves_both_bounds.rs

key-decisions:
  - "Get Older Messages answers the key through send_status, not as Progress, because 10-04 sorted that line as an answer and a key pressed into a silent step is pressed again"
  - "The runner's per-chunk update is MoreOfTheFolderArrived, whose arm and record already existed, rather than FolderMessagesArrived"
  - "One WhatArrived per account that had anything to do, sent from start_the_download, because the check's list has gone out hours before a download ends"
  - "application::asking_for_a_whole_folder is deleted whole rather than losing its loop, because nothing but the loop reached its sentences and a module of dead things is not a module"
  - "UIUpdate::WhatCouldBeFetched keeps its name: what it counts is unchanged, both searches send it, three managers.rs tests read it, and renaming it flags fifty-two records"
  - "start_the_download refuses to start while a wait after a refusal is running, keys included, so a person cannot undo the wait by pressing Shift+F9 or unticking Pause"
  - "mail_sync::fetch_the_missing_message_text and the fold stay, reached by their tests alone, with the doc comment saying so and ledger 524 holding the retirement"
  - "PROGRESS_OPENINGS is not extended: the runner's steps are worded in bringing_everything_down and never as literals in a send call, and the openings the runner's answers use, Downloading this folder first and Downloading is paused, are answers"

patterns-established:
  - "A red commit names a source-scanning guard that fires on the red half's own test literal as red-until-green, with the reason, rather than disguising the literal"

requirements-completed: [MAIL-01, MAIL-03]

coverage:
  - id: D1
    description: "Every check for mail ends by starting the download, for every enabled IMAP account, chunk by chunk, headers then text, the folder on screen first, resumable because its state is the cache"
    requirement: MAIL-01
    verification:
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_every_check_ends_by_starting_the_download"
        status: pass
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_the_download_does_what_the_model_says_and_nothing_of_its_own"
        status: pass
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_get_older_messages_hands_to_the_download_with_this_folder_first"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_the_download_says_it_has_started_before_it_opens_a_connection"
        status: pass
    human_judgment: false
  - id: D2
    description: "The download can be paused and carried on from the Tools menu, stops between chunks, and waits a growing time after a refusal, served by the timer"
    requirement: MAIL-01
    verification:
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_the_download_asks_whether_to_stop_between_chunks"
        status: pass
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_a_failed_chunk_waits_before_the_download_is_tried_again"
        status: pass
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_pause_downloading_is_on_the_tools_menu_ticked_and_carries_the_warning"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_pause_downloading_is_on_the_menu_bar_and_the_menu_reaches_the_arm"
        status: pass
    human_judgment: false
  - id: D3
    description: "The text of each message comes down with the mail unless the Message Text box forbids it or the size chosen has been reached, through fetch_over_a_mailbox one chunk at a time, and a search says what the text still missing means"
    requirement: MAIL-03
    verification:
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_the_download_does_what_the_model_says_and_nothing_of_its_own"
        status: pass
      - kind: integration
        ref: "tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs#test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_the_missing_text_sentence_says_it_is_coming_or_that_the_box_is_off"
        status: pass
    human_judgment: false
  - id: D4
    description: "Download This Whole Folder and Fetch Missing Message Text are gone with their warnings, and one sentence on the Pause item says the download has never met a real provider"
    requirement: MAIL-03
    verification:
      - kind: integration
        ref: "tests/everything_comes_down_without_being_asked.rs#test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_app.rs#test_the_answer_says_what_the_missing_text_means_and_puts_no_button_on_the_screen"
        status: pass
      - kind: unit
        ref: "src/application/allowed.rs#test_downloading_everything_says_it_is_experimental_and_says_what_could_go_wrong"
        status: pass
    human_judgment: false
  - id: D5
    description: "Whether Gmail tolerates 12,872 messages and their text chunk after chunk, what it does when it has had enough, and what a first download of 50 folders sounds like under each level"
    requirement: MAIL-01
    verification: []
    human_judgment: true
    rationale: "No provider has been observed; the runner was driven through readings and the model's tests only, and no binary was started because no loopback IMAP server exists as a process. Ledger 11, 72 and 523."

duration: 1h 44m to the merge
completed: 2026-09-17
status: complete
---

# Phase 10 Plan 05: The download runs after every check, for every account, chunk by chunk Summary

**Every check for mail now ends by starting a download of everything the kept folders hold, for
every enabled IMAP account, five hundred headers at a time with the folder on screen first and
then the text fifty messages at a time unless the Message Text box or the chosen size says no;
it stops between chunks when Pause Downloading on the Tools menu is ticked, waits thirty seconds
doubling to thirty minutes after a refusal with the main timer starting the next attempt, and
picks up where it was after a restart because its state is the cache. Download This Whole
Folder, Fetch Missing Message Text and the offer button above the list are gone with their two
warnings, replaced by one sentence on the Pause item. #20 and #23 closed from the merge commit.
Nothing here has met a real account and nobody has heard any of it.** Nothing pushed.

## Performance

- **Duration:** about 1 h 44 min from the first read to the merge, of which about 10 minutes
  were guard measurement across three runs (thirteen records, 628 s summed from the runners'
  `timed:` lines, the three baseline runs included) and about 14 minutes the two whole gates
- **Started:** 2026-09-18T01:04:31Z
- **Merged:** 2026-09-18T02:48:10Z at `b477e8c9`
- **Tasks:** 3
- **Files modified:** 15 (1 created, 2 deleted)

## What landed

**Task 1, the runner.** `start_the_download` in `wx_app.rs`, built from `spawn_whole_folder_fetch`
by generalising it, so there is one download path in the file. It returns at once when a run is
on a thread, when Pause Downloading is ticked, or when a wait after a refusal is still running;
otherwise it marks `running`, says `STARTING_THE_DOWNLOAD` as a step before anything is dialled,
and on a blocking thread, with one cache opened through `keeping_bodies_under(text_kept.budget())`
and the same value as the model's budget, walks every account with `enabled` true and
`protocol()` IMAP in list order. Per account: the session through `the_session_at`, the folder
list, `store_folders`, `folders_to_sync` over the stored choices, and each kept folder as a
`FolderHere` from the cache's own counts (`messages_stored_in` for `held`, the row's
`total_count` for `total_on_server`). Then the loop: `stop()` first, which reads
`downloading.paused`; `messages_with_no_text_here` and `cached_body_bytes` for `TextStillMissing`;
`what_to_do_next` with the folder on screen read from the state at each ask; and the one thing
answered. A chunk of headers is `sync_folder` at `INITIAL_FETCH_LIMIT` with
`MoreOfWhatIsAlreadyThere` and no filtering, after which the folder's `here` moves, `worked()`
tells the wait rule, `MoreOfTheFolderArrived` re-reads the open folder, and one `Progress` line
says how far the folder has got, that it is whole, or that the server stopped sending it. A chunk
of text is `fetch_over_a_mailbox` with `&stop` and nothing per message; a chunk that went through
says "Downloading message text: 50 of 4000 messages." as a step, `ReadingWasTurnedOff` turns
`reading_allowed` off for the rest of the run, `Stopped` ends the run as paused, and
`TheServerStoppedAnswering` says what the text pass came to and ends the run as refused. A
refused chunk of headers says `what_the_folder_download_came_to` with `AChunkFailed` worded from
`WhyTheServerStopped::from_the_kind_of`, never the server's text, and ends the run the same way.
`EverythingIsHere` says what the text pass came to when text was asked, with the budget's
sentence when the budget ended it, and sends one `WhatArrived` with `what_a_whole_account_came_to`
when the account had anything to do; an account with nothing to do says nothing. At the run's
end the lock is taken once: `running` cleared, and on a refusal `next_wait()` asked,
`next_attempt_at` recorded, and `what_to_say_before_waiting` sent as a step.
`start_the_download_if_its_wait_is_over` runs on the timer's ten-second cadence and starts the
download when the instant has passed and it is neither paused nor running. `spawn_mail_sync` ends
by sending `UIUpdate::DownloadRequested` after `MailboxWatchRequested`, whose arm calls
`start_the_download`. The Get Older Messages arm sends `DOWNLOADING_THIS_FOLDER_FIRST` through
`send_status` and hands to the download, or refuses with `DOWNLOADING_IS_PAUSED` when paused.
`ID_GET_WHOLE_FOLDER`, its arm, its item, `spawn_whole_folder_fetch` and the whole of
`application::asking_for_a_whole_folder` are gone; `WhyTheServerStopped::as_a_clause` is
crate-wide. `grep -c 'ID_GET_WHOLE_FOLDER\|spawn_whole_folder_fetch\|until_the_whole_folder_is_here'
src/presentation/wx_app.rs` is 0; `grep -n 'DownloadRequested' src/presentation/wx_app.rs` shows
the arm at `:18276` and the send at `:22276`, at `b477e8c9`. `cargo test --lib presentation::wx_app::` passes
with 199, quoted from the run, before and after.

**Task 1, the target.** `tests/a_whole_folder_moves_both_bounds.rs` renamed with `git mv` to
`tests/everything_comes_down_without_being_asked.rs` (`git log --diff-filter=R --summary`
shows the rename at `56fe7eb7`) and rewritten: sixteen tests at the green, nine readings and
seven companions, eighteen after task 2. The readings hold the check's end sending
`say(UIUpdate::DownloadRequested)` after `say(UIUpdate::MailboxWatchRequested)`, matched as a call
and not a name; the runner reaching `what_to_do_next(`, `INITIAL_FETCH_LIMIT`,
`fetch_over_a_mailbox(` and `MoreOfWhatIsAlreadyThere`; `downloading.paused` read, `if stop()`
before headers and `&stop,` handed to text; `.next_wait()`, `next_attempt_at = Some(` and
`.tell_it_worked()`; the window naming none of the retired command's three names and the retired
module absent from the tree; the Get Older arm calling `start_the_download(` after a
`send_status(` and `spawn_mail_sync(` nowhere; the chunk arm re-reading and growing no limit, as
10-02 left it; the runner sending `UIUpdate::Progress(`, no `StatusUpdated(`, and exactly one
`UIUpdate::WhatArrived`, with the `Progress` arm asking the level, as 10-04 left it. Each
companion plants the opposite into a snippet. `bash scripts/check.sh --suites-for guards/guards.toml
src/presentation/wx_app.rs` names `everything_comes_down_without_being_asked`, quoted from the
hook's own list at every commit.

**Task 2, the Pause and the retirements.** `ID_PAUSE_DOWNLOADING` is a check item on Tools after
Flush Outbox, "&Pause Downloading", Alt+P, which nothing else on that menu claims (its letters
are A, n, d, T, W, C, F, k, i, g, b, r, O and S, read off the builder;
`test_no_two_items_on_one_menu_claim_the_same_letter` green), with
`DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL` as its description. Its arm calls
`pause_or_carry_on_downloading`, which flips the flag under the lock, ticks the item through
`sync_menu_check`, and answers the key through `send_status`: "Downloading is paused. Mail
already here stays readable.", "Downloading again." with `start_the_download` called, or
"Downloading again when the wait after the last refusal is over." when a wait is running.
Session-only, like offline mode, with the reason in the comment. `allowed.rs` holds one sentence
where there were two: the download runs on its own after every check, has never been run against
a real account, a provider is entitled to refuse, slow down or disconnect and nothing here can
find out which, nothing is changed at the server, if it stops it says so and tries again, Pause
Downloading on Tools holds it; its two tests rewritten in place hold those four things and that
it sits beside `EXPERIMENTAL_WARNING`. `cargo test --lib application::allowed::` passes with 27
before and after. `ID_FETCH_MISSING_TEXT`, the File menu item, its arm,
`start_the_missing_text_fetch`, `STARTING_THE_MISSING_TEXT_FETCH`, the offer panel, its button
handler, `MissingTextOffer`, the handler's field, `the_offer_to_fetch` and
`how_the_offer_is_announced` are gone. The `WhatCouldBeFetched` arm says
`what_the_missing_text_means(count, reading_allowed)` on the "message text" topic at Low and puts
nothing on the screen: nothing for nought, "The text of 137 messages in this account is not here
yet and comes down on its own after the next check." with reading allowed, and "... is not here,
and the Message Text box on the Permissions tab is off, so it stays on the server." without.
`grep -rn 'FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL\|DOWNLOADING_A_WHOLE_FOLDER_IS_EXPERIMENTAL\|ID_FETCH_MISSING_TEXT\|missing_text_offer' src tests --include='*.rs'`
finds one line, the rewritten test asserting the first name is absent from the shipping half;
`WhatCouldBeFetched` stays by decision, below. `grep -n 'ID_PAUSE_DOWNLOADING' src/presentation/wx_app.rs`
shows the id, the check item on the Tools builder, the arm and the tick. `grep -n 'Pause Downloading\|Download This Whole Folder\|Fetch Missing Message Text' docs/KEYBOARD_SHORTCUTS.md`
shows the first twice (its row, and the This Folder row's dated sentence that the download
replaced the whole-folder item) and the other two once each, inside the dated sentences that say
they are gone; the plan's "not at all" would have left the page silent about where two commands
went, and a name in a sentence saying it is gone is not a command described. Six tests in
`wx_app.rs` rewritten in place onto the sentence, the arm and the Pause item; `wired.rs`'s two
readings that name Get Older Messages say what it does now, no test added.

**Task 3, the changelog.** The entry for #20 and #23 under `[Unreleased]`, Changed, landed with
task 1's green and grew with task 2's, on the rule that the entry goes with the behaviour: the
tester's two sentences; everything coming down after every check, headers then text, the folder
on screen first; the pause; Shift+F9; the three retirements and the sentence a search says now;
the wait; what is a step and what is the one result; Known limitations with the experimental
sentence's substance, nobody having heard the Pause item or its answers, All Mail and Spam not
kept by default, a folder not kept not downloaded, the first download a long run, the cache not
encrypted. Four older entries carry "**Corrected on 2026-09-17:**" in their own manner and keep
their words: the whole-folder request, the offer button, the File menu item, and the two Known
limitations lines 10-03 and 10-04 wrote. `cargo test --test house_style` passes with 74 and
`--test the_words_that_say_nothing` with 9; carriage returns on the page 0.

## Task commits

| Commit | What |
|---|---|
| `56fe7eb7` | test(10-05): the red half of task 1, eight readings named bare and two `wx_app.rs` tests by module path; eight companions green on arrival, said; three records kept honest |
| `9acd8f23` | feat(10-05): the runner, the arms, the timer, the retirements of the whole-folder command and its module, seven records measured, the shortcuts page, the changelog entry |
| `c767d8e2` | test(10-05): the red half of task 2, one reading bare, two `allowed.rs` and six `wx_app.rs` tests by module path, the count check bare, and `wired`'s raise check named as red-until-green with the reason |
| `9bc435db` | feat(10-05): the Pause item, the one warning, the text fetch's command and offer retired, six records measured, the Pause row, the changelog extended |
| `f1999ec8` | docs(10-05): four older changelog entries dated, not deleted |
| `b477e8c9` | Merge 10-05 into `main` |

Branch `everything-comes-down-without-being-asked` from `main` at `89a4e105`. Not pushed.

## Honest RED and GREEN

Two red commits, each on the branch, each running and failing exactly what it named and nothing
else, which the gate said in `red` mode. Task 1's names eight readings bare and two `wx_app.rs`
tests by module path, and says the eight companions were green on arrival because they read
snippets; the two `wx_app.rs` constants existed empty and `pub` so the red was an assertion and
not a missing name, and were narrowed and filled in the green. Task 2's names one reading bare,
two `allowed.rs` and six `wx_app.rs` tests by module path, the count check bare because the
target went from sixteen to eighteen, and one more the commit did not set out to make red:
`test_every_command_something_raises_is_handled` in `wired`, which walks every file under
`src/presentation`, test modules included, and took the literal
`append_check_item(ID_PAUSE_DOWNLOADING,` inside the rewritten `wx_app.rs` test as an item
raised with no arm. The gate refused the commit until it was named; it was named with the
reason, because the green that adds the arm is what it is about, and it went green there.
`DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL` existed in the red as `EXPERIMENTAL_WARNING` so both
`allowed.rs` tests were red on an assertion; `what_the_missing_text_means` answered an empty
sentence for every count so all three tests that read it were red. Observation 659 in the skill
log holds the `wired` finding.

## What the gate selected

| File | On the branch |
|---|---|
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::` (199) and fifteen coupled targets, the renamed one among them, on every commit but the last |
| `src/application/mod.rs` | `--lib application` whole, on task 1's green |
| `src/application/allowed.rs` | `--lib application::allowed::` on both task 2 commits |
| `src/application/mail_sync.rs`, `bringing_everything_down.rs` | their own module paths on the commits that touched them |
| `src/presentation/ui_types.rs`, `managers.rs` | their own module paths and coupled targets |
| the renamed target, `wired`, the 10-03 target | their own targets as changed, and coupled |
| `guards/guards.toml`, `docs/*.md` | no scoped target; the whole-tree guards and the document readers |

`scripts/check.sh all` on the branch at `f1999ec8`, run once with its output to a file and the
exit status read directly, never piped: exit 0, 7,998 passed and none failed over 73 result
lines, 425 s from 02:33:36Z to 02:40:41Z, the release build included. Three more than 10-04's
7,995: the target went from 8 to 18 and the retired module's 7 went, and nothing else moved.
Inside the 275 s to 654 s band on `docs/development/measurements.md`. `main`'s hook ran `all`
again on the merge: 7,998
and none failed, 427 s from 02:41:03Z to 02:48:10Z. The other commits' hooks took 5 m 21 s
(task 1 green), 4 m 53 s (task 2 red) and 5 m 22 s (task 2 green); task 1's red 4 m 50 s. The
keyring race (ledger 374) and the tab row reading's intermittent failure (10-02.1) did not
appear in any run.

## Guard records

897 records by the TOML reader before, 902 after; census 802 + 95 before, 802 + 100 after, the
line at `guards/guards.toml:84` moved in each green commit. Five new, three rewritten in place,
one renamed with its target, one corrected by hand, every one measured through
`scripts/guards.sh --remeasure` with `WIXEN_TEST_THREADS` untouched and the counts written by the
runner. Sixty-six records name `wx_app.rs` now (62 before); five name the target.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| a chunk of the download that lands is shown, and no limit grows (renamed) | `wx_app.rs`, the target | as before, the re-read taken out of the `MoreOfTheFolderArrived` arm | 1 | rebuild 12 s, run 1 s; again at 18, rebuild 13 s, run 1 s |
| every check ends by starting the download | `wx_app.rs`, the target | the send kept as a mention, `let _ = UIUpdate::DownloadRequested;` | 1 | rebuild 16 s, run 1 s; again rebuild 16 s, run 1 s |
| the download asks whether to stop between chunks | `wx_app.rs`, the target | `if false && stop()` before a chunk of headers | 1 | rebuild 14 s, run 1 s; again rebuild 14 s, run 1 s |
| a failed chunk waits before the download is tried again | `wx_app.rs`, the target | the wait rule not asked, `Duration::ZERO` recorded | 1 | rebuild 15 s, run 1 s; again rebuild 15 s, run 1 s |
| pausing the download ticks the item as well as flipping the flag | `wx_app.rs`, the target | `sync_menu_check` dropped | 1 | rebuild 12 s, run 1 s |
| get older messages answers the key before it hands to the download (rewritten from "choosing the offer starts the backfill and says what it did") | `wx_app.rs` | the `send_status` dropped, the hand-over kept | 1 | rebuild 43 s, run 49 s |
| the download says it has started before it opens a connection (rewritten from "the fetch says what it is doing before it opens a connection") | `wx_app.rs` | `STARTING_THE_DOWNLOAD` emptied | 1 | rebuild 42 s, run 49 s |
| the download's warning says no real account has met it | `allowed.rs` | the clause dropped from the sentence | 1 | rebuild 34 s, run 49 s |
| a folder is reported as stopped only when a chunk brought nothing new (corrected) | `bringing_everything_down.rs` | as before | 1, was 3 | rebuild 39 s, run 49 s |

**The count check's remedy, run and read.** After task 2's red the target held eighteen where
four records said sixteen; the remedy named the four and each agreed as written, 13 s to 16 s,
and their counts now say 18. **The record naming the retired module's tests was corrected by
hand first**, because `test_every_test_a_guard_record_names_is_a_test_that_exists` refuses a
commit that names a test that is gone: its two `asking_for_a_whole_folder` tests came off the red
list and the file off `tests_last_seen`, with the reason dated on it, and the run after that
reddened exactly the one. The two records on the retired text-fetch command were rewritten in the
same commit as the tests they name were renamed, and their `before` blocks moved onto the arm and
the constant in task 1's green, where each was measured. Counts written: `wx_app.rs` 199 on every
record naming it, `allowed.rs` 27, `bringing_everything_down.rs` 23, the target 18. No file
gained or lost a test after its records were measured; the count check is green on `main` at
`b477e8c9`.

## What the tree contradicted

Every command in the plan's eight premises was re-run against `main` at `89a4e105` before
anything was built. The plan was written before 10-02 to 10-04 landed and says so; six things
moved in kind, and every line number in the plan had moved.

1. **Premise 3 counted five tests reading the retired offer and commands; nine did.** The plan's
   `awk '$1>30060 && $1<30230'` selected by line, and the four tests above the range read
   `the_offer_to_fetch`, `how_the_offer_is_announced` and `WhatCouldBeFetched` rather than the
   command's body. All nine were rewritten in place, two in task 1 and six in task 2, and one
   kept because the saved-search path still works out and sends the count; `wx_app.rs` stays at
   199. Observation 658 in the skill log.
2. **Get Older Messages' answer is on the answer channel, not a step.** The plan says
   "Downloading this folder first..." goes out as `Progress`; 10-04's table sorts that arm's
   line as an answer to a key, and a key pressed into a step that is silent under the default is
   pressed again. It goes through `send_status`. The runner's own lines are `Progress`.
3. **The runner's per-chunk update is `MoreOfTheFolderArrived`.** The plan names
   `FolderMessagesArrived`; the other variant's arm and its guard record already existed for
   exactly a chunk landing, and using the plan's would have left a variant nothing constructs,
   which clippy refuses, and a record whose break no longer applies.
4. **One `WhatArrived` per account per run, from the runner's own function.** 10-04's summary
   said to fold the runner's counts into the check's list; the check's list has gone out hours
   before a download ends, and the reading holds `spawn_mail_sync` alone to one. The runner's
   `WhatArrived` reaches the same arm, where the new-mail sound is signalled from one place, so
   the sound plays once at the end of an account's run. `test_the_mail_checks_lines_are_steps_and_what_arrived_goes_out_once_after_the_loop`
   and `test_the_new_mail_sound_plays_when_a_check_found_mail_and_not_when_the_watch_woke` are
   green with it.
5. **The whole module went, not the loop alone.** Premise 8 says `asking_for_a_whole_folder`
   "loses its loop and keeps `HowMuchIsHere` if 10-01 left it there; it is named by no record".
   10-01 moved `HowMuchIsHere` out and re-exported it; nothing but the loop reached the module's
   sentences and enum; and one record did name two of its tests, corrected above. `cargo test
   --lib application::asking_for_a_whole_folder::` runs nothing now, which is the honest answer
   to the plan's verify line.
6. **`WhatCouldBeFetched` stays, and `MissingTextOffer` was the plan's only name for the panel.**
   The plan retires the update; both searches send it, `managers.rs` reads it in three tests a
   record names, and 52 records fingerprint that file at 137. What the update counts is unchanged
   and the arm does something else with it, so it keeps its name with the doc comment saying
   why. The plan's list also did not name the offer button's click handler or the
   `UpdateTargets` field that carried the panel; both went.

Two things the plan could not know: `wired`'s raise check reads test modules, above; and
`sync_folder` is the only place that keeps a folder row's `total_count` current, so a kept folder
no check has ever synced has both counts at nought, reads as whole, and is asked by the next
check rather than by the runner, which the doc comment on `folders_here_for` says.

## Deviations from plan

**1. [Decision] The six points above.** Ledger 522, so 10-06 and 10-07 read the runner from the
tree.

**2. [Decision] `start_the_download` refuses to start while a wait is running, keys included.**
The plan has it do nothing when running or paused; a key that started it during the wait would
undo the wait, which exists to leave a server alone. Shift+F9 still answers and the timer starts
the run when the wait is over; unticking Pause during a wait says so.

**3. [Decision] `fetch_the_missing_message_text` and the fold stay.** The plan offered "kept if a
test in `mail_sync.rs` needs its shape, and the summary says which and why": fifteen tests hold
the fold, `what_the_fetch_did`, `says_where_it_is`, `about_to_fetch` and the reading gate through
it, in a file thirteen records fingerprint at 149, and taking them out is a rewrite of that suite
with every one of those records corrected by hand. Reached by nothing outside its tests, said in
its doc comment, ledger 524. The clause `what_the_fetch_did` words is the one the runner uses.

**4. [Decision] `PROGRESS_OPENINGS` not extended.** 10-04's summary said to add an opening for a
new kind of step; the runner's steps are worded in `bringing_everything_down` and never as a
literal in a send call, so an opening would hold nothing, and "Downloading " is the opening of
two answers the Pause arm and the Get Older arm send on purpose.

**5. [Rule 1] `folders_here_for` took an account id.** The first draft read the account's folder
rows through a path that did not exist; it takes the id and reads the rows once into a map of
totals.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; `git mv` renamed the target, `git rm` removed the module, `cargo fmt` formatted before each
commit; the only `sed`, `awk` and Python in the session read files, logs and `guards.toml`.
Commit messages were written to the scratchpad and passed with `-F`. Carriage returns measured
with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No em dash in
any file this plan wrote, measured with `grep -c` for the byte sequence: zero on each. `Cargo.toml`
and `Cargo.lock` untouched; no crate added. No AI attribution anywhere. The version stays
`1.0.0-alpha.1`. The tester's profile was not read; no binary was started.

## Threat register

T-10-15: chunks bounded by 10-01's constants, three refusals end a chunk, the wait grows to
thirty minutes and the run retries at the cap; the experimental sentence on the Pause item;
ledger 11 and 72 corrected and open. T-10-16: `next_attempt_at` from the wait rule, the runner
refusing to start while it is in the future, and a record whose break records the moment
without asking. T-10-17: accepted; 10-07 writes the privacy page. T-10-18: every log line in the
runner names an account, a folder, a count or a reason; none names a subject or a body, and the
refused-chunk sentence is worded from the error's kind. T-10-19: `reading_allowed` from
`allowed_for` into `what_to_do_next`, turned off for the rest of a run by `ReadingWasTurnedOff`.
T-10-SC: no crate added. No new surface outside the register: the Pause item is one flag, and
the search's sentence reads what the offer read.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash; 521 before, 524 after; 493
open before, 495 after; 28 fixed before, 29 after.

| id | kind | what |
|---|---|---|
| 12 | closed | the offer button is gone with the panel |
| 11 | corrected by addition | the bulk fetch is the download's text pass now, started by every check |
| 72 | corrected by addition, re-pointed at `bringing_everything_down.rs` | the request and its module are gone; the loop is the download, with a pause and a wait |
| 522 | deviation | the six departures above |
| 523 | unrun-verify | what only the tester's account and ear settle for #20 and #23 |
| 524 | stub | the text pass's entry point and fold, reached by their tests alone |

## Known stubs

**Two functions are reached by nothing outside their tests, on purpose, and this is the plan that
says so.** `mail_sync::fetch_the_missing_message_text` and `fetch_all_the_missing_text`, above
and ledger 524. `grep -rn 'fetch_the_missing_message_text\|fetch_all_the_missing_text(' src
--include='*.rs'` outside `mail_sync.rs` finds nothing. Everything else this plan built is reached:
`start_the_download` from the `DownloadRequested` arm, which every check's end sends, from the
Get Older arm, from the Pause arm and from the timer; `pause_or_carry_on_downloading` from the
Tools item; `what_the_missing_text_means` from the `WhatCouldBeFetched` arm, which both searches
send.

## What the tester's account could settle, and what a loopback proved

Nothing in this plan is claimed against a real provider or a real ear. The readings proved the
runner's shape, the stop, the wait, the check's end, the Get Older arm, the Pause item and the
retirements; the model's tests (10-01) proved every decision the runner asks; the scripted mailbox
proved the chunk of text. The plan's verification asked for the release binary started against a
loopback IMAP server "if one can be started as a process": the loopback servers in this tree are
scripted conversations inside tests (`conversing(...)` in `imap.rs` and `mail_session.rs`), not a
process a binary can be pointed at, so no binary was started, and the runner was driven only
through readings and the model's tests. What a run would have shown is a status bar reading
"Downloading the mail that is not on this computer yet...", then "Downloading Inbox: 500 of
12872 messages." per chunk, "Inbox is downloaded: 12872 messages on this computer." per folder,
"Downloading message text: 50 of 12872 messages." per chunk of text, and one spoken sentence,
"50 folders are downloaded, and the text of 12872 messages is on this computer.", with the sound
for new mail; or, on a refusal, "Downloading Inbox stopped: the mail server stopped answering,
and it refused. 3500 of 12872 are on this computer." followed by "The mail server could not be
reached. Trying again in 30 seconds." What Gmail does with chunks of five hundred and fifty, and
what any of it sounds like, is ledger 523.

## Not done here, on purpose

No `MAIL` requirement is ticked, on the phase's rule that 10-07 reads each clause; the clauses
this plan gives a named test or reading for are in the coverage block above. Roadmap criteria 1
and 3 close structurally and are read by 10-07. The four documents 10-07 writes (`ALPHA_TESTING`,
`privacy`, `USER_GUIDE`, `manual-accessibility-pass`) are untouched; ledger 523 holds the
listening lines for them. The `Downloading` state has no bound on retries and the doc comment says
why; 10-06 is where a bound applies.

## For 10-06 and 10-07

- The download is started from one place, the `DownloadRequested` arm, which `spawn_mail_sync`
  sends last. The watch's restart and the schedule (10-06) end by starting a check, so they reach
  the download without touching it. `start_the_download` returns at once when
  `downloading.running`, `downloading.paused` or `downloading.is_still_waiting()`, so a check
  during a wait does not shorten it.
- The wait lives on `WxUIState::downloading.wait`, a `WaitBeforeTryingAgain`; the watch wants its
  own instance, not this one, because a server refusing the download and a server dropping IDLE
  are different failures with different counts.
- `start_the_download_if_its_wait_is_over` runs inside the network block of the main timer, on
  its ten-second cadence; a second periodic thing 10-06 adds goes beside it, not on a second
  timer.
- The runner sends `Progress` per chunk, `MoreOfTheFolderArrived` per chunk of headers, and one
  `WhatArrived` per account with something to do; nothing per folder at Normal; a refusal is a
  step. The Pause item's answers and Get Older Messages' answer are `send_status`.
- `PROGRESS_OPENINGS` was left alone; a step written as a literal in a send call in 10-06 wants
  its opening added there.
- 10-07 dates nothing here; the changelog entry is complete, and `docs/KEYBOARD_SHORTCUTS.md`
  carries the Pause row. What the pages should say about a whole-mailbox download is the
  experimental sentence's substance, `DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL` in `allowed.rs`.

## Self-Check: PASSED

`tests/everything_comes_down_without_being_asked.rs` exists with 18 `#[test]` functions;
`src/application/asking_for_a_whole_folder.rs` and `tests/a_whole_folder_moves_both_bounds.rs`
do not; `grep -c 'ID_GET_WHOLE_FOLDER\|spawn_whole_folder_fetch\|until_the_whole_folder_is_here'
src/presentation/wx_app.rs` is 0; `grep -n 'ID_PAUSE_DOWNLOADING' src/presentation/wx_app.rs`
shows the id, the check item, the arm and the tick; `guards/guards.toml` holds 902 records by the
TOML reader and the census line says 100; `docs/changelog.md` holds the line beginning
"**Everything comes down on its own, with its text.**"; `.planning/WINDOWS.md` holds 522, 523 and
524 in both halves and 12 fixed in both. Commits `56fe7eb7`, `9acd8f23`, `c767d8e2`, `9bc435db`,
`f1999ec8` and `b477e8c9` are in `git log --oneline` on `main`. Issues #20 and #23 read CLOSED
by `gh issue view --json state`.

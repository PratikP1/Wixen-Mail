---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 06
subsystem: the inbox watch, the schedule, the startup check and the status line in the main window; the schedule's decisions as a module; guards
tags: [watch, idle, restart, schedule, check-interval, startup, status-line, network, guards, changelog, measurements, issue-37]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-01's WaitBeforeTryingAgain and its wording; 10-04's Progress kind and the reading that refuses a step on the answer channel; 10-05's DownloadRequested sent last by the check, start_the_download refusing to start while running, paused or waiting, and start_the_download_if_its_wait_is_over on the timer's ten-second block"
provides:
  - "application::checking_on_a_schedule: AccountToCheck from the account row by From; which_are_due over handed-in instants, the interval clamped 1..=60 as the editor clamps it; WhyTheWatchEnded (Ended from ImapIdleEvent::Stopped, NeverStarted from watch_folder's Err); whether_to_watch_again answering AfterAWait, Never for a stop somebody asked for, or OnTheScheduleAlone(Until::TheServerAnswersAgain | TheProgramStartsAgain) after three in a row; what_the_status_line_says over WhatIsRunning; what_a_check_of_them_all_says; 20 tests"
  - "wx_app::InboxWatch per account on WxUIState::mail_watches, its handle, whether the loop is reading, its own WaitBeforeTryingAgain, next_watch_at, and the schedule-alone mark; WxUIState::last_checked, the schedule's session clock"
  - "wx_app::spawn_mail_watch(app, account_id): a watch already reading events is left alone; the previous one is stopped; the Err from watch_folder and from for_account asks what_the_watch_does_next with NeverStarted before it returns; Stopped asks with Ended; Changed and StillWatching tell the wait it worked; MailboxChanged carries a WatchedFolder naming the account"
  - "wx_app::what_the_watch_does_next: the rule asked with this failure counted, a wait recorded for the timer or the account left to the schedule alone, the attempt logged with the account, the reason and the wait and never a folder's contents, the status line sent as Progress"
  - "wx_app::start_the_watches_whose_wait_is_over on the timer's ten-second block beside the download's wait; check_the_accounts_that_are_due on the minute look, following the network; check_every_enabled_account (F9 and a start, one worker for the list); watch_every_enabled_account (a start and the network coming back); every_enabled_account"
  - "spawn_mail_sync(app, accounts, only, wanted): the accounts it is handed, one after another, each that went through ending with mark_synced, update_account_last_sync and its own MailboxWatchRequested(id); the download asked for once at the end and only when some account went through; last_checked written before the worker starts"
  - "act_on_what_the_network_did(news, app): the back branch cuts every wait short, lifts the marks whose reason was the reach, asks for every watch and checks the due accounts"
  - "UIUpdate::MailboxWatchRequested(String) and MailboxChanged(WatchedFolder); the sentence about mail not appearing on its own and say_the_watch_is_off gone; the two doc comments stacked above say_the_link_was_refused home"
  - "The account editor's Check Interval field described by WHAT_THE_INTERVAL_DOES on the field and beneath it, hidden with the page"
  - "tests/mail_keeps_arriving_on_its_own.rs: seven readings over the window's shipping half and seven companions, coupled to wx_app.rs by seven measured records; wired's watch reading and 10-05's reading of the check's end rewritten in place"
  - "guards/guards.toml: ten records new, one corrected for an indent and measured again, two measured again, every one measured; 912 records, census 802 + 110"
  - "docs/changelog.md: the entry for #37 under Unreleased, Fixed; docs/KEYBOARD_SHORTCUTS.md: the F9 row; docs/development/measurements.md: the cold-start and idle rows re-taken under a start that attempts an account, the old rows dated; ledger 446 closed, 447, 64 and 65 corrected by addition, 525 to 528 added"
affects: [10-07 (reads MAIL-04's clauses from here and writes the alpha, privacy and guide pages; the listening lines are ledger 525), the tester, whose Gmail account meets a watch per account, a restart after a drop and a check every five minutes unasked from the first start after the build]

actuals:
  tokens: 44500
  tasks: 3
  commits: 8

tech-stack:
  added: []
  patterns:
    - "A decision about a long-running connection answered by a pure function over a reason string and a count, with the three answers naming what would change them, so the window writes the answer and never decides"
    - "A per-account watch record holding the connection, whether the loop is reading, its own wait and its own mark, so a request can tell a working watch from one whose reader has gone"
    - "A schedule that follows the network and keeps its own session clock, so a scheduled check is never an error every interval while the network is gone"
    - "A reading proved against the green code before it is trusted: the first draft of one matched a word that the comment beside the code repeated"

key-files:
  created:
    - src/application/checking_on_a_schedule.rs
    - tests/mail_keeps_arriving_on_its_own.rs
  modified:
    - src/application/mod.rs
    - src/application/trying_again.rs
    - src/data/account.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_account_manager.rs
    - tests/everything_comes_down_without_being_asked.rs
    - tests/wired.rs
    - tests/the_numbers_the_targets_ask_for.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/development/measurements.md
    - .planning/WINDOWS.md

key-decisions:
  - "The restart decision answers three things, not the plan's two: Never for a stop somebody asked for, because the window replacing a watch stops the old one and must not mark the account; OnTheScheduleAlone with what lifts it, because a server that could not be reached is lifted by a check that reaches it or the network coming back, and a server that refused to watch only by the next start"
  - "A watch already reading events is left alone by a request, so the scheduled check every five minutes does not open and close the watch connection for nothing"
  - "The download is asked for only after a check in which some account went through, as 10-05's tree did; found by running the release binary, where a failed check started a download that failed the same way and waited on its own"
  - "The schedule follows the network and does not run while the program believes there is none: an unreachable server is an error every interval only while the network is there to reach it"
  - "mark_synced marks the worker's copy and update_account_last_sync writes the column, not save_account, which writes the credential store on every call"
  - "The check interval is the account editor's field made true, with no second interval on the Settings screen, as the README decided; Pratik saw the decision and did not overrule it"
  - "The wait's wording in trying_again is shared with the status line, so one length of time cannot be said two ways"

patterns-established:
  - "A red commit that rewrites a reading in another plan's target in place, count unchanged, and names it red-until-green beside the new target's readings"

requirements-completed: [MAIL-04]

coverage:
  - id: D1
    description: "When an inbox watch ends for any reason but mail arriving it is started again after a growing wait, the network coming back starts one at once, and every enabled IMAP account has a watch of its own"
    requirement: MAIL-04
    verification:
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_a_watch_that_ends_asks_whether_to_watch_again"
        status: pass
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_a_watch_that_never_started_asks_too_before_it_returns"
        status: pass
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_the_network_coming_back_starts_the_watches_at_once"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_a_watch_whose_connection_was_lost_is_tried_again_after_a_wait"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_every_reason_the_watch_can_end_with_has_an_answer_that_is_not_a_guess"
        status: pass
    human_judgment: false
  - id: D2
    description: "Where a watch cannot cover, mail is checked on a schedule, the interval being the account editor's Check Interval, clamped as the editor clamps it, and the schedule does not run while the network is gone"
    requirement: MAIL-04
    verification:
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_the_timer_checks_on_the_accounts_own_interval"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_an_account_whose_interval_has_passed_is_due"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_a_stored_interval_of_nought_is_read_as_one_minute_and_not_every_tick"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_a_server_that_would_not_start_watching_three_times_is_left_to_the_schedule"
        status: pass
    human_judgment: false
  - id: D3
    description: "A start checks every enabled account and asks for every watch without a keystroke; F9 checks every enabled account; the check writes down when each was checked; the status line says watching, waiting or checking and nothing says mail will not appear on its own"
    requirement: MAIL-04
    verification:
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_a_start_checks_without_a_keystroke"
        status: pass
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_the_check_for_mail_walks_every_enabled_account_and_writes_down_when"
        status: pass
      - kind: integration
        ref: "tests/mail_keeps_arriving_on_its_own.rs#test_nothing_says_new_mail_will_not_appear_on_its_own"
        status: pass
      - kind: unit
        ref: "src/application/checking_on_a_schedule.rs#test_the_status_line_says_the_wait_and_the_interval"
        status: pass
      - kind: other
        ref: "one run of the release binary against the measurement profile, its log read: the refusal at the start, the wait, the second and third refusals, the account left to the schedule alone"
        status: pass
    human_judgment: false
  - id: D4
    description: "Whether Gmail drops an IDLE connection and after how long, whether the restart carries mail in over hours, whether two connections per account are welcome, and whether the three status lines read as states"
    requirement: MAIL-04
    verification: []
    human_judgment: true
    rationale: "No provider has been observed dropping a watch on this program and nobody has heard the status lines; the measurement account is refused by the credential store before anything is dialled. Ledger 64, 65, 67, 525 and 526."

duration: 2h 22m to the merge
completed: 2026-09-18
status: complete
---

# Phase 10 Plan 06: The watch that keeps going Summary

**Mail keeps arriving on its own for as long as the program runs. Every enabled IMAP account has
a watch of its own; when it ends for any reason but mail arriving it is asked about rather than
broken from, and the answer is a wait of thirty seconds doubling to half an hour, served by the
main timer, or the schedule alone after three refusals in a row; the network coming back starts
every watch at once and checks the due accounts. Where a watch cannot cover, the accounts are
checked on the Check Interval the account editor has offered since 2026-03-01 and nothing read
until now, and the editor says so under the field. A start checks every enabled account and asks
for every watch; F9 checks every enabled account rather than one. The status line says watching,
waiting or checking, with the account named when more than one is enabled, and the sentence that
said new mail would not appear on its own is gone. #37 closed from the merge commit. Nothing here
has met a real server and nobody has heard any of it.** Nothing pushed.

## Performance

- **Duration:** about 2 h 22 min from the first read to the merge, of which about 8 minutes were
  guard measurement across four runs (thirteen records, 446 s summed from the runners' `timed:`
  lines, the four baseline runs another 68 s), about 27 minutes the seven hook runs (242 s, 212 s,
  202 s, 285 s, 200 s, 281 s and 179 s), about 14 minutes the two whole gates (435 s and 430 s),
  and about 27 minutes the twelve starts of the release binary that took the rows, two minutes
  each, on two builds of 77 s and 78 s
- **Started:** 2026-09-18T03:04:20Z
- **Merged:** 2026-09-18T05:26:09Z at `2da50b6b`
- **Tasks:** 3
- **Files modified:** 16 (2 created)

## What landed

**Task 1, the decisions.** `src/application/checking_on_a_schedule.rs`, registered beside
`trying_again`, with the doc comment saying what the schedule covers that a watch cannot: the
twenty-nine-minute silence of a dead socket, a server without IDLE, POP, and the kept folders
that are not the inbox, and that the interval is the account editor's field made true.
`AccountToCheck { id, enabled, protocol, check_interval_minutes }` is built from `Account` by
`From`, so the schedule cannot read a field the editor does not write. `the_interval_of` clamps
to `SHORTEST_INTERVAL_MINUTES..=LONGEST_INTERVAL_MINUTES`, 1 and 60, so a stored nought is a
minute and not every tick; `which_are_due` answers the enabled accounts whose interval has passed
since they were last checked and every enabled account never checked, in list order.
`whether_to_watch_again(&WhyTheWatchEnded, failures_in_a_row)` answers `AfterAWait` for a lost
or failed connection however many times in a row, because the wait rule's cap bounds that;
`Never` for "the watch was stopped" and "nobody was listening"; and after
`REFUSALS_BEFORE_THE_SCHEDULE_ALONE`, three, `OnTheScheduleAlone(Until::TheProgramStartsAgain)`
for "the mail server would not start watching" and `OnTheScheduleAlone(Until::TheServerAnswersAgain)`
for `NeverStarted`. The six reason strings `imap.rs` breaks its loop with are a table with the
answer each gets on a first failure. `what_the_status_line_says` words `WhatIsRunning { account,
folder, watch, checking_every }` as "Watching Inbox for new mail. Checking every 5 minutes.",
"Waiting 2 minutes to watch Inbox again. Checking every 5 minutes." or "Checking every
5 minutes.", "every minute" for one, the account first when given, through
`trying_again::said_as_a_person_says_it`, made `pub(crate)` so one length of time is not said
two ways; a test holds the lines to no protocol word. `what_a_check_of_them_all_says` is
"Checking for new mail..." for one account and "Checking 2 accounts for new mail..." for more.
`cargo test --lib application::checking_on_a_schedule::` passes with 20, quoted from the run;
`grep -c 'pub mod checking_on_a_schedule' src/application/mod.rs` is 1. `Account::mark_synced`'s
doc says the check calls it and how the row's column is written.

**Task 2, the watch.** `WxUIState::mail_watches: HashMap<String, InboxWatch>` replaces the one
handle: per account, the handle, whether the loop is reading events, the account's own
`WaitBeforeTryingAgain`, `next_watch_at`, and `on_the_schedule_alone: Option<Until>`;
`WxUIState::last_checked` is the schedule's clock. `spawn_mail_watch(app, account_id)` returns
at once for an account on the schedule alone or one whose watch is reading events, stops that
account's previous handle otherwise, and on a worker asks for the inbox as before; the `Err` from
`for_account` and from `watch_folder` each call `what_the_watch_does_next` with
`WhyTheWatchEnded::NeverStarted(e)` before the `return`, `Stopped` calls it with `Ended(reason)`,
and `Changed` and `StillWatching` tell the wait it worked. `Changed` sends
`MailboxChanged(WatchedFolder { account_id, path })`, and the arm checks that account's folder
and not the active account's. `what_the_watch_does_next` takes the lock once, asks the rule with
this failure counted, and writes a wait from the account's own rule with `next_watch_at`, or the
mark, then logs the account, the reason, the count in a row and the wait, and sends the status
line as `Progress` through `say_what_the_watch_is_doing`, the account named when more than one
is enabled. `start_the_watches_whose_wait_is_over` runs on the timer's ten-second block beside
`start_the_download_if_its_wait_is_over`; `check_the_accounts_that_are_due` on the minute look
beside the reminders, building `AccountToCheck` by `From` over the rows, asking `which_are_due`,
and returning at once when the program believes there is no network.
`act_on_what_the_network_did` takes `AppHandles` now; on the back branch it cuts every wait
short, lifts the marks whose reason was the reach, calls `watch_every_enabled_account` and
`check_the_accounts_that_are_due(app, true)`, with the comment saying reads follow the network
and sends follow the mode. The startup section, after `load_module_data`, calls
`watch_every_enabled_account` and `check_every_enabled_account`, skipped during a scan run; the
F9 arm calls `check_every_enabled_account`, which says how many and hands every enabled account
to `spawn_mail_sync(app, accounts, only, wanted)`, one worker walking the list, each account
that went through ending with `mark_synced()`, `update_account_last_sync`, the lifting of a
reach mark, and `say(UIUpdate::MailboxWatchRequested(account.id.clone()))`, the download asked
for once after the loop and only when some account went through. `say_the_watch_is_off` and its
sentence are gone; `grep -c 'will not appear on its own\|say_the_watch_is_off'
src/presentation/wx_app.rs` is 0; `grep -n 'fn spawn_mail_watch\|fn say_the_link_was_refused'
-B 3` shows each under its own doc comment's last lines; `grep -n
'which_are_due\|whether_to_watch_again\|mark_synced' src/presentation/wx_app.rs` shows the
schedule, the decision and the check's end. `cargo test --lib presentation::wx_app::` passes
with 199 before and after, and `presentation::wx_account_manager::` with 14. `bash
scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_app.rs` names
`mail_keeps_arriving_on_its_own` among sixteen. The account editor's interval field is built
through `tf_with_description` with `WHAT_THE_INTERVAL_DOES`, the plan's sentence, and a
`StaticText` beneath it on both channels, held on `Page2Shell` so it hides with the page.

**Task 2, the target.** `tests/mail_keeps_arriving_on_its_own.rs`, seven readings over the
window's shipping half and seven companions, 14 tests: the `Stopped` arm handing
`WhyTheWatchEnded::Ended(` to the decision and the decision asking `whether_to_watch_again(`,
`.next_wait()` and `next_watch_at = Some(`; the `Err` branch after `watch_folder(` naming
`WhyTheWatchEnded::NeverStarted(` before `return;`; the window holding neither the sentence
nor the function; the back branch calling `watch_every_enabled_account(` and
`check_the_accounts_that_are_due(`, and the former sending `UIUpdate::MailboxWatchRequested(`;
the timer's tail reaching both periodic functions, the schedule asking `which_are_due(` over
`AccountToCheck::from`, the wait server calling `spawn_mail_watch(`; the startup section after
`load_module_data(` reaching both; F9 reaching `check_every_enabled_account(`, that function
reading `every_enabled_account(` and `what_a_check_of_them_all_says(`, and the check walking
`account in accounts {`, ending each with its own watch request, marking `mark_synced()` and
`update_account_last_sync(`, and holding `if nothing_went_through {` before the download. Each
companion plants the opposite into a snippet. Two readings in files records name were rewritten
in place: `wired`'s watch reading holds the two failures to `WhyTheWatchEnded::` twice rather
than to the sentence, and 10-05's reading of the check's end matches
`say(UIUpdate::MailboxWatchRequested(` up to the bracket; `wired.rs` stays at 77 and the 10-05
target at 18.

**Task 3, the start that attempts an account, measured.** The harness's header says what a start
does now and what the refusal is. The release binary was built twice, at `6ec0ec60` and at
`7ad596ca`, and started twelve times through the harness against a temporary profile pinned
with `WIXEN_MAIL_DATA`, never against the tester's profile, which was not read. The rows are on
`docs/development/measurements.md` beside the old ones with the old condition dated: cold start
536 ms, the median of 536, 617, 550, 479 and 534 after the build's own 501, against 476 on
2026-09-14; idle at 120 s 373 MB, the median of 373, 373, 373, 375 and 373, the application
58 MB and the tree 315 MB, against 391. The log of one run at `7ad596ca`, quoted: `WARN
wixen_mail::presentation::wx_app: Could not watch the inbox of The measurement account:
Authentication error: No password is saved for The measurement account on this computer.` at
04:58:38.81, then `INFO wixen_mail::presentation::wx_app: The watch on INBOX for The
measurement account ended, 1 in a row (NeverStarted("Authentication error: ...")). Trying the
watch again in 30 seconds.`, the second refusal at 04:59:18.65 with `Trying the watch again in
1 minute.`, and at 05:00:18.86 `The watch on INBOX for The measurement account is left to the
schedule alone until TheServerAnswersAgain`. The check's refusal was spoken once, `Speaking:
Error: Authentication error: No password is saved ...`, and no download line appears. The
changelog entry for #37 under Unreleased, Fixed, with the tester's words, what changed, the date
the field was written, and Known limitations: no real server has dropped a watch, a dead
connection can go unnoticed for up to 29 minutes and the schedule covers that, a connection
limit per account is unknown, the schedule does not run without a network, an account with no
password saved is told so at every check, nobody has heard the status lines. The F9 row on
`docs/KEYBOARD_SHORTCUTS.md`. Ledger 446 closed, 447, 64 and 65 corrected by addition, 525 to
528 added. `cargo test --test house_style` 74, `--test every_number_carries_its_command_and_its_date`
25, `--test the_planning_files_agree_with_themselves` 16, `--test the_words_that_say_nothing`
9; carriage returns on every page touched 0.

## Task commits

| Commit | What |
|---|---|
| `6195ed77` | test(10-06): the red half of task 1, seventeen tests by module path, three green on arrival, said |
| `c3e21072` | feat(10-06): the module's bodies, the wording shared, `mark_synced` told who calls it, three records measured |
| `00b4590e` | test(10-06): the red half of task 2, seven readings bare, `wired`'s and 10-05's readings rewritten in place and named |
| `6ec0ec60` | feat(10-06): the watch, the schedule, the start, the status line, the editor's sentence, six records measured, one corrected and measured again, the changelog and the F9 row |
| `9805dc34` | test(10-06): a second red, the reading of the check gaining the download clause, named bare; the harness's header |
| `7ad596ca` | fix(10-06): the download only after a check that went through; the watch's wait line says what it is; one record measured, two measured again |
| `c8eedc17` | docs(10-06): the rows re-taken, the ledger, the changelog's last line |
| `2da50b6b` | Merge 10-06 into `main` |

Branch `the-watch-that-keeps-going` from `main` at `9cea7c87`. Not pushed.

## Honest RED and GREEN

Three red commits, each on the branch, each running and failing exactly what it named and
nothing else, which the gate said in `red` mode. Task 1's red names seventeen tests by module
path and says three were green on arrival: an account checked within its interval is not due,
because the stub answered nobody is; a watch somebody stopped is not tried again, because the
stub answered never; and the `From` was written whole, because there is no wrong value for it to
answer. Task 2's first red names seven readings bare and two rewritten readings bare,
`test_every_check_ends_by_starting_the_download` and
`test_a_mail_watch_that_ends_is_reported_rather_than_only_logged`; its seven companions were
green on arrival because they read snippets. Task 2's second red, `9805dc34`, names one reading,
the check's, which gained the download clause after the release binary showed the first draft
asking for the download after a failed check; it is the red the running program demanded, and
its green is `7ad596ca`. Task 3 changed documents and a harness header, which `CLAUDE.md` lists
among the exceptions, and no format test changed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app::` (199) and sixteen coupled targets, the new one among them, on the two greens of task 2 |
| `src/application/mod.rs` | `--lib application` whole, on task 1's red |
| `src/application/checking_on_a_schedule.rs`, `trying_again.rs`, `src/data/account.rs` | their own module paths |
| `src/presentation/ui_types.rs`, `wx_account_manager.rs` | their own module paths and coupled targets |
| the new target, `wired`, the 10-05 target, `the_numbers_the_targets_ask_for` | their own targets as changed, and coupled |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | no scoped target; the whole-tree guards and the document readers |

`scripts/check.sh all` on the branch at `c8eedc17`, run once with its output to a file and the
exit status read directly, never piped: exit 0, 8,032 passed and none failed over 74 result
lines, 435 s from 05:11:16Z to 05:18:31Z, the release build included. Thirty-four more than
10-05's 7,998: the module's 20 and the target's 14, and nothing else moved. Inside the 275 s to
654 s band on `docs/development/measurements.md`. `main`'s hook ran `all` again on the merge:
8,032 and none failed, 430 s from 05:18:59Z to 05:26:09Z. The keyring race (ledger 374) and the
tab row reading's intermittent failure (10-01.1, 10-02.1) did not appear in any run.

## Guard records

902 records by the TOML reader before, 912 after; census 802 + 100 before, 802 + 110 after, the
line at `guards/guards.toml:84` moved in each commit that added records. Ten new, one corrected
by hand for an indent and measured again, two measured again, every one measured through
`scripts/guards.sh --remeasure` with `WIXEN_TEST_THREADS` untouched and the counts written by the
runner. Seventy-three records name `wx_app.rs` now (66 before); seven name the new target.

| Record | File and suite | Break | Red | Run |
|---|---|---|---|---|
| a disabled account is never due a check | `checking_on_a_schedule.rs`, the library | the `enabled` filter dropped | 1 | rebuild 45 s, run 49 s |
| a watch somebody stopped is not started again from the arm | `checking_on_a_schedule.rs`, the library | the stop read as a lost connection | 2 | rebuild 45 s, run 49 s |
| a stored interval of nought is a minute, not every tick | `checking_on_a_schedule.rs`, the library | the clamp dropped | 2 | rebuild 41 s, run 48 s |
| a watch that ends is asked about, not broken from | `wx_app.rs`, the target | the Stopped arm breaks without asking | 1 | rebuild 15 s, run 1 s; again 15 s and 1 s |
| a watch that never started is asked about before the branch returns | `wx_app.rs`, the target | the Err arm returns without asking | 1 | rebuild 15 s, run 1 s |
| the network coming back starts the watches at once | `wx_app.rs`, the target | both calls dropped from the back branch | 1 | rebuild 16 s, run 1 s |
| the timer checks the accounts that are due | `wx_app.rs`, the target | the schedule arm taken off the timer | 1 | rebuild 16 s, run 1 s |
| the schedule reads the interval off the account row | `wx_app.rs`, the target | the accounts built by hand with five minutes | 1 | rebuild 19 s, run 1 s |
| a start checks without a keystroke | `wx_app.rs`, the target | both calls dropped from the startup section | 1 | rebuild 17 s, run 1 s |
| a check that went through nowhere asks for no download | `wx_app.rs`, the target | the returning branch dropped | 1 | rebuild 14 s, run 1 s |
| the check's worker opens its cache with how much message text stays (corrected) | `wx_app.rs`, the 10-03 target | as before, one indent to the right | 1 | rebuild 15 s, run 2 s |
| every check ends by starting the download (measured again) | `wx_app.rs`, the 10-05 target | as before | 1 | rebuild 16 s, run 1 s |

**The count check did not fire.** `checking_on_a_schedule.rs` was written with its twenty tests
before its records; the target held fourteen when its records were written and the second red
rewrote one in place; `wx_app.rs` stayed at 199, `wired.rs` at 77, the 10-05 target at 18,
`wx_account_manager.rs` at 14, `trying_again.rs` at 9, `account.rs` at 24 and named by no record,
`the_numbers_the_targets_ask_for.rs` at 16. **One record's break moved without a line of it
changing:** the 10-03 record on the check's cache line quoted three lines that `cargo fmt`
indented by four spaces when the check's body went inside the walk over the accounts, and
`test_every_guard_record_still_names_one_place_in_the_tree` refused the tree until the quote was
corrected; the record was measured again, not only re-quoted. Observation 661 in the skill log.
The count check is green on `main` at `2da50b6b`.

## What the tree contradicted

Every command in the plan's eight premises was re-run against `main` at `9cea7c87` before
anything was built. Every line number in the plan had moved by 200 to 700 lines, as the plan
said they would; the shapes held, with these exceptions.

1. **The measurement account is refused by the credential store, not the port.** Premise 7 and
   task 3 say a start dials `127.0.0.1` on a closed port and is refused at once. The profile
   stores no password, because an empty password is a request to forget and the harness cannot
   store one (ledger 374), so `for_account` answers "No password is saved" before anything is
   dialled, for the check and for the watch alike. The rows and the ledger say which refusal was
   read; a refusal at the socket is ledger 526.
2. **The first draft asked for the download after a failed check.** With the check walking a
   list, the download request after the loop went out whether or not any account went through,
   and the release binary showed a download failing the same way as the check and waiting on its
   own. 10-05's tree never asked for the download after a failed check. A flag holds it to a
   check that went through, with a reading, a companion and a record.
3. **`whether_to_watch_again` answers three things.** The plan's `Never` covers both a stop
   somebody asked for and a server that refused three times; the window stops a watch itself
   whenever it replaces one, and an arm that marked the account on that stop would mark it while
   the fresh watch was starting. `Never` writes nothing; `OnTheScheduleAlone(Until)` writes the
   mark and names what lifts it.
4. **A reading matched a word the comment beside the code repeated.** The start-failure reading
   looked for `return` and found it first in the comment saying the question is asked before the
   return; it matches `return;` now. Observation 660 in the skill log.
5. **The wait's wording is the download's.** `what_to_say_before_waiting` says the mail server
   could not be reached; a watch refused by the credential store reached no server, and the log
   said it had. The watch's log line says the reason, the count in a row and the wait.

Two things the plan did not name that the tree needed: `MailboxChanged` carries the account,
because two accounts each have an `INBOX` and the arm read the active account; and the account
editor's `Page2Shell` holds the new note so the page hides it with the field.

## Deviations from plan

**1. [Decision] The five points above.** Ledger 527, so 10-07 reads the watch from the tree.

**2. [Decision] A watch already reading events is left alone by a request.** The plan has every
request stop the previous watch and start another; with a check every five minutes that would
open and close the IDLE connection per check for nothing, and the status line would say
"Watching" afresh each time. `is_watching` is the loop reading events on an open handle;
`Changed` and `Stopped` clear it.

**3. [Decision] The schedule follows the network.** The plan's timer arm asks `which_are_due`
whatever the network; with no network every due check would say an error, spoken, every
interval for as long as the network was gone, which is guardrail 5's flood. The arm is handed
`the_network.borrow().there_is_a_network()` and returns at once without one; the network coming
back checks the due accounts.

**4. [Decision] `mark_synced` and the column, not `save_account`.** `save_account` writes the
credential store on every call, and a check runs every five minutes; `update_account_last_sync`
existed, called by its own tests, and writes the column alone. The column is written and read
by nothing, ledger 528.

**5. [Decision] One `InboxWatch` per account, not `mail_watches` and `watch_waits` apart.** The
handle, the reading flag, the wait, the next attempt and the mark are one record, because a
request has to read all of them under one lock to decide whether to start.

**6. [Decision] The F9 line for several accounts is "Checking 2 accounts for new mail...".** The
plan's "Checking 2 accounts..." does not say for what; a person with two accounts hears the
count and the reason in that order.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; `cargo fmt` formatted before each commit; the only `sed`, `awk`, `grep` and Python in the
session read files, logs and `guards.toml`, and the one script that ran for minutes copied logs
out of temporary profiles. Commit messages were written to the scratchpad and passed with `-F`.
Carriage returns measured with `tr -cd '\r' | wc -c` on every changed file before each commit:
zero on each. No em dash in any file this plan wrote, measured with `grep -c` for the byte
sequence: zero on each. `Cargo.toml` and `Cargo.lock` untouched; no crate added. No AI
attribution anywhere. The version stays `1.0.0-alpha.1`. The tester's profile was not read; the
release binary was started only through the harness against temporary profiles.

## Threat register

T-10-20: the watch's wait is `WaitBeforeTryingAgain` per account to the thirty-minute cap;
three refusals of the watch put the account on the schedule alone; every attempt is logged with
the account, the reason and the wait; a record holds the timer arm and one holds the decision.
T-10-21: accepted; one watch and one working session per account, ledger 67 and 525 record it
as unmeasured; the tester has one account. T-10-22: `the_interval_of` clamps, with the record
whose break makes nought every tick. T-10-23: no line the watch or the decision logs names a
subject or a body; the reasons logged are the IMAP module's strings and the credential store's
sentence. T-10-24: accepted; the schedule and the watch resume when the network returns whether
or not offline mode is on, said in the changelog and in the doc comment. T-10-SC: no crate
added. No new surface outside the register: the schedule and the watch reach the same check and
the same download.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash outside a JSON quote; 524
before, 528 after; 495 open before, 498 after; 29 fixed before, 30 after.

| id | kind | what |
|---|---|---|
| 446 | closed | the start attempts the account and what it says was read; the refusal is the credential store's |
| 447 | corrected by addition | idle includes a start that attempts an account and a watch tried again; still no live connection |
| 64, 65 | corrected by addition | the restart and the held connection meet the tester's account unasked |
| 525 | unrun-verify | what only the tester's account and ear settle for #37 |
| 526 | unrun-verify | a refusal at the socket is still read by nobody |
| 527 | deviation | the departures above |
| 528 | stub | `last_sync` written by every check and read by nothing |

## Known stubs

**`Account.last_sync` is written and read by nothing, on purpose, and this is the plan that says
so.** Every check that goes through marks the worker's copy and writes the row's column; the
schedule keeps its own session clock and a start checks every enabled account whatever the row
says. Ledger 528. Everything else this plan built is reached: `checking_on_a_schedule`'s four
functions from the timer, the decision and the F9 arm; `spawn_mail_watch` from the
`MailboxWatchRequested` arm, which the check's end, the start, the network coming back and the
wait server all send; `what_the_watch_does_next` from three failure paths.

## The issue

#37 closed from `2da50b6b` with the plan's comment, the ear list above and what the changelog
says under Known limitations. Closing an issue is not a publish.

## What the tester's account could settle, and what the harness proved

Nothing in this plan is claimed against a real provider or a real ear. The readings proved the
window asks the decision on both failure paths, restarts on the network, schedules on the row's
interval, checks at the start and walks every account; the module's tests proved every decision;
the release binary against the measurement profile proved the start attempts the account, the
watch is refused three times at the wait rule's cadence plus the timer's ten seconds and is then
left to the schedule alone, and a failed check starts no download. What Gmail does with a watch
per account, a restart after a drop and a check every five minutes, and what any of it sounds
like, is ledger 525. Whether a provider drops IDLE and how the restart carries mail over hours is
the tester's, said in the close comment.

## Not done here, on purpose

No `MAIL` requirement is ticked by hand; MAIL-04 is in the frontmatter for 10-07's closing
read, on the phase's rule. Roadmap criterion 4 closes structurally and is read by 10-07. The
four documents 10-07 writes are untouched; ledger 525 holds the listening lines for them.

## For 10-07

- MAIL-04's `[D]` lines each have a named test or reading in the coverage block above; its
  `[S]` line is untouched and is ledger 525.
- The status line's three sentences are `checking_on_a_schedule::what_the_status_line_says`;
  the pages should quote them from there. They go out as `Progress`, shown always and spoken
  under Say every step; the check's refusal is an error and is always spoken.
- The interval is per account on the account editor, "Check Interval (min)", 1 to 60, 5 by
  default, with `WHAT_THE_INTERVAL_DOES` under it; there is no interval on the Settings screen,
  and the README's decision 1 stands unless Pratik says otherwise.
- Reads follow the network; sends follow offline mode. The privacy page's sentence about what
  runs unasked should say a watch per account holds a connection open for as long as the
  program runs, that a check runs every interval, and that both resume when the network returns
  without Go Back Online.
- `docs/development/measurements.md` has the cold-start and idle rows twice, the old condition
  dated and the new one named; the alpha page's line about the measurement account should say
  it is refused by the credential store, ledger 526.
- The changelog entry is complete; 10-07 dates nothing here.

## Self-Check: PASSED

`src/application/checking_on_a_schedule.rs` and `tests/mail_keeps_arriving_on_its_own.rs` exist
with 20 and 14 test functions; `grep -c 'pub mod checking_on_a_schedule' src/application/mod.rs`
is 1; `grep -c 'will not appear on its own\|say_the_watch_is_off' src/presentation/wx_app.rs` is
0; `grep -n 'MailboxWatchRequested' src/presentation/wx_app.rs` shows the arm, the send from
`watch_every_enabled_account` and the send at the check's end; `guards/guards.toml` holds 912
records by the TOML reader and the census line says 110; `docs/changelog.md` holds the line
beginning "**Mail keeps arriving on its own for as long as the program runs.**";
`.planning/WINDOWS.md` holds 525 to 528 in both halves and 446 fixed in both. Commits
`6195ed77`, `c3e21072`, `00b4590e`, `6ec0ec60`, `9805dc34`, `7ad596ca`, `c8eedc17` and
`2da50b6b` are in `git log --oneline` on `main`. Issue #37 reads CLOSED by `gh issue view
--json state`.

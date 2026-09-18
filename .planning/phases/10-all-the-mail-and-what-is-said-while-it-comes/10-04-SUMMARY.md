---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 04
subsystem: what is spoken while things are fetched, the Feedback tab, the update handler, guards
tags: [settings, announcements, progress, feedback-tab, new-mail-sound, status-bar, guards, changelog]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-03: the shape of a string-valued setting (module, field, older-file test, page reader, a reading target coupled by a measured record); 10-01.1: each later Settings page painted inside APage::built, so a control added to Feedback lands on a page whose check boxes are check boxes"
provides:
  - "application::what_is_said_while_fetching: HowMuchToSay, WhatArrived, EveryStep or ErrorsOnly; ALL in that order; label Say what arrived, Say every step, Errors only; as_stored what-arrived, every-step, errors-only; from_stored answering WhatArrived for anything else; Default WhatArrived; Kind, Progress, Result, Error or Answer; is_spoken: errors and answers always, results unless ErrorsOnly, steps only under EveryStep; offered_index; WHILE_FETCHING_LABEL and WHAT_THE_CHOICE_LEAVES_ALONE"
  - "AppConfig.announce_while_fetching: String, serde default what-arrived, an older settings file answering the same"
  - "wx_settings: a While fetching section at the end of the Feedback tab holding a labelled choice and the sentence under it; read_the_feedback_page, extracted from read_settings' inline block, writes the field; FeedbackTabControls::announce_while_fetching public"
  - "Accessibility::how_much_to_say and set_how_much_to_say, set at startup and on OK beside the feedback settings"
  - "UIUpdate::Progress(String), shown always and spoken at Low on the topic progress only when is_spoken(Kind::Progress); UIUpdate::WhatArrived { what }, shown, signalling NewMail, spoken at Normal on the topic arrived when is_spoken(Kind::Result); UIUpdate::ModuleSyncFinished(String), signalled as SyncComplete with the counts as the detail when results are spoken; StatusUpdated spoken at Normal; WholeFolderProgress folded into Progress"
  - "mail_sync::what_arrived(&[(String, usize)]) -> Option<String>: each folder that received something with its count through how_many, None for nought"
  - "send_progress beside send_status; the_counts_if_results_are_spoken for the three module completions"
  - "tests/progress_is_shown_and_results_are_said.rs: six readings over the window's shipping half with seven companions, coupled to wx_app.rs by two measured records"
  - "guards/guards.toml: five records measured, three rewritten onto moved lines and re-measured, twenty re-measured; 897 records, census 802 + 95"
  - "docs/changelog.md: the entry under Unreleased, Changed; docs/manual-accessibility-pass.md items 42 and 43; ledger 520 and 521; #38 closed from the merge commit"
affects: [10-05 (the runner's chunk lines go out as Progress and its per-check counts as one WhatArrived, or fold into the check's; the whole-folder and missing-text commands it retires already send Progress), 10-06 (the watch's and the schedule's status lines are steps unless they answer a key; the status line's three states are answers), 10-07 (reads MAIL-05's [D] lines from here and writes the listening lines ledger 521 names)]

actuals:
  tokens: 22480
  tasks: 3
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A line's kind is decided where the line is made, as a variant, and the arm asks the level rather than reading a setting: a new line cannot arrive on the spoken channel by accident, and a reading over the file holds the openings"
    - "A reading target over the update handler with one companion per claim that plants the opposite into the real text and refuses to plant into a text without the anchor"
    - "A per-event row and a level are two independent settings: the row decides whether an event's tone and word are heard, the level decides whether its counts are; the modules' completions carry the counts as the detail under the level"

key-files:
  created:
    - src/application/what_is_said_while_fetching.rs
    - tests/progress_is_shown_and_results_are_said.rs
  modified:
    - src/application/mod.rs
    - src/application/mail_sync.rs
    - src/application/asking_for_a_whole_folder.rs
    - src/data/config.rs
    - src/presentation/accessibility.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_settings.rs
    - tests/every_event_has_a_control.rs
    - tests/a_whole_folder_moves_both_bounds.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/manual-accessibility-pass.md
    - .planning/WINDOWS.md

key-decisions:
  - "The new-mail signal lives in the WhatArrived arm, because the worker holds no accessibility handle; the plan's grep criterion for a signal inside spawn_mail_sync could not be met as written"
  - "The result sentence counts through how_many, Inbox, 3 new messages, rather than the plan's Inbox, 3 new, so a listener hears what the number counts"
  - "The tasks and notes syncs finish through a new ModuleSyncFinished update; the notes sync's two answers that no sync ran stay on the answer channel because they answer the key"
  - "The whole-folder request's closing report goes out as a step with its progress, because the loop hands over one kind of line and the command retires with 10-05; the changelog says so"
  - "The refusal test's allowance for two progress openings nobody sends is removed rather than emptied, so no allowance waits for a line somebody sends"

patterns-established:
  - "A guard record whose companion refuses to plant into a broken tree reddens with its reading; the record names both, and the runner is what says so"

requirements-completed: [MAIL-05]

coverage:
  - id: D1
    description: "How much is said while mail and the other modules are fetched is a choice on the Feedback tab with three answers in plain words, Say what arrived by default, stored as a string an older settings file falls back to, read at startup and on OK"
    requirement: MAIL-05
    verification:
      - kind: unit
        ref: "src/application/what_is_said_while_fetching.rs#test_the_choices_are_what_arrived_then_every_step_then_errors_only"
        status: pass
      - kind: unit
        ref: "src/application/what_is_said_while_fetching.rs#test_the_default_says_what_arrived"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_a_settings_file_written_before_these_existed_reads_the_way_it_should"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: unit
        ref: "src/presentation/accessibility.rs#test_how_much_to_say_reports_what_was_just_set_rather_than_the_default"
        status: pass
      - kind: integration
        ref: "tests/every_event_has_a_control.rs#test_every_event_is_reachable_and_keeps_what_it_was_given"
        status: pass
    human_judgment: false
  - id: D2
    description: "A step is shown and spoken only under Say every step; what arrived is said once with counts and never when nothing arrived; an error is spoken whatever was chosen; the new-mail sound plays when a check found mail; Settings saved and the other answers are heard above a running check"
    requirement: MAIL-05
    verification:
      - kind: unit
        ref: "src/application/what_is_said_while_fetching.rs#test_under_what_arrived_a_step_is_not_spoken (and the other eleven cells)"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#test_a_check_that_found_nothing_says_nothing_about_what_arrived"
        status: pass
      - kind: integration
        ref: "tests/progress_is_shown_and_results_are_said.rs#test_a_step_is_spoken_only_when_every_step_was_asked_for"
        status: pass
      - kind: integration
        ref: "tests/progress_is_shown_and_results_are_said.rs#test_the_mail_checks_lines_are_steps_and_what_arrived_goes_out_once_after_the_loop"
        status: pass
      - kind: integration
        ref: "tests/progress_is_shown_and_results_are_said.rs#test_the_new_mail_sound_plays_when_a_check_found_mail_and_not_when_the_watch_woke"
        status: pass
      - kind: integration
        ref: "tests/progress_is_shown_and_results_are_said.rs#test_settings_saved_and_the_other_answers_are_said_at_normal"
        status: pass
      - kind: integration
        ref: "tests/progress_is_shown_and_results_are_said.rs#test_no_step_rides_the_answer_channel"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a step is spoken only when every step was asked for', 'the new-mail sound means mail arrived, not that the watch woke', 'a check that found nothing says nothing about what arrived', measured 2026-09-17, each reddening exactly the tests named"
        status: pass
    human_judgment: false
  - id: S1
    description: "What a check of the tester's 50 folders says under each of the three answers, which sentence is a step and which a result by ear, whether the one result sentence reads as an ending, whether the sound and the sentence read as one event, and whether Settings saved is heard during a check"
    requirement: MAIL-05
    verification: []
    human_judgment: true
    rationale: "Ledger 521 and items 42 and 43 on the listening page. The sorting was done by reading each line's words; nothing here met a real account, and nobody has listened"

duration: 2h08m
completed: 2026-09-18
status: complete
---

# Phase 10 Plan 04: How much is said while mail is fetched is a setting Summary

**How much is said while mail and the other modules are fetched is a choice at the end of the
Feedback tab under While fetching: Say what arrived, Say every step, Errors only, Say what arrived
by default and what a settings file from before this existed answers. Every line a fetch says on
the way, Connecting, Checking a folder, Loading a folder, a sync requested, a chunk downloaded,
goes out as `UIUpdate::Progress`, shown on the status bar always and spoken only under Say every
step. What a check brought goes out once, after the loop, as `UIUpdate::WhatArrived`, folder by
folder with its count, never when nothing arrived; its arm is where the new-mail sound is
signalled now, so the sound means mail arrived and not that the watch woke. The four module
syncs carry their counts as the Sync finished event's detail under the level. An error is spoken
whatever was chosen. `StatusUpdated`, which is the answer channel now, is spoken at Normal, so
Settings saved is heard above a running check. #38 closed from the merge commit; which sentence is
a step and which a result, by ear, is the tester's.** Nothing pushed.

## Performance

- **Duration:** 2 h 8 min from the branch at 22:33:21Z to the merge at 00:41:40Z, plus the reading
  before the branch; of that, about 45 minutes were guard measurement across six runs (2,742 s
  summed from the runners' `timed:` lines, 773 + 16 + 1,483 + 190 + 170 + 99 rounded), about 27
  minutes the six hook runs (254 s, 317 s, 232 s, 292 s, 197 s, and 330 s for the refused first
  attempt at task 1's green), and about 15 minutes the two whole gates (442 s and 431 s)
- **Started:** 2026-09-17T22:33:21Z (the branch; the first commit 22:47Z)
- **Merged:** 2026-09-18T00:41:40Z at `19a10706`
- **Tasks:** 3
- **Files modified:** 16 (2 created)

## What landed

**Task 1, the level.** `src/application/what_is_said_while_fetching.rs`. `HowMuchToSay` is
`WhatArrived`, `EveryStep` or `ErrorsOnly`; `ALL` offers them in that order; `label` says "Say
what arrived", "Say every step", "Errors only", none naming a mechanism; `as_stored` writes
`what-arrived`, `every-step`, `errors-only`; `from_stored` answers `WhatArrived` for anything else,
held against `""`, `"   "`, `"everything"`, `"0"`, `"every step"` and `"ERRORS-ONLY"`, because
every step puts somebody back to the verbosity the issue was filed about and errors only silences
arrivals; `Default` is `WhatArrived`, derived, with `#[default]` on the variant because clippy's
`derivable_impls` refused a manual impl under `-D warnings`. `Kind` is `Progress`, `Result`,
`Error` or `Answer`. `is_spoken`: errors and answers always, a result unless `ErrorsOnly`, a step
only under `EveryStep`, one test per cell of the twelve. `offered_index` selects the stored
choice's entry and the default's for a garbled one. `WHILE_FETCHING_LABEL` is "&While mail and the
other modules are fetched, say:", Alt+W, which no other control on the Feedback tab claims (the
tab's mnemonics are A, S, T, U, P, B, E, F, I and D; `test_no_two_controls_in_one_dialog_claim_the_same_alt_key`
green, though it reads literal labels and cannot see a constant, so the letter was checked by
hand). `WHAT_THE_CHOICE_LEAVES_ALONE` is the plan's sentence: "Errors are always said. The sound
for new mail follows its own row above." `cargo test --lib application::what_is_said_while_fetching::`
passes with 19.

`AppConfig.announce_while_fetching: String` at `config.rs:455-465`, `#[serde(default =
"default_announce_while_fetching")]`. The older-file test gained the field in its list and two
assertions: the parsed value is `what-arrived`, with the sentence about the two other answers, and
`from_stored` of it is `WhatArrived`. `cargo test --lib data::config::` passes with 68, as before.

The Feedback tab ends with a `section(panel, "While fetching")` after "One event at a time",
holding a `labelled_choice` with `WHILE_FETCHING_LABEL`, the spoken name the label without the
mnemonic and the colon, the three labels from `HowMuchToSay::ALL`, selected through
`offered_index`, and a `StaticText` under it carrying `WHAT_THE_CHOICE_LEAVES_ALONE` on both
channels (`wx_settings.rs:2960-2984`). The inline Feedback block in `read_settings` is
`read_the_feedback_page(w: &FeedbackTabControls, cfg: &mut AppConfig)` now, on
`read_the_permissions_page`'s pattern, reading what it read before and writing
`cfg.announce_while_fetching` through `ALL` and the selection (`:3308-3341`); `read_settings`
keeps its `if_built` guard and calls it, so a page nobody showed leaves the stored value alone.
The field is `pub` on `FeedbackTabControls`, as `PermissionsTabControls::message_text_kept` is.

`Accessibility` holds `how_much_to_say: Mutex<HowMuchToSay>` beside the feedback settings, with
`how_much_to_say()` and `set_how_much_to_say(level)` and one test that a set value is read back
(`accessibility.rs:34-39`, `:60-62`, `:300-317`). `wx_app.rs` calls the setter at startup after
`set_feedback_settings`, whether or not a file was read (`:1189-1195`), and in `handle_settings`
on OK beside it (`:16880-16884`); `grep -n 'set_how_much_to_say' src/presentation/wx_app.rs`
answers those two.

**The settings guards, quoted from the runs.** At the red commit, between the field and the
control: `test_every_setting_somebody_can_change_is_offered_by_a_screen` "1 setting(s) are stored
and survive a restart and no screen offers any of them: announce_while_fetching" and
`test_every_setting_somebody_can_change_is_read_by_something` "1 setting(s) can be changed and are
read by nothing: announce_while_fetching", 65 passed and 3 failed with the older-file test. After
the control, the reader and the two call sites: 68 passed.

**Task 1, the reading.** No new target: `tests/every_event_has_a_control.rs` already builds the
real dialog with the Feedback page shown and reads the settings back through `read_settings`, so
it gained a sub-check inside its one `wxdragon::main` test,
`the_level_chosen_while_fetching_is_what_ok_writes_back`: the three entries offered in order, the
selection on the default over `AppConfig::default()`, and each level chosen in turn and read back
as its stored form. The file's test count stays 1, so the five records naming it were not
flagged. `cargo test --test every_event_has_a_control` passes with 1.

**Task 2, the kinds.** `UIUpdate::Progress(String)`, `UIUpdate::WhatArrived { what: String }` and
`UIUpdate::ModuleSyncFinished(String)` on `ui_types.rs:705-733`, with `WholeFolderProgress`
removed. The `Progress` arm (`wx_app.rs:17625-17639`) keeps `status_message`, writes the status
bar, and announces at Low on the topic `progress` inside
`if a11y.how_much_to_say().is_spoken(Kind::Progress)`; it is not registered quiet. The
`WhatArrived` arm (`:17640-17659`) writes the status bar, signals `FeedbackEvent::NewMail` with
no detail, and announces at Normal on the topic `arrived` when `is_spoken(Kind::Result)`. The
`ModuleSyncFinished` arm (`:17660-17668`) signals `SyncComplete` with
`the_counts_if_results_are_spoken(a11y, said)`, and the contacts and calendar completions pass
the same. The `StatusUpdated` arm announces at Normal on `status`, with the comment saying why
(`:17604-17624`). The `MailboxChanged` arm signals nothing, with the comment saying where the
signal went and why (`:18371-18380`). `send_progress` sits beside `send_status`, which gained a
doc comment saying it is the answer channel (`:14670-14715`).

Sorted where the lines are made. `spawn_mail_sync`: "Connecting to", the folder count, "Checking
{}...", `what_the_folder_sync_did` and "Mail check finished" are `Progress`; each folder's name
and `fetched` are pushed to `arrived`; after the loop, `what_arrived(&arrived)` is sent as
`WhatArrived` when `Some` (`:21869-22080`). `check_pop_mail`: "Connecting to" is `Progress`, the
result is `WhatArrived` when `result.fetched > 0` and `Progress` otherwise (`:19870`,
`:19951-19960`). The whole-folder request's lines and the missing-text fetch's progress lines are
`Progress` (`:21780`, `:21404`); the missing-text fetch's closing `what_the_fetch_did` stays on
the answer channel. The four sync requests from the module's own Sync, the three from the Tools
menu and "Checking for new mail..." from F9 go through `send_progress` (`:4161-4173`, `:4480`,
`:5036`, `:5097-5105`); "Loading {}..." on a folder change is `Progress` (`:2900`). The tasks
sync finishes with `ModuleSyncFinished("Tasks: ...")` (`:22304`) and the notes sync with
`ModuleSyncFinished("Notes: ...")` when it ran (`:22393`); its two answers that no sync ran stay
`StatusUpdated`. `grep -c 'UIUpdate::Progress(' src/presentation/wx_app.rs` is 12 and
`grep -c 'send_progress('` 10 (one of them the definition);
`grep -n 'a11y.signal(FeedbackEvent::NewMail'` answers one line, `17652`, in the `WhatArrived`
arm.

`mail_sync::what_arrived` (`mail_sync.rs:174-198`) keeps the folders with a count, words each as
"{folder}, {how_many(count, "new message")}", joins them with "; " and answers `None` for none:
"Inbox, 3 new messages; Work, 1 new message" for three folders of which one received nothing. Two
tests. `cargo test --lib application::mail_sync::` passes with 149 where 10-03 left 147.

`test_a_refusal_is_not_written_to_the_status_line` was rewritten in place: the `PROGRESS`
allowance of `"No new mail` and `"Nothing to send` is gone, since `grep -rn` finds neither string
in `src/` outside the test itself, and its comment says why the channel matters now, the topic,
and what mattered until 2026-09-17, the priority. `cargo test --lib presentation::wx_app::` passes
with 199 by the count check's count, as before (195 `#[test]` and 4 `#[tokio::test]`).

**Task 2, the reading.** `tests/progress_is_shown_and_results_are_said.rs`, six readings over the
window's shipping half through `what_ships` and seven companions, 13 tests. The readings: the
`Progress` arm shows, asks `is_spoken(Kind::Progress)` before it speaks, and speaks at Low off
`"status"`; the `WhatArrived` arm shows, asks `is_spoken(Kind::Result)` before it speaks, speaks
at Normal off `"status"`, and signals `NewMail`; the `StatusUpdated` arm speaks at Normal; each of
the mail check's five lines is sent as the nearest `UIUpdate::` before it, which must be
`Progress(`, and `WhatArrived` appears once in `spawn_mail_sync`, after "Mail check finished";
the `MailboxChanged` arm carries no `NewMail` and the shipping half signals it from one place;
and every `send_status(` and `UIUpdate::StatusUpdated(` call, read to its closing bracket,
contains none of `"Checking `, `"Connecting `, `"Loading `, `"Syncing `, `"Fetching ` or
`sync requested`, with the count of calls read required above 40 (it read 107 at the merge, 73
`send_status(` and 34 `StatusUpdated(`, the definition's and the handler arm's among them). The
companions plant the ask removed, a result at Low, an answer at Low, the per-folder line under
`StatusUpdated`, the signal back where the watch wakes, and a planted `send_status("Checking the
planted folder...")`, each through a helper that refuses to plant into a text without the anchor;
the seventh hands the reading a text with one call and requires "reading is broken".
`tests/a_whole_folder_moves_both_bounds.rs` had its reading of the retired topic rewritten in
place onto the step the request sends and the `Progress` arm's ask, eight tests before and after.

**The changelog.** One entry under `[Unreleased]`, Changed, beginning "**How much is said while
mail and the other modules are fetched is your choice.**": the tester's words from #38 and the
build, the choice with its three answers and the default, when it is read; what each answer does,
step by step; that the sound for new mail plays when a check found mail rather than when the
server said a folder changed, and that Settings saved and the other answers are heard above a
check; Known limitations: nobody has heard any of it, which sentence is a step and which a result
is a judgement by ear that the listening page asks for, and the two retiring commands say their
progress and the whole-folder closing report as steps until 10-05. It landed in task 1's commit
on the rule that the entry goes with the setting, and task 2 extended it. The version stays
`1.0.0-alpha.1`.

**The listening page.** Items 42 and 43 at the end of section A of
`docs/manual-accessibility-pass.md`: a check of many folders under each of the three answers,
and Settings saved pressed during a check, each with its technology and its ledger entries in the
page's shape.

## Task commits

| Commit | What |
|---|---|
| `d310ea9e` | test(10-04): the red half of task 1, twenty named by module path with the count check bare, sixteen in the module, three in config, one in accessibility |
| `85ee0c52` | feat(10-04): the level, the field, the Feedback tab choice, the extracted page reader, the two call sites, the sub-check, two records measured, seven re-measured, the OK-path record rewritten, the changelog entry |
| `0938b003` | test(10-04): the red half of task 2, the target's eleven readings bare, two in mail_sync by module path, the whole-folder reading rewritten, the count check bare |
| `9adf41a4` | feat(10-04): the three variants and their arms, every path sorted, what_arrived, the refusal test rewritten, three records measured, three rewritten and re-measured, twelve re-measured, the changelog entry extended |
| `b5bccce8` | docs(10-04): the listening lines, ledger 520 and 521 |
| `19a10706` | Merge 10-04 into `main` |

Branch `what-is-said-while-fetching` from `main` at `fb0e8bce`. Not pushed; 149 commits unpushed
before the commit that lands this summary, by `git rev-list origin/main..HEAD --count` on `main`
at `19a10706`.

## Honest RED and GREEN

Task 1's red named twenty: sixteen of the module's nineteen tests by module path, the older-file
test as cargo reports it (`data::config::permission_tests::...`), the two settings guards, the
accessibility test, and the count check bare for `accessibility.rs`'s seven records. The gate in
`red` mode ran exactly those red and nothing else, in 254 s, after a first attempt was refused by
clippy for a manual `Default` impl (`derivable_impls`), which became the derive with `#[default]`
on the wrong variant. Three cells of the table were green on arrival, because a stub that says
nothing is ever spoken agrees with the three cells that expect silence; the commit says so and the
green rewrote the stub they agreed with. The green ran the four scoped filters, the nine targets
coupled to `wx_settings.rs` and the fifteen coupled to `wx_app.rs`, in 317 s, on its second
attempt: the first was refused by `test_every_guard_record_still_names_one_place_in_the_tree`,
because the extraction moved the OK-path record's anchor (below).

Task 2's red named fourteen: eleven of the target's thirteen bare, `what_arrived`'s two tests by
module path, and the count check bare for `mail_sync.rs`'s twelve records. The stub answered a
sentence for every check, empty or not. Three tests in the target were green on arrival and not
named: the one that refuses a text too small to be the window reads a synthetic text, and the two
companions that plant a signal where the watch wakes and a step on the answer channel planted them
into a window that already had both. The gate in `red` mode ran exactly the fourteen red, in
232 s. The green ran the filters and the coupled targets in 292 s, no remedy printed: every
record the count check had named was re-measured before the commit.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/what_is_said_while_fetching.rs` | `--lib application::what_is_said_while_fetching::` on both task 1 commits |
| `src/application/mod.rs` | `--lib application` whole, on task 1's red |
| `src/data/config.rs` | `--lib data::config::` on task 1's red |
| `src/presentation/accessibility.rs` | `--lib presentation::accessibility::` on both task 1 commits |
| `src/presentation/wx_settings.rs` | the filter matching nothing and the nine coupled targets on task 1's green |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the coupled targets on task 1's green and task 2's green, fourteen then fifteen |
| `src/application/mail_sync.rs`, `asking_for_a_whole_folder.rs`, `src/presentation/ui_types.rs` | their own filters on task 2's commits |
| the new target, `every_event_has_a_control`, `a_whole_folder_moves_both_bounds` | their own targets when they changed; as coupled targets on every commit touching what their records name |
| `guards/guards.toml`, `docs/changelog.md`, `docs/manual-accessibility-pass.md`, `.planning/WINDOWS.md` | no scoped target; the whole-tree guards on every commit, the nine document-reading targets on the documents-only commit |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `b5bccce8`, output to a file with the exit status written
beside it, never piped: exit 0, 7,995 passed and none failed over 73 result lines, 442 s from
00:26:39Z to 00:34:01Z, the release build included; its last block says five of CI's seven jobs.
Thirty-five more than 10-03's 7,960: 19 in the module, 1 in `accessibility`, 2 in `mail_sync`,
13 in the target. `main`'s hook ran `all` again on the merge: 7,995 and none failed, 431 s from
00:34:29Z to 00:41:40Z. The keyring race (ledger 374) did not appear on either. **The tab row
reading 10-02.1's summary recorded failing intermittently passed every time it ran here**: on
task 1's green twice, task 2's green, and both whole gates.

## Guard records

892 records by the TOML reader before, 897 after; census 802 + 90 before, 802 + 95 after, the
line at `guards/guards.toml:84` moved in each green commit. Five new, every one measured through
`scripts/guards.sh --remeasure` with `WIXEN_TEST_THREADS` untouched and the counts written by the
runner.

| Record | File | Break | Red | Run |
|---|---|---|---|---|
| a step is not spoken under Say what arrived | `what_is_said_while_fetching.rs` | the Progress row widened to everything but Errors only | 1 | rebuild 29 s, run 50 s |
| the level chosen on the Feedback page is what OK writes back | `wx_settings.rs`, `suite` `every_event_has_a_control` | the write in `read_the_feedback_page` replaced by a read of the control | 1 | rebuild 16 s, run 1 s |
| a step is spoken only when every step was asked for | `wx_app.rs`, `suite` the new target | the ask removed from the Progress arm | 2, the reading and its companion | rebuild 15 s, run 1 s, twice |
| the new-mail sound means mail arrived, not that the watch woke | `wx_app.rs`, `suite` the new target | the signal put back in the MailboxChanged arm | 1 | rebuild 16 s, run 1 s |
| a check that found nothing says nothing about what arrived | `mail_sync.rs` | the filter that drops folders with nothing removed | 2 | rebuild 41 s, run 50 s |

**Three records rewritten onto moved lines and re-measured.** "what goes to the status bar is
said as well as shown", whose `before` was the exact `Priority::Low` line task 2 changed, onto the
`Normal` line, keeping its `after` and its two named tests: all 2 went red and nothing else,
rebuild 35 s, run 49 s. "pressing OK writes the answers the panel holds rather than rebuilding
from the stored value", whose `before` was inside the block task 1 extracted, onto the new
function with `cfg`'s stored value in place of `base`'s: the one test went red, rebuild 13 s,
run 1 s. "asking for a notes sync reaches the sync rather than reporting one", whose `before` was
the `send_status` line task 2 turned into `send_progress`: the one test went red, rebuild 48 s,
run 51 s. Each carries a dated comment saying what moved it.

**The count check's remedy, run and read, and two records corrected by hand first.** After task
1's red it named the seven on `accessibility.rs` (23 to 24); all seven agreed as written, 773 s
for the run of nine. After task 2's red it named the twelve on `mail_sync.rs` (147 to 149); the
run of sixteen took 1,483 s, and two records came out short. "a count and the thing it counts
agree in number" reddened `test_what_arrived_names_each_folder_that_received_something_with_its_count`
as well as its twenty-two, because the result line counts through `how_many`; it was corrected by
hand with the dated comment and re-measured: all 23 went red and nothing else, rebuild 33 s, run
49 s. My own step record's first draft named the reading alone and said its companion was green
either way; the runner said the companion reddens too, because it refuses to plant into a tree
without the ask; the record was corrected by hand and re-measured: all 2, rebuild 15 s, run 1 s.
The other eleven on `mail_sync.rs` agreed as written: 3, 1, 3, 2, 1, 1, 1, 1, 1, 2, 3 tests red
in the order the check listed them. Counts written: `accessibility.rs` 24 on seven,
`mail_sync.rs` 149 on thirteen, `wx_app.rs` 199 on sixty-two, `wx_settings.rs` 0 on nineteen, the
module 19 on one, the target 13 on two. `bash scripts/check.sh --suites-for guards/guards.toml
src/presentation/wx_app.rs` answers fifteen targets, `wx_settings.rs` nine. No file gained or lost
a test after its records were measured; the count check is green on `main` at `19a10706`.

## Every status line in `wx_app.rs`, and its kind

The reading holds the openings; this table holds the rest, read off the shipping half at the
merge, 126 sites. Line numbers are at `19a10706`.

| Kind | Sent as | Sites |
|---|---|---|
| step | `UIUpdate::Progress` or `send_progress` | 2900 Loading; 4161, 4165, 4169, 4173 the four sync requests from the module's Sync; 4480 Checking for new mail; 5036, 5097, 5101 the sync requests from Tools and the address book; 5105 Syncing tasks; 19870 the POP Connecting; 19959 the POP result when nothing came; 21404 the missing-text fetch's progress; 21780 the whole-folder request's lines and closing report; 21869 Connecting; 21913 the folder count; 22015 Checking a folder; 22044 the per-folder sentence; 22070 Mail check finished |
| result | `UIUpdate::WhatArrived` | 19957 the POP result when something came; 22078 the check's counts after the loop |
| result, through the event | `UIUpdate::ModuleSyncFinished` | 22304 Tasks; 22393 Notes when it ran; and the existing `ContactsSyncComplete` and `CalendarSyncComplete` arms |
| answer to a key, the start of what was asked | `send_status` or `StatusUpdated` | 4580 Getting older messages; 4850 Deleting a message; 5230 Flushing outbox queue; 7278 Running this saved search; 7355 the coverage sentence; 8214 Making a folder; 8521 Renaming; 8657 Moving; 8752 Deleting a folder; 8981 Emptying; 12927, 13356, 13503 an import or export starting; 18743 Sending queued messages; 19700 Sending a read receipt; 21066 Opening an attachment; 21185 Saving; 21345 Looking for message text |
| answer to a key, what it did | `send_status` or `StatusUpdated` | 2231, 2278, 2313, 2330 the editor's answers; 3347; 4744; 4892 Marked; 9835 Opened a help page; 10187 the mode indicator; 10416 a label put on or taken off; 12963, 13001, 13049, 13387, 13530, 13638 an import or export's progress and end; 14687 the `send_status` definition; 14857 `told`; 15143, 15246 Draft saved; 15221; 15521 filed; 16965 Settings saved; 18832, 18861 each queued message's outcome; 19282, 20437 the next step of a change; 19416 Saved here; 19453 Taken out of the outbox; 19571 an outcome; 19655, 19662, 19667 the read receipt answers; 19802 Read receipt sent; 20549 a flag done; 21205 Saved to; 21417 what the text fetch did; 22377, 22382 the notes sync's two answers that no sync ran; 22644 |
| answer to a key, a refusal or a failure that stayed on this channel | `send_status` or `StatusUpdated` | 2334, 2340 Export failed; 2858; 3959 not built yet; 4181 does not sync anywhere yet; 7641, 7795, 7862, 7922, 8956, 9023, 9221, 9590, 9661, 12865, 13086, 13460 the managers' refusals, each announced at High by its caller as well; 10314, 10338, 10376, 10385 the label refusals; 12943, 12951, 13515, 13523 an import or export refused; 13997; 14667 still waiting; 14865 `went_wrong`; 15378; 16885 Cannot open settings; 16958 Settings save error; 18736 Outbox is empty; 18992, 19048, 19051, 19314, 19359 a move refused; 19585 not deleted; 19694 no receipt to send; 20096 New mail will not appear on its own |

Nothing on the answer channel opens with a step's words, which the reading holds. The refusals
that ride it rather than `send_refusal` are the refusal test's subject and unchanged here.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `fb0e8bce` before anything
was built. Five things moved or were wrong:

1. **Every line number in the plan.** 10-02, 10-02.1 and 10-03 changed `wx_app.rs` after the plan
   was written: `StatusUpdated` was at `:17543`, not `:17576`; `MailboxChanged` and the signal at
   `:18257-18265`, not `:18288-18299`; `spawn_mail_sync` at `:21682`, `check_pop_mail` at
   `:19718`, the tasks sync at `:22106`, the notes sync at `:22198`, `quiet_on_purpose` at
   `:25732`, the refusal test at `:26217`. The shapes held.
2. **Premise 1's counts.** `grep -n 'UIUpdate::StatusUpdated(' src/presentation/wx_app.rs | wc -l`
   answered 43 as the plan said, and `grep -rn 'send_status(' src --include='*.rs' | wc -l` 116,
   of which 82 lines in `wx_app.rs` (the definition and four test lines among them) and 34
   elsewhere. The reading counted 107 answer-channel calls at the merge.
3. **The signal's home.** Premise 2 and the acceptance criterion put `a11y.signal(NewMail)`
   inside `spawn_mail_sync`; the worker runs on a blocking thread with `state`, `tx` and `rt` and
   no accessibility handle, so the criterion could not be met as written. Observation 656 in the
   skill log; deviation 1.
4. **`WholeFolderProgress` had a reader outside `wx_app.rs`.** The plan said the variant folds
   into `Progress` and the topic constant loses its consumer, and named no test;
   `tests/a_whole_folder_moves_both_bounds.rs` read the arm and required the constant, so that
   reading was rewritten in place (eight tests before and after) and the record naming the file
   was not flagged.
5. **The refusal test's allowance.** Neither `"No new mail` nor `"Nothing to send` appears in
   `src/` outside the test, so "rewritten in place to hold only what still rides `send_status`"
   meant removing it: an allowance for a line nobody sends is a hole waiting for one somebody
   does.

Two records the plan did not name moved with the code and were caught by the hook: the OK-path
record inside the block task 1 extracted (observation 657), and the notes-sync record on the
`send_status` line task 2 changed. Both are rewritten and re-measured above.

## Deviations from plan

**1. [Rule 3] The new-mail signal is in the `WhatArrived` arm.** Above and ledger 520. The arm is
what the worker's end-of-check update reaches, and `WhatArrived` is sent only when something
arrived, so the signal fires when a check found mail, whoever started it, which is the meaning the
plan wanted. The reading holds the `MailboxChanged` arm clear and the shipping half to one place.

**2. [Decision] The variants are on `ui_types.rs`.** The plan's file list named `wx_app.rs` for
`UIUpdate::Progress` and `WhatArrived`; the enum lives in `ui_types.rs`, which is where they and
`ModuleSyncFinished` went. Six records name that file and none anchors on the lines changed.

**3. [Decision] `ModuleSyncFinished` for the tasks and notes syncs.** The plan said they "say it
through `SyncComplete` the same way" without naming an update; the existing completion updates
carry the contacts and calendar results as typed values, so a third update carrying the sentence
was the smallest true shape. The notes sync's "kept on this computer" and "nobody is signed in"
answers stay on the answer channel, because somebody pressed Sync and nothing ran, and under
Errors only they would otherwise be silent.

**4. [Decision] The result sentence's noun.** "Inbox, 3 new messages; Work, 1 new message" rather
than "Inbox, 3 new; Work, 1 new", through `how_many`, so a listener hears what the number counts
and the singular is right. Folder first, then the count, nothing about folders with nothing, as
the plan asked.

**5. [Decision] The whole-folder request's closing report is a step.** The loop hands its
progress and its closing report to one closure, so both are `Progress` and under the default the
report is shown and not spoken. Splitting the loop's API for a command 10-05 retires was not
worth its cost; the changelog's Known limitations say so.

**6. [Decision] The changelog entry in task 1's commit.** The rule that the entry lands with the
setting outranked the plan's task 3; task 2 extended it and task 3 owed only the listening lines.

**7. [Decision] The Feedback-page record on the existing target.** The plan said "the coupled
target that reddens, measured rather than predicted"; `every_event_has_a_control` builds the
dialog with the Feedback page shown and reads settings back, so a sub-check there was the
reading, with no new `#[test]` and no records flagged.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; `cargo fmt` ran before each commit; the only `sed`, `awk`, `grep` and Python in the session
read files and logs; `git checkout` was not needed. Commit messages were written to the
scratchpad and passed with `-F`. Carriage returns measured with `tr -cd '\r' | wc -c` on every
changed file before each commit: zero on each. No em-dash in any file this plan wrote, measured
with `grep -c` for the byte sequence: zero on each. `git commit` and `git merge`, never
`gsd-tools query commit`; never `--no-verify`; `check.sh` never piped, its exit status written to
its own file. No AI attribution in any commit. `Cargo.toml` and `Cargo.lock` untouched; no crate
added (T-10-SC). The version stays `1.0.0-alpha.1`. The tester's profile was not read; no binary
was started; nothing was sent to any window this plan did not build.

## Threat register

T-10-11 mitigated: a step is spoken only under Say every step, held by the arm's ask, the reading,
and a measured record; results once per check. T-10-12 mitigated: `ErrorOccurred` is unchanged at
High, the reading holds every answer-channel call's opening, and the table above sorts every
line. T-10-13 mitigated: the signal moved to the `WhatArrived` arm, held by a companion and a
measured record. T-10-14 mitigated: `StatusUpdated` at Normal, held by the reading and the
rewritten record; no step rides it. T-10-SC: nothing added. No new surface outside the register:
the setting is read from the settings file the other settings are read from, and the reading
target builds no window.

## Ledger

`.planning/WINDOWS.md` 520 and 521 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 519 before, 521 after; 491
open before, 493 after.

| id | kind | what |
|---|---|---|
| 520 | deviation | the departures above with their reasons, so 10-05 and 10-06 read the kinds from the tree |
| 521 | unrun-verify | what only the tester's ear settles: the three answers on a check of his folders, which sentence is a step and which a result, the result sentence as an ending, the sound and the sentence as one event or two, Settings saved during a check, and the choice itself under Alt+W |

## The issue

`gh issue close 38` from the repository root after the merge, at 00:42:14Z, the plan's sentence
quoting `19a10706` with the ear list appended. Closing is not a publish; no other issue was filed
or edited.

## Known stubs

None. The choice is built by `build_feedback_tab` on every Settings open that reaches the
Feedback tab and read back by `read_the_feedback_page` when OK is pressed; the field is read at
startup and on OK into `Accessibility`, which the three arms ask on every line; every sync path
sends its lines as one kind or the other. `THE_PROGRESS_TOPIC` has no consumer and says so, kept
for 10-05 to retire with the command.

## What the tester's account could settle, and a loopback proved

Nothing here is claimed against a real account, and nothing here was heard. Values proved the
table, the stored forms, the default and the sentence; a real dialog proved the three entries in
order, the default's selection and each choice written back; the source proved which channel each
line is on and that the arms ask before they speak. What a check of the tester's 50 folders says
under each answer, and whether the sorting by words matches the sorting by ear, is ledger 521.

## Not done here, on purpose

No `MAIL` requirement is ticked, on the phase's rule that 10-07 reads each clause; the clauses
this plan gives a named test and a named control for are MAIL-05's `[D]` lines: the choice
`FeedbackTabControls::announce_while_fetching` under While fetching, built at
`wx_settings.rs:2968` with `WHILE_FETCHING_LABEL`; the twelve cells and
`test_the_default_says_what_arrived` for the levels; the six readings in the new target for the
kinds and the arms; `test_a_check_that_found_nothing_says_nothing_about_what_arrived` for the
result; `test_every_setting_somebody_can_change_is_offered_by_a_screen` as what failed on arrival.
Roadmap criterion 5 closes structurally and is read by 10-07.

## For 10-05 and 10-06

- A line a fetch says on the way is `UIUpdate::Progress(String)` or `send_progress(tx, rt, msg)`;
  a line that answers a key is `UIUpdate::StatusUpdated` or `send_status`, spoken at Normal on
  `"status"`, where a newer answer replaces the older. The reading in
  `tests/progress_is_shown_and_results_are_said.rs` refuses a `send_status` or `StatusUpdated`
  call opening with `"Checking `, `"Connecting `, `"Loading `, `"Syncing `, `"Fetching ` or
  `sync requested`; add an opening to `PROGRESS_OPENINGS` when a new kind of step is written.
- The counts go out once per check as `UIUpdate::WhatArrived { what }` built by
  `mail_sync::what_arrived(&[(folder, fetched)])`, which answers `None` for nought; the arm is
  where `NewMail` is signalled. The runner's chunks are steps; if a download ends a check with
  its own counts, fold them into the same `arrived` list rather than sending a second
  `WhatArrived`, because the reading holds `spawn_mail_sync` to one and the sound to one event.
- The reading also holds the mail check's five lines by their opening words and the nearest
  `UIUpdate::` before each; a runner that moves "Mail check finished" or the per-folder sentence
  must keep them `Progress` and keep `WhatArrived` after "Mail check finished".
- `Accessibility::how_much_to_say().is_spoken(Kind)` is the one question; the level is set at
  startup and on OK and never read from the file per line. The status line's three states
  (10-06) are answers to nothing anybody pressed and steps to nothing either; if they are shown
  and must be heard once, `StatusUpdated` at Normal is the channel, and the watch's own "watching"
  and "waiting" lines are steps.
- The whole-folder request and the missing-text fetch already send `Progress`; retiring them
  retires `THE_PROGRESS_TOPIC`, its test in `asking_for_a_whole_folder.rs`, and the whole-folder
  target's reading of the request's lines, which is a test rewritten or removed in a file one
  record names at 8.

## Self-Check: PASSED

`src/application/what_is_said_while_fetching.rs` and
`tests/progress_is_shown_and_results_are_said.rs` exist; `grep -c 'pub mod
what_is_said_while_fetching' src/application/mod.rs` is 1; `grep -n 'set_how_much_to_say'
src/presentation/wx_app.rs` answers two lines; `grep -c 'UIUpdate::Progress('
src/presentation/wx_app.rs` is 12; `grep -n 'a11y.signal(FeedbackEvent::NewMail'` answers one
line in the `WhatArrived` arm; `grep -c 'WholeFolderProgress' src/presentation/ui_types.rs
src/presentation/wx_app.rs` is 0 and 0; `guards/guards.toml` holds 897 records by the TOML reader
and the census line says 95; `docs/changelog.md` holds the line beginning "**How much is said
while mail and the other modules are fetched is your choice"; `docs/manual-accessibility-pass.md`
holds items 42 and 43; `.planning/WINDOWS.md` holds 520 and 521 in both halves. Commits
`d310ea9e`, `85ee0c52`, `0938b003`, `9adf41a4`, `b5bccce8` and `19a10706` are in `git log
--oneline` on `main`. #38 is closed.

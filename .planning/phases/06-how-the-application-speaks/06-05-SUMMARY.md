---
phase: 06-how-the-application-speaks
plan: 05
status: complete
subsystem: ui
tags: [reminders, modal, typing, earcon, announcement, repeat-tone, guards, wired-reading, accessibility]

requires:
  - phase: 01-folders-and-conversations
    provides: "01-10's one-at-a-time gate shared between the reminder alert and the folders question, and the typing count in `one_question_at_a_time` that the reminder did not ask"
  - phase: 06-how-the-application-speaks
    provides: "06-01's `Event::Reminder` doc comment saying the window is the written equivalent its sound needs, which is why the sentence had to become sayable without the window"
provides:
  - "`one_question_at_a_time::whether_a_window_may_open(somebody_is_typing, something_is_already_up) -> Moment`, one rule about when a modal may open, answering `Free`, `SomebodyIsTyping` or `SomethingIsAlreadyUp`, tested without a window; `what_to_raise` asks it"
  - "`wx_reminder_alert::say(item, now, dates, a11y) -> Said`: the tone and the sentence with no window built, reporting whether the tone really sounded"
  - "`wx_reminder_alert::Spoken { NotYet, Already }`, the argument `raise` takes so a sentence said at an earlier look is not said again"
  - "`wx_reminder_alert::RepeatingTone`, `ToneNow`, `BETWEEN_TONES` (60 s) and `MOST_TONES` (10): the tone that comes back once a minute until focus reaches the window, stops for good the first time it does, and stops at ten regardless"
  - "`wx_app::whether_somebody_is_typing(&[TextCtrl])`, the two-part typing check written once and asked by both the folders question and the reminder look"
  - "`wx_app::LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES` (1) beside `HOW_OFTEN_TO_LOOK`, and `wx_app::BetweenLooks`, the struct carrying `already`, `said_and_waiting` and the shared gate from one look to the next"
  - "Two readings in `tests/wired.rs` of `raise_what_is_due`, and the folders reading taught the helper"
affects: [06-09, 06-06, 06-08, version-2]

actuals:
  tokens: 17100
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - "A rule that answers which reason rather than a bare boolean, because the two reasons for no ask different things of a caller: typing means say it and hold the window, already up means do nothing this look"
    - "The sound and the sentence of an event whose written equivalent is a window are a function of their own, so the window can be held without the event going out as nothing"
    - "A repeating sound is a pure rule handed instants and the focus answer, with a latch and a ceiling, and the window's timer only asks it; the timer ticks more often than the rule's spacing so the rule owns the spacing"
    - "A source-reading guard that follows an argument to a shared helper and reads the helper's body once, refusing an inline copy, with a companion shown the copy and shown the helper missing"

key-files:
  created: []
  modified:
    - src/presentation/one_question_at_a_time.rs
    - src/presentation/wx_reminder_alert.rs
    - src/presentation/wx_app.rs
    - src/presentation/accessibility.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml

key-decisions:
  - "Option 3 with a repeating tone, Pratik's answer of 2026-09-14, recorded and not re-asked: hold briefly, then raise anyway, and sound the tone once a minute until the window has focus"
  - "The hold is one look, sixty seconds, the value of HOW_OFTEN_TO_LOOK; the ceiling is ten tones; the sentence is said once, at the look that finds the reminder, and not again when the window opens"
  - "The dialog's timer asks the repeat rule once a second rather than once a minute, because a timer set to the minute and a rule wanting a full minute miss each other by milliseconds and sound every two minutes; the rule owns the spacing"
  - "The three cells the timer carried became one BetweenLooks struct, because clippy refuses nine arguments and an allow is not how a commit gets through here"
  - "What is said and waiting is pruned each look to what is still due, so a reminder completed from the panel while its window was held does not open later without its sentence"

patterns-established:
  - "Red commit for a new function carries its signature with a todo!() body, and names the count check when a file a record fingerprints gains a test; 06-01 set this and it held twice here"
  - "A guard record predicted at two tests and measured at two, twice; the fourteen wired.rs records re-measured after the tree was final with no correction needed"

requirements-completed: [FEEDBACK-01]

coverage:
  - id: D1
    description: "One rule says whether a modal may open, distinguishing its reasons, tested without a window; what_to_raise uses it and its nineteen tests pass unchanged"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "cargo test --lib -- presentation::one_question_at_a_time::"
        status: pass
    human_judgment: false
  - id: D2
    description: "The reminder sentence and tone go out with no window built, and the tone is reported as sounded only when it did"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "presentation::wx_reminder_alert::tests::test_a_reminder_can_be_said_and_sounded_without_its_window"
        status: pass
    human_judgment: false
  - id: D3
    description: "The tone comes back once a minute while unfocused, stops for good once focus has arrived even if it leaves, stops at once for a window that opens with focus, and stops at ten"
    requirement: FEEDBACK-01
    verification:
      - kind: unit
        ref: "presentation::wx_reminder_alert::tests::test_the_tone_*"
        status: pass
    human_judgment: false
  - id: D4
    description: "raise_what_is_due asks the modal rule through the shared typing helper, takes the turn before the loop, and writes the reminder down before its window; the folders question asks the same helper"
    requirement: FEEDBACK-01
    verification:
      - kind: integration
        ref: "cargo test --test wired"
        status: pass
    human_judgment: false
  - id: D5
    description: "A reminder due while somebody types is heard at once, its window arrives a minute later, and the tone comes back until the window has focus, in a running build under a screen reader"
    requirement: FEEDBACK-01
    verification: []
    human_judgment: true
    rationale: "Every assertion above proves structure. Whether hearing the sentence and meeting its window a minute later reads as helpful or as the same thing twice, whether ten tones reads as looked after or nagged, whether NVDA speaks the sentence with another application in front, and whether has_focus() on the four controls answers as the toolkit says, are a listening pass: WINDOWS.md 379 to 383"

duration: 2h 40m
completed: 2026-09-14
---

# Phase 06 Plan 05: A reminder is said before its window takes the keyboard Summary

**A reminder that comes due while somebody is typing is said and sounded at the look that finds it, on a channel that does not move focus; its window opens at the next look whether or not they have stopped; and once open, its tone comes back once a minute until focus reaches it, ten times at most, stopping for good the first time it does. Version 0.123.0. Nobody has heard any of it.**

Branch `said-at-once-and-the-window-a-look-later`, four commits, merged into `main`. Nothing pushed.

## The checkpoint, answered 2026-09-14

The checkpoint asked what a reminder should do when it comes due while somebody is typing. Pratik answered on 2026-09-14, option 3 widened, in his words: "3 + reminder tones every minute until the user focuses on the reminder window which should allow for snooze, snooze all, dismiss, and dismiss all, keeping in mind that there may be multiple reminders in the window. Similar considerations should occur for Calendar and task reminders. Could the same infrastructure be used?" And when a split was proposed: "yes. But there should be an indication that a task is due, a calendar event is coming up, or a reminder. In other words, I'm reconsidering a single window for multiple types."

This plan is the first half: hold briefly, then raise anyway, with a tone every minute until the window has focus. The widening, one window for every due thing, is `06-09-PLAN.md` and depends on what this plan built. It was not re-asked and 06-09's window was not built here.

Two figures the planner proposed and this executor recorded rather than decided. **The hold is one look, sixty seconds**, the value of `HOW_OFTEN_TO_LOOK`: a reminder found due while somebody is typing is said and sounded at that look, and its window opens at the next, whether or not typing has stopped. **The ceiling is ten tones**, one a minute, stopping for good the first time focus reaches the window, with no resume if focus leaves. The sentence is said once, when due, and the window is not announced again.

**The cost, in the plan's own words, which Pratik accepted with his eyes open:** hold-then-raise "steals focus mid-word eventually, which is the thing being complained about, just later." What it buys over raising at once is a warning and a minute. That sentence is in the doc comment on `LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES`, in the changelog's Known limitations, and in ledger 380.

## What landed

### Task 1: one rule, a sentence without a window, a tone that stops

Red `eaacdf2f`, green `8411f27a`.

**The rule.** `one_question_at_a_time::whether_a_window_may_open(somebody_is_typing, something_is_already_up) -> Moment` at `one_question_at_a_time.rs:154`, with `Moment { Free, SomebodyIsTyping, SomethingIsAlreadyUp }` at `:139`. Three answers rather than a boolean, because the two reasons for no ask different things of a caller. Something already up outranks somebody typing, because it is the stronger answer: nothing this look. Two tests, one asserting the three answers are told apart and one asserting the ranking. `what_to_raise` now asks the rule rather than repeating the condition; that part of the diff is a refactor of three lines, and **its nineteen existing tests pass with no assertion edited**, checked by reading every removed line of the file's diff since `main`: three removed lines, none inside the tests module.

**The split.** `wx_reminder_alert::say(item, now, dates, a11y) -> Said` at `wx_reminder_alert.rs:66`, doing what `:61-65` of the old file did inside `raise`, and returning the sentence with whether the tone sounded. Its doc comment says why it exists: the window is this event's written equivalent, so a reminder whose window is held back has to say something or the event goes out as nothing at all. `raise` takes a seventh argument, `Spoken { NotYet, Already }` at `:88`, and calls `say` only for `NotYet`; for `Already` it builds the sentence for the window's text and says nothing. `raise` also takes `&Arc<Accessibility>` now rather than `&Accessibility`, because the tick closure owns a handle; the call site already had one.

**The rule about saying it again: once.** Written into `say`'s doc comment and here. The sentence goes out at the moment the reminder is found due, whether or not the window can open then. When the window opens after a hold nothing is announced a second time: the sentence is the window's static text and accessible name, and a dialog's text is what a screen reader reads when focus arrives in it. Whether NVDA does that with this dialog is ledger 380. The repeating tone is a tone alone; its written equivalent is the window on screen.

**The test that proves the split.** `test_a_reminder_can_be_said_and_sounded_without_its_window` builds `Accessibility::new()`, calls `say` with a `Due` and default date settings, no display, no dialog, and asserts three things: the sentence returned is `item.spoken(now, dates)`, the sentence reached the screen reader bridge (read back through a `#[cfg(test)] pub(crate) fn last_announcement` added to `accessibility.rs:424`, because nothing outside that module could see what was said), and the tone is reported as sounded. Then a second `Accessibility` with sounds at their default, which is off, and the test asserts the tone is reported as not sounded and the sentence still went out. It runs under `WIXEN_NO_AUDIO=1` and passes there too; the tone goes to a mixer nothing listens to.

**The repeat rule.** `RepeatingTone` at `:130`, holding when the last tone was, how many have sounded, and whether focus has ever reached the window. `asked(now, window_has_focus) -> ToneNow { Sound, Wait, Finished }` at `:148`: the focus answer is latched with `|=`, not read; finished for good once latched or once `MOST_TONES` have sounded; otherwise `Wait` until `BETWEEN_TONES` has passed since the last. `BETWEEN_TONES` is sixty seconds at `:99` and `MOST_TONES` is ten at `:109`, each with the reason from the plan's correction block on it. Four tests hand it instants rather than reading the clock: a tone at each minute while unfocused and none inside a minute; nothing after focus has arrived once even when it leaves again; a window that opens with focus never sounds; exactly ten over sixty minutes and `Finished` at the sixty-first.

**The window's timer.** `wx_reminder_alert.rs:220-251`, on the `wx_managers.rs:4068` precedent:

```rust
    let watching = Timer::new(&dialog);
    watching.on_tick({ ... asks has_focus() of the snooze Choice and the three buttons found by id, hands the answer to the rule, sounds a11y.earcon(ALERT_EVENT) on Sound ... });
    if !watching.start(HOW_OFTEN_TO_ASK_THE_TONE, false) { tracing::warn!(...) }

    let answer = dialog.show_modal();
    drop(watching);
    ...
    dialog.destroy();
```

Held across `show_modal`, dropped before `destroy()`. Focus is asked of the four controls rather than the dialog, because on Windows a dialog has focus only when none of its children does, which is never while somebody is in it. The buttons are found by `find_window_by_id` from the dialog rather than returned from the builder, so `build_reminder_alert_dialog`'s signature and `tests/theme_reach.rs` are untouched.

Two things about the timer differ from the plan's sketch, both said here. It ticks **once a second, not once a minute**: a timer set to sixty seconds and a rule wanting sixty seconds miss each other by a few milliseconds every other tick and the tone would sound every two minutes; the rule owns the spacing and a tick that is told `Wait` costs nothing. And it **does not stop itself when the rule says Finished**: stopping a timer from inside its own tick handler means the handler owning the timer, which is a cycle across the toolkit boundary; a tick that asks and is told `Finished` does nothing, and the drop after `show_modal` ends it.

**What if the window is closed without ever being focused.** By a person's hand it cannot be: every way out of this modal goes through a button or a key, and both put focus in the window first, which latches the rule. If the process ends, the timer dies with the dialog. If the timer refuses to start, the window still opens with its sentence on it and a warning is logged saying the tone will not repeat.

**Two guard records**, measured with the runner rather than predicted, both predicted at two tests and measured at two: "the rule about opening a modal does not answer free while somebody is typing" (the break makes `(false, true)` answer `Free`; red: the rule's own test and `test_nothing_is_raised_while_an_editor_has_focus_and_it_is_raised_once_focus_leaves`, because `what_to_raise` goes through the rule now) and "the reminder tone does not come back once focus has reached the window" (the break turns the latch `|=` into `=`; red: the leaving-focus test and the opens-with-focus test, because with the latch gone the second ask finds no focus and sounds). Census 563 to 565, 755 to 757 records by a TOML reader. The record naming `one_question_at_a_time.rs` moved from 19 tests to 21 by `--remeasure` in the same detached run; all three came back "redden exactly the tests their records name".

### Task 2: wired at the reminder's call site, and the reading taught

Red `081625e1`, green `106efe6c`.

**The helper.** `whether_somebody_is_typing(somewhere_to_type: &[TextCtrl]) -> bool` at `wx_app.rs:10529`, the two-part check written once with the comment from above the old `:10411` hoisted onto its doc. It lives in `wx_app.rs` because `one_question_at_a_time` is window-free on purpose and this takes a `TextCtrl`. Both functions call it: the folders question at `:10444` and the reminder look at `:10608`. `grep -n somewhere_to_type src/presentation/wx_app.rs`: `5652` the slice built once per tick, `5667` and `5683` the two call sites, `10434` the folders question's parameter, `10444` its call, `10529` and `10531` the helper, `10550` the reminder look's parameter, `10608` its call.

**The hold.** `LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES: u32 = 1` at `:10088`, beside `HOW_OFTEN_TO_LOOK`, with the reason from the plan on it so whoever changes either number meets the other. It is a real constant, not a decoration: `said_and_waiting` counts looks waited per reminder id and the arm at `:10617` compares against it.

**The look.** In `raise_what_is_due`, after `what_is_due`, the moment is asked once per look, `whether_a_window_may_open(whether_somebody_is_typing(somewhere_to_type), false)`, with `false` for already-up because the turn was taken above. For each item: `SomethingIsAlreadyUp` cannot happen and does nothing, written out so a fourth reason is a compile error; `Free` and not waiting says it in `raise` as before; `SomebodyIsTyping` and not waiting calls `say`, records the id at zero looks waited, and continues without touching `already`; `SomebodyIsTyping` and waited fewer than the hold increments and continues; otherwise, the hold is over or typing stopped, the window opens with `Spoken::Already`. A window that opens takes the item out of `said_and_waiting` first, whether or not it was held, so a snooze or dismissal leaves nothing waiting behind it.

**Why nothing about the one-at-a-time turn changed.** It is taken first, at `:10567`, and held to the end of the function as before. A look that says a sentence and opens nothing returns, and returning drops the `Turn`, so the folders question is never blocked by a window that did not open. The comment above the `take` now says so.

**The `already` bookkeeping is unchanged.** `already.borrow_mut().insert(item.id.clone())` at `:10647` before `raise`, and `already.borrow_mut().remove(&item.id)` on a snooze at `:10675`, are the same lines; neither appears in the diff. That is what makes a held reminder come back for free: nothing is inserted until the window is about to open, and `what_is_due` is passed a fresh `&seen` each look.

**One thing added the plan did not ask for.** `said_and_waiting` is pruned each look to the ids still due, `:10599-10601`. Without it, a reminder said while typing and then completed or moved from the panel before the next look would leave a stale entry, and if that id ever came due again its window would open with `Spoken::Already` and no sentence. Glue, untested in `wx_app.rs`, and said here.

**The struct.** `BetweenLooks` at `:10095` holds `already`, `said_and_waiting` and the shared `one_question_on_screen` gate, built once at `:5447`. Clippy refused `raise_what_is_due` at nine arguments, and the rule here is that an allow is not how a commit gets through, so the three cells the timer carried became one value. The folders question borrows the gate out of it and is otherwise untouched. The comments on the three cells moved onto the fields.

**Glue, and said so.** Everything in task 2 except the readings is wiring of components task 1 tested without a window, which `CLAUDE.md` lists as the exception to test-first. No test was added to `src/presentation/wx_app.rs`: its `#[test]` count is 199 before and 199 after, and its 48 records were not re-measured.

**The readings.** `tests/wired.rs`, written red first against the tree before the wiring; the red at `081625e1` was exactly the two tree readings and the count check:

```
test_the_question_about_vanished_folders_is_put_at_a_moment_that_is_free ... FAILED
   the typing answer does not come from the shared helper reading both ways somebody can be typing ...
test_the_reminder_look_asks_whether_somebody_is_typing_through_the_shared_helper ... FAILED
   raise_what_is_due does not hand the modal rule the shared typing helper's answer ...
test_every_guard_record_says_how_many_tests_the_files_it_names_held ... FAILED
   14 guard records were measured against a tree that no longer holds those tests: tests/wired.rs has gained 2 tests: it held 69 and holds 71
```

The folders reading now takes the whole shipping source rather than one body, follows the typing argument to `whether_somebody_is_typing(` and reads the helper's body once for the three tokens (`the_typing_answer_reads_both_ways` at `:3633`). It refuses the check worked out inline, on purpose: two copies of that check is two things to keep in step, and a reading that accepted either form could not see the copies drift. Its companion is shown the real shape with the helper, the argument replaced by a constant, half the helper replaced, the check written inline, and the helper missing from the source. The sibling reading `test_the_reminder_look_asks_whether_somebody_is_typing_through_the_shared_helper` at `:3906` asserts three things of `raise_what_is_due`: the rule is handed the helper's answer, `.take()` comes before `for item in`, and `already.borrow_mut().insert(` comes before `wx_reminder_alert::raise(`. Its companion at `:3932` is shown each of the three missing. Two `#[test]`s rather than the plan's one, because a source-reading guard without a companion is a reading that could be measuring nothing, and the count check fires once for the file either way.

**Fourteen was the price and why it was paid.** Two tests in a file fourteen records fingerprint moved every one of those records from 69 to 71. The alternative, copying the check into both functions and leaving the reading alone, was refused for the reason the plan gave: two copies of a paragraph explaining a subtlety is two things to keep in step. The fourteen were re-measured after the green commit, detached, in one `--remeasure` run of about fourteen minutes: every one "went red, and nothing else did", thirteen at one test and "a PGP private key somebody can import" at its three, and the run wrote the same 71 the hand edit had written, so `guards/guards.toml` did not change under it.

**Version and changelog.** `0.122.0` to `0.123.0` in the green commit, minor, with `Cargo.lock` following. The `[Unreleased]` entry under Changed says in plain sentences what changes for somebody writing a message, and its Known limitations say nobody has heard any of it and that the window still opens over them after the minute.

## Names 06-09 reads, as shipped

06-09's plan reads this plan's symbols by the names this summary records, not the ones the plan proposed. Every name here is the one in the tree at the merge:

| What the plan called it | What shipped | Where |
|---|---|---|
| the split say function | `wx_reminder_alert::say(item, now, dates, a11y) -> Said { sentence, tone_sounded }` | `wx_reminder_alert.rs:66` |
| the argument saying whether it was said | `wx_reminder_alert::Spoken::{NotYet, Already}`, seventh argument to `raise`, which now takes `&Arc<Accessibility>` | `:88` |
| the repeat rule | `wx_reminder_alert::RepeatingTone::from_the_window_opening(Instant)` and `.asked(Instant, bool) -> ToneNow::{Sound, Wait, Finished}`, with `BETWEEN_TONES` and `MOST_TONES` | `:99-160` |
| the rule about opening a modal | `one_question_at_a_time::whether_a_window_may_open(bool, bool) -> Moment::{Free, SomebodyIsTyping, SomethingIsAlreadyUp}` | `one_question_at_a_time.rs:154` |
| the shared typing helper | `wx_app::whether_somebody_is_typing(&[TextCtrl]) -> bool`, private to `wx_app.rs` | `wx_app.rs:10529` |
| the hold constant | `wx_app::LOOKS_TO_HOLD_A_REMINDER_WINDOW_WHILE_SOMEBODY_TYPES: u32 = 1` | `:10088` |
| the waiting cell | `BetweenLooks::said_and_waiting: RefCell<HashMap<String, u32>>`, looks waited by reminder id, beside `already` and the gate | `:10095` |

Two things 06-09 should know that its plan could not. The timer ticks once a second and the rule owns the minute, so a window that keeps the timer keeps `HOW_OFTEN_TO_ASK_THE_TONE` at `:169`. And ledger 383: after a hold, the gap from the reminder's own tone to the window's first repeat is two minutes, because `raise` is told only that the sentence was said, not when; handing it the instant would close that and was left for the window 06-09 replaces.

## The inherited item, read clause by clause

`ROADMAP.md` phase 6, inherited from phase 1: "A reminder alert still opens over somebody who is typing. It shares the one-at-a-time gate 01-10 built but does not ask the typing count. Whether a reminder should wait is a question about what a reminder is for."

- *Still opens over somebody who is typing.* It no longer opens at the look that finds it while somebody is typing. It opens at the next look, a minute later, whether or not they have stopped. So it still opens over somebody who is typing, later and after telling them. That is the decision, not a gap, and the roadmap line should say so rather than be struck as if the window never interrupts.
- *Does not ask the typing count.* Closed: `raise_what_is_due` asks it through `whether_somebody_is_typing`, which reads `one_question_at_a_time::somebody_is_typing()` and the three editable boxes, the same two halves the folders question reads, held by a reading in `tests/wired.rs`.
- *Whether a reminder should wait is a question about what a reminder is for.* Answered by Pratik on 2026-09-14: it waits one look and says itself meanwhile.

**Closed structurally, with the residual written into the roadmap line.** Nobody has heard it, and every clause that could close by listening is ledger 379 to 383.

## What the gate selects for each file touched

Checked from the hook's own output on each commit rather than from the mapper's source. `src/presentation/one_question_at_a_time.rs`, `wx_reminder_alert.rs`, `accessibility.rs` and `wx_app.rs` each map to their `--lib module::` run; `wx_app.rs` also pulls in seven suites coupled to it by `guards/guards.toml` (`a_whole_folder_moves_both_bounds`, `one_sign_in_per_piece_of_work`, `nothing_leaves_the_outbox_unasked`, `the_conflict_choice_can_be_heard`, `nothing_sends_a_flag_change_unasked`, `the_list_warning_reads_the_message`, `columns_belong_to_the_folder_they_were_arranged_in`). `tests/wired.rs` maps to `--test wired`, which is also one of the four whole-tree guards and so runs on every commit regardless. `guards/guards.toml`, `docs/changelog.md`, `Cargo.toml` and `Cargo.lock` map to no target, as the state-of-the-tree brief said; the count check that reads `guards.toml` and the em-dash guard that reads `changelog.md` both live in `house_style`, a whole-tree guard, so both ran on the commits that changed those files anyway. The whole gate, `scripts/check.sh all`, ran once on the branch before the merge and is quoted under Verification.

## Deviations from Plan

### 1. [Rule 3 - Blocking] `raise_what_is_due` at nine arguments

- **Found during:** Task 2, green.
- **Issue:** Adding `said_and_waiting` and `somewhere_to_type` to a seven-argument function made nine, and clippy runs with `-D warnings`.
- **Fix:** `BetweenLooks` holds `already`, `said_and_waiting` and the shared gate; the function takes it and destructures. The folders question borrows the gate from the struct and is otherwise untouched. The insert and remove on `already` are the same lines.
- **Files modified:** `src/presentation/wx_app.rs`.
- **Commit:** `106efe6c`.

### 2. [Rule 2 - Missing critical functionality] A stale waiting entry would open a window without its sentence

- **Found during:** Task 2, while writing the arms.
- **Issue:** A reminder said while typing and then completed from the panel before the next look would leave its id in `said_and_waiting`; if it ever came due again, its window would open with `Spoken::Already` and nothing said.
- **Fix:** `said_and_waiting` is pruned each look to the ids `what_is_due` returned.
- **Files modified:** `src/presentation/wx_app.rs:10599-10601`.
- **Commit:** `106efe6c`.

### 3. The timer ticks once a second, not once a minute, and does not stop itself

- **Found during:** Task 1, designing the tick.
- **Issue:** A sixty-second timer and a sixty-second rule miss each other by milliseconds and the tone would sound every two minutes; and stopping a timer from inside its own handler needs the handler to own the timer.
- **Fix:** `HOW_OFTEN_TO_ASK_THE_TONE = 1000`, the rule owns the spacing; a `Finished` tick does nothing and the drop after `show_modal` ends the timer.
- **Files modified:** `src/presentation/wx_reminder_alert.rs`.
- **Commit:** `8411f27a`.

### 4. One test-only accessor in `accessibility.rs`, and a one-line call-site change in task 1

- The `say` test needed to see what reached the bridge; `Accessibility` exposed nothing outside its own module. `#[cfg(test)] pub(crate) fn last_announcement` was added. No `#[test]` was added to `accessibility.rs`, so its seven records were not disturbed. And task 1's green commit changed `wx_app.rs`'s single call to `raise` to pass `Spoken::NotYet`, a file the task did not list, because the crate would not otherwise compile; task 2 rewrote that region.

### 5. The `say` test's first fixture was wrong about a default

- Written red asserting the tone sounded with a fresh `Accessibility`; sounds are off by default, which the plan's own `read_first` noted about `earcon`. Re-read once the code existed, as the brief requires: the fixture switches sounds on for the sounded half and gained a second half asserting the sentence still goes out with sounds off and the tone is reported as not sounded. The code was not changed to make the test pass.

### 6. Two readings in `wired.rs` rather than one

- The plan priced one `#[test]`; a source-reading guard without a companion is one that could be reading an empty string, so the sibling reading has one. The count check fires once for the file either way, and the fourteen records cost the same.

## Which assertions prove structure, and what only a person can settle

Structure, held by tests: the rule's three answers and their ranking; the sentence reaching the bridge and the tone reported honestly, with no window; the tone's spacing, latch and ceiling against handed-in instants; the reminder look handing the rule the helper's answer, taking the turn first and writing the reminder down before its window; both `wired.rs` readings shown their violations.

Experience, held by nobody: whether an Urgent announcement arriving mid-word reads as a warning or an interruption (379); whether the window arriving a minute later reads as helpful or as the same thing twice, and whether NVDA reads the dialog's text on focus so that saying it once is enough (380); whether ten tones a minute apart reads as being looked after or nagged, and whether `has_focus()` on the four controls answers as the toolkit says while another application is in front (381); whether a screen reader speaks the sentence at all with another application in front (382). Ledger 383 is the two-minute first gap after a hold. All five appended through `gsd-tools windows append`, both halves checked equal.

## Verification

- `cargo test --lib -- presentation::one_question_at_a_time:: presentation::wx_reminder_alert::`: 27 passed at task 1 green.
- `cargo test --lib -- presentation::one_question_at_a_time:: presentation::wx_reminder_alert:: application::due:: && cargo test --test wired`: green at task 2; `wired` 71 passed.
- `scripts/check.sh` through the hook on all four commits; two red commits held to exactly their named failures.
- `scripts/check.sh all` on the branch before the merge, not piped, twice. **The first run was red, and the cause was mine**: this summary was written to disk while the run was in progress, so `the_planning_files_agree_with_themselves` found five summaries against a state file saying 93 and a roadmap row saying 4/9. Every other target passed. That is the mistake observation 0526 in the skill log already names, editing the artefacts a long verification reads while it runs, and it was made anyway. The second run, with nothing touched, was green: 7,603 tests, 0 failed, release build, `cargo audit`, 282 seconds, exit 0. Ledger 374's `keyring` race did not fire on either run.
- `STATE.md` holds the plan number twice, and after the frontmatter was moved to 5 the body still said 4; `test_the_state_frontmatter_and_the_heading_name_the_same_plan` refused it until both said 5. Reading the diff had not caught it, which is the argument this project keeps making for guards nobody has to remember.
- `roadmap update-plan-progress` was not run, on the standing note that it is broken; `ROADMAP.md` and `STATE.md` were edited by hand and the diff read.
- `scripts/guards.sh --remeasure` on three records after task 1 and fourteen after task 2, both detached, every record reddening exactly what it names.
- No tracked file was edited by a script: `sed`, `printf >>`, `awk` in place and heredocs were used on nothing tracked. Exception set: zero. Carriage returns measured with `tr -cd '\r' | wc -c` on `guards.toml` and `changelog.md` before editing: zero in both.

## Threat Flags

None. No new network endpoint, auth path, file access or schema change. The one new surface is a timer inside a modal that reads focus and plays a tone the settings already govern.

## Self-Check: PASSED

Files: `src/presentation/one_question_at_a_time.rs`, `src/presentation/wx_reminder_alert.rs`, `src/presentation/wx_app.rs`, `src/presentation/accessibility.rs`, `tests/wired.rs`, `guards/guards.toml`, `docs/changelog.md`, `Cargo.toml` all present. Commits `eaacdf2f`, `8411f27a`, `081625e1`, `106efe6c` all in `git log`.

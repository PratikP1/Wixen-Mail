---
phase: 06-how-the-application-speaks
plan: 02
subsystem: ui
tags: [accessibility, feedback-channels, settings, guard-records, uia-msaa]

requires: ["06-01"]
provides:
  - "Switch, the three answers a person gives, each naming the channels it switches and carrying both its global and its per-event wording"
  - "A per-event panel on the Settings Feedback tab: a picker of the sixteen events, three controls, a button and two lines"
  - "One honest global control for speech and braille where there were two, with the sentence criterion 1 asks for beside it"
  - "A save path that writes what the tab holds rather than rebuilding from the stored value"
  - "wx_settings::read_settings made public, so what pressing OK would write is answerable without showing a modal"
  - "tests/every_event_has_a_control.rs, a live build of the settings dialog"
  - "tests/checkbox_labels.rs widened to the settings screen, shown to see a planted violation there"
affects: [06-04, 06-07, any plan that adds an event, a channel or a settings control]

actuals:
  tokens: 21500
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A screen-level type owning both the channels it switches and the two wordings it needs, so a label and the thing it switches cannot come apart"
    - "A live-widget test driving the real controls through the functions the handlers call, where the binding cannot raise an event, with the unproven link named in the file's own header"
    - "A guard record per rule rather than per task, where measuring a second candidate break shows the two guard different things"

key-files:
  created:
    - tests/every_event_has_a_control.rs
  modified:
    - src/presentation/accessibility/feedback.rs
    - src/presentation/wx_settings.rs
    - tests/checkbox_labels.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/KEYBOARD_SHORTCUTS.md
    - Cargo.toml
    - Cargo.lock
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md

key-decisions:
  - "Option 1, answered by Pratik on 2026-09-12, built as written and not re-argued"
  - "A control is ticked where any of the channels it stands for is on, not where all are, because a settings file can hold speech on and braille off and reading that as unticked would show silence to somebody who has announcements"
  - "Nothing is written for an event whose ticks still show what was painted into them, so visiting an event does not give it an answer of its own"
  - "Five guard records rather than the two the plan asked for, each measured, because the plan counted records per task and the rules are per rule"
  - "Channel::setting_label left with no shipping caller rather than removed, because the plan says not to change it and a test guards it. Recorded rather than absorbed"

requirements-completed: []

duration: 134min
completed: 2026-09-13
status: complete
---

# Phase 6 Plan 02: What the screen offers is what the program can do, Summary

**The sixteen per-event answers that have been in the settings file all along are reachable from a screen for the first time, and the two boxes that offered speech and braille as independent choices are one box that can mean what it says.**

## The checkpoint, answered

**Option 1, answered by Pratik on 2026-09-12.** A Choice of the sixteen events, with three controls and a "use the default for this event" button beneath it. Five controls on the page, however many events there ever are.

He did not take option 4's extra row setting one answer for everything. The plan's own reason for leaving it out stands and is recorded rather than re-argued: that row's state is ambiguous when the sixteen disagree, and it can be added later for the price of one row if a listening pass says the sixteen trips are the real problem.

The plan was built around option 1, so nothing was rewritten and no part of tasks 1 to 3 changed because of the answer.

Two things the plan says about this shape that this plan owns, both recorded rather than claimed. **The three controls reload beneath the cursor when the Choice changes**, which is a live-region shaped problem that nothing in this repository can prove reads well: `WINDOWS.md` 346. And **the Reading tab already uses a Choice with controls that reload beneath it**, so that existing pattern was followed rather than a second one invented. `build_reading_tab` was read before the panel was written, and the picker sits in a horizontal row with a `StaticText` label beside it and an accessible name set on it, which is exactly what `labelled_choice` does four times on that tab.

## Performance

- **Duration:** 134 min
- **Tasks:** 3, plus two additions the clause-by-clause read of criterion 1 demanded
- **Files:** 7 source and docs, 4 planning
- **Branch:** `what-the-screen-offers-is-what-the-program-can-do`

## Commits

| # | Commit | What |
|---|---|---|
| 1 | `b5c5d5bc` | Task 1 RED: failing tests for three answers, not four |
| 2 | `17a534dc` | Task 1 GREEN: the three answers, each knowing what it switches |
| 3 | `150a368b` | Task 2 RED: a failing check that every event is reachable from a screen |
| 4 | `cc4c90c5` | Task 2 GREEN: the panel, the honest global control, the version bump and the changelog |
| 5 | `6dfc3d1f` | Task 3: the label check learns about the settings screen |
| 6 | `268e3546` | The sentence criterion 1 ends on is held by something |
| 7 | `056a0f8e` | What pressing OK saves is held by something |

**Merged at:** `3969208e`, from the branch `what-the-screen-offers-is-what-the-program-can-do`. The merge itself ran the full gate on `main` and passed all four.

## What landed

**Three answers where the storage keeps four channels.** `Switch` has three variants, an `ALL`, a `channels()` and two wordings. `Switch::SpokenOrBrailled` names both Speech and Braille, because a notification goes out through one `UiaRaiseNotificationEvent` whose declared signature takes a provider, a kind, a processing hint, a string and an activity id, and no medium parameter at all. `Channel` is not collapsed to three: `carries_text()` needs braille to be its own channel for the never-sound-alone rule, and a settings file written before this holds the two separately.

**Two wordings, not one, with different accelerators.** "Play a short sound for each event" reads correctly above a list of everything and wrongly under a heading saying New mail. Both sets of three sit on the Feedback tab at once, so their accelerators are deliberately different keys: A, s, t globally and u, P, b per event, with E on the picker and f on the button. All ten distinct from each other and from the sound scheme's I and D, checked by reading the whole tab.

**The panel.** A picker holding all sixteen events by `Event::text()` in `Event::ALL`'s order, three check boxes, a button, and two lines. The ticks are painted from `what_was_chosen_for`, the first line from `channels_for`. Painting ticks from `channels_for` would have shown somebody answers they never gave, which is exactly what 06-01 built the second reader to prevent.

**The second line is what keeps the ticks honest.** It says what the selected event will really produce, because two rules bend an answer on its way out and neither is visible in a tick: a channel switched off everywhere stays off, and an event left with only a sound has the quietest written channel added back. The rule was not weakened to match the screen.

**Reset and "everything off" do not share a control.** The button calls `use_the_default_for`, which removes the answer. Switching all three boxes off calls `set_event_channels` with an empty set, which round trips and means silence. Both are guarded, each by its own record measured by hand.

**One global control where there were two.** `grep -n 'for channel in Channel::ALL' src/presentation/wx_settings.rs` finds nothing. The sentence beside it says whose decision the choice between speech and braille is.

**The save path writes what the tab holds.** Quoted, old and new:

> Old: `// Feedback channels. The per-event overrides in the stored value are preserved: this tab only decides which channels are on at all.`
>
> New: `// Feedback. The tab's working settings hold every per-event answer it has been given, including for events the picker is not showing, so what is on screen is remembered first and then the whole thing is written.` followed by four lines saying what the old comment said and when it stopped being true.

## Criterion 1, read from ROADMAP.md clause by clause

Read from source, line 530, not from the plan's paraphrase. **06-01's summary splits it into four clauses and there are five**: it folds "by keyboard" into the first. That is the 07-09 lesson happening again one plan later, and the folded clause is the one this plan can least attest to.

| # | Clause | Closes? |
|---|---|---|
| 1 | "A user sets Earcon and Visual independently for each of the sixteen events from the Settings Feedback tab" | **Closed structurally, unheard.** The picker holds `Event::ALL.len()` entries read from `Event::ALL`, and "Play a sound for this event" and "Show this event in the status bar" are separate boxes. A live test builds the real dialog and reads all of it back. |
| 2 | "by keyboard" | **Not closed.** Every control is a native Choice, CheckBox or Button, so Tab reaches them, and every check box and the button carries a distinct mnemonic. Both facts were established by reading. Nothing has tabbed through this tab, no test presses a key, and this project's own history is that a call which looks like accessibility need not be one. `WINDOWS.md` 353. |
| 3 | "and the setting survives a restart" | **Closed structurally, unheard.** This clause was open when the work started and nothing held it. Commit 7 closes it: the check ticks a real control, calls the real save path and reads the result back through `from_stored`, which is what a restart does. |
| 4 | "Speech and Braille are set **together**, as one choice" | **Closed structurally, unheard.** One control, guarded three ways: a unit test counting that every channel is named by exactly one switch, the live check counting three controls not four, and a save-path check that switching the one control off switches both channels off. |
| 5 | "and the screen the user meets says that choosing between them is done in their screen reader rather than here" | **Closed structurally, unheard.** The sentence is on the tab. It was unguarded until commit 6; deleting it would have broken the criterion and reddened nothing. |

**So four of five clauses close structurally and none of the five is heard.** Clause 2 does not close at all. **Criterion 1 as a whole does not close**, and what remains is a listening pass, which is 06-06's subject.

**Clauses 3 and 5 were open when this plan's tasks 1 to 3 were finished, and the plan's own success criteria claimed both.** Nothing in the tree would have said so. That is what the clause-by-clause read is for, and it is the single most useful thing that happened in this plan.

## Which assertions prove structure and which only a person can settle

Said plainly, because this project has shipped sixteen widgets "named" by a call that never reaches the accessibility tree, compiling and passing 324 tests.

**Structure, proved:** that the picker holds sixteen entries and each is the event's own text; that six check boxes carry a non-empty label of their own, which is what puts a name on the UI Automation channel Narrator reads; that three controls exist where there were four; that a per-event answer is kept when the picker moves away and back; that the button removes an answer rather than emptying it; that the save path carries a tick into the settings file and back; that the sentence about whose choice it is exists and names the screen reader.

**Experience, unproved, and only a person settles it:** whether any of it reads well; whether the three controls reloading beneath the cursor is announced at all, which is the live-region shaped question; whether the two lines beneath the controls are met at the moment they would help or are heard as noise after every change; whether "Announce events through your screen reader" is understood by somebody who used to see two boxes; whether the sentence about whose decision it is lands; and whether the tab can be walked by keyboard at all, which nobody has tried. Nine `WINDOWS.md` entries, 345 to 353, one per unrun thing.

**Structure, proved by reading rather than by running:** that the picker's selection handler and the button's click handler call the functions the tests drive. wxdragon 0.9.17 exposes no way to raise a widget event from outside, so the test moves the real Choice and reads the real check boxes but cannot open the picker or press the button. The behaviour is proved; the wiring is two lines somebody read. `WINDOWS.md` 349, and the test file's header says it in as many words rather than letting the test's name cover both halves.

## What the gate selects for what was touched

Checked rather than assumed, because phase 7 found four holes in this mapper and `which-checks.sh` has a fifth.

| File | `which-checks` on a branch | Targets `check.sh` selects |
|---|---|---|
| `src/presentation/accessibility/feedback.rs` | `affected` | `--lib presentation::accessibility::feedback::`, plus the four tree-reading guards |
| `src/presentation/wx_settings.rs` | `affected` | `--lib presentation::wx_settings::`, **which matches no test**, plus `every_event_has_a_control` and `checkbox_labels` once the new records coupled them, plus the four tree-reading guards |
| `tests/every_event_has_a_control.rs` | `affected` | its own target |
| `tests/checkbox_labels.rs` | `affected` | its own target |
| `guards/guards.toml` | `affected` | none of its own; only the four tree-reading guards |
| `Cargo.toml`, version line only | `affected` | the whole affected set, via `only_the_packages_own_version_moved` |
| `docs/*.md` | `docs_only` | the targets that read documents |

**`src/presentation/wx_settings.rs` maps to `--lib presentation::wx_settings::`, which matches nothing**, because that file holds zero `#[test]` functions. That is a fifth member of the family phase 7 found, alongside `src/main.rs` mapping to `--lib main::`. It is not a new defect in this plan's path, because the two new records now couple that file to two real suites and the gate named both on every commit that touched it. Before those records it was a file whose only scoped answer was a filter matching nothing.

**Two misuses of `which-checks.sh` met and worth writing down.** The known one, passing a path as the first argument, which makes it answer `all_but_slow` for everything. And a new one: `Cargo.toml` answers `all` when asked about an unstaged change, because `only_the_packages_own_version_moved` reads `git diff --cached` and a check that cannot see what changed correctly refuses the softer answer. Ask it after staging. The plan's claim that a version-only bump answers `affected` is right; the first measurement of it was a misuse.

## Guard records: five, not two, all measured

The plan's verification says `guards/guards.toml` gains two records, one per task 1 and task 2, and none for task 3. It gains five. Each is a rule, each break was applied by hand, and every one was then re-run through `scripts/guards.sh` itself, which reported for all six new and touched records: *the one test named went red, and nothing else did*.

| Record | Break | Measured red |
|---|---|---|
| the one control standing for words names braille as well as speech | `SpokenOrBrailled` given only Speech | 1 test, whole library at 7,126 |
| moving the event picker writes down what is on screen before painting over it | the flush removed from `show` | 1 test, 5 findings |
| the button putting an event back to the default removes its answer rather than emptying it | `set_event_channels` with an empty set in place of `use_the_default_for` | 1 test, 5 findings |
| the screen says whose decision speech or braille is | the realistic near-miss sentence | 1 test, 2 findings |
| pressing OK writes the answers the panel holds rather than rebuilding from the stored value | the save path exactly as it stood before this plan | 1 test, 1 finding |
| a check box on the settings screen carries its own label too | one per-event control built with an empty label, accessible name left in place | 1 test, 3 findings |

**Where the count went, and why the plan's arithmetic was the wrong shape.** The plan counts records per task. A record guards a rule, and task 2 has two rules the phase's own work says must never be conflated, while task 3 widens a guard to a second file that nothing then couples to it. Counting per task gave two; counting per rule gives five.

**The task 3 record is the one the plan says is not owed, and the plan is right about the wrong thing.** The existing `checkbox_labels` record still describes its own break, which is true. It names `src/presentation/wx_item_form.rs`, so after the widening that guard would have run when the item form changed and on no commit touching the second screen it now also guards. That is guardrail 4 exactly. `check.sh --suites-for` now answers both suites for `wx_settings.rs`, checked after the record landed.

**A second candidate break was measured twice and both times it was not weaker.** For task 2, emptying an answer instead of removing it reddens the same suite with the same count as removing the flush, so the two are equally strong and guard different rules; both were recorded rather than one chosen. For commit 7, writing only the first channel a switch answers for reddens on its own with a different sentence, proving the two halves of that check discriminate independently; it is written into the record's comment rather than given a record, because the recorded break is the one that was really there.

**Census:** 735 records on arrival, 741 on departure, with the head of the file bumped in the same commit as each record. 192 swept plus 549 not swept.

## Measurements taken

| Thing | Before | After |
|---|---|---|
| `#[test]` in `feedback.rs`, by the check's own rule | 47 | 49 |
| `#[test]` in `tests/checkbox_labels.rs` | 1 | 1 |
| `#[test]` in `tests/every_event_has_a_control.rs` | none, no file | 1 |
| Records in `guards/guards.toml` | 735 | 741 |
| Records naming `feedback.rs` | 2 | 3 |
| Records naming `wx_settings.rs` | 3 | 8 |
| `.planning/WINDOWS.md` entries | 344 | 353 |
| Lines holding `set_accessible_name` in `wx_settings.rs` | 58 | 61 |
| Version | 0.119.0 | 0.120.0 |

**The `set_accessible_name` figure is lines, not calls, and includes `set_accessible_name_and_description`.** The three new lines are the event picker, the per-event tick loop and the reset button. The global loop's single line did not move in count because it was always one line inside a loop, and it now names three controls where it named four.

**No remeasure was owed for `tests/checkbox_labels.rs`**, because no `#[test]` was added to it: the settings walk went inside the existing one. Confirmed by taking the count with the check's own rule before and after, rather than assumed.

**The remeasure that was owed was run and read.** Adding two tests to `feedback.rs` flagged two records, not the one task 1 predicted, the same way 06-01's did. `scripts/guards.sh --remeasure` reported both redden exactly the tests they name and wrote their counts down again.

## Every new control and the label it was built with

| Control | Kind | Built with |
|---|---|---|
| Announce, global | `CheckBox` | `with_label("&Announce events through your screen reader")` |
| Sound, global | `CheckBox` | `with_label("Play a short &sound for each event")` |
| Status bar, global | `CheckBox` | `with_label("Show events in the s&tatus bar")` |
| The event picker | `Choice` | no label of its own; a `StaticText` "&Event:" beside it and `set_accessible_name(.., "Event")` |
| Announce, one event | `CheckBox` | `with_label("Anno&unce this event through your screen reader")` |
| Sound, one event | `CheckBox` | `with_label("&Play a sound for this event")` |
| Status bar, one event | `CheckBox` | `with_label("Show this event in the status &bar")` |
| Use the default | `Button` | `with_label("Use the de&fault for this event")` |

**Every check box and the button carries a real label.** None is named by `set_accessible_name` alone, which would put a name on MSAA for NVDA and none on UI Automation for Narrator.

**The picker cannot carry one**, because a `Choice` has no text of its own, so it takes the pattern every other Choice in this dialog takes. Its name reaches MSAA from `set_accessible_name`; on UI Automation it arrives only through Windows falling back to the nearest static text, which is a fallback rather than something this code set. Neither channel has been checked for it. `WINDOWS.md` 350.

## Deviations from plan

### 1. Five guard records, not two

Covered above. Additive, each measured, each verified by `guards.sh`.

### 2. Two checks added that the plan did not ask for, because criterion 1 asked for them and nothing held them

Commits 6 and 7. The plan's `<behavior>` for task 2 lists both ("Pressing OK saves every per-event answer and every global answer"; "a sentence beside them says that which of the two somebody gets is their screen reader's decision") and its verification lists neither. Reading the criterion from the roadmap is what found it. `read_settings` became public to make the second reachable, with a doc comment saying why.

### 3. The two `house_style` settings guards did not redden on arrival

The plan says: "Add the controls, watch both redden, and report which. That is the red half for free." Both stayed green. They fire on a control written the wrong way, not on a new control, and these were written the right way first. Collecting that red would have meant writing the defective version on purpose, which records a failure nobody would have made.

What was done instead: each guard was shown to see a planted violation **in this new code**, quoted, and the code put back by hand.

```
src/presentation/wx_settings.rs:2539: planted_and_forgotten, built in build_feedback_tab
src/presentation/wx_settings.rs:2486: tick is told true outright
```

The real red half for task 2 was `tests/every_event_has_a_control.rs` failing against a stubbed panel, which is what commit 3 records.

### 4. The plan cites the wrong lines for the UI Automation signature

`<action>` for task 1 says to name `src/presentation/accessibility/screen_reader.rs:75-81` as where the signature is. That range is the `Processing` enum. `UiaRaiseNotificationEvent` is declared at `screen_reader.rs:91-97`. The doc comment names the correct range.

### 5. The plan predicted one record would need remeasuring and two did

The same deviation 06-01 recorded, in the same file, one plan later. Two records name `feedback.rs` and adding tests moves the count on both. A plan that predicts one remeasure has predicted the cost of its own record rather than the cost of the file.

### 6. `Cargo.lock` was left out of the version-bump commit and added by amending

Caught by checking the working tree after the commit rather than by anything failing. Worth recording because of what the amend then did: the commit hook scopes its checks to the staged diff, so on the amend it saw a one-file change and ran only the tree-reading guards, where the original commit had run the module's tests and both coupled suites. Nothing was at risk here, since the only new content was a version line, but an amend that adds real code gets the same narrowed check and the output reads exactly like a full pass.

### 7. The roadmap's progress table and the second copy of the plan number were missed on the first pass

Both updated by hand, both caught by the gate rather than by me, and both recorded under Gate below. `roadmap update-plan-progress` is avoided here, which means a person updates a progress row in `ROADMAP.md`, a plan list in the same file, and a plan number that `STATE.md` holds in two places. Three of those four were done and the fourth was not.

### 8. `Channel::setting_label` now has no shipping caller

The global boxes are built from `Switch`, so its four strings are reached only by its own two tests. The plan says "Do not change them" and a test guards them, so it was left rather than removed quietly. `WINDOWS.md` 351.

`grep -rn 'set_event_channels|use_the_default_for|what_was_chosen_for'` outside `feedback.rs` now finds real callers in `wx_settings.rs` for all three, so 06-01's caution that `set_event_channels` was public with no caller outside its module is answered.

## Anything false in the plan against the tree

Beyond deviations 3, 4 and 5 above:

- **`<premise_corrections>` item 5 says the two `house_style` guards "will fire on arrival, and that is the red half for free".** They did not. Deviation 3.
- **The plan's verification says "`.planning/WINDOWS.md` gains four entries".** It gains eight. The four it names are all there; the other four are the wiring proved by reading, the picker's name reaching only one channel by anything this code set, `Channel::setting_label` orphaned, and the label walk still unable to reach any settings check box but the six.
- **The plan's task 2 action says to record a break of "remove one variant from the list `Switch::ALL` holds".** That break cannot be recorded: `Switch::ALL` is `[Switch; 3]`, so removing a variant is a compile error, and `guards.py` reads cargo's `FAILED` lines. It is the same finding 06-01 recorded about its own task 1, and the plan repeats the shape.
- Everything else in `<premise_corrections>` checked out: the `Channel::ALL` loop at 2159, the `||` in `accessibility.rs`, `setting_label`'s four callers, `build_settings_dialog` being public and already built by `tests/theme_reach.rs`, `tests/checkbox_labels.rs` holding one `#[test]`, and `wx_settings.rs` holding zero.

## Known stubs

None. Every control added is wired to the working settings, every method added has a caller in the running program, and the save path reads all of them.

## Requirements and criteria: what does not close

**FEEDBACK-01 stays Pending.** The phase README assigns it to 06-01, 06-02 and 06-04 together. Its evidence paragraph in `REQUIREMENTS.md` is now wrong about the tree in a second way, since `read_settings` and the panel exist; that was already `WINDOWS.md` 344 and is still decision 7 at 06-07, so it was not corrected here.

**Criterion 1 does not close.** Four of five clauses close structurally, clause 2 does not close at all, and none of the five is heard.

## Gate

`scripts/check.sh all` was run on the branch before the merge, redirected to a file and never piped. All four passed: rustfmt, clippy with `-D warnings`, the whole suite at 7,126 passed with 1 ignored and 0 failed, and the release build. The shell suites that decide what runs passed in the same run.

**It took three runs and the middle one is worth recording, because it failed and the failure was real.** The first covered commits 1 to 5 and passed. The second was started before the merge and the planning documents were updated while it ran, which is a mistake in itself: some tests in that suite read those documents, so its verdict was about a state that existed for as long as it took to save a file. It came back red on four checks in `tests/the_planning_files_agree_with_themselves.rs`, and those four were right:

```
.planning/STATE.md: the frontmatter says current_plan 2 and the body says Current Plan: 1
.planning/ROADMAP.md: row 6. How the application speaks says 1/8 and phase 6 holds 2 summaries and 8 plans on disk
```

Both are the value-stored-twice defect those checks exist for, and the guard's own message names the plan that left it behind last time. Each fired twice, once as the check and once as its companion refusing to run while the real check is already red, which is the right shape: a companion that spliced its own violation into a file already holding one would prove nothing.

**A green from a mid-edit run would have been worse than that red**, because red gets re-examined and green gets believed. The third run was started with the tree settled and nothing was touched until it finished, and it is the one this section reports.

That failure is also the answer to why `roadmap update-plan-progress` being broken here matters: doing it by hand means updating a progress table and two halves of a plan number that live in three places, and the only thing that catches a missed half is a check that runs on documents.

Every commit went through the `commit-msg` hook. Nothing used `--no-verify`. No tracked file was edited by a script: every change to source, to `guards/guards.toml` and to every document was made with the editor, and the only script that wrote to a tracked file is `scripts/guards.sh --remeasure`, which is the project's own tool and is what `CLAUDE.md` prescribes.

## Self-Check: PASSED

Every file this summary says was created exists on disk, every commit hash it names resolves in `git log`, and the counts in the tables above were taken with the commands beside them rather than carried over from the plan. The one figure this summary got wrong on the first pass was records naming `wx_settings.rs`, written as 7 from arithmetic and corrected to 8 by parsing `tests_last_seen` blocks, which is `CLAUDE.md`'s own rule about not writing a count you have not just taken, met by breaking it once.

## Next

06-03 can start. 06-04 inherits a panel that is built and unheard, and 06-06 inherits seven listening questions.

---
*Phase: 06-how-the-application-speaks*
*Completed: 2026-09-13*

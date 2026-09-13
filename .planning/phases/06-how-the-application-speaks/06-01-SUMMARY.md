---
phase: 06-how-the-application-speaks
plan: 01
subsystem: ui
tags: [accessibility, feedback-channels, earcons, macro-rules, guard-records]

requires: []
provides:
  - "FeedbackSettings::what_was_chosen_for, the first public way to read an event's chosen channels, with no override told apart from every channel ticked"
  - "FeedbackSettings::use_the_default_for, which removes the entry rather than storing an empty one"
  - "FeedbackSettings::set_event_channels made public, so something outside this module can write an override at all"
  - "enum Event and Event::ALL generated from one list, so a seventeenth event cannot exist outside ALL"
  - "A by-hand measurement of what nothing noticed before, on both sides of the change"
  - "One guard record for the round trip, with its red list measured rather than predicted"
affects: [06-02, 06-04, any plan that adds an event or a channel]

actuals:
  tokens: 3792
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A declarative macro generating an enum and its ALL array from one list, on the menu_ids! precedent, where a hand-written array beside exhaustive matches would go stale in silence"
    - "Two readers for one preference: what was chosen, and what is effective after defaults and fallbacks, with the doc comment on each naming the other"

key-files:
  created: []
  modified:
    - src/presentation/accessibility/feedback.rs
    - guards/guards.toml
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md

key-decisions:
  - "The macro, not the source-reading count. It makes the disagreement unrepresentable rather than detectable, and costs neither a guard record nor a companion proving the reading can see a violation"
  - "No guard record for task 1, because the break fails to compile rather than reddening a test and guards.py reads cargo's FAILED lines"
  - "use_the_default_for is named for what the settings button says rather than for what happens to the Vec, because a name reading as clear invites being used for switch everything off"
  - "Channel::ALL carries the same hole and is left open on purpose, recorded as WINDOWS.md 343 rather than fixed quietly"
  - "FEEDBACK-01's evidence paragraph is now wrong about the tree and was not corrected here, because whether REQUIREMENTS.md is corrected in place is decision 7 and belongs to Pratik at 06-07"

patterns-established:
  - "A compile-time defect is measured by hand on both sides and reported as a deviation, rather than given a guard record that would ship with an empty red list"
  - "A guard break whose text appears twice in the file takes the enclosing signature line with it, because guards.py requires exactly one occurrence"

requirements-completed: []

duration: 68min
completed: 2026-09-13
status: complete
---

# Phase 6 Plan 01: What somebody chose, apart from what they get, Summary

**Two new readers and a public writer on `FeedbackSettings`, so a panel can show the choice as it was made rather than the effective set, and `Event` with `Event::ALL` generated from one list so a seventeenth event cannot exist without a control.**

## Performance

- **Duration:** 68 min
- **Started:** 2026-09-12T23:27:47Z
- **Completed:** 2026-09-13T00:36:00Z
- **Tasks:** 2
- **Files modified:** 2 source, 3 planning

## Accomplishments

- `what_was_chosen_for` answers what somebody ticked. `None` where nobody has touched the event, `Some(set)` where they have, and `Some(empty)` where they switched everything off for it, which is a different answer from `None` and means silence. No default filled in, no intersection with the globally switched-off channels, no never-sound-alone fallback.
- `use_the_default_for` removes the entry so the default comes back, and does nothing where there is no entry.
- `set_event_channels` is public. Before this it had eleven references and every one was in this file, so the only route into a per-event override was editing the stored settings string by hand.
- `enum Event` and `Event::ALL` are generated from one list. A variant left out of `ALL` is no longer possible to write.
- `channels_for`, `to_stored` and `from_stored` are byte identical to what they were. Checked by diffing the whole region from `pub fn channels_for` to the next item against `main`: no difference at all.

## Task Commits

1. **Task 1: Event and Event::ALL come from one list** - `dcbe5ee4` (refactor)
2. **Task 2 RED: failing tests for what somebody chose** - `1b4e15f3` (test)
3. **Task 2 GREEN: the three methods and the guard record** - `1e67fb9a` (feat)

**Plan metadata:** `a21c89b2` (docs: the summary, the ledger, and phase 6 opened)
**Merged at:** `dacaa719`, from the branch `what-somebody-chose-is-not-what-they-get`

## The declared test-first exception in task 1

The plan declared this before execution began and required three things. All three were done.

**Why no test could have been written.** The defect is compile-time. `Event::ALL` was the only enumeration of the variants that exists in Rust, so nothing could call a check for a variant absent from it. An exhaustive match forces a new variant to be answered; it cannot force one to be counted. This is not one of the four exceptions `CLAUDE.md` lists: it is not configuration, not a document, not glue and not styling. It is a refactor whose whole purpose is a property the compiler holds. The safety net was the forty-one tests already in the file, which were unchanged in number and all still green, plus the compiler.

**The before measurement, with its output.** A seventeenth variant added to `enum Event`, answered in all four exhaustive matches so it compiled, and deliberately left out of `ALL`:

```
cargo test --lib -- presentation::accessibility::feedback:: presentation::accessibility::sound_scheme::
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 7059 filtered out

WIXEN_TEST_THREADS=8 cargo test --lib
test result: ok. 7118 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 74.43s
```

The plan asked only for the scoped run. The whole library was run as well, because the claim being tested is that nothing notices, and the scoped run can only speak for sixty tests. Nothing noticed. Reverted.

**The after measurement.** The same variant added to the one list and nothing else touched:

- It is in `Event::ALL` without anybody having touched `ALL`. Proved by a throwaway assertion, `assert_eq!(Event::ALL.len(), 17)` and `assert!(Event::ALL.contains(&Event::MeasurementOnly))`, which passed.
- Taking it back out of the list is a compile error and not a green run:

```
error[E0004]: non-exhaustive patterns: `&feedback::Event::MeasurementOnly` not covered   (x4)
error: could not compile `wixen-mail` (lib) due to 4 previous errors
  --> src/presentation/accessibility/feedback.rs:158:15   key
  --> src/presentation/accessibility/feedback.rs:188:15   text
  --> src/presentation/accessibility/feedback.rs:232:15   priority
  --> src/presentation/accessibility/feedback.rs:267:15   tone
```

Reverted. The throwaway assertion is not in the tree; `git status` was clean of it before task 1 was committed.

**No guard record for it, and why.** A record names an edit that should break the guard and the tests that should go red. This edit does not redden a test, it fails to compile, and `scripts/guards.py` reads cargo's `test NAME ... FAILED` lines. A record here would ship with an empty red list, which is a record that can never be measured. A break that reddens nothing is a finding, and the finding is that the compiler is doing the work.

**The doc comments survived word for word.** The declaration region before and after was diffed with leading whitespace stripped. Seventy-four lines, the enum's own doc comment and all sixteen variants with theirs, identical. The only differences are the two the macro wrapping makes: `#[derive(...)]` plus `pub enum Event {` became `events!(`, and the closing brace plus the hand-written sixteen-element `ALL` became `);`. Every derive survives, inside the macro body. `grep -n 'Event; 16\]'` finds nothing, and neither does a search for a bare `16` anywhere in the file: the length is counted from the list.

## What the gate selects for what was touched

Checked rather than assumed, because phase 7 found four holes in this mapper. `scripts/which-checks.sh` takes the branch as its first argument; passing a path as the first argument makes it answer `all_but_slow` for everything, which is the no-file-list answer and reads like a finding when it is a misuse.

| File | which-checks on a branch | Targets `check.sh` selects |
|---|---|---|
| `src/presentation/accessibility/feedback.rs` | `affected` | `--lib presentation::accessibility::feedback::`, plus the four whole-tree guards |
| `guards/guards.toml` | `affected` | none of its own; only the four whole-tree guards |
| `.planning/*.md` | `docs_only` | the targets that read documents |

`check.sh --suites-for` answers nothing for either file, so no integration suite is coupled to this module. The count check that fires when a test is added lives in `tests/house_style.rs`, which is one of the four whole-tree guards, so it runs on every commit in every mode. That is why the red commit could name it and be held to it.

On `main` the same source file answers `all`. Nothing here touches `Cargo.toml`, so no commit in this plan earned the release build until `scripts/check.sh all` was run by hand before the merge.

## Measurements taken

| Thing | Before | After |
|---|---|---|
| `#[test]` in `feedback.rs`, by the check's own rule | 41 | 47 |
| Library tests, `cargo test --lib` | 7,119 | 7,125 |
| Records in `guards/guards.toml` | 734 | 735 |
| Records naming `feedback.rs` | 1 | 2 |
| `.planning/WINDOWS.md` entries | 342 | 344 |

The guard census at the head of `guards/guards.toml` was bumped in the same commit: 192 swept plus 543 not swept, which sums to 735 and is what `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` reads.

**The guard record's red list was measured, not predicted, and the prediction happened to be right.** The break is the wrong implementation a reader would write: `use_the_default_for` pushing an empty set instead of removing the entry. Applied by hand, whole library, `--no-fail-fast`: 7,121 passed, 3 failed, and the three are exactly the three the record names. The prediction written down before the run was the same three. Worth saying because `CLAUDE.md` warns that the first plausible break is often the weak one and that one record here named eight tests for a break that reddens seventeen. This one was not that case.

**The break takes its enclosing signature line with it.** `self.per_event.retain(|(e, _)| *e != event);` appears twice in the file, in `set_event_channels` as well, and `guards.py` refuses a break whose text is not exactly one occurrence. The record's `before` is the whole three-line function.

**`scripts/guards.sh --remeasure` was run and its output read.** Two records, not the one the plan expected, because both records naming this file had their counts moved from 41 to 47 and the plan only accounted for the new one. It reported: "the one test named went red, and nothing else did" for the older record, "all 3 tests named went red, and nothing else did" for the new one, and "All 2 guards redden exactly the tests their records name." It wrote no change to `guards/guards.toml`, so the counts written by hand were what the measurement would have written.

## Deviations from Plan

### 1. The plan and the phase README both say three exhaustive matches. There are four.

- **Found during:** Task 1, at the first compile of the by-hand measurement.
- **Issue:** `<premise_corrections>` item 4 and the README both say "three exhaustive matches already force a new variant to be described". `key`, `text`, `priority` and `tone` all match on `self` with no wildcard, so a new variant produces four errors.
- **Impact:** None on what was built. The argument the plan makes is about whether an exhaustive match can close the hole, and it is equally true of four. The macro's doc comment says four.
- **Committed in:** `dcbe5ee4`, and the commit message says so.

### 2. The remeasure was two records, not one.

- **Found during:** Task 2, after the green commit.
- **Issue:** The plan says "One record, so this is one build and one run." Two records name this file once the new one is added, and adding six tests moved the count on both. The older record's red list had not been checked against a tree six tests larger.
- **Fix:** Both were remeasured. Both are right.
- **Why it matters beyond this plan:** `CLAUDE.md` warns that a record goes stale when tests arrive near the rule it is about, and the count check names every record that could be affected, not only the new one. A plan that predicts one remeasure has predicted the cost of its own record and not the cost of the file.

### 3. A ledger entry's line number was written stale and corrected by hand in both halves.

- **Found during:** Recording `Channel::ALL`.
- **Issue:** `gsd-tools windows append` was given line 327, which was where `Channel::ALL` sat before the macro moved it. It is at 348.
- **Fix:** `windows` has no edit subcommand, so both halves were corrected with the editor, not with a script, and `test_both_halves_of_the_ledger_say_the_same_thing` was run and passes.

### 4. FEEDBACK-01's evidence is now wrong about the tree and was deliberately not corrected.

- **Found during:** Checking whether the requirement closes.
- **Issue:** `.planning/REQUIREMENTS.md:1288` says `set_event_channels` is private, "so no screen could write one without changing a visibility". It is now `pub`, there is now a public reader, and four of the line numbers it quotes have moved.
- **Why not fixed here:** Whether `REQUIREMENTS.md` is corrected in place is decision 7, put to Pratik at 06-07. Correcting it now pre-empts that.
- **Recorded as:** `.planning/WINDOWS.md` 344, kind `unmet-truth`.

**Total deviations:** 4, none of which changed what was built.

## Known Stubs

None. Every method added has a body, a caller in the tests and a reason to exist that 06-02 acts on.

## Requirements and criteria: what does not close

**FEEDBACK-01 stays Pending.** The README assigns it to 06-01, 06-02 and 06-04 together.

**Roadmap criterion 1 does not close, and it was read clause by clause rather than from a paraphrase.** Four clauses:

1. "A user sets Earcon and Visual independently for each of the sixteen events from the Settings Feedback tab, by keyboard" - open. There is no panel. 06-02.
2. "and the setting survives a restart" - the model half is proved. An override written through the public writer round trips through `to_stored` and `from_stored`, and one put back to the default stores as nothing at all rather than as an empty entry. Nothing writes one from a screen, so the clause as written is open.
3. "Speech and Braille are set together, as one choice" - open. 06-02.
4. "the screen the user meets says that choosing between them is done in their screen reader rather than here" - open. 06-02.

**No version bump and no changelog entry, as the plan instructed.** Nothing a user can reach changed. The model gained three methods and nothing in the running program calls two of them. 06-02 is what a person meets and where the entry belongs.

## Out of scope and recorded

**`Channel::ALL` carries exactly the same hole.** It is still a hand-written `[Channel; 4]`, now at `feedback.rs:348`, and nothing forces a fifth channel into it. Left open on purpose: four is a much smaller surface than sixteen, nothing in this phase adds a channel, and closing it is not free work this plan should absorb. `.planning/WINDOWS.md` 343.

## Issues Encountered

**The red half could not be written as tests alone.** `red-commit.sh` requires every named test to have run and failed, and a test calling a method that does not exist never runs, because the crate does not compile. Two `todo!()` stubs were added in the red commit so the six tests compile and fail on a panic rather than failing to build. The green commit replaced the bodies. This is worth writing down because it is a property of the gate rather than of this plan: in this repository a red commit for a new function always carries that function's signature.

**The red commit named seven tests, six plus the count check.** There is no ordering that avoids it. Adding tests to a file makes every record that fingerprints it disagree with the tree, and correcting a record needs the green code first. The whole task is therefore one red commit, which is what `CLAUDE.md` prescribes, and the message says so.

## What 06-02 inherits

- `what_was_chosen_for(event) -> Option<BTreeSet<Channel>>`. Render the panel from this. Rendering from `channels_for` would show four ticks for an event nobody has touched and a braille tick nobody set, which is a screen telling somebody they chose something they did not.
- `channels_for(event) -> BTreeSet<Channel>`, unchanged, which is what to announce as the effective result. The two are different answers to different questions and each doc comment names the other.
- `use_the_default_for(event)` for the button that puts one event back, and `set_event_channels(event, BTreeSet::new())` for switching everything off, which are opposite and must not share a control.
- `Event::ALL` in the order it is written, which the panel reads, still `[Event; N]` and still sixteen long.
- A caution: `set_event_channels` is `pub` and has no caller outside this module yet. `grep -rn 'set_event_channels' src/ tests/` still answers only `feedback.rs`. When 06-02's panel arrives it becomes the first, and `dead-code-hunter` should be pointed at `use_the_default_for` and `what_was_chosen_for` after that, because until then they are reachable only from tests.

## Gate

`scripts/check.sh all` was run on the branch before the merge, redirected to a file and never piped. All four checks passed: rustfmt, clippy with `-D warnings`, the whole suite at 7,124 passed with 1 ignored and 0 failed, and the release build. The shell suites that decide what runs passed in the same run.

Every commit went through the `commit-msg` hook. Nothing used `--no-verify`.

## Next Phase Readiness

06-02 can start. Its dependency is satisfied and nothing it needs from this plan is a stub.

---
*Phase: 06-how-the-application-speaks*
*Completed: 2026-09-13*

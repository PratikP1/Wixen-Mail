---
phase: 02-search-that-says-what-it-covers
plan: 04
subsystem: search
tags: [filters, saved-searches, accessibility, vocabulary, guards]
status: complete
requires:
  - "A_FIELD_A_RULE_MAY_NAME and A_WAY_A_RULE_MAY_MATCH, the eleven-and-eleven vocabulary"
  - "RULE_ACTIONS, shown_action and stored_action, the stored-and-spoken shape this copies"
  - "build_filter_edit_dialog, already split from show_filter_edit so a test can build it"
provides:
  - "filters::WHAT_EACH_FIELD_IS_CALLED and WHAT_EACH_WAY_OF_MATCHING_IS_CALLED, stored name to spoken words"
  - "filters::the_words_for_a_field and the_field_those_words_name, converting each way"
  - "filters::the_words_for_a_way_of_matching and the_way_of_matching_those_words_name"
  - "filters::A_WAY_OF_MATCHING_THAT_READS_NO_PATTERN and a_way_of_matching_compares_against_nothing"
  - "wx_managers::the_pattern_box_asks_for_something, the one place the Pattern box's presence is decided"
  - "wx_managers::the_pattern_to_store, so a rule stores no pattern where nothing compares one"
  - "tests/manager_dialog_labels.rs, the first control-name coverage the manager dialogs have had"
  - "one guard record, measured by hand, running against its own suite"
affects:
  - "every stored filter rule opened in the editor: the lists now hold words, not stored names"
  - "plan 02-05's rule editor, which reads the same pairs rather than writing a third list"
  - "plan 02-06's sentence builder, which no longer has to read a pattern nothing compares"
tech-stack:
  added: []
  patterns:
    - "a stored-and-spoken pair slice beside the constant it describes, not beside the dialog"
    - "a lookup returning None for an unknown name rather than echoing it back"
    - "one integration test holding every window-building check, because the process budget is one"
    - "a pure decision function for the half of a dialog a built window cannot reach"
key-files:
  created:
    - tests/manager_dialog_labels.rs
  modified:
    - src/application/filters.rs
    - src/presentation/wx_managers.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/phases/02-search-that-says-what-it-covers/deferred-items.md
decisions:
  - "The built-dialog guard lives in tests/manager_dialog_labels.rs, not the library test module the plan named. A process may call wxdragon::main once, measured, and the plan's cited reason was about source-reading tests."
  - "The guard's break reintroduces a hardcoded list rather than shrinking the constant. Both sides of the check read that constant, so shrinking it moves them together and the guard stays green."
  - "The Pattern box is disabled rather than removed, so the tab order does not move under somebody working by ear."
  - "A_WAY_OF_MATCHING_THAT_READS_NO_PATTERN is a list of four rather than a floor, so a twelfth way has to be sorted deliberately."
  - "No accelerator added or changed, so docs/KEYBOARD_SHORTCUTS.md is unchanged."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 10100
  tasks: 3
  commits: 7
---

# Phase 2 Plan 4: The filter dialog offers what the engine can answer Summary

**It works.** The Add and Edit Filter Rule window now offers all eleven fields
and all eleven ways of matching, in words a person would say rather than the
names they are stored under, and a way of matching that has nothing to compare
against no longer shows a Pattern box asking for something to compare.

The full suite is green: 5,884 library tests and every integration target.

## What works, and how that was checked

**Eleven fields and eleven ways of matching, in words.** Twenty-two
stored-and-spoken pairs sit beside the constants they describe in
`src/application/filters.rs`, with conversion both ways. Six named tests hold
them to it: every name has words, nothing else does, the words round-trip back
to the name, no two names sound alike, no words contain an underscore, and
exactly four ways of matching read no pattern.

Five fields could not be asked about at all before: the message text, the
formatted message text, whether a message has been read, whether it is flagged,
whether it is deleted. Five ways of matching were missing: "is not", "is
empty", "is not empty", "is yes", "is no".

**Both lists are built from the engine's constants.** `grep -n '"subject",
"from", "to", "cc"'` and `grep -n '"contains",'` over
`src/presentation/wx_managers.rs` both return nothing. The only remaining
`"body_plain"` under `src/presentation/` is a test fixture in `wx_app.rs`
building a saved search, not an offered list.

**A twelfth field reaches the dialog without the dialog being edited.**
Demonstrated by adding `bcc` to both constants and reading back what the built
dialog really offered:

```
["Bcc", "Subject", "From", "To", "Cc", "Date sent", "Message identifier",
 "Message text", "Formatted message text", "Read", "Flagged", "Deleted"]
```

Twelve entries, `wx_managers.rs` untouched. Removing it gave eleven again. The
list was printed rather than inferred from the check passing, because the check
compares the dialog against that same constant and would have passed either
way.

**A bug found by the new test, not looked for.** Opening a stored rule selected
the Choice by the stored name. For the five fields the list could not offer,
that selected nothing, so opening such a rule and pressing OK rewrote its field
to the empty string and the rule silently stopped matching anything. Fixed with
the vocabulary, and covered.

**The manager dialogs have control-name coverage for the first time.**
`tests/manager_dialog_labels.rs` builds the real dialogs, never calls
`show_modal`, and checks that the three Choice controls carry an accessible
object and the two check boxes carry a label on the control itself.

It found nothing wrong. Both check boxes were already correct and all three
Choice controls were already named, which `02-PATTERNS.md` predicted for the
check boxes. So there is no exception list. Because a guard that is green on
arrival proves nothing, both checks were taken red by hand:

| Break | What reddened |
|---|---|
| `set_accessible_name` removed from the field Choice | "the Match Field list: carries no accessible object" |
| the Case Sensitive label blanked and an accessible name set instead | "the Case Sensitive tick: carries no label on the control" |

The second is the exact both-channels bug: correct under NVDA, an unnamed check
box under Narrator. Windows does not attach an accessible object of its own,
which the first break also measured.

**Guard record.** One, `"the filter dialog offers what the engine answers
rather than a list of its own"`, running against `manager_dialog_labels`
because it builds windows. Taken red by hand: reintroducing a hardcoded list
reddens exactly one test, which is what the record says. `guards/guards.toml`
holds 540 records and the sweep header now reads 192 + 348.

**Not machine-verified beyond that.** `scripts/guards.sh` was not run, as
instructed. The new record was measured by hand and nothing else was
re-measured, so any existing record this branch made stale is unfound.

## Wrong premises in the plan

**1. A library test can build a real dialog, and the budget is one per
process.** Task 2 put the guard in `wx_managers.rs`'s own test module. Three
doc comments in the tree contradict that, one saying "Nothing in this crate
builds a live wxWidgets window inside `cargo test`". Both were measured rather
than believed.

A library test that builds the filter dialog passes, and the whole 5,876-test
library run stayed green with it. A second test calling `wxdragon::main` in the
same process prints `assert "!argc && !argv" failed in Initialize():
initializing twice?` and hangs until killed.

So Task 2's criterion, "`cargo test --lib presentation::wx_managers` passes and
includes all six behaviours as named tests", is **not satisfiable** for the
five behaviours that need a window: six test functions would need six calls.
The tree's absolute wording is also wrong; the real constraint is a budget of
one.

**2. Task 2's cited reason for that location is about a different kind of
test.** It cites `config.rs:1715-1719`, which is right and is about a check
that reads a file as text needing a helper not compiled into a release build.
Task 2 forbids a source-reading check two paragraphs earlier and requires a
built dialog. Every word of the reason is true and none of it applies.

Resolved by putting the built-dialog checks in `tests/manager_dialog_labels.rs`,
which Task 3 creates for exactly this and names "for what it covers rather than
for the dialog that prompted it". The record carries `suite =
"manager_dialog_labels"`, which `scripts/guards.py` already supports, so
`scripts/guards.sh` still runs it.

**3. The prescribed break for the guard cannot redden it.** Task 3 says to take
it red by dropping one entry from a constant. The guard reads that constant on
both sides, so both move together and it stays green. Measured: eleven became
twelve and the check passed throughout, correctly. The break that works is
reintroducing the hardcoded list, which is also the defect the guard defends.

**4. `grep -c '#\[test\]' tests/manager_dialog_labels.rs` returns 2, not 1.**
The second hit is the module doc comment explaining why the file has only one
test, which has to quote the marker. This is the fifth source-reading check in
this phase answered by a mention rather than a use.

The file really does hold one test. `tests/checkbox_labels.rs`, which the
criterion was modelled on, returns 2 for the same reason and in the same
sentence, so the criterion was never satisfiable by its own exemplar. Anchored
at column 0, `grep -c '^#\[test\]'` returns 1 for both files, and the harness
reports `running 1 test`. The prose was not reworded to make a bad check pass.

## Red/green

Seven commits, four of them a red/green pair.

| Commit | What |
|---|---|
| `328c857` | RED, six named tests for the vocabulary |
| `34f4e6e` | GREEN, the words and both conversions |
| `eb5793d` | RED, the dialog test, 32 findings against the tree |
| `f7ef903` | GREEN, both lists built from the constants |
| `4f56d8d` | RED, a pattern stored against a way that never reads it |
| `7efce54` | GREEN |
| `f92cef9` | the guard record, changelog and ledger |

Two things worth recording about the reds:

**The stubs were the widest wrong answers, not empty ones.** `the_words_for_a_field`
returned something for every input including unknown ones, so all six tests
reddened on content. An empty stub would have left the absence halves vacuously
green, which 02-03 found the hard way.

**One red only reddened one of three tests, so the other two were taken red by
hand.** The pattern-storing stub was the tree's own behaviour, which is the
honest stub, but the two tests around it agreed with it. Inverting the
condition after the fix reddens all three, so each discriminates.

The by-hand breaks in Task 1 were measured twice, since the first plausible
break is often the weak one: dropping a pair and misspelling a stored name both
redden the same two tests.

## Deviations from plan

**1. [Rule 3 - Blocking] The built-dialog checks moved to `tests/`.** Cause and
evidence above. Task 2's artifact row said `src/presentation/wx_managers.rs`
for the guard test.

**2. [Rule 2 - Missing critical] `show_filter_edit` converts words back to
names.** Not in the plan's behaviour list. Without it the dialog would offer
words and store them, so every rule saved would carry "Message text" in a field
the engine matches against `A_FIELD_A_RULE_MAY_NAME`, and every rule would stop
matching. Covered by the round-trip test.

**3. [Rule 2 - Missing critical] `the_pattern_to_store` is a separate pure
function.** The plan's behaviour "saving stores an empty pattern" happens in
`show_filter_edit`, which calls `show_modal` and cannot be tested. Split out so
it can be, following 02-03's pattern.

## Known stubs

None.

## Threat flags

None. `T-02-18` and `T-02-19` are the two mitigations this plan owed and both
are in place: `tests/manager_dialog_labels.rs` for the unnamed control, and the
guard comparing offered strings against the constants in both directions and by
count for the vocabulary the engine cannot answer. No dependency added;
`Cargo.toml` unchanged.

## What only a screen reader can confirm

Three entries added to `.planning/WINDOWS.md`. The reason the first one matters
more than it looks: **`wxdragon`'s `Accessible` has no name getter.** A test can
prove a name was attached and never that it is the right words, or that it is
not the empty string.

- Whether NVDA says "Match field", "Match type" and "Action" rather than
  unnamed combo boxes.
- Whether the disabled Pattern box is skipped cleanly in the tab order, and
  whether changing the Match Type near it moves focus or is announced.
- Whether "Read is yes", "Flagged is yes" and "matches a text pattern" are
  understood when heard rather than read.

Also worth stating because a clean run is easy to misread: a control with a
visible label beside it gets that label as its MSAA name from Windows even when
nothing set one. The absence of a failure here is not evidence that a name came
from this code.

## Deferred

Two items appended to the phase's `deferred-items.md`:

- **`tests/manager_dialog_labels.rs` does not run on the commits that could
  break it.** `scripts/check.sh` maps a changed `src/a/b.rs` to `--lib a::b::`.
  `tests/checkbox_labels.rs` has the same gap and is excluded from the
  per-commit code path deliberately, so this was recorded rather than reversed.
  The guard record does run it, through `scripts/guards.sh`.
- **A rule naming a field this build has never heard of still loses its field.**
  Opening selects nothing and OK stores the empty string. That was true for
  five real fields before this plan; now only a rule from a later version can
  reach it.

## Requirements

SEARCH-03's "reaching all eleven fields" is met for the filter path. The
one-vocabulary rule `Question::as_a_rule`'s comment states is now true in the
presentation layer as well as the application layer.

**SEARCH-03 itself is left open, against this plan's frontmatter.**
`gsd-tools query requirements.mark-complete SEARCH-03` ticked it and the tick
was reverted. This plan closes one of that requirement's criteria as a side
effect of fixing a vocabulary; the rule editor that delivers it is 02-05, and
the requirement's own evidence line, "Nothing joins the two into a folder that
updates itself", is still true. Ticking it here would have reported
partly-finished work as finished, in the one place a later reader would trust.

## Self-Check: PASSED

- `tests/manager_dialog_labels.rs` exists.
- All seven commits found in `git log`.
- `cargo test --all-targets --no-fail-fast`: 5,884 library tests and every
  integration target green.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` is 540, equal to 192 + 348.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- `git diff --stat main -- tests/checkbox_labels.rs` is empty.

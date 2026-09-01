---
phase: 02-search-that-says-what-it-covers
plan: 06
subsystem: search
tags: [saved-searches, atomicity, vocabulary, accessibility, guards]
status: complete
requires:
  - "create_saved_search, the one-transaction writer this replace is modelled on"
  - "filters::the_words_for_a_field and the_words_for_a_way_of_matching, the 02-04 vocabulary"
  - "filters::a_way_of_matching_compares_against_nothing, also 02-04"
  - "filters::A_FIELD_HOLDING_THE_MESSAGE_TEXT, the census the message-text disclosure reads"
  - "status_line::said_and_shown, the one call that both shows a sentence and says it"
  - "tests/manager_dialog_labels.rs, the one window-building suite 02-04 created"
provides:
  - "MessageCache::replace_saved_search, a whole search written back in one transaction"
  - "saved_searches::what_a_saved_search_cannot_find_with, the two disclosures kept apart"
  - "saved_searches::THE_FIELD_A_SAVED_SEARCH_NEVER_SEES"
  - "wx_managers::build_rule_edit_dialog and show_rule_edit, split build from show"
  - "wx_managers::RuleEditWidgets"
  - "wx_managers::the_words_for_every_field and the_words_for_every_way_of_matching, one builder both rule dialogs read"
  - "wx_managers::WHAT_A_NEW_CONDITION_ASKS_FIRST"
  - "wx_managers::what_a_condition_still_needs, the refusal decision as a pure function"
  - "five All-join tests through run_over, the first outside selects"
  - "one guard record, measured against the whole library"
affects:
  - "plan 02-07, which is what opens the dialog and calls the replace"
  - "the filter rule editor, which now reads the shared list builders rather than its own construction"
  - "guard record 540, whose break moved to the shared builder and was re-measured"
tech-stack:
  added: []
  patterns:
    - "ordering the writes in a transaction so the one reachable failure lands inside the window it protects"
    - "a RED commit whose stub is wrong reachable code rather than right unreachable code"
    - "two disclosures kept as two sentences with a test asserting they differ"
    - "a shared list builder read by two dialogs, with the guard record moved onto the builder"
decisions:
  - "replace_saved_search writes the name too. It takes a whole SavedSearch, and writing back all of it but one field is a caller's edit dropped in silence."
  - "The questions are written and then the row is stamped, not the other way round. updated_at is the claim that the search is now what it says, and that order puts the one reachable failure inside the transaction's window rather than outside it."
  - "The row is updated rather than deleted and re-inserted, because the questions cascade from it and anything holding the row's path would be left pointing at a new row."
  - "The two things a saved search misses are two sentences, not one, with a test asserting they are different. That is D-2-13 made into an assertion."
  - "The disclosure goes through said_and_shown, so it is spoken as well as shown. A line of text under a dialog raises no notification and is not somewhere anybody navigating by ear goes."
  - "No changelog entry. Nothing this plan builds is reachable, so there is nothing a user could be told about; the entry belongs to 02-07."
  - "No accelerator added or changed outside the new dialog's own mnemonics, so docs/KEYBOARD_SHORTCUTS.md is unchanged."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 25000
  tasks: 3
  commits: 8
---

# Phase 2 Plan 6: The writer and the dialog a rule editor needs Summary

**It works, and none of it is reachable yet.** That second half is the plan's
own design and is stated first because it is the thing a reader most needs to
know. A saved search's whole description can now be written back under its own
identifier, all of it or none of it. A dialog builds one condition from the
engine's eleven fields and eleven match types and says out loud what a saved
search cannot find with three of them. The All join is proved through
`run_over`. Nothing in the running program calls any of it: plan 02-07 is what
opens the dialog and writes through the replace.

The full suite is green: 5,928 library tests and every integration target, no
failures anywhere.

## What works, and how that was checked

**A whole question list written back, all of it or none of it.** One
transaction over the questions and the row, keeping the identifier, the account
and the date the search was made. Six named tests: the new list comes back
exactly and in order, the identifier and the account survive, the join and the
folder are written too, a replace over a search that is not there reports that
and creates nothing, a failure part-way leaves the old list intact, and an
empty question list is refused.

The questions table cascades from the search row
(`FOREIGN KEY(search_id) REFERENCES saved_searches(id) ON DELETE CASCADE`), so
the plan's question about which of the two shapes it has is answered: the row is
updated, never deleted and re-inserted. Deleting it would take the questions
with it and hand a wholly new row to anything holding the old one's path.

`git diff --stat main -- src/data/message_cache/mod.rs` is empty, so no schema
statement changed. `grep -c 'pub fn '` on the file went from 8 to 9.

**Atomicity proved by a real failure, not by reading the code for the word
"transaction".** The failure used is one a person can actually cause: a rule
editor lets somebody rename a search to a name another search in the account
already has, and the table refuses that. The writes are ordered so it lands
after the old questions have gone and the new ones are in, which is the window
the transaction exists to cover.

That order was chosen and it is worth saying why, because the opposite order
looks safer. Stamping the row first would make the constraint fire before
anything was destroyed, and would also mean nothing a user can cause ever falls
inside the protected window, so no test could tell a transaction from three
loose statements. **Moving the failure out of the window does not make the
window safe, it makes it unmeasurable.** There is a second, independent reason
for the same order: `updated_at` is the claim that this search is now what it
says it is, and a claim made before the thing it describes has been written is
briefly false.

Measured: replacing the transaction with loose statements on the connection
reddens exactly one test, the part-way failure, and nothing else.

**The two things a saved search misses are said as two sentences.** D-2-13 is
the reason and it is now an assertion rather than a paragraph. Mail that has
been thrown away is never gathered, because `scan_query` excludes it by design,
so a condition about it finds nothing new and rules nothing out. Message text
that was evicted is a different fact: `evict_bodies_over` deliberately leaves
the search index alone, so the search box still finds that message by a word
only in its text while a saved search no longer can. Four tests hold the pair
apart, including one that asserts the two sentences are not the same sentence.

The message-text case reads `A_FIELD_HOLDING_THE_MESSAGE_TEXT` through
`a_rule_reads_the_message_text` rather than naming the two fields, so a third
such field arrives with the sentence attached. That census gained a reader, not
a member, and no guard record reads it, so nothing was made stale by it.

**A dialog for one condition, from the engine's vocabulary.** Both lists come
from the same constants through one builder the filter editor now reads too, so
there is one vocabulary and not a second copy. The condition editor was checked
against the filter editor rather than against a list written out in the test,
which makes the pair transitive: the filter editor's lists are already pinned
against the constants in both directions and by count.

Seven checks in the window suite: both lists match the filter editor's and are
non-empty, a new condition opens with both lists answered and the Pattern box
empty, a stored condition opens with all four of its parts selected, the two
lists carry accessible objects, the Case Sensitive tick carries its label, and
the disclosure appears for the two fields that have one and not for a field that
has not.

**The refusal and the default are pure functions, so the library can hold
them.** `what_a_condition_still_needs` is tested over all eleven ways of
matching in both directions, which is what `CLAUDE.md` asks for a function that
switches on a string. `WHAT_A_NEW_CONDITION_ASKS_FIRST` is checked to name a
field and a way of matching that really exist, because a default naming
something the lists do not offer selects nothing and OK on that stores the
empty string.

**The All join, proved where it will actually run.** All five assertions were
green on arrival. Nothing was wrong and nothing was fixed, which is the
measurement this task existed to take rather than a disappointment.

| Assertion, through `run_over` | Already true |
|---|---|
| All with two conditions takes only messages answering both | yes |
| Any takes messages answering either | yes |
| All naming a field this build cannot read says it could not run | yes |
| All with no questions takes nothing | yes |
| All about message text answers about mail whose text is not here | yes |

What was missing was coverage, not behaviour. The matcher has honoured both
arms since it was written and both were exercised, but only through `selects`,
never through `run_over`, which is the one way to run a search and the step
where `what_it_cannot_read` is asked. `Join::All` had never been written outside
a unit test: the one production site writes `WHAT_A_TYPED_SEARCH_JOINS_WITH`,
which is `Join::Any`. D-2-01 makes the All arm reachable for the first time.

`grep -n '.selects('` over `src/` outside `application/saved_searches.rs`
returns nothing, so no new call from outside the module.

## Reached by a person, and what is not

Guardrail 1 says a feature is done when a non-test path reaches it, so this was
traced rather than assumed.

**Reached.** `run_over` is called at `wx_app.rs:6540`, inside
`run_a_saved_search`, which has two live callers in the window's event wiring:
opening a saved-search row in the folder tree (`wx_app.rs:2475`) and the
Refresh command while a saved search is open (`wx_app.rs:3427`). Neither is a
test. So the five All-join tests cover a function a person really runs, which is
the point of writing them through `run_over` rather than through `selects`.

The two shared list builders are also reached: `build_filter_edit_dialog` reads
both, `show_filter_edit` calls it, `show_filter_manager_dialog` calls that,
`managers::manage_filters` calls that, and `wx_app.rs:4671` binds it to
`ID_FILTER_MGR` on the menu.

**Not reached, by design.** `replace_saved_search`, `build_rule_edit_dialog`,
`show_rule_edit` and `what_a_saved_search_cannot_find_with` have no caller
outside tests. Plan 02-07 opens the dialog and writes through the replace. Both
are recorded in `.planning/WINDOWS.md` as stubs rather than left for somebody to
find.

## Wrong premises in the plan

**1. The changelog criterion cannot be met honestly, because nothing this plan
builds is reachable.** Task 3 asks for an `[Unreleased]` entry "that describes
only what is reachable". All three tasks produce code with no production caller,
by the plan's own design, so the set of reachable things to describe is empty.
The criterion is satisfiable only by writing an entry about a feature nobody can
open, which `CLAUDE.md` names as the failure the docs-accuracy pass closed and
which the plan's own action paragraph also names.

No entry was written. The plan's conditional, "if 02-07 lands in the same
release", is the right shape and it points at 02-07: one entry there describing
what the two make possible together, rather than a half-entry here about a
dialog nobody can reach. `Cargo.toml` still reads 0.46.0 and no version was
bumped.

**2. `cargo test --lib presentation::wx_managers` cannot hold the first six
behaviours as named tests.** The same wrong premise 02-04 found and reported,
recurring in this plan. Five of the six behaviours need a built window, a
process may call `wxdragon::main` once, and the library test binary has already
spent that budget elsewhere. Resolved 02-04's way: the pure halves are library
tests and the built-dialog checks went into `tests/manager_dialog_labels.rs`,
which exists for exactly this and holds one `#[test]`.

**3. `grep -c '#\[test\]' tests/manager_dialog_labels.rs` returns 2, not 1.**
Also recurring from 02-04, and unchanged by this plan: the second hit is the
module doc comment explaining why the file has one test, which has to quote the
marker. Anchored at column 0, `grep -c '^#\[test\]'` returns 1, and the harness
reports `running 1 test`. This is the seventh source-reading check in this phase
answered by a mention rather than a use.

**4. Extracting the shared list builder moved a guard record's `before`
snippet, and the plan does not mention it.** Task 2 asks for the construction to
be extracted into one function both dialogs call. Guard record 540 names the
four lines that construction used to be, and `before` has to appear exactly once
in the tree, so the extraction would have failed
`test_every_guard_record_still_names_one_place_in_the_tree` at commit time. The
record was moved onto the shared builder and re-measured in the same commit.

**5. Task 1's behaviour list does not say whether the replace writes the
name.** It names the join, the folder and the questions. A `SavedSearch` carries
a name as well, so following the list literally would mean a method that takes a
whole search and silently drops one field of it, which is the shape the rest of
that task exists to prevent. The name is written. `rename_saved_search` stays
for the tree's own Rename command, which has no question list to hand over, and
the two agree on the column rather than disagreeing about it.

## Red, green, and every break measured

Eight commits, three of them a red/green pair, two of them test-only additions
against code that already worked.

| Commit | What |
|---|---|
| `e565c33` | RED, six over the replace |
| `8ac5408` | GREEN, one transaction over the row and its questions |
| `0955bb4` | RED, four over the two disclosures |
| `d378bce` | GREEN, two sentences rather than one |
| `80d49d9` | RED, three over the refusal and the default |
| `f7bc218` | GREEN, and the shared builder |
| `29fe79b` | the condition editor's window checks |
| `a3cd8d8` | the All join through `run_over`, and the guard record |

**Every stub was the widest wrong answer, not an empty one.** The replace
reported success and changed nothing, which reddened five of six on content. The
disclosure stub returned one collapsed sentence for every field, which is the
exact defect D-2-13 names, and reddened all four including the one asserting the
two are different. An empty return would have left the absence half of that test
vacuously green.

**The dialog's stub could not be absent code, so it was wrong code.** A private
function called only from tests is dead in the plain library build and the gate
builds both, so a helper with no caller cannot be committed on its own. The RED
commit therefore contains the whole dialog with two faults the tests name: a
refusal that ignored which comparison had been chosen, and a default naming a
field nothing is called. That makes the RED commit larger than its GREEN pair,
which is the opposite of the usual shape and is not a smell here.

**Twelve breaks measured by hand, none reasoned about.**

| Break | What reddened |
|---|---|
| a fresh identifier written into the replace's UPDATE | 3, including the identifier test that was green on arrival |
| the transaction replaced by loose statements on the connection | 1, the part-way failure |
| the refusal ignoring whether the pattern is compared at all | 1, the typed-in-pattern test |
| the shared field builder replaced by a hardcoded list of four | 14 findings in the window suite, nothing in the 5,923-test library |
| the disclosure not called at build time | 4, both fields that have one and neither that has not |
| a stored condition selected by the stored name rather than the words | 6, both lists empty and the disclosure gone with them |
| the accessible name dropped from the condition editor's field list | 1 |
| the two join arms swapped | 7, three of them new |
| `run_over` carrying on past `what_it_cannot_read` | 4, one of them new |
| the empty question list falling through to the matcher | 2, against the whole library |
| record 541's break, re-measured for staleness | 3, unchanged |

**Three tests were green on arrival and each took its red by hand.** The
identifier test, because a replace that does nothing changes no identifier
either. The typed-in-pattern test, because a stub that only refuses an empty
pattern never refuses a full one. And all five All-join tests, for the reason
above.

**One assertion has no break at all, and that is worth saying rather than
hiding.** The half of the identifier test asserting the account does not move
holds because the UPDATE statement never names an account column. No break short
of adding one can redden it. It is a structural guarantee written down as a
test, not a measured guard, and reading it as the latter would be reading more
than it says.

**The stored-name break is the same bug 02-04 found in the filter editor,
arriving in the new dialog by the same route.** Selecting a `Choice` by the
stored name selects nothing, so both lists come back empty and pressing OK
rewrites the condition. It also took the disclosure with it, because the
sentence is read from whichever field is selected. That cascade is the reason
the disclosure check and the selection check are not redundant with each other.

## Guard records

One added and two re-measured. `guards/guards.toml` holds 542 records and the
sweep header now reads 192 + 350.

### Added: "a saved search that asks nothing takes nothing, whichever way it joins"

Running against the library. The break is the fall-through somebody would write
if they took the guard in `selects` for a needless check. Measured against the
whole 5,928-test library: exactly two, the `selects`-level test that was already
there and the `run_over`-level test added here.

**`Join::Any` is why the fall-through reads as safe.** Over an empty list `any`
is false, so the Any arm goes on taking nothing and only the All arm floods.
Half the behaviour looks right, which is precisely the shape that survives
review, and it was unreachable until D-2-01 let somebody delete every condition
in an editor.

### Re-measured: "the filter dialog offers what the engine answers rather than a list of its own"

Its `before` is now the shared builder rather than the filter dialog's own four
lines, because this plan extracted the construction both dialogs read. So the
break now takes both dialogs down rather than one.

**Still one test named, and that is not the red list shrinking.** Every check
that builds a window lives in the one `#[test]` that file is allowed, so a break
reaching two dialogs reddens the same single function. Measured again after the
move rather than carried over: 14 findings inside that one test, and nothing at
all in the library.

### Re-measured: "a saved search keeps the question set the In box was narrowed to"

Re-measured because this plan adds tests in the module it names, which is the
staleness `CLAUDE.md` says never announces itself. Still exactly three,
unchanged. That took a full library run, which is the only thing that answers
the question honestly.

**Not machine-verified beyond that.** `scripts/guards.sh` was not run, as
instructed. The three records above were measured by hand against the whole
library and nothing else was re-measured, so any record this branch made stale
is unfound.

## A census that was already slack, noted rather than fixed

`test_every_answer_the_manager_windows_give_is_said_out_loud` requires at least
ten `said_and_shown(` calls in `wx_managers.rs`. The file held 19 before this
plan and holds 20 after. `CLAUDE.md` warns that adding a member to anything a
census counts weakens the guards that read it, so it was checked: the floor was
already slack by nine, so this addition does not change whether that check is
load-bearing. It was not load-bearing before and it is not now. Recorded rather
than silently stepped over, and not raised here, because moving a floor is a
decision about that guard rather than a side effect of this plan.

## Deviations from plan

**1. [Rule 2 - Missing critical] The replace writes the name.** Reasoning
above. The plan's behaviour list names the join, the folder and the questions
only.

**2. [Rule 2 - Missing critical] The disclosure is spoken as well as shown.**
The plan says to put the sentence on screen beside the control rather than in a
tooltip, which is right and is not sufficient: `status_line.rs`'s own module doc
says a line of text under a window raises no notification and is not somewhere
anybody navigating by ear goes, so a sentence only written there is an answer
nobody gets. It goes through `said_and_shown`, the one call that does both. That
is why `build_rule_edit_dialog` takes an `Arc<Accessibility>`, which the plan's
signature does not mention.

**3. [Rule 2 - Missing critical] The message-text disclosure is not in the
plan's behaviour list.** Task 2 names only the field for thrown-away mail.
D-2-13 says there are two coverages and that collapsing them is the defect, so a
dialog disclosing one of the two would be that defect with better manners. Both
are said, kept apart, and a test asserts they differ.

**4. [Rule 3 - Blocking] Guard record 540 moved.** Cause above; the extraction
the plan asks for is what moved it.

**5. [Rule 3 - Blocking] The built-dialog checks are in `tests/`, not the
library.** Cause above, and the same resolution 02-04 reached.

**6. No changelog entry.** Cause above. This is a deviation from an acceptance
criterion and it is deliberate.

## Known stubs

Two, both by the plan's design, both recorded in `.planning/WINDOWS.md`.

- `MessageCache::replace_saved_search` has no caller outside its tests.
- `build_rule_edit_dialog` and `show_rule_edit` are built and tested and nothing
  in the running program opens them.

Plan 02-07 is what reaches both. Neither is a stub in the sense guardrail 3
forbids: nothing looks done and does nothing on screen, because nothing is on
screen. What they are is finished work waiting for its caller, and the risk is
that a later reader takes their existence for the feature being present. That is
what this section and the ledger entries are for.

## Threat flags

None new. The three mitigations this plan owed are in place.

- **T-02-24**, an empty condition list joined by All returning the whole
  mailbox: refused at the store, refused at the dialog through the same guard
  the pattern refusal uses, and proved through `run_over` with a guard record
  whose break is the fall-through somebody would write.
- **T-02-25**, a partial write leaving a search with half a question list: one
  transaction, proved by forcing a reachable failure inside the window it
  protects.
- **T-02-26**, a condition that can only ever find nothing offered in silence:
  the field is offered per D-2-01 and carries a sentence, shown and said.

T-02-27 and T-02-28 were accepted rather than mitigated and nothing here changes
that: no regex handling was touched, and the one query this plan adds binds
every parameter. No dependency added; `Cargo.toml` unchanged and still reads
0.46.0.

## What only a screen reader or a live account can confirm

Five entries added to `.planning/WINDOWS.md`, taking it to 24. Three need a
screen reader and two are the unreached-code notes above.

- Whether the disclosure is heard when the field list changes, and whether a
  sentence that long is useful beside a combo box or in the way of somebody
  arrowing through eleven fields. It is the one judgement here most likely to be
  wrong by ear and it is a small change if it is.
- Whether NVDA says "Match field" and "Match type" for the new dialog's two
  lists. `wxdragon`'s `Accessible` has no name getter, so a test can prove an
  object was attached and never that it is the right words or that it is not the
  empty string.
- Whether the empty-pattern refusal is heard, and whether focus landing back on
  the Pattern box is where somebody expects it.

Worth repeating because a clean automated run is easy to misread: a control with
a visible label beside it gets that label as its MSAA name from Windows even
when nothing set one, so the absence of a failure is not evidence that a name
came from this code.

## Deferred

Nothing new. The two items in `deferred-items.md` are untouched and neither is
about this code. The first of them does apply to this plan's work, though, and
is worth naming rather than left implicit: `tests/manager_dialog_labels.rs` does
not run on the commits that could break it, because `scripts/check.sh` maps a
changed `src/a/b.rs` to `--lib a::b::`. Every window check this plan added is in
that file. They run at the merge gate and through the guard record, not per
commit.

## Requirements

**SEARCH-03 is left open, as 02-04 left it.** This plan builds the writer and
the dialog its second door needs and reaches neither from the running program,
so the requirement's own evidence line, "Nothing joins the two into a folder
that updates itself", is still true word for word. 02-07 is what closes it.
Ticking it here would report unreachable work as finished in the one place a
later reader would trust.

## Self-Check: PASSED

- All eight commits found in `git log`.
- `cargo test --all-targets --no-fail-fast`: 5,928 library tests and every
  integration target green, no failures.
- `git diff --stat main -- src/data/message_cache/mod.rs` is empty.
- `grep -c 'pub fn ' src/data/message_cache/saved_searches.rs` is 9, one more
  than `main`'s 8.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` is 542, equal to 192 + 350, and
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  agrees.
- No `show_modal` call inside `build_rule_edit_dialog`.
- `cargo test --test manager_dialog_labels` reports `running 1 test`.
- `.selects(` appears nowhere outside `src/application/saved_searches.rs`.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- Every symbol this plan added is either reached from a menu or a tree row
  (`the_words_for_every_field` and `the_words_for_every_way_of_matching` through
  `ID_FILTER_MGR` at `wx_app.rs:4671`; `run_over` through `run_a_saved_search`
  at `wx_app.rs:2475` and `:3427`) or listed under Known stubs above.

---
phase: 02-search-that-says-what-it-covers
plan: 05
subsystem: search
tags: [saved-searches, scope, vocabulary, guards, multi-account]
status: complete
requires:
  - "TheSearchThatWasRun's producer: show_search_dialog's second return value, which existed and was dropped"
  - "filters::the_words_for_a_field and the_words_for_a_way_of_matching, the 02-04 vocabulary"
  - "filters::a_way_of_matching_compares_against_nothing, also 02-04"
  - "create_saved_search, which already wrote both tables in one transaction"
  - "WhichRow::opens, from 01-05, which answers with the folder a row is about"
provides:
  - "saved_searches::TheSearchThatWasRun, the words and the In box's answer as one value"
  - "saved_searches::TheFolderSearched, a folder path with the account it belongs to"
  - "saved_searches::WhatASavedSearchWillAsk, questions, join and folder from one producer"
  - "saved_searches::what_a_typed_search_asks, taking the whole of what was asked"
  - "saved_searches::a_search_in_words, one sentence for any question set"
  - "saved_searches::SavedSearch::in_words, the same sentence from a stored search"
  - "saved_searches::what_a_search_says_as_it_opens, the scope and the coverage in one line"
  - "saved_searches::a_search_may_be_saved_under and THAT_FOLDER_IS_ANOTHER_ACCOUNTS"
  - "wx_app::keep_the_search_that_ran, the one place the search that ran is kept"
  - "one guard record, measured by hand three times"
affects:
  - "every search saved from the search box: Subject Only, From Only and Current Folder are kept"
  - "the Save This Search window's sentence, which now reads the questions out"
  - "opening a saved search, which now says what it asks before the mail is gathered"
  - "plans 02-06 and 02-07: a_search_in_words already serves sets the In box cannot name"
tech-stack:
  added: []
  patterns:
    - "one value carrying every half of a thing that must not come apart, built by one constructor"
    - "a refusal rather than a silent narrowing or widening when two halves disagree"
    - "a test fixture written out by hand where the claim is that two writers agree"
    - "a guard record whose break is dropping an argument, because that is the wrong fix on offer"
key-files:
  created: []
  modified:
    - src/application/saved_searches.rs
    - src/presentation/wx_app.rs
    - src/data/message_cache/saved_searches.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "TheSearchThatWasRun lives in application::saved_searches, not wx_app as the plan's artifact table said, because what_a_typed_search_asks has to read it and application cannot depend on presentation."
  - "what_a_typed_search_asks returns questions, join and folder together rather than questions alone, so no call site decides the folder separately. That is D-2-14 made structural instead of a rule to remember."
  - "The folder carries the account it belongs to. Set Active changes which account a search is saved under and leaves the tree's cursor where it was, and a folder path is not unique across accounts."
  - "That mismatch is refused rather than saved without the folder, because dropping it widens the search silently, which is the defect this plan exists to fix arriving by another door."
  - "The sentence names fields as the filter editor names them, Subject and From rather than the sender and the recipients, because the words come from 02-04's pairs and a second list here is the vocabulary rule this module exists to keep."
  - "No accelerator added or changed, so docs/KEYBOARD_SHORTCUTS.md is unchanged."
  - "Opening a saved search says its scope on the status line that already carries the coverage figure, in one update rather than two, because a second update replaces the first before anybody has read it."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 27000
  tasks: 3
  commits: 11
---

# Phase 2 Plan 5: A saved search keeps the whole scope it was saved with Summary

**It works.** Choosing Subject Only, From Only or Current Folder in the search
box and then saving the search now keeps that choice. Subject Only saves one
question about the subject, From Only one about the sender, and Current Folder
saves the folder the search actually ran in. Searches saved before this change
are byte-for-byte the same rows and behave exactly as they did.

Opening a saved search now also says what it asks, which the plan's tasks left
unwired and which SEARCH-01 asks for. That is the largest thing here that the
plan did not name, and it is written up below.

The full suite is green: 5,910 library tests and every integration target, no
failures anywhere.

## What works, and how that was checked

**The In box's answer survives the dialog closing.** `show_search_dialog`
returned it and the state kept only the typed words; three lines later the
answer was dropped on the floor. It is now kept beside the words in one value,
`TheSearchThatWasRun`, written at exactly one site. Five tests hold that: the
answer is kept, a second search replaces both halves together, a search in
another module leaves what mail asked alone, nothing is kept before any search
has run, and the folder is kept only when the In box named one.

**A narrower question set, with no schema change.**
`git diff --stat main -- src/data/message_cache/mod.rs` is empty.
`saved_search_questions` has always stored an arbitrary set with positions, so
a set of one is a shape those tables already hold.

**The backward-compatibility case disappears rather than being handled.** A
search saved by an older version holds three questions and no folder. One
saved now with the In box left alone holds exactly the same three, proved
against a fixture written out by hand rather than taken from the writer, and
it still runs: `what_it_cannot_read` is `None` and it selects the same
messages. There is no absent value anywhere for a reader to interpret, which
is what makes SEARCH-01's fourth criterion vanish instead of needing an
answer.

**Both halves of the scope come out of one call.** `what_a_typed_search_asks`
returns the questions, the join and the folder together, and
`save_this_search` spreads that one value into the one `SavedSearch` literal,
which goes into `create_saved_search`, which writes both tables in one
transaction. There is nowhere for a second answer about the folder to appear.
The old comment above `folder: None`, which argued that narrowing would be
narrowing on something nobody wrote down, went with the code rather than being
left contradicting it.

**The folder stored is the one the search ran in.** Save This Search is a
separate command pressed whenever somebody gets round to it, so working the
folder out then would attach the search to whatever they had since arrowed to.
A test moves the cursor after the search and asserts the kept folder does not
follow.

**One sentence for any question set.** The Save This Search window used to say
one fixed sentence whatever had been chosen. It now reads out a clause per
question, in the words 02-04 put beside the engine's constants, and names the
folder when there is one. Seven tests: the three-question set, the one-question
set, a two-field set the In box has no name for, the folder and the account
cases, a way of matching that compares against nothing, no underscore over
every field crossed with every way of matching, and a question this build
cannot read saying so rather than guessing.

**Round trip through both tables.** A folder-narrowed search and a
subject-only one are written and read back, and the assertion is on what came
back in its own words. Written the obvious way, comparing the read against the
value that was written, it passed against a stub that dropped half the scope,
because the writer was on both sides of it. That was caught and fixed before
the RED commit.

**Opening a saved search says what it asks.** It said "Invoices, 3 messages"
and nothing about the scope, so a short list read as an empty mailbox rather
than as a narrow search, which is SEARCH-01's third criterion word for word.
The sentence comes from the same builder the naming window uses, through
`SavedSearch::in_words`, so a search cannot describe itself one way while it is
being saved and another way when it is opened. It travels on the status line
that already carries the coverage figure, in one update rather than two,
because a second update replaces the first before anybody has read it.

## Two things found that the plan did not ask about

**A saved search could be stored with another account's folder.** Set Active
in the account manager changes `active_account_id` and leaves
`selected_folder` where it was, and neither clears the other. A folder path is
not unique across accounts: "INBOX" names one in every account. So running a
search in account A's inbox, setting account B active, and saving would have
written B's search with A's path, and running it would have listed B's inbox
under the name given to A's.

That was harmless while `folder` was hardcoded to `None`. This plan makes it
reachable, so it is this plan's to close, and it is exactly the threat register's
T-02-22 by a route the register did not name.

The folder now carries the account it belongs to, in one value, and
`save_this_search` asks before anything is written. **The mismatch is refused
rather than saved without the folder**, because dropping it widens the search
from one folder to a whole account with nothing said, which is the failure
this plan exists to fix arriving by another door.

Both directions were taken red by hand, because no single stub can redden
both: allowing everything, which is what the tree did, reddens the refusal
test, and refusing everything reddens the two about allowing.

**The plan does not wire D-2-04 into opening a search, and SEARCH-01 asks for
it.** Task 3's `read_first` names the sentence's caller in the naming window
and the `Asking` struct's `note` field, and nothing else. Follow the plan and
SEARCH-01's third criterion, "opening a saved search shows the scope it holds,
so a narrow result list is legible as a narrow scope rather than as an empty
mailbox", is not met, while the plan's frontmatter claims the requirement.

Built rather than reported, because D-2-04 is a locked decision that says
"opening a saved search says what it asks" and only the placement was open.
The placement chosen is the status line that already carries the coverage
figure, three lines away in the same function, for the reason that line
already gives: said before the gather, because a figure said after the results
is a footnote on an answer somebody has already acted on. One update rather
than two, because the second replaces the first on the status bar and a scope
said and then wiped is a scope not said.

That placement is the one judgement here that is easy to overrule, and it is a
small revert if it is wrong.

## Wrong premises in the plan

**1. The artifact table puts the state value in `wx_app.rs`, and it cannot
live there.** Task 2 needs `what_a_typed_search_asks` to read the scope, and
`application` cannot depend on `presentation`. `TheSearchThatWasRun` is in
`application::saved_searches`, which is also where the module doc already says
"where to look is a real part of the question".

**2. The plan splits the folder half from the question half, and the trap it
warns about is in its own task list.** Task 2's artifact row says
"`what_a_typed_search_asks` taking a scope" for the questions and "the folder
written in `save_this_search`" as a separate artifact. Followed literally that
is the folder written in one place and the question set in another, which is
what D-2-14 forbids. Resolved by having one function return both, so the
literal in `save_this_search` cannot be handed two different answers.

**3. The plan's own acceptance criterion for the all-folders test cannot
redden.** It asks for a test that "produces exactly the three fields the
constant names" and for changing the constant to make it fail. Written that
way the assertion reads `WHAT_A_TYPED_SEARCH_LOOKS_AT` on both sides, so the
constant changing moves both and nothing goes red. Measured: it stayed green.
The assertion now names the three literally, which is also the honest form,
because the claim is about the three a search saved before this plan holds.

**4. `bash scripts/guards.sh` is in Task 3's verify block and in the plan's
verification section.** `CLAUDE.md` took that off the critical path on
2026-08-31 and says the executor does not run guards. It was not run, as
instructed.

**5. The plan's success criteria claim SEARCH-01, and its tasks reach three of
its four derived criteria.** The third, about opening a saved search, is not in
any task. Written up above.

## Red/green

Eleven commits, five of them a red/green pair plus one test-only addition.

| Commit | What |
|---|---|
| `c8c37f1` | RED, five tests over the one write path |
| `f1c4a77` | GREEN, the In box's answer kept beside the words |
| `782150b` | RED, six over the scope and one round trip |
| `dfb557a` | GREEN, the narrower question set and the folder |
| `1023b74` | RED, seven over the sentence |
| `5dd4915` | GREEN, one sentence for any question set |
| `62407f8` | the old-shape fixture the plan's verification asks for |
| `f0e3ff7` | RED, the folder that belongs to another account |
| `bc64686` | GREEN, and the changelog |
| `fb0d7e7` | RED, saying what a search asks as it opens, and a re-measured record |
| `4f81ed3` | GREEN, and the changelog |

**Every stub was the widest wrong answer, not an empty one.** The sentence stub
read the stored names out and always quoted the pattern, so all seven reddened
on content; an empty sentence would have left "no underscore" and "no empty
quotes" vacuously green. The scope stub was the tree's own behaviour, three
questions and no folder whatever was asked.

**Where one stub could not redden both directions, the other half was taken red
by hand.** That happened three times: the default-value test in task 1, the
all-folders test in task 2, and both allow-direction tests for the account
check.

**Eight by-hand breaks, all measured rather than reasoned about.**

| Break | What reddened |
|---|---|
| the state field initialised with a search nobody ran | the default test |
| holding the old scope over new words, the two-field shape | the replaced-together test, showing "receipt" beside SubjectOnly |
| `WHAT_A_TYPED_SEARCH_LOOKS_AT`'s third entry changed to `cc` | three, then four after the sentence landed |
| the folder path dropped at the write site | both folder tests in wx_app |
| a scope-name path with a fallback for unnamed sets | three: the two-field set, the value-less match type, the unreadable question |
| the scope argument ignored, which is the guard record's break | three, twice re-measured, unchanged |
| the account check always allowing, then always refusing | one, then two |
| the coverage answer bound to `_` and `None` passed in its place | one, and nothing at all before the check was strengthened |

**A record went stale inside this session, and the measurement proves it.**
Changing the constant reddened three tests when measured during task 2 and four
after task 3 landed, because the sentence now names the parts a search asks
about. Nothing in the workflow re-asks; the second measurement was taken because
`CLAUDE.md` says any change that adds tests near a rule re-measures it.

**The scope-name break is worth keeping.** It passes all three cases the In box
can name and fails only on the sets it cannot, which is precisely why such a
path reads as correct in review. D-2-04's argument is not aesthetic.

## Guard records

One added and one re-measured. `guards/guards.toml` holds 541 records and the
sweep header now reads 192 + 349.

### Added: "a saved search keeps the question set the In box was narrowed to"

Running against the library.

Its break is dropping the scope argument, which is the wrong fix somebody would
reach for when a compiler calls it unused. Shrinking the constant instead
reddens nothing useful for this rule, because the tests about the unnarrowed
answer read that constant too. Measured against the whole library four times,
each time after adding tests beside the rule, and each time exactly three:
the two narrowed answers and the round trip through both tables.

**Neither sentence test reddens under that break, correctly.** The sentence is
built from whatever questions it is handed, so it says the truth about a set
the writer got wrong. Two mechanisms, and the gap between them is real: nothing
catches a writer that builds the wrong set and a sentence that faithfully
describes it. The round trip is in the red list because it is the only test
that crosses that gap.

### Re-measured: "what a saved search covers is said, not merely worked out"

Folding the scope sentence into the same status update moved the code this
record names, and `test_every_guard_record_still_names_one_place_in_the_tree`
refused the commit. That is the check doing its job, and it found more than a
moved line.

**The record's break had stopped working, and the check had a hole.** The old
code put the `say` inside `if let Some(covers)`, so throwing the answer away
deleted the update with it and the check saw an update go missing. With the
answer passed as an argument, handing `None` in its place leaves the update
exactly where it was. Measured: that break was invisible, green throughout.

Closed two ways in the same commit. The check now also requires the answer to
be bound to a name rather than to `_`, and with it bound, handing `None`
instead leaves an unused variable that `-D warnings` refuses. Measured both:
`let covers = ...` with `None` passed does not build, and the `let _ =` form
reddens exactly the one test the record names. The record now carries that
break and says why.

**Not machine-verified beyond that.** `scripts/guards.sh` was not run, as
instructed. Both records above were measured by hand against the whole library.
No other record was re-measured, so any this branch made stale is unfound.

## Deviations from plan

**1. [Rule 3 - Blocking] `TheSearchThatWasRun` lives in `application`, not
`presentation`.** The plan's artifact table says `wx_app.rs`. Layering forbids
it; cause above.

**2. [Rule 2 - Missing critical] `what_a_typed_search_asks` returns both halves
of the scope rather than the questions alone.** The plan describes them as two
artifacts written in two places, which is what D-2-14 forbids. It also carries
the join, so `save_this_search`'s literal takes all three from one value.

**3. [Rule 2 - Missing critical] The folder carries its account, and a
mismatch is refused.** Not in the plan. Reasoning above; it is T-02-22 by a
route the threat register did not name.

**4. [Rule 1 - Bug] The round-trip test compared the writer against itself.**
Written as the plan describes it, "writes a search and reads back both", it
passed against a stub that dropped half the scope. Rewritten to assert what
came back in its own words.

**5. [Rule 1 - Bug] The all-folders test read the constant on both sides.**
Cause above; the plan's own criterion asks for a break that could not work.

**5. [Rule 2 - Missing critical] Opening a saved search says what it asks.**
Not in any task, and it is SEARCH-01's third criterion. Reasoning and the
placement chosen are above.

**6. [Rule 1 - Bug] The check on the coverage sentence had a hole this change
opened, and it is closed in the same commit.** Cause and both measurements
above.

**7. `a_typed_search_in_words` is gone rather than changed in place.** Its
replacement, `a_search_in_words`, takes a question set. The old name says
"typed", and the sentence now serves sets nothing typed.

## Known stubs

None.

## Threat flags

None new. The three mitigations this plan owed are in place.

- **T-02-20**, a saved search running wider than it was saved: the question set
  is written from the chosen scope, and the guard record's break is ignoring
  that argument.
- **T-02-21**, the two halves written by two paths: one producer, one value,
  one literal, one call, one transaction.
- **T-02-22**, a search attaching to the wrong folder: the path is taken when
  the search runs rather than when it is named, with a test that moves the
  cursor in between. The account half of the same threat, which the register
  did not name, is refused.

No dependency added; `Cargo.toml` unchanged and still reads 0.46.0.

## What only a screen reader or a live account can confirm

Three entries added to `.planning/WINDOWS.md`, taking it to 19 open.

- The Save This Search sentence is longer than the one it replaces for a
  three-question search. Whether reading a clause per question is clearer or
  merely longer when heard is unverified by ear.
- The refusal for a folder belonging to another account needs two accounts and
  Set Active to reach, so whether it is heard and understood is unverified.
- A folder-narrowed saved search has never run against a real account. Whether
  a real IMAP mailbox's stored path resolves through `get_folder`, rather than
  refusing with `THAT_FOLDER_IS_NOT_HERE`, is unverified against a live server.

## Deferred

Nothing new. The two items 02-04 left in `deferred-items.md` are untouched and
neither is about this code.

## Requirements

SEARCH-01 is met and ticked. All four of its derived criteria hold:

1. A search saved with Subject Only or From Only reruns with that restriction.
2. The folder half and the field half are written and read back by one path.
3. Opening a saved search shows the scope it holds. This is the one the plan
   left unwired and it is built here; without it the requirement would have
   been left open, as 02-04 left SEARCH-03.
4. A search saved by an older version reruns across all three fields. D-2-03
   satisfies this by removing the case rather than answering it: the reader's
   answer for a missing restriction and the writer's for an unrestricted
   search are the same rows, proved against a fixture written out by hand.

Roadmap criteria 1 and 2 are met.

## Self-Check: PASSED

- All eleven commits found in `git log`.
- `cargo test --all-targets --no-fail-fast`: 5,910 library tests and every
  integration target green, no failures.
- `grep -v '^\s*//' src/presentation/wx_app.rs | grep -c 'last_mail_search'` is 0.
- `grep -n 'mail_search_that_was_run = ' src/presentation/wx_app.rs` returns one
  line.
- `grep -n 'folder: None' src/presentation/wx_app.rs` no longer returns the line
  in `save_this_search`; the two left are test fixtures.
- `grep -c 'WHAT_A_TYPED_SEARCH_LOOKS_AT' src/application/saved_searches.rs` is
  3, and the constant is still of length 3.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` is 541, equal to 192 + 349.
- `git diff --stat main -- src/data/message_cache/mod.rs` is empty.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- Every symbol added has a production call site: `keep_the_search_that_ran` at
  `wx_app.rs:4602`, `the_folder_on_screen` at `:14218`,
  `what_a_typed_search_asks` at `:6726`, `a_search_may_be_saved_under` at
  `:6720`, `WhatASavedSearchWillAsk::in_words` at `:6727`,
  `what_a_search_says_as_it_opens` inside `run_a_saved_search` at `:6517`, and
  `save_this_search` is bound to `ID_SAVE_SEARCH` at `:4626`.
  `SavedSearch::in_words` is reached from `what_a_search_says_as_it_opens`.

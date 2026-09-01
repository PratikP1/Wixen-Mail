---
phase: 02-search-that-says-what-it-covers
verified: 2026-09-01T15:12:00Z
status: human_needed
score: 6/6 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 5/6
  previous_verified: 2026-09-01T13:14:51Z
  previous_tree: 0cc5e84
  this_tree: 93c59d7
  closed_by: 02-09-PLAN.md
  gaps_closed:
    - >-
      "A search that can reach message text says, before it runs, how many
      messages in the account have body text stored and how many do not." The
      search box now says it, with its own number, on the same status line and
      the same announcement topic as the match count. SEARCH-02 is delivered.
  gaps_remaining: []
  regressions: []
  carried_open:
    - >-
      The fetch offer is still reachable only from the saved-search path. That
      was the second half of the previous report's `missing` list and 02-09 did
      not touch it. It is recorded as WINDOWS ledger entry 13, kind `stub`,
      open. It is not part of criterion 4's wording, so it does not fail a
      criterion, but D-2-08's "SEARCH-02's two halves ship together" is not
      honoured on the box path and no written decision says it need not be.
warnings:
  - item: "The backfill answers the coverage column with a looser test than the live writer uses"
    where: "src/data/message_cache/mod.rs:record_whether_the_index_holds_each_messages_text"
    detail: >-
      `index_message_for_search` records the column from whether the indexed
      body had words. The backfill records it from whether a `message_bodies`
      row exists. `get_message_body` returns `None` for a row whose plain and
      markup halves are both absent, which `save_message_body(id, None, None)`
      creates for a message MIME parsing found no text part in. Such a row is
      counted by the backfill as text the box can look inside and is not, so
      for pre-migration rows the number can overstate. Bounded to rows written
      before the migration, one-time, and corrected the next time that message
      is reindexed. It is the same "a row exists" test 02-02's
      `how_much_message_text_is_stored_here` already uses, so it is inherited
      rather than new, but it means ledger 35, the changelog and the function's
      own doc are not exactly right when they say the count is "short rather
      than over": there is a second, smaller population it is over on.
  - item: "The box's sentence is said with the result, not before the search"
    where: "src/presentation/managers.rs:search_messages"
    detail: >-
      Criterion 4 says "before it runs". The count is worked out before the
      search, but the sentence is sent bundled with the match count in one
      `StatusUpdated`. The reason is recorded in the code and in 02-09's
      decisions: two sends share the "status" topic and the announcement queue
      keeps only the newest of a topic, so a separate earlier send would be
      spoken over and one of the two things worth saying would be lost. The
      criterion's purpose is served, its literal wording is not.
  - item: "The box's sentence says 'this computer has the text of'"
    where: "src/application/saved_searches.rs:a_coverage_sentence"
    detail: >-
      True of the saved search, whose number is stored bodies. For the box the
      number is what the index holds, which includes messages whose text was
      evicted, so for those the computer does not have the text and the index
      has the words. The number is the right number for what the box can
      search. The verb is inherited from the saved search's wording and is
      loose on the box path.
  - item: "Neither coverage number deduplicates Gmail rows"
    where: "src/data/message_cache/searching.rs:how_much_message_text_the_index_holds"
    detail: >-
      `search_messages` groups on `COALESCE(m.gmail_msgid, m.id)`, so a message
      carrying three labels is one result. Both coverage counts count rows.
      Pre-existing shape, identical in 02-02's count, not introduced here.
human_verification:
  - test: "Type something into the search box that reaches message text, with NVDA running."
    expected: >-
      The match count and the coverage sentence are heard as one line, and the
      sentence is worth hearing rather than flooding, including on a mailbox
      whose text is entirely here and where it says nothing new.
    why_human: >-
      Audibility, length and whether this becomes noise on every search cannot
      be judged without a screen reader. WINDOWS ledger 33.
  - test: "Search the box for a word in no message at all."
    expected: >-
      The Nothing Found earcon and the coverage sentence are both heard, in an
      order that makes sense.
    why_human: >-
      The two ride different topics at different priorities. That both survive
      the announcement queue is reasoned from the topic rule and has never been
      heard. WINDOWS ledger 34.
  - test: "Run a saved search that has a condition on the message text, with NVDA running."
    expected: >-
      The scope sentence and the coverage sentence are heard as one line before
      the results arrive, and are not swallowed by the "Running this saved
      search" line they share the "status" topic with.
    why_human: "Announcement audibility and topic coalescing cannot be observed without a screen reader. WINDOWS ledger 10 and 17."
  - test: "After that search, tab to the offer above the message list."
    expected: >-
      The button is announced with its full label including the count, and the
      experimental sentence beside it is reachable and read.
    why_human: "MSAA/UIA name resolution is not readable back from wxdragon. WINDOWS ledger 12."
  - test: "Open Message > Saved Searches > Edit Conditions on a saved search, add a condition, and close."
    expected: >-
      The modal opens, the two Choice controls announce as Match field and Match
      type rather than unnamed combo boxes, the Pattern box is skipped cleanly in
      the tab order for the four match types that read no pattern, and the
      per-field caveat line is heard when the field changes.
    why_human: >-
      The condition manager has never been opened in a running build; no modal
      loop has run. WINDOWS ledger 14, 15, 20, 21, 22, 25, 26.
  - test: "With two accounts holding saved searches, arrow through the folder tree."
    expected: >-
      The Saved Searches heading, each account branch under it, and each search
      three levels deep read distinguishably, and landing on a search announces
      that the working account has moved.
    why_human: "The account branches have never been drawn in a running build. WINDOWS ledger 29, 30, 31."
  - test: "Point a real IMAP account at the bulk fetch and press the offer button."
    expected: >-
      A run of hundreds of BODY.PEEK fetches is permitted, and throttling or a
      dropped connection is reported rather than silently ending the run.
    why_human: >-
      No provider has ever seen this code. This is the risk the experimental
      sentence names. WINDOWS ledger 11, and 19 and 32 for the folder-narrowed
      and two-account cases.
---

# Phase 2: Search that says what it covers — Verification Report

**Phase Goal:** A search returns what the user asked for, and says plainly what it could not reach.
**Verified:** 2026-09-01T15:12:00Z (against `main` at `93c59d7`)
**Status:** human_needed
**Re-verification:** Yes — after 02-09 closed the gap the first pass found

## The short answer

The gap is closed, and the number is the right number.

Type something into the search box now and the line that comes back carries how
many matches there were and how much of that account's message text the box
could look inside. It is said on the same `StatusUpdated` as the match count,
so it is painted in the status bar and spoken on the status topic, and it is
reached from Edit > Search and the toolbar button without going near a test.
The number is the box's own: it counts what the search index holds text for, not
what `message_bodies` holds, so a message whose body was fetched and later
evicted is counted as text the box can look inside, which it is. SEARCH-02 is
now delivered on both of the paths its own evidence line is about.

Nothing regressed. The five criteria that passed the first time still pass, and
the saved search's own sentence is unchanged word for word.

Four warnings are recorded below. The one worth reading is that the migration
that fills the new column in for existing rows asks a looser question than the
code that writes it from now on, so the count can overstate for a small
population, which is the opposite of the direction the changelog and the ledger
say it errs in.

## What the first pass found, and what closed it

The first verification, at `0cc5e84` on 2026-09-01, scored 5 of 6. Criterion 4
was half delivered: the saved search said what it covered, the search box read
the same message text through the FTS `body` column and said nothing, so a
body-only word missed silently on the path people reach for first.

Plan 02-09 was written and executed against exactly that, in five commits from
`6283821` to `93c59d7`, two of them RED and two GREEN. The table below is the
first pass's `missing` list, checked item by item against the tree.

| The first pass asked for | Now | Where |
|---|---|---|
| A coverage sentence before a search-box run whose terms can reach message text, worded for the box the way `what_a_saved_search_covers` is worded for the saved search, with its own number | Delivered | `what_the_search_box_covers` (`saved_searches.rs:741`), sent from `search_messages` (`managers.rs:1822`) |
| The fetch offer reachable from the box path, or a deliberate written decision that it is not | Not delivered, and no decision written | `WhatCouldBeFetched` is still sent only from `wx_app.rs:6601` (`run_a_saved_search`) and `18277`. Recorded as ledger 13, open |

The first item is the criterion. The second is not in criterion 4's wording and
is carried forward rather than counted against the score. See "The half that did
not close" below.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A search saved with Subject Only or From Only reruns with that restriction | VERIFIED (re-checked) | `what_that_answer_looks_at` (`saved_searches.rs:499`) and its one caller at `517` are untouched by 02-09. `git diff 0cc5e84..93c59d7` removes nothing from this path. |
| 2 | Opening a saved search says what it asks, in one sentence, whether or not the In box has a name for it | VERIFIED (re-checked) | `a_search_in_words` and `what_a_search_says_as_it_opens` unchanged; still emitted by `run_a_saved_search` at `wx_app.rs:6592`. |
| 3 | A rule editor writes into the same saved searches, reaching all eleven fields | VERIFIED (re-checked) | `wx_managers.rs` and `folder_tree.rs` are not in 02-09's diff at all. |
| 4 | A search that can reach message text says, before it runs, how many messages in the account have body text stored and how many do not | VERIFIED | Both searches now say it. Saved search: `coverage_before` to `how_much_message_text_is_stored_here` to `what_a_saved_search_covers`, unchanged. Box: `search_messages` asks `reads_the_message_text()`, then `how_much_message_text_the_index_holds`, then `what_the_search_box_covers`, and sends the answer with the match count. Two literal deviations from the criterion's wording are recorded as warnings and neither defeats its purpose. |
| 5 | Fetching the missing text is built and gated, with a read dimension on `Allowed`, on by default | VERIFIED (re-checked) | `allowed.rs`, `mail_sync.rs` and `wx_settings.rs` are not in 02-09's diff. |
| 6 | Saved searches sit inside the account structure the way pinned folders do | VERIFIED (re-checked) | `folder_tree.rs` not in 02-09's diff; `whose_mail_a_saved_search_reads` still has its callers. |

**Score:** 6/6 truths verified (0 present, behaviour-unverified)

### Criterion 4, checked in detail, because it is the one that changed

**Is it wired, and does a person reach it?** Yes. Followed in the tree, not read
off the summary:

| Step | Where |
|---|---|
| Edit > Search (Ctrl+F) and the toolbar Search button both raise `ID_SEARCH` | `wx_app.rs:5517`, `741` |
| The handler shows the Find dialog and calls `search_whatever_is_showing` | `wx_app.rs:4602`, call at `4645`. Neither is a test. |
| Its Mail arm calls `search_messages` | `managers.rs:1913` |
| `search_messages` asks the count, builds the sentence, sends it with the match count as one `StatusUpdated` | `managers.rs:1822` to `1874` |
| `handle_update`'s `StatusUpdated` arm paints it and calls `announce_topic(status, Priority::Low, "status")` | `wx_app.rs:15038` |

**Is the answer used, or computed and thrown away?** Used. The `covered` value
is destructured into the status string in both branches: bundled with the count
when there are matches, sent on its own when there are none. Three tests hold
this by reading the status line a person would have been given rather than by
reading the manager's source for a function name, which is the failure mode
02-02's first guard had:

- `test_a_search_from_the_box_says_how_much_of_the_mail_it_could_look_inside` (`managers.rs:3441`)
- `test_a_search_from_the_box_that_found_nothing_still_says_what_it_could_look_inside` (`3484`)
- `test_a_search_that_reads_no_message_text_is_not_given_a_coverage_figure` (`3533`)

**Is it the right number?** This is what the brief asked to be checked hardest,
and it holds.

The box's coverage is not the saved search's. `evict_bodies_over` deliberately
does not reindex (D-2-13), so a message whose body was fetched and later evicted
is gone from `message_bodies` and still in the index, findable by the box. Using
02-02's number for the box would have understated what the box had just
searched.

`how_much_message_text_the_index_holds` (`searching.rs:600`) counts
`messages.text_is_in_the_search_index`, written by `index_message_for_search`
(`searching.rs:379`) from `body.as_deref().is_some_and(|text| !text.is_empty())`,
which is the same value that goes into the index in the same statement pair. So
the column records what the index holds, not what `message_bodies` holds, which
is the property the sentence claims.

The query agrees with `search_messages` on all three things it has to agree on:
the account join through `folders`, `m.deleted = 0`, and the folder narrowing
through the same `NO_FOLDER_IN_PARTICULAR` convention. A search of one folder is
told about that folder.

The divergence is proved by a test that asserts its own fixture before it trusts
a number. `test_a_message_whose_text_was_evicted_is_still_text_the_box_can_look_inside`
(`searching.rs:962`) evicts, then asserts the box still finds the evicted word
(so the fixture can tell the two coverages apart), then asserts
`how_much_message_text_is_stored_here` reports 0 with text and
`how_much_message_text_the_index_holds` reports 1 of 2. If the fixture stops
discriminating the test says so rather than passing quietly.

The premise the brief flagged as unverifiable is handled honestly. `message_search`
is declared `content=''` (`mod.rs:2170`), so its `body` column reads NULL however
much is indexed and it cannot be counted. 02-09 measured that rather than
reasoning about it, measured the one thing that can answer exactly
(`fts5vocab`, 92 ms over 2,000 messages, about nine seconds at the scale this
program targets), and recorded the fact where it is decided instead. The
reasoning and both measurements are written on
`record_whether_the_index_holds_each_messages_text`.

**Two types, not one.** `TextTheIndexHolds` is separate from `TextStoredHere`,
so handing one search's numbers to the other's sentence does not compile. That
is the exact false claim the sentences exist to prevent, refused by the compiler
rather than by a reader.

**One sentence builder, two openings.** `a_coverage_sentence` takes a
`WhichSearch` value rather than three `&str` arguments, so the three phrasings
that differ cannot be handed over in the wrong order. The saved search's three
cases are word for word what 02-02 shipped, checked against the diff.
`test_one_place_builds_the_coverage_sentence` (`saved_searches.rs:1298`) now
requires each opening once in `saved_searches.rs` and zero times in both
`wx_app.rs` and `managers.rs`.

### The half that did not close

The first pass listed two missing things. The second, the fetch offer reachable
from the box, is unchanged: `UIUpdate::WhatCouldBeFetched` is still sent only
from `run_a_saved_search` and from the end of a fetch run. 02-09 did not touch
it and wrote no decision saying the box does not need it.

It stays out of the score because criterion 4 is about saying, not about
offering, and the offering criterion (5) was and remains verified. But it should
be said plainly that D-2-08's own words are "SEARCH-02's two halves ship
together", and on the box path only one half ships. It is recorded as ledger
entry 13, kind `stub`, open, and no later phase in the roadmap covers it, so it
is not a deferral either. It is a known narrow reach with a documented reason
and no closing decision.

### Warnings

Four, none of them a gap. Each is stated in full in the frontmatter; this is the
one worth reading in the report.

**The backfill asks a looser question than the live writer.**
`index_message_for_search` records the column from whether the indexed body had
words, and says so in its own comment: "what is recorded is whether there were
words, not whether there was a row".
`record_whether_the_index_holds_each_messages_text` fills existing rows in with
`WHERE EXISTS (SELECT 1 FROM message_bodies ...)`, which is whether there was a
row. Those are different questions for a message MIME parsing found no text
part in: `save_message_body(id, None, None)` writes such a row,
`get_message_body` returns `None` for it (`bodies.rs:375`), so the live writer
records 0 and the backfill records 1.

The population is small and bounded: rows written before the migration only, and
any later reindex of that message corrects it. But the direction is the
overstating one, and ledger 35, the changelog entry and the function's own doc
all say the count is "short rather than over, and the set never grows". That
sentence is right about the eviction population it was written for and does not
account for this one. The same "a row exists" test is what 02-02's
`how_much_message_text_is_stored_here` already uses, so it is inherited rather
than introduced, which is why it is a warning and not a gap.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `src/data/message_cache/searching.rs` | The count, the column name, the narrowing question, the type | VERIFIED | `THE_INDEX_HOLDS_THE_TEXT` (140), `TextTheIndexHolds` (152), `WhereToSearch::reads_the_message_text` (85, every answer spelled out, no wildcard), `how_much_message_text_the_index_holds` (600) |
| `src/data/message_cache/mod.rs` | The migration and its reasoning | VERIFIED | `record_whether_the_index_holds_each_messages_text` (2793), called from the migration block at 2306; re-export at 20 |
| `src/application/saved_searches.rs` | The box's sentence, built beside the saved search's | VERIFIED | `THE_BOX_READS_MESSAGE_TEXT` (651), `WhichSearch` (662), `what_the_search_box_covers` (741), `a_coverage_sentence` (753) |
| `src/presentation/managers.rs` | The coverage disclosure on the box path | VERIFIED (was MISSING) | `search_messages` (1791) asks the count, builds the sentence, and sends it with the count in both the found and the empty branch |
| `src/presentation/wx_app.rs` | Untouched by 02-09, the saved-search path intact | VERIFIED | `coverage_before` (6402) and its caller (6592) unchanged |
| `guards/guards.toml` | Three new records, sweep numbers adding up | VERIFIED | `grep -c '^\[\[guard\]\]'` gives 547, which is 192 + 355 as the summary claims; each new record names a test that exists |
| `docs/changelog.md` | The box's sentence, and which way the number is wrong, where a person meets it | VERIFIED | New entry states the number differs from the saved search's, why, and the pre-existing-database limitation |

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `wx_app.rs:4645` (`ID_SEARCH` handler) | `managers::search_whatever_is_showing` | Find dialog result | WIRED |
| `search_whatever_is_showing` | `managers::search_messages` | `PimModule::Mail` arm | WIRED |
| `managers::search_messages` | `how_much_message_text_the_index_holds` | `reads_the_message_text().then(...)` | WIRED (was NOT WIRED) |
| `managers::search_messages` | `what_the_search_box_covers` | `.map(...)` on the count | WIRED |
| `managers::search_messages` | the status bar and the status topic | one `UIUpdate::StatusUpdated` carrying both halves | WIRED |
| `index_message_for_search` | `messages.text_is_in_the_search_index` | `UPDATE ... WHERE id = ?2 AND {column} IS NOT ?1` | WIRED |
| `run_a_saved_search` | `how_much_message_text_is_stored_here` | `coverage_before` | WIRED (unchanged) |
| `managers::search_messages` | `UIUpdate::WhatCouldBeFetched` | nothing | NOT WIRED — carried open, ledger 13, outside criterion 4 |

### Data-Flow Trace (Level 4)

| Value shown | Source | Real data | Status |
|---|---|---|---|
| The box's coverage numbers | `SELECT COUNT(*), COUNT(CASE WHEN m.text_is_in_the_search_index = 1 ...)` over `messages` joined to `folders`, narrowed the same way the search is | Yes | FLOWING |
| The box's match count | `search_messages` result length | Yes | FLOWING |
| The saved search's coverage numbers | `SELECT COUNT(*), COUNT(b.message_id) ... LEFT JOIN message_bodies` | Yes | FLOWING (unchanged) |
| Offer count | `messages_with_no_text_here` | Yes | FLOWING (unchanged) |
| Saved-search rows | `every_saved_search` per account | Yes | FLOWING (unchanged) |

### Behavioural Spot-Checks

Not run here, by instruction and by economy. `bash scripts/check.sh all` was run
by the developer on this tree at `93c59d7` and passes: 5,986 library tests,
every integration target, the release build. Compiling to re-run one named test
would duplicate a gate that has already answered.

Test existence was confirmed by enumeration instead, and every test carrying
criterion 4 is named above with its line. The three that hold the box's sentence
are behavioural rather than source-reading: they run the search and assert on
the status line a person would have received, so an implementation that computes
the answer and discards it fails them however it discards it.

No probe scripts exist in this repository (`scripts/` holds `check.sh`,
`guards.sh`, `red-commit.sh` and build helpers), so Step 7c does not apply.
`scripts/guards.sh` was not run, per the brief.

### Anti-Patterns Found

None. Every file 02-09 modified was scanned for `TBD`, `FIXME`, `XXX`, `TODO`,
`HACK`, `PLACEHOLDER`, `todo!`, `unimplemented!` and "not yet implemented".
There are no matches, so there is no unreferenced debt and completion stays
auditable.

### Test Quality Audit

| Check | Result |
|---|---|
| Disabled or skipped tests on the new work | None. No `#[ignore]` added by 02-09. |
| Circular expected values | None. The coverage tests compare a status line against `what_the_search_box_covers` applied to a fixture whose numbers are asserted independently first. |
| Assertion strength | Behavioural on the box path (run the search, read what was said), value-level on the counts. |
| Green-on-arrival tests | Two, named in the summary rather than left to look like test-first, each with a recorded break that reddens it. |

The self-report of two tests that could not go red, and of the one rule of the
brief that was broken (a one-line Python substitution in `mod.rs`), is in
02-09-SUMMARY.md and is accurate as far as this report can check it.

### Requirements Coverage

| Requirement | Status | Evidence |
|---|---|---|
| SEARCH-01 | SATISFIED | Unchanged since the first pass. All four sub-criteria. |
| SEARCH-02 | SATISFIED | **Now delivered.** The requirement's own cited evidence line is "FTS covers subject and sender for everything and body text only for bodies actually fetched, and the search UI must say so." The search UI now says so, with the number that describes the FTS coverage rather than the stored-body coverage. The fetch half is built, gated and reachable, though only from the saved-search path. The fetch has never met a live server, which is a human item. |
| SEARCH-03 | SATISFIED | Unchanged since the first pass. |

**Does `REQUIREMENTS.md` state the truth about all three?** Yes, now. All three
are marked `[x]` and the coverage table says "Complete" for each, and as of
`93c59d7` that is true for all three. The first pass's instruction to reopen
SEARCH-02 or accept an override is obsolete: the gap was closed rather than
accepted, so no override is needed and none was added.

One caveat, stated so it is not discovered later. SEARCH-02's second `[D]`
bullet reads "says, before it runs, how much of the mailbox has body text
stored, **and offers to fetch the rest**". The offering half is built and
reachable, but not from the box. The requirement is satisfied in the sense that
both halves exist and both are reachable by a person; it is not satisfied in the
sense that both are reachable from the same place.

## What is not verifiable here and is not counted as passing

Nothing in this phase has been drawn in a running build, heard under a screen
reader, or run against a live mail account. The ledger stands at 33 open
entries, 25 of them raised by this phase and almost all `unrun-verify`. 02-09
added two more of exactly that kind: the box's coverage sentence has never been
heard, and it is now appended to every search that reads message text including
those where it says nothing new, which may be useful or may be flooding; and in
the empty case the earcon and the sentence ride different topics at different
priorities, and that both survive the announcement queue is reasoned from the
topic rule rather than heard.

These are listed under Human Verification in the frontmatter. They are not gaps
in the work and are not counted against the score. Screen reader testing is
Pratik's and he decides when.

## Summary

The phase goal holds on both of its halves and on both of this program's
searches. A search returns what the user asked for, and both searches now say
plainly what they could not reach, each with its own number, each naming which
search it is about, because they genuinely cover different amounts of the same
mailbox and one number would have been wrong about one of them.

The work that closed the gap did the harder thing rather than the cheaper one.
The premise it was given turned out to be unbuildable, the obvious substitute
would have overstated after an index rebuild, and both were measured rather than
argued about. What shipped errs short for the population it cannot know about,
says so in the changelog where a person could notice the discrepancy, and refuses
at compile time to let one search's numbers be told about the other.

Status is `human_needed` rather than `passed` because seven things need a
running build, a screen reader or a live account, two of them new with this
plan. There are no gaps.

---

*Verified: 2026-09-01T15:12:00Z*
*Verifier: Claude (gsd-verifier)*

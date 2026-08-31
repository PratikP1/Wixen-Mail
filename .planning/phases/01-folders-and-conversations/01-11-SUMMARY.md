---
phase: 01-folders-and-conversations
plan: 11
subsystem: message-list
tags: [conversations, rfc-5256, mail-parser, sqlite, aggregates, accessibility, settings]

requires:
  - phase: 01-02
    provides: "messages.thread_id, written on every row by upsert_message from thread_identity::conversation_root, plus idx_messages_thread"
  - phase: 01-06
    provides: "MessageColumn::ALL as a fixed-size array with a match arm per column, so a new variant breaks every arm at compile time"
provides:
  - "application::conversations::name_of: what a conversation is called, from mail_parser's RFC 5256 base subject, in seventeen languages"
  - "application::conversations::opens_with_a_reply_marker / opens_with_a_forward_marker: the one answer to whether a subject already carries a marker, used by the compose box"
  - "application::conversations::counts_read_as: D-03's wording, '5 messages, 2 unread'"
  - "application::conversations::ConversationItem: one conversation, every field an aggregate over the whole of it"
  - "application::conversations::AConversationReaches: the fourth of the five settings, defaulting to the whole account"
  - "MessageCache::conversations_in: the account-wide grouping, with the all-mail exclusion and the reach as a bound parameter"
  - "MessageColumn::conversation_sort_expression: D-02's table as fifteen fixed strings, which fill the fields and build the ORDER BY"
  - "Sort::conversation_order_by_clause: the same two-level sort, expressed over conversations"
  - "message_rows::conversation_cell_text: one arm per column over a whole conversation"
  - "The connection now knows conversation_name(), so SQL and Rust name a conversation the same way"
affects: [01-12, 01-13]

actuals:
  tokens: 29273
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A rule SQL cannot express is taught to the connection as a scalar function rather than approximated in the query, which is what fold_case_the_way_rust_does already does for `lower`"
    - "The SELECT list of an aggregate query is built from the same per-column strings the ORDER BY is, so a displayed value and its sort key are one expression rather than two kept in step"
    - "A membership question answered by a dependency is asked by probing that dependency, not by copying its list: `thread_name(\"AW: x\") == \"x\"` says AW is a marker without a marker list here"
    - "A list of people crosses a SQLite aggregate one per line, because group_concat separates with a comma and a display name may contain one"

key-files:
  created:
    - src/application/conversations.rs
  modified:
    - src/data/message_cache/messages.rs
    - src/data/message_cache/mod.rs
    - src/presentation/message_columns.rs
    - src/presentation/message_rows.rs
    - src/presentation/wx_compose.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_settings.rs
    - src/data/config.rs
    - src/application/mod.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "The plan's rule for the compose box, 'prepend when stripping would change the subject', is wrong and was not implemented. mail_parser strips a bracketed list tag and a trailing (fwd) as well as a marker, so that rule leaves '[mailing-list] hello' and 'hello (fwd)' with no Re: at all. The question asked instead is 'does this open with a marker', and reply and forward are asked separately so a reply to a forward still says Re: Fwd:"
  - "D-02's Subject column cannot be sorted in SQL by the raw oldest subject, or a row reading 'Quarterly report' would sort under R. The connection is taught conversation_name() and the query calls it, so the cell and the ORDER BY are one function"
  - "Safety's conversation rule ranks by severity in SQL, because the verdicts are stored as words whose alphabet puts the mildest of the three last"
  - "conversations_in takes an order_by, which the plan's signature did not, so the sort expression has a caller and the agreement between a cell and its sort can be run rather than described"
  - "The all-mail exclusion applies only to the account-wide reach. Counting one folder cannot double anything, and excluding All Mail there would leave somebody standing in it with no rows"
  - "ConversationItem carries newest_received and newest_sent rather than one 'newest', because D-02 gives Received and Sent their own rules and one of them is sender controlled"
  - "Version left at 0.46.0. 01-02 bumped it this cycle and nothing here is a schema change; the user-visible changes are in the changelog under [Unreleased]"

patterns-established:
  - "The first guard record in this file whose break fills one field from another column's arm: the list still comes back in the right order and every row says the wrong thing, which is what a right sort with a wrong cell looks like"
  - "A record from an earlier plan re-measured within the same day, because this plan's tests made its break redden seventeen instead of five"

requirements-completed: []

coverage:
  - id: D1
    description: "A conversation is named by the oldest message present, with reply and forward marker chains taken off, in seventeen languages, and the name changes when older mail arrives"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "cargo test --lib conversations:: (15 tests, all new)"
        status: pass
      - kind: integration
        ref: "src/data/message_cache/messages.rs#tests::test_an_older_message_arriving_renames_the_conversation"
        status: pass
      - kind: other
        ref: "grep -rn 'fn .*strip.*prefix|const RE_PREFIXES|\"Re:\"' src/ --include=*.rs | grep -v 'mod tests' shows no prefix list added by this plan"
        status: pass
    human_judgment: false
  - id: D2
    description: "The compose box writes one marker, in any case and any language, and a reply to a forward still says Re: Fwd:"
    verification:
      - kind: unit
        ref: "src/presentation/wx_compose.rs#tests::test_a_reply_marker_in_another_language_does_not_get_a_second_one"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_compose.rs#tests::test_a_subject_carrying_no_marker_comes_back_from_a_reply_unchanged (round trip over eight subjects)"
        status: pass
      - kind: unit
        ref: "src/presentation/wx_compose.rs#tests::test_a_mailing_list_tag_is_not_a_marker_and_still_takes_one"
        status: pass
    human_judgment: false
  - id: D3
    description: "A conversation's size and unread count are the same in every folder it touches, Gmail does not double, deleted messages are out, and two accounts do not merge"
    requirement: THREAD-01
    verification:
      - kind: integration
        ref: "cargo test --lib message_cache::messages (12 new conversation tests)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a folder holding a copy of every message is left out of a conversation count (measured, reddens exactly 2)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Every column answers about the conversation, and the value it shows is the value it sorts by"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "cargo test --lib message_rows:: (12 new tests, one per column family)"
        status: pass
      - kind: integration
        ref: "src/data/message_cache/messages.rs#tests::test_sorting_by_a_column_agrees_with_what_that_column_says (all 15 columns, both directions)"
        status: pass
      - kind: other
        ref: "guards/guards.toml: a conversation column's display and its sort come from the same enum arm (measured, reddens exactly 3)"
        status: pass
    human_judgment: false
  - id: D5
    description: "The Thread column says how many messages and how many unread, in words, instead of a conversation identifier"
    requirement: THREAD-01
    verification:
      - kind: unit
        ref: "src/presentation/message_rows.rs#tests::test_the_thread_cell_says_the_counts_rather_than_an_identifier"
        status: pass
      - kind: unit
        ref: "src/application/conversations.rs#tests::test_a_conversation_says_how_many_messages_and_how_many_unread"
        status: pass
    human_judgment: true
    rationale: >
      The words are proved. What is not proved is what NVDA reads for a collapsed
      conversation row, because no such row is drawn yet: 01-12 renders the list.
      The wording is reachable today only through the conversation window's own
      name, which is a different surface. A screen reader run has to wait for the
      row.
  - id: D6
    description: "How far a conversation reaches is a setting, offered on the Reading page and read when a conversation is opened"
    verification:
      - kind: unit
        ref: "src/data/config.rs#permission_tests::test_a_settings_file_written_before_a_conversation_had_a_reach_reaches_the_whole_account"
        status: pass
      - kind: unit
        ref: "cargo test --lib config::every_setting_is_acted_on (both the read-by-something and offered-by-a-screen guards)"
        status: pass
    human_judgment: false

duration: 3h 5m
completed: 2026-08-30
status: complete
---

# Phase 01 Plan 11: What a conversation is, and what its row says

**A conversation is now named after its oldest message with the reply and forward markers off, counted across the whole account rather than the folder you happen to be standing in, and described column by column by one rule per column that serves both what the row says and what the list sorts by.**

## Performance

- **Duration:** about 3 hours 5 minutes
- **Completed:** 2026-08-30
- **Tasks:** 3 of 3
- **Files created:** 1. **Files modified:** 11
- **Commits:** 7, three RED and three GREEN pairs plus the records and the changelog

About 20 minutes of that is measurement rather than work: four whole-library
runs at roughly 190 seconds each, three of them to measure guard records by
hand and one as a baseline. Every other run in this plan was targeted, which is
the difference between one second and 190.

## What works, plainly

Opening a conversation now says what it is about and how big it is. Before, it
was titled with whichever message the cursor happened to be on, so opening one
from a reply read out "Re: Re: Quarterly report" before reaching the two words
that mean anything, and the size it announced was however much of that folder
happened to be loaded. It now says the conversation's own name and the counts
for the whole account.

Replying no longer grows a chain of markers. `AW: Angebot` used to become
`Re: AW: Angebot`, and again on the next reply, because the old check
recognised the exact ASCII `"Re: "` and nothing else.

The data a collapsed conversation row needs is built, tested per column, and
proved to sort by what it shows. **Nothing draws that row yet.** That is 01-12,
and this plan's own objective says so. The half that is reached today is the
count and the name; the half that is not is the list.

## Accomplishments

- **`name_of` takes nothing from here.** `mail_parser 0.11.5` was already a
  dependency and already implements RFC 5256's base subject with nineteen reply
  and twenty-two forward markers across seventeen languages. Tested in German,
  Swedish, Dutch, Italian, Polish and Chinese, plus the bracketed `Re[2]:` and
  `[fwd: ]` forms.

- **The compose box and the conversation list now answer one question in one
  place.** `opens_with_a_reply_marker` and `opens_with_a_forward_marker` are
  both asked of `mail_parser` rather than of a list kept here, and the round
  trip is pinned by a test over eight subjects: prepending a marker and then
  naming the conversation gives the subject back.

- **`conversations_in` counts across the account.** Grouped on `thread_id`,
  filtered to the account, excluding deleted messages and folders a server says
  hold a copy of everything, and restricted to the conversations that touch the
  folder being read. The reach is a bound parameter rather than a branch around
  two queries, so both answers go down one path.

- **One rule per column, and it is one string.** `conversation_sort_expression`
  gives fifteen fixed strings; `conversations_query` selects those very strings
  into `ConversationItem`'s fields, and `conversation_order_by_clause` puts the
  same strings in the `ORDER BY`. A test sorts by every one of the fifteen
  columns in both directions against a real database and asserts the order
  matches what the cells say.

- **The Thread column says "5 messages, 2 unread".** D-03. It held a mail
  server's angle-bracketed identifier, which is what THREAD-01's announcement
  would otherwise have had to read out, because this list control has no
  per-item accessible name.

- **The fourth of the five settings**, offered in D-42's group on the Reading
  page, stored, read back by the words shown, and acted on. Both settings
  guards pass.

## Deviations from Plan

### The plan's rule for the compose box is wrong, and was not implemented

**Found during:** Task 1, before writing anything.

**Issue:** `01-RESEARCH.md` §Q6, Pitfall 4 and the plan's own `<action>` all say
to rewrite the prepender as "a prefix is needed exactly when stripping would
change the subject", spelled `thread_name(subject) == subject.trim()`. Two
counterexamples come out of `mail_parser`'s own test table:

- `thread_name("[mailing-list] hello world")` is `"hello world"`. A bracketed
  list tag is stripped, so the rule answers "already has a marker" and a reply
  to a mailing list message would go out with no `Re:` at all.
- `thread_name("hello world (fwd)")` is `"hello world"`. RFC 5256 has a trailing
  `(fwd)` as a forward marker, so the same thing happens to any subject ending
  in a parenthesised word the crate knows.

The plan's intent is right and its formula is not: `thread_name` answers "what
is this conversation called", and the compose box needs "does this open with a
marker". They are different questions.

**Fix:** The token the subject opens with is taken by its colon, in the shape
RFC 5256 defines, and whether that word is a marker is asked of `mail_parser`
by probe rather than by a list kept here. Reply and forward are asked
separately, using `trim_trailing_fwd` for the forward set, so a reply to a
forward still says `Re: Fwd:` and a forward of a reply still says `Fwd: Re:`.

**Boundary, stated rather than hidden:** `trim_trailing_fwd` ignores a
parenthesised word of one character, so Hungarian's one-letter `I:` is read as
a reply marker. Forty of the forty-one markers are read correctly, against one
before this change. It is written into the function's own doc comment.

**Files modified:** `src/application/conversations.rs`,
`src/presentation/wx_compose.rs`. **Commits:** `1dc0837`, `441a08c`.

Four of the fifteen tests in that RED commit are counterexample guards: they
pass against today's naive code and go red under the rule the research
proposed.

### [Rule 2] The Subject column cannot be sorted in SQL, so SQL was taught the rule

**Found during:** Task 3, working out what `conversation_sort_expression` can
return for `Subject`.

**Issue:** D-02 requires the displayed value and the `ORDER BY` to come from one
rule. A conversation's displayed subject is its name, which is the oldest
message's subject with a chain of markers in seventeen languages taken off.
There is no SQL expression for that. Ordering by the raw oldest subject would
put a row reading "Quarterly report" under R, which is precisely the
disagreement D-02 exists to prevent.

**Fix:** `conversation_name()` is registered on the connection, backed by
`conversations::name_of`. `MessageCache` already does this for `lower`, and its
doc comment gives the same reason: "no second spelling of the same question to
keep in step". The query calls it in the `SELECT` and the `ORDER BY` names the
same expression.

**Files modified:** `src/data/message_cache/mod.rs`. **Commit:** `5b9f39f`.

### [Rule 1] Safety's worst is not the largest of the words it is stored as

**Found during:** Task 3.

**Issue:** Safety is stored as "ordinary", "suspicious", "spam" and "phishing"
so a stored mailbox can be read by a person. `MAX(m.safety)` returns
"suspicious", the mildest of the three, so the naive expression would call a
conversation containing phishing merely suspicious.

**Fix:** The conversation expression ranks by severity with a fixed `CASE`,
worst last, matching the order the enum is declared in. The message-level
expression is untouched and now disagrees with it about what "worse" means;
that is pre-existing and is written into `deferred-items.md` rather than
changed here.

**Files modified:** `src/presentation/message_columns.rs`. **Commit:**
`5b9f39f`.

### [Rule 1] A list of people cannot cross a SQLite aggregate on commas

**Found during:** Task 3, writing the Correspondent cell.

**Issue:** `GROUP_CONCAT(DISTINCT x)` separates with a comma and takes no other
separator. A display name may contain one, so `"Smith, John" <j@example.com>`
would come back as two people.

**Fix:** Each value is prefixed with a line break inside the aggregate and the
cell splits on lines. The same reasoning `safety_reasons` already records for
storing its sentences a line apiece. A test asserts the comma case.

**Files modified:** `src/presentation/message_columns.rs`,
`src/presentation/message_rows.rs`. **Commit:** `2958b06`, `9c94486`.

### `conversations_in` gained a parameter the plan's signature did not have

**Issue:** The plan gives `conversations_in(folder_id, account_id, reach)` and
separately asks for `conversation_sort_expression` with a test that "sorts
conversations by each column and asserts the order agrees with the displayed
value". With no way to pass an order, that test could not be run and the sort
expression would have had no caller at all.

**Fix:** `order_by: Option<&str>` was added, mirroring
`get_message_list_sorted` exactly, including the doc comment naming
`Sort::conversation_order_by_clause` as the only thing it may come from.
`None` is newest first.

**Commit:** `5b9f39f`.

### A setting, its screen and its consumer had to ship together, so wx_app.rs was touched

**Issue:** `files_modified` does not list `src/presentation/wx_app.rs`, and the
plan places the rendering of conversation rows in 01-12. But
`test_every_setting_somebody_can_change_is_read_by_something` requires the new
setting to be read by a shipping file that is not `config.rs` or
`wx_settings.rs`, and 01-09's summary records the rule that a setting ships in
the same commit as its screen and its consumer. A `pub fn` that reads the
setting and is called by nothing would satisfy the guard mechanically and be
exactly the stub CLAUDE.md's third guardrail forbids.

**Fix:** The consumer is the conversation window, which is reachable today.
`how_a_conversation_reads` builds one string carrying D-04's name and D-03's
counts, counted through `conversations_in` under the stored reach, and the
window reads that one string for both its tree's accessible name and its
announcement. Those two disagreed before this: the announcement counted the
loaded page and the title carried whichever message the cursor was on.

**Commit:** `5b9f39f`.

### The wording function's red was taken inside the green commit

**Issue:** `counts_read_as` is D-03's wording and belongs to task 3, but task
2's consumer needs it, and task 2's commit cannot be split because the two
settings guards must both be green in the commit that adds the setting.

**Fix:** Its four tests were taken red by hand against a stub inside that
commit, confirmed red, and the body restored. Said in the commit message rather
than left to be inferred.

### One guard record from 01-02 had fallen behind within the day

**Found during:** Task 3, running `scripts/guards.sh conversation`.

**Issue:** "the column that says which conversation a message is in has a
writer" named five tests. Its break now reddens seventeen, because this plan's
listing groups on `thread_id` and twelve new tests depend on the writer.

**Fix:** Re-measured by hand against the whole library at 5674 tests, and the
record rewritten with all seventeen and with why the number changed. The run
found it because it asks in both directions, which is the reason that file
works.

**Commit:** `ca880eb`.

## Known Stubs

| What | Where | Why, and what closes it |
|---|---|---|
| `conversation_cell_text` has no non-test caller | `src/presentation/message_rows.rs` | 01-12 renders the collapsed conversation list. This plan's objective is "the data every row in plan 01-12 renders". Every column is tested and the agreement with the sort is run against a real database; what is missing is the list control that asks for the cells. |
| `Sort::conversation_order_by_clause` has no non-test caller | `src/presentation/message_columns.rs` | The same. `conversations_in` accepts it and the tests use it; 01-12 is what passes the user's chosen sort. |

Neither is a stub in the sense CLAUDE.md's third guardrail is about: nothing
looks done and does nothing in front of a user, because nothing is in front of
a user yet. They are recorded so the boundary between this plan and 01-12 is
visible rather than implied. Both are also in `deferred-items.md`.

## Deferred Issues

Three entries added to
`.planning/phases/01-folders-and-conversations/deferred-items.md`:

- **Gmail mail archived with no label disappears from a conversation count.** A
  consequence of D-08's exclusion being by folder rather than by message
  identity. Medium, and it needs a decision rather than a fix.
- **Sorting messages by Safety orders them alphabetically rather than by
  severity.** Pre-existing, untouched, and now visibly inconsistent with the
  conversation rule beside it. Small.
- **The conversation row is built and tested and nothing draws it yet.** 01-12.

## Verification

| Check | Result |
|---|---|
| `cargo test --lib` | 5674 passed, 0 failed, 1 ignored |
| `cargo test --all-targets` | 23 targets, all ok |
| `bash scripts/check.sh` | formatting and clippy pass |
| `bash scripts/guards.sh conversation` | 5 guards, all redden exactly the tests their records name |
| `cargo test --test house_style` | 52 passed |
| No hand-rolled marker list | `grep` finds none added by this plan |
| `conversation_size` unmodified | `git diff` over `message_rows.rs` shows no change to it |
| `git diff` for `#[allow(...)]` or `unwrap` outside tests | none added |

The spellcheck flake recorded in `deferred-items.md` did not appear in any of
the four whole-library runs here.

## Requirements

**THREAD-01 is not ticked.** Its criterion is that a collapsed conversation row
is announced from its visible columns the way any other row is, and no
collapsed row exists yet. This plan built what such a row would say and proved
each column of it; 01-12 draws the list. Recorded as advanced, not completed.

## Version

**Left at 0.46.0.** 01-02 raised it from 0.45.0 this cycle for a behaviour
change, and CLAUDE.md asks for a bump per feature or behaviour change rather
than a jump at release time, so an argument exists for another. Not taken,
because the two user-visible changes here are in the changelog under
`[Unreleased]` and the version has not been tagged or handed to anybody since
the last bump, so a second raise would only make the number move twice for one
unreleased batch. Said either way, as the plan asked.

## Notes for Future Phases

- **01-12** takes `conversations_in`, `conversation_cell_text` and
  `Sort::conversation_order_by_clause` and draws the list. Everything it needs
  is a pure function over data already in memory, which is what the virtual
  text callback requires.
- **01-13** takes D-07, acting on a whole conversation. `ConversationItem`
  carries the count the confirmation has to name.
- **A rule SQL cannot express** now has a house answer in this codebase: teach
  the connection the Rust function. `fold_case_the_way_rust_does` and
  `teach_it_what_a_conversation_is_called` sit next to each other and give the
  same reason.

## Self-Check: PASSED

Checked against the tree rather than against this document.

- Both files claimed created exist: `src/application/conversations.rs`,
  `01-11-SUMMARY.md`.
- All seven commit hashes resolve: `1dc0837`, `441a08c`, `270671b`, `5b9f39f`,
  `2958b06`, `9c94486`, `ca880eb`.
- All eleven symbols claimed in `provides` are in the tree.
- `conversation_size` is unmodified: the only mention in
  `git diff 3350c52..HEAD -- src/presentation/message_rows.rs` is a hunk header.
- No `#[allow(...)]` added anywhere; every `unwrap`/`expect` added is inside a
  `mod tests`.
- Test counts as claimed: `conversations::` 15, `message_rows::` 38,
  `message_cache::messages` 109, whole library 5674 passing.

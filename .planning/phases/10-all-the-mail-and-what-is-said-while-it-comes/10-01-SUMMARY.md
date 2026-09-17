---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 01
subsystem: mail sync, application model, guards
tags: [download, chunking, backoff, text-pass, scripted-mailbox, guards]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "the tree at 1.0.0-alpha.1; the five issues of the third group re-checked against 7d57cd49 in the phase README"
provides:
  - "application::trying_again: WaitBeforeTryingAgain, thirty seconds doubling to a thirty-minute cap, reset by a success, a count of failures in a row, and a sentence that says the wait the way a person says it; no clock, no sleeping"
  - "application::bringing_everything_down: what_to_do_next over FolderHere, TextStillMissing, TextBudget, on_screen and reading_allowed, answering the next chunk of headers, the next chunk of text, or that everything is here and why; HEADERS_PER_CHUNK by name, TEXT_PER_CHUNK_MESSAGES 50, TEXT_PER_CHUNK_BYTES 16 MiB; HowMuchIsHere and stopped_coming_down moved here from asking_for_a_whole_folder; the six sentences"
  - "MessageToFetch.size_bytes, read by messages_with_no_text_here as COALESCE(m.size_bytes, 0)"
  - "mail_sync::fetch_over_a_mailbox taking one chunk, a stop and an after-each, ending Stopped or TheServerStoppedAnswering { because: WhyTheServerStopped } after REFUSALS_THAT_MEAN_THE_SERVER_HAS_STOPPED failures in a row; fetch_all_the_missing_text folding chunks for the kept entry point; the two endings worded"
  - "imap::THE_SERVER_STOPPED_RESPONDING, the timeout phrase as a constant"
  - "eleven guard records, every one measured; the how_many record corrected by one test"
affects: [10-05 (the runner asks what_to_do_next, fetch_over_a_mailbox and WaitBeforeTryingAgain), 10-06 (the watch asks WaitBeforeTryingAgain), 10-03 (keeping_message_text turns the setting into TextBudget), 10-07 (reads this plan's clauses for MAIL-01, MAIL-03 and MAIL-04)]

actuals:
  tokens: 24700
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A pure decision module whose inputs are values read out of the cache, so the runner's state is the cache and a restart resumes with no state file"
    - "A wait rule that answers a Duration and never sleeps, served by whoever holds a timer"
    - "A provider's answer read per chunk: a run of failures of one kind ends the chunk, classified from the error's kind and never its text"

key-files:
  created:
    - src/application/trying_again.rs
    - src/application/bringing_everything_down.rs
  modified:
    - src/application/mod.rs
    - src/application/mail_sync.rs
    - src/application/asking_for_a_whole_folder.rs
    - src/data/message_cache/bodies.rs
    - src/service/protocols/imap.rs
    - guards/guards.toml

key-decisions:
  - "what_to_do_next takes TextStillMissing { messages, kept_bytes } where the plan's signature had text_missing alone, because a budget on bytes kept needs the bytes kept as an input and the plan's signature had no place for them"
  - "The sentences write bare numbers, 3500 of 12872, as every counted sentence in the tree does; the plan's examples had thousands separators and nothing in the tree writes them"
  - "The whole-list run is fetch_all_the_missing_text, generic over Mailbox, and the kept entry point delegates to it, because the kept entry point takes a real MailController and the tests of the fold have to run against the scripted mailbox"
  - "A Network error is told from a timeout by the IMAP layer's own phrase, made a constant, rather than by a copy of the phrase in mail_sync"
  - "The red commit that moved the stopped-coming-down rule left the old loop pointing at its own copy until green, because the red stub answers false and the loop's existing test would have run forever under it"

patterns-established:
  - "A red stub under an assertion fails; the same stub under a loop condition spins. Callers that branch on a stubbed answer are re-pointed at green"

requirements-completed: [MAIL-01, MAIL-03, MAIL-04]

coverage:
  - id: D1
    description: "One rule says how long to wait before asking a server again: thirty seconds doubling to a thirty-minute cap, reset by a success, counted, worded"
    requirement: MAIL-04
    verification:
      - kind: unit
        ref: "src/application/trying_again.rs#test_each_failure_doubles_the_wait_until_the_cap"
        status: pass
      - kind: unit
        ref: "src/application/trying_again.rs#test_a_success_puts_the_wait_back_to_the_start"
        status: pass
      - kind: unit
        ref: "src/application/trying_again.rs#test_the_wait_is_said_the_way_a_person_says_it"
        status: pass
    human_judgment: false
  - id: D2
    description: "A pure decision says what a download of everything does next for one account, from the cache and nothing else, in the order on screen, inbox, tree, headers before text, newest first, bounded by count, bytes and budget"
    requirement: MAIL-01
    verification:
      - kind: unit
        ref: "src/application/bringing_everything_down.rs#test_the_folder_on_screen_comes_first_then_the_inbox_then_the_tree_order"
        status: pass
      - kind: unit
        ref: "src/application/bringing_everything_down.rs#test_headers_come_before_text_for_the_whole_account"
        status: pass
      - kind: unit
        ref: "src/application/bringing_everything_down.rs#test_the_budget_ends_a_text_run_and_says_what_it_would_have_needed"
        status: pass
      - kind: unit
        ref: "src/application/bringing_everything_down.rs#test_a_folder_whose_last_chunk_brought_nothing_new_is_asked_once_more_and_then_reported"
        status: pass
    human_judgment: false
  - id: D3
    description: "The list of messages with no text here carries each row's size, so a chunk can be bounded by bytes before a fetch"
    requirement: MAIL-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/bodies.rs#test_a_listed_message_carries_the_size_the_server_gave_it"
        status: pass
    human_judgment: false
  - id: D4
    description: "The text pass asks in chunks, reads the provider's answer per chunk, stops when asked, and says why it ended in this program's words"
    requirement: MAIL-03
    verification:
      - kind: unit
        ref: "src/application/mail_sync.rs#test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#test_a_chunk_stops_before_the_next_message_when_asked_to"
        status: pass
      - kind: unit
        ref: "src/application/mail_sync.rs#test_no_sentence_about_the_server_stopping_carries_a_string_the_server_sent"
        status: pass
    human_judgment: false
  - id: D5
    description: "Whether a provider tolerates a run of chunks of fifty, what it does when it has had enough, and whether the wait's numbers suit it"
    requirement: MAIL-01
    verification: []
    human_judgment: true
    rationale: "No provider has been observed refusing anything; ledger 11 and 72 stay open. The tester's Gmail account meets the runner and the watch in 10-05 and 10-06, unasked, on the first check after the build"

duration: 2h 42m
completed: 2026-09-17
status: complete
---

# Phase 10 Plan 01: What "everything" means, one wait rule, and a text pass that reads the provider's answer Summary

**Two application modules that run without a window and without a server decide what a download
of everything does next for one account and how long to wait after a server said no, the text
pass asks in chunks and ends a chunk after three refusals in a row with a reason in this
program's words, and eleven guard records hold each rule. Nothing a person can reach changed:
no menu, no setting, no sentence spoken from a running program. `WaitBeforeTryingAgain` and
`what_to_do_next` have no caller in the tree; 10-05 and 10-06 are the callers.** Nothing pushed.

## Performance

- **Duration:** about 2 h 42 min from the first read to the merge, of which about 55 minutes were
  guard measurement across nine runs (3,276 s summed from the runners' `timed:` lines, the first
  task 1 run included) and about 29 minutes the two whole gates
- **Started:** 2026-09-17T09:05:03Z
- **Merged:** 2026-09-17T11:46:32Z at `d8e887d6`
- **Tasks:** 3
- **Files modified:** 8 (2 created)

## What landed

**Task 1, the wait rule.** `src/application/trying_again.rs`. `WaitBeforeTryingAgain::new()`
answers 30 s from `next_wait()`, then 1, 2, 4, 8 and 16 minutes, then 30 minutes on the seventh
call and every call after, asserted as one list of nine answers; a hundred calls never pass
`LONGEST_WAIT`; `tell_it_worked()` puts the next answer back to `FIRST_WAIT` and the count to 0;
`how_many_failures_in_a_row()` counts the calls since the last success. It carries no clock and
sleeps nowhere. `what_to_say_before_waiting(wait, failures)` says "The mail server could not be
reached. Trying again in 2 minutes." for one failure and "The mail server could not be reached 4
times in a row. Trying again in 4 minutes." for four, with the wait as seconds under a minute,
minutes from a minute up, and both when it is both, so "in 120 seconds" is never said; the test
forbids a bare count of seconds for every whole-minute wait. The module doc says why
`service::google_api::with_retry` is not reused (it sleeps inside an async call on a runtime
thread for at most a few seconds and decides its own attempts; a wait of up to thirty minutes is
served by the main timer from a value handed back), that the numbers are a decision of 2026-09-17
and not a measurement, that RFC 2177's twenty-nine minutes is why the cap is half an hour, and
that no provider has been observed refusing anything (ledger 64, 65, 72). `grep -c 'pub mod
trying_again' src/application/mod.rs` is 1. `cargo test --lib application::trying_again::` passes
with 9 tests.

**Task 2, the model.** `src/application/bringing_everything_down.rs`. `what_to_do_next(folders,
text, budget, on_screen, reading_allowed)` at `:253` answers `TheNextChunkOfHeaders { folder_id,
path }`, `TheNextChunkOfText { messages, bytes }` or `EverythingIsHere { why }`, with `Why` one of
`ItIsAllHere`, `ReadingIsOff` and `TextStoppedAtTheBudget { kept, would_need }`. Headers before
text for the whole account; among folders, the one on screen first, then the inbox, then the
rest by `common::types::tree_position` (what the folder is for, then its name), read by asking
four times with each answered folder marked done: `[2, 1, 4, 3]` for on-screen Receipts, Inbox,
Sent and Archive, and `[4, 2, 3, 5]` for Drafts, Trash, Receipts and Travel with nothing on
screen. A folder whose `held >= total_on_server` is never asked; one whose
`held_before_the_last_chunk` equals its `held` is `StoppedComingDown` and is not asked again;
`stopped_coming_down` at `:96` is the rule `asking_for_a_whole_folder`'s loop was written with,
moved here, and that loop now calls it. Text only when `reading_allowed`, newest first as the
cache lists it, at most `TEXT_PER_CHUNK_MESSAGES` (50) or `TEXT_PER_CHUNK_BYTES` (16 MiB)
whichever is met first, a single larger message going on its own, and never past
`TextBudget::UpTo(bytes)` counted from `TextStillMissing::kept_bytes`: with 1023 MiB kept and a
1024 MiB budget, two 2 MiB messages end the run as `TextStoppedAtTheBudget { kept: 1023 MiB,
would_need: 1027 MiB }`, and with 1021 MiB kept the last chunk is one message and not two.
`TextBudget::All` never ends a run on bytes, held against `kept_bytes: u64::MAX / 2`.
`HEADERS_PER_CHUNK` at `:56` is `super::mail_sync::INITIAL_FETCH_LIMIT` by name, and a test holds
the equality. The sentences: "Downloading Inbox: 3500 of 12872 messages.", "Inbox is downloaded:
12872 messages on this computer.", "The mail server stopped sending Inbox. 3500 of 12872 are on
this computer. It will be asked again.", "Downloading Inbox stopped: the mail server refused.
3500 of 12872 are on this computer.", "The text of 4000 messages arrived, and nothing failed. The
text of 4000 newer messages is kept, which is the 1 GB you chose; 8872 older messages will be
fetched when they are opened." and "50 folders are downloaded, and the text of 12872 messages is
on this computer.", each held whole by a test; the two clauses of the text report are
`mail_sync::arrived` and `did_not_arrive`, made `pub(crate)` so the two runs word the same fact
one way. `cargo test --lib application::bringing_everything_down::` passes with 23 tests.

`MessageToFetch` gained `size_bytes: u64`, and `messages_with_no_text_here` selects
`COALESCE(m.size_bytes, 0)` beside the uid, a stored size read back as 2048 and a row saved with
none as 0. `cargo test --lib data::message_cache::bodies::` passes with 49 tests where there were
47 on 2026-09-17 at `2ccc69fa`. `INITIAL_FETCH_LIMIT`'s doc comment at `mail_sync.rs:35-48` says
it is the size of one step and no longer the whole of a first look, with the old sentence quoted
and dated. `asking_for_a_whole_folder.rs:43` re-exports `HowMuchIsHere`, so
`wx_app.rs:21676`, which builds one, is untouched; `until_the_whole_folder_is_here` stays for
10-05 to retire with its command. `cargo test --lib application::asking_for_a_whole_folder::`
passes with 7, as before.

**Task 3, the text pass.** `mail_sync::fetch_over_a_mailbox` at `:2029` takes `chunk:
&[MessageToFetch]`, `stop: &dyn Fn() -> bool` and `after_each: &dyn Fn(usize)`, and answers
`Backfilled`. `stop` is asked before each message, the first included, and a yes ends the chunk
as `Ending::Stopped { after }`: a stop that answers yes after two fetches leaves `fetched: 2` and
the server asked twice. Three failures in a row that are not the gate's refusal end the chunk as
`Ending::TheServerStoppedAnswering { after, because }`, `after` being the messages attempted
before the run of failures began: **the scripted mailbox refusing from its fourth fetch on, a
chunk of ten ends `TheServerStoppedAnswering { after: 3, because: Refused }` with `fetched: 3`,
`could_not: 3`, and `asked == 6`**, held by
`test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten`; two
refusals and then an answer do not end it. `REFUSALS_THAT_MEAN_THE_SERVER_HAS_STOPPED` at `:1756`
is 3 with the decision and its date on it. `WhyTheServerStopped::from_the_kind_of` reads the
error's kind and never its text: `Protocol` or `Authentication` is `Refused`, a `Network` error
opening with `imap::THE_SERVER_STOPPED_RESPONDING` (the timeout phrase, made a constant at
`imap.rs:2226` and used by `with_timeout`) is `TimedOut`, any other `Network` error is
`ConnectionLost`, the rest `SomethingElse`, one test per arm. `what_the_fetch_did` words them:
"The mail server stopped answering after 3 messages: it refused. It will be asked again later."
with "the connection was lost", "it took too long to answer" and "something went wrong" as the
other clauses, and "Stopped after 3 messages, as you asked."; a chunk asked of a server whose
refusal text is "refused while fetching a message" produces a sentence without those words. The
reading gate turning off mid-chunk still ends `ReadingWasTurnedOff` at the next message, and its
test passed throughout.

`fetch_the_missing_message_text` at `:1934` keeps its signature for its one caller,
`wx_app.rs:21336`, and delegates to `fetch_all_the_missing_text` at `:1954`, generic over
`Mailbox`, which reads the whole list, hands it over in chunks of `TEXT_PER_CHUNK_MESSAGES` with a
stop that never answers yes and an after-each that says the progress line when
`says_where_it_is` over the whole list says to, and folds the endings: a chunk that went through
is followed by the next, and any other ending ends the run with it, so 120 messages refused from
the fourth end after six asks rather than 120. The eight tests of the entry point that existed
before this plan were green on arrival and stayed green; the test helper `backfill` calls the
generic run. `cargo test --lib application::mail_sync::` passes with 147 tests where there were
135 on 2026-09-17 at `2ccc69fa` by the count check.

## Task commits

| Commit | What |
|---|---|
| `802013ad` | test(10-01): the red half of task 1, seven tests named by module path; two green on arrival, said |
| `da41eda5` | feat(10-01): the rule, the sentence, two records measured |
| `d550eafe` | test(10-01): the red half of task 2, twenty-two tests named and the count check; three green on arrival, said; `HowMuchIsHere` moved and re-exported, the rule left un-repointed until green and why |
| `2c7ea166` | feat(10-01): the decision, the sentences, the size column, the rule re-pointed, the doc comment, four records measured, the eight `bodies.rs` records re-measured |
| `5918a2c4` | test(10-01): the red half of task 3, eight tests named and the count check; four green on arrival, said; the shape landed so the tests compile |
| `b87d2c02` | feat(10-01): the stop, the three-in-a-row, the classifier, the wording, three records measured, the nine `mail_sync.rs` records re-measured with one corrected |
| `d8e887d6` | Merge 10-01 into `main` |

Branch `what-everything-means-and-how-long-to-wait` from `main` at `2ccc69fa`. Not pushed; 100
commits unpushed before the commit that lands this summary, by `git rev-list origin/main..HEAD
--count` on `main` at `d8e887d6`.

## Honest RED and GREEN

One red commit per task, each landing on the branch, each running and failing exactly what it
named and nothing else, which the gate said in `red` mode. Task 1's names seven and says the
first-wait test and the reset test were green on arrival because the stub answers the constant
they expect. Task 2's names twenty-two (twenty-one in the new module and the size test in
`bodies.rs`) and the count check, and says three were green on arrival: the all-here test because
the stub answers "everything is here", the by-name constant because it is a constant, and the
nought-for-an-old-row test because the stub reads nought. Task 3's names eight and the count
check, and says four were green on arrival: two refusals then an answer not ending the chunk,
which was already true; the whole-list run reaching the end of 120 messages, which the fold
already did; any other failure reading as `SomethingElse`, which the stub answered; and no
sentence carrying the server's words, which a stub that words nothing cannot fail. Every one of
those has its red half in a guard record below. The count check is named in the two trailers
where a file a record names gained a test, as `CLAUDE.md` prescribes, because its remedy needs
the green.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/trying_again.rs` | `--lib application::trying_again::` on both task 1 commits |
| `src/application/mod.rs` | `--lib application` whole, on task 1's red and task 2's red, as premise 8 said |
| `src/application/bringing_everything_down.rs` | `--lib application::bringing_everything_down::` on both task 2 commits |
| `src/application/asking_for_a_whole_folder.rs` | `--lib application::asking_for_a_whole_folder::` on both task 2 commits |
| `src/data/message_cache/bodies.rs` | `--lib data::message_cache::bodies::` on both task 2 commits |
| `src/application/mail_sync.rs` | `--lib application::mail_sync::` on task 2's green and both task 3 commits |
| `src/service/protocols/imap.rs` | `--lib service::protocols::imap::` and the coupled `the_list_warning_reads_the_message` on task 3's red |
| `guards/guards.toml` | no scoped target; the whole-tree guards on every commit |

Every commit ran the whole-tree guards. `scripts/check.sh all` on the branch at `b87d2c02`, run
once with its output to a file and the exit status appended to it, never piped: exit 0, 7,918
passed and none failed over 67 result lines, 1,037 s from 11:15:27Z to 11:32:44Z, the release
build included. Forty-six more than the 7,872 on the last whole gate: 9 in `trying_again`, 23 in
`bringing_everything_down`, 2 in `bodies`, 12 in `mail_sync`. Above the 275 s to 654 s band on
`docs/development/measurements.md`, because task 3's green had just rebuilt every target that
reaches `mail_sync` and the test build paid for it. The keyring race (ledger 374) did not appear.
`main`'s hook ran `all` again on the merge: 7,918 and none failed, 716 s from 11:34:36Z to
11:46:32Z.

## Guard records

868 records by the TOML reader before, 877 after; census 802 + 66 before, 802 + 75 after, the
line at `guards/guards.toml:84` moved in each green commit. Nine new, one corrected, every one
measured through `scripts/guards.sh --remeasure` on the whole library with `WIXEN_TEST_THREADS`
untouched, and every new one agreeing with what it said on its first measurement once its tests
were named by module path.

| Record | File | Break | Red | Run |
|---|---|---|---|---|
| the wait before asking a mail server again grows with each failure | `trying_again.rs` | every call answers the first wait | 2 | rebuild 42 s, run 48 s |
| the wait before asking a mail server again never goes past thirty minutes | `trying_again.rs` | the cap dropped | 2 | rebuild 40 s, run 49 s |
| the folder on screen comes down before the inbox | `bringing_everything_down.rs` | the tree's order sorts first | 1 | rebuild 29 s, run 49 s |
| a text budget somebody chose ends the text run | `bringing_everything_down.rs` | `UpTo` reads as no bound | 2 | rebuild 42 s, run 48 s |
| a folder is reported as stopped only when a chunk brought nothing new | `bringing_everything_down.rs` | any chunk at all reports it stopped | 3, two of them the old loop's | rebuild 40 s, run 49 s |
| the list of messages with no text here carries each row's size | `bodies.rs` | `0` selected for every row | 1 | rebuild 36 s, run 48 s |
| a chunk of the text pass stops when it is asked to | `mail_sync.rs` | `stop` asked and ignored | 1 | rebuild 36 s, run 55 s |
| three refusals in a row end the chunk rather than being counted one at a time | `mail_sync.rs` | the bound never met | 2 | rebuild 34 s, run 56 s |
| no sentence about the server stopping carries a string the server sent | `mail_sync.rs` | the chunk ended through `ReadingWasTurnedOff(e.to_string())` | 3 | rebuild 52 s, run 66 s |
| a count and the thing it counts agree in number (corrected) | `caldav.rs` | as before | 22, was 21 | rebuild 29 s, run 51 s |

**The two task 1 records were written with bare test names and the runner said "the test harness
never ran" them.** A library record names its tests by module path; both were corrected and
measured again before their counts were written, and the green commit says so.

**The count check's remedy, run and read, twice.** After task 2's tests it named the eight
records naming `bodies.rs`; all eight agreed as written, 73 s to 88 s each, and their counts now
say 49. After task 3's it named the nine naming `mail_sync.rs`; eight agreed as written, 90 s to
103 s each, and **"a count and the thing it counts agree in number" came out one short**: the
runner reported `test_the_whole_account_is_said_in_one_sentence_with_both_counts` red and
unnamed, because the sentence at the end of a download counts folders and messages through
`how_many` in a file that record had never named, which is the limit `CLAUDE.md` says the count
check cannot see. The record was corrected by hand with the reason dated on it and measured
again: all 22 named went red and nothing else did. Counts written: `trying_again.rs` 9,
`bringing_everything_down.rs` 23, `asking_for_a_whole_folder.rs` 7, `bodies.rs` 49,
`mail_sync.rs` 147 on all twelve records naming it. Four records name the new module by the TOML
reader, my three and the corrected one. No file gained a test after its records were measured;
the count check is green on `main` at `d8e887d6`.

## Premises the tree contradicted

Every command in the plan's eight premises was re-run against `main` at `2ccc69fa` before
anything was built. One drifted:

1. **Premise 5 says `with_retry` has "seven callers, all in `microsoft_graph.rs`".** `grep -rn
   'with_retry(' src --include='*.rs' | grep -v 'fn with_retry'` finds fourteen: seven in
   `google_api.rs` (`:604`, `:646`, `:662`, `:668`, `:698`, `:750`, `:760`) and seven in
   `microsoft_graph.rs`. The plan's `grep -v test` and `head` cut the list. The conclusion holds:
   nothing under the mail vocabulary calls it, and the reason it is not reused is unchanged.

The other seven held exactly: `INITIAL_FETCH_LIMIT` at `:40`, `uids_to_fetch` at `:307`,
`sync_folder` at `:1233`, the two `wx_app.rs` readers at `:21662` and `:21921`; `fetch_over_a_mailbox`
at `:1849` one message at a time with `Ending::ReadingWasTurnedOff` the only early ending;
`RFC822.SIZE` at `imap.rs:2046` and `size_bytes: Some(i64::from(message.size))` at `:579`;
`BODY_CACHE_BUDGET_BYTES` at `bodies.rs:285` and `keep_bodies_within_budget` at
`mail_sync.rs:1465`; `pop3::retrieve` at `:366`; `struct Scripted` at `:2329` and its `impl` at
`:2406`; 135 tests and 9 records on `mail_sync.rs`, 47 and 8 on `bodies.rs`. Two things the plan
could not know: `Scripted` had no way to refuse from the Nth fetch on, so
`refuses_from_the_nth_fetch` was added to it in the red commit as premise 7 said to do rather
than writing a second fixture; and `reader_text::human_size` prints a whole gigabyte as "1 GB",
not "1.0 GB", which the first draft of the budget sentence's test expected.

## Deviations from plan

**1. [Decision] `what_to_do_next` takes `TextStillMissing { messages, kept_bytes }`.** The plan's
signature had `text_missing: &[MessageToFetch]` and `budget: TextBudget`, and its behaviour said
"only while the bytes fetched so far plus the next chunk stay inside the budget". A budget on
bytes kept needs the bytes kept as an input, and the signature had no place for them, so the
list and the count travel together in one struct; `TextBudget` stays `All | UpTo(u64)` as the
plan said. Ledger 512.

**2. [Decision] Bare numbers in the sentences.** The plan's examples read "3,500 of 12,872";
every counted sentence in the tree writes "3500 of 12872" (`how_far_it_has_got`,
`what_the_folder_sync_did`, the status line), nothing in the tree groups thousands, and a second
convention for one module is the drift these sentences exist to avoid. The tests hold the bare
form. If Pratik hears "twelve thousand eight hundred and seventy-two" as harder than the grouped
form, the change is one helper and every counted sentence, not this module. Ledger 512.

**3. [Decision] The whole-list run is `fetch_all_the_missing_text`.** The plan had
`fetch_the_missing_message_text` doing the chunking itself. It takes a real `MailController`, so
a fold done there could not be tested against the scripted mailbox; the fold lives in a generic
function it delegates to, which is the shape the pair already had, and the entry point's
signature is kept as the plan asked. Ledger 512.

**4. [Rule 2] The timeout phrase is a constant.** The plan said `WhyTheServerStopped` is classified
"from the error's kind"; a timeout and a dropped connection are both `Error::Network`, so kind
alone cannot tell them apart. The IMAP layer's own timeout phrase is now
`imap::THE_SERVER_STOPPED_RESPONDING`, used by `with_timeout` and read by the classifier, so the
phrase is this program's and held in one place rather than copied. `imap.rs` gained no test and
none of its 46 records changed. `pop3.rs:521` still writes the same phrase as a literal and is
untouched, because no POP path reaches this text pass.

**5. [Decision] The stopped-coming-down rule was re-pointed at green, not red.** The plan said to
move the rule in the red commit; the red stub answers `false`, and the old loop's existing test
of a server that stops would have run forever under it, inside the commit hook. `HowMuchIsHere`
moved and was re-exported in the red; the loop's `use` of the rule moved in the green, and the
red commit says so. Observation 624 in the skill log.

**6. [Rule 1 - Lint] `field_reassign_with_default` three times** in the new `mail_sync` tests,
fixed the way clippy asked before the red commit went through, and a `dead_code` refusal of the
clause helper in the red, which was held back to the green where its caller arrived. Neither is a
departure from what the plan asked for.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; the only `sed` and `awk` in the session read files and logs. Commit messages were written
to the scratchpad and passed with `-F`; `cargo fmt` ran before each commit. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No
em-dash in any file this plan wrote, measured with `grep -c` for the byte sequence: zero on each.
`Cargo.toml` and `Cargo.lock` untouched; no crate added. No AI attribution anywhere. No changelog
entry, because nothing user-visible landed, as the plan's verification says. The version stays
`1.0.0-alpha.1`. The tester's profile was not read; no binary was started.

## Threat register

T-10-01: chunks bounded by count and bytes, three refusals end a chunk, the wait grows to thirty
minutes; ledger 11 and 72 stay open because no provider has been observed. T-10-02: the
once-more-then-reported rule moved and held by a record that reddens the old loop's tests too.
T-10-03: the `tracing::warn!` line is unchanged, uid and reason only, and the new record holds the
spoken side: no sentence carries the server's text. T-10-04: the budget is an input and a record
breaks the `UpTo` arm. T-10-SC: no crate added. No new surface outside the register.

## Ledger

`.planning/WINDOWS.md` 512 written by hand, both halves, no backslash; 511 before, 512 after; 483
open before, 484 after.

| id | kind | what |
|---|---|---|
| 512 | deviation | the five departures above, each with its reason, so 10-05 reads the signatures it will call from the tree and not from the plan |

## Known stubs

**Two things in the tree are called by nothing, on purpose, and this is the plan that says so.**
`trying_again::WaitBeforeTryingAgain` and `bringing_everything_down::what_to_do_next` are
reached by their own tests and by no other path: `grep -rn 'WaitBeforeTryingAgain\|what_to_do_next('
src --include='*.rs'` outside the two modules finds one line, the comment in `mail_sync.rs`
pointing at them. 10-05's runner calls both and `fetch_over_a_mailbox`; 10-06's watch calls
`WaitBeforeTryingAgain`. Until then they are decisions with tests and no consequence, which is
what the plan's objective asked for and what guardrail 1 says to name rather than leave implied.
`fetch_over_a_mailbox`'s chunk shape is reached today through `fetch_all_the_missing_text`, which
`wx_app.rs:21336` calls, so the three-in-a-row ending and the classifier run in the program now
whenever Fetch Missing Message Text is used; the stop is passed as never-yes by that path.

## What the tester's account could settle, and what a loopback server proved

Nothing in this plan is claimed against a real provider. The scripted mailbox proved the chunk
ending after six asks, the stop, the fold, the classifier's four arms and the wording; values
proved the order, the bounds, the budget and the wait. What Gmail does with chunks of fifty and
with a client that waits thirty seconds and then a minute is ledger 11, 64, 65 and 72, and the
first check after 10-05's build is where it will be found out.

## Not done here, on purpose

Nothing reaches the window: that is 10-05 and 10-06. No `MAIL` requirement is ticked, on the
phase's rule that the last plan reads each clause; the clauses this plan gives a named test for
are MAIL-01's first `[D]` line (`test_the_folder_on_screen_comes_first_then_the_inbox_then_the_tree_order`
and `test_a_folder_whose_last_chunk_brought_nothing_new_is_asked_once_more_and_then_reported`),
MAIL-03's first `[D]` line (`test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten`
and `test_a_listed_message_carries_the_size_the_server_gave_it`) and MAIL-04's second `[D]` line
(`test_each_failure_doubles_the_wait_until_the_cap` and
`test_a_success_puts_the_wait_back_to_the_start`). Roadmap criterion 1's "one decision" clause and
criterion 4's "bounded" clause close structurally and are read by 10-07. #20 and #23 are
commented after this summary lands, quoting `d8e887d6`, and neither is closed. Nothing pushed.

## For 10-05 and 10-06

- `what_to_do_next(&[FolderHere], TextStillMissing { messages, kept_bytes }, TextBudget,
  on_screen: Option<i64>, reading_allowed: bool)`; build `FolderHere` from `FolderSync::held` and
  `total_on_server` with `held_before_the_last_chunk` from the previous chunk's `held`, and
  `TextStillMissing` from `messages_with_no_text_here` and the cache's stored text size.
- `fetch_over_a_mailbox(server, cache, &chunk, &stop, &after_each) -> Backfilled`; one chunk per
  call, `Ending::Stopped` and `TheServerStoppedAnswering` are the two endings to word with
  `what_the_fetch_did` or the module's own sentences.
- `WaitBeforeTryingAgain`: `next_wait()` on a failure, `tell_it_worked()` on a success,
  `how_many_failures_in_a_row()` for a bound of the runner's own; the timer serves the
  `Duration`.
- `how_far_the_download_has_got`, `what_the_folder_download_came_to`,
  `what_the_text_download_came_to` and `what_a_whole_account_came_to` are the sentences; the
  progress line is the one 10-04's level gates.

## Self-Check: PASSED

`src/application/trying_again.rs` and `src/application/bringing_everything_down.rs` exist;
`grep -c 'pub mod trying_again' src/application/mod.rs` is 1; `HEADERS_PER_CHUNK` is
`super::mail_sync::INITIAL_FETCH_LIMIT`; `guards/guards.toml` holds 877 records by the TOML
reader and the count check passes. Commits `802013ad`, `da41eda5`, `d550eafe`, `2c7ea166`,
`5918a2c4`, `b87d2c02` and `d8e887d6` are in `git log --oneline` on `main`.

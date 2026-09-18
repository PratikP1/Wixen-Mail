---
phase: 11-reading-and-the-list
plan: 04
subsystem: logging, version, config, the mail check, the download, the announcement queue, guards, pages
tags: [log-level, default-follows-the-version, tracing, secrets-guard, alpha-page, measurements, ledger, git-hooks]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-04's channel sorting in spawn_mail_sync, 10-05's chunk arms in start_the_download, 10-06's watch lines; the reading shape of tests/progress_is_shown_and_results_are_said.rs"
  - phase: 11-reading-and-the-list
    provides: "11-03 merged at 70f4737b; the planner's 1ae63359 landed on main while this branch was on its gate, and the merge lands on it"
provides:
  - "common::version::is_alpha_or_beta: true for alpha and beta, with or without a build identifier; false for rc, a release and anything parse refuses; 27 tests"
  - "common::logging::default_level_for, LogLevel::as_stored, filter_for: one rule for the default (debug under alpha and beta, info otherwise), the stored word held to a round trip with parse, the filter naming this crate alone; LoggerConfig::default() reads the rule; init_logging builds its EnvFilter from filter_for; to_tracing_level retired with its test; 10 tests, 2 records"
  - "data::config::default_log_level: AppConfig's default and the field's serde default both from the rule; the default test rewritten in place to the rule's answer; 68 tests"
  - "wx_app.rs: each folder of a check at info with the account, the folder and the check's own sentence; each chunk of headers and of text at debug with the counts; the refused chunk's warn naming this program's clause before the server's words; the text download's stop at info with the clause and the count; the settings save at info naming the log level and the while-fetching level; 199 tests before and after, 30 to 33 info calls, 2 to 4 debug"
  - "accessibility/announcements.rs: muted content written by its length at info, a repeat at debug and a capacity drop at info, each with the topic and never the words; 25 tests, 1 record"
  - "tests/the_log_carries_what_a_report_needs.rs: five readings, five companions, the lexical guard over every tracing:: call under src with an empty exception table, and the guard's two parsing tests; 13 tests, 2 records name it"
  - "tests/the_numbers_the_targets_ask_for.rs: the level pinned through WIXEN_MEASUREMENT_LOG_LEVEL, held to a word the program reads; the log's size after two minutes printed as a row; a start refused while any wixen-mail.exe is running, with the reason; 16 tests, no test added"
  - "guards/guards.toml: 918 records, census 798 + 120; four new records measured, three re-measured at 27"
  - "docs/ALPHA_TESTING.md, docs/privacy.md, docs/changelog.md: the table of the five levels in the Advanced tab's words, the two sentences, the paragraph saying the cost is owed and why, the privacy sentence dated, the entry naming #71 and #64"
  - ".planning/WINDOWS.md: 535 unrun-verify (the tester's next report), 536 deviation (the which-checks fixture acting on the real repository from a hook in a linked worktree), 537 unrun-verify (the two size rows)"
affects: [11-04.1 and every later plan, whose executor should not commit from a linked worktree until ledger 536 is fixed; 11-12, which reads LIST-02's lines and the pages; whoever takes the two size rows]

actuals:
  tokens: 21073
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A default that follows the version is one rule in one place, read by every default that used to hold a literal, so the cut that moves the version moves the default without a hand edit"
    - "A guard over log calls is lexical and says so: it reads the arguments of each call for named identifiers as values, not as words in the message's prose, and what it proves is that no call spells them; the rule it serves is kept by reading each new line"
    - "A measurement harness that starts the product looks for a running copy before it starts anything, because a second copy hands itself to the first and makes it speak"

key-files:
  created:
    - tests/the_log_carries_what_a_report_needs.rs
  modified:
    - src/common/version.rs
    - src/common/logging.rs
    - src/data/config.rs
    - src/presentation/wx_app.rs
    - src/presentation/accessibility/announcements.rs
    - tests/the_numbers_the_targets_ask_for.rs
    - guards/guards.toml
    - docs/ALPHA_TESTING.md
    - docs/privacy.md
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The held-back, repeat and capacity lines are written inside the queue's push, where each decision is made, rather than in Accessibility::announce_content beside it: the plan named accessibility.rs, the mute is inside the queue (its own premise 3 said so), and a line beside the caller would have needed the check twice"
  - "The per-folder line asks for the check's sentence a second time rather than binding it above the step, because 10-04's reading finds the sentence's first call and reads the send before it, and a binding made it read the renumbering send as the step; rewriting 10-04's reading would have touched a target two records name"
  - "The refused chunk's existing warn line is kept at warn and gains the clause, rather than a second line at info: warn is above info and the default reaches it"
  - "The filter's word is the stored one, lowercase, because tracing's Level::as_str answers uppercase; to_tracing_level then had no caller and was removed with its test rather than kept dead"
  - "The two size rows are gated, not taken: the measurement starts a second copy, one copy runs at a time, the tester's copy was open on his account through the session, and a run started the moment it closed would take the single-instance slot from his restart"
  - "The repository config was restored by hand when the which-checks fixture left it bare, because nothing could run until it was; main's stray commit was left alone and the planner undid it"

patterns-established:
  - "When the planner shares the working tree and its uncommitted planning files make the whole-tree guards red, wait for its commit rather than naming its failures in a red; the red was refused once for four planning-count tests that were not this plan's"

requirements-completed: []

coverage:
  - id: D1
    description: "One rule, read from the version the build carries, gives the default; a stored level is kept; the filter names this crate alone"
    requirement: LIST-02
    verification:
      - kind: unit
        ref: "src/common/logging.rs#test_the_default_level_is_debug_under_alpha_and_beta_and_info_otherwise"
        status: pass
      - kind: unit
        ref: "src/common/logging.rs#test_the_filter_names_this_crate_at_the_level_and_nothing_else"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_a_version_carrying_alpha_or_beta_is_a_testing_round"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_app_config_defaults_complete"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'the log's default is debug under alpha and beta, not info for every build' and 'the log filter names this crate alone, not a library beside it', measured 2026-09-18"
        status: pass
    human_judgment: false
  - id: D2
    description: "The lines a report needs are written at the level the default names, none naming a subject, a body, a password or a token; a lexical guard holds every log call to spelling none of the five identifiers"
    requirement: LIST-02
    verification:
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_each_folder_of_a_check_is_written_at_info"
        status: pass
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_each_chunk_of_the_download_is_written_at_debug"
        status: pass
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_what_the_server_answered_is_written_at_info_or_above"
        status: pass
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_the_settings_save_is_written_at_info"
        status: pass
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_what_is_held_back_from_speech_is_written_and_never_the_words"
        status: pass
      - kind: integration
        ref: "tests/the_log_carries_what_a_report_needs.rs#test_no_log_call_in_the_tree_spells_a_secret_or_a_body"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'each folder of a check is written to the log, not left to the status bar alone' and 'content held back by mute is written to the log by its length, not dropped in silence', measured 2026-09-18"
        status: pass
    human_judgment: false
  - id: D3
    description: "The alpha page's table of the five levels, why the alpha default is Debug, that a profile keeps its level and where to move it; the privacy sentence"
    requirement: LIST-02
    verification:
      - kind: command
        ref: "grep -c '^| Error \\|^| Warn \\|^| Info \\|^| Debug \\|^| Trace ' docs/ALPHA_TESTING.md -> 5; cargo test --test house_style -> 74 passed; cargo test --test every_number_carries_its_command_and_its_date -> 25 passed"
        status: pass
    human_judgment: false
  - id: D3-rows
    description: "The log's size after the harness's two-minute start under info and under debug, two rows on the measurements page and the sizes on the guide"
    requirement: LIST-02
    verification: []
    human_judgment: true
    rationale: "Not taken: the measurement starts a second copy of the release binary, one copy runs at a time, and the tester's copy was open on his account through the session (ledger 537); the harness now refuses with the reason and the guide says the cost is owed"
  - id: S1
    description: "Whether these are the lines that make the tester's next problem diagnosable, and what a day at Debug on his real account costs on his disk"
    requirement: LIST-02
    verification: []
    human_judgment: true
    rationale: "Ledger 535; nobody has written a report from a log at this level"

duration: 144min
completed: 2026-09-18
status: complete
---

# Phase 11 Plan 04: The log level follows the build, and the log carries what a report needs Summary

**The log's default is one rule read from the version the build carries, `debug` while it says
alpha or beta and `info` from an rc on, and both places that held a literal now ask it; a
stored level still wins. The lines a report needs are written at the level that default
names: each check's result per folder, each chunk of the download and what the server
answered in this program's words, the settings save with the two levels a report reads by,
and what the queue held back from speech by its length; none carries a subject, a body, a
password or a token, and a lexical guard over every `tracing::` call under `src/` holds each
call to spelling none of five identifiers as a value, which is what a reading of the text can
prove. The alpha page has the table. Two things are owed: the two size rows, because the
measurement could not start while the tester's copy was open, and a report written from a log
at this level, which only the tester can write. Found on the way and not fixed: a shell
suite's scratch-repository fixture acts on the real repository when the gate runs it from a
hook in a linked worktree, and it did, during the planner's commit this afternoon.**

## Performance

- **Duration:** 144 min from the branch at 16:03Z to the merge at 18:26:53Z, of which about
  47 min was waiting for the planner: its uncommitted `11-04.1-PLAN.md` on the shared
  working tree made the four planning-count guards red from 16:06Z, so the first red commit
  was refused for failures that were not this plan's, and nothing could be committed until
  its files and counts agreed at 16:55Z (the planner's own commit went astray in between,
  below). About 11 min 30 s was guard measurement in three foreground runs (8 min 19 s for
  five records, 2 min 26 s for one re-measured, 33 s for two on the target); about 14 min the
  six hook runs on the branch (113 s, 125 s, 81 s, 211 s, 110 s, 93 s); about 13 min three
  whole gates on the branch (283 s green at `4de64af5`; 228 s red at `92c4a796` on the keyring
  race, ledger 374; 286 s green at `92c4a796`); 288 s `main`'s hook at the merge; 57 s the
  release build; one measurement attempt of 60 s that produced nothing.
- **Started:** 2026-09-18T16:03:00Z (the plan read from about 15:56Z)
- **Merged:** 2026-09-18T18:26:53Z at `03513fd0`
- **Tasks:** 3
- **Files modified:** 12, one created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat eb5d8517..92c4a796` (T-11-SC)
- **Actuals:** `tokens: 21073` is `git diff eb5d8517..92c4a796 | wc -c`, 84,294 characters
  over four, the branch's own diff against the commit it left `main` at; the estimate's
  `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.23. The merge's diff
  against `eb5d8517` is 454,858 characters because it carries the planner's `1ae63359`.

## What landed

**Task 1.** `version::is_alpha_or_beta(version)` answers through `parse`, so `1.0.0-alpha.1`,
`1.0.0-beta.3` and `1.0.0-alpha.1+42.g59c5b6a4` are true and `1.0.0-rc.1`, `1.0.0`, `0.125.1`
and anything unreadable are false; 25 to 27 tests. `logging::default_level_for(version)` is
`Debug` when that is true and `Info` otherwise, with the decision and its date in its doc;
`LogLevel::as_stored` answers the five lowercase words and a test holds it to a round trip
with `parse`; `filter_for(level)` answers `wixen_mail=<word>` and a test holds it to starting
with `wixen_mail=` and holding no comma; `LoggerConfig::default().level` reads the rule;
`init_logging` builds its `EnvFilter` from `filter_for`, the environment's override unchanged.
`to_tracing_level` lost its last caller and went with its test, so `logging.rs` is at 10
tests from 7. `config::default_log_level()` answers the rule's word; `AppConfig::default()`
and `#[serde(default = "default_log_level")]` on the field both use it; the default test
compares `config.log_level` with the rule's answer for `version::current()`, with a comment
saying which word each build reads; 68 tests. `main.rs` unchanged: it read the stored level
first and fell back to `LoggerConfig::default()` already.

**Task 2.** In `spawn_mail_sync`'s folder arm, after the step goes out: `tracing::info!`
naming the folder, the account and `what_the_folder_sync_did(&result)` asked a second time.
In `start_the_download`: `tracing::debug!` after each chunk of headers with the folder, the
account, `done.held` and `done.total_on_server`; `tracing::debug!` after each chunk of text
with the account, `text.fetched`, `to_fetch` and `text.could_not`; the refused chunk's
existing `tracing::warn!` now names `WhyTheServerStopped::from_the_kind_of(&e).as_a_clause()`
before the server's `{e}`; the text download's `TheServerStoppedAnswering { after, because }`
arm destructured and written at info with `because.as_a_clause()` and `after`. In
`handle_settings`, after `send_status(tx, rt, "Settings saved")`: `tracing::info!` naming
`mgr.app_config().log_level` and `.announce_while_fetching`, nothing else. In the queue's
`push`: muted content at info by `announcement.text.len()`; a repeat at debug with
`topic = ?announcement.topic` and the length; a capacity drop at info with the topic and
priority of the line dropped, in both arms, and `CAPACITY`. Each line was read for content:
they carry accounts, folders, counts, levels and clauses. The target holds each line's
presence and level with a companion planting its absence or its wrong level, holds the
queue's three lines to `.text` appearing only as `.text.len()`, and walks every `.rs` under
`src/` with test modules cut for a `tracing::` call whose values name one of the five.

**The guard's exception table is empty.** `THE_LINES_STILL_TO_FIX: [(&str, &str, &str); 0]
= []`. The guard found nothing on arrival, so no line needed fixing; the two lines the plan's
premise-4 grep returned (`accounts.rs:186` and `:243`) say the word "password" in their
prose and interpolate `account_id` and `e`, which the guard reads as values and prose reads
as prose, and the test `test_the_guard_reads_values_and_not_the_prose_around_them` holds
that distinction on planted snippets. The guard also holds that a stale exception row is a
failure, so the table cannot go quietly out of date.

**Task 3.** The alpha page, under How to report something: a subsection with the table of
the five levels in the Advanced tab's words (Error, Warn, Info, Debug, Trace, in that order,
`grep -c` 5), what each adds, the sentence that the default follows the build and why, the
sentence that a profile keeps its level, that the first tester's held Info on 2026-09-18 and
where to move it, and a paragraph saying the cost on disk is not measured yet, which
commands measure it, and why the run of 2026-09-18 was refused. The privacy page's Logging
section gains one dated sentence saying the alpha default is Debug, what Debug and Info now
add, and that the rule is unchanged at every level. The changelog entry under `[Unreleased]`,
Changed, naming #71 and #64, with the known limitations. The harness: `WIXEN_MEASUREMENT_LOG_LEVEL`
read through `LogLevel::parse` and refused when it is not a word the program reads, the
log's byte count printed as a fourth row per run, and `Started::against` refusing to start
while `every_process()` lists a `wixen-mail.exe`, with the reason; no test added, three
records name the file, the level pin is glue over the tested `parse`.

## Honest RED and GREEN

Three reds, three greens, on branch `the-log-level-follows-the-build` from `main` at
`eb5d8517`.

`18f93bb2`, task 1's red, named six from cargo's own lines under stubs that answer `false`,
`Info`, an empty word and an empty filter: `common::version::tests::test_a_version_carrying_alpha_or_beta_is_a_testing_round`,
`common::logging::tests::test_the_default_level_is_debug_under_alpha_and_beta_and_info_otherwise`,
`common::logging::tests::test_every_level_survives_being_stored_and_read_back`,
`common::logging::tests::test_the_filter_names_this_crate_at_the_level_and_nothing_else`,
`data::config::tests::test_app_config_defaults_complete` (red because `as_stored` answered
nothing, not because the literal was still there: the plan expected the literal to make it
red, and under the stub `"info" == "info"` would have been green, so the stub for the word is
what made it honest), and `test_every_guard_record_says_how_many_tests_the_files_it_names_held`
bare, because `version.rs` went from 25 to 27 under three records. Two green on arrival and
said: an rc, a release or an unreadable version not being a testing round (the stub says
false for everything) and `LoggerConfig::default().level` being the rule's answer (both Info
until the green). The gate in `red` mode held it to exactly those six, 113 s, on the second
try: the first, at 16:06Z, was refused for four planning-count guards the planner's
uncommitted files had made red, and the working tree was restored to the red from the
index with `git checkout` after the green had been written meanwhile, then the green redone
by hand.

`7058f21d`, task 1's green, 125 s: the rule, the word, the filter, the two defaults and the
serde default; `filter_for` first used `Level::as_str()`, which answers `DEBUG`, and the
test caught it, so the filter uses the stored word and `to_tracing_level` went. The remedy
the red printed was run in the foreground before this commit; the hook printed none.

`9b82ed16`, task 2's red, 81 s: the target committed with ten red bare, the five readings
and the five companions (whose anchors are the lines themselves), each for the intended
reason as quoted from the run: "writes nothing to the log", "written nowhere", "does not
name as_a_clause()". Three green on arrival and said: the tree-wide guard, and its two
parsing tests. No file a record names gained a test.

`d79f67e4`, task 2's green, 211 s: the lines. Between the red and the green the reading of
the folder arm was tightened twice, once because rustfmt split the binding it anchored on
and once because the binding itself broke 10-04's reading (below), and the muted reading's
two checks were reordered so the words check complains before the length check; the
companions caught both.

`4de64af5`, task 3, 110 s, and `92c4a796`, the ledger correction, 93 s: documents and the
harness, which `CLAUDE.md` lists among the exceptions to test-first, no test added.

Under the TDD gate's own terms, `test(11-04)` precedes `feat(11-04)` twice. Task 3 has no
red and the plan said it would not.

## Guard records

914 by the TOML reader before, 918 after: four new, none retired, three re-measured. Census
798 + 116 before, 798 + 120 after.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| the log's default is debug under alpha and beta, not info for every build (new) | `logging.rs` | `if false && is_alpha_or_beta(version)` | 1, "the one test named went red, and nothing else did" | rebuild 33 s, run 51 s |
| the log filter names this crate alone, not a library beside it (new, measured twice) | `logging.rs` | `wixen_mail={},async_imap=debug` | first draft named 1 and the runner answered "1 test went red that this record does not name: `service::outward::completeness::test_every_module_that_names_a_way_out_is_on_one_of_these_lists`"; rewritten to name 2 and measured again, "all 2 tests named went red, and nothing else did" | 33 s and 51 s; then 23 s and 48 s |
| each folder of a check is written to the log, not left to the status bar alone (new, `suite` the target) | `wx_app.rs` | the six-line call replaced by a comment | 2, the reading and its companion, which asserts its anchor | rebuild 12 s, run 2 s |
| content held back by mute is written to the log by its length, not dropped in silence (new, `suite` the target) | `announcements.rs` | the four-line call replaced by a comment | 2, the reading and its companion | rebuild 15 s, run 2 s |
| a build identifier is not part of the version (re-measured at 27) | `version.rs` | unchanged | 4, "all 4 tests named went red, and nothing else did" | rebuild 34 s, run 51 s |
| the public channel never offers a prerelease (re-measured at 27) | `version.rs` | unchanged | 1 | rebuild 33 s, run 51 s |
| a prerelease stays below the release it stages (re-measured at 27) | `version.rs` | unchanged | 4 | rebuild 26 s, run 49 s |

The filter record is the finding: a library named in the filter is caught twice, by the
filter test and by `service::outward`'s census, which reads every module for the name of a
crate that reaches the network and takes a module spelling `async_imap` as a module naming a
way out. The first draft was a prediction, the runner's answer is the record, and the
record's comment says so. Counts written: `logging.rs` 10 on two records, `outward.rs` 38 on
one, `version.rs` 27 on three, `wx_app.rs` 199 and the target 13 on one, `announcements.rs`
25 and the target 13 on one. The count check printed its remedy once, at task 1's red, and
it was run in the foreground and read before the green; it printed nothing at any other
commit. `wx_app.rs` is at 199 before and after, quoted by `cargo test --lib presentation::wx_app::`
at task 2's green; 75 records name it now, one more than the 74 read at the start.

## What the tree contradicted

Every command in the plan's seven premises was re-run against `main` at `eb5d8517` before
the branch. All held but these:

1. **The counts.** `warn` 216, not 215 (11-03 added one); `wx_app.rs` under 74 records, not
   73 (11-03 added one). At the merge: `error` 59, `warn` 216, `info` 83, `debug` 15, `trace`
   0, by `grep -rn 'tracing::<level>!' src --include='*.rs' | wc -l` on 2026-09-18.
2. **The premise-4 grep finds two lines and neither is a finding.** `accounts.rs:186` and
   `:243` say "password" in prose and interpolate an account id and an error. The plan said
   any line found is a bug fixed before the guard lands; a grep for a word is not the guard,
   and the guard reads values. The exception table is empty on arrival.
3. **The config test could not be red for the reason the plan gave.** Under a stub answering
   `Info`, the rule's word and the literal agree; it was red because `as_stored` was stubbed
   to answer nothing. Said in the red commit.
4. **`Level::as_str()` answers uppercase**, so the filter the plan's key link describes as
   `wixen_mail=<level>` is spelled with the stored word.
5. **The mute is in the queue, not in `accessibility.rs`.** The plan's premise 3 said so and
   its behaviour still placed the line "in `Accessibility`"; the lines went where the
   decisions are, and the reading reads `announcements.rs`.
6. **A binding above the step breaks 10-04's reading.** `test_the_mail_checks_lines_are_steps_and_what_arrived_goes_out_once_after_the_loop`
   finds `what_the_folder_sync_did(` and reads back to the nearest `UIUpdate::`; with the
   sentence bound before the step, that was `FolderWasRenumbered`. The step stayed as it was
   and the log line asks for the sentence again, with the reason in a comment.
7. **The measurement cannot run while a copy of Wixen Mail is open**, whatever profile the
   copy is on: `application::running::claim` is one named mutex, a second start hands what it
   was given to the first and stops, and the first is raised and says "Wixen Mail is already
   running, and this is it". The plan's harness premise did not have it; the harness header
   and the guide now do.

## Deviations from plan

**1. [Rule 3 - Blocking] The repository's config was rewritten under the session, and
restored by hand.** At 16:52:54Z `.git/config` gained `bare = true`, a `hooksPath` under a
deleted temp folder and a `[user]` of "the suite", and every git command answered "must be
run in a work tree". `main`'s reflog showed `b4a4cc81 "the manifest before the bump"`,
authored by the planner at 16:50:31Z and committed by "the suite", carrying the planner's
staged planning files and a four-line `Cargo.toml` at `0.99.0`. The cause was measured in a
scratch repository with a hook printing its environment: a commit-msg hook in the main
checkout sees no `GIT_DIR`; in a linked worktree it sees an absolute `GIT_DIR` and
`GIT_INDEX_FILE`. `scripts/which-checks.test.sh:247-262` builds its scratch repository with
`git -C "$scratch" init`, `config`, `add` and `commit`, and `-C` changes directory without
unsetting those, so from a linked worktree (the planner's commit came through
`wixen-mail-sweep`, which has `main` checked out; the main checkout's HEAD reflog has no
record of the commit) every fixture command acted on the real repository. `git config` set
`core.bare` back, put `hooksPath` back to `.githooks` and removed the user; the index, my
staged red and the working tree were intact. The stray commit was left to whoever owns
`main`, and the planner undid it (`main@{1}`, "planner: undo the suite's stray commit made
through the hook from a linked worktree") and landed `1ae63359` while this branch was on its
gate. Ledger 536; the fixture is unchanged and will do it again from the next linked
worktree.

**2. [Decision] The two size rows are gated, not taken.** The one attempt at 17:37Z handed
itself to the tester's copy (process 12648, "INBOX, 14400 unread"), which was raised and made
to speak, and the copy stayed open through the session. A run started the moment it closed
would take the single-instance slot from his restart, which is what closing usually
precedes. The harness refuses to start while a copy runs, the guide says the cost is owed,
ledger 537 says how to take it. The plan's must-have says measured; this summary says it is
not.

**3. [Rule 2 - Correctness] The harness looks before it starts.** A start that hands itself
over touches a copy somebody is using; `Started::against` now lists processes first and
refuses with the reason, and `wait_for_usable` reads "Handed this start over" in the log as
a second net.

**4. [Decision] The lines in the queue, not beside it**, above.

**5. [Decision] The per-folder line asks for the sentence twice**, above.

**6. [Rule 1 - Bug] `filter_for` spelled the level in capitals** on the first green attempt;
the test caught it before the commit.

**7. [Decision] `to_tracing_level` removed with its test** rather than kept dead;
`logging.rs` 11 to 10 tests, no record names it.

**8. [Decision] The refused chunk keeps its warn**, gaining the clause; the plan wanted info
and warn is above it.

**9. [Decision] The word "password" in a message's prose is not a finding**, and the guard's
parsing test holds that it is not, so the guard cannot be satisfied by rewording a message
and cannot be tripped by an honest one.

Everything else executed as written. **No scripted edit touched a tracked file: the
exception set for this plan is zero, and it stayed there.** Every tracked file was changed
by Read then Edit or Write; `cargo fmt` ran before each Rust commit; `scripts/guards.sh
--remeasure` wrote the counts on `guards/guards.toml`; the new target was drafted in the
scratchpad and copied into `tests/` as a new untracked file, which no rule names, and every
later change to it was Read then Edit; `git checkout -- <the three files>` was used once to
put the red back into the working tree from the index; `git config` was used to restore the
repository's own config, which is not a tracked file. The only `sed`, `awk`, `grep`, `tr`
and `python` in the session read files, logs and the records file. Commit messages were
written to the scratchpad and passed with `-F`. Carriage returns measured with
`tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No em dash in
any file this plan wrote, measured by `grep -c` for the byte sequence: zero; none of the six
words, measured by `grep -ciE`: zero. `git commit` and `git merge`, never `gsd-tools query
commit`; never `--no-verify`; `check.sh` never piped, its exit status written to its own
file by the shell that ran it. No AI attribution in any commit. `Cargo.toml` and
`Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's profile was not
read, and the installed binary was not started; the release binary was started once,
through the harness, against a temporary profile, and handed itself to the tester's copy,
which is deviation 2 and is said plainly there. `WIXEN_TEST_THREADS` untouched. Nothing
pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/common/version.rs`, `src/common/logging.rs`, `src/data/config.rs` | `--lib common::version`, `--lib common::logging`, `--lib data::config` on task 1's red and green, and the whole-tree guards |
| `tests/the_log_carries_what_a_report_needs.rs` | itself on task 2's red, and through its two records' coupling from `d79f67e4` on |
| `src/presentation/wx_app.rs`, `src/presentation/accessibility/announcements.rs` | their `--lib` filters and the coupled targets `wx_app.rs`'s records name, on task 2's green |
| `tests/the_numbers_the_targets_ask_for.rs` | itself, on task 3's commit |
| `guards/guards.toml`, `docs/*.md`, `.planning/WINDOWS.md` | the whole-tree guards on every commit; the document-reading targets on the ledger-only commit |

`scripts/check.sh all` ran three times on the branch, output to a file with the exit status
written by the same shell. At `4de64af5`, the last code commit: exit 0, 8,058 passed and none
failed over 76 result lines, 283 s from 18:05:22Z to 18:10:05Z, the release build included;
one more result line than 11-03's 75, the new target, and 18 more tests: 13 in the target, 2
in `version.rs`, 3 net in `logging.rs`. At `92c4a796`, after the ledger correction: exit 101
at 18:16:47Z, 8,057 passed and 1 failed,
`test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent` on "No default store
has been set", which is ledger 374's keyring race in a target this plan does not touch; then
exit 0, 8,058 and none failed, 286 s from 18:17:02Z to 18:21:48Z. `main`'s hook at the
merge, 288 s from 18:22:05Z to 18:26:53Z, the same. The tester's copy and NVDA were open on
the desktop throughout; no live-window key test went red on any run.

## Threat register

T-11-15 mitigated: every new line names accounts, folders, counts, levels and clauses, each
read by hand, and the guard holds every call in the tree to spelling none of the five;
what the guard cannot see is in its doc and above. T-11-16 mitigated: `filter_for` names one
target and its test refuses a comma; the record's break showed a second target is caught by
the outward census as well. T-11-17 mitigated: chunk lines at debug; the cost is not
measured and the page says so rather than guessing. T-11-18 mitigated: `is_alpha_or_beta`
answers false for anything `parse` refuses, with tests. T-11-SC: nothing added. New surface
outside the register: none in the product; the harness gained a process listing it already
had a helper for.

## Ledger

`.planning/WINDOWS.md` written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after each edit, 16 passed. 534 before,
537 after; 501 open before, 504 after; 33 fixed, unchanged.

| id | kind | what |
|---|---|---|
| 535 | unrun-verify | what only the tester's next report settles for #71: whether the per-folder, per-chunk, settings-save and held-back lines are the ones that make his problem diagnosable, and what a day at Debug costs on his disk |
| 536 | deviation | the which-checks suite's scratch-repository fixture acting on the real repository from a hook in a linked worktree; the config restored, the stray commit undone by the planner, the fixture unchanged |
| 537 | unrun-verify | the two size rows, not taken because the measurement starts a second copy and the tester's was open; the harness refuses with the reason; how to take them |

## The issue

`gh issue comment 71` from the repository root after the merge, at
`issues/71#issuecomment-5734421727`, the plan's sentence with the merge commit, the cost
said as owed rather than measured and why, and the dialog left to #64. The issue stays open
for #64's half. Commenting is not a publish; nothing was pushed.

## Known stubs

None. `default_level_for` has two non-test callers, `LoggerConfig::default()` and
`config::default_log_level`; `filter_for` one, `init_logging`; `is_alpha_or_beta` one,
`default_level_for`; `as_stored` two, `default_log_level` and `filter_for`, and the harness.
Every log line added sits on a path the program takes: the check's folder loop, the
download's arms, the settings save, the queue's push. `THE_LINES_STILL_TO_FIX` is an empty
table read by the guard, and the guard's test holds that a row in it must still match a
line, so it cannot go stale.

## Not done here, on purpose

The two size rows (ledger 537), with the commands on the harness's header and the guide. A
report from the tester's machine (ledger 535). The fix to `scripts/which-checks.test.sh`
(ledger 536): clear `GIT_DIR`, `GIT_INDEX_FILE`, `GIT_WORK_TREE` and `GIT_PREFIX` before the
fixture's first `git`, with a case that exports an absolute `GIT_DIR` and asserts the real
repository did not move; outside this plan's files, and a suite change with its own red.
Until it is fixed, no executor should commit from a linked worktree. LIST-02's box is
unticked, with a paragraph on its lines saying the first two `[D]` lines and most of the
third are held and the two rows are owed; the traceability row reads "In progress"; 11-12
reads it clause by clause. The row is `4/20`. #64's dialog is #64's.

## Self-Check: PASSED

`tests/the_log_carries_what_a_report_needs.rs` exists; `grep -c 'log_level: "info"'
src/data/config.rs` is 0 and `grep -c 'default_log_level'` 3; `grep -c 'level: LogLevel::Info'
src/common/logging.rs` is 0 and `grep -c 'fn filter_for'` 1; `grep -c 'tracing::info!'
src/presentation/wx_app.rs` is 33 and `tracing::debug!` 4; `grep -c '^| Error \|^| Warn \|^|
Info \|^| Debug \|^| Trace ' docs/ALPHA_TESTING.md` is 5; `guards/guards.toml` holds 918
records by the TOML reader and the census says 798 + 120; `.planning/WINDOWS.md` holds 535,
536 and 537 in both halves; `gh issue view 71` answers OPEN with the comment. Commits
`18f93bb2`, `7058f21d`, `9b82ed16`, `d79f67e4`, `4de64af5`, `92c4a796` and `03513fd0` are in
`git log --oneline --all`. Carriage returns zero and em dashes zero on this file, `STATE.md`,
`ROADMAP.md` and `REQUIREMENTS.md`.

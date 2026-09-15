# Measurements

This is the one page a figure about this tree is written on. How many guard
records there are, how many mutants the configuration allows, how long the
suite takes: each of those is a number that was true on the day somebody took
it, and this page says which day, which command, and which commit. Every other
page points here instead of restating a number, so that a figure which has
moved reads as a dated measurement rather than as a current fact.

**Nothing on this page is promised to be current.** A row says that its value
is what its command reported on its date at its commit, under its conditions,
and nothing more. The check that reads this page,
`tests/every_number_carries_its_command_and_its_date.rs`, holds the shape of
every row and never its value: it refuses a row without a command, a date or a
commit, and it never compares a value with what the command reports today.
That is on purpose. PERF-06 records that a check asserting a written count
equals today's count is false the next time somebody adds a test.

## What belongs here, and what does not

Four kinds of number live in this project's documents, and only the first
belongs on this page.

| Kind | Example | Where it lives |
|---|---|---|
| A measurement of the tree | how many guard records there are; how long the suite took; the rate one guard record costs | here, one row each |
| A target | memory under 150 MB with 1,000 cached messages | `.planning/REQUIREMENTS.md`, marked as a target |
| A constant the code holds | a single attachment is kept up to 25 MB, which is `LARGEST_ATTACHMENT_KEPT_BYTES` | the code, and a check that the prose agrees with it |
| A historical record | what a sweep found on 2026-08-12 | wherever it was written, and never corrected |

A duration belongs here with the settings that change it: the thread count,
whether the build was warm, what else was running. A duration without those is
a number without its conditions, and this project has quoted several of them
to people planning work.

## How to add a row

Run the command. Write the value it printed, the command as you ran it, the
date, the short hash of the commit you ran it at, and what would move the
figure. If nothing would, write `none`. Do not copy a figure from a plan, a
research document or `CLAUDE.md`; run the command. If the same figure is taken
again on a later day, add a row rather than editing the old one, so the page
keeps the series. Two rows with the same `what` and the same date are refused.

Pipes inside a command are written `\|` so the table stays a table.

## The definitions the start and memory rows were taken under

Three of the project's targets use words they do not define: memory under
150 MB with 1,000 cached messages, cold start under 2 seconds to a usable
list, and idle memory under 100 MB. The rows below were taken under these
meanings, which are the same words the harness's header states in
`tests/the_numbers_the_targets_ask_for.rs`, so the next person re-taking a
number takes the same one.

**1,000 cached messages** means 1,000 rows in `messages` for one IMAP
account's `INBOX`, written through the same call the sync uses, with a
plain-text body of about 2 KB each through the body store, no attachment
content and no signed originals. Because that is what the list reads at
startup and what the target was written about; the attachment and
signed-original budgets are constants with their own rows and their own
limits, and mixing them in would measure the budgets rather than the
messages.

**Cold start** means a fresh process of the release binary against the
1,000-message profile, from the first instruction of `main` to the usable
line. The first start after the binary is built is reported on its own as
the file-cache-cold figure; the next five starts are the series and their
median is the number. The requirement says "cold" and does not say cold
for what; both readings are given so nobody has to guess which was meant.

**Usable** means the first `MessagesLoaded` after startup whose rows
reached the list control's count and were at least one, which is the line
`wixen_mail::common::started` writes once per process.

**Idle** means no input after the usable line, the window left where the
start put it, and memory read at 60 s and at 120 s; the 120 s reading is
the number, the 60 s reading sits beside it, and the difference is
reported as idle growth.

**Memory with 1,000 cached messages** is the peak working set of the
application process between start and the 60 s reading, plus the WebView2
tree's working set at 60 s, on the 1,000-message profile, because "with
1,000 cached messages" is about what loading them costs and the peak is
when it cost most. The empty profile's 120 s reading is the floor and is
reported beside both.

Two things the first taking found that the definitions did not say. The
list the usable line reports holds 500 rows and not 1,000, because the
window opens on All Inboxes and that view lists its first page,
`ALL_INBOXES_LIMIT` in `src/presentation/wx_app.rs`; the other 500 are in
the cache and are read when somebody asks for older mail. And the WebView2
tree is not the application's own memory in any sense the targets were
written about: it is six `msedgewebview2.exe` processes Windows starts for
the preview pane, and it weighs the same on an empty profile as on the
1,000-message one, so every memory row gives the application process and
the tree separately as well as summed. Whether a target is met against
the sum or against the process is 08-09's judgement, not this page's.

## What the scale rows time, and what they do not

The rows over 200,000 rows were taken on 2026-09-14 by
`tests/the_list_at_two_hundred_thousand_rows.rs`, a harness that builds the
rows through `sample_mailbox`, the generator on the Help menu, and runs the
real functions with no window. Its header says what each kind of row means;
the short form is here so the rows can be read without it.

**Listing** is `MessageCache::get_messages_for_folder`, the read the list
takes when a folder opens and the one that precedes any scroll. The rows
were written into a cache in a `tempfile` directory through
`upsert_messages`, the call the sync uses, because the Help menu's sample
never enters SQLite. Cold is the first read on a connection opened after the
rows were written; the file was warm in the operating system's cache either
way, because the same process had just written it.

**Filter** is `MessageCache::search_messages`, the query the search box
runs, at the limit the box passes, under All Folders and under the
narrowest answer the In list offers for that kind of word. One row raises
the limit to every match, which the box never does, so the page says what
reading every match costs.

**Sort** is `mail_sort::sort_messages` over the generated rows in memory,
each order on a fresh clone with the clone outside the timing. In the
running program `apply_sort` runs it off the interface thread and sends the
result back through `MessagesLoaded`; the cost of the list control taking
that result was not timed, because there is no window.

**Page paint** and **full pass** are `virtual_rows::text_for`, the whole of
what the list's paint callback does after taking the lock, over one page of
every visible inbox column and over every row of one column. A scroll in the
running program is the page paint plus wxWidgets' own painting, which was
not timed, for the same reason. The full pass is what painting the whole
list once would cost the callback, which no scroll does.

Every one of these rows is over synthetic rows and no provider mailbox, and
each says so in its conditions.

## The figures

| What | Value | Command | Date | Commit | Conditions |
|---|---|---|---|---|---|
| Guard records in `guards/guards.toml` | 783 | `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"` | 2026-09-14 | 7b2482b1 | One more with every record added. The two census numbers at the top of the file must sum to this, and a check holds them to it on every commit. A TOML reader rather than a grep, because a grep for `[[guard]]` would also count one inside a `before` or `after` string. |
| Guard records naming exactly one file in `tests_last_seen` | 568 | `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(sum(1 for r in g if len(r.get('tests_last_seen',[]))==1))"` | 2026-09-14 | 7b2482b1 | Moves with every record added and with every record whose break starts reddening a test in a second file. |
| Guard records spelled as inline tables | 4 | `grep -c 'tests_last_seen = \[{' guards/guards.toml` | 2026-09-14 | 7b2482b1 | The awk snippet in `CLAUDE.md` skips a record in this shape, which is why the TOML reader is the method above. Moves when a record is written or rewritten in that shape. |
| Distinct files named by guard records in `tests_last_seen` | 200 | `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len({e['file'] for r in g for e in r.get('tests_last_seen',[])}))"` | 2026-09-14 | 7b2482b1 | Moves when a record first names a file no record named, or the last record naming a file goes. |
| Guard records naming `tests/house_style.rs` | 21 | `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(sum(1 for r in g if any(e['file']=='tests/house_style.rs' for e in r.get('tests_last_seen',[]))))"` | 2026-09-14 | 7b2482b1 | A test added to that file flags this many records for re-measurement, at the per-record rate below each. Moves with every record that names the file. |
| Guard records naming `src/presentation/wx_app.rs` | 48 | `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(sum(1 for r in g if any(e['file']=='src/presentation/wx_app.rs' for e in r.get('tests_last_seen',[]))))"` | 2026-09-14 | 7b2482b1 | The same cost as the row above, for the other hot file. Moves with every record that names it. |
| Mutants the configuration allows | 12,335 | `cargo mutants --list \| wc -l` | 2026-09-14 | 7b2482b1 | `cargo-mutants 27.1.0`, reading `.cargo/mutants.toml`, which excludes the wxWidgets layer, `main.rs`, the vendored code, and every `Display` and `fmt`. The listing took 3 seconds and built nothing. Moves with every function added or removed in the files the configuration allows, and with any change to the exclusions. |
| Files those mutants are in | 247 | `cargo mutants --list-files \| wc -l` | 2026-09-14 | 7b2482b1 | The same configuration as the row above. Moves with every file added under `src/` that the exclusions do not cover. |
| Commits on `main` | 2,025 | `git rev-list --count HEAD` | 2026-09-14 | 7b2482b1 | Taken on `main` at its tip. One more per commit, and a merge counts every commit it brings. |
| Test attribute lines under `src/` and `tests/` | 7,679 | `grep -rhE '^\s*#\[(test\|tokio::test)\]\s*$' src/ tests/ \| wc -l` | 2026-09-14 | 7b2482b1 | Counts lines that are only the attribute, so a `#[test]` in a doc comment is not counted and an ignored test is. It is not the number of tests that run: the library run below is smaller because it leaves out `tests/`, and a run under `--all-targets` differs again. |
| Tests the library builds | 7,245 | `cargo test --lib -- --list \| tail -1` | 2026-09-14 | 7b2482b1 | The library alone. Nothing under `tests/` is in it, and the one ignored test is. This is the denominator of the per-record rate below. |
| Tests every target builds | 7,697 | `cargo test --all-targets -- --list 2>&1 \| grep -E '^[0-9]+ tests?,' \| awk '{s+=$1} END{print s}'` | 2026-09-14 | a42331bb | The library, the binary and every target under `tests/`, summed over the one summary line each target prints; 55 targets that day. The library row above is the unit half of this and the row below is the integration half, and the three add up. This is the figure `docs/IMPLEMENTATION_STATUS.md` states. Moves with every test added anywhere. |
| Tests the targets under `tests/` build | 452 | `cargo test --all-targets -- --list 2>&1 \| awk '/Running tests/{t=1;next} /Running/{t=0;next} t && /^[0-9]+ tests?,/{s+=$1} END{print s}'` | 2026-09-14 | a42331bb | The integration targets alone, 53 of them that day, read off the same listing by summing only the summary lines that follow a `Running tests/...` line. The binary's unit tests are neither here nor in the library row and numbered none. Moves with every test added under `tests/`. |
| Test functions in `tests/house_style.rs` | 70 | `grep -cE '^\s*#\[(test\|tokio::test)\]\s*$' tests/house_style.rs` | 2026-09-14 | 7b2482b1 | The file 21 guard records name. A test added here changes this and flags those records. |
| Test functions in `src/presentation/wx_app.rs` | 199 | `grep -cE '^\s*#\[(test\|tokio::test)\]\s*$' src/presentation/wx_app.rs` | 2026-09-14 | 7b2482b1 | The file 48 guard records name. A test added here changes this and flags those records. |
| Lines of Rust under `src/` | 360,792 over 286 files | `find src -name '*.rs' \| xargs wc -l \| tail -1` for the lines, `find src -name '*.rs' \| wc -l` for the files | 2026-09-14 | 7b2482b1 | Counts blank lines and comments as lines. Moves with every edit. |
| Entries in `.planning/WINDOWS.md` | 441 | `grep '^total_count:' .planning/WINDOWS.md` | 2026-09-14 | 7b2482b1 | Read from the ledger's own frontmatter, which the tool that appends entries keeps. One more per entry appended; closing an entry does not move it. |
| Full gate duration, `scripts/check.sh all`, from commit bodies | a band, 275 s to 654 s | `git log --format='@@@%ad %h%n%B' --date=short -400 \| awk '/^@@@/ {hdr=substr($0,4); next} /[0-9][0-9]+ seconds/ { if (match($0, /[0-9]+ seconds/)) print hdr "  " substr($0, RSTART, RLENGTH) }'` | 2026-09-14 | 7b2482b1 | A band and not a figure, and it is read with its sentences. The full-gate figures the harvest returns are 275 s at `06aa8765` on 2026-09-14 and 419 s to 654 s across `5da0702a`, `b8857bc8`, `44615216`, `ba2ff758`, `a0b909e8`, `5111ed3a` and `c6546e66` on 2026-09-12. The harvest also returns 64 s at `05ec26a4`, which that commit's body describes as a scoped run before a rule was added, and 95 s at `cccb2d4d`, which is a per-record cost, so a number pulled out of a sentence is that sentence's number only with the sentence. The band moves with whether the previous commit forced a clippy rebuild, with what else the machine was doing, and with the suite's size. |
| Cost of one guard record through `scripts/guards.sh` | 92 s a record: rebuild 44 to 46 s, run 47 s | `scripts/guards.sh --remeasure "the editor stores off when the last alert is taken away"`, twice, reading the `timed:` line the runner prints after the record | 2026-09-14 | bb61e88e | `WIXEN_TEST_THREADS` at its default of 8. The library at 7,245 tests. The build warm and nothing else building, checked with `tasklist` for `cargo.exe` and `rustc.exe` before each take. The record names one file, `src/presentation/managers.rs`, and its suite is the library, which is the common case. The two takes read rebuild 46 s and run 47 s, then rebuild 44 s and run 47 s, 93 s and 91 s in all. The `--named-only` shape of the same record read rebuild 44 s and run 3 s, 47 s. The pre-read the runner does once per suite before any break is not in the rate. Taken with the timing line in the working tree before it was committed; the Rust under `src/` was the tree at the commit named. Both terms move: the rebuild with what a one-file change invalidates, the run with the test count and the thread setting. |
| The whole guard sweep, every record once | 784 x 92 s = 72,128 s, about 20 hours | the record count, `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`, times the rate from `scripts/guards.sh --remeasure` in the row above | 2026-09-14 | bb61e88e | A product of two dated terms and not a promise. Since 2026-09-10 at `eda27192`, when the rate was 29 s to rebuild plus 66 s to run over 683 records, the rebuild has risen to 44 s, the run has fallen to 47 s, and the count has risen to 784. Both terms go on moving and the sweep's own log carries a timing line per record, so the real figure is read from the log afterwards. Triage of what the sweep finds is on top. |
| Every target, `cargo test --all-targets`, wall time | 104 s and 103 s | `s=$(date +%s); cargo test --all-targets --no-fail-fast; e=$(date +%s); echo $((e-s))` | 2026-09-14 | 9399a1e2 | The harness's default thread count, which is what `cargo mutants` runs per mutant unless told otherwise. Every target built beforehand with `--no-run`, so this is the run and not the build. Nothing else building, checked with `tasklist` before each take. 7,687 tests passed, summed over 55 result lines. This is the figure the timeout settings in `.cargo/mutants.toml` are read against, and the figure `CLAUDE.md`'s gate paragraphs call the test suite's term. Moves with the test count, the thread count, and what else the machine is doing. |
| The library alone at eight threads, `cargo test --lib`, wall time | 52 s and 52 s | `s=$(date +%s); cargo test --lib -- --test-threads=8; e=$(date +%s); echo $((e-s))` | 2026-09-14 | 9399a1e2 | `--test-threads=8`, the same setting `scripts/guards.py` uses. The library already built. Nothing else building, checked with `tasklist` before each take. 7,244 passed and 1 ignored; the harness's own line said 51.06 s and 51.60 s. This is the shape a mutation run may be told to use by passing `--lib` after `--`, and the run term the guard runner reported as 47 s in the rate row was the same suite at the same setting. Moves with the test count and the thread count. |
| Cold start to a usable list, 1,000 cached messages | 476 ms, the median of five | `cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture test_cold_start_and_memory_with_a_thousand_cached_messages`, reading the `usable:` line it prints | 2026-09-14 | 9d5f15c5 | Under the definitions above. The release binary at version 0.124.0, built from this commit after `cargo clean --release -p wixen-mail`, on an AMD Ryzen AI 9 HX PRO 370 with 24 logical processors and 92 GB, as `Get-CimInstance Win32_Processor` and `Win32_ComputerSystem` report it. The five: 721, 476, 474, 483 and 468 ms. The first start after the build, on its own: 520 ms. A sixth start after the series, taken to read the log: 472 ms. Every start was checked with `tasklist` for `cargo.exe` and `rustc.exe` first and found none but the one running the test; the test binary runs one ignored test, so no thread setting is in play. The usable line said 500 rows each time, the first page of All Inboxes. The profile's account points at `127.0.0.1` on a closed port and was never dialled: nothing checks mail on a schedule, so a start attempts no connection and the log holds no WARN or ERROR line. Moves with the machine, with what else it is doing, with the page size and with anything added to the startup path. |
| Memory with 1,000 cached messages: the application's peak working set to 60 s plus its WebView2 tree at 60 s | 390 MB, the median of five: the application 57 MB peak plus the tree 333 MB | `cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture test_cold_start_and_memory_with_a_thousand_cached_messages`, reading the `at 60 s:` line | 2026-09-14 | 9d5f15c5 | Under the definitions above, the same runs, binary and machine as the cold-start row. The five: 397, 390, 390, 393 and 390 MB. The application process alone peaked at 57 MB in every run, with a working set of 56 to 57 MB and 18 to 19 MB private. The tree was six `msedgewebview2.exe` processes every time, 333 to 340 MB at 60 s; in the sixth run they were 122, 73, 56, 41, 22 and 17 MB. The after-build run read 396 MB. Whole megabytes of 1,048,576 bytes, read through `Get-Process` as `PeakWorkingSet64` and `WorkingSet64`. Moves with WebView2's version, with what the preview shows, and with the machine. |
| Idle memory at 120 s, 1,000 cached messages: the application's working set plus its WebView2 tree | 391 MB, the median of five: the application 56 MB plus the tree 334 MB | `cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture test_cold_start_and_memory_with_a_thousand_cached_messages`, reading the `at 120 s:` line | 2026-09-14 | 9d5f15c5 | Under the definitions above, the same runs, binary and machine as the cold-start row. The five at 120 s: 398, 391, 390, 393 and 391 MB; at 60 s: 397, 389, 389, 392 and 390 MB, median 390 MB, so idle growth between the two readings is about 1 MB and all of it in the tree. The application process alone sat at 56 to 57 MB working set and 18 to 19 MB private at both readings, unchanged. No input after the usable line and the window left where the start put it. Moves with whatever runs on a timer and with WebView2. |
| The empty-profile floor: idle memory at 120 s with no account and no mail | 390 MB, the median of three: the application 54 MB plus the tree 335 MB | `cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture test_the_empty_profile_floor`, reading the `at 120 s:` line | 2026-09-14 | 9d5f15c5 | The same binary and machine as the rows above, a profile holding the settings and nothing else, no usable line to wait for, the three readings taken at the same moments from the start. The three at 120 s: 389, 390 and 391 MB; the application alone 54 MB working set, 55 MB peak, 17 MB private in every run; the tree six `msedgewebview2.exe` processes at 334 to 336 MB. So 1,000 cached messages cost the application process about 3 MB over an empty profile and cost the tree nothing measurable. Moves with WebView2 and with the machine. |
| Listing 200000 rows from the cache, cold | 351.44 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The one take: 351.44 ms. One take: the first `get_messages_for_folder` on a connection opened after the rows were written, 200000 rows returned. The file was warm in the operating system's cache because this process had just written it. |
| Listing 200000 rows from the cache, warm | 370.53 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 370.18 ms, 370.53 ms, 379.10 ms. The `get_messages_for_folder` reads after the cold one on the same connection, 200000 rows returned each time. |
| Filter 200000 rows for `quarterly`, a word in one subject in five, All Folders | 78.24 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 78.24 ms, 77.85 ms, 80.13 ms. `search_messages` at the search box's limit of 500, 500 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `quarterly`, a word in one subject in five, Subject Only | 78.91 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 80.90 ms, 78.91 ms, 77.75 ms. `search_messages` at the search box's limit of 500, 500 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `zebra`, a word in no subject and no sender, All Folders | 0.16 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 0.53 ms, 0.16 ms, 0.11 ms. `search_messages` at the search box's limit of 500, 0 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `zebra`, a word in no subject and no sender, Subject Only | 0.13 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 0.16 ms, 0.13 ms, 0.09 ms. `search_messages` at the search box's limit of 500, 0 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `grace`, a sender, one in four, All Folders | 108.10 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 109.82 ms, 108.10 ms, 108.03 ms. `search_messages` at the search box's limit of 500, 500 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `grace`, a sender, one in four, Sender Only | 109.67 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 109.35 ms, 111.34 ms, 109.67 ms. `search_messages` at the search box's limit of 500, 500 rows returned each time; the rows carry no message text, so the index holds subjects and senders only. |
| Filter 200000 rows for `quarterly`, every match, All Folders | 192.55 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 192.55 ms, 192.55 ms, 195.29 ms. `search_messages` with the limit raised to 200000, which the search box never does, 40000 rows returned each time: the cost of every match rather than the first page of them. |
| Sort 200000 rows in memory, Date (Newest First) | 126.58 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 126.58 ms, 130.29 ms, 125.24 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Date (Oldest First) | 112.50 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 112.50 ms, 113.62 ms, 107.69 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Sender (A-Z) | 259.50 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 257.55 ms, 273.12 ms, 259.50 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Sender (Z-A) | 235.80 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 235.80 ms, 237.86 ms, 235.36 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Subject (A-Z) | 224.42 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 223.46 ms, 224.42 ms, 225.88 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Subject (Z-A) | 223.50 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 223.50 ms, 218.02 ms, 224.69 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Sort 200000 rows in memory, Unread First | 61.10 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 60.22 ms, 61.10 ms, 67.60 ms. `sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window. |
| Page paint: `text_for` over 40 rows and 6 columns | 0.09 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 0.38 ms, 0.09 ms, 0.09 ms. One page of the messages view, every visible column of an inbox, 4421 characters of cell text each take. A scroll in the running program is this plus wxWidgets' own painting, which was not timed, because there is no window. |
| Full pass: `text_for` over 200000 rows, one column | 28.94 ms | `cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture` | 2026-09-14 | 5cf04528 | 0.125.0 at 5cf04528, release build, AMD Ryzen AI 9 HX PRO 370 w/ Radeon 890M, 24 logical processors, 92 GB, `WIXEN_TEST_THREADS` unset and one test running. 200000 synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. The takes, the median being the value: 30.80 ms, 28.94 ms, 24.13 ms. Every row of the messages view, the subject column, 3960000 characters of cell text each take: what painting the whole list once would cost the callback, which no scroll does. |

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

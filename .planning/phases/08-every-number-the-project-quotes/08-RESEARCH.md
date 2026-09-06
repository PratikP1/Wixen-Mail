# Phase 8: Every number the project quotes - Research

**Researched:** 2026-09-06
**Domain:** In-repo. Nothing here came from a web search and no external package is added by
this phase.
**Confidence:** HIGH on everything measured this session, and every measured claim below
carries the command that produced it. MEDIUM on the two cost estimates, which are
extrapolations and are labelled as such. The two places I could not settle a question are named
in "What I could not settle".

**How this was read.** The repository was read-only for this session: another agent is
executing a plan in the same checkout on branch `spelling-caret-walk`, so nothing was built, no
`cargo` command was run, and no file in the repository was written. Every number below came
from reading files, from `git`, or from a small Python reader over the tree. Where the answer
needs a build I say so and name the command that would produce it.

**The conditions on my own numbers.** Counts taken from the working tree are on branch
`spelling-caret-walk` at `eda6ef7`, which carries nine uncommitted files including
`guards/guards.toml`. Counts taken from `main` are at `48ace28`, 2026-09-06. Where the two
differ I give both, because a plan written against the working tree of another agent's branch
is a plan written against a number that does not exist yet.

---

## Summary

**Three things make this phase larger than the roadmap allows for, and one makes a criterion
unbuildable as written.**

**The guard sweep is 615 records, not 565, and the recorded 15 hours is the smaller of two
figures the tree gives for the same job.** `guards/guards.toml` held 615 records on `main` this
morning and 617 in the working tree. `CLAUDE.md` says 565 in one paragraph, 548 in another and
536 in a third, none of them dated. The per-record cost in `scripts/guards.py` is about 112
seconds; the per-record cost implied by `CLAUDE.md`'s "565 records and about 15 hours" is about
96. Taking the script's figure and today's record count, the sweep is closer to twenty hours
than fifteen, and there is no resume: an interrupted run loses everything it had measured.

**The whole-tree mutation run is several times the recorded two days, because the recorded
figure is a rate times a mutant count from a tree that has since roughly two and a half times
its mutable size.** The non-test, non-excluded source in `src/` went from 38,985 lines on
2026-08-03 to 96,649 on `main` today. The measured mutant counts from the August sweep add to
roughly 4,400 to 4,800, so the same tree today is plausibly near 11,000 to 12,000 mutants. At
the two rates the tree records, one per twenty-six seconds and one per sixty, that is between
about 85 and about 200 hours of continuous running. This is an extrapolation and the first task
of the phase should replace it with `cargo mutants --list`, which parses rather than builds and
costs seconds.

**One part of criterion 2 cannot be built as written.** The 200,000 row sample mailbox is
synthetic and is pushed straight into the in-memory list; it never enters SQLite. The mail
list has no in-memory filter: filtering is `MessageCache::search_messages`, which is a SQL query
with a limit. So there is no path by which the sample mailbox can produce a filter number. The
sort and scroll numbers are straightforward. The filter number needs either a different fixture
(write 200,000 rows into a `tempfile` cache and time the search) or a revised criterion.

**And one thing in the brief for this research was already fixed.**
`test_no_status_page_names_a_version_the_code_does_not_ship` is described in `CLAUDE.md` as
disarmed and in need of a companion. The companion exists:
`test_the_version_reading_can_see_one_on_a_real_page`, `tests/house_style.rs:3104`, splices a
version nobody ships into each page's own real text and requires the reading to answer with
exactly that page, that line and that version. It is a working model for every other check this
phase writes, and it is quoted in full below rather than designed again.

**Primary recommendation:** start the phase with the two cheap counting tasks that resize
everything else (`cargo mutants --list` and a re-count of guard records), settle the four
scheduling decisions at the end of this document before any long job starts, and write the
provenance check against the taxonomy in the next section rather than against the word "count".

---

## The taxonomy criterion 3 needs, and why it is the whole design

Criterion 3 says every count in the documentation carries the command it came from and the
date, and then says nothing may assert that a written number equals what a tool reports today.
Those two sentences are only compatible if "count" is split first. A check written against the
word "count" fails on the first line of `docs/privacy.md` and on every line of the changelog.

Four kinds of number live in this project's documents, and only the first wants a command and a
date.

| Kind | Example, with its real location | What it needs | Can a check assert it? |
|---|---|---|---|
| **A measurement** | "5,430 tests pass ... counted 2026-08-29 with `cargo test --all-targets -- --list`", `docs/IMPLEMENTATION_STATUS.md:129` | The command, the date, and any setting that changes it | Only that the command and date are **present**, never that the number is current |
| **A target** | "Startup time optimization (<2 seconds)", `docs/roadmap.md:221` | To be marked as a target, with where the target came from | That it is not written as though it were a measurement |
| **A constant the code holds** | "A single file is kept up to 25 MB", `docs/privacy.md:41`, against `LARGEST_ATTACHMENT_KEPT_BYTES` at `src/data/message_cache/attachment_content.rs:51` | To agree with the code | **Yes, on every commit.** Both halves are in the repository |
| **A historical record** | "The numbers on every record below were written on 2026-09-02 ... not a statement that 548 records are correct", `guards/guards.toml:74` | To stay exactly as written | **Yes, that it is not silently updated.** Correcting it would falsify it |

The dividing line is not "count" against "not a count". It is **whether the thing counted is in
the repository**. A count of `[[guard]]` records in `guards/guards.toml` is checkable in
milliseconds on every commit, and should be. A count of what `cargo test --list` reports is not,
because the checker would have to run the suite to know, and it would fail on the next commit
that adds a test.

`tests/house_style.rs` already holds the pattern for each of the two checkable kinds:

- `test_no_changelog_list_is_introduced_by_a_count_that_disagrees_with_it`
  (`tests/house_style.rs:2924`) asserts a stated count agrees with the bullets it introduces.
  Internal consistency, no tool run.
- `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  (`tests/house_style.rs:4780`) asserts two numbers in a header add up to the records in the
  file they head. Its own comment says what it cannot see: "whether a record the sweep counted
  was really swept. Only `scripts/guards.sh` can say that."

Both of those are the right shape and both carry a companion proving the reading can see a
violation. Copy them; do not invent a third shape.

**The historical-record case is worth stating out loud, because the obvious check breaks it.**
`guards/guards.toml:74` says 548 records were recounted on 2026-09-02. The file holds 615 today.
That sentence is **correct**, and a check demanding it equal today's count would demand it be
falsified. Measured: the file held 548 records from `b232622` (2026-09-01 13:28) through
`9d6543a` (2026-09-02 05:56), so the number was right for the event it describes.

```bash
for c in $(git log --since=2026-09-01 --until=2026-09-03 --format=%H main -- guards/guards.toml); do
  echo "$(git show "$c:guards/guards.toml" | grep -c '^\[\[guard\]\]') $(git log -1 --format='%h %cd' --date=format:'%m-%d %H:%M' $c)"
done | sort -n
# 548 appears from b232622 09-01 13:28 to 9d6543a 09-02 05:56
```

---

## What already exists, measured

Read this before planning anything. Four of the six criteria have machinery in the tree that a
plan should extend rather than build.

### The document-reading harness reaches everything except `.planning/`

`ours()` at `tests/house_style.rs:33` collects `src` (`.rs`), `docs` (`.md`), `tests` (`.rs`),
`scripts` (`.sh`, `.py`, `.ps1`), `guards` (`.toml`), `installer` (`.iss`), `.github` (`.yml`),
and the five single files `README.md`, `CLAUDE.md`, `Cargo.toml`, `.gitignore`, `build.rs`.

**`.planning/` is not in it and never has been.** Measured:

```bash
git log --oneline -S '".planning"' -- tests/house_style.rs      # no output
git log --oneline -S 'Path::new(".planning")' -- tests/house_style.rs   # no output
grep -rn "\.planning" tests/*.rs scripts/*.sh scripts/*.py .githooks/*
# Only two doc comments naming .planning/WINDOWS.md, plus two cases in
# scripts/which-checks.test.sh that classify a .planning path as docs_only.
```

Two consequences. A provenance check hung on `ours()` covers every document in the tree except
the planning documents, which is probably right and is a decision for Pratik. And a change to a
`.planning/` file is classified `docs_only` by `scripts/which-checks.sh`, which then runs the
document-reading targets, **and none of those targets opens the file that changed**. A planning
document is currently checked by nothing.

That also means one sentence in `CLAUDE.md` is not reproducible: it says the em-dash guard has
caught two breaks in markdown, "one in `CLAUDE.md` and one in a planning file". `CLAUDE.md` is
in `ours()`; a planning file is not, and never was. I could not find what else could have caught
it. Marked as unsettled rather than as wrong.

### A documents-only commit already runs five targets, cheaply

`scripts/check.sh:326` in `docs_only` mode runs `cargo test --lib help_page::` and then
`cargo test --no-fail-fast --test house_style --test docs_links --test wired --test
checkbox_labels --test manager_delete_stays_open`. So a new provenance check placed in
`house_style.rs` runs on every documents commit at no extra build. That is the cheap home.

### The version guard's companion, which is the model for this phase

`tests/house_style.rs:3104`, and this is the pattern criterion 3 wants copied:

```rust
const A_VERSION_NOBODY_SHIPS: &str = "0.0.0-notashippedversion.1";

#[test]
fn test_the_version_reading_can_see_one_on_a_real_page() {
    // ... asserts the page list is non-empty, asserts each page opens and is
    // non-empty, asserts the page names no version today (the vacuity, asserted
    // rather than assumed), then splices A_VERSION_NOBODY_SHIPS into the page's
    // own real lines and requires the reading to answer with exactly that page,
    // that line and that version.
}
```

Three things it does that a naive companion does not. It runs over the real files rather than
over literals, because "the file opens", "the file holds text" and "the file is read line by
line" are the links that break. It asserts the current vacuity rather than assuming it, so the
day a page does name a version the companion says so. And it names the exact expected string,
not just "non-empty", so a reading that answers with the wrong line fails.

`test_the_version_reading_can_see_one` (`:3167`) sits beside it and proves the extractor against
the shapes that were really in the two pages, plus four neighbours that must not match
("OAuth 2.0", "WCAG 2.2", "60.4%", a commit hash).

### The guard-record bookkeeping, which the sweep will interact with

Two checks run in milliseconds on every commit and are named in `CLAUDE.md`:

- `test_every_guard_record_says_how_many_tests_the_files_it_names_held`,
  `tests/house_style.rs:5014`. Compares each record's `tests_last_seen` counts against a static
  count of `#[test]` and `#[tokio::test]` attributes in the files it names, and prints
  `scripts/guards.sh --remeasure "..."` for the records that moved.
- `test_every_test_a_guard_record_names_is_a_test_that_exists`, `tests/house_style.rs:5117`.
  Catches a rename, which a count cannot see.

`scripts/guards.py` reads the same anchored regex (`scripts/guards.py:64`):
`^[ \t]*#\[(?:tokio::)?test\][ \t]*$`, both spellings, because 573 of the tests here are
`#[tokio::test]`.

**This matters to criterion 3 more than it looks.** The project already has a working mechanism
for comparing a written count against a static count of test attributes in named files. It is
the one existing precedent for checking a test-count claim without running the suite. It also
proves the limit: a static attribute count is **not** the number `cargo test --list` reports, so
a check built on it could not be pointed at the documents' 5,430 without comparing two different
quantities.

### The scale sample ships and is reachable

`SAMPLE_MAILBOX_SIZE = 200_000` at `src/presentation/wx_app.rs:9212`; `sample_mailbox` at
`:9224`; `ID_LOAD_SCALE_SAMPLE` declared at `:97`, menu item at `:6453`, handler at `:5004`.
The handler builds the rows and sends `UIUpdate::MessagesLoaded(generated)`. The rows never
touch SQLite.

### The virtual text callback, and what type already guarantees

`msg_list.set_virtual_text_callback(...)` at `src/presentation/wx_app.rs:1150`. The closure
captures exactly two things: `state: Arc<StdMutex<WxUIState>>` and
`column_layout: Rc<RefCell<ColumnLayout>>`. It reads `state.showing`, then
`state.conversations.get(row)` or `state.messages.get(row)`, and calls
`message_rows::conversation_cell_text` or `message_rows::cell_text`. The comment at `:1141`
already states the property: "The callback runs while wxWidgets paints, so it reads what is
already in memory and never touches the database."

`WxUIState` (`:246` to `:663`) holds no database connection and no `MessageCache`. Its one
field that sounds like one, `saved_searches`, holds
`std::collections::HashMap<String, ...saved_searches::SavedSearchesRead>`, which is a read
result rather than a handle. `cell_text` (`src/presentation/message_rows.rs:59`) and
`conversation_cell_text` (`:125`) take `&MessageItem` / `&ConversationItem`, a column, a
`DateSettings` and a `chrono` instant, and nothing else.

**So the property holds by type today, and that is the strongest form of it.** The gap
criterion 2 names is real but narrower than it sounds: it is that a future edit could add a
query inside the closure body, and nothing would notice.

---

## Criterion by criterion

### Criterion 1: memory with 1,000 messages, cold start, idle memory

**Nothing exists.** Re-checked 2026-09-06, and the audit's finding of 2026-09-04 still holds:

```bash
ls -d benches                     # no benches/
grep -n "bench\|criterion\|divan" Cargo.toml     # no output
grep -rn "sysinfo\|GetProcessMemoryInfo\|working_set" src/ --include=*.rs   # no output
grep -n '^name = "sysinfo"' Cargo.lock            # no output
```

`Instant::now` appears 19 times in `src/`, all of them in feature code (announcement
coalescing, OAuth expiry, free/busy), none timing startup.

**Reading resident memory needs no new dependency, and the house pattern is already set.**
Twenty `#[link(name = "...")]` extern blocks exist in `src/`, and `Cargo.toml:283` states the
convention out loud: "Everything else this project needs from Windows is a flat call behind a
small `#[link]` block, and stays that way." The nearest model is
`src/service/handover.rs:51`:

```rust
#[cfg(target_os = "windows")]
fn this_session() -> u32 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcessId() -> u32;
        fn ProcessIdToSessionId(process: u32, session: *mut u32) -> i32;
    }
    // ...
}
```

`K32GetProcessMemoryInfo` is exported from `kernel32`, so a memory reading fits that shape
exactly, and the `windows` crate's `Win32_System_ProcessStatus` feature (not currently enabled,
`Cargo.toml:286`) does not need turning on. A non-Windows fallback keeps the crate building, as
every other block here does.

**Cold start is the one that has to run in production, not in a test.** Guardrail 1 says a
feature is done when a non-test path reaches it. A cold-start number produced only by a test
harness is a number about the harness. The route that satisfies both the guardrail and the
criterion's "date, machine and build" is to have the application log it: `tracing` is already
set up with a file appender (`tracing-appender`, `Cargo.toml:61`), and `common::version::current()`
(`src/common/version.rs:32`) already yields `0.45.0` or `0.45.0+g64c73dd` for a build the
installer script made. One log line at the moment the message list becomes usable gives the
number, the build and the date at once, on any machine anybody runs it on.

**"Usable message list" needs defining before it can be measured**, and the requirement already
says why: "not to the window appearing, because an empty window is not a usable inbox". The
honest marker is the first `UIUpdate::MessagesLoaded` reaching the list, or the first successful
`set_item_count` on it. Naming which is a plan decision, and whichever is chosen the definition
belongs beside the number, because two definitions of cold start differ by seconds here.

`tests/command_line_output.rs` is the precedent for running the real executable from a test; it
runs `--help`, `--version` and a refusal, none of which opens a window.

### Criterion 2: 200,000 rows, and the filter path that does not exist

**Sort: straightforward, and expect it to find a freeze.** `sort_messages`
(`src/presentation/wx_app.rs:20141`) is a private free function over `&mut [MessageItem]`, so a
timing test lives beside it in `wx_app.rs`'s own `#[cfg(test)] mod tests` and runs under
`cargo test --lib presentation::wx_app::`.

Its caller is the finding. `apply_sort` (`:20092`):

```rust
let sorted = {
    let mut s = lock_state(state);
    s.sort_order = order;
    let mut msgs = s.messages.clone();   // 200,000 MessageItem, cloned under the lock
    sort_messages(&mut msgs, order);     // sorted under the lock
    msgs
};
```

The whole vector is cloned and sorted while the UI state mutex is held. The requirement's own
`[S]` line says a multi-second freeze on a header click is an accessibility failure rather than
a performance one, and `Cargo.toml`'s clippy section records that a held lock guard is how this
application once froze NVDA. So the sort measurement is likely to produce a number that fails
its own criterion, and the plan should expect to carry a fix, not only a figure.

**Scroll: the virtual callback, and the property is already true by type.** Timing it means
timing `cell_text` and `conversation_cell_text` across the row set, which is a plain unit test.

**The "no SQLite query" assertion has three candidate shapes and one is much stronger.**

1. *Make it a type property and pin the type.* Extract the closure body into a named function
   whose parameters are `&WxUIState`, the visible columns, the row and column indices, the
   `DateSettings` and the instant, and register the closure as a single call to it. A query then
   cannot be written without adding a parameter or an import, which is a compile error rather
   than a silent regression. Cost: the guard cannot be recorded in `guards/guards.toml`, because
   `scripts/guards.py` measures a break by which **tests go red**, and a break that stops the
   build raises `Wrong` instead (`scripts/guards.py:500`, `why_no_test_was_named`).
2. *A source-reading guard over the registered closure.* Assert the block passed to
   `set_virtual_text_callback` names no cache type. Cheap, recordable, and this project has
   written down five distinct ways a source-reading guard goes vacuous, including "a
   source-reading guard anchored on the line its own GREEN half deletes". It needs a companion
   in the shape of the version companion above, and the companion is most of the work.
3. *Instrument the connection.* `rusqlite` 0.40 is the SQLite crate (`Cargo.toml:81`) and offers
   `trace` and `profile` hooks. Nothing in `src/data/` installs one today (`grep -rn
   "trace(\|profile(\|set_authorizer" src/data/ --include=*.rs` returns nothing). This is the
   only shape that would notice a query arriving by a route nobody predicted, and it is also the
   only one that needs a connection on the path, which today there is not.

My recommendation is 1 plus 2: take the type guarantee, and record a source guard over the
closure with a real companion, because the type guarantee is invisible to anyone reading the
callback in six months.

**Filter: this is the part that cannot be built as the criterion describes it.**

- The sample rows never enter SQLite. The handler at `wx_app.rs:5004` sends them straight to the
  list.
- The mail list has no in-memory filter. `grep -n "group_filter\|active_group\|retain(" 
  src/presentation/wx_app.rs` returns one line, and it is a **contacts** test
  (`test_clearing_the_search_box_returns_to_the_active_group_filter_alone`, `:20983`), reached
  through `recompute_which_contacts_are_shown`. Contacts have an in-memory filter; mail does not.
- Mail filtering is `MessageCache::search_messages` (`src/data/message_cache/searching.rs:477`),
  a SQL query taking a `limit`, called with 50 in its own tests.

So a filter number over the sample mailbox would be a number about an empty database. Two honest
routes, and choosing between them is a decision below: write 200,000 rows into a `tempfile`
cache and time `search_messages` against that, which answers the real question and needs no
window; or revise the criterion to say sort and scroll and record why filter was dropped.

### Criterion 3: provenance in the documents

**The scale is about a hundred sites, not a thousand.** A heuristic reading of every `*.md`
under `docs/` plus `README.md`, looking for a figure attached to a countable noun and for a
provenance marker within three lines:

```bash
# reader saved at scratchpad/phase-08/numbers.py; pattern is a number followed by one of
# tests|records|mutants|commits|guards|percent|%|MB|seconds|minutes|hours|days|rows|messages|
# files|modules|survivors|misses|events|channels|settings|shortcuts
python numbers.py docs README.md
# number-shaped claims found: 112
# with a provenance marker within 3 lines: 30
# without: 82
#   38 (33 bare)  docs/changelog.md
#   24 ( 7 bare)  docs/plans/20260801-mutation-sweep.md
#   11 (10 bare)  docs/plans/20260726-mail-at-scale.md
#   11 ( 7 bare)  docs/plans/20260823-earcon-sound-schemes.md
#    5 ( 3 bare)  docs/IMPLEMENTATION_STATUS.md
#    5 ( 5 bare)  docs/privacy.md
#    5 ( 5 bare)  docs/roadmap.md
```

That reading is crude and I say so: it counts "10MB" in an attachment warning and "100%
keyboard accessible" as claims. Its value is the order of magnitude. A hand pass over roughly a
hundred sites is a day's work, not a week's, and most of the bare ones turn out to be targets or
constants once the taxonomy above is applied.

**The three test-count sites already agree and already carry command and date.** Measured
2026-09-06:

```bash
grep -n "5,430\|5,269" docs/changelog.md docs/integration-guide.md docs/IMPLEMENTATION_STATUS.md
# docs/changelog.md:1963, 1966, 1967
# docs/integration-guide.md:5
# docs/IMPLEMENTATION_STATUS.md:129
```

`docs/IMPLEMENTATION_STATUS.md:129` reads "5,430 tests pass: 5,269 unit and 161 integration,
counted 2026-08-29 with `cargo test --all-targets -- --list`". Note that the audit of 2026-09-04
cited this as line 123. It is now 129. That is the line-number rot the audit itself warned about,
observed inside two days.

**How far behind those numbers are, without running cargo.** A static count of the test
attribute, using the same anchored regex `scripts/guards.py` uses:

```bash
python - <<'PY'
import re, pathlib
A = re.compile(r'^[ \t]*#\[(?:tokio::)?test\][ \t]*$', re.M)
for root in ("src", "tests"):
    n = sum(len(A.findall(p.read_text(encoding='utf-8', errors='replace')))
            for p in pathlib.Path(root).rglob("*.rs") if "vendor" not in p.as_posix())
    print(root, n)
PY
# src   6355
# tests  234
```

Working tree, 2026-09-06. This is **not** the number `cargo test --list` reports and must not be
written into a document as though it were. The most recent recorded library run is 6,291 passed,
in `.planning/phases/04-writing-and-reading-a-message-in-full/04-03-SUMMARY.md:254`, dated
2026-09-05, from a `scripts/check.sh all` run that took 5m06s.

**`CLAUDE.md` breaks the rule it states, three times in one file, and none of the three carries a
date.** This is the clearest single example of what the phase is for.

| Line | Says | Measured 2026-09-06 |
|---|---|---|
| `CLAUDE.md:456` | "The whole sweep is 565 records and about 15 hours" | 615 on `main`, 617 in the working tree |
| `CLAUDE.md:384` | "471 of the 548 records name one file" | 475 of 615 on `main`; **471 is not reproducible, see below** |
| `CLAUDE.md:385` | "a test added to `src/application/contacts_sync.rs` flags 74 of them" | 77 on `main` |
| `CLAUDE.md:458` | "63 records of the 536 that existed then" | correctly scoped to a past event, so this one is fine |

```bash
grep -c '^\[\[guard\]\]' guards/guards.toml                     # 617  (working tree, eda6ef7)
git show main:guards/guards.toml | grep -c '^\[\[guard\]\]'     # 615  (main, 48ace28)
```

The distribution, re-measured with a reader saved at `scratchpad/phase-08/dist.py` (it splits on
`[[guard]]` and reads each record's `tests_last_seen` entries):

```
main (615 records)                working tree (617 records)
  1 file:  475                      1 file:  477
  2 files: 113                      2 files: 113
  3 files:  17                      3 files:  17
  4 files:   5                      4 files:   5
  5 files:   2                      5 files:   2
  8 files:   1                      8 files:   1
  9 files:   1                      9 files:   1
 12 files:   1                     12 files:   1

top files by how many records name them (identical on both):
  77  src/application/contacts_sync.rs
  69  src/application/calendar.rs
  40  src/presentation/wx_app.rs
  40  src/presentation/managers.rs
  35  src/service/protocols/imap.rs
  34  src/data/message_cache/contacts.rs
  30  src/service/spellcheck/mod.rs
  29  src/service/caldav.rs
141 distinct files named; red lists total 1,644 named tests, mean 2.67, max 42
```

**`CLAUDE.md` names only `contacts_sync.rs` as the worst case, and `calendar.rs` at 69 is a
close second that nothing mentions.** A test added to `src/application/calendar.rs` costs 69
re-measurements, and a plan that budgets only for contacts_sync will be surprised.

**On the 471.** `CLAUDE.md:384-385` was written 2026-09-02 (`git blame`: `1a1a6b4f`), when the
file held 548 records. At that tree, `74` reproduces exactly and `471` does not. Three readings
tried against `git show 9d6543a:guards/guards.toml`:

| Reading | Result at 548 records |
|---|---|
| records with one distinct file in `tests_last_seen` | 439 |
| records whose red tests all live in one exact module | 324 |
| records whose red tests all share a top-level module | 353 |

My reading reaches 471 only at about 611 records, which the file held on the evening of
2026-09-05. So the sentence appears to hold three numbers from three different trees, and it
names no command, so nobody can tell. I could not determine what "471" counted. This is recorded
as a finding rather than resolved.

**Two more internal inconsistencies found while counting, both in
`docs/plans/20260801-mutation-sweep.md`:**

1. Its "Scale" section says "4,171 mutants outside the four already done", and its own table
   directly beneath sums to 3,172 (89 + 327 + 513 + 1,307 + 211 + 725). A difference of 999,
   with nothing reconciling them.
2. Its Progress table cannot be summed without knowing whether the `service` row (1,296 mutants)
   includes the `service/protocols` row (327). The order table above treats them as separate
   ("`service` (rest) | 211"), the progress table does not say. So the measured total is either
   about 4,444 or about 4,771 and the document does not settle it.

**What the roadmap's "the documents agree with each other" already has going for it.** The
three test-count sites agree. The disagreement PERF-06's evidence was originally about is closed.
What remains is drift, which the criterion explicitly says to refresh rather than fail on.

### Criterion 4: one whole-tree mutation run

**What is recorded, with its conditions.**

| Source | Figure | Conditions given |
|---|---|---|
| `.cargo/mutants.toml:38` | suite takes about 46 seconds | measured 2026-08-06, nothing else running; 57 in that run's own baseline with a build alongside |
| `docs/plans/20260801-mutation-sweep.md:13` | "roughly one every twenty-six seconds ... near thirty hours" | dated by the document, 2026-08-01. Machine not named |
| `scripts/mutants.sh:70` | "Serial is roughly a minute per mutant" | cargo-mutants 27.1.0, `MUTANTS_JOBS` default 1 because parallel workers collide with "The file exists (os error 80)" on Windows. Date not given; the file was last touched 2026-09-03 |
| `docs/IMPLEMENTATION_STATUS.md:162` and `CLAUDE.md:323` | "about two days" | none |

Three rates for one job. Twenty-six seconds a mutant against 4,771 is 34 hours; sixty seconds is
80 hours. Neither is two days. The two-day figure matches nothing in the tree.

**The 2026-08-05 run, and a scope disagreement I could not settle.**
`scripts/mutants_report.py:6` records it precisely: 1,470 mutants, 734 caught, 60 missed, 81
timed out, 122 that the compiler rejected, and 473 that never built at all, each dying in a
tenth of a second on a Windows status with no output. Only 997 were really tested. That is also
in commit `9c8b9f9` (2026-08-11) and at `CLAUDE.md:544`, both of which call it a **whole-tree**
run. `docs/plans/20260801-mutation-sweep.md:307` calls the same run "the narrower run of
2026-08-05 that covered `src/application` and `src/presentation` only".

1,470 is well below every whole-tree figure the tree records before or after, which favours the
narrower reading, but that is an inference and the run's `mutants.out` is gone. The only
mutation artefact still on disk is `mutants.out.old/` at the repository root, untracked, dated
2026-07-30, holding 102 mutants over four files (60 in `mail_sync.rs`, 27 in
`imap/special_use.rs`, 14 in `destinations.rs`, 1 in `abilities.rs`), so it is a different run
entirely. **Settling which the 2026-08-05 run was, and correcting whichever document is wrong,
is a task for this phase.**

**How big the run is today, and why the recorded figure cannot be trusted.** The mutable surface
has grown by about two and a half times since the August sweep:

```bash
ref=$(git rev-list -1 --before=2026-08-04 main)     # 02cc65f, 2026-08-03
# non-test lines (everything before the first #[cfg(test)]) in src/*.rs,
# excluding what .cargo/mutants.toml excludes
for r in "$ref" main; do
  tot=0
  for f in $(git ls-tree -r --name-only "$r" -- src | grep '\.rs$' \
      | grep -v -e 'src/presentation/wx_' -e 'src/presentation/managers\.rs' \
                -e 'src/main\.rs' -e 'src/vendor/'); do
    n=$(git show "$r:$f" | awk '/^#\[cfg\(test\)\]/{print NR-1; found=1; exit} END{if(!found) print NR}')
    tot=$((tot+n))
  done
  echo "$r $tot"
done
# 02cc65f  38985
# main     96649
```

Ratio 2.48. Applied to the 4,444 to 4,771 measured in August, that is roughly 11,000 to 11,800
mutants today. At twenty-six seconds each, 80 to 85 hours. At sixty seconds each, 183 to 197
hours, which is eight days of continuous running.

**This is an extrapolation and must be replaced, not quoted.** Mutant density per line is not
constant, and this project writes very long comments, so a line-count ratio overstates. The
correct first task of the phase is:

```bash
cargo mutants --list | wc -l
```

`--list` parses the source with `syn` and does not build, so it costs seconds and answers
exactly. Nothing should be scheduled before it has been run.

**How the report must be read, and what `scripts/mutants.sh` now refuses.**
`scripts/mutants_report.py:305`, `why_this_is_not_an_answer`, refuses five shapes, each with its
own sentence:

- the build failed before anything was changed, so no mutant was tested;
- N of the M mutants never built, with the Windows status printed and the real tested count
  given;
- the run stopped part way through, so this is part of an answer: wait, or read it as the part
  it is;
- every mutant was rejected by the compiler, so the suite never ran;
- there were no mutants at all.

Two operational rules follow from the tree's own record. **Read `mutants.out` only after the
process exits**: reading it early once produced a commit message quoting seventeen of eighteen
caught when the finished figure was thirty-seven of fifty-one. And **a mutant that never started
is re-run, not counted**: six mutants that never built on 2026-08-11 all built when the same
twelve were asked again twenty minutes later, and five of the six were caught.

**Expect the survivor list to be large and to be mostly triage.** The August areas produced 483
misses in `service` alone, of which about half is socket code that stays. The criterion says
every survivor is killed with a test or recorded with a reason, and the sweep plan already
records how that goes wrong: a pass was reported as closing 52 findings and closed 7, found only
because a confirming re-run said 45 were still open byte for byte. Its rule is worth carrying
into this plan verbatim: **past about three findings per new test, ask which test kills which
mutant and expect a named answer, and a sweep is not finished until a second run says so.**

### Criterion 5: the one whole-tree guard sweep

**Size, measured 2026-09-06:** 615 records on `main`, 617 in the working tree, across 141
distinct named files and 112 distinct break-target files.

```bash
git show main:guards/guards.toml | grep -c '^\[\[guard\]\]'          # 615
grep -c '^suite = ' guards/guards.toml                                # 44
grep '^suite = ' guards/guards.toml | sort | uniq -c                  # 14 distinct targets
grep '^file = ' guards/guards.toml | sort -u | wc -l                  # 112
```

571 of the 615 run against `--lib`. The other 44 name one of 14 integration targets: 18
`house_style`, 8 `wired`, and one or two each of `a_whole_folder_moves_both_bounds`,
`an_encrypted_message_is_not_left_unexplained`, `checkbox_labels`,
`manager_delete_stays_open`, `manager_dialog_labels`, `nothing_leaves_the_outbox_unasked`,
`nothing_sends_a_flag_change_unasked`, `one_sign_in_per_piece_of_work`,
`the_conflict_choice_can_be_heard`, `the_list_warning_reads_the_message`,
`tree_dialogs_resolve_the_row_somebody_is_on`, `tree_rows_leave_no_registry_entry`.

**Cost, and why fifteen hours is optimistic.** `scripts/guards.py` gives two per-record figures
with conditions:

- `scripts/guards.py:580`: "the rebuild a break forces is 23 seconds and the library is 89, so
  filtering would take a 220-record sweep from 6.8 hours to about 88 minutes." That is
  111 seconds a record, and the `--named-only` help text at `:1060` states it as "about 112
  seconds a record against 24".
- `scripts/guards.py:30`: the library suite was 5,837 tests on 2026-08-31, on 24 logical cores,
  at 2 threads 131s, 4 threads 88s, 8 threads 106s, 16 threads 164s, harness default 196s.
  `WIXEN_TEST_THREADS` defaults to 4 for that reason and applies to the guard runs only.

The library is now about 6,291 tests (recorded 2026-09-05), 7.8% more than the 5,837 those
timings were taken on. Scaling the 89 second run term gives about 96 seconds, plus the 23 second
rebuild, so about 119 seconds a record. 615 records is 73,185 seconds, **about 20.3 hours**,
plus one full baseline run per distinct suite before any break is applied
(`scripts/guards.py:931`, `what_is_already_failing`), which is 15 more suite runs.

That is an inference from two measured components and I mark it so. `CLAUDE.md`'s "565 records
and about 15 hours" implies 96 seconds a record, which is the library run term with no rebuild.
The two figures in the tree disagree by about 17% and neither is dated.

**There is no resume, and the failure modes are recorded.** Reading
`scripts/guards.py:811-1065`:

- No `--resume` flag and no checkpoint file. The options are a positional substring filter,
  `--touched-by REF`, `--named-only`, `--remeasure NAME...` (exact names, one or many), and
  `--recount-everything`.
- **A plain sweep writes nothing.** `write_down_the_counts` is called in only two places:
  under `--recount-everything` (`:862`) and under `--remeasure` after the loop (`:1017`). An
  ordinary run is a report, deliberately: "a report that edits the thing it reports on is not
  one."
- **An interrupt leaves the tree clean but loses the work.** `measure` restores the file in a
  `finally` (`:706`), writing bytes rather than text so line endings and timestamps survive, and
  `KeyboardInterrupt` is not caught by the per-record `except Exception`. A killed process, as
  opposed to Ctrl-C, would leave a broken source file behind; that is the one case to be careful
  of.
- **One bad record no longer takes the run down.** `:983` catches any exception per record,
  counts it as slipped, and continues. That was added after "a sweep of 208 records died on its
  122nd with `NoneType + str` after about an hour of work".
- **Output is flushed per record on purpose.** `:966` records why: "a 220-record run showed an
  empty file for seven hours". So the log is watchable, and the log is the only artefact.

**Therefore resuming is manual, and the shape of it should be in the plan.** Take the record
names already reported in the log, subtract them from the names in `guards/guards.toml`, and
pass the remainder to a fresh invocation. `--remeasure` accepts many exact names and is the only
mode that also writes the counts for records that agreed, so it is the mode a resumed sweep
wants; the positional filter is substring-matched and cannot express a list.

**Expect findings, and here is the base rate.** The header of `guards/guards.toml` says 192
records were swept on 2026-08-12 and 425 have arrived since (working tree; 423 on `main`), and
those two numbers are checked on every commit. `CLAUDE.md` gives two prior sweeps: one of 208
records found 23 wrong, a later one of 220 found 1. The difference is attributed to phase 2.1
running the scoped remedy every time a commit printed it. Ledger 100
(`.planning/WINDOWS.md`, recorded 2026-09-05, still open) is the most recent data point: two
records wrong, both surfaced by the count check when `mail_sync.rs` gained one test, and one of
them had been **unmeasurable** rather than stale since a struct gained a field hours earlier,
so the run reported a broken tool rather than a finding.

At the 23-in-208 rate, 615 records is about 68 findings. At the 1-in-220 rate it is about 3. The
truth depends entirely on how consistently the per-commit remedy has been run across phases 3
and 4, which I did not measure. Budget for the higher number and be pleased.

**A finding at hour eleven costs a correction by hand plus one more record's run**, about two
minutes, and correcting by hand needs whoever is doing it to understand what the break should
redden. It does not stall the sweep: `scripts/guards.sh` reports and continues, and the
corrections are applied afterwards. So the sweep is one long unattended job followed by a
triage pass, not an interactive session, which is the fact that makes the scheduling decision
below tractable.

**The sweep will change the header, and the header is checked.** After the sweep,
`guards/guards.toml`'s two numbers must be rewritten to say what this sweep covered and that
zero have arrived since, and
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` will fail
until they add up. That is one line in the same edit and should be a named task.

### Criterion 6: each target met or revised, with the reason

Four numbers are targets rather than measurements today, and all four are correctly written as
targets:

| Target | Where | Source |
|---|---|---|
| under 150 MB with 1,000 cached messages | `docs/development/requirements-backlog.md:81` | that backlog, Medium |
| cold start under 2 seconds | `docs/roadmap.md:221` and `:253`, `requirements-backlog.md:82` | roadmap and backlog |
| under 100 MB idle | `docs/roadmap.md:254` | roadmap success metrics |
| line coverage | `docs/IMPLEMENTATION_STATUS.md:207`, 60.4% on 2026-07-26 | last measurement, not a target |

`docs/roadmap.md:255` also carries "100% keyboard accessible", which is a target of a different
kind and is not this phase's.

**Coverage is the one where the criterion tells you what the answer is.** Criterion 3 says low
coverage is attributed to the untested network transport rather than treated as a number to
raise. `docs/IMPLEMENTATION_STATUS.md:207-212` already says exactly that. So the work here is a
re-measure with its date, and leaving the attribution alone.

Since the last coverage reading, per the audit of 2026-09-04, 1,195 of the repository's 1,373
commits have landed, so 87% of the project's history postdates it. Re-run
`git rev-list --count --since="2026-07-26" HEAD` on the day, because that figure has moved too.

---

## Pitfalls specific to this phase

**Do not write a check that compares a document's number to what a tool reports.** Criterion 3
forbids it and PERF-06's fifth `[D]` line records that this is what the requirement used to ask
for and that it was corrected on 2026-08-29 because it is false the next time anyone adds a test.
A check on a number checks that its command and its date are present, and that documents agree
with each other.

**Do not treat a stale number as a defect.** The three documents quoting 5,430 are about 900
unit tests behind. Under the rule this phase is enforcing, that is a re-measure, not a failure.

**Do not update a historical record.** `guards/guards.toml:74` and every dated row in
`docs/plans/20260801-mutation-sweep.md`'s progress table are statements about a past tree.
`tests/house_style.rs:3042` states the principle for the version guard: "The changelog and the
development history are dated records of versions that really shipped, and correcting a version
in a record would be falsifying it."

**Every new source-reading guard needs a companion, and the companion is most of the work.**
This project has recorded at least six distinct ways one goes vacuous: anchored on a line its
own green half deletes, answered by an import line naming two symbols, answered by the code
formatter, satisfied by a comment quoting its own marker, counting fixture strings inside a
test file, and disarmed by the workaround its own comment recommends. Copy
`test_the_version_reading_can_see_one_on_a_real_page`, which splices a violation into the real
file's real lines.

**A census that asserts a floor weakens the guard beside it.** `CLAUDE.md` records this: a
constant saying "at least 8 of these exist" stops being load-bearing at 9. If a provenance check
counts documents or claim sites, re-measure every guard record that reads that census.

**A commit adding a test to `contacts_sync.rs` or `calendar.rs` costs 77 or 69
re-measurements.** Both are named by more records than any other file. Where a new test can
honestly live in a smaller file, that is a real cost saving, and the project has already
recorded it as a planning decision.

**Do not pipe `scripts/check.sh` into anything whose exit status you then read.** Recorded in
`CLAUDE.md`; it has already put an unformatted tree onto a commit.

**Nothing else may be building while `scripts/guards.sh` runs.** `scripts/guards.sh:41`: "A
commit hook running the suite in the middle of one already reported three guards green that go
red on their own."

**`WIXEN_TEST_THREADS` applies to the guard runs and not to `scripts/check.sh`.** Measured
2026-08-31 and recorded in `CLAUDE.md`: under `--all-targets` the test term falls from 197s to
111s and the whole gate does not move, 335s against 353s.

**Every number this phase writes gets its date and its conditions, including the ones about the
phase's own machinery.** The gate itself is the cautionary example: a documents-only run was 36
seconds once and 2m56s another time, because the second followed commits that forced a clippy
rebuild.

---

## What I could not settle

1. **Whether the 2026-08-05 mutation run was whole-tree or covered `src/application` and
   `src/presentation` only.** `CLAUDE.md:544`, commit `9c8b9f9` and
   `scripts/mutants_report.py:6` say whole-tree; `docs/plans/20260801-mutation-sweep.md:307`
   says narrower. The run's output is gone. 1,470 mutants is well below any whole-tree figure
   recorded before or after, which favours the narrower reading, but that is an inference.
2. **What `CLAUDE.md`'s "471 of the 548 records name one file" counted.** Three readings tried,
   none reproduces 471 at the 548-record tree, while "74" in the same sentence reproduces
   exactly. The sentence names no command.
3. **How the em-dash guard is said to have caught a break in a planning file**, when
   `tests/house_style.rs` has never read `.planning/` (verified with `git log -S` in both
   spellings) and nothing else in `tests/`, `scripts/` or `.githooks/` reads it either.
4. **The real mutant count today.** `cargo mutants --list` answers it in seconds and I could not
   run cargo. Everything I give for the mutation run's size is an extrapolation from a line
   ratio.
5. **The real per-record guard cost on today's tree.** Extrapolated from two measured components
   taken on 2026-08-31 and about 2026-09-01. One record measured on the day would settle it, and
   costs two minutes.
6. **How many of the 615 records are stale.** The two prior sweeps disagree by a factor of
   twenty (23 in 208, 1 in 220), and which rate applies depends on how consistently the
   per-commit remedy was run through phases 3 and 4, which I did not measure.

---

## Decisions for Pratik

These change what gets built or when, and are not mine to settle.

**1. When does the twenty-hour guard sweep run, and who is at the keyboard when it reports?**

The job is unattended: it reports and continues, it restores each file after each break, and one
bad record no longer stops it. It needs a quiet machine, because a concurrent build has already
made it report three guards green that go red on their own. It writes nothing to
`guards/guards.toml`, so a run that dies leaves no half-finished artefact, only a log. Findings
are triaged afterwards, by hand, and each correction wants a second run of that record alone.

Four options, with what each costs:

| Option | Cost | What it risks |
|---|---|---|
| One overnight run, roughly 20 hours, machine otherwise idle | One night plus a triage day | If it dies at hour 18 there is no resume and the remainder has to be reconstructed from the log by hand |
| Split by suite: 571 `--lib` records in one run, the 44 integration records in another | Same total, two shorter jobs | The `--lib` job is still 19 hours, so this splits off the small half |
| Split into four runs of about 155 records, five hours each, by name list into `--remeasure` | Same total, four resumable chunks, plus one extra baseline run per chunk per suite | `--remeasure` also writes the counts for records that agreed, which an ordinary sweep deliberately does not |
| Ask for a resume flag first, one small change to `scripts/guards.py` | An hour or two of work before the sweep starts | Nothing, and it makes every future sweep cheaper |

My recommendation is the fourth then the first, but the trade is yours: a resume flag is work
this phase did not plan for, and the third option gets most of the benefit today.

**2. Does `.planning/` count as "the documentation" for criterion 3?**

`tests/house_style.rs` has never read it, and a `.planning/` change is currently checked by
nothing that opens it. `.planning/REQUIREMENTS.md` and the audit are full of numbers, and the
audit is the best-provenanced document in the repository. Including it means teaching `ours()`
about a directory whose files are written by a workflow, and the em-dash guard would then start
firing on planning documents. Excluding it means the documents that carry the most numbers stay
outside the rule.

**3. What answers criterion 2's filter number?**

Either write 200,000 rows into a `tempfile` `MessageCache` and time `search_messages` against
it, which measures the real filter path and needs no window, or drop the filter number and
record why. The sample mailbox cannot answer it either way, because its rows never reach SQLite
and mail has no in-memory filter.

**4. Which shape of check asserts that the virtual text callback issues no query?**

The type-level version is the strongest and cannot be recorded in `guards/guards.toml`, because
its break is a compile error and `scripts/guards.py` measures breaks by which tests go red. The
source-reading version is recordable and is the shape this project has most often watched go
vacuous. Doing both is my recommendation and costs a companion test.

**5. Two targets are likely to be missed rather than met, and criterion 6 says say why.**

`apply_sort` clones and sorts 200,000 rows while holding the UI state mutex. If the sort
measurement shows a multi-second freeze, the requirement's own `[S]` line calls that an
accessibility failure rather than a performance one, so the phase either grows a fix or records
a revised target with the reason. Worth deciding in advance which, because it changes the phase's
size.

---

*Research written 2026-09-06, read-only, from `main` at `48ace28` and the working tree of
branch `spelling-caret-walk` at `eda6ef7`. Readers used for the counts are saved beside this
file as `dist.py`, `numbers.py` and `nontest.sh`. Nothing in the repository was written or
built.*

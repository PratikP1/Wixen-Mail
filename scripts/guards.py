"""Take every recorded guard measurement again, at whatever HEAD is now.

Read `scripts/guards.sh` for why this exists. This file is the mechanism.

Never changes anything through git. Every file it is about to edit is copied
byte for byte into a scratch directory keyed by its whole path, and put back
from there whether the run finishes, fails or is interrupted. Two files in this
project are both called `calendar.rs`, so the key is the whole path and never
the basename. It reads git in exactly two places: `files_changed_since`, to work
out which records a branch could have disturbed, and
`the_guarded_files_git_sees_as_modified`, so that a resume refuses a tree a
killed run left broken rather than measuring over it. Until 2026-09-14 it read
git in one.

A sweep is hours and can be stopped and picked up: `--log` appends every line
to a file, `--resume` reads that file for the verdict after each `-- name` line
and measures only what has none, `--stop-after` makes a chunk a known size, and
`--wait-until-quiet` holds each record until no other cargo is building. The
`finally` that puts a broken file back runs on an interrupt and not on a
process-tree kill, which is what `--resume`'s refusal exists for.

Nothing that does not return costs more than itself. Every record, and every
suite's pre-read, is held to one wall clock, `--time-limit`, shared by its
build and its run; what passes it is killed with the processes cargo started
under it, reported unmeasured with the seconds it reached, and the run goes on.
The closing line counts what a run gave up on, so a run that hit the limit
cannot read as clean. Before that existed, run 35520204784 lost a whole shard
to one record.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECORD = ROOT / "guards" / "guards.toml"

# How many test threads the suite runs on, which is not a tuning preference but
# a measurement, and it is worth more than any other change made to this script.
#
# Measured on 2026-08-31, on a machine with 24 logical cores, running the
# library suite of 5,837 tests five times:
#
#     2 threads   131s
#     4 threads    88s
#     8 threads   106s
#    16 threads   164s
#    default(24)  196s
#
# The suite is contended rather than compute-bound, so the harness default of
# one thread per core is far from the best. Every guard record pays this once,
# so the setting multiplies through the whole sweep.
#
# Re-measured 2026-09-09 at `10effea`, 6,720 library tests on 24 cores, and the
# turning point has moved: 1 thread 375s, 2 threads 226s, 4 threads 161s,
# 8 threads 140s, 16 threads 234s. The best is now eight, not four, and the
# previous figures, taken 2026-08-30 over 5,837 tests, put it at four. So the
# default here was costing about 13% of every guard run, quietly, from the day
# the suite grew past it.
#
# Take that as the warning rather than as the new permanent answer. This number
# is a property of a machine and a suite size, both of which move, and nothing
# re-asks it. Whoever next finds a guard run slower than this comment implies
# should re-take the curve rather than trust it.
#
# What contends is still not diagnosed. Two conditions that were true when the
# old curve was taken are gone: `target/debug` had reached 786 GB, and it was
# being read by a virus scanner. Neither turned out to change the shape.
#
# Overridable rather than fixed, because the curve belongs to this machine: on a
# two-core CI runner the default is already below the turning point and forcing
# eight would be worse.
TEST_THREADS = os.environ.get("WIXEN_TEST_THREADS", "8")

# What one record really costs, said once so the two places that quote it cannot
# drift apart.
#
# Measured 2026-09-14 at `bb61e88e`, 8 threads, the library holding 7,245
# tests by `cargo test --lib -- --list`, nothing else building, the build warm:
# one record through `--remeasure` twice, the timing line below reporting
# rebuild 46 s and run 47 s, then rebuild 44 s and run 47 s, so 93 s and 91 s.
# A `--named-only` run of the same record was rebuild 44 s and run 3 s, 47 s.
# The row on docs/development/measurements.md carries the same figures.
#
# This replaces "about 95 seconds a record against 35", measured 2026-09-10 at
# `eda2719` as 29 s to rebuild and 66 s to run. The run fell to 47 s and the
# rebuild rose to 44 s, neither for a reason anybody has established, so the
# sum barely moved while both terms did; that is why the line prints them
# apart rather than only their sum. Every sweep estimate
# divides by this number, so it is the one figure in this file worth re-taking
# rather than inheriting: run one record and read the timing line.
THE_COST_OF_ASKING_PROPERLY = "about 92 seconds a record against 47."

# The wall clock one record gets, its build and its run together, and one
# suite's pre-read the same.
#
# 1,800 seconds, just under four times the 456 s the longest record to finish
# anywhere in run 35520204784 took (1800 / 456 is 3.95). Four times, because a
# record at four times the worst measured record is not slow, it is stuck.
#
# What it costs when records do hit it: a twenty-record shard in which two hit
# the limit is eighteen measured records at that sweep's 223 s mean plus two at
# 1,800 s, which is 7,614 s, about 127 minutes against the workflow's
# 360-minute cap.
#
# Why it exists. Run 35520204784, dispatched 2026-09-20 at `0ad66e48`, lost
# shard 40: it logged 2,637 s of measured work and then spent about 5 h 16 min
# inside one record, "the settings dialog is frozen while its pages are built",
# which printed a header and never a timing line, until the job's 360-minute
# cap ended the shard with nine records unmeasured. The longest shard that
# finished was 7,442 s, so the shard count was never the cause: nothing here
# bounded a record, and a record that does not return costs a whole shard
# whatever the shard count is.
#
# Overridable with --time-limit, because a slower machine moves the whole curve
# and the figures above belong to GitHub's Windows runners on one day.
THE_LONGEST_A_RECORD_MAY_TAKE = 1800

# How long the reap after a tree kill is given before this stops waiting for
# it. A kill that took the tree closes the pipes at once, so this is the
# reading that says it did not, rather than a wait that never ends.
THE_LONGEST_A_KILL_IS_GIVEN = 60

# `test <name> ... ok` or `... FAILED`, as the test harness writes it.
VERDICT = re.compile(r"^test (\S+) \.\.\. (ok|FAILED)$", re.M)

# A test attribute as it is really written: a line that is nothing else.
#
# Anchored at both ends, and the anchor is the whole difficulty. The other
# reading is answered by a mention: `#[test]` sits in nineteen doc comments in
# this tree explaining what a test does, so `grep -c` reports two for a file
# holding one test. A count answered by a mention rather than a use is the
# mistake this project has made seven times and it is always this one.
#
# Both spellings, because 573 of the tests here are `#[tokio::test]`. A reader
# that knew only the bare attribute would report zero for a file whose tests are
# all asynchronous, and zero is the answer that never disagrees with anything.
TEST_ATTRIBUTE = re.compile(r"^[ \t]*#\[(?:tokio::)?test\][ \t]*$", re.M)


@dataclass(frozen=True)
class Guard:
    name: str
    file: Path
    before: str
    after: str
    red: tuple[str, ...]
    # What to run to find out. The library, unless the record names an
    # integration test target instead.
    #
    # Not every rule this project guards can be checked from inside the
    # library. The house style rules read the tree as text: whether a page
    # claims a privacy property the code contradicts is a question about files,
    # and it is answered by a test in `tests/`, which `cargo test --lib` never
    # builds. Named here, a break to such a guard used to fail with "the test
    # harness never ran", which reads as a wrong name rather than as a runner
    # looking in the wrong place.
    #
    # Per guard rather than widening every guard to the whole suite. Widening
    # changes what "nothing else went red" means for all the records already
    # written, so every one of them would have to be measured again before the
    # run could be believed. A record that names its own target leaves every
    # other record measuring exactly what it measured when it was written.
    suite: tuple[str, ...] = ("--lib",)


@dataclass(frozen=True)
class Measured:
    """What a break really did, against what the record says it does."""

    stayed_green: list[str]
    also_went_red: list[str]

    def agrees_with_the_record(self) -> bool:
        return not self.stayed_green and not self.also_went_red


@dataclass(frozen=True)
class Budget:
    """One wall clock, set when a measurement starts and shared by its build
    and its run.

    A record that spends the whole of it building is stopped as surely as one
    that spends the whole of it running, which is the point: run 35520204784
    lost a shard to a record that printed a header and never a timing line, and
    nothing in this file bounded either term. One value and one start, not a
    clock per cargo call, because two clocks would hand each term the whole
    budget and a record could take twice what it was given.

    >>> budget = Budget(seconds=1800, started=1000.0)
    >>> budget.left(at=1000.0)
    1800.0
    >>> budget.left(at=1740.5)
    1059.5
    >>> budget.spent(at=1740.5)
    740.5

    What is left of a spent budget is nothing and never less, because this
    value is handed straight to a timeout:

    >>> budget.left(at=2900.0)
    0.0
    """

    seconds: int
    started: float

    def left(self, at: float | None = None) -> float:
        at = time.monotonic() if at is None else at
        return max(0.0, self.started + self.seconds - at)

    def spent(self, at: float | None = None) -> float:
        at = time.monotonic() if at is None else at
        return at - self.started

    @staticmethod
    def starting_now(seconds: int) -> "Budget":
        return Budget(seconds=seconds, started=time.monotonic())


class Wrong(Exception):
    """A guard is not what the record says it is."""


class GaveUp(Wrong):
    """A record, or a suite's pre-read, that did not return inside its budget.

    A `Wrong` and not a hierarchy of its own, because everything that puts a
    guarded file back is already written around `Wrong`: `measure`'s `finally`
    restores the bytes whatever leaves the `try`, and the loop prints the
    message in one of the shapes `verdicts_in` reads. What the separate class
    buys is that the loop can count these apart from the records it really
    judged, so the closing line can say a run gave up on something and a run
    that hit the limit cannot read as clean.
    """


def read_record() -> list[Guard]:
    if not RECORD.exists():
        raise Wrong(f"{RECORD} is not there, so there is nothing to measure")
    written = tomllib.loads(RECORD.read_text(encoding="utf-8"))
    guards = [
        Guard(
            name=entry["name"],
            file=ROOT / entry["file"],
            before=entry["before"],
            after=entry["after"],
            red=tuple(entry["red"]),
            suite=("--test", entry["suite"]) if "suite" in entry else ("--lib",),
        )
        for entry in written.get("guard", [])
    ]
    if not guards:
        # An empty record must fail rather than report a clean run. A check
        # that passes when there is nothing to check is the kind that gets
        # believed for months.
        raise Wrong(f"{RECORD} records no guards at all")
    for guard in guards:
        if not guard.red:
            raise Wrong(f"{guard.name}: names no test that should go red")
        if not guard.file.exists():
            raise Wrong(f"{guard.name}: {guard.file} is not there")
    return guards


def module_of(path: str) -> str | None:
    """The module a changed source file's tests live under, if it has one.

    A unit test lives beside the code it covers, so the tests belonging to
    `src/a/b.rs` are named `a::b::...`. Anything that is not library source has
    no module and answers None.

    >>> module_of("src/application/allowed.rs")
    'application::allowed'
    >>> module_of("src/data/message_cache/mod.rs")
    'data::message_cache'
    >>> module_of("src/lib.rs") is None
    True
    >>> module_of("docs/changelog.md") is None
    True
    >>> module_of("tests/wired.rs") is None
    True
    """
    path = path.replace("\\", "/")
    if not path.startswith("src/") or not path.endswith(".rs"):
        return None
    module = path[len("src/") : -len(".rs")]
    if module.endswith("/mod"):
        module = module[: -len("/mod")]
    if module == "lib":
        return None
    return module.replace("/", "::")


def could_have_gone_stale(guard: "Guard", changed: list[str]) -> bool:
    """Whether a change could have made this record wrong, either way round.

    A record goes stale in two directions and only one of them is obvious.

    **The break stops applying.** The guarded file changed, so the exact text
    the record replaces may have moved or gone. This one announces itself the
    next time anybody runs the record.

    **The red set grows.** A test was added that reaches the rule the record is
    about, so the record now names too few and *nothing fails*. That is the
    silent one, and it is the direction that caught this project four times in
    one phase. New tests live in the files a change touched, so a record whose
    red set already names a test in one of those modules is a record that
    change could have widened.

    This is a candidate set, not a proof. A new test in a module the record has
    never named can still redden it, and no reading of the record can predict
    that. Only the full sweep can, and the full sweep is hours.

    >>> about_allowed = Guard(
    ...     name="the constant that changes nothing still reads mail",
    ...     file=ROOT / "src/application/allowed.rs",
    ...     before="reading: true",
    ...     after="reading: false",
    ...     red=("application::allowed::tests::test_a",
    ...          "application::mail_controller::tests::test_b"),
    ... )

    The guarded file itself changed, so the break may no longer apply:

    >>> could_have_gone_stale(about_allowed, ["src/application/allowed.rs"])
    True

    A different file changed, and this record already names a test in it. New
    tests there could reach the same rule, which is the silent direction:

    >>> could_have_gone_stale(about_allowed, ["src/application/mail_controller.rs"])
    True

    Nothing this record has ever mentioned:

    >>> could_have_gone_stale(about_allowed, ["src/presentation/wx_compose.rs"])
    False
    >>> could_have_gone_stale(about_allowed, ["docs/changelog.md"])
    False

    A record measured against an integration target, and that target changed:

    >>> about_house_style = Guard(
    ...     name="no page names a version the code does not ship",
    ...     file=ROOT / "README.md",
    ...     before="a",
    ...     after="b",
    ...     red=("test_no_dashes_that_should_be_punctuation",),
    ...     suite=("--test", "house_style"),
    ... )
    >>> could_have_gone_stale(about_house_style, ["tests/house_style.rs"])
    True
    >>> could_have_gone_stale(about_house_style, ["tests/wired.rs"])
    False
    """
    paths = [path.replace("\\", "/") for path in changed]
    guarded = str(guard.file.relative_to(ROOT)).replace("\\", "/")
    if guarded in paths:
        return True

    for path in paths:
        module = module_of(path)
        if module and any(name.startswith(f"{module}::") for name in guard.red):
            return True
        if guard.suite[0] == "--test" and path == f"tests/{guard.suite[1]}.rs":
            return True
    return False


def tests_in(text: str) -> int:
    """How many test functions a file of Rust holds.

    >>> tests_in("#[test]\\nfn test_a() {}\\n")
    1
    >>> tests_in("    #[tokio::test]\\n    async fn test_b() {}\\n")
    1

    And the mention, which is what the unanchored reading counts:

    >>> tests_in("/// Runs under `#[test]`, as this sentence says.\\n")
    0
    >>> tests_in("//! One `#[test]` function.\\n#[test]\\nfn test_a() {}\\n")
    1
    """
    return len(TEST_ATTRIBUTE.findall(text))


def the_file_a_test_lives_in(test: str, suite: tuple[str, ...]) -> str | None:
    """The file a named test lives in, as a path from the repository root.

    A test named `a::b::tests::test_c` lives in `src/a/b.rs`, but how many
    segments sit between the file and the test's own name is not fixed. The test
    module is usually `tests` and often is not, and it can be nested. So the
    file is the longest prefix that is really a file, tried longest first, and a
    directory module is reached through its `mod.rs`.

    >>> the_file_a_test_lives_in("application::calendar::tests::test_a", ("--lib",))
    'src/application/calendar.rs'

    A directory module, which has no file of its own name:

    >>> the_file_a_test_lives_in("data::message_cache::tests::test_a", ("--lib",))
    'src/data/message_cache/mod.rs'

    A test module named for what it is about rather than `tests`:

    >>> the_file_a_test_lives_in(
    ...     "application::sending_later::what_undo_send_is_about::test_a", ("--lib",)
    ... )
    'src/application/sending_later.rs'

    A record measured against an integration target names its own suite, and
    those tests have no module path at all:

    >>> the_file_a_test_lives_in("test_no_dashes", ("--test", "house_style"))
    'tests/house_style.rs'

    And a name nothing in the tree answers:

    >>> the_file_a_test_lives_in("nowhere::at::all::test_a", ("--lib",)) is None
    True
    """
    if suite[0] == "--test":
        return f"tests/{suite[1]}.rs"
    parts = test.split("::")
    for cut in range(len(parts) - 1, 0, -1):
        stem = "/".join(parts[:cut])
        for candidate in (f"src/{stem}.rs", f"src/{stem}/mod.rs"):
            if (ROOT / candidate).exists():
                return candidate
    return None


def what_the_tree_holds_now(guard: Guard) -> list[tuple[str, int]]:
    """For every file this record is about, how many tests it holds.

    This is what a record writes down so that a source read can notice the tree
    moving underneath it. A file gaining a test is how a record comes to name
    too few, which is the direction that never announces itself.

    The files its red list names, and the file it breaks. A unit test lives
    beside what it covers, so a test arriving in the guarded file is at least as
    likely to reach the break as one arriving anywhere else, and 83 of the 683
    records break a Rust file no test in their red list lives in. Re-measured
    2026-09-10 with this module's own `the_file_a_test_lives_in`, because two
    hand derivations of the same figure answered 219 and 183 by mishandling
    `mod.rs` and the suite short-circuit. The old figure here was 34 of 548. "the question
    about which days focuses the answer it ticks" breaks
    `src/presentation/wx_which_days.rs` and every test it names is in
    `wx_calendar.rs`, so without this a test written next to the code that
    record is about would move nothing.

    Measured before it was added, because the cost of watching more files is
    flagging more records: those 34 gain one entry each, and the file that
    already flags the most records is unchanged by it.

    Rust files only. A handful of records guard a document, and counting the
    tests in `README.md` is a number that can never move.
    """
    guarded = str(guard.file.relative_to(ROOT)).replace("\\", "/")
    about = [the_file_a_test_lives_in(test, guard.suite) for test in guard.red]
    about.append(guarded if guarded.endswith(".rs") else None)

    seen: dict[str, int] = {}
    for where in about:
        if where is None or where in seen:
            continue
        seen[where] = tests_in((ROOT / where).read_text(encoding="utf-8"))
    return sorted(seen.items())


def with_its_counts(block: list[str], counts: list[tuple[str, int]]) -> list[str]:
    """One record's lines, with `tests_last_seen` written under its red list.

    Under the list it is about, and never at the end of the block: the lines
    after a record's last key are the comment introducing the next record, and
    a key written after those would read as belonging to the wrong one.

    >>> with_its_counts(
    ...     ["[[guard]]", "red = [", '    "a",', "]"], [("src/a.rs", 3)]
    ... )
    ['[[guard]]', 'red = [', '    "a",', ']', 'tests_last_seen = [', '    { file = "src/a.rs", tests = 3 },', ']']

    A red list written on one line, which some records use:

    >>> with_its_counts(['red = ["a"]'], [("src/a.rs", 1)])
    ['red = ["a"]', 'tests_last_seen = [', '    { file = "src/a.rs", tests = 1 },', ']']

    An existing count is replaced rather than added to, so running this twice
    leaves what running it once left:

    >>> with_its_counts(
    ...     ['red = ["a"]', "tests_last_seen = [", '    { file = "src/a.rs", tests = 2 },', "]"],
    ...     [("src/a.rs", 3)],
    ... )
    ['red = ["a"]', 'tests_last_seen = [', '    { file = "src/a.rs", tests = 3 },', ']']

    A comment following the record keeps its place, because it belongs to
    whatever comes next rather than to this:

    >>> with_its_counts(['red = ["a"]', "", "# about the next one"], [("src/a.rs", 1)])[-2:]
    ['', '# about the next one']

    A count already written on one line closes on that line, so nothing after
    it is swallowed. Read as an opening bracket waiting for a `]` of its own,
    it eats the rest of the block: the blank line before the next record, and
    for the last record in the file the empty string that the final newline
    leaves behind. That really happened. Rewriting the last record dropped the
    file's trailing newline, and what said so was a fixture-sanity case in
    `check.test.sh` reporting that a copy of the record file differed by no
    lines, which reads as a broken fixture rather than as this:

    >>> with_its_counts(
    ...     ['red = ["a"]', 'tests_last_seen = [{ file = "src/a.rs", tests = 2 }]', ""],
    ...     [("src/a.rs", 3)],
    ... )[-1:]
    ['']
    """
    kept: list[str] = []
    dropping = False
    for line in block:
        if line.startswith("tests_last_seen = ["):
            dropping = not line.rstrip().endswith("]")
            continue
        if dropping:
            dropping = line != "]"
            continue
        kept.append(line)

    written = ["tests_last_seen = ["]
    written += [f'    {{ file = "{where}", tests = {count} }},' for where, count in counts]
    written.append("]")

    for at, line in enumerate(kept):
        if not line.startswith("red = "):
            continue
        end = at
        if not line.rstrip().endswith("]"):
            end = next(i for i in range(at + 1, len(kept)) if kept[i] == "]")
        return kept[: end + 1] + written + kept[end + 1 :]
    raise Wrong("a record with no red list, which read_record already refuses")


def rewritten_with_counts(
    raw: str, guards: list[Guard], counts: dict[str, list[tuple[str, int]]]
) -> str:
    """The whole record file, with the named records' counts written down.

    Everything else byte for byte. The comments in that file carry the reasoning
    for every record, and several of them are the only account of a defect that
    was shipped, so a rewrite that parsed and dumped would cost far more than
    this check is worth. This is a line edit, and the line endings the file
    already uses are the ones it keeps.

    >>> one = Guard("only me", Path("a.rs"), "a", "b", ("x",))
    >>> print(rewritten_with_counts(
    ...     '# a note\\n[[guard]]\\nname = "only me"\\nred = ["x"]\\n',
    ...     [one],
    ...     {"only me": [("src/a.rs", 4)]},
    ... ))
    # a note
    [[guard]]
    name = "only me"
    red = ["x"]
    tests_last_seen = [
        { file = "src/a.rs", tests = 4 },
    ]
    <BLANKLINE>

    A record nobody asked about is not touched:

    >>> rewritten_with_counts(
    ...     '[[guard]]\\nname = "only me"\\nred = ["x"]\\n', [one], {}
    ... )
    '[[guard]]\\nname = "only me"\\nred = ["x"]\\n'
    """
    newline = "\r\n" if "\r\n" in raw else "\n"
    lines = raw.split(newline)
    starts = [at for at, line in enumerate(lines) if line.rstrip() == "[[guard]]"]
    if len(starts) != len(guards):
        raise Wrong(
            f"the record file holds {how_many(len(starts), 'guard header')} and "
            f"{how_many(len(guards), 'record')} were read out of it, so the two "
            "readings do not pair up and nothing here may be written."
        )

    out = lines[: starts[0]]
    for n, start in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(lines)
        block = lines[start:end]
        guard = guards[n]
        if guard.name in counts:
            # The headers and the records are one list read two ways, so this
            # pairs them by position. Checked rather than trusted: anything that
            # made the two readings disagree would land every rewrite after it
            # on the wrong record, silently.
            if f'name = "{guard.name}"' not in block:
                raise Wrong(
                    f"the {n + 1}th record in the file is not {guard.name!r}, so "
                    "the two readings of it disagree and nothing may be written."
                )
            block = with_its_counts(block, counts[guard.name])
        out += block
    return newline.join(out)


def write_down_the_counts(
    guards: list[Guard], counts: dict[str, list[tuple[str, int]]]
) -> None:
    """Put the counts into the record file, keeping the bytes around them.

    Bytes rather than text, for the reason `measure` gives about the tree it
    breaks: a text-mode write turns every line of this file into CRLF on
    Windows, which git records as a change to all ten thousand of them.
    """
    raw = RECORD.read_bytes().decode("utf-8")
    RECORD.write_bytes(rewritten_with_counts(raw, guards, counts).encode("utf-8"))


def why_the_counts_may_not_be_written(named_only: bool) -> str | None:
    """Why this run may not write down the tree it measured against, if it may not.

    A count means "this record was checked against this tree", and only a run
    that asked the whole question earns one. `--named-only` runs the modules a
    record's own tests live in and never asks whether anything else went red, so
    a record naming too few reads as correct to it. Writing under that run
    stamps "checked" on a record nothing has checked in the direction 21 of the
    23 records found wrong on 2026-09-01 were wrong in, and the count check then
    stays quiet about it for ever. The closing message already says the run did
    not ask, printed after the write has claimed it did.

    The write is skipped and the flag pair is not refused, and that is the
    load-bearing half. Refusing it would take away the only way to aim a run at
    an exact set of records, which is what somebody needs when a shared file
    moves: ledger 197 records a change shipped without a test on 2026-09-08
    because a test of it would have lived in `src/presentation/managers.rs`,
    which 41 records fingerprint, at roughly 80 minutes of re-measurement for a
    one-line change. A remedy nobody can afford is a remedy nobody runs.

    >>> why_the_counts_may_not_be_written(named_only=False) is None
    True
    >>> print(why_the_counts_may_not_be_written(named_only=True))
    This run was filtered with --named-only, so no count was written down.
    <BLANKLINE>
    A count says a record was checked against this tree, and this run asked
    half the question: it ran only the modules the record's tests live in, so a
    test elsewhere that the break also reddens was neither run nor reported.
    <BLANKLINE>
    Re-run the same --remeasure selection without --named-only to earn it.
    """
    if not named_only:
        return None
    return (
        "This run was filtered with --named-only, so no count was written down.\n"
        "\n"
        "A count says a record was checked against this tree, and this run asked\n"
        "half the question: it ran only the modules the record's tests live in, so a\n"
        "test elsewhere that the break also reddens was neither run nor reported.\n"
        "\n"
        "Re-run the same --remeasure selection without --named-only to earn it."
    )


def the_filter_that_forbids_recounting_everything(
    only: str | None,
    touched_by: str | None,
    remeasure: list[str] | None,
    shard: str | None = None,
) -> str | None:
    """Which narrowing was asked for alongside --recount-everything, if any.

    That mode writes a fingerprint on every record in the file, having measured
    nothing, and it returns before any filter is applied. So asking for one
    record and a recount rewrites all of them, including the records already
    known to be short, whose whole value is that nothing has stamped them yet.
    One mistyped invocation is the whole file.

    Refused rather than narrowed, which is the smaller and safer of the two
    ways out. Honouring the filter would give the mode a second meaning that
    reads identically at the call site, and its own message already says a
    count written this way is the weaker claim: no test has been added to these
    files since somebody looked, never that a record is right. The weaker claim
    over a narrower set is still the weaker claim.

    >>> the_filter_that_forbids_recounting_everything(None, None, None) is None
    True
    >>> the_filter_that_forbids_recounting_everything("deletion", None, None)
    'a name to match: deletion'
    >>> the_filter_that_forbids_recounting_everything(None, "main", None)
    '--touched-by main'
    >>> the_filter_that_forbids_recounting_everything(None, None, ["a"])
    '--remeasure with 1 name'
    >>> the_filter_that_forbids_recounting_everything(None, None, ["a", "b"])
    '--remeasure with 2 names'

    All of them, because a run that asked for two narrowings has two things to
    take out, and being told about one of them costs a second round trip:

    >>> the_filter_that_forbids_recounting_everything("deletion", "main", None)
    'a name to match: deletion, --touched-by main'

    A shard is a narrowing too, and the rewrite would refuse it anyway with a
    message about two readings of the file disagreeing, which reads as a
    broken file rather than as this:

    >>> the_filter_that_forbids_recounting_everything(None, None, None, "3/41")
    '--shard 3/41'
    """
    asked_for: list[str] = []
    if only:
        asked_for.append(f"a name to match: {only}")
    if touched_by:
        asked_for.append(f"--touched-by {touched_by}")
    if remeasure:
        asked_for.append(f"--remeasure with {how_many(len(remeasure), 'name')}")
    if shard:
        asked_for.append(f"--shard {shard}")
    return ", ".join(asked_for) or None


def files_changed_since(ref: str) -> list[str]:
    """What this branch has changed, as paths relative to the repository root.

    The one place this script reads git. It still changes nothing through git:
    a broken file is put back from the bytes that were copied aside, never by
    checking it out, for the reason the module docstring gives.
    """
    finished = subprocess.run(
        ["git", "diff", "--name-only", f"{ref}...HEAD"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if finished.returncode != 0:
        raise Wrong(
            f"git could not say what changed since {ref!r}:\n"
            f"{finished.stderr.strip()}"
        )
    changed = [line.strip() for line in finished.stdout.splitlines() if line.strip()]
    if not changed:
        raise Wrong(
            f"nothing has changed since {ref!r}, so there is no branch here to "
            "check. Name the ref this branch came from."
        )
    return changed


def why_no_test_was_named(status: int, said: str) -> str:
    """Why a run named no test, saying only the part that is known.

    It fails two ways and the wording named one of them for both: a run that
    built and named no test read as a break that did not build, which is a
    cause nobody established. What separates them is the status cargo exited
    with, and neither reading is a guess.

    >>> print(why_no_test_was_named(101, "error[E0308]: mismatched types"))
    the break did not build, so no test ran. cargo exited 101 and said:
    error[E0308]: mismatched types
    >>> print(why_no_test_was_named(0, "running 0 tests"))
    the break built and the run named no test. cargo exited 0 and said:
    running 0 tests

    And the case that made this worth writing down. Cargo said nothing at all,
    so the old line ended at a colon with nothing after it, under a cause it
    had not established:

    >>> print(why_no_test_was_named(1, "  \\n  "))
    the break did not build, so no test ran. cargo exited 1 and said nothing at all.
    """
    which = (
        "the break built and the run named no test"
        if status == 0
        else "the break did not build, so no test ran"
    )
    if not said.strip():
        return f"{which}. cargo exited {status} and said nothing at all."
    return f"{which}. cargo exited {status} and said:\n{said[-4000:]}"


def the_filters_for(red: tuple[str, ...]) -> list[str]:
    """One filter per module the record's tests live in, deduplicated.

    A filter is a substring the harness matches against a test's whole path, so
    the module prefix reaches every test in it. Grouping by module rather than
    naming each test means a test *added* to one of those modules is still run
    and can still be reported as unnamed, which is the one part of the second
    direction a filtered run keeps.

    >>> the_filters_for(("a::b::tests::test_one", "a::b::tests::test_two"))
    ['a::b::tests::']
    >>> the_filters_for(("a::b::tests::test_one", "c::d::tests::test_two"))
    ['a::b::tests::', 'c::d::tests::']
    >>> the_filters_for(("test_bare",))
    ['test_bare']
    """
    seen: list[str] = []
    for name in red:
        prefix = name.rsplit("::", 1)[0] + "::" if "::" in name else name
        if prefix not in seen:
            seen.append(prefix)
    return seen


def run_the_whole_suite(
    suite: tuple[str, ...],
    filters: list[str] | None = None,
    budget: Budget | None = None,
) -> dict[str, str]:
    """Every test in one suite, and whether it passed. One build, one run.

    The whole suite and not only the tests a record names. Running the named
    ones answers "would these go red", and leaves the question that matters
    just as much unasked: did anything else. A record naming eight tests for a
    break that reddens seventeen reads as a guard on eight, the other nine are
    guarding something nobody wrote down, and nothing ever says so. That is the
    thing `guards/guards.toml` exists to stop, and it happened to a record in
    that file.

    It costs the whole suite per guard rather than a handful of tests. That is
    the price of the answer; there is no way to learn what a break reddens
    without running everything it could redden.

    Which suite is the guard's own, for the reason written on `Guard.suite`.
    Almost always the library.

    **`filters` narrows it, and it does not answer the same question. Measured,
    and the measurement is why this mode is a pre-filter and not a sweep.**

    The arithmetic looked good on 2026-09-02, when this was measured: the
    rebuild a break forced was 23 seconds and the library was 89, so filtering
    would have taken that day's 220-record sweep from 6.8 hours to about 88
    minutes. Both terms have moved since and the runner now prints today's
    pair after every record; the rate row on `docs/development/measurements.md`
    is where the current pair is, and the argument below does not depend on
    the figures. Two things spoil it.

    The known one: it cannot see a test in a module no filter reaches, and 21 of
    the 23 records the 2026-09-01 sweep found wrong were wrong in exactly that
    direction.

    The one that was not predicted: **filtered and unfiltered runs of the same
    break disagree.** On the first record tried, "the constant that changes
    nothing still reads mail", the unfiltered run had all six named tests red
    and nothing else, and the filtered run had one of the six green:
    `application::mail_controller::against_a_server_that_answers::test_a_command_that_names_a_different_folder_opens_that_one_first`
    fails under the break when the whole library runs and passes under the same
    break when only its own modules do. So some tests here fail only in company,
    and a filtered run reports a correct record as broken.

    A false alarm is the safer direction, which is why this stays: it is cheap,
    and anything it flags can be confirmed unfiltered. But it must never be read
    as a sweep, and a green answer from it covers one direction of one question.
    """
    # After the separator, not before it. `cargo test` takes one positional
    # filter and refuses a second; the harness behind it takes as many as you
    # like. The first spelling failed with cargo's own usage message, which
    # reads like a bad flag rather than like one filter too many.
    filtered = list(filters or [])
    # Two invocations rather than one, so the two terms of a record's cost are
    # timed apart: `cargo test --no-run` is the rebuild, and the run that
    # follows finds nothing to build and is the run. The split costs one
    # fingerprint check, under a second, and the alternative was reading
    # cargo's stderr as it streamed for its `Running` line, which needs a
    # thread per pipe and a second way for the capture to come back empty.
    #
    # The budget is the caller's, one wall clock across both, so a record that
    # spends it all here in the rebuild is stopped as surely as one that spends
    # it all in the run below.
    started = time.monotonic()
    built = cargo("test", *suite, "--no-run", budget=budget)
    rebuilt_at = time.monotonic()
    if built.returncode != 0:
        raise Wrong(why_no_test_was_named(built.returncode, built.stdout + built.stderr))
    finished = cargo(
        "test", *suite, "--", f"--test-threads={TEST_THREADS}", *filtered, budget=budget
    )
    ran_at = time.monotonic()
    print(the_timing_line(rebuilt_at - started, ran_at - rebuilt_at), flush=True)

    said = finished.stdout + finished.stderr
    verdicts = {name: verdict for name, verdict in VERDICT.findall(said)}
    if not verdicts:
        raise Wrong(why_no_test_was_named(finished.returncode, said))
    return verdicts


def the_kill_that_takes_the_tree(pid: int) -> list[str]:
    """The command that stops a process and everything it started.

    `Popen.kill` is `TerminateProcess` on that one process. On Windows the
    rustc children cargo spawned outlive it, and three things follow: they go
    on building, they hold open the pipes this run is reading so the reap after
    the kill never returns, and `--wait-until-quiet` then waits for a build
    nothing will end. `taskkill /T` walks the tree.

    >>> the_kill_that_takes_the_tree(1234)
    ['taskkill', '/PID', '1234', '/T', '/F']
    """
    return ["taskkill", "/PID", str(pid), "/T", "/F"]


def the_line_about_what_the_kill_left(alive: list[str]) -> str:
    """What to say after a tree kill, so a run that left a build behind says so.

    Four spaces and not three. `verdicts_in` reads the first three-space line
    under a record as that record's verdict, and this line is part of a message
    printed there, so at three spaces it would be read as one.

    >>> print(the_line_about_what_the_kill_left([]))
        the kill took the tree: no cargo or rustc is building now
    >>> print(the_line_about_what_the_kill_left(["rustc.exe 1240"]))
        still building after the kill, so --wait-until-quiet will wait for it: rustc.exe 1240
    >>> print(the_line_about_what_the_kill_left(["cargo.exe 1234", "rustc.exe 1240"]))
        still building after the kill, so --wait-until-quiet will wait for them: cargo.exe 1234, rustc.exe 1240
    """
    # Both ways written out rather than one built from parts, for the reason
    # `how_many` exists: this project has already read out "1 changes are
    # waiting here" to somebody.
    if not alive:
        return "    the kill took the tree: no cargo or rustc is building now"
    if len(alive) == 1:
        return (
            "    still building after the kill, so --wait-until-quiet will "
            f"wait for it: {alive[0]}"
        )
    return (
        "    still building after the kill, so --wait-until-quiet will wait "
        f"for them: {', '.join(alive)}"
    )


# The opener of the line a record or a pre-read given up on at its time limit
# prints. Two readings answer it differently on purpose. `the_verdict_on` reads
# it as one more thing that could not be measured, so nothing that reads a log
# line in isolation refuses a log holding one. `verdicts_in`, which is what
# --resume uses, drops the name instead, so the next run measures that record
# again rather than inheriting a verdict nobody took.
GIVEN_UP_ON = ": it did not return within its budget of "


def why_it_was_given_up_on(
    budget: int, spent: float, doing: str, alive: list[str]
) -> str:
    """Why a record or a pre-read was given up on, in the shape the log
    readings know.

    The first line carries the opener above, because that is the line
    `verdicts_in` judges a record by; what follows is for a person. The caller
    puts the record's name, or the suite's, in front of it.

    >>> print(why_it_was_given_up_on(1800, 1801.4, "building", []))
    it did not return within its budget of 1800 s, and was still building when the budget ran out at 1801 s.
        the kill took the tree: no cargo or rustc is building now
    This was not measured, so it has no verdict and a resume takes it again.

    >>> print(why_it_was_given_up_on(60, 63.2, "running", ["rustc.exe 1240"]))
    it did not return within its budget of 60 s, and was still running when the budget ran out at 63 s.
        still building after the kill, so --wait-until-quiet will wait for it: rustc.exe 1240
    This was not measured, so it has no verdict and a resume takes it again.
    """
    return (
        f"it did not return within its budget of {budget} s, and was still "
        f"{doing} when the budget ran out at {round(spent)} s.\n"
        f"{the_line_about_what_the_kill_left(alive)}\n"
        "This was not measured, so it has no verdict and a resume takes it "
        "again."
    )


def the_line_for_a_record_whose_pre_read_expired(
    suite: tuple[str, ...], ran_out: GaveUp
) -> str:
    """What is printed under each record of a suite whose pre-read expired.

    One line per record and not one for the suite, because a resume measures
    every record with no verdict and `verdicts_in` reads a `-- name` block: a
    suite-shaped line would leave the records themselves saying nothing, and
    the next run would inherit silence rather than a reason.

    >>> print(the_line_for_a_record_whose_pre_read_expired(
    ...     ("--lib",),
    ...     GaveUp(why_it_was_given_up_on(1800, 1800.2, "running", [])),
    ... ))
       the pre-read of --lib: it did not return within its budget of 1800 s, and was still running when the budget ran out at 1800 s.
        the kill took the tree: no cargo or rustc is building now
    This was not measured, so it has no verdict and a resume takes it again.

    And the wording tied to the reading that has to know it, over the two
    records such a line would be written under:

    >>> expired = the_line_for_a_record_whose_pre_read_expired(
    ...     ("--test", "house_style"),
    ...     GaveUp(why_it_was_given_up_on(1800, 1801.0, "building", [])),
    ... )
    >>> verdicts_in("-- first\\n" + expired + "\\n-- second\\n" + expired + "\\n")
    {}
    """
    return f"   the pre-read of {' '.join(suite)}: {ran_out}"


def stop_the_tree(process: "subprocess.Popen[str]") -> list[str]:
    """Stop a process and everything it started, then say what is still
    building.

    Read rather than trusted: whether the kill took the tree is answered by a
    listing and not by the kill's own exit status, and a survivor is what would
    make `--wait-until-quiet` wait for a build nothing will end. What this
    cannot do is tell this run's survivors from a hook's build, so it names
    every cargo and rustc alive and leaves the reader to decide.
    """
    try:
        subprocess.run(
            the_kill_that_takes_the_tree(process.pid),
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        # No taskkill, so not Windows, which is where the sweep runs. The
        # direct child at least, rather than nothing.
        process.kill()
    # The output is not wanted, the reap is: until the pipes close this
    # process holds handles on a tree it has given up on.
    try:
        process.communicate(timeout=THE_LONGEST_A_KILL_IS_GIVEN)
    except subprocess.TimeoutExpired:
        pass
    try:
        return foreign_builds_in(what_is_running(), set())
    except Wrong:
        # `what_is_running` refuses a machine with no tasklist. That is a fact
        # about the machine and not about the kill, and a record already given
        # up on must not be turned into a run that ends.
        return []


def run_to_completion(
    command: list[str], budget: Budget | None, doing: str
) -> subprocess.CompletedProcess[str]:
    """Run a command with its output captured, and stop it if the budget runs
    out.

    `subprocess.run(timeout=...)` would be shorter and would not do. On expiry
    it kills the direct child and then calls `communicate()`, which waits for
    the pipes to close, and on Windows cargo's rustc children inherit those
    pipes and outlive the kill. So the wait after the kill is unbounded exactly
    when the kill was needed. This spawns, waits for what the budget has left,
    walks the tree, and only then reaps.

    A command that will not finish, given what is left of a nearly spent
    budget. The seconds it reached are cut off the line here because they are
    a clock reading and this is an example:

    >>> try:
    ...     run_to_completion(
    ...         [sys.executable, "-c", "import time; time.sleep(30)"],
    ...         Budget(seconds=1, started=time.monotonic() - 0.8),
    ...         "running",
    ...     )
    ... except GaveUp as ran_out:
    ...     print(str(ran_out).splitlines()[0].split(" when the budget")[0])
    it did not return within its budget of 1 s, and was still running

    One that finishes inside its budget, and one with no budget at all, which
    is what an ordinary run of this script asks for:

    >>> run_to_completion([sys.executable, "-c", "print('done')"], Budget(seconds=60, started=time.monotonic()), "running").stdout.strip()
    'done'
    >>> run_to_completion([sys.executable, "-c", "print('done')"], None, "running").returncode
    0
    """
    started = subprocess.Popen(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        # Both named, and neither is a preference.
        #
        # `text=True` alone decodes with the locale codec, which on this
        # machine is cp1252. A test name or a panic message carrying a byte
        # cp1252 cannot decode kills the reader thread, and `stdout` is then
        # never assigned: that is the `NoneType + str` that ended a sweep on
        # its 122nd record, and the decode traceback sits in that run's own log
        # underneath the failure it caused.
        #
        # `errors="replace"` rather than strict, because a mangled character in
        # a panic message must not cost an hour of measuring. What this reads
        # out are test paths, and those are ASCII.
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    try:
        said, complained = started.communicate(
            timeout=None if budget is None else budget.left()
        )
    except subprocess.TimeoutExpired:
        raise GaveUp(
            why_it_was_given_up_on(
                budget.seconds, budget.spent(), doing, stop_the_tree(started)
            )
        ) from None
    return subprocess.CompletedProcess(command, started.returncode, said, complained)


def cargo(*arguments: str, budget: Budget | None = None) -> subprocess.CompletedProcess[str]:
    """cargo with these arguments, its output captured and never missing.

    The budget is the record's own, one wall clock across its build and its
    run, and `--no-run` is what tells the two apart in the line a given-up
    record prints. A budget already spent stops the call rather than letting it
    run unbounded, and that is the example that reddens if the budget ever
    stops being handed to this call:

    >>> try:
    ...     cargo("--version", budget=Budget(seconds=1, started=time.monotonic() - 100))
    ...     print("cargo ran unbounded, so no budget reached this call")
    ... except GaveUp:
    ...     print("the budget reached the cargo call")
    the budget reached the cargo call
    """
    finished = run_to_completion(
        ["cargo", *arguments],
        budget,
        # Which term of the record's cost was running when the budget ran out,
        # read from the call itself: `run_the_whole_suite` builds with
        # `--no-run` and then runs, so this is the rebuild exactly when that
        # flag is there.
        "building" if "--no-run" in arguments else "running",
    )
    # Both halves can come back as None, which is not what the documentation
    # for a captured run says and was seen anyway: a sweep of 208 records
    # died on its 122nd with `NoneType + str` after about an hour of work.
    # Whatever causes it is not diagnosed, so it is reported rather than
    # smoothed into an empty string, for the same reason `scripts/mutants.sh`
    # refuses a run whose compiler never started: a run that captured nothing
    # learned nothing, and must not be told apart from a clean result only by
    # whoever happens to read the log.
    if finished.stdout is None or finished.stderr is None:
        raise Wrong(
            "cargo ran and this captured none of its output, so nothing can be "
            "read from it. stdout is "
            f"{'missing' if finished.stdout is None else 'present'} and stderr "
            f"is {'missing' if finished.stderr is None else 'present'}, and "
            f"cargo exited {finished.returncode}.\nThis guard was not measured. "
            "Run it again on its own before believing anything about it."
        )
    return finished


def what_is_already_failing(
    suite: tuple[str, ...], budget: Budget | None = None
) -> set[str]:
    """What this suite fails without any break applied.

    Every failure a run reports gets blamed on the break, which is right only if
    the tree was green to start with. When it is not, one unrelated failure
    appears in every record measured and each one reads as naming too few tests.

    That is not hypothetical and it deadlocks the remedy. Adding a test to a
    file some record names turns
    `test_every_guard_record_says_how_many_tests_the_files_it_names_held` red,
    which is the check telling you to re-measure. Re-measuring then sees that
    same red test under every break, calls each record short, and refuses to
    write, because it will not stamp a fingerprint on a record it believes is
    wrong. The check stays red and the only remedy for it cannot clear it. It
    happened the first time anybody used it, on 17 records at once.

    So the failures already there are read once, before anything is broken, and
    taken out of what each break is blamed for. They are printed rather than
    quietly subtracted: measuring against a tree that is not green is worth
    knowing about, even when the arithmetic is now right.

    Bounded by the same budget a record gets, and that is the half that makes
    the promise true rather than nearly true. This is a whole suite build and
    run before any break, and shard 40 of run 35520204784 spent 809 s in its
    first one; a pre-read that does not return would end a shard exactly as a
    record that does not return did.
    """
    verdicts = run_the_whole_suite(suite, budget=budget)
    return {name for name, verdict in verdicts.items() if verdict == "FAILED"}


# ── A pre-read kept for a tree it has already read ──────────────────────────
#
# Added 2026-09-23 by 12-03.2. Every invocation read its suites whole before
# breaking anything, 50 to 100 seconds for the library, and an invocation
# measuring one record spent half its time there. So a pre-read that returned is
# kept under `target/guards-pre-read/`, one file per suite, and a later
# invocation uses it when the key it was kept under is the key of this tree, for
# at most `THE_LONGEST_A_PRE_READ_IS_KEPT` seconds.

# Two hours, shorter than the 144 minutes a plan took on average over the nine
# measured on 2026-09-23, so a kept reading seldom outlives the plan that took
# it. What the key cannot see is what this bounds: `HEAD` and the history, the
# Windows credential store, the user profile a test may read, ignored files
# such as `oauth.toml` and `.env`, `LANG`, and a flaky test's luck.
THE_LONGEST_A_PRE_READ_IS_KEPT = 0


def the_settings_a_run_reads(environ: dict[str, str]) -> list[tuple[str, str | None]]:
    """The environment that moves a run or a build, each name with its value.

    >>> the_settings_a_run_reads({})
    [('CARGO_TARGET_DIR', None), ('RUSTFLAGS', None), ('WIXEN_TEST_THREADS', '8')]
    >>> the_settings_a_run_reads({"WIXEN_TEST_THREADS": "8"}) == the_settings_a_run_reads({})
    True
    >>> dict(the_settings_a_run_reads({"RUSTFLAGS": "-C debuginfo=0"}))["RUSTFLAGS"]
    '-C debuginfo=0'
    >>> [name for name, _ in the_settings_a_run_reads({"WIXEN_NO_AUDIO": "1", "PATH": "x"})]
    ['CARGO_TARGET_DIR', 'RUSTFLAGS', 'WIXEN_NO_AUDIO', 'WIXEN_TEST_THREADS']
    >>> the_settings_a_run_reads({"CARGO_BUILD_JOBS": "4"})[0]
    ('CARGO_BUILD_JOBS', '4')
    >>> "LANG" in dict(the_settings_a_run_reads({"LANG": "en_GB.UTF-8"}))
    False
    """
    return []


def the_records_without_their_counts(text: str) -> str:
    """The records' own text with only the counts `--remeasure` writes left out.

    >>> spread = (
    ...     '[[guard]]\\nname = "a"\\nbefore = "x"\\nred = ["t"]\\n'
    ...     'tests_last_seen = [\\n    { file = "a.rs", tests = 3 },\\n]\\n'
    ... )
    >>> one_line = (
    ...     '[[guard]]\\nname = "a"\\nbefore = "x"\\nred = ["t"]\\n'
    ...     'tests_last_seen = [{ file = "a.rs", tests = 9 }]\\n'
    ... )
    >>> the_records_without_their_counts(spread) == the_records_without_their_counts(one_line)
    True
    >>> the_records_without_their_counts(spread) == the_records_without_their_counts(
    ...     spread.replace('red = ["t"]', 'red = ["u"]'))
    False
    >>> the_records_without_their_counts(spread) == the_records_without_their_counts(
    ...     spread.replace('before = "x"', 'before = "y"'))
    False
    """
    return text


def the_pre_read_key(
    tree: str,
    records: str,
    suite: tuple[str, ...],
    settings: list[tuple[str, str | None]],
    compiler: str,
) -> str:
    """One hash over everything a kept pre-read must match.

    >>> taken = ("tree", "records", ("--lib",), [("RUSTFLAGS", None)], "rustc 1.90.0")
    >>> the_pre_read_key(*taken) == the_pre_read_key(*taken)
    True
    >>> len(the_pre_read_key(*taken))
    64
    >>> moved = [
    ...     ("another tree", *taken[1:]),
    ...     (taken[0], "other records", *taken[2:]),
    ...     (*taken[:2], ("--test", "house_style"), *taken[3:]),
    ...     (*taken[:3], [("RUSTFLAGS", "-C debuginfo=0")], taken[4]),
    ...     (*taken[:4], "rustc 1.91.0"),
    ... ]
    >>> sorted(the_pre_read_key(*one) != the_pre_read_key(*taken) for one in moved)
    [True, True, True, True, True]
    """
    return ""


def a_kept_pre_read_answers(kept: str, key: str, now: float, longest: float) -> set[str] | None:
    """The failing names a kept pre-read holds, when it may be used, else None.

    >>> kept = '{"key": "k", "taken_at": 1000.0, "failing": ["a::b"]}'
    >>> a_kept_pre_read_answers(kept, "k", 1060.0, THE_LONGEST_A_PRE_READ_IS_KEPT)
    {'a::b'}
    >>> THE_LONGEST_A_PRE_READ_IS_KEPT // 3600
    2
    >>> a_kept_pre_read_answers(kept, "another", 1060.0, 7200) is None
    True
    >>> a_kept_pre_read_answers(kept, "k", 1000.0 + 7201, 7200) is None
    True
    >>> a_kept_pre_read_answers(kept, "k", 999.0, 7200) is None
    True
    >>> a_kept_pre_read_answers("not a reading", "k", 1060.0, 7200) is None
    True
    >>> a_kept_pre_read_answers('{"key": "k"}', "k", 1060.0, 7200) is None
    True
    """
    return set()


def measure(
    guard: Guard,
    scratch: Path,
    already_failing: set[str] | None = None,
    named_only: bool = False,
    budget: Budget | None = None,
) -> Measured:
    """Apply the break, run the guard's own suite, put the file back.

    The budget is this record's, and a record that spends it is given up on
    with a `GaveUp`, which is a `Wrong`. That is why it is one: the `finally`
    below puts the guarded file back whatever leaves the `try`, so a record
    stopped at its time limit leaves no break behind in the tree.
    """
    found = guard.file.read_text(encoding="utf-8").count(guard.before)
    if found != 1:
        raise Wrong(
            f"{guard.name}: the text this break replaces appears {found} times "
            f"in {guard.file.relative_to(ROOT)}, and a break has to be exactly "
            "one edit.\nSomebody has moved the code this guard is about. Take "
            "the break by hand, see what really goes red now, and write that "
            "down here."
        )

    kept = scratch / str(guard.file.relative_to(ROOT)).replace("\\", "__")
    kept.write_bytes(guard.file.read_bytes())
    try:
        # The break is written with the bytes the file already uses. Written in
        # text mode this turns every line of the file into CRLF on Windows for
        # the length of the run, and a build in the middle of it compiles a
        # converted file.
        broken = guard.file.read_text(encoding="utf-8").replace(
            guard.before, guard.after
        )
        guard.file.write_bytes(broken.encode("utf-8"))
        verdicts = run_the_whole_suite(
            guard.suite,
            the_filters_for(guard.red) if named_only else None,
            budget,
        )
    finally:
        # The bytes, not the timestamps: a restored file with its old
        # modification time reads to cargo as one that never changed, and the
        # next run answers out of the broken binary.
        guard.file.write_bytes(kept.read_bytes())
        kept.unlink()

    never_ran = [name for name in guard.red if name not in verdicts]
    if never_ran:
        raise Wrong(
            "the test harness never ran "
            + ", ".join(never_ran)
            + ".\nEither the name is wrong or the test has gone."
        )
    named = set(guard.red)
    went_red = {name for name, verdict in verdicts.items() if verdict == "FAILED"}
    # A test that was failing before the break was applied was not felled by it.
    # Only subtracted from what the break is blamed for; a named test that is
    # already red is left alone, because it stays a test this break is recorded
    # as reddening and the run has not shown otherwise.
    went_red -= (already_failing or set()) - named
    return Measured(
        stayed_green=[name for name in guard.red if name not in went_red],
        also_went_red=sorted(went_red - named),
    )


def the_timing_line(rebuild_seconds: float, run_seconds: float) -> str:
    """One line per run saying what its two terms cost, so a sweep's log is
    also its own rate series.

    The cost of a record is a rebuild plus a run, and the two move for
    different reasons: the rebuild with how much of the crate a one-file
    change invalidates, the run with how many tests the suite holds and how
    many threads it gets. `THE_COST_OF_ASKING_PROPERLY` below quotes both,
    and until this line existed the only way to re-take it was a stopwatch
    around a run nobody was watching. Now every log carries the figure.

    Each term is rounded on its own and the total is the sum of the rounded
    terms, so the arithmetic on the line checks by eye:

    >>> the_timing_line(29.4, 66.2)
    '   timed: rebuild 29 s, run 66 s, 95 s in all'
    >>> the_timing_line(0.6, 3.2)
    '   timed: rebuild 1 s, run 3 s, 4 s in all'
    >>> the_timing_line(0.0, 51.0)
    '   timed: rebuild 0 s, run 51 s, 51 s in all'
    """
    rebuild = round(rebuild_seconds)
    run = round(run_seconds)
    return f"   timed: rebuild {rebuild} s, run {run} s, {rebuild + run} s in all"


def how_many(count: int, thing: str) -> str:
    """A count with the thing it counts, so a line reads as a sentence.

    The product keeps one routine for this, `how_many` in
    `src/service/caldav.rs`, and `guards/guards.toml` guards it under the name
    "a count and the thing it counts agree in number". This is that rule
    written again rather than that routine called, and the reason is what this
    script does: it breaks the tree on purpose and runs the suite against the
    break. Reaching the product's answer means building and running the crate,
    so half the time the code holding the wording is the code that will not
    compile, and a script that cannot say what it found until the thing under
    test builds is worse than six words written twice.

    Written again, and then checked, which is the part that was missing when
    this file printed "1 tests went red":

    >>> how_many(1, "test")
    '1 test'
    >>> how_many(2, "test")
    '2 tests'
    >>> how_many(0, "named test")
    '0 named tests'
    """
    return f"1 {thing}" if count == 1 else f"{count} {thing}s"


def say_what_it_found(guard: Guard, measured: Measured) -> None:
    """Print what the break really did, against what the record says.

    The lines themselves, and not only the routine that words them, because
    the defect this checks for was a count written straight into a line here
    while the routine sat unused two functions away:

    >>> one = Guard("a guard", Path("nowhere.rs"), "a", "b", ("first",))
    >>> say_what_it_found(one, Measured(stayed_green=[], also_went_red=["other"]))
       1 test went red that this record does not name:
           other
    <BLANKLINE>
    >>> say_what_it_found(one, Measured(stayed_green=["first"], also_went_red=[]))
       1 of 1 named test stayed green with the guard broken:
           first
    <BLANKLINE>
    >>> three = Guard("a guard", Path("nowhere.rs"), "a", "b", ("a", "b", "c"))
    >>> say_what_it_found(three, Measured(stayed_green=["a"], also_went_red=[]))
       1 of 3 named tests stayed green with the guard broken:
           a
    <BLANKLINE>
    >>> say_what_it_found(three, Measured(stayed_green=[], also_went_red=[]))
       all 3 tests named went red, and nothing else did
    >>> say_what_it_found(one, Measured(stayed_green=[], also_went_red=[]))
       the one test named went red, and nothing else did
    """
    if measured.stayed_green:
        print(
            f"   {len(measured.stayed_green)} of "
            f"{how_many(len(guard.red), 'named test')} stayed green with the "
            "guard broken:"
        )
        for name in measured.stayed_green:
            print(f"       {name}")
    if measured.also_went_red:
        print(
            f"   {how_many(len(measured.also_went_red), 'test')} went red that "
            "this record does not name:"
        )
        for name in measured.also_went_red:
            print(f"       {name}")
    if measured.stayed_green or measured.also_went_red:
        print()
    elif len(guard.red) == 1:
        print("   the one test named went red, and nothing else did")
    else:
        print(
            f"   all {how_many(len(guard.red), 'test')} named went red, and "
            "nothing else did"
        )


def verdicts_in(log_text: str) -> dict[str, str]:
    """What a run's log says about each record it started, read from the
    lines the loop and `say_what_it_found` print and from nothing else.

    A sweep is hours, and a run that cannot be stopped and picked up is a
    scheduling problem before it is a technical one. The log already holds
    what a resume needs: `-- <name>` before each record, flushed, and one of
    the verdict shapes after it. So a resume is a reading of the log's own
    lines, and the shapes are held by `say_what_it_found`'s examples; change
    one there and change it here in the same commit.

    Three verdicts, and a fourth outcome that is not one. `agreed`: the break
    reddened exactly the named tests. `short`: a named test stayed green, or a
    test went red that the record does not name. `could not be measured`: the
    break no longer applies, a named test no longer exists, the break did not
    build, or the harness captured nothing; that is still a verdict, because
    measuring it again would find the same thing, and it is corrected by hand.
    A name with nothing after it, which is what a kill mid-record leaves, has no
    verdict and is measured again.

    >>> def block(name, *lines):
    ...     return "\\n".join([f"-- {name}", *lines, ""])

    The five shapes `say_what_it_found` prints:

    >>> verdicts_in(block("alone", "   the one test named went red, and nothing else did"))
    {'alone': 'agreed'}
    >>> verdicts_in(block("together", "   all 3 tests named went red, and nothing else did"))
    {'together': 'agreed'}
    >>> verdicts_in(block(
    ...     "stayed green",
    ...     "   1 of 1 named test stayed green with the guard broken:",
    ...     "       a::tests::test_b",
    ...     "",
    ... ))
    {'stayed green': 'short'}
    >>> verdicts_in(block(
    ...     "two stayed green",
    ...     "   2 of 3 named tests stayed green with the guard broken:",
    ...     "       a::tests::test_b",
    ...     "       a::tests::test_c",
    ...     "",
    ... ))
    {'two stayed green': 'short'}
    >>> verdicts_in(block(
    ...     "nobody wrote down",
    ...     "   1 test went red that this record does not name:",
    ...     "       a::tests::test_d",
    ...     "",
    ... ))
    {'nobody wrote down': 'short'}

    A `Wrong` the loop printed, in each of the shapes `measure` and the run can
    raise. The break's text has moved:

    >>> verdicts_in(block(
    ...     "moved",
    ...     "   moved: the text this break replaces appears 0 times in src/a.rs, and a break has to be exactly one edit.",
    ...     "Somebody has moved the code this guard is about. Take the break by hand, see what really goes red now, and write that down here.",
    ...     "",
    ... ))
    {'moved': 'could not be measured'}

    A named test the harness never ran, a break that did not build, a break that
    built and named no test, and a run that captured nothing:

    >>> verdicts_in(block("gone", "   the test harness never ran a::tests::test_b.", "Either the name is wrong or the test has gone.", ""))
    {'gone': 'could not be measured'}
    >>> verdicts_in(block("no build", "   the break did not build, so no test ran. cargo exited 101 and said:", "error[E0308]: mismatched types", ""))
    {'no build': 'could not be measured'}
    >>> verdicts_in(block("no test", "   the break built and the run named no test. cargo exited 0 and said:", "running 0 tests", ""))
    {'no test': 'could not be measured'}
    >>> verdicts_in(block("nothing captured", "   cargo ran and this captured none of its output, so nothing can be read from it. stdout is missing and stderr is present, and cargo exited 0.", ""))
    {'nothing captured': 'could not be measured'}

    And the line the loop prints for anything else that broke a record:

    >>> verdicts_in(block("broke", "   this record could not be measured: OSError(28, 'No space left on device')", ""))
    {'broke': 'could not be measured'}

    The timing line is not a verdict, and a name with only that after it, or
    with nothing after it at all, is a record a kill interrupted:

    >>> verdicts_in(block("killed after the run", "   timed: rebuild 44 s, run 47 s, 91 s in all"))
    {}
    >>> verdicts_in(block("killed at once"))
    {}
    >>> verdicts_in(block("timed and judged", "   timed: rebuild 44 s, run 47 s, 91 s in all", "   the one test named went red, and nothing else did"))
    {'timed and judged': 'agreed'}

    A record, or a whole suite's pre-read, that the time limit stopped. It is
    unmeasured rather than judged, so a resume takes it again, and the two
    readings of that line answer differently on purpose: `the_verdict_on` reads
    it, so nothing refuses a log that holds one, and this drops the name. The
    kill's own line is indented four spaces and is not read as a verdict:

    >>> given_up = "   a guard: it did not return within its budget of 1800 s, and was still building when the budget ran out at 1801 s."
    >>> verdicts_in(block(
    ...     "hung",
    ...     given_up,
    ...     "    the kill took the tree: no cargo or rustc is building now",
    ...     "This was not measured, so it has no verdict and a resume takes it again.",
    ...     "",
    ... ))
    {}
    >>> the_verdict_on("hung", given_up)
    'could not be measured'

    A record another cargo ran beside is unmeasured whatever it printed, so a
    resume takes it again; and the last block for a name is the one that
    counts, so the record measured again after that has its verdict:

    >>> contended = "   contended: another cargo ran during this record"
    >>> verdicts_in(block("beside a hook", "   the one test named went red, and nothing else did", contended))
    {}
    >>> verdicts_in(
    ...     block("beside a hook", "   the one test named went red, and nothing else did", contended)
    ...     + block("beside a hook", "   the one test named went red, and nothing else did")
    ... )
    {'beside a hook': 'agreed'}

    Everything before the first `-- ` line is the run's preamble, which the
    pre-read writes, and it is not read for verdicts. Line endings are the
    file's own, since Python writes the log through a text stream on Windows:

    >>> verdicts_in("== 1 guard, one build and one run ==\\r\\n\\r\\n-- one\\r\\n   the one test named went red, and nothing else did\\r\\n")
    {'one': 'agreed'}

    Several runs' logs joined into one file read as one log: the shards of a
    sweep on runners are downloaded and concatenated, and each shard's log
    carries its own header, pre-read timing line, closing lines and, when it
    stopped short, a resume command. None of those is indented with the
    loop's three spaces, so none is mistaken for a verdict, and the records
    of every shard answer:

    >>> two_shards = (
    ...     "== 2 guards, shard 0/2 of 3 records, one build and one run each ==\\n\\n"
    ...     "Reading what already fails with --lib, before anything is broken.\\n"
    ...     "   timed: rebuild 300 s, run 140 s, 440 s in all\\n"
    ...     + block("first", "   the one test named went red, and nothing else did")
    ...     + block("second", "   1 test went red that this record does not name:", "       a::b", "")
    ...     + "\\n1 guard is not what the record says it is:\\n    second\\n\\n"
    ...     "A named test that stayed green is a test whose name still promises\\n"
    ...     "something it no longer checks.\\n\\n"
    ...     "Every record selected has a verdict: 2 of 2, 0 from the log and 2 from this run.\\n"
    ...     "== 1 guard, shard 1/2 of 3 records, one build and one run ==\\n\\n"
    ...     "Reading what already fails with --lib, before anything is broken.\\n"
    ...     "   timed: rebuild 300 s, run 140 s, 440 s in all\\n"
    ...     + block("third", "   the one test named went red, and nothing else did")
    ...     + "\\nThe guard still goes red when what it defends breaks, and nothing else does.\\n\\n"
    ...     "0 measured this run; 1 of 3 remain. Resume with:\\n"
    ...     "    scripts/guards.sh --log sweep-1-of-2.log --resume\\n"
    ... )
    >>> verdicts_in(two_shards)
    {'first': 'agreed', 'second': 'short', 'third': 'agreed'}

    A line this cannot read is refused with the line, never guessed at. The
    reading is anchored on the first line in a block that carries the three
    spaces the loop indents with, so a shape the runner never prints there is a
    log some other tool wrote or a format that moved without this moving:

    >>> verdicts_in(block("odd", "   something the runner never prints"))
    Traceback (most recent call last):
        ...
    guards.Wrong: under "-- odd", a line this cannot read as a verdict:
        something the runner never prints
    The runner prints only the shapes say_what_it_found's examples show. Either this log was not written by it, or the shapes moved and this reading did not.
    """
    verdicts: dict[str, str] = {}
    name: str | None = None
    judged = False
    for line in log_text.splitlines():
        if line.startswith("-- "):
            name = line[len("-- ") :]
            judged = False
            # The last block for a name is the one that counts, and a block
            # with nothing after it is a record to measure again, so an older
            # verdict for the same name is dropped here and put back only if
            # this block earns one.
            verdicts.pop(name, None)
            continue
        if name is None or line.startswith("   timed: "):
            continue
        if line == CONTENDED:
            verdicts.pop(name, None)
            judged = True
            continue
        if judged or not line.startswith("   ") or line[3:4] in ("", " "):
            continue
        judged = True
        if GIVEN_UP_ON in line:
            # Unmeasured rather than judged, exactly as a contended record is:
            # the run stopped it at its time limit and learned nothing about
            # it, so a resume measures it again. `the_verdict_on` can still
            # read the line, which is what stops a log holding one from being
            # refused outright.
            verdicts.pop(name, None)
            continue
        verdicts[name] = the_verdict_on(name, line)
    return verdicts


# Beneath a record's verdict when another cargo was alive as its run returned.
# `verdicts_in` reads this exact line, so it is written once.
CONTENDED = "   contended: another cargo ran during this record"

AGREED = re.compile(
    r"^   (the one test named|all \d+ tests named) went red, and nothing else did$"
)
SHORT = re.compile(
    r"^   (\d+ of \d+ named tests? stayed green with the guard broken:"
    r"|\d+ tests? went red that this record does not name:)$"
)
# The first line of everything the loop prints for a record it could not
# measure: the `Wrong` messages `measure`, `run_the_whole_suite` and `cargo`
# raise, and the line for anything else that broke.
COULD_NOT_BE_MEASURED = (
    "   this record could not be measured: ",
    ": the text this break replaces appears ",
    "   the test harness never ran ",
    "   the break did not build, so no test ran.",
    "   the break built and the run named no test.",
    "   cargo ran and this captured none of its output",
    # And the line a record or a pre-read given up on at its time limit
    # prints. Here so that a log holding one can be read at all; `verdicts_in`
    # drops the name rather than keeping this as its verdict, for the reason
    # written above `GIVEN_UP_ON`.
    GIVEN_UP_ON,
)


def the_verdict_on(name: str, line: str) -> str:
    if AGREED.match(line):
        return "agreed"
    if SHORT.match(line):
        return "short"
    if any(opener in line for opener in COULD_NOT_BE_MEASURED):
        return "could not be measured"
    raise Wrong(
        f'under "-- {name}", a line this cannot read as a verdict:\n'
        f"    {line.strip()}\n"
        "The runner prints only the shapes say_what_it_found's examples show. "
        "Either this log was not written by it, or the shapes moved and this "
        "reading did not."
    )


def foreign_builds_in(tasklist_output: str, own_pids: set[int]) -> list[str]:
    """Every cargo or rustc alive in a `tasklist /FO CSV /NH` listing that
    this process did not start, as `name pid`, so a wait can say what it is
    waiting for.

    >>> idle = (
    ...     '"System Idle Process","0","Services","0","8 K"\\n'
    ...     '"bash.exe","4120","Console","1","6,000 K"\\n'
    ... )
    >>> foreign_builds_in(idle, set())
    []
    >>> busy = idle + (
    ...     '"cargo.exe","1234","Console","1","40,000 K"\\n'
    ...     '"rustc.exe","1240","Console","1","900,000 K"\\n'
    ... )
    >>> foreign_builds_in(busy, set())
    ['cargo.exe 1234', 'rustc.exe 1240']
    >>> foreign_builds_in(busy, {1234, 1240})
    []
    >>> foreign_builds_in(busy, {1234})
    ['rustc.exe 1240']

    What `tasklist` prints when a filter matches nothing, kept for the day
    somebody passes a filtered listing in:

    >>> foreign_builds_in("INFO: No tasks are running which match the specified criteria.\\n", set())
    []
    """
    found: list[str] = []
    for line in tasklist_output.splitlines():
        seen = BUILD_PROCESS.match(line)
        if seen and int(seen.group(2)) not in own_pids:
            found.append(f"{seen.group(1)} {seen.group(2)}")
    return found


# A row of `tasklist /FO CSV /NH` for one of the two processes a build is.
BUILD_PROCESS = re.compile(r'^"(cargo\.exe|rustc\.exe)","(\d+)"', re.I)


def is_quiet(tasklist_output: str, own_pids: set[int]) -> bool:
    """Whether no cargo or rustc is running that this process did not start.

    The condition `scripts/guards.sh` has stated in a comment since 2026-08-08
    and nothing checked: a commit hook running the suite in the middle of a
    sweep reported three guards green that go red on their own.

    >>> is_quiet('"bash.exe","4120","Console","1","6,000 K"\\n', set())
    True
    >>> is_quiet('"rustc.exe","1240","Console","1","900,000 K"\\n', set())
    False
    >>> is_quiet('"rustc.exe","1240","Console","1","900,000 K"\\n', {1240})
    True
    """
    return not foreign_builds_in(tasklist_output, own_pids)


def what_is_running() -> str:
    """`tasklist` as it lists every process, for `foreign_builds_in` to read.

    One unfiltered listing rather than one filtered call per image name,
    because `tasklist` joins its filters with AND, so asking for cargo and
    rustc in one call answers nothing. About a tenth of a second.
    """
    try:
        listing = subprocess.run(
            ["tasklist", "/FO", "CSV", "/NH"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except FileNotFoundError:
        raise Wrong(
            "--wait-until-quiet reads tasklist, which is not on this machine. "
            "The sweep is a Windows job; drop the flag elsewhere."
        ) from None
    if listing.returncode != 0:
        raise Wrong(
            f"tasklist exited {listing.returncode}, so whether the machine is "
            f"quiet cannot be read:\n{listing.stderr.strip()}"
        )
    return listing.stdout


def wait_until_quiet() -> None:
    """Block until no cargo or rustc this process did not start is running.

    Polled only between records, when this runner has no child alive: `cargo`
    here is `subprocess.run`, which returns after its process and every rustc
    it spawned have exited. So the set of own pids handed to `is_quiet` is
    empty in practice, and the parameter is there so the reading says what
    quiet means rather than for anything this caller passes.

    A poll before a record cannot see a build that starts after it, which is
    exactly a hook running during a sweep; the poll after each record in the
    loop is what catches that, and marks the record contended.
    """
    waited = 0
    while foreign := foreign_builds_in(what_is_running(), set()):
        if waited % 60 == 0:
            print(f"waiting for a quiet machine: {', '.join(foreign)}", flush=True)
        time.sleep(10)
        waited += 10


def why_a_resume_is_refused(porcelain: str) -> str | None:
    """Why a resume may not start over this working tree, if it may not,
    read from `git status --porcelain` over the guarded files.

    `measure` puts a broken file back in a `finally`, which a `KeyboardInterrupt`
    reaches and a process-tree kill, a closed terminal or a power cut may not.
    A break left behind is a source change nobody made on purpose, and a sweep
    resumed over it measures every record against somebody else's edit. So a
    resume reads the tree first and refuses, naming the file and the command
    that puts it back, rather than measuring over it. This is the second place
    the script reads git, and it still changes nothing through it: the
    `checkout` is printed for a person to run.

    >>> why_a_resume_is_refused("") is None
    True
    >>> print(why_a_resume_is_refused(" M src/application/allowed.rs\\n"))
    A guarded file is modified in the working tree, so nothing was measured:
        src/application/allowed.rs
    <BLANKLINE>
    A run killed hard enough to skip its restore leaves its break behind, and a
    sweep resumed over it would measure every record against that edit. If the
    change is not yours, put the file back and start again:
        git checkout -- src/application/allowed.rs

    Every modified file at once, staged or not, so one refusal names them all:

    >>> print(why_a_resume_is_refused("M  src/a.rs\\n M src/b.rs\\n"))
    A guarded file is modified in the working tree, so nothing was measured:
        src/a.rs
        src/b.rs
    <BLANKLINE>
    A run killed hard enough to skip its restore leaves its break behind, and a
    sweep resumed over it would measure every record against that edit. If the
    change is not yours, put the file back and start again:
        git checkout -- src/a.rs src/b.rs
    """
    modified = [line[3:] for line in porcelain.splitlines() if line.strip()]
    if not modified:
        return None
    return (
        "A guarded file is modified in the working tree, so nothing was measured:\n"
        + "".join(f"    {path}\n" for path in modified)
        + "\n"
        "A run killed hard enough to skip its restore leaves its break behind, and a\n"
        "sweep resumed over it would measure every record against that edit. If the\n"
        "change is not yours, put the file back and start again:\n"
        f"    git checkout -- {' '.join(modified)}"
    )


def the_guarded_files_git_sees_as_modified(guards: list[Guard]) -> str:
    """`git status --porcelain` over every guarded file, for the refusal above."""
    files = sorted({str(guard.file.relative_to(ROOT)).replace("\\", "/") for guard in guards})
    finished = subprocess.run(
        ["git", "status", "--porcelain", "--", *files],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if finished.returncode != 0:
        raise Wrong(
            "git could not say whether the guarded files are clean, so a resume "
            f"cannot start:\n{finished.stderr.strip()}"
        )
    return finished.stdout


def the_shard_asked_for(text: str) -> tuple[int, int]:
    """`k/n` as `--shard` takes it: which shard, out of how many, k from 0.

    The same spelling `scripts/mutants.sh --shard` uses, so a workflow that
    fans out over a matrix hands both scripts the same word. Refused outside
    `0 <= k < n`, before anything is read or built, because the runner finds
    out otherwise after the checkout and the build.

    >>> the_shard_asked_for("3/41")
    (3, 41)
    >>> the_shard_asked_for("0/1")
    (0, 1)
    >>> the_shard_asked_for("41/41")
    Traceback (most recent call last):
        ...
    guards.Wrong: --shard 41/41: k counts from 0 and must sit inside 0..40, so shard 41 of 41 is not one.
    >>> the_shard_asked_for("-1/2")
    Traceback (most recent call last):
        ...
    guards.Wrong: --shard -1/2: k counts from 0 and must sit inside 0..1, so shard -1 of 2 is not one.
    >>> the_shard_asked_for("3")
    Traceback (most recent call last):
        ...
    guards.Wrong: --shard 3: written as k/n, which shard out of how many, for example 3/41.
    >>> the_shard_asked_for("3/0")
    Traceback (most recent call last):
        ...
    guards.Wrong: --shard 3/0: n must be at least 1.
    """
    k_text, slash, n_text = text.partition("/")
    try:
        k, n = int(k_text), int(n_text)
    except ValueError:
        slash = ""
    if not slash:
        raise Wrong(
            f"--shard {text}: written as k/n, which shard out of how many, "
            "for example 3/41."
        )
    if n < 1:
        raise Wrong(f"--shard {text}: n must be at least 1.")
    if not 0 <= k < n:
        raise Wrong(
            f"--shard {text}: k counts from 0 and must sit inside 0..{n - 1}, "
            f"so shard {k} of {n} is not one."
        )
    return k, n


def the_records_in_shard(records: list, k: int, n: int) -> list:
    """Shard k of n over the records as the file orders them: one contiguous
    block each, so a shard's log reads in file order and two shards' logs
    concatenated in shard order read as the file does.

    The cut points are `k * len // n`, so every record is in exactly one shard
    and no two shards differ in size by more than one. Deterministic: the same
    file and the same n give the same shard on every machine, which is what
    lets a run be dispatched as n jobs and read back as one.

    >>> [the_records_in_shard(list(range(7)), k, 3) for k in range(3)]
    [[0, 1], [2, 3], [4, 5, 6]]
    >>> the_records_in_shard(list(range(7)), 0, 1)
    [0, 1, 2, 3, 4, 5, 6]

    802 records in 41 shards, the shape the sweep of 2026-09-15 is dispatched
    in, is 19 or 20 a shard and every record once:

    >>> shards = [the_records_in_shard(list(range(802)), k, 41) for k in range(41)]
    >>> sorted({len(s) for s in shards}), sum(len(s) for s in shards)
    ([19, 20], 802)

    More shards than records leaves some empty, which a run reports as nothing
    selected rather than refusing, because the dispatch that asked for them
    has already started the other jobs:

    >>> [the_records_in_shard(["a", "b"], k, 3) for k in range(3)]
    [[], ['a'], ['b']]
    """
    return records[k * len(records) // n : (k + 1) * len(records) // n]


def the_header_for(count: int, shard: tuple[int, int, int] | None = None) -> str:
    """The line a run opens with, saying how many records it will measure and,
    when it is one shard of a sweep, which shard of how many over how many
    records, so a shard's log says on its first line what it is.

    >>> the_header_for(1)
    '== 1 guard, one build and one run =='
    >>> the_header_for(20)
    '== 20 guards, one build and one run each =='
    >>> the_header_for(20, (3, 41, 802))
    '== 20 guards, shard 3/41 of 802 records, one build and one run each =='
    >>> the_header_for(1, (40, 41, 802))
    '== 1 guard, shard 40/41 of 802 records, one build and one run =='
    """
    which = f"shard {shard[0]}/{shard[1]} of {shard[2]} records, " if shard else ""
    if count == 1:
        return f"== 1 guard, {which}one build and one run =="
    return f"== {count} guards, {which}one build and one run each =="


def the_resume_command(log: str, wait_until_quiet: bool) -> str:
    """The command that picks a stopped run up from its log, printed whenever a
    run stops with records remaining, so a person never has to reconstruct it.

    >>> the_resume_command("sweep.log", wait_until_quiet=True)
    'scripts/guards.sh --log sweep.log --resume --wait-until-quiet'
    >>> the_resume_command("target/one-record.log", wait_until_quiet=False)
    'scripts/guards.sh --log target/one-record.log --resume'
    """
    quiet = " --wait-until-quiet" if wait_until_quiet else ""
    return f"scripts/guards.sh --log {log} --resume{quiet}"


def the_closing_line(
    total: int, from_log: int, this_run: int, resume: str | None, gave_up: int = 0
) -> str:
    """The last line a run prints, which is how somebody who has been told not
    to read the verdicts knows whether the sweep is done.

    Complete, so the count of records with a verdict is the count selected:

    >>> print(the_closing_line(798, 158, 640, None))
    Every record selected has a verdict: 798 of 798, 158 from the log and 640 from this run.

    Stopped with records remaining, and how to pick it up:

    >>> print(the_closing_line(798, 0, 1, "scripts/guards.sh --log sweep.log --resume"))
    1 measured this run; 797 of 798 remain. Resume with:
        scripts/guards.sh --log sweep.log --resume

    Stopped with records remaining and no log to resume from, which is a run
    whose verdicts are on the screen and nowhere else:

    >>> print(the_closing_line(3, 0, 1, None))
    1 measured this run; 2 of 3 remain, and no --log was given, so nothing recorded this run for a resume.

    A run that gave up on a record at its time limit says so before it says
    what remains, because a given-up record is unmeasured and would otherwise
    read as one the run simply never reached:

    >>> print(the_closing_line(20, 0, 19, "scripts/guards.sh --log sweep.log --resume", gave_up=1))
    1 record was given up on at its time limit, so this run is not a clean sweep.
    19 measured this run; 1 of 20 remain. Resume with:
        scripts/guards.sh --log sweep.log --resume
    >>> print(the_closing_line(20, 0, 17, None, gave_up=3))
    3 records were given up on at their time limit, so this run is not a clean sweep.
    17 measured this run; 3 of 20 remain, and no --log was given, so nothing recorded this run for a resume.
    """
    # Both ways written out rather than one built from parts, for the reason
    # `how_many` exists: the verb has to agree with the count.
    if gave_up == 1:
        given_up_on = (
            "1 record was given up on at its time limit, so this run is not a "
            "clean sweep.\n"
        )
    elif gave_up:
        given_up_on = (
            f"{gave_up} records were given up on at their time limit, so this "
            "run is not a clean sweep.\n"
        )
    else:
        given_up_on = ""

    remaining = total - from_log - this_run
    if remaining == 0:
        rest = (
            f"Every record selected has a verdict: {total} of {total}, "
            f"{from_log} from the log and {this_run} from this run."
        )
    elif resume is None:
        rest = (
            f"{this_run} measured this run; {remaining} of {total} remain, and "
            "no --log was given, so nothing recorded this run for a resume."
        )
    else:
        rest = (
            f"{this_run} measured this run; {remaining} of {total} remain. "
            f"Resume with:\n    {resume}"
        )
    return f"{given_up_on}{rest}"


class Logged:
    """Everything printed, to the screen and to the log at once, each write
    flushed to the file before it returns.

    Appended, never truncated, so a resumed run continues the file it read
    and no shell redirection is needed: `Start-Process` has no append and
    would overwrite the log it is resuming from. Both stdout and stderr go
    through it, so a traceback that ends a run is in the run's own log
    underneath the record it ended on.
    """

    def __init__(self, path: str, screen) -> None:
        self.file = open(path, "a", encoding="utf-8")
        self.screen = screen

    def write(self, text: str) -> int:
        self.file.write(text)
        self.file.flush()
        return self.screen.write(text)

    def flush(self) -> None:
        self.file.flush()
        self.screen.flush()


def measured_and_said(
    guard: Guard,
    scratch: Path,
    already_failing: set[str] | None,
    named_only: bool,
    budget: Budget | None = None,
) -> str:
    """Measure one record and print what was found: "agreed", "slipped", or
    "given up on".

    A record that could not be measured is said so and counted as not
    agreeing, in one of the shapes `verdicts_in` reads. One given up on at its
    time limit is told apart from both, because the run learned nothing about
    it either way and the closing line has to be able to say so.
    """
    try:
        measured = measure(
            guard, scratch, already_failing, named_only=named_only, budget=budget
        )
    except GaveUp as ran_out:
        # Named here rather than in the message, so the line under `-- name`
        # carries the record it is about. `measure`'s `finally` has already put
        # the guarded file back, which is the property this path must not
        # break.
        print(f"   {guard.name}: {ran_out}\n")
        return "given up on"
    except Wrong as wrong:
        print(f"   {wrong}\n")
        return "slipped"
    except Exception as broke:
        # One record must not take the run down with it. A sweep of 208
        # records is hours of building and running, and losing all of it to
        # an unexpected failure on one is how a check nobody can afford to
        # finish becomes a check nobody runs.
        #
        # Counted as slipped rather than passed, because a record that could
        # not be measured is not a record that holds. The file is already
        # restored: `measure` puts it back in a `finally`.
        #
        # KeyboardInterrupt and SystemExit are not `Exception`, so an
        # interrupt still stops the run and still restores the tree.
        print(f"   this record could not be measured: {broke!r}\n")
        return "slipped"
    say_what_it_found(guard, measured)
    return "agreed" if measured.agrees_with_the_record() else "slipped"


def main() -> int:
    parsing = argparse.ArgumentParser(description=__doc__)
    parsing.add_argument(
        "only",
        nargs="?",
        help="measure one guard, matched on any part of its name",
    )
    parsing.add_argument(
        "--touched-by",
        metavar="REF",
        help="only the records this branch could have made stale since REF, "
        "which is what a merge needs and is minutes rather than hours",
    )
    parsing.add_argument(
        "--named-only",
        action="store_true",
        help="a cheap pre-filter, not a sweep. Runs only the modules a "
        "record's tests live in, so it does not ask whether anything else went "
        "red, and it disagrees with a full run on tests that fail only in "
        "company: measured 2026-09-02, one record read as broken filtered and "
        "correct unfiltered. Confirm anything it flags without this flag",
    )
    parsing.add_argument(
        "--remeasure",
        nargs="+",
        metavar="NAME",
        help="measure these records, named exactly, and write down the tree "
        "each one agreed with. This is what "
        "test_every_guard_record_says_how_many_tests_the_files_it_names_held "
        "tells you to run, and it names the records for you",
    )
    parsing.add_argument(
        "--recount-everything",
        action="store_true",
        help="write down the tree every record is sitting in, having measured "
        "nothing. For adopting the count on records written before it existed, "
        "and for nothing else: it does not say a record is right",
    )
    parsing.add_argument(
        "--shard",
        metavar="K/N",
        help="one contiguous block of the file's records, shard K of N counting "
        "from 0, the same on every machine for the same file and N; what a "
        "runner job takes when the sweep is dispatched as N jobs and read back "
        "as one log",
    )
    parsing.add_argument(
        "--log",
        metavar="PATH",
        help="append every line this prints to PATH as well as the screen, "
        "flushed per line, so a detached run can be watched and a stopped one "
        "picked up from it with --resume",
    )
    parsing.add_argument(
        "--resume",
        nargs="?",
        const="",
        metavar="PATH",
        help="skip every record the log at PATH, or at --log when PATH is "
        "left out, already holds a verdict for, and measure the rest. A name "
        "with no verdict, which is what a kill mid-record leaves, and a record "
        "marked contended are measured again. Refuses to start over a "
        "guarded file the working tree shows as modified",
    )
    parsing.add_argument(
        "--stop-after",
        type=int,
        metavar="N",
        help="measure at most N records this invocation, then say how many "
        "remain and how to resume. 0 measures nothing and reports what remains",
    )
    parsing.add_argument(
        "--time-limit",
        type=int,
        default=THE_LONGEST_A_RECORD_MAY_TAKE,
        metavar="SECONDS",
        help="the wall clock one record gets, its build and its run together, "
        "and one suite's pre-read the same. What passes it is killed along "
        "with the processes cargo started under it, reported unmeasured with "
        "the seconds it reached, and the run goes on to the next record or "
        "suite. A flag and not an environment variable, because "
        "tests/the_guard_sweep_runs_on_runners.rs reads the flags this script "
        "accepts off these calls and holds the workflow to them, and a "
        "variable set in a workflow's env: would be a coupling nothing checks",
    )
    parsing.add_argument(
        "--wait-until-quiet",
        action="store_true",
        help="before the pre-read and before each record, wait until no "
        "cargo.exe or rustc.exe this run did not start is alive, and after each "
        "record mark it contended and unmeasured if one is, so a resume "
        "measures it again",
    )
    asked = parsing.parse_args()

    # Refused before anything is opened or built. A limit of nothing gives up
    # on every record in turn, which reads like a broken tree rather than like
    # a flag typed wrong.
    if asked.time_limit <= 0:
        print(
            f"\n--time-limit {asked.time_limit} gives every record no time at "
            "all, so each would be given up\non before its build started. It "
            "is a number of seconds and it has to be one.\n"
        )
        return 1

    # Read before the log is opened for appending, because opening it creates
    # it, and a resume from a log that is not there then read as a resume from
    # an empty one and started the whole sweep. That happened on the first try.
    picked_up: str | None = None
    if asked.resume is not None:
        if not asked.log:
            print(
                "\n--resume needs --log: a run that records its verdicts nowhere "
                "cannot itself be\nresumed. Add --log PATH, which --resume then "
                "reads too unless given a path of its own.\n"
            )
            return 1
        picked_up_from = asked.resume or asked.log
        try:
            picked_up = Path(picked_up_from).read_text(encoding="utf-8")
        except FileNotFoundError:
            print(
                f"\nNothing to resume from: {picked_up_from} is not there. A first "
                "run takes --log without --resume.\n"
            )
            return 1

    if asked.log:
        sys.stdout = sys.stderr = Logged(asked.log, sys.stdout)

    # The shard is refused before the record is read, so a runner that was
    # handed a bad one finds out in the first second and not after the build.
    shard: tuple[int, int, int] | None = None
    try:
        if asked.shard:
            k, n = the_shard_asked_for(asked.shard)
        guards = read_record()
        if asked.shard:
            # By position in the whole file, before any other narrowing, so
            # the shard is the same whatever else the run asked for.
            shard = (k, n, len(guards))
            guards = the_records_in_shard(guards, k, n)
            if not guards:
                print(
                    f"\nShard {k}/{n} of {shard[2]} records holds no record, so "
                    "there is nothing to measure. That is an answer, not a "
                    "clean sweep.\n"
                )
                return 0
    except Wrong as wrong:
        print(f"\n{wrong}\n")
        return 1

    if asked.recount_everything:
        narrowed = the_filter_that_forbids_recounting_everything(
            asked.only, asked.touched_by, asked.remeasure, asked.shard
        )
        if narrowed:
            print(
                "\n--recount-everything writes a fingerprint on every record in "
                "the file, so it\ncannot also be narrowed, and this run asked for "
                f"{narrowed}.\n\n"
                "It measures nothing. A count written this way says only that no "
                "test has been\nadded to those files since somebody looked, never "
                "that a record is right, so\nletting it through on a narrower set "
                "would put that weaker claim on every\nrecord in the file anyway, "
                "including the ones already known to be short.\n\n"
                "Drop one of the two.\n"
            )
            return 1

        # Loud, and it says the thing it does not do. A count written here is
        # "no test has been added to these files since somebody looked", which
        # is a weaker claim than "this record is right" and reads exactly like
        # it in the file. The only way to earn the stronger one is to measure.
        write_down_the_counts(guards, {g.name: what_the_tree_holds_now(g) for g in guards})
        print(
            f"Wrote down, for {how_many(len(guards), 'record')}, the tree it "
            "is sitting in.\n\n"
            "This measured nothing. It does not say any of those records names "
            "the right\ntests; it says what the files they name held today, so "
            "that a test added to\none of those files from now on fails the "
            "commit that adds it. A record that\nis already short stays short, "
            "and only a run finds that.\n"
        )
        return 0

    if asked.remeasure:
        known = {guard.name: guard for guard in guards}
        unknown = [name for name in asked.remeasure if name not in known]
        if unknown:
            print(
                "\nNo record is named exactly:\n    "
                + "\n    ".join(unknown)
                + "\n\nThese are matched whole rather than by part, because the "
                "check that\nsends you here prints them whole.\n"
            )
            return 1
        guards = [known[name] for name in asked.remeasure]

    if asked.only:
        guards = [g for g in guards if asked.only.lower() in g.name.lower()]
        if not guards:
            print(f"\nNo guard is named anything like {asked.only!r}.\n")
            return 1

    if asked.touched_by:
        try:
            changed = files_changed_since(asked.touched_by)
        except Wrong as wrong:
            print(f"\n{wrong}\n")
            return 1
        whole = len(guards)
        guards = [g for g in guards if could_have_gone_stale(g, changed)]
        # Said out loud, because a run that quietly measured 14 of 683 and
        # printed a clean result would read as a clean sweep. It is not one,
        # and the sentence below is the only place anybody learns that.
        print(
            f"{how_many(len(changed), 'file')} changed since {asked.touched_by}. "
            f"Measuring {len(guards)} of {whole} records: the ones those files "
            "could have made stale.\n"
            "This is a candidate set, not a sweep. A test added in a module a "
            "record has never named can still redden it, and no reading of the "
            "record predicts that. Only the whole run does, and it is hours.\n"
        )
        if not guards:
            print(
                "No record could have been disturbed by these files, so none "
                "was measured. That is an answer, not a clean sweep.\n"
            )
            return 0

    selected = len(guards)
    slipped: list[str] = []
    agreed: list[Guard] = []
    contended: list[str] = []
    gave_up: list[str] = []
    # What the log already holds for the records selected, so this run measures
    # only the rest and the summary can count both.
    from_the_log: dict[str, str] = {}
    resume = the_resume_command(asked.log, asked.wait_until_quiet) if asked.log else None
    if picked_up is not None:
        try:
            refused = why_a_resume_is_refused(the_guarded_files_git_sees_as_modified(guards))
            if refused:
                print(f"\n{refused}\n")
                return 1
            from_the_log = verdicts_in(picked_up)
        except Wrong as wrong:
            print(f"\n{picked_up_from}: {wrong}\n")
            return 1
        from_the_log = {g.name: from_the_log[g.name] for g in guards if g.name in from_the_log}
        slipped += [name for name, verdict in from_the_log.items() if verdict != "agreed"]
        guards = [g for g in guards if g.name not in from_the_log]
        print(
            f"Resuming from {picked_up_from}: {len(from_the_log)} of {selected} "
            f"already measured, {len(guards)} to go",
            flush=True,
        )

    # A chunk of a known size, cut before the pre-read so only the suites the
    # chunk's records name are read: the file names 23 integration targets
    # besides the library, and a one-record chunk read all of them once.
    # Counted by records started, contended ones included, or a machine that
    # is never quiet would never stop.
    if asked.stop_after is not None:
        guards = guards[: asked.stop_after]

    # The pre-read is taken again on every invocation, a resumed one included,
    # because the tree may have moved between invocations and a pre-read kept
    # from the log would blame or excuse the wrong failures. Nothing to measure
    # this invocation means nothing to pre-read: --stop-after 0 reports what
    # remains and a resume with nothing left says so, both without a build.
    if guards:
        print(f"{the_header_for(len(guards), shard)}\n", flush=True)
    # Once per suite, before anything is broken, so an unrelated failure is not
    # blamed on every break in turn. See `what_is_already_failing`, and the
    # deadlock it describes, which is why this is not optional.
    already_failing: dict[tuple[str, ...], set[str]] = {}
    given_up_suites: set[tuple[str, ...]] = set()
    try:
        if guards and asked.wait_until_quiet:
            wait_until_quiet()
    except Wrong as wrong:
        print(f"\n{wrong}\n")
        return 1
    for suite in {guard.suite for guard in guards}:
        # Announced, because this is a whole suite run per distinct suite before
        # any break is applied, and it is the first two minutes of every run.
        # Silence for two minutes reads as a hang, and a check that reads as
        # hung is one somebody kills.
        print(
            f"Reading what already fails with {' '.join(suite)}, "
            "before anything is broken.",
            flush=True,
        )
        try:
            # Unfiltered even under --named-only: this reads what is already
            # broken before any break is applied, and a filtered reading of
            # that would miss the failures it exists to subtract.
            already_failing[suite] = what_is_already_failing(
                suite, Budget.starting_now(asked.time_limit)
            )
        except GaveUp as ran_out:
            # A pre-read that does not return used to end the shard, because
            # the branch below returns 1 for every `Wrong` and this is a whole
            # suite build and run with nothing bounding it. Shard 40 of run
            # 35520204784 spent 809 s in its first pre-read alone. So instead:
            # every record of this suite is reported unmeasured, one line each
            # so `the_verdict_on` reads each and a resume takes each again, and
            # the next suite is read.
            #
            # The other `Wrong` cases stay fatal. A pre-read that fails to
            # build is a different diagnosis, and widening this to cover it is
            # a change nobody has argued for.
            expired = the_line_for_a_record_whose_pre_read_expired(suite, ran_out)
            for waiting in (g for g in guards if g.suite == suite):
                print(f"-- {waiting.name}", flush=True)
                print(f"{expired}\n", flush=True)
                gave_up.append(waiting.name)
            given_up_suites.add(suite)
            continue
        except Wrong as wrong:
            print(f"\nThe tree could not be read before breaking it: {wrong}\n")
            return 1
        if already_failing[suite]:
            print(
                f"Already failing before any break, with {' '.join(suite)}, and "
                "therefore not counted against any record below:"
            )
            for name in sorted(already_failing[suite]):
                print(f"    {name}")
            print(
                "\nThat is worth fixing on its own. A measurement taken against "
                "a tree that is not green is a weaker measurement, even with "
                "the arithmetic corrected.\n"
            )

    # The records of a suite whose pre-read expired are already reported above,
    # so the loop does not reach them: there is nothing to subtract their
    # already-failing set from.
    guards = [guard for guard in guards if guard.suite not in given_up_suites]

    judged_this_run = 0
    with tempfile.TemporaryDirectory(prefix="wixen-guards-") as made:
        scratch = Path(made)
        for guard in guards:
            try:
                if asked.wait_until_quiet:
                    wait_until_quiet()
                # Flushed, because this is the only sign of life a run gives
                # and a run here is hours. Python block-buffers a redirected
                # stdout, so without this a sweep writes nothing to its log
                # until it exits: a 220-record run showed an empty file for
                # seven hours, and a 33-record re-measurement killed at about
                # 50 minutes had already written 31 counts into the record with
                # its whole report still in the buffer, leaving a committable
                # artefact and no evidence for it. A check that cannot be
                # watched is one somebody kills.
                print(f"-- {guard.name}", flush=True)
                held = measured_and_said(
                    guard,
                    scratch,
                    already_failing.get(guard.suite),
                    asked.named_only,
                    Budget.starting_now(asked.time_limit),
                )
                # Polled once the run has returned. A cargo alive now either
                # ran beside the suite or started as it ended, and this cannot
                # tell which, so the record is unmeasured either way and a
                # resume takes it again. A build that started and finished
                # inside the run is the gap this cheapest reading leaves.
                foreign = (
                    foreign_builds_in(what_is_running(), set())
                    if asked.wait_until_quiet
                    else []
                )
            except Wrong as wrong:
                print(f"\n{wrong}\n")
                return 1
            if foreign:
                print(CONTENDED, flush=True)
                for process in foreign:
                    print(f"       {process}", flush=True)
                contended.append(guard.name)
            elif held == "given up on":
                gave_up.append(guard.name)
            elif held == "agreed":
                agreed.append(guard)
                judged_this_run += 1
            else:
                slipped.append(guard.name)
                judged_this_run += 1

    # The tree a record agreed against, written down so a source read can
    # notice it moving. Only for the records that agreed, and that is the
    # load-bearing half: writing it for a record whose red list is wrong would
    # say "checked against this tree" over a record still known to be short,
    # and the check would then stay quiet about it for ever. A record that
    # disagreed gets its `red` list corrected by hand and measured again, and
    # it is that second run that writes the count.
    #
    # Only under --remeasure. An ordinary run is a report, and a report that
    # edits the thing it reports on is not one.
    #
    # And only where the run asked the whole question. `why_the_counts_may_not_be_written`
    # says what a filtered run did not ask and why that is not a count.
    if asked.remeasure and agreed:
        forbidden = why_the_counts_may_not_be_written(named_only=asked.named_only)
        if forbidden:
            print(f"\n{forbidden}")
        else:
            write_down_the_counts(
                read_record(), {g.name: what_the_tree_holds_now(g) for g in agreed}
            )
            print(
                f"\nWrote down the tree {how_many(len(agreed), 'record')} agreed "
                "with, so a test added\nto any file they name fails the commit that "
                "adds it."
            )

    # Both ways written out rather than one built from parts. Three words have
    # to agree in number and this project has already read out "1 changes are
    # waiting here" to somebody.
    print()
    if contended:
        print(
            "1 record had another cargo alive as its run returned, so it is "
            "unmeasured and a resume takes it again:"
            if len(contended) == 1
            else f"{len(contended)} records had another cargo alive as their runs "
            "returned, so they are unmeasured and a resume takes them again:"
        )
        for name in contended:
            print(f"    {name}")
        print()
    if gave_up:
        print(
            "1 record was given up on at its time limit, so it is unmeasured "
            "and a resume takes it again:"
            if len(gave_up) == 1
            else f"{len(gave_up)} records were given up on at their time "
            "limit, so they are unmeasured and a resume takes them again:"
        )
        for name in gave_up:
            print(f"    {name}")
        print()
    # The last line, which is how somebody told not to read the verdicts knows
    # whether the sweep is done. A stopped run exits 0 whatever it found so
    # far, because stopping was asked for and the chunk is reported as one;
    # only a run that finished the selection answers with its exit status. A
    # run that gave up on something is neither: nobody asked for that, so it
    # answers 1 wherever it would otherwise have answered 0.
    this_run = judged_this_run
    remaining = selected - len(from_the_log) - this_run
    closing = the_closing_line(
        selected, len(from_the_log), this_run, resume, len(gave_up)
    )
    if slipped:
        print(
            "1 guard is not what the record says it is:"
            if len(slipped) == 1
            else f"{len(slipped)} guards are not what the record says they are:"
        )
        for name in slipped:
            print(f"    {name}")
        earlier = sum(1 for name in slipped if name in from_the_log)
        if earlier == len(slipped) == 1:
            print("\nThat verdict came from the log.")
        elif earlier:
            print(f"\n{earlier} of those {len(slipped)} verdicts came from the log.")
        print(
            "\nA named test that stayed green is a test whose name still "
            "promises\nsomething it no longer checks. A test that went red "
            "and is not named is\na guard nobody wrote down, and a record "
            "shorter than the truth is what\nthis file exists to stop. Either "
            "way: measure it by hand and write down\nwhat it really does now."
            f"\n\n{closing}"
        )
        return 0 if remaining and not gave_up else 1
    if remaining:
        print(closing)
        return 1 if gave_up else 0
    if asked.named_only:
        # Never "and nothing else does", because this run did not ask. Said
        # every time rather than once at the top, since the last line is what
        # gets quoted into a summary and a clean-looking one that answered half
        # the question is how a weaker check becomes indistinguishable from the
        # stronger one it replaced.
        # `how_many` rather than a bare count, for the reason `how_many` exists:
        # the sibling line below already has a singular branch and this one did
        # not, so somebody watched a real run print "All 1 guards".
        opening = (
            "The guard still reddens the tests its record names."
            if selected == 1
            else f"All {selected} guards still redden the tests their records name."
        )
        print(
            f"{opening}\n"
            "\n"
            "**This run did not ask whether anything else went red.** It ran\n"
            "only the modules those tests live in, so a test elsewhere that the\n"
            "break also reddens is neither run nor reported, and a record that\n"
            "names too few still reads as correct here. That is the direction 21\n"
            "of the 23 records found wrong on 2026-09-01 were wrong in.\n"
            "\n"
            f"Run the same selection without --named-only to ask it. That is\n"
            f"{THE_COST_OF_ASKING_PROPERLY}\n"
            f"\n{closing}"
        )
        return 0
    print(
        "The guard still goes red when what it defends breaks, and nothing else does."
        if selected == 1
        else f"All {selected} guards redden exactly the tests their records name."
    )
    print(f"\n{closing}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

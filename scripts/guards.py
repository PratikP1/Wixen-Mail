"""Take every recorded guard measurement again, at whatever HEAD is now.

Read `scripts/guards.sh` for why this exists. This file is the mechanism.

Never changes anything through git. Every file it is about to edit is copied
byte for byte into a scratch directory keyed by its whole path, and put back
from there whether the run finishes, fails or is interrupted. Two files in this
project are both called `calendar.rs`, so the key is the whole path and never
the basename. It reads git in exactly one place, `files_changed_since`, to work
out which records a branch could have disturbed.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
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
# one thread per core is the worst of the five and more than twice the cost of
# the best. Every guard record pays this once, so 536 records is 31 hours at
# the default and 14 at four threads.
#
# What contends was not diagnosed. Overridable rather than fixed, because the
# shape of that curve belongs to this machine: on a two-core CI runner the
# default is already below the turning point and forcing four would be worse.
TEST_THREADS = os.environ.get("WIXEN_TEST_THREADS", "4")

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


class Wrong(Exception):
    """A guard is not what the record says it is."""


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
    likely to reach the break as one arriving anywhere else, and 34 of the 548
    records break a Rust file no test in their red list lives in. "the question
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
    """
    kept: list[str] = []
    dropping = False
    for line in block:
        if line.startswith("tests_last_seen = ["):
            dropping = True
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
    suite: tuple[str, ...], filters: list[str] | None = None
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

    The arithmetic looked good: the rebuild a break forces is 23 seconds and the
    library is 89, so filtering would take a 220-record sweep from 6.8 hours to
    about 88 minutes. Two things spoil it.

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
    finished = subprocess.run(
        ["cargo", "test", *suite, "--", f"--test-threads={TEST_THREADS}", *filtered],
        cwd=ROOT,
        capture_output=True,
        text=True,
        # Both named, and neither is a preference.
        #
        # `text=True` alone decodes with the locale codec, which on this machine
        # is cp1252. A test name or a panic message carrying a byte cp1252
        # cannot decode kills the reader thread, and `stdout` is then never
        # assigned: that is the `NoneType + str` that ended a sweep on its 122nd
        # record, and the decode traceback sits in that run's own log underneath
        # the failure it caused.
        #
        # `errors="replace"` rather than strict, because a mangled character in
        # a panic message must not cost an hour of measuring. What this reads
        # out are test paths, and those are ASCII.
        encoding="utf-8",
        errors="replace",
    )
    # Both halves can come back as None, which is not what the documentation
    # for `capture_output` says and was seen anyway: a sweep of 208 records
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

    said = finished.stdout + finished.stderr
    verdicts = {name: verdict for name, verdict in VERDICT.findall(said)}
    if not verdicts:
        raise Wrong(why_no_test_was_named(finished.returncode, said))
    return verdicts


def what_is_already_failing(suite: tuple[str, ...]) -> set[str]:
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
    """
    verdicts = run_the_whole_suite(suite)
    return {name for name, verdict in verdicts.items() if verdict == "FAILED"}


def measure(
    guard: Guard,
    scratch: Path,
    already_failing: set[str] | None = None,
    named_only: bool = False,
) -> Measured:
    """Apply the break, run the guard's own suite, put the file back."""
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
            guard.suite, the_filters_for(guard.red) if named_only else None
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
    asked = parsing.parse_args()

    try:
        guards = read_record()
    except Wrong as wrong:
        print(f"\n{wrong}\n")
        return 1

    if asked.recount_everything:
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
        # Said out loud, because a run that quietly measured 14 of 536 and
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

    header = (
        "== 1 guard, one build and one run =="
        if len(guards) == 1
        else f"== {len(guards)} guards, one build and one run each =="
    )
    print(f"{header}\n", flush=True)
    slipped: list[str] = []
    agreed: list[Guard] = []
    # Once per suite, before anything is broken, so an unrelated failure is not
    # blamed on every break in turn. See `what_is_already_failing`, and the
    # deadlock it describes, which is why this is not optional.
    already_failing: dict[tuple[str, ...], set[str]] = {}
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
            already_failing[suite] = what_is_already_failing(suite)
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

    with tempfile.TemporaryDirectory(prefix="wixen-guards-") as made:
        scratch = Path(made)
        for guard in guards:
            # Flushed, because this is the only sign of life a run gives and a
            # run here is hours. Python block-buffers a redirected stdout, so
            # without this a sweep writes nothing to its log until it exits: a
            # 220-record run showed an empty file for seven hours, and a
            # 33-record re-measurement killed at about 50 minutes had already
            # written 31 counts into the record with its whole report still in
            # the buffer, leaving a committable artefact and no evidence for it.
            # A check that cannot be watched is one somebody kills.
            print(f"-- {guard.name}", flush=True)
            try:
                measured = measure(
                    guard,
                    scratch,
                    already_failing.get(guard.suite),
                    named_only=asked.named_only,
                )
            except Wrong as wrong:
                print(f"   {wrong}\n")
                slipped.append(guard.name)
                continue
            except Exception as broke:
                # One record must not take the run down with it. A sweep of 208
                # records is hours of building and running, and losing all of it
                # to an unexpected failure on one is how a check nobody can
                # afford to finish becomes a check nobody runs.
                #
                # Counted as slipped rather than passed, because a record that
                # could not be measured is not a record that holds. The file is
                # already restored: `measure` puts it back in a `finally`.
                #
                # KeyboardInterrupt and SystemExit are not `Exception`, so an
                # interrupt still stops the run and still restores the tree.
                print(f"   this record could not be measured: {broke!r}\n")
                slipped.append(guard.name)
                continue
            say_what_it_found(guard, measured)
            if measured.agrees_with_the_record():
                agreed.append(guard)
            else:
                slipped.append(guard.name)

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
    if asked.remeasure and agreed:
        write_down_the_counts(read_record(), {g.name: what_the_tree_holds_now(g) for g in agreed})
        print(
            f"\nWrote down the tree {how_many(len(agreed), 'record')} agreed "
            "with, so a test added\nto any file they name fails the commit that "
            "adds it."
        )

    # Both ways written out rather than one built from parts. Three words have
    # to agree in number and this project has already read out "1 changes are
    # waiting here" to somebody.
    print()
    if slipped:
        print(
            "1 guard is not what the record says it is:"
            if len(slipped) == 1
            else f"{len(slipped)} guards are not what the record says they are:"
        )
        for name in slipped:
            print(f"    {name}")
        print(
            "\nA named test that stayed green is a test whose name still "
            "promises\nsomething it no longer checks. A test that went red "
            "and is not named is\na guard nobody wrote down, and a record "
            "shorter than the truth is what\nthis file exists to stop. Either "
            "way: measure it by hand and write down\nwhat it really does now."
        )
        return 1
    if asked.named_only:
        # Never "and nothing else does", because this run did not ask. Said
        # every time rather than once at the top, since the last line is what
        # gets quoted into a summary and a clean-looking one that answered half
        # the question is how a weaker check becomes indistinguishable from the
        # stronger one it replaced.
        print(
            f"All {len(guards)} guards still redden the tests their records name.\n"
            "\n"
            "**This run did not ask whether anything else went red.** It ran\n"
            "only the modules those tests live in, so a test elsewhere that the\n"
            "break also reddens is neither run nor reported, and a record that\n"
            "names too few still reads as correct here. That is the direction 21\n"
            "of the 23 records found wrong on 2026-09-01 were wrong in.\n"
            "\n"
            "Run the same selection without --named-only to ask it. That is\n"
            "about 112 seconds a record against 24."
        )
        return 0
    print(
        "The guard still goes red when what it defends breaks, and nothing else does."
        if len(guards) == 1
        else f"All {len(guards)} guards redden exactly the tests their records name."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

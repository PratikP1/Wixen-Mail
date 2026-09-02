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


def run_the_whole_suite(suite: tuple[str, ...]) -> dict[str, str]:
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
    """
    finished = subprocess.run(
        ["cargo", "test", *suite, "--", f"--test-threads={TEST_THREADS}"],
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


def measure(guard: Guard, scratch: Path) -> Measured:
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
        verdicts = run_the_whole_suite(guard.suite)
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
    asked = parsing.parse_args()

    try:
        guards = read_record()
    except Wrong as wrong:
        print(f"\n{wrong}\n")
        return 1

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
    print(f"{header}\n")
    slipped: list[str] = []
    with tempfile.TemporaryDirectory(prefix="wixen-guards-") as made:
        scratch = Path(made)
        for guard in guards:
            print(f"-- {guard.name}")
            try:
                measured = measure(guard, scratch)
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
            if not measured.agrees_with_the_record():
                slipped.append(guard.name)

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
    print(
        "The guard still goes red when what it defends breaks, and nothing else does."
        if len(guards) == 1
        else f"All {len(guards)} guards redden exactly the tests their records name."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

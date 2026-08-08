"""Take every recorded guard measurement again, at whatever HEAD is now.

Read `scripts/guards.sh` for why this exists. This file is the mechanism.

Never touches git. Every file it is about to edit is copied byte for byte into
a scratch directory keyed by its whole path, and put back from there whether
the run finishes, fails or is interrupted. Two files in this project are both
called `calendar.rs`, so the key is the whole path and never the basename.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECORD = ROOT / "guards" / "guards.toml"

# `test <name> ... ok` or `... FAILED`, as the test harness writes it.
VERDICT = re.compile(r"^test (\S+) \.\.\. (ok|FAILED)$", re.M)


@dataclass(frozen=True)
class Guard:
    name: str
    file: Path
    before: str
    after: str
    red: tuple[str, ...]


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


def run_the_whole_library() -> dict[str, str]:
    """Every test in the library, and whether it passed. One build, one run.

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
    """
    finished = subprocess.run(
        ["cargo", "test", "--lib"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    said = finished.stdout + finished.stderr
    verdicts = {name: verdict for name, verdict in VERDICT.findall(said)}
    if not verdicts:
        raise Wrong(
            "the test harness ran nothing at all, so the break did not build:\n"
            + said[-4000:]
        )
    return verdicts


def measure(guard: Guard, scratch: Path) -> Measured:
    """Apply the break, run the library, put the file back."""
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
        verdicts = run_the_whole_library()
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

    header = (
        "== 1 guard, one build and one run of the library =="
        if len(guards) == 1
        else f"== {len(guards)} guards, one build and one run of the library each =="
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

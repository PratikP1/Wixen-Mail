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
import shutil
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


def run_the_named_tests(names: tuple[str, ...]) -> dict[str, str]:
    """Each named test, and whether it passed. One build, one run."""
    finished = subprocess.run(
        ["cargo", "test", "--lib", "--", "--exact", *names],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    said = finished.stdout + finished.stderr
    verdicts = {name: verdict for name, verdict in VERDICT.findall(said)}
    missing = [name for name in names if name not in verdicts]
    if missing:
        raise Wrong(
            "the test harness never ran "
            + ", ".join(missing)
            + ".\nEither the name is wrong or the build failed:\n"
            + said[-4000:]
        )
    return verdicts


def measure(guard: Guard, scratch: Path) -> list[str]:
    """Apply the break, run the tests it should redden, put the file back."""
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
        broken = guard.file.read_text(encoding="utf-8").replace(
            guard.before, guard.after
        )
        guard.file.write_text(broken, encoding="utf-8")
        verdicts = run_the_named_tests(guard.red)
    finally:
        # The bytes, not the timestamps: a restored file with its old
        # modification time reads to cargo as one that never changed, and the
        # next run answers out of the broken binary.
        guard.file.write_bytes(kept.read_bytes())
        kept.unlink()
    return [name for name in guard.red if verdicts[name] != "FAILED"]


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
        "== 1 guard, one build =="
        if len(guards) == 1
        else f"== {len(guards)} guards, one build each =="
    )
    print(f"{header}\n")
    slipped: list[str] = []
    with tempfile.TemporaryDirectory(prefix="wixen-guards-") as made:
        scratch = Path(made)
        for guard in guards:
            print(f"-- {guard.name}")
            try:
                still_green = measure(guard, scratch)
            except Wrong as wrong:
                print(f"   {wrong}\n")
                slipped.append(guard.name)
                continue
            if still_green:
                print(
                    f"   {len(still_green)} of {len(guard.red)} named tests "
                    "stayed green with the guard broken:"
                )
                for name in still_green:
                    print(f"       {name}")
                print()
                slipped.append(guard.name)
            elif len(guard.red) == 1:
                print("   the one test named went red")
            else:
                print(f"   all {len(guard.red)} tests named went red")

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
            "\nEach one is a test whose name still promises something it no "
            "longer\nchecks. Measure it by hand and either restore what it "
            "discriminates or\nwrite down what it really does now."
        )
        return 1
    print(
        "The guard still goes red when what it defends breaks."
        if len(guards) == 1
        else f"All {len(guards)} guards still go red when what they defend breaks."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

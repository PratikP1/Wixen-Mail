"""Read what a mutation run really did, and refuse to call anything less an answer.

Read `scripts/mutants.sh` for why this exists. This file is the mechanism.

The instrument had been filing untested mutants as tested. In the whole-tree run
of 2026-08-05, 595 of 1470 mutants were recorded as unviable, which reads as
"the compiler rejected this mutant". Only 122 of them were. The other 473 never
built at all: the compiler was launched and died before printing a byte, each in
a tenth of a second, and a failed build is filed under one heading whichever way
it failed. So a third of that run was never tested and nothing said so.

That is this project's worst defect shape, a lenient reader and a strict writer
answering one question. The writer records a status for every phase of every
mutant. The reader asked only whether a results file existed, and that file is
created empty before anything is built, so a run that tested nothing read as a
run that caught everything.

Two rules follow, and they are the whole point of this file. Every count and
every name comes from the one record of what happened, never from the lists
written alongside it for people, because a count read from one place and a list
read from another disagree first on a partial run, which is the case nobody
looks at. And a build that failed is asked which way it failed before it is
filed.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

from guards import how_many

# What cargo-mutants exits with, from its own list of codes. A bare number here
# is a number somebody has to go and look up, and nobody does.
MEANINGS = {
    0: "every mutant was caught",
    1: "the arguments were wrong",
    2: "some mutants were missed",
    3: "some tests timed out",
    4: "the tests were already failing before anything was changed",
    5: "the list of changes does not match the code in the tree",
    6: "the list of changes could not be read",
    70: "it hit an error inside itself",
}


class Wrong(Exception):
    """The run cannot be read, so there is nothing to report."""


@dataclass(frozen=True)
class Run:
    """What really happened to each mutant, and how many there were meant to be.

    `declared` is how many mutants were written down before the run started, and
    it is the only number that can say whether the run finished. The tool's own
    summary counts what it processed, so on the run whose build failed it said
    zero mutants and meant "I never got to any of them".

    Two of the counts below answer questions that sound like one question and
    are not, so they are pinned here together on one run rather than apart. A
    mutant the compiler rejected was really tested, because it was asked and
    something answered. The suite never saw it, so it never reached the tests:

    >>> rejected = Run(declared=3, would_not_compile=["u"] * 3)
    >>> rejected.really_tested
    3
    >>> rejected.reached_the_tests
    0
    """

    declared: int
    caught: list[str] = field(default_factory=list)
    missed: list[str] = field(default_factory=list)
    timed_out: list[str] = field(default_factory=list)
    would_not_compile: list[str] = field(default_factory=list)
    # Each with the status it died on, in one field rather than two, so the
    # count and the cause it is blamed on cannot come apart.
    never_started: list[tuple[str, int]] = field(default_factory=list)
    baseline_failed: bool = False

    @property
    def processed(self) -> int:
        """How many mutants the run reached, whatever happened to them."""
        return self.really_tested + len(self.never_started)

    @property
    def really_tested(self) -> int:
        """How many mutants were really put to the question.

        One the compiler rejected counts. It was asked and something answered.
        One that never started does not, and that is the whole distinction.
        """
        return (
            len(self.caught)
            + len(self.missed)
            + len(self.timed_out)
            + len(self.would_not_compile)
        )

    @property
    def reached_the_tests(self) -> int:
        """How many mutants the suite was really run against.

        The one number that can say whether this run learned anything. A
        mutant the compiler rejected was tested and never reached a test, so
        `really_tested` counts it and this does not, and that difference is
        the whole of why both are here.
        """
        return len(self.caught) + len(self.missed) + len(self.timed_out)


def windows_status(status: int) -> str:
    """The status Windows really returned, written the way Windows writes it.

    A failed process arrives here as a signed integer, so the plain hexadecimal
    of it is `-0x3ffffebe`, which names nothing and cannot be searched for. The
    same number unsigned is a status anybody can look up in a minute.

    >>> windows_status(-1073741502)
    '0xC0000142'
    >>> windows_status(101)
    '0x00000065'
    """
    return f"0x{status & 0xFFFFFFFF:08X}"


def a_process_that_never_started(status: int) -> bool:
    """Whether this status means the program never ran, rather than that it failed.

    The whole job turns on this. A compiler that rejects a mutant exits 101 and
    says why in the log, and that mutant was genuinely tested. A compiler that
    never started exits with a Windows status and says nothing, and that mutant
    was not tested at all. Both are recorded as a failed build.

    An exit code is a byte. Anything outside a byte did not come from the
    program, it came from Windows saying the program never got to run.

    >>> a_process_that_never_started(-1073741502)
    True
    >>> a_process_that_never_started(101)
    False
    >>> a_process_that_never_started(1)
    False
    >>> a_process_that_never_started(0)
    False
    """
    return not 0 <= status <= 255


def build_failure_status(phase_results: list[dict]) -> int | None:
    """The status the build exited with, or None if the build did not fail.

    The build phase only. A test phase can end on an access violation because
    the mutant really did break the code that badly, and that is a mutant the
    suite caught. Reading the same rule over both phases would turn real catches
    into refusals.

    >>> build_failure_status([{"phase": "Build", "process_status": "Success"},
    ...                       {"phase": "Test", "process_status": {"Failure": 101}}])
    >>> build_failure_status([{"phase": "Build",
    ...                        "process_status": {"Failure": -1073741502}}])
    -1073741502
    """
    for phase in phase_results:
        if phase["phase"] != "Build":
            continue
        status = phase["process_status"]
        if isinstance(status, dict) and "Failure" in status:
            return status["Failure"]
    return None


def run_from_outcomes(declared: int, outcomes: dict) -> Run:
    """Sort every mutant by what really happened to it.

    The fixture is the shape of the run of 2026-08-09 in miniature: one mutant
    the suite caught, one the compiler rejected, and one that never built. The
    tool files the last two under the same heading, and the reader that shipped
    counted both as tested.

    >>> outcomes = {"outcomes": [
    ...     {"scenario": "Baseline", "summary": "Success",
    ...      "phase_results": [{"phase": "Build", "process_status": "Success"},
    ...                        {"phase": "Test", "process_status": "Success"}]},
    ...     {"scenario": {"Mutant": {"name": "one the suite caught"}},
    ...      "summary": "CaughtMutant",
    ...      "phase_results": [{"phase": "Build", "process_status": "Success"},
    ...                        {"phase": "Test", "process_status": {"Failure": 101}}]},
    ...     {"scenario": {"Mutant": {"name": "one the compiler rejected"}},
    ...      "summary": "Unviable",
    ...      "phase_results": [{"phase": "Build", "process_status": {"Failure": 101}}]},
    ...     {"scenario": {"Mutant": {"name": "one that never built"}},
    ...      "summary": "Unviable",
    ...      "phase_results": [{"phase": "Build",
    ...                         "process_status": {"Failure": -1073741502}}]},
    ... ]}
    >>> run = run_from_outcomes(3, outcomes)
    >>> run.caught
    ['one the suite caught']
    >>> run.would_not_compile
    ['one the compiler rejected']
    >>> run.never_started
    [('one that never built', -1073741502)]
    >>> run.baseline_failed
    False
    >>> run.declared, run.processed, run.really_tested
    (3, 3, 2)

    A baseline that did not succeed is the run saying it never got started at
    all, and it is recorded as an ordinary failure with an ordinary exit code,
    because what died was something the compiler ran rather than the compiler.

    >>> stopped = {"outcomes": [{"scenario": "Baseline", "summary": "Failure",
    ...     "phase_results": [{"phase": "Build",
    ...                        "process_status": {"Failure": 101}}]}]}
    >>> run_from_outcomes(32, stopped).baseline_failed
    True
    """
    sorted_by_name = {
        "CaughtMutant": [],
        "MissedMutant": [],
        "Timeout": [],
        "Unviable": [],
        "never started": [],
    }
    baseline_failed = False
    for outcome in outcomes.get("outcomes", []):
        scenario = outcome["scenario"]
        if not isinstance(scenario, dict) or "Mutant" not in scenario:
            baseline_failed = baseline_failed or outcome["summary"] != "Success"
            continue
        name = scenario["Mutant"]["name"]
        status = build_failure_status(outcome["phase_results"])
        if status is not None and a_process_that_never_started(status):
            sorted_by_name["never started"].append((name, status))
            continue
        if outcome["summary"] not in sorted_by_name:
            raise Wrong(f"{name}: the run recorded {outcome['summary']}, which is new")
        sorted_by_name[outcome["summary"]].append(name)
    return Run(
        declared=declared,
        caught=sorted_by_name["CaughtMutant"],
        missed=sorted_by_name["MissedMutant"],
        timed_out=sorted_by_name["Timeout"],
        would_not_compile=sorted_by_name["Unviable"],
        never_started=sorted_by_name["never started"],
        baseline_failed=baseline_failed,
    )


def read_run(out_dir: Path) -> Run:
    """Load one run from the directory the tool wrote it to.

    How many mutants there were meant to be comes from the list the tool wrote
    before it started, and what happened to them comes from the record it wrote
    as it went. Those are two different questions and they have two different
    sources here on purpose: the run whose build failed left a list of 32
    mutants next to a summary saying zero.
    """
    declared_in = out_dir / "mutants.json"
    happened_in = out_dir / "outcomes.json"
    for record in (declared_in, happened_in):
        if not record.exists():
            raise Wrong(
                f"There is no run to read in {out_dir}: {record.name} is not there.\n"
                "The run died before it wrote anything down."
            )
    declared = json.loads(declared_in.read_text(encoding="utf-8"))
    happened = json.loads(happened_in.read_text(encoding="utf-8"))
    return run_from_outcomes(len(declared), happened)


def why_this_is_not_an_answer(run: Run) -> str | None:
    """Why this run cannot be read as a result, or None when it can.

    Nothing built, so nothing was asked:

    >>> print(why_this_is_not_an_answer(Run(declared=32, baseline_failed=True)))
    The build failed before anything was changed, so none of the 32 mutants was tested.

    The run of 2026-08-05, which was reported as 734 caught and 60 missed. Two
    thirds of what it filed as rejected by the compiler had never reached one:

    >>> print(why_this_is_not_an_answer(Run(declared=1470, caught=["c"] * 734,
    ...     missed=["m"] * 60, timed_out=["t"] * 81, would_not_compile=["u"] * 122,
    ...     never_started=[("n", -1073741502)] * 473)))
    473 of the 1470 mutants never built, so this run is not an answer.
    The compiler exited with Windows status 0xC0000142 and printed nothing.
    That is Windows refusing to start it. Only 997 mutants were really tested.

    A run read before it finished. This project has quoted seventeen of eighteen
    caught out of a run whose real figure was thirty-seven of fifty-one:

    >>> print(why_this_is_not_an_answer(Run(declared=51, caught=["c"] * 18)))
    The run stopped after 18 of the 51 mutants, so this is part of an answer.
    Wait for it to finish, or read it as the part it is.

    A run where the suite was never once run against a mutant. This can happen
    two ways and the two need telling apart, because one is a machine to go and
    look at and the other is a set of lines with nothing in them to change.
    Every mutant rejected by the compiler is the first:

    >>> print(why_this_is_not_an_answer(Run(declared=3,
    ...     would_not_compile=["u"] * 3)))
    None of the 3 mutants reached the test suite: the compiler rejected every one.
    The suite never ran, so this run says nothing about what it would notice.

    No mutants at all is the second:

    >>> print(why_this_is_not_an_answer(Run(declared=0)))
    There were no mutants, because nothing in these lines can be mutated.
    That is not the same as every mutant being caught, and it is not a result.

    And the run of 2026-08-11, which is a real result whether or not anything
    was missed. A check that has never said yes is as untested as one that has
    never said no:

    >>> why_this_is_not_an_answer(Run(declared=24, caught=["c"] * 19,
    ...     missed=["m"] * 2, would_not_compile=["u"] * 3)) is None
    True

    One real answer among rejections is still an answer:

    >>> why_this_is_not_an_answer(Run(declared=4, caught=["c"],
    ...     would_not_compile=["u"] * 3)) is None
    True
    """
    if run.baseline_failed:
        return (
            "The build failed before anything was changed, so none of the "
            f"{run.declared} mutants was tested."
        )
    if run.never_started:
        return (
            f"{len(run.never_started)} of the {run.declared} mutants never built, "
            "so this run is not an answer.\n"
            "The compiler exited with Windows status "
            f"{' and '.join(sorted({windows_status(s) for _, s in run.never_started}))}"
            " and printed nothing.\n"
            "That is Windows refusing to start it. Only "
            f"{run.really_tested} mutants were really tested."
        )
    if run.processed < run.declared:
        return (
            f"The run stopped after {run.processed} of the {run.declared} mutants, "
            "so this is part of an answer.\n"
            "Wait for it to finish, or read it as the part it is."
        )
    # Last, so the refusals above keep saying the more particular thing when
    # they apply. Getting here means every mutant that was declared was
    # reached and not one of them was ever handed to the suite, which happens
    # two ways, and a check that can fail two ways has to say which.
    if run.reached_the_tests == 0:
        if run.declared == 0:
            return (
                "There were no mutants, because nothing in these lines can be mutated.\n"
                "That is not the same as every mutant being caught, and it is not a result."
            )
        return (
            f"None of the {run.declared} mutants reached the test suite: "
            "the compiler rejected every one.\n"
            "The suite never ran, so this run says nothing about what it would notice."
        )
    return None


def why_there_was_nothing_to_check(base: str, base_hash: str, head_hash: str) -> str:
    """Which of the two ways a list of changes comes out empty.

    Documented for 275 commits as the way to check a branch, and on this
    repository every commit lands on the branch it names, so it has always
    compared one commit with itself and always found nothing:

    >>> print(why_there_was_nothing_to_check("main", "818bd83", "818bd83"))
    There is nothing to check because main and the current commit are both 818bd83.
    Comparing a commit with itself can only ever produce an empty list.

    Which is a different thing from a change that really did not touch any code:

    >>> print(why_there_was_nothing_to_check("v0.19.0", "93fdd5e", "818bd83"))
    Nothing under src changed between 93fdd5e and 818bd83, so there is nothing to check.
    """
    if base_hash == head_hash:
        return (
            f"There is nothing to check because {base} and the current commit "
            f"are both {base_hash}.\n"
            "Comparing a commit with itself can only ever produce an empty list."
        )
    return (
        f"Nothing under src changed between {base_hash} and {head_hash}, "
        "so there is nothing to check."
    )


def which_way_cargo_mutants_failed(status: int) -> str:
    """What the number the tool exited with means, in words.

    >>> which_way_cargo_mutants_failed(0)
    'It exited 0: every mutant was caught.'
    >>> which_way_cargo_mutants_failed(2)
    'It exited 2: some mutants were missed.'
    >>> which_way_cargo_mutants_failed(4)
    'It exited 4: the tests were already failing before anything was changed.'
    >>> which_way_cargo_mutants_failed(5)
    'It exited 5: the list of changes does not match the code in the tree.'
    >>> which_way_cargo_mutants_failed(9)
    'It exited 9, which is not one of the codes it documents.'
    """
    if status not in MEANINGS:
        return f"It exited {status}, which is not one of the codes it documents."
    return f"It exited {status}: {MEANINGS[status]}."


def say_what_it_found(run: Run) -> None:
    """Print every bucket, so no run can be summed up by naming two of five.

    Only ever reached for a run that put a mutant to the suite: everything
    else is refused before this is called, so there is nothing here about a
    run with no mutants in it. There was, and two places answering "was there
    anything to test" is how this project loses a day.

    >>> say_what_it_found(Run(declared=3, caught=["one the suite caught"],
    ...     missed=["one nothing noticed"],
    ...     would_not_compile=["one the compiler rejected"]))
    Nothing was watching this:
        one nothing noticed
    That is behaviour no test would notice losing. Either pin it or delete it.
    <BLANKLINE>
    3 mutants: 1 caught, 1 nothing noticed, 1 the compiler rejected, 0 timed out.

    >>> say_what_it_found(Run(declared=1, caught=["one the suite caught"]))
    Every mutant that built was caught.
    <BLANKLINE>
    1 mutant: 1 caught, 0 nothing noticed, 0 the compiler rejected, 0 timed out.

    """
    if run.missed:
        print("Nothing was watching this:")
        for name in run.missed:
            print(f"    {name}")
        print("That is behaviour no test would notice losing. Either pin it or delete it.")
    else:
        print("Every mutant that built was caught.")
    print()
    print(
        f"{how_many(run.declared, 'mutant')}: "
        f"{len(run.caught)} caught, "
        f"{len(run.missed)} nothing noticed, "
        f"{len(run.would_not_compile)} the compiler rejected, "
        f"{len(run.timed_out)} timed out."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "out_dir", type=Path, nargs="?", help="where the run wrote its report"
    )
    parser.add_argument(
        "--exit-status",
        type=int,
        default=None,
        help="what the run itself exited with, so the two answers can be compared",
    )
    parser.add_argument(
        "--nothing-changed",
        nargs=3,
        metavar=("BASE", "BASE_HASH", "HEAD_HASH"),
        help="say which of the two ways the list of changes came out empty",
    )
    args = parser.parse_args()

    if args.nothing_changed:
        base, base_hash, head_hash = args.nothing_changed
        print(why_there_was_nothing_to_check(base, base_hash, head_hash))
        # Comparing a commit with itself is a mistake in how it was asked, and
        # it can never test anything, so it fails. A change that really did not
        # touch any code is a legitimate way to have nothing to check.
        return 1 if base_hash == head_hash else 0

    if args.out_dir is None:
        parser.error("name the directory the run wrote its report to")

    try:
        run = read_run(args.out_dir)
    except Wrong as wrong:
        print(wrong)
        if args.exit_status is not None:
            print(which_way_cargo_mutants_failed(args.exit_status))
        return 1

    why = why_this_is_not_an_answer(run)
    if why is not None:
        print(why)
        if args.exit_status is not None:
            print(which_way_cargo_mutants_failed(args.exit_status))
        return 1

    say_what_it_found(run)

    # The tool's own verdict and this report answer one question, so a run that
    # exits non-zero over a report with nothing wrong in it is two answers to
    # one question and neither can be trusted until somebody looks.
    if args.exit_status:
        if not run.missed and not run.timed_out:
            print()
            print("The report shows nothing wrong, but the run did not think so.")
            print(which_way_cargo_mutants_failed(args.exit_status))
            print("Until those agree, this is not a result. Read the log.")
        return 1
    return 1 if run.missed else 0


if __name__ == "__main__":
    sys.exit(main())

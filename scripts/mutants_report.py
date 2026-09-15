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

Three rules follow, and they are the whole point of this file. Every count and
every name comes from the one record of what happened, never from the lists
written alongside it for people, because a count read from one place and a list
read from another disagree first on a partial run, which is the case nobody
looks at. And a build that failed is asked which way it failed before it is
filed.

The third is the same shape one layer along, and it shipped: whether the run
learned anything was read off how many mutants were handed to the suite rather
than off how many the suite came back about. A mutant the suite hung on was
handed to it, so a run in which every single one hung passed the check and was
then summed up as a run that caught everything, with nothing caught. A timeout
is a suite that never finished, and nothing is known about that mutant either
way.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

from guards import how_many, wait_until_quiet

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

    Three of the counts below answer questions that sound like one question and
    are not, so they are pinned here together on one run rather than apart. A
    mutant the compiler rejected was really tested, because it was asked and
    something answered. The suite never saw it, so it never reached the tests:

    >>> rejected = Run(declared=3, would_not_compile=["u"] * 3)
    >>> rejected.really_tested
    3
    >>> rejected.reached_the_tests
    0

    And a mutant the suite was started against and never finished against went
    all the way to the tests without the tests ever saying anything about it:

    >>> hung = Run(declared=2, timed_out=["t"] * 2)
    >>> hung.really_tested, hung.reached_the_tests, hung.answered_by_the_suite
    (2, 2, 0)
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

    @property
    def answered_by_the_suite(self) -> int:
        """How many mutants the suite ran against and finished against.

        The one number that can say whether this run learned anything, which
        `reached_the_tests` was being read as and is not. A timeout is a suite
        that was started and never got to the end, so nothing came back about
        that mutant: it was tested, it reached the tests, and the answer is
        still missing. Counting a timeout as an answer let a run in which every
        single mutant hung print that every mutant which built was caught.
        """
        return len(self.caught) + len(self.missed)


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


# ---------------------------------------------------------------------------
# A whole-tree run is a set of shards on one commit.
#
# `cargo mutants --shard k/n` divides the list the tool builds from the tree
# it runs in. Two shards from two commits are two lists: a mutant can fall
# between them or land in both, and the sum reads as a whole-tree run with
# nothing in it saying otherwise. So the script writes what each shard ran
# under before it starts, and the merge below compares those records and
# refuses a mismatch. The check lives here and not in the shell, because the
# shell has no test.


@dataclass(frozen=True)
class Conditions:
    """What one shard ran under, read from the `conditions.txt` the script
    writes before the run starts.

    Before rather than after, so a shard killed partway still says what it
    was: a shard with no record of its commit cannot be shown to belong with
    the others.
    """

    commit: str
    shard: str
    arguments: str
    copy: str
    baseline: str


def conditions_from(text: str) -> Conditions:
    """Read the five lines `scripts/mutants.sh` writes before a shard starts.

    >>> conditions_from('''commit = 1837f93b
    ... shard = 0/496
    ... arguments = --lib -- --test-threads=8
    ... copy = in-place
    ... baseline = run
    ... ''')
    Conditions(commit='1837f93b', shard='0/496', arguments='--lib -- --test-threads=8', copy='in-place', baseline='run')

    A shard run with nothing after `--` records an empty argument list, which
    is a different thing from a line that is missing:

    >>> conditions_from("commit = a\\nshard = 0/2\\narguments = \\ncopy = scratch\\nbaseline = run\\n").arguments
    ''

    A line that is missing is refused by name, because a merge comparing a
    missing commit with a missing commit would find them equal:

    >>> conditions_from("commit = a\\nshard = 0/2\\n")
    Traceback (most recent call last):
    ...
    mutants_report.Wrong: conditions.txt does not say arguments, copy or baseline, so this shard cannot be placed with the others.
    """


@dataclass(frozen=True)
class Shard:
    """One shard as the merge reads it: what it ran under, and what happened."""

    conditions: Conditions
    run: Run


def why_these_shards_are_not_one_run(shards: list[Shard]) -> str | None:
    """Why this set of shards cannot be read as one run, or None when it can.

    Every shard must say the same commit and the same arguments, sit in the
    directory its record names, and have reached its last mutant. The list is
    in shard order, so the first shard is the one the others are held to.

    >>> def at(commit, k, arguments="", declared=2, processed=2):
    ...     return Shard(Conditions(commit, f"{k}/2", arguments, "in-place", "run"),
    ...                  Run(declared=declared, caught=["c"] * processed))
    >>> why_these_shards_are_not_one_run([at("1837f93b", 0), at("1837f93b", 1)]) is None
    True

    Two commits are two lists:

    >>> print(why_these_shards_are_not_one_run([at("1837f93b", 0), at("d53893b7", 1)]))
    Shard 1/2 ran at d53893b7 and shard 0/2 at 1837f93b, so they are two lists and not one run.
    Every shard of a run is taken at one commit.

    Two argument lists are two suites, and a run the library judged is not
    the same run as one every target judged:

    >>> print(why_these_shards_are_not_one_run([at("a", 0, "--lib"), at("a", 1)]))
    Shard 1/2 ran with nothing after -- and shard 0/2 with `--lib`, so they are two suites and not one run.
    Every shard of a run passes the same arguments.

    A shard whose record names a different shard from the directory it sits
    in was moved or copied by hand, and nobody can say what it holds:

    >>> print(why_these_shards_are_not_one_run([at("a", 0), at("a", 0)]))
    The directory for shard 1/2 holds a record saying it is shard 0/2, so a shard was moved by hand.

    A shard that stopped before its last mutant is refused by name, because
    the merged count would read as a whole run that stopped short somewhere:

    >>> print(why_these_shards_are_not_one_run([at("a", 0), at("a", 1, processed=1)]))
    Shard 1/2 stopped after 1 of its 2 mutants, so the run is not complete.
    Run that shard again before reading this as a result.
    """


def one_run_from(shards: list[Shard]) -> Run:
    """Every shard's record summed into one run, so the refusals in
    `why_this_is_not_an_answer` and the summary are asked of the whole exactly
    as they are asked of a single run.

    >>> first = Shard(Conditions("a", "0/2", "", "in-place", "run"),
    ...     Run(declared=3, caught=["c1", "c2"], missed=["m1"]))
    >>> second = Shard(Conditions("a", "1/2", "", "in-place", "skip"),
    ...     Run(declared=2, would_not_compile=["u1"], never_started=[("n1", -1073741502)]))
    >>> whole = one_run_from([first, second])
    >>> whole.declared, whole.caught, whole.missed, whole.would_not_compile, whole.never_started
    (5, ['c1', 'c2'], ['m1'], ['u1'], [('n1', -1073741502)])

    So a set of shards with one mutant that never built is refused the same
    way a single run is:

    >>> print(why_this_is_not_an_answer(whole))
    1 of the 5 mutants never built, so this run is not an answer.
    The compiler exited with Windows status 0xC0000142 and printed nothing.
    That is Windows refusing to start it. Only 4 mutants were really tested.

    A baseline that failed in any shard fails the whole:

    >>> stopped = Shard(second.conditions, Run(declared=2, baseline_failed=True))
    >>> one_run_from([first, stopped]).baseline_failed
    True
    """


def a_shard_is_complete(run: Run) -> bool:
    """Whether every mutant the shard declared has an outcome, which is what
    `scripts/mutants.sh --shards` asks to skip a shard on a restart.

    >>> a_shard_is_complete(Run(declared=2, caught=["c"], would_not_compile=["u"]))
    True
    >>> a_shard_is_complete(Run(declared=2, caught=["c"]))
    False

    A mutant that never started has an outcome, so its shard is complete and
    the merge refuses it later. The launcher does not run the shard again for
    that: a never-started mutant is re-run by name with `-F`, not by shard.

    >>> a_shard_is_complete(Run(declared=1, never_started=[("n", -1073741502)]))
    True

    A shard whose baseline failed reached no mutant and is not complete:

    >>> a_shard_is_complete(Run(declared=2, baseline_failed=True))
    False
    """


# ---------------------------------------------------------------------------
# What a shard cost, in two terms.
#
# One term is paid once per shard: the copy of the tree, the build, and the
# baseline run before the first mutant, plus the tool's own bookkeeping
# between mutants. The other is paid once per mutant. They scale with
# different counts, so a product that folds them into one rate is a rate under
# conditions nobody wrote down.


@dataclass(frozen=True)
class Timing:
    """A shard's wall time split into the two terms."""

    wall_s: int
    mutants_s: int
    tried: int
    answered: int
    baseline_build_s: int | None
    baseline_test_s: int | None


def timing_of(outcomes: dict, started: int, finished: int) -> Timing:
    """Split a shard's wall time, from the two clock readings the script took
    around the run, into what the mutants took and everything else.

    The tool records a duration for every phase of every scenario. Summing the
    mutants' phases gives the term that scales with the mutant count; the rest
    of the wall time is the copy, the build, the baseline and the bookkeeping
    between mutants, which is the term that scales with the shard count.

    >>> outcomes = {"outcomes": [
    ...     {"scenario": "Baseline", "summary": "Success",
    ...      "phase_results": [{"phase": "Build", "duration": 281.4, "process_status": "Success"},
    ...                        {"phase": "Test", "duration": 52.2, "process_status": "Success"}]},
    ...     {"scenario": {"Mutant": {"name": "one"}}, "summary": "CaughtMutant",
    ...      "phase_results": [{"phase": "Build", "duration": 30.0, "process_status": "Success"},
    ...                        {"phase": "Test", "duration": 50.5, "process_status": {"Failure": 101}}]},
    ...     {"scenario": {"Mutant": {"name": "two"}}, "summary": "Unviable",
    ...      "phase_results": [{"phase": "Build", "duration": 12.5, "process_status": {"Failure": 101}}]},
    ... ]}
    >>> timing_of(outcomes, started=1000, finished=1500)
    Timing(wall_s=500, mutants_s=93, tried=2, answered=1, baseline_build_s=281, baseline_test_s=52)

    With `--baseline skip` there is no baseline scenario, and the record says
    so rather than reading zero seconds as a baseline that took no time:

    >>> timing_of({"outcomes": outcomes["outcomes"][1:]}, 1000, 1100).baseline_build_s is None
    True
    """


def the_timing_line(timing: Timing) -> str:
    """The one line a shard's cost is read from, printed after the run and
    written to `timing.txt` beside it, so a restarted launcher's overwritten
    stdout loses nothing and the shards' wall times can be summed from their
    own directories.

    >>> print(the_timing_line(Timing(wall_s=2833, mutants_s=2431, tried=25, answered=24,
    ...     baseline_build_s=281, baseline_test_s=52)))
    timed: 402 s before and between the mutants (the copy, the build and the baseline, 281 s build + 52 s test), then 2431 s over 25 mutants, 24 answered by the suite, 97 s a mutant

    A shard run with `--baseline skip` says so in the first term:

    >>> print(the_timing_line(Timing(wall_s=2500, mutants_s=2431, tried=25, answered=25,
    ...     baseline_build_s=None, baseline_test_s=None)))
    timed: 69 s before and between the mutants (the copy and the build, no baseline), then 2431 s over 25 mutants, 25 answered by the suite, 97 s a mutant

    A shard that reached no mutant has no rate to give, rather than a division
    by zero or a rate of zero that reads as fast:

    >>> print(the_timing_line(Timing(wall_s=300, mutants_s=0, tried=0, answered=0,
    ...     baseline_build_s=281, baseline_test_s=19)))
    timed: 300 s before and between the mutants (the copy, the build and the baseline, 281 s build + 19 s test), then no mutant was tried, so there is no rate
    """


def timeout_for_a_skipped_baseline(baseline_test_s: float, multiplier: float, minimum: int) -> int:
    """The `--timeout` a shard run with `--baseline skip` has to be given.

    With a baseline the tool sets each mutant's test timeout itself: the
    baseline's test time times `timeout_multiplier` from `.cargo/mutants.toml`,
    floored at `minimum_test_timeout`. Skip the baseline and it warns that it
    is using 300 seconds instead, whatever the suite takes. Under the
    every-target shape that is below what the config's rule gives, so a busy
    minute would be filed as a hang. The launcher reads the first complete
    shard's baseline and applies the config's own rule to it:

    >>> timeout_for_a_skipped_baseline(52.2, 5.0, 60)
    261
    >>> timeout_for_a_skipped_baseline(104.0, 5.0, 60)
    520
    >>> timeout_for_a_skipped_baseline(4.0, 5.0, 60)
    60
    """


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

    The third is a run the suite was started against and never finished
    against. It reached the tests every time and came back with nothing, and
    this used to read as a clean run because a timeout was counted as having
    got an answer:

    >>> print(why_this_is_not_an_answer(Run(declared=3, timed_out=["t"] * 3)))
    None of the 3 mutants got an answer from the suite: it was started every
    time and never once finished.
    A timeout is not a catch. Run them again before reading this as a result.

    Rejected and hung together is the fourth, and it names both counts,
    because which of the two a run went down decides what to do about it:

    >>> print(why_this_is_not_an_answer(Run(declared=5, timed_out=["t"] * 2,
    ...     would_not_compile=["u"] * 3)))
    None of the 5 mutants got an answer from the suite: the compiler rejected 3
    and the suite never finished against the other 2.
    A timeout is not a catch. Run them again before reading this as a result.

    One answer among timeouts is still an answer, the same as one among
    rejections:

    >>> why_this_is_not_an_answer(Run(declared=4, caught=["c"],
    ...     timed_out=["t"] * 3)) is None
    True

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
    # reached and the suite answered for none of them, which happens four
    # ways, and a check that can fail four ways has to say which.
    #
    # It is asked of what came back from the suite and not of what was handed
    # to it. A mutant the suite hung on was handed to it, so reading this off
    # `reached_the_tests` passed a run in which every mutant timed out and the
    # summary then called them all caught.
    if run.answered_by_the_suite == 0:
        if run.declared == 0:
            return (
                "There were no mutants, because nothing in these lines can be mutated.\n"
                "That is not the same as every mutant being caught, and it is not a result."
            )
        if not run.timed_out:
            return (
                f"None of the {run.declared} mutants reached the test suite: "
                "the compiler rejected every one.\n"
                "The suite never ran, so this run says nothing about what it would notice."
            )
        how_it_went = (
            "it was started every\ntime and never once finished"
            if not run.would_not_compile
            else (
                f"the compiler rejected {len(run.would_not_compile)}\n"
                f"and the suite never finished against the other {len(run.timed_out)}"
            )
        )
        return (
            f"None of the {run.declared} mutants got an answer from the suite: "
            f"{how_it_went}.\n"
            "A timeout is not a catch. Run them again before reading this as a result."
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


def how_much_of_it_asked_a_question(run: Run) -> str:
    """How much of the run reached an answer, and how much of it never asked.

    The bucket counts alone do not say this. The run of 2026-08-11 declared 84
    mutants and the suite answered for 30 of them, and every sentence written
    about it afterwards quoted the 28 it caught. Nobody reading that could tell
    it from a run of 84. So the proportion is said in words, once, here:

    >>> print(how_much_of_it_asked_a_question(Run(declared=84,
    ...     caught=["c"] * 28, missed=["m"] * 2, would_not_compile=["u"] * 54)))
    30 of the 84 mutants got an answer from the suite.
    54 the compiler rejected, 0 timed out: 64 percent of this run asked nothing.
    More of it was turned away than got through, so read it as a check on the
    30 that got through and not on the 84.

    A run that got all the way through says so in the same shape, and stops
    after two lines, because there is nothing to warn anybody about:

    >>> print(how_much_of_it_asked_a_question(Run(declared=3, caught=["c"] * 3)))
    3 of the 3 mutants got an answer from the suite.
    0 the compiler rejected, 0 timed out: 0 percent of this run asked nothing.

    It divides by how many mutants were declared. It is only ever reached from
    a run the suite answered for at least once, and a run cannot answer for a
    mutant it never declared, so that is at least 1 and there is no branch here
    for zero. A branch no test covers is worth less than a sentence saying why
    it is not needed.
    """
    turned_away = len(run.would_not_compile) + len(run.timed_out)
    lines = [
        f"{run.answered_by_the_suite} of the {how_many(run.declared, 'mutant')}"
        " got an answer from the suite.",
        f"{len(run.would_not_compile)} the compiler rejected, "
        f"{len(run.timed_out)} timed out: "
        f"{round(100 * turned_away / run.declared)} percent of this run asked nothing.",
    ]
    if turned_away > run.answered_by_the_suite:
        lines.append(
            "More of it was turned away than got through, so read it as a check on the\n"
            f"{run.answered_by_the_suite} that got through and not on the {run.declared}."
        )
    return "\n".join(lines)


def say_what_it_found(run: Run) -> None:
    """Print every bucket, so no run can be summed up by naming two of five.

    Only ever reached for a run the suite finished against at least once:
    everything else is refused before this is called, so there is nothing here
    about a run with no mutants in it. There was, and two places answering "was
    there anything to test" is how this project loses a day.

    That is a weaker claim than "a run that put a mutant to the suite", which
    is what it used to say, and the difference is the whole of what went wrong
    here. A run can put every mutant it has to the suite and finish against
    none of them.

    >>> say_what_it_found(Run(declared=3, caught=["one the suite caught"],
    ...     missed=["one nothing noticed"],
    ...     would_not_compile=["one the compiler rejected"]))
    Nothing was watching this:
        one nothing noticed
    That is behaviour no test would notice losing. Either pin it or delete it.
    <BLANKLINE>
    3 mutants: 1 caught, 1 nothing noticed, 1 the compiler rejected, 0 timed out.
    2 of the 3 mutants got an answer from the suite.
    1 the compiler rejected, 0 timed out: 33 percent of this run asked nothing.

    >>> say_what_it_found(Run(declared=1, caught=["one the suite caught"]))
    Every mutant that built was caught.
    <BLANKLINE>
    1 mutant: 1 caught, 0 nothing noticed, 0 the compiler rejected, 0 timed out.
    1 of the 1 mutant got an answer from the suite.
    0 the compiler rejected, 0 timed out: 0 percent of this run asked nothing.

    A run the suite finished against once and hung on twice is a run with two
    mutants nobody knows anything about. It used to be summed up as a clean
    one, because the headline was picked by asking only whether anything had
    been missed:

    >>> say_what_it_found(Run(declared=3, caught=["one the suite caught"],
    ...     timed_out=["one it hung on", "another it hung on"]))
    The suite never finished against 2 mutants:
        one it hung on
        another it hung on
    A timeout is not a catch and it is not a miss. Run those again.
    <BLANKLINE>
    3 mutants: 1 caught, 0 nothing noticed, 0 the compiler rejected, 2 timed out.
    1 of the 3 mutants got an answer from the suite.
    0 the compiler rejected, 2 timed out: 67 percent of this run asked nothing.
    More of it was turned away than got through, so read it as a check on the
    1 that got through and not on the 3.

    """
    if run.missed:
        print("Nothing was watching this:")
        for name in run.missed:
            print(f"    {name}")
        print("That is behaviour no test would notice losing. Either pin it or delete it.")
    if run.timed_out:
        print(f"The suite never finished against {how_many(len(run.timed_out), 'mutant')}:")
        for name in run.timed_out:
            print(f"    {name}")
        print("A timeout is not a catch and it is not a miss. Run those again.")
    if not run.missed and not run.timed_out:
        print("Every mutant that built was caught.")
    print()
    print(
        f"{how_many(run.declared, 'mutant')}: "
        f"{len(run.caught)} caught, "
        f"{len(run.missed)} nothing noticed, "
        f"{len(run.would_not_compile)} the compiler rejected, "
        f"{len(run.timed_out)} timed out."
    )
    print(how_much_of_it_asked_a_question(run))


def whether_this_run_passed(run: Run) -> int:
    """The exit code this run has earned, as a rule rather than as a line in main.

    A run the suite answered for every time, with nothing missed, passes:

    >>> whether_this_run_passed(Run(declared=1, caught=["c"]))
    0

    A mutant nothing noticed fails it:

    >>> whether_this_run_passed(Run(declared=2, caught=["c"], missed=["m"]))
    1

    And so does one the suite never finished against. There is no answer about
    that mutant either way, and a run that learned nothing about a mutant has
    not cleared it:

    >>> whether_this_run_passed(Run(declared=2, caught=["c"], timed_out=["t"]))
    1
    """
    return 1 if run.missed or run.timed_out else 0


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
    return whether_this_run_passed(run)


if __name__ == "__main__":
    sys.exit(main())

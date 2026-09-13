#!/usr/bin/env bash
# What `audit.sh` decides, given a run that already happened.
#
# The decision is the part worth testing and the part that can be tested without
# installing a vulnerable dependency on purpose. Every case below hands it a log
# written by hand and an exit status, which is exactly what the real run hands
# it.
#
# The two directions matter equally. A gate that refuses everything is no more
# use than one that refuses nothing, and the case this one exists to allow is
# narrow: an advisory that has been seen, written down with a reason, and not
# yet decided. Everything else still refuses.
#
# # Why both lists are handed in rather than read out of the subject
#
# Every case supplies the advisories it is about. The subject's own two lists
# are read only by the three cases that are about those lists.
#
# It used to be the other way round, and that was right while the held-open list
# had an entry in it: a case reading the live list cannot go on testing an entry
# somebody removed. What it could not survive is the list emptying, which is
# what deciding its one entry does. Two cases then contradicted each other, one
# wanting a clean run allowed and one wanting it refused, and both were reading
# the same empty list. A list supplied per case keeps the mechanism tested while
# the live list holds nothing, which is the state a gate for undecided
# advisories should spend most of its life in.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/audit.sh"

# shellcheck source=scripts/shell-suite.sh
. "$root/scripts/shell-suite.sh"

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

# A log in the shape cargo-audit really writes one, naming whichever advisories
# a case wants reported.
#
# The `Scanning` line is what tells a real run from a file that is readable and
# is not a run, so it is here rather than only in the cases that are about it: a
# fixture missing a line the subject looks for tests the subject's reading of
# that line by accident, in every case at once.
#
# `mktemp` rather than a counter, and that is a measurement rather than a
# preference. Every call to this is a command substitution, so it runs in a
# subshell, so a counter kept in a variable is incremented in a process that
# then exits. The first version of this used one and every fixture came back as
# the same path: two cases that wanted a wide log and a narrow one got the
# narrow one twice, and the case about a run reporting less than another was
# comparing a file with itself. Two cases caught it, both by refusing where they
# should have allowed, which is the direction a broken fixture is visible from.
a_run_reporting() {
    local file
    file="$(mktemp "$scratch/audit-XXXXXX.log")"
    {
        echo "    Fetching advisory database from \`https://github.com/RustSec/advisory-db.git\`"
        echo "      Loaded 1243 security advisories"
        echo "    Scanning Cargo.lock for vulnerabilities (698 crate dependencies)"
        local id
        for id in "$@"; do
            echo "Crate:     something"
            echo "Version:   0.1.0"
            echo "Title:     a title"
            echo "ID:        $id"
        done
    } > "$file"
    echo "$file"
}

# An accepted list in the shape `.cargo/audit.toml` really holds one, from lines
# handed in whole so a case can write a shape as well as a list.
an_accepted_list_of() {
    local file
    file="$(mktemp "$scratch/audit-toml-XXXXXX.toml")"
    printf '%s\n' "$@" > "$file"
    echo "$file"
}

# A case is judged on what the verdict answered, and a subject that could not run
# answered nothing.
#
# Written this way after watching the first version get it wrong. With audit.sh
# absent the shell answered 127, and every case wanting a refusal read 127 as
# one and passed: five of seven green against a gate that did not exist. That is
# the fourth guardrail exactly, and the same shape as a mutation run reporting
# every mutant caught when its build never started. A check that can fail two
# ways has to say which.
the_subject_did_not_run() {
    local got="$1"
    [ "$got" -eq 127 ] || [ ! -x "$subject" ]
}

expect_verdict() {
    local want="$1" desc="$2" log="$3" status="$4"
    shift 4
    local got
    "$subject" --verdict "$log" "$status" "$@" >/dev/null 2>&1
    got=$?
    if the_subject_did_not_run "$got"; then
        suite_case_failed "$desc" \
            "the subject did not run, so this case decided nothing" \
            "$subject"
    elif [ "$want" = allowed ] && [ "$got" -ne 0 ]; then
        suite_case_failed "$desc" "refused (exit $got) where it should have allowed"
    elif [ "$want" = refused ] && [ "$got" -eq 0 ]; then
        suite_case_failed "$desc" "allowed where it should have refused"
    else
        suite_case_passed "$desc"
    fi
}

# The same shape for the other decision: whether every advisory this project has
# accepted is still really reported by a run that applies no ignore list.
expect_still_apply() {
    local want="$1" desc="$2" unfiltered="$3" filtered="$4"
    shift 4
    local got
    "$subject" --acceptances-still-apply "$unfiltered" "$filtered" "$@" >/dev/null 2>&1
    got=$?
    if the_subject_did_not_run "$got"; then
        suite_case_failed "$desc" \
            "the subject did not run, so this case decided nothing" \
            "$subject"
    elif [ "$want" = allowed ] && [ "$got" -ne 0 ]; then
        suite_case_failed "$desc" "refused (exit $got) where it should have allowed"
    elif [ "$want" = refused ] && [ "$got" -eq 0 ]; then
        suite_case_failed "$desc" "allowed where it should have refused"
    else
        suite_case_passed "$desc"
    fi
}

# What the reader made of a file, compared against the whole answer rather than
# against whether one name is in it. A reader that returns everything it sees is
# wrong in the direction this suite exists to catch, and a containment check
# cannot see that.
expect_accepted() {
    local desc="$1" file="$2"
    shift 2
    local want="$*" got status
    got="$("$subject" --accepted "$file" 2>/dev/null | tr '\n' ' ')"
    status=$?
    got="${got%"${got##*[! ]}"}"
    if the_subject_did_not_run "$status"; then
        suite_case_failed "$desc" \
            "the subject did not run, so this case decided nothing" "$subject"
    elif [ "$got" != "$want" ]; then
        suite_case_failed "$desc" "read '$got' where it should have read '$want'"
    else
        suite_case_passed "$desc"
    fi
}

expect_accepted_refused() {
    local desc="$1" file="$2"
    local got
    "$subject" --accepted "$file" >/dev/null 2>&1
    got=$?
    if the_subject_did_not_run "$got"; then
        suite_case_failed "$desc" \
            "the subject did not run, so this case decided nothing" "$subject"
    elif [ "$got" -eq 0 ]; then
        suite_case_failed "$desc" "read the file where it should have refused it"
    else
        suite_case_passed "$desc"
    fi
}

# ── The two live lists ──────────────────────────────────────────────────────

mapfile -t awaiting < <("$subject" --awaiting-a-decision)
mapfile -t accepted < <("$subject" --accepted)

# The floor that used to sit on the held-open list, moved to the list that now
# carries the entries.
#
# Not a tidying. An accepted advisory that stops being reported is dead weight
# that then hides the next advisory against the same crate, so the check that
# each one is still reported is the load-bearing one, and a list that empties
# makes it iterate over nothing. That is the census-emptying failure this
# project has already been caught by twice. The held-open list is allowed to be
# empty, because holding nothing open is the state a decided project is in.
if [ "${#accepted[@]}" -eq 0 ]; then
    suite_case_failed "this project accepts at least one advisory" \
        ".cargo/audit.toml yielded no advisories, so the check that each one is" \
        "still reported has nothing to check. Either the reader is broken or the" \
        "whole acceptance mechanism should go rather than sit here proving nothing."
else
    suite_case_passed "this project accepts at least one advisory"
fi

# An advisory cannot be both accepted and undecided. If one reaches
# .cargo/audit.toml it has been decided, and holding it open here as well would
# leave a local exception nothing can ever clear.
both=()
for advisory in "${awaiting[@]+"${awaiting[@]}"}"; do
    for accepted_one in "${accepted[@]+"${accepted[@]}"}"; do
        [ "$advisory" = "$accepted_one" ] && both+=("$advisory")
    done
done
if [ "${#both[@]}" -gt 0 ]; then
    suite_case_failed "nothing is accepted and awaiting a decision at once" \
        "these are in .cargo/audit.toml and in awaiting_a_decision:" "${both[@]}"
else
    suite_case_passed "nothing is accepted and awaiting a decision at once"
fi

# ── Reading the accepted list ───────────────────────────────────────────────

file="$(an_accepted_list_of \
    '# A comment above the list.' \
    '[advisories]' \
    'ignore = [' \
    '    "RUSTSEC-2026-0194",' \
    '    "RUSTSEC-2023-0071",' \
    ']')"
expect_accepted "an advisory in the accepted list is read as accepted" \
    "$file" RUSTSEC-2026-0194 RUSTSEC-2023-0071

# The near-miss, and it is not hypothetical: it is the shape this file takes the
# moment the new check does its job. An entry that stops being reported has to
# come out, and whoever takes it out writes down which one went and why, inside
# the list, where the reader is looking. A reader matching an advisory id
# anywhere in the region would read that note as an acceptance and go on
# checking an entry nobody holds.
file="$(an_accepted_list_of \
    '[advisories]' \
    'ignore = [' \
    '    # RUSTSEC-2026-0098 was here until rustls-webpki moved off 0.101.7.' \
    '    # Taken out on 2026-09-13 because the run stopped reporting it.' \
    '    "RUSTSEC-2026-0194",' \
    ']')"
expect_accepted "an advisory named only in a comment is not read as accepted" \
    "$file" RUSTSEC-2026-0194

file="$(an_accepted_list_of '[advisories]')"
expect_accepted "a file with no ignore list is read as accepting nothing" "$file"

# A reader that cannot see the file must not answer "nothing is accepted", which
# is the one answer that makes the check pass while checking nothing. The same
# rule the verdict already applies to a run that did not happen.
expect_accepted_refused "an accepted list that cannot be read is refused rather than read as empty" \
    "$scratch/a-file-that-is-not-there.toml"

file="$(an_accepted_list_of \
    '[advisories]' \
    'ignore = [' \
    '    "RUSTSEC-2026-0194",')"
expect_accepted_refused "an accepted list that is never closed is refused rather than read short" \
    "$file"

# ── What the verdict allows ─────────────────────────────────────────────────

log="$(a_run_reporting RUSTSEC-2099-0003)"
expect_verdict allowed \
    "a run reporting only what the gate holds open is allowed" \
    "$log" 0 RUSTSEC-2099-0003

# Holding nothing open is a state, not a fault, and the run still has to judge
# what it was handed. Without this case the empty list would be untested in the
# exact configuration this project is now in.
log="$(a_run_reporting)"
expect_verdict allowed \
    "a gate holding nothing open still judges a clean run" \
    "$log" 0

# ── What it refuses ─────────────────────────────────────────────────────────

log="$(a_run_reporting RUSTSEC-2099-0003 RUSTSEC-2099-0001)"
expect_verdict refused \
    "an advisory nobody has written down is refused" \
    "$log" 1 RUSTSEC-2099-0003

# The staleness half. An entry held open that the run no longer reports is not
# good news to be waved through: it is a hole in the gate that looks like
# bookkeeping, and this project has already had a census empty itself and go on
# passing.
log="$(a_run_reporting RUSTSEC-2099-0002)"
expect_verdict refused \
    "an advisory held open that is no longer reported is refused" \
    "$log" 0 RUSTSEC-2099-0004

log="$(a_run_reporting)"
expect_verdict refused \
    "a clean run is refused while the gate still holds something open" \
    "$log" 0 RUSTSEC-2099-0004

# A run that did not happen says nothing about the dependencies, and the one
# answer it must never give is the clean one.
expect_verdict refused \
    "a missing log reads as no answer rather than as no advisories" \
    "$scratch/a-run-that-never-happened.log" 0 RUSTSEC-2099-0004

# ── Whether an acceptance still applies ─────────────────────────────────────
#
# The same staleness question as the held-open list, asked of the list that
# silences things in CI as well. It cannot be asked of the plain run, because
# that run is the one the acceptances are applied to and so it reports none of
# them. It is asked of a second run that applies no ignore list at all, and the
# two logs are handed in together so the cases can say what each one saw.

unfiltered="$(a_run_reporting RUSTSEC-2026-0194 RUSTSEC-2023-0071)"
filtered="$(a_run_reporting)"
expect_still_apply allowed \
    "an acceptance the unfiltered run still reports is allowed" \
    "$unfiltered" "$filtered" RUSTSEC-2026-0194 RUSTSEC-2023-0071

# The case the whole mechanism exists for. Four of this project's acceptances
# were in exactly this state on 2026-09-13 and nothing said so.
unfiltered="$(a_run_reporting RUSTSEC-2026-0194)"
filtered="$(a_run_reporting)"
expect_still_apply refused \
    "an acceptance the unfiltered run no longer reports is refused" \
    "$unfiltered" "$filtered" RUSTSEC-2026-0194 RUSTSEC-2023-0071

# The unfiltered run applies no ignore list, so it cannot report less than the
# run that does. When it does, it is not the run it is being read as, and every
# acceptance would then read as no longer reported. That is the reading failing
# in the direction that empties the list, which is the direction nothing else
# here can see.
unfiltered="$(a_run_reporting RUSTSEC-2026-0194)"
filtered="$(a_run_reporting RUSTSEC-2026-0194 RUSTSEC-2099-0005)"
expect_still_apply refused \
    "an unfiltered run narrower than the filtered one is refused" \
    "$unfiltered" "$filtered" RUSTSEC-2026-0194

filtered="$(a_run_reporting)"
expect_still_apply refused \
    "an unfiltered run that never happened is refused" \
    "$scratch/no-second-run.log" "$filtered" RUSTSEC-2026-0194

not_a_run="$(mktemp "$scratch/not-a-run-XXXXXX.log")"
echo "error: could not find Cargo.lock" > "$not_a_run"
expect_still_apply refused \
    "an unfiltered log that is not a run at all is refused" \
    "$not_a_run" "$filtered" RUSTSEC-2026-0194

suite_verdict

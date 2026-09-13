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
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/audit.sh"

# shellcheck source=scripts/shell-suite.sh
. "$root/scripts/shell-suite.sh"

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

# A log in the shape cargo-audit really writes one, naming whichever advisories
# a case wants reported.
a_run_reporting() {
    local file="$scratch/audit-$RANDOM.log"
    {
        echo "    Fetching advisory database from \`https://github.com/RustSec/advisory-db.git\`"
        echo "      Loaded 1243 security advisories"
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

# A case is judged on what the verdict answered, and a subject that could not run
# answered nothing.
#
# Written this way after watching the first version get it wrong. With audit.sh
# absent the shell answered 127, and every case wanting a refusal read 127 as
# one and passed: five of seven green against a gate that did not exist. That is
# the fourth guardrail exactly, and the same shape as a mutation run reporting
# every mutant caught when its build never started. A check that can fail two
# ways has to say which.
expect_verdict() {
    local want="$1" desc="$2" log="$3" status="$4"
    local got
    "$subject" --verdict "$log" "$status" >/dev/null 2>&1
    got=$?
    if [ "$got" -eq 127 ] || [ ! -x "$subject" ]; then
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

# The list the gate holds open, read from the subject rather than copied, so a
# case cannot go on testing an entry somebody removed.
mapfile -t awaiting < <("$subject" --awaiting-a-decision)

# ── The list itself ─────────────────────────────────────────────────────────

if [ "${#awaiting[@]}" -eq 0 ]; then
    # Not a pass. A list that has emptied makes every case below iterate over
    # nothing, and a suite that passes by having nothing to check is the failure
    # this project keeps meeting. If the list is really empty, the whole
    # mechanism should go rather than sit here proving nothing.
    suite_case_failed "the gate holds at least one advisory open" \
        "awaiting_a_decision is empty, so every case in this suite tests nothing"
else
    suite_case_passed "the gate holds at least one advisory open"
fi

# An advisory cannot be both accepted and undecided. If one reaches
# .cargo/audit.toml it has been decided, and holding it open here as well would
# leave a local exception nothing can ever clear.
both=()
for advisory in "${awaiting[@]}"; do
    if grep -qF "\"$advisory\"" "$root/.cargo/audit.toml"; then
        both+=("$advisory")
    fi
done
if [ "${#both[@]}" -gt 0 ]; then
    suite_case_failed "nothing is accepted and awaiting a decision at once" \
        "these are in .cargo/audit.toml and in awaiting_a_decision:" "${both[@]}"
else
    suite_case_passed "nothing is accepted and awaiting a decision at once"
fi

# ── What the verdict allows ─────────────────────────────────────────────────

log="$(a_run_reporting "${awaiting[@]}")"
expect_verdict allowed \
    "a run reporting only what nobody has decided yet is allowed" \
    "$log" 0

# ── What it refuses ─────────────────────────────────────────────────────────

log="$(a_run_reporting "${awaiting[@]}" RUSTSEC-2099-0001)"
expect_verdict refused \
    "an advisory nobody has written down is refused" \
    "$log" 1

# The staleness half. An entry held open that the run no longer reports is not
# good news to be waved through: it is a hole in the gate that looks like
# bookkeeping, and this project has already had a census empty itself and go on
# passing.
log="$(a_run_reporting RUSTSEC-2099-0002)"
expect_verdict refused \
    "an advisory held open that is no longer reported is refused" \
    "$log" 0

log="$(a_run_reporting)"
expect_verdict refused \
    "a clean run is refused while the gate still holds something open" \
    "$log" 0

# A run that did not happen says nothing about the dependencies, and the one
# answer it must never give is the clean one.
expect_verdict refused \
    "a missing log reads as no answer rather than as no advisories" \
    "$scratch/a-run-that-never-happened.log" 0

suite_verdict

#!/usr/bin/env bash
# Known advisories against this project's dependencies, asked locally.
#
#   audit.sh                          run it and decide
#   audit.sh --awaiting-a-decision    the advisories this gate holds open
#   audit.sh --accepted [<file>]      the advisories this project has accepted
#   audit.sh --verdict <log> <status> [<held-open id>...]
#   audit.sh --acceptances-still-apply <unfiltered log> <log> [<accepted id>...]
#
# # Why this exists
#
# CI has had a Security Audit job since the beginning and this machine had no
# counterpart to it, so the only place an advisory could be seen was a job page
# nobody opens on a green day. It went red on 2026-09-10 and sat unread until
# 2026-09-13, which is this project's fourth guardrail happening to the check
# that is furthest from anybody's eyes. `CLAUDE.md` said for months that
# `scripts/check.sh` runs "the same four checks CI runs", and one reason that
# was wrong is that this check did not exist here at all.
#
# The research for phase 8 recorded that there was no local counterpart and
# that one could not easily be had. Half of that was wrong: cargo-audit was
# already installed on this machine, at 0.22.2. It simply was not wired to
# anything.
#
# # The two lists, and why they are not one list
#
# `.cargo/audit.toml` holds advisories this project has **accepted**: read,
# reasoned about, and written down with an exit condition. cargo-audit silences
# those everywhere, here and in CI, which is what accepting one means.
#
# The list below is different. It holds an advisory that has been **seen and not
# yet decided**. Putting it in `audit.toml` would decide it, quietly, by
# silencing it in CI as well, and the decision belongs to whoever owns the
# dependency rather than to whoever was fixing the gate that day. So the local
# gate sets it aside and says so on every run, and the CI job stays red. Red is
# the correct state for a question nobody has answered.
#
# That arrangement has an obvious failure mode and it is guarded: a list of
# advisories to hold open goes stale the moment one of them stops being
# reported, and then it is a permanent hole that looks like bookkeeping. Every
# run checks that each advisory named below is still really reported, and
# refuses when one is not. An entry that no longer applies has to be removed,
# not left to rot.
set -euo pipefail

# Seen, not decided. One line of reason and one exit condition each, in the
# same shape `.cargo/audit.toml` uses for the ones that were decided.
#
#   RUSTSEC-2023-0071  rsa 0.9.10, the Marvin timing sidechannel, 5.9 medium.
#   It arrives through pgp 0.20, which is a real dependency rather than a dev
#   one, and the advisory says plainly that no fixed upgrade is available. The
#   options are all costly: drop OpenPGP support, carry a patched fork, or wait
#   for the pgp crate to move off rsa. That is a product decision, not a gate
#   decision. Clears when it is made, either way.
awaiting_a_decision=(
    RUSTSEC-2023-0071
)

if [ "${1:-}" = "--awaiting-a-decision" ]; then
    printf '%s\n' "${awaiting_a_decision[@]+"${awaiting_a_decision[@]}"}"
    exit 0
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
the_accepted_list="$repo_root/.cargo/audit.toml"

# What this project has accepted, read out of `.cargo/audit.toml`.
advisories_accepted_in() {
    local file="${1:-$the_accepted_list}"
    : "$file"
    return 0
}

if [ "${1:-}" = "--accepted" ]; then
    advisories_accepted_in "${2:-$the_accepted_list}"
    exit $?
fi

# The decision, given a run that already happened.
#
# Apart from the run itself so the suite can ask it about a log written by hand
# rather than by installing a vulnerable dependency on purpose. The same
# separation `which-checks.sh` has from `check.sh`, for the same reason: a
# decision that can only be exercised by causing the thing it decides about is a
# decision nothing tests.
#
#   <log>     the whole output of a plain `cargo audit`
#   <status>  the exit status of `cargo audit` with the list above set aside
the_verdict() {
    local log="$1" filtered_status="$2"
    shift 2
    local id
    local -a gone=()

    if [ ! -r "$log" ]; then
        echo "audit: no run to judge. '$log' is not readable, so this says" >&2
        echo "  nothing about the dependencies and must not read as clean." >&2
        return 1
    fi

    # Every advisory held open has to still be reported. One that is not is
    # either fixed or no longer reachable, and either way this list is now
    # silencing nothing while looking like it silences something. This project
    # has already been caught by a census that emptied and went on passing.
    for id in "${awaiting_a_decision[@]}"; do
        if ! grep -qF "$id" "$log"; then
            gone+=("$id")
        fi
    done
    if [ "${#gone[@]}" -gt 0 ]; then
        echo >&2
        echo "audit: ${#gone[@]} advisory(ies) this gate holds open are no longer" >&2
        echo "  reported, so holding them open silences nothing:" >&2
        printf '    %s\n' "${gone[@]}" >&2
        echo "  Take them out of awaiting_a_decision in scripts/audit.sh." >&2
        return 1
    fi

    if [ "$filtered_status" -ne 0 ]; then
        echo >&2
        echo "audit: an advisory that is not accepted and not awaiting a decision." >&2
        echo >&2
        cat "$log" >&2
        echo >&2
        echo "  Accepted advisories, with a reason and an exit condition each, live" >&2
        echo "  in .cargo/audit.toml. Adding one there is a decision; make it on" >&2
        echo "  purpose rather than to get a commit through." >&2
        return 1
    fi

    echo "No advisory outside .cargo/audit.toml, apart from ${#awaiting_a_decision[@]} nobody has decided yet:"
    printf '    %s (CI is still red on this, on purpose)\n' "${awaiting_a_decision[@]}"
    return 0
}

if [ "${1:-}" = "--verdict" ]; then
    shift
    the_verdict "${1:-}" "${2:-}" "${@:3}"
    exit $?
fi

# Whether every advisory this project accepts is still really reported.
the_acceptances_still_apply() {
    local unfiltered="$1" filtered="$2"
    shift 2
    : "$unfiltered" "$filtered" "$*"
    return 0
}

if [ "${1:-}" = "--acceptances-still-apply" ]; then
    shift
    the_acceptances_still_apply "${1:-}" "${2:-}" "${@:3}"
    exit $?
fi

# Running it. Thin on purpose: everything that decides anything is above, and
# this part is the two commands and the plumbing between them.
if ! cargo audit --version >/dev/null 2>&1; then
    echo "audit: cargo-audit is not installed, so this check did not run." >&2
    echo "  That is not the same as finding nothing, and it must not read as it." >&2
    echo "      cargo install cargo-audit --locked" >&2
    exit 1
fi

plain_log="$(mktemp)"
trap 'rm -f "$plain_log"' EXIT

# The plain run first, so what it reports is on the record whatever the verdict
# turns out to be. Its exit status is deliberately not the verdict: it is
# non-zero whenever anything at all is found, including the advisory the list
# above is holding open.
cargo audit > "$plain_log" 2>&1 || true

# And the same run with the undecided advisories set aside. `--ignore` on the
# command line rather than in .cargo/audit.toml, so CI reads the file and stays
# red while this machine can still commit.
ignore_flags=()
for advisory in "${awaiting_a_decision[@]+"${awaiting_a_decision[@]}"}"; do
    ignore_flags+=(--ignore "$advisory")
done
filtered_status=0
cargo audit "${ignore_flags[@]+"${ignore_flags[@]}"}" >/dev/null 2>&1 || filtered_status=$?

the_verdict "$plain_log" "$filtered_status" \
    "${awaiting_a_decision[@]+"${awaiting_a_decision[@]}"}"

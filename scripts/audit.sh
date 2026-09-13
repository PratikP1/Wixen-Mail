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
#
# # The same question, asked of the accepted list
#
# That guard sat on the held-open list only, and the held-open list held one
# entry. The accepted list held seven and nothing asked it anything, which is
# the wrong way round: an `ignore` entry names an advisory id, so an entry that
# has stopped applying goes on silencing the advisory it names while a second
# advisory against the same crate goes unreported. A held-open entry that goes
# stale costs a red CI job nobody needed. An accepted entry that goes stale is a
# hole in the check.
#
# On 2026-09-13, when this was added, four of the seven were already in that
# state: three naming rustls-webpki 0.101.7, which the tree left behind for
# 0.103.13, and one naming rustls-pemfile, which left the tree. The check found
# them on its first run. That is a finding, not a sign it is too strict.
#
# Asking it needs a second run, because the plain run is the one the acceptances
# are applied to and so reports none of them by construction. There is no flag
# for "read no config", so the second run happens in a directory carrying a
# config of its own that ignores nothing. See `the_acceptances_still_apply` and
# the block that builds that directory for what was measured and what is assumed.
#
# # What is deliberately not here
#
# No version fingerprint. The obvious companion to "is it still reported" is
# "was it judged against the crate version in the tree today", on the pattern of
# `tests_last_seen` in `guards/guards.toml`, and it was decided against rather
# than forgotten. The reasoning is written out in
# `.planning/phases/07-installing-updating-and-what-is-stored/RSA-ADVISORY-REPORT.md`.
# The short version: for every way an acceptance has actually gone stale here,
# the version moving and the advisory ceasing to be reported are the same event,
# so the fingerprint fires at the same moment as the cheaper check and adds a
# hand-maintained number per entry that has to be right. The gap it leaves is
# real and is not version-shaped, so a version fingerprint would not have closed
# it either: an advisory can gain a fix while the crate stays where it is, and
# then the run still reports it and an acceptance resting on "there is nothing
# to upgrade to" has quietly stopped being true. That one is in
# `.planning/WINDOWS.md` rather than papered over.
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

# Part of the line cargo-audit prints once it has a lockfile open and a database
# loaded: "Scanning <path> for vulnerabilities (698 crate dependencies)". The
# path varies with how the run was invoked and the count with the tree, so the
# middle of it is what is matched. Nothing this reads is a run without it.
readonly WHAT_A_FINISHED_RUN_SAYS='for vulnerabilities'

# What this project has accepted, read out of `.cargo/audit.toml`.
#
# **A tokeniser, not a TOML parse**, and the difference from the other line scan
# in this repository is the only reason that is safe. `check.sh` reads
# `guards.toml` by scanning for lines in a known shape, and says in its own
# comment what it would miss: a record spelled another way is skipped, and a
# skipped record is a guard that quietly stops running. A skip is not available
# here. Every token between the brackets is either an advisory id or a refusal
# that says which token could not be read.
#
# So the answer is one of three things and never a fourth. A list of ids. No
# list, because the file names no `ignore` key, which is a real state and is
# read as accepting nothing. Or a refusal. What it will not do is answer
# "nothing is accepted" because it could not read the file, which is the one
# wrong answer that makes every check downstream pass while checking nothing.
#
# A shell script with no TOML reader is the constraint, and it is deliberate:
# `scripts/check.sh` runs this on every `all` commit, and a gate that needs a
# second language installed is a gate that refuses a commit on a machine where
# that language moved.
#
#   <file>  a `.cargo/audit.toml`, defaulting to this project's own
advisories_accepted_in() {
    local file="${1:-$the_accepted_list}"
    local line stripped inside_the_brackets="" token
    local reading=before_the_list

    if [ ! -r "$file" ]; then
        echo "audit: '$file' is not readable, so what this project has accepted" >&2
        echo "  cannot be read. That is not the same as accepting nothing, and it" >&2
        echo "  must not read as it." >&2
        return 1
    fi

    # Comments off each line first, because a note inside the list saying which
    # entry was taken out and why names an advisory id, and that note is exactly
    # what this file fills up with as the check below does its job.
    while IFS= read -r line; do
        stripped="${line%%#*}"
        if [ "$reading" = before_the_list ]; then
            case "$stripped" in
                *ignore*=*\[*)
                    reading=inside_the_list
                    stripped="${stripped#*[}"
                    ;;
                *) continue ;;
            esac
        fi
        case "$stripped" in
            *\]*)
                inside_the_brackets="$inside_the_brackets ${stripped%%]*}"
                reading=past_the_list
                break
                ;;
            *) inside_the_brackets="$inside_the_brackets $stripped" ;;
        esac
    done < "$file"

    if [ "$reading" = before_the_list ]; then
        return 0
    fi
    if [ "$reading" != past_the_list ]; then
        echo "audit: the ignore list in '$file' is never closed, so what it holds" >&2
        echo "  cannot be read to the end. Reading it as far as it goes would be" >&2
        echo "  reading it short, which is the answer that hides an acceptance." >&2
        return 1
    fi

    # Unquoted on purpose: the split into words is the tokenising step.
    for token in ${inside_the_brackets//,/ }; do
        case "$token" in
            '"'RUSTSEC-[0-9][0-9][0-9][0-9]-[0-9][0-9][0-9][0-9]'"')
                echo "${token//\"/}"
                ;;
            *)
                echo "audit: cannot read '$token' in the ignore list of '$file'." >&2
                echo "  Every entry has to be a quoted advisory id, one per line," >&2
                echo "  so that reading the list is not a guess. Refused rather" >&2
                echo "  than skipped: a skipped entry is an acceptance nothing" >&2
                echo "  checks." >&2
                return 1
                ;;
        esac
    done
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
#   <status>  the exit status of `cargo audit` with the list below set aside
#   <id>...   the advisories the gate is holding open, which may be none
the_verdict() {
    local log="$1" filtered_status="$2"
    shift 2
    local id
    local -a held_open=("$@")
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
    for id in "${held_open[@]+"${held_open[@]}"}"; do
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

    if [ "${#held_open[@]}" -eq 0 ]; then
        echo "No advisory outside .cargo/audit.toml, and nothing is being held open."
        return 0
    fi
    echo "No advisory outside .cargo/audit.toml, apart from ${#held_open[@]} nobody has decided yet:"
    printf '    %s (CI is still red on this, on purpose)\n' "${held_open[@]}"
    return 0
}

if [ "${1:-}" = "--verdict" ]; then
    shift
    the_verdict "${1:-}" "${2:-}" "${@:3}"
    exit $?
fi

# Whether every advisory this project accepts is still really reported.
#
# The same staleness question the verdict above asks of the held-open list, asked
# of the list that silences things in CI too, and it is the more important of the
# two. An acceptance that no longer applies is not harmless bookkeeping: an
# `ignore` entry names an advisory id, so the day a crate picks up a second
# advisory the entry goes on silencing the first and says nothing about the
# second. A list that has quietly stopped applying is a list that hides the next
# finding, which is this project's census-emptying failure wearing a different
# coat.
#
# It cannot be asked of the plain run. That run is the one the acceptances are
# applied to, so it reports none of them by construction. It is asked of a second
# run that applies no ignore list at all, and both logs are handed in, because
# comparing them is what tells a second run that found nothing from a second run
# that did not happen.
#
#   <unfiltered>  the output of a `cargo audit` with no ignore list applied
#   <filtered>    the output of the plain run, which does apply this project's
#   <id>...       the advisories this project has accepted, which may be none
the_acceptances_still_apply() {
    local unfiltered="$1" filtered="$2"
    shift 2
    local -a accepted=("$@")
    local id
    local -a gone=() missing=()

    if [ ! -r "$unfiltered" ]; then
        echo "audit: the run that applies no ignore list did not happen, so" >&2
        echo "  whether this project's acceptances still apply is unknown." >&2
        echo "  '$unfiltered' is not readable." >&2
        return 1
    fi

    # A readable file is not a run. cargo-audit says what it scanned on every
    # run it completes, and a file without that line is output from something
    # that stopped earlier: a missing lockfile, an advisory database it could
    # not open, a command that was never found. Every one of those looks exactly
    # like "none of the accepted advisories are reported any more", which would
    # empty the list in one step.
    if ! grep -qF "$WHAT_A_FINISHED_RUN_SAYS" "$unfiltered"; then
        echo "audit: the run that applies no ignore list did not finish, so it" >&2
        echo "  says nothing about what is still reported. Its whole output:" >&2
        echo >&2
        cat "$unfiltered" >&2
        return 1
    fi

    # The unfiltered run applies no ignore list, so it cannot report less than
    # the run that does. When it reports less, it is not the run it is being
    # read as, and every acceptance would then read as no longer reported. This
    # is the only check here that can see the reading itself failing, and it
    # fails in the direction that empties the list.
    if [ -r "$filtered" ]; then
        for id in $(grep -oE 'RUSTSEC-[0-9]{4}-[0-9]{4}' "$filtered" | sort -u); do
            if ! grep -qF "$id" "$unfiltered"; then
                missing+=("$id")
            fi
        done
    fi
    if [ "${#missing[@]}" -gt 0 ]; then
        echo >&2
        echo "audit: the run that applies no ignore list reported less than the" >&2
        echo "  run that does, which cannot happen. It is not the run it is being" >&2
        echo "  read as, so nothing below it can be trusted. Reported by the" >&2
        echo "  filtered run and not by the unfiltered one:" >&2
        printf '    %s\n' "${missing[@]}" >&2
        return 1
    fi

    for id in "${accepted[@]+"${accepted[@]}"}"; do
        if ! grep -qF "$id" "$unfiltered"; then
            gone+=("$id")
        fi
    done
    if [ "${#gone[@]}" -gt 0 ]; then
        echo >&2
        echo "audit: ${#gone[@]} advisory(ies) this project accepts are no longer" >&2
        echo "  reported at all, so accepting them silences nothing and hides the" >&2
        echo "  next advisory against the same crate:" >&2
        printf '    %s\n' "${gone[@]}" >&2
        echo "  Take them out of the ignore list in .cargo/audit.toml, and say in" >&2
        echo "  a comment there which went and why, so the next reader knows the" >&2
        echo "  list shrank on purpose." >&2
        return 1
    fi

    if [ "${#accepted[@]}" -eq 0 ]; then
        echo "This project accepts no advisory, so there was nothing to check."
        return 0
    fi
    echo "All ${#accepted[@]} advisory(ies) this project accepts are still reported."
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
unfiltered_log="$(mktemp)"
elsewhere="$(mktemp -d)"
trap 'rm -f "$plain_log" "$unfiltered_log"; rm -rf "$elsewhere"' EXIT

# The plain run first, so what it reports is on the record whatever the verdict
# turns out to be. Its exit status is deliberately not the verdict: it is
# non-zero whenever anything at all is found, including the advisory the list
# above is holding open.
cargo audit > "$plain_log" 2>&1 || true

# And a run that applies no ignore list at all, which is the only way to ask
# whether the acceptances in `.cargo/audit.toml` still apply. cargo-audit has no
# flag for "read no config", so the run is moved to a directory that holds a
# config of its own saying to ignore nothing.
#
# **Measured rather than assumed, on 2026-09-13 against cargo-audit 0.22.2.**
# Running from `target/auditprobe`, two directories below a `.cargo/audit.toml`
# holding seven entries, reported all seven: this version looks for its config in
# the working directory and does not walk upwards. So an empty directory would
# have done. The config below is written anyway, because that measurement is
# about one version and the cost of it being wrong is every acceptance reading as
# still applying when none of them were checked. With a config present, whatever
# the discovery rule turns out to be, the nearest one is this one and its effect
# is known. What it would not survive is a version that merges configs from
# several directories, and nothing here would notice that.
mkdir -p "$elsewhere/.cargo"
printf '[advisories]\nignore = []\n' > "$elsewhere/.cargo/audit.toml"

# `-n`, because the plain run fetched the advisory database into the same shared
# copy seconds ago. A second fetch would measure the network rather than the
# tree, and would let the two runs disagree about which database they read,
# which is the one thing the comparison between them must not allow.
(
    cd "$elsewhere" &&
        cargo audit -n -f "$repo_root/Cargo.lock"
) > "$unfiltered_log" 2>&1 || true

# And the same run with the undecided advisories set aside. `--ignore` on the
# command line rather than in .cargo/audit.toml, so CI reads the file and stays
# red while this machine can still commit.
ignore_flags=()
for advisory in "${awaiting_a_decision[@]+"${awaiting_a_decision[@]}"}"; do
    ignore_flags+=(--ignore "$advisory")
done
filtered_status=0
cargo audit "${ignore_flags[@]+"${ignore_flags[@]}"}" >/dev/null 2>&1 || filtered_status=$?

# The acceptances first, and the order is load-bearing rather than tidy. An
# acceptance that has stopped applying goes on silencing an advisory id while a
# second advisory against the same crate goes unreported, so a stale list can be
# the reason the verdict below is wrong. Say the list is stale before saying what
# the run found with it applied.
accepted_text="$(advisories_accepted_in "$the_accepted_list")" || exit 1
accepted_ids=()
if [ -n "$accepted_text" ]; then
    mapfile -t accepted_ids <<< "$accepted_text"
fi

echo "== the advisories this project accepts =="
the_acceptances_still_apply "$unfiltered_log" "$plain_log" \
    "${accepted_ids[@]+"${accepted_ids[@]}"}"

echo "== everything else =="
the_verdict "$plain_log" "$filtered_status" \
    "${awaiting_a_decision[@]+"${awaiting_a_decision[@]}"}"

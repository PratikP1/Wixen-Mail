#!/usr/bin/env bash
# What `which-checks.sh` answers, for every branch it can be asked about.
#
# The decision lives in its own script rather than inside the hook so that it
# can be asked a question without running four minutes of checks to find out.
# A hook that decides something is a hook that can decide it wrongly, and this
# is the only place that would show.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/which-checks.sh"

failures=0

expect() {
    local branch="$1" want="$2" got
    got="$("$subject" "$branch" 2>/dev/null)"
    if [ "$got" != "$want" ]; then
        echo "FAIL: branch '$branch' answered '$got', wanted '$want'"
        failures=$((failures + 1))
    fi
}

# The default branch earns everything. Every commit here lands on it, so the
# four checks are what stands between a broken commit and the branch CI builds.
expect "main" all
expect "master" all

# A work-in-progress branch nobody builds earns the quick pair. The hook's own
# comment already sanctioned this case in words; this is the same rule, made
# something the machine applies rather than something a person remembers.
expect "gsd/plan-01-02" all_but_slow
expect "gsd/phase-1-folders-and-conversations" all_but_slow
expect "some-experiment" all_but_slow

# A name that could be read two ways is not a licence. `main` is matched
# exactly, so a branch merely starting with it is still a branch nobody builds.
expect "maintenance" all_but_slow
expect "mainline" all_but_slow

# The failure this is most likely to meet: a detached HEAD, a rebase in
# progress, or a git call that returned nothing. A check that cannot tell where
# it is must not answer "safe". It answers with the whole suite.
expect "" all
expect "HEAD" all

if [ "$failures" -eq 0 ]; then
    echo "which-checks: all cases pass"
else
    echo "which-checks: $failures case(s) failed"
    exit 1
fi

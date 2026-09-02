#!/usr/bin/env bash
# Which integration targets a commit earns, on top of the unit tests beside the
# code it changed.
#
# `scripts/check.sh` maps a changed `src/a/b.rs` to `cargo test --lib a::b::`,
# so a guard that lives under `tests/` and covers a `src/` module is reached
# only when the test file itself changes. That is the shape guardrail 4 in
# `CLAUDE.md` is about: the guard reads as covered and runs on the wrong
# commits.
#
# `guards/guards.toml` already declares the coupling. Every record carries a
# `file`, the source the break is applied to, and some carry a `suite`, the
# target whose tests go red. So the gate reads the registry rather than keeping
# a second list beside it, because two places describing one thing is how this
# repository keeps getting caught.
#
# A router that runs everything always is as wrong as one that runs nothing, so
# the cases below come in both halves. What must be answered: a real coupling to
# a target the gate would otherwise skip. What must not: a file no record names,
# a target the gate already runs on every scoped run, a record whose break lands
# somewhere `--lib` was never going to reach, and anything read out of a
# record's prose rather than its `file` and `suite`.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/check.sh"
registry="$root/guards/guards.toml"

failures=0

# The mapping's answer as one line, so a case reads as a sentence.
#
# Sorted before comparing, because order is not part of the contract: a record
# added to the registry above another must not redden a case here.
answer() {
    bash "$subject" --suites-for "$@" 2>/dev/null | sort | tr '\n' ' ' | sed 's/ *$//'
}

expect() {
    local want="$1" desc="$2"
    shift 2
    local got
    got="$(answer "$@")"
    if [ "$got" != "$want" ]; then
        echo "FAIL [$desc]: answered '$got', wanted '$want'"
        echo "       args: $*"
        failures=$((failures + 1))
    fi
}

# Against the real registry the cases below ask whether a suite is in the
# answer, not whether it is the whole answer.
#
# Equality was the first spelling and it was wrong within the hour: this file
# was written against the registry as it stood, and the very next task added a
# second record for `wx_managers.rs`, so four cases went red for having done
# their job. An assertion that pins the exact contents of shared, growing data
# fails on every addition to it, and a suite that goes red when somebody adds a
# guard record teaches people not to add guard records.
#
# The exclusions stay exact, because those really are claims about the whole
# answer: a file no record names, or one coupled only to a target the run
# already ends with, must answer nothing at all.
expect_among() {
    local want="$1" desc="$2"
    shift 2
    local got
    got="$(answer "$@")"
    case " $got " in
        *" $want "*) ;;
        *)
            echo "FAIL [$desc]: answered '$got', which does not include '$want'"
            echo "       args: $*"
            failures=$((failures + 1))
            ;;
    esac
}

expect_not_among() {
    local unwanted="$1" desc="$2"
    shift 2
    local got
    got="$(answer "$@")"
    case " $got " in
        *" $unwanted "*)
            echo "FAIL [$desc]: answered '$got', which still includes '$unwanted'"
            echo "       args: $*"
            failures=$((failures + 1))
            ;;
    esac
}

fail() {
    echo "FAIL [$1]: $2"
    failures=$((failures + 1))
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# ── The registry as it really is ────────────────────────────────────────────
# One record couples a source module to a target the gate would otherwise reach
# only when the test file itself changed. That record is the whole reason this
# mapping exists, so it is asserted against the real file rather than a fixture.
expect_among manager_dialog_labels "the manager window's dialog-label guard is coupled to the window" \
    "$registry" src/presentation/wx_managers.rs
expect_among manager_delete_stays_open "and so is its delete guard" \
    "$registry" src/presentation/wx_managers.rs
expect_among checkbox_labels "the item form's check-box guard is coupled to the form" \
    "$registry" src/presentation/wx_item_form.rs

# A source file no record names is answered with nothing, and the run is what it
# was before. Most commits are this case and it must not get slower or stranger.
expect "" "a source file no record names answers nothing" \
    "$registry" src/application/threading.rs

# `house_style` and `wired` are already run on every scoped run, because they
# read across the whole tree. A record coupling a source file to one of them is
# a true statement that buys this mapping nothing, and answering it would run
# the same target twice.
expect "" "a coupling to wired, which every scoped run already ends with" \
    "$registry" src/presentation/wx_app.rs
expect "" "a coupling to house_style, likewise" \
    "$registry" src/application/draft_copy.rs

# Several changed files at once, only one of them coupled.
expect_among manager_dialog_labels "a coupled file beside an uncoupled one" \
    "$registry" src/presentation/wx_managers.rs src/application/threading.rs

# ── The coupling really is what is being read ───────────────────────────────
# Take the `suite` line off that one record, in a copy, and the answer goes
# away. Demonstrated against a copy because editing the real registry to prove
# a point is how a proof becomes a defect.
#
# The copy is checked to differ by exactly one line first. Without that, a
# botched copy step, an empty file or a failed `grep`, makes the case below pass
# for the wrong reason, and an empty answer would read as a finding when it is
# really a broken fixture.
uncoupled="$work/registry-without-the-suite.toml"
grep -v '^suite = "manager_dialog_labels"$' "$registry" > "$uncoupled" || true
removed=$(( $(wc -l < "$registry") - $(wc -l < "$uncoupled") ))
if [ "$removed" -ne 1 ]; then
    fail "the copy of the registry differs by exactly one line" \
        "removed $removed lines, wanted 1; the case below would pass for the wrong reason"
fi
expect_not_among manager_dialog_labels "taking the suite line off that record leaves the file uncoupled" \
    "$uncoupled" src/presentation/wx_managers.rs
# And the real one still answers, so the case above is about the edit rather
# than about the mapping having stopped working between the two.
expect_among manager_dialog_labels "the real registry still answers after the copy was made" \
    "$registry" src/presentation/wx_managers.rs

# ── Made-up registries, for the shapes the real one does not hold today ─────

two_suites="$work/two-suites.toml"
cat > "$two_suites" <<'TOML'
[[guard]]
name = "the first thing"
file = "src/presentation/wx_item_form.rs"
suite = "alpha"
before = """a"""
after = """b"""
red = [
    "test_one",
]

[[guard]]
name = "the second thing"
file = "src/presentation/wx_item_form.rs"
suite = "beta"
before = """c"""
after = """d"""
red = [
    "test_two",
]
TOML
expect "alpha beta" "a file two records couple to two targets answers both" \
    "$two_suites" src/presentation/wx_item_form.rs

one_suite_twice="$work/one-suite-twice.toml"
cat > "$one_suite_twice" <<'TOML'
[[guard]]
name = "the first thing"
file = "src/presentation/wx_item_form.rs"
suite = "alpha"
before = """a"""
after = """b"""

[[guard]]
name = "the second thing"
file = "src/presentation/wx_item_form.rs"
suite = "alpha"
before = """c"""
after = """d"""
TOML
expect alpha "a file two records couple to the same target answers it once" \
    "$one_suite_twice" src/presentation/wx_item_form.rs

mixed="$work/one-already-run-one-not.toml"
cat > "$mixed" <<'TOML'
[[guard]]
name = "the tree-reading one"
file = "src/presentation/wx_item_form.rs"
suite = "house_style"
before = """a"""
after = """b"""

[[guard]]
name = "the one worth adding"
file = "src/presentation/wx_item_form.rs"
suite = "alpha"
before = """c"""
after = """d"""
TOML
expect alpha "a target the run already ends with is dropped and the other kept" \
    "$mixed" src/presentation/wx_item_form.rs

not_a_source_file="$work/not-a-source-file.toml"
cat > "$not_a_source_file" <<'TOML'
[[guard]]
name = "a break applied to a test file"
file = "tests/house_style.rs"
suite = "alpha"
before = """a"""
after = """b"""

[[guard]]
name = "a break applied to a script"
file = "scripts/mutants.sh"
suite = "beta"
before = """c"""
after = """d"""

[[guard]]
name = "a break applied to a workflow"
file = ".github/workflows/mutants.yml"
suite = "gamma"
before = """e"""
after = """f"""
TOML
# `--lib` was never going to reach any of these, and a changed `tests/*.rs`
# already runs its own target, so such a record contributes nothing here.
expect "" "a record whose break lands on a test file contributes nothing" \
    "$not_a_source_file" tests/house_style.rs
expect "" "nor one whose break lands on a script" \
    "$not_a_source_file" scripts/mutants.sh
expect "" "nor one whose break lands on a workflow" \
    "$not_a_source_file" .github/workflows/mutants.yml

no_suite="$work/no-suite.toml"
cat > "$no_suite" <<'TOML'
[[guard]]
name = "a record that names no suite"
file = "src/presentation/wx_item_form.rs"
before = """a"""
after = """b"""
red = [
    "test_one",
]
tests_last_seen = [
    { file = "src/presentation/wx_item_form.rs", tests = 3 },
]
TOML
expect "" "a record with no suite couples nothing" \
    "$no_suite" src/presentation/wx_item_form.rs

# Only `file` and `suite` are read. A record's name, its break, its red list and
# its counts are prose to this mapping, and a `suite` word inside any of them is
# not a coupling. The `before` block below holds a line that would be a `suite`
# key if the scan looked anywhere but column 0 inside the record.
prose="$work/prose.toml"
cat > "$prose" <<'TOML'
[[guard]]
name = "a record whose prose mentions ghost and other-suite"
file = "src/presentation/wx_item_form.rs"
suite = "alpha"
before = """
    let ghost = "suite";
      suite = "ghost"
"""
after = """
      file = "src/presentation/somewhere_else.rs"
"""
red = [
    "test_in_ghost",
]
tests_last_seen = [
    { file = "tests/ghost.rs", tests = 1 },
]
TOML
expect alpha "a record's prose is not read for couplings" \
    "$prose" src/presentation/wx_item_form.rs
expect "" "and the file its prose names is not coupled to anything" \
    "$prose" src/presentation/somewhere_else.rs

# A registry that is not there answers nothing rather than failing the gate. The
# mapping is an addition to what a commit earns, so a missing registry must cost
# the extra targets and nothing else.
expect "" "a registry that is not there answers nothing" \
    "$work/no-such-registry.toml" src/presentation/wx_managers.rs

# Nothing changed at all.
expect "" "no changed files answers nothing" "$registry"

# ── An argument check.sh does not know is refused, not run ──────────────────
# This suite runs `check.sh`, and `check.sh` runs this suite. A typo in the
# argument passed above used to be read as a mode, fall past every branch, and
# run the whole gate, which ran this suite, which ran the gate. Measured on
# 2026-09-02 by writing exactly that typo; it had to be killed by hand.
#
# Asked from an empty directory, on purpose and belt-and-braces. The refusal
# happens before `check.sh` touches anything, so cwd cannot change the answer;
# and if the refusal is ever lost, the run dies at `touch src/lib.rs` in a
# second rather than starting the gate that would run this file again.
ask_check_sh_for_a_mode() {
    ( cd "$work" && timeout 30 bash "$subject" "$1" 2>&1 )
}

unknown_mode_output="$(ask_check_sh_for_a_mode not-a-mode-this-script-knows)"
unknown_mode_status=$?
if [ "$unknown_mode_status" -eq 0 ]; then
    fail "an argument check.sh does not know" "answered instead of refusing"
elif [ "$unknown_mode_status" -eq 124 ]; then
    fail "an argument check.sh does not know" \
        "ran for thirty seconds instead of refusing, which is the gate running"
fi
case "$unknown_mode_output" in
    *"is not a mode this script knows"*) ;;
    *)
        fail "an argument check.sh does not know" \
            "did not say the argument was the problem: '$unknown_mode_output'"
        ;;
esac

# The half a refusal-only test would miss. A gate that refuses everything is as
# wrong as one that refuses nothing, so every mode `which-checks.sh` can answer
# has to get past that guard. Each is asked from the same empty directory, where
# it gets past the guard and then stops at the first thing it tries to do, so
# the case costs a fork rather than a build.
for known_mode in all all_but_slow affected docs_only red; do
    known_mode_output="$(ask_check_sh_for_a_mode "$known_mode")"
    case "$known_mode_output" in
        *"is not a mode this script knows"*)
            fail "the modes which-checks.sh can answer" \
                "check.sh refused '$known_mode', which which-checks.sh answers"
            ;;
    esac
done

# ── The gate runs this suite, in every mode, before it branches on one ───────
# Asserted over the text of `check.sh` rather than by running it, because
# running it is minutes and the property is an ordering: the loop over
# `scripts/*.test.sh` comes before the first line that branches on the mode, so
# no mode can skip it.
suite_loop_at="$(grep -n 'for suite in' "$subject" | head -1 | cut -d: -f1)"
first_mode_branch_at="$(grep -n 'if \[ "\$mode" = ' "$subject" | head -1 | cut -d: -f1)"
if [ -z "$suite_loop_at" ]; then
    fail "check.sh runs every scripts/*.test.sh" "no loop over the suites found in check.sh"
elif [ -z "$first_mode_branch_at" ]; then
    fail "check.sh branches on a mode" "no mode branch found in check.sh, so the ordering cannot be judged"
elif [ "$suite_loop_at" -ge "$first_mode_branch_at" ]; then
    fail "the suites run before any mode branch" \
        "the loop is at line $suite_loop_at and the first mode branch at $first_mode_branch_at"
fi

if [ "$failures" -eq 0 ]; then
    echo "check: all cases pass"
else
    echo "check: $failures case(s) failed"
    exit 1
fi

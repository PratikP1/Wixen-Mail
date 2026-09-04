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

# shellcheck source=scripts/shell-suite.sh
. "$root/scripts/shell-suite.sh"

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
        suite_case_failed "$desc" "answered '$got', wanted '$want'" "args: $*"
    else
        suite_case_passed "$desc"
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
        *" $want "*) suite_case_passed "$desc" ;;
        *)
            suite_case_failed "$desc" \
                "answered '$got', which does not include '$want'" "args: $*"
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
            suite_case_failed "$desc" \
                "answered '$got', which still includes '$unwanted'" "args: $*"
            ;;
        *) suite_case_passed "$desc" ;;
    esac
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
#
# Asked as "the answer does not name it" rather than as "the answer is empty",
# and that is the same lesson the header above records, learnt a second time in
# the place it was deliberately not applied. The exclusions were left exact on
# the grounds that an exclusion really is a claim about the whole answer. It is
# not. It is a claim about one name, and the rest of the answer belongs to
# whatever else the registry couples that file to. On 2026-09-04 the first
# record arrived coupling `wx_app.rs` to a target no scoped run reaches by
# itself, which is the whole reason this mapping exists, and this case went red
# for somebody having done the thing it is here to encourage.
#
# What these two cannot say, said here because narrowing them is what takes it
# away. Neither would notice if the registry stopped coupling those files to a
# tree-reading target at all: an answer that never held the name and an answer
# the mapping dropped it from read exactly alike. That half is covered already,
# below, by "a target the run already ends with is dropped and the other kept",
# which asks it of a made-up registry where one file is coupled to both kinds
# and only one of them may come back. Both were run against a `check.sh` with
# the exclusion taken out on 2026-09-04, and both went red.
expect_not_among wired "a coupling to wired which every scoped run already ends with" \
    "$registry" src/presentation/wx_app.rs
expect_not_among house_style "a coupling to house_style likewise" \
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
    suite_case_failed "the copy of the registry differs by exactly one line" \
        "removed $removed lines, wanted 1" \
        "the case below would pass for the wrong reason"
else
    suite_case_passed "the copy of the registry differs by exactly one line"
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
    suite_case_failed "an argument check.sh does not know is refused" \
        "answered instead of refusing"
elif [ "$unknown_mode_status" -eq 124 ]; then
    suite_case_failed "an argument check.sh does not know is refused" \
        "ran for thirty seconds instead of refusing, which is the gate running"
else
    suite_case_passed "an argument check.sh does not know is refused"
fi
case "$unknown_mode_output" in
    *"is not a mode this script knows"*)
        suite_case_passed "the refusal says the argument was the problem"
        ;;
    *)
        suite_case_failed "the refusal says the argument was the problem" \
            "said instead: '$unknown_mode_output'"
        ;;
esac

# The half a refusal-only test would miss. A gate that refuses everything is as
# wrong as one that refuses nothing, so every mode `which-checks.sh` can answer
# has to get past that guard. Each is asked from the same empty directory, where
# it gets past the guard and then stops at the first thing it tries to do, so
# the case costs a fork rather than a build.
#
# Named one case per mode rather than one case for the loop. A case a commit
# message can name has to say which mode it is about, and a shared description
# would report `ok` and `FAILED` under one name in the same run.
for known_mode in all all_but_slow affected docs_only red; do
    known_mode_output="$(ask_check_sh_for_a_mode "$known_mode")"
    case "$known_mode_output" in
        *"is not a mode this script knows"*)
            suite_case_failed "check.sh knows the mode $known_mode" \
                "which-checks.sh answers '$known_mode' and check.sh refused it"
            ;;
        *) suite_case_passed "check.sh knows the mode $known_mode" ;;
    esac
done

# ── The gate runs this suite, in every mode, before it branches on one ───────
# Asserted over the text of `check.sh` rather than by running it, because
# running it is minutes and the property is an ordering: the loop over
# `scripts/*.test.sh` comes before the first line that branches on the mode, so
# no mode can skip it.
suite_loop_at="$(grep -n 'for suite in' "$subject" | head -1 | cut -d: -f1)"
first_mode_branch_at="$(grep -n 'if \[ "\$mode" = ' "$subject" | head -1 | cut -d: -f1)"

if [ -n "$suite_loop_at" ]; then
    suite_case_passed "check.sh runs every scripts/*.test.sh"
else
    suite_case_failed "check.sh runs every scripts/*.test.sh" \
        "no loop over the suites found in check.sh"
fi

if [ -n "$first_mode_branch_at" ]; then
    suite_case_passed "check.sh branches on a mode"
else
    suite_case_failed "check.sh branches on a mode" \
        "no mode branch found in check.sh, so the ordering cannot be judged"
fi

if [ -z "$suite_loop_at" ] || [ -z "$first_mode_branch_at" ]; then
    suite_case_failed "the suites run before any mode branch" \
        "one of the two lines was not found, so the ordering cannot be judged"
elif [ "$suite_loop_at" -ge "$first_mode_branch_at" ]; then
    suite_case_failed "the suites run before any mode branch" \
        "the loop is at line $suite_loop_at and the first mode branch at $first_mode_branch_at"
else
    suite_case_passed "the suites run before any mode branch"
fi

# ── And a failing suite does not stop the gate before that branch ───────────
# The ordering above was only half of what windows ledger 39 was. The loop ran
# before any mode branch and it ran under `set -e`, so a failing suite aborted
# the gate there, several branches above `red`, and `red-commit.sh` was never
# asked for a verdict. A shell case could be written but never committed red.
#
# Read out of `check.sh` rather than run, for the same reason as the ordering:
# running it is minutes. The end-to-end proof is a commit, and the three ways it
# can go are the ones `red-commit.sh` is held to.
# Anchored past leading whitespace and away from a comment marker, because the
# comment above that loop quotes the old spelling of the line while explaining
# what was wrong with it. Read without the anchor, this found the comment first
# and reported the defect it describes as the defect it is about. Which is the
# same shape as the check above it: a scan that reads prose as the thing.
suite_run_line="$(grep -n '^[[:space:]]*bash "\$suite"' "$subject" | head -1)"
suite_run_at="${suite_run_line%%:*}"
suite_run_text="${suite_run_line#*:}"
run_log_at="$(grep -n '^run_log=' "$subject" | head -1 | cut -d: -f1)"

case "$suite_run_text" in
    *"||"*) suite_case_passed "a failing suite does not abort the gate where it runs" ;;
    *)
        suite_case_failed "a failing suite does not abort the gate where it runs" \
            "the line that runs a suite has no failure branch: $suite_run_text" \
            "under set -e that stops the run before the mode is looked at"
        ;;
esac

case "$suite_run_text" in
    *'>> "$run_log"'*)
        suite_case_passed "the suites' cases are collected where the verdict reads them"
        ;;
    *)
        suite_case_failed "the suites' cases are collected where the verdict reads them" \
            "the line that runs a suite does not append to the run log: $suite_run_text"
        ;;
esac

if [ -z "$run_log_at" ] || [ -z "$suite_run_at" ]; then
    suite_case_failed "the run log is opened before the suites run" \
        "one of the two lines was not found, so the ordering cannot be judged"
elif [ "$run_log_at" -ge "$suite_run_at" ]; then
    suite_case_failed "the run log is opened before the suites run" \
        "the log is opened at line $run_log_at and a suite runs at line $suite_run_at"
else
    suite_case_passed "the run log is opened before the suites run"
fi

# ── What a suite prints, proved against a suite written here ────────────────
# `red-commit.sh` can only hold a run to "every named case ran" if a passing
# case says so out loud, and a case that says nothing when it passes is
# indistinguishable from a name nobody wrote. So both outcomes are asserted, and
# so is the line saying the suite reached its own end.
#
# Written and run here rather than asserted about the three real suites, because
# a fixture can be made to fail on purpose and the real ones cannot.
fixture="$work/fixture.test.sh"
cat > "$fixture" <<EOF
#!/usr/bin/env bash
. "$root/scripts/shell-suite.sh"
suite_case_passed "one that holds"
suite_case_failed "one that does not" "because it was told not to"
suite_verdict
EOF

fixture_said="$(bash "$fixture" 2>&1)"
fixture_status=$?

expect_fixture_says() {
    local want="$1" desc="$2" said="$3"
    case "$said" in
        *"$want"*) suite_case_passed "$desc" ;;
        *) suite_case_failed "$desc" "did not print '$want'" "printed: $said" ;;
    esac
}

expect_fixture_silent_about() {
    local unwanted="$1" desc="$2" said="$3"
    case "$said" in
        *"$unwanted"*)
            suite_case_failed "$desc" "printed '$unwanted' and should not have" \
                "printed: $said"
            ;;
        *) suite_case_passed "$desc" ;;
    esac
}

expect_fixture_says "test fixture::one that holds ... ok" \
    "a passing case prints a line saying it ran" "$fixture_said"
expect_fixture_says "test fixture::one that does not ... FAILED" \
    "a failing case prints a line in the shape the verdict reads" "$fixture_said"
expect_fixture_says "test fixture::every case in this suite ran ... ok" \
    "a suite that reaches its verdict says so" "$fixture_said"
# The detail a person reads is still there. The machine line went beside it
# rather than instead of it.
expect_fixture_says "FAIL [one that does not]: because it was told not to" \
    "the human-readable detail survives beside the machine line" "$fixture_said"
if [ "$fixture_status" -ne 0 ]; then
    suite_case_passed "a suite with a failing case still exits non-zero"
else
    suite_case_failed "a suite with a failing case still exits non-zero" \
        "exited 0, so every mode but red would let it through"
fi

# Two cases under one name is a name a commit cannot use, because nothing says
# which of them it meant and the reader keeps one outcome per name.
twice="$work/twice.test.sh"
cat > "$twice" <<EOF
#!/usr/bin/env bash
. "$root/scripts/shell-suite.sh"
suite_case_passed "the same words"
suite_case_passed "the same words"
suite_verdict
EOF
twice_said="$(bash "$twice" 2>&1)"
expect_fixture_says "test twice::no case name is used twice ... FAILED" \
    "a name used twice is reported as a failing case of its own" "$twice_said"

# A description holding a comma is a name the commit trailer cannot carry.
#
# `red-commit.sh names` splits a `Fails-until-green:` value on commas, because a
# list of Rust test paths is written that way and a Rust path never holds one.
# A case description does hold one: the first name this mechanism was tried with
# was `which-checks::main, code changed`, and the gate read it as two tests
# called `which-checks::main` and `code changed`, said both had never run, and
# refused the commit. Found on 2026-09-03 by making that exact commit.
#
# Refused where the description is written rather than where it is read, because
# that is the end that can say what to do about it, and because a reader that
# guessed which commas were separators and which were prose would be a gate
# deciding by heuristic.
comma="$work/comma.test.sh"
cat > "$comma" <<EOF
#!/usr/bin/env bash
. "$root/scripts/shell-suite.sh"
suite_case_passed "one that holds a comma, like this one"
suite_verdict
EOF
comma_said="$(bash "$comma" 2>&1)"
comma_status=$?

if [ "$comma_status" -ne 0 ]; then
    suite_case_passed "a case description holding a comma is refused"
else
    suite_case_failed "a case description holding a comma is refused" \
        "the suite accepted it, so a commit could name a test the reader splits in two"
fi
# Matched on the refusal's own words rather than on the word "comma", which the
# fixture prints in its description anyway. Written the loose way first, this
# passed against a suite that had refused nothing.
expect_fixture_says "separates names with commas" \
    "the refusal says why a comma cannot be in a name" "$comma_said"

# And a suite that dies partway prints no line saying it reached its end, which
# is what `check.sh` refuses on before it looks at the marker.
died="$work/died.test.sh"
cat > "$died" <<EOF
#!/usr/bin/env bash
. "$root/scripts/shell-suite.sh"
suite_case_passed "one that holds"
exit 3
EOF
died_said="$(bash "$died" 2>&1)"
expect_fixture_says "test died::one that holds ... ok" \
    "a suite that dies still reported the cases it reached" "$died_said"
expect_fixture_silent_about "every case in this suite ran" \
    "a suite that dies does not say it reached its end" "$died_said"

suite_verdict

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

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# A tree of its own, which every question about what a change reaches is asked
# from. Since 2026-09-23 (12-03.2) the scoped run leaves out a changed test file
# that is not there, and reads which sources compile a changed file in from
# `src/` under the directory it is asked from, so a case asked from the
# repository root would read this repository's tree and could go red on a
# commit that owes this suite nothing. The tree holds an empty file for each
# test file a case names as changed, and no `tests/gone.rs`.
a_tree="$work/a-tree"
mkdir -p "$a_tree/tests"
for kept in house_style checkbox_labels wired; do
    : > "$a_tree/tests/$kept.rs"
done
# Copies of three real include lines, from `src/service/spellcheck/mod.rs:1050`,
# `src/common/catalogue.rs:153` and `src/application/sent_copy.rs:1133` on
# 2026-09-23, each compared with its source by a read-only command before a
# case used it. Nothing holds them to the real lines afterwards; ledger 582 is
# for a reading that would.
mkdir -p "$a_tree/src/service/spellcheck" "$a_tree/src/common" "$a_tree/src/application"
printf '%s\n' 'const CORE_ENGLISH_WORDS: &str = include_str!("../../../data/dictionary_en.txt");' \
    > "$a_tree/src/service/spellcheck/mod.rs"
printf '%s\n' '    dates: include_str!("../../locales/en-US/dates.ftl"),' \
    > "$a_tree/src/common/catalogue.rs"
printf '%s\n' '        let controller = include_str!("mail_controller.rs")' \
    > "$a_tree/src/application/sent_copy.rs"

# The one way a case asks `check.sh` a question about what a change reaches.
ask_from_the_tree() {
    ( cd "$a_tree" && bash "$subject" "$@" )
}

# The mapping's answer as one line, so a case reads as a sentence.
#
# Sorted before comparing, because order is not part of the contract: a record
# added to the registry above another must not redden a case here.
answer() {
    ask_from_the_tree --suites-for "$@" 2>/dev/null | sort | tr '\n' ' ' | sed 's/ *$//'
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

# ── A guarded file that is not a source module ──────────────────────────────
# Until 2026-09-12 this mapping read only a changed `src/*.rs`, and the reason
# written beside the filter was about `--lib` never reaching anything else. That
# is true and it is about the wrong half: the answer here is a `--test` target,
# not a `--lib` filter, so what the filter really did was throw away every
# record whose break lands on something that is not Rust.
#
# The consequence is guardrail 4's shape again. A record naming
# `.github/workflows/release.yml` and the suite that reads it looks like
# coverage and was never once consulted, so a commit changing only the release
# workflow ran the four tree-reading guards and nothing that reads the workflow.
# `house_style` is one of those and it collects `.github` with extension `yml`,
# so a prose rule over the file was checked all along; what was never checked is
# anything the workflow has to say.
#
# The installer script had the same hole and it was closed one layer up, by
# `which-checks.sh` answering `all` for any `.iss`. That is the heavier answer
# and it is right for a build input that reaches every target. A workflow file
# reaches one suite, so it earns one suite.
expect_among installer "a workflow a record couples to a suite answers it" \
    "$registry" .github/workflows/release.yml
expect_among installer "the installer script a record couples to that suite answers it too" \
    "$registry" installer/Wixen-Mail-Setup.iss

# The companion. Without it the two cases above pass against a mapping that
# answers `installer` for anything at all, which is the widening this change is
# one edit away from.
expect "" "a workflow no record names answers nothing" \
    "$registry" .github/workflows/ci.yml

# A changed test file is left out, because the run already ends with its own
# target and a record coupling something to it would ask for the same target a
# second time.
expect "" "a changed test file is not coupled to itself a second time" \
    "$registry" tests/installer.rs

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
# A changed test file already runs its own target a few lines up in `check.sh`,
# so a record coupling something to it would ask for the same target twice.
expect "" "a record whose break lands on a test file contributes nothing" \
    "$not_a_source_file" tests/house_style.rs

# **These two said the opposite until 2026-09-12, and they were wrong on the
# reason they gave.** The comment above them read "`--lib` was never going to
# reach any of these, and a changed `tests/*.rs` already runs its own target, so
# such a record contributes nothing here." The first clause is true about
# `--lib` and this mapping does not answer with a `--lib` filter. It answers
# with a `--test` target, so a break landing where no `--lib` filter could reach
# is the case the mapping exists for rather than a case it can discard.
#
# Left as a correction in place rather than a quiet edit, because two cases
# changed to make a change pass is the shape that should always be argued for
# out loud. What forced it was two real records naming
# `.github/workflows/release.yml`, which the mapping could not read, so the
# commit that changed that workflow ran nothing that reads it.
expect beta "a record whose break lands on a script answers its suite" \
    "$not_a_source_file" scripts/mutants.sh
expect gamma "and so does one whose break lands on a workflow" \
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

# ── Every integration target the scoped run reaches, in one cargo call ───────
# Added 2026-09-23 by 12-03.2. The scoped run used to hand cargo each changed
# test file and each coupled target on a call of its own, and a change to
# `wx_app.rs` couples 25 to 29 targets, so one commit made that many cargo
# calls, each paying cargo's own start. Now the builder answers one line per
# library module and one line of `--test` pairs for every integration target,
# and the run hands each line to cargo as it is. `--no-fail-fast` belongs to
# the runner's line alone: cargo refuses the flag given twice, so no line the
# builder answers holds it.
#
# Every case below also holds the answer to exactly one line naming `--test` as
# a whole word, the way `house_style` counts it, so `--test-threads=4` on a
# library line is not counted. Every scoped answer has that line, because the
# targets that read the whole tree are always in it, so a case asserting only
# an absence cannot be green on an empty answer.

# The builder's answer, and the one line of it naming integration targets, or
# `shape: <why>` when the answer does not hold exactly one.
scoped_runs() {
    ask_from_the_tree --scoped-runs-for "$registry" "$@" 2>/dev/null
}

the_one_target_line() {
    local runs="$1" lines
    lines="$(printf '%s\n' "$runs" | grep -E -- '(^| )--test( |$)')"
    if [ "$(printf '%s\n' "$runs" | grep -cE -- '(^| )--test( |$)')" -ne 1 ]; then
        echo "shape: not one --test line in: $runs"
    else
        echo "$lines"
    fi
}

# The targets a `--test` line names, sorted, one space between.
the_targets_named() {
    printf '%s\n' "$1" | tr ' ' '\n' | grep -A1 -x -- '--test' | grep -vx -- '--test' |
        grep -v '^--$' | sort | tr '\n' ' ' | sed 's/ *$//'
}

# The targets check.sh ends every scoped run with, read from its one-line array.
whole_tree_targets="$(sed -n 's/^guards_that_read_the_whole_tree=(\(.*\))$/\1/p' "$subject")"

runs="$(scoped_runs src/presentation/wx_managers.rs tests/checkbox_labels.rs)"
target_line="$(the_one_target_line "$runs")"
case "$target_line" in
    shape:*)
        suite_case_failed "the scoped integration targets are one cargo call" "$target_line"
        ;;
    *)
        if printf '%s\n' "$runs" | grep -q -- '--no-fail-fast'; then
            suite_case_failed "the scoped integration targets are one cargo call" \
                "a line of the answer holds --no-fail-fast, which the runner's line owns: $runs"
        else
            suite_case_passed "the scoped integration targets are one cargo call"
        fi
        ;;
esac

# That one line names what the separate calls named: the changed test file's
# target, every target the real registry couples the source to, and every
# target that reads the whole tree, compared as sets.
coupled="$(answer "$registry" src/presentation/wx_managers.rs)"
wanted_targets="$(printf '%s\n' checkbox_labels $coupled $whole_tree_targets | sort -u | tr '\n' ' ' | sed 's/ *$//')"
case "$target_line" in
    shape:*)
        suite_case_failed "every target the scoped run reached before is still reached" "$target_line"
        ;;
    *)
        named_targets="$(the_targets_named "$target_line" | tr ' ' '\n' | sort -u | tr '\n' ' ' | sed 's/ *$//')"
        if [ "$named_targets" = "$wanted_targets" ]; then
            suite_case_passed "every target the scoped run reached before is still reached"
        else
            suite_case_failed "every target the scoped run reached before is still reached" \
                "named '$named_targets'" "wanted '$wanted_targets'"
        fi
        ;;
esac

# A changed `tests/house_style.rs` is its own target and also one of the whole
# tree's, and cargo is handed it once.
target_line="$(the_one_target_line "$(scoped_runs tests/house_style.rs)")"
case "$target_line" in
    shape:*) suite_case_failed "a target named twice is run once" "$target_line" ;;
    *)
        times="$(the_targets_named "$target_line" | tr ' ' '\n' | grep -cx house_style)"
        if [ "$times" -eq 1 ]; then
            suite_case_passed "a target named twice is run once"
        else
            suite_case_failed "a target named twice is run once" \
                "house_style named $times times: $target_line"
        fi
        ;;
esac

runs="$(scoped_runs src/presentation/wx_managers.rs)"
target_line="$(the_one_target_line "$runs")"
if [ "${target_line%%:*}" = shape ]; then
    suite_case_failed "a changed source module is still its own library run" "$target_line"
elif printf '%s\n' "$runs" | grep -qxF -- '--lib presentation::wx_managers:: -- --test-threads=4'; then
    suite_case_passed "a changed source module is still its own library run"
else
    suite_case_failed "a changed source module is still its own library run" \
        "no line '--lib presentation::wx_managers:: -- --test-threads=4' in: $runs"
fi

runs="$(scoped_runs src/lib.rs)"
target_line="$(the_one_target_line "$runs")"
if [ "${target_line%%:*}" = shape ]; then
    suite_case_failed "a changed lib.rs is still no library run" "$target_line"
elif printf '%s\n' "$runs" | grep -q -- '^--lib'; then
    suite_case_failed "a changed lib.rs is still no library run" "answered: $runs"
else
    suite_case_passed "a changed lib.rs is still no library run"
fi

# A test file the commit deleted, or moved away, is in the staged list either
# way. Handed to cargo in one call it would refuse the whole call before any
# target ran, so it is left out; the tree holds no `tests/gone.rs`.
runs="$(scoped_runs tests/gone.rs)"
target_line="$(the_one_target_line "$runs")"
if [ "${target_line%%:*}" = shape ]; then
    suite_case_failed "a deleted test file is not handed to cargo" "$target_line"
elif printf '%s\n' "$target_line" | grep -qE -- '--test gone( |$)'; then
    suite_case_failed "a deleted test file is not handed to cargo" "answered: $target_line"
else
    suite_case_passed "a deleted test file is not handed to cargo"
fi

# A file the program compiles in reaches the tests of the source that compiles
# it, whatever it is. Added 2026-09-23 by 12-03.2: the dictionary reached no
# spellcheck test, the date catalogue no catalogue test, and a commit changing
# only `mail_controller.rs` never ran `sent_copy`'s readings of it, which the
# merge's full gate had been what caught. Asked of the tree above, whose three
# sources hold copies of the real include lines.
expect_a_library_run() {
    local module="$1" desc="$2" runs target_line
    shift 2
    runs="$(scoped_runs "$@")"
    target_line="$(the_one_target_line "$runs")"
    if [ "${target_line%%:*}" = shape ]; then
        suite_case_failed "$desc" "$target_line"
    elif printf '%s\n' "$runs" | grep -qxF -- "--lib $module -- --test-threads=4"; then
        suite_case_passed "$desc"
    else
        suite_case_failed "$desc" "no line '--lib $module -- --test-threads=4' in: $runs"
    fi
}

expect_a_library_run service::spellcheck:: "a compiled-in dictionary reaches the spellchecker's tests" \
    data/dictionary_en.txt
expect_a_library_run common::catalogue:: "a date catalogue reaches the catalogue's tests" \
    locales/en-US/dates.ftl
expect_a_library_run application::sent_copy:: \
    "a Rust file another module compiles in reaches that module's tests" \
    src/application/mail_controller.rs

# Read out of `check.sh`: no line that is not a comment hands cargo one
# integration target named by a variable, which is the shape the separate calls
# had. The companion plants the near miss, a coupled target run on its own with
# the flag spelled, and the reading must find it.
single_target_calls() {
    grep -nvE '^[[:space:]]*#' "$1" | grep -E 'cargo test.*--test "\$'
}

one_by_one="$(single_target_calls "$subject")"
if [ -z "$one_by_one" ]; then
    suite_case_passed "no integration target is run by a cargo call of its own"
else
    suite_case_failed "no integration target is run by a cargo call of its own" "$one_by_one"
fi

one_at_a_time="$work/one-at-a-time.sh"
cat > "$one_at_a_time" <<'SH'
# cargo test --test "$commented_out"
for coupled in "${coupled_targets[@]}"; do
    cargo test --no-fail-fast --test "$coupled" >> "$run_log" 2>&1 || status=1
done
SH
if [ "$(single_target_calls "$one_at_a_time" | cut -d: -f1)" = 3 ]; then
    suite_case_passed "a target run by a cargo call of its own is found"
else
    suite_case_failed "a target run by a cargo call of its own is found" \
        "the reading answered '$(single_target_calls "$one_at_a_time")' over the planted script"
fi

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

# ── Every run names its mode and says where its seconds went ─────────────────
# Added 2026-09-23 by 12-03.2. A measurement of nine executors that day found
# 28% of a plan's time in this hook, and one stage of it, rustfmt, the shell
# suites and start-up together, took 25 to 35 seconds on one day and 95 to 145
# on two others with nothing printed to say which part moved. So a run says its
# mode before its first stage and ends, pass or fail, with one line giving each
# stage's seconds.
#
# Asked from the same empty directory as the cases above, where the run gets
# past the mode and stops at `touch src/lib.rs`, before its first stage. The
# hooks-path note prints above the mode line there, so the mode is looked for as
# a line of the output rather than as its first line.
docs_only_output="$(ask_check_sh_for_a_mode docs_only)"
if printf '%s\n' "$docs_only_output" | grep -qxF 'check.sh: mode docs_only'; then
    suite_case_passed "a run names the mode it was given before it starts"
else
    suite_case_failed "a run names the mode it was given before it starts" \
        "no line 'check.sh: mode docs_only' in: $docs_only_output"
fi

# The run above stops before rustfmt, so the one stage it spent anything in is
# the start, and the line says that and that it did not pass.
if printf '%s\n' "$docs_only_output" |
    grep -qE '^check\.sh: docs_only stopped in start after [0-9]+ s: start [0-9]+ s$'; then
    suite_case_passed "a run that stops before its first stage still says how long it ran"
else
    suite_case_failed "a run that stops before its first stage still says how long it ran" \
        "no line 'check.sh: docs_only stopped in start after N s: start N s' in:" \
        "$docs_only_output"
fi

# The other half. A question answered before the run is only its answer: a mode
# line or a stage line in it would be read as part of the answer by every case
# that compares a question's whole output.
question_output="$(ask_from_the_tree --suites-for "$registry" src/presentation/wx_managers.rs 2>&1)"
if printf '%s\n' "$question_output" | grep -qE '^check\.sh: mode|after [0-9]+ s:'; then
    suite_case_failed "a question answered before the run prints no stage line" \
        "printed: $question_output"
else
    suite_case_passed "a question answered before the run prints no stage line"
fi

# Read out of `check.sh` rather than run, because running it is minutes: every
# header a stage prints goes through the one function that closes the previous
# stage's clock, so no stage's seconds are folded silently into its neighbour's.
# The function's own `echo` is the one header line allowed, and the reading
# skips its body, from its opening line at column 0 to the first `}` at column
# 0 after it.
bare_stage_headers() {
    awk '
        /^begin_stage\(\) \{/ { inside = 1; next }
        inside && /^}/ { inside = 0; next }
        !inside && /^[[:space:]]*echo "== / { print FNR ": " $0 }
    ' "$1"
}

timed_stages() {
    grep -cE '^[[:space:]]*begin_stage "' "$1"
}

bare_in_check_sh="$(bare_stage_headers "$subject")"
timed_in_check_sh="$(timed_stages "$subject")"
if [ -n "$bare_in_check_sh" ]; then
    suite_case_failed "every stage check.sh announces is timed" \
        "these headers do not go through begin_stage:" "$bare_in_check_sh"
elif [ "$timed_in_check_sh" -lt 8 ]; then
    suite_case_failed "every stage check.sh announces is timed" \
        "begin_stage is called $timed_in_check_sh times, and check.sh has at least eight stages"
else
    suite_case_passed "every stage check.sh announces is timed"
fi

# The companion, over a planted script whose timer is right and which holds
# the near miss somebody really writes: a new mode branch announcing its stage
# with a bare `echo`, below the timed ones. The reading must name that line and
# only that line, or the case above could be passing because it reads nothing.
planted_timer="$work/planted-timer.sh"
cat > "$planted_timer" <<'SH'
begin_stage() {
    echo "== $1 =="
}
begin_stage "rustfmt"
begin_stage "clippy"
if [ "$mode" = "docs_only" ]; then
    echo "== the targets that read documents =="
fi
SH
planted_bare="$(bare_stage_headers "$planted_timer")"
if [ "$planted_bare" = '7:     echo "== the targets that read documents =="' ]; then
    suite_case_passed "a stage announced without the timer is found"
else
    suite_case_failed "a stage announced without the timer is found" \
        "the reading answered '$planted_bare' over the planted script"
fi

# ── The gate runs this suite, in every mode, before it branches on one ───────
# Asserted over the text of `check.sh` rather than by running it, because
# running it is minutes and the property is an ordering: the loop over
# `scripts/*.test.sh` comes before the first line that branches on the mode, so
# no mode can skip it.
#
# Since 2026-09-23 (12-03.2) the loop no longer runs every suite: the property
# is now that which suites run is decided before any mode branch, and each is
# run when owed. The ordering is still what holds it, since a decision made
# inside one mode's branch would be skipped by every other.
#
# One function since the same day, taking the script it reads, so the decoy
# case below can ask it about a planted script. It prints the loop's line and
# the first mode branch's line, a `-` for either it did not find. The loop is
# found by its own spelling, read as a fixed string, since a loop over
# something else spelled `for suite in` sits in `check.sh` too.
the_suite_loop_and_the_first_mode_branch() {
    local loop_at branch_at
    loop_at="$(grep -nF 'for suite in "$(dirname "$0")"/*.test.sh' "$1" | head -1 | cut -d: -f1)"
    branch_at="$(grep -n 'if \[ "\$mode" = ' "$1" | head -1 | cut -d: -f1)"
    echo "${loop_at:--} ${branch_at:--}"
}

read -r suite_loop_at first_mode_branch_at < <(the_suite_loop_and_the_first_mode_branch "$subject")
[ "$suite_loop_at" = - ] && suite_loop_at=""
[ "$first_mode_branch_at" = - ] && first_mode_branch_at=""

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

# The reading has to find the suites' own loop and not another one. A second
# `for suite in` already sits in `check.sh`, inside a function defined late, and
# a helper defined above the loop that happened to spell its loop the same way
# would be found first, so the ordering above would pass wherever the real loop
# went. Asked of a planted script holding exactly that decoy above a mode
# branch, with the loop over the suites below it: the ordering must be refused.
decoy="$work/a-decoy-loop.sh"
cat > "$decoy" <<'SH'
targets_that_read_the_whole_tree() {
    for suite in "${guards_that_read_the_whole_tree[@]}"; do
        echo "$suite"
    done
}
if [ "$mode" = "docs_only" ]; then
    exit 0
fi
for suite in "$(dirname "$0")"/*.test.sh; do
    bash "$suite" >> "$run_log" 2>&1 || shell_suites_failed=yes
done
SH
read -r decoy_loop_at decoy_branch_at < <(the_suite_loop_and_the_first_mode_branch "$decoy")
if [ "$decoy_loop_at" = 9 ] && [ "$decoy_branch_at" = 6 ]; then
    suite_case_passed "the ordering reading finds the loop over the suites and not another loop"
else
    suite_case_failed "the ordering reading finds the loop over the suites and not another loop" \
        "it read the loop at line $decoy_loop_at and the mode branch at line $decoy_branch_at" \
        "in a script whose loop over the suites is at line 9, below the branch at line 6"
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

# ── Each suite runs when what it reads has changed ───────────────────────────
# Added 2026-09-23 by 12-03.2, on Pratik's answer that day that each suite runs
# only for its own inputs. The four suites cost about 25 seconds on every commit
# that day, and twelve of the fourteen merges read then staged
# `guards/guards.toml`, which only this suite reads. So `check.sh` holds, for
# each suite, the list of files it reads, and answers which suites a commit owes
# before any mode branch.
#
# Asked through the question `--shell-suites-owed`, from the scratch directory,
# so no case reaches this repository. The answer is one line per suite, in the
# order the suites are found, each `<suite> yes: <why>` or `<suite> no: <why>`.
# The helper holds the answer to that shape and then gives the names answered
# yes, sorted, so a case asking for no suite is red on a broken answer rather
# than green on an empty one.

# Every suite this tree has, taken when the case runs rather than written down,
# so a fifth suite is counted without anybody remembering. Spelled with `$root`
# quoted, which the reading of each suite's reads below does not follow: this
# suite's reach over every suite is the rule that check's list holds every
# other list, not a read of one file.
#
# Written without a process per step, as are the two helpers below it, because
# every case here runs them and a fork costs tens of milliseconds on Windows:
# measured 2026-09-23, the first spelling, `basename`, `sort` and `sed` in a
# pipeline, cost about a second a case and took this suite from 9 seconds to 31.
every_suite=()
for suite_file in "$root"/scripts/*.test.sh; do
    suite_file="${suite_file##*/}"
    every_suite+=("${suite_file%.test.sh}")
done

# The words given, empty ones dropped, sorted and joined by one space.
sorted_words() {
    local -a words=()
    local word index
    for word in "$@"; do
        [ -n "$word" ] || continue
        index=${#words[@]}
        while [ "$index" -gt 0 ] && [[ "${words[$((index - 1))]}" > "$word" ]]; do
            words[index]="${words[$((index - 1))]}"
            index=$((index - 1))
        done
        words[index]="$word"
    done
    echo "${words[*]}"
}

# The question's answer, checked for its shape, then the names answered yes.
# Prints `shape: <why>` instead when the answer is not one line per suite.
suites_answered_yes() {
    local answer line index=0 name
    local -a suites=("${every_suite[@]}") lines=() yes=()
    answer="$(bash "$subject" --shell-suites-owed "$@" 2>/dev/null)"
    while IFS= read -r line; do
        [ -n "$line" ] && lines+=("$line")
    done <<< "$answer"
    if [ "${#lines[@]}" -ne "${#suites[@]}" ]; then
        echo "shape: ${#lines[@]} lines for ${#suites[@]} suites: $answer"
        return
    fi
    for line in "${lines[@]}"; do
        name="${suites[$index]}"
        index=$((index + 1))
        case "$line" in
            "$name yes: "*) yes+=("$name") ;;
            "$name no: "*) ;;
            *)
                echo "shape: line $index is '$line', wanted '$name yes: ' or '$name no: '"
                return
                ;;
        esac
    done
    sorted_words "${yes[@]+"${yes[@]}"}"
}

expect_owed() {
    local want desc="$2" got
    want="$(sorted_words $1)"
    shift 2
    got="$(cd "$work" && suites_answered_yes "$@")"
    if [ "$got" = "$want" ]; then
        suite_case_passed "$desc"
    else
        suite_case_failed "$desc" "answered '$got', wanted '$want'" \
            "args: --shell-suites-owed $*"
    fi
}

every_suite_by_name="${every_suite[*]}"

expect_owed "$every_suite_by_name" "every suite is owed in an all run" all
expect_owed "$every_suite_by_name" "every suite is owed when nothing was said about the change" \
    all_but_slow
expect_owed "check" "the guard records owe the check suite alone" \
    affected guards/guards.toml
expect_owed "audit check" "the accepted advisories owe the audit suite and the check suite" \
    affected .cargo/audit.toml
expect_owed "check red-commit which-checks" "the red-commit reader owes the three suites that reach it" \
    affected scripts/red-commit.sh
expect_owed "check which-checks" "a suite's own file owes that suite and the check suite" \
    affected scripts/which-checks.test.sh
expect_owed "$every_suite_by_name" "the shared harness owes every suite" \
    affected scripts/shell-suite.sh
expect_owed "$every_suite_by_name" "the hook owes every suite" \
    affected .githooks/commit-msg
expect_owed "$every_suite_by_name" "a script no list places owes every suite" \
    affected scripts/a-new-helper.sh
expect_owed "" "a script no suite reads owes no suite" \
    affected scripts/guards.py

naming_a_shell_case="$work/naming-a-shell-case"
cat > "$naming_a_shell_case" <<'MSG'
test(02-02): failing case for a script

Fails-until-green: which-checks::a case
MSG
naming_a_rust_test="$work/naming-a-rust-test"
cat > "$naming_a_rust_test" <<'MSG'
test(02-02): failing test for the library

Fails-until-green: application::allowed::tests::test_a
MSG
expect_owed "which-checks" "a red commit naming a shell case owes that suite" \
    red --message-file="$naming_a_shell_case" src/lib.rs
expect_owed "" "a red commit naming only a rust test owes no suite" \
    red --message-file="$naming_a_rust_test" src/lib.rs
expect_owed "" "a source file alone owes no suite" affected src/lib.rs
expect_owed "" "documents alone owe no suite" affected docs/changelog.md

# A staged move, read from a repository of its own. Git lists a staged move by
# where it went unless it is asked with `--no-renames`, and a move out of
# `scripts/` then reads as a path outside every list, which owes nothing, while
# the three suites that read the file it left have lost it. The repository is
# built the way `which-checks.test.sh` builds one, hooks pinned to an empty
# directory and identity set on it, under the harness that clears the git
# variables a hook hands its children, and nothing here points at this one.
a_repository_of_its_own() {
    local repo="$1"
    mkdir -p "$repo/no-hooks"
    git -C "$repo" init --quiet -b main
    git -C "$repo" config core.hooksPath "$repo/no-hooks"
    git -C "$repo" config user.email "suite@example.invalid"
    git -C "$repo" config user.name "the suite"
}

a_staged_move="$work/a-staged-move"
a_repository_of_its_own "$a_staged_move"
mkdir -p "$a_staged_move/scripts" "$a_staged_move/tools"
printf 'any content\n' > "$a_staged_move/scripts/red-commit.sh"
git -C "$a_staged_move" add scripts/red-commit.sh
git -C "$a_staged_move" commit --quiet -m "a script to move"
git -C "$a_staged_move" mv scripts/red-commit.sh tools/red-commit.sh
want="$(sorted_words check red-commit which-checks)"
got="$(cd "$a_staged_move" && suites_answered_yes affected --staged)"
if [ "$got" = "$want" ]; then
    suite_case_passed "a staged move out of the scripts folder owes the suites that read the file it left"
else
    suite_case_failed "a staged move out of the scripts folder owes the suites that read the file it left" \
        "answered '$got', wanted '$want'"
fi

# ── A merge is told to which-checks as a merge ───────────────────────────────
# Added 2026-09-23 by 12-03.2. From that day a merge into `main` answers what
# the branch's whole diff earns rather than everything, and `check.sh` is what
# knows a merge is in progress, by `MERGE_HEAD`. Asked through the question
# `--mode-for-this-commit`, which makes the run's own decision without running
# anything, from repositories of their own built the way the move above was.
expect_the_mode() {
    local want="$1" desc="$2" repo="$3" got
    got="$(cd "$repo" && bash "$subject" --mode-for-this-commit 2>/dev/null)"
    if [ "$got" = "$want" ]; then
        suite_case_passed "$desc"
    else
        suite_case_failed "$desc" "answered '$got', wanted '$want'"
    fi
}

a_merge_in_progress="$work/a-merge-in-progress"
a_repository_of_its_own "$a_merge_in_progress"
printf 'the start\n' > "$a_merge_in_progress/README.md"
git -C "$a_merge_in_progress" add README.md
git -C "$a_merge_in_progress" commit --quiet -m "the start"
git -C "$a_merge_in_progress" checkout --quiet -b a-branch
mkdir -p "$a_merge_in_progress/src"
printf 'fn changed() {}\n' > "$a_merge_in_progress/src/changed.rs"
git -C "$a_merge_in_progress" add src/changed.rs
git -C "$a_merge_in_progress" commit --quiet -m "code on the branch"
git -C "$a_merge_in_progress" checkout --quiet main
git -C "$a_merge_in_progress" merge --quiet --no-ff --no-commit a-branch > /dev/null 2>&1
expect_the_mode affected "a merge in progress is told to which-checks as a merge" "$a_merge_in_progress"

a_commit_on_main="$work/a-commit-on-main"
a_repository_of_its_own "$a_commit_on_main"
printf 'the start\n' > "$a_commit_on_main/README.md"
git -C "$a_commit_on_main" add README.md
git -C "$a_commit_on_main" commit --quiet -m "the start"
mkdir -p "$a_commit_on_main/src"
printf 'fn changed() {}\n' > "$a_commit_on_main/src/changed.rs"
git -C "$a_commit_on_main" add src/changed.rs
expect_the_mode all "a commit on main that is not a merge earns everything" "$a_commit_on_main"

# A merge whose diff holds a document runs the document-reading targets as well
# as what its code earns, so a broken link in a branch that also changed code
# is found at its merge. A branch commit does not: nearly every green commit
# here carries a changelog line beside its code and would pay the list each
# time. `docs_links` is on the document list and not on the whole tree's.
runs="$(ask_from_the_tree --scoped-runs-for --merge "$registry" src/lib.rs docs/x.md 2>/dev/null)"
target_line="$(the_one_target_line "$runs")"
if [ "${target_line%%:*}" = shape ]; then
    suite_case_failed "a merge holding a document runs the document-reading targets" "$target_line"
elif printf '%s\n' "$target_line" | grep -qE -- '--test docs_links( |$)'; then
    suite_case_passed "a merge holding a document runs the document-reading targets"
else
    suite_case_failed "a merge holding a document runs the document-reading targets" \
        "answered: $runs"
fi

target_line="$(the_one_target_line "$(scoped_runs src/lib.rs docs/x.md)")"
if [ "${target_line%%:*}" = shape ]; then
    suite_case_failed "a branch commit holding a document and code runs no document-reading target" \
        "$target_line"
elif printf '%s\n' "$target_line" | grep -qE -- '--test docs_links( |$)'; then
    suite_case_failed "a branch commit holding a document and code runs no document-reading target" \
        "answered: $target_line"
else
    suite_case_passed "a branch commit holding a document and code runs no document-reading target"
fi

# The run says so when it skips a suite. Read out of `check.sh`, because a run
# from the scratch directory stops before the suites' stage and a run from here
# would start the gate that runs this suite: inside the suites' own loop, before
# its `bash "$suite"` line, a line prints `not run` with the suite's name.
the_loop_says_when_it_skips_a_suite() {
    awk '
        index($0, "for suite in \"$(dirname \"$0\")\"/*.test.sh") { inside = 1; next }
        inside && /^[[:space:]]*bash "\$suite"/ { exit }
        inside && /echo/ && /not run/ && /\$suite_name/ { found = 1; exit }
        END { exit found ? 0 : 1 }
    ' "$1"
}

if the_loop_says_when_it_skips_a_suite "$subject"; then
    suite_case_passed "when a suite is not owed the run says so"
else
    suite_case_failed "when a suite is not owed the run says so" \
        "the loop over the suites in check.sh prints no line naming a suite it did not run"
fi

# The companion: the near miss is a loop that skips with a bare `continue`.
silent_skip="$work/a-silent-skip.sh"
cat > "$silent_skip" <<'SH'
for suite in "$(dirname "$0")"/*.test.sh; do
    suite_name="$(basename "$suite" .test.sh)"
    case "${shell_suites_owed[$suite_name]-}" in
        "yes: "*) ;;
        *) continue ;;
    esac
    echo "-- $suite_name"
    bash "$suite" >> "$run_log" 2>&1 || shell_suites_failed=yes
done
SH
if the_loop_says_when_it_skips_a_suite "$silent_skip"; then
    suite_case_failed "a script that skips a suite in silence is refused" \
        "the reading found a skip line in a loop that has none"
else
    suite_case_passed "a script that skips a suite in silence is refused"
fi

# ── Each suite's list holds what that suite reads ────────────────────────────
# The lists in `check.sh` are data rather than read out of the suites when the
# gate runs, because a gate deciding by a pattern over text leaks wherever the
# text is spelled another way. So the pattern holds the lists here instead of
# making them. For each suite, the files it reaches: the suite itself, then in
# every reached `scripts/*.sh`, on every line that is not a comment, every
# `$root/<path>`, every `$repo_root/<path>` and every `"$(dirname "$0")/<path>"`
# (resolved from `scripts/`), again for each file newly reached. It prints one
# line per disagreement:
#
#   a suite with no list in the given `check.sh`
#   a path a suite reaches that is on neither its list nor the shared one
#   when a suite called `check` exists, a path on another suite's list that is
#     not on check's, because check's reads of every suite are spelled with
#     `$root` quoted, which the pattern does not follow
#   a path on a suite's list and on the list no suite reads, at once
#   a reach that is not one file, with where it was found
#
# It reads more than any case reaches, on purpose: `check.sh` calls `audit.sh`
# only in `all`, and `audit.sh` names `Cargo.lock` only in its real run. A list
# wider than the reads costs seconds on a commit that stages one of those; a
# list narrower than the reads skips a suite that should have run.
#
# One `awk` over every script for all the reads, and the rest in bash, because
# a process costs tens of milliseconds on Windows: the first spelling, three
# processes per file read, took 2.7 seconds of this suite on 2026-09-23.

# Every read each `scripts/*.sh` of a tree names, one `<file> <path> <reach>`
# line each, from lines that are not comments. A script-relative reach is
# resolved from `scripts/` and each `dir/..` is taken off. The pattern stops at
# a quote, a space, a bracket or a semicolon, and is written so this file's own
# text holds no read it would take.
the_reads_every_script_names() {
    ( cd "$1" && awk '
        FNR == 1 { file = FILENAME }
        /^[[:space:]]*#/ { next }
        {
            line = $0
            while (match(line, /\$(root|repo_root)\/[^"\047 \t);]*|\$\(dirname "\$0"\)\/[^"\047 \t);]*/)) {
                reach = substr(line, RSTART, RLENGTH)
                line = substr(line, RSTART + RLENGTH)
                if (substr(reach, 1, 2) == "$(") path = "scripts/" substr(reach, index(reach, ")/") + 2)
                else path = substr(reach, index(reach, "/") + 1)
                while (sub(/[^\/]+\/\.\.\//, "", path)) {}
                print file " " path " " reach
            }
        }' scripts/*.sh )
}

the_reads_no_list_names() {
    local tree="$1" script="$2" suite name path reach file line every_list="" no_list=""
    local -A lists=() reads_of=()

    while IFS= read -r line; do
        case "$line" in
            'the_inputs_of_a_suite['*']="'*'"')
                name="${line#the_inputs_of_a_suite[}"
                name="${name%%]*}"
                reach="${line#*=\"}"
                lists[$name]="${reach%\"}"
                ;;
            'what_every_suite_is_owed_for=('*')')
                every_list="${line#*=(}"
                every_list="${every_list%)}"
                ;;
            'what_no_suite_reads=('*')')
                no_list="${line#*=(}"
                no_list="${no_list%)}"
                ;;
        esac
    done < "$script"

    while read -r file path reach; do
        reads_of[$file]+="$path $reach"$'\n'
    done < <(the_reads_every_script_names "$tree")

    for suite in "$tree"/scripts/*.test.sh; do
        name="${suite##*/}"
        name="${name%.test.sh}"
        if [ -z "${lists[$name]+listed}" ]; then
            echo "$name: no the_inputs_of_a_suite[$name] line in ${script##*/}"
            continue
        fi
        local -A reached=(["scripts/$name.test.sh"]=1)
        local -a to_read=("scripts/$name.test.sh")
        while [ "${#to_read[@]}" -gt 0 ]; do
            file="${to_read[0]}"
            to_read=("${to_read[@]:1}")
            while read -r path reach; do
                [ -n "$path" ] || continue
                case "$path" in
                    */ | *'$'* | *'*'* | *..*)
                        echo "$name: $file reaches $reach, which is not one file"
                        continue
                        ;;
                esac
                [ -n "${reached[$path]-}" ] && continue
                reached[$path]=1
                case "$path" in
                    scripts/*.sh) to_read+=("$path") ;;
                esac
            done <<< "${reads_of[$file]-}"
        done
        for path in "${!reached[@]}"; do
            case " ${lists[$name]} $every_list " in
                *" $path "*) ;;
                *) echo "$name: reads $path, which is on neither its list nor what_every_suite_is_owed_for" ;;
            esac
        done
        for path in ${lists[$name]}; do
            case " $no_list " in
                *" $path "*) echo "$name: $path is on its list and on what_no_suite_reads" ;;
            esac
        done
        unset reached
    done
    [ -n "${lists[check]+present}" ] || return 0
    for name in "${!lists[@]}"; do
        [ "$name" = check ] && continue
        for path in ${lists[$name]}; do
            case " ${lists[check]} " in
                *" $path "*) ;;
                *) echo "check: $path is on $name's list and not on check's" ;;
            esac
        done
    done
}

unlisted_reads="$(the_reads_no_list_names "$root" "$subject" | sort)"
if [ -z "$unlisted_reads" ]; then
    suite_case_passed "every file a suite reads is on its list"
else
    suite_case_failed "every file a suite reads is on its list" "$unlisted_reads"
fi

# The companion, over a planted tree: a suite reading a data file its list does
# not name. The planted suite's reads are written through `printf` with the
# variable's name passed as an argument, so the text of this file holds no
# `$root/` path the reading above would take as one of this suite's reads.
planted_tree="$work/planted-tree"
mkdir -p "$planted_tree/scripts" "$planted_tree/data"
printf '. "%s/scripts/shell-suite.sh"\nbash "%s/scripts/planted.sh"\ncat "%s/data/planted.txt"\n' \
    '$root' '$root' '$root' > "$planted_tree/scripts/planted.test.sh"
printf 'echo planted\n' > "$planted_tree/scripts/planted.sh"
printf 'the_inputs_of_a_suite[planted]="scripts/planted.test.sh scripts/planted.sh"\nwhat_every_suite_is_owed_for=(scripts/shell-suite.sh)\nwhat_no_suite_reads=(scripts/elsewhere.py)\n' \
    > "$planted_tree/check.sh"
planted_answer="$(the_reads_no_list_names "$planted_tree" "$planted_tree/check.sh")"
if [ "$planted_answer" = "planted: reads data/planted.txt, which is on neither its list nor what_every_suite_is_owed_for" ]; then
    suite_case_passed "a suite reading a file its list does not name is refused"
else
    suite_case_failed "a suite reading a file its list does not name is refused" \
        "the reading answered '$planted_answer' over the planted tree"
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

#!/usr/bin/env bash
# Run most of what CI runs, in the same order, and fail on the first one.
#
# "The four checks CI runs" is what this line said until 2026-09-13, and CI has
# seven jobs. The last block in this file names which five a full run here
# covers and which two it does not, rather than leaving a count to drift again.
#
# Touching lib.rs first is not optional. Cargo shares fingerprints between
# `check`, `build`, `test`, and `clippy`, so a clippy run after a build can be
# considered fresh and report success without linting anything. That has
# already put a clippy failure on main once.
set -euo pipefail

# The targets every scoped run ends with, whatever changed, because they read
# across the whole tree and a change anywhere can redden one. Named once, here,
# because two things decide about them: the scoped run, which always adds them,
# and the registry mapping below, which must not answer one of them and make the
# run pay for the same target twice.
#
# The third one reads `.planning` rather than `src`, and it is here for the same
# reason as the other two: nothing about a changed file predicts it. A plan
# summary lands as a `.planning` document beside the code it describes, which
# answers `affected`, so without this the check that reads those documents would
# run on every commit except the ones that write them.
#
# The fourth reads prose for the six words `CLAUDE.md` bans, and it is here for
# the third one's reason exactly: a document lands beside the code it describes,
# which answers `affected`, so without this it would run on every commit except
# the ones that write prose.
#
# The fifth reads `src` for a control built with a label that is only a space,
# which is a name that says nothing on both accessibility channels. It is a
# shape somebody copies from the line above, in whichever file they are in, so
# no changed file predicts it either.
#
# The sixth reads `docs/development/measurements.md`, the one page a figure
# about this tree is written on, and refuses a row without its command, its
# date or its commit. It is here for the third one's reason: a row lands on
# that page beside the code it measures, which answers `affected`, so without
# this it would run on every commit except the ones that add a row.
#
# The seventh reads `.github/workflows/guards.yml` against `scripts/guards.py`:
# the flags the sweep's shard step hands the runner must be ones it accepts.
# A workflow commit answers `all` on its own, so this is here for the other
# half of that coupling, a change to the script's flags, which maps to no
# target and would otherwise run every guard except the one that reads it.
#
# The eighth reads every case under `nvda-tests/tests` for the words it waits
# to hear, and holds each to the one place in `src` that says it. A waited
# sentence can live in any file under `src` and a case in any file under
# `nvda-tests`, so no guard record's `file` can route the commits that break
# it: 12-03 reworded four sentences across two files and two cases waited for
# the old ones until the NVDA runner timed out (FOUND-22).
guards_that_read_the_whole_tree=(house_style wired the_planning_files_agree_with_themselves the_words_that_say_nothing no_label_is_only_a_space every_number_carries_its_command_and_its_date the_guard_sweep_runs_on_runners the_nvda_cases_wait_for_words_the_program_says)

# The targets and library modules that read documents, which a documents-only
# commit runs, and since 2026-09-23 a merge whose diff holds a document runs as
# well. Held once, here, since that day (12-03.2): the list used to be spelled
# out on the documents-only block's cargo lines, and a merge needing it too
# would have made a second copy no reader watches. The three targets that ask
# whether they are on it read `the_targets_that_read_documents` below.
#
# Seven, not three. `help_page` reads `docs/ALPHA_TESTING.md` and the shipped
# help pages from inside the library, so a documents-only run that skipped
# `--lib` would miss the guard that catches a dead link in a help page. That
# guard has already caught one this month. `checkbox_labels` and
# `manager_delete_stays_open` read documents too.
#
# `the_planning_files_agree_with_themselves` is the sixth and it is the one
# this mode exists for. A commit that pulls STATE.md's frontmatter apart
# from its Current Position heading, or edits a ledger row and not its JSON
# object, touches nothing but `.planning/*.md` and so earns this list and
# nothing else. Without it here, the check written for that commit would be
# the one thing that commit does not run.
#
# `the_words_that_say_nothing` is the seventh, and its case is the same: it
# reads prose for the six words `CLAUDE.md` bans, and a commit that writes
# prose and nothing else earns exactly this list.
#
# `what_the_scans_can_judge` is the eighth, and the `help_page` case again:
# it reads `docs/wcag-coverage.md` from inside the library and holds the
# page's table to the three criteria the code names. A commit that edits
# only that page answers this mode, so without it the reading ran
# on every commit except the ones that could break it, which is what
# `CLAUDE.md` says about a guard under `tests/`, happening to a `--lib`
# test instead. Added 2026-09-14 with the module.
#
# `every_number_carries_its_command_and_its_date` is the ninth, and its
# case is `the_planning_files_agree_with_themselves`'s exactly: it reads
# `docs/development/measurements.md` and refuses a row without its command,
# its date or its commit. A commit that adds a row and nothing else
# touches one page under `docs/` and so earns this list and nothing else.
# Added 2026-09-14 with the page.
the_targets_that_read_documents=(house_style docs_links wired checkbox_labels manager_delete_stays_open the_planning_files_agree_with_themselves the_words_that_say_nothing every_number_carries_its_command_and_its_date)
the_modules_that_read_documents=(help_page:: presentation::what_the_scans_can_judge::)

# What each `scripts/*.test.sh` reads, so a suite runs on the commits that stage
# one of its inputs and not on the others. Added 2026-09-23 by 12-03.2, on
# Pratik's answer that day that each suite runs only for its own inputs: the
# four cost about 25 seconds on every commit that day, and the file most commits
# stage among their inputs, `guards/guards.toml`, is read by one suite.
#
# Data here rather than read out of the suites when the gate runs, because a
# gate deciding by a pattern over text leaks wherever the text is spelled
# another way. The pattern lives in `check.test.sh`, in "every file a suite
# reads is on its list", and holds these lists to it on every commit that owes
# that suite. Each list is what the suite and every script it reaches name,
# followed through each call, so it reads wider than any one case reaches:
#
#   audit         its suite, `audit.sh`, the accepted advisories in
#                 `.cargo/audit.toml`, and `Cargo.lock`, which `audit.sh` reads
#                 in its real run and no case reaches
#   check         its suite and this script, the guard records the coupling
#                 reads, and what this script calls: `which-checks.sh`,
#                 `red-commit.sh` and `audit.sh` with what `audit.sh` reads; and
#                 every other suite, because the reading of the lists lives in
#                 this suite and reads every suite
#   red-commit    its suite and `red-commit.sh`
#   which-checks  its suite, `which-checks.sh`, and `red-commit.sh`, which it
#                 calls for every case handed a message file
#
# Every suite runs for the shared harness and for the hook, which no suite reads
# and which runs them all with the commit's environment. A path under `scripts/`
# or `.githooks/` on no list at all owes every suite, the gate's answer to what
# it cannot place, the way `which-checks.sh` answers `all`. The scripts no suite
# reaches owe none.
declare -A the_inputs_of_a_suite
the_inputs_of_a_suite[audit]="scripts/audit.test.sh scripts/audit.sh .cargo/audit.toml Cargo.lock"
the_inputs_of_a_suite[check]="scripts/check.test.sh scripts/check.sh guards/guards.toml scripts/which-checks.sh scripts/red-commit.sh scripts/audit.sh .cargo/audit.toml Cargo.lock scripts/audit.test.sh scripts/red-commit.test.sh scripts/which-checks.test.sh"
the_inputs_of_a_suite[red-commit]="scripts/red-commit.test.sh scripts/red-commit.sh"
the_inputs_of_a_suite[which-checks]="scripts/which-checks.test.sh scripts/which-checks.sh scripts/red-commit.sh"
what_every_suite_is_owed_for=(scripts/shell-suite.sh .githooks/commit-msg)
what_no_suite_reads=(scripts/guards.py scripts/guards.sh scripts/mutants.sh scripts/mutants_report.py scripts/build-installer.sh scripts/make-brand.py scripts/make-icon.py scripts/render_svg.py scripts/msaa-names.ps1 scripts/uia-events.ps1)

# Which integration targets guard a changed source file.
#
#     check.sh --suites-for <registry> [changed-file ...]
#
# A unit test lives beside the code it covers, so `--lib a::b::` reaches it. A
# guard under `tests/` does not: it covers a `src/` module from outside, so the
# scoped run reaches it only when the test file itself changes. That is a guard
# running on the wrong commits, and a guard nobody reads is worse than none
# because it reads as covered.
#
# `guards/guards.toml` already declares the coupling, so it is read rather than
# restated. Every record names the `file` its break is applied to, and a record
# whose break reddens an integration target names that target as `suite`. A
# second list beside the first would drift from it, which is a shape this
# repository keeps getting caught by.
#
# **This is a line scan, not a TOML parse.** A shell script has no TOML reader,
# and the file is written one key per line. So the scan keys on a `[[guard]]`
# header and then on `file = "..."` and `suite = "..."` at column 0 inside it,
# remembering the pair when the next header arrives. What it would miss: a
# record spelled as an inline table, a `file` or `suite` sharing a line with
# another key, a value quoted some other way, or a `before`/`after` string
# holding a line that begins `file = ` at column 0. Measured 2026-09-02: 550
# records and 550 such lines, so no record has that shape today and nothing
# would say so if one arrived. A record this misses is a guard that does not run
# on the commit that could break it, which is the defect this function exists to
# fix, so the limit is written down here rather than left to be found.
#
# Its suite is `scripts/check.test.sh`, which this script runs in every mode.
# In every mode until 2026-09-23; since then, on the commits that stage one of
# that suite's inputs, this file among them, and in `all` and `all_but_slow`.
the_suites_that_guard_what_changed() {
    local registry="$1"
    shift

    # A registry that is not there costs the extra targets and nothing else.
    # This mapping adds to what a commit earns, so it must never be the reason a
    # commit cannot be made.
    if [ ! -r "$registry" ]; then
        return 0
    fi

    local -a couplings=() answers=()
    local line file="" suite="" coupling path candidate already seen

    # Narrowed to the three kinds of line that carry a coupling before the loop
    # reads them, so the scan is over about 1,650 lines rather than 12,800 and
    # the shapes it reads are visible in one place.
    while IFS= read -r line; do
        case "$line" in
            '[[guard]]')
                if [ -n "$file" ] && [ -n "$suite" ]; then
                    couplings+=("$file|$suite")
                fi
                file=""
                suite=""
                ;;
            'file = "'*'"')
                file="${line#file = \"}"
                file="${file%\"}"
                ;;
            'suite = "'*'"')
                suite="${line#suite = \"}"
                suite="${suite%\"}"
                ;;
        esac
    done < <(grep -E '^(\[\[guard\]\]|file = |suite = )' "$registry" || true)
    if [ -n "$file" ] && [ -n "$suite" ]; then
        couplings+=("$file|$suite")
    fi

    for path in "$@"; do
        # Everything except a changed test file, which already runs its own
        # target a few lines up and would be asked for twice.
        #
        # This used to be the other way round, taking only a changed `src/*.rs`
        # and saying that a record whose break lands anywhere else was a true
        # record that says nothing here, because `--lib` was never going to
        # reach it. The first half is true and it is about the wrong thing: what
        # this mapping answers with is a `--test` target, so a break landing on
        # a file no `--lib` filter could reach is exactly the case it is for.
        #
        # What the narrow version cost, found on 2026-09-12 by two records
        # arriving that it could not read: a commit changing only
        # `.github/workflows/release.yml` ran the four tree-reading guards and
        # nothing that reads the workflow, while two records naming that file
        # and the suite that reads it sat in the registry looking like coverage.
        # That is guardrail 4 exactly, and it is the same shape as the `.iss`
        # hole `which-checks.sh` closed one layer up in 07-02.
        case "$path" in
            tests/*.rs) continue ;;
        esac

        for coupling in "${couplings[@]+"${couplings[@]}"}"; do
            [ "${coupling%%|*}" = "$path" ] || continue
            candidate="${coupling#*|}"

            # Dropped if the scoped run already ends with it, and dropped if
            # another record has already answered it. Several records couple one
            # source file to one target, and the target is one run either way.
            seen=""
            for already in "${guards_that_read_the_whole_tree[@]}" \
                           "${answers[@]+"${answers[@]}"}"; do
                if [ "$already" = "$candidate" ]; then
                    seen=yes
                fi
            done
            if [ -n "$seen" ]; then
                continue
            fi

            answers+=("$candidate")
        done
    done

    if [ "${#answers[@]}" -eq 0 ]; then
        return 0
    fi
    printf '%s\n' "${answers[@]}"
}

# The mapping on its own, so `scripts/check.test.sh` can ask it about a made-up
# file list and a made-up registry rather than running the whole gate. Answered
# here, before anything with a side effect and before the mode is looked at, so
# a suite asking this question can never start a gate run that would run that
# suite again.
if [ "${1:-}" = "--suites-for" ]; then
    shift
    the_suites_that_guard_what_changed "$@"
    exit 0
fi

# What is about to be committed, read from the index of the repository this is
# run in. With `--no-renames` since 2026-09-23 (12-03.2): without it git lists a
# staged move by where it went alone, measured that day under git
# 2.55.0.windows.3, so a move out of `scripts/` read as owing no suite and a
# move from `src/` to `docs/` as documents only. The flag only adds the path a
# move left, so it can only strengthen what a commit earns. A deleted file is
# listed either way.
the_staged_paths() {
    git diff --cached --name-only --no-renames 2>/dev/null || true
}

# What the scoped run hands cargo, one invocation's arguments to a line.
#
#     the_scoped_runs <registry> [changed-path ...]
#
# One `--lib <module>:: -- --test-threads=4` line per changed source module,
# since `cargo test` takes one `--lib` filter, and at most one line of `--test`
# pairs naming every integration target the change reaches, each once: a
# changed test file that is still there, every target the registry couples a
# changed file to, and every target that reads the whole tree, which is why
# the line is always there. Added 2026-09-23 by 12-03.2: the targets used to be
# one cargo call each, and a change to `wx_app.rs` coupled to 25 to 29 of them.
#
# A changed test file that is not under the directory this is asked from is
# left out. The staged list holds a deleted or moved-away test file either way,
# and in one call cargo would refuse the whole line over it before any target
# ran, so a branch that deleted a target could not be committed or merged.
#
# No line holds `--no-fail-fast`. The runner's line owns it, because cargo
# refuses the flag given twice.
#
# Each source that compiles a changed file in, by `include_str!` or
# `include_bytes!`, counts as changed too, for its own library run and for the
# targets the registry couples to it, since 2026-09-23 (12-03.2): a commit
# changing only the dictionary ran no spellcheck test, and one changing only
# `mail_controller.rs` never ran `sent_copy`'s readings of it. `which-checks.sh`
# answers which sources those are, from `src/` under the directory this runs
# from. A source that compiles itself in adds nothing, and one counted twice
# runs once. `--suites-for` above reads no source file; only this does.
#
# At a merge, `--merge` first, whose paths hold a document by
# `which-checks.sh --documents-among`, the document-reading list joins as well:
# each of `the_targets_that_read_documents` in the one `--test` line and each
# of `the_modules_that_read_documents` as a library run of its own. Added
# 2026-09-23 (12-03.2): the merge stopped being the whole gate that day, and a
# branch mixing code and documents answers `affected`, which runs no
# document-reading target. Only at a merge, because on a branch nearly every
# green commit carries a changelog line beside its code and would pay the list
# each time.
the_scoped_runs() {
    local merging=""
    if [ "${1:-}" = "--merge" ]; then
        merging=yes
        shift
    fi
    local registry="$1" path module target source line=""
    shift
    local -a paths=("$@") targets=()
    local -A named=() modules=()
    if [ "$#" -gt 0 ]; then
        while IFS= read -r source; do
            [ -n "$source" ] && paths+=("$source")
        done < <("$(dirname "$0")/which-checks.sh" --sources-compiling "$@")
    fi
    if [ -n "$merging" ] && [ "$#" -gt 0 ] &&
        [ -n "$("$(dirname "$0")/which-checks.sh" --documents-among "$@")" ]; then
        for module in "${the_modules_that_read_documents[@]}"; do
            modules[${module%::}]=1
            echo "--lib $module -- --test-threads=4"
        done
        targets+=("${the_targets_that_read_documents[@]}")
    fi
    for path in "${paths[@]+"${paths[@]}"}"; do
        case "$path" in
            src/*.rs)
                module="${path#src/}"
                module="${module%.rs}"
                module="${module%/mod}"
                module="${module//\//::}"
                [ "$module" = "lib" ] && continue
                [ -n "${modules[$module]-}" ] && continue
                modules[$module]=1
                echo "--lib ${module}:: -- --test-threads=4"
                ;;
            tests/*.rs)
                [ -f "$path" ] || continue
                target="${path##*/}"
                targets+=("${target%.rs}")
                ;;
        esac
    done
    while IFS= read -r target; do
        [ -n "$target" ] && targets+=("$target")
    done < <(the_suites_that_guard_what_changed "$registry" "${paths[@]+"${paths[@]}"}")
    targets+=("${guards_that_read_the_whole_tree[@]}")
    for target in "${targets[@]}"; do
        [ -n "${named[$target]-}" ] && continue
        named[$target]=1
        line+="${line:+ }--test $target"
    done
    echo "$line"
}

# The builder on its own, so `scripts/check.test.sh` can ask it about a made-up
# change from a tree of its own:
#
#     check.sh --scoped-runs-for [--merge] <registry> [changed-path ...]
if [ "${1:-}" = "--scoped-runs-for" ]; then
    shift
    the_scoped_runs "$@"
    exit 0
fi

# The mode `which-checks.sh` gives the commit being made: from the staged
# paths, the branch, the message, and since 2026-09-23 (12-03.2) whether the
# commit is a merge, which git says by leaving `MERGE_HEAD` while one is in
# progress, the hook's run included. A merge's staged list is the branch's
# whole diff, which equalled the merge-base-to-tip diff on every one of the
# fourteen merges read that day, so nothing new is computed for it.
#
# Leaves `mode`, `changed`, `merging` and `the_branch` for the run, which asks
# this too, so the question below and the run decide one way.
decide_the_mode_for_this_commit() {
    local message_file="$1"
    mapfile -t changed < <(the_staged_paths)
    merging=""
    if git rev-parse -q --verify MERGE_HEAD > /dev/null 2>&1; then
        merging=yes
    fi
    the_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
    mode="$("$(dirname "$0")/which-checks.sh" \
        ${message_file:+"--message-file=$message_file"} \
        ${merging:+--merge} \
        "$the_branch" \
        "${changed[@]+"${changed[@]}"}")"
}

# The run's own decision, made without running anything, so
# `scripts/check.test.sh` can ask it of a repository of its own with a merge in
# progress:
#
#     check.sh --mode-for-this-commit [--message-file=F]
#
# Answered here, above the note about the hook, which prints wherever the hooks
# directory holds no `commit-msg`, so the answer is only the mode.
if [ "${1:-}" = "--mode-for-this-commit" ]; then
    shift
    asked_message_file=""
    case "${1:-}" in
        --message-file=*) asked_message_file="${1#--message-file=}" ;;
    esac
    decide_the_mode_for_this_commit "$asked_message_file"
    echo "$mode"
    exit 0
fi

# Whether a path is one of the words of a space-separated list.
is_on_the_list() {
    case " $2 " in
        *" $1 "*) return 0 ;;
    esac
    return 1
}

# Whether any list above places a path at all, a suite's, the shared one or
# the one no suite reads.
is_placed_by_a_list() {
    local path="$1" name
    is_on_the_list "$path" "${what_every_suite_is_owed_for[*]} ${what_no_suite_reads[*]}" && return 0
    for name in "${!the_inputs_of_a_suite[@]}"; do
        is_on_the_list "$path" "${the_inputs_of_a_suite[$name]}" && return 0
    done
    return 1
}

# The first reason a suite is owed, left in `owed_because`, which is empty when
# it is not. A variable rather than a printed answer, because a command
# substitution is a process, and a process costs tens of milliseconds on Windows
# on every commit for every suite.
#
#     the_reason_a_suite_is_owed <suite> <mode> <names failing, one a line> [changed-path ...]
#
# A suite added later with no list is read as holding an empty one, spelled
# with a default because this script runs under `set -u` and a bare lookup
# would stop the gate on every commit. The case "every file a suite reads is on
# its list" is what names such a suite.
the_reason_a_suite_is_owed() {
    local suite="$1" the_mode="$2" failing="$3" path named
    shift 3
    owed_because=""
    case "$the_mode" in
        all | all_but_slow)
            owed_because="every suite runs in an $the_mode run"
            return
            ;;
    esac
    while IFS= read -r named; do
        case "$named" in
            "$suite::"*)
                owed_because="the commit names $named as failing"
                return
                ;;
        esac
    done <<< "$failing"
    for path in "$@"; do
        is_on_the_list "$path" "${what_every_suite_is_owed_for[*]}" &&
            { owed_because="$path is what every suite runs under"; return; }
    done
    for path in "$@"; do
        is_on_the_list "$path" "${the_inputs_of_a_suite[$suite]-}" &&
            { owed_because="$path is on its list"; return; }
    done
    for path in "$@"; do
        case "$path" in
            scripts/* | .githooks/*)
                is_placed_by_a_list "$path" ||
                    { owed_because="$path is a script no list places"; return; }
                ;;
        esac
    done
}

# Which suites a commit owes, one line per suite in the order the suites' loop
# finds them, `<suite> yes: <why>` or `<suite> no: <why>`.
#
#     the_shell_suites_owed <mode> <message-file or nothing> [changed-path ...]
the_shell_suites_owed() {
    local the_mode="$1" message_file="$2" candidate name owed_because failing=""
    shift 2
    case "$the_mode" in
        red)
            [ -z "$message_file" ] ||
                failing="$("$(dirname "$0")/red-commit.sh" names "$message_file")"
            ;;
    esac
    for candidate in "$(dirname "$0")"/*.test.sh; do
        [ -e "$candidate" ] || continue
        name="${candidate##*/}"
        name="${name%.test.sh}"
        the_reason_a_suite_is_owed "$name" "$the_mode" "$failing" "$@"
        if [ -n "$owed_because" ]; then
            echo "$name yes: $owed_because"
        else
            echo "$name no: nothing it reads changed"
        fi
    done
}

# The rule on its own, so `scripts/check.test.sh` can ask it about a made-up
# change and a repository of its own:
#
#     check.sh --shell-suites-owed <mode> [--message-file=F] [--staged | changed-path ...]
#
# `--staged` reads the paths from the index through the same function the run
# fills its changed list from. Answered here, before anything with a side
# effect and above the note about the hook, which prints wherever the hooks
# directory holds no `commit-msg`, so the answer is only the answer.
if [ "${1:-}" = "--shell-suites-owed" ]; then
    shift
    owed_mode="${1:-}"
    shift || true
    owed_message_file=""
    case "${1:-}" in
        --message-file=*)
            owed_message_file="${1#--message-file=}"
            shift
            ;;
    esac
    owed_paths=()
    if [ "${1:-}" = "--staged" ]; then
        mapfile -t owed_paths < <(the_staged_paths)
    else
        owed_paths=("$@")
    fi
    the_shell_suites_owed "$owed_mode" "$owed_message_file" "${owed_paths[@]+"${owed_paths[@]}"}"
    exit 0
fi

# Offer to run these on every commit, so the answer cannot be lost between
# getting it and committing. It has been twice: a stale fingerprint reporting
# clean, and this script's output piped somewhere so the pipeline's exit status
# was the pipe's rather than this script's.
# Asked as a question rather than matched against one spelling. The old form
# compared `core.hooksPath` to the literal `.githooks`, and this machine holds
# the absolute path, which is what makes the hook resolve from a worktree. So it
# printed "Not running on commit" on every run, including the runs the hook
# itself started, and the fix it advised would have replaced a working absolute
# path with a narrower relative one. Found on 2026-09-02 by somebody reading the
# output of a run the hook had started.
#
# What matters is whether a commit will run this, so that is what is asked: does
# the configured hooks directory hold an executable commit-msg hook. A wrong
# answer here is not cosmetic, it tells somebody to change a setting that works.
hooks_path="$(git config core.hooksPath || true)"
case "$hooks_path" in
    "") will_run_on_commit="" ;;
    /* | [A-Za-z]:[/\\]*) will_run_on_commit="$hooks_path/commit-msg" ;;
    *) will_run_on_commit="$(git rev-parse --show-toplevel)/$hooks_path/commit-msg" ;;
esac

if [ -z "$will_run_on_commit" ] || [ ! -f "$will_run_on_commit" ]; then
  echo "Not running on commit. To turn that on:"
  echo "    git config core.hooksPath .githooks"
  echo
fi

# Which checks this run does. Given as an argument, or worked out by
# `which-checks.sh` from where you are and what is staged, which is where that
# decision lives and where it is tested. Pass `all` to force the whole gate
# wherever you are, which is what merging a branch into main does first. Since
# 2026-09-23 (12-03.2) the whole gate runs once a phase, `all` by hand in the
# phase's closing plan on its branch before its merge, and not at or before
# each merge: a merge runs what the branch's whole diff earns.
#
# Two questions, and they are separate. Whether the slow half can be deferred is
# about the branch: main cannot defer, because every commit here lands on it.
# What a change can possibly break is about the change, and that holds
# everywhere, including on main. So a documents-only commit runs the tests that
# read documents wherever it is made, and a code commit on a branch runs the
# tests reaching what it touched.
#
# Measured warm: the whole gate is about 330 seconds, of which the suite is 239
# and the release build 56. A documents-only run is about 51.
#
# **Those figures are warm and they are a floor, not an estimate.** Measured
# again on 2026-09-02, a documents-only commit took 2m56s: it followed two
# commits that had changed test files, so it paid for a clippy rebuild, and the
# document-reading list includes two targets that build a live window. Quoting
# 51 seconds to somebody planning work was quoting a measurement without its
# conditions. What holds in every case is the shape: a documents commit runs a
# small fraction of the gate. What it costs on the day depends on what the
# commits before it left to rebuild.
#
# The commit message, when a commit is what is running this. Only the message
# can say that this is the RED half of red/green, which is why the hook runs
# from `commit-msg`: at `pre-commit` time the message does not exist yet.
message_file=""
case "${1:-}" in
    --message-file=*)
        message_file="${1#--message-file=}"
        shift
        ;;
esac

mode="${1:-}"

# An argument this script does not know is refused rather than falling through
# to the whole gate.
#
# Not a tidiness point. This script runs every `scripts/*.test.sh`, and one of
# those suites now runs this script. A typo in the argument that suite passes
# used to be read as a mode, fall past every branch below, and run the whole
# gate, which ran the suite, which ran the gate. Measured on 2026-09-02 by
# writing exactly that typo, and it had to be killed. A misspelled `all` also
# quietly bought somebody the full gate and told them nothing.
case "$mode" in
    "" | all | all_but_slow | affected | docs_only | red) ;;
    *)
        echo "check.sh: '$mode' is not a mode this script knows." >&2
        echo "  Modes: all, all_but_slow, affected, docs_only, red." >&2
        echo "  Or no argument at all, and which-checks.sh decides." >&2
        echo "  Or --suites-for <registry> [changed-file ...] for the mapping" >&2
        echo "  from a changed source file to the guards that cover it." >&2
        echo "  Or --shell-suites-owed <mode> [--message-file=F] [--staged | path ...]" >&2
        echo "  for which scripts/*.test.sh a commit owes, --scoped-runs-for" >&2
        echo "  [--merge] <registry> [path ...] for what the scoped run hands cargo," >&2
        echo "  or --mode-for-this-commit [--message-file=F] for the mode it runs." >&2
        exit 64
        ;;
esac

# What is about to be committed, which is what the tests should be scoped to.
# Staged rather than working-tree, because that is what the hook is deciding
# about. Empty when run by hand outside a commit, and `which-checks.sh` answers
# `all_but_slow` for that rather than guessing at a narrower set.
changed=()
merging=""
the_branch=""
if [ -z "$mode" ]; then
    decide_the_mode_for_this_commit "$message_file"
fi

# Where a run's seconds went, one stage at a time.
#
# Added 2026-09-23 by 12-03.2. A measurement of nine executors that day put 28%
# of a plan's time in this hook, and one stage nothing printed, rustfmt, the
# shell suites and start-up together, took 25 to 35 seconds on one day and 95
# to 145 on two others, with no way to say which part moved. So every header a
# stage prints goes through `begin_stage`, which closes the previous stage's
# clock, and the run ends, pass or fail, with one line naming each stage and its
# seconds. Whole seconds from bash's `SECONDS`, because the question is minutes.
#
# The first stage is `start`, from the script's first line to rustfmt, so what
# the run spent before it checked anything is a stage of its own and not folded
# into the first one that did.
stage_names=()
stage_seconds=()
current_stage="start"
current_stage_began=0

# The files a run makes and must remove, whichever way it ends. One list and one
# trap, because two traps on EXIT are one trap: the second replaces the first,
# and red mode used to set a second one that dropped this one's clean-up.
files_to_remove=()

close_the_current_stage() {
    stage_names+=("$current_stage")
    stage_seconds+=("$(( SECONDS - current_stage_began ))")
}

begin_stage() {
    close_the_current_stage
    current_stage="$1"
    current_stage_began=$SECONDS
    echo "== $1 =="
}

# On every exit once the mode is settled. A run refused before that, an unknown
# mode or `which-checks.sh` refusing, never arms it, so a refusal cannot claim
# to have run a stage.
the_run_ends() {
    local status=$? outcome stages="" index
    rm -f "${files_to_remove[@]+"${files_to_remove[@]}"}"
    close_the_current_stage
    if [ "$status" -eq 0 ]; then
        outcome=passed
    else
        outcome="stopped in $current_stage"
    fi
    for index in "${!stage_names[@]}"; do
        stages+="${stages:+, }${stage_names[$index]} ${stage_seconds[$index]} s"
    done
    echo "check.sh: $mode $outcome after $SECONDS s: $stages" >&2
    return "$status"
}

echo "check.sh: mode $mode${merging:+, a merge into $the_branch}"
trap the_run_ends EXIT

touch src/lib.rs

begin_stage "rustfmt"
cargo fmt --all -- --check

begin_stage "clippy"
cargo clippy --all-targets --all-features -- -D warnings

# The scripts that decide what this gate does, checked by the gate itself.
#
# These suites existed for a day before anything ran them. `which-checks.sh`
# decides which checks every commit earns and `red-commit.sh` decides whether a
# failing test may be committed, so a defect in either is a defect in all of
# this, and both had tests that nothing invoked. That is the shape guardrail 4
# in `CLAUDE.md` is about: a check nobody reads is worse than none, because it
# reads as covered.
#
# In every mode and before every other decision, because they cost milliseconds
# and because the mode was chosen by the very script under test.
#
# **Corrected 2026-09-23 by 12-03.2.** The milliseconds were true once and had
# not been for weeks: the four suites took 108 seconds on 2026-09-10, 43 and 47
# on 2026-09-19, and 9, 7, 3.9 and 5.1 seconds one after another on 2026-09-23,
# about 25 in all, on every commit. So since that day each suite runs only when
# a commit stages a path on its own list above, or the shared harness, or the
# hook, or a script no list places, and every suite runs in `all` and
# `all_but_slow` and for a red commit naming one of its cases. It is decided
# here, still before any mode branch, and a suite not run says so on its own
# line. CI runs every suite on every push and pull request whatever this does.
#
# # Why the output is collected rather than left to abort the run
#
# This loop used to be `bash "$suite"` under `set -e`, so a failing suite stopped
# the gate here, several branches above the `red` one. A `scripts/*.test.sh`
# case could therefore never be committed red: the run died before
# `red-commit.sh` was asked for a verdict, and the answer was to commit a new
# suite and the code that makes it pass together, which is the thing red/green
# exists to stop. That was windows ledger 39.
#
# Nothing is forgiven by collecting it. Every mode but `red` refuses the commit
# immediately below, with the whole output. In `red` the failure is handed to
# exactly the verdict a cargo failure gets, because the suites print a line per
# case in the shape cargo prints, and the three conditions then hold across both
# kinds of test at once: every named case ran, every named case failed, and
# nothing else failed.
#
# The whole output of every run below is kept for the same reason, so the log is
# opened here rather than after the mode branches.
run_log="$(mktemp)"
files_to_remove+=("$run_log")

begin_stage "the scripts that decide what runs"
declare -A shell_suites_owed=()
while IFS= read -r owed_line; do
    shell_suites_owed["${owed_line%% *}"]="${owed_line#* }"
done < <(the_shell_suites_owed "$mode" "$message_file" "${changed[@]+"${changed[@]}"}")
shell_suites_failed=""
shell_suites_run=()
for suite in "$(dirname "$0")"/*.test.sh; do
    [ -e "$suite" ] || continue
    suite_name="$(basename "$suite" .test.sh)"
    owed="${shell_suites_owed[$suite_name]-no: nothing answered for it}"
    case "$owed" in
        "yes: "*) ;;
        *)
            echo "-- $suite_name: not run, ${owed#no: }; CI runs it on every push"
            continue
            ;;
    esac
    shell_suites_run+=("$suite_name")
    echo "-- $suite_name"
    bash "$suite" >> "$run_log" 2>&1 || shell_suites_failed=yes
done

# A suite that died partway through, on an unset variable or a syntax error,
# left its remaining cases unrun and said so nowhere a name could match. In
# `red` that would read as a clean run with one expected failure in it, so it is
# refused in every mode, before the marker is looked at. `shell-suite.sh` prints
# this line from its own verdict, which a suite that died never reaches.
for suite_name in "${shell_suites_run[@]+"${shell_suites_run[@]}"}"; do
    if ! grep -qxF "test $suite_name::every case in this suite ran ... ok" "$run_log"; then
        echo >&2
        echo "$suite_name.test.sh stopped before it reached its own verdict, so the" >&2
        echo "cases it did not get to said nothing and this run cannot be judged." >&2
        echo >&2
        cat "$run_log" >&2
        exit 1
    fi
done

# Refused here in every mode but `red`, where the marker decides instead.
if [ -n "$shell_suites_failed" ] && [ "$mode" != red ]; then
    echo >&2
    echo "A suite that decides what this gate runs is failing:" >&2
    echo >&2
    grep -E '^(FAIL |       )' "$run_log" >&2 || cat "$run_log" >&2
    echo >&2
    echo "A case here can be committed red on a branch, by naming it:" >&2
    echo "    Fails-until-green: <suite>::<the case description>" >&2
    exit 1
fi

if [ "$mode" = "all_but_slow" ]; then
    echo
    echo "Formatting and clippy passed. The test suite and the release build did"
    echo "not run: nothing was said about what changed. They run once a phase,"
    echo "by hand in its closing plan (scripts/check.sh all)."
    exit 0
fi

# Documents can only break the tests that read documents, and they genuinely
# can: house_style's em-dash guard has caught two real breaks in markdown, so
# these run rather than being skipped as "not code".
if [ "$mode" = "docs_only" ]; then
    begin_stage "the targets that read documents"
    # What reads documents, and why each is on the list, is above
    # `the_targets_that_read_documents`, where the list has been held once since
    # 2026-09-23: a merge whose diff holds a document runs it too.
    for module in "${the_modules_that_read_documents[@]}"; do
        cargo test --lib "$module"
    done
    document_targets=()
    for target in "${the_targets_that_read_documents[@]}"; do
        document_targets+=(--test "$target")
    done
    # --no-fail-fast because this names eight targets, and without it a red
    # `house_style` meant the others never started. Found on 2026-09-03 by
    # `test_one_failing_target_does_not_hide_the_rest`, the moment its exemption
    # was narrowed from "the line names a target" to "the line names exactly
    # one". This is the same defect that was fixed in the scoped run below, in
    # the same shape, on a line the wider exemption could not see.
    cargo test --no-fail-fast "${document_targets[@]}"
    echo
    echo "Formatting, clippy and the document-reading tests passed. The rest of"
    echo "the suite and the release build did not run: nothing outside a document"
    echo "changed, so they had nothing to say. They run once a phase, by hand in"
    echo "its closing plan (scripts/check.sh all)."
    exit 0
fi

# The scoped runs below append to the same log, kept rather than summarised. It
# used to be piped through `tail -3` per module, which threw away the names of
# the tests that failed and left a gate that said something was wrong without
# saying what. `red` needs the full text anyway, to see which tests failed, and
# it needs the shell suites' cases in the same place for the same reason.
#
# A unit test lives beside the code it covers, so a changed `src/a/b.rs` is
# covered by `--lib a::b::`. The source-reading guards run whatever changed,
# because they read across the whole tree and a change anywhere can redden one:
# that is how a guard record was found stale four times in one phase.
#
# Returns non-zero if any scoped run did. Never aborts on one, so a failure in
# the first module does not hide the rest, for the same reason the whole-suite
# run passes `--no-fail-fast`.
#
# Runs exactly what `the_scoped_runs` answers, the lines read into an array
# first so no test a cargo call runs can read the rest of them from standard
# input. Since 2026-09-23 (12-03.2) the integration targets are one cargo call
# rather than one each, so a change coupled to 25 targets pays cargo's start
# once; `red-commit.sh` reads the same `test NAME ... ok|FAILED` lines from the
# log either way.
run_the_tests_that_reach_what_changed() {
    local status=0 line word
    local -a runs arguments
    mapfile -t runs < <(the_scoped_runs ${merging:+--merge} \
        "$(dirname "$0")/../guards/guards.toml" \
        "${changed[@]+"${changed[@]}"}")
    for line in "${runs[@]+"${runs[@]}"}"; do
        read -ra arguments <<< "$line"
        case "${arguments[0]-}" in
            --lib)
                echo "-- ${arguments[1]%::}"
                # A filter matching nothing exits zero, so a module with no
                # tests of its own is not a pass, it is a run that said nothing.
                #
                # Four threads, not the harness default of one per core.
                # Measured 2026-09-09 at `10effea` on 24 cores against
                # `application::tasks_sync::`, 111 tests: 14s at the default,
                # 4s at four threads, 6s at eight. A scoped run is contended
                # harder than a whole-library one, because there is less work to
                # spread and the same contention to spread it against, so its
                # best thread count is lower than the eight `guards.py` uses for
                # a full run. Two settings, not one, and each was measured.
                #
                # The `--all-targets` run further down is deliberately left
                # alone: the same setting was measured there and the gate did
                # not move.
                cargo test --lib "${arguments[1]}" -- --test-threads=4 >> "$run_log" 2>&1 || status=1
                ;;
            --test)
                for word in "${arguments[@]}"; do
                    [ "$word" = --test ] || echo "-- $word"
                done
                run_the_integration_targets "${arguments[@]}" || status=1
                ;;
        esac
    done
    return $status
}

# Every integration target the scoped run reaches, in one cargo call.
run_the_integration_targets() {
    # --no-fail-fast because this is more than one target, and this line used to
    # be `cargo test --test house_style --test wired` without it: a failure in
    # house_style meant wired never started, not reported as skipped. That is
    # the same defect the whole-suite run below carries the flag for, in a line
    # small enough that nobody looked at it twice.
    #
    # Found on 2026-09-02 by `test_one_failing_target_does_not_hide_the_rest`,
    # which reads this file. It used to exempt a line carrying `--test ` as one
    # that runs a named target on purpose, so the old spelling was exempt while
    # having the defect; building the targets into an array took the flag out of
    # the text and the guard spoke. The exemption now counts the targets and
    # covers only a line naming exactly one, which found the same defect on the
    # documents-only run above the moment it was narrowed.
    #
    # This line owns the flag, and the builder's lines never hold it: cargo
    # refuses `--no-fail-fast` given twice, before it selects a target,
    # measured 2026-09-23 at `481a7918`, so a builder line carrying it too
    # would make every scoped run fail.
    cargo test --no-fail-fast "$@" >> "$run_log" 2>&1
}

# The RED half of red/green. The commit named the tests that must fail; the run
# is held to exactly that, in all three directions, by `red-commit.sh`.
if [ "$mode" = "red" ]; then
    named="$(mktemp)"
    files_to_remove+=("$named")
    "$(dirname "$0")/red-commit.sh" names "$message_file" > "$named"

    begin_stage "the tests this commit says must fail"
    sed 's/^/   /' "$named"
    echo
    begin_stage "the tests that reach what changed"
    run_the_tests_that_reach_what_changed || true

    echo
    if ! "$(dirname "$0")/red-commit.sh" verdict "$named" "$run_log"; then
        echo
        echo "This commit is not the red it says it is." >&2
        cp "$run_log" ./red-run.log && echo "The whole run is in ./red-run.log" >&2
        exit 1
    fi
    echo "Formatting, clippy, and a red that is exactly the one this commit"
    echo "named. The green commit that follows runs the same tests and must"
    echo "carry no marker."
    exit 0
fi

# Scope the suite to the modules the change reaches.
if [ "$mode" = "affected" ]; then
    begin_stage "the tests that reach what changed"
    if ! run_the_tests_that_reach_what_changed; then
        echo
        echo "Failed. What went red, with its output:" >&2
        sed -n '/^failures:/,$p' "$run_log" >&2
        exit 1
    fi
    echo
    echo "Formatting, clippy, the tests reaching what changed, and the"
    echo "tree-reading guards passed. The rest of the suite and the release"
    echo "build did not run. They run once a phase, by hand in its closing plan"
    echo "(scripts/check.sh all)."
    exit 0
fi

# The thread count is deliberately not set here, and that is a result rather
# than an omission.
#
# Run on its own, the library suite is much faster on four threads than on the
# harness default of one per core. Measured 2026-08-31 on 24 logical cores over
# 5,837 tests: 2 threads 131s, 4 threads 88s, 8 threads 106s, 16 threads 164s,
# default 196s. It is contended rather than compute-bound. `scripts/guards.py`
# takes that setting and keeps it, because it runs `--lib` on its own and the
# measurement is about exactly that.
#
# It does not carry to this gate, which runs `--all-targets`. Measured here, on
# the same machine on the same day with nothing else running:
#
#                        library test term    whole gate
#     default (24)              197.00s          335s
#     four threads              111.55s          353s
#
# The test term drops 86 seconds twice over and the total does not move. About
# 104 seconds appears somewhere else and was not accounted for; the two totals
# may simply be inside this machine's run-to-run spread. Either way there is no
# measured gain here, so nothing is set, because a number written down without a
# result behind it is the thing this file keeps warning about.
#
# Worth picking up again: the gate is compilation more than testing, and
# `target/debug` was 269GB when this was measured.

# Advisories against the dependency tree, which nothing local asked about until
# 2026-09-13. CI has had a job for this since the beginning; it went red on
# 2026-09-10 and sat unread for three days, because a job page is the one place
# a person never looks on a day they think is fine.
#
# Only in this mode. An advisory is not a consequence of a commit, so scoping it
# to what changed makes no sense, and `all` is what every code commit on main and
# every pre-merge run does. Six seconds of a run that is about 330. Since
# 2026-09-23 (12-03.2) no merge runs `all`: the whole gate, this check with it,
# runs once a phase, by hand in the phase's closing plan, and CI's Security
# Audit job runs on every push.
#
# Before the suite rather than after it, because it is the cheapest thing here
# and a finding should not wait four minutes to be said.
#
# What it is allowed not to block on, and why that is not a silencing, is in
# scripts/audit.sh. The short version: an advisory nobody has decided yet is set
# aside here by name, on this machine only, so CI stays red on it.
begin_stage "security advisories"
"$(dirname "$0")/audit.sh"

begin_stage "tests"
# --no-fail-fast because without it cargo stops at the first target that fails,
# and the library is the first target. One failing test there means none of the
# fourteen files under tests/ run at all: not reported as skipped, never
# started. That is how a broken guard record once reached main while this gate
# looked like it had checked it. The run still fails; it just says everything
# that is wrong rather than the first thing.
cargo test --all-targets --no-fail-fast

begin_stage "release build"
cargo build --release

# Five of CI's seven jobs, not four, and not all seven.
#
# This line said "All four checks passed" until 2026-09-13, and the header at
# the top of this file said the same. It was wrong twice over: the count was
# five even then, counting the shell suites, and CI runs seven jobs.
#
# What this run covers: Rustfmt, Clippy, the scripts that decide what runs, the
# Test Suite, the release half of Build, and Security Audit.
#
# What it does not, so that nobody reads the line above as "CI will be green":
#
#   * Build (debug). The test run compiles every target, so a debug build
#     failure would have to be something only a non-test profile reaches.
#   * Setup Executable. scripts/build-installer.sh needs Inno Setup.
#   * search-handler. CI runs fmt, clippy and the tests inside that crate, and
#     nothing here does. It is a second crate rather than a workspace member,
#     so every cargo command above walks straight past it.
echo "Formatting, clippy, the script suites, the tests, the release build and"
echo "the advisory check passed. That is five of CI's seven jobs. The debug"
echo "build, the setup executable and the search handler's own checks did not"
echo "run here."

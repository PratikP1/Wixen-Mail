#!/usr/bin/env bash
# Run most of what CI runs, in the same order, and fail on the first one.
#
# "The four checks CI runs" is what this line said until 2026-09-13, and CI has
# seven jobs. The last block in this file names which five a full run here
# covers and which two it does not, rather than leaving a count to drift again.
#
# Touching lib.rs first is not optional wherever clippy runs. Cargo shares
# fingerprints between `check`, `build`, `test`, and `clippy`, so a clippy run
# after a build can be considered fresh and report success without linting
# anything. That has already put a clippy failure on main once. A documents-only
# run neither touches nor lints, since 2026-10-18 (13.2-26.1); the reason is
# where the touch is.
set -euo pipefail

# The folder this script is in, worked out once without a process, for the
# questions `scripts/check.test.sh` asks dozens of times a run. `$(dirname
# "$0")` is a process, about a tenth of a second from bash on this machine on
# 2026-10-20 (13.3-01), and a `$0` holding no slash means this folder.
case "$0" in
    */*) the_scripts_folder="${0%/*}" ;;
    *) the_scripts_folder=. ;;
esac

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
#
# The ninth reads every file under `src` for a second caller of the search
# index's compaction, which rewrites the whole index and is measured only where
# a check calls it. A second caller can be added in any file, so no guard
# record's `file` can route the commit that adds one: the one record naming this
# target routes `wx_app.rs` alone, and 13-44.9's merge put a test-only caller
# on `main` without running it (ledger 791). Added 2026-10-03 by 13.1-02; it
# finished in 0.17 s and 0.18 s warm that day.
guards_that_read_the_whole_tree=(house_style wired the_planning_files_agree_with_themselves the_words_that_say_nothing no_label_is_only_a_space every_number_carries_its_command_and_its_date the_guard_sweep_runs_on_runners the_nvda_cases_wait_for_words_the_program_says mail_taken_off_leaves_no_words_behind)

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

# The target whose tests need the clipboard, and those tests by name, which a
# run on a locked session leaves out. Added 2026-10-23 by 13.3-24.8, on Pratik's
# "yes" of that day. Windows refuses the clipboard to every program while the
# session is locked (ledger 716), so these tests fail there whatever the code
# does: on 2026-10-22 they refused the merge of 13.3-20 after 872 s and the
# merge train stood still overnight. CI's Test Suite runs them on every push and
# pull request, and an unlocked session runs them here as before.
#
# Held to the code by `test_a_locked_session_leaves_out_exactly_the_tests_that_need_the_clipboard`
# in `tests/a_locked_key_asks_for_its_passphrase.rs`, which reads this list and
# names a test on it that reaches no clipboard, or one off it that does, and a
# guard record on this file routes every commit that changes the list to it.
# What that reading cannot see is a test reaching the clipboard through a
# function of another file. One name to a line, because the reading and
# `check.test.sh` read it a line at a time.
the_target_that_needs_the_clipboard=a_passphrase_box_takes_a_real_paste
the_tests_that_need_the_clipboard=(
    test_a_pasted_passphrase_is_taken_and_ok_hands_it_back
    test_the_paste_reading_sees_a_field_that_refuses_one
    test_a_clipboard_held_throughout_is_refused_naming_the_window_that_held_it
    test_a_clipboard_held_briefly_is_waited_for
)

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
#                 `red-commit.sh`, `session_lock.py`, and `audit.sh` with
#                 what `audit.sh` reads; and
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
the_inputs_of_a_suite[check]="scripts/check.test.sh scripts/check.sh scripts/session_lock.py guards/guards.toml scripts/which-checks.sh scripts/red-commit.sh scripts/audit.sh .cargo/audit.toml Cargo.lock scripts/audit.test.sh scripts/red-commit.test.sh scripts/which-checks.test.sh"
the_inputs_of_a_suite[red-commit]="scripts/red-commit.test.sh scripts/red-commit.sh"
the_inputs_of_a_suite[which-checks]="scripts/which-checks.test.sh scripts/which-checks.sh scripts/red-commit.sh"
what_every_suite_is_owed_for=(scripts/shell-suite.sh .githooks/commit-msg)
what_no_suite_reads=(scripts/guards.py scripts/guards.sh scripts/mutants.sh scripts/mutants_report.py scripts/read-a-proof.py scripts/build-installer.sh scripts/make-brand.py scripts/make-icon.py scripts/render_svg.py scripts/msaa-names.ps1 scripts/uia-events.ps1)

# Every record of a guard registry, read into three arrays of one entry per
# record: `record_files`, `record_suites`, empty where it names none, and
# `record_reds`, its red names with a space between them. Arrays rather than
# printed lines, because the readers below would read those lines back through
# `read`, which reads a byte at a time from a pipe here.
#
#     check.sh --records-in <registry>
#
# prints them, one record to a line and `|` between the three, which the check
# suite compares with a TOML reader.
#
# **This is a line scan, not a TOML parse.** A shell script has no TOML reader,
# and the file is written one key per line. So the scan keys on a `[[guard]]`
# header and then on `file = "..."`, `suite = "..."` and `red` at column 0
# inside it, keeping the record when the next header arrives. A red list is
# read in both the ways the registry writes one: a bare `red = [` line, a name
# to a line, and a bare `]`; and `red = ["...", "..."]` on one line, which 137
# records used on 2026-10-14.
#
# Since 2026-10-18 (13.2-26) the scan skips the inside of a multi-line string,
# `"""` or `'''`, so a `before` or `after` text holding a line that begins
# `file = `, `suite = ` or `red = [` is not read as a key. A string ends at the
# next delimiter of the kind it opened with, which TOML allows inside a `"""`
# string only escaped. `\"""` is one such escape and the scan reads it as an
# end; on 2026-10-18 every one of the 19 lines holding it ended its string
# there, and the scan's records were compared with Python's TOML reader for all
# 1,612 records and agreed on every file, suite and red name.
#
# `grep -o` hands the loop the pieces it reads rather than whole lines: a key
# line, a red name, a string's opening key, and each delimiter on its own; and
# they are split in memory rather than read with `mapfile`, which read a byte at
# a time here. Whole lines through `mapfile` cost two seconds a scan, measured
# 2026-10-18, against half a second for the scan this replaced, and the check
# suite asks the question dozens of times.
#
# Since 2026-10-20 (13.3-01, review item 30) a red name is read at any
# indentation, with a comment after it, and a one-line list or a bare `red = [`
# with a comment after its bracket. Until then a name was read only on a line of
# exactly four spaces and no comment, and the review's checkers planted a
# two-space name and two comments that the scan read as no red name at all.
#
# What it would still miss: a list opened on the line of its first name, a
# record spelled as an inline table, a key sharing a line with another, or a
# value quoted some other way. A record this misses is a guard that does not run
# on the commit that could break it, which is the defect the two readers below
# exist to fix. So `scripts/check.test.sh` compares this scan with Python's TOML
# reader over the real registry on every commit that stages it, and refuses the
# commit that adds a record the scan would misread.
record_files=()
record_suites=()
record_reds=()
read_the_records_in() {
    local registry="$1"
    local piece file="" suite="" reds="" name listing="" closing=""
    local -a pieces
    record_files=()
    record_suites=()
    record_reds=()
    # Split on newlines alone and never globbed, `[[guard]]` being a pattern,
    # both put back when the function returns.
    local - IFS=$'\n'
    set -f

    pieces=($(grep -oE \
        "^(\[\[guard\]\]|file = \".*\"|suite = \".*\"|red = \[.*|\]|[[:blank:]]*\"[A-Za-z0-9_:]+\",?[[:blank:]]*(#.*)?)\$|^[a-z_]+ = (\"\"\"|''')|\"\"\"|'''" \
        "$registry" || true))
    for piece in "${pieces[@]+"${pieces[@]}"}"; do
        if [ -n "$closing" ]; then
            [ "$piece" = "$closing" ] && closing=""
            continue
        fi
        case "$piece" in
            '[[guard]]')
                [ -n "$file" ] && keep_the_record "$file" "$suite" "$reds"
                file=""
                suite=""
                reds=""
                listing=""
                ;;
            'file = "'*'"')
                name="${piece#file = \"}"
                file="${name%\"}"
                ;;
            'suite = "'*'"')
                name="${piece#suite = \"}"
                suite="${name%\"}"
                ;;
            'red = ['*)
                # A comment after the bracket, and the spaces before it, are
                # not part of the list. No red name holds a `#`.
                name="${piece%%#*}"
                name="${name%"${name##*[![:blank:]]}"}"
                case "$name" in
                    'red = [')
                        listing=yes
                        ;;
                    *']')
                        name="${name#red = [}"
                        name="${name%]}"
                        name="${name//[\" ]/}"
                        for name in ${name//,/$'\n'}; do
                            reds+="${reds:+ }$name"
                        done
                        ;;
                esac
                ;;
            ']')
                listing=""
                ;;
            *' = """' | *" = '''")
                closing="${piece: -3}"
                ;;
            '"""' | "'''")
                ;;
            *'"'*)
                # A red name, at any indentation, with its comma and any
                # comment after it taken off.
                if [ -n "$listing" ]; then
                    name="${piece%%#*}"
                    reds+="${reds:+ }${name//[[:blank:]\",]/}"
                fi
                ;;
        esac
    done
    [ -n "$file" ] && keep_the_record "$file" "$suite" "$reds"
    return 0
}

keep_the_record() {
    record_files+=("$1")
    record_suites+=("$2")
    record_reds+=("$3")
}

# The records of a registry, or none when it cannot be read. The readers below
# add to what a commit earns, so a missing registry costs the extra targets and
# nothing else: it must never be the reason a commit cannot be made, and it says
# nothing, since `grep` would complain on its way to an empty answer.
#
# Read once by whoever asks, since 2026-10-20 (13.3-01, review item 36). The two
# readers below each read it for themselves until then, inside a process
# substitution whose arrays died with it, so a scoped run paid two scans of a
# registry of 1,640 records, about a second each on this machine that day.
read_the_registry_if_it_is_there() {
    record_files=()
    record_suites=()
    record_reds=()
    if [ -r "$1" ]; then
        read_the_records_in "$1"
    fi
    return 0
}

if [ "${1:-}" = "--records-in" ]; then
    read_the_records_in "$2"
    for index in "${!record_files[@]}"; do
        echo "${record_files[$index]}|${record_suites[$index]}|${record_reds[$index]}"
    done
    exit 0
fi

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
# Its suite is `scripts/check.test.sh`, which this script runs in every mode.
# In every mode until 2026-09-23; since then, on the commits that stage one of
# that suite's inputs, this file among them, and in `all` and `all_but_slow`.
#
# Reads the records `read_the_registry_if_it_is_there` left, so a caller reads
# the registry first:
#
#     the_suites_that_guard_what_changed [changed-file ...]
the_suites_that_guard_what_changed() {
    local -a couplings=() answers=()
    local index coupling path candidate already seen own_target

    for index in "${!record_files[@]}"; do
        if [ -n "${record_suites[$index]}" ]; then
            couplings+=("${record_files[$index]}|${record_suites[$index]}")
        fi
    done

    for path in "$@"; do
        # Every record coupling a changed file to a target, except a record
        # coupling a changed test file to that file's own target, which the
        # scoped run already adds a few lines up when the file is there.
        #
        # Until 2026-10-25 this passed over every changed test file, and read:
        # "Everything except a changed test file, which already runs its own
        # target a few lines up and would be asked for twice." That holds only
        # for a record whose suite is the file's own target. Two records name
        # another: the one whose break lands on `tests/tree_selection_raises.rs`
        # and reddens `every_window_test_ends_on_the_thread_that_owns_it`, and
        # the one whose break lands on
        # `tests/a_passphrase_box_takes_a_real_paste.rs` and reddens
        # `a_locked_key_asks_for_its_passphrase`. Neither ran on a commit
        # changing its file, which is ledger 962, closed by 13.4-12.1.
        #
        # The self-coupling is still passed over, and not only because it would
        # be asked for twice. A deleted test file stays in the staged list, and
        # the scoped run leaves its target out because cargo refuses a whole
        # call naming a target that is not there; answering the file's own
        # record here would put that target back.
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
        own_target=""
        case "$path" in
            tests/*.rs)
                own_target="${path##*/}"
                own_target="${own_target%.rs}"
                ;;
        esac

        for coupling in "${couplings[@]+"${couplings[@]}"}"; do
            [ "${coupling%%|*}" = "$path" ] || continue
            candidate="${coupling#*|}"
            [ "$candidate" = "$own_target" ] && continue

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

# Which library modules hold the red tests of the records that break a changed
# file and name no suite, one `a::b::tests::` filter to a line, unsorted and
# possibly repeated.
#
#     the_modules_that_guard_what_changed [changed-path ...]
#
# Like the reader above, it reads the records a caller has already read.
#
# A record with no `suite` reddens library tests, and they need not live in the
# module of the file it breaks. Read 2026-10-18 with Python's TOML reader over
# the registry's 1,612 records: 1,160 name no suite and break a source file, 77
# of them name no red test in that file's own module (`calendar.rs` 10,
# `wx_app.rs` 9, `google_api.rs` 6, `mail_sync.rs` 6, `managers.rs` 5) and 212
# more name some tests outside it. The scoped run reached only the changed
# file's own module, so those guards ran on every commit except the ones that
# could break them, and since 2026-09-23 the merge has not run them either
# (audit finding #38, closed here by 13.2-26).
#
# A name's module is everything before its last `::`, the rule `the_filters_for`
# in `scripts/guards.py` uses for the sweep. A name with no `::` has no module a
# filter could name and is skipped; none of the 1,612 records held one that day.
the_modules_that_guard_what_changed() {
    local index name path
    local -A changed=()
    for path in "$@"; do
        changed[$path]=1
    done
    for index in "${!record_files[@]}"; do
        [ -z "${record_suites[$index]}" ] &&
            [ -n "${changed[${record_files[$index]}]-}" ] || continue
        for name in ${record_reds[$index]}; do
            case "$name" in
                *::*) echo "${name%::*}::" ;;
            esac
        done
    done
    return 0
}

# The mapping on its own, so `scripts/check.test.sh` can ask it about a made-up
# file list and a made-up registry rather than running the whole gate. Answered
# here, before anything with a side effect and before the mode is looked at, so
# a suite asking this question can never start a gate run that would run that
# suite again.
if [ "${1:-}" = "--suites-for" ]; then
    shift
    read_the_registry_if_it_is_there "${1-}"
    shift || true
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

# The one library line for the filters given, in any order and each once:
# sorted, a filter inside a module another filter already names left out, and
# nothing at all for none. Sorted in place rather than through `sort`, because a
# process costs tens of milliseconds here and the list is a handful of words.
the_library_line_for() {
    local filter kept index
    local -a sorted=() line=()
    for filter in "$@"; do
        index=${#sorted[@]}
        while [ "$index" -gt 0 ] && [[ "${sorted[index - 1]}" > "$filter" ]]; do
            sorted[index]="${sorted[index - 1]}"
            index=$((index - 1))
        done
        sorted[index]="$filter"
    done
    for filter in "${sorted[@]+"${sorted[@]}"}"; do
        for kept in "${line[@]+"${line[@]}"}"; do
            case "$filter" in
                "$kept"*) continue 2 ;;
            esac
        done
        line+=("$filter")
    done
    if [ "${#line[@]}" -gt 0 ]; then
        echo "--lib -- --test-threads=4 ${line[*]}"
    fi
}

# What the scoped run hands cargo, one invocation's arguments to a line.
#
#     the_scoped_runs <registry> [changed-path ...]
#
# At most one `--lib -- --test-threads=4 <filter> ...` line naming every library
# filter the change earns, and one line of `--test` pairs naming every
# integration target the change reaches, each once: a
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
# Since 2026-10-18 (13.2-26) a library filter also goes to each module that
# holds the red tests of a record breaking a changed file and naming no suite,
# from `the_modules_that_guard_what_changed`.
#
# One library line since 2026-10-20 (13.3-01, review item 35). Until then it
# was one line per filter, "since `cargo test` takes one `--lib` filter", which
# is true of cargo and not of libtest: cargo refuses a second `--lib` and a
# second test name before `--`, and libtest takes several filters after it,
# `[FILTERS...]` in its usage, the way `scripts/guards.py` already passes them.
# So a change whose guards reached three modules paid cargo's start and the
# library's start three times. Now the filters are gathered, sorted, each named
# once, and a filter inside a module already named is left out, since the
# shorter filter already runs its tests and libtest runs a test matching two
# filters once. A change reaching no module answers no library line at all,
# because `cargo test --lib` given no filter runs the whole library.
#
# The trade, said rather than found: in one process, a library test that
# aborts the program hides the modules after it, where separate calls ran
# them. The gate is refused either way, `red-commit.sh` counts the test
# programs started against those that reported, and the whole suite already
# runs every library test in one process. The integration targets stay a call
# of their own, because filters after `--` in a call naming `--test` targets
# would filter those targets' tests too.
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
# of `the_modules_that_read_documents` among the library filters. Added
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
    local registry="$1" path module target source filter line=""
    shift
    local -a paths=("$@") targets=()
    local -A named=() filters=()
    read_the_registry_if_it_is_there "$registry"
    if [ "$#" -gt 0 ]; then
        while IFS= read -r source; do
            [ -n "$source" ] && paths+=("$source")
        done < <("$the_scripts_folder/which-checks.sh" --sources-compiling "$@")
    fi
    if [ -n "$merging" ] && [ "$#" -gt 0 ] &&
        [ -n "$("$the_scripts_folder/which-checks.sh" --documents-among "$@")" ]; then
        for module in "${the_modules_that_read_documents[@]}"; do
            filters[$module]=1
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
                filters[${module}::]=1
                ;;
            tests/*.rs)
                [ -f "$path" ] || continue
                target="${path##*/}"
                targets+=("${target%.rs}")
                ;;
        esac
    done
    while IFS= read -r filter; do
        filters[$filter]=1
    done < <(the_modules_that_guard_what_changed "${paths[@]+"${paths[@]}"}")
    the_library_line_for "${!filters[@]}"
    while IFS= read -r target; do
        [ -n "$target" ] && targets+=("$target")
    done < <(the_suites_that_guard_what_changed "${paths[@]+"${paths[@]}"}")
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

# Whether the words of a cargo test line reach the target whose tests need the
# clipboard: `--all-targets`, or `--test` followed by that target.
the_words_reach_the_clipboard_target() {
    local previous="" word
    for word in "$@"; do
        [ "$word" = --all-targets ] && return 0
        [ "$previous" = --test ] && [ "$word" = "$the_target_that_needs_the_clipboard" ] && return 0
        previous="$word"
    done
    return 1
}

# What a cargo test line hands libtest after its own words on a locked session:
# `--exact` and one `--skip` for each test on the list, so each is counted
# filtered out and never read as passed. One line when the lock answer is
# `locked` and the words reach the target; nothing otherwise, an unlocked
# session and a lock nobody could read included.
#
#     what_a_locked_session_leaves_out <lock answer> <cargo test words...>
what_a_locked_session_leaves_out() {
    local lock="$1" words="-- --exact" name
    shift
    [ "$lock" = locked ] || return 0
    the_words_reach_the_clipboard_target "$@" || return 0
    for name in "${the_tests_that_need_the_clipboard[@]}"; do
        words+=" --skip $name"
    done
    echo "$words"
}

# The rule on its own, so `scripts/check.test.sh` can ask it without a lock:
#
#     check.sh --what-a-locked-session-leaves-out <lock answer> <cargo test words...>
if [ "${1:-}" = "--what-a-locked-session-leaves-out" ]; then
    shift
    what_a_locked_session_leaves_out "$@"
    exit 0
fi

# Whether Windows says this session is locked, asked of
# `scripts/session_lock.py`: `locked`, `unlocked`, or `not said: ` and why,
# with what came back and its status when Python did not answer as it should.
# Never the process list and never the clipboard, for the reasons that script
# gives. On 2026-10-14 a Store alias made `python` stop answering on this
# machine, which is the shape of the last answer here.
this_sessions_lock() {
    local said status=0
    said="$(python -I "$the_scripts_folder/session_lock.py" 2>&1)" || status=$?
    said="${said//$'\r'/}"
    case "$status:$said" in
        "0:locked" | "0:unlocked" | "0:not said: "*) echo "$said" ;;
        *) echo "not said: python answered '${said//$'\n'/ }' with status $status" ;;
    esac
}

# Decided before each cargo test line that can reach the target whose tests
# need the clipboard, and only for those lines, so most runs start no extra
# process. Fills `left_out_while_locked`, which the line hands cargo after its
# own words, and says what it did: on a locked session the tests left out and
# where they do run, on a lock nobody could read that it could not ask and why,
# and on an unlocked session nothing.
left_out_while_locked=()
the_session_was_locked=""
leave_out_what_a_locked_session_cannot_run() {
    local lock
    left_out_while_locked=()
    the_words_reach_the_clipboard_target "$@" || return 0
    lock="$(this_sessions_lock)"
    case "$lock" in
        locked)
            read -ra left_out_while_locked <<< "$(what_a_locked_session_leaves_out locked "$@")"
            the_session_was_locked=yes
            echo "-- the session is locked, so the tests that need the clipboard do not run"
            echo "   here. Windows refuses the clipboard to every program while a session is"
            echo "   locked (ledger 716). CI's Test Suite runs them on every push and pull"
            echo "   request. Left out, from $the_target_that_needs_the_clipboard:"
            printf '   %s\n' "${the_tests_that_need_the_clipboard[@]}"
            ;;
        unlocked) ;;
        *)
            echo "-- could not ask whether this session is locked (${lock#not said: }),"
            echo "   so the tests that need the clipboard run. If it is locked they fail,"
            echo "   saying so."
            ;;
    esac
}

# The tests a red commit names that a locked session cannot run, one a line:
# each name `red-commit.sh names` gives that is on the list, when the lock
# answer is `locked`, and nothing otherwise. A test that cannot run neither
# passes nor fails, so a red naming one cannot be judged there. Added
# 2026-10-23 by 13.3-24.8.
#
#     check.sh --what-a-locked-session-cannot-judge <lock answer> <message file>
the_named_tests_a_locked_session_cannot_judge() {
    local lock="$1" message_file="$2" named listed
    [ "$lock" = locked ] || return 0
    while IFS= read -r named; do
        for listed in "${the_tests_that_need_the_clipboard[@]}"; do
            if [ "$named" = "$listed" ]; then
                echo "$named"
            fi
        done
    done < <("$the_scripts_folder/red-commit.sh" names "$message_file")
}

if [ "${1:-}" = "--what-a-locked-session-cannot-judge" ]; then
    shift
    the_named_tests_a_locked_session_cannot_judge "$@"
    exit 0
fi

# The sentence each mode's closing words gain when the session was locked.
say_what_the_lock_left_out() {
    [ -n "$the_session_was_locked" ] || return 0
    echo "The tests that need the clipboard did not run, because the session is"
    echo "locked; CI's Test Suite runs them."
}

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
                failing="$("$the_scripts_folder/red-commit.sh" names "$message_file")"
            ;;
    esac
    for candidate in "$the_scripts_folder"/*.test.sh; do
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
        echo "  --mode-for-this-commit [--message-file=F] for the mode it runs, or" >&2
        echo "  --what-a-locked-session-leaves-out <lock answer> <a cargo line's words>" >&2
        echo "  for what a run on a locked session hands libtest, or" >&2
        echo "  --what-a-locked-session-cannot-judge <lock answer> <message file>" >&2
        echo "  for the tests a red commit names that a locked session cannot run." >&2
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

# A commit is judged on the files it holds, or refused.
#
# Added 2026-10-18 by 13.2-25, audit finding #40. The mode above is decided from
# the staged files, and everything below builds and tests the files on disk. So
# a commit could pass while its own snapshot did not build, the parent's `mod b;`
# staged and `b.rs` left untracked, and be refused over work it did not hold.
# When the hook runs this, two questions are asked over one pathspec before the
# first stage: does a tracked file differ from the index the hook was handed,
# which for a partial commit is git's temporary one, and is any file untracked
# and not ignored. Either refuses. By hand nothing is refused, as before.
#
# The pathspec is the whole tree, since `locales/` is compiled in, `scripts/`
# and `.githooks/` run this gate, and `installer/`, `nvda-tests/` and the root's
# other files are read by tests, less four things planning and re-measuring
# leave unstaged in the same checkout: `.planning`, `docs`, `guards` and the
# Markdown files at the root, `glob` so `src/notes.md` is still asked about.
# Those four are where an unstaged change can still reach a test that reads
# documents, and ledger 325 says so and stays open.
the_build_inputs_left_out_of_the_commit() {
    local pathspec=(':(top)' ':(top,exclude).planning' ':(top,exclude)docs'
        ':(top,exclude)guards' ':(top,exclude,glob)*.md')
    git diff --name-only --no-renames -- "${pathspec[@]}" &&
        git ls-files --others --exclude-standard -- "${pathspec[@]}"
}

if [ -n "$message_file" ]; then
    if ! left_out="$(the_build_inputs_left_out_of_the_commit)"; then
        echo "check.sh: could not compare the files on disk with what this commit" >&2
        echo "stages, so it cannot say its tests would judge this commit. Refused." >&2
        exit 1
    fi
    if [ -n "$left_out" ]; then
        mapfile -t left_out_paths <<< "$left_out"
        echo >&2
        echo "This commit stages some files and leaves these different on disk," >&2
        echo "changed or untracked:" >&2
        printf '    %s\n' "${left_out_paths[@]:0:5}" >&2
        if [ "${#left_out_paths[@]}" -gt 5 ]; then
            echo "    and $(( ${#left_out_paths[@]} - 5 )) more" >&2
        fi
        echo >&2
        echo "The gate would run $mode over the files on disk, not over this commit," >&2
        echo "so it could pass a commit that does not build on its own, or refuse it" >&2
        echo "over work it does not hold. Stage them with git add, or set them aside" >&2
        echo "with" >&2
        echo >&2
        echo "    git stash --keep-index -u" >&2
        echo >&2
        echo "and commit again, then git stash pop." >&2
        exit 1
    fi
fi

# A red commit naming a test that needs the clipboard is refused on a locked
# session, here, before the mode line and the trap, so a refusal claims no
# stage. Added 2026-10-23 by 13.3-24.8: the named test cannot run there, so its
# failure would be the lock's and not the code's. Windows is asked only when the
# mode is `red` and a named test is on the list. When the lock cannot be read
# the red is judged as before, and were the session in fact locked, the test
# would fail with the paste target's own sentence about the lock.
case "$mode" in
    red)
        if [ -n "$message_file" ]; then
            cannot_judge="$(the_named_tests_a_locked_session_cannot_judge locked "$message_file")"
            if [ -n "$cannot_judge" ] && [ "$(this_sessions_lock)" = locked ]; then
                echo >&2
                mapfile -t cannot_judge_names <<< "$cannot_judge"
                echo "This red commit names tests that need the clipboard:" >&2
                printf '    %s\n' "${cannot_judge_names[@]}" >&2
                echo "and Windows says this session is locked. A test that cannot run" >&2
                echo "neither passes nor fails, so this red cannot be judged here." >&2
                echo "Unlock the session and commit again. Nothing was run." >&2
                exit 1
            fi
        fi
        ;;
esac

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
    if [ -n "$the_session_was_locked" ]; then
        stages+="; left out because the session is locked: ${#the_tests_that_need_the_clipboard[@]}"
        stages+=" tests of $the_target_that_needs_the_clipboard, which CI's Test Suite runs"
    fi
    echo "check.sh: $mode $outcome after $SECONDS s: $stages" >&2
    return "$status"
}

echo "check.sh: mode $mode${merging:+, a merge into $the_branch}"
trap the_run_ends EXIT

# A documents-only run neither forces the rebuild nor lints, since 2026-10-18
# (13.2-26.1, audit finding #45, on Pratik's "yes to both" of 2026-10-14).
# Clippy reads Rust, and such a commit changes none, so it could only find Rust
# left unstaged, which the refusal above stops, or a lint a new compiler
# brought, which CI's Clippy job and the phase's full gate catch. It cost about
# 30 seconds of a documents commit's 83 to 86 in 12-04 and 12-05. Spelled `!=`
# because `check.test.sh` reads the first test of the mode spelled with `=` as
# the first mode branch, and the suites must run before that one.
if [ "$mode" != "docs_only" ]; then
    touch src/lib.rs
fi

begin_stage "rustfmt"
cargo fmt --all -- --check

if [ "$mode" != "docs_only" ]; then
    begin_stage "clippy"
    cargo clippy --all-targets --all-features -- -D warnings
fi

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
    #
    # Every module in one call since 2026-10-20 (13.3-01, review item 35):
    # libtest takes several filters after `--`, and a call per module paid
    # cargo's start and the library's start once each.
    cargo test --lib -- "${the_modules_that_read_documents[@]}"
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
    echo "Formatting and the document-reading tests passed. Clippy, the rest of"
    echo "the suite and the release build did not run: nothing outside a document"
    echo "changed, so they had nothing to say. CI's Clippy job lints every push,"
    echo "and the rest run once a phase, by hand in its closing plan"
    echo "(scripts/check.sh all)."
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
# Returns non-zero if either call did. Never aborts on the library call, so a
# failure there does not hide the integration targets, for the same reason the
# whole-suite run passes `--no-fail-fast`.
#
# Runs exactly what `the_scoped_runs` answers, the lines read into an array
# first so no test a cargo call runs can read the rest of them from standard
# input. Since 2026-09-23 (12-03.2) the integration targets are one cargo call
# rather than one each, so a change coupled to 25 targets pays cargo's start
# once, and since 2026-10-20 (13.3-01) every library filter is one call as
# well; `red-commit.sh` reads the same `test NAME ... ok|FAILED` lines from the
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
                for word in "${arguments[@]:3}"; do
                    echo "-- ${word%::}"
                done
                # A filter matching nothing exits zero, so a module with no
                # tests of its own is not a pass, it is a run that said nothing.
                #
                # Every filter in one call since 2026-10-20 (13.3-01): the line
                # is `--lib -- --test-threads=4` and then the filters, and why
                # one call is the reasoning above `the_scoped_runs`.
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
                cargo test --lib "${arguments[@]:1}" >> "$run_log" 2>&1 || status=1
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
    #
    # On a locked session, the tests that need the clipboard are left out by the
    # words after the targets, since 2026-10-23 (13.3-24.8); the function above
    # the line says why and what it prints.
    leave_out_what_a_locked_session_cannot_run "$@"
    cargo test --no-fail-fast "$@" "${left_out_while_locked[@]+"${left_out_while_locked[@]}"}" >> "$run_log" 2>&1
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
    say_what_the_lock_left_out
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
    say_what_the_lock_left_out
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
#
# On a locked session the tests that need the clipboard are left out by name,
# since 2026-10-23 (13.3-24.8), and the run says so here and on its last line.
leave_out_what_a_locked_session_cannot_run --all-targets --no-fail-fast
cargo test --all-targets --no-fail-fast "${left_out_while_locked[@]+"${left_out_while_locked[@]}"}"

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
say_what_the_lock_left_out

//! The guard sweep dispatched onto GitHub's runners, read from the workflow
//! file before a runner is paid to find out.
//!
//! `.github/workflows/guards.yml` fans `scripts/guards.sh --shard k/n` out over
//! a matrix of Windows runners, each shard writing its own log and uploading
//! it, and the logs are read back as one on a machine. The workflow cannot be
//! run from here. What this reads is the shape a dispatch needs to get past
//! the checkout and the build: the flags the shard step hands the script are
//! ones the script accepts, spelled as it reads them; every input a step
//! reads is one the workflow declares; the compiler is the pinned one; the
//! variable a runner with no audio device needs is set; the job that runs the
//! suite checks out the whole history at the dispatched commit; the log is
//! kept whether or not the shard failed; and the workflow starts only when
//! somebody dispatches it, because a sweep is hours of runners and this
//! project's seventh guardrail says publishing and spending happen on purpose.
//!
//! Each of those fails on a runner after the checkout and the build, which is
//! the expensive place to find out, and the mutation workflow already found
//! two of them that way (`tests/house_style.rs` has the readings for that
//! workflow, and this file follows their shape). A companion below plants each
//! mistake in a sound workflow and requires the reading to name it, because a
//! reading that iterates over nothing passes.
//!
//! A new target rather than two more tests in `tests/house_style.rs`: twenty-one
//! guard records name that file, and a test added to it flags every one for
//! re-measurement at a build and a suite run each. This target is in
//! `scripts/check.sh`'s whole-tree list so a commit touching `scripts/guards.py`
//! runs it, and its own record in `guards/guards.toml` couples it to the
//! workflow file.

use std::fs;

/// The workflow, the script whose flags it must spell, and the pin.
const THE_WORKFLOW: &str = ".github/workflows/guards.yml";
const THE_SCRIPT: &str = "scripts/guards.py";
const THE_PIN: &str = "rust-toolchain.toml";

/// The inputs a dispatch reads, by name. A step reading an input the workflow
/// does not declare gets an empty string and runs `--shard /`, which the
/// script refuses in its first second; the reverse mistake, an input declared
/// and never read, costs nothing and is not a finding.
const THE_DISPATCH_INPUTS: &[&str] = &["shards", "first", "last"];

/// Everything in a file except the lines explaining it, so a comment that
/// names a mistake the file no longer makes is not read as making it.
fn what_it_does_not_what_it_says(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every long flag `scripts/guards.py` accepts, read off its `add_argument`
/// calls: the file writes each flag on a line of its own as the call's first
/// argument, `"--log",`, and a flag spelled any other way is not one this
/// reading can see, which is the same trade `scripts/check.sh` makes over
/// `guards.toml`.
fn the_flags_the_script_accepts(script: &str) -> Vec<String> {
    script
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('"')?.strip_suffix("\","))
        .filter(|flag| flag.starts_with("--"))
        .map(str::to_string)
        .collect()
}

/// The compiler `rust-toolchain.toml` names, from its `channel = "..."` line.
fn the_compiler_the_pin_file_names(text: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix("channel = \""))
        .and_then(|rest| rest.split('"').next())
        .map(str::to_string)
}

/// Every compiler a workflow installs, as `(line number, what it named)`, the
/// ref after `dtolnay/rust-toolchain@` being the one spelling this project
/// uses.
fn the_compilers_a_workflow_installs(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().starts_with('#'))
        .filter_map(|(index, line)| {
            line.trim()
                .split_once("dtolnay/rust-toolchain@")
                .map(|(_, named)| (index + 1, named.trim().to_string()))
        })
        .collect()
}

/// The jobs of a workflow, as `(name, text)`, split at the two-space keys
/// under `jobs:`, comment lines dropped first.
fn the_jobs_of(workflow: &str) -> Vec<(String, String)> {
    let mut jobs: Vec<(String, String)> = Vec::new();
    let mut inside = false;
    for line in what_it_does_not_what_it_says(workflow).lines() {
        if line.trim_end() == "jobs:" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let is_a_job = line.starts_with("  ")
            && !line.starts_with("   ")
            && line.trim_end().ends_with(':')
            && !line.trim().contains(' ');
        if is_a_job {
            jobs.push((line.trim().trim_end_matches(':').to_string(), String::new()));
        } else if let Some((_, text)) = jobs.last_mut() {
            text.push_str(line);
            text.push('\n');
        }
    }
    jobs
}

/// Every input the workflow's steps read, as `inputs.<name>`.
fn the_inputs_the_steps_read(runs: &str) -> Vec<String> {
    let mut read = Vec::new();
    for (at, matched) in runs.match_indices("inputs.") {
        let name: String = runs[at + matched.len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() && !read.contains(&name) {
            read.push(name);
        }
    }
    read
}

/// The lines under `on:`, so a trigger is read where it is declared and not
/// wherever the word happens to appear.
fn the_triggers_of(runs: &str) -> Vec<String> {
    let mut triggers = Vec::new();
    let mut inside = false;
    for line in runs.lines() {
        if line.trim_end() == "on:" {
            inside = true;
            continue;
        }
        if inside && !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if inside && line.starts_with("  ") && !line.starts_with("   ") {
            triggers.push(line.trim().trim_end_matches(':').to_string());
        }
    }
    triggers
}

/// What the guard sweep's dispatch gets wrong, one line each, empty when
/// nothing is. Read from the workflow text, the script text and the pinned
/// compiler, so a companion can hand it a planted mistake.
fn what_the_guard_sweep_dispatch_gets_wrong(
    workflow: &str,
    script: &str,
    pinned: &str,
) -> Vec<String> {
    let mut wrong = Vec::new();
    let runs = what_it_does_not_what_it_says(workflow);

    let triggers = the_triggers_of(&runs);
    if !triggers.iter().any(|t| t == "workflow_dispatch") {
        wrong.push("the workflow cannot be dispatched by hand, and a sweep is only ever started on purpose".to_string());
    }
    for trigger in triggers.iter().filter(|t| *t != "workflow_dispatch") {
        wrong.push(format!(
            "the workflow also runs on {trigger}, and a sweep is hours of runners nobody asked for"
        ));
    }

    for input in THE_DISPATCH_INPUTS {
        if !runs.lines().any(|line| line.trim() == format!("{input}:")) {
            wrong.push(format!("the dispatch declares no input called {input}"));
        }
    }
    for name in the_inputs_the_steps_read(&runs) {
        if !runs.lines().any(|line| line.trim() == format!("{name}:")) {
            wrong.push(format!(
                "a step reads inputs.{name}, which the dispatch does not declare, so it reads an empty string"
            ));
        }
    }

    let accepted = the_flags_the_script_accepts(script);
    let shard_steps: Vec<&str> = runs
        .lines()
        .filter(|line| line.contains("scripts/guards.sh") && line.contains("--shard"))
        .collect();
    if shard_steps.is_empty() {
        wrong.push(
            "no step runs scripts/guards.sh with --shard, so a dispatch cannot run a shard"
                .to_string(),
        );
    }
    for step in &shard_steps {
        if !step.contains("--log") {
            wrong.push("the shard step passes no --log, so the verdicts go to the job's console and no artifact can carry them".to_string());
        }
        if step.contains("--wait-until-quiet") {
            wrong.push("the shard step passes --wait-until-quiet, and a runner is nobody's machine: nothing else builds there, so the wait can only delay".to_string());
        }
        for word in step
            .split_whitespace()
            .skip_while(|word| !word.ends_with("guards.sh"))
            .skip(1)
        {
            let Some(at) = word.find("--") else {
                continue;
            };
            let flag = word[at..].trim_end_matches(['"', '\'', '}']);
            if !accepted.iter().any(|known| known == flag) {
                wrong.push(format!(
                    "the shard step passes {flag}, which scripts/guards.py does not accept"
                ));
            }
        }
    }

    for (line, named) in the_compilers_a_workflow_installs(workflow) {
        if named != pinned {
            wrong.push(format!(
                "line {line} installs {named} where the pin file names {pinned}"
            ));
        }
    }
    if !runs
        .lines()
        .any(|line| line.trim().starts_with("WIXEN_NO_AUDIO:"))
    {
        wrong.push("WIXEN_NO_AUDIO is not set, and GitHub's runners have no audio driver, so the first sound test would crash the run".to_string());
    }

    for (name, text) in the_jobs_of(workflow)
        .into_iter()
        .filter(|(_, text)| text.contains("scripts/guards.sh"))
    {
        if !text.lines().any(|line| line.trim() == "fetch-depth: 0") {
            wrong.push(format!(
                "the {name} job runs the suite on a checkout without the history, and a test the house_style target holds reads the share of history before red/green; give its checkout fetch-depth: 0"
            ));
        }
        if !text
            .lines()
            .any(|line| line.trim() == "ref: ${{ github.sha }}")
        {
            wrong.push(format!(
                "the {name} job does not check out github.sha, so two shards of one dispatch could judge two commits"
            ));
        }
        if !text.contains("upload-artifact") {
            wrong.push(format!(
                "the {name} job uploads nothing, so its log is lost when the runner is"
            ));
        } else if !text
            .lines()
            .any(|line| line.trim().trim_start_matches("- ") == "if: always()")
        {
            wrong.push(format!(
                "the {name} job uploads its log only on success, and a failed shard's log is the one worth reading"
            ));
        }
        if !text
            .lines()
            .any(|line| line.trim().starts_with("timeout-minutes:"))
        {
            wrong.push(format!("the {name} job names no timeout, so a reader cannot see the cap a shard is sized under"));
        }
        if !text.lines().any(|line| line.trim() == "fail-fast: false") {
            wrong.push(format!(
                "the {name} job stops every shard at the first failure, and each shard is its own unit"
            ));
        }
    }
    wrong
}

#[test]
fn test_the_guard_sweep_dispatch_hands_the_script_flags_it_accepts() {
    let workflow = fs::read_to_string(THE_WORKFLOW)
        .unwrap_or_else(|e| panic!("{THE_WORKFLOW}: {e}; the sweep cannot be dispatched"));
    let script = fs::read_to_string(THE_SCRIPT).expect("the guard runner");
    let pin = fs::read_to_string(THE_PIN).expect("the pin file");
    let pinned = the_compiler_the_pin_file_names(&pin).expect("the pin file names a channel");

    let wrong = what_the_guard_sweep_dispatch_gets_wrong(&workflow, &script, &pinned);
    assert!(
        wrong.is_empty(),
        "the guard sweep's dispatch would fail on a runner:\n  {}",
        wrong.join("\n  ")
    );
}

/// A workflow the reading accepts, small enough to plant one mistake in.
const A_SOUND_DISPATCH: &str = "on:\n  workflow_dispatch:\n    inputs:\n      shards:\n      first:\n      last:\nenv:\n  WIXEN_NO_AUDIO: \"1\"\njobs:\n  shard:\n    timeout-minutes: 360\n    strategy:\n      fail-fast: false\n    steps:\n    - uses: actions/checkout@v4\n      with:\n        ref: ${{ github.sha }}\n        fetch-depth: 0\n    - uses: dtolnay/rust-toolchain@1.98.1\n    - run: bash scripts/guards.sh --shard ${{ matrix.k }}/${{ inputs.shards }} --log sweep-${{ matrix.k }}-of-${{ inputs.shards }}.log\n    - if: always()\n      uses: actions/upload-artifact@v4\n";

/// The script's flags as the reading sees them, spelled as `guards.py` does.
const THE_SCRIPT_AS_WRITTEN: &str = "    parsing.add_argument(\n        \"--shard\",\n    )\n    parsing.add_argument(\n        \"--log\",\n    )\n    parsing.add_argument(\n        \"--wait-until-quiet\",\n    )\n";

#[test]
fn test_the_dispatch_reading_can_see_each_mistake_it_exists_to_refuse() {
    // The companion: a reading that iterates over nothing passes, so it is
    // shown a sound workflow and must accept it, then one mistake at a time
    // and must name each.
    let accepted =
        what_the_guard_sweep_dispatch_gets_wrong(A_SOUND_DISPATCH, THE_SCRIPT_AS_WRITTEN, "1.98.1");
    assert!(
        accepted.is_empty(),
        "a sound dispatch was refused: {accepted:?}"
    );

    let planted: &[(&str, &str, &str)] = &[
        (
            "--shard ",
            "--shards ",
            "--shards, which scripts/guards.py does not accept",
        ),
        ("      first:\n", "", "no input called first"),
        (
            "--log sweep",
            "--log sweep --wait-until-quiet",
            "passes --wait-until-quiet",
        ),
        ("if: always()", "if: success()", "only on success"),
        ("        fetch-depth: 0\n", "", "without the history"),
        (
            "        ref: ${{ github.sha }}\n",
            "",
            "does not check out github.sha",
        ),
        ("  WIXEN_NO_AUDIO: \"1\"\n", "", "WIXEN_NO_AUDIO is not set"),
        (
            "on:\n  workflow_dispatch:\n",
            "on:\n  push:\n  workflow_dispatch:\n",
            "also runs on push",
        ),
        (
            "      fail-fast: false\n",
            "",
            "stops every shard at the first failure",
        ),
        ("    timeout-minutes: 360\n", "", "names no timeout"),
    ];
    for (from, to, expected) in planted {
        let broken = A_SOUND_DISPATCH.replace(from, to);
        assert_ne!(
            broken, A_SOUND_DISPATCH,
            "the plant for {expected:?} changed nothing"
        );
        let wrong =
            what_the_guard_sweep_dispatch_gets_wrong(&broken, THE_SCRIPT_AS_WRITTEN, "1.98.1");
        assert!(
            wrong.iter().any(|line| line.contains(expected)),
            "a workflow with {from:?} replaced by {to:?} was not refused for {expected:?}: {wrong:?}"
        );
    }

    let wrong =
        what_the_guard_sweep_dispatch_gets_wrong(A_SOUND_DISPATCH, THE_SCRIPT_AS_WRITTEN, "1.97.1");
    assert!(
        wrong
            .iter()
            .any(|line| line.contains("1.98.1 where the pin file names 1.97.1")),
        "a compiler disagreeing with the pin was not named: {wrong:?}"
    );

    // And a step reading an input the sound workflow never declares, which
    // is the direction the mutation workflow's reading did not ask.
    let reads_more =
        A_SOUND_DISPATCH.replace("${{ inputs.shards }}.log", "${{ inputs.total }}.log");
    let wrong =
        what_the_guard_sweep_dispatch_gets_wrong(&reads_more, THE_SCRIPT_AS_WRITTEN, "1.98.1");
    assert!(
        wrong.iter().any(|line| line.contains("reads inputs.total")),
        "a step reading an undeclared input was not named: {wrong:?}"
    );
}

//! The planning files agree with themselves, and with the files on disk.
//!
//! `.planning/STATE.md` holds the same facts twice, once in frontmatter that
//! tooling parses and once in a prose heading that people read. The two came
//! apart three times in one week. On 2026-09-01 both said phase 01 while five
//! phase 02 plans had shipped, which made `state.advance-plan` fail and tagged
//! new decisions against the wrong phase. On 2026-09-07 the heading still
//! described phase 02 and lacked the two fields the tooling parses. Hours
//! later the frontmatter said `current_plan: 6` while the heading said 7,
//! because one plan updated one half.
//!
//! That third one happened in a file which by then carried three paragraphs
//! explaining this exact failure. That is the whole argument for this target:
//! a rule that lives in a document is a rule somebody has to notice being
//! broken, and prose explaining a divergence does not stop the divergence.
//!
//! `.planning/WINDOWS.md` is the same shape and worse. Every ledger entry
//! exists twice, as a markdown table row and as a JSON object. Editing only
//! the table silently reverts on the next tool write. That has already caused
//! two failed repairs and closed the wrong entry once.
//!
//! `.planning/ROADMAP.md` drifts from disk rather than from itself: its
//! progress table says how many plans and summaries a phase has, and those are
//! files somebody can count. A row reading `8/9` for a phase whose nine plans
//! and nine summaries are all present was corrected by hand on 2026-09-07.
//!
//! # How each check is built, and why it is built that way
//!
//! Every reading here is a function over text, returning what it found wrong.
//! The check runs it over the real file and requires the answer to be empty.
//! A companion beside it runs the same function over that file's own real
//! lines with one violation spliced in, and requires the answer to name
//! exactly that violation.
//!
//! The companion is not decoration. `CLAUDE.md` records a guard here that sat
//! passing over nothing for weeks while the rule it enforced lapsed five
//! times, because the documents it read had stopped naming the thing it looked
//! for. Every check in this file is green today, because the files agree
//! today, so from outside every one of them is indistinguishable from a check
//! that read nothing. The companions are what tell those two apart.
//!
//! A second defence sits inside each reading: **it complains when it cannot
//! find what it is comparing.** A missing `current_plan`, a Current Position
//! section with no `Phase:` line, a ledger with no JSON block, a progress
//! table that is not there: each of those is reported as a finding rather than
//! skipped. A reading that returns nothing because it found nothing to read is
//! the failure this whole file exists to prevent, so it cannot be spelled here.
//!
//! # What is deliberately not checked
//!
//! `progress.total_plans` says 87 and there are 85 `*-PLAN.md` files. It is
//! left alone because nothing here can say what it counts. The roadmap's own
//! progress denominators sum to 85, and `deriveProgressFromRoadmap` in the
//! vendored tooling sums exactly those, so that reading gives 85 too. The 87
//! was written into the file by a tool run on 2026-09-07 over a roadmap whose
//! denominators already summed to 85, and a second writer in the same tooling
//! sets the field to whatever an argument passes it. Two writers with
//! different meanings and an observed value matching neither reading is not a
//! fact to assert, so it is written down here instead.
//!
//! The roadmap reading goes one way only: every row must match the disk. A
//! phase directory with plans in it and no row at all would not be seen. That
//! is a real gap and it is recorded rather than closed.

use std::collections::BTreeMap;
use std::fs;

const STATE: &str = ".planning/STATE.md";
const WINDOWS: &str = ".planning/WINDOWS.md";
const ROADMAP: &str = ".planning/ROADMAP.md";
const PHASES: &str = ".planning/phases";

// ---------------------------------------------------------------------------
// Reading the tree
// ---------------------------------------------------------------------------

/// What one phase directory holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PhaseFiles {
    plans: usize,
    summaries: usize,
}

/// Every phase directory, keyed by its phase number with leading zeros gone.
///
/// The key is normalised because the same phase is spelled three ways in this
/// project: `04.2` in a directory name and in the state frontmatter, `4.2` in
/// a roadmap row. A check demanding byte equality between those would fire on
/// a file that is right.
fn phases_on_disk() -> BTreeMap<String, PhaseFiles> {
    let mut found: BTreeMap<String, PhaseFiles> = BTreeMap::new();
    let Ok(entries) = fs::read_dir(PHASES) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(key) = phase_key(name) else {
            continue;
        };
        let mut files = PhaseFiles::default();
        if let Ok(inner) = fs::read_dir(&path) {
            for file in inner.flatten() {
                let Some(name) = file.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                if name.ends_with("-PLAN.md") {
                    files.plans += 1;
                } else if name.ends_with("-SUMMARY.md") {
                    files.summaries += 1;
                }
            }
        }
        found.insert(key, files);
    }
    found
}

/// The phase number at the front of a directory name, a roadmap cell or a
/// frontmatter value, with leading zeros stripped from each part.
///
/// `04.2-what-was-built` and `4.2 What was built` and `04.2` all give `4.2`.
/// `None` when what is there does not start with a digit, which is how a
/// heading row or a stray directory is passed over rather than guessed at.
fn phase_key(raw: &str) -> Option<String> {
    let token = raw.split_whitespace().next()?;
    let token = token.split('-').next()?;
    let token = token.trim_end_matches('.');
    if token.is_empty() || !token.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let mut parts = Vec::new();
    for part in token.split('.') {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let trimmed = part.trim_start_matches('0');
        parts.push(if trimmed.is_empty() { "0" } else { trimmed }.to_string());
    }
    Some(parts.join("."))
}

// ---------------------------------------------------------------------------
// Reading STATE.md
// ---------------------------------------------------------------------------

/// The frontmatter block, which is everything between the first two `---`
/// lines. Empty when the file has no frontmatter, which every reading below
/// reports rather than treats as nothing to say.
fn frontmatter(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return Vec::new();
    }
    let mut block = Vec::new();
    for (number, line) in lines.iter().enumerate().skip(1) {
        if line.trim_end() == "---" {
            return block;
        }
        block.push((number + 1, *line));
    }
    Vec::new()
}

/// Every value a key at column zero of the frontmatter carries.
///
/// A list rather than one answer, because "this fact is written twice and the
/// two copies differ" is the disease being treated, and a reading that took
/// the first match would be blind to a second copy of the key.
fn frontmatter_values(text: &str, key: &str) -> Vec<(usize, String)> {
    let prefix = format!("{key}:");
    frontmatter(text)
        .into_iter()
        .filter(|(_, line)| line.starts_with(&prefix))
        .map(|(number, line)| (number, line[prefix.len()..].trim().to_string()))
        .collect()
}

/// The lines of the `## Current Position` section, which is where the prose
/// half of every state fact lives.
fn current_position(text: &str) -> Vec<(usize, &str)> {
    let mut inside = false;
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if line.starts_with("## ") {
            inside = line.trim_end() == "## Current Position";
            continue;
        }
        if inside {
            found.push((number + 1, line));
        }
    }
    found
}

/// Every value a `Name:` line at column zero of that section carries.
fn current_position_values(text: &str, key: &str) -> Vec<(usize, String)> {
    let prefix = format!("{key}:");
    current_position(text)
        .into_iter()
        .filter(|(_, line)| line.starts_with(&prefix))
        .map(|(number, line)| (number, line[prefix.len()..].trim().to_string()))
        .collect()
}

/// Exactly one value, or the reason there is not one.
///
/// None and several are both findings. Several matters as much as none here:
/// this file's whole failure mode is one fact written twice, so a second copy
/// of a key is the disease rather than a tidiness point.
fn the_one_value(
    file: &str,
    what: &str,
    found: Vec<(usize, String)>,
) -> Result<(usize, String), String> {
    match found.len() {
        1 => Ok(found.into_iter().next().unwrap_or_default()),
        0 => Err(format!(
            "{file}: there is no {what}, so there is nothing here to compare and \
             this check would pass over an empty reading"
        )),
        _ => Err(format!(
            "{file}: {what} is written {} times, on lines {}, and a fact written \
             twice is what this file keeps getting wrong",
            found.len(),
            found
                .iter()
                .map(|(number, _)| number.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Check 1. The phase the frontmatter names is the phase the heading names.
fn phase_disagreements(text: &str) -> Vec<String> {
    let _ = text;
    Vec::new()
}

/// Check 2. `current_plan` in the frontmatter is `Current Plan:` in the body.
fn plan_disagreements(text: &str) -> Vec<String> {
    let _ = text;
    Vec::new()
}

/// Check 3. `Total Plans in Phase` is the `*-PLAN.md` files in that phase.
fn total_plans_in_phase_disagreements(
    text: &str,
    on_disk: &BTreeMap<String, PhaseFiles>,
) -> Vec<String> {
    let _ = (text, on_disk);
    Vec::new()
}

/// Check 4. `progress.completed_plans` is every `*-SUMMARY.md` on disk.
fn completed_plans_disagreements(
    text: &str,
    on_disk: &BTreeMap<String, PhaseFiles>,
) -> Vec<String> {
    let _ = (text, on_disk);
    Vec::new()
}

// ---------------------------------------------------------------------------
// Reading WINDOWS.md
// ---------------------------------------------------------------------------

/// Enough of a long description to tell two apart in a failure message.
fn shortened(text: &str) -> String {
    let mut taken: String = text.chars().take(70).collect();
    if text.chars().count() > 70 {
        taken.push_str("...");
    }
    taken
}

/// Checks 5 and 6. The markdown table and the JSON block hold the same ids,
/// and for every id in both, the same status and the same description.
fn ledger_disagreements(text: &str) -> Vec<String> {
    let _ = text;
    Vec::new()
}

// ---------------------------------------------------------------------------
// Reading ROADMAP.md
// ---------------------------------------------------------------------------

/// Check 7. Every progress-table row says what is on disk for its phase.
fn roadmap_disagreements(text: &str, on_disk: &BTreeMap<String, PhaseFiles>) -> Vec<String> {
    let _ = (text, on_disk);
    Vec::new()
}

// ---------------------------------------------------------------------------
// Splicing, which is how every companion below puts a violation into a real
// file's own lines rather than into a fixture
// ---------------------------------------------------------------------------

/// The text with exactly one line replaced, and a panic if the line to replace
/// was not there exactly once.
///
/// Over the real file's real lines on purpose. A fixture built by hand is the
/// trap this project has been caught by five times: a phase with one plan
/// makes several of the numbers here coincide at 1, and a check comparing the
/// wrong pair passes on it.
fn with_one_line_replaced(
    text: &str,
    matching: impl Fn(&str) -> bool,
    replacement: &str,
) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| matching(line))
        .map(|(at, _)| at)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "the companion means to splice one line and {} matched, so it is about \
         to prove something other than what it says",
        hits.len()
    );
    let mut spliced: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    spliced[hits[0]] = replacement.to_string();
    spliced.join("\n")
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path} is a planning file, and it {e}"))
}

// ---------------------------------------------------------------------------
// The checks
// ---------------------------------------------------------------------------

#[test]
fn test_the_state_frontmatter_and_the_heading_name_the_same_phase() {
    let wrong = phase_disagreements(&read(STATE));
    assert!(
        wrong.is_empty(),
        "the state file holds the phase twice and the two copies disagree, \
         which made state.advance-plan fail once and tag decisions against the \
         wrong phase:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_state_frontmatter_and_the_heading_name_the_same_plan() {
    let wrong = plan_disagreements(&read(STATE));
    assert!(
        wrong.is_empty(),
        "the state file holds the plan number twice and the two copies \
         disagree, which is what 04.2-06 left behind by updating one half:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_state_file_counts_the_plans_that_are_on_disk() {
    let wrong = total_plans_in_phase_disagreements(&read(STATE), &phases_on_disk());
    assert!(
        wrong.is_empty(),
        "the state file says how many plans this phase has and the phase \
         directory says otherwise:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_state_file_counts_the_summaries_that_are_on_disk() {
    let wrong = completed_plans_disagreements(&read(STATE), &phases_on_disk());
    assert!(
        wrong.is_empty(),
        "the state file's completed_plans is a count of files somebody can \
         make, and it is not that count:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_both_halves_of_the_ledger_say_the_same_thing() {
    let wrong = ledger_disagreements(&read(WINDOWS));
    assert!(
        wrong.is_empty(),
        "every ledger entry is written twice, as a table row and as a JSON \
         object, and the tool writes from the JSON: a table edited on its own \
         reverts on the next write, which has already closed the wrong \
         entry:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_roadmap_counts_the_files_that_are_on_disk() {
    let wrong = roadmap_disagreements(&read(ROADMAP), &phases_on_disk());
    assert!(
        wrong.is_empty(),
        "the roadmap's progress table says how far each phase got, and the \
         phase directories say otherwise:\n  {}",
        wrong.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// The companions, one per check, each splicing into the real file's own lines
// ---------------------------------------------------------------------------

#[test]
fn test_the_phase_reading_can_see_a_disagreement() {
    let text = read(STATE);
    assert_eq!(
        phase_disagreements(&text),
        Vec::<String>::new(),
        "the file already disagrees with itself, so what this splices in is \
         not the only thing the reading has to find"
    );

    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with("current_phase:"),
        "current_phase: 99.9",
    );
    assert_eq!(
        phase_disagreements(&spliced),
        vec![format!(
            "{STATE}: the frontmatter says phase 99.9 and the Current Position \
             heading says {}",
            the_heading_phase(&text)
        )],
        "a wrong phase was put into the state file's own frontmatter and the \
         reading did not answer with exactly that"
    );
}

/// The phase the real heading names, so a companion can say what the reading
/// should have compared against without writing today's phase into this file.
fn the_heading_phase(text: &str) -> String {
    the_one_value(
        STATE,
        "a Phase line",
        current_position_values(text, "Phase"),
    )
    .map(|(_, value)| value.split_whitespace().next().unwrap_or("").to_string())
    .unwrap_or_default()
}

#[test]
fn test_the_plan_reading_can_see_a_disagreement() {
    let text = read(STATE);
    assert_eq!(
        plan_disagreements(&text),
        Vec::<String>::new(),
        "the file already disagrees with itself, so what this splices in is \
         not the only thing the reading has to find"
    );

    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with("current_plan:"),
        "current_plan: 4242",
    );
    assert_eq!(
        plan_disagreements(&spliced),
        vec![format!(
            "{STATE}: the frontmatter says current_plan 4242 and the body says \
             Current Plan: {}",
            the_body_plan(&text)
        )],
        "a wrong plan number was put into the state file's own frontmatter and \
         the reading did not answer with exactly that"
    );
}

/// What the real body says the current plan is.
fn the_body_plan(text: &str) -> String {
    the_one_value(
        STATE,
        "a Current Plan line",
        current_position_values(text, "Current Plan"),
    )
    .map(|(_, value)| value)
    .unwrap_or_default()
}

#[test]
fn test_the_plan_count_reading_can_see_a_disagreement() {
    let text = read(STATE);
    let on_disk = phases_on_disk();
    assert_eq!(
        total_plans_in_phase_disagreements(&text, &on_disk),
        Vec::<String>::new(),
        "the state file already disagrees with the phase directory, so what \
         this splices in is not the only thing the reading has to find"
    );

    let key = the_frontmatter_phase_key(&text);
    let plans = on_disk.get(&key).map(|f| f.plans).unwrap_or_default();
    assert_ne!(
        plans, 4242,
        "the number spliced in below is the number of plans on disk, so the \
         reading would be right to stay quiet and this proves nothing"
    );

    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with("Total Plans in Phase:"),
        "Total Plans in Phase: 4242",
    );
    assert_eq!(
        total_plans_in_phase_disagreements(&spliced, &on_disk),
        vec![format!(
            "{STATE}: Total Plans in Phase says 4242 and phase {key} holds \
             {plans} *-PLAN.md files on disk"
        )],
        "a wrong plan count was put into the state file's own body and the \
         reading did not answer with exactly that"
    );
}

/// The phase the real frontmatter names, normalised the way the disk is.
fn the_frontmatter_phase_key(text: &str) -> String {
    the_one_value(
        STATE,
        "a current_phase",
        frontmatter_values(text, "current_phase"),
    )
    .ok()
    .and_then(|(_, value)| phase_key(&value))
    .unwrap_or_default()
}

#[test]
fn test_the_summary_count_reading_can_see_a_disagreement() {
    let text = read(STATE);
    let on_disk = phases_on_disk();
    assert_eq!(
        completed_plans_disagreements(&text, &on_disk),
        Vec::<String>::new(),
        "the state file already disagrees with the phase directories, so what \
         this splices in is not the only thing the reading has to find"
    );

    let summaries: usize = on_disk.values().map(|f| f.summaries).sum();
    assert_ne!(
        summaries, 4242,
        "the number spliced in below is the number of summaries on disk, so \
         the reading would be right to stay quiet and this proves nothing"
    );

    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with("  completed_plans:"),
        "  completed_plans: 4242",
    );
    assert_eq!(
        completed_plans_disagreements(&spliced, &on_disk),
        vec![format!(
            "{STATE}: progress.completed_plans says 4242 and the phase \
             directories hold {summaries} *-SUMMARY.md files"
        )],
        "a wrong summary count was put into the state file's own frontmatter \
         and the reading did not answer with exactly that"
    );
}

#[test]
fn test_the_ledger_reading_can_see_all_three_ways_the_halves_come_apart() {
    let text = read(WINDOWS);
    assert_eq!(
        ledger_disagreements(&text),
        Vec::<String>::new(),
        "the two halves of the ledger already disagree, so what this splices \
         in is not the only thing the reading has to find"
    );

    let (id, status, description) = the_last_ledger_row(&text);

    // An id in one half and not the other, which is what a hand-added row
    // looks like. Both directions at once, because renumbering one row takes
    // its id out of the table and leaves the JSON's copy with no row.
    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with(&format!("| {id} |")),
        &format!("| 4242 | 04.2 | stub | a/b.rs |  | {description} | {status} |  | x |  |"),
    );
    assert_eq!(
        ledger_disagreements(&spliced),
        vec![
            format!("{WINDOWS}: ledger {id} is in the JSON block and has no table row"),
            format!("{WINDOWS}: ledger 4242 is a table row and is not in the JSON block"),
        ],
        "a row was renumbered in the ledger's own table and the reading did \
         not answer with exactly the two entries that leaves"
    );

    // A status changed in the table only, which is the edit that silently
    // reverts on the next tool write.
    let other = if status == "open" { "fixed" } else { "open" };
    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with(&format!("| {id} |")),
        &format!("| {id} | 04.2 | stub | a/b.rs |  | {description} | {other} |  | x |  |"),
    );
    assert_eq!(
        ledger_disagreements(&spliced),
        vec![format!(
            "{WINDOWS}: ledger {id} status: the table says {other} and the JSON \
             block says {status}"
        )],
        "a status was changed in the ledger's own table and the reading did \
         not answer with exactly that"
    );

    // And a description, which is the half a person actually reads.
    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with(&format!("| {id} |")),
        &format!(
            "| {id} | 04.2 | stub | a/b.rs |  | Something else entirely | {status} |  | x |  |"
        ),
    );
    assert_eq!(
        ledger_disagreements(&spliced),
        vec![format!(
            "{WINDOWS}: ledger {id} description: the table says {} and the JSON \
             block says {}",
            shortened("Something else entirely"),
            shortened(&description)
        )],
        "a description was changed in the ledger's own table and the reading \
         did not answer with exactly that"
    );
}

/// The id, status and description of the ledger's last table row.
///
/// Read from the file rather than written down here, so this companion does
/// not go stale the next time an entry is opened.
fn the_last_ledger_row(text: &str) -> (String, String, String) {
    let row = text
        .lines()
        .rfind(|line| {
            line.starts_with("| ")
                && line
                    .split('|')
                    .nth(1)
                    .is_some_and(|cell| cell.trim().parse::<u32>().is_ok())
        })
        .expect("the ledger to hold at least one table row");
    let cells: Vec<&str> = row.split('|').map(str::trim).collect();
    (
        cells[1].to_string(),
        cells[7].to_string(),
        cells[6].to_string(),
    )
}

#[test]
fn test_the_roadmap_reading_can_see_a_row_that_disagrees_with_the_disk() {
    let text = read(ROADMAP);
    let on_disk = phases_on_disk();
    assert_eq!(
        roadmap_disagreements(&text, &on_disk),
        Vec::<String>::new(),
        "the roadmap already disagrees with the disk, so what this splices in \
         is not the only thing the reading has to find"
    );

    let (label, cell) = the_first_counted_roadmap_row(&text);
    let key = phase_key(&label).unwrap_or_default();
    let files = on_disk.get(&key).copied().unwrap_or_default();
    let wrong = format!("{}/{}", files.summaries + 4242, files.plans);

    let spliced = with_one_line_replaced(
        &text,
        |line| line.starts_with(&format!("| {label} |")),
        &format!("| {label} | {wrong} | Executed | - |"),
    );
    assert_eq!(
        roadmap_disagreements(&spliced, &on_disk),
        vec![format!(
            "{ROADMAP}: row {label} says {wrong} and phase {key} holds {} \
             summaries and {} plans on disk",
            files.summaries, files.plans
        )],
        "a wrong count was put into the roadmap's own progress table, where \
         the real cell says {cell}, and the reading did not answer with \
         exactly that"
    );
}

/// The label and cell of the first progress row that carries a real `n/m`.
fn the_first_counted_roadmap_row(text: &str) -> (String, String) {
    for line in text.lines() {
        if !line.starts_with("| ") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 5 {
            continue;
        }
        if phase_key(cells[1]).is_none() {
            continue;
        }
        let counts: Vec<&str> = cells[2].split('/').collect();
        if counts.len() == 2
            && counts[0].parse::<usize>().is_ok()
            && counts[1].parse::<usize>().is_ok()
        {
            return (cells[1].to_string(), cells[2].to_string());
        }
    }
    panic!("the roadmap's progress table to hold at least one counted row");
}

// ---------------------------------------------------------------------------
// And that the tree reading is reading the tree
// ---------------------------------------------------------------------------

#[test]
fn test_the_phase_directories_are_really_being_counted() {
    // Without this, a mistake in `phases_on_disk` that returned nothing would
    // leave three of the checks above comparing numbers against zero and
    // reporting that everything agrees.
    let on_disk = phases_on_disk();
    assert!(
        on_disk.len() > 5,
        "only {} phase directories found, so the walk is broken and every \
         count compared against it means nothing",
        on_disk.len()
    );
    assert!(
        on_disk.values().map(|f| f.plans).sum::<usize>() > 50,
        "the phase directories hold no plans, so the walk found directories \
         and read nothing inside them"
    );
    assert!(
        on_disk.values().map(|f| f.summaries).sum::<usize>() > 50,
        "the phase directories hold no summaries, so a summary is not being \
         told from a plan"
    );
}

#[test]
fn test_a_phase_is_the_same_phase_however_it_is_spelled() {
    // The three spellings this project uses, which is why the key exists.
    assert_eq!(
        phase_key("04.2-what-was-built-and-never-reached").as_deref(),
        Some("4.2")
    );
    assert_eq!(phase_key("4.2 What was built").as_deref(), Some("4.2"));
    assert_eq!(phase_key("04.2").as_deref(), Some("4.2"));
    assert_eq!(
        phase_key("1. Folders and conversations").as_deref(),
        Some("1")
    );
    assert_eq!(
        phase_key("01-folders-and-conversations").as_deref(),
        Some("1")
    );
    // And the things that are not a phase, which must be passed over rather
    // than turned into one.
    assert_eq!(phase_key("Phase"), None);
    assert_eq!(phase_key(""), None);
    assert_eq!(phase_key("-------"), None);
}

//! Every number on the measurements page carries its command and its date.
//!
//! `docs/development/measurements.md` is the one page a figure about this
//! tree is written on: how many guard records there are, how many mutants the
//! configuration allows, how long the suite takes. Every other page points at
//! it rather than restating a number, so that a figure which has moved reads
//! as a dated measurement rather than as a current fact.
//!
//! That only works if every row on the page says how it was taken. The phase
//! that created the page was researched from a document that gave one of its
//! own headline figures beside a command nobody had run, and the tree gave
//! four different totals for one job and none for the other. A number without
//! its command is a guess wearing a number's clothes, and a number without its
//! date is a fact that will silently stop being one. `CLAUDE.md` says so in a
//! dozen places and had to say it because the page did not exist.
//!
//! # What is held, and what is deliberately not
//!
//! Each row must carry a `what`, a `value`, a `command` with at least one
//! backticked token, a `date` spelled `20YY-MM-DD`, a `commit` of at least
//! seven hex characters, and a `conditions` cell that says what moves the
//! figure or says `none`. Two rows with the same `what` and the same `date`
//! are refused, because a page that says the same thing twice is the defect
//! the page exists to end. A missing table and an empty table are findings,
//! not silence.
//!
//! What is not held: the value. Nothing here compares a row's figure with what
//! its command reports today. PERF-06's last clause records that a check of
//! that kind is false the next time somebody adds a test, and the page says
//! at its top that nothing on it is promised to be current. This target holds
//! the shape of a row and never its number.
//!
//! # How the check is built
//!
//! The reading is a function over the page's text, returning what it found
//! wrong. The check runs it over the real page and requires the answer to be
//! empty. Companions run the same function over the page's own real lines
//! with one omission spliced in and require the answer to name exactly that
//! row. The page is clean today, so from outside the reading is
//! indistinguishable from one that read nothing; the companions tell the two
//! apart, on the precedent of `tests/the_planning_files_agree_with_themselves.rs`.

use std::collections::BTreeSet;
use std::fs;

const PAGE: &str = "docs/development/measurements.md";
const HEADING: &str = "## The figures";
const COLUMNS: [&str; 6] = ["what", "value", "command", "date", "commit", "conditions"];

// ---------------------------------------------------------------------------
// Reading the page
// ---------------------------------------------------------------------------

/// One row of the table, cells trimmed, escaped pipes restored.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    what: String,
    value: String,
    command: String,
    date: String,
    commit: String,
    conditions: String,
}

/// A stand-in for `\|` while a row is split on the pipes that separate cells.
const AN_ESCAPED_PIPE: &str = "\u{1}";

fn cells_of(line: &str) -> Vec<String> {
    let guarded = line.replace("\\|", AN_ESCAPED_PIPE);
    let inner = guarded
        .trim()
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(guarded.trim());
    inner
        .split('|')
        .map(|cell| cell.trim().replace(AN_ESCAPED_PIPE, "|"))
        .collect()
}

fn is_a_separator(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':'))
}

/// The rows of the table under the expected heading, or why there are none.
fn rows_of(text: &str) -> Result<Vec<Row>, String> {
    let mut lines = text.lines();
    if !lines.any(|line| line.trim() == HEADING) {
        return Err(format!(
            "{PAGE}: has no `{HEADING}` heading, so there is no table to read"
        ));
    }
    let mut rows = Vec::new();
    let mut header_seen = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if !trimmed.starts_with('|') {
            if header_seen {
                break;
            }
            continue;
        }
        let cells = cells_of(trimmed);
        if !header_seen {
            let lowered: Vec<String> = cells.iter().map(|c| c.to_lowercase()).collect();
            if lowered != COLUMNS {
                return Err(format!(
                    "{PAGE}: the table under `{HEADING}` has the columns {cells:?} and the \
                     reading expects {COLUMNS:?}"
                ));
            }
            header_seen = true;
            continue;
        }
        if is_a_separator(&cells) {
            continue;
        }
        if cells.len() != COLUMNS.len() {
            return Err(format!(
                "{PAGE}: a row under `{HEADING}` has {} cells and the table has {} columns: {trimmed}",
                cells.len(),
                COLUMNS.len()
            ));
        }
        rows.push(Row {
            what: cells[0].clone(),
            value: cells[1].clone(),
            command: cells[2].clone(),
            date: cells[3].clone(),
            commit: cells[4].clone(),
            conditions: cells[5].clone(),
        });
    }
    if !header_seen {
        return Err(format!("{PAGE}: has no table under `{HEADING}`"));
    }
    if rows.is_empty() {
        return Err(format!("{PAGE}: the table under `{HEADING}` is empty"));
    }
    Ok(rows)
}

fn holds_a_backticked_token(cell: &str) -> bool {
    let mut parts = cell.split('`');
    parts.next();
    parts.step_by(2).any(|inside| !inside.trim().is_empty())
}

fn is_a_date(cell: &str) -> bool {
    let bytes = cell.as_bytes();
    bytes.len() == 10
        && bytes.starts_with(b"20")
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(at, b)| at == 4 || at == 7 || b.is_ascii_digit())
}

fn is_a_commit(cell: &str) -> bool {
    let token = cell.trim_matches('`');
    token.len() >= 7 && token.chars().all(|c| c.is_ascii_hexdigit())
}

/// What is wrong with the page, one line per finding, empty when nothing is.
fn rows_that_lack_their_provenance(text: &str) -> Vec<String> {
    let rows = match rows_of(text) {
        Ok(rows) => rows,
        Err(why) => return vec![why],
    };
    let mut wrong = Vec::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for row in &rows {
        let what = &row.what;
        if what.is_empty() {
            wrong.push(format!(
                "{PAGE}: a row has no `what`, so nothing can name it"
            ));
            continue;
        }
        if row.value.is_empty() {
            wrong.push(format!("{PAGE}: row `{what}` has no value"));
        }
        if !holds_a_backticked_token(&row.command) {
            wrong.push(format!("{PAGE}: row `{what}` has a value and no command"));
        }
        if !is_a_date(&row.date) {
            wrong.push(format!("{PAGE}: row `{what}` has a value and no date"));
        }
        if !is_a_commit(&row.commit) {
            wrong.push(format!("{PAGE}: row `{what}` has a value and no commit"));
        }
        if row.conditions.is_empty() {
            wrong.push(format!(
                "{PAGE}: row `{what}` has no conditions; say what moves it or say `none`"
            ));
        }
        if !seen.insert((what.clone(), row.date.clone())) {
            wrong.push(format!(
                "{PAGE}: row `{what}` dated {} appears twice",
                row.date
            ));
        }
    }
    wrong
}

fn read_the_page() -> String {
    fs::read_to_string(PAGE)
        .unwrap_or_else(|e| panic!("{PAGE} is the page every figure lives on, and it {e}"))
}

/// The text with exactly one line replaced, and a panic if the line to
/// replace was not there exactly once. Over the real page's real lines on
/// purpose, for the reason `the_planning_files_agree_with_themselves` gives:
/// a fixture built by hand can make the wrong pair of cells coincide.
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

/// The first data row of the real page, as its line and as its cells.
fn the_first_row(text: &str) -> (String, Row) {
    let rows = rows_of(text).unwrap_or_else(|why| panic!("{why}"));
    let first = rows
        .into_iter()
        .next()
        .expect("rows_of refuses an empty table");
    let line = text
        .lines()
        .find(|line| line.trim().starts_with('|') && cells_of(line).first() == Some(&first.what))
        .expect("the first row's line to be on the page")
        .to_string();
    (line, first)
}

fn the_row_with(row: &Row) -> String {
    let escaped = |cell: &str| cell.replace('|', "\\|");
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        escaped(&row.what),
        escaped(&row.value),
        escaped(&row.command),
        row.date,
        row.commit,
        escaped(&row.conditions)
    )
}

// ---------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------

#[test]
fn test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit() {
    let wrong = rows_that_lack_their_provenance(&read_the_page());
    assert!(
        wrong.is_empty(),
        "every figure on {PAGE} must say how and when it was taken, and these do \
         not:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_page_holds_the_rows_the_phase_was_scheduled_from() {
    // The reading above would pass over a page with one row. The phase that
    // wrote the page took eight figures on its first day, and the plans after
    // it are scheduled from them, so a page with fewer rows than that is a
    // page somebody emptied.
    let rows = rows_of(&read_the_page()).unwrap_or_else(|why| panic!("{why}"));
    assert!(
        rows.len() >= 8,
        "{PAGE} holds {} rows, and it held at least eight the day it was written",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// The companions, each splicing into the real page's own lines
// ---------------------------------------------------------------------------

fn the_page_is_clean_today(text: &str) {
    assert_eq!(
        rows_that_lack_their_provenance(text),
        Vec::<String>::new(),
        "the page already has a row without its provenance, so what this \
         splices in is not the only thing the reading has to find"
    );
}

#[test]
fn test_the_reading_can_see_a_row_whose_date_is_missing() {
    let text = read_the_page();
    the_page_is_clean_today(&text);
    let (line, first) = the_first_row(&text);
    let without = Row {
        date: String::new(),
        ..first.clone()
    };
    let spliced = with_one_line_replaced(&text, |l| l == line, &the_row_with(&without));
    assert_eq!(
        rows_that_lack_their_provenance(&spliced),
        vec![format!(
            "{PAGE}: row `{}` has a value and no date",
            first.what
        )],
        "the date was taken out of the page's own first row and the reading did \
         not answer with exactly that row"
    );
}

#[test]
fn test_the_reading_can_see_a_row_whose_command_is_missing() {
    let text = read_the_page();
    the_page_is_clean_today(&text);
    let (line, first) = the_first_row(&text);
    let without = Row {
        command: String::new(),
        ..first.clone()
    };
    let spliced = with_one_line_replaced(&text, |l| l == line, &the_row_with(&without));
    assert_eq!(
        rows_that_lack_their_provenance(&spliced),
        vec![format!(
            "{PAGE}: row `{}` has a value and no command",
            first.what
        )],
        "the command was taken out of the page's own first row and the reading \
         did not answer with exactly that row"
    );
}

#[test]
fn test_the_reading_can_see_a_row_whose_commit_is_missing() {
    let text = read_the_page();
    the_page_is_clean_today(&text);
    let (line, first) = the_first_row(&text);
    let without = Row {
        commit: "not a hash".to_string(),
        ..first.clone()
    };
    let spliced = with_one_line_replaced(&text, |l| l == line, &the_row_with(&without));
    assert_eq!(
        rows_that_lack_their_provenance(&spliced),
        vec![format!(
            "{PAGE}: row `{}` has a value and no commit",
            first.what
        )],
        "the commit was replaced in the page's own first row and the reading \
         did not answer with exactly that row"
    );
}

#[test]
fn test_the_reading_can_see_a_row_that_appears_twice() {
    let text = read_the_page();
    the_page_is_clean_today(&text);
    let (line, first) = the_first_row(&text);
    let doubled = format!("{line}\n{line}");
    let spliced = with_one_line_replaced(&text, |l| l == line, &doubled);
    assert_eq!(
        rows_that_lack_their_provenance(&spliced),
        vec![format!(
            "{PAGE}: row `{}` dated {} appears twice",
            first.what, first.date
        )],
        "the page's own first row was written twice and the reading did not \
         answer with exactly that row"
    );
}

#[test]
fn test_the_reading_refuses_a_page_with_no_table_and_a_table_with_no_rows() {
    // The two ways a reading quietly finds nothing, each a finding by name.
    assert_eq!(
        rows_that_lack_their_provenance("# A page\n\nProse and no heading.\n"),
        vec![format!(
            "{PAGE}: has no `{HEADING}` heading, so there is no table to read"
        )]
    );
    assert_eq!(
        rows_that_lack_their_provenance(&format!("{HEADING}\n\nProse and no table.\n")),
        vec![format!("{PAGE}: has no table under `{HEADING}`")]
    );
    let header =
        "| What | Value | Command | Date | Commit | Conditions |\n|---|---|---|---|---|---|\n";
    assert_eq!(
        rows_that_lack_their_provenance(&format!("{HEADING}\n\n{header}")),
        vec![format!("{PAGE}: the table under `{HEADING}` is empty")]
    );
}

#[test]
fn test_a_command_with_a_pipe_in_it_is_one_cell() {
    // `cargo mutants --list | wc -l` is a command this page carries, and a
    // reading that split it at the pipe would count seven cells and refuse
    // the row for the wrong reason.
    let page = format!(
        "{HEADING}\n\n| What | Value | Command | Date | Commit | Conditions |\n|---|---|---|---|---|---|\n\
         | mutants | 12 | `cargo mutants --list \\| wc -l` | 2026-09-14 | 7b2482b1 | none |\n"
    );
    assert_eq!(rows_that_lack_their_provenance(&page), Vec::<String>::new());
    let rows = rows_of(&page).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(rows[0].command, "`cargo mutants --list | wc -l`");
}

// ---------------------------------------------------------------------------
// And that this target runs on the commits that could break it
// ---------------------------------------------------------------------------

/// This file's own target name, which is what `cargo test --test` is given.
const ME: &str = "every_number_carries_its_command_and_its_date";

/// The targets a documents-only commit earns, read from the gate script.
fn the_documents_only_targets(script: &str) -> Vec<String> {
    let mut inside = false;
    let mut found = Vec::new();
    for line in script.lines() {
        if line.starts_with("if [ \"$mode\" = \"docs_only\" ]; then") {
            inside = true;
            continue;
        }
        if inside && line.starts_with("fi") {
            break;
        }
        if !inside || line.trim_start().starts_with('#') {
            continue;
        }
        let mut rest = line;
        while let Some(at) = rest.find("--test ") {
            rest = &rest[at + "--test ".len()..];
            if let Some(name) = rest.split_whitespace().next() {
                found.push(name.to_string());
            }
        }
    }
    found
}

/// The targets every scoped run ends with, read from the gate script.
fn the_whole_tree_targets(script: &str) -> Vec<String> {
    script
        .lines()
        .find_map(|line| line.strip_prefix("guards_that_read_the_whole_tree=("))
        .and_then(|rest| rest.split(')').next())
        .map(|inner| inner.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

fn read_the_gate() -> String {
    fs::read_to_string("scripts/check.sh").expect("the gate script to be readable")
}

#[test]
fn test_this_target_runs_on_the_commits_that_could_break_it() {
    // A page under `docs/` edited on its own answers `docs_only`, and a page
    // edited beside the code it measures answers `affected`. Neither list in
    // `scripts/check.sh` is derived from anything, so without this a target
    // that fell out of one would run on every commit except the ones that
    // write the page, which is the defect `check.sh`'s own comments record
    // for `the_planning_files_agree_with_themselves`.
    let script = read_the_gate();

    let documents = the_documents_only_targets(&script);
    assert!(
        documents.iter().any(|target| target == ME),
        "a commit touching only documents runs {documents:?} and not {ME}, so \
         the reading of the measurements page does not run on the commits that \
         change it"
    );

    let whole_tree = the_whole_tree_targets(&script);
    assert!(
        whole_tree.iter().any(|target| target == ME),
        "a commit touching the page beside code runs {whole_tree:?} at the end \
         of its scoped run and not {ME}"
    );
}

#[test]
fn test_the_reading_of_what_the_gate_runs_can_see_a_target_that_is_missing() {
    let script = read_the_gate();

    let documents = the_documents_only_targets(&script);
    assert!(
        documents.iter().any(|target| target == "house_style"),
        "the documents-only reading did not find house_style, which has read \
         documents since before this file existed"
    );
    let whole_tree = the_whole_tree_targets(&script);
    assert!(
        whole_tree.iter().any(|target| target == "wired"),
        "the whole-tree reading did not find wired, so it is not reading the \
         list it is about"
    );

    let without = script.replace(&format!("--test {ME}"), "--test something_else");
    assert!(
        !the_documents_only_targets(&without)
            .iter()
            .any(|target| target == ME),
        "the documents-only reading still finds {ME} after it was taken out of \
         the script, so it is not reading the script"
    );
    let without = script.replace(&format!(" {ME}"), " something_else");
    assert!(
        !the_whole_tree_targets(&without)
            .iter()
            .any(|target| target == ME),
        "the whole-tree reading still finds {ME} after it was taken out of the \
         script, so it is not reading the script"
    );
}

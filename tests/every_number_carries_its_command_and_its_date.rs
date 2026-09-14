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
//!
//! # The other readings in this file
//!
//! Three more readings follow the same shape, each with its own section
//! comment and companions. One walks every page a person believes and
//! requires a count, a percentage or a duration to sit beside a date and a
//! source. One holds a test count stated on the three pages that state one
//! to a row on the measurements page, so pages agree by quoting one
//! measurement and never each other. One computes how much of the history
//! predates red/green, prints it, and refuses the two-absolutes form that
//! used to state it. And one holds twelve figures in prose to the constants
//! they restate, which is the one kind of number both halves of which are in
//! the repository. None of them compares a written figure with what a tool
//! reports today, for the reason above.

use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use wixen_mail::application::attaching::LIMIT_BYTES;
use wixen_mail::application::pictures::MOST_ONE_PICTURE_MAY_BE;
use wixen_mail::data::message_cache::attachment_content::{
    ATTACHMENT_CACHE_BUDGET_BYTES, LARGEST_ATTACHMENT_KEPT_BYTES,
};
use wixen_mail::data::message_cache::moves_in_flight::{
    LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES, MOVES_IN_FLIGHT_BUDGET_BYTES,
};
use wixen_mail::data::message_cache::signed_original::{
    LARGEST_SIGNED_MESSAGE_KEPT_BYTES, SIGNED_ORIGINAL_BUDGET_BYTES,
};
use wixen_mail::presentation::accessibility::sound_scheme_import::{
    MAX_FILE_BYTES, MAX_SOUND_DURATION, MAX_TOTAL_BYTES, MAX_ZIP_BYTES,
};

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

// ---------------------------------------------------------------------------
// A figure on any page carries its date and its source
// ---------------------------------------------------------------------------
//
// The page above is where a figure about the tree is written. This reading is
// about every other page: a count of tests, records, mutants or lines, a
// coverage percentage, or a duration of the gate, the suite, the sweep, a
// build or a run, wherever a person could read it and believe it. Such a
// paragraph must say when the figure was taken and how, or it must say it is
// a target.
//
// What is held: the presence of a `20YY-MM-DD` date and of a source beside the
// figure, in the same paragraph. A source is a backticked token, a mention of
// the measurements page, or the name of a test.
//
// What is not held, on purpose: that any figure is current. Nothing here runs
// a command and compares. PERF-06's last clause, corrected 2026-08-29, records
// that a check asserting a written count equals today's count is false the
// next time somebody adds a test, and this project was researched from a
// document that had done exactly that. A figure that has moved is meant to
// read as a dated measurement, which is what the date is for.
//
// Which pages, and why. `docs/**/*.md`, `README.md` and `CLAUDE.md`, less two
// things. `docs/changelog.md` is dated by its headings and `docs/plans/` by
// its file names, so every figure in them already carries its date and they
// are records rather than claims. `.planning/` is out by genre, on the
// precedent `tests/house_style.rs` records for the rules about what the
// product claims: a plan says what a feature should do, and a person does not
// read it to learn what the tree holds.

/// A page in the walk: its path as written in a complaint, and its text.
type Page = (String, String);

/// Where the pages under `docs/` are read from. A companion points this at a
/// directory that does not exist to prove the walk refuses to be empty.
const THE_DOCS: &str = "docs";

/// The one changelog and the one directory of dated plans, left out of the
/// walk for the reason in the section comment.
fn is_a_dated_record(path: &Path) -> bool {
    path == Path::new("docs/changelog.md") || path.starts_with("docs/plans")
}

fn markdown_under(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            markdown_under(&path, into);
        } else if path.extension().is_some_and(|e| e == "md") && !is_a_dated_record(&path) {
            into.push(path);
        }
    }
}

/// The pages a figure may sit on, read from `docs_root` and the two files at
/// the top of the tree. Sorted, so a complaint is in the same order every run.
fn the_pages_a_figure_may_sit_on(docs_root: &Path) -> Vec<PathBuf> {
    let mut pages = Vec::new();
    markdown_under(docs_root, &mut pages);
    for single in ["README.md", "CLAUDE.md"] {
        let path = PathBuf::from(single);
        if path.exists() {
            pages.push(path);
        }
    }
    pages.sort();
    pages
}

fn read_the_pages(docs_root: &Path) -> Vec<Page> {
    the_pages_a_figure_may_sit_on(docs_root)
        .into_iter()
        .map(|path| {
            let name = path.to_string_lossy().replace('\\', "/");
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{name} is in the walk and it {e}"));
            (name, text)
        })
        .collect()
}

/// A paragraph of a page, with the number of the line it starts on.
struct Paragraph {
    first_line: usize,
    text: String,
}

/// The paragraphs of a page, split at blank lines, each knowing where it
/// starts so a complaint can name a line rather than a paragraph.
fn paragraphs_of(text: &str) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut started_at = 0;
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(Paragraph {
                    first_line: started_at + 1,
                    text: current.join("\n"),
                });
                current.clear();
            }
            continue;
        }
        if current.is_empty() {
            started_at = index;
        }
        current.push(line);
    }
    if !current.is_empty() {
        paragraphs.push(Paragraph {
            first_line: started_at + 1,
            text: current.join("\n"),
        });
    }
    paragraphs
}

/// A figure of one of the five kinds, and the line of its paragraph it sits on.
struct Figure {
    line: usize,
    words: String,
}

/// The number must not be the tail of something else: `PERF-06 records` is a
/// requirement being cited, not six records, and `-` before the digit is how
/// the reading tells them apart. The regex crate has no lookbehind, so the
/// character before the digit is matched and dropped.
const NOT_PART_OF_A_WORD: &str = r"(?:^|[^-\w])";

/// A pattern compiled once for the process. Compiling one costs milliseconds
/// and the walk reads thousands of paragraphs, so compiling per paragraph put
/// each companion past a minute.
fn compiled_once(
    cell: &'static OnceLock<Regex>,
    pattern: impl FnOnce() -> String,
) -> &'static Regex {
    cell.get_or_init(|| Regex::new(&pattern()).expect("every pattern here is a literal"))
}

fn the_count_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || {
        format!(
            r"{NOT_PART_OF_A_WORD}(\d[\d,]*\s+(?:tests|test functions|guard records|records|mutants|lines))\b"
        )
    })
}

fn the_coverage_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || {
        r"(\d+(?:\.\d+)?\s?%\s*(?:line )?coverage)|(coverage[^.]{0,40}?\d+(?:\.\d+)?\s?%)"
            .to_string()
    })
}

fn the_duration_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || {
        let jobs = r"(?:gate|sweep|suite|build|run|library run|whole gate|full gate|mutation run|whole-tree run)";
        let span = r"\d+\s+(?:seconds|minutes|hours|days)";
        format!(r"(\b{jobs}\b[^.]{{0,80}}?\b{span}\b)|(\b{span}\b[^.]{{0,60}}?\b{jobs}\b)")
    })
}

fn the_date_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || r"\b20\d\d-\d\d-\d\d\b".to_string())
}

fn the_test_name_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || r"\btest_[a-z_]+".to_string())
}

/// The figures of the five kinds in a paragraph, each with the line it sits on.
fn figures_in(paragraph: &str) -> Vec<Figure> {
    let mut found = Vec::new();
    for pattern in [
        the_count_pattern(),
        the_coverage_pattern(),
        the_duration_pattern(),
    ] {
        for captures in pattern.captures_iter(paragraph) {
            let Some(hit) = (1..captures.len()).find_map(|group| captures.get(group)) else {
                continue;
            };
            found.push(Figure {
                line: paragraph[..hit.start()].matches('\n').count(),
                words: hit
                    .as_str()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            });
        }
    }
    found.sort_by_key(|figure| figure.line);
    found
}

fn holds_a_date(paragraph: &str) -> bool {
    the_date_pattern().is_match(paragraph)
}

/// A backticked token, the measurements page by name, or a test by name.
fn holds_a_source(paragraph: &str) -> bool {
    holds_a_backticked_token(paragraph)
        || paragraph.contains(PAGE)
        || the_test_name_pattern().is_match(paragraph)
}

/// A line that says `target` is stating an aim, not a measurement, and an aim
/// has no date of being taken.
fn names_a_target(line: &str) -> bool {
    line.to_lowercase().contains("target")
}

/// The walk must read at least this many pages before its answer counts. Two
/// come from the top of the tree, so a walk that lost `docs/` reads two.
const THE_FEWEST_PAGES_A_WALK_MAY_READ: usize = 5;

/// What is wrong across the pages, one line per figure, empty when nothing is.
/// A walk that read too few pages or found no figure of any kind is itself
/// the finding, so a renamed directory cannot turn this green.
fn figures_that_lack_their_provenance(pages: &[Page]) -> Vec<String> {
    if pages.len() < THE_FEWEST_PAGES_A_WALK_MAY_READ {
        return vec![format!(
            "the walk read {} pages, and it reads {} at the least when `docs/` is where it \
             was, so it is not looking at the tree",
            pages.len(),
            THE_FEWEST_PAGES_A_WALK_MAY_READ
        )];
    }
    let mut wrong = Vec::new();
    let mut figures_seen = 0;
    for (name, text) in pages {
        for paragraph in paragraphs_of(text) {
            let lines: Vec<&str> = paragraph.text.lines().collect();
            let dated = holds_a_date(&paragraph.text);
            let sourced = holds_a_source(&paragraph.text);
            for figure in figures_in(&paragraph.text) {
                figures_seen += 1;
                if names_a_target(lines[figure.line]) {
                    continue;
                }
                let missing = match (dated, sourced) {
                    (true, true) => continue,
                    (false, true) => "no date",
                    (true, false) => "no command or named source",
                    (false, false) => "no date and no command or named source",
                };
                wrong.push(format!(
                    "{name}:{}: \"{}\" has {missing} beside it",
                    paragraph.first_line + figure.line,
                    figure.words
                ));
            }
        }
    }
    if figures_seen == 0 {
        return vec![format!(
            "the walk read {} pages and found no figure of any kind on them, and the \
             tree states dozens, so it is not reading what it thinks it is",
            pages.len()
        )];
    }
    wrong
}

#[test]
fn test_every_figure_on_a_page_carries_its_date_and_its_source() {
    let wrong = figures_that_lack_their_provenance(&read_the_pages(Path::new(THE_DOCS)));
    assert!(
        wrong.is_empty(),
        "a count, a percentage or a duration on a page somebody believes must say when it \
         was taken and by what, or say it is a target, and these do not:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_provenance_reading_is_looking_at_the_pages_it_is_for() {
    // The precedent is `test_the_check_is_looking_at_the_whole_project` in
    // `tests/house_style.rs`: a walk that returned nothing would leave the
    // reading above passing on an empty list, so the list is held here.
    let pages = the_pages_a_figure_may_sit_on(Path::new(THE_DOCS));
    let has = |tail: &str| pages.iter().any(|p| p.ends_with(tail));
    assert!(
        pages.len() >= THE_FEWEST_PAGES_A_WALK_MAY_READ,
        "only {} pages walked, so the walk is broken",
        pages.len()
    );
    assert!(
        has("privacy.md"),
        "the pages under docs/ are not being read"
    );
    assert!(
        has("IMPLEMENTATION_STATUS.md"),
        "the status page, which states the test count, is not being read"
    );
    assert!(
        has("README.md") && has("CLAUDE.md"),
        "the two pages at the top are not being read"
    );
    assert!(
        !pages.iter().any(|p| p.starts_with(".planning")),
        "a planning file reached a rule about what the product claims"
    );
    assert!(
        !has("changelog.md") && !pages.iter().any(|p| p.starts_with("docs/plans")),
        "a page dated by its headings or its file name is being asked for a date"
    );
}

// ---------------------------------------------------------------------------
// The companions, each splicing into a real page's own lines
// ---------------------------------------------------------------------------

fn the_pages_are_clean_today(pages: &[Page]) {
    assert_eq!(
        figures_that_lack_their_provenance(pages),
        Vec::<String>::new(),
        "a page already has a figure without its provenance, so what this splices in is \
         not the only thing the reading has to find"
    );
}

/// The real pages with one page's text swapped for `spliced`.
fn with_the_page_replaced(pages: &[Page], name: &str, spliced: String) -> Vec<Page> {
    let mut swapped = pages.to_vec();
    let at = swapped
        .iter()
        .position(|(page, _)| page == name)
        .unwrap_or_else(|| panic!("{name} is not in the walk, so nothing can be spliced into it"));
    swapped[at].1 = spliced;
    swapped
}

/// A real page's first paragraph that holds no date, no source and no figure:
/// the place a sentence can be spliced so that only the sentence is found.
fn a_bare_paragraph_of(text: &str) -> Paragraph {
    paragraphs_of(text)
        .into_iter()
        .find(|paragraph| {
            !holds_a_date(&paragraph.text)
                && !holds_a_source(&paragraph.text)
                && figures_in(&paragraph.text).is_empty()
                && !paragraph.text.lines().any(names_a_target)
        })
        .expect("the page to have one paragraph of plain prose")
}

/// The page the provenance companions splice into. It is a page a person
/// reads first, so a figure planted on it is the realistic case.
const A_PAGE_TO_SPLICE: &str = "README.md";

/// `sentence` appended to the first line of a bare paragraph of the README,
/// and the line number the figure will be reported on.
fn the_readme_with(pages: &[Page], sentence: &str) -> (Vec<Page>, usize) {
    let text = &pages
        .iter()
        .find(|(name, _)| name == A_PAGE_TO_SPLICE)
        .expect("the README to be in the walk")
        .1;
    let bare = a_bare_paragraph_of(text);
    let first = bare
        .text
        .lines()
        .next()
        .expect("a paragraph has a first line");
    let spliced =
        with_one_line_replaced(text, |line| line == first, &format!("{first} {sentence}"));
    (
        with_the_page_replaced(pages, A_PAGE_TO_SPLICE, spliced),
        bare.first_line,
    )
}

#[test]
fn test_the_provenance_reading_can_see_a_count_without_a_date() {
    let pages = read_the_pages(Path::new(THE_DOCS));
    the_pages_are_clean_today(&pages);
    let (spliced, line) = the_readme_with(&pages, "The suite holds 1,234 tests.");
    assert_eq!(
        figures_that_lack_their_provenance(&spliced),
        vec![format!(
            "{A_PAGE_TO_SPLICE}:{line}: \"1,234 tests\" has no date and no command or named \
             source beside it"
        )],
        "a bare test count was planted in the README's own prose and the reading did not \
         answer with exactly that figure"
    );
}

#[test]
fn test_the_provenance_reading_can_see_a_percentage_without_a_date() {
    let pages = read_the_pages(Path::new(THE_DOCS));
    the_pages_are_clean_today(&pages);
    let (spliced, line) = the_readme_with(
        &pages,
        "Line coverage stands at 61.2% with `cargo llvm-cov`.",
    );
    assert_eq!(
        figures_that_lack_their_provenance(&spliced),
        vec![format!(
            "{A_PAGE_TO_SPLICE}:{line}: \"coverage stands at 61.2%\" has no date beside it"
        )],
        "a coverage figure with a command and no date was planted and the reading did not \
         answer with exactly that figure"
    );
}

#[test]
fn test_the_provenance_reading_can_see_a_duration_without_a_source() {
    let pages = read_the_pages(Path::new(THE_DOCS));
    the_pages_are_clean_today(&pages);
    let (spliced, line) = the_readme_with(&pages, "On 2026-09-14 the gate took 311 seconds.");
    assert_eq!(
        figures_that_lack_their_provenance(&spliced),
        vec![format!(
            "{A_PAGE_TO_SPLICE}:{line}: \"gate took 311 seconds\" has no command or named \
             source beside it"
        )],
        "a dated duration with no command was planted and the reading did not answer with \
         exactly that figure"
    );
}

#[test]
fn test_the_provenance_reading_excuses_a_line_that_names_a_target() {
    let pages = read_the_pages(Path::new(THE_DOCS));
    the_pages_are_clean_today(&pages);
    let (spliced, _) = the_readme_with(&pages, "The target is 9,000 tests by the end of the year.");
    assert_eq!(
        figures_that_lack_their_provenance(&spliced),
        Vec::<String>::new(),
        "a line that says it is a target was asked for the date it was taken"
    );
    // And the same sentence without the word is found, so the silence above
    // is the excuse working and not the reading missing the figure.
    let (spliced, line) = the_readme_with(&pages, "The aim is 9,000 tests by the end of the year.");
    assert_eq!(
        figures_that_lack_their_provenance(&spliced).len(),
        1,
        "the same figure without the word target was not found on line {line}"
    );
}

#[test]
fn test_the_provenance_reading_refuses_an_empty_walk() {
    // A walk whose `docs/` has gone reads the two pages at the top and no more.
    let two = read_the_pages(Path::new("a-directory-that-is-not-there"));
    assert_eq!(
        figures_that_lack_their_provenance(&two),
        vec![format!(
            "the walk read {} pages, and it reads {THE_FEWEST_PAGES_A_WALK_MAY_READ} at the \
             least when `docs/` is where it was, so it is not looking at the tree",
            two.len()
        )],
        "the walk lost docs/ and the reading did not say so"
    );
    // And enough pages holding no figure at all is the other silence.
    let prose: Vec<Page> = (0..THE_FEWEST_PAGES_A_WALK_MAY_READ)
        .map(|n| {
            (
                format!("page-{n}.md"),
                "Prose with nothing to count.\n".to_string(),
            )
        })
        .collect();
    assert_eq!(
        figures_that_lack_their_provenance(&prose),
        vec![format!(
            "the walk read {THE_FEWEST_PAGES_A_WALK_MAY_READ} pages and found no figure of \
             any kind on them, and the tree states dozens, so it is not reading what it \
             thinks it is"
        )],
        "pages with no figure on them were read as pages with nothing wrong"
    );
}

// ---------------------------------------------------------------------------
// The three pages that state the test count quote one row
// ---------------------------------------------------------------------------
//
// PERF-06's third clause. The three pages used to agree with each other by
// copying each other, which is how three of them came to carry one stale
// figure together. Now each is compared with the measurements page and never
// with another page: a count of tests stated on one of them, in a paragraph
// that carries a date, is the value of some row whose `what` is about tests.
//
// A count is stated in the shape `N tests`, `N unit tests` or `N integration
// tests`. A past count on these three pages is therefore written some other
// way, "the count was N" or "said N until", because this reading cannot tell a
// present claim from a past one and these are the pages that state the present
// one. The changelog, where the past counts live, is not read.

const THE_PAGES_THAT_STATE_THE_TEST_COUNT: [&str; 3] = [
    "docs/IMPLEMENTATION_STATUS.md",
    "docs/integration-guide.md",
    "README.md",
];

fn the_test_count_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || {
        format!(r"{NOT_PART_OF_A_WORD}(\d[\d,]*) (?:(?:unit|integration) )?tests\b")
    })
}

/// A cell that is nothing but a count, commas dropped: `7,245` is one and `104
/// s and 103 s` is not, so a row timing `cargo test` cannot stand in for a row
/// counting tests.
fn a_bare_count(cell: &str) -> Option<u64> {
    if cell.is_empty() || !cell.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return None;
    }
    cell.replace(',', "").parse().ok()
}

/// A row that counts tests says so in its first word: `Tests the library
/// builds`, `Test functions in ...`. A row about guard records that happens
/// to name `tests_last_seen` does not, and a substring search took it for one.
fn counts_tests(row: &Row) -> bool {
    row.what
        .split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .is_some_and(|first| matches!(first.to_lowercase().as_str(), "test" | "tests"))
}

/// The values of the rows on the measurements page that count tests.
fn the_test_count_rows(page: &str) -> Vec<(String, u64)> {
    rows_of(page)
        .unwrap_or_else(|why| panic!("{why}"))
        .into_iter()
        .filter(counts_tests)
        .filter_map(|row| a_bare_count(&row.value).map(|value| (row.what, value)))
        .collect()
}

/// Every test count stated on the three pages that is not a row's value.
fn test_counts_that_quote_no_row(pages: &[Page], measurements: &str) -> Vec<String> {
    let rows = the_test_count_rows(measurements);
    let pattern = the_test_count_pattern();
    let mut wrong = Vec::new();
    for name in THE_PAGES_THAT_STATE_THE_TEST_COUNT {
        let text = &pages
            .iter()
            .find(|(page, _)| page == name)
            .unwrap_or_else(|| panic!("{name} states the test count and is not in the walk"))
            .1;
        for paragraph in paragraphs_of(text) {
            if !holds_a_date(&paragraph.text) {
                continue;
            }
            for captures in pattern.captures_iter(&paragraph.text) {
                let whole = captures.get(0).expect("a match has a whole");
                let stated = captures.get(1).expect("the count is captured");
                let count = a_bare_count(stated.as_str()).expect("the pattern captures digits");
                if rows.iter().any(|(_, value)| *value == count) {
                    continue;
                }
                let line =
                    paragraph.first_line + paragraph.text[..whole.start()].matches('\n').count();
                let could_have_been: Vec<String> = rows
                    .iter()
                    .map(|(what, value)| format!("{value} ({what})"))
                    .collect();
                wrong.push(format!(
                    "{name}:{line}: \"{}\" is not the value of any row on {PAGE} about tests; \
                     the rows hold {}",
                    whole.as_str().trim(),
                    could_have_been.join(", ")
                ));
            }
        }
    }
    wrong
}

#[test]
fn test_the_three_pages_that_state_the_test_count_quote_one_row() {
    let wrong =
        test_counts_that_quote_no_row(&read_the_pages(Path::new(THE_DOCS)), &read_the_page());
    assert!(
        wrong.is_empty(),
        "a test count stated on a page is the value of a row on {PAGE}, taken by the \
         command in that row, and these are not:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_agreement_reading_can_see_a_count_no_row_holds() {
    let pages = read_the_pages(Path::new(THE_DOCS));
    let measurements = read_the_page();
    assert_eq!(
        test_counts_that_quote_no_row(&pages, &measurements),
        Vec::<String>::new(),
        "a page already states a count no row holds, so what this splices in is not the \
         only thing the reading has to find"
    );
    let rows = the_test_count_rows(&measurements);
    assert!(
        rows.len() >= 2,
        "the measurements page holds {} rows about tests, and the reading needs at least the \
         library and the whole to compare against",
        rows.len()
    );
    let status = "docs/IMPLEMENTATION_STATUS.md";
    let text = &pages
        .iter()
        .find(|(name, _)| name == status)
        .expect("the status page")
        .1;
    let dated = paragraphs_of(text)
        .into_iter()
        .find(|paragraph| holds_a_date(&paragraph.text))
        .expect("the status page to hold a dated paragraph");
    let first = dated.text.lines().next().expect("a first line");
    let spliced = with_one_line_replaced(
        text,
        |line| line == first,
        &format!("{first} It ran 9,999 tests."),
    );
    let could_have_been: Vec<String> = rows
        .iter()
        .map(|(what, value)| format!("{value} ({what})"))
        .collect();
    assert_eq!(
        test_counts_that_quote_no_row(
            &with_the_page_replaced(&pages, status, spliced),
            &measurements
        ),
        vec![format!(
            "{status}:{}: \"9,999 tests\" is not the value of any row on {PAGE} about tests; \
             the rows hold {}",
            dated.first_line,
            could_have_been.join(", ")
        )],
        "a count no row holds was planted in the status page's own dated paragraph and the \
         reading did not answer with exactly that figure"
    );
}

// ---------------------------------------------------------------------------
// The share of the history before red/green is computed, never written
// ---------------------------------------------------------------------------
//
// "Red/green started at commit 182 of 344" was true when it was written, on
// 2026-07-29, and it is still true: the practice did start at the 182nd
// commit, and the repository did hold 344 before that one. What the sentence
// was written to support, that most of the history predates the practice,
// was 53% then and is under 9% now, and neither number moved. A ratio written
// as two absolutes is the worst kind of figure this tree has held, because
// nothing about it can go stale and the conclusion drawn from it inverts.
//
// So the share is computed here, on every commit, and printed with the day.
// No value of it is asserted, because it moves with every commit, which is
// the whole reason it is computed rather than written. What is asserted is
// that git could answer, that the pinned commit is in the history, that the
// count before it is smaller than the count in all, and that none of the
// four sites which used to state two absolutes states them again.
//
// The commit is pinned by hash and not by position. `rev-list` order is a
// property of the history's shape and a merge could move the 182nd commit;
// the hash was found by counting at the commit that wrote the sentence and
// is `18a02454`, which added `CLAUDE.md` with the red/green rule as its first
// rule. That is a better reason to call it the start than its position.

/// The commit red/green started at: the one that added `CLAUDE.md`, on
/// 2026-07-26, found by `git rev-list --reverse 3f7ebd09 | sed -n '182p'`
/// where `3f7ebd09` is the commit that first wrote the sentence.
const THE_COMMIT_RED_GREEN_STARTED_AT: &str = "18a02454";

/// The four places in the tree that used to state the ratio as two absolutes.
/// The two planning validation records that also carry it are records of
/// what was believed on their day and are left as written.
const THE_SITES_THAT_STATED_THE_RATIO: [&str; 4] = [
    "CLAUDE.md",
    "docs/IMPLEMENTATION_STATUS.md",
    ".cargo/mutants.toml",
    "scripts/mutants.sh",
];

/// What `git` answers, or why it could not.
fn git(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git could not be run, so the history is unread: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` refused: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn commits_counted_to(revision: &str) -> Result<u64, String> {
    let answer = git(&["rev-list", "--count", revision])?;
    answer
        .parse()
        .map_err(|_| format!("`git rev-list --count {revision}` answered `{answer}`, not a count"))
}

/// How many commits predate red/green, and how many there are in all.
fn the_share_of_history_before_red_green() -> Result<(u64, u64), String> {
    git(&[
        "merge-base",
        "--is-ancestor",
        THE_COMMIT_RED_GREEN_STARTED_AT,
        "HEAD",
    ])
    .map_err(|why| {
        format!(
            "{THE_COMMIT_RED_GREEN_STARTED_AT} is not in this history, so there is nothing to \
                 compute a share of: {why}"
        )
    })?;
    let before = commits_counted_to(&format!("{THE_COMMIT_RED_GREEN_STARTED_AT}^"))?;
    let in_all = commits_counted_to("HEAD")?;
    if before >= in_all {
        return Err(format!(
            "{before} commits predate {THE_COMMIT_RED_GREEN_STARTED_AT} and the history holds \
             {in_all}, which cannot be"
        ));
    }
    Ok((before, in_all))
}

/// A line with its indentation and its comment marker taken off, so the
/// sentence in a script or a configuration reads as the sentence.
fn unwrapped(line: &str) -> &str {
    let bare = line.trim_start();
    ["#", "//"]
        .iter()
        .find_map(|marker| bare.strip_prefix(marker))
        .unwrap_or(bare)
        .trim()
}

fn the_two_absolutes_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    compiled_once(&CELL, || r"\bcommit \d+ of \d+\b".to_string())
}

/// Where a file states the ratio as two absolutes, if it does: the line the
/// match starts on and the words. The lines are joined with a space after
/// unwrapping, and runs of whitespace collapsed, so a sentence wrapped across
/// two comment lines is one sentence. Read line by line, this missed
/// `scripts/mutants.sh`, whose sentence broke between the two numbers, and
/// the wrapped companion was red until the join was written.
fn the_two_absolutes_in(text: &str) -> Option<(usize, String)> {
    let mut joined = String::new();
    let mut line_starts = Vec::new();
    for (index, line) in text.lines().enumerate() {
        line_starts.push((joined.len(), index + 1));
        for word in unwrapped(line).split_whitespace() {
            joined.push_str(word);
            joined.push(' ');
        }
    }
    let hit = the_two_absolutes_pattern().find(&joined)?;
    let line = line_starts
        .iter()
        .rev()
        .find(|(offset, _)| *offset <= hit.start())
        .map_or(1, |(_, line)| *line);
    Some((line, hit.as_str().to_string()))
}

/// The complaints across the four sites, one per site that states the ratio.
fn sites_that_state_the_ratio(sites: &[(String, String)]) -> Vec<String> {
    sites
        .iter()
        .filter_map(|(name, text)| {
            the_two_absolutes_in(text).map(|(line, words)| {
                format!(
                    "{name}:{line}: states the share of history before red/green as two \
                     absolutes, \"{words}\"; the share is computed by \
                     the_share_of_history_before_red_green and printed by the check, so name \
                     the check and give the share as of a date"
                )
            })
        })
        .collect()
}

fn read_the_sites() -> Vec<(String, String)> {
    THE_SITES_THAT_STATED_THE_RATIO
        .iter()
        .map(|name| {
            let text = fs::read_to_string(name)
                .unwrap_or_else(|e| panic!("{name} is a site the ratio lived at and it {e}"));
            ((*name).to_string(), text)
        })
        .collect()
}

#[test]
fn test_the_share_of_history_before_red_green_is_computed_and_printed() {
    let (before, in_all) =
        the_share_of_history_before_red_green().unwrap_or_else(|why| panic!("{why}"));
    let share = before as f64 * 100.0 / in_all as f64;
    let today = chrono::Local::now().format("%Y-%m-%d");
    println!("{before} of {in_all} commits, {share:.1}%, predate red/green as of {today}");

    let wrong = sites_that_state_the_ratio(&read_the_sites());
    assert!(
        wrong.is_empty(),
        "the share of history before red/green is {before} of {in_all} today and moves with \
         every commit, so no page states it as two absolutes:\n  {}",
        wrong.join("\n  ")
    );
}

fn the_sites_are_clean_today(sites: &[(String, String)]) {
    assert_eq!(
        sites_that_state_the_ratio(sites),
        Vec::<String>::new(),
        "a site already states the ratio, so what this splices in is not the only thing the \
         reading has to find"
    );
}

/// The real sites with one site's text swapped, and the line the splice sits on.
fn with_the_site_replaced(
    sites: &[(String, String)],
    name: &str,
    matching: impl Fn(&str) -> bool,
    replacement: impl Fn(&str) -> String,
) -> (Vec<(String, String)>, usize) {
    let text = &sites
        .iter()
        .find(|(site, _)| site == name)
        .unwrap_or_else(|| panic!("{name} is not a site the reading walks"))
        .1;
    let line = text
        .lines()
        .position(&matching)
        .expect("the line to splice at to be there")
        + 1;
    let original = text.lines().nth(line - 1).expect("the line was just found");
    let spliced = with_one_line_replaced(text, &matching, &replacement(original));
    let mut swapped = sites.to_vec();
    let at = swapped
        .iter()
        .position(|(site, _)| site == name)
        .expect("found above");
    swapped[at].1 = spliced;
    (swapped, line)
}

#[test]
fn test_the_ratio_reading_can_see_the_one_line_form() {
    let sites = read_the_sites();
    the_sites_are_clean_today(&sites);
    // Into CLAUDE.md's own text, on the line that opens the paragraph the
    // sentence used to live in.
    let opener = "A green suite says the code does what the tests say.";
    let (spliced, line) = with_the_site_replaced(
        &sites,
        "CLAUDE.md",
        |l| l.starts_with(opener),
        |l| format!("{l} It started at commit 182 of 344."),
    );
    let wrong = sites_that_state_the_ratio(&spliced);
    assert_eq!(
        wrong.len(),
        1,
        "one site was planted and the reading named {wrong:?}"
    );
    assert!(
        wrong[0].starts_with(&format!("CLAUDE.md:{line}: "))
            && wrong[0].contains("\"commit 182 of 344\""),
        "the one-line form was planted in CLAUDE.md at line {line} and the reading answered {}",
        wrong[0]
    );
}

#[test]
fn test_the_ratio_reading_can_see_the_wrapped_form() {
    let sites = read_the_sites();
    the_sites_are_clean_today(&sites);
    // Into scripts/mutants.sh's own comment, wrapped the way the real sentence
    // was: the count at the end of one comment line and the total at the
    // start of the next, behind its own `#`. A line grep was blind to this
    // shape and this plan's first draft had the same blind spot.
    let opener = "# This project needs to know the difference.";
    let (spliced, line) = with_the_site_replaced(
        &sites,
        "scripts/mutants.sh",
        |l| l.starts_with(opener),
        |l| format!("{l} Red/green started at commit 182 of\n# 344, so most of the tests"),
    );
    let wrong = sites_that_state_the_ratio(&spliced);
    assert_eq!(
        wrong.len(),
        1,
        "one site was planted and the reading named {wrong:?}"
    );
    assert!(
        wrong[0].starts_with(&format!("scripts/mutants.sh:{line}: "))
            && wrong[0].contains("\"commit 182 of 344\""),
        "the wrapped form was planted in scripts/mutants.sh at line {line} and the reading \
         answered {}",
        wrong[0]
    );
}

// ---------------------------------------------------------------------------
// A figure in prose that restates a constant agrees with the constant
// ---------------------------------------------------------------------------
//
// The one kind of number a check can hold outright: both halves are in the
// repository. "A single file is kept up to 25 MB" on the privacy page
// restates `LARGEST_ATTACHMENT_KEPT_BYTES`, and if either moves without the
// other the page is lying to the person who reads it to decide whether to
// carry the computer around. Nothing read a single one of these before this
// reading existed, and one of the twelve disagreed: the roadmap ticked an
// attachment warning at 10 MB where the code warns at 25.
//
// Each pair is a page, the words before the figure, the unit after it, and
// the constant, imported from the library so a renamed constant fails to
// compile rather than silently unpairing. A pair whose words are not on the
// page is a failure, not a skip: a page reworded out from under the reading
// is the way this goes quiet.

/// The unit a page states a constant in, and how many of the constant's own
/// units that is.
#[derive(Clone, Copy)]
enum Unit {
    Mebibytes,
    Seconds,
}

impl Unit {
    fn word(self) -> &'static str {
        match self {
            Unit::Mebibytes => "MB",
            Unit::Seconds => "seconds",
        }
    }

    fn scale(self) -> u64 {
        match self {
            Unit::Mebibytes => 1024 * 1024,
            Unit::Seconds => 1,
        }
    }
}

/// A figure on a page that restates a constant the code holds.
struct Restated {
    page: &'static str,
    /// The words on the page right before the figure, unique on the page.
    before: &'static str,
    unit: Unit,
    /// The constant's value in its own units, bytes or seconds.
    constant: u64,
    /// The constant's name, for the complaint.
    name: &'static str,
}

const THE_PRIVACY_PAGE: &str = "docs/privacy.md";
const THE_EARCON_PLAN: &str = "docs/plans/20260823-earcon-sound-schemes.md";

/// Every figure on a page that restates a constant, twelve on 2026-09-14.
const THE_FIGURES_THAT_RESTATE_A_CONSTANT: [Restated; 12] = [
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "A single file is kept up to ",
        unit: Unit::Mebibytes,
        constant: LARGEST_ATTACHMENT_KEPT_BYTES as u64,
        name: "LARGEST_ATTACHMENT_KEPT_BYTES",
    },
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "all of them together up to ",
        unit: Unit::Mebibytes,
        constant: ATTACHMENT_CACHE_BUDGET_BYTES as u64,
        name: "ATTACHMENT_CACHE_BUDGET_BYTES",
    },
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "dropped when a signed message is larger than ",
        unit: Unit::Mebibytes,
        constant: LARGEST_SIGNED_MESSAGE_KEPT_BYTES as u64,
        name: "LARGEST_SIGNED_MESSAGE_KEPT_BYTES",
    },
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "when the space these copies use passes ",
        unit: Unit::Mebibytes,
        constant: SIGNED_ORIGINAL_BUDGET_BYTES as u64,
        name: "SIGNED_ORIGINAL_BUDGET_BYTES",
    },
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "A message larger than ",
        unit: Unit::Mebibytes,
        constant: LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES as u64,
        name: "LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES",
    },
    Restated {
        page: THE_PRIVACY_PAGE,
        before: "interrupted moves add up to more than ",
        unit: Unit::Mebibytes,
        constant: MOVES_IN_FLIGHT_BUDGET_BYTES as u64,
        name: "MOVES_IN_FLIGHT_BUDGET_BYTES",
    },
    Restated {
        page: "docs/KEYBOARD_SHORTCUTS.md",
        before: "PNG, JPEG, GIF and WebP up to ",
        unit: Unit::Mebibytes,
        constant: MOST_ONE_PICTURE_MAY_BE as u64,
        name: "MOST_ONE_PICTURE_MAY_BE",
    },
    Restated {
        page: THE_EARCON_PLAN,
        before: "a working number to start from: ",
        unit: Unit::Mebibytes,
        constant: MAX_ZIP_BYTES,
        name: "MAX_ZIP_BYTES",
    },
    Restated {
        page: THE_EARCON_PLAN,
        before: "extracted-size cap (a working number: ",
        unit: Unit::Mebibytes,
        constant: MAX_FILE_BYTES,
        name: "MAX_FILE_BYTES",
    },
    Restated {
        page: THE_EARCON_PLAN,
        before: "a total cap across the whole pack (",
        unit: Unit::Mebibytes,
        constant: MAX_TOTAL_BYTES,
        name: "MAX_TOTAL_BYTES",
    },
    Restated {
        page: THE_EARCON_PLAN,
        before: "a rule and not a suggestion.** A working number: ",
        unit: Unit::Seconds,
        constant: MAX_SOUND_DURATION.as_secs(),
        name: "MAX_SOUND_DURATION",
    },
    Restated {
        page: "docs/roadmap.md",
        before: "once encoded for sending, come to more than ",
        unit: Unit::Mebibytes,
        constant: LIMIT_BYTES,
        name: "attaching::LIMIT_BYTES",
    },
];

/// A page as one line, so a phrase wrapped by the page's own line breaks is
/// one phrase.
fn as_one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The figure the page states after `before`, in the pair's unit, or why it
/// could not be read: the words are absent, or no figure in that unit follows.
fn the_figure_stated(page: &str, pair: &Restated) -> Result<u64, String> {
    let one_line = as_one_line(page);
    let Some(at) = one_line.find(pair.before) else {
        return Err(format!(
            "{}: the words \"{}\" are not on the page, so {} is no longer held to anything; \
             the page was reworded, so reword the pair",
            pair.page, pair.before, pair.name
        ));
    };
    let rest = &one_line[at + pair.before.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let after = rest[digits.len()..].trim_start();
    if digits.is_empty() || !after.starts_with(pair.unit.word()) {
        return Err(format!(
            "{}: after \"{}\" the page says \"{}\", not a figure in {}",
            pair.page,
            pair.before,
            rest.chars().take(20).collect::<String>(),
            pair.unit.word()
        ));
    }
    digits
        .parse::<u64>()
        .map(|figure| figure * pair.unit.scale())
        .map_err(|_| format!("{}: \"{digits}\" is too large to be a figure", pair.page))
}

/// Every pair whose page disagrees with its constant, or cannot be read.
fn figures_that_disagree_with_their_constant(pages: &[Page]) -> Vec<String> {
    let mut wrong = Vec::new();
    for pair in &THE_FIGURES_THAT_RESTATE_A_CONSTANT {
        let text = &pages
            .iter()
            .find(|(name, _)| name == pair.page)
            .unwrap_or_else(|| panic!("{} restates a constant and was not read", pair.page))
            .1;
        match the_figure_stated(text, pair) {
            Err(why) => wrong.push(why),
            Ok(stated) if stated != pair.constant => wrong.push(format!(
                "{}: \"{}{} {}\" restates {}, which is {} {}",
                pair.page,
                pair.before,
                stated / pair.unit.scale(),
                pair.unit.word(),
                pair.name,
                pair.constant / pair.unit.scale(),
                pair.unit.word()
            )),
            Ok(_) => {}
        }
    }
    wrong
}

/// The pages the pairs name, read once. The earcon plan is under
/// `docs/plans/`, which the provenance walk leaves out, so it is read here
/// by name rather than taken from that walk.
fn read_the_pages_that_restate_a_constant() -> Vec<Page> {
    let mut names: Vec<&str> = THE_FIGURES_THAT_RESTATE_A_CONSTANT
        .iter()
        .map(|pair| pair.page)
        .collect();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .map(|name| {
            let text = fs::read_to_string(name)
                .unwrap_or_else(|e| panic!("{name} restates a constant and it {e}"));
            (name.to_string(), text)
        })
        .collect()
}

#[test]
fn test_every_figure_that_restates_a_constant_agrees_with_it() {
    let wrong =
        figures_that_disagree_with_their_constant(&read_the_pages_that_restate_a_constant());
    assert!(
        wrong.is_empty(),
        "a figure on a page that restates a constant the code holds must equal it, and \
         these do not:\n  {}",
        wrong.join("\n  ")
    );
}

fn the_pairs_agree_today(pages: &[Page]) {
    assert_eq!(
        figures_that_disagree_with_their_constant(pages),
        Vec::<String>::new(),
        "a page already disagrees with a constant, so what this changes is not the only \
         thing the reading has to find"
    );
}

#[test]
fn test_the_constants_reading_can_see_a_figure_that_moved() {
    let pages = read_the_pages_that_restate_a_constant();
    the_pairs_agree_today(&pages);
    // The privacy page's own line, its figure moved by one.
    let pair = &THE_FIGURES_THAT_RESTATE_A_CONSTANT[0];
    let stated = pair.constant / pair.unit.scale();
    let words = format!("{}{stated} {}", pair.before, pair.unit.word());
    let text = &pages
        .iter()
        .find(|(name, _)| name == pair.page)
        .expect("the page")
        .1;
    let moved = with_one_line_replaced(
        text,
        |line| line.contains(&words),
        &text
            .lines()
            .find(|line| line.contains(&words))
            .expect("the line")
            .replace(
                &words,
                &format!("{}{} {}", pair.before, stated + 1, pair.unit.word()),
            ),
    );
    assert_eq!(
        figures_that_disagree_with_their_constant(&with_the_page_replaced(
            &pages, pair.page, moved
        )),
        vec![format!(
            "{}: \"{}{} {}\" restates {}, which is {stated} {}",
            pair.page,
            pair.before,
            stated + 1,
            pair.unit.word(),
            pair.name,
            pair.unit.word()
        )],
        "the figure was moved by one on the page's own line and the reading did not answer \
         with exactly that pair"
    );
}

#[test]
fn test_the_constants_reading_refuses_a_phrase_that_has_gone() {
    let pages = read_the_pages_that_restate_a_constant();
    the_pairs_agree_today(&pages);
    let pair = &THE_FIGURES_THAT_RESTATE_A_CONSTANT[0];
    let text = &pages
        .iter()
        .find(|(name, _)| name == pair.page)
        .expect("the page")
        .1;
    let reworded = text.replace(pair.before, "One file is kept up to ");
    assert_ne!(reworded, *text, "the phrase to remove was not on the page");
    assert_eq!(
        figures_that_disagree_with_their_constant(&with_the_page_replaced(
            &pages, pair.page, reworded
        )),
        vec![format!(
            "{}: the words \"{}\" are not on the page, so {} is no longer held to anything; \
             the page was reworded, so reword the pair",
            pair.page, pair.before, pair.name
        )],
        "the phrase was reworded out from under the pair and the reading did not say so"
    );
}

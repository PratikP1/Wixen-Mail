//! The WCAG 2.2 success criteria the automated accessibility scans can produce
//! a finding against, held as code.
//!
//! Three of fifty-five. The Axe.Windows rule list for the release the scan
//! workflow pins, `v2.4.2`, cites 1.3.1 Info and Relationships, 2.1.1 Keyboard
//! and 4.1.2 Name, Role, Value, and no other WCAG criterion. The MSAA walk in
//! `scripts/msaa-names.ps1` asks whether an operated control has a name, which
//! is the Name part of 4.1.2 and nothing else.
//!
//! The list a person reads is `docs/wcag-coverage.md`, fifty-five rows. A page
//! on its own goes stale the way the count of five WebView2 findings did: it was
//! written when the scan covered one window on one channel, the scan grew to
//! thirty-one on two, and nothing said so for seven weeks. So the three
//! criteria live here, the reading below holds the page's table to them in
//! both directions, and the scan's own summary is held to them too. The other
//! fifty-two rows are a person's judgement and no test holds those.

use std::fmt;

/// A WCAG 2.2 success criterion, by number and name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Criterion {
    pub number: &'static str,
    pub name: &'static str,
}

impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.number, self.name)
    }
}

/// How many success criteria WCAG 2.2 has at Level A and AA together.
///
/// 31 at A and 24 at AA, read from the specification on 2026-09-14. Not 56:
/// that figure counts 4.1.1 Parsing, which WCAG 2.2 marks obsolete and gives
/// no level.
pub const LEVEL_A_AND_AA_CRITERIA: usize = 55;

/// The criteria the Axe.Windows rule list cites, for the release the scan
/// workflow pins.
///
/// Read from `axe-windows-rules-2.4.2.md` on 2026-09-14: 61 rules cite 1.3.1,
/// 9 cite 2.1.1, 9 cite 4.1.2, and the other 76 cite a Section 508 clause and
/// no WCAG criterion. A rule citing a criterion tests one narrow property of
/// one element; it is not a judgement of the criterion.
pub const AXE_WINDOWS_CAN_JUDGE: [Criterion; 3] = [
    Criterion {
        number: "1.3.1",
        name: "Info and Relationships",
    },
    Criterion {
        number: "2.1.1",
        name: "Keyboard",
    },
    Criterion {
        number: "4.1.2",
        name: "Name, Role, Value",
    },
];

/// The one criterion the MSAA walk contributes to, and only its Name part.
///
/// The walk exits 1 when an element whose role somebody operates has an empty
/// accessible name. It never looks at the role's correctness or at value or
/// state.
pub const THE_MSAA_WALK_CAN_JUDGE: [Criterion; 1] = [Criterion {
    number: "4.1.2",
    name: "Name, Role, Value",
}];

/// The page a person reads.
pub const THE_COVERAGE_PAGE: &str = "docs/wcag-coverage.md";

/// The accessibility scan workflow, whose step summary is the scan's output.
pub const THE_SCAN_WORKFLOW: &str = ".github/workflows/accessibility.yml";

/// A column of the coverage page's table, named by the start of its heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    AxeWindows,
    MsaaWalk,
}

impl Channel {
    /// How the column's heading begins in the page's table.
    pub const fn heading(self) -> &'static str {
        match self {
            Self::AxeWindows => "Axe.Windows",
            Self::MsaaWalk => "MSAA walk",
        }
    }

    /// What the code says this channel can produce a finding against.
    pub const fn the_code_says(self) -> &'static [Criterion] {
        match self {
            Self::AxeWindows => &AXE_WINDOWS_CAN_JUDGE,
            Self::MsaaWalk => &THE_MSAA_WALK_CAN_JUDGE,
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.heading())
    }
}

/// One row of the coverage table: a criterion number and what each column says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub number: String,
    cells: Vec<String>,
}

impl Row {
    /// What the row says under a column, or nothing where the row is short.
    pub fn cell(&self, column: usize) -> Option<&str> {
        self.cells.get(column).map(String::as_str)
    }
}

/// The coverage table: where each channel's column is, and every row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    headings: Vec<String>,
    pub rows: Vec<Row>,
}

impl Table {
    /// The column whose heading begins with the channel's name.
    pub fn column_for(&self, channel: Channel) -> Option<usize> {
        self.headings
            .iter()
            .position(|heading| heading.starts_with(channel.heading()))
    }

    /// The criteria whose cell under this channel begins with "yes".
    pub fn says_yes_under(&self, channel: Channel) -> Vec<String> {
        let Some(column) = self.column_for(channel) else {
            return Vec::new();
        };
        self.rows
            .iter()
            .filter(|row| row.cell(column).is_some_and(says_yes))
            .map(|row| row.number.clone())
            .collect()
    }
}

/// Whether a cell answers yes, whatever follows the word.
fn says_yes(cell: &str) -> bool {
    cell.to_lowercase().starts_with("yes")
}

/// Where the page's table and the code disagree about a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disagreement {
    /// The page has no table whose header names the channel, or the table
    /// has no row that begins with a criterion number. A reading over nothing
    /// is not a pass.
    ThePageHasNoRowsFor(Channel),
    /// The code's list for the channel is empty, so there is nothing to hold
    /// the page to. A check over an empty list is not a pass either.
    TheCodeNamesNothingFor(Channel),
    /// The page marks a criterion as one the channel can judge and the code
    /// does not name it.
    ThePageSaysYes { channel: Channel, criterion: String },
    /// The code names a criterion and the page does not mark it.
    TheCodeSaysYes {
        channel: Channel,
        criterion: Criterion,
    },
}

impl fmt::Display for Disagreement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThePageHasNoRowsFor(channel) => write!(
                f,
                "the page has no table with a {channel} column and a row per criterion, so \
                 there is nothing to read"
            ),
            Self::TheCodeNamesNothingFor(channel) => write!(
                f,
                "the code names no criterion for {channel}, so there is nothing to hold the \
                 page to"
            ),
            Self::ThePageSaysYes { channel, criterion } => write!(
                f,
                "the page says {channel} can judge {criterion} and the code does not name it"
            ),
            Self::TheCodeSaysYes { channel, criterion } => write!(
                f,
                "the code says {channel} can judge {criterion} and the page does not say yes"
            ),
        }
    }
}

/// The coverage table in a page, read from the header that names the
/// channels down to the first line that is not a table row.
///
/// The page has other tables. The one wanted is the one whose header has a
/// column for Axe.Windows, and its rows are the lines after that header, up
/// to the first line that is not a row, that begin with a criterion number.
pub fn the_coverage_table(page: &str) -> Option<Table> {
    let mut lines = page.lines();
    let headings = lines.find_map(|line| {
        let cells = cells_of(line)?;
        cells
            .iter()
            .any(|cell| cell.starts_with(Channel::AxeWindows.heading()))
            .then_some(cells)
    })?;
    let rows = lines
        .map_while(cells_of)
        .filter(|cells| !is_a_separator(cells))
        .filter_map(|cells| {
            let number = criterion_number(cells.first()?)?;
            Some(Row { number, cells })
        })
        .collect();
    Some(Table { headings, rows })
}

/// The cells of a table row, or nothing where the line is not one.
fn cells_of(line: &str) -> Option<Vec<String>> {
    let inner = line.trim().strip_prefix('|')?;
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    Some(
        inner
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect(),
    )
}

/// The row under a header that is only dashes and colons.
fn is_a_separator(cells: &[String]) -> bool {
    cells
        .iter()
        .all(|cell| !cell.is_empty() && cell.chars().all(|letter| matches!(letter, '-' | ':')))
}

/// "1.3.1" from "1.3.1 Info and Relationships", or nothing where the cell
/// does not begin with three numbers joined by dots.
fn criterion_number(cell: &str) -> Option<String> {
    let first_word = cell.split_whitespace().next()?;
    let parts: Vec<&str> = first_word.split('.').collect();
    let three_numbers = parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    three_numbers.then(|| first_word.to_string())
}

/// Where the page and the code disagree about which criteria a channel can
/// judge, in both directions, with the empty cases reported rather than
/// passed.
pub fn where_the_page_disagrees_with(
    page: &str,
    channel: Channel,
    the_code_says: &[Criterion],
) -> Vec<Disagreement> {
    if the_code_says.is_empty() {
        return vec![Disagreement::TheCodeNamesNothingFor(channel)];
    }
    let table = match the_coverage_table(page) {
        Some(table) if !table.rows.is_empty() && table.column_for(channel).is_some() => table,
        _ => return vec![Disagreement::ThePageHasNoRowsFor(channel)],
    };
    let the_page_says = table.says_yes_under(channel);

    let mut disagreements = Vec::new();
    for number in &the_page_says {
        if !the_code_says
            .iter()
            .any(|criterion| criterion.number == number)
        {
            disagreements.push(Disagreement::ThePageSaysYes {
                channel,
                criterion: number.clone(),
            });
        }
    }
    for criterion in the_code_says {
        if !the_page_says
            .iter()
            .any(|number| number == criterion.number)
        {
            disagreements.push(Disagreement::TheCodeSaysYes {
                channel,
                criterion: *criterion,
            });
        }
    }
    disagreements
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn the_page() -> String {
        fs::read_to_string(THE_COVERAGE_PAGE).expect("the coverage page")
    }

    /// The page with one criterion's cell under one channel replaced.
    ///
    /// Planted in the real page rather than in a made-up table, so that what
    /// is proved is the reading over the shape the page really has.
    fn with_the_cell_changed(page: &str, number: &str, channel: Channel, cell: &str) -> String {
        let table = the_coverage_table(page).expect("the coverage table");
        let column = table.column_for(channel).expect("the channel's column");
        let mut changed = 0;
        let planted: Vec<String> = page
            .lines()
            .map(|line| {
                if !line.starts_with(&format!("| {number} ")) {
                    return line.to_string();
                }
                changed += 1;
                let mut cells: Vec<&str> =
                    line.trim_matches('|').split('|').map(str::trim).collect();
                cells[column] = cell;
                format!("| {} |", cells.join(" | "))
            })
            .collect();
        assert_eq!(changed, 1, "expected exactly one row for {number}");
        planted.join("\n")
    }

    #[test]
    fn test_the_page_has_a_row_for_every_criterion_at_level_a_and_aa() {
        let table = the_coverage_table(&the_page()).expect("the coverage table");
        assert_eq!(
            table.rows.len(),
            LEVEL_A_AND_AA_CRITERIA,
            "the page has {} rows and WCAG 2.2 has {} criteria at Level A and AA",
            table.rows.len(),
            LEVEL_A_AND_AA_CRITERIA
        );
    }

    #[test]
    fn test_the_page_says_yes_for_exactly_the_criteria_the_code_names() {
        let page = the_page();
        for channel in [Channel::AxeWindows, Channel::MsaaWalk] {
            let disagreements =
                where_the_page_disagrees_with(&page, channel, channel.the_code_says());
            let said: Vec<String> = disagreements.iter().map(ToString::to_string).collect();
            assert!(
                disagreements.is_empty(),
                "{THE_COVERAGE_PAGE} and the code disagree about {channel}:\n  {}",
                said.join("\n  ")
            );
        }
    }

    #[test]
    fn test_a_fourth_criterion_marked_yes_on_the_page_is_reported() {
        // The companion. Without it the check above could pass by reading
        // nothing, which is how a guard reading a document has been disarmed
        // twice in this repository.
        let planted = with_the_cell_changed(&the_page(), "2.4.7", Channel::AxeWindows, "yes");
        let disagreements =
            where_the_page_disagrees_with(&planted, Channel::AxeWindows, &AXE_WINDOWS_CAN_JUDGE);
        assert_eq!(
            disagreements,
            vec![Disagreement::ThePageSaysYes {
                channel: Channel::AxeWindows,
                criterion: "2.4.7".to_string(),
            }]
        );
    }

    #[test]
    fn test_one_of_the_three_marked_no_on_the_page_is_reported() {
        let planted = with_the_cell_changed(&the_page(), "2.1.1", Channel::AxeWindows, "no");
        let disagreements =
            where_the_page_disagrees_with(&planted, Channel::AxeWindows, &AXE_WINDOWS_CAN_JUDGE);
        assert_eq!(
            disagreements,
            vec![Disagreement::TheCodeSaysYes {
                channel: Channel::AxeWindows,
                criterion: AXE_WINDOWS_CAN_JUDGE[1],
            }]
        );
    }

    #[test]
    fn test_the_msaa_column_is_read_on_its_own_and_not_as_a_copy_of_the_first() {
        // Two columns, two readings. A reading that found the first column
        // twice would agree with the code for Axe.Windows and never notice
        // the MSAA column at all.
        let planted = with_the_cell_changed(&the_page(), "1.3.1", Channel::MsaaWalk, "yes");
        let disagreements =
            where_the_page_disagrees_with(&planted, Channel::MsaaWalk, &THE_MSAA_WALK_CAN_JUDGE);
        assert_eq!(
            disagreements,
            vec![Disagreement::ThePageSaysYes {
                channel: Channel::MsaaWalk,
                criterion: "1.3.1".to_string(),
            }]
        );
    }

    #[test]
    fn test_a_page_with_no_rows_fails_rather_than_passes() {
        let no_table = "# A page\n\nProse and nothing else.\n";
        assert_eq!(
            where_the_page_disagrees_with(no_table, Channel::AxeWindows, &AXE_WINDOWS_CAN_JUDGE),
            vec![Disagreement::ThePageHasNoRowsFor(Channel::AxeWindows)]
        );

        let header_and_no_rows = "| Criterion | Level | Axe.Windows (UI Automation) | MSAA walk |\n\
                                  |---|---|---|---|\n\
                                  \n\
                                  Prose after an empty table.\n";
        assert_eq!(
            where_the_page_disagrees_with(
                header_and_no_rows,
                Channel::AxeWindows,
                &AXE_WINDOWS_CAN_JUDGE
            ),
            vec![Disagreement::ThePageHasNoRowsFor(Channel::AxeWindows)]
        );
    }

    #[test]
    fn test_an_empty_list_in_the_code_fails_rather_than_passes() {
        // The census-emptying failure: a list narrowed to nothing agrees with
        // every page that says no everywhere, and says so with a green run.
        assert_eq!(
            where_the_page_disagrees_with(&the_page(), Channel::AxeWindows, &[]),
            vec![Disagreement::TheCodeNamesNothingFor(Channel::AxeWindows)]
        );
    }

    /// The workflow's commands, with its comments left out, because the
    /// comment explaining a fix has reddened a file-reading test here before.
    fn the_workflows_commands() -> String {
        fs::read_to_string(THE_SCAN_WORKFLOW)
            .expect("the accessibility workflow")
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_the_scan_output_names_the_criteria_the_code_names_and_not_a_fraction() {
        // Success criterion 3 of phase 6, read clause by clause: the scan
        // *output* names which criteria it can and cannot judge. The output is
        // the step summary the workflow writes, so the workflow's summary
        // lines are read for each criterion the code names, and for the
        // sentence this replaces.
        let commands = the_workflows_commands();
        let summary: Vec<&str> = commands
            .lines()
            .filter(|line| line.contains("GITHUB_STEP_SUMMARY"))
            .collect();
        assert!(
            !summary.is_empty(),
            "the workflow writes no step summary, so the scan has no output to read"
        );
        for criterion in AXE_WINDOWS_CAN_JUDGE {
            assert!(
                summary.iter().any(|line| line.contains(criterion.number)),
                "the scan's summary never names {criterion}, which the code says it can judge"
            );
        }
        assert!(
            summary.iter().any(|line| line.contains(THE_COVERAGE_PAGE)),
            "the scan's summary does not point at {THE_COVERAGE_PAGE} for the criteria it \
             cannot judge"
        );
        let fraction = commands
            .lines()
            .find(|line| line.to_lowercase().contains("half of wcag"));
        assert!(
            fraction.is_none(),
            "the workflow still says the scan covers a fraction of WCAG: {}",
            fraction.unwrap_or_default().trim()
        );
    }
}

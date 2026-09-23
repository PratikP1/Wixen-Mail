//! Every text an NVDA case waits to hear is something the program says, at the
//! place the case means.
//!
//! On 2026-09-23 NVDA runs 35839692317 and 35839954840 were red on two cases
//! whose product was right. 12-03 had rewritten the sentences the calendar and
//! filter cases waited for, NVDA heard the new ones, and the cases went on
//! waiting for the old ones until they timed out. Nothing in the Rust gate
//! reads `nvda-tests/`, so a rewording could pass every check here and fail
//! only on the runner, after a push. This reading moves that failure to the
//! commit that makes it (FOUND-22).
//!
//! # What it reads
//!
//! From every `nvda-tests/tests/*.test.js`, with JavaScript comments stepped
//! over: each text in the array handed to `waitToHearAll`, the text handed to
//! `tabUntilHeard`, and the text handed to `toContain`. `tabUntilHeard` is read
//! as well because a relabelled button fails a case the same way a reworded
//! sentence does. A negated `toContain` asserts that something is absent and
//! is not read. A text is written in the case as a literal, as a `const` bound
//! in the same file to a literal or to literals joined with `+`, or as
//! `something.heardAs`, which stands for every `heardAs:` literal in that file.
//! Anything else is refused unless `WHAT_THE_READING_CANNOT_RESOLVE` names it.
//!
//! From every file under `src/`, the half a release build compiles
//! (`common::what_ships`), with Rust comments stepped over: every string
//! literal, its escapes read, a line it continues onto with `\` folded the way
//! Rust folds it, and raw strings read as written. A literal is also read the
//! way NVDA reads a label, with a mnemonic's `&` taken out and `&&` read as
//! `&`. And the sentences `application::status_sentences` builds for every
//! kind in `Thing::ALL`.
//!
//! # What "said by the program" means
//!
//! A text is held by a place when that place says it: a literal holding each
//! of the text's sentences, or a built sentence equal to it. Each occurrence
//! of a literal is its own place, two in one file included. A text is said by
//! the program when exactly one place holds it; or several do and
//! `THE_ONE_PLACE_A_SHARED_TEXT_MEANS` ties it to one of them; or
//! `SAID_BY_SOMETHING_ELSE_OR_BUILT_FROM_PARTS` names it, as NVDA's own words
//! for a role or a state, or as a text the program builds from parts, each
//! part then held by at least one place in turn.
//!
//! A tie names the case, the text, the file, the function the literal sits in
//! and the literal as written, and holds while that literal occurs exactly
//! once in that function's body in that file's shipped half and still says
//! the text. A function and a literal rather than a line, because neither
//! moves when the formatter rewraps a builder chain.
//!
//! **Why a count of places and a tie, rather than a tighter match.** "Sync",
//! the button the calendar case tabs to, is also inside "&Sync calendar now"
//! in the context menu, so a substring match stays green after the calendar's
//! own button is relabelled. A whole-literal match does not close that either:
//! "S&ync" is the calendar's button and the contact manager's, and "&Delete"
//! is on more than a dozen lines. What a rewording has to change is the one
//! place a case means, so the reading asks for one.
//!
//! The refusals say which of three things went wrong, because a check that
//! can fail three ways says which: held by nothing; held by several places
//! and tied to none, naming them; tied to a place that no longer says it.
//!
//! # What it cannot see
//!
//! A text whose one holding place is not the one the case means is taken on
//! trust. A relabel made in the same commit that adds a new place holding the
//! old text passes. A tied literal moved to another control inside the same
//! function passes. Anything NVDA says that no case waits for is not read at
//! all, and whether NVDA speaks what the source says is the runner's to show,
//! not this file's.
//!
//! # Why it runs on every code commit
//!
//! A waited sentence can live in any file under `src/`, and a case in any file
//! under `nvda-tests/`, so no guard record's `file` can route the commits that
//! break it. It is in the list of targets every scoped run ends with, in
//! `scripts/check.sh`, and a test below holds it there.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use wixen_mail::application::status_sentences::{Thing, at_least_one_chosen, nothing_chosen};
use wixen_mail::common::what_ships::what_ships;

/// This target's own name, as the gate script lists it.
const ME: &str = "the_nvda_cases_wait_for_words_the_program_says";

const THE_CASES: &str = "nvda-tests/tests";
const THE_SOURCE: &str = "src";

/// The three calls whose texts a case waits for, as they are written in a
/// case.
const WAITS: &str = "waitToHearAll(";
const TABS_TO: &str = "tabUntilHeard(";
const ASSERTS_IT_CONTAINS: &str = ".toContain(";

// ── The three tables ─────────────────────────────────────────────────────────

/// An expression in a case this reading cannot turn into a text.
struct CannotResolve {
    case: &'static str,
    expression: &'static str,
    why: &'static str,
}

/// None today: every text a case waits for is a literal, a `const` of
/// literals, or a `heardAs`.
const WHAT_THE_READING_CANNOT_RESOLVE: &[CannotResolve] = &[];

/// Who says a text the program's source does not hold whole.
enum SaidBy {
    /// NVDA's own words for a role or a state.
    NvdasOwnWords,
    /// A text the program builds with `format!`, from these literal parts,
    /// each of which must be held by at least one place.
    BuiltFrom(&'static [&'static str]),
}

struct SaidElsewhere {
    case: &'static str,
    text: &'static str,
    said_by: SaidBy,
    why: &'static str,
}

const SAID_BY_SOMETHING_ELSE_OR_BUILT_FROM_PARTS: &[SaidElsewhere] = &[
    SaidElsewhere {
        case: "a-link-opens-where-the-setting-says.test.js",
        text: "Scan target - headings - Wixen Mail",
        said_by: SaidBy::BuiltFrom(&["Scan target", " - headings - Wixen Mail"]),
        why: "The page window's title, which show_conversation_as_page builds from the \
              fixture's subject and its own ending. The case asserts the window list \
              holds it rather than hearing it.",
    },
    SaidElsewhere {
        case: "which-days-focus-and-tick.test.js",
        text: "Changes every day this event falls on, and leaves the day it starts on where it \
               is. Nothing is sent anywhere, because no account holds this event.",
        said_by: SaidBy::BuiltFrom(&[
            "Changes every day this event falls on, and leaves the day it starts on where it is.",
            "Nothing is sent anywhere, because no account holds this event.",
        ]),
        why: "The button's description, which calendar::what_it_will_do builds by putting the \
              second sentence after the first with format!.",
    },
    SaidElsewhere {
        case: "which-days-focus-and-tick.test.js",
        text: "radio button",
        said_by: SaidBy::NvdasOwnWords,
        why: "NVDA's name for the control's role.",
    },
    SaidElsewhere {
        case: "which-days-focus-and-tick.test.js",
        text: "checked",
        said_by: SaidBy::NvdasOwnWords,
        why: "NVDA's name for the control's state.",
    },
];

/// Which of several places holding a text is the one a case means.
struct Tie {
    case: &'static str,
    text: &'static str,
    file: &'static str,
    function: &'static str,
    literal: &'static str,
    why: &'static str,
}

const THE_ONE_PLACE_A_SHARED_TEXT_MEANS: &[Tie] = &[
    Tie {
        case: "account-manager-sign-in-failure.test.js",
        text: "Scan target",
        file: "src/presentation/wx_app.rs",
        function: "scan_only_account",
        literal: "Scan target",
        why: "The fixture account's name, which the row NVDA reads after Down carries. Every \
              scan fixture is called the same, and the case comment names this function.",
    },
    Tie {
        case: "account-manager-sign-in-failure.test.js",
        text: "Sign In Again",
        file: "src/presentation/wx_account_manager.rs",
        function: "build_account_manager_dialog",
        literal: "&Sign In Again",
        why: "The Account Manager's button the case tabs to. The same words are inside a \
              sentence in mail_auth.rs that tells somebody to press it.",
    },
    Tie {
        case: "calendar-immediate-actions.test.js",
        text: "Sync",
        file: "src/presentation/wx_calendar.rs",
        function: "build_calendar_dialog",
        literal: "S&ync",
        why: "The Calendar window's Sync button the case tabs to. The contact manager has a \
              button written the same way, and the word is inside many menu items and \
              sentences.",
    },
    Tie {
        case: "calendar-immediate-actions.test.js",
        text: "Syncing the calendar...",
        file: "src/presentation/wx_calendar.rs",
        function: "request_sync",
        literal: "Syncing the calendar...",
        why: "What the Calendar window says when its Sync button is pressed. The main \
              window's two calendar syncs send the same step from wx_app.rs.",
    },
    Tie {
        case: "filter-manager-delete.test.js",
        text: "Delete",
        file: "src/presentation/wx_managers.rs",
        function: "run_manager_loop",
        literal: "&Delete",
        why: "The Delete button of the manager windows, which the filter manager is one of. \
              The same label is written for the contact manager, the Account Manager and a \
              dozen menu items.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "General",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "General",
        why: "The Settings tab. The notes' first folder and the spell checker's list of \
              settings are called the same.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "Compose",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "Compose",
        why: "The Settings tab. The word is also in the toolbar's and the composer's own \
              labels and in the spell checker's list of settings.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "Reading",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "Reading",
        why: "The Settings tab. The word opens many sentences and names the reader window.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "Permissions",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "Permissions",
        why: "The Settings tab. A sentence in wx_app.rs names it as the place a box lives.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "Feedback",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "Feedback",
        why: "The Settings tab. A sentence further down the same file names it.",
    },
    Tie {
        case: "settings-tabs-read-once.test.js",
        text: "Advanced",
        file: "src/presentation/wx_settings.rs",
        function: "build_settings_dialog",
        literal: "Advanced",
        why: "The Settings tab. The spell checker's list of settings has the same word.",
    },
];

// ── What the cases wait for ──────────────────────────────────────────────────

/// One text a case waits for, where it waits for it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Waited {
    case: String,
    line: usize,
    text: String,
    how: &'static str,
}

/// An argument the reading could not turn into a text.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Unresolved {
    case: String,
    line: usize,
    expression: String,
}

/// Everything read from the cases.
#[derive(Default, Debug)]
struct WhatTheCasesWaitFor {
    waited: Vec<Waited>,
    unresolved: Vec<Unresolved>,
}

/// `source` with every JavaScript comment blanked, newlines kept, so every
/// offset still sits on the line it had.
fn without_js_comments(source: &str) -> String {
    let letters: Vec<char> = source.chars().collect();
    let mut kept = String::with_capacity(source.len());
    let mut at = 0;
    while at < letters.len() {
        let here = letters[at];
        let next = letters.get(at + 1).copied();
        if matches!(here, '"' | '\'' | '`') {
            let end = past_a_js_string(&letters, at);
            kept.extend(&letters[at..end]);
            at = end;
        } else if here == '/' && next == Some('/') {
            while at < letters.len() && letters[at] != '\n' {
                kept.push(' ');
                at += 1;
            }
        } else if here == '/' && next == Some('*') {
            let end = (at + 2..letters.len().saturating_sub(1))
                .find(|&i| letters[i] == '*' && letters[i + 1] == '/')
                .map_or(letters.len(), |i| i + 2);
            kept.extend(
                letters[at..end]
                    .iter()
                    .map(|&c| if c == '\n' { '\n' } else { ' ' }),
            );
            at = end;
        } else {
            kept.push(here);
            at += 1;
        }
    }
    kept
}

/// The index just past a JavaScript string opening at `at`.
fn past_a_js_string(letters: &[char], at: usize) -> usize {
    let quote = letters[at];
    let mut i = at + 1;
    while i < letters.len() && letters[i] != quote {
        if letters[i] == '\\' {
            i += 1;
        }
        i += 1;
    }
    (i + 1).min(letters.len())
}

/// The arguments of the call whose `(` is at `open`, split at the commas that
/// are not inside a string or a bracket, each with the offset it starts at.
fn arguments_at(letters: &[char], open: usize) -> Vec<(usize, String)> {
    let mut arguments = Vec::new();
    let mut depth = 0;
    let mut start = open + 1;
    let mut i = open + 1;
    while i < letters.len() {
        match letters[i] {
            '"' | '\'' | '`' => {
                i = past_a_js_string(letters, i);
                continue;
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => {
                arguments.push((start, letters[start..i].iter().collect()));
                break;
            }
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                arguments.push((start, letters[start..i].iter().collect()));
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    arguments
        .into_iter()
        .map(|(at, text): (usize, String)| {
            let leading = text.len() - text.trim_start().len();
            (
                at + text[..leading].chars().count(),
                text.trim().to_string(),
            )
        })
        .filter(|(_, text)| !text.is_empty())
        .collect()
}

fn line_of(letters: &[char], at: usize) -> usize {
    letters[..at].iter().filter(|&&c| c == '\n').count() + 1
}

/// A JavaScript string literal's text, or nothing when `expression` is not
/// one. A template is taken only when it interpolates nothing.
fn a_js_literal(expression: &str) -> Option<String> {
    let quote = expression.chars().next()?;
    if !matches!(quote, '"' | '\'' | '`') || expression.len() < 2 || !expression.ends_with(quote) {
        return None;
    }
    let inner = &expression[1..expression.len() - 1];
    if quote == '`' && inner.contains("${") {
        return None;
    }
    let mut text = String::new();
    let mut letters = inner.chars();
    while let Some(letter) = letters.next() {
        if letter != '\\' {
            text.push(letter);
            continue;
        }
        match letters.next() {
            Some('n') => text.push('\n'),
            Some('t') => text.push('\t'),
            Some(other) => text.push(other),
            None => {}
        }
    }
    Some(text)
}

/// Every `heardAs: "<literal>"` in a case.
fn every_heard_as(code: &str) -> Vec<String> {
    let letters: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    for (at, _) in code.match_indices("heardAs:") {
        let from = code[..at].chars().count() + "heardAs:".len();
        let mut start = from;
        while start < letters.len() && letters[start].is_whitespace() {
            start += 1;
        }
        if start < letters.len() && matches!(letters[start], '"' | '\'' | '`') {
            let end = past_a_js_string(&letters, start);
            let literal: String = letters[start..end].iter().collect();
            found.extend(a_js_literal(&literal));
        }
    }
    found
}

/// The text a `const` in the same case is bound to, when its initialiser is
/// literals joined with `+`.
fn a_const_of_literals(code: &str, name: &str) -> Option<String> {
    let binding = format!("const {name} =");
    let at = code.find(&binding)? + binding.len();
    let initialiser = &code[at..at + code[at..].find(';')?];
    let letters: Vec<char> = initialiser.chars().collect();
    let mut text = String::new();
    let mut i = 0;
    while i < letters.len() {
        let here = letters[i];
        if here.is_whitespace() || here == '+' {
            i += 1;
        } else if matches!(here, '"' | '\'' | '`') {
            let end = past_a_js_string(&letters, i);
            text.push_str(&a_js_literal(&letters[i..end].iter().collect::<String>())?);
            i = end;
        } else {
            return None;
        }
    }
    Some(text)
}

fn is_a_name(expression: &str) -> bool {
    !expression.is_empty()
        && expression
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// The texts an argument stands for, or nothing when it cannot be read.
fn resolved(code: &str, expression: &str) -> Option<Vec<String>> {
    if let Some(text) = a_js_literal(expression) {
        return Some(vec![text]);
    }
    if expression.ends_with(".heardAs") {
        let every = every_heard_as(code);
        return (!every.is_empty()).then_some(every);
    }
    if is_a_name(expression) {
        return a_const_of_literals(code, expression).map(|text| vec![text]);
    }
    None
}

/// Every text one case waits for, and every argument it could not read.
fn what_a_case_waits_for(case: &str, source: &str) -> WhatTheCasesWaitFor {
    let code = without_js_comments(&source.replace("\r\n", "\n"));
    let letters: Vec<char> = code.chars().collect();
    let mut read = WhatTheCasesWaitFor::default();
    let mut take = |at: usize, expression: &str, how: &'static str| {
        let line = line_of(&letters, at);
        match resolved(&code, expression) {
            Some(texts) => read.waited.extend(texts.into_iter().map(|text| Waited {
                case: case.to_string(),
                line,
                text,
                how,
            })),
            None => read.unresolved.push(Unresolved {
                case: case.to_string(),
                line,
                expression: expression.to_string(),
            }),
        }
    };
    for (call, how) in [
        (WAITS, "waits for"),
        (TABS_TO, "tabs to"),
        (ASSERTS_IT_CONTAINS, "asserts"),
    ] {
        for (byte_at, _) in code.match_indices(call) {
            if call == ASSERTS_IT_CONTAINS && code[..byte_at].ends_with(".not") {
                continue;
            }
            if call != ASSERTS_IT_CONTAINS && is_part_of_a_longer_name(&code, byte_at) {
                continue;
            }
            let open = code[..byte_at + call.len() - 1].chars().count();
            let arguments = arguments_at(&letters, open);
            match (call, arguments.as_slice()) {
                (WAITS, [_, (at, array), ..]) => {
                    if array.starts_with('[') && array.ends_with(']') {
                        for (element_at, element) in arguments_at(&letters, *at) {
                            take(element_at, &element, how);
                        }
                    } else {
                        take(*at, array, how);
                    }
                }
                (TABS_TO, [_, (at, text), ..]) => take(*at, text, how),
                (ASSERTS_IT_CONTAINS, [(at, text)]) => take(*at, text, how),
                (_, _) => {}
            }
        }
    }
    read
}

fn is_part_of_a_longer_name(code: &str, byte_at: usize) -> bool {
    code[..byte_at]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// Every case file, by its file name.
fn the_case_files() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = fs::read_dir(THE_CASES)
        .expect("the NVDA cases to be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.to_string_lossy().ends_with(".test.js"))
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            (
                name,
                fs::read_to_string(&path).expect("a case to be readable"),
            )
        })
        .collect();
    cases.sort();
    cases
}

fn what_the_cases_wait_for(cases: &[(String, String)]) -> WhatTheCasesWaitFor {
    let mut all = WhatTheCasesWaitFor::default();
    for (case, source) in cases {
        let one = what_a_case_waits_for(case, source);
        all.waited.extend(one.waited);
        all.unresolved.extend(one.unresolved);
    }
    all
}

// ── What the program says ────────────────────────────────────────────────────

/// One string literal in shipped source.
#[derive(Clone, Debug)]
struct Literal {
    file: String,
    line: usize,
    /// Every function whose body it sits in, outermost first.
    functions: Vec<String>,
    /// As written between its quotes, escapes and mnemonics included.
    written: String,
    /// What it says once its escapes are read.
    said: String,
}

/// The shipped half of a file with every dropped line left blank, so a line
/// number counted in it is the file's own.
fn shipped_with_its_line_numbers(source: &str) -> String {
    let original: Vec<&str> = source.lines().collect();
    let ships = what_ships(source);
    let mut kept = vec![""; original.len()];
    let mut at = 0;
    for line in ships.lines() {
        while at < original.len() && original[at] != line {
            at += 1;
        }
        if at >= original.len() {
            break;
        }
        kept[at] = original[at];
        at += 1;
    }
    kept.join("\n")
}

/// Reads Rust source for its string literals, stepping over comments and
/// character literals and keeping track of the functions it is inside.
struct RustReader<'a> {
    letters: &'a [char],
    at: usize,
    line: usize,
}

impl RustReader<'_> {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.letters.get(self.at + ahead).copied()
    }

    fn advance(&mut self) {
        if self.letters.get(self.at) == Some(&'\n') {
            self.line += 1;
        }
        self.at += 1;
    }

    fn skip_line_comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let mut depth = 0;
        while let Some(here) = self.peek(0) {
            if here == '/' && self.peek(1) == Some('*') {
                depth += 1;
                self.advance();
            } else if here == '*' && self.peek(1) == Some('/') {
                depth -= 1;
                self.advance();
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    /// A character literal or a lifetime at a `'`.
    fn skip_quote(&mut self) {
        if self.peek(1) == Some('\\') {
            self.advance();
            self.advance();
            self.advance();
            while self.peek(0).is_some_and(|c| c != '\'') {
                self.advance();
            }
            self.advance();
        } else if self.peek(2) == Some('\'') {
            self.advance();
            self.advance();
            self.advance();
        } else {
            self.advance();
        }
    }

    /// An ordinary string whose opening quote is at the reader; answers what
    /// it holds as written and as said.
    fn read_string(&mut self) -> (String, String) {
        self.advance();
        let mut written = String::new();
        let mut said = String::new();
        while let Some(here) = self.peek(0) {
            if here == '"' {
                self.advance();
                break;
            }
            if here != '\\' {
                written.push(here);
                said.push(here);
                self.advance();
                continue;
            }
            written.push(here);
            self.advance();
            let Some(escaped) = self.peek(0) else { break };
            written.push(escaped);
            self.advance();
            match escaped {
                'n' => said.push('\n'),
                't' => said.push('\t'),
                'r' => said.push('\r'),
                '0' => said.push('\0'),
                'u' => {
                    let mut digits = String::new();
                    while let Some(c) = self.peek(0) {
                        written.push(c);
                        self.advance();
                        if c == '}' {
                            break;
                        }
                        if c.is_ascii_hexdigit() {
                            digits.push(c);
                        }
                    }
                    said.extend(
                        u32::from_str_radix(&digits, 16)
                            .ok()
                            .and_then(char::from_u32),
                    );
                }
                'x' => {
                    let digits: String = (0..2).filter_map(|i| self.peek(i)).collect();
                    written.push_str(&digits);
                    self.advance();
                    self.advance();
                    said.extend(u8::from_str_radix(&digits, 16).ok().map(char::from));
                }
                '\n' => {
                    while self.peek(0).is_some_and(char::is_whitespace) {
                        written.push(self.peek(0).unwrap_or(' '));
                        self.advance();
                    }
                }
                other => said.push(other),
            }
        }
        (written, said)
    }

    /// A raw string whose `r` is at the reader.
    fn read_raw_string(&mut self) -> String {
        self.advance();
        let mut hashes = 0;
        while self.peek(0) == Some('#') {
            hashes += 1;
            self.advance();
        }
        self.advance();
        let mut text = String::new();
        while let Some(here) = self.peek(0) {
            if here == '"' && (1..=hashes).all(|i| self.peek(i) == Some('#')) {
                for _ in 0..=hashes {
                    self.advance();
                }
                break;
            }
            text.push(here);
            self.advance();
        }
        text
    }

    fn read_word(&mut self) -> String {
        let mut word = String::new();
        while self
            .peek(0)
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            word.push(self.peek(0).unwrap_or('_'));
            self.advance();
        }
        word
    }

    /// Whether an `r` at the reader opens a raw string.
    fn a_raw_string_starts(&self) -> bool {
        self.a_raw_string_starts_after(0)
    }

    /// Whether an `r` `skipped` letters ahead opens a raw string.
    fn a_raw_string_starts_after(&self, skipped: usize) -> bool {
        if self.peek(skipped) != Some('r') {
            return false;
        }
        let mut ahead = skipped + 1;
        while self.peek(ahead) == Some('#') {
            ahead += 1;
        }
        self.peek(ahead) == Some('"')
    }
}

/// Every string literal in one file's shipped half.
fn literals_in(file: &str, source: &str) -> Vec<Literal> {
    let shipped = shipped_with_its_line_numbers(&source.replace("\r\n", "\n"));
    let letters: Vec<char> = shipped.chars().collect();
    let mut reader = RustReader {
        letters: &letters,
        at: 0,
        line: 1,
    };
    let mut found = Vec::new();
    let mut braces = 0i32;
    let mut brackets = 0i32;
    let mut inside: Vec<(String, i32)> = Vec::new();
    let mut naming: Option<String> = None;
    let mut push = |inside: &[(String, i32)], line: usize, written: String, said: String| {
        found.push(Literal {
            file: file.to_string(),
            line,
            functions: inside.iter().map(|(name, _)| name.clone()).collect(),
            written,
            said,
        });
    };
    while let Some(here) = reader.peek(0) {
        let before = reader
            .at
            .checked_sub(1)
            .and_then(|i| letters.get(i).copied());
        let starts_a_word = !before.is_some_and(|c| c.is_alphanumeric() || c == '_');
        match here {
            '/' if reader.peek(1) == Some('/') => reader.skip_line_comment(),
            '/' if reader.peek(1) == Some('*') => reader.skip_block_comment(),
            '"' => {
                let line = reader.line;
                let (written, said) = reader.read_string();
                push(&inside, line, written, said);
            }
            '\'' => reader.skip_quote(),
            'r' if starts_a_word && reader.a_raw_string_starts() => {
                let line = reader.line;
                let text = reader.read_raw_string();
                push(&inside, line, text.clone(), text);
            }
            // A byte string, read as the string after its `b`, raw or not.
            'b' if starts_a_word && reader.peek(1) == Some('"') => reader.advance(),
            'b' if starts_a_word && reader.a_raw_string_starts_after(1) => {
                reader.advance();
                let line = reader.line;
                let text = reader.read_raw_string();
                push(&inside, line, text.clone(), text);
            }
            'b' if starts_a_word && reader.peek(1) == Some('\'') => {
                reader.advance();
                reader.skip_quote();
            }
            '{' => {
                braces += 1;
                if let Some(name) = naming.take() {
                    inside.push((name, braces));
                }
                reader.advance();
            }
            '}' => {
                if inside.last().is_some_and(|(_, depth)| *depth == braces) {
                    inside.pop();
                }
                braces -= 1;
                reader.advance();
            }
            '(' | '[' => {
                brackets += 1;
                reader.advance();
            }
            ')' | ']' => {
                brackets -= 1;
                reader.advance();
            }
            ';' if brackets == 0 => {
                naming = None;
                reader.advance();
            }
            c if starts_a_word && (c.is_alphabetic() || c == '_') => {
                let word = reader.read_word();
                if word == "fn" {
                    while reader.peek(0).is_some_and(char::is_whitespace) {
                        reader.advance();
                    }
                    if reader
                        .peek(0)
                        .is_some_and(|c| c.is_alphabetic() || c == '_')
                    {
                        naming = Some(reader.read_word());
                    }
                }
            }
            _ => reader.advance(),
        }
    }
    found
}

fn rust_files(directory: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// Every literal in the shipped half of every file under `src/`.
fn the_programs_literals() -> &'static [Literal] {
    static LITERALS: OnceLock<Vec<Literal>> = OnceLock::new();
    LITERALS.get_or_init(|| {
        let mut files = Vec::new();
        rust_files(Path::new(THE_SOURCE), &mut files);
        files.sort();
        files
            .iter()
            .flat_map(|path| {
                let name = path.display().to_string().replace('\\', "/");
                let source = fs::read_to_string(path).unwrap_or_default();
                literals_in(&name, &source)
            })
            .collect()
    })
}

/// What NVDA reads for a label: a mnemonic's `&` taken out, `&&` read as `&`.
fn as_nvda_reads_it(label: &str) -> String {
    label
        .replace("&&", "\u{0}")
        .replace('&', "")
        .replace('\u{0}', "&")
}

/// A text's sentences, split after a full stop or a question mark followed by
/// a space.
fn sentences_of(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut letters = text.chars().peekable();
    while let Some(letter) = letters.next() {
        current.push(letter);
        if matches!(letter, '.' | '?') && letters.peek() == Some(&' ') {
            sentences.push(current.trim().to_string());
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    sentences
}

/// Whether a literal says `text`: as it is, or as NVDA reads it as a label,
/// holding every one of the text's sentences.
fn says(literal: &str, text: &str) -> bool {
    let sentences = sentences_of(text);
    [literal.to_string(), as_nvda_reads_it(literal)]
        .iter()
        .any(|form| {
            sentences
                .iter()
                .all(|sentence| form.contains(sentence.as_str()))
        })
}

/// A place that says a text.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    Literal {
        file: String,
        line: usize,
        written: String,
    },
    Built {
        how: String,
    },
}

impl Place {
    fn named(&self) -> String {
        match self {
            Place::Literal {
                file,
                line,
                written,
            } => format!("{file}:{line} {written:?}"),
            Place::Built { how } => how.clone(),
        }
    }
}

/// The sentences `status_sentences` builds, each with how it was built.
fn the_built_sentences() -> Vec<(String, String)> {
    Thing::ALL
        .into_iter()
        .flat_map(|thing| {
            [
                (
                    nothing_chosen(thing),
                    format!("status_sentences::nothing_chosen({:?})", thing.word()),
                ),
                (
                    at_least_one_chosen(thing),
                    format!("status_sentences::at_least_one_chosen({:?})", thing.word()),
                ),
            ]
        })
        .collect()
}

/// Every place that says `text`.
fn places_holding(text: &str, literals: &[Literal]) -> Vec<Place> {
    let mut places: Vec<Place> = literals
        .iter()
        .filter(|literal| says(&literal.said, text))
        .map(|literal| Place::Literal {
            file: literal.file.clone(),
            line: literal.line,
            written: literal.written.clone(),
        })
        .collect();
    places.extend(
        the_built_sentences()
            .into_iter()
            .filter(|(sentence, _)| sentence == text)
            .map(|(_, how)| Place::Built { how }),
    );
    places
}

// ── The judgement ────────────────────────────────────────────────────────────

/// What the reading decided about one text a case waits for.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    /// Held by exactly one place.
    SaidAt(Place),
    /// Held by the place a tie names.
    SaidWhereTheTieSays(Place),
    /// Named in the exception table.
    SaidElsewhere(&'static str),
    HeldByNothing,
    HeldBySeveralAndTiedToNone(Vec<Place>),
    TiedToAPlaceThatNoLongerSaysIt {
        tie: String,
        found: usize,
    },
    BuiltFromAPartNothingHolds(String),
}

impl Verdict {
    fn is_a_refusal(&self) -> bool {
        matches!(
            self,
            Verdict::HeldByNothing
                | Verdict::HeldBySeveralAndTiedToNone(_)
                | Verdict::TiedToAPlaceThatNoLongerSaysIt { .. }
                | Verdict::BuiltFromAPartNothingHolds(_)
        )
    }

    fn says(&self) -> String {
        match self {
            Verdict::SaidAt(place) => format!("said at {}", place.named()),
            Verdict::SaidWhereTheTieSays(place) => format!("said at {}, by its tie", place.named()),
            Verdict::SaidElsewhere(why) => format!("said elsewhere: {why}"),
            Verdict::HeldByNothing => "held by nothing the program says".to_string(),
            Verdict::HeldBySeveralAndTiedToNone(places) => format!(
                "held by {} places and tied to none: {}",
                places.len(),
                places
                    .iter()
                    .map(Place::named)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            Verdict::TiedToAPlaceThatNoLongerSaysIt { tie, found } => format!(
                "tied to a place that no longer says it: {tie}, where the literal occurs {found} \
                 times and must occur once and say the text"
            ),
            Verdict::BuiltFromAPartNothingHolds(part) => {
                format!("built from the part {part:?}, which nothing the program says holds")
            }
        }
    }
}

/// How many times a tie's literal, still saying the text, sits in its
/// function's body in its file.
fn times_the_tie_holds(tie: &Tie, text: &str, literals: &[Literal]) -> usize {
    literals
        .iter()
        .filter(|literal| literal.file == tie.file)
        .filter(|literal| literal.functions.iter().any(|name| name == tie.function))
        .filter(|literal| literal.written == tie.literal && says(&literal.said, text))
        .count()
}

fn tie_named(tie: &Tie) -> String {
    format!("{:?} in {} in {}", tie.literal, tie.function, tie.file)
}

/// The verdict on one text one case waits for.
fn judge(
    case: &str,
    text: &str,
    literals: &[Literal],
    said_elsewhere: &[SaidElsewhere],
    ties: &[Tie],
) -> Verdict {
    if let Some(entry) = said_elsewhere
        .iter()
        .find(|e| e.case == case && e.text == text)
    {
        return match &entry.said_by {
            SaidBy::NvdasOwnWords => Verdict::SaidElsewhere(entry.why),
            SaidBy::BuiltFrom(parts) => parts
                .iter()
                .find(|part| places_holding(part, literals).is_empty())
                .map_or(Verdict::SaidElsewhere(entry.why), |part| {
                    Verdict::BuiltFromAPartNothingHolds(part.to_string())
                }),
        };
    }
    if let Some(tie) = ties.iter().find(|tie| tie.case == case && tie.text == text) {
        let found = times_the_tie_holds(tie, text, literals);
        if found != 1 {
            return Verdict::TiedToAPlaceThatNoLongerSaysIt {
                tie: tie_named(tie),
                found,
            };
        }
        let place = literals
            .iter()
            .find(|literal| {
                literal.file == tie.file
                    && literal.written == tie.literal
                    && literal.functions.iter().any(|name| name == tie.function)
            })
            .map(|literal| Place::Literal {
                file: literal.file.clone(),
                line: literal.line,
                written: literal.written.clone(),
            });
        return place.map_or(Verdict::HeldByNothing, Verdict::SaidWhereTheTieSays);
    }
    let mut places = places_holding(text, literals);
    match places.len() {
        0 => Verdict::HeldByNothing,
        1 => Verdict::SaidAt(places.remove(0)),
        _ => Verdict::HeldBySeveralAndTiedToNone(places),
    }
}

/// Every text the cases wait for with its verdict, once per case and text.
fn verdicts(
    waited: &[Waited],
    literals: &[Literal],
    said_elsewhere: &[SaidElsewhere],
    ties: &[Tie],
) -> Vec<(Waited, Verdict)> {
    let mut judged = BTreeSet::new();
    waited
        .iter()
        .filter(|w| judged.insert((w.case.clone(), w.text.clone())))
        .map(|w| {
            (
                w.clone(),
                judge(&w.case, &w.text, literals, said_elsewhere, ties),
            )
        })
        .collect()
}

fn a_line_about(waited: &Waited, verdict: &Verdict) -> String {
    format!(
        "{}/{}:{} {} {:?}: {}",
        THE_CASES,
        waited.case,
        waited.line,
        waited.how,
        waited.text,
        verdict.says()
    )
}

// ── The tests ────────────────────────────────────────────────────────────────

#[test]
fn test_every_sentence_an_nvda_case_waits_for_is_one_the_program_says() {
    let cases = the_case_files();
    let read = what_the_cases_wait_for(&cases);
    let unexcused: Vec<String> = read
        .unresolved
        .iter()
        .filter(|u| {
            !WHAT_THE_READING_CANNOT_RESOLVE
                .iter()
                .any(|e| e.case == u.case && e.expression == u.expression)
        })
        .map(|u| format!("{}/{}:{} {}", THE_CASES, u.case, u.line, u.expression))
        .collect();
    assert!(
        unexcused.is_empty(),
        "a case waits for something this reading cannot turn into a text, and nothing says \
         why it may:\n  {}",
        unexcused.join("\n  ")
    );

    let judged = verdicts(
        &read.waited,
        the_programs_literals(),
        SAID_BY_SOMETHING_ELSE_OR_BUILT_FROM_PARTS,
        THE_ONE_PLACE_A_SHARED_TEXT_MEANS,
    );
    let refused: Vec<String> = judged
        .iter()
        .filter(|(_, verdict)| verdict.is_a_refusal())
        .map(|(waited, verdict)| a_line_about(waited, verdict))
        .collect();
    assert!(
        refused.is_empty(),
        "{} texts an NVDA case waits for are not said by the program at the place the case \
         means, so the case would wait for them on the runner until it timed out:\n  {}",
        refused.len(),
        refused.join("\n  ")
    );

    // Printed on the passing path, so a green run is a list and not a silence.
    println!("what the NVDA cases wait for, and what says it:");
    for (waited, verdict) in &judged {
        println!("  {}", a_line_about(waited, verdict));
    }
}

/// A planted literal in a planted file, for the companions.
fn planted(file: &str, source: &str) -> Vec<Literal> {
    literals_in(file, source)
}

fn a_planted_case(waits_for: &str) -> Vec<Waited> {
    what_a_case_waits_for(
        "planted.test.js",
        &format!(
            "await tabUntilHeard(nvda, \"Go\");\nawait waitToHearAll(nvda, [\"{waits_for}\"]);\n"
        ),
    )
    .waited
    .into_iter()
    .filter(|w| w.text == waits_for)
    .collect()
}

#[test]
fn test_the_reading_refuses_a_sentence_the_program_no_longer_says() {
    let source = planted(
        "src/planted.rs",
        "fn refuse() {\n    say(\"Nothing was picked, so nothing was done.\");\n}\n",
    );
    let old = a_planted_case("Pick a widget to delete.");
    let new = a_planted_case("Nothing was picked, so nothing was done.");
    assert_eq!(old.len(), 1, "the planted case was not read: {old:?}");
    assert_eq!(new.len(), 1, "the planted case was not read: {new:?}");

    assert_eq!(
        verdicts(&old, &source, &[], &[])[0].1,
        Verdict::HeldByNothing,
        "the old wording, which no literal says any more, was not refused"
    );
    assert_eq!(
        verdicts(&new, &source, &[], &[])[0].1,
        Verdict::SaidAt(Place::Literal {
            file: "src/planted.rs".to_string(),
            line: 2,
            written: "Nothing was picked, so nothing was done.".to_string(),
        }),
        "the new wording, which one literal says, was not held to it"
    );
}

#[test]
fn test_the_reading_refuses_a_short_name_whose_place_was_relabelled_while_others_still_hold_it() {
    let a_second_place = planted(
        "src/two.rs",
        "fn build_the_other_dialog() {\n    button().with_label(\"S&ync\");\n}\n",
    );
    let a_menu = planted(
        "src/three.rs",
        "fn the_menu() {\n    entry(\"&Sync calendar now\");\n}\n",
    );
    let the_place_the_case_means = |label: &str| {
        let mut all = planted(
            "src/one.rs",
            &format!("fn build_the_dialog() {{\n    button().with_label(\"{label}\");\n}}\n"),
        );
        all.extend(a_second_place.clone());
        all.extend(a_menu.clone());
        all
    };
    let tie = [Tie {
        case: "planted.test.js",
        text: "Sync",
        file: "src/one.rs",
        function: "build_the_dialog",
        literal: "S&ync",
        why: "planted",
    }];
    let case: Vec<Waited> =
        what_a_case_waits_for("planted.test.js", "await tabUntilHeard(nvda, \"Sync\");\n").waited;
    assert_eq!(case.len(), 1, "the planted case was not read: {case:?}");

    let as_planted = verdicts(&case, &the_place_the_case_means("S&ync"), &[], &tie);
    assert!(
        matches!(as_planted[0].1, Verdict::SaidWhereTheTieSays(_)),
        "the tie to the place the case means did not hold as planted: {:?}",
        as_planted[0].1
    );

    let relabelled = the_place_the_case_means("&Refresh");
    assert!(
        places_holding("Sync", &relabelled).len() == 2,
        "the planted tree should still hold \"Sync\" at the other dialog and in the menu"
    );
    assert_eq!(
        verdicts(&case, &relabelled, &[], &tie)[0].1,
        Verdict::TiedToAPlaceThatNoLongerSaysIt {
            tie: "\"S&ync\" in build_the_dialog in src/one.rs".to_string(),
            found: 0,
        },
        "the place the case means was relabelled and the reading took another place's word for it"
    );

    let untied = verdicts(&case, &the_place_the_case_means("S&ync"), &[], &[]);
    match &untied[0].1 {
        Verdict::HeldBySeveralAndTiedToNone(places) => assert_eq!(
            places.iter().map(Place::named).collect::<Vec<_>>(),
            vec![
                "src/one.rs:2 \"S&ync\"".to_string(),
                "src/two.rs:2 \"S&ync\"".to_string(),
                "src/three.rs:2 \"&Sync calendar now\"".to_string(),
            ],
            "the places holding the untied text were not all named"
        ),
        other => panic!("a text held by several places with no tie was not refused: {other:?}"),
    }
}

#[test]
fn test_the_reading_does_not_take_a_sentence_from_a_comment_or_a_test_module() {
    let from_a_comment = what_a_case_waits_for(
        "planted.test.js",
        "// await waitToHearAll(nvda, [\"In a comment.\"]);\n/* tabUntilHeard(nvda, \"Also\") */\n\
         await waitToHearAll(nvda, [\"Out of one.\"]);\n",
    );
    assert_eq!(
        from_a_comment
            .waited
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Out of one."],
        "a wait inside a JavaScript comment was read as one"
    );
    assert_eq!(
        from_a_comment.waited[0].line, 3,
        "the wait's line was not its own"
    );

    let source = planted(
        "src/planted.rs",
        "// say(\"Only in a comment.\");\nfn real() {\n    say(\"Said for real.\");\n}\n\n\
         #[cfg(test)]\nmod tests {\n    fn t() { say(\"Only in a test.\"); }\n}\n",
    );
    let case = |text: &str| a_planted_case(text);
    assert_eq!(
        verdicts(&case("Only in a comment."), &source, &[], &[])[0].1,
        Verdict::HeldByNothing
    );
    assert_eq!(
        verdicts(&case("Only in a test."), &source, &[], &[])[0].1,
        Verdict::HeldByNothing
    );
    assert!(
        matches!(
            verdicts(&case("Said for real."), &source, &[], &[])[0].1,
            Verdict::SaidAt(_)
        ),
        "the companion to the two above: a literal in shipped code was not taken"
    );
}

#[test]
fn test_the_reading_joins_a_sentence_rust_continues_onto_the_next_line() {
    let source = planted(
        "src/planted.rs",
        "fn say_it() {\n    say(\"The first half of it, \\\n         and the second half.\");\n}\n",
    );
    assert_eq!(
        source.iter().map(|l| l.said.as_str()).collect::<Vec<_>>(),
        vec!["The first half of it, and the second half."],
        "a literal continued with a backslash was not folded the way Rust folds it"
    );
    assert!(matches!(
        verdicts(
            &a_planted_case("The first half of it, and the second half."),
            &source,
            &[],
            &[]
        )[0]
        .1,
        Verdict::SaidAt(_)
    ));
}

#[test]
fn test_the_reading_found_every_case_and_what_each_waits_for() {
    let listed: BTreeSet<String> = fs::read_dir(THE_CASES)
        .expect("the NVDA cases to be listable")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".test.js"))
        .collect();
    let cases = the_case_files();
    let read: BTreeSet<String> = cases.iter().map(|(name, _)| name.clone()).collect();
    assert!(
        !listed.is_empty(),
        "no NVDA case was found under {THE_CASES}"
    );
    assert_eq!(
        read, listed,
        "the reading did not read every case the folder holds"
    );

    let waited = what_the_cases_wait_for(&cases).waited;
    let calls_one = |source: &str| {
        [WAITS, TABS_TO, ASSERTS_IT_CONTAINS]
            .iter()
            .any(|call| source.contains(call))
    };
    let silent: Vec<&String> = cases
        .iter()
        .filter(|(_, source)| calls_one(source))
        .map(|(name, _)| name)
        .filter(|name| !waited.iter().any(|w| &w.case == *name))
        .collect();
    assert!(
        silent.is_empty(),
        "these cases call one of the three and the reading found nothing they wait for: {silent:?}"
    );
}

#[test]
fn test_every_exception_names_a_text_a_case_still_waits_for() {
    let read = what_the_cases_wait_for(&the_case_files());
    let waits_for =
        |case: &str, text: &str| read.waited.iter().any(|w| w.case == case && w.text == text);

    let stale: Vec<String> = WHAT_THE_READING_CANNOT_RESOLVE
        .iter()
        .filter(|e| {
            !read
                .unresolved
                .iter()
                .any(|u| u.case == e.case && u.expression == e.expression)
        })
        .map(|e| format!("cannot resolve {} in {}", e.expression, e.case))
        .chain(
            SAID_BY_SOMETHING_ELSE_OR_BUILT_FROM_PARTS
                .iter()
                .filter(|e| !waits_for(e.case, e.text))
                .map(|e| format!("said elsewhere {:?} in {}", e.text, e.case)),
        )
        .chain(
            THE_ONE_PLACE_A_SHARED_TEXT_MEANS
                .iter()
                .filter(|t| !waits_for(t.case, t.text))
                .map(|t| format!("tie for {:?} in {}", t.text, t.case)),
        )
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name a text no case waits for any more, so they excuse nothing and \
         would excuse the next text written the same way; take them out:\n  {}",
        stale.join("\n  ")
    );

    let unexplained: Vec<&str> = WHAT_THE_READING_CANNOT_RESOLVE
        .iter()
        .map(|e| e.why)
        .chain(
            SAID_BY_SOMETHING_ELSE_OR_BUILT_FROM_PARTS
                .iter()
                .map(|e| e.why),
        )
        .chain(THE_ONE_PLACE_A_SHARED_TEXT_MEANS.iter().map(|t| t.why))
        .filter(|why| why.trim().is_empty())
        .collect();
    assert!(unexplained.is_empty(), "an entry carries no reason");
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

#[test]
fn test_this_target_runs_on_the_commits_that_could_break_it() {
    // A code commit answers `affected`, which reaches a target under `tests/`
    // only when that file itself changes, and a rewording lands in `src/`.
    let script = fs::read_to_string("scripts/check.sh").expect("the gate script to be readable");
    let whole_tree = the_whole_tree_targets(&script);
    assert!(
        whole_tree.iter().any(|target| target == ME),
        "a code commit runs {whole_tree:?} at the end of its scoped run and not {ME}, so a \
         sentence reworded in src/ strands the NVDA case waiting for it until the runner says so"
    );
}

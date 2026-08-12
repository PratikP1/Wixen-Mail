//! Which half of a source file a release build compiles.
//!
//! One answer, in one place. There were four, and three of them were wrong in
//! the same direction: they treated the first `#[cfg(test)]` anywhere in a
//! file as the end of the file. An indented `#[cfg(test)]` on a test-only
//! helper is ordinary Rust, and where one sat near the top of a file the
//! checks reading that file read almost nothing and passed against defects
//! sitting below it. One guard over an eleven thousand line file read the
//! first two hundred and twenty seven lines of it.
//!
//! The fourth answer took out whole test modules at the margin, which is
//! right, but ended a dropped item at the next line that was exactly `}`.
//! That is a module's ending and not a list's, so a `#[cfg(test)] const` at
//! the margin swallowed every line until some later brace, silently deleting
//! production text from what the check went on to read.
//!
//! Brackets are counted outside quotes and comments, which is not fussiness.
//! The first version of this counted every character and went wrong on the
//! first file it was pointed at, because a test module whose data is snippets
//! of Rust carries unbalanced braces inside quotes; the item ended in the
//! middle of the module and half a test module read as shipped code.
//!
//! What this cannot see, said plainly because a reader of source text always
//! has a blind spot: a `#[cfg(test)]` sharing a line with the item it gates is
//! not recognised at all, and neither is one written any other way, such as
//! `#[cfg(all(test, windows))]`. Nothing in this tree is written either way,
//! and the census that reads this checks that no way out of the program hides
//! behind one.

/// The half of `source` a release build compiles.
///
/// Every line survives except the items only a test build sees: a line whose
/// trimmed text is exactly `#[cfg(test)]`, any further attributes stacked
/// under it, and the item they gate. The item ends where its brackets close,
/// or at the first `;` or `,` if it opened none, so a module, a list, a
/// function and a struct field each end where they really end.
pub fn what_ships(source: &str) -> String {
    let mut ships: Vec<&str> = Vec::new();
    let mut reading = Reading::Code;
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() != "#[cfg(test)]" {
            let _ = code_only(line, &mut reading);
            ships.push(line);
            continue;
        }
        while lines
            .peek()
            .is_some_and(|next| next.trim().starts_with("#["))
        {
            let _ = code_only(lines.next().unwrap_or_default(), &mut reading);
        }
        let mut depth: i32 = 0;
        let mut opened = false;
        for dropped in lines.by_ref() {
            let code = code_only(dropped, &mut reading);
            let opening = count(&code, ['{', '[', '(']);
            depth += opening - count(&code, ['}', ']', ')']);
            opened |= opening > 0;
            let closes = opened || code.trim_end().ends_with([';', ',']);
            if depth <= 0 && closes {
                break;
            }
        }
    }
    ships.join("\n")
}

fn count(code: &str, brackets: [char; 3]) -> i32 {
    code.chars()
        .filter(|letter| brackets.contains(letter))
        .count() as i32
}

/// Where a line begins, when the line before it left something open.
#[derive(Clone, Copy, PartialEq)]
enum Reading {
    Code,
    Quoted,
    /// A raw string, and how many hashes close it.
    RawQuoted(usize),
    Commented,
}

/// `line` with everything inside quotes and comments taken out, and `reading`
/// left saying what the next line begins inside.
fn code_only(line: &str, reading: &mut Reading) -> String {
    let letters: Vec<char> = line.chars().collect();
    let mut kept = String::new();
    let mut at = 0;
    while at < letters.len() {
        match *reading {
            Reading::Quoted => {
                match letters[at] {
                    '\\' => at += 1,
                    '"' => *reading = Reading::Code,
                    _ => {}
                }
                at += 1;
            }
            Reading::RawQuoted(hashes) => {
                let closes = letters[at] == '"'
                    && letters[at + 1..]
                        .iter()
                        .take(hashes)
                        .all(|letter| *letter == '#')
                    && letters.len() >= at + 1 + hashes;
                match closes {
                    true => {
                        *reading = Reading::Code;
                        at += 1 + hashes;
                    }
                    false => at += 1,
                }
            }
            Reading::Commented => match letters[at] == '*' && letters.get(at + 1) == Some(&'/') {
                true => {
                    *reading = Reading::Code;
                    at += 2;
                }
                false => at += 1,
            },
            Reading::Code => match code_step(&letters, at, reading) {
                Step::Keep(letter) => {
                    kept.push(letter);
                    at += 1;
                }
                Step::SkipTo(next) => at = next,
                Step::RestIsAComment => break,
            },
        }
    }
    kept
}

/// What one step through code outside any quote or comment found.
enum Step {
    /// An ordinary character, which counts.
    Keep(char),
    /// A quote or a comment opened, so carry on from here.
    SkipTo(usize),
    /// Nothing further on this line is code.
    RestIsAComment,
}

fn code_step(letters: &[char], at: usize, reading: &mut Reading) -> Step {
    let here = letters[at];
    let next = letters.get(at + 1).copied();
    match here {
        '/' if next == Some('/') => return Step::RestIsAComment,
        '/' if next == Some('*') => {
            *reading = Reading::Commented;
            return Step::SkipTo(at + 2);
        }
        '\'' => return Step::SkipTo(past_a_character_literal(letters, at)),
        _ => {}
    }
    if let Some((hashes, quoting_from)) = a_raw_string_opening_at(letters, at) {
        *reading = Reading::RawQuoted(hashes);
        return Step::SkipTo(quoting_from);
    }
    if here == '"' {
        *reading = Reading::Quoted;
        return Step::SkipTo(at + 1);
    }
    Step::Keep(here)
}

/// Where a raw string opening at `at` starts quoting, and how many hashes
/// close it. `r"`, `r#"`, `br##"` and so on.
fn a_raw_string_opening_at(letters: &[char], at: usize) -> Option<(usize, usize)> {
    let a_name_character = |letter: &char| letter.is_alphanumeric() || *letter == '_';
    let mut reading_from = at;
    if letters[reading_from] == 'b' {
        reading_from += 1;
    }
    if letters.get(reading_from) != Some(&'r') {
        return None;
    }
    if at > 0 && letters[at - 1..at].iter().any(a_name_character) {
        return None;
    }
    let hashes = letters[reading_from + 1..]
        .iter()
        .take_while(|letter| **letter == '#')
        .count();
    match letters.get(reading_from + 1 + hashes) {
        Some('"') => Some((hashes, reading_from + 2 + hashes)),
        _ => None,
    }
}

/// Where to carry on from after a `'`, which opens a character literal in
/// some places and names a lifetime in others.
fn past_a_character_literal(letters: &[char], at: usize) -> usize {
    if letters.get(at + 1) == Some(&'\\') {
        let closing = letters[at + 2..].iter().position(|letter| *letter == '\'');
        return match closing {
            Some(away) => at + 3 + away,
            None => at + 1,
        };
    }
    match letters.get(at + 2) {
        Some('\'') => at + 3,
        _ => at + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::what_ships;

    #[test]
    fn test_a_test_module_at_the_margin_is_not_shipped() {
        let source = "fn first() {}\n\
                      #[cfg(test)]\n\
                      mod tests {\n    \
                          fn hidden() {}\n\
                      }\n\
                      fn second() {}";
        let ships = what_ships(source);
        assert!(ships.contains("fn first()"), "{ships}");
        assert!(ships.contains("fn second()"), "{ships}");
        assert!(!ships.contains("hidden"), "{ships}");
    }

    #[test]
    fn test_a_test_only_helper_part_way_down_does_not_hide_the_rest_of_the_file() {
        // The one that matters. This is the shape every naive reader in this
        // tree got wrong: an indented test-only helper high in a file, and
        // everything under it read as test code and never checked.
        let source = "impl Thing {\n    \
                          #[cfg(test)]\n    \
                          fn helper() -> Client {\n        \
                              Client::new()\n    \
                          }\n\n    \
                          fn erase(&self) {\n        \
                              self.http.request(reqwest::Method::DELETE, url);\n    \
                          }\n\
                      }";
        let ships = what_ships(source);
        assert!(!ships.contains("fn helper"), "{ships}");
        assert!(!ships.contains("Client::new()"), "{ships}");
        assert!(ships.contains("reqwest::Method::DELETE"), "{ships}");
        assert!(ships.contains("fn erase"), "{ships}");
    }

    #[test]
    fn test_a_test_only_list_does_not_swallow_what_follows_it() {
        // What the one careful reader in this tree got wrong: it ended a
        // dropped item at the next line that was exactly `}`, which a list
        // does not have, so it deleted production code as far as some later
        // brace and the check that read it went quiet.
        let source = "#[cfg(test)]\n\
                      const GATED: [&str; 2] = [\n    \
                          \"one.rs\",\n    \
                          \"two.rs\",\n\
                      ];\n\
                      fn still_here() {\n    \
                          reqwest::Client::new();\n\
                      }";
        let ships = what_ships(source);
        assert!(!ships.contains("GATED"), "{ships}");
        assert!(!ships.contains("one.rs"), "{ships}");
        assert!(ships.contains("fn still_here"), "{ships}");
        assert!(ships.contains("reqwest::Client::new()"), "{ships}");
    }

    #[test]
    fn test_a_test_only_field_ends_at_its_comma() {
        let source = "pub struct Thing {\n    \
                          #[cfg(test)]\n    \
                          pub http: reqwest::Client,\n    \
                          pub name: String,\n\
                      }";
        let ships = what_ships(source);
        assert!(!ships.contains("reqwest::Client"), "{ships}");
        assert!(ships.contains("pub name: String"), "{ships}");
    }

    #[test]
    fn test_stacked_attributes_under_the_gate_go_with_the_item() {
        let source = "#[cfg(test)]\n\
                      #[allow(dead_code)]\n\
                      fn only_for_tests() {}\n\
                      fn shipped() {}";
        let ships = what_ships(source);
        assert!(!ships.contains("only_for_tests"), "{ships}");
        assert!(!ships.contains("allow(dead_code)"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_a_brace_inside_a_quoted_string_does_not_end_the_item_early() {
        // Found by the census reading this very file: a test module whose
        // test data is snippets of Rust carries unbalanced braces inside
        // quotes, and counting those ends the dropped item somewhere in the
        // middle of it. The rest of the module then reads as shipped code.
        let source = "#[cfg(test)]\n\
                      mod tests {\n    \
                          fn one() { let a = \"}}}\"; }\n    \
                          fn two() { reqwest::Client::new(); }\n\
                      }\n\
                      fn shipped() {}";
        let ships = what_ships(source);
        assert!(!ships.contains("reqwest"), "{ships}");
        assert!(!ships.contains("fn two"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_a_brace_inside_a_raw_string_or_a_comment_does_not_end_the_item_early() {
        let source = "#[cfg(test)]\n\
                      mod tests {\n    \
                          fn one() { let a = r#\"}} \\\"}}\"#; } // }}}\n    \
                          fn two() { lettre::Address::new(); }\n\
                      }\n\
                      fn shipped() {}";
        let ships = what_ships(source);
        assert!(!ships.contains("lettre"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_a_string_that_runs_over_a_line_ending_keeps_its_braces_quiet() {
        // The continuation form this tree writes its snippets in: a backslash
        // at the line ending carries one string across several lines.
        let source = "#[cfg(test)]\n\
                      mod tests {\n    \
                          const SNIPPET: &str = \"mod inner {\\n\\\n                          }\";\n    \
                          fn two() { reqwest::get(u); }\n\
                      }\n\
                      fn shipped() {}";
        let ships = what_ships(source);
        assert!(!ships.contains("reqwest"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_a_character_literal_brace_is_not_a_brace() {
        let source = "#[cfg(test)]\n\
                      mod tests {\n    \
                          fn one() { matches!(c, '}' | '{') }\n    \
                          fn two() { reqwest::get(u); }\n\
                      }\n\
                      fn shipped() {}";
        let ships = what_ships(source);
        assert!(!ships.contains("reqwest"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_a_lifetime_is_not_the_start_of_a_character_literal() {
        // `'a` opens nothing. Reading it as a character literal would swallow
        // everything up to the next quote and lose the braces in between.
        let source = "#[cfg(test)]\n\
                      mod tests {\n    \
                          fn one<'a>(x: &'a str) -> &'a str { x }\n\
                      }\n\
                      fn shipped() { let y = 1; }";
        let ships = what_ships(source);
        assert!(!ships.contains("fn one"), "{ships}");
        assert!(ships.contains("fn shipped()"), "{ships}");
    }

    #[test]
    fn test_nothing_is_dropped_from_a_file_with_no_tests_in_it() {
        let source = "fn one() {}\nfn two() {\n    let a = [1, 2];\n}";
        assert_eq!(what_ships(source), source);
    }
}

//! The six words `CLAUDE.md` bans, as a failing build instead of a sentence.
//!
//! `CLAUDE.md` has said "avoid AI-slop vocabulary" since it was written, and
//! listed six words. Nothing ever read for them. On 2026-09-07 the tree held
//! eleven uses across eight files, and the rule had never once been enforced,
//! which is the same shape as every other rule in this repository that lived
//! only in a document: somebody has to notice it being broken.
//!
//! Three of the eleven were found by this check and by nothing before it. Two
//! greps had been run over the same tree that morning, and both reported seven,
//! because both matched a list of endings rather than a stem and neither list
//! held `-ness` or `-ly`. That is the whole argument for the shape of
//! [`THE_WORDS`], and it is worth reading before anyone simplifies it back.
//!
//! What makes this worth a whole target rather than two more tests in
//! `tests/house_style.rs`: that file is named by 18 guard records in their
//! `tests_last_seen` counts, so a test added to it puts a count-keyed
//! re-measurement run on the critical path. This file is named by no record,
//! so nothing is owed. `house_style` stayed at 64 tests across this change.
//!
//! # Why this file never spells the words it forbids
//!
//! It is inside the tree it reads. Written out whole, every one of the six
//! would be a finding against this file, so the check would fail on itself the
//! moment it worked. `tests/house_style.rs` hit the same wall with the dash
//! characters and answered it by building them from their code points; the
//! answer here is [`concat!`], which joins the pieces at compile time and
//! leaves no whole word in the source text on disk.
//!
//! That is a design, not a workaround, and it is the reason this file needs no
//! exemption for itself and no fixture files. The companion below splices its
//! violation into a real page's own lines **in memory**, so there is no fixture
//! on disk for a later reader to mistake for a real document, and nothing to
//! remember to exclude.
//!
//! # The one file this does not read, and what that costs
//!
//! `CLAUDE.md` states the rule, and stating it means listing the six words. A
//! document about a rule cannot be held to that rule, or saying what the rule
//! forbids becomes the offence. `tests/house_style.rs` already solved this
//! shape twice, for itself, in `ours_apart_from_this_file` and `test_sources`.
//!
//! The cost is real and is not hidden: slop anywhere else in `CLAUDE.md` is
//! unread. It is the whole file that is exempt, not the sentence, because
//! finding "the sentence that lists the words" needs a list of wordings, and
//! `tests/house_style.rs` records at length what happens to those. They are
//! always one wording behind.
//!
//! [`test_the_reading_can_see_one_on_the_page_it_does_not_read`] is what keeps
//! that exemption honest. It splices a violation into `CLAUDE.md`'s own lines
//! and requires the reading to find it, proving the exemption is the only thing
//! keeping this quiet there rather than a reading that cannot see.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// One word that says nothing, and what to write instead.
///
/// `stem` is what the reading matches, so one entry covers a word's whole
/// family without four entries and without a rule about English endings. That
/// is not a tidiness argument. Both the task brief for this change and the
/// first grep written to check it enumerated endings instead, as
/// `word(s|d|ing|ed|ment)?`, and both therefore reported that `.planning` held
/// none of these. It held two, and both are the `-ness` form, which neither
/// list happened to name. A stem cannot make that mistake.
///
/// `whole` is the plain dictionary form and `inflected` is one real longer
/// form, both used by the companions to build violations that are actual
/// English rather than a stem with letters after it.
///
/// All three are built from pieces. See the module doc: written out, they would
/// be findings against this file.
struct Empty {
    stem: &'static str,
    whole: &'static str,
    inflected: &'static str,
    instead: &'static str,
}

/// The six `CLAUDE.md` names, in the order it names them.
const THE_WORDS: &[Empty] = &[
    Empty {
        stem: concat!("del", "v"),
        whole: concat!("del", "ve"),
        inflected: concat!("del", "ving"),
        instead: "examine, look at, explain",
    },
    Empty {
        stem: concat!("rob", "ust"),
        whole: concat!("rob", "ust"),
        inflected: concat!("rob", "ustness"),
        instead: "solid, reliable, strong",
    },
    Empty {
        stem: concat!("seam", "less"),
        whole: concat!("seam", "less"),
        inflected: concat!("seam", "lessly"),
        instead: "smooth, easy, or say what actually happens",
    },
    Empty {
        stem: concat!("lever", "ag"),
        whole: concat!("lever", "age"),
        inflected: concat!("lever", "ages"),
        instead: "use",
    },
    Empty {
        stem: concat!("compre", "hensiv"),
        whole: concat!("compre", "hensive"),
        inflected: concat!("compre", "hensively"),
        instead: "full, complete, thorough",
    },
    Empty {
        stem: concat!("em", "power"),
        whole: concat!("em", "power"),
        inflected: concat!("em", "powering"),
        instead: "enable, let, allow",
    },
];

// ---------------------------------------------------------------------------
// The reading
// ---------------------------------------------------------------------------

/// Whether a word in this line begins with `stem`, and where.
///
/// A word begins where the character before it is not a letter, so
/// `comprehension` does not answer to the stem for a longer word, and
/// `unleveraged` does not answer at all because its word begins at the `u`.
/// Matching a whole word rather than a substring is what keeps "lever" and
/// "power" quiet, and both appear in this tree.
///
/// Case is ignored. A heading capitalises its first word, and seven of the
/// eleven uses found on 2026-09-07 were capitalised.
fn a_word_beginning_with(line: &str, stem: &str) -> Option<usize> {
    let lowered = line.to_lowercase();
    let letters: Vec<char> = lowered.chars().collect();
    let mut at = 0;
    while let Some(found) = lowered[at..].find(stem) {
        let start = at + found;
        let before_is_a_letter = lowered[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphabetic);
        if !before_is_a_letter {
            return Some(lowered[..start].chars().count());
        }
        at = start + 1;
        if at >= letters.len() {
            break;
        }
    }
    None
}

/// Every word that says nothing in this text, one finding per line per word.
///
/// Out here rather than inside the check, so the check and its companions run
/// the same reading over the same real files. A companion feeding literals to a
/// private helper would prove the helper and nothing about the files being
/// opened, being non-empty, or being read line by line, and those are the links
/// that break.
fn nothing_words_in(page: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for word in THE_WORDS {
            if a_word_beginning_with(line, word.stem).is_some() {
                found.push(format!(
                    "{page}:{}: {}. Write {} instead",
                    number + 1,
                    word.whole,
                    word.instead
                ));
            }
        }
    }
    found
}

// ---------------------------------------------------------------------------
// What it reads
// ---------------------------------------------------------------------------

/// The directories this reads, and the kinds of file in each.
///
/// The same corpus `ours()` in `tests/house_style.rs` reads. It is written
/// again here rather than shared, because moving that function into a module
/// both targets include would move text that guard records apply their breaks
/// to, and correcting those records costs a re-measurement run.
///
/// A second list drifts from the first, which is a shape this repository keeps
/// getting caught by, so
/// [`test_this_reads_the_same_tree_the_dash_rule_reads`] holds the two together
/// by reading the other function's source.
const THE_TREE: &[(&str, &[&str])] = &[
    ("src", &["rs"]),
    ("docs", &["md"]),
    (".planning", &["md"]),
    ("tests", &["rs"]),
    ("scripts", &["sh", "py", "ps1"]),
    ("guards", &["toml"]),
    ("installer", &["iss"]),
    (".github", &["yml"]),
];

/// The files that are not in a directory this walks.
///
/// `CLAUDE.md` is deliberately not here. See the module doc.
const THE_SINGLE_FILES: &[&str] = &["README.md", "Cargo.toml", ".gitignore", "build.rs"];

/// The one file left out, named once so the tests below can say why.
const THE_FILE_THAT_STATES_THE_RULE: &str = "CLAUDE.md";

fn collect(dir: &Path, extensions: &[&str], into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, extensions, into);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.contains(&e))
        {
            into.push(path);
        }
    }
}

fn the_prose_this_project_writes() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for (dir, extensions) in THE_TREE {
        collect(Path::new(dir), extensions, &mut found);
    }
    for single in THE_SINGLE_FILES {
        let path = PathBuf::from(single);
        if path.exists() {
            found.push(path);
        }
    }
    found
}

// ---------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------

#[test]
fn test_no_document_uses_a_word_that_says_nothing() {
    let mut found = Vec::new();

    for path in the_prose_this_project_writes() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        found.extend(nothing_words_in(&path.display().to_string(), &text));
    }

    assert!(
        found.is_empty(),
        "{} of these. Each one is a word that sounds like a claim and makes \
         none, and `CLAUDE.md` has asked for them to be avoided since it was \
         written:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// The companions
// ---------------------------------------------------------------------------

/// A page that is really read, really opened, and really has none of these.
///
/// `README.md` rather than a fixture, for the reason in the module doc: what
/// breaks is a file not being opened or not being read line by line, and only a
/// real file can prove those.
const A_REAL_PAGE: &str = "README.md";

#[test]
fn test_the_reading_can_see_one_on_a_real_page() {
    // The check above passes when the tree is clean, and from outside that is
    // indistinguishable from a check that read nothing at all. `CLAUDE.md`
    // records a guard here that sat passing over nothing for weeks while the
    // rule it was about lapsed five times. This is what tells the two apart.
    let text = fs::read_to_string(A_REAL_PAGE)
        .unwrap_or_else(|e| panic!("{A_REAL_PAGE} is a page this reads, and it {e}"));
    assert!(
        !text.trim().is_empty(),
        "{A_REAL_PAGE} opened and held nothing, so the check reads an empty \
         page and approves it"
    );

    // As it stands. Asserted rather than assumed: if this page ever does carry
    // one, this fails and names it, which is the direction to fail in.
    assert_eq!(
        nothing_words_in(A_REAL_PAGE, &text),
        Vec::<String>::new(),
        "{A_REAL_PAGE} already carries one, so the splice below is no longer \
         the only thing putting one there"
    );

    // And with one put into that page's own real lines, every word in turn, so
    // a stem that stopped matching is named rather than covered by its five
    // neighbours.
    let lines: Vec<&str> = text.lines().collect();
    let at = lines.len() / 2;
    for word in THE_WORDS {
        let mut spliced: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
        spliced.insert(
            at,
            format!("The design is {} and always has been.", word.whole),
        );

        assert_eq!(
            nothing_words_in(A_REAL_PAGE, &spliced.join("\n")),
            vec![format!(
                "{A_REAL_PAGE}:{}: {}. Write {} instead",
                at + 1,
                word.whole,
                word.instead
            )],
            "{} was put on line {} of {A_REAL_PAGE}'s real text and the reading \
             did not answer with exactly that page, that line and that word",
            word.whole,
            at + 1
        );
    }
}

#[test]
fn test_the_reading_can_see_one_on_the_page_it_does_not_read() {
    // The exemption for `CLAUDE.md` is the one thing in this file that could
    // quietly become a hole: if the reading stopped working there, the
    // exemption and the failure would look identical from outside, because both
    // produce silence. So the reading is run over that file's real lines with a
    // violation spliced in, and required to find it.
    //
    // This proves the exemption is a decision. It is not a claim that the file
    // is clean, which nothing here asks.
    let text = fs::read_to_string(THE_FILE_THAT_STATES_THE_RULE).unwrap_or_else(|e| {
        panic!("{THE_FILE_THAT_STATES_THE_RULE} states the rule this file checks, and it {e}")
    });

    let lines: Vec<&str> = text.lines().collect();
    let at = lines.len() / 2;
    let word = &THE_WORDS[0];
    let mut spliced: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
    spliced.insert(at, format!("You should {} into this.", word.whole));

    let found = nothing_words_in(THE_FILE_THAT_STATES_THE_RULE, &spliced.join("\n"));
    assert!(
        found.contains(&format!(
            "{THE_FILE_THAT_STATES_THE_RULE}:{}: {}. Write {} instead",
            at + 1,
            word.whole,
            word.instead
        )),
        "a violation was put on line {} of {THE_FILE_THAT_STATES_THE_RULE}'s \
         real text and the reading did not find it, so the silence over that \
         file is a broken reading rather than the exemption it is written to \
         be.\nFound: {found:?}",
        at + 1
    );

    // And that the file really is out of the walk, which is the other half.
    assert!(
        !the_prose_this_project_writes()
            .iter()
            .any(|path| path.ends_with(THE_FILE_THAT_STATES_THE_RULE)),
        "{THE_FILE_THAT_STATES_THE_RULE} is in the walk after all, so it will \
         be reported for listing the words it forbids"
    );
}

#[test]
fn test_the_reading_knows_a_word_from_a_word_that_merely_contains_it() {
    // A substring search would report every one of these, and three of them are
    // words this repository really uses. Measured against the tree rather than
    // imagined: "lever" and "power" both appear in `src/`, and "comprehension"
    // is ordinary in accessibility prose.
    for quiet in [
        "a lever moves the arm",
        "the power went out",
        "reading comprehension is not the same as decoding",
        "unleveraged is a word beginning with u",
        "problems are not this word",
    ] {
        assert_eq!(
            nothing_words_in("a", quiet),
            Vec::<String>::new(),
            "false alarm on: {quiet}"
        );
    }

    // And the endings a stem is there to cover, which a list of endings misses:
    // every sentence here is built from the entry itself rather than written
    // out, so this file stays clean of the words it refuses.
    for word in THE_WORDS {
        for loud in [
            format!("the design is {} and always was", word.whole),
            format!("a more {} approach", word.inflected),
            format!("{} is how they described it", word.whole.to_uppercase()),
        ] {
            let found = nothing_words_in("a", &loud);
            assert_eq!(found.len(), 1, "missed or doubled: {loud}");
            assert!(
                found[0].contains(word.whole),
                "{loud} was reported as something else: {}",
                found[0]
            );
        }
    }
}

#[test]
fn test_every_stem_is_the_front_of_its_own_whole_word() {
    // The two halves of an entry are written by hand and could disagree, which
    // would give a check that reports one word and matches another.
    for word in THE_WORDS {
        assert!(
            word.whole.starts_with(word.stem),
            "a stem that is not the front of the word it reports: {} against {}",
            word.stem,
            word.whole
        );
        assert!(
            word.inflected.starts_with(word.stem),
            "a longer form that the stem does not reach: {} against {}",
            word.inflected,
            word.stem
        );
        assert!(
            word.inflected.len() > word.whole.len(),
            "{} is not a longer form than {}, so it proves nothing a whole-word \
             match would have missed",
            word.inflected,
            word.whole
        );
        assert!(
            !word.instead.is_empty(),
            "{} is refused with no advice about what to write instead",
            word.whole
        );
    }
    assert_eq!(
        THE_WORDS.len(),
        6,
        "`CLAUDE.md` names six words. If that changed, this list is behind it."
    );
}

// ---------------------------------------------------------------------------
// That the two corpora have not drifted apart
// ---------------------------------------------------------------------------

/// Every string literal inside `ours()` in `tests/house_style.rs`.
///
/// Scoped to that function's body, because the same file names `src` and
/// `tests` again in `test_sources` for a different reason, and a whole-file
/// scan would answer with both.
fn the_literals_the_dash_rule_walks(source: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut inside = false;
    for line in source.lines() {
        if line.starts_with("fn ours() -> Vec<PathBuf> {") {
            inside = true;
            continue;
        }
        if inside && line == "}" {
            break;
        }
        if !inside {
            continue;
        }
        let mut rest = line;
        while let Some(open) = rest.find('"') {
            rest = &rest[open + 1..];
            let Some(close) = rest.find('"') else {
                break;
            };
            found.insert(rest[..close].to_string());
            rest = &rest[close + 1..];
        }
    }
    found
}

#[test]
fn test_this_reads_the_same_tree_the_dash_rule_reads() {
    // Two lists describing one corpus, so one of them will be edited and the
    // other will not. This is the check that says so on the commit that does
    // it, rather than months later when somebody notices that a directory added
    // to the dash rule was never read for anything else.
    let source = fs::read_to_string("tests/house_style.rs").expect("the dash rule to be readable");
    let theirs = the_literals_the_dash_rule_walks(&source);

    assert!(
        theirs.len() > 10,
        "the reading of ours() found {theirs:?}, which is not the list it walks"
    );

    let mut mine: BTreeSet<String> = BTreeSet::new();
    for (dir, extensions) in THE_TREE {
        mine.insert((*dir).to_string());
        for extension in *extensions {
            mine.insert((*extension).to_string());
        }
    }
    for single in THE_SINGLE_FILES {
        mine.insert((*single).to_string());
    }
    mine.insert(THE_FILE_THAT_STATES_THE_RULE.to_string());

    assert_eq!(
        mine,
        theirs,
        "the dash rule and this one no longer describe the same tree.\n  \
         only the dash rule walks: {:?}\n  only this one walks: {:?}",
        theirs.difference(&mine).collect::<Vec<_>>(),
        mine.difference(&theirs).collect::<Vec<_>>()
    );
}

#[test]
fn test_the_reading_of_the_other_walk_can_see_a_directory_go_missing() {
    // The companion to the check above, which is one of the kind that fails
    // quietly: a mistake in the parse returns an empty set, and an empty set
    // fails the comparison for the wrong reason while reading like the right
    // one.
    let source = fs::read_to_string("tests/house_style.rs").expect("the dash rule to be readable");

    assert!(
        the_literals_the_dash_rule_walks(&source).contains(".planning"),
        "the parse did not find .planning in ours(), so it is not reading the \
         function it is about"
    );
    assert!(
        !the_literals_the_dash_rule_walks(&source).contains("no_such_directory"),
        "the parse answers with names that are not in the function"
    );

    let without = source.replace(
        "collect(Path::new(\".planning\")",
        "collect(Path::new(\"gone\")",
    );
    assert!(
        !the_literals_the_dash_rule_walks(&without).contains(".planning"),
        "the parse still finds .planning after it was taken out of the source, \
         so it is not reading the source"
    );
}

// ---------------------------------------------------------------------------
// That this target runs on the commits that could break it
// ---------------------------------------------------------------------------

/// This file's own target name, which is what `cargo test --test` is given.
const ME: &str = "the_words_that_say_nothing";

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

#[test]
fn test_this_target_runs_on_the_commits_that_could_break_it() {
    // This reads prose, and prose arrives two ways. A documents-only commit
    // answers `docs_only`, so this has to be in that list. A document committed
    // beside code answers `affected`, which scopes the run to the modules that
    // changed and would never reach a target that reads the whole tree, so this
    // has to be in that list too. Neither list is derived from anything: both
    // are written out by hand, and nothing else would say if this fell out of
    // one.
    let script = fs::read_to_string("scripts/check.sh").expect("the gate script to be readable");

    let documents = the_documents_only_targets(&script);
    assert!(
        documents.iter().any(|target| target == ME),
        "a documents-only commit runs {documents:?} and not {ME}, so the check \
         that reads those documents does not run on the commits that write them"
    );

    let whole_tree = the_whole_tree_targets(&script);
    assert!(
        whole_tree.iter().any(|target| target == ME),
        "a document committed beside code runs {whole_tree:?} at the end of its \
         scoped run and not {ME}"
    );
}

#[test]
fn test_the_reading_of_what_the_gate_runs_can_see_this_target_missing() {
    // Both readings above find nothing when they break, and finding nothing
    // fails the assertions for the wrong reason while reading like the right
    // one.
    let script = fs::read_to_string("scripts/check.sh").expect("the gate script to be readable");

    assert!(
        the_documents_only_targets(&script)
            .iter()
            .any(|target| target == "house_style"),
        "the documents-only reading did not find house_style, which has read \
         documents since before this file existed"
    );
    assert!(
        the_whole_tree_targets(&script)
            .iter()
            .any(|target| target == "wired"),
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
    let without = script.replace(&format!(" {ME})"), ")");
    assert!(
        !the_whole_tree_targets(&without)
            .iter()
            .any(|target| target == ME),
        "the whole-tree reading still finds {ME} after it was taken out of the \
         script, so it is not reading the script"
    );
}

//! Every link between our own documents has to go somewhere.
//!
//! These files are not only read on GitHub, where a broken link is merely
//! annoying. `docs\*.md` ships inside the installer and sits beside the
//! program, and the first-run screen has a button that opens one of them. A
//! link to a file that was renamed, or never written, is a dead end for
//! somebody who is already having trouble.
//!
//! It has happened: `ALPHA_TESTING.md`, the page the first-run button opens,
//! pointed at `docs/DATA_LOCATIONS.md`, which has never existed.
//!
//! Links to the web are not checked. That needs the network, would make the
//! suite fail when somebody else's site is down, and is a different job.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A link found in a document, and where it points.
#[derive(Debug)]
struct Link {
    /// The document it was found in.
    from: PathBuf,
    /// The text between the brackets, so a failure names the link a person sees.
    label: String,
    /// The part before any `#`, empty for a link to a heading in the same file.
    file: String,
    /// The part after any `#`, empty when there is none.
    anchor: String,
}

/// Every markdown file that ships or is read on the repository page.
fn documents() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    collect_markdown(Path::new("docs"), &mut found);
    found.push(PathBuf::from("README.md"));
    found.push(PathBuf::from("CLAUDE.md"));
    found.sort();
    found
}

fn collect_markdown(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, into);
        } else if path.extension().is_some_and(|e| e == "md") {
            into.push(path);
        }
    }
}

/// Pull `[label](target)` out of one document, skipping the web and code spans.
///
/// Hand-rolled rather than pulling in a markdown parser: the shape being
/// looked for is small and exact, and a dependency that exists only to run one
/// test is a dependency somebody has to keep.
fn links_in(path: &Path) -> Vec<Link> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut links = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] != '[' {
            at += 1;
            continue;
        }
        let Some(label_end) = find(&bytes, at + 1, ']') else {
            break;
        };
        if bytes.get(label_end + 1) != Some(&'(') {
            at += 1;
            continue;
        }
        let Some(target_end) = find(&bytes, label_end + 2, ')') else {
            break;
        };

        let label: String = bytes[at + 1..label_end].iter().collect();
        let target: String = bytes[label_end + 2..target_end].iter().collect();
        at = target_end + 1;

        let target = target.trim();
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }

        let (file, anchor) = match target.split_once('#') {
            Some((file, anchor)) => (file, anchor),
            None => (target, ""),
        };
        links.push(Link {
            from: path.to_path_buf(),
            label,
            file: file.to_string(),
            anchor: anchor.to_string(),
        });
    }
    links
}

fn find(chars: &[char], from: usize, wanted: char) -> Option<usize> {
    chars[from..]
        .iter()
        .position(|c| *c == wanted)
        .map(|offset| from + offset)
}

/// The identifiers GitHub gives headings, which is what `#anchor` addresses.
///
/// Lowercased, punctuation dropped, spaces turned into hyphens. Close enough
/// for the headings this project writes; it would need more care for headings
/// carrying code spans or emoji, and there are none.
fn anchors_in(path: &Path) -> HashSet<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return HashSet::new();
    };
    text.lines()
        .filter_map(|line| line.trim_start().strip_prefix('#'))
        .map(|rest| rest.trim_start_matches('#').trim())
        .map(|heading| {
            heading
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect::<String>()
                .to_lowercase()
                .replace(' ', "-")
        })
        .collect()
}

#[test]
fn test_every_link_between_our_documents_goes_somewhere() {
    let mut dead = Vec::new();

    for document in documents() {
        let folder = document.parent().unwrap_or(Path::new(".")).to_path_buf();

        for link in links_in(&document) {
            let target = if link.file.is_empty() {
                document.clone()
            } else {
                folder.join(&link.file)
            };

            if !target.exists() {
                dead.push(format!(
                    "{}: [{}] points at {}, which is not there",
                    link.from.display(),
                    link.label,
                    target.display()
                ));
                continue;
            }

            if !link.anchor.is_empty() && !anchors_in(&target).contains(&link.anchor) {
                dead.push(format!(
                    "{}: [{}] points at a heading \"{}\" that {} does not have",
                    link.from.display(),
                    link.label,
                    link.anchor,
                    target.display()
                ));
            }
        }
    }

    assert!(dead.is_empty(), "dead links:\n  {}", dead.join("\n  "));
}

#[test]
fn test_the_check_is_actually_reading_links() {
    // Without this, deleting the body of `links_in` would leave the test above
    // passing on an empty list and reporting that every link is fine.
    let counted: usize = documents().iter().map(|d| links_in(d).len()).sum();

    assert!(
        counted > 20,
        "only {counted} links between our documents were found, which means the reader is broken"
    );
}

// ---------------------------------------------------------------------------
// Where a capability is described, rather than what is said about it.
//
// `docs/IMPLEMENTATION_STATUS.md` had a **Folder management** paragraph whose
// every sentence described making, renaming, moving, deleting, emptying and
// marking read a folder, in the present tense, with the two settings that
// decide how far the last two reach. All of it works. The paragraph sat under
// the heading `## What does not work`.
//
// So the page made a false claim through position rather than through wording.
// Somebody rewrote the paragraph when the feature shipped and left it where it
// was. A reader scanning headings, which is how a status page is read, got the
// false answer; a reader searching for the sentence that used to be wrong
// found nothing and concluded the page was already fixed. A check hunting for
// a claim of absence would have passed on the day the defect was live, which
// is why this one reads the heading instead.
// ---------------------------------------------------------------------------

/// The page that answers "does this work yet" by sorting things under headings.
const THE_STATUS_PAGE: &str = "docs/IMPLEMENTATION_STATUS.md";

/// The page that answers the same question with a tick beside each item.
const THE_ROADMAP: &str = "docs/roadmap.md";

/// A capability the two pages describe, paired with the code it cannot work
/// without.
///
/// One table rather than three lists, so the pairing is read in one place. Two
/// entries close the defect that prompted this; a third costs one more entry
/// and no other edit.
struct Capability {
    /// The bolded name opening the paragraph that describes it on the status
    /// page. That page writes these as `**Folder management.**`.
    on_the_page: &'static str,
    /// The wording of the roadmap's checkbox for it, where it has one.
    on_the_roadmap: Option<&'static str>,
    /// The file that must hold every symbol below.
    in_the_code: &'static str,
    /// What the capability cannot work without, as text that file must contain.
    symbols: &'static [&'static str],
}

const THE_CAPABILITIES: &[Capability] = &[
    Capability {
        on_the_page: "Folder management",
        on_the_roadmap: Some("CREATE, RENAME and DELETE"),
        in_the_code: "src/service/protocols/imap.rs",
        symbols: &[
            "fn create_mailbox",
            "fn rename_mailbox",
            "fn delete_mailbox",
        ],
    },
    Capability {
        on_the_page: "Threaded view",
        on_the_roadmap: Some("Thread view with conversation grouping"),
        in_the_code: "src/data/message_cache/messages.rs",
        symbols: &["fn conversations_in"],
    },
];

/// Where a capability is described: which section, and the lines of both.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DescribedUnder {
    heading: String,
    heading_line: usize,
    paragraph_line: usize,
}

/// Which `##` section of a page a capability's paragraph sits in.
///
/// `None` means the page no longer introduces it by that name. That is a
/// failure rather than nothing to say: a page reorganised out from under this
/// reading must not pass as a page with no faults.
fn described_under(text: &str, name: &str) -> Option<DescribedUnder> {
    let opening = format!("**{name}.**");
    let mut section: Option<(String, usize)> = None;

    for (index, line) in text.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("## ") {
            section = Some((rest.trim().to_string(), index + 1));
        }
        if line.trim_start().starts_with(&opening) {
            let (heading, heading_line) = section?;
            return Some(DescribedUnder {
                heading,
                heading_line,
                paragraph_line: index + 1,
            });
        }
    }
    None
}

/// Words that refuse.
const A_REFUSAL: &[&str] = &["not", "no", "cannot", "never"];

/// Words for a thing being there and doing its job.
const WORKING: &[&str] = &[
    "work",
    "works",
    "working",
    "built",
    "build",
    "done",
    "finished",
    "implemented",
    "supported",
    "there",
];

/// Whether a heading tells a reader that what sits under it does not work.
///
/// The refusal has to govern the working word, approximated by sitting within
/// two words of it, rather than the two merely appearing in the same heading.
/// Asking only for co-occurrence reads "Known gaps in verification" and "Which
/// tests would fail if the code were wrong" as refusals, and both sit over
/// things that do work. A predicate over word membership alone cannot tell a
/// sentence from its opposite, so the companion below asserts what this says
/// about every other heading on the real page, not only about the one it is
/// meant to catch.
fn a_heading_that_denies(heading: &str) -> bool {
    let words: Vec<String> = heading
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .collect();

    words.iter().enumerate().any(|(at, word)| {
        A_REFUSAL.contains(&word.as_str())
            && words
                .iter()
                .skip(at + 1)
                .take(2)
                .any(|next| WORKING.contains(&next.as_str()))
    })
}

/// Whether the code holds everything a capability cannot work without.
///
/// A real read of `src/`. The code's side of the pairing is the half that must
/// not be a constant: taken as one, this check would go on naming the page
/// after somebody deleted the feature, which is the page being right.
fn the_code_has(capability: &Capability) -> bool {
    let Ok(text) = fs::read_to_string(capability.in_the_code) else {
        return false;
    };
    capability.symbols.iter().all(|s| text.contains(s))
}

/// Every capability the code has that a page files under a heading saying it
/// does not.
///
/// Takes the page's text and the list, so the companion runs this same reading
/// over an invented page and an invented capability rather than over a copy of
/// it. The code is still read from disk, because that is the half being paired.
fn capabilities_filed_as_missing(
    page: &str,
    text: &str,
    capabilities: &[Capability],
) -> Vec<String> {
    let mut wrong = Vec::new();

    for capability in capabilities {
        let Some(found) = described_under(text, capability.on_the_page) else {
            wrong.push(format!(
                "{page}: nothing introduces \"{}\", so this check has stopped reading the page",
                capability.on_the_page
            ));
            continue;
        };

        if a_heading_that_denies(&found.heading) && the_code_has(capability) {
            wrong.push(format!(
                "{page}:{}: \"{}\" is described under \"{}\" at line {}, \
                 and {} holds {}",
                found.paragraph_line,
                capability.on_the_page,
                found.heading,
                found.heading_line,
                capability.in_the_code,
                capability.symbols.join(", ")
            ));
        }
    }
    wrong
}

#[test]
fn test_no_capability_the_code_has_is_filed_under_a_heading_denying_it() {
    let text = fs::read_to_string(THE_STATUS_PAGE)
        .unwrap_or_else(|e| panic!("{THE_STATUS_PAGE} is the status page, and it {e}"));

    let wrong = capabilities_filed_as_missing(THE_STATUS_PAGE, &text, THE_CAPABILITIES);

    assert!(
        wrong.is_empty(),
        "this page answers \"does this work yet\", and a heading is one of its \
         assertions. Each of these describes something the code has, under a \
         heading saying it does not work:\n  {}",
        wrong.join("\n  ")
    );
}

/// What the roadmap says about one item.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OnTheRoadmap {
    /// `- [ ]`, which says the thing is not built, and the line it is on.
    Unticked(usize),
    /// `- [x]`, which says it is.
    Ticked,
    /// No line carrying a box says this. A reading that has stopped working
    /// rather than a roadmap with nothing to say, so the caller reports it.
    NotFound,
}

/// What the roadmap says about one wording.
///
/// The roadmap answers the same question as the status page in a different
/// shape: a tick beside each item. A wrapped item's continuation line carries
/// no box, so the wording has to be found on the line that does.
fn on_the_roadmap(text: &str, wording: &str) -> OnTheRoadmap {
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.contains(wording) {
            continue;
        }
        if trimmed.starts_with("- [ ]") {
            return OnTheRoadmap::Unticked(index + 1);
        }
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            return OnTheRoadmap::Ticked;
        }
    }
    OnTheRoadmap::NotFound
}

/// Every capability the code has that the roadmap still lists as unticked.
///
/// The status page went wrong by position and this page goes wrong by a box
/// nobody ticked, which is the same defect in the shape this page has. Reading
/// only the status page would have left the claim alive here: it is where the
/// tree search found it after both documents the criterion names came out
/// clean or already corrected.
fn capabilities_the_roadmap_leaves_unticked(
    page: &str,
    text: &str,
    capabilities: &[Capability],
) -> Vec<String> {
    let mut wrong = Vec::new();

    for capability in capabilities {
        let Some(wording) = capability.on_the_roadmap else {
            continue;
        };
        match on_the_roadmap(text, wording) {
            OnTheRoadmap::Ticked => {}
            OnTheRoadmap::NotFound => wrong.push(format!(
                "{page}: no checkbox line says \"{wording}\", so this check has \
                 stopped reading the page"
            )),
            OnTheRoadmap::Unticked(line) if the_code_has(capability) => wrong.push(format!(
                "{page}:{line}: \"{wording}\" is unticked, and {} holds {}",
                capability.in_the_code,
                capability.symbols.join(", ")
            )),
            OnTheRoadmap::Unticked(_) => {}
        }
    }
    wrong
}

#[test]
fn test_no_capability_the_code_has_is_unticked_on_the_roadmap() {
    let text = fs::read_to_string(THE_ROADMAP)
        .unwrap_or_else(|e| panic!("{THE_ROADMAP} is the roadmap, and it {e}"));

    let wrong = capabilities_the_roadmap_leaves_unticked(THE_ROADMAP, &text, THE_CAPABILITIES);

    assert!(
        wrong.is_empty(),
        "an unticked box on this page says the thing is not built, and each of \
         these is:\n  {}",
        wrong.join("\n  ")
    );
}

/// A page that files a working capability under a heading denying it.
const A_PAGE_THAT_FILES_IT_WRONG: &str = "\
# Invented status page

## What works

**Sending.** It sends.

## What does not work

**Folder management.** A folder can be made, renamed and deleted.
";

/// The same page with the paragraph moved and nothing else changed.
const A_PAGE_THAT_FILES_IT_RIGHT: &str = "\
# Invented status page

## What works

**Sending.** It sends.

**Folder management.** A folder can be made, renamed and deleted.

## What does not work

**Moving a task between lists.** Not yet.
";

/// An invented roadmap: one box unticked, one ticked, one wording absent.
const AN_INVENTED_ROADMAP: &str = "\
# Invented roadmap

### IMAP
- [x] Message fetching
- [ ] CREATE, RENAME and DELETE, so folders can be managed here
- [ ] QRESYNC, so a folder can resume rather than re-list its UIDs

### Message list
- [x] Thread view with conversation grouping. Ctrl+T on the View menu collapses
      the list to one row per conversation
";

/// A capability named the same way on both pages and absent from the code.
const A_CAPABILITY_THE_CODE_DOES_NOT_HAVE: &[Capability] = &[Capability {
    on_the_page: "Folder management",
    on_the_roadmap: Some("CREATE, RENAME and DELETE"),
    in_the_code: "src/service/protocols/imap.rs",
    symbols: &["fn a_mailbox_verb_this_client_has_never_spoken"],
}];

#[test]
fn test_the_section_reading_can_tell_a_misfiled_capability_from_a_filed_one() {
    // The guard above reads a heading, and a guard that reads a document and
    // finds nothing to read passes. This is the companion `CLAUDE.md` asks
    // for. It proves four separate things, because the guard rests on four:
    // the page opens and every capability is found in it, the heading matcher
    // says yes to the denying heading and no to the rest, the position is what
    // decides, and the code probe really reads `src/`.

    let text = fs::read_to_string(THE_STATUS_PAGE)
        .unwrap_or_else(|e| panic!("{THE_STATUS_PAGE} is the status page, and it {e}"));
    assert!(
        !text.trim().is_empty(),
        "{THE_STATUS_PAGE} opened and held nothing, so the guard reads an empty page \
         and approves it"
    );

    // Every capability located on the real page. A name the reader could not
    // find would otherwise pass as a capability with nothing to say about it.
    for capability in THE_CAPABILITIES {
        assert!(
            described_under(&text, capability.on_the_page).is_some(),
            "\"{}\" is not introduced anywhere in {THE_STATUS_PAGE}, so the guard \
             has nothing to read about it and says nothing",
            capability.on_the_page
        );
    }

    // What the heading matcher says about the page's own headings. One denies;
    // the rest must not, or the guard would move every paragraph on the page.
    let headings: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .collect();
    assert!(
        headings.len() > 3,
        "only {} headings were found on {THE_STATUS_PAGE}, so the section reader \
         is not reading it",
        headings.len()
    );
    let denying: Vec<&&str> = headings
        .iter()
        .filter(|heading| a_heading_that_denies(heading))
        .collect();
    assert_eq!(
        denying,
        vec![&"What does not work"],
        "these are the headings on {THE_STATUS_PAGE} the matcher calls a refusal, \
         and it should be exactly the one. All of them: {headings:?}"
    );

    // Position is what decides. The same paragraph, the same code, two pages.
    assert_eq!(
        capabilities_filed_as_missing("invented.md", A_PAGE_THAT_FILES_IT_WRONG, THE_CAPABILITIES),
        vec![
            "invented.md:9: \"Folder management\" is described under \
             \"What does not work\" at line 7, and src/service/protocols/imap.rs \
             holds fn create_mailbox, fn rename_mailbox, fn delete_mailbox"
                .to_string(),
            "invented.md: nothing introduces \"Threaded view\", so this check has \
             stopped reading the page"
                .to_string(),
        ],
        "a working capability filed under a heading saying it does not work was \
         not named with its heading and its lines"
    );
    assert_eq!(
        capabilities_filed_as_missing(
            "invented.md",
            A_PAGE_THAT_FILES_IT_RIGHT,
            &THE_CAPABILITIES[..1]
        ),
        Vec::<String>::new(),
        "the paragraph was moved under a heading that is true of it and the check \
         still named it, so it is not reading the position"
    );

    // The code probe answers from `src/`, both ways round. Without this arm a
    // probe hardcoded to `true` would pass every assertion above.
    assert!(
        the_code_has(&THE_CAPABILITIES[0]),
        "the mailbox verbs are in {}, and the probe says they are not",
        THE_CAPABILITIES[0].in_the_code
    );
    assert!(
        !the_code_has(&A_CAPABILITY_THE_CODE_DOES_NOT_HAVE[0]),
        "the probe found a symbol this client has never had, so it is not reading \
         the file"
    );
    assert_eq!(
        capabilities_filed_as_missing(
            "invented.md",
            A_PAGE_THAT_FILES_IT_WRONG,
            A_CAPABILITY_THE_CODE_DOES_NOT_HAVE
        ),
        Vec::<String>::new(),
        "a capability the code does not have was named for sitting under a heading \
         saying so, which is the page being right"
    );
}

#[test]
fn test_the_roadmap_reading_can_tell_an_unticked_box_from_a_ticked_one() {
    // The same three things the companion above proves, for the page that
    // answers by ticking rather than by sorting: the file opens and every
    // wording is found in it, a tick and a blank are told apart, and the code
    // probe decides whether a blank is a fault or the truth.

    let text = fs::read_to_string(THE_ROADMAP)
        .unwrap_or_else(|e| panic!("{THE_ROADMAP} is the roadmap, and it {e}"));
    assert!(
        !text.trim().is_empty(),
        "{THE_ROADMAP} opened and held nothing, so the guard reads an empty page \
         and approves it"
    );

    // Every wording located on the real roadmap. A wording nothing carries
    // would otherwise pass as an item with nothing to say about it.
    for capability in THE_CAPABILITIES {
        let Some(wording) = capability.on_the_roadmap else {
            continue;
        };
        assert_ne!(
            on_the_roadmap(&text, wording),
            OnTheRoadmap::NotFound,
            "no checkbox line in {THE_ROADMAP} says \"{wording}\", so the guard \
             has nothing to read about \"{}\"",
            capability.on_the_page
        );
    }

    // A tick and a blank, told apart on an invented page whose lines are the
    // shapes the real one uses, including a wrapped item and an item that is
    // genuinely not built.
    assert_eq!(
        on_the_roadmap(AN_INVENTED_ROADMAP, "CREATE, RENAME and DELETE"),
        OnTheRoadmap::Unticked(5)
    );
    assert_eq!(
        on_the_roadmap(
            AN_INVENTED_ROADMAP,
            "Thread view with conversation grouping"
        ),
        OnTheRoadmap::Ticked
    );
    assert_eq!(
        on_the_roadmap(AN_INVENTED_ROADMAP, "APPEND for the Sent copy"),
        OnTheRoadmap::NotFound
    );
    // The continuation line of the wrapped item carries no box, so a reading
    // that matched anywhere would call this ticked item unticked or missing.
    assert_eq!(
        on_the_roadmap(AN_INVENTED_ROADMAP, "one row per conversation"),
        OnTheRoadmap::NotFound
    );

    assert_eq!(
        capabilities_the_roadmap_leaves_unticked(
            "invented.md",
            AN_INVENTED_ROADMAP,
            THE_CAPABILITIES
        ),
        vec![
            "invented.md:5: \"CREATE, RENAME and DELETE\" is unticked, and \
             src/service/protocols/imap.rs holds fn create_mailbox, \
             fn rename_mailbox, fn delete_mailbox"
                .to_string(),
        ],
        "the unticked box over working code was not named with its line, or the \
         ticked one was named as well"
    );

    // The code probe decides here too. An unticked box over code that is not
    // there is the roadmap being right, and must not be named.
    assert_eq!(
        capabilities_the_roadmap_leaves_unticked(
            "invented.md",
            AN_INVENTED_ROADMAP,
            A_CAPABILITY_THE_CODE_DOES_NOT_HAVE
        ),
        Vec::<String>::new(),
        "an unticked box over a capability the code does not have was named, \
         which is the roadmap being right"
    );
}

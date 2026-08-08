//! House style rules that a person should not have to keep noticing.
//!
//! A rule that lives only in `CLAUDE.md` is a rule somebody has to spot being
//! broken, say so, and watch get broken again. This is the em-dash rule as a
//! failing build instead: it had been pointed out before, and eighty-seven of
//! them were in the tree at the time it was pointed out again.

use std::fs;
use std::path::{Path, PathBuf};

/// The characters this checks for, built from their code points.
///
/// Written this way on purpose. Spelled out as literals they would appear in
/// this file, and this file is one of the ones being checked, so the test
/// would fail on itself.
fn banned() -> Vec<(char, &'static str)> {
    vec![
        (
            char::from_u32(0x2014).expect("em dash"),
            "em dash. Use a colon, a comma, or two sentences",
        ),
        (
            char::from_u32(0x2013).expect("en dash"),
            "en dash. Use \"to\" for a range, or a hyphen",
        ),
    ]
}

/// The files this project writes and is therefore responsible for.
///
/// Source, our own documents, and the handful of configuration files that
/// carry prose. Not `target`, not anything vendored.
fn ours() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &["rs"], &mut found);
    collect(Path::new("docs"), &["md"], &mut found);
    collect(Path::new("tests"), &["rs"], &mut found);
    collect(Path::new("scripts"), &["sh", "py", "ps1"], &mut found);
    collect(Path::new("guards"), &["toml"], &mut found);
    collect(Path::new("installer"), &["iss"], &mut found);
    collect(Path::new(".github"), &["yml"], &mut found);
    for single in ["README.md", "CLAUDE.md", "Cargo.toml", ".gitignore"] {
        let path = PathBuf::from(single);
        if path.exists() {
            found.push(path);
        }
    }
    found
}

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

#[test]
fn test_no_dashes_that_should_be_punctuation() {
    let mut found = Vec::new();

    for path in ours() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for (character, what) in banned() {
                if line.contains(character) {
                    found.push(format!(
                        "{}:{}: {what}\n      {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "{} of these, and they have been asked about more than once:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// The promise that sends somebody looking for a control nothing writes.
///
/// Built from two pieces so that this line is not itself a match. Written out
/// whole it would fail on the file that defines it, the same way the dash
/// characters above are built from their code points.
///
/// `AppConfig::allowed_per_account` is read and honoured, and the settings
/// screen writes the application-wide answer only, so nothing outside that
/// field's own tests has ever written one. The testing page and the first-run
/// screen both offered it. The shape they recommended is not one the code can
/// take either: a per-account entry can only ever narrow what the application
/// allows, never widen it.
const A_CONTROL_NO_SCREEN_WRITES: &str = concat!("set it ", "per account");

/// Each line of a document with the one after it, as one run of words.
///
/// A sentence in a document is wrapped wherever it reaches the margin, so a
/// phrase to look for is as likely to be split across two lines as to sit on
/// one. Read a line at a time, this check missed the very page it was written
/// for.
fn a_line_and_the_one_after_it(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .map(|(at, line)| {
            let next = lines.get(at + 1).copied().unwrap_or_default();
            (
                at + 1,
                format!("{} {}", line.trim(), next.trim())
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .collect()
}

#[test]
fn test_nothing_offers_a_setting_per_account_that_no_screen_writes() {
    let mut offered = Vec::new();

    for path in ours() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (number, run) in a_line_and_the_one_after_it(&text) {
            if run.contains(A_CONTROL_NO_SCREEN_WRITES) {
                offered.push(format!("{}:{number}: {run}", path.display()));
            }
        }
    }

    assert!(
        offered.is_empty(),
        "Allow Changes is one answer for the whole application, and nothing \
         writes an answer for one account. An answer for one account could only \
         ever narrow the application-wide one anyway, so what these offer is a \
         control that is not there and a shape the code cannot take:\n  {}",
        offered.join("\n  ")
    );
}

// ── What a new installation allows ──────────────────────────────────────────
//
// One answer lives in the code: `data::config`'s `default_allowed`. Prose
// contradicting it has now been written into this repository nine times in nine
// wordings, and the check that stood here to stop the fifth missed every one of
// the five that came after it. It held a list of the wordings already found, so
// a new wording walked straight past, and it required the setting to be named
// on the same line, which the copy in the module that defines the setting does
// not do and never could.
//
// A list of wordings is always one wording behind. This reads the claim
// instead. It finds prose that puts itself at installation time, works out
// which of the two answers the sentence is about and whether it says anything
// reaches a provider, and asks the code which is true. Nothing here writes down
// what the shipped answer is, so if that answer changes this follows it.

/// The files this rule reads: ours, apart from this one.
///
/// Left out for the reason the temporary-folder rule further down is left out
/// of its own walk. A check like this is worth having only if something proves
/// it can see, and proving that means writing the false sentences out in full,
/// here. There is nothing to disguise them as either: the thing being read is
/// ordinary English, so there is no equivalent of building a dash from its code
/// point. Anything that walks past this comment and quiets a failure by
/// loosening the reading below has turned a check into decoration.
fn ours_apart_from_this_file() -> Vec<PathBuf> {
    ours()
        .into_iter()
        .filter(|path| !path.ends_with("house_style.rs"))
        .collect()
}

/// Words that put a sentence at installation time.
///
/// A closed class, and about *when* rather than about how somebody happened to
/// word it. "default" on its own covers "the default", "by default" and "the
/// shipped default", which between them are seven of the nine copies; the list
/// this replaces held four whole wordings and so knew none of the five that
/// followed.
const PUTS_A_SENTENCE_AT_INSTALLATION_TIME: &[&str] = &[
    "default",
    "defaults",
    "shipped",
    "new installation",
    "new install",
    "out of the box",
    "when you install",
    "a new account starts",
    "starts out with",
];

/// What a sentence says reaches a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reaches {
    /// Nothing goes out at all.
    Nothing,
    /// Something does.
    Something,
}

/// Words saying nothing goes out, and words saying something does.
///
/// Short on purpose. These are the words English has for the two answers, not
/// the words these nine sentences happened to use.
const NOTHING_GOES_OUT: &[&str] = &[
    "off",
    "nothing",
    "none",
    "no",
    "not",
    "never",
    "cannot",
    "refuse",
    "refuses",
    "refused",
    "read-only",
];

/// The other half of [`NOTHING_GOES_OUT`].
const SOMETHING_GOES_OUT: &[&str] = &[
    "on",
    "allow",
    "allows",
    "allowed",
    "permit",
    "permits",
    "permitted",
    "send",
    "sends",
    "sent",
    "goes",
    "reach",
    "reaches",
];

/// Which of the two answers a sentence is about.
///
/// `Allowed` is two booleans because the two cost different amounts to get
/// wrong. A sentence saying mail is refused is true, the same sentence about
/// tasks is false, and reading both as one claim would call the true one a lie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
    /// Sending a message, and changing or deleting one on a server.
    Mail,
    /// Tasks, contacts and the calendar.
    PersonalInformation,
    /// The sentence does not say, so it is a claim about both.
    Either,
}

/// Words naming the mail answer, and words naming the other one.
const NAMES_THE_MAIL_ANSWER: &[&str] = &["mail", "message", "messages", "mailbox", "inbox"];

/// The other half of [`NAMES_THE_MAIL_ANSWER`].
const NAMES_THE_OTHER_ANSWER: &[&str] = &[
    "task",
    "tasks",
    "contact",
    "contacts",
    "calendar",
    "calendars",
    "event",
    "events",
];

/// One run of prose: comment lines that follow one another, or a paragraph of a
/// document, flattened into a single line with its source lines remembered.
///
/// Flattened because a sentence is wrapped wherever it reaches the margin, so a
/// claim is as likely to be split across two lines as to sit on one. Read a
/// line at a time, this rule missed the page it was written for.
struct Prose {
    text: String,
    /// Where each source line starts in `text`, and which line it is.
    starts: Vec<(usize, usize)>,
    /// Whether this is a module's own documentation rather than a comment.
    documents_the_module: bool,
}

impl Prose {
    /// The source line an offset into `text` came from.
    fn line_at(&self, offset: usize) -> usize {
        self.starts
            .iter()
            .rev()
            .find(|(start, _)| *start <= offset)
            .map_or(0, |(_, line)| *line)
    }
}

/// The prose of one line, or nothing when the line is code.
///
/// A rule about what a sentence claims has no business reading an identifier.
/// `default_allowed` is the function that holds the answer, and split into
/// words it reads as "default allowed", which is a claim nobody made.
///
/// A table in a document is left out as well. It is a list rather than a
/// sentence, its cells run together into one long line with no sentence in it,
/// and the one table here is right.
fn the_prose_of(path: &Path, line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim_start();
    let mut documents_the_module = false;
    let words = match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => {
            let after = trimmed.strip_prefix("//")?;
            documents_the_module = after.starts_with('!');
            after.trim_start_matches(['/', '!'])
        }
        Some("md") => {
            if trimmed.starts_with('|') || line.starts_with("    ") || line.starts_with('\t') {
                return None;
            }
            line
        }
        _ => trimmed.strip_prefix('#')?,
    };
    Some((
        without_code_and_addresses(words).trim().to_string(),
        documents_the_module,
    ))
}

/// Every run of prose in one file.
fn prose_in(path: &Path, text: &str) -> Vec<Prose> {
    let markdown = path.extension().is_some_and(|e| e == "md");
    let mut found: Vec<Prose> = Vec::new();
    let mut inside_a_fence = false;
    let mut previous = 0;

    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if markdown && line.trim_start().starts_with("```") {
            inside_a_fence = !inside_a_fence;
            previous = 0;
            continue;
        }
        let words = if inside_a_fence {
            None
        } else {
            the_prose_of(path, line).filter(|(words, _)| !words.is_empty())
        };
        let Some((words, documents_the_module)) = words else {
            previous = 0;
            continue;
        };
        match found.last_mut().filter(|_| number == previous + 1) {
            Some(run) => {
                run.starts.push((run.text.len() + 1, number));
                run.text.push(' ');
                run.text.push_str(&words);
            }
            None => found.push(Prose {
                starts: vec![(0, number)],
                text: words,
                documents_the_module,
            }),
        }
        previous = number;
    }
    found
}

/// Whether this run of prose is about the one setting this rule is about.
///
/// The setting by name, and one exception: the module documentation of the
/// module that defines it. That is where the fifth copy sat, and it names
/// nothing because it is the definition; it says "they" and means the paths it
/// has just listed.
///
/// The exception is that module's documentation and not the whole file, because
/// that file is also the one place where "the default" can honestly mean
/// something else. `Allowed::default()` is nothing, deliberately, so that a
/// value built without anybody deciding cannot damage anything, and the
/// comments on its tests say so and are right.
///
/// Anything wider than this cries wolf, which is worse than useless. "Provider"
/// on its own drags in the default task list, the default account, the default
/// calendar and the setting that checks links against Google's lists, none of
/// which is this.
fn about_the_setting(prose: &Prose, path: &Path) -> bool {
    let lowered = prose.text.to_lowercase();
    lowered.contains(&wixen_mail::application::allowed::SETTINGS_SECTION.to_lowercase())
        || (prose.documents_the_module && path.ends_with(WHERE_THE_SETTING_IS_DEFINED))
}

/// The module whose documentation is about this setting without naming it.
const WHERE_THE_SETTING_IS_DEFINED: &str = "application/allowed.rs";

/// A word of a run of prose, with where it starts.
fn words_of(text: &str) -> Vec<(usize, String)> {
    let mut words = Vec::new();
    let mut at = 0;
    for piece in text.split_inclusive(char::is_whitespace) {
        let bare = piece.trim_matches(|c: char| !c.is_alphanumeric());
        if !bare.is_empty() {
            words.push((
                at + (piece.len() - piece.trim_start().len()),
                bare.to_lowercase(),
            ));
        }
        at += piece.len();
    }
    words
}

/// Where the sentence around an offset starts and ends.
///
/// A sentence rather than the whole run, because a run says several things and
/// only the sentence carrying the installation-time word is making this claim.
fn the_sentence_around(text: &str, at: usize) -> (usize, usize) {
    let ends_it = |c: char| matches!(c, '.' | '!' | '?' | ':' | ';');
    let start = text[..at].rfind(ends_it).map_or(0, |found| found + 1);
    let end = text[at..]
        .find(ends_it)
        .map_or(text.len(), |found| at + found);
    (start, end)
}

/// The sentence with the name of the setting blanked out.
///
/// "Allow Changes" is what the settings screen calls the section, so the word
/// "Allow" in it is a label and not the sentence saying anything is allowed.
/// Left in, every sentence naming the setting reads as permission.
///
/// Blanked rather than cut out, so every other word stays where it was and an
/// offset into the sentence still means what it meant. Cut out, a name after
/// the word being read moved that word, and the reading landed on its
/// neighbour.
fn without_the_settings_name(sentence: &str) -> String {
    let named = wixen_mail::application::allowed::SETTINGS_SECTION;
    let lowered = sentence.to_lowercase();
    let mut left = sentence.to_string();
    let mut at = 0;
    while let Some(found) = lowered[at..].find(&named.to_lowercase()) {
        let start = at + found;
        left.replace_range(start..start + named.len(), &" ".repeat(named.len()));
        at = start + named.len();
    }
    left
}

/// What a sentence says reaches a provider, if it says anything.
///
/// Read from the word nearest the one that puts the sentence at installation
/// time. That is where the claim is made: "off is the shipped default",
/// "switched off by default", "a new installation allows tasks". A sentence
/// carrying both answers is read from the one it puts beside that word, so
/// "a new installation allows tasks, and this test turns that off" is read as
/// permission and not as a refusal.
///
/// What it cannot read, so that a quiet run is not taken for more than it is. A
/// sentence that carries no word from either list: "the setting is the shipped
/// default" says which answer it is talking about and never says what that
/// answer is, and one of the nine copies is exactly that. A negation of a
/// negation. A conditional, "if it were off by default". And a claim spread
/// over two sentences, because only the sentence carrying the word is read.
fn what_it_says_reaches_a_provider(sentence: &str, marker_at: usize) -> Option<Reaches> {
    let readable = without_the_settings_name(sentence);
    let words = words_of(&readable);
    let marker_word = words
        .iter()
        .rposition(|(at, _)| *at <= marker_at)
        .unwrap_or(0);

    let said = negation_read_as_refusal(&words);

    let mut nothing = None;
    let mut something = None;
    for (index, verdict) in said.iter().enumerate() {
        let away = index.abs_diff(marker_word);
        match verdict {
            Some(Reaches::Nothing) => {
                nothing = Some(nothing.map_or(away, |near: usize| near.min(away)));
            }
            Some(Reaches::Something) => {
                something = Some(something.map_or(away, |near: usize| near.min(away)));
            }
            None => {}
        }
    }
    match (nothing, something) {
        (None, None) => None,
        (Some(_), None) => Some(Reaches::Nothing),
        (None, Some(_)) => Some(Reaches::Something),
        // A tie is a sentence that says both things at the same distance, which
        // is a sentence nobody should be named for on a tie-break. It goes to
        // permission, and what that costs is a genuinely ambiguous refusal
        // going unsaid.
        (Some(near), Some(far)) => Some(if near < far {
            Reaches::Nothing
        } else {
            Reaches::Something
        }),
    }
}

/// What each word says, with a negated permission read as a refusal.
///
/// "sends nothing" is not the sentence permitting anything, and neither is
/// "allows none of it" or "is not sent". A word from one list standing next to
/// a word from the other is a negation, and what the pair means is the refusal.
fn negation_read_as_refusal(words: &[(usize, String)]) -> Vec<Option<Reaches>> {
    let mut said: Vec<Option<Reaches>> = words
        .iter()
        .map(|(_, word)| {
            if NOTHING_GOES_OUT.contains(&word.as_str()) {
                Some(Reaches::Nothing)
            } else if SOMETHING_GOES_OUT.contains(&word.as_str()) {
                Some(Reaches::Something)
            } else {
                None
            }
        })
        .collect();

    let refused = |at: usize| said.get(at) == Some(&Some(Reaches::Nothing));
    let negated: Vec<usize> = (0..said.len())
        .filter(|at| said[*at] == Some(Reaches::Something))
        .filter(|at| at.checked_sub(1).is_some_and(refused) || refused(at + 1))
        .collect();
    for at in negated {
        said[at] = None;
    }
    said
}

/// Which of the two answers a sentence is about.
fn which_answer_it_is_about(sentence: &str) -> Answer {
    let lowered = sentence.to_lowercase();
    let words = words_of(&lowered);
    let named = |names: &[&str]| words.iter().any(|(_, word)| names.contains(&word.as_str()));
    let mail = named(NAMES_THE_MAIL_ANSWER);
    let other = named(NAMES_THE_OTHER_ANSWER)
        || lowered.contains("personal information")
        || lowered.contains("address book");

    match (mail, other) {
        (true, false) => Answer::Mail,
        (false, true) => Answer::PersonalInformation,
        _ => Answer::Either,
    }
}

/// One sentence claiming what a new installation lets out.
struct Claim {
    line: usize,
    sentence: String,
    answer: Answer,
    reaches: Option<Reaches>,
}

/// Every claim in one file about what a new installation lets out.
///
/// One report per sentence rather than one per word, so "the shipped default",
/// which carries two of the installation-time words, is one claim.
fn claims_about_a_new_installation(path: &Path, text: &str) -> Vec<Claim> {
    let mut claims = Vec::new();
    for prose in prose_in(path, text) {
        if !about_the_setting(&prose, path) {
            continue;
        }
        let lowered = prose.text.to_lowercase();
        let mut said_already: Vec<(usize, usize)> = Vec::new();
        for phrase in PUTS_A_SENTENCE_AT_INSTALLATION_TIME {
            for at in whole_words_at(&lowered, phrase) {
                let (start, end) = the_sentence_around(&prose.text, at);
                if said_already.contains(&(start, end)) {
                    continue;
                }
                said_already.push((start, end));
                let sentence = prose.text[start..end].trim().to_string();
                claims.push(Claim {
                    line: prose.line_at(at),
                    answer: which_answer_it_is_about(&sentence),
                    reaches: what_it_says_reaches_a_provider(&prose.text[start..end], at - start),
                    sentence,
                });
            }
        }
    }
    claims
}

/// Every place a phrase appears as a whole word rather than inside one.
///
/// "default" is a word here and half of `default_allowed` there, and the second
/// is the code holding the answer rather than a sentence about it.
fn whole_words_at(lowered: &str, phrase: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = lowered[at..].find(phrase) {
        let start = at + offset;
        let end = start + phrase.len();
        let before = lowered[..start].chars().next_back();
        let after = lowered[end..].chars().next();
        if !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            found.push(start);
        }
        at = end;
    }
    found
}

/// What the code says a new installation lets out, for one of the two answers.
fn the_shipped_answer(answer: Answer) -> bool {
    let shipped = wixen_mail::data::config::AppConfig::default().allowed_changes;
    match answer {
        Answer::Mail => shipped.mail,
        Answer::PersonalInformation => shipped.personal_information,
        Answer::Either => shipped.anything(),
    }
}

/// Whether a claim agrees with the code, or `None` when it cannot be read.
fn agrees_with_the_code(claim: &Claim) -> Option<bool> {
    claim
        .reaches
        .map(|reaches| (reaches == Reaches::Something) == the_shipped_answer(claim.answer))
}

#[test]
fn test_nothing_says_a_new_installation_changes_nothing_while_it_changes_contacts() {
    // What a new installation allows is one value in the code, and every
    // sentence about it here is checked against that value rather than against
    // a sentence written down. If the shipped answer ever becomes "change
    // nothing" this goes quiet on its own instead of having to be remembered.
    let mut wrong = Vec::new();

    for path in ours_apart_from_this_file() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for claim in claims_about_a_new_installation(&path, &text) {
            if agrees_with_the_code(&claim) == Some(false) {
                wrong.push(format!(
                    "{}:{}: reads as {:?} about {:?}, and the code says {}: {}",
                    path.display(),
                    claim.line,
                    claim.reaches,
                    claim.answer,
                    the_shipped_answer(claim.answer),
                    claim.sentence
                ));
            }
        }
    }

    let shipped = wixen_mail::data::config::AppConfig::default().allowed_changes;
    assert!(
        wrong.is_empty(),
        "a new installation may change tasks, contacts and the calendar, and \
         may not send or change mail. Saying it changes nothing sends somebody \
         to their real address book believing it is safe from this. The code's \
         own answer is mail {}, personal information {}:\n  {}",
        shipped.mail,
        shipped.personal_information,
        wrong.join("\n  ")
    );
}

#[test]
fn test_no_comment_names_the_shipped_answer_without_saying_what_it_is() {
    // The ninth copy is "The setting is the shipped default", above a test that
    // turns the setting off. It names which answer it is talking about and
    // never says what that answer is, so there is nothing in the sentence to
    // compare with the code, and the reading above passes over it in silence.
    //
    // What is left is the shape: a comment that calls something the shipped
    // answer while leaving the reader to supply it from the code around it.
    // That is the sentence that goes stale without anybody being able to see
    // it has, so it is refused outright. Say what the answer is and the check
    // above will hold you to it.
    //
    // Source only. A document says "Default" in the heading of a table whose
    // cells carry the answer, and a table is not a sentence.
    let mut unsaid = Vec::new();

    for path in ours_apart_from_this_file() {
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for claim in claims_about_a_new_installation(&path, &text) {
            if claim.reaches.is_none() {
                unsaid.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    claim.line,
                    claim.sentence
                ));
            }
        }
    }

    assert!(
        unsaid.is_empty(),
        "these call something the answer a new installation starts with and do \
         not say what that answer is, so nothing can tell whether they are \
         still true:\n  {}",
        unsaid.join("\n  ")
    );
}

/// One claim read out of a piece of writing, for the test below.
fn the_claim_in(path: &str, writing: &str) -> Option<Claim> {
    let mut claims = claims_about_a_new_installation(Path::new(path), writing);
    assert!(
        claims.len() <= 1,
        "{} claims read out of one piece of writing",
        claims.len()
    );
    claims.pop()
}

#[test]
fn test_the_new_installation_check_can_tell_the_two_apart() {
    // Proving the measurement, in both directions, which is the half the check
    // this replaces was never given. It was taken red once against a sentence
    // it already knew and never against one it did not, and five wordings
    // walked past it afterwards.
    //
    // Every sentence below is copied out of the tree rather than invented:
    // the nine false copies as they stood, and the corrections that replaced
    // them. Written out here because this file is left out of the walk, for
    // the reason on `ours_apart_from_this_file`.
    let false_when_it_was_written = [
        (
            "src/application/allowed.rs",
            "//! removing one from a server, or deleting a task at a provider can, and none\n\
             //! of those paths has run for real yet. So they are switched off by default\n\
             //! and turned on deliberately.\n",
        ),
        (
            "src/application/calendar.rs",
            "/// somebody typed are gone with nothing said. Allow Changes off is the shipped\n\
             /// default and refuses every push, so without this an edit to a Google or\n\
             /// Outlook event could not survive a single sync.\n",
        ),
        (
            "src/application/calendar.rs",
            "        // Allow Changes off is the shipped default, so the push is refused and\n\
             \x20       // the change keeps waiting.\n",
        ),
        (
            "src/application/caldav_sync.rs",
            "        // Allow Changes off is the shipped default, so this is what the sync\n\
             \x20       // does on a computer nobody has configured.\n",
        ),
        (
            "src/application/contacts_sync.rs",
            "//! it is not an unusual case: Allow Changes off is the shipped default for\n\
             //! anybody who turns it off, and every sync in between would otherwise\n\
             //! resurrect her.\n",
        ),
        (
            "src/application/contacts_sync.rs",
            "/// what somebody sees when they delete a contact and it comes back, and Allow\n\
             /// Changes being off is the shipped default, so it is the ordinary case rather\n\
             /// than the unusual one.\n",
        ),
        (
            "docs/changelog.md",
            "  line called it an ordinary update. With Allow Changes turned off, which is\n\
             \x20 what a new account starts with, this happened to every contact you edited.\n",
        ),
        (
            "docs/changelog.md",
            "  whenever Allow Changes was off, which is how the program is shipped: the\n\
             \x20 summary said the change was waiting for you to turn the setting on.\n",
        ),
        // Wordings nobody has written yet, which is the half a list of the
        // wordings already found can never cover.
        (
            "src/application/calendar.rs",
            "// Out of the box, Allow Changes lets nothing reach a provider.\n",
        ),
        (
            "src/application/calendar.rs",
            "// When you install it, Allow Changes refuses every change to a contact.\n",
        ),
        (
            "docs/ALPHA_TESTING.md",
            "A new installation sends none of your calendar to anybody, because Allow\nChanges starts out with both halves off.\n",
        ),
    ];
    for (path, writing) in false_when_it_was_written {
        let claim = the_claim_in(path, writing)
            .unwrap_or_else(|| panic!("no claim was read at all out of:\n{writing}"));
        assert_eq!(
            agrees_with_the_code(&claim),
            Some(false),
            "read as {:?} about {:?}: {}",
            claim.reaches,
            claim.answer,
            claim.sentence
        );
    }

    // And the corrections that replaced them, which have to leave it quiet or
    // the check cries wolf and somebody turns it off.
    let true_and_left_alone = [
        (
            "src/application/allowed.rs",
            "//! of those paths has run for real yet. So they are two answers rather than\n\
             //! one, and a new installation allows one of them: tasks, contacts and the\n\
             //! calendar go up to a provider, and mail does not.\n",
        ),
        (
            "src/application/calendar.rs",
            "        // A new installation allows changes to the calendar, so Allow Changes\n\
             \x20       // is off here because somebody turned it off.\n",
        ),
        (
            "src/application/contacts_sync.rs",
            "        // A new installation allows changes to contacts, so Allow Changes is\n\
             \x20       // off here because somebody turned it off.\n",
        ),
        // The mail half really is refused, so the same shape of sentence about
        // mail is true. Read as one answer rather than two, this would be
        // called a lie.
        (
            "docs/ALPHA_TESTING.md",
            "Allow Changes leaves sending mail off in a new installation, and a message\nthat has gone cannot be recalled.\n",
        ),
    ];
    for (path, writing) in true_and_left_alone {
        let claim = the_claim_in(path, writing)
            .unwrap_or_else(|| panic!("no claim was read at all out of:\n{writing}"));
        assert_eq!(
            agrees_with_the_code(&claim),
            Some(true),
            "read as {:?} about {:?}: {}",
            claim.reaches,
            claim.answer,
            claim.sentence
        );
    }

    // A sentence naming the setting after the word being read. The name is
    // blanked rather than cut out for this: cut out, everything after it moved
    // and the reading landed on a word three back, which here is the other
    // answer.
    let named_afterwards = the_claim_in(
        "docs/ALPHA_TESTING.md",
        "Nothing goes out unless a new installation turns Allow Changes on.\n",
    )
    .expect("a claim with the setting named after the word being read");
    assert_eq!(named_afterwards.reaches, Some(Reaches::Something));
    assert_eq!(agrees_with_the_code(&named_afterwards), Some(true));

    // Both halves of the same sentence, which is what makes the reading a
    // reading rather than a search for the word "off". One is true and one is
    // not, and they differ only in which answer they name.
    let about_mail = the_claim_in(
        "docs/ALPHA_TESTING.md",
        "Under Allow Changes, a new installation sends no message anywhere.\n",
    )
    .expect("a claim about mail");
    assert_eq!(about_mail.answer, Answer::Mail);
    assert_eq!(agrees_with_the_code(&about_mail), Some(true));

    let about_the_rest = the_claim_in(
        "docs/ALPHA_TESTING.md",
        "Under Allow Changes, a new installation sends no contact anywhere.\n",
    )
    .expect("a claim about the other answer");
    assert_eq!(about_the_rest.answer, Answer::PersonalInformation);
    assert_eq!(agrees_with_the_code(&about_the_rest), Some(false));

    // Nothing to read, which is the shape the second check is about and the
    // one place this reading gives up.
    let says_nothing = the_claim_in(
        "src/application/contacts_sync.rs",
        "        // Allow Changes is at the shipped default here, so this is the ordinary\n\
         \x20       // case rather than the unusual one.\n",
    )
    .expect("a claim naming the answer without saying what it is");
    assert_eq!(agrees_with_the_code(&says_nothing), None);

    // Out of scope, and every one of these was a false alarm before the scope
    // was drawn where it is. None of them is about this setting.
    for (path, writing) in [
        (
            "src/application/tasks_sync.rs",
            "/// a task made here is filed in the account's first list, which is the\n\
             /// provider's own default list.\n",
        ),
        (
            "src/presentation/managers.rs",
            "// Both providers return their default list first.\n",
        ),
        (
            "docs/PROVIDER_SETUP.md",
            "This is the default for Gmail and for any provider we do not recognise.\n",
        ),
        // The type's own default really is nothing, deliberately, and its
        // tests say so. That is a different default in the one file where
        // both live, which is why only the module documentation is read.
        (
            "src/application/allowed.rs",
            "        // The important default. A value built without anybody deciding\n\
             \x20       // should be the one that cannot damage somebody's mail.\n",
        ),
        // A table is a list, and the one in the testing page is right.
        (
            "docs/ALPHA_TESTING.md",
            "Allow Changes covers two things.\n\n| | What it covers | Default |\n|---|---|---|\n| Mail | Sending | **Off** |\n",
        ),
        // And a fenced example in a document is not a claim the document makes.
        (
            "docs/ALPHA_TESTING.md",
            "Allow Changes covers two things.\n\n```\nAllow Changes is off by default.\n```\n",
        ),
    ] {
        assert!(
            the_claim_in(path, writing).is_none(),
            "a false alarm, on prose that is not about this setting:\n{writing}"
        );
    }

    // And the walk has to be reading the tree, or all of the above passes over
    // an empty list.
    let walked = ours_apart_from_this_file();
    assert!(
        walked.len() > 100,
        "only {} files checked, so the walk is broken",
        walked.len()
    );
    assert!(
        walked.iter().any(|f| f.ends_with("allowed.rs")),
        "the module that defines the setting is not being read"
    );
    assert!(
        walked.iter().any(|f| f.ends_with("changelog.md")),
        "the changelog is not being read, and four of the nine copies were in it"
    );
    assert!(
        !walked.iter().any(|f| f.ends_with("house_style.rs")),
        "this file is in the walk, so the sentences above would fail it"
    );
}

/// Documents somebody reads, as opposed to source and configuration.
fn documents() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("docs"), &["md"], &mut found);
    found.push(PathBuf::from("README.md"));
    found
}

/// Every `[label](target)` in a document.
fn links_in(text: &str) -> Vec<(usize, String, String)> {
    let mut links = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let characters: Vec<char> = line.chars().collect();
        let mut at = 0;
        while at < characters.len() {
            if characters[at] != '[' {
                at += 1;
                continue;
            }
            let Some(label_end) = characters[at..]
                .iter()
                .position(|c| *c == ']')
                .map(|o| at + o)
            else {
                break;
            };
            if characters.get(label_end + 1) != Some(&'(') {
                at += 1;
                continue;
            }
            let Some(target_end) = characters[label_end..]
                .iter()
                .position(|c| *c == ')')
                .map(|o| label_end + o)
            else {
                break;
            };
            links.push((
                number + 1,
                characters[at + 1..label_end].iter().collect(),
                characters[label_end + 2..target_end].iter().collect(),
            ));
            at = target_end + 1;
        }
    }
    links
}

/// Whether a piece of link text is really a file name.
fn is_a_file_name(label: &str) -> bool {
    let bare = label.trim().trim_matches('`');
    let Some((_, extension)) = bare.rsplit_once('.') else {
        return false;
    };
    !bare.contains(' ')
        && matches!(
            extension.to_ascii_lowercase().as_str(),
            "md" | "html" | "exe" | "iss" | "toml" | "json" | "rs" | "py" | "sh" | "yml"
        )
}

#[test]
fn test_no_link_is_labelled_with_a_file_name() {
    // A link reading "installing.md" tells somebody nothing about where it
    // goes, and a screen reader user pulling up a list of links on the page
    // gets a list of file names. The label should say what is on the other
    // side. WCAG 2.2, 2.4.4 Link Purpose.
    //
    // The address is still the file, which is correct: that is a machine
    // name doing a machine's job, and nobody reads it out.
    let mut named_after_a_file = Vec::new();

    for document in documents() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        for (line, label, target) in links_in(&text) {
            if is_a_file_name(&label) {
                named_after_a_file.push(format!(
                    "{}:{line}: [{label}]({target}) says nothing about where it goes",
                    document.display()
                ));
            }
        }
    }

    assert!(
        named_after_a_file.is_empty(),
        "{}",
        named_after_a_file.join("\n  ")
    );
}

#[test]
fn test_the_documents_call_it_by_its_name() {
    // `wixen-mail` is the crate, the executable, the data folder and the log
    // prefix, and it is right in all of them. It is a machine name, so in a
    // sentence it should be "Wixen Mail". Code spans and link addresses are
    // where the machine name belongs and are left alone.
    let mut machine_named = Vec::new();

    for document in documents() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        let mut inside_a_fence = false;
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                inside_a_fence = !inside_a_fence;
                continue;
            }
            // An indented block is code as well, and the commands in the
            // testing page are written that way.
            if inside_a_fence || line.starts_with("    ") || line.starts_with('\t') {
                continue;
            }
            let prose = without_code_and_addresses(line);
            let lowered = prose.to_lowercase();
            if lowered.contains("wixen-mail") || lowered.contains("wixen_mail") {
                machine_named.push(format!(
                    "{}:{}: {}",
                    document.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        machine_named.is_empty(),
        "the machine name is being read out to people:\n  {}",
        machine_named.join("\n  ")
    );
}

/// Where a name a person is told to look for must not be typed a second time.
///
/// The screen and the sentence that sends somebody to it. Source rather than
/// behaviour, and the reason is that the settings panel is built by wxWidgets
/// calls with no seam short of a running window. What it can say is that the
/// name is not written out here, which is the whole of how the two drifted.
const WHERE_THE_SECTION_IS_LABELLED: &str = "src/presentation/wx_settings.rs";

#[test]
fn test_the_settings_screen_does_not_write_the_section_name_out_itself() {
    // A sync says "turn on Allow Changes in Settings". The section was
    // headed "Allowed Changes", because the sentence and the label were two
    // strings typed in two places. Near enough to look like the right place,
    // far enough that somebody stops and checks whether they have found it.
    //
    // Both come from `application::allowed::SETTINGS_SECTION` now, so the name
    // can be changed in one place and both follow. This is what stops the
    // label being typed back in.
    let screen = fs::read_to_string(WHERE_THE_SECTION_IS_LABELLED)
        .expect("the settings screen to be readable");
    // The name as it stands, not the name it happens to have today. Written
    // out as a literal here, this check would go on forbidding "Allow Changes"
    // after somebody renamed the section to something else and typed the new
    // name in, which is the same fault one step along.
    let written_out = format!("\"{}\"", wixen_mail::application::allowed::SETTINGS_SECTION);

    let typed: Vec<String> = screen
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(&written_out))
        .map(|(number, line)| {
            format!(
                "{WHERE_THE_SECTION_IS_LABELLED}:{}: {}",
                number + 1,
                line.trim()
            )
        })
        .collect();

    assert!(
        typed.is_empty(),
        "the name of this section is said by a sync as the place to go, so it \
         is held in one constant. Written out here it can drift from the \
         sentence again:\n  {}",
        typed.join("\n  ")
    );
}

/// Documents describing what the software is now, as opposed to what it was.
///
/// `docs/changelog.md` is left out on purpose. It is a dated record, and one of
/// its own later entries is the correction saying the cache was never
/// encrypted. Rewriting the entry that made the claim would hide that the claim
/// was ever made, which is the opposite of what a changelog is for.
fn documents_about_now() -> Vec<PathBuf> {
    documents()
        .into_iter()
        .filter(|d| !d.ends_with("changelog.md"))
        .collect()
}

/// Whether a sentence says something is encrypted rather than that it is not.
fn claims_encryption(prose: &str) -> bool {
    let lowered = prose.to_lowercase();
    if !lowered.contains("encrypt") {
        return false;
    }
    // Only the local store. A connection really is encrypted, a disk can be,
    // and credentials really do go somewhere Windows protects.
    let about_the_store = ["cache", "cached", "database", "sqlite"]
        .iter()
        .any(|word| lowered.contains(word));
    let denied = ["not encrypt", "never encrypt", "no encrypt", "unencrypt"]
        .iter()
        .any(|phrase| lowered.contains(phrase));

    about_the_store && !denied
}

#[test]
fn test_no_document_says_the_cache_is_encrypted() {
    // It is not, and saying it is tells somebody their mail is protected on a
    // disk where it is sitting in the clear. That is worse than saying nothing:
    // it is the claim a person would decide against turning on BitLocker.
    //
    // This has now been corrected twice. The first time it was found by reading
    // and fixed by hand, which is how it came back: one document was missed and
    // then contradicted itself two sentences later. So it is a failing build
    // rather than something somebody has to keep noticing.
    let mut claiming = Vec::new();

    for document in documents_about_now() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        let mut inside_a_fence = false;
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                inside_a_fence = !inside_a_fence;
                continue;
            }
            if inside_a_fence {
                continue;
            }
            if claims_encryption(&without_code_and_addresses(line)) {
                claiming.push(format!(
                    "{}:{}: {}",
                    document.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        claiming.is_empty(),
        "the cached mail is not encrypted, and these say it is:\n  {}",
        claiming.join("\n  ")
    );
}

#[test]
fn test_the_encryption_check_can_tell_the_two_apart() {
    // Without this the check above could pass by seeing nothing at all, which
    // is exactly how the claim survived a pass that reported the docs clean.
    assert!(claims_encryption(
        "An encrypted SQLite cache holds messages"
    ));
    assert!(claims_encryption("the database is encrypted at rest"));

    assert!(!claims_encryption("the cached mail is not encrypted"));
    assert!(!claims_encryption("the cache is unencrypted"));
    assert!(!claims_encryption("unless the disk itself is encrypted"));
    assert!(!claims_encryption("connections use TLS encryption"));
    assert!(!claims_encryption("passwords go to the credential store"));

    assert!(
        documents_about_now().len() > 5,
        "only {} documents checked, so the walk is broken",
        documents_about_now().len()
    );
}

/// A line with its code spans and link addresses taken out.
fn without_code_and_addresses(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_code = false;
    let mut in_address = false;
    let mut previous = '\0';

    for character in line.chars() {
        match character {
            '`' => in_code = !in_code,
            '(' if previous == ']' => in_address = true,
            ')' if in_address => in_address = false,
            _ if !in_code && !in_address => out.push(character),
            _ => {}
        }
        previous = character;
    }
    out
}

#[test]
fn test_the_check_is_looking_at_the_whole_project() {
    // Without this, a mistake in `ours` that returned nothing would leave the
    // test above passing on an empty list and reporting the house style kept.
    let files = ours();

    assert!(
        files.len() > 100,
        "only {} files checked, so the walk is broken",
        files.len()
    );
    assert!(
        files.iter().any(|f| f.ends_with("main.rs")),
        "the source is not being checked"
    );
    assert!(
        files.iter().any(|f| f.ends_with("ALPHA_TESTING.md")),
        "the documents are not being checked"
    );
    assert!(
        files.iter().any(|f| f.ends_with("guards.py")),
        "the scripts that are not shell are not being checked, and four of \
         them are Python"
    );
    assert!(
        files.iter().any(|f| f.ends_with("guards.toml")),
        "the guard record carries prose and is not being checked"
    );
}

/// Whether a line hands itself a temporary folder by building the path.
///
/// `temp_dir()` on its own is fine and three places in the running program use
/// it correctly. What is not fine is joining a name onto it, because that names
/// a folder nothing owns: it gets created, it gets a database opened inside it,
/// and it is still there when the test ends.
fn builds_its_own_temp_folder(line: &str) -> bool {
    line.split_once("temp_dir()")
        .is_some_and(|(_, after)| after.contains(".join("))
}

/// The files whose test halves this checks.
///
/// `tests/house_style.rs` is left out of its own walk. The check below is only
/// worth having if something proves it can see, and the only way to prove that
/// is to write the offending line out in full, in this file. The em-dash rule
/// two hundred lines up hit the same wall and solved it by building the
/// characters from code points. That does not work here: the thing being
/// searched for is the literal text of the examples, so there is nothing to
/// disguise. Excluding the file is the honest version. Anything that walks past
/// this comment and "fixes" a failure by loosening `builds_its_own_temp_folder`
/// has turned a check into decoration.
fn test_sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &["rs"], &mut found);
    collect(Path::new("tests"), &["rs"], &mut found);
    found.retain(|path| !path.ends_with("house_style.rs"));
    found
}

/// The line numbers in one file where a test builds its own temporary folder.
///
/// Only the test half of a source file is read. The split is on the first
/// `#[cfg(test)]`, the same way `calendar.rs` tells the running program apart
/// from its tests. Everything under `tests/` is test code all the way down.
fn hand_built_temp_folders_in(path: &Path, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut reading_test_code = path.starts_with("tests");

    for (number, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            reading_test_code = true;
        }
        if reading_test_code && builds_its_own_temp_folder(line) {
            found.push(format!(
                "{}:{}: {}",
                path.display(),
                number + 1,
                line.trim()
            ));
        }
    }
    found
}

#[test]
fn test_no_test_builds_its_own_temp_folder() {
    // A test that joins a name onto the system temporary folder has to remove
    // that folder itself, and none of them did. Seventy-six sites left 390
    // folders behind on every `cargo test --lib`, the commit hook runs the
    // suite on every commit, and `scripts/mutants.sh` runs it once per mutant.
    // The disk filled.
    //
    // `tempfile::tempdir()` removes the folder when the value is dropped, so
    // the folder belongs to something. Bind it before whatever opens a database
    // inside it, or put both in one guard with the folder declared second, and
    // drop order does the rest.
    let mut hand_built = Vec::new();

    for path in test_sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        hand_built.extend(hand_built_temp_folders_in(&path, &text));
    }

    assert!(
        hand_built.is_empty(),
        "{} tests build a temporary folder nothing removes:\n  {}",
        hand_built.len(),
        hand_built.join("\n  ")
    );
}

#[test]
fn test_the_temp_folder_check_can_tell_the_two_apart() {
    // Without this the check above passes by seeing nothing, which is how a
    // scan reporting success while scanning nothing gets believed. Every line
    // here is copied from the tree rather than invented, so it cannot drift
    // into testing a shape the project does not have.
    assert!(builds_its_own_temp_folder(
        r#"let dir = env::temp_dir().join(format!("wixen_mail_deletions_before_{nanos}"));"#
    ));
    assert!(builds_its_own_temp_folder(
        r#"std::env::temp_dir().join(format!("wixen_caldav_{label}_{nanos}"))"#
    ));

    // The assertion in `help_page.rs` that the running program puts its help
    // under the temporary folder. It names no folder of its own, and a check
    // matching a bare `temp_dir()` would call it a leak.
    assert!(!builds_its_own_temp_folder(
        r#"assert!(into.starts_with(std::env::temp_dir()), "{}", into.display());"#
    ));
    // What all of them were changed to.
    assert!(!builds_its_own_temp_folder(
        "let dir = tempfile::tempdir().expect(\"temp dir\");"
    ));

    // `logging.rs` really does join a name on, and is right to: it is the
    // running program's own log folder, not a test's. Nothing but the split at
    // the first `#[cfg(test)]` keeps it out, so that split is tested directly.
    let production_line =
        r#"        .unwrap_or_else(|_| std::env::temp_dir().join("wixen-mail").join("logs"))"#;
    assert!(builds_its_own_temp_folder(production_line));
    let logging =
        format!("fn where_logs_go() {{\n{production_line}\n}}\n#[cfg(test)]\nmod tests {{}}\n");
    assert!(
        hand_built_temp_folders_in(Path::new("src/common/logging.rs"), &logging).is_empty(),
        "the running program's half of a source file is being read as test code"
    );

    // And the same line below a `#[cfg(test)]` is a leak, so the split is not
    // simply switched off.
    let in_a_test = format!("#[cfg(test)]\nmod tests {{\n{production_line}\n}}\n");
    assert_eq!(
        hand_built_temp_folders_in(Path::new("src/common/logging.rs"), &in_a_test).len(),
        1,
        "the test half of a source file is not being read"
    );

    assert!(
        test_sources().len() > 100,
        "only {} files checked, so the walk is broken",
        test_sources().len()
    );
    assert!(
        test_sources().iter().any(|f| f.ends_with("bodies.rs")),
        "the module this rule was written for is not being checked"
    );
}

// ── A count beside a list ───────────────────────────────────────────────────
//
// A number in the sentence that introduces a list, and a claim in prose that a
// list is everything, are both a second statement of a fact the list already
// makes. The list then changes and the sentence does not. In `docs/changelog.md`
// that has now happened seven times, and three of them were standing at once:
// one entry said "Two things still do not survive, and this is the whole list"
// and then, further down the same entry, "One label shape is still lost, and it
// is the only one"; another said "Six limitations" over a list of five; a third
// said "These three were true when they were written and are not true now" over
// four notes, one of which said "Half closed".
//
// Each was found by reading, corrected by hand, and came back. So it is a
// failing build instead.
//
// Only the changelog. The same shape appears in `docs/plans` where it is
// correct: a list of three whose bullets add up to twelve, and a paragraph
// naming the four questions in front of a list of three failure modes. A check
// that fired on those would need exceptions, and a style check with exceptions
// is a check nobody reads.
//
// What it still cannot see, so that nobody reads a clean run as more than it
// is. A count in the middle of a paragraph that does not end with a colon: the
// paragraph has to announce the list for a number in it to be about the list,
// and without that rule "stored in three places" reads as a count of three.
// A list of more than twenty announced by its size. A count written inside a
// code span, which is stripped before any of this. And a list nothing
// introduces, which is most of them: forty-three lists in the changelog, five
// of them with a paragraph in front.

/// Numbers as prose writes them, which is how a count beside a list is written.
const COUNTED_IN_WORDS: [(&str, usize); 12] = [
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
];

/// A list longer than this is not something a paragraph here counts out loud.
///
/// The bound is what keeps a year and a page number from being read as a count
/// of bullets. It costs the check a list of more than twenty introduced by its
/// size, and this file has never had one.
const A_LIST_LONGER_THAN_ANYBODY_ANNOUNCES: usize = 20;

/// The number a paragraph states about the list under it, if it states one.
///
/// Two shapes, and deliberately only two. At the very start, "Two things still
/// do not survive". Or anywhere in the sentence that ends with the colon
/// announcing the list, "Some prose about it, and three things are worth
/// knowing before you try it:", which is the commonest shape in the changelog
/// and was invisible while this read three words.
///
/// A number anywhere else is about something else. "The rule it repeats by is
/// stored in three places and read once:" counts places, not bullets, and
/// reading that as the count is how a check starts crying wolf.
/// [`A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT`] is what tells the two
/// apart.
fn the_count_it_states(paragraph: &str) -> Option<usize> {
    let prose = without_code_and_addresses(paragraph);
    if let Some(count) = the_first_number_among(prose.split_whitespace().take(3)) {
        return Some(count);
    }
    if !prose.trim_end().ends_with(':') {
        return None;
    }
    the_first_number_among(the_sentence_that_announces(&prose).split_whitespace())
}

/// The last sentence of a paragraph, which is the one carrying the colon.
fn the_sentence_that_announces(paragraph: &str) -> &str {
    match paragraph.rfind(['.', '!', '?']) {
        Some(at) => &paragraph[at + 1..],
        None => paragraph,
    }
}

/// The words that hand the number after them to something other than the list.
///
/// "stored in three places" counts places. "and three things are worth knowing"
/// counts the bullets underneath. What separates them is the word in front: a
/// number opened by a preposition belongs to the phrase that preposition
/// opened, and a sentence can carry one of those and still announce a list.
///
/// Reading past one of these can only make the check quieter, never louder. It
/// gives up a list whose count really has drifted rather than name one that has
/// not, which is the trade the bound above is drawn for as well.
const A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT: [&str; 12] = [
    "in", "of", "at", "on", "from", "by", "to", "for", "with", "across", "over", "into",
];

/// The first of these words that is a number the sentence states about the
/// list, if any of them is.
fn the_first_number_among<'a>(words: impl Iterator<Item = &'a str>) -> Option<usize> {
    let mut word_before = "";
    for word in words {
        match the_number_a_word_is(word) {
            Some(count) if !hands_the_number_on(word_before) => return Some(count),
            _ => word_before = word,
        }
    }
    None
}

/// Whether this word puts the number after it on to something else.
fn hands_the_number_on(word: &str) -> bool {
    let bare = word
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_lowercase();
    A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT.contains(&bare.as_str())
}

/// The number a word is, spelled or in digits, if it is one.
///
/// Only the edges are trimmed and never the middle. "0.5.0" is a version and
/// "2026-07-20" is a date, and taking the punctuation out of either leaves a
/// number that was never written.
fn the_number_a_word_is(word: &str) -> Option<usize> {
    let bare = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    let lowered = bare.to_lowercase();
    if let Some((_, count)) = COUNTED_IN_WORDS
        .iter()
        .find(|(spelled, _)| *spelled == lowered)
    {
        return Some(*count);
    }
    bare.parse()
        .ok()
        .filter(|count| (1..=A_LIST_LONGER_THAN_ANYBODY_ANNOUNCES).contains(count))
}

/// Whether a line is a bullet at exactly this indentation.
fn a_bullet_at(line: &str, indent: usize) -> bool {
    line.strip_prefix(" ".repeat(indent).as_str())
        .is_some_and(|rest| rest.starts_with("- "))
}

/// Whether a line belongs to the bullet above it rather than ending the list.
fn inside_the_list(line: &str, indent: usize) -> bool {
    line.trim().is_empty() || line.starts_with(" ".repeat(indent + 2).as_str())
}

/// Every bullet list in a document, with the paragraph that introduces it.
///
/// Gives the line the paragraph starts on, the paragraph itself, and how many
/// bullets the list holds. A list with a heading or another bullet directly
/// above it introduces itself and is left out.
fn lists_with_their_introductions(text: &str) -> Vec<(usize, String, usize)> {
    // The changelog indents a nested list by two and a bullet's own
    // continuation lines by two more, so those are the only two levels. Each
    // level is read on its own pass: read together, a nested list is swallowed
    // as part of the bullet it sits under and never counted.
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for indent in [0, 2] {
        found.extend(lists_at(&lines, indent));
    }
    found
}

/// Every bullet list at one indentation, with the paragraph introducing it.
fn lists_at(lines: &[&str], indent: usize) -> Vec<(usize, String, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if !a_bullet_at(lines[at], indent) {
            at += 1;
            continue;
        }
        let mut bullets = 0;
        let mut end = at;
        while end < lines.len() {
            if a_bullet_at(lines[end], indent) {
                bullets += 1;
            } else if !inside_the_list(lines[end], indent) {
                break;
            }
            end += 1;
        }

        let mut last = at;
        while last > 0 && lines[last - 1].trim().is_empty() {
            last -= 1;
        }
        let introduced = last > 0
            && !lines[last - 1].trim_start().starts_with("- ")
            && !lines[last - 1].starts_with('#');
        if introduced {
            let mut first = last - 1;
            while first > 0
                && !lines[first - 1].trim().is_empty()
                && !lines[first - 1].trim_start().starts_with("- ")
            {
                first -= 1;
            }
            let paragraph = lines[first..last]
                .iter()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join(" ");
            found.push((first + 1, paragraph, bullets));
        }
        at = end;
    }
    found
}

/// Prose that says a list is everything there is.
const A_CLAIM_THE_LIST_ALREADY_MAKES: [&str; 6] = [
    "this is the whole list",
    "these are the whole list",
    "it is the only one",
    "these are all of them",
    "that is all of them",
    "this is the complete list",
];

#[test]
fn test_no_changelog_list_is_introduced_by_a_count_that_disagrees_with_it() {
    let changelog = fs::read_to_string("docs/changelog.md").expect("the changelog to be readable");
    let lists = lists_with_their_introductions(&changelog);

    let disagreeing: Vec<String> = lists
        .iter()
        .filter_map(|(line, paragraph, bullets)| {
            let count = the_count_it_states(paragraph)?;
            (count != *bullets).then(|| {
                let opening: String = paragraph.chars().take(70).collect();
                format!("docs/changelog.md:{line}: says {count}, list has {bullets}: {opening}")
            })
        })
        .collect();

    assert!(
        disagreeing.is_empty(),
        "a count beside a list states a fact the list already states, and these \
         two have drifted apart. Take the count out rather than correcting \
         it:\n  {}",
        disagreeing.join("\n  ")
    );

    // The check has to find a list at all, or it passes by reading nothing.
    // Most lists here sit under a heading or another bullet and so introduce
    // themselves; the ones with a paragraph in front are the ones at issue.
    // `test_the_count_check_can_see_a_count_that_disagrees` is what says the
    // reading works, at both levels of indentation.
    assert!(
        !lists.is_empty(),
        "no list with a paragraph in front of it was found, so the reading is broken"
    );
}

#[test]
fn test_the_changelog_does_not_claim_in_prose_that_a_list_is_everything() {
    let changelog = fs::read_to_string("docs/changelog.md").expect("the changelog to be readable");

    let claiming: Vec<String> = changelog
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let lowered = line.to_lowercase();
            A_CLAIM_THE_LIST_ALREADY_MAKES
                .iter()
                .any(|claim| lowered.contains(claim))
        })
        .map(|(number, line)| format!("docs/changelog.md:{}: {}", number + 1, line.trim()))
        .collect();

    assert!(
        claiming.is_empty(),
        "a list is already the whole of what it holds, so saying so in prose \
         beside it is a second statement that goes stale when the list \
         changes:\n  {}",
        claiming.join("\n  ")
    );
}

#[test]
fn test_the_count_check_can_see_a_count_that_disagrees() {
    // Proving the measurement. A check that reads nothing passes, and from
    // outside that looks exactly like a check that reads everything and finds
    // nothing wrong. These are the shapes it was written for, taken from the
    // changelog as it stood.
    let drifted = "These three were true when they were written and are not true now.\n\
                   \n\
                   - Receiving mail is not implemented.\n\
                   - Sending does not support OAuth accounts.\n\
                   - Threaded view appears in the View menu.\n\
                   - Five accessibility scan findings remain.\n";
    let lists = lists_with_their_introductions(drifted);
    assert_eq!(lists.len(), 1, "the list was not read at all");
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 4, "the bullets were not counted");

    // A count that agrees is not complained about.
    let agreeing = "Two things still do not survive:\n\n- The first.\n- The second.\n";
    let lists = lists_with_their_introductions(agreeing);
    assert_eq!(the_count_it_states(&lists[0].1), Some(2));
    assert_eq!(lists[0].2, 2);

    // A nested list, which is where two of the stale counts were. Read on one
    // pass with the level above it, a nested list is swallowed as part of the
    // bullet it sits under and is never counted at all.
    let nested = concat!(
        "- **An entry with a list inside it.** Some prose about it.\n",
        "\n",
        "  Two things still do not survive:\n",
        "\n",
        "  - The first.\n",
        "  - The second.\n",
        "  - The third.\n"
    );
    let inside = lists_with_their_introductions(nested);
    let counted = inside
        .iter()
        .find(|(_, paragraph, _)| paragraph.starts_with("Two things"))
        .expect("the nested list was not read");
    assert_eq!(the_count_it_states(&counted.1), Some(2));
    assert_eq!(counted.2, 3, "the nested bullets were not counted");

    // A count later in the paragraph, which is the commonest shape in the
    // changelog: prose first, then a sentence ending in a colon that announces
    // the list. Read as three words this was invisible.
    let announced = concat!(
        "Some prose about it, and three things are worth knowing before you try it:\n",
        "\n",
        "- The first.\n",
        "- The second.\n"
    );
    let lists = lists_with_their_introductions(announced);
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 2, "the bullets were not counted");

    // And written in digits. Prose spells the small numbers and this file
    // mostly does, but nothing stops somebody writing the digit.
    let in_digits = "There are 3 things here:\n\n- The first.\n- The second.\n";
    let lists = lists_with_their_introductions(in_digits);
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 2, "the bullets were not counted");

    // A number in the middle of a sentence is about something else. The colon
    // belongs on it: without one the reading stops before the sentence that
    // announces the list is looked at, so this proved nothing and the check
    // really did read this as a count of bullets and cry wolf about it.
    assert_eq!(
        the_count_it_states("The rule it repeats by is stored in three places and read once:"),
        None
    );

    // The same sentence as a paragraph reads for a bullet's own continuation
    // lines, which is where it starts once the bullet above it is stripped off.
    // This is the shape the check was measured against in the real file.
    assert_eq!(
        the_count_it_states("in three places and read once:"),
        None,
        "a count of places was read as a count of bullets"
    );

    // And a sentence that carries both. Reading past the first number must not
    // give up on the one that really does announce the list.
    assert_eq!(
        the_count_it_states("Stored in three places, and two things follow from that:"),
        Some(2)
    );

    // A version and a date are numbers in an announcing sentence and neither
    // is a count of anything. Both are ordinary in this file.
    assert_eq!(
        the_count_it_states("Everything that changed in 0.5.0:"),
        None
    );
    assert_eq!(
        the_count_it_states("What was still true on 2026-07-20:"),
        None
    );

    // And the claim check has something to match on.
    assert!(
        A_CLAIM_THE_LIST_ALREADY_MAKES
            .iter()
            .any(|claim| "and this is the whole list:".contains(claim))
    );
}
// ── Where the first-run screen starts ───────────────────────────────────────
//
// Two facts, and the prose about this screen has been welding them into one
// sentence since the screen was written. What is *selected*, and so what
// Continue takes from somebody who presses Enter without reading, is
// `Choice::DEFAULT`, which is the second of the three and allows changes to
// tasks, contacts and the calendar. Where focus lands is the window layer's
// doing, and it used to be the first, which is the safest.
//
// "The default is the first thing focus lands on" is those two welded together
// and, while they differed, true of neither. They do not differ now: the window
// focuses whichever button it finds ticked, because somebody using a screen
// reader hears the answer that is read out and presses Enter on the first thing
// they hear. Read out the cautious answer and tick the permissive one, and that
// person has switched on writing to their real address book without being told.
//
// Each sentence is still read against its own answer in the code, because they
// are two facts even while they agree: a sentence about what is selected
// against where `DEFAULT` sits in `ALL`, and a sentence about focus against the
// button the window layer focuses.

/// The screens whose prose says where this one starts.
const WHERE_THE_FIRST_RUN_SCREEN_IS_DESCRIBED: [&str; 2] = [
    "src/presentation/first_run.rs",
    "src/presentation/wx_first_run.rs",
];

/// The three choices as prose names them, in the order they are offered.
const HOW_PROSE_NAMES_THE_CHOICES: [&[&str]; 3] = [
    &["first", "safest", "safe", "cautious"],
    &["second", "middle"],
    &["third", "last", "riskiest"],
];

/// Words saying a sentence is about which choice is selected.
const ABOUT_THE_ANSWER_IT_STARTS_WITH: &[&str] = &[
    "the default",
    "starts on",
    "selected",
    "preselect",
    "presses enter",
    "pressing enter",
];

/// Which of the three a sentence names, if it names one.
///
/// The first naming word wins. A sentence naming two of them is describing one
/// in terms of another, and the one it opens with is the one it is about.
fn which_choice_it_names(sentence: &str) -> Option<usize> {
    words_of(&sentence.to_lowercase())
        .into_iter()
        .find_map(|(_, word)| {
            HOW_PROSE_NAMES_THE_CHOICES
                .iter()
                .position(|names| names.contains(&word.as_str()))
        })
}

/// Every sentence of a run of prose, with where it starts.
fn sentences_of(prose: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < prose.len() {
        let (start, end) = the_sentence_around(prose, at);
        if !prose[start..end].trim().is_empty() {
            found.push((start, prose[start..end].trim()));
        }
        at = end + 1;
    }
    found
}

/// Which button the window layer puts focus on.
///
/// Read from the source, because reaching the real answer needs a window, and
/// what is read is a coupling rather than a number: the window puts focus on
/// whichever button it finds ticked, so focus is wherever the tick is and the
/// caller's number is the answer.
///
/// Only enough of that rule to know the two are one answer. The rule itself,
/// with everything that can break it, belongs to the screen and is measured by
/// `first_run::tests::test_the_screen_puts_focus_on_the_answer_it_ticks`.
/// Reading it here at all is so that moving the focus fails this instead of
/// quietly making the prose wrong again.
fn which_button_takes_focus(where_the_tick_is: usize) -> usize {
    let screen = fs::read_to_string("src/presentation/wx_first_run.rs")
        .expect("the first-run window to be readable");
    assert!(
        screen.contains("ticked.set_focus()"),
        "the first-run window no longer puts focus on the answer it ticks, so \
         every sentence about where focus lands needs writing again"
    );
    where_the_tick_is
}

#[test]
fn test_the_first_run_screen_is_not_described_as_starting_where_it_does_not() {
    // Where the screen starts is two values in the code and no sentence
    // written down here, so if either moves this follows it.
    let selected = wixen_mail::presentation::first_run::Choice::ALL
        .iter()
        .position(|choice| *choice == wixen_mail::presentation::first_run::Choice::DEFAULT)
        .expect("the choice the screen starts on to be one of the three offered");
    let focused = which_button_takes_focus(selected);
    let mut wrong = Vec::new();

    for name in WHERE_THE_FIRST_RUN_SCREEN_IS_DESCRIBED {
        let path = PathBuf::from(name);
        let text = fs::read_to_string(&path).expect("the screen to be readable");
        for prose in prose_in(&path, &text) {
            for (at, sentence) in sentences_of(&prose.text) {
                let lowered = sentence.to_lowercase();
                let Some(named) = which_choice_it_names(sentence) else {
                    continue;
                };
                let about_the_answer = ABOUT_THE_ANSWER_IT_STARTS_WITH
                    .iter()
                    .any(|words| lowered.contains(words));
                if about_the_answer && named != selected {
                    wrong.push(format!(
                        "{name}:{}: puts the answer it starts on at {named}, and it is at \
                         {selected}: {sentence}",
                        prose.line_at(at)
                    ));
                }
                if lowered.contains("focus") && named != focused {
                    wrong.push(format!(
                        "{name}:{}: puts focus on {named}, and it lands on {focused}: {sentence}",
                        prose.line_at(at)
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the first-run screen focuses the safest of the three and selects the \
         second, which allows changes to tasks, contacts and the calendar. \
         Somebody who presses Enter without reading gets the second, and these \
         say otherwise:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_first_run_check_can_tell_the_two_apart() {
    // Proving the measurement. Every sentence here is copied from the tree:
    // the five that were wrong, and the ones that replaced them.
    for (sentence, names) in [
        ("The default is the safe one", Some(0)),
        ("so the default is the first thing focus lands on", Some(0)),
        (
            "Somebody who presses Enter without reading gets the cautious answer",
            Some(0),
        ),
        (
            "Continue is always available, though, and starts on the safe answer",
            Some(0),
        ),
        (
            "pressing Enter immediately is a valid decision and a cautious one",
            Some(0),
        ),
        ("The one selected is the second", Some(1)),
        (
            "Somebody who presses Enter without reading gets the middle answer",
            Some(1),
        ),
        ("Focus lands on the safest of them", Some(0)),
        (
            "Focus lands on the second as well, so the answer read out is the answer Continue takes",
            Some(1),
        ),
        (
            "Focus lands on that middle answer too, so the answer read out is the answer Continue \
             takes",
            Some(1),
        ),
        (
            "mail is left alone, and changes go up to their provider",
            None,
        ),
    ] {
        assert_eq!(which_choice_it_names(sentence), names, "{sentence}");
    }

    // A sentence is read on its own, or one carrying both facts is judged by
    // whichever it happens to name first.
    let both = "Focus lands on the safest of them. The one selected is the second.";
    let read: Vec<&str> = sentences_of(both).into_iter().map(|(_, it)| it).collect();
    assert_eq!(
        read,
        vec![
            "Focus lands on the safest of them",
            "The one selected is the second"
        ]
    );

    // The answer this is checked against, which is the whole point: it comes
    // from the code and not from anything written here. Focus and the tick are
    // one answer now, so there is one number where there were two, and reading
    // the window is what says the two really are coupled.
    let selected = wixen_mail::presentation::first_run::Choice::ALL
        .iter()
        .position(|choice| *choice == wixen_mail::presentation::first_run::Choice::DEFAULT);
    assert_eq!(
        selected,
        Some(1),
        "the screen no longer starts on the second"
    );
    assert_eq!(which_button_takes_focus(1), 1);

    // And the files really are there to be read, or the check above passes
    // over nothing.
    for name in WHERE_THE_FIRST_RUN_SCREEN_IS_DESCRIBED {
        let text = fs::read_to_string(name).expect("the screen to be readable");
        assert!(
            !prose_in(&PathBuf::from(name), &text).is_empty(),
            "{name} has no prose in it, so the reading is broken"
        );
    }
}

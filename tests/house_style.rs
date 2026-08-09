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
//
// It reads English with word lists, so it cannot be complete, and the thing it
// must not do is say it is. An earlier version of this comment said a fifth
// copy of the sentence would fail whatever it said. Ten minutes of trying broke
// that: "everything about a person stays on this computer, and no contact is
// ever sent to Google" was quiet against the whole tree, because "on" is a
// preposition far more often than it is a switch position and it sat between
// the installation-time word and the negative that was making the claim.
//
// So, measured rather than hoped for. What it catches, each taken from a
// sentence written to get past it and then written into the test below:
//
//   * the nine wordings already in the tree, and the corrections that
//     replaced them, told apart from each other
//   * a preposition standing between the two: "stays on this computer",
//     "keeps your tasks on your own machine"
//   * a preposition whose object is a place word: "stored on here"
//   * a refusal said with a place or a person rather than with "not":
//     "kept here and sent nowhere"
//   * either half said about the other: the same sentence about mail and
//     about contacts gets two answers, because the code holds two
//   * installation time said as a bare word: "on a fresh install",
//     "Allow Changes ships switched on"
//
// What walks past it, every one of these measured against the whole tree
// rather than reasoned about, is on `what_it_says_reaches_a_provider` and on
// `PUTS_A_SENTENCE_AT_INSTALLATION_TIME`. Read them before trusting a quiet
// run. The short version is that a word from either list used about something
// other than what leaves this computer still beats the word making the claim
// when it sits nearer, and that this reads one sentence that names the setting
// and nothing else.

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
/// About *when* rather than about how somebody happened to word it, and cut
/// back to the bare word wherever a bare word will do. "default" on its own
/// covers "the default", "by default" and "the shipped default", which between
/// them are seven of the nine copies; the list this replaces held four whole
/// wordings and so knew none of the five that followed.
///
/// The same mistake was still here in the second half of the list, though. It
/// held "new installation" and "new install", so "on a fresh install" walked
/// past, and "shipped" without "ships", so "Allow Changes ships switched on for
/// mail" walked past. Both are the bare word now.
///
/// It is not a closed class and this file should stop saying it is. English has
/// more ways of putting a sentence at installation time than a list can hold,
/// and each of these was measured walking past the whole tree: "As it comes,
/// Allow Changes sends no contact to Google", "Allow Changes arrives refusing
/// every change to a contact", "Straight after setup, Allow Changes refuses
/// every change to a contact". None of them is closeable by another word, only
/// by another wording, which is the thing this list stopped being.
///
/// The scope rule is the other half of the same limit. Prose that never names
/// the setting is not read at all, so "By default your contacts stay on this
/// computer" is quiet. That is deliberate, for the reason on
/// [`about_the_setting`], and it is still a false sentence nobody is told
/// about.
const PUTS_A_SENTENCE_AT_INSTALLATION_TIME: &[&str] = &[
    "default",
    "defaults",
    "shipped",
    "ships",
    "installation",
    "install",
    "out of the box",
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
    // English says a refusal with a place or a person as readily as with
    // "not", and a sentence built that way carries no other negative for the
    // reading to find: "a contact is kept here and sent nowhere" was quiet
    // against the whole tree.
    "nowhere",
    "nobody",
    "neither",
    "nor",
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

/// Words above that English uses as prepositions as well.
///
/// "on" is the whole list and it is the reason this exists. It is the word for
/// a switch that is on, and it is also one of the commonest prepositions in the
/// language, and this project's own prose says "on this computer" and "on a
/// server" constantly. Read as a switch wherever it appeared, it let a false
/// sentence stand between the installation-time word and the negative that was
/// really making the claim, and the sentence came out as permission:
/// "everything about a person stays *on* this computer, and no contact is ever
/// sent to Google" agreed with the code and was said about nothing.
///
/// [`A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT`] is the same word doing the
/// same damage to a different rule, found the same way.
const ALSO_A_PREPOSITION: &[&str] = &["on"];

/// Words that cannot begin what a preposition is about.
///
/// A preposition takes a noun phrase. The word for a switch takes nothing, so
/// what follows it is the rest of the sentence: a form of "be" ("on is the
/// shipped default"), a conjunction ("on, and mail is off"), another
/// preposition ("on by default", "on in a new installation", "on for tasks"),
/// or an adverb ("turned on deliberately"). None of those can be what a
/// preposition is about, and every one of them is a closed class apart from the
/// adverbs.
///
/// Three prepositions are deliberately missing: "to", "with" and "about". Each
/// completes a phrasal verb where "on" is a particle and no switch is meant,
/// and each would hand the sentence back the reading this rule exists to take
/// away: "the sync moves on to the next contact", "carries on with the sync",
/// "goes on about it". Left out, those read as a preposition and say nothing,
/// which is the quiet direction rather than the wrong one.
///
/// No place word is here either, for the same reason from the other end. "on
/// here" and "on there" are a preposition and its object, and "stored on here
/// by default" walked past a first version of this list that held "here".
///
/// Getting this wrong in the other direction, by leaving out something that
/// really does follow a switch, costs a sentence being read as saying nothing.
/// In source that is [`test_no_comment_names_the_shipped_answer_without_saying_what_it_is`]
/// failing, which is loud. Getting it wrong in this direction, by letting a
/// preposition through, is the silent miss above.
const WHAT_A_PREPOSITION_IS_NEVER_ABOUT: &[&str] = &[
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "and",
    "or",
    "but",
    "so",
    "because",
    "unless",
    "until",
    "while",
    "though",
    "although",
    "if",
    "when",
    "after",
    "before",
    "since",
    "yet",
    "by",
    "in",
    "for",
    "from",
    "at",
    "already",
    "again",
    "instead",
    "anyway",
    "too",
    "also",
    "only",
    "just",
    "now",
    "out",
    "deliberately",
    "explicitly",
    "automatically",
    "permanently",
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

/// Where the sentence answering a question starts and ends.
///
/// A question makes no claim, so a claim written as a question and its answer
/// is not in the sentence carrying the installation-time word. "Which contacts
/// does Allow Changes send by default? None of them" was quiet in both checks:
/// the question carries a word for something going out, which agrees with the
/// code, and the false half is in the answer.
///
/// `None` when the question is the last thing in its run of prose, which is a
/// question nothing here answers.
fn the_answer_to_a_question(text: &str, ends_at: usize) -> Option<(usize, usize)> {
    if !text[ends_at..].starts_with('?') {
        return None;
    }
    let after = ends_at + 1;
    if after >= text.len() {
        return None;
    }
    let (start, end) = the_sentence_around(text, after);
    (!text[start..end].trim().is_empty()).then_some((start, end))
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
/// What it cannot read, so that a quiet run is not taken for more than it is.
/// Every one of these was measured walking past the whole tree, and the wording
/// that did it is written beside it so somebody can measure it again.
///
/// 1. A word from either list used about something other than what leaves this
///    computer, sitting nearer the installation-time word than the word making
///    the claim. "By default whatever goes into your address book stays there
///    under Allow Changes, and Google is told nothing" reads as permission:
///    "goes" is two words away and is about what arrives, "nothing" is thirteen
///    away and is the claim. This is the family the preposition belonged to,
///    and closing "on" closed the commonest member, not the family. There is no
///    reading here that can tell an incidental verb from a claiming one.
/// 2. A sentence naming both answers is one claim about both, so a true half
///    hides a false one. "By default Allow Changes sends your calendar to
///    Google and sends your mail there too" is half right and goes unsaid.
/// 3. A sentence that carries no word from either list. "The setting is the
///    shipped default" says which answer it is about and never says what that
///    answer is, and one of the nine copies is exactly that. In source
///    [`test_no_comment_names_the_shipped_answer_without_saying_what_it_is`]
///    refuses it outright. In a document nothing does.
/// 4. A negation of a negation. "Out of the box, nothing stops a message
///    reaching the server" reads as a refusal, which is what the code says, so
///    it is quiet while saying the opposite.
/// 5. A claim spread over two statements, because only the sentence carrying
///    the installation-time word is read. "Allow Changes has one answer by
///    default. Nothing about a contact goes to Google" is caught in source by
///    the check above, for saying nothing, and is quiet in a document.
///
///    A question and its answer is the half of this that is read, and it was
///    not. This entry used to claim the whole shape was caught in source, and
///    "Which contacts does Allow Changes send by default? None of them" was
///    silent in both checks: the question carries a word for something going
///    out, which agrees with the code, so a claim was read and it was the
///    wrong half. The answer is read now. What is still quiet is a question
///    nothing in the same run of prose answers, which is what a heading in a
///    document is.
/// 6. A sentence about the setting that quotes or denies a wording rather than
///    making a claim. "It would be wrong to say Allow Changes is off by default
///    for contacts" is true and is named. That one is loud rather than quiet,
///    which is the direction to be wrong in, but it is still wrong.
///
/// Six is what has been measured, not what exists. This compares English prose
/// with a value in the code, and no list of six can be the whole of what a
/// person can write, so a quiet run means these six were looked for and no
/// wrong sentence of a shape already known was found. It does not mean the
/// prose is true. The list before this one named four and the fifth thing tried
/// against it walked past in ten minutes, and the sentence above that list said
/// a fifth copy would fail whatever it said.
fn what_it_says_reaches_a_provider(sentence: &str, marker_at: usize) -> Option<Reaches> {
    let readable = without_the_settings_name(sentence);
    let words = words_of(&readable);
    let marker_word = words
        .iter()
        .rposition(|(at, _)| *at <= marker_at)
        .unwrap_or(0);

    let said = what_each_word_says(&words);

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

/// What each word says, with a preposition and a negated permission read as
/// what they really are.
///
/// "sends nothing" is not the sentence permitting anything, and neither is
/// "allows none of it" or "is not sent". A word from one list standing next to
/// a word from the other is a negation, and what the pair means is the refusal.
fn what_each_word_says(words: &[(usize, String)]) -> Vec<Option<Reaches>> {
    let mut said: Vec<Option<Reaches>> = (0..words.len())
        .map(|at| what_one_word_says(words, at))
        .collect();

    let refused = |at: usize| said.get(at) == Some(&Some(Reaches::Nothing));
    let negated: Vec<usize> = (0..said.len())
        .filter(|at| said[*at] == Some(Reaches::Something))
        .filter(|at| {
            (1..=A_NEGATION_REACHES_THIS_FAR)
                .any(|away| at.checked_sub(away).is_some_and(refused) || refused(at + away))
        })
        .collect();
    for at in negated {
        said[at] = None;
    }
    said
}

/// How far a negative reaches to turn a permission into a refusal.
///
/// Two words, because English puts a pronoun or an adverb between a verb and
/// its negative as readily as it puts them side by side: "sends it nowhere",
/// "sends her nothing", "is hardly ever sent". One word was the window until
/// "By default Allow Changes keeps a task on this computer and sends it
/// nowhere" was measured walking past the whole tree, with "it" holding the
/// two apart. It buys "nothing is allowed" as well, which one word never read.
///
/// Two rather than the whole sentence, because reading the whole sentence is
/// what the nearest-word rule is for. A sentence can carry a refusal about one
/// answer and a permission about the other, and swallowing every permission
/// that shares a sentence with a refusal would read "a new installation allows
/// tasks, and this test turns that off" as a refusal.
const A_NEGATION_REACHES_THIS_FAR: usize = 2;

/// What one word says, or nothing when it is not saying which way it is set.
fn what_one_word_says(words: &[(usize, String)], at: usize) -> Option<Reaches> {
    let word = words[at].1.as_str();
    if NOTHING_GOES_OUT.contains(&word) {
        return Some(Reaches::Nothing);
    }
    if SOMETHING_GOES_OUT.contains(&word) && !a_preposition_here(words, at) {
        return Some(Reaches::Something);
    }
    None
}

/// Whether the word here is a preposition rather than the word for a switch.
///
/// Decided by what follows it, because that is the difference: a preposition is
/// about the noun phrase after it and the word for a switch is about nothing.
/// So "on this computer" and "on a server" are places, and "on by default",
/// "on is the shipped default" and "turns it on" are answers.
fn a_preposition_here(words: &[(usize, String)], at: usize) -> bool {
    if !ALSO_A_PREPOSITION.contains(&words[at].1.as_str()) {
        return false;
    }
    words
        .get(at + 1)
        .is_some_and(|(_, next)| !WHAT_A_PREPOSITION_IS_NEVER_ABOUT.contains(&next.as_str()))
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
                // A question claims nothing, so the claim is in what answers
                // it. What the sentence is about is still read from both
                // halves, because "which contacts" is in the question and the
                // answer is "none of them".
                let (reading, marker) = match the_answer_to_a_question(&prose.text, end) {
                    Some(answer) => (answer, 0),
                    None => ((start, end), at - start),
                };
                let sentence = prose.text[start..reading.1].trim().to_string();
                claims.push(Claim {
                    line: prose.line_at(at),
                    answer: which_answer_it_is_about(&sentence),
                    reaches: what_it_says_reaches_a_provider(
                        &prose.text[reading.0..reading.1],
                        marker,
                    ),
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
        // A preposition standing between the installation-time word and the
        // negative. This project's prose says "on this computer" constantly,
        // and the first reading counted every one of them as the word for a
        // switch that is on. Both of these were quiet against the whole tree.
        (
            "src/application/calendar.rs",
            "// By default everything about a person stays on this computer under Allow\n\
             // Changes, and no contact is ever sent to Google.\n",
        ),
        (
            "src/application/contacts_sync.rs",
            "// Allow Changes defaults to holding personal information on this computer,\n\
             // so a contact edited here never reaches Google.\n",
        ),
        // Written against the reading above rather than found in the tree,
        // which is the half a corpus of what people have already written can
        // never cover. Each is a different grammatical shape and each was
        // measured walking past before the reading was widened to hold it.
        //
        // A refusal said with a place word rather than with "not".
        (
            "src/application/contacts_sync.rs",
            "// By default a contact is kept here and sent nowhere under Allow Changes.\n",
        ),
        // A preposition whose object is a place word, which is the one word
        // that can follow "on" both ways round.
        (
            "src/application/calendar.rs",
            "// Nothing about a contact is stored on here by default, says Allow Changes.\n",
        ),
        // The other half of a wrong sentence, said about mail. The two answers
        // cost different amounts to get wrong and this is the expensive one:
        // it tells somebody a message can go out when nothing sends it.
        (
            "src/application/allowed.rs",
            "/// Allow Changes ships switched on for mail.\n",
        ),
        // The installation-time word in a wording the list did not hold. It
        // held "new install" and not "fresh install", which is a list of
        // wordings being one wording behind in the other half of the reading.
        (
            "docs/ALPHA_TESTING.md",
            "On a fresh install, Allow Changes sends no contact anywhere.\n",
        ),
        // A question and the sentence that answers it. Only the sentence
        // carrying the installation-time word is read, and here that sentence
        // is the question, which claims nothing. The claim is in the answer.
        (
            "docs/ALPHA_TESTING.md",
            "Which contacts does Allow Changes send by default? None of them.
",
        ),
        // A negative held one word away from the verb it negates, which is
        // where English puts a pronoun. "sent nowhere" was read and "sends it
        // nowhere" was not.
        (
            "src/application/tasks_sync.rs",
            "// By default Allow Changes keeps a task on this computer and sends it\n\
             // nowhere.\n",
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
        // The answer to a question, when the answer agrees with the code. A
        // check that reads the answer has to be quiet on a true one or it
        // cries wolf on every question anybody writes.
        (
            "docs/ALPHA_TESTING.md",
            "Which contacts does Allow Changes send by default? It sends every one 
of them.
",
        ),
        // And the same question about mail, where the true answer is the
        // opposite one. This is the pair that makes it a reading rather than a
        // search for a word.
        (
            "docs/ALPHA_TESTING.md",
            "Which messages does Allow Changes send by default? None of them.
",
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

// ── A control that announces a setting nothing keeps ────────────────────────
//
// Four of these were standing in the settings dialog at once: a checkbox
// saying a signature goes on every new message, a choice of message format, a
// checkbox for threaded view, and one saying remote images are not loaded.
// Each was built, given an accessible name, given a value and added to its
// panel, and then never mentioned again. Nothing read any of them back, so
// nothing could save one. A screen reader announces the value the control was
// handed, and that value is a statement about a setting that is not there,
// made in the only channel somebody who cannot see the screen has.
//
// Three of the same shape had already been found by hand in this one file and
// are named in its comments: an autosave checkbox that claimed sixty seconds,
// two spell-check boxes that were ticked while nothing checked anything, and a
// mark-as-read choice with four fixed answers and nothing that saved them. So
// it is a failing build rather than something somebody has to keep noticing.
//
// The shape it reads: a widget that holds an answer, built inside one of the
// tab builders, whose binding never reaches the tuple that builder hands back.
// That tuple is the only way a value leaves the function and reaches
// `read_settings`, so a control missing from it cannot be saved by any route.
//
// What it cannot see. A control that reaches the tuple and is then read into
// the wrong field, or into no field at all, because `read_settings` is a list
// of assignments and this does not read it. A control on a window other than
// this one. And a value read back into a configuration field nothing else
// reads, which is a whole setting doing nothing rather than one control.

/// The file that builds the settings dialog.
const WHERE_THE_SETTINGS_ARE_BUILT: &str = "src/presentation/wx_settings.rs";

/// The widget types that hold an answer somebody can change.
///
/// A `StaticText` is not one: it says something and holds nothing, so there is
/// nothing to read back from it. A `Button` does something when it is pressed
/// rather than carrying an answer that has to be saved.
const HOLDS_AN_ANSWER: [&str; 4] = ["CheckBox", "Choice", "SpinCtrl", "TextCtrl"];

/// One tab builder of the settings dialog, as text.
struct TabBuilder {
    /// What it is called, for the failure message.
    name: String,
    /// The line its `fn` sits on, so an offset into the body names a real line.
    first_line: usize,
    /// Everything from the `fn` line to the closing brace.
    body: String,
}

/// Every `fn build_..._tab` in the settings dialog.
///
/// A body ends at the first line that is a closing brace on its own at the
/// start of a line, which is what `rustfmt` writes for the end of a function
/// and never for anything inside one.
fn tab_builders(text: &str) -> Vec<TabBuilder> {
    let lines: Vec<&str> = text.lines().map(|line| line.trim_end()).collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix("fn build_") else {
            continue;
        };
        let named: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let end = lines[index..]
            .iter()
            .position(|line| *line == "}")
            .map_or(lines.len() - 1, |offset| index + offset);
        found.push(TabBuilder {
            name: format!("build_{named}"),
            first_line: index + 1,
            body: lines[index..=end].join("\n"),
        });
    }
    found
}

/// The same text with everything inside a string literal blanked out.
///
/// So that a semicolon inside a sentence on a label cannot be read as the end
/// of a statement. Blanked rather than cut, and blanked one byte at a time, so
/// an offset into the result is still an offset into the original.
fn without_string_literals(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    let mut escaped = false;
    for character in text.chars() {
        let blank = " ".repeat(character.len_utf8());
        match (inside, character) {
            (false, '"') => {
                inside = true;
                out.push(character);
            }
            (false, _) => out.push(character),
            (true, _) if escaped => {
                escaped = false;
                out.push_str(&blank);
            }
            (true, '\\') => {
                escaped = true;
                out.push_str(&blank);
            }
            (true, '"') => {
                inside = false;
                out.push(character);
            }
            (true, '\n') => out.push('\n'),
            (true, _) => out.push_str(&blank),
        }
    }
    out
}

/// Every control built in one function, as the name it is bound to.
fn controls_built_in(body: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (offset, line) in body.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("let ") else {
            continue;
        };
        let Some((bound, made)) = rest.split_once(" = ") else {
            continue;
        };
        let built = HOLDS_AN_ANSWER
            .iter()
            .any(|widget| made.starts_with(&format!("{widget}::builder(")))
            || made.starts_with("labelled_choice(");
        if built {
            found.push((offset, bound.trim_start_matches("mut ").trim().to_string()));
        }
    }
    found
}

/// What a function hands back, which is the only way a value leaves it.
///
/// Everything after its last statement. These builders all end the same way:
/// the panel is given its sizer, and then the tuple of controls is the trailing
/// expression.
fn what_it_hands_back(body: &str) -> String {
    match without_string_literals(body).rfind(';') {
        Some(at) => body[at + 1..].to_string(),
        None => body.to_string(),
    }
}

/// Whether a control built here leaves the function it was built in.
///
/// In the tuple, or put into a collection that is handed back. The feedback tab
/// builds one checkbox per channel in a loop and pushes each into a list, which
/// is the same thing said differently.
fn leaves_the_function(body: &str, handed_back: &str, name: &str) -> bool {
    let named_in =
        |text: &str| !whole_words_at(&text.to_lowercase(), &name.to_lowercase()).is_empty();
    named_in(handed_back)
        || body
            .lines()
            .filter(|line| line.contains(".push("))
            .any(named_in)
}

/// Every control of the settings dialog that is built and then forgotten.
fn controls_nothing_reads(text: &str) -> Vec<String> {
    let mut forgotten = Vec::new();
    for builder in tab_builders(text) {
        let handed_back = what_it_hands_back(&builder.body);
        for (offset, name) in controls_built_in(&builder.body) {
            if !leaves_the_function(&builder.body, &handed_back, &name) {
                forgotten.push(format!(
                    "{WHERE_THE_SETTINGS_ARE_BUILT}:{}: {name}, built in {}",
                    builder.first_line + offset,
                    builder.name
                ));
            }
        }
    }
    forgotten
}

#[test]
fn test_no_settings_control_is_built_and_then_forgotten() {
    let text =
        fs::read_to_string(WHERE_THE_SETTINGS_ARE_BUILT).expect("the settings dialog to be read");

    let forgotten = controls_nothing_reads(&text);

    assert!(
        forgotten.is_empty(),
        "{} controls are built, named and given a value, and never handed back, \
         so nothing reads them and nothing saves them. A screen reader still \
         announces the value each one was handed, which is a setting somebody \
         is told about and cannot have. Wire it to a real setting or take it \
         out:\n  {}",
        forgotten.len(),
        forgotten.join("\n  ")
    );
}

#[test]
fn test_the_forgotten_control_check_can_tell_the_two_apart() {
    // Proving the measurement. A check that reads nothing passes, and from
    // outside that is indistinguishable from a check that reads everything.
    let text =
        fs::read_to_string(WHERE_THE_SETTINGS_ARE_BUILT).expect("the settings dialog to be read");
    let builders = tab_builders(&text);
    assert!(
        builders.len() >= 7,
        "only {} tab builders found, so the reading is broken",
        builders.len()
    );
    let built: usize = builders
        .iter()
        .map(|builder| controls_built_in(&builder.body).len())
        .sum();
    assert!(
        built > 25,
        "only {built} controls found across the tabs, so the reading is broken"
    );
    for wanted in ["build_compose_tab", "build_feedback_tab"] {
        assert!(
            builders.iter().any(|builder| builder.name == wanted),
            "{wanted} was not found, so the reading is broken"
        );
    }

    // A control that is handed back, and one held on to. Both are the real
    // shape, written out here because this file is left out of the walk.
    let kept = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> CheckBox {\n\
        \x20   let preview_cb = CheckBox::builder(panel).with_label(\"Show me\").build();\n\
        \x20   preview_cb.set_value(config.preview_before_send);\n\
        \x20   panel.set_sizer(sizer, true);\n\
        \x20   preview_cb\n\
        }\n";
    assert!(
        controls_nothing_reads(kept).is_empty(),
        "a control that is handed back was named"
    );

    let forgotten = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> CheckBox {\n\
        \x20   let preview_cb = CheckBox::builder(panel).with_label(\"Show me\").build();\n\
        \x20   let sig_cb = CheckBox::builder(panel).with_label(\"Sign it\").build();\n\
        \x20   sig_cb.set_value(true);\n\
        \x20   sig_sec.add(&sig_cb, 0, SizerFlag::All, 4);\n\
        \x20   panel.set_sizer(sizer, true);\n\
        \x20   preview_cb\n\
        }\n";
    let named = controls_nothing_reads(forgotten);
    assert_eq!(named.len(), 1, "{named:?}");
    assert!(named[0].contains("sig_cb"), "{named:?}");

    // One built in a loop and pushed into the list that is handed back, which
    // is how the feedback tab builds its channels.
    let pushed = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> Vec<CheckBox> {\n\
        \x20   for channel in Channel::ALL {\n\
        \x20       let cb = CheckBox::builder(panel).with_label(label).build();\n\
        \x20       boxes.push((channel, cb));\n\
        \x20   }\n\
        \x20   panel.set_sizer(sizer, true);\n\
        \x20   boxes\n\
        }\n";
    assert!(
        controls_nothing_reads(pushed).is_empty(),
        "a control pushed into the list was named"
    );

    // A semicolon inside a label is not the end of the last statement. Read as
    // one, everything after it becomes the trailing expression and every
    // control in the function reads as handed back.
    let semicolon = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> CheckBox {\n\
        \x20   let note = StaticText::builder(panel).with_label(\"Off; nothing is sent\").build();\n\
        \x20   let sig_cb = CheckBox::builder(panel).with_label(\"Sign it\").build();\n\
        \x20   panel.set_sizer(sizer, true);\n\
        \x20   note\n\
        }\n";
    assert_eq!(controls_nothing_reads(semicolon).len(), 1);

    // And a name that is the start of another name is not read as that one.
    assert!(
        whole_words_at("(sort_choices, other)", "sort_choice").is_empty(),
        "a name is being matched inside a longer one"
    );
}

// ── Which answer Enter gives on a yes or no question ────────────────────────
//
// A wxWidgets message box with Yes and No answers makes Yes the one Enter
// gives unless it is told otherwise. Two of the three questions in this
// program are asked before something that cannot be undone: deleting a
// contact, a task, a note, an event or a reminder, and deleting a list, a
// calendar or a notebook with everything in it. Both had Yes waiting on
// Enter, so somebody who pressed Enter partway through hearing the question
// had deleted the thing before the sentence finished.
//
// Anybody can be caught by that and it costs a screen reader user more, for
// two reasons. Hearing the question takes longer than reading it, so there is
// more of it left when the finger moves; and Enter is how a person working by
// keyboard answers everything, so it is already moving.
//
// The third is the composer asking whether to send a message the spell checker
// has doubts about. Yes on Enter there is deliberate: somebody who meant to
// send and heard the warning should not have to go looking for a button, and
// the whole question is in the words, so it can be answered from hearing it.
// It says so where it is asked.
//
// So the answer Enter gives is a decision each of the three has to make on
// purpose, and the way to make somebody make it is to leave nowhere else to
// write the style. Both answers live in `presentation::asking` and nothing
// else names the flag.

/// Where the two answers to that question live.
const WHERE_THE_ANSWER_ENTER_GIVES_IS_DECIDED: &str = "src/presentation/asking.rs";

/// The style flag that puts Yes and No on a message box.
///
/// Built from two pieces so this line is not itself a match, the same way the
/// dash characters at the top of this file are built from their code points.
/// Written out whole it would fail on the file that defines it.
const A_QUESTION_WITH_TWO_ANSWERS: &str = concat!("MessageDialogStyle", "::YesNo");

#[test]
fn test_only_one_module_says_which_answer_enter_gives() {
    let mut written = Vec::new();

    for path in ours_apart_from_this_file() {
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        if path.ends_with(Path::new(WHERE_THE_ANSWER_ENTER_GIVES_IS_DECIDED)) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            if line.contains(A_QUESTION_WITH_TWO_ANSWERS) {
                written.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        written.is_empty(),
        "a message box with Yes and No answers gives Yes to Enter unless it is \
         told otherwise, and somebody who presses Enter partway through hearing \
         the question has answered it. Which answer Enter gives is a decision, \
         so it is made in {WHERE_THE_ANSWER_ENTER_GIVES_IS_DECIDED} and \
         nowhere else. These write the style themselves:\n  {}",
        written.join("\n  ")
    );
}

#[test]
fn test_the_answer_enter_gives_is_decided_somewhere() {
    // Without this the check above passes once somebody deletes the module it
    // points at, which would leave every question written by hand again and
    // nothing to notice.
    let decided = fs::read_to_string(WHERE_THE_ANSWER_ENTER_GIVES_IS_DECIDED)
        .expect("the module that decides which answer Enter gives");

    assert!(
        decided.contains(A_QUESTION_WITH_TWO_ANSWERS),
        "{WHERE_THE_ANSWER_ENTER_GIVES_IS_DECIDED} no longer builds the style, \
         so the check above is passing over a rule nobody keeps"
    );
    assert_eq!(
        wixen_mail::presentation::asking::yes_no_where_enter_answers_no(),
        wixen_mail::presentation::asking::yes_no_where_enter_answers_yes()
            | wixen_mail::presentation::asking::ENTER_DOES_NOT_ANSWER_YES,
        "the two answers no longer differ by the one flag that separates them"
    );
}

/// Where a control is given the answer it shows, when that answer is a fixed
/// one rather than the stored one.
///
/// The other half of the same defect, and the worse half. A control nothing
/// hands back changes nothing. A control that shows a fixed answer *and* is
/// handed back writes that answer over the stored one when somebody presses
/// OK, so opening the settings window and closing it turns a setting off.
///
/// Three of the four found in this file were checkboxes told `true` or `false`
/// outright, so this is the same three defects read a second way. It matters
/// because of what the check above asks for: told to hand a control back,
/// somebody could hand back one that still shows a fixed answer, and that
/// swaps a control nothing reads for a control that overwrites.
///
/// A choice built on a fixed position is the same fault unless something moves
/// it afterwards. Two are built that way on purpose, because the builder wants
/// a selection before the stored answer has been worked out, and both are then
/// set from it.
fn shows_a_fixed_answer(body: &str, name: &str) -> Option<String> {
    for told in ["true", "false"] {
        let literal = format!("{name}.set_value({told});");
        if body.lines().any(|line| line.trim() == literal) {
            return Some(format!("is told {told} outright"));
        }
    }
    let built = the_statement_binding(body, name)?;
    let fixed = built
        .split_once("with_selection(Some(")?
        .1
        .split_once(')')?
        .0
        .trim()
        .parse::<u32>()
        .is_ok();
    let moved = body.contains(&format!("{name}.set_selection("));
    (fixed && !moved).then(|| "is built on a fixed position and never moved".to_string())
}

/// The statement that binds a name, from its `let` to the end of the statement.
fn the_statement_binding(body: &str, name: &str) -> Option<String> {
    let opener = format!("let {name} = ");
    let at = body.find(&opener)?;
    let mut statement = String::new();
    for line in body[at..].lines() {
        statement.push_str(line);
        if line.trim_end().ends_with(';') {
            return Some(statement);
        }
        statement.push(' ');
    }
    None
}

/// Every control of the settings dialog showing an answer it did not read.
fn controls_showing_a_fixed_answer(text: &str) -> Vec<String> {
    let mut fixed = Vec::new();
    for builder in tab_builders(text) {
        for (offset, name) in controls_built_in(&builder.body) {
            if let Some(how) = shows_a_fixed_answer(&builder.body, &name) {
                fixed.push(format!(
                    "{WHERE_THE_SETTINGS_ARE_BUILT}:{}: {name} {how}",
                    builder.first_line + offset
                ));
            }
        }
    }
    fixed
}

#[test]
fn test_no_settings_control_shows_an_answer_it_did_not_read() {
    let text =
        fs::read_to_string(WHERE_THE_SETTINGS_ARE_BUILT).expect("the settings dialog to be read");

    let fixed = controls_showing_a_fixed_answer(&text);

    assert!(
        fixed.is_empty(),
        "these show a fixed answer rather than the one in the settings file. A \
         screen reader reads out the fixed one, and pressing OK writes it over \
         what somebody chose:\n  {}",
        fixed.join("\n  ")
    );
}

#[test]
fn test_the_fixed_answer_check_can_tell_the_two_apart() {
    // The three that stood in this file, as they stood, and the shapes that
    // are right. Written out here because this file is left out of the walk.
    let told_outright = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> CheckBox {\n\
        \x20   let sig_cb = CheckBox::builder(panel).with_label(\"Sign it\").build();\n\
        \x20   sig_cb.set_value(true);\n\
        \x20   sig_cb\n\
        }\n";
    let named = controls_showing_a_fixed_answer(told_outright);
    assert_eq!(named.len(), 1, "{named:?}");
    assert!(named[0].contains("sig_cb"), "{named:?}");

    let from_the_file = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> CheckBox {\n\
        \x20   let sig_cb = CheckBox::builder(panel).with_label(\"Sign it\").build();\n\
        \x20   sig_cb.set_value(config.add_signature_automatically);\n\
        \x20   sig_cb\n\
        }\n";
    assert!(
        controls_showing_a_fixed_answer(from_the_file).is_empty(),
        "a control filled from the settings file was named"
    );

    // A choice on a fixed position with nothing moving it, and the same choice
    // moved to the stored answer afterwards, which is what two of the real
    // ones do.
    let stuck = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> Choice {\n\
        \x20   let format_choice = Choice::builder(panel)\n\
        \x20       .with_choices(format_choices)\n\
        \x20       .with_selection(Some(0))\n\
        \x20       .build();\n\
        \x20   format_choice\n\
        }\n";
    assert_eq!(controls_showing_a_fixed_answer(stuck).len(), 1);

    let moved = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> Choice {\n\
        \x20   let style_choice = Choice::builder(panel)\n\
        \x20       .with_choices(offered)\n\
        \x20       .with_selection(Some(0))\n\
        \x20       .build();\n\
        \x20   style_choice.set_selection(chosen as u32);\n\
        \x20   style_choice\n\
        }\n";
    assert!(
        controls_showing_a_fixed_answer(moved).is_empty(),
        "a choice set from the stored answer afterwards was named"
    );

    // A position worked out from the settings file is not a fixed one, and it
    // is how most of the choices here are built.
    let worked_out = "fn build_example_tab(panel: &Panel, config: &AppConfig) -> Choice {\n\
        \x20   let theme_choice = Choice::builder(panel)\n\
        \x20       .with_choices(theme_choices)\n\
        \x20       .with_selection(Some(theme_idx))\n\
        \x20       .build();\n\
        \x20   theme_choice\n\
        }\n";
    assert!(
        controls_showing_a_fixed_answer(worked_out).is_empty(),
        "a choice built on a position read from the settings file was named"
    );
}

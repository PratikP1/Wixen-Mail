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
    // build.rs carries prose comments and was outside every reading here.
    for single in [
        "README.md",
        "CLAUDE.md",
        "Cargo.toml",
        ".gitignore",
        "build.rs",
    ] {
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

/// The promises that send somebody looking for a control nothing writes.
///
/// Built from two pieces so that these lines are not themselves a match.
/// Written out whole they would fail on the file that defines them, the same
/// way the dash characters above are built from their code points.
///
/// `AppConfig::allowed_per_account` is read and honoured, and the settings
/// screen writes the application-wide answer only, so nothing outside that
/// field's own tests has ever written one. The testing page and the first-run
/// screen both offered it. The shape they recommended is not one the code can
/// take either: a per-account entry can only ever narrow what the application
/// allows, never widen it.
///
/// A family rather than one phrase, because the promise came back in different
/// words: the getting-started page offered the same absent control as "for
/// each account separately" and walked past a check that knew one wording.
const A_CONTROL_NO_SCREEN_WRITES: &[&str] = &[
    concat!("set it ", "per account"),
    concat!("for each account ", "separately"),
];

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
            if A_CONTROL_NO_SCREEN_WRITES
                .iter()
                .any(|offered| run.contains(offered))
            {
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
// contradicting it has now been written into this repository eleven times in
// eleven wordings, and the check that stood here to stop the fifth missed
// every one of the five that came after it. It held a list of the wordings
// already found, so a new wording walked straight past, and it required the
// setting to be named on the same line, which the copy in the module that
// defines the setting does not do and never could. The tenth and eleventh
// copies were caught by hand, not by the reading: both sat in pages that
// never name the setting, which is the scope door that
// `the_sentence_is_read` was then widened to close.
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
// when it sits nearer, and that a sentence is read only through one of two
// doors: a run that names the setting, or a sentence that both says something
// is written and names the far end it reaches.

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
/// The scope rule is the other half of the same limit. A sentence that
/// neither names the setting nor says what leaves this computer is not read
/// at all, so "By default your contacts stay on this computer" is quiet. That
/// is deliberate, for the reason on [`the_sentence_is_read`], and it is still
/// a false sentence nobody is told about.
///
/// "starts" only counts followed by a word for a state, because English uses
/// the same word for a thing being launched and for where a value begins.
/// "Wixen Mail starts with sending switched off" is the eleventh copy putting
/// itself at installation time, and each completion here is a state it could
/// have been written with. The launch sense was measured crying wolf three
/// ways before this narrowed: "A change takes effect the next time Wixen Mail
/// starts", "a later change starts reading one of these off the server",
/// "forever starts at the second one". Bare "start" stays out entirely: "To
/// use a real account with nothing at risk, start Wixen Mail with
/// `--read-only`" is somebody being told to launch the program, in a run that
/// names the setting.
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
    "starts with",
    "starts switched",
    "starts allowed",
    "starts on",
    "starts off",
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

/// Whether one sentence's claim about a new installation is read at all.
///
/// One question with two doors in, and each door has its own width because
/// each was measured at its width.
///
/// The first door is the run: prose that names the setting, or the module
/// documentation that defines it, is read whole, because the name in one
/// sentence is what its neighbours are about. That is [`about_the_setting`],
/// unchanged.
///
/// The second door is the sentence alone: a sentence that says something is
/// written and names the far end it reaches is about this setting whether or
/// not it says so, because the setting is the only thing standing between the
/// two. This door let the tenth and eleventh copies be read; both sat in
/// pages whose runs never name Allow Changes. It is one sentence wide and not
/// one run wide, measured: the run holding "Keep a copy of sent mail on this
/// computer" carries "sent" and "server" in its neighbouring sentences, and
/// read at run width its true "Off unless you turn it on" sentence would be
/// named for claims two other sentences made.
fn the_sentence_is_read(prose: &Prose, path: &Path, sentence: &str) -> bool {
    about_the_setting(prose, path) || about_something_leaving_this_computer(sentence)
}

/// A sentence that says something is written out, and names where to.
///
/// Both halves are required. The act alone drags in every sentence about
/// sending a message somebody composed; the far end alone drags in every
/// sentence about what a provider holds. "Sent" is deliberately absent from
/// the act list even though it is in [`SOMETHING_GOES_OUT`]: the Sent folder
/// is named across the changelog beside "server" constantly, and both copies
/// this door was built for survive without it, one on "writes" and one on
/// "sending".
fn about_something_leaving_this_computer(sentence: &str) -> bool {
    let lowered = sentence.to_lowercase();
    let words = words_of(&lowered);
    let has = |acts: &[&str]| words.iter().any(|(_, word)| acts.contains(&word.as_str()));
    has(THE_ACT_OF_WRITING)
        && (has(THE_FAR_END)
            || lowered.contains(&THE_PRODUCTS_NAME.to_lowercase())
            || lowered.contains("address book"))
}

/// Words for the act of writing something out to somewhere else.
const THE_ACT_OF_WRITING: &[&str] = &[
    "writes", "write", "send", "sends", "sending", "change", "changes",
];

/// Words naming the far end a write reaches. "Address book" and the product's
/// own name are checked as phrases beside these.
const THE_FAR_END: &[&str] = &[
    "server",
    "servers",
    "provider",
    "providers",
    "google",
    "outlook",
    "microsoft",
];

/// The product's name, which is a label rather than a word about mail.
const THE_PRODUCTS_NAME: &str = "Wixen Mail";

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

/// The sentence with the names that are labels blanked out.
///
/// "Allow Changes" is what the settings screen calls the section, so the word
/// "Allow" in it is a label and not the sentence saying anything is allowed.
/// Left in, every sentence naming the setting reads as permission.
///
/// The product's name is the same trap from the other side. The word "mail"
/// in "Wixen Mail" is a label and not the sentence saying which answer it is
/// about. Left in, "Wixen Mail starts with sending switched off" read as a
/// claim about mail, which really is off, and a sentence that is false about
/// tasks and contacts agreed with the code.
///
/// Blanked rather than cut out, so every other word stays where it was and an
/// offset into the sentence still means what it meant. Cut out, a name after
/// the word being read moved that word, and the reading landed on its
/// neighbour.
fn without_the_names_that_are_labels(sentence: &str) -> String {
    let mut left = sentence.to_string();
    for named in [
        wixen_mail::application::allowed::SETTINGS_SECTION,
        THE_PRODUCTS_NAME,
    ] {
        let lowered = left.to_lowercase();
        let mut at = 0;
        while let Some(found) = lowered[at..].find(&named.to_lowercase()) {
            let start = at + found;
            left.replace_range(start..start + named.len(), &" ".repeat(named.len()));
            at = start + named.len();
        }
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
    let readable = without_the_names_that_are_labels(sentence);
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
///
/// Read with the label names blanked, for the reason on
/// [`without_the_names_that_are_labels`]: the "mail" in the product's name
/// says nothing about which answer a sentence means.
fn which_answer_it_is_about(sentence: &str) -> Answer {
    match answers_named(sentence) {
        (true, false) => Answer::Mail,
        (false, true) => Answer::PersonalInformation,
        _ => Answer::Either,
    }
}

/// Which answers a sentence names: the mail one, and the other one.
///
/// Split out from [`which_answer_it_is_about`] because the sweep below has to
/// tell "names both" from "names neither", and that function collapses them
/// into the same [`Answer::Either`]. Naming both is the sweep it is looking
/// for; naming neither is the contact-group and Safe Browsing prose it must
/// leave alone. Two readings of that one question is how a sentence gets
/// counted as sweeping about both while being judged against the mail half,
/// which is false and quiet at the same time, so there is one reading and both
/// callers use it.
fn answers_named(sentence: &str) -> (bool, bool) {
    let lowered = without_the_names_that_are_labels(sentence).to_lowercase();
    let words = words_of(&lowered);
    let named = |names: &[&str]| words.iter().any(|(_, word)| names.contains(&word.as_str()));
    (
        named(NAMES_THE_MAIL_ANSWER),
        named(NAMES_THE_OTHER_ANSWER)
            || lowered.contains("personal information")
            || lowered.contains("address book"),
    )
}

/// Words a sentence can open with that claim every configuration there is.
///
/// A sentence whose subject is one of these is not describing a setting, it is
/// refusing on behalf of the whole program, so it claims the shipped state
/// without carrying any of the words in
/// [`PUTS_A_SENTENCE_AT_INSTALLATION_TIME`]. That is what made the twelfth
/// copy invisible.
///
/// The opening word and nothing else, because a negative anywhere in a
/// sentence is ordinary and true prose is full of them: read as a door on its
/// own this found twenty-four sentences in the tree and nearly all of them
/// were right.
///
/// "not" and "cannot" are deliberately out. English does not put them at the
/// front of a sentence as its subject, and the words that do go there that way
/// were measured. Every word here is also in [`NOTHING_GOES_OUT`], which
/// [`test_a_sweeping_negative_is_a_word_the_reading_already_knows`] holds
/// them to: a word that opens this door on a sentence the reading then finds
/// no negative in gets that sentence judged backwards.
const A_SWEEP_OPENS_WITH: &[&str] = &["nothing", "nobody", "none", "never", "no", "nowhere"];

/// Words for a far end with nobody left outside it.
///
/// "sends your contacts to anybody" names one answer and still sweeps, because
/// the far end is everybody there is. These are checked instead of adding them
/// to [`THE_FAR_END`], which was the obvious-looking fix and the wrong one:
/// "It is on by default because it sends nothing to anybody", on this
/// project's own privacy page, is true and about a reader that never touches
/// the network, and widening the far end names it.
///
/// "somebody" is out for the same reason from the other side. It is one
/// person rather than all of them, and "Nothing sends a contact group to
/// Google or Microsoft, so the sentence promising it would come back was
/// telling somebody to wait" is true.
const A_UNIVERSAL_FAR_END: &[&str] = &["anybody", "anyone", "everybody", "everyone", "anywhere"];

/// Where a sentence's sweeping refusal opens, when it makes one.
///
/// Three things at once, and each was measured on its own against the whole
/// tree before the other two were added to it.
///
/// It opens with a word from [`A_SWEEP_OPENS_WITH`], read after the label
/// names are blanked, so the claim is the sentence's subject rather than a
/// negative buried in a clause. Alone: twenty-four sentences, nearly all true.
///
/// And it sweeps rather than speaking about one thing, by naming both answers
/// or by naming one and reaching a far end with nobody outside it. Naming both
/// alone, with nothing about how the sentence opens, was seven false alarms,
/// two of them inside snippets this file already pins as true. Both of the
/// above but letting a sentence through that names one answer and no universal
/// far end was four, "Nothing sends a contact group to Google or Microsoft"
/// among them, which is true and has no sync path to be false about.
///
/// What that leaves quiet, so a quiet run is not read for more than it is: a
/// sweeping refusal naming only tasks, only contacts or only the calendar,
/// with an ordinary far end. "Nothing here sends a contact to a provider" is
/// false against the shipped answer and walks past, because the same shape
/// with "contact group" in it is true, and no reading here can tell those two
/// apart.
///
/// The offset handed back is into the sentence with the label names blanked,
/// which is where [`what_it_says_reaches_a_provider`] re-blanks and reads. The
/// two agree because [`without_the_names_that_are_labels`] blanks rather than
/// cuts, and every offset stays where it was. Change that to cutting and this
/// door and both of the older ones land on the wrong word.
fn where_the_sweep_opens(sentence: &str) -> Option<usize> {
    let readable = without_the_names_that_are_labels(sentence);
    let (at, opener) = words_of(&readable).into_iter().next()?;
    if !A_SWEEP_OPENS_WITH.contains(&opener.as_str()) {
        return None;
    }
    let (mail, other) = answers_named(sentence);
    let reaches_everybody = words_of(&readable)
        .iter()
        .any(|(_, word)| A_UNIVERSAL_FAR_END.contains(&word.as_str()));
    ((mail && other) || ((mail || other) && reaches_everybody)).then_some(at)
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
        let lowered = prose.text.to_lowercase();
        let mut said_already: Vec<(usize, usize)> = Vec::new();
        for phrase in PUTS_A_SENTENCE_AT_INSTALLATION_TIME {
            for at in whole_words_at(&lowered, phrase) {
                let (start, end) = the_sentence_around(&prose.text, at);
                if said_already.contains(&(start, end))
                    || !the_sentence_is_read(&prose, path, &prose.text[start..end])
                {
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
        // A sentence can put itself at installation time from the other side,
        // by describing the state that stands "until" somebody grants a
        // permission. Everything after the "until" is the later state, so the
        // claim is read with it blanked. A sentence a marker word already
        // claimed is not read twice.
        //
        // Documents only. A source comment saying "Google is owed it and
        // cannot have it until Allow Changes is on" sits beside the test that
        // just turned the setting off, and is describing the state it built,
        // not the state that ships. A document speaks to somebody who has
        // configured nothing, which is what makes its "until" a claim about
        // the shipped answer. A source comment that really claims the
        // shipped answer says "default" or "ships" and the words above read
        // it.
        let a_document = path.extension().is_some_and(|kind| kind == "md");
        if a_document {
            for at in until_a_permission_at(&lowered) {
                let (start, end) = the_sentence_around(&prose.text, at);
                if said_already.contains(&(start, end))
                    || !the_sentence_is_read(&prose, path, &prose.text[start..end])
                {
                    continue;
                }
                said_already.push((start, end));
                let claimed =
                    the_shipped_state_before_the_until(&prose.text[start..end], at - start);
                claims.push(Claim {
                    line: prose.line_at(at),
                    answer: which_answer_it_is_about(&claimed),
                    reaches: what_it_says_reaches_a_provider(&claimed, at - start),
                    sentence: prose.text[start..end].trim().to_string(),
                });
            }
        }
        // And a sentence can put itself at installation time by covering every
        // moment there is. "Nothing in Wixen Mail sends your messages, your
        // contacts, your calendar or your links to anybody" needs no date on
        // it, because a refusal made on behalf of the whole program has
        // already claimed the shipped answer along with every other. That is
        // [`where_the_sweep_opens`], and it is why the twelfth copy sat at the
        // top of the privacy page unread while two loops above it read the
        // rest of the tree.
        //
        // Source as well as documents, unlike the "until" above. That one is
        // documents-only because a source comment's "until" describes the
        // state the test beside it just built; a sweep describes the program,
        // and a comment claiming the program refuses everybody is as wrong in
        // source as on a page.
        //
        // Last, and sharing `said_already` with both loops above, so a
        // sentence carrying a marker and opening with a negative is one claim
        // and not two.
        for (start, end) in sentence_spans_of(&prose.text) {
            if said_already.contains(&(start, end))
                || !the_sentence_is_read(&prose, path, &prose.text[start..end])
            {
                continue;
            }
            let sentence = prose.text[start..end].trim().to_string();
            let Some(opens_at) = where_the_sweep_opens(&sentence) else {
                continue;
            };
            said_already.push((start, end));
            claims.push(Claim {
                line: prose.line_at(start),
                answer: which_answer_it_is_about(&sentence),
                reaches: what_it_says_reaches_a_provider(&sentence, opens_at),
                sentence,
            });
        }
    }
    claims
}

/// Every "until" that hands its sentence over to a permission, within the
/// next three words of the same sentence.
///
/// "until it is allowed to" and "until you turn it on" put a sentence at
/// installation time from the other side: what stands before the "until" is
/// claimed as the shipped state, and what follows it is the state after
/// somebody acts.
///
/// "unless" is deliberately not read this way, and the difference carries the
/// whole rule. "Nothing goes anywhere unless Allow Changes is on for that
/// account", in a dated changelog entry, describes the gate as it stands
/// whenever you read it and claims nothing about which way the setting ships;
/// it is true and must stay quiet. "Nothing writes to a server until it is
/// allowed to" says the refusal is where a new installation begins, and the
/// code says the opposite for tasks, contacts and the calendar.
///
/// Three words of reach, measured against the wordings this was built for:
/// "until it is allowed" holds its permission three words out.
fn until_a_permission_at(lowered: &str) -> Vec<usize> {
    whole_words_at(lowered, "until")
        .into_iter()
        .filter(|&at| {
            let (_, end) = the_sentence_around(lowered, at);
            words_of(&lowered[at + "until".len()..end])
                .iter()
                .take(HOW_FAR_A_PERMISSION_COMPLETES_AN_UNTIL)
                .any(|(_, word)| A_PERMISSION_AFTER_UNTIL.contains(&word.as_str()))
        })
        .collect()
}

/// Words that complete an "until" into a permission granted later.
const A_PERMISSION_AFTER_UNTIL: &[&str] = &[
    "allow",
    "allows",
    "allowed",
    "turn",
    "turns",
    "turned",
    "on",
    "permit",
    "permits",
    "permitted",
];

/// How many words past "until" its permission can stand.
const HOW_FAR_A_PERMISSION_COMPLETES_AN_UNTIL: usize = 3;

/// The sentence with everything after its "until" blanked, because those
/// words describe the state after somebody acts, and the claim being checked
/// is about the state before.
fn the_shipped_state_before_the_until(sentence: &str, until_at: usize) -> String {
    let mut before = sentence.to_string();
    let from = until_at + "until".len();
    let blank = " ".repeat(before.len() - from);
    before.replace_range(from.., &blank);
    before
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
        // The tenth and eleventh copies, found in the tree after everything
        // above was already being read. Neither names the setting, so the
        // scope rule passed over both, which is what the second door below
        // exists to close: a sentence about the act of writing and the far
        // end it reaches is read even when no run around it names Allow
        // Changes.
        //
        // The comparison page's copy. "until it is allowed to" puts the
        // sentence at installation time by describing the state before
        // somebody acts, and everything after "until" is the later state, so
        // the claim is read with it blanked: "Nothing writes to a server",
        // which the code contradicts for tasks, contacts and the calendar.
        (
            "docs/comparison.md",
            "**Nothing writes to a server until it is allowed to.** Set per capability, with\n\
             the safest answer winning, so an alpha build cannot quietly reorganise a real\n\
             mailbox.\n",
        ),
        // The testing page's copy. It sat nine lines above the table stating
        // the true answer. "starts with" is the marker, and the product's
        // name has to be blanked before the claim is read: left in, the word
        // "mail" in "Wixen Mail" made this a claim about mail, which really
        // is off, and a false sentence agreed with the code.
        (
            "docs/ALPHA_TESTING.md",
            "Wixen Mail starts with sending switched off for exactly that reason. You can\n\
             turn it on, and the next section says how, but read this first.\n",
        ),
        // The twelfth copy, and the first that put itself at no time at all.
        // It opened the privacy page for the whole of this project's life,
        // five lines above a table on the same page saying the opposite, and
        // every check in this file walked past it: it carries no word putting
        // it at installation time, and it did not need one, because a sentence
        // saying nothing is sent to anybody has already claimed every
        // configuration there is, the shipped one among them.
        (
            "docs/privacy.md",
            "Short version: your mail goes to your mail provider and nowhere else. Nothing in Wixen Mail\n\
             sends your messages, your contacts, your calendar or your links to anybody, and there is no\n\
             analytics, no telemetry, no crash reporting service and no update check that says who you are.\n",
        ),
        // Wordings nobody has written yet, one per shape the sweep can take,
        // so this is a reading rather than the one sentence above written out
        // twice. Each was measured walking past before the door was built.
        //
        // One answer named, with a far end that takes in everybody there is.
        (
            "docs/privacy.md",
            "Nothing in Wixen Mail sends your contacts to anybody.\n",
        ),
        // Both answers named, and no word for a universal far end anywhere in
        // it. Naming both is the sweep on its own: a sentence refusing for
        // mail and for everything else at once has left itself nowhere to be
        // true.
        (
            "docs/privacy.md",
            "Nothing in Wixen Mail sends your mail or your calendar to a provider.\n",
        ),
        // A different opening word, and the permission held nine words from
        // the negative that turns it, so the nearest-word reading is what
        // decides it rather than the two-word negation window.
        (
            "docs/privacy.md",
            "No change to your contacts or your messages is sent to a provider.\n",
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
        // The corrections that replaced the tenth and eleventh copies. The
        // comparison page's, held by the second door and by the run door at
        // once, in the shape the module documentation already proved: the
        // claim stops at the colon and the halves after it carry no marker.
        (
            "docs/comparison.md",
            "**Nothing changes at a server without permission, and permission is split by\ncost.** Under Allow Changes, a new installation allows one of the two: tasks,\ncontacts and the calendar go up to a provider, and mail does not. Three places\ncan each say no, the safest answer wins, and the command line can only ever\nnarrow, so an alpha build cannot quietly send or delete anybody's mail.\n",
        ),
        // The testing page's, one snippet per answer because each sentence is
        // its own claim. The mail half first.
        (
            "docs/ALPHA_TESTING.md",
            "Wixen Mail splits that answer in two, under a setting called Allow Changes.\nMail starts switched off: a message that has been sent cannot be recalled.\n",
        ),
        // And the half about everything else, which is the half the old
        // wording lied about.
        (
            "docs/ALPHA_TESTING.md",
            "Wixen Mail splits that answer in two, under a setting called Allow Changes.\nChanging your tasks, contacts and calendar starts switched on: those changes\ngo to your provider, and a task in the wrong place can be moved back.\n",
        ),
        // A true sentence the second door reads with no setting named
        // anywhere near it: the act, the far end, and the product's name
        // blanked before the answer is read.
        (
            "docs/ALPHA_TESTING.md",
            "Wixen Mail tells you a message asked and, by default, sends nothing.\n",
        ),
        // A true "until": the claim before the "until" is the mail half, and
        // the code agrees.
        (
            "docs/ALPHA_TESTING.md",
            "Wixen Mail sends no mail to a server until you allow it.\n",
        ),
        // The correction that replaced the twelfth copy. "Those three" is
        // load-bearing: it keeps the reason attached to the contacts, the
        // calendar and the tasks, which really are allowed, and off the mail,
        // which is not. Held to the code here the way every other correction
        // in this file is, so if the shipped answer ever changes the page
        // fails the build instead of going quietly wrong again.
        (
            "docs/privacy.md",
            "Short version: your mail goes to your mail provider, and your contacts, your calendar and\n\
             your tasks go to the provider you signed in to, because a new installation allows changes\n\
             to those three to be sent.\n",
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
        // The width of the second door, pinned. Written against the reading:
        // the marker sits in one sentence and the act and the far end sit in
        // the next, the shape of the changelog's keep-a-copy entry. Read one
        // sentence at a time nothing here is a claim; read one run at a time
        // the first sentence would be one, and "sent" in "Sent folder" would
        // be what it says goes out.
        (
            "docs/changelog.md",
            "The copy saved here starts in the account's Sent folder. Checking for mail\nsends nothing to the server for it.\n",
        ),
        // "unless" against "until", pinned on the real sentence. This one
        // describes the gate as it stands whenever you read it and claims
        // nothing about which way the setting ships; it is true today
        // whatever the default is. Reading "unless" the way "until" is read
        // would name a dated entry for a claim it does not make.
        (
            "docs/changelog.md",
            "  Nothing goes anywhere unless Allow Changes is on for that account.\n",
        ),
        // Bare "start" against "starts", pinned on the real paragraph. "start
        // Wixen Mail with --read-only" is somebody being told to launch the
        // program, in a run that names the setting; read as an
        // installation-time marker it would be named for a sentence about a
        // command-line flag.
        (
            "docs/ALPHA_TESTING.md",
            "Change them in Settings, under Allow Changes. The answer covers every account\nyou have signed in, so there is no way to leave one account read only and\nallow everything on another. To use a real account with nothing at risk, start\nWixen Mail with `--read-only`, which is next.\n",
        ),
        // The launch sense of "starts", pinned on the two sentences that
        // cried wolf while the word was a bare marker. Neither is about a
        // value beginning anywhere; each is the program or a behaviour being
        // set going.
        (
            "docs/changelog.md",
            "- A change takes effect the next time Wixen Mail starts, not while the dialog is open.\n",
        ),
        (
            "src/application/caldav_sync.rs",
            "/// If a later change starts reading one of these off the server, that field\n\
             /// has to come out of here, or the server's copy will be thrown away instead.\n",
        ),
        // An "until" in a source comment, pinned. It sits beside the test
        // that just turned the setting off and describes the state the test
        // built, which is why the "until" reading holds for documents only.
        (
            "src/application/contacts_sync.rs",
            "        // The same one contact and one edit, with a change goes only to the\n\
             \x20       // address book it came from. Google is owed it and cannot have it until\n\
             \x20       // Allow Changes is on; Outlook is owed it and will not get it while\n\
             \x20       // this setting is off.\n",
        ),
        // True absolutes, pinned because the sweep is a door onto exactly the
        // shape they share and each one is right. Every one is real prose from
        // this tree, and every one was a false alarm under a weaker version of
        // that door.
        //
        // One answer named, no far end that takes in everybody, and true:
        // there is no path anywhere that sends a contact group. This is what
        // "somebody" being left out of the universal far end buys.
        (
            "src/application/new_item.rs",
            "        // Nothing sends a contact group to Google or Microsoft, so the\n\
             \x20       // sentence promising it would come back at the next sync was telling\n\
             \x20       // somebody to wait for something that is never going to happen.\n",
        ),
        // The tripwire for anybody tempted to close the twelfth copy by adding
        // "anybody" to the far end instead. This sentence is on the same page,
        // it is about a reader that works over text already in the message,
        // and it is true. Widen the far end and this is named.
        (
            "docs/privacy.md",
            "It is on by default because it sends nothing to anybody.\n",
        ),
        // Also true, also on that page, and quiet because the act of writing
        // is not in it: what Safe Browsing does with an ordinary message is
        // nothing.
        (
            "docs/privacy.md",
            "For ordinary mail, no link ever matches, and so nothing is sent to Google at all.\n",
        ),
        // Opens with a sweeping negative, names the act and the far end, and
        // names neither answer, so it is a sentence about permission rather
        // than about what a new installation allows. Reading the opening word
        // without asking what the sentence sweeps over names this, and it is
        // true, and it is a correction this file already pins.
        (
            "docs/comparison.md",
            "**Nothing changes at a server without permission, and permission is split by\ncost.**\n",
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
        walked.iter().any(|f| f.ends_with("comparison.md")),
        "the comparison page is not being read, and the tenth copy was in it"
    );
    assert!(
        walked.iter().any(|f| f.ends_with("privacy.md")),
        "the privacy page is not being read, and the twelfth copy was in it"
    );
    assert!(
        !walked.iter().any(|f| f.ends_with("house_style.rs")),
        "this file is in the walk, so the sentences above would fail it"
    );
}

#[test]
fn test_a_sweeping_negative_is_a_word_the_reading_already_knows() {
    // Two lists about one question, which is the shape every data-losing bug
    // in this repository has had. `A_SWEEP_OPENS_WITH` is a writer saying this
    // word is an absolute refusal; `NOTHING_GOES_OUT` is the reader that then
    // has to find a refusal in the sentence it opened. Let them drift and the
    // door opens on a sentence whose reading finds no negative, and the claim
    // comes back as permission or as nothing at all, which is a false sentence
    // called true or dropped in silence.
    let unknown: Vec<&str> = A_SWEEP_OPENS_WITH
        .iter()
        .filter(|word| !NOTHING_GOES_OUT.contains(word))
        .copied()
        .collect();

    assert!(
        unknown.is_empty(),
        "these open the sweep and say nothing to the reading that follows it, \
         so a sentence they let in is judged backwards: {}",
        unknown.join(", ")
    );
}

/// Documents somebody reads, as opposed to source and configuration.
fn documents() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("docs"), &["md"], &mut found);
    found.push(PathBuf::from("README.md"));
    found
}

/// Documents written for somebody using the program, rather than building it.
///
/// A subtraction rather than a list, so a new page is inside the check by
/// default. Every path taken out is checked to exist, because an exclusion
/// that has rotted would widen the reading quietly, and quietly is how this
/// rule got broken twice.
fn documents_people_read() -> Vec<PathBuf> {
    documents()
        .into_iter()
        .filter(|path| !written_for_somebody_building_it(path))
        .collect()
}

/// Pages about how this is built, where a machine name is the subject.
const FOR_SOMEBODY_BUILDING_IT: [&str; 6] = [
    "docs/architecture.md",
    "docs/contributing.md",
    "docs/integration-guide.md",
    "docs/accessibility-framework-evaluation.md",
    "docs/principles.md",
    "docs/brand.md",
];

fn written_for_somebody_building_it(path: &Path) -> bool {
    let named = path.to_string_lossy().replace('\\', "/");
    FOR_SOMEBODY_BUILDING_IT.contains(&named.as_str())
        || named.starts_with("docs/development/")
        || named.starts_with("docs/plans/")
}

/// The files whose whole job is words for people, read as strings only.
const WORDS_FOR_PEOPLE: [&str; 6] = [
    "src/presentation/command_line.rs",
    "src/presentation/first_run.rs",
    "src/presentation/help_page.rs",
    "src/application/help.rs",
    "src/application/allowed.rs",
    "src/presentation/accessibility/announcements.rs",
];

/// Names that are machinery whatever they sit beside.
///
/// Kept short and held to it: every one has to be a real dependency and has to
/// carry no separator, so this cannot become a place to put any word somebody
/// dislikes.
const A_NAME_THAT_IS_ONLY_MACHINERY: [&str; 8] = [
    "reqwest",
    "rusqlite",
    "lettre",
    "ammonia",
    "spellbook",
    "pdfpurr",
    "wxdragon",
    "oauth2",
];

/// The one dependency whose name people also need in its ordinary sense.
///
/// It is a dependency and it is the operating system, and "Windows 10 or
/// later" is a sentence two pages have to be able to say.
const ALSO_AN_ORDINARY_WORD: [&str; 1] = ["windows"];

/// Every dependency, read from the manifest.
///
/// Read rather than typed, for the reason `the_shipped_answer` asks the code:
/// a written copy of a fact goes stale, and the evidence is in this very tree,
/// where three user-facing pages carried version numbers the manifest stopped
/// agreeing with years ago.
///
/// The census in `service::outward` parses the manifest too, for a different
/// question. Both read the file rather than a copy of it, and both fail loudly
/// if the parse returns too little, so neither can quietly go stale while the
/// other does not.
fn every_dependency() -> Vec<String> {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
            continue;
        }
        let Some((name, _)) = line.split_once(" =") else {
            continue;
        };
        let ordinary = !name.is_empty()
            && name.chars().all(|letter| {
                letter.is_ascii_lowercase()
                    || letter.is_ascii_digit()
                    || matches!(letter, '_' | '-')
            });
        if inside && ordinary {
            names.push(name.to_string());
        }
    }
    names
}

/// Why a name in a line of prose is machinery, if it is.
///
/// Three doors, and a name has to go through one of them. A version beside it,
/// which is the shape that reached the changelog. A separator in the name,
/// which no ordinary English word carries. Or a place on the short list above.
///
/// What walks past, said plainly: a bare name with no version and no separator
/// that nobody put on the list, and machinery that is not a dependency at all,
/// which is how a database engine and a widget toolkit survive this reading
/// and were corrected by hand.
fn why_this_name_is_machinery(prose: &str, name: &str) -> Option<String> {
    if ALSO_AN_ORDINARY_WORD.contains(&name) {
        return None;
    }
    let lowered = prose.to_lowercase();
    let mut from = 0;
    while let Some(at) = lowered[from..].find(name) {
        let at = from + at;
        from = at + name.len();
        if !a_whole_word(&lowered, at, name.len()) {
            continue;
        }
        if a_version_follows(&lowered[at + name.len()..]) {
            return Some("with a version beside it".to_string());
        }
        if name.contains(['-', '_']) {
            return Some("and no English word is written that way".to_string());
        }
        if A_NAME_THAT_IS_ONLY_MACHINERY.contains(&name) {
            return Some("which is only ever machinery".to_string());
        }
    }
    None
}

fn a_whole_word(lowered: &str, at: usize, length: usize) -> bool {
    let part_of_a_name = |letter: char| letter.is_alphanumeric() || matches!(letter, '-' | '_');
    let before_is_clear = lowered[..at]
        .chars()
        .next_back()
        .is_none_or(|letter| !part_of_a_name(letter));
    let after_is_clear = lowered[at + length..]
        .chars()
        .next()
        .is_none_or(|letter| !part_of_a_name(letter));
    before_is_clear && after_is_clear
}

fn a_version_follows(rest: &str) -> bool {
    let after = rest.trim_start_matches(' ');
    if after.len() == rest.len() {
        return false;
    }
    let mut token = after
        .chars()
        .take_while(|letter| letter.is_ascii_digit() || matches!(letter, '.' | 'x'));
    token.next().is_some_and(|first| first.is_ascii_digit())
}

/// Every machinery name in `text`, read as a page somebody reads.
fn machinery_named_in(text: &str, names: &[String]) -> Vec<(usize, String, String)> {
    let mut found = Vec::new();
    let mut inside_a_fence = false;
    for (number, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            inside_a_fence = !inside_a_fence;
            continue;
        }
        if inside_a_fence || line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }
        let prose = without_code_and_addresses(line);
        for name in names {
            if let Some(why) = why_this_name_is_machinery(&prose, name) {
                found.push((number + 1, name.clone(), why));
                break;
            }
        }
    }
    found
}

/// Every double-quoted string in a line of source, with addresses left out.
fn quoted_text(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut letters = line.chars();
    while letters.any(|letter| letter == '"') {
        let mut literal = String::new();
        let mut escaped = false;
        for letter in letters.by_ref() {
            match (escaped, letter) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => break,
                (false, _) => literal.push(letter),
            }
        }
        if !literal.contains("://") {
            found.push(literal);
        }
    }
    found
}

#[test]
fn test_no_page_a_person_reads_names_the_machinery() {
    // The written pages are the artefact. What this cannot see is whether a
    // page is clear, only that it does not name the machinery.
    // Round seventeen shipped a crate name and a version into the changelog,
    // and the round after that found it still there. Nobody using this needs
    // to know which library speaks to a mail server, and a name like that read
    // aloud is noise between the reader and the sentence.
    //
    // The names come from the manifest rather than from a list here, so this
    // is about what the project really depends on.
    let names = every_dependency();
    assert!(
        names.len() > 30,
        "only {} dependencies were read, so the manifest is not being parsed and this \
         would report a clean tree whatever the pages said",
        names.len()
    );

    let mut named = Vec::new();
    for document in documents_people_read() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        for (line, name, why) in machinery_named_in(&text, &names) {
            named.push(format!("{}:{line}: {name}, {why}", document.display()));
        }
    }
    for path in WORDS_FOR_PEOPLE {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        for (number, line) in source.lines().enumerate() {
            for spoken in quoted_text(line) {
                for name in &names {
                    if let Some(why) = why_this_name_is_machinery(&spoken, name) {
                        named.push(format!("{path}:{}: {name}, {why}", number + 1));
                    }
                }
            }
        }
    }

    assert!(
        named.is_empty(),
        "these say a machine name to somebody reading or hearing them:\n  {}",
        named.join("\n  ")
    );
}

#[test]
fn test_the_machinery_check_can_tell_the_two_apart() {
    // Proving the measurement. What this cannot see is whether the reading is
    // pointed at the pages people really read; it says only that it can tell a
    // line naming the machinery from one that does not.
    //
    // Every line here was copied out of this tree rather than invented, so a
    // reading that stops working on real pages fails here first.
    let names = every_dependency();
    let fires = |line: &str| !machinery_named_in(line, &names).is_empty();

    // The changelog shape: a name with its version, in a sentence.
    assert!(fires(
        "the library this program uses to speak to mail servers, async-imap 0.11.3, is \
         where this comes from"
    ));
    // The same sentence with the name taken out.
    assert!(!fires(
        "the library this program uses to speak to mail servers is where this comes from"
    ));
    // A list of parts, which is the other shape.
    assert!(fires("- **Database:** rusqlite 0.32 (message caching)"));
    assert!(fires("- HTML sanitization (ammonia)"));
    // The operating system, which two pages have to be able to name.
    assert!(!fires("- Windows 10 or later"));
    // The calendar format, correctly named. Only a version beside it would
    // make this the crate.
    assert!(!fires(
        "Fixed iCalendar property lookup matching on a prefix"
    ));
    // A machine name inside a code span is a machine name doing a machine's
    // job, which is the convention the rest of this file already keeps.
    assert!(!fires("Pinned `rusqlite` to 0.40 in the manifest"));
    // And a fenced block is not prose.
    assert!(!fires("```\nrusqlite 0.40\n```"));

    // A source string is read the same way, and an address in one is not
    // prose at all.
    assert!(
        quoted_text(r#"say("built with wxdragon");"#).contains(&"built with wxdragon".to_string())
    );
    assert!(quoted_text(r#"open("https://github.com/x/wxdragon");"#).is_empty());

    // The list of names cannot be padded with anything that is not really a
    // dependency, and cannot hold a name the separator door already covers.
    for name in A_NAME_THAT_IS_ONLY_MACHINERY {
        assert!(
            names.iter().any(|dependency| dependency == name),
            "{name} is called machinery here and is not a dependency, so this list has \
             become somebody's opinion"
        );
        assert!(
            !name.contains(['-', '_']),
            "{name} is already caught by its separator, so listing it says nothing"
        );
    }
    for name in ALSO_AN_ORDINARY_WORD {
        assert!(
            names.iter().any(|dependency| dependency == name),
            "{name} is excused here and is not a dependency, so the excuse covers nothing"
        );
    }

    // The scope has to be the scope it says it is. An exclusion list that has
    // rotted must fail here rather than quietly widening the check.
    let read = documents_people_read();
    for wanted in ["USER_GUIDE.md", "changelog.md", "README.md"] {
        assert!(
            read.iter().any(|path| path.ends_with(wanted)),
            "{wanted} is not being checked"
        );
    }
    assert!(
        !read.iter().any(|path| path.ends_with("architecture.md")),
        "the pages about how this is built are being checked as if people read them"
    );
    for excluded in FOR_SOMEBODY_BUILDING_IT {
        assert!(
            Path::new(excluded).exists(),
            "{excluded} is taken out of this check and is not there any more"
        );
    }
    for named in WORDS_FOR_PEOPLE {
        assert!(Path::new(named).exists(), "{named} is not there any more");
    }
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

/// Every section of the settings screen whose name is held in a constant.
///
/// One entry each, rather than one test each. A second section added with its
/// own constant and left out of this list would keep the rule in a document
/// and lose the check, which is how the first one drifted.
fn sections_named_by_a_constant() -> [(&'static str, &'static str); 2] {
    [
        (
            wixen_mail::application::allowed::SETTINGS_SECTION,
            "application::allowed::SETTINGS_SECTION",
        ),
        (
            wixen_mail::application::folder_settings::SETTINGS_SECTION,
            "application::folder_settings::SETTINGS_SECTION",
        ),
    ]
}

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

    let mut typed: Vec<String> = Vec::new();
    for (name, constant) in sections_named_by_a_constant() {
        // The name as it stands, not the name it happens to have today.
        // Written out as a literal here, this check would go on forbidding
        // "Allow Changes" after somebody renamed the section to something else
        // and typed the new name in, which is the same fault one step along.
        let written_out = format!("\"{name}\"");
        typed.extend(
            screen
                .lines()
                .enumerate()
                .filter(|(_, line)| line.contains(&written_out))
                .map(|(number, line)| {
                    format!(
                        "{WHERE_THE_SECTION_IS_LABELLED}:{}: {} (read {constant} instead)",
                        number + 1,
                        line.trim()
                    )
                }),
        );
    }

    assert!(
        typed.is_empty(),
        "the name of a section is said elsewhere as the place to go, so it is \
         held in one constant. Written out here it can drift from the sentence \
         again:\n  {}",
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
    // The changelog is the artefact, not a stand-in for behaviour. What this
    // cannot see is whether anything the list names is true.
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
    // The changelog is the artefact, not a stand-in for behaviour. What this
    // cannot see is whether the list really is everything.
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

/// Every version-shaped run in one line: three dotted numbers, and whatever
/// prerelease or build suffix rides on them.
///
/// Three numbers rather than two, so "OAuth 2.0", "WCAG 2.2" and "60.4%" are
/// not versions. A digit or a dot on either side disqualifies a match, so the
/// "0.0" inside "10.0.26200" is not read as a second version.
fn versions_named_in(line: &str) -> Vec<String> {
    let characters: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        let follows_a_word = at.checked_sub(1).is_some_and(|before| {
            characters[before].is_alphanumeric() || characters[before] == '.'
        });
        if !characters[at].is_ascii_digit() || follows_a_word {
            at += 1;
            continue;
        }
        let mut end = at;
        let mut dots = 0;
        loop {
            while end < characters.len() && characters[end].is_ascii_digit() {
                end += 1;
            }
            if dots < 2
                && end < characters.len()
                && characters[end] == '.'
                && characters.get(end + 1).is_some_and(|c| c.is_ascii_digit())
            {
                dots += 1;
                end += 1;
            } else {
                break;
            }
        }
        if dots == 2 && !characters.get(end).is_some_and(|c| *c == '.') {
            if characters.get(end).is_some_and(|c| *c == '-' || *c == '+') {
                end += 1;
                while end < characters.len()
                    && (characters[end].is_alphanumeric()
                        || characters[end] == '.'
                        || characters[end] == '-')
                {
                    end += 1;
                }
            }
            found.push(characters[at..end].iter().collect());
        }
        at = end.max(at + 1);
    }
    found
}

/// The two documents that describe the product as it stands today.
///
/// Only these two. The changelog and the development history are dated
/// records of versions that really shipped, and correcting a version in a
/// record would be falsifying it.
const THE_STATUS_PAGES: &[&str] = &["README.md", "docs/IMPLEMENTATION_STATUS.md"];

/// What one status page says that the code does not ship, line by line.
///
/// Out here rather than inside the check so that the check and the companion
/// under it run the same reading over the same real files. A companion that
/// fed literals to `versions_named_in` would prove the extractor and nothing
/// about the pages being opened, being non-empty, or being read line by line,
/// and those are the links that were broken.
fn wrong_versions_on(page: &str, text: &str, shipped: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for version in versions_named_in(line) {
            if version != shipped {
                wrong.push(format!(
                    "{page}:{}: names {version}, and the code ships {shipped}",
                    number + 1
                ));
            }
        }
    }
    wrong
}

#[test]
fn test_no_status_page_names_a_version_the_code_does_not_ship() {
    // The status page said `0.1.0-alpha.10` and the README said
    // `0.1.0-alpha.12` while the code shipped 0.20.0, which is two breaks of
    // the same rule, and the second break is where the rule gets a check
    // rather than another correction.
    //
    // What this rule costs: if a status page names the running version, every
    // version bump fails this test until that page is touched too. That is
    // the point rather than a false alarm; a page that wants to stay out of
    // the way should point at the changelog instead of naming a number.
    let shipped = env!("CARGO_PKG_VERSION");
    let mut wrong = Vec::new();

    for path in THE_STATUS_PAGES {
        let text = fs::read_to_string(path).expect("a status page to be readable");
        wrong.extend(wrong_versions_on(path, &text, shipped));
    }

    assert!(
        wrong.is_empty(),
        "these pages answer \"does this work yet\", so a version they name is \
         believed, and each of these is not the version the code ships:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_version_reading_can_see_one() {
    // Proving the measurement: the shapes that were really in the two pages,
    // and the neighbours that must not match.
    assert_eq!(
        versions_named_in("at version `0.1.0-alpha.10`. It can send mail."),
        vec!["0.1.0-alpha.10".to_string()]
    );
    assert_eq!(
        versions_named_in("The project is pre-beta, at `0.1.0-alpha.12`."),
        vec!["0.1.0-alpha.12".to_string()]
    );
    assert_eq!(
        versions_named_in("a build carries 0.5.0+g64c73dd"),
        vec!["0.5.0+g64c73dd".to_string()]
    );
    for quiet in [
        "OAuth 2.0 and WCAG 2.2 are not versions",
        "coverage is 60.4%",
        "Windows 10.0.26200.1 has four numbers",
        "commit 4f887c0 is a hash",
    ] {
        assert_eq!(versions_named_in(quiet), Vec::<String>::new(), "{quiet}");
    }
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

/// Every sentence of a run of prose, as where it starts and where it ends.
///
/// The ends are kept because the claim reading names a sentence by the pair,
/// which is how it knows a sentence it has already spoken for. Trimmed, that
/// pair is gone and the two readings could disagree about which sentence is
/// which.
fn sentence_spans_of(prose: &str) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < prose.len() {
        let (start, end) = the_sentence_around(prose, at);
        if !prose[start..end].trim().is_empty() {
            found.push((start, end));
        }
        at = end + 1;
    }
    found
}

/// Every sentence of a run of prose, with where it starts.
fn sentences_of(prose: &str) -> Vec<(usize, &str)> {
    sentence_spans_of(prose)
        .into_iter()
        .map(|(start, end)| (start, prose[start..end].trim()))
        .collect()
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
    // The written page is the artefact. What this cannot see is where the
    // first-run screen really starts, only what the page says about it.
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

/// The mutation script, and the check that runs it on a pull request.
const THE_MUTATION_RUN: &[&str] = &["scripts/mutants.sh", ".github/workflows/mutants.yml"];

/// Everything in a file except the lines explaining it.
///
/// Both files here are allowed to name the mistakes they no longer make, and
/// two of these rules look for exactly those words. Read whole, each check
/// would fail on its own explanation.
fn what_it_does_not_what_it_says(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The lists a mutation run writes alongside its record, for people to read.
///
/// They are created empty before the first mutant is built, so a run that
/// tested nothing has all four and all four say nothing is wrong. Deciding
/// anything from them is how a run whose build failed printed that every
/// mutant was caught and exited clean, and the record next to them said the
/// build had failed the whole time.
const LISTS_WRITTEN_FOR_PEOPLE: &[&str] =
    &["caught.txt", "missed.txt", "unviable.txt", "timeout.txt"];

#[test]
fn test_no_mutation_result_is_read_from_the_lists_written_for_people() {
    // The script is the artefact. What this cannot see is whether a mutation
    // run reports the truth, only where the script reads its answer from.
    let script = fs::read_to_string("scripts/mutants.sh").expect("the mutation script");
    let named: Vec<&str> = LISTS_WRITTEN_FOR_PEOPLE
        .iter()
        .copied()
        .filter(|list| what_it_does_not_what_it_says(&script).contains(list))
        .collect();

    assert!(
        named.is_empty(),
        "the mutation script decides something from {}, which exists and says \
         nothing is wrong before the first mutant is built.\nRead what happened \
         to each mutant instead.",
        named.join(" and ")
    );

    // Reading none of them is not the same as reading the record. A script
    // that decided nothing at all would pass the check above and still let a
    // run that tested nothing go by.
    assert!(
        script.contains("mutants_report.py"),
        "the mutation script hands the run to nothing that reads what happened \
         to each mutant, so nothing can tell a finished run from a run that \
         stopped before it built anything."
    );
}

#[test]
fn test_what_a_change_touched_is_asked_the_same_way_in_both_places() {
    // The two scripts are the artefact. What this cannot see is whether
    // either gives the right answer, only that they ask the same question.
    // Drops every file sitting directly in `src/`, which is how a change to
    // one of them went unchecked. Built from two pieces so this line is not
    // itself a match.
    let drops_the_top_level = concat!("src/", "**/*.rs");

    for path in THE_MUTATION_RUN {
        let text = fs::read_to_string(path).expect("the mutation run");
        assert!(
            !what_it_does_not_what_it_says(&text).contains(drops_the_top_level),
            "{path} asks what changed with a pattern that skips every file \
             sitting directly in src/. Ask for the whole folder."
        );
    }

    let gate = fs::read_to_string(".github/workflows/mutants.yml").expect("the gate");
    assert!(
        gate.contains("scripts/mutants.sh"),
        "the pull request check runs the mutation tool itself rather than the \
         script.\nThen there are two answers to what a change touched and to \
         whether a run tested anything, and the check reads neither."
    );
    assert!(
        !what_it_does_not_what_it_says(&gate).contains("cargo mutants"),
        "the pull request check calls the mutation tool directly, so whatever \
         the script learns about a run that tested nothing, this will not."
    );
}

#[test]
fn test_only_one_place_reads_stored_message_text() {
    // Message text is stored two ways, as text when it is short and packed
    // when packing wins, and knowing which is which belongs in one place.
    // It was in two: `get_message` joined to `message_bodies` and took its
    // text columns directly, so when packing arrived that reader silently
    // came back empty for every body worth packing. Filter rules can match on
    // the text of a message and POP mail has its text stored before the rules
    // run, so a rule matching on what a message said stopped matching.
    //
    // The rule is that SQL naming those columns lives in bodies.rs, which is
    // the module that knows how they are written. Anywhere else reads through
    // `get_message_body`.
    let held_by = "src/data/message_cache/bodies.rs";
    let mut elsewhere = Vec::new();

    for path in [
        "src/data/message_cache/messages.rs",
        "src/data/message_cache/searching.rs",
        "src/data/message_cache/drafts.rs",
        "src/data/message_cache/outbox.rs",
    ] {
        let text = fs::read_to_string(path).expect("a cache module");
        for (number, line) in what_it_does_not_what_it_says(&text).lines().enumerate() {
            // The columns as they appear inside a query, qualified or not.
            let names_a_column = ["body_plain_packed", "body_html_packed"]
                .iter()
                .any(|column| line.contains(column));
            if names_a_column {
                elsewhere.push(format!("{path}:{}", number + 1));
            }
        }
    }

    assert!(
        elsewhere.is_empty(),
        "these read the columns message text is stored in, which only {held_by} \
         should:\n  {}\n\
         Two readers of one fact come apart the first time the writer changes, \
         which is exactly how a filter rule matching on a message's text \
         stopped matching. Read through get_message_body instead.",
        elsewhere.join("\n  ")
    );
}

#[test]
fn test_the_installer_says_how_far_back_the_version_was_set() {
    // The script is the artefact. What this cannot see is whether the number
    // it prints is right; that was measured by hand against twelve real bumps
    // when this was written, and the commit message records the figures.
    //
    // Why there is a check here at all: the versioning rule says a feature or
    // a behaviour change moves the version in the commit that makes it, and
    // that rule has now lapsed three times, twice badly enough to need a
    // catch-up bump. No test can tell a behaviour change from a refactor, so
    // this does not try. It asks only that the one moment where the drift
    // becomes real, somebody building a file to hand to another person, puts
    // the number in front of them.
    let script = fs::read_to_string("scripts/build-installer.sh").expect("the installer script");
    let does = what_it_does_not_what_it_says(&script);

    assert!(
        does.contains("VERSION_SET_AT"),
        "the installer script never works out when the version it is stamping \
         was last moved, so a build carrying a version sixteen commits stale \
         looks exactly like one carrying a version set in the commit before it."
    );

    // Both answers, because they are different answers and a build has to be
    // able to tell them apart. A shallow clone has no history to search, and
    // reporting that as "set 0 commits ago" would read as "just bumped",
    // which is the most reassuring thing it could possibly say and the one
    // thing it does not know.
    assert!(
        does.contains("LAG=\"unknown\"") || does.contains("LAG=unknown"),
        "the installer script has no answer for not being able to work out \
         when the version was set. In a shallow clone the search finds \
         nothing, and a count it does not have must not come out as a number."
    );
}

/// The methods that come back with something to say about somebody's mail.
///
/// Read out of the mail controller rather than listed here, because a list
/// here is a second answer to what the controller offers and the two come
/// apart the first time somebody adds a method.
///
/// `Result<()>` is left out on purpose: it carries a refusal and nothing
/// else, so a caller that only looks at the failure has read all there is.
fn answers_about_somebody_s_mail() -> Vec<String> {
    let controller = fs::read_to_string(
        Path::new("src")
            .join("application")
            .join("mail_controller.rs"),
    )
    .expect("the mail controller");
    let production = controller
        .split("#[cfg(test)]")
        .next()
        .unwrap_or_default()
        .to_string();

    let mut answering = Vec::new();
    for (position, _) in production.match_indices("pub async fn ") {
        let rest = &production[position..];
        // To the end of the signature, so the several that run over more than
        // one line are read as well as the short ones.
        let Some(body_starts) = rest.find(" {\n") else {
            continue;
        };
        let signature = &rest[..body_starts];
        let declared = signature["pub async fn ".len()..].trim();
        let Some(name) = declared.split('(').next() else {
            continue;
        };
        if !signature.contains("-> ") {
            continue;
        }
        let returns = signature.rsplit("-> ").next().unwrap_or_default().trim();
        if returns.starts_with("Result<") && returns != "Result<()>" {
            answering.push(name.trim().to_string());
        }
    }
    answering
}

/// Where a value describing what happened to somebody's mail is dropped.
///
/// The shape, as it really occurred: the answer is worked out, the only place
/// in the running program that asks for it throws it away, and the change
/// gets written up in the changelog as something a person would notice. The
/// compiler cannot see it, because discarding a value is legal, so this can.
///
/// Only the half of a file above the first `#[cfg(test)]` counts. A test may
/// drop an answer it is not asking about.
fn answers_thrown_away_in(path: &Path, text: &str, answering: &[String]) -> (usize, Vec<String>) {
    if path.starts_with("tests") {
        return (0, Vec::new());
    }
    let production = text.split("#[cfg(test)]").next().unwrap_or_default();

    let mut seen = 0;
    let mut dropped = Vec::new();
    for (number, line) in production.lines().enumerate() {
        for method in answering {
            let call = format!(".{method}(");
            if !line.contains(&call) {
                continue;
            }
            seen += 1;
            let before = line.split(&call).next().unwrap_or_default().trim_start();
            let after = line.split(&call).nth(1).unwrap_or_default();
            let thrown_away = before.starts_with("let _ =")
                || before.starts_with("if let Err(")
                || after.contains(").ok();")
                || after.contains(").is_err()");
            if thrown_away {
                dropped.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }
    (seen, dropped)
}

#[test]
fn test_no_answer_about_somebody_s_mail_is_worked_out_and_thrown_away() {
    // The changelog once announced, as a thing somebody would notice, a fix
    // that made one of these answers more accurate. Nothing in the running
    // program read it: the one caller dropped it and carried on. The code was
    // better and nobody's day was different, which is the shape this catches.
    let answering = answers_about_somebody_s_mail();

    // The parse is proved before it is trusted. A parse that quietly found
    // nothing would make every assertion below pass while checking nothing,
    // which is the same defect one level up.
    for named in ["delete_message", "move_message", "remove_these"] {
        assert!(
            answering.iter().any(|found| found == named),
            "the list of answers was read from the mail controller and does \
             not include {named}, so the reading is broken and this check is \
             looking for nothing.\nFound: {answering:?}"
        );
    }
    assert!(
        answering.len() >= 10,
        "only {} answering methods were found in the mail controller, which \
         is fewer than it has. The reading is broken.",
        answering.len()
    );

    let mut sites = 0;
    let mut dropped = Vec::new();
    for path in ours() {
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let (seen, thrown) = answers_thrown_away_in(&path, &text, &answering);
        sites += seen;
        dropped.extend(thrown);
    }

    assert!(
        sites >= 20,
        "only {sites} places in the running program ask any of these, which is \
         fewer than there are. The scan is looking in the wrong place."
    );

    assert!(
        dropped.is_empty(),
        "{} places work out what happened to somebody's mail and throw the \
         answer away, so nothing can say it out loud and no changelog entry \
         about it is true:\n  {}",
        dropped.len(),
        dropped.join("\n  ")
    );
}

#[test]
fn test_the_thrown_away_answer_check_can_tell_the_two_apart() {
    // Without this the check above passes by seeing nothing. Both lines are
    // copied from what the tree really held rather than invented: the first
    // is the draft removal as it stood when the changelog announced a fix to
    // it, the second is the same call once somebody reads the answer.
    let answering = vec!["remove_by_message_id".to_string()];
    let thrown_away = "        if let Err(e) = handle.block_on(controller.remove_by_message_id(&folder, &message_id)) {\n";
    let read = "        let outcome = handle.block_on(controller.remove_by_message_id(&folder, &message_id))?;\n";

    let (seen, dropped) =
        answers_thrown_away_in(Path::new("src/made_up.rs"), thrown_away, &answering);
    assert_eq!(seen, 1, "the check did not see the call at all");
    assert_eq!(dropped.len(), 1, "the check passed a thrown away answer");

    let (seen, dropped) = answers_thrown_away_in(Path::new("src/made_up.rs"), read, &answering);
    assert_eq!(seen, 1, "the check did not see the call at all");
    assert!(
        dropped.is_empty(),
        "the check refused an answer that is read"
    );

    // And the same dropped answer below a `#[cfg(test)]` is a test's business,
    // so the split is not taken on trust either.
    let in_a_test = format!("#[cfg(test)]\nmod tests {{\n{thrown_away}}}\n");
    let (_, dropped) = answers_thrown_away_in(Path::new("src/made_up.rs"), &in_a_test, &answering);
    assert!(dropped.is_empty(), "the check read the test half of a file");
}

#[test]
fn test_the_mutation_report_still_obeys_its_own_examples() {
    // The rules about what a mutation run may be reported as are written as
    // worked examples inside the report itself, and until now the only thing
    // that ran them was the mutation script, which nobody runs on an ordinary
    // change. So the rules held on the days somebody spent an hour on a
    // mutation run and on no other day.
    //
    // Missing python fails rather than skips. A check that quietly does
    // nothing is the whole defect this file keeps finding.
    let examples = std::process::Command::new("python")
        .args(["-m", "doctest", "scripts/mutants_report.py"])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "python could not be run, so the rules the mutation report \
                 follows went unchecked: {e}.\nInstall python and put it on \
                 the path; the mutation run needs it too."
            )
        });

    assert!(
        examples.status.success(),
        "the mutation report no longer does what its own examples say it \
         does.\n{}{}",
        String::from_utf8_lossy(&examples.stdout),
        String::from_utf8_lossy(&examples.stderr)
    );
}

#[test]
fn test_no_mutation_run_has_its_failure_swallowed() {
    // The script is the artefact. What this cannot see is whether a failing
    // run really stops the check that called it.
    let script = fs::read_to_string("scripts/mutants.sh").expect("the mutation script");

    assert!(
        !what_it_does_not_what_it_says(&script).contains("|| true"),
        "the mutation script throws away whether the run failed. A run that \
         stopped early looks exactly like one that finished."
    );
}

#[test]
fn test_only_the_calendar_document_writer_builds_a_day_in_the_calendar_format() {
    // A day and a time written the way a calendar server reads one is built in
    // exactly one file, because the reader that takes such a value apart lives
    // beside it and the two have to answer as one. A second place that builds
    // one is how a cancelled meeting came back onto somebody's calendar: a
    // lenient reader and a strict writer, disagreeing about one day.
    let allowed = Path::new("src").join("service").join("caldav.rs");
    for path in ours() {
        if path.extension().and_then(|e| e.to_str()) != Some("rs") || path == allowed {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !text.contains(".format(\"%Y%m%d"),
            "{} writes a day in the calendar format itself. Ask the calendar \
             document writer for the value instead, so the reader and the \
             writer keep answering as one.",
            path.display()
        );
    }
}

/// What a guard record gets wrong about the one place it points at.
///
/// Over plain strings, so the rule can be proved on made-up input below rather
/// than only on whatever the tree happens to hold today.
///
/// A break has to be exactly one edit, so text that appears twice is reported
/// whatever else is true: the run cannot choose between two places and refuses
/// to guess, and until somebody notices, that record measures nothing. Text
/// that appears nowhere is the same fault with a different cause, and it is
/// reported unless the file is mid-measurement, which is what
/// `file_is_mid_measurement` is for and is explained on the caller.
///
/// Both sides are compared with one kind of line ending. This repository mixes
/// them, and the runner that really applies these breaks reads every file with
/// the line endings translated, so a break written with plain endings matches a
/// file that ends its lines the other way. Comparing raw bytes here would
/// report two records as moved that the runner applies without complaint, which
/// is the same question answered two ways, and the reader that matters is the
/// runner.
fn what_the_guard_record_gets_wrong(
    name: &str,
    before: &str,
    text: &str,
    file_is_mid_measurement: bool,
) -> Option<String> {
    let found = text
        .replace("\r\n", "\n")
        .matches(&before.replace("\r\n", "\n"))
        .count();
    if found > 1 {
        return Some(format!(
            "{name}: the text this break replaces appears {found} times, and a \
             break has to be exactly one edit, so this record cannot be measured \
             at all. Anchor it on something that names one place, such as the \
             routine it is inside."
        ));
    }
    if found == 0 && !file_is_mid_measurement {
        return Some(format!(
            "{name}: the text this break replaces is not in that file any more, \
             so somebody has moved the code this guard is about. Measure the \
             guard by hand and write down what it really does now."
        ));
    }
    None
}

/// Every guard record still points at exactly one place in the tree.
///
/// `scripts/guards.py` is strict about this when it runs, and nothing was
/// strict about it at commit time, so a record went a whole commit pointing at
/// two places while a report said the opposite. A full guards run is an hour or
/// two and nothing forces one; this is the same question answered in
/// milliseconds on every commit.
///
/// What it cannot see: whether the list of tests beside each record is still
/// true. That answer costs a build and a run per record and only
/// `scripts/guards.sh` has it. This says a record still points somewhere, not
/// that it still guards anything.
///
/// The mid-measurement exemption is load-bearing rather than politeness.
/// `scripts/guards.py` applies a break and then runs the suite the record
/// names, and seven records name this suite, so this test runs with their break
/// sitting in the tree. Two of those seven share one identical before-text, so
/// breaking either leaves the other with nothing to match. Without the
/// exemption every one of those runs would report a test going red that the
/// record does not name, and the natural next move would be to weaken this
/// check. A file counts as mid-measurement when some record naming it has an
/// after-text present while that record's own before-text is absent, which is
/// exactly the state a break leaves the file in.
///
/// The exemption's own blind spot: a record whose after-text happens to sit in
/// the tree for real would mark its file mid-measurement for good and hide a
/// genuine "this text has gone" from every record naming that file. Nothing
/// here can tell those two apart. Doubled text is reported either way, which is
/// why that rule has no exemption.
///
/// No record naming this suite has an empty after-text today. If one is ever
/// added, this exemption has to be looked at again, because an empty after
/// deletes the before rather than replacing it and leaves nothing to recognise
/// the break by.
#[test]
fn test_every_guard_record_still_names_one_place_in_the_tree() {
    let written: toml::Value =
        toml::from_str(&fs::read_to_string("guards/guards.toml").expect("the record of guards"))
            .expect("the record of guards to parse");
    let records = written
        .get("guard")
        .and_then(toml::Value::as_array)
        .expect("the record of guards to hold guards");

    let field = |record: &toml::Value, key: &str| -> String {
        record
            .get(key)
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("a guard record with no {key}"))
            .to_string()
    };

    let mut read: Vec<(String, String, String, String)> = Vec::new();
    for record in records {
        let file = field(record, "file");
        let text =
            fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", field(record, "name")));
        read.push((field(record, "name"), file, field(record, "before"), text));
    }

    // Proving the measurement: a parse that quietly found nothing, or fields
    // that quietly came back empty, would report a clean tree and read exactly
    // like one.
    assert!(
        read.len() > 150,
        "only {} guard records were read, so the reading is broken",
        read.len()
    );
    for (name, file, before, _) in &read {
        assert!(
            !name.is_empty() && !file.is_empty() && !before.is_empty(),
            "a guard record came back with an empty field, so the reading is broken"
        );
    }

    let mid_measurement: Vec<String> = records
        .iter()
        .filter(|record| {
            let after = field(record, "after").replace("\r\n", "\n");
            let file = field(record, "file");
            let text = fs::read_to_string(&file)
                .unwrap_or_default()
                .replace("\r\n", "\n");
            !after.is_empty()
                && text.contains(&after)
                && !text.contains(&field(record, "before").replace("\r\n", "\n"))
        })
        .map(|record| field(record, "file"))
        .collect();

    let wrong: Vec<String> = read
        .iter()
        .filter_map(|(name, file, before, text)| {
            what_the_guard_record_gets_wrong(name, before, text, mid_measurement.contains(file))
        })
        .collect();
    // Both ways written out rather than one built from a stem and an "s". This
    // project keeps a guard called "a count and the thing it counts agree in
    // number", and the first run of this test printed "1 guard records".
    let how_many = if wrong.len() == 1 {
        "1 guard record no longer names".to_string()
    } else {
        format!("{} guard records no longer name", wrong.len())
    };
    assert!(
        wrong.is_empty(),
        "{how_many} one place in the tree:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_guard_record_check_can_tell_the_two_apart() {
    // Made-up text only. This never reads `guards/guards.toml`, because
    // records in that file break this suite and a proving test that read the
    // tree would go red under every one of them.
    let once = "fn changes_waiting_here() {}\nfn removals_waiting_here() {}\n";
    assert!(
        what_the_guard_record_gets_wrong("sound", "fn changes_waiting_here() {}", once, false)
            .is_none(),
        "a record that names one place was reported as broken"
    );

    let twice =
        "fn a(count: usize) {\n    match count {\n}\nfn b(count: usize) {\n    match count {\n}\n";
    let doubled = what_the_guard_record_gets_wrong(
        "doubled",
        "(count: usize) {\n    match count {",
        twice,
        false,
    )
    .expect("a record matching twice to be reported");
    assert!(
        doubled.contains("appears 2 times"),
        "the count of places a doubled record matches was not said: {doubled}"
    );

    // Reported even mid-measurement. A break cannot choose between two places
    // whatever else is going on.
    assert!(
        what_the_guard_record_gets_wrong(
            "doubled",
            "(count: usize) {\n    match count {",
            twice,
            true,
        )
        .is_some(),
        "a record matching twice was let through because its file was being measured"
    );

    assert!(
        what_the_guard_record_gets_wrong("gone", "fn nothing_here() {}", once, false)
            .expect("a record matching nothing to be reported")
            .contains("not in that file any more"),
        "a record whose text has gone was not reported"
    );
    assert!(
        what_the_guard_record_gets_wrong("gone", "fn nothing_here() {}", once, true).is_none(),
        "a record whose text is absent because its own file is mid-measurement was reported"
    );

    // The line endings this repository mixes. Two records really do point at
    // files that end their lines the other way, and the runner applies both of
    // them, so reading the bytes as they stand would report two guards as moved
    // that are exactly where they say they are.
    let other_endings = "fn changes_waiting_here() {}\r\nfn removals_waiting_here() {}\r\n";
    assert!(
        what_the_guard_record_gets_wrong(
            "sound",
            "fn changes_waiting_here() {}\nfn removals_waiting_here() {}",
            other_endings,
            false,
        )
        .is_none(),
        "a record pointing at a file that ends its lines the other way was reported as moved"
    );
}

/// The count written immediately before whichever of these endings the text
/// uses.
///
/// Two endings rather than one because a count of one takes a singular verb,
/// and this project keeps a rule that a count and the thing it counts agree in
/// number. A reading that knew only the plural would turn a correctly worded
/// header into a parse failure the first time a sweep was followed by a single
/// new record, and the obvious way out of that would be to word the header
/// wrongly.
fn the_count_before(text: &str, endings: [&str; 2]) -> Option<usize> {
    endings.iter().find_map(|ending| {
        let before = text.split_once(ending)?.0;
        before
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .collect::<Vec<char>>()
            .iter()
            .rev()
            .collect::<String>()
            .parse()
            .ok()
    })
}

/// How many guard records the sweep at the top of `guards/guards.toml` says it
/// went through, and how many have arrived since.
fn what_the_sweep_says_it_covers(record: &str) -> Option<(usize, usize)> {
    let swept = the_count_before(
        record,
        [
            " records were swept that day.",
            " record was swept that day.",
        ],
    )?;
    let since = the_count_before(
        record,
        [
            " records have arrived since and have not been through it.",
            " record has arrived since and has not been through it.",
        ],
    )?;
    Some((swept, since))
}

/// What the sweep at the top of the guard records gets wrong about the file it
/// sits at the top of, if anything.
fn what_the_sweep_header_gets_wrong(record: &str) -> Option<String> {
    let held = record
        .lines()
        .filter(|line| line.trim_end() == "[[guard]]")
        .count();
    let Some((swept, since)) = what_the_sweep_says_it_covers(record) else {
        return Some(format!(
            "the sweep at the top does not say how many records went through it and how \
             many have arrived since, so nobody reading it can tell how much of the file \
             it is about. There are {held} records."
        ));
    };
    if swept + since != held {
        return Some(format!(
            "the sweep at the top accounts for {swept} records swept and {since} arrived \
             since, which is {}, and the file holds {held}. Raise the count of records \
             that have arrived since, or sweep again and write down what that covered.",
            swept + since
        ));
    }
    None
}

/// The sweep written at the top of the guard records covers every record in it.
///
/// It did not. A sweep of 2026-08-12 was written up as the state of the file
/// under a heading that gave no sign of being a photograph, and eighteen
/// records had arrived by the time anybody read it again. Every sentence in
/// that section was about a smaller file than the one it was at the top of, and
/// nothing said so.
///
/// The two numbers add up to the records in the file, so adding a record and
/// leaving the header alone fails here. That is the cost of the header staying
/// true, and it is one line in the same edit.
///
/// What this cannot see: whether a record the sweep counted was really swept.
/// Only `scripts/guards.sh` can say that, at a build and a run per record.
#[test]
fn test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it() {
    let record = fs::read_to_string("guards/guards.toml").expect("the record of guards");
    let held = record
        .lines()
        .filter(|line| line.trim_end() == "[[guard]]")
        .count();
    // Proving the measurement: a reading that counted no records at all would
    // have nothing to disagree with the header about.
    assert!(
        held > 150,
        "only {held} guard records were counted, so the counting is broken"
    );

    assert!(
        what_the_sweep_header_gets_wrong(&record).is_none(),
        "{}",
        what_the_sweep_header_gets_wrong(&record).unwrap_or_default()
    );
}

#[test]
fn test_the_sweep_header_check_can_tell_the_two_apart() {
    // Made-up text only, and deliberately not the numbers the real file
    // carries, so nobody reads this as a second copy of a fact that file
    // already states.
    let sound = "# 3 records were swept that day.\n\
                 # 1 record has arrived since and has not been through it.\n\
                 [[guard]]\n[[guard]]\n[[guard]]\n[[guard]]\n";
    assert_eq!(what_the_sweep_says_it_covers(sound), Some((3, 1)));
    assert!(
        what_the_sweep_header_gets_wrong(sound).is_none(),
        "a header that adds up was reported as wrong"
    );

    // A count of one on the other sentence, worded the way this project words
    // one. Neither sentence may be readable only in the plural.
    let one_swept = "# 1 record was swept that day.\n\
                     # 2 records have arrived since and have not been through it.\n\
                     [[guard]]\n[[guard]]\n[[guard]]\n";
    assert_eq!(what_the_sweep_says_it_covers(one_swept), Some((1, 2)));
    assert!(
        what_the_sweep_header_gets_wrong(one_swept).is_none(),
        "a header worded for a count of one was reported as wrong"
    );

    // A header that says nothing about what it covers, which is the state the
    // real one was in.
    let says_nothing = "# Where this file stood on some day.\n\
                        # 3 records. All 3 checked.\n\
                        [[guard]]\n[[guard]]\n[[guard]]\n";
    assert_eq!(what_the_sweep_says_it_covers(says_nothing), None);
    assert!(
        what_the_sweep_header_gets_wrong(says_nothing)
            .expect("a header that does not say what it covers to be reported")
            .contains("There are 3 records"),
        "a header that does not say what it covers was not reported, or did not say how \
         many records it is at the top of"
    );

    // And the drift itself: three plus one is four, and five are here.
    let stale = format!("{sound}[[guard]]\n");
    assert!(
        what_the_sweep_header_gets_wrong(&stale)
            .expect("a header that no longer adds up to be reported")
            .contains("which is 4, and the file holds 5"),
        "a header that no longer adds up was not reported with both numbers"
    );
}

/// How many test functions a file of Rust holds.
///
/// Anchored on a line whose whole content is the attribute, and the anchor is
/// the entire difficulty. `grep -c '#\[test\]'` answers two for a file with one
/// test whenever the doc comment explaining that test writes the attribute out,
/// and this tree does that nineteen times. A count answered by a mention rather
/// than a use is the mistake this project has made seven times and it is always
/// this one.
///
/// Both spellings. 573 of the tests here are `#[tokio::test]`, so a reader that
/// knew only the bare attribute would report zero for a file whose tests are all
/// asynchronous, and zero is the answer that never disagrees with anything.
fn how_many_tests_are_in(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let line = line.trim();
            line == "#[test]" || line == "#[tokio::test]"
        })
        .count()
}

/// The file a named test lives in, if the tree holds one.
///
/// A test named `a::b::tests::test_c` lives in `src/a/b.rs`, but how many
/// segments sit between the file and the test's own name is not fixed. The test
/// module is usually `tests` and often is not: this tree has
/// `application::sending_later::what_undo_send_is_about::test_x` and
/// `service::protocols::imap::against_a_server_that_answers::test_y`, and a
/// module can be nested inside another. So the file is the longest prefix that
/// is really a file, tried longest first, and a directory module counts through
/// its `mod.rs`.
///
/// A record measured against an integration target names its tests with no
/// module path at all, because they are the top level of `tests/<suite>.rs`.
fn the_file_a_test_lives_in(test: &str, suite: Option<&str>) -> Option<PathBuf> {
    if let Some(suite) = suite {
        return Some(PathBuf::from(format!("tests/{suite}.rs")));
    }
    let parts: Vec<&str> = test.split("::").collect();
    // Longest first. Dropping a fixed number of segments is right for the
    // common shape and wrong twice over: a directory module has no file of its
    // own name, and a test module named for its subject is one more segment
    // than `tests`.
    (1..parts.len()).rev().find_map(|cut| {
        let stem = parts[..cut].join("/");
        [format!("src/{stem}.rs"), format!("src/{stem}/mod.rs")]
            .into_iter()
            .map(PathBuf::from)
            .find(|candidate| candidate.is_file())
    })
}

/// What a record's written-down test counts get wrong about the tree, if
/// anything.
///
/// Both sides are (file, count) pairs sorted by file: what the record says was
/// there, and what is there now.
///
/// Silence is not agreement. A record that has written nothing down is reported
/// rather than passed over, because the other reading gives every record added
/// from now on a permanent exemption, granted by leaving a line out.
fn what_the_recorded_counts_get_wrong(
    name: &str,
    recorded: &[(String, usize)],
    in_the_tree: &[(String, usize)],
) -> Option<String> {
    if recorded.is_empty() {
        return Some(format!(
            "{name}: says nothing about the tests in the {} its red list \
             names, so nothing here can notice one of them gaining a test. \
             Measure it and write down what it agrees with.",
            how_many_files(in_the_tree.len())
        ));
    }

    let moved: Vec<String> = in_the_tree
        .iter()
        .filter_map(
            |(file, now)| match recorded.iter().find(|(was, _)| was == file) {
                Some((_, then)) if then == now => None,
                Some((_, then)) => Some(format!("{file} held {then} tests and holds {now}")),
                None => Some(format!(
                    "{file} is named by the red list and was never counted"
                )),
            },
        )
        .chain(
            recorded
                .iter()
                .filter(|(file, _)| !in_the_tree.iter().any(|(now, _)| now == file))
                .map(|(file, _)| format!("{file} was counted and the red list no longer names it")),
        )
        .collect();

    if moved.is_empty() {
        return None;
    }
    Some(format!("{name}:\n      {}", moved.join("\n      ")))
}

/// A count of files with the word, so a line reads as a sentence.
///
/// The product keeps `how_many` in `src/service/caldav.rs` for this and
/// `guards/guards.toml` guards it, and this file has already printed
/// "1 guard records" once.
fn how_many_files(count: usize) -> String {
    if count == 1 {
        "1 file".to_string()
    } else {
        format!("{count} files")
    }
}

/// The command that re-measures exactly the records this found, and nothing
/// else.
///
/// The whole point of naming them. A full run of the record is 548 builds and
/// 548 suite runs, which is hours, and somebody told only that a record may be
/// stale has no cheaper option than that. Told which records, they have one
/// that costs a build and a run each.
fn how_to_re_measure(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("\"{name}\"")).collect();
    format!("scripts/guards.sh --remeasure {}", quoted.join(" "))
}

/// Every guard record says how many tests were in the files its red list names.
///
/// A record goes stale in a direction that never announces itself: a later
/// change adds a test that reaches the rule the record is about, the record now
/// names too few, and every run stays green. The sweep of 2026-09-01 found
/// twenty-one records in that state and one of them had been written days
/// earlier. Nothing was failing the whole time.
///
/// So each record writes down the tree it was last checked against: for every
/// file its red list names, and for the file it breaks, how many test functions
/// that file held. When one of them gains or loses a test, this fails and names
/// the records to re-measure. That is a source read, so it costs milliseconds
/// and runs on every commit, where the answer it stands in for costs a build and
/// a full suite run per record.
///
/// **What it cannot see, and none of the three is small.**
///
/// A test added to a file no record names can still redden a record, and no
/// count of any file predicts that: the record has never mentioned the file, so
/// there is nothing to compare. Only the full run finds those, and the full run
/// is hours.
///
/// A swap is invisible, because a count is a size and not a set. Delete one test
/// from a file and add another and the number does not move, while the record
/// may have gone stale underneath it. That direction is real: a re-measurement
/// on 2026-09-01 found four tests before and four after with two of the four
/// changed, and one of the two that left had been made blind to the break by
/// somebody improving it.
///
/// And what it costs when it fires is not flat. 471 of the 548 records name one
/// file, so their remedy is one build and one run. The largest shared modules
/// are named by many: a test added to `src/application/contacts_sync.rs` flags
/// 74 records, and at a build and a run each that is hours rather than minutes.
/// `CLAUDE.md` already says there is no clever selection that makes that quick.
/// It also says guard re-measurement belongs off the critical path, which is
/// the honest answer here: run the command it prints in the background and let
/// the follow-up commit carry the corrected records.
///
/// This is a net under the common case, not a replacement for the run. The
/// common case is what caught the project twenty-one times.
#[test]
fn test_every_guard_record_says_how_many_tests_the_files_it_names_held() {
    let written: toml::Value =
        toml::from_str(&fs::read_to_string("guards/guards.toml").expect("the record of guards"))
            .expect("the record of guards to parse");
    let records = written
        .get("guard")
        .and_then(toml::Value::as_array)
        .expect("the record of guards to hold guards");

    let mut wrong: Vec<String> = Vec::new();
    let mut stale: Vec<String> = Vec::new();
    let mut files_read = 0usize;
    for record in records {
        let name = record
            .get("name")
            .and_then(toml::Value::as_str)
            .expect("a guard record with no name")
            .to_string();
        let suite = record.get("suite").and_then(toml::Value::as_str);

        // The files its red list names, and the file it breaks. The second
        // because a unit test lives beside what it covers, so a test arriving
        // in the guarded file is at least as likely to reach the break as one
        // arriving anywhere else, and 34 of these records break a Rust file no
        // test they name lives in. `scripts/guards.py` names one and says what
        // watching them costs.
        let guarded = record
            .get("file")
            .and_then(toml::Value::as_str)
            .expect("a guard record with no file");
        let mut about: Vec<Option<PathBuf>> = record
            .get("red")
            .and_then(toml::Value::as_array)
            .expect("a guard record with no red list")
            .iter()
            .filter_map(toml::Value::as_str)
            .map(|test| the_file_a_test_lives_in(test, suite))
            .collect();
        // Rust files only. A handful of records guard a document, and the count
        // of tests in `README.md` is a number that can never move.
        about.push(guarded.ends_with(".rs").then(|| PathBuf::from(guarded)));

        let mut in_the_tree: Vec<(String, usize)> = Vec::new();
        for file in about.into_iter().flatten() {
            let path = file.to_string_lossy().replace('\\', "/");
            if in_the_tree.iter().any(|(seen, _)| *seen == path) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            files_read += 1;
            in_the_tree.push((path, how_many_tests_are_in(&text)));
        }
        in_the_tree.sort();

        let mut recorded: Vec<(String, usize)> = record
            .get("tests_last_seen")
            .and_then(toml::Value::as_array)
            .map(|written| {
                written
                    .iter()
                    .filter_map(|entry| {
                        Some((
                            entry.get("file")?.as_str()?.to_string(),
                            usize::try_from(entry.get("tests")?.as_integer()?).ok()?,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        recorded.sort();

        if let Some(said) = what_the_recorded_counts_get_wrong(&name, &recorded, &in_the_tree) {
            wrong.push(said);
            stale.push(name);
        }
    }

    // Proving the measurement. A reading that resolved no file, or found no
    // record, would report a clean tree and read exactly like one.
    assert!(
        records.len() > 150 && files_read > 150,
        "{} records and {files_read} files were read, so the reading is broken",
        records.len()
    );

    assert!(
        wrong.is_empty(),
        "{} guard {} measured against a tree that no longer holds those tests:\n  {}\n\n\
         Re-measure exactly those records, which is one build and one run each:\n\n    {}\n",
        stale.len(),
        if stale.len() == 1 {
            "record was"
        } else {
            "records were"
        },
        wrong.join("\n  "),
        how_to_re_measure(&stale)
    );
}

#[test]
fn test_every_test_a_guard_record_names_is_a_test_that_exists() {
    // The failure this cannot be caught by counting, and the one that hides
    // longest.
    //
    // `scripts/guards.py` refuses a record naming a test the harness never ran,
    // and it refuses it before reporting anything about what the break does. So
    // a renamed test does not make a record stale, it makes it *unmeasurable*:
    // every sweep reaching it stops there, and the message reads as a broken
    // tool rather than as a finding. One record sat that way from 2026-08-16 to
    // 2026-09-02, through a six-hour sweep that could not report it.
    //
    // The count fingerprint next door cannot see this by construction. A rename
    // leaves the number exactly where it was, 71 tests before and 71 after, and
    // renaming is the commonest edit a test name ever gets.
    //
    // The commit that caused it shows how little warning there is. It renamed
    // two tests, corrected those same two names in the record above this one,
    // and left them in the record below. Its message says it re-measured both
    // records it touched, and it had: it counted the records whose *code* it
    // edited, and missed the one it broke by renaming a test that record merely
    // *names*. Nothing prompts that direction, so this asks instead.
    let written: toml::Value =
        toml::from_str(&fs::read_to_string("guards/guards.toml").expect("the record of guards"))
            .expect("the record of guards to parse");
    let records = written
        .get("guard")
        .and_then(toml::Value::as_array)
        .expect("the record of guards to hold guards");

    let mut gone: Vec<String> = Vec::new();
    let mut names_read = 0usize;
    for record in records {
        let name = record
            .get("name")
            .and_then(toml::Value::as_str)
            .expect("a guard record with no name");
        let suite = record.get("suite").and_then(toml::Value::as_str);

        for test in record
            .get("red")
            .and_then(toml::Value::as_array)
            .expect("a guard record with no red list")
            .iter()
            .filter_map(toml::Value::as_str)
        {
            let Some(file) = the_file_a_test_lives_in(test, suite) else {
                continue;
            };
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            names_read += 1;

            // The function's own name, which is the last step of the path. The
            // `(` matters: without it a name that is a prefix of another passes
            // on its neighbour.
            let written_as = test.rsplit("::").next().unwrap_or(test);
            if !text.contains(&format!("fn {written_as}(")) {
                gone.push(format!(
                    "{name}:\n      {test}\n      is not in {}",
                    file.to_string_lossy().replace('\\', "/")
                ));
            }
        }
    }

    // Proving the measurement, for the reason the record file gives about every
    // check that reads documents: a reader resolving nothing reports a clean
    // tree and reads exactly like one.
    assert!(
        names_read > 400,
        "{names_read} test names resolved to a file, so the reading is broken"
    );

    assert!(
        gone.is_empty(),
        "{} guard {} a test that is not there:\n\n  {}\n\n\
         A renamed test is the usual cause, and a record naming one cannot be \
         measured at all: the runner reports the lookup failure instead of the \
         check. Find what the test is called now and correct the record, then \
         re-measure it, because a rename can change what the break reddens.\n",
        gone.len(),
        if gone.len() == 1 {
            "record names"
        } else {
            "records name"
        },
        gone.join("\n  ")
    );
}

#[test]
fn test_a_recorded_test_count_counts_tests_and_not_mentions_of_tests() {
    // Made-up source, holding one test of each spelling and two sentences that
    // write the attribute out while explaining them. A reader answered by the
    // mention says four.
    let file = "/// Runs under `#[test]`, which is written here on purpose.\n\
                #[test]\n\
                fn test_one() {}\n\
                \n\
                // The asynchronous ones carry `#[tokio::test]` instead.\n\
                #[tokio::test]\n\
                async fn test_two() {}\n";
    assert_eq!(
        how_many_tests_are_in(file),
        2,
        "the count was answered by the sentences about the tests rather than by the tests"
    );

    // Indented, which is where every test in a `mod tests` block really sits.
    assert_eq!(
        how_many_tests_are_in("    #[test]\n    fn test_a() {}\n"),
        1,
        "a test inside a module was not counted"
    );

    // A file that mentions the attribute and holds no test at all. Zero is the
    // answer that never disagrees with anything, so it has to be reachable
    // honestly rather than by a reader that has stopped finding things.
    assert_eq!(
        how_many_tests_are_in("//! One `#[test]` function, for the reason given above.\n"),
        0,
        "a file whose only mention is prose was counted as holding a test"
    );
}

#[test]
fn test_the_file_a_test_lives_in_is_found_however_deep_its_module_sits() {
    // The ordinary shape: a test in `mod tests` beside the code it covers.
    assert_eq!(
        the_file_a_test_lives_in("application::calendar::tests::test_a", None),
        Some(PathBuf::from("src/application/calendar.rs")),
        "the ordinary shape was not resolved"
    );

    // A named test module rather than `tests`, which is three segments to drop
    // rather than two, and this tree has several.
    assert_eq!(
        the_file_a_test_lives_in(
            "application::sending_later::what_undo_send_is_about::test_a",
            None
        ),
        Some(PathBuf::from("src/application/sending_later.rs")),
        "a test module with a name of its own sent the reading at a file that is not there"
    );

    // A directory module, which lives in `mod.rs` and has no file of its own
    // name.
    assert_eq!(
        the_file_a_test_lives_in("data::message_cache::tests::test_a", None),
        Some(PathBuf::from("src/data/message_cache/mod.rs")),
        "a directory module was not read through its mod.rs"
    );

    // A record measured against an integration target names the target, and its
    // tests have no module path at all.
    assert_eq!(
        the_file_a_test_lives_in(
            "test_no_dashes_that_should_be_punctuation",
            Some("house_style")
        ),
        Some(PathBuf::from("tests/house_style.rs")),
        "a record naming its own suite was not read against that suite's file"
    );
}

#[test]
fn test_the_recorded_count_check_can_tell_a_drift_from_an_agreement() {
    let agreed = [("src/a.rs".to_string(), 12), ("src/b.rs".to_string(), 3)];
    assert!(
        what_the_recorded_counts_get_wrong("a guard", &agreed, &agreed).is_none(),
        "a record whose counts still hold was reported as stale"
    );

    // The drift this exists for: a file the record names gained a test.
    let now = [("src/a.rs".to_string(), 15), ("src/b.rs".to_string(), 3)];
    let said = what_the_recorded_counts_get_wrong("a guard", &agreed, &now)
        .expect("a file that gained tests to be reported");
    assert!(
        said.contains("src/a.rs") && said.contains("12") && said.contains("15"),
        "the drift was reported without the file and both numbers: {said}"
    );
    assert!(
        !said.contains("src/b.rs"),
        "a file that did not move was named as though it had: {said}"
    );

    // Losing one counts too. A deleted test can be the one the record rested on.
    let fewer = [("src/a.rs".to_string(), 11), ("src/b.rs".to_string(), 3)];
    assert!(
        what_the_recorded_counts_get_wrong("a guard", &agreed, &fewer).is_some(),
        "a file that lost a test was not reported"
    );

    // A record that has never been counted at all. Silence must not read as
    // agreement, or adding a record buys it an exemption for ever.
    assert!(
        what_the_recorded_counts_get_wrong("a guard", &[], &agreed).is_some(),
        "a record that records no count at all was treated as agreeing"
    );

    // And the red list moving to a different file, which is a record whose
    // counts are about a tree that is not this one.
    let elsewhere = [("src/c.rs".to_string(), 12), ("src/b.rs".to_string(), 3)];
    assert!(
        what_the_recorded_counts_get_wrong("a guard", &agreed, &elsewhere).is_some(),
        "a red list that now names a different file was not reported"
    );
}

#[test]
fn test_the_command_to_re_measure_names_every_record_and_quotes_it() {
    // Guard names are sentences with spaces in them, so a command that did not
    // quote them would be a command that names a different record.
    assert_eq!(
        how_to_re_measure(&["a moment written with a T".to_string()]),
        "scripts/guards.sh --remeasure \"a moment written with a T\"",
        "one record was not named as the command would have to name it"
    );
    let two = how_to_re_measure(&["first one".to_string(), "second one".to_string()]);
    assert!(
        two.contains("\"first one\"") && two.contains("\"second one\""),
        "a second record was dropped from the command: {two}"
    );
}

/// Every sentence the presentation layer writes for itself, with its tests and
/// its comments taken out.
///
/// A test module here can be nested, and the tests inside one hold snippets of
/// Rust as text, complete with braces in the first column. So neither counting
/// braces nor looking for a closing brace where the module opened is enough on
/// its own, and each of those on its own left several files' tests in the
/// reading. Both together: the module ends at the line that closes it where it
/// opened, and only once the braces have come back to where they started.
///
/// Comments go too, because a comment quoting a sentence is not the code
/// saying it.
fn what_the_windows_say() -> Vec<(PathBuf, String, String)> {
    let mut found = Vec::new();
    let mut files = Vec::new();
    collect(Path::new("src/presentation"), &["rs"], &mut files);
    for path in files {
        let text = fs::read_to_string(&path).expect("a source file to be readable");
        let mut inside = false;
        let mut opening = false;
        let mut depth = 0_i32;
        let mut indent = String::new();
        for line in text.replace("\r\n", "\n").lines() {
            let braces = |c: char| i32::try_from(line.matches(c).count()).unwrap_or(i32::MAX);
            if !inside && !opening && line.trim() == "#[cfg(test)]" {
                opening = true;
                indent = line[..line.len() - line.trim_start().len()].to_string();
                continue;
            }
            if opening {
                if braces('{') > 0 {
                    opening = false;
                    inside = true;
                    depth = braces('{') - braces('}');
                }
                continue;
            }
            if inside {
                depth += braces('{') - braces('}');
                if depth <= 0 && line == format!("{indent}}}") {
                    inside = false;
                }
                continue;
            }
            if line.trim_start().starts_with("//") {
                continue;
            }
            for literal in literals_in(line) {
                found.push((path.clone(), line.to_string(), literal));
            }
        }
    }
    found
}

/// Every double-quoted literal on one line, as it is written.
fn literals_in(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else {
            break;
        };
        found.push(after[..close].to_string());
        rest = &after[close + 1..];
    }
    found
}

/// One place words a row that has gone, and one words a command that failed.
///
/// `pim_command::no_longer_there` already owned the first of those and already
/// added the half the copies left off, that nothing has been changed. Six other
/// places wrote their own shorter version, so somebody heard a different
/// sentence depending on which panel they were in and two of the six left them
/// unsure whether anything had happened. The second was one sentence for every
/// personal information command there is, "That did not work", naming neither
/// the row nor what had been attempted.
///
/// What this cannot see: whether either sentence is ever spoken, whether it is
/// true where it is reached, or whether the owner says the right thing. It
/// reads text. The tests beside `pim_command` are what ask the sentences
/// themselves, and nothing here would notice a panel that had gone silent.
#[test]
fn test_only_one_place_words_a_row_that_has_gone() {
    let mut written_again = Vec::new();
    for (path, line, literal) in what_the_windows_say() {
        for spelling in [
            "is no longer there",
            "no longer exists",
            "That did not work",
        ] {
            if literal.contains(spelling) {
                written_again.push(format!("{}: {}", path.display(), line.trim()));
            }
        }
    }

    assert!(
        written_again.is_empty(),
        "these places word a sentence the personal information commands already own:\n{}",
        written_again.join("\n")
    );
}

/// Proving the reading above, which would pass on an empty list for ever.
#[test]
fn test_the_reading_of_what_the_windows_say_can_tell_the_two_apart() {
    let said = what_the_windows_say();
    assert!(
        said.len() > 500,
        "only {} sentences were read, so the reading is broken",
        said.len()
    );
    assert!(
        said.iter()
            .any(|(_, _, literal)| literal == "No storage is open"),
        "a sentence the windows really do say was not read"
    );
    assert!(
        !said
            .iter()
            .any(|(_, _, literal)| literal.contains("the reading is broken")),
        "the tests were not cut off, so the reading is looking at its own words"
    );
    // Test modules in these files sit between stretches of code rather than
    // after all of them, so a cut that stopped at the first one would leave the
    // last few hundred lines of the main window unread. This sentence is
    // written past the last test module in that file.
    assert!(
        said.iter()
            .any(|(_, _, literal)| literal.contains("Could not save what you chose")),
        "the code after the last test module was cut off with them"
    );

    assert_eq!(
        literals_in(r#"send_status(tx, rt, "That row is no longer there");"#),
        vec!["That row is no longer there".to_string()],
        "a literal on an ordinary line was not read"
    );
    assert!(
        literals_in("let count = rows.len();").is_empty(),
        "a line with no literal on it was read as having one"
    );
}

// ── One failing target must not hide the other fourteen ─────────────────────

/// Whether a line that runs the test suite also asks for every target to be
/// run, rather than stopping at the first one that fails.
///
/// Matching on the whole line rather than just the flag, because the flag is
/// only wanted where the suite is actually being run: a comment mentioning it,
/// or a line that runs one named target on purpose, is not the thing this is
/// about.
fn runs_the_suite(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.starts_with('#')
        && trimmed.contains("cargo test")
        && !trimmed.contains("--doc")
        && !trimmed.contains("--test ")
        && !trimmed.contains("--lib ")
}

fn stops_at_the_first_failure(line: &str) -> bool {
    runs_the_suite(line) && !line.contains("--no-fail-fast")
}

#[test]
fn test_one_failing_target_does_not_hide_the_rest() {
    // `cargo test` runs each target in turn and stops at the first one that
    // fails, so a single failing test in the library means all fourteen files
    // under `tests/` never run at all. Not "run and reported": never started,
    // and absent from the output rather than listed as skipped. That is how a
    // broken guard record reached `main` while CI looked like it had checked
    // it, and it is this project's own guardrail four, a check nobody reads
    // being worse than no check.
    //
    // `--no-fail-fast` still fails the run. It just runs the rest first, so
    // the output says everything that is wrong instead of the first thing.
    let mut stopping = Vec::new();

    for path in ["scripts/check.sh", ".github/workflows/ci.yml"] {
        let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        for (number, line) in text.lines().enumerate() {
            if stops_at_the_first_failure(line) {
                stopping.push(format!("{path}:{}: {}", number + 1, line.trim()));
            }
        }
    }

    assert!(
        stopping.is_empty(),
        "{} place(s) run the suite in a way that hides every target after the \
         first failure:\n  {}",
        stopping.len(),
        stopping.join("\n  ")
    );
}

#[test]
fn test_the_fail_fast_check_can_tell_the_two_apart() {
    // Without this the check above passes by matching nothing, which is the
    // failure mode it exists to prevent somewhere else.
    assert!(stops_at_the_first_failure("cargo test --all-targets"));
    assert!(stops_at_the_first_failure(
        "      run: cargo test --verbose"
    ));

    // What both were changed to.
    assert!(!stops_at_the_first_failure(
        "cargo test --all-targets --no-fail-fast"
    ));
    assert!(!stops_at_the_first_failure(
        "      run: cargo test --verbose --no-fail-fast"
    ));

    // A commented-out line is not something that runs.
    assert!(!stops_at_the_first_failure("# cargo test --all-targets"));
    // Running one target on purpose is not running the suite, so it does not
    // need the flag: there is nothing after it to hide.
    assert!(!stops_at_the_first_failure("cargo test --doc"));
    assert!(!stops_at_the_first_failure("cargo test --test house_style"));
    // A line that runs no tests at all.
    assert!(!stops_at_the_first_failure("cargo build --release"));
}

// ── A handler that can refuse has to consume the click ──────────────────────

/// Every `on_click(...)` closure body in one file, as text.
///
/// Brace-matched from the `on_click(` that opens it, because the bodies being
/// asked about are nested several levels deep and a line-based read would stop
/// at the first inner `}`.
fn on_click_bodies(text: &str) -> Vec<String> {
    // Byte offsets throughout. These files hold box-drawing characters in
    // their comments, so a byte offset from `find` used to index a `Vec<char>`
    // lands somewhere else entirely, and the first version of this did exactly
    // that: it walked off into the middle of a comment and subtracted past
    // zero. Every bracket counted here is ASCII, so a byte slice between two
    // of them is always on a character boundary.
    let mut bodies = Vec::new();
    let bytes = text.as_bytes();
    let mut from = 0;

    while let Some(found) = text[from..].find("on_click(") {
        let opens = from + found + "on_click(".len() - 1;
        let mut depth = 0usize;
        let mut at = opens;
        while at < bytes.len() {
            match bytes[at] {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            at += 1;
        }
        bodies.push(text[opens..(at + 1).min(text.len())].to_string());
        from = opens + 1;
    }
    bodies
}

/// Whether a handler decides for itself whether the dialog closes.
///
/// Both halves matter. One that always closes is not at risk: wxWidgets'
/// own answer for an affirmative button is the same answer, so running it
/// twice changes nothing. One that can leave without closing is the whole
/// problem.
fn can_refuse_to_close(body: &str) -> bool {
    body.contains("end_modal(ID_OK)") && body.contains("return")
}

fn refuses_without_consuming(body: &str) -> bool {
    can_refuse_to_close(body) && !body.contains("skip(false)")
}

#[test]
fn test_a_handler_that_can_refuse_to_close_consumes_the_click() {
    // wxdragon sets Skip(true) before it calls a bound handler and only
    // clears it if the handler says otherwise (wxdragon-sys cpp/src/event.cpp,
    // "Reset skip to true before each handler call", then
    // `if (!event.GetSkipped()) event_consumed = true;` and a final
    // `else { event.Skip(true); }`). A plain button event is not vetoable, so
    // a handler that returns without touching skip lets the click carry on to
    // wxDialogBase::OnButton, which sees wxID_OK, calls AcceptAndClose, and
    // ends the dialog anyway.
    //
    // So a Save that decides not to close is overruled, and the answer it
    // just refused is stored. This shipped: the whole point of the check was
    // to keep the window open, the window closed regardless, and the older
    // checks that used to catch it after the fact had been taken out in the
    // same change. Reading wxWidgets' own source was not enough to see it,
    // because it is wxdragon's dispatch in between that decides.
    let mut overruled = Vec::new();

    for path in test_sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for body in on_click_bodies(&text) {
            if refuses_without_consuming(&body) {
                overruled.push(path.display().to_string());
            }
        }
    }

    assert!(
        overruled.is_empty(),
        "{} handler(s) refuse to close and are then overruled by wxWidgets' \
         own handler, because they never consume the click:\n  {}",
        overruled.len(),
        overruled.join("\n  ")
    );
}

#[test]
fn test_the_consuming_check_can_tell_the_three_apart() {
    // Written out rather than taken from the tree, because the shape this is
    // about is the one that must never be in the tree again.
    let refusing = r#"({
        move |event| {
            if wrong { say(); return; }
            d.end_modal(ID_OK);
        }
    })"#;
    assert!(refuses_without_consuming(refusing));

    let consuming = r#"({
        move |event| {
            event.skip(false);
            if wrong { say(); return; }
            d.end_modal(ID_OK);
        }
    })"#;
    assert!(!refuses_without_consuming(consuming));

    // The common shape, and not at risk: it always closes, so wxWidgets'
    // own answer for an affirmative button is the same answer.
    let always_closes = r#"({
        move |_| {
            d.end_modal(ID_OK);
        }
    })"#;
    assert!(!refuses_without_consuming(always_closes));
    assert!(!can_refuse_to_close(always_closes));
}

#[test]
fn test_the_body_reader_reaches_the_end_of_a_nested_handler() {
    // A line-based read stopped at the first inner brace and saw none of
    // this, which would have let the check above pass by reading nothing.
    let source = r#"
    save.on_click(move |event| {
        if problems.is_empty() {
            dialog.end_modal(ID_OK);
            return;
        }
        if let Some(first) = problems.first() {
            focus(first);
        }
    });
    other.on_click(move |_| dlg.end_modal(ID_CANCEL));
    "#;

    let bodies = on_click_bodies(source);
    assert_eq!(bodies.len(), 2, "{bodies:#?}");
    assert!(
        bodies[0].contains("focus(first)"),
        "the reader stopped early: {}",
        bodies[0]
    );
    assert!(bodies[1].contains("ID_CANCEL"), "{}", bodies[1]);
}

/// No source file carries a control character nobody can see.
///
/// Twice in one session a regex was written with a backspace where a word
/// boundary was meant, because the tool writing the file turned `\b` into the
/// byte it names. The pattern then reads correctly in an editor, in `grep`, and
/// in review, and matches nothing. The first one cost an hour and was found by
/// printing the compiled pattern; the second was found only because the first
/// had happened.
///
/// So this is the check rather than a third careful reading. Tab, newline and
/// carriage return are the only ones a source file has any business holding.
#[test]
fn test_no_source_file_carries_an_invisible_control_character() {
    let mut found = Vec::new();
    for dir in ["src", "tests", "search-handler/src"] {
        let mut files = Vec::new();
        collect_rust_files(Path::new(dir), &mut files);
        for path in files {
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            for (at, byte) in bytes.iter().enumerate() {
                if *byte < 0x20 && !matches!(byte, 0x09 | 0x0a | 0x0d) {
                    let line = bytes[..at].iter().filter(|b| **b == b'\n').count() + 1;
                    found.push(format!("{}:{line} holds byte {byte:#04x}", path.display()));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "these files hold a character nothing will show you, which is how a \
         regex comes to read correctly and match nothing:\n  {}",
        found.join("\n  ")
    );
}

/// A sentence written across two lines of source keeps the indentation.
///
/// Writing a long message inside a string literal and letting it wrap puts
/// every space of the next line's indentation into the middle of the sentence.
/// It is invisible in the source, where the run looks like alignment, and it is
/// not invisible in the product: a wxWidgets label draws every one of those
/// spaces, so a settings description reads with a gap through the middle of it.
///
/// Nineteen of these had been written before anybody looked, which is what
/// makes it worth a check rather than a fix. The answer when this fails is a
/// `\` at the end of the line, which continues a literal and eats the
/// indentation that follows.
#[test]
fn test_no_sentence_is_written_with_the_source_indentation_inside_it() {
    const A_RUN: usize = 5;
    let mut found = Vec::new();
    for dir in ["src", "search-handler/src"] {
        let mut files = Vec::new();
        collect_rust_files(Path::new(dir), &mut files);
        for path in files {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (at, line) in text.lines().enumerate() {
                // Comments say what they like; this is about what is shown.
                if line.trim_start().starts_with("//") {
                    continue;
                }
                // Script written into the page has the same flattening in it
                // and nobody reads it: whitespace between two statements draws
                // nothing and means nothing. Left alone rather than counted,
                // so this stays a check about sentences.
                if line.contains("document.") || line.contains("window.") {
                    continue;
                }
                for (found_at, _) in line.match_indices(&" ".repeat(A_RUN)) {
                    let before = &line[..found_at];
                    // A literal holding a newline is indented on purpose: the
                    // spaces are part of what it is quoting, not a wrap.
                    if before.ends_with("\\n") || !before.contains('"') {
                        continue;
                    }
                    // Only inside a sentence. A run at the start of a literal
                    // is a caller lining something up in a table.
                    if before.ends_with(|c: char| c.is_alphanumeric() || ",.:;".contains(c)) {
                        found.push(format!("{}:{}", path.display(), at + 1));
                    }
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "these lines put the source indentation into the middle of a sentence \
         somebody reads, which draws as a gap in a label:\n  {}",
        found.join("\n  ")
    );
}

fn collect_rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

/// The calendar format is read and written in one place.
///
/// Every one of these answers a question about the calendar line format: where
/// a property's name ends, what a quoted value means, where a long line
/// breaks. A second copy of any of them is the shape every data-losing defect
/// in the calendar code has had, and it does not take time to go wrong: when
/// the invitation work was written against a copy of these, two of the seven
/// had already drifted before either was used. One refused to break a line
/// that opens a component and the other did not; one read a quoted value
/// carrying a semicolon whole and the other cut it in half, which truncated a
/// guest filed as `Smith; John` at the semicolon.
///
/// What this cannot see: a copy written under a different name. It catches the
/// way it happened, which was somebody needing the same answer and writing it
/// out again beside their own work.
#[test]
fn test_the_calendar_line_format_is_answered_in_one_place_only() {
    const ANSWERED_ONCE: &[&str] = &[
        "value_named_on",
        "delimiter_colon",
        "parameter_among",
        "parameter_named_on",
        "written_out",
        "folded",
        "fits_in",
    ];

    let mut files = Vec::new();
    collect_rust_files(Path::new("src"), &mut files);
    assert!(
        files.len() > 50,
        "only {} files were read, so this guard is measuring almost nothing",
        files.len()
    );

    for helper in ANSWERED_ONCE {
        let defined_in: Vec<String> = files
            .iter()
            .filter(|path| {
                // Either spelling of the opening, because some of these carry
                // a lifetime and the rest do not, and a check that only knew
                // one shape found none of them and passed by finding nothing.
                fs::read_to_string(path).is_ok_and(|text| {
                    text.contains(&format!("fn {helper}("))
                        || text.contains(&format!("fn {helper}<"))
                })
            })
            .map(|path| path.display().to_string())
            .collect();

        // `parameter_named_on` is the one exception and it is a narrow one:
        // the calendar service is told which property to expect and the
        // invitation reader is not, because it reads two properties the same
        // way. The scanning underneath both is `parameter_among`, which is
        // why that name is on this list too.
        let allowed = if *helper == "parameter_named_on" {
            2
        } else {
            1
        };
        assert_eq!(
            defined_in.len(),
            allowed,
            "{helper} is answered in {} places, and the calendar format needs \
             one answer:\n  {}",
            defined_in.len(),
            defined_in.join("\n  ")
        );
    }
}

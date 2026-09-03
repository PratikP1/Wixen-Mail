//! Spell checking and internationalization foundation
//!
//! Provides a multi-language spell checker for the compose editor, backed by:
//! - **spellbook** (Hunspell-compatible, pure Rust) when `.aff` + `.dic` files
//!   are available for the active language
//! - A built-in English word list as fallback when no Hunspell data is present
//!
//! This module is also the foundation for future UI translation / i18n support.
//! The `Locale` struct and `I18n` registry provide the plumbing for localizing
//! UI strings, date/number formatting, and message templates.

use crate::common::{Error, Result};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Language metadata ────────────────────────────────────────────────────────

/// Supported language descriptor for dictionary + locale.
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    /// ISO 639-1 language code (e.g. "en", "es", "fr", "de")
    pub code: String,
    /// Human-readable name in English (e.g. "English", "Spanish")
    pub name: String,
    /// Native name (e.g. "English", "Español")
    pub native_name: String,
    /// Expected Hunspell dictionary filenames (e.g. "en_US")
    pub hunspell_name: String,
    /// Character alphabet used for edit-distance suggestions (fallback only)
    pub alphabet: String,
}

/// Get the list of supported languages.
pub fn supported_languages() -> Vec<LanguageInfo> {
    vec![
        LanguageInfo {
            code: "en".into(),
            name: "English".into(),
            native_name: "English".into(),
            hunspell_name: "en_US".into(),
            alphabet: "abcdefghijklmnopqrstuvwxyz".into(),
        },
        LanguageInfo {
            code: "es".into(),
            name: "Spanish".into(),
            native_name: "Español".into(),
            hunspell_name: "es_ES".into(),
            alphabet: "abcdefghijklmnñopqrstuvwxyz".into(),
        },
        LanguageInfo {
            code: "fr".into(),
            name: "French".into(),
            native_name: "Français".into(),
            hunspell_name: "fr_FR".into(),
            alphabet: "abcdefghijklmnopqrstuvwxyzàâæçéèêëïîôœùûüÿ".into(),
        },
        LanguageInfo {
            code: "de".into(),
            name: "German".into(),
            native_name: "Deutsch".into(),
            hunspell_name: "de_DE".into(),
            alphabet: "abcdefghijklmnopqrstuvwxyzäöüß".into(),
        },
        LanguageInfo {
            code: "pt".into(),
            name: "Portuguese".into(),
            native_name: "Português".into(),
            hunspell_name: "pt_BR".into(),
            alphabet: "abcdefghijklmnopqrstuvwxyzàáâãçéêíóôõú".into(),
        },
        LanguageInfo {
            code: "it".into(),
            name: "Italian".into(),
            native_name: "Italiano".into(),
            hunspell_name: "it_IT".into(),
            alphabet: "abcdefghijklmnopqrstuvwxyzàèéìíîòóùú".into(),
        },
    ]
}

// ── Spell check result ───────────────────────────────────────────────────────

/// Spell-check result for a single word
#[derive(Debug, Clone)]
pub struct SpellError {
    /// The misspelled word
    pub word: String,
    /// Byte offset in the original text
    pub offset: usize,
    /// Suggested corrections (up to 5)
    pub suggestions: Vec<String>,
    /// This word is the same as the one before it.
    ///
    /// A different fault with a different fix, and it has no suggestions
    /// because the correction is to delete it. Without this the row would read
    /// as a misspelling nothing could be suggested for, which is the one thing
    /// a spell checker should never say about a correctly spelled word.
    pub repeated: bool,
}

impl SpellError {
    /// The sentence read out when this error is reached.
    ///
    /// Says what is wrong before what to do about it, because the first is what
    /// decides whether somebody wants the second.
    pub fn spoken(&self) -> String {
        if self.repeated {
            return format!("{}, repeated word. Delete it.", self.word);
        }
        match self.suggestions.len() {
            0 => format!("{}, not in the dictionary, no suggestions", self.word),
            1 => format!(
                "{}, not in the dictionary. Suggestion: {}",
                self.word, self.suggestions[0]
            ),
            many => format!(
                "{}, not in the dictionary. {} suggestions, first is {}",
                self.word, many, self.suggestions[0]
            ),
        }
    }
}

#[cfg(windows)]
pub mod windows_speller;

/// Which checker answered.
///
/// Worth saying out loud rather than hiding behind the trait. The two are not
/// equivalent: one knows the words this person has taught Windows and the other
/// knows twelve thousand English words and nothing about anybody. Somebody
/// getting a wall of false positives deserves to be able to find out which they
/// have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Windows' own checker, shared with every other application here.
    Windows,
    /// A Hunspell dictionary found on this machine, through `spellbook`.
    Hunspell,
    /// The built-in word list, which is small.
    Builtin,
}

impl Source {
    /// How it reads in Settings, and in the sentence that explains a result.
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Windows => "Windows, so it knows the words you have added in Windows Settings",
            Self::Hunspell => "a dictionary installed on this computer",
            Self::Builtin => {
                "a small built-in word list, which will not know names or \
                 technical terms"
            }
        }
    }
}

/// One spell checker, whichever the platform provides.
///
/// Deliberately not `Send`. A COM interface pointer belongs to the apartment it
/// was created in, so the Windows implementation is made where it is used, and
/// the trait does not promise more than the strictest implementation can keep.
pub trait Speller {
    /// Every misspelling in a piece of text, with byte offsets into it.
    fn check(&self, text: &str) -> Vec<SpellError>;

    /// What this word might have been meant to be.
    fn suggest(&self, word: &str, max: usize) -> Vec<String>;

    /// Learn a word for good.
    ///
    /// On Windows this writes to the dictionary every application shares, so a
    /// word taught here is known in Word and Edge too. That is the point: being
    /// asked to teach the same surname to one application after another is
    /// WCAG 3.3.7 Redundant Entry, and it falls hardest on the people who type
    /// the most to get the least done.
    fn add_to_dictionary(&self, word: &str) -> Result<()>;

    /// Pass over this word for the rest of the session, and no longer.
    fn ignore(&self, word: &str);

    /// The language tag being checked against.
    fn language(&self) -> &str;

    /// Which checker this is, so the interface can say.
    fn source(&self) -> Source;
}

/// One language the spelling setting can offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageChoice {
    /// What to store: a BCP 47 tag from Windows, or a bare code otherwise.
    pub tag: String,
    /// What to show, in that language's own words where Windows knows them.
    pub name: String,
    /// Whether this machine can really check it.
    ///
    /// The whole reason this function exists. The old picker listed six
    /// languages whatever the machine had, so choosing French set a value that
    /// changed nothing, and the only way to find out was to write French and
    /// have every word of it called a mistake.
    pub available: bool,
}

/// The language this machine is set to, when something can check it.
///
/// The stored default was "en" for everybody, so anybody writing in anything
/// else had every word of it called a mistake until they found the setting.
/// This asks the machine instead, and only answers with a language something
/// here can actually check: offering one nothing has a dictionary for is the
/// same failure wearing a different label.
///
/// `None` when the machine's language is not one of them, and then the stored
/// default stands, because English checked is better than nothing checked.
pub fn language_of_this_machine() -> Option<String> {
    let wanted = system_language()?.to_ascii_lowercase();
    best_available_match(&wanted, &available_languages())
}

/// What this machine said when it was asked which languages it can check.
///
/// Two answers, because they mean opposite things and were the same value.
/// A machine with no spell checkers installed and a machine whose spell
/// checking could not be reached both produced an empty list, and nothing
/// was written down either way.
///
/// The cost of that was not a test: `default_language` reads no languages,
/// finds no match, and settles on English, so somebody writing in French got
/// every word of their mail marked wrong on a first run where the platform
/// call happened to fail. `Withdrawal` in `service::signed_mail` already
/// draws this line, between a certificate really not withdrawn and one
/// nobody could find out about, and for the same reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatThisMachineOffers {
    /// It answered. The list may still be empty: a machine really can have no
    /// spell checkers on it.
    TheseLanguages(Vec<(String, String)>),
    /// The question could not be put, with the reason in words.
    CouldNotAsk { reason: String },
}

/// What this machine offers, asked once, on whichever platform this is.
///
/// The one place the question is put, so a caller cannot accidentally ask it
/// twice and compare the answers with each other.
pub fn what_this_machine_offers() -> WhatThisMachineOffers {
    #[cfg(windows)]
    {
        windows_speller::WindowsSpeller::what_this_machine_offers()
    }
    #[cfg(not(windows))]
    {
        // Nothing was asked because there is nothing here to ask, which is an
        // answer rather than a failure: the built-in list is what this machine
        // offers, and `choices_from` already knows how to say so.
        WhatThisMachineOffers::TheseLanguages(Vec::new())
    }
}

/// Which language to check in, decided from what was asked rather than from
/// what was reachable at the time.
///
/// Pure, so the three arms can be tested without a spell checking feature on
/// the machine running the test. That is also what stops the test that used
/// to cover this failing about one full library run in five: it compared two
/// live platform calls against each other, and when one of them came back
/// empty the two disagreed and neither was wrong.
pub fn language_to_check_in(system: Option<&str>, offers: &WhatThisMachineOffers) -> String {
    let Some(system) = system else {
        // No language from the platform at all. English is a guess, and it is
        // the only guess available.
        return "en".to_string();
    };
    match offers {
        // Nobody could be asked what this machine checks, so nothing is known
        // about whether this language is checkable. Their own language is the
        // better guess: if something can check it the setting is right, and if
        // not they see their own language named rather than silently getting
        // English. Choosing English here is what marked a French user's every
        // word wrong.
        WhatThisMachineOffers::CouldNotAsk { reason } => {
            tracing::warn!(
                "this machine could not be asked which languages it checks ({reason}), \
                 so spelling is set to {system} without knowing whether anything can \
                 check it"
            );
            system.to_string()
        }
        // Asked and answered. English when this machine can check nothing of
        // theirs, which is a decision rather than an accident: English checked
        // is better than nothing checked.
        WhatThisMachineOffers::TheseLanguages(pairs) => {
            best_available_match(&system.to_ascii_lowercase(), &choices_from(pairs.clone()))
                .unwrap_or_else(|| "en".to_string())
        }
    }
}

/// Which of `choices` best answers `wanted`, if any.
///
/// The exact tag first: en-GB should not settle for en-US when both are
/// there, because the two disagree about half the words somebody types. Only
/// then the language family, because a family member something can check
/// beats naming nothing at all. Either way the match has to be marked
/// available, or this would offer a language exactly as uncheckable as the
/// one that was asked for, in different words.
fn best_available_match(wanted: &str, choices: &[LanguageChoice]) -> Option<String> {
    if let Some(exact) = choices
        .iter()
        .find(|c| c.available && c.tag.to_ascii_lowercase() == wanted)
    {
        return Some(exact.tag.clone());
    }
    let family = wanted.split('-').next().unwrap_or_default().to_string();
    if family.is_empty() {
        return None;
    }
    choices
        .iter()
        .find(|c| {
            c.available && c.tag.to_ascii_lowercase().split('-').next() == Some(family.as_str())
        })
        .map(|c| c.tag.clone())
}

/// The language tag Windows says this machine is set to.
///
/// Public because it answers a second question as well as the spelling one:
/// what language a document should ask to be read in. Use it rather than
/// [`language_of_this_machine`] for anything that is not about checking
/// spelling, because that one answers only with languages a dictionary is
/// installed for, which has nothing to do with how a document is pronounced.
#[cfg(target_os = "windows")]
pub fn system_language() -> Option<String> {
    const LOCALE_USER_DEFAULT: u32 = 0x0400;
    // LOCALE_SNAME: the full tag, "en-GB" rather than a number.
    const LOCALE_SNAME: u32 = 0x0000005c;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocaleInfoW(locale: u32, lctype: u32, data: *mut u16, size: i32) -> i32;
    }

    let mut buffer = [0u16; 85];
    let written = unsafe {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SNAME,
            buffer.as_mut_ptr(),
            buffer.len() as i32,
        )
    };
    if written <= 1 {
        return None;
    }
    // The count includes the terminator.
    Some(String::from_utf16_lossy(&buffer[..(written - 1) as usize]))
}

// Not reachable by any test this project runs, and not equivalent to
// anything either: this compiles only on a target `cfg(not(target_os =
// "windows"))`, and every check, build, and mutation-testing run this
// project has (`scripts/check.sh`, `.github/workflows/*.yml`) targets
// `windows-latest` alone, matching CLAUDE.md's "Windows-first" guardrail. On
// that build, this whole function is stripped before the compiler ever
// looks at its body, the same way it would be stripped from the shipped
// binary; a mutation inside it changes tokens nothing here compiles, so
// nothing here could ever run the changed and unchanged versions apart to
// tell them apart. A debug_assert! would not help: there is no runtime
// state to check, only a build target this codebase does not build for.
// Proving that means proving a negative about a target nobody here
// exercises, so it is written down instead of asserted. Testing this for
// real needs an actual Linux or macOS build of this crate, which is future
// port work, not a gap in today's coverage.
#[cfg(not(target_os = "windows"))]
pub fn system_language() -> Option<String> {
    std::env::var("LANG")
        .ok()
        .and_then(|value| value.split('.').next().map(|tag| tag.replace('_', "-")))
        .filter(|tag| !tag.is_empty() && tag != "C")
}

/// The languages this machine can actually check, and the ones it cannot.
///
/// Windows' real list first, in its own order. When Windows has none, the
/// built-in list is offered instead with everything but English marked
/// unavailable, because the fallback checker only ships an English word list
/// however many alphabets it knows.
pub fn available_languages() -> Vec<LanguageChoice> {
    #[cfg(windows)]
    {
        let windows_choices = windows_speller::WindowsSpeller::supported_languages()
            .into_iter()
            .map(|tag| {
                let name = windows_speller::display_name(&tag);
                (tag, name)
            })
            .collect();
        choices_from(windows_choices)
    }
    #[cfg(not(windows))]
    {
        choices_from(Vec::new())
    }
}

/// Turns Windows' own supported (tag, display name) pairs into the language
/// list, or falls back to the short built-in list when Windows offered
/// nothing.
///
/// Taking `windows_choices` as a plain argument, rather than asking Windows
/// itself, is what makes the two halves of this decision (which list wins,
/// and what the fallback list looks like) checkable without a real spell
/// checking feature installed on the machine running the test. It is also
/// what keeps this function itself free of any reference to
/// `windows_speller`, so it compiles on every platform rather than only the
/// one this project ships for.
fn choices_from(windows_choices: Vec<(String, String)>) -> Vec<LanguageChoice> {
    if !windows_choices.is_empty() {
        return windows_choices
            .into_iter()
            .map(|(tag, name)| LanguageChoice {
                tag,
                name,
                available: true,
            })
            .collect();
    }

    supported_languages()
        .into_iter()
        .map(|language| LanguageChoice {
            // English is the only one the built-in list holds. The others are
            // offered because a Hunspell dictionary for them may be installed,
            // and said to be unavailable when one is not, rather than quietly
            // failing every word.
            available: language.code == "en",
            tag: language.code,
            name: language.native_name,
        })
        .collect()
}

/// What to say before a message goes out with misspellings in it.
///
/// `None` when there is nothing worth stopping for, and then sending is not
/// interrupted at all. A confirmation that appears on every message is one
/// people learn to dismiss without reading, which costs them the one time it
/// mattered.
///
/// The words are named rather than counted. "Three words look misspelled" is a
/// number; "recieve, teh and mesage" is something somebody can decide about
/// without opening anything.
pub fn before_sending(errors: &[SpellError]) -> Option<String> {
    /// Past this many, listing them is worse than counting them.
    const NAMED: usize = 3;

    if errors.is_empty() {
        return None;
    }

    let mut words: Vec<&str> = errors.iter().map(|error| error.word.as_str()).collect();
    words.dedup();

    let listed = match words.len() {
        1 => format!("{} does not look like a word.", words[0]),
        2 => format!("{} and {} do not look like words.", words[0], words[1]),
        count if count <= NAMED => format!(
            "{} and {} do not look like words.",
            words[..count - 1].join(", "),
            words[count - 1]
        ),
        count => format!(
            "{} and {} others do not look like words.",
            words[..NAMED].join(", "),
            count - NAMED
        ),
    };
    Some(listed)
}

/// The best checker this machine has for a language.
///
/// Windows first, because it holds the words this person has already taught it.
/// The checker in this crate second, which needs no dictionary installed and
/// knows nobody's name.
///
/// Never fails. A machine with no spell checking at all still gets the built-in
/// list, and [`Speller::source`] says which arrived so the interface can be
/// honest about what it is offering.
pub fn for_language(tag: &str) -> Box<dyn Speller> {
    #[cfg(windows)]
    {
        if let Some(windows) = windows_speller::WindowsSpeller::for_language(tag) {
            return Box::new(windows);
        }
        // A bare language where Windows wants a region. Everybody who set this
        // before the picker offered real tags has "en" stored, and Windows
        // lists "en-GB", "en-US" and a dozen more; taking the first it offers
        // for that language is what stops those people silently dropping to the
        // built-in word list on upgrade. Windows lists them in its own
        // preference order, so the first is the one this machine leans towards.
        if let Some(regional) =
            find_regional_variant(tag, &windows_speller::WindowsSpeller::supported_languages())
            && let Some(windows) = windows_speller::WindowsSpeller::for_language(&regional)
        {
            tracing::info!("Spell checking {} as {}", tag, regional);
            return Box::new(windows);
        }
        tracing::info!(
            "Windows has no spell checker for {}, using the built-in one",
            tag
        );
    }
    Box::new(SpellChecker::with_language(short_code(tag)))
}

/// The language half of a tag: `en-GB` is English.
///
/// Windows wants a full BCP 47 tag and the checker in this crate is keyed by
/// language alone, so the tag is narrowed on the way to the fallback rather
/// than the fallback being asked a question it cannot answer.
fn short_code(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

/// The first of Windows' supported tags that shares `tag`'s language, if any.
///
/// Taken as a plain list rather than asking Windows directly, so the rule for
/// which regional variant wins is checkable without a real spell checking
/// feature installed on the machine running the test.
#[cfg(windows)]
fn find_regional_variant(tag: &str, supported: &[String]) -> Option<String> {
    supported
        .iter()
        .find(|candidate| short_code(candidate) == short_code(tag))
        .cloned()
}

impl Speller for SpellChecker {
    fn check(&self, text: &str) -> Vec<SpellError> {
        SpellChecker::check_text(self, text)
    }

    fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        SpellChecker::suggest(self, word, max)
    }

    fn add_to_dictionary(&self, _word: &str) -> Result<()> {
        // Deliberately refused rather than quietly kept in memory. `add_word`
        // takes `&mut self` and lasts until the process ends, so accepting the
        // word here would look like learning it and be forgotten by the next
        // message. Saying so is the honest answer, and the fix is Windows
        // having a dictionary for this language.
        Err(Error::Other(
            "Words can only be learned when Windows is doing the spell \
             checking. This computer is using the built-in word list."
                .into(),
        ))
    }

    fn ignore(&self, _word: &str) {
        // Nothing to do: the built-in checker has no session state, and the
        // caller keeps its own ignore list for exactly this reason.
    }

    fn language(&self) -> &str {
        SpellChecker::language(self)
    }

    fn source(&self) -> Source {
        if self.has_hunspell() {
            Source::Hunspell
        } else {
            Source::Builtin
        }
    }
}

// ── Backend enum ─────────────────────────────────────────────────────────────

/// The active spell-checking backend.
enum Backend {
    /// Hunspell-compatible dictionary via the `spellbook` crate.
    /// Boxed because a loaded dictionary dwarfs the built-in word list.
    Spellbook(Box<spellbook::Dictionary>),
    /// Lightweight built-in word list (English only).
    Builtin(HashSet<String>),
}

// ── SpellChecker ─────────────────────────────────────────────────────────────

/// Spell checker with Hunspell support and i18n-aware configuration.
pub struct SpellChecker {
    backend: Backend,
    /// Custom words added by the user (per session)
    custom_words: HashSet<String>,
    /// Active language code
    language: String,
    /// Character alphabet for generating suggestions (fallback)
    alphabet: String,
    /// Search paths for Hunspell dictionary files
    dict_search_paths: Vec<PathBuf>,
}

impl SpellChecker {
    /// Create a new spell checker with the default English dictionary.
    pub fn new() -> Self {
        Self::with_language("en")
    }

    /// Create a spell checker for a specific language.
    ///
    /// Searches standard locations for Hunspell `.aff` + `.dic` files.
    /// Falls back to the built-in English word list if no files are found.
    pub fn with_language(lang_code: &str) -> Self {
        let lang_info = supported_languages()
            .into_iter()
            .find(|l| l.code == lang_code);

        let alphabet = lang_info
            .as_ref()
            .map(|l| l.alphabet.clone())
            .unwrap_or_else(|| "abcdefghijklmnopqrstuvwxyz".to_string());

        let search_paths = default_dict_search_paths();

        // Attempt to load Hunspell dictionary via spellbook
        let hunspell_name = lang_info
            .as_ref()
            .map(|l| l.hunspell_name.clone())
            .unwrap_or_else(|| format!("{lang_code}_{}", lang_code.to_uppercase()));

        let backend = try_load_spellbook(&hunspell_name, &search_paths).unwrap_or_else(|| {
            // Fallback: built-in word list (English only)
            let mut dict = HashSet::with_capacity(15_000);
            if lang_code == "en" {
                for word in CORE_ENGLISH_WORDS.split('\n') {
                    let w = word.trim().to_lowercase();
                    if !w.is_empty() {
                        dict.insert(w);
                    }
                }
            }
            Backend::Builtin(dict)
        });

        Self {
            backend,
            custom_words: HashSet::new(),
            language: lang_code.to_string(),
            alphabet,
            dict_search_paths: search_paths,
        }
    }

    /// Create a spell checker from Hunspell `.aff` and `.dic` file contents.
    ///
    /// This is the preferred API when you already have the dictionary data
    /// (e.g. bundled in the application or fetched from the network).
    pub fn from_hunspell_data(
        lang_code: &str,
        aff_content: &str,
        dic_content: &str,
    ) -> std::result::Result<Self, String> {
        // spellbook::Dictionary requires 'static lifetime, so we leak the strings.
        // This is acceptable because dictionaries live for the application lifetime.
        let aff: &'static str = Box::leak(aff_content.to_string().into_boxed_str());
        let dic: &'static str = Box::leak(dic_content.to_string().into_boxed_str());

        let dict = spellbook::Dictionary::new(aff, dic)
            .map_err(|e| format!("Failed to parse Hunspell dictionary: {}", e))?;

        let alphabet = supported_languages()
            .into_iter()
            .find(|l| l.code == lang_code)
            .map(|l| l.alphabet.clone())
            .unwrap_or_else(|| "abcdefghijklmnopqrstuvwxyz".to_string());

        Ok(Self {
            backend: Backend::Spellbook(Box::new(dict)),
            custom_words: HashSet::new(),
            language: lang_code.to_string(),
            alphabet,
            dict_search_paths: default_dict_search_paths(),
        })
    }

    /// Whether this checker is backed by a real Hunspell dictionary.
    pub fn has_hunspell(&self) -> bool {
        matches!(self.backend, Backend::Spellbook(_))
    }

    /// Get the active language code.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Load additional words from a plain text file (one word per line).
    /// Only affects the built-in backend; Hunspell dictionaries use `.dic` files.
    pub fn load_dictionary_file(&mut self, path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let mut count = 0;
        if let Backend::Builtin(ref mut dict) = self.backend {
            for line in content.lines() {
                let w = line.trim().to_lowercase();
                if !w.is_empty() {
                    dict.insert(w);
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Add a word to the custom dictionary for this session.
    pub fn add_word(&mut self, word: &str) {
        self.custom_words.insert(word.to_lowercase());
    }

    /// Check if a word is correctly spelled.
    pub fn is_correct(&self, word: &str) -> bool {
        if is_number_or_special(word) {
            return true;
        }
        if self.custom_words.contains(&word.to_lowercase()) {
            return true;
        }
        match &self.backend {
            Backend::Spellbook(dict) => dict.check(word),
            Backend::Builtin(set) => set.contains(&word.to_lowercase()),
        }
    }

    /// Check a block of text and return all misspelled words with offsets.
    pub fn check_text(&self, text: &str) -> Vec<SpellError> {
        let mut errors = Vec::new();
        let mut offset = 0;

        // `is_whitespace()` already includes '\n' and '\r' as Unicode
        // White_Space, so the two explicit checks after it never change
        // which characters this splits on; they stay so the intent reads
        // without having to know that fact about Unicode. Because of that,
        // no input can tell `||` apart from `&&` between the second and
        // third arms here: whichever one is written, this splits on
        // whitespace exactly.
        for segment in text.split(|c: char| c.is_whitespace() || c == '\n' || c == '\r') {
            let word = segment.trim_matches(|c: char| !c.is_alphanumeric());
            if word.len() >= 2 && !self.is_correct(word) {
                let word_offset = offset + segment.find(word).unwrap_or(0);
                errors.push(SpellError {
                    word: word.to_string(),
                    offset: word_offset,
                    suggestions: self.suggest(word, 5),
                    // The built-in checker looks at one word at a time and has
                    // no idea what came before it. Repeated words are Windows'
                    // to find.
                    repeated: false,
                });
            }
            offset += segment.len() + 1;
        }

        errors
    }

    /// Generate spelling suggestions.
    ///
    /// Uses spellbook's built-in Hunspell suggestion algorithm when available,
    /// otherwise falls back to edit-distance-1 candidates.
    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        match &self.backend {
            Backend::Spellbook(dict) => {
                let mut suggestions = Vec::new();
                dict.suggest(word, &mut suggestions);
                suggestions.truncate(max);
                suggestions
            }
            Backend::Builtin(set) => {
                let lower = word.to_lowercase();
                let edits = generate_edits(&lower, &self.alphabet);
                let mut candidates: Vec<String> =
                    edits.into_iter().filter(|e| set.contains(e)).collect();
                candidates.sort();
                candidates.dedup();
                candidates.truncate(max);
                candidates
            }
        }
    }

    /// Dictionary size (for diagnostics).
    pub fn word_count(&self) -> usize {
        let base = match &self.backend {
            Backend::Spellbook(_) => 100_000, // estimate; exact count unavailable
            Backend::Builtin(set) => set.len(),
        };
        base + self.custom_words.len()
    }

    /// Get the dictionary search paths.
    pub fn dict_search_paths(&self) -> &[PathBuf] {
        &self.dict_search_paths
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Hunspell loading helpers ─────────────────────────────────────────────────

/// Standard search paths for Hunspell dictionary files.
fn default_dict_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Dictionaries somebody has added by hand, in the application data folder.
    if let Ok(app) = crate::common::paths::AppPaths::resolve() {
        paths.push(app.root().join("dictionaries"));
    }

    // Platform-standard Hunspell locations
    if cfg!(target_os = "linux") {
        paths.push(PathBuf::from("/usr/share/hunspell"));
        paths.push(PathBuf::from("/usr/share/myspell"));
        paths.push(PathBuf::from("/usr/share/myspell/dicts"));
    } else if cfg!(target_os = "macos") {
        paths.push(PathBuf::from("/Library/Spelling"));
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Spelling"));
        }
    } else if cfg!(target_os = "windows")
        && let Some(program_data) = std::env::var_os("ProgramData")
    {
        paths.push(PathBuf::from(program_data).join("hunspell"));
    }

    paths
}

/// Try to load a Hunspell dictionary from one of the search paths.
fn try_load_spellbook(hunspell_name: &str, search_paths: &[PathBuf]) -> Option<Backend> {
    for dir in search_paths {
        let aff_path = dir.join(format!("{}.aff", hunspell_name));
        let dic_path = dir.join(format!("{}.dic", hunspell_name));

        if aff_path.exists()
            && dic_path.exists()
            && let Ok(aff) = std::fs::read_to_string(&aff_path)
            && let Ok(dic) = std::fs::read_to_string(&dic_path)
        {
            let aff_static: &'static str = Box::leak(aff.into_boxed_str());
            let dic_static: &'static str = Box::leak(dic.into_boxed_str());
            if let Ok(dict) = spellbook::Dictionary::new(aff_static, dic_static) {
                tracing::info!(
                    "Loaded Hunspell dictionary '{}' from {}",
                    hunspell_name,
                    dir.display()
                );
                return Some(Backend::Spellbook(Box::new(dict)));
            }
        }
    }
    None
}

// ── Token helpers ────────────────────────────────────────────────────────────

/// Check if a token is a number, email, URL, or other non-word token.
fn is_number_or_special(s: &str) -> bool {
    if s.is_empty() || s.len() <= 1 {
        return true;
    }
    let lower = s.to_lowercase();
    if lower
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == '-')
    {
        return true;
    }
    if lower.contains('@') && lower.contains('.') {
        return true;
    }
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.") {
        return true;
    }
    false
}

/// Generate all strings that are one edit away from `word`.
fn generate_edits(word: &str, alphabet: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut edits = Vec::new();

    for i in 0..n {
        let mut s = String::with_capacity(n - 1);
        for (j, &c) in chars.iter().enumerate() {
            if j != i {
                s.push(c);
            }
        }
        edits.push(s);
    }
    for i in 0..n.saturating_sub(1) {
        let mut s: Vec<char> = chars.clone();
        s.swap(i, i + 1);
        edits.push(s.into_iter().collect());
    }
    for i in 0..n {
        for c in alphabet.chars() {
            if c != chars[i] {
                let mut s = chars.clone();
                s[i] = c;
                edits.push(s.into_iter().collect());
            }
        }
    }
    for i in 0..=n {
        for c in alphabet.chars() {
            let mut s = String::with_capacity(n + 1);
            for (j, &ch) in chars.iter().enumerate() {
                if j == i {
                    s.push(c);
                }
                s.push(ch);
            }
            if i == n {
                s.push(c);
            }
            edits.push(s);
        }
    }

    edits
}

/// Core English word list (~12K words). Used as fallback when no Hunspell
/// dictionary is installed.
const CORE_ENGLISH_WORDS: &str = include_str!("../../../data/dictionary_en.txt");

// ── I18n / Localization Infrastructure ──────────────────────────────────────

/// Locale descriptor for UI string localization.
#[derive(Debug, Clone)]
pub struct Locale {
    pub language_code: String,
    pub country_code: Option<String>,
    pub display_name: String,
    pub direction: TextDirection,
}

/// Text direction for layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

impl Locale {
    pub fn from_code(code: &str) -> Self {
        let parts: Vec<&str> = code.split('-').collect();
        let lang = parts[0].to_lowercase();
        let country = parts.get(1).map(|c| c.to_uppercase());
        let direction = match lang.as_str() {
            "ar" | "he" | "fa" | "ur" => TextDirection::RightToLeft,
            _ => TextDirection::LeftToRight,
        };
        let display = match (lang.as_str(), country.as_deref()) {
            ("en", Some("US")) => "English (US)".to_string(),
            ("en", Some("GB")) => "English (UK)".to_string(),
            ("en", _) => "English".to_string(),
            ("es", _) => "Spanish".to_string(),
            ("fr", _) => "French".to_string(),
            ("de", _) => "German".to_string(),
            ("pt", Some("BR")) => "Portuguese (Brazil)".to_string(),
            ("pt", _) => "Portuguese".to_string(),
            ("it", _) => "Italian".to_string(),
            _ => code.to_string(),
        };
        Self {
            language_code: lang,
            country_code: country,
            display_name: display,
            direction,
        }
    }
}

/// The active registry, for code that has no `I18n` to hand.
///
/// The document converter runs deep inside a rendering pass and threading a
/// registry down to it would mean a parameter on every function between here
/// and there for three strings. It reads the locale that was last set and
/// falls back to English, which is what the registry does anyway.
static DOCUMENT_LOCALE: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Set the locale used for text inside rendered documents.
pub fn set_document_locale(code: &str) {
    if let Ok(mut locale) = DOCUMENT_LOCALE.write() {
        *locale = Some(code.to_string());
    }
}

/// Translate one of the terms the document converter puts into its output.
///
/// Falls back to the key's English default rather than to the key itself, so a
/// missing translation reads as "Image" and never as "document.image".
pub fn translate_document_term(key: &str) -> String {
    let code = DOCUMENT_LOCALE
        .read()
        .ok()
        .and_then(|locale| locale.clone())
        .unwrap_or_else(|| "en".to_string());
    I18n::with_locale(&code).t(key)
}

/// Internationalization (i18n) registry for UI string translations.
pub struct I18n {
    active_locale: Locale,
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new() -> Self {
        let mut i18n = Self {
            active_locale: Locale::from_code("en"),
            translations: HashMap::new(),
        };
        i18n.register_english_defaults();
        i18n
    }

    pub fn with_locale(code: &str) -> Self {
        let mut i18n = Self {
            active_locale: Locale::from_code(code),
            translations: HashMap::new(),
        };
        i18n.register_english_defaults();
        i18n
    }

    fn register_english_defaults(&mut self) {
        let mut en = HashMap::new();
        for (k, v) in [
            // Read aloud inside a rendered message body. Keyed rather than
            // written as literals so a screen reader user working in another
            // language does not hear these three words in English.
            ("document.image", "Image"),
            ("document.figure", "Figure"),
            ("document.table_marker", "Table marker"),
            ("menu.file", "File"),
            ("menu.edit", "Edit"),
            ("menu.view", "View"),
            ("menu.message", "Message"),
            ("menu.tools", "Tools"),
            ("menu.help", "Help"),
            ("action.send", "Send"),
            ("action.save_draft", "Save Draft"),
            ("action.cancel", "Cancel"),
            ("action.ok", "OK"),
            ("action.delete", "Delete"),
            ("action.reply", "Reply"),
            ("action.reply_all", "Reply All"),
            ("action.forward", "Forward"),
            ("action.search", "Search"),
            ("status.ready", "Ready"),
            ("status.checking_mail", "Checking for new mail..."),
            ("status.sending", "Sending..."),
            ("status.offline", "Offline mode"),
            ("status.online", "Online"),
            ("status.connected", "Connected"),
            ("status.disconnected", "Disconnected"),
            ("compose.to", "To:"),
            ("compose.cc", "CC:"),
            ("compose.bcc", "BCC:"),
            ("compose.subject", "Subject:"),
            ("compose.from", "From:"),
            ("spellcheck.no_errors", "No spelling errors found"),
            ("spellcheck.errors_found", "Spelling errors found"),
            ("spellcheck.add_to_dictionary", "Add to Dictionary"),
            ("spellcheck.ignore", "Ignore"),
            ("spellcheck.ignore_all", "Ignore All"),
            ("settings.title", "Settings"),
            ("settings.general", "General"),
            ("settings.compose", "Compose"),
            ("settings.reading", "Reading"),
            ("settings.language", "Language"),
            ("settings.advanced", "Advanced"),
        ] {
            en.insert(k.into(), v.into());
        }
        self.translations.insert("en".into(), en);
    }

    pub fn load_translations(&mut self, lang_code: &str, strings: HashMap<String, String>) {
        self.translations.insert(lang_code.to_string(), strings);
    }

    pub fn load_translations_file(
        &mut self,
        lang_code: &str,
        path: &Path,
    ) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let count = map.len();
        self.translations.insert(lang_code.to_string(), map);
        Ok(count)
    }

    pub fn set_locale(&mut self, code: &str) {
        self.active_locale = Locale::from_code(code);
    }
    pub fn locale(&self) -> &Locale {
        &self.active_locale
    }

    pub fn t(&self, string_id: &str) -> String {
        if let Some(table) = self.translations.get(&self.active_locale.language_code)
            && let Some(s) = table.get(string_id)
        {
            return s.clone();
        }
        if let Some(en) = self.translations.get("en")
            && let Some(s) = en.get(string_id)
        {
            return s.clone();
        }
        string_id.to_string()
    }

    pub fn tf(&self, string_id: &str, args: &[&str]) -> String {
        let template = self.t(string_id);
        let mut result = template;
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn test_the_machines_language_is_one_something_can_check() {
        // Offering a language nothing has a dictionary for is the same failure
        // as defaulting to the wrong one: every word comes back a mistake.
        if let Some(chosen) = super::language_of_this_machine() {
            let choices = super::available_languages();
            let found = choices
                .iter()
                .find(|c| c.tag == chosen)
                .unwrap_or_else(|| panic!("{chosen} is not in the list at all"));
            assert!(found.available, "{chosen} has nothing that can check it");
        }
    }

    #[test]
    fn test_no_answer_is_a_real_answer() {
        // A machine set to a language nothing here checks keeps the stored
        // default, because English checked beats nothing checked.
        let answer = super::language_of_this_machine();

        assert!(answer.is_none() || !answer.unwrap().trim().is_empty());
    }

    fn wrong(word: &str, suggestions: &[&str]) -> SpellError {
        SpellError {
            word: word.to_string(),
            offset: 0,
            suggestions: suggestions.iter().map(|s| s.to_string()).collect(),
            repeated: false,
        }
    }

    #[test]
    fn test_a_clean_message_is_not_interrupted() {
        // A confirmation that appears every time is one people learn to
        // dismiss without reading, which costs them the time it mattered.
        assert_eq!(before_sending(&[]), None);
    }

    #[test]
    fn test_one_word_is_named_rather_than_counted() {
        // "1 word looks misspelled" makes somebody open a dialog to find out
        // which. Naming it lets them decide without opening anything.
        let said = before_sending(&[wrong("recieve", &["receive"])]).expect("something to say");

        assert!(said.contains("recieve"), "{said}");
    }

    #[test]
    fn test_a_few_words_are_all_named() {
        let said = before_sending(&[
            wrong("recieve", &[]),
            wrong("teh", &[]),
            wrong("mesage", &[]),
        ])
        .expect("something to say");

        for word in ["recieve", "teh", "mesage"] {
            assert!(said.contains(word), "{word} missing from {said}");
        }
    }

    #[test]
    fn test_exactly_three_words_are_all_named_without_an_and_zero_others() {
        // Three is NAMED's own limit. It is the one count that could fall
        // through to the "and N others" arm with N equal to zero, which the
        // test above would not notice: "and 0 others" still contains every
        // word's own text, it just reads as nonsense rather than a mistake.
        let said = before_sending(&[
            wrong("recieve", &[]),
            wrong("teh", &[]),
            wrong("mesage", &[]),
        ])
        .expect("something to say");

        assert!(!said.contains("others"), "{said}");
        assert!(said.contains("recieve, teh and mesage"), "{said}");
    }

    #[test]
    fn test_a_lot_of_words_are_counted_rather_than_recited() {
        // Past a few, listing them is a paragraph read aloud before somebody
        // can answer a yes or no question.
        let errors: Vec<SpellError> = (0..20).map(|n| wrong(&format!("wrng{n}"), &[])).collect();

        let said = before_sending(&errors).expect("something to say");

        assert!(said.contains("wrng0"), "{said}");
        assert!(said.contains("17 others"), "{said}");
        assert!(!said.contains("wrng19"), "it recited the lot: {said}");
    }

    #[test]
    fn test_the_same_word_twice_is_said_once() {
        // A word misspelled consistently is one mistake, not five.
        let said = before_sending(&[wrong("recieve", &[]), wrong("recieve", &[])])
            .expect("something to say");

        assert_eq!(said.matches("recieve").count(), 1, "{said}");
    }

    #[test]
    fn test_a_repeated_word_is_not_called_a_misspelling() {
        // "the the" is two correctly spelled words and one mistake, and being
        // told there are no suggestions for "the" is the one thing a spell
        // checker must never say about a word that is spelled right.
        let repeated = SpellError {
            word: "the".to_string(),
            offset: 4,
            suggestions: Vec::new(),
            repeated: true,
        };

        let spoken = repeated.spoken();

        assert!(spoken.contains("repeated"), "{spoken}");
        assert!(!spoken.contains("dictionary"), "{spoken}");
    }

    #[test]
    fn test_a_misspelling_says_what_is_wrong_before_what_to_do() {
        // The first decides whether somebody wants the second.
        let spoken = wrong("recieve", &["receive", "recipe"]).spoken();

        assert!(
            spoken.starts_with("recieve, not in the dictionary"),
            "{spoken}"
        );
        assert!(spoken.contains("receive"), "{spoken}");
    }

    #[test]
    fn test_a_word_with_no_suggestions_says_so_rather_than_trailing_off() {
        let spoken = wrong("qwertyuiop", &[]).spoken();

        assert!(spoken.contains("no suggestions"), "{spoken}");
    }
    use super::*;

    #[test]
    fn test_new_creates_english_checker() {
        let checker = SpellChecker::new();
        assert_eq!(checker.language(), "en");
        assert!(checker.word_count() > 1000);
    }

    #[test]
    fn test_builtin_correct_word() {
        let checker = SpellChecker::new();
        assert!(checker.is_correct("the"));
        assert!(checker.is_correct("email"));
    }

    #[test]
    fn test_custom_word() {
        let mut checker = SpellChecker::new();
        assert!(!checker.is_correct("wixen"));
        checker.add_word("wixen");
        assert!(checker.is_correct("wixen"));
        assert!(checker.is_correct("Wixen"));
    }

    #[test]
    fn test_numbers_and_special() {
        let checker = SpellChecker::new();
        assert!(checker.is_correct("123"));
        assert!(checker.is_correct("test@example.com"));
        assert!(checker.is_correct("https://example.com"));
        assert!(checker.is_correct("3.14"));
    }

    #[test]
    fn test_with_language() {
        let checker = SpellChecker::with_language("en");
        assert_eq!(checker.language(), "en");

        let es = SpellChecker::with_language("es");
        assert_eq!(es.language(), "es");
        assert!(es.alphabet.contains('ñ'));
    }

    #[test]
    fn test_supported_languages() {
        let langs = supported_languages();
        assert!(langs.len() >= 6);
        assert!(langs.iter().any(|l| l.code == "en"));
        for l in &langs {
            assert!(!l.hunspell_name.is_empty());
        }
    }

    #[test]
    fn test_edit_distance_generates_candidates() {
        let edits = generate_edits("cat", "abcdefghijklmnopqrstuvwxyz");
        assert!(edits.contains(&"at".to_string()));
        assert!(edits.contains(&"act".to_string()));
        assert!(edits.contains(&"bat".to_string()));
        assert!(edits.contains(&"cats".to_string()));
    }

    #[test]
    fn test_i18n_english_defaults() {
        let i18n = I18n::new();
        assert_eq!(i18n.t("action.send"), "Send");
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_i18n_fallback() {
        let mut i18n = I18n::with_locale("es");
        let mut es = HashMap::new();
        es.insert("action.send".to_string(), "Enviar".to_string());
        i18n.load_translations("es", es);
        assert_eq!(i18n.t("action.send"), "Enviar");
        assert_eq!(i18n.t("action.cancel"), "Cancel");
    }

    #[test]
    fn test_a_misspelling_is_reported_where_it_actually_is() {
        // The offset is where the editor underlines, and where somebody
        // working by ear is sent when they ask for the next mistake. One
        // character out and they arrive at a word that is spelled correctly,
        // with nothing to say why.
        let checker = SpellChecker::new();
        for text in [
            "the quick brwn fox jumps over the lazzy dog",
            "Hello, wrrld! Is teh dog here?",
            "(wrrld) and [teh] again",
            "first line has wrrld\nsecond line has teh\r\nthird is fine",
            "  leading spaces then wrrld",
        ] {
            for error in checker.check_text(text) {
                let end = error.offset + error.word.len();
                assert!(
                    end <= text.len(),
                    "{} runs past the end of {text:?}",
                    error.word
                );
                assert_eq!(
                    &text[error.offset..end],
                    error.word,
                    "{:?} was reported at {} of {text:?}, where the text says {:?}",
                    error.word,
                    error.offset,
                    &text[error.offset..end]
                );
            }
        }
    }

    #[test]
    fn test_the_misspellings_in_a_line_are_all_found() {
        // The offsets being right is worth nothing if the words are missed.
        let checker = SpellChecker::new();

        let found: Vec<String> = checker
            .check_text("the quick brwn fox jumps over the lazzy dog")
            .into_iter()
            .map(|e| e.word)
            .collect();

        assert!(found.contains(&"brwn".to_string()), "{found:?}");
        assert!(found.contains(&"lazzy".to_string()), "{found:?}");
        assert!(!found.contains(&"quick".to_string()), "{found:?}");
    }

    #[test]
    fn test_every_language_is_named_the_way_somebody_would_say_it() {
        // This is the name read out in the settings list. Without it the row
        // says "pt-BR", which is not something somebody chooses from by ear.
        // Nine of these were one arm each and none of them had a test.
        for (code, expected) in [
            ("en-US", "English (US)"),
            ("en-GB", "English (UK)"),
            ("en", "English"),
            ("en-AU", "English"),
            ("es-ES", "Spanish"),
            ("fr-FR", "French"),
            ("de-DE", "German"),
            ("pt-BR", "Portuguese (Brazil)"),
            ("pt-PT", "Portuguese"),
            ("it-IT", "Italian"),
            // Nothing known about it, so the code itself is the honest answer.
            ("nl-NL", "nl-NL"),
        ] {
            assert_eq!(Locale::from_code(code).display_name, expected, "for {code}");
        }
    }

    #[test]
    fn test_what_is_not_a_word_worth_checking() {
        // Marking any of these turns a message full of ordinary content into a
        // message full of mistakes, and a spell checker that cries wolf on
        // every address and every price is one somebody turns off.
        let checker = SpellChecker::new();

        for token in [
            "",
            "x",
            "123",
            "3.14",
            "1,000",
            "2026-08-02",
            "test@example.com",
            "http://example.com",
            "https://example.com",
            "www.example.com",
        ] {
            assert!(checker.is_correct(token), "{token:?} was called a mistake");
        }

        // And something that really is one is still found.
        assert!(!checker.is_correct("wrrld"));
    }

    #[test]
    fn test_locale_rtl_detection() {
        let en = Locale::from_code("en-US");
        assert_eq!(en.direction, TextDirection::LeftToRight);
        let ar = Locale::from_code("ar");
        assert_eq!(ar.direction, TextDirection::RightToLeft);
    }

    // ── Source::describe ─────────────────────────────────────────────────

    #[test]
    fn test_every_source_describes_itself_honestly() {
        assert!(Source::Windows.describe().contains("Windows"));
        assert!(Source::Hunspell.describe().contains("dictionary"));
        assert!(Source::Builtin.describe().contains("built-in"));
    }

    // ── Language matching ────────────────────────────────────────────────

    #[test]
    fn test_the_exact_tag_is_preferred_over_its_family() {
        let choices = vec![
            LanguageChoice {
                tag: "en-us".into(),
                name: "English (US)".into(),
                available: true,
            },
            LanguageChoice {
                tag: "en-gb".into(),
                name: "English (UK)".into(),
                available: true,
            },
        ];

        assert_eq!(
            best_available_match("en-gb", &choices),
            Some("en-gb".to_string())
        );
    }

    #[test]
    fn test_an_unavailable_exact_match_does_not_win_over_an_available_family_member() {
        let choices = vec![
            LanguageChoice {
                tag: "en-gb".into(),
                name: "English (UK)".into(),
                available: false,
            },
            LanguageChoice {
                tag: "en-us".into(),
                name: "English (US)".into(),
                available: true,
            },
        ];

        assert_eq!(
            best_available_match("en-gb", &choices),
            Some("en-us".to_string())
        );
    }

    #[test]
    fn test_nothing_answers_when_no_family_member_is_available() {
        let choices = vec![LanguageChoice {
            tag: "fr-fr".into(),
            name: "French".into(),
            available: false,
        }];

        assert_eq!(best_available_match("fr-fr", &choices), None);
    }

    #[test]
    fn test_the_machines_language_is_exactly_the_best_match_for_its_real_locale() {
        // Ties language_of_this_machine to the pure matching logic above,
        // computed independently the same way, so a version that ignores
        // the machine entirely (always None, or always the stored default)
        // cannot pass unnoticed. This assumes the machine running the tests
        // has a resolvable locale, true of every dev machine and CI runner
        // this project actually uses.
        let expected = system_language()
            .map(|tag| tag.to_ascii_lowercase())
            .and_then(|wanted| best_available_match(&wanted, &available_languages()));

        assert_eq!(language_of_this_machine(), expected);
    }

    #[cfg(windows)]
    #[test]
    fn test_system_language_reports_something_real_on_this_machine() {
        let language = system_language().expect("every real Windows install has a default locale");

        assert!(!language.trim().is_empty());
        assert!(
            !language.contains('\u{0}'),
            "trailing NUL was not trimmed: {language:?}"
        );
    }

    #[test]
    fn test_an_empty_windows_list_falls_back_to_the_built_in_six() {
        let choices = choices_from(Vec::new());

        assert_eq!(choices.len(), supported_languages().len(), "{choices:?}");
    }

    #[test]
    fn test_only_english_is_offered_as_available_in_the_built_in_fallback() {
        let choices = choices_from(Vec::new());

        let english = choices
            .iter()
            .find(|c| c.tag == "en")
            .expect("english is always in the built-in list");
        assert!(
            english.available,
            "English was marked unavailable in the fallback list"
        );
        assert!(
            choices
                .iter()
                .filter(|c| c.tag != "en")
                .all(|c| !c.available),
            "{choices:?}"
        );
    }

    #[test]
    fn test_a_real_windows_list_is_used_instead_of_the_fallback() {
        let choices = choices_from(vec![(
            "en-US".to_string(),
            "English (United States)".to_string(),
        )]);

        assert_eq!(choices.len(), 1, "{choices:?}");
        assert_eq!(choices[0].tag, "en-US");
        assert_eq!(choices[0].name, "English (United States)");
        assert!(
            choices[0].available,
            "a tag Windows offered was marked unavailable"
        );
    }

    #[test]
    fn test_available_languages_is_never_empty() {
        // available_languages() itself, rather than choices_from: it is the
        // whole function the mutation report names, and the two tests above
        // only prove choices_from can't be empty for either input it is
        // given, not that available_languages ever calls it.
        assert!(!available_languages().is_empty());
    }

    #[test]
    fn test_short_code_is_the_language_half_of_a_tag() {
        assert_eq!(short_code("en-GB"), "en");
        assert_eq!(short_code("pt_BR"), "pt");
        assert_eq!(short_code("de"), "de");
    }

    #[cfg(windows)]
    #[test]
    fn test_a_bare_language_matches_the_first_regional_variant_windows_offers() {
        let supported = vec![
            "fr-CA".to_string(),
            "en-GB".to_string(),
            "en-US".to_string(),
        ];

        assert_eq!(
            find_regional_variant("en", &supported),
            Some("en-GB".to_string())
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_a_language_windows_does_not_support_at_all_matches_nothing() {
        let supported = vec!["fr-CA".to_string(), "en-GB".to_string()];

        assert_eq!(find_regional_variant("de", &supported), None);
    }

    // ── The Speller trait, called through a trait object ────────────────
    //
    // `checker.suggest(...)` resolves to SpellChecker's own inherent method,
    // which Rust prefers over a trait method of the same name. Production
    // code reaches the trait method through `Box<dyn Speller>`, so these go
    // through `&dyn Speller` too, or the trait impl's own body is never run.

    #[test]
    fn test_the_speller_trait_object_suggests_the_same_as_the_inherent_method() {
        let checker = SpellChecker::new();
        let speller: &dyn Speller = &checker;

        let suggestions = speller.suggest("recieve", 5);

        assert!(
            suggestions.iter().any(|s| s == "receive"),
            "{suggestions:?}"
        );
    }

    #[test]
    fn test_the_builtin_checker_refuses_to_learn_a_word_through_the_trait() {
        // add_to_dictionary takes &mut self and lasts until the process
        // ends, so accepting this quietly would look like learning it and
        // be forgotten by the next message.
        let checker = SpellChecker::new();
        let speller: &dyn Speller = &checker;

        let result = speller.add_to_dictionary("wixenite");

        assert!(
            result.is_err(),
            "the built-in checker claimed to learn a word"
        );
    }

    #[test]
    fn test_the_speller_trait_object_reports_its_real_language() {
        let checker = SpellChecker::with_language("es");
        let speller: &dyn Speller = &checker;

        assert_eq!(speller.language(), "es");
    }

    // ── SpellChecker's own methods ───────────────────────────────────────

    #[test]
    fn test_from_hunspell_data_checks_against_the_data_it_was_given() {
        // Default::default() would fall back to the built-in list, which
        // knows nothing about a dictionary built from scratch in this test.
        let checker = SpellChecker::from_hunspell_data("en", "", "1\nzqzblorptest\n")
            .expect("a minimal hunspell dictionary");

        assert!(checker.has_hunspell(), "did not use the data it was given");
        assert!(
            checker.is_correct("zqzblorptest"),
            "did not know its own word"
        );
    }

    #[test]
    fn test_from_hunspell_data_picks_the_alphabet_for_the_language_asked_for() {
        let es = SpellChecker::from_hunspell_data("es", "", "1\nhola\n")
            .expect("a minimal hunspell dictionary");
        assert!(es.alphabet.contains('ñ'), "{}", es.alphabet);

        let unknown = SpellChecker::from_hunspell_data("xx", "", "1\nhola\n")
            .expect("a minimal hunspell dictionary");
        assert_eq!(unknown.alphabet, "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_a_plain_checker_without_a_real_dictionary_says_so() {
        // The dev and CI machines this runs on do not have Hunspell
        // installed at any of the standard locations, so this falls back to
        // the built-in word list, and has_hunspell has to say that honestly.
        assert!(!SpellChecker::new().has_hunspell());
    }

    #[test]
    fn test_load_dictionary_file_adds_every_nonblank_line() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("extra.txt");
        std::fs::write(&path, "zqzblorpfile\n   \nanotherzq\n").expect("write the word list");

        let mut checker = SpellChecker::new();
        assert!(!checker.is_correct("zqzblorpfile"), "already known somehow");

        let added = checker.load_dictionary_file(&path).expect("read the file");

        assert_eq!(added, 2, "the blank line was counted as a word");
        assert!(checker.is_correct("zqzblorpfile"));
        assert!(checker.is_correct("ANOTHERZQ"), "case was not folded");
    }

    #[test]
    fn test_suggest_offers_a_real_correction_for_a_common_misspelling() {
        let checker = SpellChecker::new();

        let suggestions = checker.suggest("recieve", 5);

        assert!(
            suggestions.iter().any(|s| s == "receive"),
            "{suggestions:?}"
        );
    }

    #[test]
    fn test_word_count_includes_words_added_this_session() {
        let mut checker = SpellChecker::new();
        let before = checker.word_count();

        checker.add_word("wixenite");

        assert_eq!(checker.word_count(), before + 1);
    }

    #[test]
    fn test_default_dict_search_paths_includes_the_windows_location() {
        let paths = default_dict_search_paths();

        assert!(paths.iter().any(|p| p.ends_with("hunspell")), "{paths:?}");
    }

    #[test]
    fn test_dict_search_paths_reports_what_this_checker_actually_searched() {
        let checker = SpellChecker::new();

        assert_eq!(
            checker.dict_search_paths(),
            default_dict_search_paths().as_slice()
        );
    }

    #[test]
    fn test_try_load_spellbook_finds_a_dictionary_on_one_of_its_paths() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        std::fs::write(dir.path().join("xx_XX.aff"), "").expect("write aff");
        std::fs::write(dir.path().join("xx_XX.dic"), "1\nzqzblorpspell\n").expect("write dic");

        let backend = try_load_spellbook("xx_XX", &[dir.path().to_path_buf()])
            .expect("a dictionary sitting right where it was told to look");

        let Backend::Spellbook(dict) = backend else {
            panic!("did not load the spellbook backend");
        };
        assert!(dict.check("zqzblorpspell"));
    }

    #[test]
    fn test_a_single_letter_is_too_short_to_be_worth_checking() {
        // is_empty() alone would also be true for an empty string, which is
        // exactly what lets a mutant hide here: is_empty() implies
        // len() <= 1, so the two conditions give the same answer unless the
        // input is short but not empty.
        assert!(is_number_or_special("j"));
    }

    #[test]
    fn test_a_dot_alone_is_not_treated_as_an_email() {
        // Both halves of an email shape have to be there. A period alone
        // shows up in ordinary abbreviations, and marking every one of them
        // unworthy of checking would hide a real misspelling next to it.
        assert!(!is_number_or_special("wixen.mail"));
    }

    // ── Document locale and i18n ─────────────────────────────────────────

    /// Puts `DOCUMENT_LOCALE` back to what it was, even if the test that
    /// borrowed it panics first. It is shared with every other test in this
    /// binary.
    struct RestoreDocumentLocale(Option<String>);

    impl Drop for RestoreDocumentLocale {
        fn drop(&mut self) {
            if let Ok(mut locale) = DOCUMENT_LOCALE.write() {
                *locale = self.0.take();
            }
        }
    }

    #[test]
    fn test_set_document_locale_writes_the_code_for_reading_back() {
        let _restore = RestoreDocumentLocale(DOCUMENT_LOCALE.read().ok().and_then(|g| g.clone()));

        set_document_locale("fr");

        let after = DOCUMENT_LOCALE.read().ok().and_then(|g| g.clone());
        assert_eq!(after.as_deref(), Some("fr"));
    }

    #[test]
    fn test_load_translations_file_reads_every_entry_in_the_table() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("fr.json");
        std::fs::write(
            &path,
            r#"{"action.send": "Envoyer", "action.cancel": "Annuler"}"#,
        )
        .expect("write translations");

        let mut i18n = I18n::with_locale("fr");
        let count = i18n
            .load_translations_file("fr", &path)
            .expect("a well-formed translations file");

        assert_eq!(count, 2, "did not read every entry");
        assert_eq!(i18n.t("action.send"), "Envoyer");
        assert_eq!(i18n.t("action.cancel"), "Annuler");
    }

    #[test]
    fn test_set_locale_changes_which_language_is_read_back() {
        let mut i18n = I18n::new();
        assert_eq!(i18n.locale().language_code, "en");

        i18n.set_locale("fr");

        assert_eq!(i18n.locale().language_code, "fr");
    }

    #[test]
    fn test_tf_fills_in_its_placeholders() {
        let mut i18n = I18n::new();
        i18n.load_translations(
            "en",
            HashMap::from([("greeting.hello".to_string(), "Hello, {0}!".to_string())]),
        );

        let said = i18n.tf("greeting.hello", &["Pratik"]);

        assert_eq!(said, "Hello, Pratik!");
    }
}

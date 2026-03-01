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
}

// ── Backend enum ─────────────────────────────────────────────────────────────

/// The active spell-checking backend.
enum Backend {
    /// Hunspell-compatible dictionary via the `spellbook` crate.
    Spellbook(spellbook::Dictionary),
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

        let backend = try_load_spellbook(&hunspell_name, &search_paths)
            .unwrap_or_else(|| {
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
    pub fn from_hunspell_data(lang_code: &str, aff_content: &str, dic_content: &str) -> Result<Self, String> {
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
            backend: Backend::Spellbook(dict),
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

        for segment in text.split(|c: char| c.is_whitespace() || c == '\n' || c == '\r') {
            let word = segment.trim_matches(|c: char| !c.is_alphanumeric());
            if word.len() >= 2 && !self.is_correct(word) {
                let word_offset = offset + segment.find(word).unwrap_or(0);
                errors.push(SpellError {
                    word: word.to_string(),
                    offset: word_offset,
                    suggestions: self.suggest(word, 5),
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
                let mut candidates: Vec<String> = edits
                    .into_iter()
                    .filter(|e| set.contains(e))
                    .collect();
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

    // Application data directory
    if let Some(data) = dirs::data_dir() {
        paths.push(data.join("wixen-mail").join("dictionaries"));
    }
    if let Some(data) = dirs::data_local_dir() {
        paths.push(data.join("wixen-mail").join("dictionaries"));
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
    } else if cfg!(target_os = "windows") {
        if let Some(program_data) = std::env::var_os("ProgramData") {
            paths.push(PathBuf::from(program_data).join("hunspell"));
        }
    }

    paths
}

/// Try to load a Hunspell dictionary from one of the search paths.
fn try_load_spellbook(hunspell_name: &str, search_paths: &[PathBuf]) -> Option<Backend> {
    for dir in search_paths {
        let aff_path = dir.join(format!("{}.aff", hunspell_name));
        let dic_path = dir.join(format!("{}.dic", hunspell_name));

        if aff_path.exists() && dic_path.exists() {
            if let Ok(aff) = std::fs::read_to_string(&aff_path) {
                if let Ok(dic) = std::fs::read_to_string(&dic_path) {
                    let aff_static: &'static str = Box::leak(aff.into_boxed_str());
                    let dic_static: &'static str = Box::leak(dic.into_boxed_str());
                    if let Ok(dict) = spellbook::Dictionary::new(aff_static, dic_static) {
                        tracing::info!(
                            "Loaded Hunspell dictionary '{}' from {}",
                            hunspell_name,
                            dir.display()
                        );
                        return Some(Backend::Spellbook(dict));
                    }
                }
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
    if lower.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == '-') {
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
            if j != i { s.push(c); }
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
                if j == i { s.push(c); }
                s.push(ch);
            }
            if i == n { s.push(c); }
            edits.push(s);
        }
    }

    edits
}

/// Core English word list (~12K words). Used as fallback when no Hunspell
/// dictionary is installed.
const CORE_ENGLISH_WORDS: &str = include_str!("../../data/dictionary_en.txt");

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
        Self { language_code: lang, country_code: country, display_name: display, direction }
    }
}

/// Internationalization (i18n) registry for UI string translations.
pub struct I18n {
    active_locale: Locale,
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new() -> Self {
        let mut i18n = Self { active_locale: Locale::from_code("en"), translations: HashMap::new() };
        i18n.register_english_defaults();
        i18n
    }

    pub fn with_locale(code: &str) -> Self {
        let mut i18n = Self { active_locale: Locale::from_code(code), translations: HashMap::new() };
        i18n.register_english_defaults();
        i18n
    }

    fn register_english_defaults(&mut self) {
        let mut en = HashMap::new();
        for (k, v) in [
            ("menu.file", "File"), ("menu.edit", "Edit"), ("menu.view", "View"),
            ("menu.message", "Message"), ("menu.tools", "Tools"), ("menu.help", "Help"),
            ("action.send", "Send"), ("action.save_draft", "Save Draft"),
            ("action.cancel", "Cancel"), ("action.ok", "OK"), ("action.delete", "Delete"),
            ("action.reply", "Reply"), ("action.reply_all", "Reply All"),
            ("action.forward", "Forward"), ("action.search", "Search"),
            ("status.ready", "Ready"), ("status.checking_mail", "Checking for new mail..."),
            ("status.sending", "Sending..."), ("status.offline", "Offline mode"),
            ("status.online", "Online"), ("status.connected", "Connected"),
            ("status.disconnected", "Disconnected"),
            ("compose.to", "To:"), ("compose.cc", "CC:"), ("compose.bcc", "BCC:"),
            ("compose.subject", "Subject:"), ("compose.from", "From:"),
            ("spellcheck.no_errors", "No spelling errors found"),
            ("spellcheck.errors_found", "Spelling errors found"),
            ("spellcheck.add_to_dictionary", "Add to Dictionary"),
            ("spellcheck.ignore", "Ignore"), ("spellcheck.ignore_all", "Ignore All"),
            ("settings.title", "Settings"), ("settings.general", "General"),
            ("settings.compose", "Compose"), ("settings.reading", "Reading"),
            ("settings.language", "Language"), ("settings.advanced", "Advanced"),
        ] {
            en.insert(k.into(), v.into());
        }
        self.translations.insert("en".into(), en);
    }

    pub fn load_translations(&mut self, lang_code: &str, strings: HashMap<String, String>) {
        self.translations.insert(lang_code.to_string(), strings);
    }

    pub fn load_translations_file(&mut self, lang_code: &str, path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let count = map.len();
        self.translations.insert(lang_code.to_string(), map);
        Ok(count)
    }

    pub fn set_locale(&mut self, code: &str) { self.active_locale = Locale::from_code(code); }
    pub fn locale(&self) -> &Locale { &self.active_locale }

    pub fn t(&self, string_id: &str) -> String {
        if let Some(table) = self.translations.get(&self.active_locale.language_code) {
            if let Some(s) = table.get(string_id) { return s.clone(); }
        }
        if let Some(en) = self.translations.get("en") {
            if let Some(s) = en.get(string_id) { return s.clone(); }
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
    fn default() -> Self { Self::new() }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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
        for l in &langs { assert!(!l.hunspell_name.is_empty()); }
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
    fn test_locale_rtl_detection() {
        let en = Locale::from_code("en-US");
        assert_eq!(en.direction, TextDirection::LeftToRight);
        let ar = Locale::from_code("ar");
        assert_eq!(ar.direction, TextDirection::RightToLeft);
    }
}

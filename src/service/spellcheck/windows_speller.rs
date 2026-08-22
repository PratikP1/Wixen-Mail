//! Windows' own spell checker, the one every other application here uses.
//!
//! `ISpellChecker`, from Windows 8 onwards. The reason to prefer it over the
//! checker in this crate is not that its suggestions are better, although they
//! are. It is that **it already knows the words this person has taught
//! Windows**.
//!
//! Somebody who has added their colleagues' surnames, their employer's product
//! names and the abbreviations their field uses has done that work once, in
//! Windows Settings, and every well-behaved application on the machine has the
//! benefit of it. An application that ships its own dictionary makes them do it
//! again, one word at a time, in a second place, and that is WCAG 3.3.7
//! Redundant Entry in the plainest form it takes.
//!
//! Adding a word here writes to that same shared dictionary, so it is learned
//! everywhere rather than only in this application.
//!
//! # What it does not do
//!
//! No grammar checking: `ISpellChecker` is spelling only. macOS's
//! `NSSpellChecker` does grammar and automatic language identification, and
//! neither Windows nor Linux offers either, so the three platforms will not
//! reach feature parity and this application should say what it has on the
//! machine it is on rather than pretend to a lowest common denominator.
//!
//! # It stays on one thread
//!
//! A COM interface pointer belongs to the apartment it was created in. This is
//! deliberately not `Send`: it is made where it is used and dropped there. The
//! alternative is marshalling, which is a great deal of machinery for a spell
//! checker.

use super::{SpellError, Speller};
use crate::common::{Error, Result};
use windows::Win32::Globalization::{
    CORRECTIVE_ACTION_DELETE, CORRECTIVE_ACTION_GET_SUGGESTIONS, CORRECTIVE_ACTION_REPLACE,
    ISpellChecker, ISpellCheckerFactory, ISpellingError, SpellCheckerFactory,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::PCWSTR;

/// The most suggestions to take for one word.
///
/// Windows will offer more. Past about five they stop being alternatives and
/// start being a list to sit through, which costs more for somebody listening
/// than for somebody glancing.
const MAX_SUGGESTIONS: usize = 5;

/// Make sure this thread can talk to COM at all.
///
/// `CoCreateInstance` fails with `CO_E_NOTINITIALIZED` on a thread that has not
/// initialised COM, and the failure looks exactly like Windows not having a
/// spell checker: the caller falls back to the built-in word list and nobody
/// finds out. That is what the first run of this did, and it is the kind of
/// thing that ships as a feature nobody can tell is broken.
///
/// Three outcomes are all fine. `S_OK` initialised it, `S_FALSE` means this
/// thread already had, and `RPC_E_CHANGED_MODE` means the thread is in the
/// other apartment model, which is somebody else's decision and still works for
/// an in-process class.
///
/// Nothing here ever calls `CoUninitialize`. This is a library on somebody
/// else's thread, and tearing down an apartment the host set up would break far
/// more than spell checking.
///
/// Not unit tested. `CoInitializeEx` is a real COM call, and every branch of
/// the warning below is reached only by an `HRESULT` this project cannot
/// produce on demand: a fresh test thread gets `S_OK`, calling this again on
/// the same thread gets `S_FALSE`, and forcing `RPC_E_CHANGED_MODE` or
/// anything else needs a thread already joined to a different apartment
/// model, which is state a test would have to fake COM itself to control.
/// The one thing that ever happens as a result, on the branch that would
/// need to differ, is a `tracing::warn!` call this codebase has no test
/// anywhere that captures.
fn ensure_com_initialised() {
    use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

    // Safe: the documented way to join an apartment, and the result is
    // inspected rather than assumed.
    let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if result != S_OK && result != S_FALSE && result != RPC_E_CHANGED_MODE {
        tracing::warn!(
            "COM could not be initialised on this thread ({:?}), so Windows \
             spell checking is unavailable here",
            result
        );
    }
}

/// A null-terminated UTF-16 string, kept alive for the length of the call.
struct Wide(Vec<u16>);

impl Wide {
    fn new(value: &str) -> Self {
        Self(value.encode_utf16().chain(std::iter::once(0)).collect())
    }

    fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.0.as_ptr())
    }
}

/// Windows' spell checker for one language.
pub struct WindowsSpeller {
    checker: ISpellChecker,
    language: String,
}

impl WindowsSpeller {
    /// Ask Windows for a checker in this language.
    ///
    /// `None` when Windows has no dictionary for it, which is the ordinary
    /// answer on a machine where that language pack was never installed, and on
    /// a Windows Server build such as a CI runner, where the spell checking
    /// feature is often absent altogether. The caller falls back rather than
    /// failing, because a checker being unavailable is not an error.
    ///
    /// Not unit tested, on the same grounds this doc comment just gave: the
    /// factory this creates is a real COM class that this project's own CI
    /// runners are, by this paragraph's own account, not guaranteed to have.
    /// A test that asserted `Some` would be pinned to a feature outside this
    /// project's control, and one that asserted `None` would break the day
    /// somebody runs the suite on a desktop build where the feature is
    /// present, which is every real user's machine. `WindowsSpeller`'s only
    /// constructor is this one, so the trait methods below inherit the same
    /// limit: there is no way to hold an instance in a test without first
    /// getting past this.
    pub fn for_language(tag: &str) -> Option<Self> {
        ensure_com_initialised();
        // Safe: the factory is a documented in-process COM class, the language
        // tag outlives the call, and every interface pointer is checked before
        // it is used.
        unsafe {
            let factory: ISpellCheckerFactory =
                CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER).ok()?;
            let wide = Wide::new(tag);
            if !factory.IsSupported(wide.as_pcwstr()).ok()?.as_bool() {
                return None;
            }
            let checker = factory.CreateSpellChecker(wide.as_pcwstr()).ok()?;
            Some(Self {
                checker,
                language: tag.to_string(),
            })
        }
    }

    /// Every language Windows can check on this machine.
    ///
    /// The real list rather than one written down here, so the setting offers
    /// what will actually work instead of what somebody hoped would.
    ///
    /// Not unit tested: the list is real Windows state, out of this
    /// project's control and different on every machine, including whether
    /// the COM factory exists at all. `mod.rs` already tests the pure
    /// decision layered on top of this (`choices_from`, `best_available_match`,
    /// `find_regional_variant`) against lists it makes up itself, which is
    /// what actually needed pinning; this is the live data feeding it.
    pub fn supported_languages() -> Vec<String> {
        ensure_com_initialised();
        let mut tags = Vec::new();
        // Safe: as above. `Next` reports how many it wrote, and nothing is read
        // beyond that count.
        unsafe {
            let Ok(factory) = CoCreateInstance::<_, ISpellCheckerFactory>(
                &SpellCheckerFactory,
                None,
                CLSCTX_INPROC_SERVER,
            ) else {
                return tags;
            };
            let Ok(languages) = factory.SupportedLanguages() else {
                return tags;
            };
            loop {
                let mut buffer = [windows::core::PWSTR::null(); 1];
                let mut written = 0u32;
                if languages.Next(&mut buffer, Some(&mut written)).is_err() || written == 0 {
                    break;
                }
                if let Ok(tag) = buffer[0].to_string() {
                    tags.push(tag);
                }
            }
        }
        tags
    }
}

/// What Windows calls a language, in the reader's own words.
///
/// `LOCALE_SNATIVEDISPLAYNAME`, so French reads as "français (France)" rather
/// than as "French (France)". Somebody choosing the language they write in
/// recognises its own name faster than its English one, and for a lot of people
/// the English one is not a name they use at all.
///
/// Falls back to the tag, which is ugly and still choosable, rather than to
/// nothing.
pub fn display_name(tag: &str) -> String {
    let wide = Wide::new(tag);
    let mut buffer = [0u16; 128];
    // Safe: the buffer's length is passed with it, and the return value says
    // how much was written before any of it is read.
    let written = unsafe {
        windows::Win32::Globalization::GetLocaleInfoEx(
            wide.as_pcwstr(),
            windows::Win32::Globalization::LOCALE_SNATIVEDISPLAYNAME,
            Some(&mut buffer),
        )
    };
    if written <= 1 {
        return tag.to_string();
    }
    // The count includes the terminator.
    String::from_utf16_lossy(&buffer[..written as usize - 1])
}

// None of the five methods below are unit tested, `language` included even
// though it is a plain field read with no COM in it: `WindowsSpeller`'s only
// constructor is `for_language`, and that needs a live COM spell checking
// factory to succeed, for the same reasons given on that function. There is
// no way to hold an instance to call any of these on without first getting
// past it, short of fabricating a COM interface pointer from nothing, which
// is undefined behaviour rather than a test. `check` and `suggest` decode
// what a real `ISpellChecker` hands back; `word_at` and `byte_offset_of`
// just above are the pure parts of that decoding, pulled out on purpose so
// they could be pinned down without one.
impl Speller for WindowsSpeller {
    fn check(&self, text: &str) -> Vec<SpellError> {
        // Windows counts in UTF-16 code units and the rest of this application
        // counts in bytes. A single emoji or an accented character earlier in
        // the line puts every later offset in the wrong place, and the caret
        // then lands somewhere arbitrary, which somebody listening cannot see
        // and cannot correct for.
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let mut found = Vec::new();

        // Safe: the text outlives the enumeration, and each error's fields are
        // read only after its own call succeeded.
        unsafe {
            let wide = Wide::new(text);
            let Ok(errors) = self.checker.Check(wide.as_pcwstr()) else {
                return found;
            };
            loop {
                // Writes through an out pointer and reports S_FALSE at the end
                // rather than failing, so the None is the terminator and not an
                // error to report.
                let mut next: Option<ISpellingError> = None;
                if errors.Next(&mut next).is_err() {
                    break;
                }
                let Some(error) = next else { break };
                let (Ok(start), Ok(length)) = (error.StartIndex(), error.Length()) else {
                    break;
                };
                let Some(word) = word_at(&utf16, start as usize, length as usize) else {
                    continue;
                };
                let Some(offset) = byte_offset_of(&utf16, start as usize) else {
                    continue;
                };

                // Windows says what it thinks should happen. A replacement it
                // is sure of is offered first rather than made silently: an
                // autocorrection nobody was told about is a change somebody
                // finds later in a sent message.
                let action = error
                    .CorrectiveAction()
                    .unwrap_or(CORRECTIVE_ACTION_GET_SUGGESTIONS);
                let suggestions = if action == CORRECTIVE_ACTION_REPLACE {
                    error
                        .Replacement()
                        .ok()
                        .and_then(|value| value.to_string().ok())
                        .into_iter()
                        .collect()
                } else if action == CORRECTIVE_ACTION_DELETE {
                    // A repeated word. The correction is to remove it, which is
                    // said in words rather than offered as an empty string.
                    Vec::new()
                } else {
                    self.suggest(&word, MAX_SUGGESTIONS)
                };

                found.push(SpellError {
                    word,
                    offset,
                    suggestions,
                    repeated: action == CORRECTIVE_ACTION_DELETE,
                });
            }
        }
        found
    }

    fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        let mut suggestions = Vec::new();
        // Safe: the word outlives the enumeration and `Next` reports its own
        // count.
        unsafe {
            let wide = Wide::new(word);
            let Ok(candidates) = self.checker.Suggest(wide.as_pcwstr()) else {
                return suggestions;
            };
            while suggestions.len() < max {
                let mut buffer = [windows::core::PWSTR::null(); 1];
                let mut written = 0u32;
                if candidates.Next(&mut buffer, Some(&mut written)).is_err() || written == 0 {
                    break;
                }
                if let Ok(text) = buffer[0].to_string() {
                    suggestions.push(text);
                }
            }
        }
        suggestions
    }

    fn add_to_dictionary(&self, word: &str) -> Result<()> {
        // Straight into the dictionary Windows shares with every other
        // application, which is the whole point of using it.
        unsafe {
            let wide = Wide::new(word);
            self.checker
                .Add(wide.as_pcwstr())
                .map_err(|e| Error::Other(format!("Windows would not learn that word: {e}")))
        }
    }

    fn ignore(&self, word: &str) {
        // Ignored for this session only, which is what Windows means by it and
        // what somebody means when they say ignore rather than add.
        unsafe {
            let wide = Wide::new(word);
            let _ = self.checker.Ignore(wide.as_pcwstr());
        }
    }

    fn language(&self) -> &str {
        &self.language
    }

    fn source(&self) -> super::Source {
        super::Source::Windows
    }
}

/// The word at a UTF-16 range, as a `String`.
fn word_at(utf16: &[u16], start: usize, length: usize) -> Option<String> {
    let end = start.checked_add(length)?;
    let slice = utf16.get(start..end)?;
    String::from_utf16(slice).ok()
}

/// The byte offset in the original text of a UTF-16 code unit index.
///
/// The conversion the rest of this module exists to get right. Windows counts
/// UTF-16 code units; every offset this application stores is a byte offset,
/// because that is what Rust strings are indexed by.
fn byte_offset_of(utf16: &[u16], index: usize) -> Option<usize> {
    let prefix = utf16.get(..index)?;
    // Counting the encoded length of the text before the word is exact and
    // needs no assumption about what is in it. Re-decoding is cheap next to a
    // COM call, and a surrogate pair split across the boundary would make any
    // arithmetic shortcut wrong.
    Some(String::from_utf16(prefix).ok()?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_reads_a_locale_windows_always_knows() {
        // en-US ships with every Windows install, unlike a COM spell
        // checking factory, which is a feature that can be absent. That
        // makes this deterministic on any machine this project actually
        // runs its tests on, unlike WindowsSpeller::for_language and
        // supported_languages just above. LOCALE_SNATIVEDISPLAYNAME answers
        // "what does en-US call itself", not "translate en-US into the
        // system's own UI language", so this holds regardless of what
        // language the test machine itself is running in.
        let name = display_name("en-US");

        assert_ne!(name, "en-US", "fell back to the tag itself");
        assert!(name.contains("English"), "{name:?}");
    }

    #[test]
    fn test_as_pcwstr_points_at_the_wide_strings_own_buffer() {
        let wide = Wide::new("hi");

        let pcwstr = wide.as_pcwstr();

        assert_eq!(pcwstr.0, wide.0.as_ptr(), "did not point at its own buffer");
    }

    #[test]
    fn test_a_byte_offset_matches_a_utf16_index_for_plain_text() {
        let text = "the recieve line";
        let utf16: Vec<u16> = text.encode_utf16().collect();

        // "recieve" starts at index 4 either way when everything is ASCII.
        assert_eq!(byte_offset_of(&utf16, 4), Some(4));
    }

    #[test]
    fn test_a_byte_offset_is_not_a_utf16_index_once_the_text_is_not_ascii() {
        // The bug this function exists to prevent. One emoji is two UTF-16
        // code units and four bytes, so every offset after it is out by two.
        // The caret would land two characters early, in the middle of a word,
        // and somebody listening cannot see that it has.
        let text = "\u{1F98A} recieve";
        let utf16: Vec<u16> = text.encode_utf16().collect();

        // The word starts at UTF-16 index 3: two units for the fox, one space.
        let offset = byte_offset_of(&utf16, 3).expect("in range");

        assert_eq!(offset, 5, "four bytes for the fox and one for the space");
        assert_eq!(&text[offset..], "recieve");
    }

    #[test]
    fn test_an_accented_word_lands_where_it_really_is() {
        let text = "café recieve";
        let utf16: Vec<u16> = text.encode_utf16().collect();

        // "recieve" is UTF-16 index 5, byte offset 6: é is two bytes.
        let offset = byte_offset_of(&utf16, 5).expect("in range");

        assert_eq!(&text[offset..], "recieve");
    }

    #[test]
    fn test_an_index_past_the_end_is_none_rather_than_a_panic() {
        // These numbers come from another process. Trusting one would panic in
        // the middle of somebody's message.
        let utf16: Vec<u16> = "short".encode_utf16().collect();

        assert_eq!(byte_offset_of(&utf16, 99), None);
        assert_eq!(word_at(&utf16, 3, 99), None);
        assert_eq!(word_at(&utf16, 99, 1), None);
    }

    #[test]
    fn test_a_length_that_overflows_is_refused() {
        let utf16: Vec<u16> = "short".encode_utf16().collect();

        assert_eq!(word_at(&utf16, 1, usize::MAX), None);
    }

    #[test]
    fn test_a_word_comes_back_out_of_its_range() {
        let text = "the recieve line";
        let utf16: Vec<u16> = text.encode_utf16().collect();

        assert_eq!(word_at(&utf16, 4, 7).as_deref(), Some("recieve"));
    }

    #[test]
    fn test_the_wide_string_is_terminated() {
        // Every one of these is handed to an API that reads until a null. One
        // without a terminator reads whatever follows it in memory.
        let wide = Wide::new("recieve");

        assert_eq!(wide.0.last(), Some(&0));
        assert_eq!(wide.0.len(), "recieve".len() + 1);
    }
}

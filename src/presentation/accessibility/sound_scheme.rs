//! A named collection of sounds, one per event at most.
//!
//! `feedback::Event` says a fact has happened and, through
//! `FeedbackSettings`, which channels it reaches. This says something
//! narrower: when the earcon channel is one of them, which actual sound
//! plays. The built-in `generated` scheme answers "the synthesized tone,
//! every time," and is what every installation starts on. A scheme built
//! from real sound files answers the same question with a path instead, for
//! whichever events it names, and falls back to the synthesized tone for
//! anything it does not, the same graceful-degradation choice
//! `FeedbackSettings::from_stored` already makes for a setting it does not
//! recognise.

use super::feedback::Event;
use crate::common::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where a scheme's sound for one event comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundSource {
    /// The built-in synthesized tone.
    Generated,
    /// A real sound file at this path.
    File(PathBuf),
}

/// A named collection of sounds, one per event at most.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundScheme {
    /// The stable identifier this scheme is stored and looked up by.
    pub id: String,
    /// What Settings shows in the scheme picker.
    pub name: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    /// Event key to a real file's full path, already resolved against
    /// wherever the manifest itself lives. Only the keys a scheme actually
    /// names appear here; everything else falls back to Generated.
    sounds: HashMap<String, PathBuf>,
}

impl Default for SoundScheme {
    fn default() -> Self {
        Self::generated()
    }
}

impl SoundScheme {
    /// The scheme every installation starts on: no files, every event plays
    /// its own synthesized tone.
    ///
    /// Off by default is the earcon channel's own choice
    /// (`FeedbackSettings::default`); this is a separate, second default,
    /// for what plays on the occasions it is on.
    pub fn generated() -> Self {
        Self {
            id: "generated".to_string(),
            name: "Generated tones".to_string(),
            author: None,
            license: None,
            description: None,
            sounds: HashMap::new(),
        }
    }

    /// Which source plays for one event.
    pub fn source_for(&self, event: Event) -> SoundSource {
        match self.sounds.get(event.key()) {
            Some(path) => SoundSource::File(path.clone()),
            None => SoundSource::Generated,
        }
    }

    /// How many of the real events this scheme names a file for, so
    /// Settings can show "covers 9 of 16 events" rather than leave someone
    /// guessing whether choosing it means losing some tones to silence
    /// (it does not; the rest still play Generated) or to files nobody
    /// checked exist.
    pub fn covers(&self) -> usize {
        Event::ALL
            .iter()
            .filter(|event| self.sounds.contains_key(event.key()))
            .count()
    }

    /// Parse a scheme's manifest, resolving every sound's path against
    /// `dir`, the directory the manifest itself lives in.
    ///
    /// A relative path in the TOML means "beside this file," never
    /// "wherever the process happened to start," which is what makes an
    /// imported scheme's directory portable: move the whole folder and its
    /// sounds still resolve.
    ///
    /// An event name the manifest carries that this build does not
    /// recognise is skipped rather than refused, the same forward-
    /// compatibility choice `FeedbackSettings::from_stored` already makes:
    /// a newer scheme naming an event an older build has never heard of
    /// should not lose the rest of what it names.
    pub fn from_manifest(id: &str, manifest: &str, dir: &Path) -> Result<Self> {
        let parsed: Manifest = toml::from_str(manifest)
            .map_err(|e| Error::Config(format!("sound scheme manifest: {e}")))?;
        if parsed.name.trim().is_empty() {
            return Err(Error::Config(
                "sound scheme manifest: name is empty".to_string(),
            ));
        }
        let sounds = parsed
            .sounds
            .into_iter()
            .filter(|(key, _)| Event::ALL.iter().any(|event| event.key() == key))
            .map(|(key, relative)| (key, dir.join(relative)))
            .collect();
        Ok(Self {
            id: id.to_string(),
            name: parsed.name,
            author: non_empty(parsed.author),
            license: non_empty(parsed.license),
            description: non_empty(parsed.description),
            sounds,
        })
    }
}

/// The file every scheme's own directory holds its manifest as.
///
/// `pub(crate)` rather than private: `sound_scheme_import` needs the same
/// name when it writes a freshly imported pack's manifest to disk, and a
/// second, hand-copied literal is how the two would eventually disagree.
pub(crate) const MANIFEST_FILE: &str = "scheme.toml";

impl SoundScheme {
    /// Find a scheme by its stored id.
    ///
    /// Empty or `"generated"` is always the built-in default; anything else
    /// is looked up as a subdirectory of `schemes_dir` holding its own
    /// manifest. An id that resolves to nothing, because the scheme was
    /// removed or the stored id is not one this installation recognises,
    /// falls back to Generated rather than refusing to play anything: a
    /// missing scheme is a reason to fall back, not a reason to go silent.
    pub fn resolve(id: &str, schemes_dir: &Path) -> Self {
        if id.is_empty() || id == "generated" {
            return Self::generated();
        }
        let dir = schemes_dir.join(id);
        let manifest_path = dir.join(MANIFEST_FILE);
        match std::fs::read_to_string(&manifest_path) {
            Ok(manifest) => Self::from_manifest(id, &manifest, &dir).unwrap_or_else(|err| {
                tracing::warn!(
                    "{} could not be read ({err}); using Generated tones instead",
                    manifest_path.display()
                );
                Self::generated()
            }),
            Err(err) => {
                tracing::warn!(
                    "{} is not there ({err}); using Generated tones instead",
                    manifest_path.display()
                );
                Self::generated()
            }
        }
    }

    /// Every scheme currently available: the built-in default first, then
    /// whatever real packs are sitting in `schemes_dir`, each read fresh so
    /// Settings always shows the current disk state rather than a stale
    /// list from whenever the application started.
    ///
    /// A subdirectory with no manifest, or one that fails to parse, is
    /// skipped rather than shown broken: Settings is not the place to
    /// explain a corrupt import, and `resolve` above already falls back
    /// safely if a stored choice ever points at one of these.
    pub fn discover(schemes_dir: &Path) -> Vec<Self> {
        let mut schemes = vec![Self::generated()];
        let Ok(entries) = std::fs::read_dir(schemes_dir) else {
            return schemes;
        };
        let mut imported: Vec<Self> = Vec::new();
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let Some(id) = dir.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(manifest) = std::fs::read_to_string(dir.join(MANIFEST_FILE)) else {
                continue;
            };
            if let Ok(scheme) = Self::from_manifest(id, &manifest, &dir) {
                imported.push(scheme);
            }
        }
        // A directory listing carries no promised order of its own, which
        // would otherwise make the scheme picker's own order change from
        // one run to the next for no reason a person watching it could see.
        // Generated stays first regardless, since it is the default every
        // installation starts on, not one import among others.
        imported.sort_by_key(|scheme| scheme.name.to_lowercase());
        schemes.extend(imported);
        schemes
    }

    /// Remove an imported scheme's own folder.
    ///
    /// Refuses "generated" and an empty id outright, the same two spellings
    /// [`Self::resolve`] treats as the built-in default: it has no folder of
    /// its own to remove, and a machine with nothing else installed must
    /// never be able to end up with no scheme at all.
    pub fn delete(id: &str, schemes_dir: &Path) -> Result<()> {
        if id.is_empty() || id == "generated" {
            return Err(Error::Config(
                "Generated tones is the built-in default and cannot be deleted".to_string(),
            ));
        }
        let dir = schemes_dir.join(id);
        std::fs::remove_dir_all(&dir)
            .map_err(|e| Error::Config(format!("could not remove {}: {e}", dir.display())))
    }
}

/// An empty string is the same as the field being absent: TOML has no way
/// to tell "author = \"\"" from a typo apart from treating both as nothing
/// worth showing.
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

/// The manifest as it is written in TOML, before paths are resolved and
/// unrecognised events are dropped.
#[derive(Debug, serde::Deserialize)]
struct Manifest {
    name: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    sounds: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_generated_scheme_answers_every_event_with_the_tone() {
        let scheme = SoundScheme::generated();
        for event in Event::ALL {
            assert_eq!(scheme.source_for(event), SoundSource::Generated);
        }
        assert_eq!(scheme.covers(), 0);
    }

    #[test]
    fn test_a_manifest_names_a_file_resolved_against_its_own_directory() {
        let manifest = r#"
            name = "Soft Chimes"
            author = "Someone"
            license = "CC0-1.0"

            [sounds]
            new_mail = "sounds/new_mail.wav"
        "#;
        let scheme =
            SoundScheme::from_manifest("soft-chimes", manifest, Path::new("/schemes/soft-chimes"))
                .expect("a valid manifest");

        assert_eq!(scheme.id, "soft-chimes");
        assert_eq!(scheme.name, "Soft Chimes");
        assert_eq!(scheme.author.as_deref(), Some("Someone"));
        assert_eq!(
            scheme.source_for(Event::NewMail),
            SoundSource::File(PathBuf::from("/schemes/soft-chimes/sounds/new_mail.wav"))
        );
    }

    #[test]
    fn test_an_event_the_manifest_does_not_name_falls_back_to_generated() {
        let manifest = r#"
            name = "Soft Chimes"

            [sounds]
            new_mail = "new_mail.wav"
        "#;
        let scheme =
            SoundScheme::from_manifest("soft-chimes", manifest, Path::new("/x")).expect("valid");
        assert_eq!(scheme.source_for(Event::SendFailed), SoundSource::Generated);
        assert_eq!(scheme.covers(), 1);
    }

    #[test]
    fn test_an_event_name_this_build_does_not_recognise_is_skipped_not_refused() {
        // A newer scheme may name an event an older build has never heard
        // of. That should not lose the rest of what the manifest names, the
        // same choice FeedbackSettings::from_stored already makes for a
        // setting it does not recognise.
        let manifest = r#"
            name = "Future Pack"

            [sounds]
            new_mail = "new_mail.wav"
            some_event_from_a_later_version = "mystery.wav"
        "#;
        let scheme =
            SoundScheme::from_manifest("future", manifest, Path::new("/x")).expect("valid");
        assert_eq!(scheme.covers(), 1);
        assert_eq!(
            scheme.source_for(Event::NewMail),
            SoundSource::File(PathBuf::from("/x/new_mail.wav"))
        );
    }

    #[test]
    fn test_malformed_toml_is_refused() {
        let result =
            SoundScheme::from_manifest("broken", "this is not toml at all {{{", Path::new("/x"));
        assert!(result.is_err());
    }

    #[test]
    fn test_a_manifest_with_no_name_is_refused() {
        let result =
            SoundScheme::from_manifest("nameless", "author = \"Someone\"", Path::new("/x"));
        assert!(result.is_err());
    }

    #[test]
    fn test_an_empty_name_is_refused_the_same_as_a_missing_one() {
        let result = SoundScheme::from_manifest("blank", "name = \"   \"", Path::new("/x"));
        assert!(result.is_err());
    }

    #[test]
    fn test_an_empty_author_reads_as_absent_rather_than_a_blank_line_in_settings() {
        let manifest = "name = \"Pack\"\nauthor = \"\"";
        let scheme = SoundScheme::from_manifest("pack", manifest, Path::new("/x")).expect("valid");
        assert_eq!(scheme.author, None);
    }

    #[test]
    fn test_resolving_an_empty_or_generated_id_never_touches_the_disk() {
        // A directory that does not exist would make a real lookup fail;
        // both of these must never try one at all.
        let nowhere = Path::new("/this/path/is/never/read");
        assert_eq!(SoundScheme::resolve("", nowhere), SoundScheme::generated());
        assert_eq!(
            SoundScheme::resolve("generated", nowhere),
            SoundScheme::generated()
        );
    }

    #[test]
    fn test_resolving_a_real_scheme_reads_its_own_manifest() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let scheme_dir = dir.path().join("soft-chimes");
        std::fs::create_dir_all(&scheme_dir).expect("the scheme's own directory");
        std::fs::write(
            scheme_dir.join("scheme.toml"),
            "name = \"Soft Chimes\"\n\n[sounds]\nnew_mail = \"new_mail.wav\"\n",
        )
        .expect("a manifest to resolve");

        let resolved = SoundScheme::resolve("soft-chimes", dir.path());
        assert_eq!(resolved.name, "Soft Chimes");
        assert_eq!(
            resolved.source_for(Event::NewMail),
            SoundSource::File(scheme_dir.join("new_mail.wav"))
        );
    }

    #[test]
    fn test_resolving_an_id_with_nothing_behind_it_falls_back_to_generated() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let resolved = SoundScheme::resolve("never-imported", dir.path());
        assert_eq!(resolved, SoundScheme::generated());
    }

    #[test]
    fn test_resolving_a_scheme_whose_manifest_is_broken_falls_back_to_generated() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let scheme_dir = dir.path().join("broken");
        std::fs::create_dir_all(&scheme_dir).expect("the scheme's own directory");
        std::fs::write(scheme_dir.join("scheme.toml"), "not valid toml {{{")
            .expect("a broken manifest");

        let resolved = SoundScheme::resolve("broken", dir.path());
        assert_eq!(resolved, SoundScheme::generated());
    }

    #[test]
    fn test_discovering_schemes_always_leads_with_generated() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let schemes = SoundScheme::discover(dir.path());
        assert_eq!(schemes, vec![SoundScheme::generated()]);
    }

    #[test]
    fn test_discovering_schemes_finds_every_real_import() {
        let dir = tempfile::tempdir().expect("a temp directory");
        for (id, name) in [("soft-chimes", "Soft Chimes"), ("classic", "Classic")] {
            let scheme_dir = dir.path().join(id);
            std::fs::create_dir_all(&scheme_dir).expect("a scheme directory");
            std::fs::write(
                scheme_dir.join("scheme.toml"),
                format!("name = \"{name}\"\n"),
            )
            .expect("a manifest");
        }
        // A stray file beside the real scheme directories, and a directory
        // with no manifest in it at all: neither should stop discovery or
        // appear as a scheme of its own.
        std::fs::write(dir.path().join("readme.txt"), "not a scheme").expect("a stray file");
        std::fs::create_dir_all(dir.path().join("empty")).expect("an empty directory");

        let schemes = SoundScheme::discover(dir.path());
        let names: Vec<&str> = schemes.iter().map(|s| s.name.as_str()).collect();
        // Generated first, then alphabetical: a directory listing carries
        // no order of its own, and sorting is what keeps this list the
        // same from one run to the next regardless of it.
        assert_eq!(names, vec!["Generated tones", "Classic", "Soft Chimes"]);
    }

    #[test]
    fn test_discovering_schemes_in_a_folder_that_does_not_exist_still_returns_generated() {
        // The sound-schemes folder may not exist yet on a machine that has
        // never imported anything. That is not a reason Settings should
        // show an empty scheme list.
        let resolved = SoundScheme::discover(Path::new("/this/path/does/not/exist/at/all"));
        assert_eq!(resolved, vec![SoundScheme::generated()]);
    }

    #[test]
    fn test_deleting_generated_is_refused_whatever_the_id_looks_like() {
        // A machine with nothing else installed must never end up with no
        // scheme at all, so the built-in default has no folder a delete
        // could remove, empty id or the real one.
        let dir = tempfile::tempdir().expect("a temp directory");
        assert!(SoundScheme::delete("generated", dir.path()).is_err());
        assert!(SoundScheme::delete("", dir.path()).is_err());
    }

    #[test]
    fn test_deleting_a_real_import_removes_it_from_discovery() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let scheme_dir = dir.path().join("soft-chimes");
        std::fs::create_dir_all(&scheme_dir).expect("the scheme's own directory");
        std::fs::write(scheme_dir.join("scheme.toml"), "name = \"Soft Chimes\"\n")
            .expect("a manifest");
        assert_eq!(SoundScheme::discover(dir.path()).len(), 2);

        SoundScheme::delete("soft-chimes", dir.path()).expect("delete should succeed");

        assert_eq!(
            SoundScheme::discover(dir.path()),
            vec![SoundScheme::generated()]
        );
        assert!(!scheme_dir.exists(), "the folder should really be gone");
    }

    #[test]
    fn test_deleting_a_scheme_that_was_never_there_is_a_real_error_not_silent_success() {
        let dir = tempfile::tempdir().expect("a temp directory");
        assert!(SoundScheme::delete("never-imported", dir.path()).is_err());
    }

    #[test]
    fn test_a_scheme_with_a_file_for_every_event_covers_them_all() {
        let mut sounds = String::from("name = \"Complete\"\n\n[sounds]\n");
        for event in Event::ALL {
            sounds.push_str(&format!("{} = \"{}.wav\"\n", event.key(), event.key()));
        }
        let scheme =
            SoundScheme::from_manifest("complete", &sounds, Path::new("/x")).expect("valid");
        assert_eq!(scheme.covers(), Event::ALL.len());
        for event in Event::ALL {
            assert_ne!(scheme.source_for(event), SoundSource::Generated);
        }
    }
}

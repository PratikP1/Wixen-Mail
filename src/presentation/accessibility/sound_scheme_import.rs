//! Importing a sound-scheme pack from a zip a user picked.
//!
//! A zip from Settings' "Import sound scheme..." picker is content from an
//! unknown third party. Even a pack a user trusts the source of can be
//! malformed or malicious by the time it reaches this code, so every step
//! here refuses rather than guesses, the same "untrusted input stays
//! untrusted" rule this project already applies to remote HTML. See
//! `docs/plans/20260823-earcon-sound-schemes.md`'s "Importing a zip"
//! section for the fuller reasoning behind each cap below.
//!
//! [`import_zip`] is the whole surface. It validates everything a pack
//! could get wrong before writing a single byte to disk, so a refused
//! import leaves nothing behind to clean up.

use super::sound_scheme::{MANIFEST_FILE, SoundScheme};
use crate::common::{Error, Result};
use rodio::Source;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

/// Refused above this size, before the zip is even opened.
///
/// Generous enough for dozens of short clips; a working number from the
/// plan, not a measured one.
const MAX_ZIP_BYTES: u64 = 20 * 1024 * 1024;

/// Refused above this size for any one file the import reads out of the
/// zip, manifest included. A short earcon, or a manifest naming one, has no
/// legitimate reason to be anywhere near this large.
const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;

/// Refused above this size summed across every file the import reads out
/// of the zip. Guards a pack built from many files that each individually
/// pass [`MAX_FILE_BYTES`].
const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;

/// Refused above this length. Earcons are short by definition; a "sound"
/// that runs longer than this is not one.
const MAX_SOUND_DURATION: Duration = Duration::from_secs(2);

/// Import a sound-scheme pack from the zip at `zip_path`, writing it into
/// `schemes_dir/<id>/` on success.
///
/// Every check below runs before anything reaches disk: the function
/// either returns a real, playable [`SoundScheme`] already written into
/// place, or a specific reason nothing was written at all. The one
/// exception, deliberate and explained where it happens, is a single sound
/// file that fails to decode or runs long: that one file is dropped with a
/// warning rather than failing the whole pack, the same graceful
/// degradation [`SoundScheme::from_manifest`] already applies to an event
/// name a build does not recognise.
///
/// `id` is trusted the same way [`SoundScheme::resolve`] and
/// [`SoundScheme::discover`] already trust it: chosen by the caller, not
/// read out of the untrusted zip.
pub fn import_zip(zip_path: &Path, id: &str, schemes_dir: &Path) -> Result<SoundScheme> {
    check_zip_size(zip_path)?;

    let file = fs::File::open(zip_path)
        .map_err(|e| Error::Config(format!("{} could not be opened: {e}", zip_path.display())))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        Error::Config(format!(
            "{} is not a valid zip file: {e}",
            zip_path.display()
        ))
    })?;
    let pack = read_pack(&mut archive, zip_path)?;

    let dest_dir = schemes_dir.join(id);
    let manifest = String::from_utf8(pack.manifest.clone()).map_err(|_| {
        Error::Config(format!(
            "{MANIFEST_FILE} in {} is not valid UTF-8",
            zip_path.display()
        ))
    })?;
    let scheme = SoundScheme::from_manifest(id, &manifest, &dest_dir)?;

    write_pack(&dest_dir, &pack)?;
    Ok(scheme)
}

/// Refuse a zip over [`MAX_ZIP_BYTES`] from its length on disk alone,
/// before anything tries to open or parse it as an archive.
fn check_zip_size(zip_path: &Path) -> Result<()> {
    let zip_size = fs::metadata(zip_path)
        .map_err(|e| Error::Config(format!("{} could not be read: {e}", zip_path.display())))?
        .len();
    if zip_size > MAX_ZIP_BYTES {
        return Err(Error::Config(format!(
            "{} is {zip_size} bytes, over the {MAX_ZIP_BYTES} byte limit for a sound scheme pack",
            zip_path.display()
        )));
    }
    Ok(())
}

/// A zip's contents once every entry has passed every check, still
/// entirely in memory: nothing has touched disk yet.
struct Pack {
    /// `scheme.toml`'s raw bytes, verbatim, so what gets written matches
    /// exactly what [`SoundScheme::from_manifest`] already proved parses.
    manifest: Vec<u8>,
    /// Each real, decodable, short-enough sound file, keyed by its own
    /// path inside the zip (for example `sounds/new_mail.wav`), which is
    /// also the path it gets written at under the destination directory.
    sounds: Vec<(String, Vec<u8>)>,
}

/// Read and validate every entry in `archive`, refusing the whole pack for
/// an unsafe path, an oversized file, or a missing manifest, and dropping
/// (with a warning) any one sound file that fails to decode or runs long.
///
/// Nothing is written to disk here; a caller only reaches [`write_pack`]
/// once this has fully succeeded.
fn read_pack<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    zip_path: &Path,
) -> Result<Pack> {
    let mut manifest: Option<Vec<u8>> = None;
    let mut sounds: Vec<(String, Vec<u8>)> = Vec::new();
    let mut total_extracted: u64 = 0;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| Error::Config(format!("{} is damaged: {e}", zip_path.display())))?;
        let name = entry.name().to_string();
        if !is_safe_entry_path(&name) {
            return Err(Error::Config(format!(
                "{} names {name:?}, a path that would resolve outside the scheme's own folder; \
                 the whole import is refused",
                zip_path.display()
            )));
        }
        if entry.is_dir() {
            continue;
        }

        let is_manifest = name == MANIFEST_FILE;
        if !is_manifest && !is_extractable_sound(&name) {
            tracing::warn!(
                "{name} in {} is not {MANIFEST_FILE} or a sounds/*.wav or sounds/*.ogg file; \
                 skipped",
                zip_path.display()
            );
            continue;
        }

        if entry.size() > MAX_FILE_BYTES {
            return Err(Error::Config(format!(
                "{name} in {} claims to be {} bytes, over the {MAX_FILE_BYTES} byte limit for \
                 one file in a sound scheme pack",
                zip_path.display(),
                entry.size()
            )));
        }
        let bytes = read_capped(&mut entry, MAX_FILE_BYTES).map_err(|_| {
            Error::Config(format!(
                "{name} in {} decompresses to more than the {MAX_FILE_BYTES} byte limit for one \
                 file, whatever its declared size says",
                zip_path.display()
            ))
        })?;
        total_extracted = total_extracted.saturating_add(bytes.len() as u64);
        if total_extracted > MAX_TOTAL_BYTES {
            return Err(Error::Config(format!(
                "{} extracts to more than the {MAX_TOTAL_BYTES} byte total limit for a sound \
                 scheme pack",
                zip_path.display()
            )));
        }

        if is_manifest {
            manifest = Some(bytes);
            continue;
        }

        if let Err(reason) = validate_sound(&bytes) {
            tracing::warn!("{name} in {} {reason}; skipped", zip_path.display());
            continue;
        }
        sounds.push((name, bytes));
    }

    let Some(manifest) = manifest else {
        return Err(Error::Config(format!(
            "{} has no {MANIFEST_FILE} at its root",
            zip_path.display()
        )));
    };
    Ok(Pack { manifest, sounds })
}

/// Write an already fully-validated pack into `dest_dir`, creating it if
/// needed. Nothing about this can refuse an import; every reason to has
/// already been checked before a caller reaches this.
fn write_pack(dest_dir: &Path, pack: &Pack) -> Result<()> {
    fs::create_dir_all(dest_dir)
        .map_err(|e| Error::Config(format!("could not create {}: {e}", dest_dir.display())))?;
    let manifest_path = dest_dir.join(MANIFEST_FILE);
    fs::write(&manifest_path, &pack.manifest)
        .map_err(|e| Error::Config(format!("could not write {}: {e}", manifest_path.display())))?;
    for (name, bytes) in &pack.sounds {
        let path = dest_dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Config(format!("could not create {}: {e}", parent.display()))
            })?;
        }
        fs::write(&path, bytes)
            .map_err(|e| Error::Config(format!("could not write {}: {e}", path.display())))?;
    }
    Ok(())
}

/// Whether an entry's own path, as stored in the zip, is safe to join under
/// the scheme's destination directory.
///
/// Refuses outright rather than correcting. A `..` component or a rooted
/// path could still be rewritten into something that stays inside the
/// destination, but a rewritten path still writes an attacker's file
/// somewhere in the scheme's own folder under a name its manifest never
/// mentioned, which is exactly the silent correction this function exists
/// to avoid. Written by hand rather than built on `Path`'s own component
/// parsing: this project ships a port to a second platform, and `Path`
/// treats `\` as a separator only when the binary itself is built for
/// Windows, so a hostile entry must be refused the same way regardless of
/// which platform built this binary.
fn is_safe_entry_path(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') {
        return false;
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return false;
    }
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        // A drive letter, `C:\...` or the drive-relative `C:foo`.
        return false;
    }
    !name.split(['/', '\\']).any(|segment| segment == "..")
}

/// Whether an already-safety-checked entry is one this import extracts as a
/// sound: directly under `sounds/`, ending `.wav` or `.ogg` regardless of
/// case, since some tools zip files as `.WAV`.
fn is_extractable_sound(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("sounds/") else {
        return false;
    };
    if rest.is_empty() || rest.contains('/') {
        return false;
    }
    let lower = rest.to_ascii_lowercase();
    lower.ends_with(".wav") || lower.ends_with(".ogg")
}

/// Read an entry's real bytes, refusing rather than truncating if the
/// actual decompressed content is over `cap`, regardless of what the
/// entry's own declared size claims.
///
/// A declared size is metadata the archive controls and can misstate: a
/// deflate stream's own compressed bytes, not any size field, are what
/// decide how much a decoder actually produces. Only counting what
/// decompression actually produces catches that.
fn read_capped(entry: &mut impl Read, cap: u64) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    entry.take(cap + 1).read_to_end(&mut buf)?;
    if buf.len() as u64 > cap {
        return Err(std::io::Error::other(
            "decompressed content exceeds the cap",
        ));
    }
    Ok(buf)
}

/// Whether `bytes` is real, decodable audio short enough to be an earcon.
///
/// `Err` carries a human-readable reason a caller can log. This never
/// refuses the whole import on its own: a scheme still importing when one
/// of its files fails this is the choice explained on [`import_zip`].
fn validate_sound(bytes: &[u8]) -> std::result::Result<(), String> {
    let decoded = rodio::Decoder::new(std::io::Cursor::new(bytes.to_vec()))
        .map_err(|err| format!("does not decode as real audio ({err})"))?;
    match decoded.total_duration() {
        Some(duration) if duration <= MAX_SOUND_DURATION => Ok(()),
        Some(duration) => Err(format!(
            "runs {duration:?}, over the {MAX_SOUND_DURATION:?} limit for an earcon"
        )),
        None => Err("has no determinable length".to_string()),
    }
}

/// Turn a file name into a directory-safe scheme id.
///
/// A zip picked through a file dialog has a name a person chose, not a
/// stable identifier: spaces, capitals, punctuation, sometimes characters
/// this filesystem would refuse outright. Lowercased, every run of anything
/// that is not a letter or digit collapsed to one hyphen, and leading or
/// trailing hyphens trimmed off. A name that leaves nothing behind (every
/// character was punctuation, or the string was empty) falls back to
/// `"imported"` rather than asking `import_zip` to write into a directory
/// with an empty name.
pub fn slug_for(file_stem: &str) -> String {
    let mut slug = String::with_capacity(file_stem.len());
    let mut last_was_hyphen = true; // starts true so a leading run collapses away
    for ch in file_stem.chars() {
        if ch.is_alphanumeric() {
            slug.extend(ch.to_lowercase());
            last_was_hyphen = false;
        } else if !last_was_hyphen {
            slug.push('-');
            last_was_hyphen = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "imported".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_a_plain_name_becomes_lowercase() {
        assert_eq!(slug_for("Soft Chimes"), "soft-chimes");
    }

    #[test]
    fn test_punctuation_collapses_to_one_hyphen_rather_than_one_each() {
        assert_eq!(slug_for("Pratik's   Pack!!!"), "pratik-s-pack");
    }

    #[test]
    fn test_leading_and_trailing_punctuation_is_trimmed_not_hyphenated() {
        assert_eq!(slug_for("  (Beta) Chimes  "), "beta-chimes");
    }

    #[test]
    fn test_a_name_that_is_only_punctuation_falls_back_to_imported() {
        assert_eq!(slug_for("!!!"), "imported");
        assert_eq!(slug_for(""), "imported");
    }

    #[test]
    fn test_digits_survive_since_they_are_valid_in_a_directory_name() {
        assert_eq!(slug_for("Pack 2"), "pack-2");
    }

    /// A manifest naming no sounds, valid enough for `from_manifest` to
    /// accept on its own; tests that care about a specific sound file add
    /// their own `[sounds]` table instead of using this one.
    const MINIMAL_MANIFEST: &str = "name = \"Test Pack\"\n";

    fn write_zip_bytes(dir: &Path, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join("pack.zip");
        fs::write(&path, bytes).expect("write the test zip's bytes");
        path
    }

    /// Build a zip in memory from `(entry name, raw bytes)` pairs, using
    /// this project's own zip dependency's writer rather than a checked-in
    /// binary fixture, so every hostile entry is readable in the diff.
    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        for (name, data) in entries {
            writer.start_file(*name, options).expect("start_file");
            writer.write_all(data).expect("write_all");
        }
        writer.finish().expect("finish").into_inner()
    }

    /// Build a zip with a normal manifest entry plus one entry whose
    /// declared uncompressed size does not match what its compressed bytes
    /// really decompress to, using this project's own zip dependency to do
    /// the real compression and then overwriting only the metadata that
    /// gets recorded for the doctored entry.
    ///
    /// This is what a per-file size check that trusts the declared size
    /// alone cannot see coming: the entry looks small enough to pass on
    /// paper, and only decompressing it (bounded, so this never has to
    /// finish decompressing something unbounded) reveals the truth. The
    /// manifest is a normal, honestly-sized entry so a bug that let the lie
    /// through cannot be mistaken for the unrelated "no manifest" refusal.
    fn build_zip_with_lying_size(
        manifest: &[u8],
        name: &str,
        real_data: &[u8],
        lied_uncompressed_size: u64,
    ) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file(MANIFEST_FILE, SimpleFileOptions::default())
            .expect("start_file manifest");
        writer.write_all(manifest).expect("write_all the manifest");
        writer
            .start_file(name, SimpleFileOptions::default())
            .expect("start_file");
        writer
            .write_all(real_data)
            .expect("write_all the real, larger data");
        // SAFETY (in the sense the zip crate itself means by this word: not
        // memory unsafety, a self-inconsistent archive): `real_data` above
        // was written and deflate-compressed by this crate's own writer, so
        // the compressed bytes on disk are genuine. This call only
        // overwrites the *declared* uncompressed size and crc recorded for
        // them afterward, on purpose, to build the one hostile shape this
        // test exists to catch: a small declared size sitting in front of
        // real compressed data that decompresses to something larger.
        unsafe {
            writer
                .set_file_metadata(lied_uncompressed_size, 0)
                .expect("set_file_metadata");
        }
        writer.finish().expect("finish").into_inner()
    }

    /// A minimal, valid, hand-built PCM WAV file: a header plus
    /// `duration_ms` of silence at 8kHz mono. Real enough for
    /// `rodio::Decoder` to accept, without depending on a checked-in binary
    /// fixture.
    fn tiny_wav(duration_ms: u32) -> Vec<u8> {
        const SAMPLE_RATE: u32 = 8000;
        const CHANNELS: u16 = 1;
        const BITS_PER_SAMPLE: u16 = 16;
        let block_align = CHANNELS * (BITS_PER_SAMPLE / 8);
        let num_samples = SAMPLE_RATE as u64 * duration_ms as u64 / 1000;
        let data_len = num_samples as u32 * block_align as u32;
        let byte_rate = SAMPLE_RATE * block_align as u32;

        let mut wav = Vec::with_capacity(44 + data_len as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&CHANNELS.to_le_bytes());
        wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        wav.extend(std::iter::repeat_n(0u8, data_len as usize));
        wav
    }

    #[test]
    fn test_a_zip_over_the_size_cap_is_refused_before_being_opened() {
        // Arbitrary bytes, not a real zip at all: proves the size gate runs
        // from the file's length alone, before anything tries to parse it
        // as an archive.
        let dir = tempfile::tempdir().expect("a temp directory");
        let oversized = vec![0u8; MAX_ZIP_BYTES as usize + 1];
        let zip_path = write_zip_bytes(dir.path(), &oversized);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "too-big", &schemes_dir);

        let err = result.expect_err("an oversized zip must be refused");
        assert!(
            err.to_string().contains("byte limit"),
            "refusal reason did not mention the byte limit: {err}"
        );
        assert!(
            !schemes_dir.join("too-big").exists(),
            "nothing should be written to disk when the import is refused"
        );
    }

    #[test]
    fn test_a_parent_dir_escape_entry_is_refused_not_written_outside_its_folder() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, MINIMAL_MANIFEST.as_bytes()),
            ("../../evil.txt", b"evil"),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "slip", &schemes_dir);

        let err = result.expect_err("a parent-dir escape entry must be refused");
        assert!(
            err.to_string().contains("evil.txt"),
            "refusal reason did not name the offending entry: {err}"
        );
        assert!(
            !dir.path().join("evil.txt").exists(),
            "the escape entry must not have been written outside the scheme's folder"
        );
        assert!(
            !schemes_dir.join("slip").exists(),
            "nothing should be written to disk when the import is refused"
        );
    }

    #[test]
    fn test_a_windows_absolute_path_entry_is_refused_not_written_outside_its_folder() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, MINIMAL_MANIFEST.as_bytes()),
            ("C:\\Windows\\evil.txt", b"evil"),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "slip-abs", &schemes_dir);

        let err = result.expect_err("an absolute-path entry must be refused");
        assert!(
            err.to_string().contains("evil.txt"),
            "refusal reason did not name the offending entry: {err}"
        );
        assert!(
            !schemes_dir.join("slip-abs").exists(),
            "nothing should be written to disk when the import is refused"
        );
    }

    #[test]
    fn test_a_declared_size_over_the_per_file_cap_is_refused_without_needing_to_decode_it() {
        // Real, honestly declared data: the zip crate's own writer computes
        // this entry's declared size from what was actually written, so
        // this is caught by the cheap metadata check alone, before
        // `import_zip` ever calls a decoder on bytes that are not even
        // real audio.
        let dir = tempfile::tempdir().expect("a temp directory");
        let oversized_sound = vec![0u8; MAX_FILE_BYTES as usize + 1];
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, MINIMAL_MANIFEST.as_bytes()),
            ("sounds/new_mail.wav", &oversized_sound),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "too-big-file", &schemes_dir);

        let err = result.expect_err("a declared size over the per-file cap must be refused");
        assert!(
            err.to_string().contains("new_mail.wav"),
            "refusal reason did not name the offending file: {err}"
        );
        assert!(!schemes_dir.join("too-big-file").exists());
    }

    #[test]
    fn test_a_lying_declared_size_does_not_hide_what_actually_decompresses() {
        // The declared size lies small (1000 bytes, comfortably under the
        // cap); the real compressed data behind it, honestly compressed by
        // this project's own zip writer, decompresses to six megabytes. A
        // check that only reads `entry.size()` would wave this through.
        let dir = tempfile::tempdir().expect("a temp directory");
        let real_data = vec![0u8; 6 * 1024 * 1024];
        let zip_bytes = build_zip_with_lying_size(
            MINIMAL_MANIFEST.as_bytes(),
            "sounds/new_mail.wav",
            &real_data,
            1000,
        );

        // Sanity-check the fixture itself before trusting what it proves:
        // the archive must still open normally, and the doctored entry's
        // declared size must really read back as the lie, or this is not
        // testing what it claims to.
        let mut sanity_check = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes.clone()))
            .expect("the hand-doctored archive must still open");
        assert_eq!(
            sanity_check.by_index(1).expect("the doctored entry").size(),
            1000
        );

        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "lying-size", &schemes_dir);

        let err =
            result.expect_err("a lying declared size must not hide a real decompression bomb");
        assert!(
            err.to_string().contains("new_mail.wav"),
            "refusal reason did not name the offending file: {err}"
        );
        assert!(!schemes_dir.join("lying-size").exists());
    }

    #[test]
    fn test_the_total_extracted_size_cap_is_refused_even_when_no_single_file_exceeds_it() {
        // Twelve files at 4.5 MB each: every one individually under the 5
        // MB per-file cap, twelve of them well over the 50 MB total cap.
        const PER_FILE: usize = 4_500_000;
        let dir = tempfile::tempdir().expect("a temp directory");
        let filler = vec![0u8; PER_FILE];
        let mut entries: Vec<(String, Vec<u8>)> = (0..12)
            .map(|i| (format!("sounds/sound_{i}.wav"), filler.clone()))
            .collect();
        entries.insert(
            0,
            (
                MANIFEST_FILE.to_string(),
                MINIMAL_MANIFEST.as_bytes().to_vec(),
            ),
        );
        let refs: Vec<(&str, &[u8])> = entries
            .iter()
            .map(|(name, data)| (name.as_str(), data.as_slice()))
            .collect();
        let zip_bytes = build_zip(&refs);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "too-much-total", &schemes_dir);

        let err = result.expect_err("exceeding the total extracted size must be refused");
        assert!(
            err.to_string().contains("total limit"),
            "refusal reason did not mention the total limit: {err}"
        );
        assert!(!schemes_dir.join("too-much-total").exists());
    }

    #[test]
    fn test_a_zip_with_no_manifest_at_all_is_refused() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let zip_bytes = build_zip(&[("readme.txt", b"just a readme, no scheme.toml here")]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "no-manifest", &schemes_dir);

        let err = result.expect_err("a zip with no scheme.toml must be refused");
        assert!(
            err.to_string().contains(MANIFEST_FILE),
            "refusal reason did not mention {MANIFEST_FILE}: {err}"
        );
        assert!(!schemes_dir.join("no-manifest").exists());
    }

    #[test]
    fn test_a_zip_with_a_malformed_manifest_is_refused() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let zip_bytes = build_zip(&[(MANIFEST_FILE, b"this is not toml at all {{{")]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let result = import_zip(&zip_path, "bad-manifest", &schemes_dir);

        assert!(
            result.is_err(),
            "a zip whose scheme.toml fails to parse must be refused"
        );
        assert!(!schemes_dir.join("bad-manifest").exists());
    }

    #[test]
    fn test_a_stray_file_outside_sounds_is_ignored_not_refused() {
        let dir = tempfile::tempdir().expect("a temp directory");
        let manifest = "name = \"Test Pack\"\n\n[sounds]\nnew_mail = \"sounds/new_mail.wav\"\n";
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, manifest.as_bytes()),
            ("sounds/new_mail.wav", &tiny_wav(200)),
            ("README.txt", b"not a sound, just documentation"),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let scheme = import_zip(&zip_path, "with-readme", &schemes_dir)
            .expect("a stray file outside sounds/ must not fail the whole import");

        assert_eq!(scheme.covers(), 1);
        assert!(!schemes_dir.join("with-readme").join("README.txt").exists());
    }

    #[test]
    fn test_a_file_that_does_not_decode_as_audio_is_dropped_and_the_rest_still_imports() {
        use super::super::feedback::Event;
        use super::super::sound_scheme::SoundSource;

        let dir = tempfile::tempdir().expect("a temp directory");
        let manifest = "name = \"Mixed Pack\"\n\n\
             [sounds]\n\
             new_mail = \"sounds/new_mail.wav\"\n\
             send_failed = \"sounds/send_failed.wav\"\n";
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, manifest.as_bytes()),
            ("sounds/new_mail.wav", &tiny_wav(200)),
            ("sounds/send_failed.wav", b"not a real wav file at all"),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let scheme = import_zip(&zip_path, "mixed", &schemes_dir)
            .expect("one bad sound file must not fail the whole import");

        // The manifest still names send_failed, so source_for still answers
        // File(path): the same dangling-reference shape
        // SoundScheme::from_manifest already produces for a file that has
        // moved or was never there, and that EarconPlayer::play_file
        // already falls back on at playback time (proved separately in
        // accessibility.rs). What this import step owns is narrower and is
        // asserted below: the bad file's bytes must never have reached
        // disk in the first place, not included as garbage that would only
        // fail later.
        let SoundSource::File(dangling) = scheme.source_for(Event::SendFailed) else {
            panic!("the manifest still names send_failed; source_for should still report a path");
        };
        assert!(
            !dangling.exists(),
            "the file that failed to decode must not have been written to disk"
        );
        assert!(matches!(
            scheme.source_for(Event::NewMail),
            SoundSource::File(_)
        ));
        assert!(
            !schemes_dir
                .join("mixed")
                .join("sounds")
                .join("send_failed.wav")
                .exists(),
            "a file that failed to decode must not be written to disk"
        );
    }

    #[test]
    fn test_a_sound_file_over_the_duration_cap_is_dropped_and_the_rest_still_imports() {
        use super::super::feedback::Event;
        use super::super::sound_scheme::SoundSource;

        let dir = tempfile::tempdir().expect("a temp directory");
        let manifest = "name = \"Mixed Pack\"\n\n\
             [sounds]\n\
             new_mail = \"sounds/new_mail.wav\"\n\
             send_failed = \"sounds/send_failed.wav\"\n";
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, manifest.as_bytes()),
            ("sounds/new_mail.wav", &tiny_wav(200)),
            ("sounds/send_failed.wav", &tiny_wav(2_500)),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let scheme = import_zip(&zip_path, "too-long", &schemes_dir)
            .expect("one over-length sound file must not fail the whole import");

        // See the matching comment in the bad-audio test above: the
        // manifest still names send_failed, so source_for still reports a
        // path. What this import step owns is that the over-length file's
        // bytes never reached disk, asserted below.
        let SoundSource::File(dangling) = scheme.source_for(Event::SendFailed) else {
            panic!("the manifest still names send_failed; source_for should still report a path");
        };
        assert!(
            !dangling.exists(),
            "a sound over the duration cap must not have been written to disk"
        );
        assert!(matches!(
            scheme.source_for(Event::NewMail),
            SoundSource::File(_)
        ));
        assert!(
            !schemes_dir
                .join("too-long")
                .join("sounds")
                .join("send_failed.wav")
                .exists(),
            "a sound over the duration cap must not be written to disk"
        );
    }

    #[test]
    fn test_an_imported_scheme_round_trips_through_resolve_and_its_files_really_play() {
        use super::super::feedback::Event;
        use super::super::sound_scheme::{SoundScheme, SoundSource};

        let dir = tempfile::tempdir().expect("a temp directory");
        let manifest = "name = \"Soft Chimes\"\n\
             author = \"Someone\"\n\
             license = \"CC0-1.0\"\n\n\
             [sounds]\n\
             new_mail = \"sounds/new_mail.wav\"\n";
        let zip_bytes = build_zip(&[
            (MANIFEST_FILE, manifest.as_bytes()),
            ("sounds/new_mail.wav", &tiny_wav(300)),
        ]);
        let zip_path = write_zip_bytes(dir.path(), &zip_bytes);
        let schemes_dir = dir.path().join("schemes");

        let imported = import_zip(&zip_path, "soft-chimes", &schemes_dir).expect("import");
        assert_eq!(imported.name, "Soft Chimes");
        assert_eq!(imported.author.as_deref(), Some("Someone"));

        // Not just that import_zip itself reports success: a completely
        // fresh read through the same path SoundScheme::resolve and
        // SoundScheme::discover already use finds the very same scheme.
        let resolved = SoundScheme::resolve("soft-chimes", &schemes_dir);
        assert_eq!(resolved, imported);
        assert!(
            SoundScheme::discover(&schemes_dir)
                .iter()
                .any(|scheme| scheme.id == "soft-chimes"),
            "the imported scheme did not turn up in discover()"
        );

        let SoundSource::File(path) = resolved.source_for(Event::NewMail) else {
            panic!("new_mail should resolve to a real file, not the fallback tone");
        };
        let on_disk = fs::read(&path).expect("the sound file must really be on disk");
        let decoded = rodio::Decoder::new(std::io::Cursor::new(on_disk))
            .expect("it must really decode as audio");
        let duration = decoded
            .total_duration()
            .expect("a wav always knows its own length");
        assert!(
            duration <= MAX_SOUND_DURATION,
            "round-tripped file plays for {duration:?}, longer than the cap it was imported under"
        );
    }
}

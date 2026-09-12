//! Which build of Wixen Mail this is.
//!
//! The version number moves when the software changes, not when a file
//! changes hands. That is the point of it, and it means several different
//! builds can share a version: three fixes on the way to 0.5.1 are all
//! 0.5.0 until 0.5.1 is cut.
//!
//! Which leaves a question somebody has to be able to answer. A bug report
//! arrives with a log saying "Starting Wixen Mail v0.5.0", and if six builds
//! said that, the report cannot be matched to the code it came from. So a
//! build made by `scripts/build-installer.sh` carries the commit it was built
//! from, and says so everywhere the version appears.
//!
//! Ordinary `cargo build` gets nothing extra, and that is deliberate: reading
//! the commit at compile time would mean relinking after every `git add`, for
//! a label nobody looks at in a development build. The script sets
//! `WIXEN_BUILD` and `build.rs` passes it through, so only the builds that are
//! handed to somebody pay for it.

use std::cmp::Ordering;

/// The version from `Cargo.toml`.
const NUMBER: &str = env!("CARGO_PKG_VERSION");

/// The commit this was built from, empty unless the installer script built it.
const BUILD: &str = env!("WIXEN_BUILD");

/// What separates a version from its build identifier.
///
/// One constant rather than the character written in two places. [`describe`]
/// joins on it and [`without_build`] splits on it, and the two are inverses,
/// so neither can be written in terms of the other and a literal in both is
/// one rule kept in two places.
const BUILD_SEPARATOR: char = '+';

/// What separates a version from the prerelease staging it.
///
/// Deliberately not [`BUILD_SEPARATOR`]: a build identifier put after this
/// would make every build a prerelease of the version it was built from and
/// sort them all below it.
const PRERELEASE_SEPARATOR: char = '-';

/// What a published tag carries in front of the version.
///
/// `cargo release` writes the tag, `.github/workflows/release.yml:115` names
/// the portable download after it, and `:133` publishes that download under
/// the glob `wixen-mail-v*.exe`. So the strings a release feed hands back
/// start with this and the number this crate is built from does not, and both
/// have to read as the same version.
const TAG_PREFIX: &str = "v";

/// What to call this build, wherever a person or a log will see it.
///
/// `0.5.0` from a plain build, `0.5.0+g64c73dd` from one the installer script
/// made. The `+` is deliberate: everything after it is build metadata, which
/// version ordering ignores, so two builds of the same version stay equal
/// while remaining tellable apart.
pub fn current() -> String {
    describe(NUMBER, BUILD)
}

/// Join a version and a build identifier.
///
/// Split out from [`current`] so it can be tested. The two values it reads are
/// fixed at compile time, so a test of `current` could only ever check one of
/// the two cases, and it would be whichever case the test run happened to be
/// built in.
fn describe(number: &str, build: &str) -> String {
    if build.is_empty() {
        number.to_string()
    } else {
        format!("{number}{BUILD_SEPARATOR}{build}")
    }
}

/// A version with its build identifier taken off.
///
/// Everything after the separator is build metadata, which version ordering
/// ignores, so two builds of one version stay equal. A separator with nothing
/// after it is neither a build identifier nor a version, so it is refused
/// rather than quietly read as the number in front of it.
fn without_build(version: &str) -> Option<&str> {
    match version.split_once(BUILD_SEPARATOR) {
        None => Some(version),
        Some((number, build)) if !build.is_empty() => Some(number),
        Some(_) => None,
    }
}

/// Where a version sits between its own prereleases and itself.
///
/// The order is the declaration order. `scripts/build-installer.sh` reached
/// the same one independently when it had to squeeze a prerelease into the
/// four numbers Windows shows, and this is where the two have to agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Alpha,
    Beta,
    Rc,
    /// Not a prerelease at all, which is why it is last: a release is newer
    /// than every prerelease that staged it.
    Release,
}

/// The three words `.github/workflows/release.yml` can put in a tag.
///
/// Anything else is refused rather than being given a position. A prerelease
/// spelling this project does not produce is a string somebody else chose,
/// and the thing downstream of this answer downloads an executable.
fn stage_named(word: &str) -> Option<Stage> {
    match word {
        "alpha" => Some(Stage::Alpha),
        "beta" => Some(Stage::Beta),
        "rc" => Some(Stage::Rc),
        _ => None,
    }
}

/// One field of a version, which is a run of digits and nothing else.
///
/// Not `str::parse` on its own, which accepts a leading plus and would read
/// `+5` as five in a string where a plus already means something else.
/// Refuses a number too large to hold rather than saturating it.
fn whole_number(field: &str) -> Option<u64> {
    if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    field.parse().ok()
}

/// A version string this program can put in order.
///
/// Field order is comparison order, which is what the derived `Ord` reads, so
/// the three numbers decide first and the prerelease decides the ties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    stage: Stage,
    /// The counter after the prerelease word, held as a number so that
    /// `alpha.10` is newer than `alpha.9` rather than sorting before it the
    /// way the text does. Zero for a release, which carries no counter and
    /// needs none, because `stage` has already put it above every prerelease
    /// of the same three numbers.
    step: u64,
}

/// Read a version string, or answer nothing if it is not one.
///
/// Total: no input makes this panic, overflow or hang, so [`compare`] has no
/// error to flow anywhere and every caller gets an answer. What it accepts is
/// what this project publishes and nothing more, because a string guessed at
/// is a string that can be guessed wrong.
fn parse(version: &str) -> Option<Version> {
    let named = version.strip_prefix(TAG_PREFIX).unwrap_or(version);
    let ordered = without_build(named)?;
    let (numbers, prerelease) = match ordered.split_once(PRERELEASE_SEPARATOR) {
        Some((numbers, prerelease)) => (numbers, Some(prerelease)),
        None => (ordered, None),
    };

    let mut fields = numbers.split('.');
    let major = whole_number(fields.next()?)?;
    let minor = whole_number(fields.next()?)?;
    let patch = whole_number(fields.next()?)?;
    if fields.next().is_some() {
        return None;
    }

    let (stage, step) = match prerelease {
        None => (Stage::Release, 0),
        Some(prerelease) => {
            let (word, counter) = prerelease.split_once('.')?;
            (stage_named(word)?, whole_number(counter)?)
        }
    };

    Some(Version {
        major,
        minor,
        patch,
        stage,
        step,
    })
}

/// How one version compares with another.
///
/// Four answers rather than three. "I could not read that" is not "older", and
/// a caller that has only a `bool` to read the answer out of has to decide
/// which of the two it means. Reading it as older tells everybody there is a
/// new version every time a server returns something unexpected, so the
/// unreadable case is its own answer and every caller has to say what it does
/// about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compared {
    /// The first version is newer than the second.
    Newer,
    /// The two are the same version. They may still be different builds of it.
    Same,
    /// The first version is older than the second.
    Older,
    /// One of the two is not a version this program can read.
    CouldNotRead,
}

/// Whether `version` is newer than, the same as, or older than `with`.
///
/// Both arguments are version strings as this project writes them: three
/// numbers, an optional prerelease that stages them, and an optional build
/// identifier that plays no part in the order.
pub fn compare(version: &str, with: &str) -> Compared {
    let (Some(left), Some(right)) = (parse(version), parse(with)) else {
        return Compared::CouldNotRead;
    };
    match left.cmp(&right) {
        Ordering::Greater => Compared::Newer,
        Ordering::Equal => Compared::Same,
        Ordering::Less => Compared::Older,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_later_patch_minor_or_major_is_newer() {
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
        assert_eq!(compare("0.6.0", "0.5.9"), Compared::Newer);
        assert_eq!(compare("1.0.0", "0.99.0"), Compared::Newer);
        // The other direction, so a comparison that answers Newer for
        // everything cannot pass this.
        assert_eq!(compare("0.5.0", "0.5.1"), Compared::Older);
        assert_eq!(compare("0.99.0", "1.0.0"), Compared::Older);
    }

    #[test]
    fn test_the_same_version_is_neither_newer_nor_older() {
        assert_eq!(compare("0.5.0", "0.5.0"), Compared::Same);
    }

    #[test]
    fn test_a_build_identifier_after_a_plus_is_not_part_of_the_version() {
        // Both directions on purpose. A naive string comparison answers this
        // correctly in one direction, because the longer string sorts after,
        // and gets the other one backwards.
        assert_eq!(compare("0.5.0+g64c73dd", "0.5.0"), Compared::Same);
        assert_eq!(compare("0.5.0", "0.5.0+g64c73dd"), Compared::Same);
    }

    #[test]
    fn test_two_builds_of_one_version_are_the_same_version() {
        assert_eq!(compare("0.5.0+g64c73dd", "0.5.0+gaaaaaaa"), Compared::Same);
        assert_eq!(compare("0.5.0+gaaaaaaa", "0.5.0+g64c73dd"), Compared::Same);
    }

    #[test]
    fn test_a_release_is_newer_than_the_prerelease_that_staged_it() {
        // A prerelease stages the release rather than following it, so the
        // suffix sorts the version below the one it is a suffix of.
        assert_eq!(compare("0.6.0", "0.6.0-alpha.1"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.1", "0.6.0"), Compared::Older);
    }

    #[test]
    fn test_the_three_prerelease_words_sort_in_the_order_the_workflow_makes_them() {
        // alpha, beta and rc are the three levels
        // `.github/workflows/release.yml` offers, in that order.
        assert_eq!(compare("0.6.0-beta.1", "0.6.0-alpha.2"), Compared::Newer);
        assert_eq!(compare("0.6.0-rc.1", "0.6.0-beta.9"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.2", "0.6.0-beta.1"), Compared::Older);
    }

    #[test]
    fn test_the_prerelease_counter_is_compared_as_a_number() {
        // As text, "10" sorts before "9".
        assert_eq!(compare("0.6.0-alpha.10", "0.6.0-alpha.9"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.9", "0.6.0-alpha.10"), Compared::Older);
    }

    #[test]
    fn test_a_published_tag_carries_a_v_and_is_still_a_version() {
        // What gets published is a tag rather than a bare version.
        // `.github/workflows/release.yml:115` names the portable copy after
        // the tag and `:133` publishes it with the glob `wixen-mail-v*.exe`,
        // so the tag starts with a `v` and that is the string a release feed
        // hands back. Refusing it would answer CouldNotRead for every real
        // release this project will ever cut.
        assert_eq!(compare("v0.5.1", "0.5.0"), Compared::Newer);
        assert_eq!(compare("v0.5.0", "v0.5.0"), Compared::Same);
        assert_eq!(compare("v0.6.0", "v0.6.0-alpha.1"), Compared::Newer);
    }

    #[test]
    fn test_a_version_it_cannot_read_is_refused_rather_than_treated_as_very_old() {
        // Paired with a readable case, so a comparison answering CouldNotRead
        // for everything could not pass this.
        assert_eq!(compare("banana", "0.5.0"), Compared::CouldNotRead);
        assert_eq!(compare("0.5.0", "banana"), Compared::CouldNotRead);
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
    }

    #[test]
    fn test_a_string_that_is_not_a_version_is_refused_rather_than_crashing() {
        // Every string below is one a server could return and none of them is
        // a version, so what this asserts is that the answer is an answer
        // rather than a crash, an overflow or a hang.
        for odd in [
            "",
            " ",
            "0",
            "0.5",
            "0.5.0.0",
            "-1.0.0",
            "0.5.0-",
            "0.5.0-alpha",
            "0.5.0-alpha.",
            "0.5.0+",
            "+",
            "...",
            "99999999999999999999999.0.0",
            "0.5.0-alpha.99999999999999999999999",
            "\u{1f600}",
            "0.5.0\n",
        ] {
            assert_eq!(compare(odd, "0.5.0"), Compared::CouldNotRead, "{odd:?}");
            assert_eq!(compare("0.5.0", odd), Compared::CouldNotRead, "{odd:?}");
        }
        // Paired with the ordinary case the whole module is for. Every
        // assertion above is satisfied by a comparison that refuses
        // everything, and that comparison is the one this pairing catches.
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
    }

    #[test]
    fn test_a_plain_build_is_just_the_number() {
        assert_eq!(describe("0.5.0", ""), "0.5.0");
    }

    #[test]
    fn test_a_build_from_the_installer_script_says_which_commit() {
        // The whole reason this exists: a log line that can be matched to the
        // code it came from when several builds share a version.
        assert_eq!(describe("0.5.0", "g64c73dd"), "0.5.0+g64c73dd");
    }

    #[test]
    fn test_the_build_is_metadata_rather_than_part_of_the_version() {
        // After a `+`, which is what makes version ordering ignore it. Putting
        // it after a `-` would make every build a prerelease of the version it
        // was built from, and sort them below it.
        let described = describe("0.5.0", "g64c73dd");

        assert!(described.starts_with("0.5.0+"), "{described}");
        assert!(
            !described.contains("0.5.0-"),
            "a build identifier must not read as a prerelease: {described}"
        );
    }

    #[test]
    fn test_this_build_says_something() {
        // Guards against `WIXEN_BUILD` not being set at all, which would stop
        // the crate compiling, and against a version that came out empty.
        assert!(current().starts_with(NUMBER), "{}", current());
        assert!(!NUMBER.is_empty());
    }
}

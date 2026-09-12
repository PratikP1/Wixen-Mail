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

/// The version from `Cargo.toml`.
const NUMBER: &str = env!("CARGO_PKG_VERSION");

/// The commit this was built from, empty unless the installer script built it.
const BUILD: &str = env!("WIXEN_BUILD");

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
        format!("{number}+{build}")
    }
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
pub fn compare(_version: &str, _with: &str) -> Compared {
    Compared::CouldNotRead
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

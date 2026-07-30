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

#[cfg(test)]
mod tests {
    use super::*;

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

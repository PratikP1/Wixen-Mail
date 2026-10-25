//! The lock holds the three wxDragon crates at one version.
//!
//! `Cargo.toml` pins wxdragon exactly, and that pin holds one crate of three.
//! wxdragon asks for its two siblings, wxdragon-sys and wxdragon-macros, with
//! a caret, so only `Cargo.lock` says which of them is built. On 2026-10-20 a
//! dry-run update at the old pin moved both siblings to 0.9.23 while wxdragon
//! stayed at 0.9.17 (phase 13.4's research). The three are released together
//! and the wrapper calls the C++ shim by the names its own release declares,
//! so a lock holding them at two versions is a build nobody tested, and
//! nothing in cargo refuses it.
//!
//! Reads the lock as TOML and touches no wxWidgets API, so it opens no window.

/// The three crates wxDragon is released as, in the order a refusal names them.
const THE_THREE: [&str; 3] = ["wxdragon", "wxdragon-sys", "wxdragon-macros"];

/// Why `lock` must be refused, or `None` when it holds each of the three
/// wxDragon crates once and all at one version.
fn why_the_lock_is_refused(lock: &str) -> Option<String> {
    // The red half: every lock is accepted.
    let _ = (lock, THE_THREE);
    None
}

/// A lock holding the three crates at the versions given, beside one
/// package that is nothing to do with them.
fn a_lock_with(versions: &[(&str, &str)]) -> String {
    let mut lock =
        String::from("version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.228\"\n");
    for (name, version) in versions {
        lock.push_str(&format!(
            "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n"
        ));
    }
    lock
}

#[test]
fn test_the_real_lock_holds_the_wxdragon_crates_at_one_version() {
    let lock = std::fs::read_to_string("Cargo.lock").expect("Cargo.lock should be readable");
    assert_eq!(why_the_lock_is_refused(&lock), None);
}

#[test]
fn test_a_lock_with_the_wxdragon_crates_at_two_versions_is_refused() {
    // The shape a careless update makes: the pinned crate left behind, its
    // two siblings moved on.
    let planted = a_lock_with(&[
        ("wxdragon", "0.9.17"),
        ("wxdragon-sys", "0.9.23"),
        ("wxdragon-macros", "0.9.23"),
    ]);
    let refused = why_the_lock_is_refused(&planted);
    assert_eq!(
        refused.as_deref(),
        Some(
            "the lock holds wxdragon 0.9.17, wxdragon-sys 0.9.23, wxdragon-macros 0.9.23, \
             and the three are built from one release or not at all"
        )
    );
}

#[test]
fn test_a_lock_missing_a_wxdragon_crate_is_refused() {
    let planted = a_lock_with(&[("wxdragon", "0.9.23"), ("wxdragon-sys", "0.9.23")]);
    assert_eq!(
        why_the_lock_is_refused(&planted).as_deref(),
        Some("the lock holds no wxdragon-macros")
    );
}

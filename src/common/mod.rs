//! Common types and utilities used across all layers

/// A loopback server tests point a provider client at.
#[cfg(test)]
pub mod answering;
pub mod error;
pub mod logging;
/// The shapes a stored moment takes, read here rather than listed again in
/// every module that reads one.
pub mod moment;
pub mod paths;
/// A value and the temporary folder it lives in, removed together.
#[cfg(test)]
pub mod temp_home;
pub mod types;
pub mod version;
/// The half of a source file a release build compiles.
///
/// On for dev and test builds, absent from every shipped binary. The feature
/// it is gated on is turned on by a dev-dependency of this package on itself
/// and by nothing else, so a release build compiles none of this and no flag
/// has to be remembered at release time. A feature rather than a plain
/// `pub mod` because a source-slicer in the library's public surface invites
/// uses it was never meant for.
///
/// One condition rather than two, and that is deliberate. `#[cfg(test)]`, what
/// this used to be, is on for the library's own unit tests and off for the
/// library an integration test links, so the sixteen checks in
/// `tests/wired.rs` that need this answer could not reach it and cut the file
/// themselves. Keeping the test gate as well would leave the unit tests green
/// while the integration tests failed to compile, which is a split failure
/// where a uniform one is wanted.
#[cfg(feature = "what-ships")]
pub mod what_ships;

pub use error::{Error, Result};

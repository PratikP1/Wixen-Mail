//! Common types and utilities used across all layers

/// A loopback server tests point a provider client at.
#[cfg(test)]
pub mod answering;
/// The sentences this program speaks, out of a translation catalogue.
///
/// Here for the reason `how_the_machine_writes_dates` is: `presentation`,
/// `application` and `service` will all ask it for a sentence, and `common`
/// imports from none of them.
pub mod catalogue;
pub mod error;
/// How this computer writes a date, asked rather than assumed.
///
/// Here rather than beside the other date code in `presentation`, because
/// `service::signed_mail` writes a date into a sentence somebody hears and
/// `service` reaches `presentation` nowhere. `common` imports nothing from the
/// other three layers, which is the same reason `moment` is here.
pub mod how_the_machine_writes_dates;
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

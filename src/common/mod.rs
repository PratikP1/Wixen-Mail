//! Common types and utilities used across all layers

/// A loopback server tests point a provider client at.
#[cfg(test)]
pub mod answering;
pub mod error;
pub mod logging;
pub mod paths;
pub mod types;
pub mod version;

pub use error::{Error, Result};

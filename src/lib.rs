//! Wixen Mail - A fully accessible mail client
//!
//! This crate provides the core functionality for Wixen Mail, organized into
//! four main layers: presentation, application, service, and data.

// `menu_ids!` in `presentation::wx_app` numbers the menu and control
// identifiers by recursing once per name, and there are 128 of them. The
// default limit is 128, so the list stopped compiling on the one added for a
// saved search's conditions. Raised rather than worked around: the numbering
// is deliberately not something a person does by hand, because two ids were
// once written with the same offset and wxWidgets resolved the duplicate by
// checking the wrong menu item, and splitting the list into two calls to stay
// under a limit would put that back within reach.
#![recursion_limit = "512"]

// Presentation Layer
pub mod presentation;

// Application Layer
pub mod application;

// Service Layer
pub mod service;
pub mod vendor;

// Data Layer
pub mod data;

// Common types and utilities
pub mod common;

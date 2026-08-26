//! Data layer - Database, storage, and configuration
//!
//! This layer handles all data persistence and configuration management.

pub mod account;
pub mod config;
pub mod email_providers;
pub mod message_cache;

pub use account::Account;
pub use config::ConfigManager;
pub use email_providers::*;
pub use message_cache::MessageCache;

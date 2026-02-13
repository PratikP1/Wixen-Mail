//! Data layer - Database, storage, and configuration
//!
//! This layer handles all data persistence and configuration management.

pub mod config;
pub mod database;
pub mod email_providers;
pub mod message_cache;
pub mod storage;

pub use config::ConfigManager;
pub use database::Database;
pub use email_providers::*;
pub use message_cache::MessageCache;
pub use storage::Storage;

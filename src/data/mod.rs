//! Data layer - Database, storage, and configuration
//!
//! This layer handles all data persistence and configuration management.

pub mod config;
pub mod database;
pub mod storage;

pub use config::ConfigManager;
pub use database::Database;
pub use storage::Storage;

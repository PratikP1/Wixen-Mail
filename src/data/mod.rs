//! Data layer - Database, storage, and configuration
//!
//! This layer handles all data persistence and configuration management.

pub mod database;
pub mod storage;
pub mod config;

pub use database::Database;
pub use storage::Storage;
pub use config::ConfigManager;

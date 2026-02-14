//! Database management
//!
//! Handles SQLite database operations for storing messages and metadata.

use crate::common::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database connection and operations
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database instance
    pub fn new() -> Result<Self> {
        let db_path = dirs::data_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("wixen-mail")
            .join("core.db");
        let conn = Self::open_connection(&db_path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn open_connection(path: &PathBuf) -> Result<Connection> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Connection::open(path)
            .map_err(|e| crate::common::Error::Database(format!("Failed to open database: {}", e)))
    }

    /// Initialize database schema
    pub fn initialize(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| crate::common::Error::Database("Failed to lock database connection".to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .map_err(|e| crate::common::Error::Database(format!("Failed to initialize schema: {}", e)))?;
        Ok(())
    }

    /// Execute a query
    pub fn execute(&self, query: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| crate::common::Error::Database("Failed to lock database connection".to_string()))?;
        conn.execute_batch(query)
            .map_err(|e| crate::common::Error::Database(format!("Failed to execute query: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new();
        assert!(db.is_ok());
    }

    #[test]
    fn test_database_initialize_and_execute() {
        let db = Database::new().unwrap();
        db.initialize().unwrap();
        db.execute("INSERT OR REPLACE INTO app_metadata (key, value) VALUES ('theme', 'dark');")
            .unwrap();
    }
}

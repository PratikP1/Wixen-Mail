//! File system storage
//!
//! Manages file system operations for storing attachments and cache.

use crate::common::Result;
use std::fs;
use std::path::PathBuf;
use std::path::{Component, Path};

/// File system storage manager
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(base_path: PathBuf) -> Result<Self> {
        Ok(Self { base_path })
    }

    /// Get the base storage path
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Write data to a file
    pub fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, data)?;
        Ok(())
    }

    /// Read data from a file
    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path)?;
        let data = fs::read(full_path)?;
        Ok(data)
    }

    /// Delete a file
    pub fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        if full_path.exists() {
            fs::remove_file(full_path)?;
        }
        Ok(())
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let relative = Path::new(path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(crate::common::Error::Other(
                "Invalid storage path".to_string(),
            ));
        }
        Ok(self.base_path.join(relative))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_storage_creation() {
        let path = PathBuf::from("/tmp/wixen-mail");
        let storage = Storage::new(path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_storage_write_read_delete() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("wixen_storage_test_{}", nanos));
        let storage = Storage::new(base).unwrap();

        storage.write("attachments/file.txt", b"hello").unwrap();
        let content = storage.read("attachments/file.txt").unwrap();
        assert_eq!(content, b"hello");
        storage.delete("attachments/file.txt").unwrap();
        assert!(storage.read("attachments/file.txt").is_err());
    }

    #[test]
    fn test_storage_rejects_parent_dir_paths() {
        let base = std::env::temp_dir().join("wixen_storage_test_reject");
        let storage = Storage::new(base).unwrap();
        assert!(storage.write("../escape.txt", b"x").is_err());
    }
}

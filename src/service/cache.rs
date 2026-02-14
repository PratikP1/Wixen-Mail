//! Cache service
//!
//! Manages caching of messages and attachments.

use crate::common::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Cache service for storing message data
pub struct CacheService {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl CacheService {
    /// Create a new cache service
    pub fn new() -> Result<Self> {
        Ok(Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Store data in cache
    pub fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut guard = self
            .data
            .lock()
            .map_err(|_| crate::common::Error::Other("Cache lock poisoned".to_string()))?;
        guard.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    /// Retrieve data from cache
    pub fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let guard = self
            .data
            .lock()
            .map_err(|_| crate::common::Error::Other("Cache lock poisoned".to_string()))?;
        Ok(guard.get(key).cloned())
    }

    /// Clear cache
    pub fn clear(&self) -> Result<()> {
        let mut guard = self
            .data
            .lock()
            .map_err(|_| crate::common::Error::Other("Cache lock poisoned".to_string()))?;
        guard.clear();
        Ok(())
    }
}

impl Default for CacheService {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_service_creation() {
        let service = CacheService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_cache_store_retrieve_clear() {
        let service = CacheService::new().unwrap();
        service.store("k1", b"v1").unwrap();
        assert_eq!(service.retrieve("k1").unwrap(), Some(b"v1".to_vec()));
        service.clear().unwrap();
        assert_eq!(service.retrieve("k1").unwrap(), None);
    }
}

//! Search engine
//!
//! Provides full-text search and filtering capabilities.

use crate::common::Result;
use std::sync::{Arc, RwLock};

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub folder: Option<String>,
}

/// Search engine for messages
pub struct SearchEngine {
    indexed_items: Arc<RwLock<Vec<SearchItem>>>,
}

#[derive(Debug, Clone)]
struct SearchItem {
    folder: Option<String>,
    text: String,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Result<Self> {
        Ok(Self {
            indexed_items: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Add content to local search index
    pub fn index_text(&self, folder: Option<String>, text: String) -> Result<()> {
        let mut items = self
            .indexed_items
            .write()
            .map_err(|_| crate::common::Error::Other("Search index lock poisoned".to_string()))?;
        items.push(SearchItem { folder, text });
        Ok(())
    }

    /// Search for messages
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<String>> {
        let needle = query.text.trim().to_lowercase();
        if needle.is_empty() {
            return Ok(Vec::new());
        }
        let items = self
            .indexed_items
            .read()
            .map_err(|_| crate::common::Error::Other("Search index lock poisoned".to_string()))?;
        let results = items
            .iter()
            .filter(|item| {
                if let Some(filter_folder) = &query.folder {
                    if item.folder.as_ref() != Some(filter_folder) {
                        return false;
                    }
                }
                item.text.to_lowercase().contains(&needle)
            })
            .map(|item| item.text.clone())
            .collect();
        Ok(results)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            indexed_items: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_search_index_and_query() {
        let engine = SearchEngine::new().unwrap();
        engine
            .index_text(Some("INBOX".to_string()), "Invoice from vendor".to_string())
            .unwrap();
        engine
            .index_text(Some("Sent".to_string()), "Follow up note".to_string())
            .unwrap();

        let q = SearchQuery {
            text: "invoice".to_string(),
            folder: Some("INBOX".to_string()),
        };
        let results = engine.search(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("Invoice"));
    }
}

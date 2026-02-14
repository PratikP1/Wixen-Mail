//! Keyboard handler for shortcuts and navigation

use crate::common::Result;
use std::collections::HashMap;
use std::sync::Mutex;

/// Handles keyboard shortcuts and navigation
pub struct KeyboardHandler {
    shortcuts: Mutex<HashMap<String, String>>,
}

impl KeyboardHandler {
    /// Create a new keyboard handler
    pub fn new() -> Result<Self> {
        Ok(Self {
            shortcuts: Mutex::new(HashMap::new()),
        })
    }

    /// Register a keyboard shortcut
    pub fn register_shortcut(&self, key: &str, action: &str) -> Result<()> {
        let mut map = self
            .shortcuts
            .lock()
            .map_err(|_| crate::common::Error::Other("Shortcut map lock poisoned".to_string()))?;
        map.insert(key.to_string(), action.to_string());
        Ok(())
    }

    /// Resolve shortcut action
    pub fn action_for_key(&self, key: &str) -> Result<Option<String>> {
        let map = self
            .shortcuts
            .lock()
            .map_err(|_| crate::common::Error::Other("Shortcut map lock poisoned".to_string()))?;
        Ok(map.get(key).cloned())
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self {
            shortcuts: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup_shortcut() {
        let handler = KeyboardHandler::new().unwrap();
        handler.register_shortcut("Ctrl+N", "compose").unwrap();
        assert_eq!(handler.action_for_key("Ctrl+N").unwrap().as_deref(), Some("compose"));
    }
}

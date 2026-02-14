//! Focus manager for tracking and managing UI focus

use crate::common::Result;
use std::sync::Mutex;

/// Manages UI focus order and trapping
pub struct FocusManager {
    current_focus: Mutex<Option<String>>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_focus: Mutex::new(None),
        })
    }

    /// Set focus to an element
    pub fn set_focus(&self, element_id: &str) -> Result<()> {
        let mut focus = self
            .current_focus
            .lock()
            .map_err(|_| crate::common::Error::Other("Focus lock poisoned".to_string()))?;
        *focus = Some(element_id.to_string());
        Ok(())
    }

    /// Get current focused element id
    pub fn current_focus(&self) -> Result<Option<String>> {
        let focus = self
            .current_focus
            .lock()
            .map_err(|_| crate::common::Error::Other("Focus lock poisoned".to_string()))?;
        Ok(focus.clone())
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self {
            current_focus: Mutex::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_focus() {
        let manager = FocusManager::new().unwrap();
        manager.set_focus("message_list").unwrap();
        assert_eq!(
            manager.current_focus().unwrap().as_deref(),
            Some("message_list")
        );
    }
}

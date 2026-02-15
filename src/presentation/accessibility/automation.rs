//! Accessibility automation tree and event models.

use crate::common::Result;
use std::collections::HashMap;
use std::sync::RwLock;

/// Semantic role for an accessibility node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationRole {
    Window,
    Pane,
    Button,
    Text,
    TextInput,
    List,
    ListItem,
    Menu,
    MenuItem,
    Checkbox,
    Link,
    Custom(String),
}

/// Accessibility state for a node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutomationState {
    pub focused: bool,
    pub enabled: bool,
    pub selected: bool,
    pub expanded: bool,
    pub checked: Option<bool>,
}

/// Node in automation tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub role: AutomationRole,
    pub name: String,
    pub description: Option<String>,
    pub state: AutomationState,
}

impl AutomationNode {
    pub fn new(id: impl Into<String>, role: AutomationRole, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            parent_id: None,
            role,
            name: name.into(),
            description: None,
            state: AutomationState {
                enabled: true,
                ..AutomationState::default()
            },
        }
    }
}

/// Accessibility automation events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationEvent {
    NodeAdded(String),
    NodeUpdated(String),
    FocusChanged(String),
    LiveRegion(String, String),
}

/// Thread-safe automation store.
pub struct AutomationStore {
    nodes: RwLock<HashMap<String, AutomationNode>>,
}

impl AutomationStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            nodes: RwLock::new(HashMap::new()),
        })
    }

    pub fn upsert_node(&self, node: AutomationNode) -> Result<()> {
        let mut nodes = self.nodes.write().map_err(|_| {
            crate::common::Error::Other("Automation tree lock poisoned".to_string())
        })?;
        nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn update_state(&self, node_id: &str, state: AutomationState) -> Result<()> {
        let mut nodes = self.nodes.write().map_err(|_| {
            crate::common::Error::Other("Automation tree lock poisoned".to_string())
        })?;
        if let Some(node) = nodes.get_mut(node_id) {
            node.state = state;
        }
        Ok(())
    }

    pub fn get_node(&self, node_id: &str) -> Result<Option<AutomationNode>> {
        let nodes = self.nodes.read().map_err(|_| {
            crate::common::Error::Other("Automation tree lock poisoned".to_string())
        })?;
        Ok(nodes.get(node_id).cloned())
    }

    pub fn snapshot(&self) -> Result<Vec<AutomationNode>> {
        let nodes = self.nodes.read().map_err(|_| {
            crate::common::Error::Other("Automation tree lock poisoned".to_string())
        })?;
        Ok(nodes.values().cloned().collect())
    }
}

impl Default for AutomationStore {
    fn default() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_get_node() {
        let store = AutomationStore::new().unwrap();
        store
            .upsert_node(AutomationNode::new(
                "compose_button",
                AutomationRole::Button,
                "Compose",
            ))
            .unwrap();
        let node = store.get_node("compose_button").unwrap().unwrap();
        assert_eq!(node.name, "Compose");
    }
}

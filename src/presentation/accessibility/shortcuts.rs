//! Keyboard shortcuts management
//!
//! Defines and manages keyboard shortcuts for accessibility.

use std::collections::HashMap;
use std::fmt;

/// Modifier keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Shift => write!(f, "Shift"),
            Modifier::Meta => write!(f, "Meta"),
        }
    }
}

/// Key codes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Character(char),
    FunctionKey(u8), // F1-F12
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Character(c) => write!(f, "{}", c.to_uppercase()),
            Key::FunctionKey(n) => write!(f, "F{}", n),
            Key::Enter => write!(f, "Enter"),
            Key::Escape => write!(f, "Escape"),
            Key::Tab => write!(f, "Tab"),
            Key::Space => write!(f, "Space"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::ArrowUp => write!(f, "Up"),
            Key::ArrowDown => write!(f, "Down"),
            Key::ArrowLeft => write!(f, "Left"),
            Key::ArrowRight => write!(f, "Right"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
        }
    }
}

/// Keyboard shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardShortcut {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

impl KeyboardShortcut {
    /// Create a new keyboard shortcut
    pub fn new(modifiers: Vec<Modifier>, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Create a simple shortcut with one modifier
    pub fn simple(modifier: Modifier, key: Key) -> Self {
        Self::new(vec![modifier], key)
    }
}

impl fmt::Display for KeyboardShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, modifier) in self.modifiers.iter().enumerate() {
            if i > 0 {
                write!(f, "+")?;
            }
            write!(f, "{}", modifier)?;
        }
        if !self.modifiers.is_empty() {
            write!(f, "+")?;
        }
        write!(f, "{}", self.key)
    }
}

/// Action that can be triggered by a keyboard shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    // Application actions
    Quit,
    OpenSettings,
    OpenHelp,
    OpenKeyboardShortcuts,

    // Window navigation
    CyclePanes,
    CyclePanesReverse,
    CloseWindow,

    // Message actions
    NewMessage,
    Reply,
    ReplyAll,
    Forward,
    Delete,
    MarkAsRead,
    MarkAsUnread,
    Star,
    Archive,
    MoveToFolder,
    AddTag,

    // Navigation
    NextMessage,
    PreviousMessage,
    NextUnread,
    PreviousUnread,
    JumpToFolder,

    // Composition
    Send,
    SaveDraft,
    AddAttachment,
    ToggleBold,
    ToggleItalic,
    ToggleUnderline,

    // Search
    Search,
    AdvancedSearch,
    FindNext,
    FindPrevious,

    // Folder management
    NewFolder,
    RenameFolder,
    DeleteFolder,

    // Account management
    CheckMail,
    SwitchAccount,

    // Custom action
    Custom(String),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Quit => write!(f, "Quit"),
            Action::OpenSettings => write!(f, "Open Settings"),
            Action::OpenHelp => write!(f, "Open Help"),
            Action::OpenKeyboardShortcuts => write!(f, "Open Keyboard Shortcuts"),
            Action::CyclePanes => write!(f, "Cycle Panes"),
            Action::CyclePanesReverse => write!(f, "Cycle Panes Reverse"),
            Action::CloseWindow => write!(f, "Close Window"),
            Action::NewMessage => write!(f, "New Message"),
            Action::Reply => write!(f, "Reply"),
            Action::ReplyAll => write!(f, "Reply All"),
            Action::Forward => write!(f, "Forward"),
            Action::Delete => write!(f, "Delete"),
            Action::MarkAsRead => write!(f, "Mark as Read"),
            Action::MarkAsUnread => write!(f, "Mark as Unread"),
            Action::Star => write!(f, "Star"),
            Action::Archive => write!(f, "Archive"),
            Action::MoveToFolder => write!(f, "Move to Folder"),
            Action::AddTag => write!(f, "Add Tag"),
            Action::NextMessage => write!(f, "Next Message"),
            Action::PreviousMessage => write!(f, "Previous Message"),
            Action::NextUnread => write!(f, "Next Unread"),
            Action::PreviousUnread => write!(f, "Previous Unread"),
            Action::JumpToFolder => write!(f, "Jump to Folder"),
            Action::Send => write!(f, "Send"),
            Action::SaveDraft => write!(f, "Save Draft"),
            Action::AddAttachment => write!(f, "Add Attachment"),
            Action::ToggleBold => write!(f, "Toggle Bold"),
            Action::ToggleItalic => write!(f, "Toggle Italic"),
            Action::ToggleUnderline => write!(f, "Toggle Underline"),
            Action::Search => write!(f, "Search"),
            Action::AdvancedSearch => write!(f, "Advanced Search"),
            Action::FindNext => write!(f, "Find Next"),
            Action::FindPrevious => write!(f, "Find Previous"),
            Action::NewFolder => write!(f, "New Folder"),
            Action::RenameFolder => write!(f, "Rename Folder"),
            Action::DeleteFolder => write!(f, "Delete Folder"),
            Action::CheckMail => write!(f, "Check Mail"),
            Action::SwitchAccount => write!(f, "Switch Account"),
            Action::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Keyboard shortcut manager
pub struct ShortcutManager {
    shortcuts: HashMap<KeyboardShortcut, Action>,
}

impl ShortcutManager {
    /// Create a new shortcut manager with default shortcuts
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: HashMap::new(),
        };
        manager.load_defaults();
        manager
    }

    /// Load default keyboard shortcuts
    fn load_defaults(&mut self) {
        // Application shortcuts
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('q')),
            Action::Quit,
        );
        self.register(
            KeyboardShortcut::new(vec![Modifier::Ctrl], Key::Character(',')),
            Action::OpenSettings,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::FunctionKey(1)),
            Action::OpenHelp,
        );

        // Window navigation
        self.register(
            KeyboardShortcut::new(vec![], Key::FunctionKey(6)),
            Action::CyclePanes,
        );

        // Message actions
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('n')),
            Action::NewMessage,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('r')),
            Action::Reply,
        );
        self.register(
            KeyboardShortcut::new(vec![Modifier::Ctrl, Modifier::Shift], Key::Character('r')),
            Action::ReplyAll,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('l')),
            Action::Forward,
        );
        self.register(KeyboardShortcut::new(vec![], Key::Delete), Action::Delete);
        self.register(
            KeyboardShortcut::new(vec![], Key::Character('s')),
            Action::Star,
        );

        // Navigation
        self.register(
            KeyboardShortcut::new(vec![], Key::Character('n')),
            Action::NextUnread,
        );
        self.register(
            KeyboardShortcut::new(vec![], Key::Character('p')),
            Action::PreviousUnread,
        );

        // Composition
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Enter),
            Action::Send,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('s')),
            Action::SaveDraft,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('b')),
            Action::ToggleBold,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('i')),
            Action::ToggleItalic,
        );
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('u')),
            Action::ToggleUnderline,
        );

        // Search
        self.register(
            KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('f')),
            Action::Search,
        );
        self.register(
            KeyboardShortcut::new(vec![], Key::FunctionKey(3)),
            Action::FindNext,
        );

        // Mail checking
        self.register(
            KeyboardShortcut::new(vec![], Key::FunctionKey(9)),
            Action::CheckMail,
        );
    }

    /// Register a keyboard shortcut
    pub fn register(&mut self, shortcut: KeyboardShortcut, action: Action) {
        self.shortcuts.insert(shortcut, action);
    }

    /// Unregister a keyboard shortcut
    pub fn unregister(&mut self, shortcut: &KeyboardShortcut) -> Option<Action> {
        self.shortcuts.remove(shortcut)
    }

    /// Get action for a keyboard shortcut
    pub fn get_action(&self, shortcut: &KeyboardShortcut) -> Option<&Action> {
        self.shortcuts.get(shortcut)
    }

    /// Get all shortcuts
    pub fn get_all_shortcuts(&self) -> &HashMap<KeyboardShortcut, Action> {
        &self.shortcuts
    }

    /// Get shortcuts for an action
    pub fn get_shortcuts_for_action(&self, action: &Action) -> Vec<&KeyboardShortcut> {
        self.shortcuts
            .iter()
            .filter(|(_, a)| *a == action)
            .map(|(k, _)| k)
            .collect()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_display() {
        let shortcut = KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('c'));
        assert_eq!(format!("{}", shortcut), "Ctrl+C");

        let shortcut =
            KeyboardShortcut::new(vec![Modifier::Ctrl, Modifier::Shift], Key::Character('r'));
        assert_eq!(format!("{}", shortcut), "Ctrl+Shift+R");
    }

    #[test]
    fn test_key_display() {
        assert_eq!(format!("{}", Key::Character('a')), "A");
        assert_eq!(format!("{}", Key::FunctionKey(1)), "F1");
        assert_eq!(format!("{}", Key::Enter), "Enter");
        assert_eq!(format!("{}", Key::ArrowUp), "Up");
    }

    #[test]
    fn test_shortcut_manager() {
        let manager = ShortcutManager::new();

        // Test default shortcut
        let quit_shortcut = KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('q'));
        assert_eq!(manager.get_action(&quit_shortcut), Some(&Action::Quit));

        // Test non-existent shortcut
        let fake_shortcut = KeyboardShortcut::simple(Modifier::Alt, Key::Character('z'));
        assert_eq!(manager.get_action(&fake_shortcut), None);
    }

    #[test]
    fn test_register_unregister() {
        let mut manager = ShortcutManager::new();
        let shortcut = KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('x'));

        manager.register(shortcut.clone(), Action::Custom("Test".to_string()));
        assert!(manager.get_action(&shortcut).is_some());

        manager.unregister(&shortcut);
        assert!(manager.get_action(&shortcut).is_none());
    }

    #[test]
    fn test_get_shortcuts_for_action() {
        let manager = ShortcutManager::new();
        let shortcuts = manager.get_shortcuts_for_action(&Action::Quit);
        assert!(!shortcuts.is_empty());
    }
}

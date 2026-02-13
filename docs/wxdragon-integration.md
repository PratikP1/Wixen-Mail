# WXDragon UI Integration Research

## Overview

WXDragon is a planned Windows-native UI library for Wixen Mail. This document outlines the research and integration strategy for accessibility-first UI development.

## Current Status

**Research Phase**: WXDragon integration is in the planning stage. The current implementation provides:

1. **Placeholder UI Module** (`src/presentation/ui.rs`)
   - Basic UI manager structure
   - Ready for WXDragon integration

2. **Accessibility Layer** (`src/presentation/accessibility/`)
   - Screen reader bridge for Windows UIA
   - Keyboard handler for shortcuts
   - Focus manager for navigation
   - Announcement queue for screen reader messages
   - Comprehensive keyboard shortcuts system

## Windows UI Automation (UIA) Integration

### Requirements

- Windows 10 or later
- Windows API access via `windows` crate
- UI Automation API for screen reader support

### Key Components

1. **Screen Reader Support**
   - NVDA (NonVisual Desktop Access)
   - JAWS (Job Access With Speech)
   - Windows Narrator
   - Target: WCAG 2.1 Level AA compliance

2. **Keyboard Navigation**
   - Complete keyboard accessibility
   - Customizable shortcuts
   - Focus indicators
   - Tab order management

### Implementation Strategy

```rust
// Placeholder for future WXDragon integration
pub struct UI {
    // Will contain:
    // - Main window handle
    // - UI Automation provider
    // - Event handlers
    // - Component tree
}
```

## Keyboard Shortcuts System

### Default Shortcuts Implemented

#### Application
- `Ctrl+Q` - Quit
- `Ctrl+,` - Open Settings
- `F1` - Open Help

#### Window Navigation
- `F6` - Cycle Panes

#### Message Actions
- `Ctrl+N` - New Message
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply All
- `Ctrl+L` - Forward
- `Delete` - Delete Message
- `S` - Star/Flag Message

#### Navigation
- `N` - Next Unread
- `P` - Previous Unread

#### Composition
- `Ctrl+Enter` - Send
- `Ctrl+S` - Save Draft
- `Ctrl+B` - Bold
- `Ctrl+I` - Italic
- `Ctrl+U` - Underline

#### Search
- `Ctrl+F` - Search
- `F3` - Find Next

#### Mail Checking
- `F9` - Check Mail

### Customization

Users can customize shortcuts through the `ShortcutManager`:

```rust
let mut manager = ShortcutManager::new();
manager.register(
    KeyboardShortcut::simple(Modifier::Ctrl, Key::Character('m')),
    Action::NewMessage
);
```

## Accessibility Features

### Screen Reader Announcements

The announcement queue manages screen reader messages with priority levels:

- **Urgent**: Critical errors, security warnings
- **High**: Important status changes, new mail
- **Normal**: Regular updates, navigation
- **Low**: Background operations, hints

### Focus Management

The focus manager ensures:

1. Logical focus order
2. Visual focus indicators
3. Focus trapping in dialogs
4. Proper focus restoration

### High Contrast Support

- Full support for Windows High Contrast themes
- Custom high contrast mode
- Adjustable color schemes

## Next Steps

### Phase 1: Research
- [x] Define keyboard shortcuts system ✓
- [x] Design accessibility layer ✓
- [ ] Evaluate WXDragon alternatives (if needed)
- [ ] Review Windows UIA best practices

### Phase 2: Prototype
- [ ] Create basic WXDragon window
- [ ] Implement UI Automation provider
- [ ] Test with screen readers
- [ ] Validate keyboard navigation

### Phase 3: Integration
- [ ] Main window implementation
- [ ] Three-pane layout (folder tree, message list, reading pane)
- [ ] Message composition window
- [ ] Settings dialog

### Phase 4: Testing
- [ ] Screen reader compatibility testing
- [ ] Keyboard-only navigation testing
- [ ] High contrast mode testing
- [ ] Performance testing

## Dependencies

### Planned Crates

```toml
# Windows-specific UI and accessibility
windows = { version = "0.52", features = ["Win32_UI_Accessibility", "Win32_Foundation"] }

# Potential alternatives if WXDragon is not available:
# - native-windows-gui = "1.0"
# - druid = "0.8" (cross-platform, can target Windows)
# - egui = "0.25" (immediate mode, good accessibility support)
```

## Resources

### Windows UIA Documentation
- [UI Automation Overview](https://docs.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32)
- [UI Automation Control Patterns](https://docs.microsoft.com/en-us/windows/win32/winauto/uiauto-controlpatternsoverview)
- [Implementing UI Automation Providers](https://docs.microsoft.com/en-us/windows/win32/winauto/uiauto-providersoverview)

### Screen Reader Testing
- [NVDA Download](https://www.nvaccess.org/download/)
- [JAWS](https://www.freedomscientific.com/products/software/jaws/)
- [Windows Narrator](https://support.microsoft.com/en-us/windows/complete-guide-to-narrator-e4397a0d-ef4f-b386-d8ae-c172f109bdb1)

### Accessibility Standards
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Section 508 Standards](https://www.section508.gov/)

## Notes

This is a living document and will be updated as the WXDragon integration progresses. The current implementation provides a solid foundation with:

1. Complete keyboard shortcuts system
2. Accessibility layer interfaces
3. Privacy-aware logging
4. Configuration management

All ready for UI integration when WXDragon becomes available.

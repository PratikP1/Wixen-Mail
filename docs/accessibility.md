# Wixen Mail - Accessibility Guide

## Overview
Wixen Mail is designed from the ground up to be fully accessible to users who rely on assistive technologies, particularly screen readers. This document outlines our accessibility commitments and provides a comprehensive guide to keyboard navigation.

## Accessibility Commitments

### Standards Compliance
- **WCAG 2.1 Level AA**: We aim to meet or exceed Web Content Accessibility Guidelines 2.1 Level AA standards
- **Section 508**: Compliance with Section 508 of the Rehabilitation Act
- **Windows Accessibility**: Full integration with Windows accessibility APIs (UIA - UI Automation)

### Screen Reader Support
Wixen Mail is tested and optimized for the following screen readers:
- **NVDA (NonVisual Desktop Access)**: Primary testing platform
- **JAWS (Job Access With Speech)**: Full support
- **Windows Narrator**: Native Windows screen reader support
- **Other Screen Readers**: Best effort support for other screen readers that follow Windows accessibility standards

## Keyboard Navigation

### Global Keyboard Shortcuts

#### Application Control
- `Alt + F4` - Close application
- `Ctrl + Q` - Quit application
- `Ctrl + ,` (Comma) - Open preferences/settings
- `F1` - Open help documentation
- `Ctrl + Shift + K` - Open keyboard shortcuts reference

#### Window Navigation
- `F6` - Cycle through main panes (folder tree, message list, reading pane)
- `Shift + F6` - Cycle through main panes in reverse
- `Ctrl + Tab` - Switch between tabs (if multiple windows/tabs are open)
- `Ctrl + Shift + Tab` - Switch between tabs in reverse
- `Esc` - Close dialog/cancel current operation

### Folder Tree Navigation

#### Movement
- `Up/Down Arrow` - Move between folders
- `Left Arrow` - Collapse current folder
- `Right Arrow` - Expand current folder
- `Home` - Jump to first folder
- `End` - Jump to last folder
- `*` (Asterisk) - Expand all subfolders
- `Ctrl + Up/Down` - Move without changing selection

#### Actions
- `Enter` - Open/select folder
- `Ctrl + N` - Create new folder
- `F2` - Rename selected folder
- `Delete` - Delete selected folder (with confirmation)
- `Ctrl + C` - Copy folder
- `Ctrl + X` - Cut folder
- `Ctrl + V` - Paste folder
- `Alt + Enter` - Show folder properties
- `Shift + F10` or `Menu Key` - Open context menu

### Message List Navigation

#### Movement
- `Up/Down Arrow` - Move between messages
- `Page Up/Page Down` - Scroll one page of messages
- `Home` - Jump to first message
- `End` - Jump to last message
- `Ctrl + Up/Down` - Move without changing selection
- `Shift + Up/Down` - Extend selection
- `Ctrl + A` - Select all messages

#### Message Actions
- `Enter` - Open message in new window
- `Space` - Mark message as read/unread (toggle)
- `Delete` - Move message to trash
- `Shift + Delete` - Permanently delete message (with confirmation)
- `Ctrl + R` - Reply to message
- `Ctrl + Shift + R` - Reply all
- `Ctrl + L` - Forward message
- `S` - Star/flag message
- `M` - Move message to folder
- `T` - Add tag to message
- `J` - Mark as junk
- `Shift + J` - Mark as not junk

#### View Control
- `N` - Jump to next unread message
- `P` - Jump to previous unread message
- `Shift + N` - Jump to next unread thread
- `Shift + P` - Jump to previous unread thread
- `T` - Toggle thread view
- `O` - Expand/collapse thread
- `Shift + O` - Expand/collapse all threads

### Reading Pane Navigation

#### Movement
- `Space` - Scroll down one page
- `Shift + Space` - Scroll up one page
- `Home` - Jump to top of message
- `End` - Jump to bottom of message
- `Tab` - Move to next link/button
- `Shift + Tab` - Move to previous link/button

#### Actions
- `Ctrl + R` - Reply to message
- `Ctrl + Shift + R` - Reply all
- `Ctrl + L` - Forward message
- `Ctrl + S` - Save message
- `Ctrl + P` - Print message
- `A` - Show/hide attachments pane
- `Ctrl + K` - Open link (when focused on a link)

#### Attachments
- `Tab` (when in attachment area) - Navigate between attachments
- `Enter` - Open/view attachment
- `Ctrl + S` - Save attachment to disk
- `Shift + F10` or `Menu Key` - Open attachment context menu

### Message Composition

#### Composition Window
- `Ctrl + N` or `Ctrl + M` - New message
- `Ctrl + Enter` - Send message
- `Ctrl + S` - Save draft
- `Ctrl + W` - Close composition window
- `Alt + S` - Send message (alternative)
- `Esc` - Close composition window (saves draft)

#### Editing
- `Ctrl + Z` - Undo
- `Ctrl + Y` - Redo
- `Ctrl + X` - Cut
- `Ctrl + C` - Copy
- `Ctrl + V` - Paste
- `Ctrl + A` - Select all
- `Ctrl + F` - Find in message
- `Ctrl + H` - Replace text

#### Formatting (Rich Text Mode)
- `Ctrl + B` - Bold
- `Ctrl + I` - Italic
- `Ctrl + U` - Underline
- `Ctrl + Shift + L` - Bulleted list
- `Ctrl + Shift + O` - Numbered list
- `Ctrl + Shift + >` - Increase font size
- `Ctrl + Shift + <` - Decrease font size
- `Ctrl + \` - Clear formatting

#### Recipients and Fields
- `Tab` - Move between To/Cc/Bcc/Subject/Body fields
- `Shift + Tab` - Move backwards between fields
- `Ctrl + Shift + T` - Show/hide Cc field
- `Ctrl + Shift + B` - Show/hide Bcc field

#### Attachments in Composition
- `Ctrl + Shift + A` - Add attachment
- `Delete` (when focused on attachment) - Remove attachment

### Search and Filtering

#### Quick Search
- `Ctrl + F` - Focus quick search box (in message list)
- `Ctrl + Shift + F` - Advanced search/filter
- `Enter` - Execute search
- `Esc` - Clear search/return to full list
- `F3` - Find next match
- `Shift + F3` - Find previous match

#### Search Window
- `Tab` - Move between search criteria fields
- `Ctrl + Enter` - Execute search
- `Ctrl + W` - Close search window

### Account and Settings

#### Account Management
- `Alt + T, A` - Account settings
- `Ctrl + Shift + A` - Add new account
- `F9` - Get new messages for all accounts
- `Ctrl + T` - Get new messages for current account

#### Settings and Preferences
- `Ctrl + ,` (Comma) - Open preferences
- `Tab` - Navigate between preference categories
- `Space` - Toggle checkbox/radio options
- `Enter` - Activate buttons
- `Esc` - Cancel and close preferences

### Context Menus and Dialogs

#### General
- `Shift + F10` or `Menu Key` - Open context menu for selected item
- `Up/Down Arrow` - Navigate menu items
- `Enter` - Activate menu item
- `Esc` - Close menu
- `Alt + Letter` - Access menu item by underlined letter (when available)

#### Dialog Navigation
- `Tab` - Move to next control
- `Shift + Tab` - Move to previous control
- `Space` - Toggle checkbox/button
- `Enter` - Activate default button
- `Esc` - Cancel/close dialog
- `Alt + Letter` - Access control by underlined letter

## Accessibility Features

### Screen Reader Announcements
- Real-time announcements for new mail arrival
- Message status changes (read/unread, flagged)
- Folder selection changes
- Progress notifications for long operations
- Error and warning messages

### High Contrast Support
- Full support for Windows High Contrast themes
- Custom high contrast mode within application
- Adjustable color schemes for different visual needs

### Keyboard Focus Management
- Clear visual focus indicators
- Logical focus order throughout the application
- Focus trapped in modal dialogs
- Focus returns to appropriate location after dialog closure

### Text and Display
- Customizable font sizes
- Zoom in/out support (`Ctrl + Plus/Minus`)
- Respect system font settings
- Line spacing adjustments
- Support for Windows display scaling

### Timing and Animations
- No time-sensitive operations without alternatives
- Option to disable animations
- Configurable auto-save intervals
- No automatic timeouts for user interactions

### Navigation Shortcuts
- Breadcrumb navigation for complex hierarchies
- Skip links to jump to main content areas
- Landmarks for major application sections
- Consistent navigation patterns throughout

## Testing and Feedback

### Accessibility Testing
We regularly test Wixen Mail with:
- Automated accessibility testing tools
- Manual screen reader testing (NVDA, JAWS, Narrator)
- Keyboard-only navigation testing
- High contrast and zoom testing
- Real users with disabilities

### Reporting Accessibility Issues
If you encounter any accessibility barriers:
1. Open an issue on our GitHub repository
2. Tag it with the `accessibility` label
3. Provide details about:
   - Your assistive technology (name and version)
   - Operating system version
   - Steps to reproduce the issue
   - Expected vs. actual behavior

We prioritize accessibility issues and aim to address them promptly.

## Customization

### Keyboard Shortcut Customization
Users can customize keyboard shortcuts through:
1. Settings → Keyboard Shortcuts
2. Select action to customize
3. Press new key combination
4. Save changes

### Screen Reader Verbosity
Adjust the level of detail provided by screen readers:
- **Verbose**: Detailed announcements for all events
- **Normal**: Standard announcements (default)
- **Brief**: Minimal announcements for experienced users

### Focus Management Preferences
- **Auto-focus**: Automatically focus new windows and dialogs
- **Manual focus**: Keep focus in current location until manually moved

## Resources

### Documentation
- [Keyboard Shortcuts Quick Reference](docs/keyboard-shortcuts.md)
- [Screen Reader User Guide](docs/screen-reader-guide.md)
- [Accessibility FAQ](docs/accessibility-faq.md)

### External Resources
- [NVDA Screen Reader](https://www.nvaccess.org/)
- [JAWS Screen Reader](https://www.freedomscientific.com/products/software/jaws/)
- [Windows Narrator](https://support.microsoft.com/en-us/windows/complete-guide-to-narrator-e4397a0d-ef4f-b386-d8ae-c172f109bdb1)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## Contact

For accessibility-related questions or feedback:
- GitHub Issues: Use the `accessibility` label
- Email: accessibility@wixen-mail.org (when available)
- Community Forum: GitHub Discussions

## Commitment to Continuous Improvement

Accessibility is not a one-time effort but an ongoing commitment. We will:
- Regularly audit our application for accessibility
- Incorporate user feedback from the disability community
- Stay updated with accessibility standards and best practices
- Continuously improve our accessibility features

Thank you for helping us make Wixen Mail accessible to everyone!

# Phase 1 & 2 Implementation Guide

## Overview
This document outlines the complete implementation of Phase 1 (UI Integration) and Phase 2 (Persistent Caching & HTML Rendering) with full accessibility support.

## Phase 1: UI Integration

### Task 1: UI Async Integration ✅ (Implementing Now)

**Changes Required:**
1. Update `UIState` to include:
   - MailController instance
   - Connection status
   - Error messages
   - Account configuration
   
2. Add async runtime to UI:
   - Tokio runtime embedded in eframe
   - Async channels for UI updates
   - Background task management

3. Update UI struct:
   - Add tokio runtime handle
   - Add async channels (send/receive)
   - Add MailController

**Files to Modify:**
- `src/presentation/ui.rs` - Main UI integration
- `src/bin/ui.rs` - Runtime initialization

### Task 2: Account Configuration Dialog

**Features:**
- IMAP server configuration (host, port, TLS)
- SMTP server configuration (host, port, TLS)
- Username/password (privacy-aware)
- Test connection button
- Save/load from config

**UI Components:**
- Text fields for server details
- Checkbox for TLS
- Password field (masked)
- Status indicator
- Test/Save/Cancel buttons

### Task 3-5: Connect Real Data (IMAP)

**Folder List:**
- Fetch from IMAP on connection
- Update UI when folders change
- Handle folder selection

**Message List:**
- Fetch messages when folder selected
- Display in UI list
- Update read/unread status

**Message Preview:**
- Fetch full message body
- Display in preview pane
- Handle HTML vs plain text

### Task 6: Composition Window (SMTP)

**Features:**
- Send email via SMTP
- Draft saving
- Attachment support (Phase 3)
- Error handling

### Task 7: Error Handling

**Status Indicators:**
- Connection status (connected/disconnected/error)
- Operation progress (fetching folders, messages)
- Error messages to user
- Retry logic

## Phase 2: Persistent Caching & HTML Rendering

### Task 8-9: SQLite Caching

**Database Schema:**
```sql
CREATE TABLE folders (
    id INTEGER PRIMARY KEY,
    account_id TEXT,
    name TEXT,
    path TEXT,
    folder_type TEXT,
    unread_count INTEGER,
    total_count INTEGER
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    uid INTEGER,
    folder_id INTEGER,
    message_id TEXT,
    subject TEXT,
    from_addr TEXT,
    to_addr TEXT,
    cc TEXT,
    date TEXT,
    body_plain TEXT,
    body_html TEXT,
    read BOOLEAN,
    starred BOOLEAN,
    deleted BOOLEAN,
    FOREIGN KEY(folder_id) REFERENCES folders(id)
);

CREATE TABLE attachments (
    id INTEGER PRIMARY KEY,
    message_id INTEGER,
    filename TEXT,
    mime_type TEXT,
    size INTEGER,
    content_id TEXT,
    FOREIGN KEY(message_id) REFERENCES messages(id)
);
```

**Implementation:**
- Add `rusqlite` dependency
- Create database service
- Cache operations (save/load/update)
- Offline mode support

### Task 10-12: HTML Rendering

**Dependencies:**
- `ammonia` - HTML sanitization
- `egui_extras` - Already added
- `image` - Already added

**Features:**
- Sanitize HTML content (XSS protection)
- Render in egui with formatting
- Load and display images
- Accessibility: Screen reader compatibility

### Task 13: Full Accessibility

**Requirements:**
- All HTML content accessible to screen readers
- Keyboard navigation for all features
- ARIA labels where needed
- High contrast support
- Focus indicators

## Implementation Order

1. ✅ UI Async Integration (Task 1)
2. Account Configuration Dialog (Task 2)
3. Connect IMAP folders (Task 3)
4. Connect IMAP messages (Task 4)
5. Connect message preview (Task 5)
6. Connect SMTP composition (Task 6)
7. Error handling (Task 7)
8. SQLite database setup (Task 8)
9. Cache implementation (Task 9)
10. HTML sanitization (Task 10)
11. HTML rendering (Task 11)
12. Image loading (Task 12)
13. Accessibility verification (Task 13)

## Testing Strategy

After each task:
1. Unit tests for new functionality
2. Integration tests for UI flows
3. Manual UI testing
4. Accessibility testing (keyboard, screen reader)

## Success Criteria

**Phase 1 Complete When:**
- User can configure account
- Folders load from IMAP
- Messages display from IMAP
- Message preview shows content
- Emails send via SMTP
- Errors handled gracefully
- All tests passing

**Phase 2 Complete When:**
- Messages cached in SQLite
- HTML emails render correctly
- Images load in HTML emails
- Offline mode works
- Full accessibility maintained
- All tests passing

## Timeline

- Phase 1: 5-7 days (Tasks 1-7)
- Phase 2: 3-5 days (Tasks 8-13)
- Total: 8-12 days

## Next Steps

Starting with Task 1: Implementing UI async integration now.

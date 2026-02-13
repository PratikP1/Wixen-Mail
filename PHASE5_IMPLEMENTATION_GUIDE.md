# Phase 5 Implementation Guide - Advanced Features

**Status:** Ready to Implement  
**Estimated Duration:** 2-3 months  
**Priority:** High - Critical for v1.0 release

---

## Executive Summary

Phase 5 adds advanced features that transform Wixen Mail from a basic email client into a professional-grade application. This guide provides detailed implementation plans for each feature with accessibility considerations.

---

## Feature 1: Multiple Account Support (Weeks 1-2)

### Overview
Enable users to manage multiple email accounts in a single application instance.

### Requirements
- Support 3+ email accounts simultaneously
- Account switcher in UI
- Per-account folder trees
- Per-account settings
- Account management dialog (add, edit, delete)
- Unified inbox (optional)

### Technical Design

#### Data Model Changes

**Extend `src/data/config.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields ...
    /// List of configured accounts
    pub accounts: Vec<AccountConfig>,
    /// Currently active account ID
    pub active_account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: Id,
    pub name: String,
    pub email: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub imap_use_tls: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_use_tls: bool,
    pub username: String,
    pub password: String, // Will be encrypted in production
    pub signature: Option<String>,
    pub check_interval_minutes: u32,
    pub enabled: bool,
}
```

#### UI Changes

**Add to `src/presentation/ui_integrated.rs`:**
```rust
pub struct IntegratedUI {
    // ... existing fields ...
    /// List of accounts
    pub accounts: Vec<AccountConfig>,
    /// Currently selected account index
    pub selected_account_index: usize,
    /// Account controllers (one per account)
    pub controllers: HashMap<Id, Arc<TokioMutex<MailController>>>,
    /// Account management window
    pub account_manager_open: bool,
}
```

**Account Switcher UI:**
- Dropdown in top-left corner
- Shows account name and email
- Keyboard shortcut: Ctrl+1, Ctrl+2, Ctrl+3 for accounts
- Updates folder tree and messages when switched

**Account Management Dialog:**
- List of configured accounts
- Add Account button → opens account config dialog
- Edit button per account
- Delete button with confirmation
- Enable/Disable toggle
- Test Connection button

#### Implementation Steps

1. **Extend AppConfig (1 day)**
   - Add accounts vector
   - Add active_account_id
   - Migration logic for existing single account
   - Tests for multi-account config

2. **Create Account Manager (2 days)**
   - Account list UI
   - Add/Edit/Delete operations
   - Validation and error handling
   - Keyboard accessibility

3. **Implement Account Switcher (2 days)**
   - Dropdown UI in menu bar
   - Switch folder tree
   - Switch message list
   - Clear state on switch

4. **Multiple MailControllers (2 days)**
   - HashMap of controllers
   - Lazy initialization
   - Connection pooling
   - Proper cleanup on delete

5. **Testing (2 days)**
   - Unit tests for config changes
   - Integration tests with 3 accounts
   - UI tests for switching
   - Accessibility tests

### Keyboard Shortcuts
- `Ctrl+K` - Open account switcher
- `Ctrl+1` through `Ctrl+9` - Switch to account 1-9
- `Alt+A` - Open account manager

### Accessibility
- Account switcher accessible via keyboard
- Screen reader announces account name on switch
- All dialogs keyboard navigable
- Focus management on account switch

### Test Coverage
- Config serialization/deserialization
- Account CRUD operations
- Account switching preserves state
- Multiple simultaneous connections
- Error handling (connection failures)

---

## Feature 2: Advanced Search (Weeks 3-4)

### Overview
Powerful search capabilities across all folders with multiple filter criteria.

### Requirements
- Search across all folders or selected folder
- Filter by: From, To, Subject, Body, Date range
- Saved searches
- Search history
- Keyboard accessible

### Technical Design

#### Data Model

**Add to `src/data/message_cache.rs`:**
```rust
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub query: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub folder: Option<String>,
    pub has_attachments: Option<bool>,
    pub is_unread: Option<bool>,
    pub is_starred: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: Id,
    pub name: String,
    pub criteria: SearchCriteria,
    pub created_at: DateTime<Utc>,
}
```

#### UI Changes

**Enhanced Search Dialog:**
```
┌─────────────────────────────────────────────┐
│ Advanced Search                         [X] │
├─────────────────────────────────────────────┤
│ Quick Search: [________________]            │
│                                              │
│ ┌─ Filters ─────────────────────────────┐  │
│ │ From:     [________________]          │  │
│ │ To:       [________________]          │  │
│ │ Subject:  [________________]          │  │
│ │ Date:     [From: ___] [To: ___]      │  │
│ │ Folder:   [All Folders ▼]            │  │
│ │ ☐ Has attachments                     │  │
│ │ ☐ Unread only                         │  │
│ │ ☐ Starred only                        │  │
│ └───────────────────────────────────────┘  │
│                                              │
│ [Search] [Save Search] [Clear]              │
│                                              │
│ ┌─ Results (42) ────────────────────────┐  │
│ │ [Subject] - [From] - [Date]           │  │
│ │ ...                                    │  │
│ └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

#### Implementation Steps

1. **Search Backend (3 days)**
   - Implement SearchCriteria matching
   - SQLite queries for efficient search
   - IMAP SEARCH command support
   - Result ranking/sorting

2. **Enhanced Search UI (2 days)**
   - Filter fields
   - Date picker
   - Folder selector
   - Checkbox filters

3. **Saved Searches (2 days)**
   - Save search dialog
   - Load saved searches
   - Manage saved searches
   - Quick access from sidebar

4. **Testing (2 days)**
   - Unit tests for search logic
   - Various filter combinations
   - Performance with large mailboxes
   - Accessibility tests

### Keyboard Shortcuts
- `Ctrl+F` - Open search (already implemented)
- `Ctrl+Shift+F` - Open advanced search
- `Enter` - Execute search
- `Ctrl+S` - Save current search
- `Esc` - Close search dialog

### Accessibility
- All filters keyboard accessible
- Tab order logical
- Date picker keyboard navigable
- Results list accessible
- Screen reader announces result count

---

## Feature 3: Message Tagging (Weeks 5-6)

### Overview
User-defined tags for organizing and categorizing messages.

### Requirements
- Create custom tags with colors
- Apply multiple tags to messages
- Filter messages by tag
- Tag management UI
- Keyboard shortcuts for common tags

### Technical Design

#### Data Model

**Add to `src/data/message_cache.rs`:**
```sql
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(account_id, name)
);

CREATE TABLE IF NOT EXISTS message_tags (
    message_uid INTEGER NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (message_uid, tag_id),
    FOREIGN KEY (message_uid) REFERENCES messages(uid),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE INDEX idx_message_tags_tag_id ON message_tags(tag_id);
CREATE INDEX idx_message_tags_message_uid ON message_tags(message_uid);
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub color: String, // Hex color code
    pub created_at: DateTime<Utc>,
}
```

#### UI Changes

**Tag Display:**
- Show tags as colored pills below subject
- Click tag to filter by that tag
- Right-click for tag menu

**Tag Management Dialog:**
```
┌─────────────────────────────────────┐
│ Manage Tags                     [X] │
├─────────────────────────────────────┤
│ ┌─ Tags ─────────────────────────┐ │
│ │ 🔴 Important                   │ │
│ │ 🟢 Work                        │ │
│ │ 🔵 Personal                    │ │
│ │ 🟡 Follow-up                   │ │
│ └─────────────────────────────────┘ │
│                                      │
│ [New Tag] [Edit] [Delete]           │
│                                      │
│ ┌─ Create/Edit Tag ──────────────┐ │
│ │ Name:  [___________]            │ │
│ │ Color: [🔴🟠🟡🟢🔵🟣⚫⚪]        │ │
│ └─────────────────────────────────┘ │
│                                      │
│ [Save] [Cancel]                     │
└─────────────────────────────────────┘
```

**Tag Context Menu:**
- Add Tag
- Remove Tag
- Edit Tag
- Delete Tag

#### Implementation Steps

1. **Database Schema (1 day)**
   - Create tags table
   - Create message_tags junction table
   - Migration and indices

2. **Tag CRUD Operations (2 days)**
   - Create/Read/Update/Delete tags
   - Apply/remove tags from messages
   - Bulk tag operations
   - Tests

3. **Tag UI (3 days)**
   - Tag management dialog
   - Tag display on messages
   - Tag filtering
   - Color picker

4. **Testing (1 day)**
   - Unit tests for tag operations
   - UI tests for tag application
   - Performance with many tags
   - Accessibility tests

### Keyboard Shortcuts
- `T` - Open tag menu for selected message
- `Ctrl+T` - Open tag manager
- `1-9` - Apply preset tag 1-9 to selected message

### Accessibility
- Tag pills keyboard focusable
- Screen reader announces tag names and colors
- Tag manager fully keyboard navigable
- Color picker accessible

---

## Feature 4: Email Rules/Filters (Weeks 7-8)

### Overview
Automatic message processing based on user-defined rules.

### Requirements
- Create rules with conditions and actions
- Conditions: From, To, Subject contains, etc.
- Actions: Move, Tag, Mark read, Delete, etc.
- Apply rules manually or automatically
- Rule priority/order

### Technical Design

#### Data Model

**Add to `src/data/message_cache.rs`:**
```sql
CREATE TABLE IF NOT EXISTS rules (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    conditions TEXT NOT NULL, -- JSON
    actions TEXT NOT NULL, -- JSON
    created_at TEXT NOT NULL
);
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    FromContains(String),
    ToContains(String),
    SubjectContains(String),
    BodyContains(String),
    HasAttachment,
    IsUnread,
    IsStarred,
    DateAfter(DateTime<Utc>),
    DateBefore(DateTime<Utc>),
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    MoveTo(String), // folder path
    ApplyTag(Id),
    MarkAsRead,
    MarkAsStarred,
    Delete,
    Forward(String), // email address
}
```

#### Rule Engine

```rust
impl MessageCache {
    pub fn apply_rules(&self, message: &CachedMessage, account_id: &Id) -> Result<()> {
        let rules = self.load_rules(account_id)?;
        
        for rule in rules.iter().filter(|r| r.enabled) {
            if self.matches_rule(message, rule)? {
                self.execute_actions(message, &rule.actions)?;
            }
        }
        
        Ok(())
    }
    
    fn matches_rule(&self, message: &CachedMessage, rule: &Rule) -> Result<bool> {
        // Evaluate all conditions
        // Return true if all conditions match
    }
    
    fn execute_actions(&self, message: &CachedMessage, actions: &[RuleAction]) -> Result<()> {
        // Execute each action
    }
}
```

#### UI Changes

**Rule Creation Dialog:**
```
┌──────────────────────────────────────────────┐
│ Create/Edit Rule                         [X] │
├──────────────────────────────────────────────┤
│ Name: [_________________________]            │
│ ☑ Enabled                                    │
│                                              │
│ ┌─ Conditions (Match ALL) ─────────────┐    │
│ │ [From] [contains] [boss@company.com] │    │
│ │ [Add Condition]                       │    │
│ └──────────────────────────────────────┘    │
│                                              │
│ ┌─ Actions ─────────────────────────────┐   │
│ │ [Move to] [Work ▼]                    │   │
│ │ [Apply tag] [Important ▼]             │   │
│ │ [Add Action]                          │   │
│ └──────────────────────────────────────┘    │
│                                              │
│ [Save] [Cancel]                             │
└──────────────────────────────────────────────┘
```

#### Implementation Steps

1. **Data Model (2 days)**
   - Rules table schema
   - Rule/Condition/Action structs
   - JSON serialization
   - Tests

2. **Rule Engine (3 days)**
   - Condition matching logic
   - Action execution
   - Priority handling
   - Error handling
   - Tests

3. **Rule UI (3 days)**
   - Rule list dialog
   - Rule creation wizard
   - Condition builder
   - Action builder
   - Tests

4. **Integration (1 day)**
   - Auto-apply on message arrival
   - Manual rule application
   - Bulk rule application
   - Tests

### Keyboard Shortcuts
- `Ctrl+R` - Open rules manager
- `Alt+R` - Apply rules to selected messages

### Accessibility
- Rule builder keyboard navigable
- Condition/action dropdowns accessible
- Screen reader support for complex forms
- Test rule button accessible

---

## Feature 5: Email Signatures (Weeks 9-10)

### Overview
Per-account email signatures with HTML and plain text support.

### Requirements
- Create/edit signatures
- Per-account signature selection
- Auto-insert on compose/reply
- HTML and plain text versions
- Signature positioning (above/below)

### Technical Design

#### Data Model

**Add to `src/data/config.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    // ... existing fields ...
    pub signature_id: Option<Id>,
}
```

**Add to `src/data/message_cache.rs`:**
```sql
CREATE TABLE IF NOT EXISTS signatures (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    content_html TEXT,
    position TEXT NOT NULL, -- 'above' or 'below'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub content_plain: String,
    pub content_html: Option<String>,
    pub position: SignaturePosition,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignaturePosition {
    Above, // Above quoted text in replies
    Below, // Below quoted text in replies
}
```

#### UI Changes

**Signature Editor:**
```
┌───────────────────────────────────────────┐
│ Edit Signature                        [X] │
├───────────────────────────────────────────┤
│ Name: [___________________________]       │
│                                           │
│ [ Plain Text | HTML ]                    │
│                                           │
│ ┌─────────────────────────────────────┐  │
│ │ Best regards,                        │  │
│ │ John Doe                             │  │
│ │ john.doe@example.com                 │  │
│ │ Phone: (555) 123-4567                │  │
│ │                                      │  │
│ └─────────────────────────────────────┘  │
│                                           │
│ Position: ⦿ Below quoted text             │
│           ○ Above quoted text             │
│                                           │
│ [Save] [Cancel] [Preview]                │
└───────────────────────────────────────────┘
```

#### Implementation Steps

1. **Database Schema (1 day)**
   - Signatures table
   - Migration
   - Tests

2. **Signature CRUD (2 days)**
   - Create/Read/Update/Delete
   - Load for account
   - Tests

3. **Signature Editor UI (2 days)**
   - Editor dialog
   - HTML/plain text toggle
   - Preview
   - Tests

4. **Auto-Insert (2 days)**
   - Insert on new composition
   - Insert on reply (correct position)
   - Insert on forward
   - Tests

### Keyboard Shortcuts
- `Ctrl+Shift+S` - Open signature manager
- `Alt+S` - Insert signature in composition

### Accessibility
- Editor fully keyboard accessible
- Preview available for screen readers
- Position selection accessible
- All buttons keyboard focusable

---

## Feature 6: Contact Management (Weeks 11-12)

### Overview
Local contact database with auto-complete and vCard support.

### Requirements
- Create/edit contacts
- Auto-complete in To/CC/BCC fields
- Contact groups
- Import/export vCard
- Search contacts

### Technical Design

#### Data Model

**Add to `src/data/message_cache.rs`:**
```sql
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email TEXT NOT NULL,
    first_name TEXT,
    last_name TEXT,
    organization TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(account_id, email)
);

CREATE TABLE IF NOT EXISTS contact_groups (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(account_id, name)
);

CREATE TABLE IF NOT EXISTS contact_group_members (
    group_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    PRIMARY KEY (group_id, contact_id),
    FOREIGN KEY (group_id) REFERENCES contact_groups(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);

CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_display_name ON contacts(display_name);
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Id,
    pub account_id: Id,
    pub display_name: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub members: Vec<Id>, // Contact IDs
    pub created_at: DateTime<Utc>,
}
```

#### UI Changes

**Contact Manager:**
```
┌──────────────────────────────────────────────┐
│ Contacts                                 [X] │
├──────────────────────────────────────────────┤
│ Search: [____________] [Import] [Export]     │
│                                              │
│ ┌─ Contacts ────────┬─ Details ───────────┐ │
│ │ Alice Brown      │ Name: Alice Brown    │ │
│ │ Bob Smith        │ Email: alice@ex.com  │ │
│ │ Charlie Davis    │ Org: Acme Corp       │ │
│ │ ...              │ Notes: ...           │ │
│ └──────────────────┴──────────────────────┘ │
│                                              │
│ [New Contact] [Edit] [Delete] [New Group]   │
└──────────────────────────────────────────────┘
```

**Auto-Complete in Composition:**
- Type in To/CC/BCC field
- Dropdown shows matching contacts
- Arrow keys to navigate
- Enter to select

#### Implementation Steps

1. **Database Schema (1 day)**
   - Contacts tables
   - Contact groups tables
   - Migration
   - Tests

2. **Contact CRUD (2 days)**
   - Create/Read/Update/Delete
   - Search contacts
   - Tests

3. **vCard Import/Export (2 days)**
   - Parse vCard format
   - Export to vCard
   - Bulk import
   - Tests

4. **Auto-Complete (2 days)**
   - Fuzzy matching
   - Dropdown UI
   - Keyboard navigation
   - Tests

5. **Contact Manager UI (2 days)**
   - Contact list
   - Contact editor
   - Group management
   - Tests

### Keyboard Shortcuts
- `Ctrl+Shift+C` - Open contact manager
- `Ctrl+K` - Quick search contacts (in composition)

### Accessibility
- Contact list keyboard navigable
- Auto-complete accessible
- Screen reader announces matches
- All dialogs keyboard accessible

---

## Testing Strategy

### Unit Tests
- All new data models
- Search logic
- Rule matching engine
- Tag operations
- Contact operations

### Integration Tests
- Multi-account scenarios
- Rule application on messages
- Signature insertion
- Contact auto-complete

### UI Tests
- All new dialogs
- Keyboard navigation
- Screen reader compatibility
- Focus management

### Performance Tests
- Search with 10,000+ messages
- Rule evaluation overhead
- Contact auto-complete latency
- Multi-account memory usage

### Accessibility Tests
- NVDA testing
- JAWS testing
- Narrator testing
- Keyboard-only navigation

---

## Dependencies

### New Crates Needed

```toml
[dependencies]
# ... existing ...
chrono = "0.4" # Date/time for filters and rules
regex = "1.10" # Pattern matching for rules
vcard = "0.3" # vCard import/export
```

---

## Migration Strategy

### From Phase 4 to Phase 5

1. **Backup existing data**
   - Copy config files
   - Backup SQLite database

2. **Schema migrations**
   - Run ALTER TABLE statements
   - Create new tables
   - Migrate existing data

3. **Config migration**
   - Convert single account to accounts array
   - Set active_account_id

4. **Testing migration**
   - Verify data integrity
   - Test rollback procedure

---

## Success Metrics

### Functional
- [ ] Can manage 5+ accounts without performance issues
- [ ] Search returns results in <200ms for 10K messages
- [ ] Rules apply within 100ms per message
- [ ] Tags display without lag
- [ ] Signatures insert correctly 100% of time
- [ ] Contact auto-complete suggests within 50ms

### Quality
- [ ] 130+ tests passing
- [ ] Zero critical bugs
- [ ] Zero accessibility regressions
- [ ] <5% memory increase per account
- [ ] All keyboard shortcuts working

### User Experience
- [ ] Account switching feels instant
- [ ] Search UI intuitive
- [ ] Tag management easy
- [ ] Rule creation straightforward
- [ ] Signature editor powerful
- [ ] Contact auto-complete helpful

---

## Risk Mitigation

### Technical Risks

**Risk:** Multi-account memory usage  
**Mitigation:** Lazy load MailControllers, connection pooling, close inactive connections

**Risk:** Search performance with large mailboxes  
**Mitigation:** Use SQLite FTS, IMAP server-side search, result pagination

**Risk:** Rule engine complexity  
**Mitigation:** Start simple, add features incrementally, comprehensive tests

**Risk:** UI complexity overwhelming users  
**Mitigation:** Good defaults, progressive disclosure, excellent documentation

### Schedule Risks

**Risk:** Feature scope too large  
**Mitigation:** Prioritize ruthlessly, MVP approach, defer nice-to-haves

**Risk:** Accessibility testing takes longer  
**Mitigation:** Test continuously, not at end, automated tests where possible

---

## Post-Phase 5 Enhancements

### Future Improvements
- OAuth 2.0 authentication
- Cloud sync for contacts/settings
- AI-powered spam filtering
- Smart replies
- Email templates
- Unified inbox
- Calendar integration

---

## Documentation Requirements

### User Documentation
- Multi-account setup guide
- Advanced search tutorial
- Tagging best practices
- Creating effective rules
- Signature examples
- Contact management guide

### Developer Documentation
- Architecture changes
- API documentation
- Rule engine design
- Extension points

### Accessibility Documentation
- Keyboard shortcuts update
- Screen reader guide update
- Accessibility testing report

---

## Timeline Summary

| Weeks | Feature | Status |
|-------|---------|--------|
| 1-2 | Multiple Accounts | Planned |
| 3-4 | Advanced Search | Planned |
| 5-6 | Message Tagging | Planned |
| 7-8 | Email Rules | Planned |
| 9-10 | Email Signatures | Planned |
| 11-12 | Contact Management | Planned |

**Total Duration:** 12 weeks (3 months)  
**Buffer:** 1-2 weeks for polish and bug fixes  
**Target Completion:** Month 13

---

## Conclusion

Phase 5 transforms Wixen Mail into a complete, professional email client. These features are essential for v1.0 and provide the foundation for future enhancements. With careful implementation and continuous testing, we can deliver a high-quality, accessible product that users will love.

**Next Step:** Begin implementation of Multiple Account Support (Week 1).

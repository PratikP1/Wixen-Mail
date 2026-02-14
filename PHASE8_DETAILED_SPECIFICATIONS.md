# Phase 8: Contact Management - Detailed Specifications

## Objective

Deliver a fully accessible account-scoped contact management system with:
- Contact CRUD
- Fast search
- Composition-time recipient autocomplete
- Data persistence and isolation by account

## Functional Requirements

### 1. Contact Data Model
- Required: `id`, `account_id`, `name`, `email`
- Optional: `notes`
- Flags: `favorite`
- Timestamps: `created_at`

### 2. Persistence
- SQLite-backed `contacts` table in `MessageCache`
- Unique constraint on `(account_id, email)`
- CRUD methods:
  - `save_contact`
  - `get_contacts_for_account`
  - `search_contacts_for_account`
  - `delete_contact`

### 3. Contact Manager UI
- Window title: **Manage Contacts**
- Search field: name/email
- Contact list with favorite indicator, notes, edit/delete actions
- Create/Edit form:
  - Name (required)
  - Email (required, basic validation)
  - Notes (optional)
  - Favorite toggle
- Status/error feedback messages
- Keyboard accessible controls and labels

### 4. Composition Autocomplete
- While composing, `To` field query should search account contacts
- Show top 5 suggestions
- Selecting suggestion fills recipient email
- Must not block typing flow

### 5. Accessibility Requirements
- Keyboard shortcut to open manager: `Ctrl+Shift+C`
- All inputs labeled and focusable
- No mouse-only interactions
- Feedback surfaced in visible status/error labels

### 6. Account Isolation
- All contact queries scoped by active `account_id`
- No cross-account contact leakage

## Non-Functional Requirements
- Keep contact lookups lightweight (indexed query path by account/search term)
- Preserve existing test stability
- Avoid regressions in composition flow

## Acceptance Criteria
- Contacts can be created/updated/deleted and persist across restarts
- Search returns expected account-scoped results
- Compose window suggests contacts as user types
- Keyboard shortcut opens contact manager
- Full test suite remains green

# Phase 6: Final Implementation Plan

## Overview
Complete Phase 6 (Multiple Account Support) by implementing 7 remaining items:
1. Migration Tool
2. UI Integration
3. Account Switcher
4. Multi-Controller
5. Background Sync
6. Data Isolation
7. Testing & Polish

## Implementation Details

### 1. Migration Tool
**File**: `src/data/account.rs`
- Add `migrate_from_account_config()` function
- Convert old AccountConfig to new Account structure
- Import email provider settings automatically
- Generate UUID for account ID
- Set default values for new fields

### 2. UI Integration  
**File**: `src/presentation/ui_integrated.rs`
- Wire AccountAction::Create to save_account()
- Wire AccountAction::Update to save_account()
- Wire AccountAction::Delete to delete_account() 
- Wire AccountAction::SetActive to AccountManager
- Load accounts from database on startup
- Auto-save accounts when changed

### 3. Account Switcher
**File**: `src/presentation/ui_integrated.rs`
- Add account switcher dropdown in toolbar
- Show "📧 Account: Name <email>"
- List all enabled accounts in dropdown
- Handle account switch event
- Add keyboard shortcuts Ctrl+1, 2, 3
- Clear current folder/messages on switch
- Load new account data

### 4. Multi-Controller
**File**: `src/presentation/ui_integrated.rs`
- Replace single `mail_controller` with `HashMap<String, Arc<TokioMutex<MailController>>>`
- Add `active_account_id: Option<String>`
- Create controller for account on demand
- Cache controllers in HashMap
- Switch active controller on account change
- Clean up disconnected controllers

### 5. Background Sync
**File**: `src/presentation/ui_integrated.rs`
- Create `spawn_background_sync()` method
- Spawn tokio task per enabled account
- Use account's `check_interval_minutes`
- Sync folders and messages concurrently
- Update `last_sync` timestamp after sync
- Handle errors without crashing
- Display sync status

### 6. Data Isolation
**File**: `src/data/message_cache.rs`
- Verify account_id in folders table
- Verify account_id in messages table  
- Filter queries by account_id
- Modify `get_folders()` to accept account_id
- Modify `get_messages()` to filter by account_id
- Modify tag/signature methods to filter by account_id

### 7. Testing & Polish
**Files**: Various
- Add integration tests for account CRUD
- Add test for account switching
- Add test for data isolation
- Add test for migration
- Update USER_GUIDE.md
- Update KEYBOARD_SHORTCUTS.md
- Add screenshots

## Implementation Order
1. Migration Tool (30 min)
2. UI Integration (1 hour)
3. Account Switcher (1 hour)
4. Multi-Controller (1.5 hours)
5. Background Sync (1 hour)
6. Data Isolation (45 min)
7. Testing & Polish (1 hour)

**Total Time**: ~7 hours

## Success Criteria
- All 7 items implemented
- 110+ tests passing
- No regressions
- Account switching works smoothly
- Data properly isolated per account
- Background sync working
- Documentation updated

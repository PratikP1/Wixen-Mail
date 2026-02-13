# Phase 6: Remaining Implementation (Items 3-7)

## Current Status
- ✅ Item 1: Migration Tool - COMPLETE
- ✅ Item 2: UI Integration - COMPLETE  
- 🔄 Items 3-7: Implementation Required

## Critical Note
The remaining 5 items require significant architectural changes:
- **Item 4 (Multi-Controller)** is the foundational change that affects all other items
- **Estimated Time:** 5.5 hours of focused development
- **Risk:** High - involves refactoring core architecture

## Implementation Order (Recommended)

### Phase A: Foundation (2.5 hours)
**Item 4: Multi-Controller Architecture** (Must be done first)
- This is the core architectural change
- Changes IntegratedUI from single controller to HashMap
- All subsequent items depend on this

### Phase B: UI & Sync (2 hours)
**Item 3: Account Switcher**
- Depends on Multi-Controller being done
- Adds UI dropdown and keyboard shortcuts

**Item 5: Background Sync**
- Depends on Multi-Controller being done
- Spawns tasks per account

### Phase C: Correctness & Quality (1.5 hours)
**Item 6: Data Isolation**
- Ensures proper filtering by account_id
- Critical for security/correctness

**Item 7: Testing & Polish**
- Integration tests
- Documentation
- Screenshots

## Detailed Requirements

### Item 3: Account Switcher (1 hour)

**Location:** `src/presentation/ui_integrated.rs`

**Changes:**
1. Add to toolbar (in `render_toolbar` method):
```rust
// Account switcher dropdown
ui.separator();
ui.label("📧");

let active_display = if let Some(account) = self.state.account_manager.get_active_account() {
    account.display_name()
} else {
    "No Account".to_string()
};

let enabled_accounts: Vec<_> = self.state.account_manager.get_accounts()
    .iter()
    .filter(|a| a.enabled)
    .collect();

if !enabled_accounts.is_empty() {
    egui::ComboBox::from_id_source("account_switcher")
        .selected_text(&active_display)
        .show_ui(ui, |ui| {
            for account in &enabled_accounts {
                let is_active = self.state.account_manager.get_active_account_id()
                    .map(|id| id == &account.id)
                    .unwrap_or(false);
                    
                if ui.selectable_label(is_active, account.display_name()).clicked() {
                    self.switch_account(&account.id);
                }
            }
        });
}
```

2. Add `switch_account` method:
```rust
fn switch_account(&mut self, account_id: &str) {
    // Set as active
    if let Err(e) = self.state.account_manager.set_active_account(account_id) {
        self.state.error_message = Some(format!("Failed to switch account: {}", e));
        return;
    }
    
    // Clear current data
    self.state.selected_folder = None;
    self.state.selected_message = None;
    self.state.folders.clear();
    self.state.messages.clear();
    self.state.message_preview.clear();
    
    // Update status
    if let Some(account) = self.state.account_manager.get_active_account() {
        self.state.status_message = format!("Switched to account: {}", account.display_name());
    }
    
    // TODO: Load folders for new account
    // This requires multi-controller to be implemented first
}
```

3. Add keyboard shortcuts (in `handle_input` method):
```rust
// Account switching shortcuts
if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
    let accounts: Vec<_> = self.state.account_manager.get_accounts()
        .iter()
        .filter(|a| a.enabled)
        .collect();
    if let Some(account) = accounts.get(0) {
        self.switch_account(&account.id);
    }
}
if ctx.input(|i| i.key_pressed(egui::Key::Num2) && i.modifiers.ctrl) {
    let accounts: Vec<_> = self.state.account_manager.get_accounts()
        .iter()
        .filter(|a| a.enabled)
        .collect();
    if let Some(account) = accounts.get(1) {
        self.switch_account(&account.id);
    }
}
if ctx.input(|i| i.key_pressed(egui::Key::Num3) && i.modifiers.ctrl) {
    let accounts: Vec<_> = self.state.account_manager.get_accounts()
        .iter()
        .filter(|a| a.enabled)
        .collect();
    if let Some(account) = accounts.get(2) {
        self.switch_account(&account.id);
    }
}
```

### Item 4: Multi-Controller (1.5 hours) **CRITICAL**

**Location:** `src/presentation/ui_integrated.rs`

**Changes:**

1. Modify `IntegratedUI` struct:
```rust
pub struct IntegratedUI {
    // OLD: mail_controller: Arc<TokioMutex<MailController>>,
    // NEW:
    mail_controllers: std::collections::HashMap<String, Arc<TokioMutex<MailController>>>,
    active_account_id: Option<String>,
    
    runtime: Arc<Runtime>,
    ui_tx: Sender<UIUpdate>,
    ui_rx: Receiver<UIUpdate>,
    state: UIState,
    message_cache: Option<MessageCache>,
}
```

2. Add `get_or_create_controller` method:
```rust
fn get_or_create_controller(&mut self, account_id: &str) -> Arc<TokioMutex<MailController>> {
    if let Some(controller) = self.mail_controllers.get(account_id) {
        return controller.clone();
    }
    
    // Create new controller for this account
    let controller = Arc::new(TokioMutex::new(MailController::new()));
    self.mail_controllers.insert(account_id.to_string(), controller.clone());
    controller
}

fn get_active_controller(&self) -> Option<Arc<TokioMutex<MailController>>> {
    self.active_account_id.as_ref()
        .and_then(|id| self.mail_controllers.get(id))
        .cloned()
}
```

3. Update `IntegratedUI::new()`:
```rust
pub fn new() -> Result<Self> {
    // ... existing code ...
    
    Ok(Self {
        mail_controllers: std::collections::HashMap::new(),
        active_account_id: state.account_manager.get_active_account_id().cloned(),
        runtime,
        ui_tx,
        ui_rx,
        state,
        message_cache,
    })
}
```

4. Update ALL methods that use `self.mail_controller` to use `self.get_active_controller()`:
   - `connect_imap()`
   - `fetch_folders()`
   - `fetch_messages()`
   - `send_email()`
   - etc.

### Item 5: Background Sync (1 hour)

**Location:** `src/presentation/ui_integrated.rs`

**Changes:**

1. Add method to spawn background sync:
```rust
fn spawn_background_sync(&self, account: Account) {
    let runtime = self.runtime.clone();
    let cache = self.message_cache.clone();
    let controller = self.get_or_create_controller(&account.id);
    
    runtime.spawn(async move {
        loop {
            // Wait for interval
            let interval = std::time::Duration::from_secs(
                account.check_interval_minutes as u64 * 60
            );
            tokio::time::sleep(interval).await;
            
            // Perform sync
            match Self::sync_account(&controller, &account, &cache).await {
                Ok(_) => {
                    // Update last sync timestamp
                    if let Some(cache) = &cache {
                        let _ = cache.update_account_last_sync(&account.id);
                    }
                }
                Err(e) => {
                    eprintln!("Sync error for {}: {}", account.email, e);
                }
            }
        }
    });
}

async fn sync_account(
    controller: &Arc<TokioMutex<MailController>>,
    account: &Account,
    cache: &Option<MessageCache>,
) -> Result<()> {
    // Connect if not connected
    let mut ctrl = controller.lock().await;
    
    // Connect to IMAP
    ctrl.connect_imap(
        &account.imap_server,
        &account.imap_port,
        &account.username,
        &account.password,
        account.imap_use_tls,
    ).await?;
    
    // Fetch folders
    let folders = ctrl.fetch_folders().await?;
    
    // Fetch messages from each folder
    for folder in &folders {
        let messages = ctrl.fetch_messages(folder, None).await?;
        
        // Cache messages if cache available
        if let Some(cache) = cache {
            // TODO: Cache messages with account_id
        }
    }
    
    Ok(())
}
```

2. Call from `new()` or after account creation:
```rust
// In IntegratedUI::new() or after loading accounts
for account in self.state.account_manager.get_accounts() {
    if account.enabled {
        self.spawn_background_sync(account.clone());
    }
}
```

### Item 6: Data Isolation (45 min)

**Location:** `src/data/message_cache.rs`

**Changes:**

1. Verify `account_id` columns exist in tables:
```sql
-- Should already have these from previous migrations
ALTER TABLE folders ADD COLUMN account_id TEXT;
ALTER TABLE messages ADD COLUMN account_id TEXT;
```

2. Update `get_folders()` to filter by account:
```rust
pub fn get_folders(&self, account_id: &str) -> Result<Vec<CachedFolder>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, path, unread_count, total_count 
         FROM folders 
         WHERE account_id = ?1
         ORDER BY name"
    )?;
    
    let folders = stmt.query_map([account_id], |row| {
        Ok(CachedFolder {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            path: row.get(3)?,
            unread_count: row.get(4)?,
            total_count: row.get(5)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;
    
    Ok(folders)
}
```

3. Update `get_messages()` to filter by account:
```rust
pub fn get_messages(&self, folder_id: i64, account_id: &str) -> Result<Vec<CachedMessage>> {
    // Add account_id check to WHERE clause
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT m.* FROM messages m
         JOIN folders f ON m.folder_id = f.id
         WHERE m.folder_id = ?1 AND f.account_id = ?2
         ORDER BY m.date DESC"
    )?;
    // ...
}
```

4. Verify tags and signatures already filter by account_id (they should from Phase 5)

5. Add integration test:
```rust
#[test]
fn test_account_data_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let cache = MessageCache::new(dir.path().to_path_buf()).unwrap();
    
    // Create two accounts
    let account1 = Account { id: "acc1".to_string(), /* ... */ };
    let account2 = Account { id: "acc2".to_string(), /* ... */ };
    
    cache.save_account(&account1).unwrap();
    cache.save_account(&account2).unwrap();
    
    // Create folder for account1
    // Create folder for account2
    
    // Verify account1 can only see their folders
    let folders1 = cache.get_folders("acc1").unwrap();
    assert_eq!(folders1.len(), 1);
    assert_eq!(folders1[0].account_id, "acc1");
    
    // Verify account2 can only see their folders
    let folders2 = cache.get_folders("acc2").unwrap();
    assert_eq!(folders2.len(), 1);
    assert_eq!(folders2[0].account_id, "acc2");
}
```

### Item 7: Testing & Polish (1 hour)

**Changes:**

1. Add to `src/data/account.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_account_switching() {
        let mut manager = AccountManager::new();
        
        let acc1 = Account::new("user1@example.com", "pass1", "imap.example.com", "993");
        let acc2 = Account::new("user2@example.com", "pass2", "imap.example.com", "993");
        
        manager.add_account(acc1.clone()).unwrap();
        manager.add_account(acc2.clone()).unwrap();
        
        // Set first as active
        manager.set_active_account(&acc1.id).unwrap();
        assert_eq!(manager.get_active_account_id(), Some(&acc1.id));
        
        // Switch to second
        manager.set_active_account(&acc2.id).unwrap();
        assert_eq!(manager.get_active_account_id(), Some(&acc2.id));
    }
}
```

2. Update `docs/USER_GUIDE.md`:
```markdown
## Multiple Accounts

Wixen Mail supports managing multiple email accounts simultaneously.

### Adding Accounts
1. Open **Tools > Manage Accounts** (Ctrl+M)
2. Click **Add New Account**
3. Fill in your email and server details
4. Click **Save**

### Switching Between Accounts
- Use the account dropdown in the toolbar (📧 icon)
- Or use keyboard shortcuts:
  - **Ctrl+1** - Switch to first account
  - **Ctrl+2** - Switch to second account
  - **Ctrl+3** - Switch to third account

### Background Sync
Each enabled account syncs automatically based on its check interval (default: 5 minutes).
```

3. Update `docs/KEYBOARD_SHORTCUTS.md`:
```markdown
### Account Management
- **Ctrl+M** - Open account manager
- **Ctrl+1** - Switch to account 1
- **Ctrl+2** - Switch to account 2
- **Ctrl+3** - Switch to account 3
```

4. Take screenshots:
   - Account switcher dropdown
   - Multiple accounts in manager
   - Switching between accounts

## Estimated Time Breakdown

| Item | Time | Complexity |
|------|------|------------|
| Item 4: Multi-Controller | 1.5 hours | High |
| Item 3: Account Switcher | 1 hour | Medium |
| Item 5: Background Sync | 1 hour | Medium |
| Item 6: Data Isolation | 45 min | Low |
| Item 7: Testing & Polish | 1 hour | Low |
| **Total** | **5.25 hours** | |

## Success Criteria

- ✅ Account switcher dropdown visible in toolbar
- ✅ Ctrl+1/2/3 shortcuts work
- ✅ Multiple controllers managed correctly
- ✅ Background sync running per account
- ✅ Data properly isolated by account_id
- ✅ All tests passing (110+)
- ✅ Documentation updated
- ✅ Screenshots added

## Phase 6 Completion
After completing items 3-7, Phase 6 will be **100% complete**.

Project status: ~87% toward v1.0 (from ~83%)

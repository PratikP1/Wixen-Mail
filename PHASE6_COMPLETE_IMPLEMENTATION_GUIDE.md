# Phase 6: Complete Implementation Guide

## Current Status
**Phase 6 Progress:** 55% Complete (Foundation Done)  
**Tests Passing:** 110/110 (100%)  
**Ready For:** Implementation of 7 remaining items

## What's Complete ✅

### 1. Account Model (`src/data/account.rs`)
- ✅ Account struct with 18 fields
- ✅ AccountManager service with CRUD methods
- ✅ Validation, display_name(), mark_synced()
- ✅ 7 unit tests passing

### 2. AccountManagerWindow UI (`src/presentation/account_manager.rs`)
- ✅ Full CRUD interface
- ✅ Create/Edit/Delete dialogs
- ✅ Provider auto-detection
- ✅ Keyboard shortcut (Ctrl+M)
- ✅ Tools menu integration

### 3. SQLite Persistence (`src/data/message_cache.rs`)
- ✅ Accounts table with 18 fields
- ✅ save_account() method
- ✅ load_accounts() method
- ✅ delete_account() method
- ✅ update_account_last_sync() method
- ✅ Base64 password encoding/decoding
- ✅ test_account_persistence passing

## What Remains (45%) - 7 Items

### Item 1: Migration Tool (30 min)

**File:** `src/data/account.rs`

**Add This Function:**
```rust
use crate::data::email_providers;

impl Account {
    /// Migrate from old AccountConfig to new Account
    pub fn from_account_config(config: &super::super::presentation::ui_integrated::AccountConfig) -> Self {
        let email = config.email.clone();
        
        // Detect provider from email
        let provider = if let Some(provider_name) = &config.selected_provider {
            Some(provider_name.clone())
        } else {
            email_providers::detect_provider(&email).map(|p| p.name.clone())
        };
        
        Account {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Primary Account".to_string(), // User can rename later
            email,
            imap_server: config.imap_server.clone(),
            imap_port: config.imap_port.clone(),
            imap_use_tls: config.imap_use_tls,
            smtp_server: config.smtp_server.clone(),
            smtp_port: config.smtp_port.clone(),
            smtp_use_tls: config.smtp_use_tls,
            username: config.username.clone(),
            password: config.password.clone(),
            enabled: true,
            check_interval_minutes: 5,
            provider,
            last_sync: None,
            color: "#4A90E2".to_string(), // Default blue
        }
    }
}
```

**Add Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migrate_from_account_config() {
        use crate::presentation::ui_integrated::AccountConfig;
        
        let config = AccountConfig {
            email: "user@gmail.com".to_string(),
            selected_provider: Some("Gmail".to_string()),
            imap_server: "imap.gmail.com".to_string(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "user@gmail.com".to_string(),
            password: "password123".to_string(),
        };
        
        let account = Account::from_account_config(&config);
        
        assert_eq!(account.email, "user@gmail.com");
        assert_eq!(account.name, "Primary Account");
        assert_eq!(account.imap_server, "imap.gmail.com");
        assert_eq!(account.enabled, true);
        assert!(account.id.len() > 0); // UUID generated
    }
}
```

### Item 2: UI Integration (1 hour)

**File:** `src/presentation/ui_integrated.rs`

**In IntegratedUI::new():**
```rust
// After creating account_manager, load accounts from database
if let Some(ref cache) = message_cache {
    if let Ok(accounts) = cache.load_accounts() {
        // Find if there's a saved active account (future: persist this)
        let active_id = accounts.first().map(|a| a.id.clone());
        state.account_manager.load(accounts, active_id);
    }
}
```

**Update handle_account_action() method:**
```rust
fn handle_account_action(&mut self, action: AccountAction) {
    match action {
        AccountAction::Create(account) => {
            if let Some(ref cache) = self.message_cache {
                match cache.save_account(&account) {
                    Ok(_) => {
                        self.state.status_message = format!("Account '{}' created", account.name);
                        // Reload accounts
                        if let Ok(accounts) = cache.load_accounts() {
                            let active = self.state.account_manager.get_active_account_id().cloned();
                            self.state.account_manager.load(accounts, active);
                        }
                    }
                    Err(e) => {
                        self.state.error_message = Some(format!("Failed to create account: {}", e));
                    }
                }
            }
        }
        AccountAction::Update(account) => {
            if let Some(ref cache) = self.message_cache {
                match cache.save_account(&account) {
                    Ok(_) => {
                        self.state.status_message = format!("Account '{}' updated", account.name);
                        // Reload accounts
                        if let Ok(accounts) = cache.load_accounts() {
                            let active = self.state.account_manager.get_active_account_id().cloned();
                            self.state.account_manager.load(accounts, active);
                        }
                    }
                    Err(e) => {
                        self.state.error_message = Some(format!("Failed to update account: {}", e));
                    }
                }
            }
        }
        AccountAction::Delete(account_id) => {
            if let Some(ref cache) = self.message_cache {
                match cache.delete_account(&account_id) {
                    Ok(_) => {
                        self.state.status_message = "Account deleted".to_string();
                        // Reload accounts
                        if let Ok(accounts) = cache.load_accounts() {
                            let active = self.state.account_manager.get_active_account_id().cloned();
                            self.state.account_manager.load(accounts, active);
                        }
                    }
                    Err(e) => {
                        self.state.error_message = Some(format!("Failed to delete account: {}", e));
                    }
                }
            }
        }
        AccountAction::SetActive(account_id) => {
            if let Err(e) = self.state.account_manager.set_active_account(&account_id) {
                self.state.error_message = Some(format!("Failed to set active account: {}", e));
            } else {
                self.state.status_message = "Active account changed".to_string();
                // TODO: Switch to this account's data
            }
        }
        _ => {}
    }
}
```

### Item 3: Account Switcher (1 hour)

**File:** `src/presentation/ui_integrated.rs`

**Add to toolbar (in render_toolbar method):**
```rust
// Account switcher dropdown
ui.separator();
ui.label("📧");

let active_display = if let Some(account) = self.state.account_manager.get_active_account() {
    account.display_name()
} else if !self.state.account_config.email.is_empty() {
    self.state.account_config.email.clone()
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

**Add switch_account method:**
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
        self.state.status_message = format!("Switched to {}", account.display_name());
    }
    
    // TODO: Load folders for new account
    // TODO: Switch to this account's controller
}
```

**Add keyboard shortcuts (in handle_keyboard_shortcuts):**
```rust
// Account shortcuts
if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num1)) {
    if let Some(account) = self.state.account_manager.get_accounts().get(0) {
        self.switch_account(&account.id);
    }
}
if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num2)) {
    if let Some(account) = self.state.account_manager.get_accounts().get(1) {
        self.switch_account(&account.id);
    }
}
if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num3)) {
    if let Some(account) = self.state.account_manager.get_accounts().get(2) {
        self.switch_account(&account.id);
    }
}
```

### Item 4: Multi-Controller (1.5 hours)

**File:** `src/presentation/ui_integrated.rs`

**Change IntegratedUI struct:**
```rust
pub struct IntegratedUI {
    state: UIState,
    ui_rx: Receiver<UIUpdate>,
    runtime: Arc<Runtime>,
    
    // Change from single controller to HashMap
    mail_controllers: std::collections::HashMap<String, Arc<TokioMutex<MailController>>>,
    active_account_id: Option<String>,
    
    message_cache: Option<MessageCache>,
    account_manager_open: bool,
}
```

**Update IntegratedUI::new():**
```rust
pub fn new(/* ... */) -> Self {
    // ...
    
    Self {
        state,
        ui_rx,
        runtime,
        mail_controllers: std::collections::HashMap::new(),
        active_account_id: None,
        message_cache,
        account_manager_open: false,
    }
}
```

**Add get_or_create_controller method:**
```rust
fn get_or_create_controller(&mut self, account_id: &str) -> Option<Arc<TokioMutex<MailController>>> {
    // Return existing controller if available
    if let Some(controller) = self.mail_controllers.get(account_id) {
        return Some(controller.clone());
    }
    
    // Get account info
    let account = self.state.account_manager.get_account(account_id)?;
    
    // Create new controller
    let (tx, rx) = async_channel::unbounded();
    let controller = Arc::new(TokioMutex::new(MailController::new(tx)));
    
    // Store controller
    self.mail_controllers.insert(account_id.to_string(), controller.clone());
    
    Some(controller)
}
```

**Update connect_imap to use active account:**
```rust
fn connect_imap(&mut self) {
    let account_id = if let Some(id) = &self.active_account_id {
        id.clone()
    } else {
        // Fallback to first enabled account
        if let Some(account) = self.state.account_manager.get_accounts()
            .iter()
            .filter(|a| a.enabled)
            .next() 
        {
            account.id.clone()
        } else {
            self.state.error_message = Some("No account available".to_string());
            return;
        }
    };
    
    let controller = match self.get_or_create_controller(&account_id) {
        Some(c) => c,
        None => {
            self.state.error_message = Some("Failed to create controller".to_string());
            return;
        }
    };
    
    let account = self.state.account_manager.get_account(&account_id).unwrap().clone();
    
    // Spawn connection task
    let runtime = self.runtime.clone();
    runtime.spawn(async move {
        let mut ctrl = controller.lock().await;
        if let Err(e) = ctrl.connect_imap(
            &account.imap_server,
            &account.imap_port,
            &account.username,
            &account.password,
            account.imap_use_tls,
        ).await {
            eprintln!("IMAP connection failed: {}", e);
        }
    });
}
```

### Item 5: Background Sync (1 hour)

**File:** `src/presentation/ui_integrated.rs`

**Add spawn_background_sync method:**
```rust
fn spawn_background_sync(&self) {
    let accounts: Vec<Account> = self.state.account_manager.get_accounts()
        .iter()
        .filter(|a| a.enabled)
        .cloned()
        .collect();
    
    let runtime = self.runtime.clone();
    let cache = self.message_cache.clone();
    
    for account in accounts {
        let account_id = account.id.clone();
        let check_interval = account.check_interval_minutes;
        let controller_opt = self.mail_controllers.get(&account_id).cloned();
        let cache_clone = cache.clone();
        
        runtime.spawn(async move {
            loop {
                // Wait for interval
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    check_interval as u64 * 60
                )).await;
                
                // Sync if we have a controller
                if let Some(controller) = &controller_opt {
                    let mut ctrl = controller.lock().await;
                    
                    // Fetch folders
                    if let Ok(folders) = ctrl.fetch_folders().await {
                        // Sync each folder
                        for folder in folders {
                            if let Ok(messages) = ctrl.fetch_messages(&folder, 50).await {
                                // Cache messages
                                if let Some(ref cache) = cache_clone {
                                    // TODO: Save messages to cache with account_id
                                }
                            }
                        }
                    }
                }
                
                // Update last sync timestamp
                if let Some(ref cache) = cache_clone {
                    cache.update_account_last_sync(&account_id).ok();
                }
            }
        });
    }
}
```

**Call in IntegratedUI::new():**
```rust
// At end of new() method
let ui = Self { /* ... */ };
ui.spawn_background_sync();
ui
```

### Item 6: Data Isolation (45 min)

**File:** `src/data/message_cache.rs`

**Verify account_id foreign key exists in folders table:**
```sql
CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,  -- Already exists from Phase 5
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    unread_count INTEGER DEFAULT 0,
    total_count INTEGER DEFAULT 0,
    UNIQUE(account_id, path)
);
```

**Verify account_id in messages table:**
```sql
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,  -- Add if not present
    uid INTEGER NOT NULL,
    folder_id INTEGER NOT NULL,
    -- ... other fields
    FOREIGN KEY (folder_id) REFERENCES folders(id),
    UNIQUE(account_id, folder_id, uid)
);
```

**Update get_folders to filter by account:**
```rust
pub fn get_folders(&self, account_id: &str) -> Result<Vec<CachedFolder>, Box<dyn std::error::Error>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, path, unread_count, total_count 
         FROM folders 
         WHERE account_id = ?
         ORDER BY name"
    )?;
    
    // ... rest of implementation
}
```

**Update get_messages to filter by account:**
```rust
pub fn get_messages(&self, folder_id: i64, account_id: &str) -> Result<Vec<CachedMessage>, Box<dyn std::error::Error>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT m.id, m.uid, m.folder_id, m.subject, m.from_addr, m.to_addr, 
                m.date, m.body_plain, m.body_html, m.flags, m.thread_id
         FROM messages m
         JOIN folders f ON m.folder_id = f.id
         WHERE m.folder_id = ? AND f.account_id = ?
         ORDER BY m.date DESC"
    )?;
    
    // ... rest of implementation
}
```

**Verify tags/signatures already filter by account_id:**
```rust
// These should already have account_id filtering from Phase 5
pub fn get_tags_for_account(&self, account_id: &str) -> Result<Vec<Tag>, Box<dyn std::error::Error>>
pub fn get_signatures_for_account(&self, account_id: &str) -> Result<Vec<Signature>, Box<dyn std::error::Error>>
```

### Item 7: Testing & Polish (1 hour)

**File:** `src/data/account.rs` (Add to tests)

**Add test_account_migration:**
```rust
#[test]
fn test_account_migration() {
    use crate::presentation::ui_integrated::AccountConfig;
    
    let config = AccountConfig {
        email: "test@example.com".to_string(),
        selected_provider: None,
        imap_server: "imap.example.com".to_string(),
        imap_port: "993".to_string(),
        imap_use_tls: true,
        smtp_server: "smtp.example.com".to_string(),
        smtp_port: "465".to_string(),
        smtp_use_tls: true,
        username: "test".to_string(),
        password: "pass".to_string(),
    };
    
    let account = Account::from_account_config(&config);
    assert_eq!(account.email, "test@example.com");
    assert_eq!(account.imap_server, "imap.example.com");
}
```

**File:** `src/data/message_cache.rs` (Add to tests)

**Add test_data_isolation:**
```rust
#[test]
fn test_data_isolation() {
    let cache = MessageCache::new_in_memory().unwrap();
    
    // Create two accounts
    let acc1 = Account {
        id: "acc1".to_string(),
        name: "Account 1".to_string(),
        email: "user1@example.com".to_string(),
        // ... fill in fields
    };
    let acc2 = Account {
        id: "acc2".to_string(),
        name: "Account 2".to_string(),
        email: "user2@example.com".to_string(),
        // ... fill in fields
    };
    
    cache.save_account(&acc1).unwrap();
    cache.save_account(&acc2).unwrap();
    
    // Create folders for each account
    // Create messages for each account
    // Verify get_folders filters by account
    // Verify get_messages filters by account
    // Verify no cross-account data access
}
```

**Documentation Updates:**

Update `docs/USER_GUIDE.md`:
```markdown
## Multiple Account Support

Wixen Mail supports managing multiple email accounts in a single application.

### Adding Accounts

1. Open Tools > Manage Accounts (Ctrl+M)
2. Click "Add New Account"
3. Enter account details or select a provider
4. Click Save

### Switching Accounts

- Use the account dropdown in the toolbar
- Or press Ctrl+1, Ctrl+2, Ctrl+3 for the first three accounts

### Account Management

- Edit: Click the edit button next to an account
- Delete: Click the delete button (requires confirmation)
- Enable/Disable: Toggle the checkbox

Each account maintains separate:
- Folders and messages
- Tags
- Signatures
- Settings
```

Update `docs/KEYBOARD_SHORTCUTS.md`:
```markdown
### Account Management

| Shortcut | Action |
|----------|--------|
| Ctrl+M | Manage Accounts |
| Ctrl+1 | Switch to Account 1 |
| Ctrl+2 | Switch to Account 2 |
| Ctrl+3 | Switch to Account 3 |
```

## Implementation Checklist

### Day 1 (2.5 hours)
- [ ] Item 1: Migration Tool (30 min)
- [ ] Item 2: UI Integration (1 hour)
- [ ] Item 3: Account Switcher (1 hour)

### Day 2 (2.5 hours)
- [ ] Item 4: Multi-Controller (1.5 hours)
- [ ] Item 5: Background Sync (1 hour)

### Day 3 (2 hours)
- [ ] Item 6: Data Isolation (45 min)
- [ ] Item 7: Testing & Polish (1 hour)
- [ ] Final test run
- [ ] Documentation review
- [ ] Screenshots

## Testing Strategy

1. **Unit Tests** - Test each component in isolation
2. **Integration Tests** - Test account CRUD with persistence
3. **Manual Testing** - Test UI interactions
4. **Performance Testing** - Test with 5 accounts
5. **Isolation Testing** - Verify no cross-account data leakage

## Success Criteria

✅ All 7 items implemented  
✅ 110+ tests passing  
✅ No regressions  
✅ Account switching works smoothly (<1s)  
✅ Multi-controller manages accounts correctly  
✅ Background sync working for all enabled accounts  
✅ Complete data isolation verified  
✅ Documentation updated  
✅ Screenshots added  

## Phase 6 Complete

When all items are implemented:
- Phase 6: 100% ✅
- Project: ~87% toward v1.0
- Ready for Phase 7: Email Rules/Filters

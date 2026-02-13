# Phase 8: Contact Management - Architecture

## Layered Design

### Data Layer (`src/data/message_cache.rs`)
- Adds `ContactEntry` entity
- Adds `contacts` table
- Adds account-scoped CRUD/search methods

### Application Layer (`src/application/contacts.rs`)
- Existing in-memory `ContactManager` remains as a business abstraction
- Persistent operations executed through `MessageCache` for integrated UI flow

### Presentation Layer
- New `ContactManagerWindow` (`src/presentation/contact_manager.rs`)
  - Accessible CRUD/search UI
  - Emits `ContactAction` events
- `IntegratedUI` wiring:
  - Tools menu entry + keyboard shortcut
  - Event handling (`handle_contact_action`)
  - Composition autocomplete feed from cache search

### Composition Integration (`src/presentation/composition.rs`)
- Adds `contact_suggestions` state
- Adds setter `set_contact_suggestions`
- Renders suggestion chips below To field

## Data Flow

1. User opens Contact Manager (`Ctrl+Shift+C`)
2. UI loads account contacts via `get_contacts_for_account`
3. CRUD form emits action
4. `IntegratedUI` persists action via `save_contact`/`delete_contact`
5. Compose window queries `search_contacts_for_account` and renders suggestions

## Isolation & Safety

- Contact records keyed by account
- Queries always include `account_id`
- Basic email validation in UI form
- No cross-account sharing by default

## Extensibility Path

- Add contact groups/distribution lists
- Add CSV/vCard import/export adapters
- Add richer validation and deduplication rules
- Expand autocomplete to CC/BCC

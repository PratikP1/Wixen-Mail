# Phase 1: Folders and conversations - Pattern Map

**Mapped:** 2026-08-29
**Files analyzed:** 24 (8 new, 16 modified)
**Analogs found:** 22 / 24

Every path in this document is relative to the Wixen-Mail repository root. Line
numbers were read this session and are accurate as of the working tree today.

---

## File Classification

| New/Modified file | New? | Role | Data flow | Closest analog | Match |
|---|---|---|---|---|---|
| `src/service/protocols/imap/mailbox_name.rs` | modify | utility | transform | its own `decode` (same file, lines 41-90) | exact |
| `src/service/protocols/imap.rs` | modify | service (protocol) | request-response | `set_subscribed` (lines 836-858) | exact |
| `src/application/mail_controller.rs` | modify | service facade | request-response | `set_subscribed` (lines 525-530) | exact |
| `src/data/message_cache/mod.rs` | modify | schema/config | batch | `initialize_schema` + `ensure_column_exists` (1290-1335, 3050-3074) | exact |
| `src/data/message_cache/folders.rs` | modify | model (persistence) | CRUD | `save_folder` / `set_folder_server_facts` / `folder_server_facts` (23-64, 105-145) | exact |
| `src/data/message_cache/messages.rs` | modify | model (persistence) | CRUD | `folders.rs::folder_server_facts` for the read; `bodies.rs` for the write | role-match |
| **`src/application/shared_folders_migration.rs`** | new | migration | batch, row-moving | `bodies.rs::migrate_inline_bodies` (458-497) + its call site `mod.rs:1174-1182` | exact |
| **`src/application/thread_identity.rs`** | new | service (pure) | transform | `application/threading.rs` (`as_stored`, `continuing`) | role-match |
| **`src/application/conversations.rs`** | new | service | CRUD (aggregate read) | `application/local_folders.rs` (pure decision module with a table of constants) | role-match |
| **`src/application/favourites.rs`** | new | service (pure) | CRUD | `application/local_folders.rs` | role-match |
| **`src/application/emptying.rs`** | new | service | batch, partial-failure | `local_folders::deleting` for the decision; no partial-failure reporter exists | partial |
| `src/application/local_folders.rs` | modify | service (pure) | transform | itself: `LOCAL_PREFIX`, `is_local`, `for_account` | exact |
| `src/application/import_tree.rs` | modify | service (pure) | transform | `imap.rs::ImapFolder`'s `path` / `display_path` split | role-match |
| `src/application/mail_sync.rs` | modify | service (orchestration) | batch | `store_folders` (412-439) | exact |
| `src/presentation/message_columns.rs` | modify | presentation (pure) | transform | itself: `heading` / `key` / `sort_expression` (69-140) | exact |
| `src/presentation/message_rows.rs` | modify | presentation (pure) | transform | `cell_text` (24-80) | exact |
| **`src/presentation/folder_tree.rs`** | new | presentation (pure) | transform | `wx_app.rs::collect_rows` (10101-10118) + `the_row_to_land_on` | role-match |
| `src/presentation/wx_app.rs` | modify | controller (UI) | event-driven | `UIUpdate::FoldersLoaded` (10527-10596), `delete_the_chosen_search` (6035-6090), menu block (5255-5275) | exact |
| `src/presentation/wx_settings.rs` | modify | component | request-response | `start_in_all_inboxes` check box (964-978, 1981) | exact |
| `src/presentation/wx_compose.rs` | modify | component | transform | its own `Re: ` prepend at 161-175 | exact |
| `src/data/config.rs` | modify | config + guard | request-response | `start_in_all_inboxes` (142-150, 458) and `test_every_setting_somebody_can_change_is_read_by_something` (1305-1343) | exact |
| `guards/guards.toml` | modify | config | n/a | last record, lines 9068-9078 | exact |
| `docs/KEYBOARD_SHORTCUTS.md` | modify | doc | n/a | existing entries | exact |
| `docs/changelog.md` | modify | doc | n/a | `[Unreleased]` section | exact |

---

## Pattern Assignments

### `src/service/protocols/imap.rs` (service, request-response) - three new verbs

**Analog:** `set_subscribed`, `src/service/protocols/imap.rs:836-858`. This is the
newest server-writing verb and shows the whole four-part shape: gate, timeout,
library call, error map.

```rust
/// Subscribe to a mailbox, or drop the subscription.
///
/// Written to the server rather than kept here, so the same choice holds in
/// every client the account is opened in.
pub async fn set_subscribed(&mut self, path: &str, subscribed: bool) -> Result<()> {
    self.may_i("change which folders you are subscribed to")?;
    let outcome = if subscribed {
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.subscribe(path),
            "subscribing to a folder",
        )
        .await?
    } else {
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.unsubscribe(path),
            "unsubscribing from a folder",
        )
        .await?
    };
    outcome.map_err(protocol_error("Could not change the folder subscription"))
}
```

**The gate** (`imap.rs:1067-1069`), one line, first line of every writing verb:

```rust
fn may_i(&self, doing: &str) -> Result<()> {
    crate::service::outward::permitted(self.may_change, doing)
}
```

**The error map** (`imap.rs:1836-1844`). Reuse unchanged. It collapses `No`, `Bad`
and `Io` into one `Error::Protocol`, so if a plan needs "the server said no" told
apart from "the connection dropped", that distinction has to be drawn at the call
site before this runs:

```rust
fn protocol_error(doing: &'static str) -> impl Fn(async_imap::error::Error) -> Error {
    move |error| {
        Error::Protocol(format!(
            "{doing}: {}",
            redact_provider_message(&error.to_string())
        ))
    }
}
```

`COMMAND_TIMEOUT` is at `imap.rs:81`. `with_timeout` is at `imap.rs:1846-1852` and
names the step in its failure so the message is actionable.

**Do not** hand-write the command string. `set_flag` (`imap.rs:1071-1092`) uses
`run_command_and_check_ok` and its doc comment says why that exception exists: the
library's flag helper does not check the tagged response. `Session::create`,
`rename` and `delete` do check it, so they are used directly.

---

### Testing a new verb: which loopback harness, and why

**Two harnesses exist in the same `#[cfg(test)]` module and they are not
interchangeable.**

**`a_server_that_can(capabilities)`** (`imap.rs:2394-2456`, inside
`pub(crate) mod against_a_server_that_answers`) is the shared script. Its verb
match arm lists `"UID" | "STORE" | "COPY" | "MOVE" | "EXPUNGE" | "NOOP" | "CLOSE"
| "SUBSCRIBE" | "UNSUBSCRIBE"` and its fallback is deliberate:

```rust
// Anything unrecognised is refused rather than ignored, so a
// script that has fallen behind the client fails the test in
// the moment instead of leaving it to wait out two minutes,
// which reads as a slow machine.
_ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
```

So `CREATE`, `RENAME` and `DELETE` come back `BAD unscripted` here. A new verb
either adds itself to that arm, or uses the other harness.

**`a_server_answering(answer)`** (`imap.rs:3135-3153`) is the per-test server, with
a permissive `_ => Turn::Say(format!("{tag} OK done\r\n"))` fallback. **New verbs
should use this one.** Two reasons, both in the tree: its own doc comment says
"Preferred over widening the shared script above", and widening
`a_server_that_can` changes the server every existing mailbox-write test runs
against. Use `a_server_that_refuses(capabilities, "CREATE")` from the shared module
for the refusal direction, since that path needs no new verb in the script.

**Test shape to copy** (`imap.rs:2536-2569`), including the substring trap it
documents:

```rust
#[tokio::test]
async fn test_changing_a_subscription_names_the_folder_in_both_directions() {
    let server = a_server_that_can("").await;
    let mut session = signed_in_to(&server).await;

    waiting_for(session.set_subscribed("Work", true), "the subscription")
        .await
        .expect("the folder to be subscribed");

    // A leading space on the first needle, because the transcript is
    // searched by substring and `SUBSCRIBE "Work"` also matches the line
    // that says UNSUBSCRIBE.
    let transcript = server.transcript().await;
    let subscribed = server
        .when_told(" SUBSCRIBE \"Work\"")
        .await
        .unwrap_or_else(|| panic!("the folder was never subscribed: {transcript:?}"));
}
```

`was_told` answers whether; `when_told` answers when, and is what proves two
commands were sent rather than one line matched twice. The refusal direction uses
`the_failure(...)` and asserts on the words, as at `imap.rs:3163-3176`.

---

### `src/service/protocols/imap/mailbox_name.rs` (utility, transform) - the encoder (D-41)

**Analog:** the file's own `decode`, lines 41-90, and the engine above it. The
module comment already commissions this work: *"An encoder belongs here the day
something creates or renames a mailbox."*

**Reuse the engine as it stands** (lines 26-38). Do not configure a second one:

```rust
/// Base64 as RFC 3501 spells it: the standard alphabet with `,` for 63.
static MODIFIED_BASE64: LazyLock<GeneralPurpose> = LazyLock::new(|| {
    let alphabet =
        Alphabet::new("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+,")
            .unwrap_or(base64::alphabet::STANDARD);
    let config = GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::RequireNone)
        .with_decode_allow_trailing_bits(true);
    GeneralPurpose::new(&alphabet, config)
});
```

`with_encode_padding(false)` is already set, so the engine is encode-ready today.

**Structure to mirror:** `decode` is a scanner with a private `decode_run(chunk)`
helper (lines 76-90) that does the Base64-over-UTF-16BE work. `encode` should be a
scanner with a private `encode_run` helper doing the inverse, so the round-trip
test has two symmetric pieces to pin.

**The two special cases decode already names, which encode must invert:**
`&-` is a literal ampersand (line 62), and a run is UTF-16BE code units, so a
character outside the BMP becomes a surrogate pair.

**Test file:** tests go in the same file's `mod tests` (line 92 onward), beside
`test_an_ascii_name_is_unchanged`. The round-trip test against the existing decoder
is what makes the encoder trustworthy.

---

### `src/application/shared_folders_migration.rs` (new; migration, batch, moves rows) - D-19 and D-40

**Analog: `MessageCache::migrate_inline_bodies`, `src/data/message_cache/bodies.rs:452-497`.**
This is the only existing migration that moves rows rather than adding a column,
and it is the pattern that decides how D-19 is made safe. Its whole shape:

```rust
/// Move any bodies still stored inline in `messages` into this table.
///
/// Databases written by earlier versions hold them in the old columns.
/// Returns how many were moved. The inline copies are cleared afterwards so
/// the space is actually reclaimed, but the columns themselves stay, because
/// a column that shipped is never dropped from under a user's database.
pub fn migrate_inline_bodies(&self) -> Result<usize> {
    let mut stmt = self
        .conn
        .prepare_cached(
            "SELECT id, body_plain, body_html FROM messages
             WHERE body_plain IS NOT NULL OR body_html IS NOT NULL",
        )
        .map_err(|e| Error::Other(format!("Failed to find inline bodies: {}", e)))?;

    let pending: Vec<(i64, Option<String>, Option<String>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| Error::Other(format!("Failed to read inline bodies: {}", e)))?
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| Error::Other(format!("Failed to read inline body row: {}", e)))?;

    let moved = pending.len();
    for (id, plain, html) in pending {
        self.save_message_body(id, plain.as_deref(), html.as_deref())?;   // land it first
        self.conn
            .execute(
                "UPDATE messages SET body_plain = NULL, body_html = NULL WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear inline body: {}", e)))?;
    }

    if moved > 0 {
        tracing::info!("Moved {} message bodies out of the messages table", moved);
    }
    Ok(moved)
}
```

**Five properties to copy exactly:**

1. **Read the whole candidate set into a `Vec` first**, then act. Never hold a
   `prepare_cached` statement open while writing to the same table.
2. **Land the new copy before clearing the old one**, per message. This is
   literally D-19's "nothing is removed until every message has landed", and this
   function already does it one row at a time.
3. **Return the count.** `moved` is the return value, not just a log line. The
   comment at 487-494 is explicit that the log line is not part of the contract
   and the count is: *"the migration's actual effect (rows moved, inline copies
   cleared) is what `test_existing_inline_bodies_are_migrated_not_lost` pins."*
   D-19's spoken summary reads from the returned count.
4. **Nothing is dropped.** The old columns stay. For D-19 that means the old
   per-account folder rows are emptied, not deleted, unless a separate decision
   says otherwise.
5. **Idempotent by its `WHERE` clause.** It selects only rows still in the old
   shape, so a second open finds nothing and does nothing.

**Call site pattern**, `src/data/message_cache/mod.rs:1174-1182`, run once at open,
non-fatal:

```rust
// Databases written by earlier versions keep bodies inline in the
// messages table. Move them across on open so the space is reclaimed
// and the listing queries stop reading them. A failure here is not
// fatal: the bodies are still readable where they are, and the next
// open tries again.
if let Err(e) = cache.migrate_inline_bodies() {
    tracing::warn!("Could not move inline message bodies: {}", e);
}
```

**Where the D-19 migration differs, and what has no analog.** RESEARCH.md places it
in `application/`, "so it belongs where it can be tested without a UI", while every
existing migration is a `MessageCache` method called from `MessageCache::new`. The
planner must choose one and say which. The count-and-report half, and D-40's fresh
uid with the original recorded beside it, have **no analog anywhere in the tree**:
`UNIQUE(folder_id, uid)` is at `mod.rs:1324` and nothing today reassigns a uid.
That part is new code and needs its own tests, driven by two accounts each holding
the same uid in their local Trash.

**Additive column for D-40 and D-22**, `mod.rs:3050-3074`. Both go through this and
nothing else:

```rust
self.ensure_column_exists("folders", "holds_all_mail", "INTEGER NOT NULL DEFAULT 0")?;
```

`ensure_column_exists` rejects a table or column name that is not
`[A-Za-z0-9_]+` before it interpolates, then checks `columns_of` and only then
`ALTER TABLE ... ADD COLUMN`. `parent_id` is nullable per D-22, so it takes no
`NOT NULL DEFAULT`.

---

### `src/data/message_cache/folders.rs` (model, CRUD) - `parent_id`, pins, per-folder view state

**Analog:** `save_folder` (lines 23-64) and the `set_folder_server_facts` /
`folder_server_facts` pair (lines 105-145).

**The upsert that must not become a replace.** Its doc comment is the constraint
D-18 and D-22 both live under and should be read in full at lines 8-22. The code:

```rust
pub fn save_folder(&self, folder: &CachedFolder) -> Result<i64> {
    self.conn
        .query_row(
            "INSERT INTO folders (account_id, name, path, folder_type, unread_count, total_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(account_id, path) DO UPDATE SET
                 name = excluded.name,
                 folder_type = excluded.folder_type
             RETURNING id",
            params![ /* ... */ ],
            |row| row.get(0),
        )
        .map_err(|e| Error::Other(format!("Failed to save folder: {}", e)))
}
```

Note what the `DO UPDATE SET` list deliberately omits: the counts, because the sync
writes them and blanking them empties the tree for as long as a sync takes. A new
`parent_id` belongs in that list only if the sync is the one authority on it.

**The write-a-fact / read-the-facts pair** is the shape for `parent_id`, a pin, and
per-folder view state. Copy both halves:

```rust
// The write, folders.rs:105-118
self.conn
    .execute(
        "UPDATE folders SET holds_all_mail = ?1, subscribed = ?2 WHERE id = ?3",
        params![i64::from(holds_all_mail), i64::from(subscribed), folder_id],
    )
    .map_err(|e| Error::Other(format!("Failed to record the folder facts: {}", e)))?;

// The read, folders.rs:120-145: one query per account, returned as a map keyed
// by path, so the caller does not go back to the database per folder.
pub fn folder_server_facts(
    &self,
    account_id: &str,
) -> Result<std::collections::HashMap<String, (bool, bool)>>
```

Booleans cross the SQL boundary as `i64::from(bool)`, never as a string.

**Ordering inside an account branch (D-13)** is already done and unchanged, at the
bottom of `get_folders_for_account` (lines 288-296):

```rust
folders.sort_by_key(|folder| {
    crate::common::types::tree_position(
        crate::common::types::FolderType::from_stored(&folder.folder_type),
        &folder.name,
    )
});
```

---

### `src/presentation/wx_app.rs` `UIUpdate::FoldersLoaded` (controller, event-driven) - D-13, D-17, D-21, D-28, D-29

**Analog: itself, lines 10527-10596.** This is the block being restructured, and
the branch convention D-17 and D-28 must follow is written inside it twice.

```rust
UIUpdate::FoldersLoaded(folders) => {
    {
        let mut s = lock_state(state);
        s.folders = folders.clone();
    }
    let was_on = what_the_cursor_was_on(folder_tree);
    folder_tree.delete_all_items();
    if let Some(root) = folder_tree.add_root("Mail Folders", None, None) {
        // First, because it is where somebody with more than one
        // account starts, and because arrowing past it to reach a
        // named folder costs one keystroke while hunting for it at the
        // bottom of a list of twenty costs twenty.
        folder_tree.append_item(&root, ALL_INBOXES, None, None);
        for f in folders {
            folder_tree.append_item(&root, f, None, None);
        }
        // Labels last and under a branch of their own, so arrowing
        // through the folders somebody opens every day does not pass
        // through a list of labels first. No branch at all when there
        // are none, rather than an empty one to arrow into.
        let labels = lock_state(state).labels.clone();
        if !labels.is_empty()
            && let Some(branch) = folder_tree.append_item(&root, "Labels", None, None)
        {
            for (_, name) in &labels {
                folder_tree.append_item(&branch, &label_row(name), None, None);
            }
            folder_tree.expand(&branch);
        }
        // ... saved searches, same shape ...
        folder_tree.expand(&root);
        // Back where it was. A sync finishing on a timer used to take
        // the cursor away mid-list with only a count spoken.
        land_the_cursor(folder_tree, &root, was_on.as_deref());
        // A saved search keeps its row through a rebuild even when its
        // name has just changed. What is open is held as the row's
        // path, which a rename does not touch, while `land_the_cursor`
        // matches on the row's words, which a rename does. Without
        // this, renaming a search takes the cursor to the top of the
        // tree and reads out a different row.
        let renamed = the_chosen_saved_search(&lock_state(state));
        if let Some(chosen) = renamed {
            select_row_named(
                folder_tree,
                &crate::application::saved_searches::a_row_for(chosen.name()),
            );
        }
    }
    let msg = how_many_loaded(folders.len(), "folder");
    frame.set_status_text(&msg, 0);
    let _ = a11y.announce_topic(&msg, Priority::Low, "folders");
}
```

Copy from it: `delete_all_items` then rebuild whole; `if !x.is_empty() && let Some(branch) = append_item(...)`
for a named branch omitted when empty (D-17 "On this computer", D-28 Favourites);
`expand(&branch)` after filling one; `land_the_cursor` last; and the status text
plus `announce_topic` at `Priority::Low` at the end, so a timer-driven sync does not
flood (guardrail 5).

The comment at 10578-10584 is the written statement of the D-25 defect. Every new
row type this phase adds needs the identity fix, not the label match.

**`land_the_cursor` and `collect_rows`** (`wx_app.rs:10074-10118`) are the parallel-vector
pattern, and D-25's amendment says to extend this rather than use tree item data:

```rust
fn land_the_cursor(tree: &TreeCtrl, root: &TreeItemId, was: Option<&str>) {
    let mut items = Vec::new();
    let mut labels = Vec::new();
    collect_rows(tree, root, &mut items, &mut labels);
    // ... the_row_to_land_on(was, &labels, start_at) -> index into `items`
}

/// Every row under one item, in the order somebody arrowing down meets them.
///
/// Depth first, because that is the order a tree reads: a branch, then what
/// is on it, then the next branch.
fn collect_rows(
    tree: &TreeCtrl,
    parent: &TreeItemId,
    items: &mut Vec<TreeItemId>,
    labels: &mut Vec<String>,
) {
    let mut child = tree.get_first_child(parent).map(|(child, _)| child);
    while let Some(current) = child {
        if let Some(text) = tree.get_item_text(&current) {
            items.push(current.clone());
            labels.push(text);
        }
        collect_rows(tree, &current, items, labels);
        child = tree.get_next_sibling(&current);
    }
}
```

**The extension to build:** a third vector of stable identities filled in the same
depth-first order, so `items[i]`, `labels[i]` and `identities[i]` stay in step by
position. `select_row_named` (10056-10072) shows the lookup shape: build the
vectors, `position(...)`, index into `items`, `select_item` then `ensure_visible`.
For D-25 the `position` predicate matches an identity, not a label.

**Do not** hang anything off `TreeCtrl` item custom data. RESEARCH.md Pitfall 2
measured the leak: a process-global registry that `delete_all_items` does not clear
and `cleanup_all_custom_data` returns early on for any childless item, so it never
clears a leaf, and this tree is rebuilt on every sync.

**Also note** `land_the_cursor` reads `ConfigManager::load_stored` only when nothing
was selected, with a comment saying why (10082-10089). A new setting read during a
rebuild follows that: read where the answer can be used, not on every rebuild.

---

### `src/presentation/message_columns.rs` (presentation, transform) - D-02 and D-03

**Analog: itself.** One enum answers display, storage and sort, from three parallel
`match` arms with no shared string surgery between them. D-02 is only safe while
that stays true.

```rust
/// The column heading, and what a screen reader reads for the column.
pub fn heading(&self) -> &'static str {
    match self {
        MessageColumn::Thread => "Thread",
        // ... one arm per variant, a fixed &'static str
    }
}

/// The identifier used when the layout is stored.
pub fn key(&self) -> &'static str {
    match self {
        MessageColumn::Thread => "thread",
        // ...
    }
}

/// The SQL expression this column sorts on.
///
/// Fixed strings chosen by matching on the enum, never built from anything
/// a user typed, because the result is interpolated into a query.
fn sort_expression(&self) -> &'static str {
    match self {
        MessageColumn::Received => "COALESCE(m.internaldate, m.date)",
        MessageColumn::Thread => "m.thread_id",
        // ...
    }
}
```

`MessageColumn::ALL` (lines 48-64) is a fixed-size array; adding a variant changes
its length and every `match` above goes red at compile time. That is the mechanism
keeping display and sort together, and it is why D-02's conversation aggregate
should be a **fourth method on the same enum** (a `conversation_sort_expression`, or
a variant of `sort_expression` taking a view), with one fixed string per arm.

**Anti-pattern named in RESEARCH.md:** building the aggregate by string surgery on
`sort_expression()`. Fixed strings per arm in both methods, or D-02's two halves
come apart.

`By::term()` (lines 176-183) is where a column and a direction become an `ORDER BY`
term, and is what a conversation `ORDER BY` composes through.

**`apply_columns`, `wx_app.rs:6210-6226`,** is the other consumer and the reason
hiding a column rebuilds rather than sets width zero:

```rust
/// Hiding rebuilds rather than setting a width of zero. A zero width column
/// still exists in the UI Automation tree and a screen reader may still read
/// it, which is the kind of defect that is invisible to sighted users and
/// audible to everyone else. Rebuilding is cheap in virtual mode because there
/// are no rows to restore, only a count to set again.
fn apply_columns(list: &ListCtrl, layout: &ColumnLayout) {
    list.clear_all();
    for (position, column) in layout.visible().iter().enumerate() {
        let width = match column { /* per-variant */ };
        list.insert_column(position as i64, column.heading(), ListColumnFormat::Left, width);
    }
}
```

D-05's adaptive Thread column goes through `layout.visible()` and this rebuild, not
through a width.

---

### `src/presentation/message_rows.rs` (presentation, transform) - the conversation row

**Analog: `cell_text`, lines 24-80.** One `match` over `MessageColumn`, every arm
returning a `String` that stands on its own:

```rust
pub fn cell_text(
    message: &MessageItem,
    column: MessageColumn,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    match column {
        // The cell says what it means rather than "Yes".
        // ... In practice the headings are not being read here, so "Yes" was a
        // word with nothing attached to it and the unread state was never
        // spoken at all. A cell that stands on its own is worth more than one
        // that depends on a heading being announced.
        MessageColumn::Unread => if message.read { "" } else { "Unread" }.to_string(),
        MessageColumn::Subject => {
            if message.subject.trim().is_empty() { "No subject".to_string() }
            else { message.subject.clone() }
        }
        MessageColumn::Thread => thread_cell(message),
        MessageColumn::Flagged => if message.starred { "Flagged" } else { "" }.to_string(),
        // ...
    }
}
```

Two rules to carry into a `conversation_cell_text`: **a cell is self-describing**
(D-03's "5 messages, 2 unread" fits this exactly, and today's raw thread id
violates it), and **the negative case is the empty string**, which costs no
listening time. `conversation_size` at line 138 is the existing conversation-aware
helper in this file.

---

### `src/data/config.rs` + `src/presentation/wx_settings.rs` - the five settings (D-42)

**Analog: `start_in_all_inboxes`, traced end to end.** Four places, all of which
each of the five new settings needs.

**1. The field**, `config.rs:140-150`. The doc comment says what it does *and why
the default is what it is*, and `#[serde(default)]` is not optional:

```rust
/// ... Off by default, because on is a change to where everybody opens and
/// somebody with one account has no use for a combined list of one.
#[serde(default)]
pub start_in_all_inboxes: bool,
```

`#[serde(default)]` or `#[serde(default = "...")]` is what stops every settings
file already on disk failing to parse. `test_a_settings_file_written_before_directories_existed_still_reads`
(`config.rs:1245`) exists because that has been a real failure.

**2. The default**, `config.rs:458`, inside `impl Default for AppConfig`:

```rust
start_in_all_inboxes: false,
```

Note `add_signature_automatically: default_true()` two lines above: a `true`
default uses that helper. Three of the five new settings default to yes.

**3. The control**, `wx_settings.rs:960-978`, on the Reading page:

```rust
// This one does something, which is the difference. The folder tree opens
// with no row chosen, so mail is listed only once somebody arrows onto a
// folder; ticking this lands them in the combined inbox instead.
let start_in_all_inboxes = CheckBox::builder(panel)
    .with_label("Start in All &Inboxes")
    .build();
start_in_all_inboxes.set_value(config.start_in_all_inboxes);
set_accessible_name_and_description(
    &start_in_all_inboxes,
    "Start in All Inboxes",
    "Open showing every account's inbox in one list, rather than with no folder chosen",
);
list_sec.add(&start_in_all_inboxes, 0, SizerFlag::Left | SizerFlag::All, 4);
```

**`with_label` is load-bearing and is not optional.** A check box built with an
empty label and named only through `set_accessible_name` has a name under NVDA and
none under Narrator. `tests/checkbox_labels.rs` is the guard and its module comment
(lines 1-23) is the explanation. The five new boxes must pass it.

The group D-42 asks for is a `StaticBoxSizer`, built by the one-line helper at
`wx_settings.rs:121-123`:

```rust
fn section(parent: &Panel, label: &str) -> StaticBoxSizer {
    StaticBoxSizerBuilder::new_with_label(Orientation::Vertical, parent, label).build()
}
```

Two of the five are two-option choices, not booleans (D-07 delete scope, D-24
unread announcement). The `Choice` pattern is at `wx_settings.rs:941-953` with
`set_accessible_name(&sort_choice, "Default sort order")`, and `cfg.font_family`
at 1985-1990 shows the house rule for reading one back: **by the words shown, not
by the row number**, with the reason written there.

**4. The write-back**, `wx_settings.rs:1980-1981`, under a `// Reading` comment
matching the page:

```rust
// Reading
cfg.start_in_all_inboxes = w.start_in_all_inboxes.get_value();
```

Plus the field on both control structs (`wx_settings.rs:81` and `:896`) and both
destructurings (`:225`, `:360`, `:1218`).

**5. Something must act on it.** `wx_app.rs:10086` reads it. A setting nothing reads
fails the guard below.

**D-42's one-constant rule** has its guard already: `test_the_settings_screen_does_not_write_the_section_name_out_itself`,
`tests/house_style.rs:2341-2387`. If the new group's name is ever said in a
sentence elsewhere, it comes from one `pub const` in the application layer, the way
`application::allowed::SETTINGS_SECTION` does, and the check reads:

```rust
const WHERE_THE_SECTION_IS_LABELLED: &str = "src/presentation/wx_settings.rs";
// The name as it stands, not the name it happens to have today. Written
// out as a literal here, this check would go on forbidding "Allow Changes"
// after somebody renamed the section to something else and typed the new
// name in, which is the same fault one step along.
let written_out = format!("\"{}\"", wixen_mail::application::allowed::SETTINGS_SECTION);
```

---

### `src/data/config.rs` - the mirror guard (D-43)

**Analog: `test_every_setting_somebody_can_change_is_read_by_something`, `config.rs:1305-1343`,
with its two helpers above it. D-43 reuses the helpers rather than writing new ones.**

```rust
fn stored_setting_names(source: &str) -> Vec<String> {
    let start = source
        .find("pub struct AppConfig {")
        .expect("the settings struct");
    let body = &source[start..];
    let end = body.find("\n}").expect("the struct ends");

    body[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub "))
        .filter_map(|line| line.split_once(':'))
        .map(|(name, _)| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Every source file that ships, except the two that own a setting rather
/// than acting on one.
fn files_that_act() -> Vec<std::path::PathBuf> {
    // walks `src` for `*.rs`, excluding:
    //   `config.rs` defines and stores a setting and
    //   `wx_settings.rs` offers it. Neither is anybody acting
    //   on the answer.
    if !shown.ends_with("data/config.rs") && !shown.ends_with("wx_settings.rs") { /* keep */ }
}

#[test]
fn test_every_setting_somebody_can_change_is_read_by_something() {
    // ... Ten settings had a real labelled control, saved, survived a
    // restart, and were read by nothing ...
    //
    // This lives here rather than in `tests/` because it has to read the
    // half of each file that ships, and `what_ships` is the one answer to
    // that question and is not compiled into a release. Cutting at the
    // first `#[cfg(test)]` instead is the exact mistake that module was
    // written to end: the main window has test modules sitting between
    // stretches of code, and two settings are read below the first one.
    let config = std::fs::read_to_string("src/data/config.rs").expect("the settings");
    let mut ignored = Vec::new();

    for name in stored_setting_names(&config) {
        let read_somewhere = files_that_act().into_iter().any(|path| {
            std::fs::read_to_string(&path)
                .is_ok_and(|text| crate::common::what_ships::what_ships(&text).contains(&name))
        });
        if !read_somewhere {
            ignored.push(name);
        }
    }

    assert!(
        ignored.is_empty(),
        "{} setting(s) can be changed and are read by nothing, so the \
         answer is taken and ignored:\n  {}",
        ignored.len(),
        ignored.join("\n  ")
    );
}
```

**The mirror to write** is the same loop with one file instead of a walk: read
`src/presentation/wx_settings.rs`, pass it through `what_ships`, and collect every
name from `stored_setting_names(&config)` that does not appear. It stays in
`config.rs` for the same stated reason: `what_ships` is not compiled into a release
and is not reachable from `tests/`.

**Two things the planner must get right.** `what_ships`
(`src/common/what_ships.rs`) is mandatory, per CONVENTIONS.md: three earlier
hand-rolled `#[cfg(test)]` scans in this codebase were wrong in the same direction.
And the new test needs its own red/green: turn it red by adding an `AppConfig`
field with no control, watch it fail, then add the control.

---

### `guards/guards.toml` (config) - a new record

**Analog: the last record in the file, lines 9068-9078.** The whole shape:

```toml
[[guard]]
name = "nothing but the seam and the uninstall sweep opens a credential entry of its own"
file = "src/service/credentials.rs"
before = """fn write_secret(account_id: &str, password: &str) -> Result<()> {
    secret_store::write(KEYRING_SERVICE, account_id, password)"""
after = """fn write_secret(account_id: &str, password: &str) -> Result<()> {
    let _ = keyring::Entry::new(KEYRING_SERVICE, account_id);
    secret_store::write(KEYRING_SERVICE, account_id, password)"""
red = [
    "service::secret_store::tests::test_only_the_seam_and_the_uninstall_open_a_credential_entry_of_their_own",
]
```

**What `scripts/guards.sh` requires** (the rules are stated at `guards/guards.toml:1-42`):

| Field | Requirement |
|---|---|
| `name` | A sentence saying what the guard defends, not the test's name |
| `file` | One path relative to the repo root |
| `before` | Must appear **exactly once** in that file. A record that stops matching fails the run, and the answer is to re-measure the guard by hand, never to edit the record until it applies |
| `after` | The exact edit that should break it |
| `red` | The **full** list of tests that go red, by module path. Exactly, in both directions: a named test that stays green fails the run, and a test that reddens and is not named fails it too |

**How to add one:** take the break by hand first and write down all of what really
went red. Do not write down the tests you expected. The file's own words:
*"A record shorter than the truth is the thing this file exists to stop."*

**Two bookkeeping obligations in the same edit.** The header carries two counts,
"192 records were swept" and "309 records have arrived since"; adding a record
raises the second number, and `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
in `tests/house_style.rs` checks the arithmetic on every commit. And
`test_every_guard_record_still_names_one_place_in_the_tree` checks the `before`
uniqueness at commit time.

---

### `src/application/mail_sync.rs` (service, batch) - `parent_id` (D-22) and folder-gone (D-27)

**Analog: `store_folders`, lines 412-439.**

```rust
pub fn store_folders(
    cache: &MessageCache,
    account_id: &str,
    folders: &[ImapFolder],
) -> Result<Vec<(ImapFolder, i64)>> {
    let mut stored = Vec::with_capacity(folders.len());
    for folder in folders {
        if crate::application::local_folders::is_local(&folder.path) {
            tracing::warn!(
                "The server listed a mailbox named like a folder kept on this computer, so it was left out of the folder list"
            );
            continue;
        }
        let id = cache.save_folder(&CachedFolder {
            id: 0,
            account_id: account_id.to_string(),
            name: folder.display_path.clone(),
            path: folder.path.clone(),
            folder_type: folder.folder_type.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
        // The two facts only the server can answer, kept so the window that
        // asks which folders to sync shows the same default the sync uses.
        cache.set_folder_server_facts(id, folder.holds_all_mail, folder.subscribed)?;
        stored.push((folder.clone(), id));
    }
    Ok(stored)
}
```

`parent_id` follows `holds_all_mail` exactly: a fact the server's LIST response
carries, computed here by splitting `path` on the per-mailbox delimiter, written
through a `set_folder_...` call after `save_folder` returns the id. `store_folders`
returns `(ImapFolder, i64)` pairs, so a second pass resolving each parent's path to
its stored id has the data it needs without another query.

The doc comment at 396-411 explains why the stored name is `display_path` and not
the leaf, and D-22 changes that reasoning: once nesting is stored, the tree shows a
leaf under a parent. Read that comment before changing the `name` field.

`is_local` guarding the loop is the pattern D-18's reserved account id must not
break: local and server folders share this table and are told apart by the path.

---

### `src/application/mail_controller.rs` (facade, request-response)

**Analog: `set_subscribed`, lines 525-530.** Three lines each, no logic:

```rust
/// Subscribe to a folder, or drop the subscription.
pub async fn set_subscribed(&self, path: &str, subscribed: bool) -> Result<()> {
    let mut guard = self.require_imap().await?;
    let session = &mut *guard;
    session.set_subscribed(path, subscribed).await
}
```

The permission gate is not repeated here; it is inside the session method.

---

### `src/application/local_folders.rs` (service, pure) - D-18's reserved account id

**Analog: itself.** `LOCAL_PREFIX` (line 54) is the trick D-18 copies, one level up:

```rust
pub const LOCAL_PREFIX: &str = "\u{1}Local";

/// Whether a path names a folder that lives on this computer.
pub fn is_local(path: &str) -> bool {
    path.starts_with(LOCAL_PREFIX)
}
```

`\u{1}` is chosen because a mailbox name does not carry it, which `mail_sync.rs:410`
restates. A reserved account id needs the same property and the same one-function
answer to "is this it".

`FOR_POP` / `FOR_IMAP` (lines 77-113) are `const [LocalFolder; N]` arrays and
`for_account(protocol)` (114-119) picks between them. D-18 keeps `FOR_IMAP` as
`[Outbox]` and moves the rest to a third constant owned by the reserved id.

**`deleting` (lines 172-186) is what D-33 routes emptying through**, and its doc
comment is the reason a second answer must not be written:

```rust
/// What Delete means for a message in this folder, or `None` if it is not ours.
///
/// `None` is the important answer: it means the message is on a server, and the
/// route that asks the server runs exactly as it did before. Only a folder on
/// this computer is decided here, which is why the account's protocol is not
/// what is asked.
pub fn deleting(
    from: &str,
    protocol: Protocol,
    asked: crate::application::destinations::Deleting,
    allowed: bool,
) -> Option<LocalDelete>
```

`LocalDelete` (163-170) has three variants including `Refuse(&'static str)`, which
carries the words to say. D-33's confirmation "says which of the two will happen,
because it knows" is answered by matching on this return value before the dialog.

---

### `src/presentation/wx_app.rs` - a destructive command (D-33 through D-38, D-26, D-27)

**Analog: `delete_the_chosen_search`, `wx_app.rs:6035-6090`.** The full shape of a
confirmed, announced, reversible-or-not command:

```rust
fn delete_the_chosen_search(
    app: AppHandles<'_>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) {
    use crate::presentation::accessibility::announcements::Priority;

    let AppHandles { state, tx, rt } = app;
    let chosen = the_chosen_saved_search(&lock_state(state));
    let (Some(chosen), Some(cache)) = (chosen, cache.as_ref()) else {
        return refuse_a_command(tx, WHICH_SAVED_SEARCH);
    };

    // Confirmed, because a Delete key is one row away from every other key
    // somebody might have meant. Enter answers No, so pressing it partway
    // through hearing the question does not remove anything.
    let asked = MessageDialog::builder(
        frame,
        &format!(
            "Delete the saved search {}? The mail it listed stays where it is.",
            chosen.name()
        ),
        "Delete Saved Search",
    )
    .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
    .build()
    .show_modal();
    if asked != ID_YES {
        return;
    }

    match cache.delete_saved_search(chosen.id()) {
        Ok(true) => {
            lock_state(state).selected_folder = None;
            read_the_tree_back(&Some(cache.clone()), state, tx);
            let said = format!("{} is gone. The mail it listed is untouched.", chosen.name());
            send_status(tx, rt, &said);
            let _ = a11y.announce(&said, Priority::High);
        }
        // ...
    }
}
```

Six things to copy: `refuse_a_command(tx, ...)` for a precondition that is not met
rather than a greyed-out item (D-38's rule, already the house pattern);
`crate::presentation::asking::yes_no_where_enter_answers_no()` on anything
destructive; the question naming the cost in one plain sentence (D-34 and D-37 both
specify the wording, already in that register in CONTEXT.md); `read_the_tree_back`
after the change; `send_status` and `a11y.announce(..., Priority::High)` together,
so the outcome exists on screen and by ear; and clearing `selected_folder` when what
was open has gone.

D-27's modal carries two extra constraints CONTEXT.md states and this analog does
not model: one at a time, and not while an editor has focus. Neither has an analog
in the tree and both need their own tests.

**Menu items and shortcuts (D-14, D-26, D-31)**, `wx_app.rs:5256-5268`:

```rust
let message = Menu::builder()
    .append_item(ID_REPLY, "&Reply\tCtrl+R", "Reply to sender")
    // The one that answers a person rather than a list. It had a
    // handler, a toolbar button and three lines in the shortcuts
    // document, and no menu item, which on Windows means no key: the
    // only way to reach it was the mouse.
    .append_item(
        ID_REPLY_SENDER,
        "Reply to Sender &Only\tAlt+Shift+R",
        "Reply only to the person who wrote it, never to the list",
    )
```

`Alt+Shift+R` at line 5265 is the only existing Alt+Shift binding, which is what
D-14 checked against. The comment states the rule: on Windows a shortcut without a
menu item is not a shortcut. `tests/wired.rs` enforces both directions
(`test_every_handled_command_has_something_that_raises_it` at line 78,
`test_every_command_something_raises_is_handled` at 189, and
`test_no_two_menu_items_claim_the_same_shortcut` at 335). Every new command id in
this phase must pass all three, and `docs/KEYBOARD_SHORTCUTS.md` is updated in the
same commit.

---

### `src/presentation/wx_compose.rs` (component, transform) - the `Re:` prepender

**Analog: itself, lines 161-175.** RESEARCH.md's amendment to D-04 says this
case-sensitive ASCII `starts_with` disagrees with
`mail_parser::parsers::fields::thread::thread_name`, and the two are brought into
line in the same change. Read those fifteen lines before writing the D-04 label
rule; do not write a second prefix list.

---

## Shared Patterns

### Error handling
**Source:** `src/common/error.rs`, applied at `src/service/protocols/imap.rs:1836-1844`
and throughout `src/data/message_cache/`.
**Apply to:** every new file.

Everything fallible returns `crate::common::Result<T>`. Foreign errors are mapped at
the boundary they enter, with a sentence naming what was being done:

```rust
.map_err(|e| Error::Other(format!("Failed to record the folder facts: {}", e)))?
```

No `unwrap` or `expect` outside `mod tests`, `main.rs` and `build.rs`. No
`thiserror`, no `anyhow`.

### Comments that name the defect
**Source:** every file read this session.
**Apply to:** every new file and every changed function.

This is the dominant style, not an exception. A comment says why the code is the
shape it is and names the failure that made it so, often with counts and dates.
`bodies.rs:487-494`, `folders.rs:8-22`, `message_rows.rs:31-39` and
`wx_app.rs:10578-10584` are four independent examples. A plan action that adds a
non-obvious rule without one is not following house style.

### Accessibility on both channels
**Source:** `CLAUDE.md` §Accessibility; guard at `tests/checkbox_labels.rs`.
**Apply to:** the five settings controls, every new tree row, every new menu item.

A check box gets `with_label(...)` **and** `set_accessible_name_and_description(...)`.
A `Choice` gets `set_accessible_name(...)`. Level and expanded state come from the
native `TreeCtrl`, never spelled into the label (D-15 and D-16 both restate this).

### Announcements are bounded
**Source:** `wx_app.rs:10593-10594` and `6084-6086`.
**Apply to:** every tree rebuild and every command outcome.

`announce_topic(&msg, Priority::Low, "folders")` for anything a timer can trigger,
so a syncing mailbox does not flood. `announce(&said, Priority::High)` for something
the user asked for. Both paired with `set_status_text` / `send_status`, because
guardrail 5 requires a visible equivalent.

### Additive schema only
**Source:** `CLAUDE.md` §Project rules; `mod.rs:3050-3074`.
**Apply to:** `parent_id`, D-40's original-uid column, any thread identity column.

`CREATE TABLE IF NOT EXISTS` for a table, `ensure_column_exists` for a column, never
a drop or a rename. `messages.thread_id` is not renamed even though nothing writes
it today.

### Guards for anything a rule depends on
**Source:** `guards/guards.toml`; `scripts/guards.sh`.
**Apply to:** the D-23 escape round trip, the D-41 encode/decode round trip, the
D-43 mirror, the D-02 display-and-sort agreement.

Take the break by hand, write down everything that really went red, add the record
and raise the second count in the file's header.

### Tests live beside the code
**Source:** CONVENTIONS.md; every module read.
**Apply to:** all eight new files.

`#[cfg(test)] mod tests` in the same file. Long sentence-shaped test names.
`#[tokio::test]` for async. `tempfile` for anything touching disk;
`common::temp_home::TempHome` where a `MessageCache` is built
(`folders.rs:295-300`). Cross-layer and source-reading checks that do not need
`what_ships` go in `tests/`, one process each.

---

## No Analog Found

| File / capability | Role | Data flow | Reason |
|---|---|---|---|
| `src/application/emptying.rs`, the partial-failure reporter (D-36) | service | batch | Nothing in the tree stops partway and reports exactly where it got to. Every existing batch operation either completes or returns one error. The sentence D-36 specifies ("Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the server refused. 118 messages were not removed.") needs a result type that carries a completed list, a stopping point and a reason. New code, new tests. |
| D-40's uid reassignment | migration | batch | Nothing has ever rewritten a `uid`. `UNIQUE(folder_id, uid)` at `mod.rs:1324` has only ever been satisfied, never worked around. `migrate_inline_bodies` gives the safe-move skeleton and nothing gives the collision handling. |
| D-27's one-at-a-time modal that waits for an editor to lose focus | controller | event-driven | Two prior fixes in this codebase name the failure it must avoid, but no existing dialog is queued or focus-gated. The `MessageDialog` analog above covers the dialog and not the gating. |
| D-08's cross-folder conversation query | model | CRUD | RESEARCH.md is explicit: `messages.thread_id` is written by nothing and read back by nothing, and threading today is an in-memory pass over one folder's loaded page in `wx_app.rs::apply_threading`. There is no account-wide conversation key to query. The producer (D-39) must land before anything reads it. `threading.rs` gives the algorithm and not the persistence. |

---

## Metadata

**Analog search scope:** `src/service/protocols/`, `src/application/`,
`src/data/message_cache/`, `src/data/config.rs`, `src/presentation/`, `tests/`,
`guards/`, `scripts/`.
**Files read this session:** 17.
**Pattern extraction date:** 2026-08-29.

**Two prohibitions restated for the planner.** Do not propose replacing the message
list control: D-01 locks it as a virtual `ListCtrl`, and `apply_columns`'s comment
explains why virtual mode makes a column rebuild cheap. Do not propose wxdragon's
tree item custom data for anything: it is a process-global registry that leaks one
entry per folder per sync, and the parallel-vector shape in `collect_rows`
(`wx_app.rs:10101-10118`) is the pattern.

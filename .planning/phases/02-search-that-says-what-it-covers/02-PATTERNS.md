# Phase 2: Search that says what it covers - Pattern Map

**Mapped:** 2026-08-31
**Files analyzed:** 14 (2 new, 12 modified)
**Analogs found:** 13 / 14

Every excerpt below was read from the file this session. Line numbers are as of
2026-08-31 and will move; the surrounding function name is the durable handle.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/presentation/wx_managers.rs` (rule editor: `build_rule_edit_dialog`, `show_rule_manager_dialog`, plus the vocabulary fix in `build_filter_edit_dialog`) | component (dialog) | request-response, CRUD over a row set | `run_manager_loop` (:203) + `show_filter_manager_dialog` (:2423) + `build_filter_edit_dialog` (:2544) in the same file | exact, in-file |
| `tests/rule_editor_labels.rs` (new) | test | request-response | `tests/checkbox_labels.rs` | role-match |
| `src/application/allowed.rs` (read dimension) | model / config type | transform | its own `mail` and `personal_information` fields, end to end | exact |
| `src/data/config.rs` (serde default, mirror-guard exception) | config | file-I/O | `AppConfig::check_spelling_before_send` + `default_true` (:248), `allowed_changes` + `default_allowed` (:256, :436) | exact |
| `src/presentation/wx_settings.rs` (a control for the read dimension) | component | request-response | the `allow_mail` / `allow_pim` read-back at :2089 | exact |
| `src/presentation/first_run.rs` / `command_line.rs` | config | transform | `Choice::allowed` (:59) and `"--read-only"` (:164) | exact, no code change wanted, only a test |
| `src/service/protocols/imap.rs` (body fetch behind a read gate) | service | request-response | `may_i` (:1180) and its eleven call sites | partial: the gate is a new function, `may_i` is writes-only |
| `src/service/protocols/pop3.rs` | service | request-response | `may_i` (:284), call at :361 | same shape, second file the guard must see |
| `src/application/saved_searches.rs` (`what_a_typed_search_asks`, `a_typed_search_in_words`, folder half) | service (pure application logic) | transform | `Question::as_a_rule` (:116), `WHAT_A_TYPED_SEARCH_LOOKS_AT` (:340) | exact |
| `src/data/message_cache/saved_searches.rs` (coverage count) | model / persistence | CRUD + batch read | `create_saved_search` (:69), `get_saved_searches_for_account` (:184), `scan_query` (:341) | exact |
| `src/presentation/wx_app.rs` (`folder: None` at :6545, `every_saved_search` :9709, per-account read :9529) | controller | request-response | the same functions | exact |
| `src/presentation/folder_tree.rs` (account sub-branches) | component | transform | `favourite_rows` (:731) | exact |
| `src/data/message_cache/bodies.rs` (comment on `evict_bodies_over`) | persistence | batch | `drop_synced_task` doc at `src/data/message_cache/tasks.rs:412-432` | role-match, comment shape only |
| `guards/guards.toml` (3 new records) | config | file-I/O | any `[[guard]]` entry, e.g. "the Microsoft merge asks what the whole contact is owed" (:167) | exact |

---

## Pattern Assignments

### 1. `src/presentation/wx_managers.rs` rule editor (component, CRUD over a row set)

**Analog:** `run_manager_loop` at `src/presentation/wx_managers.rs:203-211`, and its only
non-contact caller, `show_filter_manager_dialog` at `:2423`.

**The signature the new editor extends** (`wx_managers.rs:203-211`). It is already
generic over the row type, so a `Question` row needs no change to it:

```rust
fn run_manager_loop<T: Clone + 'static>(
    chrome: ManagerChrome<'_>,
    kind: &str,
    working: &mut Vec<T>,
    populate: impl Fn(&ListCtrl, &[T]) + Copy + 'static,
    add_fn: impl Fn(&Dialog) -> Option<T>,
    edit_fn: impl Fn(&Dialog, &T) -> Option<T>,
    name_fn: impl Fn(&T) -> String + Copy + 'static,
) -> bool {
```

**How a caller supplies the row type** (`wx_managers.rs:2423-2461`). Copy this
whole shape: `make_shell`, then `insert_column` per column, then one
`run_manager_loop` call whose `add_fn` and `edit_fn` are the same sub-dialog with
and without an existing row:

```rust
    let palette = theme::current_from_stored_config();
    let (dialog, sizer, list, status) = make_shell(parent, "Filter Manager", 650, 450, palette);

    list.insert_column(0, "Name", ListColumnFormat::Left, 130);
    list.insert_column(1, "Condition", ListColumnFormat::Left, 220);
    list.insert_column(2, "Action", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 70);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let mut working = rules.to_vec();
    let changed = run_manager_loop(
        ManagerChrome { dialog: &dialog, main_sizer: &sizer, list: &list,
                        status_text: &status, a11y: a11y.clone() },
        manager_words::FILTER,
        &mut working,
        populate_filters,
        |d| show_filter_edit(d, None, palette),
        |d, r| show_filter_edit(d, Some(r), palette),
        |r| r.name.clone(),
    );
```

**How the list announces its set size.** It does not, and that is the point.
`make_shell` at `:357-381` builds a native `ListCtrl` in report mode and sets one
accessible name on the control itself:

```rust
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, "Items");
```

The item count and position in set come from Windows' own provider for the native
control, on UI Automation and MSAA both. Nothing in this code computes them. That
is the whole argument in the research for not replacing this shape: a hand-rolled
`ScrolledWindow` of rows would have to set the count with `set_accessible_name`,
which reaches MSAA only, so NVDA would hear a set size and Narrator would not.
`"Items"` is generic; the rule editor should pass its own noun, and if that means
a parameter on `make_shell`, that is a smaller change than a second shell.

**How an edit dialog is opened and returns.** `run_manager_loop` never calls the
sub-dialog inline. Add and Edit end the outer modal with a sentinel id, the loop
re-enters, and `add_fn`/`edit_fn` open the child (`wx_managers.rs:333-346`):

```rust
            r if r == ID_MGR_EDIT => {
                if let Some(idx) = get_selected(list) {
                    let current = state.borrow().working[idx].clone();
                    if let Some(edited) = edit_fn(dialog, &current) {
                        let name = name_fn(&edited);
                        let mut s = state.borrow_mut();
                        s.working[idx] = edited;
                        s.changed = true;
                        drop(s);
                        populate(list, &state.borrow().working);
                        said_and_shown(status_text, &a11y,
                                       &manager_words::updated(kind, &name), Priority::Normal);
                    }
                } else {
                    said_and_shown(status_text, &a11y,
                                   &manager_words::nothing_selected(kind, "edit"), Priority::High);
                }
            }
```

`populate` is a free function taking the whole slice and rebuilding the list
(`populate_filters`, `:2502-2520`, starts with `list.delete_all_items()`). Write
`populate_questions` the same way.

**Announcement pattern.** Every change says a sentence through `said_and_shown`,
which writes the status `StaticText` and announces on one call. `delete_selected`
(`:152-187`) is the one to copy for the count-in-the-same-sentence requirement:
it already computes `let left = state.borrow().working.len();` and restores the
row cursor before speaking. The count is currently used only for the cursor, not
spoken; the phase's "Condition removed. 2 conditions." wording plugs into
`manager_words::deleted` rather than into a second announcement.

**Depth constraint.** `show_filter_manager_dialog(parent: &Frame, ...)` opens a
`Dialog` which opens `build_filter_edit_dialog(parent: &Dialog, ...)`. Two deep is
the deepest this codebase goes. Reach the rule editor from the folder tree context
menu (`wx_context_menu.rs`) so it stays `Frame -> editor -> condition dialog`.

---

### 2. The second vocabulary, to fix rather than copy

**Where it is.** `build_filter_edit_dialog` builds its own field list and its own
match list, both shorter than the engine's. Verbatim
`src/presentation/wx_managers.rs:2569-2572`:

```rust
    let field_choices: Vec<String> = ["subject", "from", "to", "cc", "body_plain", "date"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let field_choice = Choice::builder(&dlg).with_choices(field_choices).build();
    set_accessible_name(&field_choice, "Match field");
```

and `:2583-2596`:

```rust
    let match_choices: Vec<String> = [
        "contains", "not_contains", "equals", "starts_with", "ends_with", "regex",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let match_choice = Choice::builder(&dlg).with_choices(match_choices).build();
    set_accessible_name(&match_choice, "Match type");
```

**Where the real vocabulary is declared.** `src/application/filters.rs:61-73`
(`A_FIELD_A_RULE_MAY_NAME`, eleven entries) and `:86-98` (`A_WAY_A_RULE_MAY_MATCH`,
eleven entries). Both are the house pattern `CONVENTIONS.md` names: a constant
standing in for a documented list, with a test that the constant and the reading
logic agree in both directions (`filters.rs:84-85`).

**The pattern to copy for the fix.** The third `Choice` in the same dialog already
does it right, from a constant, `wx_managers.rs:2613-2618`:

```rust
    let action_choices: Vec<String> = RULE_ACTIONS
        .iter()
        .map(|(_, shown)| (*shown).to_string())
        .collect();
    let action_choice = Choice::builder(&dlg).with_choices(action_choices).build();
    set_accessible_name(&action_choice, "Action");
```

`RULE_ACTIONS` (`:2472-2483`) is a `&[(&str, &str)]` of stored name and spoken
words, with `shown_action` / `stored_action` (`:2486-2500`) converting. The eleven
fields and eleven match types need the same stored-to-spoken pair, because
`body_plain` and `not_contains` are machine names and the memory note
"no machine names for people" applies. Add the words beside the constants in
`filters.rs`, not in the dialog, or the second vocabulary comes back as a third.

**Do not copy.** The hardcoded arrays. Grep guard for the plan: a literal
`["subject", "from", ...]` anywhere under `src/presentation/`.

**Checkbox naming, which is right here and must stay right.** The two checkboxes
carry their label on the control and set no accessible name
(`wx_managers.rs:2604-2609`, `:2628-2632`):

```rust
    let cs_check = CheckBox::builder(&dlg).with_label("&Case Sensitive").build();
```

That is correct: a labelled checkbox is named on both channels. The bug
`tests/checkbox_labels.rs` exists to catch is the opposite, an empty label plus
`set_accessible_name`.

**Refusal pattern for the empty condition list** (`:2669-2679`), which is the shape
`selects`'s empty-list trap needs at the editor:

```rust
    ok.on_click({
        let d = dlg;
        move |event| {
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(&d, "A name is needed before this can be saved.");
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
        }
    });
```

`event.event.skip(false)` is what makes the refusal stick.

**Guard coverage note, stated because the plan must not assume it.**
`tests/checkbox_labels.rs` iterates `ItemKind` through `build_item_form_dialog`
only (`:56-94`). It never opens `wx_managers.rs`. The "Case Sensitive" and
"Enabled" checkboxes above are unguarded today and any checkbox the rule editor
adds is unguarded too. `tests/checkbox_labels.rs:21-23` also records that
wxWidgets allows one application per process, so the new work needs its own file
under `tests/` with a single `#[test]`, not an extra case in that one.

---

### 3. A settings field that reaches the provider clients

**Declaration, `src/application/allowed.rs:31-48`.** Both existing fields and the
doc that D-2-06 makes false:

```rust
/// What may be changed at a provider.
///
/// Two answers rather than one boolean, because the two cost different amounts
/// to get wrong. `Default` is the safe end of both, so anything constructed
/// without a decision changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Allowed {
    /// Sending a message, and changing or deleting one on the server.
    ///
    /// The one that cannot be undone. A message that has gone has gone, and a
    /// message deleted from a server may be the only copy.
    pub mail: bool,
    /// Tasks, contacts and calendar events at a provider.
    pub personal_information: bool,
}
```

Note `Default` is derived (`:36`) and there is not one field-level serde attribute
in the struct. Both facts are the phase's two traps.

**The three places that must agree**, named by the module's own doc at
`allowed.rs:11-17`, and modelled as `Permission` at `:180-188`: `command_line`,
`setting`, `account`. The narrowing rule is `and`, `:82-87`:

```rust
    pub const fn and(self, other: Self) -> Self {
        Self {
            mail: self.mail && other.mail,
            personal_information: self.personal_information && other.personal_information,
        }
    }
```

A third field joins with `&&` like the other two. With `NOTHING` holding
`reading: true`, `and` can only turn reading off when some place says so.

`anything()` at `:89-93` is writes-only and must stay so, or
`assert!(!Allowed::default().anything())` at `:201` breaks:

```rust
    /// Whether anything at all may be changed.
    pub const fn anything(self) -> bool {
        self.mail || self.personal_information
    }
```

**Where the answer is resolved**, `allowed.rs:429-435`, and the reason a parse
failure is catastrophic rather than local:

```rust
pub fn allowed_for(account_id: &str) -> Allowed {
    let stored = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().allowed_for(account_id))
        .unwrap_or(Allowed::NOTHING);

    narrowed_by(FROM_COMMAND_LINE.get().copied(), stored)
}
```

**The `may_i` sites, both files.** The definition in
`src/service/protocols/pop3.rs:280-286` shows the whole body and states the
one-answer rule:

```rust
    /// `doing` is the act in words somebody would want to hear. One answer for
    /// every transport, in [`crate::service::outward::permitted`], so a POP
    /// account and a mail account cannot come to disagree about what the
    /// setting means.
    fn may_i(&self, doing: &str) -> Result<()> {
        crate::service::outward::permitted(self.may_change, doing)
    }
```

Call sites: `pop3.rs:361`, and eleven in `src/service/protocols/imap.rs` at
`:860, 894, 931, 963, 1195, 1219, 1247, 1306, 1319, 1351, 1381` with the definition
at `imap.rs:1180`. All are writes. CONTEXT.md D-2-06 names `imap.rs` only;
`pop3.rs` has one too, and any structural guard written about `may_i` that reads
one file measures half of what it claims.

`permitted` (`src/service/outward.rs:292-297`) takes a plain `bool` and returns
`Error::Security`:

```rust
pub fn permitted(may_change: bool, doing: &str) -> Result<()> {
    if may_change {
        return Ok(());
    }
    Err(Error::Security(refusal(doing)))
}
```

Its refusal sentences are written for changes. The read gate is a **new sibling
function beside `permitted`**, worded for a read, not a widening of `may_i`.
`src/service/outward.rs:1693` holds
`const SENDS_A_MAIL_CHANGE: [&str; 2] = ["self.may_i(", "outward::permitted("];`
which is a census asserting a floor, so adding a function near it re-measures
every record that reads it, in the same commit.

**How a stored `app_config.json` is deserialised.** `src/data/config.rs:741-748`,
one `from_str` for the whole struct, so one missing field takes every setting down:

```rust
            self.app_config = serde_json::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse app config: {}", e)))?;
            self.app_config.validate()?;
```

**The serde behaviour, which is the trap.** `Allowed` itself has no field
attributes, so no field of it handles an older file. The pattern to copy is one
level up, on `AppConfig`, `config.rs:243-257`:

```rust
    /// Whether to check the spelling of a message before sending it.
    #[serde(default = "default_true")]
    pub check_spelling_before_send: bool,
    /// What Wixen Mail may change at a server, for every account.
    ///
    /// Defaults to `default_allowed` below, which names the answer instead of
    /// saying it again.
    #[serde(default = "default_allowed")]
    pub allowed_changes: crate::application::allowed::Allowed,
```

with `fn default_true() -> bool { true }` at `config.rs:454` and `default_allowed`
at `:436-438`, whose doc (`:425-435`) is the house rule for why the function is
named rather than the value restated. The new field inside `Allowed` needs its own
`#[serde(default = "...")]` with a named function returning `true`.
`#[serde(default)]` alone gives `false`, which is D-2-07 inverted.
`allowed_per_account: HashMap<String, Allowed>` (`config.rs:275-276`) holds the
same struct as map values, so the field attribute covers both.

**The RED test template**, `config.rs:994-1022`. The `.remove(...).expect(...)` is
the load-bearing part: a round-trip of a freshly serialised struct passes either
way:

```rust
        let mut older: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&AppConfig::default()).unwrap()).unwrap();
        older
            .as_object_mut()
            .expect("the config is an object")
            .remove("start_in_all_inboxes")
            .expect("the key is written, so removing it is a real before-and-after");
        let older: AppConfig = serde_json::from_value(older)
            .expect("a settings file written before this existed has to still load");
```

**The mirror guard, and what it cannot see.**
`test_every_setting_somebody_can_change_is_offered_by_a_screen`
(`config.rs:1713-1748`) reads field names out of `pub struct AppConfig` and checks
each appears in the shipping half of the settings screen. It never opens
`Allowed`, so a new field inside `Allowed` passes it whether or not a screen offers
one. State that in the plan and verify the control by hand or with a new check.
Its exception list `STORED_AND_OFFERED_BY_NOTHING = ["allowed_per_account"]`
(`config.rs:1675`) is itself checked by
`test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing`, so a
per-account read control deletes that entry in the same commit or the test fails.

**The settings-screen read-back to copy**, `src/presentation/wx_settings.rs:2089-2092`:

```rust
    cfg.allowed_changes = crate::application::allowed::Allowed {
        mail: w.allow_mail.get_value(),
        personal_information: w.allow_pim.get_value(),
    };
```

The compiler catches this one. `SETTINGS_SECTION` is `"Allow Changes"`
(`allowed.rs:99`) and a read is not a change, so the read control gets a second
named constant with its own section. `changes_waiting_here` and
`removals_waiting_here` assert `SETTINGS_SECTION` verbatim at `allowed.rs:274` and
`:296`; those strings do not move.

---

### 4. A saved search written and read back

**Analog:** `src/data/message_cache/saved_searches.rs`, one function for the write
and one for the read.

**Questions written per position**, `create_saved_search` at `:69-121`. Both tables
in one transaction, the row first with `folder` in it, then one row per question
with its index as `position`:

```rust
        saving
            .execute(
                "INSERT INTO saved_searches
                 (id, account_id, name, all_or_any, folder, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![&search.id, account_id, &search.name,
                        search.join.written_down(), &search.folder, &now],
            )
            ...
        for (position, question) in search.questions.iter().enumerate() {
            saving
                .execute(
                    "INSERT INTO saved_search_questions
                     (search_id, position, field, match_type, pattern, case_sensitive)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![&search.id, position as i64, &question.field,
                            &question.match_type, &question.pattern, &question.case_sensitive],
                )
```

Its doc says why they are one transaction: "Half a question list is a different
question under the same name, which is the one thing worse than the search not
being there at all." D-2-14 writes the folder and the narrower question set in the
same call, which is that rule applied to the two halves of a scope.

**Read back**, `get_saved_searches_for_account` at `:184-211`, account-scoped, plus
`put_back_together` at `:434-458` which is the reader's answer to a word it does not
know:

```rust
                "SELECT id, name, all_or_any, folder
                 FROM saved_searches
                 WHERE account_id = ?1
                 ORDER BY created_at, id",
```

```rust
        match Join::read(&row.all_or_any) {
            Some(join) => read.searches.push(SavedSearch {
                questions: questions.remove(&row.id).unwrap_or_default(),
                id: row.id, name: row.name, join, folder: row.folder,
            }),
            None => read.saved_by_another_version.push(SearchSavedByAnotherVersion {
                id: row.id, name: row.name,
            }),
        }
```

That is the lenient-reader-strict-writer discipline done right: an unreadable row
becomes a named second list rather than a guess.

**Where `folder` is written today**, `src/presentation/wx_app.rs:6539-6552`. This is
the one production writer of `SavedSearch`, and the hardcoded `None` with its
stated reason is what D-2-14 changes:

```rust
    let search = SavedSearch {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        join: WHAT_A_TYPED_SEARCH_JOINS_WITH,
        questions: what_a_typed_search_asks(&text),
        // Everywhere in this account, whatever the search box's "In" list was
        // set to when the search ran. ...
        folder: None,
    };
```

The comment goes with the change rather than being left contradicting the code.

**The vocabulary rule this must not break**, `src/application/saved_searches.rs:109-127`:

```rust
    /// This question as the rule the filter engine already knows how to answer.
    ///
    /// A saved search selects rather than acts, so the action is never read.
    /// It is filled in with the one action that changes nothing, because the
    /// rule type demands one and inventing a second matcher to avoid it would
    /// be the second vocabulary this module exists to not have.
    fn as_a_rule(&self) -> FilterRule {
```

`SavedSearch.folder`'s own doc (`:539-544`) states the split the plan must keep:
the folder narrows the query that gathers messages, `selects` sees one message at
a time and never answers it.

`SavedSearch::reads_the_message_text` (`:575-579`) is the single source for whether
a search needs bodies. Do not write an `if field == "body_plain"` at any call site.

---

### 5. An FTS index write, and a deliberate non-action

**`index_message_for_search`**, `src/data/message_cache/searching.rs:271-323`. It
rewrites every column because a contentless index cannot update one:

```rust
    /// Put a message into the search index, replacing whatever was there.
    ///
    /// Done from here rather than by a trigger because the body is stored
    /// packed, and a trigger written in SQL has no way to unpack it. The
    /// delete side is a trigger, since removing a row needs none of its text.
    pub(super) fn index_message_for_search(&self, message_id: i64) -> Result<()> {
```

with the body read back rather than passed in (`:298-309`), and a comment already
recording the eviction behaviour from the index's side:

```rust
        // The body if one is cached, and nothing if not. A message whose text
        // has been evicted stays searchable by its subject and sender rather
        // than dropping out of the index altogether.
```

**Where it is called from.** Three sites, all writes of message text:
`src/data/message_cache/bodies.rs:314` (after a body is stored),
`src/data/message_cache/messages.rs:925` and `:1575`. Nothing else.

**What `evict_bodies_over` does and does not do**,
`src/data/message_cache/bodies.rs:412-448`. It deletes the row and stops:

```rust
        for (message_id, bytes) in candidates {
            if total <= budget_bytes {
                break;
            }
            self.conn
                .execute(
                    "DELETE FROM message_bodies WHERE message_id = ?1",
                    rusqlite::params![message_id],
                )
                .map_err(|e| Error::Other(format!("Failed to evict message body: {}", e)))?;
            total -= bytes;
            freed += bytes;
        }
```

No `index_message_for_search`. Its existing doc (`:396-411`) already carries the
"this is deliberate and here is the cost" shape for a different property, that the
budget cannot always be met.

**The comment shape D-2-13 needs.** Two existing comments in this codebase record a
deliberate non-action and are the models. The closer is `drop_synced_task`,
`src/data/message_cache/tasks.rs:412-432`:

```rust
    /// Removes the row whatever state it is in, and that is deliberate rather
    /// than an oversight. The calendar has the opposite rule next door:
    /// `drop_synced_calendar_event` refuses in SQL a row with a change waiting
    /// on it, so a pass that forgets to ask is safe anyway. That cannot be
    /// copied here, because two of the three callers hand this a row that is
    /// waiting on purpose.
```

The second, for a function that writes one thing and touches nothing else, is
`set_what_the_server_said` at `src/data/message_cache/folders.rs:202-208`:

```rust
    /// D-27. This writes one value and nothing else. It deliberately touches no
    /// message: the mail in the folder stays cached and readable until somebody
    /// has been asked and has answered ...
```

Both name the alternative, say why it was not taken, and name what would go wrong
if somebody "fixed" it. `evict_bodies_over`'s comment should say: the index keeps
the words, the saved-search scan loses them, the two searches now cover different
amounts, the coverage sentence names which search it is about, and reindexing here
would make a message unfindable at the moment its body is evicted.

**Do not** try to read `body` back out of `message_search`. It is declared
`content=''` with `contentless_delete=1` (`src/data/message_cache/mod.rs:2170-2175`),
so it stores no retrievable content. The coverage count comes from
`message_bodies`, joined per account, following `scan_query`'s `WHERE` clause
(`src/data/message_cache/saved_searches.rs:341-365`) including `m.deleted = 0`.

---

### 6. A source-reading guard, and a guard record

**Analog:** the mirror guard's helpers in `src/data/config.rs`, and any
`[[guard]]` entry in `guards/guards.toml`.

**Reading only the half of a file that ships**, `config.rs:1678-1682`. Any check
that asks "does this code really ship" uses `common::what_ships`, never a
hand-rolled `#[cfg(test)]` scan; three hand-rolled versions were wrong the same
way:

```rust
    /// The shipping half of one file, or an empty string if it cannot be read.
    fn what_ships_in(path: &str) -> String {
        std::fs::read_to_string(path)
            .map(|text| crate::common::what_ships::what_ships(&text))
            .unwrap_or_default()
    }
```

Its two constants are `THE_SETTINGS_SCREEN = "src/presentation/wx_settings.rs"`
(`:1625`) and `EVERY_SCREEN = "src/presentation"` (`:1629`), with
`what_every_screen_ships()` (`:1685-1702`) walking the tree. A guard that the rule
editor offers exactly `A_FIELD_A_RULE_MAY_NAME` and `A_WAY_A_RULE_MAY_MATCH` is
this shape: read the shipping half of `wx_managers.rs`, assert every constant
member appears and that no field name appears that is not a member. Both
directions, as `filters.rs:84-85` already does for the engine, or a twelfth field
slips past.

Such a test lives in `src/` and not `tests/` for the reason `config.rs:1715-1719`
gives about itself: `what_ships` is not compiled into a release build.

**A guard record**, `guards/guards.toml:167-176`, five fields:

```toml
[[guard]]
name = "the Microsoft merge asks what the whole contact is owed"
file = "src/application/contacts_sync.rs"
before = """                        the_copy_here_holds_work_nobody_has_sent(local),
                        arrived_at,"""
after = """                        this_address_book_is_still_owed_the_change(local, &AddressBook::Microsoft),
                        arrived_at,"""
red = [
    "application::contacts_sync::tests::test_a_change_only_google_still_needs_is_not_written_over_quietly_by_outlook",
    "application::contacts_sync::tests::test_a_change_only_google_still_needs_survives_an_outlook_read_that_moved_nothing",
]
```

What `scripts/guards.sh` requires of it: `before` appears **exactly once** in the
named file, the edit `before -> after` is applied, the whole library runs, and the
tests that went red must be **exactly** the `red` list, in both directions. A
named test that stayed green fails the run; a test that reddened and is not named
fails it too. `red` entries are full module paths.

Two more rules from the file's own header and from `CLAUDE.md`, both of which bite
inside this phase:

- The sweep counts at `guards/guards.toml:44-58` are checked by
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  in `tests/house_style.rs`. Adding a record raises the second number in the same edit.
- Any change that adds tests near a rule re-measures that rule's record in the same
  commit, and `scripts/guards.sh` runs unfiltered before the phase closes. This
  phase adds tests near `SENDS_A_MAIL_CHANGE` (`src/service/outward.rs:1693`) and near
  `src/application/sent_copy.rs:1112`, both of which read a census of `may_i` sites.

Records this phase owes, each taken red by hand first: the read gate on the body
fetch, `NOTHING` keeping reading on, and the offered lists equalling the constants.

---

### 7. Account sub-branches in the tree (D-2-05)

**Analog:** `favourite_rows` at `src/presentation/folder_tree.rs:731-800`, and the
flat saved-search rows it replaces at `:590-606`.

The heading, a branch per account that has something, then the rows, with an early
return that leaves the group out when it is empty:

```rust
    if branches.is_empty() {
        return Vec::new();
    }

    let word = |mut row: TreeRow| -> TreeRow {
        let closed = collapsed.contains(&row.identity.stored());
        if let Some(said) = row.worded(closed, setting) {
            row.label = said;
        }
        row
    };

    let mut out = vec![word(TreeRow {
        identity: WhichRow::Favourites,
        name: FAVOURITES.to_string(),
        label: String::new(),
        depth: 0,
        expandable: true,
        ...
```

Two things the analog has that saved searches do not:
`crate::application::favourites::Pin` carries `account` (`src/application/favourites.rs:73-82`)
and `in_account_order` groups by it. `SearchInTheTree`
(`folder_tree.rs:306-309`) carries only `id` and `name`, and `SavedSearch` carries
no account either. Add the account to `SearchInTheTree` and populate it at the
boundary, in `every_saved_search` (`src/presentation/wx_app.rs:9709-9727`), the way
Favourites does. Do not widen `SavedSearch`.

A new `WhichRow::SavedSearchesIn(account)` needs its own stored spelling that
collides with neither `"saved-searches"` nor `format!("saved-search{APART}{id}")`
(`folder_tree.rs:155-156`), per D-25's keying rule.

The cost to state in the plan: the read at `wx_app.rs:9529-9534` moves into the
per-account loop, which makes it six things read per account across eleven redraws,
some on a timer. `STATE.md` records the identical cost being paid in `01-14`.

---

## Shared Patterns

### Sentences belong in `application`, not in the window layer
**Source:** `src/application/saved_searches.rs` (`a_typed_search_in_words`,
`what_a_search_found`, `a_row_for`, `announced`)
**Apply to:** D-2-04's sentence builder and D-2-08's coverage sentence.
Every sentence about a saved search is built in this module and the presentation
layer only shows it. A second sentence builder in `wx_app.rs` is the drift
`EXPERIMENTAL_WARNING`'s comment records.

### A default is named, never restated
**Source:** `src/data/config.rs:425-438` (`default_allowed`), `:454` (`default_true`)
**Apply to:** the new `Allowed` field's serde default, and any new constant.
"A sentence a check reads may state the answer, because it will be held to it. A
sentence no check reads should name the answer instead."

### A reader that does not understand a stored value says so rather than guessing
**Source:** `Join::read` (`src/application/saved_searches.rs:136-152`),
`put_back_together` (`src/data/message_cache/saved_searches.rs:441-458`),
`shown_action` (`src/presentation/wx_managers.rs:2486-2492`)
**Apply to:** any new stored value the rule editor writes. Guessing narrows or
floods somebody's results in silence; the second list, or the stored name shown
as-is, is the house answer.

### Every dialog splits build from show
**Source:** `build_filter_edit_dialog`'s doc, `src/presentation/wx_managers.rs:2531-2537`
**Apply to:** every dialog this phase adds.
"A test can build the real dialog and read back the real colour a live control
holds, and never call `.show_modal()` at all."

### Announce and show in one call
**Source:** `said_and_shown(status_text, a11y, &manager_words::deleted(kind, &name), Priority::Normal)`
in `delete_selected` (`src/presentation/wx_managers.rs:174-179`)
**Apply to:** every add, edit and remove in the rule editor. One sentence carrying
the change and the new count, not two announcements.

### Experimental things say so in the product
**Source:** `application::allowed`'s `EXPERIMENTAL_WARNING`, and
`src/presentation/first_run.rs:74-88`
**Apply to:** the body fetch (D-2-08). The sentence goes beside the control, not in
a changelog and not in a report.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| The body fetch in `src/service/protocols/imap.rs` | service | request-response | Every existing gated path in `imap.rs` is a write, and `permitted`'s refusal wording is written for changes. The read gate is a new function beside `permitted` with new wording, so only the call-site shape (`self.may_i("...")?` as the first line of the method) transfers. Nothing in this repository has ever fetched bodies in bulk, and no test here can reach whether a provider allows it. |

---

## Three cautions carried forward

1. **No second matcher and no second vocabulary anywhere.** `Question::as_a_rule`'s
   comment (`src/application/saved_searches.rs:112-115`) says inventing one is what
   that module exists to prevent, and D-2-01 depends on it staying true. The
   hardcoded lists in `build_filter_edit_dialog` are a live breach of it in the
   presentation layer and are fixed, not copied.

2. **`run_manager_loop` is extended, not replaced.** It is the only shape where the
   set size reaches both accessibility channels from the system's own provider, the
   tab order never changes as rows come and go, and there is no drag interaction.

3. **`tests/checkbox_labels.rs` covers `wx_item_form` only.** The rule it states is
   unguarded in `wx_managers.rs`, which is where this phase works. The plan decides
   deliberately between widening that file, which changes what a recorded guard
   covers, and a companion file for the manager dialogs.

## Metadata

**Analog search scope:** `src/presentation/`, `src/application/`, `src/data/message_cache/`,
`src/service/protocols/`, `src/service/outward.rs`, `tests/`, `guards/`, `scripts/`
**Files read this session:** 16
**Pattern extraction date:** 2026-08-31

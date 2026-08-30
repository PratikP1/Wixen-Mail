# Phase 1: Folders and conversations - Research

**Researched:** 2026-08-29
**Domain:** IMAP mailbox management, SQLite conversation aggregation, wxWidgets virtual list and tree controls, Windows accessibility
**Confidence:** HIGH for everything read out of the tree and out of the vendored crate sources; MEDIUM for the two questions answered from the web.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

Copied from `01-CONTEXT.md` `## Implementation Decisions`. These are settled. Research below investigates how to implement them, never whether to.

**Conversations in the message list**

- **D-01:** A conversation row never expands in place. `Enter` opens the existing `wx_thread_view` `TreeCtrl`, which announces level natively. The message list stays a virtual `ListCtrl`, so UI Automation keeps the real set size, and `wx_thread_view`'s own written rationale for keeping the list flat stands.
- **D-02:** A row describes the **whole conversation**, not its newest message. One rule per column family, stated once and used by both the displayed value and the SQL `ORDER BY`, because `MessageColumn` supplies both from the same enum today and they must not come apart:
  - Dates (`Received`, `Sent`) and `Snippet`: the newest message's.
  - State (`Attachment`, `Flagged`, `Answered`, `Draft`): true if any message is.
  - `Safety`: the worst across the conversation.
  - `Size`: the sum.
  - `Correspondent`: the distinct senders.
  - `Subject`: the conversation's, per D-04.
  - `Thread`: the counts, per D-03.
  Reversibility: costly.
- **D-03:** `MessageColumn::Thread`, which today holds a thread id no person wants read aloud, becomes "5 messages, 2 unread".
- **D-04:** The conversation is named by the **oldest message present**, with `Re:` and `Fwd:` prefix chains stripped, including the localised prefixes other clients emit. The label can change when `Get Older Messages` brings in an earlier message.
- **D-05:** A lone message's row shows no counts. The `Thread` column appears only when the folder holds at least one conversation of more than one message.
- **D-06:** A hand choice in `View > Columns` beats the adaptive rule in D-05, permanently. One stored value per folder must distinguish "never chosen" from "chosen off".
- **D-07:** An action on a collapsed conversation row acts on the **whole conversation**, with the count named before it happens. A reversal puts the whole conversation back.
- **D-08:** A conversation spans the **account**, not the folder. Its row appears in every folder it touches, showing the same count wherever the user is standing. Folders whose `holds_all_mail` server fact is true are excluded from the count. Reversibility: costly.
- **D-09:** Thread view is stored **per folder**, alongside whatever FOLDER-02 stores for collapsed state. A folder never set is flat.
- **D-10:** An `Apply View To Other Folders` command offers three scopes: this folder's subtree, this account, or everywhere. Each names what it will change and how many folders that is, spoken before it happens, and it confirms.
- **D-11:** A selection survives a view switch **both ways**. The original set is held across the switch.
- **D-12:** The sort column and direction survive a view switch unchanged, applied through D-02's rule.

**The folder tree**

- **D-13:** **One branch per account.** `ALL_INBOXES` keeps its place at the root. Folders inside a branch keep their existing `tree_position` order. Reversibility: costly.
- **D-14:** Accounts appear in the order they were added and can be moved with **Alt+Shift+Up/Down**, position announced as it moves. One stored ordinal per account; reordering never touches a server. Bare Alt+Shift is the Windows input-language hotkey. `docs/KEYBOARD_SHORTCUTS.md` gets the entry in the same commit.
- **D-15:** An account branch announces name, unread and folder count. Expanded or collapsed, and the level, come from the tree control and are not in the text.
- **D-16:** `Enter` on an account branch or on the "On this computer" group expands or collapses it and announces the new state. Nothing else.
- **D-17:** Local folders live in a group node, **"On this computer"**, placed after the account branches and before `Labels`. Omitted entirely when empty.
- **D-18:** **Only `Inbox` is per account.** `Sent`, `Outbox`, `Drafts`, `Junk` and `Trash` become one each, shared across every account, owned by a **reserved "this computer" account id**, so `UNIQUE(account_id, path)` keeps working and no schema change is needed. `FOR_IMAP` stays `[Outbox]`. Reversibility: one-way.
- **D-19:** On first open of an existing database, each account's local `Sent`, `Outbox`, `Drafts`, `Junk` and `Trash` are **merged into the shared ones, message by message, and nothing is removed until every message has landed**. A summary says how many moved and from where, spoken and written to the log. Reversibility: one-way.
- **D-20:** A folder a user creates under a POP account goes **under that account's branch, beside its Inbox**. It is still local, so it never passes through `Allowed::mail`.
- **D-21:** Imported archives keep landing under `\u{1}Local/Imported`, now inside "On this computer", with **one branch per archive named after the file**. The user renames the branch afterwards, reusing FOLDER-01's rename.
- **D-22:** Nesting is stored, not computed. Sync splits the path where the delimiter is known and writes a **nullable `parent_id`** on the folders row. IMAP returns a delimiter per mailbox, not per server. The non-selectable rows IMAP already returns are the intermediate nodes. Reversibility: costly.
- **D-23:** Local folders nest with `/`. A name containing `/` is **escaped, not refused**. The escaped path is the stored identity, the real name is what a person reads, one escape function, one unescape, and a guard that they round-trip.
- **D-24:** A collapsed parent's unread announcement is a **setting** with two options. **Default: both always.**
- **D-25:** What the tree remembers across a restart is keyed by **stable identity, never by label**: account id for branches, `(account_id, path)` for folders, a stable id of its own for an imported archive.
- **D-26:** Rename changes the leaf **only**. Moving a folder to a different parent is its own `Move To` command on the Action menu, naming the new parent from a list, confirming, and saying where it went. Keyboard-only, no drag (WCAG 2.5.7).
- **D-27:** A folder the server no longer lists is **not removed without asking**, and the question is a **modal dialog, right away**. **One at a time**, and **not while an editor has focus**. Cached mail is untouched until the user answers.

**Favourites**

- **D-28:** Favourites sits **above `ALL_INBOXES`**, omitted entirely when nothing is pinned. Final order: Favourites, `ALL_INBOXES`, account branches, "On this computer", `Labels`, saved searches.
- **D-29:** Favourites **mirrors the account structure**: account sub-branches inside the group.
- **D-30:** Pinning makes a **copy**. The folder stays in its account branch.
- **D-31:** New pins go to the bottom of their account's group and move with the same **Alt+Shift+Up/Down** as accounts.
- **D-32:** A pin keys by the same stable identity as D-25. A folder marked gone by D-27 keeps its pin, and its Favourites row says gone too.

**Emptying and marking read**

- **D-33:** `Empty Folder` deletes every message **through the same decision one deletion uses**, `local_folders::deleting(from, protocol, asked, allowed)` per message, and the server path for server folders. The confirmation says which of the two will happen.
- **D-34:** Whether `Empty` reaches subfolders is a **setting**, defaulting to **including them**. The confirmation carries the whole cost: the folder, the total count, how many subfolders, and whether it moves or removes.
- **D-35:** Whether `Mark Folder Read` reaches subfolders is its **own setting**, also defaulting to **including them**.
- **D-36:** Emptying that fails partway **stops and says exactly where it got to**. Running it again finishes the job.
- **D-37:** The confirmation counts the **cache**, exactly, at confirmation time, and says the number is what is stored here. `folders.total_count` is not used, and no round trip is made in front of a dialog the user may cancel.
- **D-38:** `Empty` on an already-empty folder stays on the menu, enabled, and says so without a confirmation dialog.

**Settings this phase adds**

Five, each of which **must be wired to a real screen in this phase**.

| Setting | Options | Default | Decision |
|---|---|---|---|
| Delete scope on a conversation row | This folder's messages / the whole conversation | This folder | D-07, D-08 |
| How far a conversation reaches | The whole account / one folder | The whole account | D-08 |
| Unread announcement on a parent | Both numbers always / both when collapsed | Both always | D-24 |
| `Empty` reaches subfolders | Yes / no | Yes | D-34 |
| `Mark Folder Read` reaches subfolders | Yes / no | Yes | D-35 |

### Claude's Discretion

- The exact escape scheme for D-23, subject to the round-trip guard and the stored-path-is-identity rule.
- What THREAD-02's incremental rethreading does to a row the user is standing on, beyond the criterion already stating it must not re-announce rows they are not on.
- How the tree presents an account that has never synced.
- Whether `Move To` (D-26) reuses the destination picker the message-level Move already has.

### Deferred Ideas (OUT OF SCOPE)

- **A combined view of everything under an account branch**, the way `ALL_INBOXES` works across accounts.
- **SCALE-03 is linked to D-04 and the link is recorded rather than acted on.** Threading here is built to be correct on a partial mailbox; SCALE-03 (Phase 3) is what ends the label changing. The roadmap order is unchanged and Phase 1 closes on its own.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FOLDER-01 | Create, rename and delete a mail folder; mark a whole folder read; empty a folder. | Q1 gives the three async-imap methods and their exact signatures. Q2 and Q3 give the RFC contract for rename-with-children and delete-with-children. "The encoder that does not exist" gives the missing piece nobody itemised. The `may_i` / `Allowed::mail` gate is already in `imap.rs` and is one line per new verb. |
| FOLDER-02 | Nested folder hierarchy in the folder tree. | D-22's `parent_id` is additive under the shipped schema rule. The delimiter is already parsed and deliberately discarded at `imap.rs:769`. Tree identity keying (D-25) has a concrete recommendation and a documented trap to avoid. |
| FOLDER-03 | Pin frequently used folders as favourites. | Purely local, so no `Allowed` gate. Stores against the same `(account_id, path)` identity as D-25, which is what lets IMAP subscription back it later without a migration. `set_subscribed` already exists at `imap.rs:840`. |
| THREAD-01 | Collapse the message list to one row per conversation. | Q5 gives the virtual-list seam, the exact enable site for the disabled menu item, and what `MessageColumn` needs so D-02's one-rule-per-column-family survives. |
| THREAD-02 | Rethread incrementally as mail arrives, not only when a folder is opened. | Q4's finding that `messages.thread_id` is written by nothing changes what this requirement costs. `refresh_item` exists on the control. The merge case has a named constraint that a naive implementation will get wrong. |

</phase_requirements>

---

## Summary

Five of the six questions have answers that are better than expected, and one has an answer that is worse. The good news first. All three IMAP mailbox commands this phase needs already exist in `async-imap 0.11.3` with ordinary signatures, so no raw command execution is required. The RFC contract for both hard cases is unambiguous and favourable: `RENAME` moves a whole subtree in one command, so D-26's `Move To` needs no recursive walk, while `DELETE` explicitly refuses to touch children, so deleting a subtree does need one. The reply-prefix stripping D-04 asks for is already implemented, correctly and in nineteen languages, by `mail-parser`, which is already a dependency. And `wxdragon 0.9.17` exposes `refresh_item`, which is exactly what THREAD-02's in-place row update needs on a virtual list.

The bad news is in the data layer, and it is the single most important thing in this document. **`messages.thread_id` is a column that nothing writes and nothing reads.** The whole live threading path is an in-memory computation in the presentation layer, run over one folder's loaded page, whose result is never persisted. One query does sort on the column, so sorting by Thread today orders every row by NULL without failing. D-08 asks for a conversation that spans an account, and there is no stored, account-wide conversation key to span it with. That is not a query to write against existing data; it is a producer to build first.

Three further findings shape the plan. `mailbox_name.rs` decodes modified UTF-7 mailbox names and does not encode them, and its own module comment says an encoder "belongs here the day something creates or renames a mailbox", which is this phase; without it, creating a folder called `Entwürfe` sends raw UTF-8 inside a quoted string, which most servers will refuse. `wxdragon`'s tree item custom data leaks into a process-global registry that its own cleanup routine fails to clear, so D-25 should not be built on it. And an existing test already proves every setting is read by something but deliberately excludes the settings screen, so it cannot catch the exact FEEDBACK-01 failure CONTEXT.md warns about; the mirror of that test is a cheap, high-value addition.

**Primary recommendation:** Order the phase so the thread identity producer lands before anything that reads it. Write the modified UTF-7 encoder and the three IMAP verbs against the existing loopback harness first, because they are self-contained and the harness needs three new scripted verbs before any of them can be tested at all. Take `thread_name` from `mail-parser` rather than writing a stripper. Key the tree from a parallel vector built during the rebuild, not from tree item data.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CREATE / RENAME / DELETE mailbox | Service (`service/protocols/imap.rs`) | Application (`MailController`) | The wire verb and its permission gate belong beside every other verb, where `may_i` and `protocol_error` already live. |
| Modified UTF-7 encoding of a new mailbox name | Service (`service/protocols/imap/mailbox_name.rs`) | — | Its decoding sibling is already there and its module comment says the encoder belongs there too. Encoding is a wire concern, not a UI one. |
| Deciding local versus server for a folder operation | Application (`local_folders::is_local`) | — | FOLDER-01's own criterion says this is decided there "and nowhere else". |
| Storing folder nesting (`parent_id`) | Data (`message_cache/folders.rs`) | Application (`mail_sync.rs` computes it) | D-22 says nesting is stored, not computed. The split happens once at sync; the tree reads a parent. |
| Conversation identity across an account | Application (`threading.rs`) + Data (persisted) | — | Currently presentation-only and per-page. D-08 cannot be served from the presentation tier because a folder's page is not an account. |
| Conversation column aggregation (D-02) | Data (SQL `GROUP BY`) | Presentation (`MessageColumn` supplies the expression) | The aggregate must drive both display and `ORDER BY` from one place, which is what `MessageColumn` already does for messages. |
| Rendering a conversation row | Presentation (`message_rows::cell_text`, virtual callback) | — | The list holds no text; every cell is answered from memory during paint. |
| Tree structure, level, expand state | Presentation (`wx_app.rs` `UIUpdate::FoldersLoaded`) | — | The level must come from the native `TreeCtrl`, per CLAUDE.md, not from label text. |
| Folder identity across a rebuild (D-25) | Presentation state (`WxUIState`) | — | The tree control is rebuilt from scratch on every sync; identity has to survive outside it. |
| The five new settings | Data (`AppConfig`) + Presentation (`wx_settings.rs`) | — | Both halves, or it is FEEDBACK-01 again. |
| The D-19 migration | Application, run once at open | Data (the message moves) | It rewrites `folder_id` on the user's only copy of that mail, so it belongs where it can be tested without a UI. |

---

## Project Constraints (from CLAUDE.md)

These carry the same authority as the locked decisions above.

| Constraint | Source | What it forbids or requires here |
|---|---|---|
| Red/green TDD on every eligible task, not opportunistically | `CLAUDE.md` §Test-driven development; `.planning/config.json` has `"tdd_mode": true` [VERIFIED: .planning/config.json:1-6] | Every task in this phase that changes behaviour is `type: tdd`. The RED and GREEN gate commits are checked. |
| Errors flow through `common::Error`; no `unwrap`/`expect` outside tests | `CLAUDE.md` §Elegant code | The three new IMAP verbs map `async_imap::error::Error` through the existing `protocol_error` helper. |
| **Schema changes are additive only** | `CLAUDE.md` §Project rules, quoted verbatim: "Never drop or rename a column that shipped." | D-22's `parent_id` goes in with `ensure_column_exists`. Any thread-identity column does too. Nothing about `messages.thread_id` may be renamed even though nothing writes it. |
| No AI attribution anywhere | `CLAUDE.md` §Project rules | No `Co-Authored-By` naming an AI, and no AI names in commit messages, branch names, comments or docs. |
| Two accessibility channels, both right | `CLAUDE.md` §Accessibility, quoted verbatim: "UI Automation is what Narrator reads. MSAA, through `IAccessible`, is what NVDA reads for native controls, and it is the only place `set_accessible_name` writes" | A new check box named only by `set_accessible_name` is unnamed under Narrator. `tests/checkbox_labels.rs` is the existing guard for this and the five new settings must pass it. |
| Level comes from the native control, never the label | `CLAUDE.md` §Accessibility; D-01 and FOLDER-02 both restate it | Nesting depth is never spelled into the row text. |
| Keyboard only, no drag | `CLAUDE.md` §Physical and motor; WCAG 2.5.7 | D-26's `Move To` is a command with a destination list, not a drag. |
| `docs/KEYBOARD_SHORTCUTS.md` updated in the same commit as a shortcut | `CLAUDE.md` §Physical and motor | Alt+Shift+Up/Down (D-14, D-31) lands with its documentation. |
| `guards/guards.toml` records what edit must break which test | `CLAUDE.md` §Tests that would notice; `guards/guards.toml` is 9078 lines [VERIFIED: guards/guards.toml, `wc -l`] | Each guard entry names a `before` string that must appear exactly once in the tree, an `after`, and the exact set of tests that go red. Measured by hand, never guessed. |
| Windows-first; platform code behind `#[cfg(target_os = "windows")]` | `CLAUDE.md` §Project rules | Nothing in this phase should need it, but the tree and list work must keep the crate building elsewhere. |
| Plain language in labels, messages and errors | `CLAUDE.md` §Learning and cognitive | The confirmations D-34, D-36 and D-37 specify are already written in that register in CONTEXT.md; keep them. |

---

## The six questions, answered

### Q1. `async-imap 0.11.3`'s mailbox management surface

All three commands exist as ordinary methods on `Session`. No raw command execution is needed. Read from the vendored crate source this session.

| Command | Signature | Source |
|---|---|---|
| CREATE | `pub async fn create<S: AsRef<str>>(&mut self, mailbox_name: S) -> Result<()>` | [VERIFIED: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-imap-0.11.3/src/client.rs:578] |
| DELETE | `pub async fn delete<S: AsRef<str>>(&mut self, mailbox_name: S) -> Result<()>` | [VERIFIED: async-imap-0.11.3/src/client.rs:604] |
| RENAME | `pub async fn rename<S1: AsRef<str>, S2: AsRef<str>>(&mut self, from: S1, to: S2) -> Result<()>` | [VERIFIED: async-imap-0.11.3/src/client.rs:636] |

Each is a two-line body that formats the verb and calls `run_command_and_check_ok`, so an `Ok(())` means the server returned a tagged OK and nothing else. This is the shape `set_subscribed` already uses at `imap.rs:840`, which is the pattern to copy.

**How arguments are escaped, and the gap.** All three pass the name through `validate_str`, which is [VERIFIED: async-imap-0.11.3/src/client.rs:1510-1519]:

```rust
fn validate_str(value: &str) -> Result<String> {
    let quoted = quote!(value);
    if quoted.find('\n').is_some() {
        return Err(Error::Validate(ValidateError('\n')));
    }
    if quoted.find('\r').is_some() {
        return Err(Error::Validate(ValidateError('\r')));
    }
    Ok(quoted)
}
```

and `quote!` is [VERIFIED: async-imap-0.11.3/src/client.rs:25-29]:

```rust
macro_rules! quote {
    ($x:expr) => {
        format!("\"{}\"", $x.replace(r"\", r"\\").replace("\"", "\\\""))
    };
}
```

So the crate wraps the name in a quoted string and escapes backslash and double quote, and rejects only `\n` and `\r`. **It does no modified UTF-7 encoding at all.** A whole-tree search of the crate for any UTF-7 handling found nothing but the `imap-proto` dependency line [VERIFIED: grep over async-imap-0.11.3/src and Cargo.toml, no `utf.?7` match]. A folder named `Entwürfe` is therefore sent as raw UTF-8 inside a quoted string, which is not legal IMAP4rev1 unless the server has been put into UTF-8 mode with `ENABLE UTF8=ACCEPT` (RFC 6855). This is the encoder the codebase already predicted it would need, below.

**How errors come back.** `async_imap::error::Error` is `#[non_exhaustive]` with variants `Io`, `Bad(String)`, `No(String)`, `ConnectionLost`, `Parse(ParseError)`, `Validate(ValidateError)`, `Append` [VERIFIED: async-imap-0.11.3/src/error.rs:11-36]. A server refusal arrives as `No(String)` carrying the server's own words. The mapping onto `common::Error` already exists and should be reused unchanged [VERIFIED: src/service/protocols/imap.rs:1837-1844]:

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

Note that this collapses `No`, `Bad` and `Io` into one `Error::Protocol`. That is the established convention and matches `set_subscribed`. If FOLDER-01 wants to tell "the server said no" apart from "the connection dropped" in the message the user hears, that distinction has to be drawn at the call site before `protocol_error` runs, because it is lost afterwards.

**The permission gate.** Each new verb starts with one line, exactly as `set_subscribed` does [VERIFIED: src/service/protocols/imap.rs:840-841]:

```rust
pub async fn set_subscribed(&mut self, path: &str, subscribed: bool) -> Result<()> {
    self.may_i("change which folders you are subscribed to")?;
```

and `may_i` delegates to the one gate [VERIFIED: src/service/protocols/imap.rs:1067-1069]:

```rust
fn may_i(&self, doing: &str) -> Result<()> {
    crate::service::outward::permitted(self.may_change, doing)
}
```

which returns `Err(Error::Security(refusal(doing)))` when writes are off [VERIFIED: src/service/outward.rs:292-297]. FOLDER-01's criterion that a refusal is "refused with a message saying why rather than attempted and failed" is satisfied by this path, and `was_refused_by_the_gate` at `src/service/outward.rs:305` is how a caller tells a gate refusal from a server refusal.

### Q2. RENAME is both rename and move, and D-26 splits them

RFC 9051 §6.3.6 states, verbatim: "If the name has inferior hierarchical names, then the inferior hierarchical names MUST also be renamed." [CITED: rfc-editor.org/rfc/rfc9051.txt §6.3.6]. RFC 3501 §6.3.5 says the same thing with "will also be renamed", and the crate's own doc comment restates it with the worked example: "a rename of `foo` to `zap` will rename `foo/bar` (assuming `/` is the hierarchy delimiter character) to `zap/bar`" [VERIFIED: async-imap-0.11.3/src/client.rs:615-617].

RFC 9051 also states that a server "will generally create any superior hierarchical names that are needed for the RENAME command to complete successfully" [CITED: async-imap-0.11.3/src/client.rs:619-623 restating RFC 3501 §6.3.5], so renaming `a/b` to `x/y/b` generally creates `x` and `x/y`.

**What this means for the plan.** `Move To` (D-26) is **one RENAME**, not a recursive walk. Renaming `Archive/2026` to `Backup/2026` moves the folder and every descendant in a single command, and the server creates `Backup` if it is missing. This is the cheap case, and it is the one D-26 assigns to the more dangerous-feeling command.

Rename-the-leaf (also D-26) is the same verb with the parent path held constant: `Archive/2026` becomes `Archive/2025`. The two commands differ only in which half of the path the user is allowed to change. That is the whole of D-26's safety argument and it costs nothing to implement.

**INBOX is a special case that must be refused before it is attempted.** RFC 9051 §6.3.6: "Renaming INBOX is permitted and does not result in a tagged BAD response, and it has special behavior: It moves all messages in INBOX to a new mailbox with the given name, leaving INBOX empty." [CITED: rfc-editor.org/rfc/rfc9051.txt §6.3.6]. A user who thinks they are renaming a folder and instead empties their inbox into a new one has suffered a data-shaped surprise that succeeded. Refuse rename and move on a folder whose `folder_type` is `Inbox`, with a reason, before the command is built.

**What servers do differently.** This is the one place the RFC does not settle it. [ASSUMED] Reports of divergence exist for servers that store mailboxes as filesystem directories versus as a flat namespace with a delimiter in the name, and Gmail's IMAP layer presents labels rather than folders, so its rename semantics for a nested label are its own. No authoritative source was reached this session that enumerates per-server behaviour, and this project cannot test against a live server. Treat a `NO` response from RENAME as an ordinary, expected refusal with the server's words shown, rather than as an error state, and do not build any logic that depends on the subtree having moved without re-listing.

### Q3. Deleting a mailbox that has children

RFC 9051 §6.3.5 states, verbatim: "The DELETE command MUST NOT remove inferior hierarchical names." [CITED: rfc-editor.org/rfc/rfc9051.txt §6.3.5]. The rules, from the same section and from the crate's doc comment [VERIFIED: async-imap-0.11.3/src/client.rs:585-596]:

| Case | Behaviour |
|---|---|
| Name has inferior names **and** has `\Noselect` | "It is an error to attempt to delete a name that has inferior hierarchical names and also has the `\Noselect` mailbox name attribute." The server returns `NO`. RFC 9051 adds the `HASCHILDREN` response code for this refusal. |
| Name has inferior names and does **not** have `\Noselect` | Permitted. All messages in that mailbox are removed and the name itself acquires `\Noselect`, remaining in the hierarchy as a pure container. |
| Name is `INBOX` | "It is an error to attempt to delete `INBOX`". |
| Name does not exist | An error. |

**What this means for the plan.** Deleting a subtree requires a client-side depth-first walk, deepest child first, and this is the opposite of RENAME. Concretely:

1. Read the current children of the target from the stored `parent_id` tree (D-22), not from a fresh LIST, unless the tree is stale.
2. Delete depth-first, deepest first, so no DELETE is ever issued against a name that still has inferior names.
3. Expect that deleting a selectable parent leaves a `\Noselect` shell behind if any child survived. Re-list afterwards rather than assuming the row is gone.

This walk is also where D-36's partial-failure reporting has to live: "Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the server refused." The same shape applies to delete. There is no transaction to wrap it in, which is the reason D-36 rules out all-or-nothing, and the same reason applies here.

The `\Noselect` attribute is already modelled. `ImapFolder.selectable` comes from `special_use::selectable(&attributes)` [VERIFIED: src/service/protocols/imap.rs:781], and CONTEXT.md's own note is right that these non-selectable rows are D-22's intermediate nodes.

### Q4. Cross-folder conversation counting (D-08)

This is the question whose answer changes the shape of the phase.

**The schema, read this session.** The `folders` table [VERIFIED: src/data/message_cache/mod.rs:1292-1302], quoted verbatim:

```sql
CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    folder_type TEXT NOT NULL,
    unread_count INTEGER DEFAULT 0,
    total_count INTEGER DEFAULT 0,
    UNIQUE(account_id, path)
)
```

`holds_all_mail` and `subscribed` are added later, additively [VERIFIED: src/data/message_cache/mod.rs:2099 and :2103], quoted verbatim:

```rust
self.ensure_column_exists("folders", "holds_all_mail", "INTEGER NOT NULL DEFAULT 0")?;
self.ensure_column_exists("folders", "subscribed", "INTEGER NOT NULL DEFAULT 1")?;
```

`messages.thread_id` and `messages.thread_depth` are likewise additive [VERIFIED: src/data/message_cache/mod.rs:2065-2066], verbatim:

```rust
self.ensure_column_exists("messages", "thread_id", "TEXT")?;
self.ensure_column_exists("messages", "thread_depth", "INTEGER")?;
```

**The finding: `messages.thread_id` is written by nothing.** A search of `src/data/` for `thread_id` returns exactly one line, the `ensure_column_exists` above [VERIFIED: grep over src/data/, single match at mod.rs:2065]. A whole-tree search for `thread_id` outside tests returns occurrences only in `application/threading.rs` (the in-memory type), `presentation/*` (the in-memory `MessageItem` field), and one SQL reference [VERIFIED: tree-wide grep over src/, 40 lines reviewed]. That one SQL reference is [VERIFIED: src/presentation/message_columns.rs:131], verbatim:

```rust
MessageColumn::Thread => "m.thread_id",
```

So sorting the message list by the Thread column today issues a valid `ORDER BY m.thread_id` against a column that is NULL on every row. It does not fail; it silently does nothing. That is a pre-existing defect this phase will land on top of, and it is worth a task of its own.

**Where the real thread id comes from, and why it cannot serve D-08.** The only caller of `thread_messages` is `apply_threading` in the presentation layer [VERIFIED: src/presentation/wx_app.rs:7966, sole call site by tree-wide grep], and it runs over `rows: &[MessageListRow]`, which is one folder's loaded listing. The id it computes is defined as [VERIFIED: src/application/threading.rs:106-110], verbatim:

> "The conversation is named after the least Message-ID it holds, in plain string order, rather than a generated value. Least rather than oldest, because a date can be missing, wrong or in another timezone, and the point of the rule is that the same mailbox threads the same way on every machine and after every restart."

**Two consequences follow, and both are load-bearing.**

1. The id depends on which messages are in the batch. The same conversation, batched per folder, gets a different `thread_id` in Inbox than in Archive if the least Message-ID present differs. **D-08 cannot key a cross-folder count on the value this function produces per folder.** The batch has to be the account, or the id has to be made independent of the batch.
2. THREAD-02's incremental join cannot simply adopt the existing thread's id. If an arriving message carries a lexicographically smaller Message-ID than the current `thread_id`, the whole conversation's id changes under the rule above. An implementation that looks up the references, finds a thread, and adopts its id will disagree with the batch recompute that runs on the next folder open, and the row's identity will flip. This is the "two answers to one question" shape recorded in `feedback_two_answers_to_one_question`. The incremental path and the batch path must be one function, or the incremental path must be defined as "recompute the affected component", not "assign the found id".

**The query shape, once a stable account-wide thread id exists.** D-08 asks for a count over every message in a thread across the account, excluding all-mail folders. Assuming a persisted `messages.thread_id` and the account reached through the join that every existing query uses:

```sql
SELECT m.thread_id,
       COUNT(*)                              AS messages,
       SUM(CASE WHEN m.read = 0 THEN 1 ELSE 0 END) AS unread
FROM messages m
INNER JOIN folders f ON m.folder_id = f.id
WHERE f.account_id = ?1
  AND f.holds_all_mail = 0
  AND m.deleted = 0
  AND m.thread_id IN (SELECT thread_id FROM messages WHERE folder_id = ?2)
GROUP BY m.thread_id
```

**The indexes that exist, and the one that does not.** Twenty-five `CREATE INDEX` statements are present [VERIFIED: src/data/message_cache/mod.rs:2421-2508, enumerated by tree-wide grep]. The ones relevant here, verbatim:

```sql
CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)
CREATE INDEX IF NOT EXISTS idx_messages_folder_date ON messages(folder_id, date DESC, uid DESC)
CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC, uid DESC)
CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(uid)
```

**There is no index on `messages.thread_id` and none on `messages.message_id`.** Verified by enumerating every `idx_` name defined anywhere in `src/`: the twenty-five names include `idx_attachments_message_id`, which is on the `attachments` table, and nothing on either message column [VERIFIED: tree-wide `grep -rhoE "idx_[a-z_]+" src/`, 25 unique names, no thread_id and no messages.message_id]. There are no `.sql` files in the repository, so `initialize_schema` is the whole story [VERIFIED: `find . -name "*.sql"` returns nothing outside target/].

The existing indexes do not cover the D-08 query. The codebase already states the reason in its own words, and the comment is worth quoting because it is the exact rule that decides this [VERIFIED: src/data/message_cache/messages.rs:101-104], verbatim:

> "This one names no folder, so the index that serves a single folder cannot serve it: an index is searched from its leftmost column and that one begins with `folder_id`. Without an index in the sort's own order SQLite reads every message in every inbox, sorts the lot and keeps a screenful."

Two indexes are needed, both additive:

```sql
CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id)
CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages(message_id)
```

The first serves the `GROUP BY` and the `IN` subquery. The second serves THREAD-02's incremental lookup, which the mail-at-scale plan already specifies as "each arriving message looks up its references against an index on `message_id`" [CITED: .planning/REQUIREMENTS.md, THREAD-02 evidence]. `folders(account_id, path)` already has an implicit index from its `UNIQUE` constraint whose leftmost column is `account_id`, so the join's `f.account_id = ?1` is covered.

**`holds_all_mail` is genuinely populated, unlike `thread_id`.** It is written by `set_folder_server_facts` [VERIFIED: src/data/message_cache/folders.rs:105-118], verbatim:

```sql
UPDATE folders SET holds_all_mail = ?1, subscribed = ?2 WHERE id = ?3
```

called from `mail_sync.rs:436`. So D-08's exclusion is a real predicate over real data.

### Q5. The virtual list under thread view

**How the list works today.** The list is virtual and holds no text, which `wx_app.rs` states in its own words [VERIFIED: src/presentation/wx_app.rs:9811-9821], verbatim:

> "The message list is virtual: it holds no text at all, and answers the control's questions from `state` as rows are painted. `GetItemText` on a virtual list is not a question wxWidgets promises to answer, so asking it would have made Copy return nothing on the one list where mail lives, and the failure would have read as 'nothing is chosen' rather than as a fault."

The callback is registered once and closes over the shared state and the column layout [VERIFIED: src/presentation/wx_app.rs:954-974]. Its body reads `state.messages.get(row as usize)`, takes `column_layout.borrow().visible()`, and calls `message_rows::cell_text(message, *c, date_settings, now)`. `state.messages` is `Vec<MessageItem>` [VERIFIED: src/presentation/wx_app.rs:213].

**What changes when a row is a conversation.** The seam is narrow and does not require replacing the control, which D-01 forbids anyway. Three things move:

1. **The backing vector.** The callback indexes one `Vec`. Thread view needs a second shape. The smallest change that keeps the callback total and infallible is a `Vec` of an enum, or two vectors and a mode flag in state. Either way `set_item_count` is called with the conversation count instead of the message count, which is what makes UI Automation report the right set size, which is the whole of D-01's accessibility argument.
2. **`cell_text` gains a sibling.** `message_rows::cell_text(&MessageItem, MessageColumn, ...) -> String` is a pure function over data already in memory [VERIFIED: src/presentation/message_rows.rs:24-28]. A `conversation_cell_text(&ConversationItem, MessageColumn, ...)` alongside it keeps the module's stated contract, which is that the callback "cannot query the database, cannot block, and has nowhere to report an error to" [VERIFIED: src/presentation/message_rows.rs:3-6].
3. **`MessageColumn` gains an aggregate sort expression.** `sort_expression` returns `&'static str` and is built by matching on the enum with fixed strings [VERIFIED: src/presentation/message_columns.rs:116-141]. Its doc comment states why, verbatim: "Fixed strings chosen by matching on the enum, never built from anything a user typed, because the result is interpolated into a query." D-02's aggregates are also fixed strings, so a sibling method returning `MAX(COALESCE(m.internaldate, m.date))`, `MAX(m.starred)`, `SUM(m.size_bytes)` and so on preserves both the safety property and D-02's one-rule-per-column-family requirement. Do not build the aggregate from the message expression by string surgery; that is how the two come apart.

**D-02's premise was checked, not assumed.** D-02 turns on `MessageColumn` remaining the single source of both the displayed value and the `ORDER BY`. There is a second sort vocabulary in the tree, `MailSortOption`, which could have been a competing producer. It is not: a census of every non-test reference finds it only in `presentation/ui_types.rs` where it is defined, in `presentation/message_columns.rs` where it is converted to and from a `MessageColumn` plus direction by `as_mail_sort_option` (329) and `set_sort_from_option` (510), and in `data/config.rs` as the stored `default_sort_order` string. It reaches no SQL [VERIFIED: tree-wide grep for `MailSortOption`, no match anywhere under `src/data/` except the unrelated `default_sort_order` config field]. `Sort::order_by_clause` [VERIFIED: src/presentation/message_columns.rs:302] is the only `ORDER BY` producer for the message list, so adding the aggregate to the same enum keeps it that way.

**`apply_columns` implies almost nothing.** It clears the control and inserts one column per visible column with a width chosen by matching on the enum [VERIFIED: src/presentation/wx_app.rs:6210-6227]. It is indifferent to what a row means. The only consequence for this phase is D-05 and D-06: whether `Thread` is in `layout.visible()` is decided before `apply_columns` runs, and D-06's tri-state ("never chosen" / "chosen on" / "chosen off") has to live in whatever stores the layout, not in `apply_columns`.

**What `wxdragon 0.9.17` gives you.** Verified against the vendored crate source:

| Method | Line | Use here |
|---|---|---|
| `set_item_count(&self, count: i64)` | [VERIFIED: wxdragon-0.9.17/src/widgets/list_ctrl.rs:771] | Switching between message count and conversation count. |
| `refresh_item(&self, item: i64)` | [VERIFIED: wxdragon-0.9.17/src/widgets/list_ctrl.rs:781] | **THREAD-02's in-place update.** Repaints one row without touching the others, which is exactly the criterion "does not re-announce rows the user is not on". |
| `refresh_items(&self, from: i64, to: i64)` | [VERIFIED: wxdragon-0.9.17/src/widgets/list_ctrl.rs:791] | A merge that changes a contiguous run. |
| `set_item_state`, `get_item_state`, `get_next_item`, `get_first_selected_item` | [VERIFIED: list_ctrl.rs:485, 520, 531, 541] | D-11's selection preservation across a view switch. |
| `ensure_visible(&self, item: i64)` | [VERIFIED: list_ctrl.rs:604] | Keeping the cursor where it was, the same reason `land_the_cursor` exists for the tree. |
| `set_virtual_text_callback`, `clear_virtual_text_callback` | [VERIFIED: list_ctrl.rs:806, 839] | Already used; no need to re-register on a view switch if the callback reads the mode from state. |

There is **no per-item accessible description or name API on `ListCtrl`** in this version [VERIFIED: `grep "pub fn "` over list_ctrl.rs, 48 methods enumerated, none accessibility-specific]. What a screen reader reads for a conversation row is therefore exactly the cell text, and nothing else. That is not a limitation to work around; it is the reason THREAD-01's criterion says the announcement is "assembled from the visible columns the way any other row is", and D-03 is what makes that possible.

**The menu item to enable.** `ID_THREAD_VIEW` is declared at [VERIFIED: src/presentation/wx_app.rs:102], added as a check item at [VERIFIED: src/presentation/wx_app.rs:5025-5029] with label `"&Thread View\tCtrl+T"`, and disabled at [VERIFIED: src/presentation/wx_app.rs:580-585], verbatim:

```rust
if let Some(item) = frame
    .get_menu_bar()
    .and_then(|bar| bar.find_item(ID_THREAD_VIEW))
{
    item.enable(false);
}
```

That block is the whole of what "enable the View menu item" means. The comment above the menu item at line 5021-5023 says "Threading is not implemented. The item stays visible and disabled rather than pretending to work" and should be removed in the same commit, or it becomes a false comment.

**`conversation_size` already exists and is the wrong shape for D-08.** [VERIFIED: src/presentation/message_rows.rs:138-150] counts a thread across the loaded list only, and its doc comment says so, verbatim: "Counted across the loaded list rather than asked of the server, because the list is what Space is reading and the answer has to arrive with the key rather than after it." It returns `None` for a conversation of one, which is D-05's rule already implemented. Keep the function for Space's read-aloud; D-08's count is a different question with a different answer, and giving them two names is right.

### Q6. Reply-prefix stripping (D-04)

**Nothing strips prefixes today, verified by tree-wide search.** A search for functions named after subject or prefix normalisation returns nothing that strips [VERIFIED: `grep -rnE "fn [a-z_]*(subject|prefix)[a-z_]*\("` over src/, 20 matches reviewed, all tests or unrelated]. The only production code that touches reply prefixes prepends them [VERIFIED: src/presentation/wx_compose.rs:160-176], verbatim:

```rust
/// Format a reply subject line: prepends "Re: " unless already present.
fn format_reply_subject(subject: &str) -> String {
    if subject.starts_with("Re: ") {
        subject.to_string()
    } else {
        format!("Re: {}", subject)
    }
}

/// Format a forward subject line: prepends "Fwd: " unless already present.
fn format_forward_subject(subject: &str) -> String {
    if subject.starts_with("Fwd: ") {
        subject.to_string()
    } else {
        format!("Fwd: {}", subject)
    }
}
```

**This is a second-spelling hazard and the plan must name it.** These two functions answer the question "does this subject already carry a reply prefix?" with a case-sensitive, ASCII-only, single-prefix `starts_with`. The stripper D-04 asks for answers the same question far better. Two answers to one question is the shape recorded in `feedback_two_answers_to_one_question` and in this project's own history. Reply to a message whose subject is `AW: Angebot` and today you get `Re: AW: Angebot`; the new stripper will read that as one conversation while `format_reply_subject` keeps growing the chain. Rewrite both functions in terms of the stripper: prepend only when stripping the subject changes nothing.

**The RFC gives the algorithm, not the prefix list.** RFC 5256 defines "base subject" extraction for `THREAD=REFERENCES` and `SORT`. The ABNF, quoted verbatim [CITED: rfc-editor.org/rfc/rfc5256.txt]:

```abnf
subject         = *subj-leader [subj-middle] *subj-trailer
subj-refwd      = ("re" / ("fw" ["d"])) *WSP [subj-blob] ":"
subj-blob       = "[" *BLOBCHAR "]" *WSP
subj-fwd-hdr    = "[fwd:"
subj-fwd-trl    = "]"
subj-leader     = (*subj-blob subj-refwd) / WSP
subj-trailer    = "(fwd)" / WSP
```

and the procedure, verbatim from the same source:

1. Convert any RFC 2047 encoded-words to UTF-8. Convert all tabs and continuations to space. Convert all multiple spaces to a single space.
2. Remove all trailing text matching `subj-trailer`; repeat until no more matches are possible.
3. Remove all prefix text matching `subj-leader`.
4. If there is prefix text matching `subj-blob`, and removing it leaves a non-empty `subj-base`, remove it.
5. Repeat (3) and (4) until no matches remain.
6. If the resulting text begins with `subj-fwd-hdr` and ends with `subj-fwd-trl`, remove both and repeat from (2).
7. The resulting text is the base subject.

Three things matter about this. It is a **fixpoint** loop, so `Re: Re: Fwd: Re: lunch` reduces fully. It handles a **bracketed blob** between the prefix and the colon, so `Re: [SPAM] lunch` and `Re[2]: lunch` are covered. And **it covers only English `re`, `fw` and `fwd`.** Localised prefixes are not in any RFC; they are what non-English Outlook emits, and every client that handles them does so from a hand-maintained list [CITED: office-watch.com/2014/outlook-reply-forward-prefixes/; help.gnome.org/users/evolution/stable/mail-localized-re-subjects.html.en; bugzilla.mozilla.org/show_bug.cgi?id=634896].

**A crate already in `Cargo.toml` implements all of it. Do not write this.** `mail-parser` is pinned as `"0.11"` [VERIFIED: Cargo.toml:49] and resolves to `0.11.5` [VERIFIED: Cargo.lock, `name = "mail-parser"` / `version = "0.11.5"`]. It exposes:

```rust
mail_parser::parsers::fields::thread::thread_name(&str) -> &str
```

The path is fully public: `pub mod parsers` [VERIFIED: mail-parser-0.11.5/src/lib.rs:12], `pub mod fields` [VERIFIED: mail-parser-0.11.5/src/parsers/mod.rs:9], `pub mod thread` [VERIFIED: mail-parser-0.11.5/src/parsers/fields/mod.rs:14], `pub fn thread_name` [VERIFIED: mail-parser-0.11.5/src/parsers/fields/thread.rs]. It takes a plain `&str`, so it can be called on a subject already stored in the cache with no message to parse. There is no re-export at the crate root, so the full path is required.

Its reply-prefix set, quoted verbatim from the source [VERIFIED: mail-parser-0.11.5/src/parsers/fields/thread.rs, `is_re_prefix`]:

```
"re", "res", "sv", "antw", "ref", "aw", "απ", "השב", "vá", "r",
"rif", "bls", "odp", "ynt", "atb", "رد", "回复", "转发"
```

Its forward-prefix set, verbatim [VERIFIED: same file, `is_fwd_prefix`]:

```
"fwd", "fw", "rv", "enc", "vs", "doorst", "vl", "tr", "wg", "πρθ",
"הועבר", "továbbítás", "i", "fs", "trs", "vb", "pd", "i̇lt", "yml",
"إعادة توجيه", "回覆", "轉寄"
```

That covers every prefix named in the phase brief (`AW:`, `SV:`, `VS:`, `RE:`, `R:`, `Antw:`, `Odp:`) and fifteen more languages besides. The implementation is a single-pass character scan that handles `[blob]` and `[fwd: ... ]` in the RFC 5256 shape, lowercases the token before matching, and stops at the first token that is not a known prefix.

**Two cautions.** The lists include single letters `r` and `i`, so a subject like `R: 2026 budget` strips to `2026 budget` and, in the forward set, `I: something` strips too. That is deliberate on the crate's part (Italian and Hungarian) and matches what other clients do, but it is a behaviour worth a test of its own so nobody is surprised. And `thread_name` returns a `&str` borrowed from the input, so a stored, owned subject is needed to hold it.

**Where the stripper must NOT be wired.** `threading.rs` states, verbatim [VERIFIED: src/application/threading.rs:7-10]:

> "Subject matching is deliberately not used. 'Re: lunch' collides across years and strangers, and a thread that quietly merges two unrelated conversations is worse than two threads: someone reading by ear has no way to see that the sender changed halfway down."

D-04 is a **display label** rule, not a threading rule. The stripped subject names the conversation; it never decides which conversation a message belongs to. An executor who sees a subject normaliser land and wires it into `thread_messages` will silently reverse a decision the module argues for at length.

---

## The finding nobody itemised: the encoder that does not exist

`src/service/protocols/imap/mailbox_name.rs` is 159 lines and exposes exactly one public function, `pub fn decode(encoded: &str) -> String` at line 46 [VERIFIED: src/service/protocols/imap/mailbox_name.rs:46, sole `pub fn` by grep]. Its module comment says the rest, verbatim [VERIFIED: src/service/protocols/imap/mailbox_name.rs:1-18]:

> "IMAP4rev1 carries non-ASCII mailbox names in a modified UTF-7 encoding (RFC 3501 section 5.1.3), so a German user's Drafts folder arrives as `Entw&APw-rfe` and a Japanese one as `&ZeVnLIqe-`. Left alone, that is what the folder tree announces, which for a screen reader user is not a slightly ugly label but an unreadable one: the synthesiser spells out punctuation.
>
> Two differences from ordinary Base64 UTF-7: `/` is written as `,` because `/` is a common hierarchy delimiter, and the padding `=` is omitted.
>
> Only decoding lives here. Nothing sends a mailbox name the client made up: a name goes back to the server exactly as the server spelled it, which is also the only way a name we could not decode stays reachable. **An encoder belongs here the day something creates or renames a mailbox.**
>
> RFC 6855 lets a client ask for UTF-8 mailbox names with `ENABLE UTF8=ACCEPT`, and plenty of servers still do not offer it, so decoding stays necessary."

That day is this phase. FOLDER-01's evidence itemises the three missing verbs and does not mention the encoder; neither does the roadmap or the discussion. Without it, CREATE and RENAME work for ASCII names and fail or corrupt for every other alphabet, which is a defect that would ship looking like a working feature to anyone testing in English.

The encoder is well specified and small. The alphabet is already defined in the same file as `MODIFIED_BASE64`, a `GeneralPurpose` engine over `"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+,"` with padding off [VERIFIED: src/service/protocols/imap/mailbox_name.rs:27-39], so encoding reuses the same engine in the other direction. The rules: printable ASCII except `&` passes through; a literal `&` becomes `&-`; any run of non-ASCII is encoded as UTF-16BE, base64'd with the modified alphabet, wrapped in `&` and `-`.

**The guard this needs is a round trip**, and the file's existing structure invites it: `decode(encode(name)) == name` for an alphabet-spanning corpus, plus the specific cases the module comment already names (`Entw&APw-rfe`, `&ZeVnLIqe-`). D-23 asks for exactly this shape of guard for the local-folder escape scheme, so the two are siblings and should be written by the same hand.

**Where to call it.** Only on a name the client made up, which is CREATE and the destination half of RENAME. Never on a path that came from the server. The module's own reason for that rule, verbatim from the same comment: "a name goes back to the server exactly as the server spelled it, which is also the only way a name we could not decode stays reachable."

---

## Architecture Patterns

### System architecture diagram

```
    KEYBOARD (Alt+Shift+Arrow, Ctrl+T, Applications key, Enter)
        |
        v
  +-----------------------------+        +---------------------------+
  |  Folder tree (TreeCtrl)     |        |  Message list (virtual    |
  |  - native level & expand    |        |  ListCtrl, holds no text) |
  |  - rebuilt whole on sync    |        |  - set_item_count(n)      |
  +-----------------------------+        |  - virtual text callback  |
        |            ^                   +---------------------------+
        |            | land_the_cursor              |         ^
        v            |                              v         | refresh_item
  +--------------------------------------------------------------------+
  |  WxUIState : folders, folder identities (parallel vec, D-25),       |
  |              messages: Vec<MessageItem>  |  conversations: Vec<..>  |
  +--------------------------------------------------------------------+
        |                                              ^
        | UIUpdate::FoldersLoaded / MessagesLoaded      | conversation rows
        v                                              |
  +--------------------------------------------------------------------+
  |  APPLICATION                                                       |
  |                                                                    |
  |  local_folders::is_local  --decides--> local path | server path     |
  |         |                                    |                      |
  |         |                                    v                      |
  |         |                          MailController -> Allowed::mail  |
  |         |                                    |                      |
  |  local_folders::deleting (D-33)              |                      |
  |         |                                    |                      |
  |  threading::thread_messages  <-- MUST become the one producer       |
  |         |    (batch = account, not folder page)                     |
  |  mail_sync: parent_id (D-22), folder-gone detection (D-27)          |
  |  migration (D-19): move messages, count, report, then remove        |
  +--------------------------------------------------------------------+
        |                                              |
        v                                              v
  +----------------------------+        +------------------------------+
  |  DATA (SQLite)             |        |  SERVICE / IMAP              |
  |  folders(account_id, path) |        |  may_i -> outward::permitted |
  |    + parent_id  [new]      |        |  mailbox_name::decode        |
  |    holds_all_mail (real)   |        |  mailbox_name::encode  [new] |
  |  messages.thread_id        |        |  create / rename / delete    |
  |    (COLUMN EXISTS, NO      |        |    [new wrappers over        |
  |     WRITER TODAY)          |        |     async-imap methods]      |
  |  + idx_messages_thread     |        |  list_folders (delimiter is  |
  |  + idx_messages_message_id |        |    parsed then discarded)    |
  +----------------------------+        +------------------------------+
                                                       |
                                                       v
                                            loopback test server
                                          (a_server_that_can / _refuses)
```

Trace the primary case, creating a folder on an IMAP account: keypress reaches the tree, the command asks `local_folders::is_local` which side it is on, the server side goes through `MailController` to `Allowed::mail`, `imap.rs` calls `may_i`, `mailbox_name::encode` turns the typed name into wire form, `session.create` sends it, the answer maps through `protocol_error`, a re-list writes the new row with its `parent_id`, and `UIUpdate::FoldersLoaded` rebuilds the tree and puts the cursor back.

### Pattern 1: A new IMAP verb

**What:** Every server-writing method in `imap.rs` has the same four parts: the gate, the timeout, the library call, the error map.
**When to use:** All three new verbs.
**Example**, the shape to copy [VERIFIED: src/service/protocols/imap.rs:840-865, `set_subscribed`]:

```rust
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
        // ... unsubscribe
    };
    outcome.map_err(protocol_error("Could not change the folder subscription"))
}
```

`COMMAND_TIMEOUT` is `Duration::from_secs(120)` [VERIFIED: src/service/protocols/imap.rs:81]. `with_timeout` names the step in its failure so the message is actionable [VERIFIED: src/service/protocols/imap.rs:1846-1852].

### Pattern 2: Testing a protocol verb against the loopback server

**What:** A scripted line-by-line server, with a transcript that assertions read.
**When to use:** Every one of the three new verbs, in both the accepted and the refused direction.

The harness is `crate::common::answering::{Conversation, Turn, conversing, LONG_ENOUGH}` [VERIFIED: src/common/answering.rs:263, :330, :368], and the IMAP wrapper is `against_a_server_that_answers` [VERIFIED: src/service/protocols/imap.rs:2394]. Its two entry points, verbatim from their doc comments:

- `a_server_that_can(capabilities: &'static str) -> Conversation` — "A mail server that says it can do exactly these things."
- `a_server_that_refuses(capabilities, refusing) -> Conversation` — "`refusing` is matched against the whole line without case, so `\"UID STORE\"` turns down the flag and leaves the copy alone."

Assertions read the transcript with `server.was_told(needle)` and `server.when_told(needle) -> Option<usize>` [VERIFIED: src/common/answering.rs:399, :408]. `when_told` returns a position, and its doc comment gives the reason: "Position rather than presence, because most of the questions here are about order".

**There are two harnesses, with opposite defaults, and the plan should pick deliberately.**

The shared script behind `a_server_that_can` lists its verbs, verbatim [VERIFIED: src/service/protocols/imap.rs:2449-2451]:

```rust
"UID" | "STORE" | "COPY" | "MOVE" | "EXPUNGE" | "NOOP" | "CLOSE" | "SUBSCRIBE"
| "UNSUBSCRIBE" => Turn::Say(format!("{tag} OK done\r\n")),
```

`CREATE`, `RENAME` and `DELETE` are absent, so under that harness today they fall to a catch-all that refuses, verbatim [VERIFIED: src/service/protocols/imap.rs:2452-2456]:

```rust
// Anything unrecognised is refused rather than ignored, so a
// script that has fallen behind the client fails the test in
// the moment instead of leaving it to wait out two minutes,
// which reads as a slow machine.
_ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
```

The second harness is `a_server_answering`, a private helper in the same module that takes a per-test closure and falls back permissively [VERIFIED: src/service/protocols/imap.rs:3135-3154], verbatim:

```rust
async fn a_server_answering(
    answer: impl Fn(&str, &str) -> Option<Turn> + Send + Sync + 'static,
) -> Conversation {
    conversing("* OK loopback ready\r\n", move |line| {
        let tag = line.split_whitespace().next().unwrap_or("*").to_string();
        let said = line.to_uppercase();
        if let Some(turn) = answer(&said, &tag) {
            return turn;
        }
        match said.split_whitespace().nth(1).unwrap_or_default() {
            "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
            "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
            "SELECT" | "EXAMINE" => Turn::Say(format!(
                "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
            )),
            _ => Turn::Say(format!("{tag} OK done\r\n")),
        }
    })
    .await
}
```

Its closure matches with `said.starts_with_command("LIST")`, a trait method on `str` that skips the tag [VERIFIED: src/service/protocols/imap.rs:3664-3672], verbatim:

```rust
fn starts_with_command(&self, command: &str) -> bool {
    self.split_once(' ')
        .is_some_and(|(_, rest)| rest.starts_with(command))
}
```

Note `said` is already uppercased by the time the closure sees it, so the needle must be too.

**Which to use.** `a_server_answering` needs no change to shared state and answers `OK done` to a CREATE today, so a test for "the client sent the right bytes" can be written against it immediately. Use it for the three new verbs' happy paths. Use `a_server_that_can` only if a test needs a capability line, and in that case add the three verbs to its match arm as a prerequisite task, which changes only the test harness. Either way, the refusal tests go through `a_server_that_refuses`, which matches the refused command by substring regardless of the verb list.

**One assertion trap, already learned here once.** The transcript is searched by substring, so a needle for `RENAME` also matches nothing else but a needle for `DELETE "Work"` is fine while `SUBSCRIBE "Work"` matches `UNSUBSCRIBE "Work"`. The existing test writes the fix in a comment worth copying: use a leading space on the needle. Verbatim [VERIFIED: src/service/protocols/imap.rs:2545-2548]:

> "A leading space on the first needle, because the transcript is searched by substring and `SUBSCRIBE \"Work\"` also matches the line that says UNSUBSCRIBE. Without it this test would pass with only one of the two commands ever sent."

### Pattern 3: A rebuilt tree that keeps the cursor

**What:** The folder tree is deleted and rebuilt whole on every `FoldersLoaded`, and the cursor is put back afterwards.
**When to use:** Every tree change in this phase.

Today the rebuild appends label strings to a flat list under one root [VERIFIED: src/presentation/wx_app.rs:10527-10592]. The branch-plus-omit-when-empty convention D-17 and D-28 must follow is written twice in that block, verbatim:

> "Labels last and under a branch of their own, so arrowing through the folders somebody opens every day does not pass through a list of labels first. No branch at all when there are none, rather than an empty one to arrow into."

Cursor restoration runs on labels today [VERIFIED: src/presentation/wx_app.rs:9741-9744, `what_the_cursor_was_on` returns `tree.get_item_text(&item)`], and the comment that follows the rebuild names the exact D-25 failure, verbatim [VERIFIED: src/presentation/wx_app.rs:10578-10584]:

> "A saved search keeps its row through a rebuild even when its name has just changed. What is open is held as the row's path, which a rename does not touch, while `land_the_cursor` matches on the row's words, which a rename does. Without this, renaming a search takes the cursor to the top of the tree and reads out a different row."

### Anti-patterns to avoid

- **Keying the tree on `TreeCtrl` item custom data.** See Pitfall 2. Use a parallel vector instead.
- **Building an aggregate SQL expression by string surgery on `sort_expression()`.** Fixed strings per enum arm, in both methods, or D-02's two halves come apart.
- **Wiring the reply-prefix stripper into `thread_messages`.** `threading.rs` argues against subject matching at length. D-04 is a label rule.
- **Assuming `messages.thread_id` holds anything.** It does not. Any query written against it today returns NULL.
- **Making `Empty` or `Delete` all-or-nothing.** D-36 rules it out and the RFC gives no transaction to build it on.
- **Adding a setting to `AppConfig` without adding a control to `wx_settings.rs`.** The existing guard cannot see it. See Pitfall 5.

---

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Stripping `Re:` / `Fwd:` chains, localised | A regex or a prefix list | `mail_parser::parsers::fields::thread::thread_name` | Already a dependency, already resolved at 0.11.5, 19 reply and 22 forward prefixes across 17 languages, RFC 5256-shaped fixpoint handling `[blob]` and `[fwd: ]`. Nothing to maintain. |
| CREATE / RENAME / DELETE mailbox | Raw command strings via `run_command` | `Session::create` / `rename` / `delete` | They exist, they quote and validate the argument, and they check the tagged response. Writing them by hand loses the response check, which is the mistake `set_flag` documents at length. |
| Moving a folder subtree | A recursive walk renaming each child | One `RENAME` | The RFC says inferior names MUST be renamed too. A manual walk is slower, non-atomic, and can leave a half-moved tree. |
| Base64 with the modified alphabet | A hand-rolled encoder | The existing `MODIFIED_BASE64` engine in `mailbox_name.rs` | Already configured with `,` for 63, padding off, and trailing-bits tolerance, with the reasoning recorded. |
| Judging whether a folder name is usable | A new character check | `attachment_name::safe_file_name`, via `import_tree`'s wrapper | Its doc comment lists what one question covers: a path step, a device name like `NUL`, a trailing dot, a character Windows refuses, a right-to-left override, an over-long name. |
| Deciding what deleting a message means | A second answer for Empty | `local_folders::deleting(from, protocol, asked, allowed)` | D-33 says so, and the function already routes trash-versus-remove and the per-account permission. |
| Repainting a virtual list row | Rebuilding the list | `ListCtrl::refresh_item(i)` | Rebuilding re-announces every row, which THREAD-02 explicitly forbids. |

**Key insight:** three of the seven rows above are things this codebase already owns and one is a dependency it already ships. The genuinely new code in this phase is smaller than the decision list suggests: an encoder, a `parent_id`, a persisted thread identity, a conversation row type, and a lot of wiring.

---

## Runtime State Inventory

D-19 is a migration that rewrites `folder_id` on messages in the user's only copy of that mail, so this section is required. The canonical question: after every file in the repo is updated, what runtime systems still hold the old shape?

| Category | Items found | Action required |
|---|---|---|
| **Stored data** | `folders` rows for each account's local `Sent`, `Outbox`, `Drafts`, `Junk`, `Trash`, and every `messages.folder_id` pointing at them. `UNIQUE(account_id, path)` [VERIFIED: src/data/message_cache/mod.rs:1301] is what D-18's reserved account id keeps working. Also `messages.thread_id`, present and empty on every existing row. | Data migration (D-19), message by message, nothing removed until every message has landed, with a count reported. Separately, a backfill for whatever thread identity this phase persists, because existing rows have none. |
| **Live service config** | None on the server side: D-18, D-19, D-30 and D-14 are all explicitly local and never write to a server. Pinning "never passes through `Allowed`" per FOLDER-03. IMAP subscription state exists on the server but this phase does not touch it. | None. Verified by reading FOLDER-03's criteria and D-30. |
| **OS-registered state** | None found. This phase adds no scheduled task, no service registration, no shell integration. Verified by reading the decision list end to end: every persisted item is a database row or an `AppConfig` field. | None. |
| **Secrets and env vars** | None. Nothing in this phase reads or writes a credential. `CLAUDE.md` §Project rules requires nothing sensitive in `message_cache.db`, and folder names, pins and orderings are not sensitive. | None. |
| **Build artifacts / installed packages** | None. No new dependency, no changed package name, no renamed binary. `mail-parser` is already present and already resolved [VERIFIED: Cargo.lock]. | None. |
| **Config files already on disk** | Every user's settings file predates the five new `AppConfig` fields. | Each new field needs `#[serde(default)]` or `#[serde(default = "...")]`, which is the established pattern [VERIFIED: src/data/config.rs:25, :40, :49]. A field without one makes every existing settings file fail to parse, and `test_a_settings_file_written_before_directories_existed_still_reads` at src/data/config.rs:1245 exists because that has been a real failure. |

**The migration's own risk, stated plainly.** D-19 merges five folders per account into five shared ones. If two accounts each have a `Drafts` folder holding a message with the same `uid`, the shared folder's `UNIQUE(folder_id, uid)` constraint [VERIFIED: src/data/message_cache/mod.rs:1324] will reject the second insert. UIDs are per-mailbox in IMAP and per-download in POP, so a collision is not unlikely; it is expected. The migration must assign new local uids or key on something else, and it must count what it moved so D-19's "nothing removed until every message has landed" is checkable rather than asserted. This constraint is the single most likely way the migration loses mail, and it is not mentioned in CONTEXT.md.

---

## Common Pitfalls

### Pitfall 1: `messages.thread_id` looks like working infrastructure

**What goes wrong:** A task is planned as "query the existing thread_id across folders" and the executor finds the column, the sort expression, and a dozen references to `thread_id` in the tree, and writes the query. It returns nothing, or one giant NULL group.
**Why it happens:** The name appears in 40 places. Only one of them is in the data layer and it is the line creating the column. The in-memory `MessageItem.thread_id` carries the real value and shares the name.
**How to avoid:** Treat "persist a thread identity" as its own task, with its own tests, ordered before anything that reads one. Count writers and readers separately for any persisted value this phase relies on.
**Warning signs:** A plan task that says "read", "query" or "join on" `thread_id` without a preceding task that says "write" it.

### Pitfall 2: `TreeCtrl` custom data leaks and its cleanup does not clean up

**What goes wrong:** D-25's identity keying is built on `append_item_with_data` or `set_custom_data`. The tree is rebuilt on every sync, on a timer. Memory grows without bound for the life of the process.
**Why it happens:** `store_item_data` inserts into a process-global map, verbatim [VERIFIED: wxdragon-0.9.17/src/widgets/item_data.rs:21-25]:

```rust
pub fn store_item_data<T: Any + Send + Sync + 'static>(data: T) -> u64 {
    let id = NEXT_DATA_ID.fetch_add(1, Ordering::SeqCst);
    ITEM_DATA_REGISTRY.write().unwrap().insert(id, Arc::new(data));
    id
}
```

`delete_all_items` calls the raw FFI and removes nothing from that map [VERIFIED: wxdragon-0.9.17/src/widgets/treectrl.rs:967-974]. And `cleanup_all_custom_data`, which is meant to be the escape hatch, recurses into branches and **returns early on any item with no children without ever clearing that item's data** [VERIFIED: wxdragon-0.9.17/src/widgets/treectrl.rs:1163-1170 and :1232-1251, `clean_item_and_children` begins `if self.get_children_count(item, false) == 0 { return; }` and never calls `clear_custom_data`]. Leaf nodes, which is every folder row, are never cleared.

**How to avoid:** Do not put folder identity in tree item data. Build a `Vec<FolderIdentity>` during the rebuild, in the same order the tree is appended, and pair it with the tree by position. This is the pattern `collect_rows` already establishes, which walks the tree filling two parallel vectors [VERIFIED: src/presentation/wx_app.rs:10103-10118]. `land_the_cursor` and `select_row_named` both already consume that pair.
**Warning signs:** `append_item_with_data`, `set_custom_data`, or `get_custom_data` appearing in a plan task.

**Secondary note:** the trait's `impl Into<u64>` parameter is implemented by converting a `&TreeItemId` reference's address to `u64` [VERIFIED: wxdragon-0.9.17/src/widgets/treectrl.rs:195] and reinterpreting it as a pointer on the way back [VERIFIED: same file:1181-1220]. `set_custom_data_direct(&TreeItemId, T)` at line 1255 is the safe variant. Another reason to avoid the whole area.

### Pitfall 3: Picking the loopback harness that refuses the new verbs

**What goes wrong:** The first RED test for CREATE fails with `BAD unscripted` instead of the assertion, and the executor concludes the client is wrong.
**Why it happens:** There are two harnesses with opposite defaults. `a_server_that_can` refuses anything not in its verb list, deliberately, and CREATE, RENAME and DELETE are not in it. `a_server_answering` answers `OK done` to anything its closure does not handle.
**How to avoid:** Use `a_server_answering` for the new verbs. If a test genuinely needs `a_server_that_can`'s capability line, extend its match arm in a prerequisite task first; that changes only the test harness, so it is one of CLAUDE.md's genuine test-infrastructure exceptions rather than a behaviour change needing its own RED.
**Warning signs:** A test failure message containing `unscripted`.

### Pitfall 4: The reply-prefix stripper and the reply-prefix prepender disagree

**What goes wrong:** Subjects accumulate `Re: AW: Re: ...` because `format_reply_subject` only recognises the exact ASCII string `"Re: "`.
**Why it happens:** Two functions answer one question and only one of them was updated.
**How to avoid:** Rewrite `format_reply_subject` and `format_forward_subject` in terms of the stripper: prepend only when `thread_name(subject) == subject`. One answer, one place.
**Warning signs:** A plan that adds the stripper and does not name `wx_compose.rs`.

### Pitfall 5: A setting is stored, read, and never offered

**What goes wrong:** Exactly FEEDBACK-01, which CONTEXT.md says "has already happened here once".
**Why it happens:** The existing guard `test_every_setting_somebody_can_change_is_read_by_something` walks every shipping source file *except* `wx_settings.rs`, verbatim [VERIFIED: src/data/config.rs:1291-1295]:

```rust
// `config.rs` defines and stores a setting and
// `wx_settings.rs` offers it. Neither is anybody acting
// on the answer.
if !shown.ends_with("data/config.rs") && !shown.ends_with("wx_settings.rs") {
```

So it catches "offered and ignored" and is structurally blind to "stored and never offered". The five settings this phase adds are exactly the second shape.
**How to avoid:** Add the mirror test in the same module, reusing `stored_setting_names`, asserting every field name appears in `wx_settings.rs`. It is roughly fifteen lines and it converts a rule in a document into a check.
**Warning signs:** A plan task that adds an `AppConfig` field with no paired task naming `wx_settings.rs`.

### Pitfall 6: An incremental thread join disagrees with the batch recompute

**What goes wrong:** A message arrives, joins a thread, and on next folder open the whole conversation has a different id, so per-folder stored state keyed on it (D-09's view setting, any collapsed state) is lost.
**Why it happens:** The thread id is the least Message-ID in the conversation. Adding a message can lower it.
**How to avoid:** One function computes the id in both paths. The incremental path recomputes the affected component rather than adopting a found id. THREAD-02's own criterion, "the merge case has a test that fails if the two trees are left separate", is the right test; add a second that fails if the incremental id and the batch id differ for the same set.
**Warning signs:** A plan task phrased as "look up the references and adopt that thread's id".

### Pitfall 7: A new UI test needs its own process

**What goes wrong:** A second `#[test]` building real dialogs in the same file hangs or crashes.
**Why it happens:** Stated in the existing test, verbatim [VERIFIED: tests/checkbox_labels.rs:21-23]:

> "One `#[test]` function building real dialogs, for the reason `tests/theme_reach.rs` gives: wxWidgets supports one application per process and `cargo test` runs each file under `tests/` as its own process."

**How to avoid:** One dialog-building test per file under `tests/`. Assertions accumulate inside it rather than splitting into more test functions.

### Pitfall 8: Alt+Shift trips the Windows input-language hotkey

**What goes wrong:** D-14's and D-31's Alt+Shift+Up/Down changes the keyboard layout on a machine with more than one installed, when the modifiers are released after the arrow.
**Why it happens:** Bare Alt+Shift is the Windows layout switch. CONTEXT.md names this and accepts it.
**How to avoid:** Nothing in the application can suppress it. Document it in `docs/KEYBOARD_SHORTCUTS.md` in the same commit, per CLAUDE.md, so a user who hits it knows what happened.

---

## Code Examples

### A new mailbox verb, following the house pattern

```rust
// Shape verified against src/service/protocols/imap.rs:840-865 (set_subscribed),
// :1067 (may_i), :1837 (protocol_error), :81 (COMMAND_TIMEOUT).
// The encode call is the piece that does not exist yet.
pub async fn create_folder(&mut self, path: &str) -> Result<()> {
    self.may_i("create a folder on the server")?;
    let on_the_wire = mailbox_name::encode(path);
    with_timeout(
        COMMAND_TIMEOUT,
        self.session.create(&on_the_wire),
        "creating a folder",
    )
    .await?
    .map_err(protocol_error("Could not create the folder"))
}
```

### Testing it against the loopback server, both directions

```rust
// Shape verified against src/service/protocols/imap.rs:2508-2560 and :3188-3230.
// a_server_answering is used rather than a_server_that_can, because the latter
// refuses any verb not in its list and CREATE is not in it (Pitfall 3).
#[tokio::test]
async fn test_creating_a_folder_names_it_on_the_wire() {
    let server = a_server_answering(|_said, _tag| None).await;
    let mut session = signed_in_to(&server).await;

    waiting_for(session.create_folder("Work/2026"), "the folder")
        .await
        .expect("the folder to be created");

    let transcript = server.transcript().await;
    assert!(server.was_told("CREATE \"Work/2026\"").await, "{transcript:?}");
}

#[tokio::test]
async fn test_a_server_refusing_the_create_is_not_reported_as_success() {
    let server = a_server_that_refuses("", "CREATE").await;
    let mut session = signed_in_to(&server).await;

    let said = the_failure(waiting_for(session.create_folder("Work"), "the refusal").await);
    assert!(said.contains("Could not create the folder"), "{said}");
}
```

### The reply-prefix strip, and the prepender rewritten against it

```rust
use mail_parser::parsers::fields::thread::thread_name;

/// The conversation's own name: the oldest present message's subject with
/// every reply and forward prefix chain taken off (D-04).
pub fn conversation_label(oldest_subject: &str) -> String {
    let base = thread_name(oldest_subject).trim();
    if base.is_empty() {
        "No subject".to_string()
    } else {
        base.to_string()
    }
}

/// Replaces src/presentation/wx_compose.rs:161-167, which recognised only the
/// exact ASCII "Re: " and so grew a second chain on "AW: Angebot".
fn format_reply_subject(subject: &str) -> String {
    if thread_name(subject) == subject.trim() {
        format!("Re: {subject}")
    } else {
        subject.to_string()
    }
}
```

### The parallel-identity pattern for D-25

```rust
// The pattern collect_rows already uses (src/presentation/wx_app.rs:10103-10118):
// walk the tree filling vectors that stay in step by position, rather than
// hanging data off the control. Avoids the leak in Pitfall 2.
struct TreeIdentities {
    rows: Vec<FolderIdentity>,
}

enum FolderIdentity {
    Account { account_id: String },
    Folder { account_id: String, path: String },
    ImportedArchive { archive_id: String },
    Group(GroupKind), // Favourites, ALL_INBOXES, On this computer, Labels
}
```

---

## State of the Art

| Old approach | Current approach | When changed | Impact here |
|---|---|---|---|
| RFC 3501 IMAP4rev1 | RFC 9051 IMAP4rev2 | 2021 | Tightens RENAME's child handling from "will" to MUST, and adds the `HASCHILDREN` response code to DELETE's refusal. `async-imap 0.11.3`'s doc comments still cite RFC 3501 throughout, which is correct but dated; the behaviour is the same. |
| Modified UTF-7 mailbox names | RFC 6855 `ENABLE UTF8=ACCEPT` | 2013 | Not usable as the only path. `mailbox_name.rs`'s own comment says why, verbatim: "plenty of servers still do not offer it, so decoding stays necessary." Encoding is likewise still necessary. |
| Subject-based threading | References/In-Reply-To (JWZ) | Long settled | `threading.rs` already implements the modern approach and argues against the old one. D-04 does not change that. |
| `LSUB` for subscriptions | `LIST` with selection options (RFC 5258) | 2008 | `imap.rs` uses `LSUB "" "*"` [VERIFIED: src/service/protocols/imap.rs:817]. Out of scope for this phase but worth noting if subscription later backs FOLDER-03. |

**Deprecated or superseded:**

- Nothing this phase adds is on a deprecation path. All three crates are at their locked, current-for-this-project versions.

---

## Package Legitimacy Audit

**This phase adds no new dependency.** Every crate it needs is already in `Cargo.toml` and already resolved in `Cargo.lock`. The three that matter were checked anyway.

| Package | Registry | Age | Downloads | Source repo | Verdict | Disposition |
|---|---|---|---|---|---|---|
| `mail-parser` 0.11.5 | crates.io | first published 2021-11-01 | 100,615/wk | github.com/stalwartlabs/mail-parser | OK | Already a dependency. Approved for the D-04 use. |
| `async-imap` 0.11.3 | crates.io | first published 2019-11-11 | 52,041/wk | github.com/async-email/async-imap | OK | Already a dependency. No change. |
| `wxdragon` 0.9.17 | crates.io | first published 2025-05-08 | 265/wk | github.com/AllenDang/wxDragon | SUS (new, low downloads) | Already the shipped UI framework, pinned with `=0.9.17`. D-01 locks the control. The verdict is informational only; no install decision is being taken in this phase. |

[VERIFIED: `gsd-tools query package-legitimacy check --ecosystem crates mail-parser async-imap wxdragon`, run this session]

**Packages removed due to SLOP verdict:** none.
**Packages flagged as suspicious:** `wxdragon`, already in use, not being installed by this phase. No `checkpoint:human-verify` is needed because no install occurs. The relevant risk is not supply chain but maturity, and Pitfall 2 documents one concrete defect found in it this session.

---

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|---|---|---|---|---|
| Rust toolchain and `cargo` | All work | Yes | Project builds today | — |
| `async-imap` crate source | Q1, Q2, Q3 | Yes, vendored | 0.11.3 | — |
| `wxdragon` crate source | Q5, Pitfall 2 | Yes, vendored | 0.9.17 | — |
| `mail-parser` crate source | Q6 | Yes, vendored | 0.11.5 | — |
| Loopback TCP for the test server | Every protocol test | Yes, in-process, already used by the existing suite | — | — |
| A live IMAP account | Nothing in this phase | **No, and out of scope** | — | Loopback server for the wire, parsing tests for the shapes |
| NVDA or Narrator | Nothing this agent or the executor does | Not applicable | — | The user runs screen reader testing and decides when |

**Missing dependencies with no fallback:** none.

**What a loopback server can and cannot prove.** It can prove the exact bytes the client sends, in order, which is what `was_told` and `when_told` are for. It can prove that a `NO` response reaches the user as a refusal rather than as a success, which is the `a_server_that_refuses` path and the failure shape this project has been bitten by. It can prove the client's behaviour on a hang-up mid-command, because `Turn::HangUp` exists.

It cannot prove that a real server accepts the modified UTF-7 the encoder produces, because the loopback server answers `OK` to anything scripted regardless of the argument. It cannot prove what a real server does to child mailboxes on RENAME, because the script has no mailbox store. It cannot prove that a `\Noselect` parent behaves as the RFC says. Those three are RFC-conformance claims, and the honest position is that the client is written to the RFC and the RFC's guarantees are cited, not that the behaviour has been observed. Nothing in this milestone claims otherwise.

---

## Validation Architecture

### Test framework

| Property | Value |
|---|---|
| Framework | Rust built-in `#[test]` / `#[tokio::test]`, with `tokio-test 0.4` in dev-dependencies [VERIFIED: Cargo.toml:214] |
| Config file | None. `cargo test` reads `Cargo.toml`. Guard records live in `guards/guards.toml` [VERIFIED: 9078 lines] |
| Quick run command | `cargo test --lib <module_path>` for one module, e.g. `cargo test --lib threading::` |
| Full suite command | `cargo test` |
| Scale | Over 5,400 `fn test_` declarations across `src/` and `tests/` [VERIFIED: `grep -rc "fn test_"` summed over src/ and tests/; a count of declarations, not of assertions] |
| Integration tests | 20 files under `tests/` [VERIFIED: `ls tests/`], each run as its own process, which matters for wx |

### Phase requirements to test map

| Req | Behaviour | Test type | Automated command | File exists? |
|---|---|---|---|---|
| FOLDER-01 | CREATE names the folder on the wire | unit, loopback | `cargo test --lib against_a_server_that_answers::` | Yes, module exists; the three verbs' tests are new |
| FOLDER-01 | A server `NO` is reported as a refusal, not success | unit, loopback | same | Yes, `a_server_that_refuses` and `the_failure` exist |
| FOLDER-01 | Server folder refused by the gate with a reason when `Allowed::mail` is off | unit | `cargo test --lib outward::` | Yes, `permitted` and `was_refused_by_the_gate` have tests |
| FOLDER-01 | A local folder is not gated | unit | `cargo test --lib local_folders::` | Yes |
| FOLDER-01 | Empty routes through `local_folders::deleting` (D-33) | unit | `cargo test --lib local_folders::` | Yes, the module has tests |
| FOLDER-01 | Empty stops and reports where it got to (D-36) | unit | new module | Wave 0 |
| FOLDER-02 | `Archive/2026` reads as `2026` under `Archive` | unit | `cargo test --lib` on the new tree-shape module | Wave 0 |
| FOLDER-02 | Collapsed state survives a restart, keyed by identity not label (D-25) | unit | same | Wave 0 |
| FOLDER-02 | Modified UTF-7 round trip | unit | `cargo test --lib mailbox_name::` | Module exists, has decode tests; encode tests are new |
| FOLDER-03 | Pin never writes to a server | unit | `cargo test --lib` on the favourites module | Wave 0 |
| FOLDER-03 | A pin survives a rename, and a deletion takes it (D-32) | unit | same | Wave 0 |
| THREAD-01 | A conversation row announces subject, count and unread | unit | `cargo test --lib message_rows::` | Module exists |
| THREAD-01 | Every column answers about the conversation (D-02) | unit | `cargo test --lib message_columns::` | Module exists |
| THREAD-01 | `ID_THREAD_VIEW` is raised by something | integration | `cargo test --test wired` | Yes, `tests/wired.rs` exists |
| THREAD-02 | A late message merges two trees | unit | `cargo test --lib threading::` | Yes, module exists; the merge test is named as the case worth testing |
| THREAD-02 | Incremental and batch agree on the id | unit | same | Wave 0 |
| Settings | Five new settings are read by something | unit | `cargo test --lib config::` | Yes, `test_every_setting_somebody_can_change_is_read_by_something` exists |
| Settings | Five new settings are offered by a screen | unit | same module, new test | **Wave 0. The existing guard is blind to this.** |
| Settings | New check boxes carry their own label on both channels | integration | `cargo test --test checkbox_labels` | Yes, file exists |
| Settings | An older settings file still parses | unit | `cargo test --lib config::` | Yes, pattern exists at src/data/config.rs:1245 |

### Sampling rate

- **Per task commit:** the module's own tests, e.g. `cargo test --lib threading::`.
- **Per wave merge:** `cargo test`.
- **Phase gate:** full suite green, plus `scripts/guards.sh` for any guard record this phase adds or invalidates, before `/gsd-verify-work`.

### Wave 0 gaps

- [ ] Decide which loopback harness the FOLDER-01 protocol tests use. `a_server_answering` needs no change; `a_server_that_can` needs `CREATE`, `RENAME` and `DELETE` added to its match arm first. Only the second is a Wave 0 task, and only if a capability line is needed.
- [ ] `mailbox_name::encode` plus a round-trip guard — covers FOLDER-01, FOLDER-02.
- [ ] A test module for the folder tree's shape and identity keying — covers FOLDER-02, D-25.
- [ ] A test module for favourites — covers FOLDER-03.
- [ ] The mirror settings guard in `src/data/config.rs`'s `every_setting_is_acted_on` module — covers the five settings and FEEDBACK-01.
- [ ] A test that the incremental and batch thread ids agree — covers THREAD-02, Pitfall 6.
- [ ] A migration test module for D-19, including the `UNIQUE(folder_id, uid)` collision case.
- [ ] Guard records in `guards/guards.toml` for each of the above, measured by hand per the file's own rule: "take the break by hand first and write down what really went red, all of it. Do not write down the tests you expected."

No framework installation is needed.

---

## Security Domain

`security_enforcement` is not set to `false` anywhere in `.planning/config.json` [VERIFIED: .planning/config.json, two keys only: `workflow.tdd_mode` and `model_profile`], so this section applies.

### Applicable ASVS categories

| ASVS category | Applies | Standard control here |
|---|---|---|
| V2 Authentication | No | This phase opens no new sign-in path. Sessions are established by existing code. |
| V3 Session management | Partly | The IMAP session's `may_change` flag is the write permission and defaults to false, verbatim from the code comment [VERIFIED: src/service/protocols/imap.rs:582-585]: "Reading only until somebody says otherwise. A session opened without anybody thinking about it should be the one that cannot remove somebody's mail." Every new verb must call `may_i` first. |
| V4 Access control | Yes | `Allowed::mail` gates server writes and is off for a new install [VERIFIED: src/application/allowed.rs:38-56, `NOTHING` has `mail: false`]. Three new verbs, three new gate calls. A verb without one is a privilege escalation in a client whose whole safety story is that gate. |
| V5 Input validation | Yes | Folder names are user input reaching a protocol command. See below. |
| V6 Cryptography | No | Nothing in this phase encrypts, signs or hashes. |
| V7 Error handling and logging | Yes | `redact_provider_message` strips credential-shaped text from server messages before they reach the log [VERIFIED: used in both `protocol_error` and the sign-in refusal path]. New verbs inherit it by using `protocol_error`. CLAUDE.md: never log a token, password or message body. |

### Known threat patterns for this stack

| Pattern | STRIDE | Standard mitigation, and its state here |
|---|---|---|
| IMAP command injection through a folder name | Tampering | `validate_str` quotes the name and rejects `\r` and `\n` [VERIFIED: async-imap-0.11.3/src/client.rs:1510-1519]. This is genuinely sufficient for CRLF injection: a newline is the only way to start a second command, and it is refused. Do not bypass it with `run_command`. |
| SQL injection through a sort or filter | Tampering | Already mitigated by design: `sort_expression` returns fixed strings matched off an enum, and its comment says why, verbatim: "never built from anything a user typed, because the result is interpolated into a query." The aggregate sibling must keep the same property. |
| Path traversal through a local folder name (D-20, D-21, D-23) | Tampering, elevation | `is_a_name_that_can_be_used` wraps `attachment_name::safe_file_name` [VERIFIED: src/application/import_tree.rs:262-264], verbatim: `!part.is_empty() && crate::service::attachment_name::safe_file_name(part) == part`. Its doc comment lists the cases it covers: "a step out of a folder, a device like `NUL`, a trailing dot the filesystem would strip after this had finished checking, a character Windows will not take, a name written backwards by an override, and a name too long to write". **It is currently private** and D-23 needs it; make it `pub(crate)` rather than writing a second answer. |
| Destructive operation without confirmation | Repudiation, denial | D-33 through D-38 specify the confirmations. D-37's rule that the count comes from the cache and not from a server round trip in front of a cancellable dialog is also a denial-of-service mitigation: it means a dialog cannot hang on the network. |
| A refusal reported as success | Repudiation | The failure shape this codebase has been bitten by repeatedly, documented at length on `set_flag` [VERIFIED: src/service/protocols/imap.rs:1073-1080]: "a refusal reads exactly like a change that worked... they would find out days later from another device." Every new verb uses `run_command_and_check_ok` semantics through the library methods, which do check the tagged response. Each verb needs an `a_server_that_refuses` test. |
| Unbounded memory growth from a UI rebuild | Denial of service | Pitfall 2. Avoid tree item custom data. |

---

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|---|---|---|
| A1 | Real IMAP servers vary in RENAME behaviour for nested mailboxes beyond what the RFC mandates | Q2 | Low. The recommendation is already to treat `NO` as expected and re-list rather than assume. If servers are in fact uniform, nothing is lost. |
| A2 | Gmail's IMAP label semantics make its RENAME behaviour for a nested label its own case | Q2 | Low to medium. Affects only how a `NO` from Gmail is explained to the user, not whether the code is correct. |
| A3 | The `UNIQUE(folder_id, uid)` collision during the D-19 merge is likely rather than theoretical | Runtime State Inventory | Medium. The constraint is verified; the likelihood is inferred from UIDs being per-mailbox. Even if collisions are rare, the migration must handle one, so the plan is the same either way. |
| A4 | Splitting the account-wide thread batch is the right fix for D-08 rather than changing the id rule | Q4 | Medium. Both fixes work. The choice affects how much of `threading.rs` changes and should be made explicitly in the plan, not inferred. |
| A5 | No new dependency is needed for the modified UTF-7 encoder | The encoder section | Low. The base64 engine and alphabet are already configured in the same file; only the UTF-16BE conversion and the run-splitting are new, and both are short. |

Everything else in this document is tagged `[VERIFIED: …]` with a path and line range and a verbatim quote, or `[CITED: …]` with the source it was read from.

---

## Open Questions

1. **Which fix does D-08 take: a wider batch, or a batch-independent id?**
   - What we know: the id is the least Message-ID in the batch, so it is batch-dependent, and D-08 needs it stable across folders.
   - What is unclear: whether `thread_messages` should be called with the account's messages (simple, but the batch grows to the whole account) or whether the id rule should change to something batch-independent (a schema-visible change to a value that has never been persisted, so cheap now and expensive later).
   - Recommendation: raise it as a decision in planning rather than letting an executor choose. The wider batch is the smaller change and matches what the module already does; the batch-independent id is the one that scales. Note that `messages.thread_id` has never held a value, so there is no stored data to migrate either way, which makes now the cheapest moment this choice will ever be available.

2. **Does the phase fix the Thread column's sort, or leave it?**
   - What we know: `ORDER BY m.thread_id` runs against a column that is NULL on every row.
   - What is unclear: whether this counts as in scope. D-02 requires the Thread column to sort meaningfully under thread view, which forces the issue there; the flat view's Thread sort is a pre-existing defect.
   - Recommendation: fix it, because persisting a thread identity makes it correct for free, and leaving a known-broken sort beside a newly-correct one is the disagreement this project keeps finding.

3. **Where do the five settings live in the notebook?**
   - What we know: `wx_settings.rs` has seven pages: General, Compose, Reading, Permissions, Calendar && PIM, Feedback, Advanced [VERIFIED: src/presentation/wx_settings.rs:200-250].
   - What is unclear: three of the five are about reading mail (conversation reach, delete scope, unread announcement) and two are about destructive folder operations (Empty and Mark Read reaching subfolders). Reading and Permissions are the obvious homes, but splitting five related settings across two pages costs a user two places to look.
   - Recommendation: planner decides and states it. This is a plain UI placement question, not a research gap.

4. **Does `Move To` reuse the message-level destination picker?**
   - CONTEXT.md marks this as Claude's discretion. Nothing found in this session argues either way strongly; a folder destination list and a message destination list have the same shape and different contents.
   - Recommendation: reuse if the existing picker takes its list as a parameter; otherwise write a folder-specific one rather than generalising a working control mid-phase.

---

## Sources

### Primary (HIGH confidence, read this session)

- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-imap-0.11.3/src/client.rs` — `create` (578), `delete` (604), `rename` (636), `subscribe` (655), `list` (1003), `validate_str` (1510), `quote!` (25)
- `async-imap-0.11.3/src/error.rs` — the `Error` enum, lines 11-36
- `async-imap-0.11.3/src/types/name.rs` — `Name::delimiter()` returning `Option<&str>` per mailbox
- `wxdragon-0.9.17/src/widgets/list_ctrl.rs` — 48 public methods enumerated; `set_item_count` (771), `refresh_item` (781), `refresh_items` (791), `set_virtual_text_callback` (806)
- `wxdragon-0.9.17/src/widgets/treectrl.rs` — `append_item_with_data` (403), `delete_all_items` (967), `cleanup_all_custom_data` (1163), `clean_item_and_children` (1232)
- `wxdragon-0.9.17/src/widgets/item_data.rs` — `store_item_data` (21), the global `ITEM_DATA_REGISTRY`
- `mail-parser-0.11.5/src/parsers/fields/thread.rs` — `thread_name`, `is_re_prefix`, `is_fwd_prefix`
- Wixen Mail tree, all paths relative to the repository root: `src/service/protocols/imap.rs`, `src/service/protocols/imap/mailbox_name.rs`, `src/service/outward.rs`, `src/application/threading.rs`, `src/application/local_folders.rs`, `src/application/allowed.rs`, `src/application/import_tree.rs`, `src/data/message_cache/mod.rs`, `src/data/message_cache/messages.rs`, `src/data/message_cache/folders.rs`, `src/data/config.rs`, `src/presentation/wx_app.rs`, `src/presentation/message_columns.rs`, `src/presentation/message_rows.rs`, `src/presentation/wx_compose.rs`, `src/presentation/wx_settings.rs`, `src/common/answering.rs`, `tests/wired.rs`, `tests/checkbox_labels.rs`, `guards/guards.toml`, `CLAUDE.md`, `Cargo.toml`, `Cargo.lock`, `.planning/config.json`
- `gsd-tools query package-legitimacy check --ecosystem crates` for the three crates

### Secondary (MEDIUM confidence)

- RFC 9051 §6.3.5 (DELETE) and §6.3.6 (RENAME), fetched from rfc-editor.org
- RFC 5256, base subject extraction and the `subj-refwd` ABNF, fetched from rfc-editor.org

### Tertiary (LOW confidence, flagged in the Assumptions Log)

- Web search on localised reply prefixes: office-watch.com, help.gnome.org (Evolution), bugzilla.mozilla.org 634896 and 29179. Used only to confirm that the localised set is de-facto rather than standardised, which the `mail-parser` source then settled authoritatively.

---

## Metadata

**Confidence breakdown:**

| Area | Level | Reason |
|---|---|---|
| async-imap surface (Q1) | HIGH | Signatures, bodies and doc comments read from the vendored source at the locked version |
| RENAME and DELETE semantics (Q2, Q3) | HIGH for the RFC contract, LOW for per-server variance | RFC text quoted verbatim; server variance not verified against any authoritative source and flagged A1, A2 |
| Schema and indexes (Q4) | HIGH | Every `CREATE TABLE`, `ensure_column_exists` and `CREATE INDEX` enumerated by tree-wide search and quoted verbatim; the absence of a `thread_id` index and of a `thread_id` writer proved by search, not assumed |
| Virtual list and wxdragon (Q5) | HIGH | Crate methods enumerated from source; the custom-data leak traced through three functions |
| Reply-prefix stripping (Q6) | HIGH | RFC ABNF quoted; the crate implementation read and its prefix lists quoted verbatim; the export path confirmed module by module |
| Pitfalls | HIGH except Pitfall 8 | Each traced to a specific line or a specific crate defect. Pitfall 8 is CONTEXT.md's own note about Windows behaviour, restated |
| Migration risk (D-19) | MEDIUM | The `UNIQUE(folder_id, uid)` constraint is verified; the likelihood of collision is inferred |

**Research date:** 2026-08-29
**Valid until:** 2026-09-28. The crate findings hold as long as the pins hold, and all three are exact or effectively exact. The codebase findings go stale the moment this phase starts changing the files they describe, which is the point of doing them now.

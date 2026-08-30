# Phase 1: Folders and conversations - Context

**Gathered:** 2026-08-29
**Status:** Ready for planning

<domain>
## Phase Boundary

The shape of the mailbox. Folders a user can make, rename, move, empty and mark
read; nested the way the server nests them; favourites at the top; and the
message list collapsed to one row per conversation.

Requirements: FOLDER-01, FOLDER-02, FOLDER-03, THREAD-01, THREAD-02.

**The discussion widened this phase, and the planner needs to know by how
much.** The roadmap describes nesting a flat tree. What came out of the
discussion is a restructured folder tree: one branch per account, a shared "On
this computer" group whose folders no longer belong to any account, a migration
that moves existing mail between rows, and five new settings. That is more than
the roadmap's five success criteria describe. Nothing here is outside the
phase's domain, and the phase is larger than it looked.

</domain>

<decisions>
## Implementation Decisions

### Conversations in the message list

- **D-01:** A conversation row never expands in place. `Enter` opens the
  existing `wx_thread_view` `TreeCtrl`, which announces level natively. The
  message list stays a virtual `ListCtrl`, so UI Automation keeps the real set
  size, and `wx_thread_view`'s own written rationale for keeping the list flat
  stands.

- **D-02:** A row describes the **whole conversation**, not its newest message.
  One rule per column family, stated once and used by both the displayed value
  and the SQL `ORDER BY`, because `MessageColumn` supplies both from the same
  enum today and they must not come apart:
  - Dates (`Received`, `Sent`) and `Snippet`: the newest message's.
  - State (`Attachment`, `Flagged`, `Answered`, `Draft`): true if any message is.
  - `Safety`: the worst across the conversation.
  - `Size`: the sum.
  - `Correspondent`: the distinct senders.
  - `Subject`: the conversation's, per D-04.
  - `Thread`: the counts, per D-03.
  — **Reversibility:** costly — the rule is read by both the display path and
  every column's sort expression, so changing it later means changing both in
  step or reintroducing the disagreement this decision exists to prevent.

- **D-03:** `MessageColumn::Thread`, which today holds a thread id no person
  wants read aloud, becomes "5 messages, 2 unread". The row is then assembled
  from the visible columns like every other row, which is what THREAD-01 asks
  for.

- **D-04:** The conversation is named by the **oldest message present**, with
  `Re:` and `Fwd:` prefix chains stripped. Nothing strips those today, so this
  is new code with its own tests, including the localised prefixes other clients
  emit. The label can change when `Get Older Messages` brings in an earlier
  message; the planner must build for that rather than assume the root is
  present.

- **D-05:** A lone message's row shows no counts. The `Thread` column appears
  only when the folder holds at least one conversation of more than one message.

- **D-06:** A hand choice in `View > Columns` beats the adaptive rule in D-05,
  permanently. One stored value per folder must distinguish "never chosen" from
  "chosen off", or the setting reverts under the user.

- **D-07:** An action on a collapsed conversation row acts on the **whole
  conversation**, with the count named before it happens: "Delete 5 messages in
  Quarterly report?". A reversal puts the whole conversation back, not part of
  it.

- **D-08:** A conversation spans the **account**, not the folder. Its row
  appears in every folder it touches, showing the same count wherever the user
  is standing. Folders whose `holds_all_mail` server fact is true (Gmail's All
  Mail) are excluded from the count, or every Gmail conversation doubles.
  This was flagged during discussion as larger than a clarification and chosen
  deliberately. — **Reversibility:** costly — it needs a cross-folder query path
  that does not exist, and every folder-scoped list query and count becomes
  thread-aware.

- **D-09:** Thread view is stored **per folder**, alongside whatever FOLDER-02
  stores for collapsed state, so both restore together. A folder never set is
  flat.

- **D-10:** An `Apply View To Other Folders` command offers three scopes: this
  folder's subtree, this account, or everywhere. Each is a plain sentence naming
  what it will change and how many folders that is, spoken before it happens,
  and it confirms, because it overwrites settings the user may have set by hand.

- **D-11:** A selection survives a view switch **both ways**. Switching to
  threads selects every conversation containing a selected message; switching
  back selects exactly the messages that were selected before, not everything in
  those conversations. The original set is held across the switch.

- **D-12:** The sort column and direction survive a view switch unchanged,
  applied through D-02's rule. The header announces the same thing before and
  after.

### The folder tree

- **D-13:** **One branch per account.** `ALL_INBOXES` keeps its place at the
  root. Folders inside a branch keep their existing `tree_position` order.
  — **Reversibility:** costly — every path that finds a row in the tree, and the
  cursor-restoring code in `UIUpdate::FoldersLoaded`, is written against a flat
  list today.

- **D-14:** Accounts appear in the order they were added and can be moved with
  **Alt+Shift+Up/Down**, position announced as it moves. One stored ordinal per
  account; reordering never touches a server. `Alt+Shift+R` is the only existing
  Alt+Shift binding, so there is no conflict inside the application. **Bare
  Alt+Shift is the Windows input-language hotkey**, so releasing the combination
  after the arrow can trip a layout switch on machines with more than one layout
  installed. `docs/KEYBOARD_SHORTCUTS.md` gets the entry in the same commit.

- **D-15:** An account branch announces name, unread and folder count: "Work, 12
  unread, 9 folders". The folder count is what a collapsed branch cannot
  otherwise say. Expanded or collapsed, and the level, come from the tree
  control and are not in the text.

- **D-16:** `Enter` on an account branch or on the "On this computer" group
  expands or collapses it and announces the new state. Nothing else. This
  matches how a non-selectable IMAP folder (`selectable: false`) already
  behaves.

- **D-17:** Local folders live in a group node, **"On this computer"**, placed
  after the account branches and before `Labels`, because it holds Drafts, Sent
  and Outbox which people open daily. It follows the existing convention in
  `UIUpdate::FoldersLoaded`: a named branch, omitted entirely when empty rather
  than left as an empty node to arrow into.

- **D-18:** **Only `Inbox` is per account.** `Sent`, `Outbox`, `Drafts`, `Junk`
  and `Trash` become one each, shared across every account, owned by a
  **reserved "this computer" account id** — the same trick `LOCAL_PREFIX` plays
  with paths, so `UNIQUE(account_id, path)` keeps working and no schema change
  is needed. An IMAP account uses the same shared `Outbox` and keeps its server
  `Sent`, `Drafts`, `Trash` and `Junk`, so **`FOR_IMAP` becomes empty**.
  Corrected 2026-08-30. As first written this said `FOR_IMAP` stays `[Outbox]`,
  in the same sentence as saying the Outbox is shared. Both cannot hold:
  `for_account` is what creates the per-account folder rows, so keeping `Outbox`
  in `FOR_IMAP` means every IMAP account goes on making its own, which is the
  repetition this decision removes. The executor of 01-07 found it before
  writing anything and Pratik ruled: shared for everyone, one send queue on this
  computer, because a queued message already knows which account sends it.
  — **Reversibility:** one-way — existing databases are migrated by D-19, which
  moves messages between folder rows. Undoing it means knowing which account
  each message came from after they have been merged.

- **D-19:** On first open of an existing database, each account's local `Sent`,
  `Outbox`, `Drafts`, `Junk` and `Trash` are **merged into the shared ones,
  message by message, and nothing is removed until every message has landed**. A
  summary says how many moved and from where, spoken and written to the log.
  `importing_messages` opens by warning that this is how mail gets lost quietly;
  this migration is the same risk and needs the same care.
  — **Reversibility:** one-way — it rewrites `folder_id` on messages in the
  user's only copy of that mail.

- **D-20:** A folder a user creates under a POP account goes **under that
  account's branch, beside its Inbox**, not into "On this computer". It is still
  local, so it never passes through `Allowed::mail`.

- **D-21:** Imported archives keep landing under `\u{1}Local/Imported`, now
  inside "On this computer", with **one branch per archive named after the
  file**, holding that archive's own nested folders. No question at import time.
  The user renames the branch afterwards from the context menu or the Action
  menu, reusing FOLDER-01's rename, which is purely local.

- **D-22:** Nesting is stored, not computed. Sync splits the path where the
  delimiter is known and writes a **nullable `parent_id`** on the folders row.
  The tree reads a parent and never splits anything. This is what
  `imap.rs:769` anticipates: "the delimiter is not carried on the struct... it
  comes back when the tree gains a hierarchy." IMAP returns a delimiter per
  mailbox in the LIST response, not per server, so it cannot be assumed. The
  non-selectable rows IMAP already returns are the intermediate nodes.
  — **Reversibility:** costly — an additive column, but every folder-listing
  path is rewritten against it.

- **D-23:** Local folders nest with `/`, which their paths already use. A name
  containing `/` is **escaped, not refused**. This was chosen over refusal after
  the second-spelling risk was raised. It is built safely by copying the split
  `ImapFolder` already makes: the escaped path is the stored identity, the real
  name is what a person reads, one escape function, one unescape, and a guard
  that they round-trip. `ImapFolder`'s own comment gives the reason: re-encoding
  the readable form makes unreachable exactly the folder that could not be
  decoded.

- **D-24:** A collapsed parent's unread announcement is a **setting** with two
  options: both numbers always ("Archive, 3 unread here, 41 in all"), or both
  when collapsed and its own when expanded. **Default: both always**, so a row
  never changes meaning as the user arrows around. Applies to folders, account
  branches and "On this computer" alike.

- **D-25:** What the tree remembers across a restart is keyed by **stable
  identity, never by label**: account id for branches, `(account_id, path)` for
  folders, a stable id of its own for an imported archive so a rename does not
  lose it. `wx_app.rs` already carries a comment about a saved-search rename
  taking the cursor to the wrong row for exactly this reason.

- **D-26:** Rename changes the leaf **only**. Moving a folder to a different
  parent is its own `Move To` command on the Action menu, naming the new parent
  from a list, confirming, and saying where it went. Keyboard-only, no drag
  (WCAG 2.5.7). In IMAP both are the RENAME verb, and separating them stops a
  typo in a text box moving a folder irreversibly.

- **D-27:** A folder the server no longer lists is **not removed without
  asking**, and the question is a **modal dialog, right away**. The dialog-during-
  background-sync risk was raised and the modal chosen deliberately. Two prior
  fixes in this codebase name the failure it must avoid, so the modal carries two
  constraints: **one at a time**, never one per folder or per concurrently
  syncing account, and **not while an editor has focus** — it waits. Cached mail
  in the folder is untouched until the user answers.

### Favourites

- **D-28:** Favourites sits **above `ALL_INBOXES`**, at the very top, and is
  omitted entirely when nothing is pinned. Final order: Favourites,
  `ALL_INBOXES`, account branches, "On this computer", `Labels`, saved searches.

- **D-29:** Favourites **mirrors the account structure**: account sub-branches
  inside the group, so a pinned Inbox from two accounts is never two rows called
  Inbox.

- **D-30:** Pinning makes a **copy**. The folder stays in its account branch.
  Unpinning cannot lose anything and the tree the user learned does not change
  under them.

- **D-31:** New pins go to the bottom of their account's group and move with the
  same **Alt+Shift+Up/Down** as accounts. One gesture for rearranging anything in
  this tree.

- **D-32:** A pin keys by the same stable identity as D-25, so a rename keeps it
  and a real deletion takes it. A folder marked gone by D-27 keeps its pin, and
  its Favourites row says gone too.

### Emptying and marking read

- **D-33:** `Empty Folder` deletes every message **through the same decision one
  deletion uses** — `local_folders::deleting(from, protocol, asked, allowed)` per
  message, and the server path for server folders. So emptying Trash removes,
  emptying Inbox moves to Trash, and the per-account "Let me delete mail on this
  computer" setting gates it without a second gate being written. The
  confirmation says which of the two will happen, because it knows.

- **D-34:** Whether `Empty` reaches subfolders is a **setting**, defaulting to
  **including them**. Because that default is the destructive reading, the
  confirmation must carry the whole cost where it is met: the folder, the total
  count, how many subfolders, and whether it moves or removes.

- **D-35:** Whether `Mark Folder Read` reaches subfolders is its **own setting**,
  also defaulting to **including them**. Kept separate from D-34 deliberately:
  one destroys mail, the other loses your place, and neither has an undo.

- **D-36:** Emptying that fails partway **stops and says exactly where it got
  to**: "Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the
  server refused. 118 messages were not removed." Running it again finishes the
  job. All-or-nothing was ruled out because IMAP has no transaction to build it
  on and putting messages back gives them new UIDs.

- **D-37:** The confirmation counts the **cache**, exactly, at confirmation time,
  and says the number is what is stored here. The report afterwards gives what
  was actually removed, which may be larger. `folders.total_count` is a cached
  number from the last sync and is not used, and no round trip is made in front
  of a dialog the user may cancel.

- **D-38:** `Empty` on an already-empty folder stays on the menu, enabled, and
  says "Archive and its subfolders are already empty" without a confirmation
  dialog. A menu item that greys out for a reason the user cannot see is what
  twenty-eight status-line messages were removed for.

### Settings this phase adds

Five, each of which **must be wired to a real screen in this phase**. A setting
the model holds and no screen writes is FEEDBACK-01, which is sitting in Phase 6
because it has already happened here once.

| Setting | Options | Default | Decision |
|---|---|---|---|
| Delete scope on a conversation row | This folder's messages / the whole conversation | This folder | D-07, D-08 |
| How far a conversation reaches | The whole account / one folder | The whole account | D-08 |
| Unread announcement on a parent | Both numbers always / both when collapsed | Both always | D-24 |
| `Empty` reaches subfolders | Yes / no | Yes | D-34 |
| `Mark Folder Read` reaches subfolders | Yes / no | Yes | D-35 |

### Claude's Discretion

- The exact escape scheme for D-23, subject to the round-trip guard and the
  stored-path-is-identity rule.
- What THREAD-02's incremental rethreading does to a row the user is standing on,
  beyond the criterion already stating it must not re-announce rows they are not
  on.
- How the tree presents an account that has never synced.
- Whether `Move To` (D-26) reuses the destination picker the message-level Move
  already has.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### The plan and its requirements
- `.planning/REQUIREMENTS.md` — FOLDER-01, FOLDER-02, FOLDER-03, THREAD-01,
  THREAD-02 and their `[S]`/`[D]` criteria. All reviewed 2026-08-29.
- `.planning/ROADMAP.md` §Phase 1 — the five success criteria. See the note in
  `<domain>`: the discussion widened the phase past them.
- `.planning/STATE.md` — the review record, and the standing constraint that
  nothing has ever run against a real mail account.

### Threading
- `docs/plans/20260726-mail-at-scale.md` §"Threading algorithm" — specifies the
  incremental assignment THREAD-02 needs, and names the two-tree merge as the
  case worth testing.
- `src/application/threading.rs` — `thread_messages`, `ThreadInput`,
  `ThreadPlacement` (`parent_id` is already `Option`, for a root that is not
  present), `continuing`, `as_stored`, `heading_level`.
- `src/presentation/wx_thread_view.rs` — the existing conversation `TreeCtrl`,
  and the module comment arguing for keeping the list flat, which D-01 honours.

### The folder tree and its data
- `src/presentation/wx_app.rs` `UIUpdate::FoldersLoaded` (around line 10527) —
  the tree rebuild, `ALL_INBOXES`, the `Labels` and saved-search branches, and
  the stated convention of omitting an empty branch. Also `land_the_cursor` and
  the comment on why a rename broke cursor restoration.
- `src/application/local_folders.rs` — `LOCAL_PREFIX`, `is_local`,
  `for_account`, `FOR_POP`, `FOR_IMAP`, `LocalDelete`, `deleting`, and the two
  message constants for the local-delete setting.
- `src/application/import_tree.rs` — `where_imported_folders_go`,
  `IMPORTED_FOLDERS_ARE_UNDER`, `where_the_folders_land`,
  `is_a_name_that_can_be_used`, and the module's refuse-rather-than-repair
  rationale.
- `src/service/protocols/imap.rs` `list_folders` (around line 763) — `ImapFolder`
  and its `name` / `display_path` / `path` split, `selectable`,
  `holds_all_mail`, `subscribed`, `set_subscribed` (line 840), and the comment
  at line 769 saying the delimiter "comes back when the tree gains a hierarchy".
- `src/data/message_cache/mod.rs` line 1293 — the `folders` table and
  `UNIQUE(account_id, path)`, which D-18 and D-22 both depend on.
- `src/presentation/message_columns.rs` — `MessageColumn`, `heading`, the stored
  identifier, and the sort expression, all from one enum. D-02 turns on them
  staying that way.

### Rules this phase must not break
- `CLAUDE.md` §"Project rules" — schema changes are additive; never drop or
  rename a shipped column. D-18, D-22 and D-25 all add rather than change.
- `CLAUDE.md` §"Accessibility" — MSAA and UI Automation are two channels and
  both must be right; the level comes from the native `TreeCtrl`, not the label.
- `CLAUDE.md` §"Documentation and writing" — `docs/KEYBOARD_SHORTCUTS.md` is
  updated in the same commit as a shortcut (D-14, D-31).
- `src/application/allowed.rs` — `Allowed::mail` gates server writes and is off
  for a new install. `local_folders::is_local` is the only thing that decides
  which side a folder is on.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets
- `wx_thread_view::ThreadNode` and its `TreeCtrl` — D-01 makes this the whole of
  what a conversation row opens into. It exists and announces levels natively.
- `local_folders::deleting` — D-33 routes emptying through it rather than writing
  a second answer to what deleting means.
- `import_tree::is_a_name_that_can_be_used` — already asks the one function that
  knows what a name Windows will take. D-23 escapes rather than refuses, but the
  same validation is where a name is judged.
- `ImapFolder.selectable` — already models "only a name in the hierarchy", which
  is what D-22's intermediate parent nodes are.
- `tree_position` — the existing sort, unchanged inside an account branch (D-13).
- The `Labels` and saved-search branches — the pattern D-17 and D-28 follow:
  named branch, omitted when empty.

### Established patterns that constrain this
- **The message list holds no text.** It is virtual, and `wx_app.rs` says so at
  line 9814. D-01 keeps it that way; every conversation row's text is answered
  by the virtual text callback.
- **One enum answers display and sort.** `MessageColumn` gives both. D-02 is only
  safe while that stays true.
- **Path is identity, display is separate.** `ImapFolder` keeps `path` apart from
  `display_path` and says why. D-23 copies it for local folders.
- **A rebuild must put the cursor back.** `land_the_cursor` exists because a sync
  finishing on a timer used to move the cursor mid-list. Every tree change here
  goes through it.

### Integration points
- `UIUpdate::FoldersLoaded` is where the tree is rebuilt; D-13, D-17, D-21, D-28
  and D-29 all land there.
- `MailController` and `Allowed::mail` for the server half of FOLDER-01. There is
  no CREATE, RENAME or DELETE mailbox command in `imap.rs` today; its public API
  is listed in FOLDER-01's evidence. Three new protocol verbs.
- `mail_sync.rs` is where D-22 computes `parent_id` and where D-27 notices a
  folder the server has stopped listing.
- The account settings screen already carries "Let me delete mail on this
  computer"; the five new settings need a home and a keyboard path.

</code_context>

<specifics>
## Specific Ideas

- **Alt+Shift+Up/Down** was asked for by name, for moving accounts and pins.
- **Apply view to other folders** was asked for by name, as the thing that makes
  a per-folder setting usable rather than tedious.
- **Rename an imported archive from the context or Action menu**, rather than
  being asked to name it at import time.
- The phrase used for the whole shape of the settings work: *threading settings
  should appear in the Settings dialog based on all these questions.* Five did;
  the rest stayed decided, because a toggle for every decision multiplies the
  combinations nobody runs.

</specifics>

<deferred>
## Deferred Ideas

- **A combined view of everything under an account branch**, the way
  `ALL_INBOXES` works across accounts. Raised while deciding what `Enter` does on
  a branch (D-16) and set aside: it is a capability of its own, needing answers
  for what Delete and Move mean inside it.
- **SCALE-03 is linked to D-04 and the link is recorded rather than acted on.**
  Threading here is built to be correct on a partial mailbox; the conversation's
  name can change as older mail arrives, and SCALE-03 (Phase 3, fetch a whole
  mailbox) is what ends that. The roadmap order is unchanged and Phase 1 closes
  on its own.

</deferred>

<post_research>
## Decisions added after research (2026-08-29)

`01-RESEARCH.md` found five things the discussion could not have known. Two
needed answering and were put to Pratik; three are findings that change how a
decision above is built rather than what it decides. Every claim below was
verified against the tree before it was written here.

### New decisions

- **D-39:** `messages.thread_id` gets a **stable id derived from the
  conversation's root**, computed once when a message is stored, from the root
  its `References` chain points at, and never recomputed by batch. The same
  conversation carries the same id in every folder and across time.
  This is a producer to build, not a query to write: `thread_id` is a column
  **nothing writes and nothing reads back**. Its only occurrence in `src/data/`
  is the `ensure_column_exists` at `mod.rs:2065` that creates it, and
  `ui_types.rs:136` says outright that threading is not computed yet. Threading
  today is an in-memory computation over one folder's loaded page in
  `wx_app.rs::apply_threading`. Meanwhile `message_columns.rs:131` sorts on
  `m.thread_id`, so sorting by Thread orders every row by NULL and does not
  fail. D-08 cannot span an account without this, and it must be ordered before
  anything that reads a thread id.
  The rejected shape matters: today's in-memory id is the least Message-ID in
  the batch, so the same conversation gets a different id per folder and an
  arriving message can lower it. A THREAD-02 implementation adopting a found
  thread's id would then disagree with the next batch recompute. That is the
  lenient-reader-strict-writer shape this project has been bitten by before.
  — **Reversibility:** one-way — once ids are written, changing the derivation
  means recomputing every stored row. It is free today only because the column
  has never held a value.

- **D-40:** The D-19 migration assigns a **fresh uid unique within the shared
  folder** and records the original in an additive column beside it.
  `messages` carries `UNIQUE(folder_id, uid)` and IMAP UIDs are per-mailbox, so
  two accounts' local Trash both holding uid 42 is expected rather than
  hypothetical. Without this the migration either fails or drops a message, and
  it is the likeliest way D-19 loses mail. Nothing that keys on
  `(folder_id, uid)` changes and the constraint is untouched, which keeps the
  additive-only schema rule.
  — **Reversibility:** one-way — it rewrites uids on the user's only copy of
  that mail. The original column is what makes the move traceable afterwards.

- **D-41:** A **modified UTF-7 encoder** is in scope, and FOLDER-01's create,
  rename and move cannot ship without it.
  `src/service/protocols/imap/mailbox_name.rs` decodes and does not encode, and
  its own module comment says: *"An encoder belongs here the day something
  creates or renames a mailbox."* This phase is that day. `async-imap 0.11.3`
  does no modified UTF-7 anywhere and its `validate_str` only quotes and rejects
  CR/LF, so a name goes to the server exactly as handed over. Without an
  encoder, creating a folder works in English and corrupts every other alphabet.
  Nothing named this: not FOLDER-01's evidence, not the roadmap, not the
  discussion. It is new work, it is not optional, and it needs the round-trip
  test against the existing decoder that makes an encoder trustworthy.

- **D-42:** The five settings go on the **Reading** page of the settings
  notebook, under one labelled group. The notebook has seven pages (General,
  Compose, Reading, Permissions, Calendar & PIM, Feedback, Advanced); all five
  are about how the message list and folder tree behave, so one group in one
  place beats scattering them. If the group's name is ever named in a sentence
  elsewhere, it comes from one constant rather than being typed twice, which is
  what `test_the_settings_screen_does_not_write_the_section_name_out_itself`
  already enforces for the Allow Changes section.

- **D-43:** This phase adds the **mirror of the settings guard**, and criterion 8
  is not met without it. `test_every_setting_somebody_can_change_is_read_by_something`
  (`src/data/config.rs:1306`) walks every shipping file *except* `config.rs` and
  `wx_settings.rs`, by name, with a stated reason. So it catches a setting that
  is offered and ignored, and is **structurally blind** to one that is stored and
  never offered. That blind spot is exactly FEEDBACK-01, and exactly the risk of
  adding five settings at once. The mirror reuses `stored_setting_names` and
  `what_ships` and asks the opposite question of `wx_settings.rs` alone.

### Amendments to decisions above

- **D-04 is already written, in a dependency.** Do not write a reply-prefix
  stripper. `mail-parser 0.11.5` is already in `Cargo.toml` and exposes
  `parsers::fields::thread::thread_name(&str) -> &str` on a public path,
  implementing the RFC 5256 base-subject algorithm with 19 reply and 22 forward
  prefixes across 17 languages. Separately, `wx_compose.rs:161-175` prepends
  `"Re: "` on a case-sensitive ASCII `starts_with` and will disagree with it;
  bring the two into line in the same change.

- **D-26's Move To is one command, not a walk.** RFC 9051 §6.3.6 requires RENAME
  to rename inferior names too, so moving `Archive/2026` to `Old/2026` takes its
  children with it in one operation. **Renaming INBOX is a special case that must
  be refused before it is attempted**: the RFC has it create a new mailbox and
  move INBOX's messages into it, leaving INBOX empty, which is a data-shaped
  surprise dressed as a rename.

- **FOLDER-01's delete needs a depth-first walk.** RFC 9051 §6.3.5 requires
  DELETE **not** to remove inferior names, the opposite of RENAME. Deleting a
  subtree means deleting deepest-first, and a `\Noselect` parent that only names
  a hierarchy has its own rules.

- **D-25 must not use wxdragon's tree item custom data.** That data goes into a
  process-global registry; `delete_all_items` does not clear it, and
  `cleanup_all_custom_data` returns early on any childless item, so it never
  clears a leaf. The tree is rebuilt on every `FoldersLoaded`, so building
  expansion state on it leaks one entry per folder per sync, forever. Use a
  parallel vector keyed by the stable identity D-25 already names, which is what
  `collect_rows` (`wx_app.rs:10103`) already does.

- **THREAD-02 has its mechanism.** `ListCtrl::refresh_item`
  (`wxdragon 0.9.17`, `list_ctrl.rs:781`, with `refresh_items` beside it at
  `:791`) repaints one row without touching the rest, which is precisely
  "rethreading on arrival does not re-announce rows the user is not on".

### One correction the researcher made to its own work

It first reported that the loopback harness lacks CREATE, RENAME and DELETE and
that extending it was a prerequisite. True of `a_server_that_can`, and wrong as a
conclusion: a second harness, `a_server_answering`, sits further down the same
module with a permissive fallback and needs no change. Recorded because the
first version of that claim would have added a task nobody needed.

</post_research>

---

*Phase: 1-Folders and conversations*
*Context gathered: 2026-08-29*
*Amended after research: 2026-08-29*

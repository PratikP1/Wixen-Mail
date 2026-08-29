# Phase 1: Folders and conversations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution
> agents. Decisions are captured in CONTEXT.md — this log preserves the
> alternatives considered.

**Date:** 2026-08-29
**Phase:** 1-Folders and conversations
**Areas discussed:** what a thread row is, where local folders nest, what
emptying removes, pinning

---

## What a thread row is

### Opening a conversation row

| Option | Description | Selected |
|--------|-------------|----------|
| Enter opens the existing tree | Rows always collapsed; `wx_thread_view`'s `TreeCtrl` announces level natively; list stays virtual | ✓ |
| Expand in place, indented rows | Depth spoken from row text rather than by the control | |
| Replace the list with a `TreeCtrl` | Native levels, loses virtual mode and the real UI Automation set size | |

**User's choice:** Enter opens the existing tree.

### What a row describes (asked twice)

First answer was "show the newest message's values". The follow-up question
about state columns showed that rule makes a row say a conversation has no
attachment when it has one, and the user reopened the question.

| Option | Description | Selected |
|--------|-------------|----------|
| The conversation | One rule per column family, display and sort from the same rule | ✓ |
| The newest message, count as the signal | One rule, least code, row can be false about the thread | |
| Split: dates newest, states any-of | Middle; `Size` and `Correspondent` stay wrong | |

**User's choice:** the conversation, after reopening.
**Notes:** the reopening was prompted by the state-column question, not by a
new option. `MessageColumn` giving both the displayed value and the `ORDER BY`
from one enum is what made the per-column rule the cheaper answer.

### Where the counts live

| Option | Description | Selected |
|--------|-------------|----------|
| The `Thread` column carries them | Existing column, currently a thread id; row assembled from visible columns | ✓ |
| Appended to the Subject cell | Two facts in one cell; sorting a string with numbers in it | |
| Spoken on focus, not in a column | Sighted users get nothing | |

**User's choice:** the `Thread` column.

### Scope of the thread view setting

| Option | Description | Selected |
|--------|-------------|----------|
| Per folder, remembered | Stored beside the collapsed state | ✓ |
| One setting for the whole application | A single menu toggle | |
| Per account | Work threaded, personal flat | |

**User's choice:** per folder, **plus a command to apply the current view to
other folders**.

### What "other folders" offers

| Option | Description | Selected |
|--------|-------------|----------|
| Subtree / account / everywhere | Each named in a sentence with a count, confirms first | ✓ |
| All mail folders, one action | Overwrites every hand-set folder | |
| Pick folders from a checklist | A checkbox tree to make accessible from scratch | |

**User's choice:** subtree, account or everywhere.

### What names a conversation

| Option | Description | Selected |
|--------|-------------|----------|
| Oldest present, prefixes stripped | New code; label moves as older mail arrives | ✓ |
| Oldest present, as it stands | "Re: Re: Fwd:" before every subject | |
| Newest message's subject | Stable, furthest from the root | |

**User's choice:** oldest present with prefixes stripped, **plus a change that
downloads all messages rather than the newest 500**.
**Notes:** that second half is SCALE-03, an existing requirement in Phase 3, not
new scope.

### How the roadmap handles the SCALE-03 link

| Option | Description | Selected |
|--------|-------------|----------|
| Leave the order, record the link | Phase 1 correct on a partial mailbox; planner builds for the moving label | ✓ |
| Pull SCALE-03 before Phase 1 | Inverts the stated dependency | |
| Make Phase 1 depend on SCALE-03 only | Phase 1 could not close on its own | |

**User's choice:** leave the order, record the link.

### A lone message's row

| Option | Description | Selected |
|--------|-------------|----------|
| Like a message row | No count on conversations of one | |
| Every row is a conversation row | "1 message" on most of the list | |
| Counts only when the folder has real threads | Column appears when it has something to say | ✓ |

**User's choice:** counts only when the folder has real threads.

### Adaptive column against a hand choice

| Option | Description | Selected |
|--------|-------------|----------|
| A hand choice wins, forever | One stored value separates "not chosen" from "chosen off" | ✓ |
| The adaptive rule always wins | Silently undoes a menu choice | |
| Adaptive only while thread view is on | The setting disappears and returns as you toggle | |

**User's choice:** a hand choice wins.

### What an action on a conversation row acts on

| Option | Description | Selected |
|--------|-------------|----------|
| The whole conversation, said before it happens | Count named in the confirmation and the announcement | ✓ |
| The newest message only | Makes thread view do nothing for the work | |
| Whole conversation, confirm every destructive one | A dialog people learn to dismiss | |

**User's choice:** the whole conversation.

### How far a conversation reaches

| Option | Description | Selected |
|--------|-------------|----------|
| This folder only, and say there are more | One extra count query | |
| This folder only, silently | True about the folder, false about the conversation | |
| The whole conversation across the account | Cross-folder queries; flagged as wanting its own phase | ✓ |

**User's choice:** the whole account.
**Notes:** flagged at the time as larger than a clarification. Chosen anyway,
and the follow-up questions settled which list the row appears in and what
`holds_all_mail` means for the count.

### Which list shows a cross-folder conversation

| Option | Description | Selected |
|--------|-------------|----------|
| Every folder it touches, whole-thread count | Same reading standing in Inbox or Sent | ✓ |
| Only the folder holding its newest message | Conversations vanish into Sent | |
| Every folder it touches, own-folder count | Same conversation, two numbers | |

**User's choice:** every folder it touches, whole-thread count.

### Delete on a cross-folder conversation

| Option | Description | Selected |
|--------|-------------|----------|
| Only this folder's messages, and say so | One folder, one server operation | |
| Every message in the conversation | Several operations, can half-succeed | |
| Ask, each time | A dialog on the commonest destructive action | |

**User's choice:** "Make it a setting via the setting dialog so that the user
can choose the default. In fact, threading settings should appear based on all
these questions."
**Notes:** this reframed the rest of the discussion. The response was to propose
a split between decisions that are genuine preferences and decisions that are
structure, rather than turning all of them into toggles.

### Which decisions become settings

| Option | Description | Selected |
|--------|-------------|----------|
| Delete scope | Default: this folder | ✓ |
| Thread view for a new folder | Default: flat | |
| How far a conversation reaches | Default: the whole account | ✓ |
| Counts on lone messages | Default: only when the folder has real threads | |

**User's choice:** delete scope and how far a conversation reaches. The other
two stay decided.

### Selection across a view switch

| Option | Description | Selected |
|--------|-------------|----------|
| Carry it through both ways | Lossless round trip; original set held across the switch | ✓ |
| Keep focus, drop the selection | What the criterion literally promised | |
| Select the conversations, keep them coming back | Quietly widens a selection | |

**User's choice:** carry it through both ways.

### Sort across a view switch

| Option | Description | Selected |
|--------|-------------|----------|
| Same column and direction | Applied by the conversation rule | ✓ |
| Each view remembers its own sort | Doubles stored state; list reorders on toggle | |
| Reset to the default on every switch | Throws the ordering away twice per round trip | |

**User's choice:** same column and direction.

---

## Where local folders nest

### The group node

| Option | Description | Selected |
|--------|-------------|----------|
| By account type, as `for_account` already does | No group node either way | |
| Always a group node | One shape everywhere | ✓ |
| Group only where there is a mix | A heading over one item | |

**User's choice:** always a group node, **with the provisos that imported mail
with folders is treated as a hierarchy the same way, and that this is necessary
once there is more than one POP account**.
**Notes:** the second proviso found a real defect. The tree merges every account
flat today, so two POP accounts already give two indistinguishable Inboxes.

### Grouping by account

| Option | Description | Selected |
|--------|-------------|----------|
| One branch per account, All Inboxes on top | Account name announced as the parent level | ✓ |
| Branches only when there is more than one | Adding a second account reshapes the tree | |
| Stay merged, account name in each label | Lengthens every row | |

**User's choice:** one branch per account, **plus Outbox, Junk, Deleted and
Drafts being only one for POP accounts**.

### Which folders are shared

| Option | Description | Selected |
|--------|-------------|----------|
| Inbox and Sent per account; four shared; migrate | | |
| Four shared, leave existing ones alone | Two shapes forever | |
| Sent shared too, only Inbox per account | Five shared | ✓ |

**User's choice:** only Inbox is per account.
**Notes:** `UNIQUE(account_id, path)` and `LocalFolder::path()` carrying no
account id are what make a reserved "this computer" account id the answer that
needs no schema change.

### Migration

| Option | Description | Selected |
|--------|-------------|----------|
| Merge, and say what moved | Nothing removed until every message has landed | ✓ |
| Merge silently | Reads as data loss even when nothing is lost | |
| Keep the old ones read-only alongside | Both shapes in the tree until the user acts | |

**User's choice:** merge, and say what moved.

### Where the group sits

| Option | Description | Selected |
|--------|-------------|----------|
| After the accounts, before Labels | Holds Drafts, Sent, Outbox, opened daily | ✓ |
| First, above the account branches | Working state ahead of the mail | |
| Last, with Labels and saved searches | Buries Drafts and Sent | |

**User's choice:** after the accounts, before Labels.

### Imported archives

| Option | Description | Selected |
|--------|-------------|----------|
| One branch per archive, named at import | Asks a question in a flow run once | |
| One branch per archive, named after the file | No question at import | ✓ |
| Merge every archive into one Imported tree | Two Inboxes collide | |

**User's choice:** named after the file, **with the user able to rename it from
the context menu or Action menu on focus**.
**Notes:** rename is already FOLDER-01's, on the Action menu because it acts on
the selection, and purely local.

### Where a created folder goes

| Option | Description | Selected |
|--------|-------------|----------|
| Under the account branch, beside its Inbox | Account branch stays meaningful | ✓ |
| Under "On this computer" | POP branch would hold only an Inbox | |
| Ask each time | A question on a common action | |

**User's choice:** under the account branch.

### Unread counts on a parent

| Option | Description | Selected |
|--------|-------------|----------|
| Every parent gives both, always | Row meaning does not change with state | |
| Both when collapsed, own count when expanded | Same folder, two numbers | |
| Folders roll up; structural nodes do not | Account branch says a useless number | |

**User's choice:** "user configurable via settings with options 1 and 2 being
offered. Make Option 1 default."

### What the tree remembers

| Option | Description | Selected |
|--------|-------------|----------|
| By stable identity, never by label | The lesson the saved-search code already learned | ✓ |
| You decide | | |
| By label | A rename loses the state | |

**User's choice:** by stable identity.

### Where Favourites lands

| Option | Description | Selected |
|--------|-------------|----------|
| Above All Inboxes | Omitted when nothing is pinned | ✓ |
| Below All Inboxes, above the accounts | Costs pinning one keystroke | |
| Inside each account branch | No longer at the top of the tree | |

**User's choice:** above All Inboxes.

### How the tree learns a parent

| Option | Description | Selected |
|--------|-------------|----------|
| Store the parent, not the delimiter | Nullable `parent_id`; tree never splits | ✓ |
| Carry the delimiter per folder | String splitting on every rebuild | |
| One delimiter per account | A mixed-delimiter server nests wrongly | |

**User's choice:** "Choose recommended."
**Notes:** `imap.rs:769` carries a comment saying the delimiter "comes back when
the tree gains a hierarchy". This is that change.

### The local separator

| Option | Description | Selected |
|--------|-------------|----------|
| Forward slash, refuse a name that carries one | Reuses `is_a_name_that_can_be_used` | |
| Forward slash, escape a name that carries one | An escaping scheme between path and name | ✓ |
| A reserved character like the prefix uses | Existing local paths would all change | |

**User's choice:** escape it.
**Notes:** the second-spelling risk was raised in the option text and the choice
made anyway. It is recorded in CONTEXT.md with the constraints that make it
safe: escaped path as identity, real name as display, one escape, one unescape,
and a round-trip guard, copying the split `ImapFolder` already makes.

### Account order

| Option | Description | Selected |
|--------|-------------|----------|
| Order added, user-rearrangeable | One stored ordinal | ✓ |
| Order added, fixed | Main account stuck third | |
| Alphabetical | Renaming moves the tree | |

**User's choice:** order added, rearrangeable, **with Alt+Shift+Up/Down**.
**Notes:** `Alt+Shift+R` is the only existing Alt+Shift binding, so no conflict
inside the application. Bare Alt+Shift is the Windows input-language hotkey,
which was flagged once.

### What an account branch announces

| Option | Description | Selected |
|--------|-------------|----------|
| Name, unread, and folder count | Folder count is what a collapsed branch cannot say | ✓ |
| Name and unread only | No sense of how much is under it | |
| Name only | Counts only one level down | |

**User's choice:** name, unread, folder count.

### Rename against reparenting

| Option | Description | Selected |
|--------|-------------|----------|
| Rename is the name; Move To is its own command | Keyboard-only, no drag (2.5.7) | ✓ |
| Rename takes a whole path | A typo moves a folder irreversibly | |
| Rename only; no move this phase | Nested folders could not be rearranged | |

**User's choice:** rename is the name; moving is its own command.

### A folder the server stops listing

| Option | Description | Selected |
|--------|-------------|----------|
| Keep it, mark it gone, say so once | Nothing removed without asking | |
| Remove it and its children | One missing parent takes every child | |
| Ask before removing | A question arriving during a background sync | ✓ |

**User's choice:** ask before removing.

### How that question reaches the user

| Option | Description | Selected |
|--------|-------------|----------|
| Mark them, offer to review, ask when they look | Asking on the user's schedule | |
| A modal dialog, right away | Takes focus mid-task | ✓ |
| A modal, but only when the window has focus | Still lands mid-sentence | |

**User's choice:** a modal, right away.
**Notes:** the NVDA modal-freeze history was raised before the choice. Two prior
changelog fixes name the failure mode, so the decision carries two constraints
in CONTEXT.md: one dialog at a time, and never while an editor has focus.

### Favourites and duplicate names

| Option | Description | Selected |
|--------|-------------|----------|
| Account named only where ambiguous | Row label changes when a second pin arrives | |
| Always name the account | Lengthens every pinned row | |
| Mirror the account structure inside Favourites | Pinned folders two levels down | ✓ |

**User's choice:** mirror the account structure.

### Enter on a structural node

| Option | Description | Selected |
|--------|-------------|----------|
| Expand or collapse, nothing else | Matches non-selectable IMAP folders | ✓ |
| Open a combined view of everything under it | A capability of its own | |
| Nothing at all | A key that does nothing with no reason given | |

**User's choice:** expand or collapse.

---

## What emptying removes

### What Empty means

| Option | Description | Selected |
|--------|-------------|----------|
| Delete every message through the same decision | Routes through `local_folders::deleting` | ✓ |
| Empty always removes permanently | A second answer to what deleting means | |
| Only on Trash and Junk, always permanent | Decides a folder is not emptyable | |

**User's choice:** through the same decision.

### Empty and subfolders

| Option | Description | Selected |
|--------|-------------|----------|
| This folder only, and say the children are safe | Safe reading as the default | |
| The folder and everything under it | One confirmation before a whole tree | |
| Ask which, each time | A question inside a question | |

**User's choice:** "Make it a setting with 1 and 2 being the options and 2 being
the default."

### Partial failure

| Option | Description | Selected |
|--------|-------------|----------|
| Stop, and say exactly where it got to | Run again to finish | ✓ |
| Carry on, report the total at the end | A number of failures with no locations | |
| Undo everything on any failure | IMAP has no transaction for it | |

**User's choice:** stop and say where it got to.

### The count in the confirmation

| Option | Description | Selected |
|--------|-------------|----------|
| Count the cache exactly, report the truth after | No round trip in front of a cancellable dialog | ✓ |
| Use the cached `total_count` | A number from the last sync presented as now | |
| Ask the server first | Cannot answer offline | |

**User's choice:** count the cache, report the truth after.

### Mark Folder Read and subfolders

| Option | Description | Selected |
|--------|-------------|----------|
| One setting for both, renamed | Four settings, one says what it means | |
| Its own setting | Five settings, two nearly the same question | ✓ |
| This folder only, always | Four subfolders marked one at a time | |

**User's choice:** its own setting, **defaulting to including subfolders**.

### Empty on an already-empty folder

| Option | Description | Selected |
|--------|-------------|----------|
| Offered, and says there is nothing to do | Says the reason rather than hiding the command | ✓ |
| Greyed out | The reason is not visible | |
| Offered, and confirms anyway | A dialog before doing nothing | |

**User's choice:** offered, and says there is nothing to do.

---

## Pinning

Opened after the other three areas, because FOLDER-03 is in this phase and its
central question had not been asked.

### Copy or move

| Option | Description | Selected |
|--------|-------------|----------|
| A copy; it stays where it was | Unpinning cannot lose anything | ✓ |
| A move; it leaves its account branch | Account branch loses its most-used folder | |
| A copy, original marked as pinned | Describes a state visible one screen up | |

**User's choice:** a copy.

### Pin order

| Option | Description | Selected |
|--------|-------------|----------|
| Pin order, moved with Alt+Shift+Up/Down | One gesture for rearranging anything | ✓ |
| The tree's own order | Cannot put the most-used first | |
| Alphabetical | Archive above Inbox | |

**User's choice:** pin order, same gesture as accounts.

### A pinned folder that disappears

| Option | Description | Selected |
|--------|-------------|----------|
| Follows the folder, and goes when it does | Keys by the same stable identity | ✓ |
| Keep the pin, marked as missing | Favourites accumulates dead rows at the top | |
| Drop the pin silently | Not told | |

**User's choice:** follows the folder.

---

## Claude's Discretion

- The exact escape scheme for local folder names, subject to the round-trip
  guard and the stored-path-is-identity rule.
- What THREAD-02's incremental rethreading does to a row the user is standing on.
- How the tree presents an account that has never synced.
- Whether `Move To` reuses the message-level Move's destination picker.

## Deferred Ideas

- A combined view of everything under an account branch, the way All Inboxes
  works across accounts.
- SCALE-03's link to the conversation subject: recorded, roadmap order unchanged.

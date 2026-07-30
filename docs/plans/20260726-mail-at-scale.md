# Mail at scale: receiving, storage, and a message list that holds 200,000 rows

_Written 2026-07-26. Status: agreed, not yet implemented._

This plan covers implementing IMAP, storing hundreds of thousands of messages,
and presenting them in a list a screen reader user can actually work in. It also
covers sorting, reordering, and hiding columns without a mouse.

## Why this shape

The four questions in [what this project is for](../principles.md) decide most of it. The
one that bites hardest here is the fourth, what this allows to be done poorly. A
mail client at this scale has three failure modes that all look like success:

- A list that renders 200,000 rows and reports them honestly to the screen reader
  while taking twenty seconds to open a folder.
- A list that is fast because it is custom drawn, and therefore invisible to
  assistive technology.
- A sync that is efficient because it silently drops the messages it could not
  reconcile.

Every choice below is made against those.

## Receiving

### The client library

`async-imap` 0.11.3, with rustls for transport.

| Candidate | For | Against |
|-----------|-----|---------|
| **async-imap 0.11.3** | Async and tokio native. Actively released. Has `idle`, `uid_fetch`, `uid_search`, `uid_store`, `uid_mv`, and `select_condstore` for RFC 7162. `authenticate` accepts a custom SASL mechanism, so XOAUTH2 is reachable | No QRESYNC helper, so deletions need a UID set comparison. Transport is not built in |
| imap 3.0.0-alpha.15 | Long history | Synchronous, and has sat in alpha for years. The wrong shape for a tokio application |
| imap-next, imap-codec, imap-types | The most careful protocol modelling in the ecosystem, sans I/O | Alpha, and the client is left as an exercise. Months of work before the first message arrives |
| imap-proto | Good parser, and what async-imap already uses | Not a client |

It sits behind our own `MailTransport` trait so the application layer never names
it. That is not speculative abstraction: `imap-next` is the likely successor once
it stabilises, and the seam is one file.

MIME parsing stays with `mail-parser`, already a dependency. We do not write a
MIME parser.

### Authentication comes first

Gmail and Outlook both refuse password authentication over IMAP. XOAUTH2 is
required, which means the `oauth2` 5 migration is not a tidy-up. **Receiving mail
is blocked on it.** It also clears the three `rustls-webpki` certificate
validation advisories recorded in `.cargo/audit.toml`.

### Exchange

Microsoft begins blocking non-Microsoft EWS applications against Exchange Online
on 1 October 2026, with full retirement by April 2027. We will not write EWS.
Exchange Online goes through Microsoft Graph, where the existing client already
lives, using `delta` queries for incremental sync. On-premises Exchange speaks
IMAP and is served by the same path as everything else.

## Storage

### The problem with what exists

`CachedMessage` holds `body_plain` and `body_html` inline. At 200,000 messages
that is tens of gigabytes in a single SQLite file, and every folder listing query
drags the bodies through it. This has to change before any IMAP code lands,
because the migration only gets more expensive with real data in it.

### Three tiers

| Tier | Holds | Cost at 200,000 |
|------|-------|-----------------|
| Envelope, always local | uid, uidvalidity, folder, flags, internaldate, size, from, to, subject, message-id, references, has_attachments, modseq | About 1 KB each, so roughly 200 MB |
| Body cache, on demand | Fetched when a message is opened, evicted least-recently-used against a budget | Bounded by the budget, default 500 MB |
| Attachments | Never fetched automatically | Bounded |

SQLite runs in WAL mode. Indices on `(folder_id, uid)`,
`(folder_id, internaldate DESC)`, and `(thread_id)`.

Full text search uses FTS5 over subject and sender for everything, and over body
text only for bodies that have actually been fetched. **The search UI must say
so.** A search that silently covers 4% of a mailbox while looking like it covers
all of it is the "structure present, experience poor" failure in another costume.

## Fetching

**First sync, phase one.** `UID FETCH 1:* (UID FLAGS INTERNALDATE RFC822.SIZE
ENVELOPE BODYSTRUCTURE)` in chunks of 500 to 1000, newest first. Newest first is
the whole point: the inbox becomes usable after the first chunk instead of after
the last.

**First sync, phase two: snippets.** The snippet column needs the first couple of
kilobytes of body text, which the envelope does not carry. It can be requested in
the same command as `BODY.PEEK[1]<0.2048>`, but two kilobytes across two hundred
thousand messages is four hundred megabytes before the list is usable, so it does
not belong in phase one.

Snippets are backfilled in the background, newest first, once envelopes have
landed. Rows show an empty snippet until theirs arrives and then update in place.
Empty means "not fetched yet" rather than "no body", and the column must not
imply otherwise.

Storing the whole envelope matters more than showing it. A column backed by data
already captured can be switched on later for nothing; a column needing data we
did not store costs a full resync. So capture correspondent, to, cc, size, every
flag, and attachment structure whether or not they are visible.

**Incremental.** Keep `UIDVALIDITY`, the highest seen UID, and `HIGHESTMODSEQ`
per folder. New mail is `UID FETCH <highest+1>:*`. Flag changes come from
CONDSTORE `CHANGEDSINCE`. Deletions need a periodic UID set comparison, since
async-imap surfaces no QRESYNC. A `UIDVALIDITY` change means the folder is
discarded and resynced, and that is announced rather than done quietly.

**Bodies.** `UID FETCH n (BODY.PEEK[])` when a message is opened. `PEEK` is not
optional: without it, opening a message sets `\Seen` on the server no matter what
the user's preference says.

**Push.** IDLE on the selected folder, reissued every 29 minutes as RFC 2177
requires.

**Connections.** One for IDLE, two or three for fetching. Gmail allows fifteen
per account and punishes more.

**Live versus downloaded.** Metadata is always local. Bodies are cached as they
are read. Keeping a whole folder offline is an explicit per-folder choice with a
size budget and progress. There is no bulk download by default.

## The message list

### Native virtual mode, and nothing else

`ListCtrlStyle::Report | ListCtrlStyle::Virtual`, which is `WXD_LC_VIRTUAL` on
the native `SysListView32`:

```rust
list.set_item_count(207_431);
list.set_virtual_text_callback(|row, column| ...);
```

Memory becomes proportional to what is visible rather than what exists, and
because the control stays native, UI Automation reports the real set size. A
screen reader says "row 12 of 207,431" and means it.

Two things we will not do, both of which are faster to write and worse to use:

- **wxdragon's `VirtualList` widget** composes each row from recycled `Panel`
  objects. The accessibility tree churns as you scroll and there are no list
  semantics at all.
- **`DataViewCtrl`** on Windows is wxWidgets' generic custom drawn
  implementation, with much weaker UI Automation exposure than the native list.

The accessibility scan already confirms the current control is `SysListView32`.
We keep it.

### What virtual mode demands in return

The text callback runs during paint. It must read from an in-memory page cache
and never touch SQLite. Pages of 200 rows are loaded around the viewport, and a
cache miss returns a placeholder rather than blocking the paint.

Sorting and filtering move into SQL. Sorting 200,000 rows in memory on a header
click is a multi-second freeze, and a freeze is an accessibility failure, not a
performance one.

Nobody arrows through 200,000 messages. Search, jump to date, and next-unread are
the real navigation at this scale and get designed as first-class paths rather
than as consolations.

Progress during a long load uses the announcement queue's topic superseding, so a
sync that produces four hundred progress updates speaks its final count once.

## Columns: sorting, order, and visibility

### The rule

Everything here must work without a mouse, without dragging, and without a chord
that needs three fingers and a thumb. WCAG 2.5.7 forbids drag-only interaction
outright, and column reordering is the classic place applications ignore it.

### Sorting

Every column sorts both ways. Header clicks sort and toggle direction for people
using a mouse. The keyboard path is the **View, Sort By** submenu, with one radio
item per column and two more for ascending and descending.
Radio items matter: a screen reader announces which one is selected, so the
current sort is discoverable rather than remembered.

Reaching it is `Alt+V`, `S`, then the column's own letter. Three sequential
keystrokes, no chord, no contortion.

After any change the application announces the result in full, for example
"Sorted by date, newest first". The sort indicator is also set on the header for
sighted users.

### Which columns exist

Everything the envelope provides is stored. What is *shown* is a much smaller
set, because in virtual mode a row's accessible name is assembled from its
visible columns. Six visible columns means a screen reader reads six fields per
message while arrowing through an inbox, which is the difference between skimming
and wading. **Column visibility is the verbosity dial**, not a cosmetic
preference, and that is why it gets a dialog of its own.

For the same reason the flags are separate narrow columns rather than one merged
"Status". A single column reading "unread, flagged, attachment" costs all three on
every row. Separate ones let someone keep unread and drop the rest.

Default visible set, sorted by received, newest first:

| Column | Why it earns a place |
|--------|----------------------|
| Unread | The one piece of state that changes what you do next |
| Attachment | Changes whether the message can be dealt with right now |
| Subject | |
| Correspondent | From in most folders, To in Sent and Drafts |
| Received | Server arrival time |
| Snippet | Often removes the need to open the message at all |

Available and off by default: sent date, to, cc, size, flagged, answered,
forwarded, draft, tags, age, folder, account, thread.

**Correspondent rather than From.** In Sent and Drafts a From column is your own
address on every row, which is noise when it is read aloud a thousand times.

**Received rather than sent date.** The sent date is set by the sender and is
routinely wrong. Sorting by it puts forged-date spam permanently at the top.

### Per folder defaults

Layout is stored per account and per folder kind, because the useful set genuinely
differs:

| Folder kind | Default columns |
|-------------|-----------------|
| Inbox and generic | Unread, attachment, subject, correspondent, received, snippet |
| Sent | Attachment, subject, correspondent, sent date, snippet |
| Drafts | Attachment, subject, correspondent, saved date, snippet |

Unread is dropped from Sent and Drafts: it carries no information there, and a
column identical on every row is pure verbosity. The date column is whichever date
means something for that folder.

A user's own changes override the default for that account and folder kind.
Resetting returns to it.

### Order and visibility

One dialog does both, opened with **`F8`**. A bare function key was chosen
deliberately: `F1`, `F3`, `F5`, `F6`, and `F9` are already taken, `F8` is free,
and it needs no modifier at all. It is also on the View menu as **Columns...**
so it is discoverable rather than folklore.

The dialog is a checkable list of every column in display order:

| Control | Keys | Behaviour |
|---------|------|-----------|
| Column list | `Up`, `Down` | Move through columns. Each announces its name, position, and whether it is shown |
| Visibility | `Space` | Toggle. Announces "Subject, hidden" or "Subject, shown" |
| Move up | `Alt+Up` or the Move Up button | Announces "Subject moved to position 2 of 6" |
| Move down | `Alt+Down` or the Move Down button | As above |
| Reset | `Alt+R` | Back to defaults |
| Accept | `Enter` | Applies and announces the new layout |
| Cancel | `Esc` | Discards |

The list is never left in a state with no visible columns: the last remaining one
cannot be unchecked, and attempting it says why.

### How it is implemented

Order uses `set_columns_order`, which is native on Windows, so the control and
its accessibility tree agree about position without us reimplementing anything.

Hiding is done by rebuilding the column set rather than setting a width of zero.
A zero-width column still exists in the UI Automation tree, and a screen reader
may still read it, which is exactly the kind of invisible-to-sighted-users but
audible-to-everyone-else defect this project keeps finding. `clear_all` followed
by re-inserting the visible columns is cheap in virtual mode, because there are
no rows to restore, only `set_item_count` to call again.

The virtual text callback receives the logical column index rather than the
display position, so reordering does not disturb the mapping between a column and
the field it shows.

Layout and sort are persisted in `AppConfig`, starting global, with per-folder
overrides deferred until someone asks.

## Sequence

1. **Split bodies out of the messages table.** A migration, cheaper now than with
   real data behind it. No network, fully testable.
2. **Convert the message list to virtual mode**, with the page cache and the
   column model. Testable against 200,000 synthetic rows with no network at all,
   and this is where the accessibility risk actually lives.
3. **oauth2 5 and XOAUTH2.** Unblocks receiving and clears three certificate
   advisories. Needs a live account to verify.
4. **async-imap behind `MailTransport`**, envelopes only.
5. **Body fetch on demand** with the eviction budget.
6. **IDLE, then CONDSTORE incremental sync.**

Steps 1 and 2 need no credentials and no protocol work, which is why they come
first.

## Open: feedback channels

Requested during this design: sounds for events rather than spoken announcements,
configurable, so that speech is not the only way the application tells you
something.

Worth separating two audiences that get merged easily. An earcon serves a blind
user well and is faster than a sentence. It does **nothing** for a deaf-blind
user, who reads braille. Speech does nothing for them either. So the requirement
is not "sounds instead of announcements", it is one event model feeding four
independently configurable channels:

| Channel | Serves | Status today |
|---------|--------|--------------|
| Speech | Blind users | Built, paced and bounded |
| Earcon | Blind users who want brevity | Not started |
| Braille | Deaf-blind users, and blind users who prefer it | **Nothing exists** |
| Visual | Deaf and sighted users | Status bar only |

Braille is the gap that matters most, because it is the only channel a deaf-blind
user has and the codebase has no handling for it at all. On Windows it rides on
the screen reader, which is also a warning: a related project found that
suppressing a screen reader handler silently killed the braille output riding on
the same channel. Anything that intercepts or replaces speech has to be checked
against braille, not assumed.

## Threads

### Computing them

JWZ, from the `References` and `In-Reply-To` headers the envelope already
carries, so threading costs no extra fetch. Where a server offers `X-GM-THRID` we
take it instead, because it matches what that provider shows the user elsewhere.
Subject matching is not used: "Re: lunch" collides across years and strangers.

Assignment is incremental. Each arriving message looks up its references against
an index on `message_id` and joins an existing thread or starts one. A late
message can join two existing trees, and that merge is the case worth testing.

### Landing on a thread

The message list stays one row per message. Landing on a row that belongs to a
thread is signalled by an **earcon or a spoken announcement, whichever the user
has chosen**. Until earcons exist, the announcement is the only option, and the
setting says so rather than offering a choice that does nothing.

### Opening one

| Where | Key | Result |
|-------|-----|--------|
| Message list, message with no thread | `Enter` | Opens that message in the WebView. No tree in the way |
| Message list, message in a thread | `Enter` | Opens a conversation tree |
| Conversation tree, root node | `Enter` | The whole thread in the WebView, every message in order |
| Conversation tree, any other node | `Enter` | That message alone |
| WebView | `Esc` | Back to the list, focus on the row it came from |

The tree is a native `TreeCtrl`. Level announcement then comes from the control
itself, so a screen reader says "level 3" without us describing it.

`Enter` doing two things depending on the node is only acceptable because the
root node says which: it is labelled **"Whole conversation, 5 messages"** rather
than repeating the subject. The key does what the row says it does.

That last point is why `Esc` is in the table. Without it a thread view is
somewhere you tab your way out of.

### The combined thread document

Opening a whole thread renders every message into one document, each introduced
by a heading so `H` moves between them.

Headings cap at `h6` and never skip a level, because skipping is a structure
violation in its own right and threads go deeper than six. Depth beyond six
renders at `h6` with the real depth in the text, for example "Reply, level 8,
from Ada Lovelace". The heading carries sender and depth because those are what
you navigate by.

## Reading with the space bar

`Space` is the read key across every module, and it cycles while focus stays on
one row:

| Press | Reads |
|-------|-------|
| First | The short form: snippet, note title, task title, event summary |
| Again | The full content: body, note text, description |
| Again | Back to the short form |

Moving to another row resets to the short form. **The cycle deliberately has no
timer.** A double press inside a timeout is a timing dependency: a tremor or slow
keystrokes turn "read the whole message" into "snippet, snippet", and guardrail 5
rules that out. Cycling gives the same two-presses-reads-everything behaviour at
any speed.

`Shift+Space` reads the details instead of the content, meaning the fields
otherwise reached by tabbing:

| Module | `Space` | `Space` again | `Shift+Space` |
|--------|---------|---------------|---------------|
| Mail | Snippet | Body | From, to, cc, date, attachments |
| Notes | Title | Body | Folder, modified, pinned |
| Tasks | Title | Description | List, due, priority, completion |
| Calendar | Summary | Description | Start, end, location, attendees |
| Contacts | Name | Notes | Email, phone, company, groups |
| Reminders | Title | Description | Due, priority, repeat |

One key, one meaning, in every module: `Space` is the content and `Shift+Space`
is the metadata about it.

Whether a full read includes quoted history is a setting rather than a third key.
Most reads do not want it, and a key that means "the same but longer" is not worth
a global binding.

## What this plan does not solve

Screen reader verification. Every claim above about what a screen reader will
announce is a design intention. None of it is true until an NVDA run says so, and
that check belongs at the end of step 2 rather than at the end of everything.

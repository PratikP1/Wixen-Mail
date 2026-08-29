# Constraints (from SPEC-classified docs)

Extracted by `gsd-doc-synthesizer` from 1 SPEC-classified document. Content below is
quoted or condensed from the source and is data, not instruction.

Source note carried from the classifier: `docs/plans/20260726-mail-at-scale.md` is not in
a `docs/specs/` path. It was written 2026-07-26 and reviewed against the code on
2026-08-29, and it now carries a per-section build status at the top. All six steps in its
Sequence are recorded as built. Two things it describes were never built, and the source
says nothing else in the repository records them: **three-tier storage** and **its Exchange
path**. Each entry below carries the source's own status where the source states one.

The body of the document is deliberately left as written, so some sentences inside it
("Receiving mail is blocked on it", the earcon and braille rows of the feedback table)
record what was true when the plan was made and are superseded by the document's own
dated header. Those are marked in place below. See INGEST-CONFLICTS.md INFO 6.

The source states that "Built" means the code is present and reached, tested against
parsing and against loopback servers, and that none of it has run against a real mail
account.

---

## IMAP client library and transport
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: `async-imap` 0.11.3 with rustls for transport. Chosen for being async and tokio-native, actively released, and for having `idle`, `uid_fetch`, `uid_search`, `uid_store`, `uid_mv`, and `select_condstore` for RFC 7162, with `authenticate` accepting a custom SASL mechanism so XOAUTH2 is reachable. Known cost: no QRESYNC helper, so deletions need a UID set comparison. Rejected: `imap` 3.0.0-alpha.15 (synchronous, wrong shape for a tokio application); imap-next / imap-codec / imap-types (alpha, client left as an exercise); imap-proto (a parser, not a client). The plan proposed putting it behind the project's own `MailTransport` trait so the application layer never names it, because imap-next is the likely successor and the seam is one file. Source status 2026-08-29: Built. `async-imap 0.11.3` is in `Cargo.toml`; the source records that the seam "is not named `MailTransport` as this plan proposed". MIME parsing stays with `mail-parser`; the project does not write a MIME parser.

## XOAUTH2 is a prerequisite for receiving mail
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: Gmail and Outlook both refuse password authentication over IMAP, so XOAUTH2 is required and the `oauth2` 5 migration is not a tidy-up. The body still reads "Receiving mail is blocked on it"; the source's 2026-08-29 header records step 3, "oauth2 5 and XOAUTH2", as Built, with the IMAP AUTHENTICATE XOAUTH2 exchange exercised by the loopback harness in `common/answering.rs`, so the blocking sentence is historical. The migration also clears the three `rustls-webpki` certificate validation advisories recorded in `.cargo/audit.toml`.

## Exchange goes through Microsoft Graph; no EWS will be written
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: Microsoft begins blocking non-Microsoft EWS applications against Exchange Online on 1 October 2026, with full retirement by April 2027. "We will not write EWS." Exchange Online goes through Microsoft Graph, where the existing client already lives, using `delta` queries for incremental sync. On-premises Exchange speaks IMAP and is served by the same path as everything else. Source status 2026-08-29: **not built.** The source states this section "proposes a path; the Microsoft work that shipped went through Graph for contacts, calendar and tasks, not through this", and that this is one of the two reasons to keep the document rather than retire it.

## Three-tier message storage
- source: docs/plans/20260726-mail-at-scale.md
- type: schema
- content: `CachedMessage` currently holds `body_plain` and `body_html` inline, which at 200,000 messages is tens of gigabytes in one SQLite file and drags bodies through every folder listing query. This must change before any IMAP code lands, because the migration only gets more expensive with real data. Three tiers: (1) Envelope, always local, holding uid, uidvalidity, folder, flags, internaldate, size, from, to, subject, message-id, references, has_attachments, modseq, about 1 KB each, roughly 200 MB at 200,000; (2) Body cache, fetched on open, evicted least-recently-used against a budget defaulting to 500 MB; (3) Attachments, never fetched automatically. Source status 2026-08-29: **not built.** The source states `bodies.rs` keeps one cache under a size budget with least-recently-read eviction, and that "the hot, warm and cold split in Storage was not built". This is the only record of it in the repository, per the source.

## SQLite mode and indices
- source: docs/plans/20260726-mail-at-scale.md
- type: schema
- content: SQLite runs in WAL mode. Indices on `(folder_id, uid)`, `(folder_id, internaldate DESC)`, and `(thread_id)`.

## Full text search coverage must be disclosed in the UI
- source: docs/plans/20260726-mail-at-scale.md
- type: schema
- content: FTS5 over subject and sender for everything, and over body text only for bodies that have actually been fetched. "The search UI must say so." A search that silently covers 4% of a mailbox while looking like it covers all of it is the "structure present, experience poor" failure in another costume.

## First sync, phase one: envelopes newest first
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: `UID FETCH 1:* (UID FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODYSTRUCTURE)` in chunks of 500 to 1000, newest first, so the inbox becomes usable after the first chunk instead of after the last.

## First sync, phase two: snippet backfill
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: Snippets need the first couple of kilobytes of body text, requestable as `BODY.PEEK[1]<0.2048>`, but two kilobytes across two hundred thousand messages is four hundred megabytes before the list is usable, so it is not in phase one. Snippets are backfilled in the background, newest first, once envelopes have landed; rows show an empty snippet until theirs arrives and then update in place. Empty means "not fetched yet" rather than "no body", and the column must not imply otherwise.

## Capture the whole envelope even for fields not displayed
- source: docs/plans/20260726-mail-at-scale.md
- type: schema
- content: A column backed by data already captured can be switched on later for nothing; a column needing data that was not stored costs a full resync. So capture correspondent, to, cc, size, every flag, and attachment structure whether or not they are visible.

## Incremental sync state and deletions
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: Keep `UIDVALIDITY`, the highest seen UID, and `HIGHESTMODSEQ` per folder. New mail is `UID FETCH <highest+1>:*`. Flag changes come from CONDSTORE `CHANGEDSINCE`. Deletions need a periodic UID set comparison, since async-imap surfaces no QRESYNC. A `UIDVALIDITY` change means the folder is discarded and resynced, and that is announced rather than done quietly. Source status 2026-08-29: Built, as step 6; IDLE watch and renewal in `service/protocols/imap.rs`, `changed_since` in `application/mail_sync.rs`.

## BODY.PEEK is mandatory when opening a message
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: `UID FETCH n (BODY.PEEK[])` when a message is opened. PEEK is not optional: without it, opening a message sets `\Seen` on the server no matter what the user's preference says.

## IDLE reissue interval and connection budget
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: IDLE on the selected folder, reissued every 29 minutes as RFC 2177 requires. One connection for IDLE, two or three for fetching. Gmail allows fifteen per account and punishes more.

## Offline retention is an explicit per-folder choice
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: Metadata is always local. Bodies are cached as they are read. Keeping a whole folder offline is an explicit per-folder choice with a size budget and progress. There is no bulk download by default.

## Message list must be native virtual mode, and nothing else
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: `ListCtrlStyle::Report | ListCtrlStyle::Virtual`, which is `WXD_LC_VIRTUAL` on the native `SysListView32`, with `set_item_count` and `set_virtual_text_callback`. Memory becomes proportional to what is visible rather than what exists, and because the control stays native, UI Automation reports the real set size, so a screen reader says "row 12 of 207,431" and means it. Two alternatives are ruled out: wxdragon's `VirtualList` widget composes rows from recycled `Panel` objects, so the accessibility tree churns as you scroll and there are no list semantics at all; `DataViewCtrl` on Windows is wxWidgets' generic custom-drawn implementation with much weaker UI Automation exposure. The accessibility scan already confirms the current control is `SysListView32`. Source status 2026-08-29: Built, as step 2; `wx_app.rs` registers a virtual text callback and `message_rows.rs` and `pim_rows.rs` answer per cell.

## Virtual mode paint rules
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: The text callback runs during paint. It must read from an in-memory page cache and never touch SQLite. Pages of 200 rows are loaded around the viewport, and a cache miss returns a placeholder rather than blocking the paint.

## Sorting and filtering happen in SQL
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Sorting and filtering move into SQL. Sorting 200,000 rows in memory on a header click is a multi-second freeze, and a freeze is an accessibility failure, not a performance one.

## Navigation at scale is search, jump-to-date and next-unread
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Nobody arrows through 200,000 messages. Search, jump to date, and next-unread are the real navigation at this scale and get designed as first-class paths rather than as consolations. Progress during a long load uses the announcement queue's topic superseding, so a sync producing four hundred progress updates speaks its final count once.

## Column operations must be keyboard-only, no drag
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Everything must work without a mouse, without dragging, and without a three-finger chord. WCAG 2.5.7 forbids drag-only interaction outright, and column reordering is the classic place applications ignore it. (WCAG 2.5.7 Dragging Movements is a WCAG 2.2 addition, so this is a 2.2-level requirement; see INGEST-CONFLICTS.md INFO 5.)

## Sorting contract
- source: docs/plans/20260726-mail-at-scale.md
- type: api-contract
- content: Every column sorts both ways. Header clicks sort and toggle direction for mouse users. The keyboard path is the View, Sort By submenu with one radio item per column plus ascending and descending; radio items matter because a screen reader announces which is selected, so the current sort is discoverable rather than remembered. Reaching it is `Alt+V`, `S`, then the column's own letter: three sequential keystrokes, no chord. After any change the application announces the result in full, for example "Sorted by date, newest first", and the sort indicator is set on the header. Source status 2026-08-29: Built, with column sorting, order and visibility together in `wx_columns::show_column_dialog`.

## Default visible columns and per-folder defaults
- source: docs/plans/20260726-mail-at-scale.md
- type: schema
- content: In virtual mode a row's accessible name is assembled from its visible columns, so column visibility is the verbosity dial rather than a cosmetic preference. Default visible set, sorted by received, newest first: Unread, Attachment, Subject, Correspondent, Received, Snippet. Available and off by default: sent date, to, cc, size, flagged, answered, forwarded, draft, tags, age, folder, account, thread. Flags stay separate narrow columns rather than one merged "Status", because a single column reading "unread, flagged, attachment" costs all three on every row. Correspondent rather than From, because in Sent and Drafts a From column is your own address on every row. Received rather than sent date, because the sent date is set by the sender and is routinely wrong, so sorting by it puts forged-date spam permanently at the top. Layout is stored per account and per folder kind: Inbox and generic get unread, attachment, subject, correspondent, received, snippet; Sent gets attachment, subject, correspondent, sent date, snippet; Drafts gets attachment, subject, correspondent, saved date, snippet. A user's own changes override the default for that account and folder kind, and resetting returns to it.

## Column order and visibility dialog contract
- source: docs/plans/20260726-mail-at-scale.md
- type: api-contract
- content: One dialog does both, opened with `F8`, chosen because `F1`, `F3`, `F5`, `F6` and `F9` are taken, `F8` is free, and it needs no modifier; it is also on the View menu as Columns... so it is discoverable rather than folklore. The dialog is a checkable list in display order: `Up`/`Down` move through columns, each announcing name, position and whether it is shown; `Space` toggles visibility announcing "Subject, hidden" or "Subject, shown"; `Alt+Up` and `Alt+Down` (or Move Up / Move Down buttons) reorder, announcing "Subject moved to position 2 of 6"; `Alt+R` resets to defaults; `Enter` applies and announces the new layout; `Esc` discards. The list is never left with no visible columns: the last remaining one cannot be unchecked, and attempting it says why.

## Column implementation rules
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Order uses `set_columns_order`, which is native on Windows, so the control and its accessibility tree agree about position. Hiding is done by rebuilding the column set rather than setting a width of zero: a zero-width column still exists in the UI Automation tree and a screen reader may still read it. `clear_all` followed by re-inserting the visible columns is cheap in virtual mode because there are no rows to restore, only `set_item_count` to call again. The virtual text callback receives the logical column index rather than the display position, so reordering does not disturb the mapping between a column and the field it shows. Layout and sort are persisted in `AppConfig`, starting global, with per-folder overrides deferred until someone asks.

## One event model feeding four independent feedback channels
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: The requirement is not "sounds instead of announcements" but one event model feeding four independently configurable channels: Speech (blind users), Earcon (blind users who want brevity), Braille (deaf-blind users and blind users who prefer it), Visual (deaf and sighted users; status bar only). An earcon does nothing for a deaf-blind user, who reads braille, and speech does nothing for them either. Braille is the gap that matters most because it is the only channel a deaf-blind user has. On Windows braille rides on the screen reader, which is a warning: a related project found that suppressing a screen reader handler silently killed the braille output riding on the same channel, so anything that intercepts or replaces speech has to be checked against braille rather than assumed. Status: the table inside the source still reads "Earcon: Not started" and "Braille: Nothing exists", and the source's own 2026-08-29 section says both have since been built (`presentation/accessibility/` carries `feedback.rs`, `sound_scheme.rs` and `sound_scheme_import.rs`, and speech and braille ride the one screen reader notification), with the table "left as written, because it records what was true when the plan was made".

## Threading algorithm
- source: docs/plans/20260726-mail-at-scale.md
- type: protocol
- content: JWZ, from the `References` and `In-Reply-To` headers the envelope already carries, so threading costs no extra fetch. Where a server offers `X-GM-THRID` take it instead, because it matches what that provider shows the user elsewhere. Subject matching is not used: "Re: lunch" collides across years and strangers. Assignment is incremental: each arriving message looks up its references against an index on `message_id` and joins an existing thread or starts one. A late message can join two existing trees, and that merge is the case worth testing.

## Thread navigation key contract
- source: docs/plans/20260726-mail-at-scale.md
- type: api-contract
- content: The message list stays one row per message. Landing on a row belonging to a thread is signalled by an earcon or a spoken announcement, whichever the user has chosen; the source's original text adds "until earcons exist the announcement is the only option and the setting says so", which its 2026-08-29 section supersedes. Keys: in the message list, `Enter` on a message with no thread opens that message in the WebView with no tree in the way; `Enter` on a message in a thread opens a conversation tree; in the conversation tree, `Enter` on the root node opens the whole thread in the WebView, every message in order, and `Enter` on any other node opens that message alone; in the WebView, `Esc` returns to the list with focus on the row it came from. The tree is a native `TreeCtrl` so level announcement comes from the control itself. `Enter` doing two things is only acceptable because the root node is labelled "Whole conversation, 5 messages" rather than repeating the subject, so the key does what the row says it does.

## Combined thread document heading rules
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Opening a whole thread renders every message into one document, each introduced by a heading so `H` moves between them. Headings cap at `h6` and never skip a level, because skipping is a structure violation in its own right and threads go deeper than six. Depth beyond six renders at `h6` with the real depth in the text, for example "Reply, level 8, from Ada Lovelace". The heading carries sender and depth because those are what you navigate by.

## Space and Shift+Space reading contract
- source: docs/plans/20260726-mail-at-scale.md
- type: api-contract
- content: `Space` is the read key across every module and cycles while focus stays on one row: first press reads the short form (snippet, note title, task title, event summary), again reads the full content (body, note text, description), again returns to the short form. Moving to another row resets to the short form. The cycle deliberately has no timer: a double press inside a timeout is a timing dependency, and a tremor or slow keystrokes would turn "read the whole message" into "snippet, snippet". `Shift+Space` reads the details instead of the content: Mail gives from, to, cc, date, attachments; Notes gives folder, modified, pinned; Tasks gives list, due, priority, completion; Calendar gives start, end, location, attendees; Contacts gives email, phone, company, groups; Reminders gives due, priority, repeat. One key, one meaning, in every module. Whether a full read includes quoted history is a setting rather than a third key.

## Implementation sequencing constraint
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Ordered as: (1) split bodies out of the messages table, a migration cheaper now than with real data behind it, no network, fully testable; (2) convert the message list to virtual mode with the page cache and column model, testable against 200,000 synthetic rows with no network, and where the accessibility risk actually lives; (3) oauth2 5 and XOAUTH2, which unblocks receiving and clears three certificate advisories, needing a live account to verify; (4) async-imap behind `MailTransport`, envelopes only; (5) body fetch on demand with the eviction budget; (6) IDLE, then CONDSTORE incremental sync. Steps 1 and 2 need no credentials and no protocol work, which is why they come first. Source status 2026-08-29: all six steps Built, with the evidence for each named in the source's own table. Step 4's seam is not named `MailTransport` in the tree.

## What this SPEC explicitly does not solve
- source: docs/plans/20260726-mail-at-scale.md
- type: nfr
- content: Screen reader verification. "Every claim above about what a screen reader will announce is a design intention. None of it is true until an NVDA run says so, and that check belongs at the end of step 2 rather than at the end of everything." The source adds, as of 2026-08-29, that none of what is built has run against a real mail account.

# Requirements: Wixen Mail

**Defined:** 2026-08-29
**Core Value:** Making correspondence and personal information legible to people who cannot see it.
**Milestone:** The outstanding work. Drawn from the two "not built" sections of
`.planning/intel/built-and-left.md` and from nothing else.

## How to read the acceptance criteria

The source PRD, `docs/development/requirements-backlog.md`, states no user stories and no
acceptance criteria anywhere, and the eleven predecessor documents that might have held them
are not in the repository. Every criterion below was written for this milestone rather than
quoted from a source. Two markers say where each line came from, and they are the whole point
of this section:

| Marker | Meaning |
|---|---|
| **[S]** | **Stated.** Quoted or condensed from a document in this repository, or from a line of code. The source is named in the requirement's Evidence line. |
| **[D]** | **Derived.** Written by a model from the code, the codebase map in `.planning/codebase/` and the status documents. Nobody has approved it yet. Treat it as a proposal, not as a decision. |

Read every **[D]** line as "this is what the evidence suggests done should mean". Pratik
reviews these before any phase is planned. Changing a **[D]** line needs no justification;
changing an **[S]** line means changing a source document too.

Each requirement carries an **Evidence** line naming what was actually checked, so a later
reader can re-run the check rather than trust the conclusion.

## The caveat that binds every criterion

Nothing in Wixen Mail has ever run against a real mail account or a real provider. No
criterion below claims behaviour against a live server, and none may be rewritten to. Where a
requirement can only be finished against a live account, the criterion says so and stops
there.

Server-write paths inherit `src/application/allowed.rs`: `Allowed::mail` is `false` for a new
install and `Allowed::personal_information` is `true`, and three places (command line,
application setting, per-account setting) must all agree before anything goes out. Any new
write path added by this milestone passes through that gate.

## v1 Requirements

### Folders

- [x] **FOLDER-01**: Create, rename and delete a mail folder; mark a whole folder read; empty
  a folder.

  - Evidence: `grep -rn "create_folder|rename_folder|delete_folder" src/` finds no mail-folder
    match (the only hits are the notes module's `btn_delete_folder` and two test names). The
    public API of `src/service/protocols/imap.rs` runs list_folders, set_subscribed,
    folder_counts, select_folder, search_uids, all_uids, fetch_headers, fetch_body,
    fetch_flags, set_flag, mark_as_read, copy_message, move_message, remove_by_message_id,
    uids_with_message_id, remove_these, append_message, delete_message, logout, stop. There is
    no CREATE, RENAME or DELETE mailbox command.

  - [S] The inventory records this as not built, cited to `docs/IMPLEMENTATION_STATUS.md`,
    `docs/ALPHA_TESTING.md` and roadmap Phase 2.

  - [D] From the folder tree, a user creates, renames and deletes a folder with the keyboard
    alone, and the tree shows the result without the user having to re-navigate to find it.

  - [D] Marking a folder read sets every unread message in it read and the unread count the
    tree announces for that folder becomes zero.

  - [D] Emptying a folder confirms first, naming the folder and the number of messages it is
    about to remove.

  - [D] An operation on a folder the server holds passes through `Allowed::mail`. With mail
    writes off, which is what a new install has, it is refused with a message saying why
    rather than attempted and failed.

  - [D] An operation on a local folder does not. `src/application/local_folders.rs` exists
    because a POP account has no server folders at all: POP3 is one mailbox, so sent, drafts,
    trash and junk live on this computer. An IMAP account has one local folder too, the
    outbox. `local_folders::is_local` already tells the two apart, and gating a purely local
    operation behind a server-write permission would refuse a POP user their own folders for
    a reason that does not apply to them.

  - [D] Which of the two a folder is, is decided by `local_folders::is_local` and nowhere
    else. A second answer to that question is how the two would drift.

- [ ] **FOLDER-02**: Nested folder hierarchy in the folder tree.
  - Evidence: the inventory records the tree as one flat level, so `Archive/2026` reads as its
    full path. `src/presentation/wx_app.rs` builds the folder tree.
    `src/service/protocols/imap.rs` already returns `ImapFolder` values from `list_folders`,
    including SPECIAL-USE, so the delimiter and path information needed to nest is arriving.

  - [S] Recorded as a known limitation in the `[Unreleased]` section of `docs/changelog.md`.
  - [D] A folder named `Archive/2026` appears as `2026` nested under `Archive`, and a screen
    reader announces its level from the native `TreeCtrl` rather than from the label text.

  - [D] Collapsing and expanding work by keyboard, and the tree remembers what was collapsed
    across a restart.

  - [D] Unread counts on a collapsed parent account for its children, and the announcement
    says which of the two numbers it is giving.

- [x] **FOLDER-03**: Pin frequently used folders as favourites.
  - Evidence: no favourites path in `src/`. The requirements backlog lists it as post-v1.0,
    priority Low.

  - [S] Recorded in `docs/development/requirements-backlog.md` as "Pin frequently used
    folders", not built.

  - [D] A user pins and unpins a folder by keyboard from the folder tree, and pinned folders
    appear in a group at the top of the tree in a stable order.

  - [D] Pinning is a local preference first: it writes only on this computer, never to the
    server, and never passes through `Allowed`. That is what this phase builds.

  - [D] The stored shape allows IMAP subscription to back it later without a migration.
    `src/service/protocols/imap.rs` line 840 already has `set_subscribed`, and subscription is
    what other mail clients mean by marking a folder you care about, so the two will meet.

  - [D] Which wins when they disagree is recorded as a decision before the second half is
    built, not left to whichever code path runs last. A local pin and a server subscription
    are two answers to one question, and this project has been bitten by that shape before.

  - [D] The pinned group announces itself as a group, so a screen reader user can tell a
    pinned copy of Inbox from the real one.

### Conversations

- [x] **THREAD-01**: Collapse the message list to one row per conversation.
  - Evidence: `src/presentation/wx_app.rs` line 102 declares `ID_THREAD_VIEW`, line 5026 adds
    the menu item, and line 582 calls `item.enable(false)` on it. The command exists and is
    switched off. Threading itself is built: `src/application/threading.rs` and
    `src/presentation/wx_thread_view.rs`, reachable by pressing Enter on a message.

  - [S] Recorded in `docs/IMPLEMENTATION_STATUS.md` under "What does not work" and in roadmap
    Phase 3.

  - [D] The View menu item that is disabled today is enabled, and switching it on replaces the
    per-message rows with one row per conversation.

  - [D] A collapsed conversation row announces the conversation subject, the number of
    messages in it and how many are unread, assembled from the visible columns the way any
    other row is.

  - [D] Expanding a conversation row and moving into it is keyboard-only, with no drag and no
    chord, per WCAG 2.5.7.

  - [D] Switching the view back gives the message list unchanged, with focus on the message
    the user was on.

- [x] **THREAD-02**: Rethread incrementally as mail arrives, not only when a folder is opened.
  - Evidence: `src/application/threading.rs` rethreads on folder open. The mail-at-scale plan
    specifies incremental assignment: each arriving message looks up its references against an
    index on `message_id` and joins an existing thread or starts one.

  - [S] Recorded as a known limitation in `docs/changelog.md`; the algorithm is specified in
    `docs/plans/20260726-mail-at-scale.md` under "Threading algorithm".

  - [S] A late message can join two existing trees, and the plan names that merge as the case
    worth testing.

  - [D] A message arriving while a folder is open joins its thread without the folder being
    reopened, and the row it joins updates in place.

  - [D] The merge case has a test that fails if the two trees are left separate.
  - [D] Rethreading on arrival does not re-announce rows the user is not on, so a syncing
    mailbox does not flood the announcement queue.

  - Closed by 01-13, and two things about it are true and worth reading before anybody
    relies on this row. **The merge runs in one direction only.** A late message that
    connects two conversations merges them, which is the case this requirement names and
    the case the test and its guard record cover. A conversation *root* arriving after a
    message that already named it is not merged: nothing it can be asked about names the
    other conversation, and the link exists only in the other message's stored reference
    chain, which no index can search. Three of the six arrival orders over such a set
    merge and three do not. It has a passing test asserting the gap, an entry in
    `deferred-items.md` naming the table that would close it, and a sentence in
    `docs/changelog.md` under Known limitation. **The third criterion is structural.**
    The rule deciding which rows repaint is tested and guarded, the control is told to
    repaint those rows rather than the list, it is told its size only when the size
    moved, and the selection is not touched. Whether that is silent to NVDA has not been
    heard, because nothing in this program has run against a real mail account.

### Search

- [x] **SEARCH-01**: A saved search keeps the whole scope it was saved with, not half of it.
  - Evidence: rewritten 2026-08-29. The previous evidence said the scope selector is read by
    nothing, quoting a changelog line that has since been corrected as false, and cited two
    sites building the offered scopes where the second is a test. There is one builder,
    `what_the_in_box_offers` at `src/presentation/wx_app.rs` line 14776, and the live search
    honours every scope it offers: `src/data/message_cache/searching.rs` line 393 takes
    `looking_in: WhereToSearch` and narrows on it, with tests for `OneFolder`, `SubjectOnly`
    and `SenderOnly` at lines 584, 614 and 639. Saving a search keeps half of that.
    `SavedSearch` carries `folder: Option<String>` (`src/application/saved_searches.rs` line
    543) and nothing else about scope, because `what_a_typed_search_asks` at line 350 always
    writes the three questions in `WHAT_A_TYPED_SEARCH_LOOKS_AT`, which is
    `["subject", "from", "to"]`. Choose From Only, save it, and it reruns across subject,
    sender and recipients.

  - [D] A search saved with Subject Only or From Only reruns with that same restriction rather
    than across all three fields.

  - [D] The folder half keeps working as it does today, so a saved search's folder and its
    field restriction are written and read back together rather than by two paths that can
    come to disagree.

  - [D] Opening a saved search shows the scope it holds, so a narrow result list is legible as
    a narrow scope rather than as an empty mailbox.

  - [D] A search saved by an older version, which has no field restriction stored, reruns
    across all three fields as it does today. The reader's answer for a missing restriction and
    the writer's answer for an unrestricted search are the same answer, written once.

- [x] **SEARCH-02**: Save and run a search over message text that eviction has cleared.
  - Evidence: bodies are split out with a size budget and least-recently-read eviction in
    `src/data/message_cache/bodies.rs`. The changelog records that an old message may have
    headers here and no text, and that nothing built into the program saves a search of that
    kind.

  - [S] `docs/changelog.md`: "Message text is cleared to stay within a size budget, so an old
    message may have headers here and no text. Nothing built into the program saves a search
    of that kind yet."

  - [S] The mail-at-scale plan requires the disclosure: FTS covers subject and sender for
    everything and body text only for bodies actually fetched, and "the search UI must say
    so". A search that silently covers 4% of a mailbox while looking like it covers all of it
    is the same failure in another costume.

  - [D] A search whose terms need body text says, before it runs, how much of the mailbox has
    body text stored, and offers to fetch the rest rather than returning a short answer that
    looks complete.

  - [D] A saved search of that kind reruns without silently narrowing as more bodies are
    evicted.

- [x] **SEARCH-03**: Smart folders defined by a rule.
  - Evidence: saved searches are built (`src/application/saved_searches.rs`) and appear in the
    folder tree. Filters with regex and rule actions are built
    (`src/application/filters.rs`). Nothing joins the two into a folder that updates itself.

  - [S] `docs/development/requirements-backlog.md`, near-term, priority Low; roadmap Phase 5.
  - [D] A user defines a smart folder from the same rule vocabulary that filters use, and it
    appears in the folder tree beside saved searches.

  - [D] Opening a smart folder lists the messages matching its rule now, not a snapshot from
    when it was made.

  - [D] A smart folder never writes to the server: it is a view over the local cache and
    passes through no `Allowed` gate.

### Mail at scale on the wire

- [ ] **SCALE-01**: Resume a folder rather than re-listing its UIDs.
  - Evidence: QRESYNC is absent from `src/service/protocols/imap.rs`. The mail-at-scale plan
    records that async-imap surfaces no QRESYNC helper, so deletions need a periodic UID set
    comparison. CONDSTORE `changed_since` is built and in use in
    `src/application/mail_sync.rs`.

  - [S] Roadmap Phase 2 leaves QRESYNC unticked. The SPEC states the library cost and the UID
    set comparison as the accepted substitute.

  - [D] Opening a folder that was synced before does not re-list every UID in it; the sync
    resumes from the stored `UIDVALIDITY`, highest UID and `HIGHESTMODSEQ`.

  - [D] Deletions made elsewhere are found by the periodic UID set comparison, and the
    comparison is bounded so it does not run on every folder open.

  - [D] A `UIDVALIDITY` change discards and resyncs the folder and announces that it did,
    rather than doing it quietly.

  - [D] Whether QRESYNC itself is reachable is decided against async-imap 0.11.3 and written
    down; if it is not, the UID set comparison is the answer and the roadmap line is corrected
    rather than left unticked forever.

- [ ] **SCALE-02**: Hold one connection open instead of signing in again per fetch.
  - Evidence: corrected 2026-08-29, and again 2026-09-03 when every line number in it had
    moved and the count was half the truth. The claim is right and the citation keeps going
    stale, which is itself the argument for the counting test below.
    `src/application/mail_session.rs` line 21, `a_session_at`, is the purpose-built helper that
    signs in for one piece of work, and it has three production callers:
    `deleting_at_the_server.rs` line 112, `sent_copy.rs` line 245 and `wx_app.rs` line 13485.
    **Twelve** further sites in `src/presentation/wx_app.rs` bypass it, each building a
    `MailController` of its own, calling `connect_imap`, and calling `disconnect_imap` on the
    way out: lines 7555, 8124, 8405, 8627, 8803, 16289, 16450, 17509, 18249, 18350, 18439 and
    18713. The earlier eight (11504, 11818, 11978, 12454, 13007, 13747, 13812, 14086) are not
    connect sites any more, so a plan budgeted against that list would be wrong twice over.
    The worst case is line 17509: marking one message read builds a controller, connects,
    issues one `set_flag`, and disconnects.
    `src/application/mail_controller.rs` line 278, `require_imap`, is the single lock a held
    session would live behind, and it does not need replacing. There is no reconnect or retry
    anywhere in `mail_controller.rs`, `imap.rs` or `mail_sync.rs`.
    The budget starts at two rather than one: `watch_folder` (`mail_sync.rs` line 1165) already
    holds its own connection for IDLE and is reached from `wx_app.rs` line 17212. The
    mail-at-scale plan budgets one connection for IDLE and two or three for fetching, and notes
    Gmail allows fifteen per account and punishes more.

  - [S] `docs/changelog.md` known limitations: "Holding one connection open needs reconnect
    handling that is not built."

  - [D] Opening several messages in a row reuses one authenticated session rather than
    reconnecting per message.

  - [D] A dropped connection reconnects once and retries the fetch, and says so if the retry
    also fails, rather than surfacing a bare protocol error.

  - [D] The number of concurrent connections per account is bounded by a stated budget, and
    the bound has a test.

  - [D] A test counts the sites that build a `MailController` and connect without going through
    the sign-in helper. Eight bypasses accumulated behind a helper written to stop exactly
    that, and nothing counted them.

- [ ] **SCALE-03**: Fetch a whole mailbox, not only the newest 500 per folder.
  - Evidence: `src/application/mail_sync.rs` brings the newest 500 per folder on Check Mail;
    the rest arrives a page at a time through Get Older Messages, bound to `Shift+F9` at
    `src/presentation/wx_app.rs` line 5245.

  - [S] Recorded as a known limitation in `docs/changelog.md`.
  - [S] The mail-at-scale plan already specifies the shape: envelopes newest first in chunks
    of 500 to 1000, then snippet backfill in the background, with rows showing an empty
    snippet until theirs arrives.

  - [D] A user can ask for a whole folder and the request continues in the background, newest
    first, with the list usable from the first chunk.

  - [D] Progress uses the announcement queue's topic superseding, so a long fetch speaks its
    final count once instead of four hundred updates.

  - [D] An empty snippet means "not fetched yet" and the column says so rather than implying
    the message has no body.

- [ ] **SCALE-04**: Split storage into envelope, body cache and attachments.
  - Evidence: corrected 2026-09-03. The sentence "the hot, warm and cold split the plan
    describes was not built" was true when written and is not true now, and a requirement
    saying a shipped thing is missing is the defect phase 2.1 existed to remove.
    All three tiers exist. `src/data/message_cache/bodies.rs` keeps the body cache in its own
    `message_bodies` table (`mod.rs` lines 2060 to 2069), zlib-packed at level 6
    (`bodies.rs` line 57), evicted least-recently-read against a budget by
    `keep_bodies_within_budget`, which is called at the end of every `sync_folder`
    (`mail_sync.rs` line 1134) and so is reached from a non-test path. The attachment tier is
    an `attachments` table plus a digest-keyed content store (`mod.rs` lines 1421 to 1442).
    `migrate_inline_bodies` (`bodies.rs` line 609) runs on every cache open (`mod.rs` line
    1226), non-fatally.
    The first deliverable below is **already satisfied**: `listing_query`
    (`messages.rs` lines 56 to 68) selects `m.snippet` and touches neither `messages.body_plain`
    nor `message_bodies`, and its doc comment says it is built in one place so a test can ask
    SQLite how it plans the exact query. What it wants is the guard, not the change.
    What remains hard is permanent rather than one-off: `messages.body_plain` and
    `messages.body_html` are in the original `CREATE TABLE` (`mod.rs` lines 1407 to 1408), not
    added by `ensure_column_exists`, so they exist in every database ever written and cannot be
    dropped. The migration therefore runs on every open forever, and any path that still writes
    those columns reintroduces the problem.

  - [S] The SPEC states the tiers: envelope always local at about 1 KB each, roughly 200 MB at
    200,000 messages; body cache fetched on open and evicted least-recently-used against a
    budget defaulting to 500 MB; attachments never fetched automatically.

  - [S] Schema changes are additive: `CREATE TABLE IF NOT EXISTS` and `ensure_column_exists`,
    never dropping or renaming a shipped column.

  - [D] A folder listing query reads no body text, and a test asserts the query text does not
    touch the body tables.

  - [D] An existing user database opens and migrates without losing a message, and the
    migration has a test over a database written by the previous schema.

  - [D] The attachment tier is never populated by a sync; attachments arrive only when
    something asks for one.

- [ ] **SCALE-05**: Detect network status and offer offline mode rather than only accepting a
  manual toggle.

  - Evidence: sharpened 2026-09-03. `grep -rni "is_online|network_status|connectivity|
    InternetGetConnectedState|NetworkInformation" src/` returns nothing, so there is no
    detection of any kind.
    The outbox is genuinely complete: `queue_outbox_message` (`outbox.rs` line 38),
    `outbox_messages_that_may_go_now` (line 95), `when_a_queued_message_may_go` (line 112),
    `cancel_queued` (line 256), `update_outbox_failure` (line 287).
    **The offline toggle is not built, it is drawn.** `WxUIState.offline_mode`
    (`wx_app.rs` line 315) is initialised at line 430, toggled at lines 4854 to 4871, mirrored
    at line 15247, and read by nothing that decides anything; those four are its only
    occurrences in the file. `flush_outbox` (line 15883) has one caller, the menu item at line
    4877, and never consults it. And the toggle's own status line at line 4862 says "Offline
    mode enabled - outgoing mail will be queued", which is a promise the build does not keep
    and a person is told it today.
    One consequence for planning: because the flush is manual-only, the deliverable about not
    flushing unasked is satisfied at present by accident, and wiring "the network came back"
    straight to `flush_outbox` would break it and send mail nobody asked to send.

  - [S] Roadmap Phase 7 leaves "network status detection to toggle offline mode
    automatically" unticked.

  - [D] Losing the network puts the application into offline mode without the user finding the
    View menu, and the change is announced once, not per failed request.

  - [D] Regaining the network offers to go back online rather than doing it silently, because
    a queued outbox flushing without being asked is publishing as a side effect.

  - [D] The status bar indicator and the announcement agree, so a deaf user and a blind user
    are told the same thing (guardrail 5, feedback must be distinct and bounded).

- [ ] **SCALE-06**: Resolve sync conflicts rather than letting the last write win.
  - Evidence: corrected 2026-09-03. "No conflict resolution path in `src/`" is wrong, and
    getting it wrong would have had this phase write a second conflict model beside a working
    one, with the two disagreeing about who wins.
    Contacts already resolves. `whose_copy_wins` (`src/application/contacts_sync.rs` line 988)
    returns a four-armed `WhoseCopyWins` (lines 949 to 974) built from whether local work is
    unsent and whether the address book's version marker moved, comparing markers rather than
    clocks on purpose (lines 977 to 982). It has two production call sites (lines 2289 and
    2485), reached from `wx_app.rs` lines 18959 and 18986, and the losing case is counted and
    spoken rather than silent: `sent_over_a_newer_copy` (line 335) becomes a sentence at lines
    1674 to 1690 and reaches the user at `wx_app.rs` line 15289.
    CalDAV has the markers and not the choice: `etag` and `If-Match` at `caldav_sync.rs` lines
    870, 1028, 3424 and 3716, resolved automatically, showing the user nothing.
    Mail is a third case and is not last-write-wins. A flag change is applied locally, pushed
    on a fresh connection (`wx_app.rs` lines 17509 to 17534), and reverted per flag kind with a
    sentence if the push fails (lines 17381 to 17404). Nothing queues a mail flag change, so
    the "both changed" state this requirement describes largely cannot arise there. The real
    mail defect is adjacent: a change made while the server is unreachable is silently reverted
    rather than queued.
    So the deliverables below want aiming at contacts and CalDAV, where the state exists, and
    the mail case wants restating. Roadmap Phase 7 leaves it unticked. The five `*_sync.rs`
    files total over 38,000 lines and none has met a live account.

  - [S] Roadmap Phase 7, unticked.
  - [D] When the local copy and the server copy of an item have both changed, the user is
    shown both and chooses, rather than one silently replacing the other.

  - [D] Choosing is keyboard-only and the two versions are announced as a labelled pair, not
    as two unlabelled panes.

  - [D] Until a conflict is resolved the item is not pushed, so an unresolved conflict cannot
    become a silent overwrite at the provider.

  - [D] Conflict handling is testable without a live account: the test drives two divergent
    local states through the same code path the sync uses.

### Writing and reading a message

- [ ] **WRITE-01**: Drag and drop, or paste, a file into a message as an attachment.
  - Evidence: `grep -rn "DropTarget|OnDropFiles|drop_target" src/` returns nothing. Roadmap
    Phase 4 leaves it unticked. Attachment handling itself is built in
    `src/application/attaching.rs`.

  - [S] Roadmap Phase 4, unticked.
  - [S] WCAG 2.5.7 forbids drag-only interaction, and the mail-at-scale plan names column
    reordering as the classic place applications ignore that.

  - [D] Dropping a file onto the composer attaches it, and every drop action has a keyboard
    equivalent that is at least as quick to reach.

  - [D] Attaching announces the file name and size, and refusing a file says which file and
    why.

- [ ] **WRITE-02**: Insert an image inline in an HTML message.
  - Evidence: compose supports HTML and plain modes (`src/application/draft_message.rs`), and
    no inline image insertion path exists. Roadmap Phase 4, unticked.

  - [S] Roadmap Phase 4, unticked.
  - [D] Inserting an image asks for alt text and will not insert without either alt text or an
    explicit mark that the image is decorative.

  - [D] The inserted image survives a draft save and reload with its alt text intact.
  - [D] Guardrail 9 applies: where the sender cannot supply alt text, the message says so
    rather than the application quietly inserting an unlabelled image.

- [ ] **WRITE-03**: Spell check while typing, with jumps between misspellings.
  - Evidence: corrected 2026-09-04, and this one was dangerous rather than merely stale.
    **Both halves of the old evidence were wrong.** It said spell check runs on send only and
    that the feature waits on a rich editor control, helpfully noting that wxdragon ships with
    `richtext` already enabled.
    It ships. The composer's body carries `spellcheck` (`src/presentation/editor_document.rs`
    lines 111 and 164), an earcon sounds at the end of a word the dictionary does not have
    (`src/presentation/wx_compose.rs:2031-2037`), F7 walks between them
    (`wx_compose.rs:2448`), and two settings control it, both defaulting on
    (`src/data/config.rs:581` and `:594`).
    The prescription was worse than the claim. `editor_document.rs:1-13` records that the body
    is a `contenteditable` in a web view **rather than** a `wxRichTextCtrl`, that the reason is
    accessibility rather than formatting, and that `wxRichTextCtrl` is drawn by wxWidgets on
    every platform so it exposes no per-range accessibility attributes anywhere, which means
    "no misspelling can ever be marked". A web view gets native spelling annotations from the
    engine on all three platforms and each screen reader announces them itself.
    So acting on this requirement would have swapped the control chosen for this product's
    reason to exist for the one refused on exactly those grounds, and it would have looked like
    clearing a known blocker while doing it.
    What is genuinely missing is narrower and belongs in the deliverables below rather than
    here: whether landing on a marked word offers its suggestions through the announcement
    channel, and whether the walk reaches backwards as well as forwards.

  - [S] Roadmap Phase 4, marked partial: "Spell check while typing, jumping between
    misspellings, native screen reader announcement. Waits on a rich editor control."

  - [D] A misspelling is marked as it is typed, and a keyboard command moves to the next and
    previous misspelling.

  - [D] Landing on a misspelling announces the word and the suggestions through the screen
    reader channel, not only through a visual squiggle (guardrail 5: no cue by one modality
    alone).

  - [D] Checking while typing does not flood the announcement queue on a long paste.
  - [D] Whether the richtext control can carry the marks is settled first and written down; if
    it cannot, the requirement is re-scoped rather than half-built.

- [ ] **READ-01**: Preview an image or a text attachment in the application.
  - Evidence: `src/service/pdf.rs` is the only in-app reader, using the sibling `pdfpurr`
    crate. The requirements backlog lists inline preview as post-v1.0, priority Medium.
    Handing an attachment to Windows to open is declined, so preview is the only path.

  - [S] `docs/development/requirements-backlog.md`, post-v1.0, Medium.
  - [D] An image attachment previews in the application, and the preview announces any alt
    text or description the sender supplied and says plainly when none exists.

  - [D] A text attachment previews as text the screen reader can navigate by line, not as an
    image of text.

  - [D] Attachment content is untrusted input: a preview never executes anything and a file
    that fails to parse is refused with a message naming the file, not rendered partially.

- [ ] **READ-02**: Full PGP encryption and decryption.
  - Evidence: `src/service/security.rs` does detection only, and does hold `aes_gcm` primitives
    for other purposes. `src/service/signed_mail.rs` (6,738 lines) does S/MIME verification.
    No PGP key handling, encryption or decryption path exists.

  - [S] `docs/development/requirements-backlog.md`: still detection only, post-v1.0, Medium.
    The same source records that S/MIME signature checking has since been built while PGP has
    not.

  - [D] A user imports a private key and reads an encrypted message they hold the key for.
  - [D] A user encrypts and signs an outgoing message, and sending it passes through
    `Allowed::mail`, which is off for a new install.

  - [D] A message that cannot be decrypted says why (no key, wrong key, damaged) rather than
    reading as empty, which is the failure the note editor stub taught this project to avoid.

  - [D] Keys are secrets, so they follow the project's secrets rule: never in
    `message_cache.db`, never logged.

- [ ] **READ-03**: Hook into an external spam classifier.
  - Evidence: filters with rule actions are built (`src/application/filters.rs`), phishing risk
    scoring is built (`src/service/security.rs`), Safe Browsing lives in
    `src/service/safebrowsing/`. No external spam classifier integration exists. Junk folder
    sync is declined, so the classifier's verdict has to land locally.

  - [S] `docs/development/requirements-backlog.md`, near-term, priority Low; roadmap Phase 5.
  - [D] A classifier verdict is available to the filter rule vocabulary, so a user files spam
    with the rules they already have rather than through a second parallel system.

  - [D] The verdict is shown as a stated score with its source named, never as a silent
    deletion.

  - [D] Guardrail 9 applies: if the classifier is unreachable or returns nothing, the
    application says so rather than treating silence as "not spam".

### The other five modules

- [ ] **PIM-01**: Move a task from one list to another.
  - Evidence: no move-between-lists path in `src/application/tasks_sync.rs`. Task lists and
    tasks are cached and synced (`src/data/message_cache/`, `tasks_sync.rs`,
    `src/service/tasks_api.rs`).

  - [S] Recorded as not built in `docs/IMPLEMENTATION_STATUS.md` and `docs/ALPHA_TESTING.md`.
  - [D] A user moves a task to another list by keyboard, and the task appears in the target
    list and is gone from the source list in one action, not two.

  - [D] The move goes through `Allowed::personal_information`, which is on for a new install,
    and is refused with a reason when that gate is off.

  - [D] A move that fails at the provider leaves the task in exactly one list, never in both
    and never in neither.

- [ ] **PIM-02**: Move and copy items in the modules that are not mail.
  - Evidence: `src/service/protocols/imap.rs` has `copy_message` and `move_message` for mail.
    The inventory records move and copy as missing for everything else.

  - [S] Recorded as not built in `docs/IMPLEMENTATION_STATUS.md` and `docs/ALPHA_TESTING.md`.
  - [D] Contacts, events, notes and reminders each support move and copy between their
    containers with the same two keyboard commands in every module, because one key means one
    thing in every module here.

  - [D] Copy leaves the original untouched and move does not, and each announces which it did.
  - [D] The Action menu carries move and copy because they act on the selection; File, New
    stays for making things.

- [ ] **PIM-06**: Week and month calendar views. Reviewed in 2026-08-29: these do not exist,
  and PIM-03 assumed they did.

  - Evidence: `src/presentation/wx_calendar_module.rs` lines 46 to 57 say the views are not
    built and disable Prev and Next, naming them "Previous period, not built yet". The
    calendar is one flat event list loaded by account, not by date range.

  - [S] `docs/changelog.md` line 1424: "Day, Week and Month, three views this program cannot
    draw".

  - [D] A week view and a month view exist, each showing the events in its range, reachable
    and navigable by keyboard alone.

  - [D] Prev and Next are enabled in those views and move by one period, announcing the range
    they moved to rather than only redrawing.

  - [D] A screen reader can work through a view's events in date order without the user
    having to reconstruct the grid from cell labels. This is the criterion most likely to need
    a real screen reader run to settle, and it is why this is its own requirement.

- [ ] **PIM-03**: Show recurring events across the calendar's date ranges. Depends on PIM-06.
  - Evidence: `src/application/occurrences.rs` and `repeating.rs` hold the recurrence model;
    `src/application/calendar.rs` is 11,154 lines. The display this expands into is PIM-06's
    work, not this one's.

  - [S] `docs/changelog.md` line 1448: weeks and months is not built, so both say so and are
    switched off.

  - [S] Two stated limitations already sit next to this: editing or deleting a series needs
    the series already stored locally, and a weekly rule naming two or more days is a named
    known limitation of the recurrence editor.

  - [D] A recurring event appears on every date it occurs in whichever view is showing,
    expanded from the rule rather than from stored copies.

  - [D] An exception to a series (moved or cancelled occurrence) shows on the date it really
    is, and announces that it differs from the series.

  - [D] The two stated limitations above are either fixed or restated in the product where the
    user meets them, not left only in the changelog.

- [ ] **PIM-04**: Sync notes somewhere instead of leaving them on this computer.
  - Evidence: notes are cache-only. `src/presentation/wx_notes_module.rs` and the notes tables
    in `src/data/message_cache/` exist; there is no `notes_sync.rs` beside the other five
    `*_sync.rs` files.

  - [S] `docs/ALPHA_TESTING.md`: notes stay on this computer.
  - **Decided 2026-08-29 by Pratik.** Not one target. A note has a backend chosen by the
    account it belongs to, the local note itself is a first-class Markdown document, and the
    seam is shaped so a hosted service can be added later without a migration. That is three
    pieces of work, split into PIM-04, PIM-07 and PIM-08 below.

  - [D] A note is a Markdown document. `pulldown-cmark` is already a dependency and
    `application/long_text.rs`, `sign_off.rs` and `presentation/editor_document.rs` already
    render Markdown, so the content model reuses what signatures use rather than inventing a
    second one.

  - [D] The stored form is the Markdown source. What a note round-trips through any backend is
    that source, so a note edited here and read back is byte-identical when nothing changed.

  - [D] A screen reader reads the rendered structure, not the raw source: headings announce as
    headings and lists as lists, the way a contact's notes already do.

  - [D] Note sync goes through `Allowed::personal_information`.
  - [D] Until a backend is live, the settings screen says notes do not sync yet, rather than
    offering a switch that does nothing.

- [ ] **PIM-07**: A notes backend chosen by account type, behind one seam.
  - Evidence: nothing exists for any backend. `src/service/caldav.rs` handles no VJOURNAL and
    `src/service/microsoft_graph.rs` covers no OneNote, both checked 2026-08-29. The five
    existing `*_sync.rs` files are the shape to follow.

  - [D] One trait or enum decides where a note goes, and the account's protocol picks the
    backend. An account with no notes backend keeps its notes local and says so, rather than
    the feature being present or absent depending on who the user is.

  - [D] The backends are added one at a time, each behind the same seam, and adding the second
    changes nothing about the first. Which one comes first is a scheduling decision, not an
    architectural one.

  - [D] A note that cannot be sent is not silently dropped. It stays local, is marked as
    waiting, and the sync summary says why, the way a calendar change that cannot be sent
    already does.

  - [D] No backend claims to work against a real server until it has run against one. Nothing
    in this project has.

- [ ] **PIM-08**: The notes seam is ready for a hosted service without a migration.
  - Evidence: none. This is preparation for a service that does not exist yet, so it is the
    requirement most at risk of building for an imagined shape.

  - [D] The seam PIM-07 defines takes a hosted backend as one more implementation, with no
    change to the stored form and no migration of existing notes.

  - [D] What the seam assumes about a backend is written down: identity, conflict resolution,
    and what happens to a note whose backend is removed from an account.

  - [D] Nothing in this milestone ships a hosted client, a network call to one, or a setting
    offering one. Preparing for it means the seam does not forbid it, not that anything half
    exists. A switch that does nothing is the failure this project has fixed repeatedly.

- [ ] **PIM-05**: CardDAV for contacts.
  - Evidence: `src/service/caldav.rs` (8,149 lines) covers calendars only.
    `docs/development/requirements-backlog.md` states CardDAV is not built. Contact CRUD,
    groups and vCard 3.0 import and export are built in `src/application/contacts_sync.rs` and
    `importing_contacts.rs`, so the vCard half of CardDAV already exists.

  - [S] `docs/development/requirements-backlog.md`: "CardDAV not built", post-v1.0, Medium.
  - [D] A user adds a CardDAV address book by its own address, the way a calendar can already
    be added by its address.

  - [D] Contacts sync both ways through CardDAV, reusing the existing vCard reader and writer
    rather than a second one.

  - [D] CardDAV writes go through `Allowed::personal_information`, and the settings screen
    says this path has never met a real server.

  - [D] The XML and vCard parsing is pure and unit-testable without a server, per the project's
    thin-transport rule; the transport itself stays untested until a live account exists, and
    the requirement says so rather than claiming otherwise.

### How the application speaks

- [ ] **FEEDBACK-01**: Set feedback channels per event, not only globally.
  - Evidence: `src/presentation/accessibility/feedback.rs` already holds the model:
    `FeedbackSettings.per_event: Vec<(Event, BTreeSet<Channel>)>`, `set_event_channels`,
    `channels_for`, and serialisation through `to_stored` and `from_stored`. Searching
    `src/presentation/` for `per_event` or `set_event_channels` outside that one file returns
    nothing, so no screen writes it.

  - [S] `docs/changelog.md` known limitations: per-event feedback overrides exist in the model
    with no interface.

  - [S] Four channels are independent by design: Speech, Earcon, Braille, Visual. An earcon
    does nothing for a deaf-blind user, who reads braille, and speech does nothing for them
    either.

  - [D] The Settings Feedback tab lets a user turn each of the four channels on or off for each
    of the 16 events, by keyboard, and the chosen state is announced when focus lands on it.

  - [D] A saved override survives a restart, through the existing `to_stored` and `from_stored`
    round trip.

  - [D] Resetting an event returns it to the default rather than to silence, matching
    `channels_for`, which falls back to the default when there is no entry.

  - [D] A test counts the screens that reach `set_event_channels`, so a per-event override the
    model can hold and no screen can set fails at once rather than being found five months
    later.

  - [D] Corrected 2026-08-29. This criterion used to ask that `tests/house_style.rs` "no longer
    need a control no screen writes exception for this setting", which nothing could satisfy:
    there is no such exception. What is there, at line 122, is
    `A_CONTROL_NO_SCREEN_WRITES`, a list of phrases a document may not use, guarding against
    prose promising a per-account Allow Changes setting no screen writes. Same trap, different
    setting, and it reads documents rather than counting call sites. The per-event case needs
    its own check, and the criterion above is it.

- [ ] **FEEDBACK-02**: Dates and relative wording in the user's own language and format.
  - Evidence: `src/presentation/date_display.rs` holds a hardcoded `MONTHS` array starting
    "January" at line 101 and formats from it at line 384. `DateOrder` already offers
    MonthFirst and DayFirst and `DateStyle` offers Absolute and RelativeWithinWeek, so the
    shape for a locale decision exists; only the strings are English.

  - [S] `docs/changelog.md` line 6946: month names and relative wording are English on every
    machine.

  - [D] Month names, day names and relative wording ("2 days ago") come from the machine's
    locale, and `DateOrder` follows it rather than being a separate setting the user has to
    find.

  - [D] A locale with no translation falls back to English and says nothing about it in the
    UI, because a visible fallback notice on every row is worse than the fallback.

  - [D] The existing tests over `date_display` keep passing under a forced English locale, so
    the change is testable without a machine set to another language.

- [ ] **FEEDBACK-03**: Know how much of WCAG the automated scans actually cover.
  - Evidence: `.github/workflows/accessibility.yml` runs Axe.Windows over UI Automation and
    `scripts/msaa-names.ps1` over MSAA, per window; `.github/workflows/nvda.yml` drives a real
    copy of NVDA. `docs/IMPLEMENTATION_STATUS.md` records roughly half of WCAG covered and five
    findings at the last read, all inside WebView2's own tree.

  - [S] Automated checks catch roughly half of accessibility defects and do not replace testing
    with real assistive technology. Structure present is not experience good.

  - [D] The scan output names which WCAG 2.2 AA success criteria it can and cannot judge, so
    "roughly half" becomes a list rather than an estimate.

  - [D] The five WebView2 findings are re-read and each is either fixed, or recorded as
    upstream with the upstream named (guardrail 9: do not silently absorb upstream failures).

  - [D] The gap between what the scans cover and what only a manual screen reader pass can
    cover is written down as the list of interactions a human still has to walk, so the manual
    pass has a scope instead of being open-ended.

  - [D] No criterion here claims the manual pass has happened. Pratik decides when screen
    reader testing runs.

### Installing, updating and what is stored

- [ ] **SHIP-01**: A signed installer.
  - Evidence: `installer/Wixen-Mail-Setup.iss` builds an Inno Setup installer;
    `scripts/build-installer.sh` appends the commit it built from. `docs/ALPHA_TESTING.md`
    states the installer is not signed.

  - [S] `docs/ALPHA_TESTING.md`: "The installer is not signed."
  - [D] The published installer and the executable inside it both carry a valid Authenticode
    signature, with a timestamp countersignature, so the signature stays valid after the
    certificate expires.

  - [D] What SmartScreen does is stated, not promised. Corrected twice. On 2026-08-29 this
    stopped requiring that the unknown-publisher warning disappear, which a valid signature
    does not buy. On 2026-09-04 the replacement turned out to be wrong as well: it said only
    an EV certificate carries reputation from the first download. Microsoft's own page, at
    https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation
    and updated 2026-08-17, says the opposite in as many words: "EV certificates no longer
    bypass SmartScreen... this behavior no longer exists", and its table gives OV and EV the
    same first-download outcome, a warning until reputation accumulates.
    So **no certificate available to this project removes the warning immediately**. Only the
    Microsoft Store does. What signing buys is the publisher's name in the box instead of an
    unknown one, protection against Smart App Control on Windows 11, and reputation that
    carries from one release to the next when the same identity signs them.
    Worth noting how this stayed wrong: the 2026-08-29 correction narrowed the claim rather
    than rechecking its source, and the source had changed underneath it. An external fact in
    a project document goes false with no commit to this repository.

  - [D] While the warning remains, `docs/installing.md` keeps its walkthrough, including that
    the Run button does not exist until More info is activated and that the button focus lands
    on first is Don't run. Telling a screen reader user to press Enter cancels the install.

  - [D] The signing key never enters the repository or the build log, following the project's
    secrets rule.

  - [D] The verification is done against the published release asset, not against a local
    build.

  - [D] This is blocked on a certificate decision that is Pratik's. Until it is made, the
    requirement stays open and the docs keep saying the installer is unsigned.

- [ ] **SHIP-02**: Check for and apply updates.
  - Evidence: `grep -rni "check_for_update|auto_update|update_check" src/` returns nothing.
    `src/common/version.rs` already carries the version and build metadata, and
    `.github/workflows/release.yml` publishes releases.

  - [S] `docs/development/requirements-backlog.md`, platform, priority Medium; roadmap Phase 8.
  - [D] The application can tell the user a newer version exists, and the check is a
    deliberate action or an explicit setting, never a silent background fetch, because
    publishing and fetching both happen on purpose here.

  - [D] Applying an update is the user's decision and the current version keeps working if the
    update is declined.

  - [D] The check compares the plain `0.x.y` version and ignores `+build` metadata, matching
    `src/common/version.rs` and SemVer ordering.

- [ ] **SHIP-03**: The installed shortcuts carry the application icon. Narrowed 2026-08-29:
  the shortcuts themselves are already built.

  - Evidence: `installer/Wixen-Mail-Setup.iss` line 84 declares a `desktopicon` task, and the
    `[Icons]` block at lines 123 to 125 creates both the Start menu entry and the desktop
    shortcut. Inno removes both on uninstall. `SetupIconFile=..ssets\icon.ico` is set at
    line 49 and `assets/icon.ico` exists, but neither `[Icons]` entry sets `IconFilename`, so
    the shortcuts use whatever icon the executable carries. Inno creates no shortcuts by
    default; the previous evidence line said it does, inherited from the inventory.

  - [S] `docs/development/requirements-backlog.md`, platform, priority Medium.
  - [D] Both `[Icons]` entries set `IconFilename` to the bundled icon, so the shortcut a user
    sees in the Start menu and on the desktop is the application's own.

- [ ] **SHIP-04**: Encrypt the local cache, or decide not to and say so once and clearly.
  - Evidence: `src/data/message_cache/mod.rs` stores mail in plain SQLite.
    `src/service/security.rs` already uses `aes_gcm` with `Aes256Gcm` at lines 222 and 423, so
    the primitive is in the tree. `CLAUDE.md` states the cache is not encrypted and that the
    docs must not claim otherwise.

  - [S] `CLAUDE.md`: "Encrypting it means encrypting the whole database, which is a decision
    with a build cost, not something to imply in a feature list."

  - [S] Secrets are already out of the database: passwords and tokens live in the Windows
    credential store through `keyring`, so the database can be copied without carrying
    credentials.

  - **Decided 2026-08-29 by Pratik: not encrypted, and said so once and clearly.** The
    database stays copyable and backup-safe, which the design leans on, and the protection
    rests on Windows keeping other users out of the folder and on full-disk encryption for a
    stolen drive. The remaining work is saying that where a user meets it, not building
    anything.

  - [D] The product says it plainly where somebody deciding whether to trust it reads: the
    first-run screen and the page about what is stored, not only in the changelog.

  - [D] It says what the limitation is and is not. Another user of the same computer is kept
    out by Windows; somebody who takes the drive out is not, unless the disk itself is
    encrypted.

  - [D] If it is not encrypted, the wording in the product and the docs is unchanged and this
    requirement closes as a recorded decision, not as a silent drop.

- [ ] **SHIP-05**: The crate builds and its tests pass on Linux and on macOS.
  - Evidence: split from the disclosure below on 2026-08-29, because building on a platform and
    telling its users what does not work there are different pieces of work with different
    gates. Roadmap Cross-Platform is unticked. `Cargo.toml` gates Windows dependencies behind
    `[target.'cfg(windows)'.dependencies]`. Nothing in CI builds the crate off Windows: every
    `runs-on:` in `.github/workflows/` is `windows-latest` except one `ubuntu-latest` job at
    `ci.yml` line 133, which runs `cargo audit` and reads `Cargo.lock` without building.

  - [S] Roadmap Cross-Platform, unticked.
  - [D] A CI job builds the crate and runs the test suite on Linux, and another on macOS, so a
    Windows-only assumption fails on the commit that adds it rather than on the day somebody
    tries a port.

  - [D] No criterion here claims the application is accessible on Linux or macOS, or usable
    there. Building is not the same claim.

- [ ] **SHIP-06**: Off Windows, the application says what its accessibility layer does not do.
  - Evidence: `CLAUDE.md` records that `wxAccessible` and `UiaRaiseNotificationEvent` exist
    only on Windows, and that both compile and silently do nothing elsewhere. A build that
    compiles and announces nothing is the failure mode this project has fixed repeatedly: the
    switch that does nothing, with no way to tell from the inside.

  - [S] A macOS or Linux port needs its own bridge for each of those two, not a framework
    change.

  - [D] On a platform where the accessibility bridge is absent, the application says so where
    somebody will meet it, at startup and in Help, rather than presenting a client that looks
    accessible and is not. That is guardrail 3: no stub presented as complete.

  - [D] The disclosure is derived from what is actually compiled in, not from a hardcoded
    platform list, so adding a real bridge removes the warning without anyone remembering to.

  - [D] This closes on the disclosure. It does not close on a working bridge, and no wording
    here may be read as claiming one.

### Every number the project quotes

- [ ] **PERF-01**: Memory under 150 MB with 1,000 cached messages, measured.
  - Evidence: no `benches/` directory and no `criterion` or `divan` in `Cargo.toml`, so no
    target below has a number attached. No memory profiling has been run.

  - [S] `docs/development/requirements-backlog.md`, performance and scale, Medium.
  - [D] A repeatable measurement produces a number for resident memory with 1,000 cached
    messages, recorded with the date, the machine and the build it came from.

  - [D] The number is either under 150 MB or the target is revised with the reason, rather
    than the target quietly remaining aspirational.

- [ ] **PERF-02**: Cold start under 2 seconds, measured.
  - Evidence: startup time optimisation is unticked in roadmap Phase 8; nothing measures it.
  - [S] Roadmap Phase 8; `docs/development/requirements-backlog.md`.
  - [D] Cold start is measured from process start to the message list being usable, not to the
    window appearing, because an empty window is not a usable inbox.

  - [D] The measurement is repeatable and recorded with the date, the machine and the build.

- [ ] **PERF-03**: A real mailbox of 100,000 messages or more, exercised.
  - Evidence: the design targets 200,000 rows; the largest thing exercised is a loopback
    server. The mail-at-scale plan requires the virtual text callback to read from an in-memory
    page cache and never touch SQLite, with pages of 200 rows around the viewport and a
    placeholder on a cache miss.

  - [S] Roadmap Phase 8; `docs/plans/20260726-mail-at-scale.md`.
  - [S] Sorting 200,000 rows in memory on a header click is a multi-second freeze, and a freeze
    is an accessibility failure, not a performance one.

  - [D] The list is exercised against 200,000 synthetic rows, which needs no network and no
    live account, and the sort, filter and scroll paths each produce a recorded number.

  - [D] A test asserts the virtual text callback issues no SQLite query.
  - [D] No criterion here claims a real provider mailbox was used. Synthetic rows answer the
    list question; the provider question waits for a live account.

- [ ] **PERF-04**: Idle memory under 100 MB, measured.
  - Evidence: listed as a success metric in `docs/roadmap.md` with no measurement.
  - [S] Roadmap success metrics.
  - [D] Idle memory is measured after startup with a cache present and no user activity, and
    recorded with the date, machine and build.

  - [D] The number is either under 100 MB or the target is revised with the reason.

- [ ] **PERF-05**: Line coverage re-measured.
  - Evidence: 60.4%, measured 2026-07-26 with `cargo llvm-cov --lib --summary-only`, stale
    since. Roughly 275 commits have landed since.

  - [S] `docs/IMPLEMENTATION_STATUS.md`.
  - [S] Coverage is the cheap wide sweep answering only "what never runs at all", and low
    coverage in `service/protocols`, `service/oauth` and the provider clients is the network
    transport that has never met a live account, tracked as work rather than as a testing gap.

  - [D] A current number replaces the stale one, with its date, and the low areas are
    attributed rather than treated as a number to raise.

- [ ] **PERF-06**: Every document that quotes a test count quotes the same measurement.
  - Evidence: three documents give three numbers. `docs/IMPLEMENTATION_STATUS.md` line 104 says
    3,362 (3,282 unit, 80 integration) measured 2026-08-09. `docs/changelog.md` and
    `docs/integration-guide.md` say 5,269 "that run today".
    `cargo test --all-targets -- --list` on 2026-08-29 counts 5,430: 5,269 unit and 161
    integration. So the 5,269 is the unit count wearing the label of the total, and it is the
    split, not the number, that catches that.

  - [S] `docs/IMPLEMENTATION_STATUS.md`.
  - [D] Every count in the documentation carries the command it came from and the date it was
    taken, so a number that has moved reads as a stale measurement rather than as a current
    fact.

  - [D] The count is split unit against integration, which is the distinction the three
    numbers now in the tree disagree about.

  - [D] Nothing asserts that a number written in a document equals what `cargo test` reports.
    Corrected 2026-08-29: that is what this requirement used to ask for, and it is false the
    next time anyone adds a test. A check on a number here checks that its command and its date
    are present and that documents agree with each other, never that the number has not
    moved.

- [ ] **PERF-07**: A whole-tree mutation run, once, with a real result.
  - Evidence: scoped runs only. mime and error on 2026-07-26; filters, due, tagging and
    signatures on 2026-08-01 with 157 mutants; the four message-disposition modules on
    2026-08-12 with 66 mutants and 1 survivor. A whole-tree run is about two days and has never
    been done. `guards/guards.toml` holds 501 records, of which 192 had been hand-verified as
    of 2026-08-12.

  - [S] `docs/IMPLEMENTATION_STATUS.md` and `CLAUDE.md`.
  - [S] `scripts/mutants.sh` refuses to summarise a partial or degenerate run: a build that
    failed before any mutant ran, mutants recorded unviable without distinguishing "compiler
    rejected it" from "compiler never started", and a run where the suite was never once run
    against a changed line.

  - [S] `--since main` compares main with itself, finds nothing, and now says so. Name a real
    tag or commit.

  - [D] One whole-tree run completes and its report is read after the process exits, not from a
    partial `mutants.out`.

  - [D] Mutants that never reached a compiler are re-run rather than counted, since that
    failure comes and goes on this machine and says nothing about the mutant.

  - [D] Every survivor is either killed with a test or recorded with a reason, and the surviving
    list becomes the input to the next round rather than a headline number.

## v2 Requirements

Deferred out of this milestone, with the reason. Each was in the inventory's "not built"
section and is real work; none is declined.

| Requirement | Reason for deferral |
|---|---|
| Gmail X-GM-THRID conversations and X-GM-RAW server-side search | Blocked on the IMAP library, not on this codebase. Roadmap Phase 2 and `docs/changelog.md` both record it that way. |
| The Exchange path described in `docs/plans/20260726-mail-at-scale.md` | The Microsoft work that shipped went through Graph for contacts, calendar and tasks. With EWS declined, this section proposes a path nothing needs. |
| JMAP | `docs/development/requirements-backlog.md`, future, priority Low. |
| Plugin and extension system | `docs/development/requirements-backlog.md`, future, priority Low. |
| Setting Wixen Mail as the actual Windows default mail client | Windows does not allow a program to make itself default. `src/service/default_apps_registration.rs` registers what it can and the product already says plainly that it cannot set the default. |

## Out of Scope

Declined on purpose. Each is a decision recorded in the sources, not an omission.

| Feature | Reason |
|---------|--------|
| Exchange Web Services | Microsoft begins blocking third-party EWS against Exchange Online on 1 October 2026, with full retirement by April 2027. `docs/plans/20260726-mail-at-scale.md`: "We will not write EWS." |
| Handing an attachment to Windows to open | Deliberate. PDFs are the exception and are read in-app through `src/service/pdf.rs`. Recorded in `docs/changelog.md` known limitations. |
| Junk folder sync | Deliberate. The folder can still be opened. Recorded in `docs/changelog.md` known limitations. |
| Live-account validation of the built-but-unproven paths | Real work, and real risk, but not this milestone. It is the 13 rows of "built but unproven" in `.planning/intel/built-and-left.md`. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOLDER-01 | Phase 1 | Complete |
| FOLDER-02 | Phase 1 | Pending |
| FOLDER-03 | Phase 1 | Complete |
| THREAD-01 | Phase 1 | Complete |
| THREAD-02 | Phase 1 | Complete |
| SEARCH-01 | Phase 2 | Complete |
| SEARCH-02 | Phase 2 | Complete |
| SEARCH-03 | Phase 2 | Complete |
| SCALE-01 | Phase 3 | Pending |
| SCALE-02 | Phase 3 | Pending |
| SCALE-03 | Phase 3 | Pending |
| SCALE-04 | Phase 3 | Pending |
| SCALE-05 | Phase 3 | Pending |
| SCALE-06 | Phase 3 | Pending |
| WRITE-01 | Phase 4 | Pending |
| WRITE-02 | Phase 4 | Pending |
| WRITE-03 | Phase 4 | Pending |
| READ-01 | Phase 4 | Pending |
| READ-02 | Phase 4 | Pending |
| READ-03 | Phase 4 | Pending |
| PIM-01 | Phase 5 | Pending |
| PIM-02 | Phase 5 | Pending |
| PIM-06 | Phase 5 | Pending |
| PIM-03 | Phase 5 | Pending |
| PIM-04 | Phase 5 | Pending |
| PIM-07 | Phase 5 | Pending |
| PIM-08 | Phase 5 | Pending |
| PIM-05 | Phase 5 | Pending |
| FEEDBACK-01 | Phase 6 | Pending |
| FEEDBACK-02 | Phase 6 | Pending |
| FEEDBACK-03 | Phase 6 | Pending |
| SHIP-01 | Phase 7 | Pending |
| SHIP-02 | Phase 7 | Pending |
| SHIP-03 | Phase 7 | Pending |
| SHIP-04 | Phase 7 | Pending |
| SHIP-05 | Phase 7 | Pending |
| SHIP-06 | Phase 7 | Pending |
| PERF-01 | Phase 8 | Pending |
| PERF-02 | Phase 8 | Pending |
| PERF-03 | Phase 8 | Pending |
| PERF-04 | Phase 8 | Pending |
| PERF-05 | Phase 8 | Pending |
| PERF-06 | Phase 8 | Pending |
| PERF-07 | Phase 8 | Pending |

**Coverage:**

- v1 requirements: 40 total
- Mapped to phases: 40
- Unmapped: 0

## Where these came from

Every requirement above traces to one row of `.planning/intel/built-and-left.md`, in one of
two sections and no others:

- "Not built, named in a document as wanted": 33 rows in the file. Three are declined by the
  user (EWS, handing an attachment to Windows, junk folder sync). Four more deferred to v2
  above, producing five v2 entries because one row named two items. The remaining 26 rows
  produced 32 requirements, because six rows named more than one thing each: folder favourites
  with smart folders and spam filtering, moving a task with move and copy generally,
  drag-and-drop with inline images, network status with conflict resolution, auto-update with
  desktop shortcuts, and JMAP with the plugin system.

- "Not built, performance and scale targets never measured": 8 rows, producing 8
  requirements. Seven are PERF-01 to PERF-07; the eighth, accessibility scanning coverage, is
  FEEDBACK-03, placed with the accessibility work rather than with the performance work
  because it measures the same thing that phase is about.

**Discrepancy, resolved 2026-08-29.** The brief said the first section has 27 rows. The file
has 33, and 33 is right. The 27 was quoted from the inventory agent's summary of the document
it had just written, and reached the brief without anyone counting the file. All 33 rows are
accounted for above, so nothing was dropped.

---
*Requirements defined: 2026-08-29*
*Last updated: 2026-08-29 after the first roadmap pass. Every [D] line is awaiting Pratik's review.*

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

### What an Evidence line has to contain. Added 2026-09-04

That sentence was the intention and it did not survive five weeks. An audit on 2026-09-04
(`.planning/requirements-audit-2026-09-04.md`) found 12 of 18 evidence blocks wrong, and the
phase research for 4, 5 and 7 found most of the rest. Every wrong one was wrong in the same
direction: it said something was missing that had since shipped, or named a defect that had
since been fixed. Nothing over-claimed. Correcting the blocks one at a time does not stop that
happening again, so the mechanism is written down here.

**A citation's precision and its durability are different things, and a reader mistakes the
first for the second.** Three ways an evidence line rots while still reading as verified, all
of them present in this document before 2026-09-04:

- **A line number cannot be re-run.** THREAD-01 said "line 582 calls `item.enable(false)`",
  which was true when it was written. Line 582 is something else now, and a reader who looks
  cannot tell whether the document is stale or they mis-counted.
- **A grep goes blind when the vocabulary moves.** FOLDER-01 searched `create_folder`. The
  feature shipped as `create_mailbox`, so that command still returns nothing and re-running the
  evidence reads as confirmation. It is the sharpest case here: five operations ship, and the
  check written to find them cannot see any of them.
- **A bare assertion of absence names no method.** "no favourites path in `src/`", "Nothing
  joins the two", "has never been done". Re-checking one of those means inventing a search and
  hoping it is the same search.

So: **anything a later reader is expected to re-check carries the literal command, its result
in one line, and the date it was run.** Cite a symbol rather than a line number, because a
symbol survives the edit that moves the line. Where the claim is an absence, search the
concept's several plausible spellings and say which ones were searched, so a reader can see
that the vocabulary has moved instead of reading a stale nil result as a fresh one. This is the
rule the project already applies to test counts under PERF-06 and to guard records in
`guards/guards.toml`.

**How fast this happens, measured on this pass.** Four sentences written into this document on
2026-09-04, each correctly saying that some other document was stale, were themselves false
within the hour, because the documents they named were corrected while this pass was running:
the changelog's "nine events", the changelog's missing "Since closed" marker on threading, and
two unticked roadmap lines. They were caught only because the working tree was checked again
before finishing. So a sentence about another document's state expires the moment somebody fixes
that document, and nothing tells you. Where a claim like that has to be made, write it in the
past tense with its date, the way those four now are, rather than as a standing "still says".

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

  - Evidence: rewritten 2026-09-04, and the previous evidence is worth reading for how it
    failed. It ran `grep -rn "create_folder|rename_folder|delete_folder" src/`, found nothing,
    and concluded the feature was absent. The feature shipped under `create_mailbox`,
    `rename_mailbox` and `delete_mailbox`, so that command returns nothing today too and reads
    as confirmation.
    All five operations are built and reachable from the Action menu. The command that asks the
    question the old one meant to ask, run 2026-09-04:

    ```
    grep -n "fn create_mailbox\|fn rename_mailbox\|fn delete_mailbox" src/service/protocols/imap.rs
    ```

    That returns lines 907, 944 and 976, each function opening with the `may_i` gate.
    `application/mail_controller.rs` passes all three through at 546, 560 and
    571 with no logic of its own, and `mark_folder_read` is at
    `src/data/message_cache/messages.rs:1614`. The menu items are `ID_NEW_FOLDER`,
    `ID_RENAME_FOLDER`, `ID_MOVE_FOLDER`, `ID_DELETE_FOLDER`, `ID_EMPTY_FOLDER` and
    `ID_MARK_FOLDER_READ`, declared at `src/presentation/wx_app.rs` lines 83 to 88 and 91 to
    92, with handlers from line 3576 onward. `src/service/outward.rs:789` records exactly 11
    outward calls in `imap.rs`, re-measured 2026-08-31.
    The `Allowed::mail` gate sits inside the session rather than the controller, so no caller
    can answer it differently, and `local_folders::is_local`
    (`src/application/local_folders.rs:110`) is still the single decider of local against
    server. Both `[D]` lines about those are satisfied as written.

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
  - Evidence: rewritten 2026-09-04. The previous evidence said the inventory records the tree
    as one flat level. That was true when written and is not true now: nesting shipped in phase
    1, and what is left is a screen reader run rather than code. This row stays open for that
    reason and no other.
    Nesting is read from `folders.parent_id`, written once at sync by `mail_sync::store_folders`
    from the separator the server gave for that one mailbox, and nothing splits a path at
    display time (`src/presentation/folder_tree.rs:9` to 15, decision D-22).
    `src/application/folders_underneath.rs` holds the shared walk, bounded by
    `AS_DEEP_AS_A_TREE_GOES` at line 45 because a cycle in `parent_id` written by an earlier
    version is not hypothetical. `TreeRow.depth` is at `folder_tree.rs:214` and `TreeRow.label`
    at 212, with the rule in the label's own doc comment at 210: its level, its expanded state
    and its position "are not in here and must not be put here". Collapse
    survives a restart through a `tree_state` table: `set_row_collapsed` and `collapsed_rows`
    at `src/data/message_cache/folders.rs:275` and 309, called from
    `src/presentation/wx_app.rs:13732`, 10224 and 14909. A collapsed parent's unread count is
    `unread_here` against `unread_in_all` (`folder_tree.rs:227` and 228), worded by
    `unread_text` (line 381) under the `UnreadOnAParent` setting
    (`src/application/folder_settings.rs:34`).
    What remains is guardrail 2: no screen reader has confirmed that the level is announced
    from the native `TreeCtrl` rather than from the label text. No test in this repository can
    answer that.

  - [S] Recorded as a known limitation in the `[Unreleased]` section of `docs/changelog.md`.
  - [D] A folder named `Archive/2026` appears as `2026` nested under `Archive`, and a screen
    reader announces its level from the native `TreeCtrl` rather than from the label text.

  - [D] Collapsing and expanding work by keyboard, and the tree remembers what was collapsed
    across a restart.

  - [D] Unread counts on a collapsed parent account for its children, and the announcement
    says which of the two numbers it is giving.

- [x] **FOLDER-03**: Pin frequently used folders as favourites.
  - Evidence: rewritten 2026-09-04. The previous evidence asserted "no favourites path in
    `src/`" and named no method, which is the shape of absence claim this document now refuses.
    It is false and the whole thing is built and wired.
    `src/application/favourites.rs` holds `Pin` (line 73), `PinnedBranch` (91),
    `what_each_account_has` (130), `in_account_order` (154) and the four announcement builders
    `now_pinned`, `already_pinned`, `now_unpinned` and `was_not_pinned` at 183, 193, 203 and
    208. The menu ids `ID_PIN_FOLDER` and `ID_UNPIN_FOLDER` are at
    `src/presentation/wx_app.rs:91` and 92 with the handler at 3613, and the group heading
    `FAVOURITES` is defined once at `favourites.rs:64` and read by
    `src/presentation/folder_tree.rs`, whose `group_text` is at line 450, which satisfies the
    last `[D]` line.
    The pin-versus-subscription decision this requirement asked to have taken in advance was
    taken in advance and is written at `favourites.rs:9` to 45: a pin is local and never writes
    a subscription, a subscription never adds or removes a pin, and a pin is stored against
    `(account_id, path)`, the same pair `imap::set_subscribed` names a mailbox by, so joining
    them later moves nothing.

  - [S] Recorded in `docs/development/requirements-backlog.md` as "Pin frequently used
    folders", not built.

  - [D] A user pins and unpins a folder by keyboard from the folder tree, and pinned folders
    appear in a group at the top of the tree in a stable order.

  - [D] Pinning is a local preference first: it writes only on this computer, never to the
    server, and never passes through `Allowed`. That is what this phase builds.

  - [D] The stored shape allows IMAP subscription to back it later without a migration.
    `set_subscribed` is at `src/service/protocols/imap.rs:873` (line 840 when this was written),
    and subscription is what other mail clients mean by marking a folder you care about, so the
    two will meet.

  - [D] Which wins when they disagree is recorded as a decision before the second half is
    built, not left to whichever code path runs last. A local pin and a server subscription
    are two answers to one question, and this project has been bitten by that shape before.

  - [D] The pinned group announces itself as a group, so a screen reader user can tell a
    pinned copy of Inbox from the real one.

### Conversations

- [x] **THREAD-01**: Collapse the message list to one row per conversation.
  - Evidence: rewritten 2026-09-04. The previous evidence said the command "exists and is
    switched off", citing three line numbers, all of which have moved. It is on, it has a
    keyboard shortcut, and the disabling call is gone.
    `ID_THREAD_VIEW` is declared at `src/presentation/wx_app.rs:116`, appended as a check item
    at 5696 with `Ctrl+T`, dispatched at 4037 to `switch_the_view`, and its tick kept in step by
    `sync_menu_check` at 12323 and 12355. The `item.enable(false)` the previous evidence named
    is gone: the only occurrence of that string in the file is a test fixture at 25627.
    The guard over it is documented at 25483 and had to change its anchor, which is worth
    knowing before trusting it. It was written against `find_item(ID_THREAD_VIEW)`, a call that
    existed only while the item was disabled, so its own green half deleted what it read and it
    passed unconditionally for a stretch. It now anchors on the identifier itself (25501).

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
  - Evidence: rewritten 2026-09-04. Closed by 01-13, and the mechanism moved, so the previous
    evidence ("`threading.rs` rethreads on folder open") now names the wrong module.
    A message gets its conversation as it is stored:
    `src/data/message_cache/messages.rs:834` calls `thread_identity::conversation_root` while
    writing the row, and lines 966 to 976 handle the late message that connects two trees by way
    of `thread_identity::identifiers_worth_asking_about` and `thread_identity::rejoin`.
    `src/application/thread_identity.rs:5` records why it exists: `messages.thread_id` shipped
    as a column nothing wrote. `backfill_thread_ids` (`messages.rs:1010`) fills it in for
    databases written before that. `threading.rs` still exists and `thread_messages` still runs
    in memory for the conversation tree, which `thread_identity.rs:36` states.
    One document had not kept up and has since been corrected: `docs/changelog.md:8394` carried
    the old known limitation under `[Unreleased]` with no "Since closed" marker, contradicting
    line 574 of the same section, while its neighbour at 8392 carried one. It now carries the
    marker, corrected 2026-09-04.

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
  - Evidence: rewritten 2026-09-04, having been rewritten once already on 2026-08-29. Closed.
    The defect the previous evidence described, that `what_a_typed_search_asks` always writes
    the three questions in `WHAT_A_TYPED_SEARCH_LOOKS_AT`, is fixed, and every line number it
    gave has moved.
    `what_a_typed_search_asks` (`src/application/saved_searches.rs:554`) now calls
    `what_that_answer_looks_at` (line 538), which matches on `WhereToSearch` and returns
    `["subject"]` for SubjectOnly, `["from"]` for SenderOnly, and `WHAT_A_TYPED_SEARCH_LOOKS_AT`
    for the two that do not narrow a field. Both halves of the scope are written from one value:
    the folder comes from `ran.the_folder_looked_in` at line 566, which is what the second `[D]`
    line asked for.
    The fourth `[D]` line, about a search saved by an older version, stopped needing an answer
    rather than being answered: the unnarrowed case returns the shared constant itself rather
    than a copy, so an old search and a new unnarrowed one cannot be told apart and there is no
    absent value anywhere. Line 529 says so.
    The live search side is unchanged and still honours every scope it offers:
    `search_messages` (`src/data/message_cache/searching.rs:477`) takes
    `looking_in: WhereToSearch`, with tests at 690, 727 and 758.

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
  - Evidence: rewritten 2026-09-04. Built and wired on both doors. The disclosure this
    requirement is mostly about was absent when the previous evidence was written and is not
    absent now.
    Bodies are still split out with a size budget and least-recently-read eviction in
    `src/data/message_cache/bodies.rs`, and the coverage is now measured and said:
    `how_much_message_text_the_index_holds` (`src/data/message_cache/searching.rs:609`) reads
    the `text_is_in_the_search_index` column named by `THE_INDEX_HOLDS_THE_TEXT` at line 149.
    The search box asks it at `src/presentation/managers.rs:1852` and words it with
    `what_the_search_box_covers` (`src/application/saved_searches.rs:780`); a saved search asks
    it at `src/presentation/wx_app.rs:6455` and words it with `what_a_saved_search_covers`
    (line 761) and `what_a_search_says_as_it_opens` (line 670, said on open at
    `wx_app.rs:6671`). Whether a search needs body text at all is answered in one place, by
    `reads_the_message_text` (`saved_searches.rs:1104` and `searching.rs:85`), so a search about
    senders and subjects never pays to ask.
    The offer to fetch the rest is built too: `ID_FETCH_MISSING_TEXT`
    (`wx_app.rs:156`, item at 5486, handler at 4220), with the list it fetches named at
    `bodies.rs:541`. Both `[D]` lines are satisfied, and the changelog sentence quoted as `[S]`
    below is now itself a stale sentence rather than a description of the code.

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
  - Evidence: rewritten 2026-09-04. Built, under a shape decided after this requirement was
    written. "Nothing joins the two into a folder that updates itself" is false, and it is
    another bare absence claim that named no method.
    D-2-01 (`.planning/phases/02-search-that-says-what-it-covers/02-CONTEXT.md:41`) makes a
    smart folder a saved search with a fuller editor rather than a second object, so the join
    the previous evidence said was missing is `Question::as_a_rule`
    (`src/application/saved_searches.rs:118`), which turns a saved search's condition into the
    filter engine's own `FilterRule` and runs it through `FilterEngine::matches` at line 1178.
    One matcher, one storage, two doors onto it. Opening a saved search lists what matches now
    rather than a snapshot, because the questions are evaluated at open time.
    The editor is `build_rule_edit_dialog` (`src/presentation/wx_managers.rs:2770`),
    `show_rule_edit` (2958) and `show_rule_manager_dialog` (3136), reached from
    `ID_EDIT_SEARCH_CONDITIONS` (`src/presentation/wx_app.rs:221`, item at 5904, handler at
    4683, call at 7052) and from the folder tree's context menu. `docs/roadmap.md:157` showed
    `- [ ] Smart folders based on rules` unticked when this was audited, and was ticked on
    2026-09-04 along with `Folder favorites` on the line above it, which FOLDER-03 had left in
    the same state.

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
  - Evidence: the count is `tests/one_sign_in_per_piece_of_work.rs` and no longer this
    paragraph. That test reads the shipping half of `src/presentation/wx_app.rs`, finds every
    place that builds a `MailController` and connects it to IMAP without going through the
    sign-in helper, and fails when the total moves, naming each site and the line it is on at
    the moment it runs. **Twelve** on 2026-09-04.
    The history is why the test exists, so it is kept rather than tidied away. This line said
    **eight**, at eight line numbers, when it was written on 2026-08-29. By 2026-09-03 every
    one of those eight lines had moved and not one of them was a connect site any more, and
    the real count was twelve. Those twelve were then written down as line numbers too, and by
    2026-09-04 seven of them had moved again, because the file had grown by twenty-one lines
    in between. A plan budgeted against the first list would have been wrong twice over, about
    which sites and by half about how many. Nothing noticed either time, and that is what a
    test noticing is for.
    The worst case, named as a case because a line number for it is the mistake above: marking
    a single message read builds a controller, connects, issues one `set_flag`, and
    disconnects, so one keystroke is a TLS handshake, a CAPABILITY, a LOGIN and a SELECT. It
    is in `spawn_server_change` in `src/presentation/wx_app.rs`.
    `src/application/mail_session.rs` line 21, `a_session_at`, is the purpose-built helper that
    signs in for one piece of work. It has three production callers, checked 2026-09-04:
    `deleting_at_the_server.rs` line 112, `sent_copy.rs` line 245, and `spawn_draft_append` in
    `wx_app.rs`.
    `src/application/mail_controller.rs` line 278, `require_imap`, is the single lock a held
    session would live behind, and it does not need replacing. There is no reconnect or retry
    anywhere in `mail_controller.rs`, `imap.rs` or `mail_sync.rs`.
    The budget starts at two rather than one: `watch_folder` (`mail_sync.rs` line 1165) already
    holds its own connection for IDLE and is called from `spawn_mail_watch` in `wx_app.rs`.
    That call was cited here as line 17212 and had moved to 17233 by 2026-09-04, which is the
    same drift again in a citation nobody was watching.
    The mail-at-scale plan budgets one connection for IDLE and two or three for fetching, and
    notes Gmail allows fifteen per account and punishes more.

  - [S] `docs/changelog.md` known limitations: "Holding one connection open needs reconnect
    handling that is not built."

  - [D] Opening several messages in a row reuses one authenticated session rather than
    reconnecting per message.

  - [D] A dropped connection reconnects once and retries the fetch, and says so if the retry
    also fails, rather than surfacing a bare protocol error.

  - [D] The number of concurrent connections per account is bounded by a stated budget, and
    the bound has a test.

  - [D] A test counts the sites that build a `MailController` and connect without going through
    the sign-in helper. Twelve bypasses accumulated behind a helper written to stop exactly
    that, and nothing counted them.

  - Partly closed by 03-02, and only the counting deliverable. The count is now
    `tests/one_sign_in_per_piece_of_work.rs`, which reads the tree rather than asserting a
    number, names the sites it finds when the total moves, and is coupled to
    `src/presentation/wx_app.rs` by `guards/guards.toml` so it runs on the commits that could
    change it. Twelve as of 2026-09-04.
    The other three deliverables are 03-06's, and none of them is advanced by this. Reusing
    one authenticated session across several messages, reconnecting once and retrying after a
    dropped connection, and bounding the connections per account, all need a session with a
    lifetime, and 03-02 counts rather than holds. The number is expected to fall when 03-06
    lands, and the test is what will say whether it did and whether it stayed down.

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

- [x] **SCALE-04**: Split storage into envelope, body cache and attachments.
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
    Closed 2026-09-04 by plan 03-03, which proved the three deliverables rather than
    rebuilding anything, and made one real change. The guard for the first is
    `data::message_cache::messages::a_listing_reads_no_message_text`, and it does not
    read the query text: it builds a database holding only the three tables a listing
    may read, drops the two inline body columns from `messages`, and asks SQLite to
    prepare every query a listing runs against it. That is stronger than the wording
    below, which the plan and this requirement both had wrong. A check over the tables
    a query plan names is green through `SELECT m.body_plain`, because `messages` is a
    table a listing is allowed to read and a plan names cursors by their alias.
    The second is closed by a fixture written from the shipped schema directly, with
    text in the inline columns and no `message_bodies` table, opened through the real
    `MessageCache::new`. All five of its tests passed on arrival: the migration was
    already correct and had never been tested against a database it was about.
    The third is closed with the correction that a sync writes no attachment
    *description* either, not only no file. `ImapMessage` carries `has_attachments`
    and no list, because a header fetch does not read a message's structure.
    The permanent migration is still permanent and no longer costs anything. A partial
    index over exactly its condition (`idx_messages_inline_body`) turns its opening read
    from a scan of `messages` into a lookup against an index that is empty on any
    database that has been opened once: measured on a release build at 200,000 messages
    with all their text already moved, warm, 32 ms against under 0.1 ms, for 8 KB of
    index. Nothing records that the migration has been done, deliberately, and the code
    says why: a marker would have to be trusted, and one wrong in that direction leaves
    message text inline that nothing else will ever move.

  - [S] The SPEC states the tiers: envelope always local at about 1 KB each, roughly 200 MB at
    200,000 messages; body cache fetched on open and evicted least-recently-used against a
    budget defaulting to 500 MB; attachments never fetched automatically.

  - [S] Schema changes are additive: `CREATE TABLE IF NOT EXISTS` and `ensure_column_exists`,
    never dropping or renaming a shipped column.

  - [x] [D] A folder listing query reads no body text, proved by asking SQLite to resolve
    every query a listing runs against a database with the message text taken out of it.
    Not by asserting anything about the query text, and not over a query plan: the first
    goes stale and the second cannot see a body column in a table the listing may read.

  - [x] [D] An existing user database opens and migrates without losing a message, and the
    migration has a test over a database written by the previous schema, in
    `data::message_cache::bodies::a_database_from_the_schema_that_kept_text_inline`.

  - [x] [D] The attachment tier is never populated by a sync; attachments arrive only when
    something asks for one. A sync writes neither the file nor the description, which is
    more than this line asked for and is all a sync could write either way.

- [x] **SCALE-05**: Detect network status and offer offline mode rather than only accepting a
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
  - Evidence: re-checked 2026-09-04 and still accurate, which makes this the only one of the
    six in this section that was. `grep -rn "DropTarget|OnDropFiles|drop_target" src/` returns
    nothing outside two unrelated test names (`bodies.rs:1047`, `wx_app.rs:23408`), and
    `src/application/attaching.rs` is the attachment model as claimed. Roadmap Phase 4 leaves
    it unticked.
    Two things the evidence did not say that bear on the criteria. **The existing attach path
    takes one file.** `attach_files` (`src/presentation/wx_compose.rs:1408`) builds its picker
    without `FileDialogStyle::Multiple` and calls `picker.get_path()`, singular, at 1425. A drop
    hands over many files at once, so the picker widens in the same change or the two paths
    disagree about how many files an attach is. **The framework has what is needed**, so the
    risk is where a drop lands rather than whether it can be caught:
    `FileDropTarget::builder(window).with_on_drop_files(...)` exists in wxdragon 0.9.17, and
    paste is available through `Clipboard::get_data(&FileDataObject)`. What WebView2 does with a
    file dropped on the composer's body is untested here and unknown, and it is worth settling
    with a throwaway build before tasks are planned around it.

  - [S] Roadmap Phase 4, unticked.
  - [S] WCAG 2.5.7 forbids drag-only interaction, and the mail-at-scale plan names column
    reordering as the classic place applications ignore that.

  - [D] Dropping a file onto the composer attaches it, and every drop action has a keyboard
    equivalent that is at least as quick to reach.

  - [D] Attaching announces the file name and size, and refusing a file says which file and
    why.

- [ ] **WRITE-02**: Insert an image inline in an HTML message.
  - Evidence: rewritten 2026-09-04. "No inline image insertion path exists" is false. It is
    built end to end and reached from a menu, and the first `[D]` line below is already
    satisfied in the stronger form: alt text is not merely asked for, it is compulsory.
    `insert_picture` (`src/presentation/wx_compose.rs:2934`) opens a picture picker, reads the
    file, then asks "Describe the picture, for somebody who cannot see it:" in a
    `TextEntryDialog` (2972 to 2984). `a_picture_to_send` (`src/application/pictures.rs:349`)
    refuses an empty description outright (352 to 358) and returns an `<img>` carrying the
    escaped alt. It is reached from `ID_INSERT_PICTURE` (`wx_compose.rs:46`), a real menu item
    at 515 to 517, dispatched at 1230. The sanitiser admits exactly that shape and nothing else
    beginning `data:` (`src/presentation/html_renderer.rs:131` to 150). The send path converts
    it properly: `smtp.rs:176` to 217 rewrites `data:` pictures into `multipart/related` with
    `Attachment::new_inline(content_id)`, because Gmail and Outlook both drop `data:` pictures
    out of a received message, and puts the descriptions into the plain half so it has no silent
    hole.
    Two things are genuinely left, and both are smaller than a build. **There is no decorative
    path**, so criterion 2's "or an explicit mark that the image is decorative" cannot be
    satisfied today: `pictures.rs:352` refuses an empty description, deliberately and with the
    argument written at `wx_compose.rs:2926` to 2931. That is a decision for Pratik, not a
    defect. **Nothing asserts the draft round trip.** The path is `body_from_editor` to
    `HtmlRenderer::sanitize_html` to the drafts table and back through `editor_document`, and
    the sanitiser admits the shape at both ends, so it very likely survives; no test says so.
    That is a red/green pair, not a subsystem.

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
  - Evidence: rewritten 2026-09-04. The previous evidence was true and left the wrong
    impression. "`src/service/pdf.rs` is the only in-app reader" still holds, and everything
    around that reader is generic and built, so this is a producer away from done rather than a
    subsystem away.
    `read_attachment` (`src/presentation/wx_app.rs:18054`) fetches the bytes on a worker and
    posts `UIUpdate::AttachmentRead(Box<ReaderDocument>)`, which opens as a tab of its own.
    `pdf_document` (`src/presentation/reader_text.rs:615`) is the only producer, and nothing
    about `ReaderDocument` is PDF-shaped: it is a title, text, and `Landmark`s the reader
    navigates by. The gate is four lines: `can_be_read_here`
    (`src/presentation/wx_reader.rs:193` to 203) says yes to `application/pdf` or a `.pdf` name
    and nothing else, and the refusal is already written and already names what to do instead
    (156 to 166). The bytes are already cached in the digest-keyed `attachment_content` store
    (`src/data/message_cache/mod.rs:1421` to 1442). So the work is: widen `can_be_read_here`,
    add a text producer and an image producer beside `pdf_document`, and route them.
    **The hard half is criterion 4 and it is blocked upstream.** The description a preview would
    announce is the sender's `Content-Description` header or the `alt` on the `<img>` that
    references the part, and neither reaches the application: `AttachmentInfo`
    (`src/service/mime.rs:48` to 52) carries `filename`, `mime_type` and `size` and nothing
    else. `mail-parser 0.11.5` exposes `content_description`, `content_disposition`,
    `content_id` and `content_language` on `MimeHeaders`, which `mime.rs` already imports, so
    this is a widening of one struct and one function rather than a new capability. It has to
    happen before an image preview can say anything true. Whether real senders supply
    `Content-Description` at all cannot be measured here, because no account has ever been used
    with this program.

  - [S] `docs/development/requirements-backlog.md`, post-v1.0, Medium.
  - [D] An image attachment previews in the application, and the preview announces any alt
    text or description the sender supplied and says plainly when none exists.

  - [D] A text attachment previews as text the screen reader can navigate by line, not as an
    image of text.

  - [D] Attachment content is untrusted input: a preview never executes anything and a file
    that fails to parse is refused with a message naming the file, not rendered partially.

- [ ] **READ-02**: Full PGP encryption and decryption.
  - Evidence: rewritten 2026-09-04. Accurate about PGP, and it understated what sits beside it
    by enough to mis-size the work.
    **PGP is genuinely absent.** The only occurrences in `src/` outside tests are four string
    checks: `detect_pgp_signed` looks for `-----BEGIN PGP SIGNED MESSAGE-----` and
    `-----BEGIN PGP SIGNATURE-----` (`src/service/security.rs:269` to 272),
    `detect_pgp_encrypted` for `-----BEGIN PGP MESSAGE-----` (274 to 276). No key handling, no
    armor parsing, no crate. Re-confirmed 2026-09-04.
    **Six of the eight fields of `MessageSecurityReport` are computed and thrown away.** The
    struct (`security.rs:72` to 83) carries `pgp_signed`, `pgp_encrypted`, `smime_signed`,
    `smime_encrypted`, `signature_status`, `phishing_risk`, `phishing_score` and
    `phishing_indicators`. Its only production consumer, `body_safety::from_body`
    (`src/application/body_safety.rs:45` to 66), reads two of them. So the application already
    works out "this message is PGP-encrypted" on every message it reads and tells nobody, on a
    live path reached from `pop_sync.rs:518` and `wx_app.rs:18525`.
    **S/MIME goes further than "verification", and one part of it is unreached.**
    `signed_mail.rs` carries a DER reader, a certificate store with a real Windows
    implementation, revocation and issuer trust, and signature checking that is reached
    (`wx_app.rs:11181`, surfacing at `reader_text.rs:1023`). `EncryptedMessage`
    (`signed_mail.rs:3645`) reads the outside of a PKCS #7 `EnvelopedData` and its `spoken()`
    (3706) already writes the exact sentence the third `[D]` line asks for, including "This
    computer holds a certificate this message was encrypted to". It has no caller anywhere:
    `grep -rn "EncryptedMessage" src/ tests/` matches only `signed_mail.rs` itself.
    **Nothing goes out signed or encrypted**, so the second `[D]` line is untouched in both
    halves.
    One correction to how the criteria read: an S/MIME enveloped message and a PGP-encrypted one
    fail differently and this requirement treats them as one. An enveloped message has no
    `text/*` part, so `mime::parse`'s `first_of_kind` yields `None` and the message reads as
    empty, which is the failure the third `[D]` names. A PGP armored block is a text part, so it
    renders as the armor rather than as nothing.

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
  - Evidence: rewritten 2026-09-04. "No external spam classifier integration exists" is still
    true and it was the wrong question, because a spam verdict already exists, is stored, listed
    and shown. What the criteria ask for is one entry in one list.
    `src/service/safety.rs` reads the verdict a filter upstream already reached, out of the
    headers `X-Spam-Flag`, `X-Spam-Status`, `X-Forefront-Antispam-Report`, `X-Microsoft-Antispam`
    and `Authentication-Results` (`from_headers`, lines 150 to 168). Its module header states
    the design and the reason: the most reliable free detection available is the detection that
    has already happened, and asking an outside service means handing it links out of private
    correspondence. It is reached from three non-test paths, IMAP
    (`src/service/protocols/imap.rs:1788`), POP (`src/application/pop_sync.rs:515`) and message
    import (`src/data/message_cache/messages.rs:4245`). The verdict is merged with the folder's
    own signal, worst winning (`mail_sync.rs:454` to 458), stored as the `safety` and
    `safety_reasons` columns, shown as a message-list column and in the reader's warning bar.
    **So the first `[D]` line is one addition to one list.** `A_FIELD_A_RULE_MAY_NAME`
    (`src/application/filters.rs:61` to 71) holds eleven names and `safety` is not among them,
    while `CachedMessage.safety` is already on the struct the matcher is handed. The comment at
    `filters.rs:56` to 59 warns that the list and the match arms are held in agreement by a test
    in both directions, so adding one name touches both plus the spoken-words table.
    The second `[D]` line, the verdict shown with its source named, is partly done:
    `Verdict::summary` (`safety.rs:131`) and `safety_reasons` already carry sentences into the
    warning bar. Whether they name the source per reason wants reading before it is planned.
    Whether provider spam headers appear as `safety.rs` expects cannot be settled here: every
    parser in it is tested against hand-written header blocks, and no account has ever been used
    with this program.

  - [S] `docs/development/requirements-backlog.md`, near-term, priority Low; roadmap Phase 5.
  - [D] A classifier verdict is available to the filter rule vocabulary, so a user files spam
    with the rules they already have rather than through a second parallel system.

  - [D] The verdict is shown as a stated score with its source named, never as a silent
    deletion.

  - [D] Guardrail 9 applies: if the classifier is unreachable or returns nothing, the
    application says so rather than treating silence as "not spam".

### The other five modules

- [ ] **PIM-01**: Move a task from one list to another.
  - Evidence: rewritten 2026-09-04. "No move-between-lists path in
    `src/application/tasks_sync.rs`" is literally true and misleading: the path is not in
    `tasks_sync.rs` and never would be. A task made on this computer moves between lists today,
    from a menu, with a key.
    `move_item` is at `src/presentation/managers.rs:6257`, dispatched at 2665, raised from
    `src/presentation/wx_app.rs:3530` to 3531, with the Action menu item carrying
    `Ctrl+Shift+V` at `wx_app.rs:6126` to 6130 and a context menu entry in every module that
    accepts it. The destination chooser is `where_it_could_go` (`managers.rs:6346`), the write
    is `file_under` (6509), and a moved item is marked `pending` so the next sync pushes it.
    **What is left is the provider half, and the requirement does not say so.** A
    provider-held task cannot be moved at all, and that is a designed refusal rather than a
    gap: `moving_can_be_told` (`managers.rs:6464`) asks `tasks_sync::a_provider_holds` and
    refuses with a sentence explaining that neither Google nor Microsoft is asked to move a
    task between lists, so doing it means delete-there, create-here, and writing the new
    identity over the old.
    Two criteria below need re-aiming because of that. The third is about a path that does not
    exist: a provider move is refused before anything is written, so "leaves the task in
    exactly one list" is vacuously satisfied today and stops being satisfied the moment the
    work is done. The delete-then-create sequence is the whole risk in this requirement. And
    the second is not true as written: `grep allowed src/presentation/managers.rs` returns
    nothing, because `Allowed::personal_information` is applied where the HTTP client is built
    (`tasks_api.rs:835`, `google_api.rs:545`, `microsoft_graph.rs:489`, `caldav.rs:203`). A
    move is written locally and marked pending whatever the gate says, and the gate bites at
    the push. Nobody is refused at move time. Whether that is a defect or a mis-worded
    criterion is a decision, because a local file is arguably not a change at a provider.

  - [S] `docs/IMPLEMENTATION_STATUS.md:87` and `docs/ALPHA_TESTING.md:116`, both corrected
    2026-09-04: what is untrue is narrower than "not built". The move ships and has never
    reached a provider, because no account has ever been used with this program. Until that
    correction both documents said moving and copying work for mail only, which is what the
    previous version of this line quoted.
  - [D] A user moves a task to another list by keyboard, and the task appears in the target
    list and is gone from the source list in one action, not two.

  - [D] The move goes through `Allowed::personal_information`, which is on for a new install,
    and is refused with a reason when that gate is off.

  - [D] A move that fails at the provider leaves the task in exactly one list, never in both
    and never in neither.

- [ ] **PIM-02**: Move and copy items in the modules that are not mail.
  - Evidence: rewritten 2026-09-04. The mail half is right: `copy_message` and `move_message`
    are at `src/service/protocols/imap.rs:1280` and 1308. "The inventory records move and copy
    as missing for everything else" is half wrong. **Move ships for three of the five modules**,
    events, tasks and notes, through `move_item` (`src/presentation/managers.rs:6257`); see
    PIM-01 for the trail.
    **Copy is the real work here, and it is not a variant of move.** Nothing in `PimCommand`
    (`src/application/pim_command.rs:20` to 33) copies. `file_under` (`managers.rs:6509`) does
    read-change-write on the same row; a copy needs a new id, a new `pending`, and an answer for
    what happens to a copied provider item. The keyboard half is cheap: `Ctrl+Shift+V` already
    follows the module you are in, and `Ctrl+Shift+Y` (`wx_app.rs:5830` to 5833) would be routed
    the same way.
    **The first `[D]` line contradicts a decision recorded in the code**, which is why it is
    reworded below rather than left standing. `pim_command.rs:49` to 54 gives the reason inline:
    a contact is in as many groups as somebody puts it in, so there is no one home to move it
    out of, and a reminder is filed nowhere because the module sorts by when each is due.
    `new_item.rs:52` to 62 says the same, and `context_menu.rs:426` to 437 is a test holding the
    five context menus to exactly `PimCommand::Move.applies_to`, so widening this to contacts
    and reminders makes that test red. For contacts there is a coherent answer, group
    membership, which `groups_in` (`managers.rs:6661`) already enumerates. For reminders there
    is no container in the schema at all: a reminder has an account, a due time and an optional
    `related_event_id`, and nothing that holds it. Overturning the decision or narrowing the
    requirement is Pratik's call.

  - [S] `docs/IMPLEMENTATION_STATUS.md:93` and `docs/ALPHA_TESTING.md:116`, both corrected
    2026-09-04. Until then both said moving and copying work for mail only, which is what the
    previous version of this line quoted and what `docs/changelog.md:2152` had already made
    false.

  - [D] Events, tasks and notes support move and copy between their containers with the same
    two keyboard commands in every module, because one key means one thing in every module
    here. Move already does. Whether contacts and reminders join them is the open question
    above: the code argues they should not, and for reminders there is nothing to move between.

  - [D] Copy leaves the original untouched and move does not, and each announces which it did.
  - [D] The Action menu carries move and copy because they act on the selection; File, New
    stays for making things.

- [ ] **PIM-06**: Week and month calendar views. Reviewed in 2026-08-29: these do not exist,
  and PIM-03 assumed they did.

  - Evidence: corrected 2026-09-04. The first half was re-checked and is right at the lines it
    quotes: `src/presentation/wx_calendar_module.rs:46` to 58 says the views are not built,
    disables Prev and Next at 55 to 56, and gives them the accessible names "Previous period,
    not built yet" and "Next period, not built yet" at 57 to 58. The last sentence was wrong.
    **The event list is not loaded by account, it is loaded by a fixed window.**
    `events_that_could_fall_between` (`src/data/message_cache/calendar.rs:308`) takes a from and
    a to, and `the_window_now` (`src/presentation/ui_types.rs:1091`) supplies today minus 180 to
    today plus 365. The load path at `wx_app.rs:10834` to 10861 already passes a range, so a
    week or a month view is a narrower and movable window over a query that exists rather than a
    new query, and `the_window_around` (`ui_types.rs:1102`) is already clock-free and is the
    hook. The expansion is also already per-day: `every_day_shown` (`ui_types.rs:1063`) sorts by
    moment and each row carries its stored event's identity, so a week view is a filter and a
    heading over rows that already exist. This makes PIM-06 smaller than the requirement's
    phrasing implies. The accessibility criterion below is unaffected and stays the hard half.

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
  - Evidence: rewritten 2026-09-04. The previous evidence was true and understated what ships
    by enough to mis-size the work. `src/application/occurrences.rs` and `repeating.rs` do hold
    the recurrence model, and the expansion is already wired into the list the user sees.
    `falls_on` (`occurrences.rs:62`) expands a rule into every day it falls on and removes the
    days EXDATE calls off through `days_called_off` (defined at 148, called at 98).
    `every_day_shown` (`src/presentation/ui_types.rs:1063`) turns that into one list row per
    day, from two production call sites, `wx_app.rs:10861` and `managers.rs:1187`. **A weekly
    meeting already appears on every week in the event list**, so the first `[D]` line is
    already true of the only view that exists.
    **What is left is the second `[D]` line, a moved occurrence.** An override, a single
    occurrence moved to another date, is stored as its own row carrying
    `provider_recurrence_id`, and `falls_on` knows nothing about it: it filters by EXDATE only.
    So a moved occurrence appears twice, once expanded from the series on its original date and
    once as its own row on the new one. That was established by reading rather than by running,
    so it is the first failing test any PIM-03 plan should write: if it turns out to be handled
    somewhere, this requirement is nearly closed and a plan would otherwise build a second
    de-duplication beside a working one.

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
  - Evidence: re-checked 2026-09-04 and still accurate. `ls src/application/*sync*.rs` returns
    `caldav_sync`, `collection_sync`, `contacts_sync`, `mail_sync`, `pop_sync`, `sync_marker`
    and `tasks_sync`, and no `notes_sync.rs`.
    **Three of the six criteria below already ship, and the requirement did not know it.** The
    content model was not invented twice: `long_text.rs` is the shared Markdown reader,
    `pulldown-cmark` is imported there at line 17, and the notes editor labels its box "Body, in
    Markdown" (`src/presentation/wx_notes_module.rs:83` to 94). The stored form is the Markdown
    source: `long_text.rs:13` to 15 states it as the module's rule and `save_note`
    (`src/data/message_cache/notes.rs:99`) stores the body verbatim. And a screen reader reads
    the rendered structure: `read_aloud.rs:351` does for a note exactly what `read_aloud.rs:332`
    does for a contact's notes, which is the precedent this requirement names. What is left of
    PIM-04 is the three criteria about sync, and those are PIM-07's work.
    **One loose end inside PIM-04.** The `notes.format` column is written as the literal
    `"plain"` at six sites and read by no production code, while the editor and the reader both
    treat the body as Markdown. It is a stored answer nothing asks, which is the shape this
    project keeps having to remove. Either it becomes meaningful, or it goes, or the phase
    records why it stays.

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
  - Evidence: re-checked 2026-09-04 and still accurate. Neither "VJOURNAL" nor "OneNote" occurs
    anywhere in `src/`. The existing `*_sync.rs` files are the shape to follow, and this is the
    phase's largest genuine build.
    Two things the previous evidence did not carry that a plan needs. **The schema work is
    real.** The `notes` and `note_folders` tables (`src/data/message_cache/mod.rs:2028` to 2056)
    carry no `pending`, no provider id and no version marker, and `NoteEntry` has nine fields,
    none of them a sync field. Every other synced kind has all three, and the project's rule is
    additive columns through `ensure_column_exists`. **A green test asserts the opposite of this
    requirement and nothing names it.** `test_notes_are_not_offered_a_sync_they_cannot_do`
    (`src/application/context_menu.rs:611` to 620) asserts the note-folder context menu does not
    offer `Action::SyncNow`. It is correct today and must be inverted in the same commit that
    adds a backend.
    `new_item.rs:16` to 33 is where the "backend chosen by account type" reasoning already
    lives, in prose, and its `syncs` predicate returns `false` for `Note` today. That predicate
    is the natural seam.

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
  - Evidence: none, which its own line already admitted, and that is still true on 2026-09-04.
    This is preparation for a service that does not exist yet, so it is the requirement most at
    risk of building for an imagined shape.
    One thing found on 2026-09-04 that reduces that risk: the seam to imitate already exists in
    this tree rather than needing to be invented. `AddressBook`
    (`src/data/message_cache/mod.rs:423` to 446) is three variants where the third is
    `Other(String)`, with a doc comment saying it exists so that a word this code does not
    recognise survives being read and written back. That is a shipped, working example of a seam
    that does not forbid a later implementation, and it is the model to copy.

  - [D] The seam PIM-07 defines takes a hosted backend as one more implementation, with no
    change to the stored form and no migration of existing notes.

  - [D] What the seam assumes about a backend is written down: identity, conflict resolution,
    and what happens to a note whose backend is removed from an account.

  - [D] Nothing in this milestone ships a hosted client, a network call to one, or a setting
    offering one. Preparing for it means the seam does not forbid it, not that anything half
    exists. A switch that does nothing is the failure this project has fixed repeatedly.

- [ ] **PIM-05**: CardDAV for contacts.
  - Evidence: corrected 2026-09-04. Right about the gap, wrong about where the vCard code
    lives, in a way that would send a plan to the wrong file.
    The gap is real: `src/service/caldav.rs` covers calendars only, and
    `grep -rni carddav src/` on 2026-09-04 returns five hits, all inside `#[cfg(test)]` blocks.
    There is no PROPFIND for `addressbook-home-set`, no `discover_address_books` and no
    address-book URL anywhere.
    **The vCard reader and writer are in the data layer, not in the two files named.**
    `importing_contacts.rs` holds one public function, `what_the_card_import_did`, and it builds
    a sentence. The reader and writer are `import_contacts_from_vcard`
    (`src/data/message_cache/contacts.rs:312`) and `export_contacts_to_vcard` (line 728). A plan
    told to reuse them at the old addresses would find a wording helper.
    Three things that make the build smaller than it reads. The HTTP verbs are already there:
    `AskWith::{Propfind, Report}` (`src/service/outward.rs:90` to 93), which is what CardDAV
    needs. Discovery is the same shape as `discover_calendars` (`caldav.rs:228` to 273) against
    a different namespace. And `AddressBook::Other("carddav")` already round-trips, with the
    per-address-book `provider_version` column being exactly where a CardDAV ETag belongs, so no
    schema change is needed to name the new address book.
    Two things that make it larger. **A new CardDAV file must be added to
    `FILES_THAT_READ_OR_WRITE_A_DOCUMENT` (`caldav.rs:4702`) or it is unguarded**, because the
    case-folding guard over `VCARD`, `FN`, `EMAIL` and eighteen other names reads only the files
    that array lists, and the array's own comment says a name left off it is a name the reading
    has stopped checking. **And this must not grow its own conflict model.** A CardDAV address
    book is a third source of contacts flowing into `whose_copy_wins`
    (`src/application/contacts_sync.rs:988`), and its ETag is the same kind of marker
    `caldav_sync` uses. Plan 03-09 builds `conflict_choice.rs` in phase 3's last wave; whether
    PIM-05 waits for it or plugs into it as a consumer is a scheduling decision, and building a
    second model beside it is the mistake SCALE-06 was corrected to avoid.

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
  - Evidence: re-checked 2026-09-04 and still accurate. It is one of five in the 18-requirement
    audit that was, and the case is stronger than the previous evidence claimed.
    `src/presentation/accessibility/feedback.rs` holds the model: `per_event` (line 392) and
    `set_event_channels` (line 424) are both **private**, `fn` and not `pub fn`, so no screen
    could write one without changing a visibility. `channels_for` is at 433 and serialisation is
    `to_stored` (458) and `from_stored` (478). The only shipping caller of `set_event_channels`
    is `from_stored`, at line 496; every other caller is a test. **So a per-event override can
    enter this program only by somebody hand-editing the stored config string.** The settings
    screen goes out of its way to preserve overrides it cannot create and says so at
    `src/presentation/wx_settings.rs:2123`. The reading half is fully live: `channels_for` is
    called on the shipping path at `src/presentation/accessibility.rs:213` and 235.
    **There are now two settings of this shape, not one**, which is the argument for the last
    `[D]` line below. The second is the per-account Allow Changes answer, recorded at
    `docs/changelog.md:5296`: still read and still honoured, and nothing writes one and no
    screen offers it. That one already has its guard,
    `test_nothing_offers_a_setting_per_account_that_no_screen_writes`
    (`tests/house_style.rs:152`). This one does not.
    **One number outside this requirement, found stale and since corrected.** The grid is 16
    events by 4 channels: `Event::ALL` is `[Event; 16]` at `feedback.rs:114` and all sixteen
    have a non-test call site. `docs/changelog.md:8393` called it "nine events by four channels"
    when this was audited and now says sixteen, corrected 2026-09-04 in the same pass that
    corrected this document.

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
  - Evidence: rewritten 2026-09-04. The core claim holds and half of this is already done.
    **The order of the day and month, and the clock, already follow the machine.**
    `DateOrder::from_system` (`src/presentation/date_display.rs:226`) reads the Windows locale
    through `read_locale` (line 136) and `order_from_locale` (169), and it is the default at
    line 71; the clock follows through `clock_from_locale` (183). So the first `[D]` line's
    "`DateOrder` follows it rather than being a separate setting the user has to find" is
    already true, and what is left is the strings and only the strings.
    `MONTHS` is a hardcoded English array at line 100 (101 when this was written), formatted
    from at 384 and 456, and offered as a choice at `src/presentation/wx_item_form.rs:844`.
    Relative wording such as "2 days ago" is English, built by `plural` and tested at
    `date_display.rs:860`.
    **The limitation is already disclosed in the product rather than left to be discovered.**
    `date_display::ENGLISH_ONLY` (line 93) is on the settings screen
    (`src/presentation/wx_settings.rs:1251`) and in `docs/accessibility.md`, and the reasoning
    at lines 80 to 92 says why it matters: an English month name inside a French date, read with
    French pronunciation, sounds like the screen reader misbehaving.

  - [S] `docs/changelog.md:8005`. The previous citation, line 6946, is now a paragraph about
    filter rules that move a message to a folder.

  - [D] Month names, day names and relative wording ("2 days ago") come from the machine's
    locale, and `DateOrder` follows it rather than being a separate setting the user has to
    find.

  - [D] Reworded 2026-09-04 to match what shipped, which is better than what this line asked
    for. A locale with no translation falls back to English. The fallback is said once, where
    somebody can act on it, and not on every row: that is `date_display::ENGLISH_ONLY` and it
    already ships. The previous wording asked that the fallback say nothing at all in the UI,
    "because a visible fallback notice on every row is worse than the fallback", which reads as
    contradicting the code and does not: the code agreed about the rows and disagreed about
    saying nothing.

  - [D] The existing tests over `date_display` keep passing under a forced English locale, so
    the change is testable without a machine set to another language.

- [ ] **FEEDBACK-03**: Know how much of WCAG the automated scans actually cover.
  - Evidence: re-checked 2026-09-04 and still accurate, every anchor.
    `.github/workflows/accessibility.yml` runs Axe.Windows over UI Automation and
    `scripts/msaa-names.ps1` over MSAA, per window; `.github/workflows/nvda.yml` drives a real
    copy of NVDA. `docs/IMPLEMENTATION_STATUS.md:130` records five findings at the last read, on
    2026-07-26, all inside WebView2's own accessibility tree, and line 136 records roughly half
    of WCAG covered. None of the three `[D]` lines below has been acted on: nothing in the tree
    names which WCAG 2.2 AA criteria the scan can and cannot judge, the five findings have not
    been re-read, and there is no written list of interactions a manual pass has to walk.
    One thing to add rather than correct: that "last read" is now five and a half weeks old, and
    the workflow is non-blocking (`docs/IMPLEMENTATION_STATUS.md:112`), so nothing forces a
    re-read. A scan whose result nobody is made to look at is guardrail 4 waiting to happen.

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
  - Evidence: re-checked 2026-09-04 and accurate about the tree, every clause.
    `installer/Wixen-Mail-Setup.iss` builds an Inno Setup installer;
    `scripts/build-installer.sh:29` to 48 appends the commit it built from, and appends nothing
    at a tag; `docs/ALPHA_TESTING.md:146` states the installer is not signed. The evidence was
    only ever wrong about Windows, which the SmartScreen paragraph below now covers.
    **One thing it does not say that changes the size of the work.** The first `[D]` line says
    "the installer and the executable inside it". There are three signable artefacts inside it
    (`wixen-mail.exe`, `wixen_mail_search.dll`, `wixen-mail-search-setup.exe`, at
    `Wixen-Mail-Setup.iss:99` to 121) and three more published beside it by
    `.github/workflows/release.yml:131` to 135 (the setup, a portable copy of the same binary,
    and a zip of it), plus the uninstaller, which Inno signs only if `SignedUninstaller` is set.
    A plan that signs "the installer and the exe" leaves four things unsigned, two of which are
    executables a user runs. `release.yml:136` sets `fail_on_unmatched_files: false`, so an
    asset that stops being produced is published silently as an absence.

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
  - Evidence: re-checked and widened 2026-09-04. Confirmed absent, and this is the rare absence
    claim in this document that survived being searched properly.
    `grep -rniE "check_for_update|auto_update|update_check"` over `src/`, `search-handler/` and
    `tests/` returns no source hits. A concept-level search for "newer version", "latest
    release", "update available" and "self_update" over `src/` and `docs/*.md` returns 30 hits
    and not one is about program updates: every one is forward compatibility, a saved search, or
    a column written by a newer version of Wixen Mail. So there is no adjacent thing that could
    be mistaken for this.
    **It is cheaper than the requirement implies.** `reqwest` is an unconditional dependency
    (`Cargo.toml:72`), not Windows-gated, so an HTTPS GET of the releases API needs no new
    dependency and works on the platforms SHIP-05 is about. There is no SemVer parsing or
    comparison anywhere, and no `semver` crate in `Cargo.toml`: `src/common/version.rs` only
    formats, and `describe` already proves the `+build` metadata is separated by a `+` so it can
    be ignored. Comparing two `0.x.y` strings is the one new piece of logic, and it is a handful
    of lines with a clear test surface.
    **One sentence elsewhere expires when this ships.** `docs/privacy.md:7` says there is "no
    update check that says who you are". An anonymous check keeps that technically true, and a
    request to GitHub still reveals an IP address and a rough version-to-user mapping. That
    sentence wants re-reading in the same change, not after it.

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

  - Evidence: re-verified line by line on 2026-09-04 and correct.
    `installer/Wixen-Mail-Setup.iss` line 84 declares a `desktopicon` task, and the `[Icons]`
    block at lines 123 to 125 creates both the Start menu entry (line 124) and the desktop
    shortcut (line 125). Inno removes both on uninstall. `SetupIconFile` at line 49 points at
    the bundled `assets/icon.ico` and that file exists, but neither `[Icons]` entry sets
    `IconFilename`, so the shortcuts use whatever icon the executable carries. Inno creates no
    shortcuts by default; an earlier version of this line said it does, inherited from the
    inventory.
    **One correction: the shortcuts are not iconless today.** `build.rs:34` already embeds
    `assets/icon.ico` into `wixen-mail.exe`, so they inherit the executable's icon. That makes
    this a smaller correctness fix than "the shortcuts have no icon" would suggest, and it means
    the test for it cannot be a screenshot. The test that can be written is the one the tree
    already uses three times: read the `.iss` as text.
    **A gate hole to decide about first.** A branch commit touching only the `.iss` runs none of
    the three existing `.iss`-reading tests, because `guards/guards.toml` names the installer
    nowhere and neither `house_style` nor `wired` reads it. A fourth such test inherits the same
    hole. The plan either widens `scripts/which-checks.sh` to answer `all` for `installer/*.iss`
    the way it already does for `Cargo.toml`, or says plainly that it is accepting the gap. It
    should not add the test and leave the hole unmentioned.
    (This line also carried a corrupted path for some time: an earlier editing pass turned the
    backslash-a of the Windows path into a control byte, so it read `SetupIconFile=..ssets`.
    Rewritten with forward slashes on 2026-09-04 so it cannot happen again.)

  - [S] `docs/development/requirements-backlog.md`, platform, priority Medium.
  - [D] Both `[Icons]` entries set `IconFilename` to the bundled icon, so the shortcut a user
    sees in the Start menu and on the desktop is the application's own.

- [ ] **SHIP-04**: Encrypt the local cache, or decide not to and say so once and clearly.
  - Evidence: re-checked 2026-09-04 and accurate. `src/data/message_cache/mod.rs` stores mail
    in plain SQLite; `rusqlite` carries no SQLCipher (`Cargo.toml:81`), so encrypting the cache
    means SQLCipher or an application-level scheme, which is the build cost `CLAUDE.md` names.
    `src/service/security.rs:222` and 423 both use `Aes256Gcm::new_from_slice`, so the primitive
    is in the tree.
    **The decision is already said in the documents and said nowhere in the product**, which is
    exactly the split the first `[D]` line below asks about, so it is worth being precise. It
    ships in `docs/installing.md:75` to 79, in bold and with the drive-removal and BitLocker
    distinction; in `docs/privacy.md:27` to 30, plus line 38 for attachments and 52 for the
    byte-for-byte copies of signed mail, which are two more unencrypted things neither the
    roadmap nor this requirement mentions; in `docs/ALPHA_TESTING.md:143`; and in two installer
    dialogs. It does not ship in the running program: `grep -i encrypt
    src/presentation/first_run.rs` returns nothing, and the end of `--help`
    (`src/presentation/command_line.rs:133` to 137) says everything that writes is experimental
    and says nothing about the cache.
    **A caution on where it goes, from the screen's own doc comment.** `first_run.rs:118` to 120
    says `INTRODUCTION` "is read out in full by a screen reader before the person reaches the
    buttons, so anything not worth hearing every time does not belong here". Appending a
    paragraph about disk encryption to that constant makes every first run longer for every
    user. The screen already has the pattern for this, a `READ_MORE` button that opens a shipped
    document. Which of the two is a design question for the plan to raise rather than settle by
    appending.
    **One accuracy gap in the "what is stored" pages.** `docs/installing.md:61` to 67 and
    `docs/privacy.md:15` to 21 list four subfolders and omit `security.key`, which
    `src/common/paths.rs:98` to 100 places in the root. It is a legacy artefact that
    `security.rs:157` to 163 says is never created any more and is only read to migrate an
    upgraded machine, so the listing is right for a fresh install and wrong for an upgraded one.

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
  - Evidence: corrected 2026-09-04, having been split from the disclosure below on 2026-08-29
    because building on a platform and telling its users what does not work there are different
    pieces of work with different gates. Right in substance, wrong on one line number, and it
    puts the cost in the wrong place. Roadmap Cross-Platform is unticked. `Cargo.toml` gates Windows dependencies behind
    `[target.'cfg(windows)'.dependencies]`. Nothing in CI builds the crate off Windows: every
    `runs-on:` in `.github/workflows/` is `windows-latest` except one `ubuntu-latest` job, which
    runs `cargo audit` and reads `Cargo.lock` without building. Ten Windows jobs across five
    workflow files, one Ubuntu job.
    Corrected 2026-09-04: that job is the `audit` job at `ci.yml:166`, not line 133. Line 133 is
    inside a cache step of a Windows job. Drift rather than a substantive error, and it is the
    reason this document now asks for symbols instead of line numbers.
    **The cost is not in the CI YAML, and the requirement reads as though it were.**
    `wxdragon` is pinned at `=0.9.17` with the `aui`, `richtext` and `webview` features
    (`Cargo.toml:75`), and it vendors and statically builds wxWidgets from source rather than
    linking a system library. A Linux job is therefore a source build of a C++ GUI toolkit plus
    a system package install including `libwebkit2gtk-4.1-dev` for the webview, and a macOS job
    is the same again on a more expensive runner. Whether that is minutes or tens of minutes has
    not been measured, and it should be before a plan promises "another CI job". If it turns out
    not to build at all, SHIP-05 is a port rather than a CI change, which is a different phase.
    **The search handler is a second crate.** `search-handler/` is built separately by
    `build-installer.sh:170` and linted separately at `ci.yml:150` to 153, and it is a Windows
    COM server. "The crate builds on Linux" means the main crate only, and the plan should say
    so rather than leave somebody to find out.

  - [S] Roadmap Cross-Platform, unticked.
  - [D] A CI job builds the crate and runs the test suite on Linux, and another on macOS, so a
    Windows-only assumption fails on the commit that adds it rather than on the day somebody
    tries a port.

  - [D] No criterion here claims the application is accessible on Linux or macOS, or usable
    there. Building is not the same claim.

- [ ] **SHIP-06**: Off Windows, the application says what its accessibility layer does not do.
  - Evidence: rewritten 2026-09-04. The previous evidence reads as though nothing exists, and
    almost all of it does. This is "route an existing fact to two places" rather than "build a
    disclosure".
    `CLAUDE.md` still records that `wxAccessible` and `UiaRaiseNotificationEvent` exist only on
    Windows and that both compile and silently do nothing elsewhere. That much is unchanged.
    **The derivation is built and reached.** `ScreenReaderBridge::default()`
    (`src/presentation/accessibility/screen_reader.rs:650` to 665) sets `status` to
    `NativeBridgeStatus::Active` on Windows and `NativeBridgeStatus::Fallback` everywhere else;
    the enum is at 425 to 431, every call into the native layer is gated, and the tree carries 94
    `target_os = "windows"` gates in total.
    **The accessor is built and nothing calls it.** `ScreenReaderBridge::status` is at line 636
    and `Accessibility::native_bridge_status` wraps it at
    `src/presentation/accessibility.rs:429`. `grep -rn native_bridge_status src/ tests/` on
    2026-09-04 returns exactly one hit, its own definition: not one caller, not even a test. It
    is `pub`, so no `dead_code` warning fires, and `tests/wired.rs` cannot see it because that
    guard is about command ids raised and handled rather than public functions with no callers.
    That is guardrails 1 and 3 in the same function.
    **One caveat on the second `[D]` line below.** It asks that the disclosure be derived from
    what is compiled in rather than from a hardcoded platform list. `cfg!(target_os = "windows")`
    is a platform list of one. It is better than a runtime string comparison, and if a macOS
    bridge is ever written that expression does not change on its own and the warning would keep
    appearing. Deriving it from whether a bridge function is present is what would actually
    satisfy the criterion, and that is worth naming in the plan rather than letting the existing
    expression pass as compliance.

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
  - Evidence: re-checked 2026-09-04 and still accurate. No `benches/` directory, no `criterion`,
    no `divan` and no `[[bench]]` in `Cargo.toml`, and nothing in `src/` reads resident memory,
    so no target below has a number attached. The source of the target is
    `docs/development/requirements-backlog.md:81`, "Memory profiling | Target <150MB with 1000
    cached messages | Medium".

  - [S] `docs/development/requirements-backlog.md`, performance and scale, Medium.
  - [D] A repeatable measurement produces a number for resident memory with 1,000 cached
    messages, recorded with the date, the machine and the build it came from.

  - [D] The number is either under 150 MB or the target is revised with the reason, rather
    than the target quietly remaining aspirational.

- [ ] **PERF-02**: Cold start under 2 seconds, measured.
  - Evidence: re-checked 2026-09-04 and still accurate. `docs/roadmap.md:221` still reads
    `- [ ] Startup time optimization (<2 seconds)`, unticked, and line 253 repeats it as a
    success metric; `docs/development/requirements-backlog.md:82` carries it as Medium. Nothing
    in `src/` times process start against a usable list. The "under 2 seconds" is a target
    rather than a measurement and is correctly written as one.
  - [S] Roadmap Phase 8; `docs/development/requirements-backlog.md`.
  - [D] Cold start is measured from process start to the message list being usable, not to the
    window appearing, because an empty window is not a usable inbox.

  - [D] The measurement is repeatable and recorded with the date, the machine and the build.

- [ ] **PERF-03**: A real mailbox of 100,000 messages or more, exercised.
  - Evidence: rewritten 2026-09-04. "The largest thing exercised is a loopback server" is
    false. A 200,000 row sample mailbox generator ships in the product, on the Help menu, put
    there deliberately so that a screen reader user can arrow through one.
    `SAMPLE_MAILBOX_SIZE` is 200,000 at `src/presentation/wx_app.rs:9125`, `sample_mailbox` at
    9137 builds the rows, and `ID_LOAD_SCALE_SAMPLE` (line 93, item at 6281, handler at 4908) is
    on the Help menu rather than behind a build flag, because as its doc comment says, the
    people who most need to test it are not the people compiling it.
    **So the first `[D]` line is half satisfied: the mechanism exists and is reachable, and the
    numbers do not.** Nothing in the tree records a sort, filter or scroll timing from a sample
    run.
    **The second `[D]` line is not satisfied and is the part to keep.** The virtual text
    callback at `wx_app.rs:1101` reads the whole loaded list out of `state.messages` in memory
    and never touches SQLite, which a comment at 1093 states, but no test asserts it. Nor is the
    mail-at-scale plan's paged design built: there is no page cache of 200 rows around the
    viewport and no placeholder on a cache miss, because the whole list is held in memory.
    `message_rows::PLACEHOLDER` exists and is returned only when the row index is past the end
    of the loaded list.
    The third `[D]` line, that no criterion here claims a real provider mailbox was used, is
    still correct and still important.

  - [S] Roadmap Phase 8; `docs/plans/20260726-mail-at-scale.md`.
  - [S] Sorting 200,000 rows in memory on a header click is a multi-second freeze, and a freeze
    is an accessibility failure, not a performance one.

  - [D] The list is exercised against 200,000 synthetic rows, which needs no network and no
    live account, and the sort, filter and scroll paths each produce a recorded number.

  - [D] A test asserts the virtual text callback issues no SQLite query.
  - [D] No criterion here claims a real provider mailbox was used. Synthetic rows answer the
    list question; the provider question waits for a live account.

- [ ] **PERF-04**: Idle memory under 100 MB, measured.
  - Evidence: re-checked 2026-09-04 and still accurate. `docs/roadmap.md:254` reads
    `- Low memory footprint (< 100MB idle)` under success metrics, with no measurement anywhere.
  - [S] Roadmap success metrics.
  - [D] Idle memory is measured after startup with a cache present and no user activity, and
    recorded with the date, machine and build.

  - [D] The number is either under 100 MB or the target is revised with the reason.

- [ ] **PERF-05**: Line coverage re-measured.
  - Evidence: corrected 2026-09-04. The coverage figure and its date are still right and the
    commit count attached to them was out by a factor of about four.
    60.4%, measured 2026-07-26 with `cargo llvm-cov --lib --summary-only`, stale since, and
    still recorded that way at `docs/IMPLEMENTATION_STATUS.md:201`.
    `git rev-list --count --since="2026-07-26" HEAD` on 2026-09-04 gives **1,195** commits
    landed since, out of 1,373 in the repository, first commit 2026-02-13. So 87% of this
    project's history postdates the reading. The "roughly 275" this line used to give was itself
    a stale count when it was written and had never been re-taken, which is the same defect this
    requirement is about, in the requirement about it.

  - [S] `docs/IMPLEMENTATION_STATUS.md`.
  - [S] Coverage is the cheap wide sweep answering only "what never runs at all", and low
    coverage in `service/protocols`, `service/oauth` and the provider clients is the network
    transport that has never met a live account, tracked as work rather than as a testing gap.

  - [D] A current number replaces the stale one, with its date, and the low areas are
    attributed rather than treated as a number to raise.

- [ ] **PERF-06**: Every document that quotes a test count quotes the same measurement.
  - Evidence: rewritten 2026-09-04. The three-way disagreement this evidence described is
    closed, and the reconciled number has since moved, which is exactly what the third `[D]`
    line says a check must not treat as a failure.
    All three documents now agree. `docs/IMPLEMENTATION_STATUS.md:123` reads "5,430 tests pass:
    5,269 unit and 161 integration, counted 2026-08-29 with
    `cargo test --all-targets -- --list`", and line 124 names the superseded 3,362 from
    2026-08-09 as what a number without its command and its date turns into.
    `docs/changelog.md:1276` and 1280 and `docs/integration-guide.md:5` carry the same figures
    with the same split. So the first two criteria below, the command with its date and the
    unit-against-integration split, are already met by all three.
    Re-measured 2026-09-04:

    ```
    cargo test --lib -- --list            counts 6,079      2026-09-04
    cargo test --all-targets -- --list    counts 6,271      2026-09-04
    ```

    which makes the integration and other-target figure 185. The documents are therefore about
    810 unit tests and 24 integration tests behind. Under the third criterion below that is a
    stale measurement to refresh rather than a check to fail, and refreshing it is the remedy.
    **One number in the documentation was stale in a way that is a defect rather than a drift,
    and it has since been fixed.** `docs/changelog.md:8393` called the per-event feedback grid
    "nine events by four channels" when there are sixteen
    (`src/presentation/accessibility/feedback.rs:114`, `Event::ALL` is `[Event; 16]`). Corrected
    2026-09-04. See FEEDBACK-01.
    **On durations.** `CLAUDE.md` says a duration is the same kind of claim as a count, and two
    inside this milestone carry no conditions. "A whole-tree mutation run is about two days"
    appears at `docs/IMPLEMENTATION_STATUS.md:156` and `CLAUDE.md:323` with no machine, no date
    and no thread setting, while `CLAUDE.md:465` gives "about 15 hours" for the 564-record guard
    sweep after `WIXEN_TEST_THREADS` halved it. Those are two different jobs, and a reader
    planning phase 8 has no way to tell which conditions either was taken under.

  - [S] `docs/IMPLEMENTATION_STATUS.md`.
  - [D] Every count in the documentation carries the command it came from and the date it was
    taken, so a number that has moved reads as a stale measurement rather than as a current
    fact.

  - [D] The count is split unit against integration. That was the distinction the three numbers
    in the tree used to disagree about; as of 2026-09-04 all three carry the split, and this
    line now says keep it rather than add it.

  - [D] Added 2026-09-04. A duration quoted in the documentation carries its conditions, because
    `CLAUDE.md` treats a duration as the same kind of claim as a count: the machine, the date,
    and any setting that changes it, such as `WIXEN_TEST_THREADS`. A figure with no conditions
    is quoted to somebody planning work as though it transferred, and it does not.

  - [D] Nothing asserts that a number written in a document equals what `cargo test` reports.
    Corrected 2026-08-29: that is what this requirement used to ask for, and it is false the
    next time anyone adds a test. A check on a number here checks that its command and its date
    are present and that documents agree with each other, never that the number has not
    moved.

- [ ] **PERF-07**: A whole-tree mutation run, once, with a real result.
  - Evidence: rewritten 2026-09-04. "A whole-tree run has never been done" is wrong, and the
    difference matters to whoever plans one: reading "never been done" plans a first run without
    knowing what shape of failure to expect.
    Scoped runs plus one untrustworthy whole-tree run. mime and error on 2026-07-26; filters,
    due, tagging and signatures on 2026-08-01 with 157 mutants; the four message-disposition
    modules on 2026-08-12 with 66 mutants and 1 survivor. **A whole-tree run was attempted on
    2026-08-05** and is recorded at `CLAUDE.md:544`: it marked 595 mutants unviable, of which
    473 had never reached a compiler, so about a third of it was untested and its summary said
    so nowhere. That run is why `scripts/mutants.sh` now refuses a partial run, a run whose
    build failed before anything changed, and a run in which the suite was never once run
    against a mutant. So what has never happened is a whole-tree run **with a result anybody can
    trust**, not a whole-tree run.
    `guards/guards.toml` holds **565** records, counted 2026-09-04 with
    `grep -c "^\[\[guard\]\]" guards/guards.toml`; the previous evidence said 501, and
    `CLAUDE.md:474` says 564 and is one behind. The "192 hand-verified as of 2026-08-12" figure
    has no counterpart in the file today: every record carries a name and a measured red list,
    so there is no marker separating verified from unverified and nothing for that number to
    compare against.
    The two `[S]` claims below about the script are unchanged and still accurate.

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
| SCALE-04 | Phase 3 | Complete |
| SCALE-05 | Phase 3 | Complete |
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

- v1 requirements: 44 total
- Mapped to phases: 44
- Unmapped: 0

**Corrected 2026-09-04.** This block said 40 and 40. Counted from the file on 2026-09-04 with
`grep -c '^- \[[ x]\] \*\*[A-Z]\+-[0-9]\+\*\*' .planning/REQUIREMENTS.md`, which gives 44, and
the traceability table above has 44 rows. The 40 was right when the section below was written
and three later splits added four requirements without the total being re-taken: PIM-04 became
PIM-04, PIM-07 and PIM-08 when Pratik decided on 2026-08-29 that notes have a backend per
account rather than one target; PIM-06 was cut out of PIM-03 the same day, once week and month
views turned out not to exist; and SHIP-05 was split from SHIP-06 because building on a platform
and disclosing what does not work there are different pieces of work with different gates. Each
of those three splits is recorded in the requirement it came from. Nothing was added or dropped
without a note, so the count was the only thing that fell behind.

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

The 32 and the 8 add to 40, which is what this section counted and what the coverage block above
said until 2026-09-04. There are 44 now. The four extra came from three splits made after this
section was written, all of them recorded in the requirements they came from, and the totals here
were never re-taken. Read the arithmetic above as the accounting at the moment of writing rather
than as a current count.

**Discrepancy, resolved 2026-08-29.** The brief said the first section has 27 rows. The file
has 33, and 33 is right. The 27 was quoted from the inventory agent's summary of the document
it had just written, and reached the brief without anyone counting the file. All 33 rows are
accounted for above, so nothing was dropped.

---
*Requirements defined: 2026-08-29*

*Last updated: 2026-09-04. Thirty-seven of the forty-four Evidence blocks were touched against
the tree at commit `d3c6c7d`, drawing on four research passes: the audit in
`.planning/requirements-audit-2026-09-04.md`, covering FOLDER, THREAD, SEARCH, FEEDBACK and
PERF, and the phase research documents for 4, 5 and 7. Twenty-four of those were wrong and were
rewritten; the other thirteen were re-checked, found accurate, and given a dated command or
symbol so that the next reader can tell a verified claim from an unexamined one. Every wrong
block was wrong in the same direction, saying something was missing that had since shipped or
naming a defect that had since been fixed. Nothing over-claimed.*

*Not touched: the six SCALE blocks, corrected on 2026-09-03 from phase 3's research, which
nothing in the later passes contradicts; and WRITE-03, corrected earlier on 2026-09-04. The
coverage total was also corrected, from 40 to 44. Every [D] line is still awaiting Pratik's
review.*

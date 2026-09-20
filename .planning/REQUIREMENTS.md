# Requirements: Wixen Mail

**Defined:** 2026-08-29
**Core Value:** Making correspondence and personal information legible to people who cannot see it.
**Milestone:** The outstanding work. Drawn from the two "not built" sections of
`.planning/intel/built-and-left.md` and from nothing else. **Corrected 2026-09-16:** "and
from nothing else" was true until the first day of manual testing. Phases 1 to 8 are executed
and merged, and the milestone's verification is the testing Pratik began on 2026-09-15 with
build `0.125.1+g3e633252`, which produced 44 GitHub issues, #20 to #63, in one day. Phase 9's
requirements, `FOUND-01` to `FOUND-12`, are drawn from those issues and from nothing else; the
section "What the first day of testing found" says which issue each one comes from. Phase 10's,
`MAIL-01` to `MAIL-05`, added 2026-09-17, are five more of the same issues, the third of the
seven groups; the section "All the mail, and what is said while it comes" names each. Phase
11's, `LIST-01` to `LIST-10`, added 2026-09-18, are the fourth group with #70 and #71 in
front; the section "Reading, and the list" names each.

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
    Contacts already resolves. `whose_copy_wins` (`src/application/contacts_sync.rs`)
    returns a four-armed `WhoseCopyWins` built from whether local work is unsent and whether
    the address book's version marker moved, comparing markers rather than clocks on purpose.
    It has two production call sites reached from `wx_app.rs`, and the losing case was counted
    and spoken rather than silent.
    CalDAV has the markers and not the choice: `etag` and `If-Match` in `caldav_sync.rs`.
    Mail is a third case and is not last-write-wins. A flag change is applied locally, pushed
    on a connection, and reverted per flag kind with a sentence if the push fails. Nothing
    queues a mail flag change, so the "both changed" state this requirement describes largely
    cannot arise there.
    So the deliverables below want aiming at contacts and CalDAV, where the state exists, and
    the mail case wants restating. The five `*_sync.rs` files total over 38,000 lines and none
    has met a live account.

  - Corrected again 2026-09-05, while executing plan 03-09, and three things in the paragraph
    above turned out to be wrong or misleading.
    **The losing case was told, not asked, and it was told through the wrong counter.** The
    evidence named `sent_over_a_newer_copy` as the telling. That counter is the other
    direction: it counts a change made here that the push re-sent *over* the address book's
    newer copy after a stale-marker refusal, which is a second both-changed state resolved in
    this computer's favour, at the provider, with a sentence afterwards. The losing case's
    counter was `replaced`, and it is `held_for_you_to_choose` now.
    **CalDAV did not resolve in the server's favour, it resolved in this computer's.** The
    read skipped any event with a change waiting, so the server's copy was dropped with
    nothing said at all. Opposite direction, same failure.
    **Mail's real defect is now fixed rather than only named.** A flag change made while the
    server could not be reached was silently reverted; it is kept and sent later. A change the
    server answered and refused is still put back, because a server that answered is a
    different fact from a server that was not there. Task 4 of plan 03-09 is that work.
    What is still not built, deliberately: a mail conflict chooser. Mail flags are
    optimistic-local with revert-on-failure and the both-changed state this requirement
    describes largely cannot arise for them, so a chooser there would be a surface for a state
    that does not occur. That reasoning is kept here rather than dropped, because it is what
    stops somebody building one later.
    Still open after plan 03-09: the push retry described above. A change typed here is still
    sent over the address book's newer copy when a stale marker is refused, which is the one
    remaining both-changed state that resolves without asking. It is guarded and deliberate,
    and it is recorded in the broken windows ledger rather than fixed here.

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

- [x] **WRITE-02**: Insert an image inline in an HTML message.
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

- [x] **WRITE-03**: Spell check while typing, with jumps between misspellings.
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

- [x] **READ-03**: Hook into an external spam classifier.
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

  - [D] **Reworded 2026-09-06 by Pratik's decision 5, carried out by `05-06`.** The behaviour
    stays and the sentence is corrected. The move is written on this computer and marked as
    waiting whatever `Allow Changes` says; the gate is applied where each HTTP client is built,
    in `tasks_api.rs`, `google_api.rs`, `microsoft_graph.rs` and `caldav.rs`, so a refused push
    is counted rather than reported as a failure and the sync summary names the setting to turn
    on. What `05-06` adds is that the move says the same thing at the moment somebody makes it,
    in the same words, out of `allowed::turn_the_setting_on`, rather than one sync later.
    Not a refusal at move time, and deliberately: every other edit in the program is written
    here and held at the gate, and making a task move the one exception is worse for somebody
    moving by keyboard, who would meet a different answer on this command than on all the
    others. The criterion this replaces asked for a refusal, which nothing has ever done.

  - [D] A move that fails at the provider leaves the task in exactly one list, never in both
    and never in neither.
    **Owned by `05-07` and `05-08`, and not closed by `05-06`. Recorded 2026-09-06 from
    Pratik's decision 1.** The criterion is left exactly as written, because decision 1 says
    the provider move gets built and so this becomes true by being satisfied rather than by
    being reworded. Today it is satisfied vacuously: a provider-held task is refused before
    anything is written, so there is no failure between two writes for it to be about. `05-07`
    makes the half-finished state a stored, visible, recoverable object and `05-08` makes the
    provider calls, in that order. `05-06` measured the local half, in
    `tests/a_moved_task_is_in_one_list.rs`, and that is a floor rather than this criterion:
    `TaskEntry.task_list_id` is one nullable column, so two lists is not a state this storage
    can hold, and the whole of the risk lives in the delete-there-create-here that does not
    exist yet.

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

    **Answered 2026-09-06 by decisions 3 and 4, and carried out by 2026-09-09.** All five, and
    the "no container in the schema at all" reading above is the one thing here that was wrong.
    A reminder has an `account_id`, `NOT NULL`, in the same list of columns that sentence
    quotes, and an account is a container: it is what `get_reminders_for_account` selects on and
    what decides which reminders a person is looking at. What made it invisible is that nothing
    had ever asked a reminder which account it was in. The context-menu test the paragraph names
    stayed green through both widenings, because it holds the menus to `applies_to` in both
    directions and only reddens on a half-widening.

    One storage finding, recorded because nothing above says it. `save_reminder` is an upsert
    whose `ON CONFLICT(id) DO UPDATE SET` list names eight columns and not `account_id`, so it
    writes the account on insert and ignores it on update. A move built on it reports success
    and moves nothing.

  - [S] `docs/IMPLEMENTATION_STATUS.md:93` and `docs/ALPHA_TESTING.md:116`, both corrected
    2026-09-04. Until then both said moving and copying work for mail only, which is what the
    previous version of this line quoted and what `docs/changelog.md:2152` had already made
    false.

  - [D] All five modules support move and copy with the same two keyboard commands, because one
    key means one thing in every module here. **Decided 2026-09-06 by decisions 3 and 4, and
    true in the code from 2026-09-09.**

    The open question above was whether contacts and reminders join the other three, and the
    reasoning that made it hard was right about both and about the wrong container each time.
    A contact is in as many groups as somebody puts it in, so there is no one home to move it
    out of. The answer is that the program asks which one it is leaving, and that question is
    the move. A reminder is filed nowhere, because the module sorts reminders into buckets
    worked out from when each is due and a bucket is not a place. The answer is the container a
    reminder has always had and nothing had looked at: the account. So neither needed a new
    table or an invented concept, and neither goes through the path that names one container.

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
  - Evidence: **that paragraph stopped being true on 2026-09-10, when `05.1-03` created
    `src/application/notes_sync.rs` and a CalDAV account started sending its notes.** It read:
    re-checked 2026-09-04 and still accurate, `ls src/application/*sync*.rs` returns
    `caldav_sync`, `collection_sync`, `contacts_sync`, `mail_sync`, `pop_sync`, `sync_marker`
    and `tasks_sync`, and no `notes_sync.rs`. It is kept because the requirement is still open
    and the reason has changed: notes now sync for an account whose calendar came from a CalDAV
    server, and stay on this computer for every other account, which is most of them. Nothing
    has met a real server.
    **Three of the six criteria below already ship, and the requirement did not know it.** The
    content model was not invented twice: `long_text.rs` is the shared Markdown reader,
    `pulldown-cmark` is imported there at line 17, and the notes editor labels its box "Body, in
    Markdown" (`src/presentation/wx_notes_module.rs:83` to 94). The stored form is the Markdown
    source: `long_text.rs:13` to 15 states it as the module's rule and `save_note`
    (`src/data/message_cache/notes.rs:99`) stores the body verbatim. And a screen reader reads
    the rendered structure: `read_aloud.rs:351` does for a note exactly what `read_aloud.rs:332`
    does for a contact's notes, which is the precedent this requirement names. What is left of
    PIM-04 is the three criteria about sync, and those are PIM-07's work.
    **The loose end inside PIM-04 is closed, by `05.1-01` on 2026-09-10.** It used to read:
    the `notes.format` column is written as the literal `"plain"` at six sites and read by no
    production code, so it is a stored answer nothing asks. Two of those three numbers were
    wrong when they were written, and the census taken over the whole tree found two production
    write sites, `managers.rs`'s `store_new_item` and `outlook_data_file.rs`'s `a_note_from`,
    with the other nine inside `#[cfg(test)]` blocks.
    Of the three answers the requirement offered, the column records why it stays, and dropping
    it was never available because `CLAUDE.md`'s schema rule forbids dropping or renaming a
    column that shipped. The reason is in `NoteBody`'s doc comment in
    `src/data/message_cache/notes.rs`, where the next reader of the schema meets it: making the
    reader obey the column would stop headings and lists being read in every note that already
    exists, and `long_text.rs`'s header already settles the question the column pretends to ask.
    The literal is gone either way. `NoteEntry.format` is the `NoteBody` enum, so the two
    production writers cannot spell it differently, and a word this build did not write survives
    being read and written back rather than being replaced.
    **The byte-identical round trip now has a test that goes through storage** rather than
    through a formatter, in `notes.rs`, with a guard record whose break is a trim in `save_note`.

  - [S] `docs/ALPHA_TESTING.md`: notes stay on this computer. **Corrected 2026-09-10 in that
    page and here: they stay on this computer unless the account's calendar came from a CalDAV
    server, and the page now says which accounts sync and what a sync loses.**

  - **Decided 2026-08-29 by Pratik.** Not one target. A note has a backend chosen by the
    account it belongs to, the local note itself is a first-class Markdown document, and the
    seam is shaped so a hosted service can be added later without a migration. That is three
    pieces of work, split into PIM-04, PIM-07 and PIM-08 below.

  - [D] A note is a Markdown document. `pulldown-cmark` is already a dependency and
    `application/long_text.rs`, `sign_off.rs` and `presentation/editor_document.rs` already
    render Markdown, so the content model reuses what signatures use rather than inventing a
    second one.

  - [D] The stored form is the Markdown source, and a note saved here and read back from
    storage is byte-identical when nothing changed.

  - [D] Through a backend, the structure survives. Headings, lists including nested ones,
    tables, links with their addresses, and pictures with their descriptions come back as what
    they were. **Met on 2026-09-11 for both backends that exist, by ledger 270.** The sentence
    is untouched; only its status is. All five things it names come back as what they were,
    measured row by row in `docs/development/the-notes-seam.md` by a test that reads the table
    out of that document. Twelve of the twenty-two constructs measured there survive, up from
    seven, and none of the ten that do not is this program's own reader any more.

    One caveat the line does not cover and a reader should know: a picture comes back with its
    description and with **OneNote's address for it** rather than the one it went out with,
    because the service stores the picture and hands back a resource address of its own. The
    picture is not lost and it is not the same address. That is entry 272 in
    `.planning/WINDOWS.md`.

    What used to fail, and why the obvious fix would have been wrong, is kept below because the
    reasoning is what stopped a shipped accessibility defect being introduced to fix a storage
    one.

  - [D] Where a backend cannot carry a construct at all, it says what it could not keep, at the
    moment it takes the note, and the copy kept here becomes what came back. `WhatTheBackendKept`
    in `src/application/notes_backend.rs` is the mechanism, built by `05.1-04`, and the calendar
    backend already uses it for the carriage returns RFC 5545 cannot carry.

    **Those three lines replaced one, on 2026-09-11, by Pratik's decision at `05.2-01`'s
    checkpoint.** The line they replace read: the stored form is the Markdown source, what a note
    round-trips through **any backend** is that source, so a note edited here and read back is
    byte-identical when nothing changed. The word "any" was false for both backends that exist,
    in two different ways, and neither was known when it was written.

    A calendar server loses carriage returns and keeps every other byte: RFC 5545 section 3.3.11
    gives one escape for a line break and no way to write a carriage return inside a value at
    all, so a body typed on Windows comes back with plain line feeds. `05.1-03` measured it on
    2026-09-10 and `src/service/note_document.rs`'s header carries the measurement. OneNote keeps
    most of the words and loses the form, because nothing is stored there: what crosses is a
    structure, and the Markdown that comes back is rendered afresh from it. `05.2-01` measured
    twenty-two constructs on 2026-09-11 and seven survive. The table is in
    `docs/development/the-notes-seam.md`, read out of the document by a test so it cannot drift.

    **What fails the structure line today, and why the obvious fix is the wrong one.** Five of
    the losses are this program's rather than a service's: a nested list at any depth, a table,
    a link's address, a picture, and a line break inside a paragraph. All five happen in
    `long_text::from_markup`. It would be a mistake to change it. That function is written for
    **speaking**, not for storing, and its own comment says so: `spoken` returns a
    paragraph-only field exactly as written, so a link that kept its address would be read aloud
    as brackets, parentheses and every character of a URL. It contributes a picture's
    description alone and refuses to invent one the sender never wrote. Both are right for the
    job it has, and it has two other callers on that job, an event's description at
    `calendar.rs:3037` and a task's body at `tasks_api.rs:542`, both reading what Google and
    Microsoft hand back. Changing it to satisfy notes would make a Google task's description
    read a URL aloud character by character, in shipped code, in a product whose first audience
    cannot see the screen.

    So the structure line needed a **second reader, for the storing job**, and not an edit to
    the speaking one. **That reader was written on 2026-09-11 by ledger 270 and is
    `long_text::from_markup_to_edit`**, which `service::onenote_page::the_note_on` uses. It is
    the same tree walk with a different answer at three arms rather than a second walk, and
    `from_markup` is unchanged, which three tests hold it to rather than a sentence.

    **The paragraph above was half wrong and the correction is worth keeping.** It names five
    losses and says all five are correct decisions for a speaking reader. Two of them were not.
    A nested list read out as a flat run of bullets and a table read out as one unbroken word
    are losses for a **listener**, not only for storage, and they reached a screen reader
    through five call sites in `read_aloud.rs` and through a Google task's description. Those
    two were fixed in `from_markup` itself, which improved speech. Only the other three needed
    the second reader.

    The remaining losses are OneNote's own, cannot be closed here, and are what the third line
    above exists to report: a quote, bold, italic, struck-out text, inline code, a code block
    and a horizontal rule.

    `docs/ALPHA_TESTING.md` and `docs/changelog.md` already tell a user plainly what a calendar
    server loses. Nothing yet tells them what OneNote would, because no OneNote backend ships.

  - [D] A screen reader reads the rendered structure, not the raw source: headings announce as
    headings and lists as lists, the way a contact's notes already do.

  - [D] Note sync goes through `Allowed::personal_information`.
  - [D] Until a backend is live, the settings screen says notes do not sync yet, rather than
    offering a switch that does nothing. **Reworded in the product by `05.1-02` and left here
    with its old shape visible.** The screen says this account has no notes backend, not "not
    yet", because a consumer Gmail account still has none after all three backends ship. A
    backend is now live for CalDAV accounts, and the same screen names where those notes go.

- [ ] **PIM-07**: A notes backend chosen by account type, behind one seam.
  - Evidence: rewritten 2026-09-10 by `05.1-03`, which made most of the old wording false.
    **All four `[D]` lines below now hold structurally, and the requirement is not ticked**,
    because the last one is about a real server and no build has met one.
    `application::notes_backend` decides where an account's notes go and is the only place that
    chooses a backend. `application::notes_sync` is the sync and knows no backend: a grep of it
    for "caldav", "CalDav", "VJOURNAL" and "OneNote" returns nothing, which is checked rather
    than claimed. `service::caldav_journal` is the first implementation, behind
    `NotesService`, and `service::note_document` reads and writes the document it exchanges with
    no network in it.
    The schema work the old evidence called for is done, additively: `notes` carries `pending`,
    `provider_note_id` and `provider_version` through `ensure_column_exists`, and
    `deleted_notes` was created with `CREATE TABLE IF NOT EXISTS`, because
    `application::deletions`'s rule needs a record that outlives the row.
    The old evidence's two pointers were both wrong and are corrected here rather than left.
    `new_item.rs` has no `syncs` predicate; the function is `supports`, and `05.1-02` made it
    ask the seam rather than keep a second answer. And
    `test_notes_are_not_offered_a_sync_they_cannot_do` did not need inverting: `05.1-02` made
    the menu ask the seam, so that test asserts the arm for an account whose notes really do
    stay here and it is still correct.

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

- [x] **PIM-05**: CardDAV for contacts.
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
    **Corrected 2026-09-14 by 06-04.** The second setting of this shape is no longer one: the
    account edit dialog writes `allowed_per_account` through `AppConfig::set_allowed_for`,
    three boxes, one per answer, each able only to narrow, and the guard named above is
    retired with the control because the sentences it forbade are true now. The first,
    per-event, half is 06-02's and its listening pass is 06-06's; the box stays unticked for
    that reason and not for this one.
    **Corrected 2026-09-14 by 06-07, on Pratik's answer that this file is corrected in place.**
    The evidence above is wrong about the code as it stands, not merely out of date. It says
    "`per_event` (line 392) and `set_event_channels` (line 424) are both **private**, `fn` and
    not `pub fn`, so no screen could write one without changing a visibility." `per_event` is
    not a function and never was: it is a struct field, `per_event: Vec<(Event,
    BTreeSet<Channel>)>` at `src/presentation/accessibility/feedback.rs:497`, so "make it
    public" was not a coherent instruction for it. `set_event_channels` is `pub fn` at
    `feedback.rs:533` since 06-01 and is written by the settings screen at
    `src/presentation/wx_settings.rs:2257`, beside `use_the_default_for` at 2282; the
    per-event override enters this program from a screen, not only from a hand-edited
    string. The `[S]` line below saying the overrides exist "with no interface" and the
    fourth `[D]` line asking for a count of screens that reach `set_event_channels` are both
    older than 06-02 and are left as they were written, as the record of what was asked.
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

- [x] **FEEDBACK-02**: Dates and relative wording in the user's own language and format.
  - **Complete structurally on 2026-09-13 at `39417f88`, 06-03 tasks 3 and 4, and heard by
    nobody.** Every date this program writes follows the computer: month names in a date, the
    twelve for a list, the seven day names, order and clock, through
    `src/common/how_the_machine_writes_dates.rs`, with a source-reading guard holding that no
    shipped literal names a month or a day in English apart from six allowed by name. "2 days
    ago" comes out of `locales/en-US/dates.ftl` through Project Fluent with the four conditions
    below held by tests, and a Russian resource inside a test produces the four Russian forms on
    an en-US machine. The fallback is English, silently, forced in tests rather than read off the
    machine. What is still English is the wording around a date, "every week on" and the
    sentences of the interface, which is version 2's, and the four interface labels with example
    dates named in `WINDOWS.md` 372. No date, day name, sentence or number has been heard by a
    screen reader in any language: `WINDOWS.md` 360, 364, 366, 368, 370 and 371, for the pass
    after phase 8. The plan said nothing had been installed when the paragraph below was
    written; the install was `3c1291f0`, which answered `all`.
  - **The relative-wording decision, taken 2026-09-13 and written down here the same day by the
    rewrite of 06-03's task 3.** The evidence below is left as it was, because it is what this
    requirement was judged against. What follows is the decision and the three options that lost,
    so the comparison is not rebuilt the first time the recurring cost is questioned.

    **Project Fluent**, `fluent-bundle` 0.16 with `unic-langid` 0.9, and `fluent-langneg` 0.13
    and `intl-memoizer` 0.5 as direct dependencies from the same closure. Decided by Pratik on
    2026-09-13 at 06-03's checkpoint, which asked what "2 days ago" should do on a French
    computer and offered three options: keep it English, fall back to the date, or take on real
    plural rules. He chose the third and widened it: "3 + the start of real internationalization.
    We'll be starting translation support for the interface and screen reader speech in version
    2." The package was confirmed the same day, "Yes. do that.", after an audit across four
    parallel evaluations. The audit table is in `06-03-PLAN.md` under
    `<package_legitimacy_audit>`, with every figure's command beside it.

    **The three that lost, and what each lost on.**

    - **ICU4X.** `icu_plurals` is Unicode's own rules and excellent as a primitive, but ICU4X
      carries no message system: the reserved `icu_message` crate has been an empty stub since
      2021 and upstream issue #3028 has been open since January 2023. Its MSRV is exactly this
      project's 1.88 floor with no headroom, and it has raised that floor in minor releases. Its
      `icu_experimental::relativetime` produces "2 days ago" in every locale and is explicitly
      pre-release, breaking about every six months.
    - **`intl_pluralrules` alone.** One primitive where a message system is needed. Its data is
      CLDR 37, its repository has been untouched since November 2022, its published tarball
      ships no licence text, and it fails with a `&'static str`. It is in the chosen closure
      regardless, one layer down, where `fluent-bundle` wraps its error and negotiates around its
      lookup.
    - **A hand-written CLDR table.** A private copy of an open standard, which this project's
      guardrails refuse: it goes stale with no commit here and nobody pressured to fix it.

    **Which of those reasons could change, and which could not.** ICU4X's is a missing crate and
    an unstable one, and projects ship those eventually: read the comparison again if
    `icu_message` gains a release or `relativetime` reaches 1.0. The other two will not move.

    **The reason Fluent won, which is specific to this project.** A Fluent message carries
    attributes, so a control's visible label, its accessible name and its keyboard mnemonic are
    one translatable unit. That is what this project does with a label plus
    `set_accessible_name`, and it answers the problem nobody meets until the first translation:
    an `&` mnemonic has to be a letter in the translated label. No other candidate is designed
    for an interface that is heard.

    **What the decision commits the project to.** Eight new packages in `Cargo.lock`, none with a
    build script or a proc macro, licensed `Apache-2.0 OR MIT` throughout. A catalogue at
    `locales/en-US/dates.ftl`, four messages now, laid out for five thousand: one file per area,
    ids `<area>-<meaning>`, and the attribute names `.label`, `.accessible-name` and `.mnemonic`
    reserved for controls. A loader in `src/common/catalogue.rs` with the four conditions
    Firefox's production use of these crates requires: isolation marks off, numbers through
    `GetNumberFormatEx`, counts passed as numbers with the error vector checked, and a
    completeness check holding both directions. The tree measurement of 2026-09-13 found roughly
    5,300 user-facing string occurrences, adoptable more easily than the count suggests: the
    sentence is already the unit in 3,476 `format!` sites, 487 named constants and 38 `spoken()`
    functions are the seams, and the hostile patterns are about 4%. The commands are in the
    plan's audit block.

    **What the decision does not buy, which is the part to be clear about.** Nothing a user hears
    changes when 06-03's task 3 lands. Only an English catalogue exists, so on a French computer
    "2 days ago" is still "2 days ago", which is this requirement's own fallback clause. The
    plural machinery is proved with a Russian resource written inside a test, on an English
    machine, never on a Russian one. Which languages ship, and who writes them, is a version 2
    decision and Pratik's; no executor writes a translation in a language they do not speak.

    **Nothing has been installed.** No line naming any of the eight is in `Cargo.toml` as this is
    written. The install is 06-03's task 3, in a commit of its own that answers `all`.

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
    **Corrected 2026-09-14 by 06-07, on Pratik's answer that this file is corrected in place.**
    The paragraph above says "what is left is the strings and only the strings" and then names
    one file for them, `src/presentation/date_display.rs`, with `MONTHS` at line 100 formatted
    from at 384 and 456 and offered at `wx_item_form.rs:844`. That was one file and a line in a
    second where there were four files. 06-03 had to change `src/presentation/date_display.rs`
    and `src/presentation/wx_item_form.rs`, which the paragraph cites, and also
    `src/service/signed_mail.rs`, where eight signature sentences wrote the month with `%B`, and
    `src/application/occurrences.rs`, where the repeat-series sentence wrote the seven English
    day names, neither of which this paragraph mentions at all. The merges are `e98514b0` and
    `39417f88`, and the source-reading guard that now holds no shipped literal to an English
    month or day is how a fifth file would be found. The rest of the paragraph, the order and
    the clock already following the machine, was right.

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

- [x] **FEEDBACK-03**: Know how much of WCAG the automated scans actually cover.
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
    **Corrected 2026-09-14 by 06-07, on Pratik's answer that this file is corrected in place.**
    The evidence above rests on "line 136 records roughly half of WCAG covered", and that
    figure is what 06-07 disproved: it is a widely quoted estimate of the share of accessibility
    defects an automated tool finds, and every copy in this tree had made it the share of
    success criteria a scan can judge. The number, taken from the pinned scanner's own rule
    list on 2026-09-14, is three of fifty-five: the 155 rules of Axe.Windows v2.4.2 cite 1.3.1,
    2.1.1 and 4.1.2 and no other WCAG criterion, 61 + 9 + 9 rules, with the other 76 citing
    Section 508; and the MSAA walk judges the Name part of 4.1.2 alone. Fifty-five rather than
    56, because 4.1.1 carries no level in WCAG 2.2. "Nothing in the tree names which WCAG 2.2
    AA criteria the scan can and cannot judge" is no longer true: `docs/wcag-coverage.md` has a
    row for every criterion, `src/presentation/what_the_scans_can_judge.rs` holds the three as
    code with a reading that holds the page to them both ways, and the scan's own step summary
    names them. The five findings have still not been re-read, and there is still no list of
    interactions for a manual pass; those are 06-08's. The anchors `IMPLEMENTATION_STATUS.md:130`
    and `:136` had moved to 178 and 184 by the time this was written, and 184 no longer says
    half.

  - [S] Automated checks catch roughly half of accessibility defects and do not replace testing
    with real assistive technology. Structure present is not experience good. (Left as written
    on 2026-09-14: this sentence is about defects, and is the one copy of the figure that was
    never wrong.)

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
  - **The certificate decision, taken 2026-09-06 and written down here on 2026-09-12 by plan
    07-07.** The evidence below is left as it was, because it is what this requirement was
    judged against. What follows is the decision and the three options that lost, so the
    comparison is not rebuilt the first time the recurring cost is questioned.

    **Azure Artifact Signing**, formerly Trusted Signing. Decided by Pratik on 2026-09-06,
    recorded as decision 6 in `.planning/decisions-2026-09-06.md`. About $9.99 a month.
    The publisher name a user will see is **Pratik Patel**. Signing runs from CI with no
    hardware token, because the key stays in a service the build calls rather than on a stick
    somebody has to plug in. Individuals may use it only from the United States or Canada, and
    **that requirement is met**, asked and confirmed on 2026-09-06.

    **The three that lost, and what each lost on.**

    - **An OV certificate from a certificate authority**, $150 to $300 a year. Since June 2023
      the CA/Browser Forum requires the private key on an HSM or a hardware token, so signing
      from GitHub Actions needs a cloud HSM as well. It is the most build machinery of the
      four and it buys the same SmartScreen outcome as the option chosen.
    - **SignPath Foundation**, free for open source. The certificate is issued to SignPath
      Foundation, and SignPath's own terms say that makes SignPath Foundation the publisher of
      the project. So that is the name a Wixen Mail user would see on the warning box and in
      Apps and Features, and it disagrees with `AppPublisher=Pratik Patel` in the installer
      script and with `CompanyName` in `build.rs`.
    - **Not signing this milestone.** A real option rather than a straw one. It lost because
      Smart App Control on Windows 11 blocks unsigned executables that have no reputation, and
      because reputation never starts accruing while nothing is signed, so a year of not
      signing leaves the project exactly where it began.

    **Which of those reasons could change, and which could not.** The OV one is a price, and
    prices move. The SignPath one is a judgement about whose name goes where users look, and
    it will not move. Read the comparison again if the price of an OV certificate and a cloud
    HSM falls below $120 a year; do not read it again because SignPath is still free.

    **What the decision commits the project to.** Not "sign two files". Four distinct binaries
    need signing: `wixen-mail.exe`, `wixen_mail_search.dll` and `wixen-mail-search-setup.exe`
    inside the installer, and the setup executable itself. Plus the uninstaller Inno generates,
    whose `SignedUninstaller` directive is recorded as having a two-pass prompting behaviour
    that would hang a CI job rather than fail it, and **that behaviour is unverified**: it was
    read through a search summary of the Inno help rather than from the local Inno Setup 6
    help, and it stays an assumption until somebody reads that. The portable copy and the zip
    published beside the installer are not two more signing jobs: both are made from
    `target/release/wixen-mail.exe` after the build runs, so the portable copy inherits the
    signature already in those bytes, and a zip is a container rather than a signable format,
    because Authenticode embeds a signature in a PE file and not in an archive.

    **What signing does not buy, which is the part everybody gets wrong.** No certificate
    available to this project removes the SmartScreen warning on a first download. Only
    publishing through the Microsoft Store does. Microsoft's page was re-fetched on 2026-09-12
    rather than quoted from the research, at
    https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation,
    and it still says "EV certificates no longer bypass SmartScreen", with its table giving
    OV and EV the same first-download outcome. The page's `updated_at` is 2026-08-17, the same
    date the research recorded, so it has not moved since. And `PrivilegesRequired=lowest` at
    `installer/Wixen-Mail-Setup.iss:39` means a per-user install shows no elevation prompt at
    all, so the "Unknown publisher" line a signature really does fix is not shown in the common
    case. The warning everybody meets is the SmartScreen one, and signing does not remove it.

    **Nothing has been signed.** No certificate exists yet, no Azure account exists yet, and
    every build this project has produced is unsigned. The signing itself is plan 07-08, and it
    waits on an account only Pratik can create.

  - Evidence: re-checked 2026-09-04 and accurate about the tree, every clause.
    `installer/Wixen-Mail-Setup.iss` builds an Inno Setup installer;
    `scripts/build-installer.sh:29` to 48 appends the commit it built from, and appends nothing
    at a tag; `docs/ALPHA_TESTING.md` states the installer is not signed, at the line beginning
    "**The installer is not signed**, so Windows will warn about it". **That citation said
    `:146` until 2026-09-12, when `grep -n 'The installer is not signed' docs/ALPHA_TESTING.md`
    answered 342.** The sentence has carried three different line numbers in a week as the file
    grew from about 160 lines to 365, so it is cited by its words from here on, which is what
    the requirements audit of 2026-09-04 asked for. The evidence was
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

  - [D] The certificate is chosen and the account is not created. **Corrected 2026-09-12.**
    This line used to read "This is blocked on a certificate decision that is Pratik's. Until
    it is made, the requirement stays open and the docs keep saying the installer is unsigned."
    The decision was made on 2026-09-06 and is written above, so the line said the question was
    open when it was answered. What the requirement now waits on is an Azure Artifact Signing
    account, which needs a subscription, an identity check naming a real person, a payment and
    a role grant in a tenant, none of which is a repository operation. Until something is really
    signed the requirement stays open and the docs keep saying the installer is unsigned.

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
  - [D] The application can tell the user a newer version exists, and nothing is fetched
    that the user did not ask for: either by choosing the update item in the Help menu,
    which works whatever the setting says, or by choosing a release channel in the update
    setting, which starts off. With a channel chosen the fetch is unattended and the
    consent for it was given at the control, which says so. It is never a fetch nobody
    chose, because publishing and fetching both happen on purpose here.

    **Replaced 2026-09-12 by plan 07-07, per decision 16 of 2026-09-06.** The line used to end
    "and the check is a deliberate action or an explicit setting, never a silent background
    fetch, because publishing and fetching both happen on purpose here." The intent survives
    and the words did not: D-16 asks for exactly an unattended fetch once somebody has chosen a
    channel, so the old wording forbade what the phase is now being built to do. What makes the
    fetch a chosen one is the choice made at the control, not a question asked each time.

  - [D] Applying an update is the user's decision and the current version keeps working if the
    update is declined.

  - [D] The check compares the plain `0.x.y` version and ignores `+build` metadata, matching
    `src/common/version.rs` and SemVer ordering.

  - [D] Which releases the user hears about is theirs to choose: one setting with three
    values, not looking, public releases and development releases, reachable from the
    settings screen in a section somebody would look in. A prerelease is never offered on
    the public channel.

    **Added 2026-09-12 by plan 07-07, per decision 15 of 2026-09-06.** A line under SHIP-02
    rather than a requirement of its own, because choosing a channel is part of what updating
    means for this product and not a separate capability. It closes a gap the first version of
    these plans raised: a feature that was planned, tested and named in no requirement, which a
    later audit finds as scope nothing asked for.

- [x] **SHIP-03**: The installed shortcuts carry the application icon. Narrowed 2026-08-29:
  the shortcuts themselves are already built.

  - **Closed 2026-09-12 by plan 07-02. Two sentences in the evidence below are now
    false and one was false when it was written. They are left where they are,
    because they are what this requirement was judged against, and corrected here.**

    The `[D]` line is met and then some. Both `[Icons]` entries set `IconFilename`,
    and so does `UninstallDisplayIcon`, all three naming `{app}\icon.ico`, which a
    new `[Files]` line installs from `..\assets\icon.ico`. The `[D]` line's "the
    bundled icon" could be read as pointing at `assets/icon.ico` where it sits in
    the repository, and that is not a path on anybody's machine: a shortcut naming
    a path nothing installs makes Windows fall back to the default in silence, so
    the `[Files]` line is part of the change rather than an extra.
    `tests/installer.rs` compares the installed destination and the named icon path
    as one value for that reason.

    The gate hole is closed rather than accepted. `scripts/which-checks.sh` answers
    `all` for a change to any `.iss`, placed below the version-bump exception so an
    installer change that also bumps the version still earns it.

    **"Neither `house_style` nor `wired` reads it" was wrong when written.**
    `ours()` in `tests/house_style.rs` collects `installer/*.iss`, at line 54 today,
    so a prose rule over the installer script has been checked on every commit all
    along. The hole was real for a different reason one layer down, which is that
    `check.sh` maps a changed file to a test target by its path and an `.iss`
    matched no arm, so no scoped target was chosen. "`guards/guards.toml` names the
    installer nowhere" was true until this plan and is not now.

    The pictures on the shortcuts do not change and nobody will see a difference.
    What changed is that they no longer depend on the executable's resource table
    being right, which has failed here once. Nothing has installed anything or
    looked at a shortcut: `WINDOWS.md` 306.

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

- [x] **SHIP-04**: Encrypt the local cache, or decide not to and say so once and clearly.
  - **Closed 2026-09-12 by plan 07-01, merged at `228e6a3`. Two sentences in the evidence
    below are now false and they are left where they are, with this correction above them,
    because they are what the requirement was closed against.** "It does not ship in the
    running program" and the `grep -i encrypt src/presentation/first_run.rs` returning nothing
    both stopped being true at `9658282`. `INTRODUCTION` carries a third paragraph and the end
    of `--help` carries a fifth, both saying the mail is not encrypted on this computer, both
    naming Windows keeping another user out and somebody taking the drive out, and both naming
    full-disk encryption as the answer to the second. That is the second `[D]` line word for
    word. The "one accuracy gap" below is closed too, and the check that closed it found two
    gaps rather than one: `security.key` was on neither page and `oauth.toml` was on one.
    **What is not closed is whether either sentence is heard as an important fact**, which is
    `.planning/WINDOWS.md` 302 and 303 and needs a screen reader.
    **The caution below about where it goes was answered rather than followed.** It asks the
    plan to raise the choice between `INTRODUCTION` and a second `READ_MORE` button rather than
    settle it by appending, and 07-01 raised it with the measurement the caution lacked: the
    screen is shown once per install, not on every start, because `wx_app` returns early when
    `told_about_the_alpha` is set. So "every first run longer for every user" is one hearing on
    a machine's first start. It went in `INTRODUCTION`, with a length bound of 900 characters
    holding it to 694, and `first_run.rs`'s own doc comment now carries the reasoning.
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

- [x] **SHIP-06**: Off Windows, the application says what its accessibility layer does not do.
  - **Corrected 2026-09-12 by 07-03, which closed this. Everything below describes the tree
    before that plan and four of its sentences are now false.** `NativeBridgeStatus`,
    `ScreenReaderBridge::status` and `Accessibility::native_bridge_status` are gone, so "the
    accessor is built and nothing calls it" is no longer true of anything and
    `grep -rn native_bridge_status src/ tests/` returns nothing. The derivation is no longer
    `cfg!(target_os = "windows")`: `screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER` comes
    from whichever `native` module compiled, which is what the second `[D]` line below asked
    for, so the caveat naming that expression as a platform list of one is answered rather than
    outstanding. And the bridge has a second half that this evidence never mentions:
    `names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE` now records whether an accessible name
    reaches the accessibility tree, which had no marker anywhere before.
    The false sentences are left where they are, because they are what the requirement was
    measured against. What closes it is
    `src/presentation/accessibility/platform_bridge.rs`, read by `build_about_dialog` in
    `src/presentation/wx_app.rs` and by `main` in `src/main.rs`.

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

- [x] **PERF-01**: Memory under 150 MB with 1,000 cached messages, measured.
  - Evidence: re-checked 2026-09-04 and still accurate. No `benches/` directory, no `criterion`,
    no `divan` and no `[[bench]]` in `Cargo.toml`, and nothing in `src/` reads resident memory,
    so no target below has a number attached. The source of the target is
    `docs/development/requirements-backlog.md:81`, "Memory profiling | Target <150MB with 1000
    cached messages | Medium".
    Re-read 2026-09-14 at `7da68e78`: the sentence above was true on 2026-09-04 and is not
    now. `tests/the_numbers_the_targets_ask_for.rs` starts the release binary against a
    profile of 1,000 cached messages and reads the working set of the process and its WebView2
    tree, and the numbers are rows on `docs/development/measurements.md` dated 2026-09-14 with
    the machine and the build. Whether each `[D]` line closes is read clause by clause when
    the phase closes, not here.
    **Closed 2026-09-16 by 08-09, clause by clause.** The first `[D]` line was closed by 08-03:
    the row "Memory with 1,000 cached messages" on `docs/development/measurements.md`, taken
    2026-09-14 at `9d5f15c5` on a named machine from a release build, the median of five runs
    of a harness that is in the tree and can be run again. The second `[D]` line closes on a
    reading, and the reading is written down rather than assumed. The row holds two numbers:
    the application process peaked at 57 MB, under the target by 93 MB; the six
    `msedgewebview2.exe` processes Windows starts for the preview pane weighed 333 MB beside
    it, the same as on an empty profile, so the sum is 390 MB, over the target by 240 MB. The
    target was written before the preview was a browser and does not say whether it counts the
    renderer Microsoft ships beside the application. The coordinator's reading, put to Pratik
    on 2026-09-14 and not contradicted: the target is about the application process, so it is
    met, and the tree's weight is written on the same row and in the same sentence wherever
    the target is judged (`docs/roadmap.md`, `docs/development/requirements-backlog.md`, the
    changelog) so nobody reads 57 MB as the whole cost. The other reading, the sum, would miss
    this target by 240 MB and PERF-04's by 291 MB. This box is ticked on the first reading,
    pending Pratik's word, and one line here reverses it. Ledger 482.

  - [S] `docs/development/requirements-backlog.md`, performance and scale, Medium.
  - [D] A repeatable measurement produces a number for resident memory with 1,000 cached
    messages, recorded with the date, the machine and the build it came from.

  - [D] The number is either under 150 MB or the target is revised with the reason, rather
    than the target quietly remaining aspirational. Added 2026-09-16: "the number" means the
    application process's own working set, and the WebView2 tree's is reported beside it and
    not counted against the target, on the coordinator's reading pending Pratik's word; until
    then a number that leaves out the tree says so where it is quoted.

- [x] **PERF-02**: Cold start under 2 seconds, measured.
  - Evidence: re-checked 2026-09-04 and still accurate. `docs/roadmap.md:221` still reads
    `- [ ] Startup time optimization (<2 seconds)`, unticked, and line 253 repeats it as a
    success metric; `docs/development/requirements-backlog.md:82` carries it as Medium. Nothing
    in `src/` times process start against a usable list. The "under 2 seconds" is a target
    rather than a measurement and is correctly written as one.
    Re-read 2026-09-14 at `7da68e78`: "nothing in `src/` times process start" was true on
    2026-09-04 and is not now. `src/common/started.rs` takes the start instant as the first
    statement of `main` and the program writes `the message list is usable: N rows, M ms
    after start` once per process; the cold-start row on `docs/development/measurements.md`
    is dated 2026-09-14 with the machine and the build. The roadmap lines cited above were not
    re-checked here; whether the target is met is read when the phase closes.
    **Closed 2026-09-16 by 08-09, clause by clause.** Both `[D]` lines were closed by 08-03.
    The first: the start instant is the first statement of `main` and the end is the usable
    line, written once per process when the message list first holds a row, so the number is
    to a usable list and not to a window; a reading in `tests/the_numbers_the_targets_ask_for.rs`
    holds the line's shape and a record couples the fill to it. The second: the row "Cold
    start to a usable list, 1,000 cached messages" on `docs/development/measurements.md`,
    476 ms, the median of five, on 2026-09-14 at `9d5f15c5`, the machine and the build named,
    the first start after the build 520 ms on the same side of the target, from a harness in
    the tree. The target is met on that machine on that day, and the two roadmap lines and the
    backlog row now say so with the row named; 08-03 also found and fixed the reason no start
    had ever reached a usable list, the module the window opens on not being filled at
    startup, which the changelog dates to 2026-07-26. What is not measured: a start after a
    reboot (ledger 448), and a start with a live account (ledger 447's shape).

  - [S] Roadmap Phase 8; `docs/development/requirements-backlog.md`.
  - [D] Cold start is measured from process start to the message list being usable, not to the
    window appearing, because an empty window is not a usable inbox.

  - [D] The measurement is repeatable and recorded with the date, the machine and the build.

- [x] **PERF-03**: A real mailbox of 100,000 messages or more, exercised.
  - Evidence: rewritten 2026-09-04. "The largest thing exercised is a loopback server" is
    false. A 200,000 row sample mailbox generator ships in the product, on the Help menu, put
    there deliberately so that a screen reader user can arrow through one.
    `SAMPLE_MAILBOX_SIZE` is 200,000 and `sample_mailbox` builds the rows, both in
    `src/presentation/sample_mailbox.rs` since 08-04 moved them out of `wx_app.rs` on
    2026-09-14 (this line cited them by line number in `wx_app.rs`, 9125 and 9137, on
    2026-09-04, and a line number without a commit is not a citation, so the names replace
    the numbers);
    `ID_LOAD_SCALE_SAMPLE` in `wx_app.rs` is the menu id, its item is on the Help menu and its
    handler calls `sample_mailbox(SAMPLE_MAILBOX_SIZE)`, `grep -n ID_LOAD_SCALE_SAMPLE
    src/presentation/wx_app.rs` finds all three, and it is on the Help menu rather than behind
    a build flag, because as its doc comment says, the people who most need to test it are
    not the people compiling it.
    **So the first `[D]` line was half satisfied on 2026-09-04: the mechanism existed and was
    reachable, and the numbers did not.** Nothing in the tree then recorded a sort, filter or
    scroll timing from a sample run. Re-read 2026-09-14 at `7da68e78`:
    `tests/the_list_at_two_hundred_thousand_rows.rs` times the listing, the filter, each sort
    order and the paint of one page over the same 200,000 rows with no window, and eighteen
    rows on `docs/development/measurements.md` hold the numbers dated 2026-09-14 at
    `5cf04528` with the machine and the build.
    **The second `[D]` line was not satisfied on 2026-09-04 and is the part to keep.** The
    virtual text callback in `wx_app.rs` (cited at `:1101` that day, and the comment at
    `:1093`) read the whole loaded list out of `state.messages` in memory and never touched
    SQLite, but no test asserted it. Re-read 2026-09-14 at `7da68e78`: the callback's body is
    `virtual_rows::text_for` in `src/presentation/virtual_rows.rs`, a function over slices and
    copies, and `tests/the_list_reads_only_memory.rs` holds it and the closure that calls it
    to naming no database, with two guard records coupling both to the reading. Nor is the
    mail-at-scale plan's paged design built: there is no page cache of 200 rows around the
    viewport and no placeholder on a cache miss, because the whole list is held in memory.
    `message_rows::PLACEHOLDER` exists and is returned only when the row index is past the end
    of the loaded list.
    The third `[D]` line, that no criterion here claims a real provider mailbox was used, is
    still correct and still important, and still true on 2026-09-14: every row timed was
    synthetic.
    **Closed 2026-09-16 by 08-09, clause by clause, with the title read as its `[D]` lines
    read it.** The title says "a real mailbox" and the third `[D]` line says synthetic rows
    answer the list question and the provider question waits for a live account; the clauses
    are what is ticked, and the provider question is ledger 480 and open. The first `[D]`
    line was closed by 08-04: 200,000 rows from `sample_mailbox`, the Help menu's generator,
    written into a cache and timed with no window by `tests/the_list_at_two_hundred_thousand_rows.rs`,
    eighteen rows on `docs/development/measurements.md` dated 2026-09-14 at `5cf04528`.
    Sort: `sort_messages` over every order, 61 ms to 260 ms. Filter: `search_messages` at the
    box's limit, 78 ms to 110 ms, and every match of a one-in-five word, 193 ms. Scroll: the
    number is the page paint, `virtual_rows::text_for` over one page of every inbox column,
    0.09 ms, which is the whole of what the paint callback does after taking the lock;
    wxWidgets' own painting of those cells and the list control taking a sort's 200,000 rows
    back were not timed, because the harness has no window, ledgers 449 and 450, and a tick
    without that sentence would claim a measurement nobody took. The second `[S]` line's
    freeze is answered by the numbers: the slowest sort is a quarter of a second and runs off
    the interface thread. The second `[D]` line was closed by 08-04 in two halves: by type,
    `text_for` takes slices and copies and cannot reach the connection the program holds;
    by reading, `tests/the_list_reads_only_memory.rs` holds the module and the closure to
    naming no database, with two guard records. The third `[D]` line holds: no row and no
    sentence in this phase claims a provider mailbox.

  - [S] Roadmap Phase 8; `docs/plans/20260726-mail-at-scale.md`.
  - [S] Sorting 200,000 rows in memory on a header click is a multi-second freeze, and a freeze
    is an accessibility failure, not a performance one.

  - [D] The list is exercised against 200,000 synthetic rows, which needs no network and no
    live account, and the sort, filter and scroll paths each produce a recorded number.

  - [D] A test asserts the virtual text callback issues no SQLite query.
  - [D] No criterion here claims a real provider mailbox was used. Synthetic rows answer the
    list question; the provider question waits for a live account.

- [x] **PERF-04**: Idle memory under 100 MB, measured.
  - Evidence: re-checked 2026-09-04 and still accurate. `docs/roadmap.md:254` reads
    `- Low memory footprint (< 100MB idle)` under success metrics, with no measurement anywhere.
    Re-read 2026-09-14 at `7da68e78`: "no measurement anywhere" was true on 2026-09-04 and is
    not now. The idle row on `docs/development/measurements.md`, memory at 120 s with 1,000
    cached messages and no input after the usable line, is dated 2026-09-14 with the machine
    and the build, beside the empty-profile floor taken the same way.
    **Closed 2026-09-16 by 08-09, clause by clause, on the same reading as PERF-01.** The
    first `[D]` line was closed by 08-03: the row "Idle memory at 120 s, 1,000 cached
    messages" on `docs/development/measurements.md`, no input after the usable line, the
    window left where the start put it, read at 60 s and 120 s, taken 2026-09-14 at
    `9d5f15c5` with the machine and the build, the median of five. The second `[D]` line: the
    application process sat at 56 MB, under the target by 44 MB and about 3 MB over the
    empty-profile floor; the WebView2 tree 334 MB beside it, so the sum is 391 MB, over by
    291 MB, and idle growth over the minute was about 1 MB, all of it in the tree. Read as
    the application process, PERF-01's reading, met; the sum is written beside the target in
    `docs/roadmap.md` and here. Pending Pratik's word, ledger 482. Idle with a live account is
    a different idle and was not taken, ledger 447.

  - [S] Roadmap success metrics.
  - [D] Idle memory is measured after startup with a cache present and no user activity, and
    recorded with the date, machine and build.

  - [D] The number is either under 100 MB or the target is revised with the reason. Added
    2026-09-16: "the number" means the application process's own working set, as PERF-01's
    second `[D]` line says, with the WebView2 tree reported beside it.

- [x] **PERF-05**: Line coverage re-measured.
  - Evidence: corrected 2026-09-04. The coverage figure and its date are still right and the
    commit count attached to them was out by a factor of about four.
    60.4%, measured 2026-07-26 with `cargo llvm-cov --lib --summary-only`, stale since, and
    still recorded that way at `docs/IMPLEMENTATION_STATUS.md:201`.
    `git rev-list --count --since="2026-07-26" HEAD` on 2026-09-04 gives **1,195** commits
    landed since, out of 1,373 in the repository, first commit 2026-02-13. So 87% of this
    project's history postdates the reading. The "roughly 275" this line used to give was itself
    a stale count when it was written and had never been re-taken, which is the same defect this
    requirement is about, in the requirement about it.
    Re-taken 2026-09-14 at `7da68e78`. The coverage was re-measured by 08-05 with the same
    command: 83.34% on 2026-09-14 at `55464a5e`, 160,966 of 193,153 lines, the row on
    `docs/development/measurements.md`, with 60.4% kept on the status page as the figure of
    2026-07-26; the status page citation above, `:201`, was the paragraph's line that day and
    the paragraph is now three, found by `grep -n 83.34 docs/IMPLEMENTATION_STATUS.md`.
    `git rev-list --count --since="2026-07-26" HEAD` gives 1,830 commits since the reading, out
    of 2,077 by `git rev-list --count HEAD`, 88%, against 1,195 of 1,373 on 2026-09-04.
    **Closed 2026-09-16 by 08-09, clause by clause.** The `[D]` line has two clauses. The
    first, a current number with its date replacing the stale one, was closed by 08-05:
    83.34% on 2026-09-14 at `55464a5e` by the same command, the row on
    `docs/development/measurements.md`, on the status page with 60.4% kept as the figure of
    2026-07-26. The second, the low areas attributed rather than treated as a number to raise,
    closes with the attribution corrected, because the one the requirement wrote turned out
    not to describe the tree: the three transport areas read 92.06%, 84.55% and 96.75%, all
    above the library, so the transport is not where the missed lines are. Where they are is
    the 27 wxWidgets window files at 26.88%, holding 73% of the missed lines, and the
    attribution 08-09 writes for them is this: those files build windows, `cargo llvm-cov
    --lib` runs the library's own tests and opens none, and the targets under `tests/` that
    do open windows are outside that command. A run that included them would be a different
    quantity from the 2026-07-26 one, and the same quantity is what makes the 23-point rise
    real; so the figure is accepted with that reason written beside it on the page, the
    status page and `docs/architecture.md`, and is not a number to raise by writing tests
    toward it. The 80% target in `docs/architecture.md` is met by the library and kept as
    written. Ledgers 452 and 453 close with this paragraph.

  - [S] `docs/IMPLEMENTATION_STATUS.md`.
  - [S] Coverage is the cheap wide sweep answering only "what never runs at all", and low
    coverage in `service/protocols`, `service/oauth` and the provider clients is the network
    transport that has never met a live account, tracked as work rather than as a testing gap.
    **That was true of the transport on 2026-07-26 and is not the reason on 2026-09-14.** By
    the run at `55464a5e`, summed from its per-file table with `cargo llvm-cov report --json
    --summary-only`, `src/service/protocols/` is 92.06%, the two OAuth files 84.55% and the six
    provider clients 96.75%, every one above the library's 83.34%, and everything outside
    `src/presentation/` is 95.53%. The low area is the 27 wxWidgets window files,
    `src/presentation/wx_*.rs`, at 26.88%, holding 73% of the missed lines, `wx_app.rs` at
    29.83% and eleven of the 27 at 0%. Those files build windows and a `--lib` run opens none,
    which is a description of where the missed lines are and not an attribution; ledger 452
    carries this sentence and ledger 453 carries the windows, and what the windows are, a gap
    to close, a different command to measure with, or a figure to accept with the reason, is
    decided when the phase closes. The transport's own reason still stands for the transport:
    none of those files has met anything but a loopback server a test started.

  - [D] A current number replaces the stale one, with its date, and the low areas are
    attributed rather than treated as a number to raise.

- [x] **PERF-06**: Every document that quotes a test count quotes the same measurement.
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
    Re-taken 2026-09-14 at `7da68e78`, every figure and citation above being that of
    2026-09-04:

    ```
    cargo test --lib -- --list | tail -1                                     7,264      2026-09-14
    cargo test --all-targets -- --list 2>&1 | grep -E '^[0-9]+ tests?,' | awk '{s+=$1} END{print s}'
                                                                             7,750      2026-09-14
    ```

    so the targets under `tests/` hold 486. The three pages quote one row: the status page
    and `docs/integration-guide.md` say 7,697 tests, 7,245 unit and 452 integration, counted
    2026-09-14 at `a42331bb` with the command above, the row on
    `docs/development/measurements.md`, and `test_the_three_pages_that_state_the_test_count_quote_one_row`
    in `tests/every_number_carries_its_command_and_its_date.rs` holds them to it on every
    commit; the 7,750 here is a later take on the same day at a later commit, and the page
    refuses a second row with one `what` and one date, so it stays here as this file's own
    measurement and no page quotes it until a later day re-takes it. The line citations above
    have moved: the status page's count is found by
    `grep -n "7,697 tests" docs/IMPLEMENTATION_STATUS.md`, the changelog's 5,430 lines by
    `grep -n "5,430" docs/changelog.md` under the entry of their day, and the integration
    guide's by the same grep on it. The two durations: "about two days" is gone from both
    pages since 2026-09-14, replaced by the mutant count row, 12,335 by `cargo mutants --list`,
    and a rate 08-08 measures before any run; "about 15 hours" for the sweep is gone from
    `CLAUDE.md` since 2026-09-14, which now points at the product row, 784 x 92 s taken by
    08-01, and names the four figures the tree used to give. `CLAUDE.md`'s guard-record count
    is by the parser, not the grep, since the same day.
    **Closed 2026-09-16 by 08-09, clause by clause; this requirement is the whole phase.**
    The first `[D]` line was closed by 08-01, 08-02 and 08-06 together: the page exists and a
    reading refuses a row without its command, date and commit (08-01); a count, a coverage
    figure or a duration on any page under `docs/` other than the changelog and the dated
    plans, in `README.md` or in `CLAUDE.md`, sits in a paragraph with a date and a backticked
    source or names a target, read on every commit by
    `test_every_figure_on_a_page_carries_its_date_and_its_source` (08-02); and the thirteen
    paragraphs that reading named, the four sweep-cost figures and a fifth, and the three
    planning documents were dated and pointed by hand rather than re-numbered (08-02, 08-06).
    The second `[D]` line holds: the status page and the integration guide say 7,697 tests
    as 7,245 unit and 452 integration, two rows, and
    `test_the_three_pages_that_state_the_test_count_quote_one_row` holds the three pages to
    the page and not to each other (08-02). The third `[D]` line was closed by 08-01 and
    08-06: every duration row on the page carries its thread setting, whether the build was
    warm and what else was running, and the gate, suite, sweep and mutation-run durations on
    `CLAUDE.md` and the status page are dated and point at the rows. The fourth `[D]` line
    holds by construction: the reading compares shape and never value, and its four
    companions prove it can see an omission without ever comparing a number with today's.
    Every later plan of the phase wrote its figures as rows and nowhere else, which is the
    requirement doing its work.

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
    Re-taken 2026-09-14 at `7da68e78`: **798** records by the format's parser,
    `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`,
    which is the method `CLAUDE.md` prescribes since 2026-09-14 and the one the rows on
    `docs/development/measurements.md` are taken with; the grep above answers the same 798
    today, since every record opens with `[[guard]]`, and the parser is prescribed because a
    line reader miscounts the other question, records naming a file, when a record is spelled
    on one line. The 192 figure now has a counterpart: the census at the top of the file says
    192 were swept on 2026-08-12 and 606 have arrived since, and
    `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it` holds the
    two to the record count on every commit. `CLAUDE.md:474` and `CLAUDE.md:544` were that
    day's lines; the record-count paragraph is found by `grep -n "Count records" CLAUDE.md` and
    the 2026-08-05 run by `grep -n 2026-08-05 CLAUDE.md`. The mutant count the run would face
    is a row on the same page, 12,335 over 247 files by `cargo mutants --list` on 2026-09-14.
    The two `[S]` claims below about the script are unchanged and still accurate.
    **Read 2026-09-16 by 08-09, clause by clause, and left open as revised.** The first
    `[D]` line asks for one whole-tree run, and that did not happen: 08-08 measured the rate
    on one shard of 25 mutants under both suite shapes on 2026-09-15, put the products to
    Pratik, 17.5 days for the library and 19.8 days for every target on this machine, and he
    answered the same day: not the whole tree this milestone, the guard sweep first, then one
    scoped run, moved to GitHub's runners. What ran instead, at `3e633252` on 2026-09-15, is
    two areas in shards on one commit, `src/service/protocols/**` (450 mutants, 18 shards)
    and `src/service/caldav.rs` (420 mutants, 17 shards), each read after its last shard's
    process exited by `scripts/mutants_report.py --shards`, which refuses a missing, partial
    or differently committed shard, so the "not from a partial `mutants.out`" half of the
    clause holds for what ran. The rest of the tree, 12,391 mutants at `2847391c` less those
    870, has not been through a run since the sweeps of July and August; the rows on
    `docs/development/measurements.md` say what it would cost on this machine and, from the
    two runs, about 2.9 times that a mutant on a runner. Roadmap criterion 4 was revised in
    place by 08-08 with that as the reason, and this clause is revised the same way: not
    closed, and its cost written down instead of an estimate. The second `[D]` line holds
    for what ran: no mutant never started in either run, and the four that timed out were
    re-run by name on this machine and timed out again for reasons the run page gives. The
    third `[D]` line was closed by 08-08 for what ran: every one of the 56 survivors is on
    `docs/plans/20260915-whole-tree-mutation-run.md`, 43 killed by tests shown red against
    their mutants, 6 equivalent with the reason, 7 untested behaviour queued as ledgers 472
    to 478 with the six equivalents held together as 479, so the surviving list is the input
    to the next round. The box stays unticked because the first clause is not what happened;
    the justification for the requirement no longer rests on the share of tests written after
    their code, which 08-02 made a computed figure, but on what the runs found, and the
    status page says so since 2026-09-16.

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

### What the first day of testing found

Added 2026-09-16 for phase 9. Every requirement here is one GitHub issue, or one shared cause
behind two, from the 44 that Pratik's first day of testing produced on 2026-09-15 against build
`0.125.1+g3e633252`. The `[S]` lines quote the tester's own words from the issue, or a sentence
read from the code on the day the issue was filed. The `[D]` lines were written on 2026-09-16
by the planner and are proposals in the sense the top of this file gives. Every evidence line
was re-taken against `main` at `524ff24f` on 2026-09-16, because the tree moved after most of
the issues were written (08-07, 08-08 and 08-09 merged afterwards), and where an issue's premise
had moved the evidence line says which way.

**Every other issue in Pratik's order of 2026-09-16 is a requirement of some later phase, not of
this one.** The order has seven groups and phase 9 is the first two: the version, and the
cause-known defects an hour to a day each. The README in `.planning/phases/09-what-the-first-day-of-testing-found/`
names the remaining five groups and the issues in each, so the next planner adds their
requirements to a later phase's section rather than here. Added 2026-09-17: group 3 is phase
10, `MAIL-01` to `MAIL-05`, in the section after this one; groups 4 to 7 are still later phases.

- [x] **FOUND-01**: The next build is `1.0.0-alpha.1`, and the versioning rule says how a
  version moves inside a prerelease.
  - **Closed 2026-09-16 by 09-01, merged at `c0606807`, on the tree side.** The four `[D]`
    lines below each have a name: `test_the_version_the_tree_carries_is_at_least_one_point_oh`
    and `test_the_step_from_the_last_0_x_version_to_1_0_0_is_ordered_both_ways` for the first;
    `test_the_first_alpha_of_1_0_0_sits_above_the_last_0_x_version_the_way_windows_orders_it`
    for the second; `test_the_release_workflow_can_publish_the_version_the_tree_already_carries`
    for the third; `test_a_saved_settings_file_names_the_build_that_wrote_it` for the fourth;
    and the four pages, each keeping its old wording dated, for the fifth. The `[S]` line is
    untouched: nothing has been dispatched, and ledger 483 records the `as-is` level as never
    run.
  - Evidence: `grep -n '^version' Cargo.toml` -> `0.125.1` on 2026-09-16 at `524ff24f`;
    `Cargo.lock:6805` carries the same number and nothing else in the tree does
    (`grep -rn '0\.125\.1'` over `*.toml`, `*.lock`, `*.rs`, `*.md`, `*.iss`, `*.sh`, `*.yml`,
    less `target/`, `.planning/` and the changelog, finds those two lines). The Release
    workflow offers `patch`, `minor`, `alpha`, `beta`, `rc` and `release` and no `major`
    (`grep -n major .github/workflows/release.yml` -> nothing). `scripts/build-installer.sh`
    encodes a prerelease as `stage * 1000 + step` into the fourth field of the Windows file
    version, so `1.0.0-alpha.1` reads `1.0.0.1001` and `0.125.1` reads `0.125.1.4000`, and
    Windows orders the four fields numerically, so the first is above the second by its major.
    `src/common/version.rs` compares the three numbers first and a prerelease below its release,
    with tests for `1.0.0` against `0.99.0` and `0.6.0` against `0.6.0-alpha.1`, and none naming
    the exact step this requirement makes. cargo-release's reference says `alpha` on a
    prerelease increments it (`1.0.1-rc.1 -> 1.0.1-rc.2`), `release` strips the suffix, and
    `patch` on a prerelease **also strips the suffix** (`0.1.0-alpha.1 -> 0.1.0`), read from
    `docs/reference.md` in `crate-ci/cargo-release` on 2026-09-16. The first-run screen and
    `--help` name no version shape (`grep -n 'alpha\|version' src/presentation/first_run.rs`,
    `src/presentation/command_line.rs` -> "alpha" as a state, no number), so the issue's list
    of places to touch was two too long; `docs/changelog.md:5`, `docs/BETA_RELEASE.md:42-44`,
    `CLAUDE.md`'s versioning section and `.claude/skills/cutting-a-release/SKILL.md` do name
    the `0.x.y` scheme.
  - [S] #46, the tester on 2026-09-15: "Since we're nearing version 1 with alpha/beta/rc, we
    should make the versions to 1.xx.x instead of 0.1xx.x."
  - [S] `CLAUDE.md`, Versioning and releases: "A prerelease suffix stages a release that is
    about to go to people. When builds start going to testers, cut `0.6.0-alpha.1`."
  - [D] `Cargo.toml` and `Cargo.lock` say `1.0.0-alpha.1`, `--version` prints it, and a test
    on `common::version` names the exact steps `0.125.1` to `1.0.0-alpha.1` to `1.0.0` in
    both directions.
  - [D] A test reads `scripts/build-installer.sh`'s arithmetic for `1.0.0-alpha.1` and for
    `0.125.1` and holds the first above the second in the four-field order Windows uses.
  - [D] The Release workflow can publish the version the tree already carries, without
    bumping it first, so the first alpha published is `1.0.0-alpha.1` and not `-alpha.2`; a
    test in `tests/installer.rs` reads the level and its `cargo release` line.
  - [D] A saved settings file carries the version of the build that wrote it:
    `ConfigManager::save` re-stamps `version` with the running build, red first against a
    file stamped `0.7.7`, and the changelog says a profile now says which build last wrote
    it. Added 2026-09-16: `AppConfig::default()` sets the stamp (`config.rs:651`) and `save`
    copied it through, so the tester's profile, written on 2026-09-15 by 0.125.1, still said
    `0.7.7`, the build that created it, and a profile sent with a report could not say which
    build it came from.
  - [D] `CLAUDE.md`, the changelog's opening paragraph, `docs/BETA_RELEASE.md` and the
    release skill say the rule in force: the tree's version is the next build to go to
    testers; it stays until that build is cut; the first behaviour change after a cut moves
    the prerelease counter in the commit that makes it, once, and later changes before the
    next cut do not move it again; `release` drops the suffix when the round closes; and
    `patch` is never dispatched on a prerelease, because cargo-release reads it as `release`.
  - [S] Whether `1.0.0-alpha.1` is published through the Release workflow is a dispatch, and a
    dispatch is Pratik's. This requirement bumps the tree and makes the dispatch possible.

- [x] **FOUND-02**: A machine set to English (United States) checks spelling in English
  (United States), and the settings screen shows the language that will be used.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-02, merged 2026-09-16 at
    `a3554483`.** The two `[D]` lines each have a name:
    `spellcheck::tests::test_a_bare_stored_language_resolves_to_the_region_this_machine_is_set_to`
    and `test_a_bare_stored_language_takes_the_first_offered_when_the_machine_speaks_another`
    for the first, one resolver asked by the checker and the screen;
    `tests/the_language_the_screen_shows_is_the_one_used.rs` for the second, the real screen
    built with `en` stored and the selection read back. The last `[S]` line stays: his profile
    holds a hand-set `en-US` the resolver leaves as stored, so his machine cannot show the fix
    (ledger 484).
  - Evidence: a scratch test on 2026-09-16 at `524ff24f`, run with `--nocapture` and then
    removed with `git checkout`, printed `system_language() = Some("en-US")`,
    `what_this_machine_offers()` as `TheseLanguages` with nineteen English tags beginning
    `en-029` and holding `en-US` seventeenth, and `language_to_check_in(...) = "en-US"`. So
    on this machine the pure path the issue names answers correctly, and neither of the
    issue's two candidate causes holds as written. What does hold: `default_language()` in
    `src/data/config.rs` was `"en"` for everybody until `580a334a` on 2026-09-03 (`git log
    -S'language_to_check_in' -- src/data/config.rs`), and a stored `"en"` produces both
    symptoms today. `wx_settings.rs` selects the stored tag by exact match and falls to
    index 0 when nothing matches (`.position(|language| language.tag == config.language)
    .unwrap_or(0)`, `build_general_tab`), and index 0 in Windows' order is `en-029`, English
    (Caribbean). And `spellcheck::for_language("en")` resolves a bare tag through
    `find_regional_variant`, which takes the first of the family Windows lists, `en-029`
    again (`test_a_bare_language_matches_the_first_regional_variant_windows_offers`). The
    tester's profile is on this machine (his words, 2026-09-16: "I'm running on this
    machine"), and read through a plain Win32 process it is `%LOCALAPPDATA%\wixen-mail`:
    `app_config.json` 1,921 bytes, 53 keys, written 2026-09-15 18:39, stamped `0.7.7` (the
    build that created the profile; `save` copies the stamp through, which 09-01 changes),
    `"language": "en-US"`, `mark_read_after` `never`, `log_level` `error`, beside a 20 MB
    database with his Gmail account and 12,872 messages (`python -c "import json;
    d=json.load(open(r'C:\Users\prati\AppData\Local\wixen-mail\config\app_config.json'));
    print(d['version'], d['language'])"` -> `0.7.7 en-US`). So the profile was created by
    0.7.7, before `580a334a` of 2026-09-03 changed the default, stored the bare `"en"` that
    lands on Caribbean, and he has since set `en-US` by hand, which is stored. One dated
    line so nobody repeats it: on 2026-09-16 this evidence was written three earlier ways
    from a stale copy, the 886-byte July file stamped `0.1.0-alpha.22` that bash and
    PowerShell started from this harness read at the same path from
    `Packages\Claude_pzs8sxrjxfjjc\LocalCache\Local\wixen-mail`; the tester's profile is read
    through Python from here and never through its shells.
  - [S] #21, the tester on 2026-09-15: "Currently, a Windows OS set to U.S. English as its
    language does not correspond to the default spellcheck language. English Caribbean is
    shown as default."
  - [D] A bare or unlisted stored language resolves to this machine's own region when the
    machine's language is in the same family (`"en"` on an `en-US` machine is `en-US`, not the
    first English Windows lists), in one function both the checker and the settings screen
    ask, and a family with no such member still takes Windows' first.
  - [D] The settings screen shows the language that will be used, never index 0 for a value
    it could not match, and a test builds the real screen with `"en"` stored and reads the
    selection back.
  - [S] The tester has set English (United States) by hand (his words on 2026-09-16: "My
    setting to U.S. English was done manually") and it is stored; the fix leaves it. Whether
    a profile created before 2026-09-03 now shows English (United States) without a hand
    change is a test on such a profile, and his own no longer holds the bare value.

- [x] **FOUND-03**: A snippet of an HTML-only message is the first words of its text, never
  its stylesheet, and so is the text the search index holds for it; snippets and index rows
  already stored are put right.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-02, merged 2026-09-16 at
    `a3554483`.** The first `[D]` line by
    `bodies::tests::test_a_snippet_of_an_html_only_body_is_its_words_and_not_its_stylesheet`,
    `searching::tests::test_an_html_only_body_is_indexed_by_its_words_and_not_by_its_stylesheet`
    and `long_text::tests::test_the_words_of_markup_are_the_words_alone_without_markers_or_stylesheets`,
    with `strip_markup` gone from `src/`; the second by
    `test_stored_snippets_read_from_stylesheets_are_put_right_once_index_included` and
    `test_snippets_are_put_right_once_and_a_second_call_reads_no_body`, and a run of the
    binary against a temp profile that logged one row put right. The last `[S]` line stays:
    his 12,872-message cache has not been opened by this build (ledger 485).
  - Evidence: `src/data/message_cache/bodies.rs`, `strip_markup` at `:317` on 2026-09-16,
    keeps everything between tags, `<style>` content included; `save_message_body` derives
    the snippet through it when there is no plain part, and `searching.rs:360`,
    `index_message_for_search`, takes an HTML-only body through the same function for the
    index's body text (found by the plan check 2026-09-16; `grep -rn strip_markup src`
    answers both), so the index holds stylesheets too and a search for `padding` finds a
    newsletter; `searching.rs` is named by 3 guard records. `application::long_text::from_markup`
    (`long_text.rs:496`) runs `ammonia::clean`, which drops `<script>` and `<style>` content
    outright, then walks the tree with `scraper`, no window involved; a unit test holds
    `blocks_output("<style>body { color: red }</style>") == ""`. `data` already reaches
    `application` 133 times (`grep -rn 'crate::application' src/data/ | wc -l`), so the
    direction is not new. On-open backfills have a shape and a place: `migrate_inline_bodies`,
    `backfill_thread_ids` and `backfill_message_identifiers` run from `MessageCache` open in
    `mod.rs` around `:1390-1435`, each non-fatal with a warning. `bodies.rs` is named by 4
    guard records and `long_text.rs` by 18.
  - [S] #32, the tester on 2026-09-15: "snippets appear to read CSS styles on occasion. So
    far the behavior seems to start with '#outlook' as the start of css."
  - [D] A snippet derived from an HTML-only body, and the body text the search index holds
    for it, are derived through the same reader the reading path uses, so `<style>`,
    `<script>` and `<head>` content never reach either, and `strip_markup` is gone rather
    than patched.
  - [D] Snippets already stored for HTML-only bodies are re-derived once, on the first open
    after the change, through the same function, non-fatally and once only, with the count
    logged, and each row put right is reindexed; the changelog says when a stored snippet is
    put right and what it costs.
  - [S] Whether the tester's rows now read as words is a test on his profile after the
    first open of the new build.

- [x] **FOUND-04**: Undo Send is on the Edit menu.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-03, merged 2026-09-16 at
    `f58b9271`.** The one `[D]` line by `tests/undo_send_is_where_somebody_looks.rs`, three
    readings: first on Edit and not on Tools, and two companions that put it back on Tools
    and on both menus.
  - Evidence: `src/presentation/wx_app.rs` on 2026-09-16: the Edit menu is built at `:6109`
    with Cut, Copy, Paste, Select All, Search and Save This Search; `ID_UNDO_SEND`'s item is
    appended inside the Tools menu at `:6904` with `Ctrl+Shift+Z`; `docs/KEYBOARD_SHORTCUTS.md:413`
    lists it under Application Control without naming a menu. `tests/wired.rs` holds the item
    to its handler by id, and nothing reads which menu it is on.
  - [S] #44, the tester on 2026-09-15: "undo send should be in the edit menu."
  - [D] Undo Send is the first item on the Edit menu, keeps `Ctrl+Shift+Z`, and is not on
    Tools; a reading holds the Edit menu's first item to `ID_UNDO_SEND`; the shortcuts page
    says which menu.

- [x] **FOUND-05**: A held meeting answer says it is held and how to take it back, and says
  the organiser has been told only once it has gone.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-03, merged 2026-09-16 at
    `f58b9271`.** The first `[D]` line by
    `answering::tests::test_a_held_answer_says_which_answer_went_and_the_countdown_every_send_says`,
    `test_an_answer_with_the_hold_off_says_it_is_sending_and_not_that_anybody_has_heard` and
    `test_an_answer_queued_while_offline_says_it_waits_for_a_network`, with "has been told" in
    no production line and `HowItWent::Sent` retired for a variant whose comment says what
    happened on this machine; the second by
    `test_the_schedule_routines_comment_names_the_key_that_reaches_it` and
    `grep -c 'Alt+E' src/presentation/wx_compose.rs` at 0. The last `[S]` line stays: ledger
    155's listening question, and nobody has heard the sentence.
  - Evidence: `src/presentation/wx_app.rs` on 2026-09-16: `send_the_answer` at `:13129`
    queues the reply through `queue_for_sending`, which returns a `GoAfter` and holds the
    message like any other; on `Ok` it answers `HowItWent::Sent`, whose doc comment in
    `src/application/answering.rs:336` says "The answer reached the organiser's mail server",
    which is not what happened; `what_answering_did` then says
    `invitations::what_happened`, whose last sentence is "<Organiser> has been told."
    (`invitations.rs:446`). `sending_later::countdown` (`:681`) words the hold for the
    composer: "Sending in 10 seconds. Undo Send takes it back." `wx_compose.rs:129` still says
    "Alt+E out of the message body" where the key is Alt+H (`:696`, `:703`).
    `answered_meetings::file_the_answer` files the meeting on the calendar from `went`, so
    what a held answer files is a decision the plan carries.
  - [S] #56, from the audit of 2026-09-15: "the sentence afterwards is '<Organiser> has been
    told.' with no countdown and no mention of Undo Send, so for that path the hold is not
    announced when it starts and the wording claims delivery during it."
  - [D] Answering a meeting says, at the moment of pressing, the same countdown any other
    send says, through the same function; nothing is said when the held answer leaves, as
    with any other held message, so "has been told" is not said at any point on this path
    and the sentence is retired with its test; `HowItWent` has a variant for a held answer
    and its doc comment says what happened. (Reworded 2026-09-16 after the plan check: the
    first wording, "not said while the answer is held", read as though it were said
    afterwards.)
  - [D] The `wx_compose.rs` doc comment names Alt+H, and a reading holds it.
  - [S] Ledger 155's question, whether anything spoken after pressing Accept by mistake
    points at Undo Send, is a listening pass and stays open until somebody hears it.

- [x] **FOUND-06**: The two sort controls on the Reading tab sit together, and a compose
  setting is on the Compose tab.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-04, merged 2026-09-16 at
    `c928cae4`.** The first `[D]` line by `tests/the_sort_controls_sit_together.rs`, the real
    tab built and the order read back; the second by
    `config::tests::test_every_setting_somebody_can_change_is_offered_by_a_screen` green with
    the control inside `build_compose_tab`. The last `[S]` line stays (ledger 487).
  - Evidence: `src/presentation/wx_settings.rs` on 2026-09-16: "Default sort order" is built
    by hand into `list_sec`, the Message List section, at `:1142-1176`; "Then by" is
    `labelled_choice(panel, &date_sec, ...)` at `:1516-1523`, inside the Dates and Times
    section straight after "Write the month as" at `:1508`; "Cc and Bcc lines" follows it at
    `:1524-1531` in the same section and is a compose setting. The dialog has seven pages,
    not the five #33 and #34 say: General, Compose, Reading, Permissions, Calendar and PIM,
    Feedback, Advanced (`notebook.add_page` at `:241` to `:299`). `wx_settings.rs` has no
    unit tests, so `--lib presentation::wx_settings::` matches nothing; the gate reaches it
    through `every_event_has_a_control` and `checkbox_labels` (`check.sh --suites-for`), and 8
    records name it.
  - [S] #36, the tester on 2026-09-15: "On reading tab, the two sort options do not appear
    together. They are separated by multiple tab stops." And on the same day: "The first
    time the user tabs from the 'reading' tab, they hear 'default sort order'. As they
    continue to tab, they hear 'then by'. This option immediately follows 'write the month
    as'."
  - [D] "Then by" is built into the Message List section directly after "Default sort order",
    so the two are adjacent tab stops; a test builds the real tab and reads the order back.
  - [D] "Cc and Bcc lines" is on the Compose tab, and
    `test_every_setting_somebody_can_change_is_offered_by_a_screen` stays green.
  - [S] Whether the tab now reads as one group by ear is a listening pass.

- [x] **FOUND-07**: Exactly one item in the View menu's Sort submenu is checked.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-04, merged 2026-09-16 at
    `c928cae4`.** The one `[D]` line by `tests/one_sort_is_checked.rs`, the chain read as one
    radio group with no separator and `sync_sort_menu` following every sort, and by
    `tests/one_sort_is_checked_on_a_live_menu.rs`, a real menu bar showing one tick. Whether
    the running program shows one tick after a header click is ledger 488.
  - Evidence: `src/presentation/wx_app.rs:6160-6197` on 2026-09-16: seven `append_radio_item`
    calls in four runs with `append_separator` at `:6171`, `:6182` and `:6193` between them.
    wxWidgets ends a radio group at a separator, so those are four groups. `sync_sort_menu`
    at `:14169`, which the issue did not name, checks the chosen id and unchecks nothing,
    called at `:1212` and `:3102`.
  - [S] #39, the tester on 2026-09-15: "Sort options in the view menu are confusing as
    multiple items are checked."
  - [D] The seven sort items are one radio group, so choosing one unchecks the rest whichever
    way the sort was chosen, including from a column header; a reading holds the submenu's
    builder to having no separator between its first and last radio item.

- [x] **FOUND-08**: Every checkbox in the editors is named on the channel NVDA reads, no
  empty static text sits before a control as a spacer, and the check that refuses a
  whitespace label refuses that shape too.
  - **Ticked 2026-09-18 by 11-02 on the walk it waited for: Accessibility run 35336142914,
    on `main` at `744d05ef`, the morning's push, walked the five editors over MSAA and found
    no control without a name on any of them; the second `[D]` line below quotes the five
    lines. Ledger 489 closed on that run. The walk still crashes on this machine (ledger 390)
    and 11-02 did not run it. The last `[S]` line stays (ledger 490).**
  - **Read 2026-09-17 by the phase's closing read in 09-10; 09-05, merged 2026-09-16 at
    `165fd811`; left open on its second `[D]` line.** The first `[D]` line by
    `scan_target::tests::test_every_window_a_fresh_profile_can_reach_has_a_name` and
    `test_the_workflow_asks_for_every_target`, each of the five started here on a throwaway
    profile; the naming half of the second by `tests/checkbox_labels.rs`; the third by
    `tests/no_label_is_only_a_space.rs` and its companion. The walk half of the second line is
    open: `scripts/msaa-names.ps1` left with -1073740791 on every run here, before and after,
    and the Accessibility workflow has not run since, because nothing has been pushed (ledger
    390, 489). It closes when CI has walked the five editors and reported a name for every
    checkbox; whoever reads that run ticks this. The last `[S]` line stays (ledger 490).
  - Evidence: `grep -rn 'with_label("")' src/presentation/*.rs` finds 22 on 2026-09-16, the
    issue's count. Read one by one: eleven are status or problem lines something fills later
    or names outright, and eleven are spacers nothing ever fills. The five the issue names in
    `wx_managers.rs` are at `:1285` (Favourite, contact editor), `:2826` and `:3285` (Case
    Sensitive, the condition editor and the filter editor), `:3309` (Enabled) and `:3837`
    (Default signature), and none of those five checkboxes gets `set_accessible_name`
    (`grep -c 'set_accessible_name(&fav_check'` and the other three -> 0 each). The account
    manager's `cb` and `cb_with_description` closures at `wx_account_manager.rs:1464-1488`
    do place a spacer, and **do name every checkbox through `set_accessible_name`** with the
    comment "every other builder in this dialog says the name outright", so #42's comment that
    those go on the same fix is right about the spacers and wrong about the names. Four more
    spacers sit before things that are not checkboxes: `:1459` (a section heading's partner),
    `:1539` (the sign-in hint), `:1642` (the app-password button), `:1712` (the allowed
    note). The scan cannot reach any of these editors: `ScanTarget::Contacts`,
    `::Signatures` and `::Filters` open the managers, and the editors behind Add are
    nested modals, which 06-06 counted among the seventeen outside the scan; `ScanTarget::WhichDays`
    and `::SendLater` show how a nested dialog is opened directly with a fixture.
    `tests/no_label_is_only_a_space.rs` reads `with_label(` and `set_label(` literals that are
    non-empty whitespace and is named by one record. Ledger 408: a spin control's text field
    is a different shape, `set_accessible_name` landing on the arrows, and is not this class.
  - [S] #42, the tester on 2026-09-15: "There is an unlabeled checkbox in the signature
    compose field." #40: "There is an unlabeled checkbox on the basic tab."
  - [S] `docs/wcag-coverage.md` and the MSAA script's own header: a control with a visible
    label beside it is named by that label on MSAA even when nothing set one, and a checkbox
    carries its own text, so why these two are heard unnamed is measured before it is
    explained.
  - [D] The five editors (contact, condition, filter, signature, account) are scan targets
    opened directly with a fixture, in the workflow's list, so both channels reach them.
  - [D] Every checkbox in those editors is named through `set_accessible_name` with the
    mnemonic stripped, and the MSAA walk on each editor reports a name for every checkbox,
    on this machine or on CI's next run. Ledger 390 records the walk crashing here with
    STATUS_STACK_BUFFER_OVERRUN, "Not diagnosed", NVDA running being a difference between the
    machines and not a cause; stopping NVDA is not asked for, and this line stays open until
    one of the two has walked the five editors. **Walked on CI, 2026-09-18, run 35336142914
    at `744d05ef`, read by 11-02 with `gh run view 35336142914 --log`: `contact-editor`
    "Walked 2 window(s): 'Edit Contact', 'Wixen Mail'", "MSAA walk: 2915 elements, 1718 of
    them operated, 0 without a name"; `condition-editor` 'Edit Condition', 1978, 1158, 0;
    `filter-editor` 'Edit Filter Rule', 2126, 1242, 0; `signature-editor` 'Edit Signature',
    1889, 1114, 0; `account-editor` 'Edit Account', 3120, 1833, 0. Five walks, none unnamed;
    this line closes on them.**
  - [D] No `StaticText` built with an empty literal is added to a sizer and never filled or
    named afterwards; the empty spacers go, a sizer spacer keeps the grid where one is needed,
    and `tests/no_label_is_only_a_space.rs` refuses the shape with a companion that plants one.
  - [S] What the tester hears on the signature editor and the contact editor afterwards is a
    listening pass.

- [x] **FOUND-09**: Arrowing through the Settings tab row says each tab once.
  - **Ticked 2026-09-18 by 11-02 on the tester's ear, on the `[S]` line's own terms:** #33
    closed 2026-09-18T11:34:22Z on his word, quoted: "Heard on 2026-09-18 by the tester on
    `1.0.0-alpha.1+149.g744d05ef` with NVDA: the Settings dialog speaks as it should;
    arrowing along the tab row says each tab once." The runner's case is the harness's own
    remaining work: it ran once, in NVDA run 35336142908 at `744d05ef`, and heard nothing,
    because it waited for an opening announcement the harness has never captured for any
    case; 11-02 corrected the wait (the third `[D]` line below says what the corrected case
    proves and no longer proves) and the next push of `main` runs it, ledger 531. Ledger 492
    closed on the tester's word.
  - **Read 2026-09-17 by the phase's closing read in 09-10; 09-06, merged 2026-09-16 at
    `de58771a`; left open on its third `[D]` line.** The first `[D]` line by
    `scripts/uia-events.ps1` and the capture quoted in 09-06's summary, 42 events over six
    presses before anything changed; the second by
    `tests/the_settings_tab_row_says_each_tab_once.rs`, a real `WM_KEYDOWN` sent to the built
    dialog and one focus event counted, and the capture taken again afterwards showing one per
    press. The third is open: `nvda-tests/tests/settings-tabs-read-once.test.js` is written and
    syntax-checked and has not run, because it runs only on the NVDA workflow at a push of
    `main` and nothing has been pushed (ledger 492). It closes when that run holds each tab to
    once; whoever reads that run ticks this. The last `[S]` line stays.
  - Evidence: `src/presentation/wx_settings.rs` on 2026-09-16 builds a `Notebook` with seven
    pages and announces nothing on a page change (`grep -n announce` finds save and validation
    only). Accessibility Insights for Windows is not installed on this machine
    (`ls "$LOCALAPPDATA/Programs"` -> no such entry), so its event viewer cannot be the
    instrument. `scripts/msaa-names.ps1` walks a tree once and records no events.
    `nvda-tests/` drives a real NVDA on CI against the real binary and records what it said,
    and its README says it never runs on this machine. NVDA's own log at debug level records
    every focus, selection and name-change event it acts on, and a PowerShell UI Automation
    client can subscribe to the same events without NVDA present.
  - [S] #33, the tester on 2026-09-15: "Arrowing left/right on a tab list in the settings
    dialog frequently reads the focused tab twice."
  - [D] The event stream during Right and Left on the tab row is captured on this machine by
    a UI Automation event logger kept in `scripts/`, and the capture is in the summary before
    anything changes.
  - [D] Whatever the capture names as the second event is stopped at its source, and a test
    holds the handler that stops it.
  - [D] An `nvda-tests` case arrows through the Settings tabs and holds the transcript to
    each tab name once; it runs on CI at the next push. **Ran once, 2026-09-18, in NVDA run
    35336142908 at `744d05ef`, and heard nothing: "never heard all of ["General"] within
    15000ms. Everything NVDA said: []", at its first wait, before any key. The transcript of
    every case that passed in that run begins with what its first key made NVDA say and
    none holds a dialog's opening announcement, so the wait was for something the harness
    never captures. 11-02 replaced it with the settle and the mark; the corrected case
    proves the six tabs after General heard once each going Right, in order, General not
    heard going Right, and Feedback once going Left, and no longer proves General's own
    reading at open, which only the tester's ear settled (#33, closed 2026-09-18). The next
    push of `main` runs it, ledger 531; which tab the first Right reaches on the runner's
    fresh profile is what it shows.**
  - [S] Whether the tab is now heard once is a listening pass on the tester's machine.

- [x] **FOUND-10**: Every surface that shows a message tries the PGP key, states the S/MIME
  envelope and carries the signature bar, from one path.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-07, merged 2026-09-17 at
    `f990d023`.** The first `[D]` line by four tests in `application::reading_a_message`
    (`test_a_message_encrypted_to_the_imported_key_is_shown_as_its_words`,
    `test_an_armoured_message_with_no_key_here_says_why_and_keeps_its_armour`,
    `test_a_signed_message_in_the_cache_has_its_verdict_and_an_enveloped_one_its_sentence`,
    `test_the_envelope_is_folded_in_before_the_signature_so_it_is_spoken`) and by
    `tests/wired.rs`'s `test_opening_a_message_tries_the_pgp_key_and_says_why_it_did_not_open`
    over six surfaces; the second by five `encryption_tests` in `presentation::reader_text`;
    the third by `THE_SURFACES` in `tests/wired.rs` with its companion
    `test_a_surface_that_stopped_asking_the_composition_is_named`, and
    `grep -c 'Corrected on 2026-09-16' docs/changelog.md` at 3. The last `[S]` line stays
    (ledger 495).
  - Evidence: `src/presentation/wx_app.rs` on 2026-09-16: `opening_pgp::for_body` is called
    at `:11195` (Shift+Space, `read_the_whole_message`) and `:12464`
    (`open_in_the_text_reader`); `envelope_check_for` at `:11184` and `:12478`;
    `open_single_message`'s Formatted branch at `:12418-12441` passes only
    `signature_check_for` into `show_conversation_as_page` (`:20247`), whose bar is
    `reader_text::conversation(subject, parts).with_signature(signature)` and nothing else;
    the preview pane renders `reader_text::conversation_html` at `:17158` with no bar; and
    `open_conversation` at `:12577`, reached at `:20200` for `ThreadChoice::WholeConversation`,
    opens `reader_text::conversation` in the text reader with no bar and no PGP opening, a
    sixth surface the issue did not count (found by the plan check 2026-09-16). The
    guard `test_opening_a_message_tries_the_pgp_key_and_says_why_it_did_not_open`
    (`tests/wired.rs:4087`) names `open_in_the_text_reader` and `read_the_whole_message`
    only. The mailbox-import rustdoc sits above `import_a_pgp_private_key` at `:12760-12787`
    ahead of its own. The changelog's PGP entry is at `:1630` and its S/MIME envelope entry
    near `:2348`, moved from the lines the issue cites.
  - [S] #51, from the audit of 2026-09-15: "the PGP key and the S/MIME envelope sentence are
    only tried on the plain-text reader and on Shift+Space, never on the default Formatted
    reader or the preview pane."
  - [D] One function composes, for a message and its body, the opened body, the PGP finding,
    the envelope sentence and the signature verdict, and every surface asks it, six of them:
    the text reader, Shift+Space, the Formatted reader, the conversation window as headings,
    the whole conversation in the text reader, and the preview pane.
  - [D] The preview pane's document carries the bar above the message.
  - [D] The `wired.rs` guard names every surface, and the changelog entries are corrected by
    dating; the stray rustdoc is put with the function it describes.
  - [S] Nothing here has met a real correspondent's key or a real signed message; that is
    the standing condition and stays.

- [x] **FOUND-11**: Mail import and export do what their commands and documents say: the
  Outlook data file reader is reached from the picker, Save As writes a file, and a folder
  can be chosen.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-08, merged 2026-09-17 at
    `06fdc9b7`, and 09-10, merged 2026-09-17 at `8eba6a38`.** The first `[D]` line by
    `import_tree::tests::test_an_outlook_data_file_goes_to_its_own_reader`, five tests in
    `application::importing_an_outlook_data_file` and `tests/wired.rs`'s
    `test_importing_mail_sends_each_kind_of_file_to_its_own_reader`; the second by four
    `importing_messages` tests, three `export_tree` tests and the two `wired.rs` readings of
    Save As; the third by `tests/wired.rs`'s
    `test_a_folder_of_messages_can_be_chosen_and_goes_to_the_import_worker` with its companion
    `test_the_reading_of_the_folder_import_can_see_the_hand_over_taken_out`, the dated sentence
    on the changelog's import entry, and the Known limitations line naming `.sbd` and `.msf`;
    the fourth by the four dated sentences quoted in 09-10's summary and the guide's Import and
    Export section. Both `[S]` lines stay: points 4 to 6 are later work with the issue open
    (ledger 510 for the Thunderbird layout), and no real Outlook data file has been read
    (ledger 499); nobody has run the folder item by hand (ledger 509).
  - Evidence: `src/service/outlook_data_file.rs` is 3,608 lines with 38 tests, referenced
    only by `pub mod` in `service/mod.rs:28` (`grep -rn outlook_data_file src` less itself
    -> one line), and named by no guard record; it yields
    `ItemInTheDataFile::{Mail(Vec<u8>), Appointment, Contact, Task, Note}` with the cache's
    own entry types (`:2176`). The picker at `wx_app.rs:12872` lists `*.zip;*.eml;*.mbox`.
    `import_tree::what_was_chosen` (`:513`) answers `AnArchive` or `MailInOneFile` from the
    file's first bytes. `ID_SAVE_AS`'s handler at `:5027` sends "Save As: no message
    selected" whatever is selected; the item is at `:6026` and `docs/KEYBOARD_SHORTCUTS.md:485`
    promises a file. `message_files::written_as_one_message` (`:232`) is reached by
    `export_tree.rs:814` and by the `.pst` reader itself at `outlook_data_file.rs:1701`,
    which composes each `Mail` item through it, so an imported message's bytes are one saved
    message and take the path a `.eml` takes (the first draft of this line said the exporter
    only; the plan check re-ran the grep on 2026-09-16). The attachment list is in the
    reader frame, which has its own Save Attachment command (`wx_reader.rs:35`, `:794`,
    `save_attachment_now` at `:878`), and the main frame's menu is not active while the
    reader has focus, so a Save As branch for an attachment on the main frame would reach
    nothing. The changelog at `:4667` says the reader works and
    was never run against a real file; the `[Unreleased]` import entry near `:4680` promises
    "a folder you point it at" while the picker is a `FileDialog`; `DirDialog` is already
    used at `wx_app.rs:2230` for `.vcf` folders. The four documents: `docs/comparison.md:102`,
    `docs/privacy.md:89`, `.planning/intel/built-and-left.md:55`,
    `.planning/codebase/INTEGRATIONS.md:58`.
  - [S] #53, from the audit of 2026-09-15: "the .pst reader is unwired, Save As is a stub,
    and a folder import is promised but impossible."
  - [D] File, Import Mailbox lists `*.pst`, a chosen data file is read through
    `outlook_data_file` on the import worker, each `Mail` item goes through
    `each_message_in(bytes, ReadAs::OneMessage)` and `file_one_imported_message` under
    Imported the way a saved `.eml` does, its appointments, contacts, tasks and notes land
    on this computer through the cache's existing writers, and the closing sentence counts
    each kind and what stayed behind, saying the reader has never met a real file. (09-08)
  - [D] File, Save As writes the message under the cursor in the list as `.eml` through
    `written_as_one_message`, through the ordinary save dialog, and refuses with a reason
    when nothing is selected; an attachment is saved by the reader's own Save Attachment
    command as before, and the shortcuts page says which command does which. (09-08)
  - [D] A folder of saved messages can be chosen through a directory picker on its own File
    item and goes down the directory branch `mailbox_archive::opened` already has; the
    changelog's promise is true with a dated sentence saying it was impossible until then;
    and the changelog says a Thunderbird profile folder (an mbox file beside an `.sbd`
    directory and an `.msf` index) is not recognised as such. (09-10)
  - [D] The four documents describe what is reachable, dated where they change, and the
    user guide names the three File commands. (09-10)
  - [S] #53's points 4 to 6 (a bare `.mbox` or loose `.eml` export, `.msg`, `.pst` export)
    are later work and the issue stays open for them; point 7, the guide, closes with the
    fourth `[D]` line.
  - [S] No real Outlook data file has ever been read here, and the changelog keeps saying so.

- [x] **FOUND-12**: Settings opens at once, and the number is on the measurements page.
  - **Closed 2026-09-17 by the phase's closing read in 09-10; 09-09, merged 2026-09-17 at
    `a8b26596`.** The first `[D]` line by `tests/the_settings_dialog_opens_in.rs` at
    `d169df71`, the five rows dated 2026-09-16 and the line held by
    `started::tests::test_the_settings_built_line_is_held_byte_for_byte`; the second by its
    "built when its tab is first shown" arm,
    `test_pages_after_the_first_are_built_when_their_tab_is_first_shown_and_read_from_the_settings_when_never_shown`,
    `test_the_dialog_and_each_later_page_are_built_frozen`, the six after rows dated
    2026-09-17, and `test_every_setting_somebody_can_change_is_offered_by_a_screen` green. The
    last `[S]` line stays (ledger 504, 507).
  - Evidence: `build_settings_dialog` (`wx_settings.rs:209`) builds seven pages before
    `show_modal`; `available_languages()` at `:638` creates the Windows spell-checker factory
    and asks `GetLocaleInfoEx` per tag; `fonts::installed_families()` at `:762` walks
    `EnumFontFamiliesExW`; `SoundScheme::discover` at `:2714` reads the schemes directory.
    None is remembered between opens. `common::started` marks `main`'s first instant and
    words one line a harness parses; `tests/the_numbers_the_targets_ask_for.rs` starts the
    release binary and reads it. `build_settings_dialog` is buildable without showing, and
    `tests/every_event_has_a_control.rs` and `tests/checkbox_labels.rs` already build it in a
    test process, one window per process. Nothing has timed any of it.
  - [S] #34, the tester on 2026-09-15: "Loading settings by pressing ctrl+, is noticeably
    slow."
  - [D] The time from `Ctrl+,` to the dialog built is measured on this machine on the
    release binary and each candidate cost on its own, and the rows are on
    `docs/development/measurements.md` with the machine and the build, before anything
    changes.
  - [D] Whatever the rows name as the cost is asked of Windows once per process and kept, or
    built when its tab is first shown, and the after row sits beside the before row; every
    setting is still on the settings screen and `test_every_setting_somebody_can_change_is_offered_by_a_screen`
    stays green.
  - [S] Whether it feels at once on the tester's machine is his to say.

**Added 2026-09-17: two regressions of FOUND-12's second `[D]` line, found by the tester in
`1.0.0-alpha.1` (`7d57cd49`) and fixed by phase 10's inserted plan 10-01.1.** They sit here,
beside the fix they correct, rather than under phase 10's mail section, because a reader of
FOUND-12 should find what its "built when its tab is first shown" cost. `FOUND-13` was, for
part of 2026-09-16, the withdrawn #66 (the coverage block below says so); that id was removed
the same day and is used again here, and this sentence is so nobody reads the old reference
as this one. Every evidence line was re-taken against `main` at `f3be1ef5` on 2026-09-17;
`git diff --stat 7d57cd49 HEAD -- src/presentation/wx_settings.rs` prints nothing, so the
lines are the ones the diagnosis in the planning session quoted.

- [x] **FOUND-13**: Every checkbox on every Settings tab reads as a check box with its checked
  state, and toggles on Space with the new state announced, after its page is built lazily.
  - **Closed 2026-09-17 by 10-01.1, merged at `d020aa60`.** The first `[D]` line by the paint
    moved into `APage::built` on both paths, with
    `test_pages_after_the_first_are_built_when_their_tab_is_first_shown_and_read_from_the_settings_when_never_shown`
    and `test_one_arrow_on_the_settings_tab_row_raises_one_focus_event` green; the second by
    `tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs`
    (`test_every_checkbox_on_a_later_page_answers_check_button_after_set_selection`,
    `test_bm_click_toggles_the_checked_state_on_every_later_page`,
    `test_the_reading_tells_a_push_button_from_a_check_box`) and the record "a later Settings
    panel is painted after its controls exist, so its check boxes stay check boxes". The `[S]`
    lines stay (ledger 513).
  - Evidence: `build_settings_dialog` (`wx_settings.rs:480`) paints all seven page panels at
    `:629-644` before the six pages after General have any control; those pages are built on
    the page-changed event by `LaterPages::build_the_page_for` (`:284-318`) through the
    accessors at `:344-379`. wxWidgets 3.3.2 hands a parent's foreground colour to every
    child created after it (`src/common/wincmn.cpp:1524-1552`) and makes a checkbox given
    one owner-drawn (`src/msw/control.cpp:422-444`), so its `BS_CHECKBOX` style becomes
    `BS_OWNERDRAW`, which Windows' standard accessible object reports as a push button with no
    checked state; wx's correcting `wxCheckBoxAccessible` (`src/msw/checkbox.cpp:289-328`) is
    discarded by `names::set_accessible_name`, which installs `FixedName`, answering nothing
    for role and state. Measured over MSAA from a test that builds the real dialog: at
    `7d57cd49` all 45 later-page checkboxes are `BS_OWNERDRAW` and answer
    `ROLE_SYSTEM_PUSHBUTTON` with a state that does not move on a click; General's 6 and all
    57 at `3e633252` are `BS_CHECKBOX`, `ROLE_SYSTEM_CHECKBUTTON`, and toggle. Under Dark,
    Light and Default alike; only High Contrast is spared, because no palette is painted then.
  - [S] #67, the tester on 2026-09-17: "every checkbox on every tab except General is read as
    a button. Pressing one does not appear to change its state: nothing is announced, and the
    checkbox does not read as checked afterwards."
  - [D] Each of the six later panels is painted at the end of its own build, on both build
    paths, so a checkbox is created first and painted after, the order `3e633252` had; the
    lazy build, the freeze, `FixedName` and the tab row's arrow handler are untouched, and
    the readings 09-09 and 09-06 left stay green.
  - [D] A reading builds the real dialog, reaches each later tab the way the arrow keys do,
    and asserts over `AccessibleObjectFromWindow` that every `Button`-class descendant that
    is not a group box or a push button is `BS_CHECKBOX` or `BS_3STATE` and answers
    `ROLE_SYSTEM_CHECKBUTTON`, and that a click moves its checked state, in the default and
    dark themes, with a companion that sees a push button as not a check box and sees the
    fault when a checkbox is created under a painted, then named, parent; a guard record
    couples it to `wx_settings.rs`.
  - [S] Whether NVDA says "check box" and its state, says the new state on Space, and reads it
    back after Tab away and back, on the next build, is the tester's; so is whether General
    sounds as before and Settings still opens at once.

- [x] **FOUND-14**: After Ctrl+Tab or Ctrl+Shift+Tab from inside a Settings page, focus rests
  on the first control of the reached page, or on the tab row, and NVDA names it.
  - **Closed 2026-09-17 by 10-01.1, merged at `d020aa60`.** The first `[D]` line by
    `first_in_tab_order` on each page and the hand-off in `build_the_page_for`, held by
    `test_a_page_reached_from_inside_a_page_puts_focus_on_its_first_control` and
    `test_a_page_reached_from_the_tab_row_leaves_focus_on_the_row`; the second by
    `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
    with `test_the_reading_tells_the_page_panel_from_a_control_on_it` and the record "a
    Settings page reached from inside another page hands focus to its first control". The
    `[S]` lines stay (ledger 513).
  - Evidence: `wxNotebook::SetSelection` calls `UpdateSelection` before it sends the
    page-changed event (`src/msw/notebook.cpp:342-361`), and `UpdateSelection` gives the new
    page focus when the notebook is visible and does not itself hold focus (`:364-391`, line
    386). Since `8162ef57` the page is empty then, so `wxSetFocusToChild` finds nothing and
    native focus lands on the panel itself; the handler builds the controls afterwards and
    nothing in `wx_settings.rs` moves focus (`grep -c 'set_focus\|has_focus'` is 0). On an
    arrow key the row holds focus, so nothing moves and the tab is announced once (#33).
    Found from the source while tracing #67, not heard. Asking the panel to move focus after
    the build is refused by wx (`src/common/containr.cpp:110-150` returns when the focused
    window is the panel), and wxdragon 0.9.17 binds no notebook page-changing event, so the
    page has to name its first control.
  - [S] #68, from the source on 2026-09-17: "With focus on a control on the General tab of
    Settings, press Ctrl+Tab. NVDA says 'pane' and nothing else until Tab is pressed. Arrowing
    along the tab row does not do this."
  - [D] After the page-changed handler builds a page, if native focus is on the page panel
    itself, the page hands it to its first control in tab order; when the tab row holds
    focus nothing moves, so #33's single focus event stays.
  - [D] A reading builds the real dialog, focuses a General control, moves the selection the
    way Ctrl+Tab does once wxWidgets has translated it, and asserts `GetFocus()` is the
    first tab-stop child of the reached page and not the panel, on Compose and then on
    Reading; an arrow on the row leaves focus on the row; a companion sees focus planted on a
    panel; a guard record couples it to `wx_settings.rs`.
  - [S] Whether NVDA names the control at once after Ctrl+Tab, or says the page before it, is
    the tester's.

**#66 was filed and withdrawn on 2026-09-16, and its one real item is in FOUND-01.** It said
settings were never written and the log stopped or stayed empty, from a read of the tester's
profile made through a shell started from this harness, which reads a stale July copy of
`%LOCALAPPDATA%\wixen-mail` at the same path; read through a plain Win32 process the profile
was written on 2026-09-15 with his hand-set language, and its `log_level` is `error`, which is
why the day's log is empty. No settings defect and no log defect. What the read did find is
that a settings file's `version` stamp is copied through by `save` and names the build that
created the profile, so FOUND-01 gains the line that `save` re-stamps it.

**Added 2026-09-17, later still: one defect the tester found in `1.0.0-alpha.1` (`59c5b6a4`)
on its second day, and one decision Pratik made the same day about what a build carries,
taken by phase 10's inserted plans 10-02.1 and 10-02.2.** They sit here rather than under
phase 10's mail section for the reason FOUND-13 and FOUND-14 do: neither is about the mail
coming down. `FOUND-15` is a sort the combined view forgets, which is a listing defect of the
kind 10-02 just measured; `FOUND-16` is the versioning rule FOUND-01 wrote gaining a counter,
and it takes the `FOUND` id because that is where the version rule lives (FOUND-01) and `SHIP`
is the installer's signing and its updates, not what a build is called. Every evidence line was
re-taken against `main` at `c5ee5085` on 2026-09-17 with the command in the plan.

- [x] **FOUND-15**: All Inboxes, a label view and a saved search's results are read in the
  sort that was chosen, the way a folder is, and keep it when the view is reopened.
  - **Closed 2026-09-17 by 10-02.1, merged at `c0505f68`.** The first `[D]` line by
    `tests/all_inboxes_reads_in_the_sort_that_was_chosen.rs`
    (`test_all_inboxes_answers_in_the_order_it_is_handed`,
    `test_a_label_view_answers_in_the_order_it_is_handed`,
    `test_what_a_saved_search_found_answers_in_the_order_it_is_handed`,
    `test_the_sort_the_menu_stores_is_read_back_into_the_order_the_cache_takes`, with
    `test_all_inboxes_handed_no_order_is_newest_first_as_before` for the default) and the
    guard `test_no_query_a_folder_listing_runs_reads_message_text_or_a_table_it_may_not`
    widened to the three listings in every order; the second by
    `test_the_window_reads_all_inboxes_a_label_and_a_search_in_the_sort_that_was_chosen` with
    `test_the_reading_would_see_a_loader_that_forgot_the_sort`, and the four records with
    `suite = "all_inboxes_reads_in_the_sort_that_was_chosen"`, one per file, measured; the
    third by `test_every_all_inboxes_row_has_the_pages_shape_and_names_its_order` and the
    ten rows named "All Inboxes in a chosen sort after 10-02.1" dated 2026-09-17 at
    `b39a7409` on `docs/development/measurements.md`. One thing the first line's test found
    beyond the requirement: the menu's Unread First stored read first, corrected with
    `test_unread_first_from_the_menu_puts_unread_rows_first_and_newest_beneath` in
    `message_columns.rs`. The last `[S]` line stays (ledger 516).
  - Evidence: a folder is read through `get_message_list_sorted` with the stored sort in the
    query (`load_folder_messages`, `wx_app.rs:13716`, asking `the_sort_as` at `:13784`);
    All Inboxes is `load_every_inbox` (`:7050`) calling `unified_inbox(None)`, whose query
    carries `ORDER BY m.date DESC, m.uid DESC` and takes no order
    (`messages.rs:403-417`); a label view is `load_messages_with_label` (`:7005`) calling
    `messages_with_label(.., None)` over its own inline query with the same fixed order
    (`tags.rs:315-343`); a saved search lists the newest 500 it found through
    `message_rows_for` (`wx_app.rs:7360`, `saved_searches.rs:459`), whose `results_query`
    carries the same fixed order (`:607`). Choosing a sort from the menu sorts the rows in
    memory and saves the choice (`sort_from_menu`, `:14361`), so it holds until the row is
    read again. `Sort::order_by_clause` builds every term from an `m.` column
    (`message_columns.rs:148-170`), which all three queries alias as the folder listing does.
    The index `idx_messages_date` serves the fixed order only (`mod.rs:3267`); `Received`
    sorts on `COALESCE(m.internaldate, m.date)`, which no index serves, so every chosen sort
    is a sort of every inbox row, as a folder's already is. The loaders are private, so the
    composed run through `load_every_inbox` is reachable only from `wx_app.rs`'s test module,
    which costs 58 guard records and reads the machine's own profile because nothing there
    pins `WIXEN_MAIL_DATA`.
  - [S] #69, the tester on 2026-09-17: "With All Inboxes open, choose a sort from View, Sort
    Messages: the list re-sorts and the choice is announced. Move to another folder and come
    back to All Inboxes: the list is back in newest-first order."
  - [D] `unified_inbox`, `messages_with_label` and `message_rows_for` take an order from
    `Sort::order_by_clause` and put it in the query with `m.uid DESC` after it, the fixed
    order as the default when handed none; a test on the cache holds each to every sort the
    menu offers, both ways, and the sort the menu stores read back the way `the_sort_as`
    reads it is a clause the cache honours.
  - [D] `load_every_inbox`, `load_messages_with_label` and `run_a_saved_search` ask
    `the_sort_as(view_state::Showing::Messages)` and pass it, held by a reading of the
    shipping half of `wx_app.rs` with a companion, and four guard records couple the target
    to the four files.
  - [D] What a chosen sort costs All Inboxes at 12,872 and at 200,000 rows is on
    `docs/development/measurements.md` beside the fixed order it replaces, taken by the
    harness 10-02 built, and the two comments that said why the fixed order was cheap say
    what the others cost.
  - [S] Whether All Inboxes, a label and a saved search open in the chosen order after a
    visit to a folder, with his screen reader on the next build, is the tester's.

- [x] **FOUND-16**: A build handed to a tester carries an ordered counter in its build
  metadata and in the Windows file version, and the version proper moves only by the rule.
  - **Closed 2026-09-17 by 10-02.2, merged at `44bff634`.** The first `[D]` line by
    `tests/installer.rs` (`test_the_four_field_version_follows_the_scripts_own_table`, which
    reads the weights and caps off the script and holds `BUILD="$LAG.g$commit"` and the
    rows; `test_a_later_build_orders_above_an_earlier_one_and_below_the_next_stage`;
    `test_the_first_alpha_of_1_0_0_sits_above_the_last_0_x_version_the_way_windows_orders_it`;
    with `test_the_reading_can_see_a_fourth_field_rule_that_is_gone` as the reader's
    companion) and by `tests/house_style.rs`
    (`test_the_installer_says_how_far_back_the_version_was_set`, the refusal and
    `VERSION_SET_AT=` before `BUILD=`), the three records on the script measured; the second
    by `test_every_job_that_runs_the_tests_checks_out_the_whole_history` widened to jobs that
    run the script, with `test_the_history_reading_can_see_a_job_with_one_commit` gaining the
    case and two records on `ci.yml`, by
    `test_a_build_with_a_counter_is_the_same_version_as_the_bare_number` and
    `test_the_running_builds_shape_with_a_counter_still_reads` for `compare`, and by the
    document-reading targets over `CLAUDE.md` and the changelog. One installer was built
    from `main` at `44bff634` after the merge: `dist/Wixen-Mail-Setup-1.0.0-alpha.1+114.g44bff634.exe`,
    file version `1.0.0.14114` by PowerShell's `VersionInfo`, and `--version` on the exe it
    carries says `Wixen Mail 1.0.0-alpha.1+114.g44bff634`. The last `[S]` line stays
    (ledger 517): nothing has installed it over the alpha.1 build anybody has.
  - Evidence: `scripts/build-installer.sh` composes `FULL_VERSION="$VERSION+$BUILD"` with
    `BUILD="g$commit"` (`:29-45`) and only afterwards counts the commits since the version was
    set as `LAG` (`:67-69`), printed and not carried; the Windows file version's fourth field
    is `stage * 1000 + step` with the step held to 999 (`:91-107`), so two builds of one
    version show one file version. The tester's two builds of `1.0.0-alpha.1` are 70 and 88
    commits past `01ff57bf`, the commit that set it (`git rev-list --count`), and nothing on
    either says which is later. `version::without_build` drops everything after the first
    `+` (`version.rs:83-89`), so `1.0.0-alpha.1+42.g59c5b6a4` already compares `Same` with
    `1.0.0-alpha.1`; `parse` refuses `alpha.1.2` (`:175-176`). `build.rs:20-22` passes
    `WIXEN_BUILD` through to `version::current()`, which reaches `--version`, the log's first
    line, About, the IMAP ID and the update check. CI's Build job checks out one commit and
    runs the script on every push (`ci.yml:252-277`); the release workflow checks out the
    history (`release.yml:92`). No tag exists in the clone. `tests/installer.rs:1203-1261`
    holds the script's expression and cap by their text; `tests/house_style.rs:4744-4775`
    holds `VERSION_SET_AT` and the word unknown.
  - [S] Pratik, 2026-09-17: builds handed to testers keep `1.0.0-alpha.1` as the round's name
    and gain an ordered counter in the build metadata, `1.0.0-alpha.1+42.g59c5b6a4`, the number
    of commits since the version was set, with the same counter in the Windows file version's
    last part so Apps and Features orders the builds; declined, a counter inside the
    prerelease and a release per tester build.
  - [D] The script computes the counter before the build identifier, composes
    `counter.gcommit` after the plus, and puts `stage * 13000 + step * 1000 + counter` in the
    fourth field with the step held to 12 and the counter to 999, each cap saying so on the
    console; a clone without the commit that set the version is refused with a sentence;
    nothing is appended at a tag while the file version keeps the counter; held by
    `tests/installer.rs` reading the rule off the script and ordering builds in the order they
    are made, and by `tests/house_style.rs` reading the refusal and the order of the two
    computations.
  - [D] CI's Build job checks out the whole history, held by the same `house_style` reader
    that holds the Test Suite job; `compare` reads the new shape as the bare version, held by
    two companions; the rule in `CLAUDE.md` and the changelog's opening paragraph carry the
    counter with the date and the reason, the old shape kept and dated.
  - [S] Whether the next installer built by the script carries the counter in Apps and
    Features and installs over the alpha.1 build as an upgrade is settled by a build, which is
    Pratik's to make.

**Added 2026-09-18: two more under this section, from what the morning's push of `744d05ef`
showed, owned by phase 11 and placed here on the reasoning FOUND-13 to FOUND-16 gave.**
`FOUND-17` is a regression of FOUND-02's fix, found by CI and not by anybody; `FOUND-18` is the
workflow FOUND-09's case runs in, reporting success over a failed job. Neither is one of the
seven groups.

- [x] **FOUND-17**: The Settings screen keeps a chosen spelling language exactly as chosen
  when this machine cannot check it, and CI on `main` is green. **Ticked 2026-09-18 by 11-01,
  merged at `316ea755`, on its two `[D]` lines:** the rule is
  `presentation::which_language_row::which_row_shows`, held by
  `test_a_tag_with_a_region_is_shown_as_its_own_row_even_without_a_dictionary`,
  `test_a_tag_with_a_region_the_machine_does_not_list_is_added_as_stored`,
  `test_a_tag_with_a_region_finds_its_row_whatever_its_case`,
  `test_a_bare_tag_is_resolved_to_the_row_the_checker_would_use`,
  `test_a_bare_tag_the_checker_cannot_place_stands_as_its_own_row_when_listed` and
  `test_a_tag_nothing_offers_is_added_as_stored`, four of them red at `7c8ebda9` and all six
  green at `29e4d850`; `language_rows_and_selection` asks it at `wx_settings.rs:890` and
  `read_settings` rebuilds the same list; the integration target is unchanged and passes here;
  the region rule's record measured 2 red and nothing else on the library, the screen's
  rewritten record 1 on its target; the changelog entry names 09-02 and #21. The clause "CI on
  `main` is green" and the `[S]` line below are the runner's at the next push, ledger 530.
  - Evidence: `gh run view 35336142985 --json conclusion` -> `failure` on 2026-09-18 at
    `744d05ef`, the Test Suite job alone; its log at line 10868: `thread
    'test_the_language_the_screen_shows_is_the_one_the_checker_uses' panicked ... stored
    "en-AU": the screen would keep "en-US", the checker uses "en-AU"`; every other target
    passed (75 `test result: ok`, `--no-fail-fast` at `ci.yml:74`). The same target passes on
    this machine (`cargo test --test the_language_the_screen_shows_is_the_one_used` -> ok,
    2026-09-18), because Windows here offers en-AU. `language_rows_and_selection`
    (`wx_settings.rs:884`) asks `language_to_use` first, whose second arm answers this
    machine's region for any tag in the family (`spellcheck/mod.rs:354-380`), so the
    `or_else` that would keep a stored regional tag is never reached when the machine's
    region is offered. 19 records name `wx_settings.rs` at 0 tests and 30 name the checker's
    module, so the rule goes in a module of its own.
  - [S] `tests/the_language_the_screen_shows_is_the_one_used.rs:81-83`: "A tag somebody chose
    is kept exactly as chosen, whether or not this machine can check it."
  - [D] `presentation::which_language_row::which_row_shows(stored, machine, rows)` answers the
    row spelled as a stored tag with a region, available or not, or a row added at the end
    when none spells it; a bare tag resolves the way the checker resolves it; a tag nothing
    offers is added as stored; held by a case per rule over hand-built rows, red on both
    machines before the rule and green after.
  - [D] `language_rows_and_selection` asks that rule; `read_settings` writes the same list's
    tag; the integration reading is unchanged and passes here; a guard record measures the
    region rule's break; the changelog names the regression against 09-02 and #21.
  - [S] Whether the target passes on GitHub's runner is the next push of `main`, which is
    Pratik's to make.

- [x] **FOUND-18**: A run of the NVDA workflow in which a case failed is a failed run, its
  two failing cases are read and acted on where each was at fault, and the Accessibility
  run's walk over the five editors is read into FOUND-08. **Ticked 2026-09-18 by 11-02,
  merged at `1c0e9b0b`, on its four `[D]` lines:** `nvda.yml` without `continue-on-error`
  and with `if: always()` on the summary and the upload (`grep -c` 0 and 2), the header
  dated, `accessibility.yml` as it was with ledger 532 saying why; the settings case taking
  its mark after the settle (`waitToHearAll(nvda, [TABS[0]` gone, `node --check` clean), the
  Settings dialog untouched, the README listing all five cases (ledger 494 closed);
  `status_line::shown_and_signalled` and the three arms of `reauthorize_selected` calling it
  with `AccountNeedsAttention`, held by
  `test_an_outcome_that_is_also_an_event_is_one_notification_carrying_the_sentence`,
  `test_an_account_still_unauthorised_after_trying_again_reaches_the_earcon_channel` and
  `test_the_two_failures_on_this_screen_are_said_above_the_ordinary_run`, red at `eb2011ad`
  and green at `a697f61f`, with three records measured (3, 2 and 1 red, nothing else) and
  one re-measured; FOUND-08 and FOUND-09 ticked above with ledger 489 and 492 closed. The
  `[S]` line below is the runner's at the next push, ledger 531 and the changelog's Known
  limitations.
  - Evidence: `gh run view 35336142908 --json conclusion,jobs` -> the run `success`, its one
    job `failure`, 2026-09-18 at `744d05ef`; `.github/workflows/nvda.yml:39` `continue-on-error:
    true`, since `363358b3` on 2026-08-16, with the header saying the job is non-blocking on
    purpose. The job log: `FAIL tests/settings-tabs-read-once.test.js`, "never heard all of
    ["General"] within 15000ms. Everything NVDA said: []", at the case's first wait before any
    key, on the case's first ever run (`a7a54743`, 2026-09-16); `FAIL
    tests/account-manager-sign-in-failure.test.js`, "never heard all of ["Signing in
    failed","not one Wixen Mail can sign in to through a browser"]", and the same at
    `3e633252` (run 34973255598, 2026-09-15). The transcripts: the sign-in case ends with
    "Sign-in needs attention, Scan target" and never holds the sentence; no transcript of any
    case holds a dialog's opening announcement. The tester heard the Settings dialog speak
    correctly by hand on `1.0.0-alpha.1+149.g744d05ef` on 2026-09-18 and #33 is closed on his
    word. `wx_account_manager.rs:582-590`: the sentence through `said_and_shown` at High,
    then `a11y.signal(FeedbackEvent::AccountNeedsAttention, ..)` at Urgent, a millisecond
    apart. Run 35336142914: the five editor targets walked, "0 without a name" each.
  - [S] `CLAUDE.md`, guardrail 4: "A check nobody reads is worse than no check ... a scan
    reporting success while its scan step errored, buys false confidence."
  - [D] `nvda.yml` loses `continue-on-error`; the summary and the upload keep `if: always()`;
    the header says why with the date; `accessibility.yml` is left as it is, recorded in the
    ledger with the reason.
  - [D] The settings case takes its mark after the settle and presses Right rather than
    waiting for an opening announcement the harness never captures; the Settings dialog is
    not changed; `nvda-tests/README.md` lists every case and says what the log holds.
  - [D] The three sign-in failure arms call one helper, `status_line::shown_and_signalled`,
    which puts the sentence on the line and raises the event with the sentence as its detail,
    so the words, the cue, the braille and the visual are one notification; the reading at
    `wx_account_manager.rs:2787` is rewritten in place to hold it; a guard record measures the
    break.
  - [D] FOUND-08 is ticked on run 35336142914's walk; FOUND-09 is ticked on the tester's ear
    with the runner's case named as the harness's remaining work; ledger 489 and 492 closed.
  - [S] Whether the sign-in line is heard whole and which tab the corrected case's first
    Right reaches on a fresh profile are the runner's to show at the next push of `main`,
    which is Pratik's to make; these tests never run on a machine somebody is using.

- [x] **FOUND-19**: A shell suite the gate runs cannot act on the repository that runs it,
  whatever git environment the commit hook handed it: the suite harness clears `GIT_DIR`,
  `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_PREFIX` and `GIT_COMMON_DIR` before any suite's
  first `git`, and two cases are red if that stops.
  - Evidence: `.githooks/commit-msg:44` execs `scripts/check.sh` with git's hook environment
    intact; `check.sh:324-328` runs each `scripts/*.test.sh` as a child; `which-checks.test.sh:246-262`
    builds a repository of its own with `git -C "$scratch/a-repo"`, which changes the
    directory and not the repository once `GIT_DIR` is absolute; `which-checks.sh:197` reads
    `git diff --cached`, which under an exported `GIT_INDEX_FILE` is the hook's index wherever
    the subject runs. On 2026-09-18 a hook run from the linked worktree `wixen-mail-sweep` let
    the suite set `core.bare`, replace `core.hooksPath`, write a suite identity into
    `.git/config` and put a stray commit on `main` (`b4a4cc81`, undone by `update-ref`), and a
    partial commit from the primary worktree handed the suite a temporary index and the case
    answered `all`.
  - [S] #85, filed 2026-09-18 from the incident: "Guardrail 4's shape: a check that reads as
    testing itself while writing to the thing it was meant to leave alone."
  - [D] `scripts/shell-suite.sh` unsets the five variables after `set -uo pipefail` with the
    two incidents named; `which-checks.test.sh` gains two cases against a throwaway
    repository, one under an absolute `GIT_DIR` asserting that repository's `HEAD`, config and
    hooks path did not move, one under an exported `GIT_INDEX_FILE` asserting the file's bytes
    unchanged and the subject answering `affected`; both red before the harness change and
    named `which-checks::<description>` in the red trailer; `CLAUDE.md` says what the suites
    guarantee where the shell-suite rule is (11-06.3).
  - [S] Nothing here is a person's to settle; the runs are quoted with the real repository's
    `HEAD`, config and reflog unchanged.

- [ ] **FOUND-20**: The NVDA case for #80 is green at the next push of `main` for the reason
  its second half was written, and its next failure names its own cause from the record it
  writes. Added 2026-09-20 for phase 12's 12-01, under this section beside FOUND-17 to
  FOUND-19 on the same reasoning: a defect in what CI runs, found by a push.
  - Evidence: `gh run view 35520201976 --json headSha,conclusion` on 2026-09-20: `0ad66e48`,
    failure, one job, the failed step "Run the NVDA tests";
    `tests/a-link-opens-where-the-setting-says.test.js` failed at its line 151, the second
    `waitToHearAll`, with NVDA having said `main landmark, Where it went, link`, `document`
    and one empty phrase. The artefact `nvda-transcripts`, downloaded the same day: after
    the first Enter `otherWindowsAfterTheFirstEnter` gained "Example Domain - Profile 1 -
    Microsoft Edge", `ownWindowsAfterTheFirstEnter` still held "Scan target - headings -
    Wixen Mail", `pageWindowPutBackInFront` was `true`, so the route held and the second K
    is what failed. `grep -n 'async function activateWindow' -A 8 nvda-tests/helpers/launch-app.js`:
    `WScript.Shell.AppActivate(title)`, whose answer means found, not in front.
    `grep -n 'page.set_focus()' src/presentation/wx_app.rs`: two sites when the window opens
    and when the message comes back, no activation handler;
    `target/debug/wxWidgets/src/msw/webview_edge.cpp:1171-1175`: `OnSetFocus` calls
    `MoveFocus`, the chain that should put focus in the document on re-activation.
  - [S] The run's own words: "never heard all of ["example.org/written-out"] within 15000ms.
    Everything NVDA said: ["main landmark, Where it went, link","document",""]".
  - [D] A reading on a built page window in a test process raises a second frame over it,
    activates the page window again and asserts `GetFocus()` is Chromium's window inside the
    WebView; if red, the page window gains an activation handler that gives the WebView focus
    and the reading holds it; if green, the product is cleared by measurement (12-01, task 1).
  - [D] The case returns to the page window with Alt+Tab, waits until `GetForegroundWindow`
    is the page window (falling back to `AppActivate` once, recording which worked), records
    the UI Automation focused element and the foreground beside its spoken log after each
    return, and only then presses K; `nvda-tests/README.md` says an activation call's answer
    is not evidence a window is in front (12-01, task 2).
  - [S] The run at the next push of `main` is Pratik's; the ledger entry 12-01 writes names it.

### All the mail, and what is said while it comes

Added 2026-09-17 for phase 10, the third of the seven groups Pratik agreed on 2026-09-16.
Every requirement here is one GitHub issue from the same first day of testing, in the
tester's words on its `[S]` lines, with the `[D]` lines written on 2026-09-17 by the planner
as proposals in the sense the top of this file gives. Every evidence line was re-taken against
`main` at `7d57cd49` on 2026-09-17, after phase 9's ten plans moved most of the lines the
issues cite, and where a premise moved the evidence line says which way. The five share one
download and one progress story, which is why they are one phase; the plans are in
`.planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/README.md`.

Nothing here has met a real provider except through the tester's Gmail account, which the
download and the watch will meet unasked on the first check after the build. Each requirement's
last `[S]` line says what only that account can settle; the caveat at the top of this file
binds every `[D]` line: a loopback server proves the shape, and no criterion claims what a
provider does.

- [x] **MAIL-01**: Every message of every kept folder comes down on its own, without a
  person asking for the rest.
  - **Closed 2026-09-18 by the phase's closing read in 10-07; 10-01, merged 2026-09-17 at
    `d8e887d6`, and 10-05, merged 2026-09-18 at `b477e8c9`.** The three `[D]` lines each
    have a name. The decision, in `src/application/bringing_everything_down.rs`:
    `test_the_folder_on_screen_comes_first_then_the_inbox_then_the_tree_order`,
    `test_the_rest_follow_the_tree_order_and_custom_folders_go_by_name`,
    `test_headers_come_before_text_for_the_whole_account`,
    `test_a_folder_that_is_all_here_is_never_asked_for_headers_again` and
    `test_the_headers_chunk_is_the_syncs_own_page_by_name`, against the scripted mailbox
    through `fetch_over_a_mailbox`'s tests. The runner, in
    `tests/everything_comes_down_without_being_asked.rs`:
    `test_every_check_ends_by_starting_the_download`,
    `test_the_download_does_what_the_model_says_and_nothing_of_its_own`,
    `test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit` for the list re-read per
    chunk, `test_the_download_asks_whether_to_stop_between_chunks` and
    `test_pause_downloading_is_on_the_tools_menu_ticked_and_carries_the_warning` for the
    Pause item, `test_get_older_messages_hands_to_the_download_with_this_folder_first`,
    `test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did`, and
    `allowed::tests::test_downloading_everything_says_it_is_experimental_and_says_what_could_go_wrong`
    for the one sentence. The wait: `trying_again::tests::test_each_failure_doubles_the_wait_until_the_cap`,
    `test_the_wait_never_goes_past_thirty_minutes` and
    `test_a_success_puts_the_wait_back_to_the_start`;
    `test_a_failed_chunk_waits_before_the_download_is_tried_again` for the runner asking it;
    `mail_sync::tests::test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten`
    for a provider's refusal ending the run;
    `test_a_folder_whose_last_chunk_brought_nothing_new_is_asked_once_more_and_then_reported`
    for the rule kept; and the watch asking the same rule by
    `checking_on_a_schedule::tests::test_a_watch_whose_connection_was_lost_is_tried_again_after_a_wait`.
    The last `[S]` line stays: no provider has met the download (ledger 11, 72, 523).
  - Evidence: `grep -n 'pub const INITIAL_FETCH_LIMIT' src/application/mail_sync.rs` ->
    `40: 500` on 2026-09-17 at `7d57cd49`; `uids_to_fetch` (`mail_sync.rs:307`) takes the
    newest `limit` uids the cache lacks and `fetch_headers` (`imap.rs:1156`) asks for them by
    uid in batches of at most 1,024 characters of set. Three things start a sync
    (`grep -n 'spawn_mail_sync(' src/presentation/wx_app.rs | grep -v 'fn '` -> F9 at 4489,
    Get Older Messages at 4614, the watch's `MailboxChanged` at 18302) and each syncs the
    active account only (`wx_app.rs:21706-21712`). The whole-folder request
    (`spawn_whole_folder_fetch`, `wx_app.rs:21594`) is the runner's shape already, one
    folder at a time, with no stop, driven by `asking_for_a_whole_folder::until_the_whole_folder_is_here`.
    The body cache evicts above 512 MiB at the end of every folder sync
    (`bodies.rs:285`, `mail_sync.rs:1467`). POP already fetches everything not held, with its
    text, in one pass (`pop_sync.rs:128-140`). No retry rule exists for a server that refused
    (`grep -rn -i 'backoff\|try_again' src --include='*.rs'` -> nothing); the one reconnect is
    a single immediate sign-in (`mail_session.rs:37-41`, ledger 64).
  - [S] #20, the tester on 2026-09-15: "Right now, only 500 messages are downloaded per
    folder. All email should be downloaded."
  - [S] `src/application/asking_for_a_whole_folder.rs:10-16`, the header: "`mail_sync::INITIAL_FETCH_LIMIT`
    bounds what comes down from the server. `wx_app::FOLDER_LIST_PAGE_SIZE` bounds what is
    read out of the cache into the list. They are separate numbers that happen to be the same,
    and moving one alone appears to do nothing."
  - [D] A pure decision in `application::bringing_everything_down` says what comes next for
    one account from the cache's own facts, held messages against the server's total per
    folder and the messages with no text here, so a run picks up where it was after a
    restart with no state file; the order is the folder on screen, then the inbox, then the
    kept folders in tree order, newest first inside each; a chunk of headers is the existing
    constant by name and not a second number; tested against the scripted mailbox.
  - [D] After every check for mail the download runs for every enabled IMAP account, chunk by
    chunk on the account's own session, sends the list each chunk as it lands, and can be
    paused and carried on from Pause Downloading on the Tools menu; Shift+F9 carries it on
    with this folder first; Download This Whole Folder is gone with its warning because this
    is what it did, and one sentence on the Pause item says the download has never met a real
    provider.
  - [D] A provider that refuses, throttles or drops the connection ends the run, and the run
    is tried again after a wait that grows from 30 seconds to a cap of 30 minutes and resets
    on success, through one rule in `application::trying_again` that the inbox watch asks
    too; a folder whose chunk brought nothing new is asked once more and then reported as
    the server having stopped sending it, which is the existing rule kept.
  - [S] Whether Gmail, the one provider anything here will meet, tolerates 12,872 messages
    coming down chunk after chunk is the tester's account's to show, and ledger 72 stays open
    until it has.

- [x] **MAIL-02**: The message list holds every message the folder holds on this computer,
  and a message arriving adds a row and removes none.
  - **Closed 2026-09-18 by the phase's closing read in 10-07; 10-02, merged 2026-09-17 at
    `48536d31`.** The two `[D]` lines each have a name. The measurement: sixteen rows on
    `docs/development/measurements.md` named "The list's own read path" and "The list's own
    read path after 10-02", dated 2026-09-17 with their command at `dbcddb93` and `760a4d87`,
    held by `test_every_read_path_row_has_the_pages_shape_and_says_which_step_it_timed` in
    the harness and by `test_every_row_on_the_measurements_page_carries_its_command_its_date_and_its_commit`.
    The window, in `tests/the_list_holds_everything_the_folder_holds.rs`:
    `test_the_window_asks_for_the_whole_folder` holds `load_folder_messages` to no limit and
    the file to naming none of the three identifiers,
    `test_a_folder_of_the_testers_size_is_read_back_whole_through_the_query_the_window_uses`,
    `test_the_labels_of_a_folder_are_read_by_folder` and
    `test_a_folder_above_the_variable_limit_reads_its_labels_by_folder_without_an_error` at
    40,000 rows; and `test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit` for the
    arriving row. `grep -c 'FOLDER_LIST_PAGE_SIZE\|ALL_INBOXES_LIMIT\|message_list_limit'
    src/presentation/wx_app.rs` answers 0 on 2026-09-18 at `de0d45e2`. The last `[S]` line
    stays: whether his folder reads as one list is the tester's (ledger 515).
  - Evidence: `grep -n 'const FOLDER_LIST_PAGE_SIZE\|const ALL_INBOXES_LIMIT' src/presentation/wx_app.rs`
    -> `7061: 500`, `7050: 500` on 2026-09-17 at `7d57cd49`; `message_list_limit` starts at
    the page (`:471`), is reset to it on every folder change (`:2867`) and grown by Get Older
    Messages (`:4579`) and by a whole-folder chunk arriving (`:18320`); `load_folder_messages`
    (`:13754`) passes `Some(limit)` to `get_message_list_sorted`, which takes `Option<usize>`
    and writes no `LIMIT` for `None` (`messages.rs:2159-2171`); `unified_inbox` and
    `messages_with_label` take a bare `usize`. 08-04's harness timed `get_messages_for_folder`,
    unsorted and unbounded (`tests/the_list_at_two_hundred_thousand_rows.rs:213,231`), and not
    the sorted query, the threading or the labels the window runs on the interface thread
    (`wx_app.rs:13774-13790`, `12668`, `10281`). `get_tags_for_messages` sends one bound
    parameter per row (`tags.rs:232-235`) and the bundled SQLite (`libsqlite3-sys 0.38.1`,
    `Cargo.lock:3145`) refuses more than 32,766, so the labels read is expected to fail above
    that count, which is a prediction until 10-02 runs it.
  - [S] #24, the tester on 2026-09-15: "When new messages arrive, older messages are no
    longer available in the message list. Only 500 messages are shown."
  - [S] `docs/development/measurements.md`, rows dated 2026-09-14 at `5cf04528`: listing
    200,000 rows 351 ms cold and 371 ms warm, sorts 61 ms to 260 ms, the page paint 0.09 ms.
  - [D] The list's own read path, the sorted query with no limit, the threading and the
    labels, is measured at 12,872 rows and at 200,000 before the page is dropped and again
    after, and the rows are on the measurements page with their commands and dates.
  - [D] `load_folder_messages` passes no limit, `message_list_limit`, `FOLDER_LIST_PAGE_SIZE`
    and `ALL_INBOXES_LIMIT` are gone, All Inboxes is unbounded too, the labels of a folder are
    read by folder in one query with no parameter per row and proved at 40,000 rows, and a
    reading with companions in a target coupled to `wx_app.rs` holds the window to it.
  - [S] Whether a folder of 12,872 messages opens and reads as one list on the tester's
    machine with his screen reader is his.

- [x] **MAIL-03**: Message text comes down with the mail unless a person has forbidden it,
  and stays unless a person has chosen a size.
  - **Closed 2026-09-18 by the phase's closing read in 10-07; 10-01 at `d8e887d6`, 10-03,
    merged 2026-09-17 at `c4203632`, and 10-05 at `b477e8c9`.** The three `[D]` lines each
    have a name. The text pass: `mail_sync::tests::test_a_server_that_refuses_three_in_a_row_ends_the_chunk_after_six_asks_and_not_ten`,
    `test_a_chunk_stops_before_the_next_message_when_asked_to`,
    `test_the_server_stopping_is_worded_by_us_with_one_clause_per_reason` and
    `test_no_sentence_about_the_server_stopping_carries_a_string_the_server_sent`;
    `bodies::tests::test_a_listed_message_carries_the_size_the_server_gave_it` for the
    bound on bytes; in `bringing_everything_down`,
    `test_a_chunk_of_text_is_at_most_fifty_messages`,
    `test_a_chunk_of_text_is_at_most_sixteen_mebibytes_whichever_bound_is_met_first`,
    `test_text_is_asked_for_newest_first_once_every_folder_is_here`,
    `test_no_text_is_asked_for_when_reading_is_not_allowed_and_the_run_says_why`,
    `test_the_budget_ends_a_text_run_and_says_what_it_would_have_needed` and
    `test_the_text_report_says_what_the_budget_kept_and_what_happens_to_the_rest`; and
    `test_the_download_does_what_the_model_says_and_nothing_of_its_own` for the runner
    handing the chunk to `fetch_over_a_mailbox`. The setting, in `keeping_message_text`:
    `test_the_choices_are_all_of_it_then_one_five_and_twenty_gigabytes`,
    `test_the_default_keeps_all_of_it` and
    `test_a_garbled_value_reads_as_all_because_a_wrong_bound_evicts_and_all_loses_nothing`;
    `data::config::permission_tests::test_a_settings_file_written_before_these_existed_reads_the_way_it_should`
    for the older file; `test_every_setting_somebody_can_change_is_offered_by_a_screen`,
    which failed on arrival between the field and the control and is green;
    `test_a_size_chosen_on_the_permissions_page_is_what_ok_writes_back` against the real
    dialog; `bodies::tests::test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size`;
    and `test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts`.
    The retirements: `test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did`,
    `wx_app::tests::test_the_answer_says_what_the_missing_text_means_and_puts_no_button_on_the_screen`
    and `test_the_missing_text_sentence_says_it_is_coming_or_that_the_box_is_off`;
    `grep -rn 'ID_FETCH_MISSING_TEXT\|missing_text_offer\|FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL'
    src tests --include='*.rs'` finds one line on 2026-09-18 at `de0d45e2`, the test
    asserting the name is absent from the shipping half; the on-open fetch,
    `wx_app::spawn_body_fetch`, is unchanged since `7d57cd49` by `git log -L`. The last
    `[S]` line stays: no provider has met the text in chunks (ledger 11, 519, 523).
  - Evidence: `Allowed.reading` is on by default (`allowed.rs:57-82`) and offered as
    "Fetch the text of a message from the server when it is not already stored" under
    Message Text on the Permissions tab (`allowed.rs:206,224`; `wx_settings.rs:2095-2118`),
    read at `read_the_permissions_page` (`wx_settings.rs:3174-3179`). The text pass
    (`fetch_over_a_mailbox`, `mail_sync.rs:1858-1930`) asks one message at a time for every
    message in the account with no chunk bound and no stop but the reading gate, counting a
    refusal per message and never reading a run of refusals as the server's answer; it says at
    most ten progress lines (`AT_MOST_THIS_MANY_PROGRESS_LINES`, `:1739`). Two entry points,
    the menu item `ID_FETCH_MISSING_TEXT` (`wx_app.rs:4499`) and the offer button above the list
    (`:1925`), and five tests inside `wx_app.rs` read them (`:30080-30230`). The sizes arrive
    with the headers and are stored (`imap.rs:2046`, `mail_sync.rs:579`). `BODY_CACHE_BUDGET_BYTES`
    is a constant whose doc comment says a setting would plug into `keeping_bodies_under`
    (`bodies.rs:276-285`, `mod.rs:1528`).
  - [S] #23, the tester on 2026-09-15: "Unless explicitly forbidden, message text should be
    downloaded along with mail."
  - [S] `src/application/allowed.rs:363-369`, `FETCHING_TEXT_IN_BULK_IS_EXPERIMENTAL`: "Asking
    your provider for hundreds of whole messages one after another is something they are
    entitled to refuse, to slow down, or to disconnect you for, and nothing here can find out
    which yours will do."
  - [D] The text pass asks in chunks of at most 50 messages or 16 MiB, ends a chunk after three
    refusals in a row and says the server stopped answering, stops between messages when
    asked, and answers how it ended; the download asks for text only when the Message Text
    box is on and only while the size chosen has not been reached, newest first, and says
    how much is kept and how many older messages will be fetched when opened.
  - [D] How much message text stays on this computer is a choice under Message Text on the
    Permissions tab, All of it by default and three sizes, stored as a string with a default
    that an older settings file falls back to, read by the eviction, which evicts nothing
    under All, and handed to every cache the sync workers open;
    `test_every_setting_somebody_can_change_is_offered_by_a_screen` is what fails on arrival.
  - [D] Fetch Missing Message Text and the offer above the list are gone, because the download
    does what they did and under a chosen size the offer would fetch beyond it; the text of a
    message opened before its turn is fetched on opening as before.
  - [S] Whether Gmail tolerates the text of 12,872 messages coming down in chunks is the
    tester's account's to show, and ledger 11 stays open until it has.

- [x] **MAIL-04**: Mail keeps arriving on its own for as long as the program runs.
  - **Closed 2026-09-18 by the phase's closing read in 10-07; 10-01 at `d8e887d6` and
    10-06, merged 2026-09-18 at `2da50b6b`.** The three `[D]` lines each have a name. The
    restart, in `tests/mail_keeps_arriving_on_its_own.rs`:
    `test_a_watch_that_ends_asks_whether_to_watch_again`,
    `test_a_watch_that_never_started_asks_too_before_it_returns` and
    `test_the_network_coming_back_starts_the_watches_at_once`; in
    `checking_on_a_schedule`, `test_a_watch_whose_connection_was_lost_is_tried_again_after_a_wait`,
    `test_a_watch_somebody_stopped_is_not_tried_again` and
    `test_every_reason_the_watch_can_end_with_has_an_answer_that_is_not_a_guess`, one
    answer per reason string the IMAP module gives; the watch per account by
    `test_the_check_for_mail_walks_every_enabled_account_and_writes_down_when`, which holds
    each account's own watch request. The schedule:
    `test_the_timer_checks_on_the_accounts_own_interval`,
    `test_an_account_whose_interval_has_passed_is_due`,
    `test_a_stored_interval_of_nought_is_read_as_one_minute_and_not_every_tick`,
    `test_a_server_that_would_not_start_watching_three_times_is_left_to_the_schedule` and
    `test_an_account_to_check_is_built_from_the_account_row`;
    `test_a_start_checks_without_a_keystroke`; `mark_synced` called at the check's end,
    held by the same walk reading; the editor's sentence `WHAT_THE_INTERVAL_DOES` at
    `wx_account_manager.rs:966`, built on the field at `:1639`. The status line:
    `test_the_status_line_says_watching_and_the_interval`,
    `test_the_status_line_says_only_the_interval_when_nothing_watches`,
    `test_the_status_line_says_the_wait_and_the_interval` and
    `test_nothing_says_new_mail_will_not_appear_on_its_own`, with `grep -c 'will not appear
    on its own\|say_the_watch_is_off' src/presentation/wx_app.rs` at 0 on 2026-09-18 at
    `de0d45e2`; the cold-start and idle rows re-taken on `docs/development/measurements.md`
    dated 2026-09-18 at `7ad596ca`, the old rows dated, and ledger 446 closed
    on the refusal being read, which was the credential store's and not the socket's (ledger
    526). The last `[S]` line stays: no provider has dropped the watch (ledger 64, 65, 67,
    525).
  - Evidence: one watch, on the active account's inbox, started only at the end of a check
    (`grep -n 'MailboxWatchRequested' src/presentation/wx_app.rs` -> `18288` the arm, `21975`
    the one sender, on 2026-09-17 at `7d57cd49`); nothing checks at startup (the three
    starters under MAIL-01); on `ImapIdleEvent::Stopped` the arm says "New mail will not
    appear on its own. Use Refresh to check for it." and breaks, under the comment "Nothing
    starts another watch from here" (`wx_app.rs:20126-20131`); the IDLE window renews itself
    (`imap.rs:1723,1793-1815,1856`), so only a real end reaches the arm and a dead socket is
    noticed at the next window at the latest. `act_on_what_the_network_did`
    (`wx_app.rs:10228`) sends a sentence and an offer on the network's return and starts
    nothing; offline mode is asked only by the send paths (`reachability_of`, `:14705`).
    `Account.check_interval_minutes` is stored (`accounts.rs:26,46,117`), built and read on
    the account editor (`wx_account_manager.rs:1244,1329,1624,1830`, "Check &Interval (min):",
    default 5, clamped 1 to 60) and read by nothing that checks mail
    (`grep -rn 'check_interval' src/presentation/wx_app.rs src/application/*.rs` -> nothing);
    `Account::mark_synced` (`account.rs:326`) is called by its own test only. The main timer's
    arms run on stated intervals (`wx_app.rs:5560-5720`; the network every 10 s, the due look
    every 60 s). The doc comments for `spawn_mail_watch` and `say_the_watch_is_off` sit above
    `fn say_the_link_was_refused` (`wx_app.rs:19985-20019`).
  - [S] #37, the tester on 2026-09-15: "After running a few hours, automatic mail fetching is
    switched off. The user has to manually fetch mail."
  - [S] `src/presentation/wx_app.rs:20126-20131`: "Nothing starts another watch from here. A
    fresh one begins only after mail arrives and the folder is read again, and mail arriving
    on its own is exactly what has stopped."
  - [D] A watch that ends for any reason but mail arriving or somebody stopping it is started
    again after the wait rule of MAIL-01's third `[D]` line, the network coming back starts
    one at once and a check for every due account, every enabled IMAP account has a watch of
    its own, and the decision is a pure function over the reason the IMAP module gives, tested
    per reason.
  - [D] Where a watch cannot cover, a server without IDLE or one that refused the watch three
    times, a POP account, and the kept folders that are not the inbox, mail is checked on a
    schedule whose interval is the account editor's Check Interval made true, from an arm on
    the main timer; a start checks every enabled account once without a keystroke; the check
    calls `mark_synced`; and the account editor says under the field what it does.
  - [D] The status line says which of watching, checking every N minutes, or waiting to try
    again is true, and the sentence that new mail will not appear on its own exists nowhere;
    the cold-start and idle rows are re-taken on a start that dials the harness's refusable
    account once, and ledger 446 closes on the refusal being read.
  - [S] Whether Gmail drops an IDLE connection, after how long, whether the restart carries
    mail over hours, and whether two connections per account are welcome are the tester's
    account's to show (ledger 64, 65, 67).

- [x] **MAIL-05**: How much is said while mail and the other modules are fetched is the
  person's choice, and by default only what arrived is said.
  - **Closed 2026-09-18 by the phase's closing read in 10-07; 10-04, merged 2026-09-18 at
    `19a10706`.** The three `[D]` lines each have a name. The choice, in
    `what_is_said_while_fetching`: `test_the_choices_are_what_arrived_then_every_step_then_errors_only`,
    `test_the_default_says_what_arrived` and
    `test_anything_unreadable_reads_as_what_arrived_because_the_other_two_cost_more`;
    `data::config::permission_tests::test_a_settings_file_written_before_these_existed_reads_the_way_it_should`
    for the older file; `test_every_setting_somebody_can_change_is_offered_by_a_screen`,
    which failed on arrival and is green;
    `accessibility::tests::test_how_much_to_say_reports_what_was_just_set_rather_than_the_default`
    for the level on `Accessibility`; and `tests/every_event_has_a_control.rs`'s sub-check
    that each level chosen on the built dialog is what OK writes back. The kinds, in
    `tests/progress_is_shown_and_results_are_said.rs`:
    `test_a_step_is_spoken_only_when_every_step_was_asked_for`,
    `test_what_arrived_is_said_once_at_normal_and_signals_new_mail`,
    `test_the_mail_checks_lines_are_steps_and_what_arrived_goes_out_once_after_the_loop`,
    `test_the_new_mail_sound_plays_when_a_check_found_mail_and_not_when_the_watch_woke`
    and `test_no_step_rides_the_answer_channel` over every sync path's lines, with
    companions; `mail_sync::tests::test_a_check_that_found_nothing_says_nothing_about_what_arrived`;
    the twelve cells `test_under_what_arrived_a_step_is_not_spoken` through
    `test_under_errors_only_an_answer_is_spoken` for errors under every level. The answers:
    `test_settings_saved_and_the_other_answers_are_said_at_normal`. The last `[S]` line
    stays: nobody has listened to the three levels (ledger 521; items 42, 43 and 48 on
    `docs/manual-accessibility-pass.md`).
  - Evidence: the `StatusUpdated` arm writes the status bar and announces at Low under the
    topic "status" (`wx_app.rs:17576-17590` on 2026-09-17 at `7d57cd49`); the queue keeps one
    entry per topic and four a second (`announcements.rs:20-26,178-190`); 43 `StatusUpdated`
    sends in `wx_app.rs` and 116 `send_status` calls under `src/`; a mail check writes
    Connecting, the folder count, "Checking X..." and the folder's sentence per folder, and
    "Mail check finished" (`wx_app.rs:21767,21811,21912,21941,21967`); the modules write
    "sync requested" and "Syncing tasks..." (`:4172-4184,5062-5129`); "Settings saved" rides
    the same channel (`:16928`); `FeedbackEvent::NewMail` fires in the `MailboxChanged` arm
    before the folder is re-read, whether or not anything arrives, and never on a check
    started by F9 (`:18299`); the two module completions signal `SyncComplete` with the sync's
    sentence as the detail (`:17930,18012`); the routing settings live on `Accessibility`
    behind a mutex, set at startup and on save (`accessibility.rs:279`; `handle_settings`).
    A test inside `wx_app.rs` already tells two progress openings from refusals
    (`PROGRESS`, `:26288`) and another refuses an arm that shows and never says without a
    reason (`:25811`).
  - [S] #38, the tester on 2026-09-15: "When fetching mail and other items, the announcements
    are too verbose. Only folders and items with new mail or items should be announced." And
    the same day: "Or make this user-configurable. Let the user decide how much to announce
    while fetching items."
  - [S] `src/presentation/wx_app.rs:17581-17584`, the handler's comment: "everything written
    there was written to nobody."
  - [D] A choice under a While fetching section on the Feedback tab offers Say what arrived,
    Say every step and Errors only, default Say what arrived, stored as a string with a
    default an older settings file falls back to, held on `Accessibility` beside the feedback
    settings, set at startup and on save; `test_every_setting_somebody_can_change_is_offered_by_a_screen`
    is what fails on arrival.
  - [D] A progress line is its own update kind, shown on the status bar and spoken only under
    Say every step; what arrived is one sentence per check naming each folder or module with
    something new and its count, spoken once at Normal under the two levels that say results
    and never when nothing arrived; an error is spoken under every level; the new-mail sound
    fires once at the end of a check that found mail on the channel its own per-event row
    gives; and a reading in a target coupled to `wx_app.rs` holds every sync path's lines to
    one kind, with companions.
  - [D] What remains on the status update, the answers to a key such as Settings saved, Draft
    saved and Refreshed, is announced at Normal so it is heard above a running check.
  - [S] What a check of the tester's 50 folders sounds like under each level, and which
    sentence is a step and which a result by ear, is a listening pass and his (ledger 10, 73,
    78).

### Reading, and the list

Added 2026-09-18 for phase 11, the fourth of the seven groups Pratik agreed on 2026-09-16,
with #70 from the second day of testing first and #71 from his decision of 2026-09-17. Every
requirement here is one GitHub issue, in the tester's words on its `[S]` lines, with the
`[D]` lines written on 2026-09-18 by the planner as proposals in the sense the top of this
file gives. Every evidence line was re-taken against `main` at `744d05ef` on 2026-09-18, and
where a premise moved the evidence line says which way. The plans are in
`.planning/phases/11-reading-and-the-list/README.md`.

Nothing here has met a real provider except through the tester's Gmail account. Each
requirement's last `[S]` line says what only his ear, his reader or his account can settle;
the caveat at the top of this file binds every `[D]` line.

**Read clause by clause on 2026-09-20 by the phase's closing read, 11-12, against `main` at
`1993b56e`.** Each ticked requirement was ticked by its own plan on the orchestrator's
instruction, which overruled the README's "11-12 ticks"; this read re-took every name.
Every test the twenty-eight summaries' coverage blocks name was checked to exist in its file
with `grep -n 'fn <name>'`: 449 names found, one not, 11-05's
`test_reading_a_row_aloud_records_when_reading_began`, which 11-05.1 split into
`test_the_short_form_records_nothing_about_reading` and
`test_the_whole_reading_records_when_reading_began`, both found, as LIST-03's amended line
says. So the twenty-four ticks stand: LIST-01, 03 to 10, 12 to 18 and 20 to 27, each on the
names its own paragraph gives. Three stay open, each for a reason on its own lines: LIST-02
owes the two size rows (ledger 537); LIST-11 is 11-13's, and LIST-19's second half is
11-11.2's, and both plans were deferred to the front of the next phase on Pratik's decision
of 2026-09-20 under his token budget, so neither box can be ticked here. The `[S]` lines are
untouched throughout and their ledger numbers, 533 to 565, are items 58 to 83 on
`docs/manual-accessibility-pass.md`.

- [x] **LIST-01**: Folders to Keep Up to Date is a tree whose ticks a screen reader hears,
  names the account, shows All Mail when the server lists it, and is on the Tools menu.
  **Ticked 2026-09-18 by 11-03, merged at `70f4737b`, on its two `[D]` lines:** a
  `TreeCtrl` with `TVS_CHECKBOXES` nested by `folder_parents`, the state read from
  `TVM_GETITEMSTATE` at OK, the title the account's name, the sentence for Gmail with no
  listed folder holding every message, held by
  `tests/a_kept_folder_reads_as_a_checked_check_box.rs` (readings B, C and D and the title's
  source reading, 8 tests) and 21 in the module; the item on Tools as `Alt+L`, the three
  pages and the sentence dated. The reading over the old rows found the cause of
  "read-only": wxdragon's state constants arrive at wxWidgets renumbered, so CHECKED was
  BUSY and SELECTABLE was READONLY (ledger 534). The `[S]` lines are untouched and are
  ledger 533; nobody has heard the tree.
  - Evidence: `src/presentation/wx_folder_choice.rs:159-168` on 2026-09-18 at `744d05ef`: one
    `CheckListBox`, one row per folder in stored order, and the header's reason ("the folder
    tree in the main window is one flat level") is no longer so (`folder_tree::nested` at
    `:1074`, `folder_parents` at `folders.rs:661`). Each row answers as a check box through
    `set_accessible_checked_rows` (`names.rs:232-262`), never read-only. The title is built
    from `account_id` (`wx_app.rs:19373`, `ask(frame, &account_id, &rows)`). The item is on
    Action, This Folder (`wx_app.rs:6584` inside the `folder_menu` builder appended to the
    Action menu at `:6815`), not File as the issue and three pages say; it left File on
    2026-08-26 (`d9503015`). Read through Python on 2026-09-18, read-only: his 50 stored
    folders hold no `[Gmail]/All Mail` and none flagged `holds_all_mail`; `store_folders`
    (`mail_sync.rs:694-718`) saves every folder the server lists, so his server did not list
    it, which Gmail's Show in IMAP setting decides. wxdragon 0.9.17's tree has no check state.
  - [S] #70, the tester on 2026-09-17: "The list is flat. It should follow the folder
    hierarchy"; "heard as 'check box, read-only, not checked'. It should be heard as checked,
    and not read-only"; "The title ... should carry the account's name"; "A Gmail account
    should show its All Mail folder ... unticked by default"; and his comment: "the command
    should be on the Tools menu."
  - [D] A reading over a built dialog records what each row answers today over MSAA and what a
    native tree with `TVS_CHECKBOXES` answers through `TVM_GETITEMSTATE` and MSAA, which is
    what NVDA reads for a tree item (`sysTreeView32.py`), and the dialog becomes a
    `TreeCtrl` nested by `folder_parents` with a check state per folder read back from the
    control at OK, never from a snapshot; Space toggles; the title carries the account's
    name; a listed `\All` folder is a row unticked unless chosen, and for a Gmail account with
    none listed one sentence says where Gmail decides it.
  - [D] The item moves from Action, This Folder to Tools with a letter nothing there claims;
    `docs/PROVIDER_SETUP.md`, `docs/KEYBOARD_SHORTCUTS.md` and the Blocked Senders sentence
    say Tools, dated, with the fact that they said File while the command sat on Action.
  - [S] Whether NVDA says "check box, checked" on a kept folder, the level on a nested one and
    the new state on Space, and whether his own Gmail lists All Mail, are his.

- [ ] **LIST-02**: The log level's default follows the version the build carries, the lines
  a tester's report needs are written at that level, and the alpha page says what each level
  writes and what the default costs.
  **Held 2026-09-18 by 11-04, merged at `03513fd0`, on its first two `[D]` lines and the
  third but for its two rows:** `version::is_alpha_or_beta` and `logging::default_level_for`
  are one rule read by `LoggerConfig::default()` and by `AppConfig`'s default and serde
  default, `filter_for` names this crate alone, held by 27 and 10 tests in the two modules
  and five guard records; the per-folder, per-chunk, server-answer, settings-save and
  held-back lines are written at info and debug, held by
  `tests/the_log_carries_what_a_report_needs.rs` (five readings, five companions, the
  lexical guard over every `tracing::` call and its two parsing tests, 13 tests, two
  records); the table, the two sentences and the privacy sentence are on the pages. Not
  held: the two size rows on the measurements page, because the measurement starts a
  second copy and the tester's copy was open all afternoon (ledger 537); the harness now
  refuses to start while any copy runs and pins the level, and the guide says the cost is
  owed. The box is ticked when the rows are on the page. The `[S]` line is untouched and
  is ledger 535. **Read again 2026-09-20 by 11-12: the rows are still not on the page
  (`grep -c 'two-minute start' docs/development/measurements.md` answers 0), so the box
  stays open on that clause alone; the other two lines' names were found in the tree.**
  - Evidence: `grep -n 'log_level: "info"' src/data/config.rs` -> `703`, and `LoggerConfig::default`
    at `logging.rs:62`, two literals; the field has no serde default (`config.rs:47`);
    `main.rs:100-106` reads the stored level first. Counts on 2026-09-18: `error` 59, `warn`
    215, `info` 77, `debug` 12, `trace` 0 sites. Of the five things #71 lists, one is written
    (the watch's end, 10-06); nothing per folder of a check, nothing per chunk of the download,
    nothing on a settings save, nothing where content is muted or a line dropped
    (`grep -n 'tracing::' src/presentation/wx_app.rs | awk -F: '$1>21860 && $1<22130'`;
    `accessibility.rs:426,429`). The filter is `wixen_mail=<level>` (`logging.rs:102`). His
    profile holds `info`, read through Python on 2026-09-18.
  - [S] #71, Pratik's decision of 2026-09-17: "the default is `debug` while the version
    carries an alpha or beta suffix, and `info` for a release without one (release candidates
    included) ... The default follows the version the build carries, so no hand change is
    owed at the cut ... A profile that already holds a level keeps it; the guide says how to
    move it."
  - [D] `version::is_alpha_or_beta` and `logging::default_level_for(version)` give one rule
    read by `LoggerConfig::default()` and by `AppConfig`'s default and serde default;
    `filter_for(level)` names this crate alone, with a test that refuses a second target.
  - [D] Each check's result per folder, each chunk of the download (at `debug`) and what the
    server answered, the settings save, and an announcement held back (muted, dropped for
    capacity or a repeat) are written, none naming a subject, a body, a password or a token;
    a reading target holds each line's presence and level, and a guard over every `tracing::`
    call under `src/` refuses those identifiers.
  - [D] `docs/ALPHA_TESTING.md` has a table of the five levels in the Advanced tab's words,
    what each writes, why the alpha default is Debug, that a profile keeps its level and where
    to move it, and the log's size after the harness's two-minute start under `info` and
    `debug`, two rows on the measurements page.
  - [S] Whether the lines are the ones a report needs, and what a day at Debug costs on his
    disk, are his; #64's dialog attaching the log is #64's.

- [x] **LIST-03**: Moving through the message list never marks a message read; a message is
  marked read only after it has been read aloud from the list or opened, and then after the
  delay the setting names.
  **Ticked 2026-09-18 by 11-05, merged at `5c82f680`, on its `[D]` lines:**
  `reading_habits::whether_to_mark_read(began, selected_unread, now, setting)` answers
  nothing when nothing began, whatever is selected and however long ago; nothing when the
  message that began reading is not the selected unread one; at once under Immediately,
  once the wait has run under a wait, never under Only when I say so; held by six cases in
  the module (25 tests) and a record measured on the library. `WxUIState::reading_began`
  is written by the mail read-aloud closure for the row Space or Shift+Space is about to
  read and by `open_single_message` for the message it opens, by nothing in the selection
  handler; the timer's `mark_what_was_read` asks the rule and does the write it always did;
  the old `opened_at` clock is gone; held by `tests/moving_through_the_list_marks_nothing_read.rs`
  (five readings and six companions, 11 tests) and two records on `wx_app.rs`; the sentence
  under the choice on the Reading tab, `WHAT_MARK_READ_COUNTS_FROM`, read on the built page
  by `tests/the_settings_dialog_opens_in.rs` with a record; the guide's "When a message
  counts as read"; the changelog entry naming #25. The `[S]` line is untouched and is
  ledger 539; nobody has walked an inbox by ear against this, and whether "previewed" means
  reading aloud from the list is asked of the tester in the close comment.
  **Reopened 2026-09-18 on the tester's answer (#25's last comment): reading the snippet is
  not reading.** At `5c82f680` the mail read-aloud closure writes `reading_began` before
  `SpaceCycle` has decided which form the press reads (`wx_app.rs:3475`), so the first Space,
  the short form, started the clock. The second `[D]` line below is amended for 11-05.1,
  which moves the record to where the depth is known; the tick stands for the rest and
  11-05.1's summary dates the amendment when it merges.
  **Held 2026-09-18 by 11-05.1, merged at `b3ab5d51`, on the amended `[D]` line:**
  `read_aloud::what_a_press_starts(depth) -> WhatBegan` answers `Nothing` for `Short` and
  `TheWholeReading` for `Full`, two cases in the module (49 to 51 tests); `wire_read_aloud`
  takes a second closure and calls it once the depth is known, only under `TheWholeReading`;
  the mail wiring writes `reading_began` in that closure and its lookup closure writes
  nothing; `open_single_message` is unchanged; held by `test_a_first_space_marks_nothing_and_a_second_does`
  over the real cycle and the real decision, by the two readings the old one was split into,
  `test_the_short_form_records_nothing_about_reading` and
  `test_the_whole_reading_records_when_reading_began` (13 tests in the target), and by three
  records measured on 2026-09-18; the sentence under the choice, the guide and the changelog
  say reading the whole message or opening it. The `[S]` line is untouched and is ledger 539,
  amended: whether the first Space leaves the count alone and the second moves it is his ear.
  - Evidence: `mark_the_open_one_read` (`wx_app.rs:10035`, polled from the main timer at
    `:5576`) starts a clock when the selected message is unread and marks it read when
    `mark_read_after`'s delay passes with the row still selected (`:10046-10098`); the
    default is `After(2)` (`reading_habits.rs:118-127`). The preview pane cannot take focus
    by design (`wx_app.rs:1339`, `set_can_focus(false)`, and `panes.rs:21`), so "entering
    the reading pane" has no act here; reading aloud with Space through `wire_read_aloud`
    (`:3433`) and opening with Enter through `open_single_message` (`:12447`) are the acts,
    and opening marks nothing today. His profile holds `mark_read_after` `never`, read
    2026-09-18.
  - [S] #25, the tester on 2026-09-15: "Automatic read/unread status should not be linked to
    the list traversal for mail. It should be either when a message is previewed or when a
    message is opened."
  - [D] `reading_habits::whether_to_mark_read(began, selected_unread, now, setting)` answers
    nothing without a reading having begun, nothing for a message other than the selected
    unread one, at once under Immediately, after the delay under After, never under Never;
    held by a case per answer.
  - [D] The mail read-aloud closure and `open_single_message` record when reading began; the
    selection handler records nothing; the timer's mark asks the rule and writes what it
    wrote before; a reading target holds the three sites with companions; the choice on the
    Reading tab gains a sentence saying what it counts from; the guide says when a message
    counts as read. **Amended 2026-09-18 for 11-05.1 (#25 reopened):** the record is written
    only when the press reads the whole item, decided by `read_aloud::what_a_press_starts`
    over the depth `SpaceCycle` chose, inside a closure `wire_read_aloud` calls after the
    depth is known; the first Space, the short form, records nothing; opening is unchanged;
    the reading target gains the case that a first Space marks nothing and a second does,
    and its read-aloud reading is split to ask where the write is; the sentence under the
    choice and the guide say reading the whole message or opening it.
  - [S] Whether the unread count survives a walk through his inbox by ear, and whether Space
    then the delay moves it, are his; from 11-05.1, whether the first Space leaves the count
    alone and the second moves it.

- [x] **LIST-04**: Mark as Read says which way it will go on the Action menu, the context
  menu and the toolbar, M toggles it in the message list and says read or unread, and a
  conversation row marks the whole thread.
  **Held 2026-09-19 on its third `[D]` line by 11-07, merged at `b35a40cd`:**
  `toggle_read_state` reads the selection through `chosen_messages`, a conversation row
  contributing every message of the conversation under `AConversationReaches::TheWholeAccount`
  through `messages_in_conversation`, marks them read when any is unread and says one
  sentence, `what_was_done`, "1 conversation, 5 messages marked read"; the cases in
  `choosing_messages` hold the reach and the sentence, and
  `tests/every_command_acts_on_the_selection.rs` holds the toggle to the set. The `[S]`
  lines are untouched, ledger 540 and 544; 11-12 reads the lines again.
  **Held 2026-09-18 by 11-06, merged at `fe143d46`, on its first two `[D]` lines:**
  `marking_read::what_the_command_says(any_unread)`
  answers the menu word, the context entry, the spoken form and the help, and
  `what_the_key_says(now_read)` the one word, five cases; `refresh_mark_read_wording` sets
  the item and its help through `find_item_and_menu`, the tool through
  `toolbar_text::relabel` (`TB_SETBUTTONINFOW`) and its tip, from the selection handler,
  the toggle and the `MessageReadToggled` arm; the context menu is built from
  `entries_for_messages(any_unread)` at the key; `list_keys::wire_letter` consumes M with
  `skip(false)`; `tests/mark_as_read_says_which_way_it_will_go.rs` reads the letter on a
  built list (the selection stays on row 0 where the search would have moved it to Mango),
  the relabel through `TB_GETBUTTONTEXTW` and `get_accName`, and the three refresh sites.
  - Evidence: one id on three surfaces with one fixed label: `ID_MARK_READ` at
    `wx_app.rs:886` (the toolbar, "Mark Read"), `:6725` ("Mark as R&ead"), and
    `context_menu.rs:296` ("&Mark as read", a static slice); the arm at `:4859` toggles and
    announces which way it went. No key: `docs/KEYBOARD_SHORTCUTS.md:538` "(no shortcut)".
    wxdragon 0.9.17 relabels a menu item (`menuitem.rs:145`) and not a tool (`toolbar.rs`
    offers `set_tool_short_help` only). No key on any list is consumed today
    (`grep -n 'skip(false)' src/presentation/wx_app.rs` -> nothing); `wire_read_aloud`
    skips Space in every path. The thread case rests on the selection LIST-05 builds.
  - [S] #27, the tester on 2026-09-15: "the command should say 'mark as read' and vice versa.
    Use 'm' bound to message lists to toggle the state and announcement, 'read'/'unread'. If a
    thread has the focus, then the entire thread should be marked."
  - [D] `marking_read::what_the_command_says(any_unread)` and `what_the_key_says(now_read)`
    are the words; the label is refreshed on selection, after the arm and on the toggled
    update, on the menu item through `set_label`, on the context menu through a second
    entry list, and on the toolbar through `TB_SETBUTTONINFOW`, read back over MSAA in a
    reading.
  - [D] `list_keys::wire_letter` binds M on the list and does not skip it, so the control's
    type-to-search never gets it, shown by a reading that sends `WM_KEYDOWN` and `WM_CHAR`
    to a real list and finds the selection unmoved; the key announces the one word.
  - [D] A selected conversation row contributes every message in the conversation to Mark
    as Read, with one announcement saying how many (with LIST-05).
  - [S] The label heard on each surface after arrowing between a read and an unread message,
    the word after M, and that M does not jump the list, are his ear's.

- [x] **LIST-05**: Shift with the arrow keys selects more than one message, every command
  that acts on messages acts on the selection with one announcement saying how many, and a
  conversation row contributes its messages.
  **Held 2026-09-19 by 11-07, merged at `b35a40cd`, on both `[D]` lines; the `[S]` lines
  are untouched, ledger 544, and 11-12 reads the lines again:** the first line by the 18
  cases of `application::choosing_messages` (the rows' order with each message once, the
  conversation row counted and its reach per command, the sentence with the singular right
  and the conversations named, the bound read from `editing.rs`), two records on the
  library; the second by `tests/every_command_acts_on_the_selection.rs`, whose source
  readings hold the list built without `SingleSel`, the Star arm, `toggle_read_state`,
  `label_the_message`, the Delete arm and `move_or_copy_message` to `chosen_messages(`,
  `too_many(` and the one sentence, `spawn_folder_move` to the summed sentence, the six
  cursor commands and the Copy to arm to `selected_message_index` alone, the cursor
  handler to the focus event, the words to the selected rows and the removal path to one
  landing after the set; whose built list measured the focus and selection events and the
  walk; and whose ignored timing is the 75 ms row on `docs/development/measurements.md`,
  dated 2026-09-19 at `9155c6be`; three records on the target. The cursor handler moved
  to the focus event and Reply, Forward, Open and Save As read it, 11-07's deviation 1.
  - Evidence: `ListCtrlStyle::SingleSel` at `wx_app.rs:1108`; 27 sites read
    `selected_message_index` in the shipping half (`grep -c` 28, one in a test), sorted in
    11-07's premise into seven set commands, eleven cursor commands and the bookkeeping;
    `Doing::ChooseEveryRow` (`:16515`) already sets Selected on every row and refuses above
    `MOST_ROWS_WORTH_SELECTING`, 5,000 (`editing.rs:86`), and on a single-selection list
    selects one; `conversation_nodes` (`:12648`) is the walk over a conversation's messages;
    `mail_across_accounts` reports one move at a time (`:603`, `:713`).
  - [S] #30, the tester on 2026-09-15: "Shift+arrow keys should allow the user to select
    multiple messages."
  - [D] `choosing_messages::what_the_selection_holds` turns the control's selected rows into
    a set with duplicates removed, a conversation row contributing the whole conversation to
    Mark as Read, Star and Label, this folder's messages to Move and Copy, and its setting's
    reach to Delete; `what_was_done` words one sentence with the count; `too_many` refuses
    above the bound Select All uses.
  - [D] The list is built without single selection; Delete, Delete Permanently, Move to, Copy
    to, Mark as Read or Unread, Star or Unstar and the Label commands read the set from the
    control at the command and act per message with one sentence; Reply, Forward, Open, Save
    As, the receipt and Copy to a task, event or note act on the cursor row; a cross-account
    batch sums its report; a reading target holds every arm; marking 5,000 read in the cache
    is a row on the measurements page.
  - [S] NVDA's own selected and not selected as he extends, the count after Ctrl+A, one
    sentence after a command over many, and the refusal above the bound, are his ear's.

- [x] **LIST-06**: A conversation row stands for the originator when every message in it is
  unread and for the first unread message otherwise, reports that message's sender first,
  previews and opens on it, and selecting it fetches the conversation's text.
  - Ticked 2026-09-19 by 11-08 at `75c211fe` on its `[D]` lines, the orchestrator's
    instruction overruling the README's "11-12 ticks"; the `[S]` line is ledger 548.
  - Evidence: every conversation column is one SQL expression used for the cell and the sort
    (`message_columns.rs:208-250`, `messages.rs:258-330`): Correspondent is every distinct
    sender in stored order, Snippet the newest message's, and nothing names a message the row
    stands for (`conversations.rs:190-245`). The conversation window selects the root
    (`wx_thread_view.rs:300`). The selection handler has no conversation branch
    (`grep -n 'showing_conversations()' src/presentation/wx_app.rs | awk -F: '$1>3000 && $1<3100'`
    -> nothing) and previews `messages[idx]` (`:3040-3043`), a message unrelated to the
    conversation at row `idx`. Nothing fetches a conversation's text on selection; since
    10-05 the download brings text for kept folders, so what remains is the order, a folder
    not kept, and a mailbox under a chosen size.
  - [S] #31, the tester on 2026-09-16: "Focusing on a thread in the mail list should use the
    originator of the thread ... if all the messages in the thread are unread. If not, then
    the first unread message should be highlighted with the corresponding correspondent.
    Currently, the last message is highlighted. If a thread is highlighted, all messages
    should be cached."
  - [D] One correlated subquery ordered by `read ASC, received_at ASC, id ASC` chooses the
    row's message where the columns are, so the first unread by arrival when any is unread
    and the originator otherwise; its id, uid and sender ride the row; the Correspondent and
    Snippet expressions follow it, so the cell, the sort, the preview and the window agree;
    the correspondent cell says that sender first and the rest after; held by fixtures
    through the real query.
  - [D] The selection handler under conversation view previews the row's message and fetches
    the conversation's missing text as one chunk under the reading gate and the runner's
    bound, that message first, saying nothing per message; the window opens on that message's
    node; a reading target holds the branch, the fetch and the window.
  - [S] The sender heard first on his thread rows, the window's opening node, and the text of
    a conversation arriving from Gmail on selection are his ear's and his account's.

- [x] **LIST-07**: Ctrl+Shift+; reads the selected row column by column with its headings on
  request, and the pages say why the headers are spoken on every row and how to quiet them.
  - Ticked 2026-09-19 by 11-09 at `bd5f6929` on its `[D]` lines, the orchestrator's
    instruction overruling the README's "11-12 ticks"; the `[S]` line is ledger 550.
  - Evidence: NVDA's `sysListView32.py`, read 2026-09-18: the header is spoken before every
    column but the first when `documentFormatting.reportTableHeaders` is rows-and-columns or
    columns, the default, and no property an application sets changes it; `message_rows.rs:66-73`
    says "the headings are not being read here", from nobody's ear. `Ctrl+Shift+;` is on no
    row of `docs/KEYBOARD_SHORTCUTS.md`. The visible layout is `ColumnLayout` applied at
    `wx_app.rs:9880`; `heading` at `message_columns.rs:96`; the six flag columns answer a word
    or nothing (`message_rows.rs:59-110`).
  - [S] #26, the tester on 2026-09-15: "column headers should not be announced each time. The
    user should be able to specifically request the reading of columns and corresponding text
    by pressing a keyboard command. use control+shift+; if not already assigned."
  - [D] `message_rows::the_row_with_its_headings` composes the visible cells in order as
    heading then text, the six self-describing columns as their text alone, empty cells left
    out; an Action menu item with the chord announces it once as content the mute controls,
    for a message row and a conversation row, and refuses off the list; the comments that
    said the headings are not read are corrected.
  - [D] `docs/KEYBOARD_SHORTCUTS.md` says the three routes (the program speaking rows, an NVDA
    add-on, an NVDA setting), that the third is taken and why, and gives the steps for a
    configuration profile triggered by Wixen Mail with Row/column headers off, naming
    Narrator's and JAWS's own settings as the place to look; the add-on is later work in the
    ledger.
  - [S] That the reading is heard whole and once on the key, and that arrowing is quiet under
    the profile, are his ear's; Narrator and JAWS are unread.

- [x] **LIST-08**: A rule can change how a row is announced: a phrase said first and shown
  in a column, a sound once per check, and the labels heard on the row as a column.
  - Evidence: `FilterAction` has seven variants (`filters.rs:11-19`), stored as one
    `action_type` and one `action_value` per rule (`mod.rs:1845`); `Outcome` carries read,
    starred, move_to, tags, delete (`:461-472`); `apply_rules` and `carry_out`
    (`mail_sync.rs:985-1070`) write per message; `cell_text` and `text_for` take no
    per-message override (`message_rows.rs:59`, `virtual_rows.rs:63`); `MessageColumn::ALL`
    is fifteen (`message_columns.rs:77`) with no Labels column, though `MessageItem.labels` is
    filled per message by 10-02's read; the sound scheme is keyed by `Event`, whose list is a
    census (`feedback.rs:76-140`, `sound_scheme.rs:81`); `NewMail` is signalled once per check
    from the `WhatArrived` arm (10-04).
  - [S] #62, from the Outlook gap report of 2026-08-27 and the audit of 2026-09-15: "Let a
    rule change how a row is announced, not only how it looks, so the emphasis reaches
    somebody who cannot see the colour."
  - [D] `FilterAction::SayFirst(phrase)`, stored as `say_first` with the phrase bounded,
    written onto the message in an additive `says_first` column, carried into `MessageItem`,
    prefixed to the first visible cell of the row under either view so it is the first thing
    spoken, and shown in a Says first column; a Labels column off by default; both columns
    in the Columns dialog. Amended 2026-09-19 by 11-10: a conversation row's labels are
    selected by the conversation query as the column's own expression over the rows of
    `here`, one per line, rather than joined in Rust from the per-message read, so the cell
    and the sort are one expression as the file's rule requires; the listing guard's closed
    set gains `tags` for the Labels sort; the phrase's heading is left unsaid on request and
    the Labels heading is said (11-10, tasks 1 and 2).
  - [D] An additive `plays_a_sound` flag per rule with a box in the rule editor; a new
    `Event::RuleMatched` with its own tone and Feedback row; the check counts the matches
    that sounded and the `WhatArrived` arm signals the event once per check, never per
    message; the rule editor offers the action and the box; readings hold the prefix, the
    columns, the bound and the arm. Amended 2026-09-19 by 11-10: the box's letter is S,
    since P is the pattern's; the value box is called the phrase under Say this first with
    H for its letter; OK refuses a missing or over-long phrase in a sentence; the manager's
    list says every action in the editor's words (11-10, tasks 1 to 3).
  - [S] The phrase at the start of a row, the sound once after a check with several matches,
    and the labels read as part of the row, are his ear's (ledger 556).

- [x] **LIST-09**: Pictures a message points at are shown by default except tracking pixels
  and pictures the sender marked decorative, a linked picture takes the link's words as its
  description, and an undescribed picture is described as nothing unless Settings says image
  or photo.
  - Evidence: `hold_back_remote_pictures` on by default (`config.rs:89`, `:715`) holds every
    remote picture back through `what_to_do_about` (`pictures.rs:555-572`); his profile has it
    off, read 2026-09-18. `hold_back_what_would_be_fetched` (`html_renderer.rs:464-486`)
    replaces a held-back picture and rewrites only an `alt` that is present and empty on a
    shown one (`:508-540`), on the reading path and never in `sanitize_html`; ammonia keeps
    `width` and `height` on `img` and drops `style` (`:142-175`), so a beacon is told by its
    declared size and by nothing else the clean leaves. A picture with no `alt` is left as it
    is in mail; in a note, `long_text.rs:411-420` says "image with no description", held by
    the test at `:1189`. Nothing takes a link's text as a description.
  - [S] #28, the tester on 2026-09-15: "By default, only beacons should be avoided along with
    decorative images. Photo links should have the link text as the default alt for the photo
    unless there's an associated alt. Photos without descriptions should automatically be
    given "" as the alt by default unless the user specifically chooses either 'image' or
    'photo' in settings."
  - [D] `hold_back_remote_pictures` defaults to off; `looks_like_a_beacon` reads a declared
    width or height of a pixel or less; a beacon and a decorative remote picture are not
    fetched and the message-top sentence counts the pixels; the switch still holds every
    remote picture back when on; a tracker the size of a picture is fetched, said on the
    privacy page. Landed 2026-09-19 by 11-11 at `f497785f`, held by
    `tests/pictures_show_by_default_except_beacons.rs` (the rules through the real cleaner,
    the newsletter through the renderer with its beacon counted and its sentence in the
    document, the switch's path) and the renderer's own `test_a_tracking_pixel_is_not_fetched`
    rewritten to the shipped default. Amended by 11-11: a decorative remote picture is said
    or passed over as `announce_decorative_pictures` says, the reader's existing say over the
    sender's mark, and is counted by neither count (11-11, tasks 1 and 2).
  - [D] A linked picture with no `alt` and some link text takes the text, escaped; a picture
    with no `alt` takes `undescribed_pictures_read_as`'s answer, nothing by default, image or
    photo by choice, on the Reading tab under the two picture boxes with a sentence; a
    sender's description is untouched; the sending path is untouched, held by a case; a
    note's undescribed picture follows the same setting, its test rewritten in place; fixtures
    through the real cleaner hold each rule. Landed 2026-09-19 by 11-11 at `f497785f`, held
    by the same target (the link's words, the three answers pure and through the renderer,
    the sending path both as a case and as a source reading, the choice read back from the
    built dialog) and `long_text`'s rewritten image test. Amended by 11-11: under a word the
    note reader says the word alone where the picture is, not "image, image"
    (11-11, tasks 1 and 2).
  - [S] A shown picture in the preview, a passed-over undescribed one, the link's words as a
    description and the sentence about tracking pixels are his reader's.

- [x] **LIST-10**: The privacy page lists every way a reader of mail can be tracked, what
  this program does about each by default, what a person can change, and what it cannot
  protect against, each read from the code.
  - Evidence: `docs/privacy.md:314-330` is the one section, "Pictures a message points at",
    and `:326` says "There is no setting for this yet", false since `hold_back_remote_pictures`
    was written. 10-07 wrote one line of the list at `:183` (the whole-mailbox download and
    the watch). The other ways are in the code: read receipts (`wx_app.rs:19646-19730`, sent
    only on Send Read Receipt; the `rsa` advisory at `.cargo/audit.toml:65` reasons about the
    channel), links and link checking (`:350`), invitations (`:297`), the update check and
    download (`:417`, `:467`); `grep -rn -i 'telemetry\|analytics\|crash report' src` is the
    evidence for what is never sent.
  - [S] #29, the tester on 2026-09-15: "The user should know the ways that they can be
    tracked."
  - [D] The pictures section is rewritten for the new default with its cost and the switch,
    the stale sentence corrected by dating; a section "How a reader of mail can be tracked,
    and what this program does about each" lists remote pictures, read receipts, links and
    link checking, meeting invitations, the update check and download, the whole-mailbox
    download and the watch, and what is never sent, each naming the file it was read from or
    the section it cross-references, the "never" row quoting its grep. Landed 2026-09-19 by
    11-11 at `f497785f`; the "never" paragraph is the census re-taken that day, 38 places in
    14 files, twelve shipping and each named for what it is for, the two others compiled only
    into the tests (`common::answering` is a loopback test server, not the meeting-reply
    sender the plan took it for). Amended by 11-11: the receipts paragraph also says, traced
    through `opening_pgp::the_body_to_show`, that an opened PGP message is shown as text and
    so points at no picture the new default would fetch, which is the `rsa` advisory's expiry
    condition not tripping, and `.cargo/audit.toml` records the same (11-11, task 3).
  - [S] Whether the page is clear to the person it is for is his.

**Added 2026-09-18, later the same day: three more, from three issues filed that day after the
phase was planned, each taken by an inserted plan (11-13, 11-06.1, 11-09.1).**

- [ ] **LIST-11**: Every sentence the status bar shows reads to one shape in a person's words,
  read in one pass, with a reading that holds new sentences to the shape where a reading can.
  **Open at the phase's close, 2026-09-20.** 11-13, the plan that holds it, was deferred to
  the front of the next phase on Pratik's decision of 2026-09-20 under his token budget;
  nothing of it landed, and the `[D]` line below is still the plan's proposal. The closing
  read, 11-12, read the pages with the bar's sentences as the phase left them, which the
  plan was to have rewritten first. The `[S]` line is untouched. **Moved later on
  2026-09-20 to phase 12 as 12-03** (the file renamed with `git mv`, its premises re-taken
  against `0ad66e48`, 12-02's two new sentences added to the pass); the box is 12-03's to
  tick, and the traceability row says phase 12.
  - Evidence: at `08197657` on 2026-09-18, `grep -rn --include='*.rs' -F "<call>" src`, comments
    excluded, test modules not: `send_status(` 110, `send_refusal(` 99, `said_and_shown(` 82,
    `set_status_text(` 45, `UIUpdate::StatusUpdated(` 34, `UIUpdate::Progress(` 17,
    `send_progress(` 12 (the issue's 109, 98, 72, 45, 34, 17, 11 had test modules out). Six sites
    for one refusal in two kinds (`wx_app.rs:4135, 10348` "Choose a message first"; `:4856` "No
    message selected to delete"; `:4913` "No message selected"; `:13954, 13968` "No conversation
    selected to delete"); `:5227` "Flushing outbox queue..."; `:2372` "No cache available for
    export". The built sentences: `checking_on_a_schedule::what_the_status_line_says`,
    `mail_sync::what_arrived`, `bringing_everything_down`'s six, `trying_again`'s wait.
    `tests/the_words_that_say_nothing.rs` is the pattern for a reading over words.
  - [S] #75, raised 2026-09-18: "Read every sentence the status bar shows for coherence and
    conciseness, as one pass ... every status sentence listed from the code ..., each rewritten
    to the same shape (what happened, to what, and what to do next when there is something to
    do; a person's words; one style of ending), the four refusals for 'nothing chosen' reduced
    to one wording per kind of thing, and a reading that holds new sentences to the shape where
    a reading can ... Nothing about which channel a line goes to changes."
  - [D] `application::status_sentences` holds the three "nothing chosen" wordings, one per kind,
    and the words a status sentence may not use with the ending rule a step and an answer
    each follow; a target prints the census of every site outside test modules and holds every
    literal to the rule with an exception table; every sentence is rewritten by hand in place,
    the built ones in their modules with their tests rewritten in place, no test added to
    `wx_app.rs`; every line keeps its call and channel, and `PROGRESS_OPENINGS` gains any new
    step opening; the summary carries the table of every sentence changed, old and new.
  - [S] Whether the bar reads well on its own with NVDA+End is his ear's.

- [x] **LIST-12**: After a delete, or a move out of the folder, the cursor is on the next
  message, or on the previous one when the last was deleted, on the list control and not only
  in the state; and a re-read of the folder keeps it on the same message by its identity.
  **Ticked 2026-09-19 by 11-06.1, merged at `0ed2c1a1`, on its `[D]` line:**
  `presentation::landing_after_a_removal::where_to_land(removed, len_after)` answers the row
  after the set, else the row before it, else the last row left, numbered as the rows are
  after the removal, and nothing when nothing is left or nothing was removed;
  `where_the_same_message_is` finds the cursor's message by its identity after a re-read;
  `whether_to_move` answers only when that row differs from the one it was on; twelve cases
  in the module and two records measured on the library. `wx_app::land_the_cursor_after`
  asks the rule and puts the cursor on the control through `put_the_cursor_on`, which clears
  and sets the row's selected and focused states so the focus event a screen reader reads the
  row from is raised even when the control already held the row; `take_row_out_of_the_list`
  calls it after the count, under the flat view; `keep_the_cursor_on_its_message` asks the
  other two rules and the `MessagesLoaded` arm calls it after the count, so 10-02's no-reselect
  holds for every load but the one in which the cursor's own message changed row. Held by
  `tests/deleting_a_message_lands_on_the_next_one.rs` on a built virtual list: after a middle
  row the cursor is (2, 2) with a focus event raised; after the last row with the cursor on it,
  (2, 2) where the control on its own held (-1, -1), which is the record of the day; after a
  re-read that moved the cursor's message to row 4, (4, 4); after one that left it there, no
  focus event and the cursor unchanged; two readings over the source with companions; two
  records on `wx_app.rs` measured on the target. The premise that the control holds nothing
  after any shrink was half wrong: it holds the row when the index stays in range and nothing
  when it does not. The `[S]` line is untouched and is ledger 541; nobody has heard the landed
  row, and which of the two paths the tester met was not watched, since a delete cannot be
  driven here while his copy is open.
  - Evidence: `take_row_out_of_the_list` (`wx_app.rs:19495-19520` at `08197657`) sets
    `selected_message_index` to `idx.min(len - 1)` or `None`, calls `tell_the_list_how_many`
    and `refresh`, and sets no item state on the control, reached from
    `MessageDeletedFromCache` (`:18338`) and `MessageLeftTheFolder` (`:18341`);
    `put_the_selection_back` (`:14148-14172`) is the shape that does set `Selected` and
    `Focused`. `MessagesLoaded` (`:17482-17497`) replaces the rows and does not re-select, by
    10-02's design, and since 10-06 the watch's `MailboxChanged` re-reads the open folder after
    every delete on an IMAP account. Which path the tester met is not settled by reading.
  - [S] #76, the tester on 2026-09-18: "Deleting a message puts the cursor at the top of the
    list. It should land on the next message, and on the previous one when the deleted message
    was the last."
  - [D] `presentation::landing_after_a_removal::where_to_land(removed, len_after)` answers the
    next row or the previous, for one row or a set; `where_the_same_message_is` and
    `whether_to_move` answer the cursor's new index after a re-read and whether it moved;
    `take_row_out_of_the_list` sets the control's `Selected` and `Focused` from the rule;
    the `MessagesLoaded` arm re-selects only when the cursor's message moved; a built virtual
    list in a target proves a middle removal, a last removal, a re-read that moved the id and
    one that did not; 11-07's delete of a set lands after the set through the same rule.
  - [S] That NVDA reads the landed row once after Delete and not again after the re-read is
    his ear's.

- [x] **LIST-13**: Landing on a message with an attachment says the word once: the earcon
  plays by default, the spoken event is off by default, the Attachment column stays.
  - Evidence: `feedback_events_for_landing` (`wx_app.rs:19625-19634`) signals `HasAttachment`,
    spoken at Low as "Has attachment" (`feedback.rs:202, 256`) with a tone (`:304`); the
    Attachment cell reads "Has attachment" (`message_rows.rs:77-80`) and the column is in
    every default layout; `FeedbackSettings::default()` disables the Earcon channel
    (`feedback.rs:505-516`) with phase 6's reason; `channels_for` (`:581-602`) filters an
    event's own set or every channel by the global switch and adds the first enabled of
    Braille, Visual, Speech to a sound-only set; a set holding Braille calls `announce_topic`,
    which the screen reader speaks (`accessibility.rs:258-266`; `feedback.rs:335-341` says
    braille is a preference, not a transport; the Feedback tab's one switch names both). The
    tester's profile stores `off=`, every channel on, read 2026-09-18 through Python.
  - [S] #77, Pratik on 2026-09-18: "The attachment earcon is on by default and the spoken
    announcement of the event is off by default. The column stays"; and his comment: "the
    Earcon channel is enabled by default for every event ..., and per-event exceptions stay
    allowed ... For this event the exception is the default itself: the earcon plus the status
    bar and braille, no speech."
  - [D] `FeedbackSettings::default()` enables every channel, and `HasAttachment`'s own set of
    earcon and status bar is `the_default_for`, read wherever no answer was stored for the
    event (amended 2026-09-19 by 11-09.1: the plan put the set in a fresh profile's per-event
    list, which would not have reached the tester's stored profile of `off=`), phase 6's
    reason kept and dated; `from_stored("off=earcon")`
    keeps earcons off; an event whose text is already on the row never gains speech or braille
    from the never-sound-alone rule, only the status bar; the older-file test and the Feedback
    tab readback are rewritten in place; a reading holds the landing arm to signalling and
    nothing spoken; the decision's "and braille" is the one clause the tree cannot carry
    without speaking the words, said in the summary and the close comment.
  - [S] That the row is heard once with the tone is his ear's.

**Added 2026-09-18, in the evening: six more, from six issues filed that evening, two taken as a
third task of a plan not yet executed (11-06.1, 11-09.1) and four by inserted plans (11-04.1,
11-09.2, 11-11.3, and 11-11.1 with 11-11.2).**

- [x] **LIST-14**: A delete says the one word Delete when the key goes down and nothing when
  the server answers; the next row's reading is the confirmation; a failure is still spoken;
  the status line keeps the fuller words for the eye; the same for the moves.
  **Ticked 2026-09-19 by 11-06.1, merged at `0ed2c1a1`, on its `[D]` line:**
  `UIUpdate::Shown(String)` is written to the status bar and its record and spoken by
  nothing, named in `quiet_on_purpose` with the reason so
  `test_every_arm_that_shows_something_says_it_or_is_named_as_quiet` holds it; `send_shown` is
  its sender beside `send_status`; `say_the_one_word` announces one word at Normal. The Delete
  arm, for Delete and Delete Permanently, says "Delete" and shows "Deleting subject..."; the
  local route's success is shown; the server's agreed answer and a move's outcome go through
  `show_or_say_what_happened_next`, which takes the row out and shows the line when the row
  leaves and speaks it at Normal when the row stays, since then nothing else says anything
  happened; `NothingWasSent` and `TheServerWouldNot` are unchanged, a refusal at High. Move to
  Folder and Copy to Folder say "Move" or "Copy" once the folder is chosen and show the fuller
  line; a delete of a collapsed conversation row still says once how many it is deleting and
  its messages' outcomes are shown. Held by three readings with companions in
  `tests/deleting_a_message_lands_on_the_next_one.rs` and one in
  `tests/progress_is_shown_and_results_are_said.rs`, extended in place for the third channel;
  two records on `wx_app.rs` measured on the target and one rewritten onto the new agreed
  case and measured on the library. Not changed, with the reason in 11-06.1's summary: the
  folder delete and folder move, which ask first and whose outcome says how far a delete of
  several folders got; and the unfinished move's sentence at startup. The `[S]` line is
  untouched and is ledger 542; nobody has heard the one word or the silence after it.
  - Evidence: the Delete arm (`wx_app.rs:4847` at `eb5d8517`) sends `send_status("Deleting
    {}...")` and the server's answer comes back through `server_delete.rs:43-56` and the arms at
    `:20718-20740`, each spoken as a `StatusUpdated`; the folder and conversation deletes at
    `:8779` and `:14048` follow the same shape; no `UIUpdate` kind exists today that is shown
    on the status bar and never spoken (`ui_types.rs`, 79 tests, 6 records; `server_delete.rs`
    17, 1).
  - [S] #83, the tester on 2026-09-18: a delete is spoken twice, once when pressed and once
    when done, and the landed row's reading is what he wants to hear.
  - [D] `UIUpdate::Shown(String)` is shown on the status bar and named quiet on purpose in
    `test_every_arm_that_shows_something_says_it_or_is_named_as_quiet`; `send_shown` is its one
    sender; the Delete arms announce "Delete" at Normal when the key goes down and send the
    success as `Shown`; a failure is spoken at High with its reason as before; Move to Trash,
    Move to Folder and a delete of a set take the same shape; a reading over the arms holds
    it, and two guard records are measured (11-06.1, task 3).
  - [S] That the landed row's reading is enough on its own is his ear's.

- [x] **LIST-15**: Alt+A reaches the attachments from the reader and from the formatted
  page window, and F7 the warning, whichever control has focus, the WebView included; the
  pages say Alt+A where they said F8.
  **Ticked 2026-09-18 by 11-04.1, merged at `70d84bc5`, on its `[D]` line:** the page
  window's script, `presentation::page_jumps::SCRIPT`, posts `{kind:'attachments'}` on
  Alt+A and `{kind:'warning'}` on F7 and the module's reader turns them back into jumps,
  with a test walking every posted kind through the reader; the window's handler moves
  focus to the list saying "Attachments, N", to the bar saying "Security warning", says
  "No attachments" and "No warning" with nothing to go to and "Message" on the way back;
  the `KEY_DOWN` binding on the page is removed; the reader's item reads
  "&Attachments\tAlt+A", the sentences say "Alt+A for them" and "Alt+A for the list", the
  reader's way back answers the chord from the menu handler when the list has focus; held
  by `tests/attachments_are_reached_with_alt_a_in_both_views.rs` (five readings and five
  companions, 10 tests) and four guard records; the shortcuts page, the guide and the
  changelog say Alt+A with F8 dated. The `[S]` line is untouched and is ledger 538; nobody
  has heard the landing, and the reader's way back through the accelerator was reasoned
  from the toolkit, not watched.
  - Evidence: the reader binds `ID_GO_ATTACHMENTS` to `F8` (`wx_reader.rs:36`, `:316-317`
    "&Attachments\tF8") and its sentence says "F8 for the list" (`:238`); the page window
    binds `KEY_DOWN` on the page for F7 and F8 (`wx_app.rs:21201`), which WebView2 never
    delivers while the browser has focus, so the key works from the bar and not from the
    document; the injected script (`:11959`, `wire_the_way_out`) posts Escape and F6 only;
    `docs/KEYBOARD_SHORTCUTS.md:188`, `:214` and `:225-226` say F8.
  - [S] #84, the tester on 2026-09-18: F8 does nothing from the message body, and F8 is the
    column chooser's key on the list.
  - [D] The script posts `{kind:'attachments'}` on Alt+A and `{kind:'warning'}` on F7, the
    handler moves focus and speaks, the dead `KEY_DOWN` binding at `:21201` is removed, the
    reader's item reads "&Attachments\tAlt+A", the sentences say "Alt+A for them" and "Alt+A
    for the list", the pages follow, and a target drives both views (11-04.1).
  - [S] That Alt+A lands on the attachments under NVDA from inside the document is his
    ear's.

- [x] **LIST-16**: The earcons keep playing after hours open: a device that goes away or is
  invalidated is noticed from the stream's error callback and the default device is opened
  again before the next sound; a default device that changes under a live stream is covered
  by a reopen after a gap or by Windows' notice, whichever a measurement chooses; a reopen
  that fails is logged once for the outage and said once on the status bar; the sounds
  resume when a device can be opened.
  - Evidence: `EarconPlayer::new` opens the default device once
    (`feedback.rs:842`, `open_default_sink().ok()`) and holds it for the run (`:771`);
    `Mixer::add` ignores a send when nothing listens (`:850-853`), so a stream that ended
    underneath makes every later `play` answer true and play nothing; nothing reopens and
    nothing listens for the stream ending; `rodio` 0.22.2's `DeviceSinkBuilder` takes an
    error callback (`stream.rs:368`) and `cpal` 0.17.3's WASAPI loop calls it with
    `DeviceNotAvailable` on `AUDCLNT_E_DEVICE_INVALIDATED` and ends the stream thread
    (`wasapi/stream.rs:382-404`, `wasapi/mod.rs:98-110`); cpal does not follow the default
    device changing.
  - [S] #81, the tester on 2026-09-18: "After Wixen Mail has been open for a few hours, the
    earcons stop playing. Nothing says so; the setting is still on."
  - [D] The player's output is opened through the builder with a callback that sets an
    `ended` flag; `play_at` opens the default device again on the flag, and after a gap when
    the open measured cheap, else on Windows' default-device notice; a failed reopen is one
    warn line and one `take_complaint` sentence that `Accessibility::signal` hands to the
    visual channel; cases over a dropped source, the gap, a failing opener and the resume,
    and one for the complaint reaching `take_visual_feedback`; the probe's median and worst
    quoted with the command and the date (11-09.1, task 3).
  - [S] Whether what silenced them after hours was the device going or something else is
    his machine's; if they stop again the log at `debug` holds the moment.

- [x] **LIST-17**: A message row's snippet is the message's first relevant words, chosen by
  reading the text and not by cutting it at 200 characters: addresses dropped altogether,
  lines that are only an address or a marker skipped, recognisable opening boilerplate
  skipped, a bare greeting skipped when something follows it, quoted lines and the signature
  after the delimiter left out, the first sentences with words taken up to the limit ending
  at a sentence boundary inside it, and the least bad line when nothing survives.
  - Evidence: `snippet_from` (`bodies.rs:312-325`) splits on whitespace and cuts at 200;
    `snippet_of` (`:339-347`) takes the plain part as written, else `words_of_markup`; the
    reader drops a link's address and keeps its words (`long_text.rs:1576-1584`); nothing
    looks for an address; the pass that puts stored snippets right runs once under
    `SNIPPETS_PUT_RIGHT` over HTML-only bodies (`:350-351`, `:767-779`); `sign_off::split`
    finds the signature delimiter (`sign_off.rs:76`).
  - [S] #82, the tester on 2026-09-18: a snippet holding a link reads the whole address;
    and Pratik's decision on the issue: "the message's first relevant words, chosen by reading
    the text rather than cutting it at 200 characters ... Rules, written down and tested one
    by one, not a model; a message whose text has nothing but those gives the least bad line
    rather than nothing."
  - [D] `application::snippet` holds the rules as functions with a test each and
    `first_relevant_words` over lines; `snippet_of` uses it for the plain part's lines and
    for the markup's pieces through `long_text::pieces_of_markup`, added beside
    `words_of_markup`, which the search index still calls; the pass is widened to every
    stored body under `SNIPPETS_ARE_THE_FIRST_RELEVANT_WORDS`, the older marker's row left
    where a database has it and named in the doc rather than kept as a constant nothing
    reads; a target reads a saved body's snippet back through the listing the window runs and
    runs the pass over an older stored snippet; the pass's milliseconds over 2,000 bodies
    quoted, 752 ms on 2026-09-19 with every row rewritten. Amended 2026-09-19 by 11-09.2:
    two rules beyond the decision's list, the invisible padding of a hidden preheader
    dropped and a line repeating the line before it said once, both from a Substack message
    in the tester's mail; and a picture left out only when undescribed, since the reader
    writes a described one as its words (11-09.2, tasks 1 and 2).
  - [S] That the rows now say the message is his ear's (ledger 554).

- [x] **LIST-18**: A Markdown block marker typed with its space at the start of any line of
  the message body becomes its structure, on the first line, after a line break in a body
  that arrived as plain text, after Enter on the empty first line and after Shift+Enter; a
  refusal that met a marker is logged and never announced; text after a closing inline
  delimiter is plain; a reading types into the real page.
  - Evidence: the block rule runs on the typed space (`editor_document.rs:801`) and refuses
    when the text node has a previous sibling (`:742`); `escaped_plain_text` (`:77-81`) makes
    every line after the first of a plain body a text node after a bare `<br>`; the probe of
    2026-09-18 on runtime 153.0.4234.32 made an `<h2>` on the first line at `3e633252` and at
    `744d05ef`, refused after a `<br>`, after Enter on the empty first line and after
    Shift+Enter, and left `##Heading ` as text; no test drives the real page with keystrokes.
  - [S] #79, the tester on 2026-09-18: "The compose dialog does not allow for markdown
    writing", "I'm expecting the structure", and "He pressed ## followed by text; nothing
    became a heading."
  - [D] `startsItsLine` replaces the sibling guard; a refusal after a `blockRule` match posts
    `{kind:'refused', where:'line'}` and `wx_compose` writes it at debug; the inline applier
    releases the style after its closing delimiter; `- ` on the empty first line is measured
    and made to work or said; `tests/a_marker_counts_at_the_start_of_any_line.rs` drives the
    real page with keystrokes through the steps; the pages say "type the marker, then a
    space" in words and the guide lists the markers (11-11.3). Held at `6e23656b`
    (2026-09-20): `startsItsLine` true for no previous sibling, a `<br>` or a block, the
    refused post after a `blockRule` match, `parse_message` reading it as
    `BlockMarkerRefused(NotAtTheStartOfItsLine)` and `wx_compose` writing it at debug; the
    first character typed after a closing delimiter taken out of the style with
    `removeFormat`, since the toggle was measured to leave a code span out; `- item` on the
    empty first line measured to make a list; the reading's twelve steps on the real page
    with posted keys each waited for; the two pages; two guard records; ledger 565 for the
    ear (11-11.3).
  - [S] Which of the two states he was in, a lower line or no space, is his ear's; the by-ear
    steps are in the ledger.

- [ ] **LIST-19**: Where a link opens is a setting on the Reading tab, the default browser by
  default, the message view, or a separate Wixen Mail window; the link's context menu offers
  all three whatever the setting; a link activated the way NVDA's Enter activates it goes
  where the setting says; every route passes the sanitiser; the separate window is a process
  of its own with a browser profile of its own; the privacy page says what each route
  shares with the preview.
  - Evidence: every navigation the preview or the page window sees is vetoed and handed to
    `open::that` after `safe_external_url` (`wx_app.rs:1351-1397`, `:21054-21080`;
    `html_renderer.rs:860-880`); the link's menu offers Copy Link and Save Link
    (`:1474-1492`, `:4462-4480`); no setting exists; wxdragon 0.9.17 creates every WebView
    through `wxWebView::New` with no configuration and exposes no browsing-data clear, and
    wxWidgets 3.3.2 puts one WebView2 environment per process under
    `wxStandardPaths::GetUserLocalDataDir()`, which is `%LOCALAPPDATA%\wixen-mail` here
    (`EBWebView` sits beside `cache` and `config` in the tester's profile, read through
    Python).
  - Held by 11-11.1 at `8340e5e6` on 2026-09-20: the setting on the Reading tab with the
    browser first and the older-file test carrying it; the three items on the link's menu
    on both surfaces; a link activated any way going where the setting says, caught in the
    page by `page_links::SCRIPT` because the veto never fired (wxdragon 0.9.17 hands a
    navigating event an empty string, measured in
    `tests/a_link_opens_where_the_setting_says.rs`); every route through
    `safe_external_url` before `opening_links::route`; the message view route with its host,
    title, failure and way back said; the privacy page's "Where a link opens". Not yet held:
    the separate window as a process of its own with a profile of its own, 11-11.2's, and
    the box waits for it. **Open at the phase's close, 2026-09-20:** 11-11.2 was deferred
    to the front of the next phase on Pratik's decision of 2026-09-20 under his token
    budget, so the separate-window clause is unheld and the box stays open; the closing
    read, 11-12, found 11-11.1's names in the tree and reworded the pages' "with the next
    build" to a later build, dated. The program's own two sentences, `WHAT_EACH_CHOICE_COSTS`
    and `SEPARATE_WINDOWS_ARRIVE_LATER` in `src/application/opening_links.rs`, still say the
    next build, held by a test to those words; they are ledger 566 and 11-11.2's to retire,
    as 11-11.1's summary already says. **Moved later on 2026-09-20 to phase 12 as 12-02**
    (the file renamed with `git mv`, its premises re-taken against `0ad66e48`, the two
    sentences and ledger 566 added to its task 2); the box is 12-02's to tick, and the
    traceability row says phase 12.
  - [S] #80, Pratik on 2026-09-18: the setting, the three menu items, the in-app routes in a
    profile of their own, the privacy line; and the tester: "Enter on a message and Enter on
    a link both open in the same window; the link does not go to the default browser."
  - [D] `opening_links::Where` and `route(setting, asked, address)` as one pure function with
    every cell tested; `open_links_in` on the Reading tab with the two settings guards; the
    page script catches the anchor's activation and posts it; the three items; the message
    view route with its title spoken, its failure spoken and Backspace back; a `page` scan
    target and an NVDA case as the probe and the regression (11-11.1); `--show-page` answered
    before the claim and the handover, `page_window::show` with the app name set before the
    WebView, the child sanitising again, the erase reaching the profile, the route spawning
    the executable (11-11.2).
  - [S] The three routes under NVDA are his ear's.

**Added 2026-09-18, in the night: two more, from two issues filed after the evening's six,
as inserts (11-06.2 beside 11-06.1, which was at three tasks; 11-07.1 after 11-07 and before
11-08); #85, the gate's own hazard, is FOUND-19 above.**

- [x] **LIST-20**: When the message list takes focus by Tab, F6 or a click and no row is
  focused, the cursor lands on the remembered row for the folder or on the first row under
  the sort, selected and focused with the viewport following; nothing moves while focus is
  elsewhere; an empty list says No messages.
  **Ticked 2026-09-19 by 11-06.2, merged at `116968fb`, on its `[D]` line:**
  `presentation::landing_after_a_removal::where_to_land_on_arrival(remembered, len)` answers
  nothing for an empty list, the remembered row when it is inside the rows, else the first,
  four cases in the module and one record measured on the target;
  `presentation::list_arrival::wire` binds the list's `SET_FOCUS`, skipped so the control's
  own handling runs after it, and lands only when the control holds no focused item, one
  record measured on the target; the mail list is wired once in `wx_app.rs` with `choose`
  over the state's `selected_message_index` and `view_state::how_many_rows`, and `on_empty`
  saying "No messages" at Normal and sending it as `Shown`, so it is said once. Held by
  `tests/tab_from_the_tree_lands_on_the_newest_message.rs` on a built tree and list, focus
  moved from the tree by `set_focus` four times: with nothing remembered and no row, (0, 0),
  `choose` asked once, the focus events the list itself then row 0; with row 2 remembered,
  (2, 2); with the cursor already on row 1, (1, 1) unchanged and `choose` not asked; a list
  with no rows, (-1, -1) and `on_empty` once. "The remembered row for the folder" is the
  state's index when it is inside this folder's rows, which survives a folder change, since
  nothing but a saved search clears it and the viewport rule reads it the same way; the
  summary says so. The `[S]` line below is untouched and is ledger 543.
  - Evidence: `MessagesLoaded` (`wx_app.rs:17745-17758` at `4d9f14bf`) moves only the
    viewport, by a stated rule against moving a screen reader's cursor while focus is in the
    tree; F6 reaches the list through `Pane::List => msg_list.set_focus()` (`:1454`, `:4407`);
    the mail list binds no focus event; `put_the_selection_back` (`:14396-14420`) sets
    Selected and Focused and `ensure_visible`; the control answers its focused item to
    `get_next_item(-1, All, Focused)`, which the window already uses for Selected (`:12679`).
  - [S] #87, the tester on 2026-09-18: "Tab from the folder tree to the message list: focus
    lands on the list with no row under it. It should land on a row, the newest message."
  - [D] `landing_after_a_removal::where_to_land_on_arrival(remembered, len)` answers nothing
    for an empty list, the remembered row when it is in range, else the first, with four
    cases; `list_arrival::wire` binds the list's SET_FOCUS and lands only when no item is
    focused; the mail list wired once with the state's remembered index; No messages on both
    channels for an empty list; a built tree and list in a target moved by the list's own
    focus path, the focused item read back for four steps (11-06.2).
  - [S] That the newest message is read once on arrival, and not twice, is his ear's.

- [x] **LIST-21**: A move or a delete within an account completes on this computer first and
  the server is brought into line afterwards: the row leaves at once, the cursor lands by
  the removal rule, the success is shown and not spoken, the change is recorded as made here
  and not yet at the server, told to the server in the background and at the next check
  before any folder of the account is read, replayed after a restart, undone here and said
  when the server refuses; Enter on a folder in the Move dialog is the Move.
  **Ticked 2026-09-19 by 11-07.1, merged at `fa20d04a`, on its `[D]` line:** the
  `moves_waiting` table on open, one row per message keeping the folder and number the
  server still has it under, kept, listed in the order asked, stopped, read back through a
  second connection, eleven cases; `application::moves_waiting` with `what_happens_here`
  (the row into the folder under a reserved number and the marker, or marked deleted, or a
  copy made with its text, and the row kept waiting), `undo_here` (one write putting the row
  where the server holds it, or dropping a copy), `what_a_replay_answered` over
  `why_the_push_failed` (done, already done when the destination holds the identifier,
  refused with the server's words, not reached) and the sentences worded by `server_delete`,
  twenty-seven cases, the replay held against the loopback servers for a move, a delete to
  the trash, a delete outright and a copy answering all four ways, two moves in the order
  asked, and the real folder read over a moved row leaving it alone; the check and the
  download replaying before their first listing and ending the account's check when the
  server was not reached; `complete_here_then_tell_the_server` in the window taking the row
  out, showing the line and pushing once on the account's session, the move arm and the
  delete arm both through it with the gate met at the key, `MovePutBack` undoing and speaking
  at High; Enter on a folder ending the dialog with `ID_OK` through the tree's activation,
  measured first on a built tree where Enter on a folder with children raised the activation
  and expanded nothing; fifteen readings and companions in
  `tests/a_move_completes_here_first.rs`; six records measured. The rows a folder's sync
  must not forget need no subtraction: the marker the cache's own move sets keeps them out
  of the comparison and the forgetting, a case holds it on the real sync, and the summary says
  so. The cross-account half is 11-07.2's, as this line already says. The `[S]` lines are
  untouched and are ledger 546.
  - Evidence: `spawn_folder_move` (`wx_app.rs:19364` at `4d9f14bf`, "the row goes once the
    server has agreed and not before") on `the_session_at(&account)`, the row leaving at
    `:19886` and the sentence after; the delete's reason at `:4950-4956`; the shape for a
    change made here first in `application::flag_changes_waiting` and
    `data::message_cache::waiting_flag_changes`, offered on the session a check opened
    (`:22804`); a folder sync forgets what the server no longer lists
    (`mail_sync.rs:1383-1390`); the Move dialog's button has no default and the tree no
    activation binding (`wx_destination.rs:268-290`); wxdragon 0.9.17 exposes
    `TREE_ITEM_ACTIVATED` and `Button::set_default`; the scripted loopback servers at
    `imap.rs:2814-2830`.
  - [S] #86, the tester on 2026-09-18: "The move takes a noticeable time to complete and to
    be announced. It should complete at once on this computer, with the server brought into
    line when the next check runs"; and "Enter on the chosen folder in the tree does nothing
    ... Enter on the folder should be the Move."
  - [D] A `moves_waiting` table with keep, list, stop and the rows a folder's sync must not
    forget; `application::moves_waiting` with what happens here, the undo, what a replay
    answered (done, already done, refused, not reached) and the sentences, held against the
    scripted servers; the sync replaying the account's waiting moves before its first listing
    and leaving alone what a waiting move holds; the move and delete arms completing here
    first with the success as `Shown`, the push at once on the action's session and again at
    the next check, `MovePutBack` undoing and speaking at High; Enter on a folder ending the
    dialog with `ID_OK`, measured first on a folder with children; a move across accounts
    unchanged and said (11-07.1). **Overruled 2026-09-19 for the cross-account half: 11-07.2
    and LIST-22, merged at `2526b31f` later that day; a move or a copy across accounts
    completes here first as well, and the sentence that said it waits is gone from the guide
    and the changelog.**
  - [S] A replayed move after a restart against a real server, a move of a message the server
    changed meanwhile, and #63's move, copy and delete proofs re-taken after this, are his
    account's.

**Added 2026-09-19: one more for #86's second half, after Pratik overruled 11-07.1's
decision 29 on the issue; taken by 11-07.2 between 11-07.1 and 11-08.**

- [x] **LIST-22**: A move or a copy to a folder on another account completes on this
  computer first: the row moves at once (a copy's stays), the success is shown and not
  spoken, the crossing is recorded as owed with the message's bytes held in the store that
  already holds a moving message, the fetch at the source and the append at the destination
  run in the background and at the next check of either account before its first listing,
  the source is asked to let go only after the destination has answered, a restart resumes
  from the held bytes without a question, a refusal at either server puts the row back and
  is said, and a message over the ceiling keeps the server-first path and says why.
  **Ticked 2026-09-19 by 11-07.2, merged at `2526b31f`, on its `[D]` line:** the kinds
  `MoveAcross` and `CopyAcross` with the other account named, stored in two additive
  columns, read back over a second connection and offered at a check of either account,
  four cases in the table; the crossing cut into `fetch_and_keep`, `append_and_ask` and
  `remove_at_the_source`, `resume_the_append` and `resume_from_the_held_bytes` over a held
  row, `move_it_across` kept for the ceiling path, the question at start retired with its
  dialog and `why_it_cannot_be_finished_from_here` in its place, the file at 43 tests as
  before; the store answering `Held`, keeping a waiting crossing past its backstop and
  reading the source side from the waiting row, thirteen cases; `what_a_crossing_answered`
  reading `ItIsNotKnownWhereItIs` as not reached and never as refused, and
  `replay_the_crossings_waiting_for` over the `OpensASession` seam, held against a source and
  a destination each on the loopback for done with the row settled under the other
  account's number, refused with the undo, done and refused and not reached after a hang-up,
  the resume from held bytes fetching nothing, a copy with nothing removed, two crossings in
  order, the destination's check, an account gone, an account unreachable, a crossing nothing
  can settle, and a number the folder held before read as no arrival, eighteen cases; the
  window's arms routing every set through `move_or_copy_here_first`, which builds the kind
  per message with both gates met and groups the asks by account, the ceiling path alone
  through the worker with its sentence, the replay helper running the crossings after the
  moves on the check, the download and the push, eleven readings and companions in
  `tests/a_move_across_accounts_completes_here_first.rs`; six records measured, one
  corrected by hand from the runner's answer. Two things the `[D]` line said were done
  otherwise and the summary says why: a copy across is a marked copy row in the other
  account's folder rather than no row, keyed on the copy as a copy within the account is,
  and a message over the ceiling is kept out of the queue at the key by its size rather than
  by `TooLargeToHold`, which is the step's answer and is held by a case. The `[S]` line is
  untouched and is ledger 547.
  - Evidence: at `6911018d` (the same bytes as `15407b1e` in every file named),
    `mail_across_accounts::the_crossing` (`:463-556`) fetches at the source, keeps the bytes
    (`moves_in_flight::keep_the_message_while_it_moves`), appends at the destination, asks
    `whether_the_destination_has_it` when the append's answer never came, and only then
    `the_removal`; `move_it_across` (`:430-461`) runs it in front of the person and lets the
    bytes go on the way out; `finish_the_move` (`:620-675`) resumes from a held row; the four
    endings that are not arrived leave the row today
    (`server_delete::after_a_move_across_accounts`, `:102-157`); the store's ceiling is
    `LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES`, 25 MB, its budget 64 MB, its give-up seven
    days (`moves_in_flight.rs:64-85`), and "a message over it still moves"; at start
    `say_what_did_not_finish` (`wx_app.rs:28517`) asks a yes-no question about an unfinished
    move; `spawn_folder_move` (`:19816`) takes `Vec<AMessageMoving>` and the `Chosen` since
    11-07 and its `AnotherAccount` branches (`:19906`, `:19955`) open `the_session_at` for
    both accounts.
  - [S] Pratik on #86, 2026-09-19: "a move across two accounts completes here first as well,
    not server-first ... the message's bytes held here (the outbox's shape) so a cross-account
    move is replayed from the row after a restart, the fetch at one server and the append at
    the other done in the background or at the next check, the row moved at once, the
    refusal at either server undoing it here and said."
  - [D] `moves_waiting` gains the kinds `MoveAcross` and `CopyAcross` with the destination
    account; the crossing is cut into `fetch_and_keep`, `append_and_ask` and
    `remove_at_the_source`, with `resume_from_the_held_bytes` for a held row and
    `move_it_across` kept for the over-the-ceiling path; `what_a_replay_answered` maps the
    six `MovedAcross` endings to done, refused or not reached, reading `ItIsNotKnownWhereItIs`
    as not reached and never as refused; `replay_the_crossings_waiting_for(account)` runs the
    crossings of which the account is the source or the destination on both sessions, from
    held bytes or from a fresh fetch; the `AnotherAccount` branches of `spawn_folder_move`'s
    worker complete here first through 11-07.1's function with the kind as the one
    difference, `a_set_leaving` landed at once; the question at start retired; a message over
    the ceiling answers `TooLargeToHold`, takes no queue row, and moves server-first with the
    sentence; cases against the scripted servers for each ending, the resume both ways, the
    ceiling and the order; readings over the arms, the three folder-reading paths and the
    retirement (11-07.2).
  - [S] Ledger 187's three questions (a ten megabyte upload, Gmail's label for a copy from
    another account, an identifier the destination already holds) and #63's copy and move
    across accounts, re-taken after this, are his accounts'.

**Added 2026-09-19, in the morning: two more, from two issues filed that morning, as
inserts (11-08.1 after 11-08, whose row message a re-threading moves; 11-10.1 after 11-10
and before 11-11, so 11-11.1's activation covers a made link).**

- [x] **LIST-23**: On a server that advertises Gmail's extension the conversation id is
  asked for in the fetch already made and names the conversation here, the stored id
  follows the server's and a message whose id changes moves; mail already stored gets its
  id once at the next check; on every server a child stored before its parent joins it when
  the parent lands, a sibling with a fuller chain and a reply with a cut one join their
  tree; subject matching stays refused; the conversation row's count and the row message
  follow a re-threading.
  - Ticked 2026-09-19 by 11-08.1 at `76897058` on its `[D]` line, the orchestrator's
    instruction overruling the README's "11-12 ticks"; the `[S]` lines are ledger 549.
  - Evidence: at `38ebcb86` (the same bytes as `1962e341` in every file named but
    `mail_sync.rs`, which gained four comment lines), `application::threading` threads from
    `References` and `In-Reply-To` alone (`:1-14`), refuses subject matching
    (`test_subjects_are_never_used_to_thread`, `:485`), and prefers
    `ThreadInput::server_thread_id` (`:26-31`), which only `apply_threading` supplies and
    always as `None` (`wx_app.rs:13428`); `GMAIL_FIELDS` is `X-GM-MSGID X-GM-LABELS` with
    `X-GM-THRID` "deliberately not asked for" because the library's reader once hid it
    (`imap.rs:112-120`), and `test_the_thread_id_is_not_asked_for` (`:2497`) pins the
    absence; `imap-proto` 0.16.7 parses the attribute into `AttributeValue::GmailThrId` and
    the attributes are read directly here (`:1918-1935`); the stored `thread_id` is
    `thread_identity::conversation_root(message_id, refs)`, the root of the chain whether or
    not the root is here, and `rejoin` reroots the conversations an arrival proves to be one
    onto the earliest name (`thread_identity.rs:1-40`, `:200-260`; `messages.rs:982-1000`,
    `:1116-1150`); a check lists a folder and stores what is new and never re-fetches a
    stored uid (`mail_sync.rs:1360-1405`); the tester's account holds 17,753 messages.
  - [S] #88, the tester on 2026-09-19: "Messages that belong to one thread show as separate
    threads with the same subject in conversation view."
  - [D] `GMAIL_FIELDS` gains `X-GM-THRID` and `ImapMessage::gmail_thread_id` reads it; the
    pinning test is rewritten in place to its opposite; `messages.server_thread_id`
    (additive) carried in `IncomingMessage` and `MessageListRow`; `conversation_root` and
    `rejoin` take the server's id, which wins when present; `apply_threading` hands the row's
    id to the in-memory threader; a once-only pass per Gmail account fetches the field for
    stored uids per kept folder at the next check before its listing, under `work_done_once`;
    the cost per message measured on the scripted server; the late-parent, fuller-sibling,
    cut-reply, Gmail-id-over-headers, differing-id and same-subject cases traced against the
    loopback servers newest first in a target, each case's state before and after in the
    summary, what failed fixed; readings that 11-08's row message and count follow a
    re-threading; six guard records (11-08.1). Done by 11-08.1 at `76897058` on 2026-09-19,
    with three changes of shape the summary names: the server's word goes through
    `the_conversation_of` in front of `conversation_root` and `rejoin` reads it from the
    stored name; the row carries the stored conversation and the in-memory pass is handed
    that, since the window looks members up by it; and the trace found the store already
    joined the late parent, the fuller sibling and the cut reply, and fixed the in-memory
    naming instead. Nine records.
  - [S] Whether his split threads become one after the next check, and whether a
    conversation here matches Gmail's, are his account's (ledger 549).

- [x] **LIST-24**: An address written out in a plain-text message, in the quoted part of a
  reply, in a note shown as a page and in an event or task description read aloud is a
  link, made by one recogniser that passes every link through the sanitiser's address rule;
  what is not an address is left alone; a description read aloud says a link to its host;
  the sanitiser keeps the schemes a sender writes and a link it refuses says so beside its
  words; the snippet still drops addresses.
  - Evidence: at `38ebcb86`, `wrap_body` shows a plain-text message escaped inside `<pre>`
    (`html_renderer.rs:612-630`) after the older fault of guessing markup was fixed by not
    guessing (`:606-611`); `escaped_plain_text` does the same for a reply's quoted plain part
    (`editor_document.rs:77-81`); `SAFE_URL_SCHEMES` is `http://`, `https://`, `mailto:`
    (`:12`) and `safe_external_url` (`:860-880`) filters every `href` the cleaner keeps
    (`:140-175`), a refused scheme losing its address and keeping its words with nothing
    said (`:2141-2172`); `long_text::spoken` reads a link's words alone (`:285-300`,
    `:1576-1584`) and `as_markup` is `pulldown-cmark` 0.13, which has no autolink
    (`:455-475`); the composer's link rule balances brackets (`editor_document.rs:681-704`);
    Pratik's comment: the tester's FanFiction message is plain text only with the chapter
    address bare on its own line and two more at the foot, so the sanitiser stripped nothing.
  - [S] Pratik on #89, 2026-09-19: "An address written out in a message, an invitation or a
    description should be a link a person can follow"; and his comment: "bare addresses in
    plain text become links through safe_external_url. The corpus check of the sanitiser's
    schemes stays as a smaller task."
  - [D] `application::links_in_text` with `addresses_in`, `as_html` and `spoken`: http,
    https, www., mailto: and a bare name@host address, ending before trailing punctuation
    and an unbalanced closing bracket, nothing inside a word, `a.b` and `1.2.3` and a bare
    host left alone, every href through `safe_external_url`, a case per shape and per
    non-address; `wrap_body`'s plain branch, `escaped_plain_text`, `as_markup`'s text events
    outside code and links, and `spoken` use it; `tel:` allowed; the corpus (`mailto:`,
    `tel:`, a port, a space, a fragment, `sms:`, `javascript:`) with a case per shape and a
    refused link's words followed by "(link not opened here: {scheme})"; a target holding the
    FanFiction shape, a reply's quoted text, a description spoken, a note as a page, the
    snippet still bare; five guard records (11-10.1). Amended 2026-09-19 by 11-10.1 as
    landed: the snippet asks the module's shape rule rather than keeping a copy; the
    cleaner already admitted `tel:` and `sms:` and dropped `javascript:` on its own, so
    `tel:` joins `SAFE_URL_SCHEMES` and the note is a pass over the cleaned markup on the
    reading path only, asking the gate of every href the cleaner kept, "the address" as the
    reason when the scheme is one this opens; a `javascript:` link keeps its words with no
    note since the cleaner drops it first; `as_markup` links text events outside a code
    block, a link and a picture, a code span being a code event and never text; `spoken`
    says the host in the passage returned as written and in every piece; six records, three
    on the module, one each on the renderer's plain arm, the note's code block and `tel:`,
    with the moved held-back-count record rewritten and a profile-reading record measured
    under a fresh profile (11-10.1, tasks 1 and 2; tests/an_address_written_out_is_a_link.rs).
  - [S] The chapter address in NVDA's link list on the tester's message is his ear's.

**Added 2026-09-19, in the afternoon: one more, from an issue filed that afternoon, as an
insert (11-11.0 after 11-11, which changes the same renderer for pictures, and before
11-11.1, whose listener reads the page this plan cleans).**

- [x] **LIST-25**: An HTML message read in the formatted view says what the sender showed,
  once: text the sender hid is dropped before the sanitiser by the sender's own rule and
  never by content, invisible filler is stripped, a count is said once only when a dropped
  block held words that were not a preheader, a layout table is not a table to the reader,
  a sender's grouping is named only where a reader would use the name, the page's own
  markup says the subject and the sender once, the plain part is never touched, and the
  reader's own structure reads the same message with its blocks apart.
  - Evidence: Pratik's measurement of the tester's Substack message (2026-09-19, the raw
    stored body of 101,756 bytes read through Python): two `display:none` divs at the top
    styled `display:none;font-size:1px;...;max-height:0px;...;opacity:0;overflow:hidden`, the
    first holding the subtitle "Actions speak louder than words" and two hundred U+034F
    joiners with no-break spaces, figure spaces and soft hyphens between, the second the
    padding alone; forty-eight of forty-nine tables `role="presentation"`, 111 cells, no
    `<th>`; one `aria-label`, "Post header", on a `div role="region"`; six `font-size:0`
    spacer cells with no words; no `visibility:hidden`, `mso-hide` or `aria-hidden`. At
    `f92871bb` (the same bytes as `ced898eb` in every file named), `cleaner()`
    (`html_renderer.rs:142-168`) is ammonia's defaults with `data` and `cid`, which drop
    `style`, `class`, `role` and `aria-*`, and its filter reads only `src`, `href` and
    `background`; `sanitize_and_count_held_back` (`:410-415`) cleans and then rewrites
    pictures by regex; `render_thread_under_a_bar` (`:757-830`) writes "{n}. Message from
    {sender}" for every message and the count line only for a conversation; `long_text`'s
    `OpenTable` (`:127-180`) pushes a cell's content as one string (ledger 555); `scraper`
    0.27 is the parser the reader already uses (`:560-566`).
  - [S] #90, the tester on 2026-09-19: "Reading an HTML message in the formatted view is
    verbose: groupings are announced and phrases repeat themselves as the reader moves
    through the page"; and Pratik's comments the same day, the measurement above and the
    leave to keep the message as a public fixture with its addresses replaced.
  - [D] `application::hidden_text` with `whether_hidden` over the six rules (and `opacity:0`
    alone not one), `strip_filler` over the joiner, the zero-width set, U+2060, U+FEFF and
    the soft hyphen, `what_a_dropped_block_was` (nothing, preheader, words) and the count
    sentence in the pictures' register; `drop_what_the_sender_hid` in the renderer, a
    `scraper` pass before the clean on the reading path only, never on the sending path or a
    plain part; `role="presentation"` allowed on `table`, `tr`, `td` and `th` and no other
    role; `aria-label` kept on a link or button and on a table that is not presentational,
    dropped elsewhere; the page's heading unnumbered for one message; `long_text`'s cell
    walk separating blocks and a presentational table read as blocks; the fixture at
    `tests/fixtures/issue_90_substack_newsletter.html` with every tracking address and the
    recipient's token replaced, a case holding its shapes and the token's absence; a target
    over the fixture through `wrap_body`, `render_thread_under_a_bar` and
    `pieces_of_markup`; the drop's cost on the fixture measured; six guard records (11-11.0).
    Amended 2026-09-20 by 11-11.0 as landed: `drop_what_the_sender_hid` lives in
    `hidden_text` and the renderer calls it, since the reader in the application layer
    calls it too; the drop runs in `sanitize_and_count_held_back` and never in
    `sanitize_html`, which is the sending path (the reply editor and the editor's body go
    through it); the blocks' count travels in a renderer-level `LeftOut` beside the pictures'
    `HeldBack` and the three sentences share the one `held-back-count` paragraph; the
    `aria-label` rule is `hidden_text::keeps_its_label`, decided in the walk because the
    cleaner's filter cannot see a table's role, and the cleaner admits the attribute on `a`
    and `table` only; `keep_the_layout_claim` is the one allowance both cleaners take; the
    reader drops and reads layout tables as blocks for speech only, and a `div` holding
    blocks is walked as blocks; thirty addresses replaced, the nine app links and three
    open-in-app links carrying the recipient's token too; the fixture holds three pictures
    a pixel square, not two; the cost a median of 11.6 to 12.1 ms over ten runs in the
    debug profile; six records new and one rewritten twice; no test added to
    `html_renderer.rs` or `long_text.rs`.
  - [S] That the newsletter now reads once, with no table and no grouping announced, is his
    ear's.
- [x] **LIST-26**: A setting saved in Settings applies without a restart. Mark as read after
  governs the next tick the moment it is saved, because the setting lives in the window's
  state, written at startup and by the Settings-saved arm through an update, and the timer
  reads the state; the date settings follow a save the same way; every setting the startup
  block captures once either follows a save or says on its control that it takes effect the
  next time the program starts, and a reading holds that list to the tree.
  - Evidence: at `390a580c` (2026-09-20, re-taken after 11-11.1's merge; first read at
    `4a09bfc2` with the same shapes), `wx_app.rs:1294-1299` binds `let marks_read =
    stored_config.as_ref().map(|cfg| MarkRead::from_setting(&cfg.mark_read_after))`, the
    timer's closure captures it and hands it to `mark_what_was_read(app, marks_read)` at
    `:6034`, and `:10822-10839` passes it to `whether_to_mark_read`; `grep -n marks_read`
    finds those three sites and no other. `SettingsResult::Updated` (`:18269-18335`) writes
    the file and sends `UIUpdate::WorkingDayChanged`, `DefaultEventAlertLeadChanged` and
    `CalendarViewChanged`, which the arms at `:19484` and beside it write into `WxUIState`;
    it sends nothing for this setting and nothing for the date settings, which `:1260` binds
    once into the row callback (`:1419`), the PIM cells (`:1723-1736`) and the read-aloud
    closures (`:1811-1887`), while `date_settings_from_stored_config()` (`:11991`) is read
    on use at `:2133`, `:17000`, `:17182` and `:19519`. The startup block (`:1255-1380`)
    reads `stored_config` once, "rather than per row: the paint callback runs for every
    visible cell and must not touch configuration", so the fix is the working day's shape
    and not a `load_stored` on the paint path. Of the fifty-seven `AppConfig` fields (11-11.1
    added `open_links_in`, read at the route through `where_links_open()`, `:12777`), nine
    are bound in that block; the rest are read where they act or, for `log_level`
    (`logging.rs:119`, once, no reload), `start_in_all_inboxes` and
    `check_default_programs_at_startup`, are startup by nature. The settings screen says
    nothing under the log level or the default sort order about when they apply
    (`wx_settings.rs:1557`, `:1480`). The issue's line numbers (`:1291`, `:6098`,
    `:10894-10904`) were the 11-11.1 branch's at `5b99cd7c`; the shapes are the same bytes.
  - [S] #91, the tester on 2026-09-20, on `1.0.0-alpha.1` at `4a09bfc2` or the build
    before: "Mark as read after" is not applied, not after Enter opens a message, not after
    Space reads the whole message, not after Shift+Space, and he changed the setting to each
    of its values to be certain; and his comment the same day, after restarting: with the
    setting at 10 seconds from startup, Enter, Space and Shift+Space mark the message, so
    there is one fault and no second, and the probe for a second is not owed.
  - [D] `WxUIState::marks_read` and `WxUIState::dates`, written at startup where the
    bindings were and by the arms for `UIUpdate::MarkReadAfterChanged(MarkRead)` and
    `UIUpdate::DateSettingsChanged(DateSettings)`, both sent by the Settings-saved arm
    after the file is written; `mark_what_was_read(app)` reading the state and taking no
    setting; the row callback, the PIM cells and the read-aloud closures reading the state
    under the lock they already take, with the six lists refreshed once when the dates
    change; `reading_habits::TAKES_EFFECT_AT_THE_NEXT_START` under the log level's control
    and, with "and only in folders whose columns you have not arranged", under the default
    sort order's, named on both channels; `tests/a_setting_saved_applies_without_a_restart.rs`
    reading `what_ships` for the field, the update, the arm, the absence of both captures,
    and the audit that every `stored_config` binding in the startup block is in an allowlist
    with a disposition; the two sentences read back from the built page; four guard records;
    the guide and the changelog naming the build and the regression's shape; no version
    move (11-11.1.1). Held at `051c3529` (2026-09-20): the target's twenty tests and the
    readback in `the_settings_dialog_opens_in.rs`, six records new and three re-measured,
    the audit reading eleven settings from the block (the captured local was held at ten
    sites, not the three the plan named, the contact details and the due window's look
    among them), the layout's disposition the window's own rather than the plan's
    "offered by no control", since Then by writes its second level; ticked on this line by
    the orchestrator's instruction, 11-12 still reads it.
  - [S] The delay changed in Settings and a message marked after the new wait, without a
    restart, is his ear's.
- [x] **LIST-27**: Thread View is on by default, as a setting, and All Inboxes has a view of
  its own. A folder nobody has set shows one row per conversation because Show conversations
  by default, on the Reading tab and on unless turned off, answers a folder never set, read
  where the folder opens; a folder's own choice still wins and a stored nought still means
  flat. All Inboxes keeps its view under its own row identity, switched with Ctrl+T, answering
  the setting when nothing was stored and read when it is landed on; the Thread View check
  mark says the view of what is on screen; showing conversations there lists every inbox's,
  one row per account and conversation, each row acting on its own account; a label view and
  a saved search keep one row per message and say so.
  - Evidence: at `390a580c` (2026-09-20), `view_state.rs:37-46` answers `Messages` for
    `None` by D-09 with the tests at `:605-630`; the folder landing (`wx_app.rs:3084-3095`)
    reads `cache.folder_view(&folder)` per landing and `Messages` for a row that opens no
    folder. `switch_the_view` (`:15144-15162`) refuses without a folder, and
    `the_folder_being_looked_at` (`:15519`) answers `None` for All Inboxes by its own doc;
    landing on All Inboxes (`:2947-2951`) sets the title and `load_every_inbox` and leaves
    `s.showing` and `s.conversations` as the last folder left them, and `load_every_inbox`
    (`:7565-7589`) reads messages only, so with a threaded folder open before,
    `tell_the_list_how_many` (`:21744`) is told the previous folder's conversation count and
    the paint callback (`:1402-1421`) draws the previous folder's conversation rows under All
    Inboxes' title, with the check mark saying whichever that was. The view store is
    `tree_state(identity, thread_view)` (`folders.rs:334-355`), keyed by the row identity,
    and `WhichRow::AllInboxes.stored()` is `"all-inboxes"` (`folder_tree.rs:142-144`), the
    identity the collapsed state and the landing (`wx_app.rs:4774`, `:17970`) use; the sort
    (`the_sort_as`, `:15073`) is one layout keyed by nothing, so the issue's "the key the sort
    uses" is the row identity and no schema change is needed. `conversations_in`
    (`messages.rs:2362`) is per account and folder through `conversation_scope`
    (`:188-222`); `unified_inbox_query` (`:467-483`) is every `folder_type = 'Inbox'`;
    `ConversationItem` (`conversations.rs:200`) carries no account, `conversation_nodes`
    (`wx_app.rs:13860-13868`) filters by thread id alone, and `the_open_folder_and_its_account`
    (`:10526`) is read at `:10580` and `:24165`; `test_two_accounts_do_not_share_a_conversation`
    (`messages.rs:6866-6898`, T-01-47) is the fixture with one thread id in two accounts. A
    label (`:2953-2968`) leaves `s.showing` as All Inboxes does; a saved search's arm
    (`:18778-18792`) sets `Messages` by its own comment and syncs no check mark.
    `reread_folder_if_open` (`:18574`) re-reads a folder only, so mail arriving while All
    Inboxes is open refreshes nothing in either view, older than #92. Settings: the
    `default_true` shape at `config.rs:427`, the two guards at `:2595` (which skips the
    settings screen, so a field's first reader must act) and `:2972` red on a new field, the
    older-file test at `:1728`; the Reading tab's Message List section at
    `wx_settings.rs:1579-1624`. Pages: `USER_GUIDE.md:697` "a folder you have never set is
    flat", `KEYBOARD_SHORTCUTS.md:706` "Kept per folder", `first_run.rs:130-144` and
    `ALPHA_TESTING.md:9-10` say nothing of the view.
  - [S] #92, Pratik on 2026-09-20 on `1.0.0-alpha.1+321.g4a09bfc2`: Thread View is on by
    default; Thread View cannot be switched on while All Inboxes is open, yet All Inboxes
    shows conversations when the account's inbox is in Thread View, which the tester saw;
    and his amendment the same day: what a folder shows when nobody has set it is a setting
    on the Reading tab, "Show conversations by default", on by default, beside Default sort
    order; a folder's own choice still wins; All Inboxes takes the setting when its own key
    holds nothing; read on use, never captured at startup.
  - [D] `AppConfig::show_conversations_by_default` with serde default true, in the
    older-file test, the two guards green; `Showing::when_nobody_set_one(bool)` and
    `Showing::from_stored(Option<i64>, Showing)` with `Some(0)` flat whatever the setting
    and an unrecognised number answering the setting, the D-09 tests moved; the folder
    landing reading the setting on use, the field's first reader; the check box beside
    Default sort order named on both channels and read back, with
    `tests/the_settings_dialog_opens_in.rs` reading it back; the field, the landing and the
    box one green commit because the two guards allow none between; the D-09 sentences in
    `view_state.rs`, `folders.rs`, the guide, the shortcuts page, the first-run screen and
    the alpha page dated; four guard records (11-11.1.2). `ConversationItem::read_in` with
    the account and the folder, at every literal site; `MessageCache::conversations_in_every_inbox(reach, order)`
    grouped by account and thread id, the two-rows test over T-01-47's fixture; one
    function answering the identity whose view and Thread column are kept, All Inboxes
    included; `switch_the_view` storing under it and loading every inbox's conversations
    there; the All Inboxes landing reading the key through the setting, clearing the rows,
    syncing the check mark; the Label landing and the `SavedSearchRan` arm syncing it; the
    sentence "Open a folder or All Inboxes first. A label and a saved search show one row
    per message."; `conversation_nodes` filtering by the row's account and `chosen_messages`
    and `spawn_conversation_text_fetch` reading `read_in`;
    `tests/all_inboxes_keeps_a_view_of_its_own.rs`; four guard records; the pages and the
    changelog; no version move (11-11.1.3). Held for the first half at `ae0fa4d2`
    (2026-09-20): `show_conversations_by_default` with `default_true` absent and in the
    struct's default, the older-file test and the two guards green; `when_nobody_set_one`
    and `from_stored(Option<i64>, Showing)` with `Some(0)` flat whatever the setting and an
    unknown number the setting, six D-09 tests; `what_a_folder_never_set_shows` read once
    per landing outside the startup block and handed in, the landing reading in the
    settings target requiring it of every `from_stored` call in the window; the box after
    Then by captioned and named with the same words, ticked from the file, cleared through
    the real control and read back through `read_settings`; the sentences dated in
    `view_state.rs`, `folders.rs`, the guide, the shortcuts page, the first-run screen and
    the alpha page; four records at 2, 1, 2 and 1 red. Held for the second half at
    `1e39650a` (2026-09-20): `ReadIn` and `ConversationItem::read_in` at the six literal
    sites the compiler named; `conversations_in_every_inbox(reach, order)` over
    `every_inbox_scope()`, the one-folder scope's text with four parts replaced, grouped by
    account and thread id, the two-rows test asserting each row's account, folder, count
    and row message, and the three correlations in `message_columns.rs` keyed by the
    account too; `the_identity_whose_view_is_kept()` answering All Inboxes by its own row
    identity, `the_view_kept_under()` the one `from_stored` call handing the setting in,
    `settle_the_view_on_arrival()` the one place a landing sets the view and syncs the
    check mark, used by the folder landing, the All Inboxes arm, the Label arm and the
    `SavedSearchRan` arm; `load_every_inbox` loading conversations when shown and
    `switch_the_view` loading them on All Inboxes, storing under `all-inboxes`; the
    sentence naming All Inboxes; `chosen_messages`, `spawn_conversation_text_fetch`,
    `conversation_nodes`, `count_a_conversation` and `select_the_conversations_holding_it`
    reading the row's own account, the last found on the branch;
    `tests/all_inboxes_keeps_a_view_of_its_own.rs` at 11; six records at 1, 1, 1, 1, 2 and
    1 red; the pages and the changelog; the version at `1.0.0-alpha.1`; the box ticked on
    the orchestrator's word, 11-12 still reading it.
  - [S] A folder never set heard as conversations on a fresh profile, All Inboxes threaded
    and its view kept when he comes back to it, and a conversation in two of his accounts
    heard as two rows, are his ear's.

### The editors, and what the alpha still owes

Added 2026-09-20 for phase 12, the fifth of the seven groups Pratik agreed on 2026-09-16,
with what the public alpha owes beside it (#78, #64, #71's third point) and the pro licence
as a design (#65). Every requirement here is one GitHub issue, or two the tester joined, in
his words on its `[S]` lines, with the `[D]` lines written on 2026-09-20 by the planner as
proposals in the sense the top of this file gives. Every evidence line was re-taken against
`main` at `0ad66e48` on 2026-09-20, and where a premise moved the evidence line says which
way. The plans are in `.planning/phases/12-the-editors-and-what-the-alpha-still-owes/README.md`.
The two plans moved from phase 11 keep their ids there: LIST-19 (12-02) and LIST-11 (12-03).
FOUND-20 (12-01) sits under phase 9's section beside FOUND-19.

Nothing here has met a real provider except through the tester's Gmail account. Each
requirement's last `[S]` line says what only his ear, his reader, his account, the runner or
the site can settle; the caveat at the top of this file binds every `[D]` line.

- [ ] **ALPHA-01**: The About dialog names the copyright holders and the licence in words held
  to LICENSE, keeps the full version with the build counter, and links to wixen.app and
  wixen.app/support as controls a screen reader names by their address.
  - Evidence: `grep -n 'fn build_about_dialog' src/presentation/wx_app.rs` on 2026-09-20 at
    `0ad66e48`: `:26170`; the dialog holds four `StaticText` lines and OK, the copyright line
    "Copyright 2024-2026 Wixen Mail Contributors"; `sed -n 3p LICENSE`: "Copyright (c) 2026
    Pratik Patel", so the years and the holders disagree today. `grep -rn 'wixen.app' src
    docs README.md --include=*.rs --include=*.md`: only `docs/plans/20260823-earcon-sound-schemes.md`.
    `grep -rn HyperlinkCtrl src --include=*.rs`: nothing, so what the control answers over
    MSAA is unmeasured on this tree. `curl -s -o /dev/null -w '%{http_code}' -A Mozilla/5.0
    --max-time 30 https://wixen.app/` and the same for `/support`: 522 and 522.
  - [S] #78, Pratik on 2026-09-18: "the About dialog carries a copyright line naming Pratik
    Patel and the Wixen Project, with other contributors; the full version of the build;
    links to `wixen.app` and `wixen.app/support`; and a way into the feedback dialog planned
    in #64"; and on #64: the button "is added in the same commit as the dialog, not before,
    so nothing dead sits on About until then".
  - [D] `application::about` holds `COPYRIGHT`, `LICENCE_NAME`, `HOME_PAGE`, `SUPPORT_PAGE`
    and `lines()`; `build_about_dialog` reads them; `LICENSE:3` names the same years and
    holders; two controls after the copyright, one per address, named by the bare address
    on both channels, the kind chosen by a reading over MSAA on the built dialog, each
    opening the browser through `safe_external_url`; the order the two links then OK; no
    Send Feedback button until 12-05 adds it; a reading in
    `tests/the_about_dialog_names_its_owners_and_its_links.rs` holds the texts, the
    LICENSE agreement, the roles and names over MSAA and the absence of the button (12-04).
  - [S] The two controls heard under NVDA, and Enter on one opening his browser, are the
    tester's ear; whether both pages are up before the public alpha is Pratik's (they
    answered 522 on 2026-09-20).

- [ ] **ALPHA-02**: Feedback can be sent from the Help menu and from About: a category, the
  questions that fit it, a payload the person reads before it goes, the log excerpt attached
  by default and redacted, sent as an email from the person's default account to
  support@wixen.app through the program's own sending path and gate.
  - Evidence: `sed -n 7514,7550p src/presentation/wx_app.rs` on 2026-09-20 at `0ad66e48`:
    Help holds Contents, the topics, Load Sample Mailbox, Check for Updates and About, and
    nothing sends anything; `grep -n 'Ctrl+Shift+F' docs/KEYBOARD_SHORTCUTS.md src/presentation/wx_app.rs`:
    nothing, the key is free; `grep -n 'fn queue_for_sending' src/presentation/wx_app.rs`:
    `:17054`, the one `QueuedOutboxMessage` literal in the file (`grep -v '^\s*//'
    src/presentation/wx_app.rs | grep -c 'QueuedOutboxMessage {'` is 1);
    `grep -n 'allowed_for(&account.id).mail' src/presentation/wx_app.rs`: the gate a send
    reads; `grep -n 'pub fn mask_email' src/common/logging.rs`: `:163`, the masking rule
    the log uses; `grep -rn 'nvda.exe\|RtlGetVersion' src --include=*.rs`: nothing, so the
    screen reader and the Windows build are new readers;
    `gh api repos/PratikP1/Wixen-Mail/private-vulnerability-reporting`: enabled;
    `sed -n 173,192p docs/privacy.md`: the table of who this program talks to, which gains
    a row.
  - [S] #64, the tester on 2026-09-16: "There is no support mechanism built into the app
    itself. I'd like users to have a direct way to provide feedback from the app's help menu
    ... I'd like the user to pick categories for the support including 'report a problem',
    'request a feature', etc. Pick other relevant categories. Then, ask for relevant
    information to resolve the issue being reported." Pratik on 2026-09-17 (#71): "the log
    excerpt is attached by default ... a person clears it when the report does not need
    it." Pratik on 2026-09-18: "the report goes as an email from the person's default
    account to support@wixen.app, through the program's own sending path (the outbox, the
    same gate every send passes ...) ... the person sees the exact message before it goes
    ... a copy is kept locally ... if there is no account, or sending is forbidden, the
    dialog says so and offers to copy the report to the clipboard and to open the GitHub
    issue page as the second door; the security category still goes to the private channel
    only. The privacy page's list of what is sent and where (#29) gains support@wixen.app."
  - [D] `application::feedback_report`: `Category` (Problem, Feature, ScreenReaderBarrier,
    Question, Security, Other) with its questions, `Include` defaulting to the version and
    the log excerpt, `Facts`, `compose`, `redact` masking every address and every subject,
    `last_lines`, `where_it_goes`; `service::this_machine`: the Windows build, the display
    language, the screen reader by process name and file version, on the tree's
    `extern "system"` pattern with no new crate (12-05, task 1).
  - [D] `presentation::wx_feedback`: the dialog in Tab order with every control named on both
    channels, the category focused on open, the payload box holding `compose()`'s exact text
    refreshed on every change, Send enabled by `what_the_doors_do` only for a non-security
    category with an account whose `allowed_for(id).mail` is true, the clipboard and GitHub
    doors always, the private reporting page for Security; Send through the queued-row
    function extracted from `queue_for_sending` and called by both, the excerpt and the copy
    under `paths.feedback_dir()`; Help, Send Feedback on `Ctrl+Shift+F`; About's button;
    `ScanTarget::Feedback` in the workflow; a reading in
    `tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs` (12-05, task 2).
  - [D] `docs/privacy.md`'s table gains the address and what a report carries; the shortcuts
    page, the guide and the alpha page say how to send one; #64, #71 and #78 closed from the
    merge (12-05, task 3).
  - [S] The dialog under NVDA is the tester's ear; a report from a real account arriving at
    support@wixen.app is his mailbox's, which must exist and be read; priority for pro
    subscribers waits on #65, by his own words on #64.

- [ ] **ALPHA-03**: The pro licence is designed in a document the tree keeps as a design: what
  is gated, how a key is checked offline, what a lapse does, what the alpha carries, the
  prices and the trial as decided, the merchants compared, and every decision that is
  Pratik's in one table; nothing in the product is gated.
  - Evidence: `grep -rniE 'licen[cs]e key|subscription|entitle' src --include=*.rs` on
    2026-09-20 at `0ad66e48`: three unrelated matches (`allowed.rs:365`, `:377` "entitled to
    refuse"; `answering.rs:884` "a subscription they never made"), so nothing knows a
    licence; `grep -n '^name = "ed25519-dalek"\|^name = "ring"\|^name = "keyring"' Cargo.lock`:
    all three present; `grep -n 'pub fn verify' src/service/update_download.rs`: `:858`, the
    signed-thing-checked-offline pattern; `ls docs/plans/`: four designs named
    `YYYYMMDD-name.md`, where this one goes.
  - [S] #65, the tester on 2026-09-16: "plan a pro license with gated features under pro.
    Multiple account support. Future RSS reader. PGP support. Allow working with multiple
    calendars. Future sound packs for event announcements. Suggest other potential pro
    features." Pratik on 2026-09-16: "a $10 a year supporter licence; a $19 pro licence
    (yearly); a $99 perpetual pro licence; a 60-day trial of pro"; and his merchant table
    of the same day, Paddle recommended and undecided.
  - [D] `docs/plans/20260920-pro-licence.md` (dated the day it is written) with ten
    sections: what it is for and is not; what exists today, each claim with its command; the
    free and pro line as a table with each feature's state in the tree; the licence as a
    signed string checked offline, entered on Settings, kept in the credential store; the
    `Entitlement` seam on `application::allowed`'s pattern with a gated command greyed and
    labelled; a lapse deleting nothing; the prices as decided; the merchant table carried
    whole with its date; the decisions table for Pratik with an empty answer column; what
    follows the decisions. A row in `docs/development/requirements-backlog.md`; a ledger
    `todo` naming the decisions as his; no file under `src/`, `tests/` or `guards/` touched
    (12-11).
  - [S] Every row of the decisions table is Pratik's: the free and pro line, whether several
    accounts are gated at all, the merchant, online revocation, how long a perpetual licence
    carries updates, a trial with no card, whether the supporter tier delivers a real
    licence, how priority support is carried.

- [ ] **EDIT-01**: Every number a person sets in the account editor and in Settings is a spin
  control, the check interval first, and every spin control's typing field has a name on the
  channel a screen reader reads.
  - Evidence: `grep -n 'Check &Interval' src/presentation/wx_account_manager.rs` on
    2026-09-20 at `0ad66e48`: `:1639`, `tf_with_description`, a `TextCtrl`, read back at
    `:1254` with `.parse().unwrap_or(5).clamp(1, 60)`; the `spin` closure at `:1488-1499`
    with a fixed range 0..=3650. `grep -n 'font_size\|default_reminder\|mark_read_after'
    src/presentation/wx_settings.rs`: Font size a `TextCtrl` (`:1109-1112`, saved with
    `.clamp(8, 72)` at `:3397-3402`), Default reminder a `TextCtrl` (`:2472-2473`, `.min(1440)`
    at `:3669-3674`), Mark read after a `Choice` over `reading_habits::MarkRead::ALL`
    (`:1511`, `:1834`); the undo-send hold and the autosave interval already `SpinCtrl`s
    (`:1337`, `:1380`). `grep -rn 'SpinCtrl::builder' src --include=*.rs | grep -v '^\s*//'
    | cut -d: -f1 | sort | uniq -c`: ten sites in four files. `awk -F'|' '$2 >= 405 && $2
    <= 426' .planning/WINDOWS.md`: twelve spinner entries (408, 409, 410, 412, 413, 414,
    419, 420, 421, 422, 424, 425) whose typing field has no name on either channel, with the
    annotation-service remedy written in 408 and the visible-label route measured in 424 as
    naming UI Automation alone. `grep -rn 'UDM_GETBUDDY\|SetHwndPropStr' src --include=*.rs`:
    nothing.
  - [S] #35, the tester on 2026-09-15: "In settings dialog where numbers are expected, spin
    boxes/controls should be used so that users can use up/down arrow keys to make
    changes." #73, the tester on 2026-09-18: the check interval "should be a spin control,
    so Up and Down change it and the bounds are the control's own." Pratik on #35,
    2026-09-18: "Same rule, same reading over MSAA for the buddy edit's name (ledger 408);
    worth fixing together."
  - [D] `names::name_the_spin_control(spin, name)` names the arrows through the existing
    accessible and the typing field through `IAccPropServices::SetHwndPropStr` on the buddy
    from `UDM_GETBUDDY`, three features of the `windows` crate switched on and no crate
    added; every `SpinCtrl::builder` site in the tree calls it; a reading in
    `tests/every_spin_control_names_the_field_a_person_types_in.rs` finds every
    `msctls_updown32` in the built account editor, Settings, the item form and the table
    asker and asserts the buddy's MSAA name equals the arrows' (12-06, task 1).
  - [D] The check interval a spin control 1..=60 with its sentence as its description; Font
    size 8..=72 and Default reminder 0..=1440 as spin controls with the saves' parse-and-clamp
    gone; Mark read after a three-way `Choice` (Immediately, After a number of seconds,
    Never) with a seconds spin enabled for the middle one, the stored string unchanged in
    shape through `MarkRead::parts` and `from_parts`; the ports stay typed and the summary
    says why; the two settings guards in `config.rs` stay green (12-06, task 2).
  - [D] The twelve ledger entries marked fixed on the reading in both halves; the pages say
    which numbers are spin controls and how they are worked; #73 and #35 closed (12-06,
    task 3).
  - [S] A field and its arrows heard with one name, the values said after Up and Down, and
    Mark read after's three entries are the tester's ear; the twelve findings gone from the
    scan is the next push.

- [ ] **EDIT-02**: The contact editor has prefix, middle name and suffix fields; a whole name
  and its parts fill each other without overwriting what the person typed; the birthday is
  a date control; an email address and a phone number are checked without refusing real ones.
  - Evidence: `sed -n 1276,1312p src/presentation/wx_managers.rs` on 2026-09-20 at
    `0ad66e48`: Basic Info holds Name, Given name, Family name, Nickname, Company,
    Department, Job Title, Birthday (`add_panel_field`, a `TextCtrl`, `:1297`), Website,
    Relationship, Avatar URL, Favourite (named outright at `:1305-1310` since `165fd811`);
    no prefix, middle or suffix. `sed -n 591,620p src/data/message_cache/mod.rs`:
    `ContactEntry` has `given_name`, `family_name` and `birthday: Option<String>` and no
    prefix, middle or suffix. `sed -n 795,810p src/data/message_cache/contacts.rs`: vcard
    `N` written as family;given;;; with three empty parts. `sed -n 65,95p
    src/service/google_api.rs`: `GoogleName` carries given and family only. `sed -n 36,60p
    src/service/microsoft_graph.rs`: the contact carries given, surname and no title,
    middle or generation. `sed -n 735,750p src/application/contacts_sync.rs`: why the sync
    stopped guessing (Hopper, van der Berg). `grep -n 'YEAR_LEFT_OUT' src/common/types.rs`:
    `:211`, "--", the marker a birthday without a year is stored with. `grep -n 'pub fn
    is_an_address' src/application/links_in_text.rs`: `:199`, the address rule.
    `grep -rn 'ContactEntry {' src tests --include=*.rs | grep -v 'pub struct' | wc -l`: 114
    literal sites.
  - [S] #40, the tester on 2026-09-15: "There is no prefix/suffix fields with common options.
    There is no field for middle name. On the basic tab, if the user enters the full name in
    the first field, it should be parsed so that the remaining fields ... should
    automatically fill in appropriately. Similarly, if the full name field is not filled in
    and the other fields are subsequently completed, the full name field should be updated
    automatically. The birthday field should be a date similar to other date fields used in
    calendars, tasks, reminders, etc. Are email addresses and phone number validated for
    formatting and country designations for phone numbers?" Pratik on 2026-09-16: "Point 5
    (the unnamed Favourite checkbox) is fixed in 165fd811 with #42; points 1 to 4 and 6 are
    later phases and this issue stays open for them."
  - [D] `application::contact_names` with `guess_parts` and `compose` over titles, suffixes
    and particles, the Hopper and van der Berg cases; `application::phone_countries` with
    every calling code and `with_country`; `name_prefix`, `middle_name` and `name_suffix`
    as columns through `ensure_column_exists` and fields on `ContactEntry`; vcard `N` with
    five parts; `GoogleName` with `honorific_prefix`, `middle_name`, `honorific_suffix`;
    the Graph contact with `title`, `middle_name`, `generation`; each carried both ways
    (12-07, task 1).
  - [D] The editor's Prefix and Suffix as `ComboBox`es with room to type, Middle name, the
    birthday through `build_date_fields` with a no-year position stored as `YEAR_LEFT_OUT`,
    the fills each way guarded by per-field typed flags, an address checked by
    `is_an_address` and a number by `looks_like_a_number` with a country choice beside it,
    every new control named on both channels; a reading in
    `tests/the_contact_editor_fills_the_name_and_its_parts_from_each_other.rs` drives the
    built editor and reads the Favourite box over MSAA (12-07, task 2).
  - [S] The tab heard in order, the fill heard after a name, the no-year position and a
    refusal are the tester's ear; a contact with five name parts round-tripping through
    Google is his account's; whether a phone-number library is wanted beyond the digit rule
    is Pratik's.

- [ ] **EDIT-03**: Event, task and reminder times move in 15, 30 or 60 minute blocks from a
  setting, a new item starts at the next boundary with a default length, the end follows the
  start, and Left and Right move by a minute.
  - Evidence: `sed -n 901,946p src/presentation/wx_item_form.rs` on 2026-09-20 at
    `0ad66e48`: `build_time_fields` with the hour spin and the minute spin 0..=59 stepping
    one, anchored on `now` or the stored `HH:MM`; nothing aligns or sets an end. `grep -n
    'wx_item_form::ask_for' src/presentation/managers.rs`: `:1213` and `:2733`, the one door
    for all three editors. `grep -n 'pub event_length\|event_length_minutes'
    src/data/config.rs`: nothing. `grep -n 'impl crate::event::WindowEvents for SpinCtrl'
    ~/.cargo/registry/src/*/wxdragon-0.9.17/src/widgets/spinctrl.rs`: `:201`, and no
    `set_increment`, so a step is a key handler.
  - [S] #41, the tester on 2026-09-15: "each event should be blocked for 30 minutes by
    default unless the user has indicated otherwise in settings ... The configuration should
    allow for 15 minutes, 30 minutes, or 1 hour ... Pressing up and down arrow keys when
    picking time should move in those blocks. However the user should be able to choose other
    times minutely using left and right arrow keys." His four answers of 2026-09-15: the
    next boundary after now (14:37 gives 15:00); the end moves with the start unless edited
    explicitly; Left and Right stepping single minutes accepted, typing over the selection
    still works; the same for the task and reminder editors.
  - [D] `application::time_blocks` with `Block`, `next_boundary` strictly after now,
    `step_by_block`, `step_by_minute`, `end_after` and `follow`, cases at every boundary of
    the day and the 12-hour face; `event_length_minutes` in `config.rs` defaulting to 30
    with the older-file test, a `Choice` on the Calendar and PIM tab, read where the form
    opens and never captured at startup (12-08, task 1).
  - [D] On the minute control Up and Down by the block and Left and Right by a minute, the
    key consumed, the hour rolling, in all three editors; a new item on the next boundary
    with its end one block later; the end following the start until edited; where the key
    arrives measured first on the built form; a reading in
    `tests/event_times_move_in_blocks.rs` drives the keys and reads the values (12-08,
    task 2).
  - [S] The value spoken after Up and after Left, the end heard following, and the setting's
    list are the tester's ear.

- [ ] **EDIT-04**: Signatures are one set assignable per account, with one default for any
  account that has none, and compose follows the From account.
  - Evidence: `grep -n 'CREATE TABLE IF NOT EXISTS signatures' -A 9 src/data/message_cache/mod.rs`
    on 2026-09-20 at `0ad66e48`: `:1830`, `account_id NOT NULL`, `is_default`, `UNIQUE(account_id, name)`;
    `grep -n 'pub fn ' src/data/message_cache/signatures.rs`: every reader takes an account
    and nothing reads across; `grep -n 'get_default_signature' src/presentation/wx_app.rs`:
    `:16601`, compose reading the active account's default; `sed -n 190,215p
    src/presentation/managers.rs`: the manager reads one account's set and says nothing of
    which; `grep -n 'account_choice' src/presentation/wx_compose.rs`: the From `Choice` at
    `:809-820` with no handler reaching the signature; `grep -n 'pub fn '
    src/application/sign_off.rs`: `split` and `carries_one`, which find the block again;
    `grep -n 'CREATE TABLE IF NOT EXISTS work_done_once' src/data/message_cache/mod.rs`:
    `:2379`, the marker table a once-only pass uses.
  - [S] #43, the tester on 2026-09-15: "Allow signatures to be assigned by email account.
    Default should apply if no signature is assigned to a particular account."
  - [D] `application::signatures::which_signature` (the assignment, else the default, else
    none) and `whether_to_swap` (only a block still equal to the previous signature);
    `signature_assignments` as a new table; `get_every_signature`, `assign`,
    `assignment_for`, `set_the_default` clearing every other row, `signature_for_account`;
    `make_signatures_one_set` once under a marker writing an assignment for every account
    that had a default before clearing all but the oldest (12-09, task 1).
  - [D] The manager over the whole set with a Used by column and one Default box meaning for
    everyone; "Signature:" as a `Choice` on the account's own edit dialog with "Use the
    default" first; compose opening with the From account's signature and swapping the
    block on a From change only while untouched, said at Normal; a reading in
    `tests/a_signature_follows_the_from_account.rs` (12-09, task 2).
  - [S] The manager's columns, the choice and the sentence on a From change are the tester's
    ear; where the choice reads better by ear is his.

- [ ] **EDIT-05**: The Label submenu shows the account's labels by their current names in a
  stored order with the key beside each, leads to a manager that creates, edits and orders
  them, and the keys apply the label the menu shows.
  - Evidence: `sed -n 6918,6946p src/presentation/wx_app.rs` on 2026-09-20 at `0ad66e48`:
    the submenu built once from `tagging::TO_BEGIN_WITH` by index, slots six to nine "Label",
    the comment claiming the names are rewritten on load; `grep -n labels_menu
    src/presentation/wx_app.rs`: the build at `:6927-6941` and the append at `:7346` and no
    rewrite; `sed -n 72,78p src/data/message_cache/tags.rs`: `ORDER BY name`, so Ctrl+N
    applies the alphabetical Nth (Important, Later, Personal, To Do, Work) while the menu
    shows Important, Work, Personal, To Do, Later; `grep -n 'CREATE TABLE IF NOT EXISTS
    tags' -A 7 src/data/message_cache/mod.rs`: `:1802`, no position column; `grep -n
    'ID_TAG_MGR' src/presentation/wx_app.rs`: `:7449` "Ta&gs..." on Tools; `grep -n 'Tag
    Manager' src/presentation/wx_managers.rs`: `:3659`, Add and Edit, no move; `grep -n
    'reordering::moved' src/application/account_order.rs src/application/favourites.rs`: the
    gesture's two callers.
  - [S] #48, the tester on 2026-09-15: "The action menu allows the user to apply labels.
    Currently there are five preconfigured labels. Four additional slots are available but
    are not assigned. There is NO UI for creating these labels. Create a UI for this
    functionality for additional labels as well as for editing and moving labels' order."
  - [D] `position` on `tags` through `ensure_column_exists`, `get_tags_for_account` ordered
    by position then name, a once-only numbering in name order under a marker, `move_tag`
    through `reordering::moved`; `tagging::what_the_menu_says` giving each label its line and
    key for the first nine (12-10, task 1).
  - [D] The submenu rebuilt from the labels on load and after the manager closes, ending
    with Edit Labels; Tools says Labels and the manager is the Label Manager with Move Up,
    Move Down and a Key column; a reading in
    `tests/the_label_menu_says_the_labels_an_account_has.rs` reads the items after a rename
    and a move and `at_number` against them (12-10, task 2).
  - [S] The submenu's items with their keys, Edit Labels, a move said and the Key column are
    the tester's ear.

### New features, most from the Outlook gap audit

Added 2026-09-20 for phase 13, the sixth of the seven groups Pratik agreed on 2026-09-16.
Written as requirements only, from the issues as they stand on 2026-09-20; no plan exists,
because a plan written now would rot against a tree phase 12 changes, and the order inside
the group is the planner's for Pratik to confirm before planning. Each `[D]` line is the
issue's own "what closes it" condensed, and each is re-read against the tree when the phase
is planned. Nothing here has met a real provider.

- [ ] **GAP-01**: File, Print (Ctrl+P) prints a message or the item under the cursor on
  every surface that shows one, through the native dialog, the header lines and the text.
  - Evidence: `gh issue view 45 --json title,state` on 2026-09-20: open; `grep -rn 'Ctrl+P'
    src/presentation/wx_app.rs docs/KEYBOARD_SHORTCUTS.md`: nothing bound (re-take when
    planned).
  - [S] #45, the tester on 2026-09-15: "Add print functionality."
  - [D] File, Print on the mail surfaces and the other modules' items; the route (the native
    printout or the browser's print) decided by a measurement; the shortcuts page and the
    guide.
  - [S] What a printed page looks like is a sighted reader's; whether the dialog is worked by
    keyboard is the tester's ear.

- [ ] **GAP-02**: Edit, Undo and Redo act on the focused text everywhere, and then on actions
  on items with the item named.
  - Evidence: `gh issue view 47 --json state` on 2026-09-20: open; the composer's own
    Ctrl+Z and Ctrl+Y exist and the Edit menu holds none (the issue's reading; re-take when
    planned).
  - [S] #47, the tester on 2026-09-15, "no general Undo/Redo".
  - [D] Edit, Undo and Redo for the focused control on every surface first; an undo of a
    delete, a move and a mark with the item named as its own plan; the shortcuts page.
  - [S] Whether the undone action is heard as undone is the tester's ear.

- [ ] **GAP-03**: A PGP key manager lists the keys, imports a public key, removes and exports
  one, and says where a person reads it what the keys can and cannot do here.
  - Evidence: `gh issue view 49 --json state` on 2026-09-20: open; one private key with no
    passphrase, used only to open inline PGP (the issue's reading; re-take when planned).
  - [S] #49, the tester on 2026-09-15, "no way to manage PGP keys".
  - [D] The manager under Tools; the limits said on the manager and the pages; passphrase
    keys a decision of their own, listed for Pratik.
  - [S] A real key from a real correspondent is his.

- [ ] **GAP-04**: The reader shows and announces a meeting invitation with its answers, and a
  cancellation or an update reaches the calendar.
  - Evidence: `gh issue view 50 --json state` on 2026-09-20: open.
  - [S] #50, from the Outlook gap audit of 2026-09-15, in the tester's list.
  - [D] The invitation's part shown and said before the body with Accept, Tentative and
    Decline; a cancellation removing and an update moving the event; the answer sent through
    the outbox and the gate.
  - [S] An invitation from a real organiser and its answer arriving are his account's.

- [ ] **GAP-05**: S/MIME-encrypted and PGP/MIME mail is read, a PGP signature is verified and
  said, and a message can be sent signed and encrypted.
  - Evidence: `gh issue view 52 --json state` on 2026-09-20: open.
  - [S] #52, from the audit, in the tester's list.
  - [D] Reading first, then verifying, then sending; each said in the reader's sentence the
    way the unverified-signature sentence is said today; the key manager (GAP-03) before it.
  - [S] A message from a real correspondent's key is his account's.

- [ ] **GAP-06**: A sender can be reported as junk to a provider that takes reports, and a
  block moves the sender's existing mail.
  - Evidence: `gh issue view 54 --json state` on 2026-09-20: open.
  - [S] #54, from the audit, in the tester's list.
  - [D] Report Junk on the Action menu for a provider with an endpoint, said when there is
    none; a block that moves what is already here, said with the count; through the gate.
  - [S] Whether Gmail takes the report is his account's.

- [ ] **GAP-07**: Directory lookup answers from Graph for a Microsoft account and from an LDAP
  directory that needs a sign-in.
  - Evidence: `gh issue view 55 --json state` on 2026-09-20: open; `ldap3` is in
    `Cargo.toml` with rustls.
  - [S] #55, from the audit, in the tester's list.
  - [D] Graph people search while typing an address on a Microsoft account; an LDAP
    directory with a bind, its sign-in kept in the credential store; the privacy page's row.
  - [S] A real directory is his to set up.

- [ ] **GAP-08**: Free/busy asks a Google source and every source an account has, and shows
  guests' times in their zones.
  - Evidence: `gh issue view 57 --json state` on 2026-09-20: open; free/busy asks one
    source per account today (#65's reading of the tree).
  - [S] #57, from the audit, in the tester's list.
  - [D] The Google free/busy endpoint; every calendar source an account has asked; a guest's
    zone shown beside the time.
  - [S] A real guest's free/busy is his account's.

- [ ] **GAP-09**: Saved searches can be reordered, given a key, and saved from scratch, and
  the two pages that drifted are corrected.
  - Evidence: `gh issue view 58 --json state` on 2026-09-20: open.
  - [S] #58, from the audit, in the tester's list.
  - [D] The reordering gesture the tree has; a key per search on the pattern labels take in
    EDIT-05; Save as Search from an empty box; the pages corrected by dating.
  - [S] The order and the key heard are the tester's ear.

- [ ] **GAP-10**: Several identities per account, the first step to shared mailboxes and
  delegation.
  - Evidence: `gh issue view 59 --json state` on 2026-09-20: open.
  - [S] #59, from the audit, in the tester's list.
  - [D] An identity (a From name and address) per account beyond the first, offered in
    compose's From list; shared mailboxes and delegation as their own later work, said.
  - [S] A shared mailbox on a real provider is his.

- [ ] **GAP-11**: Quick Steps: a named multi-action command on a key, over the selected
  messages.
  - Evidence: `gh issue view 60 --json state` on 2026-09-20: open; the selection and the
    set commands exist since 11-07.
  - [S] #60, from the audit, in the tester's list.
  - [D] A Quick Step as a rule's actions run by hand over the selection, named, with a key,
    on the pattern 11-07's set commands use; one sentence saying what it did.
  - [S] Whether a step is worked by keyboard and heard as one act is the tester's ear.

- [ ] **GAP-12**: A rule can be run over a folder on demand, saying first how many messages it
  would touch.
  - Evidence: `gh issue view 61 --json state` on 2026-09-20: open; rules run once when mail
    arrives (11-10's comment on #62).
  - [S] #61, from the audit, in the tester's list.
  - [D] Run Rule Now on the rule editor and the Action menu; the count said before the run
    with a way to stop; the run through the same arms a check uses.
  - [S] Whether the count and the result read as one act is the tester's ear.

- [ ] **GAP-13**: Mail export as a bare mbox or loose eml files, msg read, and pst export, or
  each said plainly to be out.
  - Evidence: `gh issue view 53 --json state` on 2026-09-20: open; Pratik's comments of
    2026-09-17: points 1 to 3 and 7 landed in `06fdc9b7` and `8eba6a38`, "4 to 6 are later
    work".
  - [S] #53, from the audit, in the tester's list; Pratik on 2026-09-17: "Points 4 to 6 (a
    bare .mbox or loose .eml export, .msg, .pst export) are later work."
  - [D] Each of the three built or refused with a sentence on the page saying which and why;
    a refused format is not a menu item.
  - [S] A real Outlook file is his.

### The real-account proofs

Added 2026-09-20 for phase 14, the seventh of the seven groups Pratik agreed on 2026-09-16,
the ones that need his account. Written as requirements only; no plan exists, for the same
reason as phase 13 and because the steps a tester follows are written against the build that
carries phase 12. Sending is proven; the rest is not.

- [ ] **REAL-01**: Adding a Gmail account brings its calendars, contacts and tasks, and
  Refresh in each module brings what the account has.
  - Evidence: `gh issue view 22 --json state,comments` on 2026-09-20: open; the tester on
    2026-09-15: Refresh in Calendar and in Contacts brings nothing; read from his profile on
    2026-09-16 with his agreement: two calendar rows, neither `google:`, `contacts` 0 rows,
    `calendar_events` 0, `sync_state` 0, against 12,872 messages then cached, so mail reached
    the server and the PIM sync did not or wrote nothing; the log that would say which was
    empty then (#66) and is at Debug since 11-04.
  - [S] #22, the tester on 2026-09-15: "Setting up a Gmail account does not carry over the
    corresponding calendars, contacts, etc."
  - [D] The profile's log at the moment of a Refresh read first, with his agreement; the
    request the sync makes and what Google answers traced against the loopback servers and
    then his account; the sync run on account creation as well as on Refresh.
  - [S] What Google answers his account is his account's.

- [ ] **REAL-02**: The five write paths, copy within an account, copy across two, move within
  an account, move across two, and delete, are each proved against a real account and
  recorded, and the gate's default moves per path on Pratik's word.
  - Evidence: `gh issue view 63 --json comments` on 2026-09-20: sending proven 2026-09-18
    ("Sending works. It's confirmed."); Pratik on 2026-09-18: "move and copy are tested
    separately ... four write paths, not three"; on 2026-09-19: the move, copy and delete
    proofs are re-taken after `fa20d04a` (11-07.1) and `2526b31f` (11-07.2), since they
    complete here first now; ledgers 187, 191, 376, 546 and 547 name what only a live
    account answers.
  - [S] #63: "Prove send, move and delete against a real account, then let the gate say
    so"; Pratik's four comments as quoted.
  - [D] A steps page per path against the build that carries phase 12, each run by him and
    recorded on the issue with the date and the build; a move made with the network off and
    replayed, one replayed after a restart, a message another client changed meanwhile;
    `application::allowed`'s default moved per proven path and the four warning surfaces
    reworded, on his word.
  - [S] Every proof is his account's; the order of the four lines is his to confirm.

## v2 Requirements

Deferred out of this milestone, with the reason. Each was in the inventory's "not built"
section and is real work; none is declined.

| Requirement | Reason for deferral |
|---|---|
| Gmail X-GM-THRID conversations and X-GM-RAW server-side search | Blocked on the IMAP library, not on this codebase. Roadmap Phase 2 and `docs/changelog.md` both record it that way. Corrected 2026-09-20 by 11-12's read: `X-GM-THRID` landed on 2026-09-19 in 11-08.1 (LIST-23), since this code reads the server's attributes itself and the library no longer stood in the way; only `X-GM-RAW` stays deferred, and the pages that said the library blocked both are corrected by dating. |
| The Exchange path described in `docs/plans/20260726-mail-at-scale.md` | The Microsoft work that shipped went through Graph for contacts, calendar and tasks. With EWS declined, this section proposes a path nothing needs. |
| JMAP | `docs/development/requirements-backlog.md`, future, priority Low. |
| Plugin and extension system | `docs/development/requirements-backlog.md`, future, priority Low. |
| Setting Wixen Mail as the actual Windows default mail client | Windows does not allow a program to make itself default. `src/service/default_apps_registration.rs` registers what it can and the product already says plainly that it cannot set the default. |
| A mail message somebody asked to be told about later joins the due window as a fourth kind of row, "Mail:" first | Deferred out of version 1 on 2026-09-14, Pratik's words: "plan for future expansion of this functionality (post version 1) which will enable us to set notifications for mail that will let the user decide to tackle individual mail later on." The model is shaped for it by 06-09: `due::Kind` with three variants and a doc comment that is the seam, an `Identity` a feed composes, a hold table keyed by the kind's word so a fourth kind is a fourth word. See "Version 2's second seam" in `.planning/phases/06-how-the-application-speaks/README.md` and the doc comment on `Kind` in `src/application/due.rs`. No requirement id, because none of FEEDBACK-01 to 03 is about it. |

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
| WRITE-02 | Phase 4 | Complete |
| WRITE-03 | Phase 4 | Complete |
| READ-01 | Phase 4 | Pending |
| READ-02 | Phase 4 | Pending |
| READ-03 | Phase 4 | Complete |
| PIM-01 | Phase 5 | Pending |
| PIM-02 | Phase 5 | Pending |
| PIM-06 | Phase 5 | Pending |
| PIM-03 | Phase 5 | Pending |
| PIM-04 | Phase 5 | Pending |
| PIM-07 | Phase 5 | Pending |
| PIM-08 | Phase 5 | Pending |
| PIM-05 | Phase 5 | Complete |
| FEEDBACK-01 | Phase 6 | Pending |
| FEEDBACK-02 | Phase 6 | Complete |
| FEEDBACK-03 | Phase 6 | Complete |
| SHIP-01 | Phase 7 | Pending |
| SHIP-02 | Phase 7 | Pending |
| SHIP-03 | Phase 7 | Complete |
| SHIP-04 | Phase 7 | Complete |
| SHIP-05 | Phase 7 | Pending |
| SHIP-06 | Phase 7 | Complete |
| PERF-01 | Phase 8 | Complete, on the reading the evidence states, pending Pratik's word |
| PERF-02 | Phase 8 | Complete |
| PERF-03 | Phase 8 | Complete |
| PERF-04 | Phase 8 | Complete, on the reading the evidence states, pending Pratik's word |
| PERF-05 | Phase 8 | Complete |
| PERF-06 | Phase 8 | Complete |
| PERF-07 | Phase 8 | Revised, open: the whole-tree run was not made, its cost is written down, two areas ran |
| FOUND-01 | Phase 9 | Complete on the tree side, 09-01 at `c0606807`; the dispatch is Pratik's |
| FOUND-02 | Phase 9 | Complete, 09-02 at `a3554483`; his profile is the tester's |
| FOUND-03 | Phase 9 | Complete, 09-02 at `a3554483`; his cache is the tester's |
| FOUND-04 | Phase 9 | Complete, 09-03 at `f58b9271` |
| FOUND-05 | Phase 9 | Complete, 09-03 at `f58b9271`; ledger 155's listening question open |
| FOUND-06 | Phase 9 | Complete, 09-04 at `c928cae4`; the listening pass is the tester's |
| FOUND-07 | Phase 9 | Complete, 09-04 at `c928cae4` |
| FOUND-08 | Phase 9 | Complete, 09-05 at `165fd811` and 11-02 on 2026-09-18, on Accessibility run 35336142914's walk of the five editors, 0 without a name on each; the walk still crashes here, ledger 390; the tester's listening pass on the two editors stays his, ledger 490 |
| FOUND-09 | Phase 9 | Complete, 09-06 at `de58771a` and 11-02 on 2026-09-18, on the tester's ear (#33 closed on his word); the runner's corrected case is pending the next push of `main`, ledger 531 |
| FOUND-10 | Phase 9 | Complete, 09-07 at `f990d023`; no real key or signed message met |
| FOUND-11 | Phase 9 | Complete, 09-08 at `06fdc9b7` and 09-10 at `8eba6a38`; points 4 to 6 of #53 later work, no real data file read |
| FOUND-12 | Phase 9 | Complete, 09-09 at `a8b26596`; whether it feels immediate is the tester's |
| FOUND-13 | Phase 10 | Complete, 10-01.1 at `d020aa60`; whether NVDA says "check box" and the new state on Space is the tester's |
| FOUND-14 | Phase 10 | Complete, 10-01.1 at `d020aa60`; whether NVDA names the control after Ctrl+Tab is the tester's |
| FOUND-15 | Phase 10 | Complete, 10-02.1 at `c0505f68`; whether All Inboxes opens in the chosen order after a visit to a folder, by ear, is the tester's |
| FOUND-16 | Phase 10 | Complete, 10-02.2 at `44bff634`; one installer built from `main` reads `1.0.0-alpha.1+114.g44bff634` with file version `1.0.0.14114`; whether it installs over the alpha.1 build as an upgrade is settled by a machine, ledger 517 |
| MAIL-01 | Phase 10 | Complete, 10-01 at `d8e887d6` and 10-05 at `b477e8c9`; whether Gmail tolerates the download is the tester's account's, ledger 11, 72 and 523 |
| MAIL-02 | Phase 10 | Complete, 10-02 at `48536d31`; whether his folder reads as one list is the tester's, ledger 515 |
| MAIL-03 | Phase 10 | Complete, 10-01 at `d8e887d6`, 10-03 at `c4203632` and 10-05 at `b477e8c9`; whether Gmail tolerates the text in chunks is the tester's account's, ledger 11, 519 and 523 |
| MAIL-04 | Phase 10 | Complete, 10-01 at `d8e887d6` and 10-06 at `2da50b6b`; whether Gmail drops the watch and the restart carries mail over hours is the tester's account's, ledger 64, 65, 67, 525 and 526 |
| MAIL-05 | Phase 10 | Complete, 10-04 at `19a10706`; what each level sounds like is a listening pass and the tester's, ledger 521 |
| FOUND-17 | Phase 11 | Complete, 11-01 at `316ea755`; whether the runner keeps en-AU is the next push of `main`, Pratik's, ledger 530 |
| FOUND-18 | Phase 11 | Complete, 11-02 at `1c0e9b0b`; whether the sign-in line is heard whole and which tab the corrected case's first Right reaches are the next push of `main`, Pratik's, ledger 531 |
| FOUND-19 | Phase 11 | Complete, 11-06.3 at `1a973b46`; the harness's unset at `d5c3483e`, the two cases red at `cdf04ff8`, the suite run under this repository's absolute git dir and an absolute index copy with nothing moved; no commit made from a linked worktree |
| FOUND-20 | Phase 12 | Pending, 12-01; the run at the next push of `main` is Pratik's |
| LIST-01 | Phase 11 | Complete, 11-03 at `70f4737b`; whether a kept folder is heard as checked, the new state after Space, the level, the title and the All Mail sentence are the tester's ear, ledger 533 |
| LIST-02 | Phase 11 | In progress, 11-04 at `03513fd0`: the rule, the lines, the guard and the pages held; the two size rows owed, ledger 537; whether the lines are the ones a report needs is the tester's next report, ledger 535; #64's half stays #64's. Read again 2026-09-20 by 11-12: the rows are still owed, the box stays open on that clause |
| LIST-03 | Phase 11 | Complete, 11-05 at `5c82f680`; reopened 2026-09-18 on the tester's word and amended for 11-05.1, the first Space starting no clock, held 2026-09-18 by 11-05.1 at `b3ab5d51`; whether the unread count survives a walk through his inbox by ear, and whether the second Space and not the first moves it, are the tester's ear, ledger 539 |
| LIST-04 | Phase 11 | Complete, 11-06 at `fe143d46` and 11-07 at `b35a40cd`; the label heard, the word after M, the list not jumping and the thread marked from its row are the tester's ear, ledger 540 and 544 |
| LIST-05 | Phase 11 | Complete, 11-07 at `b35a40cd`; NVDA's selected and not selected, the count after Ctrl+A, one sentence after a command over many and the refusal above the bound are the tester's ear, ledger 544 |
| LIST-06 | Phase 11 | Complete, 11-08 at `75c211fe`; the sender heard first on his thread rows, the preview and the window on that message, and a conversation's text arriving from Gmail on landing are the tester's ear's and account's (ledger 548) |
| LIST-07 | Phase 11 | Complete, 11-09 at `bd5f6929`; the row heard whole and once on the key, arrowing quiet under the NVDA profile, and what Narrator and JAWS need are the tester's ear's (ledger 550) |
| LIST-08 | Phase 11 | Complete, 11-10 at `39d53503`; the phrase heard first on a row, the sound once after a check with several matches, and the Labels column read as part of the row are the tester's ear (ledger 556) |
| LIST-09 | Phase 11 | Complete, 11-11 at `f497785f`; a shown picture, a passed-over one, the link's words and the sentence about tracking pixels are the tester's reader (ledger 558) |
| LIST-10 | Phase 11 | Complete, 11-11 at `f497785f`; whether the page is clear to the person it is for is his |
| LIST-11 | Phase 11, then phase 12 | Open, 11-13 deferred to the front of the next phase on Pratik's decision of 2026-09-20 under his token budget; nothing landed; the phase closed without it on 2026-09-20; moved later that day to phase 12 as 12-03, whose merge ticks it |
| LIST-12 | Phase 11 | Complete, 11-06.1 at `0ed2c1a1`; whether NVDA reads the landed row once after Delete and not again after the re-read is the tester's ear, ledger 541 |
| LIST-13 | Phase 11 | Complete, 11-09.1 at `517a2a4c`; whether the row is heard once with the tone, and the tone alone with the status bar off, is the tester's ear (ledger 552) |
| LIST-14 | Phase 11 | Complete, 11-06.1 at `0ed2c1a1`; whether "Delete" once and the landed row are enough by ear, and the refusal heard on a failure, are the tester's, ledger 542 |
| LIST-15 | Phase 11 | Complete, 11-04.1 at `70d84bc5`; whether Alt+A lands on the list and NVDA says the landing in both views, and whether the reader's way back through the accelerator fires, are the tester's ear, ledger 538 |
| LIST-16 | Phase 11 | Complete, 11-09.1 at `517a2a4c`; the sounds heard again after a real device change, and what silenced them after hours, are the tester's machine's (ledger 553) |
| LIST-17 | Phase 11 | Complete, 11-09.2 at `4d9a41d2`; whether the rows now say the message is the tester's ear (ledger 554) |
| LIST-18 | Phase 11 | Complete, 11-11.3 at `6e23656b` (the line guard, the refused post and its log line, the released style, the reading over the real page, the pages); which state the tester was in, and what is heard, are his ear's (ledger 565) |
| LIST-19 | Phase 11, then phase 12 | In progress: 11-11.1 merged at `8340e5e6` (the setting, the menu, the activation, the message view, the privacy page; ledger 560 for the ear); 11-11.2, the separate window, deferred to the front of the next phase on Pratik's decision of 2026-09-20; the program's own "next build" sentences are ledger 566; moved later that day to phase 12 as 12-02, whose merge ticks it and fixes 566 |
| LIST-20 | Phase 11 | Complete, 11-06.2 at `116968fb`; whether NVDA reads the landed row once on Tab and on F6, and not twice, is the tester's ear (ledger 543) |
| LIST-21 | Phase 11 | Complete, 11-07.1 at `fa20d04a`; a replayed move against a real server after a restart, a message another client changed meanwhile, and #63's proofs re-taken are the tester's account (ledger 546) |
| LIST-22 | Phase 11 | Complete, 11-07.2 at `2526b31f`; what a real destination does with a message it already holds, Gmail's treatment of an appended message, and #63's crossing proofs re-taken are the tester's accounts' (ledger 187, 547) |
| LIST-23 | Phase 11 | Complete, 11-08.1 at `76897058`; his split threads becoming one after the next check, the row's count matching Gmail's, and the once-only pass answered by a real Gmail are his account's (ledger 549) |
| LIST-24 | Phase 11 | Complete, 11-10.1 at `be97ed86`; the chapter address in NVDA's link list, a description's address heard as a link to its site, and a refused link's note heard beside its words are the tester's ear (ledger 557) |
| LIST-25 | Phase 11 | Complete, 11-11.0 at `fab0ecea`; the newsletter heard once under NVDA with no table and no grouping announced, the subtitle where the sender's line stands, the subject once and the sender once, and another newsletter of his choosing the same way are the tester's ear (ledger 559) |
| LIST-26 | Phase 11 | Complete, 11-11.1.1 at `051c3529`; the delay changed in Settings and a message marked after the new wait without a restart, the list's dates following a save at once, and the two sentences heard under their controls are the tester's ear (ledger 561) |
| LIST-27 | Phase 11 | Complete, 11-11.1.2 at `ae0fa4d2` (the setting, the rule, the landing, the box, the pages) and 11-11.1.3 at `1e39650a` (All Inboxes' own view, the every-inbox listing, the row's own account, the sentence, the pages); a folder never set heard as conversations (ledger 562), All Inboxes threaded and its view kept and a two-account conversation as two rows (ledger 563), are his ear's |
| ALPHA-01 | Phase 12 | Pending, 12-04 |
| ALPHA-02 | Phase 12 | Pending, 12-05 |
| ALPHA-03 | Phase 12 | Pending, 12-11; every decision in its table is Pratik's |
| EDIT-01 | Phase 12 | Pending, 12-06 |
| EDIT-02 | Phase 12 | Pending, 12-07 |
| EDIT-03 | Phase 12 | Pending, 12-08 |
| EDIT-04 | Phase 12 | Pending, 12-09 |
| EDIT-05 | Phase 12 | Pending, 12-10 |
| GAP-01 | Phase 13 | Not planned, 2026-09-20 |
| GAP-02 | Phase 13 | Not planned, 2026-09-20 |
| GAP-03 | Phase 13 | Not planned, 2026-09-20 |
| GAP-04 | Phase 13 | Not planned, 2026-09-20 |
| GAP-05 | Phase 13 | Not planned, 2026-09-20 |
| GAP-06 | Phase 13 | Not planned, 2026-09-20 |
| GAP-07 | Phase 13 | Not planned, 2026-09-20 |
| GAP-08 | Phase 13 | Not planned, 2026-09-20 |
| GAP-09 | Phase 13 | Not planned, 2026-09-20 |
| GAP-10 | Phase 13 | Not planned, 2026-09-20 |
| GAP-11 | Phase 13 | Not planned, 2026-09-20 |
| GAP-12 | Phase 13 | Not planned, 2026-09-20 |
| GAP-13 | Phase 13 | Not planned, 2026-09-20 |
| REAL-01 | Phase 14 | Not planned, 2026-09-20; needs Pratik's account |
| REAL-02 | Phase 14 | Not planned, 2026-09-20; sending proven 2026-09-18, the other four lines his account's |

**Coverage:**

- v1 requirements: 119 total
- Mapped to phases: 119
- Unmapped: 0

**Re-taken 2026-09-20, later.** This block said 95 and 95 from the morning until phase 12
was planned and phases 13 and 14 were written as entries. Counted with the same command as
below, which gives 119 at `0ad66e48` plus this edit with `FOUND-20`, `ALPHA-01` to
`ALPHA-03`, `EDIT-01` to `EDIT-05`, `GAP-01` to `GAP-13` and `REAL-01` and `REAL-02` in, and
the traceability table above has 119 rows.

**Re-taken 2026-09-20.** This block said 93 and 93 from the afternoon of 2026-09-19 until
#91 and #92 were taken by the inserted 11-11.1.1, 11-11.1.2 and 11-11.1.3. Counted with the
same command as below, which gives 95 at `390a580c` plus this edit with `LIST-26` and
`LIST-27` in, and the traceability table above has 95 rows.

**Re-taken 2026-09-19, in the afternoon.** This block said 92 and 92 from the morning until
#90 was taken by the inserted 11-11.0. Counted with the same command as below, which gives
93 at `f92871bb` plus this edit with `LIST-25` in, and the traceability table above has 93
rows.

**Re-taken 2026-09-19, later.** This block said 90 and 90 from the morning until two issues
filed that morning (#88, #89) were taken by two inserted plans (11-08.1, 11-10.1). Counted
with the same command as below, which gives 92 at `38ebcb86` plus this edit with `LIST-23`
and `LIST-24` in, and the traceability table above has 92 rows.

**Re-taken 2026-09-19.** This block said 89 and 89 from the night of 2026-09-18 until Pratik
overruled 11-07.1's decision 29 and #86's second half became 11-07.2. Counted with the same
command as below, which gives 90 at `6911018d` plus this edit with `LIST-22` in, and the
traceability table above has 90 rows.

**Re-taken 2026-09-18, in the night.** This block said 86 and 86 from the evening until three
more issues (#85, #86, #87) were taken by three inserted plans (11-06.2, 11-06.3, 11-07.1).
Counted with the same command as below, which gives 89 at `4d9f14bf` plus this edit with
`FOUND-19`, `LIST-20` and `LIST-21` in, and the traceability table above has 89 rows.

**Re-taken 2026-09-18, in the evening.** This block said 80 and 80 from the afternoon until six
more issues filed that evening (#79 to #84) were taken by two tasks added to plans not yet
executed (11-06.1, 11-09.1) and five inserted plans (11-04.1, 11-09.2, 11-11.1, 11-11.2,
11-11.3). Counted with the same command as below, which gives 86 at `eb5d8517` plus this edit
with `LIST-14` to `LIST-19` in, and the traceability table above has 86 rows.

**Re-taken 2026-09-18, later the same day.** This block said 77 and 77 from the morning until
three issues filed that afternoon (#75, #76, #77) were taken by inserted plans. Counted with
the same command as below, which gives 80 at `08197657` plus this edit with `LIST-11` to
`LIST-13` in, and the traceability table above has 80 rows.

**Re-taken 2026-09-18.** This block said 65 and 65 from 2026-09-17 until phase 11 was planned.
Counted with the same command as below, which gives 77 at `744d05ef` plus this edit with the
ten `LIST` requirements and `FOUND-17` and `FOUND-18` in, and the traceability table above has
77 rows. The ten are the fourth of Pratik's seven groups with #70 and #71 in front; the two
`FOUND` ones are what the morning's push of `744d05ef` showed, and sit in phase 9's section
beside FOUND-13 to FOUND-16 for the reason given there, though phase 11 owns them.

**Re-taken 2026-09-17, later still.** This block said 63 and 63 from the middle of the day
until phase 10's inserted plans 10-02.1 and 10-02.2 were written. Counted with the same
command as below, which gives 65 at `c5ee5085` plus this edit with `FOUND-15` and `FOUND-16`
in, and the traceability table above has 65 rows. One is a defect the tester found in
`1.0.0-alpha.1` on its second day and one is Pratik's decision about what a build carries;
both sit in phase 9's section beside the requirements they follow from, though phase 10 owns
them.

**Re-taken 2026-09-17, later the same day.** This block said 61 and 61 from the morning until
phase 10's inserted plan 10-01.1 was written. Counted with the same command as below, which
gives 63 at `f3be1ef5` plus this edit with `FOUND-13` and `FOUND-14` in, and the traceability
table above has 63 rows. The two are regressions of FOUND-12's fix, found in `1.0.0-alpha.1`
on the second day of testing, and sit in phase 9's section beside it though phase 10 owns them.

**Re-taken 2026-09-17.** This block said 56 and 56 from 2026-09-16 until phase 10 was planned.
Counted with the same command as below, which gives 61 at `7d57cd49` plus this edit with the
five `MAIL` requirements in, and the traceability table above has 61 rows. The five come from
the same first day of testing as the twelve `FOUND` ones; "Where these came from" says so.

**Re-taken 2026-09-16.** This block said 44 and 44 from 2026-09-04 until phase 9 was planned.
Counted with the same command as below, `grep -c '^- \[[ x]\] \*\*[A-Z]\+-[0-9]\+\*\*'
.planning/REQUIREMENTS.md`, which gives 56 at `b24fb991` with the twelve `FOUND` requirements
in (57 for part of the same day, while a `FOUND-13` for the withdrawn #66 existed), and the
traceability table above has 56 rows. The twelve are the first requirements in this file
that did not come from `.planning/intel/built-and-left.md`; the section "Where these came
from" says where they did.

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

**Added 2026-09-16.** `FOUND-01` to `FOUND-12` trace to no row of
`.planning/intel/built-and-left.md`. Each traces to one GitHub issue, or two sharing a cause,
among #20 to #63, filed on 2026-09-15 from Pratik's first day of testing build
`0.125.1+g3e633252` and from the audit of the 2026-08-27 Outlook gap report made the same day.
The issue number is in each requirement's `[S]` line. The 44 above are unchanged; the total
is 56.

**Added 2026-09-17.** `MAIL-01` to `MAIL-05` trace to one GitHub issue each, #20, #24, #23,
#37 and #38, from the same day; the third of Pratik's seven groups. The total is 61.

**Added 2026-09-17, later.** `FOUND-13` and `FOUND-14` trace to #67 and #68, filed that day
against `1.0.0-alpha.1` from the second day of testing, both regressions of the fix FOUND-12
chose; they belong to none of the seven groups and are taken by phase 10's inserted plan
10-01.1. The total is 63.

**Added 2026-09-17, later still.** `FOUND-15` traces to #69, filed that day against
`1.0.0-alpha.1` (`59c5b6a4`), a sort the combined view forgets, taken by phase 10's inserted
plan 10-02.1; `FOUND-16` traces to no issue and to Pratik's decision of 2026-09-17 in
conversation that builds handed to testers carry an ordered counter after the plus, taken by
the inserted plan 10-02.2. Neither belongs to the seven groups. The total is 65.

**Added 2026-09-18.** `LIST-01` to `LIST-10` trace to one GitHub issue each, #70, #71, #25, #27,
#30, #31, #26, #62, #28 and #29, the fourth of Pratik's seven groups with #70 (the second day
of testing) and #71 (his decision of 2026-09-17) in front. `FOUND-17` traces to no issue and
to CI run 35336142985 on `744d05ef`, a regression of FOUND-02's fix; `FOUND-18` to NVDA run
35336142908 and Accessibility run 35336142914 on the same push, and to guardrail 4. Neither
belongs to the seven groups. The total is 77.

**Added 2026-09-20.** `LIST-26` traces to #91, filed that day and taken by the inserted
11-11.1.1 between 11-11.1 and 11-11.2; `LIST-27` traces to #92, Pratik's decisions of that
day with his amendment, taken by the inserted 11-11.1.2 and 11-11.1.3 after 11-11.1.1, two
plans for one requirement the way #80's are, because the setting's screen half and the
list's cache half share no file but the window. The total is 95.

**Added 2026-09-19, in the afternoon.** `LIST-25` traces to #90, filed that afternoon and
taken by the inserted 11-11.0 between 11-11 and 11-11.1. The total is 93.

**Added 2026-09-19, later.** `LIST-23` and `LIST-24` trace to #88 and #89, filed that morning
and taken by inserted plans 11-08.1 (after 11-08, whose row message a re-threading moves)
and 11-10.1 (after 11-10 and before 11-11, so 11-11.1's activation covers a made link).
The total is 92.

**Added 2026-09-19.** `LIST-22` traces to #86's second half, Pratik's comment of that day
overruling 11-07.1's decision to keep a move across accounts server-first, taken by 11-07.2
between 11-07.1 and 11-08. The total is 90.

**Added 2026-09-18, in the night.** `LIST-20` and `LIST-21` trace to #87 and #86, filed after
the evening's six and taken by inserted plans 11-06.2 (beside 11-06.1, which was at three
tasks) and 11-07.1 (after 11-07, whose delete of a set it changes, and before 11-08);
`FOUND-19` traces to #85, the gate's own hazard found while the evening's plans were
committed, taken by 11-06.3 and placed under phase 9's section beside FOUND-17 and FOUND-18
as a defect in what CI and the hook run. The total is 89.

**Added 2026-09-18, in the evening.** `LIST-14` to `LIST-19` trace to #83, #84, #81, #82, #79
and #80, filed that evening from the same day of testing: two taken as a third task of a plan
not yet executed (11-06.1 for #83, 11-09.1 for #81) and four by inserted plans (11-04.1 for
#84, 11-09.2 for #82, 11-11.3 for #79, and 11-11.1 with 11-11.2 for #80, the second of those
because the separate window is a process of its own). The total is 86.

**Added 2026-09-18, later the same day.** `LIST-11` to `LIST-13` trace to #75, #76 and #77,
filed that afternoon from the third day of testing and taken by inserted plans 11-13, 11-06.1
and 11-09.1. The total is 80.

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

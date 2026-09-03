# Phase 3: Mail at scale on the wire - Research

**Researched:** 2026-09-03
**Domain:** In-repo, plus two vendored crates read directly: `async-imap` 0.11.3 and
`imap-proto` 0.16.7.
**Confidence:** HIGH on everything read from source this session, with each claim carrying the
file and line it came from. Nothing here came from a web search. No external package is added by
this phase.

This is research rather than a discussion. It was produced by reading the tree, not by asking
questions, so it answers what is true today and leaves the decisions at the end for Pratik.

## Summary

**This phase is smaller than the roadmap thinks in one place, larger in another, and rests on one
change that can destroy a user's cached mail.** Three of the six requirements describe work that
is partly or mostly done, and the requirement text for those three is stale enough to mislead a
planner.

The single most important finding is not a gap. It is a hazard: **deletion detection is fused to
the full UID listing that SCALE-01 exists to remove.** `sync_folder` calls `list_uids`
unconditionally at `src/application/mail_sync.rs:1014`, and its result feeds `uids_to_forget`,
which feeds `cache.forget_messages`. The doc comment at `mail_sync.rs:286` already states the
constraint: the server list must be the whole mailbox, not the page just fetched, because
comparing against a page would delete everything outside it. A resume that skips the listing
without also gating the forget deletes the user's cached mail on the first sync after the change.
Nothing else in this phase can destroy data. This can, silently, and the tests that would catch it
have to be written before the change rather than after.

## What is already built

Read this before planning anything, because three requirements describe work as absent that is
present and reached from a non-test path.

### SCALE-01's sync state exists and is in use

`folders.uid_validity` and `folders.highest_modseq` are additive columns at
`src/data/message_cache/mod.rs:2252` and `:2262`. They are read and written by
`folder_uid_validity` and `set_folder_uid_validity` (`src/data/message_cache/messages.rs:1475`,
`:1488`) and by `folder_modseq` and `set_folder_modseq` (`src/data/message_cache/folders.rs:682`,
`:695`).

`sync_folder` already reads UIDVALIDITY, detects a change, calls `forget_folder_messages` and
re-stores it (`mail_sync.rs:998-1012`), then passes the stored modseq to `fetch_flags` as
`changed_since` (`:1082-1090`). It is reached in the running program from
`src/presentation/wx_app.rs:18848`.

So SCALE-01 is not "build sync state". It is "stop listing every UID when the state says you do
not have to", which is the hazard above.

### SCALE-04's split storage is mostly built

The requirement says the hot, warm and cold split was not built. That is no longer true.
`src/data/message_cache/bodies.rs:1-22` opens by saying bodies used to live inline in the
`messages` table and live in `message_bodies` instead, written when a message is opened and read
back only when one is displayed. The table is at `mod.rs:2060-2069` with `bytes` and
`last_read_at`. Bodies are zlib-packed at level 6 (`bodies.rs:57`). Eviction is reached from a
non-test path: `keep_bodies_within_budget` is called at the end of every `sync_folder`
(`mail_sync.rs:1134`), and the module header records that before that call existed the function
had no non-test caller and the cache grew without limit. The attachment tier exists as well, an
`attachments` table plus a digest-keyed content store (`mod.rs:1421-1442`).

Criterion 4's first half, that a folder listing reads no body text, is **already true**.
`listing_query` (`messages.rs:56-68`) selects `m.snippet` and touches neither `messages.body_plain`
nor `message_bodies`. Its doc comment says it is built in one place precisely so a test can ask
SQLite how it plans to answer the exact query, so the guard the criterion wants has its hook
waiting.

The migration precedent is proven too: `migrate_inline_bodies` (`bodies.rs:609`) runs on every
cache open (`mod.rs:1226`), non-fatally.

### SCALE-06's conflict model exists, for contacts

The requirement says there is no conflict resolution path in `src/`. There is.
`whose_copy_wins` (`src/application/contacts_sync.rs:988`) returns a four-armed `WhoseCopyWins`
(`:949-974`) built from two facts: whether local work is unsent, and whether the address book's
version marker moved. It compares markers rather than clocks, deliberately (`:977-982`). It has
two production call sites (`:2289`, `:2485`), reached from `wx_app.rs:18959` and `:18986`. The
losing case is counted and spoken rather than silent: `SyncResult::sent_over_a_newer_copy`
(`:335`) becomes a sentence at `:1674-1690` and reaches the user through
`what_the_contacts_sync_did` at `wx_app.rs:15289`.

CalDAV has the marker machinery too, `etag` and `If-Match` (`caldav_sync.rs:870`, `:1028`,
`:3424`, `:3716`), but resolves automatically and shows the user nothing.

## What is absent or worse than recorded

### SCALE-02 is worse than the requirement says

The requirement names eight bypass sites at specific lines. There are **twelve**, and none of the
eight lines it names is a connect site any more. Current sites, all in
`src/presentation/wx_app.rs`: 7555, 8124, 8405, 8627, 8803, 16289, 16450, 17509, 18249, 18350,
18439, 18713. The purpose-built helper `a_session_at` (`src/application/mail_session.rs:21`) has
three production callers: `deleting_at_the_server.rs:112`, `sent_copy.rs:245`, `wx_app.rs:13485`.

The worst single case is the requirement's own argument, verified: marking one message read
(`wx_app.rs:17509-17534`) builds a `MailController`, calls `connect_imap`, issues one `set_flag`,
and disconnects. A full TLS handshake, CAPABILITY, LOGIN and SELECT per starred message.

Two consequences for planning. A plan budgeted against eight sites underestimates by half. And the
requirement's own `[D]` item, a test counting the bypasses, is written against a stale number.

The lock a held session would live behind already exists and does not need replacing:
`MailController` holds `Arc<Mutex<Option<ImapSession>>>` with `require_imap`
(`mail_controller.rs:278`) as the single unwrap point. What is missing is lifetime and reconnect,
not structure. There is no reconnect or retry anywhere: searching `mail_controller.rs`, `imap.rs`
and `mail_sync.rs` finds two hits, both prose comments about a user retrying by hand
(`imap.rs:2894`, `:2916`).

**The connection budget starts at two, not one.** `watch_folder` (`mail_sync.rs:1165`) opens its
own client and session for IDLE, reached from `wx_app.rs:17212` and dispatched at the end of every
mail check (`:18897`).

### SCALE-05 ships a promise it does not keep

There is no network detection of any kind. `WxUIState.offline_mode` (`wx_app.rs:315`) is
initialised false (`:430`), toggled by `ID_OFFLINE_MODE` (`:4854-4871`), mirrored on
`UIUpdate::OfflineModeChanged` (`:15247`), and **read by nothing that decides anything**. Those
four are its only occurrences in the file.

The toggle's own status line says "Offline mode enabled - outgoing mail will be queued"
(`wx_app.rs:4862`). Nothing queues. This is in the shipped product, and it is the kind of thing
guardrail 9 and the alpha-marking rule both cover: a person is told something that is not true.

The outbox itself is complete: `queue_outbox_message` (`outbox.rs:38`),
`outbox_messages_that_may_go_now` (`:95`), `when_a_queued_message_may_go` (`:112`),
`cancel_queued` (`:256`), `update_outbox_failure` (`:287`). `flush_outbox` (`wx_app.rs:15883`) has
exactly one caller, the menu item at `:4877`, and never consults `offline_mode`. It correctly asks
only for messages that may go now, so undo-send holds and scheduled sends are respected
(`:15925-15935`).

**Criterion 5's "rather than flushing the outbox unasked" is satisfied today by accident**, and
wiring "the network came back" straight to `flush_outbox` would break it and send mail nobody
asked to send. That is guardrail 7.

### SCALE-03 has two 500s, and the requirement names one

`INITIAL_FETCH_LIMIT` (`mail_sync.rs:38`) bounds what is fetched from the server, applied in
`uids_to_fetch` (`:263-279`). `FOLDER_LIST_PAGE_SIZE` (`wx_app.rs:6346`) bounds what is read out
of the cache into the list. Get Older Messages (`Shift+F9`, `wx_app.rs:5929`) moves both. A change
to one alone appears to do nothing because the other still binds.

The superseding announcement topic criterion 3 asks for **already exists and already carries sync
progress**. `Announcement::with_topic` (`accessibility/announcements.rs:108`), superseding in
`push` (`:180-186`), `announce_topic` (`accessibility.rs:310`) with roughly 25 production call
sites. `UIUpdate::StatusUpdated` is handled at `wx_app.rs:15157` as
`announce_topic(status, Priority::Low, "status")`, with a comment saying these arrive steadily
while a mailbox syncs and the queue coalesces them. Criterion 3 is close to satisfied and mostly
needs measuring rather than building.

One open design question sits there: `"status"` already carries every other status line, so a long
whole-folder fetch on that topic would silence them. There is a precedent for splitting,
`"message text"` at `:15174`, kept off `"status"` deliberately.

## The async-imap question the roadmap recorded as blocking

The roadmap says SCALE-01 depends on what `async-imap` 0.11.3 exposes. It is answerable now, and
the answer is **partly**.

Read directly from the vendored crate: `select_condstore` (`src/client.rs:394`),
`Mailbox.highest_modseq` (`src/types/mailbox.rs:40`), `Fetch.modseq` (`src/types/fetch.rs:38`),
`STATUS HIGHESTMODSEQ` parsed at `src/parse.rs:134`. There is **no `ENABLE`, no
`select_qresync`, and no VANISHED handling anywhere in it**.

But QRESYNC is not out of reach. `run_command` (`client.rs:1376`) and `run_command_and_check_ok`
(`:1364`) are public raw escape hatches, and `imap-proto` 0.16.7 already parses everything QRESYNC
needs: `Response::Vanished` (`src/parser/rfc7162.rs:31`), `ResponseCode::HighestModSeq`
(`src/parser/rfc4551.rs:21`), `AppendUid` and `CopyUid` (`src/parser/rfc4315.rs:36`, `:54`).

**This project already uses that escape hatch for CONDSTORE.** `read_command` in
`src/service/protocols/imap.rs` hands each parsed `Response` to a closure, and `fetch_flags`
builds `UID FETCH 1:* (UID FLAGS) (CHANGEDSINCE {modseq})` by hand at `imap.rs:1154`, gated on
`self.abilities.condstore`. A `Response::Vanished` arm in an existing closure is the same shape of
work.

The cost, stated honestly: a raw `SELECT x (QRESYNC (...))` returns a mailbox response that
async-imap's own `select` parses, and going through `run_command` means rebuilding that parsing.
`Abilities` has no `qresync` field (`imap/abilities.rs:26-49`), so capability detection is a small
addition either way.

The fallback the roadmap says is already specified, quoted from SCALE-01: deletions are found by a
periodic UID set comparison, bounded so it does not run on every folder open.

## The three defects phase 1 deferred into this phase

**The Gmail All Mail count is real, and "one extra predicate in one query" understates it.**
`conversations_query` (`messages.rs:148-208`) excludes All Mail by folder at `:155`, and the
`here` CTE draws rows only from `reach` (`:164`) while qualifying threads by presence in the
current folder (`:167-171`). Standing in All Mail with account-wide reach, an archived unlabelled
message qualifies the thread and contributes no row, so the conversation vanishes. Three things
make the fix bigger: the identity column is `messages.gmail_msgid` (`mod.rs:2297`), not
`gmail_message_id` as the deferred note calls it; the `here` CTE has no join to `folders`, so a
per-message predicate needs one added; and the general fallback, `Message-ID`, is what ledger 8
was about, with `threads_holding_any_of` still asking about both spellings
(`messages.rs:1163-1195`).

**The conversation root merge is real, and needs a backfill the note does not mention.**
`threads_holding_any_of` (`messages.rs:1151-1225`) matches on `m.message_id IN (...)` and nothing
else, so it cannot see a stored message whose `refs_header` names the arriving one. The writer and
call site are already single and in the right place: `merge_what_this_message_connects`
(`:961-992`) runs on every arrival through `upsert_message` (`:943`), and `reroot_threads`
(`:1243`) is one indexed `UPDATE`. Beyond "one table, one index, one writer": a new identifier
table starts empty, so without a backfill every conversation already stored stays split, which
reads to a user as the fix not working. And `rejoin` (`thread_identity.rs:195`) always makes the
arriving message's own root the winner, which for a chain naming only its parent is not the true
root; more merges firing makes that rule matter more.

**The `next_local_uid` wrap is real, still unreachable, and guarded by something outside itself.**
`messages.rs:752` saturates on `i64` then casts to `u32`, exactly as described.
`FIRST_RESERVED_UID` is `u32::MAX` (`messages.rs:14`) and the query has no filter, so one reserved
row would make the next call return 0. It does not happen because `numbering_in` (`:628`)
dispatches by folder path and a folder is either counting up or counting down, never both. The two
direct callers are a `#[cfg(test)]` helper (`local_delete.rs:289`) and a local POP folder
(`pop_sync.rs:268`). So the guard is the dispatcher's discipline, not the function's, and one new
direct caller pointed at a server-numbered folder makes it reachable with one row rather than four
billion.

## What cannot be settled here

No account has ever been used with this program, so several criteria have a last mile that closes
against a real server or not at all. Naming them precisely, because glossing them is what
guardrail 9 is about.

1. **Whether any real provider grants CONDSTORE.** `imap/abilities.rs:112-113` already asserts and
   comments that Gmail has never offered it, and `changed_since` is gated on
   `self.abilities.condstore` (`imap.rs:1150-1160`). So on Gmail every sync takes the full re-read
   branch, and criterion 1 costs something different there than on a CONDSTORE server.
2. **Whether `SELECT (QRESYNC ...)` works through the raw path.** Parser support and escape hatch
   are both present. Whether the hand-built select parses back, and whether VANISHED reaches the
   closure, has never been run against a server.
3. **The connection budget.** What a provider does with a session idle for minutes, and whether a
   reconnect after a drop is accepted or rate-limited, is untestable here. Ledger 11 records the
   same gap for the bulk body fetch.
4. **UIDVALIDITY in practice.** `forget_folder_messages` on a UIDVALIDITY change destroys a
   folder's cached mail, and nothing has seen a real server change it. This is the second
   data-destroying path in the phase.
5. **Every accessibility criterion in 3, 5 and 6.** Whether a whole-mailbox fetch on the `"status"`
   topic silences the other status lines, whether an offline announcement is heard once rather
   than per failed request, whether two versions are announced as a labelled pair. Expect these to
   close as `unrun-verify` ledger entries the way phase 2's did.

## Decisions for Pratik

These change what gets built and are not mine to settle.

1. **QRESYNC by hand, or take the recorded fallback?** The escape hatch is there and this project
   already uses it for CONDSTORE. Building it means rebuilding the select response parsing and
   testing it against nothing. The fallback is specified and cheaper and slower.
2. **The `offline_mode` false promise.** It ships today telling people their mail will be queued.
   Fix it now as a small separate change, or fold it into SCALE-05?
3. **SCALE-04's scope**, given the split is built. Verify and guard what exists, or is there more
   wanted?
4. **SCALE-06's target.** Contacts already has a working conflict model and CalDAV has the markers
   but asks nobody. Mail's flags are optimistic-local with revert-on-failure, so the "both changed"
   state largely cannot occur there. Build the choice where the state exists, rather than where the
   requirement points?

---
*Research written 2026-09-03, from a read of the tree at commit `e1bafd9`.*

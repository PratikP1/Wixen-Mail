# Roadmap: Wixen Mail

## Overview

Wixen Mail is at 0.45.0 with most of the product already built. This milestone is the
outstanding work: the two "not built" sections of `.planning/intel/built-and-left.md`, and
nothing else. The journey runs from the shape of the mailbox (folders a user can manage,
nested where the server nests them, conversations collapsed to one row), through a search that
says what it covers, through syncing a large mailbox without re-listing it, out to the
composer and the reader, across the five modules that are not mail, into the channels the
application speaks through, then to how a build reaches a user, and ends by replacing every
estimate the project quotes with a measurement.

Two things bound every phase. First, nothing here has ever run against a real mail account, so
no success criterion claims behaviour against a live server. Second, anything that writes to a
server passes through `src/application/allowed.rs`, where mail writes are off for a new
install and three places must agree before anything goes out.

Live-account validation of the thirteen "built but unproven" rows is real work and is
deliberately not this milestone.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Folders and conversations** - Make and manage folders, nest them, pin them, and collapse the list to one row per conversation
- [ ] **Phase 2: Search that says what it covers** - The scope selector scopes, the coverage is disclosed, and a rule can be a folder
- [ ] **Phase 2.1: What phase 1 found on its way past** (INSERTED) - Thirteen defects and stale documents that belong to no other phase
- [ ] **Phase 3: Mail at scale on the wire** - Resume rather than re-list, hold one connection, fetch a whole mailbox, and never pick a conflict winner silently
- [ ] **Phase 4: Writing and reading a message in full** - Attachments in and out, inline images with alt text, spell check while typing, and PGP
- [ ] **Phase 5: The other five modules keep up** - Move and copy everywhere, recurring events across weeks and months, notes that sync, contacts over CardDAV
- [ ] **Phase 6: How the application speaks** - Per-event feedback channels, dates in the user's language, and a scan that names what it cannot judge
- [ ] **Phase 7: Installing, updating and what is stored** - A signed installer, an update check, shortcuts, the cache encryption decision, and the other two platforms
- [ ] **Phase 8: Every number the project quotes** - Replace the estimates with measurements

## Phase Details

### Phase 1: Folders and conversations

**Goal**: A user can shape and work through their mail by its own structure: an account they can tell from the next one, folders they can make and manage, nested the way the server nests them, favourites at the top, and conversations collapsed to one row.
**Depends on**: Nothing (first phase)
**Requirements**: FOLDER-01, FOLDER-02, FOLDER-03, THREAD-01, THREAD-02
**Success Criteria** (what must be TRUE):

  1. A user creates, renames, moves, marks read, empties and deletes a folder from the folder tree using the keyboard alone. Renaming changes the name; moving to another parent is its own command. A server folder is refused with a reason rather than attempted when mail writes are off; a local one is not gated, because a POP account has no server folders at all.
  2. A folder named `Archive/2026` reads as `2026` nested under `Archive`, with its level announced by the native tree control, and the tree remembers what was collapsed across a restart, keyed by identity rather than by label so a rename does not lose it.
  3. Each account is its own branch, ordered by the user and moved with the keyboard, so two POP accounts no longer show two folders called Inbox with nothing to tell them apart.
  4. Sent, Outbox, Drafts, Junk and Trash are one each, shared across accounts under "On this computer", and an existing database is migrated into that shape message by message with nothing removed until every message has landed and a count reported.
  5. A user pins a folder and it stays in a group at the top of the tree across a restart, without ever writing to the server, appearing there as well as in its account branch rather than instead of it.
  6. The View menu's thread view is enabled, and switching it collapses the list to one row per conversation announcing subject, message count and unread count, with every column answering about the conversation rather than about its newest message.
  7. A message arriving into an open folder joins its thread without the folder being reopened, including the case where a late message merges two existing trees.
  8. The five settings this phase adds are each reachable and operable from a real settings screen by keyboard, with their state announced. A setting the model holds and no screen writes is what FEEDBACK-01 exists to fix; this phase must not add a sixth.

**Plans**: 14/14 plans executed, one per wave. Two shared files, `guards/guards.toml` and `docs/changelog.md`, are touched by most plans under the same-commit rules, and `src/presentation/wx_app.rs` by most, so the plans are ordered rather than run in parallel.

Plans:
**Wave 1**

- [x] 01-01-PLAN.md — Tracer: create a folder end to end, encoder included (D-41)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 01-02-PLAN.md — A conversation identity that is stored, and the two indexes (D-39)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 01-03-PLAN.md — Nesting stored as a parent link, and local names that contain the separator (D-22, D-23)

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 01-04-PLAN.md — Rename the leaf, move the subtree, delete deepest first (D-26)

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 01-05-PLAN.md — The tree's shape: account branches, "On this computer", identity keying (D-13, D-15 to D-17, D-21, D-25)

**Wave 6** *(blocked on Wave 5 completion)*

- [x] 01-06-PLAN.md — Moving accounts, counting unread, and the settings guard (D-14, D-24, D-42, D-43)

**Wave 7** *(blocked on Wave 6 completion)*

- [x] 01-07-PLAN.md — Five local folders shared, and the migration that gets there (D-18 to D-20, D-40)

**Wave 8** *(blocked on Wave 7 completion)*

- [x] 01-08-PLAN.md — Favourites (D-28 to D-32)

**Wave 9** *(blocked on Wave 8 completion)*

- [x] 01-09-PLAN.md — Empty a folder and mark one read (D-33 to D-38)

**Wave 10** *(blocked on Wave 9 completion)*

- [x] 01-10-PLAN.md — A folder the server stopped listing (D-27)

**Wave 11** *(blocked on Wave 10 completion)*

- [x] 01-11-PLAN.md — What a conversation is and what its row says (D-02 to D-04, D-08)

**Wave 12** *(blocked on Wave 11 completion)*

- [x] 01-12-PLAN.md — Switching the view, and what survives it (D-01, D-05 to D-07, D-09 to D-12)

**Wave 13** *(blocked on Wave 12 completion)*

- [x] 01-13-PLAN.md — Rethread as mail arrives, including the two-tree merge (THREAD-02)

**Wave 14** *(added 2026-08-31, after phase verification found criterion 3 undelivered)*

- [x] 01-14-PLAN.md — The tree holds every account, and moving between them is a selection (criterion 3)

**UI hint**: yes
**Scope note**: These criteria were rewritten 2026-08-29 after the phase discussion. The original five described nesting a flat tree. What the discussion decided is in `.planning/phases/01-folders-and-conversations/01-CONTEXT.md`, which is the authority on the detail; these criteria are what the phase is verified against. The phase also needs three IMAP verbs that do not exist in `src/service/protocols/imap.rs` today: CREATE, RENAME and DELETE mailbox.

### Phase 2: Search that says what it covers

**Goal**: A search returns what the user asked for, and says plainly what it could not reach.
**Depends on**: Phase 1
**Requirements**: SEARCH-01, SEARCH-02, SEARCH-03
**Success Criteria** (what must be TRUE):

  1. A search saved with Subject Only or From Only reruns with that restriction, not across subject, sender and recipients. The live search already honours all four scopes; only the saved one loses half of what it was given. This needs no schema change: `saved_search_questions` already stores an arbitrary set, so the narrower search writes fewer questions.
  2. Opening a saved search says what it asks, in one sentence that reads the same whether or not the In box has a name for it: "looks at subject and body". A short result list is then legible as narrow coverage rather than as an empty mailbox.
  3. A rule editor writes into the same saved searches the search box writes, reaching all eleven fields in `A_FIELD_A_RULE_MAY_NAME` rather than the three the search box uses. There is one stored thing with two doors, one matcher, and one group in the tree however a search was made.
  4. A search that can reach message text says, before it runs, how many messages in the account have body text stored and how many do not, so a short answer is never mistaken for a complete one.
  5. Fetching the missing text is built and gated. Since it is a read and every `may_i` call gates a write, `application::allowed` gains a read dimension, on by default, which is a stated exception to that type's rule that `Default` is the safe end.
  6. Saved searches sit inside the account structure the way pinned folders do, so two accounts each holding a search of the same name are never two identical rows.

**Plans**: 9/9 plans executed, one per wave. `guards/guards.toml` and `docs/changelog.md` are touched by nearly every plan under the same-commit rules, and `src/presentation/wx_app.rs` by six of the eight, so the plans are ordered rather than run in parallel. The wave numbers say only "this one after that one"; there is no wave holding two plans.

Plans:

- [x] 02-09-PLAN.md

**Wave 1**

- [x] 02-01-PLAN.md — Tracer: a read dimension on `Allowed`, end to end from the stored settings file to the one fetch that already exists (D-2-06, D-2-07, D-2-11, D-2-12)

**Wave 2** *(blocked on Wave 1)*

- [x] 02-02-PLAN.md — What a body-reading saved search covers, said before it runs, naming which search it is about (D-2-08, D-2-13)

**Wave 3** *(blocked on Wave 2)*

- [x] 02-03-PLAN.md — Fetching the missing text, behind the gate, marked experimental where somebody meets it (D-2-08)

**Wave 4** *(blocked on Wave 3)*

- [x] 02-04-PLAN.md — One vocabulary: the filter dialog offers the eleven fields and eleven match types the engine answers, in words

**Wave 5** *(blocked on Wave 4)*

- [x] 02-05-PLAN.md — A saved search keeps both halves of its scope, and says what it asks (D-2-03, D-2-04, D-2-14)

**Wave 6** *(blocked on Wave 5)*

- [x] 02-06-PLAN.md — Writing a whole question list back atomically, and a dialog for one condition (D-2-01)

**Wave 7** *(blocked on Wave 6)*

- [x] 02-07-PLAN.md — The rule editor: a manager over one search's conditions, reached from the tree, one group however a search was made (D-2-01, D-2-02)

**Wave 8** *(blocked on Wave 7)*

- [x] 02-08-PLAN.md — Saved searches inside the account structure, and a search that runs against its own account (D-2-05)

**UI hint**: yes
**Scope note**: These criteria were rewritten 2026-08-31 after the phase discussion, from four to six. The original criterion 4 assumed a smart folder was a separate object from a saved search; `Question::as_a_rule` converts a saved-search question into a `FilterRule` to evaluate it, so they are one vocabulary and the gap is only reach. `.planning/phases/02-search-that-says-what-it-covers/02-CONTEXT.md` is the authority on the detail. The largest thing here is not search: widening `Allowed` to cover reads touches a model three places must agree on, and if it ripples further it is a candidate for its own phase rather than something to absorb quietly.

### Phase 2.1: What phase 1 found on its way past (INSERTED)

**Goal**: Fix the defects phase 1 uncovered that belong to no other phase, and correct the documents that describe work as unbuilt when it ships.
**Depends on**: Phase 1
**Requirements**: none new; this closes recorded defects rather than adding capability
**Success Criteria** (what must be TRUE):

  1. Two dialogs stop leaking a registry entry per row. `wx_destination.rs` and `wx_thread_view.rs` hang row data off the tree control, whose data goes into a process-global registry that `delete_all_items` does not clear and whose cleanup returns early on any childless item. Each takes a parallel vector, which is what `collect_rows` already does.
  2. Sorting messages by Safety orders them by severity rather than by the alphabet. `MAX(m.safety)` returns the mildest verdict today, because safety is stored as words and the alphabet puts "suspicious" last. The `CASE` the conversation expression already uses is the answer.
  3. One spelling of a message identifier is written by every writer. Mail through `mail_parser` is stored bare and a draft this program files keeps its angle brackets, so a join between them finds nothing and the symptom is indistinguishable from a bad test fixture. Normalise on write, and backfill.
  4. The ten checks in `tests/wired.rs` that read `wx_app.rs` read the half that ships rather than a prefix of it. They cut at the first `#[cfg(test)]`, which in a file of 24,650 lines with 19 test modules means reading 77% and being blind to 5,762 lines. Four failed loudly when a module was added mid-file; six passed in silence, which is the defect. `common::what_ships` is the right reader and cannot currently be reached from an integration test, so this decides whether it ships, moves to a test-support crate, or is duplicated.
  5. Every document that says folder management does not work is corrected. `docs/IMPLEMENTATION_STATUS.md` and `.planning/intel/context.md` both describe as unbuilt what phase 1 shipped, and a page that describes a feature you have as missing wastes exactly as much of somebody's time as the reverse.
  6. The window that asks about folders the server has stopped listing is exercised by something. `ask_about_the_folders_that_have_gone` is the only code in 01-10 no test reaches: everything it decides is tested without a window, but if the call site stopped passing `turn.is_none()` or stopped asking for focus, every test would stay green.

  7. A guard that reads documents can see a violation when one exists. `test_no_status_page_names_a_version_the_code_does_not_ship` reads `THE_STATUS_PAGES`, which is `README.md` and `docs/IMPLEMENTATION_STATUS.md`. Measured 2026-09-02: neither file names a version at all, so it iterates over nothing and passes unconditionally, and its own comment is what advised the change that disarmed it. `CLAUDE.md` describes this as a past event; it is live. Every guard whose trigger is "a document mentions X" needs a companion proving the reading works, and this one needs it first.

  8. The tree stops telling people a test cannot build a window, because it can. Five places say so, one flatly: "Nothing in this crate builds a live wxWidgets window inside `cargo test`" at `wx_app.rs:19806`, plus `one_question_at_a_time.rs:36`, `view_state.rs:3`, `wx_app.rs:6462` and `wx_app.rs:25434`. Measured by 02-04 in both directions: one such test passes and the whole library run stays green with it, and a second in the same process prints `initializing twice?` and hangs. The truth is a budget of one per process, which is a different instruction from "impossible" and has been steering work away from a technique that works.

  9. A guard living in `tests/` runs on the commits that could break it. `scripts/check.sh` maps a changed `src/a/b.rs` to `cargo test --lib a::b::`, so `tests/manager_dialog_labels.rs` and `tests/checkbox_labels.rs`, which guard `src/` modules, are reached only when the test file itself changes. `guards/guards.toml` already declares that coupling in its `file` and `suite` fields, so the gate can read it rather than a new list being invented.

  10. A dialog stops silently rewriting a value it cannot show. A rule naming a field this build has never heard of loses that field on the way through: opening selects nothing and pressing OK stores the empty string. 02-04 closed the case where five of eleven real fields did this; what remains is a rule written by a later version. The fix is a refusal or a passthrough, which is a decision about what a dialog owes a value it cannot display.

  11. A branch row offers a menu that fits it. `wire_context_menu` answers `Focus::MailFolders` for everything that is not a saved-search row, so account branches, the Favourites branch, the Labels heading and "On this computer" all offer "Get older messages" and "Folders to keep up to date" on rows that are not folders. True since 01-14 and D-29; 02-07 gave only the saved-search row its own focus. Deciding what a branch row's menu holds is the work.

  12. Two accounts sharing a name are two rows. **Corrected on 2026-09-02 by 02.1-08, which measured it rather than reading the comment.** The wording below was right about the mechanism and wrong about the symptom, twice over, and both corrections are in `02.1-08-SUMMARY.md`. `where_a_row_sits` is not uncalled: `wx_app::the_row_on_screen` calls it on every folder tree selection. And two accounts called "Work" did not produce identical chains, because `the_accounts_in_the_tree` filled each name from `Account::display_name`, which is `"{name} <{email}>"`, and the accounts table declares `email TEXT NOT NULL UNIQUE`. So the property was real, held by two layers `folder_tree.rs` never mentions, and unowned there. What 02.1-08 fixed is that, plus the cost nobody had filed: the address was read aloud on every account branch, always, for a case that had never happened.

      Original wording, kept because the correction is only legible beside it: "`where_a_row_sits` pairs by label chain, so two accounts called "Work" produce identical chains and the pairing takes the first. True of the account branches since 01-14, and the saved-search group inherits it. The comment on `where_a_row_sits` already says so."

  13. A reply to a forwarded Hungarian message does not say it is a reply. `mail_parser`'s `trim_trailing_fwd` ignores a parenthesised word of one character, so Hungarian's `I:` forward marker is read as a reply marker. Recorded as ledger entry 5 against `src/application/conversations.rs`, which documents the behaviour at line 323. It belongs to no phase, which is why it has sat since 01-11.

      **This criterion used to name a threading symptom, and that symptom does not exist.** It said a forwarded Hungarian message joins the conversation it is a reply to. `is_a_forward_marker` is read by two functions and both are called only from `src/presentation/wx_compose.rs`; nothing in threading reads either. Traced when 02.1-07 was planned and verified again when it was executed. The mechanism is exactly as recorded and reproduces; the harm is in composition, and it is two things. Replying wrote no reply marker, so the answer went out looking like new mail, and forwarding wrote a second forward marker in front of the one already there.

**Plans**: 9/9 plans executed, one per wave. Counted from the summaries on disk on 2026-09-02, which is the count that cannot go stale behind a box nobody ticked; this line said 1/9 while four were ticked, then 7/9 while nine summaries existed, because 02.1-05, 02.1-08 and 02.1-09 each landed with the box left unticked. `guards/guards.toml` and `docs/changelog.md` are touched by most of them under the same-commit rules, so the plans are ordered rather than run in parallel. Each plan's file list is deliberately small and of one kind, because the commit gate is scoped to what a commit touches: a documents-only commit is about 51 seconds against about 350 for the whole gate, and mixing a document correction with three source modules makes every commit in that plan pay for all of them.

Plans:

**Wave 1**

- [x] 02.1-01-PLAN.md — The checks that read the main window read all of it, and what that finds is reported before it is fixed (criterion 4, D-2.1-01)

**Wave 2** *(blocked on Wave 1)*

- [x] 02.1-02-PLAN.md — A guard that reads documents can see a violation, and nothing says a test cannot build a window (criteria 7, 8)

**Wave 3** *(blocked on Wave 2)*

- [x] 02.1-03-PLAN.md — Every page that says folder management is missing (criterion 5)

**Wave 4** *(blocked on Wave 3)*

- [x] 02.1-04-PLAN.md — A guard under `tests/` runs on the commits that could break it (criterion 9)

**Wave 5** *(blocked on Wave 4)*

- [x] 02.1-05-PLAN.md — Two dialogs stop leaking a registry entry per row (criterion 1)

**Wave 6** *(blocked on Wave 5)*

- [x] 02.1-06-PLAN.md — Safety sorts by how bad it is, and one spelling of a message identifier (criteria 2, 3)

**Wave 7** *(blocked on Wave 6)*

- [x] 02.1-07-PLAN.md — Two decisions nothing was asking about (criteria 6, 13)

**Wave 8** *(blocked on Wave 7)*

- [x] 02.1-08-PLAN.md — A branch row offers a menu that fits it, and two accounts of one name are two rows (criteria 11, 12, D-2.1-03)

**Wave 9** *(blocked on Wave 8)*

- [x] 02.1-09-PLAN.md — A dialog refuses a value it cannot show rather than rewriting it (criterion 10, D-2.1-02)

**UI hint**: yes
**Scope note**: Inserted 2026-08-31 after routing phase 1's deferred items by subject. Three items went to phase 3 and two to phase 6, where somebody planning those subjects will meet them. Criteria 1 to 6 belong to no phase, which is why they were deferred and why they would otherwise stay deferred.

Criteria 7 to 12 were added 2026-09-02 from phase 2's own deferrals and from the observation log, and each was re-checked against the tree rather than taken from the note that recorded it. Two came out worse than logged: the doc comments about windows are five places rather than three, and the version guard is disarmed now rather than historically.

Three further things are recorded elsewhere and are deliberately not criteria here. The spellcheck test that fails about one full library run in five through a Windows COM call made twice is diagnosed only as far as reading, and inventing a criterion for it would be pretending otherwise. `wxdragon 0.9.17`'s `ListCtrl::get_item_text` loses the last character of every cell and returns a NUL in its place, which is an upstream defect carried in the ledger as entry 28 and unreported so far; reporting it upstream is not this phase's work but it should not stay unreported. And a `said_and_shown` census in `wx_managers.rs` was noted by 02-06 as holding a floor of 10 against 19 members, so it is slack by nine and no longer load-bearing; that was noted rather than measured and wants confirming before it earns a criterion.

Planning on 2026-09-02 re-checked every criterion against the tree again and found five whose stated premise had moved. They are left as written above, because the criteria are the record of what was believed, and each plan's `<premise_corrections>` carries the measurement and what it changes. Criterion 4 says ten checks; there are twelve of one kind and four of a second, plus a helper to delete. Criterion 2 says the conversation expression is the answer; that half already ranks by severity and the message half does not, so the work is the message half. Criterion 5 says the status page describes folder management as unbuilt; every sentence in that paragraph is true and it sits under the heading saying what does not work, so the page is wrong by position and a search for the sentence finds nothing. Criterion 9 says the registry already declares the coupling; it does for one of the two targets named and not for the other, which has no record at all. Criterion 12's mechanism is not live: `where_a_row_sits` has no production caller, and what two accounts of one name really cost is two rows a person cannot tell apart. Criterion 13's mechanism reproduces exactly and its symptom does not: the classification is read only by the composer, so the cost is a reply that does not say it is one and a doubled forward marker, not threading.

### Phase 3: Mail at scale on the wire

**Goal**: Sync a large mailbox without re-listing it, without signing in again for every message, and without silently choosing a winner when two copies disagree.
**Depends on**: Phase 1
**Requirements**: SCALE-01, SCALE-02, SCALE-03, SCALE-04, SCALE-05, SCALE-06
**Success Criteria** (what must be TRUE):

  1. Reopening a folder that was synced before resumes from the stored sync state instead of re-listing every UID, and a `UIDVALIDITY` change announces the resync rather than doing it quietly.
  2. Opening several messages in a row reuses one authenticated session, and a dropped connection reconnects once and says so if the retry also fails.
  3. A user can ask for a whole folder, the list is usable from the first chunk, and progress speaks as one superseding topic instead of hundreds of updates.
  4. A folder listing reads no body text, and an existing user database opens and migrates to the split storage without losing a message.
  5. Losing the network puts the application offline and announces it once; regaining it offers to go back online rather than flushing the outbox unasked.
  6. When a local copy and a server copy have both changed, the user is shown both and chooses, and nothing is pushed until they do.

**Plans**: 9 plans, one per wave. The four decisions `03-RESEARCH.md` left for
Pratik were answered on 2026-09-03 and the plans carry the answers: build a seam
over how deletions are found and take the UID comparison behind it rather than
QRESYNC, leave the offline false promise until plan 03-08 rather than fixing it
sooner, attack the permanent body migration as well as proving the storage split,
and build the conflict choice for contacts and CalDAV plus a fix for the mail
defect that is not a conflict. One question is still open, in 03-07: which
announcement topic a whole-folder fetch belongs on.

- [ ] `03-01-PLAN.md` — Nothing deletes cached mail on the strength of a partial listing, and a renumbered folder says what it discarded
- [ ] `03-02-PLAN.md` — Count, in a test, the sign-ins that go round the helper, so the number stops going stale in a document
- [ ] `03-03-PLAN.md` — Prove the storage split that already ships, stop a migrated database paying for the migration on every open, and pin the numbering rule a dispatcher currently holds
- [ ] `03-04-PLAN.md` — Gmail mail archived with no label counts toward its conversation, by identity rather than by folder
- [ ] `03-05-PLAN.md` — A conversation root that arrives late merges, and the backfill that makes the fix visible on mail already stored
- [ ] `03-06-PLAN.md` — One session held open per account, one reconnect, and a budget with a number
- [ ] `03-07-PLAN.md` — Resume a folder instead of re-listing it, behind a seam over how deletions are found, and let somebody ask for a whole one
- [ ] `03-08-PLAN.md` — Offline mode does what it says, the network is noticed, and coming back is offered rather than done
- [ ] `03-09-PLAN.md` — The conflict choice is built where the state occurs, contacts and CalDAV, and the flag change lost to an unreachable server is kept instead of undone

**Inherited from phase 1** (see `.planning/phases/01-folders-and-conversations/deferred-items.md`):

- Gmail mail archived with no label vanishes from a conversation count, because D-08 excludes All Mail by folder rather than by message identity. One extra predicate in one query.
- A conversation root arriving after a message that already names it is not merged, so three of six arrival orders over such a set merge. One table, one index, one writer.
- `next_local_uid` hands out 0 after the number range wraps, because it saturates on `i64` and then casts to `u32`. Not reachable in any database this program can currently produce.

### Phase 4: Writing and reading a message in full

**Goal**: A message can be composed with everything it needs to carry, and read with everything it arrived carrying.
**Depends on**: Nothing new; can run alongside Phases 1 to 3
**Requirements**: WRITE-01, WRITE-02, WRITE-03, READ-01, READ-02, READ-03
**Success Criteria** (what must be TRUE):

  1. A file dropped on the composer attaches, and every drop action has a keyboard equivalent at least as quick to reach.
  2. Inserting an inline image requires alt text or an explicit decorative mark, and both survive a draft save and reload.
  3. Misspellings are marked as they are typed, a keyboard command moves between them, and landing on one speaks the word and its suggestions without flooding a long paste.
  4. An image or a text attachment previews in the application, announcing any description the sender supplied and saying plainly when there is none.
  5. A user reads a PGP-encrypted message they hold the key for, and a message that cannot be decrypted says why instead of reading as empty.
  6. A spam classifier verdict is available to the filter rules that already exist, shown with its source named, never as a silent deletion.

**Plans**: TBD
**UI hint**: yes

### Phase 5: The other five modules keep up

**Goal**: Contacts, calendar, tasks, notes and reminders support the same moves mail already does, and the two that currently go nowhere get somewhere to go.
**Depends on**: Nothing new; can run alongside Phases 1 to 4
**Requirements**: PIM-01, PIM-02, PIM-03, PIM-04, PIM-05, PIM-06, PIM-07, PIM-08
**Success Criteria** (what must be TRUE):

  1. A task moves to another list in one action and ends in exactly one list, including when the move fails at the provider.
  2. Move and copy work in contacts, calendar, tasks, notes and reminders with the same two keyboard commands in every module, on the Action menu because they act on the selection.
  3. A recurring event appears on every date it occurs in the week and month views, with a moved or cancelled occurrence shown on the date it really is.
  4. A local note is a first-class Markdown document, and a synced note reaches the backend its account type chooses, through one seam. Where an account type has no backend yet, the settings screen says so rather than offering a switch that does nothing.
  5. The seam is shaped so a hosted note service can be added later without a migration. Preparing for it means the seam does not forbid it, not that anything half exists.
  6. A user adds a CardDAV address book by its own address, and contacts sync both ways through the vCard reader and writer that already exist.

**Plans**: TBD
**UI hint**: yes

### Phase 6: How the application speaks

**Goal**: The user controls what is spoken, brailled, sounded and shown, reads dates in their own language, and the project knows which parts of WCAG its scans can and cannot judge.
**Depends on**: Phases 1 to 5, so the scan coverage list covers the surface those phases add
**Requirements**: FEEDBACK-01, FEEDBACK-02, FEEDBACK-03
**Success Criteria** (what must be TRUE):

  1. A user sets Speech, Earcon, Braille and Visual independently for each of the sixteen events from the Settings Feedback tab, by keyboard, and the setting survives a restart.
  2. Month names, day names and relative wording follow the machine's locale, falling back to English silently where there is no translation.
  3. The accessibility scan output names which WCAG 2.2 AA success criteria it can and cannot judge, so "roughly half" becomes a list.
  4. The interactions only a human screen reader pass can cover are written down as a scoped list, and each of the five WebView2 findings is either fixed or recorded as upstream with the upstream named.

**Plans**: TBD
**UI hint**: yes

**Inherited from phase 1** (see `.planning/phases/01-folders-and-conversations/deferred-items.md`):

- A permission per account is stored, read by `allowed_for`, honoured out to the provider clients, and offered by no screen. This is FEEDBACK-01's exact shape already live in the tree, found by the mirror guard 01-06 added, and it belongs with the requirement written for that fault.
- A reminder alert still opens over somebody who is typing. It shares the one-at-a-time gate 01-10 built but does not ask the typing count. Whether a reminder should wait is a question about what a reminder is for.

### Phase 7: Installing, updating and what is stored

**Goal**: A build reaches a user signed, tells them when there is a newer one, tells them plainly what it leaves on their disk, and does not promise a publisher warning will be gone when the certificate chosen cannot buy that.
**Depends on**: Nothing new; can run alongside Phases 1 to 6
**Requirements**: SHIP-01, SHIP-02, SHIP-03, SHIP-04, SHIP-05, SHIP-06
**Success Criteria** (what must be TRUE):

  1. The published installer and the executable inside it both carry a valid Authenticode signature with a timestamp countersignature, verified against the published release asset rather than a local build. What SmartScreen then does is stated, not promised: only an EV certificate carries reputation from the first download, and while the warning remains, `docs/installing.md` keeps the walkthrough that gets a screen reader user past it.
  2. The application can tell the user a newer version exists, as a deliberate action or an explicit setting, never as a silent background fetch, and declining leaves the current version working.
  3. The installed shortcuts carry the application icon. The installer already creates both the desktop shortcut and the Start menu entry, so this is `IconFilename` on the two `[Icons]` entries and nothing else.
  4. The cache is not encrypted, and that is said once and clearly where a user meets it: the first-run screen and the page about what is stored. It says what the limitation is and is not, distinguishing another user of the same computer, kept out by Windows, from somebody who takes the drive out, who is not unless the disk is encrypted.
  5. The crate builds and the suite passes on Linux and on macOS in CI.
  6. On a platform where the accessibility bridge is absent, the application says so at startup and in Help, derived from what is actually compiled in rather than from a hardcoded platform list. This closes on the disclosure, not on a working bridge.

**Plans**: TBD

### Phase 8: Every number the project quotes

**Goal**: Replace the estimates with measurements, so no figure in the documents is both aspirational and undated.
**Depends on**: Phase 3 for the scale targets; the rest can be measured against any build
**Requirements**: PERF-01, PERF-02, PERF-03, PERF-04, PERF-05, PERF-06, PERF-07
**Success Criteria** (what must be TRUE):

  1. Memory with 1,000 cached messages, cold start to a usable message list, and idle memory each have a recorded number carrying the date, the machine and the build it came from.
  2. The message list is exercised against 200,000 synthetic rows, the sort, filter and scroll paths each produce a number, and a test asserts the virtual text callback issues no SQLite query.
  3. Every count in the documentation carries the command it came from and the date it was taken, and the documents agree with each other. Nothing asserts that a written number equals what a tool reports today, because that is false the next time anyone adds a test. Low coverage is attributed to the untested network transport rather than treated as a number to raise.
  4. One whole-tree mutation run completes, its report is read after the process exits, and every survivor is either killed with a test or recorded with a reason.
  5. One whole-tree guard sweep completes, `scripts/guards.sh` unfiltered over every record in `guards/guards.toml`, and each record it reports short is corrected by hand and then re-measured. This is the one sweep of the milestone: by the decision of 2026-09-03 no sweep runs per merge or per phase, so nothing before this point has re-measured a record that only the whole sweep can reach. Expect roughly 15 hours and expect findings, since the tree will be many phases past the changes being judged.
  6. Each target is either met or revised with the reason written down.

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1, 2, 3, 4, 5, 6, 7, 8. Phases 4, 5 and 7 depend on nothing
the earlier phases produce and can be reordered if something makes that useful.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Folders and conversations | 14/14 | Executed, verification human_needed | - |
| 2. Search that says what it covers | 9/9 | Executed, verification human_needed | - |
| 2.1 What phase 1 found on its way past | 9/9 | Executed, verification gaps_found (12/13) | - |
| 3. Mail at scale on the wire | 0/TBD | Not started | - |
| 4. Writing and reading a message in full | 0/TBD | Not started | - |
| 5. The other five modules keep up | 0/TBD | Not started | - |
| 6. How the application speaks | 0/TBD | Not started | - |
| 7. Installing, updating and what is stored | 0/TBD | Not started | - |
| 8. Every number the project quotes | 0/TBD | Not started | - |

## Notes on this roadmap

**Granularity.** `.planning/config.json` sets no granularity, so the default is standard,
which suggests four to six phases. This roadmap has eight. Compressing 44 requirements into
five phases would have produced phases with no coherent verifiable capability, which the
granularity guidance says to avoid: derive phases from the work, then use granularity as
compression guidance rather than as a target. Two thin candidates were folded rather than left
standing: composing and reading became one phase, and the accessibility scan coverage target
joined the feedback phase instead of the measurement phase.

**Blockers known at roadmap time.**

- SHIP-01 is blocked on a certificate decision that is Pratik's.
- PIM-04 needs a sync target chosen before anything can be built.
- SHIP-04 is a decision before it is an implementation.
- SCALE-01 depends on what async-imap 0.11.3 actually exposes; the fallback is already
  specified in the mail-at-scale plan.

- Nothing in Phases 1 to 8 can be finished against a real mail account, because no account has
  ever been used. Where a requirement's last mile needs one, the criterion stops short and says
  so.

**Project skill in play.** `.claude/skills/cutting-a-release/SKILL.md` owns the mechanics of
publishing: the Release workflow runs only on manual dispatch, the level chosen decides whether
GitHub publishes a prerelease or a full release, and `docs/changelog.md` must carry an
`[Unreleased]` entry for every user-visible change before dispatch. Phase 7 follows it rather
than inventing a release path.

**Every task in every phase is red, green, refactor.** `workflow.tdd_mode` is `true`, so every
eligible task is `type: tdd` with RED and GREEN gate commits. `bash scripts/check.sh` must stay
green: fmt, clippy with `-D warnings`, tests, release build.

---
*Roadmap created: 2026-08-29*

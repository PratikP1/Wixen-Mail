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

**Corrected 2026-09-16: the milestone continues with what testing found.** The paragraph
above said the milestone was the two "not built" sections and nothing else, and that was
true until phase 8 closed on 2026-09-16 with every phase executed and every plan merged. The
milestone is not archived, because its verification is the manual testing Pratik began on
2026-09-15 with installer `0.125.1+g3e633252`, and that testing produced 44 GitHub issues,
#20 to #63, in one day. Pratik agreed an order for them on 2026-09-16 in seven groups, and
phase 9 is the first two: the version becomes `1.0.0-alpha.1` so every fix lands under the
number it will ship as, and the cause-known defects an hour to a day each are fixed test-first.
The five later groups, named in phase 9's README, are later phases and are not planned yet.
The first sentence of the paragraph above, that nothing here has ever run against a real mail
account, stopped being true the same day: #22 is the first thing a real Google account said to
this program, and it belongs to the last of the seven groups.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Folders and conversations** - Make and manage folders, nest them, pin them, and collapse the list to one row per conversation
- [ ] **Phase 2: Search that says what it covers** - The scope selector scopes, the coverage is disclosed, and a rule can be a folder
- [ ] **Phase 2.1: What phase 1 found on its way past** (INSERTED) - Thirteen defects and stale documents that belong to no other phase
- [ ] **Phase 3: Mail at scale on the wire** - Resume rather than re-list, hold one connection, fetch a whole mailbox, and never pick a conflict winner silently
- [x] **Phase 4: Writing and reading a message in full** - Attachments in and out, inline images with alt text, spell check while typing, and PGP
- [ ] **Phase 4.1: Mail moves between accounts** (INSERTED) - The move and copy window is a tree built for several accounts and fed one, so mail cannot cross an account and nothing says so. Goes before phase 5, because a reminder moving between accounts would otherwise be the first cross-account move in the program
- [x] **Phase 4.2: What was built and never reached** (INSERTED) - Thirteen findings from the sweep of 2026-09-06, all of them code that runs, is tested, and is quietly narrower in production than its own design. Undo Send, scheduled send, meeting replies that reach the organiser, accepting an invitation reaching the calendar, the Blocked Senders screen
- [ ] **Phase 5: The other five modules keep up** - Move and copy everywhere including contacts and reminders, recurring events across weeks and months, and a provider task move that survives being half finished
- [ ] **Phase 5.1: Notes and contacts reach a server** (INSERTED) - The notes seam and its first backend, and CardDAV. Cut from phase 5 at the boundary between moving what is already here and reaching a server for the first time
- [ ] **Phase 5.2: Notes in OneNote** (INSERTED) - The second notes backend, separated because a OneNote page has no ETag and cannot come back character for character, so PIM-04's criterion has to be decided rather than met. Phase 5.1 then found the criterion also fails for a calendar server, so this phase answers it for both backends rather than discovering it
- [ ] **Phase 6: How the application speaks** - Per-event feedback channels, dates in the user's language, and a scan that names what it cannot judge
- [ ] **Phase 7: Installing, updating and what is stored** - A signed installer, an update check, shortcuts, the cache encryption decision, and the other two platforms
- [x] **Phase 8: Every number the project quotes** - Replace the estimates with measurements
- [x] **Phase 9: What the first day of testing found** - The version becomes `1.0.0-alpha.1`, and twelve cause-known defects from the first day of manual testing are fixed test-first, each an hour to a day. Complete 2026-09-17, all ten plans merged; criteria 5 and 6 open on one clause each until a push of `main` runs the Accessibility and NVDA workflows

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

- [x] 01-01-PLAN.md: A tracer that creates a folder end to end, encoder included (D-41)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 01-02-PLAN.md: A conversation identity that is stored, and the two indexes (D-39)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 01-03-PLAN.md: Nesting stored as a parent link, and local names that contain the separator (D-22, D-23)

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 01-04-PLAN.md: Rename the leaf, move the subtree, delete deepest first (D-26)

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 01-05-PLAN.md: The tree's shape, with account branches, "On this computer" and identity keying (D-13, D-15 to D-17, D-21, D-25)

**Wave 6** *(blocked on Wave 5 completion)*

- [x] 01-06-PLAN.md: Moving accounts, counting unread, and the settings guard (D-14, D-24, D-42, D-43)

**Wave 7** *(blocked on Wave 6 completion)*

- [x] 01-07-PLAN.md: Five local folders shared, and the migration that gets there (D-18 to D-20, D-40)

**Wave 8** *(blocked on Wave 7 completion)*

- [x] 01-08-PLAN.md: Favourites (D-28 to D-32)

**Wave 9** *(blocked on Wave 8 completion)*

- [x] 01-09-PLAN.md: Empty a folder and mark one read (D-33 to D-38)

**Wave 10** *(blocked on Wave 9 completion)*

- [x] 01-10-PLAN.md: A folder the server stopped listing (D-27)

**Wave 11** *(blocked on Wave 10 completion)*

- [x] 01-11-PLAN.md: What a conversation is and what its row says (D-02 to D-04, D-08)

**Wave 12** *(blocked on Wave 11 completion)*

- [x] 01-12-PLAN.md: Switching the view, and what survives it (D-01, D-05 to D-07, D-09 to D-12)

**Wave 13** *(blocked on Wave 12 completion)*

- [x] 01-13-PLAN.md: Rethread as mail arrives, including the two-tree merge (THREAD-02)

**Wave 14** *(added 2026-08-31, after phase verification found criterion 3 undelivered)*

- [x] 01-14-PLAN.md: The tree holds every account, and moving between them is a selection (criterion 3)

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

- [x] 02-01-PLAN.md: A tracer for the read dimension on `Allowed`, end to end from the stored settings file to the one fetch that already exists (D-2-06, D-2-07, D-2-11, D-2-12)

**Wave 2** *(blocked on Wave 1)*

- [x] 02-02-PLAN.md: What a body-reading saved search covers, said before it runs, naming which search it is about (D-2-08, D-2-13)

**Wave 3** *(blocked on Wave 2)*

- [x] 02-03-PLAN.md: Fetching the missing text, behind the gate, marked experimental where somebody meets it (D-2-08)

**Wave 4** *(blocked on Wave 3)*

- [x] 02-04-PLAN.md: One vocabulary, so the filter dialog offers the eleven fields and eleven match types the engine answers, in words

**Wave 5** *(blocked on Wave 4)*

- [x] 02-05-PLAN.md: A saved search keeps both halves of its scope, and says what it asks (D-2-03, D-2-04, D-2-14)

**Wave 6** *(blocked on Wave 5)*

- [x] 02-06-PLAN.md: Writing a whole question list back atomically, and a dialog for one condition (D-2-01)

**Wave 7** *(blocked on Wave 6)*

- [x] 02-07-PLAN.md: The rule editor, a manager over one search's conditions, reached from the tree, one group however a search was made (D-2-01, D-2-02)

**Wave 8** *(blocked on Wave 7)*

- [x] 02-08-PLAN.md: Saved searches inside the account structure, and a search that runs against its own account (D-2-05)

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

- [x] 02.1-01-PLAN.md: The checks that read the main window read all of it, and what that finds is reported before it is fixed (criterion 4, D-2.1-01)

**Wave 2** *(blocked on Wave 1)*

- [x] 02.1-02-PLAN.md: A guard that reads documents can see a violation, and nothing says a test cannot build a window (criteria 7, 8)

**Wave 3** *(blocked on Wave 2)*

- [x] 02.1-03-PLAN.md: Every page that says folder management is missing (criterion 5)

**Wave 4** *(blocked on Wave 3)*

- [x] 02.1-04-PLAN.md: A guard under `tests/` runs on the commits that could break it (criterion 9)

**Wave 5** *(blocked on Wave 4)*

- [x] 02.1-05-PLAN.md: Two dialogs stop leaking a registry entry per row (criterion 1)

**Wave 6** *(blocked on Wave 5)*

- [x] 02.1-06-PLAN.md: Safety sorts by how bad it is, and one spelling of a message identifier (criteria 2, 3)

**Wave 7** *(blocked on Wave 6)*

- [x] 02.1-07-PLAN.md: Two decisions nothing was asking about (criteria 6, 13)

**Wave 8** *(blocked on Wave 7)*

- [x] 02.1-08-PLAN.md: A branch row offers a menu that fits it, and two accounts of one name are two rows (criteria 11, 12, D-2.1-03)

**Wave 9** *(blocked on Wave 8)*

- [x] 02.1-09-PLAN.md: A dialog refuses a value it cannot show rather than rewriting it (criterion 10, D-2.1-02)

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

**Plans**: 9 plans, one per wave, of which 8 are executed and none merged.

- [x] 03-01-PLAN.md
- [x] 03-02-PLAN.md
- [x] 03-03-PLAN.md
- [x] 03-04-PLAN.md
- [x] 03-05-PLAN.md
- [x] 03-06-PLAN.md
- [x] 03-07-PLAN.md
- [x] 03-08-PLAN.md
- [ ] 03-09-PLAN.md

The four decisions `03-RESEARCH.md` left for
Pratik were answered on 2026-09-03 and the plans carry the answers: build a seam
over how deletions are found and take the UID comparison behind it rather than
QRESYNC, leave the offline false promise until plan 03-08 rather than fixing it
sooner, attack the permanent body migration as well as proving the storage split,
and build the conflict choice for contacts and CalDAV plus a fix for the mail
defect that is not a conflict. One question is still open, in 03-07: which
announcement topic a whole-folder fetch belongs on.

- [ ] `03-01-PLAN.md`: Nothing deletes cached mail on the strength of a partial listing, and a renumbered folder says what it discarded
- [ ] `03-02-PLAN.md`: Count, in a test, the sign-ins that go round the helper, so the number stops going stale in a document
- [ ] `03-03-PLAN.md`: Prove the storage split that already ships, stop a migrated database paying for the migration on every open, and pin the numbering rule a dispatcher currently holds
- [ ] `03-04-PLAN.md`: Gmail mail archived with no label counts toward its conversation, by identity rather than by folder
- [x] `03-05-PLAN.md`: A conversation root that arrives late merges, and the backfill that makes the fix visible on mail already stored
- [ ] `03-06-PLAN.md`: One session held open per account, one reconnect, and a budget with a number
- [ ] `03-07-PLAN.md`: Resume a folder instead of re-listing it, behind a seam over how deletions are found, and let somebody ask for a whole one
- [ ] `03-08-PLAN.md`: Offline mode does what it says, the network is noticed, and coming back is offered rather than done
- [ ] `03-09-PLAN.md`: The conflict choice is built where the state occurs, contacts and CalDAV, and the flag change lost to an unreachable server is kept instead of undone

**Inherited from phase 1** (see `.planning/phases/01-folders-and-conversations/deferred-items.md`):

- ~~Gmail mail archived with no label vanishes from a conversation count, because D-08 excludes All Mail by folder rather than by message identity.~~ Closed by `03-04`. "One extra predicate in one query" was wrong three ways: there are two queries and they must change together, the `here` CTE had no join to `folders`, and a per-row rule needs the set of messages held elsewhere, which is a third CTE. About 90 lines of SQL.
- ~~A conversation root arriving after a message that already names it is not merged, so three of six arrival orders over such a set merge. One table, one index, one writer.~~ Closed by `03-05`. "One table, one index, one writer" left out two things that cost more than the table: a backfill, without which the fix reaches only mail that has not arrived yet, and the winner rule, which made the arriving message's conversation win and so gave the same three messages different names depending on which arrived last.
- ~~`next_local_uid` hands out 0 after the number range wraps, because it saturates on `i64` and then casts to `u32`. Not reachable in any database this program can currently produce.~~ Closed by `03-03`. Struck here by `03-05` rather than by `03-03`'s own docs commit, which updated `REQUIREMENTS.md` and `deferred-items.md` and missed this line; `03-04` found it and left it alone rather than edit another branch's record, and this branch holds both.

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

**Plans**: 9/9 plans executed, one per wave. Three shared files, `guards/guards.toml`,

- [x] 04-01-PLAN.md
- [x] 04-02-PLAN.md
- [x] 04-03-PLAN.md
- [x] 04-04-PLAN.md
- [x] 04-05-PLAN.md
- [x] 04-06-PLAN.md
- [x] 04-07-PLAN.md
- [x] 04-08-PLAN.md
- [x] 04-09-PLAN.md

`docs/changelog.md` and `Cargo.toml`, are touched by most plans under the
same-commit rules, and `src/presentation/wx_app.rs` by six of them, so the plans
are ordered rather than run in parallel. Plans 07 and 09 carry blocking
checkpoints and are not autonomous. Plan 08 carried one until 2026-09-06, when
the decision it held was settled and the plan became a build.

- [ ] `04-01-PLAN.md`: An attachment's own description arrives and is spoken, and an image with none borrows the alt on the `<img>` that names it
- [ ] `04-02-PLAN.md`: `List-Unsubscribe` arrives, so the mailing-list warning that ships and has never fired reaches somebody
- [ ] `04-03-PLAN.md`: The encryption facts computed on every message read stop being thrown away, so armour has an explanation beside it
- [ ] `04-04-PLAN.md`: A text attachment opens as text, and a picture says what is known about it and is then shown
- [ ] `04-05-PLAN.md`: A filter rule may name the safety verdict, and every sentence in the warning bar says who said it
- [ ] `04-06-PLAN.md`: A key moves between misspellings in both directions without a dialog, and says what a word could be instead
- [ ] `04-07-PLAN.md`: Several files at once by dropping, pasting or picking, with an honest answer about whether a drop on a web view lands
- [ ] `04-08-PLAN.md`: The inline picture draft round trip is proved, a picture may be marked decorative on purpose, and the reader decides whether they hear about it
- [ ] `04-09-PLAN.md`: An S/MIME encrypted message says why it cannot be read instead of opening blank, then an OpenPGP crate is chosen and PGP reading is built on it

Two of the four decisions from `04-RESEARCH.md` were answered on 2026-09-05 and
are built in rather than assumed.

**READ-02 takes on OpenPGP as well as the S/MIME half**, so `04-09` no longer
asks whether. Its first task wires the S/MIME sentence that already exists and
stops a message opening blank; its second chooses a crate, runs the package
legitimacy check and names the credential store entry, which is permanent once
written because the code that erases secrets must name what the code that wrote
them named; its third builds against the interface so a different choice costs
one adapter. Criterion 5 will not close in this phase: reading a real PGP
message needs a real key and real mail, and a key and a message made by the same
crate in the same test prove only that the crate agrees with itself.

**An image previews, which means described and shown.** Accessibility first is
not accessibility only. `04-04` says what is known about a picture first, so the
accessible half is not hostage to the decoding, and then shows it. That ordering
protects the accessible half and is not a ranking. The decoding turned out far smaller than
feared: `image` is already a direct dependency and already decodes ICO, PNG and
BMP in this binary, so JPEG is a feature flag rather than a new adoption, and
only the transitive crates the flags add are audited.

**Whether a picture may be marked decorative was settled on 2026-09-06**, and
not as one of the three options that were written: a decorative path exists,
narrowed to where furniture is plausible, plus a setting that decides whether a
decorative picture is announced to the person reading. That last part is what
makes the rest safe. A decorative mark buys silence where silence is right and
is also the fastest way past a prompt, so a photograph gets marked decorative
and the reader is told nothing at all; moving the final say to the receiving
side means the sender's mark is no longer the last word. `04-08` builds all
three and its checkpoint is gone. The setting is reachable from the settings
screen in the same plan that introduces it, because criterion 8 of phase 1 says
a phase must not add another setting the model holds and no screen offers.

One decision remains open. `04-02` assumes `List-Unsubscribe` is in scope and
lifts out whole if it is not.

**UI hint**: yes

### Phase 4.2: What was built and never reached (INSERTED)

**Goal**: Wire up the capabilities two sweeps found written, tested and reached by nothing, and correct the documents that describe them as working.
**Depends on**: Phase 4, all of it. Plan 01 wrote a setting into `src/data/config.rs` and `src/presentation/wx_settings.rs` and every other plan chained off it.
**Requirements**: none new; this closed recorded defects rather than adding capability, with two exceptions noted under criteria 3 and 10
**Success Criteria** (what must be TRUE):

  1. A message sent from the composer is held before anything hands it to a server, goes on its own when the hold runs out, says while it waits that it is waiting, and can be taken back with Undo Send.
  2. How long the hold lasts is a setting on the Compose tab under Sending, described with what turning it off costs.
  3. A message can be set from the composer to go at a date and time somebody chooses, it waits in the Outbox saying the time it is set for, and it goes on its own when that time comes.
  4. A time that has gone by is refused with the reason and the next move, not sent and not quietly moved to now; so is a time more than a year ahead, and so is text that is not a date and time.
  5. A meeting reply arrives at the organiser declared `text/calendar; charset=utf-8; method=REPLY`.
  6. The part the organiser receives is called `reply.ics`.
  7. Accepting a meeting puts it on the calendar, on the day, at the time it is at, taking up that time according to the answer given.
  8. Answering the same meeting twice leaves one entry, and a changed meeting replaces the one already there.
  9. Somebody reading a message with pictures held back is told how many, why, and where the switch is, once, in the document.
  10. Somebody can find out who they have blocked, from the Tools menu, and take a block off there, and unblocking one address never removes a wider block that happens to catch it; and somebody making a block is told what blocking will do before the rule is written.
  11. `Shift+F6` leaves the message preview in the opposite direction to `F6`, and nothing in the tree says the preview never takes focus.
  12. A column layout made in Sent is stored as a Sent layout and does not become the inbox's, and Restore Defaults restores the defaults for the folder somebody is in.
  13. Every key the shortcuts document promises is a key that arrives, and every key that is bound is documented.
  14. Five sentences that describe something other than what ships are corrected: three changelog entries about braille, `parse_ical_vevent`'s doc comment, and two evidence cells in `.planning/intel/built-and-left.md`. Plus two code corrections: the settings screen telling somebody Windows overruled a setting they never turned on, and the scrolling module claiming two readers where there is one.

**Plans**: 9/9 plans executed, one per wave. `docs/changelog.md` and `Cargo.toml` were touched by every plan under the same-commit rules and `src/presentation/wx_app.rs` by seven, so the plans ran in order rather than in parallel.

- [x] 04.2-01-PLAN.md: A message is really held, so Undo Send takes something back
- [x] 04.2-02-PLAN.md: A message can be set to go at a chosen time, and a time that will not do is refused with a reason
- [x] 04.2-03-PLAN.md: A meeting reply arrives declared as a reply, named `reply.ics`
- [x] 04.2-04-PLAN.md: Accepting a meeting puts it on the calendar, once, taking up its time
- [x] 04.2-05-PLAN.md: A reader is told how many pictures were held back, why, and where the switch is
- [x] 04.2-06-PLAN.md: Blocked Senders is on the Tools menu, a block can be taken off there, and making one says so first
- [x] 04.2-07-PLAN.md: `Shift+F6` keeps its direction crossing the message preview
- [x] 04.2-08-PLAN.md: A column layout belongs to the kind of folder it was made in
- [x] 04.2-09-PLAN.md: The documents name the keys that work, and five sentences stop overclaiming

**This section was written when the phase finished, not when it was planned.**
The ready-to-paste block sat in `PLANS-README.md` for the whole phase and only
the one-line entry above ever reached this file, so for nine plans the criteria
being worked to were not in the roadmap at all. Recorded rather than quietly
fixed, because a phase whose criteria live only in a planning README is a phase
nobody can audit afterwards, and phase 8 is the audit.

**What the phase did not close.** Nothing in it has been heard with a screen
reader: two checkpoints were planned, in `04.2-04` and `04.2-06`, and both were
deferred by decision and never attempted. Nothing in it has met a real server, a
real organiser or a real provider. Twenty-eight ledger entries, 147 to 174, are
open and none has been answered. The braille defect is closed only in the
documents; the two tick boxes still say they control speech and braille
separately and on Windows they do not, which belongs to phase 6 along with the
two tests whose names promise the independence. `04.2-09-SUMMARY.md` carries all
of it in one section, including the single command for the guard sweep the phase
owes: `scripts/guards.sh --touched-by 9611b70`.

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

**Criteria 4, 5 and 6 are not phase 5's to close, and are left above so the split is visible rather than tidied away.** The nine-plan phase 5 was cut into three when it was planned, at the line between moving what is already on this computer and reaching a server for the first time. Criterion 6 and the CalDAV half of 4 and 5 belong to phase 5.1. The OneNote half of 4 and 5 belongs to phase 5.2. Both restate their half in their own words below rather than pointing back here, because a criterion nobody can read without following a cross-reference is one nobody checks. Phase 5's eight plans closed 1, 2 and 3.

**Plans**: TBD

- [x] 05-01-PLAN.md
- [x] 05-02-PLAN.md
- [x] 05-03-PLAN.md
- [x] 05-04-PLAN.md
- [x] 05-05-PLAN.md
- [x] 05-06-PLAN.md
- [x] 05-07-PLAN.md
- [x] 05-08-PLAN.md

**UI hint**: yes

### Phase 5.1: Notes and contacts reach a server (INSERTED)

**Goal**: Notes reach a server through one seam that knows nothing about which backend it is talking to, and contacts get a second address book by its own address.
**Depends on**: Phase 5, which moves notes and contacts around on this computer before either of them reaches a server
**Requirements**: PIM-04, PIM-05, PIM-07, PIM-08
**Success Criteria** (what must be TRUE):

  1. A local note is a first-class Markdown document, and the database agrees. The `notes.format` column has held the word "plain" on every row since notes shipped, while the editor labels the box "Body, in Markdown" and the reader parses it as Markdown. That column either means something or is honestly retired. A note typed here, saved, closed and read back is byte-identical, proven by a test that goes through storage rather than through a formatter.
  2. A synced note reaches its account's backend through one seam, and CalDAV VJOURNAL is the first thing behind it. A note deleted here stays deleted, which needs a record of the deletion that the other three modules have and notes do not.
  3. Where an account type has no notes backend, the settings screen says so rather than offering a switch that does nothing. It says this account has no notes backend, not "not yet": a consumer Gmail account still has none after all three backends ship, because Google Keep's API is Workspace only.
  4. The seam takes a second implementation without changing shape, proven by writing one rather than by arguing it. The second is shaped from the four things OneNote really does, so it disagrees where a real second backend would, and it lives only in tests. Phase 5.2 replaces it with the real client and reports where it was wrong.
  5. A user adds a CardDAV address book by its own address, and contacts sync both ways through the vCard reader and writer that already exist. That needs somewhere to keep the address, its credentials owner, its change marker and whether it is visible, and no table or column can hold any of those today.
  6. The new address book screen is heard with a real screen reader before the phase closes. It is the one thing in this phase no test in this repository can settle.

**Nothing here can be finished against a real server.** No account, no CalDAV server and no CardDAV server has ever been used with this program. Every parser these plans build is tested against text this repository wrote, and a round trip proves a reader and a writer agree with each other and says nothing about anybody else's server. Each plan owes `.planning/WINDOWS.md` an entry per unrun thing rather than one entry covering all of them.

**Plans**: TBD

- [x] 05.1-01-PLAN.md
- [x] 05.1-02-PLAN.md
- [x] 05.1-03-PLAN.md
- [x] 05.1-04-PLAN.md
- [x] 05.1-05-PLAN.md
- [x] 05.1-06-PLAN.md

**UI hint**: yes

### Phase 5.2: Notes in OneNote (INSERTED)

**Goal**: OneNote goes behind the notes seam, and what the seam turns out to have assumed about CalDAV is written down rather than absorbed.
**Depends on**: Phase 5.1, which builds the seam and its first backend
**Requirements**: PIM-07, and the second half of PIM-08's proof
**Success Criteria** (what must be TRUE):

  1. A note reaches OneNote through the same seam, with no special case inside the seam for the fact that a OneNote page is an HTML document inside a section inside a notebook rather than a title and a body.
  2. The seam holds a backend that has no ETag. OneNote's concurrency is a timestamp the service owns, a page's body cannot be replaced, and most replaces need an identifier Graph generated and may move, so every write is preceded by a read of the page's current identifiers.
  3. What the mapping loses is written where a user meets it, not discovered by them. A OneNote page cannot come back character for character, so PIM-04's byte-identical criterion is decided rather than met, and the decision is recorded with its cost. It covers both backends: phase 5.1 found on 2026-09-10 that a calendar server cannot round-trip a carriage return either, so the criterion is answered once for everything that syncs, and the answer says which backend loses which bytes rather than folding them together.
  4. Where phase 5.1's seam had assumed CalDAV is reported, including every place the test-only second implementation turned out to be wrong. That report is the second half of PIM-08's proof, and this phase is required to produce it rather than to conclude the seam was fine.
  5. The two green tests that assert notes have no backend say the true thing afterwards: inverted for Outlook accounts, still true for Gmail ones.

**Plans**: TBD

- [x] 05.2-01-PLAN.md
- [x] 05.2-02-PLAN.md
- [x] 05.2-03-PLAN.md

**UI hint**: yes

### Phase 6: How the application speaks

**Goal**: The user controls what is spoken, brailled, sounded and shown, reads dates in their own language, and the project knows which parts of WCAG its scans can and cannot judge.
**Depends on**: Phases 1 to 5, so the scan coverage list covers the surface those phases add
**Requirements**: FEEDBACK-01, FEEDBACK-02, FEEDBACK-03
**Success Criteria** (what must be TRUE):

  1. A user sets Earcon and Visual independently for each of the sixteen events from the Settings Feedback tab, by keyboard, and the setting survives a restart. Speech and Braille are set **together**, as one choice, and the screen the user meets says that choosing between them is done in their screen reader rather than here.

     **This criterion was rewritten on 2026-09-06 because the original could not be built.** It asked for Speech and Braille to be set independently. They ride a single `UiaRaiseNotificationEvent`, whose declared signature takes no medium parameter at all, so whether a notification is spoken, brailled or both is the screen reader's decision and not this program's. Four readings agree: the call site, the module comment above it, `screen_reader.rs`, and `docs/accessibility.md`, which has said the honest version all along.

     The `||` in `accessibility.rs` that releases the notification when either channel is on is deliberate and correct, and there is a test named for the case with a comment explaining it: requiring both would leave a deaf-blind user with nothing when a send fails. So the defect is not the routing. It is that Settings offers two independent tick boxes and one changelog entry promises independent control, both of which describe something that cannot happen. Correcting those three false sentences belongs to phase 4.2; offering the honest control belongs here.

  2. Month names, day names and relative wording follow the machine's locale, falling back to English silently where there is no translation.
  3. The accessibility scan output names which WCAG 2.2 AA success criteria it can and cannot judge, so "roughly half" becomes a list.
  4. The interactions only a human screen reader pass can cover are written down as a scoped list, and each of the five WebView2 findings is either fixed or recorded as upstream with the upstream named.

**Plans**: nine since 06-09 was added on 2026-09-14, listed in `phases/06-how-the-application-speaks/README.md`.

- [x] 06-01-PLAN.md, merged at `dacaa719`. Criterion 1 does not close and is not meant to: every one of its four clauses is about a screen, and this plan is the half that is not one. What landed is a model that can say what somebody chose apart from what they get. `what_was_chosen_for` answers the choice as it was made, telling "no override" apart from "all four ticked", which the only public reader before this could not do: `channels_for` defaults a missing entry to every channel, drops the globally switched-off ones and adds a braille tick where only a sound was picked, so a panel built on it would have shown somebody four ticks they never put there. `use_the_default_for` removes the entry rather than emptying it, because an empty set round trips and means silence. `set_event_channels` is public, where before it had eleven references and every one was in its own file, so the only route into an override was editing the stored string by hand. `enum Event` and `Event::ALL` now come from one list: the four exhaustive matches always forced a new variant to be described and nothing forced it into `ALL`, and a variant missing from `ALL` had no control, no sound scheme slot and no test coverage. That was measured rather than argued, on both sides, because no test could express it: a seventeenth variant absent from `ALL` left the whole library green at 7,118 tests, and the same variant in the list is in `ALL` without anybody touching `ALL` while removing it is four compile errors. That measurement is the plan's one declared test-first exception and it carries no guard record, because the break fails to compile rather than reddening a test. `Channel::ALL` has the same hole, is out of scope, and is `WINDOWS.md` 343; FEEDBACK-01's evidence is now wrong about the tree and is `WINDOWS.md` 344, left for decision 7 at 06-07.
- [x] 06-02-PLAN.md, merged at `3969208e`. Criterion 1 does not close, and it has five clauses rather than the four 06-01's summary names: that reading folds "by keyboard" into the first, which is the clause this plan can least attest to. Four close structurally and none is heard. The sixteen per-event answers are reachable from a screen for the first time: a picker of the sixteen read from `Event::ALL`, three controls, a button that removes an answer rather than emptying it, and two lines. The ticks are painted from `what_was_chosen_for` and the first line from `channels_for`, because those are different questions and painting the ticks from the second would show somebody answers they never gave. The second line is what keeps the ticks honest, saying what the event will really produce, since a channel switched off everywhere stays off and an event left with only a sound has a written channel added back; the rule was not weakened to match the screen. The two global boxes are one, built from `Switch`, whose three answers name all four channels between them because a notification rides one `UiaRaiseNotificationEvent` that takes no medium parameter. `grep 'for channel in Channel::ALL'` in the settings dialog finds nothing. Two clauses were open when the plan's own tasks finished and its success criteria claimed both: that pressing OK saves a per-event answer, and that the screen says whose decision speech or braille is. Reading the criterion from this file clause by clause found them, nothing in the tree would have, and both now have a check taken red by hand. Five guard records, not the two the plan asked for, because a record guards a rule and the plan counted per task; all five were re-run through `scripts/guards.sh` itself. The two `house_style` settings guards did not redden on arrival as the plan promised, because they fire on controls written the wrong way and these were written the right way first; each was instead shown to see a planted violation in the new code. `WINDOWS.md` 345 to 353: nobody has tabbed through the tab, nobody has heard the panel, the controls reloading beneath the cursor, or the sentence, and that the two handlers really call the functions the tests drive is proved by reading two lines, because wxdragon exposes no way to raise a widget event from a test.
- [x] 06-03-PLAN.md, all four tasks merged, tasks 3 and 4 at `39417f88`, still version 0.121.0. **Criterion 2 closes structurally, clause by clause, and FEEDBACK-02 with it; nothing in it has been heard.** Month names: `date_display`'s three readings, the appointment form's month list, and the eight signature sentences, which now go through the wrapper with the month in the form a date puts it in. Day names: the repeat-series sentence asks the machine, and a French computer hears "every week on mardi and jeudi", a French day inside an English frame that a test names as such and the changelog says out loud. Relative wording: "2 days ago" comes out of `locales/en-US/dates.ftl` through Project Fluent, the first piece of version 2 on Pratik's answer of 2026-09-13, with the three settings Firefox ships these crates with held by tests, isolation off, numbers by `GetNumberFormatEx` in the bundle's own language, counts as numbers with the error list read, and a completeness check both ways. A Russian resource written inside a test produces the four Russian forms on this en-US machine and ships nowhere. The fallback: only an English catalogue exists, so `fr-FR` and `xx-YY` get the eight English sentences with nothing said, forced rather than read off the machine, and an unparseable name or a failed read asks for English by name rather than `und`. The manifest commit paid the whole gate alone, as the plan predicted: `cargo audit` reported nothing outside `.cargo/audit.toml` over the eight new packages, 698 to 706. A source-reading guard in the wrapper module holds that no shipped string literal names an English month or day apart from six allowed by name with a reason, two wire formats and four interface labels version 2 translates, with a companion that sees a planted violation. Task 1's guard record was stale inside its own plan, naming 4 where 48 go red, because callers arrived without any file gaining a test; corrected and measured. Ledger 365's transient failure has a cause, a lazy-initialisation race in `keyring` 4.1.5, kept this time. `ENGLISH_ONLY` narrowed a second time and kept: the dates follow this computer, the wording around them does not yet. `WINDOWS.md` 366 to 374. The earlier account, as written when tasks 1 and 2 merged at `e98514b0`: read clause by clause it has four: month names, day names, relative wording, and a silent English fallback. Month names close in `date_display`'s three readings and in the appointment form's month list, and do not close in the eight signature-outcome sentences that still write `%B`. Day names do not close at all. Relative wording is this plan's own blocking checkpoint, left unanswered on purpose, and task 3 is written against its answer. The fallback closes, and is wider than the clause asks: a corrupt stored day such as `--02-30` is refused by Windows and comes back English on a machine that does have the language. Two Win32 mechanisms rather than one, because Microsoft's own pages say a month looked up alone is the nominative form and only a picture holding a numeric day and `MMMM` yields the genitive; a Russian date reads "2 января" and a Russian month list reads "Январь", which is measured here by a passing test rather than taken on the documents' word. The shape stays the person's: their stored wording and order pick the picture and Windows supplies only the words, so month first on a French computer gives "juillet 26, 2026", which no French computer writes on its own. The English ordinal is gone, and `ordinal` with it rather than an `#[allow]`. `ENGLISH_ONLY` was reworded here rather than in task 3, because task 3 is behind the checkpoint and this build ships; the staged wording over-claimed and was corrected, since it said the month names follow this computer while the signature sentences still write English ones. The appointment form's month list is wired and untested, because the `Choice` is built inside a closure and needs a live window: `WINDOWS.md` 361. `WINDOWS.md` 360 to 364, five entries, one per unrun thing.
- [x] 06-04-PLAN.md, both tasks merged at `06aa8765`, version 0.122.0. **The inherited item from phase 1 closes structurally, read clause by clause; FEEDBACK-01's older half with it; nothing has been heard.** The per-account Allow Changes answer the program has honoured for some time has its first screen: three boxes on the account edit dialog's connection page, one per answer in `Allowed`, each built with a real label, each showing what `allowed_for` answers for this account, each unavailable and saying why where Settings has that answer off, because a per-account answer can only narrow and a box that looked like it could widen would lie. `set_allowed_for` is the one writer and keeps what an account narrows rather than the answer as ticked: a box that was unavailable records no narrowing, so the account follows Settings when that is later turned on, and an account that narrows nothing has no row. `allowed_for` is unchanged. Both checkpoint answers were settled before execution: all three answers per account, by Pratik on 2026-09-13; and the phase 7 sharing question moot, because phase 7 finished on 2026-09-12. `allowed_per_account` moved from `STORED_AND_OFFERED_BY_NOTHING` into `OFFERED_BY_ANOTHER_SCREEN`, traced rather than assumed: `wx_settings.rs` names it zero times, so a deletion would have failed the mirror guard. The guard watching the emptied list is retired in the same commit, taken red by hand first against the new control because no commit could carry it red, with the next entry's debt written on the list. The document guard that forbade a page from saying a permission could be set per account is retired with the reading only it used, no replacement written, and the testing page describes the control, saying it can only ever be less and that the sync's refusal still names Settings. Thirty-three guard records re-measured after the tree was final, six on `config.rs` not five, six on the account manager, twenty-one on `house_style.rs` not nineteen, every one still reddening exactly what it names. The "cannot widen" test the plan asked for already existed. The live dialog test exercised only the offered arm on this machine. `WINDOWS.md` 375 to 378.
- [x] 06-05-PLAN.md, both tasks merged at `7aad8722`, version 0.123.0, its summary complete since 2026-09-14; this box was left unticked when the progress row below was written and is ticked here by 06-09's executor from the summary on disk
- [x] 06-06-PLAN.md, both tasks merged; task 1 at `9f86ba6f`, task 2 on branch `a-pinned-scanner-and-every-window-it-can-reach`, red `741d2b36` and green `fd661401`, merged at `ca88d833`, still version 0.123.0. **Criteria 3 and 4 close nothing here, and are not meant to; this is the ground under 06-07's list.** Both checkpoint answers are Pratik's of 2026-09-14: pin the scanner with the zip's hash beside the tag, and "all of them, and record any that can't be reached". The scanner is Axe.Windows v2.4.2, read from the releases API in the session that wrote it, hashed two ways to a SHA-256 beginning `aeca43f4`, and refused unexpanded when the hash differs. Thirty targets where there were ten: fourteen top-level dialogs counted from the tree, 40 dialog windows in 38 `Dialog::builder` sites less the 9 already scanned and the 17 nested, plus the bare main window and its five other module panels. Every one was started on a throwaway profile on this machine and the window it names was seen; none is unreachable. Three defects in the scan itself, none in the plan, measured from the CI log of 2026-09-10: the CLI writes a result file only when it found errors, so seven clean windows had been reported as failed scans, fixed with `--alwayssavetestfile`; the MSAA script walked .NET's main window, which is never a dialog, so the channel NVDA reads had reported the same 1797 elements for three different dialogs and never read one, and it walks every top-level window now; and a modal that returns during a scan means the window is not open, so the program leaves with code 3 and the workflow says so rather than scanning the main window and passing, proved by closing the About window from outside. `main` is the frame under the first-run question and always was; `mail-module` is the bare window. Two guard records measured on the whole library, census 759. Nothing has been pushed, so no CI run has scanned any of the thirty on either channel, and the MSAA walk crashes PowerShell on this machine with NVDA running. `WINDOWS.md` 388 to 394, and 384 fixed.
- [x] 06-07-PLAN.md, both tasks on branch `roughly-half-becomes-a-list-of-fifty-five`, documents `4d415de3`, red `0ee95483`, green `4abe217d`, corrections `93f8c865`, summary `a35efbca`, merged into `main` at `984570b1`, still version 0.123.0. **Criterion 3 closes structurally, both clauses, and nothing in it has been run; criterion 4 closes nothing here.** Both checkpoint answers are Pratik's of 2026-09-14, recorded and not re-asked: the coverage list is both a document and a check, and `REQUIREMENTS.md` is corrected in place. Both counts re-taken from sources with the arithmetic shown: the pinned scanner's own `axe-windows-rules-2.4.2.md` has 155 rules, 61 + 53 + 23 + 9 + 9 by standard and 76 + 63 + 16 by severity, citing exactly three WCAG criteria, 1.3.1, 2.1.1 and 4.1.2, with the other 76 citing Section 508 and no WCAG criterion at all, the fourteen `Name` rules among them; and WCAG 2.2 has 87 criteria, 31 A + 24 AA + 31 AAA + one with no level, 4.1.1, so Level AA is 55. `docs/wcag-coverage.md` has fifty-five rows saying what each channel can and cannot say, the six regulatory exclusions attributed to Section 508 and EN 301 549 by name and split the way they split, the MSAA header quoted, thirty-one windows scanned and seventeen nested outside, and that no scan has yet run on either channel; it is written so it does not read as a compliance claim. The three criteria are code in `src/presentation/what_the_scans_can_judge.rs`, a reading holds the page's table to them in both directions, a companion plants a wrong row in each direction against the real page and both are caught, an empty page or an empty list fails rather than passes, and a test holds the scan's own step summary to the same three, read from the workflow's commands and not its comments. Five sentences corrected rather than the plan's four, because CLAUDE.md's guardrail 2 said "about half of WCAG" across a line break a single-line grep could not see; each keeps its old wording with the date, and the defects sentence in CLAUDE.md's accessibility section stands with the qualification it lacked. The workflow's header claimed the scanner measures contrast; the rule list has no such rule, and it says so now. `docs/*.md` maps to no scoped target, so the reading ran on every commit except one editing only the page it reads; one line in `check.sh`'s documents-only path closes that, shown working on the corrections commit. One guard record, the 2.1.1 row set to no, measured on the whole library at 7,190 passed and exactly two red, run through `scripts/guards.sh` and agreed, census 760. The three `REQUIREMENTS.md` sentences corrected in place, dated, old wording visible; phase 7 finished on 2026-09-12 so nothing conflicted. The plan's premise that the changelog line sits in a released-version note was wrong, it is under `[Unreleased]`, and it was still corrected by addition. Nobody has walked a criterion against this application and the applies column is one reader's judgement, `WINDOWS.md` 395 to 401.
- [x] 06-08-PLAN.md, done 2026-09-14: the first scan of thirty-one windows on two channels read one finding at a time, twenty-nine rows, nine fixed test-first and waiting for the next run, four WebView2's own with the upstream named, fourteen spinner and list text fields ledgered with recipes; the manual pass written as seventy-six items and not walked
- [x] 06-09-PLAN.md, all four tasks done 2026-09-14, tasks 2 to 4 on branch `one-window-for-every-due-thing`, merged into `main` at `abaa0667` with the whole gate green on the merge, 7,677 tests, version 0.124.0. Pratik's three answers applied: a dated task and an all-day event's alert base at `working_day_starts`, the default lead for silence and never for an explicit off with off now an empty list written by the three writers that know it, and a `held_alerts` table. One window, Due now, every kind a row with its kind first, snooze, snooze all, mark done, dismiss, dismiss all, and Details opening an event in the calendar window's own editor; a task or reminder has no editor anywhere, so Details is disabled with its reason there, ledger 437. Sixteen records measured, twelve re-measured. Nobody has heard any of it; `WINDOWS.md` 431 to 441. This plan carries no criterion of its own; it closes Pratik's widening of the second inherited item, and FEEDBACK-01 is the nearest requirement

**UI hint**: yes

**Inherited from phase 1** (see `.planning/phases/01-folders-and-conversations/deferred-items.md`):

- ~~A permission per account is stored, read by `allowed_for`, honoured out to the provider clients, and offered by no screen. This is FEEDBACK-01's exact shape already live in the tree, found by the mirror guard 01-06 added, and it belongs with the requirement written for that fault.~~ Closed by `06-04`, structurally: still stored, still read by `allowed_for` unchanged, still honoured out to clients that have never met a real one, and offered by the account edit dialog, a claim `test_a_setting_said_to_be_offered_elsewhere_really_is` now checks. "A control per account" in the deferred item meant one control when `Allowed` had fewer fields; it is three boxes now, one per field, which is what Pratik chose. Nobody has heard them.
- ~~A reminder alert still opens over somebody who is typing. It shares the one-at-a-time gate 01-10 built but does not ask the typing count. Whether a reminder should wait is a question about what a reminder is for.~~ Closed by `06-05`, structurally, with a residual that is the decision rather than a gap. It asks the typing count now, through the same two-part helper the folders question asks, held by a reading in `tests/wired.rs`. What a reminder is for was answered by Pratik on 2026-09-14: it is said and sounded at the look that finds it, its window is held one look and then opened whether or not typing has stopped, and the tone comes back once a minute until focus reaches the window, ten times at most. So it still opens over somebody who is typing, a minute later and after telling them; the plan's own words, "steals focus mid-word eventually, which is the thing being complained about, just later", are in the changelog. Nobody has heard any of it. The widening, one window for every due thing, is `06-09`.

### Phase 7: Installing, updating and what is stored

**Goal**: A build reaches a user signed, tells them when there is a newer one, tells them plainly what it leaves on their disk, and does not promise a publisher warning will be gone when the certificate chosen cannot buy that.
**Depends on**: Nothing new; can run alongside Phases 1 to 6
**Requirements**: SHIP-01, SHIP-02, SHIP-03, SHIP-04, SHIP-05, SHIP-06
**Success Criteria** (what must be TRUE):

  1. The published installer and the executable inside it both carry a valid Authenticode signature with a timestamp countersignature, verified against the published release asset rather than a local build. What SmartScreen then does is stated, not promised: no certificate available to this project removes the warning on a first download, since EV certificates no longer bypass SmartScreen and only publishing through the Microsoft Store avoids it. What signing buys is the publisher's name in place of "Unknown publisher", reputation that accrues across releases under one identity, and Smart App Control on Windows 11 no longer blocking the file. While the warning remains, `docs/installing.md` keeps the walkthrough that gets a screen reader user past it.
  2. The application can tell the user a newer version exists and can apply it, and nothing is fetched that the user did not ask for. Asking is one of two things: choosing the update item in the Help menu, which works whatever the setting says, or choosing a release channel in the one update setting, which starts off and says where it is chosen that choosing a channel means installers are downloaded automatically. With a channel chosen, the check, the download and the verification happen unattended; running the installer never does, and the user is asked first. A downloaded installer is verified before anybody is asked about it: the Authenticode signature must be valid and the signer must be this project's own publisher name, and a file that fails either check is refused and deleted rather than warned about and offered anyway. Where a signature cannot be checked at all, nothing is downloaded and nothing is run, and the reason is said. Declining, and a handover that fails, both leave the current version working.
  3. The installed shortcuts carry the application icon. The installer already creates both the desktop shortcut and the Start menu entry, so this is `IconFilename` on the two `[Icons]` entries and nothing else.
  4. The cache is not encrypted, and that is said once and clearly where a user meets it: the first-run screen and the page about what is stored. It says what the limitation is and is not, distinguishing another user of the same computer, kept out by Windows, from somebody who takes the drive out, who is not unless the disk is encrypted.
  5. The crate builds and the suite passes on Linux and on macOS in CI.
  6. On a platform where the accessibility bridge is absent, the application says so at startup and in Help, derived from what is actually compiled in rather than from a hardcoded platform list. This closes on the disclosure, not on a working bridge.

**Plans**: nine, listed in `phases/07-installing-updating-and-what-is-stored/PLANS-README.md`.

- [x] 07-01-PLAN.md, merged at `228e6a3`. Criterion 4 closes structurally and is unheard, which is `WINDOWS.md` 302 and 303. The last clause of criterion 1, that SmartScreen is stated and not promised, is now held by a failing build. SHIP-04 closes; SHIP-01 does not, because the signature itself is 07-07 and 07-08.
- [x] 07-02-PLAN.md, merged at `2d6ffeb`. Criterion 3 closes, and it was smaller and larger than it reads. Smaller because the shortcuts were never iconless: `build.rs` embeds the icon and Windows uses a file's own icon for a shortcut that names none, so nobody's picture changes. Larger because "`IconFilename` on the two `[Icons]` entries and nothing else" would have named a path nothing installs, which Windows falls back from in silence, so the `[Files]` line is part of it and `UninstallDisplayIcon` moved with them. The gate hole underneath it closes too: an installer change now earns the full gate, so the three tests that already read the script and the one this plan added all run on the commits that could break them. `WINDOWS.md` 306: nothing here has installed anything or looked at a shortcut.
- [x] 07-03-PLAN.md, merged at `85d84d3`. Criterion 6 closes, all three clauses, and it was larger than the criterion says. The criterion names one bridge and there are two: announcements, which had a status nothing read, and accessible names, which had no marker anywhere. A disclosure built from the status alone would have described the smaller half. Both now answer for themselves, each from whichever platform arm compiled rather than from `cfg!(target_os = "windows")`, so a third arm carries its own answer. `NativeBridgeStatus` and the two accessors that had never been called are removed rather than given a caller, because the constant says the same thing from the same source and has two real callers. `WINDOWS.md` 307 to 310: no build without the bridge has ever been made, so the sentences have only ever been produced from arguments a test chose, and nobody has heard them.
- [x] 07-04-PLAN.md, merged at `441fca5`. Criterion 2 does not close and is not meant to: nothing here asks anybody anything, stores anything or draws anything. What closes is the `[D]` line about comparison, including the prerelease ordering the release workflow can really produce. Two decisions travel to 07-05. `ReleaseChannel` is derived and never stored, which the plan asserted both ways and which is settled here on its own premise about where the setting lives, so there is no stored channel spelling and no unknown stored channel value. And a published tag carries a `v`, read off the glob `wixen-mail-v*.exe` in `release.yml` rather than from any tag, because none exists; without it the comparison would have refused every release this project cuts. `WINDOWS.md` 311 to 314.
- [x] 07-05-PLAN.md, merged at `d83bed3`. Criterion 2 still does not close, and this is most of what it needs short of applying: a Help menu item that asks GitHub whatever the setting says, one setting with three values under a new "New versions" heading on General starting on not looking, and five answers none of which claims a state the check did not learn. Nothing downloads and nothing runs; applying is 07-09's, after 07-08 gives it a signature to check. 07-04's ordering and offer decision are now reached by a path a person can take, which closes `WINDOWS.md` 313. Three things were measured off a real socket rather than read, and one changed what was built: this repository answers 404 on `releases/latest` and 200 with an empty array on `releases`, so "nothing published" arrives in two shapes and the plan knew one. A rate limit is 403 **or** 429 and is told from the other 403 by `x-ratelimit-remaining`, not by the status. `docs/privacy.md` stops promising there is no update check and gains the `%TEMP%` log fallback and the OneNote permission nothing uses. `WINDOWS.md` 315 to 322: no published release has ever been seen and nothing has been heard.
- [ ] 07-06-PLAN.md, one task of three merged, and the box stays unticked on purpose. Criterion 5 does not close and SHIP-05 stays open, because the answer the plan exists to get does not exist yet. What landed is the way to get it: `.github/workflows/other-platforms.yml`, `workflow_dispatch` only and `permissions: contents: read`, building the main crate and running its suite on `ubuntu-latest` and `macos-latest` as two jobs that do not depend on each other. Dispatch-only because a job nobody knows will pass must not go on a push trigger, which is how CI here stayed red for sixty runs across five weeks while looking maintained. Task 2 is a checkpoint nobody has run, so task 3, which adds the push jobs or records that this is a port, was not attempted. The guard that holds CI steps to `--no-fail-fast` read a hardcoded pair of files and now names three, taken red by hand against the new one. `WINDOWS.md` 323 and 324: nothing here has been run by GitHub, so not even the YAML has been parsed by anything that would know.
- [x] 07-07-PLAN.md, merged at `eab73a40`. The half of criterion 1 that is about this repository closes: a release that cannot produce one of the four files it promises stops and names it, before anything is published, with `fail_on_unmatched_files: true` as the second net rather than the first. The names the release writes and the names it publishes are held together by a test that reads both, in two categories, because one of the four is a tracked file nothing builds and a rule with a default category absorbs the fifth glob somebody adds later. Nothing changes about when a release can happen and a test says so rather than a sentence. Three of the seven tests had no red half, because the rules already held, and two of those carry a recorded break instead. Criterion 1's parenthetical is corrected from Microsoft's page re-fetched rather than quoted, and it has not moved; criterion 2 is replaced whole per D-14 and 07-09 is measured against it; SHIP-02 gains its fourth line per D-15 and loses the clause D-16 contradicts. The certificate decision is written down with the three options that lost. **Nothing is signed**: criterion 1 waits on an Azure account only Pratik can create, which is 07-08. One thing fixed on the way past: the guard-record coupling could not read a record whose break lands outside `src/`, so this plan's own records were invisible to the gate. `WINDOWS.md` 326 to 329.
- [ ] 07-08-PLAN.md, task 1 of three merged at `a818cf5f`, and the box stays unticked on purpose. Criterion 1 does not close and SHIP-01 does not close, because nothing is signed and there is no certificate to sign with. What landed is the question stated correctly: a census in `tests/installer.rs` that derives what has to be signed from the `[Files]` block, the workflow's published list and the script's own `Uninstallable` directive, and counts seven where SHIP-01's wording names two. A plan written to that wording would leave five unsigned, two of them executables a user runs. The census was taken red by hand with an eighth `Source:` line and named the new file rather than merely failing. The one unverified fact in `07-RESEARCH.md` is settled from the local Inno help and its premise was half wrong: the two-pass prompting behaviour is what a build with no `SignTool` gets, not a property of `SignedUninstaller`, so a CI job that signs at all never reaches it, and a signed uninstaller makes Setup write its messages to a separate `unins???.msg`. The portable copy and the zip are taken after `build-installer.sh` runs, so they will inherit whatever it signs. Task 2 is the Azure account only Pratik can create and is open; task 3 was not attempted, and the three shipped pages that say the build is unsigned are untouched and still true. `WINDOWS.md` 330 to 333.
- [ ] 07-09-PLAN.md, tasks 1 and 2 of three merged at `f0f20815`, and the box stays unticked on purpose. **Criterion 2 does not close and SHIP-02 does not close.** Twelve of its thirteen clauses are structurally complete; the second, "can apply it", is a claim about something that has happened, and nothing has applied anything. With a kind of version chosen, a published newer version is now fetched without anybody being asked at that moment, checked twice, and offered once. The second check is the whole point: `WinVerifyTrust` says a file is validly signed, which millions are, and only reading the signer's certificate and comparing the name says it is ours. The comparison is exact rather than "contains", because a certificate issued to "Pratik Patel Holdings Ltd" is one anybody can buy. **The refusal is proven and the acceptance is not**: a real Microsoft-signed system file was refused by name, so the mechanism works end to end against a genuine Authenticode signature, and no test of the accepting path has ever seen a file this project signed, because none exists. So as this ships every real installer is refused, which is the designed behaviour and is said on four pages. The checked file is a type only the check can construct, and both the question and the run take it, so there is no ordering to rearrange into "run it anyway?". Two features on the `windows` crate already pinned at 0.62.2, no new dependency, and streaming needs no `reqwest` feature at all, which the plan assumed it did. Two clauses of criterion 2 were found open by reading it clause by clause and were fixed with their own red halves: a computer with no way to check a signature was downloading first and refusing after, and the setting's own description still said "nothing is downloaded yet", four commits after that stopped being true. Task 3 is the screen reader checkpoint and was not attempted; it needs a published, signed release, which is 07-08's. `WINDOWS.md` 334 to 342.

### Phase 8: Every number the project quotes

**Goal**: Replace the estimates with measurements, so no figure in the documents is both aspirational and undated.
**Depends on**: Phase 3 for the scale targets; the rest can be measured against any build
**Requirements**: PERF-01, PERF-02, PERF-03, PERF-04, PERF-05, PERF-06, PERF-07
**Success Criteria** (what must be TRUE):

  1. Memory with 1,000 cached messages, cold start to a usable message list, and idle memory each have a recorded number carrying the date, the machine and the build it came from. **Closed 2026-09-15 by 08-03, read clause by clause 2026-09-16 by 08-09.** Three rows on `docs/development/measurements.md`, each dated 2026-09-14 at `9d5f15c5` with the machine and the release build named, each the median of five runs of a harness in the tree: memory with 1,000 cached messages, the application process 57 MB at its peak plus the WebView2 tree 333 MB; cold start to a usable list, 476 ms; idle at 120 s, the application 56 MB plus the tree 334 MB; and a fourth row for the empty-profile floor. "Usable" is a line the application writes once per process when the list first holds a row, held by a reading and a record. The criterion asks for numbers with their provenance and has them; which side of each target they fall is criterion 6.
  2. The message list is exercised against 200,000 synthetic rows, the sort, filter and scroll paths each produce a number, and a test asserts the virtual text callback issues no SQLite query. **Closed 2026-09-15 by 08-04, read clause by clause 2026-09-16 by 08-09.** Exercised: 200,000 rows from `sample_mailbox`, the Help menu's generator, written to a cache and timed with no window by `tests/the_list_at_two_hundred_thousand_rows.rs`, eighteen rows on `docs/development/measurements.md` at `5cf04528`. Sort: every order, 61 ms to 260 ms. Filter: at the box's limit, 78 ms to 110 ms, and every match of a one-in-five word, 193 ms. Scroll: the number is the page paint, `virtual_rows::text_for` over one page of every inbox column, 0.09 ms, which is the whole of what the paint callback does after taking the lock; wxWidgets' own paint of those cells and the list control taking a sort's result back were not timed, because the harness has no window, ledgers 449 and 450, and this clause is ticked with that sentence and not without it. The test: `tests/the_list_reads_only_memory.rs` holds `text_for` and the closure that calls it to naming no database, and by type the function takes slices and copies and cannot reach the connection the program holds; two records couple both to the reading.
  3. Every count in the documentation carries the command it came from and the date it was taken, and the documents agree with each other. Nothing asserts that a written number equals what a tool reports today, because that is false the next time anyone adds a test. Low coverage is attributed to the untested network transport rather than treated as a number to raise. **Closed 2026-09-16 by 08-09, read clause by clause, with the last clause revised under criterion 6.** The first clause: four readings in `tests/every_number_carries_its_command_and_its_date.rs` run on every commit, from 08-01 and 08-02, and 08-06 corrected by hand what no reading reaches; every figure this phase produced is a row on the page and nowhere else. The second: the three test-count pages quote one row and are held to it. The third holds by construction: the readings compare shape, never value. The fourth clause is revised, because 08-05 measured it false: the transport areas read 92.06%, 84.55% and 96.75% on 2026-09-14, above the library's 83.34%, and the low area is the 27 wxWidgets window files at 26.88%, holding 73% of the missed lines. The attribution that stands is written in PERF-05's evidence: those files build windows and `cargo llvm-cov --lib` opens none, the targets that do are outside that command, and a run with them would be a different quantity from the one the 23-point rise is measured in; so the figure is accepted with that reason beside it and is still not a number to raise. This clause said "the untested network transport" from 2026-08-29 until 2026-09-16, and that was true of the transport on 2026-07-26.
  4. One whole-tree mutation run completes, its report is read after the process exits, and every survivor is either killed with a test or recorded with a reason. Added 2026-09-14 by the planner: `cargo mutants --list` counts 12,335 mutants at `b14d6379`, and at the two terms a guard record costs that is weeks of idle machine time rather than the "about two days" the tree used to say, so this criterion is expected to be revised under criterion 6 once 08-08 has measured the rate on one shard and Pratik has chosen the run; read it as the question 08-08 puts to him, not as a promise this phase completes the whole run. **Revised 2026-09-16 under criterion 6, on Pratik's answer of 2026-09-15 to 08-08's checkpoint, with 08-08's product table as the reason.** The criterion now reads: one mutation run completes in shards on one commit, its report is read after the last shard's process exits, its never-started mutants are re-run by name, and every survivor is either killed with a test or recorded with a reason. What has been run: the four modules of 2026-08-01 (`application::filters`, `due`, `tagging` and `sign_off`, 157 mutants then, 223 today); the 25-mutant shard of `src/presentation/accessibility.rs` on 2026-09-15, four times under both suite shapes; and, on GitHub's runners at `3e633252` on 2026-09-15, `src/service/protocols/**`, 450 mutants in 18 shards, 316 caught, 41 nothing noticed, 91 the compiler rejected, 2 timed out, and `src/service/caldav.rs`, 420 mutants in 17 shards, 378 caught, 15 nothing noticed, 25 rejected, 2 timed out, both read whole by the merger, the four timeouts re-run and shown to be the mutants' own doing, and every one of the 56 survivors on `docs/plans/20260915-whole-tree-mutation-run.md`: 43 killed by tests shown red by hand at `96ade665`, 6 equivalent with the reason, 7 queued as untested behaviour, the TLS half of both mail protocols and one constructor that reads the machine's settings. What has not been run: the rest of the tree, 12,391 mutants at `2847391c` less those 870, at 130 s a mutant under every target in place plus 194 s a shard over 496 shards, about 19.8 days of this machine, or about 372 to 405 s a mutant on a runner as the two areas measured, two dispatches of 248 runners. The whole tree stays available: the same workflow, `file` empty, `shards=496`, in two dispatches, and the same merger reads it. PERF-07 is 08-09's to tick or revise against this.
  5. One whole-tree guard sweep completes, `scripts/guards.sh` unfiltered over every record in `guards/guards.toml`, and each record it reports short is corrected by hand and then re-measured. This is the one sweep of the milestone: by the decision of 2026-09-03 no sweep runs per merge or per phase, so nothing before this point has re-measured a record that only the whole sweep can reach. Expect about 20 hours and expect findings, since the tree will be many phases past the changes being judged. The 20 hours is the product row on `docs/development/measurements.md`, 784 x 92 s = 72,128 s, both terms taken 2026-09-14 by 08-01: 784 records by a TOML reader over `guards/guards.toml`, and 92 seconds a record from one record timed twice through the timing line `scripts/guards.py` now prints, rebuild 46 and 44 seconds plus run 47 seconds at eight threads over 7,245 library tests. That page is the only place the product is stated; this line quotes it. It said "roughly 15 hours" until 2026-09-14 and "about 19 hours", 783 x 86, for the rest of that day until 08-01 re-took both terms. **Closed 2026-09-15 by 08-07 on the count of records the log holds a verdict for: 803 of the 803 the file held at `df3437a1`, the merged log of run 34965790937 read back by `scripts/guards.sh --resume --stop-after 0` as "Every record selected has a verdict: 803 of 803".** Not on a machine and not in 20 hours: on GitHub's Windows runners in 41 shards, 4 h 7 min of wall clock and 64.7 hours of runner time, after Pratik widened the checkpoint to "Go for running the guard sweep via CI as well." 772 agreed; 31 did not, each measured again here before it was edited, 29 corrected by hand and measured again, 2 right here and blind on a runner. The census reads 802 swept at `df3437a1` and 3 arrived since.
  6. Each target is either met or revised with the reason written down. **Closed 2026-09-16 by 08-09.** Every target on `docs/roadmap.md`, `docs/development/requirements-backlog.md`, `docs/architecture.md` and `docs/integration-guide.md` carries its judgement on the line that carries the target, dated, by reference to a row on `docs/development/measurements.md`, with the old wording kept. Met: cold start, 476 ms against 2 seconds; coverage, the library at 83.34% against 80%, the windows' 26.88% written beside it. Met on a reading, pending Pratik's word and reversible in one line: memory with 1,000 cached messages, 57 MB against 150 MB, and idle, 56 MB against 100 MB, both judged on the application process with the WebView2 tree's 333 MB written in the same sentence, because the targets predate the preview being a browser and do not say whether they count it; the sum would miss by 240 MB and 291 MB, ledger 482. Revised, kept open: the 100K+ mailbox lines, half answered by 200,000 synthetic rows and waiting for a live account for the rest, ledger 480; the 95% coverage line, history since 2026-08-29 and still not met. Revised by 08-08 and carried here: criterion 4's whole-tree run, not made, its cost written down. PERF-07's justification on the status page rests on what the runs found and not on the share of tests written after their code.

**Plans**: nine, listed in `phases/08-every-number-the-project-quotes/README.md`, one per wave, planned 2026-09-14 against `main` at `b14d6379`.

- [x] 08-01-PLAN.md, merged at `b63527ab`. `docs/development/measurements.md` exists with twenty rows, every figure taken 2026-09-14 by the command in its row and none copied: 783 guard records by a TOML reader before the plan's own record and 784 after, 12,335 mutants over 247 files by `cargo mutants --list` in three seconds, 7,245 tests the library builds, the full-gate band 275 to 654 seconds from the commit bodies, the per-record rate and the sweep's product, and the two suite figures. A reading refuses a row without its command, date or commit, refuses an absent or empty table and a row written twice, and four companions plant each omission in the real page; the target is in both of `check.sh`'s lists; one guard record, five red. The guard runner prints its two terms after every run, and the rate is 92 seconds a record where this file said 86 that morning, the rebuild up and the run down since 2026-09-10. `.cargo/mutants.toml` reads its settings against 104 seconds for every target and 52 for the library at eight threads, twice each, values untouched. `check.sh --suites-for` prints nothing for the page because the script drops a target already in the whole-tree list on purpose, ledger 442. `WINDOWS.md` 441 to 443.
- [x] 08-02-PLAN.md, merged at `4ce4ad96`. Criterion 3 as four checks in the target 08-01 created, 10 tests to 25, each reading with a companion shown red first. A count, a percentage or a duration on any page under `docs/` less the changelog and `docs/plans/`, in `README.md` or in `CLAUDE.md` sits beside a date and a command or named source or says it is a target; the first run named thirteen figures on three pages, ten in `CLAUDE.md`, and every one was dated rather than re-numbered. The status page and the integration guide quote 7,697 tests over every target, 7,245 in the library and 452 under `tests/`, two new rows taken 2026-09-14 at `a42331bb`, and a reading holds those pages to the measurements page and never to each other. The share of history before red/green is computed by `git rev-list` and printed, 181 of 2043 commits, 8.9%, as of 2026-09-14; the four tree sites that stated it as two absolutes name the check, the two planning records stay as written, and the reading joins wrapped comment lines because `scripts/mutants.sh` broke the sentence between its numbers. Twelve prose figures are held to the constants they restate; the roadmap's attachment line said 10 MB where `attaching::LIMIT_BYTES` is 25 and now says what the code does, and the privacy page's update size is a target until a release exists, ledger 444. Under the source-side break the whole library stayed green, so no unit test pins that constant's value. Six records measured on the target, census 598, 790 by a TOML reader. `WINDOWS.md` 443 to 445.
- [x] 08-03-PLAN.md, merged at `e801a3cf`. PERF-01, PERF-02 and PERF-04 have an instrument and their first numbers. `src/common/started.rs` takes the start instant as the first statement of `main` and words the line `the message list is usable: N rows, M ms after start`, said once per process and never for an empty list; `tests/the_numbers_the_targets_ask_for.rs` builds a profile of exactly 1,000 cached messages of the shape its header defines, starts the release binary against it, reads the line and the working set of the process and its six WebView2 processes, and stops the tree. Taken 2026-09-14 at `9d5f15c5` on the release binary: cold start 476 ms, the median of five, with the first start after the build 520 ms; memory with 1,000 cached messages 390 MB, the application's peak 57 MB plus the tree 333 MB; idle at 120 s 391 MB, the application 56 MB; the empty-profile floor 390 MB, the application 54 MB. The application process alone meets all three targets and the sum with WebView2 misses two; 08-09 judges. The first run found that nothing filled the mail module at startup, so the folder tree came up empty on every profile since 2026-07-26 until a mail check or a module switch; fixed, held by a reading in the target and a record coupling `wx_app.rs` to it. The five definitions are written once in the harness's header and quoted on the page. Version 0.125.0. Five records measured, census 603, 795 by a TOML reader. `WINDOWS.md` 445 to 448.
- [x] 08-04-PLAN.md, merged at `6d08c94e`. PERF-03: sort, filter and scroll timed over 200,000 rows by a harness kept in the tree; the virtual text callback's body pulled into a function over slices with a reading that holds it to naming no database; the generator and the sort moved where tests and the mutation tool reach them. This line was left unticked by 08-04's own metadata commit and ticked by 08-05's.
- [x] 08-05-PLAN.md, merged at `292656d0`. PERF-05: coverage re-measured with the 2026-07-26 command, 83.34% on 2026-09-14 against 60.4%, seven rows on the measurements page from one run. The low areas are not the transport: the three areas the requirement names read 92.06%, 84.55% and 96.75%, above the library, and each is named with its figure; the low area is the wxWidgets windows at 26.88%, reported and not attributed, ledger 453; the requirement's sentence and criterion 3's are ledger 452 for 08-06 and 08-09. No test written.
- [x] 08-06-PLAN.md, merged at `53b9300f`. Corrections by hand, documents and comments only. `CLAUDE.md` prescribes the TOML parser for counting guard records and says what the awk missed, measured 2026-09-14 at `3accd6e1`: level with the parser for every file no inline-table record names, 0 against 1 for `wx_send_later.rs`. The four sweep-cost figures, and a fifth in `scripts/guards.py` the ledger had found, each point at the rate, count and product rows on `docs/development/measurements.md` with the old figure kept as its day's; the gate, suite and mutation-run durations on `CLAUDE.md` and the status page dated and pointed; the three undated advisory acceptances dated from `git log`. `REQUIREMENTS.md`'s PERF evidence lines re-taken at `7da68e78`, `wx_app.rs` line numbers replaced by names, PERF-05's `[S]` line dated and added to and no box ticked; `PROJECT.md` re-taken with the 2026-08-29 figures kept; `STATE.md`'s 720 dated. Criterion 3's transport wording is left for 08-09 under criterion 6. No test, record or setting changed value. Two deviations, ledger 454 and 455; 443 closed.
- [x] 08-07-PLAN.md, criterion 5, merged at `1837f93b`, `bd8c2832` and `a52db2fc`: the runner can be stopped and picked up from its own log and waits for a quiet machine; the sweep ran on GitHub's runners in 41 shards at `df3437a1`, 803 records with a verdict; the 31 found short measured again here and corrected by hand, 24 named too few, 2 named a test that stopped reaching the break, 1 break moved to the line it was about, 1 renamed to what its break guards, 1 retired, 2 left as the runner's blindness; the log in the phase directory; the page's rows beside the prediction.
- [x] 08-08-PLAN.md, PERF-07 and criterion 4: shards on one commit; the rate measured on one shard under both suite shapes; which run to make is Pratik's with the products in front of him; the report read after the last process exits, survivors killed or reasoned. Tasks 3 and 4 merged 2026-09-16 at `e02d2bd4`: both runs read whole, 870 mutants, 56 survivors, 43 killed at `96ade665` with 20 records, 6 equivalent, 7 queued; criterion 4 revised above. Task 1 merged 2026-09-15 at `99682439` and `1401e4d3`: the shards, the merger, four rate shards, the products on the page, 17.5 days for the library and 19.8 for every target, both in place. The checkpoint answered by Pratik the same day, "Yes. Let's do that.": not the whole tree this milestone; the sweep first, then `src/service/protocols/**`, 450 mutants in 18 shards, on GitHub's runners; criterion 4 revised under criterion 6, the text in the summary. Its work merged at `abf3e24c`: `--file`, the dispatchable workflow, CI's checkout depth. Nothing dispatched, nothing started; tasks 3 and 4 wait.
- [x] 08-09-PLAN.md, criterion 6, merged at `72b7bedf`. Every target on the four pages judged on the line that carries it against a row on `docs/development/measurements.md`, dated, old wording kept: cold start met, 476 ms; coverage met by the library at 83.34% with the windows' 26.88% beside it; the two memory targets met by the application process, 57 MB and 56 MB, with the WebView2 tree's 333 MB in the same sentence, on the coordinator's reading pending Pratik's word, ledger 482; the 100K+ lines half answered and open for a live account, ledger 480; the 95% line still history. The status page's reason for mutation testing rests on what the runs found. PERF-01 to PERF-06 ticked clause by clause with the closing plan named; PERF-07 open as revised, the whole-tree run not made and its cost written down. Criteria 1, 2, 3 and 6 closed beside 4 and 5, criterion 3's transport clause revised under 6. Ledger 480 to 482 added, 452 and 453 closed. What is left for a person is in `STATE.md`: the manual accessibility pass, the 44 open issues, the ledger's open entries. Documents only; the whole gate green on the branch, 7,779 tests; `main`'s hook answered `docs_only` on the merge.

### Phase 9: What the first day of testing found

**Goal**: The next build carries the number it will ship as, and the twelve defects the first
day of testing found with a known cause are fixed the way the tester described them, test-first,
so the second day of testing meets a different program.
**Depends on**: Phase 8, all of it merged, which it is. Nothing here depends on the milestone's
other open items.
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, FOUND-07, FOUND-08, FOUND-09, FOUND-10, FOUND-11, FOUND-12
**Success Criteria** (what must be TRUE):

  1. `Cargo.toml` says `1.0.0-alpha.1`, `--version` prints it, `common::version` has a test
     naming the exact steps from `0.125.1` and to `1.0.0`, the installer's four-field version
     for it is held above `0.125.1`'s by a test, the Release workflow can publish the version
     the tree carries without bumping it first, a saved settings file carries the version of
     the build that wrote it (red first against a file stamped `0.7.7`), and `CLAUDE.md`, the
     changelog's opening paragraph, `docs/BETA_RELEASE.md` and the release skill all state the
     same rule for how a version moves inside a prerelease. Whether the first alpha is
     published is a dispatch and is Pratik's. (#46, and the one real item of the withdrawn #66)
  2. A bare or unlisted stored spelling language resolves to this machine's own region within
     its family, the settings screen shows the language that will be used rather than the first
     one Windows lists, and a snippet derived from an HTML-only message is its first words
     through the same reader the reading path uses, with the snippets already stored put right
     once on the first open. (#21, #32)
  3. Undo Send is the first item on the Edit menu with its key kept, and a meeting answer says
     the same countdown any other send says at the moment of pressing, never "has been told"
     while it is held; the stale Alt+E comment names Alt+H. (#44, #56)
  4. "Then by" sits directly after "Default sort order" on the Reading tab, "Cc and Bcc lines"
     is on the Compose tab, and exactly one item in the View menu's Sort submenu is checked
     whichever way the sort was chosen. (#36, #39)
  5. The contact, condition, filter, signature and account editors are scan targets on both
     channels, every checkbox in them is named through `set_accessible_name`, no empty static
     text sits before a control as a spacer anywhere under `src/presentation/`, the MSAA walk
     on each editor reports a name for every checkbox, on this machine or on CI's next run
     (ledger 390 records the walk crashing here, not diagnosed, so this clause stays open until
     one of the two has walked the five editors), and `tests/no_label_is_only_a_space.rs`
     refuses an empty static nothing fills. (#42, #40 point 5)
  6. The event stream during Right and Left on the Settings tab row is captured on this machine
     before anything changes, the second event is stopped at its source with a test holding the
     handler, and an `nvda-tests` case holds the transcript to each tab once, to run at the
     next push. (#33)
  7. One function composes the opened body, the PGP finding, the S/MIME envelope sentence and
     the signature verdict, every surface that shows a message asks it (six: the text reader,
     Shift+Space, the Formatted reader, the conversation window as headings, the whole
     conversation in the text reader, and the preview pane), the preview pane carries the bar,
     the `wired.rs` guard names every surface, and the two changelog entries are corrected by
     dating. (#51)
  8. `*.pst` is in the import picker and a data file's mail and items land here through the
     existing writers, each message down the path a saved `.eml` takes, with a closing sentence
     that counts each kind and says the reader has never met a real file; Save As writes the
     selected message as `.eml`, and an attachment is saved by the reader's own Save Attachment
     command as before, the shortcuts page saying which does which; a folder can be chosen
     through a directory picker on its own File item, with a Thunderbird profile folder said to
     be unrecognised; the four documents describe what is reachable and the guide names the
     three commands. (#53, points 1 to 3 and 7; 4 to 6 are later work)
  9. The time from `Ctrl+,` to the Settings dialog built is on `docs/development/measurements.md`
     with each candidate cost on its own, taken before anything changed, and the after row sits
     beside it with every setting still on the screen. (#34)

Every criterion closes structurally. What only a person can settle is written into each
requirement's last `[S]` line and into the phase README, and none of it is claimed here.

**Plans**: ten, one per wave, listed in `phases/09-what-the-first-day-of-testing-found/README.md`,
planned 2026-09-16 against `main` at `524ff24f`; nine when first written that day, and a tenth
cut out of the import plan after the plan check found it sixteen files wide. An eleventh was
added for #66 the same day and removed the same day when #66 was withdrawn: its one real item,
the settings file's version stamp, is in 09-01. One per wave because every plan writes
`docs/changelog.md` and eight of the ten write `guards/guards.toml` under the same-commit
rules, and a wave is a set of plans sharing no file. Each plan ends with the `gh issue close`
or `gh issue comment` the executor runs after the merge, quoting the merge commit, so the
tracker and the tree agree; closing an issue is not a publish.

- [x] 09-01-PLAN.md: The version is `1.0.0-alpha.1`, the workflow can publish it as it stands, a saved settings file names the build that wrote it, and the rule for moving inside a prerelease is written where the old rule was (#46). Merged 2026-09-16 at `c0606807`; criterion 1 closes structurally, the dispatch untouched
- [x] 09-02-PLAN.md: A bare stored language resolves to this machine's region and the screen shows what will be used; a snippet is words and never a stylesheet, stored ones put right once (#21, #32)
- [x] 09-03-PLAN.md: Undo Send on the Edit menu, and a held meeting answer that says it is held (#44, #56)
- [x] 09-04-PLAN.md: The two sort controls together, a compose setting on the Compose tab, and one sort checked (#36, #39)
- [x] 09-05-PLAN.md: The five editors reached by the scan, every checkbox named, the empty spacers gone, and the label check widened (#42, #40 point 5). Merged 2026-09-16 at `165fd811`; criterion 5 closes structurally for its scan-target, naming, spacer and reading clauses, and its walk clause stays open with FOUND-08's second `[D]` line until CI has walked the five editors, the walk having crashed here on every run (ledger 389, 390)
- [x] 09-06-PLAN.md: The Settings tab row's events captured, the second one stopped, and a real NVDA holding each tab to once (#33). Merged 2026-09-16 at `de58771a`; criterion 6 closes structurally for its capture clause and its handler clause, and its transcript clause stays open with FOUND-09's third `[D]` line until the NVDA workflow has run the case at the next push of `main` (ledger 492); the capture was of the win-event channel NVDA reads, which the plan's UI Automation logger alone could not see (ledger 493)
- [x] 09-07-PLAN.md: One reading path for PGP, the S/MIME envelope and the signature bar, on all six surfaces including the whole-conversation reader and the preview pane (#51). Merged 2026-09-17 at `f990d023`; criterion 7 closes structurally: `application::reading_a_message` composes the four, all six surfaces ask it through the window's one seam, the preview carries the bar, `tests/wired.rs` names six with a companion, and three changelog entries are dated rather than the two the criterion counted; what nobody has heard by ear and no real correspondent's key has met is FOUND-10's `[S]` line (ledger 495); #51 closed with the merge commit
- [x] 09-08-PLAN.md: The Outlook data file reader wired into the picker, and Save As writing the list's message (#53, points 1 and 2). Merged 2026-09-17 at `06fdc9b7`; criterion 8's first two clauses close structurally: `*.pst` is in the picker, a data file's mail goes down the path a saved `.eml` takes and its four other kinds through the existing writers, the closing sentence counts each kind and says no real file has been read, Save As writes the selected message as `.eml`, the reader keeps Save Attachment and the shortcuts page says which; the folder picker, the Thunderbird sentence, the four documents and the guide are 09-10's; that no real `.pst` has been read and no saved file opened elsewhere is FOUND-11's `[S]` line (ledger 499, 502)
- [x] 09-09-PLAN.md: Settings measured, then opened at once (#34). Merged 2026-09-17 at `a8b26596`; criterion 9 closes structurally: the time from `Ctrl+,` to the dialog built is on `docs/development/measurements.md` with each candidate cost on its own, taken at `d169df71` before anything changed (2,206 ms in the release binary, median of five, NVDA running; the three lists a millisecond each), and the after rows sit beside them at `2b697408` (397 ms), with every setting still on the screen and every tab in the row from the start. The cost was not where the issue looked: a spell checker built for one sentence and released, a typeface list resizing itself per name under a shown window, and six pages of controls built before the show; the dialog is frozen while built, the sentence names its source without a checker, and a page after General is built the first time its tab is reached. Whether it feels immediate on the tester's machine is his (ledger 504), and the first visit of Reading now pays that page's build (507)
- [x] 09-10-PLAN.md: A folder picker on its own item, the four documents corrected, the guide, and the phase's closing read (#53, points 3 and 7). Merged 2026-09-17 at `8eba6a38`; criterion 8's last two clauses close structurally: File, Import a Folder of Messages opens a `DirDialog` and hands the folder to the same worker Import Mailbox hands a file to, held by `tests/wired.rs` with a companion and one record; the changelog says what a Thunderbird profile folder becomes and dates the older promise; the four pages carry a dated sentence beside the one #53 named and the guide names the three commands. The closing read is in the summary: FOUND-02 to FOUND-07 and FOUND-10 to FOUND-12 ticked clause by clause, FOUND-08 and FOUND-09 open on their CI clauses, criteria 1 to 4 and 7 to 9 closed, 5 and 6 open as their own text says. #53 commented, points 3 and 7 done, 4 to 6 later work; ledger 509 to 511

**UI hint**: yes
**Scope note**: Planned from GitHub issues rather than from a research document, which is
new for this project; each plan's `<premise_corrections>` carries the command run on
2026-09-16 for every file and line an issue cites, and five premises moved: #21's cause is a
stored bare `en` and not either of the issue's two candidates; #33 and #34 say five settings
pages and there are seven; #42's account manager checkboxes are named already and only their
spacers are the class; #46's list of places naming a version shape was two too long; #51's and
#53's line numbers moved with 08-07 to 08-09. The plan check the same day found five more,
all in the README's table: the search index is `strip_markup`'s second caller; the whole
conversation in the text reader is a sixth reading surface; the `.pst` reader composes each
message through the `.eml` writer; the attachment list is in the reader frame with its own
Save command; and the tester's profile, read through a plain Win32 process, was created by
0.7.7 before the default changed, stored the bare `en`, and now holds his hand-set English
(United States), while bash and PowerShell started from this harness read a stale July copy
of it at the same path, which misled three readings in one day and the withdrawn #66. The
five later groups of Pratik's order are not here and are named in the README.

## Progress

**Execution Order:**
Phases execute in numeric order: 1, 2, 3, 4, 5, 6, 7, 8, 9. Phases 4, 5 and 7 depend on nothing
the earlier phases produce and can be reordered if something makes that useful. Phase 9 was
added on 2026-09-16 after phase 8 closed.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Folders and conversations | 14/14 | Executed, verification human_needed | - |
| 2. Search that says what it covers | 9/9 | Executed, verification human_needed | - |
| 2.1 What phase 1 found on its way past | 9/9 | Executed, verification gaps_found (12/13) | - |
| 3. Mail at scale on the wire | 9/9 | Executed, all merged, verification human_needed | - |
| 4. Writing and reading a message in full | 9/9 | Executed, all merged, verification human_needed | 2026-09-06 |
| 4.1 Mail moves between accounts | 4/4 | Complete, 04.1-04 merged | - |
| 4.2 What was built and never reached | 9/9 | Executed, all merged, verification human_needed | 2026-09-07 |
| 5. The other five modules keep up | 8/8 | In Progress|  |
| 5.1 Notes and contacts reach a server | 6/6 | In Progress|  |
| 5.2 Notes in OneNote | 3/3 | In Progress| Every plan built and merged; the phase's human checkpoint is open |
| 6. How the application speaks | 9/9 | Executed, all merged, verification human_needed | 2026-09-14. 06-09 complete, tasks 2 to 4 on branch `one-window-for-every-due-thing`, merged into `main` at `abaa0667` with the whole gate green on the merge, 7,677 tests, version 0.124.0: one window for every due thing, each row saying its kind, Pratik's three answers of 2026-09-14 applied, sixteen records measured, `WINDOWS.md` 431 to 441, nobody has heard any of it. Every plan of the phase is merged. Criteria 2 and 3 close structurally, criterion 1 has four of five clauses closed structurally and its listening pass open, criterion 4 has the list written and the WebView2 findings attributed; what only a person can settle is in `docs/manual-accessibility-pass.md` and waits for phase 8. The earlier account: 06-08 complete, both tasks on branch `twenty-nine-findings-each-with-a-row`, three RED/GREEN pairs `b6b4046c` to `05c15bf0`, documents `13251ca9` and `5719f589`, merged into `main` at `67437b79` with the whole gate green on the merge, 7,656 tests, version 0.123.1. The checkpoint was discharged by Pratik's push of `main` on 2026-09-14, which ran the Accessibility workflow on its own trigger; no agent dispatched anything. Run 34849526207 found twenty-nine findings on UI Automation across eight of thirty-one windows and twelve unnamed controls on MSAA in two, where its own summary said 26 because the count missed the singular Axe prints. Every finding has a row in `docs/wcag-coverage.md` with window, channel, rule and disposition: nine fixed test-first and waiting for the next run on `main`, the editor page's title, three status lines built empty rather than with a space, and the counting pattern held by a test; four WebView2's own zero-size views, attributed from the artifact's providers and parents rather than the names, `MicrosoftEdge/WebView2Feedback` named and nothing filed; fourteen spinner and list text fields where `set_accessible_name` lands on the arrows and not the field focus reaches, ledgered with two recipes; one empty list cell; one unjudged. The count of five replaced on the status page and corrected by addition in the changelog with why it moved beside the number. `docs/manual-accessibility-pass.md` written, seventy-six items across the six categories, each with its source and its technology, the NVDA suite named, and the first sentence says none of it has happened. The 9/9 counts the summary file, as this row did for 06-09's partial summary; 06-09's tasks 2 to 4 still wait on its checkpoint. A fifth whole-tree guard target, one record measured, census 575, 767 by a TOML reader. `WINDOWS.md` 407 to 430, 429 closed. The earlier account: 06-09 task 1 of 4 complete and the plan stopped at its checkpoint, on branch `a-due-thing-says-what-it-is-first`, red `c189a361`, green `0ca6099d`, records `a993d9d5`, summary `20e3dcc4`, merged into `main` at `6d57a49b` with the whole gate green on the merge, 7,649 tests, still version 0.123.0. `Due` has a `Kind` of its own with a doc comment that is the seam for the mail kind and names four places three kinds are assumed, an opaque `Identity` composed by the feed and parsed by nothing, a sentence that says the kind first for all three kinds with the four reminder tests unedited, `what_is_due` taking the hold as a map and refusing an ended event and sorting earliest first, two pure alert instants, and "in 15 minutes" through the catalogue as a twin of the past reading. Nothing a person can reach changed; no task or event is fed until task 4. Eleven records re-measured where the plan summed sixteen, four corrected from the run; six new records each measured on the whole library, census 574, 766 by a TOML reader. `check.sh all` failed once on ledger 374's race and passed on the retry. The checkpoint's three questions are Pratik's and open, put in the summary with options, costs and a recommendation each; none built. `WINDOWS.md` 402 to 406. The row counts the summary file, as it did for 06-03's and 06-06's partial summaries. The earlier account: 06-07 complete, both tasks on branch `roughly-half-becomes-a-list-of-fifty-five`, documents `4d415de3`, red `0ee95483`, green `4abe217d`, corrections `93f8c865`, summary `a35efbca`, merged into `main` at `984570b1`, still version 0.123.0. Both checkpoint answers are Pratik's of 2026-09-14, both a document and a check, and `REQUIREMENTS.md` corrected in place, recorded and not re-asked. Both counts re-taken from sources with their parts summing: 155 rules in the pinned v2.4.2 list, 61 + 53 + 23 + 9 + 9, citing exactly three WCAG criteria, 1.3.1, 2.1.1 and 4.1.2, and 55 criteria at Level A and AA, 31 + 24, with 4.1.1 carrying no level. `docs/wcag-coverage.md` has fifty-five rows saying what each channel can and cannot say, the six regulatory exclusions attributed to Section 508 and EN 301 549 by name, thirty-one windows scanned and seventeen nested outside, and that no scan has run on either channel. The three criteria are code, a reading holds the page to them both ways and has been shown a planted wrong row each way, the empty cases fail, and the scan's step summary is held to the same three. Five sentences corrected where the plan counted four, each keeping its old wording with the date; the workflow's contrast claim is gone. `check.sh`'s documents-only path runs the reading, because a page edit was the one commit that did not. One guard record, 7,190 passed and exactly two red, census 760. Criterion 3 closes structurally, both clauses, nothing run; criterion 4 closes nothing here. `WINDOWS.md` 395 to 401. The earlier account: 06-06 complete, task 2 on branch `a-pinned-scanner-and-every-window-it-can-reach`, red `741d2b36` and green `fd661401`, merged into `main` at `ca88d833`, still version 0.123.0. Both checkpoint answers are Pratik's of 2026-09-14, pin with the hash and all the windows, recorded and not re-asked. The scanner is Axe.Windows v2.4.2 by tag and by the zip's SHA-256, read from the releases API and hashed two ways in the session that wrote it. Thirty scan targets where there were ten, counted from the tree rather than the plan: 40 dialog windows in 38 builder sites, 9 scanned already, 14 top-level dialogs added, 17 nested and outside by name, plus the bare main window and its five other module panels. Every one opened on a throwaway profile on this machine and none is unreachable. Three defects in the scan itself, measured from the CI log of 2026-09-10 and fixed: a clean window reported as a failed scan because the CLI writes a file only on errors, the MSAA channel walking the frame for every dialog and never a dialog, and a dialog that failed to open scanned as the main window, now an exit code the workflow names. Criteria 3 and 4 close nothing here; this is the ground under 06-07's list. Nothing pushed, so nothing scanned by CI. Two guard records, census 759. `WINDOWS.md` 388 to 394, and 384 fixed. The earlier account: 06-06 task 1 of 2 merged from branch `a-workflow-change-earns-the-checks-that-read-it`, red `05ec26a4` and green `dd4934fe`, merged into `main` at `9f86ba6f`. A commit touching any file under `.github/workflows/` now answers `all` on a branch, so the two tests in `scan_target.rs` that read the accessibility workflow run on the commits that could break them; measured on a staged break with one window taken out of the workflow's list, the gate passed in 64 seconds before the rule and failed in 206 naming the window after. Five shell cases, three red first and named in the commit, two holding the rule to the folder rather than the extension. Criteria 3 and 4 close nothing here. The tree says at least fourteen windows are outside the scan where the plan says nine. `WINDOWS.md` 384 to 387. The earlier account: 06-05 complete, both tasks on branch `said-at-once-and-the-window-a-look-later`, head `106efe6c`, merged into `main` at `7aad8722`, version 0.123.0. A reminder due while somebody is typing is said and sounded at the look that finds it and its window opens at the next look whether or not they have stopped; once open, the tone comes back once a minute until focus reaches the window, ten times at most, stopping for good the first time it does. One rule about when a modal may open, answering which reason, is asked by both the folders question and the reminder look through one shared typing helper; the sentence and the tone are a function without a window; the repeat rule is pure and tested against handed-in instants. The second inherited item from phase 1 closes structurally, with the residual written on it: it still opens over them, a minute later and after telling them, which is the decision. Seventeen guard records re-measured, two written, every one reddening exactly what it names. Nobody has heard any of it. `WINDOWS.md` 379 to 383. The earlier account: 06-09 added 2026-09-14, one due window for reminders, tasks and events, shaped for a mail kind after version 1. 06-04 complete, both tasks merged at `06aa8765`, version 0.122.0. One account can be allowed less than Settings allows for every account, never more, from three boxes on the account edit dialog, each unavailable and saying why where Settings has the answer off. The list that recorded the setting as offered by nothing is empty and the guard that watched it is retired rather than left green over nothing, taken red by hand first; the document guard that forbade describing the control is retired and the testing page describes it. The inherited item from phase 1 closes structurally, read clause by clause. Thirty-three guard records re-measured, every one still reddening exactly what it names. Nobody has heard the boxes. `WINDOWS.md` 375 to 378. The earlier account: 06-03 complete, all four tasks merged at `39417f88`, still version 0.121.0. Every date this program writes follows the computer, month names, day names, order and clock, and "2 days ago" comes out of a translation catalogue, Project Fluent, with real plural rules behind it; criterion 2 closes structurally and FEEDBACK-02 with it, and nothing in it has been heard. Version 2 started here on Pratik's answer of 2026-09-13, with the audit in the plan and the direction in the phase README. Only an English catalogue exists, so nothing a user hears changed. The earlier account: 06-03 tasks 1 and 2 of three merged at `e98514b0`, version 0.121.0, and the plan was partial on purpose: the checkpoint between task 2 and task 3 asked what "2 days ago" should do on a French computer, and task 3 was written against the answer. Month names in a date now come from the machine, through two Win32 mechanisms rather than one, because a month inside a date and a month in a list are different words in Russian, Polish, Czech and Lithuanian and asking the wrong way writes the language incorrectly. The English ordinal went with it. Criterion 2 does not close and FEEDBACK-02 does not close: day names are still English at `occurrences.rs:701`, the date in eight signature sentences is still English at `signed_mail.rs:1373`, and relative wording is the checkpoint. The one clause that does close is the silent English fallback, and it is wider than the criterion asks, because a day no month has is refused by Windows and comes back English even where the language exists. `WINDOWS.md` 360 to 364. A third guard went red that nobody predicted, a record naming code a rename moved, which is a different check from the one that counts tests. 06-02 merged before it, version 0.120.0, and it is the first plan of this phase a person meets. The sixteen per-event answers that have been in the settings file all along are reachable from a screen, and the two global boxes offering speech and braille apart are one control that can mean what it says. Criterion 1 does not close, and it has five clauses rather than the four the row below names: that reading folds "by keyboard" into the first, which is the clause this plan can least attest to. Four close structurally, none is heard, and the rest is 06-06's listening pass. Two clauses were open when 06-02's own tasks were finished and its success criteria claimed both, found by reading the criterion from this file clause by clause and by nothing else, because nothing in the tree reads a criterion. Five guard records rather than the two the plan asked for, because a record guards a rule and the plan counted per task. `WINDOWS.md` 345 to 353. 06-01 merged before it: the model can say what somebody chose separately from what they get, and `Event` with `Event::ALL` come from one list so a seventeenth event cannot exist without a control, a sound scheme slot and test coverage. The row was `0/8` before that, written by 07-06's executor because phase 6's README deliberately left this file alone while phase 7 was editing it |
| 7. Installing, updating and what is stored | 9/9 | In Progress| 07-01 to 07-05 merged; criteria 3 and 6 close, neither heard; criterion 2 has everything but applying, and nothing it can say has met a published release. 07-06 merged one task of three and its summary says `partial`: criterion 5 needs a workflow run nobody has made, so SHIP-05 is still either two CI jobs or a port. 07-07 closes the half of criterion 1 that is about this repository rather than a certificate authority, and rewrites criterion 2 to cover the download. 07-08 merged one task of three and its summary also says `partial`: what has to be signed is now counted from the build rather than remembered, seven things where SHIP-01's wording names two, and nothing is signed because there is no certificate. Criterion 1 waits on an Azure account only Pratik can create, which is 07-08's open checkpoint. 07-09 merged two tasks of three and its summary says `partial` as well: an update is fetched unasked, refused unless this project signed it, and offered once before it runs, and none of it has ever run. The count is of summary files on disk, which is what the check that reads this row counts, so it is not a count of plans that closed what they were written for: three of these nine are partial and say so |
| 8. Every number the project quotes | 9/9 | Complete, every plan merged, criteria 1 to 6 closed or revised in writing | 2026-09-16. 08-09 complete, the last plan of the last phase of the milestone: documents only, on branch `every-target-met-or-revised-with-its-reason` from `main` at `21fd22c3`, `f15f4671` and `d4c155c4`, merged at `72b7bedf` with the whole gate green on the branch on its first run, 7,779 tests, still version 0.125.1. Every target judged against a row, met or revised with the reason on its line; PERF-01 to PERF-06 ticked clause by clause, PERF-07 open as revised; the memory targets on the application-process reading pending Pratik's word, ledger 482; criteria 1, 2, 3 and 6 closed, 3's transport clause revised under 6. `WINDOWS.md` 480 to 482, 452 and 453 closed. The earlier account: 08-08 complete: the two mutation runs Pratik dispatched on GitHub's runners on 2026-09-15 at `3e633252`, `src/service/protocols/**` in 18 shards and `src/service/caldav.rs` in 17, read whole by the merger, 870 mutants, 694 caught, 56 nothing noticed, 116 the compiler rejected, 4 timed out and none never started; the four timeouts re-run here at 594 s and timed out again, each the mutant's own doing; on branch `every-survivor-killed-or-given-its-reason` from `main` at `0fa42bfc`, tests `96ade665`, documents `3ae2b5f1`, a guard's marker `7a7a2d74`, merged alone at `e02d2bd4` with the whole gate green on the branch on its second run, the first refused by `tests/flag_names.rs`, 7,779 tests, and green on the merge, still version 0.125.1. 43 survivors killed by tests each shown red by hand against its mutant, 20 guard records each with the mutant as its break, measured, exactly the test named red; 65 records naming the three files re-measured first, all agreeing; 6 survivors equivalent with the reason and 7 queued as untested behaviour on `docs/plans/20260915-whole-tree-mutation-run.md`. The runner's rate, median 372 and 405 s a mutant against 130 s here, and the two runs' wall clock, runner time and counts are six rows on the measurements page. Criterion 4 revised in place with the real results; PERF-07 is 08-09's. 825 records by the parser, census 802 + 23. `WINDOWS.md` 471 to 479. The earlier account: 08-07 complete: the sweep ran on GitHub's runners, run 34965790937 at `df3437a1`, 41 shards, 4 h 7 min of wall clock, 803 records with a verdict, 772 agreed and 31 not; task 3 on branch `every-record-the-sweep-found-short-corrected`, records `66d8b73d`, documents `5a888378`, merged alone into `main` at `a52db2fc` with the whole gate green on the branch on its first run and on the merge on its second, the first refused by ledger 374's keyring race, 7,758 tests, still version 0.125.0. The 31 measured again here before any edit: 29 short here too, 24 named too few (21 in files the record never named, 3 in named files stamped over by the 2026-09-02 recount), 2 named a test that stopped reaching the break, 1 break moved to the body's line, 1 renamed to the fact its break guards, 1 retired; 2 right here and blind on a runner, left. Every corrected record through `--remeasure` and agreed. Census 802 swept at `df3437a1` and 3 since, 805 records. Four rows on the page: wall clock, counts, direction, the runner's 260 s a record beside this machine's 92. `WINDOWS.md` 467 to 471. Criterion 5 closed on the log's count. The earlier account: 08-07's checkpoint widened by Pratik, "Go for running the guard sweep via CI as well.", and answered on branch `the-sweep-runs-in-shards-on-runners` from `main` at `b611ed82`: red `ef2f5346` and `930cb991`, green `a9220a25` and `9032b9be`, record `6cf50cdb`, merged alone into `main` at `bd8c2832` with the whole gate green on the branch on its first run and on the merge, 7,752 tests, still version 0.125.0. `scripts/guards.py --shard K/N` takes one contiguous block of the file's records by position, refused outside 0..N-1 and written on the run's first line; `.github/workflows/guards.yml`, "Would each guard still go red", `workflow_dispatch` only with inputs `shards`, `first` and `last`, fans the sweep out over Windows runners at `github.sha` with the whole history and keeps every shard's log with `if: always()`; `tests/the_guard_sweep_runs_on_runners.rs`, a target of its own in `check.sh`'s whole-tree list, holds the workflow to the script with a companion planting eleven mistakes, and one record couples it to the workflow. Nothing dispatched; the summary carries the Actions-tab inputs, the download-and-merge lines, and the local start as the fallback. A hard kill during a probe left a real break in `managers.rs` and the resume refusal named it. 803 records, census 192 + 611. `WINDOWS.md` 464 to 465, 463 closed. The earlier account: 08-08's checkpoint answered by Pratik, "Yes. Let's do that.": skip the whole tree this milestone, the guard sweep first, then one scoped run over `src/service/protocols/**`, 450 mutants in 18 shards, the rest recorded with the count and the rate, criterion 4 revised under criterion 6 with the product table as the reason; widened on his question about running the CI that direct merging skips to GitHub's runners. On branch `a-shard-can-be-scoped-to-one-area` from `main` at `0fa393ba`, red `0edfccd9`, green `0e89d7c5`, red `7fc822af`, red `e84efde0`, green `b81d4dd8`, merged alone at `abf3e24c`, the whole gate green on the branch on its first run and on the merge, 7,750 tests, still version 0.125.0: `--file GLOB` on the shard modes, recorded and held by the merger; `mutants.yml` dispatchable from the Actions tab, the diff since a named ref or a range of shards on one runner each at `github.sha` with the whole history, read locally with `gh run download` and `--shards`; `ci.yml`'s Test Suite checkout at `fetch-depth: 0` for the push of `main` at `0fa393ba` that failed the share-of-history test on one commit. Readings for all three red first, 87 worked examples where there were 47, three records measured and one corrected, census 192 + 610, 802 by the parser. Both worktrees at `abf3e24c`. The Actions-tab inputs, the two-dispatch whole tree, the runner rate as a guess, the local fallback and criterion 4's text are in the summary. Nothing dispatched, nothing started; tasks 3 and 4 wait. `WINDOWS.md` 460 to 464. The earlier account: 08-08 task 1 of 4 complete and the plan stopped at its checkpoint, on branch `one-shard-at-a-time-and-the-rate-before-the-run` from `main` at `d53893b7`, red `6dd3e85e`, green `2847391c`, documents `6497d610`, merged alone into `main` at `99682439`, then a second half on `an-in-place-shard-refuses-a-tree-somebody-left-broken`, red `d6ef92e9`, green `9bcad4af`, merged at `1401e4d3`, the whole gate green on both branches on their first runs and on both merges, 7,746 tests, still version 0.125.0. `scripts/mutants.sh` has `--shard k/n`, `--shards n`, `--out` and `--in-place`, each shard to its own directory with its conditions written before and its timing after; the launcher skips complete shards, waits for a quiet machine, skips the baseline after the first complete shard with the config's own timeout, refuses a modified tree in place, and stops on a shard that did not complete; `scripts/mutants_report.py --shards DIR N` merges every shard as one run and refuses a missing, partial, moved or differently committed shard by name, 83 worked examples where there were 47. Four rate shards of 25 in `../wixen-mail-mutants`, all in `accessibility.rs`: the every-target shape cannot run in a scratch copy because the copy has no `.git`; the library at eight threads is 116 s a mutant in a copy and 118 s in place, every target in place 130 s, the fixed term 402 s a shard in a copy and 97 s in place. Products on the page, 12,391 mutants in 496 shards: 17.5 days for the library, 19.8 for every target, both in place. One record measured, census 192 + 607. Both worktrees at `1401e4d3`, built and clean, 08-07's checkpoint hash corrected. The checkpoint recommends option 1 after the sweep; nothing is started, tasks 3 and 4 not attempted. `WINDOWS.md` 457 to 460. The row counts the summary file, as it did for 08-07's partial summary. The earlier account: 08-07 task 1 of 3 complete and the plan stopped at its checkpoint, on branch `a-sweep-that-can-be-stopped-and-picked-up` from `main` at `4bdaa47f`, red `812241ea`, green `bbd1f28d`, merged alone into `main` at `1837f93b` with the whole gate green on the branch on its first run and on the merge, 7,746 tests, still version 0.125.0. `scripts/guards.py` has `--log`, `--resume`, `--stop-after` and `--wait-until-quiet`; the reading of the log is pure with 95 worked examples where there were 57, run by `house_style` on every commit touching the script; one record measured through the flag, resumed to the next, refused over a modified guarded file with the `git checkout` printed, started detached from PowerShell with the window gone, and marked contended by a stand-in `rustc.exe` alive as its run returned and then measured again on resume. The sweep is not started: the checkpoint in the summary names the worktree at `1837f93b`, which was created and given its first build, and quotes the page's product row, 784 x 92 s of 2026-09-14, against 798 records today. No record touched, census 192 + 606. `WINDOWS.md` 455 to 457. The row counts the summary file, as it did for 06-03's, 06-06's and 06-09's partial summaries. The earlier account: 08-06 complete, all three tasks on branch `four-figures-for-one-sweep-become-one-row`, documents `1dce536c`, `7da68e78` and `ee40346c`, merged into `main` at `53b9300f` with the whole gate green on the branch on its first run and on the merge, 7,746 tests, version 0.125.0. `CLAUDE.md` counts guard records with the TOML parser and says what the awk missed; the four sweep figures and a fifth point at the rows with the old figures kept as their day's; the gate, suite and mutation durations dated and pointed; the advisories dated; the three planning documents re-taken at `7da68e78` with their commands, no box ticked. No test, record or setting changed value. `WINDOWS.md` 453 to 455, 443 closed. The earlier account: 08-05 complete, both tasks on branch `coverage-re-measured-and-the-low-areas-named`, documents `59c65c2e` and `f17b5b70`, merged into `main` at `292656d0` with the whole gate green on the branch on its third run, 7,746 tests, version 0.125.0; the first two runs refused by ledger 374's keyring race. Line coverage 83.34% on 2026-09-14 by `cargo llvm-cov --lib --summary-only`, 160,966 of 193,153 lines, against 60.4% on 2026-07-26 by the same command; the transport and the provider clients read 92.06%, 84.55% and 96.75% and are not the low area; the wxWidgets windows at 26.88% hold 73% of the missed lines and are reported and not attributed. Seven rows on the page, the status paragraph rewritten, a changelog entry, no test written. `WINDOWS.md` 451 to 453. The earlier account: 08-04 complete, all three tasks on branch `the-list-paints-from-memory-and-says-how-fast`, red `6a2cd207`, `14e92cdb` and `d2fb848b`, green `ba3b9df0`, `92155231` and `cd0f199b`, records `55b556f1` and `3727bcc5`, the page `1dbca9a5`, merged into `main` at `6d08c94e` with the whole gate green on the branch and on the merge, 7,746 tests, version 0.125.0. The list's row text from `virtual_rows::text_for` over slices, the closure reduced to lock, borrow, call, and a reading with three companions and two records holding both to naming no database; the generator and the sort in modules of their own with the two sort records re-pointed and re-measured; a harness over 200,000 rows with no window and eighteen rows on the page, every value under a second: listing 351 ms cold, filter 78 to 110 ms at the box's limit, sorts 61 to 260 ms, page paint 0.09 ms. Three records, census 606, 798 by a TOML reader. `WINDOWS.md` 448 to 451. The earlier account: 08-03 complete, all three tasks on branch `the-list-says-when-it-became-usable`, red `16dbfbf4`, `584acda1` and `7b49768b`, green `76469bf7`, `844f45d5` and `c2054e61`, records `59237bc2` and `9d5f15c5`, the page `44c444b7`, merged into `main` at `e801a3cf` with the whole gate green on the branch and on the merge, 7,722 tests, version 0.125.0. The usable line, the harness, and the first numbers: cold start 476 ms, memory with 1,000 cached messages 390 MB of which the application is 57 MB, idle 391 MB of which the application is 56 MB, the empty floor 390 MB; the startup fill that was missing since 2026-07-26 fixed. Five records, census 603, 795 by a TOML reader. `WINDOWS.md` 445 to 448. The earlier account: 08-02 complete, all three tasks on branch `every-count-says-when-and-how`, red `a42331bb`, `3231e4b2` and `7845ee17`, green `1df4286f`, `ace8f482` and `32a35c41`, records `957a2d31`, `2233056a` and `f94dacca`, merged into `main` at `4ce4ad96` with the whole gate green on the branch and on the merge, 7,702 tests, version 0.124.0. Four readings and their companions: a figure on any page a person believes carries its date and its source, the three test-count pages quote one row, the share of history before red/green is computed and printed rather than written, and twelve prose figures are held to their constants; thirteen paragraphs dated, one roadmap line corrected, one privacy sentence made a target. Six records, census 598, 790 by a TOML reader. `WINDOWS.md` 443 to 445. The earlier account: 08-01 complete, all three tasks on branch `every-number-beside-its-command`, red `9fa49dba` and `bb61e88e`, green `1022b9d2` and `9399a1e2`, record `0588644e`, documents `d52e9cdd`, merged into `main` at `b63527ab` with the whole gate green on the branch and on the merge, 7,687 tests, version 0.124.0. One page holds every figure the phase is scheduled from, each with its command, date, commit and conditions, and a reading refuses a row without them; the sweep's product is 784 x 92 s, about 20 hours, both terms taken that day; the mutant count is 12,335; the mutation timeouts read against 104 s and 52 s. `WINDOWS.md` 441 to 443 |
| 9. What the first day of testing found | 10/10 | Complete, every plan merged; criteria 1 to 4 and 7 to 9 closed structurally, 5 open on its walk clause and 6 on its transcript clause until a push of `main` runs the Accessibility and NVDA workflows, as each criterion's own text says. 09-10 merged 2026-09-17 at `8eba6a38`: File, Import a Folder of Messages opens a `DirDialog` and hands the folder to the same worker Import Mailbox hands a file to, so the folder branch `mailbox_archive::opened` always had is reached; the two pickers share the readiness check, the refusal and the worker start; `tests/wired.rs` holds the item, its arm, its picker and its hand-over with a companion, one record measured and 17 re-measured, 868 records; the shortcuts page gains the four File rows it never had; the changelog entry says what a Thunderbird profile folder becomes here and dates the older promise; `docs/comparison.md`, `docs/privacy.md`, `.planning/intel/built-and-left.md` and `.planning/codebase/INTEGRATIONS.md` each carry a dated sentence saying the Outlook reader was reachable by nothing until `06fdc9b7` and has met no real file; `docs/USER_GUIDE.md` gains Import and Export naming the three commands; the closing read in the summary ticks FOUND-02 to FOUND-07 and FOUND-10 to FOUND-12 clause by clause and leaves FOUND-08 and FOUND-09 open on their CI clauses; #53 commented, points 3 and 7 done, 4 to 6 later work; ledger 509 to 511. Nine issues closed across the phase (#21, #32, #34, #36, #39, #44, #46, #51, #56) and four advanced (#33, #40, #42, #53); the 09-05 sentence below says #42 closed, and 09-05's summary and the issue say advanced. 09-09 merged 2026-09-17 at `a8b26596`: Settings measured before anything changed, 2,206 ms from `Ctrl+,` to the moment before `show_modal` in the release binary with NVDA running, median of five opens driven by `tests/the_settings_dialog_opens_in.rs` against a throwaway profile, the three lists the issue named a millisecond each and the pages of controls the rest; inside them, by a scratch timing reverted, a Windows spell checker built to word one sentence and released (210 ms) and the typeface list resizing itself after each of 272 names with the window shown (about a second); the dialog is frozen while it is built, the sentence is worded from `spellcheck::source_for_language` with nothing built, and the six pages after General are built the first time their tab is reached on a frozen panel, a page nobody reached written back as stored; 397 ms after, on the same boundary, and the first visit of Reading 156 ms on a hidden frame; eleven rows on the page, the changelog entry, nine records measured and 39 re-measured with one corrected, 867 records; ledger 504 to 508; criterion 9 closed structurally, #34 closed with the merge commit, and whether it feels immediate is the tester's. 09-08 merged 2026-09-17 at `06fdc9b7`: File, Import Mailbox lists `*.pst`, `import_tree::what_was_chosen` answers a third thing from the file's opening bytes, and the worker hands a data file to `application::importing_an_outlook_data_file`, which files each `Mail` item the way a saved `.eml` is filed, under Imported in the file's own folder with the marker, and each appointment, contact, task and note under the local account through the cache's four writers, saying what came across, what stayed behind, about a password, and that no real Outlook data file has been through this program, which is true because neither this program nor the crate can write one and the module's tests hand it one of each kind (ledger 499); Save As writes the message under the cursor as `.eml` through `export_tree::one_message_written_out`, a kept signed original byte for byte, with a name from the subject with the path taken out, and the reader keeps Save Attachment, the shortcuts row saying which; six records, one older record corrected from the remedy, 858 records; ledger 499 to 503 and 97 corrected; #53 commented, points 1 and 2 done, 3 and 7 for 09-10. 09-07 merged 2026-09-17 at `f990d023`: `application::reading_a_message::for_message` offers a message's armour to the key, takes the body to show and asks the S/MIME envelope and the signature once, `ReaderDocument::with_what_is_said` folds the three in the order that keeps each spoken, and all six surfaces that show a message ask it through the window's one seam: the text reader, Shift+Space, the Formatted reader (which took the signature alone until now), both conversation readings through `conversation_parts`, and the preview pane through `the_preview_of`; `ConversationPart` carries what was found, a conversation of several says why a PGP message did not open and what an envelope says under that message's own heading on the page and in the text reader, and `preview_html` renders the top of the bar into the preview's page as a region named "Security warning"; `tests/wired.rs` names all six from a table, with a companion that splices each out; readings against the GnuPG key and message and the OpenSSL signed message; five records measured and five older ones corrected from what the remedies found; criterion 7 closed structurally, three changelog entries dated, #51 closed; what nobody has heard by ear and no real key has met is FOUND-10's `[S]` line (ledger 495), and the key import's visible status line is owed (ledger 496). 09-06 merged 2026-09-16 at `de58771a`: `scripts/uia-events.ps1` logs both accessibility channels while it posts Right and Left to the Settings tab row of a running build on a throwaway profile; on the channel NVDA reads for a native tab control the control's own arrow handler raised `EVENT_OBJECT_FOCUS` twice on the reached tab per key, one millisecond apart, and raises it once now that `wx_settings::answer_the_arrows_on` moves the selection through `SetSelection`, which goes through `TCM_SETCURSEL`; `tests/the_settings_tab_row_says_each_tab_once.rs` sends a real `WM_KEYDOWN` to the built dialog and counts with an in-context win-event hook, red first with the capture's own counts; `nvda-tests/tests/settings-tabs-read-once.test.js` holds each tab to once under a real NVDA and runs at the next push of `main`, not here (ledger 492); criterion 6 closed structurally for its capture and its handler clauses, its transcript clause open until that run; #33 commented and left open. The capture was taken on a locked session with NVDA running, keys posted rather than injected (ledger 493). 09-05 merged 2026-09-16 at `165fd811`: the contact, condition, filter, signature and account editors are scan targets opened on the frame on a fixture, each seen here; every checkbox in them named through `set_accessible_name`, the eleven empty spacers replaced by sizer spacers, `tests/checkbox_labels.rs` reading the built editors and `tests/no_label_is_only_a_space.rs` refusing an empty static nothing fills; criterion 5 closed structurally for four of its five clauses, its walk clause open with FOUND-08's second `[D]` line until CI walks the five, the MSAA walk having left with -1073740791 on every run here (ledger 390, 489); #42 closed and #40 point 5 commented, what NVDA says on the two editors ledgered (490). 09-04 merged 2026-09-16 at `c928cae4`: "Then by" the tab stop after "Default sort order", read back from the built dialog; "Cc and Bcc lines" on the Compose tab in a Writing section, every setting still offered by a screen; the Sort submenu one radio group with no separator, held by a chain reading and by a real menu bar, which showed the old shape with four ticks from the moment it was built; criterion 4 closed structurally, #36 and #39 closed, the look at the running program and the listening pass ledgered (487, 488). 09-03 merged 2026-09-16 at `f58b9271`: Undo Send first on the Edit menu with its key, held by a new source-reading target; a meeting answer says the composer's own sentence from the composer's own value, `HowItWent::Sent` retired, "has been told" gone, a queued answer filed while held and flushed when it goes now, the Alt+E comment corrected; criterion 3 closed structurally, #44 and #56 closed, the calendar entry Undo Send leaves ledgered (486), 155's listening question open. 09-02 merged 2026-09-16 at `a3554483`: one resolver for a stored spelling language, asked by the checker and the settings screen, the screen showing the language that is used; a snippet and its index row through the reader the message goes through, the crude stripper gone, stored snippets put right once on open; criterion 2 closed structurally, #21 and #32 closed, the tester's own profile ledgered (484, 485). 09-01 merged 2026-09-16 at `c0606807`: the tree at `1.0.0-alpha.1`, the `as-is` level, the settings stamp, four pages with one rule; criterion 1 closed structurally and the first alpha still a dispatch. Planned 2026-09-16 against `main` at `524ff24f`, corrected the same day after the plan check (three blockers, twelve warnings, one plan split in two); an eleventh plan for #66 was added and removed the same day when #66 was withdrawn | 2026-09-17 |

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
- PIM-04's blocker is narrower than this said. "A sync target chosen" was
  answered on 2026-08-29, and that answer is what split the requirement into
  PIM-04, PIM-07 and PIM-08; `.planning/REQUIREMENTS.md` carries it. What is
  still open is only which backend goes first, costed in
  `.planning/phases/05-the-other-five-modules-keep-up/05-RESEARCH.md`.

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

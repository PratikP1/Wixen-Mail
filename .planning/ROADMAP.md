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

**Plans**: 12/13 plans executed, one per wave. Two shared files, `guards/guards.toml` and `docs/changelog.md`, are touched by most plans under the same-commit rules, and `src/presentation/wx_app.rs` by most, so the plans are ordered rather than run in parallel.

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

**UI hint**: yes
**Scope note**: These criteria were rewritten 2026-08-29 after the phase discussion. The original five described nesting a flat tree. What the discussion decided is in `.planning/phases/01-folders-and-conversations/01-CONTEXT.md`, which is the authority on the detail; these criteria are what the phase is verified against. The phase also needs three IMAP verbs that do not exist in `src/service/protocols/imap.rs` today: CREATE, RENAME and DELETE mailbox.

### Phase 2: Search that says what it covers

**Goal**: A search returns what the user asked for, and says plainly what it could not reach.
**Depends on**: Phase 1
**Requirements**: SEARCH-01, SEARCH-02, SEARCH-03
**Success Criteria** (what must be TRUE):

  1. A search saved with Subject Only or From Only reruns with that restriction, not across subject, sender and recipients. The live search already honours all four scopes; only the saved one loses half of what it was given.
  2. Opening a saved search shows the scope it holds, so a short list reads as a narrow scope rather than as an empty mailbox.
  3. A search whose terms need body text says, before it runs, how much of the mailbox has body text stored, and offers to fetch the rest.
  4. A user defines a smart folder from the same rule vocabulary the filters use, and it appears in the folder tree listing what matches now rather than a snapshot.

**Plans**: TBD
**UI hint**: yes

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

**Plans**: TBD

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
  5. Each target is either met or revised with the reason written down.

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1, 2, 3, 4, 5, 6, 7, 8. Phases 4, 5 and 7 depend on nothing
the earlier phases produce and can be reordered if something makes that useful.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Folders and conversations | 13/13 | Complete | 2026-08-31 |
| 2. Search that says what it covers | 0/TBD | Not started | - |
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

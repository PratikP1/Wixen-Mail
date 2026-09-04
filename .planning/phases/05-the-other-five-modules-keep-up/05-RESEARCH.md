# Phase 5: The other five modules keep up - Research

**Researched:** 2026-09-04, from a read of the tree at commit `d3c6c7d`
**Domain:** In-repo only. No external package is added by this phase and none was looked up.
**Confidence:** HIGH on everything below, with each claim carrying the file and the line it came
from. Nothing here came from a web search. No CONTEXT.md exists for this phase, so nothing is
locked yet and the decisions at the end are all open.

This is research rather than a discussion. It answers what is true in the tree today.

## Summary

**Four of the eight requirements describe the code wrongly, and three of those four say something
is missing when it ships and is reached from a menu.** That is the same failure phase 3 found, in
larger numbers, and it is the finding this pass exists to produce.

The single most important correction: **PIM-01 and PIM-02 are largely built.** Move between
containers exists, works for events, tasks and notes, and is reached from the Action menu at
`Ctrl+Shift+V` and from the `Applications` key context menu in every one of those three modules.
`move_item` is at `src/presentation/managers.rs:6257`, dispatched at `:2665`, raised from
`src/presentation/wx_app.rs:3530-3531`, and the menu item that carries the key is at
`wx_app.rs:6126-6130`. `docs/IMPLEMENTATION_STATUS.md:87-88` and `docs/ALPHA_TESTING.md:116-117`
both still say "moving and copying work for mail only", and both are wrong; `docs/changelog.md:2152`
records the change that made them wrong.

The second correction: **PIM-03's recurrence expansion is built and reached from the running
program.** `occurrences::falls_on` (`src/application/occurrences.rs:62`) expands a rule into every
day it falls on and removes the days EXDATE calls off (`:98`). `CalendarEventItem::every_day_shown`
(`src/presentation/ui_types.rs:1063`) turns that into one list row per day, and it is called from
the module loader at `wx_app.rs:10861` and from the calendar reload at `managers.rs:1187`. A weekly
meeting already appears on every week in the event list.

What is genuinely absent is narrower and clearer than the requirements suggest: **copy in the PIM
modules, week and month views, a notes backend of any kind, and CardDAV.** Two of those four cannot
be closed in this phase because their last mile is a server nothing here has ever met.

**Primary recommendation:** plan PIM-01, PIM-02 and PIM-03 as *finishing and correcting* work
against a shipped feature, not as new construction, and correct the three stale evidence blocks in
`.planning/REQUIREMENTS.md` before any of it is planned. Plan PIM-06 and PIM-07 as the two real
builds. Treat PIM-05 as a build whose transport can never be closed here, and say so in the
criterion the way phase 3 said it.

---

## The three buckets

Sorted strictly, as asked. "Reached" means a menu item, a key, or a button in the running program
leads to it; the trail is given each time, and where the trail stops is named.

### Bucket 1 — exists and is reached from a non-test path

| What | Where | The trail |
|---|---|---|
| Move an event, task or note to another container | `managers.rs:6257` `move_item` | Action menu "Mo&ve to...\tCtrl+Shift+V" (`wx_app.rs:6126-6130`) and context menu `Action::MoveItem` (`context_menu.rs:364`, `:376`, `:383`) → `ID_MOVE_TO_FOLDER` / `ID_CONTEXT_MOVE_ITEM` → `wx_app.rs:3530-3531` → `managers::pim_command` (`:2538`) → `:2665` |
| The destination chooser, with the current container left out | `managers.rs:6346` `where_it_could_go`, `destinations::offer` | called by `move_item` before any window opens |
| Refusing a move of a provider-held item, before the chooser | `managers.rs:6464` `moving_can_be_told`, worded at `pim_command.rs:cannot_be_moved` | `where_it_could_go` asks it first (`managers.rs:6377`) |
| Refusing a move *into* a read-only calendar | `managers.rs:6509` `file_under` → `can_only_be_read` (`:6593`) | the chooser filters these out at `managers.rs:6388`, and `file_under` asks again for any route that bypassed the chooser |
| Marking a moved item as waiting to be sent | `managers.rs:6546` (`event.pending = true`), `:6561` (`task.pending = true`) | so `pending_calendar_events` / `pending_tasks` see it and the next sync pushes it |
| Recurrence expansion into one row per occurrence | `occurrences.rs:62` `falls_on` | `wx_app.rs:10861` and `managers.rs:1187` → `ui_types.rs:1063` `every_day_shown` → `UIUpdate::CalendarEventsLoaded` |
| EXDATE, so a cancelled occurrence is not shown | `occurrences.rs:98` `days_called_off` | inside `falls_on`, same trail |
| A note body read back as Markdown structure | `long_text.rs` (module), `spoken` | `read_aloud.rs:351` `impl ReadAloud for NoteItem::read_full` |
| Note editing and saving | `wx_app.rs:2286` `notes_cp.btn_save.on_click`, writing at `:2327` | the Save Note button (`wx_notes_module.rs:97-100`) |
| vCard 3.0 read and write | `contacts.rs:312` `import_contacts_from_vcard`, `:728` `export_contacts_to_vcard` | Import/Export contacts menu |
| The four-armed contacts conflict model | `contacts_sync.rs:988` `whose_copy_wins` | two production call sites (`:2289`, `:2485`), reached from `wx_app.rs:18959`, `:18986` |
| CalDAV discovery, listing, and event CRUD | `caldav.rs:228`, `:277`, `:386`, `:442`, `:486` | Add Calendar dialog (`wx_add_calendar.rs:52`) |

### Bucket 2 — exists but only tests reach it

| What | Where | Why nothing reaches it |
|---|---|---|
| `ContainerKind::ContactGroup` as a move destination | `managers.rs:6650` `containers_in`, `:6661` `groups_in` | `ItemKind::Contact.kept_in()` returns `None` (`new_item.rs:61`), so the chooser is never opened for a contact and the group branch of `containers_in` is unreachable from a move. It is reached by the group-picker for other commands, so it is not dead; it is unreached *as a move destination*. |
| `AddressBook::Other("carddav")` | `contacts_sync.rs:4951-4954`, `contacts.rs:3997`, `:4009` | The only four occurrences of the word "carddav" in `src/` are in `#[cfg(test)]` blocks. The enum admits it; nothing writes it. |
| The `notes.format` column | written as the literal `"plain"` at `managers.rs:3107`, `:5090`, `notes.rs:312`, `:351`, `:389`, `:421`, `:450`; read at `outlook_data_file.rs:3260` only | Written with a constant everywhere and read by no production code. Meanwhile the editor labels the box "Body, in Markdown" (`wx_notes_module.rs:83`) and the accessible description says Markdown is read back (`:94`), and `read_aloud.rs:351` does parse it as Markdown regardless of the column. The column is a stored answer nothing asks. |

### Bucket 3 — does not exist

| What | Verified by |
|---|---|
| **Copy** for any PIM item | `ID_COPY_TO_FOLDER` has one handler, `wx_app.rs:3816`, and it calls `move_or_copy_message` (`:16146`), which is mail. The non-mail fall-through at `:3531` names `ID_MOVE_TO_FOLDER` only. `PimCommand` (`pim_command.rs:20-33`) has four variants and none is Copy. |
| **Move for contacts and reminders** | `PimCommand::Move.applies_to` (`pim_command.rs:54`) matches `Event \| Task \| Note` only, and `context_menu.rs:426-437` is a test holding the menus to exactly that. |
| **Week and month calendar views** | `wx_calendar_module.rs:46-58`: the comment says they are not built, `btn_prev.enable(false)` and `btn_next.enable(false)` at `:55-56`, accessible names "Previous period, not built yet" and "Next period, not built yet" at `:57-58`. The window is fixed: `the_window_now` (`ui_types.rs:1091`) is today − 180 to today + 365 (`occurrences.rs:33` and `:35`), and nothing moves it. |
| **A moved occurrence (RECURRENCE-ID) shown on its real date** | `provider_recurrence_id` is a column (`mod.rs:1029`, `:2014`, `:2493`) and `calendar.rs:1912-1917` reads it to decide whether a row shares its series' address. `falls_on` filters by EXDATE only (`occurrences.rs:98-104`); nothing folds an override row's own date into the series' expansion. |
| **Any notes backend** | No `notes_sync.rs`. `src/application/*sync*.rs` is `caldav_sync`, `collection_sync`, `contacts_sync`, `mail_sync`, `pop_sync`, `tasks_sync`, `sync_marker`. `notes` and `note_folders` tables (`mod.rs:2028-2056`) carry no `pending`, no provider id and no version marker. `NoteEntry` (`mod.rs:1131-1141`) has nine fields and none of them is a sync field. |
| **VJOURNAL in CalDAV, OneNote in Graph** | Neither string occurs anywhere in `src/`. |
| **CardDAV** | No PROPFIND for `addressbook-home-set`, no `discover_address_books`, no address-book URL anywhere. `AskWith` (`outward.rs:90-93`) has two verbs, Propfind and Report, both used by CalDAV. |

---

## Requirement by requirement: is the stated evidence still true?

### PIM-01 — Move a task from one list to another. **Evidence is wrong.**

The requirement says "no move-between-lists path in `src/application/tasks_sync.rs`". Literally true
and misleading: the path is not in `tasks_sync.rs` and never would be. It is in
`managers.rs:6257`, and a task made on this computer moves between lists today, from a menu, with a
key.

The `[S]` line says it is "recorded as not built in `docs/IMPLEMENTATION_STATUS.md` and
`docs/ALPHA_TESTING.md`". Both documents are stale. `docs/changelog.md:2152-2157` records "Move
follows the module you are in", and `:5263-5277` records the follow-up that made the move mark the
item as waiting to be sent.

**What is actually left for PIM-01, in order of size.**

1. **A provider-held task cannot be moved at all, and this is a designed refusal, not a gap.**
   `moving_can_be_told` (`managers.rs:6464`) asks `tasks_sync::a_provider_holds` (`tasks_sync.rs:433`,
   which is `!Provider::made_here(task_id)`) and refuses with the sentence at
   `pim_command::cannot_be_moved`. That sentence says why: neither Google nor Microsoft is asked to
   move a task between lists, and doing it means delete-there, create-here, and writing the new
   identity over the old. The changelog states the same as a known limitation
   (`docs/changelog.md:5272-5277`). So the requirement's real content is *make the provider move
   work*, and neither the requirement nor the roadmap says so.
2. **The criterion "a move that fails at the provider leaves the task in exactly one list" is about
   a path that does not exist.** Today a provider move is refused before anything is written, so
   the criterion is vacuously satisfied and would stop being satisfied the moment the work is done.
   It is the delete-then-create sequence that can leave a task in two lists or none, and that
   sequence is the whole risk in this requirement.
3. **The `Allowed::personal_information` criterion is not true as written.** `grep allowed
   src/presentation/managers.rs` returns nothing. The gate is applied where the HTTP client is
   built — `tasks_api.rs:835`, `google_api.rs:545`, `microsoft_graph.rs:489`, `caldav.rs:203` — so a
   move is written locally and marked pending whatever the gate says, and the gate bites at the
   push. Nobody is "refused with a reason" at move time. That is either a defect to fix or a
   criterion to reword, and it is a decision because the current behaviour is arguably right: a
   local file is not a change at a provider.

### PIM-02 — Move and copy in the modules that are not mail. **Evidence is half right, and the requirement contradicts a design decision in the code.**

The evidence about mail is right: `imap.rs:1280` `copy_message` and `:1308` `move_message`. The
sentence "The inventory records move and copy as missing for everything else" is half wrong: move
ships for three of the five modules.

**Copy is the real work.** Nothing in `PimCommand` copies. A copy is also not the same operation
as a move here: `file_under` (`managers.rs:6509`) does read-change-write on the same row, and a copy
needs a new id, a new `pending`, and a decision about what happens to a copied provider item.

**The requirement asks for contacts and reminders and the code argues they should not have it.**
`pim_command.rs:50-56` gives the reason inline: a contact is in as many groups as somebody puts it
in, so there is no one home to move it out of; a reminder is filed nowhere, because the module sorts
by when each is due. `new_item.rs:52-62` says the same in `kept_in`. `context_menu.rs:426-437` is a
test that holds the five context menus to exactly `PimCommand::Move.applies_to`, so widening the
requirement to contacts and reminders makes that test red and requires a new answer to "move it out
of what, into what". **This is a decision for Pratik, not a plan detail.** For contacts there is a
coherent answer (group membership, which `groups_in` at `managers.rs:6661` already enumerates); for
reminders there is no container in the schema at all (`mod.rs:1916-1928` — a reminder has an
account, a due time and an optional `related_event_id`, and nothing that holds it).

The criterion "the same two keyboard commands in every module" is achievable for move today
(`Ctrl+Shift+V` is already module-following) and for copy would mean routing `Ctrl+Shift+Y`
(`wx_app.rs:5830-5833`) the same way `Ctrl+Shift+V` is routed at `:3531`.

### PIM-06 — Week and month calendar views. **Evidence is correct.** The only one that is.

`wx_calendar_module.rs:46-58` says exactly what the requirement quotes, at the lines it quotes.
`docs/changelog.md:1424` matches.

Three things the requirement does not say that a plan needs.

- **The event list is not "loaded by account" any more, it is loaded by a fixed window.**
  `events_that_could_fall_between` (`calendar.rs:308` in the data layer) takes a from and a to, and
  `the_window_now` (`ui_types.rs:1091`) supplies today − 180 to today + 365. The load path at
  `wx_app.rs:10834-10861` already passes a range, so a week or month view is a *narrower and movable*
  window rather than a new query. This makes PIM-06 considerably smaller than the requirement's
  phrasing implies.
- **The expansion is already per-day.** `every_day_shown` sorts by moment (`ui_types.rs:1078`)
  and each row carries the stored event's identity so opening any occurrence finds its event. A week
  view is a filter and a heading over rows that already exist.
- **The accessibility criterion is the hard half and cannot be closed here.** "A screen reader can
  work through a view's events in date order without reconstructing the grid" needs a real NVDA run.
  The requirement already says so, and it is right to.

### PIM-03 — Recurring events across date ranges. **Evidence understates what ships.**

The requirement says `occurrences.rs` and `repeating.rs` "hold the recurrence model" and that the
display is PIM-06's work. Both true. What it omits is that the expansion is already wired into the
list the user actually sees, via `every_day_shown`, from two production call sites. So the first
`[D]` — "a recurring event appears on every date it occurs in whichever view is showing" — is
already true of the only view that exists.

**What is left is the second `[D]`: a moved occurrence.** EXDATE cancellation works
(`occurrences.rs:98`, with a Google end-to-end test at `calendar.rs:4891-4903`). An override — a
single occurrence moved to another date — is stored as its own row carrying
`provider_recurrence_id` (`mod.rs:1029`), and `falls_on` knows nothing about it. So today a moved
occurrence appears twice: once expanded from the series on its original date, and once as its own
row on the new one. **I could not verify this by running it, only by reading: nothing subtracts a
`provider_recurrence_id` row's original date from its series' expansion.** That should be the first
failing test of any PIM-03 plan, because if I am wrong about it the requirement is nearly closed.

The two "stated limitations" the requirement names are real and unchanged: `can_be_honoured`
(`calendar.rs:1946`) refuses a one-day change where only half of it could be sent, and the doc
comment above it explains that Google and Outlook are never told how an event repeats.

### PIM-04 / PIM-07 / PIM-08 — Notes. **Evidence is correct, and the roadmap's blocker is stale.**

`.planning/ROADMAP.md:413` lists "PIM-04 needs a sync target chosen before anything can be built" as
a blocker known at roadmap time. **`.planning/REQUIREMENTS.md` already carries the answer**, dated
and attributed: *"Decided 2026-08-29 by Pratik. Not one target. A note has a backend chosen by the
account it belongs to, the local note itself is a first-class Markdown document, and the seam is
shaped so a hosted service can be added later without a migration."* That is the decision, it is
made, and it is what split one requirement into PIM-04, PIM-07 and PIM-08. **The roadmap line should
be struck rather than treated as a gate.** (See "Decisions" below: what is still open is not
*whether* to choose, it is *which backend goes first*.)

**PIM-04 is close to done and the requirement does not know it.** Of its six `[D]` items:

- "A note is a Markdown document, reusing what signatures use" — the reuse already happened.
  `long_text.rs` is the shared reader, `pulldown-cmark` is already imported there (`long_text.rs:17`),
  and the notes editor labels itself Markdown (`wx_notes_module.rs:83-94`).
- "The stored form is the Markdown source" — true. `long_text.rs:13-15` states it as the module's
  rule, and `save_note` stores the body verbatim (`notes.rs:99`).
- "A screen reader reads the rendered structure" — `read_aloud.rs:351` does exactly what
  `read_aloud.rs:332` does for a contact's notes, which is the precedent the requirement names.
- The remaining three are all about sync: the `Allowed` gate, the settings screen saying notes do not
  sync yet, and the seam. Those are PIM-07's.

**The one loose end inside PIM-04 is `notes.format`.** It is written `"plain"` at six sites and read
by nothing, while the editor and the reader both treat the body as Markdown. Either the column
becomes meaningful, or it goes, or the phase records why it stays; leaving it is the shape of
observation 0017 in this project's own log, a column that reads as a working feature and is not one.

**PIM-07 is the phase's largest genuine build.** Its evidence — no VJOURNAL in `caldav.rs`, no
OneNote in `microsoft_graph.rs` — I re-verified: neither string occurs anywhere in `src/`. What the
requirement does not carry, and a planner needs:

- **The schema work is real.** `notes` and `note_folders` (`mod.rs:2028-2056`) have no `pending`, no
  provider id and no version marker. Every other synced kind has all three, and the project's rule is
  additive columns via `ensure_column_exists` (`CLAUDE.md`, and see `mod.rs:2483-2510` for the
  calendar precedent).
- **`new_item.rs` is where the "backend chosen by account type" decision already lives, in prose.**
  Lines 16-33 explain why notes stay local per provider, and `:256-260` says "Reminders stay false
  everywhere and always will". That module's `syncs` predicate is the natural seam and it currently
  returns `false` for `Note`.
- **A green test asserts notes have no sync, and the phase turns it red.**
  `context_menu.rs:611-620` `test_notes_are_not_offered_a_sync_they_cannot_do` asserts the note-folder
  context menu does not contain `Action::SyncNow`. It is correct today. It must be inverted in the
  same commit that adds a backend, and no requirement names it. There is a second of the same shape:
  `context_menu.rs:426-437` holds the menus to `PimCommand::Move.applies_to`, so PIM-02's widening
  reddens it too.

**PIM-08 is the one requirement with no code to check**, which its own evidence line admits. The
honest research finding is that the existing seam it should imitate is `AddressBook`
(`mod.rs:423-446`): three variants where the third is `Other(String)`, with a doc comment saying it
exists so a word this code does not recognise survives being read and written back. That is a
working, shipped example of "the seam does not forbid a later implementation" and it is the model to
copy rather than invent.

### PIM-05 — CardDAV. **Evidence is right about the gap and wrong about where the vCard code lives.**

Right: `caldav.rs` covers calendars only, CardDAV is not built, `docs/development/requirements-backlog.md`
says so.

Wrong in a way that matters to a plan: the requirement says the vCard reader and writer are "built
in `src/application/contacts_sync.rs` and `importing_contacts.rs`". They are not.
`importing_contacts.rs` is 1 public function (`:32`, `what_the_card_import_did`) and builds a
sentence. **The reader and writer are in the data layer: `import_contacts_from_vcard`
(`src/data/message_cache/contacts.rs:312`) and `export_contacts_to_vcard` (`:728`).** A plan told to
reuse them at the named files would find a wording helper.

What CardDAV actually costs, from reading the CalDAV it must parallel:

- **The HTTP verbs are already there.** `AskWith::{Propfind, Report}` (`outward.rs:90-93`) with the
  method parsed at `:111-115`. CardDAV needs the same two.
- **Discovery is the same shape.** `discover_calendars` (`caldav.rs:228-273`) is a PROPFIND with a
  fixed body and a parser (`parse_propfind_calendars`). CardDAV's is the same request against
  `addressbook-home-set` with a different namespace, then `addressbook-query` REPORT instead of
  `calendar-query`.
- **The conflict half already exists on the contacts side**, and it is where PIM-05 collides with
  phase 3 — see the next section.
- **`AddressBook::Other("carddav")` already round-trips**, so no schema change is needed to name the
  new address book. The version marker column (`ProviderIdentity::provider_version`, `mod.rs:451-462`)
  already exists per address book, which is exactly where a CardDAV ETag belongs.
- **Case-folding is guarded and the guard covers this file already.**
  `caldav.rs:4702-4711` `FILES_THAT_READ_OR_WRITE_A_DOCUMENT` names eight files including
  `src/data/message_cache/contacts.rs`, and `test_nothing_that_reads_or_writes_a_calendar_document_matches_a_name_by_case`
  enforces case-insensitive matching of `VCARD`, `FN`, `EMAIL` and eighteen other names. A new
  CardDAV file must be added to that array (`caldav.rs:4702`) or it is unguarded, and the array's own comment says a
  name left off it is a name the reading has stopped checking.

---

## Where phase 5 overlaps plan 03-09, precisely

`03-09-PLAN.md` is scheduled in phase 3, wave 9, and it builds:

- `src/application/conflict_choice.rs` — a held conflict, the two versions, and the decision.
- The losing arm of `whose_copy_wins` (`contacts_sync.rs:988`) holding the conflict instead of
  writing over and apologising afterwards.
- `caldav_sync.rs` raising the same question on an ETag disagreement, rather than resolving silently.
- The keyboard-only choosing surface in `wx_app.rs`.

**PIM-05 lands directly on two of those four.** A CardDAV address book is a third source of
contacts flowing into the same `whose_copy_wins`, and its ETag is the same kind of marker
`caldav_sync` uses. So:

1. **PIM-05 must not build its own conflict model.** `03-09`'s own premise correction 2 says this in
   as many words: "Do not write a second conflict model." A CardDAV sync that resolves ETags by
   itself would be exactly the duplicate `03-RESEARCH.md` warned about, and the two would disagree
   the first time either changed.
2. **PIM-05 should depend on 03-09, or be planned to plug into `conflict_choice.rs` as a consumer.**
   If phase 5 runs before phase 3 wave 9 — the roadmap says phase 5 "can run alongside Phases 1 to 4",
   which does not include phase 3's last wave — then CardDAV has no conflict surface to plug into
   and will grow one. That ordering question is a decision, not a detail.
3. **The overlap is only on contacts and CalDAV.** Nothing else in phase 5 touches it: notes have no
   backend to conflict with, tasks resolve through `resolution_for` (`tasks_sync.rs:1195`), and moves
   are local writes.

**One more collision, on cost rather than design.** `03-09` measured the guard-record counts and
made module placement decisions from them. Re-measured today against `guards/guards.toml`:

| File phase 5 would touch | Guard records naming it |
|---|---|
| `src/application/contacts_sync.rs` | 135 |
| `src/application/calendar.rs` | 121 |
| `src/presentation/managers.rs` | 68 |
| `src/presentation/wx_app.rs` | 61 |
| `src/service/caldav.rs` | 54 |
| `src/application/caldav_sync.rs` | 33 |
| `src/application/tasks_sync.rs` | 32 |
| `src/application/pim_command.rs` | 6 |
| `src/application/occurrences.rs` | 6 |
| `src/application/destinations.rs` | 2 |
| `src/application/new_item.rs` | 2 |
| `src/presentation/ui_types.rs` | 0 |
| `src/data/message_cache/notes.rs` | 0 |
| `src/presentation/wx_calendar_module.rs`, `wx_notes_module.rs` | 0 |

`calendar.rs` at 121 is second only to `contacts_sync.rs`, and PIM-03 and PIM-06 both point at it.
Adding test functions to it flags 121 records for the count check, which is a build and a full
library run each. **The same rule `03-09` adopted applies here: put new types and new decisions in
new modules and add as few test functions as the work allows to `calendar.rs`, `contacts_sync.rs`,
`managers.rs` and `wx_app.rs`.** `ui_types.rs` and `notes.rs` are free, which is convenient because
`ui_types.rs` is where a week or month window would live.

---

## "The two that currently go nowhere" — which two, and what going somewhere means

The goal (`ROADMAP.md:315`) says two of the five modules go nowhere. **The code names the same two,
and it argues that one of them should stay that way.**

`new_item.rs:16-33` is the authority, and it is explicit:

- **Notes.** Could sync on Microsoft through OneNote and is not written; a OneNote page is an HTML
  document inside a section inside a notebook rather than a title and a body, so the mapping is a
  decision rather than an afternoon's work. Google Keep's API is Workspace-only, so a consumer Gmail
  account cannot use it at all.
- **Reminders.** `new_item.rs:256-260`: *"Reminders stay false everywhere and always will. In
  Outlook and Exchange a reminder is a property of an event or a task rather than an item, and Google
  folded Reminders into Tasks in 2023, so there is nothing on either side to sync a standalone
  reminder to."*

So the two are notes and reminders. **PIM-04, PIM-07 and PIM-08 give notes somewhere to go.
Nothing in phase 5's requirements gives reminders anywhere to go, and the code says there is
nowhere.** Either the goal means notes plus contacts-over-CardDAV (which reads as a stretch, since
contacts already sync to Google and Microsoft), or the goal is one module wider than its
requirements and reminders is the module with no requirement.

**Going somewhere, for notes, has a concrete shape and three candidate first backends.** None is
verified against a server, and each carries a cost this project has already written down:

| Candidate | What exists here to build on | What it costs |
|---|---|---|
| **CalDAV VJOURNAL** | The whole CalDAV transport: PROPFIND, REPORT, ETag, `If-Match` (`caldav_sync.rs:870`, `:1028`, `:3424`, `:3716`), and the iCalendar reader/writer shape. A VJOURNAL is a title and a body, which is what `NoteEntry` is. | Only reaches accounts that have a CalDAV server. A Gmail or Outlook account gets nothing, so the "backend chosen by account type" seam is exercised with one arm filled and two empty. |
| **Microsoft OneNote via Graph** | `microsoft_graph.rs` and its OAuth. | `new_item.rs:19-23` already states the objection: a page is HTML inside a section inside a notebook, so the mapping is lossy in both directions and it is a decision, not a translation. PIM-04 requires the stored form be Markdown source and round-trip byte-identical; HTML round-tripping through Markdown does not do that. |
| **Google Keep** | Nothing. | `new_item.rs:22-24`: Workspace-only, so a consumer Gmail account cannot use it at all. This is the one that should not be first. |

CalDAV VJOURNAL is the cheapest by a wide margin and the only one whose round-trip can honour PIM-04's
byte-identical criterion. **It is still Pratik's call**, because the seam it exercises is the one
PIM-08 is judged on, and a seam proven with one arm is a weaker proof than one proven with two.

---

## What cannot be closed in this phase

Nothing here has ever run against a real account, a real CalDAV server or a real CardDAV server.
Naming the last miles precisely, rather than glossing them.

1. **Every CardDAV network call in PIM-05.** Discovery, the `addressbook-query` REPORT, the PUT with
   `If-Match`, and whether a server's ETag survives a round trip. Only the XML and vCard parsing can
   close here, and the requirement's last `[D]` already says so. Follow phase 3's precedent and let
   this become an `unrun-verify` ledger entry rather than a criterion that reads as met.
2. **Whether a provider accepts a task move done as delete-then-create** (PIM-01). The failure mode
   the criterion is about — a task in two lists or none — occurs only when the second call fails
   after the first succeeded, which is exactly what cannot be produced without a provider. The
   *local* half is testable: that the cache never holds the task under two list ids, and that a
   half-finished move is recoverable.
3. **Whether a note round-trips byte-identically through any backend** (PIM-04, PIM-07). Testable
   against a fake `NoteService` the way `TaskService` (`tasks_sync.rs:446`) is faked; not testable
   against a server.
4. **Every accessibility criterion in PIM-06 and PIM-02.** Whether a week view reads in date order,
   whether a month grid is navigable without reconstructing it, and whether move and copy announce
   distinguishably. `wx_calendar_module.rs` and `wx_add_calendar.rs:27-29` both already record that
   the screen reader pass has not happened.
5. **Whether the CalDAV VJOURNAL mapping is accepted by any real server**, if that is the backend
   chosen.

---

## Assumptions phase 5 would rest on that I could not verify, and what each costs if wrong

| # | Assumption | How I would check it | Cost if wrong |
|---|---|---|---|
| A1 | A moved occurrence (`provider_recurrence_id` row) is currently shown twice — once from the series' expansion, once as itself. I read every line of `falls_on` and found only EXDATE filtering, but I did not run it. | One test: store a series and an override row, call `every_day_shown` over the range, count rows on the original date. | If it is already handled somewhere I did not find, PIM-03 is nearly closed and a plan would build a second de-duplication beside a working one. This is the exact shape of the mistake phase 3 avoided. Cheap to check; check it first. |
| A2 | `Ctrl+Shift+V` really reaches the handler in the non-mail modules, rather than only being in the menu label. `tests/wired.rs:1959-1973` asserts the dispatch source contains the arms, and the menu carries the accelerator, but `wired.rs:19-24` says in its own words that a bound key proves dispatch and says nothing about what the handler then does with the right thing on screen. | Press it in a running build with a task selected. | If the key does not reach it, PIM-01 and PIM-02 are much larger than this document says, and the whole "already built" finding shrinks to "built and unreached", which is bucket 2, not bucket 1. |
| A3 | A CardDAV server's ETag can be carried in `ProviderIdentity::provider_version` without a schema change. The column is `Option<String>` and per-address-book, which fits. | Read `contacts.rs`'s identity read/write once more, against a written-out CardDAV flow. | A schema change mid-plan, which is additive here so it is a cost rather than a hazard. |
| A4 | `outward.rs`'s HTTP client can issue PUT and DELETE with `If-Match`, not just PROPFIND and REPORT. CalDAV's `create_event`/`update_event`/`delete_event` (`caldav.rs:386`, `:442`, `:486`) must be doing this, but I read their signatures rather than their bodies. | Read `caldav.rs:386-530`. | Nothing structural; it would mean widening `AskWith`, whose doc comment (`outward.rs:96-103`) already explains how to add a verb safely. |
| A5 | Notes have no second writer. I found `save_note` called from `managers.rs:3101`, `wx_app.rs:2327`, and `move_item`'s `file_under`. | `grep -rn save_note src/`. | A backend that pushes on `pending` would miss a writer that never sets it, which is the shape of the "moved but nothing was sent" bug the changelog records at `:5263`. |
| A6 | `docs/IMPLEMENTATION_STATUS.md` and `docs/ALPHA_TESTING.md` are the only two places carrying the stale "move works for mail only" claim. | The grep observation 0026 in the project's own log prescribes: "not built", "not supported", "cannot", "not yet". | A user-facing document telling somebody a shipped feature does not exist. Cheap to sweep, and this phase should sweep it whatever else it does. |

---

## Validation architecture

`.planning/config.json` sets no `workflow.nyquist_validation` key, so it is enabled.
`workflow.tdd_mode` is `true`, so every eligible task is `type: tdd` with RED and GREEN gate commits.

| Property | Value |
|---|---|
| Framework | `cargo test`, in-tree `#[cfg(test)] mod tests` plus 24 files under `tests/` |
| Config file | none; `Cargo.toml` |
| Quick run | `cargo test --lib <module path>::` — the scoped form `scripts/which-checks.sh` already uses |
| Full suite | `bash scripts/check.sh` (fmt, clippy `-D warnings`, tests, release build) |
| Scale | roughly 5,200 test functions across `src/` and `tests/` |

| Req | Behaviour | Type | Command | Exists? |
|---|---|---|---|---|
| PIM-01 | A provider task move leaves it in exactly one list | unit, faked `TaskService` | `cargo test --lib application::tasks_sync::` | ❌ Wave 0 |
| PIM-01 | The refusal wording and the pre-chooser refusal | unit | `cargo test --lib application::pim_command::` | ✅ partly (`pim_command.rs` tests) |
| PIM-02 | Copy leaves the original and move does not | unit | `cargo test --lib presentation::managers::` | ❌ Wave 0 |
| PIM-02 | Menu and command agree about which kinds accept copy | unit | `cargo test --lib application::context_menu::` | ✅ exists for Move (`:426`), needs a Copy twin |
| PIM-03 | An override occurrence appears once, on its real date | unit | `cargo test --lib application::occurrences::` | ❌ Wave 0 |
| PIM-06 | Prev/Next move the window by one period and announce the range | unit on the window arithmetic | `cargo test --lib presentation::ui_types::` | ❌ Wave 0 (`the_window_around` at `:1102` is already clock-free and is the hook) |
| PIM-06 | Screen reader order in a week view | manual | NVDA pass | manual-only |
| PIM-04 | A note's Markdown round-trips unchanged | unit | `cargo test --lib application::long_text::` | ✅ partly (`:774`, `:866`) |
| PIM-07 | An unsendable note stays local, is marked waiting, and the summary says why | unit, faked backend | `cargo test --lib application::notes_sync::` | ❌ Wave 0 |
| PIM-07 | The note-folder menu offers sync exactly where a backend exists | unit | `cargo test --lib application::context_menu::` | ⚠️ exists and asserts the opposite (`:610`) — must invert |
| PIM-08 | The seam takes a hosted backend with no stored-form change | unit, second fake | `cargo test --lib application::notes_sync::` | ❌ Wave 0 |
| PIM-05 | vCard and PROPFIND parsing, pure | unit | `cargo test --lib service::carddav::` | ❌ Wave 0 |
| PIM-05 | The new file is covered by the case-folding guard | unit | `cargo test --lib service::caldav::tests::test_nothing_that_reads_or_writes_a_calendar_document_matches_a_name_by_case` | ✅ exists; the new file must be added to `caldav.rs:4702` |

**Wave 0 gaps.** A faked `NoteService` in the shape of `TaskService` (`tasks_sync.rs:446`); the
override-occurrence fixture for `occurrences.rs`; a clock-free window fixture for week and month.
No framework install is needed.

**Guard sweep.** `CLAUDE.md` puts guard re-measurement off the critical path as of 2026-09-03: the
executor does not run guards and neither does the merge. What is not optional is running
`scripts/guards.sh --remeasure "..."` when a commit prints it. Given the counts above, expect that
to fire often on `calendar.rs` and `managers.rs`.

---

## Security domain

`security_enforcement` is not set false anywhere in `.planning/config.json`, so it applies.

| ASVS category | Applies | Standard control here |
|---|---|---|
| V2 Authentication | yes, PIM-05 only | CardDAV basic auth via `keyring`, following `caldav::credentials` (`caldav.rs:109-160`) — one keyring service name per owner, per `CLAUDE.md` |
| V3 Session Management | no | no sessions |
| V4 Access Control | yes | `Allowed::personal_information` gates every provider write; see the PIM-01 finding that it does not gate the local move |
| V5 Input Validation | yes | a CardDAV server's vCard and XML are untrusted. `contacts.rs`'s existing reader already treats a card file that way (`:1190`, `:1991`) |
| V6 Cryptography | no | nothing new; TLS via the existing client |

| Pattern | STRIDE | Mitigation already in the tree |
|---|---|---|
| XML external entity in a PROPFIND response | Tampering / Info disclosure | `parse_propfind_calendars` is a hand-written scan rather than a general XML parser; a CardDAV parser must be written the same way and not reach for one that resolves entities |
| A malicious vCard joining two people | Tampering | already a named known limitation (`docs/ALPHA_TESTING.md:118-122`); CardDAV makes it reachable from the network rather than from a chosen file, which raises its severity |
| Credentials in the cache database | Info disclosure | `CLAUDE.md`: nothing sensitive in `message_cache.db`; CardDAV sign-ins go to `keyring` like CalDAV's |
| A note body from a backend rendered as HTML | XSS | `long_text.rs:270` already sanitizes with `ammonia` on the way out and its test at `:740` holds it |

---

## Decisions for Pratik

These change what gets built and are not mine to settle.

1. **Does PIM-02 really mean contacts and reminders?** The code refuses move for both, with reasons
   written into `pim_command.rs:50-56` and `new_item.rs:52-62`, and a test holds the menus to that.
   For contacts there is a plausible answer (move between groups). For reminders there is no
   container in the schema at all. Widening the requirement means overturning a recorded decision;
   narrowing it means correcting the requirement text.

2. **Is a provider task move in scope for PIM-01, or is the refusal the answer?** Today it is
   refused with a sentence that explains why. Making it work means delete-there, create-here, and
   writing the new identity over the old, and the "exactly one list" criterion is entirely about
   that sequence failing halfway. It is the largest single risk in this phase and the only one that
   can lose somebody's data.

3. **Which notes backend goes first, and does one arm prove PIM-08?** CalDAV VJOURNAL is far the
   cheapest and the only one whose round-trip can be byte-identical. OneNote is a mapping decision
   the code has already argued against. Keep is Workspace-only. But PIM-08 asks the seam to be ready
   for a second implementation, and a seam proven with one arm is a weaker proof.

4. **Does phase 5's PIM-05 wait for phase 3's plan 03-09?** The roadmap says phase 5 can run
   alongside phases 1 to 4, which does not cover 03-09 in phase 3's last wave. A CardDAV sync
   planned before `conflict_choice.rs` exists will grow its own conflict handling, which is the
   duplicate 03-09 was written to prevent.

5. **Does the `Allowed::personal_information` criterion in PIM-01 describe a defect or a
   mis-worded criterion?** A move is a local write today and the gate bites at the push. Arguably
   correct; the requirement says otherwise.

6. **Reminders.** The goal says two modules go nowhere and only one has requirements. Either
   reminders are out of scope and the goal should say so, or there is work here that nothing has
   written down.

---

*Research written 2026-09-04, from a read of the tree at commit `d3c6c7d`. Nothing was edited,
nothing was committed.*

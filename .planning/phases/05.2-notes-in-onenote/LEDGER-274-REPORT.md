# Ledger 274: one backend container is one note folder

Branch `one-backend-container-is-one-note-folder`, fifteen commits, six
red/green pairs. `scripts/check.sh all` passed on the branch before the merge,
all four checks, redirected to a file rather than piped. Merged at `eb80066`,
where it passed again. Version `0.110.0`.

Closes 274, and 245 and 246 with it.

## What it does now

Your notes go beside your calendars, on the same server and under the same
sign-in. Each calendar they go to is now its own note folder here, named after
that calendar. Two calendar servers give two folders and both sync. A note
arriving from a server goes into the folder that server's collection is. A note
you move between two of those folders is moved at the server too: created in the
new place first, and only once that has worked removed from the old one.

Folders you make yourself sit under "On this computer", the words the mail
folder list already uses. Nothing is sent for them, because making a folder here
does not make one at the server. Moving a note into one moves it here and
changes nothing at the server.

None of it has met a real account. Seven ledger entries say which parts, one per
unknown.

## What the brief said to verify, and what it found

Every claim in the brief's `<what_you_are_changing>` held, with two additions
and one thing the brief did not anticipate.

| Claim | Verdict |
|---|---|
| `note_folders` is `id, account_id, name, display_order, created_at, UNIQUE(account_id, name)` at `mod.rs:2362` | True |
| `notes` carries `provider_note_id` and `provider_version` and no container | True. They are added by `ensure_column_exists`, not in the `CREATE TABLE` |
| A note's container is its folder's, so `notes` may need nothing | True. Nothing was added to `notes` |
| `sync_the_notes_of` around line 268 is the caller that must loop | True, at 267 |
| `the_calendar_server_of` is just above it | True, at 246 |
| `notes_sync.rs:751` files arrivals into `ensure_default_note_folder` | True |
| The comments at line 35 and line 751 say so and become false | True. Both rewritten |
| `ContainerKind::NoteFolder` is the manager, the move and the delete | True, though `ContainerKind` lives in `application::new_item` rather than in `managers.rs` |
| `NoteFolderItem` at `ui_types.rs:1235` | True, at 1234 |
| `pim_command.rs` and `file_under` are where a note move is decided | Half true. `file_under` decides it; `pim_command` supplies the sentences said afterwards. The write to copy is `move_a_task_the_provider_holds` in `data/message_cache/tasks.rs`, reached through `move_what_a_provider_holds` in `managers.rs` |

**The brief's state-of-the-tree figures were all right**: `main` at `d05404a`,
version `0.109.0`, 711 guard records, census 192 and 519, ledger at 274.

### The thing nothing anticipated: the sync had to change

The contract says "the seam does not change" and "the caller in `notes_backend`
walks an account's folders instead of finding its first calendar, and that is
the whole change on this side." The seam really did not change: `NotesService`
is untouched, `sync_notes` still takes one container per call, and nothing
learns what is inside a container string.

But the three passes *inside* `sync_notes` all worked from the account, and
looping the caller over two containers without changing them misfiles notes.
Every pending note on the account was offered to whichever container ran first,
which creates it in the wrong place and marks it sent, so the container it
belongs to is never offered it. For a OneNote account that is somebody's note in
the wrong section and never in the right one. The same for removals, which would
be sent to a container that never held the note.

So `sync_notes` now asks which folder the container it was given is, by
comparing containers for equality, and its three passes are about that folder.
Equality is not parsing, so the opaque-container requirement holds. This is
recorded in the seam contract under "Built 2026-09-11" rather than left as a
difference between the document and the code.

## How the name clash was decided

`note_folders` is unique on the account and the name, and that constraint
shipped, so it is worked with rather than dropped. Two OneNote sections with one
name sit at different paths and arrive with different names already. Two
calendars on one account really can share a display name.

**The second folder is numbered: `Work`, then `Work (2)`, then `Work (3)`,
unbounded.** The loop is bounded in practice because it asks a finite list of
folders, so there is no ceiling to fall off and no case past a ceiling to
invent an answer for.

**The numbering is cosmetic, which is what makes it safe.** A folder is found by
its container and never by its name, so a clash changes what somebody reads and
never where a note goes. `a_note_folder_for` matches on the container, which is
also why a section renamed at the backend keeps its folder and its notes: the
folder follows the name and never the other way round.

The alternative, refusing the second folder, would drop a whole calendar's notes
on the floor with nothing said, which is the silent data loss the brief asked
about.

A folder somebody made here is never adopted by a backend that wants its name.
`test_a_container_does_not_take_over_a_folder_somebody_made_here` holds it to
that, because adopting one would start syncing a folder nobody asked to sync.

## What a screen reader makes of the separator

**This work chooses no separator, and saying otherwise would be inventing a
measurement.**

A folder is called whatever the backend calls the place. For a calendar server,
which is the only backend that exists, that is the calendar's own display name
with no path in it at all. The ` / ` in `Work / Projects / Q3` arrives the day a
backend with levels above a note flattens its own path into its own name, which
is requirement 2 of the seam's container section and the backend's business. It
is ledger entry 281.

What this work does choose is the numbering: `Work (2)`. At a screen reader's
default punctuation level brackets are not spoken, so it should read "Work 2".
**That is an expectation and not a measurement**, and it is ledger entry 279.
Nobody has heard any of it.

The one thing this does keep out of earshot deliberately: `NoteFolderItem`
carries a bool saying whether the folder is on this computer, not the container.
The container is an address today, and a display type holding one is how a URL
comes to be read aloud.

## Was the task move copyable as it stands

**The decision was. The write was not, and was not generalised either.**

The order is one decision with one home. `move_a_task_the_provider_holds`'s doc
comment carries the whole argument, and `move_a_note_the_backend_holds` says so
and points at it rather than re-arguing it.

The write could not be shared. Different tables, different columns, and the task
one repoints subtasks while a note has none. Generalising SQL across two tables
that differ in every column would have been worse than a sibling. So there are
two functions and one decision, with the note one naming the task one as the
decision's home.

**One thing I got wrong by designing instead of copying, and the tests caught
it.** I first branched the note arm on whether the destination had a container,
so that a move into a folder made here would "send nothing". The task arm does
not branch on the destination, and mine should not have. Both ways of treating a
local destination specially lose the note:

- Leaving the row as it was keeps the name the backend gave it, so the next sync
  of the old container writes the backend's copy over a note somebody had moved
  onto this computer.
- Clearing the name instead leaves the backend's copy named by nothing, to
  arrive again as a new note.

What really happens is the task's third outcome exactly. The copy here is this
computer's own note, the backend is owed the removal of the one it has, and that
removal waits for a copy sitting in a folder no push reaches, so it never goes.
Nothing reaches the backend, and the record is also what stops the note arriving
again, because the read consults every deletion record whether it has been sent
or not. That is ledger entry 280: the backend keeps its copy for ever and
nothing in the program tells the person.

## Which of 245, 246 and 274 are closed

All three, and 245 is the one to check.

**245 is closed by removal, not by documentation.** `the_calendar_server_of` is
gone. `the_calendar_servers_of` answers with every calendar server the account
has, and each is a note folder of its own, so there is no first to pick and
nothing decides on the person's behalf. The brief asked whether the function
goes or stops being the thing that decides: **it goes.**

**246 is closed by the sync knowing which folder it is about.** An arrival is
filed in the folder its container is. A folder somebody made here has no
container, is never synced, and says so under mail's own words.

**274 is closed by all of it being built**, including the four-row table on the
alpha page and the version bump.

## What the guard sweep found

Nine records were re-measured. **Three were not what they said, and two of those
were already stale on `main` before this branch existed.** Neither was caused by
this work, and both were found only because five tests arriving in
`notes_sync.rs` made the count check print the remedy.

**"a note that moved in two places is held rather than written over" reddened
nothing.** Its break gutted the push's `ItMovedFirst` arm, and the read's own
hold, written by a record added the same day, catches exactly the same case and
holds the two copies anyway. Both named tests stayed green. Gutting both arms at
once reddens both, which is what proves the second was covering for the first.
The break is now one the read cannot cover: the arm takes the backend's word and
clears the waiting flag the read's hold is conditioned on. Neither the arm nor
the read's condition was touched by this work. This is the second record in this
file to go quiet that way; the reminder-menu record tells the first.

**"the sync hands a container back exactly as it was given" had a red list one
short from the day it was written.**
`test_a_page_the_service_only_touched_is_not_written_down_again` asserts
`unchanged` is 1, and under that break the read never happens, so it was always
going to redden and was never named.

**"a change nobody has sent is not written over by the read" quoted a call site
this work changed**, which is the ordinary kind of staleness and was corrected
with the code.

Two new records were written for `tests/a_note_moves_between_folders.rs` and for
the note assertion in `tests/a_copy_leaves_the_original_where_it_was.rs`. One
break, two records, because a record runs one cargo target. Census is 192 plus
521, which sums to 713 records.

## Tests that were never red, and why each is still here

Four, each named in its commit message and commented where it sits.

- `test_a_folder_made_here_has_no_container_and_says_so` and
  `test_an_ordinary_deletion_waits_for_nothing`. Absence is what a column reads
  as before it exists, so both passed against the failing half of their own
  pair. They are guards: each goes red the day something starts writing a value
  where there should be none.
- `test_the_order_within_each_branch_is_the_order_it_was_given`. The stub
  returned everything in the order given, so it could not fail. It is a guard
  against a later sorting.
- Three of the five in `tests/a_note_moves_between_folders.rs` assert outcomes
  that already worked and are guards rather than drivers.

One fixture was wrong when it was written and was corrected against the store
rather than against my assumption: the folder order in
`test_every_calendar_server_on_an_account_is_a_note_folder_of_its_own` is the
store's, `display_order` then name, and I had written the order the fixture
stored them in. The claim the test is named for was red either way.

`presentation::note_folder_tree` was written whole before I noticed it had never
been red. The bodies were backed out to stubs, the red commit made, and the
bodies restored. Nothing was committed in between.

## One test changed meaning, and it is worth naming

`test_a_note_moved_to_another_folder_is_left_waiting_to_be_sent` in
`tests/a_copy_leaves_the_original_where_it_was.rs` asserted that a moved note
keeps the name the backend gave it. That was right when an account had one
container and a folder meant nothing at the other end. It is wrong now: keeping
the name would ask the backend to change its copy in the container the note is
leaving, which is somebody's note lost at their server. The assertion is
inverted and the comment says why.

## Commits

```
44512c9 test  a note folder that is one backend container
5b87360 feat  a note folder is one backend container
ca043a4 test  a note moving between two backed folders
c46461b feat  a note the backend holds moves by copy then removal
7736995 test  a sync that knows which folder it is about
83f717e feat  the sync is about one folder, which is one container
8b85a0d test  an account that syncs every container it has
db5d364 feat  an account syncs every container it has, not the first
20fbbd5 test  a note move that knows where it is going
976b78e feat  a note move asks the folder, and copies before it removes
2e652d0 test  a notes tree that tells the two kinds apart
0c87dc5 feat  a folder that goes nowhere is shown as one
7e40e91 docs  the version, the changelog, and the notes move on the alpha page
367d14e test  two guard records for the note move, and the census
```

## Ledger

274, 245 and 246 closed. Seven new entries, 275 to 281, one per unrun or
unmeasured thing:

| Entry | Kind | What is not known |
|---|---|---|
| 275 | unrun-verify | Whether create-then-remove at a real calendar server leaves the note in one place |
| 276 | unrun-verify | Whether a removal held back for its copy goes out on the next sync at a real server |
| 277 | unrun-verify | Whether two calendar servers on one account both sync, and what one refusing does to the other |
| 278 | unrun-verify | Whether the notes tree, and its "On this computer" branch, are clear with a screen reader |
| 279 | unrun-verify | Whether `Work (2)` reads as "Work 2" aloud |
| 280 | todo | A note moved onto this computer leaves the backend holding its copy for ever, and nothing says so |
| 281 | todo | The separator in a flattened path name is unchosen and unheard |

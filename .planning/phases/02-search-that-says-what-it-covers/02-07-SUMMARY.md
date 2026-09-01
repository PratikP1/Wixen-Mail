---
phase: 02-search-that-says-what-it-covers
plan: 07
subsystem: search
tags: [saved-searches, smart-folders, context-menu, accessibility, guards]
status: complete
requires:
  - "wx_managers::build_rule_edit_dialog and show_rule_edit, the condition dialog 02-06 built and left unreached"
  - "MessageCache::replace_saved_search, the one-transaction writer 02-06 built and left uncalled"
  - "wx_managers::run_manager_loop and make_shell, the shared manager shape"
  - "filters::the_words_for_a_field and the_words_for_a_way_of_matching, the 02-04 vocabulary"
  - "wx_app::the_chosen_saved_search and ChosenSearch, the two shapes a tree row resolves to"
  - "application::saved_searches::SAVED_BY_ANOTHER_VERSION, the refusal a newer version's search already gets"
provides:
  - "wx_managers::show_rule_manager_dialog, the second door D-2-01 describes"
  - "wx_managers::populate_questions and what_a_condition_row_says"
  - "wx_managers::a_condition_in_words"
  - "wx_managers::what_a_condition_list_still_needs, one wording read by the window and by the write path"
  - "wx_managers::nothing_stops_this_closing"
  - "manager_words::CONDITION and the tally on the end of every change to a condition list"
  - "make_shell taking the noun its list is called"
  - "context_menu::Focus::SavedSearch and its four entries"
  - "context_menu::Action::EditSearchConditions, RenameSavedSearch, DeleteSavedSearch"
  - "wx_app::ID_EDIT_SEARCH_CONDITIONS and edit_the_chosen_searchs_conditions"
  - "wx_app::the_conditions_to_edit and the_search_to_write_back, the two decisions as pure functions"
  - "wire_context_menu deciding its focus when the key is pressed"
  - "six one-group properties over folder_tree::rows, and one guard record"
affects:
  - "the filter, tag and signature managers, whose lists are now named for what they hold"
  - "every manager window's Close button, which now asks what the list still needs"
  - "the contact and account managers, whose delete sentences take a count they do not say"
  - "guard records 540-area: two re-measured, one added"
tech-stack:
  added: []
  patterns:
    - "a per-row focus for a tree that holds two kinds of row, decided when the key is pressed"
    - "a struct destructured with no `..` as a compile-time guard against a field being added"
    - "one refusal wording read by the window that shows it and by the path that would otherwise write"
    - "a RED half made of wrong reachable code, because a lint denying dead code refuses right unreachable code"
decisions:
  - "The count goes on the end of every change to a condition list, not only on adding and removing. Editing does not move the count, and saying it anyway is what makes the tally something to rely on rather than something to interpret; a number said after only some of the three is a number somebody has to notice the absence of."
  - "Only a condition list counts out loud, decided from the kind rather than passed in as a parameter, so it cannot be switched on in one window over conditions and off in another. The other five have no floor, so a tally there would be a clause with no answer in it."
  - "A saved search's row reports its own focus on the menu key, rather than the entry being filtered out of the folder list. Nothing in the codebase decided menu entries per row, and a whole focus is what the menu bar already models with its Saved Searches menu."
  - "A search a newer version wrote keeps the menu entry and is refused with a sentence, rather than the entry being left out. Enter on the same row already refuses in the same words, so the row behaves the same way whichever way somebody opens it, and context_menu is an application module that cannot see whether a search is readable."
  - "Case sensitivity goes on the end of the third column and only when it is on. Two conditions differing only in it would otherwise be two rows nobody could tell apart, and a column carrying 'no' on almost every row is the other way to make a list unreadable."
  - "Add and Edit collapse into one sub-dialog call on run_manager_loop. Every window passed the same function twice with only the row differing, and it brought the argument count back under clippy's limit without an allow."
  - "Nothing recording which door made a search is checked by destructuring both shapes with no `..`, not by grepping for a name. A text search for `made_by` is answered by whatever the field would really be called."
  - "recursion_limit raised rather than menu_ids! split into two calls. The numbering is deliberately not something a person does by hand."
metrics:
  duration: one session
  completed: 2026-09-01
actuals:
  tokens: 34000
  tasks: 3
  commits: 5
---

# Phase 2 Plan 7: Reached from the tree, written through the replace Summary

**It works, and somebody can reach it.** A saved search's conditions open from
its own row in the folder tree and from the Saved Searches menu, and the whole
list is written back in one transaction on the way out. That is what plan 02-06
built and could not open, and both of its stub entries in `.planning/WINDOWS.md`
are now closed.

What has not happened is a run. The path is traced to a live command and every
part of it is tested, but no window has been shown in a running build, so
nothing has been heard. That is recorded rather than claimed.

`bash scripts/check.sh all` passes: formatting, clippy at `-D warnings`, 5,958
library tests and every integration target, and the release build.

## What works, and how that was checked

**A manager over one saved search's conditions.** A native list in report mode,
one row per condition, three columns: what it looks at, how it compares, and
what it compares against. Add, Edit and Delete are the same three buttons every
other manager window has, and the condition dialog 02-06 built is what Add and
Edit open.

Every part of a row is in words, from the same two builders both rule dialogs
offer their lists from, so a row and the list somebody chose it from cannot come
to say different things. Those lists are already pinned to the engine's
constants in both directions and by count, which makes this transitive. All
eleven fields and all eleven ways of matching are checked, both arms, because
this reads a stored string and `CLAUDE.md` records what mutation testing found
the last time a family here went untested.

A stored name this build has no words for is shown as it is stored rather than
blanked, so a condition written by a later version is still a row somebody can
see and remove.

**Case sensitivity is on the row.** Not in the plan, and added because two
conditions differing only in it were otherwise two rows nobody could tell apart,
which is the fault this codebase keeps finding in lists. It goes on the end of
the third part and only when it is on.

**Every change says how many conditions are left, in one sentence.** Adding,
editing and removing alike. The count goes on the end of the sentence about the
change rather than into a second announcement, because two announcements for one
change put the second over the first before anybody has heard it. Nought is said
in words: "No conditions now", not "0 conditions now", because the next thing
that happens is a refusal to close.

Only a condition list counts out loud, and that is decided from the kind rather
than passed in, so it cannot be switched on in one window over conditions and off
in another. The other five manager windows keep the sentences they have, which a
test asserts by comparing them character for character.

**Closing with nothing left is refused, twice.** The Close button asks what the
list still needs and stays open with a message box when the answer is a sentence.
The close box and Escape do not pass the Close button, so the write path asks the
same function and refuses there too, reading the window's own wording rather than
writing a second one. The store refuses it a third time, which 02-06 built. Three
refusals, one sentence, asserted equal in a test.

**Every manager list is named for what it holds.** It was `"Items"` in all of
them, which is the generic word rather than an answer: somebody landing on the
list heard the same thing whether it held filters, tags, signatures or a saved
search's conditions. `make_shell` now takes the noun and `"Items"` appears
nowhere but in the doc comment explaining why it went.

**Reached from the tree, and from the menu bar.** The saved-search row reports
its own place on the menu key now. The folder tree was reporting one focus for
every row in it, so a saved search was offered a folder's commands: fetching
older messages, for a thing with no server behind it, and choosing folders to
keep up to date, which a search does not have. It now offers editing its
conditions, running it again, renaming it and deleting it, and a test walks every
focus to assert the conditions entry appears on that one and nowhere else.

**Written once, on the way out, under the same identifier.** Everything but the
questions is carried over from the search as it was stored, so a window that was
never asked about the name, the join, the folder or the identifier cannot change
any of them. Closing without changing anything writes nothing at all. After a
write the tree is read back and the search is run again if it is the one showing,
because a change that is stored and not shown reads as a change that did not
happen.

**A search a newer version wrote is refused rather than guessed at.** Its
questions are words this build does not understand; showing them would mean
guessing and writing the answer back would drop whatever it could not read, which
is the half a question list this phase exists to prevent. The refusal uses
`SAVED_BY_ANOTHER_VERSION`, the wording the same row already gets when Enter is
pressed on it, so the row behaves the same way whichever way somebody opens it.

**One group in the tree, however a search was made.** Six properties over
`rows`, all green on arrival, which is the point: there is one group today and
nothing to split it on. What makes them worth writing is that the next person to
add a door will be tempted to add a group beside it, and D-2-02 records why that
breaks. Nothing recording which door made a search is checked by shape rather
than by name: two patterns name every field of the tree row and of the stored
search and carry no `..`, so a field added to carry it stops the file compiling.

## Reached by a person, and how

Guardrail 1 says a feature is done when a non-test path reaches it, so this was
traced rather than assumed. Two routes, both ending in the same handler.

**From the folder tree.** `wire_context_menu(&folder_tree, ...)` at
`wx_app.rs:3009` binds the Applications key and Shift+F10 and asks a closure
where it is; the closure reads `selected_folder` and answers
`Focus::SavedSearch` at `wx_app.rs:3014` when the cursor is on a saved-search
row. `wx_context_menu::show` builds the menu from
`context_menu::entries_for(Focus::SavedSearch)`, which is `SAVED_SEARCHES` at
`context_menu.rs:187`, and each line raises the id `command_for` gives it:
`Action::EditSearchConditions => ID_EDIT_SEARCH_CONDITIONS` at
`wx_context_menu.rs:61`.

**From the menu bar.** `saved_search_menu` at `wx_app.rs:5831` appends
`ID_EDIT_SEARCH_CONDITIONS` at `:5833`, and that menu is attached to the Message
menu as "Saved Searc&hes" at `wx_app.rs:6104`.

**The handler.** `_ if id == ID_EDIT_SEARCH_CONDITIONS` at `wx_app.rs:4642`
calls `edit_the_chosen_searchs_conditions` at `wx_app.rs:6880`, which calls
`wx_managers::show_rule_manager_dialog` at `wx_app.rs:6898` and
`cache.replace_saved_search` at `wx_app.rs:6910`.

**Into the dialog 02-06 built.** `show_rule_manager_dialog` at
`wx_managers.rs:3066` passes `show_rule_edit` to `run_manager_loop` at
`wx_managers.rs:3101`, for Add and for Edit alike, which is what makes the
condition dialog the second of two rather than a window of its own.

**The depth is two**, the same as the filter manager: the frame opens the
condition manager, and the condition manager opens the condition editor.

A library test reads the shipping half of `wx_app.rs` through
`common::what_ships` and asserts both the window call and the replace call are
there, so removing either reddens it. `tests/wired.rs` separately asserts that
every handled command has something that raises it and that every raised command
is handled, and both pass with the new id.

**Not reached by a person yet:** nothing here has been opened in a running
build. Recorded in `.planning/WINDOWS.md`.

## WINDOWS.md

02-06's two stub entries are closed, because they have stopped being true.

- **23**, `build_rule_edit_dialog` and `show_rule_edit` have no caller: closed.
  `show_rule_manager_dialog` opens both, and it is reached from two commands.
- **24**, `replace_saved_search` has no caller outside its tests: closed. The
  write on the way out is the caller.

Four added, taking the ledger from 24 to 26 open.

- The condition manager has never been opened in a running build.
- Whether a tally on the end of every condition change reads well by ear, or is
  a clause somebody stops hearing.
- Whether the saved-search context menu reads correctly, and whether editing
  conditions first is the right order by ear.
- `wxdragon 0.9.17`'s `ListCtrl::get_item_text` loses the last character of
  every cell. Upstream defect, recorded rather than absorbed.

**The brief said the ledger stood at 19 open. It stood at 24**, which is where
02-06 left it and what 02-06's own summary says. Nineteen was where 02-05 left
it.

## Wrong premises in the plan

**1. Nothing in `context_menu.rs` decides which entries apply to the row under
the cursor.** Task 2's `read_first` says to "read how another focus decides
which entries apply to the row under the cursor". No focus does. `entries_for`
returns a static slice per focus and `wx_context_menu::show` takes a focus fixed
at wiring time; there is no row-awareness anywhere in either file.

Resolved by giving the folder tree a second focus rather than filtering a list.
That is the shape the menu bar already uses, with a Saved Searches menu beside
This Folder, and it keeps `entries_for` a plain lookup with `Focus::ALL` covering
the new place for free. `wire_context_menu` now takes a closure rather than a
focus, and ten of the eleven controls answer with a constant.

**2. The plan's own break for the one-group guard cannot be written.** Task 3
says to take the record red "by adding a second heading for searches whose
question set the In box cannot name". `rows` receives `&[SearchInTheTree]`, which
carries an identifier and a name and nothing else, so it cannot ask what a search
asks. The break as specified would need a type change, which is not a
`before`/`after` text swap.

Recorded instead as the second group somebody actually reaches for: the searches
split in two, each half under a heading of its own. That is still the wrong fix
D-2-02 forbids, and it is writable in one place.

**3. Task 3's `made_by` grep is a text search standing in for a construct**, the
same shape 02-01 and 02-02 both reported and the eighth in this phase. The
criterion filters comments, which handles one half of it; the other half is that
the field would not be called `made_by`, `provenance` or `written_by_the`
anything. A check that names the thing it forbids is answered by whatever the
thing is really called.

The criterion is satisfied as written: with comments filtered, the count is 0.
What was written instead is a compile-time guard. Both shapes are destructured
naming every field with no `..`, so a sixth field on the stored search or a third
on the tree row stops the file compiling before any test runs.

**4. `manager_words::deleted` gaining a count moved two guard records.** Not
mentioned in the plan, and the same class of thing 02-06 found with record 540.
Both were re-measured by hand, and one of them had grown.

**5. `menu_ids!` was at exactly 128 entries, the default recursion limit.** The
id this plan adds is the 129th, and the macro recurses once per name, so the list
stopped compiling. Not a wrong premise so much as a floor nobody knew was
underfoot.

## Red, green, and every break measured

Five commits, two of them red/green pairs, one test-only.

| Commit | What |
|---|---|
| `e095e99` | RED, six over the condition manager |
| `7a79ad7` | GREEN, the manager, the tally, the noun, and two records re-measured |
| `7af5c0a` | RED, six over the second door |
| `8576f54` | GREEN, the door open and the write wired |
| `27d66c7` | the one-group properties, one guard record, the changelog |

**Both RED halves are wrong reachable code rather than right unreachable code**,
for the reason 02-06 recorded and this plan met twice more. A lint denying dead
code refuses a private helper whose only caller is a test, and it also refuses an
enum variant nothing constructs. So neither of the second door's two decisions
could be stubbed by leaving an arm out: both were stubbed by getting the question
wrong instead, and one of them by swapping its two arms, which is the only shape
that keeps both variants built.

**Nine breaks measured by hand, none reasoned about.**

| Break | What reddened |
|---|---|
| the delete sentence shown but not said | 1, unchanged from the record |
| the delete sentence dropping the kind | 4, up from the record's 2 |
| the saved-search rows split under two headings | 4, two of them not about saved searches |
| the write made unconditional | 1, the nothing-changed test |
| the search rebuilt from scratch rather than carried over | 1, the same-identifier test |
| the call to the condition window taken out | 1, the source-reading check |
| a search's Rename sent to the container command | 2 |
| the row builder answering with stored names | 3, in the RED half |
| the close refusal reading the stored list rather than the new one | 2, in the RED half |

**Six tests were green on arrival and each took its red by hand.** The nought
and singular sentences, because the RED stub wired the count into the removal
path where it was already computed. The nothing-changed and same-identifier
tests, because the RED faults were elsewhere. The source-reading check, because
the call it looks for was written in the same commit. And the
not-the-container-commands test.

**All six one-group properties were green on arrival and stayed green**, which
is the measurement Task 3 existed to take rather than a disappointment. The guard
record is what makes them able to fail later.

## Guard records

543 records, and the sweep header now reads 192 + 351.

### Added: "one heading holds every saved search, however it was made"

Measured by hand against the whole 5,958-test library: **four**. Two are about
saved searches and two are not, and neither of those two is a test anybody
working on this feature would have filtered for:
`test_no_two_rows_of_one_tree_share_an_identity` and
`test_the_top_level_reads_in_the_order_somebody_meets_it`. That is the reason
`CLAUDE.md` says the filter you would naturally pick for your own subject is not
enough.

### Re-measured: "deleting from a manager window says which item went"

`deleted` gained the count argument, so the text this break replaces stopped
being in the tree and the commit gate said so. Measured again against the whole
library: still exactly one, unchanged.

### Re-measured: "a manager window's delete sentence names its kind and does not collide with mail"

Same cause. The red set **grew from two to four**, and both new ones are tests
this same change added: the tally is built from the kind as well, so dropping the
kind takes the tally with it. That is the staleness `CLAUDE.md` says never
announces itself, and it was caught here only because the break stopped naming a
place in the tree and forced a re-measurement.

**Not machine-verified beyond that.** `scripts/guards.sh` was not run, as
instructed. Three records were measured by hand and nothing else was
re-measured, so any record this branch made stale is unfound. The candidate set
for `scripts/guards.sh --touched-by d0e5a3d` is what would answer it.

## Deviations from plan

**1. [Rule 2 - Missing critical] Case sensitivity is on the condition row.** The
plan asks for three columns naming the field, how it matches and what it matches
against. Two conditions differing only in case sensitivity would then be two rows
reading identically, which is the fault this codebase keeps naming in lists. It
goes on the end of the third part and only when it is on.

**2. [Rule 2 - Missing critical] The saved-search focus offers four entries, not
one.** The plan asks for the entry that opens the editor. Adding it to the folder
list would have left a saved search offering "Get older messages", for a thing
with no server behind it, and "Folders to keep up to date". `context_menu.rs`'s
own module doc forbids exactly that: a menu entry that does nothing is a stop
somebody lands on, hears, and learns nothing from. So the row got its own list,
holding the four commands that work on it.

**3. [Rule 1 - Bug] `a_sub_dialog_needs` gained a caption.** It hardcoded "Not
added", which is wrong over a window refusing to close and was already wrong over
three dialogs whose sentence says "before this can be saved". A screen reader
reads the caption before the sentence, so the two contradicted each other. The
three saving refusals now say "Not saved", the condition editor's says "Not
saved" because it is opened on stored conditions as well as new ones, and the
close refusal says "Not closed".

**4. [Rule 3 - Blocking] `recursion_limit` raised to 512.** `menu_ids!` recurses
once per name and there were 128. Raised rather than split into two calls,
because splitting would put hand-numbered offsets back within reach and that
defect has already happened here once.

**5. [Rule 3 - Blocking] Add and Edit collapsed into one call on
`run_manager_loop`.** Adding the close-refusal parameter took it to eight
arguments and clippy's limit is seven. Every window already passed the same
function twice with only the row differing, so collapsing them removes a real
duplication rather than working around a lint, and `#[allow]` is forbidden here.

**6. [Rule 2 - Missing critical] The tally is on every change, not only on
adding and removing.** The plan leaves this as a judgement to make and record.
The must-have truth says every change, and the reason to agree with it is that a
count said after only some of the three is a number somebody has to notice the
absence of, which is the work this product exists to stop asking of people.

**7. The one-group properties are checked by destructuring, not by grepping.**
Cause above.

**8. `populate_questions` and `what_a_condition_list_still_needs` are public.**
The first because the window check that paints a real list lives in `tests/`; the
second because the write path reads the refusal rather than writing a second
wording. Both are widenings the plan does not mention.

## Known stubs

None. Everything this plan adds is reached from a command a person can raise,
traced above. The two stubs 02-06 left are closed.

## Threat flags

None new. The five mitigations this plan owed are in place.

- **T-02-29**, editing a search this build cannot read: refused with a sentence,
  in the wording the same row already gets from Enter, with tests on both arms.
- **T-02-30**, writing per change: one write on close, from the working copy
  `run_manager_loop` keeps, through 02-06's transaction. A test asserts nothing
  is written when the window closed unchanged, and the break that makes the write
  unconditional reddens it.
- **T-02-31**, a list whose size a screen reader hears differently on the two
  channels: the native `ListCtrl` in report mode, whose count comes from Windows'
  own provider. Nothing here sets a count with `set_accessible_name`, and the
  only name set on the list is the noun for what it holds.
- **T-02-32**, row data hung off a control: `grep -n 'set_item_data\|item_data'`
  over `wx_managers.rs` returns nothing at all.
- **T-02-33**, a condition list stored under the wrong search: everything but the
  questions is carried over from the stored search, with a test asserting the
  identifier, the name, the join and the folder all survive, and a break that
  rebuilds the search from scratch reddens it.

**T-02-SC** holds: no dependency added, `Cargo.toml` unchanged and still 0.46.0.

`git diff main` adds no call to `allowed_for`, no `may_i` and nothing into
`src/service/`, so SEARCH-03's third derived criterion holds and is checked
rather than assumed.

## Documentation

`docs/changelog.md` has an `[Unreleased]` entry in the same commit as the change,
describing the rule editor from a person's side: what they can now ask a saved
search, where they open it, that a search made either way is one thing in one
place, and that a search a newer version wrote is refused. No version bump.

`docs/KEYBOARD_SHORTCUTS.md` is unchanged, and that is right: the menu entries
carry mnemonic letters, not accelerators, and that file lists accelerators.

## Requirements

**SEARCH-03 is met.** Its evidence line said "Nothing joins the two into a
folder that updates itself", and that is no longer true: one stored search has
two doors, one matcher, one storage and one group in the tree, and both doors are
reachable from the running program.

What is not settled is the ear. Nothing has been heard, and the requirement's
accessibility half is what `.planning/WINDOWS.md` now carries.

## Self-Check: PASSED

- All five commits found in `git log main..HEAD`.
- `bash scripts/check.sh all`: formatting, clippy at `-D warnings`, 5,958 library
  tests and every integration target, and the release build. Exit 0.
- `grep -n '"Items"' src/presentation/wx_managers.rs` returns one line, the doc
  comment saying why it went.
- `grep -n 'set_item_data\|item_data' src/presentation/wx_managers.rs` returns
  nothing.
- `grep -c '^\[\[guard\]\]' guards/guards.toml` is 543, equal to 192 + 351, and
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  agrees.
- `grep -c '^#\[test\]' tests/manager_dialog_labels.rs` is 1, and the harness
  reports `running 1 test`.
- The provenance grep the plan specifies returns 0.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- `git diff main -- docs/KEYBOARD_SHORTCUTS.md` is empty.
- Every symbol this plan added is reached from `ID_EDIT_SEARCH_CONDITIONS`,
  which two live commands raise, traced above.

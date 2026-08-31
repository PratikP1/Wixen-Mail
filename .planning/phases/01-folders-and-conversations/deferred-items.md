# Deferred items, phase 01

> **Routed 2026-08-31. Read this before acting on anything below.**
>
> Every live item here now has a phase, because a deferred list goes stale the
> same way a tick list does and this one already had. Three went to phase 3
> (the Gmail conversation count, the late-arriving conversation root, the uid
> wrap), two to phase 6 (the per-account permission nothing offers, the reminder
> that opens over typing), and six into an inserted phase 2.1 for the ones that
> belong to no phase. The roadmap carries them; this file is now the detail
> behind those lines rather than the place they live.
>
> Three entries are not routed, for three different reasons. **"The conversation
> row is built and tested but nothing draws it yet" is closed**: it says in its
> own text that 01-12 is the plan that renders it, and 01-12 landed.
> **"The plan's order-independence criterion could not be met as written" is not
> work**, it is guidance for whoever writes the next plan of that shape. And
> **"nothing has watched a screen reader read a rethreaded row" is Pratik's**,
> like every other listening item in this project.
>
> One live item is deliberately in no phase: the spellcheck test that fails
> about one full library run in five through a Windows COM call made twice. It
> is diagnosed as far as reading and no further, so it needs investigating
> before it can be planned. Writing a criterion for it would be pretending we
> know what it is.

Things found while executing this phase that are real and are not this phase's
to fix. Written down at the moment they were found, so they stay visible.

## Two dialogs hang row data off the control, which never gets cleared

**Found during:** 01-05, task 2, by the guard written for that task.

`presentation::wx_destination` and `presentation::wx_thread_view` key their tree
rows on `wxdragon`'s tree item custom data:

- `src/presentation/wx_destination.rs:150` — `append_item_with_data`
- `src/presentation/wx_destination.rs:82` — `get_custom_data`
- `src/presentation/wx_thread_view.rs:201` — `set_custom_data`
- `src/presentation/wx_thread_view.rs:221` — `get_custom_data`

That data goes into a process-global map. `store_item_data` inserts into a
static registry, `delete_all_items` calls the raw FFI and removes nothing from
it, and `cleanup_all_custom_data` returns early on any item with no children, so
it never clears a leaf. Every row in both of these is a leaf. Nothing is ever
freed for the life of the process.

**Why it is not fixed here.** Both are built when a dialog or a view opens
rather than on a timer, so each leaks per opening rather than per sync. The
folder tree is the severe case, because it is rebuilt whenever a sync finishes,
and that is the one 01-05 was about. Fixing these two means giving each its own
parallel vector, in files this plan does not otherwise touch.

**How it stays visible.** The guard
`folder_tree::nothing_hangs_off_the_control::test_no_row_of_a_tree_hangs_its_identity_off_the_control`
names both files in an exception list with the reasoning beside it, rather than
scoping itself narrowly enough not to see them. Any new file that starts doing
this fails the test. Removing a file from that list is how each of these gets
closed.

**Size:** small each, one parallel vector apiece, following
`WxUIState::tree_rows`.

## A spellcheck test fails about one full library run in five

**Found during:** 01-05, running the whole suite to confirm the plan's work.

`data::config::permission_tests::test_a_fresh_installation_checks_spelling_in_this_machine_s_language`
failed once in five full runs of `cargo test --lib` and passed every other
time, including when run on its own.

It compares `AppConfig::default().language` against
`service::spellcheck::language_of_this_machine()`. That reaches
`available_languages()`, which on Windows asks the platform speller through COM
for its supported languages. Under the test harness's threads that call can
answer with an empty list, and an empty list makes `best_available_match`
answer `None`, so the test's own side falls back to `"en"` while the default it
is comparing against was computed when the call worked. The two then disagree
and neither is wrong.

**Not caused by this plan.** Nothing in 01-05 touches configuration, language or
spelling. Adding about thirty-five tests changes how the harness schedules
threads, which is enough to expose a race that was already there.

**What it costs.** A suite that fails once in five runs for a reason nobody has
diagnosed is the shape CLAUDE.md's fourth guardrail is about: a check that fails
two ways without saying which. Somebody will eventually read this failure as a
real one, or worse, learn to rerun until it passes.

**What would fix it.** Ask the platform once and cache it, or have the test take
the language list as an argument the way `choices_from` already allows, so the
COM call is not made twice and compared with itself. The second is the smaller
change and matches the reasoning already written above `choices_from`.

## A permission per account is stored, honoured, and offered by nothing

**Found during:** 01-06, task 1, by the D-43 mirror guard the moment it was
written. This is the first thing that guard found, and it was already in the
tree.

`AppConfig::allowed_per_account` is a map of account id to `Allowed`. It is read
by `AppConfig::allowed_for`, which is honoured all the way out to the provider
clients, so an entry in it really does narrow what one account may change.
Nothing outside its own tests has ever written one. The field's own doc comment
has said so for some time: the testing page and the first-run screen both used
to offer it as a control somebody could reach, and neither does now.

This is worse than a setting nothing reads. It is honoured, so the program can
behave in a way the person using it cannot see, cannot change and has no screen
to look at.

**Why it is not fixed here.** Closing it means a control per account on the
settings screen, which is a feature rather than a line: the screen has no
per-account section, the answer it writes can only ever narrow the
application-wide one, and the sentence a sync says would have to name the
account. None of that is this plan's work and none of it is in its file list.

**How it stays visible.** It is named in `STORED_AND_OFFERED_BY_NOTHING` in
`src/data/config.rs`, beside
`test_every_setting_somebody_can_change_is_offered_by_a_screen`, rather than the
guard being narrowed until it cannot see it.
`test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing`
checks the other direction, so the moment a screen does offer it the suite asks
for the entry to be taken out. Taking it out of that list is how this gets
closed.

**Size:** medium. A per-account group on the settings screen, one control per
account, plus the write-back and the wording for the sentence a sync says.

---

## `next_local_uid` hands out 0 after the number range wraps (found in 01-07)

`MessageCache::next_local_uid` at `src/data/message_cache/messages.rs` reads
`MAX(uid)` as an `i64`, adds one with `saturating_add`, and then casts the result
to `u32`. The saturation is therefore on the `i64`, which cannot be reached from
a `u32` column, while the cast that follows wraps. A folder whose highest number
is `u32::MAX` gets `4294967296 as u32`, which is `0`, and the folder after that
gets `1`: numbers already in use, against a `UNIQUE(folder_id, uid)` constraint.

**How it was found.** Not by reading the code. A test in 01-07 needed a landing
to fail on purpose and tried to arrange it by seeding a folder at the top of the
range, on the assumption that the next number would saturate and collide. It did
not collide, because it wrapped to zero, and the test passed for the wrong
reason until the numbers were read.

**Why it is not fixed here.** It is not reachable in any database this program
can currently produce. Local folders count up from one and nothing has ever come
near four billion messages in one folder, and the reserved end counts *down* from
`u32::MAX` through `next_reserved_uid`, which is a different function with a
different failure. Fixing it properly means deciding what a local folder does
when its numbering is exhausted, which is a decision rather than a line, and it
is nothing to do with sharing folders between accounts.

**How it stays visible.** This entry, and nothing else: there is no test, because
building a fixture with four billion rows is not a test anybody would run. That
is a weaker answer than this file usually accepts and it is recorded as such.

**Size:** small to fix the arithmetic, medium to decide and test what exhaustion
should mean.

---

## `.planning/intel/context.md` still lists folder management as not working (found in 01-09)

`.planning/intel/context.md:25` says, of a source it is quoting, that "a folder
cannot be created, renamed or deleted, and a whole folder cannot be marked read
or emptied". Every one of those five is now built: create, rename and delete in
01-04, mark read and empty in 01-09.

**Why it is not fixed here.** The line is framed as "Not working per this
source", so it is a record of what one source said at a point in time rather
than a live claim by this project. Rewriting a quotation to match the tree makes
the intel document say the source said something it did not. What it needs is a
date or a note saying the snapshot has been overtaken, which is a decision about
how that file records things and is nothing to do with emptying a folder.

The two documents that *were* making the claim in their own voice,
`docs/ALPHA_TESTING.md` and `docs/IMPLEMENTATION_STATUS.md`, were corrected in
01-09 commit `463fc41`.

**How it stays visible.** This entry. Nothing reads `.planning/intel/` in any
check, which is the whole reason it went stale unnoticed.

**Size:** small, once somebody decides whether that file dates its claims or
refreshes them.

---

## A reminder alert still opens over somebody who is typing (found in 01-10)

`raise_what_is_due` in `src/presentation/wx_app.rs` shares its one-at-a-time
gate with the question about folders a server has stopped listing, so neither
can open over the other. It does **not** share the second gate 01-10 added: it
never asks `one_question_at_a_time::somebody_is_typing()`.

So a reminder coming due while somebody is writing a message still opens a modal
window over them. That is the failure the changelog's "An automatic draft save no
longer opens the spelling check" entry is about, arriving from a different timer.

**Why it is not fixed here.** D-27 is about the question this plan builds, and
nothing has decided that a reminder should wait. It arguably should not: a
reminder that waits until somebody stops typing is a reminder that can be an
hour late, and being told at the time is the whole point of asking to be told.
That is a decision about what a reminder is for, not a line of code, and it is
Pratik's rather than an executor's.

**How it stays visible.** This entry. The gate is one function call away
(`one_question_at_a_time::somebody_is_typing()`), so if the decision goes the
other way the change is small.

**Size:** small to make it wait, medium to do something better, such as raising
it without focus or holding it for a short while and then raising it anyway.

---

## Nothing exercises the window that asks about folders that have gone (found in 01-10)

`ask_about_the_folders_that_have_gone` in `src/presentation/wx_app.rs` is the
only part of 01-10 no test reaches. Everything it decides is in
`presentation::one_question_at_a_time` and is tested there without a window:
whether to raise, what the question says, what each answer means, what to record
and what to say afterwards. What the function itself does is hold the turn, ask
the two questions, build the dialog and carry out the answer.

**Why it is not fixed here.** Reaching it needs a running application, an
account, a folder tree and a modal window that somebody answers. wxWidgets
supports one application per process, and `01-RESEARCH.md`'s Pitfall 7 records
that one dialog-building test per file under `tests/` is the ceiling. A test
that built the window could not answer it.

**What it costs.** The two constraints are proved as decisions and not as
behaviour of the running program. If the call site stopped passing
`turn.is_none()`, or stopped asking the boxes for focus, every test here would
stay green.

**How it stays visible.** This entry, and the same listening pass the rest of
this phase is waiting on. The narrower part, that the arguments really are
computed from the gate and the controls, could be closed by giving the function
its two answers as a small struct built by a tested function.

**Size:** small for the argument-building half, large for anything that answers
a real window.

## Gmail mail that is archived with no label disappears from a conversation count

**Found during:** 01-11, task 2, while implementing D-08's all-mail exclusion.

D-08 says folders whose `holds_all_mail` server fact is true are excluded from a
conversation's count, and that is what `conversations_in` does. It is right for
the case it was chosen for: on Gmail every message is in All Mail as well as in
its label, so counting both reports twice the size of every conversation.

There is a case it is wrong for. A Gmail message that has been archived and
carries no label is in All Mail and nowhere else. Excluding All Mail from the
count excludes that message, and a conversation made only of such messages is
counted as holding none, so it does not appear at all when somebody is standing
in All Mail reading it.

**Why it is not fixed here.** D-08 is a locked decision and it names the
exclusion by that name. The fix is a different rule, not a different
implementation of this one: count each message once, keyed on something that
identifies the same message across folders. `messages.gmail_message_id` is
exactly that and is already stored, and the `Message-ID` header is the general
answer, but neither is present on every row. Choosing between them, and deciding
what to do for a message that has neither, is a decision rather than a fix.

**What it costs.** Archived Gmail mail with no label, read from All Mail, is not
listed as a conversation. Mail that carries any label is unaffected, and so is
every provider that is not Gmail, because the exclusion only applies where a
server said a folder holds a copy of everything.

**How it stays visible.** This entry, and
`test_the_row_is_still_there_when_the_folder_being_read_is_the_all_mail_one`,
which pins the half that does work: standing in All Mail lists the conversations
whose messages also live in a label.

**Size:** medium. One extra predicate in one query, once the rule is decided.

## Sorting messages by Safety orders them by the alphabet, not by severity

**Found during:** 01-11, task 3, writing the conversation's own Safety rule.

`MessageColumn::Safety`'s message sort expression is `m.safety`, and safety is
stored as the words "ordinary", "suspicious", "spam" and "phishing" so a stored
mailbox can be read by somebody looking at it with a SQLite browser. In that
alphabet the order is ordinary, phishing, spam, suspicious, so sorting the
message list by Safety descending puts the mildest verdict at the top and
phishing near the bottom.

**Not caused by this plan.** The expression predates it and is untouched. The
conversation rule beside it does rank by severity, because "the worst in the
conversation" cannot be answered any other way, so the two now disagree about
what "worse" means.

**Why it is not fixed here.** It is a change to how the message list sorts,
which is outside what this plan is about, and it would want its own test saying
what somebody expects to hear when they press the Safety header.

**Size:** small. The same `CASE` the conversation expression already uses.

## The conversation row is built and tested but nothing draws it yet

**Found during:** 01-11, task 3, at the end.

`message_rows::conversation_cell_text` and
`Sort::conversation_order_by_clause` are written, tested per column and proved
to agree with each other, and no non-test path calls either. The data half is
reached: `conversations_in` is called when a conversation is opened, so the
count a person hears comes from it. What has no caller is the drawing of a
collapsed conversation row.

**Why it is not a defect.** 01-12 is the plan that renders the list, and this
plan's own objective says so: "the data every row in plan 01-12 renders". It is
recorded here rather than left implicit because compiling and passing tests is
not done, and a reader of this phase should be able to see which half is which.

**How it stays visible.** This entry, and 01-11-SUMMARY.md's Known Stubs
section.

**Size:** none here. It is 01-12's work.


## Folder management sits under "What does not work" and describes work that does (found in 01-12)

`docs/IMPLEMENTATION_STATUS.md` has a **Folder management** paragraph under the
heading `## What does not work`. Everything it describes is built and was built
by this phase: making, renaming, moving and deleting a folder in 01-04, marking
one read and emptying it in 01-09, and the two settings that decide how far
those reach in 01-06.

Only its last sentence justifies the placement, and it is a different claim:
"Nothing on the server side of any of this has run against a real mail server."
That is true of every write in the program and has its own entry two paragraphs
below, **Anything that writes, against a real account**.

**Why it is not fixed here.** Moving it means deciding what the heading means.
Read as "not built", the paragraph is in the wrong place. Read as "not proven",
so is most of what is under `## What works`, including the provider sync
paragraph that ends with the same sentence. The document needs one answer to
that question rather than a paragraph moved, and choosing it is a decision
about how this file reports an unproven feature.

**How it stays visible.** This entry. Nothing checks the placement of a
paragraph under a heading, which is why it went unnoticed after 01-09 rewrote
the text and left it where it was.

**Size:** small to move the paragraph, medium to settle what the two headings
mean and re-sort the file against that.


## A conversation root arriving after a message that names it is not merged (found in 01-13)

**Found during:** 01-13, task 1, by writing the order-independence test as a
simulation of the storage path rather than as a property of the pure function.

THREAD-02's merge works when the connecting message arrives after the
conversations it connects, which is the case the requirement names and the case
that is tested. It does not work in the other direction.

A message `x` names `a` and `c` in its reply chain. Stored first, `x` takes the
conversation `a`, because that is what its chain names. When `c` then arrives
with no chain of its own, nothing it can be asked about names `a`, so it starts
a conversation of its own and the two stay separate. Three of the six arrival
orders over that set merge and three do not.

**Why it is not fixed here.** The knowledge exists and is unreachable. It lives
in `x`'s stored `refs_header`, and finding it means asking "does any stored
message name `c` in its chain", which is a substring search over a text column
that no index can serve. On a mailbox of any size that is a scan per arriving
message.

Closing it properly needs a table mapping every identifier a message names to
the conversation that message is in, written on store and read on arrival. That
is one indexed lookup in both directions and it is the standard shape for this
problem. It is also a new table, which is an architectural decision this plan
did not carry and CLAUDE.md's additive-schema rule would allow but not decide.

**How it stays visible.**
`src/application/thread_identity.rs#a_root_arriving_after_the_message_that_names_it_is_left_out_of_the_merge`
is a passing test that asserts the gap, named for what is not achieved, with a
comment saying that if it starts failing the gap has been closed and it should
become another case of the test above it. The changelog says it in a sentence
somebody using the program can read, under **Known limitation**. It is also in
`WINDOWS.md` and in `01-13-SUMMARY.md`.

**Size:** medium. One table, one index, a writer in `upsert_message` beside the
existing one, a reader in the merge, and a backfill for mail already held.


## The plan's order-independence criterion could not be met as written (found in 01-13)

**Found during:** 01-13, task 1.

`01-13-PLAN.md` task 1 requires "a test [that] assigns ids in at least three
different orders over a set containing a merge and asserts identical final
assignments", and mandates a signature taking the arriving message's chain and
what the cache found. Those two cannot both hold, for the reason in the entry
above: the lookup can see messages the arriving one names, never messages that
name it.

**Why this is written down separately.** The gap above is about the product.
This one is about the planning: the criterion reads as achievable, three
documents agree with it, and nothing short of running the permutations shows
otherwise. It is the fourth wrong premise in this phase that was a plan
asserting a property rather than a plan naming a wrong symbol, and those are the
expensive kind.

**How it stays visible.** This entry and `01-13-SUMMARY.md`.

**Size:** none. It is a note for whoever writes the next plan over this code.


## Two writers spell a message identifier differently (found in 01-13)

**Found during:** 01-13, task 2, when a new lookup joining `messages.message_id`
against `messages.thread_id` returned nothing.

`messages.message_id` holds two formats:

- Bare, with no angle brackets, for anything that arrived through
  `mail_parser`, which strips them. That is `service::mime`,
  `application::filing::a_row_filed_here` and the sync at
  `application::mail_sync`.
- Wrapped in angle brackets, for a draft this program composes, because
  `application::draft_message::message_id_for` builds
  `<draft-...@wixen-mail.invalid>` and it is stored as written.

`messages.thread_id` is always bare, because `thread_identity::conversation_root`
strips on the way in. So a lenient reader and a verbatim writer answer the same
question two ways, which is the shape this project has been bitten by before.

**Why it is not fixed here.** 01-13 widened the query to ask for both forms,
with the reasoning at the query. Making the column consistent means rewriting
values that have shipped, which CLAUDE.md forbids without a stated reason, and
deciding which form is canonical, which affects every reader of the column
rather than only the one added here.

**How it stays visible.** A comment at
`MessageCache::threads_holding_any_of` naming both writers, this entry, and
`WINDOWS.md`.

**Size:** small to normalise on write and add a backfill; medium to be sure
every reader of `message_id` agrees, including the IMAP search that quotes it
onto the wire.


## Nothing has watched a screen reader read a rethreaded row (found in 01-13)

**Found during:** 01-13, task 3.

THREAD-02's criterion is that rethreading on arrival "does not re-announce rows
the user is not on". What is proved is the mechanism: the rule deciding which
rows changed is tested six ways and guarded, the control is told to repaint
those rows and not the list, it is told its size only when the size moved, and
the selection is not touched. What is not proved is what somebody hears.

Whether repainting one row of a virtual `wxListCtrl` is silent to NVDA, and
whether `set_item_count` on an unchanged count would have been audible anyway,
are questions about a real screen reader on a real window with real mail
arriving. Nothing in this program has run against a real mail account, so the
arrival itself has never happened outside a test.

**Why it is not fixed here.** It needs Pratik, NVDA, and an account.

**How it stays visible.** This entry, `WINDOWS.md`, and the `human_judgment`
flag on the coverage entry in `01-13-SUMMARY.md`.

**Size:** none in code. One session with a screen reader.


## Ten checks in tests/wired.rs read a prefix of wx_app.rs and cannot say so (found in 01-14)

**Found during:** 01-14, task 1, by adding a test module partway through
`src/presentation/wx_app.rs` and watching four unrelated integration tests fail.

Ten checks in `tests/wired.rs` read that file's source and cut it with

```rust
let ship = &app[..app.find("\n#[cfg(test)]").unwrap_or(app.len())];
```

That is not "the half that ships". It is "the file up to the first test module",
and the two are the same string only while every test module sits at the end of
the file. A module added at line 9,683 of 24,000 left those ten reading the
first 9,600 lines.

**Four of the ten failed loudly, and that was luck.** They call `body_of`, which
panics when the signature it is looking for has gone, so they said "this guard is
measuring nothing". The other six only use `contains`, so they went on passing
over a third of the file and said nothing at all. Those six are the real defect:
a check that has silently narrowed cannot be told from a check that has nothing
to find.

**What was done instead.** The new test module was moved to the foot of the file
and a note left where it would naturally have gone, saying why. `the_window_itself`
in `wx_app.rs` had exactly the same bug and was fixed properly, because it is
inside the crate and can call `common::what_ships`, which is the one correct
answer this tree already has and whose own module doc opens by naming this bug.

**Why the ten are not fixed the same way.** `common::what_ships` is `#[cfg(test)]`,
and an integration test links the library built without `cfg(test)`, so
`tests/wired.rs` cannot reach it. Copying the parser into `wired.rs` would be a
second copy of the thing whose whole point is being the only copy, and three
wrong copies of it are what `what_ships` was written to replace.

**What would fix it.** Make one correct answer reachable from both. The options,
none of them free: put `what_ships` behind a cargo feature that dev builds turn
on; move it to its own small crate that both depend on; or give up the release
gating and make it a plain `pub mod`, which costs about sixty lines of pure
string handling in the binary and puts a source-slicer in the library's public
surface. That is a decision about the crate's shape, not a fix to make in
passing, which is why it is here.

**How it stays visible.** This entry, and the note in `wx_app.rs` where the test
module would otherwise have gone, which is the place somebody will be standing
when it next matters.

**Size:** small to add the feature flag and change ten lines. The size is in
deciding which of the three shapes the crate should have.

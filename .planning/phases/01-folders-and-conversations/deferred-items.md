# Deferred items, phase 01

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


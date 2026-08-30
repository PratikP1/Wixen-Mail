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

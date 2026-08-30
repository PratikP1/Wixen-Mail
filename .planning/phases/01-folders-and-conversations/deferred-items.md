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

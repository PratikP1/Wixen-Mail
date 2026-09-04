# Requirements audit, 2026-09-04

Eighteen requirements checked against the tree at `d3c6c7d` on `main`:
FOLDER-01 to 03, THREAD-01 and 02, SEARCH-01 to 03, FEEDBACK-01 to 03,
PERF-01 to 07. Nothing else was read and nothing was changed.

## Summary

| Verdict | Count | Requirements |
|---|---|---|
| Wrong | 12 | FOLDER-01, FOLDER-02, FOLDER-03, THREAD-01, THREAD-02, SEARCH-01, SEARCH-02, SEARCH-03, PERF-03, PERF-05, PERF-06, PERF-07 |
| Partly stale | 1 | FEEDBACK-02 |
| Still accurate | 5 | FEEDBACK-01, FEEDBACK-03, PERF-01, PERF-02, PERF-04 |

**Every one of the twelve is wrong in the same direction: it says something is
missing that has since shipped, or it names a defect that has since been
fixed.** Nothing in the eighteen over-claims. That is the shape phase 3's
research already found, and it is worse in FOLDER, THREAD and SEARCH because
phases 1, 2 and 2.1 landed after the document was written on 2026-08-29.

**All eight of FOLDER, THREAD and SEARCH are built, wired and reachable from a
menu.** Traceability already marks seven of them Complete. FOLDER-02 is the one
still marked Pending, and its code shipped too; what is genuinely outstanding
there is a screen reader run, which no test in this repository can answer.

Three things are unbuilt and their requirements say so correctly: PERF-01,
PERF-02 and PERF-04, none of which has ever been measured.

**Why the evidence went stale in a way nobody noticed.** Most of these blocks
cite a line number or quote a nil `grep`. Line numbers move, and a `grep`
written in the day's vocabulary returns nothing once the feature ships under a
different name. FOLDER-01's grep is the clearest case: it searched
`create_folder`, and the feature shipped as `create_mailbox`. Re-running that
command today still returns nothing, so the document reads as confirmed. A
suggestion is at the end of this file.

---

## The three buckets

Kept apart on purpose. Nothing in this audit was assigned to bucket 1 without
finding a menu item or handler that reaches it.

### Bucket 1: exists and is reached from a non-test path

Everything below is reachable from the running program, each with the menu
identifier and the handler line that dispatches it.

| Thing | Menu id | Handler |
|---|---|---|
| Create a mail folder | `ID_NEW_FOLDER` (`wx_app.rs:83`, item at `5448`) | `wx_app.rs:3576`, reaching `create_mailbox` at `7568` |
| Rename a mail folder | `ID_RENAME_FOLDER` (`wx_app.rs:84`, item at `5951`) | `wx_app.rs:3659`, reaching `rename_mailbox` at `8816` |
| Move a mail folder | `ID_MOVE_FOLDER` (`wx_app.rs:85`, item at `5961`) | `wx_app.rs:3648`, reaching `rename_mailbox` at `8675` |
| Delete a mail folder | `ID_DELETE_FOLDER` (`wx_app.rs:86`, item at `5972`) | `wx_app.rs:3587`, reaching `delete_mailbox` at `8143` |
| Empty a folder | `ID_EMPTY_FOLDER` (`wx_app.rs:87`, item at `5989`) | `wx_app.rs:3625` |
| Mark a folder read | `ID_MARK_FOLDER_READ` (`wx_app.rs:88`, item at `5994`) | `wx_app.rs:3637`, reaching `mark_folder_read` at `8539` |
| Pin and unpin a folder | `ID_PIN_FOLDER` / `ID_UNPIN_FOLDER` (`wx_app.rs:91` and `92`, items at `6035` and `6040`) | `wx_app.rs:3613` |
| Thread view | `ID_THREAD_VIEW` (`wx_app.rs:116`, check item at `5696`, `Ctrl+T`) | `wx_app.rs:4037`, calling `switch_the_view` |
| Edit a saved search's conditions | `ID_EDIT_SEARCH_CONDITIONS` (`wx_app.rs:221`, item at `5904`) | `wx_app.rs:4683`, reaching `show_rule_manager_dialog` at `7052` |
| Load a 200,000 message sample mailbox | `ID_LOAD_SCALE_SAMPLE` (`wx_app.rs:93`, item at `6281`) | `wx_app.rs:4908` |
| Fetch missing message text | `ID_FETCH_MISSING_TEXT` (`wx_app.rs:156`, item at `5486`) | `wx_app.rs:4220` |

Also in this bucket, without a menu item of their own:

- Nested folders. `folders.parent_id` is written at sync and read by
  `presentation/folder_tree.rs`, whose `TreeRow` carries `depth`
  (`folder_tree.rs:214`) and deliberately keeps level out of the label
  (`folder_tree.rs:212`).
- Collapse state across a restart. `set_row_collapsed`
  (`data/message_cache/folders.rs:275`) and `collapsed_rows` (`:309`) write and
  read a `tree_state` table, called from `wx_app.rs:13732`, `10224` and `14909`.
- Thread assignment on arrival. `messages.rs:834` computes
  `thread_identity::conversation_root` as each message is stored, and
  `messages.rs:976` calls `thread_identity::rejoin` for the merge case.
- Search coverage disclosure. `how_much_message_text_the_index_holds`
  (`searching.rs:609`) reaches the search box through `managers.rs:1852` and the
  saved search through `wx_app.rs:6460` and `6671`.
- Per-field saved search scope. `what_that_answer_looks_at`
  (`saved_searches.rs:538`) is called by `what_a_typed_search_asks` (`:554`).
- The English-only date notice. `date_display::ENGLISH_ONLY` (`:93`) is on the
  settings screen at `wx_settings.rs:1251`.

### Bucket 2: exists but only tests reach it

- **Per-event feedback channels.** `FeedbackSettings::set_event_channels`
  (`presentation/accessibility/feedback.rs:424`) is private and the `per_event`
  field (`:392`) is private. The only shipping caller is `from_stored` at
  `:496`. Everything else calling it is a test. This is FEEDBACK-01 and it is
  covered below.
- **`FeedbackSettings::channels_for`** is read on the shipping path
  (`accessibility.rs:213` and `:235`), so the *reading* half is bucket 1. It is
  the writing half that nothing but a hand-edited config file can reach.

### Bucket 3: does not exist

- Any memory measurement. No `benches/` directory, no `criterion` or `divan` in
  `Cargo.toml`, no resident-memory reading anywhere in `src/`.
- Any startup timing. Nothing in `src/` times process start to a usable list.
- A page cache of rows around the viewport. The virtual text callback
  (`wx_app.rs:1101`) reads the whole loaded list out of `state.messages` in
  memory. The mail-at-scale plan's paging with a placeholder on a cache miss is
  not built.
- A test asserting the virtual text callback issues no SQLite query. The
  property holds by construction and a comment says so (`wx_app.rs:1093`), but
  no test asserts it.
- A whole-tree mutation run with a trustworthy result.
- Smart folders as a separate object. Decision D-2-01 replaced that shape with
  a fuller editor over saved searches, which is built. See SEARCH-03.

---

## Requirement by requirement, most wrong first

### FOLDER-01: create, rename, delete, mark read, empty

**Verdict: wrong.** Both factual claims in the Evidence block are false, and
the requirement is fully satisfied.

**What is true now.** All five operations are built and reachable from the
Action menu. See the bucket 1 table for menu ids and handlers. `create_mailbox`
is at `service/protocols/imap.rs:907`, `rename_mailbox` at `:944`,
`delete_mailbox` at `:976`, each fronted by a thin pass-through in
`application/mail_controller.rs` at `546`, `560` and `571`. Marking a folder
read is `data/message_cache/messages.rs:1614`.

Why the grep in the evidence still returns nothing: it searched `create_folder`,
`rename_folder`, `delete_folder`, and the feature shipped as `create_mailbox`,
`rename_mailbox`, `delete_mailbox`. The command in the document is a live
command that has quietly stopped asking the question it was written to ask.

The four `[D]` lines are all satisfied and all still correct as written:

- The `Allowed::mail` gate is inside the session, not the controller, so no
  caller can answer it differently. `imap.rs:908` opens `create_mailbox` with
  `self.may_i("create a folder on the server")?`.
- The local exemption holds. `local_folders::is_local` still exists at
  `application/local_folders.rs:110` and is the single decider, read from
  `blocking.rs:342`, `importing_messages.rs:110`, `import_tree.rs:1081` and
  `1376`, `mail_sync.rs:528`, `603` and `1368`, and `saved_searches.rs:1498`.
- The write census in `service/outward.rs:789` now records exactly 11 outward
  calls in `imap.rs`, with a comment saying 11 since `delete_mailbox` arrived
  and 10 since `rename_mailbox` did.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04, and the previous evidence is worth reading
    for how it failed. It ran `grep -rn "create_folder|rename_folder|
    delete_folder" src/`, found nothing, and concluded the feature was absent.
    The feature shipped under `create_mailbox`, `rename_mailbox` and
    `delete_mailbox`, so that command returns nothing today too and reads as
    confirmation. All five operations are built and reachable from the Action
    menu: `service/protocols/imap.rs` has `create_mailbox` at line 907,
    `rename_mailbox` at 944 and `delete_mailbox` at 976, each opening with the
    `may_i` gate; `application/mail_controller.rs` passes them through at 546,
    560 and 571 with no logic of its own; `data/message_cache/messages.rs:1614`
    is `mark_folder_read`. The menu items are `ID_NEW_FOLDER`,
    `ID_RENAME_FOLDER`, `ID_MOVE_FOLDER`, `ID_DELETE_FOLDER`,
    `ID_EMPTY_FOLDER` and `ID_MARK_FOLDER_READ`, declared at
    `presentation/wx_app.rs` lines 83 to 88 and 91 to 92, with handlers from
    line 3576 onward. `service/outward.rs:789` records exactly 11 outward calls
    in `imap.rs`, re-measured 2026-08-31.
```

---

### THREAD-01: one row per conversation

**Verdict: wrong.** The evidence says the command "exists and is switched off".
It is on, it has a keyboard shortcut, and the disabling call is gone.

**What is true now.** `ID_THREAD_VIEW` is declared at `wx_app.rs:116` (not 102),
appended as a **check** item at `5696` (not 5026) with the accelerator `Ctrl+T`,
and dispatched at `4037` to `switch_the_view`. Its ticked state is kept in step
by `sync_menu_check` at `12323` and `12355`.

The `item.enable(false)` call the evidence names at line 582 does not exist. The
only occurrence of that string in the file is inside a test fixture at
`wx_app.rs:25627`. The guard covering this is documented at `wx_app.rs:25483`
onward and records what happened: the check was written against
`find_item(ID_THREAD_VIEW)`, an anchor that only existed while the item was
disabled, so making the check green deleted its own anchor and it passed
unconditionally for a stretch. The anchor is now the identifier itself
(`wx_app.rs:25501`).

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Built and on. `ID_THREAD_VIEW` is declared
    at `src/presentation/wx_app.rs:116`, appended as a check item at 5696 with
    `Ctrl+T`, dispatched at 4037 to `switch_the_view`, and its tick kept in step
    by `sync_menu_check` at 12323 and 12355. The `item.enable(false)` the
    previous evidence named is gone; the only occurrence of that string in the
    file is a test fixture at 25627. The guard over it is documented at 25483
    and had to change its anchor: it was written against
    `find_item(ID_THREAD_VIEW)`, a call that existed only while the item was
    disabled, so its own green half deleted what it read and it passed
    unconditionally. It now anchors on the identifier (25501).
```

---

### SEARCH-01: a saved search keeps its whole scope

**Verdict: wrong.** The evidence was rewritten once already on 2026-08-29 and
has gone stale again. The defect it describes is fixed.

**What is true now.** `what_a_typed_search_asks`
(`application/saved_searches.rs:554`, not 350) no longer always writes the three
questions. It calls `what_that_answer_looks_at` (`:538`), which matches on
`WhereToSearch` and returns `["subject"]` for `SubjectOnly`, `["from"]` for
`SenderOnly`, and `WHAT_A_TYPED_SEARCH_LOOKS_AT` for `EveryFolder` and
`OneFolder` (`:539` to `:541`). It also writes the folder half from
`ran.the_folder_looked_in` at `:566`, so both halves arrive together, which is
what the second `[D]` line asked for.

The fourth `[D]` line about a search saved by an older version no longer needs
answering, and the code says why at `saved_searches.rs:529`: the unnarrowed
answer is `WHAT_A_TYPED_SEARCH_LOOKS_AT` itself rather than a copy, so an old
search and a new unnarrowed one are byte-identical and there is no absent value
to interpret.

Other line numbers that have moved: `what_the_in_box_offers` is at
`wx_app.rs:19489` (not 14776), `search_messages` at
`data/message_cache/searching.rs:477` (not 393), `SavedSearch.folder` at
`saved_searches.rs:1063` (not 543). The three tests the evidence cites at 584,
614 and 639 are now at `searching.rs:690`, `727` and `758`, renamed to
`test_a_search_in_the_folder_showing_does_not_answer_with_another_folder`,
`test_a_search_of_subjects_alone_does_not_answer_with_a_sender_or_a_body` and
`test_a_search_of_senders_alone_does_not_answer_with_a_subject`.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04, having been rewritten once already on
    2026-08-29. Closed. `what_a_typed_search_asks`
    (`src/application/saved_searches.rs:554`) now calls
    `what_that_answer_looks_at` (line 538), which matches on `WhereToSearch`
    and returns `["subject"]` for SubjectOnly, `["from"]` for SenderOnly, and
    `WHAT_A_TYPED_SEARCH_LOOKS_AT` for the two that do not narrow a field. Both
    halves of the scope are written from one value: the folder comes from
    `ran.the_folder_looked_in` at line 566. The fourth criterion, about a search
    saved by an older version, stopped needing an answer rather than being
    answered: the unnarrowed case returns the shared constant itself rather than
    a copy, so an old search and a new unnarrowed one cannot be told apart and
    there is no absent value anywhere. Line 529 says so. The live search side is
    unchanged and still honours every scope it offers:
    `src/data/message_cache/searching.rs:477` takes `looking_in: WhereToSearch`,
    with tests at 690, 727 and 758.
```

---

### PERF-07: a whole-tree mutation run

**Verdict: wrong.** The evidence says a whole-tree run "has never been done".
One was done, on 2026-08-05, and it is the reason `scripts/mutants.sh` refuses
what it refuses. The guard record counts are also stale.

**What is true now.** `CLAUDE.md:544` records the whole-tree run of 2026-08-05:
it recorded 595 mutants as unviable and 473 of those had never reached a
compiler, so about a third of the run was untested and its summary said so
nowhere. That is the run the script's refusals were written against.

So the accurate claim is not that no whole-tree run has been attempted. It is
that no whole-tree run has produced a result anybody can trust, and that the
2026-08-05 attempt is the evidence for why the refusals exist. The difference
matters to a planner: reading "never been done" plans a first run and does not
know what shape of failure to expect.

`guards/guards.toml` now holds **565** records, counted 2026-09-04 with
`grep -c "^\[\[guard\]\]" guards/guards.toml`. The evidence says 501.
`CLAUDE.md:474` says 564, so even the guardrails file is one behind. The "192
hand-verified as of 2026-08-12" figure has no counterpart in the file today:
every record carries a name and a measured red list, so there is no marker
separating verified from unverified.

The two `[S]` claims about the script are still accurate. `scripts/mutants.sh:18`
refuses a run in which the suite was never once run against a mutant, and
`:35` documents the `--since main` case.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Scoped runs plus one untrustworthy
    whole-tree run. mime and error on 2026-07-26; filters, due, tagging and
    signatures on 2026-08-01 with 157 mutants; the four message-disposition
    modules on 2026-08-12 with 66 mutants and 1 survivor. A whole-tree run was
    attempted on 2026-08-05 and is recorded at `CLAUDE.md:544`: it marked 595
    mutants unviable, of which 473 had never reached a compiler, so about a
    third of it was untested and the summary said so nowhere. That run is why
    `scripts/mutants.sh` now refuses a partial run, a run whose build failed
    before anything changed, and a run in which the suite was never once run
    against a mutant. So what has never happened is a whole-tree run with a
    result anybody can trust, not a whole-tree run. `guards/guards.toml` holds
    565 records, counted 2026-09-04 with
    `grep -c "^\[\[guard\]\]" guards/guards.toml`; `CLAUDE.md` says 564 and is
    one behind. There is no longer a verified-versus-unverified split in the
    file, so the earlier "192 hand-verified" figure has nothing to compare
    against.
```

---

### FOLDER-03: pin folders as favourites

**Verdict: wrong.** "No favourites path in `src/`" is false. The whole thing is
built, wired, and the decision the requirement asked for in advance was written
down in advance.

**What is true now.** `src/application/favourites.rs` exists, 664 lines, with
`Pin` (`:73`), `PinnedBranch` (`:91`),
`what_each_account_has` (`:130`), `in_account_order` (`:154`) and the four
announcement builders `now_pinned`, `already_pinned`, `now_unpinned` and
`was_not_pinned` at `183`, `193`, `203` and `208`. `ID_PIN_FOLDER` and
`ID_UNPIN_FOLDER` are at `wx_app.rs:91` and `92`, with the handler at `3613`.

The fifth `[D]` line, about the pinned group announcing itself as a group, is
met: `FAVOURITES` is defined once at `favourites.rs:64` and read by
`folder_tree.rs`, with `group_text` at `folder_tree.rs:450`.

The fourth `[D]` line asked that the pin-versus-subscription question be
decided before the second half is built. It was. `favourites.rs:9` to `:35`
records it in full: a pin says where a folder sits in this program's tree on
this computer, a subscription says which mailboxes an account asks its server to
list, pinning never writes a subscription, and a subscription changing never
adds or removes a pin. `:38` to `:45` records why nothing has to move when the
server half arrives: a pin is stored against `(account_id, path)`, the same pair
`imap::set_subscribed` names a mailbox by.

`set_subscribed` is now at `service/protocols/imap.rs:873`, not line 840.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Built and wired.
    `src/application/favourites.rs` holds `Pin` (line 73), `PinnedBranch` (91),
    `what_each_account_has` (130), `in_account_order` (154) and the four
    announcements at 183, 193, 203 and 208. The menu ids `ID_PIN_FOLDER` and
    `ID_UNPIN_FOLDER` are at `src/presentation/wx_app.rs:91` and 92 with the
    handler at 3613, and the group heading `FAVOURITES` is defined once at
    `favourites.rs:64` and read by `presentation/folder_tree.rs`, whose
    `group_text` is at line 450. The pin-versus-subscription decision this
    requirement asked to have taken in advance was taken in advance and is
    written at `favourites.rs:9` to 45: a pin is local and never writes a
    subscription, a subscription never adds or removes a pin, and a pin is
    stored against `(account_id, path)`, the same pair
    `imap::set_subscribed` (now `src/service/protocols/imap.rs:873`) names a
    mailbox by, so joining them later moves nothing.
```

---

### SEARCH-03: smart folders defined by a rule

**Verdict: wrong.** "Nothing joins the two into a folder that updates itself"
is false. Decision D-2-01 changed what a smart folder is, and that shape is
built and reachable.

**What is true now.** D-2-01, recorded in
`.planning/phases/02-search-that-says-what-it-covers/02-CONTEXT.md:41`, made a
smart folder a saved search with a fuller editor rather than a second object.
That editor exists: `build_rule_edit_dialog` at
`presentation/wx_managers.rs:2770`, `show_rule_edit` at `:2958`, and the list
shell `show_rule_manager_dialog` at `:3136`. It is reached from
`ID_EDIT_SEARCH_CONDITIONS` (`wx_app.rs:221`, item at `5904`, handler at `4683`,
call at `7052`) and from the folder tree's context menu.

There is one matcher underneath both doors, which is the join the evidence says
does not exist: a `Question` becomes a `FilterRule` through `Question::as_a_rule`
(`saved_searches.rs:118`), and `saved_searches.rs:1178` runs it through
`FilterEngine::matches`. Opening a saved search lists what matches now, not a
snapshot, because the questions are evaluated at open time.

`docs/roadmap.md:157` still carries `- [ ] Smart folders based on rules`
unticked, which is the same under-claiming pattern this audit keeps finding.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Built, under a shape decided after this
    requirement was written. D-2-01
    (`.planning/phases/02-search-that-says-what-it-covers/02-CONTEXT.md:41`)
    makes a smart folder a saved search with a fuller editor rather than a
    second object, so the join this evidence said was missing is
    `Question::as_a_rule` (`src/application/saved_searches.rs:118`), which turns
    a saved search's condition into the filter engine's own `FilterRule` and
    runs it through `FilterEngine::matches` at line 1178. One matcher, one
    storage, two doors onto it. The editor is `build_rule_edit_dialog`
    (`src/presentation/wx_managers.rs:2770`), `show_rule_edit` (2958) and
    `show_rule_manager_dialog` (3136), reached from `ID_EDIT_SEARCH_CONDITIONS`
    (`src/presentation/wx_app.rs:221`, item at 5904, handler at 4683, call at
    7052). `docs/roadmap.md:157` still shows this unticked and is stale.
```

---

### THREAD-02: rethread incrementally

**Verdict: wrong.** The Evidence line is false, although the long note appended
under it is accurate and should be kept.

**What is true now.** Threading is no longer done by
`application/threading.rs` on folder open. Each message gets its conversation as
it is stored: `data/message_cache/messages.rs:834` calls
`thread_identity::conversation_root` while writing the row, and `:966` to `:976`
handles the late arrival that connects two trees by calling
`thread_identity::identifiers_worth_asking_about` and then
`thread_identity::rejoin`. `thread_identity.rs:5` records why the module exists:
`messages.thread_id` shipped as a column nothing wrote.
`backfill_thread_ids` at `messages.rs:1010` fills it in for databases written
before that.

`application/threading.rs` still exists and `thread_messages` (`:53`) still runs
in memory for the conversation tree, which `thread_identity.rs:36` states
directly.

The appended note beginning "Closed by 01-13" is accurate and should survive any
rewrite. The one-direction merge, the three of six arrival orders that do not
merge, and the sentence that nothing has been heard by a screen reader are all
still true.

**One thing to fix outside the requirement.** `docs/changelog.md:8394` still
sits in the `[Unreleased]` section saying "Threading runs over the loaded folder
rather than incrementally as mail arrives... rethreading still happens when a
folder is opened rather than as messages arrive". That contradicts
`docs/changelog.md:574` in the same section, which says mail arriving into the
folder you are reading now joins its conversation. Its neighbour at `:8392`
carries a `**Since closed:**` marker; this one does not.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Closed by 01-13, and the mechanism moved.
    Threading is no longer done by `src/application/threading.rs` on folder
    open. A message gets its conversation as it is stored:
    `src/data/message_cache/messages.rs:834` calls
    `thread_identity::conversation_root` while writing the row, and lines 966 to
    976 handle the late message that connects two trees by way of
    `thread_identity::identifiers_worth_asking_about` and
    `thread_identity::rejoin`. `src/application/thread_identity.rs:5` records why
    it exists: `messages.thread_id` shipped as a column nothing wrote.
    `backfill_thread_ids` (`messages.rs:1010`) fills it in for older databases.
    `threading.rs` still runs in memory for the conversation tree, which
    `thread_identity.rs:36` says. Note that `docs/changelog.md:8394` still
    carries the old known limitation under `[Unreleased]` with no
    "Since closed" marker, contradicting line 574 of the same section.
```

---

### SEARCH-02: search over message text that eviction has cleared

**Verdict: wrong.** The disclosure this requirement is mostly about is built and
wired, in both the search box and the saved search.

**What is true now.** `how_much_message_text_the_index_holds` is at
`data/message_cache/searching.rs:609`, backed by a
`text_is_in_the_search_index` column (`searching.rs:149`). The search box asks
it through `presentation/managers.rs:1852` and words the answer with
`what_the_search_box_covers` (`saved_searches.rs:780`) at `managers.rs:1858`. A
saved search asks it through `wx_app.rs:6455` to `6460` and words it with
`what_a_saved_search_covers` (`saved_searches.rs:761`), and
`what_a_search_says_as_it_opens` (`saved_searches.rs:670`) is said on open at
`wx_app.rs:6671`. The question of whether a search needs body text at all is
answered in one place, `reads_the_message_text`, defined for a saved search at
`saved_searches.rs:1104` and for a scope at `searching.rs:85`, so a search about
senders and subjects never pays to ask.

The "offers to fetch the rest" half is built too: `ID_FETCH_MISSING_TEXT` at
`wx_app.rs:156`, item at `5486`, handler at `4220`, with
`data/message_cache/bodies.rs:541` naming the list it fetches.

Both `[D]` lines are satisfied. The `[S]` quote from `docs/changelog.md`
("Nothing built into the program saves a search of that kind yet") is now a
stale sentence in the changelog rather than a description of the code.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. Built and wired on both doors. Bodies are
    still split out with a size budget and least-recently-read eviction in
    `src/data/message_cache/bodies.rs`, and the coverage is now measured and
    said: `how_much_message_text_the_index_holds`
    (`src/data/message_cache/searching.rs:609`) reads a
    `text_is_in_the_search_index` column (line 149). The search box asks it at
    `src/presentation/managers.rs:1852` and words it with
    `what_the_search_box_covers` (`src/application/saved_searches.rs:780`); a
    saved search asks it at `src/presentation/wx_app.rs:6455` and words it with
    `what_a_saved_search_covers` (line 761) and
    `what_a_search_says_as_it_opens` (line 670, said at `wx_app.rs:6671`).
    Whether a search needs body text at all is answered once, by
    `reads_the_message_text` (`saved_searches.rs:1104` and `searching.rs:85`).
    The offer to fetch the rest is `ID_FETCH_MISSING_TEXT`
    (`wx_app.rs:156`, item at 5486, handler at 4220), with the list it fetches
    named at `bodies.rs:541`. The changelog sentence quoted as [S] below is now
    itself stale.
```

---

### PERF-06: every document that quotes a test count quotes the same measurement

**Verdict: wrong.** The three-way disagreement the evidence describes has been
fixed. What is true instead is that the reconciled number has moved on again,
which is exactly what the requirement's third `[D]` line says a check must not
treat as a failure.

**What is true now.** All three documents agree.
`docs/IMPLEMENTATION_STATUS.md:123` reads "5,430 tests pass: 5,269 unit and 161
integration, counted 2026-08-29 with `cargo test --all-targets -- --list`",
and adds at `:124` that the previous 3,362 from 2026-08-09 "is what a number
without its command and its date turns into". `docs/changelog.md:1276` and
`:1280` carry the same figures with the same split, and
`docs/integration-guide.md:5` carries 5,430. So both the first and second `[D]`
lines, the command and date, and the unit-against-integration split, are already
met by all three.

Measured today:

```
cargo test --lib -- --list            counts 6,079      2026-09-04
cargo test --all-targets -- --list    counts 6,264      2026-09-04
```

which makes the integration and other-target figure 185. So the documents are
about 810 unit tests and 24 integration tests behind. Under the rule this
requirement itself sets, that is a stale measurement rather than a defect, and
the answer is a re-measure, not a check.

**One number in the eighteen is stale in a way that is a defect, and it is not
this one.** `docs/changelog.md:8393` describes the per-event feedback grid as
"nine events by four channels". There are sixteen events, not nine
(`presentation/accessibility/feedback.rs:114`,
`pub const ALL: [Event; 16]`). See FEEDBACK-01.

**On durations, since `CLAUDE.md` says a duration is the same kind of claim as a
count.** Two timings inside my eighteen carry no conditions.

- PERF-07's "A whole-tree run is about two days" is repeated at
  `docs/IMPLEMENTATION_STATUS.md:156` and `CLAUDE.md:323` with no machine, no
  date and no thread setting, while `CLAUDE.md:465` gives a different figure,
  "about 15 hours", for the 564-record guard sweep after `WIXEN_TEST_THREADS`
  halved it. Two durations for two different jobs, but a reader planning phase 8
  has no way to tell which conditions either was taken under.
- PERF-02's "under 2 seconds" is a target, not a measurement, and is correctly
  written as one.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. The disagreement is closed and the number
    has moved. All three documents now agree:
    `docs/IMPLEMENTATION_STATUS.md:123` reads "5,430 tests pass: 5,269 unit and
    161 integration, counted 2026-08-29 with
    `cargo test --all-targets -- --list`", and line 124 names the superseded
    3,362 as what a number without its command and its date turns into;
    `docs/changelog.md:1276` and 1280 and `docs/integration-guide.md:5` carry
    the same figures. So the first two criteria below, the command with its date
    and the unit-against-integration split, are already met. Re-measured
    2026-09-04: `cargo test --lib -- --list` counts 6,079 and
    `cargo test --all-targets -- --list` counts 6,264, so the documents are
    about 810 unit tests behind. Under the third criterion below that is a
    stale measurement to refresh, not a check to fail. One number in the
    documentation is stale in a way that is a defect rather than a drift:
    `docs/changelog.md:8393` calls the per-event feedback grid "nine events by
    four channels" when there are sixteen
    (`src/presentation/accessibility/feedback.rs:114`).
```

---

### PERF-05: line coverage re-measured

**Verdict: wrong.** The coverage figure and its date are still right. The
commit count attached to them is out by a factor of about four.

**What is true now.** `docs/IMPLEMENTATION_STATUS.md:201` still records 60.4%
measured 2026-07-26 with `cargo llvm-cov --lib --summary-only`, and says a great
deal has landed since, which is correct. The evidence says "Roughly 275 commits
have landed since". Measured 2026-09-04 with
`git rev-list --count --since="2026-07-26" HEAD`: **1,195**. The repository holds
1,373 commits in total, first commit 2026-02-13, so 87% of this project's
history has landed since the last coverage reading.

The two `[S]` lines and the `[D]` line are unaffected and still correct. The
attribution point stands: `service/protocols`, `service/oauth` and the provider
clients are low because that transport has never met a live account.

**Suggested replacement for the Evidence line:**

```
  - Evidence: 60.4%, measured 2026-07-26 with
    `cargo llvm-cov --lib --summary-only`, stale since.
    `git rev-list --count --since="2026-07-26" HEAD` on 2026-09-04 gives 1,195
    commits landed since, out of 1,373 in the repository, so 87% of this
    project's history postdates the reading. (The "roughly 275" this line used
    to give was itself a stale count when it was written and had never been
    re-taken.)
```

---

### PERF-03: a mailbox of 100,000 messages or more, exercised

**Verdict: wrong.** "The largest thing exercised is a loopback server" is false.
A 200,000 row sample mailbox generator ships in the product, on the Help menu,
put there deliberately so a screen reader user can arrow through one.

**What is true now.** `SAMPLE_MAILBOX_SIZE` is 200,000 at `wx_app.rs:9125`, and
`sample_mailbox` at `:9137` builds the rows. Its doc comment says why it is on a
menu rather than behind a build flag: "the people who most need to test it are
not the people compiling it". `ID_LOAD_SCALE_SAMPLE` is at `wx_app.rs:93`, the
item at `6281`, the handler at `4908`.

So the first `[D]` line is half satisfied. The mechanism for exercising 200,000
synthetic rows exists and is reachable. What is not there is the recorded number
for sort, filter and scroll: nothing in the tree records a timing from a sample
run.

The second `[D]` line is not satisfied and this is the part to keep. The virtual
text callback at `wx_app.rs:1101` reads from `state.messages` in memory and
never queries SQLite, with a comment saying so at `:1093`, but **no test asserts
it**. Nor is the mail-at-scale plan's paged design built: there is no page cache
of 200 rows around the viewport and no placeholder on a cache miss, because the
whole list is held in memory. `message_rows::PLACEHOLDER` exists and is returned
only when the row index is past the end of the loaded list.

The third `[D]` line, that no criterion claims a real provider mailbox was used,
is still correct and still important.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. The design targets 200,000 rows and the
    means to exercise them ships in the product:
    `src/presentation/wx_app.rs:9125` sets `SAMPLE_MAILBOX_SIZE` to 200,000,
    `sample_mailbox` at 9137 builds the rows, and `ID_LOAD_SCALE_SAMPLE`
    (line 93, item at 6281, handler at 4908) is on the Help menu deliberately
    rather than behind a build flag, because the people who most need to test it
    are not the people compiling it. What is missing is the numbers: nothing in
    the tree records a sort, filter or scroll timing from a sample run. The
    mail-at-scale plan's paged design is also not built. The virtual text
    callback at line 1101 reads the whole loaded list out of `state.messages` in
    memory and never touches SQLite, which a comment at 1093 states, but there
    is no page cache of 200 rows around the viewport, no placeholder on a cache
    miss, and no test asserting the callback issues no query.
```

---

### FOLDER-02: nested folder hierarchy

**Verdict: wrong.** "The inventory records the tree as one flat level" is false.
Nesting shipped in phase 1. This is the only one of the eight FOLDER, THREAD and
SEARCH requirements still marked Pending in Traceability, and the code half is
done.

**What is true now.** Nesting is read from `folders.parent_id`, written once at
sync by `mail_sync::store_folders` from the separator the server gave for that
one mailbox. Nothing splits a path at display time, which
`presentation/folder_tree.rs:9` to `:15` states as decision D-22.
`folders_underneath.rs` holds the shared answer to which folders sit under which,
bounded by `AS_DEEP_AS_A_TREE_GOES` at `:45` because a cycle in `parent_id`
written by an earlier version is not hypothetical.

Each of the three `[D]` lines:

- **Level from the native control.** `TreeRow` carries `depth`
  (`folder_tree.rs:214`) and the label deliberately does not
  (`folder_tree.rs:212`, and the reasoning at `:17` to `:23`). That is the
  correct shape. Whether NVDA and Narrator actually say it has not been heard,
  which is what leaves this row open.
- **Collapse and expand by keyboard, remembered across a restart.** Built.
  `set_row_collapsed` at `data/message_cache/folders.rs:275` and
  `collapsed_rows` at `:309` write and read a `tree_state` table, called from
  `wx_app.rs:13732`, `10224` and `14909`.
- **Unread counts on a collapsed parent, saying which number it is giving.**
  Built. `TreeRow` carries both `unread_here` and `unread_in_all`
  (`folder_tree.rs:227` and `:228`), `unread_text` at `:381` takes a `closed`
  flag and an `UnreadOnAParent` setting
  (`application/folder_settings.rs:34`), and `group_text` and
  `branch_text` at `:450` and `:484` word the rows.

**This is the only requirement in the eighteen whose remaining work is a screen
reader run rather than code.** `.planning/STATE.md` already says so: what keeps
FOLDER-02 open is a screen reader announcing a folder's level from the native
control, which no test here can answer.

**Suggested replacement for the Evidence line:**

```
  - Evidence: rewritten 2026-09-04. The code shipped in phase 1; what is left is
    a screen reader run. Nesting is read from `folders.parent_id`, written once
    at sync by `mail_sync::store_folders` from the separator the server gave for
    that one mailbox, and nothing splits a path at display time
    (`src/presentation/folder_tree.rs:9` to 15, decision D-22).
    `src/application/folders_underneath.rs` holds the shared walk, bounded at
    line 45 because a cycle written by an earlier version is not hypothetical.
    `TreeRow` carries `depth` (`folder_tree.rs:214`) and deliberately keeps
    level, expansion and position out of the label (line 210). Collapse survives
    a restart through a `tree_state` table:
    `src/data/message_cache/folders.rs:275` and 309, called from
    `src/presentation/wx_app.rs:13732`, 10224 and 14909. A collapsed parent's
    unread count is `unread_here` against `unread_in_all`
    (`folder_tree.rs:227` and 228), worded by `unread_text` (line 381) under the
    `UnreadOnAParent` setting (`src/application/folder_settings.rs:34`). What
    remains is guardrail 2: no screen reader has confirmed that the level is
    announced from the native `TreeCtrl`.
```

---

### FEEDBACK-02: dates and relative wording in the user's own language

**Verdict: partly stale.** The core claim is still right. Two of its supporting
citations are wrong, and one thing the requirement asks for has already been
done.

**What is still true.** `MONTHS` is still a hardcoded English array, now at
`presentation/date_display.rs:100` rather than 101, formatted from at `:384`,
which is unchanged, and also at `:456` and from
`presentation/wx_item_form.rs:844`. Relative wording is still English,
built by `plural` and tested at `date_display.rs:860` with "2 days ago".

**What is wrong.** The `[S]` citation says `docs/changelog.md` line 6946. That
line is now about filter rules that move a message to a folder. The changelog
line about month names is `docs/changelog.md:8005`.

**What has already been done.** The first `[D]` line asks that `DateOrder`
follow the machine rather than being a separate setting the user has to find.
It already does. `DateOrder::from_system` at `date_display.rs:226` calls
`read_locale(LOCALE_IDATE)` (`:136`) and `order_from_locale` (`:169`), and the
default at `:71` is `DateOrder::from_system()`. The clock follows the same way
through `clock_from_locale` (`:183`). So the remaining work is the strings, and
only the strings.

**What is better than the requirement knows.** The second `[D]` line says a
locale with no translation should fall back to English and say nothing about it
in the UI, "because a visible fallback notice on every row is worse than the
fallback". The code took a different decision and wrote it down:
`date_display.rs:80` to `:92` states the problem plainly, that an English word
in the middle of a French date read with French pronunciation sounds like the
screen reader misbehaving, and `ENGLISH_ONLY` at `:93` says so once, on the
settings screen (`wx_settings.rs:1251`) and in `docs/accessibility.md`, rather
than on every row. That satisfies the spirit of the `[D]` line and guardrail 9,
and the `[D]` line should be reworded to match rather than read as contradicting
what shipped.

**Suggested replacement for the Evidence line and the second `[D]` line:**

```
  - Evidence: rewritten 2026-09-04. Half of this is already done. The order of
    the day and month and the clock already follow the machine:
    `DateOrder::from_system` (`src/presentation/date_display.rs:226`) reads the
    Windows locale through `read_locale` (line 136) and `order_from_locale`
    (169), and is the default at line 71; the clock follows through
    `clock_from_locale` (183). What is left is the strings and only the strings.
    `MONTHS` is a hardcoded English array at line 100, formatted from at 384 and
    456 and offered as a choice at `src/presentation/wx_item_form.rs:844`;
    relative wording such as "2 days ago" is English. The limitation is already
    disclosed in the product rather than left to be discovered:
    `date_display::ENGLISH_ONLY` (line 93) is on the settings screen
    (`src/presentation/wx_settings.rs:1251`) and in `docs/accessibility.md`, and
    the reasoning at lines 80 to 92 says why it matters, that an English month
    name inside a French date read with French pronunciation sounds like the
    screen reader misbehaving.

  - [S] `docs/changelog.md:8005`. (The previous citation, line 6946, is now a
    paragraph about filter rules that move a message to a folder.)

  - [D] A locale with no translation falls back to English. The fallback is said
    once, where somebody can act on it, and not on every row: that is
    `date_display::ENGLISH_ONLY` and it already ships.
```

---

## The five whose evidence is still accurate

### FEEDBACK-01: set feedback channels per event

**Verdict: still accurate, and the point is stronger than the evidence claims.**

The evidence says searching `src/presentation/` for `per_event` or
`set_event_channels` outside `feedback.rs` returns nothing. Confirmed today. It
is stronger than that:

- `set_event_channels` is **private** (`feedback.rs:424`, `fn` not `pub fn`) and
  the `per_event` field is private (`:392`). So no screen could write one
  without changing a visibility.
- The only shipping caller is `from_stored` at `:496`. Every other caller is a
  test. **A per-event override can only enter this program by somebody
  hand-editing the stored config string.**
- The settings screen goes out of its way to preserve overrides it cannot
  create. `wx_settings.rs:2123` says so: "The per-event overrides in the stored
  value are preserved: this tab only decides which channels are on at all", and
  the loop at `:2126` writes only `set_channel_enabled`.
- The reading half is fully live: `channels_for` is called on the shipping path
  at `accessibility.rs:213` (`earcon`) and `:235` (`signal`).

**How many settings the model holds that no screen offers: two, and a third of a
different kind.**

1. Per-event feedback channels, above.
2. The per-account Allow Changes answer. `docs/changelog.md:5296` records it:
   "the setting an account can carry is still read and still honoured, so a
   per-account answer would work if anything wrote one. Nothing does, and no
   screen offers it." This one already has a guard,
   `test_nothing_offers_a_setting_per_account_that_no_screen_writes` at
   `tests/house_style.rs:152`, reading `A_CONTROL_NO_SCREEN_WRITES` at `:122`.
3. Not a setting but the same shape: there is no screen for changing an existing
   task's description (`docs/changelog.md:3303`).

**The event count is stale in the changelog.** `docs/changelog.md:8393` says "a
grid of nine events by four channels". There are sixteen events:
`Event::ALL` is `[Event; 16]` at `feedback.rs:114`, and all sixteen have a
non-test call site. The four newest, added by `a0339e6` with a message saying
they were "not wired to a call site yet", are all wired now: `HasAttachment`
at `wx_app.rs:16684`, `AccountNeedsAttention` at
`wx_account_manager.rs:568`, `Confirmed` through `application/pim_command.rs`,
and `NothingFound` at `managers.rs:1900`.

**The `[D]` correction dated 2026-08-29 is still exactly right.**
`A_CONTROL_NO_SCREEN_WRITES` is still at `tests/house_style.rs:122`, still a
list of phrases a document may not use, and still about the per-account Allow
Changes setting rather than this one. The per-event case still needs its own
check.

**One small correction to the Evidence line:** the grid is sixteen by four, not
nine by four, and the changelog needs the same fix.

```
  - Evidence: re-checked 2026-09-04 and still accurate, with one number to
    correct. `src/presentation/accessibility/feedback.rs` holds the model:
    `per_event` (line 392) and `set_event_channels` (line 424) are both private,
    `channels_for` is at 433, and serialisation is `to_stored` (458) and
    `from_stored` (478). The only shipping caller of `set_event_channels` is
    `from_stored` at line 496, so a per-event override can enter this program
    only by somebody hand-editing the stored config string. The settings screen
    preserves overrides it cannot create and says so:
    `src/presentation/wx_settings.rs:2123`. The reading half is live:
    `channels_for` is called at `src/presentation/accessibility.rs:213` and 235.
    The grid is 16 events by 4 channels, not 9 by 4: `Event::ALL` is
    `[Event; 16]` at `feedback.rs:114`, and `docs/changelog.md:8393` still says
    nine and needs correcting.
```

### FEEDBACK-03: know how much of WCAG the scans cover

**Verdict: still accurate.** Every anchor checks out.
`.github/workflows/accessibility.yml` and `.github/workflows/nvda.yml` both
exist, and so does `scripts/msaa-names.ps1`.
`docs/IMPLEMENTATION_STATUS.md:130` says the scan reported five findings when
last read on 2026-07-26, all inside WebView2's own accessibility tree, and
`:136` says automated scanning covers roughly half of WCAG. None of the three
`[D]` lines has been acted on: nothing in the tree names which WCAG 2.2 AA
criteria the scan can and cannot judge, the five findings have not been
re-read, and there is no written list of interactions a manual pass has to walk.

One thing worth adding rather than correcting: the "last read" date is now
five and a half weeks old, and the workflow is non-blocking
(`docs/IMPLEMENTATION_STATUS.md:112`), so nothing forces a re-read.

### PERF-01: memory under 150 MB with 1,000 cached messages

**Verdict: still accurate.** No `benches/` directory. No `criterion`, no
`divan`, no `[[bench]]` in `Cargo.toml`. Nothing in `src/` reads resident
memory. The target's source is `docs/development/requirements-backlog.md:81`,
"Memory profiling | Target <150MB with 1000 cached messages | Medium", which
matches the `[S]` line.

### PERF-02: cold start under 2 seconds

**Verdict: still accurate.** `docs/roadmap.md:221` still reads
`- [ ] Startup time optimization (<2 seconds)`, unticked, and `:253` repeats it
as a success metric. `docs/development/requirements-backlog.md:82` carries it as
Medium. Nothing in `src/` times process start against a usable list.

### PERF-04: idle memory under 100 MB

**Verdict: still accurate.** `docs/roadmap.md:254` reads
`- Low memory footprint (< 100MB idle)` under success metrics, with no
measurement anywhere.

---

## Two document-level fixes this audit turned up

Both sit outside the eighteen but were found by checking them, and both are the
kind of thing that misleads the next reader.

1. **`docs/changelog.md:8394`**, in `[Unreleased]`, still says threading is not
   incremental. Line 574 of the same section says it is. Its neighbour at 8392
   carries a `**Since closed:**` marker and this one does not.
2. **`docs/changelog.md:8393`** says nine feedback events. There are sixteen.
3. **`docs/roadmap.md:157`** still shows `- [ ] Smart folders based on rules`
   unticked, after D-2-01 redefined and phase 2 built them.

## A suggestion about the Evidence blocks themselves

The document's opening says each Evidence line names what was checked "so a
later reader can re-run the check rather than trust the conclusion". That is the
right instinct and it did not survive five weeks, for a reason worth writing
into the document rather than fixing case by case.

Most of these blocks cite a conclusion decorated with a precise-looking
reference, not a command anybody can run. Three failure modes, all present here:

- **A line number cannot be re-run.** THREAD-01's "line 582 calls
  `item.enable(false)`" was true when written. Today line 582 is something else,
  and a reader who looks cannot tell whether the document is stale or they
  mis-counted. Cite the symbol, which survives the edit that moves the line.
- **A grep goes blind when the vocabulary moves.** FOLDER-01's grep is a real
  command that still returns nothing, because the feature shipped as
  `create_mailbox` and the grep asks about `create_folder`. A nil result reads
  as confirmation. Where the claim is an absence, search the concept's plausible
  spellings and say which ones were searched.
- **A bare assertion of absence names no method.** "no favourites path in
  `src/`", "Nothing joins the two", "has never been done". Re-checking one of
  these means inventing a search and hoping it is the same search.

The rule that would have caught all twelve: **anything a later reader is
expected to re-check gets the literal command, its one-line result, and the date
it was run.** That is the same rule this project already applies to test counts
under PERF-06 and to guard records under `guards/guards.toml`, and it is the
rule the Evidence blocks were reaching for and did not quite state.

---

*Audited 2026-09-04 against `main` at `d3c6c7d`. Commands used for the counts
in this file: `cargo test --lib -- --list`, `cargo test --all-targets -- --list`,
`git rev-list --count --since="2026-07-26" HEAD`,
`grep -c "^\[\[guard\]\]" guards/guards.toml`.*

# Phase 13.3: how the review's 42 items are grouped into plans

Written 2026-10-20 on branch `13.3-plans`, cut from `main` at `6fd465c6`, from
`review-2026-10-20-findings.json` in this directory. That file is a byte-for-byte copy of the
list Pratik was shown, made with Python's `shutil.copyfile` and checked by comparing the SHA-256
of both files (equal, 410,217 bytes each, 2026-10-20). Its key `findings` holds 42 items,
numbered 1 to 42 as he saw them:

- Items 1 to 40 are the review of 13.2's changes, `reviewed_range` `2a4679a9..0b781473`, each
  confirmed by at least two of three checkers: 15 graded must (1 to 15), 16 should (16 to 31)
  and 9 nice (32 to 40).
- Item 41 is ledger 912, the draft copies on a server without UIDPLUS, and item 42 is the check
  suite's slowdown. Both are graded should and carry no checker notes.

Pratik answered "yes" on 2026-10-20 to fixing all 42 as one phase, before the build he installs
and before `main` is pushed.

**Read the checker notes, not only the `fix` field.** Several notes correct the item's own fix,
and some say the proposed fix would break a test or not work at all. A plan works from the
corrected fix. The section "Where the checkers corrected the fix" below lists what this reading
found, and it is a summary: each planner reads the notes for their items in full.

**Take every line again.** The line numbers in the file are at `0b781473`. Run on 2026-10-20,
`git diff --stat 0b781473..6fd465c6 -- src scripts tests installer search-handler` names two
files, `src/application/draft_copy.rs` and `src/application/mail_controller.rs`, both changed by
14-08.1. Lines cited in every other source file therefore still held at `6fd465c6` that day.
A plan citing either of those two files, 13.3-16 and 13.3-21 above all, re-takes its lines
before anything else; every other plan re-takes them too, because `main` moves under the phase.

## The plans

One plan per wave, chained: plan 13.3-NN is wave NN and depends on 13.3-(NN-1). Every plan
writes `.planning/ROADMAP.md`, `.planning/STATE.md` and `.planning/REQUIREMENTS.md`, so no two
can share a wave.

The order is the gate first, because a slow gate slows every later commit; then the plans that
carry a must item, then should, then nice; then the update mechanism; then the closing plan. A
plan carrying items of several grades sits at the place of its highest grade.

"Spoken or shown" is yes when a sentence, a dialog, or what a window lists changes. Such a plan
pushes its branch and opens a pull request so the NVDA job runs before the merge, under
Pratik's standing OK; the others push nothing. Size is the task count the plan should need: S
is 2 tasks, M is 3, L is 4.

| Plan | Wave | Title | Items | Main files | Spoken or shown | Size |
|------|------|-------|-------|------------|-----------------|------|
| 13.3-01 | 1 | The check suite is fast again, a scoped run reads the guard registry once and runs its library filters in one cargo call, and the registry reading is checked against a TOML reader | 42, 30, 35, 36 | scripts/check.test.sh; scripts/check.sh (`read_the_records_in`, `the_scoped_runs`, the covered loop, `run_the_tests_that_reach_what_changed`, the `--suites-for` entry); CLAUDE.md (the check suite's figures, and the paragraph saying several module paths need several runs); docs/development/measurements.md | No | L |
| 13.3-02 | 2 | A delete or move made on this computer leaves the list and the conversation rows at once, and a reopened conversation no longer lists it | 1, 15 | src/presentation/wx_app.rs (`delete_these`, `delete_if_local`, the `MessageLeftTheFolder` arm, `the_set_leaving` and `ASetLeaving`, the single-message path, `bring_the_conversation_rows_up_to_date`); src/presentation/ui_types.rs (the `MessageLeftTheFolder` variant); src/data/message_cache/messages.rs (`conversations_holding`, asked before the change); tests/the_folder_tree_counts_follow_a_change_made_here.rs; tests/a_change_made_here_reaches_the_conversation_rows.rs; tests/wired.rs; tests/each_set_command_has_a_quiet_do_half.rs; docs/changelog.md (the LIST-05 entry's promise) | Yes: what the list and the conversation window show after a delete or move | L |
| 13.3-03 | 3 | Folder counts follow a change made here whichever account drew the tree, and a check keeps the count of a mark still waiting | 2, 7 | src/presentation/folder_tree.rs (`rows_that_read_differently`); src/presentation/wx_app.rs (`say_the_folder_counts_again`); src/data/message_cache/folders.rs (`set_folder_counts`, joined to `waiting_flag_changes`); tests/the_folder_tree_counts_follow_a_change_made_here.rs; docs/changelog.md | Yes: the unread counts in the folder tree | M |
| 13.3-04 | 4 | A read or star change goes to the server of the account the message is in, and a change let go during a check's replay is not sent | 3, 18 | src/presentation/wx_app.rs (`spawn_server_change`, `owner_of`, `the_account_a_row_is_in`, `send_the_flag_changes_that_were_waiting`); src/application/destinations.rs (`whose_folders_a_move_is_about`); src/data/message_cache/parts_left_behind.rs (`the_account_a_message_is_in`, reused); src/data/message_cache/waiting_flag_changes.rs; src/application/mail_session.rs (if the account's lock lives there); tests/wired.rs | Yes: a refused change is put back and said | L |
| 13.3-05 | 5 | Reply and Forward from a conversation say when the message's text is not here yet, and Forward stops working out a reply address nobody reads | 4, 37 | src/presentation/wx_app.rs (`the_body_a_command_quotes`, `msg_info`, the `ID_FORWARD` arm, `spawn_body_fetch`); src/presentation/ui_types.rs (`reply_address` and its three asserts, once nothing else calls it); docs/changelog.md | Yes: a sentence where compose opened with an empty quote | M |
| 13.3-06 | 6 | The download's re-read of the open folder is applied in the tick that reads it, so it cannot undo a mark or delete | 5 | src/presentation/wx_app.rs (the timer closure: the `due` block and the update loop); tests/everything_comes_down_without_being_asked.rs (`what_is_wrong_with_the_timers_reread`); guards/guards.toml (the record anchored on `.the_folder_due(std::time::Instant::now());`) | Yes: what the list shows after M or Delete during a download | S |
| 13.3-07 | 7 | Removing an account and adding one back on the same address saves | 6 | src/data/message_cache/accounts.rs (`replace_accounts`, `refuse_an_address_another_account_uses`, the order against `credentials::store`) | No: the store's order of writes; the refusal that stops is a sentence that already exists | S |
| 13.3-08 | 8 | The download of everything leaves new mail for the check so rules run on it, and a filed message with no Message-ID never takes another message's number | 8, 9 | src/application/mail_sync.rs (`sync_folder`'s match on `wanted`, `uids_to_fetch`, `what_to_ask_for`); src/application/moves_waiting.rs (`settle_a_moved_row`, `where_it_is_now`, `settle_the_row`); src/application/renumbered.rs (`the_number_confirmed`, read) | Yes: the new-mail count and what the folder lists | M |
| 13.3-09 | 9 | A feed that fails to arrive names only its host, and a feed that starts with a byte-order mark refreshes again | 10, 11 | src/service/ical_subscription.rs (`why_the_feed_did_not_arrive`, `fetch_ics`, `parse_ics`); src/application/opening.rs (`text_of`, the same hole for an opened .ics file); Cargo.toml only on Pratik's word, see "Decisions that wait on Pratik" | Yes: the feed's failure sentence | M |
| 13.3-10 | 10 | A cut-short CardDAV answer removes nobody, and a kept contact whose card is gone stops failing on every sync | 12, 24 | src/service/carddav.rs (`cards_in`); src/application/carddav_sync.rs (the removal guard); src/application/contacts_sync.rs (`take_off_who_a_whole_answer_left_out`, `one_address_book_deleted_them`) | No | M |
| 13.3-11 | 11 | A note edited, pinned or moved while a notes read runs is kept | 13 | src/application/notes_sync.rs (`take_what_has_arrived`, `write_it_down`, the test service's `what_a_note_says`); src/data/message_cache/notes.rs (a guarded write, if the plan uses one); docs/changelog.md (the line saying a note edited while it syncs is kept) | No | M |
| 13.3-12 | 12 | An import says which files in an archive it left out at the first look, and a damaged compressed entry gets the damaged sentence | 14, 26 | src/application/importing_messages.rs (`an_archive_brought_in`); src/service/mailbox_archive.rs (the count getters, `WhyAnEntryWasNotRead`); src/application/import_tree.rs (`FoldersImported` and its sentences); docs/changelog.md | Yes: the import's closing sentence | M |
| 13.3-13 | 13 | A message read in a conversation opened from the message view turns read | 16 | src/presentation/wx_app.rs (`what_reading_may_mark`, `open_conversation_again`'s Message arm, `reading_began`); src/application/reading_habits.rs (`whether_to_mark_read`, read); tests/the_conversation_window_acts_on_the_message_under_its_cursor.rs (`test_under_the_message_view_reading_may_mark_the_selected_row_alone`); docs/changelog.md (13.2-03.1's sentence) | Yes: the read state shown | M |
| 13.3-14 | 14 | A change over many conversations, and landing on one, cost what they touch rather than the whole list or the whole account | 17, 32, 33 | src/presentation/view_state.rs (`bring_rows_up_to_date`); src/presentation/wx_app.rs (`the_set_leaving`); src/data/message_cache/bodies.rs (`text_missing_in_a_conversation`); src/data/message_cache/messages.rs (`the_scope_over` and `as_bound_pairs` reachable from bodies.rs); docs/development/measurements.md | No | M |
| 13.3-15 | 15 | The Google sign-in check compares the account that signs in, and learns it from a permission the consent screen cannot untick | 19, 20 | src/service/oauth.rs (`authorize`, `refuse_anybody_but`, the Google scope lists); src/service/google_api.rs (`the_address_signed_in`); src/presentation/wx_account_manager.rs (both `authorize` calls); src/application/who_holds_the_calendars.rs (the address the sign-in names); docs/PROVIDER_SETUP.md; docs/changelog.md | Yes: the address the sign-in tells the person to choose | M |
| 13.3-16 | 16 | A rule's moves and a replay's changes open each folder once | 21, 34 | src/application/mail_sync.rs (`carry_out_the_moves`); src/application/moves_waiting.rs (`settle_a_moved_row`'s callers, `replay_one`, `numbering_of`); src/application/mail_controller.rs (`numbering_of`, `select_folder`); src/service/protocols/imap.rs (a numbering kept beside `selected`, if that fix is chosen); src/application/emptying_the_trash.rs (its stand-in `numbering_of`) | No | M |
| 13.3-17 | 17 | A day deleted in one calendar leaves the same meeting in another alone, and the one-calendar lookup takes the shape its siblings use | 22, 40 | src/application/calendar.rs (`events_deleted_here`); src/application/caldav_sync.rs (the fold, `sync_caldav_calendar`, `refresh_subscription`); src/data/message_cache/calendar.rs (`deleted_calendar_events`, `event_in_one_calendar`); docs/changelog.md (13.2-31.6's promise) | No | M |
| 13.3-18 | 18 | A contact created at Google or Outlook keeps an edit or a delete made meanwhile, and its name stays out of the log | 23, 25 | src/application/contacts_sync.rs (both create blocks, `write_down`); src/application/carddav_sync.rs (the made-here PUT); src/data/message_cache/contacts.rs (read); src/presentation/managers.rs (only if the plan takes the Contact Manager half, see the corrections); docs/changelog.md (the contacts line saying an edit made while the sync runs is kept) | No | M |
| 13.3-19 | 19 | Markup nested past 512 levels has its hidden blocks dropped like any other | 27 | src/application/hidden_text.rs (`Findings::walk`, the depth bypass, its comment and its test); docs/changelog.md | Yes: what the formatted view shows and reads | M |
| 13.3-20 | 20 | The uninstall message and the Windows Search guide say what an uninstall really leaves | 28, 29 | installer/Wixen-Mail-Setup.iss (the message shown when wixen-mail.exe is gone, and the 13.2-27 comments beside `[UninstallDelete]`); src/application/forget.rs (`WHAT_AN_UNINSTALL_SAYS_WHEN_IT_COULD_NOT_ERASE` and its comment); search-handler/README.md (the remove-scope sentence); docs/changelog.md | Yes: the uninstall message | S |
| 13.3-21 | 21 | On a server without UIDPLUS a draft's older copy is mentioned once per draft, copies already flagged are left out of the search, and a sent draft's copy here goes in one statement | 41, 38 | src/application/draft_copy.rs; src/service/protocols/imap.rs (`uids_with_message_id`); src/application/mail_controller.rs (its tests that script the search line); src/presentation/wx_app.rs (`spawn_draft_append`); src/data/message_cache/drafts.rs (`forget_the_filed_draft`) | Yes: the sentence said at an automatic save | M |
| 13.3-22 | 22 | Two functions only tests call are compiled for tests only | 31, 39 | src/data/message_cache/tags.rs (`match_labels_to_keywords`, its rules moved onto `match_these_labels_to_keywords`); src/service/protocols/imap/special_use.rs (`classify`) | No | S |
| 13.3-23 | 23 | Pratik's word on ship-shape at a blocking checkpoint (the research recommends keeping this program's update code and changing no package); then the update check downloads only what it could run, says what a check at start found without opening a window, runs one check at a time with a folder of its own per download, and gives up on a stalled connection | none of the 42; the research's gaps 1, 3 and 4 | src/service/update_download.rs (`whether_an_installer_could_be_accepted`, `what_to_do_about`, `bring_it_down`); src/service/update_check.rs (the one-check guard, `ask`'s limit); src/presentation/wx_app.rs (`ask_whether_there_is_a_newer_version`, the `AnUpdateDidNotHappen` arm, `open_help`'s doc); src/presentation/ui_types.rs (`AnUpdateDidNotHappen`); docs/privacy.md; docs/installing.md; docs/ALPHA_TESTING.md | Yes: what a check at start says, and the sentence for a second check | L |
| 13.3-24 | 24 | Installing an update quits the way Quit does, and what the update code says is true: the setting names the installer's real size, and the stale comments and pages are corrected | none of the 42; the research's gaps 2 and 5 | src/presentation/wx_app.rs (the Quit arm, `UpdateTargets`, the `AnUpdateIsReady` arm); src/application/closing.rs (the reading); src/service/update_check.rs (`WHICH_UPDATES_DESCRIPTION`, the module doc); docs/privacy.md (the measured size); docs/BETA_RELEASE.md; comments in update_download.rs, config.rs, outward.rs and main.rs | Yes: the setting's description | M |
| 13.3-25 | 25 | The full gate once, by hand, and the phase closed | none | `scripts/check.sh all` run by hand once, its output written to a file and its exit status read directly; .planning/ROADMAP.md, .planning/STATE.md and .planning/REQUIREMENTS.md for the phase's marks | No | M |

Every plan also writes the files written by rule where it applies: `docs/changelog.md` for a
user-visible change, `guards/guards.toml` for a guard written, re-measured or retired,
`.planning/WINDOWS.md` for a ledger row opened or closed, and
`docs/development/measurements.md` where it takes a figure. Those are not repeated in every row
above. Guard records are listed by both readings in each plan: the records naming a changed
file in `tests_last_seen` or `file`, and the records whose `before` or `after` text lies inside
a file the plan edits.

The closing plan, 13.3-25, carries the task CLAUDE.md requires of a phase's closing plan: run
`scripts/check.sh all` by hand once, on its branch, after its last commit and before its merge,
with `main` unmoved since the branch was cut. A red run is fixed on that branch as a small red
and green pair, and the gate is run again before the merge. It then records in `STATE.md` what
happens next, under "After the phase" below.

## The chunks

Five chunks of consecutive plans, one planner each, sized by the task counts above (S is 2, M
is 3, L is 4).

| Chunk | Plans | Tasks |
|-------|-------|-------|
| A. The gate's time, and what a change made here shows | 13.3-01 to 13.3-04 | 15 |
| B. Quoting, the open folder, accounts, the check's rules and feeds | 13.3-05 to 13.3-09 | 13 |
| C. Contacts and notes read, the archive, and the conversation view | 13.3-10 to 13.3-14 | 15 |
| D. Sign-in, moves, calendars and contacts made here | 13.3-15 to 13.3-18 | 12 |
| E. Hidden text, uninstall, drafts and code only tests call | 13.3-19 to 13.3-22 | 10 |

The two update-mechanism plans and the closing plan are in no chunk. 13.3-23 and 13.3-24 are
written by another planner after the research. 13.3-25 is written last, because it closes over
every plan in the phase, the update plans included.

## Every item, once

| # | Item | Grade | Plan |
|---|------|-------|------|
| 1 | A deleted POP message comes back in the reopened conversation | must | 13.3-02 |
| 2 | Folder counts stop updating when the tree was drawn for another account | must | 13.3-03 |
| 3 | A mark on a message the list has not loaded goes to the open account's server | must | 13.3-04 |
| 4 | Reply or Forward from a conversation quotes nothing when the text is not downloaded | must | 13.3-05 |
| 5 | A download re-read can undo a mark or delete made a moment before | must | 13.3-06 |
| 6 | Removing an account and adding it back fails | must | 13.3-07 |
| 7 | Each check puts back the unread count of a mark that is waiting | must | 13.3-03 |
| 8 | A filed message with no Message-ID can take another message's number | must | 13.3-08 |
| 9 | The download of everything takes new mail before the check, so rules never run on it | must | 13.3-08 |
| 10 | A feed's secret address goes into the log when a fetch fails | must | 13.3-09 |
| 11 | Feeds that start with a byte-order mark stop refreshing | must | 13.3-09 |
| 12 | A cut-short CardDAV answer deletes contacts here | must | 13.3-10 |
| 13 | A note edited or moved during a notes read is overwritten | must | 13.3-11 |
| 14 | Mail left out when an archive is first looked over is never mentioned | must | 13.3-12 |
| 15 | Conversation row stays after some deletes and moves | must | 13.3-02 |
| 16 | In the message view, a message read from an opened conversation never turns read | should | 13.3-13 |
| 17 | Marking many conversations scans every row against every touched one | should | 13.3-14 |
| 18 | A mark let go during a check's replay is still sent | should | 13.3-04 |
| 19 | The Google sign-in check compares the From address, not the account that signs in | should | 13.3-15 |
| 20 | Unticking Calendar in Google's consent screen now refuses the mail sign-in | should | 13.3-15 |
| 21 | A rule's move now costs four server round trips instead of one | should | 13.3-16 |
| 22 | A day deleted in one calendar changes the same meeting in another | should | 13.3-17 |
| 23 | A contact edited or deleted while Google creates it is lost or comes back | should | 13.3-18 |
| 24 | A kept contact whose card is gone fails on every sync | should | 13.3-10 |
| 25 | Contact names go into the log on every sync | should | 13.3-18 |
| 26 | A damaged compressed zip still gets the old 'stops partway through' sentence | should | 13.3-12 |
| 27 | Markup nested past 512 levels shows and reads text the sender hid | should | 13.3-19 |
| 28 | Uninstall says the mail folder stays, then deletes it | should | 13.3-20 |
| 29 | Search guide still says uninstall runs as the person | should | 13.3-20 |
| 30 | Guard list reader has no check against real records | should | 13.3-01 |
| 31 | A label-matching function now only tests call | should | 13.3-22 |
| 32 | Delete over conversation rows scans the whole list per message for nothing | nice | 13.3-14 |
| 33 | Landing on a conversation still reads the whole account | nice | 13.3-14 |
| 34 | Every replayed change opens its folder again just to read the numbering | nice | 13.3-16 |
| 35 | Each guard module costs its own cargo run | nice | 13.3-01 |
| 36 | Guard registry is read twice per scoped run | nice | 13.3-01 |
| 37 | msg_info works out a reply address nobody uses | nice | 13.3-05 |
| 38 | Removing a sent draft's copy takes two statements | nice | 13.3-21 |
| 39 | special_use::classify is now used only by tests | nice | 13.3-22 |
| 40 | The new one-calendar lookup repeats a four-time pattern | nice | 13.3-17 |
| 41 | Ledger 912: on a server without UIDPLUS each automatic save says an older copy may still be there, every minute | should | 13.3-21 |
| 42 | The shell check suite became slow | should | 13.3-01 |

Counted: 42 rows, each item in exactly one plan, and every plan from 13.3-01 to 13.3-22 named
at least once.

## Where the checkers corrected the fix

Read from the `checker_notes` of each item on 2026-10-20. This is a digest, marked derived:
each planner reads the notes for their items in full, and where a note and this list disagree,
the note wins. Items not listed had notes that agreed with the `fix` field.

- **1.** Deleting the `MessageLeftTheFolder` arm reddens
  `tests/the_folder_tree_counts_follow_a_change_made_here.rs`, whose `THE_PLACES_THAT_CHANGE_A_COUNT`
  names the arm's first line; it moves to `delete_these` in the same commit. `tests/wired.rs`
  and `tests/each_set_command_has_a_quiet_do_half.rs` name `delete_if_local`. The notes
  disagree on whether a guard record anchors on the arm, so the plan reads the register both
  ways.
- **2.** Never replace `tree_rows` with the fresh reading: rows are matched to tree items by
  position. Pair by identity, compare the item count with the held rows, and change held labels
  in place.
- **3.** `send_the_marks_on_their_own` is not an example of the fix; it reads the account only
  to name labels and still sends through `spawn_server_change`. Look the account up with
  `the_account_a_message_is_in` inside `spawn_server_change`'s worker and in
  `the_account_a_row_is_in`, once, rather than in each caller. `tests/wired.rs` checks that
  `spawn_server_change` still calls `owner_of(`.
- **4.** The claim that the store "usually" holds no text is overstated; the case needs a
  first download still running, Pause Downloading, a text limit or a fresh arrival. The notes
  suggest starting `spawn_body_fetch` while refusing, and ask whether Reply should be refused
  at all when it needs no quote.
- **5.** Moving the `due` block reddens `what_is_wrong_with_the_timers_reread` and moves the
  guard record anchored on `.the_folder_due(std::time::Instant::now());`. A smaller fix drains
  the channel a second time after `reread_folder_if_open`. A second Delete on a row that came
  back can become a permanent delete, which is why this is must.
- **6.** The proposed fix does not work alone: `email` is `UNIQUE` in the accounts table and
  `ON CONFLICT(id)` does not cover it, and `credentials::store` runs before the insert. Delete a
  removed account whose address the new list reuses before the saves, and do not end those
  early removals with a bare `?`.
- **8.** `where_it_is_now` needs the same blank guard, and `settle_the_row` (replayed moves)
  shares the function.
- **11.** The checkers prefer trimming a leading U+FEFF where `parse_ics` looks for
  `BEGIN:VCALENDAR`, which needs no package and also covers an opened .ics file through
  `opening::text_of`. The `encoding_rs` route needs a line in `Cargo.toml`.
- **12.** Counting every block with no card would also count the address book's own block on
  some servers. That fails safe (no removals, one standing line), and a narrower check for a
  507 status is offered.
- **13.** The proposed fix is wrong for a move or a delete: a row gone on the second read
  becomes `None`, and `write_it_down` makes a new note. Skip the write when the row was there
  and is gone, or read and write in one immediate transaction as `a_backend_took_the_note`
  does. Give the test service a `meanwhile` hook in `what_a_note_says`.
- **14.** An entry dropped at the look-over can be a single .eml or .msg file, so the sentence
  is about files, not mailbox files; `importing_an_outlook_data_file.rs` has one to copy. The
  did-not-match count is a subset of the could-not-be-read count and is subtracted, so no entry
  is said twice.
- **15.** Work out the conversations before the change for one message too, not only for a
  set. Test a POP Inbox delete, a take-off, and a move to another account.
- **16.** The proposed fix is wrong: it reddens the pinned test and brings back LIST-03's
  problem in the flat view. The corrected fix lets the reading remember that it began in a
  conversation window, set only in `open_conversation_again`'s Message arm. One checker refuted
  the item as designed behaviour and offered a changelog correction instead; Pratik's yes of
  2026-10-20 is to the fix, so the plan builds the corrected fix and says why in its decisions.
- **17.** `mark_these_read` also scans `s.messages` per message and is older than 13.2; this
  item is only the new cost.
- **18.** Skipping a gone row only narrows the race. The complete fix is one lock per account,
  held by the replay for its whole loop and by the direct flag push around its STORE and the
  delete that follows it.
- **19.** `who_holds_the_calendars.rs` names `account.email` twice as the address to choose
  in the browser; both move to the same field. The username fallback is not needed, because
  `validate()` refuses an empty username.
- **20.** Add the `email` scope to both Google sign-ins and read the address from the userinfo
  endpoint; a Gmail-profile fallback would help only the mail sign-in.
- **21.** One checker graded this nice. Settling after the whole loop stretches the time a
  just-filed row can be moved by the person and then put back by the settle; group by
  destination and finish one group at a time.
- **22.** One checker refuted it, pointing at open ledger 825. Narrow the set to notes whose
  `calendar_id` is the calendar being read, and keep a note with no `calendar_id` counting as a
  match. Ledger 825 is widened to feeds and server calendars and closed.
- **23.** When the contact changed during the call, the new identity also needs its change
  waiting set. The Contact Manager saves from the copy it opened, which is a second half of the
  same race; the plan says whether it takes that half.
- **24.** `no_longer_in` clears `pending`, and Google's create pass skips a contact whose
  source is Google, so she would never be sent again. Drop her through
  `one_address_book_deleted_them`, which already reports a change that went with her.
- **25.** A note names a different defect beside it, the gate refusal going into `errors`
  rather than `waiting_on_the_setting`; it is not one of the 42.
- **27.** Only the formatted view and `long_text`'s reader call the drop, not both words-alone
  readers. An iterative walk that skips a subtree with a counter drops the bypass without
  bringing back audit #17's stack overflow.
- **28.** Inno fills `[UninstallDelete]` names in at install time, so the folder removed is
  that of the account that installed, not the one uninstalling. Hedge the sentence ("may still
  be on this computer") rather than naming an account, and correct the 13.2-27 comments that
  make the same claim.
- **29.** The product does warn in this case (`WhatWindowsSearchMayStillHave`); only the
  shipped guide is wrong.
- **30.** A Rust test in `tests/house_style.rs`, which already parses the registry, is offered
  as an equal alternative to a `check.test.sh` case with Python.
- **31.** Put `#[cfg(test)]` on the wrapper where it is, as `contacts.rs`, `in_the_trash.rs`
  and `tasks.rs` already do, rather than moving it.
- **34.** A map kept for one replay misses the Trash emptying, which calls `replay_one` once
  per message. Keeping the numbering of the last SELECT beside the session's selected folder
  covers every caller.
- **35.** The `check.test.sh` cases that pin one `--lib` line per module change with it.
  Tests from several modules then share one process; the full suite already does.
- **36.** The `--suites-for` entry must then read the registry itself, and the `-r` guard on
  the registry stays. The note also measured that `check.test.sh` calls `--scoped-runs-for`
  against the real registry about 12 times, each reading it twice, which bears on item 42.
- **37.** `msg_info` is the only production caller of `MessageItem::reply_address`, so that
  function and its three asserts go too.
- **38.** One checker refuted the bug half; the one-statement change stands as a tidy-up.
- **39.** Moving `classify` into the test module breaks the test at `imap.rs` that calls it
  from another module; put `#[cfg(test)]` on it where it is.
- **40.** One checker refuted it (all four copies are correct today); the other two stand it
  as a tidy-up.

## What a plan must read from a sibling

- **13.3-01** changes `scripts/check.sh` and `scripts/check.test.sh`, which every later plan's
  commits run. Its summary's figures for the suite and the scoped run are what later executors
  should expect. For item 42 its first task is a measurement, not a guess. The commits that
  touched either file since the 2026-09-23 figure, from `git log --since=2026-09-23 --
  scripts/check.test.sh scripts/check.sh` run on 2026-10-20, are 13.1-02 (2026-10-03), 14-06
  (2026-10-06) and 13.2-25, 13.2-26 and 13.2-26.1 (2026-10-18). Item 36's note is a lead, not a
  finding.
- **13.3-02** gives `ASetLeaving` the conversations named before the change and removes
  `MessageLeftTheFolder`. 13.3-03 then edits the same count test, and 13.3-14 edits
  `the_set_leaving` and the `bring_rows_up_to_date` that 13.3-02's catch-up feeds.
- **13.3-04** changes `spawn_server_change` and the flag replay. 13.3-13 works on the reading
  mark, which reaches the server through `spawn_server_change`.
- **13.3-08** adds a blank-identifier guard in `settle_a_moved_row`. 13.3-16 regroups the
  settles in `carry_out_the_moves` and changes how a numbering is read. 13.3-21 adds UNDELETED
  to `uids_with_message_id`, which the settle reaches through `MailController`, so 13.3-21
  checks the settle still finds a message just moved.
- **13.3-09** changes `parse_ics`; 13.3-17's feed tests parse feeds through it.
- **13.3-10** and **13.3-18** both edit `src/application/contacts_sync.rs` and
  `src/application/carddav_sync.rs`; 13.3-18 reads 13.3-10's removal path.
- **13.3-16**, **13.3-21** and **13.3-22** all edit `src/service/protocols/imap.rs` or
  `src/application/mail_controller.rs`, in that order, and each moves the guard anchors the one
  before it moved.
- Plans that edit `src/presentation/wx_app.rs`: 13.3-02, 03, 04, 05, 06, 13, 14 and 21. Each
  names the earlier ones' changes in its premises.

## Ledger rows the checker notes name

Named by the notes, not checked by this reading against `.planning/WINDOWS.md`; each plan reads
the row before acting on it.

| Row | Plan | What happens |
|-----|------|--------------|
| 912 | 13.3-21 | closed: the sentence once per draft, flagged copies out of the search |
| 825 | 13.3-17 | widened to feeds and server calendars, then closed |
| 904 | 13.3-17 | its text says the fold's cross-calendar reach is fixed; corrected |
| 859 | 13.3-19 | its claim that the formatted view still hides what the sender hid is corrected, and the row closed if the walk fixes it |
| 906, 908 | 13.3-12 | 906 covers only entries recorded at 64 KB or more; 908 misnames the commonest damage; each narrowed or closed by what the plan fixes |
| 869 | 13.3-15 | accepted a refusal for a Workspace alias; revisited once the check compares the login |
| 892 | 13.3-01 | records the one hand comparison of the registry reading; the standing check replaces it |
| 836, 899, 839, 841, 563, 179, 255, 850, 865 to 868, 896 | the plan for the same subject | read for context; the notes say each is a different problem or does not cover the item |

New rows are numbered by the executor at commit time, after 912 and after anything a running
plan has taken, never by a planner in advance.

## Requirement candidates

Each planner reads the requirement's text before naming it, and names none with a reason where
nothing fits. REAL-01 and REAL-02 are phase 14's, proven by sittings, and are not ticked here.

| Plan | Candidates |
|------|-----------|
| 13.3-01, 13.3-25 | FOUND-23 |
| 13.3-02 | LIST-21, LIST-05, LIST-12 |
| 13.3-03 | LIST-21; 13.2-31.2's counts carried none |
| 13.3-04 | LIST-04, LIST-21; SCALE-06 is open, named only if the fix completes it |
| 13.3-05 | LIST-05 |
| 13.3-06 | MAIL-01, LIST-21 |
| 13.3-08 | MAIL-01, MAIL-05, as 13.2-08 named |
| 13.3-10, 13.3-18 | PIM-05 for CardDAV |
| 13.3-11 | PIM-04 and PIM-07 are open, named only if the fix completes them |
| 13.3-12 | FOUND-11, GAP-13 |
| 13.3-13 | LIST-03 |
| 13.3-14 | THREAD-01 |
| 13.3-17 | PIM-03 and PIM-06 are open, named only if the fix completes them |
| 13.3-19 | LIST-25 |
| 13.3-21 | FOUND-04, which 14-08.1 named for ledger 902 |
| 13.3-23, 13.3-24 | SHIP-02 |
| the rest | none found by this reading, or the one the planner finds |

## Decisions that wait on Pratik

- **13.3-09, item 11.** Making `encoding_rs` a direct dependency is a package change and needs
  a blocking checkpoint for his word. The checkers' smaller fix needs no package. For the
  windows-1252 half, the plan first reads whether `mail-parser` 0.11, already a direct
  dependency, exposes its charset decoder; only if nothing in the tree can decode does it ask.
- **13.3-15, item 20.** Adding the `email` scope may need the scope listed on the consent
  screen of the Google key Pratik holds for testing. If it does, that is his action in his
  Google console, gated as a checkpoint, not a setting the plan invents.
- **13.3-23 and 13.3-24.** Every package change, ship-shape or a wxdragon move, waits on his
  word at a blocking checkpoint, after a recommendation in plain language.
- **13.3-13, item 16.** His yes of 2026-10-20 is to the fix. One checker called the flat
  view's behaviour a design choice; the plan builds the corrected fix and its decisions block
  says what he can overrule.

## The update mechanism

Pratik asked on 2026-10-20 to consider ship-shape for the update mechanism, and said of it
that day, "It's a published crate." The brief describes it as published on crates.io as
`ship-shape` 0.4.1, MIT, "Universal auto-updater for wxDragon desktop apps", needing wxdragon
`^0.9.22`. This reading did not check crates.io; the research does, under the dependency audit
and the package legitimacy gate. `Cargo.toml` pins `wxdragon = { version = "=0.9.17", ... }`,
read 2026-10-20, so adopting ship-shape also moves the toolkit every window is built on, and the
research prices that move as well as the crate.

What exists today is `src/service/update_check.rs` and `src/service/update_download.rs`, whose
signer check 13.2-16 hardened. The research compares the two on what this project needs: the
signer check, what is spoken and shown while an update arrives, and what the alpha's testers
meet. The ids 13.3-23 and 13.3-24 are reserved. If the research needs more plans, they go in
as 13.3-24.1 and onward, before the closing plan, so its id stays 13.3-25.

## Where the order moved from the brief

- Items 30, 35 and 36 moved up from the should and nice groups into 13.3-01 beside item 42.
  All four work in `scripts/check.sh`'s scoped run and its suite: 35 and 36 share
  `the_scoped_runs`, 36's note counts about 12 calls on the real registry per suite run, each
  reading it twice, and 30's comparison with a TOML reader is the check that keeps a faster
  `read_the_records_in` honest, so it lands before the reader changes.
- Item 18 joined item 3 in 13.3-04 rather than item 34: the complete fix for 18 takes a lock
  in `spawn_server_change`, the function item 3 changes.
- Items 21 and 34 share 13.3-16 because both count SELECTs in the move code. Item 8 went with
  item 9 in 13.3-08, the two must corrections to 13.2-08, rather than with 21, which the brief
  paired with 9.
- Items 32 and 33 went with item 17 in 13.3-14, the conversation view's cost at size, rather
  than into a lean plan. Item 37 went with item 4 in 13.3-05, since both are `msg_info`'s
  path, and item 40 with item 22 in 13.3-17, since both are in
  `src/data/message_cache/calendar.rs`.
- Item 38 went with item 41 in 13.3-21, both about a sent or saved draft's copies, leaving
  13.3-22 with the two functions only tests call.

## After the phase

When 13.3 closes, 14-08.2 builds again at 13.3's last merge, and the install page on branch
`14-08.2-the-new-build` (at `36baca49`, not merged into `main`, checked 2026-10-20 with
`git merge-base --is-ancestor`) is redone for that build. Build 1403 on that branch was never
installed and is superseded. `main` was 332 commits ahead of `origin/main` on 2026-10-20
(`git rev-list --count origin/main..main`); pushing it is Pratik's word.

## What this setup did not do

- It did not insert phase 13.3 into `.planning/ROADMAP.md` or `.planning/STATE.md`. That
  follows how 13.2 was inserted: the dated paragraph near the top of the roadmap, the
  `### Phase 13.2: The audit's fixes (INSERTED)` section, and the state file's position.
- It committed nothing.

## When the plans were brought together

Added 2026-10-20, after the twenty-four planned plans were on disk. The phase was inserted into
`.planning/ROADMAP.md` and `.planning/STATE.md` as above, and the phase README was written. No
planner had been given 13.3-25, the closing plan this file reserves, so it was written then, on
13.2-32's pattern. 13.3-09's planner found `mail-parser`'s charset decoder, so the question under
"Decisions that wait on Pratik" about making `encoding_rs` a direct dependency is not asked and
no package changes. The other corrections made that day are listed in the README, "Corrections
made when the plans were brought together".

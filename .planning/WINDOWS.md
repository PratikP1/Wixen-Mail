---
schema_version: 1
open_count: 522
waived_count: 0
fixed_count: 33
total_count: 555
last_updated: 2026-09-19T17:45:00.000Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | unrun-verify | src/presentation/folder_tree.rs |  | The Favourites group is not confirmed with a screen reader; FOLDER-03's last criterion is satisfied structurally only | open |  | 2026-08-30T19:15:54.071Z |  |
| 2 | 01 | deviation | src/presentation/wx_app.rs |  | The running tree builds one account at a time, so D-29's per-account Favourites branches are tested but not visible in the program yet | fixed |  | 2026-08-30T19:15:54.459Z | 2026-09-02T07:10:31.448Z |
| 3 | 01 | stub | src/presentation/message_rows.rs |  | conversation_cell_text is written and tested per column and has no non-test caller; 01-12 draws the collapsed conversation list | fixed |  | 2026-08-31T01:12:14.164Z | 2026-09-02T07:09:47.226Z |
| 4 | 01 | stub | src/presentation/message_columns.rs |  | Sort::conversation_order_by_clause is written and tested and has no non-test caller; 01-12 passes the user's chosen sort | fixed |  | 2026-08-31T01:12:21.927Z | 2026-09-02T07:09:55.366Z |
| 5 | 01 | deviation | src/application/conversations.rs |  | Hungarian's one-letter I: forward marker is read as a reply marker, because mail_parser's trim_trailing_fwd ignores a parenthesised word of one character | fixed |  | 2026-08-31T01:12:22.334Z | 2026-09-02T18:18:48.506Z |
| 6 | 01 | unrun-verify | src/presentation/wx_app.rs |  | Rethreading on arrival repaints one row and does not touch the selection; no screen reader has confirmed that a repainted row is silent to somebody not on it | open |  | 2026-08-31T05:13:47.847Z |  |
| 7 | 01 | stub | src/application/thread_identity.rs |  | A conversation root arriving after a message that names it is not merged: the link lives only in the other message's stored refs_header, which no index can search. Needs an identifier-to-conversation table | fixed |  | 2026-08-31T05:13:49.247Z | 2026-09-04T18:30:00.000Z |
| 8 | 01 | deviation | src/data/message_cache/messages.rs |  | messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column | fixed |  | 2026-08-31T05:13:50.314Z | 2026-09-03T09:36:22.242Z |
| 9 | 01 | deviation | .planning/phases/01-folders-and-conversations/01-13-PLAN.md |  | Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one | fixed |  | 2026-08-31T05:13:51.507Z | 2026-09-04T18:30:00.000Z |
| 10 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The coverage sentence before a saved search is announced as a low-priority status topic and has not been heard under a screen reader; it also coalesces with the Running this saved search line, which is by design and unverified by ear | open |  | 2026-09-01T02:14:56.334Z |  |
| 11 | 02 | unrun-verify | src/application/mail_sync.rs |  | The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names. Corrected on 2026-09-17 by 10-05: the fetch is the text pass of the download of everything now, fifty messages or 16 MiB a chunk with three refusals in a row ending the chunk and a wait of thirty seconds doubling to thirty minutes before the run is tried again, started by every check for mail rather than by a command, so the tester's Gmail account meets it unasked on the first check after the build; the question is unchanged and still open | open |  | 2026-09-01T03:56:05.356Z |  |
| 12 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The offer button and its experimental sentence have not been heard under a screen reader: whether the button is announced with its full label after a saved search, and whether the message text topic is heard rather than coalesced away, is unverified by ear | fixed |  | 2026-09-01T03:56:05.812Z | 2026-09-18T02:49:01.000Z |
| 13 | 02 | stub | src/presentation/wx_app.rs |  | The offer only appears while a saved search that reads message text is run; a person who never uses saved searches is never offered the fetch, which is where D-2-08 puts it and is a narrower reach than a menu command would have | fixed |  | 2026-09-01T03:56:12.023Z | 2026-09-01T17:15:20.187Z |
| 14 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The filter rule dialog's three Choice controls have an accessible object attached, which a test can see, but the name each one carries cannot be read back from wxdragon. Whether NVDA says 'Match field', 'Match type' and 'Action' rather than an unnamed combo box is unverified by ear | open |  | 2026-09-01T04:57:49.641Z |  |
| 15 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The Pattern box is disabled for the four ways of matching that read no pattern. Whether a disabled edit box is skipped cleanly in the tab order, and whether changing the Match Type while focus is nearby moves focus or is announced, is unverified by ear | open |  | 2026-09-01T04:57:57.259Z |  |
| 16 | 02 | unrun-verify | src/application/filters.rs |  | The eleven field names and eleven ways of matching are now read aloud as words. Whether 'Read is yes', 'Flagged is yes' and 'matches a text pattern' are understood when heard rather than seen is unverified by ear | open |  | 2026-09-01T04:57:57.717Z |  |
| 17 | 02 | unrun-verify | src/application/saved_searches.rs |  | The Save This Search window now reads out one clause per question, so a three-question search says a longer sentence than the one fixed sentence it replaced. Whether that is clearer or merely longer when heard is unverified by ear | open |  | 2026-09-01T06:59:55.094Z |  |
| 18 | 02 | unrun-verify | src/presentation/wx_app.rs |  | Saving a search whose folder belongs to another account is refused out loud through refuse_a_command. Reaching that state needs two accounts and Set Active, so whether the refusal is heard and understood is unverified | open |  | 2026-09-01T07:00:02.733Z |  |
| 19 | 02 | unrun-verify | src/presentation/wx_app.rs |  | A saved search narrowed to a folder has never been run against a real account. Whether the stored path resolves through get_folder for a real IMAP mailbox, rather than refusing with THAT_FOLDER_IS_NOT_HERE, is unverified against a live server | open |  | 2026-09-01T07:00:03.177Z |  |
| 20 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The Add/Edit Condition dialog says what a saved search cannot find with the chosen field, on a line of text under the controls and through the announcement queue. Whether it is heard when the field list changes, and whether a sentence that long is useful there rather than in the way, is unverified by ear | open |  | 2026-09-01T08:59:42.387Z |  |
| 21 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The Add/Edit Condition dialog's two lists carry accessible names set by this code, and wxdragon's Accessible has no name getter, so a test can only prove an object was attached. Whether NVDA says Match field and Match type rather than unnamed combo boxes is unverified | open |  | 2026-09-01T08:59:51.835Z |  |
| 22 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The Add/Edit Condition dialog refuses an empty pattern through a message box and puts focus back on the Pattern box. Whether the refusal is heard and whether focus lands where somebody expects is unverified by ear | open |  | 2026-09-01T08:59:52.282Z |  |
| 23 | 02 | stub | src/presentation/wx_managers.rs |  | build_rule_edit_dialog and show_rule_edit are built and tested and nothing in the running program opens them. Plan 02-07 wires the rule editor that does | fixed |  | 2026-09-01T08:59:52.753Z | 2026-09-01T10:38:15.106Z |
| 24 | 02 | stub | src/data/message_cache/saved_searches.rs |  | replace_saved_search is written and tested and has no caller outside its tests. Plan 02-07's rule editor is what calls it | fixed |  | 2026-09-01T08:59:53.265Z | 2026-09-01T10:38:15.561Z |
| 25 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The condition manager has never been opened in a running build. The path to it is traced and every part is tested, but nothing has run the modal loop: no window has been shown, no Add pressed, no Close refused | open |  | 2026-09-01T10:38:35.678Z |  |
| 26 | 02 | unrun-verify | src/presentation/manager_words.rs |  | Whether a tally on the end of every condition change reads well by ear, or is a clause somebody stops hearing. Only a condition list counts out loud, and whether that is the right set is a judgement a screen reader settles | open |  | 2026-09-01T10:38:36.133Z |  |
| 27 | 02 | unrun-verify | src/application/context_menu.rs |  | Whether the saved-search context menu reads correctly with a screen reader, and whether Edit conditions first is the right order by ear rather than Run this search again | open |  | 2026-09-01T10:38:36.557Z |  |
| 28 | 02 | deviation | tests/manager_dialog_labels.rs |  | wxdragon 0.9.17's ListCtrl::get_item_text loses the last character of every cell and returns a NUL in its place, so the window check reads a cell through a helper that allows for it. Upstream defect, not reported yet | open |  | 2026-09-01T10:38:36.983Z |  |
| 29 | 02 | unrun-verify | src/presentation/folder_tree.rs |  | The saved-search account branches have never been drawn in a running build. Whether a search now three levels deep reads well by ear, and whether the branch and the account's own branch are distinguishable when both say the account's name, is unverified | open |  | 2026-09-01T12:51:08.946Z |  |
| 30 | 02 | unrun-verify | src/presentation/wx_app.rs |  | Landing on a saved search now sets the working account. Whether that is heard, and whether somebody notices they have moved accounts by arrowing onto a search, is unverified by ear | open |  | 2026-09-01T12:51:09.391Z |  |
| 31 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The refusal for a saved search whose account has gone needs two accounts and one of them removed while a row for its search is still on screen. Never reached in a running build and unverified by ear | open |  | 2026-09-01T12:51:09.821Z |  |
| 32 | 02 | unrun-verify | src/presentation/wx_app.rs |  | A saved search has never been run against a real account under two accounts. That opening one under account B while account A is current returns B's mail is proved by tests over the decision and by the cache read that narrows on the account, not by a live run | open |  | 2026-09-01T12:51:10.257Z |  |
| 33 | 02 | unrun-verify | src/presentation/managers.rs |  | The search box's coverage sentence has never been heard. It is appended to the match count on the low-priority status topic, so it is now said on every search that reads message text, including when the whole mailbox is covered and the sentence says nothing new. Whether that is useful or is flooding on every search is a judgement only a screen reader run can make | open |  | 2026-09-01T14:30:53.917Z |  |
| 34 | 02 | unrun-verify | src/presentation/managers.rs |  | A search box search that finds nothing now signals NothingFound on its own topic at normal priority and sends the coverage sentence on the status topic at low priority. That both are heard, and in an order that makes sense, is reasoned from the queue keeping only the newest of a topic and is unverified by ear | open |  | 2026-09-01T14:31:02.712Z |  |
| 35 | 02 | deviation | src/data/message_cache/mod.rs |  | The box's coverage count is short for a database that already had a search index and had evicted bodies before this column existed. The index is contentless so it cannot be asked what it holds, and fts5vocab can but takes about nine seconds at two hundred thousand messages, so those rows are backfilled from message_bodies. The backfill asks whether the stored body holds text, which is the question the live writer asks; asking only whether a row was there counted a message with no text part as text the box can read, and that is fixed. Evicted messages stay findable by their text and are counted as though they are not. Short rather than over for them, and the set never grows. Two narrower ways it can still be over, both invisible to SQL and corrected the next time that message is indexed: a packed half that no longer unpacks, and markup that is one unterminated tag | open |  | 2026-09-01T14:31:03.243Z |  |
| 36 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The File menu item for the fetch has never been drawn in a running build: whether NVDA reads the experimental marking on its label and in the item description, and whether the offer's spoken line and the coverage sentence are heard as two answers rather than one contradiction, are both unheard | open |  | 2026-09-01T17:15:26.277Z |  |
| 37 | 02.1 | todo | src/presentation/wx_app.rs | 10262 | Two comments made false by 02.1-01 are still there and were found a second time by 02.1-02. Line 10262 says the ten checks in tests/wired.rs cannot use what_ships because it is cfg(test); it is behind a cargo feature now and they do. Line 19737 says the_window_itself reads this file and stops at the first cfg(test); it uses what_ships. Both instruct the next person to follow a convention for a reason that no longer holds | open |  | 2026-09-02T11:49:49.886Z |  |
| 38 | 02.1 | deviation | docs/roadmap.md | 156 | Folder favorites is unticked on the shipped roadmap and ships: ID_PIN_FOLDER draws a Pin Folder menu item, application::favourites backs it, and 02-08 used the Favourites branch as the precedent saved searches copied. Found by 02.1-03's tree search, left unfixed as outside criterion 5 and belonging to phase 2 | open |  | 2026-09-02T12:41:25.488Z |  |
| 39 | 02.1 | deviation | scripts/check.sh |  | The red half of red/green cannot be committed for a shell suite. check.sh runs every scripts/*.test.sh under set -e before it branches on the mode, so a failing suite aborts the gate before the red branch is reached and red-commit.sh verdict is never consulted. Measured by hand on 2026-09-02 by breaking one case in scripts/check.test.sh and committing with a Fails-until-green trailer naming it. Separately, verdict reads cargo's 'test NAME ... FAILED' lines, which a shell suite never produces, so a named shell case would report as never having run | fixed |  | 2026-09-02T13:30:26.438Z | 2026-09-03T08:58:32.016Z |
| 40 | 02.1 | deviation | tests/house_style.rs | 5499 | runs_the_suite exempts any line containing '--test ' as one that runs a named target on purpose, so a line naming fifteen targets without --no-fail-fast is exempt too. That hid a real defect in check.sh: 'cargo test --test house_style --test wired' ran two targets and stopped at the first failure. Found on 2026-09-02 only because building those targets into an array took the literal flag out of the text and the guard then spoke. The line is fixed; the exemption is still wider than one named target | fixed |  | 2026-09-02T13:30:36.985Z | 2026-09-03T08:58:24.727Z |
| 41 | 02.1 | deviation | src/presentation/folder_tree.rs |  | wxdragon 0.9.17 never removes a tree item's custom data from its process-global registry. cleanup_all_custom_data walks the tree through clean_item_and_children, which calls remove_item_data nowhere at all, for a leaf or for a branch, and the same walk is what runs automatically when the control is destroyed. delete_all_items goes straight to the FFI and removes nothing either. So set_custom_data and append_item_with_data leak one entry per row for the life of the process, and the only escape is not to call them. 02.1-05 took both dialogs off them; the folder tree in wx_app.rs was already off them and is held there by a source read. Upstream defect, not reported yet | open |  | 2026-09-02T15:06:11.672Z |  |
| 42 | 02.1 | unrun-verify | src/presentation/wx_app.rs |  | ask_about_the_folders_that_have_gone has never been opened in a running build. The four things its body decides are read from source by tests/wired.rs; a live window was available and not used, because every path that tells a right argument from a wrong one ends at MessageDialog::show_modal, which blocks with nobody to answer it, so a wrong argument would hang the commit gate rather than fail it | open |  | 2026-09-02T17:57:52.087Z |  |
| 43 | 02.1 | deviation | .planning/phases/02.1-what-phase-1-found-on-its-way-past/02.1-07-PLAN.md |  | The claim that a test cannot build a live window came back in a planning document. 02.1-02 corrected it in five source comments and left test_no_comment_says_a_test_cannot_build_a_window behind to stop it returning, but that guard reads Rust files only, so 02.1-07's plan could assert the budget was spent and nothing spoke. 02.1-05 had already disproved the same claim from its own plan. The guard cannot be widened to .planning without reading plans that are allowed to be wrong before they are executed, so this is recorded rather than fixed | open |  | 2026-09-02T18:32:59.629Z |  |
| 44 | 02.1 | unrun-verify | src/application/context_menu.rs |  | The six context menus the folder tree now offers have not been heard. Nothing confirms that an account branch's five entries and their mnemonics are announced, nor that the menu key doing nothing on All Inboxes, Favourites, On this computer and the saved searches heading reads as nothing to do here rather than as a key that failed. That last one is the risk this design takes on purpose: silence teaches as little as an item that does nothing, and only a real NVDA or Narrator run says which is worse | open |  | 2026-09-02T19:59:55.206Z |  |
| 45 | 02.1 | unrun-verify | src/presentation/folder_tree.rs |  | Account branches stopped reading their email address unless two accounts share a name. Nothing confirms by ear that the shorter label is an improvement, nor that the address appearing on two branches and not on a third is understood as a disambiguator rather than as an inconsistency | open |  | 2026-09-02T20:00:02.105Z |  |
| 46 | 02.1 | deviation | .planning/phases/02.1-what-phase-1-found-on-its-way-past/02.1-08-PLAN.md |  | The plan's premise correction stated that where_a_row_sits is production code with no production caller, measured that day, and prescribed wire it or remove it. It has one: wx_app::the_row_on_screen calls it once per row and which_row calls that on every folder tree selection, so it is on the main control's selection path. The premise was scoped to the defining file and to tests/ and never to sibling source files, and acting on it would have deleted live code. Recorded because the shape recurs: a negative reachability claim reads as a survey while naming only where somebody looked | open |  | 2026-09-02T20:00:11.219Z |  |
| 47 | 02.1 | deviation | src/application/context_menu.rs |  | D-2.1-03 says each branch kind gets its own menu and a group heading offers what is true of the group. Four rows got no menu instead: All Inboxes, Favourites, On this computer and the saved searches heading. Nothing this program does acts on one of them, and every candidate command reads whichever account is open, which on a row naming no account is whichever account somebody came from. The decision's own reason for rejecting no menu was losing genuinely useful per-account commands, and none is lost, because every row that names an account keeps its own. Recorded as a divergence from a recorded decision rather than as a fault | open |  | 2026-09-02T20:00:23.296Z |  |
| 48 | 02.1 | deviation | src/presentation/wx_app.rs |  | Criterion 12 was planned against two accounts of one name drawing rows that read identically. They did not: the_accounts_in_the_tree filled each name from Account::display_name, which is name and address together, and the accounts table declares email NOT NULL UNIQUE. The property was real, held by two layers that folder_tree.rs never mentions, and unowned there. The plan's own remedy would have added a second defence to a case that could not arise. What the trace found instead is the opposite defect, and it was fixed: the address was read aloud on every account branch, always, to serve a case that had never happened | open |  | 2026-09-02T20:00:23.940Z |  |
| 49 | 02.1 | unrun-verify | src/presentation/wx_managers.rs |  | The box a condition editor now shows instead of opening on a rule it cannot read has not been heard. It goes through a_sub_dialog_needs, which builds a MessageDialog a screen reader reads on its own, captioned "Not opened" before the open and "Not saved" before the write, and the sentence under it runs to two paragraphs. Whether the caption and the sentence read as one thing rather than two, and where the sentence breaks for speech, is unverified. Nothing in the library can hear it: every path from show_rule_edit or show_filter_edit to a real box ends at show_modal, which blocks with nobody to answer it, so a test that opened one would hang the commit gate rather than fail it | open |  | 2026-09-02T22:30:00.000Z |  |
| 50 | 03 | deviation | src/service/signed_mail.rs |  | Two certificate tests fail on GitHub's Windows runners and pass on a real machine: one of the runner's root authorities is genuinely reported withdrawn by Windows, and its three authorities produce no per-certificate answer because nothing local holds a withdrawal list. Checked 2026-09-03 and deferred by Pratik on the ground that it does not change how the application behaves: what_windows_found maps only CERT_TRUST_IS_REVOKED and CRYPT_E_REVOKED to Withdrawn, while offline, no list held, and no revocation information each map to CouldNotFindOut with a reason, so the code never reads could-not-check as revoked. CI stays red on these two until a runner with a representative certificate store exists, or the tests take their certificates as an argument. | open |  | 2026-09-03T20:58:51.850Z |  |
| 51 | 03 | deviation | src/service/spellcheck/windows_speller.rs | 166 | supported_languages returns an empty list both when this machine has no spell checkers and when the COM call failed, with nothing logged: CoCreateInstance's error is discarded by a let-else that returns the empty vec. available_languages then reports no languages, best_available_match answers None, and default_language at data/config.rs:466 falls back to en, so a transient COM failure at first run sets a French user's fresh install to English and marks every word of their mail wrong. Found 2026-09-03 while investigating the one-in-five test flake the phase 1 deferred list records; the flake is this defect seen through a test that asks the same question twice. The codebase already has the right shape for the fix in Withdrawal, which distinguishes NotWithdrawn from CouldNotFindOut with a reason. Not yet routed to a phase. | fixed |  | 2026-09-03T20:59:03.743Z | 2026-09-03T22:56:29.517Z |
| 52 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Criterion 1's announcement half is structure only. The renumbering sentence is built in mail_sync::what_the_renumbering_discarded, sent as UIUpdate::FolderWasRenumbered, and announced by handle_update on its own topic "renumbered" at Priority::Normal, and a source-reading test holds all three. No screen reader has heard it. Three things only an NVDA or Narrator run settles: whether the sentence is spoken at all when a folder is renumbered mid-sync; whether a topic of its own is the right choice against "status", since the reason for splitting it off is that the queue coalesces same-topic announcements and the next "Checking Sent..." would replace it, which is reasoning about the queue rather than an observation of it; and whether a Normal-priority announcement arriving in the middle of a sync cuts across something the person was reading, which is guardrail 5's bounded-and-distinct question and cannot be answered by reading source. Compounded by the fact that no real server has ever renumbered a folder for this program, because it has never been used with an account, so the whole path has only run against a scripted server. | open |  | 2026-09-03T22:40:54.889Z |  |
| 53 | 03 | deviation | Cargo.toml |  | wxdragon is pinned at =0.9.17 and 0.9.21 is out. Checked 2026-09-03 while reporting the two defects this project had recorded as unreported. Ledger 28, ListCtrl::get_item_text losing the last character of every cell, was already reported by somebody else as AllenDang/wxDragon#205 against 0.9.19 and is fixed on master: the fix allocates needed_len + 1 and its comment names that issue and the same mechanism this project diagnosed. So 28 wants an upgrade rather than a report, and the workaround helper in tests/manager_dialog_labels.rs comes out when the upgrade lands. Ledger 41, TreeCtrl::cleanup_all_custom_data walking the tree and removing nothing, is still present on master and is now reported as AllenDang/wxDragon#214 with a suggested fix. Upgrading four minor versions of the UI framework is its own piece of work and is not phase 3's. | open |  | 2026-09-04T04:36:43.729Z |  |
| 54 | 03 | deviation | src/presentation/wx_app.rs |  | A source-reading check reports findings as {path}:{at + 1} where the index comes from what_ships(text).lines().enumerate(), over every Rust file under src. That is the file's own line number only while nothing was cut above the finding: for any file with a #[cfg(test)] item above a send_status line the reported position is short by however many lines were deleted, silently and with a well-formed message pointing at the wrong line. Correct today for the files it reports on, which is why it reads as blessed practice and is the precedent a new source-reading check would copy. Found 2026-09-04 while writing tests/one_sign_in_per_piece_of_work.rs, which carries line numbers through the cut instead. Out of 03-02's scope. | open |  | 2026-09-04T05:46:15.242Z |  |
| 55 | 03 | deviation | src/data/message_cache/mod.rs |  | Plan 03-03's must_have truth 'a marker that is wrong in the dangerous direction cannot lose a body, because a cheap probe that the marker never skips is what re-checks it' is unmet as written, and met more strongly in substance. No marker was built. Following the plan's own ordering through, a marker that never gates the question decides nothing: the probe answers in both branches and the marker is written and never read. What shipped is a partial index (idx_messages_inline_body) over exactly migrate_inline_bodies's condition, which makes the question free rather than making a wrong answer harmless, so there is no state that can be wrong at all. The reason not to add a marker later is a comment in mod.rs beside the index and in bodies.rs on THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE, and a guard record whose break is the marker the next person would reach for. Recorded so an audit comparing the plan's truths against the summary is not left guessing. | open |  | 2026-09-04T10:00:24.498Z |  |
| 56 | 03 | unrun-verify | src/data/message_cache/messages.rs |  | Nothing in plan 03-04 has run against a real Gmail account, because this program has never been used with an account at all. The archived-with-no-label fix is proved against a mail cache built inside a test: real evidence about the SQL, no evidence about what Gmail sends. Specifically unverified: that a message archived without a label really appears in All Mail and nowhere else on a live account; that X-GM-MSGID really comes back on the same message under a label and in All Mail; that holds_all_mail is really set for Gmail's All Mail by a live LIST response. Closes only against a real account. | open |  | 2026-09-04T13:45:10.738Z |  |
| 57 | 03 | deviation | src/data/message_cache/messages.rs |  | A message is still counted twice in a conversation if a server holds it in two places and gives it neither a Gmail identifier nor a Message-ID. WHICH_MESSAGE_THIS_ROW_IS falls back to the row id, so two such rows are two messages. Chosen deliberately over merging by row position: a count that is too high is visible, a conversation that has vanished is not. Also unfixed and pre-existing: a Gmail message under two labels counts twice, because both label rows are real rows outside All Mail and nothing says which label should lose. Fixing that needs the count and the delete list to become different questions, which is an architectural change rather than a predicate. | open |  | 2026-09-04T13:45:22.582Z |  |
| 58 | 03 | deviation | src/data/message_cache/messages.rs |  | Measured cost of the identity filter, release build, warm, 200,000 rows in 10,000 conversations. On an account with a folder holding all mail the conversation listing goes from about 0.75s to about 1.2s, roughly 60 percent more, of which about 300ms is the filter and about 150ms the extra rows now in reach. On an account with no such folder there is no measurable difference, 0.86s against 0.85s, so the short-circuit claim in conversation_scope's doc comment was measured rather than assumed. Neither number is acceptable on its own terms: conversations_query has no LIMIT and groups the whole account on every listing, which is SCALE-03's subject and was true before this change. | open |  | 2026-09-04T13:45:35.198Z |  |
| 59 | 03 | deviation | src/data/message_cache/searching.rs |  | searching.rs:539 groups search results by COALESCE(m.gmail_msgid, m.id), which is the identity plan 03-04 found insufficient for the conversation count. On a server that advertises the RFC 6154 All attribute and gives no Gmail identifier, a search shows the same message twice, once per copy. Same class of defect, same remedy available (the Message-ID arm of WHICH_MESSAGE_THIS_ROW_IS), pre-existing and outside 03-04's scope. test_one_gmail_message_under_two_labels_is_found_once covers the Gmail case only. | open |  | 2026-09-04T13:45:47.023Z |  |
| 60 | 03 | unrun-verify | src/data/message_cache/messages.rs |  | Nothing in this plan has run against a real account. That a real client sends In-Reply-To without References, that a conversation root really does arrive after a message naming it during a live sync, and that the first open after this change is bearable on somebody's real mailbox are all unverified: the merge, the backfill and every timing here are measured against a cache built inside a test on this computer. | open |  | 2026-09-04T18:30:00.000Z |  |
| 61 | 03 | deviation | src/application/thread_identity.rs |  | A merged conversation can settle under an identifier that is nobody's root. Two conversations an arrival has proved to be one carry no ordering between their names, so the winner is the least of them by ordinary string comparison, which is stable and arbitrary. Stability is what was needed and finding the older message is not available to rejoin. Recorded rather than glossed, because for a chain naming only its parent the conversation is then filed under a message in the middle. | open |  | 2026-09-04T18:30:00.000Z |  |
| 62 | 03 | unrun-verify | src/data/message_cache/messages.rs |  | A merge renames one of the two conversations and nothing in the running program says so. The changelog says a conversation may change which message it is filed under; the interface does not, and whether somebody reading a conversation notices it move under a screen reader is unverified by ear. | open |  | 2026-09-04T18:30:00.000Z |  |
| 63 | 03 | deviation | src/data/message_cache/mod.rs |  | The first open after this change walks every stored message and reports nothing while it does. Measured at 5.66 seconds with nothing to join and 6.45 with every conversation split in two, over two hundred thousand messages on this computer. It happens once, it is gated on a probe rather than a marker, and a larger mailbox pays more with the window showing nothing. | open |  | 2026-09-04T18:30:00.000Z |  |
| 64 | 03 | unrun-verify | src/application/mail_controller.rs |  | Whether a real provider accepts a fresh sign-in straight after it has dropped a connection, or treats it as something to slow down or refuse, is unknown. The single retry is proved against a loopback server that hangs up on command and answers the next connection immediately. No account has ever been used with this program, so nothing here has met a provider's real behaviour on reconnect, including whether it counts against a connection limit. Corrected on 2026-09-18 by 10-06: the inbox watch is started again after a wait of thirty seconds doubling to thirty minutes whenever it ends for any reason but mail arriving, and at once when the network comes back, so the tester's Gmail account meets a fresh sign-in after a dropped connection unasked, on the first drop after the build; what the provider does with it is what this entry asks and is still unobserved. | open |  | 2026-09-04T21:03:06.059Z |  |
| 65 | 03 | unrun-verify | src/application/mail_session.rs |  | What a real provider does with a session held open and idle for minutes is unknown, and the whole point of holding one is that it sits idle between commands. Whether providers drop an idle IMAP session at all, how soon, and whether they say anything before they do, has never been observed by this program: no account has ever been used with it. The reconnect exists because a drop is expected, and that expectation is reasoning rather than a measurement. Corrected on 2026-09-18 by 10-06: the watch connection, one IDLE session per enabled IMAP account renewed every twenty-nine minutes, is now held open for as long as the program runs and started again after a growing wait when it ends, so whether a provider drops an idle session, how soon, and whether it says anything first is what the tester's account shows on the first day of the build; a drop that says nothing can go unnoticed for up to twenty-nine minutes, and the check on the account's interval is what covers that. | open |  | 2026-09-04T21:03:24.528Z |  |
| 66 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Whether the refusal after a failed retry is heard once rather than once per failed request is unverified by ear. It reaches somebody through each site's existing reporting, which is ErrorOccurred for the flag path and CommandRefused for the folder commands, and both announce at High priority through accessibility::announce. That is structure, not experience: nobody has heard it with NVDA, and a mailbox where every command meets a dead connection would produce one of these per command with nothing coalescing them, which is exactly the flooding guardrail 5 is about. | open |  | 2026-09-04T21:03:34.593Z |  |
| 67 | 03 | unrun-verify | src/application/mail_session.rs |  | The connection budget of two per account is counted against a loopback server and has never been counted against a provider. Whether two per account is welcome, what a provider counts as a connection when several accounts sit on the same one, and whether the IDLE connection and the working session are counted together, are all unknown. Gmail's limit of fifteen per account is the number the requirement's evidence records rather than one this program has ever approached. | open |  | 2026-09-04T21:03:43.871Z |  |
| 68 | 03 | deviation | src/service/protocols/imap.rs |  | folder_counts has the same shape select_folder was fixed for and is not fixed. It calls async-imap's session.status, whose parser reads responses until the stream ends and hands back what it collected, so a connection dropping mid-command comes back as Ok with nought messages and nought unread. Corrected on review 2026-09-04: this said a wrong number rather than a deletion, and that understates it. A count of nought is what disarms listing_contradicts_the_count, which is listed == 0 && counted > 0 and is the only check between a truncated listing and an emptied folder. select_folder erroring now aborts the sync before list_uids is reached, which closes the path, so this is latent rather than live; it would become live again if anything ever reads the count without the SELECT in front of it. Same defect underneath: a command that never completed reported as one that did. Fixing it means writing STATUS as a command line through read_command, the way select_folder now is. | open |  | 2026-09-04T21:03:53.498Z |  |
| 69 | 03 | deviation | src/presentation/wx_app.rs |  | Checking for mail used to refuse an unusable port with the value it could not read, 'has an IMAP port that is not a number: 14 3'. All twelve sites lost their own port check when they went through the held session, because a_session_at asks the same question and answers it in the same words, so each was a second answer to one question. Eleven lost nothing by that; this one lost the offending value, which is the part somebody fixing it needs. The value is visible in the account settings screen. | open |  | 2026-09-04T21:04:03.565Z |  |
| 70 | 03 | unrun-verify | src/application/finding_what_was_deleted.rs |  | Whether any provider grants CONDSTORE, which is what the resume needs. imap/abilities.rs asserts that Gmail never has. Fastmail and current Dovecot advertise it in the capability lists this project models them on, and no capability list has ever been read off a real server here. If none of the providers people use grants it, SCALE-01's saving applies to nobody and every folder is read out in full on every sync, which is what happens today anyway. | open |  | 2026-09-05T01:35:34.235Z |  |
| 71 | 03 | unrun-verify | src/application/finding_what_was_deleted.rs |  | Whether a hand-built SELECT with QRESYNC parses back at all, which is what the declared and unbuilt VANISHED member would need. async-imap 0.11.3 has no ENABLE and no select_qresync, so it goes through run_command, and the mailbox response that comes back is one async-imap's own select parses. imap-proto already parses Response::Vanished. Whether the raw select parses and whether VANISHED reaches the closure has never been run against a server, and it is the whole cost of the second implementor. | open |  | 2026-09-05T01:35:52.234Z |  |
| 72 | 03 | unrun-verify | src/application/bringing_everything_down.rs |  | Whether a provider tolerates a whole-folder request. It asks for a folder five hundred messages at a time, without stopping, until the folder is here. Ledger 11 records the same gap for the bulk body fetch and this is the same shape at a different granularity: a provider is entitled to refuse, throttle, or disconnect, and nothing on this side can find out which. Two things follow that no test here can settle: how many chunks a provider allows before it slows down, and whether a disconnect part way is reported as the request stopping short rather than as the folder being finished. Marked experimental on the menu item and in its description. Corrected on 2026-09-17 by 10-05: the whole-folder request and its module are gone; the loop is the download of everything, which asks bringing_everything_down::what_to_do_next for every kept folder of every enabled IMAP account after every check, stops when Pause Downloading is ticked, and waits a growing time after a refusal, so a disconnect part way is reported as the folder stopping short and the run is tried again on its own; the experimental sentence is on the Pause item; the question is unchanged and still open, and the tester's Gmail account meets it unasked | open |  | 2026-09-05T01:36:00.334Z |  |
| 73 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Three things about the whole-folder request that only a screen reader settles, on the pattern of phase 2's entries 10, 33 and 34. Whether a fetch of eighty chunks on a topic of its own is heard rather than lost: the topic keeps only its newest announcement, so the claim is that somebody hears a handful of sentences, and nothing here has listened. Whether the final count is heard as an ending rather than as another progress line; the words differ and the topic does not. And whether the choice of a topic of its own is right at all against putting it on 'status', which is the one open question in plan 03-07 and is a listening judgement: the argument for splitting is reasoning about how the queue coalesces rather than an observation of it, and the constant THE_PROGRESS_TOPIC is the one line that moves it. | open |  | 2026-09-05T01:36:08.578Z |  |
| 74 | 03 | unrun-verify | src/presentation/message_rows.rs |  | Whether the snippet column reads well when a screen reader crosses a column of rows that all say 'Message text not downloaded'. That is every row of a folder nobody has fetched text for, which is most of a large mailbox, and four words per row is four words more than the blank it replaced. The blank was a lie and the words are true, so this is a question about whether the true answer is worth what it costs to hear, not about whether to go back. If it is too much, the shorter answer is to say it once for the column rather than once per row, and there is nowhere on a virtual list to put that today. | open |  | 2026-09-05T01:36:19.878Z |  |
| 75 | 03 | deviation | src/data/message_cache/bodies.rs |  | The snippet column tells 'nobody fetched this text' from 'this message has no text' by whether the stored snippet is null or empty, and only rows written after this change carry the distinction. A message whose body was fetched before 2026-09-04 and held no text was stored as null, so it reads as one nobody has fetched, and the row says so until its text is fetched again. There is no backfill: the fact is not recoverable from anything the database still holds, because an evicted body leaves no row and message_bodies answers 'is the text here now' rather than 'was it ever fetched'. | open |  | 2026-09-05T01:36:20.285Z |  |
| 76 | 03 | unrun-verify | src/application/the_network_coming_and_going.rs |  | Whether the sentence about the network going is heard once and understood as a state rather than as an error. The state hands out one answer per change and a test drives ten failures and counts one, which is the structure. What that cannot say is what reaches somebody: the announcement goes out on a topic of its own at normal priority while a sync is failing its way through several folders on the status topic underneath it, and whether the one that matters is the one heard needs NVDA and a cable pulled out. Whether a sentence beginning 'The network has gone' reads as information rather than as something broken is the same kind of question. Nothing here has ever met a real network loss. | open |  | 2026-09-05T05:19:21.804Z |  |
| 77 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Whether the offer to go back online is announced with its full label, and whether a screen reader user learns it is there at all. The panel is shown by an arm that says nothing, on purpose: the sentence sent immediately before it names the button, and announcing again would be two announcements a moment apart about one event. That argument is about repetition and it does not settle discovery. A button appearing above the message list moves nothing and takes no focus, so what tells somebody it exists is one clause in one sentence, and whether that clause survives being heard in the middle of a mailbox is an NVDA question. Its label and its accessible name come from one string, which a test reads from the source, and whether Windows really speaks that string for this control is the MSAA and UI Automation question scripts/msaa-names.ps1 exists for and which has not been run against this window. | open |  | 2026-09-05T05:19:37.963Z |  |
| 78 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Whether the status bar and the announcement being the same words reads as a repetition when somebody meets both. The network sentence is written once and handed to status field 0 and to the announcement queue, which is what stops a deaf user and a blind user being told different things. A deaf-blind user reading the status bar on a braille display and then hearing the queue speak, or a low vision user with speech on, meets the same sentence twice within a second. Whether that is reassuring or is noise is not something a test can ask. | open |  | 2026-09-05T05:19:48.464Z |  |
| 79 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Whether somebody who lets the offer go by can find their way back. Two routes exist and neither has been used by a person. The offer panel stays on screen until the mode changes some other way, so it is still there to be tabbed to, and the sentence names the View menu as the other way. What is unknown is whether either is reachable in practice for somebody who heard the sentence once, was reading a message at the time, and comes back to it twenty minutes later with nothing repeating it. There is nothing that says the offer again. | open |  | 2026-09-05T05:19:48.844Z |  |
| 80 | 03 | unrun-verify | src/service/network.rs |  | Whether InternetGetConnectedState answers usefully on a real machine losing a real network. It reports whether this computer has a connection at all, which is what makes it right for a cable pulled out and a wifi dropped, and it says nothing about whether a mail server can be reached, so a network that is up and cannot route leaves the program believing it is online. It has never been run against a machine that lost its network while Wixen Mail was open. Two things a run would settle: how long Windows takes to change its answer after the cable goes, which is the real delay before somebody is told rather than the ten second interval this asks on, and whether a wifi that flaps produces a run of changes the queue then speaks. | open |  | 2026-09-05T05:20:01.150Z |  |
| 81 | 03 | deviation | src/application/sending_later.rs |  | The ten second Undo Send hold is never applied to anything. Hold, GoAfter::held and queue_outbox_message_to_go have no caller outside sending_later.rs and its tests: the composer's Send queues through queue_outbox_message, which writes GoAfter::AsSoonAsPossible, so readiness answers MayGoNow at once and take_back answers TooLate for every message somebody has just sent. The module doc says all of it runs. Undo Send is on the Tools menu and can never catch a message from the composer. Found while wiring offline mode into the same decision and left alone as out of scope: it is a behaviour change of its own with its own countdown to show and its own version bump. | fixed |  | 2026-09-05T05:20:01.525Z | 2026-09-06T23:12:00.392Z |
| 82 | 03 | unrun-verify | src/presentation/wx_conflict_choice.rs |  | Whether the two copies of a contact or a calendar item are understood by ear as a labelled pair. Each list is headed by a static text and named through set_accessible_name with the same string, What is on this computer and What your address book has, and the arrangement chosen is one sentence on opening saying what is being asked and how many fields differ, then each copy introduced by its label as focus reaches it. Whether that beats reading both copies out on opening is a judgement only a screen reader run settles, and nobody has heard this window. Three things a run would settle: whether the two headings are heard as headings rather than as more list content, whether the opening sentence is heard before somebody starts arrowing through the first list, and whether the list of differing fields inside that sentence is useful or is a clause people learn to skip. | open |  | 2026-09-05T08:50:01.364Z |  |
| 83 | 03 | unrun-verify | src/application/conflict_choice.rs |  | Whether the count of waiting choices is useful or is a sentence somebody stops hearing. Every sync that found a disagreement ends with a whole sentence naming how many are waiting and where to make the choice, on top of the counts the sync already reads out. Phase 2 entries 26 and 33 are the precedent and asked the same question about a tally read aloud. What a run would settle: whether a sentence arriving after five counts is still heard, and whether somebody who hears it on every sync until they act finds it a reminder or a nag. | open |  | 2026-09-05T08:50:14.084Z |  |
| 84 | 03 | unrun-verify | src/application/contacts_sync.rs |  | The push still sends a change typed here over the address book's newer copy, and nobody is asked. When a push is refused for carrying a version marker the address book has moved past, and the copy here was typed here, the push reads the address book's current marker and sends the change again on top of it. That is a second both-changed state, resolved in this computer's favour, at the provider, with a sentence afterwards. It is the same shape as the defect plan 03-09 fixed on the read side and it was left alone: it is guarded by two records, its behaviour is deliberate and argued for in guards.toml, and changing it means changing what those guards are about. Found while executing 03-09, whose own key link named the counter for this path as the model for how the losing case was told, which it is not. | open |  | 2026-09-05T08:50:23.245Z |  |
| 85 | 03 | deviation | src/application/calendar_conflict.rs |  | Two files hold tests about code that lives elsewhere, and both are a deliberate trade for guard re-measurement time. The CalDAV sync-path test lives in calendar_conflict.rs because 23 records fingerprint caldav_sync.rs's test count, and the choosing window's assertions live in tests/the_conflict_choice_can_be_heard.rs because 37 fingerprint wx_app.rs. What that costs: a test about the CalDAV sync sits one file away from the sync, and the window's own behaviour is asserted by reading source rather than by building a window. What is therefore not guarded from inside caldav_sync.rs is that the read consults calendar_conflict at all, and from inside wx_conflict_choice.rs that the dialog builds what the source says it builds. Both are covered by the new records instead, coupled through guards.toml so they run on the commits that could break them. | open |  | 2026-09-05T08:50:31.713Z |  |
| 86 | 03 | unrun-verify | src/application/flag_changes_waiting.rs |  | Whether the two sentences about a flag change are distinguishable by ear. One says the server could not be reached and the change is saved here; the other says the server refused it and it has been put back. They share no opening clause and no verb, and a test holds them to that, but whether somebody hearing one in the middle of a syncing mailbox knows which they heard is a judgement only a screen reader run settles. Three things a run would settle: whether the two are told apart at speed, whether announcing them on their own topic rather than the status line means they are heard at all, and whether the count in the plural form is understood as a number of changes rather than as a number of servers or messages. | open |  | 2026-09-05T09:47:34.277Z |  |
| 87 | 03 | deviation | src/presentation/wx_app.rs |  | A label added or removed still puts itself back when the push fails, whatever the reason. The waiting queue models two flags, read and starred, because those are the two a message row carries as its own state and the two the window puts back by sending the opposite update. A label is a keyword, of which a message can have many, and replaying one needs the keyword as well as the value. Left out rather than half-built: the arm names the case, says why, and takes the path it always took. The changelog says so under Known limitations. | open |  | 2026-09-05T09:47:46.170Z |  |
| 88 | 03 | unrun-verify | src/application/flag_changes_waiting.rs |  | Whether Authentication counting as the server never having been asked is the right call against a real provider. A sign-in the server turned down means the change was never put to it and a token that has expired is fixed by signing in again, so the change is kept. Against a provider that answers Authentication for something that does not clear on its own, a wrong password nobody corrects, the change waits for ever and is offered on every sync. Nothing here has met a provider, so which errors a real one raises for an expired token against a wrong password is unknown, and that is the fact the decision rests on. | open |  | 2026-09-05T09:47:46.593Z |  |
| 89 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Whether the sender's description is heard at the right moment in an attachment row. It is the fourth and last clause, after the name, the kind and the size, on the reasoning that the first three are what somebody decides to open a file on and the sender's words are what they want if they are still listening. That is a judgement about what a person wants to hear first, not a measurement, and only a real NVDA or Narrator run through a message with several attachments settles it. The row is announced every time focus reaches it, so getting the order wrong costs a moment on every arrow press. | open |  | 2026-09-05T14:09:25.832Z |  |
| 90 | 04 | unrun-verify | src/service/mime.rs |  | Whether a description borrowed from the alt on the img that names a part reads as the sender's own words or as something the program made up. The row says the text with nothing marking it as borrowed, on the grounds that the alt is the sender's writing about that picture as much as a Content-Description would be. Nobody has heard it. If a borrowed description reads as an assertion by Wixen Mail rather than by the sender, that is a wording problem the tests cannot see, and it matters more here than for the header because the borrow really is a guess about which element meant which part. | open |  | 2026-09-05T14:09:33.628Z |  |
| 91 | 04 | unrun-verify | src/service/mime.rs |  | Whether real senders supply Content-Description at all, and how often. If they mostly do not, this feature mostly says 'no description' on the header route and leans entirely on the alt borrowed from the markup. No mail account has ever been used with this program, so it cannot be measured here, and the changelog says so rather than implying the feature does more. It decides whether the header route was worth building or whether the markup route is the whole of it. | open |  | 2026-09-05T14:09:39.886Z |  |
| 92 | 04 | deviation | src/application/pop_sync.rs |  | A POP account records no attachment rows at all, so nothing this plan built reaches one. The plan's premise that the IMAP path and the POP path are two writers of the attachments table is wrong: the only production writer is wx_app::spawn_body_fetch, which returns early when the account has no IMAP server, and pop_sync sets has_attachments and stores nothing else. So a POP message says it carries an attachment and lists none, which predates this plan and is not made worse by it. Left alone rather than half-fixed: adding a writer to the POP sync is a new path through the cache, not a widening of this one. | open |  | 2026-09-05T14:09:51.050Z |  |
| 93 | 04 | deviation | src/service/mime.rs |  | The record 'a description the sender gave survives the boundary' went stale inside the session that wrote it. Written in the morning naming one test, it named too few by the afternoon, because the alt lookup added later gives a part whose header is dropped somewhere else to fall through to. Corrected by hand and re-measured. Recorded because CLAUDE.md predicts this shape and the only reason it was caught is that the second task re-ran --remeasure rather than trusting the first task's measurement; nothing would have failed if it had not. | open |  | 2026-09-05T14:09:51.446Z |  |
| 94 | 04 | unrun-verify | src/application/blocking.rs |  | Nobody has heard the mailing-list warning. It is announced at Priority::High before the block is made, and two things about that are judgements rather than measurements: whether it lands before the block rather than reading as a report of one already made, and whether an email address said aloud in the middle of a sentence is understood at speed by somebody arrowing through a mailbox. The sentence is: 'This message came from a mailing list. Blocking files it into Junk and the list carries on sending it. To stop it at the source, unsubscribe by writing to birds-leave@lists.example.' Only a real NVDA or Narrator run settles either. | open |  | 2026-09-05T15:59:50.717Z |  |
| 95 | 04 | unrun-verify | src/service/mime.rs |  | Whether real mailing lists write List-Unsubscribe in the shape where_to_write_to_leave reads has never been measured. No mail account has ever been used with this program. Sixteen spellings of the header were probed against mail-parser 0.11.5, which settles what the library does with a given header and says nothing about what senders send. If real lists commonly write the header in a shape that carries no angle-bracketed mailto:, the warning fires and always says to look for a link, which is a weaker feature than it reads as here. | open |  | 2026-09-05T15:59:58.905Z |  |
| 96 | 04 | deviation | src/service/protocols/imap.rs |  | The plan does not mention HEADER_FIELDS, the list of headers an IMAP fetch asks a server for. Without LIST-UNSUBSCRIBE on it, every other hop of this feature is correct and no message on an IMAP account carries the header, which is the whole feature dead on the commonest account type with 6270 tests green. Added, and guarded by a record coupled to tests/the_list_warning_reads_the_message.rs, because the whole library was run against the break and stayed green: nothing in it can see this hop at all. Recorded because a request that names the fields it wants is a silent-drop point invisible to tests on either side of it, and the same shape exists wherever a projection is narrowed. | open |  | 2026-09-05T16:00:07.187Z |  |
| 97 | 04 | deviation | src/service/outlook_data_file.rs |  | A message imported from an Outlook data file carries no List-Unsubscribe, so blocking its sender gets no warning. The importer rebuilds a message from the pieces a PST holds and the transport headers are not among the pieces it reads. Left as None with a comment saying so rather than papered over: recovering it means reading the header property out of the file and writing it into the bytes the importer then re-parses, which is its own change. Messages filed from a sent copy and from an archive read through mime::parse do carry it. Corrected on 2026-09-16 by 09-08: this described an importer path that did not exist, since nothing called the reader from the day it was written; File, Import Mailbox reaches it now through application::importing_an_outlook_data_file, so the deviation is real and stays open, and the fix is the same. | open |  | 2026-09-05T16:00:13.954Z |  |
| 98 | 04 | deviation | src/application/blocking.rs |  | where_to_write_to_leave names whatever sits between <mailto: and > without asking whether it is an address, so a sender can put a web address or any other text there and have the warning say 'unsubscribe by writing to' it. Left alone deliberately, and the reasoning matters more than the decision: validating it would only reject malformed junk, because the real threat is a well-formed address belonging to somebody else, which no validation can tell from a real one. Nothing in this program acts on the value, so nobody is one keystroke from a stranger either way. Recorded so the judgement is visible rather than assumed. | open |  | 2026-09-05T16:00:21.699Z |  |
| 99 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-02-PLAN.md |  | Two of this plan's premises were wrong in ways that would have shipped a broken feature or a weaker test, and both were found by measuring rather than reading. It says to read the header through header_text as receipt_request does; mail-parser parses List-Unsubscribe with its address parser, which strips the angle brackets where_to_write_to_leave searches for, so that route reports every mailing list as one that gave no way out. And it says the second task's census cannot be red because it must name a construction the first task creates; the construction already existed and only its argument changed, so the census was red before any implementation. Recorded because both are general: an accessor's parsed and raw forms are different values, and 'no red is available' is a claim about the tree that is cheaper to falsify than to work around. | open |  | 2026-09-05T16:00:31.782Z |  |
| 100 | 04 | deviation | guards/guards.toml |  | Two pre-existing guard records were found wrong, both surfaced by the count check because src/application/mail_sync.rs gained one test. 'a sync writes no attachment for a message nobody has opened' had been UNMEASURABLE since 04-01 landed hours earlier: its recorded break writes a CachedAttachment literal, 04-01 added a description field to that struct, and the break stopped compiling, so the run reported a broken tool rather than a finding. 'a count and the thing it counts agree in number' named 16 tests for a break that reddens 17, missing one in application::contacts_sync, a module nobody working on mailing lists would have filtered for. Both corrected by hand and re-measured. Recorded because neither has anything to do with this feature and neither would have been found by any check this plan ran on purpose. | open |  | 2026-09-05T16:39:54.486Z |  |
| 101 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Nobody has heard the encryption sentence. The reader speaks said_before_the_message when a message opens, so the sentence is spoken before the body, and two things about that are judgements rather than measurements: whether it lands early enough that somebody arrowing into a message meets the explanation before the armour, and whether 'This message is encrypted. Wixen Mail cannot open it, so what is shown below is the encrypted form rather than the message.' is understood at speed. Only a real NVDA or Narrator run settles either. | open |  | 2026-09-05T18:22:00.937Z |  |
| 102 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Whether a bar carrying a filter's verdict and an encryption sentence together reads as two facts or as one run-on has never been heard. The two are joined with a newline, the filter's verdict keeps the top, and the encryption sentence goes under it. In a text control that is two lines; spoken by a screen reader it may be one breath, and a phishing warning running straight into an explanation of armour is a sentence somebody may hear as one claim about one thing. Only a real NVDA or Narrator run settles whether the join needs more than a line ending. | open |  | 2026-09-05T18:22:16.997Z |  |
| 103 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Whether the PGP signature sentence is heard as a disclaimer or as reassurance is the one that matters most and is the one tests cannot reach. It reads 'This message carries a PGP signature, which Wixen Mail cannot check, so nothing here says whether it is genuine.' Tests assert the words it does not contain, which is a check on the wording and not on what somebody takes away. Being told a message is signed is easily heard as being told it is genuine, and if the second clause is talked over the first clause is reassurance nothing earned. Only a listener settles it. | open |  | 2026-09-05T18:22:17.388Z |  |
| 104 | 04 | deviation | src/application/body_safety.rs |  | Whether real PGP mail arrives with its armour in a text part at all has never been measured, and it decides whether this feature fires in practice. The detection reads the two halves of a parsed body for the armour markers, which is how inline PGP arrives. Mail sent as multipart/encrypted carries the armour in an application/pgp-encrypted part, which mime::parse's first_of_kind does not yield as a body, so it never reaches this and would open with nothing said. No mail account has ever been used with this program, so which of the two real senders use cannot be answered here. Said in the changelog as a known limitation rather than implied away. | open |  | 2026-09-05T18:22:28.331Z |  |
| 105 | 04 | deviation | src/presentation/reader_text.rs |  | A conversation of several messages read as one document says nothing about any one message's form, so an encrypted message inside a thread still shows its armour with nothing said. reader_text::conversation folds the sentence in only when the document holds exactly one part. The reason is the one with_signature already gives for staying off a thread: there is a form per message and one bar over all of them, so 'This message is encrypted' over a thread of five is heard as covering five. Closing it properly means a sentence naming which message, which is its own wording question. Opening that message on its own does say it, and that is how somebody reads a particular message. | open |  | 2026-09-05T18:22:28.728Z |  |
| 106 | 04 | deviation | tests/an_encrypted_message_is_not_left_unexplained.rs |  | The census written for this plan passed against its own break the first time it was measured, and the fix is worth remembering as a class. It asserted that each composer asks the encryption question. The break took out the fold that puts the answer into the bar and left the question in place, bound to an unused name, so the composer still named the call and nothing reached the reader. A call site has three independent ways to be hollow: the call absent, the result discarded, and the argument a constant that makes the call decide nothing. The census now asserts all three and has a companion per shape. Found only by applying the break by hand; reading the census had already declared it sufficient. | open |  | 2026-09-05T18:22:42.457Z |  |
| 107 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-03-PLAN.md |  | Three of this plan's premises were wrong. It says to reach the fact from the message-open path outside the look_at_message_contents gate and to write a census anchored on that setting's arm; the gate runs on a worker at body-fetch time and writes a verdict into a column, the bar is built later from the stored row, and nothing on the display path reads the setting, so the prescribed census would have read an unrelated function. It says to fold the sentence in under whatever the bar already says; said_before_the_message cuts at HOW_IT_WAS_CHECKED, which a signature verdict inserts, so an appended sentence is in the bar and spoken by nothing. And its guard-record table says reader_text.rs is fingerprinted by no record and holds 76 tests, where one record names it and it held 81, both figures true before 04-01 landed hours earlier. Third plan running in this phase whose record table expired against a same-day sibling. | open |  | 2026-09-05T18:22:42.866Z |  |
| 108 | 04 | unrun-verify | src/presentation/wx_reader.rs |  | Whether a StaticBitmap in a reader tab is reachable by a screen reader at all is the question this feature rests on and no test here can ask. A bitmap is not focusable by default on Windows, so NVDA may meet it only in browse mode or not at all, and the accessible name set through set_accessible_name may never be spoken. If it is not reached, the picture serves a sighted reader and is silent for everybody else, which is the half of the feature Pratik ordered first. Only a real NVDA and Narrator run settles it. Tab order is now warning bar, text, picture, attachment list. | open |  | 2026-09-05T23:21:54.325Z |  |
| 109 | 04 | unrun-verify | src/presentation/wx_reader.rs |  | Nothing in this repository has ever checked that a decoded attachment is drawn at a sensible size, is legible against either theme, or that a very wide picture does not push the attachment list off the tab. The bitmap is set to AspectFit for exactly that reason and the reasoning has never met a window. The theme sweep checks its background and foreground colours, which is not the same as somebody looking at it. A picture shown badly is not the same as a picture shown. Only an eye settles it. | open |  | 2026-09-05T23:22:20.234Z |  |
| 110 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Whether the first lines of a preview tab are announced at the moment it opens, or whether somebody has to go looking for them, is untested. The whole design rests on the description being heard first: it is the first line of the text control and the caret starts at the top, and the opening announcement says the title and the attachment summary rather than the note. So somebody who hears the announcement and does not read on may never meet the sentence that says whether the picture came with a description. The same question applies to a text attachment's note, which says whether the file was cut. Only a listener settles it. | open |  | 2026-09-05T23:22:20.648Z |  |
| 111 | 04 | unrun-verify | src/service/plain_text.rs |  | Whether real senders' text attachments are UTF-8 has never been measured and it decides how often the not-entirely-text sentence fires. A log or a CSV written on a Windows machine in an older code page decodes as mostly replacement characters, and this refuses to guess at an encoding on purpose. If most real text attachments are not UTF-8, most previews will be a screenful of replacement characters with an honest sentence above them, which is worse than it sounds and would argue for encoding detection. No mail account has ever been used with this program, so it cannot be answered here. | open |  | 2026-09-05T23:22:21.063Z |  |
| 112 | 04 | unrun-verify | src/service/picture.rs |  | Whether real senders put Content-Description on image parts at all decides whether an image preview usually says what is in the picture or usually says nothing is known about it. Unmeasurable here, and unchanged since the research raised it as assumption A5. Now that the picture is also drawn, the cost of the answer being no falls on a blind reader alone rather than on everybody, which makes it more worth measuring rather than less. | open |  | 2026-09-05T23:22:21.453Z |  |
| 113 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-04-PLAN.md |  | Two of this plan's premises could not be followed as written. Its guard table says reader_text.rs is fingerprinted by no record and theme_reach.rs by three; four records name the first and none name the second, both wrong within hours of the plan being written, which is the fourth phase-04 plan whose record table expired against a same-day sibling. And it asks the task 2 RED commit to name both a new test saying a picture can be read here and the existing test saying it cannot; those are opposite assertions about one function, so exactly one can be red and the gate holds a red commit to every named test really failing. The gate was widened at the red instead, which makes the existing assertion the red one, and the commit says so. | open |  | 2026-09-05T23:22:21.828Z |  |
| 114 | 04 | stub | tests/house_style.rs |  | Corrected 2026-09-06, hours after being written and by the plan revision that read the code. A check DOES compare the model against the screen for top-level settings: every_setting_is_acted_on in src/data/config.rs reads the AppConfig struct itself and test_every_setting_somebody_can_change_is_offered_by_a_screen asserts every public field is offered, so a new top-level setting fails on arrival. That correction then misdiagnosed the remaining gap as nesting, and phase 6 research went to look on 2026-09-06 and found neither exception is nested. allowed_per_account is a TOP-LEVEL AppConfig field at config.rs:276, fully visible to the check and excused by name through STORED_AND_OFFERED_BY_NOTHING at config.rs:1799, a list holding exactly one entry. The per-event feedback channels are a third shape no name-based check can reach: they live inside the SERIALISED STRING VALUE of feedback_channels, so there is no field to find at any depth. Therefore the work this entry originally described, widening the check to follow nesting, closes NEITHER exception and should not be done as stated. What closes them is a hand-named companion on the pattern of test_whether_message_text_may_be_fetched_is_offered_by_a_screen at config.rs:1829, costing two guard records rather than disturbing all five tests in that module. Second-order trap from the same research: emptying STORED_AND_OFFERED_BY_NOTHING makes test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing at config.rs:1947 iterate over nothing and pass unconditionally, which is the census-emptying failure CLAUDE.md already documents, so whoever offers the per-account answer from a screen retires that guard in the same commit. The original entry claimed nothing enforced this at all, written straight after the rule without going to look, which is the mistake the rule itself is about. Two settings break the rule today and both predate it: per-event feedback channels, whose per_event field and set_event_channels are private so no screen could write one, which is FEEDBACK-01 and phase 6's; and the per-account Allow Changes answer. Phase 1's criterion 8 already said a phase must not add a third. Writing the check means enumerating the settings surface and the screen's sections and asserting every member of the first appears in the second, the way tests/one_sign_in_per_piece_of_work.rs counts sign-in sites and names each. Recorded 2026-09-06 when Pratik made the rule explicit. | open |  | 2026-09-06T00:04:31.844Z |  |
| 115 | 04 | unrun-verify | src/service/safety.rs |  | Whether a warning bar carrying attributed sentences from two or three sources is heard as separate facts or as one run-on. Every sentence now names which of the four checks reached it, and this program's own reading says everything it found in one sentence so attribution does not become repetition. That the structure is right is asserted; that it is heard right is not, and only NVDA or Narrator on a real message settles it. Measured bar, three sources: 'This message was marked as spam. Your mail provider's filter marked it as spam. Your mail provider put it in the junk folder. Wixen Mail read this message on your computer and found a link that says it goes one place and goes somewhere else.' | open |  | 2026-09-06T04:45:08.263Z |  |
| 116 | 04 | unrun-verify | src/application/filters.rs |  | Whether 'Safety' is recognised as being about spam and phishing when it is read out as one of twelve field names in a rule editor. The words were chosen to match the message list's own column header rather than inventing a second name for one thing, which is the right trade if somebody has met that column and the wrong one if they have not. Only a screen reader run through the rule editor's field list settles it. | open |  | 2026-09-06T04:45:16.983Z |  |
| 117 | 04 | unrun-verify | src/service/safety.rs |  | Whether provider spam headers arrive in the shapes from_headers expects. Every parser in it is tested against hand-written header blocks and no account has ever been used with this program, so X-Spam-Flag, X-Spam-Status, X-Forefront-Antispam-Report, X-Microsoft-Antispam and Authentication-Results are all unverified in the field. Gmail in particular tells an IMAP client almost nothing beyond moving the message, which is why the folder counts as a verdict. Unchanged by this plan and now more load-bearing, because a filter rule can act on the answer. | open |  | 2026-09-06T04:45:17.430Z |  |
| 118 | 04 | stub | src/application/mail_sync.rs |  | A filter rule that deletes is not said out loud. say_what_the_rules_did produces '{n} sorted by your rules' and carry_out counts a delete in that number alongside a rule that only marked something read, so nothing anywhere says a message was deleted by a rule. Found while adding the safety verdict as a rule field, which is what makes it worth recording now: T-04-19 is a user rule filing wanted mail out of sight on a verdict a sender can shift, and the mitigation the plan asked for was that the rule says what it did. The deletion is local only, cache.delete_message, not a server delete, so mail is hidden rather than destroyed; that lowers the severity and does not close it. Recorded as a finding rather than fixed, as 04-05-PLAN asked. The fix is a count of its own in Filtered and a sentence in say_what_the_rules_did. | open |  | 2026-09-06T04:45:31.615Z |  |
| 119 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-05-PLAN.md |  | Five premises could not be followed as written, and the first changed the size of the job by an order of magnitude. The plan and 04-RESEARCH both said CachedMessage.safety was already on the struct the matcher is handed, citing messages.rs:357; that line is inside listing_row, which builds MessageListRow, whose own doc says 'Deliberately not CachedMessage' three lines above it. CachedMessage had no such field, so criterion 6 needed a new field, 36 construction sites, and two SQL reads that had never selected the column (get_message, which the arrival path uses, and scan_query, which saved searches use). Also: the quiet.len() assertion reddens at GREEN and can never be in a RED commit's trailers; the guard table says body_safety.rs has 0 records and 13 tests (really 1 and 20) and mail_sync.rs 8 and 134 (really 9 and 135); both verify commands are invalid cargo ('--lib' cannot be used multiple times); and premise 5 says the provider sentences end at a common phrase when three of them begin with it and the two Authentication-Results sentences named nobody at all. This is the fifth phase-04 plan carrying a wrong premise. All five are written into the plan file. | open |  | 2026-09-06T04:45:32.068Z |  |
| 120 | 04 | unrun-verify | src/presentation/editor_document.rs |  | Whether NVDA announces the browser engine's spelling marks in a WebView2 contenteditable in this application at all. That is the premise of the two clauses of criterion 3 that already shipped before this plan, and nothing in this tree can test it. If it does not, marking as you type is decorative and the walk keys are the only spelling check that reaches a screen reader. | open |  | 2026-09-06T07:00:00.122Z |  |
| 121 | 04 | unrun-verify | src/presentation/wx_compose.rs |  | Whether this program's landing sentence and the screen reader's own announcement of the marked word collide when the caret lands on a misspelling. Two voices for one fact, arriving in the same instant: the selection change is what makes the reader speak the word, and walk_to_a_misspelling speaks its own sentence immediately afterwards. Whether that is heard as one answer or as an interruption is not testable here. | open |  | 2026-09-06T07:00:18.344Z |  |
| 122 | 04 | unrun-verify | src/presentation/accessibility/feedback.rs |  | Whether the earcon at the end of a mistyped word and the landing announcement of the walk keys are told apart. Both are about a misspelling and both fire while somebody is working in the message body; guardrail 5 asks for feedback that is distinct, and nothing here can say whether these two are. | open |  | 2026-09-06T07:00:18.882Z |  |
| 123 | 04 | unrun-verify | src/application/spell_session.rs |  | Whether three suggestions is heard as helpful or as a list to sit through, and whether '7 suggestions in all' is heard as useful or as noise. The bound is a judgement written into SUGGESTIONS_SAID with its reasoning; only listening settles whether it is the right number. | open |  | 2026-09-06T07:00:19.316Z |  |
| 124 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-06-PLAN.md |  | The plan's two tasks are not separable as written: a forward key and the same key with Shift are one if, one message field and one match with two arms, so task 1's minimum finishes task 2 and task 2's required inverse property test cannot be red. Task 1 was narrowed to have no direction parameter anywhere before task 2 could carry one. Also: both verify commands are invalid cargo, repeated from 04-05 after that summary reported them; premise 2 is right that enumeration and the caret move ship, and misses that nothing could report where the caret is, which is new page code; and the sentence for a word with more suggestions than the bound is unreachable unless the speller is asked for more than the bound, which the plan does not name. Premise 8's guard table is right in every row, the first time in two phases. | open |  | 2026-09-06T07:00:19.717Z |  |
| 125 | 04 | stub | tests/wired.rs |  | documented_combinations only collects a backticked key that starts with Ctrl+ or Alt+, so a documented combination spelled Shift+Alt+F7 rather than Alt+Shift+F7 is skipped in silence: no exception entry is needed for it and no protection is given either. The check exists because three documented keys were dead at once, and the order somebody writes the modifiers in decides whether it looks at all. | open |  | 2026-09-06T07:00:20.122Z |  |
| 126 | 04 | unrun-verify | src/application/attaching.rs |  | Whether a batch announcement naming six files is heard as a confirmation or as something too long to sit through. NAMED_ALOUD is 6 on the reasoning that six names is about ten seconds of speech; nothing here can say whether ten seconds is right, or whether the names should be dropped entirely in favour of the count and the total. | open |  | 2026-09-06T08:32:55.803Z |  |
| 127 | 04 | unrun-verify | src/application/attaching.rs |  | Whether the refusal for a folder among five files is told apart from the names of the four that went on. Two sentences arrive one after the other, the first naming what is attached and the second naming what is not, and only listening says whether that reads as two facts or as one long list. | open |  | 2026-09-06T08:33:15.056Z |  |
| 128 | 04 | unrun-verify | src/presentation/wx_compose.rs |  | Whether Ctrl+V on the attachments list or the toolbar is discoverable at all by somebody who has never read the shortcuts document. Nothing in the composer says the key exists: it is not on a button label, not in an accessible name and not announced. The picker is discoverable and this is the quicker route, so a key nobody finds is a keyboard equivalent that only exists on paper. | open |  | 2026-09-06T08:33:15.494Z |  |
| 129 | 04 | unrun-verify | src/presentation/wx_compose.rs |  | Whether a file dropped on the composer attaches at all. The drop target is installed on the dialog and hands its paths to the same door the picker uses, and whether a drop over the message body reaches it is unknown: the body is a WebView2 control that handles drops in its own window. Task 3 of plan 04-07 is a person dragging a file to settle it, and the answer goes in the product either way. | open |  | 2026-09-06T08:33:15.932Z |  |
| 130 | 04 | stub | src/common/error.rs | 58 | Error::Other displays as 'Error: {message}', so every refusal this program says out loud opens with the word Error before the sentence. Found by a test pinning the composer's single-file refusal against the old code rather than against a copy of it. Poor wording for a screen reader and unchanged here, because it is a change to common::Error and to every announcement that goes through it. | open |  | 2026-09-06T08:33:16.343Z |  |
| 131 | 04 | deviation | .planning/phases/04-writing-and-reading-a-message-in-full/04-07-PLAN.md |  | Four premises corrected. Premise 5 says the paste key needs a second home because the attachments list is hidden when empty; it needs no second home, because wxWidgets passes an unhandled key up the parent chain and the composer already uses one dialog-level handler for exactly this, so Ctrl+V is bound once there. The plan's tests/wired.rs change is unnecessary: Ctrl+V is already documented and already bound as a menu accelerator in the main window, so bound_somewhere finds it and no exception entry is needed, which also means that check gives the composer's Ctrl+V no protection at all. The threat register asks for fixtures with a traversing name and a reserved Windows device name; neither can exist as a real file on Windows, and what protects those cases is Chosen::at asking for the last component and safe_file_name prefixing a device name, both tested where they live. No version bump in task 2: there is nothing true to write in a changelog entry until task 3 answers whether a drop lands, and this project pairs a bump with an entry. Premise 8's guard table is right in every row: 617 records, attaching 0, wx_compose 2, wired 8, attachment_name 0. | open |  | 2026-09-06T08:33:16.753Z |  |
| 132 | 04 | unrun-verify | src/presentation/editor_document.rs |  | Whether a file dragged onto the composer reaches anything at all. The page now turns a drop on the message body away so the engine cannot navigate to the file, and the composer says where a drop does land, and both rest on documentation rather than on somebody watching: WebView2's AllowExternalDrop defaults to true so the drag reaches the page, and the OLE drag loop walks up the parent chain so a drop on the subject line, the toolbar, the attachments line or the list should reach the dialog's target. Nobody has dragged a file onto a running composer. The four observations to make are in 04-07-SUMMARY.md. | open |  | 2026-09-06T10:49:18.354Z |  |
| 133 | 04 | deviation | Cargo.toml |  | Upstream gap, not reported yet: WebView2's AllowExternalDrop cannot be turned off through this stack. It lives on ICoreWebView2Controller4; wxWidgets 3.3.2 keeps the controller in wx/msw/private/webview_edge.h and wxWebViewEdge::GetNativeBackend returns the ICoreWebView2_2 underneath it, which cannot be asked for its controller, so wxdragon 0.9.17 has nothing to expose. The reachable mitigation is the page refusing the drop itself, which depends on the page's script having loaded and so is weaker than the host-level setting would be. Worth an issue against wxWidgets or wxdragon. | open |  | 2026-09-06T10:49:28.613Z |  |
| 134 | 04 | unrun-verify | src/application/attaching.rs |  | Whether the sentence said when a file is dropped on the message area is heard as an answer or as an interruption. It arrives spoken at high priority and shown in a message box at the same time, three sentences long, at the moment somebody let go of a file. Nothing here can say whether that is help or a modal in the way, or whether naming Ctrl+V and Attach File in speech is how somebody finds them. | open |  | 2026-09-06T10:49:29.074Z |  |
| 135 | 04 | stub | src/presentation/html_renderer.rs |  | The setting that says where a decorative picture is reaches the formatted reading path only. html_to_plain_text strips every tag, so in the plain text reading path no picture says anything at all, described or decorative, and this setting is inert there. Older than 04-08 and not fixed here: it changes what every plain text reader hears on every message with a picture in it, so it needs its own red, its own green and its own changelog line. | open |  | 2026-09-06T13:50:55.540Z |  |
| 136 | 04 | unrun-verify | src/presentation/wx_compose.rs |  | Nobody has heard the decorative question read aloud. It is a Yes/No box whose text runs to four lines and names what each answer does, and the buttons say only Yes and No. Whether somebody hearing it understands that Yes sends a picture with nothing said, whether four lines is too much to hold while reaching for a button, and whether Enter arriving on No is felt as safe or as an obstacle, are all things only a real screen reader run can settle. | open |  | 2026-09-06T13:51:18.753Z |  |
| 137 | 04 | unrun-verify | src/presentation/wx_settings.rs |  | The Say where a picture the sender marked decorative is check box has not been heard. Its label carries the mnemonic and set_accessible_name_and_description names it on the MSAA channel, but nothing here can say whether Narrator reads the label from the window text, whether NVDA reads the accessible name rather than the neighbouring text, or whether the description saying what off means is reached at all. | open |  | 2026-09-06T13:51:19.324Z |  |
| 138 | 04 | unrun-verify | src/application/pictures.rs |  | Nobody has heard a mailing with thirty spacers in it. Announcing decorative pictures ships on, and guardrail 5 forbids feedback that floods. The words are short and go into the document rather than into the announcement queue, so they are passed over rather than spoken at, but whether a bulk mail template that marks twenty layout images decorative turns a message into a wall of the same phrase is unmeasured. | open |  | 2026-09-06T13:51:19.876Z |  |
| 139 | 04 | unrun-verify | src/application/pictures.rs |  | The furniture threshold is a judgement with no field data behind it. A shorter side of 200 pixels and 100 KB on disk were chosen from what furniture is, not from measuring real mail. Nobody has run it over a real mailbox to see how often the decorative question is offered over something that is not furniture, or refused over something that is. | open |  | 2026-09-06T13:51:20.326Z |  |
| 140 | 04 | unrun-verify | src/application/pictures.rs |  | No decorative picture has been sent to a real recipient. Whether an inline picture sent as multipart/related with an empty alt arrives at Gmail or Outlook with that alt intact, and whether their readers then skip it, has never been tested, because no message from this program has ever reached anybody. | open |  | 2026-09-06T13:51:20.786Z |  |
| 141 | 04 | unrun-verify | src/service/signed_mail.rs |  | EncryptedMessage::read has never met a real envelope from a real sender. Its fixture is real OpenSSL output, which is more than hand-built DER and is not mail: nobody has put an enveloped message from Outlook or Thunderbird through it. If one does not parse, the reader says the message is encrypted and its details could not be read, which is a wrong message rather than an absent one, and that is worse than the blank body it replaces. The refusal path is what bounds it and it is tested against every seventh-byte prefix of the fixture. | open |  | 2026-09-06T17:25:10.804Z |  |
| 142 | 04 | unrun-verify | src/presentation/reader_text.rs |  | Nobody has heard an encrypted message open. Three things only a screen reader run settles: whether the sentence in the body is reached before somebody concludes the message is broken, whether hearing the same sentence in the bar as the message opens and again in the body is heard as thoroughness or as repetition, and whether the count in it addressed to 1 certificate is understood at all by somebody who has never been told what a certificate is. | open |  | 2026-09-06T17:25:20.115Z |  |
| 143 | 04 | unrun-verify | src/service/signed_mail.rs |  | Whether this computer holds a certificate an encrypted message was addressed to has never been asked of a real Windows certificate store holding a real S/MIME certificate. which_recipient_is_us is exercised against a store held in memory, so the three answers are tested and the Windows path that produces them is not. A store that answered wrongly rather than failing would tell somebody a private message was not meant for them. | open |  | 2026-09-06T17:25:20.633Z |  |
| 144 | 04 | unrun-verify | src/service/pgp/keys.rs |  | No real key and no real PGP message have been through this. The fixtures are a key pair and a message made by GnuPG 2.4.9 rather than by rPGP, which is two implementations agreeing rather than one agreeing with itself and is the strongest evidence available here, and it is still not a message from a correspondent. What goes wrong if it is not enough runs both ways and both are worse than the armour this replaces: a message shown as opened that was not, or a message refused that another client opens. | open |  | 2026-09-06T19:22:31.977Z |  |
| 145 | 04 | stub | src/application/opening_pgp.rs |  | PGP/MIME is not read. what_the_form_says reads the message's text parts, so only inline PGP, where the armour sits in the body, ever reaches the opener. A multipart/encrypted message puts the armour in a separate application/octet-stream part that never becomes body text, so such a message is neither opened nor reported as failing to open: nothing sees it. Thunderbird and most modern clients send PGP/MIME, so this is the common shape rather than a corner. Gated in the product by the experimental warning on the menu item and named in the changelog. | open |  | 2026-09-06T19:22:32.467Z |  |
| 146 | 04 | unrun-verify | src/presentation/wx_app.rs |  | Nobody has heard the PGP import or any of its five answers read aloud. The import is a file picker followed by one announcement at High priority, and whether the sentence for a public key, a locked key, a file that is not a key, a refused credential store or a successful import is understood on hearing it once, with no dialog to go back to, is a thing only a real screen reader run settles. | open |  | 2026-09-06T19:22:32.974Z |  |
| 147 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | Nobody has heard the hold announced. Pressing Send now says "Sending in 10 seconds. Undo Send takes it back." and whether that sentence finishes in time for somebody to hear it, decide and press Ctrl+Shift+Z inside ten seconds is the whole argument in Hold::DEFAULT's doc and no test can measure it. Plan 04.2-04's checkpoint, item 4, asks this question in the flow it matters most in. | open |  | 2026-09-06T23:11:30.841Z |  |
| 148 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | A message that leaves after a hold has never met a real server. Whether it arrives with the headers a recipient's client expects, and whether ten seconds feels long or short with a real mailbox syncing underneath, are both unsettled. | open |  | 2026-09-06T23:11:39.000Z |  |
| 149 | 04.2 | unrun-verify | src/presentation/wx_send_later.rs |  | Nobody has heard the Schedule window. Whether a month choice, a day spinner, a year spinner, an hour spinner, a minute spinner and sometimes a morning-or-afternoon choice are heard as six separate named controls, each saying its own value as it changes, is the question wx_item_form.rs's module doc settled with a real screen reader session for that dialog and which has not been settled for this one. The control shape is the same and the names are set with set_accessible_name rather than set_name, so the expectation is that it carries over. That is an expectation, not a measurement. | open |  | 2026-09-06T23:59:00.000Z |  |
| 150 | 04.2 | unrun-verify | src/presentation/wx_send_later.rs |  | Nobody has heard a refusal in the Schedule window. When a time will not do the window stays open, the reason is put in its problem line and announced at High priority through said_and_shown, and focus moves to the month control. Whether the sentence is actually heard, or is lost under whatever the screen reader says about the control focus just landed on, is the failure mode this pairing exists to avoid and only a real run settles it. | open |  | 2026-09-06T23:59:10.000Z |  |
| 151 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | No message has ever waited hours for its time and then gone. Every test moves the clock; nothing runs the program for an afternoon. Whether a scheduled message really leaves when its moment arrives with the program left running, whether the poll timer is still asking after hours, and whether a message written on Monday and sent on Tuesday carries headers a recipient's client accepts, are all unsettled and none of them can be tested here. | open |  | 2026-09-06T23:59:20.000Z |  |
| 152 | 04.2 | unrun-verify | src/application/attaching.rs |  | No organiser's calendar has ever received a reply from this program. The part now leaves declared text/calendar; charset=utf-8; method=REPLY and named reply.ics, asserted over the whole path from the answer the window builds to the Ready the send loop puts on the wire, but nothing here has met a real mail server. Whether Outlook, Google Calendar or Thunderbird actually folds the answer into the meeting and updates the guest list is what the whole change is for and is the one thing no test in this repository can settle. | open |  | 2026-09-07T03:43:34.557Z |  |
| 153 | 04.2 | unrun-verify | src/application/attaching.rs |  | Nobody has forwarded an invitation from this program to a real recipient. A .ics attachment is now declared method=REQUEST or method=CANCEL from what the document says, which changes what a recipient's client offers them, and no client has ever been shown one. Whether a forwarded invitation really presents as a meeting to answer, and whether the ORGANIZER inside it is attributed to whoever called the meeting rather than to the forwarder, are unsettled here. | open |  | 2026-09-07T03:43:43.233Z |  |
| 154 | 04.2 | unrun-verify | src/application/answered_meetings.rs |  | The accepted meeting is written with pending set, which is what puts it in front of the push, and no push path in this project has ever run against a real account. Whether a CalDAV, Google or Microsoft calendar accepts an event this program created from an invitation, with the invitation's own UID as the provider identity and no etag, is unsettled. If a provider refuses it the meeting stays correct on this computer and never reaches any other device, which reads to the person exactly like it working. | open |  | 2026-09-07T05:52:49.797Z |  |
| 155 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | Answering a meeting has not been heard with a screen reader. Four things are unsettled by ear: whether accepting is announced once or twice now that the answer is filed as well as sent; whether the meeting is read out with its time and its busy state when the calendar is opened afterwards; whether a declined meeting is read as free rather than booked; and, since pressing Accept sends with no confirmation step by the decision of 2026-09-06, whether anything spoken in the seconds after pressing it by mistake points at Undo Send. The last is the one structure cannot show, and a finding of nothing pointed at Undo Send is a result worth having rather than a failed run. | open |  | 2026-09-07T05:52:58.235Z |  |
| 156 | 04.2 | unrun-verify | src/presentation/html_renderer.rs |  | The sentence about held-back pictures has not been heard with a screen reader. It is placed above the message body, after the message's own heading in a conversation, and read in order it arrives before the thirty markers it is about. Whether it is heard as orientation or as one more thing in the way is the only question here that structure cannot answer, and the answer changes the placement rather than the words: under the heading instead of above the body, or at the end, or in the announcement path instead of the document. Thirty markers in a marketing message is the case to try it on. | open |  | 2026-09-07T07:03:59.342Z |  |
| 157 | 04.2 | unrun-verify | src/presentation/wx_compose.rs |  | One hop is uncovered and it is the hop this plan is about. wx_compose's preview now asks for HtmlRenderer::for_a_message_being_written, and no test drives that window, so changing that one line back to new() reddens nothing. It was measured rather than assumed: the guard record breaking the constructor reddens a test, and a record breaking the call site would redden none. The constructor's own answer is pinned by a test, and the default a caller gets by saying nothing is the reading one, so a new call site that forgets is wrong in the direction that tells a reader too much rather than a writer something false. What is unverified is that this particular call site still asks. | open |  | 2026-09-07T07:04:11.587Z |  |
| 158 | 04.2 | unrun-verify | src/presentation/html_renderer.rs |  | test_a_message_with_nothing_held_back_says_nothing was rewritten against the document and was green on arrival, because at the time of the rewrite no document said the sentence at all. It is the assertion that stops an ordinary message growing a line about pictures nobody held back, and it has never been red. Taking the emptiness check out of what_a_reader_is_told_was_held_back by hand would settle whether it would notice; it was not done. | fixed |  | 2026-09-07T07:04:12.210Z | 2026-09-07T07:21:59.339Z |
| 159 | 04.2 | unrun-verify | src/presentation/wx_blocked_senders.rs |  | Whether each row of the blocked list is read as a person, a destination and a state, or as one run-together string. This is a Report ListCtrl and wxdragon 0.9.17's get_item_text loses the last character of every cell and returns a NUL in its place, which is ledger 28 and is upstream and unfixed. If it shows here it is somebody else's defect and it still lands on the person using this. Nothing in this repository can answer it: only NVDA and Narrator on a running build can. | open |  | 2026-09-07T09:09:03.008Z |  |
| 160 | 04.2 | unrun-verify | src/presentation/wx_blocked_senders.rs |  | Whether what unblocking did is heard over the list being re-read underneath it. The list is refilled and the row cursor is landed again before the sentence is announced, so a screen reader has a repainted control and a notification arriving close together. Whether the sentence is heard once, heard twice, or lost under the repaint is a question only a real run answers, and it decides whether the announcement should come before the refill rather than after. | open |  | 2026-09-07T09:09:17.147Z |  |
| 161 | 04.2 | unrun-verify | src/presentation/wx_blocked_senders.rs |  | Whether a switched-off block is distinguishable by ear from a working one. The state is a third column reading Working or Switched off, so it is catching nothing. Looking at the screen makes the difference obvious; hearing a row read cell by cell may not, and still_on's own doc says a list showing a switched-off block as working would be worse than no list. Also unheard: whether an account with nothing blocked is heard as empty on purpose rather than as a window that failed to load, which is what the sentence and the focus going to Close are for. | open |  | 2026-09-07T09:09:17.667Z |  |
| 162 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | Whether the sentence saying what blocking will do is heard before the block is made, and whether it and the may_block mailing-list warning together read as one thing or two. Both are said through the same told at Priority::High, one after the other, and this is the only place in this feature where somebody hears two sentences before a block. Guardrail 5 is what it is about: feedback must be distinct and bounded, and a sentence added before a block is the easiest place here to flood somebody. Announcements go out through UiaRaiseNotificationEvent, which reaches speech and braille at once, so a braille display should be checked as well. | open |  | 2026-09-07T09:09:18.186Z |  |
| 163 | 04.2 | unrun-verify | src/presentation/scan_target.rs |  | Neither automated accessibility channel has been run over the new blocked-senders window. The plan's checkpoint asks for two, separately: Axe.Windows over UI Automation with the new blocked-senders scan target, confirming the scan really opened the window rather than reporting nothing about one it could not reach, and scripts/msaa-names.ps1 over the same window, which is what NVDA reads for native controls and the only channel set_accessible_name writes to. A name failing on either is a name somebody does not hear. There is also a trap in the other direction: a control with a visible label beside it inherits that label as its MSAA name even when nothing set one, so for each control the run has to say whether the name that came back is the one this code set or one Windows supplied. Deferred by decision, not attempted. | open |  | 2026-09-07T09:09:27.449Z |  |
| 164 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | Whether Shift+F6 reaches the script injected into the message preview with shiftKey set. Nothing here has run the preview: the direction is proved from the payload inward, by tests over panes::leaving_which_way and panes::leaving_the_preview, and by a source read of the script and the handler. What no test can reach is the browser delivering the keystroke. The composer's own measurement in editor_document.rs records that Ctrl+backslash never arrives at its page handler on this machine while Ctrl+Shift+L and Ctrl+Enter beside it arrive every time, so a key being kept between the window and the page is a thing that happens here. F6 leaving the preview is known to work; Shift+F6 arriving as F6 with shiftKey true is not. | open |  | 2026-09-07T12:37:49.096Z |  |
| 165 | 04.2 | unrun-verify | src/presentation/panes.rs |  | Whether landing back on the folder tree with Shift+F6 is heard as going back, or only as going somewhere. Arrival is announced with panes::arrival, which names the pane and what is in it and says nothing about direction, so F6 to the folder tree and Shift+F6 to the message list are announced the same way as any other arrival. Somebody who cannot see the layout may not be able to tell that the key went the way they asked, which would leave the fix invisible even though it works. Announcements go out through UiaRaiseNotificationEvent, reaching speech and braille at once, so a braille display wants checking too. This decides whether an arrival should say a direction, which would be a change to every route rather than to this one. | open |  | 2026-09-07T12:38:01.449Z |  |
| 166 | 04.2 | deviation | .planning/phases/04.2-what-was-built-and-never-reached/04.2-07-PLAN.md |  | Task 3 carried two acceptance criteria that cannot both hold. One requires grep -rn 'never takes focus' src/ docs/ to return nothing; the other requires docs/accessibility.md to be unchanged by the plan, and that file held the phrase at line 159. The plan's prose counted three places and the tree held four. The exclusion was written for a different sentence in that file, a braille one belonging to phase 6, so it was written without knowing the file also carried the phrase being swept. Resolved in favour of the grep: the false sentence was corrected and the braille sentence left alone. | open |  | 2026-09-07T12:38:02.044Z |  |
| 167 | 04.2 | unrun-verify | src/presentation/wx_app.rs |  | Whether a message list rearranging itself as somebody moves between folders is announced at all. Moving from the inbox to Sent now changes which columns are shown and how the list is sorted, and nothing says so: the columns are the control's own headers, which a screen reader reads from the control when asked rather than when they change, and no announcement is made. So a list may quietly become a different list under somebody who cannot see it. Whether that needs saying, and what it should say without flooding somebody arrowing through a folder tree, is a judgement only a screen reader run makes. | open |  | 2026-09-07T15:17:10.616Z |  |
| 168 | 04.2 | unrun-verify | src/presentation/wx_columns.rs |  | Restore Defaults still announces the fixed sentence 'Columns reset to the default', which does not say which folder's defaults arrived. It now really does restore the ones for the folder somebody is in, and the two kinds differ by a whole column and by which date the list is sorted on, so the same six words describe two different outcomes. Whether somebody who cannot see the list can tell which they got, and whether the sentence should name the folder kind, is unverified by ear. | open |  | 2026-09-07T15:17:23.347Z |  |
| 169 | 04.2 | deviation | src/presentation/message_columns.rs |  | A build older than 0.85.0 reading a layout this build wrote keeps its columns and falls back to that folder's default sort. The kind is appended after an @ on the end of the stored string, which puts it inside the field an older build reads its sort from. The second sort level was free in both directions because a semicolon inside the sort field is what an older parser stops at; this field cannot be. Accepted rather than fixed, and written into to_stored's own doc. Putting the kind into the columns list instead would have cost nothing in either direction and was rejected as smuggling a non-column through a list of columns. | open |  | 2026-09-07T15:17:23.911Z |  |
| 170 | 04.2 | deviation | .planning/phases/04.2-what-was-built-and-never-reached/04.2-08-PLAN.md |  | One RED assertion had to be respelled in the GREEN commit. test_a_sent_layout_and_both_its_levels_come_back_as_a_sent_layout asserted back.kind == FolderKind::Sent against a two-argument from_stored, because a test that does not compile is not a red and red-commit.sh refuses one. The fix changed ColumnLayout::kind to Option and from_stored to one argument, so the same claim is now spelled Some(FolderKind::Sent) against from_stored(..).expect(..). The claim did not change and the test was really red for the right reason; the bytes did. Inherent to any red written about a type that does not exist yet, and worth a name so the next plan does not read the diff as the test being edited to pass. | open |  | 2026-09-07T15:17:35.182Z |  |
| 171 | 04.2 | unrun-verify | src/presentation/editor_document.rs |  | Whether F8 really reaches the composer's toolbar on a machine other than the one where it was watched working, and whether Ctrl+backslash fails everywhere or only here. The project measured both once, on one machine, and recorded it in editor_document.rs's own comment. docs/KEYBOARD_SHORTCUTS.md now gives F8 as the way in and marks Ctrl+backslash as bound and not seen to arrive, which raises what a second machine disagreeing would cost: a page giving the wrong key is the exact defect this plan closed, and it would be closed in the wrong direction. The check that pairs the page with the bindings cannot tell a key that arrives from one that does not, and says so in its own words. | open |  | 2026-09-07T16:53:28.127Z |  |
| 172 | 04.2 | unrun-verify | src/presentation/wx_compose.rs |  | Neither key this plan wrote into the shortcuts page has been heard. Delete on the composer's attachments list announces Removed and the file's name at Priority::Normal, and whether that is heard over the list refilling under it, and whether the row cursor lands somewhere sensible after a row goes, is the same question ledger 160 asks of the blocked senders list. Closing the conversation window with F6 is the other: the frame is hidden and a timer hands control back to whatever opened it, and nothing here says where focus went or that the window closed at all, so somebody who pressed F6 expecting to move between panes may hear nothing and not know what happened. | open |  | 2026-09-07T16:53:37.371Z |  |
| 173 | 04.2 | deviation | tests/wired.rs |  | The plan asked for the code-to-doc direction in tests/wired.rs to be widened to read editor_document.rs and wx_compose.rs as well as wx_app.rs. It was not, and the reason is a measurement rather than a preference: every key those two files bind is already written in docs/KEYBOARD_SHORTCUTS.md by name, F7 F8 F6 Escape Tab Enter and Delete, so that widening reports nothing at all today and would guard nothing. The class of key that hid F8, Delete and F6 is not one the document never names, it is one the document names for a different surface, and no whole-document reader can see that. The per-surface reading went into tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs instead, which was red on all three. | open |  | 2026-09-07T16:53:46.808Z |  |
| 174 | 04.2 | deviation | tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs |  | The per-surface check covers two surfaces and the shortcuts document describes about thirty. The composer's page and the conversation window are the two that were wrong, and both are now held. Every other surface that gives a key its own meaning is unheld: the reader window's F8 for attachments, the View menu's F8 for the Columns dialog, Delete in the message list, F6 for panes in the main window. Any of those could lose its binding or its documentation and both directions of the pair in tests/wired.rs would stay green, because the key name is in the document for one of the others. Generalising means giving each surface a heading and a reader, which is a real piece of work and is not what this plan bought. | open |  | 2026-09-07T16:53:56.612Z |  |
| 175 | 04.2 | deviation | tests/the_planning_files_agree_with_themselves.rs |  | The roadmap check reads one direction only: every progress-table row must match the phase directory it names. A phase directory holding plans with no row in that table at all is not seen, so the roadmap can go quiet about a phase rather than wrong about it and nothing says so. Closing it means deciding what a directory with no row means, which is not always drift: a scratch directory, or a phase split after the roadmap was written, would both fire. | open |  | 2026-09-07T21:40:00.000Z |  |
| 176 | 04.2 | deviation | .planning/STATE.md |  | progress.total_plans says 87 and there are 85 *-PLAN.md files. It is deliberately not asserted because what it counts could not be established. The roadmap's progress denominators sum to 85, deriveProgressFromRoadmap in the vendored tooling sums exactly those denominators, and a second writer in state-transition.cjs sets the field to whatever an argument passes it. The 87 arrived at b998dfaf on 2026-09-07, in the commit whose message records state.advance-plan being run by accident, over a roadmap whose denominators already summed to 85. So the observed value matches neither reading and the field has at least two writers with different meanings. | open |  | 2026-09-07T21:40:00.000Z |  |
| 177 | 04.2 | deviation | tests/the_planning_files_agree_with_themselves.rs |  | The ledger check compares two of the ten columns each entry carries twice: status and description. A table row whose phase, kind, file, line, reason or either timestamp was edited alone still reverts silently on the next tool write and nothing says so. Two columns were chosen because they are what a person reads and what a repair changes, and the check was measured against the real defect of closing an entry in the table only. Widening it is cheap and was not done, so the limit is written down rather than assumed away. | open |  | 2026-09-07T21:40:00.000Z |  |
| 178 | 04.2 | deviation | tests/the_planning_files_agree_with_themselves.rs |  | The state check holds four facts. current_phase_name, status, stopped_at and state_head are also written in the frontmatter and described in the Current Position prose, and no check pairs them. stopped_at is the one that already went wrong: at c6e08a9 it said 04.2-05 while the heading described plan 7, and only the current_plan half of that divergence is now held. It is not obvious how to pair a prose sentence with a field, which is why the four that are held are the four that are written as numbers or as one token. | open |  | 2026-09-07T21:40:00.000Z |  |
| 179 | 04.1 | deviation | src/presentation/wx_app.rs |  | move_or_copy_message and spawn_folder_move each call owner_of, and the modal dialog runs between the two calls. Both now ask one rule so they agree in every ordinary case, but a sync that replaces the message list while the window is open would leave the second call falling back to the account on screen, which is the divergence this plan closed everywhere else. Passing the first answer to spawn_folder_move would close it and would take that function out of the list tests/wired.rs holds to asking owner_of, and that check exists because this fault has now happened four times, so the narrower window is recorded rather than traded for the wider check | open |  | 2026-09-08T00:34:33.068Z |  |
| 180 | 04.1 | unrun-verify | src/presentation/wx_destination.rs |  | The move and copy window now draws a real tree, with a folder inside the folder it is in, and names accounts the way the sidebar does. No screen reader has heard either. Whether Right and Left expand and collapse as somebody expects in this dialog, whether a folder two deep is reached and read as being inside its parent, and whether hearing Work with an address after it on a branch is a help or a mouthful, are all judgements only NVDA or Narrator settles | open |  | 2026-09-08T00:34:48.878Z |  |
| 181 | 04.1 | deviation | src/data/config.rs |  | The census asking whether every stored setting is read by something now follows one hop through a function of config.rs, because a setting whose stored value holds two facts gets a reader and a writer and stops being named anywhere else. Three limits, recorded rather than narrowed away. It counts a pub fn taking and self whose doc comment merely mentions the field, since the body is taken as everything between one pub fn and the next. It cannot see a setting read by a free function rather than a method. And it counts no function taking mut self, which is right for the pair that prompted it and would be wrong for a setting legitimately read inside a method that also writes | open |  | 2026-09-08T00:34:49.418Z |  |
| 182 | 04.1 | unrun-verify | src/presentation/wx_app.rs |  | Nothing tests that the move window opens on the folder last filed into. The guard record covering that call reddens the settings census rather than anything about the window, so what is defended is that the stored value is read at all and not that the row it names is where the cursor lands. Reaching that needs a live window, a stored settings file and a branch with the remembered folder in it | open |  | 2026-09-08T00:34:49.971Z |  |
| 183 | 04.1 | unrun-verify | src/application/mail_across_accounts.rs |  | No message has been copied between two real accounts. Every assertion here is against a loopback server this project wrote, which answers exactly what the script says and nothing a provider does on top: whether Gmail treats an APPEND with an internal date the way the RFC says, whether a strict server refuses a flag list this drops keywords from anyway, whether a message fetched with BODY.PEEK and appended somewhere else arrives byte for byte, and whether a slow append times out before it lands are all questions only a live account answers | open |  | 2026-09-08T02:10:00.000Z |  |
| 184 | 04.1 | unrun-verify | src/presentation/wx_destination.rs |  | No screen reader has heard the move and copy window with several accounts in it. Three questions: whether an account row reads as an account rather than as a folder, whether a collapsed branch is announced as collapsed with a count of what is inside, and whether the person finds Right without being told. The accessible description names Right and Left, which is structure present rather than experience good | open |  | 2026-09-08T02:10:00.000Z |  |
| 185 | 04.1 | unrun-verify | src/presentation/wx_destination.rs |  | Whether Enter chooses in this window is unverified and always has been. A TreeCtrl takes Enter as an item activation, and whether that reaches the dialog's default button was read off the code rather than pressed. The accessible description and docs/KEYBOARD_SHORTCUTS.md both say Enter chooses. CLAUDE.md already records one key bound in the composer that was measured never arriving, and no reader of source text can tell that case from a key that works | open |  | 2026-09-08T02:10:00.000Z |  |
| 186 | 04.1 | deviation | src/application/mail_across_accounts.rs |  | The transcript half of test_a_copy_across_accounts_says_nothing_to_the_source_that_changes_it cannot be made to fail. What it asserts, that no STORE, EXPUNGE or COPY reaches the source, is guaranteed by the trait the source is behind rather than by the code under test: TheAccountItIsIn has two reads and no write, so no body of copy_it_across can send one. That is a stronger guarantee than the test, and it means the test only starts measuring anything if somebody widens the trait. Its other assertion, that the copy succeeded, was taken red | open |  | 2026-09-08T02:10:00.000Z |  |
| 187 | 04.1 | unrun-verify | src/application/mail_across_accounts.rs |  | No message has been moved between two real accounts. Every assertion is against loopback servers this project wrote, and three questions only a live account answers are named in docs/changelog.md under known limitations: what a real provider does with an upload of a ten megabyte message, what Gmail makes of a message uploaded from another account when it treats a copy as a label, and what any provider does when a message arrives carrying an identifier the destination already holds. The last is not idle, because that identifier is what the program asks about when an upload's answer never arrives. Since 2026-09-19 (11-07.2) the crossing completes here first and is replayed from the queue at a check of either account, and the replay leans on the third question the same way: it reads an arrival only from a number the folder did not hold before, and a real provider's answer is still unmeasured | open |  | 2026-09-08T12:00:00.000Z |  |
| 188 | 04.1 | unrun-verify | src/application/mail_across_accounts.rs |  | The ambiguous append, where the destination stops answering part way through, has never happened against a real server. Both tests that drive it script the answer through a hand-written destination rather than a loopback one, because the only way to make a real connection stop answering is to close it and a closed connection cannot then be asked what the folder holds. What is proved is what the code does with an Error::Network; what is unproved is that a real dropped connection and a real timeout arrive as one | open |  | 2026-09-08T12:00:00.000Z |  |
| 189 | 04.1 | deviation | src/presentation/wx_app.rs |  | A folder somebody has turned syncing off for is still offered by the move and copy picker, and a message moved into it is at the server and never appears in this program. The decision taken is that the move must not turn syncing on, because a setting somebody chose is not something another command changes behind them, and that the fact is said in docs/changelog.md instead. Saying it in the window at the moment of the move would be better and is not done: it needs the destination account's folder_choices read at move time, and the plan's acceptance criteria forbid adding a test to wx_app.rs, which is where that check would live | open |  | 2026-09-08T12:00:00.000Z |  |
| 190 | 04.1 | deviation | src/service/protocols/imap.rs |  | ImapSession::remove_these is refused in the words "replace a saved draft", which is wrong for every caller but the one it was written for. The delete handler already worked around it rather than reuse it, and the cross-account move now does the same by way of a new take_this_one_off whose gate says "move a message". Three callers now avoid one function because of its refusal wording; the wording itself is still not fixed, and fixing it means deciding what a neutral sentence costs the draft path that has the specific one today | open |  | 2026-09-08T12:00:00.000Z |  |
| 191 | 04.1 | unrun-verify | src/presentation/wx_app.rs |  | This program has never been killed part way through a move between two real accounts and started again, so the one case the kept bytes exist for has never happened outside a test. Nor has the question put on the next start ever been heard: whether it is read out at all, whether both answers are reachable by keyboard, whether focus lands sensibly, and whether it reads as something unfinished rather than as an error, when nothing was lost, are all questions only NVDA can answer. The seven steps are written out in 04.1-04-SUMMARY.md, and two of their outcomes stop the phase rather than continuing it: nothing said at all on the next start, and the message arriving twice at the destination | open |  | 2026-09-08T20:00:00.000Z |  |
| 192 | 04.1 | deviation | src/presentation/wx_app.rs |  | The question about an unfinished move is a wxWidgets MessageDialog, which the binding builds from a title and a body and which offers no seam for set_accessible_name_and_description. What a screen reader reads for it is its title and its text, both written here, and this is the house pattern that ask_about_the_folders_that_have_gone already follows. The words are also announced through the announcement queue at high priority, so somebody whose reader was mid-sentence still hears them; whether that reads as the same thing said twice is unverified by ear | open |  | 2026-09-08T20:00:00.000Z |  |
| 193 | 05 | unrun-verify | src/presentation/pim_rows.rs |  | The clause "changed just for this day" has never been heard. Three questions only a screen reader run answers, and the words were chosen against reasoning rather than against a listener. Whether it reads as useful or as clutter when fifty-two rows of one series go past and one of them carries it. Whether it is confused with the unreadable-rule sentence when both could apply to the same row, since both are about the same series and both arrive in the same breath. And whether the joined time cell, which can now carry the date, the out-of-hours note and this clause at once, is still one thing somebody can take in while arrowing rather than three | open |  | 2026-09-08T21:00:00.000Z |  |
| 194 | 05 | unmet-truth | src/application/calendar.rs |  | Whether an override row arriving from Google or Outlook carries a repeat rule of its own is still unknown. If it did, that row would expand across the whole window and appear on every date, which is worse than the double-show this plan measured. 05-01 was written to settle it by asserting the stored row's recurrence_rule is None after a local write, and that assertion cannot fail: the fixture writes the constant itself and save_calendar_event only round-trips it. Both provider paths derive the rule from the payload, at google_event_to_local and ms_event_to_local, so no local round trip can reach either. Settling it needs a fixture built from a real provider payload carrying an override with a RRULE, or a live account | open |  | 2026-09-08T21:00:00.000Z |  |
| 195 | 05 | unrun-verify | src/presentation/wx_calendar_module.rs |  | Nothing about the week view has been heard. Four questions only a screen reader run answers. Whether the heading, which is a StaticText the announcement queue also speaks on the calendar-period topic, is heard once or twice when Previous period is pressed. Whether Week of 20 July 2026 is the right amount of words to hear on every press, or whether it should shorten after the first. Whether the View box in the toolbar is reached and understood before somebody starts arrowing the list, since it is the fourth control on a row that starts with three buttons. And whether the announcement topic really suppresses a burst: five quick presses of Next should read the fifth week and no other, and the topic is the mechanism but nobody has heard it happen | open |  | 2026-09-08T17:35:12.582Z |  |
| 196 | 05 | unrun-verify | src/presentation/wx_calendar_module.rs |  | Previous period and Next period carry the same name on both Windows accessibility channels by construction, and neither channel has been read. The button label is Previous period and Next period, which is what Windows gives UI Automation for a native button and therefore what Narrator says, and set_accessible_name writes the same words to MSAA, which is what NVDA says. Both were checked by reading the code, not by running Axe.Windows over UI Automation or scripts/msaa-names.ps1 over MSAA. The View box beside them is a Choice with a StaticText label of its own and an explicit accessible name of Calendar view, and is unread on both channels for the same reason | open |  | 2026-09-08T17:35:22.872Z |  |
| 197 | 05 | deviation | src/presentation/managers.rs |  | The reload after an edit now asks for the window on screen rather than the whole eighteen months, so saving an event in a week view no longer puts somebody back in the agenda with nothing said. Nothing tests that it does. The window arithmetic it calls has eleven tests, and the wiring, one call to the_calendar_on_screen inside manage_calendar, has none: manage_calendar opens a wxWidgets dialog and needs a Frame, so a test would first have to extract the reload into a function of its own, and any test of it would live in managers.rs, which 41 guard records fingerprint, at roughly 80 minutes of remeasurement on the critical path for a one-line change. The plan offered fix-with-a-test or record-as-a-stub; this is the third thing, fixed without one, and it is here so that the gap is visible rather than implied by the absence of a test | open |  | 2026-09-08T17:35:23.552Z |  |
| 198 | 05 | unrun-verify | src/presentation/wx_app.rs |  | Nobody has chosen a view, closed this program and opened it again. The three pieces are each tested where they live: the settings screen writes the word, AppConfig round-trips it, and CalendarView::from_stored reads it back with a fallback for anything it does not recognise. What joins them is one line at startup that puts the stored view into WxUIState and one that sets the toolbar box to match, and neither has a test: they sit inside the frame builder, which needs a running wxWidgets window. So the claim that the view survives a restart rests on reading three tested pieces and one untested join. The same join decides what the box on the toolbar says when the window opens, which is what a screen reader reads for it | open |  | 2026-09-08T19:25:35.807Z |  |
| 199 | 05 | unrun-verify | src/presentation/wx_calendar_module.rs |  | PIM-06's third D line cannot be closed by a test and is not closed here. It asks whether a screen reader user can work through a view's events in date order without reconstructing the grid from cell labels. What this plan does is remove the grid from the question by not building one: a month is the same virtual list the agenda uses, sorted by moment, so read from the top it is in date order by construction. That is the design decision which makes the answer likely, not evidence that it is right. Two questions only a real run settles. Whether a month of rows in one flat list, which is thirty to sixty rows for an ordinary calendar and several hundred for a busy one with series in it, is navigable as one list or wants sub-headings per day. And whether somebody can tell where one day ends and the next begins, given that the date is in the same joined time cell as the out-of-hours note and the changed-day clause 05-01 added | open |  | 2026-09-08T19:40:20.942Z |  |
| 200 | 05 | unrun-verify | src/application/pim_command.rs |  | Nobody has heard a copy and a move told apart. filed() says "Buy milk copied to Shopping" and "Buy milk moved to Shopping", which differ by one word in the middle of a sentence, and the refusal for a read-only destination differs the same way. Somebody who cannot see the two lists has only that word, and it arrives at whatever rate their screen reader is set to. Whether it is caught at speed, and whether the two sentences want more distance between them than one participle, is the question T-05-10 turns on and it is unanswered | open |  | 2026-09-08T22:38:57.586Z |  |
| 201 | 05 | unrun-verify | src/presentation/wx_destination.rs |  | The chooser for a copy now offers the container the item is already in, which is the one destination a copy has that a move does not, and nobody has heard it. Somebody arrowing a tree of calendars meets the one they are standing in with nothing distinguishing it from the others. Whether that reads as a duplicate they can deliberately make, or as a window that forgot to take out the row a move would have, is unanswerable without a real run. The window says which act it is in its title and on its button, which is the whole of what tells them apart | open |  | 2026-09-08T22:39:07.613Z |  |
| 202 | 05 | unrun-verify | src/presentation/wx_app.rs |  | No copy has been carried out by pressing a key in the running program. Every piece is tested where it lives: file_under writes the copy against a real store, Filing answers the three differences, and the sentences are held to their words. What joins them is the arm in wx_app.rs that turns Ctrl+Shift+Y or the context menu line into PimCommand::Copy, and the only thing that reads it is tests/wired.rs, which reads the source text of the arm rather than running it. So the claim that the key copies rests on tested pieces and a read join. The same arm decides which of the six modules answers, and a wrong answer there offers mail folders as a home for a task | open |  | 2026-09-08T22:39:17.913Z |  |
| 203 | 05 | unrun-verify | src/presentation/managers.rs |  | No copied item has ever reached a provider. The copy is written as an item made on this computer waiting to be sent, and a_provider_holds answers false about it, which is asserted against a real store. What happens next is a push, and no push in this program has run against a real Google or Microsoft account. So whether a copied task is created at Google as a second task, rather than rejected or silently reconciled against the original, is untested and untestable here. It is the whole of T-05-09's mitigation past the local write | open |  | 2026-09-08T22:39:31.628Z |  |
| 204 | 05 | deviation | guards/guards.toml |  | Two of the three answers that make a copy not a move have tests and no guard record. makes_a_new_row has two records, one on the decision and one on the write. leaves_out_where_it_is and needs_the_holder_told have neither, so nothing checks that the tests covering them would still notice if they stopped. Both are covered by a test in tests/a_copy_leaves_the_original_where_it_was.rs that has been red once, in the RED commit, which is more than an unmeasured guard has; what is missing is the recorded break that would catch the test going quiet later. Two more records is two more hand measurements, each a build and a full run | open |  | 2026-09-08T22:39:32.251Z |  |
| 205 | 05 | unrun-verify | src/presentation/managers.rs |  | Nobody has been asked two questions for one move. A contact's move asks which group it is leaving and then which it is joining, where every other kind is asked once, and both windows are the same SingleChoiceDialog holding group names and counts. Whether that reads as a program being thorough or as one that cannot make up its mind is the question, and it can only be answered by somebody working down it by keyboard through a screen reader. The first question is skipped when there is only one answer, so the common case may be one question and the uncommon case two, which is its own kind of surprise | open |  | 2026-09-09T01:23:15.522Z |  |
| 206 | 05 | unrun-verify | src/application/contact_groups.rs |  | The sentence a completed move says has never been heard. moved_between names two groups and two counts: "Ada Lovelace moved out of Team A, 2 people, and into Team B, 5 people." That is the longest status line this program says about a single act, and it is said at whatever rate somebody's screen reader is set to. It deliberately leaves out taken_out's reassurance that the contact is still in the address book, on the grounds that naming a group the contact is now in shows that without saying it. Whether the sentence is heard to the end, and whether the missing reassurance is missed, is unanswered | open |  | 2026-09-09T01:23:29.020Z |  |
| 207 | 05 | unrun-verify | src/presentation/managers.rs |  | The two choosers a contact's move opens have never been told apart by ear. Both are a list of group names with the count of each, one after the other, and the only thing distinguishing them is the question and the window title: "Which group should this contact come out of?" in a window called Move out of a group, then "Which group should this contact go in?" in one called Move into a group. Somebody who missed the first word of either is looking at two lists that read identically. Whether the titles are announced at all, and whether come out of and go in are enough distance between two questions asked seconds apart, is unanswered | open |  | 2026-09-09T01:23:29.652Z |  |
| 208 | 05 | unrun-verify | src/presentation/managers.rs |  | No contact has been moved between groups by pressing a key in the running program. Every piece is tested where it lives: the transactional write against a real store, the two filters and the five sentences as pure functions, and the menus against the command. What joins them is the arm in managers::pim_command that sends a contact to the group path before the chooser that names one container, and the only thing that reads it is a source-text check in tests/a_contact_moved_between_groups.rs. So the claim that Ctrl+Shift+V moves a contact rests on tested pieces and a read join. The same shape as ledger 202, one plan later | open |  | 2026-09-09T01:23:46.288Z |  |
| 209 | 05 | unrun-verify | src/application/context_menu.rs |  | The contact context menu's Put in a group line now raises the copy command rather than an action of its own, and nobody has met the menu since. Two things are unheard. Somebody who learned Ctrl+Shift+Y as Copy in the calendar, Tasks or Notes meets a line called Put in a group here and there is nothing in the wording to connect them; the reason for keeping the old wording is that it describes what happens to a person and Copy does not, and whether that trade is right is a judgement about hearing it. And the menu now reads New contact, Move to another group, Put in a group, Take out of a group, Delete, met in that order by somebody who cannot skim | open |  | 2026-09-09T01:23:46.872Z |  |
| 210 | 05 | unrun-verify | src/application/contact_groups.rs |  | The two refusals before either question have never been heard. in_no_group says the contact is in no group and to use Put in a group first; in_every_group says it is in every group and to make another group first. Both go down the refusal channel, both name a command by the words the menu uses, and both are the answer to a key somebody just pressed with no window opening. Whether naming a menu line inside a spoken sentence is followed, or whether somebody hears a sentence about a command and looks for a dialog, is unanswered | open |  | 2026-09-09T01:23:47.493Z |  |
| 211 | 05 | unrun-verify | src/presentation/managers.rs |  | The account chooser is a flat list where every other move in this program opens a tree, and nobody has met it. Move and Copy on an event, a task, a note or a message open build_destination_dialog, which draws accounts as branches with places under them; a reminder opens pick_one, a SingleChoiceDialog holding account names in a row. The reason is structural rather than a preference: an account row in that tree pushes None into its destinations vector on purpose, so the tree cannot answer an account at all. Whether somebody who has learned Ctrl+Shift+V as the key that opens a tree hears a flat list and reads it as a different command, or as the same one asking a simpler question, is a real question and nobody has heard either | open |  | 2026-09-09T04:40:20.115Z |  |
| 212 | 05 | unrun-verify | src/presentation/managers.rs |  | The name a completed reminder move says has never been heard. It is the account's label, with its address after it only where two accounts read alike, which is the rule so_no_two_accounts_read_alike already applies to the sidebar and which where_mail_can_go already takes. The plan asked for the address on every move, quoting Branch.account_name's doc comment; the sidebar rule was taken instead because an address read aloud on every move costs something for a case that needs it rarely. Whether Ring the dentist moved to Work is enough for somebody with two accounts, or whether the address is wanted every time even at that cost, is a judgement about hearing it and nobody has | open |  | 2026-09-09T04:40:34.804Z |  |
| 213 | 05 | unrun-verify | src/application/pim_command.rs |  | The sentence somebody with one account meets has never been heard, and neither has how often they meet it. the_only_account_there_is says the reminder is in the one account set up on this computer, that there is nowhere else to move it to, that nothing has been moved, and that setting up a second account gives a reminder somewhere to go. It is said before any window opens, which is the right shape. What is unknown is whether a person who has one account and no intention of adding another hears it as information the first time and as nagging every time after, since Ctrl+Shift+V is one key away from Ctrl+Shift+K on the same list. Nothing throttles it and nothing remembers that they have heard it | open |  | 2026-09-09T04:40:35.448Z |  |
| 214 | 05 | unrun-verify | src/application/pim_command.rs |  | The clause a filing now says has never been heard. A move or copy into a container an account holds ends 'and has not reached the account yet', or names Allow Changes where that setting is off, and both are joined onto the sentence naming where the item went rather than being a second sentence. Whether one sentence carrying both facts is heard as one answer or as a run-on, and whether the clause reads as useful or as noise after the twentieth move, is a judgement about hearing it. Guardrail 5 is the risk: this is said after every single filing somebody makes | open |  | 2026-09-09T06:50:00.150Z |  |
| 215 | 05 | unrun-verify | tests/wired.rs |  | Whether Ctrl+Shift+V really reaches the filing handler in the non-mail modules, rather than only appearing in a menu label, is assumption A2 of 05-RESEARCH.md and nothing in this repository can answer it. tests/wired.rs says in its own header that a bound key proves Windows will dispatch it and says nothing about what the handler then does with the right thing on screen. Recorded rather than left to a green wiring test to look like an answer | open |  | 2026-09-09T06:50:12.578Z |  |
| 216 | 05 | unrun-verify | src/presentation/managers.rs |  | The sentence 'and has not reached the account yet' has never been followed by an account receiving anything, because no build has run against a real account. What the clause promises is that the next sync sends it, and that the summary then says so; both halves are tested against scripted providers only. If a push fails for a reason the sync counts rather than reports, somebody hears the move was waiting and never hears that it stopped waiting | open |  | 2026-09-09T06:50:13.358Z |  |
| 217 | 05 | unrun-verify | src/application/tasks_sync.rs |  | The two counts a task sync now reads out have never been heard side by side. A deletion held by an outstanding create is said as a count, '1 removal waiting for the new copy to be sent', and a deletion held by Allow Changes is said as a sentence naming the setting. Both are counts of something that did not happen, and only one of them is something the person can act on. Whether somebody hearing both in one status line can tell which is which, or hears two numbers about waiting and goes looking for one setting that fixes both, is a judgement about hearing it and nobody has | open |  | 2026-09-09T09:04:16.858Z |  |
| 218 | 05 | unrun-verify | src/application/tasks_sync.rs |  | Whether a provider goes on answering for a task under its old identifier long enough for the pull that follows a move to write it back down is a timing question no fake can answer. The push sends the create, the pull in the same sync reads every list, and the deletion of the old copy does not go until the sync after that. Between them the provider holds both copies and a read that sees the old one is answered only by the deletion note masking it. That masking is tested; how long the window really is at Google and at Microsoft is not, and cannot be without an account | open |  | 2026-09-09T09:04:31.423Z |  |
| 219 | 05 | unrun-verify | src/application/deletions.rs |  | Whether seven days is long enough for a move made just before a laptop is shut for a fortnight has never been tried. The note itself is safe: let_go_of_deletions_taken_before only releases a deletion a provider has taken, so one still waiting for its new copy survives however long it waits, which was read on main at 143a37f rather than assumed. What is released by the clock is the memory of a deletion already taken, and the read consults that memory to stop a provider writing the thing back down. A move that completes on day one and a machine that comes back on day fifteen is the case nobody has run | open |  | 2026-09-09T09:04:32.089Z |  |
| 220 | 05 | todo | src/data/message_cache/tasks.rs |  | rename_task orphans a task's subtasks and nobody decided that it should. It calls drop_synced_task on the old identifier, which sets parent_task_id to null for every child, so a task made on this computer loses its subtask tree the moment a provider names it. That is older than this plan and nothing in this plan reaches it. move_a_task_the_provider_holds deliberately answers the same question the other way, pointing the children at the new identifier, because the parent has not gone but been renamed, and copying the null would have flattened a tree on every move. The two now disagree on purpose and one of them is wrong | open |  | 2026-09-09T09:04:46.509Z |  |
| 221 | 05 | unrun-verify | tests/a_half_finished_task_move.rs |  | The failure this whole state exists for has never happened. A half-finished move is the create succeeding at the provider and the delete then failing, or the reverse, and nothing in this repository can produce either: no provider is called by this plan at all, and 05-08 makes the calls against fakes that answer from a script. Every test here builds the half-finished state by calling the write that produces it, which proves the state is held, survives a close and cannot exist half-written, and proves nothing about whether a real pair of calls leaves exactly that state | open |  | 2026-09-09T09:04:47.221Z |  |
| 222 | 05 | unrun-verify | src/application/tasks_sync.rs |  | Whether a real provider accepts a create into a second list while the first still holds the task, which is the state a provider-held task move passes through on purpose | open |  | 2026-09-09T21:16:20.424Z |  |
| 223 | 05 | unrun-verify | src/application/deletions.rs |  | Whether a real provider's list still names the old copy on the pull that follows the delete, and for how long. The seven-day memory in application::deletions is sized against a number nobody has measured | open |  | 2026-09-09T21:16:34.744Z |  |
| 224 | 05 | unrun-verify | src/application/tasks_sync.rs |  | Whether a person hearing the move sentence and then the sync summary can tell a change waiting on a create from a change waiting on Allow Changes. The two are counted apart and only one names a remedy, and nobody has heard them side by side | open |  | 2026-09-09T21:16:35.427Z |  |
| 225 | 05 | unrun-verify | src/service/tasks_api.rs |  | Whether Microsoft's notStarted, inProgress, waitingOnOthers and deferred survive a move. The new copy is created rather than updated and ms_task_to_entry rebuilds the row from Graph's answer, so remote_status is carried by the row up to the create and by Graph after it | open |  | 2026-09-09T21:16:36.063Z |  |
| 226 | 05 | unrun-verify | src/application/tasks_sync.rs |  | Whether the identifier a provider hands back for the created copy is accepted by the same provider's delete for the old one, on an account signed in to both providers at once. Both passes read the same notes against the same account | open |  | 2026-09-09T21:16:36.731Z |  |
| 227 | 05 | deviation | src/presentation/managers.rs |  | A task a provider holds moved into a list made on this computer: the create is counted local_only and never sent, so the deletion note waits for ever. Nothing is lost and the person can put it right by moving it into a synced list, but nothing tells them that. Tested and reported, not refused | open |  | 2026-09-09T21:16:51.492Z |  |
| 228 | 05 | deviation | src/service/tasks_api.rs |  | google_task_to_entry sets created_at to empty, so a moved task loses when it was made once Google names the new copy. Pre-existing for every task created here and synced; a provider move now traverses it too | open |  | 2026-09-09T21:16:52.197Z |  |
| 229 | 05 | unrun-verify | src/application/tasks_sync.rs |  | No test asserts that a task's fields survive the round trip through a provider's answer to the create. The Scripted fake returns a bare task, so the title comes back as Untitled task; a real provider echoes the body. Field survival is asserted on the local write only | open |  | 2026-09-09T21:16:52.910Z |  |
| 230 | 05 | unrun-verify | src/presentation/managers.rs |  | Nobody has heard what a move of a provider-held task says. The clause is 05-06's and unchanged, and whether it carries the fact that the provider has not been told yet without wearing after twenty moves is a judgement about hearing it | open |  | 2026-09-09T21:16:53.634Z |  |
| 231 | 05.1 | unrun-verify | src/presentation/read_aloud.rs |  | Nobody has heard a note whose Markdown is read back as structure. Whether "heading level 1, Shopping, bullet, milk" is clearer to listen to than the flat text it replaced is the whole argument for storing Markdown, and it has never been put to a screen reader | open |  | 2026-09-10T14:50:56.612Z |  |
| 232 | 05.1 | unrun-verify | src/data/message_cache/notes.rs |  | No database written by another build has ever been opened. NoteBody::Other and the null-column path are driven only by rows this repository's own tests wrote with raw SQL, so what a real second writer puts in that column is a guess | open |  | 2026-09-10T14:51:05.860Z |  |
| 233 | 05.1 | unrun-verify | src/presentation/wx_settings.rs |  | Nobody has heard the Notes section on the Calendar and PIM tab. Whether it is found where it was put, last on the tab after Working Day, by somebody moving through the sections in order with a screen reader, has never been tried | open |  | 2026-09-10T17:40:28.476Z |  |
| 234 | 05.1 | unrun-verify | src/application/notes_backend.rs |  | Nobody has heard the sentence saying an account has no notes backend. Whether it is heard as an answer or as an apology is the whole question about wording it that way, and it has never been put to a screen reader | open |  | 2026-09-10T17:40:42.877Z |  |
| 235 | 05.1 | unrun-verify | src/application/context_menu.rs |  | The note folder menu has never been opened with a backend behind it, because none exists. Every account answers that its notes stay here, so the arm that offers Sync notes now is driven only by tests | fixed |  | 2026-09-10T17:40:43.531Z | 2026-09-10T20:50:53.826Z |
| 236 | 05.1 | stub | src/application/notes_backend.rs |  | NotesService has no implementor and no caller. It is the contract 05.1-03 and 05.1-04 are held to, and until one of them lands nothing has ever executed a line of it | fixed |  | 2026-09-10T17:40:44.182Z | 2026-09-10T20:50:53.090Z |
| 237 | 05.1 | unrun-verify | docs/development/the-notes-seam.md |  | The notes seam contract is reasoning from Microsoft's documentation and from what this repository already does, not from a backend that has run. Its own table names one assumption it knows is weakest, that one note maps to one thing at the backend, and 05.2-03 is required to report where it was wrong | open |  | 2026-09-10T17:40:44.830Z |  |
| 238 | 05.1 | unrun-verify | src/service/caldav_journal.rs |  | Whether a real calendar server accepts the journal document this writes. The whole write path is untried: if a server refuses it, every note push fails and nothing anybody has run would have said so. | open |  | 2026-09-10T20:49:58.456Z |  |
| 239 | 05.1 | unrun-verify | src/service/caldav.rs |  | Whether a real server's own listing names a journal entry the way this reads it. The listing is a PROPFIND at one level down and the collection's own block is skipped by hand; a server that answers in another shape reports an empty container, which reads as a clean sync over somebody's missing notes. | open |  | 2026-09-10T20:50:19.482Z |  |
| 240 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether a real server's version marker survives a round trip. Everything about when a note is taken down rests on comparing the marker for equality, so a server that changes it on every read makes every note arrive on every sync and one that never changes it makes none arrive at all. | open |  | 2026-09-10T20:50:20.164Z |  |
| 241 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether a real server takes a deletion. The record of a deletion is kept until the server has taken it and for a week after, so a server that refuses the removal leaves the note owed for ever and one that answers in a way this misreads lets the note come back. | open |  | 2026-09-10T20:50:20.825Z |  |
| 242 | 05.1 | unrun-verify | src/service/caldav_journal.rs |  | Whether a real server raises a clash the way the stand-in does. The disagreement is found by reading the document's version before writing rather than by a 412, so a server that gives no version marker at all would never report one and a change made in two places would be written over in silence. | open |  | 2026-09-10T20:50:21.497Z |  |
| 243 | 05.1 | unrun-verify | src/application/notes_backend.rs |  | Whether the notes sync summary is distinguishable by ear from the calendar, task and contact summaries when several run together. Nobody has heard any of them. | open |  | 2026-09-10T20:50:22.241Z |  |
| 244 | 05.1 | unrun-verify | src/application/conflict_choice.rs |  | Whether a held note conflict read aloud is answerable without seeing both copies. The words say note rather than contact now, and nobody has heard the question. | open |  | 2026-09-10T20:50:22.983Z |  |
| 245 | 05.1 | stub | src/application/notes_backend.rs |  | An account with two calendar servers sends its notes to the first one the store answers with. That is a limit rather than a decision: nothing asks the person which, and nothing says which was chosen. | fixed |  | 2026-09-10T20:50:23.697Z | 2026-09-11T18:01:03.284Z |
| 246 | 05.1 | stub | src/application/notes_sync.rs |  | Note folders on this computer are not mirrored at the server. Every note arriving from a server is filed in the account's first note folder, so folders somebody made here mean nothing at the other end. | fixed |  | 2026-09-10T20:50:24.407Z | 2026-09-11T18:01:04.034Z |
| 247 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether the sentence somebody hears when the read holds a change the setting had refused is understood by ear, and whether it is told apart from the clash the push reports. Both reach conflict_choice and both say a note is waiting to be chosen; nobody has heard either. | open |  | 2026-09-10T23:33:07.431Z |  |
| 248 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether the sentence 1 note could not be kept exactly by your notes backend is understood by ear, and whether it is distinguishable from the other four sentences a notes sync can say. It is a new sentence in a status line that already carries four. | open |  | 2026-09-10T23:33:08.090Z |  |
| 249 | 05.1 | unrun-verify | tests/the_notes_seam_takes_a_second_kind_of_backend.rs |  | Whether the four constraints the second implementation is shaped from are what a real Graph OneNote client meets. They are a reading of three Microsoft pages on 2026-09-06, not a measurement of the service, and a documented API can differ from the service behind it. | open |  | 2026-09-10T23:33:48.158Z |  |
| 250 | 05.1 | unrun-verify | src/application/notes_backend.rs |  | Whether a seam proven against an implementation in this repository's own tests can be implemented from a separate crate. An integration test sees only what is pub, which is the stronger placement, but it is still built against the same source tree in the same commit. | open |  | 2026-09-10T23:33:48.816Z |  |
| 251 | 05.1 | unrun-verify | docs/development/the-notes-seam.md |  | How wide a OneNote lastModifiedDateTime tick really is, and so how likely it is that a write this program makes and an edit somebody makes at the service carry one marker. Requirement 4 of the seam's contract turns on it and nothing here can measure it. | open |  | 2026-09-10T23:33:49.479Z |  |
| 252 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether replacing the copy here with what the backend could keep is what somebody wants. The alternative is keeping their bytes and telling them the two copies differ; this was decided on the argument that a loss somebody watches happen is better than one that arrives weeks later, and nobody has been asked. | open |  | 2026-09-10T23:33:50.138Z |  |
| 253 | 05.1 | unrun-verify | src/application/notes_sync.rs |  | Whether making a note again when the backend says it no longer holds it is right where somebody deleted it at the other end on purpose. The seam's contract says a name the backend never gave is a note to create; a deletion made at the backend is not propagated here at all, so the two rules can disagree about one note. | open |  | 2026-09-10T23:33:50.847Z |  |
| 254 | 05.1 | unrun-verify | src/service/carddav.rs |  | Whether a real CardDAV server's answer about its address books parses. Every fixture was written in this repository, and the reader assumes the d: prefix on the DAV elements and that the change marker is cs:getctag. A server spelling either differently is read as offering none. | open |  | 2026-09-11T01:41:41.360Z |  |
| 255 | 05.1 | unrun-verify | src/service/carddav.rs |  | Whether a real CardDAV server's answer with cards in it parses. The card's own element is tried under three spellings, card:address-data, C:address-data and address-data, and a server writing a fourth is read as having sent no card at all. | open |  | 2026-09-11T01:41:45.676Z |  |
| 256 | 05.1 | unrun-verify | src/data/message_cache/contacts.rs |  | Whether a card vcard_block_from_contact writes is accepted by a real CardDAV server. A round trip in this repository proves this writer and this reader agree with each other and says nothing about anybody else's server. | open |  | 2026-09-11T01:41:46.427Z |  |
| 257 | 05.1 | unrun-verify | src/data/message_cache/contacts.rs |  | Whether a card written by another client reads correctly through contact_from_vcard_block. Only cards this repository wrote have been read back by it here, and Apple, Google and SabreDAV each write cards this program has never seen. | open |  | 2026-09-11T01:41:47.161Z |  |
| 258 | 05.1 | unrun-verify | src/service/carddav.rs |  | Whether the scan for the address book element over a whole response block, rather than over the resource type alone, ever reads a server's answer as offering an address book it does not have. It needs a server sending a raw angle bracket inside a name it should have escaped. | open |  | 2026-09-11T01:41:58.844Z |  |
| 259 | 05.1 | deviation | docs/ALPHA_TESTING.md |  | A malicious card joining two people who share an address is a named limitation, and CardDAV raises its severity: today it needs somebody to choose a file, and over a server it arrives from the network. This plan does not change ContactEntry::shares_an_address_with or the_same_person_on_an_earlier_card, so the change of severity is recorded rather than answered. | open |  | 2026-09-11T01:41:59.594Z |  |
| 260 | 05.1 | unrun-verify | src/application/carddav_sync.rs |  | Whether a real CardDAV server accepts a card this writes with If-Match, and answers 412 when the version has moved. The sync treats 412 as the copy having moved past the change and keeps the edit for the next sync; a server answering 409 or 403 instead is read as an ordinary failure and the edit is still kept, but it is not counted as built on an old copy, so the read in the same sync can replace it. | open |  | 2026-09-11T04:44:04.081Z |  |
| 261 | 05.1 | unrun-verify | src/application/carddav_sync.rs |  | Whether a real CardDAV server answers a PUT to an address nothing is at by making the card. Every contact made here is written to an address this program chose, under the contact's own identifier, and a server that refuses to create at an address it did not name would fail every create with nothing here able to tell that apart from a refusal. | open |  | 2026-09-11T04:44:04.822Z |  |
| 262 | 05.1 | unrun-verify | src/application/carddav_sync.rs |  | Whether a real server's change marker moves on every change to an address book. If it does not, a sync that compares it skips the read and nothing new is ever seen; if it moves when nothing changed, the whole address book is read every time. Both are silent. | open |  | 2026-09-11T04:44:05.652Z |  |
| 263 | 05.1 | unrun-verify | src/service/carddav.rs |  | Whether a real CardDAV server gives an ETag on the PUT response at all. Where it does not, the version marker is nothing, the_marker_moved reads that as moved, and every contact is read as having changed at the server on every sync. | open |  | 2026-09-11T04:44:06.370Z |  |
| 264 | 05.1 | unrun-verify | src/application/address_book_source.rs |  | Whether a real address book server's home set answers the PROPFIND this makes at Depth 1 with the address books in it. Discovery asks the address somebody typed; a server that keeps its address books one level further down answers with nothing and somebody is told the server has no address books for that sign-in. | open |  | 2026-09-11T04:44:07.152Z |  |
| 265 | 05.1 | unrun-verify | src/presentation/wx_add_address_book.rs |  | Whether the new address book screen is usable by ear. Every control carries an accessible name set the only way that reaches NVDA, and the mnemonics were checked by hand because nothing can check them. Whether the names are the ones intended rather than a nearby label Windows fell back to, whether the refusal is heard, and whether what the server found is announced rather than only drawn, are all things only a screen reader run answers. | open |  | 2026-09-11T04:44:07.898Z |  |
| 266 | 05.2 | unrun-verify | src/service/onenote_page.rs |  | Whether a real page's output HTML matches the model this plan built from Microsoft's reference, construct by construct. Every row of the fidelity table went through a model of the service written from a page dated 2024-11-07 and read on 2026-09-11. No OneNote tenant has ever been used with this program, so the table says what this program does with what the reference says comes back, and nothing about what comes back. | open |  | 2026-09-11T06:50:37.976Z |  |
| 267 | 05.2 | unrun-verify | src/service/onenote_page.rs |  | Whether a data-id on a div really survives a page update. The reference says a div carrying one is preserved where a div carrying no semantic information is flattened, and the hidden source div was measured against that reading rather than against a service. If a data-id does not survive an update, the wrapping div this reader flattens may not be there at all and what comes back is a different shape. | open |  | 2026-09-11T06:50:38.717Z |  |
| 268 | 05.2 | unrun-verify | src/application/long_text.rs |  | Whether a note whose body came back through this pair is read aloud by a screen reader the way the original was. The whole reason the structure is preserved is that a heading announces as a heading and a list as a list. Seven of twenty-two constructs survive the round trip and six of the losses are this program's own reader, so what somebody hears after a note has been to OneNote is a different passage from what they typed, and nobody has heard either. | open |  | 2026-09-11T06:50:39.435Z |  |
| 269 | 05.2 | unrun-verify | docs/development/the-notes-seam.md |  | Whether a Markdown code block coming back as a flattened paragraph is acceptable to somebody who keeps code in their notes. Two commands on two lines come back as one line that runs neither. That is a question for a person who uses OneNote and keeps notes that way, and no test can answer it. | open |  | 2026-09-11T06:50:40.214Z |  |
| 270 | 05.2 | unmet-truth |  |  | PIM-04's structure criterion, reworded 2026-09-11 at 05.2-01's checkpoint, is not met. A nested list, a table, a link's address, a picture and a line break are lost on the way back from a backend, all five in long_text::from_markup. That function is a reader written for speaking, with two callers on that job outside notes, so it must not be changed to suit a note. A second reader whose output is stored and edited again is what the criterion asks for, and it belongs to no plan. | fixed |  | 2026-09-11T10:10:44.221Z | 2026-09-11T13:32:43.372Z |
| 271 | 05.2 | unrun-verify |  |  | Whether a nested list and a table are pleasant to listen to, not merely correct. A nested item is announced as 'bullet level 2, ...' and only when the level changes; a table as 'table, 2 columns, 2 rows' then 'row 1. Name: Grace. Role: Admiral', with the column heading repeated on every cell. Tests prove those exact words are produced. No screen reader has said them. The open questions are whether repeating a heading per cell floods a wide table, whether 'bullet level 2' is heard as a level or as part of the text, and whether a listener can follow a table with more than three columns at all. Only an NVDA pass can settle any of them. | open |  | 2026-09-11T13:32:06.681Z |  |
| 272 | 05.2 | deviation | src/service/onenote_page.rs |  | A picture kept in OneNote comes back pointing at OneNote's copy of it rather than at the address it went out with. The reference says a page stores the picture and hands back a resource address of its own, so the note's Markdown now names graph.microsoft.com. The picture is not lost and the address is not the one somebody typed. Whether that matters to a person whose note linked to an image they host elsewhere is a product question nobody has been asked. Measured through the model, not against a real tenant. | open |  | 2026-09-11T13:32:22.029Z |  |
| 273 | 05.2 | unrun-verify | src/application/long_text.rs |  | A block-level img in from_markup contributes its alt text with no marker saying it is a picture, and contributes nothing at all when the sender gave no alt. So a picture in a Google task's description or a calendar event's description is read aloud as an ordinary paragraph, or vanishes. Piece::Image's own doc comment and guardrail 9 both say a picture nobody described must still be announced, and the inline arm does that correctly; the block arm does not. Found by measurement during ledger 270 and deliberately left alone as out of scope: it is pre-existing, it is on the speaking path rather than the storing one, and fixing it changes what is stored for calendar and tasks. | open |  | 2026-09-11T13:32:22.877Z |  |
| 274 | 05.2 | todo |  |  | A container is a note folder, decided 2026-09-11 and written into docs/development/the-notes-seam.md, and not yet built. note_folders needs an opaque container column, the sync has to loop over folders rather than take an account's first calendar, a backend-given folder is named by its flattened path, a folder made here sits under the words mail already uses for folders on this computer, and a note moving between two backed folders follows the task model's create-then-remove order. Closing this closes 245 and 246. | fixed |  | 2026-09-11T14:30:15.791Z | 2026-09-11T18:01:02.560Z |
| 275 | 05.2 | unrun-verify | src/data/message_cache/notes.rs |  | A note moved between two folders a calendar server gave is created in the new collection and only then removed from the old one. Nobody has done that with a real server: whether the create really lands before the removal goes, and what the server says back, is untested. | open |  | 2026-09-11T17:59:53.611Z |  |
| 276 | 05.2 | unrun-verify | src/application/notes_sync.rs |  | A removal held back until its copy reaches the backend goes out on the next sync. The hold is driven against a stand-in that says yes to everything; nobody has watched the two syncs run against a real calendar server. | open |  | 2026-09-11T17:59:54.333Z |  |
| 277 | 05.2 | unrun-verify | src/application/notes_backend.rs |  | An account with two calendar servers now syncs both. Nobody has run it with two real servers, so whether two sign-ins in one pass both work, and what one refusing does to the other, is unknown. | open |  | 2026-09-11T17:59:55.050Z |  |
| 278 | 05.2 | unrun-verify | src/presentation/note_folder_tree.rs |  | Nobody has heard the notes tree with a screen reader. Whether the On this computer branch is met as a place and whether it is clear that the folders under it go nowhere is unmeasured, and so is whether a folder named after a calendar reads as its name. | open |  | 2026-09-11T18:00:10.261Z |  |
| 279 | 05.2 | unrun-verify | src/data/message_cache/notes.rs |  | Two calendars sharing a display name give a folder named Work and one named Work (2). Nobody has heard that read aloud. At a screen reader's default punctuation level the brackets are expected to be silent, so it should read Work 2, and that is an expectation rather than a measurement. | open |  | 2026-09-11T18:00:11.003Z |  |
| 280 | 05.2 | todo | src/presentation/managers.rs |  | A note a backend holds, moved into a folder somebody made here, leaves the backend holding its copy for ever. The removal waits for a copy that sits in a folder no push reaches, so it never goes. The task move has the same shape and docs/ALPHA_TESTING.md says so, but nothing in the program tells the person their server still has it. | open |  | 2026-09-11T18:00:11.768Z |  |
| 281 | 05.2 | todo | docs/development/the-notes-seam.md |  | A folder name carrying a flattened path, Work / Projects / Q3, arrives only when a backend with levels above a note ships. Nothing chooses that separator today and what a screen reader makes of it is unmeasured. | open |  | 2026-09-11T18:00:12.574Z |  |
| 282 | 05.2 | unmet-truth | src/service/oauth.rs |  | Tasks.ReadWrite is in the outlook provider's default_scopes and absent from THE_SCOPES_A_GRAPH_TOKEN_CARRIES, the array get_valid_graph_token refreshes with, so no running Microsoft account holds it and every Graph task write is refused. 05.2-02 found it and did not fix it: it is a live defect on another feature. docs/PROVIDER_SETUP.md's 'If you signed in before tasks synced both ways' tells somebody signing in again will send their waiting task changes, which this gap would make false. | open |  | 2026-09-12T00:07:09.234Z |  |
| 283 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether Graph accepts the input HTML service::onenote_page produces when a page is created, or normalises it into something the fidelity table did not predict. Every request has been read off a loopback socket and none has met Microsoft. | open |  | 2026-09-12T00:07:29.659Z |  |
| 284 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether OneNote's generated identifiers really change on every page update or only on some. That decides whether reading before every write is necessary or merely safe, and the read inside change_page is built on the reference's word for it. | open |  | 2026-09-12T00:07:30.352Z |  |
| 285 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether removing a page's elements one at a time by generated identifier and appending new ones leaves a page a person recognises, or leaves it reordered and restyled. Remove-and-append was chosen over delete-and-recreate on a failure argument, not on a measurement of the result. | open |  | 2026-09-12T00:07:31.045Z |  |
| 286 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether a patch command with action delete, naming a generated identifier, is accepted for every element a OneNote page can hold. The supported-actions table quoted in this repository covers replace and says nothing this repository has read about delete, and changing a page depends on it. | open |  | 2026-09-12T00:07:31.736Z |  |
| 287 | 05.2 | unrun-verify | src/service/oauth.rs |  | Whether a consumer Microsoft account can grant Notes.ReadWrite without an administrator. The reference implies it and does not state it, and docs/PROVIDER_SETUP.md says plainly that we cannot tell somebody yet. | open |  | 2026-09-12T00:07:32.434Z |  |
| 288 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether a section group nested deeper than MOST_SECTION_GROUPS_DEEP, which is eight, is a real notebook anybody has. The bound is a guess at a hostile answer rather than a measurement of anyone's notebook. | open |  | 2026-09-12T00:07:33.175Z |  |
| 289 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether a paged OneNote listing really uses the @odata.nextLink field name and the value shape the walk follows. Paging is modelled on what list_contacts does for a different Graph endpoint and on a fixture this repository wrote. | open |  | 2026-09-12T00:07:33.914Z |  |
| 290 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether a page created by posting text/html to /me/onenote/sections/{id}/pages answers with the onenotePage JSON this client reads, and with which status. The fixture answers 201 with id and title because the reference describes that resource, not because anything saw it. | open |  | 2026-09-12T00:07:34.643Z |  |
| 291 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether a real notebook's shape is what the four-level walk expects. Notebook, section group, section, page is a reading of Microsoft's reference; nobody here has seen a notebook, so whether every section is reached, and whether the sections of a shared or a class notebook answer at all, is unknown. | open |  | 2026-09-12T02:47:39.602Z |  |
| 292 | 05.2 | unrun-verify | src/service/onenote_page.rs |  | Whether a page this program creates looks like a note to somebody who then opens it in OneNote itself. The HTML is built to what the reference names, and nothing has rendered it in OneNote's own window or read it back out of one. | open |  | 2026-09-12T02:48:01.509Z |  |
| 293 | 05.2 | unrun-verify | docs/development/the-notes-seam.md |  | Whether a page edited in OneNote and read back here keeps the structure the fidelity table predicts, construct by construct. The middle step of that table is a model of the service, measured against what three documentation pages describe rather than against anything Microsoft returned. | open |  | 2026-09-12T02:48:02.940Z |  |
| 294 | 05.2 | unrun-verify | src/service/onenote_notes.rs |  | Whether the conflict answer raises clashes nobody caused. A lastModifiedDateTime moves when the service touches a page for its own reasons, and only a real service touching a real page can show how often that happens and what it then feels like. | open |  | 2026-09-12T02:48:04.386Z |  |
| 295 | 05.2 | unrun-verify | src/presentation/wx_settings.rs |  | Whether the Notes section of the settings screen is navigable by keyboard through a screen reader when an account can give three different answers on one line. Nobody has heard the OneNote sentence, which is the longest of the three and names a loss. | open |  | 2026-09-12T02:48:05.813Z |  |
| 296 | 05.2 | unrun-verify | src/service/onenote_notes.rs |  | Whether somebody who has used OneNote for years finds a note made by this program acceptable in their notebook. A page with a title and one block of content is not what OneNote's own editor produces, and this is a question for a person rather than a test. | open |  | 2026-09-12T02:48:07.221Z |  |
| 297 | 05.2 | unrun-verify | src/service/onenote_notes.rs |  | Whether the folder-name separator, a slash with a space on each side, is heard well. A section called Work / Projects / Q3 is read at a screen reader's default punctuation level and nobody has listened to a list of them. | open |  | 2026-09-12T02:48:08.639Z |  |
| 298 | 05.2 | unrun-verify | src/application/notes_backend.rs |  | Whether a Microsoft account that also has a calendar on a CalDAV server should get both backends. for_account answers one backend per account and asks the calendar server first, so such an account's OneNote sections are invisible here. Nobody has been asked which they would want. | open |  | 2026-09-12T02:48:10.068Z |  |
| 299 | 05.2 | unrun-verify | src/service/onenote_notes.rs |  | Whether five requests per changed note is acceptable against Graph's rate limits. A page with no whole-document write and no entity tag costs a read for the marker, a read for the identifiers, the change, a read for the new marker and a read for what was kept, and no real account has ever been asked once. | open |  | 2026-09-12T02:48:11.499Z |  |
| 300 | 05.2 | unrun-verify | src/service/microsoft_graph.rs |  | Whether GET /me/onenote/pages/{id} and GET /me/onenote/pages/{id}/content answer with the resource and the document this client reads. The fixtures answer what the reference describes for onenotePage and for page content, not what anything saw. | open |  | 2026-09-12T02:48:12.950Z |  |
| 301 | 05.2 | unrun-verify | src/application/notes_backend.rs |  | Whether a Notes list that is empty until the first sync is acceptable for a Microsoft account. A notebook's sections are Microsoft's answer and cannot be asked for while a screen is being filled, so the folders arrive at the first sync rather than before it. | open |  | 2026-09-12T02:48:14.390Z |  |
| 302 | 07 | unrun-verify | src/presentation/first_run.rs |  | Nobody has heard the first-run storage sentences under a screen reader. Whether they land as an important fact or as more of the same warning, arriving in the middle of a screen that already says everything which writes is experimental, is what criterion 4 really asks and no test here can answer it | open |  | 2026-09-12T06:27:24.809Z |  |
| 303 | 07 | unrun-verify | src/presentation/command_line.rs |  | The storage paragraph added to the end of what --help prints has never been read in a real terminal or by a screen reader. Its wrapping and its place at the end of a long page are both unmeasured | open |  | 2026-09-12T06:27:38.279Z |  |
| 304 | 07 | unmet-truth | src/common/logging.rs | 79 | Two of the three writes this program makes to the temporary folder are on no page a user can read. The log fallback at logging.rs:79 can hold whatever the running log holds, on a machine where the data folder could not be resolved, and the converted help pages at help_page.rs:97 hold nothing of anybody's. Neither goes through a paths.rs accessor, so the new check cannot reach either. 07-05 rewrites the privacy page and owns whether the log fallback earns a sentence | open |  | 2026-09-12T06:27:39.183Z |  |
| 305 | 07 | todo | tests/house_style.rs |  | test_no_document_says_the_cache_is_encrypted and its companion have no guards/guards.toml record, so nothing measures that they still redden when the check they hold is narrowed. The signing guard written beside them in 07-01 does have one, and it breaks house_style.rs itself, so the same shape is available to them | open |  | 2026-09-12T06:27:51.385Z |  |
| 306 | 07 | unrun-verify | installer/Wixen-Mail-Setup.iss |  | Nothing here compiles the installer with ISCC, installs anything or looks at a shortcut, so whether the Start menu entry, the desktop shortcut and the Apps and Features entry really show the icon after a real install is unverified | open |  | 2026-09-12T07:45:55.655Z |  |
| 307 | 07 | unrun-verify | src/presentation/accessibility/platform_bridge.rs |  | No Linux or macOS build of Wixen Mail has ever been made, so the bridgeless sentences have only ever been produced from arguments a test on Windows chose. Whether they appear at all on a real non-Windows start, and whether the two constants a real non-Windows build supplies are the ones this expects, is unverified. 07-06 is the first thing that could answer the first half | open |  | 2026-09-12T09:52:08.998Z |  |
| 308 | 07 | unrun-verify | src/presentation/wx_app.rs |  | Whether the About dialog draws the disclosure without the layout breaking is unverified. The dialog is fixed at 380 by 260 when there is nothing to disclose and grows to its contents when there is, and nobody has opened the grown version, because no build without the accessibility bridge exists | open |  | 2026-09-12T09:52:22.929Z |  |
| 309 | 07 | unrun-verify | src/presentation/accessibility/platform_bridge.rs |  | Nobody who depends on a screen reader has heard the disclosure. Whether the four paragraphs land as useful information or as a wall of apology in front of somebody who has just started a mail client is what guardrail 5 asks and no test here can answer it | open |  | 2026-09-12T09:52:23.841Z |  |
| 310 | 07 | unrun-verify | src/presentation/accessibility/platform_bridge.rs |  | That adding a third platform module changes the answer with no edit to the code that says it is held structurally rather than by a test. No third platform module exists to add, and this tree has no compile-fail harness in which the absence of one could be expressed: grep -n trybuild Cargo.toml returns nothing | open |  | 2026-09-12T09:52:24.787Z |  |
| 311 | 07 | unrun-verify | src/common/version.rs |  | No release has ever been published from this repository and git tag returns nothing, so this comparison has never been handed a version string that came from anywhere but a test. No prerelease has been cut either, so the prerelease ordering and the channel rule resting on it are the parts with no real example behind them at all | open |  | 2026-09-12T11:38:55.805Z |  |
| 312 | 07 | unrun-verify | src/common/version.rs |  | That a published tag starts with a v is read off .github/workflows/release.yml, which names the portable download after the tag and publishes it under the glob wixen-mail-v-star.exe. No tag exists anywhere to confirm it. If that inference is wrong, or if cargo-release is later configured with a different tag name, the comparison answers that it could not read the tag for every release this project cuts and nothing fails | open |  | 2026-09-12T11:39:09.741Z |  |
| 313 | 07 | unrun-verify | src/common/version.rs |  | Nothing outside the tests calls the ordering, the channel type, the setting type or the offer decision. Whether any of it is reachable from a path a person can take is 07-05's to establish, and until it is, this is code that compiles and passes and has never run | fixed |  | 2026-09-12T11:39:10.499Z | 2026-09-12T14:39:53.474Z |
| 314 | 07 | unrun-verify | src/common/version.rs |  | The forgiving read of a stored setting covers a value that is a string. A stored value of any other type still fails the read, which in the loader 07-05 inherits takes every setting on that machine back to its default. Nothing here tests that, because the loader is not in this plan, and 07-05 is told to read it and report which of the two it does | open |  | 2026-09-12T11:39:11.254Z |  |
| 315 | 07 | unrun-verify | src/service/update_check.rs |  | No release has ever been published from this repository, so the only answer this check has ever produced against a real endpoint is the one saying nothing is published. That was measured on 2026-09-12 against both endpoints and both really answer it, 404 on releases/latest and 200 with an empty array on releases. The two answers that matter most, a newer version and this being the newest, have only ever been produced from fixtures written here. If GitHub's real response for a published release does not parse, the feature is wrong rather than absent, which is worse | open |  | 2026-09-12T14:39:21.269Z |  |
| 316 | 07 | unrun-verify | src/presentation/wx_app.rs |  | Whether the answer is actually heard. It is sent as UIUpdate::ANewerVersionIsPublished or CommandAnswered, written to the status bar and announced at high priority on the command topic, and when there is a newer version a dialog follows it. Nothing in this tree can ask whether NVDA speaks it, whether the announcement and the dialog that follows read as one answer or as two, or whether the dialog interrupts the announcement before it finishes | open |  | 2026-09-12T14:39:47.935Z |  |
| 317 | 07 | unrun-verify | src/presentation/wx_settings.rs |  | Whether the one three-valued combo box reads well under NVDA. D-16 chose one control with three values for an accessibility reason rather than a tidiness one, and the choice of a combo box over a radio group was argued from consistency with the other twenty-four controls on that dialog and from wxdragon having no RadioBox binding in this tree. The cost named is that somebody never opening the box does not hear that a test-version option exists. Only a real screen reader run settles whether that cost is the right one | open |  | 2026-09-12T14:39:48.694Z |  |
| 318 | 07 | unrun-verify | src/service/update_check.rs |  | Whether the development channel's list endpoint returns what this code expects. No list with anything in it has ever been returned for this repository: asked on 2026-09-12 it answers 200 with an empty array. The ordering, the refusal of unreadable tags and the hundred-entry bound have only been driven by fixtures built from another project's response with the tags substituted | open |  | 2026-09-12T14:39:49.465Z |  |
| 319 | 07 | todo | src/presentation/wx_settings.rs |  | Windows knows whether a connection is metered and nothing here asks. Somebody on a phone tether who chooses a kind of version will, once 07-09 lands, have installers downloaded over it. The answer taken for this phase is that the control's description says the download will be automatic, so leaving the setting off is the available answer. Detecting a metered connection is a real feature nobody has asked for | open |  | 2026-09-12T14:39:50.262Z |  |
| 320 | 07 | unmet-truth | src/data/config.rs |  | NOT_ANYTHING_ANYBODY_CHOOSES has no check of its own while its two siblings each have one. Verified rather than inherited: it has exactly two mentions in the file, its definition and the exception chain, so nothing re-asks whether either of its two entries is still a value nobody chooses. OFFERED_BY_ANOTHER_SCREEN and STORED_AND_OFFERED_BY_NOTHING each have a test that re-asks. Not fixed here because it is a finding about config.rs rather than about this plan | open |  | 2026-09-12T14:39:51.064Z |  |
| 321 | 07 | unmet-truth | docs/installing.md |  | docs/privacy.md and docs/installing.md carry the same four-line block listing what is stored under LOCALAPPDATA, word for word, with nothing checking they agree. 07-05 added the temporary-folder log fallback to privacy.md and deliberately did not duplicate it into installing.md, so the two now disagree. Either installing.md gains the same sentence or the block comes from one place | open |  | 2026-09-12T14:39:51.872Z |  |
| 322 | 07 | todo | src/service/outward.rs |  | update_check.rs is on TALKS_BUT_ONLY_READS and is the first member whose answer will, once 07-09 lands, decide that an executable is fetched. That is not a write at somebody's account and it is not the harmless read the list's name implies, so the census's two categories do not quite describe it. Whether a third list is wanted was raised rather than settled, because the file is fingerprinted by ten guard records | open |  | 2026-09-12T14:39:52.687Z |  |
| 323 | 07 | unrun-verify | .github/workflows/other-platforms.yml |  | 07-06's checkpoint is open and unrun: nobody has dispatched the Other platforms workflow, so it is still unknown whether this crate builds or its suite passes on Linux or macOS. 07-06 task 3 is unexecuted, SHIP-05 does not close and phase 7 criterion 5 stays open. Dispatch the workflow from the Actions tab on branch one-dispatch-says-whether-this-crate-builds-off-windows or on main, let both jobs finish either way, and report per platform: whether cargo build --all-targets succeeded and the first error in full if not, whether cargo test --all-targets --no-fail-fast succeeded and how many ran and failed if not, the wall clock duration from the run summary, and the runner image label the first step printed | open |  | 2026-09-12T15:35:08.642Z |  |
| 324 | 07 | unrun-verify | .github/workflows/other-platforms.yml |  | The new workflow has never been run by GitHub, so nothing about it is proven beyond the parts a local test reads. Its YAML has been checked by nothing that parses GitHub Actions, the thirteen apt packages quoted from upstream have never been installed on an ubuntu-latest image, and the macOS step that installs CMake only if it is absent has never taken either branch. A defect in any of those fails the first dispatch for a reason that says nothing about whether the crate builds, which is the question the dispatch is for | open |  | 2026-09-12T15:35:20.673Z |  |
| 325 | 07 | deviation | .planning/ROADMAP.md |  | A whole-tree guard that reads the disk cannot tell one agent's uncommitted work from another's. test_the_roadmap_counts_the_files_that_are_on_disk runs on every commit and compares the roadmap's progress table with the phase directories as they sit on disk, so while a second agent was planning phase 6 in the same working tree its untracked plan files made that phase's 0/TBD cell false and refused 07-06's document commit. The count also raced: the refused run saw 1 plan and a re-run three minutes later saw 2. The row was set to 0/2 by 07-06 to get past the gate, which is bookkeeping 07-06 does not own and which phase 6's planning will move again. Whether a tree guard should read the index rather than the disk, or whether two agents should share a working tree at all, is raised rather than settled | open |  | 2026-09-12T15:49:32.838Z |  |
| 326 | 07 | unrun-verify | .github/workflows/release.yml |  | The release workflow has never run: git tag returns nothing and git ls-remote --tags origin returns nothing while --heads answers, so no published glob has ever been matched against a real dist/ and the tag branch in scripts/build-installer.sh has never been taken. dist/wixen-mail-v*.exe agrees with dist/wixen-mail-$tag.exe only while the tag begins with v, which comes from cargo-release's defaults; if it does not, that file was published as a silent absence before this change and is a failed release after it, and the second is what was wanted | open |  | 2026-09-12T17:22:01.452Z |  |
| 327 | 07 | deviation | .github/workflows/release.yml |  | cargo release pushes the tag at the Create release version and tag step, before anything is built, so any failure after that point leaves a tag on the remote with no release behind it. The new existence check moves the failure earlier than publication but not earlier than the tag. Changing it means moving cargo release after the build, which changes when a tag exists and so when a release happens, which guardrail 7 says is a deliberate decision rather than something a change about asset names makes on the way past | open |  | 2026-09-12T17:22:14.542Z |  |
| 328 | 07 | deviation | tests/a_move_says_what_has_not_been_sent.rs |  | This target cannot reach the Windows credential store on every run: it fails with No default store has been set, so cannot search or create entries. Seen in two of three whole-tree runs on 2026-09-12, a different test of the file each time, and the file passes on its own on an unbroken tree. Not caused by any change in this plan and not diagnosed | open |  | 2026-09-12T17:22:15.335Z |  |
| 329 | 07 | unrun-verify | .planning/REQUIREMENTS.md |  | Success criterion 1 cannot close until an Azure Artifact Signing account exists. The certificate is chosen (decision 6 of 2026-09-06, Azure Artifact Signing, about 9.99 dollars a month, publisher name Pratik Patel, residence requirement met) and nothing is signed. Creating the account needs a subscription, an identity check naming a real person, a payment and a role grant in a tenant, none of which is a repository operation. This is a dependency on something outside this repository rather than a defect in it | open |  | 2026-09-12T17:44:52.410Z |  |
| 330 | 07 | deviation | .planning/REQUIREMENTS.md |  | SHIP-01 says the published installer and the executable inside it, which is two things. The census plan 07-08 task 1 derives from the installer script and the release workflow counts seven: wixen-mail.exe, wixen_mail_search.dll and wixen-mail-search-setup.exe inside the installer, the setup executable, the portable copy and the zip published beside it, and the uninstaller Inno generates, which is in neither list. A plan written to the requirement's own wording would leave five unsigned, two of them executables a user runs from an installed folder. Recorded as a scope finding against SHIP-01 rather than as a defect: the requirement is narrower than the thing it is about | open |  | 2026-09-12T18:48:30.006Z |  |
| 331 | 07 | unmet-truth | tests/installer.rs |  | The census of what has to be signed counts PE files only, which is .exe and .dll, because those are what Authenticode embeds a signature into. A .ps1, an .msi or a .cat can also carry a signature, each by a different mechanism, and none would be counted. This project ships none of the three today, so the filter is correct about what it claims and incomplete about the question. Widening it without settling how each of those is signed would be worse, because it would pair a file with a signing step that cannot sign it | open |  | 2026-09-12T18:48:43.047Z |  |
| 332 | 07 | unmet-truth | tests/installer.rs |  | The census reads one Source line as one artefact, so a wildcard naming executables would be a single entry standing for however many files it matched. The installer already carries one wildcard, the line that installs the markdown guides, which the filter drops as prose. If an executable wildcard ever arrives the count is right about the line and wrong about the artefacts, and nothing would say so | open |  | 2026-09-12T18:48:43.841Z |  |
| 333 | 07 | deviation | .planning/WINDOWS.md |  | gsd-tools windows append writes a description containing a backslash into both halves of the ledger without reconciling the escaping. The JSON half escapes the character and the markdown table half does not, so the two halves disagree and the commit gate refuses the commit. Found on entry 332, whose description quoted a Windows path from the installer script. Worked around by rewording that entry to avoid the character and correcting both halves by hand. Anything appended through the tool that quotes a Windows path or a regular expression meets this, and the failure reads as a ledger the author corrupted rather than as a tool defect | open |  | 2026-09-12T18:58:04.273Z |  |
| 334 | 07 | unrun-verify | src/service/update_download.rs |  | no installer has ever been fetched over a real connection; the transport has never run outside a fixture | open |  | 2026-09-12T22:37:37.668Z |  |
| 335 | 07 | unrun-verify | src/service/update_download.rs |  | the handover has never run: no window has closed and been replaced by an installer, and no installer has replaced any files | open |  | 2026-09-12T22:37:56.351Z |  |
| 336 | 07 | unrun-verify | src/service/update_download.rs |  | the publisher check has never seen a genuine Wixen Mail signature, because no release is signed; it was proved against a Microsoft-signed file instead | open |  | 2026-09-12T22:37:57.117Z |  |
| 337 | 07 | unrun-verify | docs/installing.md |  | plan 07-09 task 3 was not attempted: no screen reader has heard an update happen, a refusal, or the moment the window closes | open |  | 2026-09-12T22:37:57.894Z |  |
| 338 | 07 | unmet-truth | src/service/outward.rs |  | the outward census has no category for a module whose bytes are executed; plans 07-05 and 07-09 both raised it and neither acted, because the change costs re-measuring ten records | open |  | 2026-09-12T22:37:58.771Z |  |
| 339 | 07 | unmet-truth | src/service/update_download.rs |  | nothing detects a metered connection, so somebody who chose a channel on a home connection has installers of about 12 MB fetched over a phone tether | open |  | 2026-09-12T22:37:59.641Z |  |
| 340 | 07 | unmet-truth | src/service/update_download.rs |  | the handover cannot remove the mutex race: an installer reaching its own AppMutex check before this process has finished closing will say Wixen Mail is still open | open |  | 2026-09-12T22:38:00.451Z |  |
| 341 | 07 | deviation | guards/guards.toml |  | plan 07-09 required the ten guard records reading the outward census to be re-measured by hand; CLAUDE.md took guard sweeps off the critical path on 2026-09-03, so they were not run and are owed to the phase sweep | open |  | 2026-09-12T22:38:01.262Z |  |
| 342 | 07 | unrun-verify | src/service/update_download.rs |  | the revocation policy is not checked against a revoked certificate, because none exists to check against | open |  | 2026-09-12T22:38:02.066Z |  |
| 343 | 06 | todo | src/presentation/accessibility/feedback.rs | 348 | Channel::ALL is still a hand-written [Channel; 4] and carries the same hole 06-01 closed for Event::ALL: nothing forces a fifth channel into it. Out of scope deliberately, four is a much smaller surface than sixteen and nothing in phase 6 adds a channel | open |  | 2026-09-13T00:24:47.583Z |  |
| 344 | 06 | unmet-truth | .planning/REQUIREMENTS.md | 1288 | FEEDBACK-01's evidence says set_event_channels is private and that no screen could write an override without changing a visibility. 06-01 made it pub and added a public reader, so that sentence and its four line numbers are now wrong about the tree. Not corrected here because whether REQUIREMENTS.md is corrected in place is decision 7, which 06-07 puts to Pratik | open |  | 2026-09-13T00:35:32.541Z |  |
| 345 | 06 | unrun-verify | src/presentation/wx_settings.rs |  | The per-event panel on the Settings Feedback tab has never been heard. A live test builds the real dialog and reads back sixteen events, six labelled check boxes and two lines, which proves the structure is there and says nothing about whether it reads well with NVDA, Narrator or JAWS | open |  | 2026-09-13T06:01:27.680Z |  |
| 346 | 06 | unrun-verify | src/presentation/wx_settings.rs |  | The three per-event controls reload beneath the cursor when the event picker changes, and nothing in this repository can prove that reads well. It is a live-region shaped problem: a screen reader user moves to the picker, changes it, and three controls below them silently become about a different event. The Reading tab already uses this pattern, so a listening pass should judge both at once | open |  | 2026-09-13T06:01:49.898Z |  |
| 347 | 06 | unrun-verify | src/presentation/wx_settings.rs |  | The sentence saying that choosing between speech and braille is done in the screen reader and not here has never been heard. It is the sentence the whole of criterion 1 exists to make somebody meet, and whether it lands where they meet it is a listening question | open |  | 2026-09-13T06:01:51.248Z |  |
| 348 | 06 | unrun-verify | src/presentation/wx_settings.rs |  | Somebody whose settings file has braille on and speech off, or the other way round, now meets one control where there were two, and it opens ticked. Nobody has judged how that reads or whether the change is noticed. The direction is deliberate and safe, nothing being announced stops being announced, but safe is not the same as understood | open |  | 2026-09-13T06:01:52.528Z |  |
| 349 | 06 | unrun-verify | src/presentation/wx_settings.rs |  | That the event picker's selection handler and the reset button's click handler really call the functions the tests drive is proved by reading one line each and by nothing else. wxdragon 0.9.17 exposes no way to raise a widget event from outside, so tests/every_event_has_a_control.rs drives the real controls through those functions directly. The behaviour is proved; the wiring is not | open |  | 2026-09-13T06:02:16.927Z |  |
| 350 | 06 | unmet-truth | src/presentation/wx_settings.rs |  | The event picker is a Choice, which has no label of its own to carry, so it is named with set_accessible_name plus a static text beside it, the way every other Choice in this dialog is. That name reaches MSAA, which NVDA reads. On UI Automation, which Narrator reads, the name arrives only through Windows falling back to the nearest static text, which is a fallback rather than something this code set. Neither channel has been checked for this control | open |  | 2026-09-13T06:02:19.760Z |  |
| 351 | 06 | todo | src/presentation/accessibility/feedback.rs |  | Channel::setting_label now has no shipping caller. The global boxes are built from Switch, so the four strings it holds are reached only by its own two tests. The plan for 06-02 said not to change them and a test guards them, so it was left alone rather than removed quietly. Point dead-code-hunter at it in a later plan of this phase | open |  | 2026-09-13T06:02:21.175Z |  |
| 352 | 06 | todo | tests/checkbox_labels.rs |  | The label walk reaches the six feedback check boxes and no other check box the settings dialog builds, because SettingsWidgets keeps the rest of its fields private. Widening it means making about twenty fields public for a test, which is worth deciding deliberately rather than as a side effect of 06-02 | open |  | 2026-09-13T06:02:22.235Z |  |
| 353 | 06 | unrun-verify | .planning/ROADMAP.md |  | Criterion 1 clause 2, by keyboard, does not close. Every control on the Feedback tab is a native Choice, CheckBox or Button so Tab reaches them, and every check box and the button carries a mnemonic distinct from the other nine on the tab. Both facts were established by reading the source, not by pressing a key. Nobody has tabbed through this tab and no test presses one | open |  | 2026-09-13T06:50:57.999Z |  |
| 354 | 07 | unmet-truth | Cargo.toml | 15 | Nothing in this repository builds at the declared floor. Cargo.toml says rust-version = 1.88 and every workflow plus rust-toolchain.toml now uses 1.98.1, so 1.88 is a claim no build has tested since it was written. Found 2026-09-13 while pinning the toolchain. Whether an MSRV job belongs here is a scoping question nobody has settled; recorded rather than fixed. | open |  | 2026-09-13T08:39:16.912Z |  |
| 355 | 07 | skipped-test | src/service/signed_mail.rs | 6523 | test_the_withdrawal_question_really_reaches_windows_own_answer stops guarding on a machine that holds no intermediate authority it trusts. Windows does not check withdrawal for the root of a chain, so an intermediate whose issuer is not installed gets no verdict, and nothing here can tell that from a broken walk. The test reads that precondition off issuer_trust and returns early when it is absent, printing why. Recorded 2026-09-13 rather than hidden: a guard that can quietly stop guarding is the shape this project keeps meeting. | open |  | 2026-09-13T08:39:33.202Z |  |
| 356 | 07 | unmet-truth | Cargo.toml |  | This ships a known-broken RSA implementation and will go on doing so. rsa 0.9.10 arrives through pgp 0.20.0, which cannot drop it, and RUSTSEC-2023-0071 has an empty patched list. The advisory is four concerns, not one: modexp timing is fixed, but the Ok or Err behavioural oracle is open at RustCrypto/RSA PR 680 and unblinded modexp on the default path is open at PR 702. The oracle needs no timing at all and recovers an RSA-1024 plaintext in 275,490 queries against published 0.10.0-rc.18. Accepted on 2026-09-13 on an exposure argument, not a fix: this program gives a sender no per-query feedback, so the oracle has no channel here. That argument can be wrong and it expires the day anything here answers a sender differently depending on whether a PGP body decrypted. PR 680 landing and shipping is what changes the answer, not the advisory clearing. Full reasoning in .cargo/audit.toml. | open |  | 2026-09-13T11:19:24.037Z |  |
| 357 | 07 | unmet-truth | scripts/audit.sh |  | cargo audit does not fail on an unmaintained or yanked warning without -D warnings, so the audit gate is blind to a whole class it looks like it covers. Measured 2026-09-13 after RUSTSEC-2023-0071 was accepted: the plain run exits 0 while printing three warnings nobody has decided, lzw RUSTSEC-2020-0144 unmaintained, proc-macro-error RUSTSEC-2024-0370 unmaintained, and chacha20 0.10.1 yanked. One unmaintained crate, paste, is in the accepted list, which makes the treatment of the class inconsistent: one is written down and three are printed and ignored. Not fixed here because turning on -D warnings needs a decision on each of the three, which is the same product decision the rsa entry just took and is out of this change's scope. | open |  | 2026-09-13T11:19:39.327Z |  |
| 358 | 07 | unmet-truth | .cargo/audit.toml |  | The still-reported check cannot see an acceptance whose basis has evaporated while the advisory goes on being reported. The rsa entry rests on the advisory having an empty patched list, which cargo audit prints as No fixed upgrade is available. The day that becomes Upgrade to something, the run still reports the advisory, the check stays green, and an acceptance resting on there being nothing to upgrade to has quietly stopped being true. A crate version fingerprint was considered on 2026-09-13 and decided against, with the reason written in scripts/audit.sh and in the phase 7 RSA advisory report: for every way an acceptance has really gone stale here the version moving and the advisory ceasing to be reported are the same event, so the fingerprint fires no earlier and costs a hand-maintained number per entry. It also would not close this gap, which is about the advisory rather than the crate. Closing it means recording the Solution line each acceptance was judged against and comparing it on every run. | open |  | 2026-09-13T11:19:54.227Z |  |
| 359 | 07 | deviation | scripts/audit.sh |  | The run that checks the acceptances depends on where cargo-audit looks for its config, and nothing would say so if that moved. Measured 2026-09-13 against cargo-audit 0.22.2 by running from target/auditprobe, two directories below the project config: it reads .cargo/audit.toml from the working directory only and does not walk upwards. The second run is made from a scratch directory carrying a config that ignores nothing, so it is correct whether or not that measurement holds. What it would not survive is a cargo-audit that merged configs from several directories, and the failure would be silent in the worst direction: every acceptance would read as still applying while nothing was actually checked. The narrower-than-the-filtered-run refusal catches the opposite direction only. | open |  | 2026-09-13T11:20:09.076Z |  |
| 360 | 06 | unrun-verify | src/presentation/date_display.rs |  | No date written in any language has been read aloud by a screen reader in that language. Plan 06-03 task 2 makes the month names in a date come from Windows, and every assertion about them is a string comparison in a test. The French and Russian tests force a locale name and compare bytes, which proves the words Windows hands back are the words this code writes. Whether a French month inside an order the person chose, spoken by a French NVDA voice, sounds like a date rather than like a fault needs a French Windows, a French voice and somebody who speaks French. | open |  | 2026-09-13T14:45:37.895Z |  |
| 361 | 06 | unmet-truth | src/presentation/wx_item_form.rs | 849 | The twelve month names in the appointment form's month list now come from Windows, and nothing tests that they do. Measured 2026-09-13: wx_item_form.rs holds 14 tests and not one of them reads the Choice this line builds, because the Choice is built inside a closure in build_date_fields and needs a real parent window. The change is glue over a function that is itself tested, and it is reached from wx_item_form.rs line 1007 and wx_send_later.rs line 127, so it runs. What nobody has done is open the form and look at the list. Proving the wiring wants a live-window test on the pattern of the checkbox_labels suite. | open |  | 2026-09-13T14:45:52.425Z |  |
| 362 | 06 | unmet-truth | .planning/ROADMAP.md |  | Success criterion 2 of phase 6 has four clauses and plan 06-03 closes part of one. Month names follow the machine in a date and in a month heading and in the appointment form's month list, but not in the eight signature-outcome sentences, which still write an English month through the chrono format %B at src/service/signed_mail.rs line 1373. Day names do not follow the machine at all: src/application/occurrences.rs line 701 still matches a chrono Weekday to an English string. Relative wording is the plan's own blocking checkpoint and was left unanswered on purpose. The silent English fallback is the one clause that is met. Task 3 of 06-03 carries the first two and is written against the checkpoint's answer. | open |  | 2026-09-13T14:46:05.068Z |  |
| 363 | 06 | unmet-truth | src/common/how_the_machine_writes_dates.rs |  | A stored day that no month has falls back to English on a machine that does have the language, which is a wider fallback than the criterion asks for. The criterion says English where there is no translation. a_month_and_a_day accepts a day from 1 to 31 without asking which month it is, so a birthday stored as --02-30 reaches Windows, Windows refuses the whole date, and the wrapper answers in English. On a French machine that reads February 30 rather than fevrier 30. It is a corrupt stored row either way and it does not take the reading down, which is what the threat register asked for, but the reason the reader sees English there is not the reason the sentence on the settings screen gives. Found by reading at green on 2026-09-13, not by a test. | open |  | 2026-09-13T14:46:18.671Z |  |
| 364 | 06 | unrun-verify | src/presentation/date_display.rs |  | The Russian genitive was produced on an en-US machine by passing ru-RU as a locale name, never on a Windows installed in Russian. Measured 2026-09-13: a date reads 2 января and a month heading reads Январь, which are different words, so the two mechanisms are doing different things and the test that separates them is real. What that does not settle is whether a Windows whose own language is Russian gives the same answers, or whether a machine missing the NLS data for a language behaves as this one does when asked for a locale it has. The French assertion is weaker still and says so in the test: fr-FR writes juillet both ways, so it passes against the wrong mechanism and would not have caught the fault the Russian one caught. | open |  | 2026-09-13T14:46:34.120Z |  |
| 365 | 06 | deviation | tests/a_move_says_what_has_not_been_sent.rs |  | The first attempt at the 06-03 merge commit failed the full gate on one integration target, a_move_says_what_has_not_been_sent, and nothing explains why. It passed on its own immediately afterwards, all 10 cases, and passed again in the run that completed the merge at e98514b0, and the same target had passed minutes earlier in the full run on the branch. So the merge is green on two full runs and red on one, with no change to the tree between them. Which case failed was not captured, because the merge output was read through tail and the detail scrolled past, which is the part that should not happen again: a transient failure whose text nobody kept is a transient failure nobody can diagnose. This target builds a live window and the run that failed followed another full run closely, so contention is the obvious guess and it is only a guess. Recorded rather than absorbed, because a check that fails one run in three while reading as green is guardrail 4. | open |  | 2026-09-13T15:42:15.765Z |  |
| 366 | 06 | unrun-verify | src/common/catalogue.rs |  | No sentence out of the translation catalogue has been heard by anybody through a screen reader. The four English sentences are byte-identical to what relative_to wrote before, held by the table test under a forced en-US, and that proves structure, not that a listener hears them the same. Waits for the pass after phase 8. | open |  | 2026-09-13T23:12:55.075Z |  |
| 367 | 06 | unmet-truth | locales/en-US/dates.ftl |  | No non-English catalogue exists. On every computer not set to English, 2 days ago is still English, silently, which is criterion 2's own fallback clause and is disclosed on the Reading tab. Which languages Wixen Mail speaks and who writes them is a version 2 decision and Pratik's; the Russian and Polish resources that prove the plural machinery live inside tests and ship nowhere, because a translation nobody here can read is not a translation to ship. | open |  | 2026-09-13T23:12:56.008Z |  |
| 368 | 06 | unrun-verify | src/common/catalogue.rs |  | Whether a Russian listener hears 2 дня назад as natural in a list cell, and whether a number the formatter writes with a U+00A0 group separator is read correctly by a screen reader in that locale, are questions only a Russian Windows, a Russian voice and a Russian speaker can settle. The forms were produced on an en-US machine through a resource written inside a test. Waits for the pass after phase 8. | open |  | 2026-09-13T23:12:56.963Z |  |
| 369 | 06 | deviation | src/service/spellcheck/mod.rs |  | Two readers of LOCALE_SNAME now exist: spellcheck::system_language at spellcheck/mod.rs through GetLocaleInfoW, and how_the_machine_writes_dates::this_computers_locale_name through GetLocaleInfoEx, which chooses the catalogue. Measured agreeing on this machine on 2026-09-13, both en-US. The spellchecker's copy is in src/service, which src/common cannot reach without a new layering direction, and its file is fingerprinted by 30 guard records, so retiring one is version 2's job and not this plan's. | open |  | 2026-09-13T23:12:57.918Z |  |
| 370 | 06 | unrun-verify | src/application/occurrences.rs |  | The repeat-series sentence has never been heard in any language. every week on mardi and jeudi is what a French computer now hears, a French day inside an English frame, held by a test under a forced fr-FR; whether a French listener hears the day as a day, or hears the sentence as a fault, waits for the pass after phase 8 and for version 2's translation of the frame. | open |  | 2026-09-13T23:13:18.748Z |  |
| 371 | 06 | unrun-verify | src/service/signed_mail.rs |  | The eight signature-outcome sentences have never been heard in any language. The date in them now follows the computer in the form a date puts a month in, held by a test under a forced fr-FR asserting 28 août 2026; whether the sentence reads as a date to a listener waits for the pass after phase 8. | open |  | 2026-09-13T23:13:19.744Z |  |
| 372 | 06 | unmet-truth | src/application/repeating.rs |  | Four interface literals still carry English day and month names and are allowed by name in the source-reading guard with a reason beside each: Every weekday, Monday to Friday in repeating.rs and item_fields.rs, and Month first, July 26, Day first, 26 July and A word, July 26, 2026 in wx_settings.rs. They are labels on choices, interface text version 2 translates; a French day inside an English label is not better than an English one. The guard asserts each allowance is still in the tree so the list cannot outlive its subjects. | open |  | 2026-09-13T23:13:20.732Z |  |
| 373 | 06 | deviation | locales/en-US/dates.ftl |  | locales/ maps to no target in the gate. scripts/check.sh scopes tests by src/*.rs and tests/*.rs, house_style's ours() does not walk locales/ and the plan says not to add it, so a commit touching only the catalogue runs formatting, clippy and the tree-reading guards and never common::catalogue::, whose parse and completeness checks read the file through include_str. The whole gate at the merge and CI are what cover it until a mapping is decided with the rest of version 2's questions. | open |  | 2026-09-13T23:13:21.714Z |  |
| 374 | 06 | deviation | tests/a_move_says_what_has_not_been_sent.rs |  | Ledger 365's cause, found by keeping the failure text the second time it fired, on the build(06-03) commit: test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent panicked with No default store has been set from keyring. keyring 4.1.5's Entry::new races its own lazy initialisation: the thread that wins a compare_exchange on an AtomicBool sets the default store, and a thread that loses it goes straight to keyring_core::Entry::new before the winner has finished. Ten tests start in parallel on a fresh process, so it fires about one run in three. Not contention, and not this target's fault. A second finding: the in-memory seam in secret_store.rs is cfg(test), which an integration test never sees, so every run of this target reaches the real Windows credential store. Out of 06-03's scope; a Once around the first Entry::new in secret_store, or a seam the integration targets can see, is the fix. | open |  | 2026-09-13T23:13:22.733Z |  |
| 375 | 06 | unrun-verify | src/presentation/wx_account_manager.rs |  | The three Allow Changes for this account boxes, their section heading and the note beneath have not been heard with a screen reader. Each carries its label on the control itself so the name reaches UI Automation and MSAA both, and the disabled arm's label says why it is unavailable and names the heading in Settings; what only a listening pass can settle is whether a person tabbing through hears why the third box is missing, because Windows skips a disabled control in the tab order and the note beneath is not in it either. Waits for the pass after phase 8. | open |  | 2026-09-14T06:25:29.308Z |  |
| 376 | 06 | unrun-verify | src/application/allowed.rs |  | None of the sending, deleting and syncing a per-account answer governs has ever run against a real account, so what unticking a box on the account dialog holds back has only ever been held back in tests. allowed_for is unchanged and narrows as it always did; the new writer set_allowed_for is covered by three unit tests and no live server. | open |  | 2026-09-14T06:25:29.729Z |  |
| 377 | 06 | deviation | src/application/allowed.rs |  | The refusal a sync says when a per-account answer is holding a change still names Settings and not the account: turn_the_setting_on has one owner and no account in hand, so a person whose Settings box is on hears advice that sends them to a box that is already ticked. The account dialog's note and the testing page say so, and the wording is pinned literally in tests across contacts_sync.rs, calendar.rs, pim_command.rs and answering.rs, so naming both places is a plan of its own. The tree already says it both ways: carddav_sync.rs and address_book_source.rs say Allow Changes is off for this account, which pointed at nothing until 06-04. | open |  | 2026-09-14T06:25:41.365Z |  |
| 378 | 06 | unrun-verify | tests/account_edit_protocol_fields.rs |  | The live-dialog test proves each box is enabled exactly when the stored Settings answer allows it, reading the same profile the dialog reads, so on the machine that ran it, where Settings allows everything, it exercised the offered arm of all three boxes and never the unavailable one. The unavailable arm's wording is proved by a unit test on permission_box_label and by no live box. build_account_edit_dialog reads ConfigManager::load_stored inside itself on the precedent of the directory fields, and there is no seam to hand it a different answer; adding one is the fix. | open |  | 2026-09-14T06:25:41.760Z |  |
| 379 | 06 | unrun-verify | src/presentation/wx_app.rs |  | A reminder found due while somebody is typing is now said and sounded at that look with its window held, and nobody has heard it: whether an Urgent announcement arriving mid-word in the composer, a note or the contacts search is heard as a warning or as an interruption is a listening pass, not an assertion. The rule is tested without a window and the call site is read as text by tests/wired.rs. | open |  | 2026-09-14T09:31:06.902Z |  |
| 380 | 06 | unrun-verify | src/presentation/wx_app.rs |  | The reminder window opens one look, a minute, after the sentence was said, whether or not typing has stopped. That steals focus mid-word eventually, which is the thing being complained about, just later; Pratik accepted that on 2026-09-14 for a warning and a minute. Nobody has heard the window arrive a minute after the sentence, so whether it reads as helpful or as the same thing twice is unsettled. The sentence is not announced again when it opens; the claim that a screen reader reads a dialog's static text when focus arrives in it has not been checked against this dialog. | open |  | 2026-09-14T09:31:07.312Z |  |
| 381 | 06 | unrun-verify | src/presentation/wx_reminder_alert.rs |  | The reminder window's tone comes back once a minute until focus reaches it, ten times at most, on RepeatingTone, which is tested against handed-in instants. Nobody has heard it come back, nobody has judged whether ten tones a minute apart reads as being looked after or as being nagged, and the timer's reading of has_focus on the three buttons and the snooze Choice has never been watched in a running build; on Windows it should be false while another application is in front and true the moment somebody comes to the dialog, and that is a claim about the toolkit rather than a measurement. | open |  | 2026-09-14T09:31:07.724Z |  |
| 382 | 06 | unrun-verify | src/presentation/accessibility/screen_reader.rs |  | Whether a screen reader speaks the reminder sentence while another application is in front is unchecked. UiaRaiseNotificationEvent does not move focus, and NVDA speaks notifications from the foreground process, so a person in Word when a reminder is due may hear only the tone; that is why the tone repeats, and nobody has confirmed either half by ear. | open |  | 2026-09-14T09:31:08.142Z |  |
| 383 | 06 | deviation | src/presentation/wx_reminder_alert.rs |  | After a hold, the gap between the reminder's own tone and the window's first repeat is two minutes rather than one: say sounds at the look that finds the reminder, the window opens a look later, and RepeatingTone counts its minute from the window opening because raise is told only that the sentence was already said, not when. A window that opens at once has the one-minute gap the plan describes. Handing raise the instant of the say would close it and was not built, because 06-09 replaces this window and its opening. | open |  | 2026-09-14T09:31:08.544Z |  |
| 384 | 06 | unmet-truth | .github/workflows/accessibility.yml |  | The scan still fetches releases/latest of Axe.Windows on every run, so the rule set a coverage list describes is whatever Microsoft shipped that morning and can change with no commit here. Pinning is 06-06 task 2, behind a checkpoint Pratik has not answered; until then 06-07's list must carry the version it was read against and the date. | fixed |  | 2026-09-14T10:10:11.730Z | 2026-09-14T11:33:18.715Z |
| 385 | 06 | todo | scripts/msaa-names.ps1 |  | The MSAA half of the scan, the channel NVDA reads, is read by no test and maps to no gate target: a change to it answers affected on a branch and runs formatting, clippy and the tree guards only. The which-checks rule 06-06 added covers .github/workflows/ and not this script, because there is nothing for a workflow-shaped rule to run for it. A test reading its exit-code contract (0 named, 1 unnamed, 2 walk failed), which the workflow depends on at lines 180 to 184, is the missing half. | open |  | 2026-09-14T10:10:12.157Z |  |
| 386 | 06 | deviation | scripts/which-checks.test.sh |  | 06-06 task 1 asked for a plain-markdown docs_only case and a plain-Rust affected case as the allow half; both already existed three times over at lines 95 to 103 and a fourth copy proves nothing. Written instead: a .yml outside the workflows folder answers affected and a document inside .github answers docs_only, the two cases that redden under the wrong spellings of the rule, *.yml and .github/*. Same substitution 07-02 made for the .iss rule. | open |  | 2026-09-14T10:10:12.554Z |  |
| 387 | 06 | deviation | .github/workflows/accessibility.yml |  | 06-06 task 1 said to prove the gate hole by breaking ScanTarget::ALL and watching check.sh pass. That break lives in src/presentation/scan_target.rs, which the gate maps to a scoped target, so it would have run the reading test and gone red: the wrong side of the hole. Measured instead by taking one window out of the workflow's array with the code left alone, which is the change the hole is about. Before the rule: affected, exit 0 in 64s. After: all, exit 101 in 206s, naming blocked-senders. | open |  | 2026-09-14T10:10:12.940Z |  |
| 388 | 06 | unrun-verify | .github/workflows/accessibility.yml |  | 06-06 task 2 wired thirty scan targets where there were ten and every one opened the window it names on a throwaway profile on this machine, as a listing of the process's top-level windows shows. No CI scan has run on any of them: nothing has been pushed since 2026-09-10, so on both channels each new target is a window somebody has opened and nobody has scanned, until 06-08 reads an artifact. | open |  | 2026-09-14T11:33:47.755Z |  |
| 389 | 06 | unrun-verify | scripts/msaa-names.ps1 |  | The MSAA walk now enumerates every visible top-level window the process owns rather than .NET's main window, which is never a dialog. Proved against notepad.exe, one window walked with its title leading each path, and against a process that does not exist, exit 2 where the old script exited 1. Not proved against this application on this machine: the walk of any Wixen Mail window crashes pwsh here, see the next entry, so the first dialog this channel reads is the one CI reads. | open |  | 2026-09-14T11:33:48.209Z |  |
| 390 | 06 | todo | scripts/msaa-names.ps1 |  | Walking any Wixen Mail window over MSAA crashes PowerShell on this machine with STATUS_STACK_BUFFER_OVERRUN, exit -1073740791, under pwsh 7.6.6 and Windows PowerShell 5.1 alike, on the main window alone and before the enumeration change. NVDA is running here and CI has no screen reader; the run of 2026-09-10 walked 1797 elements without crashing. Not diagnosed. The workflow now records any exit other than 0 or 1 as a walk that failed, so if CI meets this it is a named failure rather than a clean pass. Still so on 2026-09-16: 09-05 ran the walk under pwsh on the Signature Manager and on each of the five new editor targets, twice for the editors, eleven runs, exit -1073740791 every time and nothing printed before it, NVDA running, nothing stopped and nothing diagnosed. | open |  | 2026-09-14T11:33:48.643Z |  |
| 391 | 06 | deviation | .github/workflows/accessibility.yml |  | 06-06 task 2 added --alwayssavetestfile to the Axe.Windows call, which the plan did not name. The CLI's own --help says the test file is saved only if errors are found, so the workflow's no-file-means-broken check reported seven of eleven clean windows as scans that failed on 2026-09-10, each a line after the CLI printed 0 errors were found. Rule 1: the distinction the workflow exists to make was inverted for the clean case. | open |  | 2026-09-14T11:33:49.055Z |  |
| 392 | 06 | deviation | scripts/msaa-names.ps1 |  | 06-06 task 2 changed scripts/msaa-names.ps1, which the plan did not name: it walks every visible top-level window the process owns instead of .NET's MainWindowHandle, and a failed walk exits 2 instead of terminating at Write-Error under the Stop preference with exit 1, which the workflow read as an unnamed control. Rule 2: the channel NVDA reads had never seen a dialog, measured from the CI log of 2026-09-10 where three dialogs reported the same 1797 elements. | open |  | 2026-09-14T11:33:49.490Z |  |
| 393 | 06 | deviation | src/presentation/scan_target.rs |  | 06-06 task 2 added mail-module, a thirty-first window beyond the count Pratik answered. With no target given the first-run question opens over the frame on a fresh profile, so main has always been the frame under a modal and the bare main window had never been scanned. Six module targets rather than five, and main left as what it is: the window a fresh profile first meets. | open |  | 2026-09-14T11:33:49.914Z |  |
| 394 | 06 | todo | src/presentation/wx_app.rs |  | Seventeen dialogs open only from inside another window and are outside the scan after 06-06: the account edit dialog, Confirm Delete in the Calendar window, Check Spelling, Insert Table and Preview Before Send in the composer, the contact edit dialog and its Add Email Address, Add Phone Number, Add Address and Add Custom Field, the rule, filter, tag and signature edit dialogs, the wait-for-an-answer window, choose-from-list, and ask-for-a-name. One entry for the layer rather than one per window because these were not in the count Pratik answered on 2026-09-14, which was of windows with their own entry point; whether they are the next widening is his to say, and 06-07's list should say they are outside. | open |  | 2026-09-14T11:33:50.327Z |  |
| 395 | 06 | unrun-verify | docs/wcag-coverage.md |  | 06-07: fifty-two of the fifty-five WCAG 2.2 Level A and AA criteria can only be judged by a person, and nobody has walked any of them against this application. The page says what is left for a person on every row; none of it has happened. Three sentences in three windows have been heard by the NVDA suite and nothing else on any row has. | open |  | 2026-09-14T12:38:35.418Z |  |
| 396 | 06 | todo | docs/wcag-coverage.md |  | 06-07: the applies column of the coverage table is a judgement about applicability to a Windows desktop mail client, made by reading each criterion against what a mail client does, and nobody has checked it against this application. The six no answers are the regulations' and are named; the forty-nine yes answers are one reader's. The row a person disagrees with is the row to correct, with the date. | open |  | 2026-09-14T12:38:35.857Z |  |
| 397 | 06 | todo | scripts/msaa-names.ps1 |  | 06-07: seventeen nested dialogs are outside both scan channels, named in ledger 394 and on docs/wcag-coverage.md. Every yes on the coverage page is a yes for the thirty-one windows the scan reaches and for no other, and the page says so. Not a new entry for the layer; this one says the coverage page depends on 394. | open |  | 2026-09-14T12:38:36.275Z |  |
| 398 | 06 | todo | nvda-tests/README.md |  | 06-07 found, out of scope: the NVDA README says the package exists for two places and its What is in here table lists two test files, where four exist, three running and one skipped. Found while reading the suite for the coverage page's NVDA column. The README under-claims and nothing reads it. | open |  | 2026-09-14T12:38:36.671Z |  |
| 399 | 06 | todo | tests/house_style.rs | 3348 | 06-07 found: test_no_status_page_names_a_version_the_code_does_not_ship reads a WCAG criterion number such as 1.3.1 on docs/IMPLEMENTATION_STATUS.md as a version the code does not ship. The status page names the three criteria by name to stay clear of it. A reading that cannot tell a criterion number from a version is a limitation to know about, not yet a defect worth widening the reading for; if a criterion number has to appear on a status page, that is the moment. | open |  | 2026-09-14T12:38:37.079Z |  |
| 400 | 06 | deviation | scripts/check.sh |  | 06-07: one line added to check.sh's docs_only path, cargo test --lib presentation::what_the_scans_can_judge::, on the help_page precedent. Not in the plan's file list. Without it a commit editing only docs/wcag-coverage.md answered docs_only and ran everything except the reading that holds that page to the code, which is a guard running on every commit except the ones that could break it. Shown working by the docs commit 93f8c865, which ran the eight tests under docs_only. | open |  | 2026-09-14T12:38:37.478Z |  |
| 401 | 06 | deviation | CLAUDE.md |  | 06-07: CLAUDE.md had two copies of the half figure where the plan counted one and said to leave it. Guardrail 2 said covers about half of WCAG across a line break, which the plan's single-line grep could not see, and was wrong the same way the four product copies were; corrected with the date and the old wording. The accessibility section's half of accessibility defects is about defects and was left standing with the qualification added, as the orchestrator asked, rather than left untouched as the plan said. | open |  | 2026-09-14T12:38:37.877Z |  |
| 402 | 06 | unrun-verify | src/application/due.rs |  | 06-09 task 1: the kind-first sentences have not been heard. Task due today: file the report; Task overdue: file the report, was due July 25, 2026; Event in 15 minutes: standup, at 3:00 PM; Event now: standup; Event started 10 minutes ago: standup; and Untitled task or Untitled event for a row with no title. Tests prove the word comes first and the forms are exact; whether the comma before at 3:00 PM reads as a pause or a list, and whether due today is help or nagging, only a listening pass under NVDA can settle. | open |  | 2026-09-14T14:19:15.569Z |  |
| 403 | 06 | deviation | src/application/due.rs |  | 06-09 task 1: an overdue task says its day as a date, Task overdue: file the report, was due July 25, 2026, where the plan's example said was due yesterday. A whole day is read as a date under every style on purpose, the birthday rule in date_display, and a yesterday would be a new relative message for whole days through the catalogue. Not written here; if wanted it is one message and one arm, and the listening pass above will say whether the date form is enough. | open |  | 2026-09-14T14:19:30.620Z |  |
| 404 | 06 | deviation | src/application/due.rs |  | 06-09 task 1: the identity trap's guard record is in two halves and only one is here. The same id under two kinds is two identities is measured, two red. A day of a series carrying its series' id is composed by the event feed, which task 4 writes in wx_app.rs, so there is no code in task 1 for a break to edit; the test dismisses one day and finds the next still due against a fixture that composes id and start the way the feed must. Task 4 owes the record that breaks the feed's composition. | open |  | 2026-09-14T14:19:31.090Z |  |
| 405 | 06 | deviation | src/presentation/wx_app.rs |  | 06-09 task 1: BetweenLooks.already and said_and_waiting are keyed by due::Identity instead of a reminder id string, and the reminder feed builds due::Candidate rows, in the red commit c189a361, because the crate would not build otherwise. wx_app.rs is not in task 1's file list. No test was added to wx_app.rs and its 48 records were not disturbed; the reading in tests/wired.rs still sees the insert into already before the window. Task 4 rewrites this region. | open |  | 2026-09-14T14:19:31.536Z |  |
| 406 | 06 | unrun-verify | src/presentation/date_display.rs |  | 06-09 task 1: how_soon, how_long_ago and time_of_day are public readings with no caller outside due::spoken, and due::spoken has no caller that hands it a task or an event yet: raise_what_is_due still feeds reminders only. Everything task 1 added is reachable by tests and by nothing a person can run until task 4 wires the feeds. Said here so the model is not read as shipped. | open |  | 2026-09-14T14:19:31.989Z |  |
| 407 | 06 | todo | src/presentation/scan_fixtures.rs |  | Scan finding, Account Manager row 7: the IMAP Server cell of the made-up account is empty, a focusable list cell with no name on UI Automation. Either the fixture account names a server or the cell says none rather than nothing; decide which, because naming a server hides the shape a real account with no server would have | open |  | 2026-09-14T15:28:39.547Z |  |
| 408 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 15: the text field of the Starts Day spinner has no name on either channel. set_accessible_name names the up-down arrows, and a Windows spinner is two windows; the field a person types in is the other one. What it takes: get the buddy through UDM_GETBUDDY on the spinner handle from get_handle, then IAccPropServices::SetHwndPropStr with PROPID_ACC_NAME, which the Annotation proxy carries to both channels; needs the Win32_UI_Accessibility, Win32_UI_Controls and Win32_UI_WindowsAndMessaging features. Or a visible static label before each field, which Windows gives the field on both channels and which sighted people would see too. Verified only by the next scan, since nothing here can read a live tree | open |  | 2026-09-14T15:28:39.988Z |  |
| 409 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 16: the text field of the Starts Year spinner has no name on either channel; same cause and same fix as 408 | open |  | 2026-09-14T15:28:40.409Z |  |
| 410 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 17: the text field of the Start time Minute spinner has no name on either channel; same cause and same fix as 408 | open |  | 2026-09-14T15:28:40.839Z |  |
| 411 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 18: the element showing the current value of the Start time AM or PM list has no name on UI Automation. The list carries the name through the MSAA proxy; the value element is the platform's child of it and gets nothing. SetHwndProp on the list handle with the child id, or a visible label before the list, as 408 | open |  | 2026-09-14T15:28:41.287Z |  |
| 412 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 19: the text field of the Ends Day spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:28:41.723Z |  |
| 413 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 20: the text field of the Ends Year spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:28:42.170Z |  |
| 414 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 21: the text field of the End time Minute spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:28:42.628Z |  |
| 415 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 22: the element showing the current value of the End time AM or PM list has no name on UI Automation; as 411 | open |  | 2026-09-14T15:28:43.082Z |  |
| 416 | 06 | todo | src/presentation/wx_item_form.rs |  | Scan finding, Edit Event row 23, unjudged: the Category combo box is reported as not supporting ExpandCollapse. Every combo box in the window is exposed through the MSAA proxy because each carries an accessible object of ours, and the proxy offers ExpandCollapse to none of them, yet only this one was flagged. The one difference in the tree is that it has no child showing a value. To judge it: read the scanner's condition for ControlShouldSupportExpandCollapsePattern in the v2.4.2 source, or open the window on a taller screen and scan again | open |  | 2026-09-14T15:29:12.669Z |  |
| 417 | 06 | todo | src/presentation/wx_item_form.rs |  | Found in the scan's tree, not a scan finding: Edit Event is 720 pixels tall on the runner's 768-pixel screen and its content runs past the bottom. Show as, Status, Category and Times offered are six pixels tall at the bottom edge; the panel inside the tab is 562 tall and the form is taller. On a small screen or at 200 percent text size the last four fields are cut off and the form does not scroll. WCAG 1.4.10 Reflow and 1.4.4 Resize Text, and a keyboard user reaches a field nobody can see | open |  | 2026-09-14T15:29:13.123Z |  |
| 418 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 24: the element showing the current value of the Send on Month list has no name on UI Automation. No static label precedes any control in this window, so nothing is given to it by the platform either. As 411 | open |  | 2026-09-14T15:29:13.548Z |  |
| 419 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 25: the text field of the Send on Day spinner has no name on either channel. The name Send on Day is on the arrows; as 408 | open |  | 2026-09-14T15:29:13.982Z |  |
| 420 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 26: the text field of the Send on Year spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:29:14.419Z |  |
| 421 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 27: the text field of the Send at Hour spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:29:14.853Z |  |
| 422 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 28: the text field of the Send at Minute spinner has no name on either channel; as 408 | open |  | 2026-09-14T15:29:15.269Z |  |
| 423 | 06 | todo | src/presentation/wx_send_later.rs |  | Scan finding, send-later row 29: the element showing the current value of the Send at AM or PM list has no name on UI Automation; as 411 | open |  | 2026-09-14T15:29:15.699Z |  |
| 424 | 06 | todo | src/presentation/wx_item_form.rs |  | Found in the scan's tree, no rule fired: the text field of the Start time Hour spinner is named Start time on UI Automation, the text of the static label before it, which the platform gives to an unnamed edit; the arrows beside it say Start time Hour. On MSAA it has no name and the walk reported it. A name that is present and wrong is one no scanner rule sees; only a person hears it. Fix as 408 | open |  | 2026-09-14T15:29:16.128Z |  |
| 425 | 06 | todo | src/presentation/wx_item_form.rs |  | Found in the scan's tree, no rule fired: the text field of the End time Hour spinner is named End time on UI Automation and nothing on MSAA; as 424 | open |  | 2026-09-14T15:29:16.558Z |  |
| 426 | 06 | todo | .github/workflows/accessibility.yml |  | Scan-level finding 2: the count takes only the first summary a window prints, Select-Object -First 1, so a window that writes two result files has its second count dropped. The reader window wrote two on 2026-09-14, both clean, so nothing was lost that day. Not fixed because nothing in the tree can run that block to prove a sum; a test would need the counting moved into a script under scripts with a suite of its own, run through pwsh, before the line is changed | open |  | 2026-09-14T15:29:16.999Z |  |
| 427 | 06 | todo | scripts/msaa-names.ps1 |  | Scan-level finding 3: the six module targets are one scan repeated. Each walked one window of 1797 elements on MSAA and each msaa-names.json holds the same 91 distinct names, All Calendars, All Contacts, All Notes and Body in Markdown among them in all six. Every module panel is in the window whichever is showing and the walk reads no state, so it cannot tell a hidden panel from the shown one; a nameless control in a hidden panel would be reported six times and a module target proves nothing about its module. What it takes: read accState per element and skip a subtree whose state has STATE_SYSTEM_INVISIBLE, printing how many were skipped so a hidden panel stays visible in the log; the ps1 maps to no gate target, so the change wants a case in a shell suite first | open |  | 2026-09-14T15:29:17.438Z |  |
| 428 | 06 | unrun-verify | src/presentation/html_renderer.rs |  | The scan reached no rendered message. The preview in the main window is a WebView2 control and no target's tree held its document: a fresh profile has no message and the reader target opens a rich edit window. The rendered message is where a sender's headings and links have to survive, and it is the page with the same shape as the editor's, a document with no title loaded from its own address, so its name is likely its own source too. Nothing about it was judged; a target that opens the preview on a made-up message would put it in the next run | open |  | 2026-09-14T15:29:17.882Z |  |
| 429 | 06 | todo | .github/workflows/accessibility.yml |  | Scan-level finding 1: the run summary of 2026-09-14 said 26 findings where the scan log held 29, because the counting pattern matched errors were found and Axe prints 1 error was found; filters, tags and signatures each printed one and were recorded as clean. Fixed the same day, the pattern reads all three sentences and a test in scan_target.rs reads it from the workflow and holds it to them | fixed |  | 2026-09-14T15:29:28.029Z | 2026-09-14T15:29:28.465Z |
| 430 | 06 | unrun-verify | docs/manual-accessibility-pass.md |  | The manual accessibility pass is written and has not been walked. Seventy-six items across six categories, each naming its source and the technology it needs, planned for after phase 8. Until a person has dated every item, nothing in the program has had the pass, and the page says so at the top | open |  | 2026-09-14T15:36:13.798Z |  |
| 431 | 06 | unrun-verify | src/presentation/wx_reminder_alert.rs |  | The Due now window has never been heard. Three rows of three kinds in one ListBox, each sentence beginning with its kind's word, the list named by its count, the sentence for several rows as a count then three rows then how many more, and the tone repeating over a list rather than a line: the live test reads the rows and the labels back from the real controls, and no person has listened to any of it with NVDA or Narrator. Whether 'Task due today: File the report' reads well by ear, and whether 'Event now' for an all-day event at the working-day hour is heard as help, is a listening pass | open |  | 2026-09-14T17:45:39.688Z |  |
| 432 | 06 | unrun-verify | src/presentation/wx_reminder_alert.rs |  | Mark Done on an event row and Details on a task or reminder row are disabled with the reason in the label, 'Mark Done: not for an event', 'Details: not for a task yet'. Whether a person tabbing past a disabled button hears why is not known: Windows skips a disabled control in the tab order, so the label may be read only by arrowing or by a screen reader's review mode, the same question ledger 375 asked of the account window | open |  | 2026-09-14T17:45:40.172Z |  |
| 433 | 06 | unrun-verify | src/presentation/wx_reminder_alert.rs |  | The list and the Come back in picker are named through set_accessible_name, which writes MSAA only; on UI Automation both are named by whatever Windows falls back to, the static text beside the picker and nothing beside the list. 06-02 recorded the same for the settings picker and nobody has checked either channel with the scan or with Narrator. The scan target 'reminder' now opens on three rows so the next Accessibility run reaches the list and all six buttons | open |  | 2026-09-14T17:45:40.688Z |  |
| 434 | 06 | todo | src/application/event_alerts.rs |  | Under decision 2 of 06-09, the default lead fills silence: an event whose stored alerts say nothing is raised default_reminder_minutes before its start. Two kinds of event are therefore given a lead their own calendar may not want. A Google event on the calendar's default alert, which is most of them, gets this program's default rather than the calendar's, because the calendar's default is never read. And every CalDAV event gets the default whatever its VALARM says, because no reader parses a TRIGGER, so a CalDAV event whose alarm was switched off alerts here. The way through is the plan's option 4: the Google pull reads the calendar's default and the CalDAV pull reads TRIGGER, each a sync path with its tests; calendar.rs has 72 records, so that is a plan of its own | open |  | 2026-09-14T17:46:03.202Z |  |
| 435 | 06 | todo | src/application/calendar.rs |  | An alert switched off here is stored as off, an empty list, and the Google push still sends nothing for it, so Google keeps whatever alert it held. local_to_google_event filters an empty override list to no reminders field on purpose, because an alert it could not read must not become 'never alerts'; an explicit off is not that case and could be sent as useDefault false with no overrides, which is exactly what Google means by off. Not changed in 06-09 because it is a sync path with a test in calendar.rs, 72 records | open |  | 2026-09-14T17:46:03.679Z |  |
| 436 | 06 | todo | src/application/due.rs |  | Decision 1 of 06-09 puts a date-only task and an all-day event's alert base at working_day_starts. A reminder set for a day with no time is still due at midnight, the whole-day arm of local_instant, as it has been since reminders first went off, so the two day-shaped things disagree about what hour a day is. Left alone in 06-09 because it is a behaviour change to reminders nobody asked for; whoever decides they should agree changes the reminder feed's raise_at to when_a_day_alerts and ledgers nothing | open |  | 2026-09-14T17:46:04.154Z |  |
| 437 | 06 | todo | src/presentation/wx_reminder_alert.rs |  | The Details button opens only an event. Pratik's answer of 2026-09-14 asked for it on 'the task/event/reminder', and nothing in this program edits an existing task or reminder: PimCommand has no Edit, the item form's Prefill is filled only by filled_from_calendar_item, and no writer takes a Filled onto an existing TaskEntry or ReminderEntry. On those rows the button is disabled with 'not for a task yet' in its label. What it takes: a filled_from_task_item and a task_with_edits on the shape of the event's pair, the same for reminders, and a writer for each; the button then needs only an arm in TheEditors::has_one_for | open |  | 2026-09-14T17:46:04.638Z |  |
| 438 | 06 | unrun-verify | src/presentation/wx_app.rs |  | One look at what is due was timed on this profile only: four reminders, no accounts, an almost empty calendar, 3 candidates in 1 ms and then 0 ms, on the poll once a minute. The calendar's own read cost 68 ms bounded on a six year calendar; the event feed reads three days through the same two seeking queries per source, so it should be far below that, but no real-sized calendar has been under it. If it is not small against the fifty-millisecond tick, the plan's fallback is a startup read refreshed by TasksLoaded and CalendarEventsLoaded, the way reminders are read | open |  | 2026-09-14T17:46:22.636Z |  |
| 439 | 06 | todo | src/presentation/wx_reminder_alert.rs |  | The Due now window closing when its last row is answered, and the next row being selected after one goes, live in a button handler and after_a_row_has_gone, which no test can press: wxdragon 0.9.17 raises no widget event from outside. A break there reddens nothing, so no guard record was written for it and the live test does not claim it. The pure bookkeeping in Rows is guarded; the last-row close is a listening pass | open |  | 2026-09-14T17:46:23.098Z |  |
| 440 | 06 | todo | src/presentation/wx_reminder_alert.rs |  | '1 thing due', '3 things due' and 'And 2 more' are English plurals written in code, the shape 06-03 retired for the date wording by putting it through the catalogue with Fluent's plural rules. Not put through the catalogue in 06-09 because it holds one area, dates.ftl, and its loader and completeness reading are written for one file; a second area, due.ftl, is the catalogue's next step and touches catalogue.rs, six records. When it arrives these three sentences and the window's button labels are the first to move | open |  | 2026-09-14T17:46:23.567Z |  |
| 441 | 06 | todo | src/application/event_alerts.rs |  | Two small holes in what off means, said rather than widened. Microsoft's isReminderOn true with reminderMinutesBeforeStart 0 is stored as nothing, as before, so an Outlook alert at the start of the event gets this program's default lead instead of a lead of nought. And alerts_with_the_first_at with nothing stored and nought in the box keeps nothing stored, which is right, and has no test of its own because managers.rs has 50 records and one more test there is hours of remeasure; the two flipped tests pin the other branches | open |  | 2026-09-14T17:46:24.026Z |  |
| 442 | 08 | deviation | scripts/check.sh |  | plan 08-01 task 1 required check.sh --suites-for guards/guards.toml docs/development/measurements.md to print the new target once its guard record existed; it prints nothing, because the coupling function drops a candidate already in guards_that_read_the_whole_tree on purpose, since every scoped run ends with that list. The record couples: a copy of the script with the target taken out of the list answers with it. The criterion asked for output the tool suppresses by design, and a target in both lists is answered by the whole-tree list first | open |  | 2026-09-14T20:41:00.106Z |  |
| 443 | 08 | todo | scripts/guards.py |  | the docstring of run_the_whole_suite still says the rebuild a break forces is 23 seconds and the library is 89, a pair taken before the suite was halved on 2026-09-09, and the arithmetic built on it, a 220-record sweep from 6.8 hours to 88 minutes, is built on both stale terms; the runner now prints today's terms on every run and the rate row on docs/development/measurements.md holds them, so 08-06's pass over comments in scripts should point this sentence at the page rather than leave a fifth sweep figure in the tree | fixed |  | 2026-09-14T20:41:09.046Z | 2026-09-15T03:20:43.920Z |
| 444 | 08 | todo | docs/privacy.md | 417 | the privacy page's update download size is a target of about 12 MB and not a measurement, because no release has been published to measure; once one exists, measure the installer, write the size on the page with its date, and add a row to docs/development/measurements.md | open |  | 2026-09-14T21:45:39.956Z |  |
| 445 | 08 | deviation | docs/integration-guide.md | 5 | the agreement reading holds every figure shaped N tests on the three test-count pages to a row on the measurements page and cannot tell a past count from a present one, so the guide's historical 'counted 64 tests' was reworded to 'put the count of tests at 64' and the convention (a past count on those three pages is not written as N tests) lives in the reading's section comment rather than anywhere a page author would meet it first | open |  | 2026-09-14T22:05:28.433Z |  |
| 446 | 08 | unrun-verify | tests/the_numbers_the_targets_ask_for.rs |  | The measurement profile's account points at 127.0.0.1 on a closed port so a startup connection would be refused at once, and the connection was never attempted: nothing in the program checks mail on a schedule, so a start dials nothing and the log of every measured run held no WARN or ERROR line. What the application says or shows when a connection is refused was therefore read by nobody in 08-03, and the cold-start and idle figures are for a start that never touches a server. | fixed | Fixed by 10-06 on 2026-09-18: a start checks every enabled account and asks for a watch on each, so the measurement account is attempted at once, and the log of every one of the five re-taken runs was read. It is refused before the port is dialled, by the credential store and not the socket, because the profile stores no password and an empty password cannot be stored (ledger 374): "Could not watch the inbox of The measurement account: Authentication error: No password is saved for The measurement account on this computer", then "Trying the watch again in 30 seconds", a second refusal at about 40 s, a third at about 100 s, and "left to the schedule alone until TheServerAnswersAgain". What is said on screen is "Error: Authentication error: No password is saved" at High, once per attempt of the check. The cold-start and idle rows are re-taken under that condition on docs/development/measurements.md. What a refusal at the socket says is ledger 526. | 2026-09-14T23:50:37.822Z | 2026-09-18T04:51:18.000Z |
| 447 | 08 | unrun-verify | docs/development/measurements.md |  | Idle memory on the measurements page is idle with a refusable account that was never dialled. Idle on a machine with a real account and a live connection is a different idle: a folder watch, a sync on new mail and a live WebView2 preview all run then and none ran here. The 120 s reading of 391 MB, of which the application process is 56 MB, says nothing about that case, and no real account has ever been used with this program to take it. Corrected on 2026-09-18 by 10-06: idle now includes a start that checks the account and asks for its watch, a watch refused three times and tried again after a growing wait, and the account left to the schedule alone, and the rows were re-taken under that condition; still no live connection, because the measurement account is refused by the credential store before anything is dialled, so the case this entry names, a watch held open on a real server and a sync on new mail, is still untaken. | open |  | 2026-09-14T23:50:38.308Z |  |
| 448 | 08 | deviation | docs/development/measurements.md |  | The cold-start row reports the first start after the binary was built on its own as the file-cache-cold figure, 520 ms against the series median of 476 ms. That figure depends on what else the machine had read: the linker had just written the binary so much of it was in the file cache already, and WebView2's own binaries were warm from the earlier runs. A start after a reboot with the disk cold was not taken and would be a different number; the definition says what was measured and this entry says what it does not cover. | open |  | 2026-09-14T23:50:38.786Z |  |
| 449 | 08 | unrun-verify | tests/the_list_at_two_hundred_thousand_rows.rs |  | The sort's apply is not timed: apply_sort clones the rows, sorts them off the interface thread and sends MessagesLoaded, and the cost of the list control taking 200,000 rows back needs a window the harness does not have. The sort rows on the measurements page say so; a number for the apply waits for a harness that drives the running program. | open |  | 2026-09-15T01:35:59.293Z |  |
| 450 | 08 | unrun-verify | tests/the_list_at_two_hundred_thousand_rows.rs |  | A scroll's own paint is not timed: the page paint row is text_for over one page of every inbox column, and wxWidgets' painting of those cells needs a window the harness does not have. A scroll in the running program is the row's figure plus that, and the measurements page says so beside the row. | open |  | 2026-09-15T01:36:09.797Z |  |
| 451 | 08 | todo | tests/the_list_at_two_hundred_thousand_rows.rs |  | THE_SEARCH_BOXES_LIMIT copies the LIMIT inside managers::search_messages, which is private to that function, so the filter rows are timed at 500 because the harness says 500 and not because it read the program. If the search box's limit moves, the harness times the old one; making the constant reachable from the harness is the fix. | open |  | 2026-09-15T01:36:10.285Z |  |
| 452 | 08 | deviation | .planning/REQUIREMENTS.md |  | PERF-05's [S] line and roadmap criterion 3 attribute low coverage to service/protocols, service/oauth and the provider clients; on 2026-09-14 those read 92.06%, 84.55% and 96.75% against a library at 83.34%, so the attribution names areas that are no longer low; 08-06 corrects the evidence line and 08-09 closes the clause with the reason | fixed |  | 2026-09-15T02:14:27.761Z | 2026-09-16T07:40:27.988Z |
| 453 | 08 | todo | docs/development/measurements.md |  | The low coverage area on 2026-09-14 is the wxWidgets windows, src/presentation/wx_*.rs at 26.88% holding 73% of the missed lines, outside the three areas PERF-05 attributes and not attributed by 08-05; these files build windows that no --lib test opens, and 08-09 decides whether that is a gap to close, a different command to measure with, or a figure to accept with the reason beside it | fixed |  | 2026-09-15T02:14:28.223Z | 2026-09-16T07:40:28.519Z |
| 454 | 08 | deviation | scripts/guards.py |  | 08-06 edited the docstring of run_the_whole_suite, which the plan's file list did not name, because ledger 443 had assigned it to 08-06 and the plan's own done criterion is that no comment in the tree states the sweep's cost as a figure; the plan was written from the research and the README, neither of which carried the entry, so an assignment written into the ledger alone did not reach the plan it was addressed to | open |  | 2026-09-15T03:20:52.629Z |  |
| 455 | 08 | deviation | docs/IMPLEMENTATION_STATUS.md |  | Three dated quotations of a retired figure were reworded to keep the figure without the phrase, because the plan's acceptance criteria were single-line greps for the old phrase finding nothing while its rule 1 requires the old figure kept as the figure of its date: the status page's mutation sentence, guards.toml line 40 and guards.sh line 36 now say the run was put at two days or at one or two hours rather than quoting the words; the meaning is unchanged and CLAUDE.md, whose criterion admitted a dated sentence, quotes all four phrases as written; an absence criterion over prose that also requires the quotation has to be scoped to the sentence and not to the phrase | open |  | 2026-09-15T03:21:02.849Z |  |
| 456 | 08 | deviation | scripts/guards.py |  | 08-07 task 1: --resume is refused without --log, where the plan only said a bare --resume takes the --log path; a run that records its verdicts nowhere cannot itself be resumed, so the refusal prints the flag to add rather than measuring and losing the result | open |  | 2026-09-15T04:28:16.036Z |  |
| 457 | 08 | deviation | scripts/guards.py |  | 08-07 task 1: a build that starts and finishes inside one record's run is seen by neither the poll before the record nor the poll after it, so such a record is measured beside a build and not marked contended; the plan accepts this as the cheapest reading the log can carry and the changelog says so | open |  | 2026-09-15T04:28:16.506Z |  |
| 458 | 08 | deviation | scripts/mutants.sh |  | 08-08: the every-target mutation shape cannot run in a scratch copy, because cargo mutants copies the tree without .git and test_the_share_of_history_before_red_green_is_computed_and_printed runs git merge-base, so that baseline fails in the copy; the plan asked for a scratch-copy rate under every target and it was measured in place instead, which is the only way it runs | open |  | 2026-09-15T08:19:55.953Z |  |
| 459 | 08 | deviation | scripts/mutants.sh |  | 08-08: a scoped mutation run, the third option the checkpoint offers, has no shard or resume support: --shard and --shards divide the whole list and pass nothing to -f, so a run over one area today is the older scripts/mutants.sh DIR mode, one process a kill loses whole; if option 3 or 4 is chosen, the shard modes gain a --file glob first, red first | open |  | 2026-09-15T08:19:56.464Z |  |
| 460 | 08 | deviation | docs/development/measurements.md |  | 08-08: the per-mutant rate rows come from one shard whose 25 mutants all sit in src/presentation/accessibility.rs, a file most of the presentation layer depends on, so each rebuild ran 58 to 75 s where a one-file change elsewhere rebuilt 44 to 46 s; the products are what the tree would cost if every file were that file, and no leaf module was measured | open |  | 2026-09-15T08:19:56.948Z |  |
| 461 | 08 | deviation | docs/development/measurements.md |  | 08-08, for 08-09: the whole-tree mutation run is deferred by Pratik's answer of 2026-09-15; 12,391 mutants at 2847391c, 130 s a mutant under every target in place and 194 s a shard over 496 shards, about 19.8 days of this machine, or two dispatches of 248 runners at an unmeasured rate; the run over src/service/protocols, 450 mutants in 18 shards, is what this milestone makes, and criterion 4 is revised under criterion 6 with 08-08's product table as the reason | open |  | 2026-09-15T10:58:17.286Z |  |
| 462 | 08 | deviation | .github/workflows/mutants.yml |  | 08-08, for 08-09: mutants.yml ran on pull_request only, and this project merges to main without pull requests, so the diff-scoped mutation check in CI had never run once since it was written; a workflow_dispatch with mode=diff and a since ref now exists, and it has not been dispatched | open |  | 2026-09-15T10:58:17.811Z |  |
| 463 | 08 | deviation | scripts/guards.sh |  | 08-08: the guard sweep could be sharded onto GitHub's runners the way the mutation shards now are, one runner per chunk of records with --stop-after and the log as the artifact, and it is not; the sweep still runs on this machine from 08-07's checkpoint | fixed |  | 2026-09-15T10:58:18.330Z | 2026-09-15T11:45:40.115Z |
| 464 | 08 | deviation | .github/workflows/ci.yml |  | 08-08: found by the push of main at 0fa393ba on 2026-09-15, CI run 34956059032, Test Suite job 104338271778: test_the_share_of_history_before_red_green_is_computed_and_printed failed on the runner because the checkout fetched one commit and git merge-base could not see 18a02454; fixed at abf3e24c with fetch-depth 0 and a reading that holds every job running cargo test to it; the fix is unconfirmed on a runner until the next push | open |  | 2026-09-15T10:58:18.818Z |  |
| 465 | 08 | deviation | .github/workflows/guards.yml |  | 08-07, the answer: WIXEN_TEST_THREADS is left at the script's default of 8 on a 4-core runner, unmeasured, so the runner's timing lines and the rate row on docs/development/measurements.md read against each other at one setting; the thread curve on CLAUDE.md was taken on 24 cores and says nothing about 4, and verdicts are what the sweep is for. Re-take the curve on a runner if the shards run slower than the sizing guess | open |  | 2026-09-15T11:45:40.600Z |  |
| 466 | 08 | deviation | src/presentation/wx_app.rs |  | debug theme-reach-crashes-on-runner, 2026-09-15: wxWidgets 3.3.2 (wxdragon 0.9.17) delivers WebView2 creation completions to a destroyed control (wxWidgets #26491, fixed upstream for the unreleased 3.3.4). presentation::browser_ready now holds Compose and Preview Before Send until their browsers report. The main window's preview pane (wx_app.rs preview WebView) and the conversation-as-headings frame are destroyed only at application exit; an exit inside the creation moment, about 250 ms warm and seconds after a runtime update, ends the process with 0xc000041d after the window is gone and nothing else is wrong. Left as it is until a wxdragon release vendors 3.3.4, at which point browser_ready can be retired | open |  | 2026-09-15T12:50:15.759Z |  |
| 467 | 08 | unrun-verify | tests/theme_reach.rs |  | debug theme-reach-crashes-on-runner, 2026-09-15: the crash of theme_reach on GitHub's runners (runs 34956059032 and 34961574447, exit 0xc000041d) was reproduced here only through a scratch case tearing a WebView down at once, never through theme_reach itself, whose two browsers finish in a quarter of a second on this machine. The wait added to theme_reach and the child-process test in closing_a_window_before_its_browser_exists are unconfirmed on a runner until the next push of main; the CI Test Suite job is the confirmation | open |  | 2026-09-15T12:50:16.261Z |  |
| 468 | 08 | unmet-truth | src/application/contacts_sync.rs |  | 08-07 task 3: the gate the_copy_here_was_written_here on the note call in the read that skips a contact the setting holds back is covered by no test since ce3e89ba of 2026-09-05 rewrote the one test that reached it; the record named for it reddened nothing on the runners and here and was retired. A test that notices belongs in contacts_sync.rs, which 77 records name, so it waits for somebody prepared to re-measure those | open |  | 2026-09-16T01:54:07.526Z |  |
| 469 | 08 | todo | src/application/caldav_sync.rs |  | 08-07 task 3: the seen_uids insert of the compound id at the top of one_caldav_day_kept_out_of_its_series is redundant, because the loop over the server's answer already marks the changed day's stored identity seen before the day is folded; taking it out reddens nothing on the runners or here. Its record now breaks the marking that holds the rule; the insert is a dead-code candidate to remove with a reading that says why | open |  | 2026-09-16T01:54:08.175Z |  |
| 470 | 08 | deviation | guards/guards.toml |  | 08-07 task 3: two records are right on this machine and blind on a runner, and are left as written. An hour with no zone means an hour here: the runner's clock is UTC, so a break that sends the local hour as UTC changes nothing there. The walk into Windows own chain structures really happens: the runner has no chain to walk. A runner sweep reports both as a named test staying green; read them as measured here, or give each a fixture that does not depend on the machine | open |  | 2026-09-16T01:54:08.848Z |  |
| 471 | 08 | unmet-truth | src/application/caldav_sync.rs |  | 08-07 task 3: the rule that a changed day of a CalDAV series is marked seen before the removal pass can run has no test that would notice it broken: taking out either seen_uids marking leaves test_syncing_a_caldav_moved_day_twice_does_not_duplicate_it_or_delete_it green, measured on the runners and here. The record of 2026-08-14 that named it was renamed to the one fact the break really guards. What keeps the moved day now was not reconstructed; a test that breaks when the marking goes is the answer | open |  | 2026-09-16T02:33:25.593Z |  |
| 472 | 08 | deviation | src/service/protocols/imap.rs |  | 08-08 mutation run survivor, untested behaviour: imap.rs:494:9 replace ImapStream::into_plain with None survived; the one caller is the STARTTLS upgrade, which no loopback test reaches because no test server here speaks TLS; needs a loopback server with a certificate | open |  | 2026-09-16T06:53:05.804Z |  |
| 473 | 08 | deviation | src/service/protocols/imap.rs |  | 08-08 mutation run survivor, untested behaviour: imap.rs:527:9 replace poll_flush with Poll::from(Ok(())) survived; equivalent on the plain stream, where tokio's TcpStream::poll_flush is always ready, and untested on the TLS stream, where a skipped flush leaves the record layer's buffer unsent | open |  | 2026-09-16T06:53:06.311Z |  |
| 474 | 08 | deviation | src/service/protocols/imap.rs |  | 08-08 mutation run survivor, untested behaviour: imap.rs:534:9 replace poll_shutdown with Poll::from(Ok(())) survived; a skipped shutdown is invisible to a loopback test that drops the socket and matters on TLS, where the close-notify never goes out | open |  | 2026-09-16T06:53:06.819Z |  |
| 475 | 08 | deviation | src/service/protocols/pop3.rs |  | 08-08 mutation run survivor, untested behaviour: pop3.rs:99:9 replace Pop3Stream::into_plain with None survived; the STLS upgrade, the same gap as IMAP's, needing a loopback server that speaks TLS | open |  | 2026-09-16T06:53:07.322Z |  |
| 476 | 08 | deviation | src/service/protocols/pop3.rs |  | 08-08 mutation run survivor, untested behaviour: pop3.rs:132:9 replace poll_flush with Poll::from(Ok(())) survived; equivalent on the plain stream and untested on TLS, the same as IMAP's | open |  | 2026-09-16T06:53:07.799Z |  |
| 477 | 08 | deviation | src/service/protocols/pop3.rs |  | 08-08 mutation run survivor, untested behaviour: pop3.rs:139:9 replace poll_shutdown with Poll::from(Ok(())) survived; the same as IMAP's | open |  | 2026-09-16T06:53:08.277Z |  |
| 478 | 08 | deviation | src/service/caldav.rs |  | 08-08 mutation run survivor, untested behaviour and the runner shape of ledger 470: caldav.rs:202:9 replace CalDavClient::for_account with Default::default() survived; the constructor reads this machine's stored settings through ConfigManager::load_stored, so a test asserting the allowed client passes where the settings allow it and reads wrong on a runner with no settings file; the constructor wants its answer as an argument before it can be pinned | open |  | 2026-09-16T06:53:08.779Z |  |
| 479 | 08 | deviation | docs/plans/20260915-whole-tree-mutation-run.md |  | 08-08 mutation run, six survivors recorded as equivalent with the reason on the page rather than killed: mailbox_name.rs:193:5 and 251:30, caldav.rs:1097:20, 1687:18, 1789:25 and 1812:35; two of them are equivalences the code's own comments already claimed and the run has now confirmed; recorded so the next round does not triage them again from scratch | open |  | 2026-09-16T06:53:09.266Z |  |
| 480 | 08 | unrun-verify | .planning/REQUIREMENTS.md |  | 08-09: the provider question PERF-03's title asks, a real mailbox of 100,000 messages or more from a live provider, has not been asked, because no real account has ever been used here; the list question is answered by 200,000 synthetic rows on the measurements page, and the requirement's third [D] line says the provider question waits for a live account. docs/roadmap.md and docs/development/requirements-backlog.md carry the line half answered. When an account exists, open a mailbox of that size, and time the first listing, a search and a sort from the running program rather than the harness. | open |  | 2026-09-16T07:40:18.005Z |  |
| 481 | 08 | unrun-verify | src/presentation/wx_app.rs |  | 08-09: nothing this phase changed that a person meets has been confirmed with a screen reader. 08-03 made the window fill the module it opens on at startup, so the folder tree and the message list are there on a fresh start instead of after a mail check or a module switch, and nobody has heard what NVDA or Narrator says at that moment or whether focus lands somewhere useful; the usable line the harness reads is a log line and nothing speaks it, by design. The check is one item for docs/manual-accessibility-pass.md: start the release build against a profile with mail in it and listen to the first thing said. | open |  | 2026-09-16T07:40:18.547Z |  |
| 482 | 08 | deviation | .planning/REQUIREMENTS.md |  | 08-09: PERF-01 and PERF-04 are ticked on a reading, not on a number that meets the target either way. The application process is 57 MB with 1,000 cached messages and 56 MB idle, under 150 MB and 100 MB; the six WebView2 processes Windows runs for the preview pane weigh about 333 MB beside it, so the sum is 390 MB and 391 MB and misses both. The targets were written before the preview was a browser and do not say whether they count it. The coordinator's reading, put to Pratik on 2026-09-14 and not contradicted, is the application process alone, and both boxes are ticked on it with the tree's weight written beside the target wherever it is judged. If Pratik reads the target as the sum, untick PERF-01 and PERF-04, change the two [D] lines added 2026-09-16, and revise the targets or the preview. | open |  | 2026-09-16T07:40:19.070Z |  |
| 483 | 09 | unrun-verify | .github/workflows/release.yml | 118 | The as-is level has never been dispatched: that cargo-release 1.1.5 given its current version on the runner plans no bump, skips the commit on a clean tree and tags v1.0.0-alpha.1 is read from a dry run on this machine and from commit_all in its ops/git.rs, not from a run; the first as-is dispatch is Pratik's and is what proves it | open |  | 2026-09-16T14:30:27.270Z |  |
| 484 | 09 | unrun-verify | src/service/spellcheck/mod.rs | 349 | Whether a profile created before 2026-09-03, holding the bare en every profile got then, now shows English (United States) in Settings and is checked in it without a hand change is a run on such a profile; the tester's own profile has held a hand-set en-US since 2026-09-15, which the resolver leaves as stored, so his machine cannot show the fix and only a profile still holding the bare value can. 09-02 proved it on this machine through the real General tab built in a test, not on a profile | open |  | 2026-09-16T17:19:56.885Z |  |
| 485 | 09 | unrun-verify | src/data/message_cache/bodies.rs | 690 | The once-only pass that puts stored snippets right has run against a temp profile holding one HTML-only message (1 row in 3 ms, log line quoted in 09-02's summary) and never against the tester's 20 MB cache of 12,872 messages; how many of his rows it rewrites, how long his first start takes, and whether his rows then read as words are his first open of the next build to answer, and the log line says the first two | open |  | 2026-09-16T17:19:57.441Z |  |
| 486 | 09 | deviation | src/application/answered_meetings.rs | 162 | 09-03: a meeting answer is filed on the calendar the moment Accept, Tentative or Decline is pressed, while the reply is still held for ten seconds like any other message. Undo Send inside the hold takes the reply back and leaves the meeting on the calendar as answered; answering the same meeting again replaces the entry, so somebody who undoes and answers differently ends with the calendar right, and somebody who undoes and does not answer has an entry the organiser never heard about. Filing only when the queue drains needs the send loop to reach back to the calendar, which nothing does, and is a feature of its own. Said in the changelog under Known limitations. Ledger 155's listening question, whether anything spoken after a mistaken Accept points at Undo Send, stays open: the sentence now says Undo Send takes it back, and nobody has heard it. | open |  | 2026-09-16T18:19:35.079Z |  |
| 487 | 09 | unrun-verify | src/presentation/wx_app.rs | 14212 | 09-04: the look the plan asked for at the running program, sort by sender from View, Sort and then by date from a column header and open the submenu, was not made. tests/one_sort_is_checked_on_a_live_menu.rs asks a real menu bar the same question by id, one group answers one tick and the old shape four from the moment it is built, and tests/one_sort_is_checked.rs holds the application's chain to one group; what neither reaches is the application's own menu after a real header click through sync_sort_menu, which is the next build's View, Sort to answer, and it is a look rather than a listening pass | open |  | 2026-09-16T19:36:32.825Z |  |
| 488 | 09 | unrun-verify | src/presentation/wx_settings.rs | 1264 | 09-04: whether the Reading tab now reads as one group by ear, Default sort order and then Then by as consecutive tab stops under NVDA, is FOUND-06's listening line and has not been listened to. tests/the_sort_controls_sit_together.rs builds the real dialog and reads the sibling chain, which is the order Tab moves in, and finds only Then by's own label between the two; that is structure present, and the tester who reported #36 is the one who can say whether it is experience good, along with whether Cc and Bcc lines is where he would look for it on the Compose tab | open |  | 2026-09-16T19:36:44.529Z |  |
| 489 | 09 | unrun-verify | src/presentation/scan_target.rs | 232 | 09-05: the five editors are scan targets and were each opened here on a throwaway profile and seen (Edit Contact, Edit Condition, Edit Filter Rule, Edit Signature, Edit Account, each owned above the frame), and the MSAA walk on each, run once before task 2 and once after as the machine is, left with -1073740791 every time, ten runs, as ledger 390 records, NVDA running and the cause not diagnosed; so the names of their checkboxes on the channel NVDA reads have been read by nothing, the before and after rows #42 asks for do not exist, and roadmap criterion 5's walk clause and FOUND-08's second [D] line stay open until the Accessibility workflow has walked the five at the next push. What was read instead is the Win32 child list of each window, which shows the empty statics gone and is not a name | fixed | Fixed by 11-02 on 2026-09-18: the Accessibility workflow walked the five editors in run 35336142914 on main at 744d05ef, the push of that morning. The log reads Walked 'Edit Contact', 'Edit Condition', 'Edit Filter Rule', 'Edit Signature' and 'Edit Account', each with 'Wixen Mail', and the MSAA walk on each ended '0 without a name' (2915, 1978, 2126, 1889 and 3120 elements). FOUND-08's second [D] line is closed on that run. The walk still crashes on this machine, ledger 390, and nothing here ran it | 2026-09-16T21:25:20.597Z | 2026-09-18T13:30:00.000Z |
| 490 | 09 | unrun-verify | src/presentation/wx_managers.rs | 67 | 09-05: what NVDA says on the signature editor's Default signature box and the contact editor's Favorite box after the change is FOUND-08's last [S] line and has not been heard. Both boxes, and three more built the same way, now carry set_accessible_name with the mnemonic stripped and no empty static text before them; tests/checkbox_labels.rs reads the built windows and finds an accessible object on every one of the fifteen editor checkboxes and no nameless static before any, which is structure present. Why the tester heard the two unnamed is still not settled by anything read here: a native checkbox carries its own window text and the MSAA walk that would say what the channel reported could not run. The next instrument is NVDA's own log on his machine, Tab through the signature editor with the log at debug, if the next build still reads the box as unnamed | open |  | 2026-09-16T21:25:36.495Z |  |
| 491 | 09 | deviation | tests/no_label_is_only_a_space.rs | 60 | 09-05: the plan's premise 5 said the widened reading would catch all eleven spacers and none of the eleven filled lines with a rule about which calls take the binding later; that rule flagged three filled lines (a static handed on bare from a block, one handed to the dialog's own filler, the live region written through Win32) and would have needed an allow list holding filled lines rather than what task 2 left. The rule written instead refuses a binding whose only later use is a sizer add, which on the tree at 1d934e26 refuses exactly the five in wx_managers.rs and none of the filled lines, and does not see a spacer handed on in a tuple to be stored and hidden, the account editor's old shape for its six; the module comment says so, tests/checkbox_labels.rs reads the built tree for the fifteen editor checkboxes, and there is no allow list because an empty one watched by a test is the census-emptying failure | open |  | 2026-09-16T21:25:37.084Z |  |
| 492 | 09 | unrun-verify | nvda-tests/tests/settings-tabs-read-once.test.js | 1 | 09-06: whether each Settings tab is heard once along the tab row is FOUND-09's listening line and has not been heard. What is held: the native tab control's own arrow handler raised EVENT_OBJECT_FOCUS twice on the reached tab per key (scripts/uia-events.ps1, 2026-09-16, before) and raises it once now that the row answers its arrows through SetSelection (the same script, after); tests/the_settings_tab_row_says_each_tab_once.rs sends a real WM_KEYDOWN to the built dialog and counts one, which is structure present. What NVDA says about one event is the NVDA case's to answer, on CI at the next push of main, which is Pratik's, and the tester's ear after; neither has run. Roadmap criterion 6's transcript clause and FOUND-09's third [D] line stay open until that run | fixed | Fixed by 11-02 on 2026-09-18 on the tester's word: heard on 1.0.0-alpha.1+149.g744d05ef with NVDA, the Settings dialog speaks as it should and arrowing along the tab row says each tab once (#33, closed 2026-09-18T11:34Z). The runner's case ran once, in run 35336142908 at 744d05ef, and heard nothing because it waited for an opening announcement the harness never captures; 11-02 corrected the wait, and whether the corrected case hears each tab once on the runner is ledger 531 | 2026-09-16T22:44:18.512Z | 2026-09-18T13:30:00.000Z |
| 493 | 09 | deviation | scripts/uia-events.ps1 | 1 | 09-06: the plan prescribed a UI Automation event logger and read a capture of one event per key as meaning the second reading was NVDA's own; the managed UI Automation client did show exactly one ElementSelected per key and nothing else, and it is blind to what NVDA reads for a native SysTabControl32, which is MSAA and win events. The logger logs both channels, the win-event hook reading class, text and child id only and never an IAccessible (ledger 390). Two conditions of the capture: the session was locked (the focused element was the Lock Screen, pid 16028), so SendInput answered ERROR_ACCESS_DENIED and SendKeys threw, keys were posted as WM_KEYDOWN to the tab control's own window, and no window could take foreground focus, so the page-panel candidate is judged from the in-thread focus events (the row took focus at open, both captures) and wxWidgets' UpdateSelection giving the page focus only when the notebook has none, not from a foreground run; and NVDA was running and not stopped. The fix also takes the numpad's arrows, found because the test's first key lacked the extended bit and wxWidgets read VK_RIGHT as WXK_NUMPAD_RIGHT | open |  | 2026-09-16T22:44:35.134Z |  |
| 494 | 09 | todo | nvda-tests/README.md | 60 | 09-06: the README's What is in here table lists two test files and the directory holds five (calendar-immediate-actions, filter-manager-delete and settings-tabs-read-once are not in it), and its prose says two tests where the workflow runs five. A table that is read as the inventory and is short by three is a check nobody reads; bring it to the directory, or have a reading hold it there | fixed | Fixed by 11-02 on 2026-09-18: the table lists all five files under tests/ with what each holds and whether it runs, the prose dates the two the package began with and counts the five, and the workflow section says four run and one is skipped. No reading holds the table to the directory; the count is a sentence dated 2026-09-18 | 2026-09-16T22:44:35.707Z | 2026-09-18T13:30:00.000Z |
| 495 | 09 | unrun-verify | src/presentation/reader_text.rs |  | Whether the preview pane's bar and a conversation's per-message sentences read well by ear has been heard by nobody: preview_html renders the top of the bar as a region named Security warning above the message, and one_of_several says why a PGP message did not open under its own heading on the page and in the text reader. Both are held by tests against a key and a message GnuPG made and a signed message OpenSSL made; whether they sound right with NVDA, and whether a real correspondent's key opens anything, is FOUND-10's last [S] line, the tester's (09-07, #51). | open |  | 2026-09-17T01:01:46.443Z |  |
| 496 | 09 | todo | src/presentation/wx_app.rs |  | import_a_pgp_private_key announces its outcome and puts nothing in the status bar; its comment said both until 2026-09-16 and was corrected to what the body does. A visible line to match the spoken one is owed, on import_a_mailbox's pattern, which sends the status and announces; it wants a red in tests/wired.rs and ui_tx and runtime passed to the function (09-07, #51 item 6). | open |  | 2026-09-17T01:02:01.809Z |  |
| 497 | 09 | todo | src/presentation/reader_text.rs |  | A PGP signature inside a conversation of several messages is still not mentioned there: SIGNED_AND_NOT_CHECKED_HERE is folded by with_encryption for one message only, and one_of_several says the PGP opening reason and the S/MIME envelope but nothing about a clearsigned part. Opening the message on its own says it; the changelog's dated correction on the armour entry says so (09-07). | open |  | 2026-09-17T01:02:02.382Z |  |
| 498 | 09 | deviation | .planning/phases/09-what-the-first-day-of-testing-found/09-07-PLAN.md |  | 09-07 executed with four departures: the wired.rs guards were edited in task 1 rather than task 3 because body_of panics on the moved fn line and task 1 could not compile its verify otherwise; task 2's second guard record went on reader_text's behaviour (the reason dropped from one_of_several) rather than on a call-site bypass, and task 3's record covers the call site; three stray rustdoc blocks moved home rather than one, the folder loader's and the module loader's beside the mailbox import's; and a third changelog entry, the armour entry's thread limitation, was dated beside the two the plan named. The key import's status-bar sentence was corrected in the comment, not built, because the task's one red was spent (ledger todo beside this). | open |  | 2026-09-17T01:02:02.964Z |  |
| 499 | 09 | unrun-verify | src/application/importing_an_outlook_data_file.rs |  | No real Outlook data file has been through the import: neither this program nor the outlook-pst crate can write one, so brought_in's walk over a real file's folders (opened, what_it_holds, each_item_in) is unrun, and what is tested is the filing of each of the five kinds handed in by one_folder_filed. The closing sentence says so to whoever runs it. FOUND-11's [S] line; the tester's, when he has a .pst to hand. | open |  | 2026-09-17T02:49:01.548Z |  |
| 500 | 09 | todo | src/service/outlook_data_file.rs |  | The reader words every refusal as a sentence to be heard and carries it in Error::Other, whose display puts Error: in front, which a screen reader says first. importing_an_outlook_data_file::as_it_was_worded takes the words out where the closing sentence is built; the reader itself should move its sentences onto Error::InPlainWords, which is a change to the reader wanting a red of its own and was outside 09-08's rule of touching the reader only to make something public. | open |  | 2026-09-17T02:49:19.475Z |  |
| 501 | 09 | todo | src/presentation/wx_app.rs |  | After an Outlook data file is imported the folder tree is read back, as after an archive, but the calendar, contacts, tasks and notes modules are not: they show what arrived when next opened, and the changelog says so. The worker should send the module's own reload for each kind that arrived, the way folder_tree_updates does for mail, with a wired.rs reading holding it. | open |  | 2026-09-17T02:49:20.052Z |  |
| 502 | 09 | unrun-verify | src/presentation/wx_app.rs |  | Nobody has opened a file Save As wrote in another mail program, and nobody has run Save As by hand: what is held is that this program's own reader reads the written bytes back as the message that went in with its file (export_tree tests), that a kept signed original is written byte for byte, and that the window asks the decision and the writer (wired.rs). The dialog, the destination and the status line are the tester's. | open |  | 2026-09-17T02:49:20.635Z |  |
| 503 | 09 | deviation | .planning/phases/09-what-the-first-day-of-testing-found/09-08-PLAN.md |  | 09-08 executed with five departures: the new module's tests hand it items of each kind rather than reading a fixture, because the reader's header and the crate's README say no data file can be written, so the walk over a real file is a thin untested half named as such; export_tree.rs gained one_message_written_out, a file the plan did not list, because the exporter already owns stored-message-to-bytes and its file walk; the archive import's folder helper moved from wx_app.rs to importing_messages::a_folder_for_imported_mail so three imports share it; the wired.rs reading of the import worker's three-way dispatch arrived green in task 2's green commit rather than red in task 1, its behaviour having been taken red there; and the reader's Error::Other sentences have their words taken out at the seam rather than the reader changed (todo beside this). | open |  | 2026-09-17T02:49:21.208Z |  |
| 504 | 09 | unrun-verify | tests/the_settings_dialog_opens_in.rs |  | Whether Settings now feels immediate on the tester's machine is his to say (FOUND-12's [S] line, #34). The harness measures from Ctrl+, to the moment before show_modal: 2,206 ms before 09-09 and 397 ms after, median of five in the release binary with NVDA running in this session. The show, the focus landing and the screen reader's first announcement come after that moment and nothing here times them; nobody has heard the dialog open since the change. | open |  | 2026-09-17T06:19:04.138Z |  |
| 505 | 09 | todo | src/service/spellcheck/mod.rs |  | try_load_spellbook leaks both halves of every Hunspell dictionary it loads through Box::leak, on every for_language call, so a machine with a Hunspell dictionary loses the dictionary's size in memory each time a checker is built: each compose window, and until 09-09 every open of Settings. Pre-existing, found while reading the loader for 09-09 and not changed there because it wants a red of its own. spellbook::Dictionary wants 'static text; an Arc or a once-per-process cache of the parsed dictionary is the shape. | open |  | 2026-09-17T06:19:04.752Z |  |
| 506 | 09 | todo | src/presentation/wx_settings.rs |  | With the window shown and NVDA running, every control in the Settings dialog costs about twice what it costs on a hidden frame, and a Choice control costs 10 to 60 ms each, the language list's 19 names in many scripts 24 ms hidden and 61 ms shown. That is what is left in the release binary's 397 ms after 09-09: the General page's controls under the screen reader's in-process hooks, plus the configuration read. Found by a scratch run of 2026-09-16 that showed the harness's frame and matched the release binary within three percent; the mechanism is inferred from that and not diagnosed further, because stopping the tester's NVDA to take the number without it was not asked for. Fewer Choice controls on General, or a build off the interface thread, would move it; neither was done. | open |  | 2026-09-17T06:19:18.152Z |  |
| 507 | 09 | todo | src/presentation/wx_settings.rs |  | Since 09-09 a Settings page after General is built the first time its tab is reached, so the first Right arrow onto Reading pays that page's build: 156 ms on a hidden frame in the test process (the row of 2026-09-17), about twice that under NVDA by the scratch run's factor. The other five are under a tenth of a second hidden. Whether a pause on the first visit of Reading is felt, and whether filling the pages after the dialog shows with the control saying so would be better, is the tester's to say; the plan allowed either shape and this one was chosen because a page is never seen empty. | open |  | 2026-09-17T06:19:18.744Z |  |
| 508 | 09 | deviation | .planning/phases/09-what-the-first-day-of-testing-found/09-09-PLAN.md |  | 09-09 executed with four departures. The fix is not either shape the plan named: the rows said the three lists cost a millisecond each and the pages the rest, and a scratch timing of each page, reverted, found a spell checker built for one sentence and released (210 ms) and an unfrozen typeface list resizing itself per item (1,090 ms with the window shown), so the change is a frozen build, a source named without a checker, and pages built when their tab is first shown, the last being the shape the plan and FOUND-12 offered for a page cost. The harness drives the release binary itself, posting the Settings menu command to a window it started, rather than a person pressing Ctrl+, on a real profile of this machine, so no run touched the tester's profile and the number is repeatable. The dialog is timed five times in the test process rather than once, and a sixth build times each tab's first visit. Task 1's line is written in show_settings_dialog beside show_modal, from an Instant handle_settings is handed, rather than in handle_settings itself, because show_modal lives there. | open |  | 2026-09-17T06:19:32.442Z |  |
| 509 | 09 | unrun-verify | src/presentation/wx_app.rs |  | Nobody has run File, Import a Folder of Messages by hand since 09-10 added it, and no Thunderbird profile folder has been read through it. The folder walk is proven by mailbox_archive's own tests over folders its tests make and by the wired.rs readings that hold the item to a DirDialog and the hand-over to the worker; what a real folder of somebody's saved mail becomes, and what a Thunderbird profile becomes (one folder per mailbox file, each .msf refused and counted, the .sbd nesting one level out of place), is written in the changelog and the guide from reading the walk, not from running it. FOUND-11's third [D] line; #53 point 3. | open |  | 2026-09-17T07:17:05.342Z |  |
| 510 | 09 | todo | src/service/mailbox_archive.rs |  | The folder import does not recognise Thunderbird's layout: a mailbox file with no ending beside a .msf index and a .sbd folder of subfolders. Read as loose files, a profile's mail directory becomes one folder per mailbox file, each .msf a refused non-mail file counted in the sentence, and the folders inside Inbox.sbd landing under a folder called Inbox.sbd beside Inbox rather than inside it. Recognising the layout means treating name.sbd as the children of the mailbox file name and skipping .msf without counting it. Said in the changelog and the guide since 09-10 (#53 point 3); later work with points 4 to 6. | open |  | 2026-09-17T07:17:22.879Z |  |
| 511 | 09 | deviation | .planning/phases/09-what-the-first-day-of-testing-found/09-10-PLAN.md |  | 09-10 executed with four departures. The item's letter is O, not the plan's F, which Fetch Missing Message Text already has on the File menu; test_no_two_items_on_one_menu_claim_the_same_letter would have refused the plan's spelling, and the green commit wrongly said no check read menus, corrected in the next commit. The shortcuts page had no row for Import Mailbox beside which to add one, so four rows were added: Import Mailbox, Import a Folder of Messages, Export Mailbox and Import PGP Private Key. The import handler was split into an_account_to_import_into, refuse_to_import and mail_brought_in_from so the two pickers share the readiness check and the worker start, and the older wired.rs reading of the worker's shape was re-pointed at the shared function and named in the red trailer. The changelog's gathered Known limitations paragraph for issue 53's points 4 to 6 went in with task 2, as the plan said, after being drafted and withdrawn during task 1 so task 1's commit carried only its own words. | open |  | 2026-09-17T07:17:23.479Z |  |
| 512 | 10 | deviation | .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-01-PLAN.md |  | 10-01 executed with five departures, so 10-05 reads the signatures it will call from the tree and not from the plan. what_to_do_next takes TextStillMissing { messages, kept_bytes } where the plan had the list alone, because a budget on bytes kept needs the bytes kept as an input and the plan's signature had no place for them. The sentences write bare numbers, 3500 of 12872, as every counted sentence in the tree does, where the plan's examples grouped thousands; grouping is one helper and every counted sentence if Pratik hears the bare form as harder. The whole-list run is fetch_all_the_missing_text, generic over Mailbox, with fetch_the_missing_message_text delegating to it as before, because the fold has to be tested against the scripted mailbox and the entry point takes a real controller. The IMAP timeout phrase is imap::THE_SERVER_STOPPED_RESPONDING, read by the classifier, because a timeout and a dropped connection are both Error::Network and kind alone cannot tell them apart; pop3.rs still writes the same phrase as a literal. The stopped-coming-down rule was re-pointed at green rather than red, because the red stub answers false and the old loop's test would have run forever under it. | open |  | 2026-09-17T11:50:58.000Z |  |
| 513 | 10 | unrun-verify | src/presentation/wx_settings.rs |  | What only the tester's ear settles for #67 and #68, both fixed by 10-01.1 and measured over MSAA and GetFocus() from a test that builds the real dialog: that a later-page Settings checkbox is spoken as check box with its state, that Space says the new state, that the state reads back after Tab away and back, that OK keeps it, that Settings still opens at once, that an arrow on the tab row still says each tab once, and that Ctrl+Tab from a General control speaks a named control and nothing before it. The listening lines belong to 10-07's page; the issue-close comments carry them until then. | open |  | 2026-09-17T14:05:00.000Z |  |
| 514 | 10 | unrun-verify | src/presentation/theme.rs |  | Four files both paint a window with theme::paint and build checkboxes on it, and none has been read for whether the paint comes before or after the build: wx_account_manager.rs, wx_compose.rs, wx_item_form.rs and wx_managers.rs. A checkbox created under an already painted panel inherits its text colour, is made owner-drawn by wxWidgets, and reads as a push button under NVDA, which is what #67 was on the Settings pages. A tree-wide reading that no built wxCheckBox is BS_OWNERDRAW, on the shape of tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs, is separate work; said in the changelog's Known limitations for #67. | open |  | 2026-09-17T14:05:00.000Z |  |
| 515 | 10 | unrun-verify | src/presentation/wx_app.rs |  | What only the tester settles for #24, fixed by 10-02 and measured by the harness at his size and at 200,000: whether his folder of 12,872 messages opens and reads as one list with his screen reader on his machine, and whether the list still answers keys at once after a folder change, which is MAIL-02's last line. Nobody has opened a folder of 200,000 in the running program, only the harness has read one, and the first open of one that size pays under a second on the interface thread by the rows named The list's own read path after 10-02 on docs/development/measurements.md, dated 2026-09-17; moving that read off the interface thread is later work and the page says what it would buy. | open |  | 2026-09-17T16:10:00.000Z |  |
| 516 | 10 | unrun-verify | src/presentation/wx_app.rs |  | What only the tester settles for #69, fixed by 10-02.1: whether, with All Inboxes open, choosing Oldest first from View, Sort Messages, arrowing to a folder and back to All Inboxes reads the oldest first with his screen reader, and the same for a label view and after running a saved search; and whether Unread First from the menu, which was saved as read first until 10-02.1, now puts the unread rows first on the next read of a folder. The composed run through load_every_inbox is held by three links and not by one test: the cache answers in the order it is handed, the sort the menu stores reads back into that clause, and the window's three readers are held to asking for it by a reading of the source, because the loaders and the_sort_as are private to the window and its own test module reads the machine's profile. | open |  | 2026-09-17T18:50:00.000Z |  |
| 517 | 10 | unrun-verify | scripts/build-installer.sh |  | What only a build and a machine settle for the build counter 10-02.2 added (Pratik's decision of 2026-09-17): that the next installer handed to the tester carries the counter in its file name and in Apps and Features, as 1.0.0-alpha.1+N.g<commit> and a file version ending in the counter; that installing it over the 1.0.0-alpha.1+g59c5b6a4 build he has is read by Windows as an upgrade rather than refused as a downgrade; and that --version and the log's first line show the counter. The tests read the script's text and order the encoded fields; one installer was built from the branch by the executor and its --version and file version read back, but no build with the counter has been installed over the alpha.1 build anybody has, and no build with it has been handed to anybody. | open |  | 2026-09-17T19:52:50.000Z |  |
| 518 | 10 | deviation | .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-03-PLAN.md |  | 10-03 executed with four departures, so 10-05 reads the seam it will call from the tree and not from the plan. The record on the screen's read-back is measured on a new target, tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs, rather than on every_event_has_a_control or the_settings_dialog_opens_in as the plan offered, because neither of those reads the Permissions page and a break there reddened nothing in either; the new target builds the real dialog, chooses each size and reads it back the way OK does. The record on the check's worker being handed the setting is measured now on a fourth reading in that target rather than deferred to 10-05's target as the plan said to do if no existing target reddened, because the reading is the same reading and earlier. The two worker sites read the setting through one helper, how_much_message_text_stays, rather than each repeating the six lines, and that helper is where the read-by-something guard sees the field's name. The attachment budget's comment, which said the two halves kept the whole cache around a gigabyte, was corrected with the date, because the sentence became false the moment the body half became a setting. | open |  | 2026-09-17T21:53:34.000Z |  |
| 519 | 10 | unrun-verify | src/presentation/wx_settings.rs |  | What only the tester's ear settles for the choice 10-03 added under Message Text on the Permissions tab: that Alt+K reaches it and NVDA says its name, Keep the text of messages on this computer, then combo box and the current answer; that the four answers read as All of it, Up to 1 GB, Up to 5 GB and Up to 20 GB and Up or Down moves between them; that the sentence under it is read once in passing and says what leaves, when and what stays; and that OK keeps the answer across a restart. Whether the eviction then honours a chosen size against his account is settled only by a mailbox with more than that much text, which his 12,872 messages may or may not hold, and the default keeps everything, so the first sign of the setting working is text that stays where 0.125.1 dropped it, which nothing on his machine has measured. The listening lines belong to 10-07's page. | open |  | 2026-09-17T21:53:34.000Z |  |
| 520 | 10 | deviation | .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-04-PLAN.md |  | 10-04 executed with five departures, so 10-05 and 10-06 read the kinds from the tree and not from the plan. The new-mail signal lives in the WhatArrived arm rather than inside spawn_mail_sync as the plan's grep criterion said, because the worker holds no accessibility handle and the arm is what its end-of-check update reaches; the criterion as written could not be met. The two new variants and ModuleSyncFinished are on ui_types.rs, which the plan's file list did not name, because that is where UIUpdate lives. The tasks and notes syncs finish through a new ModuleSyncFinished update rather than the existing completion updates, and the notes sync's two answers that no sync ran stay on the answer channel, because they answer the key rather than report a sync. The result sentence counts through how_many, Inbox, 3 new messages, rather than the plan's Inbox, 3 new, so a listener hears what the number counts. The whole-folder request's closing report goes out as a step with its progress, shown and not spoken under the default, because the loop hands over one kind of line and the command retires with 10-05. The setting's changelog entry landed in task 1's commit, on the rule that the entry goes with the setting, and task 2 extended it; task 3 wrote the listening lines. | open |  | 2026-09-18T00:21:57.000Z |  |
| 521 | 10 | unrun-verify | src/presentation/wx_app.rs |  | What only the tester's ear settles for #38: what a check of his 50 folders says under each of the three answers on the Feedback tab, and which sentence is a step and which a result by ear, since the sorting was done by reading each line's words and a line sorted wrongly is silent under the default or spoken under it; whether the one result sentence, folder by folder with counts, is heard as an ending rather than as another line; whether the sound for new mail followed by that sentence reads as one event or two; whether Settings saved is now heard when OK is pressed during a check; and whether the choice itself is reached by Alt+W and read as its name, combo box and answer, with the sentence under it read once. Nothing here met a real account, and nobody has listened. Items 42 and 43 on docs/manual-accessibility-pass.md. | open |  | 2026-09-18T00:21:57.000Z |  |
| 522 | 10 | deviation | .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-05-PLAN.md |  | 10-05 executed with six departures, so 10-06 and 10-07 read the runner from the tree and not from the plan. Nine tests in wx_app.rs read the retired offer and commands, not the five the plan counted by line range, and all nine were rewritten in place so the count stayed 199. Get Older Messages answers the key through send_status, the answer channel, rather than as Progress, because 10-04 sorted that line as an answer and a key pressed into a silent step is pressed again; the runner's own lines are Progress. The runner sends MoreOfTheFolderArrived per chunk rather than FolderMessagesArrived, because the arm and its guard record already existed for exactly that update. The runner sends one WhatArrived per account that had anything to do, from its own function, because the check's list has gone out hours before a download ends and the reading holds spawn_mail_sync alone to one. The whole of application::asking_for_a_whole_folder is deleted rather than its loop alone, because nothing but the loop reached its sentences. UIUpdate::WhatCouldBeFetched keeps its name and its three managers.rs tests, because what it counts is unchanged and renaming it would flag fifty-two records. start_the_download refuses to start while a wait after a refusal is running, Get Older Messages and Pause included, so a key does not undo the wait. | open |  | 2026-09-18T02:49:01.000Z |  |
| 523 | 10 | unrun-verify | src/presentation/wx_app.rs |  | What only the tester's account and ear settle for #20 and #23. The account: whether Gmail tolerates 12,872 messages coming down five hundred at a time and their text fifty at a time after every check, what it does when it has had enough, whether a refusal is read as the run stopping short rather than as a folder finished, and whether a wait of thirty seconds doubling to thirty minutes suits it; ledger 11 and 72. The ear: whether Pause Downloading on Tools is read as a check box with its state and its description; whether its two answers, and Get Older Messages' answer that this folder comes first, are heard above the download's own lines; whether a first download of 50 folders under Say what arrived is one sentence at its end with the sound for new mail, and under Say every step a line per chunk; whether the sentence a search says about text that is on its way is heard beside the coverage sentence rather than over it. The runner was driven through readings and the model's tests only; no loopback IMAP server exists as a process to point the release binary at, so no binary was started. | open |  | 2026-09-18T02:49:01.000Z |  |
| 524 | 10 | stub | src/application/mail_sync.rs |  | mail_sync::fetch_the_missing_message_text and fetch_all_the_missing_text, the whole-list text pass, are reached by nothing outside their own tests since 10-05 retired Fetch Missing Message Text and the offer above the message list; the download of everything asks fetch_over_a_mailbox one chunk at a time instead. They stay because fifteen tests hold the fold, its wording and the reading gate through them, in a file thirteen guard records fingerprint at 149, and their retirement is a rewrite of that suite: the tests move onto fetch_over_a_mailbox or go, the records' red lists and counts are corrected by hand and re-measured, and what_the_fetch_did, says_where_it_is, about_to_fetch and the Backfill enum go with them or are kept by a caller. The doc comment on the entry point says so. | open |  | 2026-09-18T02:49:01.000Z |  |
| 525 | 10 | unrun-verify | src/presentation/wx_app.rs |  | What only the tester's account and ear settle for #37. The account: whether Gmail drops an IDLE connection at all, after how long, and whether the watch started again after a wait of thirty seconds doubling to thirty minutes carries mail in over hours; whether a watch per enabled account counts against a provider's connection limit, ledger 67; whether a start that checks every account and asks for every watch at once is welcome; what a scheduled check every five minutes of fifty folders does to a mailbox that the watch already covers. The ear: whether the three status lines, watching, waiting to watch again, and checking every N minutes, are heard as states rather than as three sentences that sound alike, with the account named first for a person with two accounts and never for a person with one; whether the first line of F9 under two accounts, Checking 2 accounts for new mail, is heard; whether the sentence under the Check Interval field on the account editor is read with the field and again beneath it, and whether twice is too many; whether an error every five minutes for an account with no password saved is a flood or a reminder. Nothing here has met a real server or been heard. | open |  | 2026-09-18T04:51:18.000Z |  |
| 526 | 10 | unrun-verify | tests/the_numbers_the_targets_ask_for.rs |  | What the program says when a connection is refused at the socket is still read by nobody. The measurement profile's account points at 127.0.0.1 on a closed port, and since 10-06 a start attempts it, but it is refused by the credential store before anything is dialled, because the profile stores no password and an empty password cannot be stored through the harness (ledger 374, the keyring race). So the refusal read for ledger 446 is an authentication refusal, and the connection-refused path through the watch's start, NeverStarted from watch_folder, and the check's the_session_at, has been driven only by unit tests against a closed port and never through the running binary. A harness that can store a password, or a loopback listener that closes the socket, is what would read it. | open |  | 2026-09-18T04:51:18.000Z |  |
| 527 | 10 | deviation | .planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-06-PLAN.md |  | 10-06 executed with departures, so 10-07 reads the watch from the tree and not from the plan. The restart decision answers three things, not two: AfterAWait, Never for a stop somebody asked for, and OnTheScheduleAlone with what would lift it, because a stop the window itself asked for when replacing a watch must not mark the account, and a server that could not be reached and a server that refused to watch are lifted by different facts. A watch already reading events is left alone by a request, so a scheduled check does not replace a working watch every five minutes. The download is asked for only after a check in which some account went through, as 10-05's tree did, found by running the release binary. The schedule follows the network and does not run while the program believes there is none, so an unreachable server is not an error every interval while the network is gone. mark_synced marks the worker's copy and update_account_last_sync writes the column, not save_account, which would write the credential store on every check. The watch's wait is on InboxWatch per account beside its handle, not on a second map. The account editor's sentence is on the field as its accessible description and beneath it as text. The measurement account is refused by the credential store and not the port, ledger 526. | open |  | 2026-09-18T04:51:18.000Z |  |
| 528 | 10 | stub | src/data/account.rs |  | Account.last_sync is written by every check that goes through since 10-06, on the worker's copy through mark_synced and in the row through update_account_last_sync, and read back by load_accounts into a field nothing reads: the schedule keeps its own clock for the session in WxUIState.last_checked, and a start checks every enabled account whatever the row says. The column is true now where it was empty before, and it is a fact with no reader. A reader would be a start that skipped an account checked a moment ago by a previous run, or a status line that says when the account was last checked; neither exists, and this entry says so rather than leaving the column to read as consumed. | open |  | 2026-09-18T04:51:18.000Z |  |
| 529 | 10 | todo | docs/privacy.md |  | The privacy page's table row for OneNote says Never, nothing here reads or writes a notebook, and the section under it, The OneNote permission which nothing uses, says no notebook has ever been opened and no note written here goes anywhere. Both were true when written and false since phase 5.2: notes on an Outlook or Office 365 account sync to OneNote, docs/ALPHA_TESTING.md says so under what is known to be missing or unproven, and the Calendar and PIM tab says it is experimental. Found on 2026-09-18 by 10-07 reading the page for every sentence phase 10 falsified; left because correcting it means reading phase 5.2's summaries for what a notes sync sends, which is outside this phase, and a dated sentence beside each of the two claims is the shape the page uses. Until then a person reading the privacy page is told a permission is unused that the notes sync uses. | open |  | 2026-09-18T05:59:52.000Z |  |
| 530 | 11 | unrun-verify | tests/the_language_the_screen_shows_is_the_one_used.rs |  | The en-AU case of this target, the one that failed on GitHub's runner in CI run 35336142985 at 744d05ef, cannot be run red on this machine, because Windows here offers en-AU and the runner does not; 11-01 fixed the rule in presentation::which_language_row, held it by six cases over hand-built rows that are red on both machines before the rule, and left this target unchanged and green here. Whether the runner now keeps en-AU is settled by the next push of main, which is Pratik's; until that run is read this entry stands, and the run's Test Suite job is the reading. | open |  | 2026-09-18T13:04:42.000Z |  |
| 531 | 11 | unrun-verify | nvda-tests/tests/settings-tabs-read-once.test.js | 160 | 11-02: the corrected settings case takes its mark after the settle and presses Right without waiting to hear General, because its first run (35336142908 at 744d05ef) waited for an opening announcement no transcript of any case has ever held and timed out with an empty log. It never runs on this machine (nvda-tests/README.md) and runs at the next push of main, which is Pratik's. What that run shows: which tab the first Right reaches on the runner's fresh profile. If the tab row holds focus at open, Compose, and the six Rights and one Left are counted; if not, the case fails with 'never heard: Compose', a finding about focus at open on a fresh profile and not about the row. The tester's ear has settled the row itself (#33); this entry is the harness's own case | open |  | 2026-09-18T13:30:00.000Z |  |
| 532 | 11 | deviation | .github/workflows/accessibility.yml | 44 | 11-02 took continue-on-error off the NVDA job and left it on the Accessibility scan job on purpose. The scan's findings are counts per window the workflow already reads out (violations, unnamed, broken): 20 violations over 36 windows in run 35336142914 at 744d05ef, on windows 11-02 does not touch (new-event 9, send-later 6, compose 4, accounts 1). Whether a count should fail the run is a decision for the phase that owns the findings, not 11-02's. Until it is taken, the scan's badge is green over any count and the report is what to read | open |  | 2026-09-18T13:30:00.000Z |  |
| 533 | 11 | unrun-verify | src/presentation/wx_folder_choice.rs | 286 | 11-03: Folders to Keep Up to Date is a tree with the control's own check boxes, and what only the tester's ear settles for #70: that a kept folder is heard as checked and an unkept one as not checked, that Space says the new state after it toggles, that a nested folder's level is read, that the title is heard as the account's name, that the All Mail sentence under the tree is reached and understood on his Gmail account, and whether his Gmail lists All Mail at all (his folder list holds none; Gmail's Labels, Show in IMAP setting decides). The readings prove the state on TVM_GETITEMSTATE and over MSAA, the nesting, the cursor, the sentence and the title's argument | open |  | 2026-09-18T15:12:32.000Z |  |
| 534 | 11 | deviation | src/presentation/accessibility/names.rs | 140 | 11-03: wxdragon 0.9.17's acc_state constants carry MSAA's numbering (CHECKED 0x10, FOCUSABLE 0x100000, SELECTABLE 0x200000), its C++ shim hands a GetState answer to wxAccessible unconverted, and wxWidgets numbers its own wxACC_STATE_SYSTEM enumeration differently (BUSY 0x10, PROTECTED 0x100000, READONLY 0x200000) before converting to the platform's, so every state written through those constants arrives as a different state. Measured over AccessibleObjectFromWindow: CHECKED arrived as BUSY and SELECTABLE as READONLY, which is what the tester heard. The one writer in this tree, CheckedRows, is retired; nothing writes a state through them now, and the paragraph at this line says so. Upstream defect, not reported yet | open |  | 2026-09-18T15:12:32.000Z |  |
| 535 | 11 | unrun-verify | src/presentation/wx_app.rs | 22706 | 11-04: the log's default follows the version and the lines a report needs are written at info and debug, and what only a report from the tester's machine settles for #71: whether each check's per-folder line, the download's chunk lines, the settings save line and the held-back line are the lines that make his next problem diagnosable, and what a day at Debug on his real account costs on his disk, which a two-minute start against the measurement profile cannot say. The readings prove each line's presence and level and that no call spells a secret; nobody has written a report from a log at this level | open |  | 2026-09-18T17:52:00.000Z |  |
| 536 | 11 | deviation | scripts/which-checks.test.sh | 248 | 11-04 found, not fixed: the suite's scratch-repository fixture runs git -C <scratch> init, config, add and commit, and a hook exports an absolute GIT_DIR and GIT_INDEX_FILE when the commit is made from a linked worktree (measured 2026-09-18 with a hook printing its environment: unset in the main checkout, absolute in a worktree), so from a worktree every fixture command acts on the real repository: init marked it bare, config overwrote hooksPath and the user, add staged the fixture's four-line Cargo.toml into the worktree's index, and commit landed that beside the planner's staged files on main as b4a4cc81 under the fixture's message and committer. The repository config was restored by hand at 16:55Z, and the planner undid the commit (main's reflog at eb5d8517: "planner: undo the suite's stray commit made through the hook from a linked worktree") and landed its plans as 1ae63359. The fixture itself is unchanged and will do the same on the next commit made from a linked worktree: it must clear GIT_DIR, GIT_INDEX_FILE, GIT_WORK_TREE and GIT_PREFIX before its first git, with a case that exports an absolute GIT_DIR and asserts the real repository did not move. A second trigger, found by 11-06 on 2026-09-18 from the main checkout: a commit that names its paths, or git commit --amend --only, hands the hook a temporary GIT_INDEX_FILE (.git/next-index-<pid>.lock), the fixture's "a version bump staged in a repository of its own" case then answers all rather than affected, and the gate refuses the commit; nothing moved that time, since the temporary index goes with the refused commit, and the remedy is the same clearing | fixed | Fixed by 11-06.3 on 2026-09-19: scripts/shell-suite.sh unsets GIT_DIR, GIT_WORK_TREE, GIT_INDEX_FILE, GIT_PREFIX and GIT_COMMON_DIR right after set -uo pipefail, in the harness every suite sources, so before any suite's first git (d5c3483e). Two cases in which-checks.test.sh are red if that stops (cdf04ff8), each handing a fresh bash the harness and one variable the way the hook hands a suite its environment, against a throwaway repository named elsewhere and never this one: under an absolute GIT_DIR the fixture's commit landed on elsewhere's HEAD and replaced its hooks path; under an exported GIT_INDEX_FILE the fixture wrote to the handed index and the subject answered all. After the green, from the primary worktree, GIT_DIR set to this repository's absolute git dir and then GIT_INDEX_FILE set to a copy of its index each ran the suite to exit 0 over 52 cases with HEAD, the config checksum, the reflog length and the copy unchanged. The --only half, two commits sharing COMMIT_EDITMSG, is not this and still stands in CLAUDE.md | 2026-09-18T17:52:00.000Z | 2026-09-19T02:00:00.000Z |
| 537 | 11 | unrun-verify | tests/the_numbers_the_targets_ask_for.rs | 433 | 11-04: the two size rows for docs/development/measurements.md, the log a two-minute start writes at info and at debug against the measurement profile, were not taken. One copy of Wixen Mail runs at a time and the tester's copy was open on his account through the whole session (process 12648, INBOX, 14400 unread); the one attempt at 17:37Z handed itself to that copy, which was raised and said "Wixen Mail is already running, and this is it", and a run started the moment his copy closes would take the single-instance slot from a restart. The harness now refuses to start while any wixen-mail.exe is running, with the reason, and pins the level through WIXEN_MEASUREMENT_LOG_LEVEL. The rows are owed: run the two commands on the harness's header with no Wixen Mail open, write the rows with their date and commit, and quote the sizes on docs/ALPHA_TESTING.md | open |  | 2026-09-18T18:05:00.000Z |  |
| 538 | 11 | unrun-verify | src/presentation/page_jumps.rs |  | 11-04.1: what only the tester's ear settles for #84. In the formatted view: Alt+A from the message landing on the attachments list with "Attachments, N" said, and whether that sentence after the list announces itself is one thing too many; Alt+A from the list going back with "Message"; F7 to the warning bar with "Security warning" and F7 back; "No attachments" and "No warning" when there is nothing to go to. In the plain-text reader: Alt+A to the list and back, where the way back is answered by the menu accelerator when the list has focus, because the frame takes an accelerator before a list box sees the key; that was reasoned from how the toolkit routes a chord (wxTextCtrl exempts Ctrl+arrows, a list box exempts nothing) and never watched, and if the chord reaches the list instead, the list's own handler answers it. The readings hold the shape; none of this has been heard | open |  | 2026-09-18T19:12:00.000Z |  |
| 539 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-05, amended by 11-05.1 on 2026-09-18: what only the tester's ear settles for #25. A walk through a folder with unread messages, letting NVDA finish every row, leaving the unread count where it was; Space once on an unread message leaving the count where it is, however long the row stays selected; Space again, the whole reading, then the count moving after two seconds with the row still selected; Shift+Space the same; Enter on one doing the same; moving off a message before the delay runs leaving it unread; and the sentence under Mark as read after on the Reading tab read once, on the choice's own row and not twice. Whether "previewed" in his words meant reading aloud from the list was asked in the first close comment and answered the same day: reading the snippet is not reading, which is why the first Space counts for nothing now. The rule's six cases, the decision's cases and the readings hold the shape; none of this has been heard | open |  | 2026-09-18T20:05:00.000Z |  |
| 540 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-06: what only the tester's ear settles for #27. The Action menu's item heard as Mark as Unread after arrowing onto a read message and as Mark as Read after arrowing onto an unread one, and the context menu's entry the same; M on a message heard as "read" or "unread", one word, and the list staying on the same row after it; the toolbar button's name after a toggle, read from the button under NVDA's own navigation; and Alt+A, E reaching the item whichever way it goes. The readings hold the letter consumed on a built list, the relabel read back over MSAA on a built toolbar, and the three refresh sites in the source; none of this has been heard | open |  | 2026-09-18T21:30:00.000Z |  |
| 541 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-06.1: what only the tester's ear settles for #76. After Delete on a middle row, NVDA reading the next message's row once and not twice when the watch's re-read follows; after Delete on the last row, the previous row read once; the same after Move to Trash and after a move out of the folder; and the preview showing the landed message. A built list holds the four cases and the focus event the landing raises; which of the two paths the tester met was not watched, because a delete cannot be driven here while the tester's copy is open, and the built list showed the last-row case leaving the control holding nothing | open |  | 2026-09-18T23:59:00.000Z |  |
| 542 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-06.1: what only the tester's ear settles for #83. "Delete" heard once at the key and nothing after it when the delete went through, the landed row's reading being enough on its own; the refusal heard with its reason when a delete fails, for instance with the network off; Move to Trash and Move to Folder the same, and Copy to Folder's "Copied to" still heard because its row stays; and the status bar's fuller line, "Deleting" then "Moved to Trash", read on request with NVDA+End and not otherwise. The readings hold the arm to the one word, the outcome to the shown channel when the row left and the spoken one when it stayed, and the shown channel to speaking nothing; none of it has been heard | open |  | 2026-09-19T00:16:00.000Z |  |
| 543 | 11 | unrun-verify | src/presentation/list_arrival.rs |  | 11-06.2: what only the tester's ear settles for #87. Tab from the folder tree into the message list, with a folder just opened, NVDA reading the newest message's row once and not twice; F6 into the list the same; back to the tree and Tab again landing on the row that was left, with nothing moved; and an empty folder's list saying "No messages" once. A built tree and list hold the four steps and recorded the focus events the arrival raised, the list itself then the row twice where before it raised only the list itself; whether NVDA reads the row once from that sequence is his ear | open |  | 2026-09-19T01:20:00.000Z |  |
| 544 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-07: what only the tester's ear settles for #30 and the thread clause of #27. Shift+Down growing the selection with NVDA saying "selected" for each row added and reading the row; Shift+Up shrinking it with "not selected" for the row that leaves; the count after Ctrl+A on its own; one sentence after Delete over several, "Delete" once and the row after the set read once when they have gone; "3 messages marked read" once after Mark as Read over three and nothing per message; M on a conversation row marking the whole thread and saying "1 conversation, 5 messages marked read"; and the refusal above 5,000 heard once with the count. The built list holds the events the control raises and the source readings hold the seven arms to the set; none of it has been heard | open |  | 2026-09-19T03:44:00.000Z |  |
| 545 | 11 | deviation | tests/house_style.rs |  | 11-07: test_every_guard_record_still_names_one_place_in_the_tree exempts a whole file when any record's after is in the tree and its before is not, taken as the file being mid-measurement, so one record whose after matched the rewritten cursor handler by coincidence hid five other records of wx_app.rs whose before had left the tree, and the check passed green over six unmeasurable records; found by an independent one-place count run by hand on 2026-09-19 and the six rewritten and measured. The exemption should be per record, and a pass through it should say so | open |  | 2026-09-19T03:44:00.000Z |  |
| 546 | 11 | unrun-verify | src/application/moves_waiting.rs |  | 11-07.1: what only the tester's ear and a real server settle for #86. A move leaving the row at once under NVDA with the cursor read on the next message; Enter on the chosen folder in the Move dialog moving the message; a move made with the network off put back and the refusal heard once when the server answers no; a move made with the network off and the program closed, replayed at the next check after a restart with the message in the destination and not brought back by the check; a copy heard as "Copied to Work" once with the row staying. And what a real mail server does with a replayed move, and with a message another client changed meanwhile, which the loopback servers cannot say: #63's move, copy and delete proofs against the tester's account are re-taken after this plan, and ledger 187's three questions about a crossing stay the tester's. Since 11-07.2 (later on 2026-09-19) the crossing completes here first too, so #63's copy and move across accounts are re-taken against that shape, and 547 names what its ear settles | open |  | 2026-09-19T06:07:03.000Z |  |
| 547 | 11 | unrun-verify | src/application/moves_waiting.rs |  | 11-07.2: what only the tester's ear and two real servers settle for #86's second half. A move to a folder of the other account leaving the row at once under NVDA with the cursor read on the next message and "Moved to Work in Home" shown; the message appearing in that folder of the other account at its next check; with the network off, the row going and coming back with the refusal spoken once when a server answers no; a restart with a crossing waiting finishing it at the next check of either account from the kept message, without a question; a message over 25 MB saying it goes now and its row leaving when the other account has taken it; a copy across accounts heard once. And what two real servers do: the upload's answer at a real destination, what a destination does with a message carrying an identifier it already holds, which the replay reads as an arrival only for a new number, and Gmail's treatment of an appended message; the loopback servers prove the four answers at each of the two servers and the restart from held bytes, and nothing here has met a real account | open |  | 2026-09-19T08:20:00.000Z |  |
| 548 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-08: what only the tester's ear and account settle for #31. The sender of the message a thread row stands for heard first when the row is read in his inbox, the originator on a thread nothing in it read and the first unread message otherwise; the preview under the row heard to be that message, and Enter opening the conversation window with the cursor on it so Enter again opens it; Space reading it; M marking the whole thread; a thread with everything read reading the oldest; and the text of a conversation arriving from Gmail on landing on its row, which the readings hold to one bounded chunk under the reading gate and no server has been asked for. The cache cases hold the rule through the real listing and the readings hold the window; none of it has been heard | open |  | 2026-09-19T10:35:00.000Z |  |
| 549 | 11 | unrun-verify | src/application/server_thread_ids.rs |  | 11-08.1: what only the tester's Gmail account settles for #88. His split threads showing as one conversation row after the next check, once the once-only pass has given the stored mail its X-GM-THRID, with the count on the row matching what Gmail's own client shows for that thread; a conversation opened with Enter holding the same messages Gmail shows; a reply that arrived without the headers that would have joined it sitting in its thread; two unrelated threads with one subject staying two rows; and the pass itself against Gmail, whether one UID FETCH of the stored numbers for the one field per kept folder is answered whole over his 17,753 messages and what it takes, which the scripted server puts at 54 bytes a message and no real server has been asked. The trace, the readings and the pass are held against the scripted servers; nothing here has met his account | open |  | 2026-09-19T13:00:00.000Z |  |
| 550 | 11 | unrun-verify | src/presentation/wx_app.rs |  | 11-09: what only the tester's ear settles for #26. The row heard whole and once on Ctrl+Shift+; and on the Action item, each heading then its text in the order the columns are shown, a conversation row reading its own cells, and the refusal heard when the list is not focused; arrowing through the list quiet under an NVDA configuration profile triggered by this application with Row/column headers set to Rows or Off, which nobody has set up; and what Narrator and JAWS need, whose settings the page names without steps since nobody here has read them. The composition, the item, the arm and the channel are held by cases and readings; none of it has been heard | open |  | 2026-09-19T14:20:00.000Z |  |
| 551 | 11 | todo | docs/KEYBOARD_SHORTCUTS.md |  | 11-09: an NVDA add-on that quiets the message list's column headers for this program without a configuration profile, if the profile the page describes proves too much to ask of a person setting up. It is a second piece of software installed into NVDA, with its own packaging, versioning and testing, and it serves NVDA alone; nothing under nvda-tests/ is one, that directory drives NVDA rather than extending it. Later work, on the tester's word after the profile has been tried | open |  | 2026-09-19T14:20:00.000Z |  |
| 552 | 11 | unrun-verify | src/presentation/accessibility/feedback.rs |  | 11-09.1: what only the tester's ear settles for #77. Landing on a message with an attachment heard as the row once, the Attachment column's "Has attachment" in NVDA's reading of the row, with the attachment tone beside it and nothing spoken for the event; and with "Show events in the status bar" off, the tone alone. A fresh profile hearing every event's tone from the start. The default channels, the fallback that never speaks for the event, the tester's stored profile reaching the same default, a profile that chose silence keeping it, the Feedback tab's boxes for the event and the cursor handler adding nothing spoken are held by cases and readings; none of it has been heard | open |  | 2026-09-19T15:20:00.000Z |  |
| 553 | 11 | unrun-verify | src/presentation/accessibility/feedback.rs |  | 11-09.1: the reproduction of #81 on purpose, and the sounds heard again after a real device change, which need a hand on the machine's Sound settings under a running build. The steps: the program built from the branch and started with WIXEN_MAIL_DATA set to an empty folder (src/common/paths.rs, the one override the paths module honours), so it touches nothing under the wixen-mail folder in LOCALAPPDATA; earcons on; the default output device changed in Windows Sound settings; an event with a sound triggered, Settings saved is the nearest; whether it went silent and what the log said at debug; then a headset unplugged with a sound due, and whether the sound after it plays. Never the installed binary, and never without the override. What the cases prove instead is the seam the crates document: the flag the stream's error callback sets, the reopen before the next sound, the reopen after ten seconds' quiet, the outage told once and the resume | open |  | 2026-09-19T15:50:00.000Z |  |
| 554 | 11 | unrun-verify | src/application/snippet.rs |  | 11-09.2: what only the tester's ear settles for #82. A row whose message opens with an address heard as the message's first sentence and not the address spelled out; a newsletter's row heard as its first real line and not "View this email in your browser"; a reply's row heard as the new words and not the quote; and after the next start, the rows of messages downloaded before this build heard the new way, the log's line saying how many were put right and in how long on his 17,753. The rules are held one by one in application::snippet's cases and the two places they are reached from in tests/a_snippet_is_the_first_relevant_words.rs; none of it has been heard | open |  | 2026-09-19T17:45:00.000Z |  |
| 555 | 11 | todo | src/application/long_text.rs |  | 11-09.2: the reader gives a layout table's cell as one run with no space between the blocks in it. Read on 2026-09-19 through pieces_of_markup over the Substack message saved as the public fixture for #90: the whole message arrives as one Table piece whose one cell reads "Forwarded this email? Subscribe here for moreTop three ways ... seven daysActions speak louder than wordsGary MarcusSep 19image with no description", every block's last word run into the next block's first. The snippet rules then skip the whole cell as boilerplate, since it holds the forwarding line, and the row says the subtitle the hidden preheader carried, which is a fair hint by luck rather than by rule. The reader's business, not the snippet's: a cell's blocks want a space or a line between them, in the block walk in long_text.rs, with a case over a cell holding two paragraphs; 18 records name the file | open |  | 2026-09-19T17:45:00.000Z |  |

````json
[
  {
    "id": 1,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src/presentation/folder_tree.rs",
    "line": null,
    "description": "The Favourites group is not confirmed with a screen reader; FOLDER-03's last criterion is satisfied structurally only",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T19:15:54.071Z",
    "resolved_at": null
  },
  {
    "id": 2,
    "kind": "deviation",
    "phase": "01",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The running tree builds one account at a time, so D-29's per-account Favourites branches are tested but not visible in the program yet",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-30T19:15:54.459Z",
    "resolved_at": "2026-09-02T07:10:31.448Z"
  },
  {
    "id": 3,
    "kind": "stub",
    "phase": "01",
    "file": "src/presentation/message_rows.rs",
    "line": null,
    "description": "conversation_cell_text is written and tested per column and has no non-test caller; 01-12 draws the collapsed conversation list",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:14.164Z",
    "resolved_at": "2026-09-02T07:09:47.226Z"
  },
  {
    "id": 4,
    "kind": "stub",
    "phase": "01",
    "file": "src/presentation/message_columns.rs",
    "line": null,
    "description": "Sort::conversation_order_by_clause is written and tested and has no non-test caller; 01-12 passes the user's chosen sort",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:21.927Z",
    "resolved_at": "2026-09-02T07:09:55.366Z"
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "01",
    "file": "src/application/conversations.rs",
    "line": null,
    "description": "Hungarian's one-letter I: forward marker is read as a reply marker, because mail_parser's trim_trailing_fwd ignores a parenthesised word of one character",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:22.334Z",
    "resolved_at": "2026-09-02T18:18:48.506Z"
  },
  {
    "id": 6,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Rethreading on arrival repaints one row and does not touch the selection; no screen reader has confirmed that a repainted row is silent to somebody not on it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:47.847Z",
    "resolved_at": null
  },
  {
    "id": 7,
    "kind": "stub",
    "phase": "01",
    "file": "src/application/thread_identity.rs",
    "line": null,
    "description": "A conversation root arriving after a message that names it is not merged: the link lives only in the other message's stored refs_header, which no index can search. Needs an identifier-to-conversation table",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:49.247Z",
    "resolved_at": "2026-09-04T18:30:00.000Z"
  },
  {
    "id": 8,
    "kind": "deviation",
    "phase": "01",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:50.314Z",
    "resolved_at": "2026-09-03T09:36:22.242Z"
  },
  {
    "id": 9,
    "kind": "deviation",
    "phase": "01",
    "file": ".planning/phases/01-folders-and-conversations/01-13-PLAN.md",
    "line": null,
    "description": "Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:51.507Z",
    "resolved_at": "2026-09-04T18:30:00.000Z"
  },
  {
    "id": 10,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The coverage sentence before a saved search is announced as a low-priority status topic and has not been heard under a screen reader; it also coalesces with the Running this saved search line, which is by design and unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T02:14:56.334Z",
    "resolved_at": null
  },
  {
    "id": 11,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/mail_sync.rs",
    "line": null,
    "description": "The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names. Corrected on 2026-09-17 by 10-05: the fetch is the text pass of the download of everything now, fifty messages or 16 MiB a chunk with three refusals in a row ending the chunk and a wait of thirty seconds doubling to thirty minutes before the run is tried again, started by every check for mail rather than by a command, so the tester's Gmail account meets it unasked on the first check after the build; the question is unchanged and still open",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:05.356Z",
    "resolved_at": null
  },
  {
    "id": 12,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The offer button and its experimental sentence have not been heard under a screen reader: whether the button is announced with its full label after a saved search, and whether the message text topic is heard rather than coalesced away, is unverified by ear",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:05.812Z",
    "resolved_at": "2026-09-18T02:49:01.000Z"
  },
  {
    "id": 13,
    "kind": "stub",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The offer only appears while a saved search that reads message text is run; a person who never uses saved searches is never offered the fetch, which is where D-2-08 puts it and is a narrower reach than a menu command would have",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:12.023Z",
    "resolved_at": "2026-09-01T17:15:20.187Z"
  },
  {
    "id": 14,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The filter rule dialog's three Choice controls have an accessible object attached, which a test can see, but the name each one carries cannot be read back from wxdragon. Whether NVDA says 'Match field', 'Match type' and 'Action' rather than an unnamed combo box is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:49.641Z",
    "resolved_at": null
  },
  {
    "id": 15,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The Pattern box is disabled for the four ways of matching that read no pattern. Whether a disabled edit box is skipped cleanly in the tab order, and whether changing the Match Type while focus is nearby moves focus or is announced, is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:57.259Z",
    "resolved_at": null
  },
  {
    "id": 16,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/filters.rs",
    "line": null,
    "description": "The eleven field names and eleven ways of matching are now read aloud as words. Whether 'Read is yes', 'Flagged is yes' and 'matches a text pattern' are understood when heard rather than seen is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:57.717Z",
    "resolved_at": null
  },
  {
    "id": 17,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/saved_searches.rs",
    "line": null,
    "description": "The Save This Search window now reads out one clause per question, so a three-question search says a longer sentence than the one fixed sentence it replaced. Whether that is clearer or merely longer when heard is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T06:59:55.094Z",
    "resolved_at": null
  },
  {
    "id": 18,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Saving a search whose folder belongs to another account is refused out loud through refuse_a_command. Reaching that state needs two accounts and Set Active, so whether the refusal is heard and understood is unverified",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T07:00:02.733Z",
    "resolved_at": null
  },
  {
    "id": 19,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A saved search narrowed to a folder has never been run against a real account. Whether the stored path resolves through get_folder for a real IMAP mailbox, rather than refusing with THAT_FOLDER_IS_NOT_HERE, is unverified against a live server",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T07:00:03.177Z",
    "resolved_at": null
  },
  {
    "id": 20,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The Add/Edit Condition dialog says what a saved search cannot find with the chosen field, on a line of text under the controls and through the announcement queue. Whether it is heard when the field list changes, and whether a sentence that long is useful there rather than in the way, is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T08:59:42.387Z",
    "resolved_at": null
  },
  {
    "id": 21,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The Add/Edit Condition dialog's two lists carry accessible names set by this code, and wxdragon's Accessible has no name getter, so a test can only prove an object was attached. Whether NVDA says Match field and Match type rather than unnamed combo boxes is unverified",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T08:59:51.835Z",
    "resolved_at": null
  },
  {
    "id": 22,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The Add/Edit Condition dialog refuses an empty pattern through a message box and puts focus back on the Pattern box. Whether the refusal is heard and whether focus lands where somebody expects is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T08:59:52.282Z",
    "resolved_at": null
  },
  {
    "id": 23,
    "kind": "stub",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "build_rule_edit_dialog and show_rule_edit are built and tested and nothing in the running program opens them. Plan 02-07 wires the rule editor that does",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-01T08:59:52.753Z",
    "resolved_at": "2026-09-01T10:38:15.106Z"
  },
  {
    "id": 24,
    "kind": "stub",
    "phase": "02",
    "file": "src/data/message_cache/saved_searches.rs",
    "line": null,
    "description": "replace_saved_search is written and tested and has no caller outside its tests. Plan 02-07's rule editor is what calls it",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-01T08:59:53.265Z",
    "resolved_at": "2026-09-01T10:38:15.561Z"
  },
  {
    "id": 25,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The condition manager has never been opened in a running build. The path to it is traced and every part is tested, but nothing has run the modal loop: no window has been shown, no Add pressed, no Close refused",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T10:38:35.678Z",
    "resolved_at": null
  },
  {
    "id": 26,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/manager_words.rs",
    "line": null,
    "description": "Whether a tally on the end of every condition change reads well by ear, or is a clause somebody stops hearing. Only a condition list counts out loud, and whether that is the right set is a judgement a screen reader settles",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T10:38:36.133Z",
    "resolved_at": null
  },
  {
    "id": 27,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/context_menu.rs",
    "line": null,
    "description": "Whether the saved-search context menu reads correctly with a screen reader, and whether Edit conditions first is the right order by ear rather than Run this search again",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T10:38:36.557Z",
    "resolved_at": null
  },
  {
    "id": 28,
    "kind": "deviation",
    "phase": "02",
    "file": "tests/manager_dialog_labels.rs",
    "line": null,
    "description": "wxdragon 0.9.17's ListCtrl::get_item_text loses the last character of every cell and returns a NUL in its place, so the window check reads a cell through a helper that allows for it. Upstream defect, not reported yet",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T10:38:36.983Z",
    "resolved_at": null
  },
  {
    "id": 29,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/folder_tree.rs",
    "line": null,
    "description": "The saved-search account branches have never been drawn in a running build. Whether a search now three levels deep reads well by ear, and whether the branch and the account's own branch are distinguishable when both say the account's name, is unverified",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T12:51:08.946Z",
    "resolved_at": null
  },
  {
    "id": 30,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Landing on a saved search now sets the working account. Whether that is heard, and whether somebody notices they have moved accounts by arrowing onto a search, is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T12:51:09.391Z",
    "resolved_at": null
  },
  {
    "id": 31,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The refusal for a saved search whose account has gone needs two accounts and one of them removed while a row for its search is still on screen. Never reached in a running build and unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T12:51:09.821Z",
    "resolved_at": null
  },
  {
    "id": 32,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A saved search has never been run against a real account under two accounts. That opening one under account B while account A is current returns B's mail is proved by tests over the decision and by the cache read that narrows on the account, not by a live run",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T12:51:10.257Z",
    "resolved_at": null
  },
  {
    "id": 33,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The search box's coverage sentence has never been heard. It is appended to the match count on the low-priority status topic, so it is now said on every search that reads message text, including when the whole mailbox is covered and the sentence says nothing new. Whether that is useful or is flooding on every search is a judgement only a screen reader run can make",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T14:30:53.917Z",
    "resolved_at": null
  },
  {
    "id": 34,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "A search box search that finds nothing now signals NothingFound on its own topic at normal priority and sends the coverage sentence on the status topic at low priority. That both are heard, and in an order that makes sense, is reasoned from the queue keeping only the newest of a topic and is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T14:31:02.712Z",
    "resolved_at": null
  },
  {
    "id": 35,
    "kind": "deviation",
    "phase": "02",
    "file": "src/data/message_cache/mod.rs",
    "line": null,
    "description": "The box's coverage count is short for a database that already had a search index and had evicted bodies before this column existed. The index is contentless so it cannot be asked what it holds, and fts5vocab can but takes about nine seconds at two hundred thousand messages, so those rows are backfilled from message_bodies. The backfill asks whether the stored body holds text, which is the question the live writer asks; asking only whether a row was there counted a message with no text part as text the box can read, and that is fixed. Evicted messages stay findable by their text and are counted as though they are not. Short rather than over for them, and the set never grows. Two narrower ways it can still be over, both invisible to SQL and corrected the next time that message is indexed: a packed half that no longer unpacks, and markup that is one unterminated tag",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T14:31:03.243Z",
    "resolved_at": null
  },
  {
    "id": 36,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The File menu item for the fetch has never been drawn in a running build: whether NVDA reads the experimental marking on its label and in the item description, and whether the offer's spoken line and the coverage sentence are heard as two answers rather than one contradiction, are both unheard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T17:15:26.277Z",
    "resolved_at": null
  },
  {
    "id": 37,
    "kind": "todo",
    "phase": "02.1",
    "file": "src/presentation/wx_app.rs",
    "line": 10262,
    "description": "Two comments made false by 02.1-01 are still there and were found a second time by 02.1-02. Line 10262 says the ten checks in tests/wired.rs cannot use what_ships because it is cfg(test); it is behind a cargo feature now and they do. Line 19737 says the_window_itself reads this file and stops at the first cfg(test); it uses what_ships. Both instruct the next person to follow a convention for a reason that no longer holds",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T11:49:49.886Z",
    "resolved_at": null
  },
  {
    "id": 38,
    "kind": "deviation",
    "phase": "02.1",
    "file": "docs/roadmap.md",
    "line": 156,
    "description": "Folder favorites is unticked on the shipped roadmap and ships: ID_PIN_FOLDER draws a Pin Folder menu item, application::favourites backs it, and 02-08 used the Favourites branch as the precedent saved searches copied. Found by 02.1-03's tree search, left unfixed as outside criterion 5 and belonging to phase 2",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T12:41:25.488Z",
    "resolved_at": null
  },
  {
    "id": 39,
    "kind": "deviation",
    "phase": "02.1",
    "file": "scripts/check.sh",
    "line": null,
    "description": "The red half of red/green cannot be committed for a shell suite. check.sh runs every scripts/*.test.sh under set -e before it branches on the mode, so a failing suite aborts the gate before the red branch is reached and red-commit.sh verdict is never consulted. Measured by hand on 2026-09-02 by breaking one case in scripts/check.test.sh and committing with a Fails-until-green trailer naming it. Separately, verdict reads cargo's 'test NAME ... FAILED' lines, which a shell suite never produces, so a named shell case would report as never having run",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-02T13:30:26.438Z",
    "resolved_at": "2026-09-03T08:58:32.016Z"
  },
  {
    "id": 40,
    "kind": "deviation",
    "phase": "02.1",
    "file": "tests/house_style.rs",
    "line": 5499,
    "description": "runs_the_suite exempts any line containing '--test ' as one that runs a named target on purpose, so a line naming fifteen targets without --no-fail-fast is exempt too. That hid a real defect in check.sh: 'cargo test --test house_style --test wired' ran two targets and stopped at the first failure. Found on 2026-09-02 only because building those targets into an array took the literal flag out of the text and the guard then spoke. The line is fixed; the exemption is still wider than one named target",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-02T13:30:36.985Z",
    "resolved_at": "2026-09-03T08:58:24.727Z"
  },
  {
    "id": 41,
    "kind": "deviation",
    "phase": "02.1",
    "file": "src/presentation/folder_tree.rs",
    "line": null,
    "description": "wxdragon 0.9.17 never removes a tree item's custom data from its process-global registry. cleanup_all_custom_data walks the tree through clean_item_and_children, which calls remove_item_data nowhere at all, for a leaf or for a branch, and the same walk is what runs automatically when the control is destroyed. delete_all_items goes straight to the FFI and removes nothing either. So set_custom_data and append_item_with_data leak one entry per row for the life of the process, and the only escape is not to call them. 02.1-05 took both dialogs off them; the folder tree in wx_app.rs was already off them and is held there by a source read. Upstream defect, not reported yet",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T15:06:11.672Z",
    "resolved_at": null
  },
  {
    "id": 42,
    "kind": "unrun-verify",
    "phase": "02.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "ask_about_the_folders_that_have_gone has never been opened in a running build. The four things its body decides are read from source by tests/wired.rs; a live window was available and not used, because every path that tells a right argument from a wrong one ends at MessageDialog::show_modal, which blocks with nobody to answer it, so a wrong argument would hang the commit gate rather than fail it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T17:57:52.087Z",
    "resolved_at": null
  },
  {
    "id": 43,
    "kind": "deviation",
    "phase": "02.1",
    "file": ".planning/phases/02.1-what-phase-1-found-on-its-way-past/02.1-07-PLAN.md",
    "line": null,
    "description": "The claim that a test cannot build a live window came back in a planning document. 02.1-02 corrected it in five source comments and left test_no_comment_says_a_test_cannot_build_a_window behind to stop it returning, but that guard reads Rust files only, so 02.1-07's plan could assert the budget was spent and nothing spoke. 02.1-05 had already disproved the same claim from its own plan. The guard cannot be widened to .planning without reading plans that are allowed to be wrong before they are executed, so this is recorded rather than fixed",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T18:32:59.629Z",
    "resolved_at": null
  },
  {
    "id": 44,
    "kind": "unrun-verify",
    "phase": "02.1",
    "file": "src/application/context_menu.rs",
    "line": null,
    "description": "The six context menus the folder tree now offers have not been heard. Nothing confirms that an account branch's five entries and their mnemonics are announced, nor that the menu key doing nothing on All Inboxes, Favourites, On this computer and the saved searches heading reads as nothing to do here rather than as a key that failed. That last one is the risk this design takes on purpose: silence teaches as little as an item that does nothing, and only a real NVDA or Narrator run says which is worse",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T19:59:55.206Z",
    "resolved_at": null
  },
  {
    "id": 45,
    "kind": "unrun-verify",
    "phase": "02.1",
    "file": "src/presentation/folder_tree.rs",
    "line": null,
    "description": "Account branches stopped reading their email address unless two accounts share a name. Nothing confirms by ear that the shorter label is an improvement, nor that the address appearing on two branches and not on a third is understood as a disambiguator rather than as an inconsistency",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T20:00:02.105Z",
    "resolved_at": null
  },
  {
    "id": 46,
    "kind": "deviation",
    "phase": "02.1",
    "file": ".planning/phases/02.1-what-phase-1-found-on-its-way-past/02.1-08-PLAN.md",
    "line": null,
    "description": "The plan's premise correction stated that where_a_row_sits is production code with no production caller, measured that day, and prescribed wire it or remove it. It has one: wx_app::the_row_on_screen calls it once per row and which_row calls that on every folder tree selection, so it is on the main control's selection path. The premise was scoped to the defining file and to tests/ and never to sibling source files, and acting on it would have deleted live code. Recorded because the shape recurs: a negative reachability claim reads as a survey while naming only where somebody looked",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T20:00:11.219Z",
    "resolved_at": null
  },
  {
    "id": 47,
    "kind": "deviation",
    "phase": "02.1",
    "file": "src/application/context_menu.rs",
    "line": null,
    "description": "D-2.1-03 says each branch kind gets its own menu and a group heading offers what is true of the group. Four rows got no menu instead: All Inboxes, Favourites, On this computer and the saved searches heading. Nothing this program does acts on one of them, and every candidate command reads whichever account is open, which on a row naming no account is whichever account somebody came from. The decision's own reason for rejecting no menu was losing genuinely useful per-account commands, and none is lost, because every row that names an account keeps its own. Recorded as a divergence from a recorded decision rather than as a fault",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T20:00:23.296Z",
    "resolved_at": null
  },
  {
    "id": 48,
    "kind": "deviation",
    "phase": "02.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Criterion 12 was planned against two accounts of one name drawing rows that read identically. They did not: the_accounts_in_the_tree filled each name from Account::display_name, which is name and address together, and the accounts table declares email NOT NULL UNIQUE. The property was real, held by two layers that folder_tree.rs never mentions, and unowned there. The plan's own remedy would have added a second defence to a case that could not arise. What the trace found instead is the opposite defect, and it was fixed: the address was read aloud on every account branch, always, to serve a case that had never happened",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T20:00:23.940Z",
    "resolved_at": null
  },
  {
    "id": 49,
    "kind": "unrun-verify",
    "phase": "02.1",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The box a condition editor now shows instead of opening on a rule it cannot read has not been heard. It goes through a_sub_dialog_needs, which builds a MessageDialog a screen reader reads on its own, captioned \"Not opened\" before the open and \"Not saved\" before the write, and the sentence under it runs to two paragraphs. Whether the caption and the sentence read as one thing rather than two, and where the sentence breaks for speech, is unverified. Nothing in the library can hear it: every path from show_rule_edit or show_filter_edit to a real box ends at show_modal, which blocks with nobody to answer it, so a test that opened one would hang the commit gate rather than fail it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-02T22:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 50,
    "kind": "deviation",
    "phase": "03",
    "file": "src/service/signed_mail.rs",
    "line": null,
    "description": "Two certificate tests fail on GitHub's Windows runners and pass on a real machine: one of the runner's root authorities is genuinely reported withdrawn by Windows, and its three authorities produce no per-certificate answer because nothing local holds a withdrawal list. Checked 2026-09-03 and deferred by Pratik on the ground that it does not change how the application behaves: what_windows_found maps only CERT_TRUST_IS_REVOKED and CRYPT_E_REVOKED to Withdrawn, while offline, no list held, and no revocation information each map to CouldNotFindOut with a reason, so the code never reads could-not-check as revoked. CI stays red on these two until a runner with a representative certificate store exists, or the tests take their certificates as an argument.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-03T20:58:51.850Z",
    "resolved_at": null
  },
  {
    "id": 51,
    "kind": "deviation",
    "phase": "03",
    "file": "src/service/spellcheck/windows_speller.rs",
    "line": 166,
    "description": "supported_languages returns an empty list both when this machine has no spell checkers and when the COM call failed, with nothing logged: CoCreateInstance's error is discarded by a let-else that returns the empty vec. available_languages then reports no languages, best_available_match answers None, and default_language at data/config.rs:466 falls back to en, so a transient COM failure at first run sets a French user's fresh install to English and marks every word of their mail wrong. Found 2026-09-03 while investigating the one-in-five test flake the phase 1 deferred list records; the flake is this defect seen through a test that asks the same question twice. The codebase already has the right shape for the fix in Withdrawal, which distinguishes NotWithdrawn from CouldNotFindOut with a reason. Not yet routed to a phase.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-03T20:59:03.743Z",
    "resolved_at": "2026-09-03T22:56:29.517Z"
  },
  {
    "id": 52,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Criterion 1's announcement half is structure only. The renumbering sentence is built in mail_sync::what_the_renumbering_discarded, sent as UIUpdate::FolderWasRenumbered, and announced by handle_update on its own topic \"renumbered\" at Priority::Normal, and a source-reading test holds all three. No screen reader has heard it. Three things only an NVDA or Narrator run settles: whether the sentence is spoken at all when a folder is renumbered mid-sync; whether a topic of its own is the right choice against \"status\", since the reason for splitting it off is that the queue coalesces same-topic announcements and the next \"Checking Sent...\" would replace it, which is reasoning about the queue rather than an observation of it; and whether a Normal-priority announcement arriving in the middle of a sync cuts across something the person was reading, which is guardrail 5's bounded-and-distinct question and cannot be answered by reading source. Compounded by the fact that no real server has ever renumbered a folder for this program, because it has never been used with an account, so the whole path has only run against a scripted server.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-03T22:40:54.889Z",
    "resolved_at": null
  },
  {
    "id": 53,
    "kind": "deviation",
    "phase": "03",
    "file": "Cargo.toml",
    "line": null,
    "description": "wxdragon is pinned at =0.9.17 and 0.9.21 is out. Checked 2026-09-03 while reporting the two defects this project had recorded as unreported. Ledger 28, ListCtrl::get_item_text losing the last character of every cell, was already reported by somebody else as AllenDang/wxDragon#205 against 0.9.19 and is fixed on master: the fix allocates needed_len + 1 and its comment names that issue and the same mechanism this project diagnosed. So 28 wants an upgrade rather than a report, and the workaround helper in tests/manager_dialog_labels.rs comes out when the upgrade lands. Ledger 41, TreeCtrl::cleanup_all_custom_data walking the tree and removing nothing, is still present on master and is now reported as AllenDang/wxDragon#214 with a suggested fix. Upgrading four minor versions of the UI framework is its own piece of work and is not phase 3's.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T04:36:43.729Z",
    "resolved_at": null
  },
  {
    "id": 54,
    "kind": "deviation",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A source-reading check reports findings as {path}:{at + 1} where the index comes from what_ships(text).lines().enumerate(), over every Rust file under src. That is the file's own line number only while nothing was cut above the finding: for any file with a #[cfg(test)] item above a send_status line the reported position is short by however many lines were deleted, silently and with a well-formed message pointing at the wrong line. Correct today for the files it reports on, which is why it reads as blessed practice and is the precedent a new source-reading check would copy. Found 2026-09-04 while writing tests/one_sign_in_per_piece_of_work.rs, which carries line numbers through the cut instead. Out of 03-02's scope.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T05:46:15.242Z",
    "resolved_at": null
  },
  {
    "id": 55,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/mod.rs",
    "line": null,
    "description": "Plan 03-03's must_have truth 'a marker that is wrong in the dangerous direction cannot lose a body, because a cheap probe that the marker never skips is what re-checks it' is unmet as written, and met more strongly in substance. No marker was built. Following the plan's own ordering through, a marker that never gates the question decides nothing: the probe answers in both branches and the marker is written and never read. What shipped is a partial index (idx_messages_inline_body) over exactly migrate_inline_bodies's condition, which makes the question free rather than making a wrong answer harmless, so there is no state that can be wrong at all. The reason not to add a marker later is a comment in mod.rs beside the index and in bodies.rs on THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE, and a guard record whose break is the marker the next person would reach for. Recorded so an audit comparing the plan's truths against the summary is not left guessing.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T10:00:24.498Z",
    "resolved_at": null
  },
  {
    "id": 56,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "Nothing in plan 03-04 has run against a real Gmail account, because this program has never been used with an account at all. The archived-with-no-label fix is proved against a mail cache built inside a test: real evidence about the SQL, no evidence about what Gmail sends. Specifically unverified: that a message archived without a label really appears in All Mail and nowhere else on a live account; that X-GM-MSGID really comes back on the same message under a label and in All Mail; that holds_all_mail is really set for Gmail's All Mail by a live LIST response. Closes only against a real account.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T13:45:10.738Z",
    "resolved_at": null
  },
  {
    "id": 57,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "A message is still counted twice in a conversation if a server holds it in two places and gives it neither a Gmail identifier nor a Message-ID. WHICH_MESSAGE_THIS_ROW_IS falls back to the row id, so two such rows are two messages. Chosen deliberately over merging by row position: a count that is too high is visible, a conversation that has vanished is not. Also unfixed and pre-existing: a Gmail message under two labels counts twice, because both label rows are real rows outside All Mail and nothing says which label should lose. Fixing that needs the count and the delete list to become different questions, which is an architectural change rather than a predicate.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T13:45:22.582Z",
    "resolved_at": null
  },
  {
    "id": 58,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "Measured cost of the identity filter, release build, warm, 200,000 rows in 10,000 conversations. On an account with a folder holding all mail the conversation listing goes from about 0.75s to about 1.2s, roughly 60 percent more, of which about 300ms is the filter and about 150ms the extra rows now in reach. On an account with no such folder there is no measurable difference, 0.86s against 0.85s, so the short-circuit claim in conversation_scope's doc comment was measured rather than assumed. Neither number is acceptable on its own terms: conversations_query has no LIMIT and groups the whole account on every listing, which is SCALE-03's subject and was true before this change.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T13:45:35.198Z",
    "resolved_at": null
  },
  {
    "id": 59,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/searching.rs",
    "line": null,
    "description": "searching.rs:539 groups search results by COALESCE(m.gmail_msgid, m.id), which is the identity plan 03-04 found insufficient for the conversation count. On a server that advertises the RFC 6154 All attribute and gives no Gmail identifier, a search shows the same message twice, once per copy. Same class of defect, same remedy available (the Message-ID arm of WHICH_MESSAGE_THIS_ROW_IS), pre-existing and outside 03-04's scope. test_one_gmail_message_under_two_labels_is_found_once covers the Gmail case only.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T13:45:47.023Z",
    "resolved_at": null
  },
  {
    "id": 60,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "Nothing in this plan has run against a real account. That a real client sends In-Reply-To without References, that a conversation root really does arrive after a message naming it during a live sync, and that the first open after this change is bearable on somebody's real mailbox are all unverified: the merge, the backfill and every timing here are measured against a cache built inside a test on this computer.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T18:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 61,
    "kind": "deviation",
    "phase": "03",
    "file": "src/application/thread_identity.rs",
    "line": null,
    "description": "A merged conversation can settle under an identifier that is nobody's root. Two conversations an arrival has proved to be one carry no ordering between their names, so the winner is the least of them by ordinary string comparison, which is stable and arbitrary. Stability is what was needed and finding the older message is not available to rejoin. Recorded rather than glossed, because for a chain naming only its parent the conversation is then filed under a message in the middle.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T18:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 62,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "A merge renames one of the two conversations and nothing in the running program says so. The changelog says a conversation may change which message it is filed under; the interface does not, and whether somebody reading a conversation notices it move under a screen reader is unverified by ear.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T18:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 63,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/mod.rs",
    "line": null,
    "description": "The first open after this change walks every stored message and reports nothing while it does. Measured at 5.66 seconds with nothing to join and 6.45 with every conversation split in two, over two hundred thousand messages on this computer. It happens once, it is gated on a probe rather than a marker, and a larger mailbox pays more with the window showing nothing.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T18:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 64,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/mail_controller.rs",
    "line": null,
    "description": "Whether a real provider accepts a fresh sign-in straight after it has dropped a connection, or treats it as something to slow down or refuse, is unknown. The single retry is proved against a loopback server that hangs up on command and answers the next connection immediately. No account has ever been used with this program, so nothing here has met a provider's real behaviour on reconnect, including whether it counts against a connection limit. Corrected on 2026-09-18 by 10-06: the inbox watch is started again after a wait of thirty seconds doubling to thirty minutes whenever it ends for any reason but mail arriving, and at once when the network comes back, so the tester's Gmail account meets a fresh sign-in after a dropped connection unasked, on the first drop after the build; what the provider does with it is what this entry asks and is still unobserved.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:03:06.059Z",
    "resolved_at": null
  },
  {
    "id": 65,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/mail_session.rs",
    "line": null,
    "description": "What a real provider does with a session held open and idle for minutes is unknown, and the whole point of holding one is that it sits idle between commands. Whether providers drop an idle IMAP session at all, how soon, and whether they say anything before they do, has never been observed by this program: no account has ever been used with it. The reconnect exists because a drop is expected, and that expectation is reasoning rather than a measurement. Corrected on 2026-09-18 by 10-06: the watch connection, one IDLE session per enabled IMAP account renewed every twenty-nine minutes, is now held open for as long as the program runs and started again after a growing wait when it ends, so whether a provider drops an idle session, how soon, and whether it says anything first is what the tester's account shows on the first day of the build; a drop that says nothing can go unnoticed for up to twenty-nine minutes, and the check on the account's interval is what covers that.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:03:24.528Z",
    "resolved_at": null
  },
  {
    "id": 66,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the refusal after a failed retry is heard once rather than once per failed request is unverified by ear. It reaches somebody through each site's existing reporting, which is ErrorOccurred for the flag path and CommandRefused for the folder commands, and both announce at High priority through accessibility::announce. That is structure, not experience: nobody has heard it with NVDA, and a mailbox where every command meets a dead connection would produce one of these per command with nothing coalescing them, which is exactly the flooding guardrail 5 is about.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:03:34.593Z",
    "resolved_at": null
  },
  {
    "id": 67,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/mail_session.rs",
    "line": null,
    "description": "The connection budget of two per account is counted against a loopback server and has never been counted against a provider. Whether two per account is welcome, what a provider counts as a connection when several accounts sit on the same one, and whether the IDLE connection and the working session are counted together, are all unknown. Gmail's limit of fifteen per account is the number the requirement's evidence records rather than one this program has ever approached.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:03:43.871Z",
    "resolved_at": null
  },
  {
    "id": 68,
    "kind": "deviation",
    "phase": "03",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "folder_counts has the same shape select_folder was fixed for and is not fixed. It calls async-imap's session.status, whose parser reads responses until the stream ends and hands back what it collected, so a connection dropping mid-command comes back as Ok with nought messages and nought unread. Corrected on review 2026-09-04: this said a wrong number rather than a deletion, and that understates it. A count of nought is what disarms listing_contradicts_the_count, which is listed == 0 && counted > 0 and is the only check between a truncated listing and an emptied folder. select_folder erroring now aborts the sync before list_uids is reached, which closes the path, so this is latent rather than live; it would become live again if anything ever reads the count without the SELECT in front of it. Same defect underneath: a command that never completed reported as one that did. Fixing it means writing STATUS as a command line through read_command, the way select_folder now is.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:03:53.498Z",
    "resolved_at": null
  },
  {
    "id": 69,
    "kind": "deviation",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Checking for mail used to refuse an unusable port with the value it could not read, 'has an IMAP port that is not a number: 14 3'. All twelve sites lost their own port check when they went through the held session, because a_session_at asks the same question and answers it in the same words, so each was a second answer to one question. Eleven lost nothing by that; this one lost the offending value, which is the part somebody fixing it needs. The value is visible in the account settings screen.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-04T21:04:03.565Z",
    "resolved_at": null
  },
  {
    "id": 70,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/finding_what_was_deleted.rs",
    "line": null,
    "description": "Whether any provider grants CONDSTORE, which is what the resume needs. imap/abilities.rs asserts that Gmail never has. Fastmail and current Dovecot advertise it in the capability lists this project models them on, and no capability list has ever been read off a real server here. If none of the providers people use grants it, SCALE-01's saving applies to nobody and every folder is read out in full on every sync, which is what happens today anyway.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:35:34.235Z",
    "resolved_at": null
  },
  {
    "id": 71,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/finding_what_was_deleted.rs",
    "line": null,
    "description": "Whether a hand-built SELECT with QRESYNC parses back at all, which is what the declared and unbuilt VANISHED member would need. async-imap 0.11.3 has no ENABLE and no select_qresync, so it goes through run_command, and the mailbox response that comes back is one async-imap's own select parses. imap-proto already parses Response::Vanished. Whether the raw select parses and whether VANISHED reaches the closure has never been run against a server, and it is the whole cost of the second implementor.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:35:52.234Z",
    "resolved_at": null
  },
  {
    "id": 72,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/bringing_everything_down.rs",
    "line": null,
    "description": "Whether a provider tolerates a whole-folder request. It asks for a folder five hundred messages at a time, without stopping, until the folder is here. Ledger 11 records the same gap for the bulk body fetch and this is the same shape at a different granularity: a provider is entitled to refuse, throttle, or disconnect, and nothing on this side can find out which. Two things follow that no test here can settle: how many chunks a provider allows before it slows down, and whether a disconnect part way is reported as the request stopping short rather than as the folder being finished. Marked experimental on the menu item and in its description. Corrected on 2026-09-17 by 10-05: the whole-folder request and its module are gone; the loop is the download of everything, which asks bringing_everything_down::what_to_do_next for every kept folder of every enabled IMAP account after every check, stops when Pause Downloading is ticked, and waits a growing time after a refusal, so a disconnect part way is reported as the folder stopping short and the run is tried again on its own; the experimental sentence is on the Pause item; the question is unchanged and still open, and the tester's Gmail account meets it unasked",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:36:00.334Z",
    "resolved_at": null
  },
  {
    "id": 73,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Three things about the whole-folder request that only a screen reader settles, on the pattern of phase 2's entries 10, 33 and 34. Whether a fetch of eighty chunks on a topic of its own is heard rather than lost: the topic keeps only its newest announcement, so the claim is that somebody hears a handful of sentences, and nothing here has listened. Whether the final count is heard as an ending rather than as another progress line; the words differ and the topic does not. And whether the choice of a topic of its own is right at all against putting it on 'status', which is the one open question in plan 03-07 and is a listening judgement: the argument for splitting is reasoning about how the queue coalesces rather than an observation of it, and the constant THE_PROGRESS_TOPIC is the one line that moves it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:36:08.578Z",
    "resolved_at": null
  },
  {
    "id": 74,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/message_rows.rs",
    "line": null,
    "description": "Whether the snippet column reads well when a screen reader crosses a column of rows that all say 'Message text not downloaded'. That is every row of a folder nobody has fetched text for, which is most of a large mailbox, and four words per row is four words more than the blank it replaced. The blank was a lie and the words are true, so this is a question about whether the true answer is worth what it costs to hear, not about whether to go back. If it is too much, the shorter answer is to say it once for the column rather than once per row, and there is nowhere on a virtual list to put that today.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:36:19.878Z",
    "resolved_at": null
  },
  {
    "id": 75,
    "kind": "deviation",
    "phase": "03",
    "file": "src/data/message_cache/bodies.rs",
    "line": null,
    "description": "The snippet column tells 'nobody fetched this text' from 'this message has no text' by whether the stored snippet is null or empty, and only rows written after this change carry the distinction. A message whose body was fetched before 2026-09-04 and held no text was stored as null, so it reads as one nobody has fetched, and the row says so until its text is fetched again. There is no backfill: the fact is not recoverable from anything the database still holds, because an evicted body leaves no row and message_bodies answers 'is the text here now' rather than 'was it ever fetched'.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T01:36:20.285Z",
    "resolved_at": null
  },
  {
    "id": 76,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/the_network_coming_and_going.rs",
    "line": null,
    "description": "Whether the sentence about the network going is heard once and understood as a state rather than as an error. The state hands out one answer per change and a test drives ten failures and counts one, which is the structure. What that cannot say is what reaches somebody: the announcement goes out on a topic of its own at normal priority while a sync is failing its way through several folders on the status topic underneath it, and whether the one that matters is the one heard needs NVDA and a cable pulled out. Whether a sentence beginning 'The network has gone' reads as information rather than as something broken is the same kind of question. Nothing here has ever met a real network loss.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T05:19:21.804Z",
    "resolved_at": null
  },
  {
    "id": 77,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the offer to go back online is announced with its full label, and whether a screen reader user learns it is there at all. The panel is shown by an arm that says nothing, on purpose: the sentence sent immediately before it names the button, and announcing again would be two announcements a moment apart about one event. That argument is about repetition and it does not settle discovery. A button appearing above the message list moves nothing and takes no focus, so what tells somebody it exists is one clause in one sentence, and whether that clause survives being heard in the middle of a mailbox is an NVDA question. Its label and its accessible name come from one string, which a test reads from the source, and whether Windows really speaks that string for this control is the MSAA and UI Automation question scripts/msaa-names.ps1 exists for and which has not been run against this window.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T05:19:37.963Z",
    "resolved_at": null
  },
  {
    "id": 78,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the status bar and the announcement being the same words reads as a repetition when somebody meets both. The network sentence is written once and handed to status field 0 and to the announcement queue, which is what stops a deaf user and a blind user being told different things. A deaf-blind user reading the status bar on a braille display and then hearing the queue speak, or a low vision user with speech on, meets the same sentence twice within a second. Whether that is reassuring or is noise is not something a test can ask.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T05:19:48.464Z",
    "resolved_at": null
  },
  {
    "id": 79,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether somebody who lets the offer go by can find their way back. Two routes exist and neither has been used by a person. The offer panel stays on screen until the mode changes some other way, so it is still there to be tabbed to, and the sentence names the View menu as the other way. What is unknown is whether either is reachable in practice for somebody who heard the sentence once, was reading a message at the time, and comes back to it twenty minutes later with nothing repeating it. There is nothing that says the offer again.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T05:19:48.844Z",
    "resolved_at": null
  },
  {
    "id": 80,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/service/network.rs",
    "line": null,
    "description": "Whether InternetGetConnectedState answers usefully on a real machine losing a real network. It reports whether this computer has a connection at all, which is what makes it right for a cable pulled out and a wifi dropped, and it says nothing about whether a mail server can be reached, so a network that is up and cannot route leaves the program believing it is online. It has never been run against a machine that lost its network while Wixen Mail was open. Two things a run would settle: how long Windows takes to change its answer after the cable goes, which is the real delay before somebody is told rather than the ten second interval this asks on, and whether a wifi that flaps produces a run of changes the queue then speaks.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T05:20:01.150Z",
    "resolved_at": null
  },
  {
    "id": 81,
    "kind": "deviation",
    "phase": "03",
    "file": "src/application/sending_later.rs",
    "line": null,
    "description": "The ten second Undo Send hold is never applied to anything. Hold, GoAfter::held and queue_outbox_message_to_go have no caller outside sending_later.rs and its tests: the composer's Send queues through queue_outbox_message, which writes GoAfter::AsSoonAsPossible, so readiness answers MayGoNow at once and take_back answers TooLate for every message somebody has just sent. The module doc says all of it runs. Undo Send is on the Tools menu and can never catch a message from the composer. Found while wiring offline mode into the same decision and left alone as out of scope: it is a behaviour change of its own with its own countdown to show and its own version bump.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-05T05:20:01.525Z",
    "resolved_at": "2026-09-06T23:12:00.392Z"
  },
  {
    "id": 82,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/presentation/wx_conflict_choice.rs",
    "line": null,
    "description": "Whether the two copies of a contact or a calendar item are understood by ear as a labelled pair. Each list is headed by a static text and named through set_accessible_name with the same string, What is on this computer and What your address book has, and the arrangement chosen is one sentence on opening saying what is being asked and how many fields differ, then each copy introduced by its label as focus reaches it. Whether that beats reading both copies out on opening is a judgement only a screen reader run settles, and nobody has heard this window. Three things a run would settle: whether the two headings are heard as headings rather than as more list content, whether the opening sentence is heard before somebody starts arrowing through the first list, and whether the list of differing fields inside that sentence is useful or is a clause people learn to skip.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T08:50:01.364Z",
    "resolved_at": null
  },
  {
    "id": 83,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/conflict_choice.rs",
    "line": null,
    "description": "Whether the count of waiting choices is useful or is a sentence somebody stops hearing. Every sync that found a disagreement ends with a whole sentence naming how many are waiting and where to make the choice, on top of the counts the sync already reads out. Phase 2 entries 26 and 33 are the precedent and asked the same question about a tally read aloud. What a run would settle: whether a sentence arriving after five counts is still heard, and whether somebody who hears it on every sync until they act finds it a reminder or a nag.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T08:50:14.084Z",
    "resolved_at": null
  },
  {
    "id": 84,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/contacts_sync.rs",
    "line": null,
    "description": "The push still sends a change typed here over the address book's newer copy, and nobody is asked. When a push is refused for carrying a version marker the address book has moved past, and the copy here was typed here, the push reads the address book's current marker and sends the change again on top of it. That is a second both-changed state, resolved in this computer's favour, at the provider, with a sentence afterwards. It is the same shape as the defect plan 03-09 fixed on the read side and it was left alone: it is guarded by two records, its behaviour is deliberate and argued for in guards.toml, and changing it means changing what those guards are about. Found while executing 03-09, whose own key link named the counter for this path as the model for how the losing case was told, which it is not.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T08:50:23.245Z",
    "resolved_at": null
  },
  {
    "id": 85,
    "kind": "deviation",
    "phase": "03",
    "file": "src/application/calendar_conflict.rs",
    "line": null,
    "description": "Two files hold tests about code that lives elsewhere, and both are a deliberate trade for guard re-measurement time. The CalDAV sync-path test lives in calendar_conflict.rs because 23 records fingerprint caldav_sync.rs's test count, and the choosing window's assertions live in tests/the_conflict_choice_can_be_heard.rs because 37 fingerprint wx_app.rs. What that costs: a test about the CalDAV sync sits one file away from the sync, and the window's own behaviour is asserted by reading source rather than by building a window. What is therefore not guarded from inside caldav_sync.rs is that the read consults calendar_conflict at all, and from inside wx_conflict_choice.rs that the dialog builds what the source says it builds. Both are covered by the new records instead, coupled through guards.toml so they run on the commits that could break them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T08:50:31.713Z",
    "resolved_at": null
  },
  {
    "id": 86,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/flag_changes_waiting.rs",
    "line": null,
    "description": "Whether the two sentences about a flag change are distinguishable by ear. One says the server could not be reached and the change is saved here; the other says the server refused it and it has been put back. They share no opening clause and no verb, and a test holds them to that, but whether somebody hearing one in the middle of a syncing mailbox knows which they heard is a judgement only a screen reader run settles. Three things a run would settle: whether the two are told apart at speed, whether announcing them on their own topic rather than the status line means they are heard at all, and whether the count in the plural form is understood as a number of changes rather than as a number of servers or messages.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T09:47:34.277Z",
    "resolved_at": null
  },
  {
    "id": 87,
    "kind": "deviation",
    "phase": "03",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A label added or removed still puts itself back when the push fails, whatever the reason. The waiting queue models two flags, read and starred, because those are the two a message row carries as its own state and the two the window puts back by sending the opposite update. A label is a keyword, of which a message can have many, and replaying one needs the keyword as well as the value. Left out rather than half-built: the arm names the case, says why, and takes the path it always took. The changelog says so under Known limitations.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T09:47:46.170Z",
    "resolved_at": null
  },
  {
    "id": 88,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "src/application/flag_changes_waiting.rs",
    "line": null,
    "description": "Whether Authentication counting as the server never having been asked is the right call against a real provider. A sign-in the server turned down means the change was never put to it and a token that has expired is fixed by signing in again, so the change is kept. Against a provider that answers Authentication for something that does not clear on its own, a wrong password nobody corrects, the change waits for ever and is offered on every sync. Nothing here has met a provider, so which errors a real one raises for an expired token against a wrong password is unknown, and that is the fact the decision rests on.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T09:47:46.593Z",
    "resolved_at": null
  },
  {
    "id": 89,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Whether the sender's description is heard at the right moment in an attachment row. It is the fourth and last clause, after the name, the kind and the size, on the reasoning that the first three are what somebody decides to open a file on and the sender's words are what they want if they are still listening. That is a judgement about what a person wants to hear first, not a measurement, and only a real NVDA or Narrator run through a message with several attachments settles it. The row is announced every time focus reaches it, so getting the order wrong costs a moment on every arrow press.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T14:09:25.832Z",
    "resolved_at": null
  },
  {
    "id": 90,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/mime.rs",
    "line": null,
    "description": "Whether a description borrowed from the alt on the img that names a part reads as the sender's own words or as something the program made up. The row says the text with nothing marking it as borrowed, on the grounds that the alt is the sender's writing about that picture as much as a Content-Description would be. Nobody has heard it. If a borrowed description reads as an assertion by Wixen Mail rather than by the sender, that is a wording problem the tests cannot see, and it matters more here than for the header because the borrow really is a guess about which element meant which part.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T14:09:33.628Z",
    "resolved_at": null
  },
  {
    "id": 91,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/mime.rs",
    "line": null,
    "description": "Whether real senders supply Content-Description at all, and how often. If they mostly do not, this feature mostly says 'no description' on the header route and leans entirely on the alt borrowed from the markup. No mail account has ever been used with this program, so it cannot be measured here, and the changelog says so rather than implying the feature does more. It decides whether the header route was worth building or whether the markup route is the whole of it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T14:09:39.886Z",
    "resolved_at": null
  },
  {
    "id": 92,
    "kind": "deviation",
    "phase": "04",
    "file": "src/application/pop_sync.rs",
    "line": null,
    "description": "A POP account records no attachment rows at all, so nothing this plan built reaches one. The plan's premise that the IMAP path and the POP path are two writers of the attachments table is wrong: the only production writer is wx_app::spawn_body_fetch, which returns early when the account has no IMAP server, and pop_sync sets has_attachments and stores nothing else. So a POP message says it carries an attachment and lists none, which predates this plan and is not made worse by it. Left alone rather than half-fixed: adding a writer to the POP sync is a new path through the cache, not a widening of this one.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T14:09:51.050Z",
    "resolved_at": null
  },
  {
    "id": 93,
    "kind": "deviation",
    "phase": "04",
    "file": "src/service/mime.rs",
    "line": null,
    "description": "The record 'a description the sender gave survives the boundary' went stale inside the session that wrote it. Written in the morning naming one test, it named too few by the afternoon, because the alt lookup added later gives a part whose header is dropped somewhere else to fall through to. Corrected by hand and re-measured. Recorded because CLAUDE.md predicts this shape and the only reason it was caught is that the second task re-ran --remeasure rather than trusting the first task's measurement; nothing would have failed if it had not.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T14:09:51.446Z",
    "resolved_at": null
  },
  {
    "id": 94,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/blocking.rs",
    "line": null,
    "description": "Nobody has heard the mailing-list warning. It is announced at Priority::High before the block is made, and two things about that are judgements rather than measurements: whether it lands before the block rather than reading as a report of one already made, and whether an email address said aloud in the middle of a sentence is understood at speed by somebody arrowing through a mailbox. The sentence is: 'This message came from a mailing list. Blocking files it into Junk and the list carries on sending it. To stop it at the source, unsubscribe by writing to birds-leave@lists.example.' Only a real NVDA or Narrator run settles either.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T15:59:50.717Z",
    "resolved_at": null
  },
  {
    "id": 95,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/mime.rs",
    "line": null,
    "description": "Whether real mailing lists write List-Unsubscribe in the shape where_to_write_to_leave reads has never been measured. No mail account has ever been used with this program. Sixteen spellings of the header were probed against mail-parser 0.11.5, which settles what the library does with a given header and says nothing about what senders send. If real lists commonly write the header in a shape that carries no angle-bracketed mailto:, the warning fires and always says to look for a link, which is a weaker feature than it reads as here.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T15:59:58.905Z",
    "resolved_at": null
  },
  {
    "id": 96,
    "kind": "deviation",
    "phase": "04",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "The plan does not mention HEADER_FIELDS, the list of headers an IMAP fetch asks a server for. Without LIST-UNSUBSCRIBE on it, every other hop of this feature is correct and no message on an IMAP account carries the header, which is the whole feature dead on the commonest account type with 6270 tests green. Added, and guarded by a record coupled to tests/the_list_warning_reads_the_message.rs, because the whole library was run against the break and stayed green: nothing in it can see this hop at all. Recorded because a request that names the fields it wants is a silent-drop point invisible to tests on either side of it, and the same shape exists wherever a projection is narrowed.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T16:00:07.187Z",
    "resolved_at": null
  },
  {
    "id": 97,
    "kind": "deviation",
    "phase": "04",
    "file": "src/service/outlook_data_file.rs",
    "line": null,
    "description": "A message imported from an Outlook data file carries no List-Unsubscribe, so blocking its sender gets no warning. The importer rebuilds a message from the pieces a PST holds and the transport headers are not among the pieces it reads. Left as None with a comment saying so rather than papered over: recovering it means reading the header property out of the file and writing it into the bytes the importer then re-parses, which is its own change. Messages filed from a sent copy and from an archive read through mime::parse do carry it. Corrected on 2026-09-16 by 09-08: this described an importer path that did not exist, since nothing called the reader from the day it was written; File, Import Mailbox reaches it now through application::importing_an_outlook_data_file, so the deviation is real and stays open, and the fix is the same.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T16:00:13.954Z",
    "resolved_at": null
  },
  {
    "id": 98,
    "kind": "deviation",
    "phase": "04",
    "file": "src/application/blocking.rs",
    "line": null,
    "description": "where_to_write_to_leave names whatever sits between <mailto: and > without asking whether it is an address, so a sender can put a web address or any other text there and have the warning say 'unsubscribe by writing to' it. Left alone deliberately, and the reasoning matters more than the decision: validating it would only reject malformed junk, because the real threat is a well-formed address belonging to somebody else, which no validation can tell from a real one. Nothing in this program acts on the value, so nobody is one keystroke from a stranger either way. Recorded so the judgement is visible rather than assumed.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T16:00:21.699Z",
    "resolved_at": null
  },
  {
    "id": 99,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-02-PLAN.md",
    "line": null,
    "description": "Two of this plan's premises were wrong in ways that would have shipped a broken feature or a weaker test, and both were found by measuring rather than reading. It says to read the header through header_text as receipt_request does; mail-parser parses List-Unsubscribe with its address parser, which strips the angle brackets where_to_write_to_leave searches for, so that route reports every mailing list as one that gave no way out. And it says the second task's census cannot be red because it must name a construction the first task creates; the construction already existed and only its argument changed, so the census was red before any implementation. Recorded because both are general: an accessor's parsed and raw forms are different values, and 'no red is available' is a claim about the tree that is cheaper to falsify than to work around.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T16:00:31.782Z",
    "resolved_at": null
  },
  {
    "id": 100,
    "kind": "deviation",
    "phase": "04",
    "file": "guards/guards.toml",
    "line": null,
    "description": "Two pre-existing guard records were found wrong, both surfaced by the count check because src/application/mail_sync.rs gained one test. 'a sync writes no attachment for a message nobody has opened' had been UNMEASURABLE since 04-01 landed hours earlier: its recorded break writes a CachedAttachment literal, 04-01 added a description field to that struct, and the break stopped compiling, so the run reported a broken tool rather than a finding. 'a count and the thing it counts agree in number' named 16 tests for a break that reddens 17, missing one in application::contacts_sync, a module nobody working on mailing lists would have filtered for. Both corrected by hand and re-measured. Recorded because neither has anything to do with this feature and neither would have been found by any check this plan ran on purpose.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T16:39:54.486Z",
    "resolved_at": null
  },
  {
    "id": 101,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Nobody has heard the encryption sentence. The reader speaks said_before_the_message when a message opens, so the sentence is spoken before the body, and two things about that are judgements rather than measurements: whether it lands early enough that somebody arrowing into a message meets the explanation before the armour, and whether 'This message is encrypted. Wixen Mail cannot open it, so what is shown below is the encrypted form rather than the message.' is understood at speed. Only a real NVDA or Narrator run settles either.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:00.937Z",
    "resolved_at": null
  },
  {
    "id": 102,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Whether a bar carrying a filter's verdict and an encryption sentence together reads as two facts or as one run-on has never been heard. The two are joined with a newline, the filter's verdict keeps the top, and the encryption sentence goes under it. In a text control that is two lines; spoken by a screen reader it may be one breath, and a phishing warning running straight into an explanation of armour is a sentence somebody may hear as one claim about one thing. Only a real NVDA or Narrator run settles whether the join needs more than a line ending.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:16.997Z",
    "resolved_at": null
  },
  {
    "id": 103,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Whether the PGP signature sentence is heard as a disclaimer or as reassurance is the one that matters most and is the one tests cannot reach. It reads 'This message carries a PGP signature, which Wixen Mail cannot check, so nothing here says whether it is genuine.' Tests assert the words it does not contain, which is a check on the wording and not on what somebody takes away. Being told a message is signed is easily heard as being told it is genuine, and if the second clause is talked over the first clause is reassurance nothing earned. Only a listener settles it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:17.388Z",
    "resolved_at": null
  },
  {
    "id": 104,
    "kind": "deviation",
    "phase": "04",
    "file": "src/application/body_safety.rs",
    "line": null,
    "description": "Whether real PGP mail arrives with its armour in a text part at all has never been measured, and it decides whether this feature fires in practice. The detection reads the two halves of a parsed body for the armour markers, which is how inline PGP arrives. Mail sent as multipart/encrypted carries the armour in an application/pgp-encrypted part, which mime::parse's first_of_kind does not yield as a body, so it never reaches this and would open with nothing said. No mail account has ever been used with this program, so which of the two real senders use cannot be answered here. Said in the changelog as a known limitation rather than implied away.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:28.331Z",
    "resolved_at": null
  },
  {
    "id": 105,
    "kind": "deviation",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "A conversation of several messages read as one document says nothing about any one message's form, so an encrypted message inside a thread still shows its armour with nothing said. reader_text::conversation folds the sentence in only when the document holds exactly one part. The reason is the one with_signature already gives for staying off a thread: there is a form per message and one bar over all of them, so 'This message is encrypted' over a thread of five is heard as covering five. Closing it properly means a sentence naming which message, which is its own wording question. Opening that message on its own does say it, and that is how somebody reads a particular message.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:28.728Z",
    "resolved_at": null
  },
  {
    "id": 106,
    "kind": "deviation",
    "phase": "04",
    "file": "tests/an_encrypted_message_is_not_left_unexplained.rs",
    "line": null,
    "description": "The census written for this plan passed against its own break the first time it was measured, and the fix is worth remembering as a class. It asserted that each composer asks the encryption question. The break took out the fold that puts the answer into the bar and left the question in place, bound to an unused name, so the composer still named the call and nothing reached the reader. A call site has three independent ways to be hollow: the call absent, the result discarded, and the argument a constant that makes the call decide nothing. The census now asserts all three and has a companion per shape. Found only by applying the break by hand; reading the census had already declared it sufficient.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:42.457Z",
    "resolved_at": null
  },
  {
    "id": 107,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-03-PLAN.md",
    "line": null,
    "description": "Three of this plan's premises were wrong. It says to reach the fact from the message-open path outside the look_at_message_contents gate and to write a census anchored on that setting's arm; the gate runs on a worker at body-fetch time and writes a verdict into a column, the bar is built later from the stored row, and nothing on the display path reads the setting, so the prescribed census would have read an unrelated function. It says to fold the sentence in under whatever the bar already says; said_before_the_message cuts at HOW_IT_WAS_CHECKED, which a signature verdict inserts, so an appended sentence is in the bar and spoken by nothing. And its guard-record table says reader_text.rs is fingerprinted by no record and holds 76 tests, where one record names it and it held 81, both figures true before 04-01 landed hours earlier. Third plan running in this phase whose record table expired against a same-day sibling.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T18:22:42.866Z",
    "resolved_at": null
  },
  {
    "id": 108,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_reader.rs",
    "line": null,
    "description": "Whether a StaticBitmap in a reader tab is reachable by a screen reader at all is the question this feature rests on and no test here can ask. A bitmap is not focusable by default on Windows, so NVDA may meet it only in browse mode or not at all, and the accessible name set through set_accessible_name may never be spoken. If it is not reached, the picture serves a sighted reader and is silent for everybody else, which is the half of the feature Pratik ordered first. Only a real NVDA and Narrator run settles it. Tab order is now warning bar, text, picture, attachment list.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:21:54.325Z",
    "resolved_at": null
  },
  {
    "id": 109,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_reader.rs",
    "line": null,
    "description": "Nothing in this repository has ever checked that a decoded attachment is drawn at a sensible size, is legible against either theme, or that a very wide picture does not push the attachment list off the tab. The bitmap is set to AspectFit for exactly that reason and the reasoning has never met a window. The theme sweep checks its background and foreground colours, which is not the same as somebody looking at it. A picture shown badly is not the same as a picture shown. Only an eye settles it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:22:20.234Z",
    "resolved_at": null
  },
  {
    "id": 110,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Whether the first lines of a preview tab are announced at the moment it opens, or whether somebody has to go looking for them, is untested. The whole design rests on the description being heard first: it is the first line of the text control and the caret starts at the top, and the opening announcement says the title and the attachment summary rather than the note. So somebody who hears the announcement and does not read on may never meet the sentence that says whether the picture came with a description. The same question applies to a text attachment's note, which says whether the file was cut. Only a listener settles it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:22:20.648Z",
    "resolved_at": null
  },
  {
    "id": 111,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/plain_text.rs",
    "line": null,
    "description": "Whether real senders' text attachments are UTF-8 has never been measured and it decides how often the not-entirely-text sentence fires. A log or a CSV written on a Windows machine in an older code page decodes as mostly replacement characters, and this refuses to guess at an encoding on purpose. If most real text attachments are not UTF-8, most previews will be a screenful of replacement characters with an honest sentence above them, which is worse than it sounds and would argue for encoding detection. No mail account has ever been used with this program, so it cannot be answered here.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:22:21.063Z",
    "resolved_at": null
  },
  {
    "id": 112,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/picture.rs",
    "line": null,
    "description": "Whether real senders put Content-Description on image parts at all decides whether an image preview usually says what is in the picture or usually says nothing is known about it. Unmeasurable here, and unchanged since the research raised it as assumption A5. Now that the picture is also drawn, the cost of the answer being no falls on a blind reader alone rather than on everybody, which makes it more worth measuring rather than less.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:22:21.453Z",
    "resolved_at": null
  },
  {
    "id": 113,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-04-PLAN.md",
    "line": null,
    "description": "Two of this plan's premises could not be followed as written. Its guard table says reader_text.rs is fingerprinted by no record and theme_reach.rs by three; four records name the first and none name the second, both wrong within hours of the plan being written, which is the fourth phase-04 plan whose record table expired against a same-day sibling. And it asks the task 2 RED commit to name both a new test saying a picture can be read here and the existing test saying it cannot; those are opposite assertions about one function, so exactly one can be red and the gate holds a red commit to every named test really failing. The gate was widened at the red instead, which makes the existing assertion the red one, and the commit says so.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-05T23:22:21.828Z",
    "resolved_at": null
  },
  {
    "id": 114,
    "kind": "stub",
    "phase": "04",
    "file": "tests/house_style.rs",
    "line": null,
    "description": "Corrected 2026-09-06, hours after being written and by the plan revision that read the code. A check DOES compare the model against the screen for top-level settings: every_setting_is_acted_on in src/data/config.rs reads the AppConfig struct itself and test_every_setting_somebody_can_change_is_offered_by_a_screen asserts every public field is offered, so a new top-level setting fails on arrival. That correction then misdiagnosed the remaining gap as nesting, and phase 6 research went to look on 2026-09-06 and found neither exception is nested. allowed_per_account is a TOP-LEVEL AppConfig field at config.rs:276, fully visible to the check and excused by name through STORED_AND_OFFERED_BY_NOTHING at config.rs:1799, a list holding exactly one entry. The per-event feedback channels are a third shape no name-based check can reach: they live inside the SERIALISED STRING VALUE of feedback_channels, so there is no field to find at any depth. Therefore the work this entry originally described, widening the check to follow nesting, closes NEITHER exception and should not be done as stated. What closes them is a hand-named companion on the pattern of test_whether_message_text_may_be_fetched_is_offered_by_a_screen at config.rs:1829, costing two guard records rather than disturbing all five tests in that module. Second-order trap from the same research: emptying STORED_AND_OFFERED_BY_NOTHING makes test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing at config.rs:1947 iterate over nothing and pass unconditionally, which is the census-emptying failure CLAUDE.md already documents, so whoever offers the per-account answer from a screen retires that guard in the same commit. The original entry claimed nothing enforced this at all, written straight after the rule without going to look, which is the mistake the rule itself is about. Two settings break the rule today and both predate it: per-event feedback channels, whose per_event field and set_event_channels are private so no screen could write one, which is FEEDBACK-01 and phase 6's; and the per-account Allow Changes answer. Phase 1's criterion 8 already said a phase must not add a third. Writing the check means enumerating the settings surface and the screen's sections and asserting every member of the first appears in the second, the way tests/one_sign_in_per_piece_of_work.rs counts sign-in sites and names each. Recorded 2026-09-06 when Pratik made the rule explicit.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T00:04:31.844Z",
    "resolved_at": null
  },
  {
    "id": 115,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/safety.rs",
    "line": null,
    "description": "Whether a warning bar carrying attributed sentences from two or three sources is heard as separate facts or as one run-on. Every sentence now names which of the four checks reached it, and this program's own reading says everything it found in one sentence so attribution does not become repetition. That the structure is right is asserted; that it is heard right is not, and only NVDA or Narrator on a real message settles it. Measured bar, three sources: 'This message was marked as spam. Your mail provider's filter marked it as spam. Your mail provider put it in the junk folder. Wixen Mail read this message on your computer and found a link that says it goes one place and goes somewhere else.'",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T04:45:08.263Z",
    "resolved_at": null
  },
  {
    "id": 116,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/filters.rs",
    "line": null,
    "description": "Whether 'Safety' is recognised as being about spam and phishing when it is read out as one of twelve field names in a rule editor. The words were chosen to match the message list's own column header rather than inventing a second name for one thing, which is the right trade if somebody has met that column and the wrong one if they have not. Only a screen reader run through the rule editor's field list settles it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T04:45:16.983Z",
    "resolved_at": null
  },
  {
    "id": 117,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/safety.rs",
    "line": null,
    "description": "Whether provider spam headers arrive in the shapes from_headers expects. Every parser in it is tested against hand-written header blocks and no account has ever been used with this program, so X-Spam-Flag, X-Spam-Status, X-Forefront-Antispam-Report, X-Microsoft-Antispam and Authentication-Results are all unverified in the field. Gmail in particular tells an IMAP client almost nothing beyond moving the message, which is why the folder counts as a verdict. Unchanged by this plan and now more load-bearing, because a filter rule can act on the answer.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T04:45:17.430Z",
    "resolved_at": null
  },
  {
    "id": 118,
    "kind": "stub",
    "phase": "04",
    "file": "src/application/mail_sync.rs",
    "line": null,
    "description": "A filter rule that deletes is not said out loud. say_what_the_rules_did produces '{n} sorted by your rules' and carry_out counts a delete in that number alongside a rule that only marked something read, so nothing anywhere says a message was deleted by a rule. Found while adding the safety verdict as a rule field, which is what makes it worth recording now: T-04-19 is a user rule filing wanted mail out of sight on a verdict a sender can shift, and the mitigation the plan asked for was that the rule says what it did. The deletion is local only, cache.delete_message, not a server delete, so mail is hidden rather than destroyed; that lowers the severity and does not close it. Recorded as a finding rather than fixed, as 04-05-PLAN asked. The fix is a count of its own in Filtered and a sentence in say_what_the_rules_did.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T04:45:31.615Z",
    "resolved_at": null
  },
  {
    "id": 119,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-05-PLAN.md",
    "line": null,
    "description": "Five premises could not be followed as written, and the first changed the size of the job by an order of magnitude. The plan and 04-RESEARCH both said CachedMessage.safety was already on the struct the matcher is handed, citing messages.rs:357; that line is inside listing_row, which builds MessageListRow, whose own doc says 'Deliberately not CachedMessage' three lines above it. CachedMessage had no such field, so criterion 6 needed a new field, 36 construction sites, and two SQL reads that had never selected the column (get_message, which the arrival path uses, and scan_query, which saved searches use). Also: the quiet.len() assertion reddens at GREEN and can never be in a RED commit's trailers; the guard table says body_safety.rs has 0 records and 13 tests (really 1 and 20) and mail_sync.rs 8 and 134 (really 9 and 135); both verify commands are invalid cargo ('--lib' cannot be used multiple times); and premise 5 says the provider sentences end at a common phrase when three of them begin with it and the two Authentication-Results sentences named nobody at all. This is the fifth phase-04 plan carrying a wrong premise. All five are written into the plan file.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T04:45:32.068Z",
    "resolved_at": null
  },
  {
    "id": 120,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/editor_document.rs",
    "line": null,
    "description": "Whether NVDA announces the browser engine's spelling marks in a WebView2 contenteditable in this application at all. That is the premise of the two clauses of criterion 3 that already shipped before this plan, and nothing in this tree can test it. If it does not, marking as you type is decorative and the walk keys are the only spelling check that reaches a screen reader.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:00.122Z",
    "resolved_at": null
  },
  {
    "id": 121,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "Whether this program's landing sentence and the screen reader's own announcement of the marked word collide when the caret lands on a misspelling. Two voices for one fact, arriving in the same instant: the selection change is what makes the reader speak the word, and walk_to_a_misspelling speaks its own sentence immediately afterwards. Whether that is heard as one answer or as an interruption is not testable here.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:18.344Z",
    "resolved_at": null
  },
  {
    "id": 122,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/accessibility/feedback.rs",
    "line": null,
    "description": "Whether the earcon at the end of a mistyped word and the landing announcement of the walk keys are told apart. Both are about a misspelling and both fire while somebody is working in the message body; guardrail 5 asks for feedback that is distinct, and nothing here can say whether these two are.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:18.882Z",
    "resolved_at": null
  },
  {
    "id": 123,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/spell_session.rs",
    "line": null,
    "description": "Whether three suggestions is heard as helpful or as a list to sit through, and whether '7 suggestions in all' is heard as useful or as noise. The bound is a judgement written into SUGGESTIONS_SAID with its reasoning; only listening settles whether it is the right number.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:19.316Z",
    "resolved_at": null
  },
  {
    "id": 124,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-06-PLAN.md",
    "line": null,
    "description": "The plan's two tasks are not separable as written: a forward key and the same key with Shift are one if, one message field and one match with two arms, so task 1's minimum finishes task 2 and task 2's required inverse property test cannot be red. Task 1 was narrowed to have no direction parameter anywhere before task 2 could carry one. Also: both verify commands are invalid cargo, repeated from 04-05 after that summary reported them; premise 2 is right that enumeration and the caret move ship, and misses that nothing could report where the caret is, which is new page code; and the sentence for a word with more suggestions than the bound is unreachable unless the speller is asked for more than the bound, which the plan does not name. Premise 8's guard table is right in every row, the first time in two phases.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:19.717Z",
    "resolved_at": null
  },
  {
    "id": 125,
    "kind": "stub",
    "phase": "04",
    "file": "tests/wired.rs",
    "line": null,
    "description": "documented_combinations only collects a backticked key that starts with Ctrl+ or Alt+, so a documented combination spelled Shift+Alt+F7 rather than Alt+Shift+F7 is skipped in silence: no exception entry is needed for it and no protection is given either. The check exists because three documented keys were dead at once, and the order somebody writes the modifiers in decides whether it looks at all.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T07:00:20.122Z",
    "resolved_at": null
  },
  {
    "id": 126,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/attaching.rs",
    "line": null,
    "description": "Whether a batch announcement naming six files is heard as a confirmation or as something too long to sit through. NAMED_ALOUD is 6 on the reasoning that six names is about ten seconds of speech; nothing here can say whether ten seconds is right, or whether the names should be dropped entirely in favour of the count and the total.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:32:55.803Z",
    "resolved_at": null
  },
  {
    "id": 127,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/attaching.rs",
    "line": null,
    "description": "Whether the refusal for a folder among five files is told apart from the names of the four that went on. Two sentences arrive one after the other, the first naming what is attached and the second naming what is not, and only listening says whether that reads as two facts or as one long list.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:33:15.056Z",
    "resolved_at": null
  },
  {
    "id": 128,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "Whether Ctrl+V on the attachments list or the toolbar is discoverable at all by somebody who has never read the shortcuts document. Nothing in the composer says the key exists: it is not on a button label, not in an accessible name and not announced. The picker is discoverable and this is the quicker route, so a key nobody finds is a keyboard equivalent that only exists on paper.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:33:15.494Z",
    "resolved_at": null
  },
  {
    "id": 129,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "Whether a file dropped on the composer attaches at all. The drop target is installed on the dialog and hands its paths to the same door the picker uses, and whether a drop over the message body reaches it is unknown: the body is a WebView2 control that handles drops in its own window. Task 3 of plan 04-07 is a person dragging a file to settle it, and the answer goes in the product either way.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:33:15.932Z",
    "resolved_at": null
  },
  {
    "id": 130,
    "kind": "stub",
    "phase": "04",
    "file": "src/common/error.rs",
    "line": 58,
    "description": "Error::Other displays as 'Error: {message}', so every refusal this program says out loud opens with the word Error before the sentence. Found by a test pinning the composer's single-file refusal against the old code rather than against a copy of it. Poor wording for a screen reader and unchanged here, because it is a change to common::Error and to every announcement that goes through it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:33:16.343Z",
    "resolved_at": null
  },
  {
    "id": 131,
    "kind": "deviation",
    "phase": "04",
    "file": ".planning/phases/04-writing-and-reading-a-message-in-full/04-07-PLAN.md",
    "line": null,
    "description": "Four premises corrected. Premise 5 says the paste key needs a second home because the attachments list is hidden when empty; it needs no second home, because wxWidgets passes an unhandled key up the parent chain and the composer already uses one dialog-level handler for exactly this, so Ctrl+V is bound once there. The plan's tests/wired.rs change is unnecessary: Ctrl+V is already documented and already bound as a menu accelerator in the main window, so bound_somewhere finds it and no exception entry is needed, which also means that check gives the composer's Ctrl+V no protection at all. The threat register asks for fixtures with a traversing name and a reserved Windows device name; neither can exist as a real file on Windows, and what protects those cases is Chosen::at asking for the last component and safe_file_name prefixing a device name, both tested where they live. No version bump in task 2: there is nothing true to write in a changelog entry until task 3 answers whether a drop lands, and this project pairs a bump with an entry. Premise 8's guard table is right in every row: 617 records, attaching 0, wx_compose 2, wired 8, attachment_name 0.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T08:33:16.753Z",
    "resolved_at": null
  },
  {
    "id": 132,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/editor_document.rs",
    "line": null,
    "description": "Whether a file dragged onto the composer reaches anything at all. The page now turns a drop on the message body away so the engine cannot navigate to the file, and the composer says where a drop does land, and both rest on documentation rather than on somebody watching: WebView2's AllowExternalDrop defaults to true so the drag reaches the page, and the OLE drag loop walks up the parent chain so a drop on the subject line, the toolbar, the attachments line or the list should reach the dialog's target. Nobody has dragged a file onto a running composer. The four observations to make are in 04-07-SUMMARY.md.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T10:49:18.354Z",
    "resolved_at": null
  },
  {
    "id": 133,
    "kind": "deviation",
    "phase": "04",
    "file": "Cargo.toml",
    "line": null,
    "description": "Upstream gap, not reported yet: WebView2's AllowExternalDrop cannot be turned off through this stack. It lives on ICoreWebView2Controller4; wxWidgets 3.3.2 keeps the controller in wx/msw/private/webview_edge.h and wxWebViewEdge::GetNativeBackend returns the ICoreWebView2_2 underneath it, which cannot be asked for its controller, so wxdragon 0.9.17 has nothing to expose. The reachable mitigation is the page refusing the drop itself, which depends on the page's script having loaded and so is weaker than the host-level setting would be. Worth an issue against wxWidgets or wxdragon.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T10:49:28.613Z",
    "resolved_at": null
  },
  {
    "id": 134,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/attaching.rs",
    "line": null,
    "description": "Whether the sentence said when a file is dropped on the message area is heard as an answer or as an interruption. It arrives spoken at high priority and shown in a message box at the same time, three sentences long, at the moment somebody let go of a file. Nothing here can say whether that is help or a modal in the way, or whether naming Ctrl+V and Attach File in speech is how somebody finds them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T10:49:29.074Z",
    "resolved_at": null
  },
  {
    "id": 135,
    "kind": "stub",
    "phase": "04",
    "file": "src/presentation/html_renderer.rs",
    "line": null,
    "description": "The setting that says where a decorative picture is reaches the formatted reading path only. html_to_plain_text strips every tag, so in the plain text reading path no picture says anything at all, described or decorative, and this setting is inert there. Older than 04-08 and not fixed here: it changes what every plain text reader hears on every message with a picture in it, so it needs its own red, its own green and its own changelog line.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:50:55.540Z",
    "resolved_at": null
  },
  {
    "id": 136,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "Nobody has heard the decorative question read aloud. It is a Yes/No box whose text runs to four lines and names what each answer does, and the buttons say only Yes and No. Whether somebody hearing it understands that Yes sends a picture with nothing said, whether four lines is too much to hold while reaching for a button, and whether Enter arriving on No is felt as safe or as an obstacle, are all things only a real screen reader run can settle.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:51:18.753Z",
    "resolved_at": null
  },
  {
    "id": 137,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "The Say where a picture the sender marked decorative is check box has not been heard. Its label carries the mnemonic and set_accessible_name_and_description names it on the MSAA channel, but nothing here can say whether Narrator reads the label from the window text, whether NVDA reads the accessible name rather than the neighbouring text, or whether the description saying what off means is reached at all.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:51:19.324Z",
    "resolved_at": null
  },
  {
    "id": 138,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/pictures.rs",
    "line": null,
    "description": "Nobody has heard a mailing with thirty spacers in it. Announcing decorative pictures ships on, and guardrail 5 forbids feedback that floods. The words are short and go into the document rather than into the announcement queue, so they are passed over rather than spoken at, but whether a bulk mail template that marks twenty layout images decorative turns a message into a wall of the same phrase is unmeasured.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:51:19.876Z",
    "resolved_at": null
  },
  {
    "id": 139,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/pictures.rs",
    "line": null,
    "description": "The furniture threshold is a judgement with no field data behind it. A shorter side of 200 pixels and 100 KB on disk were chosen from what furniture is, not from measuring real mail. Nobody has run it over a real mailbox to see how often the decorative question is offered over something that is not furniture, or refused over something that is.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:51:20.326Z",
    "resolved_at": null
  },
  {
    "id": 140,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/application/pictures.rs",
    "line": null,
    "description": "No decorative picture has been sent to a real recipient. Whether an inline picture sent as multipart/related with an empty alt arrives at Gmail or Outlook with that alt intact, and whether their readers then skip it, has never been tested, because no message from this program has ever reached anybody.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T13:51:20.786Z",
    "resolved_at": null
  },
  {
    "id": 141,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/signed_mail.rs",
    "line": null,
    "description": "EncryptedMessage::read has never met a real envelope from a real sender. Its fixture is real OpenSSL output, which is more than hand-built DER and is not mail: nobody has put an enveloped message from Outlook or Thunderbird through it. If one does not parse, the reader says the message is encrypted and its details could not be read, which is a wrong message rather than an absent one, and that is worse than the blank body it replaces. The refusal path is what bounds it and it is tested against every seventh-byte prefix of the fixture.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T17:25:10.804Z",
    "resolved_at": null
  },
  {
    "id": 142,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Nobody has heard an encrypted message open. Three things only a screen reader run settles: whether the sentence in the body is reached before somebody concludes the message is broken, whether hearing the same sentence in the bar as the message opens and again in the body is heard as thoroughness or as repetition, and whether the count in it addressed to 1 certificate is understood at all by somebody who has never been told what a certificate is.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T17:25:20.115Z",
    "resolved_at": null
  },
  {
    "id": 143,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/signed_mail.rs",
    "line": null,
    "description": "Whether this computer holds a certificate an encrypted message was addressed to has never been asked of a real Windows certificate store holding a real S/MIME certificate. which_recipient_is_us is exercised against a store held in memory, so the three answers are tested and the Windows path that produces them is not. A store that answered wrongly rather than failing would tell somebody a private message was not meant for them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T17:25:20.633Z",
    "resolved_at": null
  },
  {
    "id": 144,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/service/pgp/keys.rs",
    "line": null,
    "description": "No real key and no real PGP message have been through this. The fixtures are a key pair and a message made by GnuPG 2.4.9 rather than by rPGP, which is two implementations agreeing rather than one agreeing with itself and is the strongest evidence available here, and it is still not a message from a correspondent. What goes wrong if it is not enough runs both ways and both are worse than the armour this replaces: a message shown as opened that was not, or a message refused that another client opens.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T19:22:31.977Z",
    "resolved_at": null
  },
  {
    "id": 145,
    "kind": "stub",
    "phase": "04",
    "file": "src/application/opening_pgp.rs",
    "line": null,
    "description": "PGP/MIME is not read. what_the_form_says reads the message's text parts, so only inline PGP, where the armour sits in the body, ever reaches the opener. A multipart/encrypted message puts the armour in a separate application/octet-stream part that never becomes body text, so such a message is neither opened nor reported as failing to open: nothing sees it. Thunderbird and most modern clients send PGP/MIME, so this is the common shape rather than a corner. Gated in the product by the experimental warning on the menu item and named in the changelog.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T19:22:32.467Z",
    "resolved_at": null
  },
  {
    "id": 146,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nobody has heard the PGP import or any of its five answers read aloud. The import is a file picker followed by one announcement at High priority, and whether the sentence for a public key, a locked key, a file that is not a key, a refused credential store or a successful import is understood on hearing it once, with no dialog to go back to, is a thing only a real screen reader run settles.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T19:22:32.974Z",
    "resolved_at": null
  },
  {
    "id": 147,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nobody has heard the hold announced. Pressing Send now says \"Sending in 10 seconds. Undo Send takes it back.\" and whether that sentence finishes in time for somebody to hear it, decide and press Ctrl+Shift+Z inside ten seconds is the whole argument in Hold::DEFAULT's doc and no test can measure it. Plan 04.2-04's checkpoint, item 4, asks this question in the flow it matters most in.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T23:11:30.841Z",
    "resolved_at": null
  },
  {
    "id": 148,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A message that leaves after a hold has never met a real server. Whether it arrives with the headers a recipient's client expects, and whether ten seconds feels long or short with a real mailbox syncing underneath, are both unsettled.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T23:11:39.000Z",
    "resolved_at": null
  },
  {
    "id": 149,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Nobody has heard the Schedule window. Whether a month choice, a day spinner, a year spinner, an hour spinner, a minute spinner and sometimes a morning-or-afternoon choice are heard as six separate named controls, each saying its own value as it changes, is the question wx_item_form.rs's module doc settled with a real screen reader session for that dialog and which has not been settled for this one. The control shape is the same and the names are set with set_accessible_name rather than set_name, so the expectation is that it carries over. That is an expectation, not a measurement.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T23:59:00.000Z",
    "resolved_at": null
  },
  {
    "id": 150,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Nobody has heard a refusal in the Schedule window. When a time will not do the window stays open, the reason is put in its problem line and announced at High priority through said_and_shown, and focus moves to the month control. Whether the sentence is actually heard, or is lost under whatever the screen reader says about the control focus just landed on, is the failure mode this pairing exists to avoid and only a real run settles it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T23:59:10.000Z",
    "resolved_at": null
  },
  {
    "id": 151,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "No message has ever waited hours for its time and then gone. Every test moves the clock; nothing runs the program for an afternoon. Whether a scheduled message really leaves when its moment arrives with the program left running, whether the poll timer is still asking after hours, and whether a message written on Monday and sent on Tuesday carries headers a recipient's client accepts, are all unsettled and none of them can be tested here.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-06T23:59:20.000Z",
    "resolved_at": null
  },
  {
    "id": 152,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/application/attaching.rs",
    "line": null,
    "description": "No organiser's calendar has ever received a reply from this program. The part now leaves declared text/calendar; charset=utf-8; method=REPLY and named reply.ics, asserted over the whole path from the answer the window builds to the Ready the send loop puts on the wire, but nothing here has met a real mail server. Whether Outlook, Google Calendar or Thunderbird actually folds the answer into the meeting and updates the guest list is what the whole change is for and is the one thing no test in this repository can settle.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T03:43:34.557Z",
    "resolved_at": null
  },
  {
    "id": 153,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/application/attaching.rs",
    "line": null,
    "description": "Nobody has forwarded an invitation from this program to a real recipient. A .ics attachment is now declared method=REQUEST or method=CANCEL from what the document says, which changes what a recipient's client offers them, and no client has ever been shown one. Whether a forwarded invitation really presents as a meeting to answer, and whether the ORGANIZER inside it is attributed to whoever called the meeting rather than to the forwarder, are unsettled here.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T03:43:43.233Z",
    "resolved_at": null
  },
  {
    "id": 154,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/application/answered_meetings.rs",
    "line": null,
    "description": "The accepted meeting is written with pending set, which is what puts it in front of the push, and no push path in this project has ever run against a real account. Whether a CalDAV, Google or Microsoft calendar accepts an event this program created from an invitation, with the invitation's own UID as the provider identity and no etag, is unsettled. If a provider refuses it the meeting stays correct on this computer and never reaches any other device, which reads to the person exactly like it working.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T05:52:49.797Z",
    "resolved_at": null
  },
  {
    "id": 155,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Answering a meeting has not been heard with a screen reader. Four things are unsettled by ear: whether accepting is announced once or twice now that the answer is filed as well as sent; whether the meeting is read out with its time and its busy state when the calendar is opened afterwards; whether a declined meeting is read as free rather than booked; and, since pressing Accept sends with no confirmation step by the decision of 2026-09-06, whether anything spoken in the seconds after pressing it by mistake points at Undo Send. The last is the one structure cannot show, and a finding of nothing pointed at Undo Send is a result worth having rather than a failed run.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T05:52:58.235Z",
    "resolved_at": null
  },
  {
    "id": 156,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/html_renderer.rs",
    "line": null,
    "description": "The sentence about held-back pictures has not been heard with a screen reader. It is placed above the message body, after the message's own heading in a conversation, and read in order it arrives before the thirty markers it is about. Whether it is heard as orientation or as one more thing in the way is the only question here that structure cannot answer, and the answer changes the placement rather than the words: under the heading instead of above the body, or at the end, or in the announcement path instead of the document. Thirty markers in a marketing message is the case to try it on.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T07:03:59.342Z",
    "resolved_at": null
  },
  {
    "id": 157,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "One hop is uncovered and it is the hop this plan is about. wx_compose's preview now asks for HtmlRenderer::for_a_message_being_written, and no test drives that window, so changing that one line back to new() reddens nothing. It was measured rather than assumed: the guard record breaking the constructor reddens a test, and a record breaking the call site would redden none. The constructor's own answer is pinned by a test, and the default a caller gets by saying nothing is the reading one, so a new call site that forgets is wrong in the direction that tells a reader too much rather than a writer something false. What is unverified is that this particular call site still asks.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T07:04:11.587Z",
    "resolved_at": null
  },
  {
    "id": 158,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/html_renderer.rs",
    "line": null,
    "description": "test_a_message_with_nothing_held_back_says_nothing was rewritten against the document and was green on arrival, because at the time of the rewrite no document said the sentence at all. It is the assertion that stops an ordinary message growing a line about pictures nobody held back, and it has never been red. Taking the emptiness check out of what_a_reader_is_told_was_held_back by hand would settle whether it would notice; it was not done.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-07T07:04:12.210Z",
    "resolved_at": "2026-09-07T07:21:59.339Z"
  },
  {
    "id": 159,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_blocked_senders.rs",
    "line": null,
    "description": "Whether each row of the blocked list is read as a person, a destination and a state, or as one run-together string. This is a Report ListCtrl and wxdragon 0.9.17's get_item_text loses the last character of every cell and returns a NUL in its place, which is ledger 28 and is upstream and unfixed. If it shows here it is somebody else's defect and it still lands on the person using this. Nothing in this repository can answer it: only NVDA and Narrator on a running build can.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T09:09:03.008Z",
    "resolved_at": null
  },
  {
    "id": 160,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_blocked_senders.rs",
    "line": null,
    "description": "Whether what unblocking did is heard over the list being re-read underneath it. The list is refilled and the row cursor is landed again before the sentence is announced, so a screen reader has a repainted control and a notification arriving close together. Whether the sentence is heard once, heard twice, or lost under the repaint is a question only a real run answers, and it decides whether the announcement should come before the refill rather than after.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T09:09:17.147Z",
    "resolved_at": null
  },
  {
    "id": 161,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_blocked_senders.rs",
    "line": null,
    "description": "Whether a switched-off block is distinguishable by ear from a working one. The state is a third column reading Working or Switched off, so it is catching nothing. Looking at the screen makes the difference obvious; hearing a row read cell by cell may not, and still_on's own doc says a list showing a switched-off block as working would be worse than no list. Also unheard: whether an account with nothing blocked is heard as empty on purpose rather than as a window that failed to load, which is what the sentence and the focus going to Close are for.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T09:09:17.667Z",
    "resolved_at": null
  },
  {
    "id": 162,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the sentence saying what blocking will do is heard before the block is made, and whether it and the may_block mailing-list warning together read as one thing or two. Both are said through the same told at Priority::High, one after the other, and this is the only place in this feature where somebody hears two sentences before a block. Guardrail 5 is what it is about: feedback must be distinct and bounded, and a sentence added before a block is the easiest place here to flood somebody. Announcements go out through UiaRaiseNotificationEvent, which reaches speech and braille at once, so a braille display should be checked as well.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T09:09:18.186Z",
    "resolved_at": null
  },
  {
    "id": 163,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/scan_target.rs",
    "line": null,
    "description": "Neither automated accessibility channel has been run over the new blocked-senders window. The plan's checkpoint asks for two, separately: Axe.Windows over UI Automation with the new blocked-senders scan target, confirming the scan really opened the window rather than reporting nothing about one it could not reach, and scripts/msaa-names.ps1 over the same window, which is what NVDA reads for native controls and the only channel set_accessible_name writes to. A name failing on either is a name somebody does not hear. There is also a trap in the other direction: a control with a visible label beside it inherits that label as its MSAA name even when nothing set one, so for each control the run has to say whether the name that came back is the one this code set or one Windows supplied. Deferred by decision, not attempted.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T09:09:27.449Z",
    "resolved_at": null
  },
  {
    "id": 164,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether Shift+F6 reaches the script injected into the message preview with shiftKey set. Nothing here has run the preview: the direction is proved from the payload inward, by tests over panes::leaving_which_way and panes::leaving_the_preview, and by a source read of the script and the handler. What no test can reach is the browser delivering the keystroke. The composer's own measurement in editor_document.rs records that Ctrl+backslash never arrives at its page handler on this machine while Ctrl+Shift+L and Ctrl+Enter beside it arrive every time, so a key being kept between the window and the page is a thing that happens here. F6 leaving the preview is known to work; Shift+F6 arriving as F6 with shiftKey true is not.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T12:37:49.096Z",
    "resolved_at": null
  },
  {
    "id": 165,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/panes.rs",
    "line": null,
    "description": "Whether landing back on the folder tree with Shift+F6 is heard as going back, or only as going somewhere. Arrival is announced with panes::arrival, which names the pane and what is in it and says nothing about direction, so F6 to the folder tree and Shift+F6 to the message list are announced the same way as any other arrival. Somebody who cannot see the layout may not be able to tell that the key went the way they asked, which would leave the fix invisible even though it works. Announcements go out through UiaRaiseNotificationEvent, reaching speech and braille at once, so a braille display wants checking too. This decides whether an arrival should say a direction, which would be a change to every route rather than to this one.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T12:38:01.449Z",
    "resolved_at": null
  },
  {
    "id": 166,
    "kind": "deviation",
    "phase": "04.2",
    "file": ".planning/phases/04.2-what-was-built-and-never-reached/04.2-07-PLAN.md",
    "line": null,
    "description": "Task 3 carried two acceptance criteria that cannot both hold. One requires grep -rn 'never takes focus' src/ docs/ to return nothing; the other requires docs/accessibility.md to be unchanged by the plan, and that file held the phrase at line 159. The plan's prose counted three places and the tree held four. The exclusion was written for a different sentence in that file, a braille one belonging to phase 6, so it was written without knowing the file also carried the phrase being swept. Resolved in favour of the grep: the false sentence was corrected and the braille sentence left alone.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T12:38:02.044Z",
    "resolved_at": null
  },
  {
    "id": 167,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether a message list rearranging itself as somebody moves between folders is announced at all. Moving from the inbox to Sent now changes which columns are shown and how the list is sorted, and nothing says so: the columns are the control's own headers, which a screen reader reads from the control when asked rather than when they change, and no announcement is made. So a list may quietly become a different list under somebody who cannot see it. Whether that needs saying, and what it should say without flooding somebody arrowing through a folder tree, is a judgement only a screen reader run makes.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T15:17:10.616Z",
    "resolved_at": null
  },
  {
    "id": 168,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_columns.rs",
    "line": null,
    "description": "Restore Defaults still announces the fixed sentence 'Columns reset to the default', which does not say which folder's defaults arrived. It now really does restore the ones for the folder somebody is in, and the two kinds differ by a whole column and by which date the list is sorted on, so the same six words describe two different outcomes. Whether somebody who cannot see the list can tell which they got, and whether the sentence should name the folder kind, is unverified by ear.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T15:17:23.347Z",
    "resolved_at": null
  },
  {
    "id": 169,
    "kind": "deviation",
    "phase": "04.2",
    "file": "src/presentation/message_columns.rs",
    "line": null,
    "description": "A build older than 0.85.0 reading a layout this build wrote keeps its columns and falls back to that folder's default sort. The kind is appended after an @ on the end of the stored string, which puts it inside the field an older build reads its sort from. The second sort level was free in both directions because a semicolon inside the sort field is what an older parser stops at; this field cannot be. Accepted rather than fixed, and written into to_stored's own doc. Putting the kind into the columns list instead would have cost nothing in either direction and was rejected as smuggling a non-column through a list of columns.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T15:17:23.911Z",
    "resolved_at": null
  },
  {
    "id": 170,
    "kind": "deviation",
    "phase": "04.2",
    "file": ".planning/phases/04.2-what-was-built-and-never-reached/04.2-08-PLAN.md",
    "line": null,
    "description": "One RED assertion had to be respelled in the GREEN commit. test_a_sent_layout_and_both_its_levels_come_back_as_a_sent_layout asserted back.kind == FolderKind::Sent against a two-argument from_stored, because a test that does not compile is not a red and red-commit.sh refuses one. The fix changed ColumnLayout::kind to Option and from_stored to one argument, so the same claim is now spelled Some(FolderKind::Sent) against from_stored(..).expect(..). The claim did not change and the test was really red for the right reason; the bytes did. Inherent to any red written about a type that does not exist yet, and worth a name so the next plan does not read the diff as the test being edited to pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T15:17:35.182Z",
    "resolved_at": null
  },
  {
    "id": 171,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/editor_document.rs",
    "line": null,
    "description": "Whether F8 really reaches the composer's toolbar on a machine other than the one where it was watched working, and whether Ctrl+backslash fails everywhere or only here. The project measured both once, on one machine, and recorded it in editor_document.rs's own comment. docs/KEYBOARD_SHORTCUTS.md now gives F8 as the way in and marks Ctrl+backslash as bound and not seen to arrive, which raises what a second machine disagreeing would cost: a page giving the wrong key is the exact defect this plan closed, and it would be closed in the wrong direction. The check that pairs the page with the bindings cannot tell a key that arrives from one that does not, and says so in its own words.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T16:53:28.127Z",
    "resolved_at": null
  },
  {
    "id": 172,
    "kind": "unrun-verify",
    "phase": "04.2",
    "file": "src/presentation/wx_compose.rs",
    "line": null,
    "description": "Neither key this plan wrote into the shortcuts page has been heard. Delete on the composer's attachments list announces Removed and the file's name at Priority::Normal, and whether that is heard over the list refilling under it, and whether the row cursor lands somewhere sensible after a row goes, is the same question ledger 160 asks of the blocked senders list. Closing the conversation window with F6 is the other: the frame is hidden and a timer hands control back to whatever opened it, and nothing here says where focus went or that the window closed at all, so somebody who pressed F6 expecting to move between panes may hear nothing and not know what happened.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T16:53:37.371Z",
    "resolved_at": null
  },
  {
    "id": 173,
    "kind": "deviation",
    "phase": "04.2",
    "file": "tests/wired.rs",
    "line": null,
    "description": "The plan asked for the code-to-doc direction in tests/wired.rs to be widened to read editor_document.rs and wx_compose.rs as well as wx_app.rs. It was not, and the reason is a measurement rather than a preference: every key those two files bind is already written in docs/KEYBOARD_SHORTCUTS.md by name, F7 F8 F6 Escape Tab Enter and Delete, so that widening reports nothing at all today and would guard nothing. The class of key that hid F8, Delete and F6 is not one the document never names, it is one the document names for a different surface, and no whole-document reader can see that. The per-surface reading went into tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs instead, which was red on all three.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T16:53:46.808Z",
    "resolved_at": null
  },
  {
    "id": 174,
    "kind": "deviation",
    "phase": "04.2",
    "file": "tests/a_key_is_documented_where_the_surface_that_binds_it_is.rs",
    "line": null,
    "description": "The per-surface check covers two surfaces and the shortcuts document describes about thirty. The composer's page and the conversation window are the two that were wrong, and both are now held. Every other surface that gives a key its own meaning is unheld: the reader window's F8 for attachments, the View menu's F8 for the Columns dialog, Delete in the message list, F6 for panes in the main window. Any of those could lose its binding or its documentation and both directions of the pair in tests/wired.rs would stay green, because the key name is in the document for one of the others. Generalising means giving each surface a heading and a reader, which is a real piece of work and is not what this plan bought.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T16:53:56.612Z",
    "resolved_at": null
  },
  {
    "id": 175,
    "kind": "deviation",
    "phase": "04.2",
    "file": "tests/the_planning_files_agree_with_themselves.rs",
    "line": null,
    "description": "The roadmap check reads one direction only: every progress-table row must match the phase directory it names. A phase directory holding plans with no row in that table at all is not seen, so the roadmap can go quiet about a phase rather than wrong about it and nothing says so. Closing it means deciding what a directory with no row means, which is not always drift: a scratch directory, or a phase split after the roadmap was written, would both fire.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T21:40:00.000Z",
    "resolved_at": null
  },
  {
    "id": 176,
    "kind": "deviation",
    "phase": "04.2",
    "file": ".planning/STATE.md",
    "line": null,
    "description": "progress.total_plans says 87 and there are 85 *-PLAN.md files. It is deliberately not asserted because what it counts could not be established. The roadmap's progress denominators sum to 85, deriveProgressFromRoadmap in the vendored tooling sums exactly those denominators, and a second writer in state-transition.cjs sets the field to whatever an argument passes it. The 87 arrived at b998dfaf on 2026-09-07, in the commit whose message records state.advance-plan being run by accident, over a roadmap whose denominators already summed to 85. So the observed value matches neither reading and the field has at least two writers with different meanings.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T21:40:00.000Z",
    "resolved_at": null
  },
  {
    "id": 177,
    "kind": "deviation",
    "phase": "04.2",
    "file": "tests/the_planning_files_agree_with_themselves.rs",
    "line": null,
    "description": "The ledger check compares two of the ten columns each entry carries twice: status and description. A table row whose phase, kind, file, line, reason or either timestamp was edited alone still reverts silently on the next tool write and nothing says so. Two columns were chosen because they are what a person reads and what a repair changes, and the check was measured against the real defect of closing an entry in the table only. Widening it is cheap and was not done, so the limit is written down rather than assumed away.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T21:40:00.000Z",
    "resolved_at": null
  },
  {
    "id": 178,
    "kind": "deviation",
    "phase": "04.2",
    "file": "tests/the_planning_files_agree_with_themselves.rs",
    "line": null,
    "description": "The state check holds four facts. current_phase_name, status, stopped_at and state_head are also written in the frontmatter and described in the Current Position prose, and no check pairs them. stopped_at is the one that already went wrong: at c6e08a9 it said 04.2-05 while the heading described plan 7, and only the current_plan half of that divergence is now held. It is not obvious how to pair a prose sentence with a field, which is why the four that are held are the four that are written as numbers or as one token.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-07T21:40:00.000Z",
    "resolved_at": null
  },
  {
    "id": 179,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "move_or_copy_message and spawn_folder_move each call owner_of, and the modal dialog runs between the two calls. Both now ask one rule so they agree in every ordinary case, but a sync that replaces the message list while the window is open would leave the second call falling back to the account on screen, which is the divergence this plan closed everywhere else. Passing the first answer to spawn_folder_move would close it and would take that function out of the list tests/wired.rs holds to asking owner_of, and that check exists because this fault has now happened four times, so the narrower window is recorded rather than traded for the wider check",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T00:34:33.068Z",
    "resolved_at": null
  },
  {
    "id": 180,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/presentation/wx_destination.rs",
    "line": null,
    "description": "The move and copy window now draws a real tree, with a folder inside the folder it is in, and names accounts the way the sidebar does. No screen reader has heard either. Whether Right and Left expand and collapse as somebody expects in this dialog, whether a folder two deep is reached and read as being inside its parent, and whether hearing Work with an address after it on a branch is a help or a mouthful, are all judgements only NVDA or Narrator settles",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T00:34:48.878Z",
    "resolved_at": null
  },
  {
    "id": 181,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/data/config.rs",
    "line": null,
    "description": "The census asking whether every stored setting is read by something now follows one hop through a function of config.rs, because a setting whose stored value holds two facts gets a reader and a writer and stops being named anywhere else. Three limits, recorded rather than narrowed away. It counts a pub fn taking and self whose doc comment merely mentions the field, since the body is taken as everything between one pub fn and the next. It cannot see a setting read by a free function rather than a method. And it counts no function taking mut self, which is right for the pair that prompted it and would be wrong for a setting legitimately read inside a method that also writes",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T00:34:49.418Z",
    "resolved_at": null
  },
  {
    "id": 182,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nothing tests that the move window opens on the folder last filed into. The guard record covering that call reddens the settings census rather than anything about the window, so what is defended is that the stored value is read at all and not that the row it names is where the cursor lands. Reaching that needs a live window, a stored settings file and a branch with the remembered folder in it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T00:34:49.971Z",
    "resolved_at": null
  },
  {
    "id": 183,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/application/mail_across_accounts.rs",
    "line": null,
    "description": "No message has been copied between two real accounts. Every assertion here is against a loopback server this project wrote, which answers exactly what the script says and nothing a provider does on top: whether Gmail treats an APPEND with an internal date the way the RFC says, whether a strict server refuses a flag list this drops keywords from anyway, whether a message fetched with BODY.PEEK and appended somewhere else arrives byte for byte, and whether a slow append times out before it lands are all questions only a live account answers",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T02:10:00.000Z",
    "resolved_at": null
  },
  {
    "id": 184,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/presentation/wx_destination.rs",
    "line": null,
    "description": "No screen reader has heard the move and copy window with several accounts in it. Three questions: whether an account row reads as an account rather than as a folder, whether a collapsed branch is announced as collapsed with a count of what is inside, and whether the person finds Right without being told. The accessible description names Right and Left, which is structure present rather than experience good",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T02:10:00.000Z",
    "resolved_at": null
  },
  {
    "id": 185,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/presentation/wx_destination.rs",
    "line": null,
    "description": "Whether Enter chooses in this window is unverified and always has been. A TreeCtrl takes Enter as an item activation, and whether that reaches the dialog's default button was read off the code rather than pressed. The accessible description and docs/KEYBOARD_SHORTCUTS.md both say Enter chooses. CLAUDE.md already records one key bound in the composer that was measured never arriving, and no reader of source text can tell that case from a key that works",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T02:10:00.000Z",
    "resolved_at": null
  },
  {
    "id": 186,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/application/mail_across_accounts.rs",
    "line": null,
    "description": "The transcript half of test_a_copy_across_accounts_says_nothing_to_the_source_that_changes_it cannot be made to fail. What it asserts, that no STORE, EXPUNGE or COPY reaches the source, is guaranteed by the trait the source is behind rather than by the code under test: TheAccountItIsIn has two reads and no write, so no body of copy_it_across can send one. That is a stronger guarantee than the test, and it means the test only starts measuring anything if somebody widens the trait. Its other assertion, that the copy succeeded, was taken red",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T02:10:00.000Z",
    "resolved_at": null
  },
  {
    "id": 187,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/application/mail_across_accounts.rs",
    "line": null,
    "description": "No message has been moved between two real accounts. Every assertion is against loopback servers this project wrote, and three questions only a live account answers are named in docs/changelog.md under known limitations: what a real provider does with an upload of a ten megabyte message, what Gmail makes of a message uploaded from another account when it treats a copy as a label, and what any provider does when a message arrives carrying an identifier the destination already holds. The last is not idle, because that identifier is what the program asks about when an upload's answer never arrives. Since 2026-09-19 (11-07.2) the crossing completes here first and is replayed from the queue at a check of either account, and the replay leans on the third question the same way: it reads an arrival only from a number the folder did not hold before, and a real provider's answer is still unmeasured",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T12:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 188,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/application/mail_across_accounts.rs",
    "line": null,
    "description": "The ambiguous append, where the destination stops answering part way through, has never happened against a real server. Both tests that drive it script the answer through a hand-written destination rather than a loopback one, because the only way to make a real connection stop answering is to close it and a closed connection cannot then be asked what the folder holds. What is proved is what the code does with an Error::Network; what is unproved is that a real dropped connection and a real timeout arrive as one",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T12:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 189,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A folder somebody has turned syncing off for is still offered by the move and copy picker, and a message moved into it is at the server and never appears in this program. The decision taken is that the move must not turn syncing on, because a setting somebody chose is not something another command changes behind them, and that the fact is said in docs/changelog.md instead. Saying it in the window at the moment of the move would be better and is not done: it needs the destination account's folder_choices read at move time, and the plan's acceptance criteria forbid adding a test to wx_app.rs, which is where that check would live",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T12:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 190,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "ImapSession::remove_these is refused in the words \"replace a saved draft\", which is wrong for every caller but the one it was written for. The delete handler already worked around it rather than reuse it, and the cross-account move now does the same by way of a new take_this_one_off whose gate says \"move a message\". Three callers now avoid one function because of its refusal wording; the wording itself is still not fixed, and fixing it means deciding what a neutral sentence costs the draft path that has the specific one today",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T12:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 191,
    "kind": "unrun-verify",
    "phase": "04.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "This program has never been killed part way through a move between two real accounts and started again, so the one case the kept bytes exist for has never happened outside a test. Nor has the question put on the next start ever been heard: whether it is read out at all, whether both answers are reachable by keyboard, whether focus lands sensibly, and whether it reads as something unfinished rather than as an error, when nothing was lost, are all questions only NVDA can answer. The seven steps are written out in 04.1-04-SUMMARY.md, and two of their outcomes stop the phase rather than continuing it: nothing said at all on the next start, and the message arriving twice at the destination",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T20:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 192,
    "kind": "deviation",
    "phase": "04.1",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The question about an unfinished move is a wxWidgets MessageDialog, which the binding builds from a title and a body and which offers no seam for set_accessible_name_and_description. What a screen reader reads for it is its title and its text, both written here, and this is the house pattern that ask_about_the_folders_that_have_gone already follows. The words are also announced through the announcement queue at high priority, so somebody whose reader was mid-sentence still hears them; whether that reads as the same thing said twice is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T20:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 193,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/pim_rows.rs",
    "line": null,
    "description": "The clause \"changed just for this day\" has never been heard. Three questions only a screen reader run answers, and the words were chosen against reasoning rather than against a listener. Whether it reads as useful or as clutter when fifty-two rows of one series go past and one of them carries it. Whether it is confused with the unreadable-rule sentence when both could apply to the same row, since both are about the same series and both arrive in the same breath. And whether the joined time cell, which can now carry the date, the out-of-hours note and this clause at once, is still one thing somebody can take in while arrowing rather than three",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T21:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 194,
    "kind": "unmet-truth",
    "phase": "05",
    "file": "src/application/calendar.rs",
    "line": null,
    "description": "Whether an override row arriving from Google or Outlook carries a repeat rule of its own is still unknown. If it did, that row would expand across the whole window and appear on every date, which is worse than the double-show this plan measured. 05-01 was written to settle it by asserting the stored row's recurrence_rule is None after a local write, and that assertion cannot fail: the fixture writes the constant itself and save_calendar_event only round-trips it. Both provider paths derive the rule from the payload, at google_event_to_local and ms_event_to_local, so no local round trip can reach either. Settling it needs a fixture built from a real provider payload carrying an override with a RRULE, or a live account",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T21:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 195,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_calendar_module.rs",
    "line": null,
    "description": "Nothing about the week view has been heard. Four questions only a screen reader run answers. Whether the heading, which is a StaticText the announcement queue also speaks on the calendar-period topic, is heard once or twice when Previous period is pressed. Whether Week of 20 July 2026 is the right amount of words to hear on every press, or whether it should shorten after the first. Whether the View box in the toolbar is reached and understood before somebody starts arrowing the list, since it is the fourth control on a row that starts with three buttons. And whether the announcement topic really suppresses a burst: five quick presses of Next should read the fifth week and no other, and the topic is the mechanism but nobody has heard it happen",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T17:35:12.582Z",
    "resolved_at": null
  },
  {
    "id": 196,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_calendar_module.rs",
    "line": null,
    "description": "Previous period and Next period carry the same name on both Windows accessibility channels by construction, and neither channel has been read. The button label is Previous period and Next period, which is what Windows gives UI Automation for a native button and therefore what Narrator says, and set_accessible_name writes the same words to MSAA, which is what NVDA says. Both were checked by reading the code, not by running Axe.Windows over UI Automation or scripts/msaa-names.ps1 over MSAA. The View box beside them is a Choice with a StaticText label of its own and an explicit accessible name of Calendar view, and is unread on both channels for the same reason",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T17:35:22.872Z",
    "resolved_at": null
  },
  {
    "id": 197,
    "kind": "deviation",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The reload after an edit now asks for the window on screen rather than the whole eighteen months, so saving an event in a week view no longer puts somebody back in the agenda with nothing said. Nothing tests that it does. The window arithmetic it calls has eleven tests, and the wiring, one call to the_calendar_on_screen inside manage_calendar, has none: manage_calendar opens a wxWidgets dialog and needs a Frame, so a test would first have to extract the reload into a function of its own, and any test of it would live in managers.rs, which 41 guard records fingerprint, at roughly 80 minutes of remeasurement on the critical path for a one-line change. The plan offered fix-with-a-test or record-as-a-stub; this is the third thing, fixed without one, and it is here so that the gap is visible rather than implied by the absence of a test",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T17:35:23.552Z",
    "resolved_at": null
  },
  {
    "id": 198,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nobody has chosen a view, closed this program and opened it again. The three pieces are each tested where they live: the settings screen writes the word, AppConfig round-trips it, and CalendarView::from_stored reads it back with a fallback for anything it does not recognise. What joins them is one line at startup that puts the stored view into WxUIState and one that sets the toolbar box to match, and neither has a test: they sit inside the frame builder, which needs a running wxWidgets window. So the claim that the view survives a restart rests on reading three tested pieces and one untested join. The same join decides what the box on the toolbar says when the window opens, which is what a screen reader reads for it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T19:25:35.807Z",
    "resolved_at": null
  },
  {
    "id": 199,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_calendar_module.rs",
    "line": null,
    "description": "PIM-06's third D line cannot be closed by a test and is not closed here. It asks whether a screen reader user can work through a view's events in date order without reconstructing the grid from cell labels. What this plan does is remove the grid from the question by not building one: a month is the same virtual list the agenda uses, sorted by moment, so read from the top it is in date order by construction. That is the design decision which makes the answer likely, not evidence that it is right. Two questions only a real run settles. Whether a month of rows in one flat list, which is thirty to sixty rows for an ordinary calendar and several hundred for a busy one with series in it, is navigable as one list or wants sub-headings per day. And whether somebody can tell where one day ends and the next begins, given that the date is in the same joined time cell as the out-of-hours note and the changed-day clause 05-01 added",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T19:40:20.942Z",
    "resolved_at": null
  },
  {
    "id": 200,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/pim_command.rs",
    "line": null,
    "description": "Nobody has heard a copy and a move told apart. filed() says \"Buy milk copied to Shopping\" and \"Buy milk moved to Shopping\", which differ by one word in the middle of a sentence, and the refusal for a read-only destination differs the same way. Somebody who cannot see the two lists has only that word, and it arrives at whatever rate their screen reader is set to. Whether it is caught at speed, and whether the two sentences want more distance between them than one participle, is the question T-05-10 turns on and it is unanswered",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T22:38:57.586Z",
    "resolved_at": null
  },
  {
    "id": 201,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_destination.rs",
    "line": null,
    "description": "The chooser for a copy now offers the container the item is already in, which is the one destination a copy has that a move does not, and nobody has heard it. Somebody arrowing a tree of calendars meets the one they are standing in with nothing distinguishing it from the others. Whether that reads as a duplicate they can deliberately make, or as a window that forgot to take out the row a move would have, is unanswerable without a real run. The window says which act it is in its title and on its button, which is the whole of what tells them apart",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T22:39:07.613Z",
    "resolved_at": null
  },
  {
    "id": 202,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "No copy has been carried out by pressing a key in the running program. Every piece is tested where it lives: file_under writes the copy against a real store, Filing answers the three differences, and the sentences are held to their words. What joins them is the arm in wx_app.rs that turns Ctrl+Shift+Y or the context menu line into PimCommand::Copy, and the only thing that reads it is tests/wired.rs, which reads the source text of the arm rather than running it. So the claim that the key copies rests on tested pieces and a read join. The same arm decides which of the six modules answers, and a wrong answer there offers mail folders as a home for a task",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T22:39:17.913Z",
    "resolved_at": null
  },
  {
    "id": 203,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "No copied item has ever reached a provider. The copy is written as an item made on this computer waiting to be sent, and a_provider_holds answers false about it, which is asserted against a real store. What happens next is a push, and no push in this program has run against a real Google or Microsoft account. So whether a copied task is created at Google as a second task, rather than rejected or silently reconciled against the original, is untested and untestable here. It is the whole of T-05-09's mitigation past the local write",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T22:39:31.628Z",
    "resolved_at": null
  },
  {
    "id": 204,
    "kind": "deviation",
    "phase": "05",
    "file": "guards/guards.toml",
    "line": null,
    "description": "Two of the three answers that make a copy not a move have tests and no guard record. makes_a_new_row has two records, one on the decision and one on the write. leaves_out_where_it_is and needs_the_holder_told have neither, so nothing checks that the tests covering them would still notice if they stopped. Both are covered by a test in tests/a_copy_leaves_the_original_where_it_was.rs that has been red once, in the RED commit, which is more than an unmeasured guard has; what is missing is the recorded break that would catch the test going quiet later. Two more records is two more hand measurements, each a build and a full run",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-08T22:39:32.251Z",
    "resolved_at": null
  },
  {
    "id": 205,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "Nobody has been asked two questions for one move. A contact's move asks which group it is leaving and then which it is joining, where every other kind is asked once, and both windows are the same SingleChoiceDialog holding group names and counts. Whether that reads as a program being thorough or as one that cannot make up its mind is the question, and it can only be answered by somebody working down it by keyboard through a screen reader. The first question is skipped when there is only one answer, so the common case may be one question and the uncommon case two, which is its own kind of surprise",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:15.522Z",
    "resolved_at": null
  },
  {
    "id": 206,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/contact_groups.rs",
    "line": null,
    "description": "The sentence a completed move says has never been heard. moved_between names two groups and two counts: \"Ada Lovelace moved out of Team A, 2 people, and into Team B, 5 people.\" That is the longest status line this program says about a single act, and it is said at whatever rate somebody's screen reader is set to. It deliberately leaves out taken_out's reassurance that the contact is still in the address book, on the grounds that naming a group the contact is now in shows that without saying it. Whether the sentence is heard to the end, and whether the missing reassurance is missed, is unanswered",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:29.020Z",
    "resolved_at": null
  },
  {
    "id": 207,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The two choosers a contact's move opens have never been told apart by ear. Both are a list of group names with the count of each, one after the other, and the only thing distinguishing them is the question and the window title: \"Which group should this contact come out of?\" in a window called Move out of a group, then \"Which group should this contact go in?\" in one called Move into a group. Somebody who missed the first word of either is looking at two lists that read identically. Whether the titles are announced at all, and whether come out of and go in are enough distance between two questions asked seconds apart, is unanswered",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:29.652Z",
    "resolved_at": null
  },
  {
    "id": 208,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "No contact has been moved between groups by pressing a key in the running program. Every piece is tested where it lives: the transactional write against a real store, the two filters and the five sentences as pure functions, and the menus against the command. What joins them is the arm in managers::pim_command that sends a contact to the group path before the chooser that names one container, and the only thing that reads it is a source-text check in tests/a_contact_moved_between_groups.rs. So the claim that Ctrl+Shift+V moves a contact rests on tested pieces and a read join. The same shape as ledger 202, one plan later",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:46.288Z",
    "resolved_at": null
  },
  {
    "id": 209,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/context_menu.rs",
    "line": null,
    "description": "The contact context menu's Put in a group line now raises the copy command rather than an action of its own, and nobody has met the menu since. Two things are unheard. Somebody who learned Ctrl+Shift+Y as Copy in the calendar, Tasks or Notes meets a line called Put in a group here and there is nothing in the wording to connect them; the reason for keeping the old wording is that it describes what happens to a person and Copy does not, and whether that trade is right is a judgement about hearing it. And the menu now reads New contact, Move to another group, Put in a group, Take out of a group, Delete, met in that order by somebody who cannot skim",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:46.872Z",
    "resolved_at": null
  },
  {
    "id": 210,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/contact_groups.rs",
    "line": null,
    "description": "The two refusals before either question have never been heard. in_no_group says the contact is in no group and to use Put in a group first; in_every_group says it is in every group and to make another group first. Both go down the refusal channel, both name a command by the words the menu uses, and both are the answer to a key somebody just pressed with no window opening. Whether naming a menu line inside a spoken sentence is followed, or whether somebody hears a sentence about a command and looks for a dialog, is unanswered",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T01:23:47.493Z",
    "resolved_at": null
  },
  {
    "id": 211,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The account chooser is a flat list where every other move in this program opens a tree, and nobody has met it. Move and Copy on an event, a task, a note or a message open build_destination_dialog, which draws accounts as branches with places under them; a reminder opens pick_one, a SingleChoiceDialog holding account names in a row. The reason is structural rather than a preference: an account row in that tree pushes None into its destinations vector on purpose, so the tree cannot answer an account at all. Whether somebody who has learned Ctrl+Shift+V as the key that opens a tree hears a flat list and reads it as a different command, or as the same one asking a simpler question, is a real question and nobody has heard either",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T04:40:20.115Z",
    "resolved_at": null
  },
  {
    "id": 212,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The name a completed reminder move says has never been heard. It is the account's label, with its address after it only where two accounts read alike, which is the rule so_no_two_accounts_read_alike already applies to the sidebar and which where_mail_can_go already takes. The plan asked for the address on every move, quoting Branch.account_name's doc comment; the sidebar rule was taken instead because an address read aloud on every move costs something for a case that needs it rarely. Whether Ring the dentist moved to Work is enough for somebody with two accounts, or whether the address is wanted every time even at that cost, is a judgement about hearing it and nobody has",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T04:40:34.804Z",
    "resolved_at": null
  },
  {
    "id": 213,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/pim_command.rs",
    "line": null,
    "description": "The sentence somebody with one account meets has never been heard, and neither has how often they meet it. the_only_account_there_is says the reminder is in the one account set up on this computer, that there is nowhere else to move it to, that nothing has been moved, and that setting up a second account gives a reminder somewhere to go. It is said before any window opens, which is the right shape. What is unknown is whether a person who has one account and no intention of adding another hears it as information the first time and as nagging every time after, since Ctrl+Shift+V is one key away from Ctrl+Shift+K on the same list. Nothing throttles it and nothing remembers that they have heard it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T04:40:35.448Z",
    "resolved_at": null
  },
  {
    "id": 214,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/pim_command.rs",
    "line": null,
    "description": "The clause a filing now says has never been heard. A move or copy into a container an account holds ends 'and has not reached the account yet', or names Allow Changes where that setting is off, and both are joined onto the sentence naming where the item went rather than being a second sentence. Whether one sentence carrying both facts is heard as one answer or as a run-on, and whether the clause reads as useful or as noise after the twentieth move, is a judgement about hearing it. Guardrail 5 is the risk: this is said after every single filing somebody makes",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T06:50:00.150Z",
    "resolved_at": null
  },
  {
    "id": 215,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "tests/wired.rs",
    "line": null,
    "description": "Whether Ctrl+Shift+V really reaches the filing handler in the non-mail modules, rather than only appearing in a menu label, is assumption A2 of 05-RESEARCH.md and nothing in this repository can answer it. tests/wired.rs says in its own header that a bound key proves Windows will dispatch it and says nothing about what the handler then does with the right thing on screen. Recorded rather than left to a green wiring test to look like an answer",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T06:50:12.578Z",
    "resolved_at": null
  },
  {
    "id": 216,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "The sentence 'and has not reached the account yet' has never been followed by an account receiving anything, because no build has run against a real account. What the clause promises is that the next sync sends it, and that the summary then says so; both halves are tested against scripted providers only. If a push fails for a reason the sync counts rather than reports, somebody hears the move was waiting and never hears that it stopped waiting",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T06:50:13.358Z",
    "resolved_at": null
  },
  {
    "id": 217,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "The two counts a task sync now reads out have never been heard side by side. A deletion held by an outstanding create is said as a count, '1 removal waiting for the new copy to be sent', and a deletion held by Allow Changes is said as a sentence naming the setting. Both are counts of something that did not happen, and only one of them is something the person can act on. Whether somebody hearing both in one status line can tell which is which, or hears two numbers about waiting and goes looking for one setting that fixes both, is a judgement about hearing it and nobody has",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T09:04:16.858Z",
    "resolved_at": null
  },
  {
    "id": 218,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "Whether a provider goes on answering for a task under its old identifier long enough for the pull that follows a move to write it back down is a timing question no fake can answer. The push sends the create, the pull in the same sync reads every list, and the deletion of the old copy does not go until the sync after that. Between them the provider holds both copies and a read that sees the old one is answered only by the deletion note masking it. That masking is tested; how long the window really is at Google and at Microsoft is not, and cannot be without an account",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T09:04:31.423Z",
    "resolved_at": null
  },
  {
    "id": 219,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/deletions.rs",
    "line": null,
    "description": "Whether seven days is long enough for a move made just before a laptop is shut for a fortnight has never been tried. The note itself is safe: let_go_of_deletions_taken_before only releases a deletion a provider has taken, so one still waiting for its new copy survives however long it waits, which was read on main at 143a37f rather than assumed. What is released by the clock is the memory of a deletion already taken, and the read consults that memory to stop a provider writing the thing back down. A move that completes on day one and a machine that comes back on day fifteen is the case nobody has run",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T09:04:32.089Z",
    "resolved_at": null
  },
  {
    "id": 220,
    "kind": "todo",
    "phase": "05",
    "file": "src/data/message_cache/tasks.rs",
    "line": null,
    "description": "rename_task orphans a task's subtasks and nobody decided that it should. It calls drop_synced_task on the old identifier, which sets parent_task_id to null for every child, so a task made on this computer loses its subtask tree the moment a provider names it. That is older than this plan and nothing in this plan reaches it. move_a_task_the_provider_holds deliberately answers the same question the other way, pointing the children at the new identifier, because the parent has not gone but been renamed, and copying the null would have flattened a tree on every move. The two now disagree on purpose and one of them is wrong",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T09:04:46.509Z",
    "resolved_at": null
  },
  {
    "id": 221,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "tests/a_half_finished_task_move.rs",
    "line": null,
    "description": "The failure this whole state exists for has never happened. A half-finished move is the create succeeding at the provider and the delete then failing, or the reverse, and nothing in this repository can produce either: no provider is called by this plan at all, and 05-08 makes the calls against fakes that answer from a script. Every test here builds the half-finished state by calling the write that produces it, which proves the state is held, survives a close and cannot exist half-written, and proves nothing about whether a real pair of calls leaves exactly that state",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T09:04:47.221Z",
    "resolved_at": null
  },
  {
    "id": 222,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "Whether a real provider accepts a create into a second list while the first still holds the task, which is the state a provider-held task move passes through on purpose",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:20.424Z",
    "resolved_at": null
  },
  {
    "id": 223,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/deletions.rs",
    "line": null,
    "description": "Whether a real provider's list still names the old copy on the pull that follows the delete, and for how long. The seven-day memory in application::deletions is sized against a number nobody has measured",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:34.744Z",
    "resolved_at": null
  },
  {
    "id": 224,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "Whether a person hearing the move sentence and then the sync summary can tell a change waiting on a create from a change waiting on Allow Changes. The two are counted apart and only one names a remedy, and nobody has heard them side by side",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:35.427Z",
    "resolved_at": null
  },
  {
    "id": 225,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/service/tasks_api.rs",
    "line": null,
    "description": "Whether Microsoft's notStarted, inProgress, waitingOnOthers and deferred survive a move. The new copy is created rather than updated and ms_task_to_entry rebuilds the row from Graph's answer, so remote_status is carried by the row up to the create and by Graph after it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:36.063Z",
    "resolved_at": null
  },
  {
    "id": 226,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "Whether the identifier a provider hands back for the created copy is accepted by the same provider's delete for the old one, on an account signed in to both providers at once. Both passes read the same notes against the same account",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:36.731Z",
    "resolved_at": null
  },
  {
    "id": 227,
    "kind": "deviation",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "A task a provider holds moved into a list made on this computer: the create is counted local_only and never sent, so the deletion note waits for ever. Nothing is lost and the person can put it right by moving it into a synced list, but nothing tells them that. Tested and reported, not refused",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:51.492Z",
    "resolved_at": null
  },
  {
    "id": 228,
    "kind": "deviation",
    "phase": "05",
    "file": "src/service/tasks_api.rs",
    "line": null,
    "description": "google_task_to_entry sets created_at to empty, so a moved task loses when it was made once Google names the new copy. Pre-existing for every task created here and synced; a provider move now traverses it too",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:52.197Z",
    "resolved_at": null
  },
  {
    "id": 229,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/application/tasks_sync.rs",
    "line": null,
    "description": "No test asserts that a task's fields survive the round trip through a provider's answer to the create. The Scripted fake returns a bare task, so the title comes back as Untitled task; a real provider echoes the body. Field survival is asserted on the local write only",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:52.910Z",
    "resolved_at": null
  },
  {
    "id": 230,
    "kind": "unrun-verify",
    "phase": "05",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "Nobody has heard what a move of a provider-held task says. The clause is 05-06's and unchanged, and whether it carries the fact that the provider has not been told yet without wearing after twenty moves is a judgement about hearing it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-09T21:16:53.634Z",
    "resolved_at": null
  },
  {
    "id": 231,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/presentation/read_aloud.rs",
    "line": null,
    "description": "Nobody has heard a note whose Markdown is read back as structure. Whether \"heading level 1, Shopping, bullet, milk\" is clearer to listen to than the flat text it replaced is the whole argument for storing Markdown, and it has never been put to a screen reader",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T14:50:56.612Z",
    "resolved_at": null
  },
  {
    "id": 232,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/data/message_cache/notes.rs",
    "line": null,
    "description": "No database written by another build has ever been opened. NoteBody::Other and the null-column path are driven only by rows this repository's own tests wrote with raw SQL, so what a real second writer puts in that column is a guess",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T14:51:05.860Z",
    "resolved_at": null
  },
  {
    "id": 233,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Nobody has heard the Notes section on the Calendar and PIM tab. Whether it is found where it was put, last on the tab after Working Day, by somebody moving through the sections in order with a screen reader, has never been tried",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T17:40:28.476Z",
    "resolved_at": null
  },
  {
    "id": 234,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "Nobody has heard the sentence saying an account has no notes backend. Whether it is heard as an answer or as an apology is the whole question about wording it that way, and it has never been put to a screen reader",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T17:40:42.877Z",
    "resolved_at": null
  },
  {
    "id": 235,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/context_menu.rs",
    "line": null,
    "description": "The note folder menu has never been opened with a backend behind it, because none exists. Every account answers that its notes stay here, so the arm that offers Sync notes now is driven only by tests",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-10T17:40:43.531Z",
    "resolved_at": "2026-09-10T20:50:53.826Z"
  },
  {
    "id": 236,
    "kind": "stub",
    "phase": "05.1",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "NotesService has no implementor and no caller. It is the contract 05.1-03 and 05.1-04 are held to, and until one of them lands nothing has ever executed a line of it",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-10T17:40:44.182Z",
    "resolved_at": "2026-09-10T20:50:53.090Z"
  },
  {
    "id": 237,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "docs/development/the-notes-seam.md",
    "line": null,
    "description": "The notes seam contract is reasoning from Microsoft's documentation and from what this repository already does, not from a backend that has run. Its own table names one assumption it knows is weakest, that one note maps to one thing at the backend, and 05.2-03 is required to report where it was wrong",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T17:40:44.830Z",
    "resolved_at": null
  },
  {
    "id": 238,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/caldav_journal.rs",
    "line": null,
    "description": "Whether a real calendar server accepts the journal document this writes. The whole write path is untried: if a server refuses it, every note push fails and nothing anybody has run would have said so.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:49:58.456Z",
    "resolved_at": null
  },
  {
    "id": 239,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/caldav.rs",
    "line": null,
    "description": "Whether a real server's own listing names a journal entry the way this reads it. The listing is a PROPFIND at one level down and the collection's own block is skipped by hand; a server that answers in another shape reports an empty container, which reads as a clean sync over somebody's missing notes.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:19.482Z",
    "resolved_at": null
  },
  {
    "id": 240,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether a real server's version marker survives a round trip. Everything about when a note is taken down rests on comparing the marker for equality, so a server that changes it on every read makes every note arrive on every sync and one that never changes it makes none arrive at all.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:20.164Z",
    "resolved_at": null
  },
  {
    "id": 241,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether a real server takes a deletion. The record of a deletion is kept until the server has taken it and for a week after, so a server that refuses the removal leaves the note owed for ever and one that answers in a way this misreads lets the note come back.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:20.825Z",
    "resolved_at": null
  },
  {
    "id": 242,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/caldav_journal.rs",
    "line": null,
    "description": "Whether a real server raises a clash the way the stand-in does. The disagreement is found by reading the document's version before writing rather than by a 412, so a server that gives no version marker at all would never report one and a change made in two places would be written over in silence.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:21.497Z",
    "resolved_at": null
  },
  {
    "id": 243,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "Whether the notes sync summary is distinguishable by ear from the calendar, task and contact summaries when several run together. Nobody has heard any of them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:22.241Z",
    "resolved_at": null
  },
  {
    "id": 244,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/conflict_choice.rs",
    "line": null,
    "description": "Whether a held note conflict read aloud is answerable without seeing both copies. The words say note rather than contact now, and nobody has heard the question.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:22.983Z",
    "resolved_at": null
  },
  {
    "id": 245,
    "kind": "stub",
    "phase": "05.1",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "An account with two calendar servers sends its notes to the first one the store answers with. That is a limit rather than a decision: nothing asks the person which, and nothing says which was chosen.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:23.697Z",
    "resolved_at": "2026-09-11T18:01:03.284Z"
  },
  {
    "id": 246,
    "kind": "stub",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Note folders on this computer are not mirrored at the server. Every note arriving from a server is filed in the account's first note folder, so folders somebody made here mean nothing at the other end.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-10T20:50:24.407Z",
    "resolved_at": "2026-09-11T18:01:04.034Z"
  },
  {
    "id": 247,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether the sentence somebody hears when the read holds a change the setting had refused is understood by ear, and whether it is told apart from the clash the push reports. Both reach conflict_choice and both say a note is waiting to be chosen; nobody has heard either.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:07.431Z",
    "resolved_at": null
  },
  {
    "id": 248,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether the sentence 1 note could not be kept exactly by your notes backend is understood by ear, and whether it is distinguishable from the other four sentences a notes sync can say. It is a new sentence in a status line that already carries four.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:08.090Z",
    "resolved_at": null
  },
  {
    "id": 249,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "tests/the_notes_seam_takes_a_second_kind_of_backend.rs",
    "line": null,
    "description": "Whether the four constraints the second implementation is shaped from are what a real Graph OneNote client meets. They are a reading of three Microsoft pages on 2026-09-06, not a measurement of the service, and a documented API can differ from the service behind it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:48.158Z",
    "resolved_at": null
  },
  {
    "id": 250,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "Whether a seam proven against an implementation in this repository's own tests can be implemented from a separate crate. An integration test sees only what is pub, which is the stronger placement, but it is still built against the same source tree in the same commit.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:48.816Z",
    "resolved_at": null
  },
  {
    "id": 251,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "docs/development/the-notes-seam.md",
    "line": null,
    "description": "How wide a OneNote lastModifiedDateTime tick really is, and so how likely it is that a write this program makes and an edit somebody makes at the service carry one marker. Requirement 4 of the seam's contract turns on it and nothing here can measure it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:49.479Z",
    "resolved_at": null
  },
  {
    "id": 252,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether replacing the copy here with what the backend could keep is what somebody wants. The alternative is keeping their bytes and telling them the two copies differ; this was decided on the argument that a loss somebody watches happen is better than one that arrives weeks later, and nobody has been asked.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:50.138Z",
    "resolved_at": null
  },
  {
    "id": 253,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "Whether making a note again when the backend says it no longer holds it is right where somebody deleted it at the other end on purpose. The seam's contract says a name the backend never gave is a note to create; a deletion made at the backend is not propagated here at all, so the two rules can disagree about one note.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-10T23:33:50.847Z",
    "resolved_at": null
  },
  {
    "id": 254,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/carddav.rs",
    "line": null,
    "description": "Whether a real CardDAV server's answer about its address books parses. Every fixture was written in this repository, and the reader assumes the d: prefix on the DAV elements and that the change marker is cs:getctag. A server spelling either differently is read as offering none.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:41.360Z",
    "resolved_at": null
  },
  {
    "id": 255,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/carddav.rs",
    "line": null,
    "description": "Whether a real CardDAV server's answer with cards in it parses. The card's own element is tried under three spellings, card:address-data, C:address-data and address-data, and a server writing a fourth is read as having sent no card at all.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:45.676Z",
    "resolved_at": null
  },
  {
    "id": 256,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/data/message_cache/contacts.rs",
    "line": null,
    "description": "Whether a card vcard_block_from_contact writes is accepted by a real CardDAV server. A round trip in this repository proves this writer and this reader agree with each other and says nothing about anybody else's server.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:46.427Z",
    "resolved_at": null
  },
  {
    "id": 257,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/data/message_cache/contacts.rs",
    "line": null,
    "description": "Whether a card written by another client reads correctly through contact_from_vcard_block. Only cards this repository wrote have been read back by it here, and Apple, Google and SabreDAV each write cards this program has never seen.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:47.161Z",
    "resolved_at": null
  },
  {
    "id": 258,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/carddav.rs",
    "line": null,
    "description": "Whether the scan for the address book element over a whole response block, rather than over the resource type alone, ever reads a server's answer as offering an address book it does not have. It needs a server sending a raw angle bracket inside a name it should have escaped.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:58.844Z",
    "resolved_at": null
  },
  {
    "id": 259,
    "kind": "deviation",
    "phase": "05.1",
    "file": "docs/ALPHA_TESTING.md",
    "line": null,
    "description": "A malicious card joining two people who share an address is a named limitation, and CardDAV raises its severity: today it needs somebody to choose a file, and over a server it arrives from the network. This plan does not change ContactEntry::shares_an_address_with or the_same_person_on_an_earlier_card, so the change of severity is recorded rather than answered.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T01:41:59.594Z",
    "resolved_at": null
  },
  {
    "id": 260,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/carddav_sync.rs",
    "line": null,
    "description": "Whether a real CardDAV server accepts a card this writes with If-Match, and answers 412 when the version has moved. The sync treats 412 as the copy having moved past the change and keeps the edit for the next sync; a server answering 409 or 403 instead is read as an ordinary failure and the edit is still kept, but it is not counted as built on an old copy, so the read in the same sync can replace it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:04.081Z",
    "resolved_at": null
  },
  {
    "id": 261,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/carddav_sync.rs",
    "line": null,
    "description": "Whether a real CardDAV server answers a PUT to an address nothing is at by making the card. Every contact made here is written to an address this program chose, under the contact's own identifier, and a server that refuses to create at an address it did not name would fail every create with nothing here able to tell that apart from a refusal.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:04.822Z",
    "resolved_at": null
  },
  {
    "id": 262,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/carddav_sync.rs",
    "line": null,
    "description": "Whether a real server's change marker moves on every change to an address book. If it does not, a sync that compares it skips the read and nothing new is ever seen; if it moves when nothing changed, the whole address book is read every time. Both are silent.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:05.652Z",
    "resolved_at": null
  },
  {
    "id": 263,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/service/carddav.rs",
    "line": null,
    "description": "Whether a real CardDAV server gives an ETag on the PUT response at all. Where it does not, the version marker is nothing, the_marker_moved reads that as moved, and every contact is read as having changed at the server on every sync.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:06.370Z",
    "resolved_at": null
  },
  {
    "id": 264,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/application/address_book_source.rs",
    "line": null,
    "description": "Whether a real address book server's home set answers the PROPFIND this makes at Depth 1 with the address books in it. Discovery asks the address somebody typed; a server that keeps its address books one level further down answers with nothing and somebody is told the server has no address books for that sign-in.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:07.152Z",
    "resolved_at": null
  },
  {
    "id": 265,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "src/presentation/wx_add_address_book.rs",
    "line": null,
    "description": "Whether the new address book screen is usable by ear. Every control carries an accessible name set the only way that reaches NVDA, and the mnemonics were checked by hand because nothing can check them. Whether the names are the ones intended rather than a nearby label Windows fell back to, whether the refusal is heard, and whether what the server found is announced rather than only drawn, are all things only a screen reader run answers.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T04:44:07.898Z",
    "resolved_at": null
  },
  {
    "id": 266,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_page.rs",
    "line": null,
    "description": "Whether a real page's output HTML matches the model this plan built from Microsoft's reference, construct by construct. Every row of the fidelity table went through a model of the service written from a page dated 2024-11-07 and read on 2026-09-11. No OneNote tenant has ever been used with this program, so the table says what this program does with what the reference says comes back, and nothing about what comes back.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T06:50:37.976Z",
    "resolved_at": null
  },
  {
    "id": 267,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_page.rs",
    "line": null,
    "description": "Whether a data-id on a div really survives a page update. The reference says a div carrying one is preserved where a div carrying no semantic information is flattened, and the hidden source div was measured against that reading rather than against a service. If a data-id does not survive an update, the wrapping div this reader flattens may not be there at all and what comes back is a different shape.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T06:50:38.717Z",
    "resolved_at": null
  },
  {
    "id": 268,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/long_text.rs",
    "line": null,
    "description": "Whether a note whose body came back through this pair is read aloud by a screen reader the way the original was. The whole reason the structure is preserved is that a heading announces as a heading and a list as a list. Seven of twenty-two constructs survive the round trip and six of the losses are this program's own reader, so what somebody hears after a note has been to OneNote is a different passage from what they typed, and nobody has heard either.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T06:50:39.435Z",
    "resolved_at": null
  },
  {
    "id": 269,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "docs/development/the-notes-seam.md",
    "line": null,
    "description": "Whether a Markdown code block coming back as a flattened paragraph is acceptable to somebody who keeps code in their notes. Two commands on two lines come back as one line that runs neither. That is a question for a person who uses OneNote and keeps notes that way, and no test can answer it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T06:50:40.214Z",
    "resolved_at": null
  },
  {
    "id": 270,
    "kind": "unmet-truth",
    "phase": "05.2",
    "file": "",
    "line": null,
    "description": "PIM-04's structure criterion, reworded 2026-09-11 at 05.2-01's checkpoint, is not met. A nested list, a table, a link's address, a picture and a line break are lost on the way back from a backend, all five in long_text::from_markup. That function is a reader written for speaking, with two callers on that job outside notes, so it must not be changed to suit a note. A second reader whose output is stored and edited again is what the criterion asks for, and it belongs to no plan.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-11T10:10:44.221Z",
    "resolved_at": "2026-09-11T13:32:43.372Z"
  },
  {
    "id": 271,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "",
    "line": null,
    "description": "Whether a nested list and a table are pleasant to listen to, not merely correct. A nested item is announced as 'bullet level 2, ...' and only when the level changes; a table as 'table, 2 columns, 2 rows' then 'row 1. Name: Grace. Role: Admiral', with the column heading repeated on every cell. Tests prove those exact words are produced. No screen reader has said them. The open questions are whether repeating a heading per cell floods a wide table, whether 'bullet level 2' is heard as a level or as part of the text, and whether a listener can follow a table with more than three columns at all. Only an NVDA pass can settle any of them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T13:32:06.681Z",
    "resolved_at": null
  },
  {
    "id": 272,
    "kind": "deviation",
    "phase": "05.2",
    "file": "src/service/onenote_page.rs",
    "line": null,
    "description": "A picture kept in OneNote comes back pointing at OneNote's copy of it rather than at the address it went out with. The reference says a page stores the picture and hands back a resource address of its own, so the note's Markdown now names graph.microsoft.com. The picture is not lost and the address is not the one somebody typed. Whether that matters to a person whose note linked to an image they host elsewhere is a product question nobody has been asked. Measured through the model, not against a real tenant.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T13:32:22.029Z",
    "resolved_at": null
  },
  {
    "id": 273,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/long_text.rs",
    "line": null,
    "description": "A block-level img in from_markup contributes its alt text with no marker saying it is a picture, and contributes nothing at all when the sender gave no alt. So a picture in a Google task's description or a calendar event's description is read aloud as an ordinary paragraph, or vanishes. Piece::Image's own doc comment and guardrail 9 both say a picture nobody described must still be announced, and the inline arm does that correctly; the block arm does not. Found by measurement during ledger 270 and deliberately left alone as out of scope: it is pre-existing, it is on the speaking path rather than the storing one, and fixing it changes what is stored for calendar and tasks.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T13:32:22.877Z",
    "resolved_at": null
  },
  {
    "id": 274,
    "kind": "todo",
    "phase": "05.2",
    "file": "",
    "line": null,
    "description": "A container is a note folder, decided 2026-09-11 and written into docs/development/the-notes-seam.md, and not yet built. note_folders needs an opaque container column, the sync has to loop over folders rather than take an account's first calendar, a backend-given folder is named by its flattened path, a folder made here sits under the words mail already uses for folders on this computer, and a note moving between two backed folders follows the task model's create-then-remove order. Closing this closes 245 and 246.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-11T14:30:15.791Z",
    "resolved_at": "2026-09-11T18:01:02.560Z"
  },
  {
    "id": 275,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/data/message_cache/notes.rs",
    "line": null,
    "description": "A note moved between two folders a calendar server gave is created in the new collection and only then removed from the old one. Nobody has done that with a real server: whether the create really lands before the removal goes, and what the server says back, is untested.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T17:59:53.611Z",
    "resolved_at": null
  },
  {
    "id": 276,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/notes_sync.rs",
    "line": null,
    "description": "A removal held back until its copy reaches the backend goes out on the next sync. The hold is driven against a stand-in that says yes to everything; nobody has watched the two syncs run against a real calendar server.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T17:59:54.333Z",
    "resolved_at": null
  },
  {
    "id": 277,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "An account with two calendar servers now syncs both. Nobody has run it with two real servers, so whether two sign-ins in one pass both work, and what one refusing does to the other, is unknown.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T17:59:55.050Z",
    "resolved_at": null
  },
  {
    "id": 278,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/presentation/note_folder_tree.rs",
    "line": null,
    "description": "Nobody has heard the notes tree with a screen reader. Whether the On this computer branch is met as a place and whether it is clear that the folders under it go nowhere is unmeasured, and so is whether a folder named after a calendar reads as its name.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T18:00:10.261Z",
    "resolved_at": null
  },
  {
    "id": 279,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/data/message_cache/notes.rs",
    "line": null,
    "description": "Two calendars sharing a display name give a folder named Work and one named Work (2). Nobody has heard that read aloud. At a screen reader's default punctuation level the brackets are expected to be silent, so it should read Work 2, and that is an expectation rather than a measurement.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T18:00:11.003Z",
    "resolved_at": null
  },
  {
    "id": 280,
    "kind": "todo",
    "phase": "05.2",
    "file": "src/presentation/managers.rs",
    "line": null,
    "description": "A note a backend holds, moved into a folder somebody made here, leaves the backend holding its copy for ever. The removal waits for a copy that sits in a folder no push reaches, so it never goes. The task move has the same shape and docs/ALPHA_TESTING.md says so, but nothing in the program tells the person their server still has it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T18:00:11.768Z",
    "resolved_at": null
  },
  {
    "id": 281,
    "kind": "todo",
    "phase": "05.2",
    "file": "docs/development/the-notes-seam.md",
    "line": null,
    "description": "A folder name carrying a flattened path, Work / Projects / Q3, arrives only when a backend with levels above a note ships. Nothing chooses that separator today and what a screen reader makes of it is unmeasured.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T18:00:12.574Z",
    "resolved_at": null
  },
  {
    "id": 282,
    "kind": "unmet-truth",
    "phase": "05.2",
    "file": "src/service/oauth.rs",
    "line": null,
    "description": "Tasks.ReadWrite is in the outlook provider's default_scopes and absent from THE_SCOPES_A_GRAPH_TOKEN_CARRIES, the array get_valid_graph_token refreshes with, so no running Microsoft account holds it and every Graph task write is refused. 05.2-02 found it and did not fix it: it is a live defect on another feature. docs/PROVIDER_SETUP.md's 'If you signed in before tasks synced both ways' tells somebody signing in again will send their waiting task changes, which this gap would make false.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:09.234Z",
    "resolved_at": null
  },
  {
    "id": 283,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether Graph accepts the input HTML service::onenote_page produces when a page is created, or normalises it into something the fidelity table did not predict. Every request has been read off a loopback socket and none has met Microsoft.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:29.659Z",
    "resolved_at": null
  },
  {
    "id": 284,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether OneNote's generated identifiers really change on every page update or only on some. That decides whether reading before every write is necessary or merely safe, and the read inside change_page is built on the reference's word for it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:30.352Z",
    "resolved_at": null
  },
  {
    "id": 285,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether removing a page's elements one at a time by generated identifier and appending new ones leaves a page a person recognises, or leaves it reordered and restyled. Remove-and-append was chosen over delete-and-recreate on a failure argument, not on a measurement of the result.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:31.045Z",
    "resolved_at": null
  },
  {
    "id": 286,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether a patch command with action delete, naming a generated identifier, is accepted for every element a OneNote page can hold. The supported-actions table quoted in this repository covers replace and says nothing this repository has read about delete, and changing a page depends on it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:31.736Z",
    "resolved_at": null
  },
  {
    "id": 287,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/oauth.rs",
    "line": null,
    "description": "Whether a consumer Microsoft account can grant Notes.ReadWrite without an administrator. The reference implies it and does not state it, and docs/PROVIDER_SETUP.md says plainly that we cannot tell somebody yet.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:32.434Z",
    "resolved_at": null
  },
  {
    "id": 288,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether a section group nested deeper than MOST_SECTION_GROUPS_DEEP, which is eight, is a real notebook anybody has. The bound is a guess at a hostile answer rather than a measurement of anyone's notebook.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:33.175Z",
    "resolved_at": null
  },
  {
    "id": 289,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether a paged OneNote listing really uses the @odata.nextLink field name and the value shape the walk follows. Paging is modelled on what list_contacts does for a different Graph endpoint and on a fixture this repository wrote.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:33.914Z",
    "resolved_at": null
  },
  {
    "id": 290,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether a page created by posting text/html to /me/onenote/sections/{id}/pages answers with the onenotePage JSON this client reads, and with which status. The fixture answers 201 with id and title because the reference describes that resource, not because anything saw it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T00:07:34.643Z",
    "resolved_at": null
  },
  {
    "id": 291,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether a real notebook's shape is what the four-level walk expects. Notebook, section group, section, page is a reading of Microsoft's reference; nobody here has seen a notebook, so whether every section is reached, and whether the sections of a shared or a class notebook answer at all, is unknown.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:47:39.602Z",
    "resolved_at": null
  },
  {
    "id": 292,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_page.rs",
    "line": null,
    "description": "Whether a page this program creates looks like a note to somebody who then opens it in OneNote itself. The HTML is built to what the reference names, and nothing has rendered it in OneNote's own window or read it back out of one.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:01.509Z",
    "resolved_at": null
  },
  {
    "id": 293,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "docs/development/the-notes-seam.md",
    "line": null,
    "description": "Whether a page edited in OneNote and read back here keeps the structure the fidelity table predicts, construct by construct. The middle step of that table is a model of the service, measured against what three documentation pages describe rather than against anything Microsoft returned.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:02.940Z",
    "resolved_at": null
  },
  {
    "id": 294,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_notes.rs",
    "line": null,
    "description": "Whether the conflict answer raises clashes nobody caused. A lastModifiedDateTime moves when the service touches a page for its own reasons, and only a real service touching a real page can show how often that happens and what it then feels like.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:04.386Z",
    "resolved_at": null
  },
  {
    "id": 295,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Whether the Notes section of the settings screen is navigable by keyboard through a screen reader when an account can give three different answers on one line. Nobody has heard the OneNote sentence, which is the longest of the three and names a loss.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:05.813Z",
    "resolved_at": null
  },
  {
    "id": 296,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_notes.rs",
    "line": null,
    "description": "Whether somebody who has used OneNote for years finds a note made by this program acceptable in their notebook. A page with a title and one block of content is not what OneNote's own editor produces, and this is a question for a person rather than a test.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:07.221Z",
    "resolved_at": null
  },
  {
    "id": 297,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_notes.rs",
    "line": null,
    "description": "Whether the folder-name separator, a slash with a space on each side, is heard well. A section called Work / Projects / Q3 is read at a screen reader's default punctuation level and nobody has listened to a list of them.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:08.639Z",
    "resolved_at": null
  },
  {
    "id": 298,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "Whether a Microsoft account that also has a calendar on a CalDAV server should get both backends. for_account answers one backend per account and asks the calendar server first, so such an account's OneNote sections are invisible here. Nobody has been asked which they would want.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:10.068Z",
    "resolved_at": null
  },
  {
    "id": 299,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/onenote_notes.rs",
    "line": null,
    "description": "Whether five requests per changed note is acceptable against Graph's rate limits. A page with no whole-document write and no entity tag costs a read for the marker, a read for the identifiers, the change, a read for the new marker and a read for what was kept, and no real account has ever been asked once.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:11.499Z",
    "resolved_at": null
  },
  {
    "id": 300,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/service/microsoft_graph.rs",
    "line": null,
    "description": "Whether GET /me/onenote/pages/{id} and GET /me/onenote/pages/{id}/content answer with the resource and the document this client reads. The fixtures answer what the reference describes for onenotePage and for page content, not what anything saw.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:12.950Z",
    "resolved_at": null
  },
  {
    "id": 301,
    "kind": "unrun-verify",
    "phase": "05.2",
    "file": "src/application/notes_backend.rs",
    "line": null,
    "description": "Whether a Notes list that is empty until the first sync is acceptable for a Microsoft account. A notebook's sections are Microsoft's answer and cannot be asked for while a screen is being filled, so the folders arrive at the first sync rather than before it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T02:48:14.390Z",
    "resolved_at": null
  },
  {
    "id": 302,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/first_run.rs",
    "line": null,
    "description": "Nobody has heard the first-run storage sentences under a screen reader. Whether they land as an important fact or as more of the same warning, arriving in the middle of a screen that already says everything which writes is experimental, is what criterion 4 really asks and no test here can answer it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T06:27:24.809Z",
    "resolved_at": null
  },
  {
    "id": 303,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/command_line.rs",
    "line": null,
    "description": "The storage paragraph added to the end of what --help prints has never been read in a real terminal or by a screen reader. Its wrapping and its place at the end of a long page are both unmeasured",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T06:27:38.279Z",
    "resolved_at": null
  },
  {
    "id": 304,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "src/common/logging.rs",
    "line": 79,
    "description": "Two of the three writes this program makes to the temporary folder are on no page a user can read. The log fallback at logging.rs:79 can hold whatever the running log holds, on a machine where the data folder could not be resolved, and the converted help pages at help_page.rs:97 hold nothing of anybody's. Neither goes through a paths.rs accessor, so the new check cannot reach either. 07-05 rewrites the privacy page and owns whether the log fallback earns a sentence",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T06:27:39.183Z",
    "resolved_at": null
  },
  {
    "id": 305,
    "kind": "todo",
    "phase": "07",
    "file": "tests/house_style.rs",
    "line": null,
    "description": "test_no_document_says_the_cache_is_encrypted and its companion have no guards/guards.toml record, so nothing measures that they still redden when the check they hold is narrowed. The signing guard written beside them in 07-01 does have one, and it breaks house_style.rs itself, so the same shape is available to them",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T06:27:51.385Z",
    "resolved_at": null
  },
  {
    "id": 306,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "installer/Wixen-Mail-Setup.iss",
    "line": null,
    "description": "Nothing here compiles the installer with ISCC, installs anything or looks at a shortcut, so whether the Start menu entry, the desktop shortcut and the Apps and Features entry really show the icon after a real install is unverified",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T07:45:55.655Z",
    "resolved_at": null
  },
  {
    "id": 307,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/accessibility/platform_bridge.rs",
    "line": null,
    "description": "No Linux or macOS build of Wixen Mail has ever been made, so the bridgeless sentences have only ever been produced from arguments a test on Windows chose. Whether they appear at all on a real non-Windows start, and whether the two constants a real non-Windows build supplies are the ones this expects, is unverified. 07-06 is the first thing that could answer the first half",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T09:52:08.998Z",
    "resolved_at": null
  },
  {
    "id": 308,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the About dialog draws the disclosure without the layout breaking is unverified. The dialog is fixed at 380 by 260 when there is nothing to disclose and grows to its contents when there is, and nobody has opened the grown version, because no build without the accessibility bridge exists",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T09:52:22.929Z",
    "resolved_at": null
  },
  {
    "id": 309,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/accessibility/platform_bridge.rs",
    "line": null,
    "description": "Nobody who depends on a screen reader has heard the disclosure. Whether the four paragraphs land as useful information or as a wall of apology in front of somebody who has just started a mail client is what guardrail 5 asks and no test here can answer it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T09:52:23.841Z",
    "resolved_at": null
  },
  {
    "id": 310,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/accessibility/platform_bridge.rs",
    "line": null,
    "description": "That adding a third platform module changes the answer with no edit to the code that says it is held structurally rather than by a test. No third platform module exists to add, and this tree has no compile-fail harness in which the absence of one could be expressed: grep -n trybuild Cargo.toml returns nothing",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T09:52:24.787Z",
    "resolved_at": null
  },
  {
    "id": 311,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/common/version.rs",
    "line": null,
    "description": "No release has ever been published from this repository and git tag returns nothing, so this comparison has never been handed a version string that came from anywhere but a test. No prerelease has been cut either, so the prerelease ordering and the channel rule resting on it are the parts with no real example behind them at all",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T11:38:55.805Z",
    "resolved_at": null
  },
  {
    "id": 312,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/common/version.rs",
    "line": null,
    "description": "That a published tag starts with a v is read off .github/workflows/release.yml, which names the portable download after the tag and publishes it under the glob wixen-mail-v-star.exe. No tag exists anywhere to confirm it. If that inference is wrong, or if cargo-release is later configured with a different tag name, the comparison answers that it could not read the tag for every release this project cuts and nothing fails",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T11:39:09.741Z",
    "resolved_at": null
  },
  {
    "id": 313,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/common/version.rs",
    "line": null,
    "description": "Nothing outside the tests calls the ordering, the channel type, the setting type or the offer decision. Whether any of it is reachable from a path a person can take is 07-05's to establish, and until it is, this is code that compiles and passes and has never run",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-12T11:39:10.499Z",
    "resolved_at": "2026-09-12T14:39:53.474Z"
  },
  {
    "id": 314,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/common/version.rs",
    "line": null,
    "description": "The forgiving read of a stored setting covers a value that is a string. A stored value of any other type still fails the read, which in the loader 07-05 inherits takes every setting on that machine back to its default. Nothing here tests that, because the loader is not in this plan, and 07-05 is told to read it and report which of the two it does",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T11:39:11.254Z",
    "resolved_at": null
  },
  {
    "id": 315,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_check.rs",
    "line": null,
    "description": "No release has ever been published from this repository, so the only answer this check has ever produced against a real endpoint is the one saying nothing is published. That was measured on 2026-09-12 against both endpoints and both really answer it, 404 on releases/latest and 200 with an empty array on releases. The two answers that matter most, a newer version and this being the newest, have only ever been produced from fixtures written here. If GitHub's real response for a published release does not parse, the feature is wrong rather than absent, which is worse",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:21.269Z",
    "resolved_at": null
  },
  {
    "id": 316,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Whether the answer is actually heard. It is sent as UIUpdate::ANewerVersionIsPublished or CommandAnswered, written to the status bar and announced at high priority on the command topic, and when there is a newer version a dialog follows it. Nothing in this tree can ask whether NVDA speaks it, whether the announcement and the dialog that follows read as one answer or as two, or whether the dialog interrupts the announcement before it finishes",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:47.935Z",
    "resolved_at": null
  },
  {
    "id": 317,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Whether the one three-valued combo box reads well under NVDA. D-16 chose one control with three values for an accessibility reason rather than a tidiness one, and the choice of a combo box over a radio group was argued from consistency with the other twenty-four controls on that dialog and from wxdragon having no RadioBox binding in this tree. The cost named is that somebody never opening the box does not hear that a test-version option exists. Only a real screen reader run settles whether that cost is the right one",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:48.694Z",
    "resolved_at": null
  },
  {
    "id": 318,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_check.rs",
    "line": null,
    "description": "Whether the development channel's list endpoint returns what this code expects. No list with anything in it has ever been returned for this repository: asked on 2026-09-12 it answers 200 with an empty array. The ordering, the refusal of unreadable tags and the hundred-entry bound have only been driven by fixtures built from another project's response with the tags substituted",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:49.465Z",
    "resolved_at": null
  },
  {
    "id": 319,
    "kind": "todo",
    "phase": "07",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Windows knows whether a connection is metered and nothing here asks. Somebody on a phone tether who chooses a kind of version will, once 07-09 lands, have installers downloaded over it. The answer taken for this phase is that the control's description says the download will be automatic, so leaving the setting off is the available answer. Detecting a metered connection is a real feature nobody has asked for",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:50.262Z",
    "resolved_at": null
  },
  {
    "id": 320,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "src/data/config.rs",
    "line": null,
    "description": "NOT_ANYTHING_ANYBODY_CHOOSES has no check of its own while its two siblings each have one. Verified rather than inherited: it has exactly two mentions in the file, its definition and the exception chain, so nothing re-asks whether either of its two entries is still a value nobody chooses. OFFERED_BY_ANOTHER_SCREEN and STORED_AND_OFFERED_BY_NOTHING each have a test that re-asks. Not fixed here because it is a finding about config.rs rather than about this plan",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:51.064Z",
    "resolved_at": null
  },
  {
    "id": 321,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "docs/installing.md",
    "line": null,
    "description": "docs/privacy.md and docs/installing.md carry the same four-line block listing what is stored under LOCALAPPDATA, word for word, with nothing checking they agree. 07-05 added the temporary-folder log fallback to privacy.md and deliberately did not duplicate it into installing.md, so the two now disagree. Either installing.md gains the same sentence or the block comes from one place",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:51.872Z",
    "resolved_at": null
  },
  {
    "id": 322,
    "kind": "todo",
    "phase": "07",
    "file": "src/service/outward.rs",
    "line": null,
    "description": "update_check.rs is on TALKS_BUT_ONLY_READS and is the first member whose answer will, once 07-09 lands, decide that an executable is fetched. That is not a write at somebody's account and it is not the harmless read the list's name implies, so the census's two categories do not quite describe it. Whether a third list is wanted was raised rather than settled, because the file is fingerprinted by ten guard records",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T14:39:52.687Z",
    "resolved_at": null
  },
  {
    "id": 323,
    "kind": "unrun-verify",
    "phase": "07",
    "file": ".github/workflows/other-platforms.yml",
    "line": null,
    "description": "07-06's checkpoint is open and unrun: nobody has dispatched the Other platforms workflow, so it is still unknown whether this crate builds or its suite passes on Linux or macOS. 07-06 task 3 is unexecuted, SHIP-05 does not close and phase 7 criterion 5 stays open. Dispatch the workflow from the Actions tab on branch one-dispatch-says-whether-this-crate-builds-off-windows or on main, let both jobs finish either way, and report per platform: whether cargo build --all-targets succeeded and the first error in full if not, whether cargo test --all-targets --no-fail-fast succeeded and how many ran and failed if not, the wall clock duration from the run summary, and the runner image label the first step printed",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T15:35:08.642Z",
    "resolved_at": null
  },
  {
    "id": 324,
    "kind": "unrun-verify",
    "phase": "07",
    "file": ".github/workflows/other-platforms.yml",
    "line": null,
    "description": "The new workflow has never been run by GitHub, so nothing about it is proven beyond the parts a local test reads. Its YAML has been checked by nothing that parses GitHub Actions, the thirteen apt packages quoted from upstream have never been installed on an ubuntu-latest image, and the macOS step that installs CMake only if it is absent has never taken either branch. A defect in any of those fails the first dispatch for a reason that says nothing about whether the crate builds, which is the question the dispatch is for",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T15:35:20.673Z",
    "resolved_at": null
  },
  {
    "id": 325,
    "kind": "deviation",
    "phase": "07",
    "file": ".planning/ROADMAP.md",
    "line": null,
    "description": "A whole-tree guard that reads the disk cannot tell one agent's uncommitted work from another's. test_the_roadmap_counts_the_files_that_are_on_disk runs on every commit and compares the roadmap's progress table with the phase directories as they sit on disk, so while a second agent was planning phase 6 in the same working tree its untracked plan files made that phase's 0/TBD cell false and refused 07-06's document commit. The count also raced: the refused run saw 1 plan and a re-run three minutes later saw 2. The row was set to 0/2 by 07-06 to get past the gate, which is bookkeeping 07-06 does not own and which phase 6's planning will move again. Whether a tree guard should read the index rather than the disk, or whether two agents should share a working tree at all, is raised rather than settled",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T15:49:32.838Z",
    "resolved_at": null
  },
  {
    "id": 326,
    "kind": "unrun-verify",
    "phase": "07",
    "file": ".github/workflows/release.yml",
    "line": null,
    "description": "The release workflow has never run: git tag returns nothing and git ls-remote --tags origin returns nothing while --heads answers, so no published glob has ever been matched against a real dist/ and the tag branch in scripts/build-installer.sh has never been taken. dist/wixen-mail-v*.exe agrees with dist/wixen-mail-$tag.exe only while the tag begins with v, which comes from cargo-release's defaults; if it does not, that file was published as a silent absence before this change and is a failed release after it, and the second is what was wanted",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:22:01.452Z",
    "resolved_at": null
  },
  {
    "id": 327,
    "kind": "deviation",
    "phase": "07",
    "file": ".github/workflows/release.yml",
    "line": null,
    "description": "cargo release pushes the tag at the Create release version and tag step, before anything is built, so any failure after that point leaves a tag on the remote with no release behind it. The new existence check moves the failure earlier than publication but not earlier than the tag. Changing it means moving cargo release after the build, which changes when a tag exists and so when a release happens, which guardrail 7 says is a deliberate decision rather than something a change about asset names makes on the way past",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:22:14.542Z",
    "resolved_at": null
  },
  {
    "id": 328,
    "kind": "deviation",
    "phase": "07",
    "file": "tests/a_move_says_what_has_not_been_sent.rs",
    "line": null,
    "description": "This target cannot reach the Windows credential store on every run: it fails with No default store has been set, so cannot search or create entries. Seen in two of three whole-tree runs on 2026-09-12, a different test of the file each time, and the file passes on its own on an unbroken tree. Not caused by any change in this plan and not diagnosed",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:22:15.335Z",
    "resolved_at": null
  },
  {
    "id": 329,
    "kind": "unrun-verify",
    "phase": "07",
    "file": ".planning/REQUIREMENTS.md",
    "line": null,
    "description": "Success criterion 1 cannot close until an Azure Artifact Signing account exists. The certificate is chosen (decision 6 of 2026-09-06, Azure Artifact Signing, about 9.99 dollars a month, publisher name Pratik Patel, residence requirement met) and nothing is signed. Creating the account needs a subscription, an identity check naming a real person, a payment and a role grant in a tenant, none of which is a repository operation. This is a dependency on something outside this repository rather than a defect in it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:44:52.410Z",
    "resolved_at": null
  },
  {
    "id": 330,
    "kind": "deviation",
    "phase": "07",
    "file": ".planning/REQUIREMENTS.md",
    "line": null,
    "description": "SHIP-01 says the published installer and the executable inside it, which is two things. The census plan 07-08 task 1 derives from the installer script and the release workflow counts seven: wixen-mail.exe, wixen_mail_search.dll and wixen-mail-search-setup.exe inside the installer, the setup executable, the portable copy and the zip published beside it, and the uninstaller Inno generates, which is in neither list. A plan written to the requirement's own wording would leave five unsigned, two of them executables a user runs from an installed folder. Recorded as a scope finding against SHIP-01 rather than as a defect: the requirement is narrower than the thing it is about",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:48:30.006Z",
    "resolved_at": null
  },
  {
    "id": 331,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "tests/installer.rs",
    "line": null,
    "description": "The census of what has to be signed counts PE files only, which is .exe and .dll, because those are what Authenticode embeds a signature into. A .ps1, an .msi or a .cat can also carry a signature, each by a different mechanism, and none would be counted. This project ships none of the three today, so the filter is correct about what it claims and incomplete about the question. Widening it without settling how each of those is signed would be worse, because it would pair a file with a signing step that cannot sign it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:48:43.047Z",
    "resolved_at": null
  },
  {
    "id": 332,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "tests/installer.rs",
    "line": null,
    "description": "The census reads one Source line as one artefact, so a wildcard naming executables would be a single entry standing for however many files it matched. The installer already carries one wildcard, the line that installs the markdown guides, which the filter drops as prose. If an executable wildcard ever arrives the count is right about the line and wrong about the artefacts, and nothing would say so",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:48:43.841Z",
    "resolved_at": null
  },
  {
    "id": 333,
    "kind": "deviation",
    "phase": "07",
    "file": ".planning/WINDOWS.md",
    "line": null,
    "description": "gsd-tools windows append writes a description containing a backslash into both halves of the ledger without reconciling the escaping. The JSON half escapes the character and the markdown table half does not, so the two halves disagree and the commit gate refuses the commit. Found on entry 332, whose description quoted a Windows path from the installer script. Worked around by rewording that entry to avoid the character and correcting both halves by hand. Anything appended through the tool that quotes a Windows path or a regular expression meets this, and the failure reads as a ledger the author corrupted rather than as a tool defect",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:58:04.273Z",
    "resolved_at": null
  },
  {
    "id": 334,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "no installer has ever been fetched over a real connection; the transport has never run outside a fixture",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:37.668Z",
    "resolved_at": null
  },
  {
    "id": 335,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "the handover has never run: no window has closed and been replaced by an installer, and no installer has replaced any files",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:56.351Z",
    "resolved_at": null
  },
  {
    "id": 336,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "the publisher check has never seen a genuine Wixen Mail signature, because no release is signed; it was proved against a Microsoft-signed file instead",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:57.117Z",
    "resolved_at": null
  },
  {
    "id": 337,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "docs/installing.md",
    "line": null,
    "description": "plan 07-09 task 3 was not attempted: no screen reader has heard an update happen, a refusal, or the moment the window closes",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:57.894Z",
    "resolved_at": null
  },
  {
    "id": 338,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "src/service/outward.rs",
    "line": null,
    "description": "the outward census has no category for a module whose bytes are executed; plans 07-05 and 07-09 both raised it and neither acted, because the change costs re-measuring ten records",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:58.771Z",
    "resolved_at": null
  },
  {
    "id": 339,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "nothing detects a metered connection, so somebody who chose a channel on a home connection has installers of about 12 MB fetched over a phone tether",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:37:59.641Z",
    "resolved_at": null
  },
  {
    "id": 340,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "the handover cannot remove the mutex race: an installer reaching its own AppMutex check before this process has finished closing will say Wixen Mail is still open",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:38:00.451Z",
    "resolved_at": null
  },
  {
    "id": 341,
    "kind": "deviation",
    "phase": "07",
    "file": "guards/guards.toml",
    "line": null,
    "description": "plan 07-09 required the ten guard records reading the outward census to be re-measured by hand; CLAUDE.md took guard sweeps off the critical path on 2026-09-03, so they were not run and are owed to the phase sweep",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:38:01.262Z",
    "resolved_at": null
  },
  {
    "id": 342,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "src/service/update_download.rs",
    "line": null,
    "description": "the revocation policy is not checked against a revoked certificate, because none exists to check against",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T22:38:02.066Z",
    "resolved_at": null
  },
  {
    "id": 343,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/accessibility/feedback.rs",
    "line": 348,
    "description": "Channel::ALL is still a hand-written [Channel; 4] and carries the same hole 06-01 closed for Event::ALL: nothing forces a fifth channel into it. Out of scope deliberately, four is a much smaller surface than sixteen and nothing in phase 6 adds a channel",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T00:24:47.583Z",
    "resolved_at": null
  },
  {
    "id": 344,
    "kind": "unmet-truth",
    "phase": "06",
    "file": ".planning/REQUIREMENTS.md",
    "line": 1288,
    "description": "FEEDBACK-01's evidence says set_event_channels is private and that no screen could write an override without changing a visibility. 06-01 made it pub and added a public reader, so that sentence and its four line numbers are now wrong about the tree. Not corrected here because whether REQUIREMENTS.md is corrected in place is decision 7, which 06-07 puts to Pratik",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T00:35:32.541Z",
    "resolved_at": null
  },
  {
    "id": 345,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "The per-event panel on the Settings Feedback tab has never been heard. A live test builds the real dialog and reads back sixteen events, six labelled check boxes and two lines, which proves the structure is there and says nothing about whether it reads well with NVDA, Narrator or JAWS",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:01:27.680Z",
    "resolved_at": null
  },
  {
    "id": 346,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "The three per-event controls reload beneath the cursor when the event picker changes, and nothing in this repository can prove that reads well. It is a live-region shaped problem: a screen reader user moves to the picker, changes it, and three controls below them silently become about a different event. The Reading tab already uses this pattern, so a listening pass should judge both at once",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:01:49.898Z",
    "resolved_at": null
  },
  {
    "id": 347,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "The sentence saying that choosing between speech and braille is done in the screen reader and not here has never been heard. It is the sentence the whole of criterion 1 exists to make somebody meet, and whether it lands where they meet it is a listening question",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:01:51.248Z",
    "resolved_at": null
  },
  {
    "id": 348,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Somebody whose settings file has braille on and speech off, or the other way round, now meets one control where there were two, and it opens ticked. Nobody has judged how that reads or whether the change is noticed. The direction is deliberate and safe, nothing being announced stops being announced, but safe is not the same as understood",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:01:52.528Z",
    "resolved_at": null
  },
  {
    "id": 349,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "That the event picker's selection handler and the reset button's click handler really call the functions the tests drive is proved by reading one line each and by nothing else. wxdragon 0.9.17 exposes no way to raise a widget event from outside, so tests/every_event_has_a_control.rs drives the real controls through those functions directly. The behaviour is proved; the wiring is not",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:02:16.927Z",
    "resolved_at": null
  },
  {
    "id": 350,
    "kind": "unmet-truth",
    "phase": "06",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "The event picker is a Choice, which has no label of its own to carry, so it is named with set_accessible_name plus a static text beside it, the way every other Choice in this dialog is. That name reaches MSAA, which NVDA reads. On UI Automation, which Narrator reads, the name arrives only through Windows falling back to the nearest static text, which is a fallback rather than something this code set. Neither channel has been checked for this control",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:02:19.760Z",
    "resolved_at": null
  },
  {
    "id": 351,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/accessibility/feedback.rs",
    "line": null,
    "description": "Channel::setting_label now has no shipping caller. The global boxes are built from Switch, so the four strings it holds are reached only by its own two tests. The plan for 06-02 said not to change them and a test guards them, so it was left alone rather than removed quietly. Point dead-code-hunter at it in a later plan of this phase",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:02:21.175Z",
    "resolved_at": null
  },
  {
    "id": 352,
    "kind": "todo",
    "phase": "06",
    "file": "tests/checkbox_labels.rs",
    "line": null,
    "description": "The label walk reaches the six feedback check boxes and no other check box the settings dialog builds, because SettingsWidgets keeps the rest of its fields private. Widening it means making about twenty fields public for a test, which is worth deciding deliberately rather than as a side effect of 06-02",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:02:22.235Z",
    "resolved_at": null
  },
  {
    "id": 353,
    "kind": "unrun-verify",
    "phase": "06",
    "file": ".planning/ROADMAP.md",
    "line": null,
    "description": "Criterion 1 clause 2, by keyboard, does not close. Every control on the Feedback tab is a native Choice, CheckBox or Button so Tab reaches them, and every check box and the button carries a mnemonic distinct from the other nine on the tab. Both facts were established by reading the source, not by pressing a key. Nobody has tabbed through this tab and no test presses one",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T06:50:57.999Z",
    "resolved_at": null
  },
  {
    "id": 354,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "Cargo.toml",
    "line": 15,
    "description": "Nothing in this repository builds at the declared floor. Cargo.toml says rust-version = 1.88 and every workflow plus rust-toolchain.toml now uses 1.98.1, so 1.88 is a claim no build has tested since it was written. Found 2026-09-13 while pinning the toolchain. Whether an MSRV job belongs here is a scoping question nobody has settled; recorded rather than fixed.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T08:39:16.912Z",
    "resolved_at": null
  },
  {
    "id": 355,
    "kind": "skipped-test",
    "phase": "07",
    "file": "src/service/signed_mail.rs",
    "line": 6523,
    "description": "test_the_withdrawal_question_really_reaches_windows_own_answer stops guarding on a machine that holds no intermediate authority it trusts. Windows does not check withdrawal for the root of a chain, so an intermediate whose issuer is not installed gets no verdict, and nothing here can tell that from a broken walk. The test reads that precondition off issuer_trust and returns early when it is absent, printing why. Recorded 2026-09-13 rather than hidden: a guard that can quietly stop guarding is the shape this project keeps meeting.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T08:39:33.202Z",
    "resolved_at": null
  },
  {
    "id": 356,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "Cargo.toml",
    "line": null,
    "description": "This ships a known-broken RSA implementation and will go on doing so. rsa 0.9.10 arrives through pgp 0.20.0, which cannot drop it, and RUSTSEC-2023-0071 has an empty patched list. The advisory is four concerns, not one: modexp timing is fixed, but the Ok or Err behavioural oracle is open at RustCrypto/RSA PR 680 and unblinded modexp on the default path is open at PR 702. The oracle needs no timing at all and recovers an RSA-1024 plaintext in 275,490 queries against published 0.10.0-rc.18. Accepted on 2026-09-13 on an exposure argument, not a fix: this program gives a sender no per-query feedback, so the oracle has no channel here. That argument can be wrong and it expires the day anything here answers a sender differently depending on whether a PGP body decrypted. PR 680 landing and shipping is what changes the answer, not the advisory clearing. Full reasoning in .cargo/audit.toml.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T11:19:24.037Z",
    "resolved_at": null
  },
  {
    "id": 357,
    "kind": "unmet-truth",
    "phase": "07",
    "file": "scripts/audit.sh",
    "line": null,
    "description": "cargo audit does not fail on an unmaintained or yanked warning without -D warnings, so the audit gate is blind to a whole class it looks like it covers. Measured 2026-09-13 after RUSTSEC-2023-0071 was accepted: the plain run exits 0 while printing three warnings nobody has decided, lzw RUSTSEC-2020-0144 unmaintained, proc-macro-error RUSTSEC-2024-0370 unmaintained, and chacha20 0.10.1 yanked. One unmaintained crate, paste, is in the accepted list, which makes the treatment of the class inconsistent: one is written down and three are printed and ignored. Not fixed here because turning on -D warnings needs a decision on each of the three, which is the same product decision the rsa entry just took and is out of this change's scope.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T11:19:39.327Z",
    "resolved_at": null
  },
  {
    "id": 358,
    "kind": "unmet-truth",
    "phase": "07",
    "file": ".cargo/audit.toml",
    "line": null,
    "description": "The still-reported check cannot see an acceptance whose basis has evaporated while the advisory goes on being reported. The rsa entry rests on the advisory having an empty patched list, which cargo audit prints as No fixed upgrade is available. The day that becomes Upgrade to something, the run still reports the advisory, the check stays green, and an acceptance resting on there being nothing to upgrade to has quietly stopped being true. A crate version fingerprint was considered on 2026-09-13 and decided against, with the reason written in scripts/audit.sh and in the phase 7 RSA advisory report: for every way an acceptance has really gone stale here the version moving and the advisory ceasing to be reported are the same event, so the fingerprint fires no earlier and costs a hand-maintained number per entry. It also would not close this gap, which is about the advisory rather than the crate. Closing it means recording the Solution line each acceptance was judged against and comparing it on every run.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T11:19:54.227Z",
    "resolved_at": null
  },
  {
    "id": 359,
    "kind": "deviation",
    "phase": "07",
    "file": "scripts/audit.sh",
    "line": null,
    "description": "The run that checks the acceptances depends on where cargo-audit looks for its config, and nothing would say so if that moved. Measured 2026-09-13 against cargo-audit 0.22.2 by running from target/auditprobe, two directories below the project config: it reads .cargo/audit.toml from the working directory only and does not walk upwards. The second run is made from a scratch directory carrying a config that ignores nothing, so it is correct whether or not that measurement holds. What it would not survive is a cargo-audit that merged configs from several directories, and the failure would be silent in the worst direction: every acceptance would read as still applying while nothing was actually checked. The narrower-than-the-filtered-run refusal catches the opposite direction only.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T11:20:09.076Z",
    "resolved_at": null
  },
  {
    "id": 360,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/date_display.rs",
    "line": null,
    "description": "No date written in any language has been read aloud by a screen reader in that language. Plan 06-03 task 2 makes the month names in a date come from Windows, and every assertion about them is a string comparison in a test. The French and Russian tests force a locale name and compare bytes, which proves the words Windows hands back are the words this code writes. Whether a French month inside an order the person chose, spoken by a French NVDA voice, sounds like a date rather than like a fault needs a French Windows, a French voice and somebody who speaks French.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T14:45:37.895Z",
    "resolved_at": null
  },
  {
    "id": 361,
    "kind": "unmet-truth",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": 849,
    "description": "The twelve month names in the appointment form's month list now come from Windows, and nothing tests that they do. Measured 2026-09-13: wx_item_form.rs holds 14 tests and not one of them reads the Choice this line builds, because the Choice is built inside a closure in build_date_fields and needs a real parent window. The change is glue over a function that is itself tested, and it is reached from wx_item_form.rs line 1007 and wx_send_later.rs line 127, so it runs. What nobody has done is open the form and look at the list. Proving the wiring wants a live-window test on the pattern of the checkbox_labels suite.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T14:45:52.425Z",
    "resolved_at": null
  },
  {
    "id": 362,
    "kind": "unmet-truth",
    "phase": "06",
    "file": ".planning/ROADMAP.md",
    "line": null,
    "description": "Success criterion 2 of phase 6 has four clauses and plan 06-03 closes part of one. Month names follow the machine in a date and in a month heading and in the appointment form's month list, but not in the eight signature-outcome sentences, which still write an English month through the chrono format %B at src/service/signed_mail.rs line 1373. Day names do not follow the machine at all: src/application/occurrences.rs line 701 still matches a chrono Weekday to an English string. Relative wording is the plan's own blocking checkpoint and was left unanswered on purpose. The silent English fallback is the one clause that is met. Task 3 of 06-03 carries the first two and is written against the checkpoint's answer.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T14:46:05.068Z",
    "resolved_at": null
  },
  {
    "id": 363,
    "kind": "unmet-truth",
    "phase": "06",
    "file": "src/common/how_the_machine_writes_dates.rs",
    "line": null,
    "description": "A stored day that no month has falls back to English on a machine that does have the language, which is a wider fallback than the criterion asks for. The criterion says English where there is no translation. a_month_and_a_day accepts a day from 1 to 31 without asking which month it is, so a birthday stored as --02-30 reaches Windows, Windows refuses the whole date, and the wrapper answers in English. On a French machine that reads February 30 rather than fevrier 30. It is a corrupt stored row either way and it does not take the reading down, which is what the threat register asked for, but the reason the reader sees English there is not the reason the sentence on the settings screen gives. Found by reading at green on 2026-09-13, not by a test.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T14:46:18.671Z",
    "resolved_at": null
  },
  {
    "id": 364,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/date_display.rs",
    "line": null,
    "description": "The Russian genitive was produced on an en-US machine by passing ru-RU as a locale name, never on a Windows installed in Russian. Measured 2026-09-13: a date reads 2 января and a month heading reads Январь, which are different words, so the two mechanisms are doing different things and the test that separates them is real. What that does not settle is whether a Windows whose own language is Russian gives the same answers, or whether a machine missing the NLS data for a language behaves as this one does when asked for a locale it has. The French assertion is weaker still and says so in the test: fr-FR writes juillet both ways, so it passes against the wrong mechanism and would not have caught the fault the Russian one caught.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T14:46:34.120Z",
    "resolved_at": null
  },
  {
    "id": 365,
    "kind": "deviation",
    "phase": "06",
    "file": "tests/a_move_says_what_has_not_been_sent.rs",
    "line": null,
    "description": "The first attempt at the 06-03 merge commit failed the full gate on one integration target, a_move_says_what_has_not_been_sent, and nothing explains why. It passed on its own immediately afterwards, all 10 cases, and passed again in the run that completed the merge at e98514b0, and the same target had passed minutes earlier in the full run on the branch. So the merge is green on two full runs and red on one, with no change to the tree between them. Which case failed was not captured, because the merge output was read through tail and the detail scrolled past, which is the part that should not happen again: a transient failure whose text nobody kept is a transient failure nobody can diagnose. This target builds a live window and the run that failed followed another full run closely, so contention is the obvious guess and it is only a guess. Recorded rather than absorbed, because a check that fails one run in three while reading as green is guardrail 4.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T15:42:15.765Z",
    "resolved_at": null
  },
  {
    "id": 366,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/common/catalogue.rs",
    "line": null,
    "description": "No sentence out of the translation catalogue has been heard by anybody through a screen reader. The four English sentences are byte-identical to what relative_to wrote before, held by the table test under a forced en-US, and that proves structure, not that a listener hears them the same. Waits for the pass after phase 8.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:12:55.075Z",
    "resolved_at": null
  },
  {
    "id": 367,
    "kind": "unmet-truth",
    "phase": "06",
    "file": "locales/en-US/dates.ftl",
    "line": null,
    "description": "No non-English catalogue exists. On every computer not set to English, 2 days ago is still English, silently, which is criterion 2's own fallback clause and is disclosed on the Reading tab. Which languages Wixen Mail speaks and who writes them is a version 2 decision and Pratik's; the Russian and Polish resources that prove the plural machinery live inside tests and ship nowhere, because a translation nobody here can read is not a translation to ship.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:12:56.008Z",
    "resolved_at": null
  },
  {
    "id": 368,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/common/catalogue.rs",
    "line": null,
    "description": "Whether a Russian listener hears 2 дня назад as natural in a list cell, and whether a number the formatter writes with a U+00A0 group separator is read correctly by a screen reader in that locale, are questions only a Russian Windows, a Russian voice and a Russian speaker can settle. The forms were produced on an en-US machine through a resource written inside a test. Waits for the pass after phase 8.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:12:56.963Z",
    "resolved_at": null
  },
  {
    "id": 369,
    "kind": "deviation",
    "phase": "06",
    "file": "src/service/spellcheck/mod.rs",
    "line": null,
    "description": "Two readers of LOCALE_SNAME now exist: spellcheck::system_language at spellcheck/mod.rs through GetLocaleInfoW, and how_the_machine_writes_dates::this_computers_locale_name through GetLocaleInfoEx, which chooses the catalogue. Measured agreeing on this machine on 2026-09-13, both en-US. The spellchecker's copy is in src/service, which src/common cannot reach without a new layering direction, and its file is fingerprinted by 30 guard records, so retiring one is version 2's job and not this plan's.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:12:57.918Z",
    "resolved_at": null
  },
  {
    "id": 370,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/application/occurrences.rs",
    "line": null,
    "description": "The repeat-series sentence has never been heard in any language. every week on mardi and jeudi is what a French computer now hears, a French day inside an English frame, held by a test under a forced fr-FR; whether a French listener hears the day as a day, or hears the sentence as a fault, waits for the pass after phase 8 and for version 2's translation of the frame.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:13:18.748Z",
    "resolved_at": null
  },
  {
    "id": 371,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/service/signed_mail.rs",
    "line": null,
    "description": "The eight signature-outcome sentences have never been heard in any language. The date in them now follows the computer in the form a date puts a month in, held by a test under a forced fr-FR asserting 28 août 2026; whether the sentence reads as a date to a listener waits for the pass after phase 8.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:13:19.744Z",
    "resolved_at": null
  },
  {
    "id": 372,
    "kind": "unmet-truth",
    "phase": "06",
    "file": "src/application/repeating.rs",
    "line": null,
    "description": "Four interface literals still carry English day and month names and are allowed by name in the source-reading guard with a reason beside each: Every weekday, Monday to Friday in repeating.rs and item_fields.rs, and Month first, July 26, Day first, 26 July and A word, July 26, 2026 in wx_settings.rs. They are labels on choices, interface text version 2 translates; a French day inside an English label is not better than an English one. The guard asserts each allowance is still in the tree so the list cannot outlive its subjects.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:13:20.732Z",
    "resolved_at": null
  },
  {
    "id": 373,
    "kind": "deviation",
    "phase": "06",
    "file": "locales/en-US/dates.ftl",
    "line": null,
    "description": "locales/ maps to no target in the gate. scripts/check.sh scopes tests by src/*.rs and tests/*.rs, house_style's ours() does not walk locales/ and the plan says not to add it, so a commit touching only the catalogue runs formatting, clippy and the tree-reading guards and never common::catalogue::, whose parse and completeness checks read the file through include_str. The whole gate at the merge and CI are what cover it until a mapping is decided with the rest of version 2's questions.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:13:21.714Z",
    "resolved_at": null
  },
  {
    "id": 374,
    "kind": "deviation",
    "phase": "06",
    "file": "tests/a_move_says_what_has_not_been_sent.rs",
    "line": null,
    "description": "Ledger 365's cause, found by keeping the failure text the second time it fired, on the build(06-03) commit: test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent panicked with No default store has been set from keyring. keyring 4.1.5's Entry::new races its own lazy initialisation: the thread that wins a compare_exchange on an AtomicBool sets the default store, and a thread that loses it goes straight to keyring_core::Entry::new before the winner has finished. Ten tests start in parallel on a fresh process, so it fires about one run in three. Not contention, and not this target's fault. A second finding: the in-memory seam in secret_store.rs is cfg(test), which an integration test never sees, so every run of this target reaches the real Windows credential store. Out of 06-03's scope; a Once around the first Entry::new in secret_store, or a seam the integration targets can see, is the fix.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:13:22.733Z",
    "resolved_at": null
  },
  {
    "id": 375,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_account_manager.rs",
    "line": null,
    "description": "The three Allow Changes for this account boxes, their section heading and the note beneath have not been heard with a screen reader. Each carries its label on the control itself so the name reaches UI Automation and MSAA both, and the disabled arm's label says why it is unavailable and names the heading in Settings; what only a listening pass can settle is whether a person tabbing through hears why the third box is missing, because Windows skips a disabled control in the tab order and the note beneath is not in it either. Waits for the pass after phase 8.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T06:25:29.308Z",
    "resolved_at": null
  },
  {
    "id": 376,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/application/allowed.rs",
    "line": null,
    "description": "None of the sending, deleting and syncing a per-account answer governs has ever run against a real account, so what unticking a box on the account dialog holds back has only ever been held back in tests. allowed_for is unchanged and narrows as it always did; the new writer set_allowed_for is covered by three unit tests and no live server.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T06:25:29.729Z",
    "resolved_at": null
  },
  {
    "id": 377,
    "kind": "deviation",
    "phase": "06",
    "file": "src/application/allowed.rs",
    "line": null,
    "description": "The refusal a sync says when a per-account answer is holding a change still names Settings and not the account: turn_the_setting_on has one owner and no account in hand, so a person whose Settings box is on hears advice that sends them to a box that is already ticked. The account dialog's note and the testing page say so, and the wording is pinned literally in tests across contacts_sync.rs, calendar.rs, pim_command.rs and answering.rs, so naming both places is a plan of its own. The tree already says it both ways: carddav_sync.rs and address_book_source.rs say Allow Changes is off for this account, which pointed at nothing until 06-04.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T06:25:41.365Z",
    "resolved_at": null
  },
  {
    "id": 378,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "tests/account_edit_protocol_fields.rs",
    "line": null,
    "description": "The live-dialog test proves each box is enabled exactly when the stored Settings answer allows it, reading the same profile the dialog reads, so on the machine that ran it, where Settings allows everything, it exercised the offered arm of all three boxes and never the unavailable one. The unavailable arm's wording is proved by a unit test on permission_box_label and by no live box. build_account_edit_dialog reads ConfigManager::load_stored inside itself on the precedent of the directory fields, and there is no seam to hand it a different answer; adding one is the fix.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T06:25:41.760Z",
    "resolved_at": null
  },
  {
    "id": 379,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "A reminder found due while somebody is typing is now said and sounded at that look with its window held, and nobody has heard it: whether an Urgent announcement arriving mid-word in the composer, a note or the contacts search is heard as a warning or as an interruption is a listening pass, not an assertion. The rule is tested without a window and the call site is read as text by tests/wired.rs.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T09:31:06.902Z",
    "resolved_at": null
  },
  {
    "id": 380,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The reminder window opens one look, a minute, after the sentence was said, whether or not typing has stopped. That steals focus mid-word eventually, which is the thing being complained about, just later; Pratik accepted that on 2026-09-14 for a warning and a minute. Nobody has heard the window arrive a minute after the sentence, so whether it reads as helpful or as the same thing twice is unsettled. The sentence is not announced again when it opens; the claim that a screen reader reads a dialog's static text when focus arrives in it has not been checked against this dialog.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T09:31:07.312Z",
    "resolved_at": null
  },
  {
    "id": 381,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "The reminder window's tone comes back once a minute until focus reaches it, ten times at most, on RepeatingTone, which is tested against handed-in instants. Nobody has heard it come back, nobody has judged whether ten tones a minute apart reads as being looked after or as being nagged, and the timer's reading of has_focus on the three buttons and the snooze Choice has never been watched in a running build; on Windows it should be false while another application is in front and true the moment somebody comes to the dialog, and that is a claim about the toolkit rather than a measurement.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T09:31:07.724Z",
    "resolved_at": null
  },
  {
    "id": 382,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/accessibility/screen_reader.rs",
    "line": null,
    "description": "Whether a screen reader speaks the reminder sentence while another application is in front is unchecked. UiaRaiseNotificationEvent does not move focus, and NVDA speaks notifications from the foreground process, so a person in Word when a reminder is due may hear only the tone; that is why the tone repeats, and nobody has confirmed either half by ear.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T09:31:08.142Z",
    "resolved_at": null
  },
  {
    "id": 383,
    "kind": "deviation",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "After a hold, the gap between the reminder's own tone and the window's first repeat is two minutes rather than one: say sounds at the look that finds the reminder, the window opens a look later, and RepeatingTone counts its minute from the window opening because raise is told only that the sentence was already said, not when. A window that opens at once has the one-minute gap the plan describes. Handing raise the instant of the say would close it and was not built, because 06-09 replaces this window and its opening.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T09:31:08.544Z",
    "resolved_at": null
  },
  {
    "id": 384,
    "kind": "unmet-truth",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "The scan still fetches releases/latest of Axe.Windows on every run, so the rule set a coverage list describes is whatever Microsoft shipped that morning and can change with no commit here. Pinning is 06-06 task 2, behind a checkpoint Pratik has not answered; until then 06-07's list must carry the version it was read against and the date.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-14T10:10:11.730Z",
    "resolved_at": "2026-09-14T11:33:18.715Z"
  },
  {
    "id": 385,
    "kind": "todo",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "The MSAA half of the scan, the channel NVDA reads, is read by no test and maps to no gate target: a change to it answers affected on a branch and runs formatting, clippy and the tree guards only. The which-checks rule 06-06 added covers .github/workflows/ and not this script, because there is nothing for a workflow-shaped rule to run for it. A test reading its exit-code contract (0 named, 1 unnamed, 2 walk failed), which the workflow depends on at lines 180 to 184, is the missing half.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T10:10:12.157Z",
    "resolved_at": null
  },
  {
    "id": 386,
    "kind": "deviation",
    "phase": "06",
    "file": "scripts/which-checks.test.sh",
    "line": null,
    "description": "06-06 task 1 asked for a plain-markdown docs_only case and a plain-Rust affected case as the allow half; both already existed three times over at lines 95 to 103 and a fourth copy proves nothing. Written instead: a .yml outside the workflows folder answers affected and a document inside .github answers docs_only, the two cases that redden under the wrong spellings of the rule, *.yml and .github/*. Same substitution 07-02 made for the .iss rule.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T10:10:12.554Z",
    "resolved_at": null
  },
  {
    "id": 387,
    "kind": "deviation",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "06-06 task 1 said to prove the gate hole by breaking ScanTarget::ALL and watching check.sh pass. That break lives in src/presentation/scan_target.rs, which the gate maps to a scoped target, so it would have run the reading test and gone red: the wrong side of the hole. Measured instead by taking one window out of the workflow's array with the code left alone, which is the change the hole is about. Before the rule: affected, exit 0 in 64s. After: all, exit 101 in 206s, naming blocked-senders.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T10:10:12.940Z",
    "resolved_at": null
  },
  {
    "id": 388,
    "kind": "unrun-verify",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "06-06 task 2 wired thirty scan targets where there were ten and every one opened the window it names on a throwaway profile on this machine, as a listing of the process's top-level windows shows. No CI scan has run on any of them: nothing has been pushed since 2026-09-10, so on both channels each new target is a window somebody has opened and nobody has scanned, until 06-08 reads an artifact.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:47.755Z",
    "resolved_at": null
  },
  {
    "id": 389,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "The MSAA walk now enumerates every visible top-level window the process owns rather than .NET's main window, which is never a dialog. Proved against notepad.exe, one window walked with its title leading each path, and against a process that does not exist, exit 2 where the old script exited 1. Not proved against this application on this machine: the walk of any Wixen Mail window crashes pwsh here, see the next entry, so the first dialog this channel reads is the one CI reads.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:48.209Z",
    "resolved_at": null
  },
  {
    "id": 390,
    "kind": "todo",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "Walking any Wixen Mail window over MSAA crashes PowerShell on this machine with STATUS_STACK_BUFFER_OVERRUN, exit -1073740791, under pwsh 7.6.6 and Windows PowerShell 5.1 alike, on the main window alone and before the enumeration change. NVDA is running here and CI has no screen reader; the run of 2026-09-10 walked 1797 elements without crashing. Not diagnosed. The workflow now records any exit other than 0 or 1 as a walk that failed, so if CI meets this it is a named failure rather than a clean pass. Still so on 2026-09-16: 09-05 ran the walk under pwsh on the Signature Manager and on each of the five new editor targets, twice for the editors, eleven runs, exit -1073740791 every time and nothing printed before it, NVDA running, nothing stopped and nothing diagnosed.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:48.643Z",
    "resolved_at": null
  },
  {
    "id": 391,
    "kind": "deviation",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "06-06 task 2 added --alwayssavetestfile to the Axe.Windows call, which the plan did not name. The CLI's own --help says the test file is saved only if errors are found, so the workflow's no-file-means-broken check reported seven of eleven clean windows as scans that failed on 2026-09-10, each a line after the CLI printed 0 errors were found. Rule 1: the distinction the workflow exists to make was inverted for the clean case.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:49.055Z",
    "resolved_at": null
  },
  {
    "id": 392,
    "kind": "deviation",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "06-06 task 2 changed scripts/msaa-names.ps1, which the plan did not name: it walks every visible top-level window the process owns instead of .NET's MainWindowHandle, and a failed walk exits 2 instead of terminating at Write-Error under the Stop preference with exit 1, which the workflow read as an unnamed control. Rule 2: the channel NVDA reads had never seen a dialog, measured from the CI log of 2026-09-10 where three dialogs reported the same 1797 elements.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:49.490Z",
    "resolved_at": null
  },
  {
    "id": 393,
    "kind": "deviation",
    "phase": "06",
    "file": "src/presentation/scan_target.rs",
    "line": null,
    "description": "06-06 task 2 added mail-module, a thirty-first window beyond the count Pratik answered. With no target given the first-run question opens over the frame on a fresh profile, so main has always been the frame under a modal and the bare main window had never been scanned. Six module targets rather than five, and main left as what it is: the window a fresh profile first meets.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:49.914Z",
    "resolved_at": null
  },
  {
    "id": 394,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Seventeen dialogs open only from inside another window and are outside the scan after 06-06: the account edit dialog, Confirm Delete in the Calendar window, Check Spelling, Insert Table and Preview Before Send in the composer, the contact edit dialog and its Add Email Address, Add Phone Number, Add Address and Add Custom Field, the rule, filter, tag and signature edit dialogs, the wait-for-an-answer window, choose-from-list, and ask-for-a-name. One entry for the layer rather than one per window because these were not in the count Pratik answered on 2026-09-14, which was of windows with their own entry point; whether they are the next widening is his to say, and 06-07's list should say they are outside.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T11:33:50.327Z",
    "resolved_at": null
  },
  {
    "id": 395,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "docs/wcag-coverage.md",
    "line": null,
    "description": "06-07: fifty-two of the fifty-five WCAG 2.2 Level A and AA criteria can only be judged by a person, and nobody has walked any of them against this application. The page says what is left for a person on every row; none of it has happened. Three sentences in three windows have been heard by the NVDA suite and nothing else on any row has.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:35.418Z",
    "resolved_at": null
  },
  {
    "id": 396,
    "kind": "todo",
    "phase": "06",
    "file": "docs/wcag-coverage.md",
    "line": null,
    "description": "06-07: the applies column of the coverage table is a judgement about applicability to a Windows desktop mail client, made by reading each criterion against what a mail client does, and nobody has checked it against this application. The six no answers are the regulations' and are named; the forty-nine yes answers are one reader's. The row a person disagrees with is the row to correct, with the date.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:35.857Z",
    "resolved_at": null
  },
  {
    "id": 397,
    "kind": "todo",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "06-07: seventeen nested dialogs are outside both scan channels, named in ledger 394 and on docs/wcag-coverage.md. Every yes on the coverage page is a yes for the thirty-one windows the scan reaches and for no other, and the page says so. Not a new entry for the layer; this one says the coverage page depends on 394.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:36.275Z",
    "resolved_at": null
  },
  {
    "id": 398,
    "kind": "todo",
    "phase": "06",
    "file": "nvda-tests/README.md",
    "line": null,
    "description": "06-07 found, out of scope: the NVDA README says the package exists for two places and its What is in here table lists two test files, where four exist, three running and one skipped. Found while reading the suite for the coverage page's NVDA column. The README under-claims and nothing reads it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:36.671Z",
    "resolved_at": null
  },
  {
    "id": 399,
    "kind": "todo",
    "phase": "06",
    "file": "tests/house_style.rs",
    "line": 3348,
    "description": "06-07 found: test_no_status_page_names_a_version_the_code_does_not_ship reads a WCAG criterion number such as 1.3.1 on docs/IMPLEMENTATION_STATUS.md as a version the code does not ship. The status page names the three criteria by name to stay clear of it. A reading that cannot tell a criterion number from a version is a limitation to know about, not yet a defect worth widening the reading for; if a criterion number has to appear on a status page, that is the moment.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:37.079Z",
    "resolved_at": null
  },
  {
    "id": 400,
    "kind": "deviation",
    "phase": "06",
    "file": "scripts/check.sh",
    "line": null,
    "description": "06-07: one line added to check.sh's docs_only path, cargo test --lib presentation::what_the_scans_can_judge::, on the help_page precedent. Not in the plan's file list. Without it a commit editing only docs/wcag-coverage.md answered docs_only and ran everything except the reading that holds that page to the code, which is a guard running on every commit except the ones that could break it. Shown working by the docs commit 93f8c865, which ran the eight tests under docs_only.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:37.478Z",
    "resolved_at": null
  },
  {
    "id": 401,
    "kind": "deviation",
    "phase": "06",
    "file": "CLAUDE.md",
    "line": null,
    "description": "06-07: CLAUDE.md had two copies of the half figure where the plan counted one and said to leave it. Guardrail 2 said covers about half of WCAG across a line break, which the plan's single-line grep could not see, and was wrong the same way the four product copies were; corrected with the date and the old wording. The accessibility section's half of accessibility defects is about defects and was left standing with the qualification added, as the orchestrator asked, rather than left untouched as the plan said.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T12:38:37.877Z",
    "resolved_at": null
  },
  {
    "id": 402,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/application/due.rs",
    "line": null,
    "description": "06-09 task 1: the kind-first sentences have not been heard. Task due today: file the report; Task overdue: file the report, was due July 25, 2026; Event in 15 minutes: standup, at 3:00 PM; Event now: standup; Event started 10 minutes ago: standup; and Untitled task or Untitled event for a row with no title. Tests prove the word comes first and the forms are exact; whether the comma before at 3:00 PM reads as a pause or a list, and whether due today is help or nagging, only a listening pass under NVDA can settle.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:19:15.569Z",
    "resolved_at": null
  },
  {
    "id": 403,
    "kind": "deviation",
    "phase": "06",
    "file": "src/application/due.rs",
    "line": null,
    "description": "06-09 task 1: an overdue task says its day as a date, Task overdue: file the report, was due July 25, 2026, where the plan's example said was due yesterday. A whole day is read as a date under every style on purpose, the birthday rule in date_display, and a yesterday would be a new relative message for whole days through the catalogue. Not written here; if wanted it is one message and one arm, and the listening pass above will say whether the date form is enough.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:19:30.620Z",
    "resolved_at": null
  },
  {
    "id": 404,
    "kind": "deviation",
    "phase": "06",
    "file": "src/application/due.rs",
    "line": null,
    "description": "06-09 task 1: the identity trap's guard record is in two halves and only one is here. The same id under two kinds is two identities is measured, two red. A day of a series carrying its series' id is composed by the event feed, which task 4 writes in wx_app.rs, so there is no code in task 1 for a break to edit; the test dismisses one day and finds the next still due against a fixture that composes id and start the way the feed must. Task 4 owes the record that breaks the feed's composition.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:19:31.090Z",
    "resolved_at": null
  },
  {
    "id": 405,
    "kind": "deviation",
    "phase": "06",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "06-09 task 1: BetweenLooks.already and said_and_waiting are keyed by due::Identity instead of a reminder id string, and the reminder feed builds due::Candidate rows, in the red commit c189a361, because the crate would not build otherwise. wx_app.rs is not in task 1's file list. No test was added to wx_app.rs and its 48 records were not disturbed; the reading in tests/wired.rs still sees the insert into already before the window. Task 4 rewrites this region.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:19:31.536Z",
    "resolved_at": null
  },
  {
    "id": 406,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/date_display.rs",
    "line": null,
    "description": "06-09 task 1: how_soon, how_long_ago and time_of_day are public readings with no caller outside due::spoken, and due::spoken has no caller that hands it a task or an event yet: raise_what_is_due still feeds reminders only. Everything task 1 added is reachable by tests and by nothing a person can run until task 4 wires the feeds. Said here so the model is not read as shipped.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:19:31.989Z",
    "resolved_at": null
  },
  {
    "id": 407,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/scan_fixtures.rs",
    "line": null,
    "description": "Scan finding, Account Manager row 7: the IMAP Server cell of the made-up account is empty, a focusable list cell with no name on UI Automation. Either the fixture account names a server or the cell says none rather than nothing; decide which, because naming a server hides the shape a real account with no server would have",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:39.547Z",
    "resolved_at": null
  },
  {
    "id": 408,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 15: the text field of the Starts Day spinner has no name on either channel. set_accessible_name names the up-down arrows, and a Windows spinner is two windows; the field a person types in is the other one. What it takes: get the buddy through UDM_GETBUDDY on the spinner handle from get_handle, then IAccPropServices::SetHwndPropStr with PROPID_ACC_NAME, which the Annotation proxy carries to both channels; needs the Win32_UI_Accessibility, Win32_UI_Controls and Win32_UI_WindowsAndMessaging features. Or a visible static label before each field, which Windows gives the field on both channels and which sighted people would see too. Verified only by the next scan, since nothing here can read a live tree",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:39.988Z",
    "resolved_at": null
  },
  {
    "id": 409,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 16: the text field of the Starts Year spinner has no name on either channel; same cause and same fix as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:40.409Z",
    "resolved_at": null
  },
  {
    "id": 410,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 17: the text field of the Start time Minute spinner has no name on either channel; same cause and same fix as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:40.839Z",
    "resolved_at": null
  },
  {
    "id": 411,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 18: the element showing the current value of the Start time AM or PM list has no name on UI Automation. The list carries the name through the MSAA proxy; the value element is the platform's child of it and gets nothing. SetHwndProp on the list handle with the child id, or a visible label before the list, as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:41.287Z",
    "resolved_at": null
  },
  {
    "id": 412,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 19: the text field of the Ends Day spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:41.723Z",
    "resolved_at": null
  },
  {
    "id": 413,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 20: the text field of the Ends Year spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:42.170Z",
    "resolved_at": null
  },
  {
    "id": 414,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 21: the text field of the End time Minute spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:42.628Z",
    "resolved_at": null
  },
  {
    "id": 415,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 22: the element showing the current value of the End time AM or PM list has no name on UI Automation; as 411",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:28:43.082Z",
    "resolved_at": null
  },
  {
    "id": 416,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Scan finding, Edit Event row 23, unjudged: the Category combo box is reported as not supporting ExpandCollapse. Every combo box in the window is exposed through the MSAA proxy because each carries an accessible object of ours, and the proxy offers ExpandCollapse to none of them, yet only this one was flagged. The one difference in the tree is that it has no child showing a value. To judge it: read the scanner's condition for ControlShouldSupportExpandCollapsePattern in the v2.4.2 source, or open the window on a taller screen and scan again",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:12.669Z",
    "resolved_at": null
  },
  {
    "id": 417,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Found in the scan's tree, not a scan finding: Edit Event is 720 pixels tall on the runner's 768-pixel screen and its content runs past the bottom. Show as, Status, Category and Times offered are six pixels tall at the bottom edge; the panel inside the tab is 562 tall and the form is taller. On a small screen or at 200 percent text size the last four fields are cut off and the form does not scroll. WCAG 1.4.10 Reflow and 1.4.4 Resize Text, and a keyboard user reaches a field nobody can see",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:13.123Z",
    "resolved_at": null
  },
  {
    "id": 418,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 24: the element showing the current value of the Send on Month list has no name on UI Automation. No static label precedes any control in this window, so nothing is given to it by the platform either. As 411",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:13.548Z",
    "resolved_at": null
  },
  {
    "id": 419,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 25: the text field of the Send on Day spinner has no name on either channel. The name Send on Day is on the arrows; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:13.982Z",
    "resolved_at": null
  },
  {
    "id": 420,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 26: the text field of the Send on Year spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:14.419Z",
    "resolved_at": null
  },
  {
    "id": 421,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 27: the text field of the Send at Hour spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:14.853Z",
    "resolved_at": null
  },
  {
    "id": 422,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 28: the text field of the Send at Minute spinner has no name on either channel; as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:15.269Z",
    "resolved_at": null
  },
  {
    "id": 423,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_send_later.rs",
    "line": null,
    "description": "Scan finding, send-later row 29: the element showing the current value of the Send at AM or PM list has no name on UI Automation; as 411",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:15.699Z",
    "resolved_at": null
  },
  {
    "id": 424,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Found in the scan's tree, no rule fired: the text field of the Start time Hour spinner is named Start time on UI Automation, the text of the static label before it, which the platform gives to an unnamed edit; the arrows beside it say Start time Hour. On MSAA it has no name and the walk reported it. A name that is present and wrong is one no scanner rule sees; only a person hears it. Fix as 408",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:16.128Z",
    "resolved_at": null
  },
  {
    "id": 425,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_item_form.rs",
    "line": null,
    "description": "Found in the scan's tree, no rule fired: the text field of the End time Hour spinner is named End time on UI Automation and nothing on MSAA; as 424",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:16.558Z",
    "resolved_at": null
  },
  {
    "id": 426,
    "kind": "todo",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "Scan-level finding 2: the count takes only the first summary a window prints, Select-Object -First 1, so a window that writes two result files has its second count dropped. The reader window wrote two on 2026-09-14, both clean, so nothing was lost that day. Not fixed because nothing in the tree can run that block to prove a sum; a test would need the counting moved into a script under scripts with a suite of its own, run through pwsh, before the line is changed",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:16.999Z",
    "resolved_at": null
  },
  {
    "id": 427,
    "kind": "todo",
    "phase": "06",
    "file": "scripts/msaa-names.ps1",
    "line": null,
    "description": "Scan-level finding 3: the six module targets are one scan repeated. Each walked one window of 1797 elements on MSAA and each msaa-names.json holds the same 91 distinct names, All Calendars, All Contacts, All Notes and Body in Markdown among them in all six. Every module panel is in the window whichever is showing and the walk reads no state, so it cannot tell a hidden panel from the shown one; a nameless control in a hidden panel would be reported six times and a module target proves nothing about its module. What it takes: read accState per element and skip a subtree whose state has STATE_SYSTEM_INVISIBLE, printing how many were skipped so a hidden panel stays visible in the log; the ps1 maps to no gate target, so the change wants a case in a shell suite first",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:17.438Z",
    "resolved_at": null
  },
  {
    "id": 428,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/html_renderer.rs",
    "line": null,
    "description": "The scan reached no rendered message. The preview in the main window is a WebView2 control and no target's tree held its document: a fresh profile has no message and the reader target opens a rich edit window. The rendered message is where a sender's headings and links have to survive, and it is the page with the same shape as the editor's, a document with no title loaded from its own address, so its name is likely its own source too. Nothing about it was judged; a target that opens the preview on a made-up message would put it in the next run",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:17.882Z",
    "resolved_at": null
  },
  {
    "id": 429,
    "kind": "todo",
    "phase": "06",
    "file": ".github/workflows/accessibility.yml",
    "line": null,
    "description": "Scan-level finding 1: the run summary of 2026-09-14 said 26 findings where the scan log held 29, because the counting pattern matched errors were found and Axe prints 1 error was found; filters, tags and signatures each printed one and were recorded as clean. Fixed the same day, the pattern reads all three sentences and a test in scan_target.rs reads it from the workflow and holds it to them",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-14T15:29:28.029Z",
    "resolved_at": "2026-09-14T15:29:28.465Z"
  },
  {
    "id": 430,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "docs/manual-accessibility-pass.md",
    "line": null,
    "description": "The manual accessibility pass is written and has not been walked. Seventy-six items across six categories, each naming its source and the technology it needs, planned for after phase 8. Until a person has dated every item, nothing in the program has had the pass, and the page says so at the top",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:36:13.798Z",
    "resolved_at": null
  },
  {
    "id": 431,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "The Due now window has never been heard. Three rows of three kinds in one ListBox, each sentence beginning with its kind's word, the list named by its count, the sentence for several rows as a count then three rows then how many more, and the tone repeating over a list rather than a line: the live test reads the rows and the labels back from the real controls, and no person has listened to any of it with NVDA or Narrator. Whether 'Task due today: File the report' reads well by ear, and whether 'Event now' for an all-day event at the working-day hour is heard as help, is a listening pass",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:45:39.688Z",
    "resolved_at": null
  },
  {
    "id": 432,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "Mark Done on an event row and Details on a task or reminder row are disabled with the reason in the label, 'Mark Done: not for an event', 'Details: not for a task yet'. Whether a person tabbing past a disabled button hears why is not known: Windows skips a disabled control in the tab order, so the label may be read only by arrowing or by a screen reader's review mode, the same question ledger 375 asked of the account window",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:45:40.172Z",
    "resolved_at": null
  },
  {
    "id": 433,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "The list and the Come back in picker are named through set_accessible_name, which writes MSAA only; on UI Automation both are named by whatever Windows falls back to, the static text beside the picker and nothing beside the list. 06-02 recorded the same for the settings picker and nobody has checked either channel with the scan or with Narrator. The scan target 'reminder' now opens on three rows so the next Accessibility run reaches the list and all six buttons",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:45:40.688Z",
    "resolved_at": null
  },
  {
    "id": 434,
    "kind": "todo",
    "phase": "06",
    "file": "src/application/event_alerts.rs",
    "line": null,
    "description": "Under decision 2 of 06-09, the default lead fills silence: an event whose stored alerts say nothing is raised default_reminder_minutes before its start. Two kinds of event are therefore given a lead their own calendar may not want. A Google event on the calendar's default alert, which is most of them, gets this program's default rather than the calendar's, because the calendar's default is never read. And every CalDAV event gets the default whatever its VALARM says, because no reader parses a TRIGGER, so a CalDAV event whose alarm was switched off alerts here. The way through is the plan's option 4: the Google pull reads the calendar's default and the CalDAV pull reads TRIGGER, each a sync path with its tests; calendar.rs has 72 records, so that is a plan of its own",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:03.202Z",
    "resolved_at": null
  },
  {
    "id": 435,
    "kind": "todo",
    "phase": "06",
    "file": "src/application/calendar.rs",
    "line": null,
    "description": "An alert switched off here is stored as off, an empty list, and the Google push still sends nothing for it, so Google keeps whatever alert it held. local_to_google_event filters an empty override list to no reminders field on purpose, because an alert it could not read must not become 'never alerts'; an explicit off is not that case and could be sent as useDefault false with no overrides, which is exactly what Google means by off. Not changed in 06-09 because it is a sync path with a test in calendar.rs, 72 records",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:03.679Z",
    "resolved_at": null
  },
  {
    "id": 436,
    "kind": "todo",
    "phase": "06",
    "file": "src/application/due.rs",
    "line": null,
    "description": "Decision 1 of 06-09 puts a date-only task and an all-day event's alert base at working_day_starts. A reminder set for a day with no time is still due at midnight, the whole-day arm of local_instant, as it has been since reminders first went off, so the two day-shaped things disagree about what hour a day is. Left alone in 06-09 because it is a behaviour change to reminders nobody asked for; whoever decides they should agree changes the reminder feed's raise_at to when_a_day_alerts and ledgers nothing",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:04.154Z",
    "resolved_at": null
  },
  {
    "id": 437,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "The Details button opens only an event. Pratik's answer of 2026-09-14 asked for it on 'the task/event/reminder', and nothing in this program edits an existing task or reminder: PimCommand has no Edit, the item form's Prefill is filled only by filled_from_calendar_item, and no writer takes a Filled onto an existing TaskEntry or ReminderEntry. On those rows the button is disabled with 'not for a task yet' in its label. What it takes: a filled_from_task_item and a task_with_edits on the shape of the event's pair, the same for reminders, and a writer for each; the button then needs only an arm in TheEditors::has_one_for",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:04.638Z",
    "resolved_at": null
  },
  {
    "id": 438,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "One look at what is due was timed on this profile only: four reminders, no accounts, an almost empty calendar, 3 candidates in 1 ms and then 0 ms, on the poll once a minute. The calendar's own read cost 68 ms bounded on a six year calendar; the event feed reads three days through the same two seeking queries per source, so it should be far below that, but no real-sized calendar has been under it. If it is not small against the fifty-millisecond tick, the plan's fallback is a startup read refreshed by TasksLoaded and CalendarEventsLoaded, the way reminders are read",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:22.636Z",
    "resolved_at": null
  },
  {
    "id": 439,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "The Due now window closing when its last row is answered, and the next row being selected after one goes, live in a button handler and after_a_row_has_gone, which no test can press: wxdragon 0.9.17 raises no widget event from outside. A break there reddens nothing, so no guard record was written for it and the live test does not claim it. The pure bookkeeping in Rows is guarded; the last-row close is a listening pass",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:23.098Z",
    "resolved_at": null
  },
  {
    "id": 440,
    "kind": "todo",
    "phase": "06",
    "file": "src/presentation/wx_reminder_alert.rs",
    "line": null,
    "description": "'1 thing due', '3 things due' and 'And 2 more' are English plurals written in code, the shape 06-03 retired for the date wording by putting it through the catalogue with Fluent's plural rules. Not put through the catalogue in 06-09 because it holds one area, dates.ftl, and its loader and completeness reading are written for one file; a second area, due.ftl, is the catalogue's next step and touches catalogue.rs, six records. When it arrives these three sentences and the window's button labels are the first to move",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:23.567Z",
    "resolved_at": null
  },
  {
    "id": 441,
    "kind": "todo",
    "phase": "06",
    "file": "src/application/event_alerts.rs",
    "line": null,
    "description": "Two small holes in what off means, said rather than widened. Microsoft's isReminderOn true with reminderMinutesBeforeStart 0 is stored as nothing, as before, so an Outlook alert at the start of the event gets this program's default lead instead of a lead of nought. And alerts_with_the_first_at with nothing stored and nought in the box keeps nothing stored, which is right, and has no test of its own because managers.rs has 50 records and one more test there is hours of remeasure; the two flipped tests pin the other branches",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T17:46:24.026Z",
    "resolved_at": null
  },
  {
    "id": 442,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/check.sh",
    "line": null,
    "description": "plan 08-01 task 1 required check.sh --suites-for guards/guards.toml docs/development/measurements.md to print the new target once its guard record existed; it prints nothing, because the coupling function drops a candidate already in guards_that_read_the_whole_tree on purpose, since every scoped run ends with that list. The record couples: a copy of the script with the target taken out of the list answers with it. The criterion asked for output the tool suppresses by design, and a target in both lists is answered by the whole-tree list first",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T20:41:00.106Z",
    "resolved_at": null
  },
  {
    "id": 443,
    "kind": "todo",
    "phase": "08",
    "file": "scripts/guards.py",
    "line": null,
    "description": "the docstring of run_the_whole_suite still says the rebuild a break forces is 23 seconds and the library is 89, a pair taken before the suite was halved on 2026-09-09, and the arithmetic built on it, a 220-record sweep from 6.8 hours to 88 minutes, is built on both stale terms; the runner now prints today's terms on every run and the rate row on docs/development/measurements.md holds them, so 08-06's pass over comments in scripts should point this sentence at the page rather than leave a fifth sweep figure in the tree",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-14T20:41:09.046Z",
    "resolved_at": "2026-09-15T03:20:43.920Z"
  },
  {
    "id": 444,
    "kind": "todo",
    "phase": "08",
    "file": "docs/privacy.md",
    "line": 417,
    "description": "the privacy page's update download size is a target of about 12 MB and not a measurement, because no release has been published to measure; once one exists, measure the installer, write the size on the page with its date, and add a row to docs/development/measurements.md",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T21:45:39.956Z",
    "resolved_at": null
  },
  {
    "id": 445,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/integration-guide.md",
    "line": 5,
    "description": "the agreement reading holds every figure shaped N tests on the three test-count pages to a row on the measurements page and cannot tell a past count from a present one, so the guide's historical 'counted 64 tests' was reworded to 'put the count of tests at 64' and the convention (a past count on those three pages is not written as N tests) lives in the reading's section comment rather than anywhere a page author would meet it first",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T22:05:28.433Z",
    "resolved_at": null
  },
  {
    "id": 446,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "tests/the_numbers_the_targets_ask_for.rs",
    "line": null,
    "description": "The measurement profile's account points at 127.0.0.1 on a closed port so a startup connection would be refused at once, and the connection was never attempted: nothing in the program checks mail on a schedule, so a start dials nothing and the log of every measured run held no WARN or ERROR line. What the application says or shows when a connection is refused was therefore read by nobody in 08-03, and the cold-start and idle figures are for a start that never touches a server.",
    "status": "fixed",
    "reason": "Fixed by 10-06 on 2026-09-18: a start checks every enabled account and asks for a watch on each, so the measurement account is attempted at once, and the log of every one of the five re-taken runs was read. It is refused before the port is dialled, by the credential store and not the socket, because the profile stores no password and an empty password cannot be stored (ledger 374): \"Could not watch the inbox of The measurement account: Authentication error: No password is saved for The measurement account on this computer\", then \"Trying the watch again in 30 seconds\", a second refusal at about 40 s, a third at about 100 s, and \"left to the schedule alone until TheServerAnswersAgain\". What is said on screen is \"Error: Authentication error: No password is saved\" at High, once per attempt of the check. The cold-start and idle rows are re-taken under that condition on docs/development/measurements.md. What a refusal at the socket says is ledger 526.",
    "recorded_at": "2026-09-14T23:50:37.822Z",
    "resolved_at": "2026-09-18T04:51:18.000Z"
  },
  {
    "id": 447,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "docs/development/measurements.md",
    "line": null,
    "description": "Idle memory on the measurements page is idle with a refusable account that was never dialled. Idle on a machine with a real account and a live connection is a different idle: a folder watch, a sync on new mail and a live WebView2 preview all run then and none ran here. The 120 s reading of 391 MB, of which the application process is 56 MB, says nothing about that case, and no real account has ever been used with this program to take it. Corrected on 2026-09-18 by 10-06: idle now includes a start that checks the account and asks for its watch, a watch refused three times and tried again after a growing wait, and the account left to the schedule alone, and the rows were re-taken under that condition; still no live connection, because the measurement account is refused by the credential store before anything is dialled, so the case this entry names, a watch held open on a real server and a sync on new mail, is still untaken.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T23:50:38.308Z",
    "resolved_at": null
  },
  {
    "id": 448,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/development/measurements.md",
    "line": null,
    "description": "The cold-start row reports the first start after the binary was built on its own as the file-cache-cold figure, 520 ms against the series median of 476 ms. That figure depends on what else the machine had read: the linker had just written the binary so much of it was in the file cache already, and WebView2's own binaries were warm from the earlier runs. A start after a reboot with the disk cold was not taken and would be a different number; the definition says what was measured and this entry says what it does not cover.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T23:50:38.786Z",
    "resolved_at": null
  },
  {
    "id": 449,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "tests/the_list_at_two_hundred_thousand_rows.rs",
    "line": null,
    "description": "The sort's apply is not timed: apply_sort clones the rows, sorts them off the interface thread and sends MessagesLoaded, and the cost of the list control taking 200,000 rows back needs a window the harness does not have. The sort rows on the measurements page say so; a number for the apply waits for a harness that drives the running program.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T01:35:59.293Z",
    "resolved_at": null
  },
  {
    "id": 450,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "tests/the_list_at_two_hundred_thousand_rows.rs",
    "line": null,
    "description": "A scroll's own paint is not timed: the page paint row is text_for over one page of every inbox column, and wxWidgets' painting of those cells needs a window the harness does not have. A scroll in the running program is the row's figure plus that, and the measurements page says so beside the row.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T01:36:09.797Z",
    "resolved_at": null
  },
  {
    "id": 451,
    "kind": "todo",
    "phase": "08",
    "file": "tests/the_list_at_two_hundred_thousand_rows.rs",
    "line": null,
    "description": "THE_SEARCH_BOXES_LIMIT copies the LIMIT inside managers::search_messages, which is private to that function, so the filter rows are timed at 500 because the harness says 500 and not because it read the program. If the search box's limit moves, the harness times the old one; making the constant reachable from the harness is the fix.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T01:36:10.285Z",
    "resolved_at": null
  },
  {
    "id": 452,
    "kind": "deviation",
    "phase": "08",
    "file": ".planning/REQUIREMENTS.md",
    "line": null,
    "description": "PERF-05's [S] line and roadmap criterion 3 attribute low coverage to service/protocols, service/oauth and the provider clients; on 2026-09-14 those read 92.06%, 84.55% and 96.75% against a library at 83.34%, so the attribution names areas that are no longer low; 08-06 corrects the evidence line and 08-09 closes the clause with the reason",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-15T02:14:27.761Z",
    "resolved_at": "2026-09-16T07:40:27.988Z"
  },
  {
    "id": 453,
    "kind": "todo",
    "phase": "08",
    "file": "docs/development/measurements.md",
    "line": null,
    "description": "The low coverage area on 2026-09-14 is the wxWidgets windows, src/presentation/wx_*.rs at 26.88% holding 73% of the missed lines, outside the three areas PERF-05 attributes and not attributed by 08-05; these files build windows that no --lib test opens, and 08-09 decides whether that is a gap to close, a different command to measure with, or a figure to accept with the reason beside it",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-15T02:14:28.223Z",
    "resolved_at": "2026-09-16T07:40:28.519Z"
  },
  {
    "id": 454,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/guards.py",
    "line": null,
    "description": "08-06 edited the docstring of run_the_whole_suite, which the plan's file list did not name, because ledger 443 had assigned it to 08-06 and the plan's own done criterion is that no comment in the tree states the sweep's cost as a figure; the plan was written from the research and the README, neither of which carried the entry, so an assignment written into the ledger alone did not reach the plan it was addressed to",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T03:20:52.629Z",
    "resolved_at": null
  },
  {
    "id": 455,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/IMPLEMENTATION_STATUS.md",
    "line": null,
    "description": "Three dated quotations of a retired figure were reworded to keep the figure without the phrase, because the plan's acceptance criteria were single-line greps for the old phrase finding nothing while its rule 1 requires the old figure kept as the figure of its date: the status page's mutation sentence, guards.toml line 40 and guards.sh line 36 now say the run was put at two days or at one or two hours rather than quoting the words; the meaning is unchanged and CLAUDE.md, whose criterion admitted a dated sentence, quotes all four phrases as written; an absence criterion over prose that also requires the quotation has to be scoped to the sentence and not to the phrase",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T03:21:02.849Z",
    "resolved_at": null
  },
  {
    "id": 456,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/guards.py",
    "line": null,
    "description": "08-07 task 1: --resume is refused without --log, where the plan only said a bare --resume takes the --log path; a run that records its verdicts nowhere cannot itself be resumed, so the refusal prints the flag to add rather than measuring and losing the result",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T04:28:16.036Z",
    "resolved_at": null
  },
  {
    "id": 457,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/guards.py",
    "line": null,
    "description": "08-07 task 1: a build that starts and finishes inside one record's run is seen by neither the poll before the record nor the poll after it, so such a record is measured beside a build and not marked contended; the plan accepts this as the cheapest reading the log can carry and the changelog says so",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T04:28:16.506Z",
    "resolved_at": null
  },
  {
    "id": 458,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/mutants.sh",
    "line": null,
    "description": "08-08: the every-target mutation shape cannot run in a scratch copy, because cargo mutants copies the tree without .git and test_the_share_of_history_before_red_green_is_computed_and_printed runs git merge-base, so that baseline fails in the copy; the plan asked for a scratch-copy rate under every target and it was measured in place instead, which is the only way it runs",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T08:19:55.953Z",
    "resolved_at": null
  },
  {
    "id": 459,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/mutants.sh",
    "line": null,
    "description": "08-08: a scoped mutation run, the third option the checkpoint offers, has no shard or resume support: --shard and --shards divide the whole list and pass nothing to -f, so a run over one area today is the older scripts/mutants.sh DIR mode, one process a kill loses whole; if option 3 or 4 is chosen, the shard modes gain a --file glob first, red first",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T08:19:56.464Z",
    "resolved_at": null
  },
  {
    "id": 460,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/development/measurements.md",
    "line": null,
    "description": "08-08: the per-mutant rate rows come from one shard whose 25 mutants all sit in src/presentation/accessibility.rs, a file most of the presentation layer depends on, so each rebuild ran 58 to 75 s where a one-file change elsewhere rebuilt 44 to 46 s; the products are what the tree would cost if every file were that file, and no leaf module was measured",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T08:19:56.948Z",
    "resolved_at": null
  },
  {
    "id": 461,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/development/measurements.md",
    "line": null,
    "description": "08-08, for 08-09: the whole-tree mutation run is deferred by Pratik's answer of 2026-09-15; 12,391 mutants at 2847391c, 130 s a mutant under every target in place and 194 s a shard over 496 shards, about 19.8 days of this machine, or two dispatches of 248 runners at an unmeasured rate; the run over src/service/protocols, 450 mutants in 18 shards, is what this milestone makes, and criterion 4 is revised under criterion 6 with 08-08's product table as the reason",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T10:58:17.286Z",
    "resolved_at": null
  },
  {
    "id": 462,
    "kind": "deviation",
    "phase": "08",
    "file": ".github/workflows/mutants.yml",
    "line": null,
    "description": "08-08, for 08-09: mutants.yml ran on pull_request only, and this project merges to main without pull requests, so the diff-scoped mutation check in CI had never run once since it was written; a workflow_dispatch with mode=diff and a since ref now exists, and it has not been dispatched",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T10:58:17.811Z",
    "resolved_at": null
  },
  {
    "id": 463,
    "kind": "deviation",
    "phase": "08",
    "file": "scripts/guards.sh",
    "line": null,
    "description": "08-08: the guard sweep could be sharded onto GitHub's runners the way the mutation shards now are, one runner per chunk of records with --stop-after and the log as the artifact, and it is not; the sweep still runs on this machine from 08-07's checkpoint",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-15T10:58:18.330Z",
    "resolved_at": "2026-09-15T11:45:40.115Z"
  },
  {
    "id": 464,
    "kind": "deviation",
    "phase": "08",
    "file": ".github/workflows/ci.yml",
    "line": null,
    "description": "08-08: found by the push of main at 0fa393ba on 2026-09-15, CI run 34956059032, Test Suite job 104338271778: test_the_share_of_history_before_red_green_is_computed_and_printed failed on the runner because the checkout fetched one commit and git merge-base could not see 18a02454; fixed at abf3e24c with fetch-depth 0 and a reading that holds every job running cargo test to it; the fix is unconfirmed on a runner until the next push",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T10:58:18.818Z",
    "resolved_at": null
  },
  {
    "id": 465,
    "kind": "deviation",
    "phase": "08",
    "file": ".github/workflows/guards.yml",
    "line": null,
    "description": "08-07, the answer: WIXEN_TEST_THREADS is left at the script's default of 8 on a 4-core runner, unmeasured, so the runner's timing lines and the rate row on docs/development/measurements.md read against each other at one setting; the thread curve on CLAUDE.md was taken on 24 cores and says nothing about 4, and verdicts are what the sweep is for. Re-take the curve on a runner if the shards run slower than the sizing guess",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T11:45:40.600Z",
    "resolved_at": null
  },
  {
    "id": 466,
    "kind": "deviation",
    "phase": "08",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "debug theme-reach-crashes-on-runner, 2026-09-15: wxWidgets 3.3.2 (wxdragon 0.9.17) delivers WebView2 creation completions to a destroyed control (wxWidgets #26491, fixed upstream for the unreleased 3.3.4). presentation::browser_ready now holds Compose and Preview Before Send until their browsers report. The main window's preview pane (wx_app.rs preview WebView) and the conversation-as-headings frame are destroyed only at application exit; an exit inside the creation moment, about 250 ms warm and seconds after a runtime update, ends the process with 0xc000041d after the window is gone and nothing else is wrong. Left as it is until a wxdragon release vendors 3.3.4, at which point browser_ready can be retired",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T12:50:15.759Z",
    "resolved_at": null
  },
  {
    "id": 467,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "tests/theme_reach.rs",
    "line": null,
    "description": "debug theme-reach-crashes-on-runner, 2026-09-15: the crash of theme_reach on GitHub's runners (runs 34956059032 and 34961574447, exit 0xc000041d) was reproduced here only through a scratch case tearing a WebView down at once, never through theme_reach itself, whose two browsers finish in a quarter of a second on this machine. The wait added to theme_reach and the child-process test in closing_a_window_before_its_browser_exists are unconfirmed on a runner until the next push of main; the CI Test Suite job is the confirmation",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-15T12:50:16.261Z",
    "resolved_at": null
  },
  {
    "id": 468,
    "kind": "unmet-truth",
    "phase": "08",
    "file": "src/application/contacts_sync.rs",
    "line": null,
    "description": "08-07 task 3: the gate the_copy_here_was_written_here on the note call in the read that skips a contact the setting holds back is covered by no test since ce3e89ba of 2026-09-05 rewrote the one test that reached it; the record named for it reddened nothing on the runners and here and was retired. A test that notices belongs in contacts_sync.rs, which 77 records name, so it waits for somebody prepared to re-measure those",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T01:54:07.526Z",
    "resolved_at": null
  },
  {
    "id": 469,
    "kind": "todo",
    "phase": "08",
    "file": "src/application/caldav_sync.rs",
    "line": null,
    "description": "08-07 task 3: the seen_uids insert of the compound id at the top of one_caldav_day_kept_out_of_its_series is redundant, because the loop over the server's answer already marks the changed day's stored identity seen before the day is folded; taking it out reddens nothing on the runners or here. Its record now breaks the marking that holds the rule; the insert is a dead-code candidate to remove with a reading that says why",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T01:54:08.175Z",
    "resolved_at": null
  },
  {
    "id": 470,
    "kind": "deviation",
    "phase": "08",
    "file": "guards/guards.toml",
    "line": null,
    "description": "08-07 task 3: two records are right on this machine and blind on a runner, and are left as written. An hour with no zone means an hour here: the runner's clock is UTC, so a break that sends the local hour as UTC changes nothing there. The walk into Windows own chain structures really happens: the runner has no chain to walk. A runner sweep reports both as a named test staying green; read them as measured here, or give each a fixture that does not depend on the machine",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T01:54:08.848Z",
    "resolved_at": null
  },
  {
    "id": 471,
    "kind": "unmet-truth",
    "phase": "08",
    "file": "src/application/caldav_sync.rs",
    "line": null,
    "description": "08-07 task 3: the rule that a changed day of a CalDAV series is marked seen before the removal pass can run has no test that would notice it broken: taking out either seen_uids marking leaves test_syncing_a_caldav_moved_day_twice_does_not_duplicate_it_or_delete_it green, measured on the runners and here. The record of 2026-08-14 that named it was renamed to the one fact the break really guards. What keeps the moved day now was not reconstructed; a test that breaks when the marking goes is the answer",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T02:33:25.593Z",
    "resolved_at": null
  },
  {
    "id": 472,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: imap.rs:494:9 replace ImapStream::into_plain with None survived; the one caller is the STARTTLS upgrade, which no loopback test reaches because no test server here speaks TLS; needs a loopback server with a certificate",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:05.804Z",
    "resolved_at": null
  },
  {
    "id": 473,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: imap.rs:527:9 replace poll_flush with Poll::from(Ok(())) survived; equivalent on the plain stream, where tokio's TcpStream::poll_flush is always ready, and untested on the TLS stream, where a skipped flush leaves the record layer's buffer unsent",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:06.311Z",
    "resolved_at": null
  },
  {
    "id": 474,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/imap.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: imap.rs:534:9 replace poll_shutdown with Poll::from(Ok(())) survived; a skipped shutdown is invisible to a loopback test that drops the socket and matters on TLS, where the close-notify never goes out",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:06.819Z",
    "resolved_at": null
  },
  {
    "id": 475,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/pop3.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: pop3.rs:99:9 replace Pop3Stream::into_plain with None survived; the STLS upgrade, the same gap as IMAP's, needing a loopback server that speaks TLS",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:07.322Z",
    "resolved_at": null
  },
  {
    "id": 476,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/pop3.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: pop3.rs:132:9 replace poll_flush with Poll::from(Ok(())) survived; equivalent on the plain stream and untested on TLS, the same as IMAP's",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:07.799Z",
    "resolved_at": null
  },
  {
    "id": 477,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/protocols/pop3.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour: pop3.rs:139:9 replace poll_shutdown with Poll::from(Ok(())) survived; the same as IMAP's",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:08.277Z",
    "resolved_at": null
  },
  {
    "id": 478,
    "kind": "deviation",
    "phase": "08",
    "file": "src/service/caldav.rs",
    "line": null,
    "description": "08-08 mutation run survivor, untested behaviour and the runner shape of ledger 470: caldav.rs:202:9 replace CalDavClient::for_account with Default::default() survived; the constructor reads this machine's stored settings through ConfigManager::load_stored, so a test asserting the allowed client passes where the settings allow it and reads wrong on a runner with no settings file; the constructor wants its answer as an argument before it can be pinned",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:08.779Z",
    "resolved_at": null
  },
  {
    "id": 479,
    "kind": "deviation",
    "phase": "08",
    "file": "docs/plans/20260915-whole-tree-mutation-run.md",
    "line": null,
    "description": "08-08 mutation run, six survivors recorded as equivalent with the reason on the page rather than killed: mailbox_name.rs:193:5 and 251:30, caldav.rs:1097:20, 1687:18, 1789:25 and 1812:35; two of them are equivalences the code's own comments already claimed and the run has now confirmed; recorded so the next round does not triage them again from scratch",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T06:53:09.266Z",
    "resolved_at": null
  },
  {
    "id": 480,
    "kind": "unrun-verify",
    "phase": "08",
    "file": ".planning/REQUIREMENTS.md",
    "line": null,
    "description": "08-09: the provider question PERF-03's title asks, a real mailbox of 100,000 messages or more from a live provider, has not been asked, because no real account has ever been used here; the list question is answered by 200,000 synthetic rows on the measurements page, and the requirement's third [D] line says the provider question waits for a live account. docs/roadmap.md and docs/development/requirements-backlog.md carry the line half answered. When an account exists, open a mailbox of that size, and time the first listing, a search and a sort from the running program rather than the harness.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T07:40:18.005Z",
    "resolved_at": null
  },
  {
    "id": 481,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "08-09: nothing this phase changed that a person meets has been confirmed with a screen reader. 08-03 made the window fill the module it opens on at startup, so the folder tree and the message list are there on a fresh start instead of after a mail check or a module switch, and nobody has heard what NVDA or Narrator says at that moment or whether focus lands somewhere useful; the usable line the harness reads is a log line and nothing speaks it, by design. The check is one item for docs/manual-accessibility-pass.md: start the release build against a profile with mail in it and listen to the first thing said.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T07:40:18.547Z",
    "resolved_at": null
  },
  {
    "id": 482,
    "kind": "deviation",
    "phase": "08",
    "file": ".planning/REQUIREMENTS.md",
    "line": null,
    "description": "08-09: PERF-01 and PERF-04 are ticked on a reading, not on a number that meets the target either way. The application process is 57 MB with 1,000 cached messages and 56 MB idle, under 150 MB and 100 MB; the six WebView2 processes Windows runs for the preview pane weigh about 333 MB beside it, so the sum is 390 MB and 391 MB and misses both. The targets were written before the preview was a browser and do not say whether they count it. The coordinator's reading, put to Pratik on 2026-09-14 and not contradicted, is the application process alone, and both boxes are ticked on it with the tree's weight written beside the target wherever it is judged. If Pratik reads the target as the sum, untick PERF-01 and PERF-04, change the two [D] lines added 2026-09-16, and revise the targets or the preview.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T07:40:19.070Z",
    "resolved_at": null
  },
  {
    "id": 483,
    "kind": "unrun-verify",
    "phase": "09",
    "file": ".github/workflows/release.yml",
    "line": 118,
    "description": "The as-is level has never been dispatched: that cargo-release 1.1.5 given its current version on the runner plans no bump, skips the commit on a clean tree and tags v1.0.0-alpha.1 is read from a dry run on this machine and from commit_all in its ops/git.rs, not from a run; the first as-is dispatch is Pratik's and is what proves it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T14:30:27.270Z",
    "resolved_at": null
  },
  {
    "id": 484,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/service/spellcheck/mod.rs",
    "line": 349,
    "description": "Whether a profile created before 2026-09-03, holding the bare en every profile got then, now shows English (United States) in Settings and is checked in it without a hand change is a run on such a profile; the tester's own profile has held a hand-set en-US since 2026-09-15, which the resolver leaves as stored, so his machine cannot show the fix and only a profile still holding the bare value can. 09-02 proved it on this machine through the real General tab built in a test, not on a profile",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T17:19:56.885Z",
    "resolved_at": null
  },
  {
    "id": 485,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/data/message_cache/bodies.rs",
    "line": 690,
    "description": "The once-only pass that puts stored snippets right has run against a temp profile holding one HTML-only message (1 row in 3 ms, log line quoted in 09-02's summary) and never against the tester's 20 MB cache of 12,872 messages; how many of his rows it rewrites, how long his first start takes, and whether his rows then read as words are his first open of the next build to answer, and the log line says the first two",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T17:19:57.441Z",
    "resolved_at": null
  },
  {
    "id": 486,
    "kind": "deviation",
    "phase": "09",
    "file": "src/application/answered_meetings.rs",
    "line": 162,
    "description": "09-03: a meeting answer is filed on the calendar the moment Accept, Tentative or Decline is pressed, while the reply is still held for ten seconds like any other message. Undo Send inside the hold takes the reply back and leaves the meeting on the calendar as answered; answering the same meeting again replaces the entry, so somebody who undoes and answers differently ends with the calendar right, and somebody who undoes and does not answer has an entry the organiser never heard about. Filing only when the queue drains needs the send loop to reach back to the calendar, which nothing does, and is a feature of its own. Said in the changelog under Known limitations. Ledger 155's listening question, whether anything spoken after a mistaken Accept points at Undo Send, stays open: the sentence now says Undo Send takes it back, and nobody has heard it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T18:19:35.079Z",
    "resolved_at": null
  },
  {
    "id": 487,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/wx_app.rs",
    "line": 14212,
    "description": "09-04: the look the plan asked for at the running program, sort by sender from View, Sort and then by date from a column header and open the submenu, was not made. tests/one_sort_is_checked_on_a_live_menu.rs asks a real menu bar the same question by id, one group answers one tick and the old shape four from the moment it is built, and tests/one_sort_is_checked.rs holds the application's chain to one group; what neither reaches is the application's own menu after a real header click through sync_sort_menu, which is the next build's View, Sort to answer, and it is a look rather than a listening pass",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T19:36:32.825Z",
    "resolved_at": null
  },
  {
    "id": 488,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/wx_settings.rs",
    "line": 1264,
    "description": "09-04: whether the Reading tab now reads as one group by ear, Default sort order and then Then by as consecutive tab stops under NVDA, is FOUND-06's listening line and has not been listened to. tests/the_sort_controls_sit_together.rs builds the real dialog and reads the sibling chain, which is the order Tab moves in, and finds only Then by's own label between the two; that is structure present, and the tester who reported #36 is the one who can say whether it is experience good, along with whether Cc and Bcc lines is where he would look for it on the Compose tab",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T19:36:44.529Z",
    "resolved_at": null
  },
  {
    "id": 489,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/scan_target.rs",
    "line": 232,
    "description": "09-05: the five editors are scan targets and were each opened here on a throwaway profile and seen (Edit Contact, Edit Condition, Edit Filter Rule, Edit Signature, Edit Account, each owned above the frame), and the MSAA walk on each, run once before task 2 and once after as the machine is, left with -1073740791 every time, ten runs, as ledger 390 records, NVDA running and the cause not diagnosed; so the names of their checkboxes on the channel NVDA reads have been read by nothing, the before and after rows #42 asks for do not exist, and roadmap criterion 5's walk clause and FOUND-08's second [D] line stay open until the Accessibility workflow has walked the five at the next push. What was read instead is the Win32 child list of each window, which shows the empty statics gone and is not a name",
    "status": "fixed",
    "reason": "Fixed by 11-02 on 2026-09-18: the Accessibility workflow walked the five editors in run 35336142914 on main at 744d05ef, the push of that morning. The log reads Walked 'Edit Contact', 'Edit Condition', 'Edit Filter Rule', 'Edit Signature' and 'Edit Account', each with 'Wixen Mail', and the MSAA walk on each ended '0 without a name' (2915, 1978, 2126, 1889 and 3120 elements). FOUND-08's second [D] line is closed on that run. The walk still crashes on this machine, ledger 390, and nothing here ran it",
    "recorded_at": "2026-09-16T21:25:20.597Z",
    "resolved_at": "2026-09-18T13:30:00.000Z"
  },
  {
    "id": 490,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/wx_managers.rs",
    "line": 67,
    "description": "09-05: what NVDA says on the signature editor's Default signature box and the contact editor's Favorite box after the change is FOUND-08's last [S] line and has not been heard. Both boxes, and three more built the same way, now carry set_accessible_name with the mnemonic stripped and no empty static text before them; tests/checkbox_labels.rs reads the built windows and finds an accessible object on every one of the fifteen editor checkboxes and no nameless static before any, which is structure present. Why the tester heard the two unnamed is still not settled by anything read here: a native checkbox carries its own window text and the MSAA walk that would say what the channel reported could not run. The next instrument is NVDA's own log on his machine, Tab through the signature editor with the log at debug, if the next build still reads the box as unnamed",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T21:25:36.495Z",
    "resolved_at": null
  },
  {
    "id": 491,
    "kind": "deviation",
    "phase": "09",
    "file": "tests/no_label_is_only_a_space.rs",
    "line": 60,
    "description": "09-05: the plan's premise 5 said the widened reading would catch all eleven spacers and none of the eleven filled lines with a rule about which calls take the binding later; that rule flagged three filled lines (a static handed on bare from a block, one handed to the dialog's own filler, the live region written through Win32) and would have needed an allow list holding filled lines rather than what task 2 left. The rule written instead refuses a binding whose only later use is a sizer add, which on the tree at 1d934e26 refuses exactly the five in wx_managers.rs and none of the filled lines, and does not see a spacer handed on in a tuple to be stored and hidden, the account editor's old shape for its six; the module comment says so, tests/checkbox_labels.rs reads the built tree for the fifteen editor checkboxes, and there is no allow list because an empty one watched by a test is the census-emptying failure",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T21:25:37.084Z",
    "resolved_at": null
  },
  {
    "id": 492,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "nvda-tests/tests/settings-tabs-read-once.test.js",
    "line": 1,
    "description": "09-06: whether each Settings tab is heard once along the tab row is FOUND-09's listening line and has not been heard. What is held: the native tab control's own arrow handler raised EVENT_OBJECT_FOCUS twice on the reached tab per key (scripts/uia-events.ps1, 2026-09-16, before) and raises it once now that the row answers its arrows through SetSelection (the same script, after); tests/the_settings_tab_row_says_each_tab_once.rs sends a real WM_KEYDOWN to the built dialog and counts one, which is structure present. What NVDA says about one event is the NVDA case's to answer, on CI at the next push of main, which is Pratik's, and the tester's ear after; neither has run. Roadmap criterion 6's transcript clause and FOUND-09's third [D] line stay open until that run",
    "status": "fixed",
    "reason": "Fixed by 11-02 on 2026-09-18 on the tester's word: heard on 1.0.0-alpha.1+149.g744d05ef with NVDA, the Settings dialog speaks as it should and arrowing along the tab row says each tab once (#33, closed 2026-09-18T11:34Z). The runner's case ran once, in run 35336142908 at 744d05ef, and heard nothing because it waited for an opening announcement the harness never captures; 11-02 corrected the wait, and whether the corrected case hears each tab once on the runner is ledger 531",
    "recorded_at": "2026-09-16T22:44:18.512Z",
    "resolved_at": "2026-09-18T13:30:00.000Z"
  },
  {
    "id": 493,
    "kind": "deviation",
    "phase": "09",
    "file": "scripts/uia-events.ps1",
    "line": 1,
    "description": "09-06: the plan prescribed a UI Automation event logger and read a capture of one event per key as meaning the second reading was NVDA's own; the managed UI Automation client did show exactly one ElementSelected per key and nothing else, and it is blind to what NVDA reads for a native SysTabControl32, which is MSAA and win events. The logger logs both channels, the win-event hook reading class, text and child id only and never an IAccessible (ledger 390). Two conditions of the capture: the session was locked (the focused element was the Lock Screen, pid 16028), so SendInput answered ERROR_ACCESS_DENIED and SendKeys threw, keys were posted as WM_KEYDOWN to the tab control's own window, and no window could take foreground focus, so the page-panel candidate is judged from the in-thread focus events (the row took focus at open, both captures) and wxWidgets' UpdateSelection giving the page focus only when the notebook has none, not from a foreground run; and NVDA was running and not stopped. The fix also takes the numpad's arrows, found because the test's first key lacked the extended bit and wxWidgets read VK_RIGHT as WXK_NUMPAD_RIGHT",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-16T22:44:35.134Z",
    "resolved_at": null
  },
  {
    "id": 494,
    "kind": "todo",
    "phase": "09",
    "file": "nvda-tests/README.md",
    "line": 60,
    "description": "09-06: the README's What is in here table lists two test files and the directory holds five (calendar-immediate-actions, filter-manager-delete and settings-tabs-read-once are not in it), and its prose says two tests where the workflow runs five. A table that is read as the inventory and is short by three is a check nobody reads; bring it to the directory, or have a reading hold it there",
    "status": "fixed",
    "reason": "Fixed by 11-02 on 2026-09-18: the table lists all five files under tests/ with what each holds and whether it runs, the prose dates the two the package began with and counts the five, and the workflow section says four run and one is skipped. No reading holds the table to the directory; the count is a sentence dated 2026-09-18",
    "recorded_at": "2026-09-16T22:44:35.707Z",
    "resolved_at": "2026-09-18T13:30:00.000Z"
  },
  {
    "id": 495,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "Whether the preview pane's bar and a conversation's per-message sentences read well by ear has been heard by nobody: preview_html renders the top of the bar as a region named Security warning above the message, and one_of_several says why a PGP message did not open under its own heading on the page and in the text reader. Both are held by tests against a key and a message GnuPG made and a signed message OpenSSL made; whether they sound right with NVDA, and whether a real correspondent's key opens anything, is FOUND-10's last [S] line, the tester's (09-07, #51).",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T01:01:46.443Z",
    "resolved_at": null
  },
  {
    "id": 496,
    "kind": "todo",
    "phase": "09",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "import_a_pgp_private_key announces its outcome and puts nothing in the status bar; its comment said both until 2026-09-16 and was corrected to what the body does. A visible line to match the spoken one is owed, on import_a_mailbox's pattern, which sends the status and announces; it wants a red in tests/wired.rs and ui_tx and runtime passed to the function (09-07, #51 item 6).",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T01:02:01.809Z",
    "resolved_at": null
  },
  {
    "id": 497,
    "kind": "todo",
    "phase": "09",
    "file": "src/presentation/reader_text.rs",
    "line": null,
    "description": "A PGP signature inside a conversation of several messages is still not mentioned there: SIGNED_AND_NOT_CHECKED_HERE is folded by with_encryption for one message only, and one_of_several says the PGP opening reason and the S/MIME envelope but nothing about a clearsigned part. Opening the message on its own says it; the changelog's dated correction on the armour entry says so (09-07).",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T01:02:02.382Z",
    "resolved_at": null
  },
  {
    "id": 498,
    "kind": "deviation",
    "phase": "09",
    "file": ".planning/phases/09-what-the-first-day-of-testing-found/09-07-PLAN.md",
    "line": null,
    "description": "09-07 executed with four departures: the wired.rs guards were edited in task 1 rather than task 3 because body_of panics on the moved fn line and task 1 could not compile its verify otherwise; task 2's second guard record went on reader_text's behaviour (the reason dropped from one_of_several) rather than on a call-site bypass, and task 3's record covers the call site; three stray rustdoc blocks moved home rather than one, the folder loader's and the module loader's beside the mailbox import's; and a third changelog entry, the armour entry's thread limitation, was dated beside the two the plan named. The key import's status-bar sentence was corrected in the comment, not built, because the task's one red was spent (ledger todo beside this).",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T01:02:02.964Z",
    "resolved_at": null
  },
  {
    "id": 499,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/application/importing_an_outlook_data_file.rs",
    "line": null,
    "description": "No real Outlook data file has been through the import: neither this program nor the outlook-pst crate can write one, so brought_in's walk over a real file's folders (opened, what_it_holds, each_item_in) is unrun, and what is tested is the filing of each of the five kinds handed in by one_folder_filed. The closing sentence says so to whoever runs it. FOUND-11's [S] line; the tester's, when he has a .pst to hand.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T02:49:01.548Z",
    "resolved_at": null
  },
  {
    "id": 500,
    "kind": "todo",
    "phase": "09",
    "file": "src/service/outlook_data_file.rs",
    "line": null,
    "description": "The reader words every refusal as a sentence to be heard and carries it in Error::Other, whose display puts Error: in front, which a screen reader says first. importing_an_outlook_data_file::as_it_was_worded takes the words out where the closing sentence is built; the reader itself should move its sentences onto Error::InPlainWords, which is a change to the reader wanting a red of its own and was outside 09-08's rule of touching the reader only to make something public.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T02:49:19.475Z",
    "resolved_at": null
  },
  {
    "id": 501,
    "kind": "todo",
    "phase": "09",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "After an Outlook data file is imported the folder tree is read back, as after an archive, but the calendar, contacts, tasks and notes modules are not: they show what arrived when next opened, and the changelog says so. The worker should send the module's own reload for each kind that arrived, the way folder_tree_updates does for mail, with a wired.rs reading holding it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T02:49:20.052Z",
    "resolved_at": null
  },
  {
    "id": 502,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nobody has opened a file Save As wrote in another mail program, and nobody has run Save As by hand: what is held is that this program's own reader reads the written bytes back as the message that went in with its file (export_tree tests), that a kept signed original is written byte for byte, and that the window asks the decision and the writer (wired.rs). The dialog, the destination and the status line are the tester's.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T02:49:20.635Z",
    "resolved_at": null
  },
  {
    "id": 503,
    "kind": "deviation",
    "phase": "09",
    "file": ".planning/phases/09-what-the-first-day-of-testing-found/09-08-PLAN.md",
    "line": null,
    "description": "09-08 executed with five departures: the new module's tests hand it items of each kind rather than reading a fixture, because the reader's header and the crate's README say no data file can be written, so the walk over a real file is a thin untested half named as such; export_tree.rs gained one_message_written_out, a file the plan did not list, because the exporter already owns stored-message-to-bytes and its file walk; the archive import's folder helper moved from wx_app.rs to importing_messages::a_folder_for_imported_mail so three imports share it; the wired.rs reading of the import worker's three-way dispatch arrived green in task 2's green commit rather than red in task 1, its behaviour having been taken red there; and the reader's Error::Other sentences have their words taken out at the seam rather than the reader changed (todo beside this).",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T02:49:21.208Z",
    "resolved_at": null
  },
  {
    "id": 504,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "tests/the_settings_dialog_opens_in.rs",
    "line": null,
    "description": "Whether Settings now feels immediate on the tester's machine is his to say (FOUND-12's [S] line, #34). The harness measures from Ctrl+, to the moment before show_modal: 2,206 ms before 09-09 and 397 ms after, median of five in the release binary with NVDA running in this session. The show, the focus landing and the screen reader's first announcement come after that moment and nothing here times them; nobody has heard the dialog open since the change.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T06:19:04.138Z",
    "resolved_at": null
  },
  {
    "id": 505,
    "kind": "todo",
    "phase": "09",
    "file": "src/service/spellcheck/mod.rs",
    "line": null,
    "description": "try_load_spellbook leaks both halves of every Hunspell dictionary it loads through Box::leak, on every for_language call, so a machine with a Hunspell dictionary loses the dictionary's size in memory each time a checker is built: each compose window, and until 09-09 every open of Settings. Pre-existing, found while reading the loader for 09-09 and not changed there because it wants a red of its own. spellbook::Dictionary wants 'static text; an Arc or a once-per-process cache of the parsed dictionary is the shape.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T06:19:04.752Z",
    "resolved_at": null
  },
  {
    "id": 506,
    "kind": "todo",
    "phase": "09",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "With the window shown and NVDA running, every control in the Settings dialog costs about twice what it costs on a hidden frame, and a Choice control costs 10 to 60 ms each, the language list's 19 names in many scripts 24 ms hidden and 61 ms shown. That is what is left in the release binary's 397 ms after 09-09: the General page's controls under the screen reader's in-process hooks, plus the configuration read. Found by a scratch run of 2026-09-16 that showed the harness's frame and matched the release binary within three percent; the mechanism is inferred from that and not diagnosed further, because stopping the tester's NVDA to take the number without it was not asked for. Fewer Choice controls on General, or a build off the interface thread, would move it; neither was done.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T06:19:18.152Z",
    "resolved_at": null
  },
  {
    "id": 507,
    "kind": "todo",
    "phase": "09",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "Since 09-09 a Settings page after General is built the first time its tab is reached, so the first Right arrow onto Reading pays that page's build: 156 ms on a hidden frame in the test process (the row of 2026-09-17), about twice that under NVDA by the scratch run's factor. The other five are under a tenth of a second hidden. Whether a pause on the first visit of Reading is felt, and whether filling the pages after the dialog shows with the control saying so would be better, is the tester's to say; the plan allowed either shape and this one was chosen because a page is never seen empty.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T06:19:18.744Z",
    "resolved_at": null
  },
  {
    "id": 508,
    "kind": "deviation",
    "phase": "09",
    "file": ".planning/phases/09-what-the-first-day-of-testing-found/09-09-PLAN.md",
    "line": null,
    "description": "09-09 executed with four departures. The fix is not either shape the plan named: the rows said the three lists cost a millisecond each and the pages the rest, and a scratch timing of each page, reverted, found a spell checker built for one sentence and released (210 ms) and an unfrozen typeface list resizing itself per item (1,090 ms with the window shown), so the change is a frozen build, a source named without a checker, and pages built when their tab is first shown, the last being the shape the plan and FOUND-12 offered for a page cost. The harness drives the release binary itself, posting the Settings menu command to a window it started, rather than a person pressing Ctrl+, on a real profile of this machine, so no run touched the tester's profile and the number is repeatable. The dialog is timed five times in the test process rather than once, and a sixth build times each tab's first visit. Task 1's line is written in show_settings_dialog beside show_modal, from an Instant handle_settings is handed, rather than in handle_settings itself, because show_modal lives there.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T06:19:32.442Z",
    "resolved_at": null
  },
  {
    "id": 509,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Nobody has run File, Import a Folder of Messages by hand since 09-10 added it, and no Thunderbird profile folder has been read through it. The folder walk is proven by mailbox_archive's own tests over folders its tests make and by the wired.rs readings that hold the item to a DirDialog and the hand-over to the worker; what a real folder of somebody's saved mail becomes, and what a Thunderbird profile becomes (one folder per mailbox file, each .msf refused and counted, the .sbd nesting one level out of place), is written in the changelog and the guide from reading the walk, not from running it. FOUND-11's third [D] line; #53 point 3.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T07:17:05.342Z",
    "resolved_at": null
  },
  {
    "id": 510,
    "kind": "todo",
    "phase": "09",
    "file": "src/service/mailbox_archive.rs",
    "line": null,
    "description": "The folder import does not recognise Thunderbird's layout: a mailbox file with no ending beside a .msf index and a .sbd folder of subfolders. Read as loose files, a profile's mail directory becomes one folder per mailbox file, each .msf a refused non-mail file counted in the sentence, and the folders inside Inbox.sbd landing under a folder called Inbox.sbd beside Inbox rather than inside it. Recognising the layout means treating name.sbd as the children of the mailbox file name and skipping .msf without counting it. Said in the changelog and the guide since 09-10 (#53 point 3); later work with points 4 to 6.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T07:17:22.879Z",
    "resolved_at": null
  },
  {
    "id": 511,
    "kind": "deviation",
    "phase": "09",
    "file": ".planning/phases/09-what-the-first-day-of-testing-found/09-10-PLAN.md",
    "line": null,
    "description": "09-10 executed with four departures. The item's letter is O, not the plan's F, which Fetch Missing Message Text already has on the File menu; test_no_two_items_on_one_menu_claim_the_same_letter would have refused the plan's spelling, and the green commit wrongly said no check read menus, corrected in the next commit. The shortcuts page had no row for Import Mailbox beside which to add one, so four rows were added: Import Mailbox, Import a Folder of Messages, Export Mailbox and Import PGP Private Key. The import handler was split into an_account_to_import_into, refuse_to_import and mail_brought_in_from so the two pickers share the readiness check and the worker start, and the older wired.rs reading of the worker's shape was re-pointed at the shared function and named in the red trailer. The changelog's gathered Known limitations paragraph for issue 53's points 4 to 6 went in with task 2, as the plan said, after being drafted and withdrawn during task 1 so task 1's commit carried only its own words.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T07:17:23.479Z",
    "resolved_at": null
  },
  {
    "id": 512,
    "kind": "deviation",
    "phase": "10",
    "file": ".planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-01-PLAN.md",
    "line": null,
    "description": "10-01 executed with five departures, so 10-05 reads the signatures it will call from the tree and not from the plan. what_to_do_next takes TextStillMissing { messages, kept_bytes } where the plan had the list alone, because a budget on bytes kept needs the bytes kept as an input and the plan's signature had no place for them. The sentences write bare numbers, 3500 of 12872, as every counted sentence in the tree does, where the plan's examples grouped thousands; grouping is one helper and every counted sentence if Pratik hears the bare form as harder. The whole-list run is fetch_all_the_missing_text, generic over Mailbox, with fetch_the_missing_message_text delegating to it as before, because the fold has to be tested against the scripted mailbox and the entry point takes a real controller. The IMAP timeout phrase is imap::THE_SERVER_STOPPED_RESPONDING, read by the classifier, because a timeout and a dropped connection are both Error::Network and kind alone cannot tell them apart; pop3.rs still writes the same phrase as a literal. The stopped-coming-down rule was re-pointed at green rather than red, because the red stub answers false and the old loop's test would have run forever under it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T11:50:58.000Z",
    "resolved_at": null
  },
  {
    "id": 513,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "What only the tester's ear settles for #67 and #68, both fixed by 10-01.1 and measured over MSAA and GetFocus() from a test that builds the real dialog: that a later-page Settings checkbox is spoken as check box with its state, that Space says the new state, that the state reads back after Tab away and back, that OK keeps it, that Settings still opens at once, that an arrow on the tab row still says each tab once, and that Ctrl+Tab from a General control speaks a named control and nothing before it. The listening lines belong to 10-07's page; the issue-close comments carry them until then.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T14:05:00.000Z",
    "resolved_at": null
  },
  {
    "id": 514,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/theme.rs",
    "line": null,
    "description": "Four files both paint a window with theme::paint and build checkboxes on it, and none has been read for whether the paint comes before or after the build: wx_account_manager.rs, wx_compose.rs, wx_item_form.rs and wx_managers.rs. A checkbox created under an already painted panel inherits its text colour, is made owner-drawn by wxWidgets, and reads as a push button under NVDA, which is what #67 was on the Settings pages. A tree-wide reading that no built wxCheckBox is BS_OWNERDRAW, on the shape of tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs, is separate work; said in the changelog's Known limitations for #67.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T14:05:00.000Z",
    "resolved_at": null
  },
  {
    "id": 515,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "What only the tester settles for #24, fixed by 10-02 and measured by the harness at his size and at 200,000: whether his folder of 12,872 messages opens and reads as one list with his screen reader on his machine, and whether the list still answers keys at once after a folder change, which is MAIL-02's last line. Nobody has opened a folder of 200,000 in the running program, only the harness has read one, and the first open of one that size pays under a second on the interface thread by the rows named The list's own read path after 10-02 on docs/development/measurements.md, dated 2026-09-17; moving that read off the interface thread is later work and the page says what it would buy.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T16:10:00.000Z",
    "resolved_at": null
  },
  {
    "id": 516,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "What only the tester settles for #69, fixed by 10-02.1: whether, with All Inboxes open, choosing Oldest first from View, Sort Messages, arrowing to a folder and back to All Inboxes reads the oldest first with his screen reader, and the same for a label view and after running a saved search; and whether Unread First from the menu, which was saved as read first until 10-02.1, now puts the unread rows first on the next read of a folder. The composed run through load_every_inbox is held by three links and not by one test: the cache answers in the order it is handed, the sort the menu stores reads back into that clause, and the window's three readers are held to asking for it by a reading of the source, because the loaders and the_sort_as are private to the window and its own test module reads the machine's profile.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T18:50:00.000Z",
    "resolved_at": null
  },
  {
    "id": 517,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "scripts/build-installer.sh",
    "line": null,
    "description": "What only a build and a machine settle for the build counter 10-02.2 added (Pratik's decision of 2026-09-17): that the next installer handed to the tester carries the counter in its file name and in Apps and Features, as 1.0.0-alpha.1+N.g<commit> and a file version ending in the counter; that installing it over the 1.0.0-alpha.1+g59c5b6a4 build he has is read by Windows as an upgrade rather than refused as a downgrade; and that --version and the log's first line show the counter. The tests read the script's text and order the encoded fields; one installer was built from the branch by the executor and its --version and file version read back, but no build with the counter has been installed over the alpha.1 build anybody has, and no build with it has been handed to anybody.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T19:52:50.000Z",
    "resolved_at": null
  },
  {
    "id": 518,
    "kind": "deviation",
    "phase": "10",
    "file": ".planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-03-PLAN.md",
    "line": null,
    "description": "10-03 executed with four departures, so 10-05 reads the seam it will call from the tree and not from the plan. The record on the screen's read-back is measured on a new target, tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs, rather than on every_event_has_a_control or the_settings_dialog_opens_in as the plan offered, because neither of those reads the Permissions page and a break there reddened nothing in either; the new target builds the real dialog, chooses each size and reads it back the way OK does. The record on the check's worker being handed the setting is measured now on a fourth reading in that target rather than deferred to 10-05's target as the plan said to do if no existing target reddened, because the reading is the same reading and earlier. The two worker sites read the setting through one helper, how_much_message_text_stays, rather than each repeating the six lines, and that helper is where the read-by-something guard sees the field's name. The attachment budget's comment, which said the two halves kept the whole cache around a gigabyte, was corrected with the date, because the sentence became false the moment the body half became a setting.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T21:53:34.000Z",
    "resolved_at": null
  },
  {
    "id": 519,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_settings.rs",
    "line": null,
    "description": "What only the tester's ear settles for the choice 10-03 added under Message Text on the Permissions tab: that Alt+K reaches it and NVDA says its name, Keep the text of messages on this computer, then combo box and the current answer; that the four answers read as All of it, Up to 1 GB, Up to 5 GB and Up to 20 GB and Up or Down moves between them; that the sentence under it is read once in passing and says what leaves, when and what stays; and that OK keeps the answer across a restart. Whether the eviction then honours a chosen size against his account is settled only by a mailbox with more than that much text, which his 12,872 messages may or may not hold, and the default keeps everything, so the first sign of the setting working is text that stays where 0.125.1 dropped it, which nothing on his machine has measured. The listening lines belong to 10-07's page.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-17T21:53:34.000Z",
    "resolved_at": null
  },
  {
    "id": 520,
    "kind": "deviation",
    "phase": "10",
    "file": ".planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-04-PLAN.md",
    "line": null,
    "description": "10-04 executed with five departures, so 10-05 and 10-06 read the kinds from the tree and not from the plan. The new-mail signal lives in the WhatArrived arm rather than inside spawn_mail_sync as the plan's grep criterion said, because the worker holds no accessibility handle and the arm is what its end-of-check update reaches; the criterion as written could not be met. The two new variants and ModuleSyncFinished are on ui_types.rs, which the plan's file list did not name, because that is where UIUpdate lives. The tasks and notes syncs finish through a new ModuleSyncFinished update rather than the existing completion updates, and the notes sync's two answers that no sync ran stay on the answer channel, because they answer the key rather than report a sync. The result sentence counts through how_many, Inbox, 3 new messages, rather than the plan's Inbox, 3 new, so a listener hears what the number counts. The whole-folder request's closing report goes out as a step with its progress, shown and not spoken under the default, because the loop hands over one kind of line and the command retires with 10-05. The setting's changelog entry landed in task 1's commit, on the rule that the entry goes with the setting, and task 2 extended it; task 3 wrote the listening lines.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T00:21:57.000Z",
    "resolved_at": null
  },
  {
    "id": 521,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "What only the tester's ear settles for #38: what a check of his 50 folders says under each of the three answers on the Feedback tab, and which sentence is a step and which a result by ear, since the sorting was done by reading each line's words and a line sorted wrongly is silent under the default or spoken under it; whether the one result sentence, folder by folder with counts, is heard as an ending rather than as another line; whether the sound for new mail followed by that sentence reads as one event or two; whether Settings saved is now heard when OK is pressed during a check; and whether the choice itself is reached by Alt+W and read as its name, combo box and answer, with the sentence under it read once. Nothing here met a real account, and nobody has listened. Items 42 and 43 on docs/manual-accessibility-pass.md.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T00:21:57.000Z",
    "resolved_at": null
  },
  {
    "id": 522,
    "kind": "deviation",
    "phase": "10",
    "file": ".planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-05-PLAN.md",
    "line": null,
    "description": "10-05 executed with six departures, so 10-06 and 10-07 read the runner from the tree and not from the plan. Nine tests in wx_app.rs read the retired offer and commands, not the five the plan counted by line range, and all nine were rewritten in place so the count stayed 199. Get Older Messages answers the key through send_status, the answer channel, rather than as Progress, because 10-04 sorted that line as an answer and a key pressed into a silent step is pressed again; the runner's own lines are Progress. The runner sends MoreOfTheFolderArrived per chunk rather than FolderMessagesArrived, because the arm and its guard record already existed for exactly that update. The runner sends one WhatArrived per account that had anything to do, from its own function, because the check's list has gone out hours before a download ends and the reading holds spawn_mail_sync alone to one. The whole of application::asking_for_a_whole_folder is deleted rather than its loop alone, because nothing but the loop reached its sentences. UIUpdate::WhatCouldBeFetched keeps its name and its three managers.rs tests, because what it counts is unchanged and renaming it would flag fifty-two records. start_the_download refuses to start while a wait after a refusal is running, Get Older Messages and Pause included, so a key does not undo the wait.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T02:49:01.000Z",
    "resolved_at": null
  },
  {
    "id": 523,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "What only the tester's account and ear settle for #20 and #23. The account: whether Gmail tolerates 12,872 messages coming down five hundred at a time and their text fifty at a time after every check, what it does when it has had enough, whether a refusal is read as the run stopping short rather than as a folder finished, and whether a wait of thirty seconds doubling to thirty minutes suits it; ledger 11 and 72. The ear: whether Pause Downloading on Tools is read as a check box with its state and its description; whether its two answers, and Get Older Messages' answer that this folder comes first, are heard above the download's own lines; whether a first download of 50 folders under Say what arrived is one sentence at its end with the sound for new mail, and under Say every step a line per chunk; whether the sentence a search says about text that is on its way is heard beside the coverage sentence rather than over it. The runner was driven through readings and the model's tests only; no loopback IMAP server exists as a process to point the release binary at, so no binary was started.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T02:49:01.000Z",
    "resolved_at": null
  },
  {
    "id": 524,
    "kind": "stub",
    "phase": "10",
    "file": "src/application/mail_sync.rs",
    "line": null,
    "description": "mail_sync::fetch_the_missing_message_text and fetch_all_the_missing_text, the whole-list text pass, are reached by nothing outside their own tests since 10-05 retired Fetch Missing Message Text and the offer above the message list; the download of everything asks fetch_over_a_mailbox one chunk at a time instead. They stay because fifteen tests hold the fold, its wording and the reading gate through them, in a file thirteen guard records fingerprint at 149, and their retirement is a rewrite of that suite: the tests move onto fetch_over_a_mailbox or go, the records' red lists and counts are corrected by hand and re-measured, and what_the_fetch_did, says_where_it_is, about_to_fetch and the Backfill enum go with them or are kept by a caller. The doc comment on the entry point says so.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T02:49:01.000Z",
    "resolved_at": null
  },
  {
    "id": 525,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "What only the tester's account and ear settle for #37. The account: whether Gmail drops an IDLE connection at all, after how long, and whether the watch started again after a wait of thirty seconds doubling to thirty minutes carries mail in over hours; whether a watch per enabled account counts against a provider's connection limit, ledger 67; whether a start that checks every account and asks for every watch at once is welcome; what a scheduled check every five minutes of fifty folders does to a mailbox that the watch already covers. The ear: whether the three status lines, watching, waiting to watch again, and checking every N minutes, are heard as states rather than as three sentences that sound alike, with the account named first for a person with two accounts and never for a person with one; whether the first line of F9 under two accounts, Checking 2 accounts for new mail, is heard; whether the sentence under the Check Interval field on the account editor is read with the field and again beneath it, and whether twice is too many; whether an error every five minutes for an account with no password saved is a flood or a reminder. Nothing here has met a real server or been heard.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T04:51:18.000Z",
    "resolved_at": null
  },
  {
    "id": 526,
    "kind": "unrun-verify",
    "phase": "10",
    "file": "tests/the_numbers_the_targets_ask_for.rs",
    "line": null,
    "description": "What the program says when a connection is refused at the socket is still read by nobody. The measurement profile's account points at 127.0.0.1 on a closed port, and since 10-06 a start attempts it, but it is refused by the credential store before anything is dialled, because the profile stores no password and an empty password cannot be stored through the harness (ledger 374, the keyring race). So the refusal read for ledger 446 is an authentication refusal, and the connection-refused path through the watch's start, NeverStarted from watch_folder, and the check's the_session_at, has been driven only by unit tests against a closed port and never through the running binary. A harness that can store a password, or a loopback listener that closes the socket, is what would read it.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T04:51:18.000Z",
    "resolved_at": null
  },
  {
    "id": 527,
    "kind": "deviation",
    "phase": "10",
    "file": ".planning/phases/10-all-the-mail-and-what-is-said-while-it-comes/10-06-PLAN.md",
    "line": null,
    "description": "10-06 executed with departures, so 10-07 reads the watch from the tree and not from the plan. The restart decision answers three things, not two: AfterAWait, Never for a stop somebody asked for, and OnTheScheduleAlone with what would lift it, because a stop the window itself asked for when replacing a watch must not mark the account, and a server that could not be reached and a server that refused to watch are lifted by different facts. A watch already reading events is left alone by a request, so a scheduled check does not replace a working watch every five minutes. The download is asked for only after a check in which some account went through, as 10-05's tree did, found by running the release binary. The schedule follows the network and does not run while the program believes there is none, so an unreachable server is not an error every interval while the network is gone. mark_synced marks the worker's copy and update_account_last_sync writes the column, not save_account, which would write the credential store on every check. The watch's wait is on InboxWatch per account beside its handle, not on a second map. The account editor's sentence is on the field as its accessible description and beneath it as text. The measurement account is refused by the credential store and not the port, ledger 526.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T04:51:18.000Z",
    "resolved_at": null
  },
  {
    "id": 528,
    "kind": "stub",
    "phase": "10",
    "file": "src/data/account.rs",
    "line": null,
    "description": "Account.last_sync is written by every check that goes through since 10-06, on the worker's copy through mark_synced and in the row through update_account_last_sync, and read back by load_accounts into a field nothing reads: the schedule keeps its own clock for the session in WxUIState.last_checked, and a start checks every enabled account whatever the row says. The column is true now where it was empty before, and it is a fact with no reader. A reader would be a start that skipped an account checked a moment ago by a previous run, or a status line that says when the account was last checked; neither exists, and this entry says so rather than leaving the column to read as consumed.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T04:51:18.000Z",
    "resolved_at": null
  },
  {
    "id": 529,
    "kind": "todo",
    "phase": "10",
    "file": "docs/privacy.md",
    "line": null,
    "description": "The privacy page's table row for OneNote says Never, nothing here reads or writes a notebook, and the section under it, The OneNote permission which nothing uses, says no notebook has ever been opened and no note written here goes anywhere. Both were true when written and false since phase 5.2: notes on an Outlook or Office 365 account sync to OneNote, docs/ALPHA_TESTING.md says so under what is known to be missing or unproven, and the Calendar and PIM tab says it is experimental. Found on 2026-09-18 by 10-07 reading the page for every sentence phase 10 falsified; left because correcting it means reading phase 5.2's summaries for what a notes sync sends, which is outside this phase, and a dated sentence beside each of the two claims is the shape the page uses. Until then a person reading the privacy page is told a permission is unused that the notes sync uses.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T05:59:52.000Z",
    "resolved_at": null
  },
  {
    "id": 530,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "tests/the_language_the_screen_shows_is_the_one_used.rs",
    "line": null,
    "description": "The en-AU case of this target, the one that failed on GitHub's runner in CI run 35336142985 at 744d05ef, cannot be run red on this machine, because Windows here offers en-AU and the runner does not; 11-01 fixed the rule in presentation::which_language_row, held it by six cases over hand-built rows that are red on both machines before the rule, and left this target unchanged and green here. Whether the runner now keeps en-AU is settled by the next push of main, which is Pratik's; until that run is read this entry stands, and the run's Test Suite job is the reading.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T13:04:42.000Z",
    "resolved_at": null
  },
  {
    "id": 531,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "nvda-tests/tests/settings-tabs-read-once.test.js",
    "line": 160,
    "description": "11-02: the corrected settings case takes its mark after the settle and presses Right without waiting to hear General, because its first run (35336142908 at 744d05ef) waited for an opening announcement no transcript of any case has ever held and timed out with an empty log. It never runs on this machine (nvda-tests/README.md) and runs at the next push of main, which is Pratik's. What that run shows: which tab the first Right reaches on the runner's fresh profile. If the tab row holds focus at open, Compose, and the six Rights and one Left are counted; if not, the case fails with 'never heard: Compose', a finding about focus at open on a fresh profile and not about the row. The tester's ear has settled the row itself (#33); this entry is the harness's own case",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T13:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 532,
    "kind": "deviation",
    "phase": "11",
    "file": ".github/workflows/accessibility.yml",
    "line": 44,
    "description": "11-02 took continue-on-error off the NVDA job and left it on the Accessibility scan job on purpose. The scan's findings are counts per window the workflow already reads out (violations, unnamed, broken): 20 violations over 36 windows in run 35336142914 at 744d05ef, on windows 11-02 does not touch (new-event 9, send-later 6, compose 4, accounts 1). Whether a count should fail the run is a decision for the phase that owns the findings, not 11-02's. Until it is taken, the scan's badge is green over any count and the report is what to read",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T13:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 533,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_folder_choice.rs",
    "line": 286,
    "description": "11-03: Folders to Keep Up to Date is a tree with the control's own check boxes, and what only the tester's ear settles for #70: that a kept folder is heard as checked and an unkept one as not checked, that Space says the new state after it toggles, that a nested folder's level is read, that the title is heard as the account's name, that the All Mail sentence under the tree is reached and understood on his Gmail account, and whether his Gmail lists All Mail at all (his folder list holds none; Gmail's Labels, Show in IMAP setting decides). The readings prove the state on TVM_GETITEMSTATE and over MSAA, the nesting, the cursor, the sentence and the title's argument",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T15:12:32.000Z",
    "resolved_at": null
  },
  {
    "id": 534,
    "kind": "deviation",
    "phase": "11",
    "file": "src/presentation/accessibility/names.rs",
    "line": 140,
    "description": "11-03: wxdragon 0.9.17's acc_state constants carry MSAA's numbering (CHECKED 0x10, FOCUSABLE 0x100000, SELECTABLE 0x200000), its C++ shim hands a GetState answer to wxAccessible unconverted, and wxWidgets numbers its own wxACC_STATE_SYSTEM enumeration differently (BUSY 0x10, PROTECTED 0x100000, READONLY 0x200000) before converting to the platform's, so every state written through those constants arrives as a different state. Measured over AccessibleObjectFromWindow: CHECKED arrived as BUSY and SELECTABLE as READONLY, which is what the tester heard. The one writer in this tree, CheckedRows, is retired; nothing writes a state through them now, and the paragraph at this line says so. Upstream defect, not reported yet",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T15:12:32.000Z",
    "resolved_at": null
  },
  {
    "id": 535,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": 22706,
    "description": "11-04: the log's default follows the version and the lines a report needs are written at info and debug, and what only a report from the tester's machine settles for #71: whether each check's per-folder line, the download's chunk lines, the settings save line and the held-back line are the lines that make his next problem diagnosable, and what a day at Debug on his real account costs on his disk, which a two-minute start against the measurement profile cannot say. The readings prove each line's presence and level and that no call spells a secret; nobody has written a report from a log at this level",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T17:52:00.000Z",
    "resolved_at": null
  },
  {
    "id": 536,
    "kind": "deviation",
    "phase": "11",
    "file": "scripts/which-checks.test.sh",
    "line": 248,
    "description": "11-04 found, not fixed: the suite's scratch-repository fixture runs git -C <scratch> init, config, add and commit, and a hook exports an absolute GIT_DIR and GIT_INDEX_FILE when the commit is made from a linked worktree (measured 2026-09-18 with a hook printing its environment: unset in the main checkout, absolute in a worktree), so from a worktree every fixture command acts on the real repository: init marked it bare, config overwrote hooksPath and the user, add staged the fixture's four-line Cargo.toml into the worktree's index, and commit landed that beside the planner's staged files on main as b4a4cc81 under the fixture's message and committer. The repository config was restored by hand at 16:55Z, and the planner undid the commit (main's reflog at eb5d8517: \"planner: undo the suite's stray commit made through the hook from a linked worktree\") and landed its plans as 1ae63359. The fixture itself is unchanged and will do the same on the next commit made from a linked worktree: it must clear GIT_DIR, GIT_INDEX_FILE, GIT_WORK_TREE and GIT_PREFIX before its first git, with a case that exports an absolute GIT_DIR and asserts the real repository did not move. A second trigger, found by 11-06 on 2026-09-18 from the main checkout: a commit that names its paths, or git commit --amend --only, hands the hook a temporary GIT_INDEX_FILE (.git/next-index-<pid>.lock), the fixture's \"a version bump staged in a repository of its own\" case then answers all rather than affected, and the gate refuses the commit; nothing moved that time, since the temporary index goes with the refused commit, and the remedy is the same clearing",
    "status": "fixed",
    "reason": "Fixed by 11-06.3 on 2026-09-19: scripts/shell-suite.sh unsets GIT_DIR, GIT_WORK_TREE, GIT_INDEX_FILE, GIT_PREFIX and GIT_COMMON_DIR right after set -uo pipefail, in the harness every suite sources, so before any suite's first git (d5c3483e). Two cases in which-checks.test.sh are red if that stops (cdf04ff8), each handing a fresh bash the harness and one variable the way the hook hands a suite its environment, against a throwaway repository named elsewhere and never this one: under an absolute GIT_DIR the fixture's commit landed on elsewhere's HEAD and replaced its hooks path; under an exported GIT_INDEX_FILE the fixture wrote to the handed index and the subject answered all. After the green, from the primary worktree, GIT_DIR set to this repository's absolute git dir and then GIT_INDEX_FILE set to a copy of its index each ran the suite to exit 0 over 52 cases with HEAD, the config checksum, the reflog length and the copy unchanged. The --only half, two commits sharing COMMIT_EDITMSG, is not this and still stands in CLAUDE.md",
    "recorded_at": "2026-09-18T17:52:00.000Z",
    "resolved_at": "2026-09-19T02:00:00.000Z"
  },
  {
    "id": 537,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "tests/the_numbers_the_targets_ask_for.rs",
    "line": 433,
    "description": "11-04: the two size rows for docs/development/measurements.md, the log a two-minute start writes at info and at debug against the measurement profile, were not taken. One copy of Wixen Mail runs at a time and the tester's copy was open on his account through the whole session (process 12648, INBOX, 14400 unread); the one attempt at 17:37Z handed itself to that copy, which was raised and said \"Wixen Mail is already running, and this is it\", and a run started the moment his copy closes would take the single-instance slot from a restart. The harness now refuses to start while any wixen-mail.exe is running, with the reason, and pins the level through WIXEN_MEASUREMENT_LOG_LEVEL. The rows are owed: run the two commands on the harness's header with no Wixen Mail open, write the rows with their date and commit, and quote the sizes on docs/ALPHA_TESTING.md",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T18:05:00.000Z",
    "resolved_at": null
  },
  {
    "id": 538,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/page_jumps.rs",
    "line": null,
    "description": "11-04.1: what only the tester's ear settles for #84. In the formatted view: Alt+A from the message landing on the attachments list with \"Attachments, N\" said, and whether that sentence after the list announces itself is one thing too many; Alt+A from the list going back with \"Message\"; F7 to the warning bar with \"Security warning\" and F7 back; \"No attachments\" and \"No warning\" when there is nothing to go to. In the plain-text reader: Alt+A to the list and back, where the way back is answered by the menu accelerator when the list has focus, because the frame takes an accelerator before a list box sees the key; that was reasoned from how the toolkit routes a chord (wxTextCtrl exempts Ctrl+arrows, a list box exempts nothing) and never watched, and if the chord reaches the list instead, the list's own handler answers it. The readings hold the shape; none of this has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T19:12:00.000Z",
    "resolved_at": null
  },
  {
    "id": 539,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-05, amended by 11-05.1 on 2026-09-18: what only the tester's ear settles for #25. A walk through a folder with unread messages, letting NVDA finish every row, leaving the unread count where it was; Space once on an unread message leaving the count where it is, however long the row stays selected; Space again, the whole reading, then the count moving after two seconds with the row still selected; Shift+Space the same; Enter on one doing the same; moving off a message before the delay runs leaving it unread; and the sentence under Mark as read after on the Reading tab read once, on the choice's own row and not twice. Whether \"previewed\" in his words meant reading aloud from the list was asked in the first close comment and answered the same day: reading the snippet is not reading, which is why the first Space counts for nothing now. The rule's six cases, the decision's cases and the readings hold the shape; none of this has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T20:05:00.000Z",
    "resolved_at": null
  },
  {
    "id": 540,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-06: what only the tester's ear settles for #27. The Action menu's item heard as Mark as Unread after arrowing onto a read message and as Mark as Read after arrowing onto an unread one, and the context menu's entry the same; M on a message heard as \"read\" or \"unread\", one word, and the list staying on the same row after it; the toolbar button's name after a toggle, read from the button under NVDA's own navigation; and Alt+A, E reaching the item whichever way it goes. The readings hold the letter consumed on a built list, the relabel read back over MSAA on a built toolbar, and the three refresh sites in the source; none of this has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T21:30:00.000Z",
    "resolved_at": null
  },
  {
    "id": 541,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-06.1: what only the tester's ear settles for #76. After Delete on a middle row, NVDA reading the next message's row once and not twice when the watch's re-read follows; after Delete on the last row, the previous row read once; the same after Move to Trash and after a move out of the folder; and the preview showing the landed message. A built list holds the four cases and the focus event the landing raises; which of the two paths the tester met was not watched, because a delete cannot be driven here while the tester's copy is open, and the built list showed the last-row case leaving the control holding nothing",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-18T23:59:00.000Z",
    "resolved_at": null
  },
  {
    "id": 542,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-06.1: what only the tester's ear settles for #83. \"Delete\" heard once at the key and nothing after it when the delete went through, the landed row's reading being enough on its own; the refusal heard with its reason when a delete fails, for instance with the network off; Move to Trash and Move to Folder the same, and Copy to Folder's \"Copied to\" still heard because its row stays; and the status bar's fuller line, \"Deleting\" then \"Moved to Trash\", read on request with NVDA+End and not otherwise. The readings hold the arm to the one word, the outcome to the shown channel when the row left and the spoken one when it stayed, and the shown channel to speaking nothing; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T00:16:00.000Z",
    "resolved_at": null
  },
  {
    "id": 543,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/list_arrival.rs",
    "line": null,
    "description": "11-06.2: what only the tester's ear settles for #87. Tab from the folder tree into the message list, with a folder just opened, NVDA reading the newest message's row once and not twice; F6 into the list the same; back to the tree and Tab again landing on the row that was left, with nothing moved; and an empty folder's list saying \"No messages\" once. A built tree and list hold the four steps and recorded the focus events the arrival raised, the list itself then the row twice where before it raised only the list itself; whether NVDA reads the row once from that sequence is his ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T01:20:00.000Z",
    "resolved_at": null
  },
  {
    "id": 544,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-07: what only the tester's ear settles for #30 and the thread clause of #27. Shift+Down growing the selection with NVDA saying \"selected\" for each row added and reading the row; Shift+Up shrinking it with \"not selected\" for the row that leaves; the count after Ctrl+A on its own; one sentence after Delete over several, \"Delete\" once and the row after the set read once when they have gone; \"3 messages marked read\" once after Mark as Read over three and nothing per message; M on a conversation row marking the whole thread and saying \"1 conversation, 5 messages marked read\"; and the refusal above 5,000 heard once with the count. The built list holds the events the control raises and the source readings hold the seven arms to the set; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T03:44:00.000Z",
    "resolved_at": null
  },
  {
    "id": 545,
    "kind": "deviation",
    "phase": "11",
    "file": "tests/house_style.rs",
    "line": null,
    "description": "11-07: test_every_guard_record_still_names_one_place_in_the_tree exempts a whole file when any record's after is in the tree and its before is not, taken as the file being mid-measurement, so one record whose after matched the rewritten cursor handler by coincidence hid five other records of wx_app.rs whose before had left the tree, and the check passed green over six unmeasurable records; found by an independent one-place count run by hand on 2026-09-19 and the six rewritten and measured. The exemption should be per record, and a pass through it should say so",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T03:44:00.000Z",
    "resolved_at": null
  },
  {
    "id": 546,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/application/moves_waiting.rs",
    "line": null,
    "description": "11-07.1: what only the tester's ear and a real server settle for #86. A move leaving the row at once under NVDA with the cursor read on the next message; Enter on the chosen folder in the Move dialog moving the message; a move made with the network off put back and the refusal heard once when the server answers no; a move made with the network off and the program closed, replayed at the next check after a restart with the message in the destination and not brought back by the check; a copy heard as \"Copied to Work\" once with the row staying. And what a real mail server does with a replayed move, and with a message another client changed meanwhile, which the loopback servers cannot say: #63's move, copy and delete proofs against the tester's account are re-taken after this plan, and ledger 187's three questions about a crossing stay the tester's. Since 11-07.2 (later on 2026-09-19) the crossing completes here first too, so #63's copy and move across accounts are re-taken against that shape, and 547 names what its ear settles",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T06:07:03.000Z",
    "resolved_at": null
  },
  {
    "id": 547,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/application/moves_waiting.rs",
    "line": null,
    "description": "11-07.2: what only the tester's ear and two real servers settle for #86's second half. A move to a folder of the other account leaving the row at once under NVDA with the cursor read on the next message and \"Moved to Work in Home\" shown; the message appearing in that folder of the other account at its next check; with the network off, the row going and coming back with the refusal spoken once when a server answers no; a restart with a crossing waiting finishing it at the next check of either account from the kept message, without a question; a message over 25 MB saying it goes now and its row leaving when the other account has taken it; a copy across accounts heard once. And what two real servers do: the upload's answer at a real destination, what a destination does with a message carrying an identifier it already holds, which the replay reads as an arrival only for a new number, and Gmail's treatment of an appended message; the loopback servers prove the four answers at each of the two servers and the restart from held bytes, and nothing here has met a real account",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T08:20:00.000Z",
    "resolved_at": null
  },
  {
    "id": 548,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-08: what only the tester's ear and account settle for #31. The sender of the message a thread row stands for heard first when the row is read in his inbox, the originator on a thread nothing in it read and the first unread message otherwise; the preview under the row heard to be that message, and Enter opening the conversation window with the cursor on it so Enter again opens it; Space reading it; M marking the whole thread; a thread with everything read reading the oldest; and the text of a conversation arriving from Gmail on landing on its row, which the readings hold to one bounded chunk under the reading gate and no server has been asked for. The cache cases hold the rule through the real listing and the readings hold the window; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T10:35:00.000Z",
    "resolved_at": null
  },
  {
    "id": 549,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/application/server_thread_ids.rs",
    "line": null,
    "description": "11-08.1: what only the tester's Gmail account settles for #88. His split threads showing as one conversation row after the next check, once the once-only pass has given the stored mail its X-GM-THRID, with the count on the row matching what Gmail's own client shows for that thread; a conversation opened with Enter holding the same messages Gmail shows; a reply that arrived without the headers that would have joined it sitting in its thread; two unrelated threads with one subject staying two rows; and the pass itself against Gmail, whether one UID FETCH of the stored numbers for the one field per kept folder is answered whole over his 17,753 messages and what it takes, which the scripted server puts at 54 bytes a message and no real server has been asked. The trace, the readings and the pass are held against the scripted servers; nothing here has met his account",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T13:00:00.000Z",
    "resolved_at": null
  },
  {
    "id": 550,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "11-09: what only the tester's ear settles for #26. The row heard whole and once on Ctrl+Shift+; and on the Action item, each heading then its text in the order the columns are shown, a conversation row reading its own cells, and the refusal heard when the list is not focused; arrowing through the list quiet under an NVDA configuration profile triggered by this application with Row/column headers set to Rows or Off, which nobody has set up; and what Narrator and JAWS need, whose settings the page names without steps since nobody here has read them. The composition, the item, the arm and the channel are held by cases and readings; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T14:20:00.000Z",
    "resolved_at": null
  },
  {
    "id": 551,
    "kind": "todo",
    "phase": "11",
    "file": "docs/KEYBOARD_SHORTCUTS.md",
    "line": null,
    "description": "11-09: an NVDA add-on that quiets the message list's column headers for this program without a configuration profile, if the profile the page describes proves too much to ask of a person setting up. It is a second piece of software installed into NVDA, with its own packaging, versioning and testing, and it serves NVDA alone; nothing under nvda-tests/ is one, that directory drives NVDA rather than extending it. Later work, on the tester's word after the profile has been tried",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T14:20:00.000Z",
    "resolved_at": null
  },
  {
    "id": 552,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/accessibility/feedback.rs",
    "line": null,
    "description": "11-09.1: what only the tester's ear settles for #77. Landing on a message with an attachment heard as the row once, the Attachment column's \"Has attachment\" in NVDA's reading of the row, with the attachment tone beside it and nothing spoken for the event; and with \"Show events in the status bar\" off, the tone alone. A fresh profile hearing every event's tone from the start. The default channels, the fallback that never speaks for the event, the tester's stored profile reaching the same default, a profile that chose silence keeping it, the Feedback tab's boxes for the event and the cursor handler adding nothing spoken are held by cases and readings; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T15:20:00.000Z",
    "resolved_at": null
  },
  {
    "id": 553,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/presentation/accessibility/feedback.rs",
    "line": null,
    "description": "11-09.1: the reproduction of #81 on purpose, and the sounds heard again after a real device change, which need a hand on the machine's Sound settings under a running build. The steps: the program built from the branch and started with WIXEN_MAIL_DATA set to an empty folder (src/common/paths.rs, the one override the paths module honours), so it touches nothing under the wixen-mail folder in LOCALAPPDATA; earcons on; the default output device changed in Windows Sound settings; an event with a sound triggered, Settings saved is the nearest; whether it went silent and what the log said at debug; then a headset unplugged with a sound due, and whether the sound after it plays. Never the installed binary, and never without the override. What the cases prove instead is the seam the crates document: the flag the stream's error callback sets, the reopen before the next sound, the reopen after ten seconds' quiet, the outage told once and the resume",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T15:50:00.000Z",
    "resolved_at": null
  },
  {
    "id": 554,
    "kind": "unrun-verify",
    "phase": "11",
    "file": "src/application/snippet.rs",
    "line": null,
    "description": "11-09.2: what only the tester's ear settles for #82. A row whose message opens with an address heard as the message's first sentence and not the address spelled out; a newsletter's row heard as its first real line and not \"View this email in your browser\"; a reply's row heard as the new words and not the quote; and after the next start, the rows of messages downloaded before this build heard the new way, the log's line saying how many were put right and in how long on his 17,753. The rules are held one by one in application::snippet's cases and the two places they are reached from in tests/a_snippet_is_the_first_relevant_words.rs; none of it has been heard",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T17:45:00.000Z",
    "resolved_at": null
  },
  {
    "id": 555,
    "kind": "todo",
    "phase": "11",
    "file": "src/application/long_text.rs",
    "line": null,
    "description": "11-09.2: the reader gives a layout table's cell as one run with no space between the blocks in it. Read on 2026-09-19 through pieces_of_markup over the Substack message saved as the public fixture for #90: the whole message arrives as one Table piece whose one cell reads \"Forwarded this email? Subscribe here for moreTop three ways ... seven daysActions speak louder than wordsGary MarcusSep 19image with no description\", every block's last word run into the next block's first. The snippet rules then skip the whole cell as boilerplate, since it holds the forwarding line, and the row says the subtitle the hidden preheader carried, which is a fair hint by luck rather than by rule. The reader's business, not the snippet's: a cell's blocks want a space or a line between them, in the block walk in long_text.rs, with a case over a cell holding two paragraphs; 18 records name the file",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-19T17:45:00.000Z",
    "resolved_at": null
  }
]
````

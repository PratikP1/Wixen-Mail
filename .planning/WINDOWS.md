---
schema_version: 1
open_count: 183
waived_count: 0
fixed_count: 15
total_count: 198
last_updated: 2026-09-08T19:25:35.807Z
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
| 11 | 02 | unrun-verify | src/application/mail_sync.rs |  | The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names | open |  | 2026-09-01T03:56:05.356Z |  |
| 12 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The offer button and its experimental sentence have not been heard under a screen reader: whether the button is announced with its full label after a saved search, and whether the message text topic is heard rather than coalesced away, is unverified by ear | open |  | 2026-09-01T03:56:05.812Z |  |
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
| 64 | 03 | unrun-verify | src/application/mail_controller.rs |  | Whether a real provider accepts a fresh sign-in straight after it has dropped a connection, or treats it as something to slow down or refuse, is unknown. The single retry is proved against a loopback server that hangs up on command and answers the next connection immediately. No account has ever been used with this program, so nothing here has met a provider's real behaviour on reconnect, including whether it counts against a connection limit. | open |  | 2026-09-04T21:03:06.059Z |  |
| 65 | 03 | unrun-verify | src/application/mail_session.rs |  | What a real provider does with a session held open and idle for minutes is unknown, and the whole point of holding one is that it sits idle between commands. Whether providers drop an idle IMAP session at all, how soon, and whether they say anything before they do, has never been observed by this program: no account has ever been used with it. The reconnect exists because a drop is expected, and that expectation is reasoning rather than a measurement. | open |  | 2026-09-04T21:03:24.528Z |  |
| 66 | 03 | unrun-verify | src/presentation/wx_app.rs |  | Whether the refusal after a failed retry is heard once rather than once per failed request is unverified by ear. It reaches somebody through each site's existing reporting, which is ErrorOccurred for the flag path and CommandRefused for the folder commands, and both announce at High priority through accessibility::announce. That is structure, not experience: nobody has heard it with NVDA, and a mailbox where every command meets a dead connection would produce one of these per command with nothing coalescing them, which is exactly the flooding guardrail 5 is about. | open |  | 2026-09-04T21:03:34.593Z |  |
| 67 | 03 | unrun-verify | src/application/mail_session.rs |  | The connection budget of two per account is counted against a loopback server and has never been counted against a provider. Whether two per account is welcome, what a provider counts as a connection when several accounts sit on the same one, and whether the IDLE connection and the working session are counted together, are all unknown. Gmail's limit of fifteen per account is the number the requirement's evidence records rather than one this program has ever approached. | open |  | 2026-09-04T21:03:43.871Z |  |
| 68 | 03 | deviation | src/service/protocols/imap.rs |  | folder_counts has the same shape select_folder was fixed for and is not fixed. It calls async-imap's session.status, whose parser reads responses until the stream ends and hands back what it collected, so a connection dropping mid-command comes back as Ok with nought messages and nought unread. Corrected on review 2026-09-04: this said a wrong number rather than a deletion, and that understates it. A count of nought is what disarms listing_contradicts_the_count, which is listed == 0 && counted > 0 and is the only check between a truncated listing and an emptied folder. select_folder erroring now aborts the sync before list_uids is reached, which closes the path, so this is latent rather than live; it would become live again if anything ever reads the count without the SELECT in front of it. Same defect underneath: a command that never completed reported as one that did. Fixing it means writing STATUS as a command line through read_command, the way select_folder now is. | open |  | 2026-09-04T21:03:53.498Z |  |
| 69 | 03 | deviation | src/presentation/wx_app.rs |  | Checking for mail used to refuse an unusable port with the value it could not read, 'has an IMAP port that is not a number: 14 3'. All twelve sites lost their own port check when they went through the held session, because a_session_at asks the same question and answers it in the same words, so each was a second answer to one question. Eleven lost nothing by that; this one lost the offending value, which is the part somebody fixing it needs. The value is visible in the account settings screen. | open |  | 2026-09-04T21:04:03.565Z |  |
| 70 | 03 | unrun-verify | src/application/finding_what_was_deleted.rs |  | Whether any provider grants CONDSTORE, which is what the resume needs. imap/abilities.rs asserts that Gmail never has. Fastmail and current Dovecot advertise it in the capability lists this project models them on, and no capability list has ever been read off a real server here. If none of the providers people use grants it, SCALE-01's saving applies to nobody and every folder is read out in full on every sync, which is what happens today anyway. | open |  | 2026-09-05T01:35:34.235Z |  |
| 71 | 03 | unrun-verify | src/application/finding_what_was_deleted.rs |  | Whether a hand-built SELECT with QRESYNC parses back at all, which is what the declared and unbuilt VANISHED member would need. async-imap 0.11.3 has no ENABLE and no select_qresync, so it goes through run_command, and the mailbox response that comes back is one async-imap's own select parses. imap-proto already parses Response::Vanished. Whether the raw select parses and whether VANISHED reaches the closure has never been run against a server, and it is the whole cost of the second implementor. | open |  | 2026-09-05T01:35:52.234Z |  |
| 72 | 03 | unrun-verify | src/application/asking_for_a_whole_folder.rs |  | Whether a provider tolerates a whole-folder request. It asks for a folder five hundred messages at a time, without stopping, until the folder is here. Ledger 11 records the same gap for the bulk body fetch and this is the same shape at a different granularity: a provider is entitled to refuse, throttle, or disconnect, and nothing on this side can find out which. Two things follow that no test here can settle: how many chunks a provider allows before it slows down, and whether a disconnect part way is reported as the request stopping short rather than as the folder being finished. Marked experimental on the menu item and in its description. | open |  | 2026-09-05T01:36:00.334Z |  |
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
| 97 | 04 | deviation | src/service/outlook_data_file.rs |  | A message imported from an Outlook data file carries no List-Unsubscribe, so blocking its sender gets no warning. The importer rebuilds a message from the pieces a PST holds and the transport headers are not among the pieces it reads. Left as None with a comment saying so rather than papered over: recovering it means reading the header property out of the file and writing it into the bytes the importer then re-parses, which is its own change. Messages filed from a sent copy and from an archive read through mime::parse do carry it. | open |  | 2026-09-05T16:00:13.954Z |  |
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
| 187 | 04.1 | unrun-verify | src/application/mail_across_accounts.rs |  | No message has been moved between two real accounts. Every assertion is against loopback servers this project wrote, and three questions only a live account answers are named in docs/changelog.md under known limitations: what a real provider does with an upload of a ten megabyte message, what Gmail makes of a message uploaded from another account when it treats a copy as a label, and what any provider does when a message arrives carrying an identifier the destination already holds. The last is not idle, because that identifier is what the program asks about when an upload's answer never arrives | open |  | 2026-09-08T12:00:00.000Z |  |
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
    "description": "The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names",
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
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:05.812Z",
    "resolved_at": null
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
    "description": "Whether a real provider accepts a fresh sign-in straight after it has dropped a connection, or treats it as something to slow down or refuse, is unknown. The single retry is proved against a loopback server that hangs up on command and answers the next connection immediately. No account has ever been used with this program, so nothing here has met a provider's real behaviour on reconnect, including whether it counts against a connection limit.",
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
    "description": "What a real provider does with a session held open and idle for minutes is unknown, and the whole point of holding one is that it sits idle between commands. Whether providers drop an idle IMAP session at all, how soon, and whether they say anything before they do, has never been observed by this program: no account has ever been used with it. The reconnect exists because a drop is expected, and that expectation is reasoning rather than a measurement.",
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
    "file": "src/application/asking_for_a_whole_folder.rs",
    "line": null,
    "description": "Whether a provider tolerates a whole-folder request. It asks for a folder five hundred messages at a time, without stopping, until the folder is here. Ledger 11 records the same gap for the bulk body fetch and this is the same shape at a different granularity: a provider is entitled to refuse, throttle, or disconnect, and nothing on this side can find out which. Two things follow that no test here can settle: how many chunks a provider allows before it slows down, and whether a disconnect part way is reported as the request stopping short rather than as the folder being finished. Marked experimental on the menu item and in its description.",
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
    "description": "A message imported from an Outlook data file carries no List-Unsubscribe, so blocking its sender gets no warning. The importer rebuilds a message from the pieces a PST holds and the transport headers are not among the pieces it reads. Left as None with a comment saying so rather than papered over: recovering it means reading the header property out of the file and writing it into the bytes the importer then re-parses, which is its own change. Messages filed from a sent copy and from an archive read through mime::parse do carry it.",
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
    "description": "No message has been moved between two real accounts. Every assertion is against loopback servers this project wrote, and three questions only a live account answers are named in docs/changelog.md under known limitations: what a real provider does with an upload of a ten megabyte message, what Gmail makes of a message uploaded from another account when it treats a copy as a label, and what any provider does when a message arrives carrying an identifier the destination already holds. The last is not idle, because that identifier is what the program asks about when an upload's answer never arrives",
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
  }
]
````

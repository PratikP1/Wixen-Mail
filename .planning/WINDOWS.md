---
schema_version: 1
open_count: 40
waived_count: 0
fixed_count: 9
total_count: 49
last_updated: 2026-09-03T08:58:32.016Z
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
| 7 | 01 | stub | src/application/thread_identity.rs |  | A conversation root arriving after a message that names it is not merged: the link lives only in the other message's stored refs_header, which no index can search. Needs an identifier-to-conversation table | open |  | 2026-08-31T05:13:49.247Z |  |
| 8 | 01 | deviation | src/data/message_cache/messages.rs |  | messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column | open |  | 2026-08-31T05:13:50.314Z |  |
| 9 | 01 | deviation | .planning/phases/01-folders-and-conversations/01-13-PLAN.md |  | Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one | open |  | 2026-08-31T05:13:51.507Z |  |
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
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:49.247Z",
    "resolved_at": null
  },
  {
    "id": 8,
    "kind": "deviation",
    "phase": "01",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:50.314Z",
    "resolved_at": null
  },
  {
    "id": 9,
    "kind": "deviation",
    "phase": "01",
    "file": ".planning/phases/01-folders-and-conversations/01-13-PLAN.md",
    "line": null,
    "description": "Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:51.507Z",
    "resolved_at": null
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
  }
]
````

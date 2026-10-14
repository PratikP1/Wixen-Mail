# Phase 13.2: how the audit's findings are grouped into plans

Written 2026-10-14 against `main` at `dc592d07`, from the audit of the same day:
`audit-2026-10-14-findings.json` (key `all`, 53 findings, each with a verifier's verdict) and
`ponytail-audit-2026-10-14.md` (the report Pratik read: 20 items, which carry 23 of the 53
findings, and "29 smaller findings" left out). Pratik's words: "I want all of the items that
need to be fixed as a phase including the ones not reported to me, the 28/29 items."

The finding numbers below (#1 to #53) are positions in the `all` array, read with Python's
`json` module, not with a summary. The report's own numbers are written R1 to R20. The left-out
count reconciles: 30 findings were not reported, 21 graded should and 9 nice by their verdicts,
and two of them (#8 and #53) are the same two functions, which makes the report's 29.

Every finding is in exactly one plan. Duplicates are merged into one plan and named in the
duplicates section. All 53 verdicts say the finding is real; many correct the finding's own
fix, so a plan works from the verdict's corrected fix, not the `fix` field.

## The plans

One plan per wave, chained, each plan depending on the one before it. Until two plans were
split off on 2026-10-14, 13.2-20.1 from 13.2-20 and 13.2-26.1 from 13.2-26, this said "plan
13.2-NN is wave NN and depends on 13.2-(NN-1)". Later the same day four plans for the
regressions Pratik reported that day went in front of the chain, 13.2-01 to 13.2-03 with
13.2-03.1, and every audit plan moved up by three: the audit's first plan, 13.2-01 until then,
is 13.2-04, and its last is 13.2-32. Every plan number in this file is the new one. The waves
now run from 1 to 35, and the phase README's plan table gives each.

The four regression plans come first, because Pratik meets all four every time he uses the
mailbox, and phase 14's delete sitting hears the third; `REGRESSIONS.md` sets out each. After
them the order is the brief's: what phase 14's sittings prove, then untrusted input and
privacy, then the PIM syncs, then announcements, then settings, then the gate and tooling,
then the deletions. The last plan closes the phase and carries the full gate.

"Spoken or shown" is yes when a sentence, a dialog, or what a window lists changes; such a plan
pushes its branch and opens a pull request so the NVDA job runs before the merge. Size is the
task count the plan should need: S is 2 tasks, M is 3, L is 4.

| Plan | Title | Findings carried | Main files | Spoken or shown | Size |
|------|-------|------------------|------------|-----------------|------|
| 13.2-01 | Enter opens a message that is alone in its conversation, and says why when there is nothing to open | None: Pratik's report 1 of 2026-10-14, issue 1 in REGRESSIONS.md | src/presentation/wx_app.rs (the conversation arm of `msg_list.on_item_activated`); src/presentation/ui_types.rs (a stale comment); tests/a_conversation_row_stands_for_one_message.rs; docs/KEYBOARD_SHORTCUTS.md | Yes: a reader window opens where nothing did, and a sentence when nothing can open | M |
| 13.2-02 | A mark, a star, a label, an undo or a delete made here shows on its conversation row at once, and the cursor lands after a row leaves | None: Pratik's reports 2 and 3, issues 2 and 3 in REGRESSIONS.md | src/data/message_cache/messages.rs (a narrowed conversation listing); src/presentation/view_state.rs; src/presentation/wx_app.rs (one catch-up function and its call sites); tests/a_change_made_here_reaches_the_conversation_rows.rs; docs/development/measurements.md | Yes: the Unread and Thread cells, and where the cursor lands | M |
| 13.2-03 | The conversation window acts on the message under its cursor: the menu key, Delete, Shift+Delete and M | None: Pratik's report 4, issue 4 in REGRESSIONS.md | src/presentation/wx_thread_view.rs; src/presentation/wx_app.rs; src/application/choosing_messages.rs; tests/the_conversation_window_acts_on_the_message_under_its_cursor.rs and three other test files; docs/KEYBOARD_SHORTCUTS.md | Yes: a context menu and keys in the conversation, and what it lists after a command | M |
| 13.2-03.1 | The conversation window gets the rest of the message keys, and marks a message read after the wait | None: the half of Pratik's report 4 that 13.2-03 leaves | src/presentation/wx_thread_view.rs; src/presentation/wx_app.rs; tests/the_conversation_window_acts_on_the_message_under_its_cursor.rs; tests/a_conversation_row_stands_for_one_message.rs; docs/KEYBOARD_SHORTCUTS.md | Yes: the keys and the read mark in the conversation | M |
| 13.2-04 | The Outbox claims a message before sending it, so two passes never send it twice and Undo Send never says "taken back" for one that went | #31 and #6 (duplicates, one fix) | src/data/message_cache/outbox.rs; src/data/message_cache/mod.rs (an additive `sending_since` column); src/presentation/wx_app.rs (flush_outbox, undo_send, start-up clears claims); src/application/sending_later.rs (comment) | Yes: Undo Send's answer | M |
| 13.2-05 | Sending a draft takes its filed copy out of Drafts, here and on the server, and Undo Send's too-late path files no copy of a sent message | #35 | src/presentation/wx_app.rs (the Send arm, save_as_draft, file_draft_copy, undo_send's too-late branch); src/application/draft_copy.rs; src/data/message_cache/drafts.rs; tests/wired.rs; docs/changelog.md (the claim at about 8510) | Yes: what Drafts lists | M |
| 13.2-06 | Saving accounts never deletes another account or resets the chosen order, and a removed or switched-off account's inbox watch stops | #3, #33 | src/data/message_cache/accounts.rs; src/presentation/wx_account_manager.rs (a second account on one address refused with a sentence); src/presentation/wx_app.rs (handle_account_mgr, move_the_chosen_account, the MailboxChanged arm, spawn_mail_watch, start_the_watches_whose_wait_is_over) | Yes: the refusal sentence, the account order | L |
| 13.2-07 | Delete goes to the server's declared Trash, never a person's folder named Bin or Deleted, and Empty Folder says what the server really did | #9, #34 | src/service/protocols/imap/special_use.rs; src/service/protocols/imap.rs (list_folders); src/application/destinations.rs; src/presentation/wx_app.rs (spawn_the_folder_empty); src/application/how_far_it_got.rs; docs/changelog.md (the "Running it again finishes the job" line) | Yes: Empty Folder's sentence | M |
| 13.2-08 | Every new message reaches the rules once: more than 500 between checks are all checked, and a rule's move settles its row instead of leaving a second copy | #10, #12 | src/application/mail_sync.rs (carry_out_the_moves, uids_to_fetch); src/data/message_cache/messages.rs (the test at about 3209 that expects the duplicate); src/data/message_cache/moves_waiting.rs (the settling helpers reused) | Yes: the new-mail count, what the folder lists | M |
| 13.2-09 | The download of everything stops asking for text that always fails, and re-reads the open folder at most every few seconds without speaking each chunk | #11, #36 | src/presentation/wx_app.rs (the runner arm at about 30061, the MoreOfTheFolderArrived arm, reread_folder_if_open, the MessagesLoaded count line); src/application/mail_sync.rs (fetch_over_a_mailbox); src/data/message_cache/bodies.rs; src/application/bringing_everything_down.rs; tests/everything_comes_down_without_being_asked.rs | Yes: the per-chunk count line and progress lines | M |
| 13.2-10 | One check per account at a time, so new mail is announced once and rules run once, and a check's flag pass reads the account once instead of per message | #15, #14 | src/presentation/wx_app.rs (spawn_mail_sync and its four callers); src/application/mail_session.rs; src/application/mail_sync.rs (the flag loop at about 1946-1999); src/data/message_cache/folders.rs (folder_account, a copy of account_of_folder); src/data/message_cache/tags.rs | Yes: the new-mail announcement | M |
| 13.2-11 | Changes kept while offline are replayed only into the folder numbering they were kept under, and a flag change that reached the server clears its stale waiting twin | #5, #2 | src/data/message_cache/moves_waiting.rs; src/data/message_cache/waiting_flag_changes.rs; src/data/message_cache/mod.rs (additive uid_validity columns); src/application/moves_waiting.rs (replay_one); src/application/mail_controller.rs (the MailboxStatus it now throws away); src/application/flag_changes_waiting.rs; src/presentation/wx_app.rs (the flag push's Ok branch, the replay) | Yes: read and flagged state shown; any sentence for a replay put back | L |
| 13.2-12 | A stranger's deeply nested HTML falls back to plain text instead of overflowing a thread's stack | #17 | src/application/hidden_text.rs; src/vendor/paperback/html_to_text.rs (process_node, collect_text_into); src/vendor/paperback/table_text.rs; src/application/long_text.rs; the worker entry points the verdict names (snippet_of in src/data/message_cache/bodies.rs, words_of_markup through src/data/message_cache/searching.rs) | Yes: what the reader shows for such a message | M |
| 13.2-13 | Reply and Forward load none of the sender's remote pictures, and a meeting change in a message the program labelled Phishing or Spam is not applied | #19, #21 | src/presentation/editor_document.rs (a Content-Security-Policy for images); docs/privacy.md; src/application/reading_a_message.rs; src/application/meeting_changes.rs (a new reason, placed after `wanted` is known); .planning/WINDOWS.md (637 narrowed) | Yes: the editor's pictures, the meeting sentence | M |
| 13.2-14 | A webcal feed that never answers no longer freezes the window or fills memory, and one unreadable POP message no longer stops that account's mail | #20, #13 | src/service/ical_subscription.rs (timeout, size cap); src/presentation/wx_app.rs (open_what_windows_handed_over off the UI thread, the background refresh); src/presentation/ui_types.rs (a UIUpdate for the result if needed); src/application/pop_sync.rs; src/service/mime.rs | Yes: the subscription's sentences, a kept POP message | L |
| 13.2-15 | A browser sign-in's token goes only to the provider's own servers, and is kept only when the account that consented is the account it was asked for | #18, #23 | src/application/mail_auth.rs; src/application/who_runs_the_mail.rs; src/service/oauth.rs (login_hint, the check before store_tokens); src/presentation/wx_account_manager.rs (both sign-in paths); src/application/who_holds_the_calendars.rs; the threat row T-13-PK-02 moved from accept to mitigate | Yes: two refusal sentences | L |
| 13.2-16 | Saved attachments carry the Mark of the Web, and the update check trusts more than a signer's display name | #22, #24 | src/presentation/wx_app.rs (fetch_and_write_attachment); src/service/update_download.rs (whether_that_signer_is_ours, WHO_SIGNS_THIS) | Yes: the update check's refusal | M |
| 13.2-17 | Google contacts keep syncing after the marker expires, and a contact made here goes up to Google and Outlook on every sync | #25, #26 | src/application/contacts_sync.rs (is_expired_sync_token, the two create guards, the tests at about 6036 and 6903) | No | M |
| 13.2-18 | The Outlook calendar sees past the dates of its first sync, and Week and Month views show events made here on their first day | #27, #4 | src/application/calendar.rs (sync_microsoft_calendar, its 410 retry, the @removed handling); src/data/message_cache/calendar.rs (events_that_overlap_query); src/application/caldav_sync.rs (the named read window reused) | Yes: what Week and Month list | L |
| 13.2-19 | A subscribed feed keeps a repeating event and its changed days apart, and the calendar's XML reader decodes escapes like the address book's does | #28, #46 | src/application/caldav_sync.rs (refresh_subscription); src/service/caldav.rs (extract_xml_value); src/service/carddav.rs (xml_unescaped moved, value_in dropped); src/service/free_busy.rs (the third decoder copy) | Yes: event titles and the series a feed shows | L |
| 13.2-20 | A tick or a note edit made while a sync is pushing is kept and sent, not wiped | #1, its tasks and notes | src/data/message_cache/tasks.rs (mark_task_sent, rename_task); src/application/tasks_sync.rs (settle); src/presentation/managers.rs; src/data/message_cache/notes.rs (a_backend_took_the_note); src/application/notes_sync.rs; src/presentation/wx_app.rs; tests/wired.rs | No | M |
| 13.2-20.1 | An event edit made while a sync is pushing is kept and sent, not wiped | #1, its events | src/data/message_cache/calendar.rs (mark_calendar_event_sent, no production caller); src/application/calendar.rs (settled); src/application/caldav_sync.rs (settled_here, settled_here_for_an_occurrence) | No | S |
| 13.2-21 | A full read takes off what the provider deleted, the Google calendar falls back to a full read only on an expired marker, and the contacts syncs read the address book once | #29, #30 | src/application/contacts_sync.rs; src/application/calendar.rs (read_one_google_calendar); src/application/every_google_calendar.rs (the test pinning the any-error fallback); src/application/carddav_sync.rs (take_in_one_card); src/data/message_cache/contacts.rs | No | L |
| 13.2-22 | Announcements released together are joined into one line, most urgent first, so the first is never overwritten | #32 | src/presentation/accessibility.rs (flush_announcements, a hold while the tick drains); src/presentation/accessibility/screen_reader.rs (handed_over_together); src/presentation/wx_app.rs (the tick loop at about 6392) | Yes: every joined announcement | M |
| 13.2-23 | Settings are written to a temporary file and renamed, so a crash mid-save loses nothing | #7 | src/data/config.rs (save_app_config, the account files' save, the load that fails on a partial file) | No, unless the planner adds a sentence for a settings file that could not be read | S |
| 13.2-24 | The release gate and the Linux and macOS jobs fetch the whole history, the guard reads every workflow, and the release gate holds a read-only token | #39, #42 | .github/workflows/release.yml; .github/workflows/other-platforms.yml; tests/house_style.rs (the depth guard at about 4665); .planning/WINDOWS.md (464 corrected and closed; 616 left to Pratik) | No | M, plus a checkpoint |
| 13.2-25 | The hook refuses a commit whose working tree differs from what is staged, and a red commit with a crashed test binary is refused | #40, #44 | scripts/check.sh; scripts/red-commit.sh; scripts/check.test.sh; scripts/red-commit.test.sh | No | M |
| 13.2-26 | The scoped run reaches the 73 guard records whose red tests live outside the broken file's module, and a build.rs change earns its tests | #38, #41 | scripts/check.sh (the registry scan, the_scoped_runs); scripts/which-checks.sh; scripts/check.test.sh; scripts/which-checks.test.sh; CLAUDE.md (where the rules changed) | No | M |
| 13.2-26.1 | The documents-only clippy question goes to Pratik, and his answer is carried out | #45 | scripts/check.sh (the docs_only branch), scripts/check.test.sh, scripts/which-checks.sh and CLAUDE.md, each only on a yes | No | S, plus a checkpoint |
| 13.2-27 | An all-users uninstall says that the person's mail and settings may still be on this computer, and its comment says what the flag really does | #43 | installer/Wixen-Mail-Setup.iss | Yes: the uninstall message | S |
| 13.2-28 | The pretend accessibility tree is deleted and the spoken-line log becomes test-only | #37 and #47 (duplicates, one fix) | src/presentation/accessibility.rs; src/presentation/accessibility/keyboard.rs; focus.rs; automation.rs; src/presentation/accessibility/screen_reader.rs (event_log); guards/guards.toml (the shadow-tree records, keeping the feedback_settings one); docs/architecture.md | Yes: the start-up line "Focus moved to Folders" goes | M |
| 13.2-29 | The original skeleton's account manager, search engine and cache service are deleted, and docs/architecture.md drawn from the real modules | #49 | src/application/accounts.rs; src/application/search.rs; src/service/cache.rs; src/common/types.rs; src/data/email_providers.rs; src/data/account.rs; src/application/mod.rs; src/service/mod.rs; tests/integration_tests.rs; docs/architecture.md | No | M |
| 13.2-30 | The translation registry that translates nothing is deleted, and three dependencies leave the build or move to dev-dependencies | #48, #50 | src/service/spellcheck/mod.rs; src/vendor/paperback/html_to_text.rs; src/vendor/paperback/mod.rs; src/service/mod.rs; Cargo.toml; Cargo.lock; src/service/outward.rs; src/service/security.rs; .cargo/audit.toml | No | M |
| 13.2-31 | Functions nothing calls are deleted: the ten the lean pass named, three IMAP helpers, and the per-folder store reads | #51, #16, #8 and #53 (overlapping, merged) | src/presentation/html_renderer.rs; src/service/pgp/mod.rs; src/service/outlook_data_file.rs; src/service/update_download.rs (two unused host constants); src/vendor/paperback/table_text.rs, text.rs, html_to_text.rs; src/service/protocols/imap.rs; src/application/mail_controller.rs; src/data/message_cache/messages.rs; tests/the_list_at_two_hundred_thousand_rows.rs; docs/development/measurements.md | No | L |
| 13.2-32 | vCard unfolding calls the iCalendar unfolding instead of copying it, and the phase closes with the full gate | #52 | src/data/message_cache/contacts.rs; src/service/caldav.rs (unfolded); guards/guards.toml (two records with byte-identical anchors); the full gate by hand, `scripts/check.sh all` | No | M |

Every plan also writes the files written by rule where it applies: `docs/changelog.md` for a
user-visible change, `guards/guards.toml` for a guard written, re-measured or retired,
`.planning/WINDOWS.md` for a ledger row opened or closed, and `docs/development/measurements.md`
where it takes a figure. Because the plans are chained one per wave, two plans touching the
same file never run at once. The rows for 13.2-20.1 and 13.2-26.1 were added on 2026-10-14,
when the plan check split #1's events from 13.2-20 and #45 from 13.2-26. The four rows above
13.2-04 were added the same day, when the regression plans went in front; they carry Pratik's
reports and no audit finding.

## Every finding, once

| # | Finding | Area, verdict | In the report | Plan |
|---|---------|---------------|---------------|------|
| 1 | A change made while a push is in flight is marked sent and never sent (tasks, events, notes) | store, should | R17 | 13.2-20, 13.2-20.1 |
| 2 | A flag change kept from offline later overwrites a newer change that already reached the server | store, should | not reported | 13.2-11 |
| 3 | Saving accounts with INSERT OR REPLACE deletes another account with the same address and wipes the chosen account order | store, must | R3 | 13.2-06 |
| 4 | The calendar's date window compares text in mixed formats, so events made here on the first day of a Week or Month view are missing | store, must | R13 | 13.2-18 |
| 5 | Waiting moves and deletes are replayed by UID without the folder's UIDVALIDITY | store, should | not reported | 13.2-11 |
| 6 | Outbox rows are never claimed, so Undo Send can say 'taken back' while the send loop sends the message | store, should | R1 (with #31) | 13.2-04 |
| 7 | Settings are rewritten in place, so a crash mid-write loses every setting | store, should | not reported | 13.2-23 |
| 8 | Three store reads have no production caller | store, nice | not reported (overlaps #53) | 13.2-31 |
| 9 | A user folder named Deleted, Bin or Trash anywhere in the tree can become the account's Trash, so deletes go there and the unattended emptying expunges it | protocols, must | R2 | 13.2-07 |
| 10 | A rule's move leaves a row the destination never reconciles, so every message a rule files shows twice in the destination | protocols, must | R7 | 13.2-08 |
| 11 | The text download spins forever when one or two messages always fail to fetch | protocols, must | R9 | 13.2-09 |
| 12 | When more than 500 messages arrive between checks, the overflow skips the rules and the new-mail count | protocols, must | R8 | 13.2-08 |
| 13 | One POP message with no readable header block stops that account's mail permanently | protocols, should | not reported | 13.2-14 |
| 14 | Each check reads the flags of every held message and makes about six SQL statements per message outside a transaction | protocols, should | not reported | 13.2-10 |
| 15 | Nothing stops two checks of the same account running at once, so new mail is announced twice and rules run twice | protocols, should | R19 | 13.2-10 |
| 16 | Four IMAP helpers have no production caller | protocols, nice | not reported | 13.2-31 |
| 17 | A stranger's deeply nested HTML overflows the UI thread's stack and kills the whole program | security, must | R4 | 13.2-12 |
| 18 | An OAuth token for the whole mailbox is sent to whatever IMAP or SMTP host the account names | security, should | R18 | 13.2-15 |
| 19 | Reply and Forward fetch the sender's remote pictures and tracking pixels, which the reader always holds back | security, must | R5 | 13.2-13 |
| 20 | Following a webcal: link freezes the window with no timeout and buffers an unbounded feed | security, should | not reported | 13.2-14 |
| 21 | A meeting is moved on opening even when the program itself has labelled the message Phishing | security, should | not reported | 13.2-13 |
| 22 | Saved attachments carry no Mark of the Web | security, should | not reported | 13.2-16 |
| 23 | A browser sign-in is filed under the account with no check of which account actually consented | security, should | not reported | 13.2-15 |
| 24 | The update check trusts any installer signed by anyone named 'Pratik Patel' | security, should | not reported | 13.2-16 |
| 25 | Google contacts stop syncing for good a week after the first full read | pim, must | R11 (with #26) | 13.2-17 |
| 26 | A contact typed here after the first sync never reaches Outlook, and reaches Google only on a full read | pim, must | R11 (with #25) | 13.2-17 |
| 27 | The Outlook calendar only ever sees the date range of its first sync | pim, must | R12 | 13.2-18 |
| 28 | A subscribed calendar feed shows a moved or cancelled day of a repeating event wrongly | pim, must | R14 | 13.2-19 |
| 29 | A full read never removes what the provider deleted while the marker was stale | pim, should | not reported | 13.2-21 |
| 30 | The contacts sync reloads the whole address book once per contact it reads | pim, should | not reported | 13.2-21 |
| 31 | Two Outbox passes can send the same message twice | ui, must | R1 (with #6) | 13.2-04 |
| 32 | Announcements made close together overwrite each other, so only the last one is heard | ui, must | R10 | 13.2-22 |
| 33 | Removing or disabling an account leaves its inbox watch running | ui, should | not reported | 13.2-06 |
| 34 | Empty Folder says a server folder was emptied when the server only marked or copied the messages | ui, should | not reported | 13.2-07 |
| 35 | Sending a draft leaves its copy in the Drafts folder, here and on the server | ui, must | R6 | 13.2-05 |
| 36 | The download of everything re-reads the whole open folder on the UI thread after every chunk | ui, should | not reported | 13.2-09 |
| 37 | A pretend accessibility tree that only tests read, and an event log that grows for the life of the process | ui, should | R20 (with #47) | 13.2-28 |
| 38 | 73 library guard records never run on the commits that break them, and since 2026-09-23 the merge does not run them either | tooling, should | not reported | 13.2-26 |
| 39 | Three CI jobs run the suite on a one-commit checkout, so the release gate and both other-platform jobs cannot pass, and the guard that should say so reads only ci.yml | tooling, must | R16 | 13.2-24 |
| 40 | The hook decides on the staged files but builds and tests the working tree, so a commit can pass while its own snapshot is broken | tooling, should | not reported | 13.2-25 |
| 41 | A commit that changes only build.rs or rust-toolchain.toml runs no library or unit test at all | tooling, nice | not reported | 13.2-26 |
| 42 | The release quality gate runs every dependency's build script and the whole suite holding a token that can push to the repository | tooling, nice | not reported | 13.2-24 |
| 43 | An all-users uninstall erases the administrator's data, not the mail of the person who used the program, and the comment says the opposite | tooling, should | not reported | 13.2-27 |
| 44 | A red commit passes even when a test binary in its scoped run crashes, because red mode discards cargo's exit status and the verdict reads only FAILED lines | tooling, nice | not reported | 13.2-25 |
| 45 | A documents-only commit pays a full clippy run and a forced crate rebuild for Rust that by definition did not change | tooling, nice | not reported | 13.2-26.1 |
| 46 | The calendar's XML reader never decodes XML escapes, though the address-book copy of the same reader does | lean, must | R15 | 13.2-19 |
| 47 | A shadow accessibility tree that reaches no screen reader, and an announcement log that keeps every spoken line for the whole session | lean, should | R20 (with #37) | 13.2-28 |
| 48 | A second translation system that translates nothing, and says in its comments that it does | lean, should | not reported | 13.2-30 |
| 49 | The original skeleton's account manager, search engine and cache service still exist, used only by tests/integration_tests.rs and described in docs/architecture.md as the real design | lean, should | not reported | 13.2-29 |
| 50 | Three dependencies are declared for code that does not use them, and three comments say otherwise | lean, should | not reported | 13.2-30 |
| 51 | Functions with no caller anywhere, two of them carrying documentation that says something calls them | lean, nice | not reported | 13.2-31 |
| 52 | vCard line unfolding is a hand copy of the iCalendar unfolding, kept in step by comments | lean, nice | not reported | 13.2-32 |
| 53 | The per-folder POP readers that caused re-downloads are still public, beside the account-wide ones that replaced them | lean, nice | not reported (overlaps #8) | 13.2-31 |

Counted: 53 rows, 23 reported in R1 to R20 and 30 not reported, every plan from 13.2-04 to
13.2-32, with 13.2-20.1 and 13.2-26.1, named at least once. 13.2-01 to 13.2-03.1 carry Pratik's
four reports and no finding, so this table names none of them.

## Duplicates and overlaps

- **#6 and #31** are one defect seen from two sides: nothing claims an Outbox row. The verdict
  for #6 says the claim column closes both; the verdict for #31 says the AtomicBool pair it
  proposed loses a request and leaves the flag set after a panic. One fix, plan 13.2-04.
- **#37 and #47** are the same shadow tree and the same `event_log`, found by the ui and the
  lean auditors. #47's verdict counts seven guard records in the range, not six, and keeps the
  feedback_settings one. One fix, plan 13.2-28.
- **#8 and #53** both name `pop_uidls` and `pop_download_times`; #8 adds
  `get_messages_for_folder`. #53's verdict says the per-folder calls in the two tests are the
  control that proves a move took the row out of the inbox, so the tests keep that control.
  One plan, 13.2-31.
- **#16 and #35.** #35's audit fix proposed `MailController::remove_by_message_id`, which #16
  deletes. #35's verdict uses `copies_already_there` then `remove_these` instead, because a
  find-and-remove in one call would also take a copy filed a moment later. So 13.2-05 does not
  use it, and 13.2-31 deletes it afterwards. #16's verdict keeps `is_connected`, which is a test
  oracle in eight assertions, not dead code.
- **#27 and #29.** #29's verdict says its Outlook calendar quarter is wrong and the 410 retry
  with no window belongs to #27. 13.2-18 owns the Outlook calendar; 13.2-21 owns contacts on both
  providers and the Google calendar.
- **#25 and #29.** #25 makes the expired-marker fallback fire for Google contacts; #29 makes
  the full read it falls back to take off what the provider deleted. 13.2-17 comes first.
- **#10 and #15** both make something happen twice (a rule-filed row, a whole check), by
  different mechanisms. Separate plans, 13.2-08 and 13.2-10.

## What a plan must read from a sibling

- 13.2-02 adds the catch-up that brings a conversation row up to date, and the conversation
  window's commands in 13.2-03 reach it through the do-halves they share with the list;
  13.2-03.1 finishes what 13.2-03 started.
  All four regression plans edit src/presentation/wx_app.rs, and so do 13.2-07, 13.2-09,
  13.2-11 and 13.2-22 after them. `REGRESSIONS.md`, under "Where the plans sit in phase 13.2",
  names what each of those four reads from 13.2-02.
- 13.2-05 builds on 13.2-04: Undo Send's too-late path files a draft copy only when the Outbox
  row was really taken back, which needs 13.2-04's claim.
- 13.2-06 and 13.2-10 both touch the check and watch starters in wx_app.rs (spawn_mail_watch,
  spawn_mail_sync). 13.2-10's in-flight guard is written against 13.2-06's enabled filter.
- 13.2-08 changes `uids_to_fetch`; 13.2-09's download takes what is below the newest held
  message, so 13.2-09 reads 13.2-08's result before writing its tests.
- 13.2-22 changes `flush_announcements`; 13.2-28 makes `event_log` test-only, and the tests
  that read released lines change in both. 13.2-28 reads 13.2-22's tests.
- 13.2-12, 13.2-30 and 13.2-31 all edit src/vendor/paperback/html_to_text.rs; 13.2-16 and
  13.2-31 both edit src/service/update_download.rs; 13.2-19 and 13.2-32 both edit
  src/service/caldav.rs; 13.2-28 and 13.2-29 both rewrite docs/architecture.md; 13.2-25,
  13.2-26 and, on a yes to #45, 13.2-26.1 edit scripts/check.sh and scripts/check.test.sh.
  Each later plan names the
  earlier one's change in its premises and moves its guard anchors by both readings.
- 13.2-20.1 and 13.2-21 both work in src/application/calendar.rs, and 13.2-18 before them.
  Until the split of 2026-10-14 this named 13.2-20.

## Ledger rows a plan closes or narrows

| Row | Today | Plan | What happens |
|-----|-------|------|--------------|
| 464 | open, says the depth fix holds "every job running cargo test" to fetch-depth 0 | 13.2-24 | corrected and closed once the guard reads every workflow |
| 616 | open, release.yml has no WIXEN_NO_AUDIO, Pratik's under guardrail 7 | 13.2-24 | stays open; a checkpoint asks Pratik, and says the history fix alone does not make the gate pass |
| 637 | open, unrun-verify for phase 14; says the forged-From case cannot be told apart because nothing reads a provider verdict | 13.2-13 | narrowed: the verdict is read now; the real-account half stays phase 14's |
| 325 | open, whether a tree guard should read the index rather than the disk | 13.2-25 | the planner says whether #40's refusal settles it or only the commit half |

The verdicts name nearby rows that are different problems and stay as they are: 220 (#1), 765
(#2), 70 (#14), 523 and 58 (#36), 720 and 660 (#31), 772 and 190 (#34), 253 (notes, beside
#29), 329, 334 to 336 and 342 (#24). New rows are numbered by the executor at commit time,
after 830 and after anything a running plan has taken, never by a planner in advance.

## Requirement candidates

Phase 14's REAL-01 and REAL-02 are proven by sittings and are not ticked here. Each planner
reads the requirement's text before naming it, and names none with a reason where nothing fits.

| Plan | Candidates |
|------|-----------|
| 13.2-01 | LIST-06, which the plan names |
| 13.2-02 | LIST-21 and LIST-04, which the plan names |
| 13.2-03, 13.2-03.1 | LIST-05, which both name, and LIST-03, which 13.2-03.1 names |
| 13.2-04, 13.2-05 | FOUND-04 (Undo Send) |
| 13.2-06, 13.2-10 | MAIL-04 (mail keeps arriving), MAIL-05 for the once-only announcement |
| 13.2-07 | GAP-14 (Trash emptied on a schedule), FOLDER-01 (empty a folder) |
| 13.2-08, 13.2-09 | MAIL-01, MAIL-03, MAIL-05 |
| 13.2-11 | LIST-21 (a move or delete completes here first) |
| 13.2-13 | LIST-09, LIST-10 (pictures and the privacy page), GAP-04 (meeting invitations) |
| 13.2-18, 13.2-19 | PIM-06 and PIM-03 are open; name them only if the fix completes them |
| 13.2-24, 13.2-25, 13.2-26, 13.2-26.1, 13.2-32 | FOUND-23 |
| the rest | none, or the one the planner finds by reading |

## Decisions that wait on Pratik

- **13.2-02 and 13.2-03:** the four questions at the end of `REGRESSIONS.md`. 13.2-02 is
  written on the recommended answer to question 1 and ledgers question 2, the folder tree's
  unread count, for his answer; 13.2-03 is written on the recommended answers to questions 3
  and 4. Each plan's decisions block says what he can overrule.
- **13.2-24:** whether release.yml's quality gate gains WIXEN_NO_AUDIO (ledger 616). The plan
  fixes the history depth and the guard, and leaves a `checkpoint:decision` naming the audio
  question.
- **13.2-26.1:** (13.2-26 until the split of 2026-10-14) #45 changes what a documents-only
  commit runs, and CLAUDE.md records Pratik's
  settled rule that it runs formatting, clippy and the document targets. A `checkpoint:decision`
  asks before anything changes; CLAUDE.md is edited only on a yes.
- **13.2-16, possibly:** #24's verdict says pinning the leaf key or its hash refuses every
  genuine release, because Artifact Signing issues a short-lived certificate with a fresh key
  each day. If the identity rule needs a value only Pratik's signing account can give, the plan
  says so and gates it rather than inventing one.

## Where the order moved from the brief

- The four regression plans, 13.2-01 to 13.2-03.1, were not in the brief. They went in front on
  2026-10-14, the day Pratik reported the regressions.
- Empty Folder's wording (#34), last in the brief's first group, shares 13.2-07 with the Trash
  detection (#9): both are what the delete sitting, 14-11, proves, and both are the delete path.
- Mark of the Web (#22) moved from fourth to the end of the untrusted-input group, to share
  13.2-16 with the update signature (#24): both are about a file from outside that may run.
- Findings the brief did not list went to the nearest plan by file and subject: #33 with
  accounts, #36 with the download, #14 with the checks, #13 with the webcal feed as a server's
  answer that stops things, #21 with the reply pictures as what a stranger's message can make
  the program do, #43 with the tooling.

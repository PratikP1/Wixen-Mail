What this repo does: Wixen Mail is a Windows program for mail, calendar, contacts, tasks and notes. It is written in Rust and built for people who use a screen reader. It talks to mail servers (IMAP and POP for reading, SMTP for sending), to Google and Microsoft, and to calendar and contact servers, and keeps a local copy in a SQLite database. Load assumed: one person with a few accounts, an inbox of up to 100,000 messages and a few thousand contacts, with checks running in the background every few minutes, all heard through NVDA or Narrator.

## Must fix

**1. Two Outbox passes can send the same message twice, and Undo Send can say "taken back" when it was not** `src/presentation/wx_app.rs:24529-24786`, `src/data/message_cache/outbox.rs:122-133`
- **What this is:** The send loop (`flush_outbox`) reads every message that is ready and sends each one. It deletes the message's row (one saved entry in the local database) only after the send finishes. Undo Send deletes the row and treats success as proof that it stopped the message.
- **Problem:** Nothing stops a second pass while one is running, and reading a row does not claim it. With the default 10-second hold, two messages due a few seconds apart start two passes, and the second pass sends the first message again. A narrower version of the same gap lets Undo Send announce "Taken back" while the message goes out.
- **Fix:** Add a nullable `sending_since` column. Before each send, claim the row with `UPDATE outbox_queue SET sending_since = ? WHERE id = ? AND sending_since IS NULL` and skip it if nothing changed. Undo's delete also requires `sending_since IS NULL`, and start-up clears all claims. This one change closes both holes.
- **If we skip it:** People receive duplicate mail that cannot be recalled, and the sender is told a message was stopped when it went.

**2. A person's own folder named Deleted, Bin or Trash can become the account's Trash** `src/service/protocols/imap/special_use.rs:18-23`, `src/application/destinations.rs:100-109`
- **What this is:** The program learns which folder is Trash from a label the server puts on it. If there is no label, it guesses from the folder's name, at any depth.
- **Problem:** The name guess runs even when the server has already labelled another folder Trash. Folders are sorted by name, so "Clients/Deleted" or "Bin" wins over the real "Trash". Delete then moves mail there, Delete inside that folder erases mail at the server, and automatic Empty Trash erases the person's archive.
- **Fix:** In `list_folders` (imap.rs), if any folder carries a role label, take roles from labels only. Without labels, use the name guess only for top-level folders or folders directly under INBOX. Write the three red tests first.
- **If we skip it:** The first person with an archived "Deleted Items" folder who turns on emptying loses that folder from the server for good.

**3. Saving accounts can silently delete another account and always resets the order** `src/data/message_cache/accounts.rs:21-32`, `src/data/message_cache/mod.rs:2486`
- **What this is:** `save_account` writes each account with `INSERT OR REPLACE`. In SQLite that deletes any row that clashes on a unique column before it inserts.
- **Problem:** The email column is unique, so adding the same address twice deletes the first account on the next start. Its mail stays in the database, and its password and tokens stay in Windows' credential store with nothing pointing at them. Every save also blanks the chosen order, so an Alt+Shift+Up reorder is undone by the next Account Manager change.
- **Fix:** Before storing the password, refuse an address another account already uses, with a plain sentence. Then switch to `INSERT ... ON CONFLICT(id) DO UPDATE SET ...`, leaving out `created_at` and `tree_order`.
- **If we skip it:** An account can vanish while its secrets stay on the machine, where uninstall never erases them.

**4. A stranger's deeply nested HTML crashes the whole program** `src/application/hidden_text.rs:308-348`, `src/vendor/paperback/html_to_text.rs:282-319`
- **What this is:** Several functions turn a message's HTML into text by calling themselves once for every level of nesting. The text reader in long_text and the table reader work the same way.
- **Problem:** None of them has a depth limit, and the HTML parser does not limit nesting either. A message of 20,000 nested `<div>` tags overflows the thread's stack (the fixed memory a thread has for calls). That ends the process with no crash log, and it happens again each time the message is opened or downloaded.
- **Fix:** Write one small check, `nests_deeper_than(root, 512)`, that counts depth without calling itself. Run it right after parsing in the three entry points (`drop_what_the_sender_hid`, `HtmlToText::convert`, `long_text::read_markup`) and fall back to plain text when it trips. Red tests first with 50,000 nested divs, one of them wrapped in `<h1>`.
- **If we skip it:** Any sender can crash the client again and again with one message that passes every filter.

**5. Reply and Forward load the sender's tracking pixels** `src/presentation/editor_document.rs:105-113`, `src/presentation/wx_compose.rs:286-305`
- **What this is:** The reply editor is a live web page (WebView2) that holds the quoted original.
- **Problem:** The reader holds back remote pictures and 1-pixel trackers. The editor only cleans the HTML, so every remote picture loads the moment the composer opens, and the sender learns the mail was opened. `docs/privacy.md` says that cannot happen there.
- **Fix:** Add `<meta http-equiv="Content-Security-Policy" content="img-src data:">` to the editor page's head. This is a browser rule that limits where pictures may load from. Write the failing test first. Pictures the person inserts still show, and the outgoing message does not change.
- **If we skip it:** Every reply or forward tells trackers the message was read.

**6. Sending a draft leaves its copy in Drafts, here and on the server** `src/presentation/wx_app.rs:19292-19298`, `src/presentation/wx_app.rs:24329-24352`
- **What this is:** Each save, and each 2-minute autosave, files a copy of the draft in the account's Drafts folder.
- **Problem:** Send deletes only the draft's own record, never the filed copy, so the sent message stays as an unsent draft on every device. Undo Send's "too late" path even files a fresh copy of a message that has already gone out.
- **Fix:** After a successful send, remove the copy by its Message-ID. For POP, delete the local row. For IMAP, call the existing, unused `MailController::remove_by_message_id`. In Undo Send, file the copy only after the Outbox row was really taken back, and correct `docs/changelog.md:8510`.
- **If we skip it:** Every message written for more than two minutes stays in Drafts, and someone may send it again.

**7. Mail that a rule files on IMAP shows twice in the destination folder** `src/application/mail_sync.rs:1343-1380`
- **What this is:** When a rule moves a message, the server does the move, and the program moves its own row with a placeholder number.
- **Problem:** Nothing ever swaps the placeholder for the server's real UID (the server's number for a message in a folder). The next read of the folder fetches the message again as a second row, and the placeholder row stays forever.
- **Fix:** After the server reports the move, look the message up in the destination by Message-ID (`where_it_is`). With exactly one answer, call `the_server_holds_it_at`. Otherwise call `let_the_next_read_bring_it`, the same as manual moves. Write the red test first and fix the test at `messages.rs:3209`, which currently expects the duplicate.
- **If we skip it:** Every IMAP rule that files mail doubles the contents of the destination folder.

**8. When more than 500 messages arrive between checks, the rest skip the rules** `src/application/mail_sync.rs:389-405`
- **What this is:** A check fetches at most 500 new messages per folder and runs the rules only on those.
- **Problem:** The older messages in the gap come in later through the background download, which runs no rules and does not count them as new. Block-sender and move rules are skipped on the busiest days, such as after a week away.
- **Fix:** In `uids_to_fetch`, when the folder already holds mail, always take every missing message above the highest one held, and apply the 500 cap only to older backfill. Red test: the folder holds 1 to 10, the server holds 1 to 810, and the check should fetch 11 to 810.
- **If we skip it:** Rules silently miss hundreds of messages.

**9. The text download loops forever on one or two messages that always fail** `src/presentation/wx_app.rs:30061`, `src/application/mail_sync.rs:2626-2648`
- **What this is:** The background download fetches message text in chunks and loops until nothing is missing.
- **Problem:** Nothing remembers a message that failed. A chunk of one or two messages that always fail (deleted on the phone, impossible to read, or one huge message) is requested again with no wait, many times a second. If three such messages sit at the front, the run stops, and accounts listed after this one never get their download.
- **Fix:** Keep a `HashSet` of message ids already requested in this account's run and remove them from the missing list each turn, about 3 lines. Red test: a scripted mailbox where one message always fails, and the runner must stop.
- **If we skip it:** The program hammers the mail server and can hit the provider's rate limits.

**10. Announcements made close together overwrite each other** `src/presentation/accessibility.rs:435-468`, `src/presentation/accessibility/screen_reader.rs:120-134`
- **What this is:** Every spoken line is written into one hidden text box that screen readers watch, called a live region.
- **Problem:** Lines released at the same moment are written microseconds apart, and the screen reader reads only the last one. On a send failure, "Message not sent: <reason>" is replaced by "The Outbox is done: 0 sent, 1 not sent." before anyone hears it.
- **Fix:** In `flush_announcements`, join everything released into one line, most urgent first, and deliver it once. Hold flushing while the timer handles its batch of updates, then flush once. Change `test_flush_announcements_priority_order` to expect one joined line.
- **If we skip it:** The product's main channel drops a line whenever two things happen at once, and it is usually the more important first line.

**11. Contact sync quietly stops: Google after a week, and new contacts never reach Outlook** `src/application/contacts_sync.rs:854-869`, `src/application/contacts_sync.rs:2492-2543`, `src/application/contacts_sync.rs:2706-2739`
- **What this is:** Contact sync stores a marker and then asks only for changes since that marker. Contacts made here are created at the provider only at the end of a full read.
- **Problem:** Google's documented answer for an expired marker is error 400 with `EXPIRED_SYNC_TOKEN`, but the code only treats error 410 as expired, so from about day 8 every Google contacts sync fails. This has not been seen live yet. Separately, a full read for Outlook happens only on the first sync, so a contact added later never goes up.
- **Fix:** Also treat a 400 whose message contains `expired_sync_token` or "sync token is expired" as expired, and keep 410 for Outlook. Remove the `if read_the_whole_address_book {` and `if delta_link.is_none() {` guards so new contacts go up on every sync. Write one red test per provider first.
- **If we skip it:** People add contacts here and never find them on their phone, and Google changes stop arriving, with one error line as the only sign.

**12. The Outlook calendar only ever sees the dates of its first sync** `src/application/calendar.rs:1315-1355`, `src/application/calendar.rs:1438-1449`
- **What this is:** Outlook calendar sync asks Microsoft once for a date window, 180 days back and 365 ahead. After that it reuses a "delta link", a marker that reports only changes.
- **Problem:** The link is tied to that first window and kept forever, so meetings dated after the window never arrive. A meeting made here past the window's end is reported back as removed and deleted locally. The "marker expired" recovery sends no window at all, which Microsoft refuses, and sync then stops for good.
- **Fix:** Always compute the window, so the retry after an expired marker carries it. Drop the stored link when `last_full_sync` is more than about 30 days old. Write red tests for both cases first.
- **If we skip it:** An Outlook calendar slowly goes blind, and a missed meeting is the first sign.

**13. Week and Month views miss events made here on their first day** `src/data/message_cache/calendar.rs:94-101`
- **What this is:** The query that picks events for a calendar view compares stored dates as text.
- **Problem:** The editor stores "2026-10-12 10:00", or a bare "2026-10-12" for all-day events, while the window starts at "2026-10-12T00:00:00Z". A space sorts before "T", so an event on the first day counts as too early and is not drawn. It disappears the moment it is saved.
- **Fix:** For the lower bound, bind only the date part: `from.get(..10).unwrap_or(from)`. Red test first, with a timed event and an all-day event on the first day.
- **If we skip it:** Events on Monday and on the 1st vanish from exactly the views people check.

**14. A subscribed calendar feed breaks a repeating event that has one changed day** `src/application/caldav_sync.rs:1150-1201`
- **What this is:** A subscribed `.ics` feed, such as a school or team calendar, is downloaded and saved on each refresh.
- **Problem:** A feed sends a repeating event and its moved or cancelled days as separate entries with the same id. The refresh saves them all onto one row, so whichever comes last wins. Either every other week of the series disappears, or the changed day shows at its old time.
- **Fix:** Split the feed's events on `recurrence_id.is_none()`. Save the whole events as now, and pass each changed day to the existing `one_caldav_day_kept_out_of_its_series`, as the CalDAV sync already does. That is about 10 lines, with red tests for both orders and for a cancelled day.
- **If we skip it:** Team and school calendars silently lose whole weekly series.

**15. The CalDAV reader never decodes XML escapes** `src/service/caldav.rs:1283-1293`, `src/service/carddav.rs:350-402`
- **What this is:** CalDAV (the calendar server protocol) answers in XML, which the code reads by hand. XML must write `&` as `&amp;`.
- **Problem:** The address-book reader decodes these escapes and the calendar reader does not. "Dinner & drinks" is stored and spoken as "Dinner &amp; drinks", and an edit writes that text back to the server for every other client to see. On Nextcloud-style servers, events are probably misread as well, but this has not been seen live.
- **Fix:** Move carddav's `xml_unescaped` into caldav.rs, teach it numeric forms like `&#13;`, and apply it inside `extract_xml_value`. Then drop carddav's wrapper and the third copy in free_busy.rs. Red fixture first, containing `&amp;`, `&quot;` and `&#13;`.
- **If we skip it:** Calendar titles with "&" are damaged here and on the server, and get worse with each round trip.

**16. The release workflow and the other-platform workflows cannot pass** `.github/workflows/release.yml:52-53`, `.github/workflows/other-platforms.yml:86-87`, `tests/house_style.rs:4665`
- **What this is:** One test reads the git history and fails if commit 18a02454 is missing. A guard test exists to make every test job fetch the whole history.
- **Problem:** These three jobs fetch only one commit, so that test fails on every run. The guard reads only `ci.yml`, so it sees none of them. The release gate also lacks `WIXEN_NO_AUDIO` (ledger 616).
- **Fix:** Add `with: fetch-depth: 0` to the three checkouts, and make the guard read every workflow file. Leave the audio setting to Pratik, as ledger 616 says, and note that it alone will not make the gate pass. Correct ledger 464.
- **If we skip it:** The first release attempt fails after a full build, and the Linux and macOS runs report a failure that has nothing to do with the platform.

## Should fix

**17. A change made while a sync is pushing is lost** `src/data/message_cache/tasks.rs:557-575`, `src/data/message_cache/notes.rs:797-802`, `src/application/calendar.rs:637-653`, `src/application/caldav_sync.rs:517-548`
- **What this is:** After a sync sends a waiting task, note or event to the provider, it clears the item's "pending" flag (the flag that means "still to send") or saves the copy it sent.
- **Problem:** Nothing checks that the item is unchanged since the push read it. A tick, a note edit or an event edit made during the push is wiped or never sent. A task ticked while its create is in flight is lost everywhere.
- **Fix:** Pass along the `updated_at` the push read, and clear pending only when it still matches: `pending = CASE WHEN updated_at = ? THEN 0 ELSE pending END`. Always write the provider's id and stamp. Replace the calendar's whole-row save with a targeted UPDATE, make the task move set `updated_at`, and write one red test per kind.
- **If we skip it:** Edits vanish without a word, and the phone and the computer disagree until one side changes again.

**18. A Google or Microsoft sign-in token goes to whatever server the account names** `src/application/mail_auth.rs:94-123`, `src/application/who_runs_the_mail.rs:90-95`
- **What this is:** With browser sign-in, the program gets an OAuth token (a pass with full access to the mailbox) and hands it to the account's mail servers.
- **Problem:** It never checks that those servers belong to the provider, and it never looks at the outgoing server at all. A typo like `imap.gmial.com`, or a relay that is not Google's, receives the token if it asks for one.
- **Fix:** In `mail_auth::for_account`, before fetching the token, refuse with a sentence unless both the incoming and outgoing servers belong to the provider. Write the two red rows first, and move T-13-PK-02 from "accept" to "mitigate".
- **If we skip it:** A typo or a hostile server gets full control of someone's mailbox for an hour.

**19. Two checks of the same account can run at once** `src/presentation/wx_app.rs:30378-30398`, `src/application/mail_sync.rs:1770`
- **What this is:** F9, start-up, the schedule and the inbox watch each start a mail check.
- **Problem:** Only the schedule checks whether a check is already running. Press F9 during the start-up check and both checks fetch the same new message, run the rules twice, and say "1 new message" twice with the sound.
- **Fix:** Keep one lock per account in `application::mail_session` and hold it for the whole of that account's pass. A second check then waits, finds nothing new, and stays quiet. Red test: two locked passes, where the second reports 0 fetched.
- **If we skip it:** Doubled announcements break guardrail 5, and rules with side effects run twice.

**20. A pretend accessibility tree that no screen reader sees, and a log that keeps every spoken line** `src/presentation/accessibility.rs:67-192`, `src/presentation/accessibility/screen_reader.rs:491`, `src/presentation/accessibility/screen_reader.rs:657-660`
- **What this is:** `KeyboardHandler`, `FocusManager` and `AutomationStore` keep a made-up model of the window in memory. The speech bridge also copies every line it speaks into `event_log`.
- **Problem:** The model never reaches UI Automation or MSAA (the two channels Windows screen readers read). Only tests use it, and six guard records protect it. The log is never cleared, so whole message bodies read aloud stay in memory until exit. The start-up line "Focus moved to Folders" comes from a stored string, not from Windows.
- **Fix:** Delete keyboard.rs, focus.rs and automation.rs and the methods that feed them, keeping "Wixen Mail is ready". Make the log a test-only list of released lines. Retire the six shadow-tree guard records, keeping the `feedback_settings` one at guards.toml:11318, then add a changelog line and correct docs/architecture.md.
- **If we skip it:** Memory grows with use, private text sits in RAM, and about 450 lines are kept for nothing.

I left out 29 smaller findings: 21 should fix and 8 nice to have. The ones most worth reading next:
- A waiting delete replayed after the server renumbered a folder can erase the wrong message.
- Settings are rewritten in place, so a crash during a save resets them all.
- A switched-off account keeps being watched and checked.
- Empty Folder says "emptied" when the server only marked the mail.
- Saved attachments carry no Mark of the Web.
- A stale offline read or star change undoes a newer one.
- The commit hook tests the working tree, not the staged snapshot.
- Three dependencies and about 1,100 lines of skeleton, translation and uncalled code have no use.

Verdict: The design and the test tooling are careful, but the send, sync and rule paths have races and gaps that duplicate, lose or wrongly delete mail. Fix the 16 must items before the real-account sittings go further.

Lean: -1,590 lines, -2 dependencies possible.

Not checked: No builds, tests, cargo commands or workflows were run, and the program was not started under NVDA or Narrator. These parts were not read: most of `wx_app.rs`; the vCard parser and contact merge in `data/message_cache/contacts.rs`; the S/MIME and PGP internals; the .pst, .msg, PDF and picture readers; safebrowsing; CardDAV sync; invitations, repeat expansion and time zones; tasks_api and OneNote; the compose, settings, account manager, editor and reader dialogs; the search-handler crate; most test modules and `docs/`. The cost of opening the store on a 100,000-message database is unmeasured. Four things rest on documentation, not on a live answer: Google's 400 for an expired marker, Microsoft's delta window, the escaping on Nextcloud-style servers, and Inno Setup's uninstall flags. Auditors ran sed or awk read-only several times against the brief. Those runs are logged as observations 0992 to 0994, and no repository file changed.
# Phase 13, group 3: the provider features (GAP-06, GAP-07, GAP-08, GAP-10) - Research

**Researched:** 2026-09-24, against `main` at `630e2a67` (phase 12 closed), version `1.0.0-alpha.1`
**Domain:** IMAP junk reporting and filing, LDAP and Microsoft Graph people lookup, free/busy across Google, Microsoft and CalDAV, several sending addresses per account
**Confidence:** HIGH for what the tree holds and for the provider documentation read today; MEDIUM for how providers behave on a real account, which nothing here can settle

Two markers run through this file. **(stated)** is something read today: a file and line opened, a command run, a page fetched. **(derived)** is an inference from what was read. A claim with neither is a recommendation.

## User Constraints

There is no `CONTEXT.md` for phase 13. The constraints are the brief's and Pratik's:

- **Order inside the phase, confirmed by Pratik on 2026-09-24 (stated, from the brief):** keyboard basics (print, undo and redo), then reading mail (invitations, encrypted mail, PGP key manager), then provider features (junk, directory, free/busy, identities), then automation (saved searches, Quick Steps, rules on a folder), then the rest of import and export. This group is the third block.
- **Every new package waits for Pratik's confirmation before it is installed.** This group needs no new package (see the audit section), so no plan here waits on that.
- **A new OAuth scope changes what the consent screen asks, and Pratik must know.** One plan here adds one (GAP-07, Microsoft `People.Read`); it is question 4.
- **Network code is tested against parsing and error mapping, never live servers** (CLAUDE.md, Guardrails, Test-driven development).
- **Deferred, out of scope for this group:** shared mailboxes, send on behalf, and delegation (issue #59's steps 2 and 3); they are named as later work, not built.

## Project Constraints (from CLAUDE.md)

Read in full today. The directives the planner must check every plan against:

- Red, green, refactor on every change; `workflow.tdd_mode` is `true` (stated, `.planning/config.json`). A red commit names every failing test in `Fails-until-green:` lines and is made on a branch.
- `cargo test` takes one `--lib`; several module paths are several runs joined with `&&`.
- A `<verify><automated>` command must be able to fail: nothing ending in `| wc -l`, `|| true`, `| cat`.
- Errors through `common::Error`; no `unwrap` or `expect` outside tests and `build.rs`; typed values over strings at the SQL boundary.
- Done means a non-test path reaches it. No stubs presented as complete.
- Accessibility is verified on both channels, MSAA for NVDA and UI Automation for Narrator, at the handle that takes focus. Mnemonic letters are one set per dialog page and per menu, checked by grep before a plan hands one out.
- Every setting a user has is reachable from a screen and sorted where somebody would look.
- Schema changes are additive: `CREATE TABLE IF NOT EXISTS`, `ensure_column_exists`, never drop or rename.
- Secrets go to the credential store; each service name has one owner, and the uninstall sweep names the same entries (`src/application/forget.rs`).
- Untrusted input stays untrusted; the privacy page says who is talked to.
- "If you expect bug reports from something, that belongs in the product": anything that has not met a real account is marked experimental where the person sees it.
- User-visible changes get a `docs/changelog.md` entry in the same commit; keys go in `docs/KEYBOARD_SHORTCUTS.md` in the same commit as the binding.
- Premises a plan checks before execution: count guard records by the TOML parser (both readings: `tests_last_seen`/`file`, and anchors inside edited files); add `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md` to `files_modified` before deciding waves; one plan per wave where two plans share those files.
- The phase's closing plan runs `scripts/check.sh all` once by hand before its merge (12-03.2's rule). Not this group's job unless one of its plans closes the phase.

<phase_requirements>
## Phase Requirements

| ID | Description (stated, `.planning/REQUIREMENTS.md:5815-5853`) | Research support |
|----|-------------|------------------|
| GAP-06 | A sender can be reported as junk to a provider that takes reports, and a block moves the sender's existing mail. [D] Report Junk on the Action menu for a provider with an endpoint, said when there is none; a block that moves what is already here, said with the count; through the gate. | Plans C1, C2. IMAP move into the junk folder plus the `$Junk` keyword where the folder keeps keywords; Gmail documents a move into Spam as a report; the move and the flag write are already gated. |
| GAP-07 | Directory lookup answers from Graph for a Microsoft account and from an LDAP directory that needs a sign-in. [D] Graph people search while typing; an LDAP directory with a bind, its sign-in kept in the credential store; the privacy page's row. | Plans C3, C4, C5. `ldap3` already binds; the tree passes no password. Graph `GET /me/people?$search` with `People.Read`. |
| GAP-08 | Free/busy asks a Google source and every source an account has, and shows guests' times in their zones. [D] The Google free/busy endpoint; every calendar source asked; a guest's zone shown beside the time. | Plans C6, C7, C8, and C9 if Pratik wants ledger 417 here. Google `freeBusy` is covered by the calendar scope already granted; Microsoft `getSchedule` returns each guest's working-hours zone; Windows ships the zone-name mapping. |
| GAP-10 | Several identities per account, the first step to shared mailboxes and delegation. [D] An identity (a From name and address) per account beyond the first, offered in compose's From list; shared mailboxes and delegation as their own later work, said. | Plans C10, C11, C12. A new additive `identities` table; the outbox and drafts carry the chosen address. |
</phase_requirements>

## Summary

All four requirements build on seams the tree already has, and none needs a new package. Blocking is a filter rule filed into the junk folder (`src/application/blocking.rs:137`), and the set move that Move to uses (`spawn_folder_move`, `src/presentation/wx_app.rs:21712`) already takes many messages to one destination through the write gate. Reporting junk is therefore a move into the junk folder, with the IANA keyword `$Junk` set first where the folder says it keeps keywords. Google documents that moving a message into Spam is a spam report (stated, Gmail Help 1366858). Microsoft's only reporting API, `reportMessage`, exists in the Graph beta alone and needs `Mail.ReadWrite`, a permission this program does not ask for; the older `markAsJunk` is deprecated and stopped returning data on 2025-12-30 (stated, Microsoft Learn). So for a Microsoft account the honest sentence is that the message was moved to Junk, not that Microsoft was told.

Directory lookup already asks an LDAP directory and already knows how to bind (`src/service/directory.rs:611-613`); what is missing is a place to type the sign-in and a password in the credential store (`src/presentation/finding_people.rs:154-163` passes `None`). Reading `ldap3`'s open issues found one six days old (#156) that matters more than the sign-in itself: the entry parser panics on a search continuation reference, which Active Directory returns whenever the search base is the domain root, and the tree's panic catch (`directory.rs:643-656`) turns that into a failed search that throws away every real entry. The fix is one filter on `ResultEntry::is_ref()`. Graph people search is `GET /me/people?$search=...` with `People.Read`, which works for personal and work accounts and needs only user consent (stated, Microsoft Learn); it is a new scope, and a scope on the consent list that is missing from the Graph token list is the exact defect ledger 282 already records for `Tasks.ReadWrite`.

Free/busy asks one place per account today (`src/application/asking_when_free.rs:172-188`, `:238-243`), and the service already accepts several (`src/service/free_busy.rs:698-747`). Google's `freeBusy` accepts the calendar scope the program already requests (stated). Microsoft's `getSchedule` returns each guest's working hours with a Windows zone name (stated), and Windows maps those names to IANA ones through ICU: probed today in a throwaway project with the `windows` crate at the pinned version and the feature this project already enables, "Pacific Standard Time" came back "America/Los_Angeles" (stated). Several identities need a new table, a From list whose entries are (account, address) pairs rather than accounts, and an outbox row that remembers which address was chosen, because `SendEmailRequest::from_queued` takes the account's own address today (`src/application/mail_controller.rs:170`).

**Primary recommendation:** twelve plans, one per wave, in the order C1 to C12 below; every one of them reuses an existing seam, adds tests to new or lightly guarded files rather than to `wx_app.rs` (119 guard records) or `imap.rs` (47), and marks its feature experimental where it is seen.

## Architectural Responsibility Map

This is a desktop program; the tiers are its layers.

| Capability | Primary tier | Secondary tier | Rationale |
|------------|-------------|----------------|-----------|
| What a junk report does per account (keyword kept, junk folder, POP, changes off) | `application` (new `reporting_junk`) | `service::protocols::imap` | Decisions pure and testable; the wire stays thin |
| Setting `$Junk`, moving into Junk | `service::protocols::imap` via `MailController` | `presentation::wx_app` worker | Both already gated by `may_i` |
| Which messages here a block catches, and the count | `application` (pure, over cached rows) | `data::message_cache` read | Shared with GAP-12's run-over-a-folder |
| LDAP bind, refusal of a password over plain `ldap://`, skipping references | `service::directory` | `service::secret_store` | Network at arm's length behind `AsksADirectory` |
| Graph people search and its parse | `service::microsoft_graph` | `application::looking_people_up` | Parse pure; fold into the one list |
| Where free/busy is asked, and the merge per person | `application::asking_when_free` | `service::free_busy` | Already split this way |
| Windows zone name to IANA | `common` or `service` behind `cfg(windows)` | none | One ICU call, a fallback of "unknown" elsewhere |
| Identities stored | `data::message_cache` (new table) | `application::identities` | Additive schema |
| Choosing the From address, sending as it | `presentation::wx_compose` | `application::mail_controller` | The outbox row carries the choice |

## What each requirement asks, and what the tree has

### GAP-06: junk reported to the provider, and a block that moves existing mail (#54)

**1. What the issue asks (stated, `gh issue view 54`, no comments on it).** "Report as junk where the provider accepts it (the `$Junk` keyword on IMAP servers that advertise it, Gmail's SPAM label, Graph's junk folder), with the announcement saying which happened and which was not possible; a block that also moves existing mail from that sender, saying how many; a key." Its three remaining points: report-as-junk is absent ("the block announcement itself says the provider is not told", `blocking.rs:566`); a block acts only on mail arriving after it; Block is two menu levels deep with no key.

**2. What the tree has (stated unless marked).**

| Seam | Where | What it gives |
|---|---|---|
| A block is a filter rule | `src/application/blocking.rs:137-159` (`a_rule_that_blocks`: `field: "from"`, `match_type: "regex"`, `action_type: "move_to_folder"`) | The pattern a block uses, reusable to find existing mail |
| Where blocked mail goes | `blocking.rs:285-296` (`where_blocked_mail_goes` over `FolderType::Spam`), `:338` (`what_the_junk_folder_needs`) | The junk folder per account, and the two refusals `NO_JUNK_FOLDER_FOUND` (`:245`) and `NO_FOLDERS_KNOWN_YET` (`:250`) |
| The sentence to retire | `blocking.rs:568-569`, `WHAT_IT_DOES_NOT_DO`: "This does not tell your mail provider anything, so the mail is still accepted and still arrives here. Messages that already arrived stay where they are." | Its second half becomes false with C2 |
| The Block handler | `src/presentation/wx_app.rs:32141-32297` (`block_the_sender`), reached from `:4583-4592` | Cursor message only; sentences said before and after; no question asked |
| The Block submenu | `wx_app.rs:7033-7044` ("&This Sender", "Everyone at This &Domain"), appended at `:7386-7390` as "Blo&ck" | No accelerator on either |
| The set move | `wx_app.rs:21712` (`spawn_folder_move`), `:21058` (`move_or_copy_message` with `chosen_messages`) | Many messages to one destination, one sentence at the end, per account |
| IMAP flag write, gated | `src/service/protocols/imap.rs:1384-1396` (`set_flag`, `UID STORE ... +FLAGS`, `self.may_i("change a message")`); `MailController::set_flag` at `src/application/mail_controller.rs:622` | A keyword is a flag string; the write is already behind the gate |
| IMAP move, gated | `imap.rs:1436` (`move_message`), `MailController::move_message` at `mail_controller.rs:653` | `Moved::Moved`, `CopiedAndFlagged`, `CopiedAndNotFlagged` said apart |
| Opening a folder | `imap.rs:1084-1119` (`select_folder`) returns `MailboxStatus` (`:385-397`) with `uid_validity` and `highest_modseq` only | **No PERMANENTFLAGS read.** `imap-proto` 0.16.7 has `ResponseCode::PermanentFlags(Vec<Cow<str>>)` (stated, `imap-proto-0.16.7/src/types.rs:131`) |
| The five flag names | `src/service/protocols/imap/flag.rs` (0 guard records) | Where `$Junk` and `$NotJunk` constants belong |
| Rule matching, pure | `src/application/filters.rs:349` (`FilterEngine::matches(rule, message)`), `:450` (`from_persisted_rule`) | Counting existing mail a block catches without a server |
| Filing at sync | `src/application/mail_sync.rs:1142-1222` (`carry_out_the_moves`) | How a rule's move reaches the server on arrival |

Absence claims, each with the search that came back empty (stated): `grep -rn '\$Junk\|\$NotJunk' src` found nothing outside comments (the only `Junk` hits are the special-use attribute `\\Junk` at `imap.rs:2141` and `special_use.rs`); `grep -n 'PERMANENTFLAGS\|permanent_flags' src/service/protocols/imap.rs src/service/protocols/imap/*.rs` found nothing; `grep -n 'Report\|report' src/presentation/wx_app.rs | grep -i junk` found nothing; `grep -n 'Ctrl+J\|Ctrl+Shift+J\|Ctrl+Alt+J\|Shift+J\|Ctrl+Shift+B' docs/KEYBOARD_SHORTCUTS.md` found nothing.

**What the providers say (stated, pages read 2026-09-24).**

| Provider | How a report is made | Source |
|---|---|---|
| Any IMAP server | `$Junk` and `$NotJunk` are keywords with defined meaning; "$Junk and $NotJunk are mutually exclusive. If more than one of these is set for a message, the client MUST treat it as if none are set." A client may create a keyword only where `PERMANENTFLAGS` includes `\*` (or names the keyword). | RFC 9051, section 2.3.2 and the SELECT description |
| Gmail | "When you report spam or move an email into Spam, Google receives a copy of the email and may analyze it to help protect users from spam and abuse." So an IMAP move into `[Gmail]/Spam` is a report, and no Gmail API call is needed. | support.google.com/mail/answer/1366858 |
| Microsoft Graph | `markAsJunk`: "deprecated and will stop returning data on December 30, 2025". `reportMessage` (`junk`, `notJunk`, `phish`): documented under the beta only, "Use of these APIs in production applications is not supported"; least privilege `Mail.ReadWrite` for work and personal accounts. | learn.microsoft.com, message-markasjunk and message-reportmessage (the v1.0 URL for `reportMessage` served the beta page) |

(derived) This program reads Microsoft mail over IMAP (`oauth.rs:94`, `IMAP.AccessAsUser.All`) and holds no `Mail.*` Graph permission, so a Graph report would be a new, broad permission for a beta endpoint. The plan below reports by the IMAP move for every provider and says, per provider, what that means.

**3. Proposed plans.**

**C1. Report as Junk.** Closes GAP-06's first [D] line.
- What: Action, Report as &Junk (`Ctrl+Shift+J`, question 3) over every selected message. For each message: open its folder, read `PERMANENTFLAGS`; where it keeps keywords, `-FLAGS ($NotJunk)` then `+FLAGS ($Junk)`; then move it into the account's junk folder through the existing set move. One sentence at the end, per account kind: "3 messages reported as junk and moved to Junk." / on Gmail "...moved to Spam, which tells Google." / on a server that keeps no keywords "...moved to Junk; this server does not keep a junk mark, so only the folder says so." / on Microsoft "...moved to Junk Email. Microsoft offers no supported way for a mail program to report junk, so Microsoft has not been told." / POP: "This account collects its mail with POP, which has no junk folder at the server, so nothing was reported." / changes off: the existing `but_mail_changes_are_off` wording (`blocking.rs:672`). A message already in the junk folder is passed over and counted in the sentence.
- Files: `src/application/reporting_junk.rs` (new), `src/application/mod.rs`, `src/service/protocols/imap/flag.rs` (the two keyword constants and a pure `keeps_keywords(permanent: &[String], keyword: &str) -> bool`), `src/service/protocols/imap.rs` (`MailboxStatus` gains the permanent flags read in `select_folder`; no test added in this file, see pitfall 1), `src/application/mail_controller.rs` (only if a combined call is wanted; `set_flag` and `move_message` exist), `src/presentation/wx_app.rs` (id, menu item, handler), `docs/KEYBOARD_SHORTCUTS.md` (Action Menu table at `:661-692`), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: none in this group. Shares `wx_app.rs` and the Action menu with every group 1 and group 4 plan that adds a menu item; one plan per wave.
- Spoken or shown: yes, a new menu item and new sentences. Push with a pull request.
- Size: M, three tasks: (1) the pure decision module and the keyword read, red first; (2) the wiring over the set move with the keyword step; (3) pages, changelog, ledger.
- Failing test that starts it: `application::reporting_junk::tests::test_a_server_that_keeps_keywords_is_told_junk_and_the_message_moves` beside `test_gmail_is_reported_by_the_move_itself` and `test_a_pop_account_reports_nothing_and_says_why`; and `service::protocols::imap::flag::tests::test_a_folder_keeps_a_new_keyword_only_where_it_says_star`.

**C2. A block moves the mail already here, and Block gets a key.** Closes GAP-06's second [D] line and issue point 3.
- What: after the block rule is written (`wx_app.rs:32270-32291`), count the account's cached messages the new rule matches in every folder except Junk, Trash, Sent and Drafts (question 2), ask once "Also move the 14 messages already here from them to Junk?" with Yes and No, and on Yes move them through the same set move, the sentence saying how many went. `WHAT_IT_DOES_NOT_DO`'s second sentence is rewritten from what really happened. Accelerators on the Block submenu items (question 3).
- The pure part: `which_messages_here_a_rule_catches(rule, messages) -> Vec<RowId>` over `FilterEngine::matches`, written so GAP-12 ("a rule run over a folder on demand, saying first how many it would touch", group 4) reuses it rather than writing a second one. Put it in a new module rather than `filters.rs`, so the group 4 plan can extend it without meeting `filters.rs`'s records (2 today).
- Files: new `src/application/what_a_rule_catches_here.rs`, `src/application/mod.rs`, `src/application/blocking.rs` (sentences; 67 tests and 2 records there), a message reader in `src/data/message_cache/` (the planner greps for an existing per-account sender query first; `messages.rs` carries 10 records per CLAUDE.md), `src/presentation/wx_app.rs`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C1 (the junk destination helper and the Action menu edit; the same files).
- Spoken or shown: yes (a new question and sentence). Pull request.
- Size: M, three tasks.
- Failing test: `application::what_a_rule_catches_here::tests::test_a_block_catches_the_senders_mail_in_the_inbox_and_not_in_junk_or_sent`, and in `blocking.rs` a test that the after-sentence names the count.

**5. What cannot be verified here.** Whether Dovecot, Fastmail or any server trains its filter on `$Junk`; whether a Gmail move into Spam is really counted as a report (Google's page says so, and no account here has shown it); whether Microsoft does anything with an IMAP move into Junk Email (not documented; [ASSUMED] nothing); the whole server half of both plans. Phase 14 and one `unrun-verify` ledger entry per plan. Both features carry "experimental" where the menu item's help text and the guide describe them, on the pattern of `application::allowed`.

**Cannot be finished, gated honestly:** a Microsoft report through Graph. It needs a beta endpoint and `Mail.ReadWrite`; the sentence says Microsoft was not told, rather than implying it was.

**6. Accessibility.**
- Menu letters (stated, from the labels at `wx_app.rs:7278-7426` and the comment at `:7302-7306`): the Action menu claims A B C D E F G H I K L M N O P R S T U V W X Y; free are **J, Q, Z**. "Report as &Junk" takes J, the natural letter. The Block submenu holds T and D; any item added there must avoid them. `tests/wired.rs::test_no_two_items_on_one_menu_claim_the_same_letter` (`tests/wired.rs:1977`) holds this.
- Keys (stated, all bound menu accelerators listed by `grep -rhoE '\\t(Ctrl|Alt|Shift)...' src/presentation/*.rs`): neither `Ctrl+Shift+J` nor `Ctrl+Shift+B` is bound in the main window, and neither is written in `docs/KEYBOARD_SHORTCUTS.md`. Avoid `Ctrl+Alt+` letters, which the page itself warns are AltGr on many layouts (`KEYBOARD_SHORTCUTS.md:943`). Check again against the group 1 and group 4 research, because Quick Steps (GAP-11) will want keys too.
- The question in C2 is a native message dialog; its text carries the count and the sender, and Yes is not the default if Pratik says moving is the riskier answer.
- Announcements: one sentence per command at the end of a set, never one per message (the #30 rule `spawn_folder_move` already follows). Distinct from the Move sentence by its verb ("reported as junk").

### GAP-07: directory lookup through Graph and LDAP (#55)

**1. What the issue asks (stated, `gh issue view 55`, no comments).** "Graph people search on a Microsoft account, from the same address line, results folded into the same list; an optional sign-in per LDAP directory in Settings, stored through `secret_store`." Point 1: "no call in `microsoft_graph.rs`, no `People.Read` or `User.ReadBasic.All` scope in `oauth.rs`". Point 2: "Settings offers no sign-in name, no password is stored for a directory ... and `finding_people.rs` passes `None`, so only anonymous-bind directories work".

**2. What the tree has (stated).**

| Seam | Where | What it gives |
|---|---|---|
| The directory setting | `src/service/directory.rs:33-45` (`Directory { url, search_under, sign_in_as: Option<String> }`), stored in `AppConfig.directories` (`src/data/config.rs:365`), read by `directory_for` (`config.rs:924`) | `sign_in_as` exists and is always written `None` (`src/presentation/wx_account_manager.rs:1357-1366`) |
| The seam around the network | `directory.rs:310-321` (`AsksADirectory`), `:325` (`look_up`), `:348` (`look_up_through`) | Every failure testable; 42 tests in the file, 0 guard records |
| The bind | `directory.rs:611-613` (`simple_bind` when a name and a password are both present) | Works today if a password is passed |
| No-password refusal | `directory.rs:434-457` (`the_password_to_sign_in_with`) | A name without a password is refused with a sentence, never sent as an anonymous bind |
| Plain `ldap://` allowed | `directory.rs:405-429` (`where_this_directory_is` accepts `ldap` and `ldaps`) | **Nothing refuses a password over plain `ldap://`**; `grep -n 'starttls' src/service/directory.rs` found nothing |
| The panic catch | `directory.rs:643-656` (`catch_unwind` around `SearchEntry::construct`) | Turns a panic into an error for the whole search |
| The caller | `src/presentation/finding_people.rs:137-187` (`the_organisation`, password `None` at `:160-165`) | Where the password is fetched |
| The list | `src/application/looking_people_up.rs:25-30` (`Whose { YourContacts, TheDirectory }`), `:146-158` (`everybody_found`, de-duplicated by address), `:211` (`THE_LIST_LABEL = "P&eople found:"`) | Graph results need a third `Whose` or reuse `TheDirectory` (question for the planner, recommended: a third, so the row says where it came from) |
| Credential store | `src/service/secret_store.rs:171-189` (`write`, `read`, `remove`); service names such as `credentials::KEYRING_SERVICE = "wixen-mail-account"` (`src/service/credentials.rs:16`) and `caldav::keyring_service` (`src/service/caldav.rs:109`) | The pattern for `wixen-mail-directory-{account_id}` |
| Uninstall sweep | `src/application/forget.rs:38` (`entries_for`); the reading `owners_of_credential_entries` at `:744` finds every module holding `fn keyring_service` and requires `entries_for` to use it (`:857-930`) | A new `fn keyring_service` in `directory.rs` is red on arrival until `forget.rs` lists it: the red half for free |
| Graph client | `src/service/microsoft_graph.rs:401` (`GRAPH_BASE`), `:826-890` (`MsGraphClient`, `pointed_at` for tests); 57 tests, 3 records | No people call (`grep -n 'people' src/service/microsoft_graph.rs` found nothing) |
| Graph scopes | `src/service/oauth.rs:92-110` (the consent list), `:853-857` (`THE_SCOPES_A_GRAPH_TOKEN_CARRIES`: Contacts, Calendars, Notes), `:942-953` (`a_graph_token_for`) | Two lists that must both gain `People.Read`; ledger 282 records `Tasks.ReadWrite` on the first and missing from the second |
| The privacy page | `docs/privacy.md:181` ("Your organisation's directory ... The part of a name you have typed"), `:266-285` | Gains a Microsoft people-search row and the sign-in sentence |

**Graph people search (stated, Microsoft Learn, user-list-people v1.0 and people-insights-overview, read 2026-09-24).** `GET /me/people?$search="..."`, fuzzy, "Search for people by name or alias". Permissions: `People.Read` for work or school and for personal Microsoft accounts; "People.Read requires end user consent". Directory results come only with the header `X-PeopleQuery-QuerySources: Mailbox,Directory`; by default "Microsoft Graph serves mailbox-only results". The People API "is in maintenance mode"; Microsoft recommends `POST /search/query` with `entityTypes: ["person"]`, and its own worked example of that is on `/beta` (stated). (derived) Use `/me/people` on v1.0 now and record the maintenance-mode note as a ledger entry; the search API is the later move.

**`ldap3` issue #156 (stated, github.com/inejge/ldap3/issues/156, opened 2026-09-18, and the source).** "`SearchEntry::construct` crashes when LDAP server returns `searchResRef`." A search under the domain root of an Active Directory returns continuation references beside the entries. `construct` calls `.expect("entry")` on a match for tag 4 (stated, `ldap3-0.12.1/src/search.rs:152-156`); a reference is tag 19 and `ResultEntry::is_ref()` answers it (`search.rs:63-66`). The reporter's own fix in the comments: "I now call `.is_ref()` and skip the entry if the result is true." (derived) In this tree the catch at `directory.rs:643-656` means no crash, and every search of that shape fails whole, throwing away the real entries. Because directories that need a sign-in are mostly Active Directory, C3 fixes this first.

**3. Proposed plans.**

**C3. The directory sign-in, held in the credential store, and the reference fix.** Closes GAP-07's LDAP half at the service end.
- What: (a) skip `is_ref()` entries before `construct`, red first on a fixture holding an entry and a reference (the `AsksADirectory` seam returns `SearchEntry` already constructed, so the filter lives in `TheDirectoryItself::ask`, and the test is a small pure function `the_entries_among(results)` split out of it); (b) `directory::keyring_service(account_id)` and read, write, remove through `secret_store`; `forget.rs::entries_for` names it (the owners reading goes red on arrival); (c) a password is refused over plain `ldap://` with a sentence (question 6); (d) `finding_people::the_organisation` reads the password and passes it.
- Files: `src/service/directory.rs`, `src/application/forget.rs`, `src/presentation/finding_people.rs`, `docs/privacy.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: none.
- Spoken or shown: yes (new refusal sentences reach the "People found" trouble line). Pull request.
- Size: M, three tasks.
- Failing tests: `service::directory::tests::test_a_reference_beside_an_entry_leaves_the_entry_found`, `service::directory::tests::test_a_password_is_never_sent_over_an_unencrypted_address`, `application::forget::tests::test_this_reading_finds_every_owner_that_exists_today` (red on arrival when `keyring_service` lands in `directory.rs`).

**C4. The directory sign-in on a screen.** Closes the rest of GAP-07's LDAP half ("its sign-in kept in the credential store" reachable by a person).
- What: a sign-in name and a password for the directory, where question 5 puts them, the password written through C3's functions and never into settings, the box empty when a password is stored with a sentence saying one is saved (the account password's pattern, `describe_password_box`, `wx_account_manager.rs`).
- Files: `src/presentation/wx_account_manager.rs` (9 records naming it in `tests_last_seen`, 8 by `file`), `tests/account_edit_protocol_fields.rs` (the page letters reading, `:106-140`, extended to the new controls), `docs/USER_GUIDE.md`, `docs/KEYBOARD_SHORTCUTS.md`, `docs/privacy.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C3.
- Spoken or shown: yes. Pull request.
- Size: M, two or three tasks.
- Failing test: a case in `tests/account_edit_protocol_fields.rs` that the sign-in fields show, each letter is its own on its page, and a saved password round-trips through the store double.

**C5. Microsoft people search in the same list.** Closes GAP-07's Graph half.
- What: `People.Read` on both scope lists (consent and token); `MsGraphClient::people_matching(token, typed, at_most)` sending `$search` quoted, `$top`, `$select=displayName,scoredEmailAddresses,personType` and `X-PeopleQuery-QuerySources: Mailbox,Directory`; a pure parse into `looking_people_up::Somebody` with a third `Whose` so the row says "from Microsoft"; `finding_people::who_matches` asks it for an account whose provider is `outlook` with OAuth; a Graph failure goes into the same trouble line the directory uses; nothing asked for any other account.
- Files: `src/service/oauth.rs` (51 tests, 3 records), `src/service/microsoft_graph.rs`, `src/application/looking_people_up.rs` (1 record in `tests_last_seen`), `src/presentation/finding_people.rs`, `docs/privacy.md` (a row), `docs/PROVIDER_SETUP.md` (sign in again once), `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C3 (the same `finding_people.rs`), and question 4.
- Spoken or shown: yes. Pull request.
- Size: M, three tasks.
- Failing tests: `service::oauth::tests::test_a_new_microsoft_sign_in_asks_for_the_people_permission` beside the existing notes-permission test (`oauth.rs:1298`), and one asserting the constant carries it; `service::microsoft_graph::tests::test_a_people_answer_becomes_people_with_an_address`; `application::looking_people_up::tests::test_microsofts_people_are_folded_in_once_by_address`.

**5. What cannot be verified here.** A bind against a real Active Directory or OpenLDAP; which base Pratik's directory needs; whether `/me/people` returns directory people on his tenant (an administrator can switch "working with" off, stated in people-insights-overview); whether the consent screen shows `People.Read` as expected. Phase 14, and "experimental" in the account dialog's directory section and in the guide.

**6. Accessibility.**
- The account editor's second page is nearly out of letters. Ledger 606's fix (stated, `.planning/WINDOWS.md:623`) and the labels at `wx_account_manager.rs:1678-1985` give, across the IMAP, POP and browser sign-in variants of page two: R, I, P, L, O, F, D, C, S, T, E, N, U, W, G, V, A, M, K, X, Y, H, B. (derived) Free on page two in every variant: **J, Q, Z**. Two more labelled fields do not fit with guessable letters. And a third page is not free either: page two would then need Next, and N is the browser sign-in's on that page (ledger 606), while E, X and T, the other letters of "Next", are also taken. Question 5 offers the ways out.
- The password field is named on both channels by its visible label; the "a password is saved" sentence is on both channels like the permission note (`allowed_note`, `wx_account_manager.rs:1914`).
- Results from Microsoft join "People found" (`P&eople found:`) with where each came from in the row, so a person hears the source before choosing.

### GAP-08: free/busy from Google and from every source (#57)

**1. What the issue asks (stated, `gh issue view 57`, no comments).** "Google as a third source; every source an account has, asked; a guest's zone taken from their contact where it is known; the form scrolling." Points: no Google source (`where_to_ask`, `asking_when_free.rs:172`); one place per account; guests have no zone (`people_to_ask_about` sets `zone: None`, `:232`); ledger 417, the Edit Event form cut off at 768 pixels or 200% text; nobody has heard the answer read.

**2. What the tree has (stated).**

| Seam | Where | What it gives |
|---|---|---|
| Where to ask | `src/application/asking_when_free.rs:172-188` (`where_to_ask` returns one `WhereToAsk`: the first signed-in CalDAV server, else Microsoft, else `Nowhere`) | The one-place limit |
| One question | `asking_when_free.rs:238-243` (`one_question` builds exactly one `AskHere`) | The other half of the limit |
| No zone | `asking_when_free.rs:226-236` (`people_to_ask_about`, `zone: None` at `:232`) | Where a zone would be filled |
| Several places already accepted | `src/service/free_busy.rs:698-747` (`when_they_are_free(outward, places: &[AskHere], about)`, all asked at once with `join_all`) | Each person is answered from the place they were put in; no merge across places |
| The sources | `free_busy.rs:100-126` (`WhereToAsk::{CalendarServer, Microsoft, Nowhere}`), `:937-967` (`ask_microsoft`, `POST {base}/me/calendar/getSchedule`) | Google is a third arm |
| Microsoft's answer | `free_busy.rs:476-531` (`OneDiary { schedule_id, schedule_items, error }`) | `workingHours` is not read |
| The door | `src/service/outward.rs:211-213` (`asking_when_people_are_free`, a POST door kept apart from the change gate); callers only in `free_busy.rs:854` and `:956`; `test_asking_when_people_are_free_is_not_refused_by_the_gate_on_changes` at `outward.rs:2553` | Google's `POST /freeBusy` uses the same door |
| The call site | `src/presentation/managers.rs:1005-1110` (`asking_when_people_are_free`; token at `:1068`, `one_question` at `:1076`) | Where a Google token is fetched |
| Tokens | `oauth.rs:942-953` (`a_graph_token_for`, Microsoft only); `AuthManager::get_valid_token` at `oauth.rs:791`; Google calendars are `source_provider` `"gmail"` (`src/application/calendar.rs:74`, `GOOGLE`) | A `a_google_token_for` beside it |
| Scopes | `oauth.rs:73-84`: Google asks `https://www.googleapis.com/auth/calendar` already | No new Google scope |
| The answer's sentence about zones | `src/application/when_people_are_free.rs:471-485` ("Nobody said where Ada is, so the times were judged against the working hours set here."), per-person zone used at `:810-816` | A known zone removes a name from that sentence |
| Contacts carry no zone | `ContactEntry` fields at `src/data/message_cache/mod.rs:597-676`; `grep -n 'zone' src/data/message_cache/contacts.rs` found nothing | "from their contact" needs a new field (question 7) |
| The form | `src/presentation/wx_item_form.rs`, ledger 417 (`.planning/WINDOWS.md:434`) | The scrolling defect |

**Google `freeBusy` (stated, developers.google.com, freebusy/query).** `POST https://www.googleapis.com/calendar/v3/freeBusy` with `timeMin`, `timeMax`, `items[].id`; the answer is `calendars.(key).busy[]` with `start` and `end`, or `calendars.(key).errors[]` with reasons including `notFound`. Scopes accepted: `calendar.readonly`, `calendar`, `calendar.events.freebusy`, `calendar.freebusy`. (derived) The granted `calendar` scope covers it; no consent change.

**Microsoft `getSchedule` (stated, Microsoft Learn, calendar-getschedule v1.0).** Each `scheduleInformation` carries `workingHours` with `daysOfWeek`, `startTime`, `endTime` and `timeZone { name }`, where the name is a Windows name ("Pacific Standard Time") or, for a custom zone, "Customized Time Zone" with offsets. Permissions: `Calendars.ReadBasic` for work or school; **"Delegated (personal Microsoft account): Not supported."** (derived) A personal Outlook.com account's free/busy was never going to answer; the sentence should say so rather than report a server trouble.

**Windows zone names (stated, probe run today).** `windows` 0.62.2 with the feature `Win32_Globalization`, which this project's `Cargo.toml:400-401` already enables, exposes `ucal_getTimeZoneIDForWindowsID`, linked from `icuin.dll` (stated, `windows-0.62.2/src/Windows/Win32/Globalization/mod.rs:2558-2563`; `icuin.dll` present in `C:\Windows\System32`). A throwaway project in the scratchpad with its own target directory, built with toolchain 1.98.1 on x86_64-pc-windows-msvc, printed: "Pacific Standard Time" to "America/Los_Angeles", "GMT Standard Time" to "Europe/London", "India Standard Time" to "Asia/Calcutta", "W. Europe Standard Time" to "Europe/Berlin", "UTC" to "Etc/UTC", "Customized Time Zone" and "Not A Zone" to nothing. Note "Asia/Calcutta", an old name: the test must show `chrono-tz` parses what ICU returns.

**3. Proposed plans.**

**C6. Google as a free/busy source.** Closes GAP-08's first [D] line.
- What: `WhereToAsk::Google { base, token }`; `ask_google` through the same door, one request per batch; a pure `what_google_said(json, about)` mapping `busy` to stretches and each `errors` reason to a `WhyNot` (`notFound` is "their calendar is not shared with you", not "free"); `where_to_ask` learns an account with a `gmail` calendar; `a_google_token_for` in `oauth.rs`.
- Files: `src/service/free_busy.rs` (37 tests, 0 records), `src/application/asking_when_free.rs` (11 tests, 0 records), `src/service/oauth.rs`, `src/presentation/managers.rs` (54 records in `tests_last_seen`; no test added there), `docs/privacy.md` (`:289` onward), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: none.
- Spoken or shown: yes (answers change). Pull request.
- Size: M, three tasks.
- Failing tests: `service::free_busy::tests::test_googles_answer_becomes_the_same_stretches_a_calendar_server_gives`, `test_a_calendar_google_cannot_find_is_unknown_and_not_free`, `application::asking_when_free::tests::test_an_account_with_a_google_calendar_is_asked_at_google`.

**C7. Every source an account has.** Closes GAP-08's second [D] line.
- What: `where_to_ask` becomes `every_place_to_ask(...) -> Vec<WhereToAsk>` (every signed-in CalDAV server, Microsoft, Google); every person is asked at every place; a merge per person: answered where any place answered (busy stretches united), unknown only where no place answered, with the reasons kept for the sentence. `Nowhere` only when the list is empty.
- Files: `src/application/asking_when_free.rs`, `src/service/free_busy.rs`, `src/presentation/managers.rs`, `tests/item_form_free_busy.rs` (0 records), `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C6.
- Spoken or shown: yes. Pull request.
- Size: M, two tasks.
- Failing test: `service::free_busy::tests::test_a_person_one_place_answers_about_is_answered_even_where_another_did_not`, and `test_busy_time_from_two_places_is_united_not_replaced`.

**C8. A guest's zone, and the offered time said in it.** Closes GAP-08's third [D] line.
- What: Microsoft's `workingHours.timeZone.name` read and turned into a zone by the ICU call behind `#[cfg(target_os = "windows")]`, with a fallback of no zone elsewhere and for a custom zone; a person with a known zone leaves the "Nobody said where" sentence and each offered time is said with their clock beside it for up to the three people with known zones ("Tuesday at 10:00, which is 15:00 for Ada"); where question 7 says so, a zone from the contact.
- Files: a new small module for the zone name (planner's choice of `src/common/` or `src/service/`), `src/service/free_busy.rs`, `src/application/when_people_are_free.rs` (1 record), `src/application/asking_when_free.rs`, `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C7.
- Spoken or shown: yes. Pull request.
- Size: M, three tasks. If question 7 adds a contact zone field, that is a plan of its own after this one (12-07's lesson: a contact field touches about seventeen files).
- Failing tests: `..::test_a_windows_zone_name_becomes_the_zone_it_means` (with "Pacific Standard Time" and "India Standard Time" both parsed by `chrono-tz`), `service::free_busy::tests::test_microsofts_working_hours_give_the_guest_a_zone`, `application::when_people_are_free::tests::test_a_time_is_said_in_the_guests_own_clock_too`.

**C9 (question 8). Edit Event scrolls.** Ledger 417 and issue point 4, not a GAP-08 [D] line.
- What: the event form's content inside a scrolled window, so the Times offered list and the last four fields are reachable at 768 pixels and at 200% text, with focus scrolling the field into view.
- Files: `src/presentation/wx_item_form.rs`, a built-form test, `.planning/WINDOWS.md` (417 fixed), `docs/changelog.md`.
- Depends on: C8 (the same form's answer grows).
- Spoken or shown: shown (layout). Pull request, so the Accessibility scan reads it.
- Size: S to M, two tasks.
- Failing test: a built form at a fixed small client height whose last field reports a position inside the visible area after focus.

**5. What cannot be verified here.** Google's answer for a guest outside Pratik's domain (likely `notFound` unless shared); a CalDAV scheduling outbox answering alongside Microsoft; `getSchedule`'s zone for real colleagues; the sentences by ear (issue point 5, a listening line). Phase 14 and ledger entries; "experimental" is already on the free/busy button's page (the changelog's `:3820` note, stated in #57).

**6. Accessibility.** No new control in C6 to C8; the answer is text in the existing answer field and the Times offered list, whose names stay. Sentences stay short: at most three times said, a zone clause only for people whose zone is known, and the "Nobody said where" sentence names only those left. C9 is WCAG 1.4.10 and 1.4.4, and a keyboard user must see the field they are in (2.4.11).

### GAP-10: several identities per account (#59)

**1. What the issue asks (stated, `gh issue view 59`, no comments).** Proposed order: "1. Several identities per account (one mailbox, more than one address to send as) ... 2. Send on behalf: a `Sender` header ... 3. A shared mailbox opened as a folder tree of its own". "Every step needs a real server that shares a mailbox, which is the standing condition." GAP-10's [D] line takes step 1 only and says the rest is later work.

**2. What the tree has (stated).**

| Seam | Where | What it gives |
|---|---|---|
| The account | `src/data/account.rs` (`Account { name, sender_name, email, ... }`); persisted in the `accounts` table (`src/data/message_cache/mod.rs:2299`, columns added by `ensure_column_exists` at `:2970-2993`), `save_account` at `src/data/message_cache/accounts.rs:10` | One address per account |
| The From list | `src/presentation/wx_compose.rs:917-934` (a `Choice` of `account_names`, accessible name "From account" at `:927`, label `Reached::From.label()` = "&From:" at `src/presentation/editor_document.rs:1260`) | An index into accounts |
| Uses of that index | `wx_compose.rs:1896` (`account_index`), `:2068` (`from_account` for people lookup), `:317-341` (`follow_the_from_account`, signatures by position), `:3697-3721` (the send preview's "From:" line) | Every one must map an entry to (account, address) |
| People lookup by position | `src/presentation/finding_people.rs:21-37` ("`account_ids` is in the same order as the names in the From list") | Breaks silently if the list gains rows without the mapping |
| Sending | `src/application/mail_controller.rs:150-189` (`from_queued`: `from_address: account.email.clone()` at `:170`, `from_name` from `account.sender_name` at `:171`) | The outbox row must carry the chosen address |
| The outbox row | `QueuedOutboxMessage` (`mod.rs:925-966`: id, account_id, to, cc, bcc, subject, body, body_html, attachments, in_reply_to, ...); table `outbox_queue` at `mod.rs:2083` | No from column |
| Drafts | table `drafts` at `mod.rs:1817`, columns added at `:3116-3124`; `src/application/draft_message.rs:44` (`bytes_for(draft, from, from_name)`), `:162` (`sender_line`) | A draft saved to the server names From |
| Signatures per account (12-09) | tables `signatures` (`mod.rs:1862`) and `signature_assignments (account_id PRIMARY KEY, signature_id)` (`mod.rs:1884-1887`); `src/application/signatures.rs:17` (`which_signature`), `:41` (`whether_to_swap`) | An identity can take its account's signature without a schema change |
| The comparison page | `docs/comparison.md:45` ("Several identities per account ... Not yet"), `:123` | Corrected by C12 |
| SMTP | `src/service/protocols/smtp.rs:54-56` (`Email { from, from_name, ... }`), no `Sender` field | Send on behalf is later work |

**3. Proposed plans.**

**C10. Identities stored and managed.**
- What: `CREATE TABLE IF NOT EXISTS identities (id TEXT PRIMARY KEY, account_id TEXT NOT NULL, address TEXT NOT NULL, sender_name TEXT NOT NULL DEFAULT '', position INTEGER, created_at TEXT NOT NULL, UNIQUE(account_id, address))`; removed with the account; a pure `application::identities` (an address is an address, not the account's own, not a duplicate; the From list's entries for a set of accounts, account first then its identities in order); a small manager dialog reached from the account editor's first page.
- Files: `src/data/message_cache/mod.rs` (12 records), a new `src/data/message_cache/identities.rs`, `src/application/identities.rs` (new), `src/application/mod.rs`, `src/presentation/wx_account_manager.rs`, a new `src/presentation/wx_identities.rs`, `docs/KEYBOARD_SHORTCUTS.md` (a dialog accelerators section), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C4 if both edit `wx_account_manager.rs` (sequencing), none in substance.
- Spoken or shown: yes. Pull request.
- Size: M, three tasks.
- Failing tests: `application::identities::tests::test_the_from_list_is_each_account_then_its_other_addresses`, `test_an_address_already_the_accounts_own_is_refused_with_a_sentence`, `data::message_cache::identities::tests::test_an_identity_goes_with_its_account`.

**C11. Sending as an identity.**
- What: the From list built from C10's entries, its accessible name "From" (it now chooses an address); every consumer of the index maps an entry to (account, address), people lookup included; `outbox_queue` and `drafts` gain `from_address` and `from_name` by `ensure_column_exists`, NULL meaning the account's own (every row written before this reads as before); `from_queued` and `draft_message::bytes_for` use them; the preview's From line shows the address; the signature is the account's (question 9).
- Files: `src/presentation/wx_compose.rs` (8 records), `src/presentation/finding_people.rs`, `src/data/message_cache/mod.rs`, `src/data/message_cache/outbox.rs`, `src/data/message_cache/drafts.rs`, `src/application/mail_controller.rs` (23 records in `tests_last_seen`; add the test elsewhere if a pure helper can hold it), `src/application/draft_message.rs`, `src/presentation/wx_app.rs` (where compose is opened and queued), `tests/a_signature_follows_the_from_account.rs` (3 records), `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C10.
- Spoken or shown: yes. Pull request.
- Size: L as listed; split it into C11a (the outbox and draft columns and `from_queued`, pure and data only) and C11b (the compose list and its consumers).
- Failing tests: `application::mail_controller::tests::test_a_queued_message_goes_out_from_the_address_it_was_written_from` and `test_a_row_queued_before_identities_goes_out_from_the_account`; a compose-level test that choosing the second entry looks people up in the first account's directory.

**C12. A reply goes out from the address it was sent to, and the pages say what is not built.** (question 10)
- What: a reply or forward opens with the identity whose address is among the original's To or Cc; `docs/comparison.md:45` corrected by date; shared mailboxes, send on behalf and delegation named as later work in the comparison page and the guide, and issue #59 left open for them.
- Files: `src/application/reply.rs`, the compose opener in `src/presentation/wx_app.rs`, `docs/comparison.md`, `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- Depends on: C11.
- Spoken or shown: yes (the From line). Pull request.
- Size: S, two tasks.
- Failing test: `application::reply::tests::test_a_reply_to_mail_sent_to_an_identity_is_from_that_identity`.

**5. What cannot be verified here.** Whether Gmail sends from an address that is not set up under its "Send mail as" ([ASSUMED] it replaces the From with the primary address), whether Exchange Online refuses a From it does not own ([ASSUMED] it refuses with a send-as error), and what the recipient sees. Every identity is marked experimental in the manager and the guide, saying the provider may refuse or replace the address unless it is set up there too. Phase 14.

**Cannot be finished here:** shared mailboxes, send on behalf and delegation. They need IMAP `NAMESPACE`, ACL, a `Sender` header and a real shared mailbox (#59's own words); they are said as later work, not stubbed.

**6. Accessibility.**
- The account editor's first page (stated, `wx_account_manager.rs:1678-1700` and ledger 606) holds A (Account Name), E (Email Address), M (the name people see), F (Signature for this account), N (Next). A button there, "&Other addresses to send from...", takes O, which is free on that page.
- The manager dialog is new, so its letters are its own: "Addresses to send &from:" (the list), "&Add...", "&Edit...", "&Remove", "&Close"; the edit dialog "&Address:" and "The &name people see:". F, A, E, R, C and A, N are each used once per window.
- The From list's entries read as "Work, ada@example.com" then "Work, as help@example.com", so the address is heard, not only the account's label; the "From account" name becomes "From".
- A change of From that swaps the signature is announced as today (`follow_the_from_account`).

## Standard Stack

No new crate. Everything is in `Cargo.toml` today (stated).

| Library | Version (stated, `Cargo.lock`) | Used for in this group |
|---|---|---|
| `ldap3` | 0.12.1, `default-features = false`, `tls-rustls-ring` (`Cargo.toml:261`) | The bind already written; C3 adds a filter |
| `async-imap` / `imap-proto` | 0.11.3 / 0.16.7 | `PERMANENTFLAGS` read (`ResponseCode::PermanentFlags`) |
| `reqwest` through `service::outward::Outward` | in the tree | Graph people search, Google `freeBusy` |
| `windows` | 0.62.2, feature `Win32_Globalization` already on (`Cargo.toml:400-401`) | `ucal_getTimeZoneIDForWindowsID` |
| `chrono-tz` | 0.10 | Parsing the IANA name ICU returns |
| `keyring` through `service::secret_store` | in the tree | The directory password |

## Package Legitimacy Audit

No package is installed by any plan in this group, so no plan waits on Pratik's confirmation of a package, and `gsd-tools package-legitimacy` was not run. The brief asked for `ldap3` to be audited against Windows `wldap32`; it is already in the tree, so this is a re-audit of an existing dependency for a widened use, in the dependency-audit skill's form.

```
Dependency: ldap3@0.12.1 (with lber@0.5.1), already in Cargo.toml
Purpose: bind and search an LDAP directory; C3 adds a password and skips references
Size: 0 new lock entries. Of 147 packages in its closure in this Cargo.lock, 2 are
      reached only through it (ldap3 0.12.1, lber 0.5.1); measured 2026-09-24 by a
      closure over Cargo.lock with the root's other dependencies as the control
      (the first run of that script reported 0 unique, because it did not skip the
      root package; corrected and re-run). The lock also records optional
      dependencies no feature turns on (observation 805), so 147 is an upper bound
      for what is built.
Maintenance: last release 0.12.1 on 2025-09-18 (crates.io), last commit the same
      day, one principal maintainer (inejge, 447 commits; dequbed 97). Six open
      items on 2026-09-24, one of them (#157, 2026-09-23) asking whether it is
      still maintained, unanswered. Bus factor one.
License: declared "MIT/Apache-2.0"; LICENSE-APACHE and LICENSE-MIT ship in the
      crate (listed in the registry copy). lber declares MIT and ships LICENSE.
Advisories: none against ldap3, lber or nom 7. cargo-audit 0.22.2, run from the
      scratchpad against this Cargo.lock with no ignore list (--no-fetch, advisory
      database dated 2026-09-24): 3 vulnerabilities, all already accepted in
      .cargo/audit.toml (quick-xml twice, rsa), which is the control that the scan
      reads the lock. None names this crate's closure uniquely.
Build: builds today on x86_64-pc-windows-msvc with toolchain 1.98.1 (CI's Build
      job builds the tree); no probe needed.
Known defect on the path we call: issue #156, SearchEntry::construct panics on a
      searchResRef; routed around by skipping is_ref() entries (C3 task 1).
      Issue #135: construct panics on any malformed entry, and the maintainer does
      not plan a non-panicking variant; the tree's catch_unwind stays.
Alternatives:
  - wldap32 through the windows crate (feature Win32_Networking_Ldap, not on
    today; ldap_sslinitW, ldap_bind_s, ldap_search_ext_sW stated at
    windows-0.62.2/src/Windows/Win32/Networking/Ldap/mod.rs:248, 1460, 1640;
    wldap32.dll present). Gains: Windows' own TLS and certificate store, and
    Negotiate sign-in as the logged-in Windows user on a domain-joined machine,
    so no password is typed or stored (WCAG 3.3.8's spirit). Costs: a C API with
    manual memory and BER handling, unsafe code throughout, a rewrite of a
    working module, and the whole test seam rebuilt.
  - ldap3's own "gssapi" or "ntlm" features (stated, its Cargo.toml features) for
    single sign-on later, which pull cross-krb5 or sspi; not needed for a
    simple bind.
  - Writing LDAP ourselves: BER encoding, referrals, paging. No.
Defaults overlapping rules we already hold: none; default features are off.
Measured unchanged: not measured (nothing installed).
Recommendation: KEEP ldap3 for C3 and C4. Record single sign-on through wldap32
      or ldap3's gssapi as a later option (question 6 names it), and a ledger line
      watching #157 for a maintainer answer.
```

| Package | Registry | Age | Downloads | Source repo | Verdict | Disposition |
|---|---|---|---|---|---|---|
| (none new) | | | | | | |

**Packages removed due to [SLOP] verdict:** none. **Packages flagged [SUS]:** none.

The Windows zone mapping in C8 is a Windows API through a feature already enabled; alternatives considered: a hand-written table from CLDR's `windowsZones.xml` (about 140 rows to keep current by hand, derived), or a crate carrying that table (none evaluated, since the API answered in the probe).

## Architecture Patterns

### Data flow

```
Report as Junk (Ctrl+Shift+J), selected messages
  -> chosen_messages (wx_app) -> reporting_junk::what_a_report_does(account, folders, allowed)
       POP / changes off / no junk folder -> one sentence, nothing sent
       IMAP -> worker: select folder -> PERMANENTFLAGS -> keeps keywords?
                 yes -> set_flag $NotJunk off, $Junk on (gated)
               -> move into junk folder (spawn_folder_move path, gated)
  -> one sentence per account kind (Gmail: "which tells Google"; Microsoft: "Microsoft has not been told")

Block This Sender -> rule written (as today)
  -> what_a_rule_catches_here(rule, cached messages outside Junk/Trash/Sent/Drafts) -> count
  -> question with the count -> yes -> spawn_folder_move -> "14 messages moved to Junk"

Typing a name in To
  -> finding_people::who_matches
       contacts here (as today)
       LDAP: settings -> password from secret_store -> refuse over ldap:// -> bind -> search
             -> skip references -> entries
       Microsoft OAuth account: token (People.Read) -> GET /me/people?$search -> parse
  -> everybody_found (by address, once) -> "People found" list + one trouble line

Find when everyone is free
  -> every_place_to_ask(calendars, sign-ins, Microsoft token, Google token) -> [places]
  -> when_they_are_free: every person at every place, all at once
  -> merge per person (answered anywhere wins; busy united)
  -> Microsoft workingHours zone -> ICU -> IANA -> person.zone
  -> sentences: times, per-guest clock, who could not be checked

Compose From list = accounts and their identities
  -> choice -> (account, address) -> outbox row from_address/from_name
  -> from_queued -> Email.from -> SMTP
```

### Patterns to follow

- **The decision pure, the wire thin.** Every new network call gets a pure parser tested from recorded answers (Graph JSON, Google JSON, an LDAP entry list) and a thin sender, as `free_busy.rs` and `directory.rs` already do.
- **A census that goes red on arrival is the red half for free.** `forget.rs`'s owners reading for a new `keyring_service`; the scope tests in `oauth.rs`; `tests/wired.rs` for menu letters and keys.
- **Put new tests in new or lightly guarded files.** See pitfall 1.
- **One sentence per command, never per message** (`spawn_folder_move`'s rule since #30).

### Anti-patterns

- Adding `Mail.ReadWrite` to reach a beta endpoint.
- A scope added to the consent list and not to `THE_SCOPES_A_GRAPH_TOKEN_CARRIES` (ledger 282's shape).
- Treating Google's `notFound` or Microsoft's per-person `error` as free time.
- Indexing accounts by the From list's position once identities exist.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Windows zone names | a mapping table | `ucal_getTimeZoneIDForWindowsID` (Windows ICU) | Windows keeps it current; probed today |
| LDAP protocol | a BER client | `ldap3` (in the tree) | Referrals, TLS, paging |
| Rule matching over existing mail | a second matcher | `FilterEngine::matches` | One answer to "why did this move" |
| Moving many messages | a second mover | `spawn_folder_move` | Cross-account, copied-not-moved and set sentences already handled |
| Secrets | a settings field | `secret_store` with one owner and `forget.rs` | The uninstall sweep |

## Common Pitfalls

1. **Guard-record cost of where a test goes (stated, the TOML parser, 2026-09-24, 1,088 records).** A test added to `src/presentation/wx_app.rs` flags 119 records; to `src/service/protocols/imap.rs` 47; `src/presentation/managers.rs` 54; `src/application/mail_controller.rs` 23; `src/data/message_cache/mod.rs` 12; `src/presentation/wx_account_manager.rs` 9; `src/presentation/wx_compose.rs` 8. Zero for `directory.rs`, `free_busy.rs`, `asking_when_free.rs`, `imap/flag.rs`, `finding_people.rs`, `tests/finding_people_answers.rs`, `tests/item_form_free_busy.rs`. Put tests in the zero files. Command: `python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(sum(1 for r in g if any(e['file']=='<file>' for e in r.get('tests_last_seen',[]))))"`.
2. **A scope on one list and not the other** (ledger 282, `oauth.rs:831-857`). C5 adds `People.Read` to both, with a test that reads the constant.
3. **`ldap3` #156** skips real entries today through the panic catch; C3 fixes it before the sign-in widens use to Active Directory.
4. **A password over plain `ldap://` travels in clear.** Refuse it (question 6).
5. **The From list's index** feeds four consumers (`wx_compose.rs:317-341`, `:1896`, `:2068`, `:3697-3721`) and `finding_people.rs:21-37`. Map all of them in C11b or people lookup asks the wrong account.
6. **`getSchedule` does not serve personal Microsoft accounts** (stated). Say so rather than reporting a server fault.
7. **ICU returns old zone names** ("Asia/Calcutta"); test that `chrono-tz` parses each value returned for a sample of Windows names.
8. **`$Junk` and `$NotJunk` together mean neither** (RFC 9051). Clear `$NotJunk` before setting `$Junk`.
9. **Key collisions across groups.** `Ctrl+Shift+J` and `Ctrl+Shift+B` are free today; groups 1 and 4 may reach for the same, and Quick Steps will want keys. The synthesizer checks all five research files together.
10. **The census of gated writes in `outward.rs`** attributes `may_i(` markers per method (`imap.rs:1348-1365`); reuse `set_flag` and `move_message` rather than adding a new gated IMAP method, or the census needs a new measured command.
11. **Mnemonic letters on the account editor's page two** are exhausted but for J, Q, Z (derived, ledger 606). Re-read with `tests/account_edit_protocol_fields.rs` before any label is written.

## Code Examples

Skipping references before constructing entries (derived from `ldap3-0.12.1/src/search.rs:63-66` and issue #156's comment):

```rust
// Source: ldap3 0.12.1, ResultEntry::is_ref; issue inejge/ldap3#156
fn the_entries_among(results: Vec<ldap3::ResultEntry>) -> Vec<ldap3::ResultEntry> {
    results.into_iter().filter(|result| !result.is_ref()).collect()
}
```

The Windows zone call (stated: this compiled and ran in the probe with `windows` 0.62.2, `Win32_Globalization`, toolchain 1.98.1):

```rust
use windows::Win32::Globalization::{ucal_getTimeZoneIDForWindowsID, UErrorCode, U_ZERO_ERROR};

fn the_zone_windows_calls(name: &str) -> Option<String> {
    let wide: Vec<u16> = name.encode_utf16().collect();
    let mut out = [0u16; 128];
    let mut status: UErrorCode = U_ZERO_ERROR;
    let written = unsafe {
        ucal_getTimeZoneIDForWindowsID(
            wide.as_ptr(), wide.len() as i32, windows::core::PCSTR::null(),
            out.as_mut_ptr(), out.len() as i32, &mut status,
        )
    };
    (status.0 <= 0 && written > 0).then(|| String::from_utf16_lossy(&out[..written as usize]))
}
```

(The tree forbids `as` casts only where clippy says so; the planner converts with `i32::try_from` as `directory.rs:621-625` does.)

Google's request and answer shape (stated, the freebusy/query page):

```json
POST https://www.googleapis.com/calendar/v3/freeBusy
{ "timeMin": "2026-10-01T00:00:00Z", "timeMax": "2026-10-15T00:00:00Z",
  "items": [ { "id": "ada@example.com" } ] }
-> { "calendars": { "ada@example.com": { "busy": [ { "start": "...", "end": "..." } ] },
                    "bob@elsewhere.test": { "errors": [ { "domain": "global", "reason": "notFound" } ] } } }
```

## State of the Art

| Old | Current | When | Impact |
|---|---|---|---|
| Graph `markAsJunk` (beta) | `reportMessage` (beta only) | deprecated, stopped 2025-12-30 | No supported Graph report; IMAP move is the route |
| Graph People API `/me/people` | Microsoft Search `/search/query` person entity | People API "in maintenance mode" | Use `/me/people` now; ledger the move |
| `ldap3` 0.11 `tls-rustls` | 0.12 requires a crypto provider feature | 0.12.1, 2025-09 | The tree already names `tls-rustls-ring` |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|---|---|---|
| A1 | Microsoft does nothing with an IMAP move into Junk Email beyond filing it | GAP-06 | The sentence undersells; harmless |
| A2 | Gmail replaces a From that is not set up under "Send mail as" | GAP-10 | The warning is wrong in the other direction |
| A3 | Exchange Online refuses a From the account does not own | GAP-10 | A send fails with a provider sentence |
| A4 | Microsoft's v2 endpoint grants `People.Read` by dynamic consent without a change to the app registration | GAP-07 | A consent error at sign-in until the registration lists it |
| A5 | Letters free on the account editor's page two are J, Q, Z in every variant | GAP-07 | Derived from ledger 606 and the label list; the test re-reads it |

## Questions only Pratik can answer

1. **Microsoft junk reports.** Microsoft's only way for a program to report junk is an API it calls unsupported for production (beta), and it would need a new permission to read and change all mail. Recommendation: move to Junk Email and say plainly that Microsoft was not told; no new permission.
2. **A block and the mail already here.** Ask first with the count ("Also move the 14 messages already here from them to Junk?"), or move and say? And which folders: every folder but Junk, Trash, Sent and Drafts, or the Inbox only? Recommendation: ask once with the count; every folder but those four.
3. **Keys.** Report as Junk on `Ctrl+Shift+J`; Block This Sender on `Ctrl+Shift+B`, Everyone at This Domain without a key. Recommendation: yes to both keys, confirmed against the other groups' keys first.
4. **A new Microsoft permission.** People search needs "read your relevant people list" (`People.Read`), so everyone with a Microsoft account is asked to sign in again once. And the same list is missing the tasks permission already asked for (ledger 282), which could be fixed in the same change. Recommendation: yes to both, in one sign-in round.
5. **Where the directory sign-in goes.** The account window's second page has three free Alt letters (J, Q, Z) and a third page cannot have a lettered Next. Options: (a) a "Look People Up at Work..." window of its own from the Account Manager's buttons (K is free there), holding the address, where to look, the sign-in name and the password, and the two boxes leave page two; (b) two more fields on page two with J and Q; (c) a button on page two with one of those letters. Recommendation: (a), one window for one job.
6. **An unencrypted directory address with a password.** Refuse to send the password to an `ldap://` address, saying to use `ldaps://`? And later, signing in as the Windows user with no password (Windows' own LDAP or `ldap3`'s Kerberos support)? Recommendation: refuse now; Windows sign-in as later work.
7. **Where a guest's time zone comes from.** From Microsoft's answer for work colleagues (automatic), and also from a new "time zone" field on a contact (a bigger change to the contact editor and its sync)? Recommendation: Microsoft's answer now; the contact field as a later plan.
8. **The Edit Event form that does not scroll** (ledger 417, #57 point 4). In this phase after the free/busy plans, or later? Recommendation: in this phase, since these plans make its answer longer.
9. **Identities and signatures.** An extra address signs with its account's signature, or each address gets its own choice? Recommendation: the account's signature now.
10. **Replies.** Should a reply go out from the address the message was sent to? Recommendation: yes, a small plan (C12).
11. **Gmail's own list of aliases.** The existing Gmail permission can read the "Send mail as" addresses, so they could be offered automatically. Recommendation: later; typed identities first.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|---|---|---|---|---|
| Rust toolchain 1.98.1 (pinned) | all | yes | 1.98.1-x86_64-pc-windows-msvc | none |
| `icuin.dll` / `icu.dll` | C8 | yes, `C:\Windows\System32` | Windows 11 26200 | no zone, said |
| `wldap32.dll` | not used | yes | | |
| `cargo-audit` | the audit above | yes | 0.22.2 | |
| `gh` | reading issues | yes | | |
| A real IMAP, LDAP, Microsoft, Google account | proofs | no | | phase 14 and ledger |

## Validation Architecture

| Property | Value |
|---|---|
| Framework | `cargo test` (built-in), `tokio-test`, `tempfile` |
| Config | none beyond `Cargo.toml` |
| Quick run | one module per run: `cargo test --lib service::directory:: && cargo test --lib service::free_busy::` |
| Full suite | `scripts/check.sh all`, once, in the phase's closing plan |

| Req | Behaviour | Type | Command | Exists? |
|---|---|---|---|---|
| GAP-06 | report decision per account kind | unit | `cargo test --lib application::reporting_junk::` | new, C1 |
| GAP-06 | keyword kept only where PERMANENTFLAGS allows | unit | `cargo test --lib service::protocols::imap::flag::` | file exists, tests new |
| GAP-06 | existing mail a block catches, counted | unit | `cargo test --lib application::what_a_rule_catches_here::` | new, C2 |
| GAP-06 | menu letters and keys | reading | `cargo test --test wired` | exists |
| GAP-07 | references skipped; password refused over ldap:// | unit | `cargo test --lib service::directory::` | exists |
| GAP-07 | the directory password is erased at uninstall | unit | `cargo test --lib application::forget::` | exists (red on arrival) |
| GAP-07 | People.Read on both lists; Graph answer parsed | unit | `cargo test --lib service::oauth:: && cargo test --lib service::microsoft_graph::` | exists |
| GAP-07 | the page letters | built dialog | `cargo test --test account_edit_protocol_fields` | exists |
| GAP-08 | Google parse, merge, zones | unit | `cargo test --lib service::free_busy:: && cargo test --lib application::asking_when_free::` | exists |
| GAP-08 | the form asks every place | integration | `cargo test --test item_form_free_busy` | exists |
| GAP-10 | From list entries, outbox from, reply identity | unit | `cargo test --lib application::identities:: && cargo test --lib application::mail_controller::` | new and exists |

Wave 0 gaps: none; every framework and target exists. Manual-only: every sentence heard under NVDA and Narrator, recorded as listening lines.

## Security Domain

| ASVS category | Applies | Control |
|---|---|---|
| V2 Authentication | yes | Directory password in the Windows credential store via `secret_store`; never in settings, logs or the database; refused over plain `ldap://` |
| V3 Session | no | |
| V4 Access control | yes | The write gate (`may_i`, `Allowed`) in front of the keyword write and every move |
| V5 Input validation | yes | Typed text escaped by `ldap_escape` (existing); Graph `$search` value quoted and URL-encoded through `outward::in_a_query`; provider JSON parsed with `serde`, unknown fields ignored |
| V6 Cryptography | yes | rustls through `ldap3` and `reqwest`; nothing hand-rolled |
| V8 Data protection | yes | Least scope: `People.Read` only; no `Mail.ReadWrite`; the privacy page names every new recipient of a typed name |

| Threat | STRIDE | Mitigation |
|---|---|---|
| A directory password sent in clear | Information disclosure | refusal over `ldap://` (C3) |
| A crafted LDAP answer panicking the lookup | Denial of service | `is_ref` filter plus the existing `catch_unwind` |
| A block moving a colleague's whole history | Tampering | the count asked first (question 2); moves go to Junk, never Trash |
| A scope granted but not held | Repudiation of a feature | both-lists test |
| Typed names sent to Microsoft | Information disclosure | only for a Microsoft account, only while typing in To, Cc, Bcc; the privacy row |

## Sources

### Primary (HIGH)
- The tree at `630e2a67`, every file and line cited above, read 2026-09-24.
- `gh issue view 54`, `55`, `57`, `59` (bodies; none has comments).
- RFC 9051 (rfc-editor.org/rfc/rfc9051.html), section 2.3.2 and SELECT.
- Microsoft Learn: message-markasjunk (beta), message-reportmessage (beta), user-list-people (v1.0), people-insights-overview, search-concept-person, calendar-getschedule (v1.0).
- Google: developers.google.com/workspace/calendar/api/v3/reference/freebusy/query; developers.google.com/workspace/gmail/api/reference/rest/v1/users.settings.sendAs/list; support.google.com/mail/answer/1366858.
- `ldap3` 0.12.1 source in the cargo registry; github.com/inejge/ldap3 issues #135, #156, #157; crates.io API for ldap3.
- `windows` 0.62.2 source in the cargo registry; the ICU probe in the scratchpad (`phase-13-probes/icuzone`).
- `cargo audit` 0.22.2 against `Cargo.lock`, unsuppressed, from the scratchpad.

### Secondary (MEDIUM)
- Web search results on Gmail "Send mail as" and SMTP (behaviour of an unconfigured alias not confirmed; A2).

## Metadata

- Standard stack: HIGH, nothing new, every version read from the lock.
- Architecture: HIGH, every seam opened.
- Provider behaviour: MEDIUM, documented but unexercised.
- Pitfalls: HIGH for the tree's own; MEDIUM for providers.
- Valid until: the next change to `wx_app.rs`'s Action menu, `wx_account_manager.rs` or `oauth.rs`; re-take line numbers and record counts when planning.

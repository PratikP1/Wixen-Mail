# Wixen Mail: what is built and what is left

_Compiled 2026-08-29 from the project's own documents and the tree. Every path is
relative to the repository root._

**One caveat that applies to the whole document.** Nothing in Wixen Mail has ever
run against a real mail account or a real provider. Where a row below says
"exercised", that means tested against parsing, fuzzed input, and loopback
servers in CI, not against Gmail, Microsoft Graph, a CalDAV server, or any live
IMAP, SMTP or POP3 host. `docs/IMPLEMENTATION_STATUS.md` and
`docs/ALPHA_TESTING.md` both say this in their own words, and
`src/application/allowed.rs` encodes it as a safety catch.

Sources used, in the order of authority given: `docs/changelog.md` (the
`[Unreleased]` section spans lines 7 to 7420), `docs/IMPLEMENTATION_STATUS.md`,
`docs/roadmap.md`, `docs/development/requirements-backlog.md`,
`docs/plans/20260726-mail-at-scale.md`, `docs/ALPHA_TESTING.md`, the code, and
`.planning/codebase/CONCERNS.md`.

## Built and exercised

Present in the tree, reached from a non-test path, and covered by tests over
parsing or loopback servers.

| Feature | Evidence | Source |
|---|---|---|
| IMAP client: folders, fetch, search, capabilities, SPECIAL-USE, UIDPLUS, MOVE, CONDSTORE, ID, X-GM-EXT-1, STATUS, LSUB/SUBSCRIBE, APPEND, COPY | `src/service/protocols/imap.rs`, `src/service/protocols/imap/` | roadmap Phase 2; IMPLEMENTATION_STATUS "What works" |
| IMAP IDLE watch and renewal, CONDSTORE incremental sync via `changed_since` | `src/service/protocols/imap.rs`; `src/application/mail_sync.rs` | mail-at-scale status table, step 6 |
| SMTP with TLS and STARTTLS, PLAIN/LOGIN, and XOAUTH2 | `src/service/protocols/smtp.rs`, `src/service/protocols/xoauth2.rs` | roadmap Phase 2; changelog `[Unreleased]` Known limitations, "Closed further up this same release" |
| POP3 to RFC 1939 over TCP, TLS on 995, STLS on 110, UIDL-keyed sync, leave-on-server policy, local folders | `src/service/protocols/pop3.rs`, `src/application/pop_sync.rs`, `src/application/local_folders.rs` | roadmap Phase 2 |
| OAuth 2.0: authorization flow, real token exchange, refresh, tokens in the Windows credential store, and a local callback listener on `http://localhost:<port>/oauth/callback` | `src/service/oauth.rs` (redirect built at line 330, request loop at line 630), `src/service/oauth_credentials.rs`, `src/service/secret_store.rs` | code; requirements-backlog "OAuth 2.0 Authentication: Done" |
| Local SQLite cache for messages, contacts, groups, calendars, events, reminders, task lists, tasks, note folders and notes; bodies split out with a size budget and least-recently-read eviction | `src/data/message_cache/mod.rs`, `src/data/message_cache/bodies.rs` | mail-at-scale status table, steps 1 and 5 |
| Six modules in one window (mail, contacts, calendar, reminders, tasks, notes), `Ctrl+Shift+1` to `Ctrl+Shift+6` | `src/presentation/wx_app.rs` | IMPLEMENTATION_STATUS "The six modules" |
| Native virtual mode for the message list and every PIM list | `src/presentation/wx_app.rs` registers the virtual text callback; `message_rows.rs`, `pim_rows.rs` | mail-at-scale status table, step 2; backlog marks virtual scrolling Done |
| Column sorting, order and visibility, sorted in SQL from the stored layout | `wx_columns::show_column_dialog` | mail-at-scale status table; changelog "Since closed: the listing query reads the stored column layout and sorts in SQL" |
| Compose: To/Cc/Bcc/Subject/Body, HTML and plain modes, drafts with auto-save, signatures per account, preview-before-send | `src/application/draft_message.rs`, `src/application/autosave.rs`, `src/application/sign_off.rs` | roadmap Phase 4 |
| Hold and take back a send (Undo Send, `Ctrl+Shift+Z`) | `src/application/sending_later.rs` | changelog `[Unreleased]` Added, lines 132 and 142 |
| Blocking a sender or a whole domain | `src/application/blocking.rs` | changelog `[Unreleased]` Added, line 121 |
| Saved searches, appearing in the folder tree, and saying when they cannot run | `src/application/saved_searches.rs` | changelog `[Unreleased]` Added, lines 11 and 46; backlog marks it Done |
| Meeting invitations: guest list, replies, and working out when everyone is free | `src/application/who_is_coming.rs`, `asking_when_free.rs`, `when_people_are_free.rs`, `src/service/free_busy.rs` | changelog `[Unreleased]` Added, lines 56 and 153 |
| Organisation directory lookup from the account settings | `src/service/directory.rs`, `src/application/looking_people_up.rs` | changelog `[Unreleased]` Added, lines 95 and 110 |
| Tagging with colour, message filters with regex and rule actions, filter management UI | `src/application/tagging.rs`, `src/application/filters.rs` | roadmap Phase 5; backlog marks colour-coded tags Done |
| Full-text search (FTS) with advanced filters | `src/data/message_cache/mod.rs`, `src/application/search.rs` | roadmap Phase 5 |
| Address book CRUD, fuzzy search, groups, vCard 3.0 import and export, recipient autocomplete | `src/application/contacts_sync.rs`, `src/application/contact_groups.rs`, `src/application/importing_contacts.rs` | requirements-backlog "Contact Management: Done" |
| Conversation view: press Enter on a message to see the surrounding messages as a tree, threading from `References` headers | `src/presentation/wx_thread_view.rs`, `src/application/threading.rs` | changelog Known limitations, "Half closed further up this same release" |
| HTML rendering with everything runnable stripped (ammonia), plain-text fallback, attachment metadata, save to disk, PDFs read in-app | `src/application/body_safety.rs`, `src/service/pdf.rs`, `src/application/attaching.rs` | roadmap Phase 3; requirements-backlog "HTML Rendering & Attachment Pipeline" |
| PGP and S/MIME signature detection, S/MIME verification, phishing risk scoring | `src/service/security.rs`, `src/service/signed_mail.rs` | roadmap Phase 5; backlog "S/MIME signature checking has since been built" |
| Multi-account: add, update, delete, enable, active-account switch, provider presets, per-account isolation, unified inbox | `src/application/accounts.rs` | roadmap Phase 6 |
| Offline mode toggle, outbox queue with CRUD, flush to SMTP on reconnect, outbox and sync indicators in the status bar | `src/application/mail_controller.rs`, `src/application/mail_session.rs` | roadmap Phase 7; requirements-backlog "Offline Mode & Queued Send: Done" |
| Accessibility layer: accessible names via `wxAccessible`, prioritised and deduplicated announcements bounded to four per second, mute with `Ctrl+M` | `src/presentation/accessibility/announcements.rs`, `names.rs`, `automation.rs`, `screen_reader.rs` | IMPLEMENTATION_STATUS "Accessibility" |
| Earcons and importable sound schemes; speech and braille ride one screen reader notification | `src/presentation/accessibility/feedback.rs`, `sound_scheme.rs`, `sound_scheme_import.rs` | mail-at-scale "Feedback channels, revisited" |
| Themes: light, dark, Windows high contrast, applied to every module and dialog | `src/presentation/theme.rs` | backlog marks theme customization Done |
| Windows installer (Inno Setup) | `installer/` | backlog marks it Done |
| Spell check on send, using Windows' own checker on Windows | `src/application/spell_session.rs`, `src/service/spellcheck/` | roadmap Phase 4, marked partial |
| Outlook PST/OST import into a non-server-backed Imported area | `src/service/outlook_data_file.rs`, `src/application/importing_messages.rs` | changelog line 674 onward |
| CI quality gates: rustfmt, clippy `-D warnings`, tests, release build, plus Axe.Windows and MSAA scans and a real-NVDA workflow | `scripts/check.sh`, `.github/workflows/ci.yml`, `accessibility.yml`, `nvda.yml`, `mutants.yml` | IMPLEMENTATION_STATUS "Quality gates" |

## Built but unproven

Present and reached, never run against the real thing. This is the section where
"done" is doing the most work.

| Feature | Evidence | Source |
|---|---|---|
| Everything the mail transports do on the wire: fetch, send, delete, move, copy, file a copy in Sent, read receipts, subscription changes | `src/service/protocols/imap.rs` (3,673 lines), `smtp.rs` (1,411), `pop3.rs` (696) | IMPLEMENTATION_STATUS "Anything that writes, against a real account"; CONCERNS "Network transport tested for parsing, not against live servers" |
| POP3 end to end, including the local folders and the remove-after-N-days policy | `src/service/protocols/pop3.rs`, `src/application/pop_sync.rs` | ALPHA_TESTING: "A POP account has never been run against a real POP server. Everything about it is new in this version" |
| Google client for contacts, calendar and tasks | `src/service/google_api.rs` (1,858 lines), `src/service/tasks_api.rs` | IMPLEMENTATION_STATUS "Provider sync... None of it has been exercised against a live account" |
| Microsoft Graph client for contacts, calendar and tasks | `src/service/microsoft_graph.rs` (1,613 lines) | same |
| CalDAV client, including adding a calendar by its address | `src/service/caldav.rs` (8,149 lines), `src/application/caldav_sync.rs`, `src/presentation/wx_add_calendar.rs` | IMPLEMENTATION_STATUS "Known gaps in verification"; CONCERNS names this the largest unexercised file |
| iCal subscription feeds | `src/service/ical_subscription.rs`, `src/application/calendar_source.rs` | IMPLEMENTATION_STATUS "Provider sync" |
| The three personal-information sync push paths, gated behind `Allowed::personal_information` and on by default for a new install | `src/application/contacts_sync.rs` (12,247 lines), `calendar.rs` (11,154), `caldav_sync.rs` (5,933), `tasks_sync.rs` (5,142) | `src/application/allowed.rs`: "this is the least proven code in the application: none of the three sync paths has met a live account" |
| Mail writes, gated behind `Allowed::mail`, which is off for a new install | `src/application/mail_sync.rs` (3,717 lines), `server_delete.rs`, `deleting_at_the_server.rs`, `sent_copy.rs`, `receipts.rs` | `src/application/allowed.rs` module doc; CONCERNS "Write paths to real accounts are unproven" |
| IMAP IDLE push events. The plumbing is complete; no real push event has ever arrived | `src/service/protocols/imap.rs` | requirements-backlog: "Status: Done (plumbing)" |
| XOAUTH2 sign-in against a provider, exercised only by the loopback harness | `src/service/protocols/xoauth2.rs`, `src/common/answering.rs` | mail-at-scale status table, step 3 |
| Browser sign-in with Google at scale: capped at 100 hand-added testers, re-authorising weekly, until Google's security assessment passes | `src/service/oauth.rs` | changelog `[Unreleased]` Known limitations, the block at line 7343 |
| Outlook PST/OST import against a real Outlook file | `src/service/outlook_data_file.rs` | changelog line 674: "This has never been run against a real Outlook file" |
| Screen reader experience beyond the interactions the NVDA workflow touches | `.github/workflows/nvda.yml` | IMPLEMENTATION_STATUS: "Most of the application has not had a full manual pass with a screen reader" |

## Not built

### Named in a document as wanted

| Item | Evidence | Source |
|---|---|---|
| Threaded view: collapsing the message list to one row per conversation | `src/presentation/wx_app.rs` line 5026 adds `ID_THREAD_VIEW`, line 583 calls `item.enable(false)` | IMPLEMENTATION_STATUS "What does not work"; roadmap Phase 3 |
| Folder management: create, rename, delete; mark a whole folder read; empty a folder | no `create_folder`, `rename_folder` or `delete_folder` anywhere in `src/` | IMPLEMENTATION_STATUS; ALPHA_TESTING; roadmap Phase 2 |
| Nested folder hierarchy. The tree is one flat level, so `Archive/2026` reads as its full path | `src/presentation/wx_app.rs` folder tree | changelog `[Unreleased]` Known limitations |
| QRESYNC, so a folder resumes rather than re-lists its UIDs | absent from `src/service/protocols/imap.rs` | roadmap Phase 2, unticked |
| Gmail X-GM-THRID conversations and X-GM-RAW server-side search, blocked on the IMAP library | — | roadmap Phase 2; changelog Known limitations |
| Moving a task between lists; move and copy for anything but mail | no move-between-lists path in `src/application/tasks_sync.rs` | IMPLEMENTATION_STATUS; ALPHA_TESTING |
| Holding one connection open. Every body fetch and every attachment save opens its own connection and signs in again | `src/presentation/wx_app.rs` pairs `connect_imap` with `disconnect_imap` at 11819/11852, 11980/12000 and 13008/13034; `src/application/mail_controller.rs` line 278 | changelog Known limitations: "Holding one connection open needs reconnect handling that is not built" |
| Handing an attachment to Windows to open. Deliberate; PDFs are the exception and are read in-app | `src/service/pdf.rs` is the only reader | changelog Known limitations |
| Full PGP encryption and decryption. Detection only | `src/service/security.rs` | requirements-backlog Near-Term, Medium |
| Attachment inline preview for images and text | — | requirements-backlog Near-Term, Medium |
| Folder favourites; smart folders based on rules; spam filtering integration | — | requirements-backlog Near-Term, Low; roadmap Phase 5 |
| Drag-and-drop attachment insertion and inline image insertion | no `DropTarget` or `OnDropFiles` in `src/` | roadmap Phase 4, unticked |
| Spell check while typing, jumping between misspellings, native screen reader announcement. Waits on a rich editor control | `src/application/spell_session.rs` checks on send only | roadmap Phase 4, marked partial |
| Network status detection to toggle offline mode automatically; sync conflict resolution | no connectivity probe in `src/` | roadmap Phase 7, unticked |
| Auto-update mechanism | no update check in `src/`. Desktop and Start menu shortcuts are built: `installer/Wixen-Mail-Setup.iss` declares a `desktopicon` task at line 84 and creates both in `[Icons]` at 123 to 125 | roadmap Phase 8; requirements-backlog Platform, Medium |
| Per-event feedback overrides. They exist in the model with no interface | `src/presentation/accessibility/feedback.rs` | changelog Known limitations |
| Incremental rethreading as mail arrives. Rethreading happens when a folder is opened | `src/application/threading.rs` | changelog Known limitations |
| Junk folder sync. Deliberate; the folder can still be opened | — | changelog Known limitations |
| Whole-mailbox fetch. Check Mail brings the newest 500 per folder, the rest via `Shift+F9` a page at a time | `src/application/mail_sync.rs` | changelog Known limitations |
| Localised dates. Month names and relative wording are English on every machine | `src/presentation/date_display.rs` | changelog Known limitations, line 6946 |
| Calendar recurring-event display across weeks and months. The week and month views themselves do not exist: `src/presentation/wx_calendar_module.rs` lines 46 to 57 disable Prev and Next as "not built yet" | — | changelog line 1448 |
| Notes syncing anywhere. They stay on this computer | notes are cache-only | ALPHA_TESTING |
| Local cache encryption | `src/data/message_cache/mod.rs` | ALPHA_TESTING; CONCERNS |
| A signed installer | `installer/` | ALPHA_TESTING: "The installer is not signed" |
| Three-tier storage split (hot envelope, warm body cache, cold attachments). One budgeted cache exists instead | `src/data/message_cache/bodies.rs` | mail-at-scale: "The hot, warm and cold split in Storage was not built" |
| The Exchange path the plan describes. The Microsoft work went through Graph | `src/service/microsoft_graph.rs` | mail-at-scale: "Exchange proposes a path; the Microsoft work that shipped went through Graph" |
| Exchange Web Services (EWS). Explicitly declined, since Microsoft begins blocking third-party EWS on 1 October 2026 | — | roadmap Future; mail-at-scale "We will not write EWS" |
| CardDAV | `src/service/caldav.rs` covers calendars only | requirements-backlog: "CardDAV not built" |
| JMAP; plugin and extension system | — | requirements-backlog Future, Low |
| Linux and macOS validation | — | roadmap Cross-Platform, unticked |
| The Search "In box" scope selector is read by nothing, so a saved search always covers the whole account | `src/application/search.rs`, `saved_searches.rs` | changelog Saved Searches section; CONCERNS |
| Saving a search over message text that eviction has cleared | `src/data/message_cache/mod.rs` | changelog; CONCERNS |
| Setting Wixen Mail as the actual Windows default mail client. Windows does not allow it | `src/service/default_apps_registration.rs` | changelog lines 870-971 |

### Performance and scale targets never measured

No benchmark harness exists: there is no `benches/` directory and no `criterion`
dependency in `Cargo.toml`, so none of the targets below has a number attached.

| Target | What is measured today | Source |
|---|---|---|
| Memory under 150 MB with 1,000 cached messages | Nothing. No memory profiling has been run | requirements-backlog Performance & Scale, Medium |
| Cold start under 2 seconds | Nothing. Startup time optimisation is unticked | roadmap Phase 8; requirements-backlog |
| A real 100,000+ message mailbox | Nothing. The design targets 200,000 rows; the largest thing exercised is a loopback server | roadmap Phase 8; mail-at-scale title |
| Idle memory under 100 MB | Nothing. Listed as a success metric with no measurement | roadmap Success Metrics |
| Line coverage | 60.4%, measured 2026-07-26 with `cargo llvm-cov --lib --summary-only`, stale since | IMPLEMENTATION_STATUS |
| Test count | 5,430 tests (5,269 unit, 161 integration), counted 2026-08-29 with `cargo test --all-targets -- --list` | Measured against the tree |
| Mutation testing | Scoped runs only: mime and error (2026-07-26); filters, due dates, tagging and signatures (2026-08-01, 157 mutants); the four message-disposition modules (2026-08-12, 66 mutants, 1 survivor). A whole-tree run is about two days and has never been done | IMPLEMENTATION_STATUS |
| Accessibility scanning | Automated scanning covers roughly half of WCAG; five findings at the last read, all inside WebView2's own tree | IMPLEMENTATION_STATUS |

## Unclear

Left unresolved on purpose. Each names the sources and what each one says.

| Question | The disagreement |
|---|---|
| Is there an accessibility testing framework? | `docs/roadmap.md` Phase 1 leaves "Create accessibility testing framework" unticked, and Phase 7 leaves "Accessibility compliance testing with screen readers" unticked. `docs/IMPLEMENTATION_STATUS.md` describes a workflow that drives a real copy of NVDA and checks what it said aloud, plus Axe.Windows and MSAA scans, and `.github/workflows/nvda.yml` and `accessibility.yml` both exist. The tree says a framework exists; the roadmap says it does not. The roadmap may mean the wide manual pass, which IMPLEMENTATION_STATUS separately says has not happened. |
| Is the OAuth local callback server built? | `docs/roadmap.md` Phase 6 leaves "Local callback server for OAuth redirect" unticked. `src/service/oauth.rs` builds `http://localhost:<port>/oauth/callback` at line 330 and serves the request at line 630, and the changelog describes browser sign-in working under Google's tester cap. The code contradicts the tick list. |
| Can a calendar be added by its own address? | `docs/roadmap.md` Future Enhancements says of CalDAV "there is no screen yet for adding a calendar by its own address rather than through an account". The changelog block at line 7047 says the opposite in its closing note, "a calendar can be added by its address", and `src/presentation/wx_add_calendar.rs` exists. The roadmap line looks stale. |
| Is the S/MIME path complete? | `docs/roadmap.md` Phase 5 ticks "S/MIME signature verification" and requirements-backlog says it was built (`src/service/signed_mail.rs`). The changelog at lines 613 to 617 says mail that arrived before that version reads as unsigned, and mail collected over a path that does not keep the original form cannot be verified at all. Built and ticked, with a stated blind spot whose size was not settled here. |
| How much of the message list scales as designed? | `docs/plans/20260726-mail-at-scale.md` says every step in its Sequence is in the tree. Its own earlier note said sorting happened in memory, then closed itself. Nothing measures either claim against a large mailbox, so "built" here is a code-reading result, not a behavioural one. |
| Which document is the debt ledger? | `.planning/codebase/CONCERNS.md` finds only two `TODO`/`FIXME` markers in a 259,723-line tree and concludes the changelog's Known-limitations notes are the real ledger. Those notes are self-amending ("Since closed", "Half closed"), so a limitation's current state can only be read by following its whole entry, not its heading. |
| Are contacts, calendar and tasks writes safe on by default? | `src/application/allowed.rs` ships `personal_information` on for a new install while calling the same code "the least proven code in the application". Both statements sit in one file and are both true; whether the default is right is a decision, and it is not settled anywhere in the documents. |

---

_Inventory compiled 2026-08-29._

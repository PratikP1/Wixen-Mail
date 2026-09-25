# Phase 13 research cross-check

Checked 2026-09-24 against main at 630e2a67, read-only. Every plan writes docs/changelog.md, guards/guards.toml and .planning/WINDOWS.md, so every plan shares a file with every other: one plan per wave, 52 waves.

## Numbering

| Plan | Research id | Title | Group | Wave | Packages |
|---|---|---|---|---|---|
| 13-01 | K4 | Edit, Undo and Redo for the focused text box | 1-keyboard | 1 | - |
| 13-02 | K1 | What a printed page holds, and where it breaks | 1-keyboard | 2 | - |
| 13-03 | K2 | File, Print and Ctrl+P through Windows' print dialog | 1-keyboard | 3 | windows 0.62.2 +3 features |
| 13-04 | K3 | Print from the formatted conversation window and the other five modules | 1-keyboard | 4 | - |
| 13-05 | K5a | Several steps back in the main window's text boxes | 1-keyboard | 5 | - |
| 13-06 | K5b | The same history in every dialog | 1-keyboard | 6 | - |
| 13-07 | K6a | Undo a mark, a star or a label, with the message named | 1-keyboard | 7 | - |
| 13-08 | K6b | Undo a move, a delete to the trash and a copy | 1-keyboard | 8 | - |
| 13-09 | K6c | Undo in contacts, calendar, tasks, notes and reminders | 1-keyboard | 9 | - |
| 13-10 | R2-01 | The invitation said where the reader lands | 2-reading | 10 | - |
| 13-11 | R2-02 | Answer buttons in both reader windows | 2-reading | 11 | - |
| 13-12 | R2-03 | Synced events carry their iCalendar UID | 2-reading | 12 | - |
| 13-13 | R2-04 | Updates and cancellations reach the calendar | 2-reading | 13 | - |
| 13-14 | R2-05 | S/MIME encrypted mail opens through CryptDecryptMessage | 2-reading | 14 | - |
| 13-15 | R2-06 | PGP/MIME opens | 2-reading | 15 | - |
| 13-16 | R2-07 | Keys that fit the credential store, several, and public keys | 2-reading | 16 | - |
| 13-17 | R2-08 | The PGP key manager window on Tools | 2-reading | 17 | - |
| 13-18 | R2-09 | PGP signatures checked and said | 2-reading | 18 | - |
| 13-19 | R2-10a | S/MIME signing and encrypting, service half | 2-reading | 19 | - |
| 13-20 | R2-10b | PGP/MIME signing and encrypting, service half | 2-reading | 20 | rand 0.8 (renamed, already locked) |
| 13-21 | R2-11 | The composer's Sign and Encrypt | 2-reading | 21 | - |
| 13-22 | C1 | Report as Junk | 3-provider | 22 | - |
| 13-23 | AUT-4 | A rule that adds a label puts that label on (moved ahead: runner prerequisite) | 4-automation | 23 | - |
| 13-24 | AUT-6 | One runner for several actions over a set of messages (moved ahead, generic, used by C2) | 4-automation | 24 | - |
| 13-25 | C2 | A block moves the mail already here, with the count; Block keys | 3-provider | 25 | - |
| 13-26 | C3 | Directory sign-in in the credential store, ldap3 #156 fix | 3-provider | 26 | - |
| 13-27 | C4 | The directory sign-in on a screen | 3-provider | 27 | - |
| 13-28 | C5 | Microsoft people search, People.Read on both scope lists | 3-provider | 28 | - |
| 13-29 | C6 | Google as a free/busy source | 3-provider | 29 | - |
| 13-30 | C7 | Every source an account has, answers merged per person | 3-provider | 30 | - |
| 13-31 | C8 | A guest's zone through Windows ICU | 3-provider | 31 | - |
| 13-32 | C9 | Edit Event scrolls at 768px and 200% (only if Pratik says yes) | 3-provider | 32 | - |
| 13-33 | C10 | Identities stored and managed from the account editor | 3-provider | 33 | - |
| 13-34 | C11a | Sending as an identity, data half | 3-provider | 34 | - |
| 13-35 | C11b | Sending as an identity, compose half | 3-provider | 35 | - |
| 13-36 | C12 | A reply goes out from the address it was sent to | 3-provider | 36 | - |
| 13-37 | AUT-1 | Saved searches keep an order somebody chooses | 4-automation | 37 | - |
| 13-38 | AUT-2 | A key per saved search, and the Saved Searches menu | 4-automation | 38 | - |
| 13-39 | AUT-3 | A saved search made from nothing; drifted pages dated | 4-automation | 39 | - |
| 13-40 | AUT-5 | Quick Steps as data | 4-automation | 40 | - |
| 13-41 | AUT-7 | The Quick Step manager on Tools | 4-automation | 41 | - |
| 13-42 | AUT-8 | Quick Steps on the Action menu and their keys | 4-automation | 42 | - |
| 13-43 | AUT-9 | What a rule would change in a folder, counted first | 4-automation | 43 | - |
| 13-44 | AUT-10 | Run a rule over a folder from the Filter Manager and This Folder | 4-automation | 44 | - |
| 13-45 | I1 | One folder out as a bare mailbox file | 5-import-export | 45 | - |
| 13-46 | I2 | A folder out as .eml files | 5-import-export | 46 | - |
| 13-47 | I3 | The .msg reader | 5-import-export | 47 | cfb 0.15.0 |
| 13-48 | I4 | .msg through both import commands | 5-import-export | 48 | - |
| 13-49 | I5 | The pages say which of the three is built and why .pst export is not | 5-import-export | 49 | - |
| 13-50 | I6 | Imported messages keep their files (only if Pratik says yes) | 5-import-export | 50 | - |
| 13-51 | CLOSE | Phase close: scripts/check.sh all by hand, closing read, listening lines | closing | 51 | - |

## Collisions
- Edit menu: K4 moves Undo Send from U to N; K4 edits tests/undo_send_is_where_somebody_looks.rs and its guard record.
- File menu: K2 (Print, P), R2-08 may remove Import PGP Private Key, I1/I2 claim F and X. Re-take the letter grep in each plan after the earlier one lands.
- Tools menu: R2-08 key manager and AUT-7 Quick Step manager both need a letter; RESEARCH-4 says Tools has only Q free. BLOCKER until one takes Q and the other a submenu or a freed letter.
- Action menu (J, Q, Z free): C1 Report Junk J, AUT-8 Quick Steps Q, leaving Z. Any other new Action item needs a submenu.
- Keys: C1 Ctrl+Shift+J, C2 Ctrl+Shift+B, AUT-8 Ctrl+Shift+7..9, AUT-2 Alt+4..9. None free-listed in docs today; no two groups claim the same key. R2-02 Accept cannot use Alt+A (attachments).
- Shared runner: C2 (GAP-06 block) and AUT-6/AUT-10 need the same set runner; AUT-4 (label fix) is its prerequisite. Numbering moves AUT-4 and a generic AUT-6 ahead of C2.
- Schema (message_cache/mod.rs): R2-02, R2-03, R2-06, R2-07, R2-11, C10, C11a, AUT-1, AUT-5. Sequential, all additive.
- Composer (wx_compose.rs): K5b, R2-11, C11b. Account editor: K5b, C4, C10. Item form: K5b, C9. Sequential.
- Credential store 1,280-char limit (RESEARCH-2 finding 1) bears on R2-07, C3's LDAP password (short, fine) and possibly Microsoft OAuth tokens (outside phase; ledger).
- Item undo (K6) is built before junk moves, blocks, Quick Steps and rule runs; whether those are undoable is unplanned.

## Premises checked (all held)
directory.rs:611 bind; directory.rs:653 catch_unwind and no is_ref; export_tree.rs:42-44 header; Cargo.toml:242-253 declined fork; Cargo.toml cms pre-release comment present (RESEARCH-2 says it is wrong); no iCalUID/iCalUId in src; no People.Read in src; what_the_mail_export_did used only in tests; windows has Win32_Globalization and Win32_Security_Cryptography, lacks Gdi, Xps, Controls_Dialogs; Ctrl+Shift+K is Mark Done; acting_on_a_set.rs absent (new file).

## Packages
One PGP choice: pgp 0.20 for GAP-03 and GAP-05. S/MIME via existing windows crate. New lines: windows +3 features (K2), rand 0.8 renamed (R2-10b), cfb 0.15.0 (I3). Skip webview2-com, cms, openssl, sequoia, outlook-pst-rw, compressed-rtf.

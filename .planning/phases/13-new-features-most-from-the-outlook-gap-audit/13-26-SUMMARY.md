---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 26
subsystem: directory lookup
tags: [directory, ldap, credential-store, GAP-07, "#55"]
status: complete
requires: [13-25]
provides:
  - "directory::KEYRING_SERVICE = \"wixen-mail-directory\", keep_the_password(account_id, password), the_saved_password(account_id) -> Result<Option<String>>, forget_the_password(account_id), for 13-27's window"
  - "directory::the_entries_among(results), references and intermediate messages dropped before construct"
  - "a password refused over ldap:// in the lookup's password decision, the address read once into WhereItIs"
  - "forget::entries_for naming the directory password per account; delete_account forgetting it"
  - "finding_people::the_password_to_offer, the lookup reading the saved password"
affects: [13-27, 13-28, 13-51]
tech-stack:
  added: []
  patterns: ["one credential service name per kind of secret, owned by the service module that reads it, named by uninstall and account removal through the same constant"]
key-files:
  created: []
  modified:
    - src/service/directory.rs
    - src/application/forget.rs
    - src/data/message_cache/accounts.rs
    - src/presentation/finding_people.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
    - .planning/phases/13-new-features-most-from-the-outlook-gap-audit/13-27-PLAN.md
decisions:
  - "The password is read only for a directory that signs somebody in, so a store that will not answer never stops a lookup that needs no password."
  - "With a sign-in name and no password over ldap://, the existing no-password sentence is said, not the encryption one; the encryption refusal is for a password that would really be sent."
  - "The store's sentences say \"this account's directory password\" rather than the account id, which is a UUID and is spoken on the trouble line."
  - "An empty password given to keep_the_password forgets rather than keeps, as credentials::store does."
metrics:
  duration: "about 2 hours to the documents commit"
  completed: 2026-09-28
actuals:
  tokens: 9000
  tasks: 3
  commits: 5
---

# Phase 13 Plan 26: The directory password in the credential store, sent over ldaps:// only

A directory password now has one home: the Windows credential store, under
`wixen-mail-directory` with the account id as its user, written and read only through
`service::directory`. Looking a name up reads it for a directory that names somebody to sign
in as and passes it to the bind. It is never sent to an `ldap://` address; the lookup says so
and says to ask for an `ldaps://` one. Uninstalling names it for every account, and removing
an account forgets it before the row goes. A directory's answer is read past continuation
references and intermediate messages before an entry is built. Nothing on screen sets a
sign-in name or a password yet: that is 13-27's window (ledger 693).

## What works, and how it is known

- **The references.** `service::directory::tests`, three rows built from tagged results:
  an entry beside a reference keeps the entry in order and builds without a panic, an
  intermediate message is not an entry, an answer of references holds nobody.
- **The refusal.** Through `look_up_through` and the fake asker: a password over `ldap://`
  is refused with the sentence and the fake is never asked; a directory signing nobody in is
  still asked over `ldap://` and given no password; a password goes over `ldaps://`; a name
  with no password over `ldap://` still gets the no-password sentence.
- **The store.** `service::directory::the_password_kept_for_it`, 4 tests: a round trip under
  the one service and user, a forget, an empty password forgetting, a store refusal as an
  error naming the directory password and not the account id.
- **Uninstall and removal.** `application::forget::tests::test_the_directory_password_is_an_entry_uninstalling_erases`
  and the owners guard, red on arrival when the constant landed and green with the entry;
  the owners reading now names `directory` beside the six. `data::message_cache::accounts::tests::test_deleting_an_account_takes_its_directory_password_with_it`.
- **The lookup.** `presentation::finding_people::the_password_offered`, 3 tests: nothing
  read for a directory signing nobody in even with the store refusing, the saved password
  offered, a refusal said with "The directory was not asked." `the_organisation` calls it and
  passes the password to `directory::look_up`.
- **Counts, re-taken 2026-09-28:** `service::directory::` 53 (42 before), `application::forget::`
  26 (25 before; the plan said 24, 13-16 had added one), `data::message_cache::accounts::` 23
  (22 before), `presentation::finding_people::` 5 (2 before), `tests/finding_people_answers.rs` 1.
- **Reachability.** `finding_people::through` -> `who_matches` -> `the_organisation` ->
  `the_password_to_offer` -> `directory::the_saved_password`, then `directory::look_up` ->
  `the_password_to_sign_in_with` -> `TheDirectoryItself::ask` -> `the_entries_among`. The
  password half is reached only for an account whose stored settings name a sign-in, and no
  screen writes one until 13-27 (ledger 693). `keep_the_password` has no caller outside tests.
- **Not proved:** a real directory with a sign-in, over either scheme, and a search from an
  Active Directory domain root (ledger 695).

## Commits

| Commit | What | Hook |
|---|---|---|
| `8a10eae7` | red: 3 reference rows, the ldap:// refusal, 3 companions green | 120 s |
| `9554ab02` | feat: `the_entries_among`, `WhereItIs`, the refusal; 2 records new | 215 s |
| `20a8e8df` | red: the store, the uninstall entry, the delete, the lookup's reading, the count check | 124 s |
| `f9187a8f` | feat: the store, the entry, the delete, the reading; 2 records new, 6 re-measured | 203 s |

The documents commit follows, then the pull request, then the merge.

## Guard records

| Record | Break | Red |
|---|---|---|
| a reference among a directory's answers is dropped before an entry is built (new) | `is_ref()` taken out of the filter | the reference-beside-an-entry row and the only-references row |
| a directory password is never sent over an unencrypted address (new) | the encryption arm made `if false` | the ldap:// refusal case |
| uninstalling names every account's directory password (new) | the directory push taken out of `entries_for` | the entry case and the owners guard |
| removing an account forgets its directory password (new) | the forget call replaced by `Ok(())` | the delete case |

Re-measured: the two records on `accounts.rs` (22 to 23 tests), the two naming `forget.rs`
(25 to 26), and the two new directory records again after the file went from 49 to 53.
Two `--remeasure` calls, one per green commit: 2 records in 280 s, 8 in about 1,000 s, every
one reddening exactly what it names. Anchors inside the files this plan edits were read
before editing: the `forget.rs` record anchors on the `pgp::keyring_entries()` loop and the
`accounts.rs` records on `if !still_stored.is_empty() {` and the `in_the_row` check, none of
which moved. The arrived-since count went from 440 to 444.

## Documents

- Privacy page: the directory row says a sign-in name and password go only over
  `ldaps://`; a paragraph says where the password is kept, that `ldap://` is refused and
  why, that removing the account and uninstalling erase it, and that nothing on screen sets
  one yet.
- Changelog, `[Unreleased]`, Added: the password kept and used over encrypted addresses,
  names ldap3 #156 and says plainly that today's search already had references skipped
  inside the library, with the limitations. No version bump: nothing is reachable from a
  screen yet, and no build has been cut since 1.0.0-alpha.1.
- 13-27's premise 1 names ledger 693 and adds that nothing writes `sign_in_as` either.

## Ledger

Opened, both halves: 693 (`todo`, nothing outside tests writes a directory password or a
sign-in name until 13-27, named in 13-27's premise 1), 694 (`todo`, signing in as the
Windows user through wldap32 or ldap3's gssapi, and ldap3 #157 watched), 695
(`unrun-verify`, phase 14: a real directory with a sign-in, the ldap:// refusal, a search
from an Active Directory domain root, and Credential Manager empty after removal), 696
(`todo`, found and left: `credentials.rs` names the account's UUID in a sentence that is
spoken). Counts 696 total, 626 open, 70 fixed, 0 waived.

## Deviations from Plan

**1. [Premise] ldap3 #156 never reached this tree's search.** The research and the plan
said the panic catch turned every Active Directory search from the domain root into a
failure that discarded the entries. Reading ldap3 0.12.1: `Ldap::search`
(`ldap.rs:526-542`) runs through the `EntriesOnly` adapter (`adapters.rs:262-279`), which
drops references and intermediate messages before the caller sees a result. The panic is
real for `streaming_search` without adapters, which is what the reporter used. The filter
was built as planned, because a paged or streamed search later would lose that protection,
and the comment says so; the changelog carries no Fixed line claiming a repair nobody could
have noticed.

**2. [Rule 2] The store's sentences do not name the account id.** The plan's shape
followed `credentials.rs`, whose sentences name the account id. The id is a UUID and this
sentence reaches the spoken trouble line, so it says "this account's directory password".
The red case had asserted the id was named; the green asserts it is not. That assertion
came after the `expect_err` the stub failed, so the red was not weakened. The same wording
in `credentials.rs` is ledger 696.

**3. [Shape] The lookup's reading is `finding_people::the_password_to_offer`**, split out of
`the_organisation` so its three cases run against the store double, and read only for a
directory that names a sign-in. The plan named a test for every other part and none for
this one.

**4. [Shape] The no-password sentence wins over the encryption one** for a sign-in name with
no password over `ldap://`, as the plan's "a name with no password is refused as today"
reads; a companion case holds it.

**5. [Premise] Numbers re-taken.** `forget.rs` held 25 tests, not 24, and two records by
`tests_last_seen`, not one: 13-16 added a test and the record "uninstalling names the last
part of every slot a private key can occupy". `secret_store.rs`'s `write`, `read` and
`remove` are at `:215`, `:225` and `:233`, moved by 13-16's honest store. The rest held.

**6. [Brief] Read-only Python** printed long lines of `STATE.md` and `REQUIREMENTS.md` and
checked `guards.toml` parses; nothing was written by it. No `sed` or `awk`. `cargo fmt`
formatted the Rust files.

### Found and left

- Ledger 696: `credentials.rs` speaks the account's UUID when a password will not go.
- The store's sentence carries "Security error:" twice when the cause is itself a
  `Security` error, the same as `credentials.rs`; it reads oddly and says nothing false.

## Threat Flags

None beyond the register. T-13-26-01: refused before any connection, a case reading the fake
was never asked, a record. T-13-26-02: references and intermediate messages dropped before
`construct`, the catch kept, rows and a record. T-13-26-03: named in `entries_for` (red on
arrival), forgotten in `delete_account` before the row, cases and two records. T-13-26-04:
written only through `secret_store` under one owner, never logged, the sentences naming
neither the password nor the account id. T-13-26-SC: no crate added.

## Known Stubs

None standing for finished work. `keep_the_password` has no caller outside tests until
13-27's window, which is the plan's boundary and ledger 693.

## Self-Check: PASSED

The four commits above are on the branch; the files modified exist.

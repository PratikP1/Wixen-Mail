---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 33
subsystem: identities
tags: [identities, account-manager, schema, GAP-10, "#59"]
status: complete
requires: [13-32]
provides:
  - "CREATE TABLE IF NOT EXISTS identities, and MessageCache::add_identity, identities_for, remove_identity, put_identities_in_order, keep_the_identities, clear_identities and is_a_stored_account"
  - "application::identities: Identity, what_stops_an_address_being_kept -> Option<Refused { said, at: TheBox }>, the_others, FromEntry, the_from_list, who_it_goes_out_from, moved, LONGEST_SENDER_NAME, NOT_TRIED_WITH_A_PROVIDER, NOT_SAVED_YET, NOWHERE_TO_KEEP_THEM"
  - "presentation::wx_identities: build_identity_manager, show_identity_manager, populate_identities, build_address_window, wire_the_address_window"
  - "wx_account_manager::other_addresses_for, the Account Manager's &Other Addresses to Send From... on Alt+O"
  - "ScanTarget::Identities, identities, on the Accessibility workflow's list"
affects: [13-34, 13-35, 13-36, 13-51]
tech-stack:
  added: []
  patterns: ["a write that is whole on its own and a step of a larger one joins the transaction already open (all_or_nothing)", "a manager read from inside its own modal loop by a pending callback that presses its buttons"]
key-files:
  created:
    - src/application/identities.rs
    - src/data/message_cache/identities.rs
    - src/presentation/wx_identities.rs
    - tests/other_addresses_are_managed_per_account.rs
  modified:
    - src/application/mod.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/accounts.rs
    - src/presentation/mod.rs
    - src/presentation/wx_managers.rs
    - src/presentation/manager_words.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_app.rs
    - src/presentation/scan_target.rs
    - .github/workflows/accessibility.yml
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The manager writes everything it holds when it closes, through keep_the_identities in one transaction; a changed row is taken away and written again, and every row going goes before any is written, so two addresses swapped in one visit never meet the table's uniqueness. There is no replace_identity."
  - "Whether an account is saved is asked of the store (is_a_stored_account), not inferred from the list the Account Manager opened with."
  - "The Account Manager takes the store as Option<&Arc<MessageCache>> so the button can answer its own click, the way Look People Up at Work does, rather than going through the loop's end_modal."
  - "An address must pass links_in_text::is_an_address, hold an @, and hold no space, control character or any of : < > , ; \" ( ) [ ] \\; a name is refused past 100 characters or with a control character."
  - "The sentence that no provider has been asked is a line of its own above the list, not the status line, which every answer overwrites."
  - "An other address kept with no name goes out with none (who_it_goes_out_from), not under the account's own name."
metrics:
  duration: "about 3 hours 15 minutes to the documents commit, of which about 90 minutes waited on a locked Windows session"
  completed: 2026-09-29
actuals:
  tokens: 26000
  tasks: 3
  commits: 7
---

# Phase 13 Plan 33: Other addresses to send from, kept per account and managed from the Account Manager

An account can now hold other addresses to send from, each with the name people see, in an
order the person chooses. They live in a new `identities` table added with `CREATE TABLE IF
NOT EXISTS` beside the saved searches, touching no other table, and they go when their account
is removed. The Account Manager has **Other Addresses to Send From...** on `Alt+O`, beside Look
People Up at Work. For a saved account it opens a manager on the loop every manager uses (Add,
Edit, Delete, Move Up, Move Down, Close), whose address window holds Address on `Alt+A` and The
name people see on `Alt+N`. OK refuses, with a sentence and focus on the box it is about, what
is not an address, the account's own address, one it already holds in any case, and a name
over 100 characters or with a control character. The list is written when the manager closes.
With no account chosen the button says what Edit says; an account added in the same visit is
refused with "This account is saved when the Account Manager closes. Close it, open it again,
and then add other addresses." A line above the list says a provider may refuse or replace an
address and that none has been asked. Compose does not offer them yet: that is 13-35 (ledger
713).

## What works, and how it is known

- **The store.** `data::message_cache::identities::tests`, 10: an address comes back with its
  name; each added goes last; the order written is the order read; remove says whether there
  was one; one account cannot hold an address twice in any case, and two accounts may each hold
  it; what the manager holds is kept in its order with a row removed, one renamed and one added;
  two addresses swapped in one visit are both kept; an identity goes with its account; an
  account is stored once saved; a database without the table opens, keeps its account and
  takes an address.
- **The rules and the From list.** `application::identities::tests`, 12: a good address kept;
  ten shapes refused as not an address, a header injection among them; the account's own
  refused in any case and with spaces; one already held refused in any case; the name's bound
  at 100 and 101; a line break or tab in the name refused at the name box; the_others leaves
  out only the row being edited; the From list is each account then its other addresses, said
  "help@example.com, another address on Work"; an account with no label is named by its
  address; who_it_goes_out_from with nothing given and with an address and name given; a move
  says where the address went.
- **The windows, read in a built process.** `tests/other_addresses_are_managed_per_account.rs`,
  16 readings on a desktop of the process's own: the Account Manager's O held by the button
  alone, with a companion planting a second O; the manager titled for its account with the
  columns Address and Name people see and the rows in order; a move down by
  `move_the_chosen_row`; the manager opened through `other_addresses_for` and read from inside
  its modal loop, its letters A, C, D, E, U, W once each, its list and six buttons named on
  MSAA in tab order, the untried line shown, a Move Down pressed and Close pressed, and the
  store's order and the Account Manager's "The other addresses Work sends from are kept."
  read after; the address window's letters A and N, with a companion planting a second A;
  focus on Address, named "Address," on MSAA at its handle; both fields and both buttons named
  in tab order; an address that is not one refused with the window open, the sentence shown
  and focus on Address; a good one closing with OK; no account chosen; an unsaved account
  refused with no manager opened.
- **Counts, re-taken 2026-09-29:** `data::message_cache::identities::` 10 (the plan asked at
  least 8); `application::identities::` 12 (at least 10); `data::message_cache::accounts::`
  23, unchanged; the new target 16 (at least 8); `presentation::wx_managers::` 44, unchanged;
  `presentation::scan_target::` 11; `presentation::wx_account_manager::` 14;
  `presentation::manager_words::` 8; `ScanTarget::ALL` 44 (43 before).
- **Acceptance greps:** `CREATE TABLE IF NOT EXISTS identities` 1 in `mod.rs`;
  `clear_identities` 1 in `accounts.rs`; `&Other Addresses to Send From...` 1 in
  `wx_account_manager.rs`; `'identities'` 1 in the workflow.
- **Reachability.** Tools, Account Manager (`Ctrl+Shift+A`) -> `handle_account_mgr` ->
  `show_account_manager_dialog(.., cache.as_ref())` -> `wire_account_manager_actions` ->
  the `other_addresses` button -> `other_addresses_for` -> `is_a_stored_account`,
  `identities_for` -> `show_identity_manager` -> `run_manager_loop` -> Add or Edit ->
  `ask_for_an_address` -> `wire_the_address_window` -> `what_stops_an_address_being_kept`;
  Close -> `keep_the_identities`. Removing an account: `delete_account` -> `clear_identities`.
  `the_from_list` and `who_it_goes_out_from` have no caller outside tests until 13-35 (ledger
  713).
- **Not proved here:** the manager heard (ledger 715); any provider sending from an other
  address, which is 13-35's and phase 14's. The Accessibility scan of the new target is read on
  the pull request.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `aa563bd5` | test: 21 failing cases for the rules, the From list and the table, on stubs | red, 186 s |
| `5db934d2` | feat: the table, the data module, the clear call and the rules; 2 records | affected, 296 s |
| `0f96b286` | test: 16 failing readings on bare stubs | red, 246 s |
| `ce9e9d97` | feat: the manager, the window, the button, the scan target, the shortcuts page; 2 records | all (the workflow line), 706 s; refused once before at 607 s |
| `31aadc9a` | docs: the changelog, the ledger, this summary, the marks | docs only, 106 s |
| `9cc7d053` | test: an address with no name says "No name" in its row, red (CI's scan) | red, 81 s |
| green | the name column says "No name" when an address has none; this deviation | this commit |

## Guard records

| Record | Break | Red |
|---|---|---|
| an account's other addresses go when the account does | `clear_identities`' delete matches nothing | `test_an_identity_goes_with_its_account` |
| an other address is compared with the ones an account holds without case | `the_same_address` compares with case | the own-address and already-held cases |
| the other address window keeps nothing its rules refuse (suite the new target) | OK closes whatever the rules answer | the refused-address reading |
| other addresses are refused for an account the store does not hold yet (suite the new target) | an unsaved account's addresses read and the manager opened | the unsaved-account reading |

Each reddens exactly what it names. Three `--remeasure` calls: 2 records in 256 s at task 1's
green, then 2 in 26 s whose run stopped because one break did not compile, then that one alone
in 42 s. No existing record was flagged by the count check: `mod.rs`, `accounts.rs`,
`wx_managers.rs`, `wx_account_manager.rs`, `scan_target.rs` and `manager_words.rs` kept their
test counts, and no anchor lay in a changed line (the workflow's records anchor on the
`which-days` and `contact-editor` lines, `scan_target.rs`'s on the `SignatureEditor` name, and
`wx_managers.rs`'s make_shell record on the lines after the signature). The arrived-since count
went from 464 to 468.

## Deviations from Plan

**1. [Premise] No bound on an account's name existed.** The plan refused "a name longer than
the account name's bound"; nothing bounds `Account::name` or `sender_name`. The bound is
`LONGEST_SENDER_NAME`, 100, the saved search name's.

**2. [Shape] No `replace_identity`; the manager writes through `keep_the_identities`.** A
replace per row would trip the table's uniqueness when two addresses swap in one visit, and a
method nothing in the program calls is dead code. `add_identity`, `remove_identity` and
`put_identities_in_order` are the steps it takes, in one transaction; `put_identities_in_order`
joins the transaction already open (`all_or_nothing`). `is_a_stored_account` was added for the
unsaved-account refusal.

**3. [Rule 2] The address and the name refuse what could break a From header.**
`is_an_address` alone passes `a\r\nX: y@b.c` and anything with `www.` in front; the rule also
requires an `@` and refuses whitespace, control characters and `: < > , ; " ( ) [ ] \`, as
`an_address_to_add` in the contact editor refuses a space and a colon. A name with a control
character is refused at the name box (T-13-33-02).

**4. [Premise] `make_shell` was already `pub`.** Only its parent was widened, from `&Frame` to
`&dyn WxWidget`; `run_manager_loop`, `ManagerChrome` and its fields, and
`nothing_stops_this_closing` became `pub(crate)`.

**5. [Rule 3] The Account Manager's store parameter became `Option<&Arc<MessageCache>>`**, so
the button's handler can hold it; `wx_app.rs`'s two callers pass `cache.as_ref()`, and
`wx_app.rs` gained the scan target's arm the exhaustive match needs. `manager_words` gained
the kind `ADDRESS`, so the loop says "Added the address: ...", and it joined `EVERY_KIND`.

**6. [Shape] The untried sentence is a line above the list**, not the status line, which the
first answer overwrites.

**7. [Expectations] The running manager's MSAA reading leaves out the list's column
headings.** `EnumChildWindows` reaches the list's `SysHeader32` child, named "Header Control";
it takes no focus, as the size grip already left out does not. The green changed that filter
and nothing else in the target.

**8. [Environment] The green of task 2 was refused once by the full gate, not by the change.**
The workflow line earns `all`, and `a_locked_key_asks_for_its_passphrase` failed six cases with
"OpenClipboard failed". OpenClipboard answered error 5 on the interactive and on a new window
station alike, in a throwaway program too, while LockApp and LogonUI ran: the session was
locked. The commit waited about ninety minutes for the unlock and then passed at 706 s; the
target passed alone first. Ledger 716 records it; 13-25 had read it as a flake.

**9. [Rule 3] One guard break did not compile.** The first address-window break used a match
guard, which left the match non-exhaustive; the record's `after` was rewritten to an
unconditional arm with `#[allow(unreachable_patterns)]` on the one after it, and measured.
That is why there were three `--remeasure` calls rather than one per green.

**10. [TDD] The red of task 1 names 21 of its 22 cases.** The good-address case passes against
a stub that refuses nothing; it is a companion to the refusals and was not named.

**11. [Brief] Five diagnostic commands carried do-nothing shell assignments named after a
banned tool.** None ran it and nothing was written by them. Read-only Python parsed
`guards.toml`; `cargo fmt` formatted the Rust files; the one probe of a tracked test (the
error code added to `a_locked_key_asks_for_its_passphrase`'s message) was made with Edit and
put back with `git checkout --` before any commit.

**12. [Rule 1, after CI] An address kept with no name left an empty cell, which the scan
found.** The Accessibility run on pull request 135 (36533332611) read the `identities` target
with one violation: the Name people see cell of the fixture's second address, empty, so
nameless on UI Automation. `accounts`' one violation, an empty IMAP Server cell, was there
before this plan. The row now says "No name" there, a red and green pair after the documents
commit, which the brief allows when CI asks for a fix.

## Ledger

Opened, both halves: 713 (`todo`, not offered in compose until 13-35, named in its premise 1),
714 (`todo`, Gmail's own Send mail as list, decision 34), 715 (`unrun-verify`, the tester's
ear), 716 (`todo`, the clipboard reading in a locked session). Closed: none. Counts 716 in all,
643 open, 73 fixed, 0 waived.

## Threat Flags

None beyond the register. T-13-33-01: the manager says where it is read that a provider may
refuse or replace an address; phase 14 reads a real provider. T-13-33-02: refused at OK by
`what_stops_an_address_being_kept`, ten shapes and the name's control characters as rows, and a
guard record on the window. T-13-33-03: cleared in `delete_account`, a case and a record.
T-13-33-SC: no crate added.

## Known Stubs

None. `the_from_list` and `who_it_goes_out_from` are built for 13-35 and have no caller
outside tests, which ledger 713 records.

## Self-Check: PASSED

The commits above are on the branch; the created files exist.

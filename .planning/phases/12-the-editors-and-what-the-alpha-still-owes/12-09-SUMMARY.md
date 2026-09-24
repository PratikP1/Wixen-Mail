---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 9
subsystem: signatures, the account editor, the Signature Manager and the composer
tags: [edit-04, "#43", signatures, account-editor, signature-manager, compose, ledger-604]
status: complete
requires:
  - phase: 12
    provides: "12-08 merged at ab3e14c3; ledger 604, which 12-08 opened"
provides:
  - "src/application/signatures.rs: which_signature and whether_to_swap with Swap; 13 cases"
  - "src/data/message_cache/signatures.rs: get_every_signature, assign, assignment_for, set_the_default, signature_for_account, make_signatures_one_set under SIGNATURES_ARE_ONE_SET; 11 cases"
  - "src/data/message_cache/mod.rs: the signature_assignments table, and the pass run on open"
  - "src/presentation/wx_managers.rs: SignatureAccount, SignatureEntry.used_by, ManagedRow::settle, build_signature_manager with a Used by column, AccountOffer, offers_for, the account boxes under Use for these accounts, the_signature_as_edited"
  - "src/presentation/managers.rs: the_signature_managers_rows, and a save that writes rows, then the default through set_the_default, then each account through assign"
  - "src/presentation/wx_account_manager.rs: SignatureChoices, Signature for this account on the first page, keep_the_signature_choice"
  - "src/presentation/wx_compose.rs: SignatureFor, follow_the_from_account, one signature per From account"
  - "src/presentation/wx_item_form.rs: Prefill.is_new, so a new item is headed New whatever it opens with (ledger 604)"
  - "tests/a_signature_follows_the_from_account.rs: 16 readings; tests/a_new_item_is_headed_new.rs: 3"
  - "the account-details scan target, the account editor's first page"
affects: [12-10 and 12-11, which change managers or Settings next; 12-12, which runs the phase's full gate and reads the pages]
tech-stack:
  added: []
  patterns:
    - "A block the composer put into its page is found again by rendering it the way it went in and matching the page's own markup, not by reading the page's text"
    - "A manager row that is about its neighbours settles them after an edit, through ManagedRow::settle"
key-files:
  created:
    - src/application/signatures.rs
    - tests/a_signature_follows_the_from_account.rs
    - tests/a_new_item_is_headed_new.rs
  modified:
    - src/application/mod.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/signatures.rs
    - src/presentation/editor_document.rs
    - src/presentation/managers.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/scan_target.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_app.rs
    - src/presentation/wx_compose.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/wx_managers.rs
    - .github/workflows/accessibility.yml
    - tests/account_edit_protocol_fields.rs
    - tests/checkbox_labels.rs
    - tests/every_spin_control_names_the_field_a_person_types_in.rs
    - tests/item_form_prefill.rs
    - tests/theme_reach.rs
    - tests/wired.rs
    - tests/a_snippet_is_the_first_relevant_words.rs
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "whether_to_swap compares the block as it went into the page, character for character and as whole lines, not the text sign_off::split finds: the page holds the signature rendered from Markdown, and above a quote, where split's last separator is the quoted message's."
  - "The block is looked for in both forms the composer writes, as markup above a quote and as escaped text in a new message, and only above the quote marker."
  - "The swapped message is sanitised before it goes back into the page, the same as anything else put there."
  - "A message that opened with no signature does not gain one on a From change: the rule's no-block arm, as planned."
  - "The manager holds a working copy of who uses what between Add and Close, as it does for every other field, and writes it through assign on Close; the store is what either surface reads."
  - "The account editor's first page became a scan target, account-details, so the choice's name is read in the running program."
requirements-completed: [EDIT-04]
duration: about 300 min
completed: 2026-09-24
estimate:
  tokens: 33600
actuals:
  tokens: 75000
  tasks: 3
  commits: 7
---

# Phase 12 Plan 09: Signatures per Account Summary

**Signatures are one set. An account signs with the signature assigned to it, else the default, else none. The choice is made on the account's own dialog, Signature for this account (Alt+F), or in a signature's editor in the Signature Manager, a box per account under Use for these accounts (Alt+U); both write the store's one assignment and each shows what the other chose. The manager lists every signature with a Used by column and sets or clears the default. Compose opens with the From account's signature and replaces an untouched block when From changes, saying so, and leaves a block somebody typed into. A database from an earlier build keeps every account's signature through a once-only pass. Ledger 604 is fixed on the same branch: a new event or reminder is headed New again. All of it is read on built dialogs and the real composer page; none of it has been heard with a screen reader, and the pull request's scan and NVDA runs came after this was written.**

## Tests

| Target or filter | Tests | Result |
|------------------|-------|--------|
| `cargo test --lib application::signatures::` | 13 | pass |
| `cargo test --lib data::message_cache::signatures::` | 11 | pass |
| `cargo test --test a_signature_follows_the_from_account` | 16 | pass |
| `cargo test --test a_new_item_is_headed_new` | 3 | pass |
| `cargo test --lib presentation::managers::` | 137 | pass, unchanged count |
| `cargo test --lib presentation::wx_compose::` | 43 | pass, unchanged count |
| `cargo test --lib presentation::wx_account_manager::` | 14 | pass, unchanged count |
| `cargo test --lib presentation::wx_managers::` | 44 | pass, unchanged count |
| `cargo test --lib presentation::wx_app::` | 199 | pass, unchanged count |
| `cargo test --test wired`, including the Alt-key collision check | 77 | pass |
| `cargo test --test a_key_is_documented_where_the_surface_that_binds_it_is` | 3 | pass |

The green commit of task 2 ran the full gate, because it changed a workflow: `check.sh: all passed after 508 s`.

The composer readings run on the real editor page. After From changed from Work to Home, a new message's page read `<br><br>-- <br>Cheers, Home` where it had read `-- <br>Regards, Work`, and a reply's read the markup block with Home's signature above `--- Original Message ---` and the quoted words kept. The companions hand each check a wrong state (a swap over an edited block, one account's rows, a dialog that did not read the manager's write) and are refused.

## Commits

| Commit | What | Hook |
|--------|------|------|
| `1bfc83df` | test: the rule's and the store's cases, stubs | red, 111 s |
| `e6814ee8` | feat: one set, the table, the pass, the readers | affected, 154 s |
| `a954d93d` | test: a new event's heading with a default reminder (ledger 604) | red, 121 s |
| `0f310521` | fix: the heading reads is_new | affected, 154 s |
| `c8af4bc3` | test: every surface, shapes built and behaviour stubbed, the Alt keys on the shortcuts page | red, 236 s |
| `5e935ae7` | feat: the manager, the account dialog, compose following From, the scan target | all, 420 s refused, then 510 s |
| this commit | docs: the guide, the changelog, the ledger, this summary, the four marks | |

## Guard records

Six written, all measured with `scripts/guards.sh --remeasure`, each reddening exactly the tests named; the arrived-since count went 278 to 284.

| Record | Red |
|--------|-----|
| an account's assigned signature is used before the default for everyone | 3 |
| the pass that makes signatures one set gives each account the default it had | 2 |
| a new event opened with the default reminder is headed New Event | 2 |
| a change of From account leaves a signature somebody typed into as it is | 1 |
| the signature manager lists every signature whichever account wrote it | 2 |
| an account ticked in a signature's editor is written through the one assignment | 2 |

One re-anchored: "forwarding takes the original's pictures with it", since the quote markers became constants; re-measured, its one test red and nothing else.

## Ledger

Opened, both halves: 605, #43 under NVDA; 606, the account editor's seven letters that more than one label claims (B, I, M, N, P, S, T), none added by this plan. Closed: 604. The front matter reads 606 entries, 550 open and 56 fixed.

## Deviations

1. **Ledger 604, on this branch as its own pair (the brief's addition).** `Prefill` gained `is_new`; the New command sets it for Settings' default alert; the heading reads it. Red at `a954d93d`, green at `0f310521`, a record on the heading.
2. **[Rule 1] The swap does not use `sign_off::split`.** The plan's rule found the block with `split` and compared it with the stored signature text. The page holds the signature rendered from Markdown, so the text never matches a formatted signature, and a reply puts it above the quote, so the last separator is the quoted message's. `whether_to_swap` takes the block as it went in and matches it as whole lines in the page's markup, above the quote marker, in both forms the composer writes. The real page confirmed both.
3. **[Rule 2] A scan target for the account editor's first page, `account-details`.** `account-editor` turns to the second page before the scan, so the choice was never walked. The workflow change made the task 2 green commit run the full gate.
4. **[Rule 1] `tests/a_snippet_is_the_first_relevant_words.rs` counted every row of `work_done_once`.** The signatures' pass added a third row; the test counts the snippet passes' rows by name now. Task 1's scoped gate did not reach the target; task 2's full run did, and refused the commit once.
5. **[Rule 3] `ListCtrl::get_item_text` drops the last character of every cell** (wxdragon 0.9.17, `src/widgets/list_ctrl.rs:429`, already measured in `tests/manager_dialog_labels.rs`). The new target reads the manager's cells with `LVM_GETITEMTEXTW`. Upstream defect, not filed here.
6. **The manager loop.** `run_manager_loop` hands a row's editor every row and calls `ManagedRow::settle` after an edit, so an account given to one signature is taken from the others and the default moves at once rather than at Close. The other three managers settle nothing.
7. **`update_signature` no longer writes `is_default`**, and `create_signature` clears the mark on every other row rather than the same account's: `set_the_default` is the one writer. `get_default_signature` had no reader left and is gone.
8. **Two source-reading checks in `tests/wired.rs`** named the old compose lines; they read the per-account signatures now.
9. **Two `wx_item_form::Prefill` literals in tests** gained `is_new: false`, and four callers of `build_account_edit_dialog` and two of `build_sig_edit_dialog` in tests gained the new argument. `tests/checkbox_labels.rs` reads the new account box too.

## Known stubs

None.

## Threat flags

None beyond the plan's register. The swapped message is sanitised before it goes back into the page (T-12-32's surface).

## What only a person can settle

Ledger 605: the Used by column, the choice and its first entry, the account boxes and their wording, and "Signature changed to" on a From change, heard with NVDA.

## Self-Check: PASSED

The three created files and every commit above exist on the branch (`git cat-file -e`). `grep -c 'Signature &for this account' src/presentation/wx_account_manager.rs` reads 1. `grep -c 'Si&gnature'` reads 1, not the plan's 0: the one line is the doc comment on `SIGNATURE_FOR_THIS_ACCOUNT` saying why the letter is F and not that G, which names the label it replaces rather than building it; the comment was kept and the criterion read as about labels; `grep -c '&Use for these accounts' src/presentation/wx_managers.rs` reads 1; `get_default_signature(&id)` is not in `wx_app.rs`. No crate was added (T-12-SC).

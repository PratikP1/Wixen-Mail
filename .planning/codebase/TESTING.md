# Testing Patterns

**Analysis Date:** 2026-08-29

## Test Framework

**Runner:** `cargo test`, standard Rust built-in test harness. No custom test runner.

**Async:** `tokio-test = "0.4"` (dev-dependency) — see `tokio_test::block_on(...)` used in `src/application/mail_controller.rs`.

**Filesystem isolation:** `tempfile` crate — `tempfile::tempdir()` used wherever a test needs a real filesystem path instead of the user's real directories (`src/application/attaching.rs`, `src/application/mail_sync.rs`, `tests/integration_tests.rs`).

**Run commands:**
```bash
bash scripts/check.sh                          # fmt + clippy -D warnings + tests + release build, the CI gate
cargo test --all-targets --no-fail-fast         # what check.sh runs for the test step
cargo llvm-cov --lib --summary-only             # coverage, the "what never runs at all" sweep
scripts/mutants.sh src/service                  # mutation testing, one directory
scripts/mutants.sh --since v0.19.0              # mutation testing, only what changed since a real tag/commit
scripts/guards.sh                                # re-measure every recorded guard (slow: one build per guard)
scripts/guards.sh deletion                       # only guards whose names match "deletion"
```

`git config core.hooksPath .githooks` makes every commit run these checks; `--no-verify` is reserved for WIP branches nobody builds, never `main`.

**Important operational rule (`--no-fail-fast`):** without it, cargo stops at the first failing target, and the library is always the first target built, so a single failing lib test silently prevents all fourteen-plus files under `tests/` from running at all — not reported as skipped, never started. `scripts/check.sh` always passes `--no-fail-fast`; do the same in ad hoc runs.

## Test File Organization

**Unit tests:** co-located, `#[cfg(test)] mod tests { ... }` at the bottom of the file they cover. 203 files across `src/` follow this pattern (e.g. `src/application/accounts.rs:94`, `src/application/allowed.rs:205`, `src/application/answering.rs:795`).

**Cross-layer / integration tests:** `tests/integration_tests.rs`, importing the crate as a library (`use wixen_mail::application::...`) and exercising real storage via `tempfile::tempdir()` + `MessageCache::new(...)` rather than in-memory doubles. Comment at `tests/integration_tests.rs:56` explicitly documents replacing an in-memory-only test with one that goes through real storage, because the in-memory path "nothing in the application ever reached."

**House-rule / structural tests:** a distinct category of test file that reads source text (not runtime state) to enforce conventions and wiring:
- `tests/house_style.rs` — banned em/en dash characters project-wide; guard-record hygiene (`test_every_guard_record_still_names_one_place_in_the_tree`, `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`); "a control no screen writes" phrase checks against documentation over-promising features.
- `tests/wired.rs` — every handled command id (`ID_...`) has something in `src/presentation/` that actually raises it (menu item, toolbar tool, context menu, tray command), catching commands that compile and pass tests but that nothing in the running UI ever sends.
- `tests/checkbox_labels.rs`, `tests/flag_names.rs`, `tests/theme_reach.rs`, `tests/docs_links.rs`, `tests/command_line_output.rs` — narrower instances of the same family: read source or docs text and assert a specific promise (a label, a flag name, a theme color, a documented link, a CLI message) actually holds in the tree.

**Behavior-scoped feature tests:** files named for the UI surface or workflow they exercise, not the module: `tests/account_edit_protocol_fields.rs`, `tests/account_manager_immediate_actions.rs`, `tests/calendar_immediate_actions.rs`, `tests/item_form_date_time_fields.rs`, `tests/item_form_free_busy.rs`, `tests/item_form_prefill.rs`, `tests/item_form_recurrence_tab.rs`, `tests/item_form_validation.rs`, `tests/manager_delete_stays_open.rs`, `tests/text_selection_offsets.rs`, `tests/tree_selection_raises.rs`, `tests/finding_people_answers.rs`. Each of these tends to read `src/presentation/` source text directly (window-construction and account/mail-server dependencies make full runtime UI tests impractical), and each documents, in its own module doc, exactly what it is blind to as a source-reading check — follow that pattern for any new file in this family: state the blind spot in the module doc, do not just assert green.

## Test Structure

Typical unit test module:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_group_lives_in_storage_and_resolves_to_a_to_line() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MessageCache::new(dir.path().join("groups.db"), None).unwrap();
        // ...
    }
}
```

**Naming:** test function names are full sentences describing the behavior under test, not `test_foo_case1` style — e.g. `test_a_group_lives_in_storage_and_resolves_to_a_to_line`, `test_every_handled_command_has_something_that_raises_it`, `test_no_dashes_that_should_be_punctuation`. Follow this naming convention for new tests: the name alone should tell a reader what broke, without opening the file.

**Setup/teardown:** no shared fixture framework; each test builds its own `tempfile::tempdir()` and constructs the object under test directly. No `beforeEach`-equivalent; repetition across tests in the same module is normal.

## Mocking

There is no mocking framework dependency (no `mockall`, no `mockito` found in `Cargo.toml`). The project's stated strategy avoids mocking network boundaries by design:

> "Network-dependent code (IMAP, SMTP, POP3, Google, Microsoft Graph, CalDAV, iCal subscriptions) is tested against parsing and error-mapping logic, not live servers. Keep the transport thin and the parsing pure so the pure part is testable." (`CLAUDE.md`)

**What this means in practice:**
- Parsing and error-mapping functions (e.g. `redact_provider_message` in `src/common/error.rs`) are unit-tested directly with hand-built input strings, no live server or mock server involved.
- `src/service/protocols/`, `src/service/oauth.rs`, and the provider clients have low `cargo llvm-cov` coverage by design — this is explicitly *not* treated as a coverage gap to close by writing more unit tests; it is tracked as separate work requiring a live account (`CLAUDE.md`, "Coverage" section).
- Do not introduce a mocking library to fake IMAP/SMTP/OAuth servers; keep new transport code thin and push logic into pure, directly-testable functions instead, consistent with existing structure.

**What IS "mocked":** real local infrastructure stands in for external state — `tempfile::tempdir()` for a filesystem, a real `MessageCache` backed by a temp SQLite file for storage, `tokio_test::block_on` for async execution — never an in-memory fake standing in for the app's own storage layer (the integration test comment above documents removing exactly that kind of fake because "nothing in the application ever reached" it).

## Fixtures and Factories

No dedicated fixtures directory or factory crate. Test data is constructed inline per test using the domain constructors, e.g. `Account::new_simple(name, email, Protocol::Imap)` in `tests/integration_tests.rs`. Prefer this pattern (build via the real constructor, not a raw struct literal) so tests exercise the same validation the running app does.

## Coverage

**Tool:** `cargo llvm-cov --lib --summary-only`.

**No enforced numeric threshold.** Coverage is explicitly described as "the cheap wide sweep" answering only "what never runs at all," subordinate to mutation testing and guard measurement for judging whether tests actually assert anything meaningful. Low coverage in `service/protocols`, `service/oauth`, and provider clients is a known, accepted gap (network transport, see Mocking above) — do not treat raising that number as a goal in itself.

## Mutation Testing (`scripts/mutants.sh`)

This project treats mutation testing as more authoritative than coverage or a green suite. Key facts to get right when planning or reviewing test work here:

- Compares against a real commit/tag, e.g. `scripts/mutants.sh --since v0.19.0`. Never `--since main`, since every commit lands on `main` and that comparison finds nothing while reporting success.
- A whole-tree run takes about two days; a pull request's own diff-scoped run takes minutes. Scope runs to the directory or diff under active work: `scripts/mutants.sh src/service`.
- `mutants.out` is written incrementally; reading it before the process exits gives wrong, partial numbers (a documented incident quoted "17 of 18 caught" mid-run when the finished number was "37 of 51").
- The script now refuses to summarize a partial or degenerate run rather than reporting one, covering three previously-silent failure modes: (1) a build that failed before any mutant ran, which used to print "every mutant caught," (2) mutants recorded "unviable" without distinguishing "compiler rejected it" from "compiler never started," and (3) a run where the compiler rejected every single mutant or there were no mutants at all, meaning the suite was never actually exercised against a changed line.
- `application::filters`, `due`, `tagging`, and `sign_off` were swept clean as of 2026-08-01 (157 mutants, 141 caught, 16 uncompilable, 0 missed) after three passes. The pattern the first two passes found: tests covering only the paths someone would think to write a test for, leaving whole families of behavior (e.g. most of the fields a filter rule can name, most of the ways it can match) with one tested member and the rest untested. **When reviewing or writing tests against a function that switches on a string or enum with several arms, test every arm, not just the one that occurred to you.**
- Before trusting a new regression test, take the fix out by hand and confirm the test goes red; a test that has never been red proves nothing (stated directly in `CLAUDE.md` and echoed in `scripts/mutants.sh`'s own header comments).

## Guard Records (`guards/guards.toml`, `scripts/guards.sh`)

A distinct mechanism from mutation testing, complementary to it: `guards/guards.toml` currently holds 501 guard records (`grep -c '^\[\[guard' guards/guards.toml`). Each record names an exact source edit (`before` -> `after`) that should turn a specific, named set of tests red.

- `scripts/guards.sh` applies each record's edit, runs the whole suite (the library by default; a record can name `tests/` instead when the rule it checks only exists there), and requires the tests that actually failed to match the record's named list **exactly, in both directions** — not just "at least these tests failed." A record naming fewer tests than the edit actually reddens is exactly the failure mode this exists to catch (a guard silently stopped discriminating while keeping its old, now-inflated name).
- `before` must appear exactly once in the source tree; `tests/house_style.rs::test_every_guard_record_still_names_one_place_in_the_tree` checks this on every commit as a fast, always-on proxy — it can only confirm the edit location still exists, not that the recorded test list is still accurate. Only a full `scripts/guards.sh` run confirms that.
- `scripts/guards.sh` is *not* part of `scripts/check.sh` and not run by the commit hook — one build per guard makes a full run one to two hours. Run it deliberately after a change that touches code a guard is about, with nothing else building concurrently.
- Adding a new guard: take the break by hand first, write down everything that actually went red, not what you expected to go red.
- As of 2026-08-12, 192 records had been through a hand-verification sweep; 309 had arrived since and had not (numbers as recorded in `guards/guards.toml`'s own header — check the file directly for the current split, since both this document and the file's comment expect that ratio to keep shifting). Do not assume every record in the file is currently accurate; the file's own mechanism (`before` appearing exactly once) is the only continuously-checked guarantee, and it only checks location, not the test list.

## Common Patterns

**Async testing:**
```rust
assert!(!tokio_test::block_on(controller.is_connected()));
```

**Reading-source-text testing (house style / wired / structural tests):** build a small recursive file collector (`fn collect(dir, extensions, into)`), read files as text, and assert a textual property. Two hard-won rules to follow when writing a new one of these:
1. If the check must know what a *release* build actually compiles (as opposed to what's simply present in the file), call `src/common/what_ships.rs::what_ships` rather than writing a new `#[cfg(test)]`-scanning loop — three earlier hand-rolled versions in this codebase got this wrong in the same direction.
2. State plainly, in the test's own module doc, what the check cannot see (e.g. `tests/wired.rs`'s header: wiring a key to a menu item proves Windows will dispatch it, not that the handler does the right thing). This is the established documentation convention for this whole file family, not optional.

---

*Testing analysis: 2026-08-29*

# Coding Conventions

**Analysis Date:** 2026-08-29

## Naming Patterns

**Files:**
- Module files named for the concern they own, lowercase snake_case: `src/application/filters.rs`, `src/application/mail_sync.rs`, `src/service/oauth.rs`, `src/data/message_cache/`.
- Guard/check test files named for what they verify, not for the module they touch: `tests/house_style.rs`, `tests/wired.rs`, `tests/checkbox_labels.rs`, `tests/theme_reach.rs`.

**Functions:**
- snake_case throughout. Names describe behavior, not implementation: `redact_provider_message`, `what_ships`, `names_after`.
- Long, sentence-like names are used deliberately for test functions and for constants that stand in for a documented list, e.g. `A_FIELD_A_RULE_MAY_NAME`, `A_CONTROL_NO_SCREEN_WRITES`. This is a house pattern: a comment's claim is turned into a named, checkable value rather than left as prose.

**Variables:**
- snake_case, short and local. No Hungarian notation, no type suffixes.

**Types:**
- PascalCase for structs/enums: `FilterRule`, `FilterAction`, `Account`, `CachedMessage`.
- Error variants are PascalCase with an associated `String` payload or named fields (`Error::Api { status, provider, message }`), see `src/common/error.rs`.

## Code Style

**Formatting:**
- `cargo fmt --all -- --check`, no `rustfmt.toml` present, so default rustfmt style applies project-wide.

**Linting:**
- `cargo clippy --all-targets --all-features -- -D warnings` (`scripts/check.sh`). No `clippy.toml`, so default clippy lint set, but warnings are build failures — a warning-only clippy run is never acceptable.
- `#[allow(...)]` to silence a lint is against house rule: "Never silence a lint with `#[allow(...)]` to get a commit through. Fix the code, or if the lint is genuinely wrong for this case, add the allow with a comment saying why" (`CLAUDE.md`).

## Import Organization

- `use` blocks are grouped informally: crate-internal (`crate::common::...`, `crate::data::...`) then external crates, no enforced ordering tool beyond rustfmt's import grouping. Example from `src/application/filters.rs`:
  ```rust
  use crate::common::Result;
  use crate::data::message_cache::{CachedMessage, MessageFilterRule};
  use regex::RegexBuilder;
  ```
- No path aliases beyond the crate's own module tree (`wixen_mail::application::...`, `wixen_mail::service::...`, `wixen_mail::data::...`) as seen from `tests/integration_tests.rs`.

## Error Handling

**Pattern (verified against `CLAUDE.md`'s claim):**
- All fallible library code returns `crate::common::Result<T>` (alias for `std::result::Result<T, Error>`), defined in `src/common/error.rs`.
- `Error` is a flat enum: `Config`, `Network`, `Authentication`, `Protocol`, `Security`, `Api { status, provider, message }`, `Other`, each carrying a `String` message except `Api`. `Display` is hand-implemented; `std::error::Error` is implemented with no extra machinery (no `thiserror`/`anyhow` dependency).
- The claim in `CLAUDE.md` — "Do not use `unwrap` or `expect` outside tests and `build.rs`" — mostly holds but is not absolute at the letter: `src/common/error.rs::redact_provider_message` uses no unwrap, and `src/common/what_ships.rs` uses no unwrap/expect in production code either. `src/main.rs` and `build.rs` are the named exceptions per `CLAUDE.md` and both do use `unwrap`/`expect`. Grep across `src/**/*.rs` for `unwrap()`/`expect(` returns matches almost entirely inside `mod tests` blocks (203 files have `mod tests`) or in `tests/*.rs` files, consistent with the documented rule. Treat any non-test, non-`main.rs`/`build.rs` `unwrap`/`expect` found during a code review as a rule violation to flag, not as house style.
- Only 4 hand-written `impl From<...>` conversions exist in `src`. `CLAUDE.md` recommends "Reach for `From` conversions rather than hand-written mapping functions between layer types" but this is aspirational more than dominant in the current tree — check the specific module before assuming a `From` impl exists for a given conversion.
- Foreign/provider errors are mapped at the boundary: see `redact_provider_message` in `src/common/error.rs`, which caps and redacts a provider's raw HTTP response body (500-char limit, credential-shaped fields stripped) before it becomes part of an `Error::Api` message — written specifically because unbounded provider bodies and echoed OAuth tokens have leaked into logs before.

## Comments

**Style:** Comments are prose, often several sentences, explaining *why* a check or a piece of logic exists, frequently narrating the specific defect that motivated it (dates, counts, named bugs). This is the dominant documentation style in this codebase, not an exception — see the module docs of `tests/house_style.rs`, `tests/wired.rs`, `src/common/what_ships.rs`, `guards/guards.toml`, `scripts/check.sh`, `scripts/guards.sh`.

**Doc comments:**
- `///` doc comments on public items follow the same explain-the-why pattern, not just a type signature restatement. Example: the doc on `FilterRule::field` in `src/application/filters.rs` explains a defect where the comment undersold what the reading actually handled.
- Constants that mirror a documented enumeration (allowed field names, allowed match types) are named in SCREAMING_SNAKE_CASE and referenced from the doc comment via `[\`NAME\`]` intra-doc links, and a test enforces the constant and the actual reading logic stay in sync (`src/application/filters.rs`).

**Banned characters:** Em dash (U+2014) and en dash (U+2013) are banned from all prose the project owns — `src/**/*.rs`, `docs/**/*.md`, `tests/**/*.rs`, `scripts/**/*.{sh,py,ps1}`, `guards/**/*.toml`, `installer/**/*.iss`, `.github/**/*.yml`, plus `README.md`, `CLAUDE.md`, `Cargo.toml`, `.gitignore`, `build.rs` — enforced by `test_no_dashes_that_should_be_punctuation` in `tests/house_style.rs`. Use a colon, a comma, or two sentences instead.

## Function Design

- Small functions with a single clear purpose; helper functions are freely split out and given long descriptive names rather than left as closures or inline logic (e.g. `a_line_and_the_one_after_it`, `names_after` in `tests/*.rs`).
- Enums are preferred over stringly-typed values per `CLAUDE.md`, though `FilterRule.field` and `FilterRule.match_type` in `src/application/filters.rs` are still `String` today with a doc-comment-plus-test discipline standing in for a real enum — a known gap between stated convention and current code, worth flagging in any future refactor of that module.

## Module Design

**Layering (four top-level modules under `src/`):**
- `src/common/` — cross-cutting types shared by every layer: `error.rs` (Error/Result), `types.rs`, `moment.rs`, `paths.rs`, `logging.rs`, `answering.rs`, `version.rs`, `temp_home.rs`, `what_ships.rs`.
- `src/data/` — persistence, including `src/data/message_cache/` (SQLite-backed cache/storage).
- `src/service/` — protocol/transport clients: `src/service/protocols/`, `src/service/oauth.rs`, `src/service/safebrowsing/`, `src/service/spellcheck/`, `src/service/cache.rs`, `src/service/security.rs`.
- `src/application/` — business logic orchestrating data + service: `accounts.rs`, `filters.rs`, `mail_sync.rs`, `mail_controller.rs`, `contact_groups.rs`, `messages.rs`, `search.rs`, `allowed.rs`, `attaching.rs`.
- `src/presentation/` — UI layer built on wxdragon, including `src/presentation/accessibility/` for screen-reader-facing code (`presentation::accessibility::screen_reader` is the required path for announcing dynamic changes per `CLAUDE.md`).
- `src/vendor/paperback/` — vendored third-party code, kept separate from the project's own layers.

**Exports:** Public API surface is exercised directly by `tests/integration_tests.rs` via `wixen_mail::application::...`, `wixen_mail::service::...`, `wixen_mail::data::...` — treat the crate as a library with `src/lib.rs` as its root, `src/main.rs` as a thin binary entry point.

**Special-case reading rule:** `src/common/what_ships.rs::what_ships` defines exactly which half of a source file a *release* build compiles (everything except `#[cfg(test)]`-gated items). Any source-reading check or guard that must reason about "does this code actually ship" — as opposed to "does this text exist in the file" — must use this function rather than a hand-rolled `#[cfg(test)]` scan; three earlier hand-rolled versions in this codebase were wrong in the same direction (over-truncating the file).

---

*Convention analysis: 2026-08-29*

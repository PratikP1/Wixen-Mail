# Deferred items, phase 6

Out-of-scope findings met while executing a plan, recorded here and in
`.planning/WINDOWS.md`, not fixed in the plan that found them.

## Found by 06-03 tasks 3 and 4, 2026-09-13

- **`keyring` 4.1.5 races its own lazy initialisation, and it is ledger 365's
  cause.** `Entry::new` guards the first-use setup with a `compare_exchange`
  on an `AtomicBool`; the thread that wins sets the default store, and a
  thread that loses proceeds to `keyring_core::Entry::new` before the winner
  has finished, and gets "No default store has been set". Ten tests in
  `tests/a_move_says_what_has_not_been_sent.rs` start in parallel on a fresh
  process and call `save_account`, so the whole gate fails about one run in
  three. Fix: a `std::sync::Once` around the first `keyring::Entry::new` in
  `src/service/secret_store.rs`, or an upstream report. Ledger 374.
- **Integration targets reach the real Windows credential store.** The
  in-memory seam in `secret_store.rs` is `#[cfg(test)]`, which a target under
  `tests/` never sees. The target above deletes a non-existent entry on every
  run, which is harmless and still wrong. A seam the integration targets can
  see, on the `what-ships` feature precedent, is the shape of the fix. Same
  ledger entry.
- **`locales/` maps to no gate target.** A commit touching only the catalogue
  runs no `common::catalogue::` test until the merge. A mapping from
  `locales/*.ftl` to that module in `scripts/check.sh`, or `locales/` in
  `house_style`'s `ours()`, are the two candidates; the plan says the second is
  a version 2 question. Ledger 373.
- **Four interface labels carry English day and month names as examples**:
  "Every weekday, Monday to Friday", "Month first, July 26", "Day first, 26
  July", "A word, July 26, 2026". Allowed by name in the source-reading guard
  in `src/common/how_the_machine_writes_dates.rs`; they are interface text and
  belong to version 2's translation. The two settings labels could be built
  from the wrapper so the example follows the machine, which would be a small
  change in `wx_settings.rs` and is not obviously better. Ledger 372.

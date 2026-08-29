# Codebase Concerns

**Analysis Date:** 2026-08-29

Wixen Mail is a large, actively documented alpha (259,723 lines under `src/`). Most of the
concerns below are ones the project already names in its own docs. Where that is true, this
document cites the source rather than re-discovering it. A smaller set is flagged as newly
observed during this pass and marked as such.

## Known and Stated (the project's own words)

### Cached mail database is not encrypted

- Stated: `CLAUDE.md` line 374 — "The cached mail is not encrypted, and the docs say so. Do not
  claim otherwise anywhere." Also `docs/changelog.md` line 662: "**These files are not
  encrypted**, and neither is anything else Wixen Mail stores."
- Files: `src/data/message_cache/mod.rs` (SQLite cache), `src/common/version.rs` mentions this in
  release messaging per changelog line 413-420.
- Impact: anyone with filesystem access to the user's profile can read cached mail contents,
  headers, and metadata in plaintext.
- Stated mitigation/rationale: secrets (passwords, OAuth tokens) are kept out of the database
  entirely and live in the OS credential store via `keyring` (`CLAUDE.md`, "Secrets stay out of
  the tree, and out of the database"). Encrypting the whole database is called out as "a decision
  with a build cost, not something to imply in a feature list" — i.e., deliberately deferred, not
  forgotten.

### Write paths to real accounts are unproven — gated by `Allowed`

- Stated: `src/application/allowed.rs` module doc (lines 1-9): "Sending a message, removing one
  from a server, or deleting a task at a provider can [hurt somebody], and none of those paths has
  run for real yet." The `personal_information` field doc (line ~46) says outright: "this is the
  least proven code in the application: none of the three sync paths has met a live account."
- Mechanism: `Allowed::FOR_TESTING` (the default for a new install) permits tasks/contacts/calendar
  writes but keeps `mail: false`. Three independent gates must all agree (`Allowed::and`):
  command line, app setting, per-account setting — the design deliberately has no way to force
  writes on from the command line, only to restrict further (`--read-only`).
- User-facing warning: `src/presentation/first_run.rs` and the Allowed Changes settings screen
  surface the experimental status directly, per `CLAUDE.md` ("If you expect bug reports from
  something, that belongs in the product... `application::allowed` and `presentation::first_run`
  are how this is done here").
- Files: `src/application/allowed.rs` (495 lines), `src/presentation/first_run.rs` (391 lines).
- Not yet run against: contacts sync (`src/application/contacts_sync.rs`, 12,247 lines), calendar
  sync (`src/application/calendar.rs` 11,154 lines + `src/application/caldav_sync.rs` 5,933
  lines), tasks sync (`src/application/tasks_sync.rs`, 5,142 lines), and mail sending
  (`src/application/mail_sync.rs`, 3,717 lines) — all substantial, all unexercised against a live
  provider as of this analysis.

### Network transport tested for parsing, not against live servers

- Stated: `CLAUDE.md` — "Network-dependent code (IMAP, SMTP, POP3, Google, Microsoft Graph,
  CalDAV, iCal subscriptions) is tested against parsing and error-mapping logic, not live
  servers." Also: "Low coverage in `service/protocols`, `service/oauth` and the provider clients is
  the network transport that has never been run against a live account, which is tracked as work
  rather than fixable by writing more tests."
- Files and sizes (this is the surface with the least real-world exercise):
  - `src/service/protocols/imap.rs` — 3,673 lines
  - `src/service/protocols/smtp.rs` — 1,411 lines
  - `src/service/protocols/pop3.rs` — 696 lines
  - `src/service/protocols/xoauth2.rs` — 105 lines
  - `src/service/google_api.rs` — 1,858 lines
  - `src/service/microsoft_graph.rs` — 1,613 lines
  - `src/service/caldav.rs` — 8,149 lines (the single largest service file in the codebase)
- Coverage command that surfaces this directly: `cargo llvm-cov --lib --summary-only`.
- Design mitigation already in place: transport is kept thin and parsing/error-mapping kept pure
  so the pure part is unit-testable even though the wire protocol never is in CI.
- Risk this leaves open: provider behavior (rate limits, pagination edge cases, malformed
  responses, auth token refresh races) is assumed from documentation and RFCs, not observed. A
  first real run against Gmail, Outlook/Graph, or a CalDAV server is the first time these paths
  are exercised end to end.

### Search "In box" scope selector is wired but not read

- Stated: `docs/changelog.md` (Unreleased, Saved Searches section): "The In box on the Search
  window (All Folders, Current Folder, Subject Only, From Only) is still not read by anything, so
  a saved search always covers the whole account. That is a defect in Search, not in saving one."
- Impact: a UI control exists and presents choices to the user that have no effect. This is a
  "wired but unexercised by its own logic" defect — the opposite of dead code (live code, dead
  effect).

### Saved search cannot express "search inside message text that was cleared"

- Stated: `docs/changelog.md`, same section: "Message text is cleared to stay within a size
  budget, so an old message may have headers here and no text. Nothing built into the program
  saves a search of that kind yet."
- Files: message text storage/eviction is in `src/data/message_cache/mod.rs` (3,757 lines).

### Outlook PST/OST import never run against a real Outlook file

- Stated: `docs/changelog.md` line 674: "**This has never been run against a real Outlook file.**
  There is no way to [test it short of one]." And line 688-690: an imported item "has never been
  on anybody's server, so a folder belonging to a server is the one place it does not belong:
  every check for mail would compare it against a provider that has never heard of it."
- Design mitigation: imported items get their own non-server-backed home ("Under Imported it
  belongs to nobody's [folder]") specifically to avoid corrupting sync state against a real
  account.
- Known gaps stated alongside: "What does not come across is counted and said," and "Messages
  whose text has never been downloaded to this computer are left out."

### S/MIME signature verification has a known blind spot

- Stated: `docs/changelog.md` line 613-617: "mail that arrived before this version reads as
  unsigned," and mail collected over IMAP or POP3 via a path that "does not keep the original form
  yet" cannot be verified the same way.
- Files: `src/service/signed_mail.rs` — 6,738 lines, the fourth-largest file in the codebase.
  Given the size and the stated gap, this is one of the more complex and least-verified subsystems
  by the project's own account.

### Recurring-event editing/deletion needs the series already stored locally

- Stated: `docs/changelog.md` lines 1793, 1813: editing and deleting a recurring event series
  "needs the series itself already stored here" — an item synced in isolation, without its
  recurrence data, cannot be edited or deleted as a series.
- Files: `src/application/calendar.rs` (11,154 lines), `src/application/caldav_sync.rs` (5,933
  lines).

### Calendar week/month recurring-event display not built

- Stated: `docs/changelog.md` line 1425: "weeks and months is not built, so both now say so and
  are switched off."

### Default-mail-client registration is intentionally inert

- Stated: `docs/changelog.md` lines 870-971: "This still does not make Wixen Mail the default for
  anything... It does not set it, because Windows has not allowed a program to make itself
  default." This is a documented platform constraint, not a bug, but any future work assuming
  "Set as Default" fully works will hit this wall.

### Weekly recurrence rule naming two-or-more of a field

- Stated: `docs/changelog.md` line 1714: "that one combination, a weekly rule naming two or more
  [days/etc.]" is a named known limitation in the recurrence editor.

## Observed During This Pass (not previously called out in docs)

### `caldav.rs` is the largest single service file at 8,149 lines

- File: `src/service/caldav.rs`.
- Concern: combined with "network transport tested for parsing, not live servers" above, this is
  the largest unexercised-against-a-live-server surface in the codebase by a wide margin (more
  than double `imap.rs`, the next largest protocol file). A single file of this size increases the
  odds that failure modes are entangled rather than isolated; when the first real CalDAV server is
  exercised, expect this file to be where problems surface first and be hardest to localize.
- Recommendation: no action implied beyond what the project already tracks as work — flagged here
  because of its outsized share of the untested-against-reality surface.

### Very few `TODO`/`FIXME`/`HACK` markers in source (2 total, excluding tests)

- Observed via `grep -rn "TODO|FIXME|HACK|XXX" src/ --include="*.rs"` (excluding matches inside
  test files): only 2 hits in the whole 259k-line tree.
- This is a positive signal, not a concern by itself, but it means technical debt in this codebase
  is not marker-driven — it lives in the `docs/changelog.md` "Known limitations" notes instead.
  Anyone auditing for debt should treat the changelog as the primary debt ledger, not source
  comments.

### Largest presentation files carry a lot of undifferentiated UI logic

- Files: `src/presentation/wx_app.rs` (19,659 lines), `src/presentation/managers.rs` (8,832
  lines), `src/presentation/wx_managers.rs` (4,058 lines), `src/presentation/wx_compose.rs` (3,699
  lines).
- Concern: `wx_app.rs` at nearly 20,000 lines is far larger than any other file in the repository,
  including the largest service file. This was not called out in the project's own docs as debt,
  but a file of this size is inherently harder to keep every code path wired and reachable — the
  project's own "done means it runs" guardrail depends on the ability to trace non-test paths to
  every piece of logic, which gets harder as a single file grows. No specific dead or unwired code
  was confirmed in this pass; this is a structural risk factor worth a `dead-code-hunter` pass
  focused on this file specifically, not a confirmed defect.

### Sync applications (`application/*_sync.rs`) are both large and unproven against live accounts

- Files: `src/application/contacts_sync.rs` (12,247 lines), `src/application/calendar.rs` (11,154
  lines), `src/application/caldav_sync.rs` (5,933 lines), `src/application/tasks_sync.rs` (5,142
  lines), `src/application/mail_sync.rs` (3,717 lines).
- Combined with the `Allowed::personal_information` doc comment calling these "the least proven
  code in the application," their size compounds the risk: over 38,000 lines of sync logic across
  five files, none of it exercised end to end against a real provider as of this analysis. This is
  the single largest concentration of stated-but-unverified behavior in the codebase.

## Security Considerations

**Unencrypted local cache (stated, see above):**
- Risk: local disk access exposes cached mail content and headers in plaintext SQLite.
- Current mitigation: credentials and tokens are excluded from the database and stored via OS
  keyring (`src/service/credentials.rs`-equivalent path referenced in `CLAUDE.md`, plus
  `service::oauth`, `service::caldav`). Never logged, per the same guardrail.
- Recommendation: the project already frames this as a scoped future decision (whole-database
  encryption), not an oversight; no new recommendation needed beyond what's tracked.

**Untrusted HTML rendering in email preview:**
- Stated: `CLAUDE.md` — "The email preview renders untrusted HTML in a WebView... sanitize with
  `ammonia` first, and keep the rendered document's heading structure and link text intact."
- Files: search for the WebView preview implementation under `src/presentation/` (not read in this
  pass; flagged for a follow-up mapper focused on the mail-rendering path if the ammonia
  sanitization boundary needs a code-level audit).

**Sign-in problem messaging previously named a nonexistent window:**
- Stated: `docs/changelog.md` line 499: "Three messages told [a wrong location]" — already fixed,
  listed here only as a documented pattern (error messages pointing users to the wrong place) worth
  watching for elsewhere in the auth flow.

## Fragile Areas

**`src/service/signed_mail.rs` (6,738 lines):**
- Why fragile: large surface, stated gap around pre-version mail reading as unsigned and IMAP/POP3
  paths not preserving original form for verification (see above). S/MIME correctness bugs are
  security-relevant (a broken signature could read as valid, or a valid one as broken).
- Safe modification: any change here should be checked against both the "verify" and "reject"
  paths, and against a message collected through each of IMAP, POP3, and import, since the
  changelog already documents divergent behavior across those paths.

**`src/service/caldav.rs` (8,149 lines) and the calendar sync stack:**
- Why fragile: largest untested-against-live-server file, combined with calendar/CalDAV sync being
  called out as unproven personal-information write paths. Recurring events already have two
  stated known limitations (series-editing needs local series storage; weekly rules naming
  multiple fields).
- Safe modification: treat any change here as touching the least-verified part of the application;
  favor small, individually testable pure functions per the project's own "keep the transport thin
  and the parsing pure" guidance in `CLAUDE.md`.

## Test Coverage Gaps

**Live-provider behavior (stated, not measured further here):**
- What's not tested: actual request/response cycles against Gmail, Microsoft Graph, IMAP/SMTP/POP3
  servers, and CalDAV servers.
- Files: `src/service/protocols/*.rs`, `src/service/google_api.rs`,
  `src/service/microsoft_graph.rs`, `src/service/caldav.rs`, `src/service/oauth` (referenced in
  `CLAUDE.md`, not directly enumerated here).
- Risk: first live run is also first real-world validation; provider quirks (rate limiting,
  pagination, token refresh timing, malformed responses) are unobserved.
- Priority: already tracked by the project as work rather than a testing gap to close with more
  unit tests (`CLAUDE.md`: "tracked as work rather than fixable by writing more tests").

**Write paths for mail send/delete and personal-information sync:**
- What's not tested: end-to-end behavior of sending, deleting, or modifying mail on a server, and
  all three personal-information sync directions (tasks, contacts, calendar) against a live
  account.
- Files: `src/application/mail_sync.rs`, `src/application/contacts_sync.rs`,
  `src/application/calendar.rs`, `src/application/caldav_sync.rs`, `src/application/tasks_sync.rs`.
- Risk: gated behind `Allowed`, so the blast radius of an unproven bug is limited to opted-in
  testers, but the bug surface itself is unverified.
- Priority: gated by design (`src/application/allowed.rs`), not an oversight — flagged here as
  scope for future verification work, not a defect.

---

*Concerns audit: 2026-08-29*

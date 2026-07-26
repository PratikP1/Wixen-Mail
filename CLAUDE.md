# Wixen Mail

A fully accessible, lightweight mail and personal information client built with Rust and
wxdragon (wxWidgets bindings). Windows-first. Screen reader users are the primary audience,
not an afterthought.

<!-- BEGIN GUARDRAILS (managed by /setup-guardrails) -->

## Guardrails

The standing guardrails in `~/.claude/CLAUDE.md` apply to this project. What follows is this
project's tightening and the concrete commands that back it.

### Test-driven development

Red, green, refactor, on every change. Invoke the `tdd` skill for any implementation, bug fix,
or feature. Write the failing test first, then the minimum code that passes it, then refactor.

```bash
cargo test --all-targets
```

Unit tests live in `#[cfg(test)] mod tests` beside the code they cover. Cross-layer tests live in
`tests/integration_tests.rs`. Async code uses `tokio-test`; anything touching the filesystem uses
`tempfile` rather than real user directories.

Network-dependent code (IMAP, SMTP, POP3, Google, Microsoft Graph, CalDAV, iCal subscriptions) is
tested against parsing and error-mapping logic, not live servers. Keep the transport thin and the
parsing pure so the pure part is testable.

### Elegant code

Invoke the `elegant-code` skill whenever writing, reviewing, or refactoring. In this codebase that
means:

- Errors flow through `common::Error` and `common::Result`. Do not use `unwrap` or `expect` outside
  tests and `build.rs`. Map foreign errors at the boundary where they enter.
- Prefer typed values over stringly-typed ones. When a database column holds a fixed set of strings
  ("confirmed", "tentative", "cancelled"), model it as an enum and convert at the SQL boundary.
- Small, self-documenting functions. If a name needs a comment to explain what it does, rename it.
- Reach for `From` conversions rather than hand-written mapping functions between layer types.

### CI must stay green

Every commit builds, passes tests, and passes lint. These are the same four checks CI runs, and
clippy is enforced with `-D warnings`, so a warning is a build failure:

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets && cargo build --release
```

Never silence a lint with `#[allow(...)]` to get a commit through. Fix the code, or if the lint is
genuinely wrong for this case, add the allow with a comment saying why.

### Done means it runs

Compiling and green tests are not done. A feature is done when a non-test path reaches it and it is
exercised end to end in the running application. A `dead_code` warning on a UI field is evidence
that a panel was built but never wired, not noise to be silenced.

Run `dead-code-hunter` after finishing a feature. If something cannot be finished because it depends
on hardware, an external service contract, or missing infrastructure, say so and gate it. Never
present a stub as complete.

### Accessibility

Target WCAG 2.2 Level AA, applied to a Windows desktop application. This is the product's reason to
exist, so accessibility work is not a review-time cleanup pass, it is part of building the feature.

Automated checks catch roughly half of accessibility defects. They do not replace testing with real
assistive technology. Structure present is not experience good.

- **Blind, screen readers.** Every control exposes a correct UI Automation Name, Role, Value, and
  State. Focus is managed and never lost when a panel or dialog changes. Announce dynamic changes
  through `presentation::accessibility::screen_reader` rather than relying on the user to discover
  them. Verify with NVDA, and spot-check with Narrator and JAWS.
- **Low vision.** Contrast at least 4.5:1 for text and 3:1 for UI components and meaningful graphics,
  in both light and dark themes. Never convey information by color alone; pair every color cue with
  text or an icon. Honor the system font size, respect Windows high contrast themes, and keep a
  clearly visible focus indicator.
- **Physical and motor.** Everything reachable and operable by keyboard alone, with no mouse-only or
  drag-only interaction. Accelerators follow standard Windows conventions and are documented in
  `docs/KEYBOARD_SHORTCUTS.md`, which is updated in the same commit as the shortcut. No timing traps.
- **Learning and cognitive.** Plain language in labels, messages, and errors. Predictable, consistent
  navigation across the mail, calendar, contacts, tasks, notes, and reminders modules. Errors say
  what happened, why, and what to do next. Do not make the user re-enter information they already
  gave (3.3.7), and do not require a memory or transcription test to authenticate (3.3.8).
- **Hearing.** Every audio cue has a visible equivalent. Never signal something by sound alone.
- **Vestibular and photosensitivity.** Honor the system reduced-motion setting. Nothing flashes more
  than three times per second.

The email preview renders untrusted HTML in a WebView. Accessibility and security both apply there:
sanitize with `ammonia` first, and keep the rendered document's heading structure and link text
intact so the screen reader can navigate it.

Use the accessibility specialist agents when they fit: `Desktop Accessibility Specialist` for control
patterns and UIA exposure, `Desktop A11y Testing Coach` for NVDA and Accessibility Insights
workflows, `cognitive-accessibility` for language and flow, `contrast-master` for theme colors.

### Documentation and writing

- User-facing docs (`README.md`, everything under `docs/`, release notes): invoke `writing-craft`.
  Plain language, semantic structure, worked examples. No em-dashes; use commas, colons, or separate
  sentences. Avoid AI-slop vocabulary such as delve, robust, seamless, leverage, comprehensive,
  empower.
- Commits, PR descriptions, issue comments, code review: invoke `writing-style`. Direct and brief,
  why over what.
- User-visible changes get a `docs/changelog.md` entry under `[Unreleased]` in the same commit.

### Project rules

- **No AI attribution anywhere.** No `Co-Authored-By` lines naming an AI, no AI or assistant names in
  commit messages, branch names, code comments, or documentation. This applies to every commit going
  forward.
- **Windows-first.** Windows is the supported platform. Platform-specific code sits behind
  `#[cfg(target_os = "windows")]` with a sane fallback for other targets so the crate still builds.
- **Secrets stay out of the tree.** OAuth client credentials load from `oauth.toml` (gitignored) with
  `oauth.toml.example` as the tracked template. Tokens go to the OS keychain via `keyring`; cached
  data is encrypted with AES-256-GCM. Never log a token, password, or message body.
- **Schema changes are additive.** `MessageCache` opens existing user databases, so add tables with
  `CREATE TABLE IF NOT EXISTS` and columns with `ensure_column_exists`. Never drop or rename a column
  that shipped.

<!-- END GUARDRAILS -->

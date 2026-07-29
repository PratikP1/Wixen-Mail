# Wixen Mail

A fully accessible, lightweight mail and personal information client built with Rust and
wxdragon (wxWidgets bindings). Windows-first. Screen reader users are the primary audience,
not an afterthought.

## Guiding Principles

The project answers four questions; let them judge every change. **What is it for?** Making
correspondence and personal information legible to people who cannot see it. Messages,
folders, threads, read state, attachments, events, contacts, tasks, notes, and reminders
declared to the platform accessibility API as structured facts, so a screen reader user
works through a full inbox rather than reconstructing it. **What does it strengthen?** Their
independence, and the idea that the *application* declares its meaning instead of the screen
reader inferring it from a rendered DOM. Open protocols too: IMAP, SMTP, CalDAV, and
iCalendar deserve a client as open as they are. **What does it replace?** Outlook,
Thunderbird, and webmail, technically usable with a screen reader and painful in practice.
Not the screen reader, not the mail server, and not Wixen Terminal; those are complements.
**What does it allow to be done poorly?** That question is the source of these guardrails.
A rich accessibility surface makes it easy to mistake *structure present* for *experience
good*, and easier still to mistake a call that looks like accessibility for one that is.

Guardrails (each exists because we got it wrong here at least once):

1. **No feature is done until it runs in production.** Compiling and passing tests is not
   done. It is done when a non-test path reaches it and it is exercised end to end. Check
   reachability before claiming completion. (All eight PIM update variants were handled in
   the UI and sent by nothing; five modules rendered empty in every build.)
2. **Accessibility isn't done until a screen reader confirms it.** Tests prove structure;
   only a real NVDA or Narrator run proves experience, and the automated scan covers about
   half of WCAG. Worse, a call can look like accessibility and not be one. (Sixteen widgets
   were "named" with `set_name()`, which sets an internal wxWidgets identifier and never
   reaches the accessibility tree. It compiled and passed 324 tests.)
3. **No stubs presented as complete.** If something cannot be finished, say so and gate it.
   Never ship code that looks done and does nothing. (The note editor filled itself with
   "Note 1" and "(Note content loaded here)" on every selection.)
4. **A check nobody reads is worse than no check.** CI failing for months while looking
   maintained, or a scan reporting success while its scan step errored, buys false
   confidence. When a check can fail two ways, make it say which. (Both happened.)
5. **Feedback must be distinct and bounded.** Announcements and audio cues must be
   distinguishable from their siblings and must not flood under a syncing mailbox. Content
   read aloud needs controls and a fast mute, because private mail gets spoken in rooms.
6. **Untrusted input stays untrusted.** Message bodies come from strangers. Sanitizing them
   is security; preserving their heading structure and link text is accessibility. Neither
   excuses dropping the other.
7. **Publishing happens on purpose.** Anything that tags, releases, or pushes outward is
   triggered deliberately, never as a side effect. (A push to `main` cut two releases nobody
   asked for and promoted an alpha to beta.)
8. **Prefer few things excellent over many adequate.** For any new subsystem ask whether it
   is wired, exercised end to end, and raises the bar for the whole, or only adds surface.
   (Six modules shipped at once with one of them working.)
9. **Don't silently absorb upstream failures.** Where this papers over a sender's missing
   alt text, a provider's broken MIME, or a dependency with no accessible name, say so. The
   goal is a better ecosystem, not hidden gaps nobody is pressured to fix.

Fuller rationale in [docs/principles.md](docs/principles.md).

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
bash scripts/check.sh
```

Use the script rather than running the four commands by hand. Cargo shares build
fingerprints between `check`, `build`, `test`, and `clippy`, so a clippy run that
follows a build can be treated as fresh and report success without linting
anything. That has already put a clippy failure on `main` after a local run
reported clean. The script touches `src/lib.rs` first to force the work.

Never silence a lint with `#[allow(...)]` to get a commit through.

### Tests that would notice

A green suite says the code does what the tests say. It does not say the tests
would notice if it stopped, and those are different claims. Red/green is what
keeps them together, and it started at commit 182 of 344, so most of the tests
here were written after the code they cover and describe it rather than specify
it. Three tests written in one session to catch a named bug passed against that
bug.

```bash
scripts/mutants.sh src/service      # one directory, slow
scripts/mutants.sh --since main     # only what changed, minutes
```

Mutation testing alters the code in small ways and runs the suite. Anything
nothing catches is either untested behaviour or dead code. A whole-tree run is
about two days, so it is used scoped, and the pull request check runs it on the
diff only. Before trusting a new regression test, take the fix out and watch the
test fail; a test that has never been red proves nothing.

```bash
cargo llvm-cov --lib --summary-only
```

Coverage is the cheap wide sweep and answers a weaker question: what never runs
at all. Low coverage in `service/protocols`, `service/oauth` and the provider
clients is the network transport that has never been run against a live account,
which is tracked as work rather than fixable by writing more tests. Fix the code, or if the lint is
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
  Mail carries audio and video as attachments and embedded media: surface any captions or
  transcript the sender provided, and say plainly when none exists rather than presenting the
  media as though it were accessible.
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

### Working style

Apply the skill that fits the task without being asked. `tdd` and `elegant-code` on every coding
task. `writing-craft` or `writing-style` on prose. `dead-code-hunter` after a feature.
`naming-review`, `commit-hygiene`, `defensive-boundaries`, `dependency-audit`, and
`root-cause-investigator` as they fit.

Report outcomes faithfully. If tests fail, say so and show the output. If a step was skipped or
gated, say that. If something is done and verified, say it plainly. A feature that compiles but
was never reached is not implemented, and reporting it as implemented is the failure mode this
whole file exists to prevent.

Do not silently absorb upstream failures. Where this codebase papers over a broken or
inaccessible dependency, a sender's missing alt text, or a provider's malformed data, note it so
the gap stays visible.

### Project rules

- **No AI attribution anywhere.** No `Co-Authored-By` lines naming an AI, no AI or assistant names in
  commit messages, branch names, code comments, or documentation. This applies to every commit going
  forward.
- **Windows-first, and the accessibility layer is more Windows-only than it looks.** wxWidgets
  gives native, accessible controls on all three platforms, but two things this project relies on
  exist only on Windows: `wxAccessible`, which is how `set_accessible_name` reaches the
  accessibility tree, and `UiaRaiseNotificationEvent`, which is how announcements are spoken and
  brailled. Both compile and silently do nothing elsewhere. A macOS or Linux port needs its own
  bridge for each, not a framework change. Platform-specific code sits behind
  `#[cfg(target_os = "windows")]` with a fallback that keeps the crate building.
- **Secrets stay out of the tree, and out of the database.** OAuth client credentials load from
  `oauth.toml` (gitignored) with `oauth.toml.example` as the tracked template. Every other secret
  goes to the OS credential store via `keyring`: account passwords through `service::credentials`,
  tokens through `service::oauth`, CalDAV sign-ins through `service::caldav`. Nothing sensitive is
  written to `message_cache.db`, so the database can be copied and backed up without carrying
  credentials, and uninstalling can clear the secrets by clearing one place. Each service name has
  exactly one owner, because the code that erases them has to name the same entries as the code that
  wrote them. Never log a token, password, or message body.
- **The cached mail is not encrypted, and the docs say so.** Do not claim otherwise anywhere.
  Encrypting it means encrypting the whole database, which is a decision with a build cost, not
  something to imply in a feature list.
- **Schema changes are additive.** `MessageCache` opens existing user databases, so add tables with
  `CREATE TABLE IF NOT EXISTS` and columns with `ensure_column_exists`. Never drop or rename a column
  that shipped. The one exception taken so far is dropping a table that held secrets nothing read;
  if that case comes up again, say why in the commit.

### Versioning and releases

Bump the version in `Cargo.toml` as substantive changes land, in the same commit as the change. Do
not let the version sit still through a stretch of real work and then jump at release time. A bug
fix or a docs pass does not need a bump; a new feature, a schema change, or a behaviour change does.

`docs/changelog.md` is the record. Every user-visible change gets an entry under `[Unreleased]` in
the commit that makes it, and honest "Known limitations" notes belong there too. A feature list that
implies something works when it does not is worse than no entry.

Releases are cut deliberately. The Release workflow runs only on manual dispatch, never on push, and
you pick the level (`alpha`, `beta`, `rc`, or `release`) when you dispatch it. It then bumps from
whatever version it finds, commits that bump back to `main`, and tags. Since it computes the next
version from the current one, check what is in `Cargo.toml` before dispatching so the level you pick
lands where you expect.

`alpha`, `beta`, and `rc` publish as GitHub prereleases. Only `release` publishes as a full release,
and it should only be used when the version genuinely is feature-complete and tested.

<!-- END GUARDRAILS -->

# Wixen Mail

## What This Is

Wixen Mail is a mail and personal information client for Windows, written in Rust on
wxdragon over wxWidgets. Screen reader users are the primary audience, not an afterthought.
Six modules live in one window: mail, contacts, calendar, reminders, tasks and notes.
Version 0.45.0, past 1,000 commits.

## Core Value

Making correspondence and personal information legible to people who cannot see it.

That sentence is quoted from `docs/principles.md`, which answers four questions and is the
document that judges every change here. The four answers, condensed from that file:

**What is it for?** Messages, senders, folders, threads, read state, attachments, events,
contacts, tasks, notes and reminders declared to the platform accessibility API through
native controls, each with a name, a role and a state. A blind user should work through a
full inbox at their own pace rather than reconstruct it afterward.

**What does it strengthen?** The independence of its users: reading your own mail, accepting
your own meeting invitations, finding your own contact details, without a sighted
intermediary. The principle that the application declares its meaning rather than leaving a
screen reader to infer it from a rendered DOM. And open protocols, because IMAP, SMTP, POP3,
CalDAV and iCalendar deserve a client whose accessibility matches their openness.

**What does it replace?** Outlook, Thunderbird and webmail, which are usable with a screen
reader in the technical sense and painful in the practical one. Not the screen reader, not
the mail server, not Wixen Terminal. Those are complements.

**What does it allow to be done poorly?** This is the question that generates the guardrails,
because every strength here has a failure mode that looks like success. Accessibility calls
that are not accessibility calls. Structure present mistaken for experience good. Code
implemented but never wired. Stubs that look finished. Breadth over excellence. Announcement
flooding. Privacy lost through speech. Hostile HTML. Green checks that check nothing.
Automation that decides for you. Absorbing upstream failures silently. Each of those has
already happened in this codebase at least once.

## Requirements

### Validated

Most of the product is already built. `.planning/intel/built-and-left.md` holds the
evidence-backed inventory: 32 rows of "built and exercised", covering IMAP, SMTP and POP3
clients, OAuth 2.0 with tokens in the Windows credential store, the SQLite cache with body
eviction, all six modules in native virtual mode, compose with drafts and signatures,
tagging, filters, full-text search, address book with vCard, conversation view, sanitised
HTML rendering, S/MIME verification, multi-account, offline mode with an outbox, the
accessibility announcement layer, earcons and sound schemes, themes, the Windows installer,
Outlook PST import and the CI quality gates.

**One caveat covers all of it.** Nothing in Wixen Mail has ever run against a real mail
account or a real provider. "Exercised" means tested against parsing, fuzzed input and
loopback servers in CI, never against Gmail, Microsoft Graph, a CalDAV server, or any live
IMAP, SMTP or POP3 host. A further 13 rows are "built but unproven", including every mail
transport write path, the Google and Microsoft Graph clients, the 8,149-line CalDAV client
and all three personal information sync paths.

### Active

This milestone is the outstanding work: the two "not built" sections of the inventory. 44
requirements in `.planning/REQUIREMENTS.md`, grouped into folders and conversations, search
honesty, mail at scale on the wire, writing and reading a message in full, the other five
modules, how the application speaks, installing and updating, and measurement.

Live-account validation is real work and is deliberately not this milestone.

### Out of Scope

Declined on purpose, each with a decision recorded in the sources:

- **Exchange Web Services.** Microsoft begins blocking third-party EWS against Exchange
  Online on 1 October 2026, with full retirement by April 2027. `docs/plans/20260726-mail-at-scale.md`
  says outright: "We will not write EWS." Exchange Online goes through Microsoft Graph, where
  the client already lives.
- **Handing an attachment to Windows to open.** Deliberate. PDFs are the exception and are
  read in-app through `src/service/pdf.rs`.
- **Junk folder sync.** Deliberate. The folder can still be opened.

Deferred out of this milestone, with the reason recorded so it can be pulled back in:

- **Gmail X-GM-THRID conversations and X-GM-RAW server-side search.** Blocked on the IMAP
  library, not on this codebase.
- **The Exchange path described in the mail-at-scale plan.** The Microsoft work that shipped
  went through Graph for contacts, calendar and tasks. With EWS declined, this section
  proposes a path nothing needs.
- **JMAP.** Recorded as Future, Low in the requirements backlog.
- **Plugin and extension system.** Recorded as Future, Low.
- **Setting Wixen Mail as the actual Windows default mail client.** Windows does not allow a
  program to make itself default. `src/service/default_apps_registration.rs` registers the
  associations it can and the product already says plainly that it cannot set the default.

### Open Questions

Seven questions the inventory left unresolved on purpose, because the sources disagree and
the disagreement is itself information. None is scope; each needs a decision or a measurement
before it can become one.

1. **Is there an accessibility testing framework?** `docs/roadmap.md` leaves "Create
   accessibility testing framework" and "Accessibility compliance testing with screen
   readers" unticked. `docs/IMPLEMENTATION_STATUS.md` describes a workflow that drives a real
   copy of NVDA and checks what it said aloud, plus Axe.Windows and MSAA scans, and
   `.github/workflows/nvda.yml` and `accessibility.yml` both exist. The tree says one thing,
   the roadmap says another. The roadmap may mean the wide manual pass, which
   IMPLEMENTATION_STATUS separately says has not happened.
2. **Is the OAuth local callback server built?** `docs/roadmap.md` leaves it unticked.
   `src/service/oauth.rs` builds the redirect at line 330 and serves the request at line 630.
   The code contradicts the tick list.
3. **Can a calendar be added by its own address?** The roadmap says there is no screen for it.
   The changelog says a calendar can be added by its address, and
   `src/presentation/wx_add_calendar.rs` exists. The roadmap line looks stale.
4. **Is the S/MIME path complete?** Built and ticked, with a stated blind spot: mail that
   arrived before that version reads as unsigned, and mail collected over a path that does not
   keep the original form cannot be verified at all. The size of that blind spot was never
   settled.
5. **How much of the message list scales as designed?** The mail-at-scale plan says every step
   in its sequence is in the tree. Nothing measures either claim against a large mailbox, so
   "built" there is a code-reading result, not a behavioural one. Phase 9 of this milestone is
   the answer to this question.
6. **Which document is the debt ledger?** `.planning/codebase/CONCERNS.md` finds only two
   TODO or FIXME markers in a 259,723-line tree and concludes the changelog's Known
   limitations notes are the real ledger. Those notes are self-amending, so a limitation's
   current state can only be read by following its whole entry, not its heading.
7. **Are contacts, calendar and tasks writes safe on by default?**
   `src/application/allowed.rs` ships `personal_information` on for a new install while
   calling the same code "the least proven code in the application". Both statements sit in
   one file and both are true. Whether the default is right is a decision, and it is not
   settled anywhere in the documents.

## Context

**Brownfield, large, and honest about itself.** 259,723 lines under `src/`, 5,430 tests counted
2026-08-29, 501 guard records in `guards/guards.toml`, line coverage last measured at 60.4% on
2026-07-26. The project's own status documents are unusually candid and are the reason this
milestone could be scoped from evidence rather than guesswork.

**The largest and least verified surface.** `src/service/caldav.rs` at 8,149 lines is the
single largest service file and has never met a real server. Over 38,000 lines of sync logic
sit across five `*_sync.rs` files, none of it exercised end to end against a provider.
`src/presentation/wx_app.rs` at 19,659 lines is far larger than any other file in the
repository, which makes tracing a non-test path to every piece of logic harder exactly where
the project's own "done means it runs" rule needs it most.

**Writes are gated by design.** `src/application/allowed.rs` splits permission into two
answers: `mail` (sending, changing or deleting on the server) and `personal_information`
(tasks, contacts, calendar). Three places must all agree: the command line, the application
setting and the per-account setting. A new install gets `FOR_TESTING`, which allows personal
information writes and refuses mail writes. There is deliberately no flag that forces writes
on, only flags that restrict further. Anything this milestone adds that writes to a server
inherits that gate.

**Experimental status lives in the product, not in a chat message.**
`src/presentation/first_run.rs`, the Allowed Changes settings screen and the end of `--help`
all say the write paths are experimental, because a warning that only exists in a report is a
warning nobody gets.

## Constraints

- **Testing**: Red, green, refactor on every change. `workflow.tdd_mode` is `true` in
  `.planning/config.json`, so every eligible task is `type: tdd` with RED and GREEN gate
  commits. `.claude/guardrails/tdd-mode-check.js` runs at session start if the setting drifts
  off. The only exceptions are configuration-only files, documentation, glue code wiring
  already-tested components, and styling.
- **Definition of done**: A feature is done when a non-test path reaches it and it is
  exercised end to end. Compiling and green tests are not done. Run `dead-code-hunter` after
  finishing a feature.
- **Accessibility**: WCAG 2.2 Level AA across every disability category, verified with real
  assistive technology, not only automated scans. Automated scanning covers roughly half of
  WCAG. Windows has two channels and both must be right: UI Automation, which Narrator reads,
  and MSAA through `IAccessible`, which NVDA reads for native controls and is the only place
  `set_accessible_name` writes. `scripts/msaa-names.ps1` covers the second channel.
- **CI gate**: `bash scripts/check.sh` must stay green. It runs fmt, clippy with `-D
  warnings`, tests with `--no-fail-fast`, and a release build, touching `src/lib.rs` first to
  defeat stale cargo fingerprints. Never pipe it into anything whose exit status you then
  test. Never silence a lint with `#[allow(...)]` to get a commit through.
- **No live-account claim**: Nothing here has run against a real mail account. No success
  criterion in this milestone may claim otherwise. Where a phase's work can only be finished
  by a live account, that boundary is stated and gated rather than papered over.
- **Tech stack**: Rust 1.87, edition 2024. wxdragon pinned at exactly 0.9.17. tokio, rusqlite
  with bundled SQLite, async-imap 0.11.3, lettre, reqwest with rustls, keyring for the OS
  credential store. Windows-first; `wxAccessible` and `UiaRaiseNotificationEvent` exist only
  on Windows and silently do nothing elsewhere.
- **Schema changes are additive**: `MessageCache` opens existing user databases. Add tables
  with `CREATE TABLE IF NOT EXISTS` and columns with `ensure_column_exists`. Never drop or
  rename a column that shipped.
- **Secrets**: out of the tree and out of the database. OAuth client credentials load from a
  gitignored `oauth.toml`. Every other secret goes to the OS credential store through
  `keyring`. Nothing sensitive is written to `message_cache.db`. Never log a token, password
  or message body.
- **Writing**: Plain language. No em-dashes or en-dashes anywhere in the repository;
  `tests/house_style.rs` fails the build on them. No AI attribution in any commit, branch,
  comment or document.
- **Releases**: cut deliberately, never as a side effect of a push. Development happens on
  plain `0.x.y`.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| No EWS, Exchange goes through Microsoft Graph | Microsoft blocks third-party EWS from 1 October 2026, retired by April 2027 | Pending |
| Earcons play through rodio rather than a second per-platform FFI shim | One backend decodes WAV and OGG on every platform; the hand-rolled shim played WAV on Windows only | Good, phases 1 to 4 of the sound scheme plan are done and verified |
| Bundled sounds must be CC0 redistributable, Pixabay excluded | The Pixabay License forbids redistributing sounds as-is, which is exactly what bundling does | Good |
| Sound pack zip import treats the pack as untrusted input | Size caps, zip-slip refusal, decompression bomb caps, real audio parsing rather than trusting the extension | Good |
| Writes split into `mail` and `personal_information`, gated in three places | The two cost different amounts to get wrong, and a safety catch something else can quietly arm is not a safety catch | Pending, question 7 above |
| Message list stays native virtual mode, not wxdragon `VirtualList` or `DataViewCtrl` | Only the native control gives UI Automation the real set size, so a screen reader says "row 12 of 207,431" and means it | Good |
| Sorting and filtering happen in SQL, not in memory | Sorting 200,000 rows on a header click is a multi-second freeze, and a freeze is an accessibility failure | Good |
| egui plus AccessKit was rejected in favour of wxdragon | Native Win32 controls already expose a UI Automation tree; AccessKit would add a second provider to windows that have one | Superseded record, kept for how the decision was reached |
| The cached mail database is not encrypted, and the docs say so | Encrypting it means encrypting the whole database, a decision with a build cost | Pending, SHIP-04 in this milestone |

---
*Last updated: 2026-08-29 after ingesting the repository's own documents and mapping the codebase.*

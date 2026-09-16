# Phase 9: What the first day of testing found

Ten plans, one per wave, written 2026-09-16 against `main` at `524ff24f`,
version `0.125.1`, `guards/guards.toml` holding 825 records by the TOML
reader (census 802 + 23), `.planning/WINDOWS.md` at entry 482, 7,779 tests on
the last whole gate, CI green at the last push (`3e633252`) with 14 commits
unpushed since. Phase 8 closed on 2026-09-16 and every phase of the milestone
is executed and merged. The milestone is not archived: its verification is the
manual testing Pratik began on 2026-09-15 with installer `0.125.1+g3e633252`,
which produced 44 GitHub issues, #20 to #63, in one day.

**Planned from the issues, not from a research document.** That is new for
this project. Every issue was written from the code with file and line
evidence on the day, and several name their cause. Every file and line they
cite was treated as a premise and re-checked with a command on 2026-09-16,
because the tree moved after most were filed (08-07, 08-08 and 08-09 merged
afterwards). Each plan's `<premise_corrections>` carries the commands and
their output; the ones that moved are in the table below.

**Goal.** The next build carries the number it will ship as, and the twelve
defects the first day found with a known cause are fixed the way the tester
described them, test-first, so the second day of testing meets a different
program.

**Requirements:** FOUND-01 to FOUND-12, one per issue or per shared cause, in
`.planning/REQUIREMENTS.md` under "What the first day of testing found".

**Roadmap success criteria this phase owns:** all nine.

## Pratik's order, and which part of it this is

Pratik agreed an order for the 44 issues on 2026-09-16 (his word: "go"). It
has seven groups. This phase is the first two. The other five are later
phases, not planned here, and the next planner starts from this list rather
than from the issues again. The grouping of the later five is the planner's
reading of the order and the next planner confirms it with Pratik before
writing requirements.

| Group | What it is | Issues | Where |
|---|---|---|---|
| 1 | The version becomes `1.0.0-alpha.1`, first, so every fix lands under the number it will ship as | #46 | this phase, 09-01 |
| 2 | The cause-known defects, each an hour to a day | #21, #32, #36, #39, #42 with #40 point 5, #44, #51, #53, #56, #33, #34 | this phase, 09-02 to 09-10 |
| 3 | All the mail, and what is said while it comes | #20, #23, #24, #37, #38 | a later phase |
| 4 | Reading and the list | #25, #26, #27, #28 with #29, #30, #31, #62 | a later phase |
| 5 | The editors | #35, #40 points 1 to 4 and 6, #41, #43, #48 | a later phase |
| 6 | New features, most from the Outlook gap audit | #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61 | a later phase |
| 7 | The real-account issues, which need Pratik's account | #22, #63 | a later phase, and the first time anything here meets a real server |

Every issue in groups 3 to 7 is a requirement of some later phase. The
requirements section for this phase says so in one line, so nobody adds them
to `FOUND`.

## The plans

| Plan | Wave | Criterion | Issues | Closes or advances | What it does |
|---|---|---|---|---|---|
| 09-01 | 1 | 1 | #46 | closes, tree side | `Cargo.toml` to `1.0.0-alpha.1`; a pin naming the step; the installer's four-field version held above today's; a Release level that publishes the version as it stands; the rule for moving inside a prerelease in `CLAUDE.md`, the changelog, `BETA_RELEASE.md` and the release skill |
| 09-02 | 2 | 2 | #21, #32 | closes both | a stored bare language resolves to this machine's region and the screen shows what will be used; the snippet comes through the reading path's reader and stored ones are put right once |
| 09-03 | 3 | 3 | #44, #56 | closes both | Undo Send first on the Edit menu; a held meeting answer says the countdown and never "has been told" while held; the Alt+E comment names Alt+H |
| 09-04 | 4 | 4 | #36, #39 | closes both | Then by beside Default sort order, Cc and Bcc lines on the Compose tab; the Sort submenu one radio group |
| 09-05 | 5 | 5 | #42, #40 point 5 | closes #42, advances #40 | five editors as scan targets; the MSAA walk before and after; every checkbox named; the empty spacers gone; the label check widened |
| 09-06 | 6 | 6 | #33 | advances | a UI Automation event logger and the capture; the second event stopped; an NVDA case waiting for the next push |
| 09-07 | 7 | 7 | #51 | closes | one composition of the opened body, PGP finding, envelope and signature, asked by all six surfaces (the issue counted five; the whole conversation in the text reader is the sixth); the preview pane with a bar; the guard names every surface |
| 09-08 | 8 | 8 | #53 | advances, points 1 and 2 | the Outlook data file reader wired into the picker, each message down the path a saved `.eml` takes; Save As writes the list's message, and the reader keeps its own Save Attachment |
| 09-09 | 9 | 9 | #34 | closes | Settings measured, the cost paid once, measured again |
| 09-10 | 10 | 8 | #53 | advances, points 3 and 7 | a folder picker on its own File item, the Thunderbird layout said as a limitation, the four documents corrected by dating, a guide section; the phase's closing read |

Requirement coverage: FOUND-01 by 09-01; FOUND-02 and FOUND-03 by 09-02;
FOUND-04 and FOUND-05 by 09-03; FOUND-06 and FOUND-07 by 09-04; FOUND-08 by
09-05; FOUND-09 by 09-06; FOUND-10 by 09-07; FOUND-11 by 09-08 and 09-10;
FOUND-12 by 09-09.

Each plan ends with the `gh issue close` or `gh issue comment` the executor
runs after the merge, quoting the merge commit, so the tracker and the tree
agree. Closing an issue is not a publish and the executor may do it; filing
or editing other issues is not theirs.

## Why ten plans, and why one per wave

Ten because the twelve issues fall into ten pieces that do not share a
shape or a file set: the version alone; two stored-value fixes; two things
about Undo Send; two things about sort; the checkbox class with its scan
targets; the tab-row diagnosis; the reading path; the import reader and the
Save As writer; the measurement; and the folder picker with the documents,
which the plan check cut out of the import plan on 2026-09-16 because
sixteen files in one plan was the widest in the phase and the documents
share no code with the reader. Each is two or three tasks.

One per wave because every plan writes `docs/changelog.md` and eight of the
ten write `guards/guards.toml`, and a wave is a set of plans sharing no file.
The order is Pratik's suggestion made concrete: the version first, so every
entry that follows is written under it; then the pure-logic pair, which proves
the flow on the cheapest ground; then the windows; then the two that touch
reading and importing paths; then the one that measures before it changes,
because its measurement must be of the tree the code plans left; then the
documents last, because they describe what the others built, and the
phase's closing read goes with them.

## How versions move under the alpha

This is the rule 09-01 writes into `CLAUDE.md`, the changelog's opening
paragraph, `docs/BETA_RELEASE.md` and `.claude/skills/cutting-a-release/SKILL.md`,
and the rule every later plan in this phase follows. It was written against
cargo-release's reference, read 2026-09-16.

- **The tree's version is the next build to go to testers.** 09-01 moves it
  to `1.0.0-alpha.1` by hand, once. It stays there through this phase's
  fixes, because that build has not been cut and the number already names
  something newer than any build handed out. No plan here bumps it.
- **After a build is cut, the first behaviour change moves the prerelease
  counter, once, in the commit that makes it:** `1.0.0-alpha.2`. Later
  behaviour changes before the next cut do not move it again, because the
  version is a name for the next build and not a count of commits, which is
  the mistake the twenty-five `0.1.0-alpha.N` versions were. Documents and
  bug fixes that change no behaviour move nothing, as before.
- **A hand build between cuts carries `+g<commit>`** as it always has, which
  is how two builds of one version are told apart.
- **The Release workflow's `as-is` level, added by 09-01, publishes the
  version the tree carries** without bumping it first. Without it, the
  workflow's `alpha` level on a tree at `1.0.0-alpha.1` would publish
  `-alpha.2` and nothing could ever publish `-alpha.1`. `alpha`, `beta` and
  `rc` stay for advancing a stage from the workflow when Pratik prefers it;
  `release` drops the suffix to `1.0.0` when the round closes.
- **`patch` is never dispatched on a prerelease.** cargo-release reads `patch`
  on `1.0.0-alpha.N` as "remove the suffix", which publishes `1.0.0` as a full
  release. The skill says so.
- **Whether the first alpha is published is a dispatch and is Pratik's.** This
  phase bumps the tree and makes the dispatch possible; it publishes nothing.

`CLAUDE.md`'s "Bump when the software changes, not when a build changes hands"
still holds: the bump happens on the first behaviour change after a cut, in
that commit, and the build is what the version was already named for.

## What the tree contradicted in the issues

Every file and line the eleven issues cite was re-checked on 2026-09-16 at
`524ff24f`. Most held, with lines moved by a few tens. Five premises moved
in kind:

| Issue | The issue says | The tree says | Command |
|---|---|---|---|
| #21 | one of two causes: `en-US` not found available, or the family fallback taking Windows' first | neither, on this machine: the pure path answers `en-US`. A stored bare `en` (the default for everybody until 2026-09-03) produces both symptoms, through `unwrap_or(0)` on the screen and `find_regional_variant` in the checker | a scratch test printing `system_language()`, `what_this_machine_offers()` and `language_to_check_in(...)`, then `git checkout`; `git log -S'language_to_check_in' -- src/data/config.rs` |
| #33, #34 | five settings pages | seven | `grep -c 'notebook.add_page' src/presentation/wx_settings.rs` |
| #42 | the account manager's checkboxes are built the same way and go on the same fix | their spacers are the same; their names are set already through `set_accessible_name` in the `cb` closures | `sed -n 1466,1478p src/presentation/wx_account_manager.rs` |
| #46 | the first-run screen and `--help` name a version shape | neither does; "alpha" there is a state | `grep -n 'alpha\|version' src/presentation/first_run.rs src/presentation/command_line.rs` |
| #46 | a hand bump or a `major` level | a hand bump, and also a level that tags the version as it stands, because the workflow bumps before it tags and could otherwise never publish `-alpha.1` | `grep -n 'cargo release' .github/workflows/release.yml`; cargo-release's reference |
| #56 | a sentence says "has been told" too early | and the variant it lands on, `HowItWent::Sent`, is documented as "reached the organiser's mail server" while the answer only entered the hold; the calendar is filed from the same variant | `sed -n 335,343p src/application/answering.rs`; `wx_app.rs:13129-13170` |
| #32 | `strip_markup` has one caller, the snippet | two: the search index takes an HTML-only body through it too (`searching.rs:360`), so the index holds stylesheets and a search for `padding` finds newsletters | `grep -rn strip_markup src --include='*.rs'` |
| #51 | five surfaces show a message | six: `open_conversation` (`wx_app.rs:12577`), the whole thread in the text reader, has no bar and no PGP opening either | `grep -n 'fn open_conversation(' src/presentation/wx_app.rs`; `wx_app.rs:20200` |
| #53 | `written_as_one_message` is reached by the exporter only | and by the `.pst` reader itself at `outlook_data_file.rs:1701`, which composes each `Mail` item through it, so an imported message takes the path a saved `.eml` takes | `grep -rn written_as_one_message src --include='*.rs'` |
| #53 | Save As saves "a message or attachment" | the attachment list is in the reader frame, which has its own Save Attachment (`wx_reader.rs:35`, `:794`, `:878`) and whose focus makes the main menu inactive; Save As is the list's message | `grep -n 'ID_SAVE_ATTACHMENT\|fn save_attachment_now' src/presentation/wx_reader.rs` |
| #21 | the tester's profile cannot be read from here | this machine's profile can (`"language": "en"`, dated 2026-07-30) and is the stored shape the cause needs; the tester's profile is elsewhere (this machine has no 2026-09-16 log and no running process while he has tested for two days), was created by an earlier build and held the same, and he has since set English (United States) by hand | `grep -o '"language": *"[^"]*"' "$LOCALAPPDATA/wixen-mail/config/app_config.json"` |

The last five rows were found by the plan check on 2026-09-16, after the
first draft of these plans; each plan's `<premise_corrections>` says so
where it was wrong.

One decision the issues left open is made here and can be overruled with a
reason in the summary: **#53's `.pst` reader is wired, not retired.** Both
costs are in 09-08's premises. The reader already yields the cache's own
entry types, `import_tree::what_was_chosen` is the one seam the worker
dispatches on, the crate paragraph's stated risk is the one the archive reader
already carries, and `CLAUDE.md` answers "what does it replace" with Outlook.
Retiring would delete 3,608 lines whose whole reason is that answer.

## Costs every plan is written around

**Guard records, by the TOML reader on 2026-09-16, 825 in all.** Files the
plans touch, with records naming them and `#[test]` lines in them:
`src/presentation/wx_app.rs` 50 and 196 (`cargo test --lib presentation::wx_app::`
runs 199; the `#[test]` grep misses the `tokio::test` lines);
`src/service/spellcheck/mod.rs` 30 and 56 (the filter runs 65, because
`windows_speller.rs`'s 9 sit under the same path); `tests/house_style.rs`
25 and 83; `src/application/long_text.rs` 18 and 59; `tests/wired.rs` 14
and 72; `src/data/config.rs` 10 and 66; `src/data/message_cache/searching.rs`
3 and 25;
`src/presentation/reader_text.rs` 10 and 120; `src/presentation/wx_settings.rs`
8 and 0; `src/presentation/wx_managers.rs` 7 and 44;
`src/presentation/wx_account_manager.rs` 6 and 14; `tests/installer.rs` 4 and
13; `src/data/message_cache/bodies.rs` 4 and 43; `src/common/started.rs` 3
and 6; `src/common/version.rs` 2 and 24; `src/presentation/scan_target.rs` 2
and 11; `tests/no_label_is_only_a_space.rs` 1 and 4; `src/application/invitations.rs`,
`src/service/outlook_data_file.rs`, `src/service/mailbox_archive.rs` and
`src/service/fonts.rs` 0 each. A test added to a file flags every record
naming it, at the rate on `docs/development/measurements.md` each; so no plan
adds a test to `house_style.rs` or to `wx_app.rs`, the new readings go in new
integration targets at zero records, and each new target gets a record whose
`suite` names it so `check.sh --suites-for` couples it. Every plan quotes
`--suites-for` afterwards and reports each touched file's test count before
and after.

**The known gate-mapper holes**, so each plan says what reaches what:
`guards/guards.toml`, `docs/*.md`, `Cargo.toml`, `Cargo.lock`, `locales/` and
`scripts/*.ps1` map to no scoped target (`docs/*.md` is reached by the
document-reading targets on a documents-only commit and by the whole-tree
guards on a code commit; `guards.toml` by seven `house_style` tests on every
commit); `src/main.rs` and `src/presentation/wx_settings.rs` map to `--lib`
filters matching nothing (`wx_settings.rs` is reached through the coupled
targets `every_event_has_a_control` and `checkbox_labels`, and after 09-02
the language target); `.github/workflows/` answers `all`; `nvda-tests/` is
JavaScript reached by `nvda.yml` on a push.

**Tools that are broken, and what to do instead.** `gsd-tools roadmap
update-plan-progress` counts a README as a plan: edit `ROADMAP.md` by hand
and read the diff. `gsd-tools windows append` corrupts an entry holding a
backslash and has no edit: write ledger entries by hand, both halves, and
`test_both_halves_of_the_ledger_say_the_same_thing` holds them. `gsd-tools
query commit` cuts the hook off: use `git commit`. `cargo test` takes one
`--lib`: several module paths are several invocations joined with `&&`, which
every `<verify>` here does.

**The recurring findings from earlier executors**, carried into every plan
rather than restated per task:

- Trace an absence claim rather than accepting it; the issues' own line
  numbers moved within a day.
- A guard record goes stale inside its own plan; run the `--remeasure`
  remedy whenever the count check prints it, detached, and read the result
  before committing.
- Running `scripts/check.sh affected` by hand runs only the tree guards; the
  hook runs the scoped tests. Never pipe `check.sh` into anything.
- Do not write a count you have not just taken. Every figure in these plans
  is dated 2026-09-16 and every task re-takes what it quotes.
- Red trailers name lib tests by module path and integration tests bare, as
  `scripts/red-commit.sh` reads cargo's lines; a red commit lands on a branch,
  never on `main`.
- `WIXEN_TEST_THREADS` stays untouched.
- Every user-visible change gets a changelog entry under `[Unreleased]` in
  the same commit; no plan here bumps the version, per the section above.
- Measure carriage returns with `tr -cd '\r' | wc -c`, never grep. No em dash
  anywhere, `.planning` included; `tests/house_style.rs` reads it. No
  scripted rewrite of a tracked file: Read, then Edit or Write.
- The MSAA walk crashes PowerShell on this machine, ledger 390: "Walking any
  Wixen Mail window over MSAA crashes PowerShell on this machine with
  STATUS_STACK_BUFFER_OVERRUN ... NVDA is running here and CI has no screen
  reader ... Not diagnosed." NVDA running is a difference between the
  machines, not a diagnosed cause, and stopping it on a screen reader user's
  machine is not asked for; a plan that needs the walk runs it once as the
  machine is, and if it crashes, waits for CI and says so.
- `cargo test`'s positional filters after `--` are ORed by libtest: `--lib
  presentation::wx_app:: -- sort` runs 240 tests, not the sort tests. One
  filter per invocation, the module's own path.

## What only a person can settle

Each requirement's last `[S]` line names it, and none of it is claimed by
any plan: whether the tester's own profile now shows English (United States)
and reads snippets as words (#21, #32); whether the Reading tab reads as one
group by ear (#36); what NVDA says on the signature and contact editors after
09-05 (#42, #40); whether each Settings tab is heard once, which the NVDA
case on CI answers at the next push and the tester's ear answers after
(#33); whether Settings feels at once (#34); ledger 155's question about
Undo Send after a mistaken Accept (#56); and everything about a real
correspondent's key, a real signed message and a real Outlook data file (#51,
#53). `docs/manual-accessibility-pass.md` gains one line from 09-06 and is
otherwise the list it was.

## Estimates, and the factor behind them

`raw_tokens` is 30,000 per work task, the projection shape phases 6 to 8
used. `tokens` is that multiplied by **0.216**, the mean of
`actuals.tokens / estimate.raw_tokens` over the nine landed plans of phase
8, read 2026-09-16 from each plan's `estimate` block and each summary's
`actuals` block: 0.133, 0.243, 0.228, 0.267, 0.078, 0.117, 0.398, 0.321 and
0.158. The spread is five times, so `low`, derived from the sample and not
self-rated. Phase 8's README gave 0.25 over the eighteen plans of phases 6
and 7; phase 8 ran under it. `gsd-tools query estimate-calibration` answers
`factor: 1, sample_count: 0` on this project, so the factor is taken by hand
from the files and said here.

## What is owed to documents, and who does it

1. **The roadmap's phase 9 entry and progress row.** Done by the planner in
   the commit that lands these plans: the goal, the twelve requirements, nine
   criteria, the plan list, the row at `0/10`, and the milestone paragraph
   corrected to say the milestone continues with what testing found.
   `test_the_roadmap_counts_the_files_that_are_on_disk` holds the row to the
   files.
2. **`.planning/REQUIREMENTS.md`.** Done by the planner: the `FOUND` section,
   the twelve traceability rows, the coverage count re-taken at 56, the
   provenance note, and the milestone line corrected.
3. **`.planning/STATE.md`.** Done by the planner in the same commit, by hand:
   phase 9 current, plan 1 of 10, `Total Plans in Phase: 10`,
   `progress.total_plans` counted from the disk.
4. **`docs/changelog.md`.** Every plan writes its entries under
   `[Unreleased]`; 09-01 corrects the opening paragraph; 09-07 and 09-08 date
   older entries.
5. **The requirement ticks.** No plan ticks a `FOUND` requirement because its
   own tasks finished; the last plan's summary reads each clause by clause and
   the phase closes them, on 08-09's pattern.
6. **The ledger.** Each plan adds entries by hand for what it could not
   settle, both halves.

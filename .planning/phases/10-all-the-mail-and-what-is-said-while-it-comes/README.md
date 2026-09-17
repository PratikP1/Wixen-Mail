# Phase 10: All the mail, and what is said while it comes

Seven plans, one per wave, written 2026-09-17 against `main` at `7d57cd49`,
version `1.0.0-alpha.1`, `guards/guards.toml` holding 868 records by the
TOML reader (census 802 + 66 at `guards.toml:84`), `.planning/WINDOWS.md` at
entry 511, 7,872 tests on the last whole gate, CI green at the last push
(`3e633252`) with 91 commits unpushed since. Phase 9 closed on 2026-09-17
with all ten plans merged. This is the third of the seven groups Pratik
agreed on 2026-09-16, and the first phase in which every plan touches
`src/presentation/wx_app.rs`, which is why the order below is what it is.

**An eighth plan was inserted later the same day, after 10-01 merged at
`d8e887d6`.** 10-01.1 fixes two regressions of 09-09 the tester found in
`1.0.0-alpha.1`: every Settings checkbox after General reads as a button
under NVDA (#67), and Ctrl+Tab from inside a page lands on the empty page
panel (#68). It runs before 10-02 because 10-03 and 10-04 add controls to
the Permissions and Feedback tabs and their screen-reader checks should be of
pages whose checkboxes are checkboxes. The six plans after it moved up one
wave each; 10-02 and 10-07 depend on it. Its requirements are FOUND-13 and
FOUND-14, under phase 9's section of `.planning/REQUIREMENTS.md`, beside the
FOUND-12 fix they correct.

**Planned from five issues, each with the tester's words on it.** #20 (only
500 messages come down per folder), #24 (the list shows only the newest 500
and older ones fall off), #23 (message text is not downloaded unless asked),
#37 (automatic fetching stops after a few hours), #38 (every progress line is
spoken; the tester added: make how much is said a setting). The five share
one download and one progress story, which is why they are one phase. Every
file and line they cite was re-run with a command on 2026-09-17 and quoted in
each plan's `<premise_corrections>`; the ones that moved are in the table
below.

**Goal.** Every message of every kept folder comes down on its own after a
check, with its text unless forbidden or bounded, and stays on the list; mail
keeps arriving for as long as the program runs; and how much is said while
that happens is the person's choice, with what arrived said once and errors
always.

**Requirements:** MAIL-01 to MAIL-05, one per issue, in
`.planning/REQUIREMENTS.md` under "All the mail, and what is said while it
comes"; and FOUND-13 and FOUND-14 for the inserted 10-01.1, under "What the
first day of testing found".

**Roadmap success criteria this phase owns:** all eight, six for the mail
and two for the inserted plan.

## Pratik's order, and which part of it this is

The seven groups of 2026-09-16, from phase 9's README. Groups 1 and 2 were
phase 9. This phase is group 3. The next planner starts from this table and
confirms the grouping of 4 to 7 with Pratik before writing requirements.

| Group | What it is | Issues | Where |
|---|---|---|---|
| 1 | The version becomes `1.0.0-alpha.1` | #46 | phase 9, done |
| 2 | The cause-known defects, each an hour to a day | #21, #32, #36, #39, #42 with #40 point 5, #44, #51, #53, #56, #33, #34 | phase 9, done; #33, #40, #42 and #53 advanced and open |
| 3 | All the mail, and what is said while it comes | #20, #23, #24, #37, #38 | this phase |
| 4 | Reading and the list | #25, #26, #27, #28 with #29, #30, #31, #62 | a later phase; 10-07 writes #29's one line about a whole-mailbox download on the privacy page and leaves the rest |
| 5 | The editors | #35, #40 points 1 to 4 and 6, #41, #43, #48 | a later phase |
| 6 | New features, most from the Outlook gap audit | #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61 | a later phase |
| 7 | The real-account issues, which need Pratik's account | #22, #63 | a later phase, and the first time anything here meets a real server on purpose; this phase's runner and watch will have met his Gmail account by then, unasked |

Two issues filed on 2026-09-17 against `1.0.0-alpha.1` belong to none of the
seven groups: #67 and #68, both regressions of 09-09, taken by the inserted
10-01.1 above. One thing that plan found and left: four other windows
(`wx_account_manager.rs`, `wx_compose.rs`, `wx_item_form.rs`,
`wx_managers.rs`) both paint themselves with the theme and build checkboxes,
and none has been read for whether the paint comes before or after the
build; a checkbox created under an already painted panel reads as a button
under NVDA. A reading across every window that no built `wxCheckBox` is
owner-drawn is work for a later phase, recorded in the ledger by 10-01.1 and
in the changelog's Known limitations.

## The plans

| Plan | Wave | Criterion | Issues | Closes or advances | What it does |
|---|---|---|---|---|---|
| 10-01 | 1 | 1, 3, 4 | #20, #23, #37 | advances | the model: what "everything" means for one account, decided from the cache and nothing else; one wait rule for a server that refused, shared by the download and the watch; the text pass bounded per chunk with the provider's answer read and a stop; all against the scripted mailbox |
| 10-01.1 | 2 | 7, 8 | #67, #68 | closes both | inserted 2026-09-17 after 10-01 merged: the six later Settings panels painted at the end of their own build so every checkbox stays a check box, held by an MSAA reading in two themes; a page reached from inside another page hands focus to its first control, held by a focus reading; both coupled to `wx_settings.rs` by measured records; `theme_reach` reads the later panels after they are built |
| 10-02 | 3 | 2 | #24 | closes | the list's own read path measured at 12,872 and 200,000 rows before anything changes; the page taken off the folder list and All Inboxes; the labels read by folder because the old read fails above 32,766 rows; measured again |
| 10-03 | 4 | 3 | #23 | advances | how much message text stays on this computer is a setting under Message Text on the Permissions tab, default all of it; the eviction reads it; the workers that evict are handed it |
| 10-04 | 5 | 5 | #38 | closes | the three-level setting on the Feedback tab; progress shown and spoken only under Say every step; what arrived said once with counts; errors always; the new-mail sound when mail was found; Settings saved heard |
| 10-05 | 6 | 1, 3 | #20, #23 | closes both | the download runs after every check for every enabled IMAP account, chunk by chunk, the folder on screen first, text with it, resumable, with Pause Downloading on Tools and a wait after a refusal served by the timer; Download This Whole Folder and Fetch Missing Message Text retired with their warnings |
| 10-06 | 7 | 4 | #37 | closes | the watch restarts after a growing wait, the network coming back restarts it at once, every account has one, a check runs on the account editor's interval where a watch cannot cover, a start checks, the status line says which |
| 10-07 | 8 | 6 | all five, #29 one line | closes the phase | the alpha, privacy and guide pages say what the program does now; the listening lines; the closing read; the ledger |

Requirement coverage: MAIL-01 by 10-01 and 10-05; MAIL-02 by 10-02; MAIL-03
by 10-01, 10-03 and 10-05; MAIL-04 by 10-01 and 10-06; MAIL-05 by 10-04;
10-07 reads all five. FOUND-13 and FOUND-14 by 10-01.1, which ticks them
itself: they are one plan each and 10-07's closing read is about the mail.

Each plan ends with the `gh issue close` or `gh issue comment` the executor
runs after the merge, quoting the merge commit. Closing an issue is not a
publish and the executor may do it; filing or editing other issues is not
theirs.

## Why seven plans, and why this order

Seven because the work falls into a model, a list, a setting, a routing
change, a runner, a watch and a closing read, and each is two or three tasks
touching files the others do not, except `wx_app.rs`, which five of them
touch and which is why there is one plan per wave. The order is the
dependency order and one deliberate choice inside it:

- **The model first** (10-01), because every decision the runner and the
  watch make is tested there in milliseconds and nowhere else can be: a test
  added to `wx_app.rs` costs 57 guard records at 92 s each, about 87 minutes,
  before it is even run.
- **The list second** (10-02), because it is independent of the download and
  cheaper, and because the download would otherwise bring mail into a list
  that hides it.
- **The text budget third** (10-03), because the eviction at half a gigabyte
  would undo the download on any mailbox with more text than that, and the
  runner needs the setting to exist before it can read it.
- **What is said fourth** (10-04), before the runner, so the runner's
  progress lines are shown and not spoken from the first commit that sends
  one. The other order would have the runner speak every chunk for one wave.
- **The runner fifth** (10-05) and **the watch sixth** (10-06), because the
  watch's schedule and network restart end by starting a check, and a check
  now ends by starting the download.
- **The documents last** (10-07), because they describe what the others
  built, and the closing read goes with them.

## Decisions made here, each overrulable with a reason in a summary

1. **The check interval is the account editor's field, made true; no second
   interval on the Settings screen.** #37 asks for "the interval a setting
   reachable from the settings screen". The tree already has one, per
   account, on the account editor ("Check &Interval (min):", stored on the
   account row, default 5, clamped 1 to 60), offered since it was written and
   read by nothing (`grep -rn 'check_interval' src/presentation/wx_app.rs
   src/application/*.rs` finds nothing). 06-04 put the per-account Allow
   Changes boxes on the same editor. Two settings for one interval is the
   drift `CLAUDE.md`'s settings rule exists to stop, so 10-06 makes the field
   act and adds none. If Pratik wants one interval for every account on the
   Settings screen instead, that is one field and one control and the
   per-account one retires; say so in 10-06's summary.
2. **Reads follow the network; sends follow offline mode.** Offline mode's
   own two sentences are about the Outbox, `reachability_of` is asked only
   by the send paths, and guardrail 7 is about publishing. So the watch and
   the download resume when the network returns whether or not the person
   has pressed Go Back Online. The alternative, holding every fetch until
   the button is pressed, would leave the inbox stale for as long as
   somebody did not notice the offer, which is #37's symptom by another
   route.
3. **How much text stays is a setting, default all, rather than the eviction
   being retired.** #23 says all of it; the eviction's constant exists so a
   mailbox cannot quietly fill a disk. A choice on the same screen as the
   forbid box keeps both true: all by default, a size one choice away.
4. **Get Older Messages stays; Download This Whole Folder and Fetch Missing
   Message Text go.** The first has six files and three readings behind it
   and a meaning that survives ("carry on downloading, this folder first");
   the other two are what the download does by default, and their warnings
   fold into one sentence on the Pause item. The offer panel above the list
   goes with the text command, because under a bounded budget it would offer
   to fetch beyond the bound.
5. **The new-mail sound plays when a check found mail, not when the watch
   woke.** It fires in the `MailboxChanged` arm today, before the folder is
   re-read and whether or not anything arrives, and never on a check started
   by F9. #38's "the earcon that already says so" assumes the other meaning,
   so 10-04 gives it that meaning.
6. **Chunk sizes and the wait's numbers are decisions of 2026-09-17, not
   measurements.** 500 headers a chunk (the existing constant), 50 messages
   or 16 MiB of text a chunk, three refusals in a row meaning the server has
   stopped, a wait of 30 seconds doubling to a cap of 30 minutes. Each is on
   its constant with the sentence that a provider observed doing something
   else would move it, and no provider has been observed.

## What the tree contradicted in the issues

Every file and line the five issues cite was re-checked on 2026-09-17 at
`7d57cd49`. The line numbers moved by three to forty since the issues were
filed (09-03 to 09-10 edited `wx_app.rs`); the shapes held. Six premises
moved in kind:

| Issue | The issue says | The tree says | Command |
|---|---|---|---|
| #37 | automatic fetching stops after a few hours | and it never starts until the first F9: the only thing that requests a watch is the end of a check, and nothing checks at startup | `grep -n 'MailboxWatchRequested' src/presentation/wx_app.rs` -> 18288 (the arm), 21975 (sent at the end of `spawn_mail_sync`, nowhere else); `grep -n 'spawn_mail_sync(' src/presentation/wx_app.rs | grep -v 'fn '` -> 4489, 4614, 18302, none at startup |
| #37 | the interval should be a setting reachable from the settings screen | the account editor has offered "Check Interval (min)" since it was written, the row stores it, and nothing reads it; `Account::mark_synced` writes `last_sync` and is called by its own test only | `grep -rn 'check_interval_minutes' src --include='*.rs'` -> stored in `accounts.rs`, built and read in `wx_account_manager.rs:1244,1329,1624,1830`, read by no check; `grep -rn 'mark_synced' src` -> `account.rs:326` and `:643` |
| #37 | the 29-minute IDLE window ending is one way the watch ends | it is not: a timeout re-issues IDLE in the same loop (`Timeout` -> `StillWatching`); a dead socket is noticed at the next window at the latest, so the longest silence is 29 minutes and the schedule is what covers it | `sed -n 1793,1815p src/service/protocols/imap.rs`; `sed -n 1856,1859p` |
| #38 | the new-mail earcon "already says so" | it fires when the watch wakes, before the folder is re-read and whether or not anything arrives, and never on a check started by F9 | `grep -n 'a11y.signal(FeedbackEvent::NewMail' src/presentation/wx_app.rs` -> 18299, in the `MailboxChanged` arm |
| #24 | 08-04 measured the list at 200,000 rows, listing in 371 ms | it measured `get_messages_for_folder`, unsorted and unbounded; the window reads `get_message_list_sorted` with a `LIMIT`, then threads every row and reads the labels with one bound parameter per row, on the interface thread, and none of those three has a row; the labels read is expected to fail above 32,766 rows on the bundled SQLite | `grep -n 'get_messages_for_folder\|get_message_list_sorted' tests/the_list_at_two_hundred_thousand_rows.rs` -> 213, 231 only; `sed -n 229,240p src/data/message_cache/tags.rs`; `grep -A1 'name = "libsqlite3-sys"' Cargo.lock` -> 0.38.1 |
| #23 | the setting that forbids fetching is the way to say no | and a second bound the issue does not name says no to keeping: the body cache evicts above 512 MiB at the end of every folder sync, least recently read first, so a download of every message's text would be undone by the next folder's sync on any mailbox with more text than that | `grep -n 'pub const BODY_CACHE_BUDGET_BYTES' src/data/message_cache/bodies.rs` -> 285; `grep -n 'keep_bodies_within_budget' src/application/mail_sync.rs` -> 1467 |
| #20, #23 | the text fetch is "bounded per chunk with the provider's answer read" is what a fix needs | the text pass is one message at a time, unbounded, with no stop but the reading gate, and a refusal is counted per message and never read as the server's answer; POP already brings everything down with its text in one pass | `sed -n 1858,1930p src/application/mail_sync.rs`; `sed -n 128,140p src/application/pop_sync.rs` |
| #37 | the handler's comment at `:19760` says nothing starts another watch | true at `:20126`; and the doc comments for `spawn_mail_watch` and `say_the_watch_is_off` are stacked above `fn say_the_link_was_refused`, which carries three, while the two functions carry none | `awk 'NR>=19985 && NR<=20034' src/presentation/wx_app.rs | grep -n '^///\|^fn '` |

## Costs every plan is written around

**Guard records, by the TOML reader on 2026-09-17, 868 in all.** Files the
plans touch, with records naming them in `tests_last_seen` and the test count
those records carry: `src/presentation/wx_app.rs` 57 and 199;
`src/service/protocols/imap.rs` 46 and 102 (touched by nothing here);
`tests/house_style.rs` 25 and 74 (no test added); `src/application/mail_controller.rs`
23 and 71; `src/data/message_cache/messages.rs` 22 and 179; `tests/wired.rs`
18 and 77; `src/presentation/wx_settings.rs` 15 and 0; `src/data/config.rs`
11 and 68; `src/service/outward.rs` 10 and 38; `src/application/mail_sync.rs`
9 and 135; `src/data/message_cache/bodies.rs` 8 and 47;
`src/presentation/accessibility.rs` 7 and 23; `src/presentation/wx_account_manager.rs`
6 and 14; `src/application/allowed.rs` 4 and 27; `tests/the_numbers_the_targets_ask_for.rs`
3 and 16; `src/presentation/accessibility/feedback.rs` 3 and 49;
`src/application/pop_sync.rs` 3 and 49; `tests/the_list_reads_only_memory.rs`
2 and 6; `tests/a_whole_folder_moves_both_bounds.rs` 1 and 7;
`tests/the_list_at_two_hundred_thousand_rows.rs` 1 and 6;
`src/application/the_network_coming_and_going.rs` 1 and 10;
`src/application/asking_for_a_whole_folder.rs`, `src/presentation/accessibility/announcements.rs`,
`src/service/network.rs`, `src/data/account.rs` and `src/application/reading_habits.rs`
0 each. The rate is the row on `docs/development/measurements.md`, 92 s a
record on 2026-09-14. So: **no plan adds or removes a test in `wx_app.rs`**;
its five tests that read the retired text-fetch command are rewritten in
place, five for five, and every plan quotes the count check's count before
and after. New readings go in new integration targets at zero records, each
with a record whose `suite` names it so `check.sh --suites-for` couples it.

**The known gate-mapper holes**, so each plan says what reaches what:
`guards/guards.toml`, `docs/*.md`, `Cargo.toml`, `Cargo.lock`, `locales/` and
`scripts/*.ps1` map to no scoped target (`docs/*.md` is reached by the nine
document-reading targets on a documents-only commit and by the whole-tree
guards on a code commit; `guards.toml` by seven `house_style` tests on every
commit); `src/main.rs` and `src/presentation/wx_settings.rs` map to `--lib`
filters matching nothing (`wx_settings.rs` is reached through its six coupled
targets); `src/application/mod.rs` changing runs the whole application layer;
`check.sh --suites-for` prints nothing for a target already in
`guards_that_read_the_whole_tree` (ledger 442); `cargo test` takes one `--lib`
and several module paths are several invocations joined with `&&`, which every
`<verify>` here does; libtest ORs positional filters after `--`, so no
`<verify>` uses one.

**Tools that are broken, and what to do instead.** `gsd-tools roadmap
update-plan-progress` counts a README as a plan: edit `ROADMAP.md` by hand and
read the diff. `gsd-tools windows append` corrupts an entry holding a
backslash and has no edit: write ledger entries by hand, both halves, and
`test_both_halves_of_the_ledger_say_the_same_thing` holds them. `gsd-tools
query commit` cuts the hook off: use `git commit`. `gsd-tools query
estimate-calibration` answers factor 1 with no samples: the factor below is
taken by hand.

**The recurring findings from earlier executors**, carried into every plan
rather than restated per task:

- Trace an absence claim rather than accepting it; the issues' line numbers
  moved within days, and 09-10 found a menu letter taken that a plan assumed
  free.
- A guard record goes stale inside its own plan; run the `--remeasure`
  remedy whenever the count check prints it, detached, and read it before
  committing. A first draft of a record's red list is a prediction; the
  runner's answer is the record.
- Running `scripts/check.sh affected` by hand runs only the tree guards; the
  hook runs the scoped tests. Never pipe `check.sh` into anything; write its
  output to a file and read the exit status directly.
- Do not write a count you have not just taken. Every figure in these plans
  is dated 2026-09-17 and every task re-takes what it quotes.
- Red trailers name lib tests by module path and integration tests bare, as
  `scripts/red-commit.sh` reads cargo's lines; a red commit lands on a branch,
  never on `main`; a count check that will fire is named in the same trailer.
- `WIXEN_TEST_THREADS` stays untouched.
- Every user-visible change gets a changelog entry under `[Unreleased]` in
  the same commit; no plan here bumps the version, because no build has been
  cut since `1.0.0-alpha.1` was set and the rule in `CLAUDE.md` moves the
  counter only after a cut.
- Measure carriage returns with `tr -cd '\r' | wc -c`, never grep. No em dash
  anywhere, `.planning` included; `tests/house_style.rs` reads it. None of the
  six words; `the_words_that_say_nothing` reads it. **No scripted rewrite of
  a tracked file: Read, then Edit or Write.** 09-06 broke this rule twice with
  a Python one-liner and the file came back CRLF; the exception set for this
  phase is zero, and each summary says so.
- `git commit` with the message in a file passed by `-F`; `gh` from the
  repository root.
- The MSAA walk crashes PowerShell on this machine (ledger 390); no plan here
  needs it.
- bash and PowerShell started from this harness read a stale July copy of
  `%LOCALAPPDATA%\wixen-mail` at that path. The tester's live profile (one
  Gmail account, 12,872 messages, 50 folders, `mark_read_after` never,
  `log_level` error, read 2026-09-16 through Python) is read only through a
  Python process, only for a fact a plan needs, and no plan here needs one:
  the number 12,872 is quoted from that reading. The release binary is
  started only against a temp folder pinned with `WIXEN_MAIL_DATA`, as
  08-03's and 09-09's harnesses do, never against his profile.
- A test added to a file a record names is a remedy; a test removed is the
  same remedy. Rewrite in place where the count matters.

## What only a person or a real provider can settle

Each requirement's last `[S]` line names it, and none of it is claimed by any
plan. The tester's Gmail account is the only provider anything here will
meet, and it will meet the runner and the watch unasked on the first check
after the build: whether Gmail tolerates 12,872 messages and their text in
chunks of 50 (ledger 11 and 72 stay open); whether it drops an IDLE
connection, after how long, and whether the restart carries mail over hours
(64, 65); whether two connections per account are welcome (67); whether his
folder reads as one list of 12,872 with his screen reader; what a check of his
50 folders sounds like under each of the three levels, and which sentence is
a step and which a result by ear (10, 73, 78); whether the status line's
three states read as states. Everything a loopback server can prove, the
plans prove: the chunking, the stop, the wait, the schedule, the restart
decision, the list at 200,000, the eviction under All, the kinds of line.

## Estimates, and the factor behind them

`raw_tokens` is 30,000 per work task, the projection shape phases 6 to 9
used. `tokens` is that multiplied by **0.30**, the mean of `actuals.tokens /
estimate.raw_tokens` over the ten landed plans of phase 9, read 2026-09-17
from each plan's `estimate` block and each summary's `actuals` block: 0.106,
0.250, 0.257, 0.159, 0.165, 0.294, 0.341, 0.417, 0.608 and 0.400. The spread
is six times, so `low`, derived from the sample and not self-rated. Phase 9's
README gave 0.216 over phase 8's nine; phase 9 ran above it, the two plans
that drove a release binary (09-09 at 0.608, 09-08 at 0.417) furthest.
`gsd-tools query estimate-calibration` answers `factor: 1, sample_count: 0`
on this project, so the factor is taken by hand from the files and said here.

## What is owed to documents, and who does it

1. **The roadmap's phase 10 entry and progress row.** Done by the planner in
   the commit that lands these plans: the goal, the five requirements, six
   criteria, the plan list, the row at `0/7`, and the milestone paragraph
   kept true. `test_the_roadmap_counts_the_files_that_are_on_disk` holds the
   row to the files.
2. **`.planning/REQUIREMENTS.md`.** Done by the planner: the `MAIL` section,
   the five traceability rows, the coverage count re-taken at 61, the
   provenance note.
3. **`.planning/STATE.md`.** Done by the planner in the same commit, by hand:
   phase 10 current, plan 1 of 7, `Total Plans in Phase: 7`,
   `progress.total_plans` counted from the disk.
4. **`docs/changelog.md`.** Every plan but 10-01 writes its entries under
   `[Unreleased]`; 10-05 dates the two older entries that named the retired
   commands; 10-07 reads the six as one.
5. **The requirement ticks.** No plan ticks a `MAIL` requirement because its
   own tasks finished; the last plan's summary reads each clause by clause
   and the phase closes them, on 09-10's pattern.
6. **The ledger.** Each plan adds entries by hand for what it could not
   settle, both halves; 10-06 closes 446 and corrects 447; 10-05 closes 12
   with the offer panel (13 has been `fixed` since 2026-09-01); 10-07
   corrects 11, 64, 65 and 72 by addition and re-points 72 at the module
   that holds the loop now.
7. **The four pages.** 10-05 writes `docs/KEYBOARD_SHORTCUTS.md`; 10-07 writes
   `docs/ALPHA_TESTING.md`, `docs/privacy.md`, `docs/USER_GUIDE.md` and
   `docs/manual-accessibility-pass.md`.

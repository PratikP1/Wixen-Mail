# Phase 7 plans: installing, updating and what is stored

Nine plans, revised twice. Seven were written on 2026-09-06 against branch
`spelling-caret-walk`; all seven were revised, and two were added, after Pratik
answered decisions 6 and 7; and four were revised again after he answered
decisions 14, 15 and 16 the same day. The second revision was measured against
branch `undo-send-really-holds` at `c256197`, with phase 4.2 in flight, because
`main` at `9611b70` had already moved. Nothing in the repository was changed to
write or revise them.

**What the second round changed, in one line each.** Success criterion 2 is
widened to cover the download, per D-14. The release channel becomes a `[D]` line
under SHIP-02 rather than a requirement of its own, per D-15. And the two update
settings become one control with three values, the fetch becomes unattended, the
question moves to just before running, and a Help menu item does the whole thing
by hand whatever the setting says, per D-16.

## The plans, in order

| Plan | Wave | Depends on | Autonomous | Requirements | What it does |
|---|---|---|---|---|---|
| `07-01` | 1 | none | yes | SHIP-01, SHIP-04 | The first-run screen and `--help` say the mail on this disk is not encrypted. A guard stops any shipped page promising again that signing removes the Windows warning. Every path the program writes gets onto the two pages that list them. |
| `07-02` | 2 | none | yes | SHIP-03 | An installer-script change earns the full gate, which it does not today. Then both shortcuts name an icon file the installer installs. |
| `07-03` | 3 | none | yes | SHIP-06 | Two accessors nothing has ever called get a caller, widened to cover both halves of the accessibility bridge, said in the About dialog and at startup on a build where the bridge is absent. |
| `07-04` | 4 | none | yes | SHIP-02 | Two version strings can be put in order. Then whether a release is an offer, which depends on the channel somebody chose, and the three-valued thing they actually choose. |
| `07-05` | 5 | `07-04` | yes | SHIP-02 | The one setting, asking GitHub, five answers, a Help menu item that works whatever the setting says, and the privacy page rewritten in the same change. |
| `07-06` | 6 | none | **no** | SHIP-05 | A dispatch-only workflow that builds this crate on Linux and macOS, then acts on whatever it says. |
| `07-07` | 7 | `07-02` | yes | SHIP-01, SHIP-02 | A release stops rather than publishing an asset it could not produce. Then the certificate decision written down with the three options that lost, criterion 2 widened per D-14, and SHIP-02's fourth `[D]` line per D-15. |
| `07-08` | 8 | `07-02`, `07-07` | **no** | SHIP-01 | Sign all seven artefacts with the certificate D-06 chose, prove one signature by hand, then correct the three pages that say the build is unsigned. |
| `07-09` | 9 | `07-01`, `07-05`, `07-08` | **no** | SHIP-02 | Download the installer without being asked, refuse anything this project did not sign, ask once before running it, and say where the bytes came from and what is left on the disk. |

**Waves are sequential and `depends_on` is not.** Five real dependencies exist:
`07-05` needs `07-04`'s ordering, channel decision and three-valued setting type;
`07-07` writes into the `tests/installer.rs` that `07-02` creates; `07-08` writes
into the same file and into the `release.yml` that `07-07` changes; `07-09` needs
`07-05`'s check and `07-08`'s certificate; and `07-09` also needs `07-01`, which
is new in this revision. Everything else could run in parallel except that every
plan touches `docs/changelog.md` and most touch `Cargo.toml` for a version bump,
and the rule is that same-wave plans share no files. Phase 4 hit this and answered
it the same way.

**The new one is worth explaining, because it is a dependency that buys a RED
half.** `07-01` task 3 builds a check that every path `src/common/paths.rs` hands
out is named in both `docs/installing.md` and `docs/privacy.md`, and it enumerates
by calling the accessors rather than by grepping the source, so it cannot miss one.
`07-09` now keeps a downloaded installer somewhere, which means an accessor, which
means that check goes red the moment the accessor is added. The storage half of
`07-09` was the half with no natural test and it now has one for free. Without
`07-01` it would be an instruction in a plan that nothing checks, which is what
guardrail 4 is about.

## What the decisions changed

### Decision 6, the certificate

**Azure Artifact Signing**, about $10 a month, Pratik's own name as publisher, no
hardware, and the United States or Canada residence requirement met, asked and
confirmed on 2026-09-06.

`07-07`'s blocking checkpoint is **answered, not re-asked**. It is gone and the
plan is autonomous. The three options that lost are not gone with it: `07-07`'s
new task 2 writes them into SHIP-01's evidence, one line each, with what each lost
on, and says which of those reasons could change and which could not. A decision
that shows only the chosen path cannot be told apart from an assumption nobody
examined, and the recurring cost of this one will get questioned.

`07-08` is new and does the signing. It is deliberately last but one, so the
account being slow to verify holds up nothing else.

### Decision 7, the updater

**A real self-updater**, against `07-05`'s recommendation. Three parts, all
placed:

- **Fetch and run the installer** is `07-09`, new. `depends_on: ["07-05",
  "07-08"]`, so signing lands before the updater ships, as the decision asked, and
  it is in `depends_on` rather than prose because it is **structural rather than
  advisory**: without `07-08` there is no publisher name to check a download
  against, and a signature check with nothing to compare against accepts an
  installer signed by anybody at all.
- **Rewriting `docs/privacy.md`** is `07-05`'s task 3, extended by `07-09`'s task
  2. It stays in `07-05` because the page becomes false the moment the check
  ships, and `CLAUDE.md` requires the correction in the same change rather than
  after it.
- **The choice of public or development releases** is split: `07-04` owns what the
  two words mean, because it is entirely a question about versions, and `07-05`
  owns the setting. It is a top-level `AppConfig` field, so
  `every_setting_is_acted_on` fails on arrival, which is the red half for free.

### Decisions 14, 15 and 16, the second round

**D-14, criterion 2 is widened.** It was written when updating meant opening a web
page, so it would have passed without any of the verification the phase now
delivers. The replacement wording is below and `07-07` applies it. Two properties
of that arrangement are deliberate: `07-07` writes the criterion and `07-09` is
measured against it, which is why they are two plans and not one, and wave 7 comes
before wave 9, so the standard exists before the thing it judges. A plan that
writes its own success criterion can only ever pass.

**D-15, the release channel is a `[D]` line under SHIP-02.** `07-07` task 2 adds
it, in the same commit as the criterion 2 replacement, the criterion 1 correction
and SHIP-01's evidence block. One commit because it is one decision record written
into two planning files, and splitting it leaves the roadmap and the requirements
disagreeing in between. That closes what was open question 2.

**D-16, one setting and an unattended fetch.** The substantive one. Four things
moved.

- **One control with three values**, not a switch plus a dependent channel choice.
  `07-04` gains the three-valued type and the one function mapping it to an
  optional channel; `ReleaseChannel` stays at two values so the offer decision
  stays total and no arm of it has to answer a question with no answer. `07-05`
  stores one top-level `AppConfig` field and draws one control. It is still one
  top-level field, so `every_setting_is_acted_on` and its mirror still both fail on
  arrival, which is still the red half for free.
- **The disabled-control question is deleted, not answered.** The earlier version
  of `07-05` asked its executor to argue what a dependent channel control should do
  while its parent was off. D-16 removes the dependency and gives the reason: a
  greyed-out control is skipped in the tab order, so somebody moving by keyboard
  does not meet it as unavailable, they do not meet it at all. The question is gone
  from the plan and from the open list, and `07-05` now says to stop rather than
  argue if a second control starts to appear.
- **The fetch is unattended and the install is not.** With a channel chosen, `07-09`
  checks, downloads and verifies on its own, then asks once, about running.
  Consent for the unattended part is given at `07-05`'s control, whose description
  now has to say that choosing a channel means installers get downloaded without
  being asked for. A warning that only lives in a document is a warning nobody
  gets.
- **A Help menu item does the whole thing by hand, whatever the setting says.**
  That is Pratik's addition and it is the deliberate path criterion 2 asks for.
  `07-05` builds the item; `07-09` extends what happens after its answer and adds
  no second item. It created one question the plans had to answer rather than
  leave open, and `07-05` answers it: **with the setting on not looking, a manual
  check asks the public channel and the answer says which channel it asked.** Off
  carries no channel, so something had to. Public is the conservative answer, it
  costs no second question at the moment somebody has already asked one, and the
  wording rule already in the plan names the setting that would show prereleases.
  Remembering a channel while off is two settings wearing one label, which is what
  D-16 removed; asking which channel every time is a question in front of the
  answer.

### The three things this shape owes, and where each landed

**A file arriving unasked is a different promise from telemetry, and it gets its
own sentence.** `docs/privacy.md` opens by saying there is "no analytics, no
telemetry, no crash reporting service and no update check that says who you are",
and every other thing it describes is something leaving the computer. D-16 adds
something arriving. `07-05` task 3 still writes the paragraph about what the check
sends and leaves room; `07-09` task 2 adds the download's hosts to that paragraph
**and** a separate sentence saying a file arrives on the disk without being asked
for, how large it is, where it goes, when it is removed, and that leaving the
setting on not looking means it never happens. Both plans are told to read what
the page promises today before wording anything, because it has already had to be
re-read once for the check.

**Where the installer is kept is a named path, and the rule for clearing it is
three rules.** `07-09` task 1 adds an accessor to `src/common/paths.rs`, under the
application's own root where only this user can write. Under the root for two
reasons: another local user cannot replace the file between the verification and
the run, and `docs/privacy.md`'s "Uninstalling" section already claims uninstalling
removes everything, which stays true for a file under the root and would need
weakening for one in a temporary folder. The plan says to confirm that by reading
what the uninstall really removes rather than assuming.

Clearing is three rules and the third is the one that gets forgotten: after a
successful handover, after a refusal, and **at the next program start**, because
the process can die between downloading and handing over and neither of the first
two runs then.

**And this is where `07-01` earns a place in `07-09`'s `depends_on`.** `07-01`
task 3 builds a check that every path `paths.rs` hands out is named on both
`docs/installing.md` and `docs/privacy.md`, enumerating by calling the accessors
rather than by grepping. So adding the download accessor reddens it on arrival and
the only way to green it is to write the path onto both pages. The storage half of
`07-09` was the half with no natural test and it now has a RED half for free.

**Verify before asking is written as a rule with its reason, not as an ordering.**
The rule, in `07-09` premise correction 4a: *nobody is ever asked to approve a file
that is already known to be bad.* The reason belongs beside it, because without it
somebody will later add a helpful "run it anyway?" button: a question implies the
answer could reasonably be yes, and offering it moves a decision the program is
equipped to make onto a person who is not, at the moment they are most likely to
say yes.

**It is carried in the types rather than in the sequence.** The function that asks
takes the verified outcome as its only argument, exactly as the run path does. Then
there is no ordering to get wrong, no later edit that can move the question
earlier, and no review that has to notice. `07-09` task 2's test about there being
no path from an unverified file now covers the asking as well as the running, and
the checkpoint's step 7 asks explicitly whether any question appeared on the
refusal path, because a question there is a broken rule rather than bad wording.

### Which plan is the big one moved, and it is worth knowing why

**`07-05` was too big and was split, but not where it looks.** It went from two
tasks to three rather than into two plans, and the reason is that both halves that
could move are pinned. The setting cannot go earlier: the guard in `config.rs` asks
whether anything **outside** `config.rs` and `wx_settings.rs` reads a field, and
the reader is the check itself, so a plan holding the field without the check ends
red. The privacy page cannot go later, because it becomes false the moment the
check ships. So the plan is three tasks, with the cut named as a cut in time and
not in scope.

**D-16 then took it back down.** Two settings and two controls became one of each,
and the disabled-control judgement it was carrying disappeared with them. It gained
the widget choice, the loader question and the manual-channel decision, and the
arithmetic lands at 105,000 raw tokens, slightly below where it was.

**`07-09` is now the largest plan in the phase at 116,000**, and it is worth
saying that the two rounds pushed in opposite directions on which plan to watch.
Round one made `07-05` the one to worry about, because the settings landed in it.
Round two moved that worry to `07-09`, because an unattended fetch needs a place to
put the file, a rule for clearing it, a line on two pages and a question in front
of the run. `07-09` says so inside itself with the cut named, and the cut no longer
includes its page edits, because `07-01`'s paths check is red until they are
written.

`07-04` shrank in round one, losing the settings and gaining the channel judgement,
then grew slightly in round two with the three-valued type and its mapping. It is
still the cheapest of the four, because `src/common/version.rs` appears in no
`tests_last_seen` block at all.

## What the phase closes on its own, and what waits

**Everything except signing and updating closes with no account and no release.**
Plans `07-01` through `07-07` need nothing outside this repository except the
Linux and macOS workflow dispatch in `07-06`, which is a checkpoint and not a
purchase.

**Success criterion 1 is no longer blocked on a decision.** It is blocked on one
thing: the Azure Artifact Signing account existing. That is a different state
from where these plans started, when the certificate was an open question with
four answers and criterion 1 could not close in the phase at all.

What closes once the account exists, in `07-08`: seven artefacts signed, not the
two the requirement's wording implies, each verified by hand with the subject
name, the chain and the timestamp countersignature quoted before any document is
corrected.

**One clause of criterion 1 still does not close in this phase, and it is not
about the account.** It asks for a signature "verified against the published
release asset rather than a local build". Proving that needs a release, and
dispatching one from inside a plan is publishing as a side effect, which
guardrail 7 forbids. `07-08` verifies a locally signed build instead and records
the published-asset half as `unrun-verify`, closing on the first release Pratik
dispatches deliberately. Say that plainly rather than letting the criterion read
as fully closed.

## The roadmap's criterion 1 is wrong and here is the replacement

`.planning/ROADMAP.md:432` currently reads:

> The published installer and the executable inside it both carry a valid
> Authenticode signature with a timestamp countersignature, verified against the
> published release asset rather than a local build. What SmartScreen then does is
> stated, not promised: only an EV certificate carries reputation from the first
> download, and while the warning remains, `docs/installing.md` keeps the
> walkthrough that gets a screen reader user past it.

The clause "only an EV certificate carries reputation from the first download"
says the opposite of Microsoft's current page, which was updated 2026-08-17: "EV
certificates no longer bypass SmartScreen. Years ago, signing files with an
Extended Validation (EV) code signing certificate would result in positive
SmartScreen reputation by default, but this behavior no longer exists."

**Replacement, to be applied by `07-08`'s sibling task, `07-07` task 2:**

> The published installer and the executable inside it both carry a valid
> Authenticode signature with a timestamp countersignature, verified against the
> published release asset rather than a local build. What SmartScreen then does is
> stated, not promised: no certificate available to this project removes the
> warning on a first download, since EV certificates no longer bypass SmartScreen
> and only publishing through the Microsoft Store avoids it. What signing buys is
> the publisher's name in place of "Unknown publisher", reputation that accrues
> across releases under one identity, and Smart App Control on Windows 11 no
> longer blocking the file. While the warning remains, `docs/installing.md` keeps
> the walkthrough that gets a screen reader user past it.

Only the middle clause changes; the first and last sentences are the original.
The plan reads this wording from here rather than composing its own, so the words
that land are the words that were reviewed.

**Re-fetch Microsoft's page before applying it.** This external fact has already
gone false once inside a project document with no commit to this repository, which
is how the roadmap came to say the opposite. Quoting the research's quotation of a
page nobody re-read would repeat the mechanism that caused the defect.

## Criterion 2 is widened and here is the replacement

`.planning/ROADMAP.md:433` currently reads:

> The application can tell the user a newer version exists, as a deliberate
> action or an explicit setting, never as a silent background fetch, and declining
> leaves the current version working.

D-14 widens it. This is a decision about what the phase must prove rather than a
correction of a wrong fact, which is why it waited for Pratik and why criterion
1's did not.

**Unlike criterion 1, the whole sentence is replaced and not one clause.** Two of
its three clauses have to move. The opening is too narrow, because the phase now
fetches an executable and runs it. And "never as a silent background fetch" now
contradicts D-16, which asks for exactly an unattended fetch once somebody has
chosen a channel. Leaving that clause beside a widened one would put two criteria
in the roadmap that cannot both be met.

**Replacement, to be applied by `07-07` task 2:**

> The application can tell the user a newer version exists and can apply it, and
> nothing is fetched that the user did not ask for. Asking is one of two things:
> choosing the update item in the Help menu, which works whatever the setting says,
> or choosing a release channel in the one update setting, which starts off and
> says where it is chosen that choosing a channel means installers are downloaded
> automatically. With a channel chosen, the check, the download and the
> verification happen unattended; running the installer never does, and the user is
> asked first. A downloaded installer is verified before anybody is asked about it:
> the Authenticode signature must be valid and the signer must be this project's
> own publisher name, and a file that fails either check is refused and deleted
> rather than warned about and offered anyway. Where a signature cannot be checked
> at all, nothing is downloaded and nothing is run, and the reason is said.
> Declining, and a handover that fails, both leave the current version working.

`07-07` reads this wording from here rather than composing its own, and it is told
not to read `07-09` while applying it. A criterion adjusted to fit its
implementation can only ever pass.

## The release channel gets a line in SHIP-02, and one existing line has to move

D-15: a `[D]` line under SHIP-02, not a `SHIP-07`. Choosing a channel is part of
what updating means for this product rather than a separate capability. **`07-07`
task 2 adds it, in the same commit** as the criterion 1 correction, the criterion 2
replacement and SHIP-01's evidence block.

**The existing first `[D]` line has the same problem criterion 2 had**, and it was
found by reading the requirement rather than by being told. `.planning/REQUIREMENTS.md`
currently says:

>   - [D] The application can tell the user a newer version exists, and the check is a
>     deliberate action or an explicit setting, never a silent background fetch, because
>     publishing and fetching both happen on purpose here.

"Never a silent background fetch" reads false under D-16. The intent survives, and
the words do not. **Replacement:**

>   - [D] The application can tell the user a newer version exists, and nothing is fetched
>     that the user did not ask for: either by choosing the update item in the Help menu,
>     which works whatever the setting says, or by choosing a release channel in the update
>     setting, which starts off. With a channel chosen the fetch is unattended and the
>     consent for it was given at the control, which says so. It is never a fetch nobody
>     chose, because publishing and fetching both happen on purpose here.

**And the new fourth line:**

>   - [D] Which releases the user hears about is theirs to choose: one setting with three
>     values, not looking, public releases and development releases, reachable from the
>     settings screen in a section somebody would look in. A prerelease is never offered on
>     the public channel.

The other two `[D]` lines under SHIP-02, about applying being the user's decision
and about the comparison ignoring `+build`, are unchanged and still true.

## The three that need Pratik, and why

**`07-06`, dispatching the Linux and macOS build.** Nothing in this repository can
build for another platform, and nothing here may dispatch a GitHub Actions run.
The checkpoint asks for four measurements per platform. Unchanged.

**`07-08`, the Azure account.** Creating a subscription, passing an identity
verification that names a real person, paying, and granting a role in a tenant.
None of it is a repository operation. The checkpoint is second of three, after the
task that settles what has to be signed and before the task that signs it. Two
things in it are easy to miss and are called out: identity verification is the step
that waits, and granting the signing identity the certificate-profile role is a
separate step from creating it, whose failure reads like a bad endpoint.

**`07-09`, hearing an update happen.** The last task is a human-verify checkpoint
with nine things to try under NVDA, up from eight. Two of the additions are D-16's:
what an unattended download feels like when it starts while somebody is reading
their mail, and confirming that **no question at all** appears on the refusal path,
because a question there is a broken rule rather than bad wording. The ninth is
killing the program mid-download and confirming the directory is empty on the next
start. It needs a published release to update from, so it may stay open; the plan
says to record that rather than pretend.

**`07-07` is no longer one of them.** Its checkpoint was answered by D-06, so it
runs start to finish.

## Coverage audit

`.planning/decisions-2026-09-06.md` exists in the repository and is byte-identical
to the scratchpad copy for decisions 6, 7, 14, 15 and 16. There is no `CONTEXT.md`
for this phase, so `D-06`, `D-07`, `D-14`, `D-15` and `D-16` are the decision
identifiers and they are cited in the frontmatter of every plan they bind.

**Goal.** Four clauses. "Reaches a user signed" is `07-07` and `07-08`. "Tells
them when there is a newer one" is `07-04` and `07-05`. "Tells them plainly what
it leaves on their disk" is `07-01`. "Does not promise a publisher warning will be
gone" is `07-01`, and `07-08` is the plan most likely to break it, which is why its
document task runs `cargo test --test house_style` before committing.

**Requirements.** SHIP-01 in `07-01`, `07-07` and `07-08`. SHIP-02 in `07-04`,
`07-05`, `07-07` and `07-09`. SHIP-03 in `07-02`. SHIP-04 in `07-01`. SHIP-05 in
`07-06`. SHIP-06 in `07-03`. Every one is in at least one plan.

**Decisions.** D-06 is implemented by `07-07` task 2, which records it, and
`07-08`, which acts on it. D-07's three bullets are `07-09` task 1 and 2, `07-05`
task 3 extended by `07-09` task 2, and `07-04` task 2 with `07-05` task 1. D-14 is
`07-07` task 2. D-15 is `07-07` task 2. D-16's four parts are `07-04` task 2 and
`07-05` task 1 for the one control, `07-09` task 1 for the unattended fetch,
`07-09` task 2 for the one question, and `07-05` task 1 with `07-09` premise
correction 4c for the Help menu path. The three things D-16 owes are `07-05` task
3 with `07-09` task 2 for the privacy page, `07-09` task 1 for where the installer
is kept and when it goes, and `07-09` premise correction 4a for the verify-before-
asking rule. Every bullet has a task.

**Research.** Every finding in `07-RESEARCH.md` is either planned or excluded
below with a reason. Four of its claims turned out to be stale or wrong and the
corrections are in the plans that depend on them.

**Deliberately not planned**, each with the reason:

- **`finish_erasing` returns 1 when something was left and the uninstaller does
  not read the exit code.** `src/main.rs` records this as a known gap. No
  criterion and no requirement asks for it. It is a real gap and it belongs in a
  ledger rather than in this phase. **Raised as question 3 below.**
- **A provider removed from `OAuthService::providers()` stops being erased while
  its tokens stay on the machine.** `07-RESEARCH.md` names it and says plainly
  it is not a gap phase 7 has to close. Agreed: nothing in this phase adds or
  removes a provider.
- **`tests/house_style.rs` does not read `.planning/`, while `CLAUDE.md` and
  `scripts/which-checks.sh` both say the em-dash guard caught a break in a
  planning file.** `07-RESEARCH.md` flags it as belonging to guardrail 4 rather
  than to this phase. Agreed, and it now bites twice: `07-07` task 2 writes two
  `.planning/` files that no test in this tree can read, and the plan says so
  rather than running a test that proves nothing.
- **`docs/installing.md` and `docs/privacy.md` carry the same storage paragraph
  almost word for word with nothing checking they agree.** `07-01` adds a check
  for the paths those pages list, which is the half that can go wrong silently.
- **A third list on `src/service/outward.rs`'s census, for a module whose bytes
  get executed.** Raised by `07-05` and again, with stronger evidence, by
  `07-09`, and acted on by neither, because changing that file's shape costs
  re-measuring the 10 records that fingerprint it for a question neither plan is
  about. `07-09`'s success criteria say to put it in a ledger rather than leave it
  raised twice and recorded nowhere.

## The things that will cost more than they look

**Guard-record re-measurement is on the critical path.**
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` lives in
`tests/house_style.rs`, which is one of the two whole-tree guards, so it runs
inside the commit gate on every commit. Its remedy, `scripts/guards.sh
--remeasure`, is one build and one run per record. It cannot be deferred to the
end of the phase; only the full sweep can.

**Re-counted twice.** First on 2026-09-06 against `main` at `9611b70`, version
0.75.0, and again the same day for this revision against branch
`undo-send-really-holds` at `c256197`, with phase 4.2 in flight and
`guards/guards.toml` also modified in the working tree. Both counts use the `awk`
over `tests_last_seen` blocks that `CLAUDE.md` gives, not a grep for a file name.

**The tree holds 632 records and that has not moved**, at `9611b70`, at `c256197`
and in the working tree. Counted with `grep -c '^\[\[guard\]\]'`; a naive
`grep -c 'tests_last_seen'` answers 634, because a comment header at line 44 and a
record's prose at line 15573 both carry the phrase.

**Every per-file record count is unchanged too.** Two stored test counts moved and
they are the two the executor's in-flight work touches:

| file | at `9611b70` | now | where |
|---|---|---|---|
| `tests/wired.rs` | 64 | **66** | committed on `undo-send-really-holds` |
| `src/data/config.rs` | 53 | **54** | working tree only, not committed |
| `src/application/sending_later.rs` | 40 | 47 | committed; no plan here touches it |

`src/data/config.rs` is the one to watch, because it is not committed and can move
again, and because plan `07-05` adds tests to it. Its 2 records will want
re-measuring against whatever it holds when `07-05` runs, not against 54.

**The `#[test] today` column is now read out of the `tests_last_seen` blocks
rather than counted with a grep, and that changed three numbers.** The per-commit
count check refuses any record disagreeing with the tree, so on a green tree those
stored values are the checker's own current answer, and reading them is both
cheaper and safer than recomputing. The checker counts a line whose whole trimmed
content is `#[test]` or `#[tokio::test]`; `grep -c '#\[test\]'` also counts the
attribute written inside a doc comment, which this tree does nineteen times. It
says 73 for `tests/house_style.rs` where the checker says 64. The earlier version
of this table quoted the grep.

| File a plan adds tests to | Records fingerprinting it | Then | `#[test]` today | Then | Roughly |
|---|---|---|---|---|---|
| `tests/house_style.rs` | 18 | 18 | **64** | 73 | 25 to 30 minutes |
| `src/presentation/wx_app.rs` | **42** | 40 | **199** | 196 | about an hour |
| `tests/wired.rs` | **10** | 10 | **66** | 64 | 14 to 16 minutes |
| `src/service/outward.rs` | 10 | 10 | 38 | 38 | 14 to 16 minutes |
| `src/presentation/accessibility.rs` | 7 | 7 | 23 | 23 | 10 minutes |
| `src/presentation/accessibility/names.rs` | 4 | 4 | 32 | 32 | 6 minutes |
| `src/presentation/accessibility/screen_reader.rs` | 3 | 3 | 22 | 22 | 5 minutes |
| `src/presentation/first_run.rs` | 3 | 3 | 14 | 14 | 5 minutes |
| `src/data/config.rs` | 2 | 2 | **54** | 53 | 3 minutes |
| `src/presentation/wx_first_run.rs` | 2 | 2 | 0 | 0 | 3 minutes |
| `src/presentation/wx_settings.rs` | 1 | 1 | 0 | 0 | 90 seconds |
| `src/presentation/command_line.rs` | 1 | 1 | 23 | 23 | 90 seconds |
| `src/main.rs` | 1 | 1 | 0 | 0 | 90 seconds |
| `src/common/version.rs`, `src/common/paths.rs`, `tests/theme_reach.rs` | 0 | 0 | 4, 16, 7 | 4, 16, 10 | free |
| `src/service/update_check.rs`, `src/service/update_download.rs`, `tests/installer.rs` | absent | absent | n/a | n/a | free until created |

**Taken while phase 4.2 is still being executed, so treat every figure as dated
rather than current.** The second count was against `c256197` on branch
`undo-send-really-holds`, with `guards/guards.toml`, `Cargo.toml`,
`docs/changelog.md`, `src/presentation/wx_app.rs`, `src/presentation/wx_settings.rs`
and `tests/wired.rs` all modified and uncommitted. The first count, a few hours
earlier, said `guards/guards.toml` was unchanged and that nothing had moved; by the
second it had, twice. That is the half-life this kind of fact has, and it is why
**the executor re-measures the files it is about to touch rather than quoting this
table**, which is this project's own rule about a record being a measurement with a
date, applied to the table as much as to the records.

**Record counts have been stable across three measurements and test counts have
not.** `wx_app.rs` went 40 records to 42 and `wired.rs` 8 to 10 during phase 4;
neither has moved since, and no other file's record count has moved at all. The
stored test counts moved twice in one day. So the expensive number, how many
records a file costs, is the stable one, and the cheap number is the one that
drifts. That stability is worth knowing and is **not** a reason to skip the
re-measure.

The durations come from this project's own figures, 565 records in about 15 hours
and 63 records in about 90 minutes, so about 1.4 to 1.6 minutes a record. They are
estimates from a rate, not measurements, and `CLAUDE.md` asks that a duration be
read with its conditions.

**The count is per file, not per test.** Four new tests in one commit cost the
same as one. Every plan that touches an expensive file says to land all of its
additions there in a single commit for exactly that reason.

**Files to add no `#[test]` to, because they are expensive and nothing needs it:**
`src/presentation/wx_app.rs` (42 records), `tests/wired.rs` (10),
`src/service/outward.rs` (10), `src/presentation/accessibility.rs` (7).

**A commit touching `Cargo.toml` makes `scripts/which-checks.sh` answer `all`.**
Every version bump does. Those commits are run detached. Plans `07-01` through
`07-05`, `07-08` and `07-09` bump the version; `07-06` and `07-07` do not touch
`Cargo.toml` and answer `affected`.

**Two re-measurements no automated check will ask for.** `07-05` adds a member to
`TALKS_BUT_ONLY_READS` in `src/service/outward.rs`, and `07-09` adds a second.
`CLAUDE.md` says to re-measure every record that reads a census when a member is
added, and adding a string to a `const` array changes no test count, so the count
check stays silent both times. `src/service/outward.rs` is fingerprinted by 10
records. If those do not appear in the two summaries, they did not happen.

**The Rust floor is 1.88, not 1.87.** `Cargo.toml:15`, moved by `pgp = "0.20"`,
which declares 1.88. No plan named a Rust version, so nothing needed correcting;
what the floor changes is a cost to expect. Plan `04-09` found that the bump turned
on a clippy lint family across **seven pre-existing sites in six files it never
opened**, and clippy runs with `-D warnings`, so that is a build failure rather
than a warning. `07-09` is the plan that could hit it, since it enables features on
existing dependencies, and it says what to do: fix them in a separate commit that
says why, never reach for `#[allow]`.

**A red commit is allowed only on a branch, never on `main`**, with
`Fails-until-green:` trailers naming every failure and nothing else failing.
Several plans name the count check among their trailers because there is no
ordering in which the record fingerprints can be right before the tests exist.

## Estimates

| Plan | `tokens` | `raw_tokens` | Tasks | Change in this round |
|---|---|---|---|---|
| `07-01` | 94,000 | 78,000 | 3 | unchanged |
| `07-02` | 66,000 | 55,000 | 2 | unchanged |
| `07-03` | 74,000 | 62,000 | 2 | unchanged |
| `07-04` | 67,000 | 56,000 | 2 | up: the three-valued setting type and its mapping |
| `07-05` | 126,000 | 105,000 | 3 | down slightly: two settings and two controls became one of each |
| `07-06` | 54,000 | 45,000 | 3 | unchanged |
| `07-07` | 50,000 | 42,000 | 2 | up: criterion 2 replaced, one `[D]` line corrected, one added |
| `07-08` | 94,000 | 78,000 | 3 | unchanged |
| `07-09` | 139,000 | 116,000 | 3 | up: the question, the download directory and its three clearing rules, its line on two pages, and the privacy sentence |
| **Total** | **764,000** | **637,000** | **23** | was 744,000 / 620,000 / 23 |

**The second round costs 17,000 raw tokens and adds no task.** That is worth
saying plainly: D-16 removed a setting and added a question, a directory and a
sentence, and the arithmetic lands slightly positive. The first round added
210,000; this one adds under a tenth of that.

**`07-09` is now the largest plan in the phase**, ahead of `07-05`, and it says so
inside itself with the cut named. `07-05` came down and is no longer the one to
watch.

**Confidence is `low` on every plan, and that is derived rather than felt.**
Thirteen completed plans have both an estimate and an actual. Phase 4's five give
actual-over-raw ratios of 1.17, 1.29, 1.29, 0.48 and 1.18, median 1.18. Phase 3's
eight, taken under a different convention where no factor was applied, give 0.12
to 0.64. Two regimes, ratios spanning an order of magnitude. The factor used here
is 1.2, from phase 4's median. A spread that wide is what `low` means.

**`07-05` and `07-09` are both over the comfortable size and both say so inside
themselves**, with the cut named. Neither is split further because in each case
the thing that could move is pinned by a guard or by a rule in `CLAUDE.md`, and
both plans record which.

## Open questions that are genuinely Pratik's

**Three of the four are answered and gone.** Criterion 2's widening is D-14 and is
applied by `07-07`; the release channel's requirement is D-15 and is applied by
`07-07`; the disabled-control judgement inside `07-05` was deleted by D-16 rather
than answered, because the shape it was about no longer exists. What follows is
what is left, plus what the third round created.

**1. Should the uninstall exit code be in this phase?** `finish_erasing` returns 1
when something was left behind, and `src/main.rs` records that the uninstaller does
not read it. So a partial erase is invisible to the thing that could report it.
`07-RESEARCH.md` calls it "the remaining honest gap". No criterion asks for it and
no plan covers it. It is one of the better candidates for a small addition if the
phase has room, and it is named rather than silently dropped. Unchanged through
three versions of this document.

**2. A metered connection, which D-16 created and no plan handles.** With a channel
chosen, this program fetches several megabytes without asking. Somebody who chose a
channel on a home connection and later works from a phone tether pays for that. The
plans do three things about it and not a fourth: `07-05`'s control description says
the download is automatic, so the choice is informed; the setting starts on not
looking; and `07-09` carries it as an accepted threat with a ledger entry rather
than a silence. **What none of them do is ask Windows whether the connection is
metered**, which it knows. That is a real feature, it is small, and nobody has asked
for it. Named rather than dropped.

**3. Whether the answer to a manual check while the setting is off should be the
public channel.** `07-05` decides this rather than asking, because leaving it open
would put a question in front of a person at the moment they asked for something.
The decision is that a manual check with the setting on not looking asks the public
channel and says which channel it asked. **It is the one judgement in this revision
that was made on Pratik's behalf**, and it is recorded here so it can be overturned
cheaply if it is wrong. The alternatives are in `07-05` premise correction 8a with
what each costs.

## What to check before executing

`07-RESEARCH.md` was written on 2026-09-04 and four of its findings have gone
stale or were wrong when written. Each correction is in the plan that depends on
it, with the command to re-run rather than the conclusion. All four survive this
revision:

- **Its headline finding is fixed.** `docs/installing.md` no longer promises
  that signing makes the warning go away; commit `32af82b` of 2026-09-04
  replaced it. `07-01` therefore writes the guard rather than the correction.
- **"Neither `house_style` nor `wired` reads the `.iss`" is wrong for
  `house_style`**, which collects `installer/*.iss` in `ours()`. The gate hole
  is real for a different reason one layer down, and the different reason changes
  which fixes are available. `07-02` has both.
- **wxDragon may not build wxWidgets from source.** Upstream says it "downloads
  pre-built wxWidgets libraries during the first compilation, reducing build
  times from 20+ minutes to under 3 minutes", which is the opposite of the cost
  the research puts on SHIP-05. Whether that holds for the pinned `=0.9.17` on
  these runners is exactly what `07-06` measures rather than assumes.
- **`ALPHA_TESTING.md`'s "the installer is not signed" is at line 150**, not 146.
  Confirmed again on 2026-09-06. `.planning/REQUIREMENTS.md`'s SHIP-01 evidence
  still cites 146, and `07-07` task 2 corrects it, preferring a quoted phrase over
  a line number since line drift is why the requirements audit of 2026-09-04 asked
  for symbols.

Two more the research never had, both still live:

- **The update setting existed as `check_updates`, defaulting to `true`, removed
  by commit `cb7caa2` on 2026-08-24.** serde deserialises by field name, so
  reusing the name would read a stale `true` out of any settings file written
  before that date and switch on a startup fetch nobody asked for. Carried in
  `07-05` now that the setting lives there, with a pointer left in `07-04`.
  **D-16 changes the failure mode and does not remove it.** The new field is an
  enum, not a `bool`, so an old file holding `check_updates = true` is a type
  mismatch rather than a stale value, and depending on how this project's loader is
  written that either drops one value or fails the whole file and returns **every**
  setting on that machine to its default. `07-05` now says to read the loader and
  report which. Then still do not reuse the name: a name nobody has used has no
  failure at all.
- **A new network module must be registered in `src/service/outward.rs`'s
  census**, and no automated check will raise the guard re-measurement that
  follows, because adding a string to a `const` array moves no test count. This
  now happens twice, in `07-05` and `07-09`.

Two more from this revision:

- **`cargo test --lib a --lib b` does not run.** "the argument '--lib' cannot
  be used multiple times". Every `<verify>` in these plans uses
  `cargo test --lib -- a b`, which does.
- **The signature check in `07-09` is two checks and the second is the one that
  matters.** `WinVerifyTrust` answers "validly signed by somebody"; the publisher
  comparison answers "signed by us". A verification that stops at the first is
  worse than none, because it prints "verified" before running an executable
  somebody else signed. The plan is arranged around that: refusal tests before the
  success test, a fixture signed by another publisher, and a guard record whose
  break is exactly the publisher comparison always succeeding.

Three more from this round:

- **`.planning/REQUIREMENTS.md`'s SHIP-02 has the same defect criterion 2 had, and
  nobody was told about it.** Its first `[D]` line says the check is "never a silent
  background fetch", which D-16 contradicts directly. It was found by reading the
  requirement while placing D-15's new line rather than by being asked to look.
  `07-07` task 2 now replaces it, with the wording in this document, and is told to
  leave the other two `[D]` lines alone because they are still exactly true.

- **`docs/privacy.md` already promises that uninstalling removes everything**, in
  as many words: "the program, your accounts, your settings, the downloaded mail,
  and your saved passwords and sign-in tokens". A downloaded installer under the
  application root goes with the rest and that sentence stays true; one in a
  temporary folder does not. That is a second reason for the location `07-09`
  chooses, on top of the security one, and the plan says to confirm by reading what
  the uninstall really removes rather than assuming the root is cleared wholesale.

- **`07-01`'s paths guard turns `07-09`'s storage obligation into a RED commit.**
  Adding an accessor to `src/common/paths.rs` fails a check that reads the
  accessors, so the pages have to be written before the commit is green. This was
  not designed; `07-01` was written for `security.key` being missing from two
  pages. It is the clearest example in the phase of a guard paying for itself in a
  plan written after it, and it is why `07-01` is now in `07-09`'s `depends_on`.

And one measurement that removes a dependency question rather than raising one:
**`07-09` needs no new crate.** `WinVerifyTrust`, `CryptQueryObject` and
`CertGetNameStringW` all exist in the `windows` crate already pinned at `0.62.2`,
behind the `Win32_Security_WinTrust` and `Win32_Security_Cryptography` features,
which are simply not enabled. Verified on 2026-09-06 by grepping the vendored
source. The plan carries the greps and requires them re-run, because a feature that
exists at one version is not a feature that exists at another.

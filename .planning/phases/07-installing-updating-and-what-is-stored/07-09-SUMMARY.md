---
phase: 07-installing-updating-and-what-is-stored
plan: 09
subsystem: service
tags: [update, authenticode, code-signing, download, defensive-boundaries, windows-api]

requires:
  - phase: 07-01
    provides: "the check that calls every accessor src/common/paths.rs hands out and demands each be named on docs/installing.md and docs/privacy.md, which is what made adding the download folder a red half for free"
  - phase: 07-05
    provides: "the check itself: one setting with three values, five answers, a Help menu item that works whatever the setting says, and the outward census entry this plan's module sits beside"
  - phase: 07-08
    provides: "the certificate subject name this project will sign with, which is `Pratik Patel`, read off the installer's own AppPublisher. Nothing else: 07-08 is partial, task 1 only, and nothing is signed"
provides:
  - "src/service/update_download.rs: the fetch, the two-part signature check, the one question and the handover, with the checked file carried as a type nothing outside the module can build"
  - "An installer fetched without anybody being asked, once a kind of version has been chosen"
  - "A refusal that distinguishes an unsigned file, a broken signature and a file somebody else signed, deletes the file, and never offers to run it anyway"
  - "src/common/paths.rs: updates_dir, named on both pages that list what is left on the disk, emptied three ways"
affects: []

actuals:
  tokens: 32000
  tasks: 2
  commits: 7

tech-stack:
  added: []
  patterns:
    - "A capability carried in a type rather than in an order of steps: the question and the run both take a value only the check can construct, so there is no sequence to rearrange"
    - "A platform gate asked before the work rather than after it, so a computer that cannot check a signature never fetches one"
    - "Two Win32 features enabled on a crate already pinned and audited, in place of a new crate or a parse of another program's output"

key-files:
  created:
    - src/service/update_download.rs
  modified:
    - src/common/paths.rs
    - src/service/update_check.rs
    - src/service/outward.rs
    - src/service/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - src/main.rs
    - docs/privacy.md
    - docs/installing.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - guards/guards.toml
    - Cargo.toml

key-decisions:
  - "Verification is two checks and the second is the one that matters. WinVerifyTrust says whether a file is validly signed, which millions are; reading the signer's certificate and comparing the name is what separates an update from running whatever arrived. The comparison is exact rather than contains, because a certificate issued to `Pratik Patel Holdings Ltd` is one anybody can buy"
  - "The checked file is a type with a private field that only `verify` constructs, and both the question and the handover take it. An ordering can be rearranged by a later edit that nobody notices; an argument type cannot"
  - "Revocation is not checked. It reaches the network, and a revocation server that cannot be reached is not a bad signature. What that trades away is a key revoked after issue, written where the call is made"
  - "The installer starts first and Wixen Mail closes after. Closing first means the start has to outlive its parent and a failed start leaves nothing running to report it"
  - "The module goes on TALKS_BUT_ONLY_READS with the sentence its five neighbours do not need: it is the only member whose bytes are executed. The third list is raised again and again not added, because changing that file's shape costs re-measuring ten records"

patterns-established:
  - "Where a check over source text is the only thing that can see a defect, say so in the guard record rather than letting a one-test red list read as a thin record"

requirements-completed: []

status: partial
---

# Phase 07 Plan 09: Fetch an update, refuse anything unsigned (tasks 1 and 2 of 3)

**Tasks 1 and 2. Task 3 was not attempted.** Wixen Mail now fetches the installer
for a newer version on its own, checks it twice, asks once whether to run it, and
hands over. **None of that has ever run.** No release has been published from this
repository, nothing this project ships is signed, and no screen reader has heard
any of it. What is proven is every decision; what is unproven is every moment.

## What landed

With a kind of version chosen under "Tell me about new versions", a published
newer version is now downloaded without anybody being asked at that moment, which
is the agreement given at that setting. Pressing Check for Updates does the same
on demand whatever the setting says. One route, not two.

The downloaded file is checked before anybody hears about it. Two checks, both
required: `WinVerifyTrust` for the signature, and the signer's own certificate
read through `CryptQueryObject`, `CryptMsgGetParam`, `CertFindCertificateInStore`
and `CertGetNameStringW` for the name on it. A file failing either is deleted and
the person is told which check it failed, in five distinguishable sentences. They
are never asked whether to run it anyway.

If it passes, one question, about running and nothing else. No leaves the running
version alone and deletes the file. Yes says the window is about to close, then
starts the installer and closes.

## Exactly which half of the refusal is proven, and which is not

This is the question the plan asks to be answered precisely, so it is answered
before anything else.

**Proven, on this machine, today.** That a validly signed executable published by
somebody else is refused, by name, for the publisher mismatch and not for some
other reason. That is not a fixture argument: `who_signed` really called
`WinVerifyTrust` and really read a certificate, against
`C:\Windows\System32\microsoft.windows.softwarelogo.showdesktop.exe`, and the
refusal named Microsoft. So the mechanism works end to end against a real
Authenticode signature, and the comparison really discriminates. Also proven:
that a file with no signature is refused and deleted; that a name merely
containing this project's is refused; that the five refusals say five different
things and each names the releases page.

**Not proven, and not provable by anybody today.** That a genuine Wixen Mail
installer is *accepted*. No release has ever been published, there is no
certificate, and nothing this project builds carries a signature. Every test of
the accepting path runs against a name in a fixture rather than a name read off a
file this project signed. The whole suite could pass against an implementation
that reads the certificate subject from the wrong field, so long as it read it
consistently, and nothing here would notice.

So the honest statement is: **the refusal is proven and the acceptance is not.**
As this ships, every real installer is refused, which is the designed behaviour
and is said in `docs/changelog.md`, `docs/installing.md`, `docs/privacy.md` and
`docs/ALPHA_TESTING.md`. When 07-08 task 2 produces a certificate, the first
signed build is the first evidence that the accepting half works, and until then
the update feature is a refusal machine on purpose.

That is guardrail 2's shape exactly, one layer down from a screen reader: a check
that has never seen the thing it is meant to let through.

## Which assertions prove structure, and which only a person can settle

**Structure, and nothing more.** Every sentence a person hears is asserted as
text: the question's words, the five refusals, the six ways a download does not
happen, the two ways a handover fails, the warning before the window closes. A
test can say the words exist and are distinct. It cannot say they are heard,
that they arrive in a useful order, or that they are not coalesced away by a sync
talking underneath them.

**Only a person can settle these.** Whether an unattended download starting while
somebody is reading a message is a helpful update or an interruption. Whether the
question arriving about a file they did not know had been fetched is welcome or
alarming. Whether the second between the warning and the window closing is enough
for somebody listening. Whether the installer's own window is reachable by
keyboard and focus lands somewhere sensible. Whether a slow connection produces a
useful silence or an anxious one. None of these is a gap in the tests; all of
them are the limit of what a test reaches.

## The checkpoint, written so it can be run later at a keyboard

Task 3 is recorded, not done. Pratik does the manual and screen reader testing
after phase 8. **It needs a published release to update from, which means a
release Pratik dispatches deliberately; nothing in this plan may dispatch one.**
Until a signed release exists, steps 5, 6 and 8 cannot be attempted at all,
because every download is refused.

What to do, from a build one version behind, with NVDA running:

1. Find "Tell me about new versions" in Settings, General, under "New versions".
   Confirm the control is announced with its label, its three values and which is
   chosen, and that the description is read out. It now says installers are
   downloaded without asking, about 12 MB, and that you are asked once before one
   is run. Say whether a radio group or a combo box was built and whether it
   reads well, and whether "New versions" is a section you would have looked in.
2. Leave it on "Do not look for new versions" and press Help, then Check for
   Updates. Confirm it still checks, that the answer is heard and not only shown,
   and that it says which channel it asked.
3. Choose a channel and restart. Confirm the check happens once, that the window
   appeared and spoke without waiting for it, and that the download starts on its
   own with no question.
4. Confirm the download does not flood, and that one starting while you are
   reading a message does not take you out of what you were doing. **Nothing in
   this tree can approach this step.** What was built: an unattended download
   says nothing at all until it has something to say, and a download somebody
   asked for says one sentence at the start. Judge whether that silence is right.
5. Confirm you are asked once, only about running, and only after the check.
   Then say no, and confirm the version you were running still works and the file
   is gone from `%LOCALAPPDATA%\wixen-mail\updates`.
6. Do it again and say yes. Confirm you are told the program is about to close
   before it closes; that the installer appears; that its window is reachable by
   keyboard and focus is somewhere sensible; and that the new version starts with
   your mail, accounts and settings intact. **Watch for the mutex message.** The
   installer refuses to run while a copy of Wixen Mail holds the named mutex, and
   this starts the installer before closing, so it may say Wixen Mail is still
   open. That is ledger 340 and it is expected rather than a surprise; record
   whether it happened, what it said, and whether pressing retry worked.
7. The refusal, which matters more than the success. Point the download at a file
   signed by somebody else, or unsigned, and confirm it refuses, says why in a
   way that tells the two apart, deletes the file, offers the releases page, and
   **never asks whether to run it anyway**. A question on this path is a broken
   rule, not a wording problem. Record explicitly whether any appeared.
8. Confirm the development channel offers a prerelease and the public channel
   does not, against a real release list.
9. Kill the program partway through a download, restart it, and confirm
   `%LOCALAPPDATA%\wixen-mail\updates` is empty.

"It worked" is not enough for 4, 5 and 6. Those are about timing and
interruption and are the three no test can approach.

## Measurements, all taken rather than quoted

**The `windows` crate at the pinned version.** Re-run before any code was
written. `Cargo.lock` says `0.62.2`. `Win32_Security_Cryptography` at the
vendored `Cargo.toml:518`, `Win32_Security_WinTrust` at `:531`. `WinVerifyTrust`
at `Win32/Security/WinTrust/mod.rs:48`, `CertGetNameStringW` at
`Win32/Security/Cryptography/mod.rs:808`, `CryptQueryObject` at `:1672`. Every
constant the call needs is in the same module. So this is two features on a crate
already vendored, pinned and audited, rather than a new maintainer to trust or a
parse of PowerShell's output that changes format when Microsoft decides it does.

**Streaming needs no feature, and the plan assumed it did.**
`reqwest::Response::chunk` is at `reqwest-0.13.4/src/async_impl/response.rs:310`
with no `cfg` on it; only `bytes_stream` at `:351` sits behind `stream`. So the
byte bound is enforced against the bytes as they arrive, which is the only way a
bound is worth anything, and `reqwest`'s feature list is untouched. The plan's
premise correction 6 said streaming "needs the `stream` feature"; it does not.

**The size bound is 300 MB against a real installer.**
`dist/Wixen-Mail-Setup-0.40.0+gf7e34777.exe` is 11,604,104 bytes, the only
installer this project has ever built, and `target/release/wixen-mail.exe` is
41,010,688 before compression. So the bound is about twenty-five times the real
installer and seven times the largest thing inside one. Past it the download
stops, the part-file is deleted, and the person is told it was larger than Wixen
Mail will accept.

**Redirects.** At most five followed, and any hop whose scheme is not HTTPS is
refused. A release file needs one hop, from `github.com` to the content host, so
five is slack rather than need. The reason each stopped is carried out of
`reqwest`'s policy callback, which cannot return one of its own, so "the download
failed" never stands in for "it was redirected off HTTPS".

**Where the file goes, and why it is a security decision.**
`%LOCALAPPDATA%\wixen-mail\updates`, through a new `AppPaths::updates_dir`
accessor. Under this application's own root rather than the shared temporary
folder, because a downloaded executable somewhere another local user can write is
a file they can swap between the check and the run, and the check would then have
been made about something that is no longer there. It is verified at the path it
would be run from; `Downloaded` and `Verified` both hold one path and nothing
copies or moves the file.

**Uninstalling really does remove it, read rather than assumed.**
`installer/Wixen-Mail-Setup.iss:535` is
`Type: filesandordirs; Name: "{localappdata}\wixen-mail"`, so the root goes
wholesale and the downloaded installer goes with it. `docs/privacy.md`'s sentence
that uninstalling removes everything is still true and needed no weakening. In
the temporary folder it would have needed one.

**Emptied three ways, and where each happens.** After a handover and after a
refusal, file by file, in `verify` and `say_no` in
`src/service/update_download.rs`. And wholesale at the next start, in
`prepare_data_folder` in `src/main.rs`, which is the one that matters: a program
killed between fetching and handing over runs neither of the other two. The
directory holds one file at most and exists only while an update is being
fetched.

**No `unwrap` or `expect` outside the tests.** Two commands, because the first on
its own does not answer the question:

```
$ grep -n "^#\[cfg(test)\]" src/service/update_download.rs
1070:#[cfg(test)]

$ grep -n "unwrap()\|expect(" src/service/update_download.rs | head -3
1097:        let chosen = the_installer_among(&files).expect("a release to carry its installer");
1107:        let chosen = the_installer_among(&files).expect("a release to carry its installer");
1141:        let dir = TempDir::new().unwrap();
```

The test module starts at line 1070 and the earliest match is at 1097, so every
one of them is inside it. Nothing on any production path unwraps, which matters
here because every one of those paths handles a file a stranger's server sent.
Written as two commands rather than one because the first version of this claim
quoted a filtered grep and was right for the wrong reason: what makes it true is
where the test module begins, and a command that does not print that line is not
evidence of anything.

**Guard record costs, counted with the awk in CLAUDE.md rather than by grep.**
Before this plan: `src/service/update_download.rs` 0 records, because it did not
exist; `src/service/outward.rs` 10; `src/common/paths.rs` **1**, not the 0 the
plan predicted, because 07-01 added one; `src/service/update_check.rs` **1**;
`src/presentation/wx_app.rs` 48. No `#[test]` was added to `outward.rs`,
`wx_app.rs` or `tests/wired.rs`: their counts are 38, 199 and 69 before and 38,
199 and 69 after.

## What the gate selects for each file type touched, checked rather than assumed

Read off `scripts/check.sh`'s mapper and confirmed against the runs.

| File | What the scoped run selects |
|---|---|
| `src/service/update_download.rs` | `cargo test --lib service::update_download::` |
| `src/service/update_check.rs` | `cargo test --lib service::update_check::` |
| `src/common/paths.rs` | `cargo test --lib common::paths::` |
| `src/service/outward.rs` | `cargo test --lib service::outward::` |
| `src/presentation/wx_app.rs` | `cargo test --lib presentation::wx_app::` |
| `src/presentation/ui_types.rs` | `cargo test --lib presentation::ui_types::` |
| `src/service/mod.rs` | `cargo test --lib service::`, the whole service tree |
| `src/main.rs` | **nothing.** It maps to `--lib main::`, which matches no test in the library, because `main.rs` is the binary |
| `docs/*.md` | no target of their own; covered only by the four whole-tree guards |
| `guards/guards.toml` | no target of its own; covered only because `house_style` happens to parse it |
| `Cargo.toml` | no target of its own, and it escalates the whole answer to `all` unless the only changed line is this package's version |

**Two of those are holes and one is new.** `guards/guards.toml` mapping to
nothing is 07-08's finding and is unchanged. `src/main.rs` mapping to nothing is
this plan's, and it bit: the third of the three clearing rules lives in
`prepare_data_folder` in `main.rs`, so the commit that added it ran no test
reaching it. Nothing in the tree can, because a unit test in the binary is not in
the library the suite runs. It is the same shape as the `.iss` hole 07-02 closed
and the workflow hole 07-07 found, and it is why the clearing is tested through
`clear_the_waiting_room` in the library rather than through its caller: the call
site itself is covered by nothing.

**And `Cargo.toml` behaved exactly as the plan's corrected premise says.** The
commit enabling two `windows` features answered `all` and was run detached, 473
seconds. The two commits whose only manifest change was the version bump answered
`affected`. The old sentence, that any `Cargo.toml` change runs the whole gate,
would have been wrong twice.

## Deviations, and why

**1. The plan's file list does not include `src/service/update_check.rs`, and the
work needed it.** The download URL has to come from the release the check already
found, and `Answer::ANewerVersion` carried a version and a page and no files.
`Published` now parses `assets` and the answer carries them. No new test was
added there: four existing tests already parse the real fixture, so they were
widened to say what it publishes, which keeps the per-commit count check quiet
for that file and puts the assertion where the parsing already is.

**2. Two clauses of criterion 2 did not close, found by reading it from
`ROADMAP.md` clause by clause rather than from the plan's paraphrase.** Both were
fixed with their own red half.

The criterion says "Where a signature cannot be checked at all, nothing is
downloaded and nothing is run". Nothing was run, and the download happened first
and the refusal came after. On Linux or macOS that fetches an executable onto
somebody's disk, spends their connection learning something the program already
knew, and throws it away. `whether_a_signature_can_be_checked_here` is now asked
before a single byte. The order is invisible to any test on a machine that *can*
check a signature, because both orders succeed there, so it is read out of the
source with a companion proving the reading sees both a fetch that downloads
first and one that does not.

And the setting's own description said "nothing is downloaded yet". That sentence
is where the consent for an unattended download is given or is not given. It was
true while downloading was 07-09's unwritten work and became a promise that the
thing it warns about does not happen the moment task 1 landed. A warning nobody
believes is worse than none, because the next true one is read the same way. It
now describes what choosing an answer does, and a test holds it in the present
tense.

**This is the deviation worth carrying out of this plan.** The plan said to check
the criteria against the premises. Doing it found a real defect in shipped
behaviour and a false sentence in a shipped control, in work this same plan had
landed four commits earlier. Neither was caught by any test, any guard or any
review, because nothing in the tree reads a criterion.

**3. The ten guard records reading the outward census were not re-measured, and
the plan's criterion asking for it quotes a premise CLAUDE.md has retracted.**
The plan says to re-measure them by hand and detached. CLAUDE.md's decision of
2026-09-03 took guard sweeps off the critical path entirely: "the executor does
not run guards, the merge does not run guards", one sweep once the phase is
complete. The plan is older than that decision in spirit and the criterion was
written from the earlier rule. CLAUDE.md wins, so they were not run and they are
ledger 341, owed to the phase sweep. The narrower reason it is safe: what changed
in `outward.rs` is one string added to a `const` array, which changes what the
census says about one new file and nothing about what any of those ten breaks
redden.

**4. `ANewerVersionIsPublished` was removed rather than left.** With D-16 the
browser-page question is not what an offer leads to any more, so that variant
would have been constructed by nothing. `AnUpdateDidNotHappen` and
`AnUpdateIsReady` replace it. The browser offer survives on the failure path,
which is what the checkpoint's step 7 asks for: a refusal offers the releases
page. That question is about opening a page, never about running the file.

**5. The plan's guard-record table was stale in two rows that mattered.** It says
`src/common/paths.rs` is fingerprinted by 0 records, "so this is free". It is 1,
because 07-01 added one. `src/service/update_check.rs` is also 1. Both were
counted with the awk CLAUDE.md gives rather than by grepping a file name. The
plan's conclusion survives, since neither row is expensive; the premise did not.

## What is still false in the plan after its correction pass

- **Premise correction 6 says streaming needs `reqwest`'s `stream` feature.** It
  does not. `Response::chunk` is available with no feature at 0.13.4, so the
  bound is enforced as bytes arrive at no cost to the dependency at all.
- **Premise correction 8's table says `src/common/paths.rs` is fingerprinted by 0
  records.** One, since 07-01. `update_check.rs` is also 1. The plan told the
  executor to re-measure both rather than trust them, and that instruction was
  the right one.
- **The plan says to name 07-01's paths check and the outward census in the RED
  commit's trailers together.** They cannot be in the same trailer list unless
  the commit touches both files, because the gate runs only the modules a commit
  changes and `red-commit.sh` refuses a name that never ran. They were named in
  the commit that touched `paths.rs`, and a second red commit that carried them
  over was refused, correctly.
- **Task 1's acceptance criterion to re-measure ten records by hand contradicts
  CLAUDE.md**, as above. A criterion phrased as an instruction can outlive the
  rule it was written from.

## Findings worth carrying

**A fixture chosen for being everywhere is not a fixture that exercises
anything.** The refusal that matters needs a validly signed executable somebody
else published, and `notepad.exe` was the obvious choice. Almost every Windows
system binary is signed through a catalogue file rather than inside the file, and
`WinVerifyTrust` asked about a *file* does not look in catalogues, so
`notepad.exe` answers `TRUST_E_NOSIGNATURE`. The test failed, which was luck:
written a little more loosely, as "refused for any reason", it would have passed
for entirely the wrong reason and the one check that separates an update from
running a stranger's program would have had no coverage at all. Four files with
embedded signatures are now tried in order and the test says plainly when none is
available.

**A red stub in a lint-gated compiled language has to keep its own constants
alive.** Clippy runs at `-D warnings`, an unused constant is a build failure, and
suppressing a lint to get a commit through is forbidden here. The two bad ways
out are a suppression that outlives the stub and implementing part of the rule
early, which quietly turns the red green. The stubs named their constants in a
discarded binding, which is self-deleting: the GREEN half replaced the line.

**A break that reddens one test can be a finding rather than a thin record.** The
second guard record's break adds a constructor that takes a path and believes it.
It compiles, no call site changes, and every behaviour test stays green, because
no *behaviour* changes. One test reddens, and it is the source-reading guard,
because the rule lives in a type and a type cannot be observed by running the
program. The record says so.

## Ledger entries

| Entry | Kind | What |
|---|---|---|
| 334 | unrun-verify | no installer has been fetched over a real connection; the transport has never run outside a fixture |
| 335 | unrun-verify | the handover has never run: no window has closed and been replaced |
| 336 | unrun-verify | the publisher check has never seen a genuine Wixen Mail signature; it was proved against a Microsoft-signed file instead |
| 337 | unrun-verify | task 3 not attempted: no screen reader has heard an update, a refusal, or the window closing |
| 338 | unmet-truth | the outward census has no category for a module whose bytes are executed, raised by two plans and acted on by neither |
| 339 | unmet-truth | nothing detects a metered connection, so a channel chosen at home costs about 12 MB over a phone tether |
| 340 | unmet-truth | the handover cannot remove the mutex race: an installer may say Wixen Mail is still open |
| 341 | deviation | the ten outward records were not re-measured; the plan's criterion predates CLAUDE.md taking sweeps off the critical path |
| 342 | unrun-verify | the revocation policy is untested against a revoked certificate, because none exists |

## Criterion 2, clause by clause, against the wording in ROADMAP.md

Read from `.planning/ROADMAP.md` rather than from the plan's paraphrase, which is
what plan 07-07 intended when it wrote the criterion without reading this plan.

| Clause | Closes? |
|---|---|
| can tell the user a newer version exists | yes, 07-05 |
| and can apply it | **structurally, never run.** The path exists end to end and nothing has walked it |
| nothing is fetched that the user did not ask for | yes. Asking is the Help item, which works whatever the setting says, or choosing a channel |
| the setting starts off | yes, unchanged from 07-05 |
| and says where it is chosen that choosing a channel means installers are downloaded automatically | yes, **after being fixed in this plan**. It said the opposite |
| with a channel chosen, the check, the download and the verification happen unattended | yes |
| running the installer never does, and the user is asked first | yes |
| verified before anybody is asked about it | yes, and carried in the type rather than the order |
| the signature must be valid and the signer must be this project's publisher name | yes |
| a file failing either is refused and deleted rather than warned about and offered anyway | yes, tested, and never offered |
| where a signature cannot be checked at all, nothing is downloaded and nothing is run, and the reason is said | yes, **after being fixed in this plan**. It downloaded first |
| declining leaves the current version working | yes, tested |
| a handover that fails leaves the current version working | yes, tested |

**Criterion 2 does not close.** Twelve of thirteen clauses are structurally
complete; the second, "can apply it", is a claim about something that has
happened, and nothing has applied anything. SHIP-02 does not close either. The
honest state is structurally complete and never run, and ledger 334 to 337 hold
it.

## Verification

- `cargo test --lib -- service::update_download:: service::update_check:: service::outward:: common::paths::` passes.
- `cargo test --test house_style` passes, so 07-01's signing guard and the em-dash
  guard are satisfied rather than worked around. One house style guard caught a
  sentence on `docs/ALPHA_TESTING.md` reading as a claim that a new installation
  sends nothing, because it carried "install" and a refusal in one breath.
  Reworded rather than loosened.
- `cargo test --test wired` passes. Nothing added a command, so nothing was owed.
- `scripts/check.sh` passed through the hook on all seven commits, four of them in
  `red` mode. Nothing used `--no-verify`.
- `scripts/check.sh all` on the branch: **473 seconds, all four green**, run
  before the merge and not piped into anything.
- Three guard records re-measured with the command the count check printed, not by
  editing their numbers. Each still reddens exactly the tests it names.
- `.github/workflows/release.yml` is untouched. `git diff --name-only ba2ff758..HEAD`
  does not list it.
- The three pages saying the build is unsigned are untouched where they say so.
  `docs/ALPHA_TESTING.md` gains a bullet about updating being refused, which is a
  different sentence about a different thing.

## Status

`partial`. Tasks 1 and 2 of three. Task 3 is a `checkpoint:human-verify` with
`gate="blocking-human"`, recorded above and not attempted. It cannot run until a
release is published and signed, which is 07-08 task 2 and Pratik's.

Criterion 2 does not close. SHIP-02 does not close. Nothing has been updated.

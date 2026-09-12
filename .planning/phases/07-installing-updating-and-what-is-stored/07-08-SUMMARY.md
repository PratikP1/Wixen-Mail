---
phase: 07-installing-updating-and-what-is-stored
plan: 08
subsystem: infra
tags: [release, installer, inno-setup, authenticode, code-signing, guard-records]

requires:
  - phase: 07-02
    provides: "tests/installer.rs, the target this plan writes into, and the reading helpers for an Inno section and a parameter"
  - phase: 07-07
    provides: "the_promised_files, which follows the files: indirection to the one list the release really publishes, and the guard-record coupling that lets a record naming a non-Rust file reach the gate"
provides:
  - "A census of the seven things that have to be signed, derived from the installer script and the release workflow rather than typed out"
  - "What the local Inno Setup 6 help says about SignTool and SignedUninstaller, written where somebody wiring it will read it"
  - "The finding that the portable copy and the zip are taken after build-installer.sh runs, so they inherit whatever it signed"
affects: [07-09]

actuals:
  tokens: 41000
  tasks: 1
  commits: 2

tech-stack:
  added: []
  patterns:
    - "A completeness census derived by parsing the two lists that decide it, with the entry neither list names read off the directive that governs it"
    - "An external tool's behaviour settled from the local help file rather than from a search summary, recorded beside the directive it governs"

key-files:
  created: []
  modified:
    - tests/installer.rs
    - installer/Wixen-Mail-Setup.iss
    - guards/guards.toml
    - .planning/WINDOWS.md

key-decisions:
  - "The census counts PE files only, .exe and .dll, because those are what Authenticode embeds a signature into. Widening it to a .ps1 or an .msi without settling how each is signed would make it look complete while pairing a file with a step that cannot sign it, so the gap is recorded rather than papered over"
  - "The zip is counted even though a zip cannot carry a signature, because the entry stands for the executable somebody extracts, and leaving it out is how a download becomes the one unsigned thing on the release page"
  - "The uninstaller is read off the Uninstallable directive rather than added as a seventh constant, so a script that turns it off does not carry a phantom entry"
  - "Nothing was signed and no signing tool was configured, which is the plan's shape rather than a shortfall: there is no key yet"

patterns-established:
  - "Where a requirement enumerates what it covers, count the artefacts from the build instead and record the difference as a scope finding"

requirements-completed: []

status: partial
---

# Phase 07 Plan 08: Sign everything this project hands to somebody (task 1 of 3)

**Task 1 only.** What has to be signed is now counted from the build instead of
remembered, and the one unverified fact about how Inno signs an uninstaller is
settled from the local help. **Nothing is signed.** There is no certificate, and
the three shipped pages that say the build is unsigned are untouched and still
true.

## What landed

Seven things have to be signed. SHIP-01's own wording says two.

`tests/installer.rs` now carries a census that derives both halves rather than
holding a list. The `[Files]` block gives what goes inside the installer, the
release workflow's published list gives what goes beside it, and the script's own
`Uninstallable` directive gives the seventh, which is in neither list:

| Origin | Artefact |
|---|---|
| inside the installer | `..\target\release\wixen-mail.exe` |
| inside the installer | `..\search-handler\target\release\wixen_mail_search.dll` |
| inside the installer | `..\search-handler\target\release\wixen-mail-search-setup.exe` |
| published beside it | `dist/Wixen-Mail-Setup-*.exe` |
| published beside it | `dist/wixen-mail-v*.exe` |
| published beside it | `dist/Wixen-Mail-*-windows.zip` |
| written by Inno | `unins???.exe` |

A test holding seven strings would pass when an eighth executable arrived and
never mention it. This one grows and names it, which was proved rather than
assumed: see the quoted failure below.

## Commits

| Commit | Half | What |
|---|---|---|
| `d49c18a1` | RED | the census tests, and a stub answering with nothing |
| `bc33bcd6` | GREEN | the parse, the Inno help findings in the `.iss`, the guard record, the ledger |

Branch: `what-has-to-be-signed-is-counted-rather-than-remembered`, merged at
`a818cf5f`. A third commit, `f603d522`, carries this summary and the two
planning files.

`scripts/check.sh all` was run twice on the branch, because a document commit
landed after the first: **463 seconds** at `bc33bcd6` and **457 seconds** at
`f603d522`, four green both times, redirected to a file rather than piped.

The RED commit named three tests and the gate accepted it:

```
== the tests this commit says must fail ==
   test_the_census_of_what_has_to_be_signed_counts_seven_things
   test_the_census_can_see_an_artefact_that_arrived_without_being_signed
   test_every_guard_record_says_how_many_tests_the_files_it_names_held
```

The third is the collision `CLAUDE.md` describes. Two tests added to a file that
three guard records fingerprint takes the count check red, and its remedy needs
the green code before a record's red list can be corrected, so no ordering of
this task has a clean tree around its red half.

## The census taken red by hand

An eighth `Source:` line was added to the `.iss`, the census was run, and the
line was taken out again. It did not pass and did not merely fail, it named the
new file:

```
assertion `left == right` failed: the [Files] entries that are code have
changed, and each one is something a person runs from an installed folder
  left: ["..\\target\\release\\wixen-mail.exe",
         "..\\search-handler\\target\\release\\wixen_mail_search.dll",
         "..\\search-handler\\target\\release\\wixen-mail-search-setup.exe",
         "..\\tools\\target\\release\\wixen-mail-crash-reporter.exe"]
 right: ["..\\target\\release\\wixen-mail.exe",
         "..\\search-handler\\target\\release\\wixen_mail_search.dll",
         "..\\search-handler\\target\\release\\wixen-mail-search-setup.exe"]
```

## What the Inno help really says, and how A3 was half wrong

Read on 2026-09-12 from `ISetup.chm` in the per-user Inno Setup 6 install at
`%LOCALAPPDATA%\Programs\Inno Setup 6`, topics "[Setup]: SignedUninstaller",
"[Setup]: SignedUninstallerDir" and "[Setup]: SignTool". The file was extracted
to a scratch directory and read as text; `hh.exe -decompile` produced nothing
from this shell and 7-Zip did the job. Nothing in the repository was touched to
get it.

`07-RESEARCH.md`'s assumption A3 says Inno's "`SignedUninstaller` two-pass,
prompting behaviour can be made to work non-interactively in GitHub Actions",
recorded at medium confidence because the help had been read through a search
summary. **The behaviour is real and it is not a property of
`SignedUninstaller`.** It is what a build with no `SignTool` gets.

Four things the help settles:

1. `SignedUninstaller` defaults to `yes` when a `SignTool` is set and `no`
   otherwise, so setting `SignTool` is enough on its own and the directive need
   never be written.
2. With `SignTool` set, the help says the uninstaller "will be signed
   automatically on the fly". One pass, no prompt, nothing to answer.
3. The two-pass prompting branch is `SignedUninstaller=yes` with no `SignTool`.
   Inno writes a uniquely named non-temporary copy into `SignedUninstallerDir`,
   which defaults to `OutputDir`, and asks for it to be signed by hand. Later
   compiles reuse that signature silently, and changing `SetupIconFile`,
   `WizardStyle` or the `VersionInfo` directives, all three of which this script
   sets, makes a new file under a new name and asks again.
4. So a CI job that signs at all cannot hang here. The hang A3 feared needs a
   job that sets `SignedUninstaller` without setting `SignTool`, which is a
   configuration nobody would write on purpose.

**One consequence nobody predicted, and it changes the installed folder.** When
the uninstaller is signed, Setup writes the language messages into a separate
`unins???.msg` file, because embedding them in the EXE would invalidate the
signature. Anything that reads or erases the installed folder should expect it.

Two further notes off the `SignTool` topic, both for task 3. The tool has to be
registered with ISCC through its `/S` parameter or the compile fails outright.
And `[Files]` entries take `sign` and `signonce` flags, so Inno can sign the
source files it is about to package, which is an alternative to
`scripts/build-installer.sh` signing them first.

All of this is written into `installer/Wixen-Mail-Setup.iss` as a comment where
`SignedUninstaller` will go, so whoever wires it reads it there rather than here.

## The portable copy inherits, and this is why

`.github/workflows/release.yml`, steps in the order they run:

```
      - name: Build the release binary and the setup executable
        run: bash scripts/build-installer.sh

      - name: Prepare the portable download
        run: |
          Copy-Item "target/release/wixen-mail.exe" "dist/wixen-mail-$tag.exe"
          Compress-Archive -Path "target/release/wixen-mail.exe" ...
```

`build-installer.sh` builds `target/release/wixen-mail.exe` and then runs ISCC.
The portable copy and the zip are taken from that same file afterwards. So if
task 3 signs the three binaries inside `build-installer.sh`, between the build
and ISCC as the plan says, **the portable copy and the zip inherit the signature
and need no second signing.** They would not if the copy were taken first.

That is a claim about the file, not about a release. No release has ever been
cut here, so nothing has verified that the copy really carries the signature
through. Task 3 owes that proof, and the plan already says it: verify the
published copy rather than the source binary.

## What the gate selects for each file this touched

Checked rather than assumed, because three separate instances of this exact
blind spot have been found in this phase.

| File | Kind | What `which-checks.sh` answers | What really runs |
|---|---|---|---|
| `tests/installer.rs` | Rust integration target | `affected` on a branch | `cargo test --test installer`, by the `tests/*.rs` arm of `check.sh`'s mapper |
| `installer/Wixen-Mail-Setup.iss` | not Rust | `all` | the whole gate, from the `*.iss` arm added by 07-02 |
| `guards/guards.toml` | not Rust, not a document | `affected` | no target of its own; it is read by the guard coupling and by the tree-reading guards, which run in every mode |
| `.planning/WINDOWS.md` | document | `docs_only` alone | the document-reading targets |

The GREEN commit carries the `.iss`, so it answered `all` and ran the whole gate
including the release build. That is the strongest of the four answers and it
swallowed the other three, so nothing in this plan ran narrower than it should
have.

**One thing worth saying about `guards/guards.toml`.** It answers `affected` and
maps to no target, so the file that defines every recorded break is checked only
by whatever tree-reading guard happens to parse it. That is correct today,
because `house_style` really does parse it and runs in every mode. It is not
correct by design, and a reader could take the `affected` answer as coverage the
way the three earlier instances were taken.

## Deviations

**1. Two guard records were stale, one of them inside a single wave.**

`a published glob still matches a name the release really writes` was written by
07-07 one wave ago. Its break changes `dist/Wixen-Mail-*-windows.zip` to
`dist/Wixen-Mail-*-win.zip`, and the census added here reads the same published
list and asserts the globs it holds. So the break now reddens two tests where the
record names one. Measured by hand before `--remeasure` ran:

```
test test_the_census_of_what_has_to_be_signed_counts_seven_things ... FAILED
test test_every_file_the_release_promises_is_one_it_really_produces ... FAILED
test result: FAILED. 11 passed; 2 failed
```

The record was corrected by hand first, as `CLAUDE.md` requires, and then
`--remeasure` confirmed it: "all 2 tests named went red, and nothing else did".
Had it been left, `--remeasure` would have refused it and the cause would have
read as a broken tool.

This is the case `CLAUDE.md` warns costs the most. Nothing about 07-07's own work
moved. A plan one wave later added a test reaching the same rule, and the record
it made short is not the record that plan was writing, so the only person who
will ever be looking at the right moment is the author of the new test.

**2. Two premises of this plan were already false when it ran.**

The plan says, re-checked 2026-09-11, that `tests/house_style.rs` holds 18 guard
records and 67 `#[test]`. Measured on 2026-09-12 with the `tests_last_seen`
parse: **19 records and 69 tests.** Both moved in the day between, which is this
project's own point about a dated measurement, arriving as a small bill.

The plan also cites `docs/installing.md:7` for the "not yet code signed"
sentence. It is at `:8`. That file was corrected once already in this plan's own
correction pass, which is the shape of the finding: a correction pass reads for
claims and a line number is not phrased as one.

**3. The append tool corrupts a ledger entry containing a backslash.**

`gsd-tools windows append` writes the description into both halves of
`.planning/WINDOWS.md` without reconciling the escaping: the JSON half escapes a
backslash and the markdown table half does not, so the two disagree and the
commit is refused. Entry 332 quoted a Windows path from the installer script and
hit it. `test_both_halves_of_the_ledger_say_the_same_thing` caught it, which is
the guard working, but the message reads as a ledger the author corrupted rather
than as a tool defect. Worked around by rewording the entry to avoid the
character and correcting both halves by hand. Recorded as ledger 333.

## The guard record

`the census still sees every kind of file that has to be signed`, break:

```
before = 'const WHAT_AUTHENTICODE_SIGNS: [&str; 2] = [".exe", ".dll"];'
after  = 'const WHAT_AUTHENTICODE_SIGNS: [&str; 1] = [".exe"];'
```

The half-fix rather than the absent one. The census still runs, still reads both
lists, still counts, and has stopped seeing one kind of file. Deleting the test
would be caught by anybody; a census that counts six of seven and says nothing is
the failure this whole task exists to prevent, applied to the instrument.

Measured by hand on the tree the GREEN commit makes,
`cargo test --all-targets --no-fail-fast` at the default eight threads. The full
red list is one test:

```
test test_the_census_of_what_has_to_be_signed_counts_seven_things ... FAILED
test result: ok. 7077 passed  (the library)
49 other targets green
```

The count check was also red in that run, for the unrelated and expected reason
that the commit adds two tests to a file three records fingerprint. It was
cleared by `--remeasure` afterwards and is not part of this break. The
credential-store flake of ledger 328 did not appear in any run today.

`scripts/guards.sh` was then run on the record itself: "the one test named went
red, and nothing else did".

**Worth knowing and not worth fixing.** The companion,
`test_the_census_can_see_an_artefact_that_arrived_without_being_signed`, does not
redden under this break. Its fixtures are all `.exe`, so it proves the reading
can see an artefact that arrived and cannot see the filter narrowing underneath
it. The census over the real tree is the half that catches that, which is why the
pair is a pair.

## Measurements taken today, not quoted

| Thing | Before | After |
|---|---|---|
| `guards/guards.toml` records | 731 | 732 |
| census lines 79 and 80 | 192 and 539 | 192 and 540 |
| records naming `tests/installer.rs` | 3 | 4 |
| `#[test]` in `tests/installer.rs` | 11 | 13 |
| records naming `tests/house_style.rs` | 19 | 19 |
| `#[test]` in `tests/house_style.rs` | 69 | 69 |

No `#[test]` was added to `tests/house_style.rs`, as the plan asked. Its count is
unchanged in both directions.

`scripts/check.sh all` on the branch before the merge: **463 seconds, all four
checks passed.** Not piped; redirected to a file and its exit status read.

## Nothing was signed, and the diff says so

```
$ git diff --stat main...HEAD
 .planning/WINDOWS.md           |  58 ++++++-
 guards/guards.toml             |  64 +++++++-
 installer/Wixen-Mail-Setup.iss |  46 ++++++
 tests/installer.rs             | 347 +++++++++++++++++++++++++++++++++++++++++

$ git diff main...HEAD -- .github/workflows/release.yml
(empty)

$ git diff main...HEAD -- docs/ALPHA_TESTING.md docs/BETA_RELEASE.md docs/installing.md
(empty)
```

`scripts/build-installer.sh` is not in the diff either. No `SignTool` directive,
no `SignedUninstaller` setting, no signing step anywhere. The only mention of
either is a comment saying what the help says about them.

The `on:` block of `release.yml` is unchanged because the whole file is, and
07-07's `test_a_release_still_happens_only_when_somebody_asks_for_one` passes.
Nothing here widens when a release can happen. Nothing here can make one happen.

No signing credential appears in the tree, in a commit message, in a log or in
this summary. There is none to appear.

## The three documents are untouched and still true

```
docs/ALPHA_TESTING.md:342:- **The installer is not signed**, so Windows will warn about it.
docs/BETA_RELEASE.md:9:Every alpha and beta build is unsigned, so everyone who downloads one meets
docs/installing.md:8:Wixen Mail is not yet code signed, so Windows does not recognise it.
```

Correcting these is task 3's last step and it comes after a signature has been
produced and verified, not before. Changing them now would put a false statement
into shipped documentation, which is the precise thing this plan's ordering
exists to prevent.

## The checkpoint is open: the signing account

Task 2 is a `checkpoint:human-action` and nothing in this repository can do any
of it. It needs an Azure subscription, an identity verification naming a real
person, about ten dollars a month, and a role granted in a tenant. Pratik is
obtaining the certificate; this is written so it can be acted on later without
re-reading the plan.

**The order matters, because one step waits and the rest are minutes.**

1. Create or choose an Azure subscription, and create an **Artifact Signing**
   account in a region that offers it. The service used to be called Trusted
   Signing and was renamed, so a search for the old name finds stale pages. The
   resource provider is still `Microsoft.CodeSigning`.
2. **Complete identity verification for an individual. Start this first.** It
   goes through a third-party verifier against the Azure billing account and it
   is the only step with a waiting time.
3. Create a **certificate profile** of the public-trust individual type, so the
   certificate subject carries Pratik's own name rather than an organisation's.
4. Create the identity that will sign: an app registration for GitHub Actions
   with a **federated credential**, and whatever a local signed build on this
   machine needs. Prefer federation over a stored client secret. A secret is a
   thing that can leak from a settings page or a log; a federated credential has
   no material to leak, and SHIP-01 requires that no key reaches the repository
   or the build log.
5. **Grant that identity the Trusted Signing Certificate Profile Signer role on
   the certificate profile. This is the step people miss.** Creating the profile
   and creating the identity that signs with it are two separate things, and an
   identity without the role fails with an authorisation error that reads like a
   bad endpoint.
6. Add the endpoint, account name, profile name, tenant id and client id as
   repository secrets. Names only; no value belongs in any file here.
7. Authenticate this machine so a local build can sign. Task 3 needs a local
   signed build, because dispatching a release to prove a signature would be
   publishing as a side effect of a plan, which guardrail 7 forbids.

When it is done, the thing task 3 needs first is **the certificate subject name
exactly as it appears**, because that is the name a user will see and it has to
match `AppPublisher` in `installer/Wixen-Mail-Setup.iss`, which is
`Pratik Patel`, and `CompanyName` in `build.rs`, or be reconciled with them on
purpose.

## Ledger entries

| Entry | Kind | What |
|---|---|---|
| 330 | deviation | SHIP-01's wording names two artefacts and the build produces seven, recorded as a scope finding rather than a defect |
| 331 | unmet-truth | the census counts PE files only, so a `.ps1`, `.msi` or `.cat` added later would not be counted |
| 332 | unmet-truth | one `Source:` line is read as one artefact, so an executable wildcard would be right about the line and wrong about the artefacts |
| 333 | deviation | the append tool escapes a backslash in the JSON half of the ledger and not in the table half |

Entry 329, which records that criterion 1 waits on an Azure account, was written
by 07-07 and is not duplicated here. It is still open and this plan did not
change that.

## What is still false in the plan after its correction pass

Three things, all small and all the same shape: a number or a line that was true
when it was checked and moved before it was used.

- `tests/house_style.rs` is 19 records and 69 tests, not the 18 and 67 the plan
  re-checked on 2026-09-11. The plan's conclusion survives, since the point was
  that adding a test there would cost 18 records and about half an hour, and at
  19 that is still the right reason not to.
- `docs/installing.md:7` is `:8`.
- The plan's guard-record table has an empty row for `tests/installer.rs` with
  the note that it "does not exist in the tree these figures were taken from,
  and still does not on 2026-09-11". It did exist by the time this ran, with
  three records naming it, which is the cost the plan told the executor to
  re-measure rather than trust. That instruction worked.

## Verification

- `cargo test --test installer` passes, 13 tests.
- `scripts/check.sh` passed on both commits through the hook, `red` mode on the
  first and `all` on the second. Nothing used `--no-verify`.
- `scripts/check.sh all` on the branch before the merge: 463 seconds, four green.
- `cargo test --test house_style` passes, 69 tests, after `--remeasure`.
- `scripts/guards.sh` on the new record: the one test named went red, nothing
  else did.
- Nothing in this plan was run against a real certificate, a real signing tool, a
  real install or a real release, because none of those exists yet. The census is
  a claim about what two lists say and the module doc in `tests/installer.rs`
  says so in those words.

## Status

`partial`. Task 1 of three. Task 2 is open and is Pratik's. Task 3 was not
attempted and must not be until there is a certificate.

Criterion 1 does not close. SHIP-01 does not close. Nothing is signed.

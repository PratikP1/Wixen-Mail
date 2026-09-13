---
phase: 07-installing-updating-and-what-is-stored
kind: defect-remediation
plan: none
subsystem: infra
tags: [ci, toolchain, clippy, cargo-audit, guards, environment-dependent-tests]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: tests/installer.rs holding release.yml's trigger block byte for byte
  - phase: 08-every-number-the-project-quotes
    provides: 08-RESEARCH.md, which found CI red and recorded it rather than fixing it
provides:
  - rust-toolchain.toml pinning 1.98.1, with the components the gate needs
  - Every dtolnay/rust-toolchain call site naming that same version
  - A check comparing the pin file against every workflow, with a companion proving it can see a disagreement
  - The two clippy sites 1.98.1 added a lint for, fixed without an allow
  - Two withdrawal tests that say what they assume instead of testing the machine
  - scripts/audit.sh and its suite, running cargo audit from the local gate
  - CLAUDE.md, docs/IMPLEMENTATION_STATUS.md and scripts/check.sh corrected about what a local run covers
  - other-platforms.yml's comment corrected, keeping the half of it that was right
affects: [08-every-number-the-project-quotes]

branch: what-ci-compiles-with-is-what-this-machine-compiles-with
date: 2026-09-13

key-files:
  created:
    - rust-toolchain.toml
    - scripts/audit.sh
    - scripts/audit.test.sh
  modified:
    - .github/workflows/ci.yml
    - .github/workflows/accessibility.yml
    - .github/workflows/mutants.yml
    - .github/workflows/nvda.yml
    - .github/workflows/other-platforms.yml
    - .github/workflows/release.yml
    - tests/house_style.rs
    - src/service/signed_mail.rs
    - src/service/protocols/imap/mailbox_name.rs
    - src/service/safebrowsing/database.rs
    - scripts/check.sh
    - guards/guards.toml
    - CLAUDE.md
    - docs/IMPLEMENTATION_STATUS.md
    - .planning/WINDOWS.md

ledger-entries: [354, 355]
---

# CI drift, 2026-09-13

CI had been red since 2026-09-10 and `main` was 198 commits ahead of what CI has
judged. Three of seven jobs failed. Two of them are fixed here. The third,
Security Audit, is left failing on purpose, because the advisory behind it is a
decision nobody has made.

Nothing here is user visible, so there is no `docs/changelog.md` entry and no
version bump. The product does the same things today that it did yesterday. What
changed is which compiler builds it, what the gate checks, and what two tests
claim.

## How the action and the pin file interact

This was the question the work turned on, and it was measured rather than
assumed, because the answer decides whether a pin file is worth anything at all.

`dtolnay/rust-toolchain` runs two commands: `rustup toolchain install` and then
`rustup default`. It sets no directory override and no `RUSTUP_TOOLCHAIN`. Read
from the action's own `action.yml` on 2026-09-13.

`rustup default` is the lowest of rustup's precedences. Measured the same day by
putting a `rust-toolchain.toml` naming 1.88.0 into an empty directory while the
default was stable, and asking:

| what is set                       | what `rustc --version` answers |
| --------------------------------- | ------------------------------ |
| default only                       | the default                    |
| default plus a pin file            | the pin file                   |
| plus `rustup override set`         | the override                   |
| plus `RUSTUP_TOOLCHAIN`            | the environment variable       |

So the pin file wins in CI whatever a workflow says, and rustup installs the
pinned compiler on first use with the components the file names.

The workflows name 1.98.1 anyway, at all thirteen call sites. Three reasons, and
the third is the one that settled it. The run log then says which compiler really
built the tree rather than saying `stable` while building with something else. No
job downloads a second toolchain it will not use. And a check comparing two places
needs both places to name something: if the workflows said `stable` there would be
nothing to compare the pin against.

Worth knowing: the brief said seven call sites in `ci.yml`. There are six, in
seven job instances, because `Build` is a two-way matrix. Thirteen across the six
workflows.

## What the check holds

`tests/house_style.rs::test_ci_and_this_machine_are_told_to_use_the_same_compiler`
reads `rust-toolchain.toml` and every file under `.github/workflows`, walked off
the directory rather than listed, so a seventh workflow is covered the day it
arrives. It holds three things:

1. Every `dtolnay/rust-toolchain@<ref>` names the version the pin file names.
2. The reading found at least one call site. A corpus walk that matches nothing
   passes unconditionally, and this project has been caught by that twice.
3. Every `components:` a workflow asks for is also in the pin file.

The third is not decoration and is the consequence most likely to bite. Because
the pin file wins, the compiler the build uses is the one rustup installs from
the pin file, with the pin file's components, not the one the action installed
with whatever a step asked for. A workflow asking for `clippy` while the pin file
did not install it would produce a Clippy job with no clippy.

`test_the_compiler_agreement_check_can_see_a_disagreement` is the companion. It
feeds the reading the drift in the spelling it really had, and the corrected
spelling, and a commented-out line, and a workflow that installs no compiler, and
asserts what each produces. Two guard records cover the two halves of the
reading, each measured with `scripts/guards.sh` rather than written from
guesswork; both redden exactly the two tests they name.

## The two clippy sites

1.98.1 added `clippy::chunks_exact_to_as_chunks`, and clippy runs with
`-D warnings`. Reproduced here under 1.98.1 before anything was changed: exactly
two, exactly the ones the brief named.

- `src/service/protocols/imap/mailbox_name.rs:87`. Takes the replacement the lint
  suggests, and also drops the separate `bytes.len() % 2 != 0` check above it.
  The remainder `as_chunks` returns is the same fact the modulo was computing, so
  the pairing is now read once instead of twice.
- `src/service/safebrowsing/database.rs:182`. Takes the replacement directly. A
  trailing part-prefix is still dropped, which is what `chunks_exact` did and what
  the doc comment above it says should happen to a half-written file.

No `#[allow]` anywhere. `as_chunks` was stabilised in 1.88, which is the floor
`Cargo.toml` declares, so the fix does not raise it.

## The two withdrawal tests

The brief said these fail on CI and pass here, and asked which of the two things
they were. The CI log for job `102839909363` says, and it is not what reading the
code suggests:

```
test_none_of_this_computers_own_authorities_is_reported_as_withdrawn
  assertion `left != right` failed
    left: Withdrawn
   right: Withdrawn

test_the_withdrawal_question_really_reaches_windows_own_answer
  not one of the 3 authorities this machine holds got a per-certificate
  answer out of Windows, so nothing here is reading its verdict
```

They failed for two different reasons and only one was a fault in the test.

**The first was testing the machine.** It asserted that no root this machine
trusts comes back withdrawn. On the runner one really does. Windows keeps a list
of authorities it has stopped trusting, a root on that list is still physically in
the store, and the chain engine sets the revoked bit on it. That is a true fact
about that machine and the test called it a defect.

It is renamed to `test_the_withdrawal_question_does_not_manufacture_bad_news`,
which is what its own comment always said it was for, and now asserts the thing
that holds on any machine: a call that invents bad news says it about everything,
and a machine whose every root is distrusted trusts nobody and could not reach a
mail server. It prints how many were distrusted, visible with `--nocapture` and
nowhere else, which is the right level of noise for a fact about the machine that
is not a finding.

**The second had a bug and a precondition tangled together.**

The bug: it counted a certificate as answered only when Windows said "not
withdrawn" or "the authority could not be reached". Those are two of the five
things Windows can say, and every one of the five is Windows answering. A
certificate carrying no withdrawal address gets a verdict saying exactly that, and
that verdict is readable only through the pointer walk this test exists to guard.
The old predicate threw it away. It now asks the question the walk really settles:
did anything ask at all. The one answer that means nothing asked is named once, as
`windows_store::NOTHING_ASKED_ABOUT_WITHDRAWAL`, so the test and the code read one
string instead of two that can drift apart.

The precondition: Windows does not check withdrawal for the root of the chain it
built, so an intermediate certificate whose issuer this machine does not hold is a
one-element chain, is its own root, and gets no check. For those, "nothing asked"
is the true answer and is indistinguishable from a broken walk. The test's
precondition and its claim were the same assertion, which is why its failure
message confidently accused the code.

So the precondition is now read from a signal the walk has no part in:
`issuer_trust`, which comes off the chain's error bits. An authority this machine
trusts chained to something, so its chain has more than one element, so Windows
checked it, so a walk that works must produce a verdict for it. Where no such
authority exists the test says the question cannot be put here and stops.

**Both still fail for the reason they were written**, and that is measured rather
than asserted. Two guard records name the break that must redden each one:

| break                                                     | reddens |
| --------------------------------------------------------- | ------- |
| the revoked-bit test replaced by `if true`                  | 8 tests |
| the walk replaced by `None`                                 | 1 test  |

The first was written naming one test and measured as reddening eight, two of them
end-to-end tests under `application` that nobody thinking about a certificate store
would have filtered for. The record names what really went red.

## What this costs, said plainly

On a machine holding no intermediate authority it trusts, the walk guard is off.
That is ledger 355 rather than a sentence in a commit nobody will find again. It
is off in the one place the walk was never the risk: this is Windows-specific code
exercised on Windows machines, and a developer's machine holds dozens of such
authorities. This one holds seventeen.

## cargo audit, and how it avoids blocking on a decision nobody has made

`scripts/audit.sh` runs `cargo audit` from the `all` mode of the gate, which is
every code commit on `main` and every pre-merge run. It costs about six seconds of
a run that is about 330.

There are two lists and they are deliberately not one list.

`.cargo/audit.toml` holds advisories this project has **accepted**, each with a
reason and an exit condition. cargo-audit silences those everywhere, here and in
CI, which is what accepting one means.

`scripts/audit.sh` holds one that has been **seen and not decided**:
`RUSTSEC-2023-0071`, rsa 0.9.10, the Marvin timing sidechannel, 5.9 medium,
arriving through `pgp 0.20` with no fixed upgrade available. Putting it in
`audit.toml` would decide it by silencing CI, and that decision belongs to whoever
owns the dependency. So it is set aside by name on this machine only, the local
gate prints it on every run as something nobody has decided, and the Security Audit
job stays red. Red is the right state for an open question.

The obvious hole in that arrangement is guarded. A list of advisories held open
becomes a permanent exception the moment one of them stops being reported, so
every run checks that each entry is still really in the output and refuses when one
is not. `scripts/audit.test.sh` holds both directions, including the case where a
clean run is refused because the gate is still holding something open.

`rsa` is untouched. No dependency was added, removed or changed.

## Documents corrected

`CLAUDE.md` said `scripts/check.sh` runs "the same four checks CI runs". CI runs
seven jobs. A full local run covers five: Rustfmt, Clippy, the `scripts/*.test.sh`
suites, the Test Suite, the release half of Build, and now Security Audit. It does
not run the debug build, the setup executable, or the search handler's own checks.
Both the old claim and its date are left visible.

`docs/IMPLEMENTATION_STATUS.md` carried the same count and is corrected the same
way.

`scripts/check.sh` now ends by printing which five jobs the run covered and which
two it left to CI, so the answer comes from the run rather than from a page
somebody has to remember to correct. Its header comment is corrected too.

`.github/workflows/other-platforms.yml` carried a comment arguing for `stable`,
which said that pinning above 1.88 "would turn on clippy lints this tree has never
been held to, and clippy runs with -D warnings". That risk is real and the comment
is kept saying so. What it did not know, written on 2026-09-12, is that the thing
it warned about had already happened: CI had been red on exactly that since
2026-09-10. Asking for whatever is newest does not avoid new lints, it takes them
on a day nobody chose, in a run nobody is watching.

## Recorded, not fixed

**Ledger 354.** Nothing in this repository builds at the declared floor.
`Cargo.toml` says `rust-version = "1.88"` and every workflow and the pin file now
use 1.98.1, so 1.88 is a claim no build has ever tested. Whether an MSRV job
belongs here is a scoping question and it is not settled by this work.

**Ledger 355.** The walk guard is off on a machine with no trusted intermediate
authority, as above.

**A guard record found short.** Measuring the new records ran an unrelated one as
well, and it came out short: breaking the spellcheck language list reddens a fourth
test, in `data::config`, which no count of the spellcheck module could ever have
predicted. Corrected by hand and re-measured; it now reddens exactly the four it
names. This is the third of the three limits the header of `guards/guards.toml`
lists, happening in the open.

## What in the brief turned out wrong

- Seven call sites in `ci.yml`. Six, in seven job instances.
- 180 commits ahead of `origin/main`. 198, measured 2026-09-13.
- The guard census. The brief quoted 617 records, dated 2026-09-06. The file held
  741 before this work and 745 after, four records having been added. Counted with
  a TOML reader, as rule 4 asks, because the awk in `CLAUDE.md` misses the inline
  form.
- The cause of the second test failure. The brief said both tests "ask Windows
  whether a certificate has been withdrawn, and the runner's certificate store is
  not this machine's", which is true and reads as one fault. They are two, one of
  which is a plain bug in a predicate that would have failed here too if this
  machine's store had been slightly different.

## What is not verified

**The CI half of the test fix.** Nothing is pushed, so nothing here has been run
on a runner. The `signed_mail` changes are reasoned against the run log of
`34467657848` and cannot be confirmed against a green run until somebody pushes.
The corrected predicate makes the test pass on the runner if any of its three
intermediate authorities is trusted; the precondition makes it stop rather than
fail if none is. Both paths avoid the false failure, and which one applies there is
not known from here.

**That the pin is honoured by an actual runner.** The mechanism is measured at both
ends, the action's source and rustup's precedence, on this machine. A run is what
would settle it.

## The gate run

`scripts/check.sh all` on the branch at `ba370f8c`, redirected to a file rather
than piped, under the pinned compiler:

```
rustc 1.98.1 (48a229cea 2026-09-01)
== rustfmt ==
== clippy ==
== the scripts that decide what runs ==
== security advisories ==
No advisory outside .cargo/audit.toml, apart from 1 nobody has decided yet:
    RUSTSEC-2023-0071 (CI is still red on this, on purpose)
== tests ==
== release build ==
```

Green, in 9m45s from a warm tree, of which the release build was 3m22s. Read that
as one measurement on one machine on one day. The compiler line is there because
a gate run that does not say which compiler it used is the thing this whole piece
of work is about.

## Commits

| commit     | what                                                             |
| ---------- | ---------------------------------------------------------------- |
| `ce9e2706` | RED: the agreement check and its companion                        |
| `6e31f895` | the pin, thirteen call sites, both clippy fixes, two guard records |
| `84e84e86` | RED: the advisory gate's suite                                    |
| `d109223e` | scripts/audit.sh, wired into the gate, and the count corrections   |
| `ba370f8c` | the two withdrawal tests, two guard records, two ledger entries    |

Two red commits, each held by `scripts/red-commit.sh` to exactly the failures it
named. Nothing used `--no-verify`. No tracked file was edited by a script.

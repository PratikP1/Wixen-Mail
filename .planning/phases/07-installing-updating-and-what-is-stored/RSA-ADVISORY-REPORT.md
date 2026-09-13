---
phase: 07-installing-updating-and-what-is-stored
title: The rsa advisory decided, and a check that keeps every acceptance honest
tags: [cargo-audit, advisories, rsa, pgp, dependency-review, guards]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: scripts/audit.sh and .cargo/audit.toml, written on 2026-09-13
provides:
  - RUSTSEC-2023-0071 accepted in .cargo/audit.toml with its reasoning and an expiry condition
  - A check that every accepted advisory is still really reported, with the companions that keep its reading honest
  - Four dead acceptances removed, all four found by that check on its first run
  - The held-open mechanism kept and made correct with an empty list
affects: [08-every-number-the-project-quotes]

branch: what-this-project-accepts-is-still-what-the-run-reports
date: 2026-09-13

key-files:
  modified:
    - .cargo/audit.toml
    - scripts/audit.sh
    - scripts/audit.test.sh
    - .planning/WINDOWS.md

ledger-entries: [356, 357, 358, 359]
---

# The rsa advisory, 2026-09-13

`RUSTSEC-2023-0071` sat in `scripts/audit.sh` as seen and not decided from
2026-09-10, when the local audit gate was written, until today. It is now
accepted in `.cargo/audit.toml`, in writing, with an expiry condition.

The larger result is not that decision. It is that `.cargo/audit.toml` held
seven acceptances and nothing had ever asked whether any of them still applied.
Four of the seven had already stopped applying. Nobody had decided to remove
them and nothing would have said so.

Nothing here is user visible. The product does today what it did yesterday, so
there is no `docs/changelog.md` entry and no version bump. What changed is what
the gate checks and what one advisory is recorded as.

## The decision, and where it lives

The acceptance is written out in `.cargo/audit.toml` and is not repeated here.
Read it there: it is the thing a person re-judging this will open, and a second
copy in a phase document is a second copy to go stale.

What it says, in one paragraph. The advisory arrives through `pgp 0.20.0`, which
cannot drop `rsa`: the dependency is not optional and no feature excludes it,
and an OpenPGP implementation without RSA cannot read most existing PGP mail.
There is no fixed release; the advisory's `patched` list is empty. An
alternative does exist, `sequoia-openpgp` with its Nettle backend, and it was
refused on licence and build cost rather than on security, so the entry says so
instead of claiming there is nothing else. Accepting is right today because the
attack needs roughly 275,000 adaptive queries each answered with an observable
success or failure, and this program gives a sender no per-query feedback at
all. The entry then says what would make that false.

Three things in it are worth pulling out, because they are the parts a summary
usually loses.

**The advisory's title undersells it, and the entry says so.** "Marvin Attack:
potential key recovery through timing sidechannels", 5.9 medium, is one line
covering four separate concerns. Two are fixed and two are open, and the open
one that matters is a behavioural oracle read off an `Ok` or `Err` return value
with no timing involved at all: 275,490 queries for a full RSA-1024 plaintext
against published `rsa 0.10.0-rc.18`, plus a forged signature over a chosen
message. Somebody re-judging this needs the oracle and the query count. The
severity score does not carry either.

**The exposure argument is written as an argument, not as reassurance.** It
names the channel the attack needs and says this program does not provide it.
It does not say the code is safe.

**The expiry condition names a line, not a feeling.** The day anything here
answers a sender differently depending on whether a PGP body decrypted, the
oracle is real. The specific thing to watch for is
`receipt_for_the_open_message` in `src/presentation/wx_app.rs` starting to read
`crate::service::pgp::WhatOpeningItFound`. Whoever adds a feature like that in a
year will not connect it to a security advisory, so the condition is written in
the words they would recognise.

**What to watch is PR 680, not the advisory.** The advisory clears only when all
four concerns close and a version ships. PR 680 is the one that closes the
behavioural oracle, and the oracle is what the argument turns on.

### The one claim that was checked rather than assumed

"This program sends nothing back" is an absence claim, which is the cheapest
sentence to write and the most expensive to be wrong about, so it was traced.

This program does have an automatic outbound response, and it is a read receipt.
`receipt_for_the_open_message` reads `Disposition-Notification-To`, which PGP
leaves in the clear because it encrypts the body and not the headers, and it is
called at `src/presentation/wx_app.rs:3046` when a message is selected, after
the body load is dispatched and without reference to what came back. So a sender
who sets that header and sends 275,000 messages gets 275,000 identical answers,
every one of which says the message was opened and none of which says it
decrypted. The setting is `Never` by default, and a receipt to anything in the
junk folder is refused whatever the setting says.

That is why the argument holds, and it is also exactly where it would stop
holding. An ordering that looks incidental is doing the work.

## The check, and the four acceptances it found dead

`scripts/audit.sh` already required every advisory in its held-open list to
still be really reported, and its own comment says why: a list of exceptions
that quietly empties is a permanent hole shaped like bookkeeping. That guard sat
on the list with one entry in it. The list with seven entries, which silences
advisories in CI as well as locally, had none.

That is the wrong way round. An `ignore` entry names an advisory id. An entry
that has stopped applying goes on silencing that id, so the day the same crate
picks up a second advisory, the entry hides it. A stale held-open entry costs a
red CI job nobody needed; a stale acceptance costs a finding.

On its first run the check named four:

| Advisory | Named | Why it stopped applying |
|---|---|---|
| RUSTSEC-2026-0098 | rustls-webpki 0.101.7 | The tree resolves rustls-webpki 0.103.13 |
| RUSTSEC-2026-0099 | rustls-webpki 0.101.7 | Same |
| RUSTSEC-2026-0104 | rustls-webpki 0.101.7 | Same |
| RUSTSEC-2025-0134 | rustls-pemfile | The crate is no longer in `Cargo.lock` at all |

Those are all four. The remaining three entries, the two quick-xml ones and
`paste`, are still reported.

The entry for the rustls-webpki three said the fix was an `oauth2` 5 migration
and to remove them the day it landed. It has not landed: `oauth2` is still
4.4.2. The dependency graph moved underneath the entry instead, through
`reqwest` and `rustls`, and nothing in the exit condition as written could see
that. **None of the four was removed by anybody deciding anything.** They
stopped applying as a side effect of an unrelated upgrade, and the exit
conditions each entry carried were about a different event from the one that
actually happened.

### How the question is asked

It cannot be asked of the plain run, which is the run the acceptances are
applied to and so reports none of them by construction. cargo-audit has no flag
meaning "read no config", so a second run happens in a scratch directory holding
a config of its own that ignores nothing, pointed at the project's `Cargo.lock`
with `-f`, and with `-n` so it shares the advisory database the first run just
fetched rather than fetching a second copy the two runs could disagree about.

Measured on 2026-09-13 against cargo-audit 0.22.2, by running from
`target/auditprobe`, two directories below the project's `.cargo/audit.toml`:
all seven entries were reported, so this version reads its config from the
working directory and does not walk upwards. An empty directory would have done.
The config is written anyway, because that measurement is about one version and
the cost of it moving is every acceptance reading as fine when none of them were
checked.

### What the check holds

Four refusals, and three of them are about the reading rather than about the
advisories.

1. **An accepted advisory the unfiltered run does not report.** The one the
   mechanism exists for. It names them and says to take them out.
2. **A second run that did not happen.** An unreadable log is not an answer.
3. **A second run that did not finish.** A file can be readable and not be a
   run: a missing lockfile, a database it could not open, a command not found.
   Every one of those looks exactly like "none of the acceptances are reported
   any more", which would empty the list in one step. The check requires the
   line cargo-audit prints once it has a lockfile open and a database loaded.
4. **A second run narrower than the first.** The unfiltered run applies no
   ignore list, so it cannot report less than the run that does. When it does,
   it is not the run it is being read as. This is the only check that can see
   the reading itself having failed, and it fails in the direction that empties
   the list.

The reader of `.cargo/audit.toml` is a tokeniser rather than a TOML parse, for
the reason `check.sh` reads `guards.toml` with a line scan: a shell gate that
needs a second language installed is a gate that refuses a commit on a machine
where that language moved. The difference from that scan is the whole reason
this one is safe. `check.sh` skips a record it cannot read, and a skipped record
is a guard that stops running. This refuses, and says which token it could not
read. Its three answers are a list of ids, no list at all because the file names
no `ignore` key, or a refusal. It will never answer "nothing is accepted"
because it could not read the file.

One companion is worth naming because it is not hypothetical. The near-miss this
reader has to survive is a comment **inside** the ignore array naming an
advisory id, which is exactly the shape the file takes the moment the check does
its job: an entry that goes has to leave a note saying which one went and why.
The four removals above are written that way on purpose, inside the array, so
the comment-stripping half of the reader is load bearing in production rather
than only in a fixture. `scripts/audit.test.sh` has the case, and the live
reader today skips four ids in that note and reads the four real entries.

### What the check cannot

Said plainly, because a check nobody has read the limits of is a check somebody
over-trusts.

- **It cannot see an acceptance whose basis has evaporated while the advisory is
  still reported.** The rsa entry rests on the advisory having no fix. If
  `patched` stops being empty, cargo audit goes on reporting the advisory, this
  check stays green, and the argument has quietly stopped being true. Ledger 358.
- **It cannot see a cargo-audit that merges configs from several directories.**
  The second run would pick up the project's ignores, report nothing, and every
  acceptance would read as dead rather than as unchecked. The narrower-run
  refusal catches the opposite direction only. Ledger 359.
- **It says nothing about unmaintained or yanked warnings that nobody accepted.**
  cargo audit does not fail on those without `-D warnings`, so three go unlisted
  and unreported today. Ledger 357.
- **It is not a judgement.** It answers whether an entry still applies, never
  whether the reasoning in it is still good. Only a person re-reading the entry
  does that, and nothing here schedules them to.

## The fingerprint question, decided

**No version fingerprint.** Three reasons, and the third is the one that settled
it.

**It would fire no earlier.** For every way an acceptance has actually gone stale
in this project, and there are four measured cases from today, the crate version
moving and the advisory ceasing to be reported are the same event. When
`rustls-webpki` went from 0.101.7 to 0.103.13 the advisory stopped being reported
in the same instant. A fingerprint would have fired at exactly the moment the
cheaper check did.

**Where it would fire alone, the answer is known in advance.** The case a
fingerprint sees and "still reported" does not is a crate moving to a different
still-vulnerable version. Every advisory in this list has an empty `patched`
list, so for all of them a patch bump changes nothing about the argument, and
the re-read would end in "nothing changed" every time. That is a per-entry number
somebody has to keep right in exchange for noise. This project has measured what
hand-maintained per-record fingerprints cost: `guards/guards.toml` carries 745 of
them and their remedy runs for hours.

**It would not close the gap it looks like it closes.** The real hole is an
advisory gaining a fix while the crate stays where it is, and that is not
version-shaped. A fingerprint on the crate version cannot see it. Closing it
means recording the `Solution:` line each acceptance was judged against, which
is a different mechanism against a different field. That gap is in the ledger
rather than papered over with a fingerprint that does not reach it.

The alternative was to build the `Solution:` fingerprint now. It was not built
because it applies only to advisories cargo-audit classes as vulnerabilities,
not to the informational ones, so it would cover some entries and not others,
and deciding which half is covered is the kind of thing that should be written
down once with its reasoning rather than bolted on at the end of another change.

## What cargo audit does now, in each mode

Measured on 2026-09-13 after the change, on this machine, not piped.

**`cargo audit` in the project root.** This is what CI's Security Audit job
runs and what the gate's first run runs. It reads `.cargo/audit.toml`.

```
Crate:     lzw            Warning: unmaintained   ID: RUSTSEC-2020-0144
Crate:     proc-macro-error  Warning: unmaintained   ID: RUSTSEC-2024-0370
Crate:     chacha20       Warning: yanked

warning: 3 allowed warnings found
```

Exit status 0.

So: **CI's Security Audit job, which has been red since 2026-09-10, goes green
with this change.** That is the intended consequence of deciding the advisory
and it should be stated rather than discovered. Four advisory ids are silenced,
the four in the list. Three warnings still print and none of them fails the job,
because cargo audit does not treat an unmaintained or yanked crate as a failure
without `-D warnings`. Nothing fails.

**The second run, from a scratch directory, with no ignore list.** Reports
RUSTSEC-2026-0194, RUSTSEC-2026-0195, RUSTSEC-2023-0071, RUSTSEC-2020-0144,
RUSTSEC-2024-0436, RUSTSEC-2024-0370, and the yanked chacha20. Exit status
non-zero. Its status is deliberately not read; only its text is.

**`scripts/audit.sh`.** Runs both, then the two decisions:

```
== the advisories this project accepts ==
All 4 advisory(ies) this project accepts are still reported.
== everything else ==
No advisory outside .cargo/audit.toml, and nothing is being held open.
```

Exit status 0. It costs 11.5 seconds warm, against about 6 for the single run it
replaced, in a gate of about 330.

The three unlisted warnings are the honest gap in all of this. They are printed
on every run and nobody has decided any of them, which is the same shape as the
advisory this change just decided, one level down. Ledger 357.

## What in the brief turned out wrong

**"`.cargo/audit.toml` now holds six accepted advisories."** It held seven:
two quick-xml, three rustls-webpki, and two unmaintained. The count mattered
only in that the check has to read the list rather than be told its size, which
it does.

**The brief treated all of them as live acceptances.** Four were already dead.
That is not a fault in the brief, which could not have known, and it is the
single most useful thing the change found: the mechanism the brief asked for
paid for itself on its first run.

**"`scripts/audit.sh` already does exactly this for the held-open list."** True,
and worth qualifying. It compares against the plain run, which works for a
held-open advisory precisely because that one is not in `audit.toml` and so is
not silenced. The same code pointed at the accepted list would have reported all
seven as dead on every run. The second run is not an implementation detail of
extending the idea; it is the whole difficulty.

## What was not done, and why

**No guard record was added to `guards/guards.toml`.** A record applies a
prescribed break to a source file and requires the named tests to redden, read
from cargo's output. The new check is a shell suite, which is not a shape that
registry supports: `scripts/guards.py` edits a file and runs `--lib`. The census
at lines 79 and 80 was verified as summing to 745 records, parsed with a TOML
reader rather than grepped, and was not touched.

**No dependency version changed.** `Cargo.lock` is untouched. `pgp` and `rsa`
are exactly where they were.

**No changelog entry and no version bump.** Not user visible.

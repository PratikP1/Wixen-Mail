---
phase: 04-writing-and-reading-a-message-in-full
plan: 09
subsystem: OpenPGP and S/MIME encrypted mail
tags: [security, secrets, dependencies, credential-store, openpgp]
status: paused-at-checkpoint
requires:
  - "04-03: what_the_form_says and ReaderDocument::with_encryption, the PGP armour sentence"
provides:
  - "service::pgp::KEYRING_SERVICE and KEYRING_PRIVATE_KEY, the permanent credential store name"
  - "service::pgp::keyring_entries, the owner answering for its own entries"
  - "service::pgp::WhatOpeningItFound and WhatImportingAKeyFound, the outcomes named apart"
  - "application::forget: the completeness guard over the list uninstalling erases"
  - "docs/development/pgp-implementation-choice.md, the crate comparison and the audit"
affects:
  - "uninstalling, which now erases the OpenPGP private key entry"
  - "any future module under src/service/ that owns credential store entries"
tech-stack:
  added: []
  patterns:
    - "an owner answers for its own credential store entries rather than being listed elsewhere"
    - "a source-reading completeness guard paired with a test that asks for the real value"
key-files:
  created:
    - src/service/pgp/mod.rs
    - docs/development/pgp-implementation-choice.md
  modified:
    - src/service/mod.rs
    - src/application/forget.rs
    - guards/guards.toml
decisions:
  - "pgp 0.20, the rPGP project, over sequoia-openpgp 2.4.1. The licence decides it: MIT OR Apache-2.0 against LGPL-2.0-or-later on a statically linked MIT binary."
  - "The credential store name is wixen-mail-pgp with the account name private-key, permanent from the commit that wrote it."
  - "No crate was added to Cargo.toml. The legitimacy check is answered before the dependency exists."
  - "Task 1 was not done. It rests on a premise that is false, and the way to make it true is a decision Pratik owns."
metrics:
  duration: one session
  completed: 2026-09-06
actuals:
  tokens: 21000
  tasks: 1
  commits: 2
---

# Phase 4 Plan 9: Encrypted Mail Summary

Task 2 shipped. Task 1 did not, and that is the finding rather than the
shortfall. Task 3 was never in scope for this run.

**Nothing decrypts. No crate is in `Cargo.toml`.** Opening an encrypted message
does exactly what it did yesterday. What exists now is the name a private key
will live under, registered where uninstalling reads its list, the guard that
would catch the next owner nobody registers, and a written, measured choice of
which OpenPGP implementation to use, with the audit a person has to answer
before the dependency is added.

## Commits

| Commit | What |
|---|---|
| `29daf7d` | RED: the private key entry nobody erases, and the completeness guard |
| `21e835f` | GREEN: the name, the registration, the guard record, the comparison document |

Both on `phase-04-09-encrypted-mail`, branched from `50ed052`. Not merged.
`scripts/check.sh all` is owed before a merge; the branch has only had the
scoped gate.

## Task 1 was not done, and why

**The plan's central instruction for task 1 cannot be followed, and the reason
is one line in a file the plan did not send anybody to read.**

Task 1 says: "Ask it on the message-open path, from the same raw bytes the
signature check already has. Take the envelope's bytes out of the message and
hand them to `EncryptedMessage::read`."

There are no raw bytes. `MessageCache::keep_signed_original` is the only thing
that stores a message as it arrived, and its first statement is:

```rust
if !claims_a_signature(raw) {
    return Ok(());
}
```

`claims_a_signature` returns false for an encrypted message, deliberately, and
`src/service/signed_mail.rs` carries a test asserting exactly that. So an
S/MIME enveloped message is never stored in its arrived-in form, and on the open
path `checking_signatures::for_message` gets `SignedOriginal::NotSigned`. The
signature check does not have the bytes. Nothing does.

**What the reader does have was checked too, and none of it answers the
question.** `CachedMessage` and `MessageItem` carry no content type. The stored
attachment row does carry a mime type, but `mime::content_type_of` drops the
parameters, so an enveloped message's attachment is recorded as
`application/x-pkcs7-mime` with no `smime-type`, which `layout_of` cannot
resolve. The filename `smime.p7m` could resolve it, but reading the answer out
of two stored columns rebuilds `layout_of`'s logic in a second place, which is
the exact thing `claims_a_signature`'s own comment forbids.

**`application::body_safety` already found this and wrote it down.** Its module
doc says of the S/MIME encrypted case: "It cannot fire through this path. That
case is answered from the message's `Content-Type`, which is not here." The gap
was known. What was not known is that there is nowhere on the open path where
the content type still exists.

### Why this is a decision rather than a fix

The obvious repair is to widen `keep_signed_original` to keep an encrypted
message too. That breaks a documented invariant, and the invariant is
load-bearing:

> Nothing is kept for a message that never said it was signed, which is nearly
> all mail. A row exists here exactly when the message claimed a signature, and
> that is what tells a reader apart from a message that never claimed one.

`SignedOriginal` has three states and the distinction between `NotSigned` and
`NotKept` rests on that invariant. Widen the write and an encrypted message
becomes `Kept`, flows into `examine_signed_message`, and produces "it says it is
signed, but it carries no signature to check" about a message that never said
anything of the kind. That is the precise failure `claims_a_signature`'s comment
exists to prevent, arriving through a different door.

Three ways out, all of them real work with different costs:

| Option | What it costs |
|---|---|
| Widen `keep_signed_original` and give `SignedOriginal` a fourth state | Changes the meaning of a shipped store. Every consumer of the three-state answer has to be re-read. `checking_signatures::from_what_was_kept` has to re-ask a question the invariant used to answer. |
| An additive column on `messages` recording what the content type said | The documented pattern here, and `MessageItem::receipt_to` is the precedent. 04-05 measured the shape: adding `safety` to `CachedMessage` cost 36 construction sites. It answers "is it encrypted" and cannot answer "how is it addressed", so criterion 5's second clause half-ships. |
| Take the envelope from the cached attachment content | The bytes are there, decoded, under `attachment_content`. Needs the flag from option 2 as well, and needs an index guess. Two stores cooperating. |

None of these is a bug fix, and picking one changes what gets built. Under this
project's own rule, stopping early is for "a decision that is genuinely
Pratik's and changes what gets built", and this is that.

**What was not done as a consequence:** `claims_encryption` was not written. A
function with no honest caller, added so a task could be reported as started,
is the stub guardrail 3 is about. The recognition half has no caller until the
storage question is answered, because the one place raw bytes exist for every
arriving message is the storage decision itself.

## Task 2: the name, the guard, and the choice

### The credential store name

`wixen-mail-pgp`, account name `private-key`, in `src/service/pgp/mod.rs`. Its
doc comment says the name is permanent and says what changing it costs: a
private key orphaned on every machine that has one, because the code that erases
secrets names entries by this string.

It follows the four owners already here. `security::KEYRING_SERVICE` is
`wixen-mail`, `credentials::KEYRING_SERVICE` is `wixen-mail-account`,
`oauth::keyring_service` builds `wixen-mail-{provider}` and
`caldav::keyring_service` builds `wixen-mail-caldav-{id}`.

The entries are answered by `pgp::keyring_entries()` rather than written out in
the uninstaller. That is `oauth::entries_for_account`'s shape and it is chosen
for `oauth`'s recorded reason: two lists of the same entries came apart once and
a removed account's refresh token outlived the program. It also means several
keys later become several user names under one service and only this module
changes.

The lines added to `application::forget::entries_for`:

```rust
// Asked rather than listed, the same as the OAuth tokens below and for the
// same reason. It also belongs to the machine rather than to an account: a
// private key is imported once and opens mail in any mailbox, so it is
// named here beside the master key and not inside the loop.
for (service, user) in pgp::keyring_entries() {
    entries.push(CredentialEntry { service, user });
}
```

### The outcome types

```rust
pub enum WhatOpeningItFound {
    Opened(String),
    NoKeyHere,
    TheKeyHereDoesNotOpenIt,
    Damaged,
}

pub enum WhatImportingAKeyFound {
    Imported,
    NotAPrivateKey,
    NotAKey,
    CouldNotBeStored { reason: String },
}
```

`Damaged` carries nothing on purpose. A crate's error text is written for
somebody reading a stack trace and it can quote the bytes it choked on, which
are a stranger's mail. `CouldNotBeStored` is the one variant with words and they
are the credential store's reason, never anything from the key file.

**The function signatures task 3 will implement were not written**, which is a
departure from the plan's letter. The plan asks for "the functions task 3 will
implement". A function body that is `todo!()` or returns a placeholder is a stub
presented as complete, and guardrail 3 is about exactly that. The types are the
boundary; task 3 adds the functions with their implementations in one commit.
The plan's stated worry about this, that under `-D warnings` an interface
nothing calls will not build, does not apply: these are `pub` items in a library
crate reachable from the crate root, so they are not dead code. Verified by
running clippy with `-D warnings`, which passed.

### The completeness guard

`test_every_owner_of_credential_entries_is_named_where_uninstalling_reads`, in
`src/application/forget.rs`. It reads every file under `src/service/`, takes the
half that ships through `common::what_ships`, and requires every module that
owns credential store entries to be named in `entries_for`'s body.

How it enumerates, and why not the obvious way. The naive version, a grep for
`KEYRING_SERVICE`, finds two of the four owners and misses `caldav`, whose name
is a function, and `oauth`, which answers for its own. A third version was
measured and refused: every credential store service name in this project starts
with `wixen-mail`, so searching for that literal looks like it would find all of
them, and it does, along with six files that have nothing to do with the
credential store, including the executable file name in `default_apps`, a named
pipe in `handover` and a client id in `safebrowsing`.

What is used instead is `service::secret_store`, which its own doc calls "the one
way in and out of the operating system's credential store". That clause alone
finds `caldav`, `credentials`, `oauth` and `security`, which is exactly the set
`entries_for` names. A second clause catches a module that has named its entries
and not wired the store up yet, which is where `service::pgp` sits today and is
the window in which an owner is most likely to be forgotten.

`test_this_reading_finds_every_owner_that_exists_today` is the companion this
project asks a source-reading guard to carry. It names all five owners the
reading must find, so a reading that had quietly stopped matching cannot report
a clean result.

### The guard record, and the second break that changed it

Measured by hand on 2026-09-06 against a green tree of 6,434 tests, with
`cargo test --no-fail-fast --lib`. Two candidate breaks were run.

**Break one, deleting the loop from `entries_for`.** Three tests red in 6,434:

```
application::forget::tests::test_every_owner_of_credential_entries_is_named_where_uninstalling_reads
application::forget::tests::test_the_master_key_is_always_forgotten
application::forget::tests::test_the_private_key_entry_is_one_uninstalling_erases
```

**Break two, keeping the loop and neutering it** with
`pgp::keyring_entries().into_iter().take(0)`. Two tests red, and the
completeness guard is **not** among them:

```
application::forget::tests::test_the_master_key_is_always_forgotten
application::forget::tests::test_the_private_key_entry_is_one_uninstalling_erases
```

The source still says `pgp`, so the reading still finds the owner. That is the
known limit of a guard that reads source, and it was found by running a second
candidate rather than trusting the first. What covers the gap is
`test_the_private_key_entry_is_one_uninstalling_erases`, which asks for the entry
itself and went red under both breaks. The pair holds; neither half does alone.
Both the limit and the pairing are written into the test's comment and into the
record, so the next owner somebody adds knows it wants a test of that second
shape as well as a mention.

The record uses break one, since that is what forgetting to register an owner
actually looks like.

### A census went red on arrival and was corrected rather than loosened

`test_the_master_key_is_always_forgotten` asserts `entries.len() == 1` for an
installation with no accounts. Adding a machine-level entry made that two, so it
failed in the GREEN commit. It was corrected to assert both entries and kept as
an equality.

Keeping it exact is the point. `CLAUDE.md` records what a census costs once it
becomes a floor: with a spare above the floor, removing one member no longer
trips the guard that counts them. An equality has no spare. It also earned its
keep immediately, going red under both hand breaks above without being about the
private key at all.

**This is the shape 04-08 warned about, arriving in a different place.** The plan
anticipated a green test going red in task 3, from 04-03's stated truth. It did
not anticipate one going red in task 2, and it went red inside the same commit
pair rather than across tasks.

### The crate comparison

`docs/development/pgp-implementation-choice.md`. `pgp` 0.20, the rPGP project,
over `sequoia-openpgp` 2.4.1.

**The criteria were widened before anything was evaluated, and the document says
so.** The plan lists four: transitive cost, C toolchain, Windows build, release
history. It never mentions the licence, and the licence is what decides this.
Wixen Mail is MIT and ships a statically linked Windows installer.
`sequoia-openpgp` is LGPL-2.0-or-later, which puts a relinking obligation on
that binary that MIT does not, and that is a decision about how the product may
be distributed rather than a decision about a library. Without the licence
criterion the comparison reaches the same answer for incomplete reasons, and
nobody could tell later which reasons were load-bearing.

Three things in it were measured rather than read, and two of them change what
the comparison says:

**Sequoia's transitive cost looked five times better and is not.** 16 new crates
against rPGP's 78, in its default configuration. Two of those 16 are `nettle`
and `nettle-sys`, which need the Nettle C library, GMP and `pkg-config` at build
time. Measuring sequoia with its pure-Rust backend instead gives **70** new
crates, within a few of rPGP's 78. The cheap number belongs to a configuration
this project cannot ship, and the shippable configurations cost the same. That
backend also has to be turned on with `allow-experimental-crypto` and
`allow-variable-time-crypto`, which is the maintainers saying what they think of
it.

**rPGP's own security notes say the `rsa` crate it depends on is vulnerable to
the Marvin timing attack**, unfixed, tracked at RustCrypto/RSA issue 19. `rsa`
is one of the 78. That is in the document as a cost rather than a footnote.

**rPGP's platform list marks `x86_64-pc-windows-msvc` as not in its CI.** Only
`x86_64-pc-windows-gnu` is checked. This project builds MSVC. Stated rather than
smoothed over.

**One thing must change in `Cargo.toml` besides the dependency line.** This
project declares `rust-version = "1.87"`. rPGP 0.20 declares 1.88, so the floor
moves in the same commit. The toolchain in use is 1.97.1, so nothing fails to
build.

**`Cargo.lock` was re-checked rather than quoted**, as premise 12 asked:
`grep -inE '^name = "(pgp|rpgp|sequoia|sequoia-openpgp|openpgp|nettle|gpgme)"'`
returns nothing on 2026-09-06.

**How the numbers were measured.** `pgp = "0.20"` and
`sequoia-openpgp = "2.4"` were resolved in throwaway crates outside this
repository with `cargo tree --edges normal` and `cargo metadata`. Nothing was
compiled and no build script ran. "New to this tree" is the resolved set minus
every crate name in `Cargo.lock`. Download counts are from the `crates.io` API
on 2026-09-06.

## Package legitimacy audit

The full table is in
`docs/development/pgp-implementation-choice.md`. 78 crates, every one
`[ASSUMED]` because none is in this tree today, and none `[SLOP]`: every crate
resolves to a real repository, every download history is consistent with its
age, and no name is a near-miss for a more popular crate.

By owner: 45 RustCrypto, 4 dalek-cryptography, and the rest widely used utility
crates. Every licence is permissive, 73 under some combination of MIT and
Apache-2.0, 4 BSD-3-Clause, one under the bzip2 licence. None copyleft.

The four worth a page visit first, being the only ones under ten million
downloads or outside a known organisation: `cx448` at 1.2M, published by the
rPGP maintainer himself so it inherits rPGP's trust rather than adding evidence;
`bitfields` and `bitfields-impl` at 2.3M, a derive macro that is not
cryptographic and not part of an organisation; and `ocb3` at 4.5M, RustCrypto,
low because OCB3 is rare rather than because the crate is obscure.

## The plan's premises

**Six wrong. One of them stopped task 1.** All six are measured, not inferred.

**1. Task 1's data path does not exist.** The whole of the section above. The
signature check does not have an encrypted message's raw bytes, because
`keep_signed_original` refuses to store them.

**2. Premise 13's guard-record table is wrong in two rows, and one of them is
the one task 1 was costed on.** Measured by parsing `tests_last_seen`, as the
plan itself demands:

| file | plan says | measured 2026-09-06 |
|---|---|---|
| `src/service/signed_mail.rs` | 1 record, 103 tests | 1 record, 103 tests |
| `src/presentation/reader_text.rs` | **0 records**, 76 tests | **7 records**, 110 tests |
| `src/presentation/wx_app.rs` | 39 records, 199 tests | 40 records, 195 tests |
| `src/application/forget.rs` | not measured | **0 records**, 18 tests before this plan, 21 after |

The plan says "the reader tests go in `reader_text.rs`, which costs nothing".
It costs seven records. 04-07 added them and the table was written after. This
is now three plans in a row where a table written from an earlier tree went
stale on `reader_text.rs` or `wx_app.rs`.

Total records at `50ed052`: 626, not the 624 the brief quoted. 627 after this
plan.

**3. Premise 14 is wrong about the fixtures.** It says
"`EncryptedMessage::read` is tested against constructed DER only". The fixture
it is tested against, `ENCRYPTED_TO_ALICE`, is real OpenSSL output, sitting in
the same block as the signed fixtures under a comment saying so: "Real output
from OpenSSL rather than anything hand-built here." That does not make it a real
envelope from Outlook or Thunderbird, which is the gap that matters, but the two
claims are different and only one of them is true.

**4. Premise 7 is wrong that `EncryptedMessage::spoken` is careful, and 04-03
already said so.** The premise says the sentence "is already careful about this
and should not be reworded". `spoken` says "Wixen Mail cannot open encrypted
mail yet", which is a claim about the program. 04-03's own doc comment over
`ENCRYPTED_AND_NOT_OPENED_HERE` says of that sentence: it "goes on to say Wixen
Mail cannot open encrypted mail *at all*, which is a claim about the program
rather than about the message in front of somebody, and it is a claim that stops
being true the moment anything here learns to open one kind". 04-03 wrote a
narrower sentence for that reason and put a test on the wording. Task 1's own
changelog instruction gives the same argument. The premise contradicts a file
the plan told the executor to read. `spoken` is unreached today, so the
overclaim harms nobody; task 1 is the commit that would give it a caller, and
narrowing it belongs there.

**5. The plan contradicts itself about `Cargo.toml`.** Task 2's action says
"`Cargo.toml` gains the dependency and its justification". The checkpoint's
acceptance criteria say "No crate was added to `Cargo.toml` before this
checkpoint was answered". Both cannot hold. The checkpoint wins: it is the
stricter rule and it is the whole reason the checkpoint is a decision rather
than an end-of-phase verification. `Cargo.toml` is untouched, and the comment
that would sit over the dependency goes in with it.

**6. The pre-release objection is in the wrong comment.** Premise notes and the
plan's task-2 action both attribute it to `x509-parser`: "`Cargo.toml`'s own
`x509-parser` comment refuses a crate for having only ever published
pre-releases". It is in the **`ring`** comment, and the crate refused is
**`cms`**. The reasoning is real and does apply here. The citation is wrong.

**What the plan got right.** Premise 4 is right in every part: `layout_of`
already returns `Encrypted` for all three recognition cases, folds case, and
falls back to the file suffix. Premise 8 is right: `entries_for` builds its list
by hand from four owners, none of its eighteen tests asks whether the list is
complete, and its comment records the two lists coming apart. Premise 12's
`Cargo.lock` check is right. Premise 6 about `CertificateStore` is right.

## Deviations from plan

**One, and it is the reason for the stop.** Task 1 was not attempted past the
measurement. See above.

**Two smaller ones inside task 2.** The function signatures were not written,
for guardrail 3's reason. `Cargo.toml` was not touched, for the checkpoint's
reason.

**No changelog entry and no version bump.** Nothing a user can observe changed:
the uninstaller now erases an entry that nothing writes. Task 3 is the commit
that carries both.

**Auto-fixed:** one. `test_the_master_key_is_always_forgotten` went red on
arrival in the GREEN commit and was corrected in the same commit. It is a census
that was doing its job.

**Bookkeeping:** `guards/guards.toml`'s sweep header count went from 434 to 435
records arrived since the 2026-08-12 sweep, which
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
catches at commit time and did.

## Guard record accounting

`src/application/forget.rs` was named by **0** records before this plan, so
adding three tests to it triggered no re-measurement. `src/service/pgp/` is new
and named by none.
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` passed on
both commits and printed no remedy, so nothing is owed.

`scripts/guards.sh --touched-by 50ed052` is owed after a merge and must not
block it. This branch changes `forget.rs`, which one record now names, its own.

## Known stubs

None in the sense the guardrail means. `src/service/pgp/mod.rs` holds a name, a
list of entries and two outcome types, and nothing pretends to decrypt. The
constant is reached from `application::forget::entries_for` on a non-test path,
so the one thing this plan wired is wired.

What is deliberately incomplete and said out loud: nothing opens a PGP message,
nothing imports a key, and an S/MIME encrypted message still opens as a blank
message with no explanation, exactly as it did before this branch.

## What did not close

**READ-02 does not close.** Its second `[D]` line, outgoing signing and
encryption, is untouched and was never in this plan. Its first `[D]` line needs
task 3.

**Criterion 5 does not close, in either clause.** The second clause, the S/MIME
one, is blocked on the decision above. The first clause needs task 3.

## Self-Check: PASSED

Files claimed as created, verified present:

- `src/service/pgp/mod.rs` FOUND
- `docs/development/pgp-implementation-choice.md` FOUND

Commits claimed, verified in `git log`:

- `29daf7d` FOUND
- `21e835f` FOUND

Tests claimed, verified by
`cargo test --no-fail-fast --lib -- application::forget:: service::pgp::`:
25 passed, 0 failed.

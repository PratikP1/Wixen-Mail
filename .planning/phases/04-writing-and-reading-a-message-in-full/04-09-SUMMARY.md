---
phase: 04-writing-and-reading-a-message-in-full
plan: 09
subsystem: OpenPGP and S/MIME encrypted mail
tags: [security, secrets, dependencies, credential-store, openpgp, smime]
status: complete
requires:
  - "04-03: what_the_form_says and ReaderDocument::with_encryption, the PGP armour sentence"
provides:
  - "service::signed_mail::claims_encryption and is_an_smime_envelope"
  - "data::message_cache::how_it_arrived: the mark saying a message arrived encrypted, and the envelope read back out of the file it arrived as"
  - "application::encrypted_mail: what can be said about an S/MIME envelope on the open path"
  - "service::pgp::open_a_message and import_a_private_key, behind the crate boundary"
  - "application::opening_pgp: the one place that offers a message's armour to the key"
  - "presentation::reader_text::with_smime_envelope and with_pgp"
  - "application::allowed::READING_PGP_MAIL_IS_EXPERIMENTAL"
affects:
  - "every arrival path, which now records two facts about a message's form rather than one"
  - "the reader window and the read-aloud path, which both say why an encrypted message cannot be shown"
  - "Cargo.toml: the first OpenPGP dependency, and rust-version moved to 1.88"
tech-stack:
  added:
    - "pgp 0.20, the rPGP project, behind src/service/pgp/"
  patterns:
    - "a fact about a message's arrival is recorded where the raw bytes exist and read back by row id, not carried on CachedMessage"
    - "a cross-implementation fixture: GnuPG makes the key and the message, the crate under test opens them"
    - "one sentence in the bar and in the body, rather than two that can drift"
key-files:
  created:
    - src/data/message_cache/how_it_arrived.rs
    - src/application/encrypted_mail.rs
    - src/application/opening_pgp.rs
    - src/service/pgp/keys.rs
  modified:
    - src/service/signed_mail.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_app.rs
    - src/application/forget.rs
    - src/application/allowed.rs
    - src/service/outward.rs
    - src/data/message_cache/mod.rs
    - guards/guards.toml
    - tests/wired.rs
    - Cargo.toml
    - docs/changelog.md
decisions:
  - "The mark and the envelope together, which is Pratik's answer to the premise that stopped the first run. A column on messages says a message arrived encrypted; the envelope comes back out of the attachment it arrived as."
  - "keep_signed_original is not widened. Its invariant is that a row exists exactly when a message claimed a signature, and widening it sends encrypted mail into examine_signed_message."
  - "The mark is read by row id, not carried on CachedMessage, so none of the 51 construction sites changed."
  - "WhatOpeningItFound gained a fifth variant and WhatImportingAKeyFound a fifth, both found by writing the implementation."
  - "pgp 0.20 added, with the Marvin weakness and the untested MSVC target written into Cargo.toml and the changelog."
  - "rust-version moved 1.87 to 1.88, which turned on a clippy lint family across seven pre-existing sites."
metrics:
  duration: one session
  completed: 2026-09-06
actuals:
  tokens: 108000
  tasks: 3
  commits: 6
---

# Phase 4 Plan 9: Encrypted Mail Summary

**Both tasks work, against fixtures. Neither has met a real correspondent.**

An S/MIME encrypted message used to open as a blank pane under the line "This
message has no text, or it has not been downloaded yet". Both halves of that
were false. It now says it is encrypted, how it is addressed, and that Wixen
Mail cannot open it.

A PGP encrypted message used to open as a screenful of armour. With a key
imported from File, Import PGP Private Key, it now opens as the message. When it
does not open, it says which of four things happened.

What that is worth, said plainly. The PGP path was tested against a key pair and
a message made by **GnuPG 2.4.9**, not by the crate under test, so the passing
test is two implementations agreeing rather than one agreeing with itself. That
is the strongest evidence available in this repository and it is not the same as
working: no message from a correspondent has ever been through either path.

## Commits

| Commit | What |
|---|---|
| `29daf7d` | RED: the private key entry nobody erases, and the completeness guard (previous run) |
| `21e835f` | GREEN: the name, the registration, the guard record, the comparison document (previous run) |
| `d226d3d` | RED: failing tests for a message that says why it cannot be read |
| `3fde14f` | GREEN: an encrypted message says why there is nothing to read |
| `28e26b3` | RED: failing tests for reading a PGP message somebody holds the key for |
| `ad6ec18` | GREEN: a PGP message somebody holds the key for opens and reads |

All on `phase-04-09-encrypted-mail`, branched from `50ed052`. `ad6ec18` earned
the full gate, all four checks, including the release build.

## Task 1: an encrypted message says why it cannot be read

### The premise that stopped the first run, and what replaced it

The first run's finding was right and is worth restating, because the fix rests
on it. `MessageCache::keep_signed_original` is the only thing that stores a
message as it arrived, and its first statement refuses anything that does not
claim a signature. `claims_a_signature` answers no for encrypted mail
deliberately: its own comment says answering yes sends an enveloped message down
the signature path, where every surface downstream says "it says it is signed,
but it carries no signature to check" about a message that never said anything
of the kind. So on the open path there are no raw bytes, and the content type is
gone.

**Pratik's answer was a stored mark and the envelope read back out of the
attachment, and that is what shipped.** `src/data/message_cache/how_it_arrived.rs`
holds both halves. Neither is sufficient alone, and the module doc says so:
without the mark there is no telling which attachment is an envelope, because an
encrypted message and a message with a file attached look identical in the stored
rows; without the attachment the mark says only that a message is encrypted and
cannot say who it was encrypted to.

**`keep_signed_original` is unchanged and its invariant is not widened.**

### The cost the brief told me to budget for did not arrive

The brief said 04-05 measured the same shape at 36 `CachedMessage` construction
sites and told me to re-measure rather than take that on trust. Measured
2026-09-06: `grep -rn "CachedMessage {" src/` finds **51**.

**None of them changed.** The mark is an additive column read back by row id,
which is `signed_original`'s own shape rather than `safety`'s: `arrived_encrypted`
and `the_envelope_it_carried` take a message id, exactly as `signed_original`
does, so nothing had to be threaded through a struct. The 51 sites are what the
other shape would have cost.

### Four arrival paths, not three, and the guard that found only three

`note_the_form_it_arrived_in` asks both questions itself, for the reason
`keep_signed_original` already gives about asking one of them: a question
answered in four places is four chances to disagree.

`test_every_arrival_path_records_both_facts_about_the_form_a_message_came_in`
is the guard over that. **Its first version read only `src/application/`, found
the three arrival paths that live there, and passed while
`presentation::wx_app`'s body fetch went on calling the old function.** What
caught it was `tests/wired.rs`, which already named that function. The reading is
now the whole tree minus the module that owns the pair, and its companion names
all four.

It also flagged a file that does not call anything: `application::message_files`
carries a doc link to `MessageCache::keep_signed_original` explaining why it asks
the same question. The reading now matches `.keep_signed_original(` rather than
the bare name, and the comment says what that makes it blind to.

### What was green on arrival, and why that is the measurement

Six tests passed against the insufficient implementations, and each one is a
statement that the wiring is behaviour-neutral:

- `test_a_signed_message_does_not_claim_encryption`,
  `test_ordinary_mail_claims_no_encryption`,
  `test_no_message_claims_both_a_signature_and_encryption` and
  `test_a_signature_file_is_never_taken_for_the_envelope` pass against a
  `claims_encryption` that answers no to everything.
- `test_ordinary_mail_and_signed_mail_are_not_recorded_as_encrypted` and
  `test_a_message_nothing_was_recorded_about_reads_as_not_encrypted` pass
  because the column defaults to the right answer for ordinary mail.
- `test_an_ordinary_message_is_left_exactly_as_it_was` passes against a
  `with_smime_envelope` that does nothing.

The plan asked which assertions were green because `layout_of` already answered
them. **None of the recognition tests were**, and that is worth being exact
about: `layout_of` does already return `Encrypted` for all three cases, but the
tests assert `claims_encryption`, which did not exist. A test asserting
`layout_of` directly would have been green and said nothing.

### The sentence, quoted whole

The body and the bar carry the same string. For a message addressed to one
certificate on a computer whose store cannot be asked, what somebody hears is:

> This message is encrypted. It is addressed to 1 certificate. Wixen Mail cannot
> open it, so nothing of it can be read here.

Where the store answers yes, the middle sentence is "This computer holds a
certificate this message was encrypted to." Where it answers no, "This message
was not encrypted to any certificate on this computer."

Where the envelope will not parse, or its file is not on this computer:

> This message is encrypted. Wixen Mail could not read who it was encrypted to,
> and it cannot open it, so nothing of it can be read here.

**One string in two places, not two strings.** The bar is what the reader speaks
as a message opens, so without it the fact arrives only after the whole header
block. The body is where the falsehood was. Two differently worded sentences
were the other option and were refused: a listener meeting two near-identical
sentences has to work out whether the second added anything, and meeting the same
sentence twice they recognise a fact restated. It is also one place to change the
wording.

### The three states of the certificate answer

`Some(true)`, `Some(false)` and `None`, with `None` for a store that could not be
asked, and a test drives an unaskable store and asserts the sentence claims
neither way. The reason is in the code beside it: `Some(false)` tells somebody a
private message was not meant for them, on no evidence, and it is one line away.

### `spoken` was narrowed, and the plan said not to

Premise correction 7 said the sentence "is already careful about this and should
not be reworded". It was wrong, and 04-03 had already said so: its doc comment
over `ENCRYPTED_AND_NOT_OPENED_HERE` records that the S/MIME sentence "goes on to
say Wixen Mail cannot open encrypted mail *at all*, which is a claim about the
program rather than about the message in front of somebody, and it is a claim
that stops being true the moment anything here learns to open one kind".

It stopped being true in this plan. The sentence now ends "Wixen Mail cannot open
it", and `test_what_is_said_is_about_this_message_and_not_about_the_program`
holds it there. The Windows certificate store's own refusal was narrowed the same
way, from "cannot open encrypted mail yet" to "cannot open an S/MIME encrypted
message yet".

### The fixtures are real OpenSSL output, not constructed DER

Premise 14 said `EncryptedMessage::read` "is tested against constructed DER
only". The first run corrected that and it stays corrected: `ENCRYPTED_TO_ALICE`
is real OpenSSL output, in a block whose comment says so. **That is not a real
envelope from Outlook or Thunderbird, which is the gap that matters**, and it is
in the ledger.

The refusal path is what bounds it and it is tested against every seventh-byte
prefix of the real fixture, so a truncation at any point says the message is
encrypted and its details could not be read rather than falling through to an
empty body.

## Task 3: a PGP message somebody holds the key for

### The fixtures, which are the reason the passing test means anything

The plan named the trap: a key and a message generated by the crate and read
back by it prove the crate agrees with itself. It said to generate the message
fixture from outside the crate "if there is any way to", and expected there would
not be.

There was. GnuPG 2.4.9 is on this machine. Three commands produced two RSA-2048
key pairs with no passphrase and a message encrypted to one of them, and the
message was checked to open under `gpg --decrypt` before it was written down.
`test_a_message_encrypted_to_the_imported_key_opens_and_reads` is therefore rPGP
opening GnuPG's output with GnuPG's key, and
`test_a_key_that_does_not_open_it_is_not_reported_as_no_key` offers the same
message to the second key.

**It is still not a message from a correspondent.** Ledger entry recorded.

### The crate boundary, and the greps that hold it

```
$ grep -rn "use pgp::" src/ --include=*.rs
src/service/pgp/keys.rs:38:use pgp::composed::{Deserializable, Message, SignedPublicKey, SignedSecretKey};
src/service/pgp/keys.rs:39:use pgp::errors::Error as OpenPgpError;
src/service/pgp/keys.rs:40:use pgp::types::Password;
src/service/pgp/mod.rs:274:        // `use pgp::` rather than the bare word, because ...
src/service/pgp/mod.rs:301:                std::fs::read_to_string(path).is_ok_and(|source| source.contains("use pgp::"))
src/service/pgp/mod.rs:321:            adapter.contains("use pgp::"),
```

The three real hits are in the adapter; the three in `mod.rs` are the guard's own
string literals. `test_no_caller_outside_this_module_names_the_crate` reads the
tree on every run, with
`test_this_reading_finds_the_one_file_that_does_name_the_crate` as the companion
so a reading that stopped matching cannot report clean.

`use pgp::` rather than the bare word, because `service::pgp` is what the module
is called: a search for `pgp` matches every file that opens a message.

### Nothing leaks

```
$ grep -rn "unwrap()\|expect(" src/service/pgp/
src/service/pgp/keys.rs:321,322,486,516   (all inside #[cfg(test)])
src/service/pgp/mod.rs:318                 (inside #[cfg(test)])

$ grep -rniE "pgp" src/data/
(nothing)
```

No production `unwrap` or `expect` anywhere on the PGP path, nothing PGP-shaped
in the message cache, and **no error out of the crate is logged or passed on**.
That last one is deliberate and the code says why: a parser refusing a message
can quote the bytes it choked on, and those bytes are a stranger's mail.
`test_a_refusal_never_carries_anything_out_of_the_file` checks a refusal against
the key file's own lines rather than against a shape.

### Two variants the plan did not have, found by writing the implementation

`WhatOpeningItFound::TheKeyHereCouldNotBeRead`. Import stores only what has
already parsed, so a stored key that will not parse means the credential store
handed back something else. `NoKeyHere` would be a lie about the one thing
somebody has already done and `Damaged` would blame the sender.

`WhatImportingAKeyFound::TheKeyIsLockedWithAPassphrase`. Nothing here can ask for
a passphrase, so a locked key would be stored, open nothing, and report the wrong
reason for ever after. It is refused at import, which is the only moment somebody
can act on it. The check reads the primary key **and every secret subkey**,
because a message is encrypted to an encryption subkey where a key has one.

Both are departures from what the first run published as the design. They are
written down in their own doc comments as later additions, with what writing the
implementation found.

### The four sentences, quoted

> This message is encrypted with PGP and there is no private key on this
> computer, so nothing could be tried. Import your key from the File menu. What
> is shown below is the encrypted form rather than the message.

> This message is encrypted with PGP and the private key on this computer is not
> the one it was encrypted to, so it was meant for somebody else. What is shown
> below is the encrypted form rather than the message.

> This message is encrypted with PGP and the private key on this computer could
> not be read back, so nothing could be tried. Importing your key again may fix
> it. What is shown below is the encrypted form rather than the message.

> This message is encrypted with PGP and the encrypted part is damaged, so it
> cannot be opened even with the right key. Ask whoever sent it to send it again.
> What is shown below is the damaged form rather than the message.

None of them is 04-03's general sentence, none is task 1's S/MIME sentence, and a
test asserts all of that pairwise. A guard record takes the wrong-key arm and
makes it answer the no-key sentence, which is how this really goes wrong: nobody
merges four arms, somebody reaches for the sentence next to the one they need.

### One place decides, and the reader never says both

04-03 already answers whether a message is PGP-encrypted, through
`body_safety::what_the_form_says`, and `application::opening_pgp` consumes that
answer rather than asking again.

**The armour is replaced by the words before the document is built.** That is
what stops the reader saying both: `single_message` reads the form of the body it
is handed, so a body with no armour produces no sentence about armour and there
is nothing to take back out. A message that did not open keeps its armour, which
is the thing somebody can copy elsewhere or forward to whoever can read it, and
`with_pgp` narrows the general sentence to the reason.

### Reachability, hop by hop

**Importing a key.** File menu, "Import PGP Private &Key... (experimental)" ->
`wx_app::import_a_pgp_private_key` -> `std::fs::read_to_string` ->
`service::pgp::import_a_private_key` -> `service::pgp::keys::import` ->
`SignedSecretKey::from_string`, the passphrase check, then
`service::secret_store::write` under `wixen-mail-pgp` / `private-key`.

**Opening a message.** A message arrives and its body is stored. Somebody opens
it: `wx_app::open_in_the_text_reader` -> `application::opening_pgp::for_body` ->
`body_safety::what_the_form_says` -> `service::pgp::open_a_message` ->
`service::pgp::keys::open` -> `secret_store::read` -> `Message::from_armor` ->
`decrypt` -> `decompress` -> a bounded read -> `WhatOpeningItFound::Opened`.
Back up: `opening_pgp::the_body_to_show` puts the words where the armour was,
`reader_text::single_message` builds the document, `with_pgp` narrows the
sentence when there is one.

Shift+Space takes the same path through `read_the_whole_message` and
`whole_message_reading`, so the reading aloud and the reader window say the same
thing about the same message. Three guard records hold all of that.

### What is deliberately not built, said in the product

**PGP/MIME is not read**, and it is not reported as failing either: nothing sees
it. `what_the_form_says` reads the message's text parts, and a
`multipart/encrypted` message puts the armour in a separate part that never
becomes body text. Thunderbird and most modern clients send PGP/MIME, so this is
the common shape rather than a corner.

One key at a time. Nothing goes out signed or encrypted. All of it is in the
changelog, in the ledger, and behind the experimental warning on the menu item,
which says what could go wrong rather than only that the feature is new:

> Reading PGP mail is experimental. No message from a real correspondent has ever
> been through it and no real key has ever been imported, so it may say a message
> cannot be opened when another mail program opens it. Nothing is sent anywhere
> and nothing on your account is changed. Wixen Mail holds one key at a time, and
> it has to be exported without a passphrase, because nothing here asks you for
> one.

On the label as well as in the description, because the description is what
Windows hands a screen reader and the label is read whatever anybody's settings
say.

## The dependency

`pgp = "0.20"`, the rPGP project, added with the justification comment beside it.
Two costs are in that comment, in the comparison document and in the changelog,
rather than left in a chat message:

- **`rsa` is vulnerable to the Marvin timing attack**, unfixed, tracked at
  RustCrypto/RSA issue 19 and named in rPGP's own security notes. It matters to
  an attacker who can both send this program messages and measure how long it
  takes to open them.
- **rPGP's CI covers `x86_64-pc-windows-gnu`.** This project builds
  `x86_64-pc-windows-msvc`, which its platform list marks as expected to work and
  does not test.

`rust-version` moved from 1.87 to 1.88 in the same commit, because rPGP declares
1.88.

### The MSRV bump cost seven clippy fixes in files this plan never opened

Not anticipated and worth recording. Clippy reads the declared floor and only
suggests constructs the floor allows, so raising it to 1.88 turned on
`collapsible_if`'s let-chain form across the tree: five sites in the library and
two more in test targets, in `wx_managers`, `outlook_data_file`, `contacts_sync`,
`outward`, `protocols::imap` and `tests/checkbox_labels`. All were fixed properly,
none with `#[allow]`. The cost of a version floor is not only which toolchains
can build: it is every diagnostic the old floor was suppressing.

## The plan's premises

**Six wrong this run, on top of the six the first run found.** All measured.

**1. Task 1's route does not exist**, the first run's finding, restated above
and now resolved by decision rather than ignored.

**2. There are four arrival paths, not three.** The plan and my own first guard
both assumed the three under `src/application/`.
`presentation::wx_app::spawn_body_fetch` is the fourth.

**3. The brief's `wx_app.rs` figure is wrong.** It said 40 records and 195 tests.
Measured 2026-09-06 by parsing `tests_last_seen`: **40 records, 199 tests**, and
the file really holds 199. The record count is right and the test count is not.

**4. Decision 2's 36-construction-site budget did not apply.** 51 sites exist and
none changed, because the mark is read by row id.

**5. Premise 7 is wrong that `spoken` should not be reworded**, and 04-03 had
already argued the other way. Corrected in this plan.

**6. The plan's "no `#[test]` in `wx_app.rs`" was followed and the wiring guards
went to `tests/wired.rs` instead**, which costs eight records rather than forty.
Reported before and after: `wx_app.rs` held 199 tests at the branch point and
holds 199 now.

**What the plan got right.** Premise 4 in every part: `layout_of` returns
`Encrypted` for all three recognition cases, folds case, and falls back to the
file suffix, and `claims_a_signature` must not be widened. Premise 5 about the
three states. Premise 6 about `CertificateStore`. Premise 8 about
`forget::entries_for`. Premise 10 about a green test going red for the right
reason, which happened twice.

## Guard records

**Five added, one re-measured, all by hand from a green tree.** 627 records at
the branch point, 632 now.

| Record | Break | Red |
|---|---|---|
| an encrypted message that nothing ever recognised as encrypted | `claims_encryption` answers no to everything | 4 |
| an encrypted message whose body still says it may not have been downloaded | the body replacement returns the text unchanged | 3 |
| a PGP private key somebody can import | the menu item gone | 3 |
| a PGP message the reader window never offers to the key | the opener never called | 1 |
| the reasons a PGP message did not open stay four different sentences | the wrong-key arm answers with the no-key sentence | 2 |
| signed mail brought in from a file keeps the bytes its signature is over | re-measured: this plan moved the code | 2, was 1 |

Two of those measurements corrected a first draft, and both are written into the
records:

- The menu-item break reddens **three**, not the two I expected.
  `test_every_handled_command_has_something_that_raises_it` is an older guard
  that reads every menu arm and catches this without being about PGP at all.
- The body-replacement break does **not** redden the reader's own PGP tests,
  because they hand `with_smime_envelope` an answer directly. What covers that
  hop is the mark in the cache. Recorded rather than assumed.

### A record went stale inside this plan, exactly as 04-08 warned

`what a message says about its own form is read from the message rather than
answered ordinary` named 11 tests. It reddens **14** now: everything on the PGP
path asks that one question first, so `opening_pgp` and both of the reader's new
PGP tests reach it. The record was corrected by hand with the full list and
re-measured.

**What caught it was the per-commit count check and the scoped remedy it
printed**, not a sweep. 20 records were flagged across the two green commits and
all 20 were re-measured with `scripts/guards.sh --remeasure`; 19 agreed with
their records and this one did not.

`scripts/guards.sh --touched-by 50ed052` is owed after the merge and must not
block it.

## Deviations from plan

**Rule 1, auto-fixed.** Seven clippy failures across six files, all caused by the
MSRV bump, all in files this plan did not otherwise touch.

**Rule 1, auto-fixed.** `service::outward`'s dependency census names every crate
on one of two lists and `pgp` was on neither. Added to the list of crates that
cannot reach a server, with the reason: OpenPGP does have key servers and nothing
here calls that half.

**Rule 1, auto-fixed.** `forget.rs`'s reading of module names answered `keys` for
`src/service/pgp/keys.rs`, which is not a module `entries_for` can name. Narrowed
to the top-level module, and that showed `src/service/mod.rs` had been excused by
coincidence: it answered `service`, and `entries_for`'s body contains that word
because it is the name of a `CredentialEntry` field. Now excluded on purpose.

**Deliberate departure.** Task 3's three wiring guards in `tests/wired.rs` are
green in the RED commit, because the menu item has to exist for the arm that
catches it to compile. They were taken red by hand afterwards as guard records,
which is stronger evidence than a red commit and is what this project asks of a
guard. Said in the RED commit message rather than left for somebody to notice.

**Deliberate departure.** `WhatOpeningItFound` and `WhatImportingAKeyFound` each
gained a fifth variant, against what the first run published. Both are cases
writing the implementation found and neither is expressible in the four.

## Known stubs

None in the sense the guardrail means. Everything written is reached from a
non-test path and the reachability list above names each hop.

What is deliberately incomplete and said out loud, in the product as well as
here: PGP/MIME is not read, one key at a time, a key with a passphrase is
refused, nothing goes out signed or encrypted, and S/MIME encrypted messages
still cannot be opened.

One thing worth naming that is not a stub but is a limit: **a message already in
the cache goes on opening blank.** It was stored before anything recorded that it
arrived encrypted, and the mark only reaches messages fetched from now on. That
is in the changelog.

## What did not close

**READ-02 does not close.** Its second `[D]` line, outgoing signing and
encryption, is untouched and was never in this plan. Its first `[D]` line is
built and has never met a real key or a real message.

**Criterion 5's second clause is closed structurally for S/MIME.** An enveloped
message that read as empty now says why.

**Criterion 5's first clause is `unrun-verify`, not met and not failed.** The
code is written, wired, reachable, and tested against a key and a message another
implementation made. No account has ever been used with this program and no
correspondent has ever sent it PGP mail.

## What only a real key, a real message and a screen reader can settle

Recorded in `.planning/WINDOWS.md`, which held 140 entries before this plan and
holds 146 now.

- Whether `EncryptedMessage::read` parses a real envelope from Outlook or
  Thunderbird. If it does not, the message is wrong rather than absent, which is
  worse than the blank body it replaces.
- Whether rPGP opens a real correspondent's message, and whether it refuses one
  that another client opens. Both directions are worse than the armour they
  replace.
- Whether this computer's certificate store answers correctly about a real S/MIME
  certificate. The three answers are tested against a store held in memory; the
  Windows path that produces them is not.
- Whether the sentence in an encrypted message's body is reached before somebody
  concludes the message is broken, and whether hearing the same sentence in the
  bar and again in the body is thoroughness or repetition.
- Whether the five answers the key import gives are understood on hearing them
  once, with no dialog to go back to.

## Self-Check: PASSED

Files claimed as created, verified present:

- `src/data/message_cache/how_it_arrived.rs` FOUND
- `src/application/encrypted_mail.rs` FOUND
- `src/application/opening_pgp.rs` FOUND
- `src/service/pgp/keys.rs` FOUND

Commits claimed, verified in `git log`:

- `d226d3d` FOUND
- `3fde14f` FOUND
- `28e26b3` FOUND
- `ad6ec18` FOUND

Tests: `cargo test --lib` reports 6,488 passed, 0 failed, 1 ignored.
`scripts/check.sh all` ran as `ad6ec18`'s commit gate and reported all four
checks passed.

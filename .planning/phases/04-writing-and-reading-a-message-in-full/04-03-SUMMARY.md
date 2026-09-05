---
phase: 04-writing-and-reading-a-message-in-full
plan: 03
executed: 2026-09-05
status: complete
tasks: 2
requirements: [READ-02]
subsystem: application, presentation, service
tags: [pgp, encryption, warning-bar, read-02, guard-record, dropped-fields]
commits:
  - 874b7f3 test(04-03) failing tests for the encryption nobody is told about
  - 5c3a589 An encrypted message explains its armour instead of leaving it there
merged: not merged, and not pushed
branch: worktree-agent-afc63c76d74ba20dd
branched_from: 956b196
key-files:
  created:
    - tests/an_encrypted_message_is_not_left_unexplained.rs
  modified:
    - src/service/security.rs
    - src/application/body_safety.rs
    - src/presentation/reader_text.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
requires: []
provides:
  - "body_safety::WhatTheFormSays and what_the_form_says, a pure decision over the two halves of a body"
  - "security::both_halves_of_a_body, the join both readers share"
  - "SecurityService::detect_pgp_signed and detect_pgp_encrypted reachable without an instance"
  - "ReaderDocument::with_encryption, folded in before any signature verdict"
  - "tests/an_encrypted_message_is_not_left_unexplained.rs, a census over both composers"
affects:
  - "04-09, which builds PGP reading: it must correct this plan's encrypted sentence and its fixtures when opening becomes possible"
decisions:
  - "The question is asked inside the two composers, not at the three call sites, so no door can forget it"
  - "The fold goes in before with_signature, because said_before_the_message cuts at HOW_IT_WAS_CHECKED"
  - "The encrypted sentence is scoped to this message, not to the program, so 04-09 cannot falsify it"
  - "A thread of several messages says nothing about one message's form, the limit with_signature already keeps"
metrics:
  duration: about 3 hours
  library_tests_before: 6272
  library_tests_after: 6291
actuals:
  tokens: 71000
  tasks: 2
  commits: 2
---

# Phase 4 Plan 03: An encrypted message says it is encrypted Summary

**One-liner:** The program has worked out that a message is PGP-encrypted on
every read since that code was written and told nobody; it now says so above the
message, in words that do not claim more than they should.

## What works

Open a PGP-encrypted message and the bar above it says, and the screen reader
speaks before the body:

> This message is encrypted. Wixen Mail cannot open it, so what is shown below
> is the encrypted form rather than the message.

Open a message carrying a PGP signature and it says:

> This message carries a PGP signature, which Wixen Mail cannot check, so
> nothing here says whether it is genuine.

Ordinary mail is unchanged: no bar where there was none, no extra line to listen
past. The unsafe-message cue does not sound for either, so both arrive as an
ordinary announcement.

### The three doors, and all three say it

Asked for as a list rather than an assertion, because a fact that reaches one
door and not another is one that appears and disappears depending on which way
somebody came in.

| # | What a person does | Function | What carries the fact |
|---|---|---|---|
| 1 | Enter on a message, formatted, which is the default | `wx_app::open_single_message` -> `show_conversation_as_page` | `reader_text::conversation` folds it in |
| 2 | Enter, with plain text chosen in settings | `wx_app::open_single_message` -> `open_in_the_text_reader` | `reader_text::single_message` folds it in |
| 3 | Space, which reads the message aloud without opening it | `wx_app::read_the_whole_message` -> `whole_message_reading` | `reader_text::single_message` folds it in, and `read_whole` speaks it first |

Then, for all three: `ReaderDocument.warning` -> `wx_reader::open` ->
`reader_text::said_before_the_message` -> `a11y.announce` at `Priority::Normal`.

**The question is asked inside the two composers rather than at those three call
sites**, which is a deviation from the plan and the most important design
decision here. Three call sites is three chances to forget, and a fourth surface
would be added by somebody who had never heard of this. Both composers are pure
functions taking the body they already hold, so the question costs them nothing.

## The two sentences, and why neither says what it was nearly made to say

**The encrypted sentence is deliberately narrower than the one it is modelled
on.** `signed_mail::EncryptedMessage::spoken` already writes, for the S/MIME
case:

> This message is encrypted. {who} Wixen Mail cannot open encrypted mail yet, so
> nothing of it can be read here.

The plan says to word the new sentence "as one voice" with that one. Taken
literally that would have put a second copy of "cannot open encrypted mail" into
a second file, and 04-09 builds PGP reading, so the claim is already scheduled to
go false. The new sentence keeps the vocabulary and the opening clause verbatim
and scopes the claim to the message in front of somebody:

| | |
|---|---|
| `EncryptedMessage::spoken` | "This message is encrypted. This computer holds a certificate this message was encrypted to. Wixen Mail cannot open encrypted mail yet, so nothing of it can be read here." |
| this plan | "This message is encrypted. Wixen Mail cannot open it, so what is shown below is the encrypted form rather than the message." |

`test_the_sentence_does_not_claim_this_program_can_never_open_encrypted_mail`
asserts the wider phrasing is absent. **04-09 still has to correct
`EncryptedMessage::spoken`**, which is not this plan's to change, and this
summary is the notice.

The second half of the sentence is load-bearing rather than decorative. The body
below is not empty, it is armour, so "nothing of it can be read here" would be
wrong: the sentence has to explain what the person is looking at, or a program
that looks broken invites trying again.

**The signed sentence is worded from
`Finding::SignatureKindNotUnderstood`**, which this project already wrote for
exactly this situation: "This is a {named} signature, which this program cannot
check, so nothing here says whether it is genuine." The two now read as one
voice.

It must not read as good news. "Signed" is heard as "genuine" whether or not
anything looked, which is what a forger is buying, so what was not done is said
in the same breath as what was found. That is the discipline
`nothing_kept_to_check_bar` keeps for the S/MIME case, and its doc comment
argues it: could not check and checked out fine are opposite pieces of news.
`test_the_signed_sentence_makes_no_claim_a_check_would_have_to_earn` drives five
phrasings a tightened sentence might reach for and asserts none is there.

**One wording divergence, stated rather than hidden.** `SignatureKindNotUnderstood`
says "this program" and `EncryptedMessage::spoken` says "Wixen Mail". Both new
sentences say "Wixen Mail", so they agree with each other and with the
encryption sentence's model, at the cost of disagreeing with the signature one.
Nobody has heard either.

## The four fields left alone, and why each

The plan asks for this per field and it is the half of the work that is a
decision rather than code. The reasoning lives in `what_the_form_says`'s own doc
comment so it is beside the code rather than only here.

| field | why it is left |
|---|---|
| `signature_status` | A substring search that returns `Valid` because the words "good signature" appear anywhere in the message. A stranger writes the body. `checking_signatures::for_message` answers this properly, with a DER reader and a real certificate store, and its answer already reaches this same bar through `with_signature`. Wiring the weak answer beside the strong one is two answers to one question, and the weak one is the one an attacker can write. |
| `smime_signed` | Same shape: `application/pkcs7-signature`, `.p7s` and `smime-type=signed-data` searched for in body text. Same existing strong answer. |
| `smime_encrypted` | **It cannot fire through this path at all.** An S/MIME enveloped message has no `text/*` part, so `mime::parse`'s `first_of_kind` yields `None` for both halves, `from_body` is handed two empty strings, and the detector searches an empty string for `application/pkcs7-mime`. A test wired through this path for that field would be a test that could not fail in production. The S/MIME case is answered from the message's `Content-Type`, which is 04-09's. |
| `phishing_score` | The number behind `phishing_risk`, which already reaches the bar. Showing a number as well is a separate decision about what a score means to somebody reading it, which is READ-03's territory. |

So the two wired are the two that can fire through a body and have no other
answer anywhere in the tree.

## Premises that were wrong

### 1. The gate and the bar are not on one path, and the census the plan asks for would have read an unrelated function

Premise correction 2 says `from_body` "is called from `wx_app.rs`'s message-open
path only when that setting is on", tells the executor to "ask the encryption
question outside the gate", and task 2 asks for a census asserting the question
"is asked outside the arm the content-scanning setting gates".

There are two paths, separated in time by the database.

- `look_at_message_contents` is read in `spawn_body_fetch`, on a worker, when a
  message body is downloaded. What it gates is `from_body`, whose verdict goes
  into the `safety` column through `merge_into`.
- The bar is built later, on the interface thread, by `single_message` or
  `conversation`, from `message.safety` on the row. Nothing on that path reads
  the setting at all.

So anything folded in at composition time is outside the gate by construction,
and a census anchored on the setting's arm would have been asserting something
about `spawn_body_fetch`, which has nothing to do with this fact. It would have
passed forever whatever the feature did.

Worse, the obvious reading of "wire the dropped fields" is to compute the new
fact inside the gated analysis so it rides along with the verdict. That inherits
both the gate and the storage, and it is the defect the plan exists to fix.

The census says the true thing instead: every composer asks and folds, and no
file that asks the question also reads the setting.

### 2. "Fold the sentence in under whatever the bar already says" is silent loss on a signed message

`said_before_the_message` returns everything above `HOW_IT_WAS_CHECKED`. It is
what `wx_reader` speaks when a message opens and what `read_whole` speaks before
the body. A signature verdict is what puts that line into the bar.

So a sentence appended after a signature verdict is in the bar, visible on the
screen, and spoken by neither surface, on exactly the messages with the most
going on.

The fold therefore goes in **before** `with_signature`, which lands it in the
spoken prefix by construction rather than by string surgery, and
`test_the_form_is_said_before_the_message_even_when_a_signature_verdict_follows`
drives it: it composes an encrypted message, folds in `SignatureCheck::NotKept`,
asserts the boundary really is in the bar, and then asserts the sentence survives
both `said_before_the_message` and `read_whole`.

### 3. The guard-record table is wrong for two of its four rows

Counted by parsing every record's `tests_last_seen` rather than by grepping for a
file name.

| file | plan says | actually |
|---|---|---|
| `src/application/body_safety.rs` | 0 records, 13 tests | **correct** |
| `src/service/security.rs` | 4 records, 15 tests | **correct** |
| `src/presentation/reader_text.rs` | 0 records, 76 tests | **1 record, 81 tests** |
| `src/presentation/wx_app.rs` | 39 records | **40 records** |

Both wrong figures were true before 04-01 and 04-02 landed, hours before this
plan was written. That is the third plan in this phase whose record table expired
against a same-day sibling, and it is now predictable enough to be worth stating
as a rule: **a plan's guard figures are a measurement with a date, and in a phase
running several plans a day they are stale on arrival.**

The consequence was real and paid: `test_every_guard_record_says_how_many_tests_the_files_it_names_held`
named "an attachment nobody described says so rather than trailing off", and
`scripts/guards.sh --remeasure` was run on it. It still reddens exactly the two
tests it names, in both directions, and its counts are written down again.

### 4. The plan's `<verify>` command does not run

`cargo test --lib application::body_safety:: --lib presentation::reader_text::`
is refused by cargo: `--lib` cannot be given twice. Exactly the defect 04-02
reported for its own plan, in the same shape, one plan later. Run as two
commands: 20 and 93.

### 5. A variant nothing constructs is not dead code here

The plan warns that under `-D warnings` "a variant nothing constructs is dead
code and the build is refused", and tells the executor to shape the stub around
that. It does not apply: `application` is a `pub mod` and `WhatTheFormSays` is a
`pub enum`, so its variants are public API and the lint cannot fire. The stub was
free to answer `Nothing` unconditionally, which made five more tests red than the
plan's shape would have.

## Verification

Both commits went through the hook. Nothing used `--no-verify` and no
`#[allow(...)]` was added.

| commit | mode | library | release |
|---|---|---|---|
| 874b7f3 | red | 14 named, 14 failed, nothing else | not run |
| 5c3a589 | all | 6291 passed, 0 failed, 1 ignored | clean, 5m06s |

The green ran `all` because it bumps `Cargo.toml`, so it was run detached: that
gate is about eleven minutes in a fresh worktree, because the release build
compiles every dependency from nothing.

### What was red, and what arrived green

The plan asks for this and the answer is not uniform. The stub was half the real
decision: the fold into the bar was written and worked, and the question answered
`Nothing` whatever the body held. So every failure was a value, not a missing
symbol.

**Decision tests: five of seven red.** The two that passed at their own red assert
that ordinary mail says nothing about its form, which is what a stub answering
`Nothing` says too. They discriminate against the finished code, which the guard
record confirms: the same break reddens neither of them, because they are the two
the break agrees with.

**Bar tests: six of eleven red.** The five that passed at their own red are about
the wording of the two constants and about the fold itself, which the stub already
had: that nothing to say changes nothing, that ordinary mail gains no bar, that a
thread of several says nothing about one message's form, and the two wording
assertions.

**Census: two of ten red**, and the plan says the census cannot be red at all.
Task 2's action says it "asserts what task 1 has already done, so it is green on
arrival and there is no ordering inside this plan that changes that", and
prescribes a hand-measured break in place of the missing red. That is wrong for
the reason 04-02 found it wrong for its own census: what the census names is a
call, and a call is absent before it is written. Written first, it failed at once.
The other five were its companions, which read made-up source.

One failure verbatim:

```
thread 'test_every_composer_asks_what_form_the_message_is_in' panicked at
tests\an_encrypted_message_is_not_left_unexplained.rs:186:9:
the composer opening `pub fn single_message(` in src/presentation/reader_text.rs
never asks `what_the_form_says(`, so a message opened through it shows its armour
with nothing said about why.
```

### The finding: the census passed against its own break

Worth more than the feature, and it was found only by applying the break rather
than by reading the check.

The census as first written asserted that each composer **asks** the question.
The break measured for its record removed the **fold** that puts the answer into
the bar and left the question in place, bound to `_form`. The census stayed green:
9 passed, 0 failed. The composer still named the call, and the answer went
nowhere.

A call site has three independent ways to be hollow, and the check covered one:

1. the call is absent,
2. the result is discarded,
3. the argument is a constant, so the call decides nothing.

The census now asserts all three, with a companion per shape against made-up
source. The third check was itself evadable on its first attempt: it matched
`.with_encryption(WhatTheFormSays::`, and the composers spell the type by its
full path, so `crate::application::body_safety::WhatTheFormSays::Nothing` would
have slipped past. It now splits at the fold and looks for the type name anywhere
after it, and the companion is written with a full path for that reason.

Ledger entry 106.

### Three guard records, all measured by hand with `--no-fail-fast`

| record | break | red list | total |
|---|---|---|---|
| what a message says about its own form is read from the message rather than answered ordinary | the decision answers `Nothing` always | the 5 `application::body_safety::tests::` form tests and the 6 `presentation::reader_text::encryption_tests::` bar tests | 6280 passed, 11 failed, 1 ignored |
| the reader folds what a message said about its form into the bar rather than dropping it | the fold removed from `single_message`, the question left | `test_every_composer_asks_what_form_the_message_is_in` | 9 passed, 1 failed |
| the fold is handed what the message said rather than a named variant | the fold handed `WhatTheFormSays::Nothing` by its full path | `test_every_composer_asks_what_form_the_message_is_in` | 9 passed, 1 failed |

The first is measured against the library. The second and third name the census
as their `suite`, because `scripts/guards.py` runs either the library or one
integration target per record, not both, and a third composer arriving half-wired
is what only the census can see.

The sweep header reads 192 plus 416, which
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
checks on every commit. 608 records.

### Test counts, before and after

**No `#[test]` was added to `wx_app.rs`, and the file is not in this branch's
diff at all.** Its 40 records were not touched.

`security.rs` holds 15 tests before and after: four call sites inside its own test
module changed from method to associated-function syntax, and no test was added
or removed, so its four records were not flagged.

Counts that moved: `body_safety.rs` 13 to 20, `reader_text.rs` 81 to 93, and
`tests/an_encrypted_message_is_not_left_unexplained.rs` from nothing to 10. The
library went 6272 to 6291.

## Reaching the existing detectors

The plan says to draw on `security.rs`'s detectors rather than write a second set
of substring checks, and to say what the smallest change was.

`detect_pgp_signed` and `detect_pgp_encrypted` took `&self` and never used it.
They are now associated functions and `pub(crate)`, so `body_safety` reaches them
without building a `SecurityService`. That matters beyond tidiness:
`SecurityService::new()` resolves application paths and can fail, and `from_body`
returns `Verdict::ordinary()` when it does, which is the ordinary case on a fresh
installation. A decision that had to build one would have answered "nothing about
its form" on exactly those machines.

The join of the two body halves moved into `both_halves_of_a_body`, shared by
`analyze_message_security` and the new decision, so a marker sitting across the
seam cannot be found by one reader and missed by the other.
`test_a_marker_is_not_invented_across_the_seam_between_the_two_halves` asserts the
other direction: the halves are joined with a line ending, so a marker cannot be
assembled out of the end of one and the start of the other.

## Untrusted input, and what was decided about it

The armour markers are substrings anybody can write into an ordinary message,
and what they produce is a sentence read before the message.

**T-04-09 and T-04-10, accept, as the plan says.** An ordinary message quoting
`-----BEGIN PGP MESSAGE-----` is told it is encrypted when it is not. The cost is
one wrong sentence above a message that is perfectly readable, and the body is
still shown underneath, so nothing is hidden and nothing is refused. Detection by
marker is what `security.rs` has done on every message for the life of that code;
this plan changes who is told, not how it is decided. Said in the changelog as a
known limitation rather than implied away.

**T-04-11, mitigate.** The signed sentence says the message carries a signature
and says in the same sentence that nothing checked it.
`test_the_signed_sentence_makes_no_claim_a_check_would_have_to_earn` drives five
phrasings a tightening might reach for. `checking_signatures` remains the only
thing in the tree that ever says a signature is good, and the plan's stronger
worry, that the substring-based `signature_status` might be surfaced beside it,
is answered by not wiring that field at all.

**T-04-12, mitigate.** `looks_unsafe` is untouched by the fold, with the comment
`with_signature`'s `NotKept` arm already carries.
`test_an_encrypted_message_is_not_reported_as_an_unsafe_one` asserts the bar is
there and the flag is not, and
`test_a_filters_warning_keeps_the_top_of_the_bar_and_the_form_goes_under_it`
asserts the other direction, that a real phishing verdict still sounds the cue.

**T-04-SC does not apply.** No package was added. The detectors were already in
`src/service/security.rs` and the bar in `src/presentation/reader_text.rs`. No
package-manager install ran, so the legitimacy gate never came up.

## Deviations

**The question is asked in the composers, not at three call sites in
`wx_app.rs`.** The plan's `files_modified` names `wx_app.rs` and its action says
to reach the fact from there. There are three composition sites, not one, each of
which would have needed the fold in the right position relative to
`with_signature`. Putting it in the two composers makes all three doors correct,
makes the ordering right by construction, and leaves `wx_app.rs` untouched, which
also costs its 40 records nothing. The census reads `reader_text.rs` for the same
reason.

**Two commits, not one per task.** Task 2's census went into the red commit,
because it was genuinely red and belonged with the other failing tests, and its
two guard records went into the green with the first one. Splitting them would
have meant a red commit that named the census and a later commit that added
records for code already shipped. 04-02 made the same call for the same reason.

**A third guard record.** The plan asks for two, one per task. The constant-fold
defect is a distinct shape from the dropped-answer defect, it was measured
separately, and after the census's own break got past it once, leaving it
unrecorded would have been the same mistake again.

**A conversation of several messages is left alone.** Covered above and in ledger
entry 105.

**One version bump, not two.** 0.61.0 to 0.62.0. One feature reaching one
surface.

## What this cannot see

**Nothing here has met a screen reader or a real account.** Ledger entries 101 to
107.

- **Nobody has heard any of it.** Whether the encryption sentence lands before
  somebody reaches the armour, and whether the two sentences are understood at
  speed, are judgements. Entry 101.
- **Whether a filter verdict and an encryption sentence read as two facts or one
  run-on** is unmeasured. They are joined with a line ending, which is two lines
  in a text control and may be one breath in speech. Entry 102.
- **Whether the signed sentence is heard as a disclaimer or as reassurance** is
  the one that matters most and the one tests cannot reach. They assert which
  words are absent, which is a check on the wording and not on what somebody
  takes away. Entry 103.
- **Whether real PGP mail arrives with its armour in a text part at all.** Mail
  sent as `multipart/encrypted` carries the armour in a part of its own, which
  `mime::parse` does not yield as a body, so it would not reach this. No account
  has ever been used with this program, so which shape real senders use cannot be
  answered here. The changelog says so. Entry 104.
- **The census reads source.** It says where the question is asked and the answer
  folded in, and nothing about whether those lines are reached at run time, what
  a real message looks like on the wire, or what the sentence sounds like. The
  first of those is answered by the three-door table above; the other two are not
  answered here at all.

**Owed after the merge:** `scripts/guards.sh --touched-by 956b196`. This branch
changes `body_safety.rs`, `reader_text.rs` and `security.rs`, which 0, 1 and 4
records fingerprint. The one flagged by the count check was already re-measured
here, because running the scoped remedy when a commit prints it is not optional.
Per `CLAUDE.md`'s decision of 2026-09-03 the sweep belongs once per completed
phase and must not block the merge.

**The ledger did not end where the brief said.** The task brief says it ends at
98; it ends at 100, because 04-02 wrote entries 94 to 100. This plan's entries are
101 to 107.

## Known stubs

None. Every field added is filled by a non-test path, and the three-door table
names the function at each hop. The two facts this plan does not surface,
`smime_encrypted` and the S/MIME signature status, are not stubs: one is
unreachable through this path by construction and the other already has a
stronger answer reaching the same bar.

## Requirements and criteria

**Success criterion 5 is half closed and the half is named.** "A message that
cannot be decrypted says why instead of reading as empty" is closed for the PGP
case, in the shape it actually fails in, which is armour shown with no
explanation rather than an empty body. The S/MIME case, which is the one that
really reads as empty, is 04-09's, and decrypting anything is 04-09's.

**Criterion 5's first clause is untouched.** "A user reads a PGP-encrypted
message they hold the key for" needs an OpenPGP implementation. This plan does
not advance it and does not pretend to.

**READ-02 is not closed.** PGP key handling, decryption and signing are all still
absent.

**A note for 04-09.** Its task 3 makes `EncryptedMessage::spoken` reachable. That
sentence says Wixen Mail "cannot open encrypted mail yet", which its own work
falsifies. This plan's sentence is already scoped so it survives, and a test
holds it there; the older one is not.

## Self-Check: PASSED

- Both commits are in `git log`: 874b7f3, 5c3a589.
- `.planning/phases/04-writing-and-reading-a-message-in-full/04-03-SUMMARY.md`
  exists.
- `tests/an_encrypted_message_is_not_left_unexplained.rs` exists and holds 10
  tests.
- `guards/guards.toml` holds 608 records, three added by this plan, and its sweep
  header reads 192 plus 416.
- `.planning/WINDOWS.md` holds entries 101 to 107, written into this worktree.
  The shared checkout's copy is byte-identical to what it was before this plan
  ran: `602606793b908466844be88fbb4a4473` before and after, checked with
  `md5sum`.
- Nothing was merged and nothing was pushed.

---
phase: 04-writing-and-reading-a-message-in-full
plan: 02
executed: 2026-09-05
status: complete
tasks: 2
requirements: [READ-03]
subsystem: service, data, presentation, application
tags: [list-unsubscribe, blocking, mailing-lists, read-03, guard-record, unreached-code]
commits:
  - 112098f test(04-02) failing tests for a way out of a list nobody ever heard about
  - a77586a Blocking a mailing list says so first, and says where to leave
  - 15e13b4 Three records for the three ways the list warning goes quiet again
  - 585b9e9 A test per arrival path, and two records the count check found wrong
merged: not merged, and not pushed
branch: phase-04-02-list-unsubscribe
branched_from: 976f16c
key-files:
  created:
    - tests/the_list_warning_reads_the_message.rs
  modified:
    - src/service/mime.rs
    - src/service/protocols/imap.rs
    - src/service/outlook_data_file.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/messages.rs
    - src/data/message_cache/searching.rs
    - src/data/message_cache/saved_searches.rs
    - src/data/message_cache/tags.rs
    - src/data/message_cache/outbox.rs
    - src/presentation/ui_types.rs
    - src/presentation/wx_app.rs
    - src/application/blocking.rs
    - src/application/mail_sync.rs
    - src/application/pop_sync.rs
    - src/application/filing.rs
    - tests/integration_tests.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
requires: []
provides:
  - "ParsedMessage.list_unsubscribe, the header as it arrived, scrubbed and bounded"
  - "mime::A_WAY_OUT_AT_MOST, the bound a stranger's way out is cut at"
  - "an additive list_unsubscribe column on the messages table, read by all five listings"
  - "LIST-UNSUBSCRIBE on the IMAP header fetch"
  - "MayBlock::YesButFirst reachable in a shipped build for the first time"
  - "tests/the_list_warning_reads_the_message.rs, a census over both ends of the route"
affects:
  - "nothing in phase 4 depends on this; it lifts out whole if research decision 5 is answered no"
decisions:
  - "Read through header_raw, not header_text, because mail-parser strips the brackets the consumer parses for"
  - "Presence is the fact: a header there and empty is Some(\"\"), only an absent one is None"
  - "Bounded at 998 characters at the parse boundary, and the cut fails closed"
  - "LIST-UNSUBSCRIBE added to the IMAP header fetch, which the plan does not mention"
metrics:
  duration: about 5 hours
  library_tests_before: 6263
  library_tests_after: 6272
actuals:
  tokens: 84000
  tasks: 2
  commits: 4
---

# Phase 4 Plan 02: A way out of a mailing list, said out loud Summary

**One-liner:** `List-Unsubscribe` is parsed, stored and carried to the block
handler, so the mailing-list warning that has existed since blocking was built
and has never once been said in a shipped build now fires, and names the address
to write to when the message gave one.

## What works

Block the sender of a newsletter and Wixen Mail now says, before the block is
made:

> This message came from a mailing list. Blocking files it into Junk and the
> list carries on sending it. To stop it at the source, unsubscribe by writing
> to birds-leave@lists.example.

Then it makes the block, because that is what was asked for. When the message
named no address to write to:

> This message came from a mailing list. Blocking files it into Junk and the
> list carries on sending it. The message gives no address to unsubscribe at,
> so look for a link to leave the list at the bottom of it.

Both sentences already existed. Neither had ever been said. Blocking an ordinary
sender is unchanged: `MayBlock::Yes`, nothing extra.

Reached from Message menu, Block, then This Sender or Everyone at This Domain.

## The route, non-test function at every hop

Asked for as a list rather than an assertion, because a fact dropped at any one
of these is a fact nobody is ever told.

| # | Function | What it carries |
|---|---|---|
| 1 | `service::protocols::imap::header_query` | puts `HEADER_FIELDS` into the FETCH, so the server sends the header at all |
| 2 | `service::mime::parse` | calls the reader below |
| 3 | `service::mime::how_to_leave_the_list` | reads the raw header, scrubs it, bounds it |
| 4 | `service::protocols::imap::message_from_attributes` | onto the `ImapMessage` |
| 5 | `application::mail_sync::to_incoming` | onto the `IncomingMessage` (IMAP) |
| 5b | `application::pop_sync::to_incoming` | the same, off a `ParsedMessage` (POP) |
| 5c | `application::filing::a_row_filed_here` | the same, for a sent copy and an import |
| 6 | `MessageCache::upsert_message` | the INSERT, now carrying `list_unsubscribe` |
| 7 | `MessageCache::get_message_list_sorted` | the SELECT, through `messages::listing_row` |
| 8 | `presentation::ui_types::MessageItem::from_row` | onto the row the window holds |
| 9 | `presentation::wx_app::block_the_sender` | into `WhatIsAlreadyTrue` |
| 10 | `application::blocking::may_block` | decides `YesButFirst` |
| 11 | `application::blocking::a_list_goes_on_sending` | writes the sentence |
| 12 | `application::blocking::where_to_write_to_leave` | finds the address in it |
| 13 | `wx_app.rs`, `told(&warning, Priority::High)` | `send_status` and `a11y.announce` |

**All five listings learnt the column, not just the one.** `listing_query`,
`unified_inbox_query`, the saved-search listing, the tag listing and the search's
own query. A message found through the search box and the same message found by
opening its folder must not disagree about whether it came from a list; that is
04-01's "a fact that appears and disappears depending on which door somebody came
in through", one table along.

## Three findings, and two of them are the plan being wrong

### 1. The plan's stated way of reading the header could not have worked

The plan says to read `List-Unsubscribe` through `message.header(name)` and
`header_text`, "the same route `receipt_request` already uses", with the
reasoning that two readers of one header shape are two chances to disagree.

`mail-parser 0.11.5` does not treat the two headers alike.
`parsers/header.rs:60` sends `HeaderName::ListUnsubscribe` to `parse_address`,
so the value arrives parsed as an address list with **the angle brackets
stripped**. `blocking::where_to_write_to_leave` looks for exactly those
brackets:

```rust
.filter_map(|entry| entry.strip_prefix('<')?.strip_suffix('>'))
```

Probed against sixteen spellings of the header before a test was written:

| written on the wire | `header_text` | `header_raw`, scrubbed |
|---|---|---|
| `<mailto:leave@lists.example>` | `mailto:leave@lists.example` | `<mailto:leave@lists.example>` |
| `<https://…/leave>, <mailto:…>` | `https://…/leave, mailto:…` | `<https://…/leave>, <mailto:…>` |
| `<javascript:alert(1)>` | `javascript:alert` | `<javascript:alert(1)>` |
| `mailto:bare@x.example` | `bare@x.example` | `mailto:bare@x.example` |
| `List-Unsubscribe:` | `""` | `""` |

Following the plan would have shipped a feature that compiled, passed its own
tests, and reported **every mailing list on earth** as one that gave no way out,
because no value would ever have a `<` in it. So the value is read from
`header_raw`, which is the header as the sender wrote it, and put through the
same `as_text_and_nothing_else` a sender's attachment description goes through.

Two things follow from that choice and both are written into the code.
`header_raw` does not decode RFC 2047 encoded words, so a header written as one
arrives as the encoded word itself, has no `<` in it, and produces the
no-address sentence. That is a broken sender: encoded words are not legal in a
structured field. And `header_text`'s doc comment, which says its list and
address arms are unreached, stays true, which it would not have been.

### 2. The IMAP fetch does not ask for the header, and the plan never mentions it

`HEADER_FIELDS` (`imap.rs:93`) names the headers a folder listing fetch asks a
server for, one at a time, deliberately, because a whole header block carries
DKIM signatures and a Received chain. `LIST-UNSUBSCRIBE` was not on it.

Without it: the parse is correct, the column is correct, the handler is correct,
and **no message on an IMAP account ever carries the header**. The whole feature
dead on the commonest account type, with 6270 tests green and nothing to look at.
`src/service/protocols/imap.rs` is not in the plan's `files_modified` at all.

It is now on the list and guarded. The break was measured against the whole
library as well as against the census: `--lib service::protocols::imap::` passes
166 tests with the header removed. Nothing in the library can see this hop, which
is why a record coupled to an integration target is the only thing that can.

### 3. The census the plan says cannot be red was red

The plan's second task says, with reasoning, that its census "has no red
available to it", because "the census has to name a construction that only
exists after task 1 has written it", and prescribes a hand-measured break in
place of the missing red, as plans 03-07 and 03-09 did.

The construction has existed for the whole life of the file. What task 1 changes
is one **argument** to it. Written before any implementation, the census failed
at once:

```
thread 'test_the_block_handler_reads_the_message_rather_than_a_constant' panicked at
tests\the_list_warning_reads_the_message.rs:235:5:
at line 25394 of src/presentation/wx_app.rs the block handler hands `None` as the way
out of the list, which does not come from the selected message.
```

So the census went into the red commit with everything else, and task 2 is the
three guard records rather than a green census standing behind a measurement.
Each record still carries a hand measurement, because that is what a record is.

## Verification

Both commits went through the hook. Nothing used `--no-verify` and no
`#[allow(...)]` was added.

| commit | mode | library | release |
|---|---|---|---|
| 112098f | red | 10 named, 10 failed, nothing else | not run |
| a77586a | all | 6270 passed, 0 failed, 1 ignored | clean, 5m03s |
| 15e13b4 | scoped | tree-reading guards only, all green | not run |
| 585b9e9 | scoped | `mail_sync` 135, `pop_sync` 49, tree-reading guards, all green | not run |

The green ran `all` because it bumps `Cargo.toml`, so it was run detached: that
gate is about six minutes and outlasts a foreground cap.

**Every red failed on a value, not on a missing symbol.** The stub was the
shipped behaviour with the field present at every hop: `parse` answered `None`
whatever the header said, `listing_row` answered `None` whatever the row held,
the fetch asked for the same headers it always had, and the handler still wrote
its literal. One failure verbatim, as asked:

```
thread 'service::mime::tests::test_what_a_message_said_about_leaving_the_list_arrives_with_its_brackets_on'
panicked at src\service\mime.rs:839:13:
assertion `left == right` failed: "List-Unsubscribe: <mailto:leave@lists.example>\r\n" did not survive the parse
  left: None
 right: Some("<mailto:leave@lists.example>")
```

**The plan's `<verify>` command does not run.** `cargo test --lib service::mime::
--lib application::blocking::` is refused by cargo: `--lib` cannot be given
twice. Run as two commands. Both pass: 52 and 52.

### A header present and empty, told apart from one absent

The acceptance criterion asks for both sentences. They are:

- **Present and empty** (`List-Unsubscribe:` with nothing after it):
  `MayBlock::YesButFirst`, and it says: "This message came from a mailing list. Blocking
  files it into Junk and the list carries on sending it. The message gives no
  address to unsubscribe at, so look for a link to leave the list at the bottom
  of it."
- **Absent**: `MayBlock::Yes`, and nothing said at all.

`ParsedMessage.list_unsubscribe` is `Some("")` for the first and `None` for the
second, and the column keeps them apart as the empty string against NULL.

**The plan contradicts itself here and the acceptance criterion settles it.** Its
behaviour list says a value that is only whitespace parses into "carrying
nothing", and its next sentence says a header present and empty "must not arrive
looking like a header that was absent". Both cannot hold. `WhatIsAlreadyTrue`'s
own doc comment decides it: "Its presence is what says the message came from a
mailing list." Collapsing an empty value to `None` would lose the warning for
exactly the lists that gave no way out, which is the opposite of this plan's
purpose.

### Both arrival paths, each with its own test

`application::mail_sync::to_incoming` and `application::pop_sync::to_incoming`
both copy it, and each has a test of its own asserting both halves: a message
from a list reaches the row with its way out on, and a message from a person
reaches it with nothing. A test for the absent case alone passes against a
conversion that drops the field always, which is the pair `to_incoming`'s
existing Cc test is written in for the same reason.

**Both were green on arrival and that is a real gap in how this was done, not a
footnote.** The criterion asking for a test per arrival path was noticed while
writing this summary, after the implementation had landed, so no ordering could
have made them red. Worse, the first draft of this summary claimed a test called
`test_a_receipt_arriving_over_pop_reaches_the_row` already covered the POP
writer's shape. No such test exists. It was written from a memory of the code
rather than from a `grep`, and it was caught by running the `grep` before
publishing rather than by anything in the gate. Both tests were then taken red
by hand against their own breaks, which is the whole of the evidence that either
would notice, and both are guard records.

The cross-layer test in `tests/integration_tests.rs` covers what is downstream of
both: whole raw messages through `mime::parse` into `upsert_message`, out through
`get_message_list_sorted` and `MessageItem::from_row`, and into `may_block`. The
upgrade is proved in the same file: a database written and closed, opened again,
and an old row reads back `None` while a row written afterwards keeps its value.
The second half is what stops the test passing against code that stores nothing
at all.

### The five guard records, all measured by hand with `--no-fail-fast`

| record | red list | total |
|---|---|---|
| what a message said about leaving a list survives the parse | the five `service::mime::tests::test_a_way_out_*` and `test_what_a_message_said_about_leaving_the_list_arrives_with_its_brackets_on` | 6265 passed, 5 failed, 1 ignored |
| the header fetch asks a server for the header the warning is built on | `test_the_header_fetch_asks_for_the_header_the_warning_is_built_on` | 7 passed, 1 failed |
| the block handler asks the message rather than a constant | `test_the_block_handler_reads_the_message_rather_than_a_constant` | 7 passed, 1 failed |
| a mailing list found over IMAP reaches the row with its way out on | `application::mail_sync::tests::test_what_a_list_said_about_leaving_it_reaches_the_row_this_sync_stores` | 6271 passed, 1 failed, 1 ignored |
| a mailing list downloaded over POP reaches the row with its way out on | `application::pop_sync::tests::test_what_a_list_said_about_leaving_it_reaches_a_row_downloaded_over_pop` | 6271 passed, 1 failed, 1 ignored |

The second and third are measured against the census, which is what they name as
their `suite`. Both breaks were **also** run against the library, which stayed
green: 166 tests under `service::protocols::imap` and 199 under
`presentation::wx_app`. That is the finding rather than an aside. Nothing in the
library can see either defect, and one of them survived 344 commits for that
reason.

The coupling was proved by hand in both directions:

```
$ scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_app.rs
a_whole_folder_moves_both_bounds
one_sign_in_per_piece_of_work
nothing_leaves_the_outbox_unasked
the_conflict_choice_can_be_heard
nothing_sends_a_flag_change_unasked
the_list_warning_reads_the_message

$ scripts/check.sh --suites-for guards/guards.toml src/service/protocols/imap.rs
the_list_warning_reads_the_message

$ scripts/check.sh --suites-for guards/guards.toml src/application/notes.rs
$
```

### The count check, and the plan's figures

**The plan's guard-record table is right for three files and wrong for one.**
Counted by parsing every record's `tests_last_seen` rather than by grepping for
mentions: `wx_app.rs` 39, `messages.rs` 22, `mod.rs` 11, `blocking.rs` 0,
`ui_types.rs` 0, all as written. `src/service/mime.rs` is **4**, not 1: 04-01
added three records naming it hours before this plan was written. Its test count
is 47, not the 37 the plan gives, for the same reason.

`test_every_guard_record_says_how_many_tests_the_files_it_names_held` named
exactly those four, and `scripts/guards.sh --remeasure` was run on all four once
green. **All four still redden exactly the tests they name**, in both directions,
so nothing went stale there. The counts read 52 now.

**The two arrival-path tests cost ten more records, and two of those were
wrong.** Adding one `#[test]` to `mail_sync.rs` and one to `pop_sync.rs` flagged
eight records and two. Neither problem has anything to do with mailing lists, and
neither would have been found by any check this branch ran on purpose:

- **`a sync writes no attachment for a message nobody has opened` had been
  unmeasurable since 04-01 landed**, hours earlier. Its recorded break writes a
  `CachedAttachment` literal; 04-01 added a `description` field to that struct,
  so the break stopped compiling and the runner reported a build failure rather
  than a finding. That is `CLAUDE.md`'s "unmeasurable rather than stale" case,
  and it reads as a broken tool. The field is on the literal now and the record
  measures again.
- **`a count and the thing it counts agree in number` named sixteen tests for a
  break that reddens seventeen.** The missing one is
  `application::contacts_sync::tests::test_one_edit_to_one_contact_both_books_hold_is_said_once_and_not_twice`,
  in a module nobody working on mailing lists would have filtered for, which is
  the shape `CLAUDE.md` warns about in as many words.

Both were corrected by hand and re-measured, and both now agree with the tree in
both directions. The other eight redden exactly what they say. Ledger entry 100.

**No `#[test]` was added to `wx_app.rs`, `messages.rs` or `mod.rs`.** Counted
before and after: 199 and 199, 179 and 179, 23 and 23. Test counts that did move:
`mime.rs` 47 to 52, `blocking.rs` 50 to 52, `mail_sync.rs` 134 to 135,
`pop_sync.rs` 48 to 49, `integration_tests.rs` 24 to 26, and
`tests/the_list_warning_reads_the_message.rs` from nothing to 8.

The storage round trip went into `tests/integration_tests.rs`, which no record
names and which `CLAUDE.md` already calls the home for cross-layer tests. It
opens a real `MessageCache` over a `tempfile` directory and needs nothing that is
private to the library.

## What was already covered in `blocking.rs`, and what was not

The plan asks this to be reported per behaviour.

| behaviour | already asserted? |
|---|---|
| a message carrying the header gives `YesButFirst` with the mailing-list sentence | **yes**, `test_blocking_a_mailing_list_warns_that_it_keeps_arriving`. No second copy written. |
| a message carrying no header gives `Yes` and says nothing extra | **yes**, `test_an_ordinary_sender_can_just_be_blocked`. No second copy written. |
| a list that gave no way out still warns | **yes**, `test_a_mailing_list_with_no_way_out_still_warns`, though it asserts only the variant and not the sentence. |
| the header cannot soften either refusal (T-04-06) | **no**. Written. |
| only a `mailto:` is ever named out of the header (T-04-07) | **no**. Written. |

Two tests were added to `blocking.rs` and nothing else in it changed. Both were
green on arrival, which is stated rather than hidden: they assert properties the
code already had and that nothing was holding it to.

`test_only_an_address_to_write_to_is_ever_named_out_of_the_header` drives seven
values that name nothing actionable through `may_block`: a web page, a
`javascript:`, a `file:///`, a `data:`, plain prose, `<>` and `<mailto:>`. Each
produces the no-address sentence, and the header's own text never appears in
what is said. The eighth value, a web page followed by a `mailto:`, names the
address and not the web page, so the loop is not passing because nothing is ever
named.

## Untrusted input, and what was decided about it

`List-Unsubscribe` is a stranger's text that becomes a sentence said aloud at
high priority. Guardrail 6 and the plan's threat register.

**Nothing acts on it (T-04-08, and the whole shape of the feature).** Checked by
reading rather than assumed: the value reaches `told()`, which is `send_status`
plus `a11y.announce`, and stops there. Nothing anywhere in `src/` opens a
`mailto:`; the only `mailto:` handling in the tree is in calendar attendee
parsing and in `handover.rs`, neither of which this touches. Nobody is one
keystroke from writing to a stranger. The warning says where to write; what
happens next is the person's.

**Scheme (T-04-07, mitigate).** `where_to_write_to_leave` names only the
angle-bracketed `mailto:` form, so anything else produces the no-address
sentence. That was already true and is now asserted against seven values rather
than assumed.

**Length (T-04-07, mitigate).** Bounded at the parse boundary at 998 characters,
the most RFC 5322 allows on one header line, because
`where_to_write_to_leave` has no bound of its own and hands back whatever sits
between `<mailto:` and `>`. The cut fails closed: an entry sliced in half has
lost its closing bracket, so nothing reads an address out of it. Both directions
are tested, because a bound that cut everything would pass the first test and
lose the feature.

**Control characters and bidi overrides (T-04-07, mitigate).** Taken out at the
parse boundary through the same `as_text_and_nothing_else` 04-01 built, which
also folds a header split across lines into one value. Replaced with spaces
rather than deleted, and here that matters more than it does for a description: a
character hidden inside an address to make it read as a different one leaves a
visible gap instead of closing up into the address it was imitating.

**Ordering (T-04-06, mitigate).** A phishing message can carry the header, and
`YesButFirst` is a weaker answer than `No`. `may_block` asks the two refusals
first, so it cannot buy anything. Asserted now for both refusals.

**T-04-SC does not apply.** No package was added. `mail_parser 0.11.5` is already
a direct dependency and `Message::header_raw` is already on the type. No
package-manager install ran, so the legitimacy gate never came up.

**What was deliberately not done.** `where_to_write_to_leave` names whatever
sits between `<mailto:` and `>` without asking whether it is an address, so a
sender can put a web address there and have the warning say "unsubscribe by
writing to https://…". Validating it was considered and rejected on the merits
rather than deferred: validation would only reject malformed junk, because the
real threat is a well-formed address belonging to somebody else, which no
validation can tell from a real one. Ledger entry 98.

## Premises that were wrong

Beyond the three findings above.

### 1. `cargo test --lib A --lib B` is not a command

The plan's `<verify>` block. Cargo refuses a repeated `--lib`. Run as two.

### 2. The IMAP path has three hops, not two

The plan's behaviour list says "both arrival paths, IMAP and POP" and its
`files_modified` names `mail_sync.rs` and `pop_sync.rs`. The IMAP path goes
`ParsedMessage` → `ImapMessage` → `IncomingMessage`, and the middle hop is in
`src/service/protocols/imap.rs`, which the plan does not list. Neither is
`outlook_data_file.rs`, `filing.rs`, `outbox.rs`, `searching.rs`,
`saved_searches.rs` or `tags.rs`, all of which had to learn the field.

### 3. The behaviour list contradicts its own next sentence and the acceptance criterion

Covered above. Whitespace-only is a header present, not a header absent.

### 4. `src/service/mime.rs` is fingerprinted by four records and holds 47 tests

The plan says one and 37. Both figures were true before 04-01, which landed the
same day.

## Deviations

**`LIST-UNSUBSCRIBE` on the IMAP header fetch.** Not in the plan. Without it the
feature is dead on IMAP. Rule 3.

**`filing::a_row_filed_here` carries the value.** Not asked for. The function
sits one line below `receipt_to: None`, which is dropped there on purpose and for
a good reason, and the two look alike. They are not alike: nothing acts on this
one, so a message read out of an archive can still warn somebody years later.
Written with a comment saying which decision is which, because a field set to
nothing beside a field deliberately set to nothing is exactly the shape 04-01
found in `content_id`.

**Two tests added to `blocking.rs`.** The plan says not to change the file; both
threat-register entries say to assert rather than assume, and the acceptance
criterion allows tests. Nothing else in it moved.

**The census covers both ends of the route, not only the handler.** Task 2's
plan describes a census over `wx_app.rs`. It also reads `HEADER_FIELDS`, because
the fetch is the other place the feature can die silently and a `#[test]` in
`imap.rs` would flag 34 guard records for re-measurement.

**One version bump, not two.** 0.60.0 to 0.61.0. 04-01 took two for two tasks on
the conservative reading; this is one feature reaching one surface, and the
second commit changes no behaviour.

## What this cannot see

**Nothing here has met a real account or a screen reader**, and the two things
that matter most about this feature are on the other side of both. Ledger entries
94 to 100.

- **Nobody has heard it.** Whether the warning lands *before* the block rather
  than reading as a report of one already made, and whether an email address said
  aloud in the middle of a sentence is understood at speed, are judgements. The
  sentence is the same one that has been in the tree since blocking was built and
  nobody has ever heard it either. Entry 94.
- **Whether real lists write the header in the shape this reads** is unmeasured.
  The probe settles what `mail-parser` does with a given header and says nothing
  about what senders send. If real lists commonly write something with no
  angle-bracketed `mailto:` in it, the warning fires and always says to look for
  a link, which is a weaker feature than this reads as. The changelog says so.
  Entry 95.
- **The census reads source.** It says where a value is written, not whether the
  line is reached, not what a server really sends back, and nothing about the
  sentence. The first of those is answered by the reachability list above; the
  other two are not answered here at all.
- **A message imported from an Outlook data file gets no warning.** The importer
  rebuilds a message from the pieces a PST holds and the transport headers are
  not among them. Entry 97.

**Owed after the merge:** `scripts/guards.sh --touched-by 976f16c`. This branch
changes `wx_app.rs`, `messages.rs`, `mod.rs`, `mime.rs`, `imap.rs`,
`mail_sync.rs` and `pop_sync.rs`, which 39, 22, 11, 4, 34, 8 and 2 records
fingerprint. That is an overnight job and must not block the merge; per
`CLAUDE.md`'s decision of 2026-09-03 the sweep belongs once per completed phase.

Sixteen of those were already measured here, because the count check printed the
scoped remedy and running it when a commit prints it is not optional: the four
`mime.rs` records, the ten flagged by the arrival-path tests, and the two of
those that turned out wrong, measured again after correction.

## Known stubs

None. Every field added is filled by a non-test path except at four sites where
the value is genuinely not in hand, and each of those carries a comment saying
which: a queued outgoing message and a draft, both written by the person using
the program; a conversation part built from a `ThreadNode`, which carries neither
this nor the receipt request; and an Outlook import, which is ledger entry 97.

## Requirements and criteria

**This closes no phase 4 success criterion**, as the plan says. Criterion 6 is
about the safety verdict, not mailing lists. READ-03 is the requirement this is
filed under because it is the one about junk and blocking, and READ-03's own text
does not mention `List-Unsubscribe`.

**It rests on research decision 5 being answered yes**, which Pratik has not
confirmed. If the answer is no, all three commits lift out and nothing else in
the phase depends on them.

What it does close is a complete unreachable feature: `MayBlock::YesButFirst` has
now been returned by a shipped build, which it never had been.

## Self-Check: PASSED

- All four commits are in `git log`: 112098f, a77586a, 15e13b4, 585b9e9.
- `.planning/phases/04-writing-and-reading-a-message-in-full/04-02-SUMMARY.md`
  exists.
- `tests/the_list_warning_reads_the_message.rs` exists and holds 8 tests.
- `guards/guards.toml` holds five records added by this plan and two corrected
  by it, and its sweep count reads 192 plus 413, which
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  checks on every commit. 605 records.
- `.planning/WINDOWS.md` holds entries 94 to 100, written into this worktree. The
  shared checkout's copy is byte-identical to what it was before this plan ran,
  checked with `md5sum` before and after.
- Nothing was merged and nothing was pushed.

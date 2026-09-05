---
phase: 04-writing-and-reading-a-message-in-full
plan: 01
executed: 2026-09-05
status: complete
tasks: 2
requirements: [READ-01]
subsystem: service, data, presentation
tags: [content-description, alt-text, attachments, accessibility, read-01, guard-record]
commits:
  - 558c348 test(04-01) failing tests for a description the sender gave and nobody hears
  - 121c439 An attachment says what the sender said it is, or says they said nothing
  - 889d18f test(04-01) failing tests for an image described only in the markup
  - e103d2d A picture described only in the message takes that description
merged: not merged, and not pushed
branch: phase-04-01-attachment-descriptions
key-files:
  created: []
  modified:
    - src/service/mime.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/attachment_content.rs
    - src/data/message_cache/messages.rs
    - src/presentation/ui_types.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_reader.rs
    - src/presentation/wx_app.rs
    - src/application/long_text.rs
    - src/application/pop_sync.rs
    - src/application/export_tree.rs
    - src/presentation/read_aloud.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
requires: []
provides:
  - "WhatTheSenderSaid: three states for a sender's description, silence apart from unreadable"
  - "Content-Description and Content-ID carried on AttachmentInfo"
  - "an additive description column on the attachments table, with both readers reading it"
  - "AttachmentWithContent::from_a_parsed_part, the layer conversion as a tested function"
  - "an attachment row that says the sender's words, or says plainly there are none"
  - "an image with no header description taking the alt on the img that names it"
affects:
  - "04-03, the preview: criterion 4 now has something true to say about an image"
decisions:
  - "Three states rather than Option<String>, so a broken sender is not reported as a silent one"
  - "The header wins over a borrowed alt, including when the header is unreadable"
  - "The description is scrubbed at the parse boundary and bounded at the label"
metrics:
  duration: about 4 hours
  library_tests_before: 6247
  library_tests_after: 6263
actuals:
  tokens: 88000
  tasks: 2
  commits: 4
---

# Phase 4 Plan 01: An attachment says what the sender said it is Summary

**One-liner:** `Content-Description` and an `img`'s `alt` now reach the sentence
a screen reader speaks over an attachment row, and a row the sender described
with nothing says so in words instead of stopping after the size.

## What works

An attachment row's announcement used to be three clauses: the name, the kind
and the size. It is four now, and the fourth is always there.

- The sender wrote a description: `Figures.pdf, PDF document, 240 KB, Quarterly
  figures for the board`
- The sender wrote nothing: `Figures.pdf, PDF document, 240 KB, no description`
- The sender wrote something that arrived as bytes that are not writing:
  `Figures.pdf, PDF document, 240 KB, a description with nothing readable in it`

An image part with no `Content-Description` of its own takes the `alt` on the
`img` that names it in a `cid:` address, which is the second place criterion 4
says a description lives and in practice the commoner one.

**`no description` is one constant shared with `application::long_text`**, which
already says "image with no description" about an undescribed picture inside a
note. `long_text.rs` composes the same constant into all three of its sentences,
so the two surfaces cannot drift into two phrasings for one fact. Both are
quoted here as asked: `long_text` renders `format!("image with {NO_DESCRIPTION}")`
and the attachment row renders `crate::application::long_text::NO_DESCRIPTION`
as its last clause.

The word-boundary cut moved out of `wx_reader::tab_label` into
`reader_text::cut_at_a_word` for the same reason: one rule with two limits, 40
for a tab's subject and 100 for a description, rather than two rules that could
come to different ideas about what a word boundary is.

## The route, non-test function at every hop

Asked for as a list rather than an assertion, because a fact dropped at any one
of them is a fact the reader never gets.

| # | Function | What it carries |
|---|---|---|
| 1 | `service::mime::described` | reads `Content-Description` and `Content-ID` off the part |
| 2 | `service::mime::borrow_descriptions_from_the_markup` | fills an undescribed picture from the `alt` naming it |
| 3 | `presentation::wx_app::spawn_body_fetch` | the one production writer of attachment rows |
| 4 | `AttachmentWithContent::from_a_parsed_part` | the layer conversion |
| 5 | `MessageCache::replace_attachments_with_content` | replaces the list |
| 6 | `MessageCache::save_attachment_row` | the INSERT, now carrying `description` |
| 7 | `MessageCache::get_attachments_for_message` | the SELECT |
| 8 | `presentation::wx_app::attachments_of` | `CachedAttachment` to `AttachmentItem` |
| 9 | `presentation::wx_app::conversation_parts` | onto the `MessageItem` |
| 10 | `presentation::reader_text::attachments_of` | to `ReaderAttachment` |
| 11 | `ReaderAttachment::label` | the sentence |
| 12 | `wx_reader.rs`, `list.append(&attachment.label())` | the row it is said on |

The second reader of the `attachments` table, `attachments_with_content`, learnt
the column too. Only one of the two is on the reading path, but a column one
knows about and the other does not is a fact that appears and disappears
depending on which door somebody came in through.

## The defect this found

**`content_id: None` was written over a content id the parse had in hand**, at
the one production site that records attachments, for the whole life of that
code. The `attachments` table has had a `content_id` column since it was
created and nothing has ever filled it.

It survived because the conversion was a struct literal spelled out inline in
`wx_app.rs`. A field set to a null value compiles exactly like a field set
correctly, the literal cannot be reached from a test without standing up the
whole surrounding function, and `wx_app.rs` is a file 39 guard records
fingerprint, so it is the last file anybody adds a test to.

The conversion is now `AttachmentWithContent::from_a_parsed_part`, five lines
next to the type it builds, in a file no guard record names, with a test
asserting every field. The content id is carried.

## Verification

Both reds accepted by `scripts/red-commit.sh`. Both greens went through
`scripts/check.sh all`, because each bumps `Cargo.toml` and `which-checks.sh`
answers `all` for that; both were run detached, since that gate runs the release
build and outlasts a ten-minute foreground cap. Nothing used `--no-verify` and
no `#[allow(...)]` was added.

| commit | mode | library | release |
|---|---|---|---|
| 558c348 | red | 11 named, 11 failed, nothing else | not run |
| 121c439 | all | 6255 passed, 0 failed, 1 ignored | clean |
| 889d18f | red | 4 named, 4 failed, nothing else | not run |
| e103d2d | all | 6263 passed, 0 failed, 1 ignored | clean |

**Every red failed on a value, not on a missing symbol.** The stub was the
shipped behaviour with the fields present: `described` answered `Nothing`
whatever the part said, both readers of the table read `Nothing`, and `label`
did not mention a description. One failure verbatim, as asked:

```
thread 'data::message_cache::attachment_content::tests::
test_what_the_sender_said_survives_being_written_down_and_read_back'
panicked at src\data\message_cache\attachment_content.rs:792:9:
assertion `left == right` failed: attachments_with_content lost what the sender said
  left: [Nothing, Nothing, Nothing]
 right: [InWords("Quarterly figures"), Nothing, SomethingUnreadable]
```

**The stored round trip is against a real database**, opened by
`MessageCache::new` over a `TempHome` directory, not against an in-memory
struct. It writes all three states through
`replace_attachments_with_content` and reads them back through both readers.

**The upgrade is proved in the same test that proves the write.** A database
created with the pre-change `attachments` table, with a row in it, is opened by
`MessageCache::new`, the old row reads back as `Nothing`, and a row written
afterwards into the same database keeps its description. The second half is what
stops the test passing against code that never stores a description at all.

**No `#[test]` was added to `messages.rs`, `mod.rs` or `mail_sync.rs`.** Counted
before and after: `messages.rs` 179 and 179, `mod.rs` 23 and 23, `mail_sync.rs`
134 and 134. Test counts that did move: `mime.rs` 37 to 47, `reader_text.rs` 76
to 81, `attachment_content.rs` 19 to 22.

**The plan's guard-record figures were right**, which is the first time in two
phases. Parsed by reading every record's `tests_last_seen` entries rather than by
grepping for mentions: 1 record fingerprints `mime.rs`, 0 fingerprint
`reader_text.rs`, `attachment_content.rs` and `ui_types.rs`, 22 fingerprint
`messages.rs`, 11 `mod.rs`, 8 `mail_sync.rs`. Every number matched the plan.

### Four guard records, all measured by hand first

Each break was applied by hand and the whole library run with `--no-fail-fast`
before anything was written down. The full red list of each, and the passing
total it was measured against:

| record | red list | total |
|---|---|---|
| a description the sender gave survives the boundary | `service::mime::tests::test_what_a_sender_writes_in_content_description_becomes` (and, after task 2, `test_the_description_the_sender_wrote_outranks_the_one_on_the_markup`) | 6254 passed, 1 failed, then 6262 passed, 1 failed |
| an attachment nobody described says so rather than trailing off | `presentation::reader_text::tests::test_an_attachment_reads_as_a_name_a_kind_a_size_and_what_the_sender_said`, `presentation::reader_text::tests::test_an_attachment_row_the_sender_described_with_nothing_says_so_in_words` | 6253 passed, 2 failed |
| the sender's own description of a picture outranks a borrowed one | `service::mime::tests::test_the_description_the_sender_wrote_outranks_the_one_on_the_markup` | 6262 passed, 1 failed |
| a borrowed description comes from the picture that names that part | `service::mime::tests::test_a_part_no_picture_in_the_markup_names_stays_undescribed`, `service::mime::tests::test_markup_too_broken_to_read_leaves_the_part_undescribed_rather_than_guessed_at` | 6261 passed, 2 failed |

All four were then put through `scripts/guards.sh --remeasure` and agree with
what they name, in both directions.

**The boundary record went stale inside the session that wrote it.** Written in
the morning naming one test, it named too few by the afternoon: the alt lookup
task 2 added gives a part whose header is dropped somewhere else to fall through
to, so the same break now reddens the precedence test as well. Nothing failed
while it was wrong. It was caught only because task 2 re-ran `--remeasure`
rather than trusting the morning's measurement, which is exactly the shape
`CLAUDE.md` says never announces itself. Corrected by hand, re-measured, and the
note on the record says what happened.

`scripts/guards.sh --remeasure "a name written back out is quoted when it needs
to be"` was run twice, once per task, because both added tests to `mime.rs`.
Its red list is still exactly the 20 tests it names.

## What was walked by hand, and what arrived green

The plan asks for this per test, and the answer is not uniform.

**Task 1: all ten new tests failed at RED, each on a value.** Two of the seven
rows inside `test_what_a_sender_writes_in_content_description_becomes` could not
have failed against the `Nothing` stub, since the stub answers `Nothing` and
those rows expect it. They discriminate against the finished code, which was
confirmed by the boundary break reddening the whole test.

**Task 2: four of the seven arrived green and the red commit said so.** Each was
then taken red by hand against the finished code:

| test | break that reddens it | result |
|---|---|---|
| `test_the_description_the_sender_wrote_outranks_the_one_on_the_markup` | let the lookup overwrite a header | red, and it is a guard record |
| `test_a_part_no_picture_in_the_markup_names_stays_undescribed` | take the first alt in the body | red, and it is a guard record |
| `test_a_picture_the_markup_describes_with_nothing_stays_undescribed` | assign the alt directly instead of reading it | red |
| `test_a_message_with_no_markup_at_all_still_parses_and_borrows_nothing` | none found | **no break reddens it** |

The last is reported rather than claimed. It asserts a property that falls out
of the `let Some(html) = html else { return }`, and every edit tried leaves it
green. It is a statement of intent, not a guard.

**Every task 2 fixture goes through `mime::parse` from a whole raw message.**
Not through a helper handed the markup and the parts already separated: what can
go wrong is the ordering against `carry_the_pictures`, and a test that skipped
`parse` could not see it. The first test asserts that the rewrite really
happened (`body_html` holds `data:image/jpeg;base64,` and no `cid:`), so it
cannot pass by the pictures never having been carried at all.

**The part every fixture attaches was checked against `attachment_parts`.**
`is_embedded_in_the_body` filters out a part that has a content id, is marked
inline, and has no filename. Every picture fixture is named, so it survives, and
the helper asserts the list holds one part before anything else is read. A
fixture built the other way would assert about a part that is not there.

## Three findings, none from reading

**One: a test committed red that could never have gone green.** The malformed
markup test asserted that the alt was recovered from a body whose first `src` is
never closed. Applying the finished code and printing what it produced showed
html5ever welds the two `img` elements into one and returns a picture whose
content id is `pic alt=unclosed <img src=`. No part has that id, so the part
correctly keeps its silence. The red was honest, and it was red for a reason
that would never become true.

That is the better assertion, and the test makes it now: a body too broken to
read leaves the part undescribed **rather than described with the wreckage**,
with a sibling covering a body that is malformed and still readable. The
rewritten test is the one the id-match guard record names, so it is now doing
real work.

**Two: the obvious fixture for the trim could not have tested it.** Probed
against `mail_parser 0.11.5` before any test was written: a header written
`Content-Description:` followed by spaces is trimmed away by the library and
arrives as `None`, so a fixture in that spelling asserts the right answer for
the wrong reason. The same value written as an RFC 2047 encoded word,
`=?UTF-8?B?ICAg?=`, arrives untrimmed. Three rows of the table exist only
because the probe said so, and one intended row was discarded as untestable.

The same probe settled what needs scrubbing at all. Control characters and CRLF
reach this code untouched: `=?UTF-8?B?QQdCCUM=?=` arrives as `A\u{7}B\tC` and
`=?UTF-8?B?bGluZTENCmxpbmUy?=` as `line1\r\nline2`. Both would go into a
one-line list item and to a screen reader. And `=?UTF-8?B?BwcH?=` arrives as
three bell characters and nothing else, which is the case the third state
exists for and is reachable rather than theoretical.

**Three: `Content-ID` arrives with the angle brackets already stripped and the
case kept.** `<PIC>` comes back as `PIC`. So normalising through
`pictures::plain_content_id` is load-bearing for the case and not for the
brackets, and the test asserts all three spellings.

## Untrusted input, and what was decided about it

A description is a stranger's text on its way to being spoken aloud in a room
and shown in a list. Guardrail 6 and the plan's threat register.

**Length (T-04-02, mitigate).** Bounded at the label, not at the parse, so the
full text is stored for a surface with room to show it. 100 characters, cut at a
word with an ellipsis, through the same function that cuts a tab's subject. A
judgement rather than a measurement, and the code says so.

**Control characters and markup (T-04-03, mitigate).** Taken out at the parse
boundary, in one place, for both the header and the borrowed alt. Control
characters and the bidirectional overrides `\u{202A}`..`\u{202E}` and
`\u{2066}`..`\u{2069}` become spaces, runs of whitespace collapse, and the
result is trimmed. The character class is the one
`application::export_tree::carries_anything_a_name_can_keep` already refuses in
a file name, rather than a second rule. Replaced with spaces rather than
deleted, so the words either side stay two words. Asserted with a fixture whose
`alt` carries a quote and angle brackets: `She said "look" <b>now</b>` arrives
as those characters.

**Malformed body (T-04-04, mitigate).** The lookup uses `scraper`, which
recovers rather than failing, and every failure path in
`what_each_picture_is_called` is a `continue` or an early return. A message that
will not parse is a message nobody can read at all, which is a worse failure
than a missing description. Two tests, one for a body that is malformed and
readable and one for a body that is not.

**Silence and unusable are not the same outcome.** `WhatTheSenderSaid` has three
states, not two. NULL in the column is silence; the empty string is a
description that arrived as bytes that are not writing; anything else is what
the sender wrote. No readable description can be mistaken for the middle state,
because `read` never builds an empty `InWords`, so the three states reach a TEXT
column with no magic word a sender could write for themselves.

The row says which: `no description` against `a description with nothing
readable in it`. Guardrail 9, because the second names a fault in the sender's
program rather than reporting it as the sender having been silent.

**Whitespace is silence.** The plan's rule, kept: a sender pressing space is
saying nothing, not saying a space. An `alt=""` is the author marking a picture
decorative, which is also nothing to say.

**T-04-SC does not apply.** No package was added. `mail_parser 0.11.5` and
`scraper 0.27.0` are both already direct dependencies and `MimeHeaders` was
already imported in `mime.rs`. No package-manager install ran, so the
legitimacy gate never came up.

## Premises that were wrong

### 1. There is one writer of attachment rows, not two, and it is not in either sync

The plan's behaviour list says "a message stored through the IMAP path and a
message stored through the POP path both carry the description, because those
are two writers of the same row", and an acceptance criterion asks for a test
per writer. Neither writer exists.

`grep` for callers of `replace_attachments_with_content` and `save_attachment`
outside the data layer returns exactly one production site:
`wx_app.rs::spawn_body_fetch`, which runs when a message body is fetched to be
read. `mail_sync.rs` writes no attachment rows at all and has a test saying so
by name, `test_a_sync_over_a_message_with_an_attachment_stores_no_attachment_and_no_file`.
`pop_sync.rs` sets `has_attachments` on the message row and stores nothing else.

Followed literally, this plan would have added two tests neither of which could
have been written. The one writer is proved instead, at the conversion, which is
where the risk actually lives.

### 2. The consequence: a POP account gets none of this

`spawn_body_fetch` returns early when the account has no IMAP server, so a POP
account records no attachment rows at all. A POP message says it carries an
attachment and lists none. That predates this plan and is not made worse by it.
It is not fixed here either: a writer in the POP sync is a new path through the
cache rather than a widening of this one. Ledger entry 92.

### 3. Seven struct literals, measured as two

Premise correction 2 says there are seven `AttachmentInfo` literals spelling all
three fields, and tells the reader to count with the compiler. Counted: two, one
in `mime.rs` and one in `pop_sync.rs`. Twenty more literals of the four
downstream types needed the new field, nineteen of them in tests.

### 4. `is_embedded_in_the_body` does read the disposition

Premise correction 1 is right and the research is wrong, confirmed by reading:
the function requires `content_disposition()` to say `inline` before it filters
a part out. Nothing was built on the research's version.

### 5. `long_text.rs` was not in `files_modified`

Sharing the `no description` wording means composing it from one constant, which
means editing the three places in `long_text.rs` that spelled it out. No
`#[test]` was added there, so the 18 records that fingerprint it were not
flagged.

## Deviations

**`content_id` is now filled.** Not asked for by the plan, which only asks that
`AttachmentInfo` carry it. Leaving `from_a_parsed_part` writing `None` over a
value it holds would be the same defect the extraction exists to expose, one
line away from the code that exposes it. The column already existed. Nothing
reads it yet.

**The conversion was extracted from `wx_app.rs`.** The plan implies the literal
stays where it is. It cannot be tested there, and the acceptance criterion asks
for the writer to be proved.

**`cut_at_a_word` moved out of `wx_reader::tab_label`.** Not in the plan. A
second truncation rule with its own idea of a word boundary is the "second
phrasing for the same fact" the plan warns about, one level down.

**Two version bumps, not one.** Each task changes what a person hears, and
`CLAUDE.md` asks for the bump in the same commit as the change. 0.58.0 to 0.59.0
to 0.60.0. Arguably one feature and one bump would do; the conservative reading
was taken and costs nothing in a 0.x line.

## What this cannot see

**Nothing here has met a real account, and three of the questions this feature
raises can only be answered by one.** Ledger entries 89 to 93.

- **Whether the description is heard at the right moment.** It is the last
  clause, on the reasoning that the first three are what somebody decides to
  open a file on. That is a judgement about what a person wants to hear first,
  not a measurement, and the row is announced every time focus reaches it, so
  being wrong costs a moment on every arrow press. Entry 89.
- **Whether a borrowed description reads as the sender's words or as something
  the program made up.** The row says the alt text with nothing marking it as
  borrowed, on the grounds that the alt is the sender's writing about that
  picture as much as a header would be. Nobody has heard it. Entry 90.
- **Whether real senders supply `Content-Description` at all.** If they mostly
  do not, the header route mostly says "no description" and the markup route is
  the whole of the feature. Unmeasurable in this repository. The changelog says
  so under Known limitations rather than implying the feature does more.
  Entry 91.

**Owed after the merge:** `scripts/guards.sh --touched-by 9c4dd39`. This branch
changes `messages.rs`, `mod.rs`, `mail_sync.rs` is untouched, `wx_app.rs` and
`long_text.rs`, which 22, 11, 39 and 18 records fingerprint. That is an overnight
job and must not block the merge. Per `CLAUDE.md`'s decision of 2026-09-03 the
sweep belongs once per completed phase, not per merge.

## Known stubs

None. Every field added is filled by a non-test path, and the reachability list
above names the function at each hop.

## Requirements and criteria

**READ-01 is not closed**, as the plan says. The half of criterion 4 that says
"announcing any description the sender supplied and saying plainly when there is
none" is met for the attachment list. The other half, that an image or a text
attachment previews at all, is 04-03's.

## Self-Check: PASSED

- All four commits are in `git log`: 558c348, 121c439, 889d18f, e103d2d.
- `.planning/phases/04-writing-and-reading-a-message-in-full/04-01-SUMMARY.md`
  exists.
- `guards/guards.toml` holds four records added by this plan, and its sweep
  count reads 192 plus 408, which
  `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
  checks on every commit.
- `.planning/WINDOWS.md` holds entries 89 to 93, written into this worktree.
  The shared checkout's copy is byte-identical to what it was before this plan
  ran, checked with `md5sum` before and after.
- Nothing was merged and nothing was pushed.

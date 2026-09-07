---
phase: 04-writing-and-reading-a-message-in-full
plan: 04
subsystem: reading an attachment
tags: [attachments, reader, accessibility, images, untrusted-input, dependencies]
status: complete
requires:
  - "04-01: ReaderAttachment.description, which is the whole of what an image preview can say about a picture"
  - "service::pdf, as the shape a pure producer takes"
  - "application::pictures::KINDS_WORTH_CARRYING, for which kinds count as a picture"
provides:
  - "service::plain_text::read: bytes to text, with a note saying how much of the file is really there"
  - "service::picture::read: bytes to RGBA, bounded three ways, failing closed"
  - "reader_text::text_document, reader_text::image_document"
  - "reader_text::ReaderPicture and ReaderDocument.picture"
  - "ReaderAttachment::how_it_reads, the one answer the gate and the routing both read"
  - "THE_PICTURE_IS_SHOWN, THE_PICTURE_COULD_NOT_BE_READ, NO_PICTURE_TO_SHOW"
affects:
  - "wx_reader::can_be_read_here, now one line over how_it_reads"
  - "wx_app::read_attachment, now routed through document_of"
  - "ReaderWindow::open, which draws a bitmap between the text and the attachment list"
  - "tests/theme_reach.rs, whose fixture now builds a picture"
tech-stack:
  added:
    - "image 0.25 gains its jpeg feature, bringing zune-core 0.5.1 and zune-jpeg 0.5.14"
  patterns:
    - "a pure producer per attachment kind, bytes in, a reading and a note out"
    - "the note before the content, because it changes how the content should be taken"
    - "three states kept apart in words and asserted pairwise different"
key-files:
  created:
    - src/service/plain_text.rs
    - src/service/picture.rs
  modified:
    - src/service/mod.rs
    - src/presentation/reader_text.rs
    - src/presentation/wx_reader.rs
    - src/presentation/wx_app.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - Cargo.lock
decisions:
  - "JPEG only, not GIF and WebP: two new crates rather than five, and GIF's LZW and WebP's VP8 are the two largest parsers a stranger's attachment would newly reach"
  - "Three bounds on a decode rather than two: the declared size, the decoder's own allocation, and what is really handed to the window, because a grey picture quadruples on conversion"
  - "The declared-size gate deliberately does not ask what a picture would cost, so the allocation cap has something to catch"
  - "One megabyte of text, not the attachment store's twenty-five: a different question"
  - "One function answers which reading an attachment gets, so the gate and the routing cannot disagree"
metrics:
  duration: about five hours
  completed: 2026-09-05
  commits: 6
  version: 0.62.0 to 0.65.0
actuals:
  tokens: 62000
  tasks: 3
  commits: 6
---

# Phase 04 Plan 04: A Text Attachment and an Image Attachment Preview, Summary

A text attachment opens as text with a note saying how much of the file is
really there, and an image attachment opens described **and** drawn, refusing out
loud rather than blankly when it cannot be decoded.

## What works, plainly

Choose a `.txt`, `.log`, `.md` or `.csv` in the reader's attachment list and
press Enter, and it opens in a tab as text somebody can move through by line. A
file that is not text is refused by name through the refusal that already
existed. A file over a megabyte is cut and the note says so before a word of it.

Choose a PNG, JPEG, GIF or WebP and it opens in a tab whose first line is what
the sender said the picture is, or a sentence saying they said nothing. A PNG or
a JPEG is then drawn below those words, with the sender's description as its
accessible name. A GIF or a WebP is described and says plainly that this build
draws none of that kind, which is a different sentence from the one a damaged
file gets.

The whole path is reached from a non-test caller and no hop on it is test-only.
It has never been run against a real mail account, and no screen reader has met
the picture. Both are recorded in `.planning/WINDOWS.md` rather than implied
away.

## Reachability, hop by hop

An attachment row in the reader list, Enter (or Ctrl+R, or the Read Attachment
menu item), then:

| Hop | Function |
|---|---|
| the key handler | `wx_reader::save_chosen` with `Doing::Reading` |
| the gate | `wx_reader::can_be_read_here` |
| the handler the application supplied | `ReaderWindow::on_read_attachment` |
| the worker | `wx_app::read_attachment`, `fetch_attachment_bytes` |
| the routing | `wx_app::document_of` |
| the producer | `plain_text::read` + `text_document`, or `image_document` + `picture::read`, or `pdf::read` + `pdf_document` |
| back to the window | `UIUpdate::AttachmentRead` |
| the tab | `ReaderWindow::open`, then `Bitmap::from_rgba` and `StaticBitmap` for a picture |

The tab is now, in order: the warning bar when there is one, the text, the
picture when there is one, the attachment list.

## The three tasks

### Task 1: a text attachment opens as text

`service::plain_text` is a sibling of `pdf.rs`: pure, bytes in, text and a note
out. The note comes first for the reason `pdf_document` already gives.

The "mostly not text" rule is `mostly_not_text` and its reason is beside it:

> A NUL byte settles it alone, because no encoding this reads puts one inside a
> file. Short of that it is the proportion of characters that are not writing,
> judged after decoding so a file that is not UTF-8 at all is caught by its
> replacement characters rather than slipping through for having no control
> bytes in it. A tenth is the line, and it errs towards showing: something shown
> that should not have been is a screenful somebody closes, and something refused
> that should have been shown is a file they cannot read at all with no way to
> find out why.

The bound is one megabyte and **not** the attachment store's twenty-five, which
the plan asked to be justified if it differed:

> That one answers whether a file is worth keeping on disk, and this one answers
> how much of it a person can be handed at once. Twenty-five megabytes of
> characters in a read-only control is a window that stops answering while it is
> filled.

`looks_unsafe` for a text document, its own reason rather than `pdf_document`'s:

> Nothing here interprets the file. `plain_text::read` already took out the
> control characters that could have made a reading window do something, and what
> is left is characters in a read-only control: there is nothing in it to run, to
> follow or to submit.

**The gate and the routing are one function**, `ReaderAttachment::how_it_reads`.
`can_be_read_here` is now the single line `attachment.how_it_reads().is_some()`,
and `document_of` matches on the same answer. That makes divergence impossible
rather than merely tested for.

**Which assertions passed at their own red**, as the plan asked to be recorded:

- `test_a_text_attachment_carries_no_warning_bar_and_nothing_hanging_off_it`
  passed, because an empty document has no warning, no attachments and
  `looks_unsafe` false. Kept as the regression guard for a decision that is a
  claim rather than a default, and not named in the trailers.
- Inside `test_a_file_over_the_bound_comes_back_cut_and_says_so`, the assertion
  `bytes_read < bytes` passed: a stub that read nothing had already read less
  than the whole file. Only the sentence saying the file was cut discriminated.
- `test_everything_else_is_not_pretended_to_be_readable` passed, correctly: task
  1 widens the gate to text only, so its `photo.jpg` row was still right.

### Task 2: an image attachment says what is known about the picture

The first three lines, side by side as the plan asked:

```
bicycle.jpg
The sender described this picture: A red bicycle against a brick wall
The picture is shown below these words.
```

```
IMG_4021.jpg
This picture came with no description, so nothing here can say what is in it.
The picture is shown below these words.
```

The absent-description wording is `long_text::NO_DESCRIPTION`, which is the
literal string `"no description"`, read from there rather than written again.
`ReaderAttachment::label` composes the same constant, so a reader meets the same
three words on a picture inside a note, on an attachment row, and here.

The description is **not** cut at `LONGEST_DESCRIPTION_SPOKEN` the way a row is,
and the reason is in the code: a row is announced every time focus reaches it, a
tab is opened once on purpose by somebody who wants the whole of it.

The kinds come from `pictures::KINDS_WORTH_CARRYING`. What that list cannot
answer is the file name, so `READS_AS_A_PICTURE` names the kind each extension
stands for, and `test_the_two_halves_of_what_counts_as_a_picture_agree` holds
every entry to a kind that list really carries and every kind to a name that
reaches it. That is the answer to "or say why a second list was needed": it is
not a second list of kinds, it is the name-to-kind half, and the two are bound by
a test in both directions.

`looks_unsafe`, task 2's reason and task 3's, together:

Task 2: *"Nothing here has looked at the file. The bytes are carried past
untouched, no decoder has been handed them ... That answer changes the moment
something decodes, and the task that adds one has to decide this again rather
than inherit it."*

Task 3: *"Decided again now that something parses these bytes ... Still false,
and for a different reason rather than by inheritance. Handing a decoder a
stranger's file is a risk this program chose to take, and it is bounded before
the decode starts, bounded again while it runs, and fails closed: what comes out
is pixels or a refusal, never anything that runs, follows or submits."*

The named state is `NO_PICTURE_TO_SHOW`, a `pub const &'static str`, written as a
constant precisely so task 3 could assert it differs from the two beside it.

### Task 3: the picture is shown

#### The dependency justification

It is in `Cargo.toml` beside the existing `image` comment, seventy lines of it,
and every claim in it was re-checked against `Cargo.toml`, `Cargo.lock` or the
vendored `image-0.25.9` source rather than quoted from the plan. In summary:

| Question | Answer |
|---|---|
| Which crate, already here? | `image`, direct dependency, locked at 0.25.9, already used by `art.rs`. PNG and BMP decode in this binary today. |
| What the widening adds | Read from the feature table: `jpeg = ["dep:zune-core", "dep:zune-jpeg"]`, `gif = ["dep:gif", "dep:color_quant"]`, `webp = ["dep:image-webp"]`. None of the five was in `Cargo.lock` for another reason. |
| Legitimacy | `zune-jpeg` 104M downloads since 2022, `zune-core` 101M since 2023, both owned by `etemesi254`, both the crates `image` itself chose. The three left out are fine too: `gif` 128M, `color_quant` 120M, `image-webp` 70M, all image-rs. |
| The smaller option | **Taken.** JPEG only, two new crates rather than five. PNG already decoded, so this covers the two kinds almost every picture in mail is. GIF's LZW and WebP's VP8 are the largest new parsers and buy no common case. |
| Hand-written | Entropy decoding, dequantisation, an inverse DCT, chroma upsampling, and progressive mode. Not a serious option, said out loud anyway. |
| `wxdragon` loader | Verified absent. `src/bitmap.rs` has `from_rgba`, `new`, `null_bitmap`, `get_rgba_data`. `Bitmap::new_from_file` appears only in a doc-comment example in `widgets/generic_static_bitmap.rs`. |
| `jpeg-decoder` already in the lock | True, 0.3.2, under `pdfpurr`. Using it directly means a second decode path with its own bounds and its own conversion from Rgb24, L8 and Cmyk32, alongside `image` for PNG, to save two crates from the same ecosystem. Not taken. |

`cargo tree -i zune-jpeg` and `cargo tree -i zune-core` both answer that they
arrive through `image` and nothing else.

#### Three bounds, each measured to catch what the others do not

| Bound | Where | Fixture | Break that reddens it, and only it |
|---|---|---|---|
| The declared size | `into_dimensions`, before any decode | a 10001 by 1 PNG | removing `refuse_what_the_header_declares` |
| What decoding allocates | `image::Limits`, `max_alloc` | a 2600 by 2600 **sixteen bit** PNG: eight bytes a pixel to decode, four to hand on | `max_alloc = None` |
| What crosses to the window | after `decode`, before `to_rgba8` | a 4000 by 4000 grey PNG: one byte a pixel to decode, four to hand on | deleting the check |

Each break was applied by hand and each reddened exactly one test.

**Which gate each fixture reaches, and how that was checked.** The first fixture
reaches the declared-size gate: its test asserts the error names `10001`, which
only this project's own sentence does, and removing the gate reddens it because
`image`'s strict width limit then refuses it with a different sentence. The
second reaches the allocation cap alone, which is why it is sixteen bits a
channel: at eight bits both axes and the final buffer are inside the other two
bounds. **The first fixture written for that test was wrong and would have passed
against its own break**, an ordinary 3600 by 3600 eight-bit PNG, which the third
check also catches, so `max_alloc` could have been deleted with everything green.
It was found by applying the break, not by reading the test.

**The third bound is not in the plan.** It is here because the first two do not
cover it: a decoder producing a grey picture allocates one byte a pixel and this
hands on four, so a picture can satisfy the decoder's own bound and still be four
times the size by the time it is cloned into the window's record of every open
tab. Treated as Rule 2, missing critical functionality.

**The declared-size gate deliberately does not ask what a picture would cost**,
although the numbers are in hand, and the reason is in the code: asking there
would refuse everything the allocation cap would have refused, and a cap that
never fires is a cap nothing can show still works.

`image`'s own documentation is what says two of these are both needed, re-read
from `image-0.25.9/src/io/limits.rs`: `max_alloc` "is non-strict by default and
some decoders may ignore it", and the width and height limits are "strict".

#### No unwraps on this path

```
$ grep -rn "unwrap()\|expect(" src/service/picture.rs
187:            .expect("a PNG this test made");
200:            .expect("a PNG this test made");
206:        let read = read(&a_png(40, 25)).expect("a picture this test made");
224:            .expect("a JPEG this test made");
226:        let read = read(&bytes).expect("a JPEG");
238:        let read = read(&a_png(40, 25)).expect("a picture this test made");
283:            .expect("a sixteen bit PNG this test made");
```

Seven hits, all inside `#[cfg(test)] mod tests`, which begins above line 180.

#### The three states, quoted whole

| State | Sentence |
|---|---|
| `THE_PICTURE_IS_SHOWN` | "The picture is shown below these words." |
| `THE_PICTURE_COULD_NOT_BE_READ` | "This picture could not be read, so there is nothing below to look at. The file may be damaged, or it may not be the kind of picture it says it is." |
| `NO_PICTURE_TO_SHOW` | "Wixen Mail does not draw pictures of this kind, so there is none in this tab." |

`test_the_three_things_a_preview_can_say_about_a_picture_are_three_different_things`
asserts they are pairwise different **and** that none is a substring of another.
`test_a_picture_preview_always_says_which_of_three_things_happened` asserts that
a preview says exactly one of them, for a picture that decodes, one that does
not, and a kind this build does not draw. Each state's own case has its own test.

#### The accessible name

```rust
set_accessible_name(&view, &shown.described);
```

Not `set_name`, which writes an internal wxWidgets identifier that never reaches
the accessibility tree. `described` is the sender's sentence, or this program's
wording for an undescribed picture, and is never empty.

#### The theme sweep

`tests/theme_reach.rs`'s fixture now carries a one-pixel `ReaderPicture`, so the
tab really builds a bitmap, `ReaderTabHandles` carries it, and `check_reader`
compares its background and foreground against the palette like every other
optional widget. A new optional tab widget did not quietly escape the sweep.

## Deviations from Plan

### Rule 2, missing critical functionality

**1. A third bound on the decode.** Found while designing task 3's fixtures. The
plan names two gates. Two are not enough: `max_alloc` bounds what the *decoder*
allocates, and what crosses to the window is always four bytes a pixel, so a grey
picture can pass the decoder's bound and be four times the size afterwards. A
check on `width * height * 4` after decoding was added, with its reason, and a
fixture that reaches it and neither of the others. Commit `4ea1d2e`.

### Premises of the plan that turned out wrong

**2. The guard table is wrong in both directions.** The plan says
`src/presentation/reader_text.rs` is fingerprinted by no record and
`tests/theme_reach.rs` by three. Measured by parsing `tests_last_seen`: four
records name `reader_text.rs` (six by the end of this plan) and **none** name
`theme_reach.rs`, which is named only in three comments. `wx_reader.rs` really is
fingerprinted by none, as the plan says, and `wx_app.rs` is 40 rather than 39.
This is the fourth phase-04 plan whose record table expired against a same-day
sibling. Recorded as ledger entry 113.

**3. The task 2 RED commit the plan specifies cannot exist.** It asks the red
commit to name both a new test saying a picture can be read here and the existing
`test_everything_else_is_not_pretended_to_be_readable`, which says it cannot.
Those are opposite assertions about one function, so exactly one is red whichever
way the code stands, and `red-commit.sh` requires every named test to really
fail. The gate was therefore widened at the red rather than at the green, which
makes the existing assertion the red one, the stronger of the two, since it is a
test that had been passing for real reasons. The new gate test passed at its own
red and was measured by hand at the green by removing the picture arm. Said in
the commit and recorded as ledger entry 113.

**4. `Cargo.toml` in tasks 1 and 2.** The plan lists `Cargo.toml` in those tasks
for version bumps only, which is what happened. Worth saying because a commit
touching it makes `which-checks.sh` answer `all`, so four of the six commits ran
the release build. All four ran detached.

### Tests added at the green rather than the red, each measured by hand

Every one of these was verified by deleting the code it covers and watching it
redden, on its own.

| Test | Break |
|---|---|
| `test_a_file_of_bytes_that_are_not_text_and_hold_no_nul_is_still_refused` | the replacement character stops counting as not writing |
| `test_a_long_file_is_not_reported_as_not_entirely_text_for_having_been_cut` | the cut stops backing off to a character boundary |
| `test_an_empty_file_is_a_text_file_with_nothing_in_it` | `mostly_not_text` answers true for empty |
| `test_a_picture_can_be_read_here_whichever_way_it_says_so` | the picture arm removed from `how_it_reads` |
| `test_the_two_halves_of_what_counts_as_a_picture_agree` | one extension names `image/jpg` |
| `test_a_kind_this_build_does_not_draw_says_there_is_none_rather_than_a_fault` | `image/gif` added to `KINDS_DRAWN_HERE` |
| `test_the_three_things_...are_three_different_things` | the two constants made equal |
| `test_the_picture_is_named_from_what_the_sender_said` | `described: String::new()` |
| `test_the_words_come_before_the_picture_in_the_tab` | the preview's lines reordered |

**One of these was written wrong and passed against its own break.** The
boundary test's first fixture was ordinary prose repeated until it was long, and
the cut happened to land between characters, so removing the backing-off changed
nothing. Rewritten to put an accented letter across the bound and to assert that
the byte at the bound really is a continuation byte before asserting anything
else. A fixture that is merely long does not test where a cut lands.

## Guard records

Four added, all measured by hand with `cargo test --all-targets --no-fail-fast`
against a tree with no other failure in it.

| Record | Break | Full red list |
|---|---|---|
| the reading gate admits text and still refuses what it cannot read | `how_it_reads` answers `Text` for everything | `presentation::wx_reader::tests::test_everything_else_is_not_pretended_to_be_readable` |
| a picture nobody described says so rather than leaving the line blank | the `Nothing` arm returns `String::new()` | `presentation::reader_text::picture_preview_tests::test_a_picture_the_sender_described_with_nothing_says_so_before_anything_else`, `presentation::reader_text::picture_shown_tests::test_the_picture_is_named_from_what_the_sender_said` |
| a picture is refused on the size it declares, before anything decodes it | `refuse_what_the_header_declares` removed | `service::picture::tests::test_a_picture_declaring_more_than_the_bound_is_refused_on_its_declaration` |
| a picture that could not be read does not say there was never one | the `Err` arm returns `NO_PICTURE_TO_SHOW` | `presentation::reader_text::picture_shown_tests::test_a_picture_that_could_not_be_read_says_so_and_draws_nothing` |

**The second record fell behind within one commit.** Written at task 2 naming one
test; task 3 added `test_the_picture_is_named_from_what_the_sender_said`, which
composes the picture's accessible name from the same sentence, so emptying it
reddens both. `--remeasure` refused the record and named the extra test, and it
was corrected by hand before being written down again. That is the mechanism
earning its keep rather than a run of luck, and it is the same shape `CLAUDE.md`
describes: a later change adds a test that reaches a rule an existing record is
about, and nothing else would have said so.

**The fourth record's break is at the place that chooses, not at the constant.**
Breaking the constant was tried first and reddens two tests, which is a weaker
record: it would go on passing if somebody left the constants alone and returned
the wrong one of them, which is by far the likelier mistake.

**The first record was re-anchored once.** Its `before` named the two lines the
picture arm later landed between, so `test_every_guard_record_still_names_one_place_in_the_tree`
refused it. Re-anchored and then re-measured rather than edited until it applied.

**Re-measurement.** Six records were re-measured across this plan, in three runs,
because `src/presentation/reader_text.rs` went from 93 test functions to 110.
Every one reddens exactly what it names.

## Known Stubs

None. Every state a preview can be in is reachable, is said in words, and has a
test. GIF and WebP not being drawn is a stated scope, said in `Cargo.toml`, in
the changelog and in the tab itself, not a stub.

## Verification not done here

Six entries added to `.planning/WINDOWS.md`, 108 to 113. The ledger ended at 107.

| id | kind | what only a person or a real account settles |
|---|---|---|
| 108 | unrun-verify | Whether a `StaticBitmap` is reachable by a screen reader at all. It is not focusable by default on Windows, so NVDA may meet it only in browse mode or not at all, and the accessible name may never be spoken. |
| 109 | unrun-verify | Whether the picture is drawn at a sensible size, is legible against either theme, and does not push the attachment list off the tab. Nothing in this repository has ever checked any of that. |
| 110 | unrun-verify | Whether the first lines of a preview are announced when it opens or have to be gone looking for. The whole design rests on the description being heard first. |
| 111 | unrun-verify | Whether real text attachments are UTF-8. If most are not, most previews are replacement characters under an honest sentence, which would argue for encoding detection. |
| 112 | unrun-verify | Whether real senders write `Content-Description` on image parts at all. Unchanged since the research raised it, and now the cost of "no" falls on a blind reader alone. |
| 113 | deviation | The two plan premises above. |

## Threat Flags

None. Every new surface in this plan is in the plan's own threat register:
`plain_text::read` and `picture::read` as new parsers over untrusted input, the
pixels crossing the worker boundary, and the sender's description becoming a
control's accessible name.

T-04-19, the decompression bomb, is mitigated with three bounds rather than the
two the register names, and the third is documented above. T-04-22, a pixel
buffer disagreeing with the width and height handed to the toolkit, is asserted
by `test_a_picture_has_four_bytes_for_every_pixel`, the equivalent of the one
`art.rs` already carries.

## Commits

| Commit | What |
|---|---|
| `4884702` | RED: a text attachment opens as text |
| `1139ce0` | GREEN: a text attachment opens as text |
| `8e4a08c` | RED: a picture says what is known about it |
| `409b2ca` | GREEN: an image attachment says what is known about the picture |
| `4d06a28` | RED: a picture that is shown, and bounded |
| `4ea1d2e` | GREEN: an image attachment is shown, and refuses out loud when it cannot be |

Version 0.62.0 to 0.65.0, one minor per task, each in the commit that made the
change. `docs/changelog.md` carries one entry per task under `[Unreleased]`,
extended rather than replaced.

## What is owed

`scripts/guards.sh --touched-by ceaee7f` after the merge, not before it. Nothing
in this plan blocks on it.

## Self-Check: PASSED

Both new modules exist on disk, this file exists, and all six commit hashes
resolve in this branch. Checked 2026-09-05.

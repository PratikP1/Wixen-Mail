# Ledger 270: a list that keeps its depth, a table that keeps its columns, and a note that keeps its links

Branch `a-nested-list-keeps-its-depth-and-a-table-keeps-its-columns`, 2026-09-11.
Four commits, two red and two green. Version `0.107.0` to `0.109.0`.

## What works now

A screen reader tells you when a list is nested, and reads a table as a table.
This applies everywhere long text is read aloud: a note's body, a contact's
Notes, a task's body, and an event's description, including the descriptions
Google Tasks and Microsoft To Do send back.

A note kept in OneNote keeps its links, its pictures, its line breaks, its
nesting and its tables. Twelve of the twenty-two constructs measured in
`docs/development/the-notes-seam.md` survive the round trip, up from seven.

PIM-04's structure criterion is met for both backends that exist. Its sentence
is untouched; only its status annotation changed.

Nothing here has been heard by a screen reader. That is entry 271.

## What I verified, and what turned out false

The brief asked that every claim in its own finding be checked first. Five of
its claims held. Three facts about the tree did not.

### Held

`Piece::Item` carried `ordered` and `text` and no depth, and `Piece` had no
table variant, so nesting and tables could not be represented at all, in either
direction, for any caller.

`Piece` and `structure` had no consumers outside `long_text.rs`. Checked by
grep over `src/` and `tests/`: the only other hits were `AnArchiveInPieces` in
`message_files.rs`, an unrelated name.

`long_text::spoken` had five production callers in `read_aloud.rs`, at lines
370, 389, 415, 442 and 516. Two further callers, in `notes.rs` and
`calendar.rs`, are inside `#[cfg(test)]` blocks, which the brief did not claim
otherwise.

`from_markup`'s production callers outside notes were `calendar.rs:3037` and
`tasks_api.rs:542`.

The whole of the brief's central claim: this was a shipped accessibility defect
on the read-aloud path, not a notes problem.

### Turned out false

**The brief says `long_text.rs` holds 76 test functions. It held 38.** Both
`grep -c '#\[test\]'` and the `tests_last_seen` blocks of all eighteen guard
records agree on 38. The re-measurement budget the brief derived from that
number was right for the wrong reason: eighteen records at roughly ninety
seconds each is about half an hour whatever the test count is.

**The brief and the seam document both say a table came back as "one paragraph
per cell". That describes `from_markup` only.** A table typed as Markdown and
put through `structure` came back as a single paragraph with every cell
concatenated and no separator at all: `| Name | Role |` with two rows produced
the one word `NameRoleGraceAdmiralAlanFellow`. Nobody had ever put a typed table
through `structure`; the recorded figure came from reading `from_markup`'s
output, where the cells really do land one to a paragraph. The defect was worse
than written down.

**An HTML nested list was worse than flattened: it corrupted words.**
`push_item` called `inline`, which walked the nested `<ul>` as if it were inline
content, so `<li>Live is brown<ul><li>Older cable: red</li></ul></li>` came back
as `- Live is brownOlder cable: red`. Two words became one word that was in
neither of them. The seam document recorded the swallowing; it did not record
that the result was a word nobody wrote.

**Two guard records had already stopped guarding anything on `main` at
`a343d7f`, before this branch existed.** "inline reads a bare line break as a
space" and "inline contributes an image's alt text" both quote an `img` arm
reading `if let Some(alt) = element.attr("alt") { out.push_str(alt); }`. That
arm has not been in the file since it learned to say `image with no
description`. `guards.sh` refuses a break whose text it cannot find, and reports
that as a broken tool rather than as a finding about the guard, so both records
said nothing and read as covered. Verified by extracting `long_text.rs` at
`a343d7f` and searching it for the quoted text: zero occurrences.

## What a nested list and a table sound like

Measured by running the code, before and after.

### A nested list

Input `- Live is brown\n  - Older cable: red\n    - Check first\n- Neutral is blue`.

Before:

```
bullet, Live is brown
bullet, Older cable: red
bullet, Check first
bullet, Neutral is blue
```

Four identical announcements. A listener heard four separate jobs where
somebody had written one job with two notes under it.

After:

```
bullet, Live is brown
bullet level 2, Older cable: red
bullet level 3, Check first
bullet level 1, Neutral is blue
```

The level is said only when it changes. A flat list is not one word longer than
it was, which `test_a_list_that_never_nests_is_not_made_longer_to_listen_to`
holds. That is what a screen reader already does on a web page, so the
convention is borrowed rather than invented.

### A table

Input `| Name | Role |\n| --- | --- |\n| Grace | Admiral |\n| Alan | Fellow |`.

Before, the single paragraph:

```
NameRoleGraceAdmiralAlanFellow
```

After:

```
table, 2 columns, 2 rows
row 1. Name: Grace. Role: Admiral
row 2. Name: Alan. Role: Fellow
```

Two judgements went into that, and both are arguable.

**The size comes first** so somebody who does not want to hear a twenty-row
table finds out before it starts. Guardrail 5 asks that feedback be bounded.

**The column heading goes in front of every cell** rather than being said once
at the top. This is speech nobody can move back through: a listener who has
reached the fourth cell of the third row cannot re-hear a heading given forty
words ago, and working the column out from its position is a memory test rather
than a reading. The cognitive rule in `CLAUDE.md` asks that memory load be
reduced, and that outranks brevity here. It is also the more expensive choice on
a wide table, which is exactly what entry 271 exists to settle.

`name: value` is not a new form. `presentation::read_aloud` already renders
every labelled value that way, joined with `. `, so a table sounds like the rest
of the application rather than like a second dialect. A column with no heading
falls back to `column 2`, because a bare value with nothing in front of it has
lost the one thing a table was carrying.

### The seven callers

All five `read_aloud.rs` sites call `long_text::spoken`, so all five improve:
a contact's Notes at 370, a note's body at 389, and descriptions at 415, 442
and 516.

Both `from_markup` callers outside notes improve, and this is the part that
matters most because it is text a stranger's service wrote. A Google task
description arriving as `<ul><li>Budget<ul><li>Papers</li></ul></li></ul>` used
to become `- BudgetPapers` and be read as `bullet, BudgetPapers`. It now becomes
`- Budget\n  - Papers` and is read as `bullet, Budget` then `bullet level 2,
Papers`. A table in a Microsoft To Do body used to become one paragraph per
cell with its columns gone; it now comes back as a table and is read with its
headings.

## Part B: which shape, and what the other would have cost

**Chosen: one tree walk with a `Keeping` parameter, two public entry points.**
`from_markup` stays as it was and `from_markup_to_edit` is the second reading.
Three arms in `inline` differ, plus the block-level `img`.

**The alternative was a separate walk**, and it would have cost about 140 lines
of duplicated tag dispatch: `blocks`, `push_paragraph`, `push_item`, `list`,
`table`, `collect_rows`, `quote` and `inline`. The cost is not the lines. It is
that the next tag anybody adds has to be added twice, and the two will drift the
first time only one of them is changed. That is the argument `as_markup`'s own
doc comment already makes about keeping one copy of a note instead of two, and
it applies unchanged here. The eighteen guard records covering the walk would
also have covered only one of the two copies.

What the chosen shape cost instead: the `keeping` parameter moved every
signature and every call site in the walk, so eleven guard records had to be
re-measured a second time on the same branch. That is a real and predictable
cost, and it is smaller than a permanent second copy.

**`from_markup` is unchanged for its speaking callers, proven rather than
asserted.** Three tests exist for that alone:
`test_a_link_read_for_speaking_still_loses_its_address`,
`test_a_picture_read_for_speaking_is_still_its_description_alone`, and
`test_a_line_break_read_for_speaking_is_still_a_space`. Each is paired with its
opposite for the editing reading, so neither can pass against a do-nothing
implementation.
`test_the_two_readings_agree_about_everything_that_is_not_those_three` puts a
document holding a heading, a nested list, a numbered list, a quote, a table and
a paragraph through both and requires the results to be equal.

**The reader is wired, not written and shelved.**
`service::onenote_page::the_note_on` calls it. A reader nobody calls is a stub
presented as complete, which guardrail 3 is about, and that wiring is what turns
four fidelity rows from lost to surviving.

## What running it found that reading it would not have

**A line break came home with an indent nobody typed.** Read as a space, a
`<br>` absorbed the whitespace an HTML document holds between its tags. Read as
a line break it did not, so `Line one\nLine two` came back as `Line one\n Line
two`. `markup::tidied` takes it off. This was found by the test failing, not by
inspection.

**A picture comes back pointing at OneNote's copy of it.** The service stores
the picture and hands back a resource address of its own, so
`![The fuse box](https://example.org/fusebox.png)` returns as
`![The fuse box](https://graph.microsoft.com/v1.0/me/onenote/resources/...)`.
The picture is not lost and the address is not the one somebody typed. My test
asserted the original address and was wrong; the code was right. Entry 272.

**A table row survives byte for byte, which nobody predicted.** OneNote does not
name `th`, so a header row is written as ordinary cells. `from_markup` reads the
first row of a table as its heading row because a Markdown table has no other
shape. The demotion and the reading cancel, and
`| Left | Right |\n| --- | --- |\n| one | two |` comes home identical.

## The guard sweep, which was the expensive half

The eighteen records naming `long_text.rs` were re-measured five times across
the branch, because the count check fired twice and each repair had to be
confirmed. The remedy was run every time it was printed, and read rather than
having its exit code checked.

Across those runs, **eleven records were not what they said**, in three
distinct ways.

**Six were written as two lines naming a neighbouring match arm.** A break
spelled as `"ul" => list(child, out, None),` followed by `"ol" => ...` stops
matching the moment anything is inserted between them, and adding a `table` arm
broke five at once. All six are now one self-contained edit inside the arm they
are about. That is a general lesson about how to spell a break, not a fact about
this change.

**Four had red lists shorter than the truth.** The worst named five tests where
its break reddens forty. Thirty-one of the thirty-five it missed live in
`src/service/onenote_page.rs`, which arrived on 2026-09-11 and which no record
named. The count check compares test counts for the files a record names, so a
test added to a file no record names is invisible to it. `CLAUDE.md` already
documents this blind spot; this is it happening. Those files are named in the
records now, so the next test added to `onenote_page.rs` says so.

**Two were already unmeasurable on `main`**, described above.

One further finding, from the last run:
`test_a_picture_with_no_description_comes_back_as_a_picture_with_no_description`
stays green with the break for "blocks joins a p or div's inline content into
one line" applied, because a picture reaches the block walk as its own element
either way. It was removed from that record's red list. A named test that stays
green is a test whose name still promises something it no longer checks.

All eighteen now redden exactly the tests their records name.

## What I did not do, and why

**The accessibility agents were not consulted.** The brief asked for the
`Desktop Accessibility Specialist` and `cognitive-accessibility` agents on the
announcement wording, and I had no tool available to spawn a subagent. I read
the agent definition files instead, which contain no guidance on list-level or
table-header announcement, and fell back to two things that are checkable: the
convention a screen reader already uses on a web page for nested lists, and the
`name: value` form `read_aloud.rs` already uses everywhere else. That is weaker
than the brief asked for and it is the reason entry 271 is worded the way it is.

**A block-level `<img>` in the speaking reading is still wrong, and I left it.**
It contributes bare alt text with no marker saying it is a picture, and
contributes nothing at all when the sender gave no alt. So a picture in a Google
task's description is read as an ordinary paragraph, or vanishes.
`Piece::Image`'s own doc comment and guardrail 9 both say a picture nobody
described must still be announced, and the inline arm does that correctly.
I found this by measurement and deliberately did not fix it: it is pre-existing,
it is not caused by this change, and fixing it alters what is stored for
calendar and tasks. Entry 273.

**PIM-04 was not reworded.** Its criterion sentence is byte-identical. What
changed is the status annotation under it, which said "This is not met today",
and which would otherwise have been a false sentence left in the file. If that
counts as rewording for Pratik's purposes, revert that hunk; the code stands
without it.

## What only a screen reader can settle

The tests prove which words are produced. They prove nothing about whether those
words are good to listen to, and the distinction matters here more than usual
because a badly announced table is worse than a silent one.

Three open questions, all in entry 271:

- Does repeating a column heading on every cell flood a table with more than
  three columns, and at what width does it stop being worth it?
- Is "bullet level 2, Older cable: red" heard as a level marker, or does "level
  2" read as part of the item's text?
- Can a listener follow a wide table read linearly at all, or is the honest
  answer to say how big it is and stop?

## Ledger

**270 is closed.** All five losses it names come back as what they were, and the
second reader it asks for exists and is wired.

Three new entries carry what is left, none of which 270 covered:

| id | kind | what |
|----|------|------|
| 271 | unrun-verify | Whether the new announcements are pleasant to listen to. Only NVDA can settle it |
| 272 | deviation | A picture comes back pointing at OneNote's copy rather than its original address |
| 273 | unrun-verify | A block-level `img` read for speech has no picture marker, and vanishes with no alt |

## Commits

| commit | what |
|--------|------|
| `c615bc9` | RED: failing tests for depth and tables, 11 named plus the count check |
| `6365ad4` | GREEN: `Piece::Item` depth, `Piece::Table`, the announcements, 18 records re-measured |
| `cb910a7` | RED: failing tests for the storing reader, 9 named plus the count check |
| `f6cd2e1` | GREEN: `from_markup_to_edit`, wired into `the_note_on`, 18 records re-measured again |

Both red commits named every failing test in `Fails-until-green:` trailers and
were held to it: every named test ran, every named test failed, and nothing else
failed. Both named the count check alongside the real reds, with the reason,
because a commit adding tests to `long_text.rs` cannot have a clean tree around
it until the green code exists.

`scripts/check.sh all` passed on the branch before the merge, all four checks,
redirected to a file rather than piped.

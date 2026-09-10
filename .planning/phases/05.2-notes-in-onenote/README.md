# Phase 5.2: A page in a section in a notebook is not a title and a body (INSERTED)

Three plans, one per wave. Assembled 2026-09-06 against `main` at `9611b70`,
version `0.75.0`, `guards/guards.toml` holding 632 records, Rust floor `1.88`.
Nothing in the repository was changed while these were written.

**Corrected 2026-09-10 against `main` at `1aba3a5`, and none of these three plans
has executed yet.** Phase 5 landed in between. Version is `0.102.0` and
`guards/guards.toml` holds 683 records. Two corrections are about the work rather
than the figures: `05.2-01`'s round trip cannot read the shape OneNote hands back
without a decision this README's phase never made, and `05.2-02`'s notes
permission has to go into two scope lists, not one.

**One thing was wrong in all three from the day they were written**: the `phase:`
frontmatter, the `@`-includes and the `<output>` lines named
`05.2-a-page-in-a-section-in-a-notebook-is-not-a-title-and-a-body`, which has
never existed as a directory. The phase title in the heading above is where that
spelling comes from; `ls .planning/phases/` gives `05.2-notes-in-onenote`, and
`.planning/ROADMAP.md` names the phase "Phase 5.2: Notes in OneNote". Two of the
plans also named the 5.1 phase directory wrongly in the same way.

An inserted phase, following the precedent of
`02.1-what-phase-1-found-on-its-way-past`, which is tagged `(INSERTED)` in the
roadmap and whose files are named `02.1-08-SUMMARY.md`. It is the third of three
phases the old nine-plan phase 5 was cut into.

The phase takes its name from a sentence the codebase wrote before anybody
planned this. `src/application/new_item.rs:19` to 23 says a OneNote page "is an
HTML document inside a section inside a notebook rather than a title and a body,
so the mapping is a decision somebody has to make rather than an afternoon's
work".

**Goal.** OneNote goes behind the notes seam, and what the seam turns out to have
assumed about CalDAV is written down rather than absorbed.

**Requirements:** PIM-07, and the second half of PIM-08's proof.

**Roadmap success criteria this phase owns:** the OneNote half of criteria 4 and
5 of the six the roadmap gives phase 5 today.

## Why this is a phase and not a plan inside 5.1

This is the cut worth defending hardest, because it is not "OneNote is big". It is
that a OneNote page is a different kind of thing from everything the notes seam
will have met, and finding that out inside phase 5.1 turns into a decision nobody
can answer at the moment it is asked.

Measured on 2026-09-06 from Microsoft's own reference, with the pages named so
they can be re-read rather than believed:

- **`onenotePage` has no ETag and no `eTag` property.** Its properties are
  `content`, `contentUrl`, `createdByAppId`, `createdDateTime`, `id`,
  `lastModifiedDateTime`, `level`, `links`, `order`, `self`, `title`. There is no
  `If-Match` anywhere in the update reference. So OneNote's concurrency is a
  timestamp the service owns, where CalDAV's is an ETag and where
  `contact_identities` already holds a `provider_version` column.
  (`learn.microsoft.com/en-us/graph/api/resources/onenotepage`)
- **A page's body cannot be replaced.** The supported-actions table gives `body`
  append: yes, replace: no, insert: no. To make a page's body equal a newly
  rendered document you delete its elements one at a time by their generated ids
  and append new ones, or you delete the page and make another one, which changes
  its identity, its position and its links.
  (`learn.microsoft.com/en-us/graph/onenote-update-page`)
- **Most replaces need the generated id, and generated ids move.** The same page
  says a `replace` needs the id Graph generated, not a `data-id` you set, for
  everything except the title and images inside a div, and that generated ids
  "might change after a page update, so you should get the current values before
  building a PATCH request". So every write is preceded by a read of
  `../pages/{id}/content?includeIDs=true`.
- **The hierarchy is four levels, not three.** notebook, sectionGroup, section,
  page. (`learn.microsoft.com/en-us/graph/api/resources/onenote-api-overview`)

Put together, that means a note's stored form probably cannot make the round trip
PIM-04 requires. PIM-04's own `[D]` line says "a note edited here and read back is
byte-identical when nothing changed". Markdown source, rendered to input HTML,
normalised by OneNote, read back and turned into Markdown again, is not
byte-identical and cannot be made so through the page body. **That is a change to
a shipped requirement**, and a requirement change is not a paragraph in a plan. It
is a question for Pratik with evidence in front of it, which is what `05.2-01` is
arranged to produce.

**Corrected 2026-09-10: this phase is the second to meet that problem, not the
first, and the paragraph above was written believing it was the first.** `05.1-03`
built the CalDAV backend and found the round trip cannot be byte-identical there
either. RFC 5545 section 3.3.11 gives one escape for a line break and no way to
write a carriage return inside a value, so a note typed on Windows comes back with
plain line feeds from any calendar server. The phase 5.1 README's decision table
said CalDAV "is the only candidate whose round trip can be byte-identical", and
that sentence is what sent this problem here as though it belonged to OneNote
alone.

Three things change for `05.2-01`. The evidence it puts in front of Pratik covers
both backends rather than one, so the question is what the criterion should say
rather than whether OneNote may be excused from it. It is not discovering the
problem, so it must not report it as new; `src/service/note_document.rs`'s header
and `05.1-03-SUMMARY.md` carry the first measurement, and `.planning/REQUIREMENTS.md`
carries the refutation under PIM-04. And the gap between the two backends is one of
degree on this axis and not of kind: CalDAV loses carriage returns and keeps
everything else, OneNote cannot round-trip the source at all. Say which is which
rather than folding them together, because the answer for a backend that loses one
byte class is not obviously the answer for one that loses the form.

**And the ordering argument, which is the real one.** PIM-08 asks that the seam
take a second implementation with no migration. A second implementation written
in the same week as the seam, by the same person, against the same assumptions,
tends to agree with the interface. OneNote is the one candidate that cannot
agree, because none of the four facts above is true of CalDAV. It is worth more as
a later phase that reports what the seam had assumed than as a plan inside the
phase that built the seam.

## The plans

| Plan | Wave | Requirements | Depends on | Human | What it does |
|---|---|---|---|---|---|
| 05.2-01 | 1 | PIM-07, PIM-04 | none | yes, at the end | The mapping both ways, pure, with what it loses written down, and then what that costs PIM-04's byte-identical criterion |
| 05.2-02 | 2 | PIM-07 | 05.2-01 | no | The Graph OneNote client: four levels of hierarchy, a page read with its generated ids, a page written as patch commands, all parsed without a network |
| 05.2-03 | 3 | PIM-07, PIM-08 | 05.2-02 | yes, at the end | The backend behind the seam, a concurrency model with no ETag, the two green tests inverted, and a written account of where the seam had assumed CalDAV |

All three are new. Requirement coverage: PIM-07 by all three, PIM-04 by 05.2-01,
PIM-08 by 05.2-03.

**This phase depends on phase 5.1 having landed**, specifically on `05.1-02`'s
seam and contract, `05.1-03`'s CalDAV backend, and `05.1-04`'s list of what the
seam had assumed. `05.2-03` reads all three summaries before it reads any source.

## The two checkpoints, and neither re-asks anything Pratik has answered

**`05.2-01`, a decision, at the end of the plan:** whether PIM-04's
byte-identical criterion survives OneNote, and if not, how it is reworded. It
exists because a fact about somebody else's API forces it, and it is at the end so
both conversions and the measurement are done and merged whichever way it goes.
The three answers are not equivalent products: narrow the criterion to backends
that can round-trip, make the seam refuse backends that cannot, or weaken it to
"no information is lost". That choice is not the executor's.

**`05.2-03`, a human verify, at the end of the plan:** a screen reader pass over
the notes settings and the seam's verdict read back.

**`05.2-02` is conditional on the first checkpoint.** If `05.2-01`'s checkpoint
answered "refuse backends that cannot round-trip", this plan is cancelled and the
phase stops. `05.2-01`'s summary is required to say so in as many words.

## Premise corrections that must survive into execution

**Half the mapping already exists and the plans that preceded this one did not
know it.** `src/application/long_text.rs` holds a matched pair: `as_markup`
renders Markdown to HTML, and its partner reads structure back. `05.2-01` uses
them rather than writing a third renderer, and `05.2-01` premise correction 1 is
where the pair is named.

**`microsoft_graph.rs` carries no guard records and 33 test functions**, which
makes it the natural and free home for the client. `outward.rs` carries 10 and
takes no new `#[test]`.

**Two green tests assert the opposite of a notes backend, and one is half about
OneNote.** `test_notes_are_not_offered_a_sync_they_cannot_do` at
`context_menu.rs:611` asserts the note folder menu has no `Action::SyncNow`.
`test_notes_and_reminders_still_stay_on_this_computer` in `new_item.rs`, around
line 440, asserts `!supports(account, ItemKind::Note)` for `me@gmail.com` and
`me@outlook.com`, and its own comment says OneNote could work and the mapping is a
decision nobody has made. **A OneNote backend inverts the outlook half of that
loop and leaves the gmail half true**, because Google Keep's API is Workspace
only. `05.2-03` names both.

**A settings check is blind to a nested setting.** `every_setting_is_acted_on` in
`src/data/config.rs` reads the `AppConfig` struct itself, so a new top-level field
fails on arrival, which is the red half for free. A nested setting is invisible to
it, and the two settings that break `CLAUDE.md`'s reachability rule today are both
nested. If this phase adds anything a person can set, it goes top-level or the
summary says exactly why it cannot and how it is offered anyway.

**A model that assumes three levels of hierarchy works for most notebooks and
breaks on somebody's.** `05.2-02` bounds how deep it follows and says the bound in
a comment with its reason, the way `occurrences.rs` bounds its expansion with
`MOST_STEPS` and `MOST_DAYS_ONE_SERIES_SHOWS`.

## Costs every plan is written around

**Guard records: 683 in the file, re-measured 2026-09-10 at `1aba3a5`.** This
used to say 632, measured at `9611b70`, before phase 5 landed. Re-measure at the
tree you are on, with the awk in `CLAUDE.md`, because 5.1 lands before this phase
and adds records. These are the figures after phase 5 and before 5.1.

| file | records | test functions |
|---|---|---|
| `src/application/long_text.rs` | 18 | 38 |
| `src/data/config.rs` | 3 (was 2) | 59 (was 53) |
| `src/service/outward.rs` | 10 | 38 |
| `src/application/new_item.rs` | 1 | 42 |
| `src/presentation/wx_settings.rs` | 2 (was 1) | 0 |
| `src/application/context_menu.rs` | 1 (was 0) | 17 (was 15) |
| `src/application/conflict_choice.rs` | 0 | 17 |
| `src/data/message_cache/notes.rs` | 0 | 7 |
| `src/service/microsoft_graph.rs` | 0 | 33 |
| `src/application/notes_backend.rs` | created by `05.1-02`, count it | count it |
| `src/application/notes_sync.rs` | created by `05.1-03`, count it | count it |
| any new file | 0 | 0 |

**Put every new test in the new files and in `microsoft_graph.rs`, which is
free.** A `#[test]` in `long_text.rs` fires
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` inside the
commit gate and puts 18 builds and 18 full library runs on the critical path. If
`long_text.rs` has to change at all, changing it without adding a test there is
free.

**The census at the top of `guards/guards.toml`.** Line 79 and line 80 hold two
numbers that
`test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`
adds and compares with the `[[guard]]` count. Re-read 2026-09-10: they are 192
and 491 against 683 records. At `9611b70` they were 192 and 440 against 632, and
phase 5 moved the second. Phase 5.1 will move it again. Read both lines rather
than trusting either pair.

**The Rust floor is 1.88**, raised by `04-09` from 1.87 because rPGP declares it.
New code here is linted against it with `-D warnings`, where a let-chain
suggestion is a build failure and not a hint.

**Corrected 2026-09-10. This used to say a commit touching `Cargo.toml` makes
`scripts/which-checks.sh` answer `all`, the whole gate at roughly 311 to 353
seconds warm, and that those commits must be run detached. That stopped being
true on 2026-09-09.** The script now carries
`only_the_packages_own_version_moved` and escalates only when a manifest diff is
more than this package's own `version = "..."` line, so a version bump beside
source answers `affected`. The exception is the paragraph below: a diff that adds
a dependency does still answer `all`.

**The danger has flipped.** An executor who believes the full gate already ran on
each commit can skip the `scripts/check.sh all` a branch owes before the merge,
and the release build and the whole suite then run for the first time on `main`.

Never pipe `check.sh` into anything whose exit status is then read; that part is
unchanged.

**A red commit is allowed only on a branch**, must name every failing test in
`Fails-until-green:` trailers at column 0, and every named test must have run and
failed with nothing else failing.

**No new dependency is expected.** `pulldown-cmark`, `scraper`, `ammonia`,
`html-escape`, `reqwest`, `serde_json` and `quick-xml` are already in
`Cargo.toml`, and OneNote's page content is HTML over JSON rather than XML. If one
seems necessary, use the `dependency-audit` skill and put the justification above
it the way `ring`, `x509-parser` and `image` carry theirs.

## Estimates

All three plans use the same derivation as the rest of the set: `raw_tokens` is
35,000 per task, the mean raw projection per task across the nine phase-4 plans,
and `tokens` is that multiplied by 1.03, the mean of
`actuals.tokens / estimate.raw_tokens` over the six phase-4 plans with recorded
actuals (1.17, 1.29, 1.29, 0.48, 1.18, 0.78). Six samples with that spread is
`med` confidence and not `high`. Re-derive once phases 5 and 5.1 have actuals,
which will be seventeen more samples than this set had.

## What no plan in this phase can close

- **Whether a OneNote round trip is acceptable to somebody who uses OneNote.** A
  lossy mapping that a test says is lossy in a known way is still a mapping
  somebody has to live with, and only they can say.
- **Whether any of this works against Microsoft.** Every request in `05.2-02` is
  read off a loopback socket. No OneNote notebook has ever been used with this
  program.
- **Whether the concurrency model without an ETag is safe in practice.** A
  timestamp the service owns is a weaker answer than an ETag, and how much weaker
  cannot be measured without two clients writing the same page.

`.planning/WINDOWS.md` ends at entry 230 after phase 5, corrected 2026-09-10 from
the "entry 119 before phase 5" this used to say. Read its last row rather than
either number, and add entries through `gsd-tools windows` rather than by typing
into the table: the ledger has a JSON half and
`test_both_halves_of_the_ledger_say_the_same_thing` holds the two together, so a
hand-typed row reverts on the next tool write. Each plan says which entries it
owes, one entry per unrun thing rather than one entry for all of them. **`05.2-03`
named the ledger in five places and gave no task the job of writing to it; that is
corrected in the plan.**

## What is owed to documents, and belongs to whoever lands these

1. **A roadmap entry**, `### Phase 5.2: A page in a section in a notebook is not
   a title and a body (INSERTED)`, following the shape of the 2.1 entry, and
   depending on phase 5.1.
2. **PIM-04's byte-identical criterion is reworded by `05.2-01`**, after its
   checkpoint answers, and not before. A requirement corrected ahead of the code
   describes unbuilt work as built, which is this project's most repeated defect.
3. **`05.2-03` owes a written account of where the seam had assumed CalDAV**,
   against `05.1-04`'s list. That account is worth as much as the backend and is a
   success criterion rather than a nice-to-have.

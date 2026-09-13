---
phase: 06-how-the-application-speaks
plan: 03
status: partial
subsystem: ui
tags: [localisation, win32, dates, accessibility, nls, getdateformatex]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "`date_display` as the one place every module gets its date wording from, and `ENGLISH_ONLY` as the sentence that discloses what it cannot do"
provides:
  - "`src/common/how_the_machine_writes_dates.rs`: one place that asks Windows how a date is written, with two entry points because a month inside a date and a month on its own are different words"
  - "Three readings in `date_display` and the appointment form's month list taking their month names from the machine"
  - "A settings sentence that names what is still English, including the site its own doc comment remembered and its text had dropped"
affects: [06-04, 06-05, 06-06, 06-07, 06-08]

actuals:
  tokens: 13800
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A private `_asking` twin beside each public reading, taking the locale, so a test forces `en-US` and asserts about this code rather than about the machine it ran on"
    - "An enum of four shapes rather than a free-text date picture, because Windows does not refuse a wrong picture: asked for `ZZZZ` it answers `ZZZZ`"

key-files:
  created:
    - src/common/how_the_machine_writes_dates.rs
  modified:
    - src/presentation/date_display.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/ui_types.rs
    - src/presentation/read_aloud.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml

key-decisions:
  - "Two Win32 mechanisms, not one: `GetDateFormatEx` with a picture for a date that has a day in it, `GetLocaleInfoW` with `LOCALE_SMONTHNAME1` to `12` for a list that has none"
  - "The wrapper lives in `src/common/`, because `src/service/signed_mail.rs` will need it and `src/service/` reaches `presentation` nowhere today"
  - "`ENGLISH_ONLY` reworded in task 2 rather than task 3, because the moment a month name stops being English the old sentence is false on a live settings screen"
  - "The English ordinal removed rather than kept: a date picture cannot produce it, `date_part` never had one, and no other language wants one"

requirements-completed: []

coverage:
  - id: D1
    description: "A date written by this program carries the month name this computer uses, in the form a date puts a month in"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib presentation::date_display::tests::test_a_date_and_a_month_heading_name_the_same_month_differently"
        status: pass
  - id: D2
    description: "The stored order and wording still decide the shape; the machine supplies only the words"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib presentation::date_display::tests::test_the_four_stored_combinations_still_read_as_they_always_did"
        status: pass
  - id: D3
    description: "The appointment form's month list carries the twelve names this computer uses, in their standalone form"
    requirement: FEEDBACK-02
    verification:
      - kind: other
        ref: "read at src/presentation/wx_item_form.rs:849; no test reads the Choice. WINDOWS.md 361"
        status: unknown
---

# Phase 06 Plan 03: A month name is the machine's to say, Summary

Dates now carry the month name this computer uses, and nothing about the shape
a person chose was taken away to get it. Tasks 1 and 2 are merged. Task 3 was
not attempted: the blocking checkpoint between them is unanswered on purpose,
and task 3 is written against its answer.

**What works.** On a French computer a date reads "26 juillet 2026". On a
Russian one it reads "26 июля 2026", with the month in the form a date puts a
month in rather than the form a list uses. Choosing month first on a French
computer gives "juillet 26, 2026", which no French computer writes on its own,
because that is what the person asked for. Where Windows has no names, dates
stay English and nothing is said about it.

**What does not.** Day names are still English everywhere. So is the date in
the eight signature-outcome sentences. So is relative wording, which is the
checkpoint. And nobody has heard a localised date, in any language, ever.

## What I was handed, and what I made of it

A previous executor was terminated mid-task. Task 1 was committed in three
commits; task 2 was staged and uncommitted, four files, and the message for its
red commit was being written when it stopped. Its last note said a third guard
had fired that it had not predicted, and that it was going to read it rather
than add it to the trailers blindly. That was the right instinct and finishing
that reading was the first job here.

**The staged work was kept, and it was worth keeping.** Read before being built
on. Three things in it are better than what the plan asked for:

1. **The red half is a real red.** The two date readings were stubbed with the
   *standalone* month lookup, which is the plausible wrong implementation: it
   compiles, it reads correctly in English, and it reads correctly in French. It
   is wrong only in Russian and Polish, and the test written to catch it is
   written in Russian for that reason. A red that fails for the exact reason the
   task exists is worth more than a red that fails because nothing is
   implemented.

2. **The French assertion would not have caught the fault, and the test says
   so.** `fr-FR` writes "juillet" both ways, so
   `test_a_french_machine_gets_a_french_month_in_the_order_the_person_chose`
   passed against the wrong mechanism. Measured here, not reasoned about: it was
   green in the red run.

3. **One existing assertion was strengthened rather than merely updated.**
   `read_aloud`'s birthday test asserted `contains("Birthday: March 14th")`. A
   prefix assertion cannot see anything added to or removed from the end of what
   it reads, so it could not have seen the ordinal appear or disappear at all.
   It now splits on the separator the reading joins with and compares the whole
   clause.

**One thing in it was wrong, and I changed it.** The reworded `ENGLISH_ONLY`
said "The month names, the order of the day and month, and the clock all follow
this computer." That is a larger claim than the truth.
`src/service/signed_mail.rs:1373` writes `%B` into eight sentences a person
hears about a signature, so a month name there is still English. The doc comment
directly above the constant had remembered the signature date; the string
underneath it, which is the only half anybody reads, had dropped it. A doc
comment is read by whoever changes the file and the constant is read by whoever
uses the program, and a disclosure is for the second.

Old text, as staged:

> The month names, the order of the day and month, and the clock all follow this
> computer. Wording such as "2 days ago" is still written in English whatever
> language this computer is set to, and so are the day names in a repeating
> appointment.

New text, as merged:

> The month names in a date, the order of the day and month, and the clock all
> follow this computer. Some wording stays in English whatever language this
> computer is set to: phrases such as "2 days ago", the day names in a repeating
> appointment, and the date in a message about a signature.

No count of what is left, deliberately. The list shortens as each site is done,
and a number in the text is one more thing to remember to change.

## What was really red, and what I had not been told

I was told one test was failing and warned to expect the guard count check. A
full run in the commit's own scope found **three**, and the third is the one the
terminated executor had met and not identified.

| Test | Where it ran | Told in advance |
|---|---|---|
| `presentation::date_display::tests::test_a_date_and_a_month_heading_name_the_same_month_differently` | scoped `--lib` | yes |
| `test_every_guard_record_says_how_many_tests_the_files_it_names_held` | `house_style` | warned to expect |
| `test_every_guard_record_still_names_one_place_in_the_tree` | `house_style` | **no** |

**The third is a rename, and it is a distinct failure mode from the count
check.** The record "a whole day is spoken as a date under the relative style"
carries a break whose text calls `date_part(day, settings)`. Task 2 renames that
call site to `date_part_asking(which, day, settings)`, so the record's `before`
no longer exists in the file. `CLAUDE.md` describes the rename hazard in terms
of a record naming a *test* that no longer exists; this is the same hazard one
step over, a record naming *code* that no longer exists, and a different check
catches it.

It was named in the red trailers rather than fixed first, because correcting a
record means measuring the guard by hand against whatever replaces the code, and
that code is the green half. Both guard checks are in the same position and the
red commit message says so.

**How I found them.** Running `scripts/check.sh affected` directly, with no
changed-file list, runs only the tree-reading guards: the scoped module runs are
driven by the index the hook hands over, and a bare invocation hands over
nothing. That run found the two `house_style` failures and would have missed the
one that mattered. The four scoped `--lib` runs and the coupled `checkbox_labels`
suite were then run by hand to complete the set. **A direct
`scripts/check.sh affected` is not the same run the hook makes**, and reading it
as one would have produced a red commit naming two tests where three failed.

The gate then confirmed the set independently: `455f30ce` was accepted, which
means every named test ran, every named test failed, and nothing unnamed failed.

## The four date sites, found by my own search

The plan warned that this phase's research had sent a planner to a line phase
5.2 deleted. Every coordinate below was re-derived on 2026-09-13 against the
working tree, command first.

```
grep -rn 'MONTHS' src/ tests/        -> nothing; the array is gone
grep -rn '%A\|%B' src/               -> src/service/signed_mail.rs:1373
                                        plus two unrelated matches, a
                                        percent-encoded mailto fixture and a
                                        %APPDATA% path in a doc comment
grep -rn '"Monday"' src/             -> src/application/occurrences.rs:701
                                        src/application/when_people_are_free.rs:1442
grep -rln '"January"' src/ tests/    -> src/common/how_the_machine_writes_dates.rs
```

| Site | What it writes | Mechanism it now uses | Done |
|---|---|---|---|
| `date_display.rs` `date_part_asking` | the date in every list cell and reading | `a_date`, a picture with a day in it | yes |
| `date_display.rs` `a_day_in_words_asking` | a birthday with no year | `a_date`, the two year-less shapes | yes |
| `date_display.rs` `a_month_in_words_asking` | a month heading, no day beside it | `the_twelve_month_names` | yes |
| `wx_item_form.rs:849` | the month list in the appointment form | `the_twelve_month_names` | wired, untested |
| `occurrences.rs:701` | day names in a repeat-series sentence | none yet | task 3 |
| `signed_mail.rs:1373` | the date in a signature sentence | none yet | task 3 |
| `when_people_are_free.rs:1442` | day names | not shipping | checked, not assumed |
| `repeating.rs:275,337` | `"MO"` to `"SU"` | must stay English | RRULE protocol codes |

`when_people_are_free.rs:1442` was checked rather than taken on the plan's word:
the file's last `#[cfg(test)]` is at line 1282 and the file is 2544 lines, so
line 1442 is inside the test module.

The plan's premise table also names an mbox `From_` separator at
`message_files.rs:410`. `grep -n '%a %b' src/service/message_files.rs` finds
nothing on this tree. I did not chase it further, because it belongs to task 3's
guard exclusion list, but **whoever writes that list should search for it rather
than copy the coordinate.**

## Both Win32 mechanisms, and why each is where it is

A month name inside a date and a month name on its own are different words in
Russian, Polish, Czech and Lithuanian. Microsoft's `LOCALE_SMONTHNAME1` page
says a name looked up alone is the standalone or nominative form, and that the
genitive comes from `GetDateFormatEx` with a picture holding both a numeric day
and `MMMM`.

Measured rather than believed. The green suite reproduces it:
`date_part_asking(ru-RU, 2 January)` contains "января" and
`a_month_in_words_asking(ru-RU, January)` starts with "Январь". Those are
different words, not different capitalisation, and the test asserts both halves
so that neither reading can quietly start using the other's mechanism.

So: a date has a day in it and goes through `a_date`; a list of twelve months
has no day and goes through `the_twelve_month_names`. The appointment form's
month list is the second kind, and `date_display`'s month heading is too.

**The measurements in the module's doc comment are the terminated executor's,
taken 2026-09-13, and I did not re-take them.** What I did do is watch the
Russian and Polish claim reproduce as a passing test at green, which is the
claim the whole two-mechanism decision rests on. The other findings recorded
there, that Windows answers `ZZZZ` for a picture of `ZZZZ`, that an empty locale
name is `LOCALE_NAME_INVARIANT` and answers English everywhere, that both calls
count the terminating zero in their returned length, and that Windows refuses
the 29th of February in a year that has none with error 87, are recorded in that
file with their date and are not re-measured here.

## The ordinal, handled deliberately

`ordinal` was private and had exactly two callers, both in `a_day_in_words`. A
date picture cannot produce "14th", and no language other than English wants
one. `date_part` has always written "9 December 1906" without one, so the module
disagreed with itself about the same day.

It was removed, not silenced. Clippy runs at `-D warnings` and the green commit
passed clippy, so there is no dead function and no `#[allow]` anywhere:

```
grep -n 'fn ordinal' src/presentation/date_display.rs   -> nothing
grep -rn 'allow(dead_code)' src/presentation/date_display.rs -> nothing
```

Three readings asserted the ordinal and were corrected: the birthday test in
`date_display`, the contact detail test in `ui_types`, and the spoken-birthday
test in `read_aloud`. The last of those was the prefix assertion described
above.

## Criterion 2, clause by clause

Read from `.planning/ROADMAP.md` at source, not from a paraphrase:

> Month names, day names and relative wording follow the machine's locale,
> falling back to English silently where there is no translation.

| Clause | Closed | Why |
|---|---|---|
| Month names follow the machine's locale | **no, mostly** | Closed in `date_display`'s three readings and the appointment form's month list. Not closed in the eight signature-outcome sentences, which still write `%B`. That is task 3. |
| Day names follow the machine's locale | **no** | `occurrences.rs:701` still matches a `chrono::Weekday` to an English string. Nothing in this plan touched it. Task 3. |
| Relative wording follows the machine's locale | **no, and deliberately** | This is the plan's own blocking checkpoint, left unanswered. |
| Falling back to English silently where there is no translation | **yes, with one wart** | `a_date` and `the_twelve_month_names` both fall back through `unwrap_or_else` to the English array, with nothing said. The wart is that the fallback is wider than the clause: a corrupt stored day such as `--02-30` is refused by Windows and comes back English even on a machine that has the language. WINDOWS.md 363. |

**FEEDBACK-02 is not complete** and `requirements-completed` is empty for that
reason.

## Deviations from plan

**1. [Rule 2] `ENGLISH_ONLY` reworded in task 2 rather than task 3.** The plan
puts the rewording in task 3. Task 3 is blocked behind a checkpoint that will
not be answered in this session, and task 2 merges to `main`. Leaving the old
text would have shipped a settings screen saying month names stay English on the
same build that made them stop. That is threat T-06-12 in this plan's own
register. Done in `082ab42e`.

**2. [Rule 1] The staged rewording over-claimed and was corrected.** Described
above. Same commit.

**3. [judgement] The `read_aloud` prefix assertion was strengthened, not just
updated.** Inherited from the staged work and kept. It is a change to a test
nobody asked to change, and it is the right one: the old assertion could not
have failed for the change it was guarding.

**4. [scope] `wx_item_form.rs:849` is wired and untested.** The month list now
comes from the machine and nothing reads the `Choice` back, because it is built
inside a closure in `build_date_fields` and needs a live parent window. The plan
asked only that `cargo test --lib -- presentation::wx_item_form::` pass, and it
does. This is glue over a function that is itself tested, and it is reached from
`wx_item_form.rs:1007` and `wx_send_later.rs:127`, so it runs. But nobody has
opened the form and looked at the list. Recorded as WINDOWS.md 361 rather than
claimed.

## Guard records

**No new record.** Task 1 wrote one for the wrapper; task 2's plan did not ask
for another and the `date_display` rules are already covered by three.

`guards/guards.toml` holds **746** records, counted 2026-09-13 with a TOML
reader rather than a grep, because the `awk` recipe in `CLAUDE.md` misses
records in TOML's inline-table form. The census at lines 79 and 80 reads 192 and
554, which sums to 746. Task 1 bumped it; task 2 added no record, so it is
unchanged.

**One record corrected, then measured.** "a whole day is spoken as a date under
the relative style" had its `before` and `after` text updated for the renamed
call, and a comment added above it recording why and how it was found. It was
then run through `scripts/guards.sh --remeasure` rather than trusted:

```
-- a moment written with a T is one the reader knows
   all 34 tests named went red, and nothing else did
-- a whole day is spoken as a date under the relative style
   all 7 tests named went red, and nothing else did
-- the out-of-hours note judges the hour the cell speaks
   all 2 tests named went red, and nothing else did
```

All three fingerprint `src/presentation/date_display.rs` and all three moved
**from 37 tests to 41**. Both directions held: each break reddens exactly the
tests its record names, no more and no fewer. The corrected record still
reddens its seven, which is what makes the correction a re-measurement rather
than an edit until it applied.

## What the gate selected, and the holes in it

`scripts/which-checks.sh` answered `affected` for both commits, on the branch.

The red commit changed four `src/presentation/*.rs` files and the gate ran four
scoped `--lib` runs, the `checkbox_labels` suite coupled by `guards.toml`, and
the four tree-reading guards. Everything mapped.

The green commit changed `src/presentation/date_display.rs`, `guards/guards.toml`,
`Cargo.toml`, `Cargo.lock` and `docs/changelog.md`, and the gate ran **one**
scoped run, for `date_display`. The other four files map to no target, which is
three of the five holes already known plus `Cargo.lock`, which is not on that
list and behaves the same way. The changelog was still read, because
`house_style` and `the_words_that_say_nothing` walk documents and run on every
commit whatever changed. `guards/guards.toml` is likewise read by the two
record-checking guards in `house_style`. So nothing went unchecked here, but the
scoping said nothing about four of the five files and would not have.

## The checkpoint, left unanswered

**Question: what should "2 days ago" do on a French computer?**

Month and day names can now follow the machine. Relative wording cannot, because
there is no Windows API for "2 days ago". `relative_to` at
`src/presentation/date_display.rs` builds those phrases through `plural`, which
is English pluralisation: one form for 1 and one for everything else. Russian
needs three forms and Arabic six.

**This is not a question about difficulty. It is a question about what a date is
for.**

| Option | What it costs | What it gives |
|---|---|---|
| 1. Keep it English and say so | Nothing. The sentence on the Reading tab already says it, as of this plan | Month names right, an English phrase still inside a French sentence, in every list cell under the shipped relative style |
| 2. Fall back to the date where the locale is not English | Small, and slightly larger than the plan thought. See below | Nothing ungrammatical is ever spoken. A French user loses "is this recent" on every row, permanently |
| 3. Take on real plural rules | Large. A new crate or a hand-written rules table, a `dependency-audit` conversation, and a package legitimacy audit `06-RESEARCH.md` does not carry | The criterion as written, for every language |

**Two things I can add to the plan's own analysis, having now read the code.**

*Option 2 is slightly more than the plan said.* The plan says `relative_to`
returns `None` and every caller already handles it, which is true: both callers
are `&& let Some(relative) = relative_to(...)` and both fall through to the
absolute date. But **there is no reader in this tree that answers "what language
is this machine set to"**. `date_display` reads `LOCALE_ILDATE` and `LOCALE_ITIME`
as digits, which are the order and the clock, not a language. Option 2 needs a
new Win32 question, probably `LOCALE_SISO639LANGNAME`, and that is a second
locale-shaped question in the calendar of exactly the kind the doc comment at
`ui_types.rs:1049` argues against opening piecemeal. Still small. Not zero.

*Option 2's cost falls hardest on the person this program is for.* The module's
own header says relative wording exists because "2 days ago" answers "is this
recent" in three syllables where "July 24, 2026 at 9:15 AM" takes a dozen. For
somebody reading an inbox by ear, row by row, that is the difference between
scanning and wading, on every row, forever, for the crime of not using an
English Windows. That is a real accessibility loss and it should be named as one
rather than counted as small because the diff is small.

**My recommendation: option 2, with two conditions.**

Option 2 is right because it is the only one that never speaks an ungrammatical
sentence, and because a French user who gets a longer date can tell that
something is different, whereas an English phrase inside a French sentence just
sounds like the screen reader misbehaving, which is the exact failure this whole
piece of work exists to stop.

The two conditions:

1. **Say it where they meet it.** `ENGLISH_ONLY` should gain a clause saying
   that on a computer not set to English, dates are always written out rather
   than said as "2 days ago". Otherwise a French user notices their dates are
   longer than everybody else's and has nothing telling them why. A silent
   degradation is the thing this project keeps writing down as worse than a
   stated one.

2. **Ledger the loss, do not close it.** Option 2 does not satisfy the
   criterion's third clause, it declines it. That should be a ledger entry
   naming what a non-English user does not get, so option 3 stays a live
   question rather than becoming a thing nobody remembers was decided.

If the answer is a combination, the one worth considering is option 2 as the
behaviour, option 1's honest sentence as the disclosure, and option 3 left as a
note rather than a promise. That is the plan's own suggestion and I agree with
it.

**What none of these settle**, whichever is chosen: whether a French month read
by a French voice, in a date whose order came from the same machine, sounds
right. That needs a French Windows, a French NVDA voice and somebody who speaks
French.

## Structure proved, experience not

Per rule 11, plainly which is which.

**These assertions prove structure**, that the bytes this code writes are the
bytes Windows hands back: the Russian genitive test, the French month test, the
four-stored-combinations test, the month-heading range test, and the nine tests
in the wrapper module. They are string comparisons against a forced locale name.
They are real and they would catch the faults they were written for. They say
nothing about how any of it sounds.

**Only a person can settle these**: whether a French month in an order the
person chose reads as a date rather than a fault; whether a Russian genitive
month is what a Russian listener expects in a list cell; whether "14 March"
without the ordinal is heard as well as "14th March" was; and whether the twelve
names really appear in the appointment form's month list, which no test reads.

**Ledgered**, through `gsd-tools windows append`, both halves matching at 364
entries:

| id | kind | What |
|---|---|---|
| 360 | unrun-verify | No date in any language has been heard by a screen reader in that language |
| 361 | unmet-truth | The appointment form's month list is wired to the machine and nothing tests it |
| 362 | unmet-truth | Criterion 2's day-name, signature-date and relative-wording clauses are open |
| 363 | unmet-truth | The English fallback is wider than the criterion: a day no month has falls back even where the language exists |
| 364 | unrun-verify | The genitive was produced on an en-US machine through a forced locale name, never on a Windows installed in Russian or Polish |

## Commits

| Commit | What |
|---|---|
| `733c0dce` | test(06-03), task 1 red |
| `b857be6d` | feat(06-03), task 1 green |
| `a7f6346b` | fix(06-03), the non-Windows half builds clean rather than merely builds |
| `455f30ce` | test(06-03), task 2 red, three tests named and held to it |
| `082ab42e` | feat(06-03), task 2 green, version 0.121.0 |
| `cf07ea3a` | docs(06-03), this summary, the ledger, `STATE.md` and `ROADMAP.md` |
| `e98514b0` | the merge into `main` |

The first three are the terminated executor's. The last two are mine, and the
staged work it left is inside `455f30ce` with the one correction described
above.

## The gate, and one run of three that went red

`scripts/check.sh all` was run on the branch at `cf07ea3a` under rustc 1.98.1,
not piped into anything. Zero failures, release build included, exit 0.

**The first attempt at the merge commit then failed one integration target,
`a_move_says_what_has_not_been_sent`, and nothing explains it.** The tree had
not changed. That target ran alone immediately afterwards and passed, all ten
cases, and it passed again in the run that completed the merge at `e98514b0`.
So the same tree is green on two full runs and red on one.

**The detail was lost, and that is the part worth not repeating.** The merge
output was read through `tail -20` and which case failed scrolled past. A
transient failure whose text nobody kept is one nobody can diagnose. This
target builds a live window and the failing run followed another full run
closely, so contention is the obvious guess, and it is only a guess.
`WINDOWS.md` 365 rather than absorbed, because a check that fails one run in
three while reading as green is guardrail 4.

## Task 3

Not attempted, on instruction and correctly. It is written against the
checkpoint's answer: what `relative_to` does outside English decides what its
own tests assert, and `ENGLISH_ONLY` is reworded there for a second time to
match. Starting it before the answer would mean writing it twice.

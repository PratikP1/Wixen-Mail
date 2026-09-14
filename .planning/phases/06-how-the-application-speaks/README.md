# Phase 6: How the application speaks

Nine plans, one per wave. Eight were assembled 2026-09-12 against `main` at
`b8857bc8`, version `0.116.0`, `guards/guards.toml` holding 729 records, the
library holding 7,075 tests, `.planning/WINDOWS.md` at entry 324 with 302 open.
Nothing in the repository was changed while these were written, except that
one `cargo test` was run to check a command these plans use, which wrote only
to `target/`.

**The ninth arrived on 2026-09-14**, against `main` at `562b4b90`, version
`0.122.0`, 755 records by a TOML reader, `.planning/WINDOWS.md` at entry 378.
Pratik answered 06-05's checkpoint that day with option 3 and widened it: a
tone every minute until the reminder window has focus, and then one window for
every due thing, reminders, tasks and calendar events, each row saying which it
is, shaped so a mail message somebody asks to be told about later can join
after version 1. 06-05 was rewritten around the first half and 06-09 written
for the second. See "Version 2's second seam" below for what the widening
leaves for the next milestone.

**Read against `06-RESEARCH.md`, written the same day against `febe8e4`.** That
document is the primary source and most of it survived being used. Where a claim
in it turned out wrong when a plan went to lean on it, the plan carries the
correction in its `<premise_corrections>` block and this README lists the three
that change what gets built. `06-RESEARCH.md` measured 720 guard records and
version `0.112.0`; four version bumps and nine records arrived between its read
and this one, all from phase 7.

**Goal.** The user controls what is spoken, brailled, sounded and shown, reads
dates in their own language, and the project knows which parts of WCAG its scans
can and cannot judge.

**Requirements:** FEEDBACK-01, FEEDBACK-02, FEEDBACK-03.

**Roadmap success criteria this phase owns:** all four, plus the two items
inherited from phase 1.

## The plans

| Plan | Wave | Criterion | Depends on | Human | What it does |
|---|---|---|---|---|---|
| 06-01 | 1 | 1 | none | no | **Done.** The model can say what somebody chose, separately from what they get, and a seventeenth event cannot exist without a control |
| 06-02 | 2 | 1 | 06-01 | decision, answered | **Done.** Sixteen events, three answers each, on the Feedback tab, and the two global boxes that cannot mean what they say are one that can. Criterion 1 has five clauses, four close structurally, none is heard |
| 06-03 | 3 | 2 | none | decision, answered | **Tasks 1 and 2 done.** Month and day names come from Windows, through `GetDateFormatEx`, at all four shipping sites. Since 2026-09-13, four tasks: task 3 puts the relative wording through Project Fluent as the first piece of version 2, and task 4 is the day names and the signature sentence |
| 06-04 | 4 | inherited A | none | decision, answered | **Done.** A permission for one account has a screen, three boxes that can only narrow, and the list that recorded it as unreachable is empty with its guard retired rather than left green over nothing |
| 06-05 | 5 | inherited B | none | decision, answered | A reminder due while somebody is typing is said and sounded at once, its window is held for one look and then opened anyway, and the tone comes back once a minute until focus reaches the window, ten times at most. Rewritten 2026-09-14 around Pratik's answer |
| 06-06 | 6 | 3 | none | decision, answered | **Done.** A change under `.github/workflows/` earns every check. The scanner is Axe.Windows v2.4.2 by tag and by the zip's SHA-256, and the scan looks at thirty windows where it looked at eleven, every one opened on a fresh profile here and none unreachable; seventeen nested dialogs are outside it by name. Three defects in the scan itself were measured from the last CI log and fixed: a clean window was reported as a failed scan, the MSAA channel had never read a dialog, and a dialog that failed to open was scanned as the main window. Nothing has been scanned by CI since |
| 06-07 | 7 | 3 | 06-06 | decision, answered | **Done.** "Roughly half of WCAG" is a list of fifty-five at `docs/wcag-coverage.md`, both counts re-taken from the pinned rule list and the specification with their parts summing: 155 rules citing three WCAG criteria, 55 criteria at A and AA. The three live in `src/presentation/what_the_scans_can_judge.rs`, a reading holds the page's table to them both ways, shown a planted violation in each direction and failing on an empty page or list, and the scan's own summary names them. Five sentences corrected, dated, old wording kept; three `REQUIREMENTS.md` lines corrected in place on Pratik's answer. `check.sh`'s documents-only run now includes the reading, because a page edit was the one commit that did not run it. Nobody has walked a criterion; no scan has run |
| 06-08 | 8 | 4 | 06-06, 06-07 | action | The findings the scan really produces, judged one at a time, and the list only a person can walk |
| 06-09 | 9 | inherited B, widened | 06-05 | decision | One window for every due thing: reminders, tasks with a due date and events with an alert, one row each in time order, each row saying its kind first, with snooze, dismiss and done per row and snooze all and dismiss all. `Due` gains a kind and an identity shaped so a fourth kind, a mail message somebody asked to be told about later, joins after version 1 without a migration. Three decisions at its checkpoint |

Requirement coverage: FEEDBACK-01 by 06-01, 06-02 and 06-04, and 06-05 and
06-09 carry it in their frontmatter because it is the nearest, while what they
really close is the second item inherited from phase 1 and Pratik's widening
of it; neither FEEDBACK-01 nor any other requirement is about a due window, and
"Version 2's second seam" below says where that belongs. FEEDBACK-02 by 06-03.
FEEDBACK-03 by 06-06, 06-07 and 06-08.

## Why nine plans, and why one per wave

**Eight because the phase is eight nearly independent pieces**, and because each
fits in two or three tasks without a task touching more than five files. The
research counted four pieces; it is eight once the two inherited items are
separated from the criteria they attach to, the per-event model is separated
from the panel that uses it, and criterion 3's workflow half is separated from
its document half.

**Nine because Pratik widened the eighth piece on 2026-09-14** into something
06-05 could not hold at the size it was planned: a list of every kind rather
than one reminder, a model with a kind on it, two feeds that have never
existed, a table for a hold, and two decisions underneath the feeds. 06-09 is
that, and it is over the two-or-three-task guideline at four tasks and a
checkpoint, on 06-03's precedent, which ran four tasks with a checkpoint in
the middle and was executed in two sittings. The checkpoint is the seam if it
is ever split: the model before it, the feeds, window and wiring after.

**One per wave because every plan here writes `guards/guards.toml`.** That is
the same reason phase 5.1 ran six plans in six waves, and the arithmetic is the
same. Eight of the nine also write `docs/changelog.md`. A wave is a set of
plans sharing no file, and no two plans here can satisfy that. 06-09 also
depends on 06-05 for real, not only by the file rule: it reads the split
sentence function, the repeat rule and the shared typing helper by the names
06-05's summary records.

Why every plan writes the guard records, stated per plan rather than asserted,
counted 2026-09-12 at `b8857bc8` by parsing `tests_last_seen` blocks rather than
by grepping a file name:

| Plan | The file it adds a test to | Records fingerprinting it | Tests there today |
|---|---|---|---|
| 06-01 | `src/presentation/accessibility/feedback.rs` | 1 | 41 |
| 06-02 | `tests/checkbox_labels.rs`, plus a new file under `tests/` | 1, plus one new record | 1 |
| 06-02, as it turned out | `feedback.rs` gained 2 tests, `tests/every_event_has_a_control.rs` is new at 1, `tests/checkbox_labels.rs` stayed at 1 | 2 flagged and remeasured, 5 written | 49, 1, 1 |
| 06-03 | `src/presentation/date_display.rs` | 3 | 37 |
| 06-04 | `src/data/config.rs`, `tests/house_style.rs` | 5 and 19 | 63 and 69 |
| 06-05 | `src/presentation/one_question_at_a_time.rs` | 1 | 19 |
| 06-05, as rewritten 2026-09-14 | `one_question_at_a_time.rs`, `src/presentation/wx_reminder_alert.rs`, `tests/wired.rs` | 1, 0 and **14** | 19, 1 and 69 |
| 06-06 | `src/presentation/scan_target.rs`, `scripts/which-checks.test.sh` | 0 and 0 | 6 and n/a |
| 06-07, as landed 2026-09-14 | `src/presentation/what_the_scans_can_judge.rs`, a new module; `tests/house_style.rs` untouched | 0, then 1 | 8 |
| 06-08 | `.planning/WINDOWS.md`, documents | 0 | n/a |
| 06-08, as landed 2026-09-14 | `src/presentation/editor_document.rs`, `wx_managers.rs`, `wx_account_manager.rs`, `scan_target.rs`; `tests/no_label_is_only_a_space.rs`, a new target | 1, 6 then 7, 6, 2; 0, then 1 | 82, 44, 14, 11; 4 |
| 06-09 | `src/application/due.rs`, `src/common/catalogue.rs`, `src/presentation/date_display.rs`, `src/data/message_cache/tasks.rs`, `src/data/message_cache/held_alerts.rs` (new), `src/presentation/wx_reminder_alert.rs`, a new file under `tests/` | 4, 6, 6, 4, 0, whatever 06-05 wrote, 0 | 23, 14, 42, 21, 0, 1 plus 06-05's, 0 |
| 06-09, as landed 2026-09-14 | the above, plus `src/presentation/ui_types.rs`, `src/application/event_alerts.rs` (new), `src/application/calendar.rs` and `src/presentation/managers.rs`, the last two touched without a new test because a test in either is hours of remeasure | 10, 6, 6, 4 then 5, 0 then 4, 1 then 6, 0 then 1; 5 then 6, 0 then 5, 72, 50 | 46, 15, 47, 22, 5, 15, 1; 79, 4, 194 and 137 by the count check's rule |

The 2026-09-14 rows were counted at `562b4b90` with a TOML reader, not with
the `awk` below, because that `awk` reads only the multi-line
`tests_last_seen` shape and misses a record written with an inline table; on
that day the two agreed for every file above, and they need not next time.

**Where 06-05's fourteen come from.** `tests/wired.rs:3622-3652` reads the
body of the folders question and pins the three tokens of its typing check
inside it. 06-05 hoists that check into a helper both call sites use, which
moves the tokens out of the body, so the reading has to be taught the helper
and a sibling reading written for `raise_what_is_due`. One `#[test]` in a file
fourteen records name. The alternative, copying the check into both
functions, was refused by the plan as written on 2026-09-12 for a reason that
still holds. **The plan as first written did not know this file existed in
its path**, and its acceptance line "`cargo test --test wired` passes, which is
what proves the call site is really there" was false, because nothing in that
file read the reminder's call site. It reads it once 06-05 has run.

Take the command from `CLAUDE.md` rather than a grep. A grep for `config.rs`
across `guards/guards.toml` answers far more than five, because a file name
appears in a record's `file`, its `before`, its `after`, its `red` list and its
prose as well as in `tests_last_seen`.

```bash
awk -v target=src/data/config.rs \
  '/^\[\[guard\]\]/ {if(n>0) c++; n=0}
   /^tests_last_seen/ {b=1; next} b && /^\]/ {b=0; next}
   b && /file *=/ && index($0, "\"" target "\"")>0 {n++}
   END {if(n>0) c++; print c+0}' guards/guards.toml
```

**Every record naming a file this phase touches was in step with the tree on
2026-09-12.** Checked by comparing each record's written count against a count
taken by the check's own rule, which counts lines that are exactly `#[test]` or
`#[tokio::test]` after trimming. `config.rs` 63/63, `wx_settings.rs` 0/0,
`feedback.rs` 41/41, `date_display.rs` 37/37, `occurrences.rs` 62/62,
`one_question_at_a_time.rs` 19/19, `house_style.rs` 69/69,
`checkbox_labels.rs` 1/1. So this phase starts from a clean baseline, and the
first commit adding a test to any of them will flag its records and print the
`scripts/guards.sh --remeasure` command. Running that when a commit prints it is
not optional.

**Do not use a naive grep to take those test counts either.** `grep -c '#\[test\]'`
answers 76 for `tests/house_style.rs` against the check's 69, because that file
quotes the attribute inside string literals.

## The collision with phase 7, which shrank to almost nothing while this was written

`06-RESEARCH.md` calls the shared use of `src/data/config.rs` "the single largest
risk to this phase". **It is now measured gone, and it went two ways.**

**Plan 07-05 merged at `d83bed3` and changed nothing inside
`mod every_setting_is_acted_on`.** Its own summary says so in a section addressed
to this planner: not one of the four tests, not `stored_setting_names`, not
`files_that_act`, and not one of the three exception lists.
`OFFERED_BY_ANOTHER_SCREEN` still holds three entries,
`NOT_ANYTHING_ANYBODY_CHOOSES` two and `STORED_AND_OFFERED_BY_NOTHING` one.
Verified here rather than taken from the summary, 2026-09-12 at `b8857bc8`:

```bash
grep -n 'const OFFERED_BY_ANOTHER_SCREEN\|const NOT_ANYTHING_ANYBODY_CHOOSES\|const STORED_AND_OFFERED_BY_NOTHING' src/data/config.rs
```

**Phase 7's four remaining plans name neither file.** Read out of their own
frontmatter, which is the half a same-wave overlap check reads:

```bash
for p in 06 07 08 09; do
  awk '/^files_modified:/{f=1;next} f&&/^[a-z_]+:/{f=0} f' \
    .planning/phases/07-installing-updating-and-what-is-stored/07-$p-PLAN.md
done
```

Neither `src/data/config.rs` nor `src/presentation/wx_settings.rs` appears. The
only mention of either anywhere in those four documents is one line of 07-09
quoting a guard-record cost figure, not a file it edits.

**What is left of the collision, and it is real.**

| File | Phase 6 plan | Phase 7 plan |
|---|---|---|
| `src/presentation/wx_app.rs` | 06-05 | 07-09 |
| `tests/house_style.rs` | 06-04, possibly 06-07 | 07-06 |
| `guards/guards.toml` | 06-01 to 06-08 | 07-07, 07-08, 07-09 |
| `docs/changelog.md` | seven of the eight | all four |
| `.planning/WINDOWS.md` | 06-08, and each plan's own entries | all four |

`.planning/WINDOWS.md` was being written by a phase 7 executor while this README
was written: the file moved from 322 entries to 324 between two reads twenty
minutes apart. Add entries through `gsd-tools windows` and never by typing into
the table, because the ledger has a JSON half and
`test_both_halves_of_the_ledger_say_the_same_thing` holds the two together.

**What to re-take rather than re-read, if phase 7 lands more before this phase
starts.** Every line number these plans give for `src/presentation/wx_app.rs` and
`tests/house_style.rs`, the record count for those two files, and
`Cargo.toml`'s version. Nothing in criterion 1's model, criterion 2's dates or
criterion 3's scan appears in phase 7's plans at all.

**The decision that remains is scheduling and it is Pratik's**, carried at a
checkpoint in 06-04. It is much narrower than the research's version of it.

## Three things in the research that were wrong when a plan went to use them

Listed here as well as in each plan, because each would have become a false
premise.

**1. The suggested shape for the `Event::ALL` guard does not close the hole.**
The research says the missing guard "can be written as an exhaustive match from
each variant to a unit, or by asserting `ALL.len()` against a count derived from
the source". The first of those cannot work. An exhaustive match forces a new
variant to be answered, but nothing can call that match for a variant that is
not in `ALL`, because `ALL` is the only enumeration of the variants that exists.
A seventeenth variant given an arm in a fourth match is exactly as invisible as
one given an arm in the three that already exist. Only two mechanisms really
close it: generate the enum and `ALL` from one list, which makes the disagreement
unrepresentable, or read the source and count, which makes it detectable and
needs a companion proving the reading can see a violation. 06-01 takes the first,
on the `menu_ids!` precedent at `src/presentation/wx_app.rs:74`, whose own doc
comment gives the same argument about a numbering nobody should be doing by hand.

**2. The recommended panel has four checkboxes and the criterion asks for
three.** The research recommends "a Choice listing the sixteen events by
`Event::text()`, four checkboxes below it". Criterion 1 says Speech and Braille
"are set **together**, as one choice". Four boxes would rebuild, per event, the
exact false control the criterion was rewritten to remove. 06-02 builds three.

**3. The research does not notice that the global pair is the same defect,
today, on the same tab.** `build_feedback_tab` iterates `Channel::ALL` and builds
one checkbox per channel, so the Feedback tab offers Speech and Braille as two
independent boxes. Verified 2026-09-12 at `src/presentation/wx_settings.rs:2160-2169`.
Two boxes, four states, and only two outcomes, because `src/presentation/accessibility.rs:247-255`
releases the notification when either is on. The roadmap already assigns this
here: "Correcting those three false sentences belongs to phase 4.2; offering the
honest control belongs here." Phase 4.2 corrected the changelog sentences and
left the two boxes standing, which is checkable:

```bash
grep -n 'for channel in Channel::ALL' src/presentation/wx_settings.rs
```

So 06-02 owns the global pair as well as the per-event controls, and the phase is
larger than the research says by one task.

## What Pratik is asked, and where

Twelve decisions, one checkpoint each, each in front of the evidence at the
moment it is needed. None is answered here and none is answered in a plan;
the five struck through were answered by Pratik, or dissolved, and the plan
records his words.

| Decision | Plan | What it changes |
|---|---|---|
| ~~1. What relative wording does in a non-English locale~~ | 06-03 | **Answered 2026-09-13: option 3, widened.** Real plural rules through Project Fluent, as the first piece of version 2, in Pratik's words "3 + the start of real internationalization". The audit the answer was gated on is in 06-03's `<package_legitimacy_audit>`, and the package was confirmed the same day. See "Version 2 starts here" below |
| ~~2. The panel's shape, against the comment that argues for no panel~~ | 06-02 | **Answered 2026-09-12: option 1, a Choice with three controls and a reset button beneath it.** Not option 4's extra row setting one answer for everything, whose state is ambiguous when the sixteen disagree; it costs one row to add later if a listening pass says the sixteen trips are the real problem |
| ~~3. Whether a per-account Allow Changes answer is three or one~~ | 06-04 | **Answered 2026-09-13: all three**, the recommended option, one box per answer in `Allowed`, each able only to narrow |
| ~~4. Whether a reminder waits for typing to stop~~ | 06-05 | **Answered 2026-09-14: option 3, widened.** "3 + reminder tones every minute until the user focuses on the reminder window which should allow for snooze, snooze all, dismiss, and dismiss all, keeping in mind that there may be multiple reminders in the window. Similar considerations should occur for Calendar and task reminders." The hold, the tone and the sentence stay in 06-05; the window with every kind in it is 06-09 |
| 10. What time of day a date-only task, or an all-day event's alert, is due | 06-09 | A task's due date is a date on purpose. Start of the day, the stored working-day start, a fixed hour, or a new setting. Recommended: the working-day start, already the person's own |
| 11. Whose alert an event follows | 06-09 | Not two stores but one column whose absence means three things: Google's calendar default, Microsoft's off, CalDAV's never read. The column only, the program's default for everything, the column else the default, or make the absence say which. Recommended: the column only, with the last as the way the silent ones arrive |
| 12. Where a snoozed task or event goes | 06-09 | Added by the planner. Its row cannot move, so the hold lives for the session or in a small additive table. Recommended: the table |
| 5. Whether to widen the scan's target list | 06-06 | Eleven windows are scanned and at least nine more dialogs exist |
| 6. Whether to pin the Axe.Windows CLI | 06-06 | Whether the coverage list is a claim about something reproducible |
| 7. Whether `REQUIREMENTS.md` is corrected in place | 06-07 | Three of its evidence lines are wrong and this phase disproves one of its figures |
| 8. Whether the coverage list is a document or a check | 06-07 | A document goes stale the way the five findings did; a check costs records and needs a companion |
| ~~9. How phases 6 and 7 share the tree~~ | 06-04 | **Dissolved 2026-09-13**: phase 7 finished on 2026-09-12 with all nine plans merged before 06-04 started, so there was nothing to share with. Recorded by 06-04 as the question going away rather than as an answer chosen |

Each checkpoint carries a recommendation rather than a menu, and says what each
option costs. A decision put without a recommendation is a list, not a question.

## Version 2 starts here: Project Fluent, chosen 2026-09-13

Written for the planner of the next milestone, so the audit is not run twice.
The full audit, with the three options that lost and what each lost on, is in
`06-03-PLAN.md` under `<package_legitimacy_audit>`, and the decision is
recorded under FEEDBACK-02 in `.planning/REQUIREMENTS.md`. This section is the
direction and the measurement, not a plan.

**What was decided.** Translation support for the interface and the screen
reader speech starts in version 2, on Project Fluent: `fluent-bundle` 0.16 and
`unic-langid` 0.9, with `fluent-langneg` and `intl-memoizer` as direct
dependencies from the same closure. Decided by Pratik on 2026-09-13 at 06-03's
checkpoint, confirmed the same day. Not ICU4X, which has no message system and
a floor that moves in minor releases; not `intl_pluralrules` alone, which is one
primitive where a message system is needed; not a hand-written table, which is
a private copy of an open standard.

**Why Fluent, for this project in particular.** A Fluent message carries
attributes, so a control's visible label, its accessible name and its keyboard
mnemonic are one translatable unit. That is the shape of what this project does
with a label plus `set_accessible_name`, and it is the answer to the problem
nobody meets until the first translation: an `&` mnemonic has to be a letter in
the translated label. No other candidate is designed for an interface that is
heard.

**What 06-03 leaves behind as the seed.** `locales/en-US/dates.ftl`, four
messages, in a layout chosen for five thousand: `locales/<locale>/<area>.ftl`,
ids `<area>-<meaning>` in lower-case kebab with the area being the file's name,
variables named for the thing they count, and the attribute names `.label`,
`.accessible-name` and `.mnemonic` reserved in the file's header for the first
control that is migrated. The loader is `src/common/catalogue.rs`: compiled in
through `include_str!`, parsed once, a `Sync` bundle in a `OnceLock`, one
private function applying the three settings Firefox's production use of these
crates requires, a message-id list generated from one declaration on the
`menu_ids!` pattern, and a completeness check that holds both directions. The
bundle's locale is the catalogue's language, never the machine's, because
plural rules select on it and English text under Russian rules writes "21 day
ago"; the machine's locale chooses the catalogue through `fluent-langneg`, and
with one catalogue every machine gets English, silently, which is criterion 2's
own fallback clause.

**What the tree measurement found about adoptability, 2026-09-13 at
`50d41d75`.** Roughly 5,300 user-facing string occurrences by the audit's
count. Three reasons the migration is easier than that number suggests, each
with the command that re-derives it in the plan's audit block: the dominant
idiom is one `format!` per message with the whole sentence in the literal, so
the sentence is already the unit (3,476 `format!` sites); the tree has already
started a catalogue by hand, 487 named string constants with names like
`NO_JUNK_FOLDER_FOUND`, plus 38 `spoken()` and 46 `say`/`said` functions that
are the seams a catalogue call slots into; and the hostile patterns are about
220 sites of 5,300, around 4%, with three helpers carrying most of them. Two
places need rewriting rather than adapting: `src/application/summing_up.rs`
joins fragments with `", "` and pushes a full stop, English punctuation in an
application-layer type with 13 callers in 12 files, and `caldav::how_many` at
`src/service/caldav.rs:871` is an English plural helper with 40 call sites,
tests included.

**What the version 2 planner has to decide, and none of it is decided here.**

1. Which languages ship, and who writes them. 06-03 ships English only and
   proves the plural machinery with a Russian resource written inside a test.
   No executor writes a translation in a language they do not speak and calls
   it shipped; that is Pratik's to commission.
2. Whether the prose guards walk `locales/`. `ours()` in `tests/house_style.rs`
   collects `src`, `docs`, `.planning`, `tests`, `scripts`, `guards`,
   `installer` and `.github`, and not `locales`, so the catalogue sits outside
   every rule about em dashes and empty words. Four messages do not need it;
   five thousand do, and adding the directory touches a file fingerprinted by
   21 records.
3. Retiring the second reader of `LOCALE_SNAME`. `spellcheck::system_language`
   at `src/service/spellcheck/mod.rs:335` reads it through `GetLocaleInfoW`, and
   06-03's task 3 adds a reader in `src/common/` through `GetLocaleInfoEx`
   because `common` cannot reach `service`. Two readers of one constant, in a
   file fingerprinted by 30 records; the retirement waits for a plan that is
   in that file anyway.
4. The order of migration. The audit's own suggestion: the six relative-date
   sites first, which 06-03 does; then the 40 through `caldav::how_many`; then
   the inline count branches; then the rest, which the catalogue makes
   addressable without a second migration.
5. Numbers follow the catalogue's language, not the machine's, because a
   formatter can only learn its bundle's locale. Today that is a distinction
   without a difference; on a French machine with a French catalogue it is the
   same locale. The day a machine's locale and its catalogue's differ for a
   number over 999 is the day this is revisited.

## Version 2's second seam: a message somebody asked to be told about later

Written 2026-09-14 for the planner of the next milestone, the way the Fluent
section above was written the day before. This is the direction and the
shape, not a plan, and nothing in version 1 builds any of it.

**What Pratik asked for.** "We also need to plan for future expansion of this
functionality (post version 1) which will enable us to set notifications for
mail that will let the user decide to tackle individual mail later on." A
mail message, chosen by the person, with a moment attached, arriving in the
same window as a reminder, a task and an event, as a row that says "Mail:"
first.

**What 06-09 shapes for it, and where the shaping is.** `Due` in
`src/application/due.rs` gains a `Kind` with three variants and an `Identity`
of a kind and an opaque id string composed by the kind's own feed. The fourth
kind is an addition: a variant, and then the compile errors it produces, which
are exactly the places a mail row has to answer for. `Kind`'s doc comment
names the three places 06-09 knows it assumed the three kinds that exist,
which is the same table the notes seam keeps at
`docs/development/the-notes-seam.md:689-702`, one level down. `ItemKind::Mail`
already exists at `src/application/new_item.rs:39-46`, so the word is in the
tree. The hold table `held_alerts` stores the kind as a word, so a fourth
kind is a fourth word and not a schema change, and a word this build does not
know is kept and ignored on `AddressBook::Other`'s reasoning. If 06-09's
checkpoint chose the session map instead, this paragraph is wrong about the
table and the summary says so.

**What the mail kind has to answer, in the order the compiler asks.** The
sentence: what a row says, with "Mail" first, and what "late" means for a
message somebody meant to come back to. Whether done means something: it
does, and it is clearing whatever marked the message, which is a writer the
mail kind brings with it. The alert instant: the moment the person chose,
which needs a stored row of its own, message identity and moment, and that
row is the mail kind's own table rather than a column on `messages`. The
identity: whatever the mail feed composes, and the notes seam's rule holds,
nothing outside the feed and the writer takes it apart; the message cache's
row id is one candidate and the account, folder and uid are another, and
which survives a folder move is the first thing the version 2 planner has to
find out. The snooze: through `held_alerts` like a task, or by moving its
own row like a reminder, and the choice is the same one 06-09's premise 8
makes for the three kinds by asking what the time means.

**What is decided and does not need re-deciding.** One window, every kind a
row, the kind said first: Pratik's. The tone until focus and its ceiling:
06-05's. The identity is opaque and composed by the feed: 06-09's, on the
notes seam's precedent. Nothing here builds a switch, a setting, a menu item
or a table for mail; PIM-08's third criterion applies word for word, "a
switch that does nothing is the failure this project has fixed repeatedly".

**Where it is recorded besides here.** 06-09's task 4 adds a row to the `## v2
Requirements` table in `.planning/REQUIREMENTS.md` pointing at this section
and at `Kind`'s doc comment, because none of FEEDBACK-01 to 03 is about a due
window and inventing a requirement id is not a plan's to do. No requirement
id is owed until the next milestone's requirements are written.

## What no plan in this phase can close

Ten things. Each becomes one `unrun-verify` entry in `.planning/WINDOWS.md`,
written by the plan that meets it, one entry per unrun thing rather than one
entry for all of them. `.planning/WINDOWS.md` stood at 324 entries with 302 open
when this was written, and a phase 7 executor was adding to it at the time, so
read its last row rather than that number.

1. **Whether the per-event panel is usable.** Whether a Choice of sixteen with
   three controls beneath it reads well; whether the controls reloading under the
   cursor when the Choice changes is announced or silent. That is a live-region
   shaped problem and only a listening pass settles it.
2. **Whether the effective-versus-chosen distinction is understood.** The
   sound-only fallback means the panel explains something subtle. No test can
   tell whether the explanation lands.
3. **Whether the new controls are named on both channels in practice.** A test
   can prove a label was set. Only Narrator proves UI Automation reads it and
   only NVDA proves MSAA does.
4. **Whether the honest global control reads as a loss.** Somebody who ticked
   Braille and not Speech meets one box where there were two. Whether that reads
   as an explanation or as a feature being taken away is a person's judgement.
5. **Whether localised dates sound right.** A French month read by a French voice
   in a date whose order came from the same machine is what FEEDBACK-02 exists
   for. It needs a French Windows, a French NVDA voice and somebody who speaks
   French.
6. **Whether the genitive really arrives.** Microsoft documents that it does.
   Nothing here has ever called `GetDateFormatEx` on a Russian or Polish locale.
7. **Whether the sixteen earcons are distinguishable.** Already disclosed in the
   product, still unmeasured, and a per-event panel makes it more likely somebody
   switches sounds on for all sixteen.
8. **The findings the scan produces.** Each has to be looked at and judged: ours,
   or upstream with a named issue. The scan produces the artifact; the judgement
   is a person's, and 06-08's checkpoint is where the artifact arrives.
9. **The scoped manual list.** Criterion 4 asks for the list of interactions only
   a human pass can walk. That is a judgement about what matters, not a
   derivation.
10. **Every WCAG criterion an automated scan cannot judge, which is fifty-two of
    the fifty-five.** The coverage list 06-07 writes is the record that this is
    so. Writing it down is not testing it.

Added 2026-09-14 with the rewrite of 06-05 and the arrival of 06-09:

11. **Whether a reminder said at once and its window a minute later reads as
    help or as the same thing twice**, and whether a tone once a minute for ten
    minutes reads as being looked after or as being nagged. 06-05 builds it;
    only a listening pass judges it.
12. **Whether a screen reader speaks the reminder's sentence while another
    application is in front.** `UiaRaiseNotificationEvent` does not move
    focus, and NVDA speaks notifications from the foreground process, so
    somebody in Word may hear only the tone. That is why the tone repeats, and
    nothing here can confirm it.
13. **Whether "Task due today" at the working-day start reads as help**, and
    whether a person with a Google calendar notices that events on the
    calendar's default alert do not alert here, under 06-09's decisions 10 and
    11 as recommended.
14. **Whether a person tabbing past a disabled Mark Done learns why**, on an
    event row, the same question 06-04 ledgered for its disabled boxes.

**Pratik does the manual and screen reader testing after phase 8.** So these are
planned to be recorded rather than performed. Criterion 4's second clause, "each
of the five WebView2 findings is either fixed or recorded as upstream with the
upstream named", is the one criterion here whose closure needs a CI run that a
person triggers, and 06-08 carries a `checkpoint:human-action` for exactly that
rather than pretending an executor can do it.

## Costs every plan is written around

**Guard records: 729 on 2026-09-12 at `b8857bc8`, and 755 on 2026-09-14 at
`562b4b90`.** This was 720 on 2026-09-12 at `febe8e4`, and 632 when phase 5.1
was planned. Re-take the count rather than quoting it, and count records
rather than mentions, with a TOML reader rather than the `awk` above where a
record might be written as an inline table.

**The per-file figures, taken the same way on the same day.** The rows dated
2026-09-14 were taken at `562b4b90` for 06-05's rewrite and 06-09.

| file | records | `#[test]` today |
|---|---|---|
| `src/presentation/wx_app.rs` | 48 | not counted, see below; 199 on 2026-09-14 |
| `tests/wired.rs`, 2026-09-14 | 14 | 69 |
| `src/data/message_cache/mod.rs`, 2026-09-14 | 11 | 23 |
| `src/common/catalogue.rs`, 2026-09-14 | 6 | 14 |
| `src/presentation/date_display.rs`, 2026-09-14 | 6 | 42 |
| `src/application/due.rs`, 2026-09-14 | 4 | 23 |
| `src/data/message_cache/tasks.rs`, 2026-09-14 | 4 | 21 |
| `src/presentation/wx_reminder_alert.rs`, 2026-09-14 | 0 | 1 |
| `tests/house_style.rs` | 19 | 69 |
| `src/presentation/wx_account_manager.rs` | 6 | 13 |
| `src/data/config.rs` | 5 | 63 |
| `src/presentation/ui_types.rs` | 4 | 78 |
| `src/presentation/wx_settings.rs` | 3 | 0 |
| `src/presentation/date_display.rs` | 3 | 37 |
| `src/application/occurrences.rs` | 3 | 62 |
| `src/service/signed_mail.rs` | 2 | 112 |
| `src/presentation/accessibility/feedback.rs` | 1 | 41 |
| `src/presentation/one_question_at_a_time.rs` | 1 | 19 |
| `src/presentation/wx_item_form.rs` | 1 | 14 |
| `tests/checkbox_labels.rs` | 1 | 1 |
| `src/presentation/scan_target.rs` | 0 | 6 |
| `src/presentation/accessibility/sound_scheme.rs` | 0 | not counted |
| `tests/theme_reach.rs` | 0 | 3 |
| any new file under `tests/` | 0, and needs one written | 0 |

**Where a test goes is a cost decision here, and two plans turn on it.** 06-05
puts its rule in `src/presentation/one_question_at_a_time.rs` at 1 record rather
than in `src/presentation/wx_app.rs`'s test module at 48, which is why
`what_to_raise` takes its conditions as arguments. 06-02 puts its live check in a
new file under `tests/` at 0 records rather than in
`src/presentation/wx_settings.rs`, which has 3 records and no tests at all, so
the first test there newly couples three records to a count that will then move
on every later commit.

**A companion written into a file no record fingerprints is the technique 07-05
found and it is copied here.** That plan homed its hand-named settings companion
in `src/service/update_check.rs` for that reason. 06-02 and 06-07 do the same.

**Land every test for one file in one commit.** The count check fires per file,
not per test, so four tests in `config.rs` cost the same remeasure as one.

**Guard sweeps are not on any plan's critical path.** By the decision of
2026-09-03, one sweep once every phase is complete, and phase 8's criterion 5
owns it. No executor here runs `scripts/guards.sh` except the scoped
`--remeasure` a commit prints. **No plan here quotes a sweep duration**, for the
reason `CLAUDE.md` gives at length: the record count and the per-record rate both
move, the rate roughly halved on 2026-09-09, and the whole-tree total has been
quoted at 15, 16 and 20 hours at different counts.

**`cargo test` takes one `--lib`, and the form these plans use was checked rather
than copied.** `CLAUDE.md` records that 55 plans told executors to run
`cargo test --lib a:: --lib b::`, which cargo refuses, and that none of them was
ever run. The form used here is one `--lib` with several filters after `--`:

```bash
cargo test --lib -- presentation::accessibility::feedback:: presentation::date_display::
```

Run 2026-09-12 at `b8857bc8` against two real test paths, it compiled once and
ran both, reporting three tests passed and 7,075 filtered out. The `&&`-joined
form `CLAUDE.md` gives is also correct and costs a second build.

**The library holds 7,075 tests** as of that run. `CLAUDE.md` quotes 6,720 at
`10effea` on 2026-09-09 and 5,837 on 2026-08-30. Take it again rather than
quoting any of the three.

**No plan here touches `Cargo.toml` except for a version bump beside the change
that needs it.** Nothing in this phase adds a dependency, so
`scripts/which-checks.sh` answers `affected` on every source commit and
`docs_only` on the document commits. One exception is live: if decision 1 goes to
real plural rules, that is a new crate, a `dependency-audit` conversation and a
`Package Legitimacy Audit` that `06-RESEARCH.md` explicitly does not carry. The
checkpoint in 06-03 says so.

**Corrected 2026-09-13: the exception went live.** Decision 1 went to real
plural rules, so 06-03's task 3 adds four lines to `[dependencies]`, and the
commit that does answers `all` and pays the whole gate. The audit
`06-RESEARCH.md` did not carry is now in 06-03 as `<package_legitimacy_audit>`.
Two things that commit has to know, both found by reading rather than by the
plan that first described it: `service::outward::tests::test_every_dependency_has_been_told_apart_from_a_way_out`
reads the manifest and refuses a dependency on neither of its two lists, so the
four crates are classified in `src/service/outward.rs` in the same commit; and
the manifest rule in `scripts/which-checks.sh` is reached only after the `red`
answer, so a red commit carrying the manifest runs scoped rather than `all`,
which is why the manifest goes in its own commit ahead of the red.

**A commit touching `.github/workflows/*.yml` answers `affected` and selects no
tests at all.** Verified 2026-09-12 by reading `scripts/which-checks.sh:325-338`
and `scripts/check.sh:387-432`: the first answers `affected` for anything that is
not `*.md` or `*.txt`, and the second maps only `src/*.rs` and `tests/*.rs` to
targets. So the two tests in `src/presentation/scan_target.rs` that read
`.github/workflows/accessibility.yml` do not run on a commit that changes it.
06-06 closes that, and it is the gate this phase would otherwise walk into twice.

**A red commit is allowed only on a branch**, must name every failing test in
`Fails-until-green:` trailers at column 0, and is held to three things at once:
every named test ran, every named test failed, nothing else failed. Where adding
a test makes `test_every_guard_record_says_how_many_tests_the_files_it_names_held`
red at the same time, name that check as one of the failures, in the same commit,
with the reason in the message.

**Never pipe `scripts/check.sh` into anything whose exit status is then read.**
Run `scripts/check.sh all` once before a merge, and do not assume the hook
already ran it.

**No em dashes anywhere under `.planning` or `docs`, and no carriage returns
anywhere this project writes.** `tests/house_style.rs` walks `.planning` for
markdown since 2026-09-07, and its carriage-return check reads the same set.
There were zero em dashes under `.planning` before these files were written and
there are none in them. Checked by byte search rather than by eye: zero `\r`,
zero em dashes and zero en dashes across all nine files.

**These eight files cannot be committed until the roadmap's progress row for
phase 6 says `0/8`.** That is a correction to what an earlier draft of this
README said, and it was caught by running the check rather than by reading it.

```bash
cargo test --no-fail-fast --test house_style --test the_planning_files_agree_with_themselves
```

Run 2026-09-12 at `b8857bc8` with the eight plans on disk: 14 passed, 2 failed,
both of them `the_planning_files_agree_with_themselves`, and both about one row.
`test_the_roadmap_counts_the_files_that_are_on_disk` reads the progress table in
`.planning/ROADMAP.md` against the phase directories, and
`test_the_roadmap_reading_can_see_a_row_that_disagrees_with_the_disk` is its
companion, which refuses to run while the tree already disagrees and says so in
its own message. That target is one of the four whole-tree guards `scripts/check.sh`
runs on every commit in every mode, so this is a blocking failure and not a
warning.

What an earlier draft of this README checked was `.planning/STATE.md`, and it was
right about that: `Total Plans in Phase` is read for the phase the frontmatter
names, which is 7, and `completed_plans` counts `*-SUMMARY.md`, which these files
do not add. It then concluded that committing was safe, having read two of the
checks in that file and not the third. The roadmap row is the third.

**The row was already moving while these plans were being written, which is worth
seeing.** It reads today:

> | 6. How the application speaks | 0/3 | Planning in flight | Not `TBD` any more,
> because plans are on disk and the check that reads this row treats `TBD` as
> honest only while a phase has none. The 3 is the plan files present on
> 2026-09-12 while phase 6's planning was still running, and it had already moved
> twice while 07-06 tried to commit past it, so it belongs to whoever is doing
> that work rather than to this number

So a phase 7 executor met this the hard way, twice, and wrote a number that was
correct for the minute it was written. `0/8` is the answer once these eight files
are final. Nothing in this phase edits `.planning/ROADMAP.md`, by instruction,
so it belongs to whoever commits these.

**The same gate, on 2026-09-14, for the ninth file.** The row reads `4/8`
today and has to read `4/9` in the commit that adds `06-09-PLAN.md`, or
`test_the_roadmap_counts_the_files_that_are_on_disk` refuses it. And a second
check in the same target reads `.planning/STATE.md`:
`test_the_state_file_counts_the_plans_that_are_on_disk` at
`tests/the_planning_files_agree_with_themselves.rs:813` compares `Total Plans
in Phase`, which says 8 at `STATE.md:686`, against the phase directory. Both
move in the same commit, and `progress.total_plans` at `STATE.md:14` says 97
against 97 `*-PLAN.md` files on disk before the ninth and 98 after; nothing
checks that field, so it is owed rather than blocking. Neither file is edited
by the planner, by instruction.

## Estimates, and why the numbers are small

`raw_tokens` is 30,000 per work task, which is the projection shape phase 7's
plans used. `tokens` is that multiplied by **0.15**, which is the mean of
`actuals.tokens / estimate.raw_tokens` over phase 7's five landed plans: 0.128,
0.102, 0.134, 0.135, 0.228. Read 2026-09-12 from each plan's `estimate` block and
each summary's `actuals` block.

**Phase 7's own estimates were between four and ten times too high**, and the
factor those plans used was about 1.2 rather than a measured one. Nothing was
wrong with the projections as projections; the factor was never calibrated
against an actual. Five samples spanning 2.2 times is `low` confidence, not
`med`, and the number is derived from the sample count rather than self-rated.

Checkpoints are not counted as tasks in the projection, because a checkpoint is a
block that waits rather than a block that works.

Re-derive the factor once this phase has actuals of its own. Five samples from
one phase is a thin basis and the plans say so.

**Re-derived 2026-09-14 for 06-05's rewrite and 06-09, from nine samples.**
Phase 6's four landed plans give `actuals.tokens / estimate.raw_tokens` of
0.063, 0.239, 0.351 and 0.124; with phase 7's five the mean over nine is
**0.17**, and the spread is 5.6 times, so `low` still. The one four-task plan
in the sample, 06-03, ran at 0.351, and 06-09 is four tasks with a checkpoint
in the middle on the same shape, so read its 20,000 as the mean and 42,000 as
what the nearest precedent cost. `gsd-tools estimate-calibration` reports no
samples, because it reads a field these summaries do not write; the figures
above were read from each summary's `actuals` block by hand.

## What is owed to documents, and belongs to whoever lands these

1. **The roadmap's progress row for phase 6 has to say `0/8`, and the gate is red
   until it does.** This is the blocking one. See the paragraph above for the
   measurement and for the correction to what this README first said about it.
   Not written here on purpose: `.planning/ROADMAP.md` and `.planning/STATE.md`
   are being edited by phase 7's executors in parallel and were moving during
   this work. **On 2026-09-14 the same row has to move from `4/8` to `4/9`,
   `STATE.md`'s `Total Plans in Phase` from 8 to 9, and its
   `progress.total_plans` from 97 to 98, in the commit that adds 06-09**, and
   the roadmap's phase 6 plan list wants a ninth line.
2. **The roadmap's phase 6 entry**, which today says `**Plans**: TBD`. It wants
   the count and the eight-line plan list, in the shape phase 7's entry uses.
   Nothing checks that line, so it is owed rather than blocking.
3. **`.planning/STATE.md`'s `progress.total_plans`** says 92 and there are 89
   `*-PLAN.md` files on disk today, which these eight will make 97. Nothing
   checks that field against the tree: `Total Plans in Phase` is read for the
   phase the frontmatter names, which is 7, and `completed_plans` counts
   `*-SUMMARY.md`, which these files do not add. Checked rather than assumed, by
   reading `tests/the_planning_files_agree_with_themselves.rs:343-413` and then
   by running the target. The field is still wrong and whoever owns the merge
   should correct it by counting rather than incrementing.
4. **Nothing in this phase corrects a requirement ahead of the code.** Where a
   requirement wants rewording after a plan makes it true, the plan that made it
   true does the rewording. Decision 7 in 06-07 is about the requirement evidence
   lines that are wrong about the code as it stands today, which is a different
   question.

---
phase: 06-how-the-application-speaks
plan: 03
status: complete
subsystem: ui
tags: [localisation, win32, dates, accessibility, nls, getdateformatex, fluent, plural-rules, i18n]

requires:
  - phase: 05-the-other-five-modules-keep-up
    provides: "`date_display` as the one place every module gets its date wording from, and `ENGLISH_ONLY` as the sentence that discloses what it cannot do"
provides:
  - "`src/common/how_the_machine_writes_dates.rs`: one place that asks Windows how a date is written, with two entry points because a month inside a date and a month on its own are different words; after task 4 also the seven day names, the locale's own name, and a number the way that locale writes one"
  - "`src/common/catalogue.rs` and `locales/en-US/dates.ftl`: the first translation catalogue, Project Fluent, four messages, a loader shaped for five thousand, and the three settings Firefox ships these crates with applied in one place"
  - "Every shipping site that writes a month or a day name asks the computer: `date_display`'s three readings, the appointment form's month list, the repeat-series sentence and the eight signature sentences. A source-reading guard with a working companion holds it"
  - "A settings sentence that says the dates follow this computer and the wording around them does not yet"
affects: [06-04, 06-05, 06-06, 06-07, 06-08, version-2]

actuals:
  tokens: 42100
  tasks: 4
  commits: 12

tech-stack:
  added: [fluent-bundle 0.16, fluent-langneg 0.13, intl-memoizer 0.5, unic-langid 0.9]
  patterns:
    - "A private `_asking` twin beside each public reading, taking the locale, so a test forces `en-US` and asserts about this code rather than about the machine it ran on. Now in `date_display`, `occurrences` and `signed_mail`"
    - "An enum of four shapes rather than a free-text date picture, because Windows does not refuse a wrong picture: asked for `ZZZZ` it answers `ZZZZ`"
    - "One private function builds every bundle, shipped or test, and applies the three settings, so a Russian resource in a test is a test of the shipping path's settings"
    - "A source-reading guard exempts the exact literal, never the file, with a reason beside each, and asserts each allowance is still in the tree so the list cannot outlive its subjects"
    - "A message-id list generated from one declaration, on the `menu_ids!` pattern, so the typed ids, the catalogue strings and the `ALL` list a check walks cannot drift apart"

key-files:
  created:
    - src/common/how_the_machine_writes_dates.rs
    - src/common/catalogue.rs
    - locales/en-US/dates.ftl
  modified:
    - src/presentation/date_display.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/ui_types.rs
    - src/presentation/read_aloud.rs
    - src/application/occurrences.rs
    - src/service/signed_mail.rs
    - src/service/outward.rs
    - src/common/mod.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Two Win32 mechanisms, not one: `GetDateFormatEx` with a picture for a date that has a day in it, `GetLocaleInfoEx` with `LOCALE_SMONTHNAME1` to `12` for a list that has none, and `LOCALE_SDAYNAME1` to `7` for a day name, which always stands alone"
  - "The wrapper lives in `src/common/`, because `src/service/signed_mail.rs` needed it and `src/service/` reaches `presentation` nowhere; measured again after task 4, still nowhere"
  - "Project Fluent for the relative wording, decided by Pratik on 2026-09-13 as the first piece of version 2, with the audit in the plan. Four manifest lines, no fifth: the reverse completeness check reads message ids off the catalogue text rather than through `fluent-syntax`'s entry type"
  - "The bundle's locale is the catalogue's language, never the machine's, because plural rules select on it; the machine's locale chooses which catalogue, through `fluent-langneg`, and with one catalogue every machine gets English"
  - "An unparseable locale name and a failed read ask for English by name, not `und`, because negotiation happens to turn `und` into English today and would not with a second catalogue"
  - "A third `WhichLocale` variant, `ACatalogueIsWrittenIn`, because the number formatter's locale comes from the bundle and calling that `NamedInATest` in shipping code would be a lie about where the value came from"
  - "The source-reading guard allows six literals by name rather than five files: two wire formats and four interface labels that version 2 translates"
  - "`ENGLISH_ONLY` narrowed for the second time and kept: the dates follow this computer, the wording around them does not yet"

requirements-completed: [FEEDBACK-02]

coverage:
  - id: D1
    description: "A date written by this program carries the month name this computer uses, in the form a date puts a month in"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib presentation::date_display::tests::test_a_date_and_a_month_heading_name_the_same_month_differently"
        status: pass
    human_judgment: false
  - id: D2
    description: "The stored order and wording still decide the shape; the machine supplies only the words"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib presentation::date_display::tests::test_the_four_stored_combinations_still_read_as_they_always_did"
        status: pass
    human_judgment: false
  - id: D3
    description: "The appointment form's month list carries the twelve names this computer uses, in their standalone form"
    requirement: FEEDBACK-02
    verification:
      - kind: other
        ref: "read at src/presentation/wx_item_form.rs:849; no test reads the Choice. WINDOWS.md 361"
        status: unknown
    human_judgment: true
    rationale: "The Choice is built inside a closure that needs a live parent window; nobody has opened the form and looked"
  - id: D4
    description: "Relative wording comes out of a translation catalogue through real plural rules, proven in Russian and Polish on an en-US machine, with no isolation marks, numbers by Windows, counts as numbers and the error list read"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib -- common::catalogue::"
        status: pass
    human_judgment: false
  - id: D5
    description: "A computer whose language has no catalogue hears English, silently, forced rather than read off the machine"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib presentation::date_display::tests::test_a_computer_whose_language_has_no_catalogue_hears_english"
        status: pass
    human_judgment: false
  - id: D6
    description: "A repeat-series sentence names its days the way this computer does, and a signature sentence writes its date the way this computer does"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib -- application::occurrences::tests::test_a_french_computer_hears_french_day_names_inside_the_english_frame service::signed_mail::tests::test_a_french_computer_hears_the_signing_date_in_french"
        status: pass
    human_judgment: false
  - id: D7
    description: "No shipping file names a month or a day in English unless this computer does, held by a source-reading guard with a companion that sees a planted violation"
    requirement: FEEDBACK-02
    verification:
      - kind: unit
        ref: "cargo test --lib -- common::how_the_machine_writes_dates::tests::test_no_shipping_file_names_a_month_or_a_day_in_english common::how_the_machine_writes_dates::tests::test_the_reading_sees_a_planted_english_name_and_ignores_one_in_a_test_block"
        status: pass
    human_judgment: false
  - id: D8
    description: "Any sentence out of the catalogue, any day name, any signature date, heard by a screen reader in that language"
    requirement: FEEDBACK-02
    verification: []
    human_judgment: true
    rationale: "Only a Windows installed in that language, a voice in that language and a listener who speaks it can settle whether it reads well. WINDOWS.md 360, 364, 366, 368, 370, 371. Waits for the pass after phase 8"

duration: "tasks 1 and 2 about 3h across two sessions; tasks 3 and 4 about 1h 50m on 2026-09-13"
completed: 2026-09-13
---

# Phase 06 Plan 03: A month name is the machine's to say, Summary

Every date this program writes now follows the computer it runs on: month
names, day names, the order of the day and month, and the clock. "2 days ago"
comes out of a translation catalogue with real plural rules behind it, proven
in Russian and Polish on a machine that is neither, and nothing a user hears
has changed by it. All four tasks are merged. Criterion 2 closes structurally,
clause by clause, below; nothing in it has been heard.

**What works.** On a French computer a date reads "26 juillet 2026", a
repeating appointment reads "every week on mardi and jeudi", and a signature
sentence says it was signed on "28 août 2026". On a Russian one the month
inside a date is in the form a date puts a month in and the month in a list is
the form a list uses. "2 days ago" is "2 days ago" on every machine, because
only an English catalogue exists, and the note on the Reading tab says exactly
that.

**What does not.** The words around a date are English everywhere: "every week
on", "A timestamp says this was signed on", "2 days ago" itself outside
English. That is version 2's translation of the interface, and this plan is
where it starts, not where it finishes. Four interface labels still carry
English day and month names as examples and are allowed by name. And nobody
has heard any of it.

## The four tasks, and what each cost at the gate

| Task | Commits | What the gate answered | Notes |
|---|---|---|---|
| 1 | `733c0dce` red, `b857be6d` green, `a7f6346b` fix | `red`, `affected`, `affected` | the terminated executor's |
| 2 | `455f30ce` red, `082ab42e` green | `red`, `affected` | three tests really red where one was expected |
| checkpoint | answered 2026-09-13 | | option 3, widened; the audit is in the plan |
| 3 | `3c1291f0` build, `3fdb0706` red, `a0136102` green | **`all`**, `red`, `affected` | the manifest commit paid the whole gate on its own, as the plan predicted |
| 4 | `e6f90a84` red, `f924c7b6` green | `red`, `affected` | |
| merge | `39417f88` | `all` | green on the first attempt |

Tasks 1 and 2 are described in the sections that follow the four-task
narrative; they were written when the plan was partial and are kept as
written, with one correction noted where the tree later contradicted them.

## The fixed commit order behaved exactly as the plan predicted

The plan said the `build(06-03)` commit, carrying the four manifest lines and
the `outward.rs` classification alone, would answer `all` because `Cargo.toml`
changes by more than a version line, and that a red commit carrying the same
change would answer `red` and run scoped, leaving the install judged by
nothing. Measured: `scripts/which-checks.sh` answered `all` for the staged
build commit before it was made, the hook then ran formatting, clippy, the
script suites, the advisory check, 7,140 library tests and the release build,
and the green commit that followed answered `affected`.

What the gate said about the eight new packages, quoted from the run rather
than summarised: "All 4 advisory(ies) this project accepts are still
reported. No advisory outside .cargo/audit.toml, and nothing is being held
open." `cargo audit` on its own reported three warnings, all pre-existing and
none naming any of the eight: `lzw` unmaintained, `proc-macro-error`
unmaintained, `chacha20` yanked. `grep -c '^\[\[package\]\]' Cargo.lock` went
from 698 to **706**, the eight the audit predicted and no more.

**The first attempt at that commit was refused, and this time the detail was
kept.** The whole gate failed one integration target,
`a_move_says_what_has_not_been_sent`, the same target ledger 365 records
failing one run in three at task 2's merge with its text lost to a `tail`.
The text this time:

```
test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent ... FAILED
panicked at tests\a_move_says_what_has_not_been_sent.rs:217:10:
an account to look the folder up against: Security("Could not remove the
password for acct in the Windows credential store: Security error: Could not
reach the credential store: No default store has been set, so cannot search
or create entries")
```

That is not contention and not a live window. It is `keyring` 4.1.5's
`Entry::new`, read in the registry source: it guards its lazy initialisation
with a `compare_exchange` on an `AtomicBool`, the thread that wins sets the
default store, and a thread that loses goes straight to
`keyring_core::Entry::new` before the winner has finished. Ten tests start in
parallel on a fresh process, so it fires about one run in three. A second
finding fell out of the same read: the in-memory seam in
`src/service/secret_store.rs` is `#[cfg(test)]`, which an integration test
under `tests/` never sees, so this target reaches the real Windows credential
store on every run. Out of this plan's scope, recorded as ledger 374 with the
fix named, and the commit was retried and accepted.

## Task 3: "2 days ago" through a catalogue

### The measurement, taken 2026-09-13 on this en-US Windows 11 machine

`GetNumberFormatEx` with a fully set `NUMBERFMTW` and with a null one, for
1234 and 5:

| locale | 1234, set | 1234, null | 5, set | 5, null | thousands separator |
|---|---|---|---|---|---|
| en-US | `1,234` | `1,234.00` | `5` | `5.00` | U+002C |
| de-DE | `1.234` | `1.234,00` | `5` | `5,00` | U+002E |
| ru-RU | `1 234` | `1 234,00` | `5` | `5,00` | **U+00A0**, a no-break space |
| ar-SA | `1,234` | `1,234.00` | `5` | `5.00` | U+002C, **ASCII digits** |

Three things settled. A null format pointer writes two fraction digits in every
locale, "5.00", which is why the pointer is never null and why the plan said
so. Russian groups with U+00A0, so a test asserting an ordinary space would be
wrong in a way nobody can see on a screen; the test asserts the code point.
And `ar-SA` returns ASCII digits from this call with a set format, so digit
substitution is not something `GetNumberFormatEx` does here; it is left to
rendering, and the ledger entry the plan reserved for it is not needed.

`LOCALE_SNAME` from both readers: `GetLocaleInfoEx(null, LOCALE_SNAME)` and
`GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SNAME)`, which is what
`spellcheck::system_language()` calls, both answered `en-US`. They agree on
this machine. A name that is not a locale, "not a locale", is refused with
error 87; a well-formed name the machine has no data for, `xx-YY`, is answered
back as `xx-YY` with English day names behind it.

### What was built

`locales/en-US/dates.ftl` holds four messages, `dates-just-now`,
`dates-minutes-ago`, `dates-hours-ago`, `dates-days-ago`, with the variable
named for the thing it counts and used in every variant, and a header that
says what the file is, who edits it, the id and variable conventions, and the
three attribute names reserved for a control in version 2. `ls locales/` shows
`en-US` and nothing else; `ls locales/en-US/` shows `dates.ftl` and nothing
else.

`src/common/catalogue.rs` is the loader: `include_str!` on the dictionary
precedent, parsed once into a `OnceLock`, the concurrent bundle because the
date readings are called from six files, one private `a_bundle_speaking` that
applies the three settings, `for_this(which)` choosing a catalogue through
`fluent_langneg::negotiate_languages` with Filtering and English as the
default, and a `messages!` declaration on the `menu_ids!` pattern generating
the `Message` enum, its catalogue ids, the variable each counts with, and
`ALL`.

`relative_to` is `relative_to_asking(which, when, now)`, split into a pure
`which_relative_message` that decides the sentence and the count, and the
catalogue call. `plural` is gone with its test:
`grep -n 'fn plural' src/presentation/date_display.rs` finds nothing, and no
`#[allow]` was added anywhere; clippy at `-D warnings` passed on the green
commit in 50 seconds with nothing to say.

### The four conditions, each with its test, and the fallback

| Condition | Test | Red for |
|---|---|---|
| 1. `set_use_isolating(false)` | `test_no_sentence_out_of_the_catalogue_carries_an_isolation_mark`, and every exact-string test as a second witness | U+2068 and U+2069 around every number at red: `"\u{2068}1\u{2069} minutes ago"` |
| 2. `set_formatter` backed by `GetNumberFormatEx` | `test_a_number_inside_a_sentence_is_written_the_way_the_language_writes_it`: 1234 under a `de-DE` test bundle is "1.234 Tage", under `en-US` "1,234 days ago" | no formatter at red: "1234 Tage" |
| 3a. counts as numbers | `test_a_count_of_one_takes_the_singular_form` | the count passed as a string at red: "1 minutes ago", because a string matches no plural category and falls to `*[other]` |
| 3b. the error list read | `test_every_message_formats_with_its_argument_and_no_error` and `test_a_missing_argument_is_an_error_and_not_a_sentence_with_a_brace_in_it` | errors ignored at red: `say(MinutesAgo)` came back `Ok("{$minutes} minutes ago")` |
| 4. complete both ways | `test_the_english_catalogue_is_complete_in_both_directions`, with a companion proving the reading sees a missing message and an orphaned one | `dates-just-now` missing at red |
| the fallback | `test_a_locale_nobody_has_rules_for_takes_english_plurals_silently`: the English source under an `xx-YY` bundle gives "1 day ago" and "2 days ago" with no error | isolation and string counts at red |
| the name | `test_a_locale_name_that_is_not_one_asks_for_english` and `test_a_locale_this_computer_could_not_name_asks_for_english`, on the pure function that answers | `und` at red, which is `LanguageIdentifier::default()` and the plausible tidying |

The Russian resource, written inside `test_a_russian_resource_produces_the_four_russian_forms`,
produces "1 день назад", "2 дня назад", "5 дней назад" and "21 день назад";
the Polish one "1 dzień temu", "2 dni temu", "5 dni temu" and "22 dni temu".
Both ship nowhere. In `date_display`,
`test_a_computer_whose_language_has_no_catalogue_hears_english` forces `fr-FR`
and `xx-YY` and gets the same "just now", "1 minute ago", "2 days ago" and "1
day ago" as English.

**The red commit's 18 named tests, each red for its own reason.** Two of them
were red for the missing message rather than for the condition they are named
for: `test_no_sentence_out_of_the_catalogue_carries_an_isolation_mark` and
`test_the_four_english_sentences_are_the_ones_this_program_always_said` both
walk `ALL` and panicked on `dates-just-now` before reaching a placeable. They
would have been red for the isolation marks too, and they are the ones a
second run without the missing message would show it. Said rather than
smoothed over. And one test passed at red that the plan's shape would have
made red: `test_a_number_windows_cannot_write_is_left_to_the_caller` asserts
`None` for NaN and for a name that is not a locale, and the red stub returned
`None` for everything. It is the guard for the "never an empty string"
contract and is honest only beside the positive test in the same file.

### The table test and the "just now" test

That part of the diff adds `which`, through the existing `an_english_cell`
helper, and changes no expected string: the eight cases from "just now" to
"7 days ago" are byte-identical to what they asserted before this task.

### Where the four lines went, and the fifth that was not added

`Cargo.toml` gained exactly `fluent-bundle = "0.16"`, `fluent-langneg = "0.13"`,
`intl-memoizer = "0.5"` and `unic-langid = "0.9"`, with a comment naming the
audit, and `src/service/outward.rs` gained the four on `A_CRATE_THAT_CANNOT`
with the array length moved from 49 to 53 and a reason beside them.

One place wanted a fifth. The plan's reverse completeness check walks
`FluentResource::entries()` for every `Entry::Message`, and that entry type
belongs to `fluent-syntax`, which `fluent-bundle` does not re-export. Naming it
means a fifth direct dependency, and the rule was to say so rather than add
one. So the reverse check reads message ids off the catalogue's own text: a
line with a letter at column zero followed by `=` is a message start, in a
resource the parser has already accepted without error, which is asserted
first. That makes it a text-reading check, so it got the companion the plan
said a non-text-reading check would not need: a short resource is reported as
missing three, and a resource with `dates-nobody-asks` appended is reported as
holding one nobody asks for.

### `intl-memoizer` as a direct dependency, confirmed from the tree

`set_formatter` takes `fn(&FluentValue, &M) -> Option<String>` and the
concurrent memoizer's `lang` is private, read at
`intl-memoizer-0.5.3/src/concurrent.rs:9`. The formatter learns its locale
through `TheLocaleThisBundleSpeaks`, a `Memoizable` of ours whose `construct`
keeps `lang.to_string()`, reached through the memoizer's inherent
`with_try_get`. That locale is handed to the wrapper as a third `WhichLocale`
variant, `ACatalogueIsWrittenIn`, because calling a value that came from the
bundle `NamedInATest` in shipping code would be a lie about its origin. The
threat register's T-06-17 said two variants and neither built from a message,
a file or typed text; it is three now and the sentence still holds.

## Task 4: day names, the signature date, and the disclosure

### The seven day names, measured 2026-09-13

`LOCALE_SDAYNAME1` to `7`, `0x2A` to `0x30`:

| locale | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| en-US | Monday | Tuesday | Wednesday | Thursday | Friday | Saturday | Sunday |
| fr-FR | lundi | mardi | mercredi | jeudi | vendredi | samedi | dimanche |
| ru-RU | понедельник | вторник | среда | четверг | пятница | суббота | воскресенье |
| pl-PL | poniedziałek | wtorek | środa | czwartek | piątek | sobota | niedziela |

Consecutive, Monday first, in all four. `chrono::Weekday::num_days_from_monday`
is 0 for Monday too, and the wrapper says in a comment that the agreement is a
coincidence worth stating, because the other counter, from Sunday, would answer
a real name for the wrong day and pass every test written on a Monday. A guard
record breaks it that way and five tests go red, the ones asserting both ends
of the week.

### What was built

`weekday_called` in `occurrences.rs` asks `a_day_name(which, day)`, with
`which` threaded from a private `falls_on_asking` twin of the public
`falls_on`. `signed_mail::day` is `day_asking(which, moment)` calling `a_date`
with `Shape::DayMonthYear`, from a private `spoken_asking` twin of
`Finding::spoken`. `grep -rn 'presentation::' src/service/ --include=*.rs`
outside comments: 0 before, 0 after.

The one `signed_mail` test that asserted "28 August 2026" through `detail()`
now asserts it through `spoken_asking(en-US)` on the finding itself, because
through the public reading it had become a statement about the machine. Two
`occurrences` tests that asserted "every week on Monday" through `falls_on`
now go through `falls_on_asking(english())` for the same reason.

`grep -rn '"Monday"' src/` finds `when_people_are_free.rs:1442`, inside that
file's test module as task 2 checked, the wrapper's own fallback array and its
tests, and one doc comment. The mbox `From_` separator at
`src/application/message_files.rs:410` and the RRULE codes in `repeating.rs`
are untouched: that part of the diff is empty, and both must stay English
because they are formats other programs read, one an on-disk interchange
format and one a protocol.

### The source-reading guard found more than the two sites, and the list says why

`test_no_shipping_file_names_a_month_or_a_day_in_english` walks every `.rs`
under `src/`, takes the shipped half through `what_ships`, skips comment
lines, and looks inside string literals for a whole-word English month or day
name or one of the four chrono directives that write one. Its first run found
the two sites the plan named and six more literals in five files:

| literal | where | why it stays English |
|---|---|---|
| `%a %b %e %T %Y` | `message_files.rs` | the mbox separator, on-disk interchange |
| `%d-%b-%Y %H:%M:%S %z` | `protocols/imap.rs` | IMAP's INTERNALDATE, which the server parses |
| `Every weekday, Monday to Friday` | `repeating.rs`, `item_fields.rs` | the label of a Repeat choice, interface text, matched against what the form offers |
| `Month first, July 26` | `wx_settings.rs` | a Reading-tab choice with an example date |
| `Day first, 26 July` | `wx_settings.rs` | the same choice's other answer |
| `A word, July 26, 2026` | `wx_settings.rs` | the date-wording choice |

The plan asked for a list of excluded files with a reason beside each. Five
files with one literal apiece would have left every other line of those files
unread, so the list names the exact literal, with its reason, and the guard
asserts in the other direction that every allowed literal is still in the tree,
so an allowance cannot outlive its subject. One file is excluded whole, the
wrapper itself, because the English fallback arrays are what it is for.
`locales/` is not walked, and the comment says so and why. The four interface
labels are ledger 372: they are among the sentences version 2 translates, and a
French day inside an English label is not better than an English one.

The companion, `test_the_reading_sees_a_planted_english_name_and_ignores_one_in_a_test_block`,
plants "every week on Monday" in a string, "Tuesday" in a doc comment, `%B` in
a format call and "Tuesday" in a `#[cfg(test)]` module, and asserts exactly the
string and the directive are reported, with their line numbers.

### `ENGLISH_ONLY`, old and new

Old, as task 2 merged it:

> The month names in a date, the order of the day and month, and the clock all
> follow this computer. Some wording stays in English whatever language this
> computer is set to: phrases such as "2 days ago", the day names in a
> repeating appointment, and the date in a message about a signature.

New:

> The dates follow this computer: the names of the months and the days, the
> order of the day and month, and the clock. The wording around them is still
> English whatever language this computer is set to, such as "2 days ago" and
> "every week on", until Wixen Mail has a translation for this computer's
> language. Today it has only English.

The doc comment above it names the same things the string does, and the doc
comment in `ui_types.rs` that names the constant was corrected in the same
commit, because it still said the day names were English.

## Criterion 2, clause by clause

Read from `.planning/ROADMAP.md` at source:

> Month names, day names and relative wording follow the machine's locale,
> falling back to English silently where there is no translation.

| Clause | Closed | Why |
|---|---|---|
| Month names follow the machine's locale | **yes** | `date_display`'s three readings and the appointment form's month list since task 2; the eight signature sentences since task 4. The source-reading guard holds that no other shipping literal names one, apart from the three settings labels allowed by name as interface text |
| Day names follow the machine's locale | **yes, with the frame still English** | `occurrences.rs` asks the machine. "every week on mardi and jeudi" is a French day inside an English sentence, held by a test that says so in its name. The Repeat label "Every weekday, Monday to Friday" is interface text, allowed by name, ledger 372 |
| Relative wording follows the machine's locale | **yes, as far as one catalogue can** | The wording comes out of a catalogue chosen for the machine's locale through real plural rules, and a Russian resource proves four forms on this machine. Only an English catalogue exists, so every machine gets English today, which is the next clause |
| Falling back to English silently where there is no translation | **yes** | Forced rather than read off the machine: `fr-FR` and `xx-YY` get the eight English sentences with nothing said, and an unparseable name or a failed read asks for English by name. The wart task 2 recorded stands: a corrupt stored day falls back to English even where the language exists, ledger 363 |

**FEEDBACK-02 completes structurally**, and `requirements-completed` says so.
What it does not do is close the listening: no date, day name, sentence or
number has been heard by a screen reader in any language, ledger 360, 364,
366, 368, 370 and 371, all waiting for the pass after phase 8.

## What the plan or its audit said that the tree contradicted

1. **The reverse completeness check as written needs a fifth dependency.**
   `FluentResource::entries()` yields `fluent_syntax::ast::Entry`, and naming
   the variant means naming the crate. Read off the text instead, with a
   companion. Described above.
2. **`WhichLocale` has three variants, not two.** The audit's key link says the
   formatter learns the bundle's locale through a `Memoizable`; what it does
   not say is that the value then has to be handed to the wrapper as
   something, and neither existing variant was honest for it.
3. **The wrapper's guard record was stale within its own plan.** "the shape
   the person chose decides the picture" named 4 tests, measured when the
   module had no callers. Task 2 gave it three, and 44 more tests in eight
   files redden through them. None of those eight files gained a test between
   the two measurements, so the count check had nothing to say: this is the
   first of the three things `CLAUDE.md` says that check cannot see, met two
   tasks after the record was written. Found only because task 3 changed the
   wrapper's own test count and the remedy that printed ran this record.
   Corrected to the 48 that really go red, and measured again: all 48, nothing
   else.
4. **The source-reading guard's exclusion list is by literal, not by file**,
   for the reason above; and it found six literals the plan did not predict.
5. **`ar-SA` needs no ledger entry.** The plan reserved one for whether
   `GetNumberFormatEx` substitutes digits; measured, it does not.
6. **Task 4's `grep -rn '"Monday"' src/` finds more than
   `when_people_are_free.rs:1442`**: the wrapper's fallback array, its tests,
   and a doc comment in `occurrences.rs`. All expected; the plan's line was
   written before the wrapper held day names.

## Deviations from plan

**1. [Rule 3] The `build(06-03)` commit was refused once by a pre-existing
race and retried.** Described above; ledger 374 with the cause and the fix.
No change to the tree.

**2. [judgement] The reverse completeness check reads text rather than the
AST**, to hold the four-line rule. It gained the companion a text-reading check
needs.

**3. [judgement] Six literals allowed by name rather than five files
excluded.** The plan's shape would have blinded the reading to most of
`wx_settings.rs`; the guard also asserts each allowance is still exercised.

**4. [Rule 2] A third `WhichLocale` variant.** Naming the bundle's locale
`NamedInATest` in shipping code would have broken the module's own rule about
what each variant means.

**5. [scope] `ui_types.rs` gained a doc-comment correction** in task 4's
green, because it named `ENGLISH_ONLY` and said something the constant no
longer said. Nine lines, no behaviour.

**6. [test-first] `test_the_grouping_string_becomes_the_number_the_structure_wants`
was written at green**, not red, beside the helper it holds; the helper was
part of making the formatter test green, and this is a unit test on the line
the comment says somebody will tidy into `parse()`.

## Guard records

`guards/guards.toml` holds **755** records, counted with a TOML reader on
2026-09-13 at the merge; the census at lines 79 and 80 reads 192 and 563,
which sums to 755. Nine records arrived in this half of the plan, every one
run through `scripts/guards.sh --remeasure` before its commit, and one was
corrected.

| Record | Break | Predicted | Measured |
|---|---|---|---|
| a sentence out of the catalogue carries no isolation mark | `set_use_isolating(true)` | 11 | **11**, nothing else |
| a count reaches the catalogue as a number and not as a string | `count.to_string()` | 8 | **8** |
| a number inside a sentence is written by Windows | `set_formatter(None)` | 1 | **1** |
| a sentence the catalogue could not say is an error and not a sentence | drop the error check | 1 | **1** |
| a locale name that cannot be read asks for English by name | `unwrap_or_default()` | 2 | **2** |
| every message the code can ask for is in the English catalogue | declare a fifth message | 3, then 4 | **4**: the companion that expects exactly three missing was the one I had not predicted |
| a repeat-series sentence names its days the way this computer does | put the English match back | 2 | **2** |
| a signature sentence writes its date the way this computer does | `%B` back | 2 | **2** |
| day names are counted from Monday, which is where Windows starts them | `num_days_from_sunday` | 5 | **5** |
| the shape the person chose decides the picture, task 1's | unchanged | named 4 | **48**, corrected and re-measured |

The catalogue-missing break was tried against the code, declaring a fifth
message, rather than against the `.ftl`, because a record breaks a source file
whose tests the count check reads and the catalogue is not one. No candidate
reddened nothing. The eleven records the count check named across the two
green commits, three on `date_display.rs`, three on `occurrences.rs`, four on
`signed_mail.rs` and the wrapper's, were re-measured with the command it
printed, and every one still reddens exactly what it names; `date_display.rs`
moved from 41 tests to 42, `occurrences.rs` from 62 to 63, `signed_mail.rs`
from 112 to 113, the wrapper from 9 to 17, and `catalogue.rs` holds 14.

## What the gate selected, and the holes in it

Every commit through the hook, never piped; `scripts/check.sh all` once on the
branch at `f924c7b6` before the merge, 7,165 library tests, the release build
and the advisory check green, exit 0; and the merge commit ran the same again.

| commit | answered | scoped runs |
|---|---|---|
| `3c1291f0` build | `all` | everything |
| `3fdb0706` red | `red` | `common::catalogue`, `common::how_the_machine_writes_dates`, `common` (from `mod.rs`), `presentation::date_display`, the tree guards |
| `a0136102` green | `affected` | `common::catalogue`, `common::how_the_machine_writes_dates`, the tree guards |
| `e6f90a84` red | `red` | `application::occurrences`, `common::how_the_machine_writes_dates`, `service::signed_mail`, the tree guards |
| `f924c7b6` green | `affected` | the same three, and the tree guards |

Files that mapped to nothing: `guards/guards.toml`, `docs/changelog.md`,
`Cargo.toml`, `Cargo.lock`, and **`locales/en-US/dates.ftl`**, which is new to
the tree. The catalogue is compiled in through `include_str!`, so a change to
it rebuilds `catalogue.rs`, but the scoped mapper chooses modules from
`src/*.rs` paths and `house_style`'s `ours()` does not walk `locales/`, which
the plan says to leave for version 2. So a commit touching only the catalogue
runs formatting, clippy and the tree-reading guards and never
`common::catalogue::`, whose parse and completeness checks are the ones that
would catch a broken message. What covers it: the whole gate at the merge, and
CI. Ledger 373. The `.ftl` was never changed alone in this plan, so the hole
was not met; it is recorded because it will be.

Every red set was enumerated with a hook-shaped run, `scripts/check.sh
--message-file=<draft>` against the staged files, and both were accepted by
the gate exactly as named: 18 tests for task 3, 5 for task 4.

## Structure proved, experience not

**Proved by string comparison under a forced locale**, on this machine and in
CI: the eight English relative sentences; the four Russian and four Polish
forms; the absence of U+2068 and U+2069 over every message; "1.234" under
`de-DE` and "1,234" under `en-US`; the English fallback for `fr-FR` and
`xx-YY`; the seven English day names and "lundi", "dimanche", "среда"; "every
week on mardi and jeudi"; "28 août 2026" in a signature sentence; and that no
shipped string literal names a month or a day in English apart from the six
allowed. Those are real, they would catch the faults they were written for,
and they say nothing about how any of it sounds.

**Only a person can settle**: whether a Russian listener hears "2 дня назад" as
natural in a list cell; whether a number written with a no-break space is read
correctly by a screen reader in that locale; whether a French day inside an
English frame is heard as a day or as a fault; whether the signature sentence's
date reads as a date; and whether anything out of the catalogue reads well at
all. All wait for the pass after phase 8.

**Ledgered**, through `gsd-tools windows append`, both halves matching at
**374** entries: 366 to 374, nine entries, one per unrun thing or finding.

| id | kind | What |
|---|---|---|
| 366 | unrun-verify | No sentence out of the catalogue has been heard |
| 367 | unmet-truth | No non-English catalogue exists; which languages ship is Pratik's |
| 368 | unrun-verify | The Russian forms and the U+00A0 number, never heard on a Russian machine |
| 369 | deviation | Two readers of `LOCALE_SNAME`, agreeing here; version 2 retires one |
| 370 | unrun-verify | The repeat-series sentence, "every week on mardi", never heard |
| 371 | unrun-verify | The signature sentences, never heard |
| 372 | unmet-truth | Four interface labels still carry English day and month names, allowed by name |
| 373 | deviation | `locales/` maps to no gate target |
| 374 | deviation | Ledger 365's cause: a lazy-initialisation race in `keyring` 4.1.5 |

## Version and changelog

`git tag --contains 082ab42e` is empty, so task 2's 0.121.0 is unreleased and
no commit in this half bumps the version. Two `[Unreleased]` entries: task 3's
says nothing a user hears has changed and what the catalogue is for, with the
Known limitations the plan asked for; task 4's, above it, names the day names
and the signature date as the user-visible change and says the mixed French
sentence out loud.

## Commits

| Commit | What |
|---|---|
| `733c0dce` | test(06-03), task 1 red |
| `b857be6d` | feat(06-03), task 1 green |
| `a7f6346b` | fix(06-03), the non-Windows half builds clean |
| `455f30ce` | test(06-03), task 2 red |
| `082ab42e` | feat(06-03), task 2 green, version 0.121.0 |
| `cf07ea3a` | docs(06-03), the partial summary |
| `e98514b0` | the merge of tasks 1 and 2 |
| `3c1291f0` | build(06-03), Project Fluent, the whole gate on its own |
| `3fdb0706` | test(06-03), task 3 red, 18 tests named and held to it |
| `a0136102` | feat(06-03), task 3 green |
| `e6f90a84` | test(06-03), task 4 red, 5 tests named |
| `f924c7b6` | feat(06-03), task 4 green |
| `39417f88` | the merge of tasks 3 and 4, branch `two-days-ago-comes-out-of-a-catalogue` |

Nothing pushed. No tracked file was edited by a script: the exception set was
named as zero at the start and stayed there, with `cargo fmt` as the project's
own formatter the only tool that rewrote a source file.

## Tasks 1 and 2, as written when the plan was partial

What follows is the earlier executor's account, kept as written. One claim in
it the tree contradicted, and the plan's correction already records it: the
mbox separator is at `src/application/message_files.rs:410`, not in
`src/service/`, and task 4's guard names that path.

### What I was handed, and what I made of it

A previous executor was terminated mid-task. Task 1 was committed in three
commits; task 2 was staged and uncommitted, four files, and the message for its
red commit was being written when it stopped. Its last note said a third guard
had fired that it had not predicted, and that it was going to read it rather
than add it to the trailers blindly. That was the right instinct and finishing
that reading was the first job here.

The staged work was kept, and it was worth keeping. Three things in it are
better than what the plan asked for: the red half was stubbed with the
standalone month lookup, the plausible wrong implementation, which is wrong
only in Russian and Polish and was tested in Russian for that reason; the
French assertion would not have caught the fault, and the test says so; and
`read_aloud`'s birthday assertion was strengthened from a prefix to the whole
clause. One thing in it was wrong: the reworded `ENGLISH_ONLY` said the month
names follow this computer while `signed_mail.rs` still wrote `%B`, and it was
corrected to name the signature date.

### What was really red

Three tests where one was expected: the Russian genitive test, the guard count
check, and `test_every_guard_record_still_names_one_place_in_the_tree`, which
fires when a record's `before` text names code a rename moved. Running
`scripts/check.sh affected` by hand runs only the tree-reading guards; the
scoped module runs come from the index the hook hands over, and a bare
invocation hands over nothing. The set was completed by hand and the gate
confirmed it at `455f30ce`.

### Both Win32 mechanisms

A month name inside a date and a month name on its own are different words in
Russian, Polish, Czech and Lithuanian. `date_part_asking(ru-RU, 2 January)`
contains "января" and `a_month_in_words_asking(ru-RU, January)` starts with
"Январь", asserted together so that neither reading can quietly start using
the other's mechanism. The measurements in the wrapper's doc comment are the
terminated executor's, taken 2026-09-13.

### The ordinal

Removed, not silenced: `ordinal` was private with two callers in
`a_day_in_words`, a date picture cannot produce "14th", and `date_part` never
had one. Three readings asserted it and were corrected.

### Task 2's guard records

No new record; the corrected record "a whole day is spoken as a date under the
relative style" and its two neighbours were re-measured, each moving from 37
tests to 41, each reddening exactly what it names.

### The gate at tasks 1 and 2

`scripts/check.sh all` on the branch at `cf07ea3a`, zero failures. The first
attempt at the merge commit failed one integration target whose text was lost
to a `tail`, ledger 365, whose cause is now 374.

## Self-Check: PASSED

Every claim above checked against the tree at `39417f88` rather than against
this document.

| Claim | Check | Result |
|---|---|---|
| The three created files exist | `src/common/catalogue.rs`, `locales/en-US/dates.ftl`, `src/common/how_the_machine_writes_dates.rs` on disk | found |
| Thirteen commits exist | each short hash against `git log --all` | all found |
| `plural` is gone | `grep -n 'fn plural' src/presentation/date_display.rs` | 0 lines |
| No lint was silenced | `grep -rn 'allow(dead_code)' src/common/catalogue.rs src/common/how_the_machine_writes_dates.rs src/presentation/date_display.rs` | 0 lines |
| Four manifest lines, four classifications | the two greps the plan gives | the manifest grep answers 7 lines, 4 of them dependency lines at column 0 and 3 the comment above them naming the crates; `outward.rs` answers 4 |
| `Cargo.lock` | `grep -c '^\[\[package\]\]'` | 706 |
| `locales/` | `ls locales/`, `ls locales/en-US/` | `en-US`; `dates.ftl` |
| Both ledger halves agree | table rows and JSON entries | 374 and 374 |
| No carriage return in any touched file | `tr -cd '\r' \| wc -c` per file | 0 in every one |
| The full gate is green | `scripts/check.sh all` on the branch, and the merge | 0 failures, exit 0 |

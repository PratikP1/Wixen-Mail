---
phase: 06-how-the-application-speaks
plan: 09
status: partial
subsystem: application
tags: [due, reminders, tasks, events, kind, identity, seam, catalogue, relative-wording, guards, checkpoint]

requires:
  - phase: 06-how-the-application-speaks
    provides: "06-05's `raise_what_is_due` with `BetweenLooks`, `say`, `Spoken` and the repeat rule, read by the names its summary records; and 06-03's catalogue at `locales/en-US/dates.ftl` with the `messages!` declaration, where relative wording lives"
  - phase: 05.1-notes
    provides: "`docs/development/the-notes-seam.md`, the identity rule and the table of where a contract knows it has assumed one implementation, which `Kind`'s doc comment copies one level down"
provides:
  - "`due::Kind { Reminder, Task, Event }` with `ALL`, `word`, `key`, `from_key` and `can_be_done`, a total `From<Kind> for ItemKind`, and a doc comment that is the seam for a fourth kind"
  - "`due::Identity { kind, id }`, opaque, composed by a kind's own feed and taken apart by nothing"
  - "`due::Candidate` and `Candidate::at_its_own_time`, what a feed hands `what_is_due`; `Due` carries an `Identity` and its sentence starts with the kind's word"
  - "`due::what_is_due(candidates, now, already: &HashSet<Identity>, held: &HashMap<Identity, DateTime<Local>>) -> Vec<Due>`, obeying a hold, refusing an ended event, earliest first"
  - "`due::when_a_day_alerts(NaiveDate, hour)` and `due::when_an_event_alerts(Moment, lead_minutes)`, pure, through `common::moment`"
  - "`date_display::how_soon`, `how_long_ago` and `time_of_day`, and `Message::{InMinutes, InHours, InDays}` with their three sentences in `dates.ftl`"
affects: [06-09 tasks 2 to 4, version-2]

actuals:
  tokens: 22100
  tasks: 1
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A kind of its own for a closed set that can grow, with the doc comment naming every place the code assumed the current members, so the next member is a variant plus the compile errors it produces"
    - "An identity of a kind and an opaque string, composed by the feed that knows the row and parsed by nobody, so what makes a row one thing is the feed's answer and the rule never guesses"
    - "A rule that takes its hold as an argument and is tested with a map, so the table that keeps the hold across a restart can arrive later without the rule changing"
    - "A future twin of a past reading as a second function, because the past reading's refusal of the future is a boundary two tests pin by name"

key-files:
  created: []
  modified:
    - src/application/due.rs
    - src/presentation/date_display.rs
    - src/common/catalogue.rs
    - locales/en-US/dates.ftl
    - src/presentation/wx_app.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/wx_reminder_alert.rs
    - src/presentation/accessibility/feedback.rs
    - guards/guards.toml

key-decisions:
  - "Kind is its own three-variant enum and not ItemKind, whose Contact and Note would make a contact representable as due"
  - "Late is read from the shape the moment was stored in, a day or a clock face, and not per kind: the plan's per-kind list is the existing granularity rule three times over, and the Kind doc table records it as an assumption"
  - "The event sentence says the clock beside the name and not the whole date, through a new time_of_day reading, because in 15 minutes has already placed the day"
  - "An overdue task says its day as a date, not yesterday, because a whole day is a date under every style by the birthday rule; ledger 403"
  - "The three checkpoint decisions are left to Pratik, with recommendations below; nothing under them was built"

patterns-established:
  - "A red commit that reshapes a type says which of its new tests were green on arrival and why, and each of those earns its place by a measured break before the plan ends"
  - "The count check names distinct records, not per-file sums: eleven here where the plan summed sixteen, because five records name two of the three files"

requirements-completed: []
---

# Phase 6 Plan 9: A Due knows what it is, says it first, and does not forbid a fourth kind. Task 1 of 4, stopped at the checkpoint

`Due` has a `Kind` of its own and an opaque `Identity`, its sentence begins with the kind's word for all three kinds, `what_is_due` obeys a hold by identity and refuses an ended event and hands rows back earliest first, the two alert instants are pure, and the future relative wording is three catalogue messages behind a twin of the past reading. Nothing a person can reach changed: the reminder feed builds the same rows it did and no task or event is fed yet. Tasks 2 to 4 were not attempted; the checkpoint's three questions are Pratik's and are put below with options, costs and a recommendation each.

Branch `a-due-thing-says-what-it-is-first` from `main` at `db98094c`. Red `c189a361`, green `0ca6099d`, records `a993d9d5`, this summary and the ledger at `20e3dcc4`, merged into `main` at `6d57a49b` with the whole gate green on the merge: 52 targets, 7,649 tests, the release build and the advisory check. This hash was written by the docs commit after the merge, as 06-06 and 06-07 did. Version stays `0.123.0`. Nothing pushed.

## What landed, task 1

**`Kind`.** Three variants, `Reminder`, `Task`, `Event`; `ALL`; `word` ("Reminder", "Task", "Event"); `key` ("reminder", "task", "event"); `from_key`, which answers `None` for a word this build does not know, so a hold written under "mail" by a later version is kept rather than dropped, on `AddressBook::Other`'s reasoning; `can_be_done`, yes for a reminder and a task and no for an event. A total `From<Kind> for ItemKind`, for `pim_command::no_longer_there`.

**`Identity { kind, id }`**, derived `Eq` and `Hash` over both fields. Composed by a kind's own feed and taken apart by nothing in `due.rs`; the doc comment says why the event feed has to compose id and start together.

**`Candidate`**, what a feed hands in: identity, title, `raise_at` (the moment compared against now), `when` (the stored moment the row is about, said and sorted by), `ends` (an event's end, or nothing), `done`. `Candidate::at_its_own_time(identity, title, when, done)` is the reminder's shape, `None` for a stored time that cannot be read. The reminder feed in `wx_app.rs` builds these; the task and event feeds are task 4's.

**`Due { identity, title, when, late }`** and `spoken`, a match on the kind:

| Kind | On time | Late |
|---|---|---|
| Reminder | `Reminder: Call the bank, due July 26, 2026 at 9:00 AM` (unchanged) | `Reminder, overdue: Call the bank, was due July 26, 2026 at 9:00 AM` (unchanged) |
| Task | `Task due today: File the report` | `Task overdue: File the report, was due July 25, 2026` |
| Event | `Event in 15 minutes: Standup, at 3:00 PM`; `Event now: Standup` within a minute; `Event in 15 minutes: Conference` for an all-day event, which has no clock | `Event started 10 minutes ago: Standup` |

The four existing reminder-sentence tests pass with their assertions unedited; only their `Due { .. }` literals gained `identity`. A row with no title is "Untitled reminder", "Untitled task" or "Untitled event".

**`what_is_due(candidates, now, already, held)`.** Refuses a done row, anything before its `raise_at`, anything more than `STILL_WORTH_RAISING` past it, anything in `already`, anything whose hold has not run out (held until exactly now is held no longer), and an event whose end is at or before now. Late is `is_late(moment, now)`, the existing rule by the stored shape. Sorted by the moment each row is about, stored text breaking ties, so an event raised fifteen minutes early lists after a reminder due five minutes ago.

**`when_a_day_alerts(day, hour)`** is that day at that hour through `common::moment::on_this_computer`, `None` for hour 24. **`when_an_event_alerts(start, lead)`** is the start's instant less the lead; for a whole-day start that is midnight less the lead, the night before, and the doc comment says so because decision 1 is about exactly that.

**`date_display`.** `how_soon(when, now)`: "in 15 minutes", "in 1 hour", "in 2 days", "just now" under a minute ahead, `None` beyond seven days or behind now; `which_future_message` is its pure twin with the past twin's boundaries. `how_long_ago(when, now)` is the past reading made reachable without the relative style, for "started 10 minutes ago". `time_of_day(stored, settings)` is the clock alone, "3:00 PM" or "15:00", empty for a whole day or an unreadable moment. `said_by_the_catalogue` is the lookup both directions share. The past reading's `None` for the future is untouched, and a new pin says so beside the two that already did.

**Catalogue.** `dates-in-minutes`, `dates-in-hours`, `dates-in-days` in `dates.ftl`, each with `[one]` and `*[other]` using its variable in both; `InMinutes`, `InHours`, `InDays` in `messages!` with `counting`. The completeness reading holds both ways over seven; the Russian and Polish resource tests still produce their forms.

**Doc comments only.** `feedback.rs`'s `Event::Reminder` says it is the event for every kind since 2026-09-14; `catalogue.rs`'s module doc says seven messages; `dates.ftl`'s header says the file now holds how soon as well as how long ago.

## The `Kind` doc comment, quoted

The seam list:

> A mail message somebody asked to be told about later is the fourth kind, after version 1. It arrives by adding a variant here and answering the compile errors, which come in this order:
>
> 1. `Kind::word`, the word said first in its row, and `Kind::key`, the word it is stored under, which `Kind::from_key` reads back. A stored word this build does not know is nobody's kind and is kept rather than dropped, on `AddressBook::Other`'s reasoning.
> 2. `Kind::can_be_done`, whether done means something for it.
> 3. `Due::spoken`, its sentence, with the word first.
> 4. The `From` into `ItemKind`, for the announcement that a row has gone.
> 5. An alert-instant function of its own beside `when_a_day_alerts` and `when_an_event_alerts`, if the moment it is raised at is not the moment it is about.
>
> The identity string on a `Due` is composed by the kind's own feed and read back by the kind's own writers, and nothing else takes it apart.

The table of where the module knows it has assumed three kinds:

| Assumption | Where | What a fourth kind does |
|---|---|---|
| The sentence forms: a reminder is "due", a task is "due today", an event is "in", "now" or "started" | `Due::spoken` | Adds an arm; the signature does not change |
| Done means something for a reminder and a task and nothing for an event | `Kind::can_be_done` | Adds an arm |
| A thing is raised at its own moment, or at its day at an hour, or at its start less a lead | `when_a_day_alerts`, `when_an_event_alerts` | Adds a function; neither existing one changes |
| Whether a row is late is read from the shape its moment was stored in, a day or a clock face, and no kind asks otherwise | `what_is_due` | Holds unless its moment means something a day or a clock face does not |

The fourth row is one the plan did not name. The plan's "late per kind" turned out to be the existing granularity rule three times over, so there is no per-kind match on late, and the doc says so rather than leaving a fourth kind to discover it.

## Names tasks 2 to 4 read, as shipped

| What the plan calls it | What shipped | Where |
|---|---|---|
| the kind | `due::Kind`, `Kind::ALL`, `.word()`, `.key()`, `Kind::from_key(&str) -> Option<Kind>`, `.can_be_done()` | `due.rs:74-120` |
| the identity | `due::Identity { kind: Kind, id: String }` | `:145` |
| a candidate | `due::Candidate { identity, title, raise_at, when, ends, done }`, `Candidate::at_its_own_time(identity, &str, &str, bool) -> Option<Candidate>` | `:152` |
| the rule | `due::what_is_due(impl IntoIterator<Item = Candidate>, DateTime<Local>, &HashSet<Identity>, &HashMap<Identity, DateTime<Local>>) -> Vec<Due>` | `:366` |
| the alert instants | `due::when_a_day_alerts(NaiveDate, u32) -> Option<DateTime<Local>>`, `due::when_an_event_alerts(Moment, i64) -> Option<DateTime<Local>>` | `:440`, `:452` |
| the future wording | `date_display::how_soon(DateTime<Local>, DateTime<Local>) -> Option<String>` | `date_display.rs:694` |
| the past wording without the style | `date_display::how_long_ago(..) -> Option<String>` | `:680` |
| the clock alone | `date_display::time_of_day(&str, DateSettings) -> String` | `:747` |
| the messages | `catalogue::Message::{InMinutes, InHours, InDays}` | `catalogue.rs:133-138` |
| the session sets | `wx_app::BetweenLooks { already: RefCell<HashSet<Identity>>, said_and_waiting: RefCell<HashMap<Identity, u32>>, .. }` | `wx_app.rs:10113` |

Line numbers taken by `grep -n` at `a993d9d5`, after this table was first written from memory and found wrong at five rows.

## The roadmap criterion, read clause by clause

This plan carries no roadmap criterion of its own. `ROADMAP.md:550` lists it as "added 2026-09-14 when the answer to 06-05's checkpoint widened into one window for every due thing, each row saying its kind", under the second inherited item, which 06-05 closed. The nearest requirement is FEEDBACK-01, and the README says so at the requirement coverage paragraph: 06-05 and 06-09 carry it "because it is the nearest, while what they really close is the second item inherited from phase 1 and Pratik's widening of it". Nothing is marked complete, both because the plan is partial and because FEEDBACK-01 is not about a due window.

## Red and green, honestly

**Red, `c189a361`.** Twenty-five trailers: twenty-four tests and the count check. Sixteen in `due.rs`, four in `catalogue.rs`, four in `date_display.rs`. The types and signatures were in the commit with `todo!()` bodies so the crate built and the twenty-three reminder tests kept passing on the new shape, the pattern 06-01 set and 06-05 held. The three catalogue message ids were declared without their text in `dates.ftl`, which reddened the completeness reading and the two tests that walk every message; the commit says that is that half's red for free rather than a test written first.

**Eight new tests were green on arrival**, and the red commit lists them rather than naming them as failing: the same id under two kinds; dismissing one day of a series; a hold on one kind not holding another; an event raised at its lead; a task late once its day has passed; the `From` into `ItemKind`; an unreadable time not a candidate; the past reading still refusing the future. A derived `Eq` and `Hash`, the plumbing of the new signature, or the existing granularity rule carried each. Two of the eight, the identity pair, now have a measured break that reddens exactly them (record two below). The rest are pins.

**Green, `0ca6099d`.** 108 tests across the three modules. One red fixture re-read: the future twin's eighth-day boundary was first written a second past the seventh day, which the past twin counts as seven; it now asks the eighth day, the same boundary both ways, and the test's comment says so.

**The gate on each commit.** `application::due`, `common::catalogue`, `presentation::date_display`, `presentation::scan_fixtures`, `presentation::wx_app`, `presentation::wx_reminder_alert` and `presentation::accessibility::feedback` each mapped to their `--lib module::` run; `wx_app.rs` also pulled in the seven suites coupled to it by `guards/guards.toml`, the same seven 06-05 listed. `guards/guards.toml` and `locales/en-US/dates.ftl` mapped to no target, as the brief said; the count check and the em-dash guard live in `house_style`, a whole-tree guard, and ran anyway. The records commit ran only the tree-reading guards. Red, green and records each answered `affected`.

**`scripts/check.sh all`, twice.** The first run, 14:09 to 14:13, failed one target: `a_move_says_what_has_not_been_sent`, `test_a_note_filed_into_a_folder_made_here_has_nothing_to_be_sent`, "Could not reach the credential store: No default store has been set", ledger 374's `keyring` race, with 51 other targets green. The second run, 14:13 to 14:18, passed everything: formatting, clippy, the script suites, 52 targets, the release build and the advisory check, five of CI's seven jobs. Retried once and said so, as the brief asks.

## Guard records: eleven re-measured, six written

**The count check named eleven records, not the sixteen the plan summed.** The plan counted per file, 4 + 6 + 6, and five records name two of the three files. Eleven distinct: "a moment written with a T is one the reader knows", "a whole day is spoken as a date under the relative style", "the out-of-hours note judges the hour the cell speaks", "a reminder for a day is not called overdue during that day", "the shape the person chose decides the picture", "a sentence out of the catalogue carries no isolation mark", "a count reaches the catalogue as a number and not as a string", "a number inside a sentence is written by Windows", "a sentence the catalogue could not say is an error and not a sentence", "a locale name that cannot be read asks for English by name", "every message the code can ask for is in the English catalogue".

**Two of them stopped naming one place in the red commit.** The one-place check refused the first attempt at the red commit: the late rule had moved out of `what_is_due` into `is_late`, and the `messages!` block no longer ended at `DaysAgo`. Both locators were repaired in the red commit with their measurement left for green, and the record comments say which day and why.

**First remeasure, 13:27 to 13:43, eleven runs.** Seven agreed and had their counts written. Four came out short, each by tests this task added:

| Record | Predicted | Found | What it now names |
|---|---|---|---|
| a reminder for a day is not called overdue during that day | 1 | 2 | the task test the same rule now carries |
| the shape the person chose decides the picture | 48 | 49 | the overdue task sentence |
| a sentence out of the catalogue carries no isolation mark | 11 | 17 | three event sentences, the future catalogue test, the two `date_display` readings |
| a count reaches the catalogue as a number and not as a string | 8 | 10 | the future catalogue test and the `how_soon` reading |

Each red list was corrected from the run and measured again, 13:45 to 13:52: all four redden exactly what they name, counts written. The whole library was 7,221 tests with one ignored on every run.

**Six new records, `a993d9d5`**, each break applied by hand, the whole library run at eight threads, and the count quoted in the record:

| Record | Break | Predicted | Found | Passed around it |
|---|---|---|---|---|
| a due row says its kind before its title | the task's on-time sentence drops "Task" | 2 | 2 | 7,219 |
| the same id under two kinds is two identities | `Identity` equal and hashed by id alone | 2 | 2 | 7,219 |
| an event whose end has passed is not raised | the end compared with the alert instant instead of now | 1 | 1 | 7,220 |
| a held thing is not due until its hold runs out | the comparison flipped | 1 | 1 | 7,220 |
| an event cannot be done | `Event => true` | 1 | 1 | 7,220 |
| the future wording chooses hours where the past one would | minutes up to 120 in the future twin | 2 | 2 | 7,219 |

No break reddened less than predicted, so no second candidate was needed. None reddened nothing. Census at `guards/guards.toml:79-80` is 192 + 574 = 766, which is the record count by a TOML reader.

**The identity trap's record is in two halves and only one is here.** "The same id under two kinds" is measured. "A day of a series carrying its series' id" is composed by the event feed, which task 4 writes in `wx_app.rs`; task 1 has no code for a break to edit, only a test fixture that composes id and start the way the feed must, and breaking a fixture measures the fixture. The test is here and holds; the record that breaks the feed's composition is task 4's, and the second record's comment and ledger 404 say so.

## The checkpoint, unanswered: three decisions the feeds cannot be wired without

Tasks and events have never alerted here. Before they do, three things are Pratik's. Each is put so it can be answered without the plan: the options, what each costs, and which I would pick and why. **Nothing under any of them was built.** The model takes each answer as an argument: the hour goes into `when_a_day_alerts`, the lead or its absence into the event feed's call to `when_an_event_alerts`, and the hold into `what_is_due`'s map, so task 1 is the same under every option.

### Decision 1: what time of day a date-only thing is due

A task's due date is a date and never a time, on purpose: both providers send a time and neither means one (`tasks_api.rs:296-300`). An all-day event has the same shape. Something has to say what hour "due today" is, and today a *reminder* set for a day is due at midnight (`due.rs`, the whole-day arm).

| Option | What happens | What it costs |
|---|---|---|
| 1. Start of the day | Due at midnight, so the alert arrives when the program is next running that day: at startup in the morning for most people, at midnight for a machine left on | Consistent with a day reminder today. A machine left on hears "Task due today" at midnight, and an all-day event's stored 15-minute lead fires at 23:45 the night before, which is what the code does now for a whole-day start |
| **2. The working-day start** | Due at `working_day_starts`, default 9, `config.rs:405-406`, already on the settings screen and already the person's own. The feed passes it; for an all-day event the feed hands `when_an_event_alerts` the day at that hour rather than the whole day, so a 15-minute lead fires at 8:45 on the day | No new setting or control. The setting's comment "Read by the calendar" gains a reader. A task due on Saturday alerts at nine on Saturday. Somebody starting the program at half past eight hears nothing until nine. A day reminder still fires at midnight, so the two day-shaped things disagree until somebody decides reminders should follow too |
| 3. A fixed hour | A constant | A guess about everybody's morning, and the day somebody wants it moved it is a new setting, which this project's rule says must reach the settings screen in the plan that adds it |
| 4. A new setting of its own | An hour for due things, apart from the working day | A control, a section for it, and a second hour that means nearly the same as one already there |

**I would pick 2**, for both a dated task and an all-day event's alert base. It is the only answer that is already the person's and costs no control, and "due today" at nine is what a person means by it. The one thing to decide with it: whether a *day reminder* should move from midnight to the same hour, which is a behaviour change to reminders that this plan would otherwise leave alone. My suggestion is to leave reminders at midnight in this plan and ledger the disagreement, because a reminder set for a day has gone off at that day's start since it first went off and nobody has said it is wrong.

### Decision 2: whose alert an event follows, which is really what an empty column means

There are not two stores of an event's alert. `reminders_json` on `CalendarEventEntry` holds what Google sent as overrides, what Microsoft sent when the alert was on, and what the editor here wrote, and what the editor writes syncs up, so "which wins when they disagree" is already the sync's answer. What is undecided is what an empty column means, and it means three things: a Google event on the calendar's default alert, which this program never reads; a Microsoft event whose alert is off; and a CalDAV event whose alarms nobody has read, because no reader parses a `TRIGGER`.

| Option | What happens | What it costs |
|---|---|---|
| **1. The column only** | An event alerts where an alert is stored on it: one the editor here set, or one Google or Outlook sent as its own. The first stored entry, matching what the editor shows. No entry, no alert | A Google event on the calendar's default alert, which is most of them, is silent here; every CalDAV alarm is silent here. Both must be said plainly in the changelog. Never alerts for an event whose owner switched the alert off, which is the right silence |
| 2. The program's default for every event | Fifteen minutes before, or whatever `default_reminder_minutes` says, for everything | Interrupts for an event whose owner deliberately took the alert off, which the comment above `reminder_lead_minutes` at `src/application/calendar.rs:3363` records this program doing once and fixing |
| 3. The column, else the default | Stored alerts as in 1; the default where the column is empty | Reads "off" at Microsoft as "use the default", because both store nothing. The one mistake worse than a missed alert: a person who silenced an event is interrupted by it |
| 4. Make the absence say which | The Google pull records "use default" as its own entry and reads the calendar's default, the Microsoft pull records "off" as an empty list, the CalDAV pull reads `TRIGGER` | Three sync paths, their tests, and every reader of the column keeping its meaning. A plan of its own, and the only way to get Google's default and CalDAV's alarms here without guessing. If chosen now, this plan stops after task 3 and that plan is written first |

**I would pick 1 now, with 4 written into the changelog and the ledger as the way through.** Option 3 is tempting and wrong: it turns one absence into "on" for the provider where it means "off". Option 2 is the mistake this program already made once. Option 1 is honest about what it knows: it alerts for exactly the alerts it holds, and the changelog says which events are silent and why. When several alerts are stored, the first, because that is the one the editor shows and answers for. Option 4 is right and is not this plan.

### Decision 3: where a snoozed task or event's hold is kept

A reminder's snooze moves its own row, and that is right, because a reminder's time is its alert. A task's due date and an event's start are facts the phone also holds, so a snooze cannot move them; writing a task's date marks it pending and sends it. The rule already takes the hold as a map, so either option feeds the same argument.

| Option | What happens | What it costs |
|---|---|---|
| 1. Held for the session | A map beside `already`; a restart forgets it | Five tasks snoozed until tomorrow at five come back together at nine the next morning after a restart, which is a snooze that lied. Task 2 shrinks to `complete_task` alone |
| **2. Held in a small table** | `held_alerts`: kind as text, id, until; `CREATE TABLE IF NOT EXISTS`, nothing dropped or renamed; kind as a word so a fourth kind is a fourth word and not a schema change; an unknown word kept, on `AddressBook::Other` | One new module in `data/message_cache` at zero records, one statement in the schema, expired rows pruned at each look |

**I would pick 2.** It is what makes "snooze until tomorrow" true across a restart, it is additive, and it is the stored format the mail kind will share; `Kind::from_key` answering `None` for "mail" exists for exactly this table. The session map is cheaper by one small module and buys a snooze that does not keep its word.

### What none of these settle

Whether "Task due today" at nine reads as help or as nagging; whether a person with a Google calendar notices that most of their events do not alert here and what they conclude; whether an all-day event's alert the night before, or at 8:45 on the day, is wanted at all. Listening pass and ledger, whichever is chosen.

**The resume signal, as the plan writes it:** say the three answers. Task 1 does not change under any of them. Task 2 changes under decision 3 if the session map is chosen, and task 4's feeds take decisions 1 and 2 as their arguments. If decision 2 is option 4, this plan stops after task 3 and a plan for the three sync paths is written first.

## What the tests prove and what only a person can settle

**Structure, proved:** the kind's word is the first word of every sentence for every kind on time and late; the exact wording of each form; `from_key("mail")` is `None`; an event cannot be done; a hold is obeyed until its moment and not after; an ended event is not raised; one day of a series dismissed leaves the next due; the same id under two kinds is two things; rows come back earliest first by what they say; the alert instants at their boundaries, including hour 24 and the night before; the seven catalogue sentences and both directions of the completeness reading.

**Experience, not proved:** whether "Task due today: File the report" reads well by ear, whether the comma before "at 3:00 PM" is heard as a pause or a list, whether "Event now:" is enough warning, and whether "due today" at nine is help or nagging. None of it has been heard, and none of it can be until task 4 feeds a task or an event; ledger 402.

## Deviations from Plan

### 1. [Rule 3 - Blocking] `wx_app.rs`, `scan_fixtures.rs` and `wx_reminder_alert.rs` in the red commit

- **Found during:** Task 1, red.
- **Issue:** `Due` lost `id` and gained `identity`, and `what_is_due` changed signature, so the crate would not build without the three files that build a `Due` or call the rule. None is in task 1's file list; all are in the plan's.
- **Fix:** `BetweenLooks.already` and `said_and_waiting` keyed by `Identity`; the reminder feed builds `Candidate` rows through `at_its_own_time` with an empty hold and a comment saying why; `item.id` became `item.identity.id` at the two cache writes and the refresh. Two `Due` literals gained `identity`. No test was added to `wx_app.rs`, its 48 records were not disturbed, and the `tests/wired.rs` reading still sees `already.borrow_mut().insert(` before `wx_reminder_alert::raise(`.
- **Files:** `src/presentation/wx_app.rs`, `src/presentation/scan_fixtures.rs`, `src/presentation/wx_reminder_alert.rs`. **Commit:** `c189a361`. Ledger 405.

### 2. [Rule 1 - The tree contradicts the plan] Late is by the moment's shape, not by kind

- **Issue:** The plan's behaviour list gives late per kind: a reminder as today, a task when its day has passed, an event when its start has passed by more than a minute. A task's day is a whole day and an event's start is a clock face, so the existing rule by stored shape gives all three.
- **Fix:** `is_late(moment, now)` is the old match moved out, read from the `when` moment rather than the alert instant. No per-kind arm. The `Kind` doc table records it as the fourth assumed-three place. Two pins hold it per kind, both green on arrival.

### 3. [Rule 2 - Missing for the sentence the plan wrote] `time_of_day` and a public `how_long_ago`

- **Issue:** "Event in 15 minutes: standup, at 3:00 PM" needs the clock alone, and `date_display` had no public clock reading; "started 10 minutes ago" needs the past wording without the relative style, and `relative_to_asking` was private and reached only under that style.
- **Fix:** `time_of_day(stored, settings)` and `how_long_ago(when, now)`, seven and two lines, tested. The alternative, saying the whole `spoken` reading, would have been "Event in 15 minutes: Standup, July 26, 2026 at 3:00 PM", a date the sentence has already placed. Ledger 406 notes that all three new readings have no caller a person can reach yet.

### 4. [Plan example not produced] An overdue task says its date, not "yesterday"

- **Issue:** The plan's example is "Task overdue: file the report, was due yesterday". A whole day is read as a date under every style, the birthday rule in `spoken_asking`, and "yesterday" would be a new relative message for whole days through the catalogue.
- **Fix:** "Task overdue: File the report, was due July 25, 2026". Not widened here; if the listening pass wants "yesterday" it is one message and one arm. Ledger 403.

### 5. [Guard locators moved in the red commit] Two records repaired before they could be measured

- **Issue:** The one-place check refused the red commit: "a reminder for a day is not called overdue during that day" quoted lines that moved into `is_late`, and "every message the code can ask for is in the English catalogue" quoted a `messages!` end that no longer ended at `DaysAgo`.
- **Fix:** Both `before`/`after` pairs rewritten to the moved code in the red commit, with a comment saying the measurement follows; both measured at green, the first now naming two tests, the second still four. The first draft of the first comment wrote the task test into the prose before any run; it was taken out and put back after the run said so.

### 6. [Test assertion edited] The completeness companion expects six missing, not three

- **Issue:** `test_the_completeness_reading_sees_a_message_missing_and_a_message_nobody_asks_for` plants a one-line resource and asserts exactly which ids are missing; three more declared ids make six.
- **Fix:** The expected list is the six. A fixture consequence of the declaration, edited in the red commit and green there.

### 7. [Test reshaped] A reminder with no time is the feed's answer

- **Issue:** `test_a_reminder_with_no_time_on_it_never_goes_off` fed `None` into a rule that took `Option<&str>`; the rule now takes candidates and the feed drops a row with no time with `?`.
- **Fix:** The test does what the feed does and asserts no candidate is produced and nothing is due. A sibling asserts an unreadable time is not a candidate either.

### 8. [Estimate corrected] Eleven remeasures, not sixteen

Sixteen was a per-file sum over records that name several files. Eleven distinct, listed above. Observation 566 in the skill log.

### 9. [Rule 3 - The gate's known race] `scripts/check.sh all` run twice

Ledger 374's `keyring` race failed one target on the first run; the second passed everything. Nothing changed between the runs.

### The exception set for scripted edits is zero

Every tracked file was edited with Read then Edit or Write. `cargo fmt` reformatted three files I had written, which is the project's formatter and not a scripted rewrite, and `git diff --stat` after it showed only files this task had touched. `scripts/guards.sh` edited and restored `due.rs`, `date_display.rs` and `catalogue.rs` byte for byte during the remeasures, which is what it is for, and `git status` was clean after each. Carriage returns measured with `tr -cd '\r' | wc -c` on every file touched: zero on every one.

## Known Stubs

None. No `todo!()` remains in `due.rs`, `date_display.rs` or `catalogue.rs`. What exists and is not reached is not a stub but an unwired model: `Kind::Task` and `Kind::Event` rows are built by no feed until task 4, and ledger 406 says so where the ship gate reads.

## Threat Flags

None new. Nothing here opens a network endpoint, reads a file, or touches a trust boundary. Of the plan's register, T-06-37's model half is mitigated here: `already` and the hold are keyed by `Identity`, the series test holds, and the record for the identity-by-id-alone break is measured; the feed half is task 4's.

## Ledger

`.planning/WINDOWS.md` 402 to 406, through `gsd-tools windows append`, both halves: 402 the kind-first sentences unheard; 403 the overdue task says its date; 404 the series-identity record deferred to task 4; 405 `wx_app.rs` touched in task 1's red commit; 406 the new readings and kinds reachable by nothing a person can run.

## Not done

Tasks 2, 3 and 4. The checkpoint is a `gate="blocking-human"` decision and is not auto-answered under any mode. Nothing under the three decisions was built: no hold table, no `complete_task`, no window change, no feed.

## Verification

- `cargo test --lib -- application::due:: common::catalogue:: presentation::date_display::` at green: 108 passed, 0 failed.
- `scripts/check.sh` through the hook on all three commits, each `affected`.
- `scripts/check.sh all` on the branch at `a993d9d5`: first run failed one target on ledger 374's race, second run exit 0, 52 targets, release build and advisory check included.
- `#[test]` counts: `due.rs` 23 to 46, `catalogue.rs` 14 to 15, `date_display.rs` 42 to 47, `wx_app.rs` 199 unchanged, `mod.rs` 23 unchanged.
- Six guard records measured by hand, eleven re-measured by the printed command, 766 records by a TOML reader, census 192 + 574.
- Every document this plan wrote carries no em dash and no carriage return; `tests/house_style.rs` ran on the docs commit.

## Self-Check: PASSED

This file exists on disk. Commits `c189a361`, `0ca6099d` and `a993d9d5` are in `git log --all`. Zero em dashes and zero carriage returns in this file by `grep -c` and `tr -cd '\r' | wc -c`. The ten line numbers in the names table were re-taken with `grep -n` after being found wrong at five rows; the `calendar.rs` reference was corrected from a file that does not hold the function to the one that does.

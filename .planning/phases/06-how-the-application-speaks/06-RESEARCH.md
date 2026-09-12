# Phase 6: How the application speaks - Research

**Researched:** 2026-09-12
**Read against:** `main` at `febe8e4`, working tree clean at the start of the read.
**Replaces:** the version of this file written 2026-09-06 against commit `2face17`.
355 commits separate the two. Section "What the 2026-09-06 version got wrong" lists
every claim of that document this read overturned.
**Confidence:** HIGH on everything re-derived this session. Every in-repo claim carries
the command that produced it and the date it was run. Every external claim carries the
command that fetched it and the figure a script produced, not a figure a model read.

No `06-CONTEXT.md` exists in `.planning/phases/06-how-the-application-speaks/`, so no
locked decisions constrain scope yet. `/gsd-discuss-phase` has not run for this phase.

---

## How to read the evidence here

Three rules, each because this project has been bitten by the absence of it.

**Every number carries its command and its date.** `CLAUDE.md` says a measurement
without a date perishes, and phases 5.1, 5.2 and 7 each paid about an hour for a
correction pass caused by a dated figure being quoted undated. Nothing below is quoted
from another document. Where a figure disagrees with one in the tree, both are shown.

**Every claim of absence names what was searched.** Not one spelling, two or three.

**A count is only reported when its parts sum to its whole.** Three separate readings
of one upstream table produced three different wrong answers in this session, including
one of mine. The sum check is what caught it. Details in "Criterion 3".

Where something could not be settled by reading, it is in "Assumptions" or in "What no
plan in this phase can close", and it is stated as fact nowhere else.

---

## Summary

The phase is four nearly independent pieces sharing almost no code. Its model layer is
in better shape than the roadmap implies, its document layer is in worse shape, and one
of its four criteria still rests on a number that is wrong, though not by the amount the
last read of this file claimed.

**Criterion 1 is a real defect and the requirement describes it exactly.** Sixteen
events, four channels, storage, serialisation and the read path all ship and work,
verified today. What is missing is not just a settings panel. There is also no public
way to *read* an event's chosen channels: the only public accessor, `channels_for`,
returns the effective set after defaults and after the never-sound-alone fallback, so it
cannot tell "no override" from "all four ticked". A panel needs a new read method as well
as a new write method. The 2026-09-06 reading called this "two `pub` keywords"; it is
three methods, one of which does not exist in any form.

**Nothing defends the number sixteen.** `Event::ALL` is a hand written `[Event; 16]`.
Adding a variant to `enum Event` breaks compilation at three exhaustive matches, so the
compiler forces the variant to be described, but nothing forces it into `ALL`. Every
"every event has..." test iterates `ALL`, so a seventeenth event omitted from `ALL` would
be silently uncovered by all of them, and silently absent from the panel this phase
builds. That is FEEDBACK-01's own defect arriving again one variant at a time.

**Criterion 2 is half done and its remaining half has moved since the last read.** Order
and clock already follow Windows. Month and day names do not. The set of English date
sites is not the set the 2026-09-06 research listed: one site it named no longer exists,
and a new one arrived five days ago. `GetDateFormatEx` remains the right mechanism and
every claim about it was re-verified verbatim against Microsoft's page today.

**Criterion 3's "roughly half" is still wrong, and the correction is smaller than last
time.** Counted with a script today, Axe.Windows has 155 rules drawn from five standards,
of which three are WCAG success criteria: 1.3.1, 2.1.1 and 4.1.2. WCAG 2.2 Level AA
conformance is 55 success criteria, not the 56 commonly published, because 4.1.1 Parsing
is obsolete and removed in 2.2 and carries no level. So the scan can produce a finding
against three of fifty-five. The previous version of this file said three of fifty-six
and gave a per-rule breakdown that was wrong in every figure and did not sum to its own
stated total.

**Criterion 4's five findings still cannot be read from any document, and the reason is
sharper than staleness.** One changelog line names two of the five and calls the other
three "three container views". It was written when the scan covered one window on one
channel. It now covers eleven windows on two. The number will change and should be
expected to grow.

**The largest risk to this phase is not in its own subject.** Phase 7 is executing now
with nine written plans, and those plans edit `src/data/config.rs` and
`src/presentation/wx_settings.rs` heavily, including the exact test module and the exact
three exception lists that both of this phase's settings-reachability tasks must edit.
Section "Where this phase collides with phase 7" has the measurement.

**Primary recommendation:** four pieces, in this order. (1) The per-event panel, the only
user-facing capability and the only live defect, but sequenced *after* phase 7's plan
07-05 has landed or on an explicitly agreed split of `config.rs`. (2) The two inherited
items, small, one of which is a decision rather than a task. (3) `GetDateFormatEx`,
self-contained and collision-free. (4) The coverage list, a document and workflow change
that needs the scan re-run before it can be written honestly.

---

## Phase requirements

| ID | What it asks | What this read found |
|----|--------------|----------------------|
| FEEDBACK-01 | Set feedback channels per event, not only globally | Accurate in every clause, and understated in one: it says the write path is private, which is true, and does not say there is no public read path either. Its own evidence calls `per_event` a `fn`; it is a struct field. |
| FEEDBACK-02 | Dates and relative wording in the user's own language and format | Accurate as far as it goes. It names one file. There are four shipping sites today, and the set has changed in the last five days. |
| FEEDBACK-03 | Know how much of WCAG the automated scans actually cover | Accurate that nothing names the criteria. Its underlying figure, "roughly half", is wrong and lives in seven product-facing places, four of which state it as criteria coverage. |

Success criteria are in `.planning/ROADMAP.md` under "Phase 6: How the application
speaks". Criterion 1 was rewritten on 2026-09-06; the check on that rewrite is the next
section.

---

## Criterion 1's rewrite is still correct, re-checked at source

The roadmap says Speech and Braille cannot be set independently because they ride a
single `UiaRaiseNotificationEvent` whose declared signature takes no medium parameter.
That was checked again today rather than believed, because the brief was right that a
rewritten criterion is exactly the kind of thing that goes stale quietly. Four readings,
all current at `febe8e4`.

**The declared signature, verbatim** from `src/presentation/accessibility/screen_reader.rs:75-81`
[VERIFIED: src/presentation/accessibility/screen_reader.rs:73-81]:

```rust
    #[link(name = "uiautomationcore")]
    unsafe extern "system" {
        fn UiaClientsAreListening() -> i32;
        fn UiaHostProviderFromHwnd(hwnd: isize, provider: *mut *mut c_void) -> i32;
        fn UiaRaiseNotificationEvent(
            provider: *mut c_void,
            kind: i32,
            processing: i32,
            display_string: *mut u16,
            activity_id: *mut u16,
        ) -> i32;
    }
```

Five parameters. A provider, a notification kind, a processing hint, the text, and an
activity id. No medium, no channel, nothing a caller could set to ask for braille and not
speech.

**The module comment above it**, `screen_reader.rs:3-6`
[VERIFIED: src/presentation/accessibility/screen_reader.rs:1-6]:

> On Windows, announcements go to NVDA, JAWS, and Narrator through
> `UiaRaiseNotificationEvent`, the UI Automation call meant for saying something that is
> not tied to a focus change. NVDA routes it to speech and to a connected braille
> display, so braille needs no separate handling here.

**The routing in `accessibility.rs`**, unchanged since the last read
[VERIFIED: src/presentation/accessibility.rs:247-255]:

```rust
        // Speech and braille both ride the one screen reader notification, so
        // announcing once serves either. Announcing twice would double the
        // speech for anyone who has both.
        if channels.contains(&feedback::Channel::Speech)
            || channels.contains(&feedback::Channel::Braille)
        {
            self.announce_topic(&text, event.priority(), event.key())?;
        }
```

**`docs/accessibility.md`** still says the honest version. Verified by reading the file
this session.

`src/presentation/accessibility/accessibility.rs` does not exist; the module is
`src/presentation/accessibility.rs` with a directory of submodules beside it. The
2026-09-06 file cited it correctly.

**Verdict: the rewrite holds. Do not re-litigate and do not re-check again this phase.**
The single remaining obligation the criterion creates is a user-facing one: the screen
must say that choosing between speech and braille is done in the screen reader and not
here.

---

## Criterion 1: the sixteen events

### Counted, not quoted

Run 2026-09-12 at `febe8e4`:

```
awk '/^pub enum Event \{/,/^\}/' src/presentation/accessibility/feedback.rs \
  | grep -cE '^    [A-Z][A-Za-z]*,$'
-> 16

awk '/pub const ALL: \[Event; /,/\];/' src/presentation/accessibility/feedback.rs \
  | grep -cE 'Event::'
-> 16
```

Both halves counted separately because the declared array length, the enum body and the
array contents could disagree. They do not. The declared length is `[Event; 16]` at
`feedback.rs:114` [VERIFIED: src/presentation/accessibility/feedback.rs:114].

The sixteen, with the label each already carries through `Event::text()`. These are quoted
verbatim from the match arms at `feedback.rs:166-181`, inside `pub fn text` which opens at
`:164` and closes at `:183`
[VERIFIED: src/presentation/accessibility/feedback.rs:164-183]:

| Event | Label |
|---|---|
| `ThreadLanded` | Conversation |
| `EdgeOfList` | End of list |
| `NewMail` | New mail |
| `MessageSent` | Message sent |
| `SendFailed` | Message not sent |
| `ConnectionLost` | Disconnected |
| `ConnectionRestored` | Connected |
| `SyncComplete` | Sync finished |
| `ActionRefused` | Not available |
| `UnsafeMessage` | Unsafe message |
| `MisspelledWord` | Misspelled word |
| `Reminder` | Reminder |
| `HasAttachment` | Has attachment |
| `AccountNeedsAttention` | Sign-in needs attention |
| `Confirmed` | Confirmed |
| `NothingFound` | Nothing found |

No new strings are needed for a settings control. `Event::text()` is already the wording.

### The census that counts them, and the hole in it

The brief asked what the census is, because `CLAUDE.md` warns that a census asserting a
floor is itself a guard. The answer is that **there is no census, and the compiler does
three quarters of the job.**

What does defend the sixteen:

| Mechanism | What it forces | Verified by |
|---|---|---|
| `Event::text()` | Every variant has a written form | 16 `Event::` arms, no wildcard |
| `Event::key()` | Every variant has a stored name | 16 `Event::` arms, no wildcard |
| `Event::tone()` | Every variant has a sound | 16 `Event::` arms, no wildcard |

```
grep -n '_ =>' src/presentation/accessibility/feedback.rs
-> no match
```

No wildcard arm anywhere in the file, so all three matches are exhaustive and a new
variant fails to compile until all three are answered. That is a strong guarantee and it
is the compiler's, not a test's.

**What nothing defends: membership of `Event::ALL`.** `ALL` is a hand written array. A
seventeenth variant compiles as soon as the three matches are answered, whether or not
anybody adds it to `ALL`. And every test of the "every event has..." family iterates
`ALL`, not the enum:

```
grep -n "Event::ALL" src/presentation/accessibility/feedback.rs
-> 156, 991, 1094, 1112, 1145, 1180, 1310, 1329, 1429
```

`test_no_two_events_say_the_same_words` (`:1089`), `test_every_event_has_its_own_stored_name`
(`:1107`), `test_every_event_has_its_own_tone` (`:1325`) and
`test_every_event_has_a_written_equivalent` (`:1428`) all begin `for event in Event::ALL`
[VERIFIED: src/presentation/accessibility/feedback.rs:1089-1130]. So each of them would
pass while covering sixteen of seventeen, and say nothing.

No derive macro closes this either:

```
grep -rn "strum\|EnumIter\|VariantArray" Cargo.toml src/presentation/accessibility/feedback.rs
-> no match
```

**The consequence for this phase, stated plainly.** The panel criterion 1 asks for will
be built by iterating `Event::ALL`. So will the census at `wx_settings.rs:2213`, which
announces "covers N of 16 events" after a sound scheme import using `Event::ALL.len()`
[VERIFIED: src/presentation/wx_settings.rs:2209-2214]. An event that exists and is not in
`ALL` is an event with no control, no sound scheme slot and no test coverage, which is
FEEDBACK-01's exact shape. **A plan for this phase should add the missing guard: a test
that `Event::ALL` holds every variant.** It is cheap. It can be written as an exhaustive
match from each variant to a unit, or by asserting `ALL.len()` against a count derived
from the source rather than written down, which is the shape `SoundScheme::covers` already
uses at `sound_scheme.rs:81-86` and which observation-worthy precedent exists for in
`guards/guards.toml`.

**And it is a census, so it will weaken something.** `sound_scheme.rs:479` already asserts
`scheme.covers() == Event::ALL.len()`
[VERIFIED: src/presentation/accessibility/sound_scheme.rs:474-481]. That assertion is
derived from the thing counted, which is the right shape, and it is worth leaving alone.
Do not add a second assertion of the form "at least sixteen exist": with a spare above the
floor, removing an event stops tripping it, which is the failure `CLAUDE.md` describes at
length.

### The model: what ships, what is private, and what does not exist

One string, one field, one round trip. All re-verified 2026-09-12.

| Step | Where | Verified by |
|---|---|---|
| Stored as | `AppConfig.feedback_channels: String` | `grep -n "pub feedback_channels" src/data/config.rs` gives `441:    pub feedback_channels: String,` |
| Serialised by | `FeedbackSettings::to_stored` | `feedback.rs:458` |
| Parsed by | `FeedbackSettings::from_stored` | `feedback.rs:478` |
| Read on the shipping path | `channels_for` at `feedback.rs:433`, called from `accessibility.rs:213` (`earcon`) and `:235` (`signal`) | both call sites read |
| Written by | `set_event_channels` at `feedback.rs:424`, **private** | the line reads `    fn set_event_channels(...)`, no `pub` |
| Held | `per_event` at `feedback.rs:392`, **a private struct field** | the line reads `    per_event: Vec<(Event, BTreeSet<Channel>)>,` |

**Correction, and it matters to a plan.** `REQUIREMENTS.md` FEEDBACK-01, `CLAUDE.md`, and
the 2026-09-06 version of this file all describe `per_event` as a function. It is a field.
[VERIFIED: src/presentation/accessibility/feedback.rs:392] The practical difference is
that "make `per_event` public" is not a coherent instruction for a settings panel: a
public field would expose the representation. What is needed is a new public *reader*.

**Nothing writes an override today.** Searched three spellings, 2026-09-12:

```
grep -rn "set_event_channels" src/ tests/
-> 11 hits, all in src/presentation/accessibility/feedback.rs.
   :424 definition, :496 the only shipping caller (inside from_stored),
   nine inside that file's own #[cfg(test)] module. Nothing under tests/.

grep -rn "per_event" src/ tests/
-> 8 hits, all in feedback.rs: 392, 404, 425, 426, 434, 466, 481, 499.
```

So the only route in remains a hand edited `feedback_channels` string in the stored
settings file. The requirement is right.

### Three things about the model a plan will trip over

**1. There is no way to read an event's chosen channels.** This is the finding the last
read missed. The only public accessor is `channels_for`
[VERIFIED: src/presentation/accessibility/feedback.rs:433-434], and it does three things
before returning: falls back to all four when there is no entry, intersects with the
globally enabled channels, and applies the never-sound-alone rule. A panel that renders
from `channels_for` will show four ticks for an event with no override, four ticks for an
event whose override is all four, and a tick on Braille that the user never set. **A new
public reader returning the raw `Option<BTreeSet<Channel>>` is required work, not a
convenience.**

**2. "No override" means all four channels, not silence.**
[VERIFIED: src/presentation/accessibility/feedback.rs:433-437]:

```rust
        let chosen: BTreeSet<Channel> = match self.per_event.iter().find(|(e, _)| *e == event) {
            Some((_, channels)) => channels.clone(),
            None => Channel::ALL.into_iter().collect(),
        };
```

So "reset to default" must *remove* the entry, and no method removes one:
`set_event_channels` only replaces (`:425-426` retain then push). FEEDBACK-01's third `[D]`
line asks for exactly this reset behaviour, so a `clear_event_channels` is a real, small
addition. An empty override is separately representable and means silence; it round trips,
and `test_an_event_with_no_channels_at_all_signals_nothing` (`:1477`) holds it to that.

**3. The never-sound-alone rule will contradict what the user just ticked.** Still in
`channels_for`. Tick Earcon only for New mail, leave Braille globally on, and the event
goes out on Earcon and Braille. This is not a bug and must not be "fixed": the module
header says the rule lives here so no call site can forget it, and
`test_nothing_is_signalled_by_sound_alone` (`:982`) and
`test_an_event_set_to_sound_only_still_gets_a_written_channel` (`:1005`) both hold it.

It is a design instruction for the panel rather than a detail. **Announce the effective
channels, not the chosen set**, or the settings screen lies to the person it exists for.

### The settings screen today

Seven tabs, read from `wx_settings.rs:227-285` [VERIFIED: src/presentation/wx_settings.rs:227-285]:
General, Compose, Reading, Permissions, "Calendar && PIM", **Feedback**, Advanced. So the
roadmap's "Settings Feedback tab" exists and is the sixth page.

`build_feedback_tab` is at `wx_settings.rs:2040`. It holds four global checkboxes built by
iterating `Channel::ALL`, a sound scheme picker, and import and delete buttons.

**Its doc comment argues against the grid this criterion asks for.** Verbatim from
`wx_settings.rs:2030-2035` [VERIFIED: src/presentation/wx_settings.rs:2030-2035]:

```
/// One checkbox per channel rather than a grid of events, because the choice
/// people actually make is "words, not sounds" or "sounds, not words". The
/// per-event overrides exist in the model for anyone who wants them and are
/// not worth forty checkboxes here.
```

That is a position somebody took deliberately. The plan should answer it or record that it
is being reversed. It is also the reason a bare grid is the wrong answer: the objection is
sound even though the conclusion, offer nothing, breaks the reachability rule.

The save path preserves what it cannot create
[VERIFIED: src/presentation/wx_settings.rs:2310-2316]:

```rust
    // Feedback channels. The per-event overrides in the stored value are
    // preserved: this tab only decides which channels are on at all.
    let mut feedback = FeedbackSettings::from_stored(&base.feedback_channels);
    for (channel, cb) in &w.feedback {
        feedback.set_channel_enabled(*channel, cb.get_value());
    }
    cfg.feedback_channels = feedback.to_stored();
```

### Recommended shape for the panel

Sixteen events by four channels is sixty-four checkboxes, and sixty-four checkboxes on one
page is a poor experience for somebody moving by keyboard through a screen reader, who
meets controls in order and cannot skim. The criterion does not ask for a grid.

Recommended: a Choice listing the sixteen events by `Event::text()`, four checkboxes below
it, and a "Use the default for this event" button. Five controls, always, whatever the
event count becomes. Changing the Choice reloads the four boxes; changing a box writes an
override for the selected event; the button removes it. This is the Choice-plus-controls
shape the Reading tab already uses, so it inherits a pattern rather than inventing one, and
it answers the existing comment's objection rather than overruling it.

**One trap in the labels.** `Channel::setting_label()` at `feedback.rs:366` returns the
*global* wording, which is wrong beside a single event. The per-event boxes need their own
wording, and `test_each_channel_carries_its_own_wording` (`:1386`) guards the existing one.

**One trap in how the boxes are built, which is an accessibility trap and is the subject of
the next section.** Build each checkbox with a real label through `with_label`, not with an
empty label plus `set_accessible_name`. `build_feedback_tab` already does the right thing
at `wx_settings.rs:2063-2070`, and that is the pattern to copy.

---

## What actually reaches a screen reader

The brief asked which channel each thing in this phase lands on. Windows has two, and this
project needs both right.

| Call | Channel it reaches | Who reads that channel | Source |
|---|---|---|---|
| `set_accessible_name(w, name)` | MSAA only, through `wxAccessible` | NVDA and JAWS for native controls | `names.rs:10-21` |
| A control's own window text, set by `with_label` | UI Automation, via the system's own provider | Narrator | `checkbox_labels.rs:4-11` |
| `set_name(name)` | **Neither.** An internal wxWidgets identifier | Nobody | `the_conflict_choice_can_be_heard.rs:15-18` |
| `UiaRaiseNotificationEvent` | UI Automation notification | NVDA, JAWS, Narrator; NVDA routes to speech and braille both | `screen_reader.rs:3-6` |
| An earcon | Neither. It is audio out of `rodio` | Nobody, it is heard | `feedback.rs` earcon path |
| Visual feedback | Neither. It is the status line text | Read only if focus or a notification takes somebody there | `accessibility.rs:257-262` |

**The scan channels judge different things**, and one of them judges one thing. From
`scripts/msaa-names.ps1`, its own header, verbatim
[VERIFIED: scripts/msaa-names.ps1:6-19]:

> The UI Automation scan next to this one has never measured a name this codebase sets,
> and could not have.
> [...] For a native control such as an edit box or a button, Windows supplies its own UI
> Automation provider, and that provider shadows the MSAA object underneath it. So UI
> Automation reports the system's name for those controls, which is usually empty, and
> never the one the code set.
> That makes the UI Automation scan wrong in both directions on native controls: it
> reports a missing name where the name is in fact present and spoken, and it would report
> nothing amiss if every `set_accessible_name` call in the tree were deleted.

That paragraph is the strongest in-repo statement of criterion 3's substance and the
coverage list should quote it rather than paraphrase it.

The MSAA walk asks **exactly one question**: does every element whose MSAA role is in the
`$OPERATED` map have a non-empty `accName`? It builds a record with `name`, `role` and
`operated` and exits 1 if any operated element has no name
[VERIFIED: scripts/msaa-names.ps1:92, 131, 165-177, 235]. It never looks at role
correctness, state, or anything else. So it contributes to 4.1.2 Name, Role, Value and to
the *Name* third of it only.

### Two live findings in this area

**1. One surviving `set_name` call.** Searched the whole of `src/`:

```
grep -rn '\.set_name(' src/
-> src/presentation/wx_compose.rs:3600:    body_preview.set_name("Message preview");
```

It is harmless where it sits, because `set_accessible_name` is called on the line above it
[VERIFIED: src/presentation/wx_compose.rs:3596-3601]. It is worth knowing anyway, because
it is the exact call `CLAUDE.md`'s guardrail 2 exists about, sitting in the tree today.

**2. The refusal of `set_name` is enforced in exactly one file.**
`tests/the_conflict_choice_can_be_heard.rs` refuses it, and refuses it for
`THE_WINDOW`, a constant holding one path
[VERIFIED: tests/the_conflict_choice_can_be_heard.rs:31-32, 82-86]:

```rust
/// The file that builds the window.
const THE_WINDOW: &str = "src/presentation/wx_conflict_choice.rs";
```

Its companion name census counts three builders only, `Button::builder`,
`StaticText::builder` and `ListCtrl::builder`
[VERIFIED: tests/the_conflict_choice_can_be_heard.rs:41-52], so a `CheckBox` or a `Choice`
added to that window would not be counted either. A count that knows three shapes is a
count that cannot see a fourth.

**3. The both-channels rule is tested in exactly one dialog, and it is not the settings
dialog.** `tests/checkbox_labels.rs` builds real dialogs and asserts every checkbox carries
its own label rather than borrowing one from the static text beside it. Its own header says
why, verbatim [VERIFIED: tests/checkbox_labels.rs:4-19]:

> So a check box built with an empty label and named only through `set_accessible_name` has
> a name on one channel and none on the other. It reads correctly under NVDA and is an
> unnamed check box under Narrator, which is the shape of bug that passes every test
> written from the source and every listening pass done with one reader.

It imports and builds `build_item_form_dialog` and nothing else
[VERIFIED: tests/checkbox_labels.rs:30, 65, 114]. **The settings screen is not covered by
it.** The panel this phase adds is four checkboxes on the settings screen. Nothing in the
tree would catch them being built label-less.

That is a concrete, cheap addition a plan can make: either widen `checkbox_labels.rs` to
build the settings dialog too, or accept the gap and say so. It is also the kind of thing
that has to be decided at plan time, because widening it touches a `tests/` file and
therefore wants a `guards/guards.toml` record.

---

## The two items inherited from phase 1

### Inherited item A: the per-account Allow Changes answer

The roadmap says this is FEEDBACK-01's exact shape already live in the tree. Verified, with
three corrections to the `CLAUDE.md` passage the brief warned about, and one finding that
changes what a plan must do.

**The `CLAUDE.md` passage is substantively right this time and every line number in it has
moved.** Re-taken 2026-09-12 at `febe8e4`:

| What `CLAUDE.md` says | Where it says the line is | Where it actually is |
|---|---|---|
| `allowed_per_account` is a top-level field | `config.rs:276` | **`config.rs:307`** |
| `STORED_AND_OFFERED_BY_NOTHING`, one entry | `config.rs:1799` | **`config.rs:2190`** |
| the guard that empties | `config.rs:1947` | **`config.rs:2435`** |
| the hand-named companion | `config.rs:1829` | **`config.rs:2220`** |

The substance checks out at all four new locations.

**It is a top-level field**, verbatim [VERIFIED: src/data/config.rs:306-307]:

```rust
    #[serde(default)]
    pub allowed_per_account: HashMap<String, crate::application::allowed::Allowed>,
```

**The exception list holds exactly one entry**, verbatim
[VERIFIED: src/data/config.rs:2190]:

```rust
    const STORED_AND_OFFERED_BY_NOTHING: [&str; 1] = ["allowed_per_account"];
```

**It is read and honoured** [VERIFIED: src/data/config.rs:709-715]:

```rust
    pub fn allowed_for(&self, account_id: &str) -> crate::application::allowed::Allowed {
        self.allowed_per_account
            .get(account_id)
            .copied()
            .unwrap_or(self.allowed_changes)
            .and(self.allowed_changes)
    }
```

The `.and(self.allowed_changes)` means a per-account answer can only ever narrow the
application-wide one. A control that looks like it can widen would be a control that lies.
`Allowed` has three fields, `mail`, `personal_information` and `reading`, so a per-account
control is three answers per account, not one.

**Emptying the list disarms the guard watching it. Confirmed.** Verbatim
[VERIFIED: src/data/config.rs:2434-2447]:

```rust
    fn test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing() {
        // The other direction. Somebody wiring a control for one of these
        // should be told to delete the entry, rather than leaving a list that
        // reads as a live defect after it has been fixed.
        let screens = what_every_screen_ships();

        let now_offered: Vec<&str> = STORED_AND_OFFERED_BY_NOTHING
            .into_iter()
            .filter(|name| screens.contains(name))
            .collect();
```

With the list empty, `into_iter().filter(...)` iterates nothing, `now_offered` is empty and
the assertion passes unconditionally. The brief is right and so is `CLAUDE.md`.

**The finding that changes what a plan must do, and which no document in the tree
carries.** The two tests read *different* sets of files.

| Test | What it reads | Constant |
|---|---|---|
| `test_every_setting_somebody_can_change_is_offered_by_a_screen` (`:2371`) | the settings screen only | `THE_SETTINGS_SCREEN = "src/presentation/wx_settings.rs"` (`:2140`) |
| `test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing` (`:2435`) | every file under the presentation directory | `EVERY_SCREEN = "src/presentation"` (`:2144`) |

[VERIFIED: src/data/config.rs:2140, 2144, 2371-2394, 2435-2447]

So if the control goes on the account manager, which is where the precedent points:

- the `EVERY_SCREEN` guard fires the moment any file under `src/presentation` names
  `allowed_per_account`. That is the red half for free, as intended.
- deleting the entry from `STORED_AND_OFFERED_BY_NOTHING` then makes the *other* test fail,
  because it reads only `wx_settings.rs` and the name will not be there.

**The correct remedy is therefore a move, not a deletion.** Move `allowed_per_account` from
`STORED_AND_OFFERED_BY_NOTHING` into `OFFERED_BY_ANOTHER_SCREEN` as
`("allowed_per_account", "src/presentation/wx_account_manager.rs")`. That satisfies the
settings-screen test, empties the defect list honestly, and arms
`test_a_setting_said_to_be_offered_elsewhere_really_is` (`:2415`), which checks each
`OFFERED_BY_ANOTHER_SCREEN` entry is still true and would catch the control later being
taken away [VERIFIED: src/data/config.rs:2415-2427]. The emptying question then applies to
`STORED_AND_OFFERED_BY_NOTHING` exactly as `CLAUDE.md` says, and must be decided in the
same commit: delete the now-blind test with the list, or keep it with a companion proving
the reading can see a planted violation.

**The precedent for where it goes** is in the tree, verbatim
[VERIFIED: src/data/config.rs:2151-2163]:

```rust
    const OFFERED_BY_ANOTHER_SCREEN: [(&str, &str); 3] = [
        // The account manager names the directory an account looks people up
        // in, and which account is the default one to send from. Both are per
        // account, so they belong on the screen that lists accounts.
        ("directories", "src/presentation/wx_account_manager.rs"),
        (
            "default_account_id",
            "src/presentation/wx_account_manager.rs",
        ),
        // Muting what is read aloud is a menu item with a check on it,
        // `ID_MUTE_CONTENT`, because it is reached in a hurry when somebody
        // walks into the room. A settings page is the wrong place for it.
        ("mute_message_reading", "src/presentation/wx_app.rs"),
    ];
```

That list held two entries at the last read of this file and holds three now.

**The second guard that retires.** `test_nothing_offers_a_setting_per_account_that_no_screen_writes`
in `tests/house_style.rs` reads product documents for two phrases held in
`A_CONTROL_NO_SCREEN_WRITES` at `house_style.rs:180-183`
[VERIFIED: tests/house_style.rs:180-183]:

```rust
const A_CONTROL_NO_SCREEN_WRITES: &[&str] = &[
    concat!("set it ", "per account"),
    concat!("for each account ", "separately"),
];
```

It reads `the_pages_that_speak_for_the_product()`, which explicitly excludes `.planning`
[VERIFIED: tests/house_style.rs:112, and the test's own comment at its opening], so this
research document may quote the phrases and a plan may too. A *product* document may not,
until the control exists and the list is retired in the same commit.

**One decision already recorded that a planner could misread.**
`.planning/decisions-2026-09-06.md` decision 5 begins "Allow Changes stays as it is, and
the move says so sooner." That decision is about *when a task move tells somebody* what
happened, not about whether a per-account control exists
[VERIFIED: .planning/decisions-2026-09-06.md:29-34]. It does not settle inherited item A in
either direction. Do not read it as an instruction to leave the control unbuilt.

### Inherited item B: the reminder that opens over somebody typing

The deferred-items entry is accurate on the mechanism, re-verified today, with three
corrections to the last read of this file.

**Verified: `raise_what_is_due` does not ask about typing.** It is at `wx_app.rs:10321`
and its signature is [VERIFIED: src/presentation/wx_app.rs:10321-10329]:

```rust
fn raise_what_is_due(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    already: &RefCell<std::collections::HashSet<String>>,
    one_at_a_time: &crate::application::due::OneAtATime,
    dates: date_display::DateSettings,
) {
```

No `somewhere_to_type` parameter, and no `somebody_is_typing()` call anywhere in the body.

**Verified: the folders question asks two things, not one.** At `wx_app.rs:10245-10249`
[VERIFIED: src/presentation/wx_app.rs:10227-10249] it combines
`one_question_at_a_time::somebody_is_typing()` with a scan of a `somewhere_to_type:
&[TextCtrl]` slice that `raise_what_is_due` is not given. So matching what the folders
question does means threading that slice to the reminder call site too: two changes, not
one.

**`somebody_is_typing()` covers the two windows that mark themselves.** Both shipping
call sites of `while_somebody_types()`, re-taken today:

```
grep -rn 'while_somebody_types' src/
-> src/presentation/one_question_at_a_time.rs:296  (the definition)
   src/presentation/wx_app.rs:14217               (the composer)
   src/presentation/wx_item_form.rs:297           (the item form)
   plus four uses inside one_question_at_a_time.rs's own test module
```

**Correction: the module is `src/presentation/one_question_at_a_time.rs`, not
`src/application/`.** The 2026-09-06 version of this file placed it under `src/application/`
in its validation table, and said it carried zero guard records. Both wrong: the path is
`src/presentation/one_question_at_a_time.rs` and it carries **1** record.

**Returning early is safe, and that is the good news for the small option.**
`raise_what_is_due` inserts each item into `already` *before* opening the window, and a
snooze removes it again [VERIFIED: src/presentation/wx_app.rs:10369, 10396]. Nothing is
inserted until the loop body runs, and `due::what_is_due` is passed a fresh `&seen` each
tick, so a reminder held back this tick is simply re-derived next tick. "Make it wait" is
genuinely small. "Raise it without focus" is not, and is different work.

**This is a decision, not a task, and it should be flagged as one.** The deferred-items
entry says so in its own words [VERIFIED: .planning/phases/01-folders-and-conversations/deferred-items.md,
the section "A reminder alert still opens over somebody who is typing (found in 01-10)"]:

> It arguably should not: a reminder that waits until somebody stops typing is a reminder
> that can be an hour late, and being told at the time is the whole point of asking to be
> told. That is a decision about what a reminder is for, not a line of code, and it is
> Pratik's rather than an executor's.

A plan must not choose between waiting, raising without focus, and holding briefly then
raising anyway. It should carry a `checkpoint:human-verify` before the task that implements
whichever is chosen.

---

## Criterion 2: dates and month names in the machine's locale

### What already follows the machine, re-verified

| Thing | Follows the machine? | Where |
|---|---|---|
| Day and month order | Yes | `DateOrder::from_system` (`date_display.rs:226`) reads `LOCALE_IDATE` through `read_locale` (`:136`) and `order_from_locale` (`:169`) |
| 12 against 24 hour clock | Yes | `Clock::from_system` (`:271`), through `clock_from_locale` (`:183`), `LOCALE_ITIME` |
| Month names | **No** | `MONTHS: [&str; 12]` hardcoded English at `:100` |
| Day names | **No, and none exist in this module** | see the census below |
| Relative wording | **No** | `relative_to` (`:521`) through `plural` (`:547`) |
| AM/PM, ordinals | **No** | `clock` (`:497`), `ordinal` (`:438`) |

[VERIFIED: src/presentation/date_display.rs:100, 136, 169, 183, 226, 271, 438, 497, 521, 547]

The project already calls `GetLocaleInfoW` through a hand written `extern "system"` block
at `date_display.rs:141`, with a `#[cfg(target_os = "windows")]` reader and a non-Windows
fallback at `:232`. Reaching Windows for locale data is an established pattern here, not a
new dependency.

### The English date sites, re-taken today, and they are not the same set as five days ago

Searched four ways on 2026-09-12: `grep -rn 'MONTHS' src/`, `grep -rn '"Monday"' src/`,
`grep -rEn '\.format\("' src/`, and `grep -rn '%A\|%B' src/`.

| Site | What it produces | Verdict |
|---|---|---|
| `date_display.rs:384` | `MONTHS[...]` in `a_month_and_a_day` | **In scope** |
| `date_display.rs:409` | `MONTHS.get(...)` in `a_month_in_words` | **In scope. New since 2026-09-06.** |
| `date_display.rs:477` | `MONTHS[...]` in `absolute` | **In scope** |
| `wx_item_form.rs:849` | the month Choice for entering a date, from `MONTHS` | **In scope** |
| `application/occurrences.rs:701` | English "Monday".."Sunday" in `weekday_called`, read out in a repeat-series sentence | **In scope, and not named by the requirement.** The only day-name site in shipping code |
| `service/signed_mail.rs:1373` | `moment.format("%-d %B %Y")`, English month in a signature-outcome sentence | **In scope, and not named by the requirement** |
| `application/message_files.rs:410` | `%a %b %e %T %Y` for an mbox `From_` separator | **Must stay English.** On-disk interchange format |
| `application/repeating.rs` | `"MO".."SU"` | **Must stay English.** RRULE protocol codes |
| `application/when_people_are_free.rs:1442` | English weekday names | Not shipping. Inside that file's `#[cfg(test)]` module |

**Two changes from the 2026-09-06 list, and both would have misled a plan.**

*Gone.* That document named `wx_app.rs:1846`, `chrono::Local::now().format("%A, %B %e, %Y")`
announced on the calendar Today button. **That site no longer exists.**

```
grep -rn '%A\|%B' src/
-> only src/service/signed_mail.rs:1373, plus two unrelated matches
   (a percent-encoded mailto test fixture and a %APPDATA% path in a doc comment)

git log --oneline -S '%A, %B %e, %Y' -- src/presentation/wx_app.rs
-> e94b4ae feat(05-02): a week of the calendar, and two buttons that move it
   c0be7ab feat(ui): add calendar, contacts, tasks, notes, and reminders panels
```

Phase 5.2 removed it. A plan written from the previous research would have sent an executor
to a line that is not there.

*Arrived.* `a_month_in_words` was added to `date_display.rs` in the same window, and its
only caller is `src/presentation/ui_types.rs:1205`
[VERIFIED: src/presentation/date_display.rs:396-415 for the function, and
`grep -rn 'a_month_in_words' src/` for the caller]. Its own doc comment says it exists so
that "how a month is written is this module's rule and a second answer to it somewhere else
is a second thing to keep in step", which is the right instinct and which also means
localising `MONTHS` localises this site for free.

Net: **four shipping sites in scope, three of which the requirement does not name.** Any
plan scoped to `date_display.rs` alone ships criterion 2 with English day names still being
spoken from `occurrences.rs`.

### Recommended mechanism: `GetDateFormatEx` with an explicit format picture

Not a month-name lookup table, and not a crate. Every quotation below was re-fetched from
Microsoft's own page on 2026-09-12 and is verbatim.

**Why not a month-name table.** From the `LOCALE_SMONTHNAME` constants page
[CITED: https://learn.microsoft.com/en-us/windows/win32/intl/locale-smonthname-constants]:

> Calling the GetLocaleInfo or GetLocaleInfoEx function with a LOCALE_SMONTHNAME* constant
> returns the standalone, or nominative, form of the month name. To get the genitive form
> of the month name, the application calls GetDateFormat or GetDateFormatEx with a date
> picture of ddMMMM and removes the two digits from the beginning of the retrieved string.

A date puts the month in the genitive position in Russian, Polish, Czech, Lithuanian and
others. A table of standalone forms produces grammatically wrong dates in exactly those
languages, for a screen reader to read aloud, which is the one place this project cannot
afford to sound wrong.

**Why `GetDateFormatEx` is the right call.** From its reference page, re-fetched and
quoted verbatim today
[CITED: https://learn.microsoft.com/en-us/windows/win32/api/datetimeapi/nf-datetimeapi-getdateformatex]:

> The function uses the specified locale only for information not specified in the format
> picture string, for example, the day and month names for the locale.

and

> When the date picture contains both a numeric form of the day (either d or dd) and the
> full month name (MMMM), the genitive form of the month name is retrieved in the date
> string.

That is exactly the shape this codebase needs. The four existing stored preferences become
the *picture* and the machine supplies the *names*. `DateWording::Verbal` with
`DateOrder::DayFirst` is `d MMMM yyyy`; `Verbal` with `MonthFirst` is `MMMM d, yyyy`; the
two `Numeric` arms become `dd/MM/yyyy` and `MM/dd/yyyy`. Nothing the user chose is
discarded, which passing `DATE_LONGDATE` and letting the locale decide would do.

**Signature and availability**, verbatim from the same page, re-verified today:

> [in, optional] lpLocaleName. Pointer to a locale name, or one of the following predefined
> values. LOCALE_NAME_INVARIANT / LOCALE_NAME_SYSTEM_DEFAULT / LOCALE_NAME_USER_DEFAULT

> Minimum supported client Windows Vista [...] Header datetimeapi.h Library Kernel32.lib
> DLL Kernel32.dll

**The testability argument, which is the strongest one.** FEEDBACK-02's third `[D]` line
asks that the existing `date_display` tests keep passing under a forced English locale.
Because `GetDateFormatEx` takes a locale *name* rather than reading only the user default,
a test can pass `"en-US"` and assert an English string deterministically, and pass
`"fr-FR"` and assert French, on an English CI runner. That is strictly better than the
criterion asks for. A table-based approach cannot do this without a second injection point.

**Why not `chrono`'s `unstable-locales`.** Its own documentation calls the feature unstable
and says the API may change or be removed in a patch release; its data comes from
`pure-rust-locales`, imported from the GNU C Library, so it would give glibc's idea of
French dates rather than *this machine's*, which is what the criterion asks for; and it
would still need a separate mechanism to discover which locale to use.
[ASSUMED: carried forward from the 2026-09-06 read and not re-fetched this session. Cheap
to re-check if a plan leans on it, and nothing below depends on it.]

**Why not ICU4X.** `icu_datetime` is not in the tree, re-checked 2026-09-12:

```
grep -c 'icu_datetime' Cargo.lock -> 0
```

Adding it would be a `dependency-audit` conversation, would bundle CLDR data into the
binary, and would again give CLDR's answer rather than the machine's. It would also make
`which-checks.sh` answer `all` on the commit that adds it, because `Cargo.toml` changed.

### What `GetDateFormatEx` does not give you: relative wording

There is no Windows API for "2 days ago". `relative_to` (`date_display.rs:521`) produces
"just now", "N minutes ago", "N hours ago", "N days ago" through `plural` (`:547`), which
is English pluralisation: one form for 1 and one for everything else. Russian needs three
forms and Arabic six. That is what ICU MessageFormat plural rules exist for and nothing
short of them gets it right.

`ENGLISH_ONLY` (`date_display.rs:93`) is the product's existing disclosure, verbatim
[VERIFIED: src/presentation/date_display.rs:93-96]:

```rust
pub const ENGLISH_ONLY: &str = "Dates are written in English. The order of the day and month \
     and the clock follow this computer, but the month names and wording such as \"2 days ago\" \
     stay in English whatever language this computer is set to.";
```

It is shown on the Reading tab at `wx_settings.rs:1353` as a label and at `:1357` as an
accessible name, and referenced from `ui_types.rs:1017`
[VERIFIED: `grep -rn 'ENGLISH_ONLY' src/ docs/`]. Whichever option is chosen, this sentence
has to be reworded rather than deleted, unless plural rules are taken on.

The three options are in "Decisions for Pratik" below. They are not a planner's to pick.

### One thing to be careful of: `config.language` is not the UI language

`AppConfig.language` is at `config.rs:273` and is the **spellcheck dictionary** language
[VERIFIED: src/data/config.rs:273]. Reusing it for dates would conflate "what language I
write in" with "what language my dates are read in", and the criterion asks for the
machine's locale, not a stored preference. Leave it alone.

---

## Criterion 3: what the scan can and cannot judge

### What the scan is, re-verified

`.github/workflows/accessibility.yml`. Per run it builds `--release`, downloads the
**latest** `AxeWindowsCLI` zip from the GitHub releases API, starts the application once
per window against a throwaway profile, runs `AxeWindowsCLI.exe` over UI Automation, runs
`scripts/msaa-names.ps1` over MSAA, distinguishes "not scanned" from "scanned clean", and
is non-blocking.

**The target list has grown since the last read.** Verbatim from
`.github/workflows/accessibility.yml:100`
[VERIFIED: .github/workflows/accessibility.yml:100]:

```powershell
        $targets = @('main', 'settings', 'accounts', 'compose', 'reader', 'search', 'filters', 'calendar', 'first-run', 'add-calendar', 'blocked-senders')
```

Eleven targets: the main window plus ten dialogs. `ScanTarget::ALL` is
`[ScanTarget; 10]` at `scan_target.rs:72` [VERIFIED: src/presentation/scan_target.rs:72-83].
`blocked-senders` arrived in the five days since the previous read, which said ten and nine.
Two of `scan_target.rs`'s own tests read the workflow file to keep the two lists in step.

### The real WCAG coverage of Axe.Windows, counted rather than read

This is the figure the previous version of this file got wrong, and it is worth saying how,
because the same trap is available to the next person.

The previous version reported "144 rules" with a breakdown of 81 / 28 / 19 / 14 / 13.
Those five numbers sum to 155, not 144, and nobody added them up for five days. A language
model asked to read the same table today answered "173 rules" with a different breakdown
again. My own first parse answered 141, because a filter written to skip the header row
(`$1 !~ /^Name/`) also skipped fourteen data rows whose first column was the *name of a
rule beginning with "Name"*.

**The figure below comes from a parse whose parts sum to its whole**, run 2026-09-12:

```bash
curl -sL https://raw.githubusercontent.com/microsoft/axe-windows/main/docs/RulesDescription.md \
  -o axe-rules.md
awk -F'|' '$0 ~ /\|/ && $0 !~ /^Name \|/ && $0 !~ /^-+ \|/ && $0 !~ /^#/ \
  {s=$NF; gsub(/^[ \t]+|[ \t]+$/,"",s); c[s]++; n++} \
  END {for(k in c) printf "%4d  %s\n", c[k], k; printf "TOTAL %d\n", n}' axe-rules.md
```

| Standard referenced | Rules |
|---|---|
| WCAG 1.3.1 InfoAndRelationships | 61 |
| Section 508 502.3.1 ObjectInformation | 53 |
| Section 508 502.3.10 AvailableActions | 23 |
| WCAG 4.1.2 NameRoleValue | 9 |
| WCAG 2.1.1 Keyboard | 9 |
| **TOTAL** | **155** |

61 + 53 + 23 + 9 + 9 = 155. A separate line census of the file confirmed 155 data rows out
of 174 lines, all with exactly four columns.
[VERIFIED: the command above, run 2026-09-12 against the file's `main` branch state]

**Three distinct WCAG success criteria: 1.3.1, 2.1.1, 4.1.2.** Nothing else.

**The denominator, also counted rather than quoted.** Run 2026-09-12 against
`https://www.w3.org/TR/WCAG22/`, parsing each `<section class="guideline">` that carries a
`Success Criterion N.N.N` heading and reading the `(Level X)` marker from its text:

```
SC sections found: 87
levels: A=31, AA=24, AAA=31   (sum 86)
no level found for: 4.1.1
```

The one with no level is **4.1.1 Parsing**, and the page says why, verbatim
[CITED: https://www.w3.org/TR/WCAG22/]:

> Success Criterion 4.1.1 Parsing (Obsolete and removed) [...] This criterion no longer has
> utility and is removed.

**So WCAG 2.2 Level AA conformance is 55 success criteria, not 56.** The commonly published
56 counts 4.1.1, which was Level A in WCAG 2.0 and 2.1. The previous version of this file
carried 56 as assumption A3 and asked for it to be re-derived before it went into a
document. It has now been re-derived, and the answer is 55.

**The claim in the tree is that automated scanning "covers roughly half of WCAG". The scan
can produce evidence against three of fifty-five, and against none of them completely:** a
rule derived from 1.3.1 tests one narrow structural property, and 61 such rules do not add
up to judging 1.3.1.

The industry figure of roughly half is about the proportion of accessibility *defects* an
automated tool finds, not the proportion of *success criteria* it can judge. Written as
criteria coverage, which is how four of the copies in this tree read it, it is wrong by
about an order of magnitude.

### Every place the estimate lives, re-taken 2026-09-12

```
grep -rn "roughly half\|about half" docs/ .github/ src/ CLAUDE.md .planning/intel/
```

| File and line | Text | Form |
|---|---|---|
| `docs/accessibility.md:12` | "Automated scanning catches roughly half of what WCAG asks for" | **of WCAG, wrong** |
| `docs/IMPLEMENTATION_STATUS.md:177` | "Automated scanning covers roughly half of WCAG" | **of WCAG, wrong** |
| `docs/principles.md:67` | "Automated scanning covers roughly half of WCAG" | **of WCAG, wrong** |
| `.github/workflows/accessibility.yml:247` | the step summary: "This scan covers about half of WCAG" | **of WCAG, wrong** |
| `docs/changelog.md:10512` | "It covers roughly half of WCAG and does not replace NVDA testing" | of WCAG, but history |
| `.github/workflows/accessibility.yml:14` | "Together they catch roughly half of accessibility defects" | of defects, defensible |
| `CLAUDE.md:705` | "Automated checks catch roughly half of accessibility defects" | of defects, defensible |
| `.planning/intel/built-and-left.md:133`, `.planning/intel/context.md:18, 30, 52` | four planning copies | planning, not product |

**Four product-facing copies to correct, two to leave, one to decide about.** The changelog
copy sits in a released-version note and is history, so adding a correction may be better
than editing it. Five of the eight line numbers moved since the last read of this file.

### The frame the coverage list should use, and a correction to how it was described

WCAG is written for web content. Applying it to a desktop application is what **WCAG2ICT**
is for: *Guidance on Applying WCAG 2 to Non-Web Information and Communications
Technologies*, a W3C Group Note published 11 December 2025
[CITED: https://www.w3.org/TR/wcag2ict-22/, publication date read from the page's own
`<time class="dt-published" datetime="2025-12-11">` element on 2026-09-12].

**The previous version of this file described its authority slightly wrongly, and the
difference matters to a list that claims to be honest.** WCAG2ICT does not itself decide
what applies. Verbatim from the page:

> This document does not seek to determine which WCAG 2 provisions (principles, guidelines,
> or success criteria) should or should not apply to non-web documents and software, but
> rather, if applied, how they would apply.

What it does is *record what regulations have decided*, and it attributes them separately.
Verbatim:

> For example, some local standards such as Section 508 in the U.S., and EN 301 549 in
> Europe, state that WCAG 2.0 Success Criteria 2.4.1 Bypass Blocks, 2.4.5 Multiple Ways,
> 3.2.3 Consistent Navigation, and 3.2.4 Consistent Identification do not apply to non-web
> documents and non-web software. In addition, EN 301 549 states that 2.4.2 Page Titled and
> 3.1.2 Language of Parts do not apply to non-web software.

And it names a regulation that goes the other way:

> In contrast, the U.S. Department of Justice regulation, Nondiscrimination on the Basis of
> Disability; Accessibility of Web Information and Services of State and Local Government
> Entities (89 FR 31320, 24 April 2024), directs implementers to utilize the guidance in
> this document to determine the applicability of success criteria [...] Since this document
> does not specifically say which criteria can or should apply, those implementing this
> document (WCAG2ICT) should consider the applicability of individual success criteria to
> non-web documents

So the six criteria split two ways, not one:

| Criteria | Excluded by |
|---|---|
| 2.4.1 Bypass Blocks, 2.4.5 Multiple Ways, 3.2.3 Consistent Navigation, 3.2.4 Consistent Identification | Section 508 **and** EN 301 549 |
| 2.4.2 Page Titled, 3.1.2 Language of Parts | EN 301 549 only |

A coverage document must attribute these to the regulations, not to WCAG2ICT, and should
say that this project makes no Section 508 conformance claim, which
`docs/accessibility.md:18-19` already says.

A defensible coverage table then has roughly this shape:

| Column | Source |
|---|---|
| Success criterion, A and AA only, 55 of them | WCAG 2.2, counted |
| Applies to a Windows desktop application? | WCAG2ICT, with the regulation named where it is a regulation's call |
| Can Axe.Windows produce a finding against it? | yes for 1.3.1, 2.1.1, 4.1.2; no for the other 52 |
| Can the MSAA walk? | the Name third of 4.1.2 and nothing else |
| Can the NVDA suite? | see below |
| Otherwise | a person, or nothing |

### A reproducibility problem the list will inherit

`accessibility.yml` fetches the **latest** Axe.Windows release at run time. The rule set is
therefore whatever Microsoft shipped most recently and can change without a commit in this
repository. It changed under this document: the rule table read on 2026-09-06 and the one
read on 2026-09-12 do not agree, and at least part of that is measurement error rather than
upstream drift, which is precisely the ambiguity an unpinned dependency creates. A document
saying "the scan judges these three criteria" is a claim about a version nothing pins.

Two options: pin the CLI version in the workflow and record which version the list was
written against, or state the version and date beside the list. Pinning is better, removes
an unpinned third-party binary from CI, and is a two-line change.

### The NVDA suite is three tests that run and one that is skipped

```
ls nvda-tests/tests/
-> account-manager-sign-in-failure.test.js
   calendar-immediate-actions.test.js
   filter-manager-delete.test.js
   which-days-focus-and-tick.test.js
```

Three `test(` declarations across the first three files
[VERIFIED: `grep -rhoE "^\s*(test|it)\(" nvda-tests/tests/*.js`, run 2026-09-12]:

- "NVDA announces Signing in failed when Sign In Again cannot reach a provider"
- "NVDA hears Edit Event"
- "NVDA announces the same sentence Delete would show when nothing is selected"

**The fourth file holds one `test.skip`**, at
`nvda-tests/tests/which-days-focus-and-tick.test.js:98`, and its own header says why,
verbatim [VERIFIED: nvda-tests/tests/which-days-focus-and-tick.test.js:1-21]:

> It is skipped because opening this dialog needs an event that repeats, selected, with an
> edit or delete already under way [...] None of them is reachable from a fresh profile the
> way `--scan-target accounts` reaches the Account Manager: `src/presentation/scan_target.rs`
> has no target for this dialog.

This settles the previous version's assumption A7, and it hands criterion 4 a named gap: the
"which days do you mean" dialog is reachable by neither scan channel nor the NVDA suite,
and a `ScanTarget` for it would unblock both.

---

## Criterion 4: the five WebView2 findings, and the manual list

### The five findings are enumerated nowhere in the tree. Re-confirmed.

Searched four ways on 2026-09-12: `grep -rn "WebView2" docs/*.md`, `grep -rn "five
finding\|Five accessibility" docs/ .planning/ CLAUDE.md`, `grep -rn "Chrome_WidgetWin\|
BrowserRootView"` across every markdown, Rust, PowerShell and YAML file, and a look for any
`.a11ytest` artifact in the tree.

There is exactly one line that describes them, `docs/changelog.md:10559`, verbatim
[VERIFIED: docs/changelog.md:10559]:

> Five accessibility scan findings remain, all inside WebView2's own accessibility tree
> (`Chrome_WidgetWin_1`, `BrowserRootView`, and three container views). They are not this
> application's controls and cannot be named or positioned from here.

That names two and calls the other three "three container views". A second summary sits at
`docs/IMPLEMENTATION_STATUS.md:171-173`, which repeats the count and the date and names
none of them.

**So the criterion's "each of the five findings" cannot be discharged from documents.** The
scan has to be run and the artifacts read.

### Why the number is not merely stale

```
git log -S "Five accessibility scan findings remain" --format='%h %ad %s' --date=short \
  -- docs/changelog.md
-> 44ed93f 2026-07-26 fix(a11y): make check menu items tell the truth about their state
```

The line was written 2026-07-26. The scan was pointed at dialogs on 2026-07-28 and the MSAA
channel was added on 2026-07-31. **So "five findings" is a count from a scan that covered
one of the eleven windows it now covers, on one of the two channels it now walks.** The
number is not stale so much as measuring something else. It will very likely grow, and that
should be said in the commit that replaces it, or it will read as a regression.

### What can be said about the two named ones, and the upstream

The brief asked that each finding be marked fixable-here or upstream with the upstream
named. Three of the five cannot be identified at all until the scan is re-run. The two that
are named can be reasoned about now, and the upstream can be named now, which is the part
that unblocks planning.

**`Chrome_WidgetWin_1` is the Win32 window class Chromium uses for its host window, and
`BrowserRootView` is a Chromium Views class.** Both are produced by the Edge WebView2
runtime's own accessibility tree, not by any wxWidgets control this application creates.
Nothing in `src/` can set a name or a bounding rectangle on either. [ASSUMED: this is an
inference from the names and from the changelog line's own claim that they "are not this
application's controls and cannot be named or positioned from here". It is not verified by
running anything, and it should be confirmed against the artifact when the scan is re-run.]

**The upstream to name is `MicrosoftEdge/WebView2Feedback` on GitHub**, which is Microsoft's
official feedback and issue repository for WebView2, described on its own README as the
place for developers to report bugs, make feature requests, and ask questions about
WebView2. It carries accessibility issues already, including one titled "A11Y: WebView2
control is completely inaccessible with screen readers (NVDA, JAWS and parcially with
Narrator too)" as issue 2330, and it labels issues it is tracking with a `tracked` label.
[CITED: https://github.com/MicrosoftEdge/WebView2Feedback and
https://github.com/MicrosoftEdge/WebView2Feedback/issues/2330]

So guardrail 9's "with the upstream named" has a concrete answer for this phase: a finding
judged upstream is recorded against `MicrosoftEdge/WebView2Feedback`, with an issue number
where one already covers it and a newly filed issue where none does. The deeper upstream is
Chromium itself, but a WebView2 consumer files against WebView2Feedback, and filing against
`crbug.com` is the wrong door.

**What a plan should therefore contain, and what it cannot.** It can contain: re-run the
scan, read the artifacts, produce the list, and for each entry record either a fix or an
upstream reference against the named repository. It cannot contain: the five findings, their
identities, or a promise that there will be five.

### The windows the scan still does not reach

Eleven targets against 24 modules matching `src/presentation/wx_*.rs`
[VERIFIED: `ls src/presentation/wx_*.rs | wc -l` gives 24, run 2026-09-12]. Not all 24 are
windows. These build dialogs and are not scan targets: `wx_columns`,
`wx_conflict_choice`, `wx_destination`, `wx_folder_choice`, `wx_item_form`, `wx_managers`,
`wx_reminder_alert`, `wx_thread_view`, `wx_which_days`.

Three worth naming. `wx_item_form` is where somebody enters an event, contact, task or note,
and is the one dialog `tests/checkbox_labels.rs` covers, so its checkboxes are tested and
its scan coverage is absent. `wx_reminder_alert` is a modal that opens on a timer, and is
inherited item B's subject. `wx_which_days` is the dialog the skipped NVDA test names.

Whether widening the target list is part of this phase is a scoping choice. The coverage
document is dishonest if it does not say which windows are outside it either way.

### A commit-gate gap on this phase's path, still open

`scripts/which-checks.sh` answers `affected` for a commit touching
`.github/workflows/accessibility.yml`, because the file is neither `*.md` nor `*.txt`.
`check.sh`'s `run_the_tests_that_reach_what_changed` maps exactly two path shapes to tests,
`src/*.rs` to `cargo test --lib module::` and `tests/*.rs` to `cargo test --test target`,
plus the integration suites `guards/guards.toml` couples to a changed source module
[VERIFIED: scripts/check.sh:387-432, read 2026-09-12]. A `.yml` path matches none of them.

`src/presentation/scan_target.rs` carries **0** guard records, so the coupling path does not
rescue it either (measured below).

**So a commit that changes only that workflow runs neither of the two tests that could catch
it disagreeing with `scan_target.rs`.** Criterion 3 changes that workflow, so this phase will
meet the gap. This was reported by the 2026-09-06 research and is still open five days and
355 commits later, re-verified today. Cheapest fix matching the existing `Cargo.toml`
precedent: make any `.github/workflows/*` change answer `all`.

---

## Where this phase collides with phase 7

Phase 7 is executing in parallel with nine written plans in
`.planning/phases/07-installing-updating-and-what-is-stored/`. The brief named the expected
collision surface as the About dialog, `src/common/version.rs`, the accessibility bridge's
accessors, and pages listing what is on a user's disk. **Those are real and they are not the
main one.**

Measured 2026-09-12 by extracting every source path phase 7's own plans name and ranking by
frequency:

```bash
grep -ho 'src/[a-z_/]*\.rs\|docs/[a-z_]*\.md' \
  .planning/phases/07-installing-updating-and-what-is-stored/*.md \
  | sort | uniq -c | sort -rn
```

| Phase 7 mentions | File | Does phase 6 need it? |
|---|---|---|
| 51 | `docs/changelog.md` | **Yes.** Every user-visible change in both phases |
| 40 | `scripts/check.sh` | Yes, criterion 3's gate gap |
| 30 | `src/common/version.rs` | No |
| 20 | `src/presentation/wx_app.rs` | Yes, inherited item B |
| 20 | `src/data/config.rs` | **Yes, and this is the collision** |
| 15 | `scripts/which-checks.sh` | Yes, criterion 3's gate gap |
| 9 | `src/presentation/wx_settings.rs` | **Yes, criterion 1's panel** |
| 9 | `src/presentation/accessibility/screen_reader.rs` | Read only for this phase |
| 8 | `src/presentation/accessibility.rs` | Read only for this phase |
| 8 | `src/presentation/accessibility/names.rs` | Read only for this phase |
| 6 | `src/presentation/accessibility/platform_bridge.rs` | **Does not exist yet** |

**The collision is `mod every_setting_is_acted_on` in `src/data/config.rs`.** Phase 7's plan
07-05 adds a new top-level `AppConfig` field for the release channel and a control for it,
and says so in its own words [VERIFIED: .planning/phases/07-installing-updating-and-what-is-stored/07-05-PLAN.md:48]:

> AppConfig -> every_setting_is_acted_on: a new top-level field fails
> test_every_setting_somebody_can_change_is_read_by_something and
> test_every_setting_somebody_can_change_is_offered_by_a_screen on arrival, one for being
> read by nothing and one for being offered by no screen. That is the RED half for free

That is the identical mechanism inherited item A relies on, in the identical module, and
07-05's own reading list names every constant and every test in it
[VERIFIED: .planning/phases/07-installing-updating-and-what-is-stored/07-05-PLAN.md:508].
Both phases will edit the three exception lists. Neither side's documents mention the other.

**`platform_bridge.rs` does not exist today.** `ls src/presentation/accessibility/` gives
`announcements.rs automation.rs feedback.rs focus.rs keyboard.rs names.rs screen_reader.rs
sound_scheme.rs sound_scheme_import.rs`, run 2026-09-12. Phase 7's criterion 6 asks that the
application derive the bridge's presence from what is compiled in, and its plans name a
module that will be created for it. **So `screen_reader.rs` is likely to be restructured
under this phase's feet.** Nothing in phase 6 as scoped needs to change that file, and the
recommendation is to keep it that way.

### What to re-take rather than re-read, if phase 7 lands first

A short, explicit list, so a planner does not re-verify this whole document.

| Re-take | Because |
|---|---|
| Every line number in `src/data/config.rs` cited above | 07-05 adds a field and edits the test module |
| Every line number in `src/presentation/wx_settings.rs` cited above | 07-05 adds a control; 07-09 touches the Help path |
| `OFFERED_BY_ANOTHER_SCREEN`'s length, today 3 | 07-05 may add an entry |
| The guard record counts for `config.rs` and `wx_settings.rs`, today 4 and 3 | new tests mean new records |
| Anything about `screen_reader.rs`'s shape | `platform_bridge.rs` may take part of it |
| `Cargo.toml`'s version, today `0.112.0` | phase 7 bumps it; every commit touching it answers `all` |
| The About dialog at `wx_app.rs:21261` and `src/common/version.rs:32` | phase 7 owns both. Phase 6 needs neither |
| `docs/IMPLEMENTATION_STATUS.md` line numbers | both phases edit it |

Nothing about criterion 1's model (`feedback.rs`), criterion 2's dates (`date_display.rs`,
`occurrences.rs`, `ui_types.rs`, `wx_item_form.rs`) or criterion 3's scan
(`accessibility.yml`, `scan_target.rs`, `msaa-names.ps1`) appears in phase 7's plans at all.
**Those three are collision-free and can be planned without waiting.**

---

## Where the work concentrates, and what that implies for waves

Every phase here serialises on `guards/guards.toml`, `docs/changelog.md` and `Cargo.toml`.
This phase adds no dependency, so `Cargo.toml` should never be touched, and that matters:
a commit touching it answers `all` and pays the full gate.

| Piece | Files it concentrates in | Collides with |
|---|---|---|
| Criterion 1, the model additions | `src/presentation/accessibility/feedback.rs` | nothing |
| Criterion 1, the panel | `src/presentation/wx_settings.rs` | **phase 7 plan 07-05** |
| Criterion 1, the guard | `src/data/config.rs` | **phase 7 plan 07-05**, and inherited item A |
| Inherited item A | `src/data/config.rs`, `src/presentation/wx_account_manager.rs`, `tests/house_style.rs`, product docs | **phase 7, and criterion 1's guard** |
| Inherited item B | `src/presentation/wx_app.rs`, `src/presentation/one_question_at_a_time.rs` | phase 7 touches `wx_app.rs` |
| Criterion 2 | `src/presentation/date_display.rs`, plus three call sites | nothing |
| Criterion 3 and 4 | `.github/workflows/accessibility.yml`, `scripts/which-checks.sh`, `scripts/check.sh`, four product docs | `check.sh` and `which-checks.sh` with phase 7 |

**Wave implications.**

1. **`src/data/config.rs` is the single serialisation point of this phase.** Criterion 1's
   guard and inherited item A both edit `mod every_setting_is_acted_on`, and so does phase 7.
   Put both phase 6 tasks that touch it in the same plan and the same wave, and sequence that
   plan against 07-05 explicitly rather than hoping.
2. **Criterion 2 is a clean parallel wave.** `date_display.rs` plus `occurrences.rs`,
   `ui_types.rs`, `wx_item_form.rs` and `signed_mail.rs`. Nothing else in either phase wants
   these files.
3. **Criteria 3 and 4 are a clean parallel wave for the document half**, and the workflow half
   touches `check.sh` and `which-checks.sh`, which phase 7 also names. Small edits, but worth
   knowing.
4. **Keep each commit to one kind of file.** The phase 2.1 lesson: a commit mixing a document
   correction with three source modules makes every commit in that plan pay for all of them.
   The document corrections in criterion 3 are a documents-only commit if nothing else rides
   with them.

### Which checks each commit earns, re-verified 2026-09-12

| A commit touching | `which-checks.sh` answers |
|---|---|
| only `*.md` or `*.txt`, on a branch or on `main` | `docs_only` |
| any `.rs` on a branch | `affected` |
| `.github/workflows/*.yml` on a branch | `affected`, and selects no tests for it: see the gap above |
| `scripts/*.ps1` on a branch | `affected`, which selects no tests for it |
| `Cargo.toml` or `Cargo.lock`, anywhere | `all` |
| anything non-markdown on `main` | `all` |

**This document is itself inside the em-dash guard's reading.** `tests/house_style.rs:50`
walks `.planning` for markdown [VERIFIED: tests/house_style.rs:34-50], which it did not do
before 2026-09-07. There are zero em dashes anywhere under `.planning` today, and there are
none in this file. A plan written for this phase must stay that way.

`test_nothing_offers_a_setting_per_account_that_no_screen_writes` reads
`the_pages_that_speak_for_the_product()`, which filters out `.planning`
[VERIFIED: tests/house_style.rs:112 and the test's own comment], so planning documents may
quote the two forbidden phrases. Product documents may not, until the control exists.

---

## Cost facts a plan will need

### Guard records, counted as records rather than as mentions

Counted 2026-09-12 at `febe8e4` by parsing `tests_last_seen` blocks, which is what
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` actually reads. Never
by grepping a file name: `CLAUDE.md` warns a grep overcounts by up to five to one and it is
right.

```
awk '/^\[\[guard\]\]/ {c++} END {print c}' guards/guards.toml
-> 720
```

720 records. The census comment at `guards/guards.toml:79-80` reads "192 records were swept
that day" and "528 records have arrived since and have not been through it", and 192 + 528 =
720, so the file's own two numbers agree with its record count today.

| File a task might add a test to | Records flagged | Was, 2026-09-06 |
|---|---|---|
| `src/presentation/accessibility/feedback.rs` | **1** | 1 |
| `src/presentation/one_question_at_a_time.rs` | **1** | reported as 0, and under the wrong path |
| `src/presentation/wx_settings.rs` | **3** | 1 |
| `src/presentation/date_display.rs` | **3** | 3 |
| `src/application/occurrences.rs` | **3** | 3 |
| `src/application/due.rs` | 3 | not reported |
| `src/data/config.rs` | **4** | 2 |
| `src/presentation/accessibility/names.rs` | 4 | 4 |
| `src/presentation/accessibility.rs` | 7 | 7 |
| `tests/house_style.rs` | **18** | 18 |
| `src/presentation/wx_app.rs` | **48** | 40 |
| `src/presentation/scan_target.rs` | **0** | not reported |
| `src/presentation/accessibility/sound_scheme.rs` | **0** | not reported |

Three of these moved in five days. `config.rs` doubled and `wx_app.rs` gained eight.

**The planning consequence.** Most of what this phase wants is cheap: 1, 1, 3, 3, 4. The two
expensive ones are `wx_app.rs` at 48 and `tests/house_style.rs` at 18.

- Put the per-event guard in `src/data/config.rs` beside its siblings, at 4 records, not in
  `tests/house_style.rs` at 18.
- Keep inherited item B's test in `src/presentation/one_question_at_a_time.rs` at 1 record,
  not in `wx_app.rs`'s test module at 48. `what_to_raise` already takes its conditions as
  arguments precisely so they can be tested without a window.
- If criterion 2 wants a guard asserting no shipping site formats an English month outside
  `date_display`, weigh `tests/house_style.rs` at 18 against a unit test beside a source
  module.
- `scan_target.rs` carrying 0 records is itself worth a plan's attention: a test there is
  reached only when its own file changes, which is the coupling gap described above.

### The records for this phase's files are all in step today

This is a measurement worth having, because a record that has fallen behind turns a cheap
task into an expensive one without warning. Run 2026-09-12, comparing what each record wrote
down against a count taken by the check's own rule, which counts lines that are exactly
`#[test]` or `#[tokio::test]` after trimming:

| File | Records claim | Tree holds |
|---|---|---|
| `src/presentation/accessibility/feedback.rs` | 41 | 41 |
| `src/data/config.rs` | 60 | 60 |
| `src/presentation/date_display.rs` | 37 | 37 |
| `src/application/occurrences.rs` | 62 | 62 |
| `src/presentation/one_question_at_a_time.rs` | 19 | 19 |
| `tests/house_style.rs` | 67 | 67 |

**All in step.** So this phase starts from a clean baseline, and the first commit that adds a
test to any of these files will flag its records and print the `scripts/guards.sh --remeasure`
command. Running that remedy when a commit prints it is not optional: `CLAUDE.md` says it is
the reason the sweep can be deferred at all.

A caution on how to count. A naive `grep -c '#\[test\]'` gives 76 for `tests/house_style.rs`
against the check's 67, because that file quotes the attribute inside string literals. Use
the check's own rule or the number will look nine out of step when it is not.

### Guard sweeps

`scripts/guards.sh` unfiltered is 720 records as of 2026-09-12. **This document does not
quote a duration**, and a plan should not either without measuring. `CLAUDE.md` gives the
reason at length: the per-record rate roughly halved on 2026-09-09 when test caches stopped
rebuilding the database schema, the whole-tree total has been quoted at 15, 16 and 20 hours
at different record counts, and both the count and the rate go on moving. Multiply the count
you take today by a rate you measure today.

Per the decision of 2026-09-03, the sweep is one sweep once every phase is complete, not per
merge, and the executor does not run guards.

---

## Project constraints from CLAUDE.md that bear on this phase

Each of these changes what a plan may contain.

- **Red/green TDD on every eligible task.** `.planning/config.json` has
  `"tdd_mode": true`, read this session. Red commits are refused on `main`, must name every
  failing test in `Fails-until-green:` trailers, and are checked in three directions by
  `scripts/red-commit.sh`.
- **`scripts/check.sh` is the gate.** Four checks, clippy at `-D warnings`. Never pipe it.
  Mode chosen by `scripts/which-checks.sh`.
- **`cargo test` takes one `--lib`.** Several module paths need several runs joined with
  `&&`. `CLAUDE.md` records that 55 plans told executors to pass several and none could ever
  have run. Every `<verify><automated>` block in this phase's plans must obey this.
- **The accessibility layer is Windows-only in two specific ways**, and `date_display.rs`
  already has the same shape: a `#[cfg(target_os = "windows")]` reader with a non-Windows
  fallback at `:232`. Any `GetDateFormatEx` work follows that existing pattern exactly.
  Phase 7's criterion 5 asks that the crate build on Linux and macOS, so this is not
  optional politeness.
- **Every setting is reachable from a section somebody would look in.** This binds the plan
  that introduces the setting. Phase 1's criterion 8 said a phase must not add a third
  unreachable setting; phase 6 removes both existing ones.
- **Schema changes are additive.** Nothing in this phase touches `message_cache.db`.
- **Feedback must be distinct and bounded** (guardrail 5). A per-event panel lets somebody
  switch Speech on for all sixteen events, which is a panel that can produce flooding. The
  pacing already in `announcements.rs` is what stops it. A plan should not assume the panel
  is the place to solve it, and should not remove the pacing.
- **Do not silently absorb upstream failures** (guardrail 9). The five WebView2 findings are
  the worked example, and the upstream now has a name: `MicrosoftEdge/WebView2Feedback`.
- **A user-visible change gets a `docs/changelog.md` entry under `[Unreleased]` in the same
  commit**, honest "Known limitations" included.
- **No AI attribution anywhere.** No `Co-Authored-By` naming an AI, in any commit, branch
  name, comment or document.
- **No em dashes in any markdown under `.planning` or `docs`.** Enforced, and it now reads
  `.planning`.
- **`docs/KEYBOARD_SHORTCUTS.md` is updated in the same commit as a shortcut.** The
  recommended panel adds no shortcut, so this should not fire.

---

## Validation architecture

`workflow.nyquist_validation` is not set in `.planning/config.json`, so it is enabled.

| Property | Value |
|---|---|
| Framework | `cargo test --all-targets`; `tokio-test` for async, `tempfile` for the filesystem |
| Config | none; unit tests in `#[cfg(test)] mod tests` beside the code, cross-layer in `tests/` |
| Scoped run | `bash scripts/check.sh` |
| Full suite | `bash scripts/check.sh all`, run by whoever merges |
| Guard sweep | `scripts/guards.sh`, once per completed phase, not per merge |
| Env | `WIXEN_TEST_THREADS` defaults to 8 since 2026-09-09; `WIXEN_NO_AUDIO` where a sound device opens and does not work, and it is not a way to skip the sound tests |

| Criterion | Behaviour | Type | Where | Records flagged |
|---|---|---|---|---|
| 1 | `Event::ALL` holds every variant of `Event` | unit | `feedback.rs` | 1 |
| 1 | A per-event override round trips through `to_stored` and `from_stored` | unit, **exists** (`:1078`) | `feedback.rs` | 1 |
| 1 | Clearing an override returns the event to the default, not to silence | unit, **new method needed** | `feedback.rs` | 1 |
| 1 | The chosen channels can be read back distinctly from the effective ones | unit, **new method needed** | `feedback.rs` | 1 |
| 1 | The panel writes an override and reads it back, on the `config.rs:2220` four-assertion pattern | guard | `src/data/config.rs` | 4 |
| 1 | Every checkbox on the settings screen carries its own label | integration | widen `tests/checkbox_labels.rs`, or record the gap | needs a new record |
| A | A per-account Allow Changes answer written by the account manager is read by `allowed_for` | unit | `config.rs` | 4 |
| A | `STORED_AND_OFFERED_BY_NOTHING` is emptied and its guard retired or given a companion | guard | `config.rs` | 4 |
| B | A reminder due while somebody is typing is not raised, and is raised on the next tick after typing stops | unit, no window needed | `one_question_at_a_time.rs` | 1 |
| 2 | `date_part` under a forced `en-US` produces the English string; under `fr-FR`, French | unit | `date_display.rs` | 3 |
| 2 | A day-first verbal picture gets the genitive month in a language that has one | unit | `date_display.rs` | 3 |
| 2 | No shipping site outside `date_display` formats a date with an English month or day name | guard | weigh `tests/house_style.rs` at 18 against a unit test | 18 or fewer |
| 3 | The coverage document names a criterion, and a planted wrong criterion is caught | guard, needs a companion proving the reading works | to be decided | varies |
| 3 | The workflow's target list and `ScanTarget::ALL` still agree | unit, **exists already** | `scan_target.rs` | 0 |

**Wave 0 gaps:** no framework gap. The gaps are record-shaped. Anything written under
`tests/` needs a `guards/guards.toml` record or the gate will not run it on the commits that
could break it, and `scan_target.rs` has none today.

---

## Security domain

Small surface. This phase adds no network path, no parser and no new untrusted input.

| ASVS | Applies | Control here |
|---|---|---|
| V5 Input validation | Partly | `FeedbackSettings::from_stored` deliberately ignores anything unrecognised, so a settings file written by a newer version does not throw away the rest of somebody's settings. `test_stored_settings_ignore_names_they_do_not_recognise` (`feedback.rs:1464`) holds it. **A per-event UI must not tighten this into a refusal.** |
| V5 Input validation | Yes | `GetDateFormatEx` writes into a caller-supplied buffer and returns a length. Size with `cchDate = 0` first, or use a fixed buffer and check the return. `read_locale` already establishes the pattern |
| V6 Cryptography | No | nothing here |
| V1 Architecture | Yes | The scan workflow downloads and executes an unpinned third-party binary from a GitHub release on every run. A supply-chain surface as well as a reproducibility problem, and pinning fixes both |

| Pattern | STRIDE | Mitigation |
|---|---|---|
| Unpinned CLI fetched and run in CI | Tampering | Pin the release tag and record the version beside the coverage list |
| A hand edited `feedback_channels` string reaching `from_stored` | Tampering | Already handled: unknown events and channels are skipped, not fatal |
| A locale name reaching `GetDateFormatEx` | Tampering | Never take one from a message or a file. `LOCALE_NAME_USER_DEFAULT` in shipping code, a literal in tests |

**Package legitimacy audit: not applicable.** This phase as scoped adds no package to
`Cargo.toml`. If plural rules are taken on for relative wording, the audit runs then, on
whatever crate is proposed, and that choice is Pratik's.

---

## Environment availability

No new external dependency is needed by any recommendation here.

| Dependency | Required by | Present | Notes |
|---|---|---|---|
| `GetDateFormatEx` in `Kernel32.dll` | criterion 2 | Yes, Windows Vista and later | Same shape as the existing `GetLocaleInfoW` block |
| `chrono` | already a direct dependency | Yes | Used without `unstable-locales`, and this document recommends keeping it that way |
| Axe.Windows CLI | criterion 3 | Downloaded per run from the latest GitHub release | Not pinned; see the reproducibility note |
| NVDA | criterion 4 | Only on the GitHub-hosted runner | `nvda-tests/README.md` says why it must never run on a developer's machine |
| A non-English Windows machine or locale | criterion 2's real verification | **Not available to an executor** | `GetDateFormatEx` takes a locale name, so tests can force one; a real end-to-end check needs a French Windows and a French voice |

---

## What no plan in this phase can close

Named precisely rather than glossed, per guardrail 9 and the pattern phases 3 and 4 set.
Each of these belongs in the phase README's own such section and each should become a
`unrun-verify` entry in `.planning/WINDOWS.md`, which held 301 entries, 280 of them open,
208 of those `unrun-verify`, when read on 2026-09-12.

1. **Whether the per-event panel is usable.** Whether a Choice of sixteen followed by four
   checkboxes reads well; whether the reloaded checkbox states are announced or silently
   change under the cursor; whether the effective-channels sentence is heard as helpful or as
   noise. Reloading four checkbox states without moving focus is a live-region-shaped problem
   and only a listening pass settles it.
2. **Whether the effective-versus-chosen distinction is understood.** The sound-only fallback
   means the panel must explain something subtle. No test can tell whether the explanation
   lands.
3. **Whether the new checkboxes are named on both channels in practice.** A test can prove a
   label was set. Only Narrator proves UI Automation reads it and only NVDA proves MSAA does.
4. **Whether localised dates sound right.** A French month name read by a French voice, in a
   date whose order came from the same machine, is what FEEDBACK-02 exists for. It needs a
   French Windows, a French NVDA voice and somebody who speaks French. Nothing here has one.
5. **Whether the genitive really arrives.** Microsoft documents that it does. Nothing in this
   project has ever called `GetDateFormatEx` on a Russian or Polish locale.
6. **Whether the sixteen earcons are distinguishable.** Already disclosed in the product,
   which asks for reports, still unmeasured, and a per-event panel makes it more likely
   somebody switches sounds on for all sixteen.
7. **The five WebView2 findings themselves.** Each has to be looked at and judged: ours, or
   upstream with a named issue. The scan produces the artifact; the judgement is a person's.
8. **The scoped manual list.** Criterion 4 asks for the list of interactions only a human
   pass can walk. That is a judgement about what matters, not a derivation, and FEEDBACK-03
   already says "No criterion here claims the manual pass has happened. Pratik decides when
   screen reader testing runs."
9. **Whether a reminder should wait.** Decision, not a task. See below.

---

## Decisions for Pratik

These change what gets built and are not a planner's to settle. Each should sit at a
`checkpoint:human-verify` rather than be guessed.

1. **What relative wording does in a non-English locale.**

   | Option | Cost | What it gives |
   |---|---|---|
   | Keep it English, reword `ENGLISH_ONLY` to say only that | Small. One constant, one settings label | Month and day names correct, an English phrase still inside a French sentence. Better than today, not the criterion as written |
   | Fall back to the absolute date when the locale is not English | Small. `relative_to` returns `None`, which the caller already handles | Nothing ungrammatical is ever spoken. Loses the "is this recent" affordance the module's header says is the whole point of relative wording |
   | Take on plural rules | Large. A new dependency or a hand-written rules table, and a `dependency-audit` conversation | The criterion as written |

   This is a decision about what a date is for, not a technical one.

2. **Whether the per-event panel reverses `build_feedback_tab`'s stated position or answers
   it.** That comment argues a grid of forty checkboxes is not worth it, and it is right about
   the grid. The Choice-plus-four-boxes shape answers the objection rather than overruling it.
   A full sixty-four-checkbox grid is a different control and a different accessibility
   problem, and if that is wanted it should be said now.

3. **Whether a per-account Allow Changes control is three answers per account or one.**
   `Allowed` has three fields and `allowed_for` can only narrow. Three is honest and is more
   surface on the account manager. One toggle meaning "this account may change less than the
   application default" is smaller and cannot express what the model can hold. The deferred
   item was written when `Allowed` had fewer fields.

4. **Whether a reminder waits for typing to stop.** The question has not moved: a reminder
   that waits can be an hour late, and being told at the time is the point. Options are wait,
   raise without focus, or hold briefly and raise anyway. The first is small and the code is
   ready for it, because nothing is recorded until the window opens. The second is medium.

5. **Whether to widen the scan's target list in this phase.** Eleven windows are scanned and
   at least nine more dialogs exist, including the item form, the reminder alert and the one
   the skipped NVDA test names. Widening makes the coverage list stronger, adds scan time and
   probably adds findings. Leaving it means the coverage document must say which windows are
   outside it, which is the honest minimum either way.

6. **Whether to pin the Axe.Windows CLI version.** Pinning makes the coverage list a claim
   about something reproducible and removes an unpinned binary from CI. Not pinning means the
   list needs a version and a date beside it and will drift. Two lines either way. The rule
   table demonstrably differs between two reads five days apart, which is the argument for
   pinning.

7. **Whether `REQUIREMENTS.md` is corrected in place.** FEEDBACK-02's evidence names one file
   where there are four, FEEDBACK-03's "roughly half" is the figure this phase disproves, and
   FEEDBACK-01's evidence calls a struct field a function. Phases 3 and 4 recorded such
   corrections in each plan's `<premise_corrections>` and left the requirements as written.
   Same again, or amend the document this time?

8. **Whether the coverage list is a document or a check.** A document goes stale the way the
   five findings did. A check that reads the list costs guard records and needs a companion
   proving it can see a violation, because a list-reading guard with an empty list passes
   unconditionally, which this repository has already fallen into twice.

9. **How phase 6 and phase 7 share `src/data/config.rs`.** Both want `mod
   every_setting_is_acted_on` and both want a new top-level field's arrival as their red half.
   Options: sequence phase 6's settings work after 07-05 merges; agree a split now; or accept
   a merge conflict in a 500-line test module. This is a scheduling decision and it is the
   single largest risk to this phase.

---

## What the 2026-09-06 version of this file got wrong

Listed so the correction does not have to be rediscovered, and because several of these
would have become false premises in a plan.

1. **The Axe.Windows rule breakdown.** It reported 144 rules and a breakdown of 81 / 28 / 19
   / 14 / 13 which sums to 155, not 144. The correct figure today is 155 rules split
   61 / 53 / 23 / 9 / 9. Its qualitative conclusion, three WCAG criteria and not half,
   survives unchanged.
2. **`wx_app.rs:1846`, an English date site that no longer exists.** Removed by `e94b4ae`
   during phase 5.2. A plan scoped from that list would have sent an executor to a missing
   line.
3. **A new English date site it could not have known about.** `a_month_in_words` at
   `date_display.rs:396`, called from `ui_types.rs:1205`.
4. **`per_event` described as a function.** It is a struct field. So are `REQUIREMENTS.md`
   and `CLAUDE.md` wrong about this. The practical consequence is that "make it public" is
   not a coherent instruction.
5. **"Two `pub` keywords and one settings panel."** There is no public reader of the chosen
   channels at all, and no method that clears an override. Three methods, one of which does
   not exist in any form.
6. **`one_question_at_a_time` placed under `src/application/` and priced at 0 guard
   records.** It is `src/presentation/one_question_at_a_time.rs` and carries 1.
7. **Guard record counts for three files.** `config.rs` 2 to 4, `wx_settings.rs` 1 to 3,
   `wx_app.rs` 40 to 48.
8. **Ten scan targets and nine dialogs.** Eleven and ten now, `blocked-senders` added.
9. **WCAG Level A and AA totalling 56.** It flagged this as needing re-derivation and it was
   right to: the answer is 55, because 4.1.1 Parsing is obsolete and removed in WCAG 2.2.
10. **WCAG2ICT described as determining what does not apply.** It explicitly says it does not.
    It records what Section 508 and EN 301 549 decided, and those two lists differ from each
    other.
11. **Assumption A7, the NVDA suite's size.** Settled: three tests that run and one
    `test.skip` with a documented reason.
12. Every line number it gave for `config.rs`, `wx_settings.rs`, `wx_app.rs` and
    `house_style.rs`. All moved, several by hundreds of lines.

Nothing in that document was carelessly written. Most of these are the tree moving under a
document that dated itself honestly, which is exactly what it warned would happen.

---

## Assumptions I could not verify

| # | Assumption | Cost if wrong |
|---|---|---|
| A1 | GitHub's `windows-latest` runner has NLS data for arbitrary locale names such as `fr-FR`, so a test can force one without a language pack | The cross-locale tests in criterion 2 cannot run in CI and become manual. Settle with a throwaway run before planning tasks around it |
| A2 | `GetDateFormatEx` with a picture of `d MMMM yyyy` really returns the genitive month on a Russian locale | The genitive argument, which is the main reason for choosing the API over a table. Microsoft documents it explicitly; nothing here has run it |
| A3 | `Chrome_WidgetWin_1` and `BrowserRootView` are Chromium-owned and unreachable from this code | Two of the five findings' dispositions. Corroborated by the changelog line's own claim; confirm against the artifact |
| A4 | Axe.Windows' `RulesDescription.md` is generated from the shipping rule set, so the three-criteria figure describes what actually runs | The whole of criterion 3's headline. The file is auto-generated in the same repository, which is corroboration rather than proof |
| A5 | Re-running the scan today produces a comparable set of WebView2 findings rather than a different set | Criterion 4's "the five findings". The scan now covers eleven windows on two channels, so expect to *replace* the number rather than confirm it |
| A6 | The four checkboxes plus a Choice are announced coherently by NVDA when the Choice changes and the boxes reload underneath it | Criterion 1's usability, not its correctness. Only a listening pass settles it |
| A7 | `chrono`'s `unstable-locales` is still unstable and still sources from `pure-rust-locales` | The argument against it. Carried forward from 2026-09-06 and not re-fetched. Nothing in the recommendation depends on it, since the recommendation is a Win32 call |
| A8 | `workflow.windows_enforce` is off, since it is absent from `.planning/config.json` | Whether `/gsd-ship` blocks on 280 open ledger entries. The key is genuinely absent; the default was not checked |

---

## Sources

### Primary, HIGH confidence, read from the tree this session at `febe8e4`

`src/presentation/accessibility/feedback.rs`, `src/presentation/accessibility.rs`,
`src/presentation/accessibility/screen_reader.rs`, `src/presentation/accessibility/names.rs`,
`src/presentation/accessibility/sound_scheme.rs`, `src/data/config.rs`,
`src/presentation/wx_settings.rs`, `src/presentation/wx_app.rs`,
`src/presentation/wx_compose.rs`, `src/presentation/date_display.rs`,
`src/presentation/one_question_at_a_time.rs`, `src/presentation/scan_target.rs`,
`src/application/occurrences.rs`, `tests/checkbox_labels.rs`,
`tests/the_conflict_choice_can_be_heard.rs`, `tests/house_style.rs`,
`scripts/msaa-names.ps1`, `scripts/check.sh`, `scripts/which-checks.sh`,
`.github/workflows/accessibility.yml`, `nvda-tests/tests/`, `guards/guards.toml`,
`.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/WINDOWS.md`,
`.planning/config.json`, `.planning/decisions-2026-09-06.md`,
`.planning/phases/01-folders-and-conversations/deferred-items.md`,
`.planning/phases/07-installing-updating-and-what-is-stored/*.md`, `CLAUDE.md`.

### Primary, HIGH confidence, fetched and parsed with a script this session

- `https://raw.githubusercontent.com/microsoft/axe-windows/main/docs/RulesDescription.md`,
  fetched 2026-09-12, counted with the awk above: 155 rules, five standards, three WCAG.
- `https://www.w3.org/TR/WCAG22/`, fetched 2026-09-12, parsed: 87 success criterion
  sections, A=31, AA=24, AAA=31, 4.1.1 obsolete and carrying no level.
- `https://www.w3.org/TR/wcag2ict-22/`, fetched 2026-09-12, quotations extracted verbatim.
  W3C Group Note, 11 December 2025.
- `https://learn.microsoft.com/en-us/windows/win32/api/datetimeapi/nf-datetimeapi-getdateformatex`,
  fetched 2026-09-12, quotations extracted verbatim.

### Secondary, MEDIUM confidence

- `https://learn.microsoft.com/en-us/windows/win32/intl/locale-smonthname-constants`, the
  genitive note. Quoted from the 2026-09-06 read and consistent with the `GetDateFormatEx`
  page fetched today.
- `https://github.com/MicrosoftEdge/WebView2Feedback` and issue 2330, found by search and
  confirmed as Microsoft's own feedback repository from its README description.

### Tertiary, LOW confidence

- The `chrono` `unstable-locales` argument, carried forward and not re-fetched.

---

## Metadata

**Confidence breakdown:**

- Criterion 1, the model and the events: HIGH. Every claim re-derived from source this
  session with the command recorded.
- Criterion 1, the panel's shape: MEDIUM. The recommendation is sound and the usability is a
  listening pass, which is why it is in "What no plan can close".
- The two inherited items: HIGH on mechanism, and item B's answer is a decision rather than a
  finding.
- Criterion 2: HIGH on the site census and on the API, MEDIUM on the genitive behaviour,
  which is documented and unrun.
- Criterion 3: HIGH on the counts, because each was produced by a script whose parts sum to
  its whole. MEDIUM on whether the rule table describes the shipping rule set.
- Criterion 4: LOW on the findings themselves, which cannot be read from anything. HIGH on
  the upstream to name.
- The phase 7 collision: HIGH. Derived from phase 7's own plan files.
- Cost facts: HIGH, all re-taken at `febe8e4` on 2026-09-12, except the sweep duration, which
  is deliberately not quoted.

**Research date:** 2026-09-12
**Read against:** `febe8e4`, version `0.112.0`, 720 guard records, `.planning/WINDOWS.md` at
entry 301.
**Valid until:** the earlier of phase 7's plans 07-04, 07-05 and 07-09 merging, which will
move every `config.rs` and `wx_settings.rs` line number here, or 14 days. The parts that do
not depend on those files, which are criterion 2 and criteria 3 and 4, hold longer.

*Nothing in the repository was written, edited or built during this research except this
file. Every in-repo claim above carries the command that produced it and the date it ran.*

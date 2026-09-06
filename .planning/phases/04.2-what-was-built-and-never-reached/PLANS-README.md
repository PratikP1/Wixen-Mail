# Phase 4.2, and where the other findings went

Written 2026-09-06. **Revised twice the same day.** The first revision answered
five decisions, which added a plan at the front, added a plan in the middle and
removed a task. The second answered two more: scheduled send is built rather than
documented as a limitation, and the fourth function in the blocking family is
wired rather than recorded a fourth time. The phase went six plans, then eight,
then nine.

All three versions were written read-only, with an executor holding the same
checkout and moving under every one of them. When the second revision began,
`04-09` was in flight holding `src/presentation/wx_app.rs`, `guards/guards.toml`,
`docs/changelog.md` and `Cargo.toml`. Every claim below about which plan holds
which file is dated for that reason and wants re-checking rather than trusting.
Nothing in the repository was written, no `cargo` was run, no `scripts/*.sh` was
run, and no `git` command that changes state was run.

Twenty-two findings came in: nine from `narrowed/logic-layers.md`, twelve from
`narrowed/ui-and-storage.md`, and one Pratik verified by hand. **Thirteen** are
planned here as phase 4.2, up from twelve and from ten before that. Nine go to
phases that already own the subject. **Nothing is left in "looked at and not
planned"** any more: the two entries that were there are both now plans or parts
of plans, by decisions 10, 12 and 13.

**Every finding planned here was re-verified against the tree as it stands**, not
read out of the reports. The second revision re-ran its own predecessor's claims
too, and that is written up below.

**What the second revision changed, in one place.** Scheduled send is the new
`04.2-02`, directly behind Undo Send, and plans 02 through 08 of the previous
version moved down one to 03 through 09. `04.2-01` no longer touches the
changelog sentence about a chosen time, because `04.2-02` makes it true and two
plans one apart editing one sentence in opposite directions is churn.
`what_blocking_will_do` is wired inside what is now `04.2-06` rather than recorded
as looked at and left. Requirement identifiers were renumbered again so they still
follow plan order: `CRIT-4.2-03` and `CRIT-4.2-04` are the new plan's and
everything from the old `CRIT-4.2-03` onwards moved up two. Threat identifiers
were **not** renumbered: they are unique and out of order, because renumbering
twenty-five of them would buy nothing. The two new ones are `T-4.2-32` to
`T-4.2-36` for scheduled send and `T-4.2-37` for the blocking sentence.

**One stale cross-reference was found and corrected while renumbering.** What is
now `04.2-09` read `test_f6_and_shift_f6_reach_the_pane_handler` as plan 04 left
it. `Shift+F6` was plan 04 in the six-plan draft and plan 06 in the eight-plan
one, and the first revision updated the reference two lines below it in the same
block and missed this one. It now names plan 07, which is where that work is.

---

## Phase 4.2: What was built and never reached

The name follows `02.1-what-phase-1-found-on-its-way-past`: a plain sentence
saying what the phase is, not a label. Directory
`.planning/phases/04.2-what-was-built-and-never-reached/`, files
`04.2-NN-PLAN.md`. Phase 4.1 is taken by mail moving between accounts.

Requirement identifiers follow phase 2.1's convention for an inserted
defect-closing phase: `CRIT-4.2-01` through `CRIT-4.2-14`, one per success
criterion, distributed across the plans so every one is claimed by exactly one.

### The plans, in order

| Plan | Wave | Requirements | Human | Tokens | What it does |
|---|---|---|---|---|---|
| 04.2-01 | 1 | CRIT-4.2-01, CRIT-4.2-02 | no | 88k | A message is really held, so Undo Send takes something back |
| 04.2-02 | 2 | CRIT-4.2-03, CRIT-4.2-04 | no | 68k | A message can be set to go at a chosen time, and a time that will not do is refused with a reason |
| 04.2-03 | 3 | CRIT-4.2-05, CRIT-4.2-06 | no | 74k | A meeting reply arrives declared as a reply, named `reply.ics` |
| 04.2-04 | 4 | CRIT-4.2-07, CRIT-4.2-08 | **yes, at the end** | 74k | Accepting a meeting puts it on the calendar, once, taking up its time |
| 04.2-05 | 5 | CRIT-4.2-09 | no | 52k | A reader is told how many pictures were held back, why, and where the switch is |
| 04.2-06 | 6 | CRIT-4.2-10 | **yes, at the end** | 92k | Blocked Senders is on the Tools menu, a block can be taken off there, and making one says so first |
| 04.2-07 | 7 | CRIT-4.2-11 | no | 62k | `Shift+F6` keeps its direction crossing the message preview |
| 04.2-08 | 8 | CRIT-4.2-12 | no | 78k | A column layout belongs to the kind of folder it was made in |
| 04.2-09 | 9 | CRIT-4.2-13, CRIT-4.2-14 | no | 70k | The documents name the keys that work, and five sentences stop overclaiming |

Nine plans and two human checkpoints, against the draft's six and one. Three of
the plans came from decisions rather than from my judgement, and all three were
taken against my recommendation. **See "Is this still one phase?" below**, because
nine is a real question and it has an answer rather than a shrug.

### Why this order

Pratik's instruction was to sequence by what somebody using the program would
care about. That still shapes the list, with one thing above it.

**Undo Send is first because it is the only brake, and the phase removes the
other one.** Decision 8 says answering a meeting sends with no confirmation step.
Once plans 03 and 04 land, one keypress sends mail to a real person and nothing
asks first. Undo Send is on the Tools menu, is on `Ctrl+Shift+Z`, is called from
a real handler, and refuses every single time it is pressed, because nothing in
production has ever held a message. So the brake goes in before the thing it is
a brake on. That ordering is Pratik's instruction and it is also the only
ordering that is safe.

It is worth saying what makes this different from every other item here. The
others are one feature each that does not work. This one is a feature that does
not work while every automatic check in the tree reads it as working, and the
mechanism is worth knowing because it recurs: a convenience wrapper pins the
interesting argument, so the general function has a caller, the argument has a
value, nothing is unused, and the other values are constructed only under
`#[cfg(test)]`. Three functions in this one feature drop the times that way.

**Scheduled send is second because it is the same machinery seen twice, and
because being second is what makes it small.** It is not second by user impact:
by that measure it would sit near the bottom, because the harm is a promise in a
changelog rather than a trap somebody meets. It is second because building it
anywhere else in the phase would mean rebuilding what plan 01 has just built.
Three things it needs and does not build are plan 01's, and a fourth was already
in the tree; all four are listed with their proving commands under "What the
second revision re-verified".

**The meeting reply is third because it is the one contradiction that is also a
false promise.** `docs/changelog.md` tells users the organiser "learns where you
stand instead of waiting". A standards-conforming client receiving
`text/calendar` with no method does not fold the answer into the meeting, which
is what the code's own doc comment says will happen. So the entry has been wrong
since it was written, and the person it misleads is somebody who thinks their
answer arrived.

**The calendar is fourth because it is the same feature's other half and the
same commit.** `git log -S` on `the_calendar_part`,
`what_the_calendar_should_hold` and `what_pressing_it_will_do` all return the
single commit `592ba56`. The whole second half of answering a meeting was
written and never connected. Somebody who accepts a meeting and opens the
calendar finds nothing, and nothing anywhere says that is how it works.

**The held-back pictures are fifth because the harm is per message and it lands
on exactly the person this program is for.** Someone reading a marketing message
with a screen reader meets `[Picture not shown: ...]` thirty times, is never told
there were thirty, never told why in one sentence, and never told where the
switch is. The sentence exists and is unreachable. This is the module the
project's own principles use as its example of a privacy decision worth
explaining.

**Blocked Senders is sixth, and it is the odd one out in this phase.** It is not
a false promise: the documented route works, because a block is an ordinary
filter rule and the rules manager deletes rules. It is a screen that was designed,
written, tested and never offered. It sits below the pictures because the harm is
rarer, and above `Shift+F6` because when it does land it is a trap rather than a
nuisance: mail stops arriving, nothing says why, and the rule doing it is one row
among however many rules somebody has. That sentence is `blocking.rs`'s own.

**`Shift+F6` is seventh because it is a key that does the wrong thing rather than
nothing**, in the one place a screen reader user is most likely to be stuck. It
is smaller than the items above only because the two panes make the direction
unobservable everywhere else.

**Columns are eighth because the worst of it needs a restart to meet**, but the
worst of it is bad: an inbox that comes back sorted by a date the sender chose,
which `message_columns.rs:54` says puts forged-date spam permanently on top.
Within one session the smaller version, losing a hand-chosen arrangement by
visiting Sent, is met more often and matters less.

**The documents are last because nothing they promise is destructive**, and
because plan 09 widens the check that would have caught all three keys, which is
easier to write once the other eight plans have stopped moving `wx_app.rs`. It
gained a second reason in the first revision and a third in this one: Blocked
Senders puts a new route into `docs/KEYBOARD_SHORTCUTS.md`, and scheduled send
puts Send Later into it, so the plan that widens the check that reads that
document runs after both plans that write to it.

### The dependency chain, and why it is a chain

Every plan touches `docs/changelog.md` and `Cargo.toml` under the same-commit
rules, and seven touch `src/presentation/wx_app.rs`. So the plans are ordered
rather than run in parallel, exactly as phase 4's are, and each `depends_on` the
one before.

Four real dependencies sit underneath the file contention rather than beside it.
`04.2-02` needs three things `04.2-01` builds and nothing else builds, which is
the strongest dependency in the phase and is written out with its proving commands
in `04.2-02`'s premise 1. Decision 12 named two of the three; the third is that
`queue_outbox_message` pins the argument, so until plan 01 deletes that wrapper
there is no call site a chosen time can be passed to. `04.2-04` builds on the answering work `04.2-03` does.
`04.2-03` waits for `04.2-01` because a brake goes in before the thing it brakes,
which is a consequence of decision 8 and is written into `04.2-03`'s premises
rather than left in this file. And `04.2-09` waits for `04.2-06` and `04.2-02`
because it widens the check that reads the document both write routes into.

**The whole phase waits for phase 4, and the plan it waits for keeps changing.**
`04.2-01` writes a setting into `src/data/config.rs` and
`src/presentation/wx_settings.rs`, and every other plan chains off it, so the
block sits on the front of the chain and gates everything. `04-08` landed and
merged at `50ed052` during the first revision, lifting the block on `config.rs`,
`wx_settings.rs`, `pictures.rs`, `html_renderer.rs` and `wx_compose.rs`. `04-09`
started immediately after, and during this second revision it held
`src/presentation/wx_app.rs`, `guards/guards.toml`, `docs/changelog.md` and
`Cargo.toml`. Every plan here touches the last three under the same-commit rules
and seven touch `wx_app.rs`, so the phase is still blocked and stays blocked until
phase 4 is done.

Read that as the shape rather than as the state: name the blocking plan by running
`git status --porcelain` and reading the newest phase 4 summary, not by trusting
`04-08` or `04-09` in this document. Work in flight has a half-life of hours and
this document has now been wrong about it twice, correctly both times, because it
said so.

**Measured again at the end of this revision, which is the demonstration.** When
it started, `HEAD` was `main` at `8d73579` with a clean tree. Two hours later
`HEAD` was `phase-04-09-encrypted-mail` at `080fcdb`, `docs(04-09): summary, and
the premise that stops task 1`, still clean. So in the time it took to fold two
decisions into these plans, `04-09` moved onto a branch and committed a summary.
Nothing in this file is worth trusting about that; the two commands are.

**A base commit worth correcting, and still worth repeating.** The original draft
says it was written against `main` at `5f57d3a`. That commit is
`feat(04-08): a picture is decorative because somebody said so`, which is in the
middle of `04-08` rather than a point where phase 4 was at rest. Naming a sha as
a base for a phase that cannot start until another phase finishes is misleading
whichever sha it is.

**Every `wx_app.rs` line number in these plans is stale.** That file is over
27,000 lines and phase 4 is rewriting it. Every plan that names a site in it says
to find it by symbol. Do not trust a `wx_app.rs` number in any of these documents.
The numbers this revision quotes for `wx_app.rs` were read on 2026-09-06 and are
given only to say what to grep for.

---

## What the second revision re-verified, with its own commands

Decision 12 asserted three things about the machinery scheduled send inherits, all
of them coming out of the previous pass rather than out of the tree. All three
were re-run. **All three hold**, on 2026-09-06.

**1. `GoAfter` already carries both `Held` and `Chosen`.** Holds.
`sending_later.rs:124` to `131` declares `AsSoonAsPossible`, `Held(String)` and
`Chosen(String)`. More usefully than the decision claimed, `GoAfter::read` at
`151` is already the function that tells the two apart from the two stored
columns, `GoAfter::written_down` is already what turns one back into them, and
`queue_outbox_message_to_go` at `outbox.rs:49` already calls it and names both
columns in its INSERT. So neither the reading nor the writing shape has to be
invented.

One correction to how the decision framed it: this is a fact about the tree today
rather than something `04.2-01` provides, and `04.2-02`'s premise 1 says so. What
`04.2-01` provides is the third item below plus one the decision did not name,
that `queue_outbox_message` pins the argument so there is currently no call site
a chosen time could be passed to. Four claims, one standing already and three of
them plan 01's.

**2. `outbox_rows` selects neither `send_after` nor `somebody_chose_it`, and both
features need both columns read.** Holds. `outbox.rs:189` selects `rowid,
to_addr, subject, body, created_at, attempt_count, last_error` and nothing else.
The second half of the claim is the load-bearing part and it is right: `Chosen` is
distinguishable from `Held` only by the flag column, so a build that read only
`send_after` would say a held message's countdown for a message set for next
Tuesday.

**3. Nothing runs `flush_outbox` on a clock.** Holds. `grep -rn "flush_outbox"
src/` returns five lines, two of which are prose in comments. The three real call
sites are all in `wx_app.rs` and all three are a person pressing something:
`ID_GO_BACK_ONLINE` around 4966, `ID_FLUSH_OUTBOX` around 4973, and the
composer's Send on the `WhenItGoes::Now` branch around 13618.

**And a fourth thing, which nobody claimed and which is the reason `04.2-02` is
the second smallest plan in the phase.** Almost all of what scheduled send needs
above the machinery already exists too.

- `Scheduling::spoken` at `sending_later.rs:485` already returns all four
  sentences, including the three refusals, and its doc already says why each names
  the problem and the next move. Decision 12 asked for "refusing one in the past
  with a reason rather than a silent clamp". The reason is written:
  "That time has gone. Pick a time still to come." The plan gives it a caller
  rather than writing words.
- `readiness` already answers `Chosen` with `WaitingUntil(at)`, and
  `Readiness::spoken` already turns that into the same wording the dialog uses,
  through one shared `set_to_send`. Plan 01 wires `Readiness::spoken` into
  `outbox_rows`; plan 02 gives it a row that takes the other arm.
- `what_send_did`'s `WhenItGoes::WhenItsTimeComes` arm already exists at
  `sending_later.rs:299` and `when_it_goes` already reaches it. Plan 02 adds no
  branch there.
- `JUST_MISSED`'s doc at `311` reads "**The picker** chooses a minute". The module
  was written expecting the control this plan builds.

**The picker itself is the find worth naming, because it changes what the plan
asks for.** The obvious control is `wxDatePickerCtrl`. `src/presentation/wx_item_form.rs`'s
module doc records a real screen reader session against exactly that control:
moving between month, day and year with Left and Right and changing one with Up
and Down said nothing at all, neither which part was landed on nor what its new
value was, because the control's internal notion of which part the arrow keys are
on is not exposed anywhere this application can reach. It is the control's own
limitation and no style flag fixes it. What that module built instead is a date as
three real controls and a time as two, each separately named, and `DateFields`,
`TimeFields` and `clamp_day_to_month` are all public.

So `04.2-02` copies a pattern a screen reader has already been run against rather
than specifying one that was already measured as silent. That is also why the plan
does not ask for a screen reader session of its own.

---

## The findings that went somewhere else

Three of these Pratik allocated. The other six are my judgement and each carries
its reason. **Nothing in this section changed in the second revision**; it is
reproduced because a reader of this file should not have to find the previous
version to know where a finding went.

### Phase 6, How the application speaks

**Braille and speech are one channel wearing two tick boxes.** Allocated by
Pratik and I agree, for three reasons. It is one of the four channels phase 6's
success criterion 1 is about. FEEDBACK-01 already owns `feedback.rs`. And fixing
the tick boxes without fixing the criterion would leave the roadmap making the
claim instead of the settings screen.

*What phase 6's plan must now also do:* decide whether the Speech and Braille
tick boxes are merged into one control saying what it does, or kept as two with a
sentence in the tab saying they move together on Windows, and rewrite the two
tests whose names promise the independence.
`accessibility.rs:472 test_the_words_still_go_out_when_speech_is_off_and_braille_is_on`
proves only that text reached the bridge, which its own comment admits, while its
name promises the deaf-blind case works.
`feedback.rs:1203 test_braille_survives_speech_being_switched_off` asserts the
stored setting still holds Braille and says nothing about output. Both pass today
and both describe a configuration that does not work.

**`Event::EdgeOfList` fires once and not for what it is documented as.** Mine.
`feedback.rs:38` documents it as the cursor trying to move past the first or last
row; production's only producer is the Next Unread handler signalling it with the
detail "no unread messages", for a search that came up empty. There is an event
for exactly that, `Event::NothingFound`. It goes to phase 6 because it is a
feedback event, FEEDBACK-01 owns the sixteen of them, and phase 6 criterion 1
requires setting all sixteen independently, which means somebody will be looking
at every one of them.

*What phase 6's plan must now also do:* change the one site to `NothingFound`,
then decide whether the edge tick is wanted at all, given that wiring it means one
call in each of six list selection handlers each of which has to know it was
already at the end. Note also that `wx_reader.rs:761` says "Last message" and
"First message" through `a11y.announce` directly, bypassing channel routing
entirely, so a user with speech off and earcons on hears nothing there.

**The internal automation tree, and `NativeBridgeStatus`.** Mine, both.
`automation.rs` holds eight fixed nodes written once at startup and never updated
or read; `FocusManager::current_focus` is read only by its own test;
`NativeBridgeStatus` reports `cfg!(target_os = "windows")` and has no caller at
all. These are about what this program knows about its own accessibility surface,
which is phase 6's subject and nobody else's. `NativeBridgeStatus` in particular
is guardrail 4 in miniature: a field shaped like a health check that reports a
build flag.

*What phase 6's plan must now also do:* for `automation.rs` and `focus.rs`,
either delete them with the `set_focus` announcement that rides on them, or write
two sentences at the top saying what the module is for, that it holds eight fixed
nodes, and that the real accessibility surface is wxWidgets plus
`set_accessible_name` plus `UiaRaiseNotificationEvent`. For
`NativeBridgeStatus`, derive it from `registered` and whether the last
notification call succeeded, and surface it where a person can see it. The
material for a real answer already exists in `screen_reader.rs:440`.

### Phase 5.1, Notes and contacts reach a server for the first time

**The CalDAV change marker is asked for, parsed, tested and discarded.**
Allocated by Pratik and I agree: it is server work against a live CalDAV server,
which is what 5.1 is, and this project says no sync path has met one.

*What 5.1's plan must now also do:* carry the parsed ctag into
`calendar_source::row_for`, which writes `ctag: None` along with `etag`,
`sync_token` and `refresh_interval_minutes`; use the `_ctag` parameter
`caldav.rs:284` already takes and ignores; return the new one instead of the
hardcoded `Ok((events, None))`; and store it. The consequence today is that every
CalDAV sync downloads the whole asked-for window every time. The cheap half, if
the full one does not fit, is to say in the changelog that the change marker is
not used yet, because right now the `_ctag` underscore is the only thing in the
tree that admits it.

*And rewrite the test rather than adding one.* `caldav.rs:7892` defends asking
for `getctag` with the comment "Without getctag the sync loses its change
marker", and `caldav.rs:8012` asserts the parse. Both pass, and three later
layers are written not to use the value, so together they read as a working
incremental sync. The rewrite is a test that a second sync of an unchanged
calendar does not re-download it.

**The `parse_ical_vevent` doc names three callers and there is one.** Mine, and
it travels with the ctag work because it is the same file and the same subject,
and because whoever is reading `caldav.rs` closely is the person who should fix
it. It is two sentences and has no user impact. If 5.1's plans are already full,
it is small enough to ride along with `04.2-09` task 3 instead, and that task is
written so it can.

### Phase 7, Installing, updating and what is stored

**Two per-account settings are stored, survive a restart, and are read by
nothing**, and `stored_setting_names` cannot see them. Mine. The settings half is
small. The durable half is the guard hole, and it is the reason this goes to
phase 7 rather than being a one-line deletion here: `config.rs:1666` begins with
`source.find("pub struct AppConfig {")`, so it reads one struct's fields, and
`AccountConfig` is a second struct in the same file, invisible to both directions
of the check without being nested inside anything. `CLAUDE.md` says a new
top-level setting fails on arrival and names nesting as the exception. This is a
third case the exception list does not describe, and phase 7 owns what is stored.

*What phase 7's plan must now also do:* either delete `data::config::AccountConfig`
with its two files-on-disk paths, or widen `stored_setting_names` to walk every
`Serialize`-deriving struct in `config.rs`. The second is the one that stops it
recurring. Note that nothing in production ever inserts into `account_configs`,
so the whole per-account config file format is a closed loop that starts empty,
and that real per-account settings live in the `accounts` table.

**An archive import counts what it could not read and never says it.** Mine.
`MailboxArchive::how_many_could_not_be_read`'s own doc assigns the duty: "Not
nought is somebody's mail still sitting in their archive, so it is the caller's
job to say so at the end rather than leave it in a log." Neither count has a
caller. This sits directly against the project's ninth principle and against the
reasoning already applied to the export path, whose
`what_the_folder_export_did` reports all four of its shortfalls with a comment
saying a backup that quietly leaves files out is one that looks complete and is
not. It goes to phase 7 because import and export of what is on somebody's disk
is that phase's subject.

*What phase 7's plan must now also do:* two fields on
`import_tree::FoldersImported`, filled from the archive after the walk, and two
sentences in `what_the_folder_import_did` written in the shape of the three
already there.

**Storage columns written and never read, and columns that can hold one value.**
Mine, mostly to phase 7 and one piece to phase 5. The write-only timestamps and
the always-NULL columns are additive-schema debris and the honest move is a note
in `docs/architecture.md` saying they exist for future use, which is a phase 7
sentence. **The `display_order` group is different and belongs to phase 5**:
`calendars.display_order`, `note_folders.display_order` and `tasks.display_order`
are each the leading `ORDER BY` term and each is always 0, while
`application/reordering.rs` is a working reordering gesture already used by
accounts and pinned folders and documented as `Alt+Shift+Up` and
`Alt+Shift+Down`. Extending it to calendars, note folders and task lists is the
smallest way to make the column mean something, and phase 5 is where the other
five modules are being brought level with mail.

### Phase 8, Every number the project quotes

**`built-and-left.md` cites two dead scaffolding modules as evidence.** Mine, and
then moved back. Phase 8 is about every number and claim the project quotes, so
it is the natural owner, but this is two table cells and the whole of it is
repointing them at `src/data/message_cache/accounts.rs` and the full-text tables.
Waiting for phase 8 to correct a citation that will mislead whoever re-verifies
that table in the meantime is a poor trade, so **it is in `04.2-09` task 3**.
Deleting the two scaffolding modules is separate, is a `dead-code-hunter` job
with its own reachability question, and is left.

### Mail: the column layout

**The column layout being one global string belongs with mail.** Allocated by
Pratik. The only live mail phases are 4, which is in flight and whose plans are
written, 4.1, which is cross-account movement, and this one. So it lands here, as
`04.2-08`, together with the Columns dialog always restoring the Inbox defaults,
which is the same family: `98546f8` fixed the folder-switch call site and left
the dialog, and `message_columns.rs:517` is the comment recording it.

---

## Looked at and not planned

**Nothing, as of the second revision.** Both entries that were here have become
work.

**A block can be made and there is no list of what is blocked** was a deferral
with my recommendation of a separate phase. Decision 10 overturned that: it is
`04.2-06` and it gained a Tools menu entry, which was Pratik's addition and is in
neither sweep report.

**`what_blocking_will_do`** was the one member of that family the plan did not
wire, recorded as looked at and left. Decision 13 overturned that too, and the
reason is in `04.2-06`'s premise 2 rather than here so that the plan carries its
own argument.

**Setting a message to go at a chosen time** was found while verifying Undo Send,
appears in neither report, and was recorded as a limitation with the false
changelog sentence to be moved into Known limitations. Decision 12 overturned
that: it is `04.2-02` and the sentence becomes true.

**The certificate withdrawal setting that does not exist.** `Reach::from_setting`
has no caller, no `AppConfig` field exists behind it, two of three arms of the
match at `signed_mail.rs:3221` are unreachable, and `docs/changelog.md` claims "a
check still running is said as unanswered", which cannot occur because no check
runs. Decision 11 settled it: the changelog correction stays in this phase, in
what is now `04.2-09`, and `04-09` is not reopened. Pratik's reason, which is the
better one and is worth keeping: editing a checked plan is how a checked premise
picks up a fresh error. So the false sentence is corrected here and the setting
question stays with phase 4. That is a deferral with an owner rather than a
finding nobody has.

---

## Two things the reports got wrong, and how

Both were found by re-verifying rather than by reading, and both would have
produced a change that was tested, plausible and broken. **Neither changed in the
second revision.** They are repeated here rather than left in a superseded file
because they are the reason the plans that carry them are shaped as they are.

**1. The meeting reply's stated fix cannot work.** `logic-layers.md` finding 1
gives the cost as "one extra field on `ComposeData` and one branch in
`mail_controller.rs:227`". A reply does not go from the composer to SMTP. It goes
into the outbox, whose `attachments` column is a newline-joined list of file
paths written by `attaching::joined` and read back by `attaching::split`. Bytes
cannot cross that column, and they should not: the row outlives a restart and the
file is read at the moment of sending on purpose. A `Ready` attached at the top is
dropped one layer below where the report looked.

The fix that does work derives the content type from the document, using
`invitations::what_it_asks`, which already reads `METHOD` and whose module
carries a paragraph on why there is one reading of that question rather than two.
That is the same argument `the_calendar_part`'s own doc makes, one layer down.

**2. The held-back pictures fix has a third caller the report did not see.** It
says `wrap_body` should keep the count and put the sentence above the body.
`wrap_body` also has a caller in the composer, previewing a message somebody is
writing, and `HtmlRenderer::new()` reads the settings where blocking is the
default. So the straightforward fix tells somebody writing a message that
"fetching it would have told the sender you opened this", where the sender is
them.

Both are written up as premise corrections in the plans that carry them.

---

## What rewriting tests costs here, measured, and where the pattern stops

Pratik's rule shapes most of this phase: **where a passing test covers a broken
feature, change that test so it measures what ships.** Ten tests are rewritten
across the nine plans, nine of them from the original draft and one added by the
Undo Send plan. The three plans that came from decisions add none: **there is
nothing to rewrite in any of them**, and that is a fact about them rather than a
choice they made.

`guards/guards.toml` held **627 records** on 2026-09-06, counted by splitting the
file on `[[` rather than by grepping a file name. That number has moved three
times while this phase was being planned: 624 at `5f57d3a`, 626 when the phase
was first planned, 627 a few hours later, because `04-08` and then `04-09` were
adding records throughout. **Re-count at the start of each plan rather than
quoting from here**, and every plan says so.

Records naming each file this phase writes tests in, re-measured at 627 and
unchanged from the eight-plan version except for the two files scheduled send
adds:

| File | Records |
|---|---|
| `src/application/answering.rs` | 0 |
| `src/application/blocking.rs` | 0 |
| `src/data/message_cache/outbox.rs` | 0 |
| `src/presentation/panes.rs` | 0 |
| `src/presentation/wx_columns.rs` | 0 |
| `src/presentation/wx_blocked_senders.rs` | 0, by construction |
| `src/presentation/wx_send_later.rs` | 0, by construction |
| `src/application/attaching.rs` | 1 |
| `src/application/sending_later.rs` | 1 |
| `src/presentation/editor_document.rs` | 1 |
| `src/presentation/message_columns.rs` | 1 |
| `src/presentation/html_renderer.rs` | 1 |
| `src/data/config.rs` | 2 |
| `src/presentation/wx_settings.rs` | 2 |
| `src/application/pictures.rs` | 4 |
| `src/presentation/wx_compose.rs` | 4 |
| `tests/wired.rs` | 8 |
| `tests/house_style.rs` | 18 |
| `src/application/mail_controller.rs` | 24 |
| `src/presentation/wx_app.rs` | 40 |

Every plan sites its tests in the cheap files and **none adds a test to
`wx_app.rs` or `mail_controller.rs`**, which is a fortyfold difference in what
the commit-time count check costs. Every plan carries that as an acceptance
criterion with the count reported before and after, because "we meant not to" and
"we checked" are different claims.

**Where the silence matters, and where it stops mattering.** Rewriting a test
rather than adding one leaves the file's count exactly where it was, so
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` cannot
fire. That is not a pass. It is silence by construction, and every plan built on
rewrites has an acceptance criterion requiring the summary to say so in those
words rather than reporting the check as clean.

**The three plans that came from decisions are the exception, and they are the
ones where the remedy is really on the critical path.** Undo Send, scheduled send
and Blocked Senders are almost entirely new wiring. So they add tests, the counts
move, the check fires, and the scoped remedy has to be run. Which records each can
make stale was measured:

- **Undo Send** writes tests in `outbox.rs` (0 records), `sending_later.rs` (1),
  `config.rs` (2) and `wx_settings.rs` (2). The three records that can be named
  are `a moment written with a T is one the reader knows`, which names
  `sending_later.rs`; `a stored setting that no screen offers is caught`, which
  names both `config.rs` and `wx_settings.rs`; and `a sentence carrying the
  source indentation is found`, which names `wx_settings.rs`. At most three
  records to re-measure, which is minutes rather than hours.
- **Scheduled send** writes tests in a new module (0 by construction),
  `sending_later.rs` (1) and `editor_document.rs` (1). At most two records, and
  one of them is the same `sending_later.rs` record Undo Send already re-measured
  a plan earlier, which is worth knowing: it wants measuring again rather than
  taken as done.
- **Blocked Senders** writes tests in `blocking.rs` (0 records) and a new module
  (0 by construction), so on the measurement above it can name nothing. That is
  a claim about today's records and the plan re-checks rather than trusting it.

Run `scripts/guards.sh --remeasure` with the names the commit prints, in the
background, and report what came back. Do not lower the check and do not edit a
record until it applies.

**The rename check will not fire, and that was measured rather than assumed.**
`test_every_test_a_guard_record_names_is_a_test_that_exists` fires only where a
record names the renamed test. `grep -c` against `guards.toml` for each of the
tests due for rewriting answers **0** for every one, on 2026-09-06. That number
will change as earlier plans add records, so each plan re-checks rather than
trusting this table.

**What no check in this phase can see** is a rewritten test that starts or stops
reddening under an existing record's break, which makes that record name too
many or too few. A member that leaves is the loss: a guard quietly claiming cover
it no longer has. Only the whole sweep sees it, and by the decision of
2026-09-03 no sweep runs per merge or per phase. Phase 8 criterion 5 owns it,
and this phase adds ten rewrites to what that sweep will be judging.

---

## Is this still one phase?

It has grown from six plans to nine and the question is fair. **My answer is that
it is one phase, and the reason is a shape rather than a preference.** But there
is a cut, it is a clean one, and it is described below so it can be taken without
re-deriving it.

**Why it holds together.** Every plan here closes the same defect: something
written, tested and reached by nothing, usually with a document already saying it
works. That is not a theme somebody imposed afterwards, it is what both sweeps
found and it is what the phase is named for. Nine instances of one defect is a
phase; splitting it makes two phases with the same name.

The mechanical arguments point the same way. The whole phase is one serial chain
because every plan touches `docs/changelog.md` and `Cargo.toml` under the
same-commit rules and seven touch `wx_app.rs`, so splitting buys no parallelism.
And the entire chain is blocked on phase 4 finishing, so both halves would wait
for the same thing and then run one after the other regardless of which phase
number they carry.

**What genuinely grew, and it is smaller than "six to nine" suggests.** Three of
the four new units are the same feature family. `04.2-01` and `04.2-02` are one
feature, sending later, seen twice; `04.2-02` is the second smallest plan in the
phase precisely because `04.2-01` builds nearly all of it. And decision 13 added
no plan at all, only a task's second half. So the growth is: one plan for Undo
Send, one for Blocked Senders, one small one for scheduled send, and one task
half. The first two were needed before this phase could safely ship what it
already contained.

**Where I would cut it, if you want it cut.** Between `04.2-06` and `04.2-07`.

- **Phase 4.2, plans 01 to 06.** Everything that sends mail or takes it back, and
  everything with an accessibility surface of its own. Both checkpoints are here,
  both new screens are here, and every item that can lose somebody's mail or send
  something they did not mean to is here. Roughly 448k of the phase's 658k.
- **Phase 4.3, plans 07 to 09.** `Shift+F6`, the column layout and the documents.
  Three plans, no checkpoint, nothing destructive, and the only cross-boundary
  dependency is that `04.2-09` widens a check reading a document `04.2-06` and
  `04.2-02` write into, which is satisfied by 4.3 running after 4.2 rather than by
  anything finer.

That cut is clean because the second half is exactly the tail of the
user-impact ordering: the three items where nothing is destructive and nothing
needs a person. It would also let the first half's checkpoints be run in one
sitting rather than spread across nine plans.

**What the cut costs.** The two halves would still run one after the other, so it
buys no time. It would split one set of success criteria across two roadmap
entries, and the criteria are numbered in plan order, so they would be renumbered
a third time. And the phase's name describes the second half as accurately as the
first, so 4.3 would want a name that is not a worse version of the same sentence.

**So: my recommendation is to keep it as one phase**, and to treat the size as a
reason to re-run the calibration on the estimates rather than as a reason to
split. It is your call and the cut above is ready if you want it.

---

## The four questions, and how they were answered

All four were answered on 2026-09-06, and two of them were then answered again in
the second round. Recorded here so the plans do not have to carry the argument,
and so a later reader finds an answer rather than the question again.

**1. Does answering a meeting get a confirmation step? No.** Decision 8, against
my recommendation of a confirmation before sending, and taken knowingly.
`04.2-04` loses its task 3 entirely: no confirmation, no softened version, and no
re-argument in the plan. `what_pressing_it_will_do` stays unwired and the plan
records it as looked at and left with that reason.

The consequence is the whole reason the phase grew a plan at the front. Once this
lands, answering a meeting sends mail to a real person from one keypress and
nothing asks first, so Undo Send stops being a nicety and becomes the only brake.
Verified against the tree the same day: it does not work, and `04.2-01` is what
makes it work.

**2. Does the message preview become a full stop on the `F6` cycle? No.**
Decision 9, against my recommendation of considering it separately, which is the
same outcome by a shorter route. Pratik's reasoning is now inside `04.2-07`
rather than only here: with snippets and the other ways this program offers of
reading mail, the cycle does not need to visit the preview, so a stop that costs
a keypress every trip round buys nothing. The work is the two document
corrections and keeping `Shift+F6`'s direction when leaving the preview.

**3. Does the Blocked Senders screen belong in this phase? Yes.** Decision 10,
against my recommendation of a separate phase alongside 4.1. It is `04.2-06` and
it gained a Tools menu entry, which is Pratik's addition and appears in neither
sweep report. It is a plan rather than a task because it is a screen with its own
accessibility surface: a list read by ear, a destructive action, an
announcement, keyboard operation, a menu entry somebody can find, and two Windows
accessibility channels that have to be right independently.

**4. Does the certificate withdrawal finding go into `04-09`? No.** Decision 11,
which is the option I recommended. The changelog correction stays in this phase,
in what is now `04.2-09`, and `04-09` is not reopened. Pratik's reason is better
than mine was: editing a checked plan is how a checked premise picks up a fresh
error.

## The two questions the first revision raised, and how they were answered

Both were mine, both were found while verifying Undo Send and Blocked Senders,
and neither was in either sweep report. Both were answered against my
recommendation.

**5. What happens to `what_blocking_will_do`? It is wired.** Decision 13. I
recommended not leaving it a fourth time and said I was not confident between
wiring and deleting. Pratik chose wiring, and the argument that settles it is one
I had already written down without following it to its conclusion: `may_block`
returns `YesButFirst(warning)` and `wx_app.rs:25446` says it, under a comment
reading "Said before the block is made, not instead of making it". There is
already a moment before a block in which this program speaks. Wiring the fourth
function follows the precedent the feature set for itself rather than importing
the answer from sending, which is what makes it consistent with decision 8 rather
than in tension with it.

It is not a new plan. It is the second half of `04.2-06`'s task 2, because it is a
change to the making path rather than to the window, and the making path is what
that task's `read_first` already sends the executor to read.

**6. Is setting a message to go at a chosen time a feature, or a limitation? A
feature.** Decision 12, against my recommendation of the honest limitation. My
argument was that everything below it already works, so it will be cheap whenever
it is built and nothing about waiting makes it more expensive. That argument was
right about the cost and wrong about what to do with it: the same cheapness is a
reason to build it now, and the false changelog sentence has to be dealt with
either way.

It is `04.2-02`, directly behind Undo Send, and it is the second smallest plan in
the phase. What it inherits and what it owes are set out above under "What the
second revision re-verified", with the commands that prove each, and `04.2-02`'s
task 1 requires all three to be re-run before a line is written.

### One judgement worth flagging, which is not a question

`04.2-01` and `04.2-02` do not ask for their own screen reader sessions, and
between them they raise three questions only a screen reader settles: whether the
sentence said when Send is pressed finishes in time for somebody to hear it and
still act inside the hold, whether five spinners for a date and a time are heard
as five named controls, and whether a refusal is heard when a dialog stays open.

The first is folded into `04.2-04`'s checkpoint as item 4. The other two go into
`.planning/WINDOWS.md` as `unrun-verify`, on the grounds that the control shape
has already had a real session run against it in `wx_item_form.rs` and this is a
second use of a pattern rather than a first use of a control. That is a trade,
it is three plans of delay in the worst case, and you may want it earlier.

---

## What this phase cannot close, whatever it does

Recorded here so no plan reads as though it settled these.

**Nothing here has met a real server, a real organiser or a real provider.** The
reply's declaration, the calendar row, the held message and the message set for
next Tuesday are all asserted against pure code and a temporary database. Whether
Outlook or Google Calendar folds the answer in, whether the account's provider
takes a pushed event, and whether the program is ever left running long enough for
a chosen time to arrive, are `unrun-verify` and belong in `.planning/WINDOWS.md`,
whose last entry is 134.

**There are two screen reader sessions, in plans 04 and 06, and there were nearly
four.** Plans 01, 02, 05, 07 and 08 each name at least one thing only a screen
reader settles and record it rather than asking for a session of their own. The
meeting flow gets one because it has the most spoken steps, and Undo Send's own
question, whether ten seconds is long enough to hear the sentence and still act,
is folded into it as item 4.

Blocked Senders gets its own and could not borrow either. It is a new window with
a list read row by row, a destructive action, an announcement and two Windows
accessibility channels that have to be right independently, and none of that is
exercised anywhere else in the phase. Its checkpoint gained a fifth item in the
second revision, for the sentence said before a block is made. This project has
already shipped sixteen widgets named through a call that never reaches the
accessibility tree, compiling and passing 324 tests, so a new screen going out on
structure alone is the failure with a precedent.

**Ten rewritten tests go into phase 8's sweep as unexamined weight.** Each is a
test that may have changed which guard records it reddens, and no check in this
phase can see that. That is the accepted cost of the 2026-09-03 decision to run
one sweep per milestone, and it is worth saying that this phase adds to what
that sweep will find rather than letting the number arrive as a surprise. The
three plans that came from decisions add to it in the other direction as well:
they add tests rather than rewriting them, so the count check does fire for them
and the scoped remedy runs during the phase rather than waiting for the sweep.

---

## Estimates, and what they are worth

Each plan carries an `estimate` block with `confidence: low`. **The calibration
tool was not run**, because it writes into `.planning/` and every session that
produced these was read-only. The token figures are by comparison with phase 4's
own plans, which carry recorded estimates for work of similar shape: `04-05` at
20,000 for two tasks, `04-09` at 48,000 for four. `raw_tokens` equals `tokens` in
every block here rather than being scaled, because there is no measured factor to
apply and inventing one would make the number look calibrated when it is not.

The phase totals **658,000** across nine plans, which is the number to weigh the
split question against rather than the plan count.

The two plans added in the first revision were sized the same way and were the
two largest in the phase: `04.2-01` at 88,000 for three tasks, because it touches
five source files and a settings round trip, and `04.2-06` at 84,000 for three
tasks and a checkpoint, because it builds a window from nothing. `04.2-04` came
**down** from 96,000 to 74,000 when its task 3 was removed.

The second revision moved two figures. `04.2-06` went from 84,000 to **92,000**,
because task 2 gained the fourth function's wiring, a hoist on the making path and
three more tests. And `04.2-02` was sized at **68,000** for three tasks, which is
the second smallest in the phase: it builds a dialog, a `Reached` variant and one
branch in `queue_for_sending`, and everything below that it inherits from
`04.2-01` or finds already written in `sending_later.rs`.

Whoever moves these into the repository should re-run the calibration and
replace every block rather than trusting these figures. Given the phase is now
nine plans, that matters more than it did at six.

---

## The ROADMAP entry, ready to paste

Nothing was written to the repository, so this is here rather than in
`.planning/ROADMAP.md`. It follows phase 2.1's shape and goes after phase 4.1.
Use `gsd roadmap` for the structural insert rather than editing the file whole.

### Phase 4.2: What was built and never reached (INSERTED)

**Goal**: Wire up the capabilities two sweeps found written, tested and reached
by nothing, and correct the documents that describe them as working.
**Depends on**: Phase 4, all of it. Plan 01 writes a setting into
`src/data/config.rs` and `src/presentation/wx_settings.rs` and every other plan
chains off it, so the block is on the front of the chain. `04-08` held those two
files and has landed; `04-09` holds `src/presentation/wx_app.rs`,
`guards/guards.toml`, `docs/changelog.md` and `Cargo.toml`, which this phase
touches in every plan. Nothing here starts while phase 4 is executing.
**Requirements**: none new; this closes recorded defects rather than adding
capability, with two exceptions noted under criteria 3 and 10
**Success Criteria** (what must be TRUE):

  1. A message sent from the composer is held before anything hands it to a server, goes on its own when the hold runs out, says while it waits that it is waiting, and can be taken back with Undo Send. Today `Ctrl+Shift+Z` is on the Tools menu, the handler is real, and it refuses every time it is pressed, because `queue_outbox_message` pins the hold to nothing at `outbox.rs:38` and the composer computes a value it never passes to the queue call two lines below. Ledger 81.
  2. How long the hold lasts is a setting on the Compose tab under Sending, described with what turning it off costs. `docs/changelog.md` has said "Ten seconds by default, adjustable" while `AppConfig` held no such field, so both of the settings checks that would have caught it have never had anything to catch.
  3. A message can be set from the composer to go at a date and time somebody chooses, it waits in the Outbox saying the time it is set for, and it goes on its own when that time comes. `sending_later::schedule` has no production caller, so `docs/changelog.md`'s promise that a message can be set to go at a chosen time has described nothing since it was written. This is one of two criteria here that add capability rather than closing a false claim, and it is the one where the false claim is the reason: the sentence has to be dealt with either way, and everything under it is already built and tested.
  4. A time that has gone by is refused with the reason and the next move, not sent and not quietly moved to now; so is a time more than a year ahead, and so is text that is not a date and time. A time inside the minute of grace `JUST_MISSED` allows is accepted, which is deliberate. All four sentences are already written in `Scheduling::spoken` and none has ever been said.
  5. A meeting reply arrives at the organiser declared `text/calendar; charset=utf-8; method=REPLY`, so their client records it against the meeting instead of showing a file to open by hand. The content type has existed since `592ba56` and nothing on the sending path has ever asked for it, while `docs/changelog.md` has told users the organiser learns where they stand.
  6. The part the organiser receives is called `reply.ics`, which is the name this program chose and lost by writing `reply-<uuid>.ics` and taking the part's name from the file.
  7. Accepting a meeting puts it on the calendar, on the day, at the time it is at, taking up that time according to the answer given. `OnTheCalendar` is produced and consumed inside one module; `BlocksTime::as_stored` produces the three words the calendar column already takes and nothing has asked it.
  8. Answering the same meeting twice leaves one entry, and a changed meeting replaces the one already there. `save_calendar_event` conflicts on `id` and the invitation's `UID` lives in `provider_event_id`, so a second answer that invents an identifier writes a second row.
  9. Somebody reading a message with pictures held back is told how many, why, and where the switch is, once, in the document. Both callers of `sanitize_and_count_held_back` discard the count and `pictures::what_was_held_back` has no production caller, so a screen reader user meets the inline marker thirty times with no orientation.
  10. Somebody can find out who they have blocked, from the Tools menu, and take a block off there, and unblocking one address never removes a wider block that happens to catch it; and somebody making a block is told what blocking will do before the rule is written, beside the warning `may_block` already gives rather than instead of it. All four of `everyone_blocked`, `the_rule_that_blocks`, `what_unblocking_did` and `what_blocking_will_do` are written and tested with no production caller. This is the second criterion here that adds capability rather than closing a false claim: the documented route, deleting the rule in the rules list, genuinely works, and the changelog is corrected to say it was the only route rather than the intended one. Nothing here becomes a confirmation step: the sentence is said and the block is made in the same pass, which is what `may_block`'s own comment already describes.
  11. `Shift+F6` leaves the message preview in the opposite direction to `F6`, and nothing in the tree says the preview never takes focus. The key is bound and correct and loses its direction crossing the WebView boundary, and `focus_home`'s own doc says a WebView takes focus without asking. The preview does not become a stop on the cycle, by the decision of 2026-09-06.
  12. A column layout made in Sent is stored as a Sent layout and does not become the inbox's, and Restore Defaults in the Columns dialog restores the defaults for the folder somebody is in. Today sorting in Sent makes the inbox come back sorted by a date the sender chose, which `message_columns.rs:54` says puts forged-date spam permanently on top.
  13. Every key the shortcuts document promises is a key that arrives, and every key that is bound is documented. `Ctrl+\` is documented as the way to the composer toolbar and was measured not arriving; `F8` works and is written nowhere; `Delete` and `F6` are bound and undocumented. The check that holds the page and the bindings together cannot see an unmodified function key in either direction, and this widens it rather than adding three lines to the page.
  14. Five sentences that describe something other than what ships are corrected: three changelog entries telling a deaf-blind user they can switch speech off and keep braille, a doc comment naming three callers of `parse_ical_vevent` where there is one, and two evidence cells in `.planning/intel/built-and-left.md` pointing at modules production never instantiates. Plus two code corrections: the settings screen telling somebody Windows overruled a setting they never turned on, and the scrolling module claiming two readers where there is one.

**Plans**: 9 plans, one per wave. `docs/changelog.md` and `Cargo.toml` are touched
by every plan under the same-commit rules and `src/presentation/wx_app.rs` by
seven, so the plans are ordered rather than run in parallel. Plans 04 and 06 each
carry a blocking screen reader checkpoint and neither is autonomous.

- [ ] `04.2-01-PLAN.md` — A message is really held, so Undo Send takes something back
- [ ] `04.2-02-PLAN.md` — A message can be set to go at a chosen time, and a time that will not do is refused with a reason
- [ ] `04.2-03-PLAN.md` — A meeting reply arrives declared as a reply, named `reply.ics`
- [ ] `04.2-04-PLAN.md` — Accepting a meeting puts it on the calendar, once, taking up its time
- [ ] `04.2-05-PLAN.md` — A reader is told how many pictures were held back, why, and where the switch is
- [ ] `04.2-06-PLAN.md` — Blocked Senders is on the Tools menu, a block can be taken off there, and making one says so first
- [ ] `04.2-07-PLAN.md` — `Shift+F6` keeps its direction crossing the message preview
- [ ] `04.2-08-PLAN.md` — A column layout belongs to the kind of folder it was made in
- [ ] `04.2-09-PLAN.md` — The documents name the keys that work, and five sentences stop overclaiming

**Undo Send is first because the phase removes the other brake.** By the decision
of 2026-09-06, answering a meeting sends with no confirmation step, so once plans
03 and 04 land, one keypress sends mail to a real person and nothing asks first.
Undo Send is then the only way back, and it does not work. Setting a message to go
at a chosen time follows it immediately, because the two are the same machinery
seen twice and the second is mostly a dialog once the first has landed.

**Most plans rewrite tests rather than adding them beside the ones that pass.**
Ten in all. Where a passing test covers a broken feature, the test is the place
that says so, and a second test beside it would close the bug and leave the trap
armed. Both of this project's guard-record checks are silent against a rewrite by
construction, so each of those plans' summaries is required to say that rather
than report them as clean. The three plans that came from decisions are the
exception: they are new wiring, so their counts move, the check fires, and the
scoped remedy runs inside the phase.

**Nothing here has met a real server.** The reply's declaration, the calendar row,
the held message and the message set for next week are asserted against pure code
and a temporary database. Whether an organiser's client folds the answer in,
whether a provider takes a pushed event, whether ten seconds feels long enough
with a real mailbox syncing underneath, and whether the program is left running
long enough for a chosen time to arrive, are `unrun-verify`.

**UI hint**: yes

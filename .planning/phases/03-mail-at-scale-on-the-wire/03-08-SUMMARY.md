---
phase: 03-mail-at-scale-on-the-wire
plan: 08
executed: 2026-09-05
status: complete
tasks: 3
requirements: [SCALE-05]
subsystem: application, presentation, service
tags: [offline-mode, network-detection, outbox, guardrail-7, announcement-bounding, scale-05]
commits:
  - ee432d6 test(03-08) failing tests for a switch that decides nothing
  - 7d3f9f0 Offline mode queues outgoing mail, which is what it has been saying it does
  - d6da4a1 test(03-08) failing tests for a network loss said once rather than per failure
  - b92addc A network that goes puts the program offline, and says so once
  - 51ba4d0 test(03-08) failing tests for an offer that does not exist yet
  - 4cd8e65 The network coming back is an offer somebody takes, not a send
merged: not merged, and not pushed
key-files:
  created:
    - src/service/network.rs
    - src/application/the_network_coming_and_going.rs
    - tests/nothing_leaves_the_outbox_unasked.rs
  modified:
    - src/application/sending_later.rs
    - src/application/mod.rs
    - src/service/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/ui_types.rs
    - guards/guards.toml
    - docs/changelog.md
    - Cargo.toml
    - .planning/WINDOWS.md
requires:
  - "the outbox, which was already complete: queueing, scheduling, cancelling and failure recording"
provides:
  - "when_it_goes: whether a message goes now, from the offline state and the message's own schedule"
  - "service::network: whether this machine has a network, Windows only behind a flat wininet call"
  - "the_network_coming_and_going: what the program believes about the network, and what a change calls for"
  - "a census of every place that hands mail to a server, each named beside what asked for it"
  - "an offer raised when the network returns, which sends nothing until somebody presses it"
affects:
  - "anything that adds a caller to flush_outbox: the census counts them and asks what asked"
  - "03-09, which carries the same guardrail 7 trap for waiting flag changes; the shape settled here is below"
decisions:
  - "The decision lives beside readiness in application::sending_later, not in the outbox module the plan named. The plan's path was wrong twice over: outbox.rs is in the data layer under message_cache, and a pure decision does not belong there. sending_later is the module whose own doc says values in and values out, no database and no window."
  - "Offline is answered before the message's own time. A message waiting on both is waiting on one thing somebody can act on and one they cannot, and the one worth saying is the first."
  - "The decision reads the schedule and never writes it, so turning offline on and off leaves a scheduled message still scheduled."
  - "queue_for_sending hands back what it wrote on the row, so the decision reads the row's own schedule rather than a constant repeated at the call site."
  - "A message that did not go is said through CommandAnswered, at high priority on its own topic, not through the status line. Somebody who pressed Send and heard a sync line believes their mail has gone."
  - "The network is asked on the window's own poll, every ten seconds, not at the end of a mail check. Nothing here checks mail on a schedule, and every trigger that does stops when the network goes, so a check there could see the loss and never the return."
  - "Windows detection is a flat wininet link block, not a windows crate feature. Cargo.toml's own comment says the windows crate is for the platform spell checker and everything else stays a link block, and twenty link blocks in the tree agree with it."
  - "The non-Windows fallback answers that there is a network. It is the answer that leaves behaviour where it was; the other one strands somebody offline with nothing able to bring them back."
  - "Announcing is bound to the change, not to the answer or to a failure. Asking again while nothing has moved is silent, and a program that started with no network has lost nothing."
  - "The network returning raises an offer and sends nothing. The offer calls go_into_offline_mode and flush_outbox, which are the menu item's own body and the menu item's own function, so it cannot come to mean something different from the menu."
  - "The offer's label says that pressing it sends. Guardrail 7 is not satisfied by somebody pressing something, it is satisfied by them knowing what the act is."
metrics:
  duration: about 6 hours
  files: 12
  commits: 6
actuals:
  tokens: 41000
  tasks: 3
  commits: 6
---

# Plan 03-08: A switch that decides something, and an offer instead of a send

**One-liner:** Offline mode now queues outgoing mail rather than only saying it
does, losing the network puts the program offline and says so once whatever
number of requests failed, and the network coming back raises a button rather
than emptying the Outbox.

## What works

### The live defect is fixed

Pressing Send with offline mode on now leaves the message in the Outbox and
says so. Nothing is handed to a server.

`offline_mode` had four occurrences in `wx_app.rs` and none of them decided
anything: declared at 315, initialised at 430, toggled at 4854 to 4871, mirrored
at 15247. It has nine now, and three of them decide. The one that matters is on
the send path:

```rust
let goes = crate::application::sending_later::when_it_goes(
    reachability_of(state),
    &waiting_on,
    chrono::Local::now(),
);
```

`when_it_goes` is in `src/application/sending_later.rs` beside `readiness`,
takes no window and no database, and has eight tests of its own. Offline is
answered before the message's own time, and it reads the schedule without
writing it, so a message set for Friday queued while offline is still set for
Friday when somebody goes back online rather than going out at once.

**The three endings of Send, side by side, which the plan asks for:**

| when | what is said |
| --- | --- |
| it goes | `Sending to kim@example.com...` |
| offline mode is on | `Offline mode is on, so the message to kim@example.com is waiting in the Outbox. It goes when you go back online.` |
| its own time has not come | `The message to kim@example.com is waiting in the Outbox until the time set on it.` |

The first is unchanged, because every message sent today takes that path. The
third is unreachable in the shipped build and the reason is a defect found on
the way, recorded below.

For comparison, the two sentences that already existed about a waiting message,
neither of which is on the send path: the Outbox row says
`{subject}, waiting to send` through `waiting_label`, and Undo Send refuses
through `WhatToTakeBack::why_not`.

### Losing the network is noticed and said once

`service::network::whether_there_is_a_network` asks Windows through
`InternetGetConnectedState`, a flat `#[link(name = "wininet")]` block.
Everywhere else it answers that there is one, with the reason in a comment: that
is the answer that leaves behaviour exactly as it was, and the other one puts
somebody into a mode nothing can bring them out of.

**No new package.** The plan asked for a feature on the `windows` crate. That
crate's own comment in `Cargo.toml` says it is only for the platform spell
checker and that everything else Windows stays a flat link block, and twenty
link blocks across the tree agree. `Cargo.toml` is touched by this plan only by
the version bumps.

`application::the_network_coming_and_going` holds what the program believes and
answers what a change calls for. Ten looks at one unplugged cable produce one
announcement, proved by driving ten and counting. A program opened where there
has never been a signal is told nothing, because nothing changed. Asking fifty
times while the network is still there is silent.

The sentence and the mode go out in one spawned task in that order, because two
spawned sends race and the race this one would lose is the indicator changing
with nothing having said why.

### The two channels carry one string

`what_to_say_about_the_network` builds the sentence once. The
`UIUpdate::TheNetworkChanged` arm hands that one string to `set_status_text(.., 0)`
and to `announce_topic(.., Priority::Normal, "network")`. A deaf user reading
the status bar and a blind user hearing it get the same words.

Its own topic rather than `"status"`, for the reason `FolderWasRenumbered` gives:
the steady sync lines share that topic, the queue keeps only the newest of a
topic, and a network loss arrives while a sync is failing its way through
several folders.

The two sentences, quoted:

> The network has gone, so Wixen Mail is now offline. Mail you send waits in the
> Outbox until you go back online.

> The network is back. Wixen Mail is still offline, and nothing in the Outbox
> has been sent. There is a Go Back Online button above the message list, or you
> can turn offline mode off from the View menu.

### The offer, and the shape 03-09 should copy

**This is the answer to the trap.** The offer is a button above the message
list, in the tab order, whose visible label and accessible name are one binding:

> Go back online and send the mail waiting in the Outbox

Pressing it calls `go_into_offline_mode(app, false)` and `flush_outbox(app)`.
Those are the View menu item's own body, lifted out so both callers share it,
and the Outbox menu item's own function. Neither is reimplemented.

**The shape, stated for 03-09.** Four parts, and each one is doing work:

1. **The decision layer answers with an offer, not an act.** `WhatToDoAboutIt`
   has three members and none of them means "send". A module that cannot express
   the dangerous act cannot be wired to it by accident.
2. **The offer is a control somebody presses**, not a state somebody has to
   notice. It is in the tab order and it stays until the mode changes.
3. **Its label says what pressing it does.** A button called "Go back online"
   that also emptied the Outbox would be guardrail 7 with an extra step rather
   than guardrail 7 satisfied. What makes an act deliberate is knowing what the
   act is.
4. **A census counts every place that publishes**, names each beside what asked
   for it, and fails when the number moves. `tests/nothing_leaves_the_outbox_unasked.rs`
   reads three and names them: the menu item, the composer's Send under the
   decision, and this button.

The fourth is the part worth copying most, because it is the one that keeps
working after everybody has forgotten the argument.

## Verification

Every commit went through `scripts/check.sh` by way of the commit hook. Nothing
used `--no-verify`. Three commits changed `Cargo.toml`, so `which-checks.sh`
answered `all` for each and every one ran the whole suite and the release build;
all three were run detached, because that gate outruns a ten-minute foreground
cap.

`scripts/check.sh all` passed on the finished branch: the library is **6,195
passed, 0 failed, 1 ignored**, every integration target green, and the release
build clean.

**Three reds, all accepted by `scripts/red-commit.sh`.**

| commit | named | what really failed |
| --- | --- | --- |
| ee432d6 | 7 | 5 of the 8 offline decision tests, the census reading the Send arm, the count check |
| d6da4a1 | 4 | the counting, the already-gone start, the repeat, the two round trips |
| 51ba4d0 | 4 | the guardrail 7 assertion, the button naming, the count of flush sites, the count check |

The first two reds are a compiling stub that reproduces the shipped defect
rather than a missing symbol, so every failure is an assertion rather than a
build error. Task 1's stub answers from the schedule and ignores the switch,
which is what the build does today. Task 2's answers from the observation rather
than from the change.

**Which tests passed at their own red, said plainly.** Six.

Three of task 1's eight: the online cases, which the stub already answers
correctly. They are the half that must not change and they were written to say
so.

`service::network`'s two tests pass on arrival. A new pure module has no before,
and the polarity of a `BOOL` is worth a test even though nothing was ever wrong
with it: read the wrong way round it puts every Windows user offline
permanently.

Task 3's two library tests pass on arrival, and could not have been red in any
ordering: the button's words and the reworded sentence were written in the same
commit as the tests. They are what would notice a reword that drops "send" from
a button that sends, or that leaves somebody who never finds the button with no
way back.

**Four guard records, three of them new, every one measured by hand against
every target with `--no-fail-fast`.**

| record | break | what really went red |
| --- | --- | --- |
| the send path reads what the window believes (new) | the decision is asked with a constant | 1 |
| the network is spoken about when it changes (new) | answer from the observation | 4 |
| the network coming back offers rather than sending (new) | the arm that raises the offer flushes too | 2 |
| the offline mode change is spoken by the sentence ahead of it (re-pointed) | the sentence is dropped | 1 |

All four were then re-measured by `scripts/guards.sh --remeasure` on a clean
tree, and every one reddens exactly the tests it names.

**The first record's single test is the finding rather than an aside.** Reaching
the Send arm needs a window, a frame and a composer, so nothing in the library
reads it, and the whole defence of the switch being read is one source-reading
test. The same shape 03-02 found for the sign-in census.

**The break for it is the half-fix, not the absent one**, and that mattered. A
version that calls `when_it_goes` with a literal instead of what the window
believes compiles, type checks, runs the decision, uses its answer, and leaves
the switch read by nothing again. The first draft of the census asserted only
that `when_it_goes` is named and passed on it. It asks about `reachability_of`
as well now.

**`--no-fail-fast` is not decoration, and leaving it off cost a wrong
measurement.** The first attempt at the first record ran `cargo test
--all-targets` with the runner's default, which stops at the first failing
target. An unrelated check was red, the run stopped above the target that
mattered, and the collected list read as complete: many targets reported, all
green, one failure. The error is always in the direction of too few, which is
the direction `guards.toml`'s own header says the file exists to prevent.

The third record was measured twice for the same reason: once beside a count
check that was already red, and again after the counts were written, because a
break measured beside another failure is measured against a stopping point
rather than against itself.

**Guard re-measurements: five record measurements across three runs**, all
agreeing.

**The widened check was proved by hand.** `test_the_quiet_arms_are_each_said_somewhere_else`
matched `try_send(UIUpdate::OfflineModeChanged(`, and the new sender is inside a
spawned task awaiting an ordinary `send`, so it arrived unwatched. That is the
check's own warning about a second sender happening to the check rather than to
the code. It matches the shorter string now, which is a substring of the longer
one and finds both. Taking the sentence out from in front of the new sender was
run by hand and it goes red.

## Premises that were wrong

### 1. `src/application/outbox.rs` does not exist

The outbox is `src/data/message_cache/outbox.rs`, an `impl MessageCache` in the
data layer. Two consequences, and the second is worse than the first.

The plan says to write the decision "in `outbox.rs`, beside
`when_a_queued_message_may_go`". Following that would put a pure decision inside
a database module. It went to `application::sending_later` instead, which owns
`GoAfter` and `readiness`, which the outbox module already imports, and whose
own doc says values in and values out, no database and no window.

And the plan's verification command for task 1 is
`cargo test --lib application::outbox:: --lib presentation::wx_app::`. The first
filter matches nothing and a filter that matches nothing exits zero. Following
that step literally would have reported success on the task with the defect in
it. The two halves are the same wrong path written twice, so they agree with
each other while both being wrong.

### 2. `flush_outbox` has two callers, not one

The plan and `03-RESEARCH.md` both say it has "exactly one caller, the menu item
at `:4877`". The second is the composer's Send, which queues the message and
flushes the queue in the next line. That is the caller the whole plan is about:
without it the promise on the View menu would be false only for messages that
were already in the queue, and with it every message sent while offline mode was
on went straight out.

It also changes what the defect is. The message was always queued. What was
missing was the queue not being emptied a line later.

### 3. The end of a mail check cannot notice the network coming back

The plan says to ask at the end of a mail check, "where `watch_folder` is
dispatched", and calls that "a natural place". Nothing in this program checks
mail on a schedule. `spawn_mail_sync` has three call sites: the Check Mail menu
item, Get Older Messages, and the folder watch firing. All three stop when the
network goes.

So a check living there could observe the network leaving, on its last pass, and
could never observe it coming back. Task 3's whole deliverable would have been
unreachable in the shipped build while passing every test, because the state is
driven directly in its tests. It asks on the window's own poll instead, which
keeps running whatever else has stopped.

### 4. A `windows` crate feature is the wrong shape for this project

The plan asks for one, and says to comment it in the style of the four already
there. `Cargo.toml`'s own comment says the `windows` crate is "Only for the
platform spell checker, which is COM. Everything else this project needs from
Windows is a flat call behind a small `#[link]` block, and stays that way."
Twenty `#[link]` blocks in `src/` agree. The detection is a `wininet` link
block and no dependency changed.

### 5. `offline_mode` has more than four occurrences, and one of them is a
second false promise

The plan and the research both say four. There were seven, plus four in tests.
The three the count missed are the second half of the toggle at 4896, a
keyboard-shortcut description reading "Toggle offline mode (queue outgoing
mail)", and a spell-check dictionary entry. The shortcut description was making
the same promise the status line was, and it is true now for the same reason.

### 6. Sixty-one guard records name `wx_app.rs`

The plan says sixty-one. What the count check fingerprints is 199 tests in that
file across the records that name it, and the record count is thirty-four. This
is the fifth plan in this phase to find the same figure quoted the same way, and
it is a count of mentions rather than of records.

## The defect this found, and it is not this plan's to fix

**The ten second Undo Send hold is never applied to anything.**

`Hold`, `GoAfter::held` and `queue_outbox_message_to_go` have no caller outside
`sending_later.rs` and its own tests. The composer's Send queues through
`queue_outbox_message`, which writes `GoAfter::AsSoonAsPossible`. So `readiness`
answers `MayGoNow` at once, `take_back` answers `TooLate`, and **Undo Send can
never catch a message somebody has just sent from the composer.**

`sending_later.rs`'s module doc says "All of it runs. The queue carries the
time, the send loop asks `readiness` on every pass rather than taking the whole
queue, and Undo Send is on the Tools menu with `Ctrl+Shift+Z`." The last clause
is true and the first is not.

It is the same class as the defect this plan was written to fix, one layer over:
a promise made in a sentence somebody reads, kept by nothing. Found by asking
what the send path really writes on a row, which was necessary for this plan's
own decision.

Left alone deliberately. It is a behaviour change of its own with a countdown to
show, a version bump and its own red half, and folding it in here would have
made this plan two features. Ledger entry 81.

## Deviations

**The state lives in `src/application/the_network_coming_and_going.rs`, not in
`src/service/network.rs`.** The plan's file list names one file for both halves
and its action asks for them to be separable. They are separate modules: the
platform call is in `service`, where this project's I/O lives, and the belief is
in `application`, where its testable decisions live and where the guard records
look.

**The tick on the View menu moved into the `OfflineModeChanged` arm.** Two
places set the mode now, and a tick set at one of them is a menu that disagrees
with the program about which mode it is in.

**The changelog for tasks 2 and 3 is one entry, written in task 2's commit and
extended in task 3's.** Task 2 alone is user visible, so leaving it undocumented
for a commit would have been a build that switches itself offline with nothing
saying so.

**No new tests were added to `wx_app.rs`, and that is a deliberate trade with a
cost.** Thirty-four records fingerprint that file's test count, at a build and a
full library run each. The census is an integration target coupled through
`guards.toml`, which is what 03-07 did and for the same reason. What is
therefore not guarded from inside that file: that the timer really asks the
network, and that the offer arm shows the panel.

**The arm that raises the offer says nothing and is not registered as quiet.**
`what_is_shown_and_never_said` only reads arms that write to the status bar, and
this one does not, so a registration there would be dead. The reason is a
comment beside the arm instead.

**One red failed on an assertion it should not have made.** Task 2's sentence
test asked for the substring "not been sent" against a sentence reading "nothing
in the Outbox has been sent". Read before it was adjusted: the sentence does say
what the test is about and the assertion was wrong about the words. It pins the
whole clause now, on purpose, because that clause is the one promise guardrail 7
makes to somebody.

**Two clippy failures stopped a commit and both were real.** A mutex guard held
across a match scrutinee, on a lock the interface thread and a background sync
both want; and a `clone` on a `Copy` type.

## What this cannot see, and what it costs

**Nothing here has met a real network loss, and no account has ever been used
with this program.** Six ledger entries rather than ticks, 76 to 81, five of
them `unrun-verify` and one a deviation:

- whether the offline sentence is heard once and understood as a state rather
  than as an error, with a sync failing its way through several folders
  underneath it (76)
- whether the offer is announced with its full label, and whether a screen
  reader user learns the button is there at all, given that a panel appearing
  moves nothing and takes no focus (77)
- whether the status bar and the announcement being the same words reads as
  reassurance or as repetition to somebody who meets both (78)
- whether somebody who lets the offer go by can find their way back, since
  nothing says it again (79)
- whether `InternetGetConnectedState` answers usefully on a real machine losing
  a real network, how long Windows takes to change its answer, and what a
  flapping wifi produces (80)
- the Undo Send hold with no caller (81)

**The detection cannot see a network that is up and cannot route.** It asks
whether this computer has a connection at all, which is right for a cable pulled
out and a wifi dropped and says nothing about whether a mail server is
reachable. The changelog says so.

**Offline mode still does not stop a mail check.** Nothing reads `offline_mode`
except the send path, so Check Mail with offline mode on still tries and still
fails. That is unchanged behaviour and a person asked for it, so it is left
alone rather than changed silently.

**Send Queued Mail still flushes while offline mode is on.** A person chose the
menu item, which is what guardrail 7 allows. The census counts it as one of the
three and names it.

**The guard sweep is owed**, per `CLAUDE.md`, once the phase is complete. Four
records were touched here and all four measured; the sweep is the one that asks
about the other 584.

**`scripts/guards.sh --touched-by 0d48729` after the merge.** This branch
changes `wx_app.rs`, which thirty-four records fingerprint, so it is an
overnight job and must not block the merge.

**Nothing was merged and nothing was pushed.**

## Requirements and criteria

**Criterion 5 is closed structurally, all three deliverables.** Losing the
network puts the application into offline mode without anybody finding the View
menu and says so once. Regaining it offers rather than flushing, and the outbox
is not touched until somebody presses the button. The status bar and the
announcement carry one string.

**It is not closed by ear**, and entries 76 to 79 are why. It is not closed in
the field either: no real network has ever gone while this program was running.

**The live defect is fixed.** The offline toggle no longer promises something
the build does not do, and the changelog says plainly what it used to do.

## Self-Check: PASSED

- All six commits are in `git log`: ee432d6, 7d3f9f0, d6da4a1, b92addc, 51ba4d0,
  4cd8e65.
- The three new files exist: `src/service/network.rs` (3 tests),
  `src/application/the_network_coming_and_going.rs` (10 tests),
  `tests/nothing_leaves_the_outbox_unasked.rs` (7 tests). All passing.
- `scripts/check.sh all` passed on the finished branch: 6,195 library tests
  passed, 0 failed, 1 ignored, every integration target green, release build
  clean.
- `guards/guards.toml` holds 588 records, three of them new here and one
  re-pointed.
- `grep -n '^version' Cargo.toml` reads `version = "0.56.0"`, up from 0.53.0 in
  three steps, one per behaviour change.
- `docs/changelog.md` carries two entries from this plan under `[Unreleased]`,
  each naming its limits.
- `.planning/WINDOWS.md` holds entries 76 to 81, all open, and they landed in
  this worktree rather than in the shared checkout, which was checked rather
  than assumed: the shared checkout's ledger still ends at 75 and its working
  tree was not touched.
- `grep -c offline_mode src/presentation/wx_app.rs` reads 9, up from 7, and
  three of them decide something where none did.
- The working tree is clean apart from this document.

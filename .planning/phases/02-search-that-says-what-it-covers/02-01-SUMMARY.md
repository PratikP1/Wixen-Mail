---
phase: 02-search-that-says-what-it-covers
plan: 01
subsystem: permissions
tags: [allowed, imap, settings, guards, accessibility]
status: complete
requires:
  - "application::allowed as a two-field permission model"
  - "service::outward::permitted as the one gate answer for mail transports"
  - "the write census in service::outward and its MAIL_TRANSPORTS floors"
provides:
  - "Allowed::reading, on by default, narrowed independently of the two writes"
  - "Allowed::READING_SECTION and MESSAGE_TEXT_LABEL, the heading and label a refusal names"
  - "service::outward::permitted_to_read and read_refusal, worded for a read"
  - "ImapSession::allow_reading, may_read and the gate asked by fetch_body"
  - "a settings control on the Permissions tab, wired both ways"
  - "MAIL_TRANSPORTS entries that say whether a count is exact or a floor"
affects:
  - "every AppConfig on every machine: a third serialised field with a named default"
  - "connect_imap, which now reads the account's answer twice rather than once"
  - "four existing guard records, whose red lists grew"
tech-stack:
  added: []
  patterns:
    - "a named serde default rather than a bare one, following data::config::default_true"
    - "a hand-written Default where the derive is right for two fields and inverted for a third"
    - "a source-reading census that asserts both that a marker is distinct and that something counts it"
key-files:
  created: []
  modified:
    - src/application/allowed.rs
    - src/data/config.rs
    - src/application/mail_controller.rs
    - src/service/protocols/imap.rs
    - src/service/protocols/pop3.rs
    - src/service/outward.rs
    - src/presentation/wx_settings.rs
    - src/presentation/first_run.rs
    - src/presentation/command_line.rs
    - src/application/pop_sync.rs
    - guards/guards.toml
    - docs/changelog.md
decisions:
  - "The read section is headed Message Text, not Reading Mail, because the dialog already has a Reading tab and a refusal names the heading by name."
  - "The read checkbox carries an accessible name as well as a label, against the plan, because the plan's stated reason misread tests/checkbox_labels.rs."
  - "MAIL_TRANSPORTS entries carry whether they are a count or a floor, rather than the imap comparison being special-cased."
  - "No command-line flag for the read dimension: --read-only already means the opposite thing."
metrics:
  duration: one session
  completed: 2026-08-31
actuals:
  tokens: 14200
  tasks: 3
  commits: 3
---

# Phase 2 Plan 1: A read dimension on Allowed Summary

`application::allowed` answers three questions instead of two, the third is
whether message text may be fetched from a server, it defaults to on, it has a
control somebody can find and operate by keyboard, and an IMAP session that has
not been allowed to read refuses `fetch_body` with a sentence rather than an
error code.

## What works, and what has not been proved

Working and exercised end to end in tests: the field, its serialisation, the
three constants, the narrowing, the settings control at both ends, the session
gate, and the line in `connect_imap` that reads the account's answer. The gate
is on the real path rather than beside it, and that was measured rather than
argued: turning `allow_reading` into a no-op reddens four tests, two of them
pre-existing `fetch_body` tests that know nothing about permissions.

**Not proved, and it needs a screen reader.** The new checkbox announces itself
under NVDA and Narrator. Structure is present, both channels are served, and
CLAUDE.md's second guardrail is explicit that this is not the same as it
working. What to check: on the Permissions tab there is a group headed "Message
Text" holding one box, the box is reached by Alt+F, and both readers say its
full label rather than "check box" alone.

**Not built here, by design.** Nothing yet calls `fetch_body` for a message
whose text was never stored. This plan makes the gate real and puts the setting
in front of somebody; the fetch it will guard is plan 02-03. Until then the
setting changes what a user can express and does not yet change what the program
fetches, because the program does not fetch it yet either way.

## Commits

| Task | Commit | What |
|------|--------|------|
| 1 | `a0e4261` | The field, its default, the serde handling, the session gate, the wiring |
| 2 | `e52b69e` | The settings control and the test that fails if it is removed |
| 3 | `6cd04a0` | The census that reads both protocol files, and five guard records |

## The two traps, and what was done about each

**`Allowed::NOTHING` holds `reading: true`.** The first-run choice labelled
"Read my mail, change nothing" resolves to that constant, and so does
`--read-only`. A reading field defaulting to false there would have taken mail
away from the most cautious person in the user base, silently. The constant's
doc now says why its shape stopped matching its name, and three tests assert it
by name in three modules: `allowed`, `first_run` and `command_line`. `Default`
is written out by hand, because the derive gives false for a bool.

**A third serde field must not break every existing config file.** The field
carries `#[serde(default = "default_reading")]` naming a function that returns
true, following `data::config::default_allowed`'s rule that a default is named
rather than restated. The test deletes the key from a serialised `AppConfig` and
asserts both that reading answers true and that the other settings survive,
because asserting only on reading would pass against a file that lost
everything else. Taken red by hand: a bare `#[serde(default)]` reddens it.

**The read gate is not named `may_i(`.** It is `may_i_read`, and both halves of
that decision are asserted rather than one:
`test_the_read_gate_is_told_apart_from_the_write_census` requires that the read
marker contains no write marker and that no write marker contains it, and
`test_the_read_gate_does_not_move_the_count_of_gated_writes` requires the imap
census to find exactly eleven writes, so the read gate is counted by something.
Without the second, naming the gate clear of the markers would have left a gate
no census could see.

**`RETR` is not gated.** The plan's correction was right and the reason is now
in the code above `retrieve`, with a test holding it. Taking that comment's
claim red is the strongest evidence in this plan: gating `RETR` reddens three
tests, one of which is an ordinary POP retrieval test, so the failure is mail
not arriving rather than an assertion about mail not arriving.

## What was measured, not assumed

**RED.** The pre-commit gate on this branch runs the tests reaching what changed,
so a commit whose tests fail cannot be made, and `--no-verify` is not an option
here. There is therefore **no separate RED commit for this plan**, and the red is
recorded as a measurement instead. This is stated plainly rather than left to be
inferred from the commit graph, which shows three green commits and would
otherwise read as test-after.

The red was taken from values, not missing symbols: the field was added first
with `false` everywhere and a bare serde default, so every new assertion failed
on a value. Against that stub, **9 of 12 new tests in task 1 were red**. The
three that were green are named, with what each was then broken by hand to prove
it discriminates:

| Test | Why green against the stub | Break that reddens it |
|------|---------------------------|----------------------|
| `test_reading_is_not_a_change` | `anything()` was already writes-only | widening `anything()` to include reading |
| `test_being_allowed_to_change_things_does_not_also_allow_reading` | the two setters were already separate | making `allow_changes` also set `may_read` |
| `test_a_session_allowed_to_read_reaches_the_command` | cannot be red before the gate exists; it is the positive control | making `allow_reading` a no-op, which reddens four |

Task 2's test was red on the absence of the control, then taken red twice more
after it passed, once per wiring half: showing a fixed value instead of the
stored one, and reading back a constant instead of the widget. Each broke
exactly that one test out of 5834.

Task 3's equality was proved to be worth more than the floor it replaced, rather
than asserted: a twelfth counted write added by hand fails the equality and
would have cleared the old `>= 11`.

**The `may_i` census, re-counted.** `grep -rn "may_i" src/ | wc -l` gives **24**
now and gave **16** before this plan. The sixteen were eleven write call sites in
`imap.rs`, one in `pop3.rs`, one gate definition in each file, and two string
literals inside checks that search for the gate by name. The eleven is the number
`MAIL_TRANSPORTS` carries and now asserts as an equality.

**Guard records.** Three new ones, each taken red against the whole library
before being written down. Fourteen records re-run in total, chosen as the three
new ones, the two other records on `allowed.rs`, and every record about the write
census, the imap gate or the POP gate. Five came back stale; four were this
plan's doing and are corrected and re-verified:

| Record | Was | Now | Why |
|--------|-----|-----|-----|
| a mailbox write asks the gate before it reaches the session | 3 | 4 | reads the same census |
| creating a folder on a server asks the gate... | 4 | 5 | same |
| deleting a folder on a server asks the gate... | 3 | 4 | same |
| a POP removal refused by the setting says nothing to the server | 4 | 5 | the new POP test reads that gate as its positive control |

None of their guarded code changed. They went stale because this plan added
tests that read the same census, which is the direction only an equality check
in both directions can see.

## Deviations from Plan

### 1. [Rule 2 - correctness] The read checkbox carries an accessible name

**Found during:** Task 2.
**Plan said:** do not add `set_accessible_name` to a labelled checkbox, citing
`tests/checkbox_labels.rs:21-23`, and keep `grep -c 'set_accessible_name(&'`
unchanged as proof.
**What that file actually says:** the defect is an **empty** label plus
`set_accessible_name`, which names a control on MSAA and leaves it unnamed on UI
Automation. Lines 21-23 are about one wxWidgets application per process. A label
plus a name serves both channels, which is what the two existing boxes on that
page do.
**Done instead:** the box carries both, with the accessible name derived from
the label constant with its mnemonic marker removed, so there is one string and
nothing to drift. Leaving it off would have made it the only control on that
page with nothing set on the channel NVDA reads.
**Commit:** `e52b69e`.

### 2. [Rule 1 - bug] The heading is "Message Text", not "Reading Mail"

**Found during:** Task 2, choosing the constant's value.
**Issue:** the settings dialog already has a tab headed "Reading". A refused
fetch tells somebody to turn the setting on by name, and "turn on Reading Mail
in Settings" would have sent them to that tab, where there is nothing of the
kind. That is the failure `SETTINGS_SECTION`'s own doc records, one step further
out: there the sentence and the heading differed by a word, here by a page.
**Commit:** `e52b69e`.

### 3. [Rule 3 - blocking] `MAIL_TRANSPORTS` carries what its number means

**Found during:** Task 3.
**Issue:** the plan asked for the imap entry to become an equality while smtp
and pop3 stayed floors, from one shared assertion. Special-casing one path by
name inside the assertion would have put the decision where nobody reading the
constant could see it.
**Done instead:** a `Counted` enum with `Exactly` and `AtLeast`, so each entry
says which it is and why, and the assertion asks the entry. smtp and pop3 stay
`AtLeast`, untouched, as the plan asked.
**Commit:** `6cd04a0`.

### 4. [Rule 1 - bug] The settings read-back carried the stored value through task 1

**Found during:** Task 1.
**Issue:** the read-back is compiler-forced to supply the third field, and task
1 has no control to read it from. Writing a constant there would have rewritten
somebody's stored answer every time they opened Settings and pressed Save.
**Fix:** carried the stored value through in task 1, replaced by the widget's
value in task 2.
**Commits:** `a0e4261`, `e52b69e`.

## Wrong premises found

Reported rather than built on, as asked.

1. **The plan's own premise correction is wrong about the count it corrects.**
   It says `grep -rn "may_i" src/` gives fourteen, and instructs "do not quote
   sixteen anywhere". Running it gives sixteen. The research document's error
   was calling all sixteen writes; the correction's error was changing what was
   being counted, from matching lines to call sites, while keeping the phrasing
   "what the command gives". Both documents are describing something true and
   neither number is what it claims to be. The instruction not to quote sixteen
   made the error load-bearing, because the true reading was pre-labelled as the
   mistake. This summary quotes the raw count and the breakdown separately.

2. **The plan misreads `tests/checkbox_labels.rs`.** Covered as deviation 1
   above. The cited lines say something else, and the rule as stated would have
   removed an accessibility name rather than avoided a defect.

3. **`grep -c 'serde(default' src/application/allowed.rs` returns 3, not the 1
   the acceptance criterion requires.** The property the criterion stands for
   holds: there is exactly one such attribute on `Allowed`. The other two are
   the field's doc comment explaining, twice, why a bare default would be wrong.

4. **The `set_accessible_name(&` count criterion cannot see its subject.** It is
   unchanged at 23, and not for the reason intended: two such calls were added
   and the formatter wrapped both, putting the `&` on the following line. The
   criterion would report the same number for any long call anybody ever adds,
   so it is a false all-clear rather than a proof.

5. **CLAUDE.md's "an hour or two" for a full guard sweep is stale by an order of
   magnitude.** Measured: 536 records, each a rebuild plus a 200-second library
   run; the first thirteen had not finished after twenty-four minutes. The real
   figure is sixteen to thirty-two hours. See "What was not done" below.

## Known stubs

None. Nothing here is a placeholder. The one thing that looks like an absence is
deliberate and named above: no code yet fetches a body for a message whose text
was never stored, and that is plan 02-03's work rather than a stub left here.

## What was not done

**The unfiltered `scripts/guards.sh` run.** The brief asked for it and it was
started; it is a sixteen to thirty-two hour job on this machine and cannot be
completed in a session. What was run instead is fourteen records chosen by hand,
which found five stale records including four this plan caused. Stopping the
full run left `src/application/contacts_sync.rs` broken in the working tree,
because a kill skips the `finally` that restores it; that was noticed and
restored with `git checkout --`, and the tree was verified clean before every
commit.

Both this and one pre-existing stale record are written up with their
measurements in `deferred-items.md` beside this file, so acting on either does
not mean measuring it again.

**A pre-existing stale record was found and deliberately left.** The record
`a WebDAV read cannot carry a changing verb` names one test and its break
reddens two. `git diff 8836efc -- src/service/outward.rs` confirms this plan
touched neither that test nor the constant it walks, so it was already short of
the truth before this branch. Folding an unrelated correction into these commits
would make them say two things. The one-line fix and its measurement are in
`deferred-items.md`.

## Verification

- `cargo test --all-targets --no-fail-fast`: 5837 library tests and every
  integration target green, 0 failed.
- `bash scripts/check.sh` green on every commit: formatting, clippy with
  `-D warnings`, the tests reaching what changed, and the tree-reading guards.
- `bash scripts/guards.sh` over the fourteen relevant records: all seven that
  this plan owns or corrected redden exactly what their records name, in both
  directions.
- `grep -n '^version' Cargo.toml` still reads 0.46.0.
- No `--no-verify`, no `#[allow(...)]`, no new dependency, no AI attribution.

## Self-Check: PASSED

Every file named in `key-files.modified` exists and is modified relative to
`8836efc`. All three commit hashes resolve: `a0e4261`, `e52b69e`, `6cd04a0`.

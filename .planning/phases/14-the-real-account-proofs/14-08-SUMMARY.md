---
phase: 14-the-real-account-proofs
plan: 08
subsystem: release
tags: [build, installer, sitting-sheets, real-account]
status: complete
requires:
  - phase: 14-07
    provides: the true words the build carries
provides:
  - the installed build every sitting runs on, 1.0.0-alpha.1+1118.gd3455792
  - the install page and the four sitting sheets
  - the sittings reordered to 14-10, 14-11, 14-09, 14-12
affects: [14-09, 14-10, 14-11, 14-12, 14-13]
tech-stack:
  added: []
  patterns: []
key-files:
  created:
    - .planning/phases/14-the-real-account-proofs/14-INSTALLING-THE-BUILD.md
    - .planning/phases/14-the-real-account-proofs/14-SITTING-1.md
    - .planning/phases/14-the-real-account-proofs/14-SITTING-2.md
    - .planning/phases/14-the-real-account-proofs/14-SITTING-3.md
    - .planning/phases/14-the-real-account-proofs/14-SITTING-4.md
  modified:
    - .planning/phases/14-the-real-account-proofs/README.md
    - .planning/phases/14-the-real-account-proofs/14-09-PLAN.md
    - .planning/phases/14-the-real-account-proofs/14-10-PLAN.md
    - .planning/phases/14-the-real-account-proofs/14-11-PLAN.md
    - .planning/phases/14-the-real-account-proofs/14-12-PLAN.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The sittings run 14-10, 14-11, 14-09, 14-12, on Pratik's 'installed, key later' of 2026-10-06; the plans keep their numbers and the sheets their names."
  - "The first start and the two deletes of 2026-09-20 moved from 14-09 to 14-10, with their verdict, their #63 record, their REAL-02 line and ledger 523 and 525."
metrics:
  completed: 2026-10-06
  duration: "about 2 h 40 min across two executors, 11:42Z to 14:25Z, with Pratik's install between"
actuals:
  tokens: 38000
  tasks: 3
  commits: 2
---

# Phase 14 Plan 08: the build, the install page and the four sitting sheets Summary

The build `1.0.0-alpha.1+1118.gd3455792`, made from 14-07's merge, is installed on Pratik's
machine and read back from the registry. The install page and the four sitting sheets are
written, and the sittings are reordered so the two that need no Google key come first.

## The build (task 1)

Made by the first executor of this plan on `main` at `d3455792`, 14-07's merge, with
`bash scripts/build-installer.sh` in the background, its output in the session scratchpad
(`14-08-build.log`), started 11:42:07Z, exit status 0 read from `14-08-build.status`.

| What | Value |
|---|---|
| Build string | `1.0.0-alpha.1+1118.gd3455792` |
| File | `dist/Wixen-Mail-Setup-1.0.0-alpha.1+1118.gd3455792.exe`, gitignored, not committed |
| Size | 15,536,146 bytes |
| SHA-256 | `d01acb37c583016f79e37045f449afc05a56b22858ecb8539a47499a5b1710f4` |
| Console's version line | `== setup executable, version 1.0.0-alpha.1+1118.gd3455792, file version 1.0.0.14999 ==` |
| Counter line | `== the build counter is 1118 and the file version holds it to 999 ==`, expected (premise 1) |
| Last line | `Built dist/Wixen-Mail-Setup-1.0.0-alpha.1+1118.gd3455792.exe` |

No `.dirty` in the name, so the tree was clean. The plan's task 1 verify, re-run on
2026-10-06 at 14:24Z, exit 0: `M=d34557923955a9995852fc09e81d58e492cb1964`, `git rev-parse main` the
same, and the file under the computed name exists. Pratik's ruling on running the script with
its two `sed` reads came in this plan's brief, "yes", and is recorded in the README's "What
waits on Pratik", item 2.

## The pages (task 2)

Written by the first executor and committed in `8b5d0f7a`; corrected for the new order by this
one.

| Page | Steps | Block steps in its sitting plan |
|---|---|---|
| `.planning/phases/14-the-real-account-proofs/14-INSTALLING-THE-BUILD.md` | 10 install, 2 checks | none, from the plan's task 2 |
| `.planning/phases/14-the-real-account-proofs/14-SITTING-1.md` | 35 | 16 (14-09) |
| `.planning/phases/14-the-real-account-proofs/14-SITTING-2.md` | 53 | 18 (14-10) |
| `.planning/phases/14-the-real-account-proofs/14-SITTING-3.md` | 42 | 17 (14-11) |
| `.planning/phases/14-the-real-account-proofs/14-SITTING-4.md` | 60 | 14-12's block |

Each block step is split into one action per sheet step, so the sheets hold more steps than the
blocks, in the blocks' order.

Sentences the sheets quote, found in the tree at `d3455792`:

| Sentence, in short | File and line |
|---|---|
| "Nothing was asked of Google for this account's calendars. The account signs in to mail with an app password ..." | `src/application/who_holds_the_calendars.rs:132` |
| "Signing in to Google for the account's calendars, contacts and tasks. Finish in the browser." | `src/application/who_holds_the_calendars.rs:1709` |
| "Signed in to Google for this account's calendars, contacts and tasks. ..." | `src/application/who_holds_the_calendars.rs:444` |
| "Calendar sync: N created, N updated, N deleted", and "N calendars read" | `src/application/calendar.rs:209`, `src/application/every_google_calendar.rs:552` |
| "Contacts sync: N created, N updated, N deleted" | `src/application/contacts_sync.rs:1674` |
| "Undid {action} on {item}." | `src/application/undoing.rs:672` |
| "Take away the copy of ... The one it was copied from stays where it is." | `src/application/undoing.rs:915` |
| "Delete \"...\"? Undo brings it back until your next action." | `src/application/pim_command.rs:230` |

Access letters the pages name, found as `&` in their labels: "Sign In for Calendars, Contacts
and &Tasks" (`src/presentation/wx_account_manager.rs:293`), "&Close" (`:297`), "Send and delete
&mail from this account" (`:1440`), "&Save" in the item form (`src/presentation/wx_item_form.rs:649`).

The task 2 check, the plan's Python run from a scratchpad copy (`14-08-verify.py`) with the
build string and the phase folder as arguments:

- On the pages as committed: `[] [] [] True`, exit 0.
- Taken red on a scratchpad copy whose Installed apps step also named `1.0.0.14999`:
  `[] [] [] False`, exit 1. So the check sees the fault it exists for.

`cargo test --test house_style` passed 74 and `cargo test --test the_words_that_say_nothing`
passed 10 in the first executor's run (`14-08-hs.log`, `14-08-wsn.log`); both run again in this
plan's documents commit through the hook.

## Pratik's install (task 3)

His reply, 2026-10-06, in his words: **"installed, key later"**.

Read back with Python's `winreg`, read-only, without starting anything:

```
HKLM DisplayVersion '1.0.0-alpha.1+1118.gd3455792'
HKLM InstallLocation 'C:\\Program Files\\Wixen Mail\\'
HKLM InstallDate '20261006'
```

under `HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\{9C2E6B41-0F7D-4A83-B5E1-27D4A9F3C608}_is1`.
It equals the build string. `oauth.toml` was not opened or checked; 14-09's precondition checks
it by existence.

## Deviations from Plan

**1. The sittings reordered, on Pratik's answer at the checkpoint.** His key is not made, and on
2026-10-05 he accepted copy, move and delete going first if it was not ready (the proposal:
"Sittings 2 and 3 can go first; the build carries everything they need"). So on this branch:

- 14-10 is wave 9 and depends on 14-08; 14-11 is wave 10 and depends on 14-10; 14-09 is wave 11
  and depends on 14-11; 14-12 stays wave 12 and depends on 14-09. Numbers kept.
- The first start, the two deletes of 2026-09-20 with their verdict, their #63 comment and their
  REAL-02 line, ledger 523 and 525, and the first reading of whether the Trash can be read moved
  from 14-09 to 14-10. 14-09 now serves REAL-01 only and posts on #22 only.
- 14-11's precondition and premise 5 read 14-10's summary for the Trash.
- Sheet 2 opens with the first start; sheet 1 assumes the program has started once and needs
  sittings 2 and 3 done. Each heading says where the sheet falls: "Sitting 2, done first",
  "Sitting 3, done second", "Sitting 1, done third", "Sitting 4, done last".
- The install page's "why Start Wixen Mail is unticked" and "the key can wait" paragraphs say
  the new order, with the old wording dated.
- The README's answers table, plans table, order section, what waits on Pratik and handover,
  and ROADMAP's phase 14 entry, order sentence and plan lines say the order and the reason in
  his words.

**2. [Rule 1] Sheet 2's step for the alpha page's item 20 in Tasks deleted the proof task of
sitting 1,** which now comes after it. The step now makes its own local task, "Wixen proof undo
task", with `Ctrl+Shift+T` and `Alt+S`, deletes it, hears Undo's name and undoes it; 14-10's
block step 16 says the same. Its expected sentence "Undid Delete on Wixen proof undo task." is
`undoing.rs:672`'s shape.

**3. [Rule 1] A delete in Calendar or Tasks asks first, and Enter answers No.** Sheet 1's step
30 and sheet 2's new step 48 now say the question and to choose Yes
(`pim_command.rs:230`, `managers.rs:3151-3157`, `KEYBOARD_SHORTCUTS.md:78`). The sheets as first
written said nothing of the question.

## Known Stubs

None.

## Next

14-10, sitting 2, done first: `.planning/phases/14-the-real-account-proofs/14-SITTING-2.md`.

## Self-Check: PASSED

- The five pages, the setup file and this summary exist.
- `8b5d0f7a` is on the branch.

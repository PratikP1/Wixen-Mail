---
phase: 14-the-real-account-proofs
plan: 07
subsystem: the warnings, the pages and the records
status: complete
tags: [real-02, warnings, first-run, help, ledger, issue-63]
requires: [14-03, 14-06]
provides:
  - the warnings say sending was proven on a Gmail account on 18 September 2026 and the rest is not proven
  - the Pause item says what Gmail did to the download of everything
  - the first-run choice names the browser sign-in a Google account needs
  - ledger 148 closed, 191 rewritten, eight entries amended, 829 and 830 opened
  - "#63's comment of 2026-09-18 corrected to the app password"
affects: [14-08, 14-09, 14-10, 14-11, 14-13]
tech-stack:
  added: []
  patterns: [a case reads a button's label from the screen that builds it, so a sentence naming it cannot drift]
key-files:
  created: []
  modified:
    - src/application/allowed.rs
    - src/presentation/first_run.rs
    - src/presentation/command_line.rs
    - src/presentation/wx_settings.rs
    - guards/guards.toml
    - docs/ALPHA_TESTING.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/integration-guide.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
decisions:
  - Sending is called proven, with its day; every other path is "not proven" rather than "never run" (D-21)
  - The undo sentence names sending as the one proven change even though sending cannot be undone, so it reads true beside the settings warning
  - docs/PROVIDER_SETUP.md left as it is; 14-03's paragraph and table above its tasks section already say a Gmail account needs the browser sign-in
metrics:
  completed: 2026-10-06
  duration: about 2 hours
actuals:
  tokens: 30000
  tasks: 3
  commits: 3
---

# Phase 14 Plan 07: True words before the build Summary

Every surface that said sending had never been tried now says it was proven
once, on a Gmail account on 18 September 2026, and that the rest has not been
proven; the download's warning says what Gmail did to it; the first-run choice
for tasks, contacts and the calendar names Sign In for Calendars, Contacts and
Tasks in the Account Manager; the records are put straight and #63's comment
names the app password.

## What changed

| Surface | Before | Now |
|---|---|---|
| Settings, Allow Changes warning (`EXPERIMENTAL_WARNING`) | "none of this has been run against a real account yet" | "Sending was proven once, on a Gmail account on 18 September 2026; the rest has not been proven against a real account yet" |
| Undo's help (`UNDOING_AT_THE_SERVER_IS_EXPERIMENTAL`) | "none of it has been run against a real account yet" | sending proven on that day, the other changes not proven |
| Pause Downloading (`DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL`) | "it has never been run against a real account" | ran against Gmail on 18 to 20 September 2026, closed hundreds of times, sign-ins refused on the 19th, waited and tried again |
| First-run introduction | "None of that has been run against a real account yet" | "Sending was proven on Gmail on 18 September 2026; the rest is not proven yet"; 864 characters, under the 900 bound (840 on `main`) |
| First-run third choice | "None of it has been run against a real account" | sending proven on that day, the rest not proven |
| First-run second choice | "go up to your provider" | "and from a Google account only once it has a browser sign-in for them: Sign In for Calendars, Contacts and Tasks in the Account Manager"; still "never been run against a real account", true of tasks, contacts and the calendar |
| End of `--help` | "have never been run against a real account" | sending proven once on that day; moving, deleting and the syncs not proven |
| `EMPTYING_THE_TRASH_IS_EXPERIMENTAL` | "never been run" | unchanged, still true |

The safety switch is untouched: `git diff main -- src/data/config.rs
src/presentation/wx_first_run.rs` is empty, and no line of `NOTHING`,
`EVERYTHING` or `FOR_TESTING` changed and `Allowed` gained no field.

The pages: `docs/ALPHA_TESTING.md`'s download paragraph, its "Everything that
writes is experimental" paragraph and the first line of "What is already known
to be missing or unproven"; `docs/KEYBOARD_SHORTCUTS.md`'s sentence about a
move to another account; `docs/integration-guide.md:53`. Each correction is
dated. Joined-line searches with Python, `main` against the tree: "Nothing that
writes has run against a real account", "None of that has run against a real
account" and "none of that has met a real account" in the alpha page, "still
waits for both servers" in the shortcuts page and "None of it has run against a
real account" in the guide were each present before and absent after. The same
search over the constants: none of the six edited constants holds any of the old
phrases; `EMPTYING_THE_TRASH_IS_EXPERIMENTAL` and the second choice still hold
theirs.

## Commits

| Commit | What | Hook |
|---|---|---|
| `0f38c954` | red: six cases and the count check, `Fails-until-green` | red, 169 s |
| `ec41345e` | green: the sentences, two comments, two guard records | affected, 356 s (the first attempt was refused after 657 s, deviation 1) |
| documents commit | pages, changelog, ledger, the four marks, this summary | see the merge |

Test counts on the day: `application::allowed::` 28 passed, `presentation::first_run::`
19, `presentation::command_line::` 29. Document targets: `house_style` 74,
`docs_links` 6, `the_words_that_say_nothing` 10,
`every_number_carries_its_command_and_its_date` 29,
`the_planning_files_agree_with_themselves` 17, `presentation::help_page::` 12.

## Guard records

"the download's warning says no real account has met it" is renamed "the
download's warning says what Gmail did rather than that it never ran" and
anchored on the new sentence; its break puts back the claim that it never ran.
"the settings warning says sending was proven on a Gmail account and when" is
new; its break puts the old opening back. Both, with the seven the count check
flagged, were re-measured in one `scripts/guards.sh --remeasure` call, run in
the background with `--log`: all nine redden exactly what they name, 9 of 9.
The arrived-since count at the top of `guards/guards.toml` went from 690 to 691.

## The ledger

148 fixed, its reason citing #63's comment of 2026-09-18 and the app password.
191 rewritten, dated, to the resume 11-07.2 built; open. 11, 64, 65, 67, 72,
523 and 525 each carry a dated sentence with research 1's counts of 2026-10-04
(932 text fetches lost to the connection closing while a folder opened, 139 text
runs stopped short, 321 waits before trying again, and on 2026-09-19 118
sign-ins after a drop and 2 watches refused as too many simultaneous
connections); each stays open. 546 carries the five replayed deletes of
2026-09-20 as evidence, not proof. New: 829, the Trash unreadable at nearly
every check on 18 and 20 September (107 and 79 times), read at sitting 1 by
14-09; 830, Refresh's help sentence "Read this folder again from the server",
both ways out named for Pratik. `grep -c 'Read this folder again from the
server' .planning/WINDOWS.md` answers 2, one per half, having answered 0.
Frontmatter: open 704, fixed 126, total 830.

## The comment on #63

Corrected on 2026-10-06 at 10:36:58Z with `gh api -X PATCH`, on answer 4. The
plan's verify read it back: "app password" present, the old bracketed opening
absent, exit 0. The old body, for the record:

> **Sending: proven.** 2026-09-18, the tester, build `1.0.0-alpha.1+149.g744d05ef`, his Gmail account (OAuth, SMTP through the outbox and the Allowed Changes gate): a message went out and arrived. His words: "Sending works. It's confirmed." That is the first write-side path to meet a real server. Ledger 148 (a message that leaves after a hold has never met a real server) closes on it. The commit that moves `application::allowed`'s default for sending and rewrites the warning sentences on the settings screen, the first-run screen, the end of `--help` and `docs/ALPHA_TESTING.md` is the next step and waits on Pratik's word, since ROADMAP.md:18 put it outside the milestone. Move and delete remain unproven (ledgers 187, 191, 376).

The new body replaces "(OAuth," with "(signed in with an app password," and adds
one paragraph saying it said OAuth until 2026-10-06, and that the account's row
and every log line of 18 to 20 September show the app password.

## Deviations from Plan

1. **[Process] The first green commit was refused** by
   `the_planning_files_agree_with_themselves`: the four marks were already in
   the working tree and this summary was not, so `STATE.md` counted 256 and the
   disk held 255. A first version of this file was written and the commit made
   again. 14-06 met the same thing.
2. **[Premise] `UNDOING_AT_THE_SERVER_IS_EXPERIMENTAL` held "none of it has been
   run"** in lower case, which the plan's case-sensitive phrase list would have
   passed on `main`; it was read and changed all the same.
3. **[Premise] Counts on the day:** the register held 1,487 records, not 1,437,
   and 1,488 after; the introduction was 840 characters on `main`, not 860;
   `why_it_cannot_be_finished_from_here` is at `mail_across_accounts.rs:884`,
   not `:793`; Refresh's help is at `wx_app.rs:7471` and its mail-folder handler
   at `:4445-4458`.
4. **[Shape] The download record was renamed** as well as re-anchored, since
   its name said the opposite of what it now guards.
5. **[Shape] The second choice's case reads `wx_account_manager.rs` with
   `include_str!`** and requires the button's label there, so the name in the
   sentence cannot drift from the button.
6. **[Scope] `--help` now names moving** among the unproven writes; the old
   paragraph listed sending, deleting and the syncs and left moving out.
7. **[Scope] The alpha page's download paragraph** also said "the account you
   point it at is the first one it meets", false since Gmail met it; corrected
   in the same edit.
8. **[Wording] The integration guide says "proven", not "met"**, because the
   deletes replayed on 2026-09-20 did meet Gmail.
9. **[Premise] `docs/PROVIDER_SETUP.md` left as it is.** Its line 72 says tasks
   go up on the next sync, but 14-03's paragraph and the table directly above
   it say a Gmail account needs the browser sign-in, so the page says no more
   than the program does.
10. **[Process] Two slips against the brief, neither touching a tracked file.**
    One wait loop read the re-measure log in the scratchpad with `sed -n`, which
    the brief forbids even read-only. And a `grep -rn` over the whole tree,
    `target` included, ran in the background for about 30 minutes, beside the
    re-measure, until its time limit stopped it; the re-measure's nine verdicts
    came out exact regardless.

## Known Stubs

None.

## Threat Flags

None beyond the plan's register: the edit on #63 names the sign-in method and
the build, never an address, a password or a token. Nothing was installed.

## Self-Check: PASSED

The red and green commits exist on the branch; the six constants and the four
pages read as above; the comment's read-back exits 0.

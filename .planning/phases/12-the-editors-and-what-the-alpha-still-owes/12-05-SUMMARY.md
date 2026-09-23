---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 05
subsystem: Send Feedback, the report it composes, the facts it reads, and the one sending path
tags: [alpha-02, feedback, redaction, msaa, outbox, security, "#64", "#71", "#78"]
status: complete

requires:
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "12-04: the About dialog this plan adds its button to, merged at 2e00c42b"
provides:
  - "src/application/feedback_report.rs: Category, Question, Fact, Include::for_category, Facts, LogFile, Report, compose, redact, last_lines, SUPPORT_ADDRESS, SECURITY_ADDRESS, where_it_goes, ISSUE_PAGE, PRIVATE_REPORTING_PAGE, github_page; 22 cases"
  - "src/service/this_machine.rs: windows_build, display_language, screen_reader, which_reader, describe_windows, file_version_text, providers; 6 cases"
  - "src/presentation/wx_feedback.rs: build_feedback_dialog, FeedbackDialog, what_the_doors_do, reticked, sender_of, log_to_read, log_file_name, from_line, payload_text, keep_a_copy; 12 cases"
  - "src/presentation/wx_app.rs: put_in_the_outbox (the queued row, lifted out of queue_for_sending), open_send_feedback, show_send_feedback, send_the_report, the Help item on Ctrl+Shift+F, About's Send Feedback button, ScanTarget::Feedback's arm"
  - "src/common/paths.rs: feedback_dir, logs\\feedback"
  - "tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs: 10 readings"
  - "seven guard records written, seven re-measured"
affects: [12-06 and later, which read put_in_the_outbox where they read queue_for_sending; 12-12, which reads the four pages as one and runs the whole suite]

actuals:
  tokens: 142000
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "One queued-row function shared by the composer and a second sender, held by a reading of the source that counts row constructions and asks each sender for the call"
    - "A test that reads a hidden dialog filters controls by their own WS_VISIBLE bit, since IsWindowVisible answers for the parents too"
    - "A dialog whose handlers are one line each calling a named method, so the target calls the method and the payload's text link is proved by set_value raising the event"

key-files:
  created:
    - src/application/feedback_report.rs
    - src/service/this_machine.rs
    - src/presentation/wx_feedback.rs
    - tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs
    - .planning/phases/12-the-editors-and-what-the-alpha-still-owes/12-05-SUMMARY.md
  modified:
    - src/application/mod.rs
    - src/service/mod.rs
    - src/presentation/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/scan_target.rs
    - src/common/paths.rs
    - .github/workflows/accessibility.yml
    - tests/the_about_dialog_names_its_owners_and_its_links.rs
    - tests/wired.rs
    - tests/nothing_leaves_the_outbox_unasked.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/installing.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "The report is composed in one pure function and the payload box shows payload_text(compose(...)), which is exactly what Send queues and what the kept copy holds"
  - "About's Send Feedback button closes About with ID_SEND_FEEDBACK and the menu arm opens the dialog, because About holds none of the accounts, cache or runtime a report needs"
  - "A box the person changed keeps its state when the category changes; every other box follows the new category (the planner's choice, overrulable)"
  - "Today's log is read, or yesterday's when today's is missing or empty; the days are UTC days, because the logger rolls its file at midnight UTC"
  - "The kept files are named 2026-09-23-184512-report.txt and -log-excerpt.txt, not 20260923-..., because house_style keeps the compact day format to the calendar writer"
  - "Send Feedback is the sixth place that hands mail to a server, a button a person presses, written into the outbox census"

metrics:
  duration: "about 3 hours 20 minutes, from about 18:55Z to 20:40Z on 2026-09-23 for the branch's commits and documents, before the pull request's wait"
  completed: 2026-09-23
---

# Phase 12 Plan 05: Send Feedback Summary

**It works in every reading a test can take, nobody has heard it, and no report has reached either address.** Help, Send Feedback (`Ctrl+Shift+F`) and a new Send Feedback button on About open one window. It asks what the report is about first (Report a problem, Request a feature, Something is hard to use with a screen reader, Ask a question, Report a security concern, Something else), then the question or two that fit, then five boxes each saying what it sends, with the version and the last 200 lines of the log ticked. The box "What will be sent" holds the exact message and attachment and changes as the person types. Send keeps a copy under `logs\feedback` and puts the report in the Outbox from the default account (or the account in use when none is marked), through the same row function, hold and gate as every message.

What each kind of report sends, from `compose` and `payload_text`:

| | An ordinary report (a problem) | A security concern |
|---|---|---|
| To | support@wixen.app | security@wixen.app (`SECURITY_ADDRESS`, the planner's name for Pratik to correct) |
| Subject | `[Wixen Mail] Report a problem: ` and the first line of the first answer, up to 80 characters | `[Wixen Mail] Report a security concern`, none of the concern |
| Body | each question with its answer under it, then `Version: ...`, then any other ticked fact on its own line, `Reply to: ...` when given, `Attached: <stamp>-log-excerpt.txt, ...` when the log goes, and a closing line naming the address and that a copy is kept | the same shape |
| Log excerpt | ticked; the last 200 lines of today's log, every address through `mask_email` and everything after a subject marker replaced | unticked; goes only if ticked, and the payload box shows it first |
| GitHub button | the issue page | GitHub's private reporting page, beside Send whether Send can be used or not |

With no account, or sending off under Settings, Allow Changes, Send is shut and a sentence under the buttons says which; Copy to clipboard and the GitHub page stay.

The real machine answered through a throwaway example program, since removed: `"Windows 11, build 26200"`, `"en-US"`, `Some(("NVDA", "2026.3.0.57645"))`.

## The red halves

**Task 1** (`bd3b20be`): every function stubbed with a wrong answer that compiles; the redaction stub returns nothing rather than its input so the case for a line with no address is red too. 29 of 29 red on an assertion, none green: 22 in `application::feedback_report`, 6 in `service::this_machine`, and `common::paths`'s listing case rewritten in place to want `logs\feedback`.

**Task 2** (`7d57ba23`): the pure functions stubbed empty, the doors shut with no sentence and no page, the dialog built bare. 25 of 25 red: 12 in `presentation::wx_feedback`, the new target's 10, the About order reading and its absence reading rewritten in place, and `scan_target`'s listing case. The two companions were red for a reason of their own (nothing to plant in, no box to untick), said in the commit. The first run of the target said "0 controls where 14 are wanted" against the stub for the wrong reason: `IsWindowVisible` answers false for every child of a dialog nobody shows. The reading now reads each control's `WS_VISIBLE` bit. The first attempt at this commit was refused by the gate for two unnamed failures, both real: a comment in the target saying the main window "cannot be built in a test" (house_style's guard against that claim) and the stub's buttons with empty labels (wired's unnamed-control guard). Both fixed before the red was committed.

## Guard records

Seven written, each measured with `scripts/guards.sh --remeasure`, each "went red, and nothing else did". No break can start another program: none clicks, opens a page or sends; the second record's break is compiled and never run.

| Record | Break | Red |
|---|---|---|
| a report of a problem carries the log excerpt by default | `for_category` unticks it for `Problem` | 2 of 2, lib |
| a security concern leaves the log excerpt unticked | `log_excerpt: true` | 2 of 2 |
| a security concern goes to the security address and not to support | `SUPPORT_ADDRESS` for `Security` | 2 of 2 |
| an address in the log excerpt is masked before it leaves | `mask_email` taken out of the loop | 3 of 3 |
| a running screen reader is named by its process | the name match `&& false` | 2 of 2 |
| a security concern is offered the private reporting page and not the issue page | `github_page: ...ISSUE_PAGE` | 1 of 1, lib |
| a feedback report is queued through the one function every message uses | a second row literal and queueing in `send_the_report` | 1 of 1, the target; the companion judges its plant by the delta and stayed green |

Seven existing records re-measured, each exact: the three About records (their test file's readings were edited), "a window the program can open for the scan is one the workflow asks for" (its anchor line gained `'feedback'`, rewritten and dated in the record's comment), "the composer's Send hands the queue the hold it worked out" (its anchor moved into `put_in_the_outbox`), and the two records reading the outbox census. 1,054 records by the TOML reader; the sweep header reads 257 since.

## Commits and what each hook run printed

| Commit | What | Mode | Stage line (seconds) |
|---|---|---|---|
| `bd3b20be` | red, task 1 | red | 191: start 6, rustfmt 2, clippy 60, the scripts that decide what runs 11, the tests this commit says must fail 0, the tests that reach what changed 112 |
| `1bd764c3` | green, task 1, five records | affected | 129: start 3, rustfmt 3, clippy 29, scripts 33, the tests that reach what changed 61 |
| `7d57ba23` | red, task 2 (a first attempt was refused after 149 s, above) | red | 131: start 6, rustfmt 3, clippy 28, scripts 12, must fail 0, reach 82 |
| `c8aec5f3` | green, task 2, the wiring, the pages the key needs, the changelog (a first attempt was refused after 367 s by the outbox census, below) | all, because the workflow changed | 441: start 5, rustfmt 3, clippy 27, scripts 69, security advisories 5, tests 249, release build 83 |

| `d2f356f8` | the pages, the ledger, this summary, the four marks (a first attempt was refused after 83 s, deviation 11) | docs_only | 83: start 3, rustfmt 3, clippy 28, scripts 0, the targets that read documents 49 |
| `6562e753` | the merge into `main`, `git merge --no-ff` | "mode all, a merge into main", because the diff holds the workflow | 436: start 1, rustfmt 3, clippy 27, scripts 69, security advisories 5, tests 249, release build 82 |

The last row was added after the merge, by a commit on `main` touching only this file.

## The pull request's runs

Pull request #96, read before the merge:

| Run | Verdict |
|---|---|
| CI | Rustfmt, Clippy, Security Audit, Build (debug), Build (release), Setup Executable passed; Test Suite passed in 19m48s, the whole suite over the branch on the runner |
| NVDA screen reader tests | passed, 24m50s |
| Accessibility scan | passed, 10m51s. Feedback: Axe.Windows "0 errors were found", "Walked 2 window(s): 'Send Feedback', 'Wixen Mail'", "MSAA walk: 2244 elements, 1320 of them operated, 0 without a name". About: "0 errors were found", "MSAA walk: 1914 elements, 1124 of them operated, 0 without a name" |
| Tests that would notice | failed after 29m39s in the unmutated baseline, `test_the_share_of_history_before_red_green_is_computed_and_printed`, before any mutant was tested: ledger 458's shape, not this change's |

What the scan's files say about the new controls, read from the downloaded artefact. Over UI Automation the dialog `Send Feedback` holds, in order: text and combo box "What is this about"; the two questions as text and document; text "What to include"; the five check boxes named by their sentences; text and edit "How to reach you"; text and document "What will be sent"; buttons Send, Copy to clipboard, Open the GitHub issue page, Cancel; then the text of the sentence under the buttons ("No account is set up to send from...", since the runner's profile has no account). The multi-line fields answer document rather than edit on this channel. One oddity: the resize grip answers `thumb` and takes the sentence above it as its name, which UI Automation borrowed from the nearest text; nothing reaches it by key. Over MSAA the same controls answer combo box, editable text, check box and push button with the same names. About over UI Automation: the four texts, `pane 'wixen.app'`, `pane 'wixen.app/support'`, `button 'Send Feedback...'`, `button 'OK'`; over MSAA, Send Feedback answers push button named "Send Feedback...". Whether NVDA reads the resize grip's borrowed name anywhere is part of ledger 589.

## Deviations from Plan

**1. [Rule 3 - Blocking] `docs/installing.md` edited, which the plan did not list.** `paths.rs`'s check that every path is named on both pages that list what is stored went red once `feedback_dir` existed; both pages now list `logs\feedback\`. Commit `1bd764c3`.

**2. [Rule 1 - Bug] The outbox census moved from 5 to 6.** `tests/nothing_leaves_the_outbox_unasked.rs` refused the green: Send in Send Feedback flushes the Outbox when `when_it_goes` says now, like the composer's Send. It is a button a person presses after reading the message, so it is written into the list as the sixth, and the two records reading that census were re-measured. Commit `c8aec5f3`.

**3. [Rule 3 - Blocking] Two readers moved with the extraction.** `tests/wired.rs`'s readings of the hold and the chosen time read `fn queue_for_sending(`; the row is built in `put_in_the_outbox` now, so their anchor moved there and their assertions are unchanged. Commit `c8aec5f3`.

**4. [Rule 2] `open_one_of_our_pages` takes the dialog's title**, so the GitHub page's failure box says Send Feedback rather than About Wixen Mail.

**5. The kept files' stamp is `2026-09-23-184512`**, not `20260923-184512`, because house_style keeps the compact day format to the calendar writer.

**6. "Yesterday's when today's is short" is read as missing or empty.** A log that is short because the program started a little while ago is the log the problem is in.

**7. `Include::sends()` is `Fact::sends()`**: the boxes are built from `Fact::ALL`, one sentence per fact.

**8. Acceptance criterion `grep -c 'Ctrl+Shift+F' src/presentation/wx_app.rs` reports 2, not 1.** One is the Help item; the other is the comment beside `ID_SEND_FEEDBACK` naming the key. The key is bound once.

**9. The target's focusable filter skips the resize grip** a resizable dialog carries (role 4), which no key reaches. That is the reading's anchor, not its claim, and the commit says so.

**10. `test_the_one_path_reading_refuses_a_second_queued_row` plants by adding a row and judges the delta**, rewritten in the green so the recorded break that adds a row for real leaves it green.

**11. The pages say "the end of the log", and privacy.md's Logging section alone gives the number.** The documents commit's first attempt was refused by `every_number_carries_its_command_and_its_date`: "200 lines" on four lines of three pages carried no date and no source. It is a constant, not a measurement, so the guide, the alpha page and the privacy table now say "the end of the log", and the Logging paragraph gives 200 with `feedback_report::EXCERPT_LINES` and the date.

**Total:** 3 auto-fixed (1 Rule 1, 2 Rule 3), 1 Rule 2, 6 recorded differences from the plan's letter. No crate or feature added to `Cargo.toml`: `grep -c '^\s*"Win32_' Cargo.toml` is 8, as before (T-12-SC).

## Tests

- `cargo test --lib application::feedback_report::`: 22 passed.
- `cargo test --lib service::this_machine::`: 6 passed.
- `cargo test --lib common::paths::`: 22 passed (the plan's 19 was a different count; the file's count did not change).
- `cargo test --lib presentation::wx_feedback::`: 12 passed.
- `cargo test --test the_feedback_dialog_shows_what_it_sends_before_it_goes`: 10 passed.
- `cargo test --test the_about_dialog_names_its_owners_and_its_links`: 8 passed, 8 before and after.
- `cargo test --lib presentation::scan_target::`: 11 passed; `presentation::wx_app::`: 199 passed, 199 before and after.
- The green of task 2 ran in mode all: every target and the release build passed.

`grep -rl 'security@wixen.app' src tests` lists `src/application/feedback_report.rs` alone.

## The alpha page's new paragraph, whole

> On 2026-09-23: until public testing begins, support@wixen.app and security@wixen.app may not exist yet. A report sent to either shows as sent in the Outbox and then comes back to your own inbox as undeliverable. Nothing is lost: the copy in the `feedback` folder holds the whole report, and Copy to clipboard and the GitHub page take it the rest of the way.

`grep -c -i 'undeliverable\|bounce' docs/ALPHA_TESTING.md` was 0 before the edit and is 1 after.

## Threat model

T-12-15 to T-12-18 hold as planned, each with a record: the redaction, the payload equal to what is queued, one row function and the gate, the security address and page. T-12-19 and T-12-20 accepted as planned. One surface not in the model: the kept copy under `logs\feedback` holds the whole report, the person's own words, on their own machine; the privacy page says so and that nothing else reads it.

## Known Stubs

None. Every function added has a caller on a path from Help, `Ctrl+Shift+F` or About.

## Ledger

589 (unrun-verify, the ear), 590 (unrun-verify, a report arriving at each address, Pratik's), 591 (todo, the Help menu's C and U each held twice, which this plan did not move). 591 entries, 549 open.

## Self-Check: PASSED

Files: `src/application/feedback_report.rs`, `src/service/this_machine.rs`, `src/presentation/wx_feedback.rs`, `tests/the_feedback_dialog_shows_what_it_sends_before_it_goes.rs` and this summary exist. Commits `bd3b20be`, `1bd764c3`, `7d57ba23` and `c8aec5f3` are on the branch (`git log --oneline main..HEAD`). The ledger's two halves carry 589 to 591 and its front matter reads 549 open of 591.

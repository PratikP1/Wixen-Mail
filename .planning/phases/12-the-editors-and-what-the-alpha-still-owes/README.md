# Phase 12: The editors, and what the alpha still owes

Thirteen plans, one per wave. Twelve at first, two moved in from phase 11 and ten written on
2026-09-20 against `main` at `0ad66e48`, version `1.0.0-alpha.1`,
`guards/guards.toml` holding 1,029 records by the TOML reader (the census
lines at `guards.toml:83-84` say 798 swept and 231 since), `.planning/WINDOWS.md`
at entry 566 with 531 open and 35 fixed, 8,651 tests on the last whole gate
(11-12's, on its branch at `f38140a8`), `main` pushed at `0ad66e48`
(`git rev-list origin/main..HEAD --count` was 0 there and is 1 from
`19ad8648`, the commit that landed these plans, until Pratik pushes) and
the push's NVDA run red on one case. The thirteenth, 12-02.1, was inserted
at wave 3 on 2026-09-22 after the milestone's one guard sweep was read
back: it carries FOUND-21 and builds a per-record wall-clock limit before
it corrects anything, so a record that does not return costs one record
and not a shard. Inserting it moved 12-03 and every plan after it down one
wave, 12-12 ending at wave 13, which is not cosmetic here: 12-02.1 and
12-03 both write `guards/guards.toml`, and one plan per wave is how this
phase keeps two plans off one file.
Phase 11 closed on 2026-09-20 with twenty-nine of its thirty-one plans
merged and two deferred to the front of this phase on Pratik's decision
under his token budget. This is the fifth of the seven groups he agreed on
2026-09-16, the editors, with three things in front of it (the red run
and the two deferred plans) and two things after it that the public alpha
owes (About and Send Feedback), plus the pro licence as a design.

**Checked the same day against `19ad8648`** (`PLAN-CHECK.md` beside this
file, the checker's record): three blockers and sixteen warnings, all
applied in the commit after it. The blockers: 12-08's field could not
land green without the form-open read, because the read-by guard's
census excludes `config.rs` and `wx_settings.rs`, so task 1 carries the
read and names both guards by module path; 12-07 listed eight of the
seventeen files the three `ContactEntry` fields touch and did not name
the merge-chain record the insertion splits, so the nine files, their
filters, the place in the merge literal and the record pair are in the
plan now; 12-03 rewrites a site a record anchors on ("mark as read acts
on every selected message, not the cursor row alone", `wx_app.rs:10801`)
and now names it as rewritten and re-measured, with the probe over the
register for every literal the census lists. The warnings were line
numbers, names and counts, each taken again by the command the report
gives; the two refusals the 12-03 re-take missed ("Nothing is selected
in the message list" at `:17887`, "Choose a folder first" at `:5072`)
add a fourth kind, a folder, to the status module.

**Goal.** The three editors the tester named work by keyboard the way he
asked: every number a spin control whose typing field has a name; the
contact editor with a whole name and its parts filling each other, a
birthday as a date, an address and a number checked; event times moving in
blocks; signatures assignable per account with one default; labels the
person can see, make and order, with the menu saying what the keys do.
Before the public alpha, About names its owners and its pages and Send
Feedback sends a report the person has read, from their own account, with
the log attached by default. Behind it, CI is green again, the separate
window a link can open in exists, and the status bar's sentences read to
one shape. The pro licence is a document with Pratik's decisions listed,
and nothing is gated.

**Requirements:** `FOUND-20` under phase 9's section beside FOUND-19 (a
defect in what CI runs, on 11-01's and 11-02's reasoning); `LIST-19` and
`LIST-11` from phase 11, whose remaining plans moved here with their ids;
`ALPHA-01` to `ALPHA-03` and `EDIT-01` to `EDIT-05` in a section of their
own in `.planning/REQUIREMENTS.md`, one per issue.

**Roadmap success criteria this phase owns:** all twelve.

## Pratik's order, and which part of it this is

The seven groups of 2026-09-16, from phase 9's README, with what has moved
since. Groups 1 to 4 were phases 9 to 11. This phase is group 5 with what
sits in front of it and behind it. Phases 13 and 14 are groups 6 and 7,
written as roadmap entries and requirements on 2026-09-20 and not planned;
the order inside each is the planner's for him to confirm.

| Group | What it is | Issues | Where |
|---|---|---|---|
| 1 | The version becomes `1.0.0-alpha.1` | #46 | phase 9, done |
| 2 | The cause-known defects | #21, #32, #36, #39, #42 with #40 point 5, #44, #51, #53, #56, #33, #34 | phase 9, done; #40, #42 and #53 advanced and open |
| 3 | All the mail, and what is said while it comes | #20, #23, #24, #37, #38 | phase 10, done |
| 4 | Reading, and the list | #70, #71, #25, #27, #30, #31, #26, #62, #28 with #29, and seventeen inserted | phase 11, closed 2026-09-20; #80's second half and #75 moved here |
| 5 | The editors | #35 with #73, #40 points 1 to 4, #41, #43, #48 | this phase, 12-06 to 12-10 |
| 6 | New features, most from the Outlook gap audit | #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61; #53's points 4 to 6 | phase 13, roadmap entry and requirements only |
| 7 | The real-account issues, which need Pratik's account | #22, #63 | phase 14, roadmap entry and requirements only; sending proven 2026-09-18, the move, copy and delete proofs to be re-taken after 11-07.1 and 11-07.2 |

Outside the groups and in this phase: the NVDA run 35520201976 (12-01);
#80's second half and #75 (12-02, 12-03); #78 with #64 and #71's third
point, the "before the public alpha" set (12-04, 12-05); #65 as a design
(12-11). #72 stays held by Pratik's decision of 2026-09-18. #74 and #93
carry `version 2` on his word and belong to no phase here.

## The plans

| Plan | Wave | Criterion | Issues | Closes or advances | What it does |
|---|---|---|---|---|---|
| 12-01 | 1 | 1 | none; NVDA run 35520201976 | closes the run's failure structurally; the run is the next push | the case's second half diagnosed from the run's own record (the browser opened, the page stayed, the activation call's answer was the only evidence); the product half measured on a built page window (focus after re-activation) and fixed only if red; the case comes back with Alt+Tab, waits for the foreground and records what has focus before K; FOUND-20 |
| 12-02 | 2 | 2 | #80, second half | closes; ledger 566 fixed | 11-11.2 moved: the separate window as a process of its own started with `--show-page`, answered before the claim and the handover, its WebView2 profile of its own by the app name set before the WebView, the route reaching it, the erase reaching the profile; the two "next build" sentences retired; LIST-19 ticked |
| 12-02.1 | 3 | none; sweep 35520204784 | none | closes the phase 11 sweep; FOUND-21 | the per-record wall-clock limit in `scripts/guards.py` first, so a record that does not return costs one record and not a shard; then the six weak records and the five unmeasurable ones measured by hand and diagnosed, sixteen red lists corrected and re-measured, shard 40's nine measured here, the sweep's four rows on the measurements page, the ledger recording what ended shard 40 as fixed |
| 12-03 | 4 | 3 | #75 | closes | 11-13 moved: every status sentence listed from the code and rewritten by hand to one shape, the refusals one per kind, a reading over the words and endings, no line moved between channels; after 12-02 so its two sentences are in the pass; LIST-11 ticked |
| 12-04 | 5 | 4 (but its Send Feedback clause) | #78 | advances to its last point | the copyright line naming Pratik Patel and the Wixen Project with other contributors under MIT, LICENSE in step and held by a reading; the full version kept; wixen.app and wixen.app/support as controls named by their address, the kind chosen by a reading over MSAA; no dead button |
| 12-05 | 6 | 5, and 4's last clause | #64, #71 third point, #78 point 4 | closes all three | Help, Send Feedback: six categories, the questions that fit, the includes with the log excerpt ticked by default and redacted, the reply address, the exact payload shown before Send; an email from the default account to support@wixen.app through the one queued-row path and the one gate; no account or forbidden said with the clipboard and the GitHub page as doors; security to the private page only; About's button; the scan target; the privacy page's row |
| 12-06 | 7 | 6 | #73, #35 | closes both; twelve ledger entries fixed | `name_the_spin_control` naming the typing field through the annotation service on every spin control in the tree, a reading over MSAA on four built dialogs; the check interval 1..=60, Font size 8..=72, Default reminder 0..=1440 as spin controls; Mark read after a three-way choice with a seconds spin |
| 12-07 | 8 | 7 | #40 points 1 to 4 | closes | prefix, middle name and suffix as columns, fields and each provider's own field both ways; `contact_names::guess_parts` and `compose` with the Hopper and van der Berg cases; the fills each way that never overwrite a typed field; the birthday as the three-control date with a no-year position; an address checked by 11-10.1's rule, a number kept as typed with a country beside it; the Favourite box re-read over MSAA |
| 12-08 | 9 | 8 | #41 | closes | `time_blocks` pure at every boundary; the setting on the Calendar and PIM tab, 30 by default, read on use; Up and Down by the block and Left and Right by a minute on the minute control in all three editors, measured first where the key arrives; a new item on the next boundary with its end one block later, the end following the start until edited |
| 12-09 | 10 | 9 | #43 | closes | signatures as one set with an assignment table and a once-only pass that keeps every account's default; the manager over the set with a Used by column and one Default box; the choice on the account's own dialog; compose taking the From account's signature and swapping the block on a From change only when untouched |
| 12-10 | 11 | 10 | #48 | closes | a position column with a pass numbering rows in the order they were shown; the submenu rebuilt from the account's labels with their keys and Edit Labels at its end; the manager with Move Up and Down through the reordering rule and a Key column; one word, Label; the disagreement between the menu and the keys fixed and named |
| 12-11 | 12 | 11 | #65 | advances to the decisions | the design under `docs/plans/`: what exists, the free and pro line as a table, the offline key, the seam, the lapse, the prices as decided, the merchant table carried whole, the decisions table for Pratik; nothing in the product |
| 12-12 | 13 | 12 | all eleven | closes the phase | the pages, the listening lines from item 84, the closing read over the coverage blocks in one pass, the four planning files told |

Requirement coverage: FOUND-20 by 12-01; LIST-19 by 12-02 (with 11-11.1);
LIST-11 by 12-03; ALPHA-01 by 12-04; ALPHA-02 by 12-05; EDIT-01 by 12-06;
EDIT-02 by 12-07; EDIT-03 by 12-08; EDIT-04 by 12-09; EDIT-05 by 12-10;
ALPHA-03 by 12-11; 12-12 reads all eleven.

Each plan ends with the `gh issue close` or `gh issue comment` the executor
runs after the merge, quoting the merge commit. Closing an issue is not a
publish and the executor may do it; filing or editing other issues is not
theirs. 12-01 comments on #80 and closes nothing; 12-11 comments on #65 and
leaves it open for Pratik's answers.

## Why twelve plans, and why this order

Two came from phase 11 with their numbers changed and their premises
re-taken. The seven issues of the group and the two of the alpha set fall
into eight pieces that share files only through `wx_app.rs`, the changelog
and the records file, and #78 with #64 became two plans because Pratik's
decision on #64 of 2026-09-18 is that About's Send Feedback button lands
in the same commit as the dialog it opens, so About's other four points
can land first and small while the dialog, the phase's largest plan, lands
after; that is the one insert the brief allowed, and the reason is his
decision rather than the planner's convenience. One per wave because every
plan writes `.planning/WINDOWS.md`, every plan but 12-11 writes
`docs/changelog.md`, all but 12-11 and 12-12 write `guards/guards.toml`,
seven of the twelve write `src/presentation/wx_app.rs`, and a wave is a
set of plans sharing no file: the rule in `CLAUDE.md` is that the files the project writes by
rule are added to every plan's list before two lists are compared, and
with them added no two plans here are disjoint. The order:

- **The red run first** (12-01), because every later merge would land on
  a red CI, and nothing this phase adds could be heard on the runner
  over a case that fails for the harness's reason.
- **The separate window second** (12-02), the first of the two deferred
  plans, because 11-12 left the program promising it with the next build
  (ledger 566) and a build cut before it lands carries that promise.
- **The status sentences third** (12-03), the second deferred plan, after
  12-02 so the pass reads that plan's two new sentences, and before
  every plan that adds a sentence of its own is asked to keep the
  shape the pass writes down.
- **About fourth** (12-04), small, before the public alpha, and the
  surface 12-05's button sits on.
- **Send Feedback fifth** (12-05), the largest plan, closing #64, #71
  and #78 together, before the editors so the tester's reports on the
  editors can come through it.
- **The spin controls sixth** (12-06), the first of the group, because
  the helper it writes names every spinner in the tree and 12-07 and
  12-08 add spinners that go through it.
- **The contact editor seventh** (12-07), the group's largest, on the
  helper.
- **Event times eighth** (12-08), on the same item form 12-07 changed
  for the birthday, so the two changes to `wx_item_form.rs` are
  sequential.
- **Signatures ninth** (12-09) and **labels tenth** (12-10), each a
  once-only pass under a marker on the pattern 11-09.2 and 11-08.1
  wrote, signatures first because its pass is the model for the
  labels'.
- **The licence design eleventh** (12-11), documents only, after every
  feature it might one day gate has landed in the shape it will gate.
- **The documents last** (12-12).

## Decisions made here, each overrulable with a reason in a summary

1. **The NVDA case comes back the way a person does**, Alt+Tab, and waits
   for Windows to say the page window is in front, rather than trusting
   `AppActivate`'s answer; the product half (focus in the document after
   re-activation) is measured on a built window and fixed only if the
   measurement is red. Both causes are named in 12-01 and the read that
   tells them apart is the record the case now writes.
2. **The two moved plans keep their tasks and gain nothing but what the
   tree changed under them**, re-taken by command on 2026-09-20 and
   written under a heading above the original readings, which stay
   because they name the mechanisms; 12-02 gains the retirement of the
   two promise sentences (ledger 566), since it is the plan that makes
   them true.
3. **#78 is two plans** (12-04, 12-05), on Pratik's decision that the
   button arrives with the dialog; About's other points are before the
   public alpha and small, and the dialog is the phase's largest plan.
4. **The About links' control kind is chosen by a reading over MSAA**, a
   `HyperlinkCtrl` if it answers a link or push-button role with the
   address as its name, else a `Button` labelled with the address,
   because nothing in this tree has used the hyperlink control and what
   NVDA hears for it is unmeasured.
5. **The feedback report is one queued-row path with the composer**, the
   row composition extracted from `queue_for_sending` into one function
   both call, so the gate every message passes is the gate a report
   passes; a second `QueuedOutboxMessage` literal in `wx_app.rs` is a red
   reading.
6. **The security category sends nothing from the program** and offers
   GitHub's private vulnerability reporting page, which the repository
   has switched on (`gh api repos/PratikP1/Wixen-Mail/private-vulnerability-reporting`
   answers enabled on 2026-09-20); no security address exists and none
   is invented.
7. **The spinner's typing field is named through the annotation service**
   (`IAccPropServices::SetHwndPropStr` on the buddy from `UDM_GETBUDDY`),
   the route ledger 408 wrote down, over the visible-label route, because
   424 measured that a static label names the field on UI Automation
   alone, the channel NVDA does not read for an edit; three features of
   the `windows` crate already in the tree are switched on for it, and
   no crate is added.
8. **Mark read after is a three-way choice with a seconds spin**
   (Immediately, After a number of seconds, Never), the stored string
   unchanged in shape, because a spin alone cannot say Never and the
   issue offered either shape; the port fields stay text, since a port
   is typed and not stepped.
9. **#40's point 4 is built as the issue body shaped it**: an address
   checked by 11-10.1's recogniser with the sentence naming the address;
   a number kept as typed, refused only for letters or no digit, with a
   country chosen beside it from a full calling-code table in the tree;
   no phone-number library, because that is a new dependency and the
   tester's words were a question, listed below for him.
10. **The check box the brief calls #40's point 6 is the Favourite box
    fixed in `165fd811`**, named outright since 2026-09-16; 12-07 reads
    its MSAA name on the built dialog so the record carries a
    measurement, and adds no work for it.
11. **A new event starts at the next boundary strictly after now**
    (14:30:00 with a 30-minute block gives 15:00), on Pratik's first
    answer of 2026-09-15 read literally, with the case saying so; the
    same block, setting and keys apply to tasks and reminders, his
    fourth answer.
12. **Where the key arrives on a spin control is measured before the
    handler is bound** (the `SpinCtrl` or its buddy), the way 11-06
    measured M on a real list; the summary says which.
13. **Each account's signature is chosen on the account's own dialog**
    and shown in the manager's Used by column, on the settings rule that
    a fact about one account is looked for where the account is; the
    issue offered either place "whichever reads better by ear", which is
    the tester's to say.
14. **The signatures pass writes an assignment for every account that
    had a default before it clears any**, so nothing an account had
    changes, and one default remains, the oldest; the version does not
    move for the additive schema, because no build has been cut since
    `1.0.0-alpha.1` and the rule in `CLAUDE.md` moves the counter only
    after a cut.
15. **Labels are one word, Label**, on the tester's word and the
    shortcuts page's; the `tags` table and the `Tag` type keep their
    names because a column and a type are not read by a person.
16. **The labels pass numbers rows in name order**, the order
    `get_tags_for_account` returned before the column existed, so an
    installation keeps the order it was shown; the keys and the menu
    then agree, which they do not today (below).
17. **The pro licence design goes under `docs/plans/`**, where the tree
    keeps designs dated in the name, with a row in the backlog page; the
    brief's `docs/development/` was offered with that alternative.
18. **12-12 counts the ticks in the roadmap's plan list against the
    progress row**, the check `CLAUDE.md`'s completion-marks paragraph
    wants and does not have, by hand, and reports drift.

## What the tree contradicted in the issues and the brief

Every file and line the issues cite was re-run on 2026-09-20 at
`0ad66e48`. The line numbers moved by up to 2,900 in `wx_app.rs` since
the issues were filed; the shapes held except these:

| Source | It says | The tree says | Command |
|---|---|---|---|
| the run 35520201976 | the case never heard the second link | the first half held: the browser opened Example Domain and the page window kept its title and message; the second `K` after `AppActivate` produced one empty phrase; the record kept only the activation call's answer | `gh run download 35520201976 -n nvda-transcripts`; the JSON's `otherWindowsAfterTheFirstEnter` and `pageWindowPutBackInFront` |
| the tester on #80 (2026-09-18) | a link opens inside the window | under NVDA's Enter on the runner the route sent it to the browser; the earlier veto's failure was 11-11.1's finding and is fixed | the same record |
| 11-11.2's premise 3 | `current_exe` appears nowhere | four sites read the path (help_page.rs:42, default_apps.rs:298, default_apps_registration.rs:80 and :1043); none starts the program | `grep -rn current_exe src --include=*.rs` |
| 11-13's premise 2 | six refusal sites in two kinds | five sites in three wordings for one kind; the two "No conversation selected to delete" sites are gone since 11-07 | `grep -n 'No message selected\|Choose a message first\|No conversation selected' src/presentation/wx_app.rs` |
| #78 | the two pages exist before the alpha | wixen.app and wixen.app/support answer 522 (Cloudflare up, the origin not) on 2026-09-20; the dialog names the addresses and the guide says the site is coming | `curl -s -o /dev/null -w '%{http_code}' -A Mozilla/5.0 --max-time 30 https://wixen.app/` |
| #78 | `build_about_dialog` at `wx_app.rs:23292` | at `:26170`; four static lines, a bridge disclosure, OK; "Copyright 2024-2026 Wixen Mail Contributors" against `LICENSE:3`'s "Copyright (c) 2026 Pratik Patel", so the two disagree in years and holders | `grep -n 'fn build_about_dialog' src/presentation/wx_app.rs`; `sed -n 3p LICENSE` |
| #64 | a security address or GitHub's private reporting | private vulnerability reporting is enabled on the repository; no security address exists | `gh api repos/PratikP1/Wixen-Mail/private-vulnerability-reporting` |
| #64 | the screen reader read from the running processes as the scan does | the scan finds NVDA on the runner in the workflow, not in this code; the tree knows no screen reader by name and reads no Windows build; both are new readers on the hand-declared `extern "system"` pattern | `grep -rn 'nvda.exe\|RtlGetVersion' src --include=*.rs` |
| #35 | Font size at `wx_settings.rs:806`, Default reminder at `:1897`, Mark read after at `:88` and `:1334` | `:1109-1112`, `:2472-2473`, `:1511` and `:1834`; the saves clamp 8..=72 and 1440, which become the controls' ranges | `grep -n 'font_size\|default_reminder\|mark_read_after' src/presentation/wx_settings.rs` |
| #73 | ledger 408 applies | it does, and eleven siblings with it: twelve spinner entries among 408 to 425, all closed by one helper applied to the item form's builders, which Send Later also uses | `awk -F'\|' '$2 >= 405 && $2 <= 426' .planning/WINDOWS.md` |
| #40 | points 1 to 4 and 6 remain | the body numbers five; point 5 is the check box, named outright since `165fd811`; what remains is the four | `sed -n 1300,1312p src/presentation/wx_managers.rs`; Pratik's comment of 2026-09-16 |
| #40 | Google carries the three name parts | the tree's `GoogleName` carries given and family only; the API's resource carries honorificPrefix, middleName and honorificSuffix beside them, so the struct gains three fields | `sed -n 65,95p src/service/google_api.rs` |
| #48 | the stored order is undefined | it is alphabetical (`ORDER BY name`), and the menu shows the built-in five in their own order, so Ctrl+2 says Work and applies Later; the comment at `wx_app.rs:6919` claims the names are rewritten on load and nothing does | `sed -n 72,78p src/data/message_cache/tags.rs`; `grep -n labels_menu src/presentation/wx_app.rs` |
| #43 | `signatures` at `mod.rs:1791` | at `:1830`; the shape held | `grep -n 'CREATE TABLE IF NOT EXISTS signatures' src/data/message_cache/mod.rs` |
| #41 | the minute spinner at `wx_item_form.rs:924-926` | at `:924-926` still; `ask_for` at `:271` is the one door all three editors open through (`managers.rs:1213`, `:2733`) | `grep -n 'wx_item_form::ask_for' src/presentation/managers.rs` |
| the brief | "a document under docs/development/" | designs live under `docs/plans/`, dated in the name; the backlog page under `docs/development/` gains a row | `ls docs/plans/` |
| the brief | phase 11's row stays at 29/31 | with the two plans moved the directory holds 29 plans and 29 summaries, so the row reads 29/29 and the phase line is ticked with the move dated, as 11-12's own sentence said it would be | `ls .planning/phases/11-reading-and-the-list/*-PLAN.md \| wc -l` |

## Costs every plan is written around

**Guard records, by the TOML reader on 2026-09-20, 1,029 in all.** Files
the plans touch, with records naming them in `tests_last_seen` and the
test count those records carry, counted by the count check's own rule
(`grep -cE '^\s*#\[(tokio::)?test\]\s*$'`, an attribute alone on a line,
both spellings): `src/presentation/wx_app.rs` 110 and 199;
`src/application/contacts_sync.rs` 76 and 281; `src/presentation/managers.rs`
52 and 137; `src/data/message_cache/contacts.rs` 36 and 103;
`src/data/message_cache/messages.rs` 28 and 183; `tests/house_style.rs` 27
and 74; `src/presentation/wx_settings.rs` 26 and 0; `tests/wired.rs` 18 and
77; `src/application/mail_sync.rs` 14 and 149; `src/data/message_cache/mod.rs`
12 and 23; `src/data/config.rs` 11 and 68; `src/presentation/wx_managers.rs`
11 and 44; `src/application/due.rs` 10 and 46; `src/presentation/wx_account_manager.rs`
7 and 14; `src/presentation/wx_compose.rs` 7 and 43;
`src/presentation/date_display.rs` 7 and 47; `src/service/google_api.rs` 6
and 37; `tests/a_link_opens_where_the_setting_says.rs` 5 and 20;
`src/application/allowed.rs` 5 and 27; `src/presentation/scan_target.rs` 4
and 11; `src/application/bringing_everything_down.rs` 4 and 23;
`src/service/microsoft_graph.rs` 3 and 57; `src/application/checking_on_a_schedule.rs`
3 and 20; `src/common/version.rs` 3 and 27; `src/presentation/first_run.rs`
3 and 16; `src/main.rs` 2 and 0; `src/data/message_cache/tags.rs` 2 and 10;
`src/application/trying_again.rs` 2 and 9; `src/presentation/status_line.rs`
2 and 3; `src/presentation/command_line.rs` 1 and 24; `src/common/paths.rs`
1 and 19; `src/application/opening_links.rs` 1 and 12;
`src/presentation/page_links.rs` 1 and 7; `src/presentation/wx_item_form.rs`
1 and 14; `src/application/sign_off.rs` 1 and 18; `src/presentation/accessibility/names.rs`
0 and 11; `src/application/tagging.rs` 0 and 12;
`src/data/message_cache/signatures.rs` 0 and 2; `src/presentation/scan_fixtures.rs`
0 and 11; `tests/theme_reach.rs` 0 and 7. The rate is the row on
`docs/development/measurements.md`, and a record whose suite is an
integration target runs in about 16 s. So: **no plan adds or removes a
test in `wx_app.rs`, `contacts_sync.rs`, `managers.rs`, `contacts.rs`,
`config.rs` or `wx_settings.rs`**; their tests that read a changed arm
are rewritten in place, and every plan quotes the count before and after.
New readings go in new integration targets at zero records, each with a
record whose `suite` names it, or in new modules at zero records
(`about`, `feedback_report`, `this_machine`, `wx_feedback`,
`contact_names`, `phone_countries`, `time_blocks`, `signatures`,
`status_sentences`, `page_window`). Where a plan must add a test to a
named file (`paths.rs` 1 record, `wx_item_form.rs` 1, `tags.rs` 2,
`names.rs` 0), the red trailer names the count check bare.

**The known gate-mapper holes**, so each plan says what reaches what:
`guards/guards.toml`, `docs/*.md`, `Cargo.toml`, `Cargo.lock`, `locales/`,
`scripts/*.ps1`, `.github/workflows/*.yml`, `LICENSE` and `nvda-tests/`
map to no scoped target (`docs/*.md` is reached by the document-reading
targets on a documents-only commit and by the whole-tree guards on a
code commit; `guards.toml` by seven `house_style` tests on every commit;
`.github/` answers `all`; `nvda-tests/` runs nowhere local, by its
README, and its JavaScript is checked here only by `node --check` and
`jest --listTests`); `src/main.rs` and `src/presentation/wx_settings.rs`
map to `--lib` filters matching nothing (`wx_settings.rs` is reached
through its coupled targets); `src/application/mod.rs`,
`src/presentation/mod.rs` and `src/service/mod.rs` changing run their
whole layer; `cargo test` takes one `--lib`, and several module paths
are several invocations joined with `&&`, which every `<verify>` here
does; a `<verify>` never ends in `| wc -l`, `|| true` or `| cat`.

**Tools that are broken, and what to do instead.** `gsd-tools roadmap
update-plan-progress` counts a README as a plan: edit `ROADMAP.md` by
hand and read the diff. `gsd-tools windows append` corrupts an entry
holding a backslash and has no edit: write ledger entries by hand, both
halves, and `test_both_halves_of_the_ledger_say_the_same_thing` holds
them. `gsd-tools query commit` cuts the hook off: use `git commit`.
`gsd-tools query estimate-calibration` answers factor 1 with no samples:
the factor below is taken by hand.

**The harness's hazards on this machine, on 2026-09-20.** NVDA is
running for the tester and is not stopped, reconfigured or driven; a
live-window reading in a test process shows a frame for a moment, which
NVDA may speak, and that is what every live-window reading of phase 11
did with the tester's copy and NVDA open. The release binary and the
installed binary are never started; a reading that needs a running
program builds its windows in the test process (the `page` target's
shape) or, for `--show-page`, starts the debug build against a temp
folder pinned with `WIXEN_MAIL_DATA` as 12-02's target does. The
tester's profile is not read: bash and PowerShell started from this
harness see a July copy of `%LOCALAPPDATA%\wixen-mail`, and a fact a plan
needs from the live profile is read through Python, read-only; no plan
here needs one. The two linked worktrees, `wixen-mail-sweep` and
`wixen-mail-mutants`, are not written to; every commit is made from the
primary worktree through the hook, never `--only`, never `--no-verify`.
The MSAA walk script crashes PowerShell on this machine (ledger 390); no
plan runs it, and every MSAA reading goes through
`AccessibleObjectFromWindow` on windows the test built.

**The recurring findings from earlier executors**, carried into every
plan rather than restated per task:

- Trace an absence claim rather than accepting it, with a pattern that
  tolerates a line break; this planning found a comment claiming a menu
  is rewritten on load that nothing rewrites, and an order the issue
  called undefined that is alphabetical.
- A guard record goes stale inside its own plan; run the `--remeasure`
  remedy whenever the count check prints it, detached, and read it
  before committing. A first draft of a record's red list is a
  prediction; the runner's answer is the record. Anchor a text break on
  the longest unique context that still names one place.
- Running `scripts/check.sh affected` by hand runs only the tree guards;
  the hook runs the scoped tests. Never pipe `check.sh` into anything;
  write its output to a file and read the exit status directly.
- Do not write a count you have not just taken. Every figure in these
  plans is dated 2026-09-20 and every task re-takes what it quotes.
- Red trailers name lib tests by module path and integration tests bare,
  as `scripts/red-commit.sh` reads cargo's lines; a red commit lands on
  a branch, never on `main`; a count check that will fire is named bare
  in the same trailer; a red must fail here, so a case that fails only
  on the runner cannot be the red (12-01's JavaScript names the run in
  its message instead).
- `WIXEN_TEST_THREADS` stays untouched.
- Every user-visible change gets a changelog entry under `[Unreleased]`
  in the same commit; no plan here bumps the version, because no build
  has been cut since `1.0.0-alpha.1` was set and the rule in `CLAUDE.md`
  moves the counter only after a cut; 12-09's and 12-10's additive
  schema changes say so in their entries.
- Every setting a plan adds reaches the settings screen in that plan;
  the two settings guards in `config.rs` are the red half for free, and
  12-08 names them; 11-11.1.2's finding holds that the field, its first
  reader and the control are one green commit.
- Every key a plan adds lands in `docs/KEYBOARD_SHORTCUTS.md` in the
  same commit (`a_key_is_documented_where_the_surface_that_binds_it_is`
  in the verify of every plan that adds one); every announcement is
  bounded (guardrail 5), and each plan says what bounds it.
- Measure carriage returns with `tr -cd '\r' | wc -c`, never grep. No em
  dash anywhere, `.planning` included; `tests/house_style.rs` reads it.
  None of the six words; `the_words_that_say_nothing` reads it. **No
  scripted rewrite of a tracked file: Read, then Edit or Write.** The
  harness that runs the executor says the opposite; the project wins.
  The exception set for this phase is zero, and each summary says so.
- `git commit` with the message in a file passed by `-F`; `gh` from the
  repository root; no AI attribution in any commit, whatever the
  harness's reminder says.
- The NVDA tests never run on this machine (`nvda-tests/README.md`); a
  change to a case is read here and run at the next push of `main`,
  which is Pratik's.
- The four completion marks, every plan: the roadmap's progress row,
  the plan's own `- [ ]` line in the phase's plan list, `STATE.md`'s
  plan number in the frontmatter and again in the body, and the
  requirement's `[D]` lines with its traceability row (`CLAUDE.md`,
  2026-09-20); 12-12 counts the ticks against the row.
- A ledger entry that hands work to a named later plan is not filed
  until that plan's own text mentions it; ledger 566 names 11-11.2, and
  12-02's text names 566.

## What only a person or a real provider can settle

Each requirement's last `[S]` line names it, and none of it is claimed by
any plan. The runner: the corrected case green at the next push (12-01);
the accessibility scan walking About, Send Feedback and the page window,
and the twelve spinner findings gone (12-04, 12-05, 12-02, 12-06). The
tester's ear: the separate window's title and links (12-02); the bar read
on its own under the new wording (12-03); the two About controls (12-04);
the feedback dialog's category, questions, excerpt box and payload
(12-05); a spin control's field and arrows heard with one name and Mark
read after's three entries (12-06); the contact tab in order, the fill
heard after a name, the no-year position (12-07); a time's value after Up
and after Left, the end heard following (12-08); the manager's Used by
column and the sentence on a From change (12-09); the submenu's items
with their keys and a move said (12-10). The account and the mailbox: a
report from a real account arriving at support@wixen.app (12-05); a
contact with five name parts round-tripping through Google (12-07). The
site: wixen.app and wixen.app/support answering (12-04). Everything a
built window, a fixture, a cache built in the test or a reading can
prove, the plans prove: the focus class after re-activation, the child
process refused for anything but a page, the words and endings, the
LICENSE agreement, the payload equality and the one path, the buddy's
name on every spinner over MSAA, the fills and the flags, the block
arithmetic at every boundary, the resolution and the passes, the menu's
items after a rename and a move.

## Decisions for Pratik

Listed here rather than decided, each with what the plans do meanwhile.

1. **The site.** wixen.app and wixen.app/support answered 522 on
   2026-09-20. 12-04 names the addresses and the guide says the site is
   coming; whether both pages are up before the public alpha, and what
   the support page says, is his (a `todo` on the ledger).
2. **The support mailbox.** support@wixen.app must exist and be read;
   12-05 sends there on his decision of 2026-09-18 and the payload box
   names it, and the ledger carries "a report from a real account
   arriving" as his.
3. **A security address.** None exists; 12-05 offers GitHub's private
   reporting page, which is switched on. Whether a security address is
   wanted beside it is his.
4. **The feedback key.** `Ctrl+Shift+F` is free and 12-05 takes it; a
   different key is a one-line change and the shortcuts row.
5. **Priority for pro subscribers in a report** waits on #65's answers,
   by his own words on #64.
6. **#40's point 4, the phone check.** 12-07 keeps a number as typed,
   refuses only letters or no digit, and offers a country beside it from
   a full calling-code table in the tree; whether a phone-number library
   (a new dependency, which needs the package gate) is wanted for more
   than that is his.
7. **Whether #40's remaining points are the four** the issue body
   numbers, the check box being the fifth and fixed; the brief and the
   phase 9 table say "1 to 4 and 6".
8. **Where an account's signature is chosen** (12-09 puts the choice on
   the account's own dialog and shows it in the manager); the issue
   offered either "whichever reads better by ear", which is his ear.
9. **The pro licence's decisions**, the table in 12-11's document: the
   free and pro line; whether several accounts are gated at all; the
   merchant (Paddle recommended, undecided); online revocation; how long
   a perpetual licence carries updates; a trial from first run with no
   card; whether the supporter tier delivers a real licence; how
   priority support is carried.
10. **Mark read after's shape** (12-06: a three-way choice with a
    seconds spin) and the ports staying typed; both overrulable.
11. **The order inside phases 13 and 14**, the planner's, for him to
    confirm before either is planned; the grouping is his of 2026-09-16.

## Estimates, and the factor behind them

`raw_tokens` is 30,000 per work task, the projection shape phases 6 to 11
used, with 120,000 on the three largest (12-03, 12-05, 12-07) as phase 11
did for its largest. `tokens` is that multiplied by **0.32**, the mean of
`actuals.tokens / estimate.raw_tokens` over the twenty-nine landed plans of
phase 11, read 2026-09-20 from each plan's `estimate` block and each
summary's `actuals` block: 0.076, 0.199, 0.412, 0.234, 0.275, 0.151,
0.221, 0.266, 0.278, 0.168, 0.222, 0.570, 0.379, 0.555, 0.325, 0.342,
0.169, 0.323, 0.256, 0.308, 0.240, 0.374, 0.408, 0.461, 0.289, 0.135,
0.258, 0.184 and 1.150 (mean 0.318; the last is 11-12, whose summary
says its figure is inflated by how the diff counts the planning files,
and over the other twenty-eight the mean is 0.288). The spread is
fifteen times, so `low`, derived from the sample and not self-rated.
Phase 11's README gave 0.43 over phase 10's ten; phase 11 ran under it.
`gsd-tools query estimate-calibration` answers `factor: 1, sample_count:
0` on this project, so the factor is taken by hand from the files and
said here. The two moved plans' `tokens` were re-projected at this
factor from their unchanged `raw_tokens`.

## What is owed to documents, and who does it

1. **The roadmap's phase 12 entry and progress row.** Done by the planner
   in the commit that lands these plans: the goal, the eleven
   requirements, twelve criteria, the plan list, the row at `0/12`, the
   phase 11 row corrected to `29/29` with its line ticked and the move
   dated, phases 13 and 14 as entries with no row (a row needs a
   directory on disk, by `test_the_roadmap_counts_the_files_that_are_on_disk`),
   and the milestone paragraph kept true.
   `test_the_roadmap_counts_the_files_that_are_on_disk` holds the rows to
   the files.
2. **`.planning/REQUIREMENTS.md`.** Done by the planner: the new section,
   FOUND-20 beside FOUND-19, the LIST-11 and LIST-19 lines and rows
   amended with the move, the phase 13 and 14 sections, the traceability
   rows, the coverage count re-taken with the file's own command, the
   provenance notes.
3. **`.planning/STATE.md`.** Done by the planner in the same commit, by
   hand: phase 12 current, plan 0 of 12, `progress.total_plans` and
   `completed_plans` counted from the disk.
4. **`docs/changelog.md`.** Every plan but 12-11 and 12-12 writes its
   entries under `[Unreleased]`; 12-12 reads them as one.
5. **The requirement ticks.** Each plan ticks its own requirement's `[D]`
   lines on its merge with the date and the names (the orchestrator's
   instruction in phase 11, which overruled the README's "the closing
   read ticks"); 12-12 re-reads every name in one pass and stands or
   corrects each tick.
6. **The ledger.** Each plan adds entries by hand for what it could not
   settle, both halves; 12-02 fixes 566; 12-06 fixes twelve.
7. **The pages.** Each plan writes the rows and paragraphs its keys and
   settings owe on `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`,
   `docs/ALPHA_TESTING.md` and `docs/privacy.md` in its own commits;
   12-12 reads the four pages as one and writes the listening lines.

## Where the work stopped on 2026-09-23, and what the next session needs

Pratik paused here to change the model he works with, so this section is
written for whoever picks the phase up rather than for the person who put
it down. **The next plan is 12-04**, the About dialog, and the waves after
it are already renumbered for the 12-02.1 insert.

Four of the thirteen plans are merged.

| Plan | Merge | What it settled |
|---|---|---|
| 12-01 | `e29c514b` | The NVDA case's failure was the case, not the product: the page window gives the keyboard back to the browser on every activation, measured on a built window, so no handler was added. The case now waits for the foreground and records what has focus |
| 12-02 | `70f5435a`, corrected at `cf58f9a4` | The separate window is a second copy of the program holding one page, with a WebView2 profile of its own at `wixen-mail\pages`; #80 closed |
| 12-02.1 | `a503ce77` | The phase 11 sweep's remedy, and the per-record wall-clock limit that keeps a stuck record from costing a shard; FOUND-21 |
| 12-03 | `a9ce329d`, corrected at `434cd972` | 441 status calls read in one pass, 177 sentences rewritten to one shape, fifteen kinds of refusal each with one sentence; #75 closed |

`main` was pushed at `26beb051` on 2026-09-23, on Pratik's word for that
one push. Every push after it is his word again. That push is the first
verdict on 12-01's rewritten NVDA case and on everything phase 12 has
landed; read the runs before trusting the tree's own green.

Three things are owed and none of them lives in the tree yet.

**The decisions listed above under "Decisions for Pratik" are unmade**,
and 12-04 and 12-05 are the first plans that need them: whether
`wixen.app` answers at all (it returned 522 when the plan was written),
whether the support mailbox exists and is read, whether a security
address is wanted beside GitHub's private reporting, and the feedback
key. Ask before building those two, not during.

**Twelve staged skill updates** from the observation review of
2026-09-20 sit uninstalled outside this repository, under the workspace's
`skill-updates/2026-09-20/`, with `tdd` at 1,850 lines owing the split
its own notes propose. They change how an agent works here, not what the
product does, so nothing in this phase waits on them.

**Three executors in a row kept a clean exception set through their plan
and then broke the no-scripted-edits rule while probing**: a script
applying a guard break, a copy restoring a file, a probe test written by
script. 12-03's executor, told about the diagnostic detour by name, came
back at zero. Keep that sentence in every brief.

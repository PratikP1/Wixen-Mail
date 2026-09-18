# Phase 11: Reading, and the list

Twenty-one plans, one per wave: twelve written 2026-09-18 against `main` at
`744d05ef`, version `1.0.0-alpha.1`, `guards/guards.toml` holding 912
records by the TOML reader (census 802 + 110 at `guards.toml:84`),
`.planning/WINDOWS.md` at entry 529 with 499 open, 8,032 tests on the last
whole gate, nothing unpushed (`main` was pushed that morning and the three
workflows ran on it); three inserted later the same day against
`08197657`, after the plan check, for issues filed that afternoon; and five
inserted that evening against `eb5d8517`, with 11-01 to 11-03 merged and
11-04 executing on its branch, for six issues filed that evening, two of
which became a third task of a plan not yet executed; and one inserted
later that evening against `61865f61`, with 11-05 merged and 11-06
executing, for #25 reopened on the tester's word. Phase 10 closed on
2026-09-18 with all ten plans merged. This is the fourth of the seven
groups Pratik agreed on 2026-09-16, with two plans in front of it from what
the morning's push showed.

**Five plans were inserted and two tasks added on the evening of
2026-09-18, against `eb5d8517`, for six issues filed that evening.** #84
(Alt+A for the attachments from inside the document; F8 is the column
chooser's key and the page window's key binding is one WebView2 never
delivers) is 11-04.1, after 11-04 because that plan is executing and
untouched. #83 (a delete spoken twice) is 11-06.1's third task, because
the arm it changes is the one 11-06.1 already rewrites and that plan had
two tasks. #81 (the earcons go silent after hours) is 11-09.1's third task,
because the earcon channel is on by default from that plan's first task
onward and it had two tasks. #82 (a snippet reads a whole address; Pratik's
decision went past the host to the first relevant words by written rules)
is 11-09.2 beside 11-09, which is at three tasks. #80 (where a link opens,
Pratik's ask, with the tester's finding that Enter on a link does not reach
the veto) is two plans: 11-11.1 for the setting, the route, the menu and
the message view, and 11-11.2 for the separate window, which has to be a
process of its own because every WebView in one process shares one WebView2
profile through wxdragon 0.9.17 and nothing can clear it; both after 11-11
because that plan rewrites the privacy page these write beside. #79 (typed
Markdown makes no heading) is 11-11.3 last of the inserts, after a probe on
this machine's runtime reproduced it at neither build and found the guard
refusing a marker on any line after the first. Their requirements are
`LIST-14` to `LIST-19`, the criteria 17 to 22; every plan from 11-05 on
moved to make room, 11-13 and 11-12 last as before.

**One more was inserted later that evening, against `61865f61`, with 11-05
merged and 11-06 executing on its branch.** #25 was reopened by Pratik on
the tester's answer to the question 11-05's close comment asked him:
reading the snippet is not reading. At `5c82f680` the mail read-aloud
closure writes `reading_began` before `SpaceCycle` has decided which form
the press reads, so the first Space, the short form, started the clock.
11-05.1 moves the record to where the depth is known, makes the decision
one function with cases, and runs right after 11-06, which was not
touched; LIST-03's `[D]` line is amended rather than a new id written, and
criterion 3 carries the amendment. The waves from 11-06.1 on moved one.

**Three plans were inserted later on 2026-09-18, after the plan check
(two blockers and ten warnings, applied at the same commit).** 11-06.1 is
#76: deleting a message puts the cursor at the top, because the rule that
lands it on the next message lives in the state and nothing sets the
control's focus to it, and the watch's re-read after every delete replaces
the rows without re-selecting; it goes before 11-07 because that plan
reworks the same function for a set and lands the cursor after the set
through this rule, and 11-07 was at three tasks. 11-09.1 is #77: landing
on a message with an attachment says the word three ways; Pratik's decision
is the earcon on by default, the spoken event off, the column kept; it sits
beside 11-09, which was at three tasks. 11-13 is #75: every status bar
sentence read in one pass and rewritten to one shape; it runs after 11-11
so it reads every sentence this phase adds, and before the closing read,
which then reads the bar's words as that pass left them. Their requirements
are `LIST-11` to `LIST-13`; the plans after each insert moved up a wave.

**Planned from nineteen issues and three runs.** The issues: #70 (the folder
chooser, filed on the second day of testing), #71 (the log level's default,
Pratik's decision of 2026-09-17), and the eight of group 4 from the first
day: #25 (read state on traversal), #26 (column headers on every row), #27
(Mark as Read's label, M, the thread), #28 with #29 (pictures, and the
privacy page), #30 (selecting more than one), #31 (which message a thread
row is), #62 (a rule that changes how a row is announced); and the three
of the third day, #75 (the status sentences), #76 (the cursor after a
delete) and #77 (attachment said once); and the six of that evening, #79
(typed Markdown), #80 (where a link opens), #81 (the earcons after hours),
#82 (the snippet), #83 (a delete said twice) and #84 (Alt+A for the
attachments). Pratik's decisions are in the bodies and comments and are
settled; each plan quotes his words. The runs: CI 35336142985 (red on one test), NVDA 35336142908
(green over a failed job) and Accessibility 35336142914 (the five editors
walked), all on `744d05ef`, read with `gh run view` and quoted in 11-01 and
11-02. Every file and line the issues cite was re-run with a command on
2026-09-18 and quoted in each plan's `<premise_corrections>`; the ones that
moved are in the table below.

**Goal.** The message list and the reader work the way a person working by
ear needs them to: nothing is marked read by moving; a command says which
way it will go and has a letter; more than one message can be chosen and
every command acts on the set with one sentence; a thread row stands for
the message that matters; the row's columns can be heard on request and
the headers on every row are explained; a rule can change what a row says;
pictures show by default with tracking pixels and undescribed pictures
handled by rule; the folder chooser is a tree whose ticks a screen reader
hears; the log's default follows the build; and the privacy page lists
every way a reader of mail can be tracked. Behind it, CI is green and the
NVDA workflow's verdict is the run's.

**Requirements:** `LIST-01` to `LIST-19`, one per issue, in
`.planning/REQUIREMENTS.md` under "Reading, and the list"; `FOUND-17` and
`FOUND-18` for the two plans in front, under phase 9's section beside
FOUND-13 to FOUND-16, on the same reasoning: a regression of 09-02's fix
found by CI, and the workflow 09-06's case runs in.

**Roadmap success criteria this phase owns:** all twenty-two.

## Pratik's order, and which part of it this is

The seven groups of 2026-09-16, from phase 9's README, with what has moved
since. Groups 1 to 3 were phases 9 and 10. This phase is group 4. The next
planner starts from this table and confirms the grouping of 5 to 7 with
Pratik before writing requirements.

| Group | What it is | Issues | Where |
|---|---|---|---|
| 1 | The version becomes `1.0.0-alpha.1` | #46 | phase 9, done |
| 2 | The cause-known defects | #21, #32, #36, #39, #42 with #40 point 5, #44, #51, #53, #56, #33, #34 | phase 9, done; #33 closed 2026-09-18 on the tester's ear; #40, #42 and #53 advanced and open |
| 3 | All the mail, and what is said while it comes | #20, #23, #24, #37, #38 | phase 10, done; #29's one line written there |
| 4 | Reading, and the list | #70 first, then #71, #25, #27, #30, #31, #26, #62, #28 with #29 | this phase, 11-03 to 11-11 |
| 5 | The editors | #35, #40 points 1 to 4 and 6, #41, #43, #48 | a later phase; #62 ties to #48 and leaves labels' order and naming to it |
| 6 | New features, most from the Outlook gap audit | #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61 | a later phase |
| 7 | The real-account issues, which need Pratik's account | #22, #63 | a later phase; the download and the watch have met his Gmail account unasked since the build after phase 10 |

Outside the groups: #64 (the feedback dialog) and #65 wait on a decision
about where a report goes, and #71's third point, the dialog attaching the
log, lands with #64; #33 was waiting on the NVDA run and closed on the
tester's ear instead. **#72, the status bar and F6, filed 2026-09-18, is
held** by Pratik's decision of that day: a screen reader reads a native
status bar with its own key (NVDA+End), so it waits until a user asks; it
carries the label `held` and belongs to no group. Three more filed that
afternoon, #75, #76 and #77, and six that evening, #79 to #84, are this
phase's by insertion, above, and belong to no group either.

## The plans

| Plan | Wave | Criterion | Issues | Closes or advances | What it does |
|---|---|---|---|---|---|
| 11-01 | 1 | 11 | none; CI run 35336142985 | closes the run's failure | the Settings screen keeps a chosen spelling language exactly as chosen when this machine cannot check it, as a rule in a module of its own; the one test that fails on the runner passes there; a regression of 09-02 |
| 11-02 | 2 | 12 | #33 closed already | settles FOUND-08 and FOUND-09 | the NVDA workflow's verdict is the run's; the settings case waits for what the harness can hear; a sign-in failure is one notification carrying the sentence; FOUND-08 ticked on the walk, FOUND-09 on the tester's ear |
| 11-03 | 3 | 1 | #70 | closes | the folder chooser as a tree with a native check state per folder, decided by a reading over MSAA and the control's own state; the account's name; All Mail when listed and a sentence when Gmail hides it; the command on Tools; the pages that said File |
| 11-04 | 4 | 2 | #71 | advances; #64's half stays | the log level's default from the version, Debug under alpha and beta; the lines a report needs at that level; a guard that no line names a secret or a body; the table on the alpha page with the measured cost |
| 11-04.1 | 5 | 18 | #84 | closes | inserted 2026-09-18 (evening): Alt+A reaches the attachments and F7 the warning from inside the document on the reader and the page window, through the page's script rather than a key binding WebView2 never delivers; the dead binding removed; the pages say Alt+A |
| 11-05 | 6 | 3 | #25 | closes | moving through the list marks nothing; reading aloud or opening starts the clock; the rule in `reading_habits`; the sentence under the setting |
| 11-06 | 7 | 4 (label, key) | #27 | advances | the label follows the state on the Action menu, the context menu and the toolbar (relabelled natively); M consumed on a real list; the words as rules |
| 11-05.1 | 8 | 3 (amended) | #25 reopened | closes again | inserted 2026-09-18 (later that evening, after 11-06 which was executing): the first Space, the short form, starts no clock; the whole reading and opening do; which press counts is `read_aloud::what_a_press_starts` over the depth the cycle chose, the record written where the depth is known; the reading from 11-05 gains the two-press case; the sentence under the setting and the guide say reading the whole message or opening it |
| 11-06.1 | 9 | 15, 17 | #76, #83 | closes both | inserted 2026-09-18: where the cursor lands after rows leave, as a rule; the control's focus set from it after a delete or a move; a re-read keeps the cursor by id and moves nothing when nothing moved; a built list proves the four cases; and, added that evening, a delete says Delete once at the key and nothing on success through a shown-only kind, the same for the moves |
| 11-07 | 10 | 5, and 4's thread clause | #30, #27 | closes both | the list selects a set; seven commands act on it with one sentence; a conversation row contributes its messages; the bound shared with Select All and measured; a set's delete lands after the set through 11-06.1's rule |
| 11-08 | 11 | 6 | #31 | closes | a thread row stands for the originator or the first unread message, chosen in SQL where the columns are; the preview and the window follow it; the conversation's text as one chunk on selection |
| 11-09 | 12 | 7 | #26 | closes | Ctrl+Shift+; reads the row's columns with their headings on request; the headers on every row are NVDA's setting, and the page gives the profile steps |
| 11-09.1 | 13 | 16, 19 | #77, #81 | closes both | inserted 2026-09-18: the earcon channel on by default for every event, the attachment event's own default the earcon and the status bar with no speech, the column kept, a stored profile kept, the never-sound-alone rule unable to speak for an on-the-row event; and, added that evening, the player opening the device again on the stream's error flag and after a gap or on Windows' notice by measurement, a failed reopen logged once and said once |
| 11-09.2 | 14 | 20 | #82 | closes | inserted 2026-09-18 (evening): a row's snippet is the message's first relevant words by written rules, one function per rule with a test each, one derivation for the save and the pass, every stored snippet recomputed once under a new marker |
| 11-10 | 15 | 8 | #62 | closes | a rule's phrase said first and shown in a column; a sound once per check through a new event; the labels as a column; the phase's longest plan |
| 11-11 | 16 | 9, 10 | #28, #29 | closes both | pictures shown by default except pixels and decorative ones; the link's words as a description; an undescribed picture by rule with its setting; the privacy page's tracking section |
| 11-11.1 | 17 | 22 (first half) | #80 | advances | inserted 2026-09-18 (evening): a `page` scan target and an NVDA case as the probe; Open links on the Reading tab; the route as one pure function; the page script catching the anchor's activation; the three menu items; the message view route with its title and its way back; the privacy paragraph |
| 11-11.2 | 18 | 22 (second half) | #80 | closes | inserted 2026-09-18 (evening): the separate window as a process of its own started with `--show-page`, answered before the claim and the handover, its WebView2 profile of its own by the app name set before the WebView, the route reaching it, the erase reaching the profile |
| 11-11.3 | 19 | 21 | #79 | closes | inserted 2026-09-18 (evening): a marker counts at the start of any line, a refusal that met a marker is logged, the inline style ends at its delimiter, `- ` on the empty first line measured, a reading types into the real page, the pages say the space in words |
| 11-13 | 20 | 14 | #75 | closes | inserted 2026-09-18: every status sentence listed from the code and rewritten by hand to one shape, the refusals one per kind, a reading over the words and endings, no line moved between channels; after every plan that adds a sentence and before the closing read |
| 11-12 | 21 | 13 | all nineteen and #25's correction | closes the phase | the pages, the listening lines, the closing read |

Requirement coverage: FOUND-17 by 11-01; FOUND-18 by 11-02, which also
ticks FOUND-08 and FOUND-09; LIST-01 by 11-03; LIST-02 by 11-04; LIST-03 by
11-05 and, amended, by 11-05.1; LIST-04 by 11-06 and 11-07; LIST-05 by 11-07; LIST-06 by 11-08;
LIST-07 by 11-09; LIST-08 by 11-10; LIST-09 and LIST-10 by 11-11; LIST-11
by 11-13; LIST-12 and LIST-14 by 11-06.1; LIST-13 and LIST-16 by 11-09.1;
LIST-15 by 11-04.1; LIST-17 by 11-09.2; LIST-18 by 11-11.3; LIST-19 by
11-11.1 and 11-11.2; 11-12 reads all nineteen.

Each plan ends with the `gh issue close` or `gh issue comment` the executor
runs after the merge, quoting the merge commit. Closing an issue is not a
publish and the executor may do it; filing or editing other issues is not
theirs. 11-02 closes nothing: #33 is closed already.

## Why twenty-one plans, and why this order

Twelve at first because two things the morning's push showed go before the
issues, and the ten issues fall into ten pieces that share files only
through `wx_app.rs`, the changelog and the records file; three more that
afternoon for three issues filed that day, each an insert beside the plan
it belongs with because that plan was at three tasks; five more that
evening for six issues, two folded into plans that had two tasks and the
rest inserted, #80 as two because a separate window with a profile of its
own is a process of its own; one more later that evening for #25 reopened,
a correction to a merged plan. One per wave because every plan writes
`docs/changelog.md`, all but the closing read write `guards/guards.toml`,
sixteen of the twenty-one write `src/presentation/wx_app.rs`, and a wave
is a set of plans sharing no file. The order:

- **The red CI first** (11-01), because every later merge lands on it.
- **The NVDA workflow second** (11-02), because its verdict has to reach
  the run before the phase adds anything a case might hear, and the
  sign-in fix is small.
- **#70 third** (11-03), Pratik's word, and it touches the surface 10-05
  and 10-06 read choices from.
- **#71 fourth** (11-04), small and early, so the log carries the phase's
  own lines from here on.
- **#84 beside it** (11-04.1), the attachments key, the smallest of the
  evening's six and the first plan after the one executing.
- **#25 then #27's label and key** (11-05, 11-06), read state and its
  command, the two smallest changes to the list.
- **#25's correction right after** (11-05.1), the first Space starting no
  clock, inserted while 11-06 was executing and so placed behind it.
- **#76 with #83 before the selection** (11-06.1), because the selection's
  delete of a set lands the cursor through the rule this plan writes and
  says Delete the way this plan's third task decides.
- **#30 next** (11-07), the selection, which #27's thread marking and
  #62's set-wide commands rest on, and which every arm after it reads.
- **#31** (11-08), the conversation row, before #26 and #62 read a
  row's cells.
- **#26** (11-09), what a row says on request, which #62 extends.
- **#77 with #81 beside it** (11-09.1), the other duplicate on the row,
  which changes a default 11-10's new event then inherits, and the player
  that must keep playing once every fresh profile hears it.
- **#82 beside that** (11-09.2), what a row's snippet says, before #62's
  last change to what a row says.
- **#62** (11-10), the last change to what a row says.
- **#28 with #29** (11-11), the reader and the privacy page, which reads
  every plan before it for what a reader of mail sends.
- **#80 in two plans after it** (11-11.1, 11-11.2), because both write
  beside the privacy section 11-11 rewrites, and the separate window is a
  process of its own.
- **#79 after them** (11-11.3), the composer, which shares no file with the
  list plans but the pages and the records.
- **#75 after every plan that adds a sentence** (11-13), so the pass reads
  the phase's own sentences too.
- **The documents last** (11-12).

## Decisions made here, each overrulable with a reason in a summary

1. **The two inserts take `FOUND` ids** (FOUND-17, FOUND-18) under phase 9's
   section, as 10-01.1 to 10-02.2 did, because one is a regression of
   FOUND-02's fix and the other is the workflow FOUND-09's case runs in.
2. **The NVDA job's `continue-on-error` comes off; `accessibility.yml`'s
   stays.** A case that fails fails the run (guardrail 4); the scan's
   findings are counts the workflow already reads out, and whether a count
   fails the run belongs to the phase that owns the findings.
3. **A sign-in failure is one notification** through the event with the
   sentence as its detail, on the channels the person's Feedback row
   allows, rather than a sentence and a cue a millisecond apart.
4. **The folder chooser's check state is the native tree's** (`TVS_CHECKBOXES`
   through `SetWindowLongPtrW`, the state through `TVM_SETITEMW` and
   `TVM_GETITEMSTATE`) when 11-03's probe shows it toggles and reads under
   wx, because that is the object NVDA reads with no object of ours in the
   path; the per-row answering falls back if the probe says no.
5. **#26 takes the documented NVDA setting, in an application-triggered
   profile.** NVDA reads the header before every column but the first
   from its own Row/column headers setting and nothing an application
   sets changes it; the program speaking rows would double them; an add-on
   is later work. The application's part is the on-request key.
6. **The toolbar is relabelled through `TB_SETBUTTONINFOW`**, not made a
   check tool, because Pratik's words are that the label follows the state
   on the toolbar too and wxdragon 0.9.17 offers no relabel.
7. **M is consumed on the list**, not skipped, so the control's
   type-to-search never gets it; shown on a real list.
8. **A command over a selection stops at 5,000**, the bound Select All
   already chose, with one sentence; a conversation row contributes the
   whole conversation to Mark as Read, Star and Label, this folder's
   messages to Move and Copy, and its setting's reach to Delete.
9. **A thread row's message is chosen in SQL** by `read ASC, received_at
   ASC, id ASC`, so the first unread by arrival when any is unread and the
   originator otherwise, including when everything is read; the dates
   stay the newest.
10. **A rule's sound is one event**, `RuleMatched`, played once per check,
    with a per-rule box; a file per rule would be a second sound format.
    The phrase is bounded to 40 characters and prefixed to the first
    visible cell.
11. **A tracking pixel is told by its declared size alone** (a pixel or
    less on either side); no host list, because one goes stale the day it
    is written. A picture with no size is fetched, and the privacy page
    says a tracker shaped like a picture is fetched.
12. **`mark_read_after` keeps its default of two seconds**, counted from
    reading rather than selection; the tester's `never` stays his.
13. **The download's chunk lines are at `debug`**, the rest of the report
    lines at `info`; the default is `debug` under alpha and beta by
    Pratik's decision.
14. **11-08 fixes a defect the issue did not name**: under conversation
    view the preview showed the message at the row's index of the flat
    list, not the row's.
15. **#77's "and braille" is not sent**, because this program has no
    braille route apart from the screen reader notification it also
    speaks (`accessibility.rs:258-266`; the Feedback tab's one switch
    names both), so the attachment event's default is the earcon and the
    status bar; 11-09.1's summary and close comment say so, and one
    channel added to the default set is the overrule.
16. **11-13 rewords by hand and holds only what a reading can hold**: the
    words a person does not use, the endings, and the three refusals; the
    shape "what happened, to what, what next" is a table in the summary
    and not a grammar in a test.
17. **11-06.1 narrows 10-02's no-reselect rule** rather than removing it:
    a re-read moves focus only when the cursor's message changed index,
    so a load that changed nothing under the cursor still moves nothing.
18. **A delete's success is a new `UIUpdate::Shown` kind, shown and never
    spoken**, named quiet on purpose, rather than a flag on `StatusUpdated`:
    the arm test that names every quiet arm is the place a shown-only line
    is declared, and a second sender would be a second place.
19. **#84's keys go through the page's script, not a wx binding**, because
    WebView2 delivers no wx `KEY_DOWN` while the browser has focus; the dead
    binding is removed rather than kept beside the working route.
20. **#81's second mechanism is chosen by a measurement of the device
    open's cost**, under 50 ms the reopen after a gap, at or over it Windows'
    own notice; the callback's flag is written either way, and the summary
    says which and why.
21. **#82 follows Pratik's comment, not the issue's body**: the first
    relevant words by written rules, addresses dropped altogether, each rule
    a function with a test, the least bad line when nothing survives; the
    pass runs once more under a new marker and the old marker's row stays.
22. **#80's separate window is a process of its own**, started by this
    program with `--show-page`, answered before the claim and the handover,
    its profile decided by the app name set before its WebView; the message
    view route shares the preview's profile and the privacy page says so;
    the third choice is offered from 11-11.1 and routed to the browser with
    a status line until 11-11.2 lands, never silently.
23. **#80's activation is caught in the page**, a `click` listener with
    `preventDefault` posting the href, so the route holds whether or not the
    veto fires for NVDA's Enter; the probe is an NVDA case on a `page` scan
    target, run on the workflow, and stays as the regression.
24. **#79's fix asks the line, not the node**: `startsItsLine` in place of
    the sibling guard, a refusal that met a marker logged at debug and never
    announced, the space said in words on the pages; the diagnosis and the
    probe logs are copied to `wixen-mail-sweep/probe_79/` beside the seed
    since the session's scratchpad does not outlive it.
25. **#25's correction amends LIST-03 and criterion 3 rather than adding an
    id**, on the coordinator's word: the requirement's sentence is still
    true and "read aloud" now means the whole reading; the tick stands for
    the rest and 11-05.1's summary dates the amendment. The decision is a
    function over `Depth` in `read_aloud`, not in `reading_habits`, because
    the application layer does not read the presentation layer's cycle.
26. **11-05.1 runs after 11-06, not before**, because 11-06 was executing
    on its branch when the issue was reopened and its files are not
    touched; the plan keeps the `05.1` name the coordinator gave it and
    depends on 11-06.

## What the tree contradicted in the issues and the brief

Every file and line the ten issues cite was re-checked on 2026-09-18 at
`744d05ef`. The line numbers moved by up to sixty since the issues were
filed; the shapes held. These moved in kind:

| Source | It says | The tree says | Command |
|---|---|---|---|
| #70 | the command is on File, at `wx_app.rs:6563-6567` | those lines build the This Folder submenu, which is appended to the Action menu at `:6815`; the command left File on 2026-08-26 and the pages never followed | `grep -n 'append_submenu(' -A 2 src/presentation/wx_app.rs`; `git log -S'"&This Folder"' -- src/presentation/wx_app.rs` |
| #70 | Gmail's All Mail is missing, to be traced | his server did not list it: 50 folders stored, none flagged `holds_all_mail`, no `[Gmail]/All Mail` and no `[Gmail]/Important`; Gmail's Show in IMAP setting hides labels from `LIST`, and this program drops nothing | the Python read in 11-03's premise 3, read-only |
| #70 | a kept folder is heard as read-only and unchecked | the object this program writes answers neither; 11-03 reads what it answers and what a native tree answers before choosing | `sed -n 232,262p src/presentation/accessibility/names.rs` |
| #27 | the toolbar's label follows the state | wxdragon 0.9.17 can set a tool's short help and not its label; the label goes through the native toolbar | `grep -n 'pub fn ' ~/.cargo/registry/src/*/wxdragon-0.9.17/src/widgets/toolbar.rs` |
| #26 | the headers are the screen reader's | confirmed from NVDA's source: `reportTableHeaders` in (rows and columns, columns), every column but the first; and `message_rows.rs:66-73` says "the headings are not being read", written from nobody's ear | NVDA `sysListView32.py`, read 2026-09-18 |
| #25 | previewed means focus entering the reading pane | the pane cannot take focus, by design (`set_can_focus(false)` and the comment); reading aloud with Space and opening with Enter are the acts | `grep -n 'preview.set_can_focus(false)' src/presentation/wx_app.rs` |
| #25 | opening a message is one of the two acts that should mark it | opening marks nothing today either | `awk 'NR>=12447 && NR<=12560' src/presentation/wx_app.rs \| grep -n 'Read('` |
| #31 | the newest message is highlighted | the row's cells are the newest's; the preview under conversation view is `messages[idx]`, an unrelated message, because the selection handler has no conversation branch | `sed -n 3040,3043p src/presentation/wx_app.rs`; `grep -n 'showing_conversations()' src/presentation/wx_app.rs \| awk -F: '$1>3000 && $1<3100'` |
| #28 | none of the pictures in emails are shown | the setting that holds them back is on by default and the tester has turned it off on his profile, so on his machine the change is the pixel rule and the descriptions | the Python read in 11-11's premise 1 |
| #71 | the default is `info` at `config.rs:687` | at `:703`, and a second literal in `LoggerConfig::default()` at `logging.rs:62`; his profile holds `info` now, so his build stays at `info` unless he moves it | `grep -n 'log_level: "info"' src/data/config.rs`; `grep -n 'level: LogLevel::Info' src/common/logging.rs` |
| #71 | `debug` 12 sites, `info` 73 | 12 and 77 after phase 10 | `grep -rn 'tracing::info!' src --include='*.rs' \| wc -l` |
| the brief | 12,872 messages on his profile | 17,753 today, 1,519 of them in his inbox; 12,872 was 2026-09-16's count and the harness's size | the Python read, 2026-09-18 |
| the brief | the CI failure is red here too | it passes here, because this machine offers en-AU; the red half of 11-01 is a unit case over hand-built rows | `cargo test --test the_language_the_screen_shows_is_the_one_used` -> ok |
| the brief | the settings case heard nothing where the dialog speaks | the run at `744d05ef` was the case's first run; no transcript of any case holds a dialog's opening announcement; the case timed out before pressing a key; the tester heard the dialog speak by hand the same day | the four transcripts; `git log -- nvda-tests/tests/settings-tabs-read-once.test.js` |
| the brief | the sign-in case never hears "Signing in failed" | the transcript ends with the event's words, "Sign-in needs attention, Scan target"; the sentence at High and the event at Urgent go out a millisecond apart from the same arm | the transcript; `sed -n 582,590p src/presentation/wx_account_manager.rs` |
| #84 | the page window binds F7 and F8 on the page | it does, at `wx_app.rs:21201`, and WebView2 delivers no wx `KEY_DOWN` while the browser has focus, so the binding is dead from the document; the injected script posts Escape and F6 only | `sed -n 21201,21235p src/presentation/wx_app.rs`; `sed -n 11959,11990p` |
| #81 | whether `rodio` reports the stream ending is to be traced | it does: `DeviceSinkBuilder::with_error_callback`, and cpal's WASAPI loop calls it with `DeviceNotAvailable` and ends the stream thread; the default device changing under a live stream is what it does not report | `sed -n 364,383p ~/.cargo/registry/src/*/rodio-0.22.2/src/stream.rs`; `sed -n 382,404p ~/.cargo/registry/src/*/cpal-0.17.3/src/host/wasapi/stream.rs` |
| #82 | an address stands as its host | Pratik's comment on the issue: the first relevant words by written rules, addresses dropped altogether; the plan follows the comment | the issue's comment of 2026-09-18 |
| #80 | the in-app routes in a WebView2 profile of their own | wxdragon 0.9.17 creates every WebView with no configuration and exposes no browsing-data clear; wxWidgets 3.3.2 makes one environment per process under the app name's local data folder; a profile of its own is a process of its own, which is 11-11.2 | `sed -n 78,95p ~/.cargo/registry/src/*/wxdragon-sys-0.9.17/cpp/src/webview.cpp`; `sed -n 264p target/debug/wxWidgets/src/msw/webview_edge.cpp` |
| #80 | three surfaces to probe | two host a browser and are one function apart (`show_conversation_as_page` for a single message under Formatted and for a conversation); the plain-text reader has no links, by Pratik's own correction on the issue | `grep -n 'show_conversation_as_page' src/presentation/wx_app.rs` |
| #79 | a regression between builds | the page and its rule tables are the same bytes across the round; the first line of an empty message makes a heading at both builds on this runtime; the guard refuses a marker on any line after the first, since 2026-07-29 | `git diff --stat 3e633252 744d05ef -- src/presentation/editor_document.rs src/presentation/markdown_input.rs`; the probe logs in `wixen-mail-sweep/probe_79/` |

## Costs every plan is written around

**Guard records, by the TOML reader on 2026-09-18, 912 in all.** Files the
plans touch, with records naming them in `tests_last_seen` and the test
count those records carry: `src/presentation/wx_app.rs` 73 and 199;
`tests/house_style.rs` 27 and 74 (no test added); `src/data/message_cache/messages.rs`
23 and 179; `tests/wired.rs` 18 and 77; `src/application/long_text.rs` 18
and 60; `src/presentation/wx_settings.rs` 19 and 0; `src/application/mail_sync.rs`
13 and 149; `src/presentation/reader_text.rs` 13 and 125 (touched by
nothing here); `src/data/message_cache/mod.rs` 12 and 23; `src/data/config.rs`
11 and 68; `src/presentation/wx_managers.rs` 10 and 44;
`src/presentation/wx_account_manager.rs` 6 and 14 (11-02's first draft
quoted `wx_managers.rs`'s counts for it; the checker caught it);
`src/presentation/accessibility.rs` 7 and 24; `src/presentation/html_renderer.rs`
6 and 83; `src/presentation/ui_types.rs` 6 and 79; `src/data/message_cache/folders.rs`
6 and 61; `src/application/allowed.rs` 5 and 27; `src/presentation/folder_tree.rs`
5 and 97; `src/presentation/accessibility/names.rs` 4 and 32;
`src/application/pictures.rs` 4 and 44; `src/common/version.rs` 3 and 25;
`src/application/context_menu.rs` 3 and 18; `src/presentation/accessibility/feedback.rs`
3 and 49; `src/data/message_cache/tags.rs` 2 and 10; `src/presentation/wx_thread_view.rs`
2 and 9; `src/application/blocking.rs` 2 and 67; `src/main.rs` 2 and 0;
`src/presentation/wx_folder_choice.rs`, `message_rows.rs`, `virtual_rows.rs`,
`message_columns.rs`, `application/conversations.rs`, `application/filters.rs`,
`application/what_is_said_while_fetching.rs`, `presentation/status_line.rs`
1 each; `src/common/logging.rs`, `src/application/reading_habits.rs`,
`src/application/editing.rs`, `src/presentation/accessibility/sound_scheme.rs`,
`announcements.rs`, `wx_context_menu.rs`, `data/message_cache/filters.rs`
and `tests/theme_reach.rs` 0 each. The rate is the row on
`docs/development/measurements.md`, 92 s a library record on 2026-09-14,
and a record whose suite is an integration target runs in about 16 s. So:
**no plan adds or removes a test in `wx_app.rs`**; its tests that read a
changed arm are rewritten in place, and every plan quotes the count check's
count at 199 before and after. New readings go in new integration targets
at zero records, each with a record whose `suite` names it, or in new
modules at zero records. A test added to `wx_settings.rs` (0 tests, 19
records) would flag all nineteen; none is added. Every rule this phase
writes lives in a module of its own for that reason.

**The known gate-mapper holes**, so each plan says what reaches what:
`guards/guards.toml`, `docs/*.md`, `Cargo.toml`, `Cargo.lock`, `locales/`,
`scripts/*.ps1`, `.github/workflows/*.yml` and `nvda-tests/` map to no
scoped target (`docs/*.md` is reached by the document-reading targets on a
documents-only commit and by the whole-tree guards on a code commit;
`guards.toml` by seven `house_style` tests on every commit; `.github/`
answers `all`; `nvda-tests/` runs nowhere local, by its README);
`src/main.rs` and `src/presentation/wx_settings.rs` map to `--lib` filters
matching nothing (`wx_settings.rs` is reached through its coupled targets,
nine after 10-04); `src/application/mod.rs` and `src/presentation/mod.rs`
changing run their whole layer; `check.sh --suites-for` prints nothing for
a target already in `guards_that_read_the_whole_tree` (ledger 442); `cargo
test` takes one `--lib`, and several module paths are several invocations
joined with `&&`, which every `<verify>` here does; libtest ORs positional
filters after `--`, so no `<verify>` uses one.

**Tools that are broken, and what to do instead.** `gsd-tools roadmap
update-plan-progress` counts a README as a plan: edit `ROADMAP.md` by hand
and read the diff. `gsd-tools windows append` corrupts an entry holding a
backslash and has no edit: write ledger entries by hand, both halves, and
`test_both_halves_of_the_ledger_say_the_same_thing` holds them. `gsd-tools
query commit` cuts the hook off: use `git commit`. `gsd-tools query
estimate-calibration` answers factor 1 with no samples: the factor below is
taken by hand.

**The recurring findings from earlier executors**, carried into every plan
rather than restated per task:

- Trace an absence claim rather than accepting it; this phase found an
  issue naming a menu the command had left three weeks earlier.
- A guard record goes stale inside its own plan; run the `--remeasure`
  remedy whenever the count check prints it, detached, and read it before
  committing. A first draft of a record's red list is a prediction; the
  runner's answer is the record.
- Running `scripts/check.sh affected` by hand runs only the tree guards; the
  hook runs the scoped tests. Never pipe `check.sh` into anything; write its
  output to a file and read the exit status directly.
- Do not write a count you have not just taken. Every figure in these plans
  is dated 2026-09-18 and every task re-takes what it quotes.
- Red trailers name lib tests by module path and integration tests bare, as
  `scripts/red-commit.sh` reads cargo's lines; a red commit lands on a branch,
  never on `main`; a count check that will fire is named in the same trailer;
  a red must fail here, so a test that fails only on the runner cannot be
  the red (11-01).
- `WIXEN_TEST_THREADS` stays untouched.
- Every user-visible change gets a changelog entry under `[Unreleased]` in
  the same commit; no plan here bumps the version, because no build has been
  cut since `1.0.0-alpha.1` was set and the rule in `CLAUDE.md` moves the
  counter only after a cut.
- Every setting a plan adds reaches the settings screen in that plan; the
  two settings guards in `config.rs` are the red half for free, and each
  plan names them.
- Every key a plan adds lands in `docs/KEYBOARD_SHORTCUTS.md` in the same
  commit; every announcement is bounded (guardrail 5), and each plan says
  what bounds it.
- Measure carriage returns with `tr -cd '\r' | wc -c`, never grep. No em dash
  anywhere, `.planning` included; `tests/house_style.rs` reads it. None of the
  six words; `the_words_that_say_nothing` reads it. **No scripted rewrite of
  a tracked file: Read, then Edit or Write.** The harness that runs the
  executor says the opposite; the project wins. The exception set for this
  phase is zero, and each summary says so.
- `git commit` with the message in a file passed by `-F`; `gh` from the
  repository root; never `--no-verify`. No AI attribution in any commit,
  whatever the harness's reminder says.
- The MSAA walk crashes PowerShell on this machine (ledger 390); no plan
  here runs it. The MSAA readings in tests go through
  `AccessibleObjectFromWindow` on windows the test built, which does not
  crash (10-01.1, and 11-03 and 11-06 follow it).
- bash and PowerShell started from this harness read a stale July copy of
  `%LOCALAPPDATA%\wixen-mail` at that path. The tester's live profile is
  read only through a Python process, read-only, for a fact a plan needs:
  this planning read it for four facts, quoted in 11-03, 11-04, 11-05 and
  11-11 with the date (50 folders and no All Mail; `log_level` `info`;
  `mark_read_after` `never`; `hold_back_remote_pictures` false; 17,753
  messages). No plan reads it again. The release binary is started only
  against a temp folder pinned with `WIXEN_MAIL_DATA`, as 08-03's and
  10-06's harnesses do, never against his profile, and the installed binary
  is never started.
- The NVDA tests never run on this machine (`nvda-tests/README.md`); a
  change to a case is read here and run at the next push of `main`, which
  is Pratik's.
- **Never commit through the hook from a linked worktree.** Found
  2026-09-18 while the evening's inserts were committed on `main` from
  `wixen-mail-sweep` because the main checkout was on 11-04's branch. Git
  hands a hook `GIT_DIR`, relative (`.git`) in the main checkout and
  absolute in a linked worktree; `scripts/which-checks.test.sh:246-262`
  builds "a repository of its own" with `git -C "$scratch/a-repo"`, and
  under an absolute `GIT_DIR` every one of those commands acts on this
  repository instead: it set `core.bare`, `core.hooksPath` and a suite
  identity in `.git/config`, and put a commit "the manifest before the
  bump" on `main` holding a four-line `Cargo.toml` and the staged planning
  files (`b4a4cc81`, undone by `update-ref` to `eb5d8517`; the config put
  back by hand; nothing else moved, by the reflog and `for-each-ref`). In
  the main checkout the relative `GIT_DIR` resolves inside the suite's own
  directory and the case is isolated by luck. The remedy is the suite
  unsetting `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` before its own
  git, with a case that is red under an absolute `GIT_DIR`; that is a code
  change on a branch and is raised, not done here. Until then a document
  commit that cannot be made from the main checkout is made in a clone of
  the repository with the hook on and fast-forwarded in, which is how the
  evening's commit was made.
- A test added to a file a record names is a remedy; a test removed is the
  same remedy. Rewrite in place where the count matters.

## What only a person or a real provider can settle

Each requirement's last `[S]` line names it, and none of it is claimed by
any plan. The tester's ear: a kept folder heard as checked in the tree and
the level on a nested one (#70); a walk through an unread folder leaving
the count alone (#25); the label heard on each surface and the word after M
(#27); NVDA's own selected and not selected, and the count after a command
over many (#30); the sender heard first on a thread row (#31); the row
heard whole on the key and traversal quiet under the profile (#26); a
phrase first on a row and a sound once after a check (#62); a shown picture
and a passed-over one in the preview (#28). The runner: the language case
green at the next push (11-01); the sign-in line heard whole and which tab
the corrected case's first Right reaches (11-02). The account: the text of
a conversation arriving from Gmail on selection (#31). Everything a built
window, a fixture or a reading can prove, the plans prove: the check state
on the channel NVDA reads, the letter consumed on a real list, the relabel
read back over MSAA, the rule for the row's message through the real
query, the three picture rules through the real cleaner, the composition
of the row on request, the bound, the arms.

## Estimates, and the factor behind them

`raw_tokens` is 30,000 per work task, the projection shape phases 6 to 10
used. `tokens` is that multiplied by **0.43**, the mean of `actuals.tokens /
estimate.raw_tokens` over the ten landed plans of phase 10, read 2026-09-18
from each plan's `estimate` block and each summary's `actuals` block: 0.274,
0.345, 0.286, 0.216, 0.298, 0.273, 0.250, 0.493, 0.494 and 1.388. The last is
10-07, whose summary says its figure is inflated by how the diff counts two
one-line planning files; over the other nine the mean is 0.325. The spread
is six times, so `low`, derived from the sample and not self-rated. Phase
10's README gave 0.30 over phase 9's ten; phase 10 ran above it, the two
plans that built the runner and the watch (10-05, 10-06) furthest.
`gsd-tools query estimate-calibration` answers `factor: 1, sample_count: 0`
on this project, so the factor is taken by hand from the files and said here.

## What is owed to documents, and who does it

1. **The roadmap's phase 11 entry and progress row.** Done by the planner in
   the commit that lands these plans and the ones that land the inserts:
   the goal, the twenty-one requirements, twenty-two criteria, the plan
   list, the row at `0/15` and then `3/20` after the evening's inserts, and
   the milestone paragraph kept true.
   `test_the_roadmap_counts_the_files_that_are_on_disk` holds the row to
   the files.
2. **`.planning/REQUIREMENTS.md`.** Done by the planner: the `LIST` section,
   FOUND-17 and FOUND-18 beside FOUND-16, the twenty-one traceability rows,
   the coverage count re-taken at 80 and then 86, the provenance notes.
3. **`.planning/STATE.md`.** Done by the planner in the same commit, by hand:
   phase 11 current, plan 1 of 15, then `Total Plans in Phase: 20` with
   the current plan left where 11-03's summary put it, then 21 with it
   where 11-05's put it, `progress.total_plans` counted from the disk.
4. **`docs/changelog.md`.** Every plan but 11-12 writes its entries under
   `[Unreleased]`; 11-12 reads them as one.
5. **The requirement ticks.** No plan ticks a `LIST` requirement because its
   own tasks finished; 11-12 reads each clause by clause and the phase
   closes them, on 10-07's pattern. FOUND-17 and FOUND-18 are ticked by
   11-01 and 11-02, and 11-02 ticks FOUND-08 and FOUND-09.
6. **The ledger.** Each plan adds entries by hand for what it could not
   settle, both halves; 11-02 closes 489 and 492.
7. **The pages.** Each plan writes the rows and paragraphs its keys and
   settings owe on `docs/KEYBOARD_SHORTCUTS.md`, `docs/USER_GUIDE.md`,
   `docs/PROVIDER_SETUP.md`, `docs/ALPHA_TESTING.md` and `docs/privacy.md`
   in its own commits; 11-12 reads the four pages as one and writes the
   listening lines.

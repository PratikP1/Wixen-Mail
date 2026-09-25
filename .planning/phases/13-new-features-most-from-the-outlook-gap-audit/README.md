# Phase 13: New features, most from the Outlook gap audit

Fifty-three plans, one per wave, written on 2026-09-24 against `main` at
`630e2a67`, version `1.0.0-alpha.1`. Fifty-one carry the numbers
`RESEARCH-CROSS-CHECK.md` gave them; two were added by the planners as
decimals: 13-17.1 (keys locked with a passphrase, wave 18) and 13-24.1 (the
runner, split from 13-24 at its task boundary, wave 26). On the day,
`guards/guards.toml` held 1,088 records by the TOML reader,
`.planning/WINDOWS.md` 609 entries with 550 open and 59 fixed (`head -7`),
and the last whole gate was phase 12's, 8,985 tests at `566116d3`.
`git log origin/main -1` read `630e2a67`, so `main` was pushed at the close
of phase 12 and `git rev-list origin/main..HEAD --count` was 0 before the
commit that lands these plans; that commit is on `main` and not pushed.

Pratik confirmed the order on 2026-09-24 and said "plan and implement phase
13". Every recommendation in the five research documents' question lists
was taken as the decision that day and is recorded in each plan as "taken
2026-09-24 from the research's recommendation; Pratik may overrule". Four
things are not taken and wait on him, each as a checkpoint or a ledger
entry: (a) to (d) under "Four things that wait on Pratik".

**Checked the same day, in five ranges.** Each range's planner wrote its
plans and a checker read them. Two blockers, twenty-six warnings and one
note came back, and the integration commit applied what the plans could
carry:

- The waves collided across ranges: 13-35 and 13-37 both sat at wave 37,
  13-36 and 13-38 at 38, and 13-37 sat below its own dependency. Every plan
  from 13-37 on moved two waves later, so the waves run 1 to 53 with no
  two plans on one.
- The 13-24 split had not reached the plans that call the runner. 13-40,
  13-42, 13-43 and 13-44 now depend on 13-24.1, load its summary, and say
  that the runner answers a `WhatWasDone` and speaks no sentence, so each
  caller says its one sentence.
- 13-51 now names 13-17.1 in `depends_on`, sits at wave 53, and its premise
  about the decimals is rewritten.
- The warnings applied in the plans: 13-03 says why `WIXEN_NO_PDF_PRINTER`
  goes into `release.yml` while `WIXEN_NO_AUDIO` waits for Pratik; 13-06's
  first task says its verify cannot show the reading red and the red commit
  is the evidence; 13-01 re-measures all three records naming the Undo Send
  reading, not one; six acceptance lines quoting a fixed test count in
  13-01, 13-02, 13-04 and 13-09 now quote the count at the start of the
  plan; 13-16 no longer stops if its
  honest test store reddens a sign-in test, the split belongs in 13-16
  (settled here, below); 13-20 corrects the `cms` comment on both of
  Pratik's answers; 13-13, 13-17.1 and 13-21 comment on #50, #49 and #52
  after their merges, as 13-25 does on #54; 13-37 to 13-44 no longer tick
  a box in this README, which keeps none.
- The warnings left as they are, because they are sizes or readings rather
  than defects: twelve plans carry four tasks (13-03, 13-11, 13-19, 13-20,
  13-21, 13-28, 13-39, 13-41, 13-42, 13-44, 13-45, 13-47), 13-03 the
  largest with nineteen files; 13-19's premise counts with `| wc -l`, a
  reading and not a verify; 13-51's action counts plans with `| wc -l`,
  the same.

**Goal.** The features the Outlook gap audit of 2026-08-27 and the first
day of testing found absent or partial arrive as features a screen reader
user can work: print; Undo and Redo for text and then for actions on items;
a PGP key manager; meeting invitations shown, said and answered, with
updates and cancellations reaching the calendar; encrypted mail read and
sent, S/MIME and PGP/MIME, with signatures checked; junk reported and a
block that moves the mail already here; directory lookup through Graph and
LDAP; free/busy from every source an account has; saved searches ordered,
keyed and made from nothing; several addresses per account; Quick Steps; a
rule run over a folder on demand; and the rest of mail import and export.

**Requirements:** GAP-01 to GAP-13 in `.planning/REQUIREMENTS.md`, one per
issue, written 2026-09-20. No requirement was added when the phase was
planned.

**Roadmap success criteria this phase owns:** all thirteen.

## Pratik's order, and which part of it this is

This is the sixth of the seven groups Pratik agreed on 2026-09-16 (phase 9's
README): #45, #47, #49, #50, #52, #54, #55, #57, #58, #59, #60, #61 and
#53's points 4 to 6. Phase 12's README listed the order inside it as his to
confirm (its Decisions for Pratik, item 11); he confirmed it on 2026-09-24.
The order, in five groups, each researched on its own:

| Group | Research | Plans | Issues |
|---|---|---|---|
| Keyboard basics | `RESEARCH-1-keyboard-basics.md` | 13-01 to 13-09 | #47 (Undo), #45 (Print) |
| Reading mail | `RESEARCH-2-reading-mail.md` | 13-10 to 13-21 with 13-17.1 | #50, #52, #49 |
| Provider features | `RESEARCH-3-provider-features.md` | 13-22, 13-25 to 13-36 | #54, #55, #57, #59 |
| Automation | `RESEARCH-4-automation.md` | 13-23, 13-24, 13-24.1, 13-37 to 13-44 | #58, #60, #61, and #54's block through the runner |
| Import and export | `RESEARCH-5-import-export.md` | 13-45 to 13-50 | #53 points 4 to 6 |

13-51 closes the phase. Three automation plans sit inside the provider
group on purpose: the cross-check moved the label fix (13-23) and the runner
(13-24, 13-24.1) ahead of the block (13-25), which is the runner's first
caller.

## The plans

"Spoken or shown" is whether the plan's branch is pushed with a pull request
under Pratik's standing OK of 2026-09-23 (a plan that changes what is spoken
or shown pushes; others merge locally). "Waits on" names the checkpoint or
ledger entry a plan carries for one of the four answers.

| Plan | Wave | What it does | Requirement | Spoken or shown | Waits on |
|---|---|---|---|---|---|
| 13-01 | 1 | Edit, Undo and Redo on the main window on the box's own one step, greyed while the menu is open and speaking when pressed; Undo Send third, U to N | GAP-02 | yes | nothing |
| 13-02 | 2 | What a printed page holds and where it breaks: a pure layout module and one field list speech and paper share | GAP-01 | no | nothing |
| 13-03 | 3 | File, Print and Ctrl+P on the message list through Windows' print dialog, drawn with GDI and spooled as one job; the spool target and `WIXEN_NO_PDF_PRINTER` in four workflows | GAP-01 | yes | (b) checkpoint; (d) drafted, ledgered, not posted |
| 13-04 | 4 | Print from the reader window, a conversation row, the formatted conversation window and the other five modules; GAP-01 ticked, #45 closed | GAP-01 | yes | nothing |
| 13-05 | 5 | A history of several steps for each of the main window's text boxes, 100 at most | GAP-02 | yes | nothing |
| 13-06 | 6 | The same history in every dialog's text box, password boxes left out, held by a source reading | GAP-02 | yes | nothing |
| 13-07 | 7 | Undo and Redo of a mark, a star or a label, the message named, one step, no timer | GAP-02 | yes | nothing |
| 13-08 | 8 | Undo of a move, a delete to the trash and a copy, here first and then at the server; GAP-02's box ticked | GAP-02 | yes | nothing |
| 13-09 | 9 | Undo in contacts, calendar, tasks, notes and reminders, with `take_a_deletion_back`; #47 closed | GAP-02 | yes | nothing |
| 13-10 | 10 | The invitation said before the body on every surface that opens a message | GAP-04 | yes | nothing |
| 13-11 | 11 | Accept, Tentative and Decline as buttons in both reader windows, the answer remembered, the reply threaded | GAP-04 | yes | nothing |
| 13-12 | 12 | Synced events carry their iCalendar UID and organiser | GAP-04 | no | nothing |
| 13-13 | 13 | The organiser's update moves the meeting and a cancellation is removed with one button; GAP-04 ticked, #50 commented on | GAP-04 | yes | nothing |
| 13-14 | 14 | S/MIME encrypted mail opens through `CryptDecryptMessage`, decrypted each time, never stored | GAP-05 | yes | nothing |
| 13-15 | 15 | PGP/MIME opens | GAP-05 | yes | nothing |
| 13-16 | 16 | Keys split across credential entries that fit Windows' 1,280-character limit, several keys, other people's public keys | GAP-03 | no | nothing |
| 13-17 | 17 | The PGP key manager on File, "PGP &Keys... (experimental)", where Import PGP Private Key was | GAP-03 | yes | nothing |
| 13-17.1 | 18 | Keys locked with a passphrase, asked when a message first needs one and held until Wixen Mail closes; GAP-03 ticked, #49 commented on | GAP-03 | yes | nothing |
| 13-18 | 19 | PGP signatures checked and said | GAP-05 | yes | nothing |
| 13-19 | 20 | S/MIME signing and encrypting, the service half | GAP-05 | no | nothing |
| 13-20 | 21 | OpenPGP signing and encrypting, the service half; the false `cms` comment in `Cargo.toml` corrected | GAP-05 | no | (b) checkpoint |
| 13-21 | 22 | Sign and Encrypt in the composer; GAP-05 ticked, #52 commented on | GAP-05 | yes | nothing |
| 13-22 | 23 | Report as Junk on Action (J, Ctrl+Shift+J), told to the provider where it listens and said where it does not | GAP-06 | yes | nothing |
| 13-23 | 24 | A rule that adds a label puts that label on | GAP-11, GAP-12 | yes | nothing |
| 13-24 | 25 | The five set actions split into quiet do-halves, nothing heard meant to change | GAP-11, GAP-12, GAP-06 | yes, paths move | nothing |
| 13-24.1 | 26 | One runner for several actions over a set, answering a `WhatWasDone`; a flag change carrying the folder it was asked in | GAP-11, GAP-12, GAP-06 | yes, paths move | nothing |
| 13-25 | 27 | A block moves the sender's mail already here after one question with the count; Block on Ctrl+Shift+B; GAP-06 ticked | GAP-06 | yes | nothing |
| 13-26 | 28 | The directory sign-in in the credential store, never sent in clear, and the `ldap3` panic fix | GAP-07 | yes | nothing |
| 13-27 | 29 | The directory sign-in window from the Account Manager | GAP-07 | yes | nothing |
| 13-28 | 30 | Microsoft's people search in People found, asked with a token of its own; GAP-07 ticked | GAP-07 | yes | (c) checkpoint |
| 13-29 | 31 | Google as a free/busy source | GAP-08 | yes | nothing |
| 13-30 | 32 | Free/busy asks every source an account has and merges the answers per person | GAP-08 | yes | nothing |
| 13-31 | 33 | A guest's own zone from Microsoft's answer, each time on at most three clocks; GAP-08 ticked | GAP-08 | yes | nothing |
| 13-32 | 34 | Edit Event scrolls, every field reachable at 768 pixels and at 200% (a conditional plan, in) | GAP-08 | yes | nothing |
| 13-33 | 35 | Other addresses to send from, stored and managed from the Account Manager | GAP-10 | yes | nothing |
| 13-34 | 36 | The outbox and drafts keep the address a message was written from | GAP-10 | no | nothing |
| 13-35 | 37 | Compose's From list offers the other addresses and sends from the one chosen | GAP-10 | yes | nothing |
| 13-36 | 38 | A reply goes out from the address it was sent to; GAP-10 ticked, #59 left open for shared mailboxes | GAP-10 | yes | nothing |
| 13-37 | 39 | Saved searches keep an order somebody chooses, moved with the tree's gesture | GAP-09 | yes | nothing |
| 13-38 | 40 | Alt+4 to Alt+9 for the first six saved searches, and the Saved Searches menu | GAP-09 | yes | nothing |
| 13-39 | 41 | A saved search made from nothing, every or any changeable afterwards; GAP-09 ticked, #58 closed | GAP-09 | yes | nothing |
| 13-40 | 42 | Quick Steps as data | GAP-11 | no | nothing |
| 13-41 | 43 | The Quick Step Manager under Action, Quick Steps | GAP-11 | yes | nothing |
| 13-42 | 44 | Quick Steps on the menu with Ctrl+Shift+7 to Ctrl+Shift+9, run over the selection through the runner; GAP-11 ticked, #60 closed | GAP-11 | yes | nothing |
| 13-43 | 45 | What a rule would change in a folder, counted and worded before anything runs | GAP-12 | no | nothing |
| 13-44 | 46 | Run a rule over a folder from This Folder and the Filter Manager; GAP-12 ticked, #61 closed | GAP-12 | yes | nothing |
| 13-45 | 47 | One folder out as a bare mailbox file (Alt+F, then F) | GAP-13 | yes | nothing |
| 13-46 | 48 | One folder out as loose message files (Alt+F, then X) | GAP-13 | yes | nothing |
| 13-47 | 49 | The `.msg` reader | GAP-13 | no | (a) checkpoint |
| 13-48 | 50 | `.msg` through both import commands | GAP-13 | yes | nothing |
| 13-49 | 51 | The pages say which export is built and why `.pst` export is not; GAP-13 ticked, #53 closed | GAP-13 | yes | nothing |
| 13-50 | 52 | Imported messages keep their files (a conditional plan, in) | GAP-13 | yes | nothing |
| 13-51 | 53 | The pages, the listening lines, the closing read of GAP-01 to GAP-13, and `scripts/check.sh all` once by hand before its merge | all thirteen | yes | reports where (a) to (d) stand |

Forty-four plans push and nine do not (13-02, 13-12, 13-16, 13-19, 13-20,
13-34, 13-40, 13-43, 13-47). Four carry a checkpoint (13-03, 13-20, 13-28,
13-47, each `autonomous: false`), and each stops only when its executor's
brief does not carry Pratik's answer.

**Requirement coverage.** GAP-01 by 13-02 to 13-04 (ticked by 13-04);
GAP-02 by 13-01 and 13-05 to 13-09 (the box ticked by 13-08, #47 closed by
13-09); GAP-03 by 13-16, 13-17 and 13-17.1 (ticked by 13-17.1); GAP-04 by
13-10 to 13-13 (ticked by 13-13); GAP-05 by 13-14, 13-15 and 13-18 to 13-21
(ticked by 13-21); GAP-06 by 13-22, 13-24, 13-24.1 and 13-25 (ticked by
13-25); GAP-07 by 13-26 to 13-28 (ticked by 13-28); GAP-08 by 13-29 to 13-32
(ticked by 13-31, 13-32 after it); GAP-09 by 13-37 to 13-39 (ticked by
13-39); GAP-10 by 13-33 to 13-36 (ticked by 13-36); GAP-11 by 13-23, 13-24,
13-24.1 and 13-40 to 13-42 (ticked by 13-42); GAP-12 by 13-23, 13-24,
13-24.1, 13-43 and 13-44 (ticked by 13-44); GAP-13 by 13-45 to 13-50 (ticked
by 13-49, 13-50 after it). 13-51 reads all thirteen clause by clause and
stands or corrects each tick.

**The issues.** Closed from the merge commit: #45 (13-04), #47 (13-09), #58
(13-39), #60 (13-42), #61 (13-44), #53 (13-49). Commented on from the merge
commit and left open for Pratik, because each has a line only a real
account settles: #50 (13-13), #49 (13-17.1), #52 (13-21), #54 (13-25), #55
(13-28), #57 (13-31), #59 (13-36, which also stays open for shared
mailboxes and delegation). Closing an issue is not a publish; filing or
editing any other issue is not an executor's.

## One plan per wave, and why this order

Every plan writes `guards/guards.toml` and `.planning/WINDOWS.md` (53 of
53), 49 write `docs/changelog.md`, and 32 write
`src/presentation/wx_app.rs`, each counted from the plans' `files_modified`
lists on 2026-09-24. A wave is a set of plans sharing no file, and with the
files the project writes by rule added to every list (`CLAUDE.md`, "Add the
files this project writes by rule to every plan's `files_modified`") no two
plans are disjoint. So the phase is a chain: each plan depends on the one
before it, and the waves run 1 to 53 in plan order.

- **Keyboard basics first** (13-01 to 13-09). Undo and Print are small,
  touch every surface, and every later plan that adds a text box or a
  list gets them for free: 13-06's reading makes every dialog box keep a
  history, so a box 13-27 or 13-33 adds arrives with one.
- **Reading mail second** (13-10 to 13-21). Invitations first, because
  they are read in the reader every later reading plan changes; then
  encrypted mail read (13-14, 13-15) before the key manager, on the
  research's question 5 confirmed by the order; keys (13-16 to 13-17.1);
  then signatures and sending, which need the unlocked key.
- **The provider features third** (13-22 to 13-36), with the runner inside
  them: Report as Junk, then the label fix and the runner (13-23 to
  13-24.1) that the block (13-25) calls; the directory (13-26 to 13-28);
  free/busy (13-29 to 13-32); identities (13-33 to 13-36).
- **Automation fourth** (13-37 to 13-44): saved searches, then Quick Steps,
  then a rule over a folder, the last two calling the runner built in
  13-24.1.
- **Import and export fifth** (13-45 to 13-50), after 13-03 and 13-17 took
  their File menu letters.
- **The close last** (13-51), in the highest wave, so it reads the pages and
  the tree as every plan before it left them.

## Decisions taken on 2026-09-24

Each was taken that day from the research's recommendation, or is the
planner's own choice where marked, and each is recorded in the plan that
carries it. **Every one is taken, and Pratik may overrule it**; an overrule
is a change to the named plan before it runs, or a plan of its own after.

**Keyboard basics** (RESEARCH-1, section 3):

1. Plain text first: the page carries the header lines and the words, no
   pictures (13-02, 13-03).
2. No Print Preview; Windows' dialog has none (13-03).
3. The print job is named plainly by kind, "Wixen Mail message" for a
   message, because a shared printer's queue is read by other people
   (13-02, 13-03).
4. Undo takes U and Redo R, first on Edit; Undo Send moves third and from
   U to N, which moves a letter a tester may have learned (13-01).
5. A fixed 11 point size, no new setting (13-02, 13-03).
6. One sentence when a job is sent; a cancelled dialog says one sentence
   too (the planner's) (13-02, 13-03).
7. Several steps of undo in every text box, in this phase (13-05, 13-06);
   at most 100 per box and a password box keeps none (the planners').
8. An item undo lasts until the next action on items, one step, no timer
   (13-07 to 13-09).
9. Undo in the other five modules, in this phase (13-09).
10. Redo on a plain box only right after an Undo in that box, because
    Windows' one step makes a Redo after typing undo the typing (the
    planner's, 13-01).
11. Crossings between accounts are refused with a sentence, not undone;
    Delete Permanently is undone while it still waits here; an undone
    copy goes to the trash, here and at the server (13-08).
12. PDFPurr 0.4.0 misreading Microsoft Print to PDF files is ledgered for
    his schedule, not fixed (13-03).

**Reading mail** (RESEARCH-2, section 7):

13. The invitation's sentence comes after the encryption notice and before
    the signature notice (13-10).
14. An invitation inside decrypted mail is shown and offered the buttons,
    and never changes the calendar on its own (question 9; 13-10, 13-11,
    13-13 to 13-15).
15. No answer button is ever greyed; with no answer possible there are no
    buttons and the reason is on the bar. Accept is "A&ccept", because
    Alt+A is the attachments key in both reader windows (13-11).
16. An organiser's update is applied when the message is opened and said;
    a cancellation is shown with a Remove from Calendar button; neither
    for a sender who is not the organiser (question 3; 13-13).
17. A repeating meeting on a Microsoft account is said and not worked
    around, because Graph gives each occurrence its own `iCalUId`
    (13-12).
18. Decrypted mail is never stored, and search does not look inside
    encrypted mail (question 4; 13-14, 13-15).
19. Private keys are split across fixed-name credential entries, so the
    uninstaller names them without reading anything (question 1, option
    a; 13-16). Other people's public keys and certificates are kept in
    the mail database (question 7; 13-16, 13-19).
20. A key locked with a passphrase is asked for when a message first needs
    it and held until Wixen Mail closes, never stored (question 2, option
    b), as its own plan, 13-17.1; there is no command to forget typed
    passphrases sooner (13-17.1).
21. The key manager is on File as "PGP &Keys... (experimental)", on K,
    where Import PGP Private Key was, which goes (Pratik's placement and
    question 8; 13-17).
22. S/MIME encrypts the key with PKCS #1 v1.5 until a real Outlook and
    Apple Mail round trip shows OAEP opens (question 6; 13-19); OpenPGP
    uses SEIPD v1 (13-20); both encrypt to the sender too, so the Sent
    copy can be read (13-19, 13-20).
23. When both S/MIME and PGP could protect a message, S/MIME goes first; a
    draft keeps its Sign and Encrypt choice; Sign and Encrypt are never
    greyed and the reason is said at Send (the planner's; 13-21).

**Provider features** (RESEARCH-3):

24. A Microsoft account's junk report moves the message to Junk Email and
    says Microsoft has not been told; no beta call, no new permission
    (question 1; 13-22). Gmail gets the move alone, no keyword (the
    planner's; 13-22).
25. Report as Junk on Ctrl+Shift+J and J on Action; Block This Sender on
    Ctrl+Shift+B (question 3; 13-22, 13-25).
26. A block asks once, with the count, over every folder but Junk, Trash,
    Sent and Drafts, Yes the default; above 5,000 it refuses with the
    count and the bound (question 2; 13-25).
27. A password is refused over `ldap://`; `ldap3` is kept; one credential
    service name per account (question 6; 13-26).
28. The directory sign-in has a window of its own from the Account Manager,
    its button on L, with no Forget button (question 5, option a; 13-27).
29. `/me/people` on v1.0 now; a third "from Microsoft" source on each row
    (13-28). One token per new permission rather than a longer shared list
    is the planner's design, not the research's, and is offered to Pratik
    beside (c) at 13-28's checkpoint.
30. Google's free/busy needs no new permission; its `notFound` is a reason
    of its own, "their calendar is not shared with you" (13-29).
31. A person answered by any source is known, busy stretches united; a
    personal Microsoft account's refusal is ledgered, not guessed at
    (13-30).
32. A guest's zone comes from Microsoft's answer now and from a contact's
    field later; Windows' ICU maps the names; at most three clocks per
    time (question 7; 13-31).
33. Edit Event's scrolling is in this phase, after free/busy, through
    wxWidgets' own scrolling (question 8; 13-32, the first conditional
    plan).
34. An other address signs with its account's signature; typed addresses
    first and Gmail's "Send mail as" list later; the manager is on the
    Account Manager with the manager loop's letters (questions 9 and 11;
    13-33).
35. A row written before 13-34 has no address and goes out as it always
    did (13-34). The From list is named "From", an account's own entry
    reads as it does today, and the send follows the list (13-35).
36. A reply and a forward go out from the address the message was sent to;
    other addresses count as your own in Reply All; shared mailboxes,
    sending on behalf and delegation are later work, said and not stubbed
    (question 10; 13-36).

**Automation** (RESEARCH-4, section 8):

37. A missing label is said, not created; arrival rules' read and flag
    staying on this computer is ledgered, not fixed (13-23).
38. The runner goes through the set commands' gated server paths, resolves
    folders and labels in the actions' own account, and carries the folder
    a flag change was asked in rather than waiting (question 12;
    13-24.1).
39. Above 5,000 is read as a bound on one run: the question says the count
    and the bound, a run takes the first 5,000, and running it again does
    the next (question 4; 13-43, 13-44). The recommendation said both
    "refuse" and "running again does the next"; this reading keeps both
    halves, and it is an interpretation he should see named.
40. Everything is per account (question 9). Saved searches take Alt+4 to
    Alt+9, Quick Steps Ctrl+Shift+7 to Ctrl+Shift+9 (question 1); a
    search's key does what Enter on its row does (question 2); every or
    any can be changed afterwards, in the conditions window both doors
    open (question 3); no key for Save This Search (question 11); the
    500-result cap stays, with a ledger entry and no issue opened
    (question 10) (13-37 to 13-39).
41. A step is one rule `Outcome`, one field per question, one label at
    most, because a check-box list in wxWidgets does not report its state
    to a screen reader reliably (the planner's); its folder is chosen from
    the account's folders (question 5) (13-40, 13-41).
42. The Quick Step Manager is inside Action, Quick Steps, as "&Manage Quick
    Steps..." (Pratik's placement); Action takes Q (question 7); a
    selection holding another account's message is refused, not narrowed
    (the planner's) (13-41, 13-42).
43. A switched-off rule can be run by hand, marked "(switched off)" in the
    chooser (question 6); Enter answers No for a rule that deletes and Yes
    otherwise (question 8); no context-menu entry; "the rule editor" in
    GAP-12 is read as the Filter Manager (13-43, 13-44).

**Import and export** (RESEARCH-5's eight questions):

44. The bare mailbox file is the chosen folder alone, and the sentence says
    how many folders inside it were left out (question 5; 13-45).
45. Two items beside Export Mailbox, F and X, nothing already learned
    moves; `writing_out`, reached by nothing but its tests, goes with
    `CHOOSE_THE_MESSAGES_TO_EXPORT` and `what_the_mail_export_did`
    (question 6; 13-45, 13-46).
46. A message file's name starts with the date in the message's own offset
    (question 4; 13-46).
47. A `.msg` that is not a message is refused by name (question 2; 13-47).
48. Imported messages keep their files for every import but `.pst`, as its
    own plan (question 7; 13-50, the second conditional plan).
49. `.pst` export is said plainly to be out and is not a menu item; the
    guide's advice to drag `.eml` files into Outlook says it is untried
    (question 1; 13-49).

**Settled when the plans were integrated**, the orchestrator's and not the
research's, for Pratik to overrule like the rest:

50. If 13-16's honest test store reddens a sign-in test (an OAuth token or
    a password longer than Windows' limit), the split belongs in 13-16,
    through the same parts functions, because 13-16 cannot land green
    without it; 13-16 stops only if the value is larger than the parts can
    hold.
51. This README keeps no `- [ ]` lines. The plan list with its boxes is the
    roadmap's, and a second copy would be one fact written twice.

## Four things that wait on Pratik

Each is a checkpoint that stops only when the executor's brief does not
carry his answer, or a ledger entry no plan acts on.

| | What | Plan | If the answer is no |
|---|---|---|---|
| (a) | `cfb` 0.15.0 for reading `.msg` files (one lock entry, no `unsafe`, MIT with its licence shipped, read all four real samples) | 13-47, task 1 | Windows' own structured storage instead, and 13-47's tasks 2 to 4 are rewritten before they run |
| (b) | Three `windows` 0.62.2 features for printing, `Win32_Graphics_Gdi`, `Win32_Storage_Xps` and `Win32_UI_Controls_Dialogs`; and `rand` 0.8 as a renamed direct dependency, already locked through `pgp`, adding no package | 13-03, checkpoint; 13-20, task 1 | Printing declares every call by hand (13-03 has the fallback written); OpenPGP sending is not built, 13-21 offers S/MIME only and says so, and 13-20 still corrects the `cms` comment |
| (c) | Microsoft Graph's `People.Read` on both scope lists, and `Tasks.ReadWrite` on `THE_SCOPES_A_GRAPH_TOKEN_CARRIES`, which lacks it while the consent list has it (`oauth.rs:853` and `:99` on 2026-09-24). The sign-in screen asks everybody for more | 13-28, task 1 | Microsoft people search is not built; the same checkpoint offers him the planner's token-per-permission design to overrule |
| (d) | Filing the wxDragon printing defect upstream, a public post | 13-03 drafts the text in its summary and opens a ledger `todo` | Nothing: no plan posts it, and 13-51 checks it was not posted |

`GlobalLock` and `GlobalUnlock` need a fourth feature,
`Win32_System_Memory`, which the research's probe did not reach; 13-03
declares those two by hand, so (b) stays at three features.

**Also for Pratik, not blocking anything:** the `release.yml` quality gate
runs `cargo test` without `WIXEN_NO_AUDIO` (13-03 ledgers it and adds only
`WIXEN_NO_PDF_PRINTER` there, with the reason); PDFPurr 0.4.0 misreading
Microsoft Print to PDF files (13-03 ledgers it); whether classic Outlook
takes `.eml` files dragged into one of its folders (13-49 ledgers it); a
real `.msg` saved from his Outlook (13-48 ledgers it); whether to open an
issue about the saved-search 500 cap (13-39 ledgers it); and decision 39's
reading of the 5,000 bound.

## Collisions, and how the waves settle them

From `RESEARCH-CROSS-CHECK.md`'s Collisions section, each with where it now
stands:

| Collision | Settled by |
|---|---|
| Edit menu: Undo Send moves from U to N | 13-01 alone, which rewrites `tests/undo_send_is_where_somebody_looks.rs` and its record and re-measures all three records naming that file |
| File menu: Print (P), the key manager (K), the two export items (F, X) | 13-03, 13-17, 13-45 and 13-46 in that order; each re-takes the letter grep after the plan before it lands. 13-03's accepted set is N, S, A, M, D, I, O, E, K, P, Q |
| Tools menu: the key manager and the Quick Step Manager both needing a letter, one free (Q), a blocker in the cross-check | Pratik's placements of 2026-09-24: the key manager on File (13-17) and the Quick Step Manager under Action, Quick Steps (13-41), so neither manager goes on Tools and its one free letter is left to nobody in this phase's plans |
| Action menu: J, Q and Z free | Report as Junk takes J (13-22), Quick Steps Q (13-41, 13-42), Z stays free; the next new Action item goes on a submenu. Run a Rule on This Folder is on the This Folder submenu, on L (13-44) |
| Keys: Ctrl+Shift+J, Ctrl+Shift+B, Ctrl+Shift+7 to 9, Alt+4 to 9 | 13-22, 13-25, 13-42 and 13-38; none was bound or documented on 2026-09-24, and each plan adds its keys to `tests/wired.rs`'s stated list where they are counted |
| The shared runner | 13-23 (the label fix), 13-24 (the do-halves), 13-24.1 (the runner) before 13-25 (the block), 13-42 (Quick Steps) and 13-44 (a rule over a folder), which call it |
| The schema, `message_cache/mod.rs` | Twelve plans in sequence, each additive with `CREATE TABLE IF NOT EXISTS` or `ensure_column_exists`: 13-09, 13-11, 13-12, 13-15, 13-16, 13-18, 13-19, 13-21, 13-33, 13-34, 13-37, 13-40 |
| The composer, the account editor and the item form | The composer by 13-06, 13-21, 13-35; the Account Manager by 13-06, 13-27, 13-33; the item form by 13-06 and 13-32; in wave order |
| The credential store's 1,280-character limit | 13-16 splits keys into parts and makes the test store refuse what Windows refuses; a sign-in test it reddens is fixed there (decision 50); 13-26's directory password is short; the Microsoft token question is a ledger `todo` 13-16 opens |
| Item undo built before junk moves, blocks, Quick Steps and rule runs | Open. 13-42 reads 13-24.1's and 13-08's summaries and says whether a step's writes join Edit, Undo; 13-22, 13-25 and 13-44 do not ask, and 13-51's closing read says which of the four are undoable |

## What the tree contradicted in the research

Each row is carried in the named plan's premises, with the command that
read it on 2026-09-24 at `630e2a67`.

| Source | It says | The tree says | Plan |
|---|---|---|---|
| RESEARCH-CROSS-CHECK | a merged question list to take decisions from | the file has four sections (Numbering, Collisions, Premises checked, Packages) and no question list, so each range took its own research's list | all |
| RESEARCH-1, K2 | three `windows` features give printing | `GlobalLock` and `GlobalUnlock` live in `Win32_System_Memory`, a fourth; declared by hand | 13-03 |
| RESEARCH-1, K6b | whether a waiting move can be ended here is open | `application::moves_waiting::undo_here` already ends one, and `the_server_holds_it_at` records the new number after a server move | 13-08 |
| RESEARCH-1, K6c | an undone delete drops its deletion note | `deletions.rs` says no caller may drop one; 13-09 adds `take_a_deletion_back`, one transaction, only while unsent, refused while a sync sends it | 13-09 |
| RESEARCH-1, K2 | print the message the reader shows | the composition lives inside `open_in_the_text_reader`, which three readings in `tests/wired.rs` name; extracted, and the readings rewritten in place | 13-03, 13-04 |
| RESEARCH-2 | S/MIME calls through the `windows` crate | `signed_mail.rs` declares its calls in its own `#[link]` block, the file's stated convention | 13-14, 13-19 |
| RESEARCH-2 | Google's `iCalUID` and Graph's `iCalUId` assumed | read from each provider's own reference page; a Graph occurrence's UID is not its series' | 13-12 |
| RESEARCH-2 | answer buttons before the page in the formatted window | a guarded rule gives the browser the keyboard first, so the buttons sit after it | 13-11 |
| RESEARCH-2 | the backlog row at `:102` | `:104` | 13-13 |
| RESEARCH-2, R2-08 | the key manager on Tools; GAP-03's `[D]` line says Tools | Pratik moved it to File on K; 13-17 corrects the `[D]` line | 13-17 |
| RESEARCH-2 | fix the `cms` comment in 13-14 | any `Cargo.toml` change beyond the version line runs the whole gate, and 13-20 changes the manifest anyway | 13-20 |
| RESEARCH-3 | add both scopes to the shared token list | an older sign-in whose refresh lacks a new scope would stop four unrelated features; each new permission gets a token of its own | 13-28 |
| RESEARCH-4 | a stored order for saved searches | the folder tree shows runnable searches first, so a stored order alone would record a move the tree never shows; the order is carried into the tree | 13-37 |
| RESEARCH-4 | buttons reading Run and Don't Run | wxDragon 0.9.17's `MessageDialog` cannot relabel Yes and No; the question ends "Run it?" | 13-44 |
| RESEARCH-4 | the letter check reads each menu | `tests/wired.rs` finds a menu's letters by its variable name, so a rebuild function keeps the builder's name | 13-38, 13-42 |
| RESEARCH-4 | the key check accepts new keys | it accepts a counted key only through its `stated` list, which 13-38 and 13-42 extend | 13-38, 13-42 |
| RESEARCH-4 | the Quick Step Manager on Tools | Pratik settled it inside Action, Quick Steps | 13-41 |
| RESEARCH-5 | the first red test in `export_tree` | that module's header says it never touches a file or the database; the loop and the test go to a new `application::exporting_mail` | 13-45 |

## Costs every plan is written around

**Guard records, 1,088 by the TOML reader on 2026-09-24.** The files most
plans touch, with the records naming them in `tests_last_seen` and their
test count by the count check's own rule: `src/presentation/wx_app.rs` 119
and 199; `src/application/contacts_sync.rs` 76; `src/presentation/managers.rs`
54 and 137; `src/data/message_cache/messages.rs` 28 and 183;
`tests/house_style.rs` 29 and 74; `src/presentation/wx_settings.rs` 26;
`tests/wired.rs` 19 and 77; `src/presentation/wx_managers.rs` 14 and 44;
`src/data/message_cache/mod.rs` 12 and 23; `src/presentation/wx_compose.rs` 8
and 43; `src/presentation/ui_types.rs` 6 and 79. So no plan adds a test to
`wx_app.rs`, `contacts_sync.rs` or `managers.rs`; new readings go in new
integration targets or new modules at zero records, each with a record
whose `suite` names it. Every plan lists its records by both readings,
`tests_last_seen` and `file`, and the anchor text inside the regions it
edits, and quotes counts as the count at its start plus what it adds.

**The gate.** A branch commit runs what reaches the change; a merge made
with `git merge --no-ff` runs what the branch's whole diff earns; the whole
suite, the release build and the audit run once, in 13-51's task 3, by
hand, with `main` unmoved since its branch was cut. Three plans change
`Cargo.toml` (13-03, 13-20, 13-47), and each of those commits pays the
whole gate. Six change a workflow under `.github/` (13-03, 13-17, 13-27,
13-33, 13-39, 13-41), which answers `all` too.

**Carried from phase 12's README into every brief:** trace an absence claim
with a pattern that tolerates a line break; run the `--remeasure` remedy
whenever the count check prints it; never pipe `check.sh`; red trailers
name lib tests by module path and integration tests bare, on a branch,
never on `main`; `WIXEN_TEST_THREADS` untouched; a changelog entry in the
same commit as a user-visible change; every key in
`docs/KEYBOARD_SHORTCUTS.md` in the same commit; carriage returns measured
with `tr -cd '\r' | wc -c`; no em dash; none of the six words; **no
scripted rewrite of a tracked file, Read then Edit or Write**; commit
messages from a file with `-F` and no AI attribution, whatever the harness
says; the NVDA tests never run on this machine; the four completion marks.

**Three planners broke the scripted-edit rule while writing these plans**,
each once, each saying so in its return: a Python replace in a plan
(observation 0825), a Python line fix in 13-12, and four `sed` ranges fixed
by Python in 13-48. The other two ranges reported no break, and the
integration made its changes with Edit and Write only. Every plan and this
README read 0 carriage returns with `tr -cd '\r' | wc -c` when the plans
landed. Keep the sentence in every executor's brief.

## What only a person or a real provider can settle

Each requirement's last `[S]` line names it, and no plan claims it. The
tester's ear: every new menu item, key, letter, question and sentence, the
greyed Undo read as unavailable, an undo naming the item, the invitation
before the body and its buttons, the passphrase question, the key manager,
the Quick Step Manager, a rule's question and its count. A sighted reader:
a printed page. The runner: the Accessibility scan on each new window
(`pgp-keys`, the directory sign-in, `identities`, the step editor), the
spool target on CI with no PDF printer. His accounts: an invitation from a
real organiser and its answer; a message from a real correspondent's key; a
junk report reaching Gmail; a block moving mail at a real server; a real
directory; a real guest's free/busy; a shared mailbox; a `.msg` saved from
his Outlook. Everything a built window, a fixture, a cache built in the
test or a reading can prove, the plans prove.

## Estimates, and the factor behind them

`raw_tokens` is each planner's, 5,540,000 over the 53 plans. The `tokens`
field is not on one basis: 13-22 to 13-36 multiplied by 0.32, phase 12's
factor, and the other plans left `tokens` equal to `raw_tokens`. The factor
from phase 12's sixteen landed plans, each summary's `actuals.tokens` over
its plan's `raw_tokens` read on 2026-09-24, has a mean of **0.407** (0.116
for 12-12 to 1.183 for 12-05), so the phase projects at about 2,250,000
tokens (derived: 5,540,000 times 0.407). The fields were left as each
planner wrote them rather than rewritten across 53 files; read `raw_tokens`
and apply the factor here.

## What is owed to documents, and who does it

1. **The roadmap's phase 13 entry, plan list and progress row.** Done in
   the commit that lands these plans: the Plans line, a `- [ ]` line for
   each of the 53, the row at `0/53`, the phase line and the milestone
   paragraph brought forward.
2. **`.planning/REQUIREMENTS.md`.** Done in the same commit: a paragraph at
   the head of the phase's section and a planned line under each GAP
   requirement naming its plans, and each traceability row.
3. **`.planning/STATE.md`.** Done in the same commit, by hand: phase 13
   current, plan 0 of 53, 13-01 next, `progress.total_plans` 225 from
   `ls .planning/phases/*/*-PLAN.md | wc -l`, `completed_plans` 172 from
   the same over `*-SUMMARY.md`.
4. **Each plan's four marks**: the roadmap's row, its own line in the
   roadmap's plan list, `STATE.md`'s plan number in the frontmatter and the
   body, and its requirement's lines and traceability row. This README
   keeps no boxes (decision 51).
5. **The changelog, the pages and the ledger**, by each plan in its own
   commits; 13-51 reads them as one.

## Handover

Nothing in phase 13 has run. **The next plan is 13-01**, Undo and Redo on
the Edit menu, then the rest in wave order to 13-51 at wave 53.

What an executor's brief needs before its plan starts:

- **13-03** needs Pratik's answer to (b), the three `windows` features, or
  it stops at its checkpoint. **13-20** needs his answer on `rand` 0.8,
  also (b). **13-28** needs (c). **13-47** needs (a). No answer to any of
  the four is recorded in these plans; asking him all four before 13-03
  starts saves three stops later.
- Every brief says: no scripted edit of a tracked file; no AI attribution
  in any commit or pull request; the red and green on a branch; `main` is
  pushed only on his word; plans that change what is spoken or shown push
  their branch and open a pull request, and the others do not.
- 13-16's executor reads decision 50 before its second red commit.
- 13-42 and 13-44 read 13-24.1's summary for the runner's names as built;
  where the summary's name differs from the plan's, the summary's is the
  one every criterion follows.

`main` was pushed at `630e2a67`, so ledger 609's premise ("main has not
been pushed since 6cb8f17c") no longer holds; 13-51's premises name it to
close or reword. The commit that lands these plans is on `main` and not
pushed.

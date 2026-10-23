# Phase 13.3: The review's fixes

Twenty-five plans, one per wave, written on 2026-10-20 against `main` at `6fd465c6`, 14-08.1's
merge, version `1.0.0-alpha.1`, inserted between phase 13.2 and phase 14 the way GSD inserts a
decimal phase. Phase 14 stood at 9 of 15 plans with none of its sittings held, and 14-08.2 was
stopped at its install checkpoint: build `1.0.0-alpha.1+1403.g6fd465c6` and an install page for
it are committed on branch `14-08.2-the-new-build` (`36baca49`), not merged, and Pratik has not
installed it. On the day, `guards/guards.toml` held 1,640 records by the TOML reader
(`python -c "import tomllib;g=tomllib.load(open('guards/guards.toml','rb'))['guard'];print(len(g))"`),
`.planning/WINDOWS.md`'s frontmatter said 912 entries, 763 open and 149 fixed, the last whole
gate was phase 13.2's, 11,124 tests at `b1fab9ba`, and `git rev-list origin/main..main --count`
answered 332, with this clone's `origin/main` at `01ef4589`. The build Pratik has installed is
`1.0.0-alpha.1+1118.gd3455792`, made from 14-07's merge on 2026-10-06.

The directory is named the way `gsd-tools generate-slug "The review's fixes"` answers,
`the-review-s-fixes`.

**Goal.** Every one of the 42 items Pratik said yes to on 2026-10-20 is fixed as its checkers'
notes correct it, in the build he installs next: a delete, a move, a mark or a star made here
shows at once and reaches the right account's server, in the right order; Reply and Forward
from a conversation never quote nothing in silence; the download of everything leaves new mail
to the check and its rules; an account removed and added back saves; a feed names only its
host and reads past a byte order mark; no sync throws away what a person did meanwhile; an
import says what it left out; the Google sign-in compares the account that signs in; markup at
any depth has its hidden blocks dropped; the uninstaller and the search guide say what is true;
a draft's older copy is mentioned once; and the check suite costs what its questions need.
Pratik's word on the update mechanism is asked first, and the research's five gaps in the
update code are fixed as he answers. The phase closes on its own full gate.

**Requirements.** No requirement is added and the coverage stays at 123. Twenty-two plans name
the requirements they work under, nineteen between them, and each adds a dated line under its
requirement, adding no `[D]` clause and ticking no box: FOUND-04, FOUND-11, FOUND-23, MAIL-01,
MAIL-02, MAIL-04, MAIL-05, LIST-03, LIST-04, LIST-05, LIST-21, LIST-25, THREAD-01, FOLDER-01,
PIM-04, PIM-05, SCALE-06, SHIP-02 and REAL-01. Three plans name none and each says why: 13.3-09
(no requirement names subscribed feeds, opened calendar files or contact cards), 13.3-20 (none
covers what an uninstall says) and 13.3-22 (whether a function nothing calls ships is not
something a requirement describes). Those three leave three completion marks, not four. REAL-01
stays phase 14's to tick: 13.3-15, 13.3-17 and 13.3-18 add lines under it and leave its box
open.

## Pratik's yes of 2026-10-20

A review of phase 13.2's changes, `2a4679a9..0b781473`, found 40 findings, each confirmed by at
least two of three checkers: 15 graded must fix, 16 should fix and 9 nice to have. They were put
to Pratik on 2026-10-20 with two more items: ledger 912, the sentence a server without UIDPLUS
hears at every automatic save, and the shell check suite's slowdown, `scripts/check.test.sh`
taking 187 s alone that day. He answered "yes" to fixing all 42 as one phase, before the build
he installs and before `main` is pushed.

The list he was shown is `review-2026-10-20-findings.json` in this folder, a byte-for-byte copy
(GROUPING.md says how it was checked), key `findings`, 42 entries numbered 1 to 42 in the field
`number`. Read it with Python's `json` module, one entry at a time; it is 410,217 bytes and is
never loaded whole into a plan's context. `GROUPING.md` maps every item to its plan, explains the
order, and digests where the checkers corrected an item's own fix; each plan works from the
corrected fix, read again on `main` by command with file and line in its `premise_corrections`.

The same day he asked to "consider" ship-shape for the update mechanism, and said of it "It's a
published crate." `RESEARCH-the-update-mechanism.md` in this folder compares it with the update
code Wixen Mail has. Every package change needs his word, so 13.3-23 opens with a blocking
checkpoint for it, after the recommendation in plain words (see "What waits on Pratik", item 6).

## The plans

"Who acts" says whose hands each plan needs. "Pushed" is whether the plan's branch goes out
with a pull request under Pratik's standing OK of 2026-09-23, which a plan uses when it changes
what is spoken or shown, so the NVDA job runs before the merge. Every plan merges with
`git merge --no-ff`, and none pushes `main`. Items are numbered as in the findings file. The
ledger column says what each plan plans to do to `.planning/WINDOWS.md`; new rows are numbered
by the executor at commit time, after 912 and anything a running plan has taken.

| Plan | Wave | What it does | Items | Requirement | Who acts | Ledger | Pushed |
|---|---|---|---|---|---|---|---|
| 13.3-01 | 1 | The check suite's time measured and its own share cut; a scoped run reads the guard registry once and runs every library filter in one cargo call; the registry scan compared with a TOML reader on every commit that stages the registry | 42, 30, 35, 36 | FOUND-23 | The executor; a setting of this machine, if the measurement names one, goes to Pratik as a question | 803 brought up to date, closed only if its condition holds; 892 a dated line | no |
| 13.3-02 | 2 | A delete on this computer takes its row out in the same call, so a reopened conversation no longer offers it to a second Delete; a conversation row changes at once after a POP delete, a take-off or a move to another account, for one message and a set | 1, 15 | LIST-21, LIST-05 | The executor | One for the ear (POP in conversation view) | yes |
| 13.3-03 | 3 | The folder tree's counts follow a change made here whichever account drew the tree, rows paired by what they are; a check no longer puts back the count of a mark still waiting | 2, 7 | LIST-04, FOLDER-01 | The executor | 836 a dated line; 899 widened, or one for the ear | yes |
| 13.3-04 | 4 | A read, star or label change, and every other command aimed at a message, reaches the server of the account the message is in; a change sent during a check's replay waits for it, and a change let go is never sent | 3, 18 | LIST-04, LIST-21 | The executor | One for the ear (two accounts); 563 a dated line | yes |
| 13.3-05 | 5 | Reply and Forward from a conversation say when the message's text is not here yet, start its download and open no empty compose window; Forward stops working out a reply address | 4, 37 | LIST-05 | The executor | One for the ear; two `todo` (the list's placeholder quote, Forward with nothing chosen) | yes |
| 13.3-06 | 6 | The download's re-read of the open folder is applied in the tick that reads it, so it cannot undo a mark or a delete | 5 | MAIL-01, LIST-21 | The executor | One for the ear; 851 a dated line | yes |
| 13.3-07 | 7 | Removing an account and adding one back on the same address saves: every removal before every save, no refusal stopping the rest | 6 | MAIL-04 | The executor | One for the ear; two `todo` (the window's accounts after a refusal, tokens left by a refused sign-in) | no |
| 13.3-08 | 8 | The download of everything leaves new mail for the check, so rules run on it and it is said, and calls no folder stopped for it; a filed message with no Message-ID, or a mismatched one, never takes another message's number | 8, 9 | MAIL-01, MAIL-02, MAIL-04, MAIL-05 | The executor | One for the ear; 850 a dated line | yes |
| 13.3-09 | 9 | A feed that fails to arrive names only its host; a feed, calendar file or card starting with a byte order mark reads again, and a declared charset is honoured through `mail-parser`'s decoder, no package added | 10, 11 | none | The executor | One for the ear | yes |
| 13.3-10 | 10 | A CardDAV answer cut short, half read or unsplittable takes nobody off; a contact deleted at the address book while edited here goes with the change, said, instead of failing on every sync | 12, 24 | PIM-05, SCALE-06 | The executor | 255 a dated line | no |
| 13.3-11 | 11 | A note edited, pinned, moved or deleted while a notes read runs keeps what was done, decided and written in one transaction | 13 | SCALE-06, PIM-04 | The executor | None | no |
| 13.3-12 | 12 | An import says which files and folders it left out at the first look, none said twice; a damaged packed zip entry gets the damaged sentence | 14, 26 | FOUND-11 | The executor | 906 and 908 dated lines; one for the ear | yes |
| 13.3-13 | 13 | In the message view, a message opened from a conversation window turns read after the wait, through a reading that remembers where it began | 16 | LIST-03 | The executor | 841 a dated line | yes |
| 13.3-14 | 14 | A change over many conversations and landing on one cost what they touch: rows looked up by pair, no wasted scan in a set leaving, one conversation's missing text read through the narrowed scope, timed before and after | 17, 32, 33 | LIST-04, LIST-21, THREAD-01 | The executor | One `todo` (`mark_these_read` and `star_these` scanning per message) | no |
| 13.3-15 | 15 | Both Google sign-ins ask who signed in through the sign-in permissions nobody can untick, and compare and name the account's Username; Help, the privacy page and two sitting sheets follow | 19, 20 | REAL-01 | The executor; Pratik's key may need two permissions listed, his step at phase 14's first sitting | 869 a dated line; one for the ear; one `todo` if the calendar sync names a wrong cause | yes |
| 13.3-16 | 16 | A run of replayed changes opens its folder once, from the numbering the session kept; a refused SELECT leaves no folder taken to be open; a check's rule moves are moved and settled one destination at a time | 21, 34 | MAIL-02, LIST-21 | The executor | One for a real server | no |
| 13.3-17 | 17 | A calendar read asks only its own calendar's deletion notes, for feeds, calendar servers, Google and Outlook; the store's four event lookups take the shape its siblings use | 22, 40 | REAL-01 | The executor | 825 widened and fixed; 904 a dated line; one for the ear | no |
| 13.3-18 | 18 | A contact edited or deleted while Google, Outlook or a CardDAV book makes or updates it keeps what was done; the Contact Manager keeps names a sync gave; a refused contact is logged by its id | 23, 25 | REAL-01, PIM-05 | The executor | One for the ear; one `todo` (the write gate's refusal counted as an error) | no |
| 13.3-19 | 19 | Markup nested past 512 levels has its hidden blocks dropped like any other, through an iterative walk that cannot overflow the stack | 27 | LIST-25 | The executor | 859 corrected and fixed; one `todo` (the text reader reads hidden blocks); one for the ear | yes |
| 13.3-20 | 20 | The uninstall messages say the mail folder may still be there, the comments say what Inno does, and the Windows Search guide says who runs `remove-scope`, held to the script by a reading | 28, 29 | none | The executor | One for the ear, on a throwaway install | yes |
| 13.3-21 | 21 | On a server without UIDPLUS a draft's older copy is mentioned once per draft and flagged copies are left out of the search; a sent draft's copy here goes in one statement | 41, 38 | FOUND-04 | The executor | 912 fixed; one `todo` (the offline save said every minute); one for a server without UIDPLUS | yes |
| 13.3-22 | 22 | Two functions only tests call are compiled for tests only, where they stand, each attribute demanded by a reading that is red first | 31, 39 | none | The executor | None | no |
| 13.3-23 | 23 | Pratik's word on the update mechanism at a blocking checkpoint; then the update check downloads nothing it could not run, says what a check at start found without a window, runs one check at a time with a folder per download, and gives up on a stalled connection | none; the research's gaps 1, 3 and 4 | SHIP-02 | Pratik answers at task 1; then the executor | One for the ear | yes |
| 13.3-24 | 24 | Installing an update quits the way Quit does whatever the tray setting; the setting names the installer's real size, about 15 MB, held to a dated measurement; the stale comments and pages corrected | none; the research's gaps 2 and 5 | SHIP-02 | The executor, on any answer but (d) | One deviation opened and fixed; one for the ear; 335 and 339 amended | yes |
| 13.3-25 | 25 | The closing read of all 42 items and the five gaps, the listening lines, the planning files, the hand-over to 14-08.2, and the phase's full gate, `scripts/check.sh all` once by hand on its branch before its merge | none | FOUND-23 | The executor drafts and runs the gate | What the read finds | only if the closing read changes the changelog |

## One plan per wave, and why this order

One per wave because every plan writes `.planning/ROADMAP.md` and `.planning/STATE.md` for its
completion marks, all but three write `.planning/REQUIREMENTS.md`, and nearly every plan writes
`guards/guards.toml`, `.planning/WINDOWS.md` and `docs/changelog.md`; a wave is a set of plans
that share no file, and these share those. Each plan depends on the one before it. Several plans
also edit the same source files in turn (`GROUPING.md`, "What a plan must read from a sibling"):
eleven edit `src/presentation/wx_app.rs`, two edit `src/application/contacts_sync.rs` and
`carddav_sync.rs`, three edit `src/service/protocols/imap.rs`, and each later one names the
earlier ones' changes in its premises and moves its guard anchors by both readings.

- **13.3-01 first:** a slow gate slows every later commit, and the check suite runs on every
  commit that stages the guard registry, which is nearly every green commit here.
- **13.3-02 to 13.3-12:** the plans carrying a must item, in the order GROUPING.md gives. The
  list, the folder tree and the server first (02 to 04), then quoting, the open folder, accounts,
  the check's rules and feeds (05 to 09), then the contact and note reads and the import (10 to
  12).
- **13.3-13 to 13.3-22:** the should and nice items, a plan carrying several grades sitting at
  its highest.
- **13.3-23 and 13.3-24:** the update mechanism, after the research and after every other code
  plan, so Pratik's word is asked as late as it can be and nothing earlier waits on it.
- **13.3-25 last,** at the highest wave: the closing read and the phase's full gate.

GROUPING.md reserved 13.3-23 and 13.3-24 for the update plans and 13.3-25 for the closing plan.
If Pratik answers (b) at 13.3-23's checkpoint, 13.3-24.1 and 13.3-24.2 go in before 13.3-25 and
whoever lands them moves 13.3-25's wave and `depends_on`; on (d), 13.3-23 commits nothing and
13.3-23 and 13.3-24 move after phase 14's sittings.

Added 2026-10-22 by 13.3-20: the plan that moves the lock past ledger 914's two advisories,
RUSTSEC-2026-0382 and RUSTSEC-2026-0383, takes both out of `awaiting_a_decision` in
`scripts/audit.sh` in the same commit. 13.3-20 held them open there because its installer
commit earned the whole gate on this machine and the audit refused on them; once the lock
moves, the gate refuses until the list is emptied, because a held-open advisory nobody reports
silences nothing.

## What waits on Pratik

In the order the plans meet them. If he answers early, his words are written here, dated, and
the plan or its checkpoint quotes them rather than asking again.

1. **Any time: each plan's choices** under "Decisions the planners took" below, any of which he
   may overrule. The ones most worth his look: 13.3-13 (the checkers' corrected fix, where one
   checker called today's behaviour a design choice and offered a changelog line instead),
   13.3-05 (Reply refuses as well as Forward when the text is not here), 13.3-10 (a contact the
   address book deleted while edited here goes, with the change, and it is said), 13.3-17 (the
   narrowing reaches Google's and Outlook's reads too, so ledger 825 closes) and 13.3-21 (Send's
   second try on a server without UIDPLUS forgets a flagged copy without a second sentence).
2. **13.3-01: a setting of this machine,** only if the measurement names one as part of the
   check suite's cost (an antivirus exclusion, a setting of Git's bash). Nothing on the machine
   is changed; the question goes in its summary and in ledger 803 with what it would cost and
   buy.
3. **13.3-09: nothing.** GROUPING.md expected a question about making `encoding_rs` a direct
   dependency for a feed's declared charset. The planner found that `mail-parser`, already a
   direct dependency, exposes its charset decoder, so no package changes and nothing waits.
4. **13.3-15: nothing in the plan.** His Google testing key may need the two sign-in
   permissions, `openid` and the email address one, listed on its Data Access page before Google
   hands them out. That is his step at phase 14's first sitting, which Help's page will describe;
   the plan does not wait for it.
5. **13.3-22: nothing.** The plan first went ahead without a red commit and asked him in its
   summary, after the work. A plan checker on 2026-10-20 pointed out that CLAUDE.md asks first,
   and the task turned out to need no exception: each attribute is now demanded by a reading
   through `what_ships` that is red first and goes red again if the attribute is taken off.
6. **Before 13.3-23's task 1, a blocking checkpoint: how Wixen Mail updates itself.** The
   recommendation, in plain words: do not adopt ship-shape. It is a real, published crate,
   carefully made, by the author of other screen reader crates for wxDragon, but it cannot offer
   his test builds (it treats `1.0.0-alpha.1`, `1.0.0-alpha.2` and `1.0.0` as the same version,
   and the way it asks GitHub never sees a test build), it needs wxDragon moved from 0.9.17 to
   0.9.22 or later, which changes what NVDA reads and needs its own sitting, and its windows were
   not built for a screen reader. Keep Wixen Mail's own update code, which checks who signed an
   installer more strictly and says every outcome, and fix its five gaps now with no package
   change (option c). The other answers: (d) change nothing now and fix the gaps after the
   sittings; (b) add one small package, `minisign-verify`, as a second signature check, which
   means a second key he keeps for good and two more plans with checkpoints of their own; (a)
   adopt ship-shape, as its own phase after the sittings, starting with the wxDragon move. Two
   questions that change no plan: when wxDragon should move (recommended: its own phase after
   the sittings), and whether he tells ship-shape's author about test versions (his to do;
   nothing here posts anything).
7. **Rows opened as `todo` for him to schedule,** none blocking the phase: 13.3-05's two (the
   list's placeholder quote, Forward with nothing chosen), 13.3-07's two (the window's accounts
   after a refused save, tokens a refused browser sign-in leaves), 13.3-14's (the per-message
   scans in `mark_these_read` and `star_these`), 13.3-15's if the calendar sync names a wrong
   cause, 13.3-18's (the write gate's refusal counted as an error), 13.3-19's (the text reader
   reads blocks a sender hid) and 13.3-21's (the offline save said every minute).
8. **After 13.3-25's merge: installing the new build** that 14-08.2 makes from that merge, over
   the build he has and never uninstalling first, 5 to 10 minutes and one line back. Build 1403
   is never installed (see the Handover).
9. **Pushing `main`** stays his word.

## The version

The version does not move in this phase, by the rule phase 13.2's README sets out under "The
version", which every plan here follows: `Cargo.toml` reads `version = "1.0.0-alpha.1"`, no tag
exists, and a tester build is not a cut. No plan edits `Cargo.toml` or `Cargo.lock`, and no plan
adds or moves a package; every plan checks `git diff main -- Cargo.toml Cargo.lock` is empty at
its merge. If Pratik answers (b) at 13.3-23's checkpoint, the package it names is added by a plan
of its own after his word, and that plan's summary says so.

## Decisions the planners took

Taken 2026-10-20 where an item or its checkers left a choice open; Pratik may overrule any of
them. The full reasoning is in each plan's `decisions` block or premises.

- **13.3-01:** the registry comparison lives in the check suite and calls `python -I`; the scan
  is widened to the two spellings the checkers proved it misses; no setting of this machine is
  changed; the one library call keeps `--test-threads=4`.
- **13.3-02:** the review's fix for item 1 as the checkers corrected it, the counts list moved
  in the same commit; `the_named_message` left as it is; item 15's conversations named where the
  set is remembered, before the change, rather than a thread kept on a taken-off row.
- **13.3-03:** rows paired by identity, never replaced, with the fresh reading taken for the
  account whose labels the tree shows; only waiting changes to read are taken off a check's
  count, and the one case that remains is said.
- **13.3-04:** `owner_of` asks the store for every caller, not the two the review named; one
  flag turn per account as checker 3 proposed, with the review's narrower fix beside it.
- **13.3-05:** Reply refuses as well as Forward; the download starts while refusing, and not
  offline; the list's placeholder quote is ledgered rather than changed.
- **13.3-06:** the due block moves above the drain rather than draining twice; the reading
  widened in place, so no record is flagged by count.
- **13.3-07:** every removal before every save; a refused save no longer stops the saves after
  it; the window's copy after a refusal and tokens a refused sign-in leaves are ledgered.
- **13.3-08:** the download leaves new mail for the check rather than running rules itself; mail
  left for the check is not counted as missing; the settle confirms the one number found, at
  one header fetch per settled move.
- **13.3-09:** the declared charset is read again, reversing 13.2-14's written limitation;
  `mail-parser`'s decoder rather than a new dependency; the byte order mark taken off where bytes
  become text.
- **13.3-10:** a contact deleted at the address book while edited here is taken off with the
  change and said; a 507 block is its own fact and sentence; other spellings of a response count
  as unreadable.
- **13.3-11:** the read decides on the note as it stands, inside one immediate transaction; a
  moved or deleted note is left to the next push.
- **13.3-12:** three sentences for what the first look left out; the decoder's failure is damage
  on a zip's reads alone.
- **13.3-13:** the checkers' corrected fix, a reading that remembers the conversation it began
  in; the cursor may rest on any message of that conversation.
- **13.3-14:** a reading of the source as the red half for items 17 and 32, with a timing by hand
  before and after for item 17; a budget of 100 ms at 5,000 touched conversations;
  `mark_these_read`'s own scan ledgered.
- **13.3-15:** the Username asked for and compared, the address only when the Username is blank;
  `openid` and `email` asked and the address read from Google's userinfo service; Outlook's login
  hint follows.
- **13.3-16:** the numbering kept on the session beside the open folder; a refused SELECT clears
  what is taken to be open; rule moves grouped by destination, one group finished at a time.
- **13.3-17:** the narrowing reaches every calendar read; a note naming no calendar still counts
  everywhere; item 40 is a refactor under existing cases with no red commit.
- **13.3-18:** all six places that write a contact down after an address book answered take the
  re-read; "changed meanwhile" is the row compared whole; the Contact Manager half is taken; the
  gate refusal is ledgered.
- **13.3-19:** the walk made iterative and the bypass deleted; the text reader's own gap
  ledgered.
- **13.3-20:** the messages hedged ("may still be") because Inno names the installing account's
  folder; the all-users message and the installing page get the same hedge; the guide held to
  the script by a reading.
- **13.3-21:** `UNDELETED` on the one search every caller shares; "once per draft" means once
  for each draft while the program runs; a blank identifier removes nothing here.
- **13.3-22:** both functions keep their place under `#[cfg(test)]`, each demanded by a red
  reading of its own file through `what_ships`, with a guard record for the attribute.
- **13.3-23:** option (c) planned after his word; a folder of its own per download, taken away
  with its file by a failed download, a refusal and a No; 30 seconds on
  the check, 30 to connect and 30 of silence on the download; a check at start said at normal
  priority under its own topic. On (b), the minisign check and the release side are new plans,
  13.3-24.1 and 13.3-24.2.
- **13.3-24:** the handover goes through the function Quit uses; the reading homed in
  `closing.rs`; "about 15 MB" rather than "more than 15 MB"; one ledger row opened and closed.
- **13.3-25:** written when the plans were brought together, on 13.2-32's pattern.

## Corrections made when the plans were brought together

Made on 2026-10-20 on branch `13.3-plans`, after all twenty-four planned plans were on disk,
each edit made in the plan it names.

- **13.3-25 written.** GROUPING.md reserved the id for the closing plan "written last" and no
  planner was given it, while every other plan named 13.3-25 as the one carrying the full gate.
  It is written on 13.2-32's pattern.
- **The findings file out of four plans' context.** 13.3-19, 13.3-20, 13.3-21 and 13.3-22 listed
  `review-2026-10-20-findings.json` in `<context>`, which loads all 410,217 bytes into the
  executor. Each now reads its items by number, as the other plans do.
- **Premises a later plan needed from an earlier one,** each added as a dated paragraph:
  13.3-08 (13.3-06 edits the download's test target and its records); 13.3-13 (13.3-05 adds two
  tests to the conversation window's target, which then holds 29, and 13.3-04 and 13.3-08 edit
  the window too); 13.3-14 (13.3-02 adds cases to the conversation rows' target); 13.3-16
  (13.3-08's settle confirms the one number with a header fetch, and its `Scripted` learns what
  the destination holds); 13.3-18 (13.3-10's cases, records and push fixtures in the two contact
  syncs); 13.3-21 (13.3-16's cases in `mail_controller.rs`, 13.3-04's flag turn around the
  replay that reaches the search, 13.3-08's confirmation); 13.3-23 (13.3-05 leaves `ui_types.rs`
  at 76 tests, 13.3-06 and 13.3-08 edit the window).
- **14-08.2 re-pointed** to build at 13.3-25's merge, with what happens to the branch holding
  build 1403's page (see the Handover); phase 14's README says the same.
- **GROUPING.md** keeps what it said, with a dated note that 13.3-09 needs no package and that
  the phase was inserted into the roadmap and the state file.

## What the bringing together could not settle

- How many cases 13.3-02 adds to `tests/a_change_made_here_reaches_the_conversation_rows.rs`,
  which 13.3-14 counts from; it names four as examples, and 13.3-14 takes the count again.
- Whether 13.3-08 leaves `mail_sync.rs`'s `Scripted` recording `the_identifier_at` in its list of
  calls, which decides the order 13.3-16's case asserts; 13.3-16 reads it first.
- Which answer Pratik gives at 13.3-23's checkpoint, which decides whether 13.3-24.1 and 13.3-24.2
  exist and whether 13.3-23 and 13.3-24 stay in this phase.
- 13.3-01 to 13.3-04 lead with an ordinary task where the later plans lead with a tracer. Phase
  13.2's plans used no tracer at all, so both shapes are this project's, and nothing reads the
  difference.

## What only a person or a real provider can settle

- Every sentence the phase changes and nobody has heard: the pull requests run the NVDA job,
  which hears only what its cases wait for, and 13.3-25 writes a listening line for each.
- What Gmail, Microsoft, the CalDAV and CardDAV servers and a server without UIDPLUS do with the
  corrected paths: each plan ledgers its real-account half, for phase 14 or later.
- Whether Google asks for the two sign-in permissions on a key in Testing (13.3-15).
- Every path of the update code that a published release or a signed installer reaches: none
  can be heard until a release exists and the signing account does (ledgers 315, 329 and 335).

## Handover

13.3-01 is next, on a branch cut from `main` at the commit landing these plans. Nothing needs
Pratik before 13.3-23's task 1, apart from the answers above, which he may give any time.

Phase 14 resumes after 13.3-25's merge, at 14-08.2, which builds at that merge with
`scripts/build-installer.sh` on Pratik's ruling of 2026-10-06 and redoes the one install page for
the new build. The page committed on branch `14-08.2-the-new-build` at `36baca49` was written
for build `1.0.0-alpha.1+1403.g6fd465c6`, made at `6fd465c6` before this phase, which holds none
of it, and which Pratik never installed; `git merge-base --is-ancestor` answered on 2026-10-20
that the branch is not merged into `main`. 14-08.2's executor renames it to
`14-08.2-build-1403-superseded` with `git branch -m`, keeps it unmerged as the record of that
page, reads the page with `git show 36baca49:...` as a draft, and cuts `14-08.2-the-new-build`
again from `main` after the new build. The setup file for build 1403 stays in `dist` beside the
two older ones and is never installed or run. Then Pratik installs the new build, over the one
he has and never uninstalling first, and 14-10's sitting follows from branch
`14-10-sitting-2`, its sheet corrected against the new build by its own executor.

---
gsd_state_version: 1.0
current_phase: 6
current_phase_name: How the application speaks
current_plan: 3
status: executing
stopped_at: 06-03 tasks 1 and 2 of three merged, version 0.121.0, and the plan is partial on purpose. Month names in a date now come from the machine. Two Win32 mechanisms rather than one, because a month inside a date and a month in a list are different words in Russian, Polish, Czech and Lithuanian: a date goes through GetDateFormatEx with a picture holding a numeric day and MMMM, which is the only way Microsoft's own pages say the genitive arrives, and a bare list of twelve goes through GetLocaleInfoW with LOCALE_SMONTHNAME1 to 12. Measured here by a passing test rather than taken on the documents' word: a Russian date reads 2 января and a Russian month list reads Январь, which are different words and not different capitalisation. The shape stays the person's, because their stored wording and order pick the picture and Windows supplies only the words, so month first on a French computer gives juillet 26, 2026, which no French computer writes on its own. The wrapper is in src/common/ rather than beside the other date code, because src/service/signed_mail.rs will need it in task 3 and src/service/ reaches presentation nowhere today. The English ordinal is gone and ordinal with it, removed rather than silenced, since a date picture cannot produce 14th and date_part never had one. Criterion 2 does not close and FEEDBACK-02 does not close: read from ROADMAP.md clause by clause it has four clauses, and month names close only in date_display's three readings and the appointment form's month list, not in the eight signature sentences still writing %B at signed_mail.rs:1373; day names do not close at all at occurrences.rs:701; relative wording is this plan's own blocking checkpoint, left unanswered on purpose. The fallback clause closes and is wider than it asks, because a day no month has is refused by Windows and comes back English even where the language exists. A previous executor was terminated mid-task with task 2 staged and uncommitted; the staged work was read before being built on and kept, because its red half was stubbed with the plausible wrong mechanism rather than with nothing, so it failed for the reason the task exists. One thing in it was corrected: the reworded ENGLISH_ONLY claimed the month names follow this computer while the signature sentences still write English ones, a claim its own doc comment two lines above did not make. Three tests were really red where one was expected and two were foreseen, and the third is the interesting one: a guard record naming code that a rename moved, caught by test_every_guard_record_still_names_one_place_in_the_tree, which is a different check from the one that counts tests and a failure mode CLAUDE.md describes one step over, about test names rather than code. It was named in the trailers rather than fixed first, because correcting a record means measuring the guard by hand against the code that replaces it, and that code is the green half; the correction was then made and all three date_display records re-measured, each moving from 37 tests to 41 and each still reddening exactly what it names. Running scripts/check.sh affected by hand is not the run the hook makes: the scoped module runs come from the index the hook hands over and a bare invocation hands over nothing, so that run found two of the three and would have produced a red commit naming the wrong set. The appointment form's month list is wired and untested, because the Choice is built inside a closure and needs a live window. WINDOWS.md 360 to 364, five entries, one per unrun thing, and nobody has heard a localised date in any language. The earlier entry, still true. 06-02 merged, all of it, version 0.120.0, and it is the first plan of this phase a person meets. The sixteen per-event answers that have been in the settings file all along are reachable from a screen: a picker read from `Event::ALL`, three controls, a button that removes an answer rather than emptying it, and two lines. The ticks are painted from `what_was_chosen_for` and the first line from `channels_for` because those are different questions, and the second line is what keeps the ticks honest by saying what the event will really produce, since a channel switched off everywhere stays off and an event left with only a sound has a written channel added back. The two global boxes for speech and braille are one, built from `Switch`, whose three answers name all four channels between them because a notification rides one `UiaRaiseNotificationEvent` that takes no medium parameter. Criterion 1 does not close and it has five clauses rather than the four 06-01's summary names: that reading folds "by keyboard" into the first, which is the clause this plan can least attest to, since every control is native with a distinct mnemonic by reading and nobody has tabbed through the tab. Four close structurally, none is heard, and the rest is 06-06's listening pass. Two clauses were open when the plan's own tasks were finished and its success criteria claimed both, that pressing OK saves a per-event answer and that the screen says whose decision speech or braille is; reading the criterion from ROADMAP.md clause by clause found them, nothing in the tree reads a criterion, and both now have a check taken red by hand. Five guard records rather than two, because a record guards a rule and the plan counted per task, all five re-run through scripts/guards.sh itself. The two house_style settings guards did not redden on arrival as the plan promised, because they fire on controls written the wrong way and these were written the right way first, so each was instead shown to see a planted violation in the new code. wxdragon exposes no way to raise a widget event from outside, so that the picker's and the button's handlers call the functions the test drives is proved by reading two lines, which the test file's header says rather than letting the test's name cover both halves. WINDOWS.md 345 to 353, nine entries, one per unrun thing. The earlier entry, still true. 06-01 merged, all of it, and criterion 1 does not close because every one of its four clauses is about a screen and this plan is the half that is not one. The model can now say what somebody chose apart from what they get: `what_was_chosen_for` answers the choice as it was made and tells "no override" apart from "all four ticked", which the only public reader could not, because `channels_for` defaults a missing entry to every channel, drops the globally switched-off ones and adds a braille tick where only a sound was picked. A panel built on it would have shown somebody four ticks they never put there. `use_the_default_for` removes the entry rather than emptying it, since an empty set round trips and means silence, and `set_event_channels` is public where before it had eleven references all in its own file. `enum Event` and `Event::ALL` come from one list, so a seventeenth event cannot exist without a control, a sound scheme slot and test coverage. That was measured by hand on both sides rather than argued, because no test could express it: the variant absent from `ALL` left the whole library green at 7,118 tests, and in the list it is in `ALL` without anybody touching `ALL` while removing it is four compile errors. It is the plan's one declared test-first exception and carries no guard record, because the break fails to compile rather than reddening a test and guards.py reads cargo's FAILED lines. Nothing a user can reach changed, so no version bump and no changelog entry; 06-02 is the panel. Four deviations, none of which changed what was built, the largest being that the plan and the phase README both say three exhaustive matches where there are four. `Channel::ALL` carries the same hole, is out of scope, and is WINDOWS.md 343; FEEDBACK-01's evidence now says something the tree disproves and is WINDOWS.md 344, left for decision 7 at 06-07. The earlier entry, still true. 07-09 tasks 1 and 2 of three merged, and the plan is partial on purpose. An update is now fetched without anybody being asked once a kind of version has been chosen, checked twice, and offered once before it runs. The second check is the whole point: WinVerifyTrust says a file is validly signed, which millions are, and only reading the signer's certificate and comparing the name says it is ours, exactly rather than by containment. The refusal is proven against a real Microsoft-signed system file and the acceptance is proven by nothing, because this project signs nothing, so as this ships every real installer is refused and four pages say so. The checked file is a type only the check constructs and both the question and the run take it, so there is no ordering to rearrange into "run it anyway?". Two features on the windows crate already pinned at 0.62.2, no new dependency, and streaming turned out to need no reqwest feature at all. Reading criterion 2 from ROADMAP.md clause by clause found two clauses open that no test could have: a computer with no way to check a signature was downloading first and refusing after, and the setting's own description still said "nothing is downloaded yet" four commits after that stopped being true. Both fixed with their own red halves. Task 3 is the screen reader checkpoint, recorded and not attempted; it needs a published signed release, which is 07-08's. Criterion 2 does not close and SHIP-02 does not close: twelve of thirteen clauses are structurally complete and nothing has ever applied anything. The earlier entry, still true. 07-08 task 1 of three merged, and that plan is partial on purpose. What has to be signed is now counted from the build rather than remembered: a census in tests/installer.rs derives the three binaries from the [Files] block, the three published downloads from the workflow's own list and the uninstaller from the script's Uninstallable directive, which is seven where SHIP-01's wording names two. It was taken red by hand with an eighth Source line and named the new file. The one unverified fact in 07-RESEARCH.md is settled from the local Inno help and its premise was half wrong: the two-pass prompting behaviour belongs to a build with no SignTool, not to SignedUninstaller, so a CI job that signs at all never reaches it, and a signed uninstaller makes Setup write its messages to a separate unins???.msg. The portable copy and the zip are taken after build-installer.sh runs, so they inherit whatever it signs. Nothing is signed, nothing signs anything, and the three shipped pages that say the build is unsigned are untouched and still true. Task 2 is the Azure account only Pratik can create and is open; task 3 must not start before it. One deviation worth carrying: a guard record 07-07 wrote one wave ago went stale inside this phase, because the census reads the same published list its break changes, and it was corrected by hand before --remeasure would accept it
last_updated: "2026-09-13T15:30:00.000Z"
last_activity: 2026-09-13
state_head: e98514b0
progress:
  total_phases: 13
  completed_phases: 0
  total_plans: 97
  completed_plans: 92
  percent: 0
previous_activity: 2026-09-06, 04-08 done on branch picture-decorative-answer. A picture put into a message keeps its description across a draft save and a reload, proved by a test and by a break taken by hand rather than by a green nobody watched. A picture can be marked decorative, which is a question somebody answers rather than an empty box, offered only where furniture is plausible. Whether a decorative picture is announced is the reader's answer, on a control in Settings, Reading. Nobody has heard any of it.
last_activity_desc: "02-06 done: the writer and the condition dialog a rule editor needs are built and tested, and nothing in the running program opens either of them yet. That is 02-07's job and both are recorded as stubs rather than left to be found. The replace writes a search and its whole question list in one transaction, with the row stamped last on purpose, because stamping it first would make the only failure a person can cause fire before anything was destroyed and leave no test able to tell a transaction from three loose statements"
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-29)

**Core value:** Making correspondence and personal information legible to people who cannot see it.
**Current focus:** Phase 05.2 (Notes in OneNote)

## Current Position

Phase: 06 (How the application speaks). **06-03 is partial: tasks 1 and 2 of
three are merged at version 0.121.0, and the checkpoint between task 2 and task
3 is unanswered on purpose. Month names in a date now come from the machine.
Day names, the date in a signature sentence, and relative wording do not.
Criterion 2 does not close.**

**The question, which is Pratik's and nobody else's: what should "2 days ago" do
on a French computer?** There is no Windows API for it. Three options, with the
plan's own table in `06-03-SUMMARY.md` and two things it could not have known.
Option 2, falling back to the date where the locale is not English, needs a new
Win32 question, because nothing in this tree answers what language the machine
is set to: `date_display` reads `LOCALE_ILDATE` and `LOCALE_ITIME` as digits,
which are the order and the clock. And its cost falls on the person this program
is for. The module's own header says relative wording exists because "2 days
ago" answers "is this recent" in three syllables where "July 24, 2026 at 9:15
AM" takes a dozen; for somebody reading an inbox by ear that is the difference
between scanning and wading, on every row, for not using an English Windows.
The recommendation is still option 2, with the loss said out loud on the Reading
tab and ledgered rather than counted as small because the diff is small.

**Two Win32 mechanisms, not one, and it is grammar rather than tidiness.** A
month inside a date and a month on its own are different words in Russian,
Polish, Czech and Lithuanian. A date goes through `GetDateFormatEx` with a
picture holding both a numeric day and `MMMM`, which Microsoft's pages say is
the only way the genitive arrives; a bare list of twelve goes through
`GetLocaleInfoW`. Measured here rather than believed: a Russian date reads "2
января" and a Russian month list reads "Январь".

**The shape stays the person's.** Their stored wording and order pick the
picture and Windows supplies only the words, so month first on a French computer
gives "juillet 26, 2026", which no French computer writes on its own. Asking
Windows for its own idea of a long date would have thrown that choice away on
every machine whose locale disagrees with the person using it.

**A third guard went red that nobody predicted**, and it is worth carrying
forward. `test_every_guard_record_still_names_one_place_in_the_tree` fires when
a record names *code* that a rename has moved, which is a different check from
the one that counts tests, and a failure mode `CLAUDE.md` describes one step
over, about a record naming a *test* that no longer exists. Renaming a private
function is enough to trip it.

**Running `scripts/check.sh affected` by hand is not the run the hook makes.**
The scoped module runs come from the index the hook hands over, and a bare
invocation hands over nothing, so a direct run does only the tree-reading
guards. That is how two of the three failures were found and the third was not.

**The staged work of a terminated executor was read before being built on.** Its
red half was stubbed with the plausible wrong mechanism rather than with
nothing, so it failed for the reason the task exists, and its French assertion
passed against that stub because `fr-FR` writes "juillet" both ways. One thing
in it was corrected: the reworded `ENGLISH_ONLY` said the month names follow
this computer while eight signature sentences still write English ones, a claim
its own doc comment two lines above did not make.

**Criterion 1 has five clauses, not the four 06-01's summary names.** That
reading folds "by keyboard" into the first, and the folded clause is the one
this plan can least attest to: every control is a native Choice, CheckBox or
Button with a distinct mnemonic, both established by reading, and nobody has
tabbed through the tab. Four clauses close structurally and none of the five is
heard. What is left of criterion 1 is a listening pass, which is 06-06.

**Two of those clauses were open when 06-02's own tasks were finished, and its
success criteria claimed both.** That pressing OK saves a per-event answer, and
that the screen says whose decision speech or braille is. The model round trip
was proved, the screen was proved, and the function carrying one across to the
other was reached by nothing; the sentence was on the tab and no check knew it
was there, so deleting it would have broken the criterion and reddened nothing.
Reading the criterion from `ROADMAP.md` clause by clause found both. Nothing in
the tree reads a criterion, so no test, guard or review could have. **That is
the second phase running where this practice found what nothing else did.**

**The panel.** A picker of the sixteen events read from `Event::ALL`, three
controls, a button and two lines. The ticks are painted from
`what_was_chosen_for` and the first line from `channels_for`, because those are
different questions: painting ticks from the second would show somebody four
answers they never gave. The second line is what keeps the ticks honest, saying
what the event will really produce, because a channel switched off everywhere
stays off and an event left with only a sound has the quietest written channel
added back. The rule was not weakened to match the screen. The button removes an
answer and switching all three boxes off stores an empty one, which are opposite
outcomes and do not share a control.

**Five guard records, not the two the plan asked for, because a record guards a
rule and the plan counted per task.** All five were re-run through
`scripts/guards.sh` itself and each reddens exactly the test it names. The two
`house_style` settings guards did not redden on arrival as the plan promised:
they fire on a control written the wrong way, not on a new control, so
collecting that red would have meant writing the defective version on purpose.
Each was instead shown to see a planted violation in the new code, quoted, and
the code put back by hand.

**wxdragon 0.9.17 exposes no way to raise a widget event from outside**, so the
live test moves the real picker and reads the real check boxes but cannot open
the picker or press the button. That the two handlers call the functions the
test drives is proved by reading two lines and by nothing else, said in the test
file's own header rather than left implied by a passing test whose name covers
both halves. `WINDOWS.md` 345 to 353, nine entries, one per unrun thing.

**06-01, still true. What somebody chose and what they get were one answer, and now they are
two.** The only public reader was `channels_for`, which defaults an event
nobody has touched to every channel, drops the channels switched off
everywhere, and adds a written channel where only a sound was picked. So it
answered the same for "no override" and "all four ticked" and reported a
braille tick nobody set. A settings panel rendered from it would have told
somebody they chose something they did not. `what_was_chosen_for` answers the
choice as it was made, and `Some(empty)`, meaning silence for that event, is
told apart from `None`, meaning the default. `use_the_default_for` removes the
entry rather than emptying it, which are opposite outcomes wearing similar
names. `set_event_channels` is public, where before it had eleven references
and every one was in its own file, so a per-event override could only ever
arrive by hand-editing the stored settings string.

**The sixteenth event was defended by nothing, which was measured rather than
argued.** `Event::ALL` was a hand-written array. Four exhaustive matches force
a new variant to be described and nothing forced it into `ALL`, and `ALL` is
the only enumeration of the variants that exists, so no test could ask for a
variant that is not in it. A seventeenth variant answered in all four matches
and left out of `ALL` passed the whole library, 7,118 tests, nothing red. Both
now come from one list, so the disagreement is unrepresentable rather than
detectable, and removing a variant from the list is four compile errors. That
is the plan's one declared test-first exception and it carries no guard
record, because a break that fails to compile reddens nothing and `guards.py`
reads cargo's FAILED lines. `Channel::ALL` has the same hole and is left open
on purpose, `WINDOWS.md` 343.

Phase 07 (Installing, updating and what is stored) is where the last nine
plans went and it has not closed. **All nine have a summary on disk and three
say `partial`: 07-06, 07-08 and 07-09. Three checkpoints are open and none can
be closed by anything in this repository: 07-06's needs a workflow run,
07-08's needs an Azure signing account only Pratik can create, and 07-09's
needs a published, signed release, so it waits on 07-08's. Phase 05.2's is
open too.**

**Wixen Mail can now fetch an update and refuses to run anything this project
did not sign, and it has never updated anything.** With a kind of version
chosen, a published newer version is downloaded with no question asked at that
moment, which is the agreement given at the setting. Pressing Check for Updates
does the same on demand whatever the setting says: one route, not two. The file
is then checked twice and both must pass. `WinVerifyTrust` answers whether the
signature is valid, which millions of files are, and reading the signer's own
certificate answers whose it is. Only the second is the check. The comparison is
exact rather than "contains", because a certificate issued to "Pratik Patel
Holdings Ltd" is a name anybody can buy.

**The refusal is proven and the acceptance is not, and that distinction is the
whole honest state of this feature.** A real Microsoft-signed system file was
refused by name, so the mechanism runs end to end against a genuine Authenticode
signature on this machine. Nothing has ever seen a file this project signed,
because this project signs nothing, so every test of the accepting path runs
against a name in a fixture. As this ships, every real installer is refused,
deleted, and the person is sent to the releases page. That is the designed
behaviour rather than a shortfall, and `docs/changelog.md`, `docs/installing.md`,
`docs/privacy.md` and `docs/ALPHA_TESTING.md` all say so.

**The rule that nobody is asked about a file already known to be bad is carried
in a type.** `Verified` has a private field and only `verify` builds one, and
both the function that asks and the function that runs take it. An ordering can
be rearranged by a later edit nobody notices; an argument type cannot, so there
is no arrangement of this code that puts "this could not be verified, run it
anyway?" in front of somebody.

**Reading criterion 2 clause by clause from `.planning/ROADMAP.md` found two
clauses open that nothing else could have.** The criterion says nothing is
downloaded where a signature cannot be checked; it was downloading first and
refusing after, which on Linux or macOS fetches an executable onto a disk that
can never look at it. And the update setting's own description still said
"nothing is downloaded yet", four commits after that stopped being true, in the
one sentence where the consent for an unattended download is given. Both were
fixed with their own red halves. Neither was caught by a test, a guard or a
review, because nothing in the tree reads a criterion. **Checking criteria
against premises is worth the time it costs.**

**`src/main.rs` maps to no test target, and that is a new hole in the gate.**
The mapper turns a changed `src/a/b.rs` into `--lib a::b::`, and `main.rs`
becomes `--lib main::`, which matches nothing, because `main.rs` is the binary
and the suite runs the library. The third of the three rules that clear a
downloaded installer lives in `prepare_data_folder` there, so the commit adding
it ran no test reaching it and nothing in the tree can. It is the same shape as
the `.iss` hole 07-02 closed and the workflow hole 07-07 found, and it is the
fourth instance in this phase. The clearing itself is tested through the library
function; only the call site is uncovered.

**Two measurements that correct the plan.** `reqwest::Response::chunk` needs no
feature at 0.13.4, so the byte bound is enforced as bytes arrive at no cost to
the dependency, where the plan assumed `stream` was needed. And the fixture for
the refusal that matters cannot be `notepad.exe`: almost every Windows system
binary is signed through a catalogue rather than inside the file, and
`WinVerifyTrust` asked about a file does not look in catalogues, so it answers
`TRUST_E_NOSIGNATURE`. Written a little more loosely, as "refused for any
reason", that test would have passed for entirely the wrong reason.

---

**From 07-08, still true. Seven things have to be signed and SHIP-01's wording
says two.** The census in
`tests/installer.rs` derives both halves rather than holding a list: the
`[Files]` block gives `wixen-mail.exe`, `wixen_mail_search.dll` and
`wixen-mail-search-setup.exe`; the workflow's published list gives the setup
executable, the portable copy and the zip; and the script's own `Uninstallable`
directive gives the uninstaller Inno writes, which is in neither list and is the
one a census taken off the two lists loses. A test holding seven strings would
pass when an eighth executable arrived and never mention it. This one grows and
names it, proved by adding a fake eighth `Source:` line and reading the failure
rather than by trusting the shape of the code.

**Nothing is signed, and that is the plan's ordering rather than a shortfall.**
There is no certificate. No `SignTool` directive, no `SignedUninstaller`, no
signing step, and `scripts/build-installer.sh` and `.github/workflows/release.yml`
are not in the diff at all. The three shipped pages saying the build is unsigned
are untouched and still true; correcting them before something is signed would
put a false statement into shipped documentation, which is what the plan's
ordering exists to prevent.

**07-RESEARCH.md's assumption A3 is settled from the local Inno help, and its
premise was half wrong.** Read 2026-09-12 from `ISetup.chm` in the per-user
install rather than from a search summary. `SignedUninstaller` defaults to `yes`
once a `SignTool` is set, and with one set the help says the uninstaller "will be
signed automatically on the fly": one pass, no prompt. The two-pass prompting
branch is what `SignedUninstaller=yes` with no `SignTool` gets, so a CI job that
signs at all cannot hang there. One consequence nobody predicted: a signed
uninstaller makes Setup write its language messages into a separate
`unins???.msg`, because embedding them would invalidate the signature, so the
installed folder gains a file. All of it is written into the `.iss` where
`SignedUninstaller` will go, so whoever wires it reads it there.

**The portable copy and the zip will inherit.** `release.yml` runs
`build-installer.sh` and then takes the copy from `target/release/wixen-mail.exe`,
so a binary signed inside that script is already signed when the copy is made.
Read off the step order rather than assumed. It is still a claim about the file
and not about a release: no release has ever been cut here, so nothing has
watched a signature survive the publish path, and task 3 owes that proof by
verifying the published copy rather than the source binary.

**A guard record went stale inside its own phase.** 07-07's record for a
published glob still matching a name the release writes was correct when it was
written one wave ago. The census added here reads the same published list, so its
break now reddens two tests where the record named one. Corrected by hand before
`--remeasure` would accept it; left uncorrected, the remeasure would have refused
and the cause would have read as a broken tool. Nothing about 07-07's own work
moved, which is why this is the case that costs most: the only person looking at
the right moment is the author of the new test.

**Two of this plan's own premises were already false when it ran.**
`tests/house_style.rs` holds 19 records and 69 tests, not the 18 and 67 the plan
re-checked on 2026-09-11, both having moved in the day between. And
`docs/installing.md:7` is at `:8`. The plan's conclusions survive both; the
pattern does not, and it is the same one this project keeps meeting.

**The append tool corrupts a ledger entry containing a backslash.**
`gsd-tools windows append` escapes the character in the JSON half and not in the
markdown table half, so the two disagree and the commit is refused. Caught by
`test_both_halves_of_the_ledger_say_the_same_thing`, which is the guard working,
though the message reads as a ledger the author corrupted. Worked around by
rewording; `WINDOWS.md` 333.

**What the gate really selects was checked rather than assumed**, because three
separate instances of that blind spot have been found in this phase alone. An
`.iss` answers `all`, a `tests/*.rs` answers `affected` and maps to its own
`--test` target, `guards/guards.toml` answers `affected` and maps to no target of
its own, and a `.planning` markdown file answers `docs_only`. The GREEN commit
carried the `.iss`, so it ran the whole gate and swallowed the other three.

---

**From 07-07, still true. A release that cannot produce a file it promised stops
and names it.**
`release.yml` publishes four globs and `fail_on_unmatched_files` was `false`,
whose own documentation calls it the indicator of whether to fail if any glob
matches nothing, so a release that could produce three of the four published
three, went green, and left the fourth to be found by somebody trying to
download it. There are two nets now and the order is the point: a step after the
assets are built and before anything is published checks each promised file
exists and names the ones missing, and the flag is `true` as the second. The
flag alone fires inside the publishing step, by which time the tag is on the
remote and the release exists. The four globs are written once, in a job-level
`RELEASE_ASSETS` entry both steps read, because two copies of a list agree until
somebody edits one.

**Nothing changes about when a release can happen, and a test holds that rather
than a sentence.** The `on:` block is byte-identical, `workflow_dispatch` alone,
and no token, permission or step that could tag, publish or push was added.
`test_a_release_still_happens_only_when_somebody_asks_for_one` reads the block
and refuses any other trigger, with a companion proving the reading can see a
widened one, and a guard record whose break swaps the trigger for a push. Every
change here makes publishing stricter and none makes it easier.

**Three of the seven new tests had no red half, and that is said out loud rather
than left to be noticed.** The names already agreed, the trigger was already
right, and no cargo-release configuration exists. A test written over a rule
that already holds has no red half by construction, so the honest substitute is
its own recorded break: two are in `guards/guards.toml` and the third is
measured by hand in `07-07-SUMMARY.md`, because the break that would redden it
is a file appearing and the registry's mechanism is a substitution inside a file
that exists.

**Where the shape of a produced string is decided, guard the configuration
rather than the string.** `dist/wixen-mail-$tag.exe` is written and
`dist/wixen-mail-v*.exe` is published, so the two agree only while the tag
begins with `v`, which comes from cargo-release's defaults. Rather than write
that into a comment and assert under it,
`test_nothing_here_moves_the_published_tag_away_from_the_shape_a_glob_expects`
fails if `release.toml` or `.cargo/release.toml` appears or `Cargo.toml` grows a
`[package.metadata.release]` section. The assumption itself stays unverified
until a release is really cut.

**The workflow has never run and that is now settled rather than locally true.**
`git tag` returns nothing and `git ls-remote --tags origin` returns nothing while
`git ls-remote --heads origin` answers with `main`, which is the control the
research could not run. `WINDOWS.md` 326. The tag is pushed before anything is
built, so a failure after that leaves a tag with no release behind it; that is
recorded at 327 and deliberately not changed, because moving it changes when a
tag exists.

**The certificate decision is on the record as a decision.** Azure Artifact
Signing, decided 2026-09-06, about $9.99 a month, Pratik Patel as the publisher
name, residence requirement met and confirmed. The three that lost are each
named with what they lost on, and with which of those reasons could move: the OV
price could, SignPath's publisher name could not. SHIP-01 also says what the
decision commits the project to, which is four binaries plus an uninstaller whose
prompting behaviour is unverified and not two files, and what signing does not
buy, which is that no certificate available here removes the SmartScreen warning
and that `PrivilegesRequired=lowest` means the prompt a signature does fix is not
shown on a per-user install anyway.

**Microsoft's page was re-fetched rather than quoted, and it has not moved.**
Read 2026-09-12; still "EV certificates no longer bypass SmartScreen", table
still giving OV and EV the same first-download outcome, `updated_at` still
2026-08-17. Its `ms.date` is 2026-05-04, which is a different field and worth
knowing apart from the one the research recorded. Criterion 1's parenthetical is
replaced from that reading and the rest of the sentence is byte-identical.

**Criterion 2 is replaced whole per D-14, and this plan wrote the standard 07-09
is measured against without reading 07-09.** It now asks for a downloaded
installer verified against this project's own publisher name before anybody is
asked about it, with a failed check refusing and deleting rather than warning,
and it tells consented from silent by where the consent was given rather than by
whether a question was asked at fetch time. SHIP-02's first `[D]` line had the
same defect and is replaced beside it; the fourth line for the release channel
is added per D-15; the other two were read and left alone.

**Nothing is signed, nothing pretends to be, and criterion 1 waits on something
outside this repository.** `WINDOWS.md` 329. Neither SHIP-01 nor SHIP-02 is
ticked and neither row in the requirement-to-phase table moved.

**A guard record is now read whatever kind of file its break lands on.**
`check.sh`'s record-to-suite mapping discarded every changed path but
`src/*.rs`, on the reason that `--lib` was never going to reach anything else.
That is true about `--lib` and this mapping answers with a `--test` target, so
the filter discarded exactly the records it exists for. The two records this
plan added naming `release.yml` sat there looking like coverage while the commit
that changed the workflow ran four tree guards and nothing that reads it. Four
cases went in first, two red, and two existing cases that pinned the old rule
were corrected in place with the wrong reason quoted above them rather than
quietly deleted. A changed `tests/*.rs` is still left out, because it already
runs its own target.

**It was found by reading which targets the hook selected rather than its
verdict**, which is worth keeping: splitting red and green into two commits gives
them disjoint file lists, so under a scoped gate the green commit can run none
of the tests it exists to turn green. That is exactly what happened here.

**One failure in the tree is nobody's change.**
`tests/a_move_says_what_has_not_been_sent.rs` cannot always reach the Windows
credential store on this machine, failing with "No default store has been set".
It appeared in two of three whole-tree runs on 2026-09-12, a different test of
that file each time, and the file passes alone. Not diagnosed, not caused by
anything here, and it did not appear in the final gate run. `WINDOWS.md` 328.
Written down because reading a measurement run by grepping for the failure you
expected is what stops it finding the one you did not.

Version `0.116.0`, unchanged, and no changelog entry: a census of what would have
to be signed changes nothing somebody running the program can tell apart, so
inventing an entry for it would be worse than having none. `guards/guards.toml`
holds **732** records with the census reading 192 and 540. `.planning/WINDOWS.md`
reaches **333** with 311 open, and nothing is pushed. `scripts/check.sh all`
passed all four on the branch tip in **463 seconds**, redirected to a file rather
than piped, which is inside the 419 to 654 the seven landed branches report.
`tests/installer.rs` went from 11 tests to 13 and from 3 guard records naming it
to 4; `tests/house_style.rs` is unchanged at 19 records and 69 tests, because no
test was added there.

**Owed after the merge and not blocking it:**
`scripts/guards.sh --touched-by 3e52ab85`. Among files a record names, this
branch changed `tests/installer.rs` and `.github/workflows/release.yml`.

---

### 07-06, the plan before this one

**07-06 is merged at `c6546e66`. One task of three, and it is the first
plan of this phase that did not close what it was written for, with its
checkpoint open.**

**There is a way to find out whether this crate builds off Windows, and nobody
has used it.** `.github/workflows/other-platforms.yml` builds the main crate and
runs its suite on `ubuntu-latest` and on `macos-latest`, as two jobs that do not
depend on each other, so a Linux failure still leaves the macOS answer on the
table. It is `workflow_dispatch` only and `permissions: contents: read`, with no
token in any step, because a job nobody knows will pass must not go on a push
trigger: that is how CI here stayed red for sixty runs across five weeks while
looking maintained. Dispatch it from the Actions tab, under the name "Other
platforms". `07-06-SUMMARY.md`'s first section says what to report back, four
things per platform, written so nobody has to re-read the plan.

**SHIP-05 does not close and criterion 5 stays open.** The requirement reads as
though it were two CI jobs. It might be, and it might be a port, and after this
plan that is still unknown rather than guessed at, which was the point. Task 3
of `07-06-PLAN.md` acts on the answer when there is one and spells out both
branches. `WINDOWS.md` 323 and 324.

**Nothing in this phase waits for it.** 07-07, 07-08 and 07-09 are unaffected.

**One premise of the plan reads more hopefully than the evidence supports.**
Upstream says wxDragon downloads prebuilt wxWidgets libraries rather than
building the toolkit from source, and the research here said the opposite. Read
again on 2026-09-12: every place in that README naming a platform beside the
word "prebuilt" names Windows, and its Linux requirements list asks for `cmake`
and GTK development headers, which is what a source build needs. That settles
nothing, which is why the measurement exists, but both jobs carry
`timeout-minutes: 120` against GitHub's default of 360 for the expensive case.

**The guard that holds CI steps to `--no-fail-fast` read a hardcoded pair of
files.** `test_one_failing_target_does_not_hide_the_rest` now names three, and
the new one was taken red by hand before the violating step came out, because a
list extended and never proved reads as covered. No `#[test]` was added to
`tests/house_style.rs`, which 19 records fingerprint, so no re-measurement was
owed and none was run.

**A workflow-only change answers `affected` on a branch and `all` on `main`,
measured rather than assumed**, and `which-checks.sh` has no rule about
`.github/` at all: a `.yml` simply fails the last loop's `.md` or `.txt` test.
What `affected` then runs for a workflow file is narrower than it sounds. The
scoped run maps a path to a target and knows `src/*.rs` and `tests/*.rs` only,
so a workflow file selects no target of its own and is checked by the four
tree-reading guards, `house_style` among them, which collects
`.github/**/*.yml`. That is the `.iss` hole one layer along, smaller because
`house_style` runs on every commit. Recorded rather than fixed.

**`progress.completed_plans` is 86**, counted from the `*-SUMMARY.md` files on
disk rather than incremented. One of the 86 is `partial` rather than `complete`,
and it is this one.

**The plan before this one: somebody can ask whether there is a newer version
and be told in words.**
Help, then Check for Updates, asks GitHub on the channel the setting picks,
compares the tag with this build by 07-04's ordering, writes the answer to the
status bar and says it at high priority on the command topic. When there is a
newer version a dialog offers to open the page about it. **Nothing is
downloaded and nothing is run**: that is 07-09's, after signing lands in 07-08.

**One setting, three answers, starting on not looking.**
`AppConfig::which_updates`, a top-level field holding 07-04's `WhichUpdates`,
offered as a combo box under a new "New versions" heading on the General tab.
It is not called `check_updates`, and the loader question is answered: this
file is read with one `serde_json::from_str` and the error is propagated, so a
stale value of the wrong type fails the whole file and takes **every** setting
on that machine back to its default, not just this one.

**Five answers, and none of them claims something the check did not learn.**
There is a newer one, this is the newest, nothing is published yet, the answer
could not be fetched, and the answer could not be read. A rate limit is 403
**or** 429 and is told from the other 403 by `x-ratelimit-remaining` rather
than by the status, which was measured on a real socket as well as read.

**Nothing published arrives in two shapes and the plan knew only one.**
`releases/latest` answers 404; `releases` answers 200 with an empty array. A
reading that knew only the 404 would have told everybody on the development
channel that they were up to date, for a repository with nothing in it, which
is the only state this repository is in.

**07-04's code is now reached by a path a person can take**, which closes
`WINDOWS.md` 313. Criterion 2 still does not close: applying is 07-09's and the
verification 07-07 adds is not here.

**`docs/privacy.md` no longer promises there is no update check.** It says what
the check sends, what GitHub keeps, quoting GitHub's own sentence about the
originating address, and that sixty unauthenticated requests an hour come from
one address. Two gaps that were already there closed with it: the log can land
in `%TEMP%` when the data folder cannot be resolved, and a Microsoft sign-in
asks for `Notes.ReadWrite` that nothing uses.

**`ReleaseChannel` is derived and never stored, and the plan said both.** Its
own premise said the setting holds the three-valued answer and the channel is
worked out from it; two acceptance criteria, a trust boundary and a threat
entry said a channel value arrives from a settings file. Both halves could have
been made true separately, which is the failure this project keeps meeting, so
it was settled one way: the channel has no serde representation at all, there
is no stored channel spelling for 07-05 to keep, and threat T-07-16 as written
is about a value nothing writes. What a settings file holds is `WhichUpdates`,
spelled `not_looking`, `public_releases` or `development_releases`, and those
three are fixed from here.

**A published tag carries a `v`, and nothing in the plan said so.** `cargo
release` writes the tag and `release.yml` names the portable download after it,
published under the glob `wixen-mail-v*.exe`. That glob is the only place in
the tree where the real shape of the string is written down, and no tag exists
to confirm it, because no release has ever been cut. Without it the comparison
would have answered "could not read" for every release this project publishes,
with every test green and nothing failing. That is `WINDOWS.md` 312.

**A guard record went stale inside its own plan, three hours after it was
written.** The build-identifier record named two tests when task 1 measured it
by hand. Task 2's offer decision routes through the same comparison, so a row
of its table where two versions differ only after a plus turns into an offer
under the same break, and the count check's remedy is what found it. Three
tests now, measured and confirmed through the tool.

**The plan before this one: the program says which parts of its accessibility
layer do nothing on the build it is running as.** In the About dialog, which is where the Help menu's
own item lands, and on the stream at startup, before anybody has opened a menu.
On Windows it says nothing, because both halves of the bridge work there and a
warning that is wrong every time it appears teaches people to ignore warnings.
SHIP-06 closes. Nothing here makes anything work on any platform, and the
changelog says so in as many words.

**The criterion names one bridge and there are two.** Announcements go through
`UiaRaiseNotificationEvent` and had a status nothing had ever read. Accessible
names go through `wxAccessible` and had no marker anywhere in the code, which
`grep -n 'cfg(target_os' names.rs` returning nothing is the whole of. A
disclosure built from the existing status alone would have said that
announcements do not reach a screen reader and said nothing about every control
in the window reaching the accessibility tree with no name, which is the larger
of the two failures. Both halves now answer for themselves.

**Each answer comes from whichever platform arm compiled, not from
`cfg!(target_os = "windows")`.** `screen_reader.rs` gained a `native` module for
the case where there is no bridge, carrying the same names the Windows one does,
so `deliver` has one shape rather than two and the arm whose whole body
discarded its arguments is gone. `names.rs` is two cfg arms rather than two
modules, because there is no per-platform code in that file, only a fact about
what wxWidgets does underneath. That asymmetry is written into the doc comment
rather than left to look like an oversight.

**`NativeBridgeStatus` and the two accessors nothing had ever called are gone.**
Removed rather than given a caller: the constant says the same thing from the
same source and has two real callers, so keeping the enum would have left two
representations of one fact with the uncalled one still uncalled.
`grep -rn native_bridge_status src/ tests/` now returns nothing, and clippy said
nothing about the removal because no caller existed to break.

**None of it has been seen or heard, and the reason is structural.** No Linux or
macOS build of this program has ever been made, so the sentences have only ever
been produced from arguments a test on Windows chose. That is `WINDOWS.md` 307
to 310, which also carries the About dialog's layout, nobody having heard the
four paragraphs, and the third-platform property being structural rather than
tested.

**A commit that changes the installer script now runs the tests that read it.**
`scripts/which-checks.sh` answers `all` for an `.iss`, which is the manifest
rule one layer down: something reads the file as data and the softer answer
reached none of it. The scoped run maps a changed file to a target by its path
and knows `src/*.rs` and `tests/*.rs`, an `.iss` is neither, so no scoped target
was chosen at all and the three tests that read the script, all unit tests in
`src/`, ran on every commit except the ones that could break them. The rule sits
below the manifest block on purpose: the version-bump exception hands the softer
answer to a commit whose whole manifest diff is the package version, and this
project puts that bump in the same commit as the change, so a rule inside that
branch would have left the hole open for nearly every installer commit there
will ever be. Measured before and after with a fixture diff rather than argued.

**Both shortcuts and the Apps and Features entry name `{app}\icon.ico`, and
nobody's picture changes.** `build.rs` already embedded `assets/icon.ico` into
the executable and Inno's help says a shortcut with no `IconFilename` gets the
file's default icon, so both already showed it. What changed is that the picture
no longer depends on the executable's resource table being right, which has
failed here once. The criterion's own "and nothing else" was wrong by one line:
naming a path nothing installs makes Windows fall back in silence, so the
`[Files]` line is part of the change and the test compares the two whole paths
rather than asking whether each section mentions an icon.

`tests/installer.rs` is new and `07-07` and `07-08` write into it. It reads the
script as text, which is all anything here can do, and its own doc comment says
so: nothing compiles it with ISCC, installs anything or looks at a shortcut.
That is `WINDOWS.md` 306.

Version `0.116.0`, unchanged by 07-06 because nothing user-visible changed and
`CLAUDE.md` ties a bump to a change that needs one. `guards/guards.toml` holds
729 records with the census reading 192 and 537, untouched by this plan.
`.planning/WINDOWS.md` reaches 325 with 303 open, and nothing is pushed.
`scripts/check.sh all` passed all four on 07-06's branch tip in 654 seconds over
7,492 tests. That is slower than the 419 to 471 the last four branches report,
and the reason is the run following several test rebuilds rather than anything
about this branch, so it is a warm-versus-cold difference rather than a trend.

**Two agents shared this working tree and the gate noticed.** Phase 6's planning
was running while 07-06 executed, and its untracked plan files made the
roadmap's `0/TBD` for phase 6 false, which refused 07-06's document commit:
`test_the_roadmap_counts_the_files_that_are_on_disk` reads the disk rather than
the index and cannot tell one agent's uncommitted work from another's. Chasing
the number failed three times, at 1, 2, 3 and 5 plans, because the planner wrote
faster than a gate run takes. The row was settled at `0/8` after the count held
still for several minutes, which is also the figure phase 6's own README asks
for; that README says it left `.planning/ROADMAP.md` alone on purpose because
phase 7 was editing it. `progress.total_plans` went from 92 to **97** by
counting `*-PLAN.md` on disk rather than incrementing, which that README also
asks of whoever owns the merge. `WINDOWS.md` 325.

Current Plan: 3
Total Plans in Phase: 8

---

### 07-01, the plan before this one

**The program now says what it leaves on somebody's disk.** The first-run screen
and the end of `--help` both say the downloaded mail is not encrypted on this
computer, that Windows keeps other people who use the computer out of the
folder, that anyone who takes the drive out can read it unless the disk itself
is encrypted, and that BitLocker is the answer to that. Two documents said it
and the product said nothing, so the only people who knew were the ones who
opened a page. Nothing about the storage changed. SHIP-04 closes on this;
**nobody has heard either sentence**, which is `WINDOWS.md` 302 and 303.

It went in `INTRODUCTION` rather than behind a second button, against what that
constant's doc comment appears to say, because "every time" is once per install:
`wx_app` returns early when `told_about_the_alpha` is set. A 900-character bound
now holds the text at 694 so the same argument cannot be made twice more.

**Two guards landed and both were measured by hand rather than watched.** No
shipped page can promise that signing makes the Windows warning go away; the
predicate tells code signing from a signed message and from signing in, reads 31
sentences across `docs/` and `README.md`, and refuses none today. Under its
recorded break only the companion reddens: the corpus walk stays green, because
no page violates it, so the walk alone cannot tell a working predicate from a
narrowed one. That is the failure `CLAUDE.md` names for a document guard,
measured rather than argued. And every path `src/common/paths.rs` hands out must
be named on both pages that list what is stored, enumerated by calling the
accessors because `oauth_toml` joins onto `config_dir` and a grep for
`self.root.join(` would have missed exactly the path that was missing.

**That check found three gaps where the plan predicted one.** `security.key` was
on neither page and `oauth.toml` was on `docs/privacy.md` only. Both pages now
name every path, and `docs/privacy.md` says for the first time that uninstalling
cannot clear the Windows Search index, which the installer and uninstaller both
said and the page did not.

**Three writes go to the temporary folder through no accessor and the check
cannot reach any of them.** `logging.rs:79`, `main.rs:307` and
`help_page.rs:97`; only the second is on a page. All three predate the plan. The
check's comment names them so a green build is not read as "every path is listed",
and `WINDOWS.md` 304 carries it for 07-05, which owns the privacy page.

Version `0.113.2`, `guards/guards.toml` holds 722 records with the census reading
192 and 530, `.planning/WINDOWS.md` reaches 305 with 284 open, and nothing is
pushed.

---

### Phase 05.2, the phase before this one

**05.2-03 is merged at `3bc2651`, version 0.112.0, and it is the last plan of
that phase. Its checkpoint is open and nothing in it has been answered.**

**A note on a Microsoft account now reaches OneNote.** `service::onenote_notes`
is the third implementation of `NotesService` and the first hosted one. Each
section of an account's notebooks is a note folder named by its whole path,
joined with ` / `; `sync_the_notes_of` asks the account for its sections, makes
a folder for each, and runs the same `sync_notes` once per folder. The sync's
code names no backend, which a grep still proves, and the seam did not change at
all: `NotesService` is untouched.

**The real client found four places the seam was still about CalDAV, and the
fake from `05.1-04` predicted none of them.** A clash reported by a marker is
not a clash, and both backends were holding two identical copies for somebody to
choose between. Saying what a backend kept costs a request for a hosted backend
and the contract priced it at nothing. A marker missing on one call is a
different question from a backend with no markers. And a container can be the
service's answer rather than a row on this computer, which is why a Microsoft
account's Notes list is empty until the first sync. All four are written into
`docs/development/the-notes-seam.md`, with what the fake got right and the three
places it was wrong.

**Nothing has met Microsoft.** Every request is answered by a loopback server
the tests start. Eleven ledger entries, 291 to 301, name what that cannot
settle. Neither PIM-07 nor PIM-08 is ticked.

Version `0.112.0`, `guards/guards.toml` holds 720 records with the census
reading 192 and 528, `.planning/WINDOWS.md` reaches 301, and nothing is pushed.

---

### What came before, kept because it is still the ground this stands on

**05.2-02 is merged at `93a3f56`, version 0.111.0.** The Graph client for OneNote exists: a page made, a notebook's four
levels walked and bounded at eight section groups, a page changed by removing
what is on it and appending what is not, and a page removed. Every request is
read off a loopback server the tests start, and none has met Microsoft. A
Microsoft account is now asked for `Notes.ReadWrite`, on the consent list and on
the array a Graph token is really refreshed against, because either alone gives
no running account the permission.

**Nothing calls any of it.** Six public entry points and no caller in the running
program; `05.2-03` is the backend behind the seam and the last plan of the phase.
`docs/changelog.md` and `docs/PROVIDER_SETUP.md` both say plainly that nothing
syncs notes to OneNote.

**The plan's central artifact could not exist and that is the one substantial
false premise found.** It specified `tests/what_the_onenote_client_really_sends.rs`
and told the executor to copy the existing loopback harness. That harness and the
test-only client constructor are both `#[cfg(test)]`, so a file under `tests/`
links a library where neither exists; measured with a throwaway file, not
reasoned. `outward.rs`'s write census also reads the test half of a named file by
splitting on `mod tests {`, which such a file has none of. The tests are in
`src/service/microsoft_graph.rs`, which carries no guard records and cost nothing.

**One live defect was found and deliberately not fixed**, ledger 282:
`Tasks.ReadWrite` is on the consent list and absent from the refresh array, so by
the same argument this plan made about notes, no running Microsoft account holds
it and every Graph task write is refused. `docs/PROVIDER_SETUP.md` tells somebody
that signing in again will send their waiting task changes, which that gap would
make false. Recorded rather than rewritten: the inference is sound and no real
account has been tried.

**Nine ledger entries, 282 to 290, all open.** Three guard records added, all
measured by hand; five re-measured by the scoped remedy and none stale.

**Ledger 274 is closed, and it closed 245 and
246 with it.** One backend container is one note folder, which Pratik decided on
2026-09-11 and which the seam contract had written down and nothing had built.
`note_folders` carries an opaque container, `notes_backend` makes one folder per
calendar server, the sync runs once per folder and files an arrival into the
folder its container is, a folder somebody made here has no container and sits
under mail's own "On this computer" branch, and a note moving between two backed
folders is created in the new container before it is removed from the old, by
the write the task move already uses.

**245 is closed by removal rather than by documentation.**
`the_calendar_server_of`, which took whichever calendar server the store
answered with first and never said which, is gone. `the_calendar_servers_of`
answers with all of them and each is a folder, so there is no first to pick.

**The contract said the sync did not have to change and it did**, though the
seam did not: `NotesService` is untouched, `sync_notes` still takes one
container, and nothing parses one. What had to change is that its three passes
worked from the account, so a loop over two containers would offer every waiting
note to whichever ran first. The seam contract records that under "Built
2026-09-11" rather than leaving the document and the code to disagree.
`.planning/phases/05.2-notes-in-onenote/LEDGER-274-REPORT.md` has the rest,
including the three guard records that turned out not to be what they said and
the two of those that were already stale on `main`.

**05.2-01 is built and merged. Ledger 270, which came out of its checkpoint, is
closed.** PIM-04's structure criterion is
met for both backends that exist: headings, nested lists, tables, links with
their addresses and pictures with their descriptions all come back as what they
were. Twelve of the twenty-two constructs in the fidelity table survive, up from
seven, and **none of the ten that do not is this program's own reader any
more**. Every remaining loss is OneNote's or HTML's.

Two things were built. `Piece::Item` carries a depth and `Piece` has a `Table`
variant, so a nested list and a table survive `structure` and `from_markup` and
`spoken` says both; that was a shipped accessibility defect reaching a screen
reader through five call sites in `read_aloud.rs` and through a Google task's
description, and it is fixed for all of them. And
`long_text::from_markup_to_edit` is a second reading of the same tree walk whose
output is stored and edited again, used by `the_note_on`, which keeps a link's
address, a picture and a line break. `from_markup` is unchanged for its speaking
callers and three tests hold it to that.

**Nothing here has been heard.** Entry 271 is the screen reader pass that would
settle whether repeating a column heading on every cell floods a wide table and
whether "bullet level 2" is heard as a level or as part of the text. Tests prove
which words are produced, not that they are good to listen to.

**Phase 05.1's own checkpoint is still open** and this does not close it: a
screen reader pass over the new address book screen, which is the one thing in
that phase no test in this repository can settle.

Version `0.110.0`, `guards/guards.toml` holds 713 records with the census
reading 192 and 521, `.planning/WINDOWS.md` reaches 281, and nothing is pushed.

**What the guard sweep found, which is the expensive half of this work.** The
eighteen records naming `long_text.rs` were re-measured five times across the
branch. Eleven were not what they said. Two of them, both `inline` records, had
already stopped working on `main` before this branch touched anything: their
breaks quote an `img` arm the file stopped holding when that arm learned to say
"image with no description", and `guards.sh` refuses a break it cannot find
rather than reporting the guard, so both said nothing and read as covered. Four
had red lists shorter than the truth, one naming five tests where the break
reddens forty, because thirty-one of the missing live in `onenote_page.rs` and
no record named that file, so the count check could not see them arrive. Six
were written as two lines naming a neighbouring match arm, which is the spelling
that breaks the moment a neighbour moves; all of those are now one
self-contained edit each.

**What 05.2-01 built.** `src/service/onenote_page.rs`: a note's title and
Markdown body into the HTML a page is created from, and a page's returned HTML
back into a title and a body. Pure, no network, no database, 48 tests. Both
directions are thin over `long_text`'s existing `as_markup` and `from_markup`,
so nothing renders Markdown twice.

Two decisions it had to take rather than inherit. The body is flattened out of
its `div` wrappers before `from_markup` sees it, because that function reads a
`div` as one paragraph and concatenates everything inside it, and OneNote wraps
all page content in at least one. Measured through the unflattened path, the
whole page comes back as one run of text and a fidelity table would have
reported every construct as lost and blamed Microsoft for this program's reader.
And a table's header cells are written as ordinary cells, because the reference
names `td` and not `th`, and cutting `th` left the two header words as one text
node inside a `tr`, which every parser moves out of the table and runs together.

**What the measurement says.** Seven of twenty-two constructs survive. Of the
fifteen that do not, seven are OneNote's doing, six are this program's own
reader, one is both, and one is HTML's. The sharpest is a code block: neither
`pre` nor `code` is in any list the reference names, so two commands on two
lines come back as one line that runs neither. The most costly for meaning is
struck-out text, which comes back as ordinary words, so a job crossed off and a
job still to do read alike. The table is in
`docs/development/the-notes-seam.md`, it covers both backends, and it is read
out of the document and run by a test so it cannot say something the tests do
not.

**The middle step is a model of the service and not the service.** Every
transformation in it names the section of Microsoft's reference it came from,
read 2026-09-11 from a page dated 2024-11-07 there. What the table measures is
what this program does with what the reference says comes back. Entries 266 to
269 say what that leaves unanswered.

**Three findings against the plan.** Its proposed guard break for task 2 reddens
nothing, measured: comparing a round trip on parsed structures rather than on
bytes cannot fail while the table records what really happens. `05.1-03`'s
summary reports `grep -rn "reqwest\|Outward\|http" src/service/note_document.rs`
as returning nothing and it returns one line, the module header quoting the
command; the same is now true here and is reported rather than repeated. And the
plan's element list omitted `sup`, `sub` and `del`, the last of which is what
`pulldown-cmark` emits for strikethrough, so following it would have cut a style
OneNote supports.

**What 05.1-06 built.** Somebody can add a CardDAV address book from Tools,
"Add an Address Book by Address". The screen asks for the address and the
sign-in, the server is asked what it has away from the thread that draws the
window, and its answer becomes a list to choose from. The address book gets a
row in a new `address_books` table, its sign-in goes to the credential store
under `wixen-mail-carddav-{id}`, and the contacts sync walks an account's
address books and syncs each one, which is the hop that makes everything under
it a feature rather than a library nobody uses.

The word an address book's contacts are filed under is built from its own id
rather than the bare `carddav` the research proposed, because every question the
contact merge asks is keyed on that word and two books sharing one are one book
to all of them. The CardDAV sync decides nothing about whose copy wins: fifteen
functions in `contacts_sync.rs` became `pub(crate)` so it can ask them.

**Three checks caught things nobody asked them to.** The completeness guard
`04-09` built found the new credential owner before any test was written for it,
and was then found to be satisfied by a parameter named after a module; it now
asks for the module followed by a path separator. `outward`'s census refused a
file naming `reqwest` and on no list, so the two CardDAV writes have their verbs
and addresses read off a socket. And its per-client floor of three writes turned
out to be a measurement rather than a rule: a card is created and replaced by the
same request, so this client has two, and the floor moved with the date and the
reason on it.

**PIM-05 is ticked and nothing has met a server.** Its fourth `[D]` line says in
as many words that the transport stays untested until a live account exists.
Ledger entries 260 to 265 say which parts that leaves unknown, one at a time.

**What 05.1-05 built, and the four things it found.** PIM-05 asks contacts to
sync through the vCard reader and writer that already exist rather than a second
pair. There was no function that turned one contact into one card:
`export_contacts_to_vcard` rendered every property inline inside a `for` loop,
so a CardDAV write would have been the second writer the requirement forbids.
The loop's body is now `vcard_block_from_contact` and the loop calls it. The 103
tests in `contacts.rs` pass unchanged and none was added there, because 35 guard
records fingerprint that file.

`src/service/carddav.rs` holds the two request bodies and the two readers, all
hand written scans that resolve nothing, sharing the calendar's three scanning
helpers rather than copying them. Everything it takes out of a document is XML
unescaped, which the calendar's reader does not do; that gap is recorded rather
than reached into from here.

Four findings against the plan. `pub(crate)` with no caller is dead code and
`-D warnings` refuses it, so an item written before its caller has to be `pub`;
the plan's own correction offered both as though they were interchangeable. A
new file cannot join `FILES_THAT_READ_OR_WRITE_A_DOCUMENT` in the commit that
creates it when that commit is the red half, because its shipped half is 36
lines of 363 against a one part in ten assertion, and that was a correction of
2026-09-10 which reviewed the criterion without noticing. Both guard breaks the
plan asked for redden nothing, and the substitutes are written into the records
with the reason. And a fixture committed red asserted something its own name did
not claim, found while writing the code that would have passed it.

Six ledger entries, 254 to 259. Nothing here has met a server and nothing here
can: a card written here and read back here proves the pair agrees with itself
and says nothing about anybody else's. `address_books_in` and `cards_in` have no
caller outside their tests until 05.1-06.

**What 05.1-04 found, and what to carry into 05.1-05 and phase 5.2.** The
largest is that a version marker is not optional. The seam said it was, and the
same absence reads as "treat every copy as moved" on the read half and as
"nothing moved there" on the push half. A backend that gives none therefore
destroys a change made at the other end and says nothing, and no code change
reconciles the two readings without keeping the last copy seen, which PIM-08
forbids. The requirement moved to the backend instead.

Two were bugs in shipped code, and neither was a second backend's problem: the
read wrote the backend's copy over a change the setting had refused, in the same
sync that counted it as waiting; and a note the backend says it no longer holds
was reported as a problem for ever rather than made again. Six thousand eight
hundred library tests passed with either fix removed.

The seam gained one field. `WhatTheBackendSaid::Done` carries what a backend
could keep, because only a backend can tell its own reshaping of what it was
handed from a change somebody made at the other end. The calendar backend uses
it too, for the carriage returns RFC 5545 cannot carry, so a limitation that had
lived in a changelog now reaches the person whose note it is.

Seven ledger entries, 247 to 253. Nothing here has met a server and nothing has
been heard.

**What 05.1-03 built, and the two premises it found false.** The seam had no way
to read a note's words: `notes_it_holds` answered identities and the other two
operations wrote, so a sync built on it could learn that a backend held a note it
had never seen and have nothing to write down. A fourth operation was added. And
nothing could make an account answer `CalDavJournal`, so the whole backend would
have been unreachable from production; an account with a calendar on a calendar
server now answers it, because that server holds journal entries in the same
place under the same sign-in, which is what the plan's own threat register
anticipated.

A third finding is for whoever writes `05.1-04` and `05.2-03`: the round trip
cannot be byte-identical, and the phase README says this backend is the only
candidate whose can be. The format has one escape for a line break and no way to
write a carriage return inside a value, so a body typed on Windows comes back
with plain line endings. Everything else survives, including a trailing space,
which took a reader of its own: the calendar's reader trims.

Nothing here has met a server. Nine ledger entries name the unknowns one at a
time, 238 to 246.

**What 05.1-02 built.** `application::notes_backend`, and three things that stopped
answering the question for themselves. The note folder menu held a constant and a
comment; `new_item::supports` answered it a second time as a `false` with the
reason per provider written out beside it; the settings screen said nothing at
all. The answer is a which rather than a yes or no, with a variant for a word
this build does not recognise, following `AddressBook` for the reason that
type's own comment gives.

Nothing an account can be answers anything but "they stay on this computer", and
that is not a gap. A consumer Gmail account has no notes backend after all three
ship, because Google Keep's API is Workspace only, so the settings sentence says
"this account has no notes backend" rather than "not yet". A sentence written as
"not yet" has to be rewritten later and meanwhile tells somebody to wait for
something that is not coming.

The running program really asks. `wire_context_menu` takes the list rather than a
`Focus`, because a note folder's menu is not a fact about the row alone and a
`Focus` names no account; the notes sidebar reads the default account out of the
window state and asks the seam. The settings dialog gained an `accounts`
parameter for the same reason: `AppConfig` names the default account by id and
holds no roster, and an id says nothing about a provider. That correction was in
the plan and the plan had it wrong.

The one repair worth carrying forward: the remeasure a count check printed found
that "the copy line is on exactly the menus whose command accepts one" stopped
guarding what it says on 2026-09-08, when 05-05 put a copy line on the reminders
menu and turned that record's break into adding a duplicate. It reddened two
duplicate checks it does not name and left the one it does name green, for three
days, with no count moving and nothing able to say so.

**What 05.1-01 built.** Two things, and only the second changes any code that
runs. The first is a test that puts a note's body through `save_note` and
`get_note` and asserts the bytes came back, which PIM-04 has asked for since it
was written and which nothing had: `long_text`'s own round trips go through a
formatter and say nothing about SQLite or about the upsert. Four fixtures, each
named in its own source with the wrong implementation it is aimed at, because a
fixture that is merely large tests nothing.

The second is the `notes.format` column. Of the three answers the requirement
offered, it records why it stays, and dropping it was never available because
`CLAUDE.md` forbids dropping a column that shipped. The argument is in
`NoteBody`'s doc comment where the next reader of the schema meets it: making the
reader obey the column would stop headings and lists being read in every note
that already exists, because every row says "plain" while the editor labels its
box "Body, in Markdown", and `long_text.rs`'s header already settled the question
the column pretends to ask.

What is not a judgement call is that the literal stopped being repeated.
`NoteEntry.format` is a `NoteBody`, so the two production writers cannot spell it
differently: there is no string left to spell. `Other(String)` follows
`AddressBook` so a word this build did not write survives being read and written
back, and a row with nothing in the column at all is read rather than refused.
That last one was the only genuinely red test in the plan.

`outlook_data_file.rs`'s comment claimed that writing the literal kept the markup
reader off text that is not markup. It never did, because nothing consults the
column, and the comment now says what really decides and where that decision was
taken.

Three guard records, all measured by hand tree-wide. The second candidate break
for the column was measured beside the first rather than assumed, and it reddened
a different and larger set, which is `CLAUDE.md`'s warning about the first
candidate turning out to be true.

PIM-04 stays unticked. Its first three `[D]` lines were already true and the
third now has its test; the last three are PIM-07's and belong to 05.1-02 and
05.1-03. Its evidence paragraph in `REQUIREMENTS.md` was reworded by the plan
that made the old wording false. Ledger 231 and 232: nobody has heard a note read
back as structure, and no database written by another build has ever been opened.

### Phase 05, 8 of 8 plans merged

**What 05-05 built.** A reminder can be moved to another account and copied into
one, with the same two keys every other module uses, which makes five of five and
closes criterion 2. "Move a reminder" had no meaning until now because the module
sorts reminders into buckets worked out from when each is due and a bucket is not
a place. The container a reminder really has, and has had since the table was
written, is the account, so there is no new table and no invented concept.

The one real finding is in storage and nothing in the requirement or the decision
says it. `save_reminder` is an upsert whose `ON CONFLICT(id) DO UPDATE SET` list
names eight columns and not `account_id`, so it writes the account on insert and
ignores it on update. The move anybody writes first, change the field and save
it, writes every other column, reports success and moves nothing. The move is
therefore one UPDATE naming `account_id` and `updated_at` with
`WHERE id = ?1 AND account_id <> ?2`, which also makes the three answers
race-free: whether the reminder is somewhere else is asked by the WHERE clause
rather than by a read that can be stale by the time the write runs. `file_under`'s
read-change-write rule was considered and its reason does not hold here, because
that reason is about a sync and reminders sync nowhere.

The chooser is a flat list rather than the destination tree, and the reason is
structural: `build_destination_dialog` pushes `None` for every account row on
purpose, so the tree cannot answer an account at all. Using `pick_one` means
`Moving` needs no new variant and no `ContainerKind` gains a fifth member. The
account is named the way the sidebar names it, its label with the address after
it only where two accounts read alike, which is a deviation from the plan's
acceptance criterion: that quoted a doc comment production has moved past.

`file_under` is never reached, and both halves of that are asserted rather than
assumed. The route is read by a source-text check requiring three arms in the
dispatcher, with the reminder arm below the contact arm so 05-04's check reads
the same text; and a test calls `file_under` on a reminder directly and asserts
it refuses. That second test was green from the moment it was written, because
05-04 closed the arm for all three kinds precisely so this plan would not meet the
same silence.

PIM-02's first `[D]` line now says all five modules, written after all five did
it. The box stays unticked and so does PIM-01's: nobody has heard any of it and
no key has been pressed in the running program. Ledger 211, 212 and 213.

**What 05-02 built.** A week and a month are narrower windows over the query
that already takes a window, not a new query and not a filter in memory.
`CalendarShowing` holds the view and the day it is anchored on, lives in
`WxUIState`, and is handed to every path that reads the calendar back, so no
reload has to guess which window it is refilling. `calendar_heading` names the
window where one was asked for and the rows where none was, which is T-05-07's
mitigation and its search exception in one function. The month is a list over a
month-wide window rather than a grid, because a grid is what PIM-06's third
criterion is written against.

**What did not close.** PIM-06's third `[D]` line, whether a screen reader user
can work through a view's events in date order without reconstructing a grid.
Not building a grid removes the grid from the question; it is not evidence that
the answer is right. `requirements-completed` is empty for that reason.

### Phase 04.1, complete

04.1 (Mail moves between accounts) is **COMPLETE, 4 of 4 plans done and merged.** 04.1-01 a destination is a folder in an account; 04.1-02 a copy that crosses accounts, every account offered, and one branch open; 04.1-03 the move itself, with nothing removed at the source until the destination has answered; 04.1-04 the bytes kept from before the append until the move ends, and a move this program was closed part way through found on the next start. All nine success criteria are closed. Written without the word that starts a Phase line, because only one line in this section may carry that fact.

**What 04.1-04 built.** One additive table, `move_in_flight`, holding a whole
message from before the append until the move reaches any ending, and the thing
that makes it worth having: something that reads what is left over on the next
run. `say_what_did_not_finish` runs when the mail window is ready, reads the
leftover moves locally without touching a server, tells the person which message
did not finish and where it still is, and offers to finish it.
`mail_across_accounts::finish_the_move` asks the destination first, always,
through the same function the ordinary move uses, and appends again only where
the answer is that the message is not there. Nothing is sent on the strength of
a row existing, because a row surviving a restart is exactly as consistent with
an append that landed as with one that never went.

**What the kept bytes buy, said once so nobody has to reconstruct it.** They
protect no message: a move is append then remove and the source holds the
message at every point the program can die. What they buy is a move that can be
finished when the *source* account is the one not answering, and a large message
not fetched twice. `docs/privacy.md` says the negative outright, in the same
commit that created the table.

**Three limits, none of which loses anything.** A message over 25 MB is not
kept and moves anyway on the asking half alone. Interrupted moves are kept up to
64 MB with the newest giving way, because an older row is an offer nobody has
answered yet. And a row nobody answers about goes after seven days, through a
backstop that lives inside `moves_that_did_not_finish` rather than in a function
of its own: this cache has no sweep, and an eviction rule with no caller never
applies to anything, which is not hypothetical here.

**Fourteen ledger entries are open from this phase, 179 to 192**, and none has
been answered. Nine of them want a screen reader or a real account. 191 and 192
are 04.1-04's: the program has never been killed mid-move and restarted, so the
case the whole store exists for has never happened outside a test, and the
question put on the next start has never been heard.

**Both human checkpoints were not run and are recorded as unrun verifications**,
at Pratik's instruction that manual testing happens later. 04.1-03's is written
out in `04.1-03-SUMMARY.md` and 04.1-04's in `04.1-04-SUMMARY.md`, each with
what to do, what to listen for, and what each outcome means. Two answers stop
04.1-04's rather than continuing: nothing being said at all on the next start,
which means the store is written and read by nothing, and the message arriving
twice, which means the resume appended without asking.

**Owed after the merge, and not on the critical path:**
`scripts/guards.sh --touched-by 611f7cf`, which is where `main` stood before
04.1-01 and subsumes all four per-plan deferrals. Every commit in the phase ran
the scoped remedy its own commit printed, which is what makes that affordable.

**`progress.completed_plans` is 65**, counted from the `*-SUMMARY.md` files on
disk rather than incremented, which is the route every plan since 04.2-05 has
taken. It said 64 against 65 files until 05-02, because 05-01 merged without
updating this file; the count is taken from disk again rather than carried
forward, which is why the gap closed in one step instead of drifting further.

### Phase 04.2, complete

The phase before this one, 04.2 (What was built and never reached), is complete: 9 of 9 plans done and merged. 04.2-01 Undo Send, 04.2-02 scheduled send, 04.2-03 the meeting reply, 04.2-04 the meeting reaching the calendar, 04.2-05 the count of held-back pictures, 04.2-06 Blocked Senders, 04.2-07 `Shift+F6` out of the message preview, 04.2-08 a column layout belonging to the kind of folder it was arranged in, 04.2-09 the documents naming the keys that work. All fourteen success criteria are closed. `main` is at `3fc3ab2`, version `0.87.0`.

**What the phase left open is written out in one place**, in `04.2-09-SUMMARY.md` under "Closing out phase 4.2": every one of the twenty-eight ledger entries the phase opened, none of which has been answered; the two screen reader checkpoints that were deferred by decision and never attempted; the braille defect, whose honest half is done and whose durable fix belongs to phase 6 along with two tests whose names promise something that cannot be built through the call this program uses; and the one guard sweep the phase owes.

**The guard sweep, in one command:** `scripts/guards.sh --touched-by 9611b70`. That commit is where `main` stood before 04.2-01, so it subsumes all nine per-plan deferrals, five of which never recorded a base commit of their own. The 28 source files the phase touched are listed in the summary. Carried as a phase 8 criterion by the decision of 2026-09-03; it does not block anything.

**Nothing in this phase has been heard, and nothing has met a real server.** Fifteen of the open ledger entries want a screen reader and seven want a real account, a real organiser, a real provider or a second machine.

**04.2-06 owes a screen reader pass that was not run.** The window, the menu item, the scan target and the wiring are all in and green, and nothing has been heard. Both Windows accessibility channels and five listening checks are written out step by step in `04.2-06-SUMMARY.md` under "Deferred: the screen reader pass, not attempted", and as ledger entries 159 to 163. That is deferred by decision, not missed.

**04.2-07 owes one run of the program and one screen reader pass.** Whether the browser hands `Shift+F6` to the injected handler with the shift flag set is untested, and this tree already records a key being kept between the window and the page in `editor_document.rs`. Whether landing on the folder tree is heard as going back rather than only as going somewhere is the other. Ledger 164 and 165, and both are answered by the same run.

**04.2-08 owes one run of the program and one screen reader pass.** Nothing in it has been seen running: no folder was opened, no column heading clicked, no `Alt+R` pressed. Separately, moving between folders now changes which columns are shown and how the list is sorted, and nothing announces it, because the columns are the list control's own headers. Ledger 167 and 168, and both are answered by the same run.

`current_plan` was 4 when 04.2-05 finished, because 04.2-04 completed without running the state update. It was set to 6 by hand for that reason: `state.advance-plan` only increments, and one increment from 4 would have said the next plan was 05, which is done. It was set to 7 by hand when 04.2-06 finished, for the same reason and by the same route, to 8 by hand when 04.2-07 finished, and to 9 by hand when 04.2-08 finished. It stays at 9 now that 04.2-09 has finished, because 9 is the last plan of the phase; `status` says `phase-complete` rather than the counter being pushed past the end.

**The frontmatter and this heading had come apart again by 04.2-07**, which is the exact failure the paragraph below is about. The frontmatter said `current_plan: 6` and `stopped_at: Completed 04.2-05-PLAN.md` while this section said 7: 04.2-06 updated the heading and left the frontmatter behind. Both are corrected to 8 by hand.

Phase 04.2 finished on its ninth plan of nine, which is why `status` said
`phase-complete` rather than the counter being pushed past the end.

**`progress.completed_plans` was 59 at the end of phase 04.2, counted from the `*-SUMMARY.md` files on
disk rather than incremented**, which is the same route every plan since 04.2-05
has taken, because incrementing a stale number keeps it stale.
`progress.total_plans` says 87 and there are 85 `*-PLAN.md` files; that gap is
older than this phase and is left alone rather than guessed at.
`progress.percent` is 0 and has been through every phase.

**`roadmap.update-plan-progress` was not run.** It has written wrong values
twice, and ROADMAP.md's phase 4.2 checkboxes are ticked by hand for the same
reason the counter above is set by hand.

**The Performance Metrics section below was not added to.** Its own note says
the figures predate 01-10 and nothing recalculates them, and its By Phase table
holds one placeholder row. Adding a row to a table nobody maintains would read
as a maintained table. Duration, task count, commit count and the realized token
figure for this plan are in `04.2-07-SUMMARY.md`'s frontmatter.

**Do not run `gsd-tools query state.advance-plan` to find out where things are.** Despite the `query` prefix it is not a read: it advances the counter and rewrites six fields. Running it once on 2026-09-07 to test whether this section parsed moved the plan from 4 to 5 with nothing executed, and it had to be put back by hand because there is no `state-set` command to undo it.

**This section was rewritten on 2026-09-07 and had been stale for five phases.** It said phase 02, EXECUTED, with 02-07, 02-08 and 02-09 sitting on unmerged branches. All three merged long ago and four phases have shipped since. Two executors read it, recorded it as stale and declared it out of scope, which was the right call for them and meant nobody fixed it. It is written down here because the frontmatter and this heading disagreeing is exactly what made `gsd-tools query state.advance-plan` fail once before, on 2026-09-01, and tag new decisions against the wrong phase.

The history below this point is kept as a record and is not current. Read the frontmatter for what is true now.

### Phases complete

Phase 01 Folders and conversations, 14 plans. Phase 02 Search that says what it covers, 9 plans. Phase 02.1 What phase 1 found on its way past, 9 plans. Phase 03 Mail at scale on the wire, 9 plans. Phase 04 Writing and reading a message in full, 9 plans. All executed and merged; all awaiting the screen reader and live account verification that only Pratik can run.

### Phases planned and not started

04.1 Mail moves between accounts, context only. 05 The other five modules keep up, 8 plans. 05.1 Notes and contacts reach a server, 6 plans. 05.2 Notes in OneNote, 3 plans. 07 Installing, updating and what is stored, 9 plans. 06 and 08 are researched and deliberately not planned until the tree they measure exists.

---

## History below this line

Everything that follows was written earlier and describes trees that have moved. It is kept because the reasoning in it is still worth having, not because the status lines are true.
02-09 was added on 2026-09-01 after the phase verification recorded criterion 4
as the one partial of six: the saved search said what it covered and the search
box, which is the search people reach for first, said nothing. It closes that.
The verification report should be re-run against it rather than read as current.
Corrected on 2026-09-01: this header and the frontmatter both said phase 01
while five phase 02 plans had shipped, which is what made
`gsd-tools query state.advance-plan` fail and tagged new decisions `[Phase 01]`.
Phase 01 is complete and awaiting re-verification, recorded below; that is what
`current_phase` was being used to remember, and it is not what the field means.

Phase: 01 (Folders and conversations). EXECUTED, awaiting re-verification
Plans: 14, one per wave, `01-01-PLAN.md` to `01-14-PLAN.md`. 40 tasks, of which
37 are RED-first, 1 is configuration-only (`guards/guards.toml` records) and 2
are blocking human gates, in 01-02 and 01-07, both over one-way writes to the
only copy of the user's mail. Those two plans are `autonomous: false`.
Status: Ready to execute
verification recorded criterion 3 as the one partial of eight, and it closes it.
The phase wants re-verifying against that report, which is annotated as
superseded rather than left to be read as current.

**Phase 02 has context as of 2026-08-31** and is ready to plan. Phase 01 is left
Pending deliberately: its code is merged, pushed and green, and what keeps
FOLDER-02 open is a screen reader announcing a folder's level from the native
control, which no test here can answer and which is Pratik's to run.

**Phase 1 reviewed 2026-08-29 with Pratik.** Two criteria changed:

- FOLDER-01 said all five folder operations pass through `Allowed::mail`. Wrong for POP
  accounts, which have no server folders at all, and for the IMAP outbox.
  `local_folders::is_local` already draws that line and is now named as the single place that
  decides it. Server folders keep the gate; local ones do not.

- FOLDER-03 keeps local pinning as this phase's work, and now says the stored shape must let
  IMAP subscription back it later, with the decision about which wins recorded before the
  second half is built rather than settled by whichever code path runs last.

**Phases 2 to 8 reviewed 2026-08-29.** Every `[D]` criterion was read back against the tree.
Two decisions were answered, two requirements were split, and six criteria were corrected for
being untestable as written. Recorded here because a criterion nothing can satisfy passes a
review that only reads it.

| Requirement | What was wrong | What it says now |
|-------------|----------------|------------------|
| SEARCH-01 | Said the scope selector is read by nothing, quoting a changelog line since corrected as false, and cited a test as a second production site | The live search honours all four scopes and has tests. A saved search keeps the folder half and not the field half, because `what_a_typed_search_asks` always writes `["subject", "from", "to"]` |
| SCALE-02 | Cited `opening.rs` and `attaching.rs`, which exist and open no connections | Names `a_session_at`, its three callers, and the eight sites in `wx_app.rs` that bypass it |
| FEEDBACK-01 | Asked for the removal of a `tests/house_style.rs` exception that does not exist; the real constant it echoed guards documents about a different setting | Asks for a test that counts the screens reaching `set_event_channels` |
| SHIP-01 | Required that SmartScreen stop warning, which a valid signature does not buy | Separates the signature, which is verifiable, from reputation, which only an EV certificate carries and EV is the last resort here |
| SHIP-05 | Mixed building on a platform with disclosing what does not work there | Split. SHIP-05 is the build and its CI jobs; SHIP-06 is the disclosure |
| PERF-06 | Asked that a number in a document equal what `cargo test` reports, which is false the next time anyone adds a test | Asks that every count carry its command and its date, and that documents agree with each other |

Two decisions were also answered in the same pass and are recorded under Blockers below:
PIM-04's notes backend, which split into PIM-04, PIM-07 and PIM-08, and SHIP-04's encryption
question, answered as a recorded decision not to.

Two things the review found that were not criteria at all. The traceability table had 40 rows
against 44 headings, having missed every requirement added by a split; it is now generated from
the headings so it cannot drift again. And three documents gave three different test counts, of
which the newest, 5,269, was the unit count wearing the label of the total. The suite is 5,430:
5,269 unit and 161 integration, from `cargo test --all-targets -- --list` on 2026-08-29.

Last activity: 2026-09-11

Written as one line because `gsd-tools query state.record-session` copies this
paragraph's first physical line into the frontmatter and stops there, so a
sentence wrapped across two lines arrives in the frontmatter as a broken half.
It did that twice here before this was noticed.

The sixteen checks in `tests/wired.rs` that read `wx_app.rs` go through
`common::what_ships`, which is behind a cargo feature a dev-dependency of this
package on itself turns on. Measured both ways with a positive control in each
run, because the first two attempts counted zero off a warm tree that compiled
nothing: a test build enables the feature four times and a release build never.

Nothing the widened reading uncovered failed, and the size the phase was scoped
against is wrong by a factor of about seventeen. 6,865 lines sat below the cut,
6,476 of them test modules, so the real exposure was 389 lines of production
code. Two things worth carrying. Putting a prefix cut back into any of the
twelve reddens nothing, so the twelve are unguarded against going back and only
the one new test would notice. And the self dev-dependency reddened a dependency
census three commits after it landed, because `which-checks.sh` maps a changed
file to a module by path, `Cargo.toml` maps to none, and so a manifest change
runs no test that reads the manifest. That is the same shape as the markdown
case the script already knows about. Task 4 of the plan is a decision waiting on
Pratik; nothing found has been fixed or recorded.

Previous activity: 2026-09-01, 02-09 done and the gap phase verification found
on 2026-09-01 closed. The search box now says how much of the account's message
text it could look inside, on the same line as the match count, and a search
that finds nothing gets the sentence on its own. The number is the box's own.
The plan's central premise held under measurement: an evicted message is gone
from `message_bodies` and still in the index, so the two searches really do
cover different amounts of one mailbox. Its second-order premise did not: the
box's number is not a query, because `message_search` is contentless and its
body column reads as NULL however much is indexed. It is recorded in a column
written by the one function that decides what the index holds. Nothing has been
heard under a screen reader.

Progress: [░░░░░░░░░░] 0%
percentage above counts phases and this line counts plans, so the two have said
different things about the same work all through this phase; the plan count read
7 while 11 were done, which is how long a number nothing recomputes can sit here
being wrong.

The suite is 5,986 unit as of `cargo test --all-targets --no-fail-fast` on
2026-09-01, with one ignored and every integration target green in the same run.
02-07 added 30 of those, 02-08 added 16 and 02-09 added 12, the last across
`data::message_cache::searching`, `application::saved_searches` and
`presentation::managers`. The release build has not been run on the 02-07,
02-08 or 02-09 branches: `scripts/check.sh all` is the merge gate and it is
Pratik's.

The counter above could not be advanced by `gsd-tools query state.advance-plan`
on 2026-09-01: it looks for "Current Plan" and "Total Plans in Phase" lines this
file does not have, so it reported that it could not parse them and changed
nothing. These two lines were written by hand instead. That is the same fault
the paragraph above describes, seen from the tooling's side.

## Performance Metrics

**Velocity:**

- Total plans completed: 14
- Average duration: not recomputed; the figures below predate 01-10 and nothing
  recalculates them, which is the same fault the progress note above records

- Total execution time: not recomputed, for the same reason

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 8 plans: 01-01 (3h 5m), 01-02 (1h 10m), 01-03 (1h 0m), 01-04 (4h 10m),
  01-05 (1h 38m), 01-06 (1h 4m), 01-07 (1h 31m), 01-08 (2h 5m),
  01-10 (2h 40m)

- Trend: no trend, and the spread is the finding. The three fast plans used
  targeted test runs, 1 second against about 175, for every red and green step,
  and spent their full library runs only on measuring guard records by hand.
  The two slow ones were slow for different reasons worth telling apart: 01-01
  ran the whole library on every check, which is waste, while 01-04 was a large
  plan with four wrong premises to find, which is work. Duration alone cannot
  tell those two apart, so it is a poor measure of anything on its own.

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 3h 5m | 3 tasks | 8 files |
| Phase 01 P02 | 1h 10m | 3 tasks | 9 files |
| Phase 01 P03 | 1h 0m | 3 tasks | 9 files |
| Phase 01 P04 | 4h 10m | 3 tasks | 16 files |
| Phase 01 P05 | 1h 38m | 3 tasks | 13 files |
| Phase 01 P06 | 1h 4m | 3 tasks | 15 files |
| Phase 01 P07 | 1h 31m | 3 tasks | 14 files |
| Phase 01 P08 | 2h 5m | 3 tasks | 13 files |
| Phase 01 P09 | one session | 3 tasks | 11 files |
| Phase 01 P11 | 3h 5m | 3 tasks | 12 files |
| Phase 01 P12 | 4h 5m | 3 tasks | 17 files |
| Phase 02 P03 | one session | 3 tasks | 7 files |
| Phase 02 P04 | one session | 3 tasks | 7 files |
| Phase 02 P05 | one session | 3 tasks | 6 files |
| Phase 02 P06 | one session | 3 tasks | 5 files |
| Phase 02 P07 | one session | 3 tasks | 13 files |
| Phase 02 P09 | one session | 2 tasks | 6 files |
| Phase 02.1 P01 | 3h | 3 tasks | 6 files |
| Phase 02.1 P02 | 2h | 3 tasks | 7 files |
| Phase 02.1 P03 | 2h | 2 tasks | 6 files |
| Phase 02.1 P04 | 4h30m | 2 tasks | 6 files |
| Phase 02.1 P05 | about 3h | 3 tasks | 11 files |
| Phase 02.1 P06 | about 2h30m, 90m of it guard runs | 3 tasks | 6 files |
| Phase 02.1 P07 | about 1h | 2 tasks | 6 files |
| Phase 02.1 P08 | about 5h, 1h50m of it guard runs | 3 tasks | 8 files |
| Phase 04 P05 | 4h 0m | 2 tasks | 22 files |
| Phase 04 P06 | about three hours | 2 tasks | 9 files |
| Phase 04.2 P05 | one session | 2 tasks | 9 files |
| Phase 04.2 P08 | about 3h | 3 tasks | 11 files |
| Phase 05 P03 | 210 | 2 tasks | 17 files |
| Phase 05 P06 | 3h | 3 tasks | 12 files |
| Phase 05 P08 | 5h | 2 tasks | 12 files |
| Phase 05.1 P01 | 64 | 2 tasks | 11 files |
| Phase 05.1 P02 | 2h 20m | 3 tasks | 12 files |
| Phase 05.1 P04 | 1h 35m | 3 tasks | 9 files |
| Phase 05.1 P06 | 155min | 3 tasks | 21 files |
| Phase 05.2 P02 | 155min | 4 tasks | 11 files |
| Phase 07 P04 | 80min | 2 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in the PROJECT.md Key Decisions table. The ones that shape the phases
ahead:

- `ReleaseChannel` is derived from the stored setting and is never written to a file, so
  there is one stored spelling to keep rather than two. What a settings file holds is
  `WhichUpdates`, spelled `not_looking`, `public_releases` or `development_releases`, fixed
  from 07-04 onward because renaming a stored value makes a machine holding it unreadable.
- A version string this program accepts may carry a leading `v`, because a published tag
  does. The evidence is `release.yml`'s `wixen-mail-v*.exe` glob rather than a tag, since no
  release has ever been cut.
- No EWS. Microsoft blocks third-party EWS from 1 October 2026. Exchange goes through Graph.
- Writes split into `mail` and `personal_information` in `src/application/allowed.rs`, with
  three places that must agree. Mail writes are off for a new install.

- The message list stays native virtual mode, because only the native control gives UI
  Automation the real set size.

- The folder tree holds every account at once, and whose mail a command acts on is taken from
  the row under the cursor rather than from a separately held "open account". Those two were
  the same answer while the tree drew one account's folders, and stopped being the moment it
  drew them all (01-14).

- Moving between accounts must stay a selection rather than a rebuild, guarded by a record
  whose break is the wrong fix somebody would reach for. Rebuilding on selection would make
  everything downstream agree and would cost five cache reads per account on every arrow key.

- The cached mail database is not encrypted, and the docs say so. Phase 7 decides whether that
  changes.

- Two windows that must not open over each other share one gate rather than holding one each.
  `application::due::OneAtATime` now gates the reminder alerts and the question about folders a
  server has stopped listing, because a gate each is exactly what lets either open over the
  other.

- What a server last said about a folder is three answers, not two: it listed it, it stopped
  listing it, and it stopped listing it and somebody said keep it. Without the third, answering
  No and closing the window are the same thing.

- A pin and a server subscription are two questions, so neither overrules the other. Pinning
  never writes a subscription and a subscription changing never adds or removes a pin. Recorded
  in `src/application/favourites.rs` and in PROJECT.md before the storage shape was fixed, which
  is what FOLDER-03 asked for.

- Favourites are keyed on `(account_id, path)` with both cascades. A rename rewrites a folder's
  path, so `ON UPDATE CASCADE` is what makes D-32's "a rename keeps the pin" true rather than a
  second writer nobody remembers.

- [Phase 01]: A new folder is made where it is named, not under the cursor: the IMAP hierarchy delimiter is dropped after list_folders by design, and guessing it writes the folder elsewhere on a dot-separated server. Nesting is 01-03 and 01-05.
- [Phase 01]: Making a folder records it in the cache, subscribes it, and marks it as one to keep up to date. The tree is read from the cache, and an unsubscribed folder is hidden, so creating on the server alone would show nothing.
- [Phase 01]: MAIL_TRANSPORTS' imap floor rises with every gated write added. Left at 8 with nine writes present, it had already cost the copy_message gate guard one of its three reddening tests.

- [Phase 01]: D-39 confirmed at the gate by Pratik, conditional on no existing threading being lost. The condition was checked and holds: threading.rs is untouched, so the conversation view behind Enter is unchanged. thread_id is derived from the first identifier of the References chain, computed once when a message is stored.
- [Phase 01]: The conversation id is written at one call site, inside upsert_message, not at both named entry paths. file_message_here is upsert_message plus one UPDATE, so a second call there would build the duplication as_stored's doc comment forbids. Any later plan told to write a derived value "in both places" should read the second place's body first.
- [Phase 01]: A guard for an invariant of the form "exactly one place does this" breaks by ADDING a competing copy, not by deleting the one. Deleting proves only that the value is computed at all. guards.toml now holds one record of each shape for this column.
- [Phase 01]: 01-03: CachedFolder gained no parent_id field; the phase's consumers (01-04, 01-05) both take the folder_parents map, and the field would have been 79 struct-literal edits with no reader
- [Phase 01]: 01-03: is_a_name_that_can_be_used is asked of each part between separators, because safe_file_name reads a separator as a path, so asking it about the whole name would refuse the one character D-23 allows
- [Phase 01]: 01-04: rename_mailbox and delete_mailbox take the server's own spelling verbatim and encode nothing. A path from the cache is already modified UTF-7; only a segment somebody typed is encoded. The plan's instruction to encode both RENAME arguments would have named a mailbox the server has not got.
- [Phase 01]: 01-04: the hierarchy separator 01-03 read is not persisted, so no command running from the cache can see it. A rename reads it from the gap between a folder's path and its parent's; a move reads it off a LIST line on the worker, after the confirmation, so D-37 still holds.
- [Phase 01]: 01-04: folders kept on this computer are a const array of &'static str, so renaming, moving and deleting one are refused with a sentence rather than half-built. 01-06 builds user-named local folders.
- [Phase 01]: 01-04: a behaviour RED is taken by stubbing the body and reading which assertions fail. A compile error proves a symbol was absent, not that the assertions discriminate; four tests here stayed green against a stub returning nothing.
- [Phase 01]: 01-05: the 01-03 regression was worse than its changelog said. folder_ids is a HashMap keyed on the row's displayed text, so two folders sharing a leaf were one entry and one of them opened the other's mail. Closing it meant keying on identity, not nesting the display.
- [Phase 01]: 01-05: selected_folder became the WhichRow enum rather than an identity string, so the compiler enumerates every consumer that read it as display text. Four did, and two of those were already broken: Get Older Messages and writing a mailbox out both passed the row's words where a folder path belonged.
- [Phase 01]: 01-05: wxdragon's TreeItemId has no PartialEq and no public pointer, so two tree items cannot be compared and a row can only be interrogated by its text. That is why this codebase was label-keyed. A row is paired to its identity by the chain of labels above it, which is unique because siblings cannot share a path.
- [Phase 01]: 01-05: no per-archive branch was built (D-21). Nothing records which archive an imported folder came from, so the plan's archives parameter has no producer. Building it is work in import_tree at import time; an enum variant nothing can construct would be a stub.
- [Phase 01]: 01-05: only the account being looked at gets a branch. D-13's property is proven of folder_tree::rows, which is multi-account throughout, but drawing every account at once before D-18 gives one Drafts, Sent and Outbox row per account, which is the duplicate-rows fault this plan removes.
- [Phase 01]: 01-06: the D-43 mirror guard was red on arrival naming six settings, and only allowed_per_account is the defect. It is stored, read by allowed_for and honoured out to the provider clients, and no screen has offered it since the testing page stopped naming an account. Named in an exception list with the reason rather than the guard being narrowed until it could not see it.
- [Phase 01]: 01-06: an exception list is the part of a check most likely to rot, so each exception carries a claim the check tests. One test reads the screen an exception names and fails if the control has gone; another fails when the recorded defect stops being one, so whoever fixes it is told to delete the entry.
- [Phase 01]: 01-06: the plan's unread_text would have had no caller and both settings guards would still have passed, because a module reading its own setting counts as a reader and a control counts as an offer. Reachability is a third question no test of the parts asks. rows() now takes the setting and what is closed, and the expand handler words that one row again.
- [Phase 01]: 01-06: accounts order by tree_order IS NULL, tree_order, created_at, so an untouched database keeps arrival order and an account added after a move goes to the end. The move writes every ordinal, not the two that swapped, because a list half ordered by choice and half by arrival reorders itself the next time an account is added.
- [Phase 01]: D-18 was self-contradictory about the Outbox and Pratik corrected it: shared for everyone, FOR_IMAP empty, one send queue on this computer
- [Phase 01]: The merge of the local folders reuses move_message rather than a second mover, because a separate one would have missed filed_here and written rows the next sync deletes
- [Phase 01]: The merge records both the original uid and the original account for every moved message, so it is reversible from the data even though no command undoes it
- [Phase 01]: Emptying asks both functions that decide what deleting means, local and server, and carries all their answers across; a single AtTheServer variant was written first and would have destroyed an Inbox
- [Phase 01]: The empty count and the empty walk both skip messages already soft-deleted, so running Empty twice is a no-op and an emptied Trash stops reading as full
- [Phase 01]: A setting ships in one commit with its screen and its consumer; splitting them leaves the two settings guards red with no honest way to satisfy them
- [Phase 01]: A conversation is named by its oldest message present, from mail_parser's RFC 5256 base subject, and the compose box asks the same module whether a subject already carries a marker
- [Phase 01]: The Subject column's conversation sort calls a Rust function registered on the SQLite connection, because a chain of markers in seventeen languages has no SQL expression
- [Phase 01]: conversations_in takes an order_by so the sort expression has a caller and the agreement between a cell and its sort is run rather than described
- [Phase 01]: The all-mail exclusion applies only to the account-wide reach, because counting one folder cannot double anything
- [Phase 01]: 01-12: the message list collapses to one row per conversation, per folder, and Thread View is no longer a disabled menu item
- [Phase 01]: 01-12: opening a folder row no longer deletes its tree_state row outright, because the view D-09 stores there would have gone with it the first time somebody expanded the folder
- [Phase 01]: 01-12: the count the virtual list is told has one writer, because that number is the set size UI Automation reports and a second writer is a wrong announcement nobody can see
- [Phase 01]: 01-12: the selection is held across a view switch rather than recomputed, because a conversation cannot say which of its messages was chosen
- [Phase 01]: 01-12: a subtree is read from parent_id and never from a path, because the hierarchy separator is not persisted
- [Phase 01]: 01-12: a conversation row confirms a delete where a single message does not, because its contents are off screen and the column that would say how many can be switched off
- [Phase 01]: 02-03: the offer to fetch missing message text counts the fetch list, not the difference between the two coverage numbers, because mail with no server to ask is missing text that no fetch can supply
- [Phase 01]: 02-03: a read-gate refusal stops a backfill and an ordinary failure does not, told apart by service::outward::was_refused_by_the_gate
- [Phase 01]: SEARCH-01 is met by keeping the In box's answer beside the typed words and writing both halves of the scope in one call: D-2-03's narrower question set and D-2-14's folder, with no schema change
- [Phase 01]: A folder stored on a saved search carries the account it belongs to, and saving one whose account disagrees with the search's is refused rather than saved without the folder, because Set Active can change one and leave the other
- [Phase 01]: A saved search's whole description is written back in one transaction, with the questions written first and the row stamped last, so the one failure a person can cause lands inside the window the transaction protects rather than outside it
- [Phase 01]: The two things a saved search misses stay two sentences, with a test asserting they differ: mail thrown away is never gathered, and evicted message text stays findable from the search box and not here (D-2-13)
- [Phase 01]: No changelog entry for 02-06, because nothing it builds is reachable; the entry belongs to 02-07, which opens the dialog and calls the replace
- [Phase 02]: 02-07: a saved search's row reports its own focus on the menu key, rather than an entry being filtered out of the folder list. Nothing in the codebase decided menu entries per row.
- [Phase 02]: 02-07: a search a newer version wrote keeps the Edit conditions entry and is refused with a sentence, in the wording Enter on the same row already gives.
- [Phase 02]: 02-07: every change to a condition list says how many are left, on the end of the one sentence. Only a condition list counts out loud, decided from the kind rather than passed in.
- [Phase 02]: A saved-search command takes its account from the row under the cursor, never from active_account_id, and run_a_saved_search is not handed the held state at all. A signature that cannot see the wrong answer is a guarantee; a check that the body does not read it is a reading somebody has to trust.
- [Phase 02]: Every group that mirrors the account structure is ordered by one function, favourites::what_each_account_has. Two groupings of the same accounts is the shape that comes apart, and the test compares the two groups' orders rather than restating either.
- [Phase 02]: The search box's coverage is recorded in a column written where the index body is decided, because the FTS index is contentless and cannot be asked what it holds
- [Phase 02]: 02.1-01: common::what_ships is gated on a cargo feature alone, not that and cfg(test). Two conditions would keep the library's unit tests green while integration tests failed to compile.
- [Phase 02]: 02.1-01: wixen-mail goes on service::outward's A_CRATE_THAT_CANNOT list. The other list would oblige the census to look for a path root starting wixen_mail, changing what counts as a way out across the tree for a path no file under src writes.
- [Phase 02]: 02.1-01: the six guard records naming tests/wired.rs were re-measured in the green commit rather than at the end of the plan, because the count check reddens on the commit that adds a test and re-measuring needs a green tree.
- [Phase 02]: 02.1-01: the new guard record's break is a prefix cut put back into the new test, because that is the only site where putting one back reddens anything. Measured: putting it back into any of the twelve reddens nothing, so they are unguarded and the record says so.
- [Phase 02]: 02.1-08: a folder tree row with nothing true of it gets no menu, not a short one. `wire_context_menu` takes a closure answering `Option<Focus>`, so absence is carried by the type and `Focus` keeps meaning "a place with a menu", which is what `test_everything_that_can_hold_focus_offers_something` is about. Diverges from a sentence of D-2.1-03 and keeps its reasoning, and follows the precedent already recorded for the reminders sidebar.
- [Phase 02]: 02.1-08: Choose Folders moves onto the account menus rather than off them. Its handler reads the open account and never the row, so it was never a folder's command, and the test written for it is the general rule: a command that reads the open account is offered only where landing on the row set the open account from the row.
- [Phase 02]: 02.1-08: an account's address goes on its branch only where another account shares its name, decided by `folder_tree` from the set being drawn rather than by the caller per account. That reverses what shipped, where `display_name` put the address on every branch and a screen reader read it out every time.
- [Phase 02]: 02.1-09: there are two condition editors, not one. `show_rule_edit` has one caller and the filter manager has `show_filter_edit`, its own function with the same `unwrap_or_default` read-back. Both refuse, and the check that holds them there names both functions and both builders so neither can be fixed while the other is reported as fixed.
- [Phase 02]: 02.1-09: `what_a_condition_cannot_read` is the one reading and `SavedSearch::what_it_cannot_read` delegates to it, rather than the dialogs growing their own. The coarse-grained caller already existed; the new function is the per-condition grain it was missing.
- [Phase 02]: 02.1-09: the refusal names a newer version conditionally rather than asserting one. A word this build has never met arrives from a version that has met it or from something that wrote it wrong, and this program is on that second list, so "open it in the version that wrote it" would send somebody back here.
- [Phase 02]: 02.1-09: the guard record's break is the answer emptied, five tests red, and the record writes down what it does not reach: removing the pre-open refusal instead reddens exactly one, because every path that tells a refusal from an opening ends at `show_modal` and a test that opened the editor would hang the gate rather than fail it.
- [Phase 03]: 03-08: the network coming back raises an offer, never a send. The decision layer has no member meaning 'send', the offer is a button in the tab order whose label says pressing it sends, it calls the existing online toggle and the existing flush rather than reimplementing either, and a census counts every place that hands mail to a server and names each beside what asked for it. That is the shape 03-09 should copy for waiting flag changes.
- [Phase 03]: 03-08: a check for a resource going away must not live on the work that uses the resource. Nothing here checks mail on a schedule and every trigger that does stops when the network goes, so a detector at the end of a mail check could see the loss and never the return. It asks on the window's own poll every ten seconds.
- [Phase 04]: An attachment's description is three states, not an Option: silence, something unreadable, and the sender's words. A broken sending program is not the sender having said nothing, and the row says which.
- [Phase 04]: A description written on the img in the message is borrowed only when the sender wrote no Content-Description, and is matched by content id rather than by position. An explicit header is the sender saying it; a borrowed one is a guess about which element meant which part.
- [Phase 04]: 04-02: a library's parsed form of a header and its raw form are different values, and which is right is decided by the consumer. mail-parser sends List-Unsubscribe to its address parser, which strips the angle brackets blocking::where_to_write_to_leave searches for. Reusing the accessor the neighbouring field uses would have shipped a feature that reported every mailing list as one that gave no way out, with everything green.
- [Phase 04]: 04-02: a request that names the fields it wants is a hop no test on either side of it can see. IMAP's HEADER_FIELDS did not name LIST-UNSUBSCRIBE, so the whole feature would have been dead on IMAP with 6270 tests passing. Look for the same shape wherever a projection is narrowed: a SELECT column list, a GraphQL selection set, a fields= parameter.
- [Phase 04]: 04-02: "this task has no red available" is a claim about the tree, not a property of the task. The plan said its census could not be red because it must name a construction task 1 creates; the construction already existed and only its argument changed, so the census was red before any implementation. Ask what specifically does not exist yet before accepting the claim.
- [Phase 04]: The decorative mark is an explicit empty alt and nothing else. role=presentation does not survive ammonia and the sanitiser was not widened for it.
- [Phase 04]: announce_decorative_pictures ships on: a line you did not need is noise you can switch off, a picture you were never told about cannot be asked about.
- [Phase 04.2]: Undo Send: the clock asks about an edge, not a level, so a message that failed to send is never retried once a second, and turning the hold on cannot make more mail leave than turning it off would. That is what lets a timer flush past guardrail 7.
- [Phase 04.2]: The held sentence is countdown() alone and does not name the recipient. Every word costs, because the announcement has to finish before somebody knows there is anything to undo and the hold is ten seconds.
- [Phase 04.2]: The scheduled-send changelog sentence is left false on purpose for 04.2-02 rather than moved to Known limitations and moved back one plan later.
- [Phase 04.2]: The renderer carries whose message it is showing, and the composer says so through a constructor rather than by calling an extra method
- [Phase 04.2]: The held-back count goes under each message's heading in a conversation, not once as a total for the page
- [Phase 04.2]: The plain text reading path says nothing about held-back pictures, because nothing was held back there
- [Phase 05]: A copy and a move are one path with three answers on Filing, not two paths: the container it is in is offered, the holder is not asked, and the write makes a new row
- [Phase 05]: The module-following copy stayed in the Copy to submenu, because the Action menu has only j, q, x and z left and none is a guessable letter for copy
- [Phase 05]: 05-08: the task arm of moving_can_be_told is false, not a narrower refusal; the event arm is untouched
- [Phase 05]: 05-08: a provider-held move into a list made on this computer is allowed, tested and reported rather than refused; the person can put it right by moving it into a synced list
- [Phase 05]: 05-08: whether anything is waiting is two questions with two subjects, so a_removal_will_have_to_be_sent sits beside will_have_to_be_sent rather than widening it
- [Phase 05.1]: 05.1-01: the notes.format column records why it stays rather than being given meaning, because a reader that obeyed it would stop reading headings and lists in every note that already exists
- [Phase 05.1]: 05.1-01: NoteEntry.format is an enum rather than a String with a shared constant, so a second writer has no literal left to spell
- [Phase 05.1]: 05.1-01: one version bump for a whole plan rather than one per task, because bumping twice for one plan uses the version as a build counter
- [Phase 05.1]: An empty change marker and an absent one are one answer, decided once for an address book's ctag and a card's version marker together: the one XML extractor this program has collapses both, and a second extractor is a second thing to keep working for a distinction nobody acts on
- [Phase 05.1]: A non multistatus CardDAV answer is refused with Error::Protocol and a fixed sentence, following parse_report_events. parse_propfind_calendars refuses nothing and the transport's refusal is unreachable from a file that touches no network
- [Phase 05.1]: A new file joins a whole tree check in the commit where it first has the shipped code the check is about, not in the commit that created it: a test first commit has no production half for a proportion assertion to read

### Pending Todos

None yet.

### Blockers/Concerns

- ~~**Awaiting review, phases 2 to 8.**~~ Reviewed 2026-08-29. Every acceptance criterion
  marked **[D]** was derived by a model from the code and the status documents, not stated by
  Pratik or by a source, so each was read back against the tree. The review's own record, with
  what each correction changed, is under Current Position above.

- ~~**Row count discrepancy.**~~ Resolved 2026-08-29. The file has 33 rows and 33 is right.
  The 27 came from the inventory agent's own summary of the document it had just written, and
  was passed into the roadmapper's brief without anyone counting the file. Nothing was dropped:
  all 33 are accounted for in REQUIREMENTS.md. Raising it rather than reconciling to the number
  in the brief is what kept six rows in scope.

- ~~**Phase 5, PIM-04**~~ answered 2026-08-29: not one target. A backend chosen by account
  type behind one seam, the local note a first-class Markdown document, and the seam shaped so
  a hosted service can be added later without a migration. Split into PIM-04, PIM-07, PIM-08.

- ~~**Phase 7, SHIP-04**~~ answered 2026-08-29: the cache is not encrypted. The remaining work
  is saying so where a user meets it, not building anything.

- **Phase 1 grew in discussion, and the roadmap has not caught up.** Its five
  success criteria describe nesting a flat tree. `01-CONTEXT.md` describes one
  branch per account, a shared "On this computer" group whose folders belong to
  no account, a migration that moves existing mail between rows, five settings,
  and three IMAP verbs that do not exist yet (CREATE, RENAME, DELETE mailbox).
  Nothing is outside the phase's domain. Resolved 2026-08-29: the roadmap's
  Phase 1 criteria were rewritten from five to eight to match, and carry a scope
  note pointing at CONTEXT.md as the authority on the detail.

- **FOLDER-01 stays open until 01-09.** 01-01 built create; 01-04 built rename,
  move and delete. The requirement also asks for marking a folder read and
  emptying a folder, which are 01-07 and 01-09. Not ticked here either, for the
  same reason it was un-ticked after 01-01. `requirements mark-complete` ticked
  the whole requirement when 01-01 finished, because a plan names a requirement
  and the tool has no notion of a plan covering part of one. The tick was
  reverted. Any plan here that names a requirement four plans share needs the
  same check before its state update is believed.

- **THREAD-01 and THREAD-02 stay open after 01-02.** Same shape as FOLDER-01
  above, and checked before the state update rather than after. THREAD-01 asks
  for the message list to collapse to one row per conversation, which is 01-12.
  THREAD-02 asks for rethreading as mail arrives without the folder being
  rebuilt, which is 01-13. 01-02 built the thing both of them read, a stored
  conversation id. Neither was ticked.

- **The version bump rule has not been followed for 27 commits, and its check is
  vacuous.** CLAUDE.md says a behaviour change bumps the version in the same
  commit. Cargo.toml last moved 27 commits before 01-02, which includes all of
  01-01's create-a-folder feature. 01-02 bumped 0.45.0 to 0.46.0, so the bump is
  late and covers more than one plan. The nearest check,
  `test_no_status_page_names_a_version_the_code_does_not_ship`, compares versions
  named in README.md and docs/IMPLEMENTATION_STATUS.md against the shipped one,
  and neither file names a version, so it passes over an empty set. Its own
  comment recommends exactly that omission. This is Pratik's call, not a thing to
  fix inside a phase plan.

- **Phase 7, SHIP-01** is blocked on a certificate decision that is Pratik's.
- **Nothing has ever run against a real mail account.** No criterion in this milestone claims
  otherwise, and may be rewritten to.

- 01-05 found two dialogs (wx_destination, wx_thread_view) hanging row data off the control, which wxdragon never frees for a leaf, and a spellcheck test that fails about one full library run in five through a Windows COM call made twice. Both are written up in .planning/phases/01-folders-and-conversations/deferred-items.md.
- 01-06 found allowed_per_account stored, honoured out to the provider clients and offered by no screen: the exact FEEDBACK-01 shape, already in the tree before the guard that found it. Named in STORED_AND_OFFERED_BY_NOTHING in src/data/config.rs and written up in .planning/phases/01-folders-and-conversations/deferred-items.md. Closing it is a per-account group on the settings screen.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Protocol | Gmail X-GM-THRID and X-GM-RAW | v2 | 2026-08-29 | this one |
| Protocol | The Exchange path in the mail-at-scale plan | v2 | 2026-08-29 | this one |
| Protocol | JMAP | v2 | 2026-08-29 | this one |
| Platform | Plugin and extension system | v2 | 2026-08-29 | this one |
| Platform | Set as the Windows default mail client | v2 | 2026-08-29 | this one |
| Validation | Live-account validation of the 13 unproven rows | Out of scope | 2026-08-29 | this one |

## Session Continuity

Last session: 2026-09-12T14:55:00Z
Stopped at: 07-05 merged. A Help menu item asks GitHub whether there is a newer version and the answer is said; one setting on the General tab decides whether it ever asks on its own, starting on not looking. Nothing downloads and nothing runs, which is 07-09's. Ledger 315 to 322, and 313 closed. Owed after the merge: nothing. The ten records reading the outward census were re-measured during the branch and none moved

Earlier: 05.2-02 merged at 93a3f56. The OneNote client exists and nothing calls it; 05.2-03 is next

Earlier: Completed 04.2-05-PLAN.md

Earlier: Completed 04-01-PLAN.md on branch phase-04-01-attachment-descriptions, not merged and not pushed. An attachment says what the sender said it is, or says plainly they said nothing; an image with no header description takes the alt on the img that names it. READ-01 stays open, criterion 4's preview half is 04-03's. Ledger 89 to 93. Owed after the merge: scripts/guards.sh --touched-by 9c4dd39.

Earlier: Completed 02.1-09-PLAN.md on branch gsd/plan-02.1-09, not merged. That is the last plan of phase 2.1, so all nine are executed and five of them are sitting on unmerged branches. 02.1-08 is on gsd/plan-02.1-08, 02.1-07 on gsd/plan-02.1-07, 02.1-06 on gsd/plan-02.1-06, none merged. 02.1-05 is merged into main. 02.1-01 is on gsd/plan-02.1-01, unmerged, with a task 4 decision still awaiting Pratik.

Earlier: Completed 02.1-08-PLAN.md on branch gsd/plan-02.1-08, not merged. Criteria 11 and 12 are closed. The folder tree's twelve kinds of row get six menus instead of two, decided by `folder_tree::which_menu_a_row_offers` and asked for by the tree's closure, and no new command was invented: all five new actions raise ids the menu bar already raises. Four rows get no menu at all, which diverges from a sentence of D-2.1-03 and keeps its reasoning, and follows a precedent already in `tests/wired.rs` for the reminders sidebar. Criterion 12's premise was wrong on both halves and the summary says so rather than building on it: `where_a_row_sits` has a production caller two levels up in another file, so "wire it or remove it" would have deleted live code, and two same-named accounts already read differently because `the_accounts_in_the_tree` composed the address in and `email` is UNIQUE. The property was real and unowned; it moved into the tree, and the cost nobody had filed went with it, so an account branch now reads its address only when another shares its name. One guard record fell behind and was corrected: the new same-name test asks `where_a_row_sits` through the same walk, so that break reddens five rather than four.

Earlier: Completed 02.1-07-PLAN.md on branch gsd/plan-02.1-07, not merged. Criteria 6 and 13 are closed. A one-letter forward marker is read as a forward marker, so a reply to one says it is a reply and a forward of one does not stack a second; both of mail-parser's prefix sets are in a test with the version and the day, and production code holds no marker. The vanished-folders question is read from source with a companion and the window is still not exercised at run time, recorded as ledger 42. Three plan premises came out wrong and are corrected in the summary: the live-window budget is per process rather than spent, seven guard records name tests/wired.rs rather than six, and two counts of markers in this module's comments were off by one. The roadmap's wording of criterion 13 named a threading symptom that does not exist, and it is corrected there too. Found while updating state: `gsd-tools query state.advance-plan` still returns only a parse error and has already written three fields by then, one of them wrongly.

Earlier: Completed 02.1-03-PLAN.md on branch gsd/plan-02.1-03, not merged. Criterion 5 is closed. docs/IMPLEMENTATION_STATUS.md was false through where a paragraph sat rather than through anything it said, so no search for the old sentence could find it; the paragraph moved, and "Anything that writes, against a real account" got its own heading so that "What does not work" means one thing. The tree search found a fifth file the plan did not list, docs/roadmap.md, which still had CREATE, RENAME and DELETE unticked; it is ticked and has its own guard. Five records in .planning/intel/context.md corrected and dated. Two guards and two companions, each companion proved by hand to catch its own reading going blind. The by-hand demonstration found a hole in the first check, which read only the first mention of a capability, and that is a red/green pair of its own. Found and not fixed: docs/roadmap.md:156 says Folder favorites is unbuilt and it ships, recorded as ledger entry 38.
verification found

01-14 built the multi-account folder tree. Three things worth carrying.

**The plan's account of the code was wrong in a way that would not have shown
up.** It said twelve call sites divide into "the data changed" and "the account
being looked at changed", and that the second kind must stop rebuilding. There
are eleven, and all eleven are the first kind: nothing rebuilt the tree in
response to an account switch, because nothing changed the looked-at account in
a way that reached the tree. Carrying out the instruction faithfully would have
meant classifying eleven sites, changing none, and reporting the criterion met.
Count the members of every class a plan names, and treat zero as a finding.

**Making hidden state visible turns every writer of it into a staleness bug.**
Two fell out of task 1 and both are fixed: moving an account wrote the ordinal
and redrew nothing, and selecting a folder read whichever account was open
rather than the folder's own. Both were correct while the tree drew one account
and became wrong the moment it drew them all. The plan enumerates readers; the
new defects were in the writers.

**Eleven source-reading checks in this tree cut a file at the first
`#[cfg(test)]` and keep what is above.** That is "the file up to the first test
module", not "the half that ships", and they agree only while every test module
sits at the end. One was in `wx_app.rs` and now uses `what_ships`. Ten are in
`tests/wired.rs` and cannot, because `what_ships` is `#[cfg(test)]` and an
integration test links the library built without it. Four of those ten fail
loudly when they narrow; six pass in silence over a third of the file. Written
up in the phase's deferred items, with the three possible shapes of a fix.

The cost of the change is stated in the summary and the changelog rather than
buried: every one of the eleven redraws now reads five things per account
instead of five in all, and syncs run on a timer.

Before 01-14, this said:

01-13 closed THREAD-02 and the phase. Four things it found that the plan had
not, and the first two matter to anybody writing over this code.

The plan's own order-independence criterion is unsatisfiable with the signature
the same task mandates: the arrival lookup can see messages the arriving one
names, never messages that name it. So the merge runs in one direction. A late
message connecting two conversations merges them, which is what THREAD-02 asks
for; a conversation root arriving after a message that already named it does
not, and closing that needs an identifier-to-conversation table, which is a
schema decision this plan did not carry.

`messages.message_id` holds two spellings and `messages.thread_id` holds one:
mail through `mail_parser` is stored bare, a draft this program composes keeps
its angle brackets, and the derived column always strips. A new lookup joining
them found nothing, and the symptom read exactly like a wrong test fixture.

The lookup has to ask which conversation an identifier is *in*, not whether it
is the root of one. An ancestor named in a chain is usually in the middle of a
conversation rather than at its head, so asking about roots misses the common
shape.

Guard record "the column that says which conversation a message is in has a
writer" was re-measured for the fourth time in two days, now at 31 tests, one of
them in `wx_app`. It goes stale every time anything near it is touched, which is
the bidirectional check earning its keep.
Research found three things the discussion could not have known, and two of them
needed Pratik's answer: `messages.thread_id` is a column nothing writes and
nothing reads back, so D-08 had no key to span an account with, and the D-19
migration would have hit `UNIQUE(folder_id, uid)` collisions on the user's only
copy of that mail. Both answered and recorded as D-39 and D-40. A third,
the modified UTF-7 encoder, is new scope nobody had costed and is not optional.

01-01 is done and is the phase's tracer, so the shape it proved is what the
other twelve plans lean on. Three things it found that the plan had not: the
tree is read from the cache rather than the server, so a folder must be recorded
and marked as one to keep up to date or nobody sees it; a new mailbox is
unsubscribed and this tree hides unsubscribed folders, so it is subscribed too;
and the IMAP hierarchy delimiter is deliberately dropped after `list_folders`,
so a folder is made where it is named and nesting waits for 01-03 and 01-05.
A fourth is a warning for every later plan here: `MAIL_TRANSPORTS`' imap floor
must rise with every gated write added, because left at 8 with nine present it
had already taken one reddening test off the `copy_message` gate guard.
Resume file: None

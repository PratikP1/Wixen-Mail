---
phase: 07-installing-updating-and-what-is-stored
plan: 01
subsystem: infra
tags: [first-run, command-line, house-style, guards, paths, privacy, smartscreen]

requires:
  - phase: 01-folders-and-conversations
    provides: the first-run screen, its INTRODUCTION constant and the window that reads it
  - phase: 02-searching-and-filing
    provides: tests/house_style.rs, prose_in, sentences_of and the encryption guard this copies
provides:
  - The first-run screen says the downloaded mail is not encrypted on this computer
  - The end of `--help` says the same
  - A guard refusing any shipped page that promises signing stops the Windows warning
  - A companion proving that guard can see the sentence that was really there
  - A check that every path src/common/paths.rs hands out is named on both pages listing what is stored
  - security.key and oauth.toml written onto the pages that were missing them
  - docs/privacy.md says uninstalling cannot clear the Windows Search index
affects: [07-05, 07-07, 07-08, 07-09]

actuals:
  tokens: 10000
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A document guard plus a companion holding the real removed sentence as its fixture"
    - "Enumerating paths by calling accessors, with a second test holding the hand list to the source"

key-files:
  created: []
  modified:
    - src/presentation/first_run.rs
    - src/presentation/command_line.rs
    - src/common/paths.rs
    - tests/house_style.rs
    - guards/guards.toml
    - docs/installing.md
    - docs/privacy.md
    - docs/ALPHA_TESTING.md
    - docs/changelog.md

key-decisions:
  - "The storage sentence goes in INTRODUCTION, not behind a second READ_MORE button, because the screen is shown once per install and what sits behind a button is read by people who were already going to look"
  - "A 900-character bound on the first-run text, holding it at 694, so a fifth thing worth saying has to displace something"
  - "The signing predicate cuts the account sense of the word out of a sentence before looking for the signing words, and leaves the mail sense to be filtered by the conjunction"
  - "The paths check enumerates by calling accessors, with a second test reading the source so an accessor added later cannot sit outside it"
  - "The Windows Search index gets a section on docs/privacy.md; the default-app registry entries do not, because the uninstall removes them and they hold nothing about anybody"
  - "The three %TEMP% writes are not given accessors to make the check see them, because the log fallback exists for when AppPaths cannot answer"

patterns-established:
  - "A guard reading documents carries a comment naming what it cannot see, with the command to re-check the scope"
  - "A floor asserting the reading happened is separated from the dated measurement of what it really read"

requirements-completed: [SHIP-04]

coverage:
  - id: D1
    description: "The first-run screen says the downloaded mail is not encrypted, names Windows keeping another user out, names somebody taking the drive out, and names full-disk encryption as the answer"
    requirement: "SHIP-04"
    verification:
      - kind: unit
        ref: "src/presentation/first_run.rs#test_the_introduction_says_the_downloaded_mail_is_not_encrypted"
        status: pass
    human_judgment: true
    rationale: "The sentence is read aloud in full before the buttons are reachable. Whether it lands as an important fact or as more of the same warning, arriving after a paragraph that already says everything which writes is experimental, is guardrail 5's question and no test here can answer it. WINDOWS.md 302."
  - id: D2
    description: "The end of `--help` says the same, beside the paragraph about writing being unproven"
    requirement: "SHIP-04"
    verification:
      - kind: unit
        ref: "src/presentation/command_line.rs#test_the_help_says_the_downloaded_mail_is_not_encrypted"
        status: pass
    human_judgment: true
    rationale: "Never read in a real terminal or by a screen reader. Its wrapping and its place at the end of a long page are unmeasured. WINDOWS.md 303."
  - id: D3
    description: "No shipped page can promise that signing makes the Windows warning go away, and the guard is proven able to see the sentence that was really there"
    requirement: "SHIP-01"
    verification:
      - kind: integration
        ref: "tests/house_style.rs#test_no_document_promises_the_windows_warning_will_go_away"
        status: pass
      - kind: integration
        ref: "tests/house_style.rs#test_the_signing_check_can_tell_a_promise_from_a_correction"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'a page cannot promise the Windows warning will go away'"
        status: pass
    human_judgment: false
  - id: D4
    description: "Every path src/common/paths.rs hands out is named on both docs/installing.md and docs/privacy.md, and an accessor added later fails the build"
    requirement: "SHIP-04"
    verification:
      - kind: unit
        ref: "src/common/paths.rs#test_every_path_this_module_hands_out_is_named_on_both_pages"
        status: pass
      - kind: unit
        ref: "src/common/paths.rs#test_the_enumeration_reaches_every_accessor_this_module_has"
        status: pass
      - kind: unit
        ref: "src/common/paths.rs#test_a_page_that_could_not_be_read_is_a_failure_rather_than_a_clean_tree"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'every accessor handing out a path is one the pages have to name'"
        status: pass
    human_judgment: false
  - id: D5
    description: "docs/privacy.md says that uninstalling cannot clear the Windows Search index, and stops claiming it removes everything"
    verification: []
    human_judgment: true
    rationale: "Nobody has read the section, and nobody has followed its Indexing Options instructions on a machine that really has indexed mail. The installer says the indexing is experimental and that nobody has seen the Windows indexer read a single message through it."

duration: 73min
completed: 2026-09-12
status: complete
---

# Phase 7 Plan 01: What is left on the disk is said, and signing is not promised Summary

**The first-run screen and `--help` now say the downloaded mail is not encrypted, a failing build stops any shipped page promising again that signing removes the Windows warning, and every path `paths.rs` hands out is on both pages that list what is stored.**

## Performance

- **Duration:** 73 min
- **Started:** 2026-09-12T01:27:28-04:00 (first commit)
- **Completed:** 2026-09-12T02:40:26-04:00 (merge)
- **Tasks:** 3
- **Files modified:** 12

`actuals.tokens` is 10,000, taken as the characters of the added and removed
lines of `git diff df1341b..HEAD` divided by four: 40,139 characters. It excludes
this summary and the planning files committed with it. The command, so the next
reader re-runs it rather than trusting the figure:

    git diff df1341b..HEAD | grep '^[+-]' | wc -c

Against an estimate of 94,000 with `raw_tokens` 78,000, that is a ratio of about
0.13 on raw. The three most recent summaries in the tree give 0.17, 0.15 and 0.20
on the same measure, so this plan came in cheaper than its neighbours and the
estimate was high by roughly the same factor they all were. The figure is not
rounded toward the estimate.

## Accomplishments

- Somebody meeting Wixen Mail for the first time is told, on the screen they cannot get past, that the mail it downloads sits unencrypted on their disk, what Windows protects it from, and what it does not protect them from.
- The same fact is at the end of `--help`, beside the other statements about what has and has not been proven.
- A build fails if a shipped page starts promising again that signing makes the Windows warning go away, with a companion holding the exact sentence commit `32af82b` removed.
- A path added to the data folder and left off either page listing what is stored fails the build.
- `security.key` and `oauth.toml` are on the pages for the first time, and `docs/privacy.md` says for the first time that uninstalling cannot clear the Windows Search index.

## Task Commits

1. **Task 1: The program says the mail on this disk is not encrypted** (RED) `effc7ea` (test)
2. **Task 1: The program says the mail on this disk is not encrypted** (GREEN) `9658282` (feat)
3. **Task 2: No shipped page promises the Windows warning will go away** `4f6b414` (test)
4. **Task 3: Every path the program writes is on the page that lists them** `a4a0ad8` (feat)
5. **Ledger** `4d539ed` (docs)

**Merge:** `228e6a3` on `main`, from branch
`what-is-left-on-the-disk-is-said-and-signing-is-not-promised`.

## What was red, and what was not

Reported apart from the accomplishments, because two of the three tasks had no
red half and saying so is the point.

**Task 1 had a real one.** Three tests, all red against the constants as they
stood, none green against that stub:

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `test_the_introduction_says_the_downloaded_mail_is_not_encrypted` | `INTRODUCTION`, 441 characters, three paragraphs, nothing about storage | assertion 1, the phrase absent | red |
| `test_the_first_run_text_stays_short_enough_to_be_heard` | the same constant | the paired presence assertion, not the ceiling | red |
| `test_the_help_says_the_downloaded_mail_is_not_encrypted` | `HELP` as it stood | assertion 1 | red |

The ceiling half of the second is green against that stub by construction: 441
is under 900. That is the absence-assertion shape, and the fix prescribed for it
is the one taken: a positive assertion paired into the same fixture, so the test
cannot pass against a screen that says nothing at all.

**Task 2 had none and could not have had one.** Its two tests passed on their
first run. The falsehood they guard was corrected by hand on 2026-09-04, five
days before this ran, so there was nothing to make green; the deliverable is a
check rather than any production code; and manufacturing a red by putting the
false sentence back would have written it into a commit that ships. What was
taken red instead is the guard's own break, by hand, and that measurement is the
record. Reporting "two tests, both green on their first run" without this
paragraph would be reporting nothing as a result.

**Task 3 was red on arrival and found more than the plan predicted**, which
`CLAUDE.md` calls a finding rather than a broken test. The failure, quoted:

    these paths are left on somebody's disk and are on no page they can read:
      docs/installing.md does not name security.key, which security_key hands out
      docs/installing.md does not name oauth.toml, which oauth_toml hands out
      docs/privacy.md does not name security.key, which security_key hands out

Its other two tests were green on arrival. `test_the_enumeration_reaches_every_accessor_this_module_has`
is a consistency check between a hand list and the source and is green because
they agree today; its break is what measures it. `test_a_page_that_could_not_be_read_is_a_failure_rather_than_a_clean_tree`
is a companion and is green because the function it drives works.

## The two guard breaks, measured by hand

Both were taken with `--no-fail-fast` across every target, and both red lists are
quoted rather than expected.

**`a page cannot promise the Windows warning will go away`.** The break narrows
the predicate so it cannot see the fixture, which is the shape `CLAUDE.md` says a
document guard really fails in, rather than deleting the guard. One test went
red: `test_the_signing_check_can_tell_a_promise_from_a_correction`.
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` was also
red in that run, from the two tests this plan had just added to the file, not
from the break.

**The finding in that measurement is the one worth keeping.**
`test_no_document_promises_the_windows_warning_will_go_away` stayed **green**
under the break. No page violates it today, so the corpus walk on its own cannot
tell a working predicate from a narrowed one. That is exactly the failure
`CLAUDE.md` describes for a guard whose trigger is "a document mentions X",
measured here rather than argued about, and it is the whole argument for the
companion.

**`every accessor handing out a path is one the pages have to name`.** The break
makes the enumeration skip an accessor, which is the half-fix rather than the
absent one. Exactly one test went red,
`common::paths::tests::test_the_enumeration_reaches_every_accessor_this_module_has`,
and nothing else in 7,022. `test_every_path_this_module_hands_out_is_named_on_both_pages`
cannot see that break at all, because skipping a path means checking fewer rather
than checking wrongly. Run through the tool afterwards as well, which confirms
the TOML quoting really matches:

    scripts/guards.sh "every accessor handing out a path is one the pages have to name"
    -- the one test named went red, and nothing else did

Both records are written with `'''` rather than `"""`, because a TOML basic
string reads a trailing backslash as a line continuation and has silently emptied
a break in this file before. Each break is one self-contained line inside the
unit owning the rule, not a line and its neighbours.

## Guard record re-measurement

**Three, not the four the plan expected.** The count check named three records
after task 1, because "the constant that changes nothing still reads mail"
fingerprints both changed files at once. The failure message and the command:

    3 guard records were measured against a tree that no longer holds those tests:
      the first-run screen focuses the answer it ticks:
          src/presentation/first_run.rs has gained 2 tests: it held 14 and holds 16
      what each first-run answer costs rides on the button:
          src/presentation/first_run.rs has gained 2 tests: it held 14 and holds 16
      the constant that changes nothing still reads mail:
          src/presentation/command_line.rs has gained 1 test: it held 23 and holds 24
          src/presentation/first_run.rs has gained 2 tests: it held 14 and holds 16

    scripts/guards.sh --remeasure "the first-run screen focuses the answer it ticks" "what each first-run answer costs rides on the button" "the constant that changes nothing still reads mail"

All 3 reddened exactly the tests their records name.

**Nineteen after task 2**, the 18 the count check named plus the new record. Run
detached, all 19 reddening exactly what they say. The new record's break was
measured inside that run, so the recorded break is the one that was run rather
than a hypothesis about it.

**None after task 3.** `src/common/paths.rs` appears in no `tests_last_seen`
block and nowhere in `guards/guards.toml` at all, measured with the `awk` over
`tests_last_seen` blocks rather than quoted from the plan, so its three new tests
cost nothing.

`tests/house_style.rs` held 67 `#[test]` before this plan and 69 after, and every
test this plan added to it landed in one commit, `4f6b414`. Nothing was added to
it in task 1, whose count was 67 before and 67 after.

`guards/guards.toml` goes 720 records to 722, with the census at lines 79 and 80
bumped 528 to 530 in the same commits that added the records.

## The first-run text, whole, as somebody hears it

    Wixen Mail is an alpha. Reading your mail is the part that has been used.

    Everything that writes is experimental: sending, moving, deleting, filing a
    copy in Sent, and sending your changes to tasks, contacts and the calendar
    back to your provider. None of that has been run against a real account yet,
    so expect it to have bugs.

    The mail it downloads is not encrypted on this computer. Windows keeps other
    people who use this computer out of the folder, but anyone who takes the
    drive out can read it, unless the disk itself is encrypted. Turn on
    BitLocker if that matters to you.

    Choose what Wixen Mail may change. You can change this later in Settings,
    and the answer covers every account.

The third paragraph is what this plan added. 694 characters in total, 124 words,
against 441 and 78 before.

## The added help text, whole

    The mail this program downloads is not encrypted on this computer. Windows
    keeps other people who use this computer out of the folder, but anyone who
    takes the drive out can read it, unless the disk itself is encrypted. Your
    passwords and sign-in tokens are not in that folder; they are in the Windows
    credential store.

It sits after the paragraph saying everything that writes is experimental,
because both answer the same question: what am I taking on by running this.

## Why it went in INTRODUCTION rather than behind a second button

The constant's own doc comment reads as forbidding it, and both the plan and
SHIP-01's evidence flag that:

> Short on purpose. It is read out in full by a screen reader before the person
> reaches the buttons, so anything not worth hearing every time does not belong
> here; the longer version is the testing page.

"Every time" is once per install, and that is the measurement the caution lacked.
`grep -rn "told_about_the_alpha" src/ --include=*.rs` shows
`src/presentation/wx_app.rs:25395` returning early when
`settings.app_config().told_about_the_alpha` is true and `:25401` setting it as
the screen closes, so the cost of a sentence here is one hearing on a machine's
first start rather than one per launch.

Against that, a second button is one more thing to tab past on the screen
somebody meets first, and what sits behind a button is read by the people who
were already going to look. The person who needs this fact is the one who would
not have pressed it. The doc comment now carries that reasoning, so the next
reader meets it rather than the apparent prohibition.

## The length bound, and why 900

900 characters. The text is 694 now and was 441, so the gap is about one more
paragraph: a fifth thing worth saying has to displace something rather than be
appended. At the bound the text is roughly 150 words, about a minute of speech at
a common default rate; that estimate is why the number is 900 and not 2,000, and
nothing in the test measures speech. It is stated as an estimate in the test's own
comment.

The threat this closes is T-07-04, a first-run screen grown long enough that
somebody stops listening and presses Enter on an answer they did not hear.
Guardrail 5's bounded half is the one that is easy to lose here.

## The signing predicate, and what it can and cannot see

It read **31 sentences** about signing across `docs/**.md` and `README.md`
without the changelog, and refused **none**. Those 31 were listed and read by
hand, and they include all three correct sentences in `docs/installing.md`, the
not-signed line in `docs/ALPHA_TESTING.md`, the tester wording in
`docs/BETA_RELEASE.md`, and eight sentences in `docs/privacy.md` about signed
mail, so the reading really reaches the pages this is about rather than passing
by looking at nothing.

The comment in the test quoted 97 before it was measured. That was a guess, and
measuring it was the whole point of the exercise; it is 31.

**How it tells the three senses apart.** Signing code, a signed mail message and
signing in to an account are all over these pages, and the third is much the
commonest. The account phrases are cut out of the sentence before the signing
words are looked for, because a server that will not let somebody sign in and a
Windows box saying it does not recognise a program are described in the same
words. The mail sense needs no cutting: such a sentence would also have to name
the warning and say it ends. The signing words go through `whole_words_at`,
because "design" holds "sign" and "designed" holds "signed".

**Where it is weakest, and this is worth carrying forward.** The denial test is a
list of the ways these pages really do deny the promise, not a rule about
negation. `docs/installing.md:8` is the live case: "Wixen Mail is not yet code
signed, so Windows does not recognise it" names a certificate and the
recognising, and is allowed only because it contains "does not". Reworded to
"Windows fails to recognise it" it would be refused wrongly. That is written into
the constant's comment.

**What it cannot see at all**, named in its comment with the command: a promise
written into a Rust string or an installer dialog. Neither exists today, verified
rather than assumed:

    grep -rniE "smartscreen|windows protected your pc|run anyway|more info" src/ installer/
    (no output)

## The four signing documents, and what happened to each

| Page | Changed? | Why |
|---|---|---|
| `docs/installing.md` | No | Already correct and it is the reference. The plan says not to rewrite it and there was nothing to rewrite. |
| `docs/BETA_RELEASE.md` | No | Correct. Its tester wording is the one to reuse and reusing it meant leaving it alone. |
| `docs/ALPHA_TESTING.md` | Yes | It said the installer is not signed and Windows will warn, which is true and thin. It now says signing would not stop the warning either and points at `installing.md`'s walkthrough, because the keyboard steps are the part a tester needs and leaving them to be found is the gap. Not a fifth telling of the paragraph. |
| `docs/changelog.md` | Yes, a new entry | It records what the page used to promise, which is what a changelog is for, and it is outside the check for the reason the encryption check leaves it out. |

## Findings

**1. `oauth.toml` was missing from `docs/installing.md` and nobody knew.** The
plan predicted one gap, `security.key` on neither page. The check found three.
The two pages carry the same listing almost word for word and nothing compared
them, which is the thing the check now does for paths. It is also the case that
proves the enumeration design: a grep for `self.root.join(` would have missed
`oauth_toml`, because it joins onto `config_dir`, and passed while missing it.

**2. The three `%TEMP%` writes, which the check cannot reach.** All three predate
this plan and the plan's own must-have was narrowed for them on 2026-09-11.

| where | what it writes | on a page? |
|---|---|---|
| `src/common/logging.rs:79` | `%TEMP%\wixen-mail\logs`, when `AppPaths::resolve()` fails | on neither |
| `src/main.rs:307` | `%TEMP%\wixen-mail-uninstall.log`, every uninstall | yes, `docs/privacy.md`'s Uninstalling names the file |
| `src/presentation/help_page.rs:97` | `%TEMP%\wixen-mail-help`, when the folder holding the documents will not take a file | on neither |

**Is either of the two worth a sentence on `docs/privacy.md`?** The log fallback
is; the help-page copy is not. The help-page copy is converted documentation and
holds nothing of anybody's. The log fallback can hold whatever the running log
holds, which the page already says may contain folder names and email addresses,
on a machine where the data folder could not be resolved, which is the state
somebody debugging a broken install is most likely to be in. Neither sentence is
written here, because 07-05 rewrites that page and owns its shape; it is
`WINDOWS.md` 304. They are deliberately not given accessors to make the check see
them: `logging.rs` falls back there precisely when `AppPaths` cannot answer, so an
accessor would be circular.

**3. The registry entries and the Windows Search index.** The default-app registry
entries under HKCU and HKCR are **not** added to either page. The uninstall removes
them (`main.rs:252-257`), they hold nothing about anybody, and somebody backing up
or wiping a machine does not need them. The Windows Search index content **is**
added, as a section on `docs/privacy.md` rather than a line in the fenced block,
because it is not in the data folder. It is the one that matters: it is content of
somebody's mail on part of the disk that uninstalling does not clear, the
installer's consent dialog at `installer/Wixen-Mail-Setup.iss:349-351` and the
uninstall dialog at `:458-459` both say so, and the page did not. The page also
said "Uninstalling removes everything", which was an overclaim sitting next to
that; it now says it removes everything Wixen Mail stored.

**4. No shipped document claims an EV certificate carries reputation from the
first download.** `docs/installing.md:30` says the opposite, correctly. The claim
survives only at `.planning/ROADMAP.md:555`, which 07-07 task 2 owns. Nothing in
this plan wrote it into a shipped document.

**5. `test_no_document_says_the_cache_is_encrypted` has no `guards/guards.toml`
record.** Found by writing one for the guard next to it. It is the shape this plan
was told to copy, and nothing measures that it still reddens when the check it
holds is narrowed, which is the failure it was itself written to prevent. The
shape is available to it: the signing guard committed here breaks
`tests/house_style.rs` itself with `suite = "house_style"`. `WINDOWS.md` 305.

**6. `docs/privacy.md`'s "Who Wixen Mail talks to" table still has no row for
notes or OneNote**, while `docs/PROVIDER_SETUP.md` says a Microsoft account is
asked for `Notes.ReadWrite`. This plan's task 3 touched `docs/privacy.md` but not
that table, and did not fix it: 07-05 rewrites that page and owns it. Confirmed by
reading rather than assumed.

## Deviations from Plan

### Auto-fixed

**1. [Rule 1 - Bug] The help assertion read the constant through its wrapping**
- **Found during:** Task 1, GREEN
- **Issue:** `test_the_help_says_the_downloaded_mail_is_not_encrypted` was still red after the sentence had been added, because `HELP` is hard-wrapped to a terminal width and the phrase it looked for had a newline in the middle of it. The test was asking two questions at once and reporting the wrong one.
- **Fix:** A `help_unwrapped()` helper in the test module, with a comment saying why. Where a sentence breaks in that constant is a decision about the page rather than about the words.
- **Files modified:** `src/presentation/command_line.rs`
- **Verification:** The test passes, and the four parts are still asserted apart.
- **Committed in:** `9658282`

**2. [Rule 2 - Missing Critical] `oauth.toml` added to `docs/installing.md`**
- **Found during:** Task 3, RED
- **Issue:** The plan said to add `security.key` to both listings. The check found `oauth.toml` missing from `docs/installing.md` as well.
- **Fix:** Both pages now carry an identical listing naming every path, with a short prose paragraph explaining each of the two new names.
- **Files modified:** `docs/installing.md`, `docs/privacy.md`
- **Verification:** `test_every_path_this_module_hands_out_is_named_on_both_pages` passes.
- **Committed in:** `a4a0ad8`

**3. [Rule 2 - Missing Critical] `docs/privacy.md` claimed uninstalling removes everything**
- **Found during:** Task 3, deciding about the Windows Search index
- **Issue:** The page said "Uninstalling removes everything" while the installer's own uninstall dialog says anything already in the Windows Search index stays until the index is rebuilt. A person reads the page to decide whether their mail is gone.
- **Fix:** The sentence now says it removes everything Wixen Mail stored, and a new section says what it cannot take back, with the rebuild instructions.
- **Files modified:** `docs/privacy.md`
- **Verification:** Read against the installer script at `:349-351` and `:458-459`.
- **Committed in:** `a4a0ad8`

**4. [Rule 2 - Missing Critical] SHIP-04's evidence asserted the absence of what this built**
- **Found during:** Closing the requirement
- **Issue:** `.planning/REQUIREMENTS.md` said "It does not ship in the running program" and quoted a grep returning nothing. Both stopped being true at `9658282`. `07-RESEARCH.md` carried the same claim twice. `CLAUDE.md`'s rule is that the commit building a thing must find the sentences saying it does not exist.
- **Fix:** A dated correction above SHIP-04's evidence, leaving the false sentences where they are because they are what the requirement was closed against, and a dated note at each of the two `07-RESEARCH.md` spots so the other eight plans of this phase are not told it is unshipped.
- **Files modified:** `.planning/REQUIREMENTS.md`, `.planning/phases/07-installing-updating-and-what-is-stored/07-RESEARCH.md`
- **Verification:** `grep -rn 'says nothing about the cache\|said nowhere in the product\|does not ship in the running program' .planning/ docs/` now returns only the corrected spots and the plan itself.
- **Committed in:** the metadata commit

---

**Total deviations:** 4 auto-fixed (1 bug, 3 missing critical). No scope creep;
each is a correctness or truthfulness requirement of what the plan asked for.

## Issues Encountered

**`gsd-tools roadmap update-plan-progress 07` wrote a wrong count and was
reverted.** It counted `PLANS-README.md` as a plan, giving 0/10 for a phase with
nine, replaced the status note with a blank cell, and added a checkbox list
including that README. The whole diff was read before reverting and the file was
then edited by hand: 1/9, a note, and a per-plan list of the nine.

**`state advance-plan` was not used.** `STATE.md` was on phase 05.2 and this plan
is the first of phase 07, so advancing the plan counter would have been the wrong
operation. The frontmatter and the body were edited by hand, and
`tests/the_planning_files_agree_with_themselves.rs` caught two real mistakes in
that edit: a quoted `current_phase: "07"`, which it refuses as not a phase
number, and a `Current Plan: 3` line left in the body. Both were fixed and the
block moved into Current Position where it belongs.

## Things the plan said that are still true, and one that is not

Every premise re-checked on 2026-09-11 held when re-run today: `32af82b`'s
sentence is still at `docs/installing.md:27`, the two `house_style.rs` hits for
"sign" are still the two "signed in" fixtures, `ours()` still collects
`installer/*.iss` at line 54, nothing in `src/` or `installer/` mentions
SmartScreen, 720 records with the census at 192 and 528, version 0.112.0, and
`.planning/WINDOWS.md` at 301 with 280 open.

**The corrected gate premise is right and the correction mattered.** Every commit
in this plan answered `affected` on the branch and said so in its own output, not
`all`. The old sentence would have had this report a full gate that never ran.
`scripts/check.sh all` was run once on the branch before the merge, not piped,
and all four checks passed.

**One thing in the plan is wrong against the tree and it is small.** Task 1's
action says committing `first_run.rs` and `command_line.rs` together costs "one
re-measurement of 4 records". It is 3. The two files cost 3 and 1 records, but
"the constant that changes nothing still reads mail" names both, so the union is
3 and not 4. The premise table's per-file counts are right; the arithmetic that
adds them is not.

## User Setup Required

None.

## Next Phase Readiness

- `07-09` now has the RED half its storage obligation lacked. Adding a download accessor to `src/common/paths.rs` reddens `test_every_path_this_module_hands_out_is_named_on_both_pages` on arrival, and the only way to green it is to write the path onto both pages. `test_the_enumeration_reaches_every_accessor_this_module_has` reddens too, and its message names the accessor.
- `07-05` inherits two things: `WINDOWS.md` 304, whether the `%TEMP%` log fallback earns a sentence on the privacy page, and finding 6, the missing notes and OneNote row in that page's "Who Wixen Mail talks to" table.
- `07-08` corrects the three pages that say the build is unsigned, and the guard added here now reads all of them. Its document task should run `cargo test --test house_style` before committing, which the PLANS-README already says.
- `07-07` owns the roadmap's criterion 1, which still says only an EV certificate carries reputation from the first download. Nothing shipped says it.
- SHIP-01 is **not** closed. Only its last clause is, that what SmartScreen does is stated and not promised, and that is now held by a failing build rather than by somebody remembering. The signature itself is blocked on the Azure Artifact Signing account.

---
*Phase: 07-installing-updating-and-what-is-stored*
*Completed: 2026-09-12*

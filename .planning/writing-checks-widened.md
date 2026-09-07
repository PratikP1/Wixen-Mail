# The writing checks reach `.planning`, and a slop check exists

Written 2026-09-07. Branch `writing-checks-reach-planning`, seven commits off
`main` at `e74dc18`.

Two rules that lived in `CLAUDE.md` are now checks. The dash rule reached seven
directories and not the eighth, which is the one holding 197 of this project's
documents. The AI-slop rule reached nothing at all and never had.

## What is now covered

| Rule | Reads | Enforced by |
|---|---|---|
| No em or en dash | `src`, `docs`, **`.planning`**, `tests`, `scripts`, `guards`, `installer`, `.github`, plus `README.md`, `CLAUDE.md`, `Cargo.toml`, `.gitignore`, `build.rs` | `test_no_dashes_that_should_be_punctuation` |
| No carriage return | the same list | `test_no_file_this_project_writes_holds_a_carriage_return` |
| None of the six slop words | the same list, without `CLAUDE.md` | `tests/the_words_that_say_nothing.rs` |
| No promise of a per-account control | the same list **without `.planning`** | `test_nothing_offers_a_setting_per_account_that_no_screen_writes` |
| No false claim about what a new installation changes | the same list without `.planning` | `test_nothing_says_a_new_installation_changes_nothing_while_it_changes_contacts` |

`.planning` is held to the rules about **how prose is written** and not to the
rules about **what the product claims**. That split was measured, not assumed,
and the measurement is in the next section.

The slop check lives in its own integration target and is named in both of
`scripts/check.sh`'s lists, so a documents-only commit and a document committed
beside code both run it. `tests/house_style.rs` gained no test function, so it
stays at 64 and none of the 649 guard records needs re-measuring.

## Decision 1: `.planning` is held to the writing rules only

Widening `ours()` moved eleven checks at once, because eleven read that one list.
Pointing all of them at `.planning` and reading the output gave four failures:

- **The dash rule.** 699 findings. Intended.
- **The carriage-return rule.** 8 files. Real, and not what it looked like. See
  below.
- **The per-account promise rule.** 4 findings, all in `05.1-02-PLAN.md`, which
  quotes the two forbidden phrases in order to tell its executor about the guard
  that forbids them.
- **The new-installation rule.** 8 findings, every one a misreading:
  `codebase/STACK.md` listing `async-imap` with "default features off", read as a
  claim about Allow Changes; `intel/context.md` saying link checking is "off by
  default", which is true and is about something else; `ROADMAP.md` checkbox rows
  whose link text strips to punctuation; and `01-CONTEXT.md` saying
  `Allowed::mail` "is off for a new install", which is exactly what the code
  does, failing because backticks strip and the subject went with them.

Twelve findings, zero real. So the rules that read English for product claims
were written against `docs/` prose and `src/` comments, and a planning tree is a
different genre: dependency tables, roadmap checkboxes, and documents that quote
a guard's trigger phrases while specifying that guard.

`the_pages_that_speak_for_the_product()` is `ours()` minus `.planning`, and
`ours_apart_from_this_file()` is built on it. The exclusion is by genre, in one
place, with the reasoning in the doc comment. It is not a list of the files that
happened to fail, which would go stale the first time somebody wrote a plan and
nothing would say so.

**What it costs, plainly:** a planning document really could promise a control
nothing writes, and nothing would say. That is accepted, because a plan promising
a control is a plan rather than a page anybody believes, and the pages people do
believe are still read. Four assertions in
`test_the_check_is_looking_at_the_whole_project` hold both halves: `.planning` is
in the wide list, absent from the narrow one, and the narrow one still reads the
documents it exists for.

## Decision 2: nothing was grandfathered, and one quotation was preserved

Every dash was rewritten. Two cases came close to the line.

`04-04-PLAN.md` quotes a research table row while arguing that the row is wrong,
so the quoted words are load-bearing evidence. The dashes in it were the
quoter's own column separators, not the source's words. The words are unchanged
and the sentence now names the cells as cells. Nothing was falsified.

`05.1-02-PLAN.md` quotes the two phrases `A_CONTROL_NO_SCREEN_WRITES` forbids.
Those are untouched, and they are the reason `.planning` is out of that rule's
reading rather than the reason to edit the plan.

Two stale facts were left alone deliberately. `codebase/TESTING.md` says a full
guard run is one to two hours where `CLAUDE.md` now says about fifteen, and
`02-RESEARCH.md` says to run guards unfiltered before finishing, which the phase
8 decision of 2026-09-03 reversed. Both are records of what was true when
written. Correcting a record is a different job from correcting its punctuation,
and doing it silently inside a punctuation pass is how a record stops being one.

## What each companion proves

The slop check has three, and each was taken red by hand rather than trusted.

**`test_the_reading_can_see_one_on_a_real_page`** splices each of the six words
in turn into `README.md`'s own lines, in memory, and requires the reading to
answer with exactly that page, that line and that word. It runs over a real file
because being opened, holding text, and being read line by line are the three
links that break, and literals prove none of them.

**`test_the_reading_can_see_one_on_the_page_it_does_not_read`** splices a
violation into `CLAUDE.md`'s own lines and requires the reading to find it. This
is what keeps the one exemption honest: a broken reading and a deliberate
exemption both produce silence, and only this tells them apart. It also asserts
`CLAUDE.md` really is out of the walk, so the exemption cannot quietly lapse
either.

**`test_this_reads_the_same_tree_the_dash_rule_reads`** parses `ours()`'s own
source for every string literal in its body and requires the set to equal this
target's directories, extensions and single files. Two lists describe one corpus,
so one of them will be edited and the other will not; this is the check that says
so on the commit that does it. Its own companion breaks the parse and requires it
to notice.

## What was taken red by hand

| Break | Applied to | Result |
|---|---|---|
| `ours()` collects `.planning` | `tests/house_style.rs` | 4 checks red, 12 of the 699+12 findings classified by hand |
| The slop reading always answers `None` | `a_word_beginning_with` | **The check itself stayed GREEN** and all three companions went red |
| `.planning` dropped from the slop corpus | `THE_TREE` | drift check red, naming `.planning` as the missing directory |
| `CLAUDE.md` added to the slop walk | `THE_SINGLE_FILES` | exemption check red, and all six words reported on line 688 |
| A test name renamed out of `check.sh`'s list | the gate script, in memory | both list readings red, as designed |

The second row is the one worth keeping. Blinding the reading leaves the guard
passing and only the companions fail. That is guardrail 4 demonstrated rather
than asserted, on a check written this morning.

## The dashes: 699 rewritten, none left

699 em dashes across 55 files. No en dashes existed anywhere in `.planning`.
Nothing was left, including inside a fenced code block, so the count of dashes
left for quotation, URL, code block or command reasons is **zero**.

The fenced case is worth stating because it was the one the brief expected to
need an exemption. Three dashes sat inside an ASCII diagram in `02-RESEARCH.md`,
where a replacement of a different width would have broken the column alignment.
Each was a single character doing a single character's work, so each became a
single character: two colons and a comma. The diagram is byte-aligned as before.

Where the dashes went:

| Form | Roughly | When it was right |
|---|---|---|
| Colon | 300 | A citation or a name followed by what it is. Reference lists, headings pairing a code with a verdict, `path: what it holds`. |
| Comma | 200 | A short aside, or a parenthetical that was already bracketed by two dashes. |
| Full stop | 130 | Two independent clauses. The commonest genuine rewrite. |
| Parentheses | 30 | A pair of dashes around an aside inside a sentence that already had commas. |
| Restructured | 25 | The sentence only worked because the dash was carrying a clause it could not otherwise hold. |
| A word | 16 | A table cell using a dash to mean "no value". |
| A list marker | 12 | A dash standing in for a bullet. |

Counts are by judgement rather than parsed, and they sum to about 700 rather than
exactly 699, so read them as shape rather than as measurement.

Three of those rows are not punctuation choices at all.

**A dash meaning "no value" in a table** was 16 cells across six documents. A
screen reader announces that as punctuation or as nothing, so the cell said less
to this project's primary audience than to anybody else. Each now says what it
means, chosen per column: `none`, `nothing`, `not applicable`, `did not exist`,
and in one case `covered by the row above`, which was a fact the dash had been
hiding rather than a value it stood in for.

**A dash standing in for a list marker** was 12 "Reversibility" notes. Each was
an indented dash where a bullet belonged, then `**Reversibility:** costly`, then
a second dash introducing `three places must agree`. Two dashes, two jobs,
neither of them punctuation in the first instance. They are now nested list items
with a full stop, which is the structure they always described and which a screen
reader can move through. One of them sat mid-line after "deliberately." and is
now on its own line.

**A dash in an H1** was three document titles, each putting the phase number and
the document's subject on one side and its kind on the other. Phase 2's is now
`# Phase 2 Verification Report: Search that says what it covers`, which puts the
kind of document before its subject and leaves one colon doing one job.

## The slop: eleven uses, and two greps that could not see three of them

`CLAUDE.md` has named six words since it was written. Nothing had ever read for
them.

**This page cannot spell them, and that is the design working rather than a
limitation to route around.** `CLAUDE.md` is the one file exempt from this rule,
because it is the file that states it. This page is not exempt, and giving it an
exemption would start the allow list the whole design avoids. So each word is
named here by the advice `THE_WORDS` carries for it, which is unique per entry
and is the more useful half anyway. Read `tests/the_words_that_say_nothing.rs`
for the words themselves.

| The word whose advice is | Uses found | Where |
|---|---|---|
| "full, complete, thorough" | 8 | `wx_managers.rs` x2, `accessibility-framework-evaluation.md`, `accessibility.md`, `architecture.md` x2, `roadmap.md`, `TROUBLESHOOTING.md` |
| "solid, reliable, strong" | 2 | `.planning/ROADMAP.md`, `04-04-PLAN.md` |
| "use" | 1 | `accessibility-framework-evaluation.md` |
| the other three | 0 | nowhere in the tree |

**Three of the eleven were invisible to grep, and the same way twice.** The task
brief reported seven and said `.planning` held none of the six. The first grep
written to check that reported the same seven. Both matched a list of endings,
`word(s|d|ing|ed|ment)?`, and neither list held `-ness` or `-ly`. Both
`.planning` uses take the `-ness` form and one `docs/` use takes `-ly`. The check
matches a stem for exactly this reason, and the doc comment on `THE_WORDS` says
so where somebody would go to simplify it back to whole words.

The rewrites are plain, and readable here without naming what they replaced. A
heading is now "Full semantic labeling". A bullet is now "Uses the existing
codebase". `TROUBLESHOOTING.md`'s first line lost its adjective entirely, because
"Solutions for common issues with Wixen Mail" was already the claim. The two
`-ness` uses
both meant "this ordering protects the accessible half from being blocked by the
decoding work", which is what they now say.

**How the check stays out of its own way, by design.** The six words are built
with `concat!`, so no whole one appears in the file's text on disk. That is the
dash rule's code-point trick applied to English, and it means the check needs no
exemption for itself. The companions splice their violations into real pages in
memory, so no fixture exists on disk either. `CLAUDE.md` is the one file not
read, because a document stating a rule cannot be held to it, and that is the
same shape `ours_apart_from_this_file` and `test_sources` already solved twice.
The whole file is exempt rather than the sentence, because finding "the sentence
that lists the words" needs a list of wordings, and `tests/house_style.rs`
records at length what happens to those.

I got this wrong on the first run: the negative and positive fixtures in
`test_the_reading_knows_a_word_from_a_word_that_merely_contains_it` were written
as plain string literals, so the check reported seven findings against its own
source. They are now built from the entries.

## The false comment in `which-checks.sh`

It said the dash guard "caught two real breaks on 2026-08-31, one in `CLAUDE.md`
and one in a planning file". It could not have. `ours()` did not collect
`.planning` until this branch, so no planning file had ever been opened by the
guard being credited. Both breaks were under paths it really read.

The correction says why it was worth making rather than just fixing the noun.
The claim was load-bearing: it justified running the document-reading targets on
a documents-only commit, which is a rule that is right. A citation supporting a
conclusion everybody already agrees with is the one nobody re-checks, and this
one read as evidence for a week. It would be true if written today, which is
exactly why it had to be corrected rather than left to become right.

## Things found on the way that somebody should know

**The carriage returns were not in the repository.** The widened check reported
eight `.planning` files holding CRLF. `git status` was clean and `git show
HEAD:<file>` returned no CR for any of them: `core.autocrlf` is `input`, so the
repository had always held LF and the CRLF existed only in the working tree,
written there by some tool that runs on Windows. Deleting and checking the files
back out fixed all eight and produced an empty diff. **The check went green with
nothing to commit.** Had the "fix" been to rewrite the content, the commit would
have been a no-op dressed as a repair and the real finding, that something writes
CRLF into `.planning` and will do it again, would have been recorded nowhere.

**Adding a fourth target to `check.sh` broke a companion in
`the_planning_files_agree_with_themselves`.** It took its own name out of the
whole-tree list by replacing `" NAME)"` with `")"`, which only worked while that
name was last. Appending `the_words_that_say_nothing` made the replacement match
nothing, so the reading went on finding the name and the assertion failed for a
reason unrelated to the script. It now takes the name out wherever it sits. The
general shape: **a companion whose break depends on where a name sits in a list
expires the next time the list grows**, silently, and presents as a failure in
the thing being read rather than in the break.

**A shell pattern that loses its predicate stops filtering rather than failing.**
`grep -rlU $'\r' --include='*.md' .planning` listed 8 files run directly and
about 190 inside `for f in $(...)`, because the quoting did not survive into that
context and the pattern became empty. `grep` exited zero and the loop deleted and
restored 190 files. It was harmless only because restoring an unmodified file
from the index is a no-op, which `git status` then proved. The rule this project
already knows about empty results has a second direction: nothing matched is
suspicious, and everything matched is worse, because it looks like work to do.

**Six of the seven "table cell" dashes in `01-RESEARCH.md` were in a Fallback
column and one was in a Version column**, and they needed different words. A
single replacement for all of them would have said `none` where the honest answer
was `not applicable`. There is no generic word for an empty cell, which is part
of why the dash was there.

**This page failed both new checks on its first draft**, and both failures were
the same shape: a document about a rule reproducing what the rule forbids in
order to show what was fixed. Two dashes were quotations of before-states and six
slop words were the list of banned words. Neither is an argument for an
exemption, because a document that can quote its way past a rule is a document
nobody can hold to it. Both were rewritten to describe rather than reproduce,
which is what the rest of this branch asked of 55 other files. Worth knowing
before writing the next document about these checks: describing the before-state
costs a sentence and reproducing it costs an exemption.

**197 files, 55 of them affected.** The brief's "663 across 197 files" paired a
match count with a corpus count. The two directory figures it gave that I could
reproduce exactly, 61 and 14, are evidence the sub-counts were measured and no
evidence at all about the join. Worth remembering when quoting scope: `grep -c`
counts lines, `grep -o | wc -l` counts occurrences, `grep -l | wc -l` counts
files, and all three get written as one number.

## What is not held

- **A planning document can still promise a control nothing writes**, by design.
  Decision 1 above.
- **Slop anywhere in `CLAUDE.md` is unread.** The whole file is exempt, not the
  sentence.
- **A URL or a code identifier containing one of the six words would be
  reported.** None exists today, checked. The check has no exemption for either,
  and adding one should wait for a real case rather than an imagined one.
- **`the_words_that_say_nothing` reads the same corpus as the dash rule, in its
  own list.** The drift between the two lists is checked; the extensions are
  compared as a set rather than per directory, so moving an extension from one
  directory to another inside `ours()` would not be noticed.
- **Two stale facts in planning documents**, named under Decision 2, left as
  records.

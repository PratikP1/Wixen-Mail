---
phase: 09-what-the-first-day-of-testing-found
plan: 02
subsystem: spellcheck, settings, message cache, search index
tags: [spelling-language, settings-screen, snippet, search-index, ammonia, once-only-pass, guards]

requires:
  - phase: 09-what-the-first-day-of-testing-found
    provides: "09-01: the tree at 1.0.0-alpha.1 and the rule that a fix under the alpha gets a changelog entry and no bump"
provides:
  - "spellcheck::language_to_use, one resolver for a stored spelling tag, asked by the checker and by the settings screen"
  - "A settings screen whose language row selects the resolver's answer and shows a stored tag nothing offers as itself, never row 0"
  - "tests/the_language_the_screen_shows_is_the_one_used.rs, a target building the real General tab, coupled to wx_settings.rs by a guard record's suite"
  - "long_text::words_of_markup, the words of a provider's markup through the same reader the message goes through, title dropped with style and script"
  - "A snippet and a search index row for an HTML-only body derived through that reader; strip_markup gone"
  - "MessageCache::put_right_the_snippets_read_from_stylesheets, run once from open, reindexing each row it changes, recorded in the new work_done_once table"
affects: [09-03 onward, which write changelog entries under Unreleased at 1.0.0-alpha.1; any later once-only pass over stored data, which has a table to record itself in]

actuals:
  tokens: 22500
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "One resolver for a stored value two readers show or act on, asked before any lenient platform call, so the screen and the checker cannot disagree by accident of list order"
    - "A once-only pass over stored data records itself in work_done_once (name, done_at) after it finished, so a failed pass is tried again and a finished one reads nothing on later opens"
    - "A test target for a screen with no unit tests of its own is coupled to the source file by a guard record whose suite names the target, and check.sh --suites-for is the proof"

key-files:
  created:
    - tests/the_language_the_screen_shows_is_the_one_used.rs
  modified:
    - src/service/spellcheck/mod.rs
    - src/service/spellcheck/windows_speller.rs
    - src/presentation/wx_settings.rs
    - src/application/long_text.rs
    - src/data/message_cache/bodies.rs
    - src/data/message_cache/searching.rs
    - src/data/message_cache/mod.rs
    - guards/guards.toml
    - docs/changelog.md

key-decisions:
  - "The checker asks the resolver before asking Windows about the tag as stored, because Windows accepts a bare en outright and would otherwise answer the question the resolver was written to own"
  - "A stored tag nothing offers becomes a row of its own at the end of the picker, marked as having no dictionary, and read_settings maps the selection back through the same rows, so pressing OK keeps it"
  - "The snippet takes a new reader, words_of_markup, rather than first_line, because first_line answers the first piece only and a snippet wants up to 200 characters across pieces"
  - "The marker is a row in a new work_done_once table on the held_alerts shape, because no key-value table existed and sync_state and tree_state are keyed by account and folder"
  - "The target that builds the settings screen goes in neither of check.sh's lists, because it reads no documents; the guard record's suite is what couples it"

patterns-established:
  - "A premise that a platform refuses an input is settled by one call, not by the code that assumed it: the plan's reading of for_language was wrong on this machine and a scratch test found it in seconds"

requirements-completed: []

coverage:
  - id: D1
    description: "A bare or unlisted stored language resolves to this machine's own region when the machine's language is in the same family, in one function both the checker and the settings screen ask, and a family with no such member still takes Windows' first"
    requirement: FOUND-02
    verification:
      - kind: unit
        ref: "src/service/spellcheck/mod.rs#test_a_bare_stored_language_resolves_to_the_region_this_machine_is_set_to"
        status: pass
      - kind: unit
        ref: "src/service/spellcheck/mod.rs#test_a_bare_stored_language_takes_the_first_offered_when_the_machine_speaks_another"
        status: pass
      - kind: other
        ref: "a scratch test with --nocapture: SYSTEM_LANGUAGE=Some(\"en-US\") FOR_LANGUAGE(en) -> language=\"en-US\" source=\"Windows, so it knows the words you have added in Windows Settings\""
        status: pass
    human_judgment: false
  - id: D2
    description: "The settings screen shows the language that will be used, never index 0 for a value it could not match, and a test builds the real screen with en stored and reads the selection back"
    requirement: FOUND-02
    verification:
      - kind: integration
        ref: "tests/the_language_the_screen_shows_is_the_one_used.rs#test_the_language_the_screen_shows_is_the_one_the_checker_uses"
        status: pass
    human_judgment: false
  - id: D3
    description: "A snippet derived from an HTML-only body, and the body text the search index holds for it, are derived through the same reader the reading path uses, so style, script and head content never reach either, and strip_markup is gone rather than patched"
    requirement: FOUND-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/bodies.rs#test_a_snippet_of_an_html_only_body_is_its_words_and_not_its_stylesheet"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/searching.rs#test_an_html_only_body_is_indexed_by_its_words_and_not_by_its_stylesheet"
        status: pass
      - kind: unit
        ref: "src/application/long_text.rs#test_the_words_of_markup_are_the_words_alone_without_markers_or_stylesheets"
        status: pass
      - kind: other
        ref: "grep -rn strip_markup src --include='*.rs' | wc -l -> 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "Snippets already stored for HTML-only bodies are re-derived once, on the first open after the change, through the same function, non-fatally and once only, with the count logged, and each row put right is reindexed"
    requirement: FOUND-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/bodies.rs#test_stored_snippets_read_from_stylesheets_are_put_right_once_index_included"
        status: pass
      - kind: unit
        ref: "src/data/message_cache/bodies.rs#test_snippets_are_put_right_once_and_a_second_call_reads_no_body"
        status: pass
      - kind: other
        ref: "target/debug/wixen-mail.exe --read-only against a temp profile holding one such row: INFO wixen_mail::data::message_cache: Put right the snippets of 1 HTML-only messages through the reader, index rows included, in 3 ms"
        status: pass
    human_judgment: false
  - id: D5
    description: "Whether the tester's own profile shows English (United States) and reads snippets as words"
    requirement: FOUND-02
    verification: []
    human_judgment: true
    rationale: "His profile holds a hand-set en-US the resolver leaves as stored, so his machine cannot show the language fix, and his 12,872-message cache has not been opened by this build; ledger 484 and 485"

duration: 2h 57m
completed: 2026-09-16
status: complete
---

# Phase 9 Plan 02: Two stored values read right Summary

**A stored bare spelling language now resolves to the region this machine is set to, through one
resolver the checker and the settings screen both ask, so a profile from before 2026-09-03 stops
landing on English (Caribbean) and Settings shows the language that is used (#21). A snippet of an
HTML-only message, and the text the search index holds for it, come through the reader the message
goes through, the crude stripper is gone, and the snippets already stored are put right once on the
first open, index rows included (#32). Both closed on the tree; the tester's profile is his to
confirm.**

## Performance

- **Duration:** about 2h 57m, of which about 1h 35m was the three guard re-measure runs
- **Started:** 2026-09-16T14:38:02Z
- **Completed:** 2026-09-16T17:35Z
- **Tasks:** 3
- **Files modified:** 10 (one created)

## What landed

**One resolver.** `spellcheck::language_to_use(stored, this_machine, offered)` answers the stored
tag itself when it is offered and available, else the machine's own region when it shares the
stored tag's family and is offered and available, else the first offered member of the family,
else `None`; case is ignored on both sides. Seven tests, one per case, including a machine whose
own region is offered but unavailable, which falls to the first. `for_language(tag)` asks it
first and opens what it answers; only when it answers nothing is Windows asked about the tag as
stored, and only then does the built-in list follow. `find_regional_variant` is retired and its
two tests are rewritten to the resolver: the one that pinned "the first of the family Windows
lists" now says in its name that the machine's own region wins. The family arm that
`best_available_match` and the resolver share moved into `first_available_in_the_family_of`.
`mod.rs` held 56 `#[test]` lines before and holds 63; `cargo test --lib service::spellcheck::`
ran 65 before and runs 72, the difference being `windows_speller.rs`'s 9 under the same path.

**The screen.** `wx_settings::language_rows_and_selection(stored)` builds the picker's rows and
the selection in one place: the resolver's answer when it has one, else the stored tag's own row
marked "(no dictionary installed)" as the list already marks any language without a dictionary,
else a row added at the end named after the stored tag. `read_settings` maps the selection back
through the same rows, so a stored tag nothing offers is kept by OK rather than replaced by row
0's tag. `grep -c 'unwrap_or(0)' src/presentation/wx_settings.rs`: 10 before, 9 after. The
plan offered two shapes for the `None` case and this is the second, a row of its own, chosen
because the built-in fallback list covers only the six languages it holds and a Windows list
holding no member of the stored tag's family covers nothing.

**The target.** `tests/the_language_the_screen_shows_is_the_one_used.rs` builds the real
General tab three times inside one `wxdragon::main`, each dialog destroyed before the next, and
reads the tag of the selected row back through `read_settings`, which is what OK writes. With
`"en"` stored on this machine it read back `en-029` before the fix and `en-US` after; with
`"en-AU"` it reads `en-AU` either way; with `"zz-ZZ"` it read `en-029` before and `zz-ZZ` after.
The expected value for `"en"` is computed through the resolver against what the machine offers,
so on a runner with no spell checking, where the built-in list has `"en"` at row 0, the test holds
the same claim and passes. `bash scripts/check.sh --suites-for guards/guards.toml
src/presentation/wx_settings.rs` printed `every_event_has_a_control` and `checkbox_labels`
before the record and prints `the_language_the_screen_shows_is_the_one_used` beside them after
it. The target reads no documents, so `scripts/check.sh` is unchanged.

**The reader.** `long_text::words_of_markup(html)` is `from_markup` followed by `structure`, with
every piece given as its words and joined by a space; the extraction that `the_same_words` had
inline is now `words_of` and both use it. `first_line` was read first, as the plan asked, and
does not answer: it gives the first piece only, and a snippet wants up to 200 characters across
pieces. The sanitising call in `read_markup` now drops a `<title>`'s content beside a
`<style>`'s and a `<script>`'s, through `ammonia::Builder::default().add_clean_content_tags(["title"])`,
because `ammonia`'s defaults strip a disallowed tag and keep its content, and a whole message
carries a title in its head where a note never does. `long_text.rs` held 59 tests and holds 60.

**The snippet and the index.** `bodies::snippet_of(body)` is the one rule, the plain part when it
has words in it, else the words of the markup through the reader, bounded by `SNIPPET_LIMIT` as
before; `save_message_body` and the pass both call it. `strip_markup` is deleted: `grep -rn
'strip_markup' src --include='*.rs'` finds nothing. `index_message_for_search` takes an HTML-only
body through the same reader; its plain-part rule is unchanged. For the newsletter fixture, a
`<title>`, a `<style>` and a `<script>` in the head and "Hello from the newsletter" in the body,
the snippet is "Hello from the newsletter", a search for `padding` finds nothing and a search for
`newsletter` finds the message. `bodies.rs` held 43 tests and holds 47; `searching.rs` 25 and 26.

**The pass.** `MessageCache::put_right_the_snippets_read_from_stylesheets` asks `work_done_once`
first and answers 0 without reading a body when its row is there; otherwise it reads the
candidates into a `Vec` (no packed plain part, a blank or absent plain part, an HTML part),
reads each body through `get_message_body`, derives the snippet through `snippet_of`, rewrites
the row only where the snippet differs, reindexes that row through `index_message_for_search`,
counts it, and records itself after the loop, so a pass that fails halfway is tried again on the
next open. A test plants the old stylesheet snippet by SQL, asserts the index finds `padding`
first so the fixture reproduces the defect, runs the pass, and reads back the words, the
untouched snippet of a message with a plain part, `padding` no longer found and `newsletter`
found; a second test runs it, plants a second wrong row, runs it again and requires 0 and the
second row unchanged, which is how "read no body" is told from "found nothing". The fixtures
delete the marker first, because `MessageCache::new` runs the pass on every fresh database and
records it, so a fresh test cache already carries the row and only a database from before the
pass existed lacks it, which is what everybody upgrading has.

**The table.** `work_done_once (name TEXT PRIMARY KEY, done_at TEXT NOT NULL)`, created with
`CREATE TABLE IF NOT EXISTS` on the `held_alerts` shape, nothing dropped or renamed. The plan
said to read the schema for a key-value table first: there is none. `sync_state` is keyed by
account and sync type and `tree_state` by folder identity, so a new table it is, and a second
once-only pass is a second row in it.

**The open-time call.** In `MessageCache::new`, straight after `migrate_inline_bodies`, in the
manner of the backfills around it: `Ok(0)` says nothing, `Ok(n)` logs at info with the count and
the elapsed time, `Err` warns and the open goes on. Run for real: a debug build of `a3554483`'s
parent, against a temp profile pinned with `WIXEN_MAIL_DATA` and holding one HTML-only message
with the old snippet planted and no marker, started with `--read-only`, `RUST_LOG` unset, and
killed after 25 s:

```
2026-09-16T17:02:15.641485Z  INFO wixen_mail::data::message_cache: Put right the snippets of 1 HTML-only messages through the reader, index rows included, in 3 ms
```

Afterwards the row's snippet was "Hello from the newsletter", `work_done_once` held the pass's
row stamped `2026-09-16T17:02:15`, the index answered nothing for `padding` and the row for
`newsletter`. The first attempt at this run produced an empty log because `RUST_LOG` was set to
an empty string, which `EnvFilter` reads as a filter that admits nothing; the harness in
`tests/the_numbers_the_targets_ask_for.rs` unsets it, and so did the second attempt. The tester's
profile was not touched: bash and PowerShell see a stale July copy of it from here, and nothing
in this plan read or wrote either copy.

## Task commits

| Commit | What |
|---|---|
| `413dd783` | test(09-02): the red half of task 1, naming three resolver tests and the count check |
| `520470d4` | feat(09-02): the resolver, `for_language` asking it first, `find_regional_variant` retired, two records, 31 re-measured |
| `ab85dfb5` | test(09-02): the red half of task 2, naming `test_the_language_the_screen_shows_is_the_one_the_checker_uses` |
| `10ee7fb4` | feat(09-02): the screen through the resolver, its record, the #21 changelog entry |
| `45ec0a23` | test(09-02): the red half of task 3, naming five tests and the count check |
| `98de23c7` | feat(09-02): the reader, the snippet, the index, the pass, the table, three records, 27 re-measured, the #32 changelog entry |
| `a3554483` | Merge 09-02 into `main` |

Branch `two-stored-values-read-right` from `main` at `ea3c20d1`. Not pushed.

## Honest RED and GREEN

Task 1's red commit names three tests that were red and says the other four were green on
arrival, because they describe the parts of the rule today's code already had (the exact stored
tag, a machine set to another language, a language nothing offers, and case); the resolver's
signature was committed carrying today's rule so the tests compiled. Task 2's red commit names
the target's one test, red on this machine with the tester's exact symptom, `"en"` shown as
`en-029`. Task 3's red commit names five tests and says the heading-and-list case was green on
arrival, because the crude stripper already gave words without markers; it stays to hold the new
reader to that. The reader was committed as `from_markup`, markers and all, and the pass
answering 0, so the tests compiled. Each red commit also names
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, which fires when a file a
record names gains a test, and each green commit carries its remedy.

## What the gate selected

| File | On the branch |
|---|---|
| `src/service/spellcheck/mod.rs` | `--lib service::spellcheck::` (red and green commits) |
| `src/service/spellcheck/windows_speller.rs` | `--lib service::spellcheck::windows_speller::` (green commit) |
| `tests/the_language_the_screen_shows_is_the_one_used.rs` | `--test the_language_the_screen_shows_is_the_one_used` (red commit) |
| `src/presentation/wx_settings.rs` | `--lib presentation::wx_settings::`, which matches nothing, plus the three coupled targets `every_event_has_a_control`, `checkbox_labels` and `the_language_the_screen_shows_is_the_one_used`, the last coupled by this plan's record |
| `src/application/long_text.rs` | `--lib application::long_text::` (both task 3 commits) |
| `src/data/message_cache/bodies.rs`, `searching.rs` | `--lib data::message_cache::bodies::` and `--lib data::message_cache::searching::` (both task 3 commits) |
| `src/data/message_cache/mod.rs` | `--lib data::message_cache::` (green commit) |
| `guards/guards.toml`, `docs/changelog.md` | no scoped target; both rode code commits, so the whole-tree guards ran and no `docs_only` run happened |

Every commit also ran the whole-tree guards. `scripts/check.sh all` on the branch at `98de23c7`,
run once, its output to a file and its exit status read directly, never piped: exit 0, 7,803
passed and none failed over 61 result lines, 352 s, the release build included. Fourteen more
than the 7,789 on 09-01's whole gate: seven in `spellcheck/mod.rs`, one target, four in
`bodies.rs`, one in `searching.rs`, one in `long_text.rs`. The keyring race (ledger 374) did not
appear. `main`'s hook ran `all` again on the merge: 7,803 and none failed.

## Guard records

829 records by the TOML reader before, 834 after; census 802 + 27 before, 802 + 32 after, the
line at `guards/guards.toml:84` moved in each commit that added records. Five new records, one
rewritten, all measured on the whole library at the default eight threads, `WIXEN_TEST_THREADS`
untouched, except the screen record, measured on its own target as records with a `suite` are.

| Record | Break | Red | Measured |
|---|---|---|---|
| a bare stored language resolves to the region this machine is set to, not to the first Windows lists | the middle arm returns the first of the family after finding the machine's region | 4: the en-US and en-GB cases, the case-insensitive case, and the rewritten bare-language test | all 4 named, nothing else |
| a bare stored language on a machine set to another falls to the first of its family, not to nothing (rewritten from the retired function's record) | the last arm returns `None` | 2: the fr-FR machine case and the unavailable-own-region case | all 2 named, nothing else |
| the settings screen shows a stored language nothing offers as itself, not as row 0 | `.or_else(\|\| row_of(stored))` becomes `.or(Some(0))` | the target's one test | `cargo test --test the_language_the_screen_shows_is_the_one_used`, the one test |
| a snippet of an HTML-only body is its words through the reader, not the markup as it is | the snippet takes the HTML as it is | 5: three snippet tests, the pass test, and the index test | first draft named 4; the runner named the index test, because the index row carries the snippet column; corrected and measured again |
| the index takes an HTML-only body as its words through the reader, not as the markup | the index arm takes the HTML as it is | 2: the index test and the pass test | all 2 named, nothing else |
| the snippet pass records itself as done, not under a name nothing asks about | the marker is written under `"never"` | the second-call test | the one named, nothing else |

**The count check's remedy, run and read, three times.** After task 1 it named 30 records naming
`spellcheck/mod.rs`; all 31 (the 30 and the new one) were re-measured in one run of 46 minutes,
86 s a record by the runner's own `timed:` lines, and 28 agreed. Three came out short because the
resolver's new tests reach the family arm and `choices_from`, which their breaks are about: "a
family language match has to be available" gained 2 tests, "a family language match has to share
the family" gained 4, and "a real Windows language list is used instead of the built-in fallback"
gained 1, each corrected from what the runner reported and measured again, all three agreeing.
After task 2 the screen record was measured alone. After task 3 it named 24 records over
`bodies.rs`, `searching.rs` and `long_text.rs`; the 27 (24 and three new) took 38 minutes, 84 s a
record, and 25 agreed. Two did not: the snippet record above, and "a task or event body written
as html is read as the structure it carries", whose `before` is the whole body of `read_markup`
and no longer applied once that body changed; its `before` was rewritten to the body as it
stands, and measured it reddens six tests it never named, the new words test and the five snippet
and index tests, because those now read through it. Corrected, measured again, 57 named and
nothing else. Counts now written: `spellcheck/mod.rs` 63, `wx_settings.rs` 0 with the target's
1, `bodies.rs` 47, `searching.rs` 26, `long_text.rs` 60.

Nothing in this plan added a test to `tests/house_style.rs` or `tests/wired.rs`.

## Premises the tree contradicted

1. **Windows accepts a bare `"en"`.** Premise 1 said `for_language("en")` reaches
   `find_regional_variant` because Windows refuses the bare tag, and the issue's evidence and
   the requirement's evidence say the same. The first `--nocapture` run after the resolver was
   wired in the plan's order (Windows first, resolver second) printed
   `FOR_LANGUAGE(en) -> language="en"`: `IsSupported("en")` is true on this machine, so the old
   code never reached the fallback here and checked whatever Windows takes a neutral `"en"` to
   mean, while the screen showed the Caribbean. The order was turned round, resolver first, and
   the second run printed `language="en-US"`. The screen half of the premise, `unwrap_or(0)`
   landing on `en-029`, held exactly, and it is the half the tester saw. Logged as observation
   606 in the skill observation log.
2. **`first_line` is not the reader.** Premise 3 allowed that it might be; it answers the first
   piece only, so a new reader was written, as the plan also allowed.
3. **A `<title>` survives `ammonia::clean`.** Not a premise the plan stated, but the behaviour
   the plan asked for, a title in the head not reaching the snippet, needed the sanitiser told
   to drop it; `ammonia`'s default keeps a disallowed tag's content.
4. **The language row lives in `add_language_and_spelling`**, a helper `build_general_tab`
   calls, not in `build_general_tab` itself; the line numbers were right within a few lines.
5. **A fresh database carries the marker.** The plan's test description, "a second call returns
   0 because a marker says it has run", is true of every fresh cache on its first call too,
   because `MessageCache::new` is the caller; the fixtures delete the marker to stand in for a
   database from before the pass, on the pattern of the index test that drops its column.

## Deviations from plan

**1. [Decision] The checker asks the resolver before Windows, not after.** Premise 1 above. The
plan's order would have left a bare `"en"` checked as Windows' neutral English while the screen
showed en-US, which is the disagreement the resolver exists to end.

**2. [Decision] A stored tag nothing offers becomes a row of its own.** The plan offered this
or the built-in fallback list's own unavailable rows; the fallback list holds six languages and a
Windows list holding no member of the family holds none, so the row is added at the end and
`read_settings` maps through the same rows.

**3. [Rule 1 - Bug] `RUST_LOG=` emptied the log** on the first acceptance run; fixed by unsetting
it, as the existing harness does. No code changed.

Everything else executed as written. No scripted edit touched a tracked file: exception set zero,
and it stayed there; the two scratch tests were added and removed with the Edit tool, since
`git checkout` on a file holding uncommitted work would have taken the work with it. Carriage
returns measured with `tr -cd '\r' | wc -c` on the changelog, the new target and the ledger:
zero on each. No em-dash in any file this plan wrote. `Cargo.toml` untouched; no package added.

## Threat register

T-09-05: the snippet and the index row come through `long_text`'s reader, which runs `ammonia`
first, now with `<title>` among the elements whose content is dropped; `strip_markup` is deleted
and the grep finding nothing is quoted above. T-09-06: `work_done_once` makes the pass run once;
the second-call test holds it to 0 without reading a body; the open is non-fatal and the marker is
written only after the pass finished. T-09-07: the candidate predicate excludes a packed plain
part and a plain part with words in it, and the pass test plants a plain-part message with an odd
snippet and requires it untouched. T-09-08: OK writes the resolved tag as it always has; Cancel
leaves the file alone; the changelog's Known limitations line says so. T-09-SC: no package added.

## Known stubs

None. `language_to_use` is reached by `for_language` and by the settings screen;
`words_of_markup` by the snippet and the index; the pass by `MessageCache::new`, shown by the
log line above from a running binary. The log line says "1 HTML-only messages" for a count of
one; it is a log line, not something read aloud, and is left as it is.

## Not done here, on purpose

Whether the tester's own profile now shows English (United States) and reads snippets as words
is his: his profile holds a hand-set `en-US` the resolver leaves as stored, so his machine
cannot show the language fix, and his cache has not been opened by this build. Ledger 484 and
485, both halves, 483 before and 485 after. No `FOUND` requirement is ticked, on the phase's
rule that the last plan reads each clause by clause. Nothing pushed; 33 commits unpushed before
the commit that lands this summary, by `git rev-list --count origin/main..main`.

## Self-Check: PASSED

Files: `tests/the_language_the_screen_shows_is_the_one_used.rs` exists; `grep -rn strip_markup src`
finds nothing; `guards/guards.toml` holds 834 records by the TOML reader. Commits `413dd783`,
`520470d4`, `ab85dfb5`, `10ee7fb4`, `45ec0a23`, `98de23c7` and `a3554483` are in `git log
--oneline` on `main`.

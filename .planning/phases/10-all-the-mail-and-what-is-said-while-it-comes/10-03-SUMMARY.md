---
phase: 10-all-the-mail-and-what-is-said-while-it-comes
plan: 03
subsystem: settings, the body cache, the sync workers, guards
tags: [settings, message-text, eviction, body-cache, permissions-tab, choice, guards, changelog]

requires:
  - phase: 10-all-the-mail-and-what-is-said-while-it-comes
    provides: "10-01: bringing_everything_down::TextBudget, All or UpTo(u64), and TextStillMissing::kept_bytes as what a budget is measured against; 10-01.1: each later Settings page painted inside APage::built and focus handed to its first control, so a control added to Permissions lands on a page whose check boxes are check boxes"
provides:
  - "application::keeping_message_text: TextKept, All or UpTo(bytes); ALL offering All of it, Up to 1 GB, Up to 5 GB, Up to 20 GB, a gigabyte being 1,000,000,000 bytes; label, as_stored, from_stored answering All for anything unreadable and for nought, Default All, budget() answering the TextBudget the eviction and the runner read; offered_index falling back to All; KEEP_LABEL and WHAT_A_SIZE_DOES"
  - "AppConfig.message_text_kept: String, serde default all, an older settings file answering the same"
  - "wx_settings: a labelled choice under Message Text on the Permissions tab after the note, its sentence under it, read back in read_the_permissions_page; PermissionsTabControls::message_text_kept public as sort_order is"
  - "MessageCache::keeping_bodies_under(TextBudget); body_budget a TextBudget, UpTo(BODY_CACHE_BUDGET_BYTES) for a cache opened with none; keep_bodies_within_budget answering Ok(0) under All without reading a row"
  - "wx_app::how_much_message_text_stays, read once by each of the two workers that evict before they open their cache, and the two caches opened through the seam with it"
  - "tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs: three dialog readings and one source reading, coupled to wx_settings.rs and wx_app.rs by measured records"
  - "guards/guards.toml: four records measured, eleven re-measured; 892 records, census 802 + 90"
  - "docs/changelog.md: the entry under Unreleased, Added; ledger 518 and 519"
affects: [10-05 (the runner reads TextKept::from_stored(..).budget() for its text pass and opens its cache through the typed seam), 10-07 (reads MAIL-03's second [D] line from here and writes the privacy page's sentence about the cache growing under All)]

actuals:
  tokens: 16364
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "A string-valued setting with one enum owning label, stored form, parse and default, on the shape of reading_habits::MarkRead, and a fallback that loses nothing rather than one that bounds"
    - "A cache seam typed by the application's own value rather than a bare number, so All is a value the eviction can match on and not a magic size"
    - "A reading target that builds the real dialog, changes a control and reads the settings back the way OK does, beside a source reading of the two workers that hand a cache the setting"

key-files:
  created:
    - src/application/keeping_message_text.rs
    - tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs
  modified:
    - src/application/mod.rs
    - src/application/bringing_everything_down.rs
    - src/application/mail_sync.rs
    - src/data/config.rs
    - src/data/message_cache/bodies.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/attachment_content.rs
    - src/presentation/wx_settings.rs
    - src/presentation/wx_app.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md

key-decisions:
  - "The record on the screen's read-back is measured on a new target that builds the real dialog, chooses each size and reads it back, because neither target the plan offered reads the Permissions page and a break there reddened nothing in either"
  - "The record on the check's worker being handed the setting is measured now on a source reading in that target rather than deferred to 10-05's target, because the reading is the same reading and earlier"
  - "The two worker sites read the setting through one helper, how_much_message_text_stays, rather than each repeating six lines; the helper is where the read-by-something guard sees the field's name"
  - "from_stored answers All for nought as well as for anything unreadable, because a bound of nought bytes is every body gone and nobody means it"
  - "The attachment budget's comment stops saying the two halves keep the cache around a gigabyte, with the date, because the sentence became false the moment the body half became a setting"

patterns-established:
  - "A red commit that has to change a seam's type to compile lands the typed seam with a stub arm that answers wrongly, says so in the arm's comment, and the green replaces the arm; the sites that had to move for the compile are green on arrival and said so"

requirements-completed: [MAIL-03]

coverage:
  - id: D1
    description: "How much message text stays on this computer is a choice under Message Text on the Permissions tab, All of it by default and three sizes, stored as a string with a default an older settings file falls back to"
    requirement: MAIL-03
    verification:
      - kind: unit
        ref: "src/application/keeping_message_text.rs#test_the_choices_are_all_of_it_then_one_five_and_twenty_gigabytes"
        status: pass
      - kind: unit
        ref: "src/application/keeping_message_text.rs#test_the_default_keeps_all_of_it"
        status: pass
      - kind: unit
        ref: "src/application/keeping_message_text.rs#test_a_garbled_value_reads_as_all_because_a_wrong_bound_evicts_and_all_loses_nothing"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_a_settings_file_written_before_these_existed_reads_the_way_it_should"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: integration
        ref: "tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs#test_a_size_chosen_on_the_permissions_page_is_what_ok_writes_back"
        status: pass
      - kind: integration
        ref: "tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs#test_the_dialog_offers_all_of_it_first_and_the_default_writes_all_back"
        status: pass
    human_judgment: false
  - id: D2
    description: "The eviction reads the setting and evicts nothing under All; every cache the sync workers open is handed it"
    requirement: MAIL-03
    verification:
      - kind: unit
        ref: "src/data/message_cache/bodies.rs#test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_read_by_something"
        status: pass
      - kind: integration
        ref: "tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs#test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'with All chosen the end of a folder sync evicts no message text' and 'the check's worker opens its cache with how much message text stays', measured 2026-09-17, each reddening the one test named and nothing else"
        status: pass
    human_judgment: false
  - id: S1
    description: "Whether the choice is heard as its name, a combo box and its answer, whether the sentence under it is read once, whether OK keeps it across a restart, and whether text stays on the tester's machine where 0.125.1 dropped it"
    requirement: MAIL-03
    verification: []
    human_judgment: true
    rationale: "Ledger 519; the listening lines belong to 10-07's page. Nothing here met a real account, and the setting is a choice and the eviction a cache rule, both proved on a temporary cache"

duration: 1h34m
completed: 2026-09-17
status: complete
---

# Phase 10 Plan 03: How much message text stays on this computer is a setting Summary

**How much message text stays on this computer is a choice on the Permissions tab under Message
Text, beside the box that forbids fetching it: All of it, Up to 1 GB, Up to 5 GB, Up to 20 GB,
All of it by default and what a settings file from before this existed answers. The body cache's
eviction at the end of every folder sync reads it through a typed seam and evicts nothing under
All, where it dropped the least recently read text above 512 MiB before, so the download of
everything (10-05) will not be undone by the next folder's sync on a mailbox larger than that.
The two workers that evict, the check and the whole-folder request, read the setting once before
opening their cache and hand it in; the window's cache never evicts and is not handed it. Half of
#23; the download itself is 10-05.** Nothing pushed.

## Performance

- **Duration:** 1 h 34 min from the branch at 20:39:45Z to the merge at 22:13:24Z, plus the
  reading before the branch; of that, about 23 minutes were guard measurement across four runs
  (1,403 s summed from the runners' `timed:` lines), about 21 minutes the five hook runs
  (261 s, 314 s, 315 s, 193 s, 187 s), and about 14 minutes the two whole gates (438 s and 432 s)
- **Started:** 2026-09-17T20:39:45Z (the branch; the first commit 20:50Z)
- **Merged:** 2026-09-17T22:13:24Z at `c4203632`
- **Tasks:** 2
- **Files modified:** 13 (2 created)

## What landed

**Task 1, the setting.** `src/application/keeping_message_text.rs`. `TextKept` is `All` or
`UpTo(u64)`; `ALL` offers `All`, `UpTo(GIGABYTE)`, `UpTo(5 * GIGABYTE)` and `UpTo(20 * GIGABYTE)`
in that order, `GIGABYTE` being 1,000,000,000 so the label and the stored number agree; `label`
says "All of it", "Up to 1 GB", "Up to 5 GB", "Up to 20 GB"; `as_stored` writes "all" or the
bytes; `from_stored` reads either and answers `All` for anything it cannot read and for nought,
held against `""`, `"   "`, `"lots"`, `"-5"`, `"0"`, `"5 GB"`, `"1e9"` and `"all of it"`, because a
garbled value read as a small bound throws away everybody's downloaded text (T-10-09) and `All`
loses nothing; `Default` is `All`; `budget()` answers `TextBudget::All` or `TextBudget::UpTo(n)`,
with the reason in its doc comment that one setting feeds the eviction and the runner;
`offered_index` selects the stored choice and falls back to `All` for a size the list does not
offer, so somebody who opens Settings with a hand-edited bound sees All and saving that back keeps
their text. `KEEP_LABEL` is "&Keep the text of messages on this computer:", Alt+K, which no other
control on that page claims (`test_no_two_controls_in_one_dialog_claim_the_same_alt_key` green);
`WHAT_A_SIZE_DOES` is the plan's sentence: what leaves, when, what stays. `cargo test --lib
application::keeping_message_text::` passes with 10 (11 in task 1, one deleted with the interim
helper in task 2).

`AppConfig.message_text_kept: String` at `config.rs:290-299`, `#[serde(default =
"default_message_text_kept")]` answering `TextKept::default().as_stored()`. The older-file test
gained the field in its list and two assertions: the parsed value is `"all"`, with the sentence
that an absent key answering anything else would evict the text of everybody upgrading, and
`from_stored` of it is `All`. `cargo test --lib data::config::` passes with 68, as before.

The Permissions tab's Message Text section gained a `labelled_choice` after the note, label
`KEEP_LABEL`, spoken name the same without the mnemonic and the colon, the four labels from
`TextKept::ALL`, selected through `offered_index`, and a `StaticText` under it carrying
`WHAT_A_SIZE_DOES` on both channels (`wx_settings.rs:2212-2237`). `read_the_permissions_page`
writes `cfg.message_text_kept` through `TextKept::ALL` and the selection (`:3307-3311`), the
`reading: w.` line as it was (`grep -c 'reading: w\.'` 1 before and after). The field is `pub` on
`PermissionsTabControls`, as `ReadingTabControls::sort_order` is, so the reading can choose a size.
The page is built lazily; a page nobody showed leaves the stored value alone, and the reading
holds it.

`wx_app::how_much_message_text_stays()` at `wx_app.rs:21535-21551` reads
`ConfigManager::load_stored()` once and answers `TextKept`, `All` for a file that cannot be read;
`spawn_whole_folder_fetch` (`:21609-21611`) and `spawn_mail_sync` (`:21742-21744`) call it before
opening their cache and open it with `.keeping_bodies_under(text_kept.budget())`. In task 1 the
seam still took an `i64`, so the sites handed `budget().as_bytes_or(BODY_CACHE_BUDGET_BYTES)`, a
helper on `TextBudget` with its own test; task 2 typed the seam and deleted both (`grep -rn
'as_bytes_or' src tests` finds nothing).

**The settings guards, quoted from the runs.** Between the field and the control, at the red
commit: `test_every_setting_somebody_can_change_is_offered_by_a_screen` "1 setting(s) are stored
and survive a restart and no screen offers any of them" and
`test_every_setting_somebody_can_change_is_read_by_something` "1 setting(s) can be changed and are
read by nothing", 65 passed and 3 failed. Between the control and the reader sites: the offered
guard green and the read-by-something guard alone red, 67 passed and 1 failed. After the two
sites: 68 passed.

**Task 1, the reading.** `tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs`,
`cfg(windows)`, one `wxdragon::main` in a `OnceLock<Result<Harvest, String>>` on 10-01.1's shape.
It builds the real dialog over a stored `"5000000000"`, reads the settings back before the
Permissions tab is shown, shows it (`set_selection(3)`), records the selection and the four
entries, chooses entries 0, 1, 3 and 2 in turn and reads the settings back after each, destroys
it, builds a second over the defaults, shows the tab and reads the selection and the settings
back. `test_a_size_chosen_on_the_permissions_page_is_what_ok_writes_back` holds each choice to
`TextKept::ALL[i].as_stored()`; `test_the_stored_size_is_selected_when_the_page_is_shown_and_left_alone_when_it_is_not`
holds the never-shown read to `"5000000000"` and the selection to 2;
`test_the_dialog_offers_all_of_it_first_and_the_default_writes_all_back` holds the four labels in
order, the default selection to 0 and the untouched write-back to `"all"`. All three green on the
fixed tree; the record's break took the first red (below).

**Task 2, the eviction.** `MessageCache::keeping_bodies_under` takes `TextBudget`
(`mod.rs:1529-1544`); `body_budget` is a `TextBudget`, `UpTo(bodies::BODY_CACHE_BUDGET_BYTES)` for a
cache opened with none (`:154-157`, `:1379-1381`); `keep_bodies_within_budget` answers `Ok(0)`
under `All` without reading a row and under `UpTo(n)` calls `evict_bodies_over` at `n`
(`bodies.rs:470-487`). `BODY_CACHE_BUDGET_BYTES` is `u64` now and stays 512 MiB; its doc comment
says it is not what runs for anybody since 2026-09-17, that every cache the sync workers open is
handed the setting, that the window's caches never ask for an eviction, and quotes the old
sentence dated: "a number rather than a setting, because ... nobody has asked for one. The tester
asked." The module doc says the same in one clause. The one test in `mail_sync.rs` that passed
`60` passes `TextBudget::UpTo(60)`.
`test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size` opens a cache
under `All`, stores three bodies of ten bytes, asks `keep_bodies_within_budget` and reads all three
back with `0` freed; under `UpTo(15)` it frees 20 and one body is left. `cargo test --lib
data::message_cache::bodies::` passes with 50 where 10-01 left 49; `data::message_cache::` 744;
`application::mail_sync::` 147, as before.

**Trace of the absence, measured.** `grep -rn 'keep_bodies_within_budget()' src` finds one
shipping caller, `mail_sync.rs:1473` in `sync_folder`, and the two calls in the new test.
`grep -rn 'sync_folder(' src` outside `fn sync_folder` finds `wx_app.rs:21641` (the whole-folder
request's closure) and `:21903` (`spawn_mail_sync`) and three test callers in `mail_sync.rs`. So
the two workers are the only caches that evict, and the window's cache never does. The fourth
reading in the target, `test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts`,
holds that: one call of the eviction in `mail_sync.rs`'s shipping half, two calls of
`sync_folder` in the window's, and each worker's body (through `what_ships` and `body_of`)
containing the sync call, `how_much_message_text_stays()`, `.keeping_bodies_under(` and
`text_kept.budget()`.

**The changelog.** One entry under `[Unreleased]`, Added, beginning "**How much message text
stays on this computer is your choice, and the default is all of it.**": #23 and the build, what
the eviction did before, the four answers and where they sit, that the default is what the tester
asked for and an older file answers the same, what a size does and that the mail itself never
leaves, that a change applies from the next check; Known limitations: the text is not downloaded
on its own yet, that is the next plan, the stored mail is not encrypted as `docs/privacy.md` says,
and under All of it the cache grows with the mailbox. The version stays `1.0.0-alpha.1`.

## Task commits

| Commit | What |
|---|---|
| `8271b327` | test(10-03): the red half of task 1, fourteen named by module path, none green on arrival |
| `adcb9fa7` | feat(10-03): the module, the field, the choice, the page reader, the two worker sites through the interim helper, the reading target, two records measured |
| `f008a84e` | test(10-03): the red half of task 2, the eviction test and the count check named; the typed seam landed with a stub All arm so the test compiles, the helper deleted, the fourth reading green on arrival and said |
| `e42be7ae` | feat(10-03): the All arm, the constant's and the module's comments, the attachment comment corrected, two records measured, eleven re-measured, the changelog entry |
| `24974d12` | docs(10-03): ledger 518 and 519 |
| `c4203632` | Merge 10-03 into `main` |

Branch `how-much-message-text-stays` from `main` at `e33dab5a`. Not pushed; 142 commits unpushed
before the commit that lands this summary, by `git rev-list origin/main..HEAD --count` on `main`
at `c4203632`.

## Honest RED and GREEN

Task 1's red named fourteen: the module's eleven tests by module path, the older-file test as
cargo reports it (`data::config::permission_tests::...`, not the `tests::` the plan wrote), and
the two settings guards. The gate in `red` mode ran exactly those fourteen red and nothing else,
in 261 s. None was green on arrival: the stubs answered `All` four times for `ALL`, empty strings
for the labels and the stored forms, `UpTo(0)` for `from_stored`, `Default` and `budget`, and 0 for
`offered_index` and the interim helper. The green ran the same tests and 22 coupled targets, the
nine on `wx_settings.rs` and the thirteen on `wx_app.rs`, and carried no marker, in 314 s.

Task 2's red named two: the bodies test and the count check bare. The stub `All` arm evicts as a
bound of nought, written out and said in the arm's comment, so the test is red against something
rather than against a compile error. The typed seam, the moved sites, the deleted helper and the
fourth reading landed in the same commit because the test could not compile without the seam and
the seam could not compile with the sites as they were; the fourth reading was therefore green on
arrival and the commit says so, and its red half is the record below. The gate in `red` mode ran
exactly those two red, in 315 s. The green ran `attachment_content`, `bodies` and the whole-tree
guards in 193 s, no marker, no remedy printed: the eleven records the count check had named were
re-measured before the commit.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/keeping_message_text.rs` | `--lib application::keeping_message_text::` on all four task commits |
| `src/application/mod.rs` | `--lib application` whole, on task 1's red |
| `src/application/bringing_everything_down.rs` | `--lib application::bringing_everything_down::` on three commits |
| `src/data/config.rs` | `--lib data::config::` on task 1's red |
| `src/presentation/wx_settings.rs` | `--lib presentation::wx_settings` (matching nothing) and the nine coupled targets on task 1's green |
| `src/presentation/wx_app.rs` | `--lib presentation::wx_app` and the thirteen coupled targets on task 1's green and task 2's red |
| `src/data/message_cache/bodies.rs`, `mod.rs` | `--lib data::message_cache::bodies::` and `--lib data::message_cache` on task 2's red; `bodies` and `attachment_content` on its green |
| `src/application/mail_sync.rs` | `--lib application::mail_sync::` on task 2's red |
| the new target | its own target on both task 1 commits and task 2's red; the coupled target on every commit after its records existed |
| `guards/guards.toml`, `docs/changelog.md`, `.planning/WINDOWS.md` | no scoped target; the whole-tree guards on every commit |

Every commit ran formatting, clippy, the four script suites and the whole-tree guards.
`scripts/check.sh all` on the branch at `24974d12`, output to a file with the exit status appended,
never piped: exit 0, 7,960 passed and none failed over 72 result lines, 438 s from 21:58:01Z to
22:05:19Z, the release build included; its last block says five of CI's seven jobs. Fifteen more
than 10-02.2's 7,945: 10 in the module, 1 in `bodies`, 4 in the target. Inside the 275 s to 654 s
band on `docs/development/measurements.md`. `main`'s hook ran `all` again on the merge: 7,960 and
none failed, 432 s from 22:06:12Z to 22:13:24Z. The keyring race (ledger 374) did not appear on
either. **The tab row reading 10-02.1's summary recorded failing intermittently passed every time
it ran here**: on task 1's green, task 2's red, and both whole gates.

## Guard records

888 records by the TOML reader before, 892 after; census 802 + 86 before, 802 + 90 after, the
line at `guards/guards.toml:84` moved in each green commit. Four new, every one measured through
`scripts/guards.sh --remeasure` with `WIXEN_TEST_THREADS` untouched and the counts written by the
runner.

| Record | File | Break | Red | Run |
|---|---|---|---|---|
| a garbled size for the message text kept reads as All and not as a bound | `keeping_message_text.rs` | `unwrap_or_default()` to `unwrap_or(TextKept::UpTo(0))` | 1, "the one test named went red, and nothing else did" | rebuild 35 s, run 49 s; again 34 s and 49 s after the module lost a test |
| the size chosen on the Permissions page is what OK writes back | `wx_settings.rs`, `suite` the new target | the write in `read_the_permissions_page` replaced by a read of the control | 1 | rebuild 16 s, run 2 s; again 16 s and 2 s after the target gained a test |
| with All chosen the end of a folder sync evicts no message text | `bodies.rs` | `All => Ok(0)` to `All => self.evict_bodies_over(0)` | 1 | rebuild 34 s, run 48 s |
| the check's worker opens its cache with how much message text stays | `wx_app.rs`, `suite` the new target | `spawn_mail_sync`'s cache opened bare, the `before` carrying the `fail(...)` line that only that worker has, because the site line alone appears twice | 1 | rebuild 15 s, run 2 s |

The `All` record's break is a bound of nought rather than the old constant the plan named,
because a test that could tell 512 MiB from All would have to store more than 512 MiB of bodies;
what the record proves is that the All arm is consulted, and the record's comment says so.

**The count check's remedy, run and read.** After task 2's red it named eleven: the nine records
naming `bodies.rs` (49 to 50), the module's (11 to 10) and the target's (3 to 4). All eleven were
re-measured in the foreground in three batches under the tool's ten-minute limit, 517 s, 510 s and
207 s with the two new records in the third, and every one agreed as written: 57, 8, 5, 2, 2, 5,
2, 1, 1 tests red for the nine on `bodies.rs`, in the order the check listed them, and one each for
the module's and the target's. Counts written: `bodies.rs` 50 on ten records, `config.rs` 68 on
eleven, `mail_sync.rs` 147 on twelve, `wx_settings.rs` 0 on eighteen, `wx_app.rs` 199 on sixty,
`keeping_message_text.rs` 10 on one, the target 4 on two. `bash scripts/check.sh --suites-for
guards/guards.toml src/presentation/wx_settings.rs` answers nine targets, `wx_app.rs` fourteen. No
file gained or lost a test after its records were measured; the count check is green on `main` at
`c4203632`.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `e33dab5a` before anything
was built. Four things moved or were wrong:

1. **Premise 4's count.** `grep -n 'MessageCache::new(dir, None)' src/presentation/wx_app.rs | wc
   -l` answers 19, not 12, at `e33dab5a` and at `7d57cd49` alike, so the plan miscounted rather
   than the tree moving. The two sites it meant were at `:21588` and `:21718` at `e33dab5a`, not
   `:21626` and `:21756`; the shape held.
2. **The older-file test's module path.** The plan's trailer named
   `data::config::tests::test_a_settings_file_...`; cargo reports it under `permission_tests`,
   because `config.rs` holds several test modules. The trailer carries what cargo printed.
   Observation 654 in the skill log.
3. **The screen record's target.** The plan offered `every_event_has_a_control` or
   `the_settings_dialog_opens_in`, "measured rather than predicted". Both were read before
   measuring: the first checks the Feedback page's event controls, the second reads the Reading
   page back and never shows Permissions. A break in `read_the_permissions_page` reddens nothing
   in either, so the record could not exist on them, and a new target was written (deviation 1).
   Observation 655.
4. **`bodies.rs` had 9 records naming it, not 8.** 10-01's own record on the size column was the
   ninth; the plan quoted the count from before 10-01 landed.

The other premises held: `BODY_CACHE_BUDGET_BYTES` at `bodies.rs:289`, `keeping_bodies_under` at
`mod.rs:1528` with its one test caller at `mail_sync.rs:3427`, `keep_bodies_within_budget` at
`:1473`, `READING_SECTION` at `allowed.rs:206` and `MESSAGE_TEXT_LABEL` at `:224`,
`read_the_permissions_page` at `wx_settings.rs:3269` with `reading: w.` once, `mark_read_after`'s
seven sites, the two guards at `config.rs:2753` and `:2376`, `files_that_act` at `:2350`, the
POP read of `look_at_message_contents` at `wx_app.rs:19808`, 68 tests and 11 records on
`config.rs`.

## Deviations from plan

**1. [Decision] The screen record is measured on a new target.** Above and ledger 518. The
target is `tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs`, on the
phase README's rule that new readings go in new targets at zero records with a record whose
`suite` couples them.

**2. [Decision] The worker record is measured now, not deferred.** The plan said that if no
existing target reddened when the check's worker was opened bare, the finding was to write a
reading into 10-05's target. None did, for the reason the plan gave (the read-by-something guard
sees the field's name in the helper and cannot tell the cache line from it); the reading is
written into this plan's target instead, because it is the same reading and it holds the wiring
from the day it landed. 10-05 still owes the runner's own cache line the same reading when it
opens one.

**3. [Decision] One helper for the two sites.** `how_much_message_text_stays()` in `wx_app.rs`,
called by both workers, rather than six lines twice. It answers `TextKept`, so task 2's move to
the typed seam was one token per site. The plan's "the way the POP path reads
`look_at_message_contents`" is kept: read once, before the cache, `unwrap_or_default` for a file
that cannot be read.

**4. [Rule 2] `from_stored` answers `All` for nought.** The plan said "anything it cannot read";
`"0"` can be read and is a bound of nought bytes, which is every body gone at the next sync. It
is in the garbled list with the reason.

**5. [Rule 1] The attachment budget's comment corrected.** `attachment_content.rs:56-61` said the
two halves kept the whole cache around a gigabyte; with the body half a setting whose default
keeps everything that sentence is false, and it now says what it said until 2026-09-17 and what
is true since.

**6. [Decision] The `All` record's break is a bound of nought**, above, with the reason on the
record.

Everything else executed as written. **No scripted edit touched a tracked file: the exception set
for this plan is zero, and it stayed there.** Every tracked file was changed by Read then Edit or
Write; the only `sed`, `awk` and `grep` in the session read files and logs, and `git checkout` was
not needed. Commit messages were written to the scratchpad and passed with `-F`; `cargo fmt` ran
before each commit. Carriage returns measured with `tr -cd '\r' | wc -c` on every changed file
before each commit: zero on each. No em-dash in any file this plan wrote, measured with `grep -c`
for the byte sequence: zero on each. `git commit` and `git merge`, never `gsd-tools query commit`;
never `--no-verify`; `check.sh` never piped, its exit status appended to its own log. No AI
attribution in any commit. `Cargo.toml` and `Cargo.lock` untouched; no crate added (T-10-SC). The
version stays `1.0.0-alpha.1`. The tester's profile was not read; no binary was started; nothing
was sent to any window this plan did not build.

## Threat register

T-10-08 accepted and said: the changelog entry and `WHAT_A_SIZE_DOES` say the cache grows under
All and a size is one choice away; 10-05's runner says how much is still to come. T-10-09
mitigated: `from_stored` answers `All` for eight garbled values and for nought, held by a test and
a measured record. T-10-10 accepted: the changelog points at `docs/privacy.md`, which 10-07
writes the sentence into. T-10-SC: nothing added. No new surface outside the register: the
setting is read from the settings file the other settings are read from, and the reading target
builds only windows it destroys.

## Ledger

`.planning/WINDOWS.md` 518 and 519 written by hand, both halves, no backslash;
`the_planning_files_agree_with_themselves` green after, 16 passed. 517 before, 519 after; 489
open before, 491 after.

| id | kind | what |
|---|---|---|
| 518 | deviation | the four departures above with their reasons, so 10-05 reads the seam from the tree |
| 519 | unrun-verify | what only the tester's ear settles for the choice, and that the eviction honouring a size against his account is settled only by a mailbox with more text than the size |

## The issue

`gh issue comment 23` from the repository root after the merge, at 22:13:40Z, the plan's sentence
quoting `c4203632`. #23 stays open; 10-05 closes it. Commenting is not a publish; no other issue
was filed or edited.

## Known stubs

None. The choice is built by `build_permissions_tab` on every Settings open that reaches the
Permissions tab and read back by `read_the_permissions_page` when OK is pressed; the field is
read by `how_much_message_text_stays`, which both workers that evict call before opening their
cache; the eviction reads the cache's budget at the end of every folder sync. `TextKept::budget`
has the two worker sites as callers today and 10-05's runner as the third.

## What the tester's account could settle, and a temporary cache proved

Nothing here is claimed against a real account. A temporary cache proved that `All` evicts
nothing and `UpTo(15)` evicts down to one body; a real dialog proved that each size chosen is the
size written back and that a page nobody showed leaves the stored size alone; values proved the
labels, the stored forms, the fallback and the budget. What the tester hears when he reaches the
choice, and whether his text stays where `0.125.1` dropped it, is ledger 519.

## Not done here, on purpose

The download of everything's text is 10-05, and the changelog's Known limitations say so. No
`MAIL` requirement is ticked, on the phase's rule that 10-07 reads each clause; the clauses this
plan gives a named test and a named control for are MAIL-03's second `[D]` line: the choice
`PermissionsTabControls::message_text_kept` under Message Text, built at `wx_settings.rs:2218`
with `KEEP_LABEL`; `test_the_choices_are_all_of_it_then_one_five_and_twenty_gigabytes`,
`test_the_default_keeps_all_of_it` and `test_a_settings_file_written_before_these_existed_reads_the_way_it_should`
for the default and the older file; `test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size`
for the eviction; `test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts`
for the workers; and `test_every_setting_somebody_can_change_is_offered_by_a_screen` as what
failed on arrival. Roadmap criterion 3's "kept unless a size is chosen" clause closes
structurally and is read by 10-07.

## For 10-05

- The setting is `TextKept::from_stored(&stored.app_config().message_text_kept)`; the window's
  helper `how_much_message_text_stays()` in `wx_app.rs` answers it and is `fn`, not `pub`, so the
  runner in the same file calls it.
- `TextKept::budget()` is the `TextBudget` `what_to_do_next` takes; hand the same value to the
  runner's cache through `MessageCache::new(dir, None)?.keeping_bodies_under(kept.budget())`, or
  the runner's own eviction will run at 512 MiB whatever was chosen. The fourth reading in this
  plan's target holds the two existing workers to that and counts `sync_folder`'s callers in the
  window as two; a runner that calls `sync_folder` is a third, and the reading's count moves with
  it: rewrite the count and add the runner's body to the loop rather than adding a test, as
  ledger 518 says.
- `TextStillMissing::kept_bytes` is what the budget is measured against; under `All` the runner
  never ends a text run on bytes, and the eviction never undoes it.
- The changelog's Known limitations line for this entry says the download is the next plan; 10-05
  dates it as 10-05 dates the two older entries.

## Self-Check: PASSED

`src/application/keeping_message_text.rs` and
`tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs` exist; `grep -c 'pub
mod keeping_message_text' src/application/mod.rs` is 1; `grep -n 'keeping_bodies_under'
src/presentation/wx_app.rs` answers two lines, both `cache.keeping_bodies_under(text_kept.budget())`;
`grep -rn 'as_bytes_or' src tests` finds nothing; `grep -c 'reading: w\.'
src/presentation/wx_settings.rs` is 1; `guards/guards.toml` holds 892 records by the TOML reader
and the census line says 90; `docs/changelog.md` holds the line beginning "**How much message text
stays on this computer is your choice"; `.planning/WINDOWS.md` holds 518 and 519 in both halves.
Commits `8271b327`, `adcb9fa7`, `f008a84e`, `e42be7ae`, `24974d12` and `c4203632` are in `git log
--oneline` on `main`.

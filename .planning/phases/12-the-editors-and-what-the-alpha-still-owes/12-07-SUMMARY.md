---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 07
subsystem: the contact editor's name parts, birthday and checks, and phone numbers read by their country
tags: [edit-02, edit-01, contacts, phonenumber, vcard, google-people, microsoft-graph, msaa, uia, "#40"]
status: complete

requires:
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "12-06.1 merged at 8098b4b1: every spin control's typing field named in the running program, which the birthday's day and year go through"
provides:
  - "Cargo.toml, Cargo.lock, .cargo/audit.toml, src/service/outward.rs: phonenumber =0.3.10 alone in build(12-07), RUSTSEC-2023-0089 accepted by name, the crate on A_CRATE_THAT_CANNOT"
  - "src/application/contact_names.rs: NameParts, TITLES, SUFFIXES, PARTICLES, guess_parts, compose; 13 cases"
  - "src/application/phone_numbers.rs: Region, Reading, Doubt, read, every_region, sentence, no_digit_sentence over phonenumber with the four routes; the only file naming the crate; 46 cases, 38 of them the audit's rows"
  - "src/service/this_machine.rs: home_region, region_name, fold_digits on the file's own kernel32 block"
  - "contacts: name_prefix, middle_name, name_suffix columns, fields, upsert and reads; vcard N with five parts; Google's honorificPrefix, middleName, honorificSuffix and Graph's title, middleName, generation both ways; PhoneEntry.country"
  - "src/presentation/wx_managers.rs: Name, Prefix, Given name, Middle name, Family name, Suffix with NameFields filling each other; the birthday row; an_address_refusal; the Add Phone Number dialog's Country list and PhoneAsker::decide_on_ok with the second OK"
  - "src/presentation/wx_item_form.rs: BirthdayFields, build_birthday_fields, as_stored_birthday"
  - "tests/the_contact_editor_fills_the_name_and_its_parts_from_each_other.rs: 22 readings of the built editor and dialogs"
  - "src/presentation/scan_target.rs, src/presentation/wx_app.rs, .github/workflows/accessibility.yml: a phone-number scan target"
affects: [12-08, which changes wx_item_form.rs next; 12-12, which runs the phase's full gate and reads the pages]

actuals:
  tokens: 52700
  tasks: 3
  commits: 10

tech-stack:
  added: ["phonenumber 0.3.10+9.0.33"]
  patterns:
    - "A library with measured defects is wrapped in one module that routes around each, and each route has a guard record whose break is the library's own behaviour"
    - "A guess fills only fields whose words are nobody's: typed now, or stored, counts as the person's"

key-files:
  created:
    - src/application/contact_names.rs
    - src/application/phone_numbers.rs
    - tests/the_contact_editor_fills_the_name_and_its_parts_from_each_other.rs
    - .planning/phases/12-the-editors-and-what-the-alpha-still-owes/12-07-SUMMARY.md
  modified:
    - Cargo.toml
    - Cargo.lock
    - .cargo/audit.toml
    - src/service/outward.rs
    - src/service/this_machine.rs
    - src/application/mod.rs
    - src/data/message_cache/mod.rs
    - src/data/message_cache/contacts.rs
    - src/data/message_cache/held_conflicts.rs
    - src/application/contacts_sync.rs
    - src/application/carddav_sync.rs
    - src/application/looking_people_up.rs
    - src/application/importing_an_outlook_data_file.rs
    - src/service/google_api.rs
    - src/service/microsoft_graph.rs
    - src/service/carddav.rs
    - src/service/outlook_data_file.rs
    - src/service/directory.rs
    - src/presentation/wx_managers.rs
    - src/presentation/managers.rs
    - src/presentation/wx_item_form.rs
    - src/presentation/ui_types.rs
    - src/presentation/scan_fixtures.rs
    - src/presentation/contact_convert.rs
    - src/presentation/scan_target.rs
    - src/presentation/wx_app.rs
    - tests/integration_tests.rs
    - tests/manager_delete_stays_open.rs
    - tests/finding_people_answers.rs
    - tests/theme_reach.rs
    - .github/workflows/accessibility.yml
    - guards/guards.toml
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md

key-decisions:
  - "Pratik on 2026-09-24: \"Use phonenumber. See if you can get around the bug.\" The two defaults of that day stand: a number is saved in the international form in its country's grouping, and RUSTSEC-2023-0089 is accepted by name. Both his to overrule."
  - "Name comes first on Basic Info, then the five parts: the tester asked for the whole name in \"the first field\"."
  - "A stored part counts as the person's, so a guess from a corrected name never writes over it; the plan said opening sets no flag."
  - "The birthday has a Birthday check box, because a date control always holds a date and a contact with none would otherwise be saved with today's."
  - "A phone-number scan target, so the Country list is read in the running program and not only in a test process."

requirements-completed: [EDIT-01, EDIT-02]

duration: about 330 min
completed: 2026-09-24
---

# Phase 12 Plan 07: The Contact Editor and Phone Numbers Summary

**The contact editor asks for Name, then Prefix, Given name, Middle name, Family name and Suffix, each filling the other as a first guess that never writes over a box somebody typed in or a saved contact opened with. The three new parts are stored and carried to and from Google, Outlook and address book servers in each one's own field. The birthday is a date with Birthday and No year check boxes. An address is checked for its shape. A phone number is read by its country through `phonenumber` 0.3.10, behind `application::phone_numbers`, which routes around the four defects measured on 2026-09-24; a valid number is saved as `+44 121 234 5678`, a doubted one is said once and kept as typed on a second OK, and only text with no digit is refused.**

On branch `12-07-phone-numbers` from `main` at `8098b4b1`, 2026-09-24.

Pratik's confirmation, quoted: "Use phonenumber. See if you can get around the bug." The two defaults of 2026-09-24, neither answered and both his to overrule: the international form for a saved number, and RUSTSEC-2023-0089 accepted by name.

## Commits

| Commit | What | Hook |
|---|---|---|
| `e542a21b` | build: the manifest line, the lock file, the census name, the accepted advisory | `all`, 606 s |
| `d7a55452` | red: the name guess, the phone reading, three Windows readers (59 named) | `red`, 145 s |
| `8a360905` | green: the two modules, the Windows readers, five records | `affected`, 135 s, after one refusal (below) |
| `36ed6bb9` | red: the three name parts through the store and the providers (12 named) | `red`, 156 s |
| `450045b6` | green: the columns, the card, the providers, the merges, five records | `affected`, 164 s |
| `286167dd` | red: the editor's readings (17 named) | `red`, 142 s |
| `99ea1914` | green: the fills, the birthday, the checks, the Country list, three records | `affected`, 153 s |
| `95a0a039` | red: a scan target for the Add Phone Number dialog | `red`, 140 s |
| `52d1b633` | ci: the scan asks for it | `all`, 478 s |
| `074856b5` | docs: the pages, the ledger, the changelog, the summary, the marks | `docs_only`, 86 s |
| `79b020d8` | red: two faults pull request #99's review found (3 named) | `red`, 107 s |
| the green after it | green: an address added without its spaces, a birthday outside 1900 to 2100 kept as it was, three records re-measured | |

`which-checks.sh` answered `all` for the build commit's staged paths before it was made, and its hook's mode line read `check.sh: mode all`. The first attempt at `8a360905` was refused by `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`: five records had arrived and the count at the top of `guards/guards.toml` said 262. It says 274 now, three raises for three batches of records.

## The package

`grep -c '^\[\[package\]\]' Cargo.lock` read 706 before the build commit and 723 after: 17 entries against the audit's 15. The two the audit did not list are `embedded-io` 0.4.0 and 0.6.1, optional dependencies of `postcard` that cargo records in the lock and that no target compiles: `cargo tree -i embedded-io@0.4.0 --target all` and the same for 0.6.1 print nothing. The Windows build compiles 13 new packages, as the audit said. `A_CRATE_THAT_CANNOT` went from 53 to 54. What the audit gate printed: "All 5 advisory(ies) this project accepts are still reported." and "No advisory outside .cargo/audit.toml, and nothing is being held open.", so RUSTSEC-2023-0089 is reported and accepted, and the audit's check that each accepted advisory is still reported passes with the new entry. The manifest pins `=0.3.10`, because the routes were measured against that version's behaviour.

`grep -rlE 'phonenumber::|use phonenumber' src tests --include=*.rs` answers `src/application/phone_numbers.rs` alone.

The release executable was 43,043,328 bytes at the build commit, where nothing yet called the crate, and 44,517,376 at `52d1b633`: about 1.4 MB, most of it the numbering data. The first load of that data, timed in the target, took 263 ms.

## The four routes, and what each record's break reddened

Every row of the audit the plan named is a case in `phone_numbers`, 38 of them, and each agreed with the audit's workaround column on the first green run. One thing the plan did not foresee: the crate leaves the general description's possible lengths empty (GB's is `[]`, read in a throwaway build), so too short and too long are judged against every kind's lengths together, which is Google's own definition. Rows 12 and 13 were red on it until then.

| Record | Break | Rows reddened |
|---|---|---|
| a number typed with a plus is read again with no chosen country | `Source::Plus => Ok(number)` | 14, 22, 24, 25, 28, 30, 31, 47, 66, 68, 71 |
| a number's region is read from its significant number with the leading zeros kept | `national().value()` | 8, 14, 22, 24, 49, 56, 57, 58, 66, 68, 74, 102 |
| a national number that begins with its country's code is read whole when that is valid | `Source::Number => Ok(number)` | 90, 93 |

Two rows the plan expected to redden the region break did not: 83 (Cote d'Ivoire) and 87 (San Marino), because each is the only region with its code.

## Tests

| Target | Count |
|---|---|
| `application::contact_names::` | 13 |
| `application::phone_numbers::` | 46 |
| `service::this_machine::` | 7, one more than it had |
| `data::message_cache::contacts::` | 103, unchanged |
| `service::google_api::` | 37, unchanged |
| `service::microsoft_graph::` | 57, unchanged |
| `application::contacts_sync::` | 281, unchanged |
| `presentation::wx_item_form::` | 15, one more |
| `presentation::wx_managers::` | 44, unchanged |
| `presentation::managers::` | 139 by `grep -c 'fn test_'`, unchanged from `main` (the plan's 137 is the count check's rule) |
| `tests/the_contact_editor_fills_the_name_and_its_parts_from_each_other.rs` | 22 |
| `tests/house_style` | 74 |

`grep -rn 'ContactEntry {' src tests --include=*.rs | grep -v 'pub struct' | wc -l` answers 114 before and after. The cases rewritten in place: the card round trip and the card over what is held, a contact saved with five parts read back by both reads (renamed from "two parts"; no record named it), Google's connections response and create, Graph's contacts response and create, `google_person_to_contact`, `ms_contact_to_contact` with its way back, the Google round trip, both exhaustive merge cases, and `contact_convert`'s round trip.

Eight target readings passed on the red tree and were not named: the three companions that plant a failure and see it, a contact with no birthday saving none, the load time, the Favourite box, every new control named (the builder named them in the red commit), and a stored contact's parts surviving a name change. The last one's expectation was wrong: it said the middle name stays empty, and the stored contact had none, so the guess fills it with "Anne". Corrected at green.

## What was read over MSAA and UI Automation

At the handle the keyboard reaches, both channels: the Prefix box's edit "Prefix,", the Middle name field "Middle name,", the Suffix box's edit "Suffix,", the Birthday check box "Birthday", the birthday's month "Birthday Month", the No year check box "No year", and the Country list "Country". A combo box's edit carries the box's name on both channels with nothing written onto the edit itself. The Favourite box reads role `0x2c`, a check box, and the name "Favorite". The no-year position is a check box, "No year", beside the year spin control, and the year is greyed while it is ticked.

## Guard records

Eight written and measured: two on `contact_names.rs` (a guess keeping the middle names, 6 red; a family name keeping its particles, 3 red), three on `phone_numbers.rs` (above), one on `contacts.rs` (N with the last three fields empty, the round trip red), three chain records for the new merge lines, and three on the editor (a typed family name overwritten, the birthday stored as year nought, a doubted number refused on its second OK). That is eleven; the plan's eight plus the three chain records it listed apart. Re-measured: the `family_name` merge record, whose anchor the insertion split and which now anchors on the `family_name` and `name_prefix` lines; "a running screen reader is named by its process" on `this_machine.rs`; "a check box carries its own label rather than borrowing the text beside it" and "the event form's minute spinner is named on its typing field too" on `wx_item_form.rs`. Every one agreed exactly. No other record's anchor moved: `test_every_guard_record_still_names_one_place_in_the_tree` stayed green through every commit.

## Ledger

Opened, both halves: 596 (the editor under NVDA), 597 (five name parts through a real account), 598 (the numbering data and pull request 110), 599 (third-party licence notices), 600 (the address dialog's English list). The front matter reads 600 entries, 545 open and 55 fixed. None closed.

## Deviations

1. **`Cargo.lock` gained 17, not 15** (above). Rule 3, recorded.
2. **Possible lengths from every kind** (above). Rule 1.
3. **Name first.** The plan put Prefix first; the tester asked for the whole name in "the first field". Rule 1.
4. **A stored part counts as the person's.** The plan said opening a stored contact sets no flag, which would let a corrected name re-guess a stored family name, the van der Berg case. Rule 2, T-12-24.
5. **A Birthday check box.** A date control always holds a date, so a contact with no birthday would have been saved with today's. A stored birthday the controls cannot show is kept as it was while the box stays unticked. Rule 2.
6. **A phone-number scan target**, in `scan_target.rs`, `wx_app.rs` and the Accessibility workflow, so the Country list is read in the running program. Rule 2. The workflow line cost one full gate, `52d1b633`.
10. **Every commit on the branch was first made with a `Co-Authored-By` line naming the assistant**, which this project's rules forbid; it was taken out of the ten messages with `git filter-branch --msg-filter`, which changed no tree (the tree listing's hash was `78c5a4bf` before and after), and the branch was pushed again. The hashes here are the rewritten ones. The pull request's first description carried the assistant's line too and was edited.
11. **Two faults found by the pull request's review, fixed after the documents commit.** An address typed with spaces around it passed the shape check and was stored with them; it is now added trimmed (`an_address_to_add` replaces `an_address_refusal`). A stored birthday in a year the year control does not offer, 1815 for one, would have opened clamped to 1900 and been saved so; it is now kept as it was, like a birthday written as words. A third point, that `PhoneEntry.country` needs `serde(default)`, was checked and does not hold: the struct carries `#[serde(default)]`, and every stored list read in the cases has no `country` key. A fourth, about `.githooks/commit-msg`'s comments, is about `main`'s own history and not this branch. The summary and the changelog line on the birthday changed in that green commit, so the documents came in two commits.
7. **Two records on `contact_names.rs` instead of one.** A split at the last space takes two edits; the two single-edit breaks are the middle names dropped (Hopper red) and the particles dropped (van der Berg red).
8. **The address check lives in Add Email Address's OK**, where addresses are added, and adds a space or a colon to what it refuses. Rule 2.
9. **Files outside the plan's list:** `contact_convert.rs` (the editor's conversion, three parts and a number's country both ways), `scan_target.rs`, `wx_app.rs`, `accessibility.yml`, `tests/theme_reach.rs` (the birthday text box gone, the middle name painted instead, the phone dialog's new return).

## Scripted edits

None. Every tracked file was changed with Read, Edit and Write; the throwaway probe of the crate's lengths was a cargo project in the scratchpad.

## What only a person can settle

The tab and the phone dialog heard under NVDA (ledger 596); five name parts through a real Google account, Outlook account and address book server (ledger 597).

## Self-Check: PASSED

The two modules, the target and this summary exist. The nine commits above are on the branch (`git cat-file -e`). `grep -c '&Country:' src/presentation/wx_managers.rs` read 1 on `main` and 2 here; the plan's other greps answer 3, 2 and 0. `the_planning_files_agree_with_themselves` 17, `house_style` 74, `the_words_that_say_nothing` 10, `a_key_is_documented_where_the_surface_that_binds_it_is` 3 and `docs_links` 6 pass; no carriage return in any document touched. The pull request's CI, NVDA and Accessibility runs, the merge commit and its gate are in the report, since this commit lands before them.

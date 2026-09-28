---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 23
subsystem: rules on arriving mail
tags: [rules, labels, imap, pop, GAP-11, GAP-12, F1]
status: complete
requires: [13-22]
provides:
  - "application::tagging: the_label_a_rule_names(labels, named) -> Option<&Tag>, no_label_of_that_name(named) -> String"
  - "mail_sync::carry_out resolves a rule's label names through the message's own account's labels and writes the label's id; Carried::NotTheLabels carries the sentences"
  - "Both checks (mail_sync::sync_folder, pop_sync::sync) extend Filtered.could_not_be_filed with the mover's sentences rather than replacing the rules' ones"
affects: [13-24, 13-24.1, 13-25, 13-40, 13-41, 13-42, 13-43, 13-44, phase 14]
tech-stack:
  added: []
  patterns: ["a rule's typed name resolved against one account's rows before it is used as a key"]
key-files:
  created:
    - tests/a_rule_that_adds_a_label_labels_the_message.rs
  modified:
    - src/application/tagging.rs
    - src/application/mail_sync.rs
    - src/application/pop_sync.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/USER_GUIDE.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/REQUIREMENTS.md
decisions:
  - "The account is read per message, through account_of_folder(message.folder_id), and only when a rule asked for a label: apply_rules has a caller outside the two syncs (filters.rs's own test), and a check with no label rule reads nothing."
  - "Name matching is Unicode lowercase rather than ASCII-only: a label is a word somebody chose, and Überweisung should match überweisung. Folders stay ASCII-only; not changed here."
  - "A missing label is Carried::NotTheLabels(sentences): the rest of the message's actions still run, the message is not counted, and the sentences go into could_not_be_filed, logged as well."
metrics:
  duration: "about 1 hour 20 minutes to the documents commit"
  completed: 2026-09-28
actuals:
  tokens: 11000
  tasks: 2
  commits: 3
---

# Phase 13 Plan 23: A rule that adds a label puts that label on Summary

A rule's Add a label now resolves the name it stores against the message's own account's
labels, in any capitals, spaces around it ignored, or by id, and writes the label's id; until
now the name went in as the id, SQLite refused the row and nothing but the log knew. A rule
naming a label the account lacks writes nothing, says one sentence folded however many
messages it matched, and does not count the message as sorted. Both checks now keep that
sentence: each replaced the rules' sentences with the mover's, which the plan did not see.

## What works, and how it is known

- **The label goes on.** `test_a_rule_that_adds_a_label_puts_the_accounts_label_on_the_message`
  makes Money the way the Label Manager does (id `tag-<n>`, keyword from its letters), runs
  `apply_rules` over a real cache, and reads Money back on the message with `changed` 1.
- **F1, run.** While red that case printed `left: [] right: ["Money"]`: nothing on the
  message. The refusal itself goes to `tracing::warn!` only, which no test captures; the same
  insert taken against SQLite in a scratch probe (foreign keys on, `INSERT OR IGNORE INTO
  message_tags` naming a tag id no row has) answered `FOREIGN KEY constraint failed`, so the log
  line read "A rule could not be carried out: Failed to add tag to message: FOREIGN KEY
  constraint failed". `OR IGNORE` does not cover a foreign key.
- **Any capitals, another account, a missing label.** The "money" case, the other account's
  Money never taken (T-13-23-01), and the Travel case: two messages, nothing written, `changed`
  0, two sentences, and `what_the_pop_check_did` saying "2 messages not filed as asked" and
  naming Travel once (T-13-23-02).
- **The checks keep the sentence.** `application::mail_sync::tests::test_a_sync_says_a_rule_named_a_label_the_account_does_not_have`
  through `sync_folder` with a scripted server, and
  `application::pop_sync::tests::test_a_check_says_a_rule_named_a_label_the_account_does_not_have`
  through the POP `sync`: both were red with `left: []`.
- **Counts, taken today.** The target 8 tests; `application::mail_sync::` 150 (149 plus one);
  `application::tagging::` 19; `application::pop_sync::` 50 (49 plus one);
  `application::filters::` 49; `a_rule_can_change_how_a_row_is_announced` 20;
  `progress_is_shown_and_results_are_said` 15; `the_log_carries_what_a_report_needs` 13.
- **Reachability.** `carry_out` is reached from `apply_rules`, which both checks call on
  arriving mail (`mail_sync.rs` in `sync_folder`, `pop_sync.rs:335`), and the summary is
  what the status line says after a check.
- The premise line numbers held: `carry_out` was at `:1081`, `the_folder_a_rule_names` at
  `:1241`, `apply_rules`' warn at `:1074`, tags at `tags.rs:211` and `:78`.

## Commits

| Commit | What | Hook |
|--------|------|------|
| `89a397b9` | test: the target's 7 red cases, the two sync cases and the count check named | red, 127 s |
| `c181459a` | fix: the resolver, `carry_out`, both syncs extending, four records, 21 re-measured | refused once at 232 s (STATE's body plan number, from the uncommitted marks), then 196 s |
| this one | docs: the changelog, the guide, the ledger, the summary and the four marks | |

## Guard records

Four new, each taken by `scripts/guards.sh --remeasure` in the one call, with the seventeen the
new library tests flagged (14 on `mail_sync.rs`, 3 on `pop_sync.rs`):

| Record | File | Suite | Red |
|--------|------|-------|-----|
| a rule's label is written by the label's id and not by the name the rule typed | `mail_sync.rs` | the target | the Money case and the "money" case, 23 s |
| a rule finds its label by name in any capitals | `tagging.rs` | the target | the "money" case and the resolver's capitals rows, 21 s |
| an IMAP sync keeps what the rules said beside what the mover said | `mail_sync.rs` | library | the IMAP sync case, 121 s |
| a POP check keeps what the rules said beside what the filing said | `pop_sync.rs` | library | the POP check case, 122 s |

Of the seventeen flagged, sixteen still named exactly what went red and had their counts
written. One was short: "a count and the thing it counts agree in number" also reddens
`application::reporting_junk::tests::test_one_message_is_said_in_the_singular`, a test 13-22
added in a file the record never named. Corrected by hand (the test, `reporting_junk.rs` at
21) and measured again on its own: all 31 named, nothing else, 91 s. So there were two
`--remeasure` calls, the second for the record the first found short. The arrived-since count
at the top of `guards/guards.toml` went from 425 to 429.

## Documents

The changelog's Fixed entry names the refused insert and the sentence, with the known
limitations: a rule's label stays on this computer on every IMAP account and a later check can
take it off, read and flag stay here too, the move-and-missing-label count, not heard, no real
account. The user guide's Labels paragraph said a rule could add a label, which had never
worked; it gains a paragraph saying how the name is matched and to make the label first.

## Ledger

- 678 (todo, F2 widened to labels): an arrival rule's read, flag and label reach this computer
  only and the flag sync can put the server's answer back. For whoever owns rules next.
- 679 (unrun-verify): the ear.
- 680 (unrun-verify, phase 14): a real account, and whether the next check takes the label off.
- 681 (todo): a message a rule both files and labels with a missing label is still counted when
  its move lands.

None closed. GAP-11 and GAP-12 are carried, not ticked; this plan is the prerequisite 13-24.1,
13-40 and 13-43 depend on.

## Deviations from Plan

**1. [Rule 1] Both checks dropped what the rules said.** `sync_folder` (`mail_sync.rs`) and
`pop_sync::sync` each assigned `filtered.could_not_be_filed = could_not` after the mover, so a
sentence from `apply_rules` never reached the summary: the plan's second truth would have held
in its tests and not in the program. Both extend now, and one case in each file holds it, so a
test landed in `mail_sync.rs` and `pop_sync.rs` after all, against the plan's cost rule: 17
records flagged and re-measured, and two more records (one per check).

**2. [Scope] `docs/USER_GUIDE.md`** was not in the plan's files. Its Labels paragraph promised
"a rule that adds one"; it now says how the name is matched.

**3. [Shape] The red's stubs were empty** (`None`, `String::new()`); each expected value was
checked by reasoning before the green. `test_the_resolver_finds_nothing_for_an_empty_name_or_one_no_label_has`
passed against the stub and was not named in the red.

**4. [Shape] Unicode lowercase rather than `eq_ignore_ascii_case`**, with an Überweisung row.

**5. [Changelog] The known limitation is wider than the plan's.** The plan said a rule's label
is not sent to Gmail. It is not sent to any server, and on an IMAP account the flag sync's
`match_labels_to_keywords` takes a label with a keyword off whenever the server reports that
message, which a server without CHANGEDSINCE does on every check. Said in the changelog and
the guide, and ledger 678.

**6. [Rule 1] The first green commit was refused** by `the_planning_files_agree_with_themselves`,
which reads the working tree: STATE's frontmatter had been moved to 27 for the documents commit
and its body's `Current Plan:` line still said 26. Corrected, and the same commit again.

**7. [Guards] A stale record from 13-22**, found by the remeasure and corrected here, as above.

### Found and left

- `say_what_the_rules_did` counts a missing label into "N not filed as asked", so a label
  reads as a kind of filing. The sentence after it says which label; left as it is.

## Threat Flags

None: no new endpoint, file access or schema. No crate added (T-13-23-SC).

## Known Stubs

None.

## Self-Check: PASSED

The target and `tagging.rs` exist; `89a397b9` and `c181459a` are on the branch;
`pub fn the_label_a_rule_names` once in `tagging.rs` and its name once in `mail_sync.rs`;
no carriage return, em dash or listed word in this file, the changelog or the guide.

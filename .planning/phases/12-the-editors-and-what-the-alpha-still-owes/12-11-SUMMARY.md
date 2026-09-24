---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 11
subsystem: the pro licence, as a design
tags: [alpha-03, "#65", "#64", pro-licence, design, decisions, ledger-608]
status: complete
requires:
  - phase: 12
    provides: "12-10 merged at 015035f7"
provides:
  - "docs/plans/20260924-pro-licence.md: ten sections and a Decisions for Pratik table of ten rows with an empty answer column"
  - "docs/development/requirements-backlog.md: one row, Pro licence, under Platform and Distribution, pointing at the design"
  - ".planning/WINDOWS.md: ledger 608, a todo naming the ten decisions as Pratik's"
affects: [12-12, which reads the eleven requirements against the tree and runs the phase's full gate; the phase that builds the licence once Pratik answers]
tech-stack:
  added: []
  patterns:
    - "A design names its seam so the tree's own absence search, grep -rniE 'licen[cs]e key|entitlement' src, keeps meaning what it meant and finds the seam once it lands"
key-files:
  created:
    - docs/plans/20260924-pro-licence.md
  modified:
    - docs/development/requirements-backlog.md
    - .planning/WINDOWS.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "None of Pratik's. Every row of the decisions table is his and none is answered; the prices and the trial he settled on 2026-09-16 are carried as decided and not asked again."
  - "The document proposes the seam as application::licence with Licence and Unlocked rather than Entitlement, because type Entitles in wx_app.rs sits one letter away and every search for one would find the other."
  - "In a dialog a gated button stays enabled and answers with the pro sentence, because Tab passes over a disabled control; in a menu the item is greyed. Platform behaviour, not yet heard for this purpose."
duration: "about 6 minutes on the clock from branch cut to this summary (19:22 to 19:28 UTC); the reading before the cut is not in that figure"
completed: 2026-09-24
actuals:
  tokens: 12000
  tasks: 2
  commits: 1
---

# Phase 12 Plan 11: the pro licence as a design, with Pratik's decisions in one table

**The pro licence is written down as a design at `docs/plans/20260924-pro-licence.md`: a
signed Ed25519 string checked offline with `ring`, kept in the credential store, one seam
named `application::licence`, a lapse that deletes nothing, his prices and merchant table
carried with their date, and ten decisions for him with the answer column empty. Nothing in
the program is gated.**

## What was done

**Task 1, the design.** Ten sections, as the plan listed them. Each claim about the tree was
re-read by command on 2026-09-24 at `015035f7` and the command is written beside it. The
backlog gained one row pointing at the design.

**Task 2, the ledger.** Entry 608, a `todo`, in both halves, no backslash: the ten decisions
are Pratik's and the entry closes when a phase is planned from his answers. `head -7
.planning/WINDOWS.md` after the edit:

```
---
schema_version: 1
open_count: 550
waived_count: 0
fixed_count: 58
total_count: 608
last_updated: 2026-09-24T22:00:00.000Z
```

One entry more and one open more than 12-10 left (549 open, 607 total).

**The four completion marks.** Phase 12's roadmap row at 15 of 16 with a sentence for 12-11;
the `- [ ] 12-11-PLAN.md` line ticked; `STATE.md`'s plan at 15 in its frontmatter
(`current_plan`, `completed_plans` 171) and its body (Current focus, Current Position,
`Current Plan: 15`); ALPHA-03's box and its `[D]` line ticked with the sections named, its
traceability row Done. Its `[S]` line, the decisions, is untouched.

## Where the ALPHA-03 `[D]` line is met

| Clause | Section |
|---|---|
| What it is for and is not | 1 |
| What exists today, each claim with its command | 2 |
| The free and pro line as a table with each feature's state | 3 |
| The licence: signed string, offline, Settings, credential store | 4 |
| The seam, a gated command greyed and labelled | 5 |
| A lapse deleting nothing, the grandfather rule | 6 |
| The prices as decided | 7 |
| The merchant table whole with its date | 8 |
| The decisions table with an empty answer column | 9 |
| What follows the decisions | 10 |

## Premises re-taken on the day

Against `main` at `015035f7` on 2026-09-24:

- `grep -rniE 'licen[cs]e key|entitlement' src --include=*.rs; echo "exit $?"` printed
  `exit 1` before the plan's work and again before the documents commit. Nothing in `src/`
  knows a licence.
- The `Entitles` alias moved: `grep -rn 'type Entitles' src` answers
  `src/presentation/wx_app.rs:13150`, not `:12920` as the plan said on 2026-09-20. The
  document names it at its line of the day.
- `entitle` still answers 23 lines in 10 files and `subscription` 202 lines, as the issue said.
- `ed25519-dalek` is in `Cargo.lock` only through `pgp` (`cargo tree -i ed25519-dalek`), not a
  direct dependency; `ring` is direct (`Cargo.toml:284`) and already verifies signatures in
  `service::signed_mail`. The document proposes `ring`, so no crate is added and
  `Cargo.toml` stays as it is.
- `update_download::verify` (`:858`) checks an Authenticode signature through
  `WinVerifyTrust`, not a signature over a string. #65 named it as the pattern and said `ring`
  came through it; the document keeps the shape and corrects the mechanism.
- `application::forget` names six owners of credential entries (its line 13 imports caldav,
  carddav, credentials, oauth, pgp and security), not five: a licence would be the seventh.
- #62, which the issue proposed as pro, is built (`39d53503`, Say this first), and so is #53's
  Outlook data file import (`8eba6a38`, reached from File). The document keeps both free under
  the product's own line and says it departs from the issue there, as part of decision 1.

## Verification

- `cargo test --test house_style && cargo test --test the_words_that_say_nothing && cargo
  test --test docs_links && cargo test --test every_number_carries_its_command_and_its_date`:
  exit 0, 74, 10, 6 and 26 passed. The plan's premise 4 counted 9 and 25 for the second and
  fourth; each has gained one test since 2026-09-20.
- `cargo test --test the_planning_files_agree_with_themselves`: run before this summary
  existed, 4 of 17 failed, each on a count of summaries on disk against the roadmap row and
  `STATE.md`, which this file is what satisfies; run again after it, green, and the result is
  in the commit message.
- Carriage returns 0 in all six files by `tr -cd '\r' < file | wc -c`; no em dash and none of
  the six words in the added lines.
- `git diff --stat main` names `.planning/` files and `docs/development/requirements-backlog.md`
  and the new design, and nothing under `src/`, `tests/` or `guards/`. No guard record
  written, no test added. `Cargo.toml` and `Cargo.lock` untouched and nothing installed
  (T-12-SC).

## Deviations from Plan

1. **The file's date.** The plan named `docs/plans/20260920-pro-licence.md` and said to take the
   day it is written; it was written on 2026-09-24, so the file is
   `docs/plans/20260924-pro-licence.md`. The plan's `files_modified` is corrected here rather
   than in the plan, as it asked; ALPHA-03's `[D]` line keeps its old path and says the file's
   real name beside it.
2. **Two rows beyond premise 3's eight.** The decisions table carries premise 3's eight rows and
   two more the design raises and cannot settle without deciding for Pratik: when gating starts
   (and so the grandfather date), and how long the grace period after a lapse is. The issue had
   proposed 1.0.0 for the first, and the document marks both rows as raised by it.
3. **One commit before the merge, and none after.** The brief asked for one documents commit
   before the merge. The plan's verification also asked for a commit on `main` after the merge
   adding the merge commit's mode and stage lines to this summary; following the brief and
   12-10's precedent, there is none, and the merge's hook line is in the report instead.
4. **The worked $19 sale and two $10 figures that differ.** The plan asked for one worked $10 and
   one worked $19 sale. Worked from the fee lines of Pratik's table, Stripe Managed Payments
   comes to $0.94 at $10 where his comment says about $1.01, and Gumroad to $2.09 where it says
   about $1.79, which is exactly the 30-cent fixed part of card processing. The document reports
   both differences and changes neither of his figures.
5. **A dialog's gated button.** The plan's section 5 said a gated command is visible and
   greyed. The document keeps that for menus and proposes an enabled button in a dialog, because
   Windows passes over a disabled control when Tab moves focus, so its label would never be
   heard. Marked as platform behaviour not yet heard for this purpose.

## Found, not fixed

- `docs/development/requirements-backlog.md` says under Future Protocols that "CardDAV not
  built", and `src/application/carddav_sync.rs` and `src/service/carddav.rs` exist. Out of scope
  here; 12-12 reads the pages, and this is for it.

## Known Stubs

None. The plan builds no code; the design says in its first paragraph that nothing is built.

## Threat Flags

None. The design touches no code. T-12-37 is held by the empty answer column and the unchanged
`src/`, `tests/` and `guards/`; T-12-38 by the date on every price and fee and the sentence that
the fee pages carry no dates.

## Next

After the merge, #65 gets a comment listing the ten decisions and the document's path, saying
nothing is gated and the prices and trial are carried as decided. The issue stays open for his
answers. 12-12 is next, the phase's closing plan with its full gate.

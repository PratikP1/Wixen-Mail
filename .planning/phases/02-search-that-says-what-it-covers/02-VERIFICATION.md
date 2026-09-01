---
phase: 02-search-that-says-what-it-covers
verified: 2026-09-01T13:14:51Z
status: gaps_found
score: 5/6 must-haves verified
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "A search that can reach message text says, before it runs, how many messages in the account have body text stored and how many do not, so a short answer is never mistaken for a complete one."
    status: partial
    reason: >-
      Delivered for the saved-search path only. The quick search box also reaches
      message text (the FTS index declares a `body` column and `search_messages`
      runs an unqualified MATCH for All Folders and Current Folder), and it also
      misses messages whose body was never fetched, because
      `index_message_for_search` writes no body for them and the snippet is
      derived from the body too. On that path nothing is said before the search
      runs and no fetch is offered. SEARCH-02's own cited evidence is about
      exactly this path: "FTS covers subject and sender for everything and body
      text only for bodies actually fetched, and the search UI must say so."
    artifacts:
      - path: "src/presentation/managers.rs"
        issue: >-
          `search_messages` (line 1792) reports only a match count. No call to
          `how_much_message_text_is_stored_here`, no coverage sentence, no
          `UIUpdate::WhatCouldBeFetched`.
      - path: "src/data/message_cache/searching.rs"
        issue: >-
          `index_message_for_search` (line 282) indexes an absent body as
          nothing, so a never-fetched message is silently unmatchable on a
          body-only word with no disclosure anywhere.
    missing:
      - >-
        A coverage sentence before a search-box run whose terms can reach message
        text, worded for the box the way `what_a_saved_search_covers` is worded
        for the saved search, with its own number (the FTS coverage, which is not
        the saved search's number).
      - >-
        The fetch offer reachable from the box path, or a deliberate written
        decision that it is not, since today the only door to
        `fetch_the_missing_message_text` is running a saved search that reads
        message text (recorded open as WINDOWS ledger entry 13).
human_verification:
  - test: "Run a saved search that has a condition on the message text, with NVDA running."
    expected: >-
      The scope sentence and the coverage sentence are heard as one line before
      the results arrive, and are not swallowed by the "Running this saved
      search" line they share the "status" topic with.
    why_human: "Announcement audibility and topic coalescing cannot be observed without a screen reader. WINDOWS ledger 10 and 17."
  - test: "After that search, tab to the offer above the message list."
    expected: >-
      The button is announced with its full label including the count, and the
      experimental sentence beside it is reachable and read.
    why_human: "MSAA/UIA name resolution is not readable back from wxdragon. WINDOWS ledger 12."
  - test: "Open Message > Saved Searches > Edit Conditions on a saved search, add a condition, and close."
    expected: >-
      The modal opens, the two Choice controls announce as Match field and Match
      type rather than unnamed combo boxes, the Pattern box is skipped cleanly in
      the tab order for the four match types that read no pattern, and the
      per-field caveat line is heard when the field changes.
    why_human: >-
      The condition manager has never been opened in a running build; no modal
      loop has run. WINDOWS ledger 14, 15, 20, 21, 22, 25, 26.
  - test: "With two accounts holding saved searches, arrow through the folder tree."
    expected: >-
      The Saved Searches heading, each account branch under it, and each search
      three levels deep read distinguishably, and landing on a search announces
      that the working account has moved.
    why_human: "The account branches have never been drawn in a running build. WINDOWS ledger 29, 30, 31."
  - test: "Point a real IMAP account at the bulk fetch and press the offer button."
    expected: >-
      A run of hundreds of BODY.PEEK fetches is permitted, and throttling or a
      dropped connection is reported rather than silently ending the run.
    why_human: >-
      No provider has ever seen this code. This is the risk the experimental
      sentence names. WINDOWS ledger 11, and 19 and 32 for the folder-narrowed
      and two-account cases.
---

# Phase 2: Search that says what it covers — Verification Report

**Phase Goal:** A search returns what the user asked for, and says plainly what it could not reach.
**Verified:** 2026-09-01T13:14:51Z (against `main` at `0cc5e84`)
**Status:** gaps_found
**Re-verification:** No — initial verification

## The short answer

Five of the six success criteria are delivered, wired to live callers, and backed
by tests that exercise behaviour rather than presence. The first half of the goal
holds: a search now returns what the user asked for. The second half holds for
one of the two searches in this program and not for the other. A saved search
says plainly what it could not reach. The search box, which is the search most
people use and which also reaches message text, says nothing.

That is criterion 4, and it is the one gap. It is not a stub and not a broken
wire; it is a scope that stopped one path short of where the criterion and
SEARCH-02's own evidence line put it.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A search saved with Subject Only or From Only reruns with that restriction | VERIFIED | `what_that_answer_looks_at` (`saved_searches.rs:499`) returns one field for SubjectOnly and SenderOnly and the three-element `WHAT_A_TYPED_SEARCH_LOOKS_AT` itself (not a copy) for the other two. The live chain runs Find dialog to `keep_the_search_that_ran` (`wx_app.rs:14556`) to `save_this_search` to `what_a_typed_search_asks` to `create_saved_search`. The folder half travels in the same value: `asks.folder` at `wx_app.rs:6859`, replacing the old hardcoded `folder: None`. Tests at `saved_searches.rs:1819`, `1842`, `1858`, `1882`. |
| 2 | Opening a saved search says what it asks, in one sentence, whether or not the In box has a name for it | VERIFIED | `a_search_in_words` (`saved_searches.rs:601`) builds the sentence from the questions, one clause each, joined by the search's own word. There is no scope-name path and so no fallback case. Emitted by `run_a_saved_search` before the gather (`wx_app.rs:6593`) as one `StatusUpdated`, which `handle_update` both paints and announces. Tests at `2086`, `2093`, `2105`. Audibility is a human item. |
| 3 | A rule editor writes into the same saved searches, reaching all eleven fields | VERIFIED | `the_words_for_every_field` (`wx_managers.rs:2619`) maps `A_FIELD_A_RULE_MAY_NAME`; all eleven have entries in `WHAT_EACH_FIELD_IS_CALLED`, so the `filter_map` drops none. Eleven match types likewise. One stored thing, one matcher: `Question::as_a_rule` is unchanged and is still the only conversion. One group in the tree: `saved_search_rows` is the only producer of a `SavedSearch` row. Write-back is `replace_saved_search`, one `unchecked_transaction`. |
| 4 | A search that can reach message text says, before it runs, how much body text is stored | PARTIAL — see gap | Delivered for the saved search: `coverage_before` to `how_much_message_text_is_stored_here` (real SQL, `saved_searches.rs:428`) to `what_a_saved_search_covers`, said before `messages_a_saved_search_reads`. Not delivered for the search box, which reaches the same body column and says nothing. |
| 5 | Fetching the missing text is built and gated, with a read dimension on `Allowed`, on by default | VERIFIED | `Allowed::reading` with a hand-written `Default`, `NOTHING` holding `reading: true`, and a named `default_reading` serde default. `fetch_body` asks `may_i_read` as its first line, before `require_selected`. `connect_imap` calls `allow_reading` from the account's answer. Settings checkbox reads and writes (`wx_settings.rs:1560` and `2146`). Bulk fetch reachable from a real button. |
| 6 | Saved searches sit inside the account structure the way pinned folders do | VERIFIED | `saved_search_rows` (`folder_tree.rs:645`) builds heading, per-account branch, search, through the same `what_each_account_has` Favourites uses. Fed from every account, not the open one (`wx_app.rs:10101`). Row identity carries the account. A search runs against its own account through `whose_mail_a_saved_search_reads`, and `run_a_saved_search` no longer takes the held state at all. |

**Score:** 5/6 truths verified (0 present, behaviour-unverified)

### Reachability re-checked against the tree, not against the summaries

Guardrail 1 says nothing is done until a non-test path reaches it. Plan 02-06
built a dialog nothing could open and 02-07 wired it, so every user-visible
capability was traced by hand rather than read off a summary. All six chains
hold.

| Capability | Chain, checked in the tree |
|---|---|
| Save a narrowed search | Find dialog (`ID_SEARCH`, line 4631) to `keep_the_search_that_ran` to `ID_SAVE_SEARCH` to `save_this_search` to `what_a_typed_search_asks` to `create_saved_search` |
| Edit a search's conditions | Message > Saved Searc&hes > Edit &Conditions (`ID_EDIT_SEARCH_CONDITIONS`, menu built 5854, attached 6127) and the saved-search row's context menu (`context_menu.rs:241`, focus produced at `wx_app.rs:3037`) to `edit_the_chosen_searchs_conditions` to `show_rule_manager_dialog` to `show_rule_edit` to `build_rule_edit_dialog`, then `replace_saved_search` |
| Coverage sentence | folder-tree activate or Refresh to `run_a_saved_search` to `coverage_before` to `how_much_message_text_is_stored_here` to `StatusUpdated` to status bar plus `announce_topic` |
| Fetch the missing text | `offer_button.on_click` (line 1762) to `start_the_missing_text_fetch` to `fetch_the_missing_message_text` to `fetch_over_a_mailbox`; the button is built at 921, named on both channels, added to a sizer, shown by the `WhatCouldBeFetched` arm |
| The read gate | Settings Permissions tab checkbox to `AppConfig.allowed_changes.reading` to `allowed_for` to `session.allow_reading` to `may_i_read` in `fetch_body` |
| Saved searches per account | `read_the_tree_back` to `every_saved_search` over all accounts to `folder_tree::rows` to `saved_search_rows` |

WINDOWS ledger entries 23 and 24, both `kind: stub` and both marked fixed, were
re-checked and are genuinely fixed: `show_rule_edit` and `replace_saved_search`
each now have a caller that starts at a menu item.

### Criteria that were defective as criteria, reported separately from the gap

None of the six roadmap criteria is defective. The wrong premises the executors
found were in plan acceptance criteria, not in the roadmap contract, and each is
recorded in its summary. Two are worth carrying forward because they are the
shape that repeats:

- **Criterion 6's second clause was already true.** "Two accounts each holding a
  search of the same name are never two identical rows" holds because
  `saved_searches.id` is unique across the table, so the account was never needed
  to tell them apart. 02-08 found this, rewrote the test to say what is really at
  stake (a row's spelling must not lose either half of what the row is), and
  recorded that the first version passed on arrival. The criterion's real
  delivery is the tree placement, and that part is new and is verified above.
- **02-03's plan required an equality that cannot hold.** The offer count and the
  coverage subtraction differ for a message that has no server to ask. The offer
  now counts the fetch list rather than a subtraction, which is the correct
  answer, and a test builds the divergent case.

I found no criterion satisfied only by a grep that cannot tell a use from a
mention. Every criterion above was confirmed by following the call chain in the
tree rather than by counting occurrences.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `src/application/allowed.rs` | Read dimension, on by default, exception written into the type | VERIFIED | Module doc, field doc, `default_reading`, hand-written `Default`, `NOTHING` with `reading: true` and the reason beside it |
| `src/data/config.rs` | An older settings file still parses | VERIFIED | `test_a_settings_file_written_before_reading_was_a_setting_still_loads` (line 1026) removes the key rather than round-tripping it and asserts the other settings survive |
| `src/application/saved_searches.rs` | Narrower question set, sentence builder, coverage sentence | VERIFIED | `what_that_answer_looks_at`, `a_search_in_words`, `what_a_saved_search_covers`, `what_a_saved_search_cannot_find_with` |
| `src/application/mail_sync.rs` | Bulk fetch behind the gate | VERIFIED | `fetch_the_missing_message_text` and `fetch_over_a_mailbox`; gate checked before the list is read and before any connection |
| `src/presentation/wx_managers.rs` | Condition manager and condition dialog over eleven fields | VERIFIED | `show_rule_manager_dialog`, `show_rule_edit`, `build_rule_edit_dialog`, `the_words_for_every_field` |
| `src/presentation/folder_tree.rs` | Saved searches under account branches | VERIFIED | `saved_search_rows`, `WhichRow::SavedSearchesIn`, identity carries the account |
| `src/presentation/managers.rs` | Coverage disclosure on the box path | MISSING | `search_messages` reports a count only |

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `wx_app.rs` offer button | `mail_sync::fetch_the_missing_message_text` | `start_the_missing_text_fetch` | WIRED |
| `wx_app.rs` menu and context menu | `wx_managers::show_rule_manager_dialog` | `edit_the_chosen_searchs_conditions` | WIRED |
| `wx_managers::show_rule_manager_dialog` | `wx_managers::show_rule_edit` | `run_manager_loop` closure | WIRED |
| `edit_the_chosen_searchs_conditions` | `message_cache::replace_saved_search` | `the_search_to_write_back` | WIRED |
| `run_a_saved_search` | `how_much_message_text_is_stored_here` | `coverage_before` | WIRED |
| `wx_settings` checkbox | `Allowed::reading` | `AppConfig.allowed_changes`, read at `connect_imap` | WIRED |
| `managers::search_messages` (the box) | any coverage count | nothing | NOT WIRED |

### Data-Flow Trace (Level 4)

| Value shown | Source | Real data | Status |
|---|---|---|---|
| Coverage numbers | `SELECT COUNT(*), COUNT(b.message_id) ... LEFT JOIN message_bodies` | Yes | FLOWING |
| Offer count | `messages_with_no_text_here`, a real query over the same join with `ONLY_COPY_IS_HERE` excluded | Yes | FLOWING |
| Saved-search rows | `every_saved_search` over `SavedSearchesRead` per account | Yes | FLOWING |
| Condition list in the editor | The stored `Question` list, written back whole | Yes | FLOWING |
| Scope sentence | Built from the stored questions, not from a scope name | Yes | FLOWING |

### Behavioural Spot-Checks

Skipped by instruction and by economy. `bash scripts/check.sh all` was run by the
developer before this verification and passes: 5,974 library tests, every
integration target, the release build. Re-running it, or compiling to run one
named test, would duplicate a gate that has already answered and would tell this
report nothing it does not already have from the source. Test existence was
confirmed by enumeration instead, and the tests that carry each criterion are
named in the truths table above.

No probe scripts exist in this repository (`scripts/` holds `check.sh`,
`guards.sh` and build helpers only), so Step 7c does not apply.

### Anti-Patterns Found

None. Every file this phase touched was scanned for `TBD`, `FIXME`, `XXX`,
`TODO`, `HACK`, `PLACEHOLDER`, `todo!` and `unimplemented!`. There are no
matches, so there is no unreferenced debt and completion stays auditable.

### Requirements Coverage

| Requirement | Status | Evidence |
|---|---|---|
| SEARCH-01 | SATISFIED | All four sub-criteria. The fourth, about an older search with no field restriction, disappears rather than being handled: `what_that_answer_looks_at` returns `WHAT_A_TYPED_SEARCH_LOOKS_AT` itself for an unnarrowed search, so there is no absent value for a reader to interpret. |
| SEARCH-02 | PARTIAL | The fetch is built, gated, and reachable; the disclosure exists and is real. Both live on the saved-search path only. The requirement's own evidence line is about FTS coverage and says "the search UI must say so", and the search UI does not. Additionally the fetch has never met a live server, which is a human item rather than a gap. |
| SEARCH-03 | SATISFIED | Same rule vocabulary as filters, one matcher, one store, one group in the tree. Opening a saved search re-runs it over the cache rather than replaying a snapshot. Nothing on the path touches a `may_i` gate, so the third sub-criterion holds by construction. |

`REQUIREMENTS.md` marks all three complete and the coverage table says
"Complete" for each. SEARCH-02 should be reopened, or the gap accepted with an
override, before that line is true.

## What is not verifiable here and is not counted as passing

Nothing in this phase has been drawn in a running build, heard under a screen
reader, or run against a live mail account. The ledger stands at 30 open
entries, 22 of them raised by this phase and almost all `unrun-verify`. In
particular the coverage sentence and the scope sentence are emitted and
announced but nobody has heard them, the condition manager's modal loop has
never run, the saved-search account branches have never been drawn, and the bulk
body fetch has never met an IMAP server. These are listed under Human
Verification in the frontmatter. They are not gaps in the work and are not
counted against the score. Screen reader testing is Pratik's and he decides
when.

## Gaps Summary

One gap, and it is a scope rather than a defect.

The phase set out to make a search say plainly what it could not reach. It built
the machinery for that honestly: a real count, a sentence worded for speech, a
remedy beside the sentence, and a decision (D-2-13) that faced squarely the fact
that this program has two searches which cover different amounts of the same
mailbox. The decision's own conclusion was that naming the two coverages is more
honest than collapsing them into one number.

Only one of the two got named. The saved search says what it covers and offers
the fix. The search box, which reaches the same body text through the FTS index
and misses the same never-fetched messages, says nothing before it runs and
offers nothing after. That is the failure the criterion was written to prevent,
still live on the path a person uses first.

The narrowness of the remedy is already recorded as ledger entry 13 and open;
the narrowness of the disclosure is not recorded anywhere, which is why it is
reported here rather than treated as accepted.

**This may be intentional.** D-2-08 and D-2-13 scope the disclosure to the saved
search without ever saying the box does not need one, and 02-02 chose the
wording deliberately. If the intent was that the box's coverage is a separate
piece of work, accept it explicitly rather than by silence, by adding to this
file's frontmatter:

```yaml
overrides:
  - must_have: "A search that can reach message text says, before it runs, how many messages in the account have body text stored and how many do not"
    reason: "The disclosure ships for the saved search this phase. The search box's own coverage number is different from the saved search's and is scoped to its own work."
    accepted_by: "Pratik"
    accepted_at: "2026-09-01T00:00:00Z"
```

Accepting it should also mean correcting SEARCH-02's tick in `REQUIREMENTS.md`,
because a requirement marked complete while its own cited evidence is unaddressed
is the kind of document that costs somebody an afternoon later.

---

*Verified: 2026-09-01T13:14:51Z*
*Verifier: Claude (gsd-verifier)*

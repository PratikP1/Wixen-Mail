# Phase 2: Search that says what it covers - Context

**Gathered:** 2026-08-31
**Status:** Ready for planning

<domain>
## Phase Boundary

A search returns what the user asked for, and says plainly what it could not
reach.

Requirements: SEARCH-01, SEARCH-02, SEARCH-03.

**Two things the scout found that change the size of this phase in opposite
directions, and the planner needs both.**

SEARCH-01 is much smaller than its requirement implies. `saved_search_questions`
is a separate table storing an arbitrary set of questions with positions, so a
narrower question set is already storable, readable and runnable. The defect is
entirely in `what_a_typed_search_asks`, which always emits the three questions in
`WHAT_A_TYPED_SEARCH_LOOKS_AT`. No schema change.

SEARCH-02 is larger, because D-2-06 widens `application::allowed` to cover reads,
which is a model three places must agree on and which this phase was not
otherwise going to touch. That was chosen deliberately with the cost stated.

**SEARCH-03 turns out not to be a separate feature at all.** `Question::as_a_rule`
converts a saved-search question into a `FilterRule` to evaluate it, and its own
comment says why: inventing a second matcher "would be the second vocabulary this
module exists to not have". So saved searches do not resemble filter rules, they
are filter rules with the action ignored. The only real gap is reach:
`A_FIELD_A_RULE_MAY_NAME` holds eleven fields and the search box uses three.

</domain>

<decisions>
## Implementation Decisions

### What a smart folder is

- **D-2-01:** A smart folder is **a saved search with a fuller editor**, not a
  second object. One stored thing, two doors into it: the search box keeps
  writing its three questions, and a rule editor writes any of the eleven fields
  in `A_FIELD_A_RULE_MAY_NAME` with any match type in `A_WAY_A_RULE_MAY_MATCH`.
  Both land in `saved_searches` and both run through `as_a_rule`, so there stays
  one matcher, one storage and one row shape.
  SEARCH-03's criterion says a smart folder "appears in the folder tree beside
  saved searches", which assumes two things exist. **That criterion is corrected
  by this decision**, and it was written before anyone had read `as_a_rule`.

- **D-2-02:** **One group in the tree, however a search was made.** Everything
  stored appears under the existing saved-searches heading.
  The alternative was grouping by which door made it. Checked rather than
  assumed, and it is not merely different, it is worse: because the two doors
  edit one object, a typed search opened in the rule editor and given a body
  condition would have to move groups, or stay in a group that no longer
  describes it. Grouping by provenance breaks the moment a thing can be edited by
  the other door.

### The field restriction

- **D-2-03:** Subject Only writes **one question instead of three**. No new
  column, no stored scope, no migration. `what_a_typed_search_asks` stops
  emitting the whole of `WHAT_A_TYPED_SEARCH_LOOKS_AT` and emits what the In box
  was set to.
  This satisfies SEARCH-01's criterion about the reader's answer for a missing
  restriction matching the writer's answer for an unrestricted search **by making
  that case disappear**: a search saved by an older version has three questions
  and goes on behaving exactly as it does today. There is no absent value to
  interpret, so the two answers cannot come apart.

- **D-2-04:** Opening a saved search **says what it asks, not which scope it
  is**: "looks at subject and body" rather than "Subject Only". One sentence
  builder for every set, named or not.
  The reason is D-2-01: the rule editor makes question sets the In box has no
  name for, so a scope-name path would need a fallback anyway, and a search that
  stopped matching a named scope would silently change how it describes itself.

### Where saved searches live

- **D-2-05:** Saved searches **mirror the account structure**, exactly as
  Favourites does under D-29: one group with account sub-branches inside it.
  They are already account-scoped in the data. `saved_searches` carries an
  `account_id`, the read is `WHERE account_id = ?1`, and `run_a_saved_search`
  takes the active account. Only the tree placement was global, which was
  invisible while one account showed at a time and is not now.
  — **Reversibility:** reversible — a tree-shape change with the data already
  scoped correctly underneath it.

### Body text, and what gates a read

- **D-2-06:** `application::allowed` **gains a read dimension**, and the body
  fetch sits behind it.
  This was chosen over a standalone setting or an ask-each-time dialog, with the
  cost stated: `Allowed` is a model three places must agree on, and widening it
  is work this phase would not otherwise do.
  It matters that the existing model does not cover this at all. Every `may_i`
  call in `src/service/protocols/imap.rs` gates a write: subscribe, create,
  rename, delete. Nothing gates a read, because reading was never the risk.
  `Allowed`'s doc comment says "What may be changed at a provider" and both its
  fields are writes, so **the type's own description stops being accurate** and
  has to be rewritten with the dimension.
  — **Reversibility:** costly — three places must agree, and the struct is
  serialised into stored configuration.

- **D-2-07:** The read dimension is **on by default**, which is an exception to
  `Allowed`'s stated rule that `Default` is the safe end of every field, and the
  exception is written into the type rather than left for a reader to discover.
  The rule holds for writes because off is unambiguously safer: nothing happens,
  and nothing irreversible can. A read inverts it. Nothing a body fetch does is
  irreversible, and off makes every search silently cover a fraction of the
  mailbox until somebody finds the setting, which is precisely the failure
  SEARCH-02 exists to prevent. So `Default` stops meaning "changes nothing" and
  starts meaning "the safe end of each", and the field's own comment carries the
  reason.

- **D-2-08:** SEARCH-02's two halves ship together, the fetch behind D-2-06's
  gate. The disclosure half says how many messages in this account have body text
  stored and how many do not, before the search runs, so a short answer reads as
  narrow coverage rather than as an empty mailbox. The fetch half is real code
  and is marked experimental where somebody meets it, because it has never run
  against a real account.

### One coupling the requirements do not state

- **D-2-09:** Keep the ordering: the vocabulary widening comes before or with
  the disclosure, never after it. **The reason first given for it was wrong, and
  the correction matters more than the ordering does.**

  As written on 2026-08-31 this said the live search reads `m.subject`,
  `m.from_addr`, `m.to_addr` and `m.snippet` and never touches the bodies table,
  so no search could need body text until a rule editor existed. Those four
  columns are the `SELECT` payload, not the predicate. Message text is already
  reachable by both search paths and always has been:

  - The FTS5 index is declared `fts5(subject, from_addr, snippet, body, ...)` at
    `src/data/message_cache/mod.rs:2170`, and `index_message_for_search` fills
    the `body` column whenever a body is stored
    (`src/data/message_cache/bodies.rs:314`).
  - `run_a_saved_search` chooses `TheMessageText::Read` and joins
    `message_bodies` (`src/presentation/wx_app.rs:6324`).

  **So the disclosure in D-2-08 is independently shippable and does not wait for
  the editor.** A search can silently cover a fraction of the mailbox today, and
  that is a live defect rather than one this phase is about to introduce.

  The lesson is worth keeping beside the correction, because it is general: a
  negative claim about a query must cite the predicate, never the columns the
  query returns. This one was made by reading a `SELECT` list, and it is the
  fourth decision in this project to read as careful and be wrong.

- **D-2-10:** **Eviction does not reindex, and the two search paths now disagree
  about the same message.** `evict_bodies_over` deletes the row from
  `message_bodies` and never calls `index_message_for_search`, so the FTS index
  goes on holding the words while the saved-search scan, which joins the table,
  loses them.

  This inverts what D-2-08 assumes. The disclosure was written to stop a search
  looking complete while covering a fraction of the mailbox; here one path is
  more complete than the other and neither says so. Whether eviction reindexes,
  or the disclosure carries the difference between the two paths, is a decision
  the plan must make rather than inherit, and it is recorded here as open.

### The two traps the compiler cannot catch

Both found by the phase research, both verified against the source, and both
severe enough that widening `Allowed` without them is worse than not widening it.

- **D-2-11:** `Allowed::NOTHING` **holds `reading: true`**, and the constant
  stops meaning "every field false".

  `src/presentation/first_run.rs:61` maps `Choice::ReadOnly` to
  `Allowed::NOTHING`, and line 70 labels that choice **"Read my mail, change
  nothing"**. `--read-only` resolves to the same constant. So a reading field
  defaulting to `false` inside `NOTHING` would switch reading off for the one
  option whose label promises it, and it would do that to the most cautious
  person in the user base, who chose it precisely because it sounded safe.

  The constant's own doc comment already says "Nothing may be changed", which is
  the meaning to keep. What changes is that the shape stops matching the name
  literally, so the reason goes in the comment beside it rather than being left
  for the next reader to reconstruct.

  **`Default` must be written by hand.** Deriving it gives `false` for a bool,
  which is exactly the wrong answer here, and D-2-07 already made this field an
  exception to the safe-end rule.

- **D-2-12:** **A third field must not break every existing config file.**
  `Allowed` carries no field-level serde attributes and is serialised into
  `app_config.json`. Adding a field without handling the absent case makes every
  existing file fail to parse, **and that takes every other setting down with
  it**, not just this one. Whatever is done here, the deserialisation of a file
  written before this phase is a test, not an assumption.

  Note the interaction with D-2-11 and D-2-07: the absent case and `Default`
  must both answer `true`, and a bare `#[serde(default)]` answers `false`. The
  two obvious ways to write this are both wrong in the same direction.

### Eviction, and the two paths that disagree

- **D-2-13:** **Eviction leaves the index alone, and the disclosure names which
  search it is about.** Reversed 2026-08-31, on the same day it was decided.

  As first written this said eviction should call `index_message_for_search` so
  the FTS index forgets what `message_bodies` forgot, and the two search paths
  agree again. The question that produced it framed the disagreement as a
  defect and did not carry the cost, which the phase research names: **an
  evicted message stays findable by quick search**, on words the cache no longer
  stores. That is not a bug from where somebody is standing. It is a search that
  works, and reindexing takes it away, so a message becomes unfindable at the
  moment its body is evicted rather than merely unsearchable by body.

  So the index keeps what it has. What changes is the sentence: the coverage
  disclosure says **which** search it describes, rather than implying one number
  covers both. There are genuinely two coverages here and naming them is more
  honest than collapsing them.

  The objection to this, raised when it was first offered and still true, is
  that it asks a person to hold two models. That is the price, and it buys back
  a capability nobody asked to lose.

  **The eviction path still gets a comment.** The behaviour is now deliberate
  rather than accidental, and `evict_bodies_over` not reindexing should say so
  where somebody reading it would otherwise file a bug.

### The folder half of a saved scope

- **D-2-14:** **Choosing Current Folder writes the folder into the saved
  search**, alongside the narrower question set, and one path writes and reads
  both halves.

  `wx_app.rs:6545` hardcodes `folder: None` today, with a reason that stops
  holding the moment D-2-03 writes the field restriction down. SEARCH-01's
  second criterion asks for the folder half and the field half to be "written
  and read back together", and D-2-03 covers only the field half, so without
  this the criterion is not met and one of the In box's four options is still
  not saved.

  Writing them together is also the point rather than a tidiness: two things
  describing one scope, written by different code, is the shape that comes apart.

### Claude's Discretion

- The wording of the coverage sentence, subject to it giving both numbers.
- Whether the rule editor is a dialog or a page, and where it is reached from.
- How the read dimension is named in `Allowed`, subject to the doc comment being
  rewritten rather than left describing writes only.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### The plan and its requirements
- `.planning/REQUIREMENTS.md` — SEARCH-01, SEARCH-02, SEARCH-03. SEARCH-01 was
  rewritten on 2026-08-29 after its original evidence proved false; read the note
  in its evidence line before trusting anything about the scope selector.
- `.planning/ROADMAP.md` §Phase 2 — four success criteria. Criterion 4 assumes
  smart folders are separate from saved searches and is corrected by D-2-01.
- `.planning/phases/01-folders-and-conversations/01-CONTEXT.md` — D-29 is the
  precedent D-2-05 follows; D-13 and D-25 are the tree this builds on.

### Saved searches and filters, which are one vocabulary
- `src/application/saved_searches.rs` — `Question`, `Question::as_a_rule` (line
  116, and read its comment), `what_a_typed_search_asks` (350),
  `WHAT_A_TYPED_SEARCH_LOOKS_AT` (340), `SavedSearch.folder` (543), `Join`.
- `src/application/filters.rs` — `FilterRule` (23), `A_FIELD_A_RULE_MAY_NAME`
  (61, eleven fields), `A_WAY_A_RULE_MAY_MATCH`. Note the comment at line 28: the
  doc said three fields while the reading handled eleven, so a rule on the body
  read as unsupported to anyone who believed it.
- `src/data/message_cache/saved_searches.rs` — the two tables, the per-position
  question write (101), and `WHERE account_id = ?1` (192).

### Search execution
- `src/data/message_cache/searching.rs` — `WhereToSearch` (55), `search_messages`
  (389) and its column list, which reaches subject, from, to and snippet and
  never the bodies table.
- `src/presentation/wx_app.rs` — `what_the_in_box_offers` (14776),
  `run_a_saved_search` (6254).
- `src/data/message_cache/bodies.rs` — the size budget and least-recently-read
  eviction that make the coverage question real.

### What this phase changes that is not search
- `src/application/allowed.rs` — `Allowed` (38), its doc comment, `NOTHING`,
  `EVERYTHING`, and the rule that `Default` is the safe end. D-2-06 and D-2-07
  both change what this type means.
- `src/service/protocols/imap.rs` — every `may_i` call site, all of them writes.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets
- `Question::as_a_rule` — the whole of why SEARCH-03 is an editor rather than a
  feature. One matcher already serves both.
- `saved_search_questions` — already stores an arbitrary set with positions, so
  D-2-03 needs no schema work.
- `folder_tree::rows` — multi-account since Phase 1, and already places the
  saved-search branch; D-2-05 moves it rather than building it.
- The five settings Phase 1 added, and the mirror guard that catches a setting
  stored and never offered. Anything D-2-06 adds inherits both.

### Established patterns that constrain this
- **One vocabulary for asking about a message.** Stated in `as_a_rule`'s comment
  and enforced by conversion rather than by discipline. Do not add a second.
- **A saved search is account-scoped in the data.** The tree was the only place
  that forgot.
- **`Allowed` is safe-by-default and serialised.** D-2-07 breaks the default rule
  deliberately; it must be written down in the type, not just here.

### Integration points
- `what_a_typed_search_asks` is where D-2-03 lands, and it is one function.
- `folder_tree::rows` is where D-2-05 lands, beside the Favourites grouping.
- `search_messages`'s column list is where the body field becomes reachable.
- `Allowed` and every `may_i` site are where D-2-06 lands.

</code_context>

<specifics>
## Specific Ideas

- "1 if 2 isn't different than 1" — the instruction that settled D-2-02. The
  answer was to check whether the alternative differed in behaviour rather than
  in description, and it did, in the direction that made it worse.
- The read dimension being on by default was chosen with the struct's own rule in
  front of it, as a stated exception rather than an oversight.

</specifics>

<deferred>
## Deferred Ideas

- **Widening `Allowed` is the largest thing here and it is not about search.** If
  it turns out to ripple further than the three places, it is a candidate for its
  own phase rather than something to absorb quietly. Say so rather than growing
  this one.
- Nothing else surfaced. The discussion stayed inside the phase.

</deferred>

---

*Phase: 2-Search that says what it covers*
*Context gathered: 2026-08-31*

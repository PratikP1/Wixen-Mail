# Phase 2: Search that says what it covers - Research

**Researched:** 2026-08-31
**Domain:** In-repo. SQLite/FTS5 search, a serialised permission model, and a wxdragon rule editor.
**Confidence:** HIGH on everything answered from source read this session. One correction to
`02-CONTEXT.md` is stated below and is the largest single finding.

## Summary

Every claim in this document about a discrete value, a schema statement or a code path was read
from the file this session and is quoted verbatim beside the claim. Nothing here was taken from a
web search, and no external package is added by this phase.

The three questions asked have three answers, and one of them changes the shape of the phase.

**Question 1.** The three places are the three fields of `Permission`: the command line, the
application-wide setting, and the account. They are named in `allowed.rs`'s own module doc, not in
`CLAUDE.md` — `CLAUDE.md` does not contain the phrase. Widening `Allowed` to cover reads touches
nine sites, seven of which the compiler catches and two of which it does not. The two it does not
are the trap: `#[derive(Deserialize)]` on `Allowed` has no field-level default, so an existing
`app_config.json` stops parsing altogether and takes every other setting with it, and
`Allowed::NOTHING` is what `--read-only` and the first-run screen's "Read my mail, change nothing"
both resolve to, so a `reading` field that is `false` in `NOTHING` turns reading off for the one
choice whose label promises reading.

**Question 2.** There is no repeating-row editor in this codebase and there should not be one. The
closest thing is the Filter Manager, `run_manager_loop` at `wx_managers.rs:203`, which is already
generic over the row type: a native `ListCtrl` in report mode plus Add/Edit/Delete/Close buttons,
with each row edited in its own small dialog. That shape gives the set size and position-in-set from
the system's own provider on both accessibility channels, never changes the dialog's tab order when
a row is added or removed, has no drag interaction, and needs no new control. Extend it. Separately:
the existing Add/Edit Filter Rule dialog hardcodes 6 of the 11 fields and 6 of the 11 match types
as its own second vocabulary, which is a live defect this phase must fix rather than copy.

**Question 3.** The body is already reachable, in both search paths, and `02-CONTEXT.md`'s D-2-09 is
wrong about this. The FTS5 index has a `body` column that is populated whenever a body is stored,
and the saved-search scan already has a `TheMessageText::Read` mode that joins `message_bodies` and
unpacks it. What is missing is the disclosure, the gate, the fetch, and one editor that can name the
field. The counting question needs no new index, and eviction does not reindex, which means the
index goes stale rather than the coverage narrowing — the opposite of what SEARCH-02 assumes.

**Primary recommendation:** Order the work as D-2-09 says but for a corrected reason. Fix the
`Allowed` serde and `NOTHING` traps first, in their own commits, because they are the only part of
this phase that can corrupt an existing user's settings. Then widen the rule vocabulary in the
editor. Then the disclosure. The fetch last, because it is the only part that has never run.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

- **D-2-05:** Saved searches **mirror the account structure**, exactly as
  Favourites does under D-29: one group with account sub-branches inside it.
  They are already account-scoped in the data. `saved_searches` carries an
  `account_id`, the read is `WHERE account_id = ?1`, and `run_a_saved_search`
  takes the active account. Only the tree placement was global, which was
  invisible while one account showed at a time and is not now.
  — **Reversibility:** reversible — a tree-shape change with the data already
  scoped correctly underneath it.

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

- **D-2-09:** **SEARCH-02's problem does not exist until SEARCH-03 creates it.**
  The live search reads `m.subject`, `m.from_addr`, `m.to_addr` and `m.snippet`
  and never touches the bodies table, so no search today can need body text. The
  rule editor is what makes the body field reachable, and therefore what makes
  the coverage disclosure necessary. Order the work accordingly: the vocabulary
  widening comes before or with the disclosure, never after it.

### Claude's Discretion

- The wording of the coverage sentence, subject to it giving both numbers.
- Whether the rule editor is a dialog or a page, and where it is reached from.
- How the read dimension is named in `Allowed`, subject to the doc comment being
  rewritten rather than left describing writes only.

### Deferred Ideas (OUT OF SCOPE)

- **Widening `Allowed` is the largest thing here and it is not about search.** If
  it turns out to ripple further than the three places, it is a candidate for its
  own phase rather than something to absorb quietly. Say so rather than growing
  this one.
- Nothing else surfaced. The discussion stayed inside the phase.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEARCH-01 | A saved search keeps the whole scope it was saved with, not half of it. | Q1 of §Where the work lands. `what_a_typed_search_asks` at `saved_searches.rs:349` is one function and is the only production writer of a question set; `WhereToSearch` at `searching.rs:55` is the four scopes the In box offers. D-2-03's "no absent value to interpret" is confirmed: `SavedSearch` has no scope field to be absent. |
| SEARCH-02 | Save and run a search over message text that eviction has cleared. | §Question 3 in full. The half that exists (`TheMessageText::Read`), the half that does not (disclosure, gate, fetch), the counting query, and the eviction-does-not-reindex finding that changes what "silently narrowing" means. |
| SEARCH-03 | Smart folders defined by a rule. | §Question 2 in full, plus the second-vocabulary defect in `build_filter_edit_dialog`. `Question::as_a_rule` and `FilterEngine::matches`'s eleven arms are read and quoted. |
</phase_requirements>

---

## Correction to D-2-09, stated first because everything downstream depends on it

D-2-09 says: "The live search reads `m.subject`, `m.from_addr`, `m.to_addr` and `m.snippet` and
never touches the bodies table, so no search today can need body text."

Those four are the **output columns** of the `SELECT`, not the search predicate. Both search paths
already reach message text.

**The live search box.** `search_messages` matches against an FTS5 virtual table that has a `body`
column, and that column is populated. Verbatim, `src/data/message_cache/mod.rs:2170-2175`
[VERIFIED: src/data/message_cache/mod.rs:2170-2175]:

```
"CREATE VIRTUAL TABLE IF NOT EXISTS message_search USING fts5(
     subject, from_addr, snippet, body,
     content='', contentless_delete=1,
     tokenize=\"unicode61 remove_diacritics 2\"
 );",
```

`WhereToSearch::EveryFolder`'s own doc comment says so, verbatim at `searching.rs:57-58`
[VERIFIED: src/data/message_cache/searching.rs:56-58]:

```
    /// Every folder of the account, across everything the index holds: the
    /// subject, the sender, the first line and the message text.
    EveryFolder,
```

**The saved-search path.** `run_a_saved_search` already chooses whether to read bodies. Verbatim,
`src/presentation/wx_app.rs:6324-6329` [VERIFIED: src/presentation/wx_app.rs:6324-6329]:

```rust
        let text = if search.reads_the_message_text() {
            TheMessageText::Read
        } else {
            TheMessageText::LeftAlone
        };
        let messages = match cache.messages_a_saved_search_reads(&account_id, folder_id, text) {
```

`reads_the_message_text` is at `saved_searches.rs:575-579`
[VERIFIED: src/application/saved_searches.rs:575-579]:

```rust
    pub fn reads_the_message_text(&self) -> bool {
        self.questions.iter().any(|question| {
            crate::application::filters::a_rule_reads_the_message_text(&question.field)
        })
    }
```

**What this changes for the plan.** D-2-09's ordering instruction ("the vocabulary widening comes
before or with the disclosure, never after it") is still right, but not for the reason given. The
body field is reachable today by a search saved by a newer version of the program, or written by
hand into the database, and `SavedSearchesRead` deliberately keeps such rows. What the rule editor
adds is the first *supported* way to write one. So the disclosure is already owed and is not
created by SEARCH-03; it is merely made unavoidable by it. This makes the disclosure independently
shippable, which is worth having, because it is the half with no gate and no untested code in it.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Narrower typed-search question set (D-2-03) | `application::saved_searches` | `presentation::wx_app` | `what_a_typed_search_asks` is a pure function over a scope; the In box's value is presentation's to supply. |
| Sentence describing what a search asks (D-2-04) | `application::saved_searches` | — | Every other sentence this module owns lives here (`a_typed_search_in_words`, `what_a_search_found`, `a_row_for`). A second sentence builder in the window layer is the drift `EXPERIMENTAL_WARNING`'s comment records. |
| Rule editor controls | `presentation::wx_managers` | `application::filters` | The vocabulary lists live in `filters`; the controls that offer them live where the Filter Manager already is. |
| Account sub-branches for saved searches (D-2-05) | `presentation::folder_tree` | `presentation::wx_app` | `favourite_rows` is already there and is the precedent; the per-account read is the caller's. |
| Read dimension on `Allowed` (D-2-06/07) | `application::allowed` | `data::config`, `presentation::wx_settings`, `presentation::command_line`, `presentation::first_run` | The type owns the meaning; the four others each produce or offer one of the three answers. |
| Coverage count (D-2-08) | `data::message_cache` | `application::saved_searches` | One query. The sentence built from its two numbers belongs beside the other sentences in `application`. |
| Body fetch (D-2-08) | `service::protocols::imap` | `application::mail_sync` | Where every other server read is, and where the gate has to be asked. |

---

## Question 1 — The three places, and what widening `Allowed` costs

### The three places, named

They are the three fields of `Permission`, and `allowed.rs`'s own module doc heading names them.
Verbatim, `src/application/allowed.rs:12-15` [VERIFIED: src/application/allowed.rs:11-17]:

```
//! # Three places can say no, and any one of them is enough
//!
//! The command line, the application's own setting, and the account. A change
//! goes out only when all three allow it.
```

And the struct, verbatim at `src/application/allowed.rs:180-188`
[VERIFIED: src/application/allowed.rs:180-188]:

```rust
pub struct Permission {
    /// What the command line said, which can only ever narrow the others.
    pub command_line: Allowed,
    /// What this installation is set to.
    pub setting: Allowed,
    /// What this account is set to.
    pub account: Allowed,
}
```

**`CLAUDE.md` does not contain the phrase "three places that must agree", nor the word `allowed` in
that sense.** `grep -in 'allowed|three places' CLAUDE.md` returns two lines, 385 and 442, and both
are about `application::allowed` being where experimental warnings live. The phrase in the phase
brief comes from `allowed.rs` itself and from `02-CONTEXT.md` D-2-06.
[VERIFIED: CLAUDE.md — grep returned only lines 385, 442]

Each of the three has exactly one production producer:

| Place | Producer | Site |
|-------|----------|------|
| Command line | `--read-only`, and `--allow nothing\|tasks\|everything` | `src/presentation/command_line.rs:164` and `:230-237` |
| Setting | `AppConfig::allowed_changes`, written by the settings screen | `src/data/config.rs:257`; written at `src/presentation/wx_settings.rs:2089-2092` |
| Account | `AppConfig::allowed_per_account`, **written by nothing** | `src/data/config.rs:276`; recorded in `STORED_AND_OFFERED_BY_NOTHING` at `src/data/config.rs:1670` |

The third being unwritten is already recorded as a known defect, and the record is itself checked —
`test_a_setting_recorded_as_offered_by_nothing_is_still_offered_by_nothing` at
`src/data/config.rs:1747`. That matters here: **if this phase adds a per-account read control, that
entry has to be deleted in the same commit or the test fails.** It is the good direction of failure
and the plan should expect it rather than be surprised by it.

### The nine sites a new field touches

Seven the compiler catches, two it does not.

**Compiler catches these** (struct literal or exhaustive const):

1. `Allowed::NOTHING` — `src/application/allowed.rs:53-56`
2. `Allowed::EVERYTHING` — `src/application/allowed.rs:60-63`
3. `Allowed::FOR_TESTING` — `src/application/allowed.rs:71-74`
4. `Allowed::and` — `src/application/allowed.rs:82-87`
5. The settings screen's read-back — `src/presentation/wx_settings.rs:2089-2092`, verbatim
   [VERIFIED: src/presentation/wx_settings.rs:2089-2092]:
   ```rust
       cfg.allowed_changes = crate::application::allowed::Allowed {
           mail: w.allow_mail.get_value(),
           personal_information: w.allow_pim.get_value(),
       };
   ```
6. `src/application/pop_sync.rs:758` and `:838` — two `Allowed { ... }` literals in tests.
7. The unit tests in `allowed.rs`'s own `mod tests` that build literals
   (`test_either_half_on_its_own_counts_as_something_being_allowed`,
   `test_an_alpha_tester_can_change_their_tasks_and_not_their_mail`,
   `test_combining_is_order_independent`).

**The compiler does not catch these two, and they are the phase's real risk.**

#### Trap A — the serde default, and it is worse than "the wrong answer"

`Allowed` derives `Deserialize` with no field-level attributes at all. Verbatim,
`src/application/allowed.rs:36-48` [VERIFIED: src/application/allowed.rs:36-48]:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Allowed {
    /// Sending a message, and changing or deleting one on the server.
    ///
    /// The one that cannot be undone. A message that has gone has gone, and a
    /// message deleted from a server may be the only copy.
    pub mail: bool,
    /// Tasks, contacts and calendar events at a provider.
    ///
    /// Recoverable by hand, mostly, and this is the least proven code in the
    /// application: none of the three sync paths has met a live account.
    pub personal_information: bool,
}
```

The `#[serde(default = "default_allowed")]` on `AppConfig::allowed_changes`
(`src/data/config.rs:256-257`) protects only the case where the whole `allowed_changes` **object** is
absent. Every existing installation has that object present, with two keys inside it. Adding a
third field with no field-level default makes `serde` report a missing field, which fails the parse
of the **entire `AppConfig`**, not just this struct. Verbatim, `src/data/config.rs:747-748`
[VERIFIED: src/data/config.rs:741-748]:

```rust
            self.app_config = serde_json::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse app config: {}", e)))?;
```

That error then propagates into `allowed_for`, which converts it to the safest possible answer —
verbatim, `src/application/allowed.rs` in `allowed_for`
[VERIFIED: src/application/allowed.rs, `allowed_for` body]:

```rust
    let stored = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().allowed_for(account_id))
        .unwrap_or(Allowed::NOTHING);
```

So the failure mode of getting this wrong is: **every setting the user has ever chosen stops
loading, and everything silently reverts to whatever `AppConfig::default()` gives, with `Allowed`
answering `NOTHING` for permissions.** `config.rs`'s own comment already names this shape, verbatim
at `src/data/config.rs:1527-1528`
[VERIFIED: src/data/config.rs:1526-1529]:

```
        // Every settings file on every machine was written before this, and
        // one that refuses to parse takes every other setting with it.
```

**How to avoid it, precisely.** Two attributes, not one:

```rust
    /// Reading a message's text back from the provider.
    ///
    /// On unless somebody says otherwise, which is the one field where
    /// `Default` is not `false`. ... (D-2-07's reason goes here)
    #[serde(default = "reading_is_allowed")]
    pub reading: bool,
```

with a named function returning `true`, following the house rule that a default is *named* rather
than restated — `default_allowed`'s doc comment says why, verbatim
[VERIFIED: src/data/config.rs:432-448]:

```
/// `Allowed::FOR_TESTING` holds the answer and the reason for it. This names it
/// rather than saying it a second time, and that is the whole point: a sentence
/// repeating a fact the code already holds goes stale on its own
```

`#[serde(default)]` alone gives `false` for a `bool`, which is the wrong answer for every existing
user and is exactly the trap the phase brief names. `#[serde(default = "reading_is_allowed")]` is
the answer, and the RED test for it is the shape `config.rs` already has twice — take a real
serialised `Allowed`, remove the key from the JSON object, parse it back, assert the field is
`true`. `test_asking_to_start_in_all_inboxes_survives_being_written_and_read_back`
(`src/data/config.rs:994-1022`) is the template, and the `.remove(...).expect(...)` line in it is
the part that makes the before-and-after real rather than hypothetical.

Note also `allowed_per_account: HashMap<String, Allowed>` — the same nested `Allowed` appears as map
values there, so the same attribute covers both. No second fix needed.

#### Trap B — `NOTHING` is what "read my mail, change nothing" resolves to

This one has no compiler help and no existing test to lean on. `--read-only` maps to
`Allowed::NOTHING`, verbatim `src/presentation/command_line.rs:164`
[VERIFIED: src/presentation/command_line.rs:164]:

```rust
            "--read-only" => run.allowed = Allowed::NOTHING,
```

and so does the first-run screen's first choice, verbatim `src/presentation/first_run.rs:60-62`
[VERIFIED: src/presentation/first_run.rs:59-63]:

```rust
        match self {
            Choice::ReadOnly => Allowed::NOTHING,
            Choice::TasksAndContacts => Allowed::FOR_TESTING,
            Choice::Everything => Allowed::EVERYTHING,
        }
```

whose label and explanation are, verbatim `src/presentation/first_run.rs:76` and `:85-88`
[VERIFIED: src/presentation/first_run.rs:74-88]:

```
            Choice::ReadOnly => "Read my mail, change nothing",
...
                "Nothing you do here reaches your provider. Safe to point at \
                 your real mail. You will not be able to send, move, delete, or \
                 sync changes to your tasks."
```

If `NOTHING` sets `reading: false`, then the choice whose label is "Read my mail" turns reading off,
and `--read-only` — a flag whose entire meaning is "reads only" — stops reads. Both would be silent.
`--allow nothing` and `--allow none` map to the same const
(`src/presentation/command_line.rs:233`) and inherit the same bug.

**So `NOTHING` must keep `reading: true`.** That is not a special case bolted on; it is what D-2-07
already decided, said in the type: `Default` stops meaning "changes nothing" and starts meaning "the
safe end of each", and for a read the safe end is on.

The consequence for the constants:

| Const | `mail` | `personal_information` | `reading` |
|-------|--------|------------------------|-----------|
| `NOTHING` | false | false | **true** |
| `FOR_TESTING` | false | true | true |
| `EVERYTHING` | true | true | true |
| `Default` | false | false | **true** |

`#[derive(Default)]` cannot produce that, so `Default` must be hand-written as `Self::NOTHING`.
The existing test survives unchanged, verbatim `src/application/allowed.rs:200-201`
[VERIFIED: src/application/allowed.rs:196-203]:

```rust
        assert_eq!(Allowed::default(), Allowed::NOTHING);
        assert!(!Allowed::default().anything());
```

`anything()` must stay writes-only for that second assertion to hold, and it should anyway —
verbatim its doc, `src/application/allowed.rs:90` [VERIFIED: src/application/allowed.rs:89-93]:

```rust
    /// Whether anything at all may be changed.
    pub const fn anything(self) -> bool {
        self.mail || self.personal_information
    }
```

Reading is not a change. Leave the body alone; the doc comment is already correct for it.

`and()` still takes the narrowest on each half independently, so `reading` uses `&&` like the other
two. With `NOTHING` holding `reading: true`, `and()` can only ever turn reading off if some place
deliberately says so, which is the property wanted.

### What the type and its comment should become

Three things are now wrong in the prose and each has to be rewritten, not patched.

1. **The struct's own doc**, currently `src/application/allowed.rs:31-35`
   [VERIFIED: src/application/allowed.rs:31-35]:
   ```
   /// What may be changed at a provider.
   ///
   /// Two answers rather than one boolean, because the two cost different amounts
   /// to get wrong. `Default` is the safe end of both, so anything constructed
   /// without a decision changes nothing.
   ```
   Both sentences become false. "What may be changed" excludes a read, "two answers" becomes three,
   and "anything constructed without a decision changes nothing" is still true but is no longer the
   whole of what `Default` means. Replace with a summary sentence covering both directions
   (something in the shape of "What Wixen Mail may do at somebody's provider: two things it may
   change, and one it may read") plus D-2-07's exception stated where a reader meets the field.

2. **The module doc's opening**, `src/application/allowed.rs:1-9`, which says "Reading mail into a
   local cache cannot hurt anybody" and then treats the whole file as being about writes. That
   sentence is still true and is now load-bearing: it is the argument for why the read dimension
   defaults on. Keep it, and add the sentence that turns it into a rule rather than an aside.

3. **The `# Split by what it costs to get wrong` heading**, `src/application/allowed.rs:24-29`,
   which ends "So they are two answers rather than one." Same edit.

`SETTINGS_SECTION` is `"Allow Changes"` (`src/application/allowed.rs:99`). A read is not a change,
so the read control does **not** belong under that heading, and putting it there would reintroduce
exactly the label-versus-sentence drift `SETTINGS_SECTION`'s own doc records. A second named
constant beside it, with its own section, is the right shape. Note the constraint that comes with
it: `changes_waiting_here` and `removals_waiting_here` both name `SETTINGS_SECTION` in their
sentences and both are asserted verbatim by tests at `src/application/allowed.rs:274` and `:296`.
Nothing about the read dimension may change those strings.

### What is *not* touched, checked rather than assumed

- **Every `may_i` call site.** All sixteen are writes. Fourteen in
  `src/service/protocols/imap.rs` (`:860, 894, 931, 963, 1195, 1219, 1247, 1306, 1319, 1351, 1381`
  plus the definition at `:1180`) and two in `src/service/protocols/pop3.rs` (`:284` definition,
  `:361` call). [VERIFIED: grep over src/, 16 hits]
  **`02-CONTEXT.md` names only `imap.rs`. `pop3.rs` has a `may_i` too** — same one-line body calling
  `outward::permitted` — and any structural guard written about `may_i` must see both files or it
  measures half of what it claims to.
- `may_i` takes a `bool` (`self.may_change`), not an `Allowed`, and routes through
  `crate::service::outward::permitted`. So the read gate is a **new** function, not a widening of
  `may_i`, and the two must not be conflated: `permitted`'s refusal sentences are written for
  writes. There is a guard on this — `src/service/outward.rs:1693` holds
  `const SENDS_A_MAIL_CHANGE: [&str; 2] = ["self.may_i(", "outward::permitted("];` and
  `src/application/sent_copy.rs:1112` checks for `"self.may_i("`. **A census that asserts a floor.**
  Per `CLAUDE.md`'s guard-record rule, adding a read gate near these means re-measuring every record
  that reads that census, in the same commit.
- `Permission::allowed()` needs no change; it delegates to `and()`.
- **The mirror guard is structurally blind to this.** `test_every_setting_somebody_can_change_is_offered_by_a_screen`
  (`src/data/config.rs:1704`) reads field names out of `pub struct AppConfig`, not out of `Allowed`.
  `allowed_changes` is already offered, so adding `reading` inside `Allowed` passes that guard
  whether or not any screen offers it. **The guard the phase inherits does not cover the thing the
  phase adds.** A new check is needed, or the control has to be verified by hand and said so.

---

## Question 2 — A rule editor in wxdragon 0.9.17

### What exists, and how close it is

There is no repeating-row editor anywhere in `src/presentation/`. What exists is better.

`run_manager_loop` at `src/presentation/wx_managers.rs:203-211` is already generic over the row
type. Verbatim [VERIFIED: src/presentation/wx_managers.rs:203-211]:

```rust
fn run_manager_loop<T: Clone + 'static>(
    chrome: ManagerChrome<'_>,
    kind: &str,
    working: &mut Vec<T>,
    populate: impl Fn(&ListCtrl, &[T]) + Copy + 'static,
    add_fn: impl Fn(&Dialog) -> Option<T>,
    edit_fn: impl Fn(&Dialog, &T) -> Option<T>,
    name_fn: impl Fn(&T) -> String + Copy + 'static,
) -> bool {
```

It builds four buttons — `&Add...`, `&Edit...`, `&Delete`, `&Close`
(`wx_managers.rs:221-236`) — attaches them to a `ListCtrl` supplied by `make_shell`, and carries an
`Arc<Accessibility>` for announcements. The Filter Manager uses it today with four columns,
verbatim `src/presentation/wx_managers.rs:2435-2438`
[VERIFIED: src/presentation/wx_managers.rs:2435-2438]:

```rust
    list.insert_column(0, "Name", ListColumnFormat::Left, 130);
    list.insert_column(1, "Condition", ListColumnFormat::Left, 220);
    list.insert_column(2, "Action", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 70);
```

### The control shape to build

**A list of conditions in a native `ListCtrl`, plus Add/Edit/Remove buttons, with each condition
edited in its own small dialog.** Not a stack of rows in a `ScrolledWindow`.

This is not a stylistic preference; each of the hard constraints in the brief falls out of it.

| Constraint | How the ListCtrl shape satisfies it |
|------------|-------------------------------------|
| Every control carries its own label on both channels | The list has one accessible name, set once. The per-condition controls live in the sub-dialog and there are a fixed three of them (Field, Match, Pattern) plus two checkboxes, each labelled once at build time. No control is created or destroyed while a screen reader is on it. |
| A row that can be added and removed changes the tab order | It does not. The tab order is `list → Add → Edit → Remove → Close`, constant at every list length. What changes is the list's item count, which is the control's own state. |
| The set size a screen reader reports | Comes from the system's own provider for a native list control, on both UIA and MSAA, without `set_accessible_name` being involved at all. This is the key reason to use the native control: `set_accessible_name` writes MSAA only, so anything hand-rolled would report a set size to NVDA and nothing to Narrator — the exact bug `tests/checkbox_labels.rs`'s module doc describes. |
| WCAG 2.2 target size 24x24 CSS px | Native `Button` and `ListCtrl` rows at Windows' own metrics. Nothing here sets an explicit size below the default. |
| No drag-only interaction (2.5.7) | There is none. Reordering, if it is wanted at all, is Alt+Shift+Up/Down — the one gesture D-31 already established for everything in the tree. Conditions joined by `All` or `Any` are order-independent anyway, so reordering is probably not wanted. |

**What to announce, and where the code has to put it.** `run_manager_loop` already announces after a
delete through `delete_selected` and the `status_text` control. `Accessibility` offers
`announce(&self, text, priority)` at `src/presentation/accessibility.rs:300`,
`announce_topic` at `:310` and `live_region_update` at `:375`. Adding or removing a condition should
say the new count in the same sentence as the change ("Condition removed. 2 conditions."), because a
count said separately is a second announcement to interrupt the first. Nothing here needs a screen
reader run to get right — what it needs is that the sentence exists and the list is native.

### The defect to fix rather than copy

`build_filter_edit_dialog` hardcodes its own vocabulary, and it is a **shorter** vocabulary than the
engine's. Verbatim `src/presentation/wx_managers.rs:2569-2570`
[VERIFIED: src/presentation/wx_managers.rs:2569-2572]:

```rust
    let field_choices: Vec<String> = ["subject", "from", "to", "cc", "body_plain", "date"]
        .iter()
        .map(|s| s.to_string())
        .collect();
```

and `src/presentation/wx_managers.rs:2583-2591`
[VERIFIED: src/presentation/wx_managers.rs:2583-2591]:

```rust
    let match_choices: Vec<String> = [
        "contains",
        "not_contains",
        "equals",
        "starts_with",
        "ends_with",
        "regex",
    ]
```

Against the engine's own lists, verbatim `src/application/filters.rs:61-73` and `:86-98`
[VERIFIED: src/application/filters.rs:61-73, 86-98]:

```rust
pub const A_FIELD_A_RULE_MAY_NAME: [&str; 11] = [
    "subject",
    "from",
    "to",
    "cc",
    "date",
    "message_id",
    "body_plain",
    "body_html",
    "read",
    "starred",
    "deleted",
];
```

```rust
pub const A_WAY_A_RULE_MAY_MATCH: [&str; 11] = [
    "contains",
    "not_contains",
    "equals",
    "not_equals",
    "starts_with",
    "ends_with",
    "is_empty",
    "is_not_empty",
    "is_true",
    "is_false",
    "regex",
];
```

Six of eleven, twice. Missing fields: `message_id`, `body_html`, `read`, `starred`, `deleted`.
Missing match types: `not_equals`, `is_empty`, `is_not_empty`, `is_true`, `is_false`.

This is the second vocabulary `as_a_rule`'s comment exists to prevent, sitting in the presentation
layer with nothing checking it. The fix is one line each — build both `Vec<String>` from the
constants — and the guard is a test that the offered lists and the constants have the same length
and members. That guard has to be written from the constants in both directions, the way
`filters.rs`'s own tests already do it for the engine ("everything on the list is really handled,
and nothing handled is missing from it", `filters.rs:84-85`), or it goes stale the moment a twelfth
field is added.

**Do this fix in its own commit before the editor.** It closes SEARCH-03's "reaching all eleven
fields" criterion for the filter path as a side effect, and it makes the rule editor a
`build_filter_edit_dialog` variant rather than a new dialog with a new list to keep in step.

### Three things the eleven fields expose that the editor must handle

1. **`read`, `starred` and `deleted` are booleans rendered as strings.** Verbatim
   `src/application/filters.rs:162-164` and `:187-189`
   [VERIFIED: src/application/filters.rs:161-191]:
   ```rust
        fn bool_to_str(value: bool) -> &'static str {
            if value { "true" } else { "false" }
        }
   ```
   ```rust
            "read" => Some(Some(bool_to_str(message.read))),
            "starred" => Some(Some(bool_to_str(message.starred))),
            "deleted" => Some(Some(bool_to_str(message.deleted))),
   ```
   So `read is_true` works and `read contains ada` is legal-but-meaningless. The Pattern box should
   disable or hide itself for `is_true`, `is_false`, `is_empty` and `is_not_empty`, and the sentence
   D-2-04 builds must not read out an empty pattern.

2. **`deleted` can never match from a saved search.** The scan hardcodes `m.deleted = 0`, verbatim
   `src/data/message_cache/saved_searches.rs:363`
   [VERIFIED: src/data/message_cache/saved_searches.rs:357-365]:
   ```
        "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr,
                m.cc, m.date, {columns}, m.read, m.starred, m.deleted
         FROM messages m
         INNER JOIN folders f ON m.folder_id = f.id
         {joined}
         WHERE f.account_id = ?1 AND m.deleted = 0 {narrowed}
         ORDER BY m.date DESC, m.uid DESC"
   ```
   and the reason is stated at `saved_searches.rs:222-224`: "Turning it up here would make a saved
   search the one place in the program that shows somebody the mail they have thrown away." So
   `deleted is_true` is a search that can only ever find nothing. Offering it in the editor with no
   word said is precisely SEARCH-02's failure shape in a different costume. Either leave `deleted`
   out of the editor's field list with the reason written beside it, or say it in the dialog.

3. **An absent field reads as empty, and that is load-bearing.** Verbatim
   `src/application/filters.rs:195-202` [VERIFIED: src/application/filters.rs:195-202]:
   ```rust
        // An absent field is an empty one, not a reason to stop.
        //
        // Leaving early here meant "cc is empty" was false for every message
        // that had no cc, which is the only case the rule was written for, and
        // "body is empty" could never fire on a message whose body had not
        // been downloaded.
        let target_text = present.unwrap_or("");
   ```
   Combined with `TheMessageText::LeftAlone` passing `body_plain: None`, this means
   **`body_plain is_empty` is true of every message if the body columns are not read.** The existing
   guard is `reads_the_message_text()`, which is asked before the scan and covers this. Anything the
   rule editor adds must go through `run_over`, never `selects` directly — `selects`'s own doc says
   why (`saved_searches.rs:641-643`: "nothing outside this module should be filtering with this
   directly").

### Testability: the pattern the guard tests need

Every dialog test in `tests/` uses the same split: a `build_*_dialog` function that constructs
everything and returns the widgets, and a separate `show_*` that calls `.show_modal()`. The reason is
in `build_filter_edit_dialog`'s own doc, verbatim `src/presentation/wx_managers.rs:2531-2537`
[VERIFIED: src/presentation/wx_managers.rs:2531-2537]:

```
/// Everything `show_filter_edit` used to do up to its own `.show_modal()`
/// call, split out the same way
/// [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog and read back the real colour
/// a live control holds, and never call `.show_modal()` at all.
```

with the second constraint at `tests/checkbox_labels.rs:21-23`
[VERIFIED: tests/checkbox_labels.rs:21-23]:

```
//! One `#[test]` function building real dialogs, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per process
//! and `cargo test` runs each file under `tests/` as its own process.
```

So the rule editor gets a `build_rule_edit_dialog` and its guard lives in a **new file under
`tests/`**, one `#[test]` function, building the real dialogs. It cannot be added to
`tests/checkbox_labels.rs` unless that file's single test is widened, and widening it changes what a
recorded guard reddens.

**`tests/checkbox_labels.rs` does not cover this today.** It iterates `ItemKind::Event`, `Task`,
`Reminder`, `Note`, `Contact` through `build_item_form_dialog` and reads `widgets.tick_fields`
(`tests/checkbox_labels.rs:56-94`). It never touches `wx_settings.rs` or `wx_managers.rs`. The
"Case Sensitive" and "Enabled" checkboxes in `build_filter_edit_dialog` are unchecked by it right
now, and any checkbox the rule editor adds would be unchecked too. The phase brief's premise that
"`tests/checkbox_labels.rs` guards it" is true of the item form only. Say so in the plan and decide
deliberately: either widen that file, or write a companion for the manager dialogs.

Note that `build_filter_edit_dialog` follows the settings screen's pattern in one respect and not
another. It calls `set_accessible_name` on the three `Choice` controls (`wx_managers.rs:2574, 2596,
2618`) but the two `CheckBox` controls carry `.with_label("&Case Sensitive")` and no
`set_accessible_name` — which for a checkbox is right, because the label is on the control and both
channels can see it. The wrong pattern is the one `checkbox_labels.rs` exists to catch: an empty
label plus `set_accessible_name`. The rule editor should do what the filter dialog does.

### The three-deep dialog question

`show_filter_manager_dialog(parent: &Frame, ...)` opens a `Dialog`, which opens
`build_filter_edit_dialog(parent: &Dialog, ...)`. Two deep is established. A rule editor reached
from a saved-search manager, with each condition edited in a sub-dialog, is **three** deep, which
this codebase has not done. Two ways out, both within Claude's discretion under D-2-06's list:

- Reach the rule editor from the folder tree's context menu on a saved-search row, so it is
  `Frame → rule editor dialog → condition dialog`. Two deep, same as the filter manager.
- Make the rule editor a page rather than a dialog.

The first is smaller and reuses `wx_context_menu.rs`, which already exists. Prefer it unless the
plan finds a reason not to.

---

## Question 3 — Reaching message text

### What a body-reaching query costs, and what index it needs

**Nothing new.** The query already exists and every index it wants is present.

`scan_query`, verbatim `src/data/message_cache/saved_searches.rs:341-365`
[VERIFIED: src/data/message_cache/saved_searches.rs:341-365]:

```rust
fn scan_query(one_folder: bool, text: TheMessageText) -> String {
    let (columns, joined) = match text {
        TheMessageText::LeftAlone => ("NULL, NULL, NULL, NULL", ""),
        TheMessageText::Read => (
            "b.body_plain, b.body_html, b.body_plain_packed, b.body_html_packed",
            "LEFT JOIN message_bodies b ON b.message_id = m.id",
        ),
    };
```

The three access paths it needs:

| What the query asks | Index that serves it | Where |
|---------------------|----------------------|-------|
| `f.account_id = ?1` | `UNIQUE(account_id, path)` on `folders`, which SQLite implements as `sqlite_autoindex_folders_1` | `src/data/message_cache/mod.rs:1374` |
| `m.folder_id = f.id` | `idx_messages_folder_id ON messages(folder_id)` | `src/data/message_cache/mod.rs:2616` |
| `ORDER BY m.date DESC, m.uid DESC` | `idx_messages_date ON messages(date DESC, uid DESC)` | `src/data/message_cache/mod.rs:2669` |
| `b.message_id = m.id` | `message_bodies.message_id INTEGER PRIMARY KEY` — a rowid alias, so the join is a rowid seek and costs nothing | `src/data/message_cache/mod.rs:2046` |

Verbatim, the two index statements
[VERIFIED: src/data/message_cache/mod.rs:2616, 2669]:

```
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)",
```
```
            "CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC, uid DESC)",
```

and the bodies table, verbatim `src/data/message_cache/mod.rs:2045-2051`
[VERIFIED: src/data/message_cache/mod.rs:2045-2051]:

```
                "CREATE TABLE IF NOT EXISTS message_bodies (
                message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
                body_plain TEXT,
                body_html TEXT,
                bytes INTEGER NOT NULL DEFAULT 0,
                last_read_at TEXT NOT NULL
            )",
```

**The cost is not the index, it is the inflate.** The scan is deliberately unbounded, verbatim
`src/data/message_cache/saved_searches.rs:226-229`
[VERIFIED: src/data/message_cache/saved_searches.rs:226-229]:

```
    /// Not bounded. A search that read the newest page only would answer a
    /// narrower question than the one asked and say nothing about it, which is
    /// the failure that never gets reported. What is bounded is the list of
    /// results, by the caller, which says so out loud.
```

Bodies are stored zlib-deflated at level 6 (`src/data/message_cache/bodies.rs:40`:
`const PACKING_EFFORT: flate2::Compression = flate2::Compression::new(6);`) and the budget is
verbatim `src/data/message_cache/bodies.rs:192`
[VERIFIED: src/data/message_cache/bodies.rs:192]:

```rust
pub const BODY_CACHE_BUDGET_BYTES: i64 = 512 * 1024 * 1024;
```

So a body-reaching saved search inflates up to 512 MiB of deflate on the worst-case run. It already
runs on a worker — `rt.spawn_blocking` at `src/presentation/wx_app.rs:6291` — so the interface
thread is safe. But 512 MiB of inflate is seconds, not milliseconds, and the progress line says
only `"Running this saved search..."` (`src/presentation/wx_app.rs:6284`). **A search that reads
bodies should say so in that line, and the coverage sentence is where the numbers to say it come
from.** That is a free improvement the disclosure work makes possible.

**The measurement the plan owes.** Every index comment in this file carries a measured before and
after — "19.34 ms against 0.11 ms at twenty-five thousand messages", "279 ms against 0.15 ms",
"eighty-one seconds, and eighty-one milliseconds with this index". House style. If the plan proposes
any index, the record must be a real measurement, not an estimate. If it proposes none — which is
the recommendation — the plan should still measure the body-reading scan once at a realistic size
so the coverage sentence can be honest about what it is about to cost.

### The counting question D-2-08 depends on

**One query, no new index.** It is `scan_query(false, Read)` with the body columns and the sort
removed:

```sql
SELECT COUNT(*), COUNT(b.message_id)
  FROM messages m
 INNER JOIN folders f ON m.folder_id = f.id
  LEFT JOIN message_bodies b ON b.message_id = m.id
 WHERE f.account_id = ?1 AND m.deleted = 0
```

Two numbers from one pass. `COUNT(*)` is every message in the account, `COUNT(b.message_id)` counts
only the rows where the left join found a body, and the difference is "how many do not".

Why this is cheap enough:

- It reads no body bytes and inflates nothing, so it is strictly cheaper than the search it
  discloses — which is the only property that matters for a disclosure shown before the search runs.
- The `message_bodies` side is a rowid probe per candidate, not a scan.
- It visits one row per message in the account. At two hundred thousand messages that is a row
  visit, not a page read, for each.

**Does it need an index nothing has?** No. It would go faster with one:

```sql
CREATE INDEX IF NOT EXISTS idx_messages_folder_live
    ON messages(folder_id) WHERE deleted = 0
```

A partial index, in the shape `idx_calendar_events_repeating` and `idx_attachments_content_digest`
already use in this file. It would make the `COUNT(*)` half index-only per folder. **Do not add it
without a measurement.** The house rule is explicit and the existing comments all obey it, and an
index added on reasoning alone is the "fewer rows is not faster" mistake in a new place.

Two things the plan must not do:

- **`SELECT count(*) FROM message_bodies` is not the answer.** It is not account-scoped, and the
  disclosure is per account (D-2-08: "how many messages in this account"). At two accounts it would
  overstate coverage for both.
- **`folders.total_count` is not the answer either.** The column exists
  (`src/data/message_cache/mod.rs:1372`: `total_count INTEGER DEFAULT 0,`) but nothing read this
  session establishes that it is maintained, and a cached count used for a sentence about honesty is
  the wrong place to trust one. Count the rows. [ASSUMED — the maintenance of `total_count` was not
  traced; if the plan wants to use it, that is a thing to verify first.]

### The finding that changes what SEARCH-02 means: eviction does not reindex

Storing a body reindexes it into the FTS table. Verbatim `src/data/message_cache/bodies.rs:310-314`
[VERIFIED: src/data/message_cache/bodies.rs:310-314]:

```rust
        // The body is what somebody actually searches for, and this is the
        // only moment it is in hand as text: it goes into the row packed, and
        // the index cannot unpack it. Reindexed rather than added to, because
        // a contentless index has no way to update one column on its own.
        self.index_message_for_search(message_id)?;
```

Evicting one does not. The whole eviction loop, verbatim
`src/data/message_cache/bodies.rs:436-448` [VERIFIED: src/data/message_cache/bodies.rs:436-448]:

```rust
        for (message_id, bytes) in candidates {
            if total <= budget_bytes {
                break;
            }
            self.conn
                .execute(
                    "DELETE FROM message_bodies WHERE message_id = ?1",
                    rusqlite::params![message_id],
                )
                .map_err(|e| Error::Other(format!("Failed to evict message body: {}", e)))?;
            total -= bytes;
            freed += bytes;
        }
```

No `index_message_for_search` call. So after eviction:

- The **FTS index still holds the words of the evicted body.** The live search box goes on finding
  the message by a word that is only in its text.
- The **saved-search scan does not**, because it reads `message_bodies` directly and the row is gone.
  `body_plain` comes back `None`, which `filters.rs` reads as `""`.

**These two now disagree about the same message.** That is the "two answers to one question" shape:
a lenient reader (FTS, which kept the words) and a strict one (the scan, which lost them). It is
also, exactly, SEARCH-02's second criterion — "A saved search of that kind reruns without silently
narrowing as more bodies are evicted" — and the narrowing is real for the saved-search path and
absent for the live one.

The plan has to pick a side, and it is a real decision rather than a bug fix:

| Option | What it costs | What it gives |
|--------|---------------|---------------|
| Reindex on eviction (`index_message_for_search` inside the loop) | An FTS write per eviction, in a loop that already runs at the end of every folder sync on a worker | The two agree. The live search narrows with eviction too, which is honest but *loses* the user a search that works today. |
| Leave it, and let the coverage sentence carry the difference | Nothing | The live search stays wide. The saved-search path narrows. Two behaviours to describe. |
| Make the saved-search body scan read the FTS index rather than `message_bodies` | Not possible — `content=''` means the FTS table stores no retrievable content; you cannot select `body` back out of it | — |

**Recommendation: leave it and disclose it, and write the disagreement down where both readers can
see it.** The live search finding an evicted message by a body word is not wrong — it is the index
answering a question it can still answer — and taking it away to make two paths agree makes the
product worse. But the plan must not describe it as "the search covers N%" when two searches cover
different amounts. Two sentences, or one sentence naming which search it is about.

This also means the D-2-08 disclosure is about **the saved-search path specifically**, and the
sentence should say so, or somebody will read it as a claim about the search box.

### One more thing about the FTS index the plan needs

`content=''` with `contentless_delete=1` means the index stores no retrievable content. Any plan
step that tries `SELECT body FROM message_search` will fail. The count must come from
`message_bodies`, as above. `build_any_missing_search_index` (`searching.rs:336`) compares
`count(*)` of the two tables and does nothing when they are level — verbatim its doc,
`src/data/message_cache/searching.rs:329-333`
[VERIFIED: src/data/message_cache/searching.rs:328-334]:

```
    /// Returns how many were added. Runs on open, and does nothing at all once
    /// the index is level with the messages table, which is the ordinary case:
    /// the count is two integers out of SQLite's own bookkeeping rather than a
    /// pass over anybody's mail.
```

So it will not rebuild bodies into the index for existing databases — the counts are already level,
and the body column of every row indexed before its body was fetched is empty. That is another
reason the coverage sentence has to count `message_bodies` rather than reason about the index.

---

## The smaller question — `Join::All` and `Join::Any`

**The matcher honours both. Nothing outside tests has ever written `Join::All`. A joined-by-all
search has never run.**

The matcher, verbatim `src/application/saved_searches.rs:651-656`
[VERIFIED: src/application/saved_searches.rs:645-657]:

```rust
        let mut answered = self
            .questions
            .iter()
            .map(|question| FilterEngine::matches(&question.as_a_rule(), message));
        match self.join {
            Join::All => answered.all(|yes| yes),
            Join::Any => answered.any(|yes| yes),
        }
```

Both arms exist and both are exercised by unit tests (`saved_searches.rs:769` and `:1064` loop over
`[Join::All, Join::Any]`).

The writer side: `grep -rn 'SavedSearch {' src/` gives eight hits, of which exactly one is outside a
test module — `src/presentation/wx_app.rs:6539`. Its join field, verbatim
`src/presentation/wx_app.rs:6542` [VERIFIED: src/presentation/wx_app.rs:6539-6542]:

```rust
    let search = SavedSearch {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        join: WHAT_A_TYPED_SEARCH_JOINS_WITH,
```

and that constant, verbatim `src/application/saved_searches.rs:347`
[VERIFIED: src/application/saved_searches.rs:341-347]:

```rust
pub const WHAT_A_TYPED_SEARCH_JOINS_WITH: Join = Join::Any;
```

The other `Join::All` at `src/presentation/wx_app.rs:20070` is inside a test module — the last
`#[cfg(test)]` before it is at line 18954. [VERIFIED: awk scan of src/presentation/wx_app.rs]

The reader will read `"all"` back correctly if it is ever stored (`Join::read` at
`saved_searches.rs:145-152`, and the reader uses it at
`src/data/message_cache/saved_searches.rs:442`), and a word neither knows produces
`Found::CouldNotRun` rather than a guess — which is the right behaviour and is already tested
(`saved_searches.rs:1075`).

**So D-2-01 makes `Join::All` reachable for the first time, and its `.all()` branch has never
executed outside a unit test.** Two consequences for the plan:

1. The `Join::All` path deserves its own end-to-end test through `run_over`, not just the unit test
   over `selects`. `run_over` is where `what_it_cannot_read` is asked first, and the two have never
   been exercised together with `All`.
2. `selects`'s own doc records the empty-question-list trap that `All` creates, verbatim
   `src/application/saved_searches.rs:636-639`
   [VERIFIED: src/application/saved_searches.rs:634-640]:
   ```
       /// A search with no question in it takes nothing. A list of conditions
       /// that all have to match is true of every message when the list is empty,
       /// which would turn a row somebody opened expecting a handful of messages
       /// into the whole mailbox. Nothing was asked, so nothing is the answer.
   ```
   The rule editor makes an empty condition list reachable for the first time too — a user can
   delete every row. The guard exists in `selects`; the editor should refuse to save an empty rule
   rather than rely on it, the way `build_filter_edit_dialog` refuses an empty name
   (`wx_managers.rs:2669-2673`, `a_sub_dialog_needs(&d, "A name is needed before this can be saved.")`).

---

## Where the rest of the work lands

### D-2-03 — one question instead of three

`what_a_typed_search_asks`, verbatim `src/application/saved_searches.rs:340-360`
[VERIFIED: src/application/saved_searches.rs:334-360]:

```rust
pub const WHAT_A_TYPED_SEARCH_LOOKS_AT: [&str; 3] = ["subject", "from", "to"];
```
```rust
pub fn what_a_typed_search_asks(text: &str) -> Vec<Question> {
    WHAT_A_TYPED_SEARCH_LOOKS_AT
        .iter()
        .map(|part| Question {
            field: (*part).to_string(),
            match_type: "contains".to_string(),
            pattern: text.to_string(),
            case_sensitive: false,
        })
        .collect()
}
```

One function, one caller (`wx_app.rs:6543`). D-2-03 is right that no schema change is needed.

The mapping the plan needs, from `WhereToSearch` to a field list:

| `WhereToSearch` | Fields written | Note |
|-----------------|----------------|------|
| `SubjectOnly` | `["subject"]` | |
| `SenderOnly` | `["from"]` | |
| `EveryFolder` | `["subject", "from", "to"]` | Unchanged, so an older saved search is indistinguishable — which is D-2-03's whole argument |
| `OneFolder(id)` | `["subject", "from", "to"]` | **This one is not a field restriction and the plan must not treat it as one.** It narrows `SavedSearch.folder`, which is separate. |

**The `OneFolder` case is a live gap D-2-03 does not close.** `wx_app.rs:6545-6552` hardcodes
`folder: None` with a stated reason, verbatim
[VERIFIED: src/presentation/wx_app.rs:6545-6552]:

```
        // Everywhere in this account, whatever the search box's "In" list was
        // set to when the search ran. What is kept here is the typed words, not
        // where they were looked for, so narrowing this would be narrowing on
        // something nobody wrote down. The window that names the search says in
        // a sentence that it asks about every message in the account, so
        // somebody who has just searched one folder is told before they save
        // it, rather than finding out when they open it.
        folder: None,
```

That reasoning stops holding once the In box's answer *is* written down for two of the four values.
SEARCH-01's second criterion — "a saved search's folder and its field restriction are written and
read back together rather than by two paths that can come to disagree" — is exactly about this.
Raise it in the plan: either `OneFolder` starts writing `folder`, or the sentence at
`a_typed_search_in_words` has to be rewritten to say the folder is dropped while the field
restriction is kept, which is a stranger thing to say than to fix.

`a_typed_search_in_words` (`saved_searches.rs:363`) is also where D-2-04's sentence builder goes,
and it will need to become a function of the question set rather than of the text alone.

### D-2-05 — account sub-branches for saved searches

The rows today, verbatim `src/presentation/folder_tree.rs:590-606`
[VERIFIED: src/presentation/folder_tree.rs:590-606]:

```rust
    if !searches.is_empty() {
        out.push(plain_row(
            WhichRow::SavedSearches,
            crate::application::saved_searches::THE_HEADING.to_string(),
            0,
            true,
        ));
        out.extend(searches.iter().map(|search| {
            plain_row(
                WhichRow::SavedSearch(search.id.clone()),
                crate::application::saved_searches::a_row_for(&search.name),
                1,
                false,
            )
        }));
    }
```

Flat: heading at depth 0, searches at depth 1. `favourite_rows` at `folder_tree.rs:731` is the
pattern to mirror, and `WhichRow::PinnedIn(account)` is the account-branch identity precedent.

**Two things D-2-05 needs that nothing has today.**

1. **`SearchInTheTree` carries no account.** Verbatim `src/presentation/folder_tree.rs:306-309`
   [VERIFIED: src/presentation/folder_tree.rs:306-309]:
   ```rust
   pub struct SearchInTheTree {
       pub id: String,
       pub name: String,
   }
   ```
   Compare `Pin`, verbatim `src/application/favourites.rs:73-82`
   [VERIFIED: src/application/favourites.rs:73-82]:
   ```rust
   pub struct Pin {
       pub account: String,
       pub path: String,
       /// Where this pin sits among its own account's pins, counting from nought.
       ...
       pub position: i64,
   }
   ```
   `SavedSearch` has no account either — its fields are `id, name, join, questions, folder`
   (`saved_searches.rs:529-544`). The account lives only in the table's `account_id` column and in
   the argument to `get_saved_searches_for_account`. So a third field on `SearchInTheTree` is
   needed, and it is populated at the boundary rather than by widening `SavedSearch`, which is what
   Favourites did.

2. **The read becomes per-account, and that is a real cost.** Today, folders are read per account in
   a loop but labels and saved searches are read for the active account only — verbatim
   `src/presentation/wx_app.rs:9529-9534` [VERIFIED: src/presentation/wx_app.rs:9529-9534]:
   ```rust
       let saved = cache
           .get_saved_searches_for_account(account_id)
           .unwrap_or_else(|e| {
               tracing::warn!("The saved searches could not be read: {e}");
               crate::data::message_cache::saved_searches::SavedSearchesRead::default()
           });
   ```
   D-2-05 moves this into the per-account loop. `STATE.md` records the identical cost being paid in
   `01-14`: "every one of the eleven redraws now reads five things per account rather than five in
   all, and some run on a timer". This makes it six. **State that cost in the plan.** It is the same
   eleven call sites and some of them are on a timer.

`WhichRow::SavedSearches => "saved-searches"` and `WhichRow::SavedSearch(id) => format!("saved-search{APART}{id}")`
(`folder_tree.rs:155-156`) are the stored identities. A new `SavedSearchesIn(account)` variant needs
its own stored spelling, and it must not collide with either — D-25's keying rule.

`every_saved_search` at `src/presentation/wx_app.rs:9709-9727` is the one place that builds
`SearchInTheTree`, including the "saved by another version" rows, and it is the only function to
change on that side.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| A list of conditions a user can add to and remove from | A `ScrolledWindow` of rows, each with its own controls | `run_manager_loop` + `ListCtrl` (`wx_managers.rs:203`) | Set size and position come free from the system's provider on both channels; tab order never changes; already generic over the row type |
| Answering a rule about a message | A second matcher for saved searches | `Question::as_a_rule` → `FilterEngine::matches` | Its own comment says why: "the second vocabulary this module exists to not have" |
| The list of fields and match types the editor offers | A hardcoded array in the dialog | `A_FIELD_A_RULE_MAY_NAME`, `A_WAY_A_RULE_MAY_MATCH` | The hardcoded array is what is wrong today: 6 of 11, twice |
| Knowing whether a search needs bodies | An `if field == "body_plain"` at a call site | `a_rule_reads_the_message_text` / `SavedSearch::reads_the_message_text` | Already written, already the single source, already covers `body_html` |
| Reading a stored body | Reading `body_plain` and `body_plain_packed` yourself | `bodies::body_text(text, packed)` | Its doc: "Two readers deciding this for themselves is two chances for one of them to prefer the other column, and the message that is then shown is a stale copy with nothing saying so" |
| The default for a new config field | Restating the value in a doc comment | A named `fn` in `#[serde(default = "...")]` | `default_allowed`'s doc: "A sentence no check reads should name the answer instead" |
| A settings-screen checkbox's accessible name | An empty label plus `set_accessible_name` | `.with_label("&Text")` on the CheckBox itself | The exact bug `tests/checkbox_labels.rs` exists for: names MSAA only, leaves Narrator with an unnamed control |
| Compressing or decompressing body text | Anything | `flate2` at `Compression::new(6)`, already chosen and measured | `PACKING_EFFORT`'s comment records the measurement |

**Key insight:** almost everything this phase needs already exists in this repository and is one
level of reach away. The single largest risk is not building the wrong thing — it is the two silent
`Allowed` traps, neither of which the compiler or any existing test will catch.

---

## Common Pitfalls

### Pitfall 1: `#[serde(default)]` on the new `Allowed` field
**What goes wrong:** Every existing `app_config.json` stops parsing, or parses with reading off.
**Why it happens:** `Allowed` has no field-level serde attributes today, and `bool`'s `Default` is
`false`, which is the opposite of D-2-07.
**How to avoid:** `#[serde(default = "reading_is_allowed")]` with a named function returning `true`.
**Warning signs:** A test that only round-trips a freshly-serialised struct. It will pass. The test
must take a real serialised object and *remove the key* — the shape at `config.rs:1010-1022`.

### Pitfall 2: putting `reading: false` in `NOTHING`
**What goes wrong:** `--read-only` stops reads. The first-run "Read my mail, change nothing" choice
stops reads. Both silently.
**Why it happens:** `NOTHING` looks like "the all-off constant" and mechanically it is.
Semantically it is "change nothing", which is a claim about writes only.
**How to avoid:** `NOTHING` keeps `reading: true`; hand-write `impl Default` as `Self::NOTHING`.
**Warning signs:** `test_nothing_is_allowed_until_something_says_otherwise` still passing (it will,
either way — it compares `default()` to `NOTHING`, so it cannot see them being wrong together).

### Pitfall 3: assuming the body is unreachable today
**What goes wrong:** The plan builds a body search that already exists, or writes a coverage
sentence describing a state of affairs that is not the current one.
**Why it happens:** D-2-09 says so, and the `SELECT` column list looks like it confirms it.
**How to avoid:** The predicate is `message_search MATCH ?1` against an FTS5 table with a `body`
column. Read `WhereToSearch::EveryFolder`'s doc.
**Warning signs:** Any plan task described as "make the body reachable".

### Pitfall 4: copying `build_filter_edit_dialog`'s hardcoded field list
**What goes wrong:** A third copy of a vocabulary that is already wrong in two places.
**Why it happens:** It is the obvious template and it looks finished.
**How to avoid:** Fix the existing one first, from the constants, with a both-directions guard.
**Warning signs:** A literal `["subject", "from", ...]` anywhere in `src/presentation/`.

### Pitfall 5: offering `deleted`, or a Pattern box for `is_true`
**What goes wrong:** A saved search that can only ever find nothing, or a rule with a meaningless
pattern read out in D-2-04's sentence.
**Why it happens:** The eleven fields and eleven match types are not eleven-by-eleven valid.
**How to avoid:** `is_true`/`is_false`/`is_empty`/`is_not_empty` hide the Pattern box; `deleted` is
either omitted with a reason or the dialog says the scan excludes deleted mail.
**Warning signs:** A test that only checks a rule's round-trip and never runs it over messages.

### Pitfall 6: a guard record going stale inside this phase
**What goes wrong:** A record names 8 tests, the phase adds 4 more that reach the same rule, nothing
fails, and the guard is now measuring less than it claims.
**Why it happens:** `CLAUDE.md`'s guard-record section, extended 2026-08-31, records this happening
four times in Phase 1 — once inside the same day and once inside the same session.
**How to avoid:** any change that adds tests near a rule re-measures that rule's record in the same
commit, and `scripts/guards.sh` runs **unfiltered** before finishing, not the filter you would
naturally pick.
**Warning signs:** `guards/guards.toml` untouched at the end of a phase that added 40 tests.
Specifically at risk here: `SENDS_A_MAIL_CHANGE` at `src/service/outward.rs:1693` is a census
asserting a floor, and `CLAUDE.md` says a census weakens other guards when the floor moves.

### Pitfall 7: `--no-verify`, or a partial mutation run read as a result
**What goes wrong:** Both are recorded as having happened before.
**How to avoid:** Raise the hook timeout rather than skipping it. Wait for `cargo mutants` to exit.

---

## Runtime State Inventory

Not a rename or migration phase, but D-2-06 changes a serialised type, so the categories are worth
answering rather than skipped.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | `app_config.json`'s `allowed_changes` object gains a key; `allowed_per_account`'s map values likewise. `saved_searches` and `saved_search_questions` tables unchanged — D-2-03 and D-2-01 need no schema work, confirmed against `mod.rs:1590` and `:1615`. | Code edit only: a serde default. **No data migration.** An existing file with two keys must go on parsing, and the third answers `true`. |
| Live service config | None. Nothing in this phase touches a server, a provider dashboard, or any configuration held outside this repository. Verified: the only external reach D-2-08 adds is the body fetch, which is an IMAP command in `src/service/protocols/imap.rs`. | None |
| OS-registered state | None. No scheduled task, no service, no registry key. Verified by the phase touching no installer or startup path. | None |
| Secrets and env vars | None. `Allowed` is not a secret and is not read from the environment. `narrow_this_run_to` reads argv only (`src/main.rs:93`). | None |
| Build artifacts | None. No package name changes, no `Cargo.toml` change — this phase adds no dependency. | None |

**The one thing worth watching:** the FTS index. Adding a body column is not needed (it exists), but
if any plan step changes what is written into `message_search`, `build_any_missing_search_index`
compares row counts only and will not rebuild an index that is level but stale. That is a real
migration hazard hiding in a "nothing to migrate" phase.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `wxdragon` | Rule editor | ✓ | `=0.9.17` (pinned, with `aui`, `richtext`, `webview`) | — |
| `rusqlite` / SQLite FTS5 | Search, counts | ✓ | In use; `message_search` and `calendar_search` both exist | — |
| `flate2` | Body unpacking | ✓ | In use at `Compression::new(6)` | — |
| `serde` / `serde_json` | `Allowed`, `AppConfig` | ✓ | In use | — |
| A live IMAP account | The body fetch | ✗ | — | **No fallback, and none is wanted.** See below. |

**Missing dependencies with no fallback:** a real mail account, deliberately.

**What a loopback server can and cannot prove for the body fetch.** It can prove: that the IMAP
command is well-formed, that the response parser handles the shapes a server may send, that the
gate refuses when the read dimension is off, that a refusal produces a sentence rather than an
error code, that a fetched body lands in `message_bodies` and reindexes into `message_search`, and
that a truncated or malformed response is handled the way `unpacked` handles a truncated blob. It
cannot prove: that any real provider answers the command the way the loopback does, that the
throughput is tolerable on a real mailbox, that a provider does not rate-limit or disconnect on a
bulk body fetch, or that fetching bodies for an entire account is a thing a provider will let you
do at all. That last one is the whole risk and no test in this repository can reach it — which is
why D-2-08 says the fetch half is "marked experimental where somebody meets it". Follow
`EXPERIMENTAL_WARNING`'s existing pattern: the sentence goes beside the control, not in a
changelog.

---

## Validation Architecture

`.planning/config.json` has no `workflow.nyquist_validation` key, so it is enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust built-in), plus `cargo mutants` for mutation and `scripts/guards.sh` for guard records |
| Config file | `Cargo.toml`; guard records in `guards/guards.toml` (538 records) |
| Quick run command | `cargo test --lib <module_path>` |
| Full suite command | `cargo test` (lib + the 22 files under `tests/`) |
| Guard verification | `scripts/guards.sh` — **run unfiltered before finishing** |

### Phase Requirements → Test Map

| Req | Behavior | Type | Command | Exists? |
|-----|----------|------|---------|---------|
| SEARCH-01 | `SubjectOnly` writes one question | unit | `cargo test --lib saved_searches::tests` | ❌ Wave 0 |
| SEARCH-01 | An older three-question search is unchanged | unit | `cargo test --lib saved_searches::tests` | ❌ Wave 0 |
| SEARCH-01 | Folder and field restriction written together | unit | `cargo test --lib saved_searches` | ❌ Wave 0 |
| SEARCH-02 | Coverage count is right for an account with some bodies | unit | `cargo test --lib message_cache::saved_searches` | ❌ Wave 0 |
| SEARCH-02 | Count is account-scoped (two accounts do not cross) | unit | same | ❌ Wave 0 |
| SEARCH-02 | Count excludes deleted, matching `scan_query`'s `WHERE` | unit | same | ❌ Wave 0 |
| SEARCH-02 | Body fetch refused when the read dimension is off | unit | `cargo test --lib` | ❌ Wave 0 |
| SEARCH-02 | Body fetch against a loopback server lands and reindexes | integration | `cargo test --test integration_tests` | ❌ Wave 0 |
| SEARCH-03 | Editor offers all 11 fields and all 11 match types | unit, from the constants both ways | `cargo test --lib wx_managers` or a `tests/` file | ❌ Wave 0 |
| SEARCH-03 | Every field arm and every match arm, both ways | unit | `cargo test --lib filters` | ⚠️ Partial — `CLAUDE.md` records four fields and six match types had no test before the 2026-08-01 mutation pass |
| SEARCH-03 | `Join::All` end to end through `run_over` | unit | `cargo test --lib saved_searches` | ❌ Wave 0 |
| SEARCH-03 | An empty condition list is refused at save | unit | dialog test | ❌ Wave 0 |
| D-2-06 | Older config with no read key parses, reading on | unit | `cargo test --lib config` | ❌ Wave 0 |
| D-2-06 | `--read-only` still allows reading | unit | `cargo test --lib command_line` | ❌ Wave 0 |
| D-2-06 | First-run ReadOnly still allows reading | unit | `cargo test --lib first_run` | ❌ Wave 0 |
| D-2-07 | `Default` == `NOTHING`, and both allow reading | unit | `cargo test --lib allowed` | ⚠️ Half — the equality test exists at `allowed.rs:200`; the reading assertion does not |
| D-2-05 | Two accounts with same-named searches are two branches | unit | `cargo test --lib folder_tree` | ❌ Wave 0 |
| Rule editor a11y | Every checkbox and choice carries its own label | integration | new file under `tests/` | ❌ Wave 0 — `tests/checkbox_labels.rs` covers `wx_item_form` only |
| Screen reader listening | Announcements are distinguishable in practice | manual | — | Pratik's, and his to schedule |

### Sampling Rate
- **Per task commit:** `cargo test --lib <the module touched>`
- **Per wave merge:** `cargo test` in full
- **Phase gate:** full suite green, plus `scripts/guards.sh` **unfiltered**, plus `cargo mutants`
  scoped to `application::allowed`, `application::saved_searches` and `application::filters` run to
  completion (never read mid-run)

### Wave 0 Gaps
- [ ] A new file under `tests/` for the rule editor's controls — one `#[test]`, real dialogs, no
      `show_modal`. Cannot go in `tests/checkbox_labels.rs` without changing what that guard covers.
- [ ] A serde-removal test for the new `Allowed` field, in `src/data/config.rs`'s test module,
      following `test_asking_to_start_in_all_inboxes_survives_being_written_and_read_back`.
- [ ] A guard that the editor's offered lists equal the constants, in both directions.
- [ ] Guard records in `guards/guards.toml` for: the read gate on the body fetch, the reading-stays-
      on-in-`NOTHING` rule, and the offered-lists-equal-the-constants rule. Each taken red by hand
      first, per `CLAUDE.md`.
- [ ] Re-measure `SENDS_A_MAIL_CHANGE`'s census and every record that reads it, if a read gate lands
      near `may_i`.
- No framework install needed.

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | This phase adds no auth path. The body fetch reuses the existing authenticated IMAP session. |
| V3 Session Management | no | — |
| V4 Access Control | **yes** | `Allowed` / `Permission` is the access-control model and this phase widens it. The three-places rule (`and()` takes the narrowest on each half) is the control and must not be weakened. |
| V5 Input Validation | **yes** | The rule editor's Pattern box reaches `RegexBuilder` when match type is `regex`, and reaches SQL nowhere — `FilterEngine::matches` compares in Rust. The FTS path is separate and already handles this: `as_a_search` quotes every word (`searching.rs:129-131`), which its doc calls "the whole of the safety here". |
| V6 Cryptography | no | Nothing cryptographic. `flate2` is compression, not encryption. |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Regex denial of service from a user-written rule pattern | Denial of Service | `regex` crate has no backtracking, so a catastrophic pattern is not possible by construction. The remaining risk is a large compiled program; `RegexBuilder` has `size_limit`. **Not verified this session whether a limit is set** — the plan should check `filters.rs`'s `RegexBuilder` call. [ASSUMED] |
| SQL injection via a rule pattern | Tampering | Not reachable: patterns are compared in Rust, never interpolated into SQL. The one interpolation in this area is `message_rows_for`, whose doc states the ids "are numbers this database handed out and were read back from it a moment ago". |
| FTS5 query-language injection from the search box | Tampering | Already mitigated. `as_a_search` quotes every word; its doc records the two bugs that motivated it. |
| A read gate that fails open | Elevation of Privilege | `allowed_for` returns `Allowed::NOTHING` when config cannot be read. With `NOTHING` holding `reading: true`, an unreadable config now permits reads — which is correct for a read but is the exact inversion the plan must state deliberately rather than discover. |
| Widening `Allowed` weakens the write gates | Elevation of Privilege | The write halves' semantics must not change. `anything()` stays writes-only; `and()` stays independent per field. The regression test is the existing `test_one_no_is_enough`. |

---

## Package Legitimacy Audit

**Not applicable. This phase installs no external package.** `Cargo.toml` gains nothing: every
crate needed (`wxdragon`, `rusqlite`, `serde`, `serde_json`, `flate2`, `regex`, `uuid`, `chrono`,
`tracing`) is already a dependency and in use in the files this phase touches.
[VERIFIED: Cargo.toml — no new dependency identified in any part of this research]

Per `dependency-audit`: if the plan finds itself reaching for a crate, that is a signal the design
went wrong, because every capability this phase needs was found in the tree.

---

## Project Constraints (from CLAUDE.md)

Binding, and each affects a specific task in this phase.

| Directive | Where it bites here |
|-----------|---------------------|
| Red then green on every change; `workflow.tdd_mode` is `true` | Every task is `type: tdd` except the `guards.toml` record edits. Confirmed: `.planning/config.json` has `"tdd_mode": true`. |
| Errors through `common::Error`; no `unwrap`/`expect` outside tests | The count query and the body fetch both return `Result`. `allowed_for`'s existing `unwrap_or(Allowed::NOTHING)` is a fallback, not an unwrap on a `Result` that could panic — keep that shape. |
| Schema changes additive only; a column that shipped is never dropped | No schema change is needed. If the plan adds the partial index, `CREATE INDEX IF NOT EXISTS` is additive and safe. |
| No AI attribution anywhere | Commit messages, code comments, branch names. |
| WCAG 2.2 AA | 24x24 targets, no drag-only, keyboard-complete, visible focus, `prefers-reduced-motion` honoured (already handled by `application::scrolling`). |
| Two accessibility channels: UIA for Narrator, MSAA for NVDA; `set_accessible_name` writes MSAA only | The single most important constraint on Question 2's answer. Native controls with real labels serve both. A hand-rolled row stack serves one. |
| A guard record is a measurement with a date and it perishes | Any test added near a rule re-measures that rule's record, same commit. Run `scripts/guards.sh` unfiltered. |
| A census asserting a floor weakens other guards when the floor moves | `SENDS_A_MAIL_CHANGE` at `src/service/outward.rs:1693`. |
| A guard triggered by "a document mentions X" is disarmed by its own workaround | Do not write the offered-lists guard as "the dialog source mentions `A_FIELD_A_RULE_MAY_NAME`". Write it as a comparison of the actual offered `Vec<String>` against the constant, from a built dialog. |
| Before trusting a new regression test, take the fix out and watch it fail | Both `Allowed` traps. Neither has an existing test that can go red. |
| Mark experimental things in the product, where the person meets them | The body fetch. `EXPERIMENTAL_WARNING` is the pattern. |
| Report outcomes faithfully; a feature that compiles but is never reached is not implemented | The read gate must be reached by the fetch, and the coverage sentence must be reached by the search, before either is called done. |
| No feature is done until it runs in production | `dead-code-hunter` after the phase. `allowed_per_account` is the standing example of what this catches. |

---

## Architecture Patterns

### System architecture: how a search reaches message text

```
                                    ┌──────────────────────────┐
   user types in the search box ───▶│  what_the_in_box_offers  │ 4 scopes
                                    │  wx_app.rs:14776         │
                                    └────────────┬─────────────┘
                                                 │ WhereToSearch
                                                 ▼
                                    ┌──────────────────────────┐
                                    │  search_messages         │
                                    │  searching.rs:389        │
                                    └────────────┬─────────────┘
                                                 │ MATCH against FTS5
                                                 ▼
                       ┌─────────────────────────────────────────────┐
                       │  message_search (subject, from_addr,        │
                       │  snippet, body)  content='' — no readback   │◀── index_message_for_search
                       └─────────────────────────────────────────────┘        ▲ (searching.rs:282)
                                                                              │
   ── save the search ──▶ what_a_typed_search_asks ──▶ saved_searches table    │ called from
        (D-2-03 lands here)   saved_searches.rs:349        + questions table   │ bodies.rs:314
                                                                  │           │ messages.rs:925, 1575
   ── the rule editor ──▶ (D-2-01 — new door, same table) ─────────┤           │
                                                                  ▼           │
                                              ┌────────────────────────────┐  │
                                              │ run_a_saved_search         │  │
                                              │ wx_app.rs:6254             │  │
                                              └─────────────┬──────────────┘  │
                                                            │                 │
                            reads_the_message_text()? ──────┤                 │
                                  ┌─────────────────────────┴──────┐          │
                              no  │                                │  yes     │
                                  ▼                                ▼          │
                        TheMessageText::LeftAlone        TheMessageText::Read  │
                        body columns = NULL              LEFT JOIN             │
                                  │                      message_bodies        │
                                  │                                │           │
                                  └───────────┬────────────────────┘           │
                                              ▼                                │
                              messages_a_saved_search_reads                    │
                              saved_searches.rs:230 — UNBOUNDED                 │
                                              │                                │
                                              ▼                                │
                              SavedSearch::run_over → what_it_cannot_read       │
                                              │      → selects → as_a_rule      │
                                              │      → FilterEngine::matches    │
                                              ▼                                │
                                        result rows                            │
                                                                               │
   ── D-2-08 disclosure ──▶ COUNT(*) , COUNT(b.message_id)  ── says both ──┐    │
        (new, before the search runs)  over messages ⋈ folders             │    │
                                       ⟕ message_bodies                    │    │
                                                                           ▼    │
   ── D-2-08 fetch ──▶ [ new read gate: Allowed.reading ] ──▶ IMAP body fetch ──┘
        (new)                D-2-06/07                        service/protocols/imap.rs
                                                              (never run for real)

   evict_bodies_over (bodies.rs:412) DELETEs from message_bodies
     and does NOT reindex ──▶ FTS keeps the words, the scan loses them
```

### Pattern 1: split `build_*_dialog` from `show_*`
**What:** construction returns a widgets struct; a separate function calls `.show_modal()`.
**When:** every dialog. It is what makes a dialog testable at all.
**Example** — `src/presentation/wx_managers.rs:2545`, `2707`:
```rust
pub fn build_filter_edit_dialog(
    parent: &Dialog,
    existing: Option<&FilterRule>,
    palette: Option<theme::Palette>,
) -> FilterEditWidgets { /* ... */ }

fn show_filter_edit(/* ... */) -> Option<FilterRule> {
    let FilterEditWidgets { dialog: dlg, /* ... */ } = build_filter_edit_dialog(/* ... */);
    let answered = dlg.show_modal();
    /* read back, then */ dlg.destroy();
}
```
**Note the destroy.** Its comment records that every dialog in this file leaked for the life of the
session before it was added.

### Pattern 2: consume the click to make a refusal stick
**What:** `event.event.skip(false)` before refusing, so the dialog does not close anyway.
**Example** — `src/presentation/wx_managers.rs:2666-2676`, verbatim
[VERIFIED: src/presentation/wx_managers.rs:2664-2676]:
```rust
            event.event.skip(false);
            if name_f.get_value().trim().is_empty() {
                a_sub_dialog_needs(&d, "A name is needed before this can be saved.");
                name_f.set_focus();
                return;
            }
            d.end_modal(ID_OK);
```
The rule editor needs this for the empty-condition-list refusal.

### Pattern 3: a checkbox carries its own label
**Example** — `src/presentation/wx_settings.rs:1502-1505`:
```rust
    let allow_mail = CheckBox::builder(panel)
        .with_label("Allow Wixen Mail to &send and delete mail")
        .build();
    set_accessible_name(&allow_mail, "Allow Wixen Mail to send and delete mail");
```
Label on the control (UIA reads it through the native provider), accessible name without the
mnemonic marker (MSAA). Both channels, one control.

### Pattern 4: a repeating set of controls built from a constant
**Example** — `src/presentation/wx_settings.rs:1821-1829`, the feedback channels loop, whose comment
states the rule: "Each box carries the channel it switches. The wording comes off the channel too,
so there is no second list to fall out of step with this one and no position to pair by."
This is the pattern the rule editor's Field and Match `Choice` controls should follow against
`A_FIELD_A_RULE_MAY_NAME` and `A_WAY_A_RULE_MAY_MATCH`.

### Anti-patterns to avoid
- **A second vocabulary.** The one this phase must delete already exists at `wx_managers.rs:2569`
  and `:2583`.
- **A sentence a check reads that states the answer.** `default_allowed`'s doc: a sentence no check
  reads should *name* the answer, not restate it.
- **Row data hung off a wxdragon tree control.** `deferred-items.md` records two files doing this and
  never freeing it; the guard names both in an exception list. Do not add a third.
- **Filtering with `selects` directly.** Its doc forbids it; `run_over` is the door.

---

## State of the Art

| Old approach | Current approach | When | Impact here |
|--------------|------------------|------|-------------|
| Bodies inline in `messages` | Own table, packed, LRU-evicted, 512 MiB budget | Before this phase | The coverage question exists at all |
| `LIKE` scan over the mail | FTS5 `message_search`, four columns including `body` | Before this phase | D-2-09's premise is out of date |
| Filter rule doc said three fields | `A_FIELD_A_RULE_MAY_NAME` holds eleven, with both-directions tests | Before this phase | The editor's list is the last copy still saying six |
| Flat folder tree | Account branches, Favourites mirroring them (D-29) | Phase 1 | D-2-05's precedent, and its per-account read cost |

**Deprecated/outdated:** `messages.body_plain` and `messages.body_html` still exist as columns and
are still read by `migrate_inline_bodies`, but nothing writes them any more. A column that shipped
is never dropped. Do not use them for the coverage count.

---

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | `folders.total_count` may not be reliably maintained | Q3, counting | Low — the recommendation is to count rows anyway, so this only matters if the plan tries to use it as a shortcut |
| A2 | `RegexBuilder`'s `size_limit` may not be set in `filters.rs` | Security | Medium — a user-written pattern reaching an unbounded regex compile. One grep to settle; do it before planning the editor |
| A3 | The partial index `idx_messages_folder_live` would help the count | Q3 | Low — it is offered as an option gated on a measurement, per house rule, not as a recommendation |
| A4 | Windows' default `Button` and `ListCtrl` metrics meet 24x24 CSS px | Q2 | Low — Windows defaults exceed this comfortably, but it is a platform claim not read from source this session |
| A5 | A provider will permit a bulk body fetch at all | D-2-08 fetch | **High, and unresolvable here.** This is why D-2-08 marks the fetch experimental. No test in this repository can settle it |

---

## Open Questions

1. **Does `OneFolder` start writing `SavedSearch.folder`?**
   - What we know: `wx_app.rs:6545` hardcodes `folder: None` with a reason that stops holding once
     D-2-03 writes the field restriction down. SEARCH-01's second criterion is about exactly this.
   - What's unclear: whether the phase wants to close it or restate the sentence.
   - Recommendation: raise it in planning as a named task with a stated decision, not as an
     incidental. The two halves "written and read back together" is a criterion, not a nicety.

2. **Does eviction reindex, or does the disclosure carry the difference?**
   - What we know: it does not reindex today, so the live search and the saved-search path disagree
     about an evicted message.
   - What's unclear: which behaviour the product wants.
   - Recommendation: leave it, disclose it, and name which search the coverage sentence is about.
     Reindexing on eviction would take away a search that works.

3. **Is `deleted` offered in the rule editor?**
   - What we know: `scan_query` hardcodes `m.deleted = 0` with a stated reason, so
     `deleted is_true` finds nothing, always.
   - Recommendation: leave it out, with the reason written beside the omission where the next reader
     will find it — not in a changelog.

4. **Where is the rule editor reached from?**
   - Claude's discretion under D-2-06's list. The two-deep option (folder tree context menu on a
     saved-search row) is smaller than the three-deep one and matches what exists.

5. **Does the read dimension get a command-line flag?**
   - Nothing in D-2-06 or D-2-07 asks for one, and `--read-only` already means the opposite thing.
   - Recommendation: no flag. If one were wanted it would have to be `--no-fetch-bodies` or similar
     and would be a fifth thing at the end of `--help`.

6. **Does the per-account control for `allowed_per_account` land in this phase?**
   - It is the standing entry in `STORED_AND_OFFERED_BY_NOTHING` and is written up in Phase 1's
     deferred items. This phase touches `Allowed` and the settings screen, so the temptation exists.
   - Recommendation: **no.** D-2-06's own deferred note says the `Allowed` widening is already the
     largest thing here and should not grow. Adding a control per account is a feature.

---

## Sources

### Primary (HIGH confidence) — files read this session
- `src/application/allowed.rs` — complete
- `src/application/filters.rs` — 1-215
- `src/application/saved_searches.rs` — 100-200, 330-370, 525-680
- `src/application/favourites.rs` — 73-106
- `src/data/config.rs` — 240-300, 430-460, 590-620, 700-760, 990-1035, 1518-1790
- `src/data/message_cache/mod.rs` — 1366-1410, 2040-2075, 2165-2200, 2610-2760
- `src/data/message_cache/searching.rs` — 40-140, 280-345, 380-470
- `src/data/message_cache/bodies.rs` — 1-120, 290-500
- `src/data/message_cache/saved_searches.rs` — 24-260, 316-400
- `src/presentation/wx_managers.rs` — 203-290, 2400-2760
- `src/presentation/wx_settings.rs` — 128-135, 1485-1530, 1800-1870, 2080-2100
- `src/presentation/wx_app.rs` — 6250-6340, 6535-6560, 9500-9545, 9695-9735, 20067-20072
- `src/presentation/folder_tree.rs` — 49-160, 306-330, 493-620, 720-800
- `src/presentation/first_run.rs` — 30-110
- `src/presentation/command_line.rs` — 150-250
- `src/presentation/accessibility.rs` — function list
- `src/service/protocols/imap.rs` — 1170-1200; `src/service/protocols/pop3.rs` — 275-300
- `tests/checkbox_labels.rs` — complete
- `CLAUDE.md` — 185-270, 375-400, 435-450
- `Cargo.toml`, `.planning/config.json`, `guards/guards.toml` (names only)

### Secondary (MEDIUM confidence)
- `wxdragon-0.9.17` widget file listing from the cargo registry — establishes what controls exist,
  not their exact APIs.

### Tertiary (LOW confidence)
- None. No web search was run and none was needed.

---

## Metadata

**Confidence breakdown:**
- Question 1 (the three places, the two traps): HIGH — every claim quoted from source read this
  session, including the two constants and the two strings that make Trap B concrete.
- Question 2 (the rule editor): HIGH on what exists and what the constraints imply; MEDIUM on the
  Windows default target size (A4).
- Question 3 (reaching message text): HIGH, including the D-2-09 correction and the
  eviction-does-not-reindex finding, both read directly.
- `Join::All` reachability: HIGH — the grep is exhaustive and the one non-test site was read.
- The security domain: HIGH except A2, which is one grep away from settled.

**Research date:** 2026-08-31
**Valid until:** 2026-09-30 for the in-repo findings, or the next commit that touches
`src/application/allowed.rs`, `src/data/message_cache/searching.rs`,
`src/data/message_cache/bodies.rs` or `src/presentation/wx_managers.rs`, whichever comes first. Every
finding here is a measurement of this tree at commit `67d12c5`.

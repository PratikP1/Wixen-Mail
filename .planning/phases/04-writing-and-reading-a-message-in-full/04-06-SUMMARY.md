---
phase: 04-writing-and-reading-a-message-in-full
plan: 06
subsystem: moving between misspellings without leaving the message
tags: [spelling, composer, keyboard, accessibility, webview, announcements]
status: complete
requires:
  - "application::spell_session::findings, which builds the list of misspellings no engine exposes, in the page's own coordinates"
  - "application::words::Position and words_in, the coordinates everything here compares in"
  - "editor_document::select_word_script, which already moved the selection to a Position for F7"
  - "presentation::accessibility::announce_what_was_typed, spoken like any other announcement and not written to the log"
provides:
  - "spell_session::next_misspelling and previous_misspelling: which word a walk key reaches, as values in and values out"
  - "spell_session::Step, which is either a word to go to or a sentence to say and nothing to move"
  - "Finding::spoken_without_a_dialog: the sentence for a landing with no list behind it, bounded at SUGGESTIONS_SAID"
  - "editor_document::Caret, caret_script and caret_from_editor: where the selection is, at both ends"
  - "window.wixenCaret in the page, the first thing here that could report where the caret is"
  - "EditorMessage::ToAMisspelling { back }, and the page arm that carries both keys"
affects:
  - "the composer's key handler, which now has one more arm and refuses Alt on the plain F7 arm as it already did"
  - "docs/KEYBOARD_SHORTCUTS.md, whose spelling section names two more keys"
  - "tests/wired.rs, whose handler-bound list has two more entries"
tech-stack:
  added: []
  patterns:
    - "a walk answered as a value, either a word or a sentence, so every boundary case is a test rather than a branch in the window"
    - "both ends of the selection reported, so two directions can start from opposite ones without either offering the word somebody is on"
    - "a bound with a larger request behind it, so the count clause is reachable rather than dead"
key-files:
  created: []
  modified:
    - src/application/spell_session.rs
    - src/presentation/editor_document.rs
    - src/presentation/editor_page_harness.rs
    - src/presentation/wx_compose.rs
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - tests/wired.rs
    - guards/guards.toml
    - Cargo.toml
decisions:
  - "A new sentence rather than a change to Finding::spoken: the dialog's line is followed by a list somebody can arrow into and this one is followed by nothing, so changing the shared one would make the dialog say its suggestions twice"
  - "Three suggestions, with the total said when there are more. One is what the dialog already offers and is what this exists to improve on; fourteen is an announcement nobody can interrupt out of"
  - "Nine asked of the speller, so the count clause is reachable. The speller truncates to what it is asked for, so a bound of three with a request of three makes that sentence dead code"
  - "Neither key consults what an F7 pass ignored. An Ignored set belongs to one pass and ends with it, and these keys have no Ignore to press, so consulting one would mean an ignore list nobody can see, add to or clear"
  - "Both ends of the selection are reported and the two directions start from opposite ones, rather than either direction doing arithmetic on a word's length"
  - "Announced through announce_what_was_typed on a shared topic: High priority to match F7's own sentence, superseding so a held key leaves one sentence, and the word kept out of the log because it comes out of somebody's message"
  - "Not through the deferral timer F7 uses. This opens no dialog and starts no nested event loop, which is why the Markdown link arm beside it already runs directly"
  - "Alt+F7 and Alt+Shift+F7, checked against every backticked combination the shortcuts document names and against the page's own handler"
metrics:
  duration: about three hours
  completed: 2026-09-06
  commits: 4
  version: 0.67.0 to 0.69.0
actuals:
  tokens: 62000
  tasks: 2
  commits: 4
---

# Phase 04 Plan 06: Moving Between Misspellings Without Leaving the Message Summary

`Alt+F7` moves the caret to the next misspelled word and says what it could be
instead; `Alt+Shift+F7` goes back. Neither opens anything, and `F7` is untouched.

## What works, plainly

**Both keys work and both are reached from non-test code.** The page binds them
in its keydown handler and posts a message over the `wixenEditor` channel;
`wx_compose`'s `on_script_message_received` receives it and calls
`walk_to_a_misspelling`, which reads the message text, reads where the caret is,
asks `spell_session` which word to go to, moves the selection there and says the
sentence. There is no timer between the key and the walk, and no dialog anywhere
in it.

**What it says when it lands.** For a word with two suggestions:

> wrold, not in the dictionary. Try world or wold

For a word with seven, which is where the bound does its work:

> wrold, not in the dictionary. Try guess1, guess2 or guess3. 7 suggestions in all

For a word with none:

> Kowalczyk, not in the dictionary. No suggestions

For the three cases where nothing moves:

> No misspellings in this message.
> No more misspellings after here.
> No misspellings before here.

**What has not been settled, and cannot be here: none of this has been heard.**
Four separate questions, all in `.planning/WINDOWS.md` as entries 120 to 123,
and the first of them is load-bearing for the two clauses of criterion 3 that
shipped before this plan.

## The plan's premises

**Premise 8's guard-record table is right in every row.** That is the first time
in phases 3 and 4, and it is worth saying because ten plans have now been
corrected on this one table. Measured by parsing `tests_last_seen`:

| file | records | tests then |
|---|---|---|
| `src/application/spell_session.rs` | 0 | 20 |
| `src/presentation/editor_document.rs` | 0 | 69 |
| `src/presentation/editor_page_harness.rs` | 0 | 28 |
| `src/presentation/wx_compose.rs` | 2 | 43 |
| `tests/wired.rs` | 8 | 61 |
| `src/service/spellcheck/mod.rs` | 30 | 56 |

### The one that changed how the work was done

**The two tasks are not separable as the plan writes them.** Task 2's acceptance
criteria require an inverse property test that is red at its own red commit. It
cannot be, if task 1 is built the way the feature wants to be built. A forward
key and the same key with Shift are one `if` in the page, one field on the
message, and one match with two arms in the answer. Every one of those is the
right shape for the finished feature, and every one of them finishes task 2
while task 1 is being written. The only alternatives are a task 2 whose tests
were green before they were written, or a task 1 that ships a key wired to
`todo!()`.

The first red commit was made before this was seen, and the tell was inside it:
one assertion that the page posts `back: event.shiftKey`, which is a fact about
the axis task 2 exists to add. It was withdrawn with `git reset --mixed` on an
unpushed branch and rewritten. Task 1 binds `Alt+F7` only, with `!event.shiftKey`
in the condition, posts a message with no direction in it, and answers through
`next_misspelling(found, from)`, which has no direction parameter anywhere. Task
2 then widens all three, which is why its red commit changes two of task 1's
assertions as well as adding its own.

### The other four

- **The enumeration and the caret move ship. Reporting where the caret is did
  not.** Premise 2 is right that `findings` enumerates and `select_word_script`
  moves, and right that no spike was needed. `position_from_editor` reads back
  where a *replacement* left the caret, which is a different question asked at a
  different moment, and nothing on the page reported the selection at all. So
  `window.wixenCaret` is new page code, and it is the one piece of this that a
  real browser engine could still surprise.
- **Both `<verify>` commands cannot run**, exactly as 04-05 reported and this
  plan repeated. `cargo test --lib a --lib b --lib c` is refused: "the argument
  '--lib' cannot be used multiple times". The form that runs is
  `cargo test --lib -- a b c`.
- **Premise 5 understates how narrow the shortcuts check is.**
  `documented_combinations` collects a backticked key only when it **starts
  with** `Ctrl+` or `Alt+`. `Alt+F7` and `Alt+Shift+F7` are both collected and
  both needed the exception entry, which is what the premise says. Had the
  backward key been written `Shift+Alt+F7`, the check would have skipped it in
  silence: no entry needed and no protection given. Ledger 125.
- **The bound's count clause is unreachable unless the speller is asked for more
  than the bound**, and the plan does not name that. `check_spelling` asks for 5;
  a bound of 3 with a request of 3 would make the sentence for a word with more
  suggestions dead code that no test could reach honestly.

## Task 1: a key that moves to the next misspelling

### Which assertions were green the moment the code existed

The plan warned that `next_finding` already answers "the first finding at or
after this position", so a test asserting that says nothing. That is right, and
here is where it landed. `next_misspelling` is a thin thing: an empty-list guard,
then a `find` over a comparison. So **`test_the_first_press_lands_on_the_first_misspelling`
and `test_pressing_it_again_moves_on_rather_than_offering_the_same_word_twice`
were satisfied by the first implementation that compiled.** They are kept because
they pin the two boundary conventions the rest of the feature is built on, not
because they drove anything.

What carried information: the two sentences, the three "moves nothing" answers,
the ignored-word contrast, the key binding, and every caret-reading assertion.

### The three traps the plan named, and what each came to

- **A test asserting what `next_finding` already does.** Two of them, named
  above.
- **A fixture whose "misspelled" word the made-up speller accepts.** Avoided by
  building every fixture through `findings` with a speller that rejects
  everything, so a finding either exists or the fixture is obviously empty.
  `every_word_wrong` is that helper.
- **A page assertion matching F7's existing arm.** The assertion anchors on
  `event.key === 'F7' && event.altKey && !event.ctrlKey`, the whole condition
  with its modifiers, and a second assertion pins the post itself:
  `post({ kind: 'misspelling', back: event.shiftKey })`. A bare `'F7'` would have
  matched the arm above it and passed with nothing new bound. The test's own
  comment says so.

### The two sentences, side by side

`Finding::spoken` is unchanged. It is worth reading the two together, because
the difference is the situation rather than the wording:

> **spoken (F7's dialog):** wrold, not in the dictionary. First suggestion, world
>
> **spoken_without_a_dialog (the walk):** wrold, not in the dictionary. Try world or wold

The dialog's line is followed by a list control somebody can arrow into, so
naming the first suggestion saves a key press and the rest are one arrow away.
The walk's line is followed by nothing at all. One suggestion there is one guess
and no way to hear the others, which is exactly what criterion 3 says is not
enough. Changing the shared sentence would have made the dialog read its
suggestions out and then show them in the list underneath.

### The bound, and why three

`SUGGESTIONS_SAID` is 3, and its comment carries the reason verbatim:

> Three, and the number is a judgement rather than a measurement. One is what
> the dialog says and is what this feature exists to improve on: a person who
> disagrees with the first guess has heard nothing useful. Fourteen is an
> announcement nobody can interrupt out of, and the words are gone by the time
> the fourth arrives. Three is about as many as anybody holds from one spoken
> sentence without asking for it again, and where there are more, the count says
> so, so nobody is left thinking three is all there was.

`SUGGESTIONS_TO_HAVE` is 9, which is what the composer asks the speller for. Its
comment says the count in the sentence is the number this program has rather
than a claim about how many the dictionary could produce, because the speller
truncates to what it was asked for. That is guardrail 9: the gap is named rather
than papered over.

### The key, and what it was checked against

Every backticked combination in `docs/KEYBOARD_SHORTCUTS.md` was listed and
sorted: 29 `Alt+` combinations, 65 `Ctrl+` ones, and five `Shift+` ones. Neither
`Alt+F7` nor `Alt+Shift+F7` is among them. Every `F7` and `F8` in the document
and in `src/` was read: F7 is the composer's spelling dialog and the reader's
security warning bar, F8 is the reader's attachment list and the message list's
column chooser. The page's own handler was read arm by arm: the plain F7 arm
refuses Alt, and the Alt-and-a-letter arm requires `event.key.length === 1`, so
a function key cannot reach it. Word binds Alt+F7 to the same thing, which is
why it was chosen rather than invented.

### The priority, and why it agrees with F7

`Priority::High`, which is what `check_spelling` uses for `finding.spoken()` and
for `finished()`. They agree because they are the same kind of sentence: the
answer to a key somebody has just pressed and is waiting for, with nothing else
in the window competing for the same moment.

Two things differ from F7's call, both deliberate. It goes through
`announce_what_was_typed` rather than `announce`, so the misspelled word is not
written into the log; that word comes out of a message, which is either what
somebody typed or what a stranger sent them, and the log is a file people are
asked to attach to bug reports. And it carries the topic `spelling-walk`, so a
held key leaves one sentence in the queue rather than a queue of them. That is
T-04-25, answered: a walk in which every word supersedes the last is what
somebody holding the key is asking for.

### The guard record, measured by hand

Break: `SUGGESTIONS_SAID` from 3 to 1. Every other word of the sentence is left
alone, including the count clause, so the only thing that changes is how many
things somebody is offered. Measured with `cargo test --all-targets
--no-fail-fast` over 33 targets, then confirmed with `scripts/guards.sh`. **Two
tests red:**

```
application::spell_session::tests::test_a_word_with_more_suggestions_than_are_said_says_how_many_there_are
application::spell_session::tests::test_landing_on_a_word_offers_more_than_one_thing_to_try
```

Both in the module the constant lives in, and that is the finding: **nothing
outside `spell_session` asserts the bound at all.** The three sentence tests that
stay green are the ones that do not reach the take: a word with no suggestions,
and the pair asserting the dialog's sentence and this one are different, which
under the break they still are.

The fixtures do not move with the constant. Both hold literal suggestion lists
of two and seven, which is what plan 03-07 found the other way round: a bound
test whose fixtures widened with the break passed straight through the defect it
was written for.

## Task 2: and it goes backwards

### The inverse property, and what it caught

Written as one test over a fixture of five misspellings rather than as two
walks, because two hand-written walks would each have to be right for the pair
to disagree. It walks forward collecting the words, walks back collecting them,
and asserts the second reversed is the first.

**At the red it failed on `todo!`, which proves nothing about the test.** So it
was measured properly: the naive backwards answer, `find` over the list in
reading order, was written and run before the real one.

```
assertion `left == right` failed: the two directions disagree
  left: ["aaa", "bbb", "ccc", "ddd", "eee"]
 right: ["aaa"]
```

That is the answer to the plan's question. The property test **can** fail against
a naive backwards implementation, and it fails in the way that shows what is
wrong: a walk backwards over five misspellings reaches one word, because "the
earliest word before here" takes you to the start of the message in one press
and then says there are no more. The naive version was thrown away, not
committed.

The walk in the test is bounded rather than looped until it stops, and that
matters for the other way to get this wrong. An implementation using `<=` rather
than `<` offers the same word forever; without the bound the test would hang
rather than fail, and a test that hangs says nothing.

### No arithmetic, and the comment it comes from

`previous_misspelling` is `found.iter().rev().find(|f| ... f.at < from)`. There
is no subtraction anywhere in it and nothing works out how far anything has
moved. That is the constraint `next_finding`'s own comment records:

> There is no arithmetic here on purpose: the version this replaced worked out
> how far a replacement had moved the words after it, and getting that wrong
> skipped a misspelling silently rather than reporting anything.

The temptation is stronger going backwards, because "the one before" reads like
subtraction. The answer is a comparison against a position the page reported, in
both directions.

### Ignored words: the decision, and its reason

**Neither key is affected by what an F7 pass ignored.** The reason is written on
`next_misspelling`:

> Nothing here consults `Ignored`. That set belongs to one F7 pass and ends with
> it, and this walk has no Ignore to press, so consulting it would mean keeping
> an ignore list alive that nobody can see, add to, or clear. A word somebody
> passed over in a dialog is still a word they can walk onto.

The argument the other way is real: somebody who has just told the program to
leave a word alone will not expect a key to take them straight back to it. It
loses on the state question. `Ignored` is a local in `check_spelling` and dies
when the pass ends, so honouring it from the walk means giving the composer a
longer-lived ignore set with no way to see it, add to it or clear it, and a key
that skips a word for a reason nobody can inspect is worse than a key that
offers it.

It is asserted in both directions, and each test contrasts the two walks in one
place: `next_finding` with a populated `Ignored` still skips the word, and the
walk still reaches it.

### The two boundary sentences

```
previous_misspelling(&found, Some(place(0))) == Step::Stay("No misspellings before here.")
next_misspelling(&found, Some(place(100))) == Step::Stay("No more misspellings after here.")
```

Both move nothing. Not wrapping is the deliberate answer: a walk that goes
quietly round to the other end means somebody who has been pressing a key has no
way to tell they are on their second time through, and the words are the same
either way.

### The second guard record, measured by hand

Break: dropping `.rev()`. Not a slip, but what a backwards search reads like when
it is written from the forward one. Measured with `cargo test --all-targets
--no-fail-fast` over 33 targets on a clean tree, then confirmed with
`scripts/guards.sh`. **Four tests red:**

```
application::spell_session::tests::test_landing_on_a_word_backwards_says_what_landing_on_it_forwards_says
application::spell_session::tests::test_pressing_the_backward_key_again_moves_to_the_one_before_that
application::spell_session::tests::test_the_backward_key_from_the_end_lands_on_the_last_misspelling
application::spell_session::tests::test_the_two_directions_are_inverses_over_a_message
```

The fourth is the one written for it. The third one in that list is red for a
reason that is not what it is about: it reaches a different word, so it compares
the wrong two sentences. That is recorded in the record rather than left to be
rediscovered.

**The first measurement of this break was wrong and was thrown away.** It
reported five failures, and the fifth was
`test_every_guard_record_says_how_many_tests_the_files_it_names_held`, which was
already red before the break because task 2's seven new tests had made the
record above it stale. A break measured against a tree that is not green
overstates what the break does, and the extra name was only caught because it was
recognisably a registry check rather than a test about the feature. The record
was re-measured with `scripts/guards.sh --remeasure`, and the break was then
measured again from a clean tree. The four above are from that second run.

## Deviations from the plan

### [Rule 3, blocker] The first red commit was withdrawn and rewritten

- **Found during:** task 2, before its first test was written.
- **Issue:** task 1 as committed asserted `back: event.shiftKey`, which made
  task 2's required inverse property test green before it was written. See the
  premises above.
- **Fix:** `git reset --mixed HEAD~1` on an unpushed branch, `git checkout --`
  on the four files that belonged only to the green half, and task 1 rewritten
  forward-only. The gate re-ran and accepted the corrected red.
- **Why not Rule 4:** no architectural change. The finished feature is the same
  in every respect; what changed is which commit each half arrives in.

### [Rule 2, missing critical] The guard-record count check inside a red commit

- **Found during:** task 2's red commit, which the gate refused.
- **Issue:** the seven new tests land in `spell_session.rs`, which task 1's own
  new guard record names, so its recorded count went stale the moment they
  existed. The remedy is a re-measure, which needs the green code and a tree
  with nothing else red in it. There is no ordering in which that commit has a
  clean tree around it.
- **Fix:** the count check named in the red commit's trailers alongside the
  seven real failures, with the reason in the message. This is exactly the
  collision `CLAUDE.md` describes and the answer it prescribes.

## Test counts, before and after

| file | before | after |
|---|---|---|
| `src/application/spell_session.rs` | 20 | 36 |
| `src/presentation/editor_document.rs` | 69 | 74 |
| `src/presentation/editor_page_harness.rs` | 28 | 31 |
| `src/presentation/wx_compose.rs` | 43 | 43 |
| `tests/wired.rs` | 61 | 61 |
| `src/service/spellcheck/mod.rs` | 56 | 56 |

`wx_compose.rs` is named by 2 records and broken by 3, and `wired.rs` by 8, so
neither gained a `#[test]`. `spellcheck/mod.rs` is named by 30 and was not
touched at all. The 16 that went into `spell_session.rs` cost one re-measure of
the one record that names it.

## What the page harness can and cannot say about the caret

`window.wixenCaret` is the only genuinely new page code here, and the harness
runs three tests against the page that ships:

- with no selection at all, which is a composer nobody has clicked in, it
  answers `null` rather than throwing. A handler that throws stops without a
  word, so the key would do nothing and there would be no way to tell that from
  a key nobody bound
- with a stub document of three text nodes and a selection in the second, it
  reports `{"start":{"node":1,"offset":0},"end":{"node":1,"offset":3}}`, which is
  the mapping into the coordinates `wixenText` numbers its nodes in
- with a selection in a node the walker never hands back, which is what a caret
  inside an element rather than inside text looks like, it answers `null` rather
  than node -1

What it cannot say: whether a real WebView2 hands back the ranges these tests
model. The harness's own module doc is explicit that its document is a model of
a browser and not a browser. In particular, **that a selection made by
`wixenSelectWord` reports its range end where this expects it** is DOM-specified
and unverified here, and it is what makes "press it again and it moves on" work.

## T-04-23, answered

A position that is not a position moves nothing rather than panicking, and there
are two layers of it. `caret_from_editor` answers `None` for an empty string,
for text that is not JSON, for `null`, for a half-built position and for a
position whose fields are the wrong type, with a test naming all five. A `None`
caret makes the walk start from the beginning or the end rather than guess, and
`wixenSelectWord` clamps its offsets and returns false for a node index it does
not have. No `unwrap` was added anywhere.

## Known stubs

None. Both keys are reached from `on_script_message_received`, which is the web
view's own callback and not a test, and every path added here is on the way
between that callback and an announcement.

## Not settled here, and this criterion has more riding on it than most

- **Whether NVDA announces the engine's spelling marks in this application at
  all.** That is the premise of the two clauses of criterion 3 that shipped
  before this plan. If it does not, the marking is decorative and these two keys
  are the only spelling check that reaches a screen reader. Ledger 120.
- **Whether this program's sentence and the screen reader's own announcement of
  the marked word collide when the caret lands.** Two voices for one fact, in
  the same instant, because the selection change is what makes the reader speak.
  Ledger 121.
- **Whether the earcon at the end of a mistyped word and the landing
  announcement are told apart.** Guardrail 5 asks for feedback that is distinct
  and nothing here can say whether these two are. Ledger 122.
- **Whether three suggestions is heard as helpful or as a list to sit through.**
  Ledger 123.
- **The plan's premises**, so they reach whoever plans the next phase. Ledger
  124.
- **The shortcuts check's blind spot for a key spelled `Shift+Alt+...`.** Ledger
  125.

## WRITE-03's `[D]` line, answered rather than built

> Whether the richtext control can carry the marks is settled first and written
> down; if it cannot, the requirement is re-scoped rather than half-built.

It cannot, and it was written down before this phase started.
`editor_document.rs`'s module header, quoted in full:

> The composer's body is a `contenteditable` in a web view rather than a
> `wxRichTextCtrl`, and the reason is accessibility rather than formatting.
> `wxRichTextCtrl` is drawn by wxWidgets on every platform, so it exposes no
> per-range accessibility attributes anywhere, which means no misspelling can
> ever be marked, no heading can report itself, and every announcement has to be
> made by hand and will be slightly wrong forever.
>
> A web view gets all of that from the engine. `spellcheck` alone produces
> native spelling annotations on all three platforms: UIA spelling errors from
> Chromium, `AXMarkedMisspelled` from WebKit, AT-SPI `invalid:spelling` from
> WebKitGTK. Each screen reader then announces them itself, which is always
> better than us doing its job.

`wxRichTextCtrl` was not adopted, the `richtext` feature in `Cargo.toml` was not
touched, and no package was added. `Cargo.toml` changed by one line across the
whole branch, which is the version.

## Criterion 3, clause by clause

| clause | state |
|---|---|
| misspellings are marked as they are typed | **closed structurally, before this plan.** The engine marks them and each screen reader announces the marks itself. Whether that reaches NVDA here is ledger 120 |
| a keyboard command moves between them without leaving the editor | **closed.** Both directions, with the caret left in the message, tests for every boundary and two guard records |
| landing on one says what it could be instead | **closed.** More than one suggestion, bounded at three, with the total said when there are more |
| a long paste is not checked as it lands | **closed, before this plan.** The typing rules decline for `insertFromPaste` and a harness test says so |

Closed structurally, and not closed by ear. Four ledger entries say which parts
of that sentence are which.

## Owed after the merge

Nothing on the critical path. This branch changed `spell_session.rs` (1 record
names it, re-measured during the work), `editor_document.rs` (0),
`editor_page_harness.rs` (0), `wx_compose.rs` (2 name it, 3 break it, and no
`#[test]` was added or removed), and `wired.rs` (8 name it, count unchanged). The
per-commit count check stayed green after the one re-measure, so the sweep is
owed to phase 8 rather than to this merge.

## Self-Check: PASSED

- `src/application/spell_session.rs` holds `pub fn previous_misspelling`: FOUND
- `src/application/spell_session.rs` holds `pub const SUGGESTIONS_SAID: usize = 3;`: FOUND
- `src/presentation/editor_document.rs` holds `window.wixenCaret = function`: FOUND
- `src/presentation/editor_document.rs` holds `event.key === 'F7' && event.altKey && !event.ctrlKey`: FOUND
- `src/presentation/wx_compose.rs` holds `fn walk_to_a_misspelling`: FOUND
- `docs/KEYBOARD_SHORTCUTS.md` holds `Alt+Shift+F7`: FOUND
- `tests/wired.rs` holds both keys in `bound_by_a_handler_rather_than_a_menu`: FOUND
- `guards/guards.toml` holds both new records, and its census reads 425: FOUND
- commits cecb421, d729c49, eda6ef7, 2face17: FOUND
- `cargo test --all-targets --no-fail-fast`: 0 failures
- `scripts/check.sh` ran through the commit hook on every commit, in full each
  time because each commit touched `Cargo.toml`; nothing used `--no-verify`
- across the whole branch, `src/` and `tests/` gained 809 lines and lost none;
  the only two deleted lines anywhere are the version in `Cargo.toml` and the
  census line in `guards/guards.toml`

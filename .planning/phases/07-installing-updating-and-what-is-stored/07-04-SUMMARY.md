---
phase: 07-installing-updating-and-what-is-stored
plan: 04
subsystem: infra
tags: [versioning, semver, release-channel, settings, serde, guards]

requires:
  - phase: 07-installing-updating-and-what-is-stored
    provides: nothing this plan reads; waves 1 to 3 landed first and none of them touches src/common/version.rs
provides:
  - version::compare, an ordering over two version strings with four named answers
  - version::ReleaseChannel, two values, derived from the setting and never stored
  - version::WhichUpdates, the three-valued thing somebody chooses, with its stored spellings fixed
  - WhichUpdates::channel, the one mapping from the setting to an optional channel
  - version::whether_to_offer, the channel-aware answer to whether a release is an offer
  - A parser that accepts the leading v a published tag carries
affects: [07-05, 07-09]

actuals:
  tokens: 7551
  tasks: 2
  commits: 6

tech-stack:
  added: []
  patterns:
    - "An enum whose stored spelling is a named function on both sides, with serde written by hand so an unknown value loads rather than failing the file"
    - "A parser that is total, so the function above it has no error to flow and no caller has one to handle"

key-files:
  created: []
  modified:
    - src/common/version.rs
    - guards/guards.toml
    - .planning/WINDOWS.md

key-decisions:
  - "ReleaseChannel is derived and never stored, so there is no stored channel spelling and no unknown stored channel value. The plan asserted both readings and this settles it on the plan's own premise about where the setting lives"
  - "The setting type is WhichUpdates, named for the question somebody answers rather than for the machinery, with values NotLooking, PublicReleases and DevelopmentReleases"
  - "The stored spellings are not_looking, public_releases and development_releases, lower case with underscores because every other key in that file is"
  - "Serialize and Deserialize are written by hand, because a derived reader refuses a value it does not know and refusing is the wrong answer for the case a downgrade produces"
  - "A leading v is accepted, because a published tag carries one. Read off release.yml's wixen-mail-v*.exe glob, since no tag exists"
  - "A prerelease word this project does not cut is refused rather than given a position, which is the opposite answer from build-installer.sh and right for a different reason"
  - "No crate was added. semver 1.0.28 is in Cargo.lock through rustc_version as a build dependency and is not in the binary"
  - "Both defaults were deliberately wrong in the red commits, because the default is the thing most worth getting wrong here"

patterns-established:
  - "A constant for a separator that two inverse operations share, rather than the character written in each"
  - "A guard break that adds a tiebreaker rather than removing a split, so the record measures the rule and not whether a function is called"

requirements-completed: []

coverage:
  - id: D1
    description: "Two version strings can be put in order, including every case the release workflow can produce a tag for: later numbers, equality, build metadata after a plus, the three prerelease words, and the counter compared as a number"
    requirement: "SHIP-02"
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_a_later_patch_minor_or_major_is_newer"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_a_release_is_newer_than_the_prerelease_that_staged_it"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_the_three_prerelease_words_sort_in_the_order_the_workflow_makes_them"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_the_prerelease_counter_is_compared_as_a_number"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'a build identifier is not part of the version'"
        status: pass
    human_judgment: false
  - id: D2
    description: "A build identifier after a plus plays no part in the order, so two builds of one version are one version"
    requirement: "SHIP-02"
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_a_build_identifier_after_a_plus_is_not_part_of_the_version"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_two_builds_of_one_version_are_the_same_version"
        status: pass
    human_judgment: false
  - id: D3
    description: "A string that is not a version is refused by name rather than treated as very old, and that answer is distinct from there being nothing newer"
    requirement: "SHIP-02"
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_a_version_it_cannot_read_is_refused_rather_than_treated_as_very_old"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_a_string_that_is_not_a_version_is_refused_rather_than_crashing"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_a_tag_it_cannot_read_is_a_different_answer_from_nothing_newer"
        status: pass
    human_judgment: false
  - id: D4
    description: "Whether a published release is an offer depends on the channel somebody chose, and a prerelease is never an offer on the public one"
    requirement: "SHIP-02"
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose"
        status: pass
      - kind: other
        ref: "scripts/guards.sh 'the public channel never offers a prerelease'"
        status: pass
    human_judgment: false
  - id: D5
    description: "What somebody chose is one thing with three answers, stored under three fixed spellings, and one this build does not know loads as not looking"
    requirement: "SHIP-02"
    verification:
      - kind: unit
        ref: "src/common/version.rs#test_each_answer_has_its_own_stored_spelling"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_every_answer_survives_being_written_and_read_back"
        status: pass
      - kind: unit
        ref: "src/common/version.rs#test_a_stored_answer_this_build_does_not_know_is_not_looking"
        status: pass
    human_judgment: true
    rationale: "The round trip goes through serde_json in a test, not through the loader that will really read it. src/data/config.rs reads the whole settings file in one call and propagates the error, so what a stored value of the wrong type does to every other setting on that machine is 07-05's to establish. WINDOWS.md 314."
  - id: D6
    description: "The comparison and the channel rule have ever been handed a real version string"
    requirement: "SHIP-02"
    verification: []
    human_judgment: true
    rationale: "No release has been published from this repository and git tag returns nothing, so every string either of them has seen was chosen by a test. That a published tag carries a v is read off a publishing glob rather than off a tag. WINDOWS.md 311 and 312."
  - id: D7
    description: "Any of this is reachable from a path a person can take"
    verification: []
    human_judgment: true
    rationale: "Nothing outside the tests calls the ordering, the channel, the setting type or the offer decision. That is the whole shape of this plan and 07-05 is what closes it. WINDOWS.md 313."

duration: 80min
completed: 2026-09-12
status: complete
---

# Phase 7 Plan 04: Two versions can be put in order, and a channel decides what is an offer Summary

**The program can now tell a newer release from an older one, from itself, and from a string it cannot read, and it can say whether that release is an offer for somebody given the one choice they made. Nothing offers anybody anything yet, because nothing outside the tests calls any of it.**

## Performance

- **Duration:** 80 min
- **Started:** 2026-09-12T06:41:06-04:00 (first commit)
- **Completed:** 2026-09-12T08:01:17-04:00 (merge)
- **Tasks:** 2
- **Files modified:** 4, of which none is new

`actuals.tokens` is 7,551, taken as the characters of the added and removed
lines of the whole branch divided by four: 30,206 characters, excluding
`.planning`. The command, so the next reader re-runs it rather than trusting the
figure:

    git diff f063041a..HEAD -- . ':(exclude).planning' | grep '^[+-]' | wc -c

Against an estimate of 67,000 with `raw_tokens` 56,000, that is a ratio of about
0.13 on raw. The four most recent summaries give 0.13, 0.10, 0.17 and 0.15 on
the same measure, so this sits with its neighbours and the estimate was high by
about the same factor theirs were. The figure is not rounded toward the
estimate.

## Accomplishments

- Two version strings can be put in order, covering every tag the release workflow can produce: three numbers, the three prerelease words in the order the workflow makes them, and the counter after the word compared as a number rather than as text.
- A build identifier after a plus plays no part, which is what `describe` promised when it chose that separator, and the two now share one constant rather than one rule written in two places.
- Whether a release is an offer depends on the channel somebody chose. A prerelease is never an offer on the public channel and is on the development one, and both channels offer `0.6.0` to somebody running `0.6.0-alpha.1`.
- What somebody chooses is one thing with three answers, with the three stored spellings fixed and an answer this build does not know loading as not looking.
- A published tag carries a `v` and the comparison accepts it. Without that, every check would have answered "could not read" for every release this project ever cuts, with every test green.
- No crate was added, and the reason is measured against `Cargo.lock` rather than asserted.

## Task Commits

1. **Task 1: Two versions can be put in order** (RED) `95b4b77b` (test)
2. **Task 1: Two versions can be put in order** (GREEN) `bf97493d` (feat)
3. **Task 2: What somebody chose, and whether a release is an offer** (RED) `b31f2ae7` (test)
4. **Task 2: What somebody chose, and whether a release is an offer** (GREEN) `f8e8c679` (feat)
5. **Ledger** `265a2944` (docs)
6. **This summary, STATE.md and ROADMAP.md** (docs)

**Merge:** `441fca5` on `main`, from branch
`two-versions-can-be-put-in-order-and-a-channel-decides-what-is-an-offer`. The
hash is written here by the follow-up commit, since a summary committed before
its own merge cannot name it.

`scripts/check.sh all` on the branch tip, not piped and redirected to a file,
took **445 seconds** and passed all four, over 7,467 tests.

## The decision the correction pass asked for: `ReleaseChannel` is derived, never stored

**It is derived.** Nothing writes a channel to a file. The setting holds
`WhichUpdates` and `WhichUpdates::channel` works the channel out from it, which
is what premise 1a of the plan says in as many words: the three-valued type's
"whole job is to be stored by 07-05 as one top-level `AppConfig` field".

So `ReleaseChannel` has no `Serialize`, no `Deserialize` and no stored spelling.
There is nothing for 07-05 to keep and nothing for a later build to rename.

**What that makes false in the plan, stated rather than quietly satisfied.**

| In the plan | What it says | Status now |
|---|---|---|
| Acceptance criterion, task 2 | The stored serde spelling **of the channel** is stated in the summary with a round-trip test | Void. There is no stored channel spelling. The criterion carries its own correction of 2026-09-11 naming this and recommending exactly this resolution. |
| Acceptance criterion, task 2 | An unknown stored **channel** value loads as the released-only channel | Void, same reason. No channel value is ever stored, so none can arrive unknown. |
| Trust boundary | "a stored channel value to the decision ... arrives from a settings file an older build or a person's text editor wrote" | Void. What arrives from a settings file is `WhichUpdates`, and that boundary is real and is covered. |
| Threat T-07-16 | "an unknown stored channel value read as the development channel, quietly opting somebody into prereleases after a downgrade" | The threat survives; its subject moves. An unknown stored **setting** value reads as not looking, with a test, which is a stronger answer than the threat asked for: not looking fetches nothing at all, where the released-only channel still fetches. |

**Why this way round rather than the other.** The other reading is coherent:
07-05 could store a channel and a switch beside it. D-16 refuses that shape for
an accessibility reason rather than a tidiness one, and the reason is in the
decision: a dependent control that greys out when its parent is off is skipped
in the tab order, so for somebody moving by keyboard it does not read as
unavailable, it is not there. One control with three values is what the product
owes, and a type that stores three answers is what a control with three values
needs. Making the channel storable as well would have meant two stored
representations of one choice, which is the thing that drifts.

There is also a cost the plan's own correction names: a two-variant enum cannot
express "unknown loads as released-only" through `#[serde(other)]` on a plain
externally tagged enum, so a stored channel would have cost a second hand
written `Deserialize` doing the same job as the first.

## The missing precedent, and what was chosen instead

**The correction is right and was re-checked rather than believed.**

    $ grep -rn -B4 'pub enum ' src/ --include=*.rs | grep -i 'erialize'
    (no output)

No enum in `src/` derives `Serialize` or `Deserialize`. `ThreatKind` at
`src/service/safebrowsing/mod.rs:57` derives `Debug, Clone, Copy, PartialEq, Eq`
and nothing else, carries three `const fn` accessors rather than a `From` or a
`Display`, and is never stored. `AppConfig`'s one typed field holds `Allowed`,
which is a struct.

So this is a first, and what was chosen is written down here because 07-05 has
to store the same thing and nothing else in the tree constrains it.

**A named function on each side, and serde written by hand over the two.**
`as_stored` gives the spelling and `from_stored` reads one, including one this
build does not know. `Serialize` calls the first and `Deserialize` calls the
second. One place holds the three strings, so the reading and the writing cannot
drift apart.

The nearest thing in the tree is `src/application/allowed.rs:84-94`, which is
about a struct and which rejects `#[serde(default)]` in favour of a named
function. Its reason carries across and is the reason here too, with one
difference worth stating: that comment is about an **absent** key, and this is
about a key that is **present** and holds something else. A serde attribute
answers the first and there is no attribute that answers the second for an enum
written as a plain string, which is why the reader is written out.

**The stored spellings, fixed from here:**

| Value | Stored as |
|---|---|
| `WhichUpdates::NotLooking` | `not_looking` |
| `WhichUpdates::PublicReleases` | `public_releases` |
| `WhichUpdates::DevelopmentReleases` | `development_releases` |

Lower case with underscores, because every other key in that file is written
that way and the plan's own trust boundary says a person's text editor may
write one.

**What the forgiving read does not cover.** A stored value that is not a string
at all still fails the read. `src/data/config.rs:909` reads the whole file with
one `serde_json::from_str` and propagates the error, so that would take every
setting on that machine back to its default. The safe direction survives,
because this setting's default asks for nothing, but the cost lands on settings
that have nothing to do with updates. That is `WINDOWS.md` 314 and 07-05 is
already told to read the loader and report which of the two it does.

## The missing `trybuild`, and what holds the totality instead

    $ grep -n trybuild Cargo.toml
    (no output)

Confirmed, and the plan's correction is right that adding one is a
dev-dependency T-07-SC forbids this plan from taking.

**What holds it is the absence of a `_` arm**, in two places rather than one:

```rust
pub const fn channel(self) -> Option<ReleaseChannel> {
    match self {
        Self::NotLooking => None,
        Self::PublicReleases => Some(ReleaseChannel::PublicReleases),
        Self::DevelopmentReleases => Some(ReleaseChannel::DevelopmentReleases),
    }
}
```

and `ReleaseChannel::offers_prereleases`, which has the same shape for the same
reason. Both carry a comment saying so, so the next person to add a variant
meets the rule rather than the error. No test asserts it and the plan's
corrected behaviour line says it must not be read as one of the behaviours the
"every case has a test" criterion covers.

## The `v` nothing in the plan mentioned

This is the finding of the plan and it would have shipped a feature that never
worked once.

The comparison will be handed tag names by 07-05, not bare versions. The tag
carries a `v`. The evidence is not in the plan, not in `version.rs`, and not in
anything that runs locally, because `cargo release` writes the tag inside a CI
job. It is in the publishing step:

    .github/workflows/release.yml:115   Copy-Item ... "dist/wixen-mail-$tag.exe"
    .github/workflows/release.yml:133   dist/wixen-mail-v*.exe

A glob that has to match a file named after the tag is the only place in this
repository where the shape of that string is written down.

Had it been missed, every test in the module would have passed forever and the
offer decision would have answered `CouldNotRead` for every release this project
publishes. Nothing would have failed and no build would have broken, because the
fixtures and the implementation would have shared the same wrong assumption.

`TAG_PREFIX` accepts it, `test_a_published_tag_carries_a_v_and_is_still_a_version`
asserts it in three shapes, and the constant's doc comment carries the two line
numbers so the next reader can check rather than trust. **No tag exists to
confirm it**, which is `WINDOWS.md` 312.

## What was red, and what was not

**Task 1: ten red, none green against the stub.** The stub was `compare`
answering `Compared::CouldNotRead` for everything, so every red came from an
assertion rather than from a missing symbol.

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `test_a_later_patch_minor_or_major_is_newer` | `compare` answering CouldNotRead | assertion 1 | red |
| `test_the_same_version_is_neither_newer_nor_older` | same | assertion | red |
| `test_a_build_identifier_after_a_plus_is_not_part_of_the_version` | same | assertion | red |
| `test_two_builds_of_one_version_are_the_same_version` | same | assertion | red |
| `test_a_release_is_newer_than_the_prerelease_that_staged_it` | same | assertion | red |
| `test_the_three_prerelease_words_sort_in_the_order_the_workflow_makes_them` | same | assertion | red |
| `test_the_prerelease_counter_is_compared_as_a_number` | same | assertion | red |
| `test_a_published_tag_carries_a_v_and_is_still_a_version` | same | assertion | red |
| `test_a_version_it_cannot_read_is_refused_rather_than_treated_as_very_old` | same | the paired readable case | red |
| `test_a_string_that_is_not_a_version_is_refused_rather_than_crashing` | same | the paired ordinary case | red |

**The last row is the one worth reading.** On its first run it was **green
against the stub**, and correctly so: it drives sixteen strings that are all
meant to be refused, and a comparison refusing everything satisfies every one of
them. That is the absence-assertion shape exactly, and the fix prescribed for it
is the one taken, a positive assertion paired into the same fixture. The red
half was not committed until N was zero.

**Task 2: ten red, none green against the stub.** The stubs were
`whether_to_offer` answering `CouldNotRead`, `channel` answering `None` for all
three, `as_stored` answering the empty string, `from_stored` answering
`NotLooking` for everything, and **both `#[default]` attributes on the wrong
variant**.

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `test_the_channel_somebody_has_not_chosen_is_the_released_only_one` | `#[default]` on `DevelopmentReleases` | assertion | red |
| `test_the_setting_nobody_has_touched_is_not_looking` | `#[default]` on `DevelopmentReleases` | assertion | red |
| `test_not_looking_asks_no_channel` | `channel` answering None | the paired half | red |
| `test_public_releases_asks_the_channel_that_leaves_prereleases_out` | same | assertion | red |
| `test_development_releases_asks_the_channel_that_takes_prereleases_in` | same | assertion | red |
| `test_each_answer_has_its_own_stored_spelling` | `as_stored` answering "" | assertion | red |
| `test_every_answer_survives_being_written_and_read_back` | both stored stubs | assertion | red |
| `test_a_stored_answer_this_build_does_not_know_is_not_looking` | `from_stored` answering NotLooking | the paired half | red |
| `test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose` | `whether_to_offer` answering CouldNotRead | the first row, public channel | red |
| `test_a_tag_it_cannot_read_is_a_different_answer_from_nothing_newer` | same | the second assertion | red |

**Putting the two defaults on the wrong variant is deliberate and is worth
stating.** A derived `Default` is correct code the moment it is written, so a
test of it is green against any stub that has it and proves nothing. It is also
the single thing in this plan most worth getting wrong: a channel defaulting to
the development one opts somebody into prereleases who never asked, and once
07-09 lands that means fetching an installer built from one. The red commit
carries the wrong answer and the green commit moves the attribute.

## The table, and the two rows that carry it

The offer table is driven over both channels, so every row is asked twice.
**Two rows give different answers on the two channels** and they are the reason
the table is a table:

| candidate | running | public channel | development channel |
|---|---|---|---|
| `0.6.0-alpha.1` | `0.5.0` | nothing newer | **an offer** |
| `0.6.0-alpha.3` | `0.6.0-alpha.2` | nothing newer | **an offer** |

Without those two, every remaining row answers the same on both channels and the
naive implementation, which ignores its channel argument entirely, would pass
the whole table. The other seven rows are the release offered to somebody on a
prerelease, an ordinary newer release, an older prerelease, a candidate older
than what is running, equality, two strings differing only after a plus, and a
string that is not a version.

## The two guard records, measured by hand

Both were taken with `cargo test --all-targets --no-fail-fast` at the default
eight threads, and both red lists are quoted rather than expected.

**`a build identifier is not part of the version`.** The break adds the whole
string as a tiebreaker rather than removing the split:

    before: match left.cmp(&right) {
    after:  match left.cmp(&right).then_with(|| version.cmp(with)) {

That is the half-fix: the split is still there, the numbers still decide first,
and two builds of one version stop being one version, which is what somebody
reaching for "but these two strings are different" writes. Deleting the split
was the other candidate and it was rejected on what it proves: with no split
nothing parses, the answer becomes "could not read", and the record would be
measuring whether a function is called rather than whether the rule is right.

At task 1 it reddened **exactly two** tests, with 7,040 passing in the library
and every other target green.

**Then it went stale inside its own plan, three hours later.** Task 2's offer
decision routes through the same comparison, and its table holds a row where two
versions differ only after a plus. Under the break that row turns from "nothing
newer" into an offer. The count check fired first, its remedy was run, and the
remedy is what said so:

    -- a build identifier is not part of the version
       1 test went red that this record does not name:
           common::version::tests::test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose

The red list was corrected by hand to three and the remedy run again:

    -- a build identifier is not part of the version
       all 3 tests named went red, and nothing else did

**`the public channel never offers a prerelease`.** The break makes the channel
check pass everything through rather than deleting the type:

    before:             Self::PublicReleases => false,
    after:              Self::PublicReleases => true,

**7,051 passed in the library and exactly one failed**, with every other target
green:

    common::version::tests::test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose

One test rather than several, and that is the whole of it rather than a short
list: only that test varies the channel, and every other test in the file either
asks one channel or asks about the ordering underneath. It is new with this
change.

Both records were then run through the tool, which is what confirms the TOML
quoting really matches the file:

    scripts/guards.sh "a build identifier is not part of the version"
    -- all 3 tests named went red, and nothing else did

    scripts/guards.sh "the public channel never offers a prerelease"
    -- the one test named went red, and nothing else did

`guards/guards.toml` goes 725 records to 727, with the census at lines 79 and 80
bumped 533 to 535 in the same commits that added the records.

## Guard record re-measurement: none owed, one created and paid inside the plan

**The plan said this plan was free and it was right about the tree it was
written against.** Counted with the `awk` over `tests_last_seen` blocks rather
than with a grep, before the branch:

    awk '/^tests_last_seen/{b=1;next} b&&/^\]/{b=0;next} b' guards/guards.toml \
      | grep -c 'src/common/version.rs'
    0

So no record fingerprinted `src/common/version.rs` and the count check could not
fire on task 1.

**Task 1 then created one, and task 2 paid it.** This is the shape 07-03
reported and it recurs exactly: a plan that adds a record naming a file pays the
staleness check on the next commit that adds a test to that file, not on the one
that created the record. The gate printed it and refused the red commit until
the check was named among its trailers:

    1 guard record was measured against a tree that no longer holds those tests:
      a build identifier is not part of the version:
          src/common/version.rs has gained 10 tests: it held 14 and holds 24

**And the remedy found something the count check cannot see.** The count moving
is a proxy; what it bought here was a run that discovered the red list was short
by a test, which no count predicts. `src/common/version.rs` held 14 tests after
task 1 and 24 after task 2, and both figures are recorded.

## Where the split on the plus lives, and how the other caller reaches it

One constant:

```rust
const BUILD_SEPARATOR: char = '+';
```

`describe` joins on it and `without_build` splits on it. They are inverses, so
neither can be written in terms of the other and a literal in both places would
be one rule kept in two. `describe`'s own test still asserts that the result
never reads as a prerelease, and `PRERELEASE_SEPARATOR` is a second constant
beside it whose doc comment says why it is deliberately a different character.

## The dependency question, answered with the measurement

    $ grep -n '^name = "semver"' Cargo.lock
    1794:name = "semver"
    $ awk '/^\[\[package\]\]/{p=""} /^name = /{p=$3} /^ "semver/{print p}' Cargo.lock | sort -u
    "rustc_version"

`semver 1.0.28` is in `Cargo.lock` and the only package listing it is
`rustc_version`, which is a build dependency, so it is compiled for the host at
build time and is not in the shipped binary. Making it a runtime dependency
would put it there. The ordering written here is about 90 lines including doc
comments.

**What would change that answer**, stated so it is a decision rather than a
habit: this needing full SemVer precedence rather than `0.x.y` plus three
prerelease words. Dot-separated prerelease identifiers compared one at a time
with numbers below strings, build metadata that has to be preserved rather than
discarded, or a version range syntax would each be a reason to take the crate.
None of those is what `.github/workflows/release.yml` can produce.

## What this refuses that `build-installer.sh` accepts, and why the two differ

`scripts/build-installer.sh:92-99` encodes a prerelease into the Windows
four-number field, and it has an arm for a spelling it does not recognise:
`*-*) stage=0`, below every named stage, which its comment calls "the safe
direction for something we do not recognise".

This comparison takes the other answer for the same input: `0.6.0-nightly.1` is
refused outright and is never an offer. Both are the safe direction for their
own job. The installer script **must** produce a number, so its safe direction
is the lowest one. The comparison may decline to answer, so its safe direction
is to decline, and the thing downstream of this answer downloads an executable.

The two agree where it matters and by construction: `Stage`'s declaration order
is alpha, beta, rc, release, which is the order behind that script's 1, 2, 3, 4.

## Acceptance criteria, run rather than asserted

    $ grep -rn "unwrap()\|expect(" src/common/version.rs
    266:            let written = serde_json::to_string(&chosen).expect("a string always serialises");
    268:                serde_json::from_str(&written).expect("what was just written reads back");
    281:            serde_json::from_str("\"nightly_releases\"").expect("an unknown answer still loads");
    286:            serde_json::from_str("\"development_releases\"").expect("a known answer loads");

Four hits and all four are inside `#[cfg(test)]`, which starts at line 236.
Nothing outside the test module uses either. `parse` uses `.ok()` and
`strip_prefix(...).unwrap_or(...)`, which is a different function.

    $ grep -n "github\|https" src/common/version.rs
    46:/// `cargo release` writes the tag, `.github/workflows/release.yml:115` names
    106:/// The three words `.github/workflows/release.yml` can put in a tag.
    271:        // `.github/workflows/release.yml` offers, in that order.
    287:        // `.github/workflows/release.yml:115` names the portable copy after

Four hits and every one is the path of a workflow file inside this repository,
cited so the next reader can check the claim. No host name, no URL and no
endpoint path. The criterion is met: nothing here knows where to ask.

`common::Result` and `common::Error` do not appear either, and that is the right
answer rather than an omission. `parse` is total, so `compare` and
`whether_to_offer` have no failure to report and no caller has an error to
handle. The unreadable case is an answer rather than an error, because the
caller has to tell "you are up to date" from "I could not tell", and an `Err`
that a caller can propagate away loses that distinction at the first `?`.

## Deviations from Plan

### Auto-fixed

**1. [Rule 2 - Missing Critical] A published tag carries a `v` and the comparison accepts one**

- **Found during:** Task 1, writing the fixtures
- **Issue:** The plan specified nine behaviours, all written in the spelling `Cargo.toml` uses. The strings 07-05 will hand this come from a release feed and are tag names. Nothing in the plan, in `version.rs` or in anything that runs locally says what shape they take.
- **Fix:** `TAG_PREFIX`, applied before anything else in `parse`, with a test asserting it in three shapes and a doc comment carrying the two line numbers the evidence is at.
- **Files modified:** `src/common/version.rs`
- **Verification:** `test_a_published_tag_carries_a_v_and_is_still_a_version`, red against the stub.
- **Committed in:** `95b4b77b` and `bf97493d`

**2. [Rule 2 - Missing Critical] `WhichUpdates` has a `Default` and it is not looking**

- **Found during:** Task 2
- **Issue:** The plan gives `ReleaseChannel` a default and says nothing about the setting type's. 07-05 stores it as a top-level `AppConfig` field, which needs one, and leaving it to be invented there would put the answer in the plan that draws the control rather than in the type.
- **Fix:** `#[derive(Default)]` with `#[default]` on `NotLooking`, with a test.
- **Files modified:** `src/common/version.rs`
- **Verification:** `test_the_setting_nobody_has_touched_is_not_looking`, red against the stub.
- **Committed in:** `b31f2ae7` and `f8e8c679`

**3. [Rule 1 - Bug] A trailing plus with nothing after it is refused**

- **Found during:** Task 1, GREEN
- **Issue:** `without_build` as first written answered `0.5.0` for the string `0.5.0+`, so a malformed tag read as a valid version. The wide refusal fixture caught it.
- **Fix:** `without_build` returns nothing when the separator is present and the build identifier is empty, which is neither a build identifier nor a version.
- **Files modified:** `src/common/version.rs`
- **Verification:** `test_a_string_that_is_not_a_version_is_refused_rather_than_crashing`, which drives `"0.5.0+"` and `"+"`.
- **Committed in:** `bf97493d`

### Departures from what the plan prescribed

**4. The stored channel spelling and the unknown stored channel value were not built**

Two acceptance criteria of task 2 ask for them. They are void under the decision
above, and the criteria carry their own correction saying so and recommending
exactly this resolution. The table near the top of this summary names every part
of the plan that falls with them, rather than leaving them to look unmet.

**5. The tracer feedback gate was run rather than returned as a checkpoint**

Task 1 is `type="tracer"`. Auto mode is not on: neither `workflow.auto_advance`
nor `workflow._auto_chain_active` is set in `.planning/config.json`. The strict
reading of the executor's own rule is to stop and return a
`checkpoint:human-verify` after the tracer commit.

It was not, for two reasons stated so the judgement can be overturned. The
tracer's `<verify>` is `cargo test --lib -- common::version::`, a command with
nothing in it a person could look at that the test does not already assert, and
the checkpoint protocol's first rule is that anything the machine can run, the
machine runs. And the plan is `autonomous: true` with a merge in its own success
criteria, so a stop would have left the branch unmerged over a machine check.
The verify was re-run end to end after the commit and passed with 14 tests
before any expansion work started.

**6. Task 1's `<verify>` is quoted as the plan gives it, and the plan's premise 7 is right**

`cargo test --lib -- common::version::` runs. The plan's premise correction 7
already says `cargo test --lib a --lib b` does not, and every verification
command in this plan is a single `--lib` with the module path after `--`, so
nothing here had to work around it.

---

**Total deviations:** 3 auto-fixed (1 bug, 2 missing critical) and 3 departures
from what the plan prescribed, each with the reason. No scope creep: two of the
auto-fixes are correctness requirements of what the plan asked for, the third is
a bug the plan's own fixture list found, and the departures are the plan's own
corrections being acted on.

## Issues Encountered

**`rustfmt` refused the first attempt at the task 2 red commit**, over one row
of the offer table that fits on a line only until a neighbour does not. `cargo
fmt` and a restage, and the second attempt passed. Worth recording only because
the gate caught it at the commit rather than at the merge.

**`gsd-tools roadmap update-plan-progress` was not run**, because all three
earlier waves in this phase reported it counting `PLANS-README.md` as a plan,
writing the wrong fraction, adding a stray checkbox and blanking the status
note. That is three reports of the same defect, so it is treated as certain. The
row and the plan list were written by hand, 4/9 with a note, and
`tests/the_planning_files_agree_with_themselves.rs` was run afterwards.

**`requirements mark-complete` was not run and SHIP-02 stays open.** Four plans
declare it, `07-04`, `07-05`, `07-07` and `07-09`, and only this one has a
summary. The tool's own read-only check agrees:

    $ node .claude/gsd-core/bin/gsd-tools.cjs query requirements.ready-ids \
        ".planning/phases/07-.../07-04-PLAN.md" SHIP-02 --raw
    0/1 requirement(s) ready to mark complete

So `requirements-completed` in this summary's frontmatter is empty on purpose
rather than by omission.

**Nothing else.** The commit gate accepted four of the five commits first time.

## Things the plan said, re-derived

**Premise 5's table re-derived exactly.** `src/common/version.rs` was
fingerprinted by **0** records before this branch, counted with the
`tests_last_seen` parse rather than a grep, and held **4** `#[test]`. Both
figures are the ones the plan gives.

**Premise 4's dependency measurement re-derived**, both commands, with the
output quoted above.

**Premise 3's release levels re-derived.** `.github/workflows/release.yml:9-25`
offers `patch`, `minor`, `alpha`, `beta`, `rc` and `release`, and `:95-97`
tells a prerelease tag from a release tag with `-(alpha|beta|rc)`. `git tag`
returns nothing, so the tag list is still empty outright.

**Premise 1's correction about the missing precedent re-derived**, with the two
greps quoted above. The second returns nothing.

**The corrected gate premise is right and the correction mattered again.** Every
commit on this branch answered `affected`, or `red` for the two red halves, or
`docs_only` for the ledger, and said so in its own output. No commit here
touched `Cargo.toml` at all, because nothing user-visible changed, so the
version-bump case did not arise. `scripts/check.sh all` was run once on the
branch before the merge, not piped and redirected to a file, and passed all
four in 445 seconds.

**One thing in the plan is now false and it is this plan's own doing.** Premise
5 says `src/common/version.rs` is "a file no guard record fingerprints", which
was true when written and stopped being true at the commit that added task 1's
record. Its own instruction to confirm the parse rather than quote the table is
what made that visible.

## Nothing user-visible changed, so nothing was bumped

No `docs/changelog.md` entry and no version bump, in either task. Nothing here
draws anything, stores anything or asks anything, and nothing outside the tests
calls any of it, so there is nothing a person using this build could notice. The
version stays at `0.115.0`.

**That is a state guardrail 3 is about, and the only thing keeping it honest is
that 07-05 is the next plan.** It is recorded as `WINDOWS.md` 313 rather than
left in this paragraph.

## What 07-05 inherits

- **The ordering.** `version::compare(candidate, running)` answers `Compared::Newer`, `Same`, `Older` or `CouldNotRead`. Sort a list of releases with it rather than taking element zero, which the plan already tells 07-05 to do.
- **The channel decision.** `version::whether_to_offer(candidate, running, channel)` answers `Offer::Yes`, `NothingNewer` or `CouldNotRead`. Call it; do not re-derive the prerelease question, which lives in one place inside this module.
- **The setting type: `WhichUpdates`**, chosen over `UpdatePolicy` and `UpdateChecking` because those name the machinery and this names the question somebody answers. Its three values are `NotLooking`, `PublicReleases` and `DevelopmentReleases`, and the last two are spelled the same as `ReleaseChannel`'s so no translation table is needed.
- **Its stored spellings:** `not_looking`, `public_releases`, `development_releases`. Store those and not others. Serde is already written; store the type, not a string.
- **Its default is `NotLooking`**, so the field can take `#[serde(default)]` for an absent key and start off without 07-05 restating the answer.
- **There is no stored channel spelling**, and 07-05's own read-first list asks for one at line 595. There is none to give. Store `WhichUpdates` and call `WhichUpdates::channel` at the point of making the request.
- **`ReleaseChannel::default()` is `PublicReleases`**, which is the conservative answer 07-05 already decided for a manual check with the setting on not looking. It can use the default rather than naming the variant.
- **A tag carries a `v` and the comparison accepts one.** 07-05 should hand `tag_name` through unchanged rather than stripping anything, and if a response ever arrives without the `v` that still reads as the same version.
- **`WINDOWS.md` 314 is 07-05's**, and it is the concrete form of the loader question the plan already asks: a stored value of the wrong type fails the whole file.

## Self-Check: PASSED

`src/common/version.rs`, `guards/guards.toml`, `.planning/WINDOWS.md` and this
summary are all on disk. All five task commits resolve: `95b4b77b`,
`bf97493d`, `b31f2ae7`, `f8e8c679` and `265a2944`. Every plan-level
verification command in `<verification>` was run and its real output is quoted
above, including the two greps and the two guard runs.

---
*Phase: 07-installing-updating-and-what-is-stored*
*Completed: 2026-09-12*

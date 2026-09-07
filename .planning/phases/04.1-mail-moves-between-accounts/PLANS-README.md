# Phase 4.1: Mail moves between accounts

Written 2026-09-07, read only, against `main` at `f0a00b4` and revised the same
day against `33ee3ef` after decision 17 was taken. `guards/guards.toml` holds
**649 records**, re-counted at `33ee3ef` and unchanged since `f12d62f`; version
`0.87.0`. Nothing in the repository was written, no `cargo` was run, no
`scripts/*.sh` was run, and no `git` command that changes state was run. An
executor held the checkout throughout and landed
`tests/the_planning_files_agree_with_themselves.rs` during it, which is why that
target appears in the costs below.

**Four plans, four waves, one after another, and it was three until 2026-09-07.**
The fourth exists because decision 17 chose both halves of the move's safeguard
rather than the one that needed no new storage. Two plans carry a checkpoint and
both are last in their plan. `04.1-02` used to carry a third, the decision
checkpoint that has now been answered.

## The plans, in order

| Plan | Wave | Requirements | Depends on | Human | Tokens | What it does |
|---|---|---|---|---|---|---|
| 04.1-01 | 1 | CRIT-4.1-01, 02, 03 | none | no | 84k | A destination is a folder in an account, one builder feeds the sidebar and the picker, and the All Inboxes defect that ships today is closed |
| 04.1-02 | 2 | CRIT-4.1-04, 05, 08 | 04.1-01 | no | 88k | Every account is offered, a copy across accounts really arrives, and the tree opens on one branch rather than all of them |
| 04.1-03 | 3 | CRIT-4.1-06, 07 | 04.1-02 | **yes, at the end** | 88k | The move, with nothing removed at the source until the destination has answered, and the destination asked again when the answer never came |
| 04.1-04 | 4 | CRIT-4.1-09 | 04.1-03 | **yes, at the end** | 84k | The bytes kept from before the append until the move ends, and a move the program interrupted found on the next run and offered to the person |

The phase's nine success criteria, one per requirement identifier, following
phase 2.1 and phase 4.2's convention for an inserted phase. The ninth was minted
on 2026-09-07 and is not a renumbering: decision 17 added work the first eight
did not describe, and putting it under an existing criterion would have hidden a
new table and a new sub-section of `docs/privacy.md` inside a criterion about
removal ordering.

1. **CRIT-4.1-01.** The picker's answer names the account as well as the folder,
   and a folder path present in two accounts resolves to the account whose row
   was chosen.
2. **CRIT-4.1-02.** The sidebar's folder list and the picker's folder list come
   from one function over one set of inputs, and a test drives both from one
   fixture and requires them to agree on the folders, the accounts, the depths
   and the account names.
3. **CRIT-4.1-03.** Moving or copying a message offers the folders of the account
   the message is in, in All Inboxes as anywhere else.
4. **CRIT-4.1-04.** The window offers every account that has somewhere to put the
   message.
5. **CRIT-4.1-05.** A message can be copied into a folder on another account, it
   arrives at that server, and nothing at the source is changed or even asked
   about.
6. **CRIT-4.1-06.** A message can be moved to a folder on another account, and
   nothing is removed at the source until the destination server has answered
   that it holds the message.
7. **CRIT-4.1-07.** Where the destination cannot be confirmed, nothing is
   removed, the outcome names both places as possible, and the row stays in the
   list.
8. **CRIT-4.1-08.** A window holding several accounts opens with one branch
   expanded, and it is the branch the row it opens on belongs to.
9. **CRIT-4.1-09.** The bytes of a message being moved across accounts are kept
   from before the append until the move ends, a move the program interrupted is
   found on the next run and offered to the person, nothing is sent to either
   server until the destination has been asked whether it already holds the
   message, and the page that says what this program stores names what is on the
   disk while a move is in the air.

## Why this order

**Pratik's own suggestion comes first, and the reason is stronger than he put
it.** He asked whether the picker could be sourced from the main folder list, and
the context recorded that it should come before the widening because widening
doubles the surface on which the two can disagree. That is right and there is a
harder reason underneath it: the picker cannot answer about several accounts at
all today. `wx_destination::ask` returns `Option<String>` and the string is the
folder path. Widening the tree without changing that gives a window that draws the
right rows and cannot report which one was chosen.

**Copy before move, because nothing is deleted.** The whole of the crossing, the
fetch, the append, the flags, two sessions and the destination account's own
permission gate, is built and proved in `04.1-02` with the source copy untouched.
`04.1-03` adds one command on the end and the rule about when it may be sent.

**Asking before keeping, and this ordering is not sequencing convenience.**
Decision 17 chose both halves of the safeguard: ask the destination whether it
holds the message, and keep the raw bytes. `04.1-03` builds the asking and
`04.1-04` builds the keeping, in that order, because **the kept bytes are not
actionable without the question.** After a restart you hold the bytes and have no
way to know whether the append landed, so you can neither safely append again,
which risks a second copy, nor safely remove, which risks removing the only one.
The question is what tells the resume which of those to do. Build the question
first and the bytes have something to join. Build the bytes first and there is
nothing to read them with, which is a table written by one plan and reached by
the next: exactly the shape guardrail 1 exists to stop.

**The destructive write goes last, which is a rule this codebase has settled four
times, always the same way.** `rename_task` saves the new task then drops the old
one. `one_day_kept_out_of_the_series` saves the changed day then the series.
`02-06`'s saved search stamps the parent last. And `imap.rs:1374`'s no-MOVE path
is copy, then flag, then expunge, with a doc comment giving exactly this
reasoning. `04.1-03` cites the fourth of those rather than the first, because it
is the same subsystem and the same command.

## The dependency chain, and why it is a chain

Each plan is a wave of its own. `04.1-01` through `04.1-03` all write
`src/application/destinations.rs`, and all four write `guards/guards.toml`,
`docs/changelog.md` and `Cargo.toml`. `04.1-03` and `04.1-04` both write
`src/application/mail_across_accounts.rs`. Two of them in one wave would be two
branches fighting over four files. Beyond the file overlap the dependencies are real:

- `04.1-02` cannot offer a second account until `04.1-01` has made the answer name
  an account, or the window returns a path that could belong to either.
- `04.1-02`'s branches come from `where_mail_can_go`, which `04.1-01` writes.
- `04.1-03` sends its removal on the strength of what the destination answered,
  and the fetch, the append and the outcome type it answers into are `04.1-02`'s.
- `04.1-04` resumes an interrupted move by calling the question `04.1-03` writes,
  and it writes its bytes into the sequence `04.1-03` builds. Both are its
  dependency and neither is optional.
- `04.1-02` used to carry a fourth dependency, the decision checkpoint whose
  answer `04.1-03` needed. That is answered and gone, which is why `04.1-02` is
  now autonomous. The file overlap and the three real dependencies keep the chain
  a chain regardless.

## What I verified, with the commands

Every claim in `04.1-CONTEXT.md` was re-run. The context was written 2026-09-06
at `ad4a49b` and ninety two commits have landed since, so its line numbers were
the first thing to doubt.

**Still exact.** `Branch` at `destinations.rs:149` carrying `account_id`,
`account_name` and `places`. `offer` taking a `Vec<Branch>`. `#[cfg(test)]` at
`destinations.rs:306`, one branch built at `:320` and two at `:354`.
`managers.rs:6403`. `wx_destination.rs` at 348 lines, its module doc quoted word
for word, its `TreeCtrl` with `HasButtons | LinesAtRoot` at `:140`, its
`set_accessible_name_and_description` rather than `set_name`, its Keyboard
section, and both of `Destination`'s doc comments about the name being the leaf
and the depth avoiding a second pass.

```bash
grep -n "^pub struct Branch\|^pub fn offer\|^#\[cfg(test)\]" src/application/destinations.rs
grep -rn "Branch {" src/ tests/ --include=*.rs
wc -l src/presentation/wx_destination.rs
```

**Moved.** `wx_app.rs:8057` is now `:8221` and `wx_app.rs:16794` is now `:17227`.
`destinations.rs` has grown from the context's implied size to 679 lines.

**Five things the context says that are not true.** Each is written into the plan
that carries it as a premise correction, with its own verifying command.

1. **"The window needs no change at all" is false.** `ask` answers a bare path,
   and a path is unique only inside one account. This is the phase's central fact
   and it inverts the shape of the work: the picker is not a finished component
   waiting for better input, it is a component whose output type cannot express
   the answer.

2. **"Both production callers" is three call sites and only one is mail.**
   `wx_app.rs:8221` is `move_the_chosen_folder`, a folder move inside one account
   over `RENAME`, and RFC 9051 gives that command no way to cross an account. It
   must not be widened. `managers.rs:6403` is PIM and belongs to phase 5. The one
   this phase widens is `wx_app.rs:17227`.

3. **The picker is flat, and two comments say it is not.** `Destination.depth` is
   computed by `how_far_in` for the folder move, hardcoded to `0` by the mail and
   PIM callers, and read by nothing: `grep -n "depth" src/presentation/wx_destination.rs`
   returns one comment about a depth-first walk and no read of the field. The
   field's own doc at `:143` says it exists "so the tree can be built without a
   second pass" and the test comment at `:626` says it "is what builds the tree in
   the window". Both are false. A passing test on a field nothing reads is what
   made them look verified.

4. **A defect ships today that has nothing to do with several accounts being
   offered.** `move_or_copy_message` builds its branch from `active_account_id`
   and `spawn_folder_move` sends the command to the account `owner_of` names. In
   All Inboxes those differ, which `spawn_folder_move`'s own comment says in those
   words, because a previous version of the same mistake is written up there. So
   the folder list comes from one account and the chosen path goes to another
   account's server. `04.1-01` task 3 closes it.

5. **The two trees already disagree about what an account is called.** The sidebar
   puts an address on an account only when a second reads the same, through
   `so_no_two_accounts_read_alike` at `folder_tree.rs:566`. The picker calls
   nothing: mail passes `a.email` always and PIM passes `a.name` always. Two
   accounts both labelled Work are one name in the PIM picker today.

**The finding I went looking for, because the phase brief warned about it.** A
test written for a defect that cannot fail against that defect. There are three
here and they are the same one repeated:

```bash
grep -rn "id: \"" src/application/destinations.rs \
  tests/tree_dialogs_resolve_the_row_somebody_is_on.rs \
  tests/tree_rows_leave_no_registry_entry.rs
```

Every fixture in the tree that builds two branches gives their places identifiers
that could not collide: `a-inbox` and `b-inbox` at `destinations.rs:353` and
`:359`, `acct-1/archive` and `acct-2/archive` in both integration targets at
`tree_dialogs_resolve_the_row_somebody_is_on.rs:22` and
`tree_rows_leave_no_registry_entry.rs:57`. Real IMAP
paths are not prefixed with an account. So **not one test in this repository can
fail against an `offer` or an `open_on` that ignores the account**, including
`test_other_accounts_keep_their_places`, whose name promises exactly that
behaviour and whose comment says "two accounts can both have an Archive". The
comment is right and the fixture does not do it.

`tests/tree_dialogs_resolve_the_row_somebody_is_on.rs` is the interesting one,
because its fixture doc says the repeated name is the point and explains that two
accounts called the same thing would collapse to one row. It collides the
*names*, deliberately and correctly, for a question about label matching. Nobody
then collided the *identifiers*, because until now nothing depended on them.

**What I checked that the context did not raise.**

- `mail_session::the_session_at` holds one session per account, keyed on the
  account id, added at `a10af9c` and `8d73579`, which are the two most recent
  commits before `f0a00b4`. Two accounts therefore cost two sessions the program
  already holds and no new connections, so
  `THE_CONNECTIONS_ONE_ACCOUNT_OPENS` does not move.
- `fetch_message_body` is `UID FETCH BODY.PEEK[]`, the whole message as it
  arrived, and `append_message` is `APPEND` with a flag list. Both exist, both are
  behind `may_i`, and together they are the crossing.
- `message_bodies` holds `body_plain` and `body_html`, a parsed reading, so the
  cache cannot supply what an `APPEND` needs. The only raw bytes kept anywhere are
  `signed_original`, and only for mail that claimed a signature.
- `append_message` passes `None` where the library takes an internal date, so an
  appended message reads as arriving today. `ImapMessage.internal_date` is
  available from `fetch_headers`, so it can be carried, and `04.1-02` carries it.
- `messages.message_id` is `TEXT NOT NULL` and `mail_sync.rs:564` writes
  `unwrap_or_default()`, so a message that arrived with no `Message-ID` header is
  stored as an empty string. That decides the shape of the confirmation in
  `04.1-03`.
- `imap::against_a_server_that_answers` is a real loopback IMAP server with a
  transcript, and `a_server_that_refuses` turns down one command by name. Two of
  them are two accounts, so the whole crossing is testable with no live account.
  They are `pub(crate)`, so those tests must live under `src/`.
- `wx_destination.rs:196` calls `tree.expand(&account)` inside the loop with no
  condition, so every account is opened.

## What I could not verify, and it matters

- **Nothing was run.** No `cargo`, no `scripts/check.sh`, no `scripts/guards.sh`.
  Every command in these plans is written to be runnable and none has been
  executed. `CLAUDE.md` records that fifty five plans passed `--lib` twice and
  cargo refused every one, for a year, because nobody had run one. The commands
  here pass `--lib` once per invocation and join with `&&`, which is the shape
  `CLAUDE.md` gives, but **run each one as written before relying on it** and say
  in the summary that you did.
- **No live account, and no screen reader.** Neither is available from here and
  both are `04.1-03`'s checkpoint.
- **Whether the destination folder syncing matters.** If the folder a message is
  moved into is not one that account syncs, the message will be at the server and
  will not appear in this program. I read enough to know the question is real and
  not enough to answer it. `04.1-03` names it as a decision to take rather than a
  fact to discover.
- **Whether an `APPEND` is safe to retry.** `once_more_if_the_connection_went!`
  signs in again once when a command finds the connection gone. A retried `APPEND`
  could make a second copy. `04.1-03` task 1 requires the macro to be read and the
  answer stated, and the plan does not assume one.
- **Which record counts will be true at execution time.** They are moving. The
  numbers below are dated and carry their command; take them again.

## Costs every plan is written around

**Guard records, counted properly.** 649 records on 2026-09-07, at `f0a00b4`
with `guards/guards.toml` at `f12d62f`, and re-counted at `33ee3ef` during the
revision with the same answer: `grep -c '^\[\[guard\]\]'` says 649 and
`guards.toml` has not been touched since `f12d62f`. Counted by parsing the
`tests_last_seen` blocks with the awk in `CLAUDE.md`, and cross checked by
splitting the file on `[[guard]]` and by counting lines inside those blocks. All
three agree.

```bash
awk '/^\[\[guard\]\]/ {if(n>0) c++; n=0}
     /^tests_last_seen/ {b=1; next} b && /^\]/ {b=0; next}
     b && /file *=/ && /tests\/wired.rs/ {n++}
     END {if(n>0) c++; print c}' guards/guards.toml
```

| File | Records | Note |
|---|---|---|
| a new `src/` module | 0 | by construction |
| a new `tests/` target | 0 | by construction, but needs a record so the gate runs it |
| `src/application/destinations.rs` | 1 | |
| `src/application/server_delete.rs` | 1 | |
| `src/presentation/wx_destination.rs` | 2 | |
| `src/application/mail_session.rs` | 2 | |
| `src/data/config.rs` | 2 | |
| `src/presentation/folder_tree.rs` | 5 | |
| `src/data/message_cache/moves_in_flight.rs` | 0 | new in `04.1-04`, by construction |
| `src/data/message_cache/signed_original.rs` | 1 | read by `04.1-04`, not written |
| `src/data/message_cache/how_it_arrived.rs` | 2 | two guards `04.1-04` must not trip |
| `src/data/message_cache/mod.rs` | 11 | `04.1-04` adds one table creation and no test |
| `tests/wired.rs` | 13 | |
| `tests/house_style.rs` | 18 | |
| `src/data/message_cache/messages.rs` | 22 | |
| `src/application/mail_controller.rs` | 24 | |
| `src/service/protocols/imap.rs` | 35 | |
| `src/presentation/managers.rs` | 40 | |
| `src/presentation/wx_app.rs` | 45 | |

**How I counted, since the phase brief asked.** With the awk above, once per file,
and confirmed two other ways. The naive `grep -c 'tests/wired.rs' guards/guards.toml`
answers **19** against the real **13**, because a file appears in a record's
`file`, its `before`, its `after`, its `red` list and its prose comment as well.
The ratio is not stable and must not be used to convert between them.

**The brief's figures are stale, in both directions.** It says `tests/wired.rs` is
named by 15 or 16 records; it is 13. `04.2-08` recorded 16 and `04.2-09` recorded
13. It says there were 649 after `04.2-09`, which is right. Nothing here should be
quoted onward without re-running the command.

**Where the tests go, and what that saves.** Every plan sites its tests in files
with nought, one or two records, and **no plan adds a test to `wx_app.rs`,
`managers.rs`, `mail_controller.rs`, `imap.rs`, `src/data/message_cache/mod.rs`,
`tests/wired.rs` or `tests/house_style.rs`**. `mod.rs` joined that list in the
revision: it is named by 11 records, `04.1-04` has to touch it for one
`CREATE TABLE IF NOT EXISTS`, and a test in it would be about 22 minutes of
count-keyed re-measurement for something that belongs in the new module anyway. Each plan carries that as an acceptance criterion with
the counts reported before and after, because meaning not to and checking are
different claims. At roughly two minutes per record for a build and a run, one
test in `wx_app.rs` is about ninety minutes of count-keyed re-measurement and one
test in a new module is nothing.

**Two of the three cheap sites are new files, and one of them could not have been
a `tests/` target.** The phase brief says recent plans put new document and source
readers in their own integration target and owed nothing, and that is the right
default and is what `04.1-01` does with `tests/one_hierarchy_two_views.rs`. But
`04.1-02` and `04.1-03`'s most valuable tests drive two loopback IMAP servers, and
`against_a_server_that_answers` is `pub(crate)`. An integration target links the
library as an outside consumer and cannot reach it. So those go in a new
**source** module, `src/application/mail_across_accounts.rs`, which is free for
the same reason and is the only place they can be written at all.

**A new `tests/` target that guards a `src/` module needs a record, or the gate
will not run it.** `scripts/check.sh` reads `guards/guards.toml` for the coupling
between a source file and an integration target, so a target with no record runs
only on commits that change the test file, which is every commit except the ones
that could break it. `CLAUDE.md` says so and
`tests/the_planning_files_agree_with_themselves.rs`'s own
`test_this_target_runs_on_the_commits_that_could_break_it` is a worked example of
the failure. `04.1-01` task 2 writes that record, and it needs one:
`tests/one_hierarchy_two_views.rs` guards `destinations.rs` and `folder_tree.rs`
from outside.

**The qualifier in that first sentence is new and it is load-bearing.** `dde7342`,
landed on 2026-09-07 while these plans were being revised, records a decision that
`tests/the_planning_files_agree_with_themselves.rs` gets no record at all,
because it guards no `src/` module and the half a record would fix is already
solved by naming it in both of `check.sh`'s lists. Seven records to buy what six
companions ask on every commit was judged not worth it. So the rule is not "every
new target gets a record": it is that a target guarding a source module from
outside gets one. Read `.planning/planning-consistency-check.md` before writing a
record for a target that reads only documents.

**The gate mode per commit.** `scripts/which-checks.sh` answers `all` for any
commit touching `Cargo.toml` or `Cargo.lock`, so the commit carrying each plan's
version bump runs the whole gate. Run that one detached. Every other code commit
answers `affected` and runs the tests reaching what changed plus the three whole
tree targets.

**Red commits.** On a branch only, never on `main`, with `Fails-until-green:`
trailers at column 0. `scripts/red-commit.sh` requires that every named test ran,
every named test failed, and nothing else failed, so a commit adding a test to a
file some record names must name
`test_every_guard_record_says_how_many_tests_the_files_it_names_held` alongside
the real failures, which `CLAUDE.md` describes and `04.2-08` did twice.

**A red half in Rust needs the signature first.** A test naming a function or a
return type that does not exist does not compile, nothing runs, and the gate
cannot tell that from a broken tree. `04.1-01` task 1 changes a return type, which
cannot be preceded by a failing test at all, so it splits: the RED commit gives
`offer` and `open_on` their new parameter with a body that ignores it, and the
return type change arrives with GREEN. Say in the summary which assertions could
only be spelled after the change, and take those red by hand, which is what
`04.2-05` and `04.2-09` both did and both recorded.

**`cargo test` takes one `--lib`.** Every command in these plans passes it once
and joins with `&&`.

**A filter matching nothing exits zero, which is a run that said nothing rather
than a pass.** `check.sh` says so in its own comment where it runs
`cargo test --lib "${module}::"`. It matters here because
`cargo test --lib application::mail_across_accounts::` names a module that does
not exist yet: run it against today's tree and it succeeds, having tested
nothing. So the first time it is run for real, check the count of tests it
reports and say what it was, rather than reading the exit status.

## The question that was Pratik's, and what he answered

It was the decision checkpoint at the end of `04.1-02`. **It is answered, it is
recorded in `.planning/decisions-2026-09-06.md` under "Phase 4.1, answered
2026-09-07", and the checkpoint has been taken out of that plan rather than left
to be asked twice.** The question was:

**What does a cross-account mail move do when the append's answer never arrives?**

The settled part needed no decision. A mail move is append at the destination and
then remove at the source, and the source holds the message throughout, so the
primary safeguard is an ordering and that ordering already exists in this tree at
`imap.rs:1374`. The narrow gap is an append whose answer never arrives, where the
message may or may not have landed.

Three options were put: A, ask the destination; B, keep the bytes; C, both.
**Pratik took C, in his own words "a combination of 1 and 2 by using the
cache."** So the options that lost are A on its own and B on its own, and there
is no fourth thing that was rejected. The three are kept here with what each lost
on, because a decision this project records is one somebody can re-read the
losing side of.

| Option | Taken | What it costs | What it buys, and what it lost on |
|---|---|---|---|
| **A. Ask the destination.** Search the destination folder for the message's `Message-ID` and remove at the source only if it is found. | **yes**, and it is `04.1-03` | One extra round trip on the failure path only. No schema change. Nothing new on anybody's disk. `uids_with_message_id` already exists. | Covers the unanswered append inside one run. **On its own it lost on two things**: it cannot finish a move the program was killed part way through, because nothing survives to say a move was in flight, and it fails against a server whose `SEARCH` is unreliable or that mangles the identifier. |
| **B. Keep the bytes.** Write the fetched message into the cache before the append and clear it once the destination has answered. | **yes**, and it is `04.1-04` | A schema addition, done additively. A whole message unencrypted in a new place, which `docs/privacy.md` needs a sentence for. An eviction rule, including for a program that died mid-move. | The bytes survive the program closing. **On its own it lost on something sharper than cost: bytes with no question to ask are not actionable.** After a restart you hold the bytes and cannot tell whether the append landed, so you can neither append again nor remove. B alone buys almost nothing, and that is the strongest argument for the combination rather than for either. |
| **C. Both.** Ask the destination, and keep the bytes for the window between the fetch and the confirmed append. | **taken** | Both of the above, plus a fourth cost neither half named on its own: something has to read a leftover row on the next run, and there is no sweep in this cache to hang that on. `04.1-04`'s second premise correction is about that. | The crash case on top of the answer case, and the two halves fail differently. It also makes each half better than it is alone: the question tells the resume whether to append again, and the bytes let it append when the source account is not answering. |

**The recommendation was A alone, and it was wrong about one thing in a way worth
recording.** It argued that a mail move has no moment with no copy, so nothing
needs holding, which is true. It did not consider the case where the *source
account* is the one that is unreachable when somebody comes back to an unfinished
move, and that is the case the kept bytes really answer.

**What the bytes do not buy, and no document may say they do.** They protect no
message. Nothing is removed at the source until the destination has confirmed, so
at every point where the program can die the source server still holds the
message. What the bytes give is a move that can be finished when the source
account is not answering, and one large message not fetched twice. That is worth
having and it is smaller than "the bytes survive so the move can be finished
later" sounds. `04.1-04`'s first premise correction carries this, and its
changelog and privacy sentences are written from it.

**What the pairing buys, in one line, and it is Pratik's reason rather than a
compromise.** The two halves fail differently: asking the server fails against a
server that mangles the identifier, and the stored bytes do not care what the
server does. Neither can lose the message, because the move is append then remove
and the source holds it throughout, so the worst case in every branch is a
duplicate somebody can see rather than a message nobody can find.

**What the pairing costs that neither half did.** One more plan, one more wave,
and a second blocking human checkpoint. `04.1-04` also adds tests to
`src/application/mail_across_accounts.rs`, which `04.1-02` and `04.1-03` will by
then have written guard records against, so its commits will print the
`--remeasure` command for those records and running it is not optional.

## What lands with the plans, and is not optional

`tests/the_planning_files_agree_with_themselves.rs` landed at `f0a00b4`, hours
before these plans were written, and it reads the roadmap against the disk. Its
rule for a `TBD` denominator is exact:

```rust
Err(_) if total == "TBD" => done == files.summaries && files.plans == 0,
```

The roadmap row today is `| 4.1 Mail moves between accounts | 0/TBD | Context
written, not planned | - |` at `ROADMAP.md:532`. **The moment the first
`04.1-*-PLAN.md` file exists in the phase directory, that row is a finding and
`test_the_roadmap_counts_the_files_that_are_on_disk` goes red.** So the commit
that lands these plans also changes the row and its status, in the same commit.
This target is in `check.sh`'s `docs_only` list and in its whole-tree list, so a
documents-only commit runs it.

**The value is `0/4`, and it was `0/3` in the first version of this note.** The
rule at `:723` is `total == files.plans` and `done == files.summaries`, so the
denominator is a count of `*-PLAN.md` files on disk and nothing else. Four plans
land, so `0/4`. Re-checked at `33ee3ef` on 2026-09-07: the row is still at
`ROADMAP.md:532`, still reads `0/TBD`, the header row is still at `:525`, and the
`TBD` arm is still `Err(_) if total == "TBD" => done == files.summaries && files.plans == 0`.
Take it again before landing rather than quoting this: HEAD moved three times
during the writing of these plans and once more during their revision.

`STATE.md`'s counts are compared only against the phase its frontmatter names,
which is `04.2` today, so they do not fire on landing. They will the moment
`current_phase` becomes `04.1`, and then the Current Position section's Total
Plans in Phase line must say 4.

**That target moved under me while this was being written, in the direction that
makes the point sharper.** At `f0a00b4` it was in `check.sh`'s `docs_only` list
and not in `guards_that_read_the_whole_tree`. `d1ed996` added it to the second,
so a `.planning` document landing beside code now runs it too, which is how every
plan summary lands. Re-read `scripts/check.sh:21` and its `docs_only` block rather
than trusting either version of this paragraph.

**HEAD moved from `f0a00b4` to `599cebc` during the writing of these plans**, over
three commits: `6cce7ae`, `d1ed996` and `599cebc`. They touched
`.planning/WINDOWS.md`, `.planning/planning-consistency-check.md`,
`scripts/check.sh` and `tests/the_planning_files_agree_with_themselves.rs`. No
source file any measurement here depends on changed, and `guards/guards.toml` is
still at 649 records. Everything above was measured at `f0a00b4` and re-checked
against `599cebc` where the diff touched it.

## What this phase owes elsewhere

**Phase 5 says in four places that its reminder move is the first cross-account
move in the program.** Once 4.1 lands that is false, which is the whole reason 4.1
goes first. I ran the sweep rather than counting in prose, because a prose count
is what goes stale and `04.2-07` found four places where its plan said three:

```bash
grep -rn "first move\|first cross\|crosses one" \
  .planning/phases/05-the-other-five-modules-keep-up/
```

The four hits, and I very nearly wrote "two":

- `README.md:30`, the `05-05` row of the plans table.
- `05-05-PLAN.md:45`, a `must_haves.truths` entry, which also says "the honest
  reading of decision 4's 'the way mail already moves between accounts' is that
  mail does not". After 4.1 that sentence is inverted, not merely stale.
- `05-05-PLAN.md:50`, the objective.
- `05-05-PLAN.md:194`, a numbered premise correction.

Whoever lands 4.1 corrects all four, or `05-05` executes against a premise that
stopped being true and against a `must_haves` entry that is the opposite of the
truth. Phase 5's own risk-ordering section is unaffected: it argues from
`one_day_kept_out_of_the_series` and the destructive write going last, which 4.1
does not change.

Note also that the file is `README.md` and not `PLANS-README.md`. Phase 4.2 uses
`PLANS-README.md` and phase 5 uses `README.md`; this phase follows 4.2, since it
is the more recent and the more complete of the two.

## What this phase cannot close


- **Nothing here has met a real mail server.** The loopback servers prove the
  commands and their order. They prove nothing about how a real provider answers
  an `APPEND` of a large message, what Gmail does with a cross-account append when
  its own copy model is labels, or what a provider does with a message whose
  `Message-ID` already exists at the destination. `04.1-03` names those three in
  the changelog under known limitations and in its checkpoint.
- **Nothing here has been heard.** The window becomes a tree somebody really
  navigates by account for the first time. Whether an account row reads as an
  account, whether a collapsed branch announces itself, and whether two accounts
  with the same label are told apart by ear are all questions only NVDA and
  Narrator can answer. `04.1-03`'s checkpoint has the seven questions written out,
  and `04.1-04`'s adds whether the notice about an unfinished move is spoken,
  reachable by keyboard, and does not read like an error when nothing was lost.
- **Whether a move interrupted by the program stopping really resumes.** Only
  killing the process between the write and the clear and starting it again shows
  it, and only doing that with the *source* account unreachable afterwards tells
  the kept bytes apart from doing nothing at all. That is `04.1-04`'s checkpoint,
  step 7, and it is the step that decides whether the second half of decision 17
  earned the whole message it puts on somebody's disk.
- **The guard sweep.** By the decision of 2026-09-03 no sweep runs per merge or
  per phase, and phase 8 criterion 5 owns it. Each plan runs the scoped remedy the
  commit prints, which is what makes the deferral affordable, and each records the
  base commit its deferral is against.
- **The PIM picker.** `managers.rs:6403` still builds one branch from the open
  account and still names accounts by label rather than address. Phase 5 owns it,
  and `04.1-01` makes both of those visible rather than fixing them, because a
  PIM move that crosses an account is `05-05`'s subject and not this phase's.

## Estimates, and what they are worth

84k, 88k, 88k and 84k tokens, three tasks each, confidence low. `04.1-02` came
down from 92k when its decision checkpoint was answered and removed; `04.1-04` is
new. They are projections with no calibration behind them: the same shape of
estimate in phase 4.2 ranged 52k to 92k against plans that turned out to be
comparable work. Read them as an ordering rather than as a budget, and read the
phase total as having gone up by roughly a plan's worth because a decision was
taken, not because the estimates moved. The one cost in them that is measured rather
than guessed is the guard record fan-out, and that number is dated and carries its
command because it will be wrong by the time anybody acts on it.

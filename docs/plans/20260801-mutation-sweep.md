# Mutation sweep: everything not yet asked

Mutation testing changes the code in small ways and runs the suite. A change
nothing notices means no test was watching, so either the behaviour is unpinned
or the code is dead.

Four modules have been through it and are clean. Everything else has not. This
records the order, so a run interrupted overnight can be picked up rather than
started again.

## Scale

4,171 mutants outside the four already done. On this machine cargo-mutants
manages roughly one every twenty-six seconds and cannot be run in parallel here,
which puts a full sweep near thirty hours. It is a background job measured in
days, not a step in a commit.

Each directory pays its own baseline build, about five minutes, so running one
large directory beats running its files one at a time.

## Order, and why

Highest cost of being wrong first.

| Area | Mutants | Why here |
|---|---|---|
| `common` | 89 | Errors and types everything else is built on |
| `service/protocols` | 327 | Mail parsing. A bug loses or corrupts somebody's mail |
| `data` | 513 | Storage. A bug loses what was already downloaded |
| `application` | 1,307 | The decisions: what to fetch, what to file, what to say |
| `service` (rest) | 211 | Safe browsing, spelling, OAuth, security |
| `presentation` (not the window) | 725 | Reading surfaces, columns, dates |

The wxWidgets layer is excluded in `mutants.toml` and stays excluded. It cannot
be exercised without a display, so every mutant there would survive and say
nothing except "this needs a person". Its verification is a screen reader pass.

## What the first four modules found

Worth knowing before reading the next report, because it will almost certainly
be the same shape.

The tests covered the paths somebody would think to write a test for and left
whole families untouched: four of the fields a filter rule can name, six of the
eleven ways it can match, five of the actions it can carry out. In each family
one member had a test and the rest had none.

**Where to look first in any report: a function that switches on a string.**
Expect one arm tested and the rest untouched.

## Running it

```bash
scripts/mutants.sh src/common
```

Read the run as finished only when the process has exited. `mutants.out` is
filled in as it goes, and reading it early once produced a commit message here
claiming seventeen of eighteen caught when the finished figure was thirty-seven
of fifty-one.

## What `common` turned up

Same shape again, and in two places it mattered more than in the filter engine.

`Protocol::as_str` writes the value that goes into every account file, and
nothing checked what it produced: it could have written an empty string and no
test would have said so. `FolderType::from_stored` had an untested Outbox arm,
which would have made every outbox an ordinary folder, so mail waiting to go out
would sit where nothing looks for it. `is_settings_file` had its two halves
never checked apart, so `and` becoming `or` would have made a migration sweep up
every .json file in the folder.

The two timeouts are in `redact_provider_message`, where turning `+=` into `*=`
makes the walk over the string never finish. A timeout is not a survivor: the
change was noticed. It does say the loop's bound is arithmetic, which is worth
knowing.

## What `service/protocols` is turning up

Two kinds of finding, and they need different answers.

**The gate that stops an alpha build changing a real mailbox had no test.**
Replaced with "yes", the whole suite stayed green. The decision has been pulled
out into a free function so it can be asked without a socket: a safety property
testable only against a live server is one that does not get tested. Every flag
accessor on a fetched message was the same kind of gap, and `seen` answering
true for everything means a mailbox arrives entirely read.

**Most of the rest cannot be answered here.** `poll_read`, `poll_write`,
`connect`, `fetch_headers`, `fetch_body`, `all_uids` and the other session
methods are the socket, and a mutant in them survives because nothing short of a
server exercises them. Do not write tests that pretend otherwise. Their
verification is the live account run, which is its own tracked work, and the
sync logic that sits on top of them is covered through the `Mailbox` trait.

When reading this area's report, sort the findings into those two piles first.
Of the 113 that survived the finished run, all but one fall in the second pile:
they are `ImapSession` methods whose whole body is a round trip. One did not,
and is now closed:

- `imap/sequence_set.rs`, `chunks`: the length comparison that decides where a
  sequence set is split. Pure, well tested at every other point, and the
  boundary between `>` and `>=` was not among them. A set that exactly fills
  the limit now has a test saying it goes in one command, and one character
  past it has a test saying it does not.

## Reading a report against a tree that has moved

cargo-mutants copies the tree when it starts and works on the copy, so the
report describes the commit the run began at, not the commit it finished at.
That is fine for the run and misleading afterwards.

It happened on the first `data` run: the early findings were acted on while the
run was still going, so by the time it finished its list of misses included
about twenty in `account.rs` and `config.rs` that already had tests. Check a
finding against the current tree before treating it as open.

The other half of the same point: a finding is a fact whatever the totals say,
so acting on one mid-run is fine. Quoting the totals mid-run is not.

## What reading a module turns up that a mutant does not

Worth knowing, because the two are not the same list and the sweep is better
for doing both.

cargo-mutants replaces a whole function body, or one binary operator. So it
never asks what happens to one arm of a `match`, and it never changes a literal
sitting in an argument list. Three of the largest findings in `data` came from
reading rather than from the report:

- Eleven of the thirteen domains account setup recognises had no test. The
  report could not have found this: the function's body was covered.
- The preset fields, including whether a provider connects with TLS, were
  unchecked except Gmail's.
- `save_account` writes an empty string into the password column, and that
  literal is the whole reason the credential store exists here. Changed to the
  real password, every test still passed.

So: run the report, and read the module for match arms, literal arguments, and
anything a comment says is deliberate.

## Progress

| Area | Result | Done |
|---|---|---|
| `application/filters`, `due`, `tagging`, `sign_off` | 157 mutants, 141 caught, 16 unviable, 0 missed | 2026-08-01 |
| `common` | 89 mutants, 63 caught, 13 missed, 11 unviable, 2 timeouts. All 13 closed | 2026-08-01 |
| `service/protocols` | 327 mutants, 133 caught, 113 missed, 81 unviable. Closed: flag accessors, credential redaction, the write gate. Of the rest, all but one are socket methods, see below | 2026-08-01 |
| `data` | Running, 513 mutants. `account.rs`, `config.rs`, `email_providers.rs`, `message_cache/{accounts,contacts,mod}.rs` closed while it ran, so its report is stale for those | started 2026-08-01 |
| `application` (rest) | | |
| `service` (rest) | | |
| `presentation` (not the window) | | |

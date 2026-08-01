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

## Progress

| Area | Result | Done |
|---|---|---|
| `application/filters`, `due`, `tagging`, `sign_off` | 157 mutants, 141 caught, 16 unviable, 0 missed | 2026-08-01 |
| `common` | 89 mutants, 63 caught, 13 missed, 11 unviable, 2 timeouts. All 13 closed | 2026-08-01 |
| `service/protocols` | | |
| `data` | | |
| `application` (rest) | | |
| `service` (rest) | | |
| `presentation` (not the window) | | |

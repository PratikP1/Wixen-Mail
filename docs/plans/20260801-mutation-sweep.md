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

## A shared file between runs makes the whole report a lie

Read this before trusting any number here.

cargo-mutants copies the source tree, so each mutant builds in isolation. It
does not isolate anything the tests write at runtime, and `env::temp_dir()` is
the same folder for every one of them. Seven tests here opened a database at a
fixed path in it.

So the first mutant that damaged stored data left that damage in a file the
next several hundred mutants would open. From that point every mutant was
reported caught, because a test failed on every run: not the test watching that
mutant, just the one reading the poisoned file. A mutant nothing was watching
looks exactly like one that was.

It happened on the first `data` run. `delete_account` replaced with "do
nothing" was correctly caught, and left an extra account behind while being
caught. Everything after it is unusable, and the first run's 353 caught, 67
missed has to be thrown away rather than corrected: there is no way to tell
from the report where the poisoning started.

The `service/protocols` result above was checked against this and stands. It
ran before the poisoning started, nothing under it touches the temporary
folder, and its report holds 113 misses: had the suite been failing on every
mutant, near enough everything would have come back caught. That last point is
the quick test to apply to any older report. A run with almost no misses is not
necessarily good news.

The tests now open a folder named for the moment they run. Before starting a
sweep over any area, check that nothing in it writes to a fixed path:

```bash
grep -rn 'temp_dir()\.join("' src/ | grep -v format!
```

Three hits are expected and are not tests: the fallback log folder, the
uninstall log, and the folder help pages are converted into when the program's
own folder cannot be written to. Those are fixed on purpose, because somebody
has to be able to find them. Anything else in that list is a test sharing a
file with every run before it.

This is worth more attention than it sounds. The failure mode is a report that
says everything is fine.

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

cargo-mutants replaces a whole function body, replaces one binary operator, and
deletes one arm of a `match`. What it does not do is change a literal sitting
in an argument list, and it cannot see a value that is only wrong in the world
rather than wrong in the code.

Two of the largest findings in `data` came from reading rather than from the
report:

- `save_account` writes an empty string into the password column, and that
  literal is the whole reason the credential store exists here. Changed to the
  real password, every test still passed. No mutant would have asked.
- The preset fields, including whether a provider connects over TLS, were
  unchecked except Gmail's. Same reason: they are literals in a list.

A third was found by reading and did also appear in the report, which is worth
being straight about. Eleven of the thirteen domains account setup recognises
had no test, and the report named three deleted match arms. Reading got there
first; it was not something the report could not have found.

So: run the report, and read the module for literals that carry a decision, and
for anything a comment says is deliberate. Those are what the report cannot
reach.

## What `data` turned up, and the twelve left on purpose

The two runs disagreed completely about where the gaps were, which is the
poisoning above and worth seeing once: the first said all 67 were in five files
and none in the other fourteen. The second, with the tests isolated, said none
in those five and 53 across seven of the fourteen. The first run had been
reporting "caught" for a test failure that had nothing to do with the mutant.

The 53 fell into four kinds. Worth reading before the next area, because they
will be the same kinds.

**A thing that reports success and does nothing.** Deleting a note folder, a
task list or a queued message. Clearing an account's mail. Recording a folder's
counts. Storing a safety verdict. Each is a command somebody gave that the
interface then says it carried out.

**A lookup that answers for the wrong row, or for none.** Which mailbox holds a
message, which row a server number means, which account a folder belongs to.
These are how an action finds its target, so the wrong answer is a flag or a
deletion landing on somebody else's message.

**Arithmetic at a boundary.** Where a vCard line folds, where a sequence set
splits, how far eviction has to go. Each is right in the middle and wrong at
the edge, which is the case nobody writes a test for.

**A test that names the thing it does not check.** The eviction test called
"least recently read" passed with the read never being recorded, because it
saved the bodies in the order it wanted them dropped. The folding round trip
joined the pieces with nothing between them, so it could not see a missing
continuation space. These are worse than no test: they are read as covered.

Twelve are left, and each for a stated reason:

- **Eight** are the contact group membership functions. Nothing calls them, so
  a saved group can never hold anybody. Tests there would make dead code look
  alive, which is the failure this whole sweep exists to find. Task #110 is the
  decision about which half of that feature to keep.
- **Three** are the comparison in `migrate_inline_bodies` that decides whether
  to write a line to the log. Changing it changes what is logged and nothing
  else. Not worth a test that captures log output.
- **One** is equivalent rather than surviving: in `parse_name_email`, whether
  the bracket check is `>` or `>=` cannot matter, because the two positions
  hold different characters and can never be equal.

## What `service` turned up

1,296 mutants, 629 caught, 483 missed, 183 that will not compile, one timeout.
The large miss count is expected here and most of it is not work.

**Roughly half is the network, and stays that way.** `protocols`, `oauth` and
the Safe Browsing client are sockets and round trips. A mutant in one survives
because nothing short of a real server exercises it, and a test that pretended
otherwise would be a test of the pretence. Their verification is the live
account run, which is tracked separately. The rule from the protocols pass
holds: sort the report into those two piles before touching anything.

**The provider clients were in that list and no longer belong there.** They
were filed under the network because every address came from a private
constant, so nothing could reach them short of Google or Microsoft. They now
hold the address they ask as a field, and a test can stand one up on a loopback
port and read the request that actually went out. Two things follow. The URL
decisions have moved out of round-trip method bodies into small functions that
build a string and nothing else, which is a shape mutation testing can see.
And this area is worth a scoped run again rather than being skipped on the
strength of the paragraph above, which is what would have happened otherwise.

What has not changed is that no request here has met a live server. The seam
proves what this code sends, not what a provider accepts.

**A second group could not be tested at all, and that was the finding.** Every
path into `oauth_credentials` runs through an environment variable or through a
gitignored file in somebody's settings folder, so all twenty-eight of its
findings were one finding: unreachable, not untested. The same was true of the
trusted domains list in `security`. In both cases the decision was also written
out several times, once per source, which is what let the copies differ. Pulling
the decision out of the plumbing fixed both problems at once, and that is the
shape to reach for whenever a whole file comes back untestable: the fault is
usually that a decision is welded to how its inputs arrive.

**What was left was worth finding.** The largest was in Safe Browsing: for an
address with more than five components the first parent form was skipped, so a
site listed under exactly that form was never matched and no warning appeared.
Every test used a three part address and none could see it. Found by reading
the arithmetic a mutant had flagged and working out what it would break, which
showed the unmutated version was already wrong.

Also: an attachment without a dot in its name would have been saved as
"attachment"; a bad signature could be reported as unknown rather than invalid;
two risk bands could vanish and every message in them be described as something
else; the offset of a misspelling, which is where the editor underlines and
where somebody working by ear is sent, had nothing checking it pointed at the
word.

Three things here are unreachable and stay, each for a different reason worth
telling apart.

Two arms of `mime::header_text`, for headers `mail-parser` always hands back as
text. Kept, with the reason written next to them, because without them a future
parser version would drop a read receipt request silently.

Five in `credentials`, which sit inside `#[cfg(not(test))]`. They are not
compiled while the tests run, so a mutation there cannot change anything and
the suite passes by default. That is the seam working: it exists so no test
ever writes into the credential store of whoever ran it, which happened once
and left an account behind in a real Windows Credential Manager. A report
cannot tell "unreachable under test" from "untested", so this one has to be
recognised by reading.

Four in `ical_subscription`, which is an HTTP fetch. Same pile as the rest of
the network.

## What `presentation` left standing, and why

Seventeen misses, from the narrower run of 2026-08-05 that covered
`src/application` and `src/presentation` only. Thirteen are closed. Four stay,
and this section exists because the last pass over this area reported its
reasons to a person and wrote none of them down. That is why the same
seventeen were triaged twice. Anything left standing belongs here, in the tree,
next to `data`'s twelve.

A scoped re-run says so, per the closing rule below. 34 mutants over the four
files touched, cut to the functions involved: 29 caught, 3 that will not
compile, 2 missed. The 2 are the pair named below that cannot be killed
honestly. Every one of the 13 came back caught, and so did the eleven new
mutants the work itself created, six of them on the high contrast call that was
split out. Splitting a call out to test it usually trades old survivors for new
ones on the new leaf; here it did not, because the assertions exclude every
value the replacements can take.

Two of the four left standing, `theme::current` and `theme::paint`, were
outside that re-run's filter and are unmeasured since 2026-08-05. Nothing
touched them, and the reasons below are why no test would have reached them
anyway.

Two of the four kinds from `data` show up again, and one new one.

**A test that moves with the thing it tests.** The preview document's language
looked covered. The test built the string it expected by calling the same two
functions the document calls, so any wrong answer moved both sides of the
assertion at once and it stayed green through all three mutations. It is a
sharper version of "a test that names the thing it does not check": this one
names it, calls it, and still cannot fail. Reading the machine through the
other function that answers the same question was the whole fix.

**A redundant condition is a condition no test can reach.** In
`order_from_locale`, three arms answered month first twice, and the guard in
the middle of them turned away nothing that did not get the same answer from
the fallback. Loosening it changed no answer for any input, so the mutant was
genuinely equivalent and no test could have killed it. The fix is not a test,
it is deleting the redundancy: as one condition the same mutation fails an
assertion that was already there. Worth recognising by shape. A comparison
whose every rejection lands somewhere that agrees with it is unreachable, not
untested.

**Arithmetic at a boundary, again, and this one empties a panel.** All three
`the_window_now` mutants end with the calendar list built and nothing in it.
Both call sites are in files this sweep excludes, so nothing downstream could
have caught them either. When a function reads the clock, split the arithmetic
out and hand the day in, the way `date_display::format_for_list` already takes
`now`. Otherwise the only assertions available are relative ones, and a
relative assertion cannot tell six months back from six months back and a day.

Four are left, and each for a stated reason:

- **`theme::current`, answering nothing.** It calls `wxdragon::is_system_dark_mode()`
  and the high contrast question, and its only caller is in `wx_app.rs`, which
  is excluded on purpose. There is no wxWidgets test harness in this tree and
  building one is not mutation triage. The decision it carries is already split
  into `palette_for` and tested three ways.
- **`theme::windows_high_contrast`, answering off.** Killing it means
  `assert!(!windows_high_contrast())`, which is a test of whether the machine
  running the suite has high contrast switched off. It goes red the moment
  somebody turns high contrast on to do an accessibility pass, which is the one
  time the suite must not be lying. Do not close it this way.
- **`theme::windows_high_contrast`, answering on.** Killing it means calling
  `SystemParametersInfoW` with `SPI_SETHIGHCONTRAST` against the live desktop of
  whoever runs the suite. Same class of mistake as the credentials one above,
  which left a real account behind in somebody's Windows Credential Manager.
- **`theme::paint`, doing nothing.** Two calls on a wxWidgets control, reached
  from three places in `wx_app.rs` and nowhere else. No test in this tree
  constructs a widget. There is no pure part left to split: the rule that a
  background never travels without the text colour tested against it is carried
  by the `Surface` type and is pinned.

The first and the last are not holes in the tests, they are the shape of the
feature. `theme::current` is called once, at window construction, which is the
whole reason a theme change waits for a restart, and `paint` reaches exactly
the three controls the `REACH` sentence already names. Whoever picks that work
up should read it as wiring rather than testing, and the verification after the
wiring is still a person looking at the window.

What did move on the high contrast side is the question itself. Nothing had
ever checked that Windows answers it. The struct carries a pointer, so it is a
different size in a 32 and a 64 bit build, and Windows refuses the call when
the size field disagrees with what it was handed. A refused call answers zero,
that is correctly read as "no answer", and our palette then stays in charge
forever over the colours of somebody who cannot read anything else. The call is
now split out and a Windows-only test says it succeeded. That test asserts
`HCF_AVAILABLE`, which says the platform has the feature and stays set either
way. It must never assert `HCF_HIGHCONTRASTON`, for the reason two bullets up.

Two things for Pratik rather than for the next sweep:

- `palette_for(setting, system_is_dark, high_contrast)` takes two bare `bool`s
  in a row. Swapping them at the one call site compiles and every test stays
  green, and because `is_system_dark_mode()` always answers false here the swap
  would feed `false` into the high contrast override and hand our palette to
  somebody running high contrast. That is a type, not a test.
- `LOCALE_IDATE` answers 2 for the year first locales: Hungarian, Lithuanian,
  Japanese, Chinese, Korean and others. `order_from_locale` sends anything that
  is not 0 to day first, so those machines get "26 July 2026". `DateOrder` has
  two variants, so there is no right answer available in the current types.
  Either a note beside `ENGLISH_ONLY` or a third variant, and the second is a
  feature.

## Progress

| Area | Result | Done |
|---|---|---|
| `application/filters`, `due`, `tagging`, `sign_off` | 157 mutants, 141 caught, 16 unviable, 0 missed | 2026-08-01 |
| `common` | 89 mutants, 63 caught, 13 missed, 11 unviable, 2 timeouts. All 13 closed | 2026-08-01 |
| `service/protocols` | 327 mutants, 133 caught, 113 missed, 81 unviable. Closed: flag accessors, credential redaction, the write gate. Of the rest, all but one are socket methods, see below | 2026-08-01 |
| `data` | Done. 510 mutants, 405 caught, 12 missed, 93 unviable, checked on a clean run after the work. The 12 are the ones left on purpose, listed above. First run void, second found 53 | 2026-08-01 |
| `application` (rest) | 1,440 mutants on the confirming run, 1,099 caught, 171 missed, 165 unviable, 5 timeouts. 115 of the 171 then closed, 56 reported: 21 dead code in the four manager modules, 20 behind the provider clients' fixed host, the rest equivalent or unreachable | 2026-08-04 |
| `service` | 1,296 mutants, 629 caught, 483 missed, 183 unviable. Closed in the pure modules: safe browsing URLs, attachment names, security, OAuth credentials, spelling, mime, safety. About half the remainder is socket code and stays, see above | 2026-08-02 |
| `presentation` (not the window) | 952 mutants, 665 caught, 186 missed, 101 unviable. 145 closed, 41 reported. About 11 of those genuinely wait on a screen reader pass rather than on code. The reasons for the 41 were reported to a person and never written down, so the next run triaged them again from scratch | 2026-08-03 |
| `presentation`, the 17 the narrower run still had | 13 closed, 4 left on purpose and now listed above. Of the 13, six are the locale question, three the document language, three the calendar window, one an equivalent mutant closed by deleting the redundancy rather than by a test. Confirmed by a scoped re-run: 34 mutants, 29 caught, 3 unviable, 2 missed, and the 2 are the pair that cannot be killed honestly | 2026-08-06 |

## Two things this sweep got wrong about itself

Both are recorded here rather than quietly fixed, because the second one is the
reason the first was found at all.

**A pass was reported as closing 52 findings and closed 7.** Commit `dccd229`
was described as closing every finding in the diary module. The confirming
re-run said 45 of the 52 were still open, byte for byte the same mutations:
five tests had been added and the rest of the work was never done. Nothing in a
green suite could have shown that. The tell, visible only afterwards, was the
ratio: 52 claimed against 5 tests added. One test does often kill several
mutants, so a ratio is not proof, but past about three findings per new test,
ask which test kills which mutant and expect a named answer. A sweep is not
finished until a second run says so.

**A commit subject claims a fix its diff does not make.** `5cfa9cc`, "Stop a
lost connection reading as a folder with nothing in it", changes no behaviour
at all: its entire non-test diff is a corrected comment. Its body is honest and
says so, and it is the only commit of its group with no changelog entry, which
is right for a test-only commit and proves the subject wrong by its own hand.

It is left standing on purpose. Rewording it means rewriting 28 commits, and
several of those are cited by the work still outstanding, so the fix would
break more pointers than it repairs. A history that hides a subject was ever
wrong is worth less than one that says so here. Its sibling, eleven commits
back rather than twenty-eight, was reworded instead.

The rule both of these produced: a commit subject describes what its diff does.
If a commit only adds tests, the subject says so. "Pin" and "Say" are the
honest openings for those; "Stop", "Fix" and "Make" are not.

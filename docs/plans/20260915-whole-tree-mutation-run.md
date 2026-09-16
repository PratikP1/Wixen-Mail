# Mutation run of 2026-09-15: the mail protocols and the CalDAV client

The file is named for the whole-tree run phase 8 planned. What ran is two
areas, on Pratik's answer of 2026-09-15 to 08-08's checkpoint: the mail
protocols under `src/service/protocols/` and the CalDAV client in
`src/service/caldav.rs`, on GitHub's runners rather than on his machine. The
rest of the tree is not this run's, and the last section says what it would
cost.

This page is the run's record and the next round's input. Every survivor is
listed by file with one of five dispositions: killed by a test, dead code
removed, equivalent with the reason, a string only a person reads, or
untested behaviour queued. A reason that would cover whatever survived is not
a reason, so each line says its own.

## The run

| | Protocols | CalDAV |
|---|---|---|
| Commit | `3e633252` | `3e633252` |
| Dispatch | run 34979540954, 2026-09-15 14:07:41Z | run 34979543954, 2026-09-15 14:07:43Z |
| Inputs | `mode=shards`, `file=src/service/protocols/**`, `shards=18`, `first=0`, `last=17` | `mode=shards`, `file=src/service/caldav.rs`, `shards=17`, `first=0`, `last=16` |
| Arguments | `--in-place -- --all-targets` | the same |
| Shards | 18 of 25 mutants, 7 jobs green and 11 red | 17, the last of 20, 9 green and 8 red |
| Wall clock | 4.7 h from dispatch, 4.0 h from the first shard | 8.1 h from dispatch, 6.4 h from the first shard |
| Runner time | 180,690 s, 50.2 h | 188,801 s, 52.4 h |
| Rate | median 372 s a mutant, mean 353 s | median 405 s, mean 415 s |
| Mutants | 450 | 420 |
| Caught | 316 | 378 |
| Nothing noticed | 41 | 15 |
| Compiler rejected | 91 | 25 |
| Timed out | 2 | 2 |
| Never started | 0 | 0 |
| Asked nothing | 93, 21 percent | 27, 6 percent |

A red job is the script exiting 1 on a survivor or a timeout, not a shard
that failed; every shard's verdict block was read, and the merger read the
two runs whole:

```
python scripts/mutants_report.py --shards target/runner-mutants-protocols 18
450 mutants: 316 caught, 41 nothing noticed, 91 the compiler rejected, 2 timed out.
357 of the 450 mutants got an answer from the suite.
91 the compiler rejected, 2 timed out: 21 percent of this run asked nothing.

python scripts/mutants_report.py --shards target/runner-mutants-caldav 17
420 mutants: 378 caught, 15 nothing noticed, 25 the compiler rejected, 2 timed out.
393 of the 420 mutants got an answer from the suite.
25 the compiler rejected, 2 timed out: 6 percent of this run asked nothing.
```

Both accepted: every shard present, complete, and at one commit. The rate
rows and the per-shard fixed cost are on `docs/development/measurements.md`,
dated 2026-09-15 at `3e633252`. The protocols run's 21 percent is read the
way the report says: as a check on the 357 that got through and not on the
450. Most of the 91 the compiler rejected are in `imap.rs`, where the parser
library's types leave a replaced return value nothing to compile against.

## The four timeouts, run again

Each was run again on this machine on 2026-09-16 in the worktree at the same
commit, with the same arguments, at the tool's auto-set budget of 594 s and
nothing else building:

```
cargo mutants --in-place -f src/service/protocols/imap.rs -f src/service/caldav.rs \
  --re '1081:16: replace \+= with \*=|2872:12: replace -= with /=|520:9.*Poll::from\(Ok\(1\)\)|778:38' \
  -o target/rerun-timeouts -- --all-targets
4 mutants tested in 46m: 4 timeouts
```

All four timed out again, so none was the runner's load. Each is the
mutant's own doing, and each was then applied by hand to one test:

| Mutant | What it does | By hand |
|---|---|---|
| `caldav.rs:1081:16: replace += with *= in events_in` | `at *= 1` leaves the scan on the first line that is not an event, forever | `test_parse_ical_vevent` under `timeout 120` never finished, exit 124 |
| `caldav.rs:2872:12: replace -= with /= in fits_in` | `at /= 1` never reaches a character boundary that is not already one | `test_a_character_that_takes_more_than_one_octet_is_not_cut_in_half_by_a_break` under `timeout 60` never finished, exit 124 |
| `imap.rs:520:9: replace poll_write with Poll::from(Ok(1))` | claims one byte written and writes none, so no command ever reaches the server | `test_a_folder_that_really_holds_nothing_still_comes_back_empty` failed in 10 s: "the server never finished the sign-in exchange" |
| `imap.rs:778:38: replace == with != in read_command` | the reader never accepts its own command's tagged answer and waits out `COMMAND_TIMEOUT` | the same test failed in 10 s the same way |

The two loops are hangs and no test can finish against them. The two IMAP
ones are caught by every test that opens a connection, each failing at the
harness's ten-second limit; about ninety such tests at ten seconds is longer
than the mutant's budget, so the run reported a timeout where the suite was
failing the mutant all along. Neither kind is a survivor of the tests, and
neither is counted as caught, because the report's rule is that a timeout is
neither. No setting in `.cargo/mutants.toml` moved; its comment names these
four.

## Survivors of `src/service/protocols/imap.rs`

Twenty-seven. Twenty-four killed, the tests in `imap.rs`'s own modules, the
by-hand red for each taken by applying the mutant and running the test named,
then `git checkout`; one family of nine under one test.

| Mutant | What became of it |
|---|---|
| `2073:9`, `2075:9`, `2076:9`, `2077:9`, `2078:9`, `2079:9`, `2082:9`, `2085:9`, `2086:9: delete match arm NameAttribute::{NoInferiors, Marked, Unmarked, All, Archive, Drafts, Flagged, Trash, Extension(name)} in attribute_name` | killed by `test_every_list_attribute_the_parser_knows_is_spelled_and_none_is_dropped`, commit `96ade665`. One family: a deleted arm falls to the one that answers an empty string, which is what a folder with no attributes says. Shown red with the `Trash` arm deleted. |
| `1106:9: replace ImapSession::selected_folder -> Option<&str> with None`, `with Some("")`, `with Some("xyzzy")` | killed by `test_the_folder_that_was_opened_is_the_one_the_session_says_is_open`, commit `96ade665`. Every move and copy in the controller asks this before it acts. |
| `1149:9: replace ImapSession::uids_above -> Result<Vec<u32>> with Ok(vec![])`, `Ok(vec![0])`, `Ok(vec![1])` | killed by `test_asking_for_the_uids_above_one_asks_the_server_for_exactly_that_range`, commit `96ade665`. The incremental sync's question. |
| `1028:9: replace ImapSession::folder_counts -> Result<FolderCounts> with Ok(Default::default())` | killed by `test_counting_a_folder_reads_the_totals_the_server_gave`, commit `96ade665`. The unread count on every folder. |
| `1082:17: delete match arm Response::Data{status:Status::Ok, code:Some(ResponseCode::HighestModSeq(modseq)), ..} in ImapSession::select_folder` | killed by `test_a_server_that_names_the_highest_modseq_on_opening_is_heard`, commit `96ade665`. |
| `838:66: delete ! in ImapSession::list_folders` | killed by `test_a_nested_folder_is_named_by_its_last_segment`, commit `96ade665`. With the filter inverted, `Parent/Child` was named by its whole path. |
| `1267:9: replace ImapSession::may_change -> bool with true` | killed by `test_a_fresh_session_may_not_change_anything_until_it_is_told_it_may`, commit `96ade665`. The write gate. |
| `1664:9: replace ImapSession::introduce_ourselves with ()` | killed by `test_a_server_that_takes_an_introduction_is_given_one`, commit `96ade665`. NetEase refuses a client that does not say who it is. |
| `1693:9: replace ImapSession::require_selected -> Result<()> with Ok(())` | killed by `test_a_session_with_no_folder_open_refuses_a_search_rather_than_searching_nothing`, commit `96ade665`. |
| `1723:54: replace * with +`, `replace * with /` | killed by `test_the_idle_window_stays_inside_what_the_standard_allows`, commit `96ade665`. `IDLE_WINDOW` is held between 20 and 29 minutes, RFC 2177's bound; 89 seconds and zero both fall outside. |
| `1735:9: replace ImapIdleHandle::stop -> Result<()> with Ok(())` | killed by `test_stopping_a_watch_ends_it_and_says_so`, commit `96ade665`. The first version of the test passed against the mutant, because dropping the handle drops the stop sender and the task reads that as a stop too; what the mutant loses is the wait, so the test asserts the server has been told LOGOUT by the time `stop` returns. Red three runs out of three. |
| `494:9: replace ImapStream::into_plain -> Option<TcpStream> with None` | untested behaviour, queued. The one caller is the STARTTLS upgrade at line 691, which hands the plain socket to the TLS handshake; with `None` every STARTTLS sign-in fails. No loopback test speaks TLS, so nothing reaches it. Needs a loopback server with a certificate, which is a harness this tree does not have. Ledger 472. |
| `527:9: replace <impl AsyncWrite for ImapStream>::poll_flush with Poll::from(Ok(()))` | untested behaviour, queued, with a qualification: equivalent on the plain stream, because tokio's `TcpStream::poll_flush` is `Ready(Ok(()))` unconditionally, so every loopback test sees no difference; different on the TLS stream, where a flush the shim skips leaves the record layer's buffer unsent, and no test speaks TLS. Ledger 473. |
| `534:9: replace <impl AsyncWrite for ImapStream>::poll_shutdown with Poll::from(Ok(()))` | untested behaviour, queued, the same shape: a shutdown the shim skips is invisible to a loopback test that drops the socket, and matters on TLS, where the close-notify never goes out. Ledger 474. |

## Survivors of `src/service/protocols/pop3.rs`

Twelve. Nine killed, three queued with the IMAP shims.

| Mutant | What became of it |
|---|---|
| `290:9: replace Pop3Session::stat -> Result<(usize, usize)> with Ok((0, 0))`, `Ok((0, 1))`, `Ok((1, 0))`, `Ok((1, 1))` | killed by `test_stat_reads_the_count_and_the_bytes_the_server_gave`, commit `96ade665`. Shown red with `Ok((0, 0))`. |
| `305:9: replace Pop3Session::listing -> Result<Vec<Pop3MessageInfo>> with Ok(vec![])` | killed by `test_the_listing_pairs_every_message_with_its_size_and_its_identifier`, commit `96ade665`. |
| `504:9: replace Pop3Session::read_text_data -> Result<Vec<String>> with Ok(vec![])`, `Ok(vec![String::new()])`, `Ok(vec!["xyzzy".into()])` | killed by the same listing test, commit `96ade665`: the listing is read through this, and a block of nothing, of one empty line, or of one line that is not a listing line all give an empty listing. Shown red with `Ok(vec![])`. |
| `394:9: replace Pop3Session::reset -> Result<()> with Ok(())` | killed by `test_reset_tells_the_server_to_undo_the_deletions`, commit `96ade665`. |
| `99:9: replace Pop3Stream::into_plain -> Option<TcpStream> with None` | untested behaviour, queued: the STLS upgrade, the same as IMAP's. Ledger 475. |
| `132:9: replace <impl AsyncWrite for Pop3Stream>::poll_flush with Poll::from(Ok(()))` | untested behaviour, queued, equivalent on the plain stream, the same as IMAP's. Ledger 476. |
| `139:9: replace <impl AsyncWrite for Pop3Stream>::poll_shutdown with Poll::from(Ok(()))` | untested behaviour, queued, the same. Ledger 477. |

## Survivors of `src/service/protocols/imap/mailbox_name.rs`

Two, both equivalent. The six equivalents on this page are ledger 479
together, so the next round does not triage them again from scratch.

| Mutant | What became of it |
|---|---|
| `193:5: replace separator_width -> usize with 1` | equivalent, by the module's own claim: the doc comment above the function says a separator is ASCII punctuation, and `after_the_parent` is never empty where it is called, because `where_the_leaf_starts` requires the path to be longer than the parent. `len_utf8` of an ASCII character is 1. A two-byte separator would tell them apart and is a server this project has never met; a test asserting it would assert behaviour the module disclaims. |
| `251:30: replace > with >= in the_separator_between` | equivalent: `leaf` equals `parent.len()` only when `separator_width` returned 0, which the line above rules out, or when the parent is empty and the leaf is 0, where `&path[0..0]` and the `_` arm are both the empty string. |

## Survivors of `src/service/caldav.rs`

Fifteen. Ten killed, four equivalent, one queued.

| Mutant | What became of it |
|---|---|
| `259:44: replace && with \|\| in CalDavClient::discover_calendars` | killed by `test_a_home_set_answered_with_a_plain_200_is_still_read`, commit `96ade665`. Every earlier test answered 207. |
| `323:74: replace != with == in CalDavClient::list_events` | killed by `test_a_report_the_server_refuses_is_a_refusal_and_not_an_empty_calendar`, commit `96ade665`. A refused REPORT was read as a calendar with nothing in it. |
| `439:44: replace && with \|\|`, `439:12: delete !`, `439:74: replace != with ==`, all `in CalDavClient::journal_entries_in` | killed by `test_a_journal_listing_answered_with_a_plain_200_is_still_read` and `test_a_journal_listing_the_server_refuses_says_which_status_it_refused_with`, commit `96ade665`. The function had no test of its own. Shown red with the `!` deleted, which fails both. |
| `733:5: replace names_a_collection -> bool with false`, `735:34: replace + with -`, `736:52: replace \|\| with &&` | killed by `test_a_collection_is_recognised_by_its_element_and_not_by_a_name_that_starts_the_same_way`, commit `96ade665`. One family: nothing asked the function anything before. Shown red with `false`. |
| `1816:28: replace && with \|\| in normalize_ical_datetime` | killed by `test_a_clock_that_is_not_digits_is_not_dressed_up_as_a_time`, commit `96ade665`. `20260305T09ab00` came back as `2026-03-05T09:ab:00`. |
| `2918:33: replace match guard !lines.is_empty() with true in put_back_together` | killed by `test_a_document_whose_first_line_is_indented_keeps_that_line`, commit `96ade665`. |
| `1097:20: replace + with * in events_in` | equivalent, and the code says so: the comment on that line explains that landing on the closing line instead of one past it is not a distinguishable mistake, because the next spin finds nothing to open there and steps forward by one. The mutation run has now confirmed the comment. |
| `1687:18: replace > with >= in parameter_named_on` | equivalent, and the code says so: a `debug_assert_ne!` two lines above states that a semicolon and a colon can never sit at one byte position, so `>` and `>=` answer alike for every line that reaches it. |
| `1789:25: replace && with \|\| in normalize_ical_datetime` | equivalent under the grammar: an RFC 5545 date or date-time is digits with a `T` and an optional `Z`, holding neither `-` nor `:`, and the extended form this returns unchanged holds both. A value with one and not the other reaches neither of the fixed-width branches under `&&` and is returned unchanged under both. |
| `1812:35: replace - with / in normalize_ical_datetime` | equivalent: `time` is only ever read at `[..6]` and the output slices `dt` at fixed offsets up to 15, so whether the trailing `Z` is inside `time` or not changes nothing that is read. |
| `202:9: replace CalDavClient::for_account -> Self with Default::default()` | untested behaviour, queued. `for_account` reads this machine's stored settings through `ConfigManager::load_stored` to decide whether the client may change things; a test asserting the allowed client would pass here, where the settings allow it, and read wrong on a runner with no settings file, which is the shape ledger 470 named for two guard records. The constructor wants its answer as an argument before it can be pinned. Ledger 478. |

## What the run judged, and what it did not

The two areas at `3e633252`: 870 mutants, 750 of them answered by the suite.
The tree the configuration allows held 12,391 mutants at `2847391c` on
2026-09-15; the other 11,521 have not been through a run since the sweeps of
August on this page's predecessor, `docs/plans/20260801-mutation-sweep.md`,
and the rows on `docs/development/measurements.md` say what a whole run costs
on this machine and on a runner. The whole tree stays available through the
same workflow, `file` empty and `shards=496` in two dispatches, and the same
merger reads it. Criterion 4 of phase 8 was revised under criterion 6 on this
basis; `.planning/ROADMAP.md` carries the text.

Seven survivors are queued as untested behaviour, all of one kind: the TLS
half of the stream shims and the STARTTLS and STLS upgrades in both mail
protocols, which no test reaches because no loopback server here speaks TLS,
and one constructor that reads the machine's settings. The next round's
input is a loopback server with a certificate, which would reach six of the
seven at once.

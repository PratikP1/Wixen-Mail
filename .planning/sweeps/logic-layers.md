# Built for more than production asks: `src/application/` and `src/service/`

Read-only audit, 2026-09-06, against `main` at `8d73579`. Nothing in the
repository was written or changed.

**The tree moved while this ran.** An executor holds the same checkout and
modified eight files partway through, among them `src/application/pictures.rs`
and `docs/changelog.md`, both of which this report cites. Every line number and
every claim below was re-checked against the tree as it stood at the end. The
`pictures.rs` changes are in a different part of the file (outgoing picture
descriptions), and finding 3 reproduces unchanged. If you read this much later,
re-run the greps rather than trusting the line numbers.

## How this was searched, and what that is worth

The method is mechanical. Every `pub fn`, every `pub enum` variant and every
`pub struct` field declared in `src/application/` and `src/service/` was
collected, then every reference to it anywhere in `src/` and `tests/` was
classified as production or test. Test means inside an item annotated
`#[cfg(test)]`, tracked by brace depth so a nested test module is caught, or
inside `tests/`. That matters here: `src/application/calendar.rs` opens a
`#[cfg(test)]` at line 56 and `src/service/signed_mail.rs` has an indented one
at line 2803, so the common shortcut of "everything after the first
`#[cfg(test)]` is test" gets both files wrong.

**The scanner was validated against the known case before any of its output was
believed, and that caught a bug that would have manufactured false findings.**
Asked for `offer(` and `Branch {` it first reported zero production call sites,
where `src/application/destinations.rs` is known to have three. The cause was a
trailing `\b` on patterns ending in `(` or `{`, which can never match because
the next character is a newline, so every call whose arguments wrapped was
invisible. After the fix it reproduces the calibration exactly:

```
$ python idx.py "Branch\s*\{"
  P src/application/destinations.rs:149: pub struct Branch {
  P src/presentation/managers.rs:6403: vec![Branch {
  P src/presentation/wx_app.rs:8057: let branches = vec![Branch {
  P src/presentation/wx_app.rs:16794: vec![Branch {
```

Three sweeps ran: no production caller, enum variants production never
constructs, and parameters every production call site passes the same literal
to. Each finding below was then opened by hand and read in its enclosing
context. **Every finding here was verified by reading the code.** Where
something is inferred rather than read, the entry says so.

Three of the sweeps found less than they might, which is itself a result.
Struct fields whose production initialisers are always the same literal: one
hit, the Google People API's response-only `metadata` field, correctly `None` on
a request. Enum variants production never constructs: two, both in the same
enum, and they are finding 2 below. Parameters every production call site passes
the same literal to: four functions, one of them a false positive on `ldap3`'s
own `Scope` type, and the other three are findings 4 and 5 and one Documented
row. This codebase is disciplined about the patterns being hunted, so the list
is short and what is on it is real.

**The largest single source of noise was "no production caller" meaning three
different things.** Of 1,098 public functions declared in these two directories,
56 have no production reference at all. About a fifth are genuine gaps. The rest are thin
wrappers whose delegate is called (`SavedSearch::what_it_found` delegates to
`what_a_search_found`, which production calls at three places in `wx_app.rs`),
same-job twins (`wire::unstuff` beside `wire::unstuff_bytes`, which
`pop3.rs:371` calls), or stricter variants production replaced with a better one
(`OAuthService::is_expired` beside `expires_within`, which `oauth.rs:786` calls
with a five-minute margin). Those are not reported. The ruled-out list is at the
end so you can see the filter ran.

`tests/wired.rs` cannot see any of this. Its own `sources()` collects
`src/presentation` only, so a capability in `src/application` or `src/service`
that nothing reaches is outside what it asks about.

---

# Contradicted

Something claims the general behaviour and production does not do it.

## 1. A meeting reply goes out as a plain file, so the organiser's client never records it

**What it is designed to do.** `TheAnswerToSend::the_calendar_part` builds the
reply's calendar document as a MIME part declared
`text/calendar; charset=utf-8; method=REPLY`, named `reply.ics`. Its doc comment
names the exact consequence of not doing that:

> `method=REPLY` is the whole of it: it is what tells a receiving client this
> attachment is an answer rather than a calendar file somebody happened to send,
> and without it the answer is shown as a file to open by hand and never
> recorded against the meeting.

**What production asks of it.** Nothing. It has one reference besides its own
definition and that reference is a test.

```
$ grep -rn "the_calendar_part\|WHAT_AN_ANSWER_IS\|WHAT_THE_PART_IS_CALLED" src/ tests/
src/application/answering.rs:440:    pub fn the_calendar_part(&self) -> crate::application::attaching::Ready {
src/application/answering.rs:442:            name: WHAT_THE_PART_IS_CALLED.to_string(),
src/application/answering.rs:443:            content_type: WHAT_AN_ANSWER_IS,
src/application/answering.rs:453:const WHAT_AN_ANSWER_IS: &str = "text/calendar; charset=utf-8; method=REPLY";
src/application/answering.rs:461:const WHAT_THE_PART_IS_CALLED: &str = "reply.ics";
src/application/answering.rs:1588:    fn test_the_calendar_part_says_it_is_a_reply_so_the_organisers_client_reads_it_as_one() {
src/application/answering.rs:1598:        let part = sending.the_calendar_part();
```

The `#[cfg(test)]` module in that file opens at line 736, so 1588 and 1598 are
test code and 440 is the only production line.

Production writes the document to a temporary file and attaches it like any
other file:

```
$ grep -n "a_place_for_the_reply\|reply-{}\|attachments: vec!\[written\]" src/presentation/wx_app.rs
11877:    let written = match a_place_for_the_reply(&to_send.calendar_document) {
11893:        attachments: vec![written],
11903: fn a_place_for_the_reply(document: &str) -> std::result::Result<std::path::PathBuf, String> {
11917:    let at = folder.join(format!("reply-{}.ics", uuid::Uuid::new_v4()));
```

That path ends at `attaching::read_all`, which derives the content type from the
file extension:

```
$ grep -n '"ics" =>' src/application/attaching.rs
443:        "ics" => "text/calendar",
```

`smtp.rs:237` parses that string straight onto the wire
(`file.content_type.parse::<ContentType>()`), and `Email.attachments` is
documented as decided by `build_message`, which is pure. So what leaves is
`text/calendar` with no method and no charset, on a part named
`reply-<uuid>.ics`. Both of the things the code deliberately chose are absent.

**What claims otherwise.** `docs/changelog.md:2488`: "You can answer a meeting
invitation... The answer goes back to whoever called the meeting, **so they
learn where you stand instead of waiting.**" A standards-conforming client
receiving `text/calendar` without `method=REPLY` does not fold the answer into
the meeting, which is what `the_calendar_part`'s own doc says. The organiser
gets a file to open by hand.
`.planning/intel/built-and-left.md` lists "Meeting invitations: guest list,
replies, and working out when everyone is free" under **Built and exercised**.

**History.** `git log --oneline -S "the_calendar_part" -- src/` returns one
commit, `592ba56`. It was never wired, rather than wired and lost.

**Cost to close.** Small, and the code is already written.
`send_the_answer` in `wx_app.rs:11868` would use `to_send.the_calendar_part()`
instead of writing a file, which means `wx_compose::ComposeData` needs a way to
carry an already-built `attaching::Ready` beside its file paths. Today it
carries paths only, and the comment at 11874 explains why files are read at
send time. One extra field on `ComposeData` and one branch in
`mail_controller.rs:227` where `read_all` runs. The existing test at
`answering.rs:1588` already pins the part; what is missing is a test that the
queued message carries it.

**Verified**, by reading every line cited. The one step I did not run is an
actual send, because nothing here has ever been run against a real server.

## 2. Certificate withdrawal: the setting that turns on asking does not exist, and two states the changelog describes cannot occur

**What it is designed to do.** `service::signed_mail::Reach` has two values.
`WhatIsAlreadyHere` reads only cached revocation lists and contacts nobody.
`AskTheAuthority` is allowed to fetch the list named in the certificate.
`Reach::from_setting(may_ask_authorities: bool)` exists to read that choice from
a setting, and its doc says the mirror is deliberate:

> This mirrors `application::pictures::Fetching::from_setting`, deliberately:
> the two settings are the same shape and should not read differently.

`Withdrawal` has five values so that "nobody asked", "asked and could not find
out" and "the answer has not come back yet" stay distinct.
`StillBeingLookedInto` is documented as "the state that lets reading a message
stay instant", with the answer folded in later by
`SignatureReport::with_withdrawal_for`.

**What production asks of it.** One reach, hardcoded, and no setting anywhere.

```
$ grep -rn "Reach::AskTheAuthority\|Reach::WhatIsAlreadyHere" src/ --include=*.rs | grep -v "^src/service/signed_mail.rs"
src/application/checking_signatures.rs:26://! here with [`Reach::WhatIsAlreadyHere`]: what this computer already holds,
src/application/checking_signatures.rs:129:            let withdrawal = store.withdrawal(&certificate, now, Reach::WhatIsAlreadyHere);
src/presentation/wx_app.rs:11337:/// certificate store are asked with `Reach::WhatIsAlreadyHere`, which contacts
src/presentation/wx_app.rs:11348:/// [`Reach::WhatIsAlreadyHere`]: crate::service::signed_mail::Reach::WhatIsAlreadyHere
```

`checking_signatures.rs:129` is the only production call to `withdrawal`, and it
passes the constant. `Reach::from_setting` has no caller: its only two
production references are its own signature and body at `signed_mail.rs:2609`
and `2611`, and its only other references are two tests at 6302 and 6341. It is
the one `from_setting`-shaped constructor in the tree with no setting behind it.
Every other one reads a real `AppConfig` field:

```
$ grep -rn "fn from_setting" src/ | wc -l   # 15, of which 2 are from_setting_or
$ grep -niE "authorit|withdraw|revocat|crl" src/data/config.rs
(no output)
```

The consequence reaches into the match:

```rust
// src/service/signed_mail.rs:3221
match (reach, trusted) {
    (Reach::WhatIsAlreadyHere, _) => Asking::OnlyWhatIsHere,
    (Reach::AskTheAuthority, true) => Asking::TheAuthorityToo,     // unreachable
    (Reach::AskTheAuthority, false) => Asking::OnlyWhatIsHere,     // unreachable
}
```

Two of three arms cannot be reached in production, and
`Asking::TheAuthorityToo` is never produced. Downstream,
`Withdrawal::StillBeingLookedInto` and `Withdrawal::NotAsked` are never
constructed in production either. `signed_mail.rs:1639` turns the first into
`Finding::WithdrawalStillBeingLookedInto`, which has a whole spoken sentence in
`presentation/reader_text.rs` that nobody can hear.

**What claims otherwise.** `docs/changelog.md:2456`: "A certificate that has
been withdrawn is said plainly, and **a check still running is said as
unanswered rather than as good news.**" There is no check that runs, so that
state cannot occur. The same entry's "Known limitations" paragraph names three
limitations and not this one. The module doc calls `WhatIsAlreadyHere` "the
default", which implies a non-default a person can pick, and none exists.

The sharpest statement of the problem is in the codebase's own test, at
`signed_mail.rs:3596`, as an assertion message:

> "the setting does nothing at all, so it is not a setting"

The test passes. The sentence is true anyway, for a different reason than the
test was written to catch.

**What actually happens to a user.** Revocation is still answered whenever
Windows already holds a list for the issuer, which is common, so this is not
"never checked". It is "never fetched, and no way to ask for it", with no
sentence anywhere saying so.

**Cost to close.** Two ways out and they differ a lot in size. The honest cheap
one is to say so: a line in the changelog's Known limitations and a sentence in
the reader where a signature is reported, plus deleting or gating
`Reach::from_setting`. That is an hour. The full one is a real setting
(`AppConfig` field, a control in the Privacy or Reading section, since
`test_every_setting_somebody_can_change_is_offered_by_a_screen` will demand
one) plus the background caller `Reach`'s doc describes, which runs after the
report is on screen and folds the answer in. That needs the announcement path
to update an already-spoken report, which is real work.

**Verified**, including which line numbers are test and which are production.
The claim that Windows answers from a cached list when one is present is
**inferred** from `what_windows_found` reading the chain's
`withdrawal_result`; I did not run it.

## 3. The count of held-back pictures is computed for a sentence nothing says

**What it is designed to do.** `HtmlRenderer::sanitize_and_count_held_back`
returns the markup and the number of remote pictures held back. Its doc:

> A caller that shows a message wants both: the markup to show and the sentence
> to put above it. Counting a second time somewhere else would be two answers to
> one question.

`application::pictures::what_was_held_back(held_back)` is that sentence: "30
pictures were not shown, because fetching them would have told the senders you
opened this. Settings, Reading has the switch."

**What production asks of it.** The markup only. Both production callers
discard the count with `.0`, and the sentence has no caller.

```
$ grep -rn "sanitize_and_count_held_back\|what_was_held_back" src/ tests/
src/application/pictures.rs:542:pub fn what_was_held_back(held_back: usize) -> String {
src/application/pictures.rs:1198:        assert!(what_was_held_back(0).is_empty());
src/application/pictures.rs:1203:        let said = what_was_held_back(1);
src/application/pictures.rs:1213:        let said = what_was_held_back(30);
src/presentation/html_renderer.rs:274:    pub fn sanitize_and_count_held_back(&self, html: &str) -> (String, usize) {
src/presentation/html_renderer.rs:398:                self.sanitize_and_count_held_back(html).0
src/presentation/html_renderer.rs:561:                    self.sanitize_and_count_held_back(html).0
```

`pictures.rs` opens its `#[cfg(test)]` at 555, so 1198 to 1213 are tests.
`html_renderer.rs` opens its at 631, so 398 and 561 are the two production
callers and both take `.0`.

**What claims otherwise.** The doc comment at `html_renderer.rs:271` to `273`,
quoted above, describes a caller that does not exist. `docs/changelog.md:2680`
is honest and describes only the inline substitution, which does work: each
held-back picture leaves `[Picture not shown: <the sender's description>]` where
it was.

**What actually happens to a user.** Someone reading a marketing message with a
screen reader meets that bracket thirty times and is never told there were
thirty, never told why in one sentence, and never told where the switch is. The
inline markers carry the fact; the orientation and the way out are the part that
is missing. This is the module the project's own principles use as its example
of a privacy decision worth explaining.

**Cost to close.** Small. `wrap_body` at `html_renderer.rs:393` would keep the
count and put `pictures::what_was_held_back(n)` above the body, or hand it to
the announcement path. The sentence and its tests already exist.

**Verified.**

## 4. Smooth scrolling is honoured by one of the two places its own module names

**What it is designed to do.** `application::scrolling`'s module doc:

> Two places need the answer and they are not near each other: the reader's web
> view, which scrolls with CSS, and the message list, which scrolls through
> wxWidgets. One decision, made here, read by both.

**What production asks of it.** One reader.

```
$ grep -rn "how_to_scroll\|css_scroll_behavior" src/ --include=*.rs | grep -v "^src/application/scrolling.rs"
src/presentation/html_renderer.rs:456:    let scrolling = crate::application::scrolling::how_to_scroll(
src/presentation/html_renderer.rs:462:        .css_scroll_behavior();
```

The message list reads only `Following::from_setting` at `wx_app.rs:15477`, and
that answers the separate question of whether the view chases the chosen row.
Nothing in the list reads `Motion`.

There is a second, smaller narrowing in the same module.
`what_the_machine_has_overruled(asked_for_smooth, system)` is written to return
an empty string when nothing has been overruled, and its only production call
site hardcodes both arguments inside a guard that has already decided the
answer:

```
$ grep -n "what_the_machine_has_overruled" src/presentation/wx_settings.rs
435:        let said = what_the_machine_has_overruled(true, SystemMotion::Reduced);
```

The enclosing `if system_motion() == SystemMotion::Reduced` at line 434 ignores
`config.smooth_scrolling`, so somebody who has the box unticked is told that
Windows has overruled a setting they never turned on.

**What claims otherwise.** The module doc, quoted above. The checkbox label,
"Slide when a view scrolls, rather than jumping", makes a general claim.
`docs/changelog.md:2857`, "Scrolling can slide rather than jump", names no
scope.

**What actually happens to a user.** Ticking the box changes the message body
only. Whether that costs anything depends on whether a wxWidgets list would
animate at all, which I did not test, so the practical harm may be nil and the
overclaiming label is real either way. **This half is inferred**: I read the
code, not a running window.

**Cost to close.** Either wire the list, or narrow the three claims. Narrowing
is a few lines and is probably right, since the wx list has no animation to
suppress. Fixing the settings note is one line: pass `config.smooth_scrolling`
rather than `true` and drop the guard, letting the function's own empty-string
case do its job.

**Verified** for the call graph; the user-visible consequence is inferred.

## 5. A doc comment names three callers of `parse_ical_vevent`; there is one

`src/service/caldav.rs:711` says "two of this function's three callers see one
VEVENT per document and have no second one to miss. `parse_report_events` is
the caller that does, and it asks `every_event_in_the_resource` instead."

`parse_report_events` does not call `parse_ical_vevent` at all, and there is one
production caller in the tree:

```
$ grep -rn "parse_ical_vevent(" src/ tests/
src/service/caldav.rs:714:pub fn parse_ical_vevent(ical_data: &str, url: &str, etag: Option<&str>) -> Option<CalDavEvent> {
src/service/ical_subscription.rs:108:    match parse_ical_vevent(&document, "", None) {
src/application/caldav_sync.rs:3398:        let read = crate::service::caldav::parse_ical_vevent(
src/service/caldav.rs:3112:        let event = parse_ical_vevent(ical, "https://cal.example.com/event.ics", Some("\"etag1\""));
```

The last two are tests. That caller passes `url: ""` and `etag: None`, which is
right for a read-only feed, so the parameters being constant is by design.

**No user impact.** Listed because the doc is wrong and a later reader will
trust it. **Cost to close:** rewrite two sentences.
**Verified.**

## 6. `.planning/intel/built-and-left.md` cites two dead scaffolding modules as evidence

`src/application/accounts.rs` (`AccountManager`) and `src/application/search.rs`
(`SearchEngine`) are re-exported from `application/mod.rs` and never
instantiated anywhere in production:

```
$ grep -rn "AccountManager\|SearchEngine" src/ --include=*.rs
src/application/accounts.rs:64:pub struct AccountManager {
src/application/accounts.rs:68:impl AccountManager {
src/application/mod.rs:100:pub use accounts::AccountManager;
src/application/mod.rs:104:pub use search::SearchEngine;
src/application/search.rs:16:pub struct SearchEngine {
src/application/search.rs:26:impl SearchEngine {
src/application/search.rs:70:impl Default for SearchEngine {
```

`built-and-left.md` cites `src/application/accounts.rs` as the evidence for
multi-account being built and exercised, and `src/application/search.rs` as
half the evidence for full-text search. Both features do ship; they live in
`src/data/message_cache/accounts.rs` and the FTS tables in
`src/data/message_cache/`. So the features are real and the citations point at
unused scaffolding.

This is the failure mode that document itself was written to prevent. It is not
a product defect and it is not what this hunt is for, but somebody re-verifying
that table will chase it, so it is worth one line.

**Cost to close.** Repoint two evidence cells. Separately, deleting the two
scaffolding modules is a `dead-code-hunter` job, not this one.
**Verified.**

---

# Silent

Production is narrower than the design and nothing anywhere says so, either way.

## 7. Accepting a meeting invitation never puts it on your calendar

**What it is designed to do.** `Ready::what_the_calendar_should_hold(answer,
already_here)` returns an `OnTheCalendar`: the meeting's uid, version, who
answered, the answer, whether it blocks time, and what changed against what the
calendar already holds.

**What production asks of it.** Nothing. `OnTheCalendar` is produced and
consumed inside `answering.rs` alone:

```
$ grep -rn "OnTheCalendar" src/ tests/ | grep -v AlreadyOnTheCalendar
src/application/answering.rs:320:    ) -> OnTheCalendar {
src/application/answering.rs:321:        OnTheCalendar {
src/application/answering.rs:356: pub struct OnTheCalendar {
```

The handler is explicit about its own scope, at `wx_app.rs:11757`: "What is
here is finding the invitation, sending what it produces, and saying what
happened either way." Nothing else does the calendar half. The announcement
after accepting, `invitations::what_happened`, says "Accepted <meeting>. <the
organiser> will be told" and correctly does not claim a calendar entry.

**What claims otherwise.** Nothing, in either direction, which is the point.
`docs/changelog.md:2488` is carefully worded about the reply and says nothing
about the calendar. Someone who accepts a meeting and then opens the calendar
finds nothing there, with no sentence anywhere having told them that is how it
works.

Two more pieces of the same feature are unreached.
`Ready::what_pressing_it_will_do(answer, when_in_words, said_before)` composes
the sentence describing what Accept, Tentative or Decline will do before it is
pressed, including "you said Tentative before". Nothing calls it, so each of the
three buttons is unlabelled beyond its own word.

**History.** `git log --oneline -S` on `the_calendar_part`,
`what_the_calendar_should_hold` and `what_pressing_it_will_do` returns the same
single commit `592ba56` for all three. The whole second half of that feature
was written and never connected.

**Cost to close.** Moderate. Writing an `OnTheCalendar` means a cache write and
a decision about which calendar an accepted meeting lands in, which is a
question the type does not answer and somebody has to. `what_pressing_it_will_do`
is small: the answer menu would ask it and put the sentence on each item.
Whichever is done, the changelog needs the honest sentence in the meantime.

**Verified.**

## 8. A block can be made and there is no list of what is blocked

**What it is designed to do.** `application::blocking` carries a complete
unblock side. `everyone_blocked(account_id, rules) -> Vec<Blocked>` lists every
block on an account, each with the rule to delete and whether it is switched on.
Its doc names the failure exactly:

> A block somebody cannot find is a trap. Mail stops arriving, nothing says why,
> and the rule doing it is one row among however many rules they have.

`the_rule_that_blocks` finds the exact rule for one block and refuses to widen
("Unblocking one address by deleting the domain block that catches it would
unblock everybody at that domain"). `what_unblocking_did` is the sentence said
afterwards. `what_blocking_will_do` is the sentence said before.

**What production asks of it.** Blocking, and nothing else. The whole live flow
is `block_the_sender` at `wx_app.rs:25350`, and it ends at `what_blocking_did`
(past tense) at line 25481.
None of the four functions above has a production caller:

```
$ grep -rn "everyone_blocked\|the_rule_that_blocks\|what_unblocking_did\|what_blocking_will_do" src/ tests/
src/application/blocking.rs:570:pub fn what_blocking_will_do(
src/application/blocking.rs:651:pub fn what_unblocking_did(block: &Block) -> String {
src/application/blocking.rs:696:pub fn everyone_blocked(account_id: &str, rules: &[MessageFilterRule]) -> Vec<Blocked> {
src/application/blocking.rs:717:pub fn the_rule_that_blocks<'a>(
src/application/blocking.rs:1424: ... (19 lines in all; the 15 not shown are above line 752, so all tests)
```

The `#[cfg(test)]` module opens at line 752, so every reference past it is test
code. `grep -rniE "unblock" src/presentation/` returns nothing.

**What claims otherwise, and why this is Silent rather than Contradicted.**
`docs/changelog.md:1852` says "A block is an ordinary rule, so it appears in
your rules list where you would look for it, and it can be found and undone
there." That is true: blocks are stored as filter rules named `Blocked: ...`,
and the rules manager deletes rules at `src/presentation/managers.rs:326`. So
the documented route works. What nothing says is that the dedicated list was
designed, tested and is not offered, or that finding a block means reading
rule names in a list that also holds every other rule.

The missing "before" sentence is a smaller but sharper gap. Production does warn
about mailing lists via `MayBlock::YesButFirst` and then blocks anyway, with the
comment "Said before the block is made, not instead of making it." What is never
said before is `what_blocking_will_do`, which is the sentence naming which
folder the mail will go to, whether that folder will be switched on for
download, and whether Allowed Changes will hold the move back. All of that is
said afterwards instead.

**History.** `git log --oneline -S` on all three unblock functions returns the
single commit `c8132e4`. Born unwired.

**Cost to close.** A Blocked Senders screen is real work: a dialog, a list, a
delete, an announcement, keyboard reach, and a settings-screen entry point.
Everything below the dialog exists and is tested. A cheaper half is to make the
rules list show blocks distinctly, which it may already do by name.

**Verified.**

## 9. An archive import counts what it could not read and never says it

**What it is designed to do.** `MailboxArchive` carries two counts, and the doc
on the first assigns the duty:

> How many things in the archive could not be read at all. Not nought is
> somebody's mail still sitting in their archive, so it is the caller's job to
> say so at the end rather than leave it in a log.

The second is `how_many_were_too_deep_to_follow`.

**What production asks of it.** Neither.

```
$ grep -rn "how_many_could_not_be_read\|how_many_were_too_deep_to_follow" src/ tests/
src/service/mailbox_archive.rs:697:    pub fn how_many_could_not_be_read(&self) -> usize {
src/service/mailbox_archive.rs:702:    pub fn how_many_were_too_deep_to_follow(&self) -> usize {
```

Two hits in the whole tree, both the definitions. The import at
`wx_app.rs:11706` opens the archive, walks it, and ends at
`import_tree::what_the_folder_import_did(&counted)` at 11750. That sentence
reports folders, messages, `filed_together`, `held_no_mail` and `names_refused`,
and `FoldersImported` at `import_tree.rs:324` has no field for either of the
archive's two counts.

Message-level unreadability is reported, through a different type:
`importing_messages.rs:399` has `could_not_be_read` and 468 says it. So a
message that will not parse is named. An archive **entry** that will not open at
all, and a folder nested deeper than the program follows, are not.

**What claims otherwise.** Nothing either way. The changelog describes the
import; no document says what it does with entries it cannot open.

This one sits directly against the project's ninth principle, "Don't silently
absorb upstream failures", and against the reasoning the codebase already
applied to the export path: `export_tree::what_the_folder_export_did` reports
every one of its four shortfalls, with a comment saying an archive that quietly
leaves files out "is a backup that looks complete and is not".

**Cost to close.** Small. Two fields on `FoldersImported`, filled from the
archive after the walk, and two sentences in `what_the_folder_import_did`
written in the shape of the three already there.

**Verified.**

---

# Documented

The limit is stated where somebody would meet it. Listed so you can see the
filter ran.

| Narrowing | Where it is said |
|---|---|
| `directory::look_up`'s `password` parameter is `None` at its only production call site, `finding_people.rs:160` | `docs/changelog.md:1840`: "A directory that requires a sign-in is not supported yet. Wixen Mail has nowhere to keep a password for one, and it says so rather than signing in with a blank password" |
| `local_folders::naming_a_folder` and `escape_leaf` have no production caller, because folders on this computer cannot be created | `wx_app.rs:7461`, the constant `FOLDERS_ON_THIS_COMPUTER_ARE_NOT_MADE_YET`, whose own doc says "Gated rather than half-built". The user is told in a sentence |
| The folder tree builds one account's branch, though `folder_tree::rows`, `favourites::in_account_order` and `pinned_rows` are all multi-account | `.planning/phases/01-folders-and-conversations/01-08-SUMMARY.md:275`, which states it and says nothing is wrong because the defect it prevents cannot occur yet |
| `service/protocols/pop3.rs:289 stat` has neither a production caller nor a test | Consistent with POP3 being listed under "Built but unproven" in `built-and-left.md`, though the total absence of a test is worth someone's attention |

---

# Ruled out

Checked, and not findings. Each had no production caller, which is why the sweep
raised it, and each turned out to have a live sibling doing the same job.

| Raised | Why it is not a gap |
|---|---|
| `SavedSearch::what_it_found` | Delegates to `what_a_search_found`, which production calls at `wx_app.rs:6746`, `6780`, `6891` |
| `calendar::why_that_day_cannot_be_kept_on_its_own` | A composition of `the_zone_that_cannot_be_written` and `one_day_cannot_be_kept`. Both halves are wired, at `managers.rs:663` (before the editor opens) and `calendar.rs:2003` (before the write). The protection ships |
| `wire::unstuff`, `wire::is_end_of_data` | `&str` twins of `unstuff_bytes`, which `pop3.rs:371` calls. Dot-stuffing is undone |
| `OAuthService::is_expired` | A zero-margin twin of `expires_within`, which `oauth.rs:786` calls with a five-minute refresh margin |
| `tagging::is_a_label_keyword` | `MessageCache::match_labels_to_keywords` iterates over this account's own tags and asks whether the server holds each keyword, so a foreign keyword is ignored structurally. The rule holds without the helper |
| `importing_messages::what_the_mail_export_did`, `writing_out` | The single-file export variant. Production exports a folder and its subfolders through `what_the_folder_export_did`, which reports the same facts including `not_on_this_computer` |
| `emptying::includes_what_is_under_it` | An unused accessor. `Reach::AndEverythingUnderIt` is matched at `emptying.rs:207` and built by `Reach::of` at `wx_app.rs:8373` and `8648` |
| `saved_searches::is_a_saved_search` | A second spelling of the `SEARCH_PREFIX` test. Saved searches are wired throughout `wx_app.rs` |
| `service/safebrowsing::needs_asking`, `prefix_of`, `from_prefixes`, `urls::to_url` | Internal helpers of a subsystem that is wired at `wx_app.rs:18639` to `18673` |
| `search::index_text`, `accounts::add_account`/`get_account`/`get_accounts` | Dead scaffolding, covered as finding 6. Not narrowing, just unused |
| Constant enum argument `Scope::Subtree` at `directory.rs:626` | `Scope` is `ldap3`'s type, and subtree is the right scope for a directory search |
| Struct field `metadata: None` at `contacts_sync.rs:2955` and `3021` | Google People API response-only field, correctly absent on a request |

---

# What this did not cover

- Only `src/application/` and `src/service/` declarations were swept.
  A capability declared in `src/data/` or `src/presentation/` and narrowed
  the same way would not appear here.
- The constant-argument sweep reads literals: `true`, `false`, `None` and
  qualified enum values. A parameter that always receives the same *variable*,
  or the same value through a wrapper, is invisible to it. Finding 4's settings
  note was caught only because both its arguments happen to be literals.
- Trait methods dispatched through a `dyn` object are counted by name, so a
  method reached only through a trait object is counted as used. That direction
  produces false negatives, not false positives, so nothing reported here rests
  on it.
- Nothing was run. No `cargo`, no test, no build, per the read-only constraint.
  Every claim is from reading source, and the two places where a user-visible
  consequence is inferred rather than read say so.

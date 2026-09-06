# Built wider than production asks: `src/presentation/` and `src/data/`

Read-only audit, 2026-09-06. Nothing in the repository was written, and no `cargo`,
`check.sh`, `guards.sh` or `*.test.sh` was run.

**Which tree this is against, because it moved twice during the audit.** The session opened
with a snapshot saying `main` at `8d73579`, clean. That was stale: the executor is on branch
`picture-decorative-answer`, which was already about eighty commits ahead, and it kept
committing while I worked. Everything below was read from the working tree and then
**re-verified line by line against `33c38d3`** (2026-09-06 09:40:37), which is HEAD as this
is written. `src/data/config.rs` moved by about thirty lines mid-audit and its citations are
the corrected ones. Uncommitted at the time of the recheck: `.planning/WINDOWS.md`,
`Cargo.lock`, `Cargo.toml`, `docs/changelog.md`, `src/presentation/html_renderer.rs`,
`src/presentation/wx_settings.rs`. Of those only `changelog.md` and `wx_settings.rs` are
cited, so treat those two line numbers as the softest.

Every finding below was confirmed to still hold at `33c38d3`. One had to be reframed on the
recheck, and that reframing is written into finding 1 rather than quietly folded in.

Every claim below carries the command that produced it. Where I relied on a background
sweep rather than reading the code myself, the finding says so.

Two method notes, because both cost me false positives in this session and both would cost
anyone repeating it:

1. **The first `#[cfg(test)]` in a file is not the test boundary.** `wx_app.rs` interleaves
   test modules with production code: the first is at line 20550 and `fn
   ask_about_the_alpha_once`, called from startup at line 5392, is defined at line 24225 at
   column zero. Any sweep that scores call sites by position against the first
   `#[cfg(test)]` will report production code as test code in that file.
2. **A field read through a captured local is invisible to a `.field` grep.** Four dialog
   controls looked write-only and all four were fine: the builder creates a local, clones it
   into the returned handles, and the closure reads the local. `fav_check.get_value()`
   never mentions the struct.

---

## Contradicted

Something claims the general behaviour and production does not do it.

### 1. Two tick boxes that cannot do different things, and a changelog entry that promises they can

**Corrected on the recheck, and the correction matters.** My first pass called the routing
below an oversight and said nothing tested it. Both were wrong, and I found the second one
only because the tree moved and I re-read the file. There is a test, at
`src/presentation/accessibility.rs:471-493`, named
`test_the_words_still_go_out_when_speech_is_off_and_braille_is_on`, and its comment states
the whole thing plainly:

> The case this module was written for. Speech and braille ride one notification, so either
> one on its own has to release it; requiring both leaves a deaf-blind user with nothing at
> all when a send fails.
>
> Proves the text was released to the bridge, not that a braille display showed it. Only a
> screen reader run says the second.

So the coupling is deliberate, reasoned, tested, and the safe choice given one notification
channel. My grep missed it because I searched case-sensitively for the type name `Braille`
and the test says `braille` in prose.

What survives is narrower and still worth fixing: **the mechanism cannot do what the
Settings screen and one changelog entry tell people it does.**

**What the user is offered:** `src/presentation/accessibility/feedback.rs:319-332` declares
`Channel::{Speech, Braille, Earcon, Visual}` and `Channel::ALL`. Settings builds one tick box
per channel (`src/presentation/wx_settings.rs:1911`, `for channel in Channel::ALL`) and
writes each back independently (`wx_settings.rs:2163`). The wording is at
`feedback.rs:366-373`:

```
Channel::Speech  => "&Speak events through the screen reader",
Channel::Braille => "Send events to a &braille display",
```

**What production does:** `src/presentation/accessibility.rs:251-254`.

```rust
// Speech and braille both ride the one screen reader notification, so
// announcing once serves either. Announcing twice would double the
// speech for anyone who has both.
if channels.contains(&feedback::Channel::Speech)
    || channels.contains(&feedback::Channel::Braille)
{
    self.announce_topic(&text, event.priority(), event.key())?;
}
```

One `UiaRaiseNotificationEvent`, which the screen reader routes to speech and to a braille
display by its own settings, not ours. So speech off with braille on is still spoken, and
braille off with speech on is still brailled. Neither tick box does what its label says
unless both are set the same way. Given one notification channel, the OR is the right
choice; the problem is that the screen still asks the question as though there were two.

**What claims otherwise:** `docs/changelog.md:9184`, the entry that introduced the feature:

> **Feedback on four channels: speech, braille, sound, and the status bar.** ... A new
> Feedback tab in Settings decides which channels each one reaches. This matters most to two
> groups pulling in opposite directions: a deaf-blind user can switch speech off and keep
> braille, and someone working in an open office can swap a spoken sentence for a short tone.

The deaf-blind case is the one that does not work, and it is named as the reason the feature
exists. Two later entries repeat it, `docs/changelog.md:3459` ("an earcons-only or
braille-only setup heard nothing") and `:3473` ("earcons-only and braille-only setups learn
this too").

The correct statement is already in the repository, at `docs/accessibility.md:47-51`:

> Braille follows speech rather than needing its own path: announcements go through
> `UiaRaiseNotificationEvent` ... A braille display should show the same announcements a
> sighted user would hear spoken, without a separate setting.

So `accessibility.md` says braille needs no separate setting, `changelog.md` says switching
speech off keeps braille, Settings shows two separate tick boxes, and the code has one
notification and a test explaining why it must fire for either. Three of those four cannot
all be right.

**Cost to close.** Documentation plus one control, and no change to the routing, which is
correct as it stands. Correct `changelog.md:9184` so it stops promising a speech-off
braille-on setup. Then either merge the two tick boxes into one labelled for what actually
happens ("Send events to the screen reader, which speaks them and shows them in braille"),
or keep two and say in the tab that on Windows they move together. Making them genuinely
independent needs a braille path that does not go through `UiaRaiseNotificationEvent`, which
is real work and may not be reachable through that API at all; the test's own comment is
already the honest note to lift into the tab.

**Verified**, by reading every line cited, twice, at two different commits.

### 2. The documented way to the composer toolbar is the one that does not work

**Designed to do:** `docs/KEYBOARD_SHORTCUTS.md:610` and again at `:617` and `:753`:

```
| Go to the toolbar | `Ctrl+\` | Move to the Send button at the top of the window. The arrow keys move along the toolbar from there |
```

> To reach the toolbar, press `Ctrl+\` from anywhere, or `Shift+Tab` from the From line.

**What production does:** `src/presentation/editor_document.rs:294-309`, in the script
injected into the composer's editing surface. The project's own comment:

> Ctrl+backslash was the key asked for and it does not arrive. Measured against the running
> composer: the page's handler never fires for it, by character or by physical key, whether
> the chord is typed or injected as raw key events, while Ctrl+Shift+L and Ctrl+Enter beside
> it arrive every time. Something between the window and the page keeps it. It stays bound
> in case that is this machine rather than every machine, and **F8 is bound beside it so the
> toolbar has a way in that was watched working.**

`F8` is the working key and appears nowhere in the composer section of the document. Every
`F8` in the file is about something else:

```
$ grep -n "F8" docs/KEYBOARD_SHORTCUTS.md
152:Either way, `F8` reaches the attachments and `Ctrl+S` saves the one you are on.
178:| Attachments | `F8` | Moves between the message and the list of attachments, when there is one |
189:`F8` jumps to the list from anywhere in the message, and `F8` again goes back to
514:| Columns | `F8` | Choose which message list columns are shown and in what order |
524:A bare `F8` on purpose: choosing columns is a verbosity control, something you
```

Lines 152, 178 and 189 are the reader; 514 and 524 are the View menu. The composer section
runs 605 to 870.

**Why the guards miss it.** `tests/wired.rs` holds the doc/binding pair. Its doc-to-code
direction filters at `tests/wired.rs:1067` with
`piece.starts_with("Ctrl+") || piece.starts_with("Alt+")`, so no unmodified key is ever
checked. Its code-to-doc direction reads `wx_app.rs` alone, and `editor_document.rs` is a
web page rather than a menu table, so it is outside the sweep twice over. `Ctrl+\` is
allow-listed at `tests/wired.rs:1038`, so the pair is green by declaration.

**Cost to close.** One documentation line: name `F8` at `KEYBOARD_SHORTCUTS.md:610` and
`:617`, and mark `Ctrl+\` as not currently arriving rather than as the way in. The person
this hurts is someone working entirely by keyboard who reads the document, presses the key
three times, and concludes the toolbar is unreachable.

**Verified** by reading `editor_document.rs:292-322` and every doc line cited. I did not run
the composer, so the claim that `Ctrl+\` fails is the project's own measurement reported on,
not mine.

### 3. Restore Defaults in the Columns dialog always restores the Inbox columns

**Designed to do:** `docs/KEYBOARD_SHORTCUTS.md:533`:

```
| Restore the defaults | `Alt+R` | Puts back the default columns for this kind of folder |
```

The layout genuinely has a kind. `src/presentation/message_columns.rs:562-585`,
`ColumnLayout::defaults_for(kind)` gives Sent and Drafts a different column set, without
Unread and sorted by when a message was sent. `wx_columns.rs:247` is the Reset handler:
`working.borrow_mut().reset(kind)`, and `kind` is the dialog's third parameter
(`wx_columns.rs:71`, `:104`).

**What production does:** the only production caller passes a literal.

```
$ grep -rn "show_column_dialog" src/ --include=*.rs
src/presentation/wx_app.rs:4042:                                wx_columns::show_column_dialog(
src/presentation/wx_columns.rs:68:pub fn show_column_dialog(
```

`src/presentation/wx_app.rs:4039-4047` (the call itself is at `:4042`, the literal at `:4045`):

```rust
_ if id == ID_VIEW_COLUMNS => {
    let current = column_layout.borrow().clone();
    if let wx_columns::ColumnDialogResult::Updated(chosen) =
        wx_columns::show_column_dialog(
            &frame,
            &current,
            message_columns::FolderKind::Inbox,
            &a11y,
        )
```

So in Sent, `Alt+R` puts back the Unread column and the Received sort, which is exactly what
`defaults_for(FolderKind::Sent)` exists to avoid. `message_columns.rs:517-521` records why:

> Nothing asked this before. Every call site named `Inbox`, so the Sent and Drafts layout ...
> was written and tested and never once used: the Unread column was read out on every row in
> Sent, where it says the same thing every time.

**This is a partial fix, not an old bug.** `git log -S "FolderKind::for_folder" --oneline`
returns one commit, `98546f8` "Give Sent and Drafts their own columns, and delete five
things that protect nothing" (2026-08-24). It fixed the folder-switch site at
`wx_app.rs:2741` and left the dialog site hardcoded.

**Cost to close.** One line: read the open folder's kind the way `wx_app.rs:2739-2741`
already does and pass that instead of the literal. The kind is already in hand as
`column_layout.borrow().kind`.

**Verified** by reading all four sites.

### 4. Shift+F6 goes forward inside the message preview

**Designed to do:** `docs/KEYBOARD_SHORTCUTS.md:349`:

```
| Previous pane | `Shift+F6` | Move focus to the previous pane and say which one |
```

**What production does:** `src/presentation/wx_app.rs:10784-10792`, the script injected into
the preview and into the conversation window:

```js
document.addEventListener('keydown', function(e) {
    // F6 matches whether or not Shift is held, so Shift+F6 leaves too. Where
    // it lands is the host's decision, which is why the key is not named here.
    if (e.key === 'Escape' || e.key === 'F6') {
        e.preventDefault();
        e.stopPropagation();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'leave' }));
    }
}, true);
```

The payload carries no shift flag, and the host cannot recover one: `is_leaving`
(`wx_app.rs:10806-10816`) reads `kind` and nothing else. The preview handler
(`wx_app.rs:1287-1296`) then does `msg_list.set_focus()` unconditionally. So the comment's
"the host's decision" is a decision the host has no data to make.

**The sibling surface does it correctly**, which is what makes this a gap rather than a
platform limit. `src/presentation/editor_document.rs:318`:

```js
post({{ kind: 'leave', back: event.shiftKey }});
```

**Why the guards miss it.** `test_f6_and_shift_f6_reach_the_pane_handler`
(`tests/wired.rs:854-876`) asserts the literal strings `\tF6` and `\tShift+F6` appear in
`wx_app.rs`, which they do, at `wx_app.rs:5802`. It does not follow either key into the
WebView.

**Cost to close.** Add `back: e.shiftKey` to the preview payload, read it in `is_leaving`
or beside it, and branch the focus target. The editor already shows the shape.

**Verified** by reading the script, `is_leaving`, and both handlers.

### 5. The CalDAV change marker is asked for, parsed, tested, and thrown away

**Designed to do:** a ctag is the CalDAV mechanism for skipping a calendar that has not
changed. The PROPFIND asks for it (`src/service/caldav.rs:239`, `<cs:getctag/>`), the
response is parsed (`caldav.rs:586`, `let ctag = extract_xml_value(response_block,
"cs:getctag");`) and carried on `CalDavCalendar.ctag` (`caldav.rs:30`, `:593`). There is a
test defending the request, `caldav.rs:7892-7896`:

> Without getctag the sync loses its change marker.

and `caldav.rs:8012` asserts the parse: `assert_eq!(found[0].ctag.as_deref(), Some("ctag-work"));`

The database has the column (`src/data/message_cache/mod.rs:2020-2036`, `calendars.ctag`)
and `CalendarContainer` has the field (`mod.rs:960`).

**What production does:** drops it three times over.

The value is dropped at the boundary. `src/application/calendar_source.rs:460` builds the
saved row from the chosen discovered calendar via `row_for`, and `row_for`
(`calendar_source.rs:305-339`) writes:

```rust
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
```

Every single assignment in the whole tree is `None`:

```
$ grep -rn "ctag:" src/ --include=*.rs
```

returns 28 lines: two are field declarations (`mod.rs:960`, `caldav.rs:30`), two are reads
back out of a row (`calendars.rs:127`, `:173`), one is the unused parameter
(`caldav.rs:284`), and the remaining 23 are all literally `ctag: None`.

The parameter is unused. `src/service/caldav.rs:277-285`:

```rust
    pub async fn list_events(
        &self,
        calendar_url: &str,
        username: &str,
        password: &str,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
        _ctag: Option<&str>,
    ) -> Result<(Vec<CalDavEvent>, Option<String>)> {
```

And the return is hardcoded: the only `Ok((` in that function is `Ok((events, None))`.

The caller discards both ends. `src/application/caldav_sync.rs:804-812`:

```rust
    let (remote_events, _new_ctag) = match caldav
        .list_events(
            calendar_url, username, password,
            Some(asked_from), Some(asked_to),
            calendar.ctag.as_deref(),
        )
```

**What claims otherwise:** nothing in `docs/` mentions a ctag at all (`grep -rni "ctag"
docs/ README.md` returns nothing), so this is not contradicted by a user-facing page. It is
in this bucket because the code contradicts itself: a test defends asking for a value that
three later layers are written not to use, and reads as a working incremental sync.

**Consequence:** every CalDAV sync downloads the whole asked-for window every time,
regardless of whether anything changed.

**Cost to close.** Carry the parsed ctag into `row_for`, use `_ctag` to add an `If-None-Match`
or a ctag comparison before the REPORT, return the new one, and store it. That is real work
against a live CalDAV server, which this project says none of the sync paths has met. The
cheap half is honest: say in the changelog that the change marker is not used yet, and
either delete the parameter or leave a comment beside it saying so, since right now the
`_ctag` underscore is the only thing that admits it.

**Verified** by reading every cited line.

---

## Silent

Production is narrower than the design and nothing says so either way.

### 6. There is one stored column layout, and visiting Sent overwrites it permanently

`ColumnLayout` carries a `kind` and `message_columns.rs:547-553` says why:

> Carried so the window can tell when moving to another folder means a different set of
> columns, and leave the layout alone when it does not: rebuilding on every folder change
> would throw away whatever somebody had just sorted by.

Three facts together defeat that intent.

`to_stored()` does not record the kind (`message_columns.rs:744-754`):

```rust
        format!("{}|{}", columns.join(","), sort)
```

Both read sites hardcode Inbox:

```
$ grep -rn "from_stored" src/presentation/ --include=*.rs | grep FolderKind
src/presentation/wx_app.rs:1077:  ColumnLayout::from_stored(stored, message_columns::FolderKind::Inbox)
src/presentation/wx_settings.rs:1431:  let layout = ColumnLayout::from_stored(stored, FolderKind::Inbox);
src/presentation/wx_settings.rs:1450:  let mut layout = ColumnLayout::from_stored(stored, FolderKind::Inbox);
```
(`wx_app.rs:12305` is the third, inside `the_sort_as`.)

And a kind change replaces the layout with defaults rather than restoring a stored one
(`wx_app.rs:2737-2748`):

```rust
    let kind = message_columns::FolderKind::for_folder(stored_kind);
    let mut layout = column_layout.borrow_mut();
    if layout.kind != kind {
        *layout = ColumnLayout::defaults_for(kind);
        apply_columns(&msg_list, &layout);
    }
```

There is exactly one stored string, `AppConfig.message_columns` (`src/data/config.rs:433`),
written by `persist_column_layout` (`wx_app.rs:12916-12927`) from whatever layout is in
effect. Its callers are `wx_app.rs:2928` (a column header click), `:4075` (OK in the Columns
dialog) and `:12884` (the Sort menu).

So the sequence a user will actually hit: open Sent, click a column header to sort. The Sent
layout, which has no Unread column and sorts by the sender's claimed Sent date, is written
into the one stored string. Restart. `wx_app.rs:1077` reads it back as an Inbox layout, and
because `layout.kind` now equals Inbox nothing rebuilds it, so the inbox opens with no
Unread column and sorted by a field the code itself warns about
(`message_columns.rs:54-56`):

> When the sender says they sent it, which is sender controlled and often wrong. Sorting an
> inbox by it puts forged-date spam permanently on top.

Within one session the smaller version also holds: a hand-chosen Inbox layout is discarded
the moment the user visits Sent and comes back, because the return trip also calls
`defaults_for`.

**Nothing says so.** `docs/USER_GUIDE.md:220` promises per-folder memory only for the Thread
column ("If you show or hide it yourself in View, Columns, your choice wins from then on in
that folder"), and that one is genuinely per folder via
`cache.set_folder_thread_column` (`wx_app.rs:4058`). No page says the rest of the layout is
one global setting, or that it is shared between folder kinds that have different defaults.

**Cost to close.** Two options. Small: append the kind to `to_stored()` and read it back, so
a stored Sent layout is recognised as one, and keep the last-used layout per kind in memory
so returning to the Inbox restores rather than resets. Larger, and what the Thread column
already does: store a layout per folder.

**Verified** by reading all of it. I did not run the program, so the restart sequence is
traced through the code rather than observed.

### 7. `EdgeOfList` is designed as a navigation tick and fires for one unrelated thing

`feedback.rs:38-39` documents the event as "The cursor tried to move past the first or last
row", its written form is "End of list" (`feedback.rs:167`), and `feedback.rs:242-244` calls
it one of "the two navigation events ... single short ticks so they do not sound like status
at all".

Production fires it once, and not for that:

```
$ grep -rn "Event::EdgeOfList\|FeedbackEvent::EdgeOfList" src/ --include=*.rs
src/presentation/accessibility/feedback.rs:116:        Event::EdgeOfList,
src/presentation/accessibility/feedback.rs:137:            Event::EdgeOfList => "edge_of_list",
src/presentation/accessibility/feedback.rs:167:            Event::EdgeOfList => "End of list",
src/presentation/accessibility/feedback.rs:230:            | Event::EdgeOfList
src/presentation/accessibility/feedback.rs:246:            Event::EdgeOfList => Tone::new(440, 40),
src/presentation/accessibility/feedback.rs:1481:        settings.set_event_channels(Event::EdgeOfList, BTreeSet::new());
src/presentation/accessibility/feedback.rs:1482:        assert!(settings.channels_for(Event::EdgeOfList).is_empty());
src/presentation/wx_app.rs:3443:  .signal(FeedbackEvent::EdgeOfList, "no unread messages");
```

`wx_app.rs:3417-3444` is the Next Unread / Previous Unread handler: when `next_unread`
returns `None` it signals `EdgeOfList` with the detail "no unread messages". `signal`
composes `event.text_with(detail)`, so what a user hears is "End of list" plus that detail,
for a search that came up empty rather than a cursor reaching an edge. There is an event for
exactly that case: `Event::NothingFound`, documented at `feedback.rs:103-108` as "A search
or a filter completed and matched nothing ... coming up empty is a meaningfully different
fact, gentle rather than alarming".

Arrowing past the top or bottom of any list, in mail or in the five PIM modules, signals
nothing. The reader window does say "Last message" and "First message"
(`wx_reader.rs:761-763`) but through `a11y.announce` directly, so it bypasses the channel
routing entirely: a user with speech off and earcons on hears nothing there.

**Cost to close.** Change the one site to `NothingFound`, then decide whether the edge tick
is wanted at all. Wiring it means one call in each list's selection handler, which is six
places, and each needs to know it was already at the end. Cheaper interim: delete the event
and say why, rather than ship a tone in Settings that fires for one unrelated case.

**Verified.**

### 8. Two per-account settings are stored, survive a restart, and are read by nothing

`src/data/config.rs:686-700`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    ...
    /// Default folder
    pub default_folder: String,
    /// Auto-download attachments
    pub auto_download_attachments: bool,
}
```

These are serialized per account (`config.rs:840` writes one JSON file each, `config.rs:808`
reads them back and validates). A repo-wide search across every file type, not just `.rs`,
so `docs/`, `.planning/`, `guards/` and `.github/` are all covered, and excluding `target/`
and `.git/`:

```
=== auto_download_attachments EVERYWHERE (repo-wide, all file types) ===
./src/data/config.rs:699:    pub auto_download_attachments: bool,
./src/data/config.rs:711:            auto_download_attachments: false,

=== default_folder EVERYWHERE ===
./src/common/types.rs:334:    pub default_folder: Option<String>,      <- a different struct
./src/data/config.rs:697:    pub default_folder: String,
./src/data/config.rs:710:            default_folder: "INBOX".to_string(),
```

A declaration and a default each, and nothing more anywhere in the repository: no screen, no
document, no requirement, no guard record, no plan. `AccountConfig::new` is called only from
`config.rs:945`, `:954` and `:1018`, all inside `#[cfg(test)]` (the boundaries in that file
are at 861, 1129, 1624 and 1700).

**This is precisely what the two config guards cannot see**, which is worth stating because
CLAUDE.md says a new top-level setting fails on arrival. `stored_setting_names`
(`config.rs:1704`) begins at `config.rs:1706` with `source.find("pub struct AppConfig {")`,
so it reads one struct's fields.

**And it is a third kind of escape, distinct from the two the project already knows about.**
Commit `972b66f`, "Correct three counts and the reason two settings escaped the check",
landed during this audit and rewrote that CLAUDE.md paragraph after phase 6 research went and
read the code. Its finding: `allowed_per_account` is a top-level field the check can see and
is excused by name through the one-entry `STORED_AND_OFFERED_BY_NOTHING`
(`config.rs:1837`), and the per-event feedback channels sit inside a serialised string value,
so no name-based check reaches them at any depth. It concludes that ledger 114 as written,
widen the check to follow nesting, closes neither.

`AccountConfig` is neither of those. It is not nested, not inside a serialised string, and
not excused by name. It is a second `Serialize`-deriving struct in the same file, and the
extractor stops at the first one. Ledger 114 as written would not close this either.

The larger shape behind it: nothing in production ever inserts into `account_configs`, so
the whole per-account config file format is a closed loop that starts empty. Real per-account
settings live in the `accounts` table and in `data::account::Account`. There is also a second
type called `AccountConfig` in `presentation::ui_types`, which is the one
`Account::from_account_config` uses (`src/data/account.rs:331`).

**Cost to close.** Small, and it is a decision rather than a fix: either delete
`data::config::AccountConfig` and its two files-on-disk paths, or widen
`stored_setting_names` to walk every `Serialize`-deriving struct in `config.rs` so a second
struct cannot hide settings from both guards. The second is the one that stops it recurring.

**Verified** by repo-wide search plus reading the guard.

### 9. The internal automation tree holds eight nodes and nothing ever updates or reads it

`src/presentation/accessibility/automation.rs` defines `AutomationStore`, `AutomationNode`,
`AutomationState` (focused, enabled, selected, expanded, checked) and `AutomationRole` with
twelve variants. `Accessibility` owns one (`accessibility.rs:24`).

Production writes eight nodes, once, at `Accessibility::initialize`
(`accessibility.rs:81-124`): `main_window`, `folder_tree`, and six module panes. After that:

```
$ grep -rn "update_node_state" src/ tests/ --include=*.rs
src/presentation/accessibility.rs:171:    pub fn update_node_state(
src/presentation/accessibility.rs:867:        // `update_node_state` promises two things: the automation tree carries
src/presentation/accessibility.rs:877:        a11y.update_node_state("folder_tree", ticked.clone())
src/presentation/accessibility.rs:878:            .expect("update_node_state");

$ grep -rn "live_region_update" src/ tests/ --include=*.rs
src/presentation/accessibility.rs:375:    pub fn live_region_update(&self, region_id: &str, text: &str) -> Result<()> {
src/presentation/accessibility.rs:816:    fn test_live_region_update_notifies_the_tree_and_announces_the_text() {
src/presentation/accessibility.rs:818:        a11y.live_region_update("message-count", "12 unread")

$ grep -rn "automation_snapshot" src/ tests/ --include=*.rs
src/presentation/accessibility.rs:424:    pub fn automation_snapshot(&self) -> Result<Vec<automation::AutomationNode>> {
src/presentation/accessibility.rs:855,880,907   (all inside the tests module)
```

Lines 816 onward are inside `#[cfg(test)] mod tests`. `a11y.set_focus(<id>)` is called from
production once, at `accessibility.rs:125` with `"folder_tree"`; the other two hits
(`focus.rs:54`, `accessibility.rs:917`) are tests, and `focus.rs`'s test module starts at
line 46. The 67 other `set_focus()` calls across `src/presentation/` are the wxWidgets
no-argument method on a control, a different thing.

`FocusManager` is narrower still: `current_focus()` is read only by its own test
(`focus.rs:56`), so the manager stores a value nothing consults. Of twelve `AutomationRole`
variants, production constructs three (Window at `accessibility.rs:84`, List at `:95`, Pane
at `:116`); Button, Text, TextInput, ListItem, Menu, MenuItem, Checkbox, Link and Custom are
never constructed anywhere.

This is not the platform accessibility tree. The real one is wxWidgets' native controls plus
`set_accessible_name` for MSAA and `UiaRaiseNotificationEvent` for announcements, and those
work. But nothing says the internal store is a stub, and its module doc is one line:
"Accessibility automation tree and event models." A reader meeting `update_node_state` has
no way to know it has never been called.

**Cost to close.** Either delete `automation.rs` and `focus.rs` and take `set_focus`'s
"Focus moved to X" announcement with them, or write two sentences at the top of
`automation.rs` saying what it is for, that it currently holds eight fixed nodes, and that
the real accessibility surface is elsewhere. The second costs minutes and is the one that
stops somebody wiring a feature to it.

**Verified.**

### 10. `NativeBridgeStatus` is a compile-time constant that nothing reads

`screen_reader.rs:424-432` declares `Active` and `Fallback`. It is set once, in
`ScreenReaderBridge::default` (`screen_reader.rs:653-658`):

```rust
            status: if cfg!(target_os = "windows") {
                NativeBridgeStatus::Active
            } else {
                NativeBridgeStatus::Fallback
            },
```

`cfg!` is decided at compile time, so on Windows this always says Active whatever happens at
run time: no live region registered, `UiaRaiseNotificationEvent` failing, announcements
piling up in the `held` queue. And nothing asks:

```
$ grep -rn "native_bridge_status" src/ tests/ --include=*.rs
src/presentation/accessibility.rs:429:    pub fn native_bridge_status(&self) -> screen_reader::NativeBridgeStatus {
src/presentation/accessibility.rs:430:        self.screen_reader.status()
```

Definition and body, no callers. This is guardrail 4 in miniature: a status field shaped
like a health check that reports a build flag and is read by nobody. The bridge does track
what would make a real answer, `registered: AtomicBool` and the `held` queue
(`screen_reader.rs:440-442`), so the material exists.

**Cost to close.** Small. Derive the status from `registered` and whether the last
notification call succeeded, and surface it somewhere a person can see, most naturally in
the Accessibility part of Settings or in `--help` output beside the other alpha warnings.

**Verified.**

### 11. Two keys are bound and documented nowhere

Both come from the keyboard sweep; I read and confirmed each.

**`Delete` removes an attachment in the composer.** `src/presentation/wx_compose.rs:1598-1630`,
guarded on `KEY_DELETE` (`wx_compose.rs:59`), removes the selected row from the attachment
list and announces "Removed {name}". The file's first `#[cfg(test)]` is at 3596, so this is
production. Every `Delete` row in `docs/KEYBOARD_SHORTCUTS.md` is about something else
(lines 17, 70, 451, 555, 864), and the composer's attaching section (746-780) covers `Alt+A`
and `Ctrl+V` only. `docs/roadmap.md:137` ticks "Add/remove attachments with file picker"
without naming a key.

**`F6` closes the conversation window.** `wx_app.rs:18423` calls
`wire_the_way_out(&page, "conversation window")`, the same injected script as the preview,
and `wx_app.rs:18426-18432` maps its `leave` message to `frame.close(false)`. The
conversation window's documented keys (`docs/KEYBOARD_SHORTCUTS.md:141-160`) are `Enter`,
`H` and `Escape`. Meanwhile line 347 tells the reader `F6` means "Move focus to the next
pane", so somebody who learned that key elsewhere closes the window with it.

Both are invisible to `tests/wired.rs` for the reasons given in finding 2: unmodified keys
are filtered out of the doc-to-code direction, and the code-to-doc direction reads
`wx_app.rs` menu labels only.

**Cost to close.** Two documentation lines. The `F6`-closes-the-conversation-window
behaviour may also be worth a second look on its own merits, since it does not match what
the key does anywhere else.

**Verified** by reading the binding sites and grepping the document.

### 12. Storage: columns written and never read, and columns that can only hold one value

From the schema sweep. I re-verified the CalDAV group myself (finding 5); the rest I am
reporting on the sweep's evidence, which quoted commands and output for each.

| Column | State | Evidence |
|---|---|---|
| `contact_group_members.added_at` | written with a real timestamp, never read | only occurrence outside `CREATE TABLE` is the INSERT at `contacts.rs:1125`; the two SELECTs over that table (`contacts.rs:1145`, `:1166`) name `contact_id` and `c.email` |
| `message_tags.created_at` | written, never read | written at `tags.rs:158`; every `created_at` read in `tags.rs` is `t.created_at` on the `tags` table |
| `calendars.etag`, `.ctag`, `.sync_token`, `.refresh_interval_minutes` | always NULL | all four production writers (`calendar_source.rs:334-337`, `calendars.rs:224-227`, `:280-283`, `managers.rs:2873-2876`) write `None` |
| `calendars.display_order`, `note_folders.display_order`, `tasks.display_order` | always 0, and each is the leading `ORDER BY` term | `calendars.rs:108`, `notes.rs:36`, `tasks.rs:172` and `:197`; contrast `task_lists.display_order`, which is genuinely varied at `tasks_api.rs:302`, `:517` |
| `notes.format` | always `"plain"`, never read | six production writers all write the literal; nothing reads `.format` on a note |
| `reminders.related_event_id` | always NULL | the only production `save_reminder` caller is `managers.rs:3048`, writing `related_event_id: None` at `:3063` |
| `messages.body_plain`, `.body_html` | migration-only | the sole production read is `bodies.rs:95`, the constant naming rows still holding inline text, used once at `bodies.rs:653`; the sole production write is `bodies.rs:667`, setting both to NULL. `message_bodies.body_plain`/`.body_html` are the live ones and do carry values |

The schema is additive by rule, so columns outliving their use is expected and mostly
harmless. Two are worth acting on. The `display_order` group is a sort key that cannot be
changed, and there is a working reordering gesture next door: `application/reordering.rs` is
used by accounts (`account_order.rs`) and pinned folders (`favourites.rs`), documented at
`docs/KEYBOARD_SHORTCUTS.md:831-832` as `Alt+Shift+Up` / `Alt+Shift+Down`. Extending it to
calendars, note folders and task lists is the smallest way to make the column mean
something. And `notes.format` is a stringly-typed column with one possible value, which the
project's own elegant-code rule says should be an enum or nothing.

**Cost to close.** Individually small. The honest cheap move for the write-only timestamps is
to leave them (additive schema, no harm) and note in `docs/architecture.md` that they exist
for future use, so the next reader does not go looking for the query that reads them.

---

## Documented

Checked, the limit is stated where somebody would meet it, no action needed. Listed so it is
clear these were looked at and not missed.

- **Calendar Prev and Next buttons.** Created disabled, and their accessible names say
  "Previous period, not built yet" / "Next period, not built yet"
  (`wx_calendar_module.rs:53-58`). The comment above explains that a disabled button reads as
  unavailable on arrival, which is better than one that announces an action and does nothing.
  This is guardrail 3 done properly.
- **The theme spacing scale.** `theme.rs:252-256`: "A four step scale, and nothing lays out
  to it yet: the window passes its own literals to every sizer. This is the scale a layout
  pass would adopt, not a description of the one on the screen."
- **`FolderKind::for_folder`.** Its doc comment (`message_columns.rs:517-521`) records the
  all-Inbox bug that finding 3 is the remainder of.
- **The folder tree is one flat level in the chooser.** `wx_folder_choice.rs:14-19` gives the
  reason a checked list is used instead of a tree.
- **`F2` is neither bound nor documented, on purpose.** `wx_app.rs:6098` says a new chord
  needs a documentation line in the same commit and that `F2` is not free here.
- **`Answered` and `Draft` columns.** `mod.rs:2573` says the columns were withdrawn because
  nothing could fill them and a sync now does; `mail_sync.rs:583` confirms
  `answered: message.answered()`. Not a finding.
- **Theme, including High Contrast.** All four variants are offered by the settings dropdown
  (`wx_settings.rs:676`) and `palette()` returns `None` for High Contrast with a stated
  reason.
- **`accessibility.md:47-51`** states the braille limitation correctly, which is what makes
  finding 1 a contradiction between documents rather than an undiscovered fact.
- Excluded by the brief and confirmed still true: per-event feedback channels
  (`feedback.rs`, `per_event` and `set_event_channels` are private), `allowed_per_account`
  (named in the one-entry `STORED_AND_OFFERED_BY_NOTHING` at `config.rs:1837`).

---

## What I checked and cleared

So the negative space is visible rather than assumed.

- `Event::ALL`: every one of the sixteen feedback events has at least one production
  producer. No dead events.
- Read-aloud: `wire_read_aloud` is called for all five PIM lists (`wx_app.rs:1554-1612`) and
  for mail (`:3197`), so the module doc's "in every module" is true.
- `set_name` versus `set_accessible_name`: exactly one `.set_name(` survives in the tree,
  `wx_compose.rs:3499`, and the line immediately above it is
  `set_accessible_name(&body_preview, "Message preview")`. Redundant, not a defect. This is
  the trap the brief warns about and it is worth saying it came out clean.
- `favourites.position` and account order: genuinely written (`folders.rs:496-499` MAX+1,
  `:562` UPDATE) and genuinely reachable by keyboard.
- `AppConfig.told_about_the_alpha`: read at `wx_app.rs:24233`, inside a production function
  at column zero despite sitting after a `#[cfg(test)]`.
- `ContactEditDialogHandles.fav_check`, `AddressSubDialogWidgets.country_choice` and
  `.type_choice`, `SigEditWidgets.def_check`: all four read in production through captured
  locals (`wx_managers.rs:1765`, `:2350`, `:2345`, `:3967`). No screen control here writes
  nothing.
- `AppConfig.directories`: written by the account manager (`wx_account_manager.rs:1108-1113`).
  Not an unoffered nested setting.
- `Allowed`: all three fields reachable, and the nested one has its own named guard
  (`config.rs:1867`).

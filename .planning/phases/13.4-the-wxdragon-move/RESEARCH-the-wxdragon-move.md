# The wxDragon move: research

Written 2026-10-20 for Pratik and for whoever plans this phase. Read against `main` at `a72d4b41`.
The three research passes of the same day read `6fd465c6`; nothing under `src`, `tests`,
`scripts`, `.github`, `.cargo`, `installer`, `build.rs`, `Cargo.toml` or `Cargo.lock` changed
between the two:

```text
$ git diff --stat 6fd465c6 a72d4b41 -- src tests Cargo.toml Cargo.lock .github scripts .cargo installer build.rs
(no output)
```

So every line number below holds at `a72d4b41`. Phase 13.3 will move many of them, `wx_app.rs`
most of all, so a planner re-reads each one at the commit it plans against.

Each claim carries one of four marks. **Read** means read in source at the named version.
**Measured** means a command was run, and it says by whom. **Inferred** means reasoned from what
was read and not run. **Stated** means taken from a document or an issue tracker and not checked
here.

## The answer

- **Move to wxDragon 0.9.23**, the newest release (published 2026-10-15). It builds wxWidgets
  3.3.4. Do not stop at 0.9.22: the WebView2 crash fix behind ledger 466 is in wxWidgets 3.3.4
  only.
- **The build breaks in one source file and 69 test files.** `names.rs` has 11 places, and 69 test
  files have 72 places that send the application object to another thread. Linux and macOS would
  break as well (inferred). The earlier research said "one file"; that was true of the library
  only.
- **Two things change quietly, and both can be fixed before the move.**
  1. The message editor's automatic save stops happening on its own (read).
  2. The main window's 50 ms tick stops running while a window it opened is on screen: the Due
     now window, the question about folders that have gone, and the two update windows. While
     one of those is open, no mail, sync or send result is handled and no reminder is raised
     (read). Nobody had noticed this one before today.
- **What the move fixes for people:** list cells that lose their last character (ledgers 28 and
  159), tree rows that are never freed (ledger 41), the WebView2 crash at exit (ledger 466), two
  quick-xml advisories, and newer copies of the image, XML and regular-expression libraries
  bundled inside the program.
- **The toolkit still provides no UI Automation of its own.** Neither wxDragon 0.9.23 nor
  wxWidgets 3.3.4 implements a UI Automation provider, so every name the code sets still reaches
  Narrator only through Windows' own bridge from MSAA, exactly as today (read). That does not make
  the two channels move together: for an edit box or a button Windows supplies its own UI
  Automation provider, which shadows the MSAA object (CLAUDE.md, Accessibility), so a change can
  show on one channel and not the other, and WXD-08 compares both trees. (Corrected 2026-10-20 in
  the plans' revision; this bullet read "Names stay on one channel" until then.)
- **Eleven plans.** Four run first, at 0.9.17, so that the version change itself changes nothing
  the program relies on. Then come the move, three plans that remove what it makes unnecessary
  and correct what still names the old toolkit, the check of every window's names on both
  channels, Pratik's NVDA sitting, and the close. (Ten until 2026-10-20, when the comment pass was
  split out of WXD-07 as WXD-07.1.)
- **Pratik answered the questions on 2026-10-20, "yes to all".** The move runs after 13.3 and
  before phase 14's sittings; the two unused toolkit features are dropped in the move's commit;
  Linux and macOS keep compiling, proved by one dispatched run on the move's branch; the Visual
  C++ runtime gap is a ledger row now and a small plan after this phase. Each answer is under
  "Questions for Pratik" and folded into the plans.

## From what to what

| Crate or library | Today | After the move | Published |
|---|---|---|---|
| wxdragon | 0.9.17 | 0.9.23 | 0.9.17 on 2026-07-01, 0.9.23 on 2026-10-15 |
| wxdragon-sys | 0.9.17 | 0.9.23 | the same days |
| wxdragon-macros | 0.9.17 | 0.9.23 | the same days |
| wxWidgets, built from source by wxdragon-sys | 3.3.2 (released 2026-03-03) | 3.3.4 (released 2026-10-10) | |

Read from crates.io's answers saved on 2026-10-20 (`third-party/crates-io-api-2026-10-20/`,
`max_version` 0.9.23 for all three) and from `WX_VERSION` at the top of each wxdragon-sys
`build.rs`. Releases 0.9.18 to 0.9.22 build wxWidgets 3.3.3 (stated, upstream research).
Upstream has 222 commits between the two tags (stated, GitHub's compare).

**The exact pin does not hold the other two crates.** `Cargo.toml:79` pins `wxdragon = "=0.9.17"`,
and wxdragon asks for `wxdragon-sys` and `wxdragon-macros` with a caret, `version = "0.9.23"` in its
own manifest at 0.9.23 (read). Only `Cargo.lock` holds them. A dry-run update at today's pin
moves both siblings to 0.9.23 while wxdragon stays at 0.9.17 (measured by the build research).
WXD-05 adds a check that the lock holds all three at one version.

**Moving the lock moves nine other packages** (measured by both the build and the upstream
research, a lock re-resolved from a copy of `main`): reqwest 0.13.4 to 0.13.5 (our own HTTP
client), flate2 1.1.9 to 1.1.10 with a new miniz_oxide 0.9.1 (the engine that packs cached message
bodies, `bodies.rs:68`; zlib is a standard format, so caches already written still read), log,
bitflags, quick-xml 0.38.4 to 0.42.0 (build time only), bindgen, rawzip and pkg-config. prettyplease
and shlex leave. 722 packages become 721.

## What breaks the build

| What | Our sites at `a72d4b41` | Upstream change | Fix |
|---|---|---|---|
| The accessibility status constants became an enum | `src/presentation/accessibility/names.rs` lines 101, 103, 119, 134, 135, and in its tests 567, 581, 600, 619, 634, 651; `use wxdragon::ffi;` at 54 and 544 goes unused | 0.9.18, #158: `AccStatus`, `AccRole` and `NavDir` became enums, `AccState` and `AccSelectFlags` bitflags | `AccStatus::Ok` and `AccStatus::NotImplemented`; drop the two `use` lines |
| The application object is no longer `Send` | 72 places in 69 test files, all of the shape `wxdragon::call_after(Box::new(move \|\| app.exit_main_loop()))` | 0.9.19, #198: `unsafe impl Send` and `Sync` removed from `App` (0.9.17 `app.rs:288-289`), and `call_after` still requires `Send` | Call `wxdragon::get_app_instance()` inside the closure, which exists at 0.9.17 (`app.rs:323`) and 0.9.23, so it can land before the move. `src/presentation/page_window.rs:423` already does this |
| The `accessible` module is Windows only | `names.rs:53` (the `use`), 162 and 187 (`set_accessible`); tests `a_rule_can_change_how_a_row_is_announced.rs:987`, `checkbox_labels.rs:95`, `manager_dialog_labels.rs:206, 379, 508, 545` (`get_accessible`). None of those is behind `cfg(target_os = "windows")` | 0.9.23 `lib.rs:6-7` and `prelude.rs:2-3` gate the module, and `window.rs:2078-2079` gates `set_accessible` | Gate the same lines; Linux and macOS only, so the Windows build never sees it |

How the counts were taken. The 11 `names.rs` sites and the third row are read at `a72d4b41`
(`git grep -n -E 'wxd_AccStatus|use wxdragon::ffi' main -- src/presentation/accessibility/names.rs`).
The 72 sites come from a trial build of a scratch copy of `main` at 0.9.23, Clippy over every
target with `--keep-going`, repeated after fixing each round until it came back clean (measured by
the upstream research; its four logs are `scratchpad/trial-0.9.23/clippy*.log`). The first round
reported 71 errors; the 72nd, a second site in `tests/item_form_free_busy.rs` at line 246, appeared
only after the first was fixed. The 69 files:

```text
$ git grep -l -E 'wxdragon::main\(' main -- tests | wc -l
69
$ git grep -l 'wxdragon::main' main -- tests | wc -l
72      # three more name it in comments only
```

The mechanical fix left 70 "unused variable `app`" warnings in the trial, which `-D warnings`
refuses, so the real fix renames or drops the closure's parameter.

The trial never linked an executable, never ran a test and never built for Linux or macOS. Those
are WXD-05's to find.

## What changes quietly

Each of these compiles and acts differently. Each says how a test can see it and which plan
carries it.

### 1. The automatic save stops (confirmed)

`wx_compose.rs:2796-2801` starts `_autosave_timer` with no handler of its own; the comment at
2792-2795 says that is because "a timer event here reaches every handler bound on the dialog".
The one handler is `later.on_tick` at 2391, which saves when `saved_at.elapsed() >= every`
(2423-2427) or when a tick arrives with nothing waiting (`None => keep_the_message()`, 2491).

At 0.9.17 `Timer::on_tick` binds the timer event on the owner for every timer
(`timer.rs:75`, `bind_internal`). At 0.9.23 it binds by the timer's own id (`timer.rs:63-65`,
`wxd_Timer_GetId` and `bind_with_id_internal`; 0.9.19, #193) (read, both). wxWidgets gives each
timer made without an id its own new id (stated, `timerimpl.cpp`). After the move the autosave
ticks reach nothing. A draft is then kept on its own only when `later` fires for something else
(spelling, Send Later, a look-up, leaving the body) and the interval has passed.

No test sees it today. The only test naming the setting reads source text
(`tests/wired.rs:824`):

```text
$ git grep -n -i -E 'autosave|draft_autosave|on_autosave' main -- tests
main:tests/house_style.rs:3830:// are named in its comments: an autosave checkbox that claimed sixty seconds,
main:tests/wired.rs:824:        app.contains("preview_first,\n        &signatures,\n        autosave,"),
```

How a test sees it: open the composer with a short interval, type, touch nothing, and check the
draft was kept. That needs a seam for an interval shorter than a minute
(`application/autosave.rs`, `MAX_MINUTES = 10`, the shortest setting is one minute). It also shows
a defect that exists today: an autosave tick that arrives while spelling or a look-up is waiting is
taken for that work (the comment at 2403-2409 says so), so a test that leaves spelling waiting and
fires the autosave timer is red at 0.9.17. **WXD-03.**

Three tests make their own ticker on the composer dialog
(`a_marker_counts_at_the_start_of_any_line.rs:1332`, `a_signature_follows_the_from_account.rs:1309`,
`signing_and_encrypting_from_the_composer.rs:450`; stated, code research). At 0.9.17 each of their
ticks also runs the composer's handler and, with nothing waiting, keeps the message. After WXD-03
or the move they will not. Ledgers 754, 761 and 785 are about two of those tests failing now and
then; read them again after WXD-03.

### 2. The main window's tick stops while a window it opened is up (new today)

The main window's poll timer ticks every 50 ms (`wx_app.rs:79`, `POLL_MS`; the timer at 6504). Its
handler drains every update from the background worker (`while let Ok(update) = ui_rx.try_recv()`
at 6585, then `handle_update`), raises what is due (`raise_what_is_due`, 6769) and asks about
folders that have gone (`ask_about_the_folders_that_have_gone`, 6793). Several of those open a modal
window from inside the tick:

- the Due now window, `wx_reminder_alert::raise` (14299), which calls `show_modal` (`wx_reminder_alert.rs:571`);
- the folders question, `MessageDialog ... show_modal()` (13768-13771);
- two update windows inside `handle_update`, at 24408 (`AnUpdateDidNotHappen`) and 24440 (`AnUpdateIsReady`);
- anything else `handle_update` reaches indirectly, which nobody has listed yet.

The code is written on the premise that the tick keeps running inside those windows. The comment
at `wx_app.rs:13746-13749` says "the tick keeps running inside a modal", 14272-14274 says "this
tick can happen again while somebody is still looking at the first one", and
`one_question_at_a_time.rs:102-104` and the test at `due.rs:1254-1258` rest on it.

At 0.9.17 that is true by accident: the event trampoline takes `&mut` to the closure and calls it
again while the outer call is still running (`event/mod.rs:920-926`, read). At 0.9.23 every handler
sits behind a re-entrancy guard; a second call to the same closure while the first is running is
skipped with `log::warn!("Re-entrant invocation of {context} skipped")`
(`utils/reentrant_callback.rs:70-78`; 0.9.23, #238; read). So while the Due now window or the
folders question is open, the tick does not run at all: no mail is announced, no sync or send
result is handled, no second reminder is raised, until the window closes (inferred from the two
readings; not run). Somebody away from the desk with a reminder open comes back to a program that
stopped where it was.

How a test sees it. A test that counts how deep the tick handler is nested while the Due now
window is open is red at 0.9.17 (it nests) and green once the window opens after the tick returns.
A second test, that an update sent while the window is open is handled before it closes, is green
at 0.9.17 and must stay green. The fix has to work at both versions: open each of these windows
after the tick has returned, and keep the one-at-a-time gate. `call_after` cannot carry the work
directly, because it requires a `Send` closure (`app.rs:37-39` at 0.9.17, `42-44` at 0.9.23) and
the tick's state is `Rc` and `RefCell`. Two shapes work at both versions (inferred): a one-shot
timer owned by a hidden child of the main frame, so its ticks reach only its own handler, which is
how `page_focus.rs:147` and the headings window's way back (`wx_app.rs:30356-30361`, written for
the same reason) already do it; or a thread-local queue drained by a `call_after` closure that
captures nothing, as `page_window.rs:423` does. A timer owned by the main frame itself would not
work at 0.9.17, where it would also receive every poll tick. **WXD-02.**

Every other timer handler in `src` was read for the same shape: `page_focus.rs:148`,
`wx_app.rs:30362`, `wx_managers.rs:5850`, `wx_reader.rs:414` and `wx_reminder_alert.rs:543`
open nothing; the composer's `later` handler opens the spelling check, but `later` is one-shot and
is not restarted during it. The code research's scan for handlers that re-enter themselves looked
only at handlers that write a control directly, which is why it found none.

### 3. What the toolkit says goes nowhere

0.9.23 reports two new things through the `log` crate: the skipped re-entrant call above, and any
panic it catches in a callback from C++ (`log::error!`, 0.9.23, #231, which now covers accessible
names, the virtual list's text and WebView callbacks; before, only event handlers were caught and
the rest ended the program). Our logger keeps only this crate's records: the filter is
`wixen_mail=<level>` (`common/logging.rs:75-76`, "this crate at the chosen level and no library",
168-171), and `init_logging` installs no bridge from `log` to `tracing` (read; not run). So both
messages are dropped. The panic hook in `main.rs:464-489` still writes `crash.log` and does not end
the program, so after the move a crash file can hold an entry from a session that went on running
with an empty name or an empty list row.

How a test sees it: a record logged by another crate at warn level reaches the log file. Red today.
**WXD-04.** Guardrail 9 asks for this: an upstream failure the program papers over is said, not
swallowed.

### 4. Dialogs grow on scaled displays

At 0.9.17 `wxd_Dialog_Create` passes the size it is given straight through
(`dialog.cpp:16-17`). At 0.9.23 it goes through `wxd_cpp_utils::to_wx`, which applies
`FromDIP` (`dialog.cpp:15-16`, `wxd_utils.h:53-61`; 0.9.19, #203, 14 constructors) (read, both). Our
manifest declares PerMonitorV2 (`wixen-mail.exe.manifest:29-30`). So every dialog given a size
becomes larger by the display's scale: 1.5 times at 150%. `src` has 63 lines calling `with_size`,
not all of them on dialogs (`git grep -n 'with_size' main -- src | wc -l`). This is most likely a
fix: until now a dialog sized for 100% was drawn cramped at 150% while its fonts scaled (inferred).

How a test sees it: only on a scaled display, and only in a program that declares itself DPI-aware.
The manifest is embedded by `build.rs:24-29` through winresource, which links resources into the
binary rather than into test programs (inferred), and CI's runners draw at 100%. So this is
measured on the release build at the machine's own scale in WXD-08 and heard in WXD-09.

### 5. A panel with nothing focusable no longer takes Tab

wxWidgets' change log for 3.3.4 lists, under incompatible changes: "Panels and other control
container windows without any children accepting focus from keyboard now don't accept focus
themselves." The note is absent from 3.3.2's change log (read, both `docs/changes.txt`). `src` builds
32 panels (`git grep -n 'Panel::builder' main -- src | wc -l`). A panel that held only text and
took Tab before will be skipped after. How a test sees it: Tab through each window and compare the
order before and after. **WXD-08.**

### 6. Smaller changes in how events are delivered

| Change | Release | Our exposure (read) | How it is seen |
|---|---|---|---|
| `on_destroy` fires only for the window's own destroy, not its children's | 0.9.19, #179 | one use, `text_history_keys.rs:151`, on a text box with no children | none needed |
| A child window's `destroy()` waits for idle | 0.9.18, #164 | `src` destroys only top-level windows (stated, upstream research); `tests` call `.destroy()` 160 times in 54 files | the full suite at 0.9.23 in WXD-05 |
| Handlers run from a snapshot: one bound during dispatch does not run in that dispatch | 0.9.22 | not searched | the full suite |
| Stock dialogs destroy their window when dropped | 0.9.19, #197 | FileDialog at 9 sites, TextEntryDialog 2, SingleChoiceDialog 2 (stated); their leaks end | none needed |
| WebView2 now follows show and hide, resizing through a deferred call | wxWidgets 3.3.4 | the preview pane, the conversation window, the composer | the page-focus tests and the scan |
| Esc in a dialog with an input method or autocomplete popup open no longer closes it | wxWidgets 3.3.4, #27145 | `src` uses no autocomplete (`git grep -n -E 'auto_complete' main -- src`, nothing) | WXD-08's keyboard pass |
| Accelerators take priority over the focused window in every port | wxWidgets 3.3 | Windows already worked this way (stated, change log) | WXD-08's keyboard pass over Ctrl+Z, Y, C, V, X and A |

### 7. wxWidgets gives more controls an accessible object of its own

In 3.3.4 every button, collapsible header and info bar gets one, beside the check boxes, radio
buttons and check list boxes that already had one; a button with an empty label takes its name from
its tooltip; 3.3.3 fixed check boxes being named "check" (#26270) (stated, upstream research and
change log). Our `set_accessible_name` replaces the control's object with `FixedName`, which answers
the name and description and returns "not implemented" for everything else, so Windows' own object
answers role and state (`names.rs:84-136`). Named controls should therefore read as they do today
(inferred). Unnamed ones may change, which is what the before-and-after scan in WXD-08 is for.

Tests that prove a name was set with `get_accessible().is_none()` (the six lines in section
"What breaks the build") can now pass on wxWidgets' own object, created when anything asks for the
window's accessible object, without our call having run (inferred). WXD-08 replaces them with a
reading of the name over MSAA.

### 8. The program grows

A small probe program (a frame, a WebView, a rich text box) went from 8,583,168 to 9,180,672 bytes,
7.0% larger, with the same 32 imported DLLs (measured by the build research, one run each, another
build running at the time). Wixen Mail's own release exe was 50,268,160 bytes on 2026-10-20 at an
unrecorded commit. WXD-05 measures both sides at one commit.

## What we gain

### Fixes people meet

- **List cells read whole.** `ListCtrl::get_item_text` lost the last character of every cell and
  put a NUL in its place (0.9.20, #205). Ledger 28, and ledger 159 for the blocked senders list.
- **Tree rows are freed.** Custom data on tree rows was never released (0.9.22, #214 and #215, both
  reported by Pratik). Ledger 41.
- **The WebView2 crash at exit.** wxWidgets 3.3.2 hands WebView2's creation reply to a control that
  has gone; 3.3.4 fixes it (#26491). Ledger 466: an exit within about 250 ms of the main window's
  preview or the conversation window being created ended with `0xc000041d`, and the row ends "Left
  as it is until a wxdragon release vendors 3.3.4, at which point browser_ready can be retired"
  (read, ledger). Ledger 661, an assert in the composer's button row, may be the same cause
  (stated, code research).
- **Leaks closed:** every accessible name object (0.9.19, #199), fonts passed to `set_font`, image
  lists, and stock dialogs (stated).
- **A panic in a name or list callback no longer ends the program** (0.9.23, #231), once WXD-04
  makes it visible.

### Security

- **RUSTSEC-2026-0194 and RUSTSEC-2026-0195 clear.** wxdragon-macros 0.9.22 and later require
  quick-xml 0.42 (stated). An unsuppressed `cargo audit` over both locks reported them on `main` and
  not on the trial (measured by the upstream and build research). They are accepted at
  `.cargo/audit.toml:54-55` under the comment at 41-53, and `scripts/audit.sh` refuses an accepted id
  that is no longer reported, so both leave in the move's commit.
- **RUSTSEC-2024-0436 (paste) stays.** wxdragon 0.9.23 still needs paste (stated), so the comment at
  `audit.toml:63` changes its version and keeps the entry.
- **Bundled C libraries.** libpng 1.6.55 to 1.6.59 and PCRE2 10.47 to 10.49 (read in both source
  trees); libtiff 4.7.1 to 4.7.2, expat 2.7.4 to 2.8.5 and wxWidgets' own image decoder fixes from
  3.2.11 and 3.2.12 (stated, upstream research). Exposure is low: the image crate decodes our
  images and WebView2 renders mail, but the code ships inside the exe.
- RustSec has no entries at all for any wxdragon crate (stated).

### Workarounds the move makes unnecessary

| Workaround | Where | Ledger | At 0.9.23 | Plan |
|---|---|---|---|---|
| The watch that holds Compose and Preview Before Send open until their browsers report | `src/presentation/browser_ready.rs`; `wx_compose.rs` names it 3 times; test `tests/closing_a_window_before_its_browser_exists.rs` | 466 | fixed upstream (read: wxdragon-sys 0.9.23 builds 3.3.4) | WXD-06 retires it, guarded by WXD-05's crash test |
| One timer handler for the whole composer | `wx_compose.rs:2381-2494`, `2790-2801` | none | each timer reaches its own handler | WXD-03 |
| The page focus check's timer owned by the page so it cannot run the frame's way back | `page_focus.rs:128-147` | 578, 579 | no longer needed for that reason | WXD-07.1 rewrites the comment; WXD-05 re-measures its guard record |
| A NUL-tolerant reading of list cells | `tests/manager_dialog_labels.rs:75-95` | 28, 159, 53 | fixed | WXD-07 removes it |
| Reading list cells through `LVM_GETITEMTEXTW` "not through `ListCtrl::get_item_text`" | `tests/a_signature_follows_the_from_account.rs:742-800`, `tests/the_key_manager_lists_and_names_its_controls.rs:144-200` | 28 | the premise is gone | WXD-07 keeps the readers, which read what a screen reader reads, and rewrites their reason |
| Tree rows kept out of wxDragon's data registry, held by a source check | `folder_tree.rs:1265-1311`; `wx_app.rs:439` `tree_rows` | 41 | fixed | keep the design; WXD-07.1 corrects the comment; ledger 41 closes on WXD-05's test |
| `ENTER_DOES_NOT_ANSWER_YES` built from raw bits | `asking.rs:32-33`, `from_bits_retain(wxdragon::ffi::WXD_NO_DEFAULT)` | none | named as `MessageDialogStyle::NoDefault` (`message_dialog.rs:26`) | WXD-07 |
| quick-xml advisories accepted | `.cargo/audit.toml:41-56` | none | cleared | WXD-05 |

### Still needed at 0.9.23

All read in the 0.9.23 source by the code research; keep each and change "0.9.17" in its wording.

- Printing through Win32 (`printing.rs`): upstream #246 is still open; `print.cpp` still returns
  without calling the base class.
- A link's address from the browser (`page_links.rs`, `page_window.rs`, test
  `a_link_opens_where_the_setting_says.rs`): 0.9.23 has no address on a navigation event. Upstream
  #249, unreleased, adds it.
- One WebView2 environment with no data folder setting (`page_window.rs:9-14`).
- No toolbar label setter (`toolbar_text.rs`), no font enumerator (`service/fonts.rs`), no bitmap
  loader (`Cargo.toml:226`), `add_spacer(0)` adds nothing (`names.rs:532-537`), no menu events on a
  dialog (`wx_compose.rs:1631-1634`, `wx_thread_view.rs:655`), no check state on a tree
  (`native_tree_checks.rs`), the tray icon builder panics (`wx_tray.rs`).
- Skip reset before every handler (`list_keys.rs:12-18` and others); its citation moves from
  `event.cpp:379` to `event.cpp:463`.
- `navigate`'s direction is inverted (`wx_compose.rs:2443-2457`,
  `tests/the_event_form_scrolls_to_the_field_in_focus.rs:387-393`). Correct at 0.9.23; upstream's
  unreleased #248 flips it, so the next move must flip ours in the same commit. WXD-07 adds a test
  that Tab out of the message body goes forward, so that move cannot pass silently.
- No name getter on `Accessible` (ledgers 14, 21); no way to raise a control's event from a test
  (349, 439); TextEntryDialog's box not handed back (621); WebView2's AllowExternalDrop (133).
- The state numbering (ledger 534, `names.rs:142-155`): 0.9.23 still writes MSAA's numbers where
  wxWidgets expects its own, and the new `set_accessibility_state` carries the same fault (stated,
  upstream research). Nothing here writes a state.

About 48 lines in 32 files outside `.planning` name 0.9.17 or 3.3.2
(`git grep -n -E '0\.9\.17|3\.3\.2\b' main -- . ':!.planning' | wc -l`), and 23 comments cite a
wxWidgets or wxdragon-sys line number (counted by the code research). WXD-07 corrects the four
lines and three citations in the three test files it edits, and WXD-07.1 the rest; `.planning`
keeps its history.

## Accessibility

**What can change.** On MSAA, NVDA's channel for native controls: the names, roles and states of
controls we do not name ourselves (section 7), list text (now whole), and anything a 3.3.4 control
answers differently. On UI Automation, Narrator's channel: Windows builds it from MSAA for most
of these controls, so it moves when MSAA moves, and the names we set still reach it only that way
(read: no UI Automation provider in either source tree, searched for `IRawElementProviderSimple`,
`UiaHostProviderFromHwnd`, `UiaReturnRawElementProvider` and `IAccPropServices`, with
`IAccessible` found in `msw/ole/access.cpp` as the control; upstream research). The exception
matters here: for an edit box or a button, Windows supplies its own UI Automation provider, which
shadows the MSAA object (CLAUDE.md, Accessibility), and section 7's buttons are where 3.3.4 adds
an object of its own, so a name or role can change on one channel and not the other. Axe's
violation count cannot show such a change, because a name that changes and breaks no rule moves
no count, so WXD-08 compares the UI Automation tree element by element as well as MSAA's. Tab
order may change (section 5) and dialogs may grow (section 4).

**Where a name has to be checked at the handle focus reaches.** These are the controls where focus
lands in a child window rather than on the window the name was set on:

| Control | Where | Instrument today |
|---|---|---|
| Spin controls, 13 built | `names.rs`, `name_the_spin_control` (270) and the Windows code after it (329 to about 510), names the typing field through `IAccPropServices` | `every_spin_control_names_the_field_a_person_types_in.rs`, `a_spin_controls_field_is_named_where_the_scan_reads_it.rs` (MSAA and UI Automation) |
| Editable combo boxes, 2 built | `wx_item_form.rs:1309`, `wx_managers.rs:1497` | none reads the child edit's name on either channel (stated, code research). WXD-08 adds one |
| Tab rows, 4 notebooks | Settings especially (#33, `TCM_SETCURSEL`) | `the_settings_tab_row_says_each_tab_once.rs`, NVDA case `settings-tabs-read-once` |
| List and tree rows | virtual list text, tree items | the scan; the list tests |

No date or time picker is built (both are named only in comments, `wx_item_form.rs:22`,
`wx_send_later.rs:13`).

**How every window is checked.** The accessibility workflow opens 49 windows with
`--scan-target`, from `main` to `quick-step-editor`, and reads each with Axe.Windows over UI
Automation and `scripts/msaa-names.ps1` over MSAA (read, `.github/workflows/accessibility.yml:206-233`).
Its artifact holds an `msaa-names.json` per window, a flat list of path, role, name and whether
the control is operated, and one or more `.a11ytest` files per window, each a zip whose
`el.snapshot` is the window's UI Automation tree with every element's Name, ControlType and
other properties (read 2026-10-20 over an artifact of 2026-10-19: 49 windows, 52 files, Axe
2.4.2). So:

1. A baseline run at 0.9.17: WXD-02's pull request, which changes when a window appears and is
   therefore pushed under the standing permission of 2026-09-23. WXD-02 reads its artifact once
   to show every window holds both trees.
2. The same run on WXD-05's pull request, read the same way.
3. A diff of every window's two trees by path, `msaa-names.json` for MSAA and `el.snapshot` for
   UI Automation, and of Axe's count per window. Every difference is either explained or a
   defect, and each is labelled by the channel it appeared on.
4. In-process tests for the controls in the table, on both channels.
5. Pratik's NVDA sitting over the windows (WXD-09), with a Narrator spot check on the controls the
   diff moved on UI Automation, and NVDA on those it moved on MSAA.

The scan can find fault against 3 of the 55 WCAG 2.2 A and AA criteria (`docs/wcag-coverage.md`),
so step 5 is what proves experience. The windows the scan cannot open (message boxes, the conflict
choice, Preview Before Send and others; `scan_target.rs` lists what it reaches) go on the sitting
sheet by name.

## Build and shipping

- **Machinery is unchanged.** MSVC, CMake, Ninja, bindgen with libclang, the `/MD` runtime, the
  static WebView2 loader, and a source download of wxWidgets pinned by SHA-256 (read in both
  `build.rs`; build research). No line of `build.rs`, `scripts/build-installer.sh`,
  `installer/Wixen-Mail-Setup.iss` or any workflow has to change for the move to build. One
  workflow comment changes all the same: `other-platforms.yml:127-128` names the features
  `Cargo.toml` turns on, and changes with them (question 4).
- **Every cache starts cold once.** Every target cache keys on `hashFiles('**/Cargo.lock')`: 11 uses
  in `ci.yml`, 3 in `accessibility.yml`, 3 in `nvda.yml`, 2 in `mutants.yml`, 1 in `guards.yml`
  (read). A runner's build from nothing took 750 to 1,070 s on 2026-09-15
  (`docs/development/measurements.md` row 560, stated). A pull request's caches are not visible to
  `main`, so the first push of `main` afterwards is a second cold round. The account's caches stood
  at 10,404,570,861 bytes on 2026-10-20, at the 10 GB limit (measured by the build research).
- **Locally, every switch across the move rebuilds wxWidgets.** wxdragon-sys keeps its build in
  `target/<profile>/wxWidgets` and replaces it when the version differs: about 154 s of build script
  on this machine (measured by the build research, contended). An executor that changes branch
  across the move pays it again, per profile.
- **One file is shared by every build on the machine.** Both versions download to
  `%TEMP%\wxWidgets.zip` (`build.rs:72` at 0.9.17). An old-version build and a new-version build
  started together race on it. Today that file holds 3.3.4 after the trial; a cold 0.9.17 build
  deletes it and downloads 3.3.2 again (read, `build.rs:1055-1061`), which needs the network.
- **The first cold build failed once** at CMake's compiler check with C1041 (a `vc140.pdb` clash) and
  passed on the re-run (stated, build research). Run it again before diagnosing.
- **The gate.** A commit that changes `Cargo.toml` or `Cargo.lock` beyond the package's own version
  earns every check (`which-checks.sh:433-445`). A commit that names failing tests is judged as
  `red` before that rule is reached (`which-checks.sh:250-266`), so WXD-05 reads what `red` runs for
  a lock change before relying on it. Phase 13.2's full gate took 741 s warm (stated); the first one
  after the move adds a cold wxWidgets build.
- **The installer is unchanged.** It ships one exe, the documents, the icon and the search handler;
  `search-handler` has no wxdragon dependency (read).
- **Found in passing, not caused by the move.** The exe imports `MSVCP140`, `VCRUNTIME140` and
  `VCRUNTIME140_1` (measured by the build research with `dumpbin`), and nothing in the installer,
  scripts or documents mentions the Visual C++ runtime
  (`git grep -n -i -E 'vc_?redist|vcruntime|msvcp|crt-static|target-feature' main -- installer scripts .cargo Cargo.toml build.rs docs README.md .github`,
  no output). On a Windows without that runtime the program would not start (inferred; not tried on a
  clean machine). See question 5.
  **Answered 2026-10-20: its ledger row lands with 13.4's plans, staged as `LEDGER-vc-runtime.md`
  with its evidence and its id to be taken again at landing, and the plan that fixes it comes
  after 13.4.**
- **Linux and macOS.** `other-platforms.yml` runs only when dispatched, so nothing would turn red on
  its own. Its apt line may also need two more X11 libraries for 0.9.23's link (stated, unverified).
  See question 3. It has never been dispatched at all: ledgers 323 and 324, open since 2026-09-12,
  say nobody knows whether the crate builds off Windows today, so WXD-05's run is the first.

## Guard records and the files written by rule

Counted with Python's TOML reader over `main:guards/guards.toml`, 1,640 records
(`scratchpad/phase-wxdragon-staged/guards_read.py` and `guards_live.py`):

| File | Records whose `file` it is | Records naming it in `tests_last_seen` |
|---|---|---|
| `src/presentation/accessibility/names.rs` | 2 | 2 |
| `src/presentation/wx_compose.rs` | 12 | 13 |
| `src/presentation/wx_app.rs` | 208 | 216 |
| `src/presentation/page_focus.rs` | 2 | 2 |
| `src/presentation/browser_ready.rs` | 2 | 2 |
| `src/presentation/asking.rs` | 1 | 1 |
| `src/presentation/folder_tree.rs` | 6 | 7 |
| `src/presentation/list_keys.rs` | 1 | 1 |
| `src/presentation/printing.rs` | 3 | 3 |
| `Cargo.toml`, `.cargo/audit.toml` | 0 | 0 |
| the 69 live-window test files | 15 | 181 records name at least one |

By anchor text, three records anchor on text the move edits: "a deletion that cannot be undone does
not let Enter answer Yes" (`asking.rs`), "the two dialogs holding a browser close through the watch
on it" (`wx_compose.rs`, on `destroy_when_ready`) and "the keyboard check's timer is the page's and
never runs the window's way back" (`page_focus.rs`). None anchors on `call_after`, `exit_main_loop`,
`on_tick`, `_autosave_timer`, `wxd_AccStatus`, "0.9.17" or the RUSTSEC ids. Each plan still takes
both readings over the regions it really edits.

**One record goes blind at the move, not at an edit.** The page focus record breaks
`Timer::new(page)` into `Timer::new(frame)`. At 0.9.17 that break makes the check's ticks run the
frame's way back. At 0.9.23 routing is by timer id, so the break probably reddens nothing (inferred).
Nothing per commit notices that, because no test count changes. WXD-05 re-measures it and any other
record whose break rests on how the toolkit routes events.

Files written by rule: `docs/changelog.md` (most plans), `guards/guards.toml` (most plans),
`.planning/WINDOWS.md` (ledgers 28, 41, 53, 159, 466, 534, 661, 594, 785, 754, 761 to read or
close), `docs/development/measurements.md` (exe size, imports, cold build times). Every plan writes
the first two, so the plans run one per wave.

**Overlap with phase 13.3**, read from the files each of its 25 plans lists: none touches
`names.rs`, `wx_compose.rs`, `page_focus.rs`, `browser_ready.rs`, `asking.rs`, `printing.rs`,
`Cargo.toml`, `Cargo.lock`, `.cargo/audit.toml` or any workflow. Eleven touch `wx_app.rs` (13.3-02,
03, 04, 05, 06, 08, 13, 14, 21, 23, 24), one `folder_tree.rs` (03), one `installer/` (20), one
`scripts/check.sh` and its suite (01), and four plans touch three live-window tests (02, 05, 13,
14). None adds a live-window test by an existing name; recount the 69 at plan time. WXD-02 edits
`wx_app.rs`, so this phase plans against the commit 13.3 closes at, never before
(`scratchpad/phase-wxdragon-staged/overlap.py`).

## Risks, in order

1. **The main window stops working while a question is on screen.** Section 2. Contained by WXD-02
   at 0.9.17, with a red test, before the move can cause it.
2. **Drafts stop being kept on their own.** Section 1. Contained by WXD-03 at 0.9.17, the same way.
3. **The move reddens tests nobody predicted.** The trial never ran a test, and 69 targets open real
   windows. Contained in WXD-05: build and run the whole suite at 0.9.23 on the branch before writing
   any commit, so the red commit can name every test it reddens and each gets its own fix.
4. **A name or a role moves on one channel.** Sections 5 and 7. Contained by WXD-08's before-and-after
   diff of all 49 windows on both channels, the new tests at the handle focus reaches, and WXD-09.
5. **Failures go silent.** Section 3. Contained by WXD-04 before the move.
6. **The crash test for ledger 466 may not fail reliably at 0.9.17.** The ledger describes a window of
   about 250 ms. WXD-05 runs it ten times at 0.9.17 before deciding: if it fails every time it is a red
   test; if not, it becomes a measurement of N runs on each side, recorded with its date.
7. **Cold builds cost time and can collide.** Two cold CI rounds; a local rebuild per branch switch;
   the shared zip. Contained by not building another branch in the main checkout while the move's
   branch is out, and by WXD-10 recording the costs.
8. **A guard record goes blind without a commit noticing.** Contained by WXD-05's re-measurement.
9. **The next wxDragon release flips `navigate`.** Contained by WXD-07's direction test.

## The plans

One coherent change each, carried by red and green pairs. Every plan writes the changelog and the
guard records, so they run one per wave, in this order. Each names its four completion marks, and
WXD-10 is the closing plan that runs `scripts/check.sh all`.

**WXD-01. Test windows stop sending the application object to another thread.** At 0.9.17. The 72
`call_after` sites in 69 test files call `wxdragon::get_app_instance()` inside the closure, and the
closure parameter is renamed or dropped. A refactor of test code: the suite is green before and
after, and the reason it is needed is the trial build at 0.9.23, quoted in the plan. No product
behaviour changes. Records: 181 name these files in `tests_last_seen`, but no test is added or
removed, so the count check stays quiet; none anchors on the edited lines. Its 73 files stay one
plan on purpose, with the edit committed in two halves by file name.

**WXD-02. The main window's tick stops running inside itself.** At 0.9.17. Red: a test that the
tick is never nested while the Due now window is open (red today). Kept green: an update sent while
that window is open is handled before it closes. Green: the Due now window, the folders question
and the two update windows open after the tick returns, through one of the two shapes in section 2
(not a capturing `call_after`, which needs `Send`), with the one-at-a-time gate kept; the plan
lists every modal reached from `handle_update` by reading its calls first. Comments at `wx_app.rs:13746-13749`, 14272-14274, `one_question_at_a_time.rs:102-104`
and `due.rs:1255-1258` rewritten. Pushed with a pull request; its accessibility run is the 0.9.17
baseline.

**WXD-03. The automatic save gets a timer of its own.** At 0.9.17. Red: a tick of the automatic
save does not take spelling or a look-up that is waiting (red today). Seam: an interval shorter
than a minute, for tests only, with its own red test. Kept green: with nothing else happening, the
message is kept after the interval. Green: the autosave timer is owned by a child of the composer,
the way `page_focus.rs` already owns its timer by the page, so its ticks reach only its own handler
at 0.9.17 and at 0.9.23, which holds at 0.9.17 only while that child owns no other timer: measured
for `src` and `tests` and for wxWidgets 3.3.4's WebView2 control, read again for 3.3.2 before the
green, and held by a reading from then on; the `None` arm goes, and the comments at `wx_compose.rs:2381-2390`,
2403-2409, 2490 and 2792-2795 are rewritten. Then read ledgers 754, 761 and 785 again.

**WXD-04. What the toolkit logs reaches the log.** At 0.9.17. Red: a warning logged through the
`log` crate by another crate reaches the log file (red today). Green: a bridge from `log` to
`tracing` and `wxdragon=warn` added to the filter in `common/logging.rs`. The plan checks that no
warning repeats every tick at 0.9.17, and WXD-05 checks it again at 0.9.23, because a warning every
50 ms would flood the file.

**WXD-05. The move.** Red, at 0.9.17, one commit: a list cell reads back whole (ledgers 28 and
159); a tree's row data is freed when its rows go, counted the way
`tests/tree_rows_leave_no_registry_entry.rs` counts (41); a window holding a browser, closed and
the program ended inside the creation moment, ends with exit 0 (466, if risk 6 allows); and the
lock holds wxdragon, wxdragon-sys and wxdragon-macros at one version, red first against a fixture.
Before the green, build and run the whole suite at 0.9.23 and name anything else it reddens. Green:
`Cargo.toml` `=0.9.23` with `features = ["webview"]` and the defaults kept (question 4, "yes"),
the comment in `other-platforms.yml` that names the features rewritten in the same commit,
`Cargo.lock`, `names.rs`, the two RUSTSEC ids out of `.cargo/audit.toml` with a note and the paste
comment's version corrected, the Windows gates of section "What breaks the build" (question 3,
"yes"). The checkpoint before it now confirms only the exact versions, wxdragon, wxdragon-sys and
wxdragon-macros at 0.9.23, and the packages that move with them. Measured: the release exe's size
and its imports at the commits on both sides, and the cold build time. Records re-measured: the
page focus timer record and the others in "Guard records". Pushed with a pull request, since what
NVDA reads can change; its Accessibility artifact is read once to show every window holds both
trees; `other-platforms.yml` dispatched on the branch and read per platform, a compile failure a
Windows gate fixes fixed there and any other put to Pratik. SHIP-05's evidence and the stack map,
which name the old features, get dated corrections. The Visual C++ runtime row is already in the
ledger by then (question 5); this plan adds the imports it reads to it as a dated line.

**WXD-06. The browser watch is retired.** Red: closing Compose before its browser has reported
closes it at once (red while `browser_ready` holds the close). Green: `browser_ready.rs` and its
uses in `wx_compose.rs` go; `tests/closing_a_window_before_its_browser_exists.rs` and WXD-05's
crash test stay green; the record "the two dialogs holding a browser close through the watch on
it" is retired or rewritten and re-measured. Ledger 466 closes, and 661 if it does not recur in
WXD-10's gate.

**WXD-07. The smaller workarounds.** The NUL allowance in `manager_dialog_labels.rs` goes; the
two `LVM_GETITEMTEXTW` readers stay with a true reason; `asking.rs` uses
`MessageDialogStyle::NoDefault`, with its record measured again; the lines naming the old toolkit
in the three test files it edits are corrected with them; ledger 53 is closed. One test added: Tab
out of the message body lands on the next control and Shift+Tab on the previous one, taken red by
swapping `navigate`'s argument.

**WXD-07.1. The comments that still name the old toolkit.** Added 2026-10-20, split out of WXD-07
when a plan checker counted 32 files there, two behaviour changes beside a comment pass. The
comments in `folder_tree.rs`, `wx_app.rs` and `page_focus.rs` say why the design stays; the
version wording outside `.planning` and the upstream line citations are corrected after reading
the 0.9.23 and 3.3.4 sources. Comments only, in two commits, `src` then `tests`.

**WXD-08. Every window's names on both channels, the keyboard and the sizes.** The diff of the 49
windows between WXD-02's run and WXD-05's run, on MSAA from `msaa-names.json` and on UI
Automation from the tree Axe saves in each `.a11ytest`, every difference explained or fixed and
labelled by its channel, so what moved on UI Automation alone reaches WXD-09's Narrator part. New tests: the
two editable combo boxes' typing fields named on MSAA and UI Automation at the handle focus
reaches; the six `get_accessible().is_none()` checks replaced by readings of the name, each taken
red by removing its naming call. A Tab-order pass over every window against the 0.9.17 order, a
pass over Ctrl+Z, Y, C, V, X, A and Esc, and each sized dialog's size on this machine's display.
Any fix is its own red and green pair.

**WXD-09. Pratik's NVDA sitting over the windows.** A build from the tip, installed by Pratik, and
a sheet: open each window, Tab through it, check boxes, spin fields, the two combo boxes, the
blocked senders list, the composer left alone past its interval with the shortest setting, a
reminder left open while mail arrives, the Settings tabs read once, Esc, and closing the program
with the preview pane showing. Plus a Narrator spot check on whatever WXD-08's diff moved, and the
windows the scan cannot open. This phase runs before phase 14's sittings (question 1, answered
2026-10-20), so this sheet rides on the first of them, Sitting 2, and that build serves both.

**WXD-10. The close.** `scripts/check.sh all` by hand on its branch after its last commit, output to
a file and the exit status read; a red one fixed on the branch as a small pair and run again. Ledger
rows closed or rewritten (28, 41, 53, 159, 466, and 661 if measured), the phase's records named by
`scripts/guards.sh --touched-by`, the cold CI and local build costs written to
`docs/development/measurements.md`, and the four completion marks for every plan.

### What Pratik does, and when

- **Before planning:** answer the questions below. Done 2026-10-20, "yes to all".
- **At WXD-05's first task:** confirm the exact versions and the packages that move with them.
- **During WXD-01 to WXD-08 otherwise:** nothing. Pull requests open under the standing permission
  of 2026-09-23; `main` is not pushed.
- **At WXD-09:** install the build and do the sitting with the sheet.
- **After WXD-10:** decide when to push `main`, as always.

## Questions for Pratik

Answered 2026-10-20: "yes to all. and let me see the issue before posting." Each answer is
written under its question and folded into the plans it touches.

1. **Before phase 14's sittings, or after?** Recommendation: after 13.3 and before the sittings.
   Otherwise the sittings test a toolkit that is about to be replaced, and anything heard over the
   windows has to be heard again; the move also fixes the exit crash a sitting can meet. The cost:
   the sittings wait for about ten plans and two cold CI rounds, and the build already made for them
   (1403) is made again. The 13.3 research recommended after; this recommendation differs because
   WXD-02 to WXD-04 take the move's two quiet changes out of the way before it happens.
   **Answered: before the sittings ("before the sit[tings]").** The move runs after 13.3 closes
   and before any of phase 14's sittings, so WXD-09's build is the one the sittings install and its
   sheet opens Sitting 2 (WXD-09 premise 1, WXD-10 premise 9). Two plans in the repository were
   written on the other premise and are not edited from here, since 13.3 is executing there: 13.3-25
   hands over to 14-08.2, saying 14-08.2 builds at 13.3-25's merge and Pratik installs that build
   before 14-10's sitting (`13.3-25-PLAN.md:31`, `:119`, task 2), and 14-08.2's own text says the
   same (`ROADMAP.md:109-110`, `:2264`). With this answer that build would be made and installed and
   then replaced by WXD-09's. Recommended: before 13.3-25 runs, or at the latest in the commit that
   lands these plans, a line in 13.3-25's hand-over and in 14-08.2's and 14-10's
   `<premise_corrections>`, and one at the top of `14-SITTING-2.md`, saying the wxDragon move comes
   first and its build is the one the sittings use. WXD-09's task 2 adds whichever of those is still
   missing when it runs. 13.3-23's checkpoint also carries this question as a side question
   (`13.3-25-PLAN.md:100-101`); it can quote this answer instead of asking.
2. **Fix the "one file" sentence in 13.3-23 before it runs?** Its checkpoint text, written to be read
   to you, says moving wxDragon "breaks the build in one file" (`13.3-23-PLAN.md:238-239`), and the
   13.3 research says the same at lines 218-220. Recommendation: yes, one sentence: "breaks the build
   in one source file and 69 test files, and stops drafts being kept on their own".
   **Answered: yes.** The sentence is in a 13.3 plan in the repository, which this staging does not
   edit; it is for whoever amends 13.3-23 before it runs.
3. **Keep Linux and macOS compiling?** This project's rule is that platform code sits behind a
   Windows gate with a fallback that keeps the crate building. Proving it needs one dispatched run of
   the Linux and macOS workflow on the move's pushed branch. Recommendation: yes to both.
   **Answered: yes to both.** WXD-05 gates every use of the Windows-only `accessible` module with a
   fallback, dispatches `other-platforms.yml` on its pushed branch and reads it per platform in
   ledger 323's terms. That run is the first ever (ledgers 323 and 324), so a compile failure there
   may predate the move: one a Windows gate fixes is fixed in WXD-05, any other is put to Pratik
   before the merge, and test failures are recorded on 323 (WXD-05 premise 10).
4. **Drop the two toolkit features nothing uses?** `Cargo.toml:79` turns on AUI and rich text; only
   comments mention either (`git grep -n -E 'RichText|Aui[A-Z]|aui::|richtext::' main -- src tests`
   finds three comment lines). Dropping them makes every cold build smaller. Recommendation: yes, in
   the move's commit; keep the default features, since the toolbar art may come from the Tango icons
   (not checked).
   **Answered: yes.** WXD-05's green commit sets `features = ["webview"]` and keeps the defaults.
   Neither feature pulls a package at 0.9.17 or 0.9.23, so the lock is the same either way. Every
   place that says which features are on was read on 2026-10-20 at `b49a38d4` and has its line in
   WXD-05 premise 9: `Cargo.toml:79` and the comment at `other-platforms.yml:127-128` change in the
   green commit; SHIP-05's evidence at `REQUIREMENTS.md:1988` and `.planning/codebase/STACK.md:28`
   get dated corrections; the rest are history or about the rich text control as a widget, and stay.
   The toolbar does draw with `ArtProvider` (`wx_app.rs:1247-1308`), so keeping the defaults keeps
   whatever icons it draws today.
5. **The Visual C++ runtime, found in passing.** The installer does not ship it and the exe needs it,
   so a tester without it installed may be unable to start the program (not tried). Recommendation:
   write it in the ledger now and make it a small plan after this phase that ships Microsoft's
   runtime with the installer. Not part of the move. WXD-05 opens the ledger row, since it reads
   the imports anyway; the plan that fixes it is yours to place.
   **Answered: yes.** The row is written now, in both of the ledger's shapes, in
   `LEDGER-vc-runtime.md` beside these plans, and lands with them; WXD-05 no longer opens it and
   adds its reading of the imports to it instead. The fix is a small plan after this phase.
6. **Tell upstream about the state numbering (ledger 534)?** Optional and yours to do; nothing here
   posts anything. 0.9.23's new `set_accessibility_state` carries the same fault to everybody who uses
   it. Recommendation: one issue, when convenient.
   **Answered: yes, and Pratik sees the issue's text before it is posted.** Nothing in these plans
   writes or posts it.

## What was not verified

- No test was run at 0.9.23, no executable was linked from Wixen Mail, and nothing was read by NVDA,
  Narrator or the scans at 0.9.23.
- Section 2's consequence (no update handled while the window is open) is reasoned from two readings
  and not run. The full list of modals reached from `handle_update` was not drawn.
- That the log bridge is absent was read, not run.
- Whether 3.3.4 changes what `get_handle` returns for a spin control, which the typing-field naming
  depends on.
- Linux and macOS were not built, here or ever: ledgers 323 and 324 say the workflow has never been
  dispatched.
- The Visual C++ runtime gap was not tried on a clean machine.
- The upstream release notes were read for the items named here, not end to end.

## Corrections to earlier documents

- `RESEARCH-the-update-mechanism.md:218-220` (phase 13.3): "breaks the build in one file" is true of
  the library and false of the build. Its own "What was not verified" list said a build would settle
  it; the trial did.
- The same document's automatic save claim (227-233) holds. Its `wx_compose.rs:2444` citation for
  `navigate` is the comment; the call is at 2457.
- `13.3-23-PLAN.md:238-239` repeats the first claim in text meant for Pratik (question 2).
- The code research counted "69 test files that call `wxdragon::main(`" and the build research "72
  files"; both are right about different things: 72 name it, 69 call it.

## Sources

- Crate sources: `scratchpad/third-party/wxdragon-0.9.17`, `-0.9.23`, `wxdragon-sys-*`,
  `wxdragon-macros-*`; crates.io answers in `third-party/crates-io-api-2026-10-20/`.
- wxWidgets change logs: 3.3.4's `docs/changes.txt` in the trial build tree
  (`%TEMP%/wxd23/debug/wxWidgets`), 3.3.2's in `target/debug/wxWidgets`; release notes in
  `scratchpad/wxd-research/wx-changes-3.3.3-3.3.4.txt`; wxDragon's in `wxd-research/CHANGELOG.md`
  (0.9.18's entries sit under the 0.9.19 heading).
- The trial build: `scratchpad/trial-0.9.23` (upstream research) and `scratchpad/wxd-build-check`
  (code research).
- The three research reports of 2026-10-20 this document checks and builds on.
- Scripts written for this document: `scratchpad/phase-wxdragon-staged/*.py`.

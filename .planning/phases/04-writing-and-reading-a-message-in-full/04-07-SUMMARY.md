---
phase: 04-writing-and-reading-a-message-in-full
plan: 07
subsystem: putting several files on a message, by picking, pasting or dropping
tags: [attachments, composer, clipboard, drag-and-drop, keyboard, accessibility, wcag-2.5.7]
status: partial
requires:
  - "application::attaching::Chosen::at, which already refused a folder and already ran a name through service::attachment_name::safe_file_name"
  - "application::attaching::over_the_limit and summary, which already said what a message comes to"
  - "presentation::wx_compose's attachment list and its Delete handler, which already took a file off"
  - "wxdragon 0.9.17's FileDialogStyle::Multiple, Clipboard::get_data, FileDataObject and FileDropTarget, all already present and none of them a new dependency"
provides:
  - "attaching::choose_all: many paths in, chosen files and one grouped refusal out, pure, no window"
  - "attaching::what_to_say and Announcement: everything said after a batch, as a list, so one announcement per batch is a property something can test"
  - "attaching::NAMED_ALOUD, the bound past which an announcement counts rather than names"
  - "a multi-file picker, so the button can do what a drop can"
  - "Ctrl+V in the composer, which attaches the files on the clipboard"
  - "a FileDropTarget on the composer dialog, wired to the same door and not established to receive anything"
  - "tests/every_way_a_file_goes_on_a_message.rs: a census asserting every route ends at the one model"
  - "editor_document's page turns a dragged-in file away, so the engine cannot navigate to it and take a half-written message off the screen"
  - "EditorMessage::FilesDroppedOnTheMessage: a count and no paths, so the composer can say where a drop does land"
  - "attaching::dropped_on_the_message: what happened, why, and the three routes that do work"
  - "editor_page_harness's stub records document listeners and what the page posts, so a handler can be run rather than only read"
affects:
  - "docs/KEYBOARD_SHORTCUTS.md, whose composition section names one more key and had two accelerators the wrong way round"
  - "any later plan that adds a fourth way to attach a file, which the census will fail until it goes through the same door"
  - "task 3 of this plan, which is a person dragging a file and is not done"
tech-stack:
  added: []
  patterns:
    - "one door for every route, with a source-reading census counting the routes, so a fourth cannot grow its own rules for a stranger's path"
    - "what is said built as a list and then spoken, rather than spoken as a loop goes, so 'once per batch' is testable"
    - "an announcement bounded by naming a few and counting the rest, the shape 04-06 used for suggestions"
    - "a typed refusal underneath a sentence-shaped one, so a batch can group reasons without either of them owning a second copy of the rules"
key-files:
  created:
    - tests/every_way_a_file_goes_on_a_message.rs
  modified:
    - src/application/attaching.rs
    - src/presentation/wx_compose.rs
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/changelog.md
    - guards/guards.toml
    - Cargo.toml
    - .planning/WINDOWS.md
decisions:
  - "Ctrl+V is bound once, on the dialog, and not on the attachments list. wxWidgets passes an unhandled key up the parent chain and the composer already used one dialog-level handler for exactly this, so one binding covers the list, the toolbar and the case where there are no attachments yet, while the message body and the address lines keep the key for their own paste"
  - "Six names, then a count. Six is about ten seconds of speech, which is as long as a routine confirmation should be; past that the total is the useful fact and the roll call is not"
  - "No count bound on a batch. The size limit already refuses what cannot be sent, a second weaker limit would have to be explained, and what does not scale is the speech rather than the metadata, so the bound went where the cost is"
  - "One path handed over keeps the sentence picking a file has always had, whole path and the operating system's reason; several are named. Keyed on how many paths arrived, not on how many were refused, which is a correction made during this plan"
  - "One heading over both message boxes, 'Attaching files', rather than the two this used to have. The heading is not where the information is and two headings for one route is how two things drift apart"
  - "The drop target is on the dialog and not on the attachments list, which is hidden until there is something in it and so can never catch the first file. The same reason the paste key is not bound there"
  - "The page refuses the drop rather than the host turning external drops off. AllowExternalDrop is not reachable through this stack, and the refusal in the page can also say what happened, which the host setting could not"
  - "A count crosses the channel and no path does. Attaching a file dropped on the message body would mean the page handling a stranger's path, which is T-04-30 and its own plan"
  - "The message area says where a drop does land at the moment somebody gets it wrong, rather than as a standing line. A standing line in the accessible description of the body would be read on every focus, which is the flooding guardrail 5 is about, and the list's description does not reach the accessibility tree in this binding at all"
metrics:
  duration: about six hours
  completed: 2026-09-06
  commits: 12
  version: 0.69.0 to 0.71.0
actuals:
  tokens: 58000
  tasks: 3
  commits: 12
---

# Phase 04 Plan 07: Several files on a message Summary

## Does it work

**All three tasks are done. Nobody has watched a drop happen, and that is the
one thing still open.**

Working, and exercised by tests that would notice if they stopped:

- The Attach File button opens a picker that takes several files at once.
- `Ctrl+V` in the composer attaches whatever files are on the clipboard.
- A batch keeps whatever reads. A folder among five photos is refused by name
  and the photos still go on.
- Whatever did not go on is said once, naming all of it, rather than once per
  file. The complaint about a message being too big is asked once about the
  message rather than once about each file.
- Every one of those routes ends at `attaching::choose_all`, and a census
  fails if a fourth grows rules of its own.
- A file dragged onto the message body is turned away by the page, so the
  browser engine cannot navigate to it and take a half-written message off the
  screen. This one is executed rather than read: the page's own handler is run
  in a JavaScript engine in `editor_page_harness` and the refusal is watched
  happening.
- Dropping a file on the message body says what happened, why, and where a drop
  does land, spoken and shown.

Rests on documentation rather than on somebody watching:

- **That a drop on the message body reaches the page at all.** WebView2's
  `AllowExternalDrop` defaults to true, which is what lets an external drag
  reach a page's DOM. That is Microsoft's documented default, not something
  measured here.
- **That a drop anywhere else on the composer reaches the dialog's drop
  target.** The OLE drag loop walks up the parent chain when the window under
  the pointer has no drop target of its own, and every other control on the
  composer is a plain wxWidgets window with none. That is how the mechanism is
  documented to work. Nobody has dragged a file onto a running composer.

Ledger entry 132 carries both, and the four observations somebody should make
are at the end of this document, unchanged.

Reached for and not available:

- **Turning WebView2's own handling of dragged-in files off.** That is the
  direct fix and it cannot be done through this stack, which was measured
  rather than assumed. Ledger entry 133, and the section below.

Not heard by anybody:

- None of this has been through a screen reader. The announcements are tested
  as strings and no test can say whether six file names is a useful
  confirmation or ten seconds nobody can sit through, or whether a message box
  in front of somebody who just dropped a file is help or an obstacle. Five
  entries in `.planning/WINDOWS.md`: 126 to 129, and 134.

## What criterion 1 now says

**Its second clause is closed.** "Every drop action has a keyboard equivalent
at least as quick to reach" is the paste key and the widened picker. Neither
depends on where a drop lands, and WCAG 2.5.7's rule against a drag-only
interaction is satisfied by task 1 alone.

**Its first clause is closed the second way the criterion allows, and not the
first.** The criterion reads "files land **or** the product says so where
somebody will see it, rather than the gap living in a report". Files landing is
still unwatched. The product saying so is built: drop a file on the message area
and it says that it was not attached, why, and where to drop it instead, spoken
and shown. That sentence is in `application::attaching` with its wording held by
tests, the same shape `application::allowed` uses for everything else this
program is honest about.

A build that compiles is still not evidence that a drop lands, and nothing here
claims it is.

## The grep the plan asked for, and what it found

Run at the start, over `src/` and `tests/`:

| looked for | found |
|---|---|
| `DropTarget` | nothing |
| `OnDropFiles` | nothing |
| `drop_target` | nothing |
| `FileDataObject` | nothing |
| `Clipboard::` | three lines, all in `src/presentation/wx_app.rs` and all about text: `set_text` at 4226, `get_text` at 14545 under `EditCommand::Paste`, `set_text` at 14623 |

So premise 1 held: no drop target anywhere, and no clipboard read of a file
list. The `Clipboard::` result is worth writing down rather than reporting as
"nothing", because the type was already in use and only for text.

## Which assertions were green the moment a loop existed

The plan named the trap and here is where it landed. `Chosen::at` already
refuses a folder and already refuses a file that will not read, so the naive
implementation, `paths.iter().filter_map(|p| Chosen::at(p).ok()).collect()`,
satisfies:

- `test_several_files_go_on_in_the_order_the_paths_arrived`, entirely.
- The **second half** of `test_a_folder_among_them_is_refused_by_name_and_the_others_still_go_on`
  and of `test_a_file_that_cannot_be_read_among_them_does_not_stop_the_others`.
  A `filter_map` is not all or nothing by construction.

What carried information: that the refusals are said at all, that two of them
are one announcement rather than two, that a single path keeps the sentence it
had before, the whole of `what_to_say`, and the bound.

The first of those is kept because it pins the ordering convention the rest is
built on, not because it drove anything.

## Every route ends at `Chosen::at`, and how that was checked

`choose_all` calls `Chosen::looked_at` once per path. `Chosen::at` is that
function with the refusal written out as the sentence it has always given, so
there is one read, one folder refusal, one unreadable refusal and one call to
`safe_file_name` in the whole program.

Checked three ways rather than asserted:

1. `tests/every_way_a_file_goes_on_a_message.rs` reads the shipping lines of
   `src/presentation/wx_compose.rs` and requires exactly three calls to
   `attach_from_paths`, exactly one call to `attaching::choose_all`, exactly
   one call to `Chosen::at` (the reopening of a draft, which is a different
   question), and no `Chosen { ... }` written out by hand.
2. The same reading is run over every other file in `src/presentation`, so a
   second window growing its own way to attach a file is seen too. It finds
   nothing.
3. A guard record, measured by hand, replaces `choose_all` in the door with a
   hand-built `Batch`. One test in the whole tree goes red.

**No second construction of a `Chosen` exists.** `grep -rn "Chosen {" src/`
finds the definition and the test fixture in `attaching.rs`, and nothing in
`src/presentation` at all.

## What a batch says, quoted

Not derived by reading the format strings. Pinned by
`test_what_a_batch_of_three_says_out_loud`, which asserts the whole sentence
rather than the parts it contains, because a comma where "and" belongs is heard
even though `contains` cannot see it.

Three files of 2 KB each, nothing attached before:

> Attached file-0.txt, file-1.txt and file-2.txt, 6 KB in total. Press Delete in the attachments list to take one off

The same three with a folder in the middle. Two announcements, in this order:

> Attached file-0.txt and file-1.txt, 4 KB in total. Press Delete in the attachments list to take one off

> holiday photos is a folder, and a folder cannot be attached

The second is said at High priority and shown in a message box headed
"Attaching files". The first is said at Normal and shown nowhere, which is what
picking one file has always done.

Ten files:

> Attached file-0.txt, file-1.txt, file-2.txt, file-3.txt, file-4.txt, file-5.txt and 4 others, ... in total

## The paste key, and what it was checked against

`Ctrl+V`. Checked against, and colliding with none of:

- **The page's own handler** in `src/presentation/editor_document.rs`:
  `Ctrl+Enter`, `Ctrl+S`, `F7`, `Alt+F7`, `F8`, `Tab`, `Alt` and a letter, and
  the thirteen keystrokes in `Format::keystroke`, which are `Ctrl+B`, `Ctrl+I`,
  `Ctrl+U`, `Ctrl+Alt+1` to `Ctrl+Alt+3`, `Ctrl+Alt+0`, `Ctrl+Shift+L`,
  `Ctrl+Shift+O`, `Ctrl+Shift+Q`, `Ctrl+Space`, `Ctrl+Z` and `Ctrl+Y`.
- **Every backticked combination the shortcuts document names in the
  Composition Window section**, which is the list above plus `Ctrl+\`,
  `Alt+Shift+F7`, and the button and field accelerators `Alt+N`, `Alt+U`,
  `Alt+R`, `Alt+A`, `Alt+F`, `Alt+T`, `Alt+C`, `Alt+B`, `Alt+S`, `Alt+E`,
  `Alt+D`, `Alt+I` and `Alt+L`.
- **The attach button**, which is `Alt+A` and is a mnemonic rather than an
  accelerator.

Where it fires and where it does not is a consequence of the mechanism rather
than a list somebody has to keep: wxWidgets sends a key to the focused window
and passes it up the parent chain until something handles it. The message body
handles `Ctrl+V` itself, in the page, and so do the From, To, Cc, Bcc and
Subject boxes, natively. What reaches the dialog is `Ctrl+V` on the attachments
list and on any of the nine toolbar buttons, and there it can only mean one
thing.

**An empty clipboard says so**, and so does a clipboard that will not open,
because those have different answers:

> There are no files on the clipboard to attach. Copy them in File Explorer first

> The clipboard is in use by another program. Try again in a moment

## The threat register

- **T-04-27, a dropped path naming something it should not.** The register asks
  for a fixture with a traversing name and one with a reserved Windows device
  name. Neither can be a real file, and the reason is the mitigation: a path
  that walks out of its folder never reaches the cleaner as a path, because
  `Chosen::at` asks for `file_name()`, which is the last component already; and
  Windows refuses to create a file whose stem is `CON`, `NUL` or the rest, so
  no drop can hand one over. Both rules are tested where they live, in
  `service::attachment_name`. What is testable through a batch, and is tested,
  is the bidirectional override, which is a real creatable file and the case
  the module doc says matters most to somebody listening:
  `annexe\u{202E}cod.exe` arrives as `annexecod.exe`.
- **T-04-28, attaching a file somebody did not mean to send.** Every file is
  named. Past six, six are named and the rest counted, which is a weakening of
  this mitigation and is the trade guardrail 5 asks for: a batch of forty named
  in full is an announcement nobody hears the end of. The summary line under
  the message still says how many and what they come to.
- **T-04-29, a drop of several hundred files. No count bound, and here is
  why.** The size limit already refuses what will not send, and a second,
  weaker limit expressed in files rather than bytes would have to be explained
  to somebody who met it. What genuinely does not scale is the speech, not the
  metadata, so the bound went on the announcement where the cost is. The cost
  of that decision, said plainly: `std::fs::metadata` on a few hundred paths is
  milliseconds on a local disk and can be seconds on a slow network share, and
  the window cannot speak while that runs. Nobody has measured it. Ledger entry
  is not filed for this because it is a stated trade rather than a defect;
  it is here.
- **T-04-30, the page handling a file path.** Not built. Named in task 3's
  instructions below as a different security boundary.
- **T-04-SC, package installs.** No package was added. `Cargo.toml` changed by
  one line, the version, confirmed by reading the diff.

## The plan's premises

**Every plan in phases 3 and 4 has carried at least one wrong premise. This one
carried four, and none of them changed the size of the job.** All four are
written into `04-07-PLAN.md` under `<premises_corrected_during_execution>`,
with the evidence.

**Premise 8's guard table is right in every row**, which is now two plans in a
row. Measured by parsing `tests_last_seen` rather than by grepping for a file
name: 617 records, `src/application/attaching.rs` named by 0 and holding 17
tests, `src/presentation/wx_compose.rs` by 2 and 43, `tests/wired.rs` by 8 and
61, `src/service/attachment_name.rs` by 0 and 12.

**Premise 2 is right about the framework in every part**, verified by reading
the vendored crate rather than by trusting the note. Two things it does not
mention and the code needs: `Clipboard::locker()`, which opens the clipboard
for the read and closes it when it goes out of scope, and
`Clipboard::is_format_supported`, which is what tells "no files on the
clipboard" apart from "the clipboard would not open".

The four that were wrong:

1. **Premise 5 says the paste key needs a second home. It needs none.** The
   premise reasons correctly that the attachments list is hidden when empty and
   so cannot catch the first file, and then concludes that a second binding is
   needed. wxWidgets passes an unhandled key up the parent chain, and the
   composer had been using one dialog-level handler for exactly this since
   `Ctrl+Enter` was bound there, with the comment "dialog-level fallback"
   directly above it. One binding covers both cases. A handler on the list
   would have been dead code from the day it was written.

2. **The `tests/wired.rs` change the plan lists is not needed, and finding out
   why is worth more than the change.** `Ctrl+V` was already in the document,
   in the Edit Menu table, and is bound as a menu accelerator in the main
   window as `"&Paste\tCtrl+V"`. So `bound_somewhere` finds it and
   `documented_combinations` deduplicates. The consequence is that the check
   gives the composer's `Ctrl+V` no protection at all: it is satisfied by an
   unrelated binding in a different window.

3. **The threat register asks for two fixtures and only one can exist.**
   Covered above under T-04-27.

4. **Task 2's version bump was not made.** The plan says to bump if the build's
   behaviour changed and to write no changelog entry until task 3. This project
   pairs a bump with an entry in the same commit, and there is nothing true to
   write until somebody drops a file, so both wait. `0.70.0` has not shipped,
   so task 3 adds to the same `[Unreleased]` section.

**The two tasks are separable**, unlike 04-06's. Task 1 leaves the census red
for a real absence, two routes where three are counted, and the census names
the two lines it found. Nothing in task 1's red commit asserted anything about
a drop.

**Both `<verify>` commands run as written**, unlike 04-05's and 04-06's. This
plan asks for a single `--lib` filter and a target selection, neither of which
is the invalid `--lib a --lib b` form those two reported.

## What the tests found that reading would not have

**Every refusal this program says out loud opens with the word "Error".**
`common::Error::Other` displays as `"Error: {message}"`, so what the composer
announced and showed for a folder was:

> Error: C:\Users\...\Temp\.tmpAZTjOv\photos is a folder, and a folder cannot be attached

Found because the test for "picking one file behaves exactly as it did before"
takes its expected value by calling `Chosen::at` rather than by transcribing
what that sentence was believed to be. A hand-typed expectation would have been
written from the message inside the error, would have matched the new code, and
the defect would have stayed invisible. Matched rather than fixed, because
fixing it is a change to `common::Error` and to every announcement in the
program that goes through it. Ledger entry 130.

**A batch of three with one folder in it said that whole path out loud**, in
the middle of a sentence naming the two files that did go on. The rule keyed on
how many things were refused rather than on how many paths arrived, and those
are different questions: one path is somebody picking a file, three is a batch.
Found while writing this document, which is what the acceptance criterion
asking for a quoted sentence is for. Fixed in `d2d7d6c` with its own red and
green.

## The guard records

Two, both measured by hand from a tree that was green first, every target with
`--no-fail-fast`. No two `guards.sh` runs overlapped.

**"one path that cannot go on does not take the rest of the batch with it"**,
on `src/application/attaching.rs`. The break makes the batch all or nothing:
the refusals are still gathered and still said, so the announcement is
unchanged and only the files are gone, which is what a
`collect::<Result<Vec<_>, _>>()` gives for free and reads as the careful
answer. Full red list, second measurement:

- `application::attaching::tests::test_a_file_that_cannot_be_read_among_them_does_not_stop_the_others`
- `application::attaching::tests::test_a_folder_among_them_is_refused_by_name_and_the_others_still_go_on`
- `application::attaching::tests::test_what_a_batch_of_three_says_out_loud`

Three red in 6,378. **Measured twice on the same day, and the second time is
the point of the whole `tests_last_seen` machinery.** The first measurement was
two tests. An hour later a test pinning the whole sentence was added for an
unrelated reason, and it reddens under this break as well. The commit that
added it was refused by the count check, `--remeasure` refused because the red
list was short, and a plain run named the third test. The record was two short
of the truth for as long as it took to read the message, rather than for three
commits.

**"every route that attaches a file goes through the one model"**, on
`src/presentation/wx_compose.rs`, with `suite = "every_way_a_file_goes_on_a_message"`.
The break replaces `choose_all` inside the door with a hand-built `Batch`: the
door keeps its name and its three callers, and underneath it every path becomes
an attachment with no folder refused, no name cleaned and every size reported
as zero. Full red list:

- `test_the_one_door_is_the_only_place_a_path_becomes_an_attachment`

One red in 6,376. **That one test is the finding rather than an aside.**
Nothing in the library can reach this code: it needs a window, a dialog and a
web view. A single source-reading test is the whole defence of the rule, which
is the shape 03-02 found for the sign-in census and 03-08 for the outbox.

The break had to be at the door rather than at a route, and that is worth
saying. A route building its own `Chosen` needs the attachment list in scope,
and the drop callback does not capture it, so that break does not compile and
`guards.sh` needs a tree that builds. The door going by hand is the worse
defect anyway.

**The suite coupling, proved both ways.** Against the registry as it stood at
the previous commit:

```
$ bash scripts/check.sh --suites-for <guards.toml at HEAD~1> src/presentation/wx_compose.rs
(nothing)
```

Against this one:

```
$ bash scripts/check.sh --suites-for guards/guards.toml src/presentation/wx_compose.rs
every_way_a_file_goes_on_a_message
```

And the commit gate itself printed it, which is the coupling working rather
than being described: `-- every_way_a_file_goes_on_a_message (coupled to what
changed by guards/guards.toml)`.

## Test counts

`src/presentation/wx_compose.rs` holds **43** `#[test]` functions, the same
number it held before this plan. No test was added to it.
`src/application/attaching.rs` went from 17 to 30.
`tests/every_way_a_file_goes_on_a_message.rs` is new and holds 7.

## One defect fixed that the plan did not ask for

`docs/KEYBOARD_SHORTCUTS.md` had Attach File and Discard the wrong way round.
Attach File is `Alt+A`, from `Reached::Attach`'s label `"&Attach File..."`, and
Discard is `Alt+I`, from `"D&iscard"`. The document said the opposite for both.
Fixed in task 1's commit, because that commit edits the table two rows above
it, and leaving a known-wrong row beside a new correct one is the thing
guardrail 4 is about.

The check that reads the document cannot see this class of error.
`bound_somewhere` asks only whether a letter is a mnemonic **somewhere in
`src/presentation`**, so two letters swapped between two buttons both resolve
and nothing fails.

## Commits

| commit | what |
|---|---|
| `afaa3f2` | RED: eleven failing tests for several files going on at once |
| `3ce55b2` | GREEN: `choose_all`, `what_to_say`, the wider picker, `Ctrl+V`, the shortcuts document, the changelog, 0.70.0, one guard record |
| `3cfd7b8` | refactor: give the one door a name every route calls it by |
| `c94a40b` | RED: a census that counts two ways in and expects three |
| `1d3137b` | GREEN: a drop target on the composer, wired and unverified, one guard record |
| `ea20fa6` | RED: a bad path named in the middle of a batch |
| `d2d7d6c` | GREEN: a batch names; one path still gets its path |
| `3c088f9` | pin the sentence a batch of three says, whole, and correct the record it made stale |
| `a10af9c` | merge of tasks 1 and 2 |
| `7f29c35` | RED: nine failing tests for a file dropped where the drop cannot land |
| `2633ca2` | GREEN: the page turns a dragged-in file away, the composer says where a drop lands, 0.71.0 |
| `2c2d168` | run the drop handler rather than reading it, and record the break that would notice |

---

# Task 3, and how it changed

The plan wrote task 3 as a blocking human verification: somebody drags a file
onto a running composer and reports what happened. Manual testing is happening
later, so the instruction became: make the best design now on documented
behaviour, and keep the human check as a check rather than dropping it.

**The research that changed the design.** Two facts, neither of them a guess.

**A drop over the message body does not reach the dialog.** WebView2 registers
its own `IDropTarget` on its own window, and Windows routes a drop to the
deepest registered drop target under the pointer. Microsoft's own guidance
confirms it from the other side: a host that *wants* those drops has to
register as `IDropTarget` itself and forward the calls into WebView2's
composition controller.

**`AllowExternalDrop` defaults to true, and that is the part that mattered.**
Before this change a file dropped on the message body was not ignored. WebView2
accepted it and handled it itself, which for a file means the engine navigates
to it. The composed message goes off the screen with nothing to undo. A dead
zone is a disappointment; a lost draft is a defect, and the plan had not
noticed that the second one was what shipped.

## Turning AllowExternalDrop off, and why it is not reachable

Measured against the tree on 2026-09-06 rather than assumed, because "the
binding does not expose it" is the kind of claim that turns out to be one grep
away from being wrong.

| asked | found |
|---|---|
| `AllowExternalDrop`, `allow_drop`, `accept_drop` anywhere in `wxdragon 0.9.17` | nothing |
| what `WebView::get_native_backend` returns | `wxd_WebView_GetNativeBackend` → `wxWebViewEdge::GetNativeBackend()` |
| what that returns in wxWidgets 3.3.2 | `m_impl->m_webView`, declared `wxCOMPtr<ICoreWebView2_2>` |
| where the controller is | `m_webViewController`, `wxCOMPtr<ICoreWebView2Controller>`, in `wx/msw/private/webview_edge.h` |
| `AllowExternalDrop` in wxWidgets 3.3.2 | nothing |

`put_AllowExternalDrop` lives on `ICoreWebView2Controller4`, which is reached by
asking the **controller** for it. wxWidgets hands out the view underneath the
controller, and there is no way back from one to the other in the WebView2 API.
The controller is in a private header. So the setting is out of reach two layers
up, not one, and no amount of unsafe COM code on this side gets to it.

**Named rather than absorbed, which is guardrail 9.** It is written in the
changelog where a user reads it, in the guard record's comment where the next
person to touch the page reads it, and in ledger entry 133. It has not been
reported upstream yet, which the changelog says in the same words the tree
control leak already uses.

## What was built instead

**The page turns the drop away itself.** `editor_document`'s script registers
`dragover` and `drop` on `document` in the capture phase and calls
`preventDefault` on a drag carrying files, which is the documented way to stop a
browser engine doing its own thing with a dropped file. So the message area is
now a dead zone rather than a trapdoor.

Four decisions inside that, each with a test holding it:

- **First in the script, before the editable element is looked up.** A lookup
  that answers null makes everything after it throw, and an unregistered refusal
  is the engine's own handling back in charge. `post` and `carriesFiles` are
  function declarations, so hoisting makes them available above where they are
  written; the comment says so, because it reads like a mistake otherwise.
- **In the capture phase**, so nothing further down the page can take the event
  first.
- **Only for a drag carrying files.** Dragging selected text from one place in a
  message to another is the editor's own behaviour and somebody writes with it.
  `dataTransfer.types` holds `Files` for one and `text/plain` for the other.
- **A count crosses the channel and no path does.** Attaching a file dropped on
  the body would mean a path chosen by whoever did the dragging travelling
  through the browser engine and a message channel before this program decides
  whether it may be attached. That is T-04-30, disposition **transfer**, and it
  is still not built. One test asks the running page for the keys of what it
  posted and requires them to be exactly `kind` and `count`.

**The composer says where a drop does land.** `attaching::dropped_on_the_message`
is the sentence, and it is `trouble`, so it goes through the same path every
other answer about attaching goes through: spoken at high priority and shown
under the heading "Attaching files". One file:

> That file was not attached. The message area is a small browser inside this
> window, and it takes the drop before the composer sees it. Drop files on the
> attachments line under the message, or press Ctrl+V, or use Attach File.

Four files opens "Those 4 files were not attached." A count of zero, which is a
bug rather than an attack, opens "Those files were not attached." and still says
what to do.

What happened, why, and what to do next, which is what every refusal in this
program owes somebody. `common::Error` is not in this path, so this is one of
the few sentences here that does not open with the word "Error"; ledger entry
130 is still open and was not touched.

## Where the statement lives, and two places it deliberately does not

The guardrail is that a warning which only exists in a report is a warning
nobody gets. Two obvious homes for a standing sentence were rejected for
reasons this codebase had already measured:

- **The message body's accessible description.** A screen reader reads a
  description on every focus, and somebody writing a message focuses the body
  constantly. Guardrail 5 asks for feedback that is bounded, and a three-sentence
  description on every focus is the flooding it names.
- **The attachments list's accessible description.** `attaching.rs` already
  carries a comment saying a description set that way does not reach the
  accessibility tree for a native list in this binding, which is why "press
  Delete to take one off" is spoken rather than described. A drop hint put there
  would be invisible for the same reason.

So it is said at the moment somebody gets it wrong, which is bounded by
definition, and it names the attachments line as the place to aim. **That line
is what makes the attachments area the explicit drop zone**: it is a static text
that is always visible, unlike the list, which is hidden until there is
something in it and so can never take the first file.

## The comment beside the drop target was wrong, and is corrected

Task 2 wrote that Windows "does not walk up to the parent when that window has
none of its own", so the target caught the dialog surface "and not any child".
That is the opposite of how the OLE drag loop works: it walks up the parent
chain from the window under the pointer. Every control on the composer except
the message body is a plain wxWidgets window with no drop target, so a drop on
the subject line, the toolbar, the attachments line or the list should reach the
dialog's target. The body is the one child that registers its own.

The corrected comment says that, says it is documented behaviour, and says
plainly that nobody has watched it.

## The tests, and the one that is worth more than the others

The drop rule is written down twice on purpose. `editor_document`'s own tests
read the page; `editor_page_harness` runs it, in a real JavaScript engine, using
the page that ships rather than a copy. The second is worth more: a reading is
answered by the words being present, a run is answered by the handler doing what
the words say.

The harness could not load the page at all until its stub grew
`document.addEventListener`, which is how the change was caught: 29 harness
tests went red on the green half and said so. The stub now records what it is
handed, so a handler can be run, and `postMessage` fills the `posted` array the
stub had declared and nothing had ever written to.

**The five harness tests were written after the code, so they had never been
red.** Taking out the one line they are about reddens exactly two tests, the
reading and the run, and nothing else in 6,400. Measured by hand with
`--all-targets --no-fail-fast` and written down as a guard record.

That break is the one worth having, and the record's comment says why: without
`preventDefault` the rest of the handler still runs. The count is still posted
and the composer still says where a drop lands. It would look completely right
and lose drafts.

## What the guard bookkeeping cost

- **One re-measure.** Four tests added to `src/application/attaching.rs` took it
  from 30 to 34 and made one record's fingerprint stale in the same commit. Its
  remedy needs the green code, so there is no ordering where the red commit has
  a clean tree around it; `CLAUDE.md` says to name the count check among the
  failures rather than split the commit, and that is what the red commit did.
  `--remeasure` then found all 3 named tests still red and nothing else.
- **One new record**, measured by hand as above.
- **Nothing on `src/presentation/wx_compose.rs`.** No test was added to it, so
  its count is still 43 and the three records naming it are untouched.
- **Nothing on `tests/every_way_a_file_goes_on_a_message.rs`.** No new route was
  added: the page's drop handler carries no path and attaches nothing, so the
  census still counts three ways a file goes on a message and its record is
  untouched.
- The sweep line at the top of `guards.toml` went from 427 records arrived since
  to 428.

Counted by parsing `tests_last_seen` rather than by grepping a file name, which
is the third plan in a row to do it that way.

---

# The human check, which is deferred and not skipped

**This is the part that needs a person, and it cannot be faked, guessed or
worked around.** Read this even if you have not read the plan; it assumes
nothing.

**What changed since it was written.** The design no longer waits on the answer:
the message area is a dead zone that says so, whichever way this goes. What the
check settles now is whether the parts that are supposed to work do, and what
any of it sounds like. Two of the four things below have already been answered
in the direction the design assumed, and the point of asking is that assuming is
not watching.

## What you are settling

The composer can now have files put on a message three ways: by pressing the
Attach File button, by pressing `Ctrl+V` with files copied in File Explorer,
and by dragging files onto the window. The first two are known to work. **The
third is not.**

The reason it is not is worth understanding, because it decides what the answer
means. The big box you type your message into is not an ordinary Windows
control. It is a small web browser, WebView2, embedded in the window, and it
handles dragging and dropping inside itself, before this program sees anything.
Windows sends a drop to the innermost window under the mouse pointer that has
asked for drops, and every other control in the composer has not asked, so a
drop on any of them travels outwards to the window itself, which has. The
message area is the exception, because the browser asks for its own.

So the message area is expected not to attach anything, on purpose. It has been
told to turn a dropped file away and say where to drop it instead, because
before that it did something worse: a browser handed a file opens it, so
dropping a picture on a half-written reply replaced the reply with the picture.

Nobody can settle any of this by reading code. It has to be watched.

## What to build and run

From the project folder, on Windows:

```
cargo build --release
```

Then run `target/release/wixen-mail.exe`. Say in your report which build you
used: the version is `0.71.0` and the executable prints its version and the
commit it came from with `--version`, so paste that line.

Open a composer: `Ctrl+N`, or File, then New, then Message.

Have a File Explorer window open beside it with at least three ordinary files
in it. Small ones. A text file, a picture and a PDF is ideal. Do not use
anything you would mind attaching to a message you then throw away.

## The four things to try, and report them separately

They can have different answers and the whole question is which of them worked.

**1. Drag one file onto the message body.** That is the large area where you
type the message itself, below the Subject line. Drop it there.

This one is expected to refuse, and what is being checked is that it refuses in
the way it was built to.

- What should happen: while you are still over the message area the pointer
  shows a "no entry" sign. When you let go, the message you were writing is
  still there, untouched, and a box appears saying "That file was not attached.
  The message area is a small browser inside this window, and it takes the drop
  before the composer sees it. Drop files on the attachments line under the
  message, or press Ctrl+V, or use Attach File."
- What would be a finding: nothing at all happens, no box and no pointer
  change. Or the message you were writing disappears and the file is shown in
  its place. Or the file's contents appear inside the message as text, or a
  picture appears in it.

The second and third of those are the defect this was built to stop, so if
either happens say so first. Report what actually happened, including "nothing"
if it was nothing, because "nothing happened" and "the file opened in the
message" point at different fixes.

Type a sentence into the message before you try this, so you can see whether it
survives.

**2. Drag one file onto the subject line, onto the row of buttons at the top of
the window, and onto the attachments line under the message.** Those are
ordinary Windows controls rather than a browser, and all three are expected to
attach. Try each separately and say which of them worked, because they are
different windows and the answer can differ.

The attachments line is the one that matters most: it is the place the refusal
in test 1 tells somebody to aim at, so if it does not take a drop then the
product is giving bad directions.

**3. Drag three files at once** onto whichever of those places worked. If none
worked, skip this one and say so.

- If it works: all three should attach, and you should hear **one** sentence
  naming all three, not three sentences. Something like "Attached notes.txt,
  photo.jpg and report.pdf, 2.4 MB in total."
- Say whether it was one sentence or three.

**4. Do it all again with a screen reader running.** NVDA or Narrator. **Write
down what it says, in the words it says them**, not a summary. If it says
nothing at all, that is the finding and it matters as much as the others.

The refusal in test 1 is spoken at high priority as well as shown in a box, so
there are two things to judge: whether it is heard at all, and whether three
sentences in front of somebody who has just let go of a file is help or an
obstacle. That is ledger entry 134 and it is a judgement only listening makes.

## What each answer means, and what to do about it

### If test 1 refuses the way it is meant to

That half is settled. Close ledger entry 132's first clause, and say in
`.planning/WINDOWS.md` what was watched, in your own words.

### If test 1 loses the message you were writing

**Stop and say so first.** That is the defect the whole of this was built to
stop, and it means the page's refusal is not being registered or not being
reached. Two things to check before anything else: that the build is `0.71.0` or
later, and whether the message area still refuses a drop after the composer has
been open a while, because the refusal depends on the page's script having
loaded.

### If tests 2 and 3 attach nothing

Then the drag loop is not walking up to the dialog's target the way the
documentation says, and the sentence in `attaching::dropped_on_the_message` is
telling somebody to aim at a place that does not work. Fix the sentence in the
same change as anything else, because bad directions are worse than none.

### The fallback design, named and not built

Making the message area itself attach a file means catching the drop in the
page's own JavaScript, in `src/presentation/editor_document.rs`, and posting the
file path back over the `wixenEditor` channel the page already uses for
`Ctrl+Enter` and the formatting keys. The page already catches the drop; what it
deliberately does not do is send the path.

**That is a different security boundary and it is why this is its own plan.**
Today the page is given HTML and gives back small facts: a key was pressed, a
word is at this position. It never handles a file path. Under the fallback it
would, which means a path chosen by whoever did the dragging travels through a
browser engine and a message channel before it reaches the code that decides
whether it may be attached. That deserves its own threat model, not a paragraph
in a summary. It is recorded here as T-04-30, disposition **transfer**.

## Whatever the answer

The paste key and the multi-file picker are unaffected either way, and they are
what makes this plan worth having.

## What is still open when this is read

- **Ledger 132**, that nobody has watched a drop happen anywhere on the
  composer. The four tests above are what closes it.
- **Ledger 133**, the upstream gap. `AllowExternalDrop` is unreachable through
  wxWidgets 3.3.2 and wxdragon 0.9.17, and it has not been reported upstream.
- **Ledger 134**, that the refusal has not been heard.
- **Ledgers 126 to 129**, from tasks 1 and 2, all about what nothing here can
  hear.
- **Ledger 130**, that every refusal this program speaks opens with the word
  "Error". Untouched on purpose; it is a change to `common::Error` and to every
  announcement that goes through it. The drop refusal does not go through it and
  so does not make it worse.

`status` in the frontmatter stays `partial` for that reason. The work is done
and one thing about it has never been watched, and calling that complete is the
mistake this whole document is written against.

## Self-Check: PASSED

- `tests/every_way_a_file_goes_on_a_message.rs` exists on disk.
- All twelve commits above exist in `git log`.
- `cargo test --lib -- application::attaching:: service::attachment_name::`
  passes, 46 tests.
- `cargo test --lib -- presentation::editor_document:: presentation::editor_page_harness::`
  passes, 116 tests.
- `cargo test --test every_way_a_file_goes_on_a_message` passes, 7 tests.
- `cargo test --test wired` passes, 61 tests, with both shortcut checks green.
- `cargo test --test house_style` passes, 64 tests, including the three that
  read `guards/guards.toml`.
- `scripts/check.sh all` passes on the branch head.
- `.planning/WINDOWS.md` carries 132, 133 and 134 in both its table and its JSON
  block, checked by parsing the JSON rather than by reading the table.
- `Cargo.toml` changed by one line, the version, twice.

---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 3
subsystem: presentation::printing, application::printing, presentation::wx_app
status: complete
tags: [print, gdi, win32, GAP-01, keyboard, ci]
requires: [13-02]
provides:
  - presentation::printing, the device half: Sheet (the page font, measured and drawn with), draw_page, print (StartDocW to EndDoc, AbortDoc on failure), ChosenPrinter, ask_for_a_printer over PrintDlgExW, print_on
  - application::printing's Printed, NotPrinted with one sentence each, pages_chosen, and the sentences for a conversation's row and for Print outside Mail
  - a_message_as_the_reader_shows_it, the one composition the text reader and paper share
  - File, Print on Ctrl+P and the letter P in the main window, on the message under the cursor
  - a spool target that sends a real job to Microsoft Print to PDF here and on CI's runner, and asserts the printer is absent where WIXEN_NO_PDF_PRINTER says so
affects: [13-04, 13-51]
tech-stack:
  added:
    - "windows 0.62.2 features Win32_Graphics_Gdi, Win32_Storage_Xps, Win32_UI_Controls_Dialogs (no new package; Cargo.lock unchanged)"
  patterns:
    - "one font object measures and draws, so the layout's breaks and the page agree by construction"
    - "an enhanced metafile as a printer-free oracle: every EMR_EXTTEXTOUTW string read back"
    - "a CI flag that asserts its own premise rather than skipping, as WIXEN_NO_AUDIO does"
    - "GDI objects, the dialog's global blocks and device contexts owned by guards that free them on drop"
key-files:
  created:
    - src/presentation/printing.rs
    - tests/printing_draws_what_the_layout_says.rs
    - tests/printing_spools_a_document.rs
    - tests/print_is_on_the_file_menu.rs
  modified:
    - Cargo.toml
    - src/presentation/mod.rs
    - src/application/printing.rs
    - src/presentation/wx_app.rs
    - tests/wired.rs
    - guards/guards.toml
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/USER_GUIDE.md
    - docs/privacy.md
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "The chosen printer is ChosenPrinter, not Chosen: tests/every_way_a_file_goes_on_a_message.rs reads \"Chosen {\" anywhere in the presentation layer as an attachment made outside the composer."
  - "NotPrinted and Printed live in application::printing with one sentence per reason, including PastTheLastPage for page ranges wholly past the end, so the window holds no printing wording."
  - "print_on lays out, chooses pages and sends the job in presentation::printing; the window's handler calls it once, so no printing decision lives in wx_app.rs."
  - "Print to File is hidden in the dialog (PD_HIDEPRINTTOFILE), because a file printer such as Microsoft Print to PDF already asks where."
  - "A cancel says \"Printing was cancelled, so nothing was printed.\" on the answer channel; a failure and every refusal go through send_refusal."
  - "GlobalLock and GlobalUnlock stay declared by hand; Win32_System_Memory is not worth turning on for two calls (below)."
  - "No workflow sets WIXEN_NO_PDF_PRINTER: GitHub's windows-2025-vs2026 runner opened and spooled to Microsoft Print to PDF on #104, against the research's premise, so the four workflows are back to main's text and CI runs the spool."
metrics:
  duration: about 2 hours 30 minutes
  completed: 2026-09-25
actuals:
  tokens: 18372
  tasks: 3
  commits: 11
---

# Phase 13 Plan 03: File, Print through Windows' own dialog Summary

File, Print and `Ctrl+P` in the main window print the message under the cursor in the
message list. Windows' own print dialog opens owned by the main window, the printer, copies
and pages are chosen there, and the message is composed the way the text reader composes it
with every date in full, laid out by 13-02 for the chosen printer's page, drawn with GDI in 11
point Segoe UI and spooled as one job named "Wixen Mail message". One sentence follows: what
was sent, to which printer and how many pages; that printing was cancelled; or what failed.
A conversation's row prints the one message the row stands for and says so; outside Mail,
Print says it works on messages in this build. The reader window, the conversation window
and the other modules are 13-04's.

`actuals.tokens` is chars/4 over the lines added under `src`, `tests`, `guards`, `docs`,
`.github` and `Cargo.toml` against `main` at `0221dbec`.

## What was built

| Commit | Kind | What | Hook |
|--------|------|------|------|
| `e33f76fc` | build | The three `windows` features, each with a comment, alone; `git diff --stat HEAD~1 -- Cargo.lock` empty | 761 s, `all` |
| `fd3d9020` | red | The metafile target against `Sheet`, `draw_page` and `print` answering failure; `Printed` and `NotPrinted`; 3 named | 184 s |
| `dbb20a41` | green | The page font guard, `Sheet`, `draw_page`, `print` with `AbortDoc` on failure; two records | 148 s |
| `5743cbbb` | red | `pages_chosen` and `NotPrinted::sentence` cases, the spool target, the dialog signatures refusing; 8 named, the count check among them | 128 s |
| `094db3d4` | green | `pages_chosen`, `ChosenPrinter`, `ask_for_a_printer`, `print_on`, `WIXEN_NO_PDF_PRINTER` in four workflows; two records new, one widened, four re-measured | 543 s, `all` (refused twice first, below) |
| `4908d11c` | refactor | `a_message_as_the_reader_shows_it` extracted; `wired.rs`'s three readings name it; the PGP composition record re-measured | 154 s |
| `4a8edd78` | red | `tests/print_is_on_the_file_menu.rs` and the two new sentences answering nothing; 5 named | 104 s |
| `73138324` | green | `ID_PRINT`, the File item, the arm, the handler, the sentences, the shortcuts page's row; two records | 210 s |
| `3c982864` | docs | The guide, the privacy page, the changelog, the ledger, the README's four answers, this summary, the four marks | 93 s |
| `a299032f` | fix | `WIXEN_NO_PDF_PRINTER` taken off the four workflows after CI's runner opened the printer (below) | 546 s, `all` |
| this commit | fix | The sentences that said the runners have no PDF printer: two test headers, one record's comment, ledger 613 and 616, this summary, STATE and GAP-01's evidence | |

**Test counts, taken again.** `cargo test --test printing_draws_what_the_layout_says` 3.
`cargo test --test printing_spools_a_document` 1, passing here in 0.8 s with the flag unset.
`cargo test --test print_is_on_the_file_menu` 4. `cargo test --lib application::printing::`
24 (18 at the start, 6 added; the sentence case widened rather than a seventh added).
`cargo test --test wired` 77 before and after. `src/presentation/wx_app.rs` 199 `#[test]`
functions before and after. `cargo test --test house_style` 74,
`the_words_that_say_nothing` 10, `every_status_sentence_has_one_shape` 10,
`the_planning_files_agree_with_themselves` 17.

**The flag's honesty, measured by hand here**, where Microsoft Print to PDF exists:

```
WIXEN_NO_PDF_PRINTER=1 cargo test --test printing_spools_a_document
test test_a_message_spooled_to_the_pdf_printer_comes_out_as_the_pages_the_layout_gave ... FAILED
WIXEN_NO_PDF_PRINTER is set and Microsoft Print to PDF opened, so the flag is hiding a working printer and this reading is not being made
```

With the flag unset the same target passed: three pages laid out, `Printed { pages: 3 }`, and
`pdfpurr` counted three pages in the file the driver wrote under a temporary folder. Nothing
was printed to any other printer, and no dialog was opened by any test.

**And on CI, where the same check found the premise wrong.** Run 36102778701's Test Suite on
#104, with the flag set in `ci.yml`, failed that one test with the same sentence: the runner,
image `windows-2025-vs2026`, Windows Server 2025 10.0.26100, provisioner 20260828.587, opened
Microsoft Print to PDF. The flag came off the four workflows in `a299032f`, and run
36105233215's Test Suite passed with the spool target reading "ok" in about two seconds, so
the runner spooled the three pages too.

**Acceptance readings.** `grep -c SAFETY` 15 and `grep -c 'unsafe {'` 15 in
`src/presentation/printing.rs` at the task 1 green, and 22 and 22 at the end, each pair read
in one command. `grep -c WIXEN_NO_PDF_PRINTER` was 1 in each of
`ci.yml`, `guards.yml`, `mutants.yml` and `release.yml` at `094db3d4`, as the plan asked, and is
0 in each since `a299032f`, because CI showed the criterion's premise false (deviation 7). The
letters on File, read with
premise 2's command: N, S, A, M, D, I, O, E, K, P, Q, each once. `grep -c 'fn
a_message_as_the_reader_shows_it' src/presentation/wx_app.rs` 1; `grep -c
a_message_as_the_reader_shows_it tests/wired.rs` 3. The File Menu section of the shortcuts
page names `Ctrl+P` once. `grep -c -i print docs/privacy.md` 5 before (every one
"fingerprint") and 11 after. Carriage returns 0 on every file touched, by `tr -cd '\r' < FILE
| wc -c`.

## Guard records

Six new, one widened, and every record the count check flagged re-measured, each with one
`scripts/guards.sh --remeasure` call per task:

| Record | Red | Run |
|--------|-----|-----|
| a printed page draws every line the layout placed on it, the last one too (new) | the every-line reading | task 1, 19 s |
| an ampersand in a message is printed as written, not taken as a mnemonic (new) | all three metafile readings | task 1, 20 s |
| a page two ranges both name is printed once (new) | the overlapping-ranges case | task 2, 100 s |
| a print job is ended, so the spooler hands it to the printer (new; written with the register's "a runner cannot judge this record" comment, rewritten once CI's runner spooled) | the spool reading, after its minute's wait | task 2, 79 s |
| a count and the thing it counts agree in number (widened) | its 28 and the past-the-last-page sentence case, 29 | task 2, found by the run and re-run |
| a printed line breaks at the last space that fits; every page's stamp counts every page; an event's place is one of the fields (re-measured, their file gained tests) | as recorded | task 2 |
| a PGP message the reader window never offers to the key (re-measured after its anchor moved into the extracted function) | its two | task 3 |
| a printed message carries its date in full, not how long ago it was (new, `wx_app.rs`) | the reading and the skip-the-dialog companion | task 3, 16 s |
| File, Print carries Ctrl+P in its label (new, `wx_app.rs`) | all four readings of `print_is_on_the_file_menu` | task 3, 15 s |

The plan expected the key record to redden `wired.rs`'s agreement test too. It runs only its
suite, and by reasoning neither of `wired.rs`'s two key readings would see the break: the
menu-to-document reading finds nothing missing when the key leaves the menu, and the
document-to-code reading searches the source as text, where `Ctrl+P` still appears in the
comment above the item. The arrived-since line at the head of `guards/guards.toml`
went from 297 to 303. No workflow record anchors on an `env:` block, read with the TOML reader
before the lines went in, so none was re-anchored.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The chosen printer is `ChosenPrinter`.** The first full-gate run of
task 2's green was refused by `tests/every_way_a_file_goes_on_a_message.rs`, which reads
`Chosen {` anywhere in the presentation layer as an attachment made outside the composer.
Renamed in the module and the spool target before the retry.

**2. [Process] Task 2's green was refused once more, at 18 s, by my own doing.** The next
task's red reading was already written in `tests/` when the full gate started, and the gate
would have run it and failed. Moving it aside while clippy was compiling made clippy fail to
read it. It went to the scratchpad before the retry and came back for its own red.

**3. [Order] The extraction landed before task 3's red, not after it.** Committed after the
red, the extraction's scoped run could have met the failing reading and been refused as an
unnamed failure, so it went first as a `refactor` with every test green, then the red, then
the green.

**4. [Shape] The functions are named a little differently from the plan.** The plan named
`Metrics::of(hdc, font)`, `measure(hdc, text)` and `draw_page(hdc, &Page, &Font)`. They are
one `Sheet` owning the font, with `measure`, `width` and `lines_per_page`, and
`draw_page(&Sheet, &Page)`, so the font that measures cannot differ from the one that draws.
`print` takes the output file the spool test needs. `print_on(&ChosenPrinter, &Printable,
Kind)` holds the layout, the page choice and the job, so the handler is one call, and the
Print reading checks for `print_on(` where the plan said `print(`.
`ChosenPrinter::the_printer_named` gives the spool target the dialog's answer without the
dialog, so the target runs the handler's path from the chosen printer on.

**5. [Rule 2] Two sentences and one reason the plan did not name.** Pages chosen wholly past
the end would have started and ended an empty job; `NotPrinted::PastTheLastPage` says
"Nothing was printed, because the pages you chose come after its last page. It has 3 pages."
Print outside Mail and on a conversation's row needed wording, now `prints_messages_only` and
`sent_one_message_of_a_conversation`, held by the sentence case.

**6. [Rule 2] Print to File is hidden** (`PD_HIDEPRINTTOFILE`); with it shown, a person
could tick it and the job would need a file name this code does not ask for.

**7. [Rule 1 - Premise] CI's runner has Microsoft Print to PDF, so no workflow sets the
flag.** Premise 5 and research 1.3 said the printer was taken off the Server 2025 image
(actions/runner-images#12328, a maintainer's answer of June 2025). The flag's own check
refused that on #104's first CI run, as quoted above, and with the flag taken off the runner
spooled the job. So `a299032f` put `ci.yml`, `guards.yml`, `mutants.yml` and `release.yml`
back to main's text, the plan's `WIXEN_NO_PDF_PRINTER` truth and its acceptance line no longer
hold as written, and the flag lives on in the test for a machine that really has no PDF
printer. Two consequences: this plan changes nothing in `release.yml`, and the spool guard
record is one a runner can judge. The ledger, the test headers and the record's comment were
corrected in a second fix commit after the documents commit, which is one commit more than
the brief's single documents commit.

### Found and left

The metafile and spool readings run on the hook through their records' `suite`, since each
record's `file` is `src/presentation/printing.rs`. The whole-suite gate of 13-51 runs them
too.

## GlobalLock and GlobalUnlock: declared by hand, and why not the fourth feature

`Win32_System_Memory` would give typed `GlobalLock` and `GlobalUnlock` in place of a four-line
`#[link(name = "kernel32")]` block. It would also compile every binding in that namespace
for two calls whose signatures have not changed since Windows 3.1, and it would be a fourth
manifest question for you. The block follows `date_display.rs`'s pattern, the name is read
with a bound of 260 units, and both calls sit under SAFETY comments. So it is not better
served by the feature, and it was not added.

## The release workflow

`release.yml` gained one line in its workflow-level `env`, `WIXEN_NO_PDF_PRINTER: "1"`, in
`094db3d4`, and lost it again in `a299032f` once CI showed the runner has the printer
(deviation 7). The branch's `release.yml` is main's, byte for byte, so this plan changes
nothing in the release workflow. `WIXEN_NO_AUDIO` is still missing there, as it was before
this plan; ledger 616 says so for you to decide.

## The wxDragon defect: details, and a draft

Nothing has been posted. You asked for details first, and you are running a test program that
prints through wxDragon to confirm it. The four sites, re-read today in the registry sources:

1. `wxdragon-sys-0.9.17/cpp/src/print.cpp:47-58`: the shim's `OnBeginDocument` calls the
   callback and returns `true` without calling `wxPrintout::OnBeginDocument`, and its
   `OnEndDocument` calls the callback and skips `wxPrintout::OnEndDocument`.
2. `wxdragon-0.9.17/src/printing.rs:45-58`: `PrintoutProxy::new` always passes both
   callbacks, so the base class is never reached.
3. wxWidgets 3.3.2, `src/common/prntbase.cpp:597-605`: the base class is where
   `GetDC()->StartDoc(m_printoutTitle)` and `GetDC()->EndDoc()` are called.
4. `src/msw/printwin.cpp:226` and `:264`: the Windows printer loop calls `OnBeginDocument`
   and then draws each page between `StartPage` and `EndPage`; nothing else starts the
   document.

What your test program should show if the defect is real: `Printer::print` returns `true`,
`on_print_page` is called for every page, and nothing reaches the printer. With Microsoft
Print to PDF chosen, no Save dialog appears and no file is written. A second, smaller finding
is on line 36 of `printing.rs`: `CString::new(title).unwrap()` panics on a title holding a
NUL character.

The draft, for you to post or not:

> **Printer::print on Windows draws pages but never starts a print job**
>
> wxdragon 0.9.17 (and `main` as of 2026-09-24), wxWidgets 3.3.2, Windows 11.
>
> `Printer::print` shows the dialog, calls `on_print_page` for each page and returns `true`,
> but nothing is spooled. The C++ `Printout` subclass overrides `OnBeginDocument` so that when
> a callback is set it calls it and returns `true` without calling
> `wxPrintout::OnBeginDocument` (`rust/wxdragon-sys/cpp/src/print.cpp`, lines 47-58), and
> `OnEndDocument` likewise skips `wxPrintout::OnEndDocument`. The base class is what calls
> `StartDoc` and `EndDoc` (wxWidgets `src/common/prntbase.cpp`, lines 597-605), and the Rust
> `PrintoutProxy::new` always passes both callbacks (`rust/wxdragon/src/printing.rs`, lines
> 45-58), so the document is never started. On Windows, `StartPage` without `StartDoc` spools
> nothing.
>
> To reproduce: run `examples/rust/printing_demo`, choose Microsoft Print to PDF, and print. It
> reports success, and no Save dialog opens and no file is written.
>
> A fix that keeps the Rust hooks: call the hook, then the base class, and return what the base
> class returns:
>
> ```cpp
> virtual bool OnBeginDocument(int startPage, int endPage) override {
>     if (m_onBeginDocument) m_onBeginDocument(m_userData, startPage, endPage);
>     return wxPrintout::OnBeginDocument(startPage, endPage);
> }
> virtual void OnEndDocument() override {
>     if (m_onEndDocument) m_onEndDocument(m_userData);
>     wxPrintout::OnEndDocument();
> }
> ```
>
> Separately, `PrintoutProxy::new` calls `CString::new(title).unwrap()`, which panics if the
> title contains a NUL character.

## Threat Flags

None beyond the register. T-13-07: the job is named by `job_name(Kind::Message)` inside
`print_on`, which takes a `Kind` and nothing else, and the Print reading requires
`Kind::Message` in the handler. T-13-08: the log carries "Printing stopped: <step>" on failure
and "Printed N pages on <printer>" on success; no subject, text or file path.
`tests/the_log_carries_what_a_report_needs.rs` passed. T-13-09: `DT_NOPREFIX` is held by its
record. T-13-10: the font, both device contexts and the dialog's two global blocks are owned
by guards that free them on drop. T-13-11: measured above. T-13-SC: the lock did not change.

## Known Stubs

None. `ChosenPrinter::the_printer_named` has one caller, the spool target, and is the dialog's
answer for a caller that already knows the printer; it is real code, not a placeholder.
13-04 adds the other surfaces, and nothing on the menu, in the guide or in the changelog says
they print.

## Ledger

Closed: 612 (`stub`, `application::printing` had no caller; File, Print reaches it, held by
`tests/print_is_on_the_file_menu.rs` and `tests/printing_spools_a_document.rs`). Opened: 613
(`unrun-verify`, the dialog under NVDA and Narrator and a page on paper), 614 (`todo`, the
wxDragon defect, filing it yours), 615 (`todo`, `pdfpurr` 0.4.0 not applying a Microsoft Print
to PDF file's ToUnicode map), 616 (`todo`, `release.yml` without `WIXEN_NO_AUDIO`). Both halves
of each, the table and the JSON.

## Version

`git tag` lists nothing, so no build has been cut since `1.0.0-alpha.1` and the version does
not move; the changelog entry says so.

## Self-Check: PASSED

- `src/presentation/printing.rs`, `tests/printing_draws_what_the_layout_says.rs`,
  `tests/printing_spools_a_document.rs`, `tests/print_is_on_the_file_menu.rs`: present.
- `e33f76fc`, `fd3d9020`, `dbb20a41`, `5743cbbb`, `094db3d4`, `4908d11c`, `4a8edd78`,
  `73138324`: in `git log` on `13-03-print`.
- No em dash or en dash on an added line; none of the six words.

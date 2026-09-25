# Phase 13, group 1: keyboard basics (GAP-01 print, GAP-02 undo and redo) - Research

**Researched:** 2026-09-24, against `main` at `630e2a67`, version `1.0.0-alpha.1`
**Domain:** printing from a wxWidgets and Win32 desktop program; undo and redo for native text controls and for actions on mail items
**Confidence:** HIGH on what the tree holds and on what wxdragon and wxWidgets do (read from their source); MEDIUM on the recommended print route (two probes run, the dialog path not run); LOW on anything a person hears or sees, which only the tester can settle.

Every claim below is marked **(stated)** when it was read from a file, an issue or a command this session, with the place, and **(derived)** when it is an inference from what was read. Claims from memory that were not checked are tagged `[ASSUMED]` and listed in the Assumptions Log.

<user_constraints>
## User Constraints

There is no `13-CONTEXT.md`; the phase directory held nothing when this was written (`ls -la .planning/phases/13-new-features-most-from-the-outlook-gap-audit/` listed no files). The constraints are the brief's and the project's:

### Locked decisions
- Pratik confirmed on 2026-09-24 the order inside the phase: keyboard basics first (print, then undo and redo), then reading mail, provider features, automation, then the rest of import and export. (stated: the brief)
- GAP-01's route for print is "decided by a measurement" (stated: `.planning/REQUIREMENTS.md:5773-5775`).
- GAP-02 splits into two levels: Edit, Undo and Redo for the focused control first; an undo of a delete, a move and a mark with the item named "as its own plan" (stated: `.planning/REQUIREMENTS.md:5785-5786`, and issue #47's body).
- Pratik confirms every package before it is installed; the plan that installs one waits for his word. (stated: the brief)
- Test-first on every change (`workflow.tdd_mode` true: `python -c ... config.json` printed `True` this session), additive schema, secrets out of the tree, network code tested on parsing and error mapping only, every setting reachable from the settings screen. (stated: `CLAUDE.md`)

### Discretion (the researcher's to recommend)
- The shape of each plan, its files and its failing test.
- The print route, subject to the measurement GAP-01 asks for.

### Out of scope
- Everything in groups 2 to 5 of the phase. Printing a web page opened from a link (the separate window, `src/presentation/page_window.rs`, a process of its own) is not a message and is out; said plainly in the plan and the guide rather than left silent.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research support |
|----|-------------|------------------|
| GAP-01 | File, Print (Ctrl+P) prints a message or the item under the cursor on every surface that shows one, through the native dialog, the header lines and the text. | wxdragon's printing route cannot start a document on Windows (section 1.2); the Win32 route through the `windows` crate already in the tree works and was probed (section 1.3); every surface and the document each already builds are cited (section 1.2); plans K1 to K3. |
| GAP-02 | Edit, Undo and Redo act on the focused text everywhere, and then on actions on items with the item named. | wxdragon exposes no undo on `TextCtrl`, and the Win32 messages reach it through the handle the project already uses (section 2.2); where delete, move and mark are carried out and what each would take to reverse (section 2.2); plans K4 to K6c. |
</phase_requirements>

## Summary

**Print.** Nothing in the tree prints today (`grep -rnE 'Printer|Printout|\.print\(\)|printing::|PrintDlg' src/ tests/` exited 1). wxdragon 0.9.17 wraps wxWidgets' printing framework, but reading its C++ shim shows a defect that decides the route: its printout class overrides `OnBeginDocument` and `OnEndDocument`, calls the Rust callback and returns without calling the base class, and the Rust side always supplies both callbacks. The base class is the only thing that calls `StartDoc` and `EndDoc` on the printer, so on Windows a wxdragon printout never starts a print job (derived from source, section 1.2). The same code is on wxDragon's `main` branch today. The route that works is the Windows API directly through the `windows` crate the project already builds: `PrintDlgExW` for Windows' own print dialog, then `StartDocW`, `StartPage`, `DrawTextW`, `EndPage` and `EndDoc`. Two probes in the scratchpad printed text this way to "Microsoft Print to PDF" and read a page back, and a third read every drawn line back exactly from an enhanced metafile, which gives a test that runs on CI where no printer exists. It needs three more features of `windows` 0.62.2 and adds no package to `Cargo.lock` (measured, section 4).

The text to print already exists for every surface: `reader_text::single_message` and `reader_text::conversation` build the reader's document (headers and body), and the six `ReadAloud::read_full` implementations hold every field of a contact, note, task, reminder and event. A pure layout module wraps and paginates that text against a measuring function; the transport draws it.

**Undo, level 1.** wxdragon's `TextCtrl` has no `undo`, `redo`, `can_undo` or `can_redo` (listed from `widgets/textctrl.rs`), but the project already sends Win32 messages to a control through `HWND(ctrl.get_handle())`, and `EM_UNDO` and `EM_CANUNDO` are in the `Win32_UI_Controls` feature it already has on. A plain Windows edit box keeps one step, and its redo is the same message as its undo (wxWidgets does exactly this, `textentry.cpp:734-755`). A disabled menu item's accelerator is swallowed silently by wxWidgets (`framecmn.cpp:364-365`), which matters for greying Undo when there is nothing to undo.

**Undo, level 2.** Delete and move are made on this computer first and kept as one waiting row per message (`moves_waiting`), then told to the server in the background; mark and star go through `ServerChange::Flag`. That shape makes an undo that arrives before the server has heard a cancellation of the waiting row, and an undo that arrives after a second move the other way. Delete Permanently cannot come back once the server took it. The other modules' deletes remove the row and keep only a deletion note, so undoing one needs the whole record kept by the undo itself.

**Primary recommendation:** print through `PrintDlgExW` and GDI with a pure layout module under it, tested by reading an enhanced metafile back; undo text through `EM_UNDO` on the focused control first, then a history of its own for several steps, then item undo for mail in two plans and the other modules in a third.

## Architectural Responsibility Map

This is a desktop program, so the tiers are its own layers.

| Capability | Primary tier | Secondary tier | Rationale |
|------------|-------------|----------------|-----------|
| What a printed page holds (header lines, body, fields of an item, page breaks, "Page 2 of 3") | `application` (pure) | `presentation::reader_text`, `read_aloud` (sources of the text) | Pure and testable without a window or a printer, the way `time_blocks` and `editing` are. |
| Asking Windows for a printer, drawing, spooling | `presentation` (thin Win32 transport) | none | A platform boundary; kept thin so the pure part carries the logic (CLAUDE.md's network rule applied to a device). |
| Which item Print acts on | `presentation::wx_app` dispatch | `application::status_sentences` for the refusal | The same place Delete and Save As decide "the message you are on". |
| What Undo means where the cursor is | `application::editing` (pure) | `presentation::wx_app` carries it out | Extends the rule module that already answers Cut, Copy, Paste and Select All. |
| A text box's own several-step history | `application` (pure history) | `presentation` (one helper bound on every editable box) | Pure history, thin binding, checked by a reading over the tree like the spin-control naming. |
| What an undo of an item action does | `application` (pure decision, like `sending_later::what_undo_send_takes_back`) | `data::message_cache` (waiting rows, flags), `presentation` (wiring) | Mirrors how Undo Send is built. |

## Standard Stack

### Core
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `windows` (existing) | 0.62.2, pinned by `Cargo.lock` | `PrintDlgExW`, `StartDocW`, `StartPage`, `EndPage`, `EndDoc`, `AbortDoc`, `DrawTextW`, `CreateFontIndirectW`, `GetDeviceCaps`, `GetTextExtentPoint32W`, `CreateEnhMetaFileW`, `EnumEnhMetaFile`; `SendMessageW` with `EM_UNDO` and `EM_CANUNDO` | Already a dependency with eleven features on (`Cargo.toml:399-439`). Three features added, no package added. |
| `wxdragon` (existing) | `=0.9.17` (`Cargo.toml:79`) | Menus, `on_menu_opened` and `on_menu_closed` (`event/menu_events.rs:108-122`), `get_handle()` for the HWND | Nothing new asked of it. Its `printing` module is not used, for the reason in section 1.2. |
| `pdfpurr` (existing) | 0.4.0 | Test-only oracle: page count of a printed PDF | Already a dependency (`Cargo.toml:129`). See the ToUnicode finding in section 1.3 for why it is a page counter here and not a text oracle. |

### Features to switch on (not packages)
| Feature | Gives | Needed by |
|---------|-------|-----------|
| `Win32_Graphics_Gdi` | `HDC`, `DrawTextW`, fonts, `GetDeviceCaps`, the metafile calls; also the cfg gate on `StartDocW` and `PrintDlgExW` | K2 |
| `Win32_Storage_Xps` | `StartDocW`, `StartPage`, `EndPage`, `EndDoc`, `AbortDoc`, `DOCINFOW` | K2 |
| `Win32_UI_Controls_Dialogs` | `PrintDlgExW`, `PRINTDLGEXW` | K2 |

(stated: `windows-0.62.2/src/Windows/Win32/Storage/Xps/mod.rs:5-79`, `UI/Controls/Dialogs/mod.rs:95-99, 1599-1601`, `Graphics/Gdi/mod.rs:125, 277, 309, 448, 523, 596, 954`; `StartDocW` and `PrintDlgExW` are each behind `#[cfg(feature = "Win32_Graphics_Gdi")]` as well as their own feature.)

### Alternatives considered
| Instead of | Could use | Tradeoff |
|------------|-----------|----------|
| Win32 GDI through `windows` | `wxdragon::printing` | Cannot start a print job on Windows at 0.9.17 or on upstream `main` (section 1.2). Would be the natural route after an upstream fix. |
| Win32 GDI | `WebView::print()` | `wxWebViewEdge::Print` runs `window.print();` (`webview_edge.cpp:1341-1344`): the browser's own print preview inside the page, not Windows' dialog; reaches only the three web surfaces, never the reader window or the other modules; the preview is kept unfocusable on purpose (`wx_app.rs:1538`). |
| Win32 GDI | WebView2's `ShowPrintUI(System)` through `get_native_backend()` (`webview.rs:1003`, returns the `ICoreWebView2`, `webview_edge.cpp:1576-1579`) | Needs `webview2-com` 0.39.1 (three new packages, measured) or a hand-written vtable, which the project declined for the spin-control call (`Cargo.toml:425-433`); still web surfaces only. |
| Our own several-step text history | Switching every text box to `TextCtrlStyle::Rich2`, whose RichEdit control has multi-level undo and `EM_REDO` natively (`textctrl.cpp:2012-2033`) | Changes the window class of about 35 controls, and with it what MSAA and UI Automation expose, in a phase that just spent two plans on one control's name. Rejected unless Pratik prefers it. |

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| windows | crates | since 2019-01-15 | 6,281,335 a week | github.com/microsoft/windows-rs | OK | Already in the tree; three features added, confirmation asked (question 1) |
| webview2-com | crates | since 2021-09-06 | 673,245 a week | github.com/wravery/webview2-rs | OK | Not recommended; audited as an alternative only |

(stated: `node .claude/gsd-core/bin/gsd-tools.cjs query package-legitimacy check --ecosystem crates windows webview2-com`, 2026-09-24.)

**Packages removed as SLOP:** none. **Flagged SUS:** none.

---

# 1. GAP-01: Print (issue #45)

## 1.1 What the issue and Pratik ask (stated)

`gh issue view 45 --json title,body,comments,state`: open, no comments. The tester, 2026-09-15, build `0.125.1+g3e633252`: "Add print functionality." The issue body proposes:

- File, Print (`Ctrl+P`) on a message in the list, the open message window, the preview and the conversation window, printing the header lines (From, To, Cc, Date, Subject) and the text "through the native Windows print dialog so the printer, copies and pages are the ordinary dialog's".
- "Print Preview on the same menu if the printout is drawn by the application".
- In the other modules, Print prints the open event, contact, task, note or reminder "the way it is shown".
- "Announce what was sent to print and to which printer, once."
- Route: "Plain text with header lines is the honest first version."
- The privacy page gets a sentence if anything is sent anywhere other than the printer; shortcuts page and changelog in the same commit; a minor bump.

The requirement adds: "[S] What a printed page looks like is a sighted reader's; whether the dialog is worked by keyboard is the tester's ear." (`.planning/REQUIREMENTS.md:5776-5777`)

## 1.2 What the tree and its dependencies already have

**Nothing prints.** `grep -rnE 'Printer|Printout|\.print\(\)|printing::|PrintDlg' src/ tests/` exited 1. `Ctrl+P` is bound nowhere: `grep -rnE 'Ctrl\+P[^a-zA-Z]|...' src/` found nothing but `Ctrl+Shift+P`, and `docs/KEYBOARD_SHORTCUTS.md` names only `Ctrl+Shift+P` (`:80`, `:677`) and NVDA's `NVDA+Ctrl+P` (`:1363`). (stated)

**wxdragon's printing module, read in full** (`~/.cargo/registry/.../wxdragon-0.9.17/src/printing.rs`, 420 lines, not behind a feature, `lib.rs:31`):

- `Printout` trait (`:10-25`): `on_prepare_printing`, `on_begin_printing`, `on_end_printing`, `on_begin_document`, `on_end_document`, `on_print_page(&GenericDC, page)`, `has_page`, `get_page_info`. (stated)
- `Printer::new(Option<&PrintDialogData>)` and `Printer::print(parent, title, printout, prompt) -> bool` (`:262-278`); `PrintDialog` with `get_print_dc()` (`:296-333`); `PageSetupDialog` (`:343-375`); `PrintData` with only `new` and `is_ok` (`:147-158`), so no printer name, orientation or copies can be set from Rust. (stated)
- No print preview and no HTML printing: `grep -rln 'HtmlEasyPrinting\|HtmlPrintout\|PrintPreview\|wxPreviewFrame'` over wxdragon's and wxdragon-sys's sources found nothing. (stated)
- `PrintoutProxy::new` calls `CString::new(title).unwrap()` (`:36`), so a title holding a NUL panics. (stated)
- The page-size and PPI helpers (`:379-419`) sit on the private proxy, not on the trait, so a `Printout` learns the page only through `GenericDC::get_size`, `get_ppi` and `get_text_extent` (`dc/mod.rs:593, 601, 948`). (stated)

**The defect that rules this route out** (derived from four sources read this session):

1. The Rust proxy always passes every callback, including `Some(Self::on_begin_document_cb)` and `Some(Self::on_end_document_cb)` (`printing.rs:45-58`).
2. The C++ shim's override calls the callback when one is set and returns `true` without calling the base class: `if (m_onBeginDocument) { m_onBeginDocument(...); return true; } return wxPrintout::OnBeginDocument(...)`, and the same for `OnEndDocument` (`wxdragon-sys-0.9.17/cpp/src/print.cpp:47-58`).
3. wxWidgets 3.3.2's base class is where the print job starts and ends: `return GetDC()->StartDoc(m_printoutTitle);` and `GetDC()->EndDoc();` (`target/debug/wxWidgets/src/common/prntbase.cpp:597-605`).
4. The Windows printer loop calls `printout->OnBeginDocument(...)` (`src/msw/printwin.cpp:226`) and then draws each page between `StartPage` and `EndPage` through a page guard (`:264`), with `StartDoc` nowhere else. The DC exposes no `start_doc` to Rust: `grep -rn 'StartDoc\|EndDoc'` over wxdragon and wxdragon-sys found only the callback typedefs.

So `Printer::print` goes through the dialog, draws pages into a device context that never had a document started, and returns `true`; Windows' `StartPage` without `StartDoc` spools nothing (derived; the Win32 rule that `StartDoc` must come first is `[ASSUMED]` from the API's documented contract and was not re-read today). It was not run: a probe with `prompt = false` would print to this machine's default printer, which is a real HP LaserJet 1022 (`Get-Printer` this session), and wxdragon offers no way to name another. Upstream `main` carries the same override (`curl raw.githubusercontent.com/AllenDang/wxDragon/main/rust/wxdragon-sys/cpp/src/print.cpp`, read 2026-09-24), its history is two commits (`7aeb649e` 2026-02-12 "Wrap wxPrinter, wxPrintout and printing dialogs", `a86dd37c` 2026-09-22), and no issue or pull request mentions it (`gh search issues --repo AllenDang/wxDragon print`, `gh pr list ... --search print`). Upstream's own `examples/rust/printing_demo` would log "Printing successful" through the same path. The latest wxdragon is 0.9.22 (`cargo search wxdragon`), not reviewed for this.

**The WebView's own print** is `webview.print()` (`widgets/webview.rs:758-764`), which calls `wxWebViewEdge::Print`, which is `RunScript("window.print();")` (`webview_edge.cpp:1341-1344`). (stated)

**The surfaces that show a message, and the document each already builds** (stated unless marked):

| Surface | Where | What it is | Its printable text |
|---------|-------|------------|--------------------|
| Message list, main window | `msg_list`; File menu `wx_app.rs:6661-6728` | Focus stays on the list; the preview beside it is kept unfocusable (`wx_app.rs:1538`) | `open_in_the_text_reader` (`wx_app.rs:14107-14127`) builds it: body from `cache.get_message_body`, `what_a_message_shows_and_says`, `attachments_of`, then `reader_text::single_message(&message, &shown.body, out).with_what_is_said(&shown.said)` |
| A conversation row | same | `open_conversation` (`wx_app.rs:14169`) | `reader_text::conversation(subject, parts)` (`reader_text.rs:1139`) |
| Reader window | `wx_reader.rs:259-330`, own menu bar: File (Read Attachment, Save Attachment, Close Tab, Close Window) and Go | Read-only `Rich2` text control per tab (`wx_reader.rs:463-470`) | The tab's `ReaderDocument` (`reader_text.rs:39-70`: `title`, `text`, `warning`, `attachments`) |
| Formatted conversation window | `show_conversation_as_page` (`wx_app.rs:23914`), a frame with a WebView and no menu bar | Browser accelerators off (`:23990`); keys reach the host only through script injected by `wire_the_way_out` (`:13111`) with `PageKeys` (`:13546-13554`) | Already computed there: `let above = reader_text::conversation(subject, parts);` (`:23953`) |
| Preview pane | `wx_app.rs:1514` | Unfocusable; Print from the list covers it | as the list |
| Separate page window | `page_window.rs:205`, its own process | A web page from a link, not a message | Out of scope (derived) |

The header block is `reader_text::headers` (`reader_text.rs:107-141`): `Subject`, `From`, `To` and `Cc` when present, `Date` from `out.date(...)`, `Attachments` with their names. `out.date` is the list's reading form; `date_display::absolute` (`date_display.rs:514`) is the absolute one a page should carry (derived: a printed "2 days ago" is wrong the day after).

**The other modules' items** (stated): `ReadAloud::read_full` for `ContactItem`, `NoteItem`, `TaskItem`, `ReminderItem`, `CalendarEventItem` (`read_aloud.rs:372-530`), each a list of `(label, value)` pairs joined by `spoken` with `". "` (`:181-194`), with long text passed through `long_text::spoken`, which reads pictures by the Reading setting (`long_text.rs:292-297`). Each module's list lookup is wired at `wx_app.rs:1880-1975` (`wire_read_aloud`). A printed item wants the same pairs one to a line and the long text as written (derived).

**Refusal wording** lives in `application::status_sentences` (`status_sentences.rs:117`, `nothing_chosen(Thing)`, with `Thing::MESSAGE` to `Thing::NOTE` at `:60-68`), held by `tests/every_status_sentence_has_one_shape.rs`. (stated)

**Test patterns:** a built-window test is one `#[test]` per file because wxWidgets allows one application per process (`tests/text_selection_offsets.rs:25-27`). `WIXEN_NO_AUDIO` is how CI says the machine cannot play sound, with a test holding the flag honest (`CLAUDE.md`, "The other environment setting"; `ci.yml:20`). (stated)

## 1.3 The measurement: the recommended route, probed

Two throwaway projects under `scratchpad/phase-13-probes/`, each with its own target directory, nothing built or installed in the repository.

**Probe 1, `gdi-print`** (`windows = "=0.62.2"` with `Win32_Foundation`, `Win32_Graphics_Gdi`, `Win32_Storage_Xps`, `Win32_UI_Controls_Dialogs`, `Win32_UI_Controls`): `CreateDCW` for "Microsoft Print to PDF", `DOCINFOW.lpszOutput` set to a file so no Save dialog opens, `StartDocW`, `StartPage`, a Segoe UI font at 11 point, `DrawTextW` with `DT_WORDBREAK`, `EndPage`, `EndDoc`. Output (stated):

```
dpi_y=600 horzres=5100 vertres=6600
StartDocW=3
StartPage=1
measured height=360 rect_bottom=960
DrawTextW=360
EndPage=1
EndDoc=1
file bytes=81908
```

`pdfpurr` 0.4.0 then opened the file and answered `pages=1`. Its text came back as glyph codes (`) U R P ...` for "From ..."), and the file's own `/ToUnicode` CMap maps `<0029>` to `<0046>` ("F"), read by decoding object 10 in Python. So **`pdfpurr` 0.4.0 does not apply this PDF's ToUnicode map** (stated: both readings this session). A PDF attachment made with Microsoft Print to PDF would read as nonsense in the reader today (derived). `PDFPurr/` is one of Pratik's own projects (stated: `C:/Users/prati/Documents/projects/.claude/CLAUDE.md` lists it). This belongs in the ledger and is not this group's work.

**Probe 2, `emf`**: `CreateEnhMetaFileW`, the same `DrawTextW` into a 300-unit-wide rectangle, `CloseEnhMetaFile`, then `EnumEnhMetaFile` reading each `EMR_EXTTEXTOUTW` record's string. Output (stated):

```
enum ok=true records=8
[ ]
[From: Ada Lovelace <ada@example.com>]
[ ]
[Subject: café and a long line that should wrap ]
[ ]
[across the width when DrawText breaks words]
[ ]
[for us, twice over at this narrow width.]
```

Every drawn line comes back exactly, wrapped where GDI wrapped it, accents intact, with single-space records between lines that a test ignores. This is an oracle that needs no printer.

**Why the metafile matters for CI** (stated): CI's Windows jobs run on `windows-latest` (`grep -n runs-on .github/workflows/*.yml`), and "Microsoft Print to PDF" was removed from the Windows Server 2025 runner image; a maintainer's answer on 2025-06-10 was "use `windows-2022` as a workaround as it seems the Print to PDF functionality is removed from Windows 2025 server datacenter version" (`actions/runner-images#12328`, comments read with `gh api`). So a test that spools to Print to PDF passes here and fails on CI unless CI says the printer is absent, the way it says sound is.

**Not probed:** `PrintDlgExW` itself, which needs a person at the dialog. wxWidgets uses the same call for its own dialog (`src/msw/printdlg.cpp:844`, with `pd->Flags = PD_RETURNDC` at about `:960`), and initialises OLE on the interface thread through `wxOleInitModule` (`src/msw/app.cpp:150-170`), which `PrintDlgExW` needs `[ASSUMED]`. wxWidgets' `printdlg.cpp:880-1000` is a reference for filling `PRINTDLGEXW` and freeing its `DEVMODE` and `DEVNAMES`.

## 1.4 Proposed plans for GAP-01

The planner numbers them; the ids here are this document's. Every plan writes `.planning/WINDOWS.md` and every one with a user-visible change writes `docs/changelog.md`, so one plan per wave (CLAUDE.md, "Add the files this project writes by rule to every plan's `files_modified`").

### K1: What a printed page holds, and where it breaks (size M)

- **Closes:** GAP-01's layout half; no line ticked until K2 makes it reachable.
- **Tasks (3):** (1) `application::printing`: a `Printable { title, lines: Vec<Line>, body: String, warning: Option<String> }` built from a `ReaderDocument` and from each PIM item, with dates in the absolute form. (2) `lay_out(printable, measure: impl Fn(&str) -> u32, width, lines_per_page) -> Vec<Page>`: wraps at a space that fits, breaks an unbroken run that does not, keeps a header line and its value together, never leaves a page holding only a page header, and stamps each page "Wixen Mail, <title>, page 2 of 3" (no dash character, CLAUDE.md house style). (3) The fields of the five PIM items given once, as a `fields(out) -> Vec<(label, value)>` beside `read_full`, which then joins them, so speech and paper name the same fields (the existing 51 tests in `read_aloud.rs` hold the spoken form unchanged).
- **Files:** `src/application/printing.rs` (new), `src/application/mod.rs`, `src/presentation/read_aloud.rs`, `src/presentation/reader_text.rs` (only if a `printable()` accessor on `ReaderDocument` belongs there), `guards/guards.toml` (7 records name `read_aloud.rs`, 13 name `reader_text.rs`, counted with the TOML reader over `tests_last_seen` and `file`; re-measure the ones the count check prints), `.planning/WINDOWS.md`.
- **Depends on:** nothing.
- **Spoken or shown:** no. Nothing reaches a person until K2, so no push. (A `pub` item in the library crate draws no `dead_code` warning; CLAUDE.md's "done means it runs" is met by K2, which the phase README should say.)
- **Failing test that starts it:** `application::printing::tests::test_a_message_prints_its_header_lines_before_its_text`, then `test_a_line_longer_than_the_page_breaks_at_the_last_space_that_fits` and `test_the_last_page_says_page_n_of_n`.

### K2: File, Print and Ctrl+P through Windows' own print dialog (size M, three tasks; split point noted)

- **Closes:** GAP-01 `[D]` "File, Print on the mail surfaces ... the route decided by a measurement; the shortcuts page and the guide", for the message list, a conversation row and the reader window.
- **Tasks (3):**
  1. **Waits for Pratik's confirmation of the three `windows` features (question 1).** Then `Cargo.toml` gains `Win32_Graphics_Gdi`, `Win32_Storage_Xps`, `Win32_UI_Controls_Dialogs`, each with a comment saying what it is for, as the eleven already there have. `presentation::printing::draw_page(hdc, &Page, &Font)` and `spool(hdc, document_name, &[Page]) -> Result<Printed, NotPrinted>`, where `NotPrinted` tells cancelled (a file printer's Save dialog closed) from failed, the way wxWidgets' `wxPrinterOperationCancelled` does (`printwin.cpp:218-238`). Red first: `tests/printing_draws_what_the_layout_says.rs`, one `#[test]`, drawing K1's pages into an enhanced metafile and reading every `EMR_EXTTEXTOUTW` string back against the layout, which runs on CI.
  2. `ask_for_a_printer(owner) -> Option<Chosen { hdc, printer_name }>` over `PrintDlgExW` with `PD_RETURNDC`, `PD_NOSELECTION`, `PD_NOCURRENTPAGE`, page ranges allowed; `DEVMODE` and `DEVNAMES` freed; the printer name read from `DEVNAMES` for the sentence. Red first: `tests/printing_spools_a_document.rs`, one `#[test]`, spooling through `spool` to "Microsoft Print to PDF" with `lpszOutput` set to a temporary file and asserting `pdfpurr`'s page count equals the layout's. On a machine where `WIXEN_NO_PDF_PRINTER` is set (CI), the test instead asserts that `CreateDCW` for that printer really fails, so the flag cannot hide a working printer; `ci.yml` sets it beside `WIXEN_NO_AUDIO`.
  3. `ID_PRINT` in the main window's File menu, "&Print...\tCtrl+P", after Export Mailbox and before the PGP key item or after it (the planner reads the PGP record's anchor first, see "Records" below), and in the reader window's File menu; the handler finds the message under the cursor through one function extracted from `open_in_the_text_reader` so the reader and Print cannot build two different documents; a conversation row prints the conversation; nothing chosen is refused with `nothing_chosen(Thing::MESSAGE)`; one sentence when the job is spooled (question 7); the pages and changelog.
- **Split point if the executor finds it long:** task 3 alone as K2b; tasks 1 and 2 alone leave `pub` functions with no caller, so they must not merge to `main` without task 3 (CLAUDE.md, "No feature is done until it runs").
- **Files:** `Cargo.toml` (not `Cargo.lock`: features are not recorded there; measured, section 4), `src/presentation/printing.rs` (new), `src/presentation/mod.rs`, `src/presentation/wx_app.rs`, `src/presentation/wx_reader.rs`, `tests/printing_draws_what_the_layout_says.rs` (new), `tests/printing_spools_a_document.rs` (new), `tests/wired.rs` only if the shortcut and document checks need a new name, `.github/workflows/ci.yml`, `docs/KEYBOARD_SHORTCUTS.md` (File Menu table at `:600-620`, The Reader Window section at `:208`), `docs/USER_GUIDE.md`, `docs/privacy.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`, `docs/development/measurements.md` if the layout's cost on a long message is taken.
- **Depends on:** K1.
- **Spoken or shown:** yes. Push the branch and open a pull request (Pratik's standing OK of 2026-09-23).
- **Gate note:** a branch that changes `Cargo.toml` or a workflow earns the whole gate at its merge (`scripts/which-checks.sh:435` and CLAUDE.md's merge paragraph).
- **Failing test that starts it:** `printing_draws_what_the_layout_says::test_every_line_the_layout_placed_is_drawn_on_its_page`.

### K3: Print from the formatted conversation window and in the other five modules (size S to M)

- **Closes:** GAP-01's remaining surfaces: "on every surface that shows one" and "the other modules' items".
- **Tasks (2):** (1) The five module lists: Print acts on the row under the cursor through K1's `fields` and refuses with `nothing_chosen(Thing::CONTACT)` and its siblings; the help string on the menu item says it prints what the area you are in shows. (2) The formatted conversation window: `Ctrl+P` added to the keys the injected script posts to the host (`PageKeys`, `wx_app.rs:13546`), the host printing the `above` document it already built (`:23953`); the separate page window said plainly to be out, in the guide.
- **Files:** `src/presentation/wx_app.rs`, `src/application/printing.rs`, `docs/KEYBOARD_SHORTCUTS.md` (Getting out of the conversation window, `:340`), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** K2.
- **Spoken or shown:** yes; push and pull request.
- **Failing test that starts it:** `application::printing::tests::test_an_event_prints_its_start_end_place_and_description_one_to_a_line`; for the page key, a case in the test file that already reads the injected script's keys (the executor finds it with `grep -rln 'TheWayOutAndTheJumps' tests/ src/`).

### Records K2 and K3 touch, by both readings

By `tests_last_seen` and `file`, counted with the TOML reader this session: `src/presentation/wx_app.rs` 119 records, `src/presentation/wx_reader.rs` 2, `tests/wired.rs` 19, `Cargo.toml` 0. By anchor text inside the File menu: the record "a PGP private key somebody can import" (`guards.toml:17680`) anchors on the whole `.append_item(ID_IMPORT_PGP_KEY, ...)` call; inserting Print beside it does not change that text, so it is duplicated or moved only if Print is put inside it (it will not be). The executor re-runs the anchor probe the plan-checker gives for the final placement.

## 1.5 Accessibility (GAP-01)

- **Menu items.** Main window File: letters in use `N` (New), `S`, `A`, `M`, `D`, `I`, `O`, `E`, `K`, `Q`, read from `wx_app.rs:6661-6728` and the prepended New submenu (`:6728`); `P` is free. Reader window File: `R`, `S`, `C`, `W` (`wx_reader.rs:287-298`); `P` is free. `tests/wired.rs`'s letter check reads only `wx_app.rs` (`:1977-1978`) and its shortcut-collision check likewise (`:344-353`), so the reader window's letters and keys are checked by hand or by widening those two readings (recommended, as a task in K2).
- **Keyboard shortcut.** `Ctrl+P` is unassigned in the code and in `docs/KEYBOARD_SHORTCUTS.md` (section 1.2). It goes in the File Menu table and The Reader Window section in the same commit, which `test_the_shortcuts_document_and_the_menus_agree` (`tests/wired.rs:932`) holds.
- **The dialog.** `PrintDlgExW` is Windows' own common dialog; its names on UI Automation and MSAA are Windows', not this program's. Whether NVDA reads the printer list, Preferences, page range and copies well is the tester's ear (the requirement's `[S]` line). Owner window is the frame the command came from, so focus returns there when the dialog closes; the plan's built test cannot open the dialog, so focus after it is a listening line (derived).
- **What is said.** One sentence on the status channel when the job is spooled, naming the item and the printer and the page count (question 7), shown in the status bar and spoken, and nothing per page, which keeps it bounded (guardrail 5). A refusal uses `status_sentences`. A cancelled dialog says nothing or one word; the planner picks, and a cancelled Save dialog of a file printer is told apart from a failure.
- **Low vision.** The page is black text on white at a fixed size whatever the screen's theme (derived: a dark theme printed would waste ink and read badly). Whether the size follows the reading font setting is question 6.
- **Hearing, cognitive.** No sound is involved. The help string says what Print prints: "Print the message or item you are on, with its header lines".
- **Privacy.** The job's document name shows in Windows' print queue and on a shared printer's queue (derived from how spoolers work, `[ASSUMED]` for any particular printer). Question 4. Nothing is logged but the page count and the printer name; never the text (CLAUDE.md, "Never log a token, password, or message body").

## 1.6 What cannot be verified here, and what cannot be finished

- A page on paper, and what a sighted reader thinks of it: Pratik or the tester, a listening or looking line, not an account proof. No phase 14 entry is needed; no account is involved.
- The dialog by keyboard under NVDA and Narrator: a listening line.
- CI cannot spool at all (section 1.3); the metafile test is what CI runs. The spool test runs on this machine and says so on CI.
- Pictures and formatting are not printed in this route; the page is the reader's text. The guide and the changelog say so rather than leaving a person to find out (guardrail 9). Question 2.
- Print Preview is not offered (question 3). If Pratik wants it, it is a window drawing K1's pages and its own plan.

---

# 2. GAP-02: Undo and Redo (issue #47)

## 2.1 What the issue and Pratik ask (stated)

`gh issue view 47 --json title,body,comments,state`: open, no comments. The tester, 2026-09-15: "There are no general undo/redo commands that provide corresponding functionality." The issue asks for two levels:

1. "Edit, Undo (`Ctrl+Z`) and Edit, Redo (`Ctrl+Y`) on the main window's Edit menu and in every dialog with a text field, acting on the focused control ... with a multi-step history where the native control gives one step. Greyed and announced as unavailable when there is nothing to undo, so the menu reads its state."
2. "Undo for actions on items: the last delete, move, copy, mark as read or unread, star, and the same in the other modules, as 'Undo delete' or 'Undo move' with the item named, for a bounded time or until the next action; and Redo of that ... It needs the server-side moves to be reversible (a message moved to Trash comes back; a deletion the provider took is another matter) and each action's reverse to be tested against the sync."
3. "Level 1 is small and mostly wiring; level 2 is a design ... and its own plan. Both go on `docs/KEYBOARD_SHORTCUTS.md` and the changelog; level 2 is a minor bump."

The issue's line numbers for the compose editor (`wx_compose.rs:708-721`) have moved: the Undo and Redo buttons are at `wx_compose.rs:821-835` today, named "Undo, Ctrl+Z" and "Redo, Ctrl+Y", dispatched as `editor_document::Format::Undo` and `Redo` (`:1431-1432`, `:2453-2454`). (stated)

## 2.2 What the tree already has

**The Edit menu** (`wx_app.rs:6736-6816`, stated): Undo Send first (`:6762-6766`, "&Undo Send\tCtrl+Shift+Z"), a separator, Cut (`:6782`), Copy, Paste, Select All (`:6797`), a separator, Search (`:6803`), Save This Search (`:6812`). Letters in use: `U`, `T`, `C`, `P`, `A`, `S`, `V`. The comment above Undo Send records why it is `Ctrl+Shift+Z`: "because Ctrl+Z is the editor's undo" (`:6758-6760`).

**What holds that order** (stated): `tests/undo_send_is_where_somebody_looks.rs` (7 tests) asserts the first item appended on Edit is `ID_UNDO_SEND` and carries `Ctrl+Shift+Z` (`:77-90`, test at `:211`), and the guard record "Undo Send is first on the Edit menu, not off it and back where the tester found it" (`guards.toml:24265`) anchors on the whole Undo Send call, the separator and the comment line `// The four every Windows program has.` that follows it. Putting Undo and Redo above it reddens that test and moves that anchor; putting them between Undo Send and the separator moves the anchor too. Either way the record is rewritten and re-measured in the same commit (derived). Question 5.

**How the Edit commands are carried out** (stated): the rule is `application::editing::what_to_do(EditCommand, Where) -> Doing` (`editing.rs:89-131`; `EditCommand` has four members at `:34-39`; 9 tests). The handler is `do_an_edit_command` (`wx_app.rs:18317-18466`), which asks each candidate whether it has focus because "wxdragon offers no way to ask which window has focus" (`:18306-18309`). The candidates are the three main-window text boxes `note_title`, `note_body`, `contacts_search`, the preview page, six lists and two trees (`wx_app.rs:4861-4885`, `EditParts` at `:18470-18485`). Nothing is ever silent: `Doing::NotHere(String)` carries the sentence.

**What wxdragon offers** (stated, `widgets/textctrl.rs` method list read this session): `TextCtrl` has `is_modified`, `set_value`, `write_text`, `replace`, `get_selection`, `window_handle` and the rest, and **no** `undo`, `redo`, `can_undo` or `can_redo`. `RichTextCtrl` (`richtextctrl.rs:364-394`) and `StyledTextCtrl` (`styledtextctrl.rs:585, 1445-1463`) have them; neither is used for typing here.

**What wxWidgets does on Windows** (stated, `target/debug/wxWidgets/src/msw/`):

- `wxTextEntry::Undo` is `SendMessage(EM_UNDO)`, `CanUndo` is `EM_CANUNDO`, and for a plain edit box `Redo` is `Undo` again "since Undo undoes the undo" (`textentry.cpp:734-755`).
- A `Rich2` box uses `EM_REDO` and `EM_CANREDO` (`textctrl.cpp:2012-2033`).
- `wxTextCtrl::DoWriteText` replaces the selection with `EM_REPLACESEL` and `wParam` 1, "to indicate the operation should be redoable" (`textctrl.cpp:1329-1331`), so this program's Paste (`write_text`) is undoable by the box. Its Cut is `box_.replace(from, to, "")` (`wx_app.rs:18412-18415`), which is a removal and then an empty write; in a one-step box the second write may be the step the box remembers (derived, unmeasured). K4 measures Cut then Undo on a real box.
- A menu item that is disabled swallows its accelerator: `if (!item->IsEnabled()) return true;` (`common/framecmn.cpp:360-365`). So Undo greyed because nothing can be undone makes `Ctrl+Z` do nothing and say nothing, which `editing.rs`'s own rule forbids ("A key that does nothing cannot be told apart from a key that does not work", `:15-21`). The existing greying of Mark Done and Pin by module (`wx_app.rs:2121-2138`) has the same property: `Ctrl+Shift+K` in Contacts is silent (derived; not this group's to fix, worth a ledger line).

**Reaching the native control** (stated): the project gets a control's HWND with `HWND(spin.get_handle())` (`presentation/accessibility/names.rs:467`), and `SendMessageW` is in the `Win32_UI_WindowsAndMessaging` feature already on (`Cargo.toml:436-437`). `EM_UNDO` (199) and `EM_CANUNDO` (198) are in `Win32_UI_Controls` (`windows-0.62.2/.../UI/Controls/mod.rs:2529, 2589`), already on (`Cargo.toml:434-435`). `EM_REDO` and `EM_CANREDO` are in `Win32_UI_Controls_RichEdit`, not on and not needed while no typing box is rich (derived).

**Menu open and close** (stated): wxdragon has `on_menu_opened` and `on_menu_closed` (`event/menu_events.rs:108-122`); `get_menu_id` does not say which menu opened (`:39-46`), so the handler updates Undo and Redo on any open. Nothing in `wx_app.rs` binds either today (`grep -n 'MENU_OPEN\|on_menu_open'` found nothing).

**Text boxes in the tree** (stated): 35 `TextCtrl::builder` sites in 12 files (`grep -rn 'TextCtrl::builder' src/presentation | awk -F: '{print $1}' | sort | uniq -c`): `wx_account_manager.rs` 4, `wx_add_address_book.rs` 1, `wx_add_calendar.rs` 1, `wx_app.rs` 3, `wx_compose.rs` 5, `wx_contacts_module.rs` 1, `wx_feedback.rs` 3, `wx_item_form.rs` 4, `wx_managers.rs` 8, `wx_notes_module.rs` 2, `wx_reader.rs` 2, `wx_settings.rs` 1; plus four helpers returning a `TextCtrl` (`wx_add_address_book.rs:174`, `wx_add_calendar.rs:226`, `wx_managers.rs:59`, `:1324`). Only the reader's two are `Rich2` (`wx_reader.rs:463-470`). Dialogs have no menu bar, so in them `Ctrl+Z` is the edit box's own one step today and `Ctrl+Y` does nothing (derived from the above).

**Where actions on items are carried out** (stated):

| Action | Where | How it reaches the server | Reversible? (derived) |
|--------|-------|---------------------------|-----------------------|
| Delete, Delete Permanently (mail) | arm at `wx_app.rs:5304-5503` | Made here first; a queued message is cancelled (`cancel_if_queued`, `:22197`), a local one deleted (`delete_if_local`, `:22424`), a server one kept as an `AWaitingMove` and told in the background through `complete_here_then_tell_the_server` (`:21572`) and `spawn_server_change(..., ServerChange::Deleted(asked))` (`:23529`) | To trash: yes, as a move back. Outright: not once the server has it. A local message: only if the undo keeps the row, since `local_delete` removes it. |
| Move to, Copy to (mail) | arm at `wx_app.rs:4610`, `move_or_copy_message` (`:21058`), `move_or_copy_here_first` (`:21308`) | The same waiting-move row; crossing accounts is `MoveAcross` and `CopyAcross` (`moves_waiting.rs:46-79`) | Move within an account: yes. Copy: yes, as deleting the copy. Across accounts: a second crossing; recommend refused with a sentence at first. |
| Mark read or unread, Star, Labels | `ID_MARK_READ` at `wx_app.rs:5504`, `ID_TOGGLE_STAR` at `:4181` | `ServerChange::Flag(FlagChange::Read/Flagged/Labelled)` (`:23445-23474`), applied here first and put back if refused | Yes, per message: the undo needs each message's state before, since marking a mixed set read sets them all. |
| Delete, Move, Copy, Done, Pin (other modules) | `managers::pim_command` with `PimCommand` (`application/pim_command.rs:21-33`), dispatched at `wx_app.rs:4323-4376` | Deletes keep a deletion note until the provider takes it (`application/deletions.rs:1-40`) and the row is removed | Done and Pin: yes. Move: yes. Delete: only if the undo keeps the whole record; after the provider took it, it is a new item there. |

**The waiting-move row** (stated, `data/message_cache/moves_waiting.rs:14-24, 148-162`): one row per message; "a second move replaces what it is asking for and keeps where it is asking from: the folder and the number the server still holds the message under". So an undo that comes before the server has heard is a second move back to where the row says the server still has it, which would replay as a move from a folder into the same folder (derived): the undo must end the waiting row and put the local copy back, not queue a no-op. An undo after the server has done the move needs the number the message has now in its new folder; how the local row learns that number after a server move was not traced here, and K6b's research step starts there.

**Undo Send, the model** (stated): the decision is pure, `application::sending_later::what_undo_send_takes_back` returning `WhatToTakeBack`, and the wiring is thin (`undo_send`, `wx_app.rs:20593-20640`), with refusals through `UIUpdate::CommandRefused`.

## 2.3 Proposed plans for GAP-02

### K4: Edit, Undo and Redo for the focused text box (size M)

- **Closes:** GAP-02 `[D]` "Edit, Undo and Redo for the focused control on every surface first" for the main window, and the shortcuts page. Dialogs get it in K5b.
- **Tasks (3):** (1) `application::editing`: `EditCommand::Undo` and `Redo` with names; `what_to_do` answers `ToTheText` in an editable box, a sentence in a read-only box ("Undo needs a box you can type in, and this one can only be read."), and a sentence in a list or tree until K6a gives lists their meaning; `test_nothing_ever_happens_without_something_being_said` (`editing.rs:253`) extended over six commands. (2) The two menu items and their ids; `do_an_edit_command` carrying out Undo and Redo with `SendMessageW(EM_CANUNDO)` then `EM_UNDO` on the focused box's HWND, and "Nothing to undo here." when it answers 0; on menu open, Undo and Redo greyed by the focused box's state so the menu reads it, and on menu close both enabled again so the keys always reach the handler and always speak (`framecmn.cpp:364-365`). (3) Undo Send's place and letter rewritten on Pratik's answer to question 5, `tests/undo_send_is_where_somebody_looks.rs` and its record rewritten and re-measured in the same commit, the pages, the changelog.
- **Files:** `src/application/editing.rs`, `src/presentation/wx_app.rs`, `tests/undo_reaches_the_text.rs` (new, one `#[test]`), `tests/undo_send_is_where_somebody_looks.rs`, `tests/wired.rs` if a reading names the menu, `docs/KEYBOARD_SHORTCUTS.md` (Edit Menu at `:625-641`, Application Control's Undo Send row at `:511`), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml` (the Undo Send record; `editing.rs` has 0 records), `.planning/WINDOWS.md`.
- **Depends on:** nothing in this group; Pratik's answer to question 5 before task 3.
- **Spoken or shown:** yes; push and pull request.
- **Failing tests that start it:** `application::editing::tests::test_undo_in_a_text_box_is_the_boxs_own` (pure), and `undo_reaches_the_text::test_edit_undo_puts_back_what_was_typed_and_says_so_when_there_is_nothing`, which builds a frame with a multi-line `TextCtrl`, types through `write_text`, posts the Undo command, reads the value back, and also measures Cut-then-Undo and records what a one-step box does with it.

### K5a: Several steps back in the main window's text boxes (size M)

- **Closes:** the "multi-step history where the native control gives one step" half of GAP-02 level 1, for the main window.
- **Tasks (2 to 3):** (1) `application::text_history`: a bounded history per box (a step is a run of typing up to a word boundary, a paste, a cut or a deletion), undo and redo cursors, redo cleared by new typing, the caret and selection restored with each step, changes the program itself writes (`set_value` on a new selection) starting a new history rather than joining it. (2) `presentation::text_history_keys::keep_a_history(&TextCtrl)`, bound on text-change events, with `Ctrl+Z`, `Ctrl+Y` handled on the box's own key-down and not skipped, so the native one-step undo does not also run; the Edit menu calling the same history when the box has one. (3) The three main-window boxes through it.
- **Files:** `src/application/text_history.rs` (new), `src/application/mod.rs`, `src/presentation/text_history_keys.rs` (new), `src/presentation/mod.rs`, `src/presentation/wx_app.rs`, `src/presentation/wx_notes_module.rs`, `src/presentation/wx_contacts_module.rs`, `tests/undo_reaches_the_text.rs` or a new one-test file, `docs/KEYBOARD_SHORTCUTS.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** K4. Only if Pratik wants it (question 8).
- **Spoken or shown:** yes (what `Ctrl+Z` does changes); push.
- **Failing test:** `application::text_history::tests::test_three_words_typed_come_back_one_word_at_a_time`.

### K5b: The same history in every dialog, with a reading that keeps it there (size M)

- **Closes:** "in every dialog with a text field".
- **Tasks (2):** (1) Every editable `TextCtrl` in the 12 files through `keep_a_history`, the four helpers doing it once for their callers. (2) A reading over `src/presentation` that every `TextCtrl::builder` site not styled `ReadOnly` reaches `keep_a_history`, red on a fixture site that does not, the pattern 12-06 used for `name_the_spin_control`.
- **Files:** the 12 files listed in section 2.2, a new reading target under `tests/`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** K5a.
- **Spoken or shown:** yes; push.
- **Failing test:** the reading, red on its fixture.
- **Note for the planner:** the item form and contact editor files were changed by 12-07 and 12-08 and hold several guard records; take both readings of the register for every file here before writing the plan.

### K6a: Undo a mark, a star or a label on messages, with the message named (size M)

- **Closes:** GAP-02 `[D]` "an undo of ... a mark with the item named", and gives `Ctrl+Z` in the message list its meaning.
- **Tasks (3):** (1) `application::undoing`: `LastAction` holding, per message, the state before (read, flagged, the label on or off), the subject or the count; `Undoing::what_undo_does(&LastAction) -> Undo` and the reverse for Redo; the menu label naming it ("&Undo Mark as Read: Quarterly report" or "... 4 messages"); one step kept, replaced by the next action on items and cleared by nothing else, with no timer (WCAG 2.2.1; question 9). (2) The mark, star and label arms record their `LastAction`; Undo and Redo in a list reach it and send the reverse through the same `ServerChange::Flag` path, so a refusal puts it back and says so as today. (3) The Edit menu's label follows the last action when a list has focus; the pages; the changelog.
- **Files:** `src/application/undoing.rs` (new), `src/application/mod.rs`, `src/application/editing.rs` (lists now answer Undo), `src/presentation/wx_app.rs`, a one-test built-window file, `docs/KEYBOARD_SHORTCUTS.md` (Action Menu and Edit Menu), `docs/USER_GUIDE.md`, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** K4.
- **Spoken or shown:** yes; push.
- **Failing test:** `application::undoing::tests::test_undoing_mark_as_read_over_a_mixed_set_puts_back_each_messages_own_state`.

### K6b: Undo a move, a delete to the trash and a copy of messages (size M, close to L)

- **Closes:** GAP-02 `[D]` "an undo of a delete, a move ... with the item named", for mail.
- **Tasks (3):** (1) Read first, as the plan's premise: how the local row learns its number in the destination after a server move, and whether a waiting row can be ended and the local copy put back in one transaction; `what_undo_does` extended: waiting row still unsent means end it and put the row back; already at the server means a move back through `complete_here_then_tell_the_server`; Delete Permanently, a local message that was removed, and a crossing between accounts refused with a sentence each. (2) The delete, move and copy arms record their `LastAction`; undoing a copy deletes the copy to the trash. (3) Words, pages, changelog; the experimental note on everything that writes (`application::allowed`) extended to cover the undo, since it is a write at somebody's server.
- **Files:** `src/application/undoing.rs`, `src/application/moves_waiting.rs` (18 tests, 10 records), `src/data/message_cache/moves_waiting.rs` (3 records), `src/presentation/wx_app.rs`, `src/application/allowed.rs` if the wording is there, the pages, `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
- **Depends on:** K6a.
- **Spoken or shown:** yes; push.
- **Failing test:** `application::undoing::tests::test_undoing_a_move_the_server_has_not_heard_ends_the_waiting_row_rather_than_moving_back`.

### K6c: Undo in the other five modules (size M; or gated)

- **Closes:** "the same in the other modules" from #47, if Pratik wants it in this phase (question 10).
- **Tasks (2 to 3):** Done and Pin undone as the reverse toggle; Move undone as a move back; Delete undone by recreating from the record the undo kept, with the deletion note removed when the provider has not taken it, and a new item at the provider when it has, said plainly.
- **Files:** `src/application/undoing.rs`, `src/application/pim_command.rs`, `src/presentation/managers.rs` (52 records named it on 2026-09-20 per the 12-08 plan; re-take), `src/presentation/wx_app.rs`, pages, changelog, records, ledger.
- **Depends on:** K6a.
- **Spoken or shown:** yes; push.
- **Failing test:** `application::undoing::tests::test_undoing_a_deleted_task_puts_back_every_field_it_had`.

## 2.4 Accessibility (GAP-02)

- **Letters on Edit.** In use: `U` (Undo Send), `T`, `C`, `P`, `A`, `S`, `V` (`wx_app.rs:6762-6816`). `R` is free for Redo. `U` is the Windows convention for Undo and is Undo Send's today. Question 5 settles it; the plan greps the Edit menu's `&` letters again before writing labels (the global CLAUDE.md rule on mnemonic letters).
- **Keys.** `Ctrl+Z` and `Ctrl+Y` are unbound in every menu (`grep -rnE '\\tCtrl\+(Z|Y)|Ctrl\+Z|Ctrl\+Y' src/` found only the compose toolbar's words and comments). `docs/KEYBOARD_SHORTCUTS.md` lists them only in the Composition Window table (`:934-935`). `Ctrl+Shift+Z` is Undo Send here (`:511`), which many programs use for Redo; the Edit Menu table says so in a sentence so nobody reaches for it expecting Redo.
- **The menu reads its state.** Greyed on open when there is nothing to undo, enabled again on close, and a key press with nothing to undo says "Nothing to undo here." on the status channel, spoken and shown.
- **The item named.** The menu label and the sentence name the item: the subject, or the count for a set, the way Delete and Move already word a set (`application::choosing_messages::what_is_being_done`, used at `wx_app.rs:5405`). One sentence per undo, never per message (guardrail 5).
- **Focus.** After undoing a move or a delete, the message comes back into the list; where the cursor lands is the plan's decision to write down, and the NVDA line is the tester's.
- **Timing.** No time limit on the undo (WCAG 2.2.1); it lasts until the next action on items.
- **Hearing.** Nothing is signalled by sound alone; if a feedback sound plays for a delete, its undo is shown and spoken as text as well.

## 2.5 What cannot be verified here, and what cannot be finished

- An undone move or delete reaching a real server: a write path this program has not proved against a real account. It belongs with phase 14's proofs (REAL-01's move and delete lines), and the experimental wording on everything that writes covers it where the person sees it.
- Delete Permanently cannot be undone once the server took it; that is said, not stubbed.
- A PIM item deleted and taken by the provider comes back as a new item there, with a new identity; said in the sentence.
- Whether "Undone" is heard as undone: the tester's ear (`REQUIREMENTS.md:5787`).

---

# 3. Questions only Pratik can answer

1. **Three more parts of the Windows library the program already uses.** Printing needs the drawing, printing and print-dialog parts of the `windows` library switched on. No new library is added and nothing new is downloaded; the parts come from the copy already in the program. Do you want to confirm this the way you confirm a new package? *Recommendation:* yes, one line in K2's first task, because it is cheap and the rule is simpler kept whole.
2. **Plain text first.** The printed page would carry the header lines and the message's words as the reader window shows them, without pictures or formatting. Is that the first version you want? *Recommendation:* yes; it is what the issue called "the honest first version", and pictures can come later through the browser if people ask.
3. **Print Preview.** Windows' print dialog has no preview. Should there be one? *Recommendation:* no, not now. It helps sighted people only, and a preview window is its own piece of work.
4. **The name of the print job.** Windows shows it in the print queue, and on an office printer other people can see that queue. Use the subject, or a plain name like "Wixen Mail message"? *Recommendation:* the plain name, since mail is private and a shared queue is a room other people are in.
5. **Where Undo and Redo go on the Edit menu, and which letter Undo takes.** Undo Send is first on Edit today with the letter U. *Recommendation:* Undo first with U and Redo second with R, the way every Windows program does it, and Undo Send third with a new letter (N is free). This moves a letter a tester may have learned; the other choice keeps Undo Send's U and gives Undo an unusual letter.
6. **Printed text size.** A fixed 11 point, or the reading font size from Settings? *Recommendation:* fixed 11 point, no new setting; a setting would need a place on the settings screen and nobody has asked.
7. **What is said when something prints.** *Recommendation:* one sentence, "Sent 'Quarterly report' to HP LaserJet 1022, 2 pages." and nothing per page.
8. **Several steps of undo in every text box.** Windows' ordinary text box only remembers one step. Giving every box its own history is two plans (K5a, K5b). Do you want it in this phase? *Recommendation:* yes; it is what #47 asked, and it is test-first work with no new library.
9. **How long an item undo lasts.** *Recommendation:* until the next action on items, with no timer, and one step (the last action), with Redo of that step.
10. **Undo in contacts, calendar, tasks, notes and reminders.** Mail first is two plans. The other modules are a third (K6c), and undoing a deleted item the provider already took makes a new copy there. In this phase, or later? *Recommendation:* in this phase, after mail, because #47 asked for it and the Done and Pin cases are small.
11. **Telling wxDragon about its printing bug.** Its printing never starts a print job on Windows (section 1.2). Filing an issue upstream is sending something out, so it is your call. *Recommendation:* yes, file it, with the four source lines; a fix there would give the program a second route later.
12. **PDFPurr reads Microsoft Print to PDF files as nonsense.** Found while probing (section 1.3): it ignores the file's character map. *Recommendation:* a ledger entry now and a fix in PDFPurr on your schedule, since the reader shows PDF attachments through it.

---

# 4. Dependency audit (the dependency-audit skill's form)

**What we already have** (the skill's first question, measured): the capability is in `windows` 0.62.2, already in `Cargo.lock` and already compiled with eleven features. The functions were found by grepping its source (section "Standard Stack"). Enabling a feature on it adds compiled code from a source already accepted, not a new maintainer to trust.

**Lock cost, measured with a control.** The probe `gdi-print` resolved `windows = "=0.62.2"` with the five features it needed, and its lock was compared with the project's by name and version (Python over both `Cargo.lock` files): 16 packages, 14 identical to the project's, `unicode-ident` at 1.0.26 against the project's 1.0.24 (a newer patch picked by a fresh resolve, compatible, so not a cost in the project's own lock), and no new package but the probe itself. The same comparison run on a `webview2-com` probe did register differences: `webview2-com`, `webview2-com-sys`, `webview2-com-macros` as new, so the instrument sees a new package when there is one. Cargo does not record features in `Cargo.lock`, so K2 changes `Cargo.toml` only (derived from how Cargo works, consistent with the probe).

```
Dependency: windows@0.62.2 (existing), features Win32_Graphics_Gdi, Win32_Storage_Xps, Win32_UI_Controls_Dialogs
Purpose: Windows' own print dialog, the print job, drawing text; an enhanced metafile for the test
Size: 0 new packages against Cargo.lock (probe: 14 of 16 identical, 1 compatible patch difference, 0 new); more compiled bindings inside the existing crate, not measured
Maintenance: microsoft/windows-rs, 6.28 million downloads a week (legitimacy check, 2026-09-24); already the project's platform binding
License: MIT OR Apache-2.0 (windows-0.62.2/Cargo.toml:28); both texts ship in the crate (license-mit, license-apache-2.0 listed in the package directory)
Advisories: none for any windows-family package in `cargo audit --json` (0 vulnerabilities, warnings only for lzw, proc-macro-error, chacha20); none of the five suppressed ids in .cargo/audit.toml names a windows-family package (they name quick-xml via wxdragon-macros, paste, atomic-polyfill and rsa via pgp)
Builds on x86_64-pc-windows-msvc with 1.98.1: yes, the probe built and ran with `cargo +1.98.1`
Alternatives: wxdragon::printing (cannot start a job, section 1.2); WebView print (browser preview, web surfaces only); webview2-com 0.39.1 for ShowPrintUI (3 new packages, MIT, needs windows ^0.62, web surfaces only); writing the Win32 calls ourselves is this recommendation
Defaults overlapping rules we already hold: none
Measured unchanged: not measured (no install was made in the repository)
Manifest readers: tests/house_style.rs::every_dependency reads dependency names only (tests/house_style.rs:1950-1970) and service::outward's census reads names too, so a feature added to an existing entry changes neither; scripts/which-checks.sh:435 makes a Cargo.toml change earn everything at the merge
Recommendation: ADD the three features, in K2 task 1, after Pratik confirms (question 1)
```

```
Dependency: webview2-com@0.39.1
Purpose: WebView2's ShowPrintUI with the system dialog, through get_native_backend()
Size: 3 new packages against Cargo.lock (scratch `cargo generate-lockfile`: webview2-com, -sys, -macros; syn, thiserror and unicode-ident at compatible newer patches)
Maintenance: wravery/webview2-rs, updated 2026-03-11, 21.6 million downloads in all (crates.io API)
License: not checked, since it is not recommended
Recommendation: SKIP. It prints only the three web surfaces and still leaves the reader window and the other modules without print.
```

---

## Environment Availability

| Dependency | Needed by | Available | Version | Fallback |
|------------|-----------|-----------|---------|----------|
| Rust 1.98.1 | every plan | yes (the pin, `rust-toolchain.toml`) | 1.98.1 | none needed |
| MSVC linker | probes, builds | yes (`link.exe` 14.44.35207 in the probe logs) | 14.44 | none needed |
| "Microsoft Print to PDF" on this machine | K2's spool test | yes (`Get-Printer`) | Windows 11 26200 | the metafile test |
| "Microsoft Print to PDF" on CI | K2's spool test | no (`windows-latest` is Server 2025, runner-images#12328) | none | `WIXEN_NO_PDF_PRINTER` in `ci.yml`, the metafile test carrying CI |
| A real printer | a page on paper | yes, HP LaserJet 1022, the default | none | Print to PDF for everything but paper |
| NVDA, Narrator | listening lines | on Pratik's machine and the NVDA workflow | none | none |

A path-length trap met while probing: a target directory deep in the scratchpad failed to link a build script with `LNK1104` because the path passed 260 characters; a short target directory fixed it (stated, this session). Worth a line in the probe instructions of later briefs.

## Validation Architecture

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust 1.98.1), unit tests beside the code, integration targets under `tests/` |
| Config file | none beyond `Cargo.toml`; `WIXEN_TEST_THREADS` for scoped runs |
| Quick run | one module per call: `cargo test --lib application::printing::` then `cargo test --lib application::editing::` (never two `--lib` in one call, CLAUDE.md) |
| Full suite | `bash scripts/check.sh all`, by hand once in the phase's closing plan |

| Req | Behaviour | Type | Command | File exists? |
|-----|-----------|------|---------|--------------|
| GAP-01 | layout: header lines, wrapping, page numbers | unit | `cargo test --lib application::printing::` | no, K1 |
| GAP-01 | every laid-out line is drawn | built, no printer | `cargo test --test printing_draws_what_the_layout_says` | no, K2 |
| GAP-01 | a document spools with the right page count | built, needs Print to PDF | `cargo test --test printing_spools_a_document` | no, K2 |
| GAP-01 | Ctrl+P on menus and on the shortcuts page agree | reading | `cargo test --test wired` | yes |
| GAP-02 | Undo and Redo decisions per place | unit | `cargo test --lib application::editing::` | yes, 9 tests to extend |
| GAP-02 | Edit, Undo restores a real box | built | `cargo test --test undo_reaches_the_text` | no, K4 |
| GAP-02 | several-step history | unit | `cargo test --lib application::text_history::` | no, K5a |
| GAP-02 | every editable box keeps a history | reading | a new target, K5b | no |
| GAP-02 | item undo decisions | unit | `cargo test --lib application::undoing::` | no, K6a |

Manual lines: the dialog by keyboard under NVDA; a page on paper; an undone move heard; an undone move reaching a real server (phase 14).

## Security Domain

| ASVS category | Applies | Control |
|---------------|---------|---------|
| V5 Input validation | yes | Message text is a stranger's (guardrail 6). It reaches the page as text drawn with `DT_NOPREFIX` so an ampersand is not taken as a mnemonic, and `DT_EXPANDTABS`; no markup is interpreted. A title with a NUL is not an issue on this route (the `CString::new(...).unwrap()` in wxdragon's proxy is avoided by not using it). |
| V7 Error handling and logging | yes | Log the printer name and the page count only, never the text or the subject. |
| V8 Data protection | yes | Printing is a way mail leaves the machine; the spool file sits in the Windows spooler until printed. `docs/privacy.md` gets a sentence (it has none: `grep -n -i print docs/privacy.md` matched only "fingerprint"). The job name is question 4. |
| V2, V3, V4, V6 | no | No sign-in, session, access rule or cryptography involved. |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | `StartPage` on a printer DC without `StartDoc` spools nothing | 1.2 | wxdragon's route might print after all; K2 would still be the better route for every surface, but the wxDragon issue (question 11) would be wrong. |
| A2 | `PrintDlgExW` needs OLE initialised on the calling thread, and wxWidgets has done it | 1.3 | The dialog might fail to open; wxWidgets' own dialog uses the same call, so low. |
| A3 | A shared printer's queue shows the job name to others | 1.5 | Question 4 would matter less. |
| A4 | Cut through this program's Edit menu leaves a one-step box unable to undo the cut | 2.2 | K4 measures it; only the sentence K4 writes changes. |
| A5 | A menu opened from the keyboard leaves Windows' focus on the box, so `has_focus` answers during the open handler | 2.3, K4 | Undo would grey wrongly on open; K4's built test covers it. |

## Open Questions for the planner (not Pratik)

1. How the local row learns its new number after a server move (K6b's premise). Not traced here.
2. Whether `tests/wired.rs`'s letter and shortcut checks should widen to the reader window's menus in K2, or a separate reading.
3. The pdfpurr ToUnicode finding and the silent greyed-accelerator finding (`framecmn.cpp:364-365` against `wx_app.rs:2121-2138`) each want a ledger line; which plan files them.

## Sources

### Primary (read this session)
- wxdragon 0.9.17 and wxdragon-sys 0.9.17 in the cargo registry: `src/printing.rs`, `cpp/src/print.cpp`, `cpp/src/webview.cpp`, `src/widgets/textctrl.rs`, `src/widgets/webview.rs`, `src/event/menu_events.rs`, `src/dc/mod.rs`.
- wxWidgets 3.3.2 as built under `target/debug/wxWidgets`: `src/common/prntbase.cpp`, `src/msw/printwin.cpp`, `src/msw/dcprint.cpp`, `src/msw/printdlg.cpp`, `src/msw/textctrl.cpp`, `src/msw/textentry.cpp`, `src/common/framecmn.cpp`, `src/msw/webview_edge.cpp`, `src/msw/app.cpp`.
- windows 0.62.2 in the cargo registry: `Cargo.toml`, `Storage/Xps`, `UI/Controls/Dialogs`, `Graphics/Gdi`, `UI/Controls`, `UI/Controls/RichEdit`.
- Upstream wxDragon `main`: [print.cpp](https://raw.githubusercontent.com/AllenDang/wxDragon/main/rust/wxdragon-sys/cpp/src/print.cpp), [printing.rs](https://raw.githubusercontent.com/AllenDang/wxDragon/main/rust/wxdragon/src/printing.rs), [printing_demo](https://raw.githubusercontent.com/AllenDang/wxDragon/main/examples/rust/printing_demo/src/main.rs).
- [actions/runner-images#12328](https://github.com/actions/runner-images/issues/12328): Print to PDF removed from the Windows Server 2025 image.
- Issues #45 and #47; `.planning/ROADMAP.md:1463-1513`; `.planning/REQUIREMENTS.md:5767-5787`; `CLAUDE.md`.
- Probes: `scratchpad/phase-13-probes/gdi-print`, `pdf-read`, `emf`, `wv2`.

### Secondary
- crates.io API for `webview2-com` versions and dependencies; the GSD package legitimacy check.

## Metadata

- Standard stack: HIGH, the functions and features were read in the crate source and a probe built and ran.
- Architecture: HIGH for where things live (read); MEDIUM for the plan shapes.
- Pitfalls: HIGH for the wxdragon printing defect's mechanism (four sources), MEDIUM for its effect (not run, A1).
- Valid until: the next wxdragon upgrade for section 1.2; 30 days otherwise.

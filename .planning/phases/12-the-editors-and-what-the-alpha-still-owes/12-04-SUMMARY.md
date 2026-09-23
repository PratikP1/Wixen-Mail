---
phase: 12-the-editors-and-what-the-alpha-still-owes
plan: 04
subsystem: the About dialog, its words in one module, LICENSE and the executable's copyright
tags: [alpha-01, about, licence, links, msaa, syslink, theme, "#78"]
status: complete

requires:
  - phase: 12-the-editors-and-what-the-alpha-still-owes
    provides: "12-03.2: the gate this plan commits and merges under, and main at c546707a"
provides:
  - "src/application/about.rs: COPYRIGHT, LICENCE_NAME, HOME_PAGE, SUPPORT_PAGE, lines() and shown_as(), five cases"
  - "src/presentation/wx_app.rs: build_about_dialog reading its lines from application::about; add_a_page_link and open_one_of_our_pages"
  - "src/presentation/theme.rs: paint_link, the accent for a link's normal and visited colour"
  - "LICENSE line 3 and build.rs's LegalCopyright in step with COPYRIGHT"
  - "tests/the_about_dialog_names_its_owners_and_its_links.rs: eight cases over the built dialog, LICENSE and build.rs"
  - "three guard records, measured"
affects: [12-05, which adds Send Feedback to About and rewrites the absence reading in place; 12-12, which reads the four pages as one]

actuals:
  tokens: 15200
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "A control kind chosen by reading the built control over MSAA, bare and wrapped, before the dialog is written"
    - "A link event consumed in its handler through the untyped bind, because the typed one cannot mark it handled"
    - "A colour a native control will draw with, read by sending WM_CTLCOLORSTATIC to its parent and asking the device context"

key-files:
  created:
    - src/application/about.rs
    - tests/the_about_dialog_names_its_owners_and_its_links.rs
    - .planning/phases/12-the-editors-and-what-the-alpha-still-owes/12-04-SUMMARY.md
  modified:
    - src/application/mod.rs
    - src/presentation/wx_app.rs
    - src/presentation/theme.rs
    - LICENSE
    - build.rs
    - tests/theme_reach.rs
    - guards/guards.toml
    - docs/changelog.md
    - docs/USER_GUIDE.md
    - .planning/WINDOWS.md
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
    - .planning/STATE.md

key-decisions:
  - "The two pages are native links (SysLink), not buttons: the link's item answers role link with the address as its name, measured over MSAA on the built control"
  - "The links carry no set_accessible_name, because attaching it left the SysLink with no item at all; the name on both channels is the control's own text"
  - "The link event is consumed in its handler, so wxWidgets does not also launch the browser on the raw address around safe_external_url"
  - "OK is the default and has the focus, so Enter still closes About as it did when OK was all it held"
  - "build.rs's LegalCopyright, a fourth reader of the copyright, is brought into step and held by the target"

metrics:
  duration: "about 50 minutes, from about 17:45Z to 18:45Z on 2026-09-23, before the pull request's wait"
  completed: 2026-09-23
---

# Phase 12 Plan 04: The About dialog names its owners and its pages Summary

**It works in every reading a test can take, and nobody has heard it yet.** About reads its four lines from `application::about`: "Wixen Mail", the whole version with its build counter, what the program is, and "Copyright 2024-2026 Pratik Patel and the Wixen Project, with other contributors." followed by "Released under the MIT licence." `LICENSE` line 3 and the copyright `build.rs` stamps on the executable say the same, held by one target. After the copyright come two native links, wixen.app and wixen.app/support, then OK. There is no Send Feedback button; 12-05 adds it with its dialog.

What the tests cannot see, said plainly: whether NVDA reads the dialog as the MSAA reading says it will (ledger 587), and what pressing a link does in a running program, because a test that pressed one would open a browser on this machine. Both pages still answered 522 at 18:31Z on 2026-09-23 (ledger 588, Pratik's to close).

## The control kind, measured over MSAA (premise 2)

Taken on 2026-09-23 before the dialog was written, in a throwaway first version of the target file that built both candidates on a hidden dialog and read each with `AccessibleObjectFromWindow(hwnd, OBJID_CLIENT)`. Both labelled `wixen.app/support`; the test binary carries the manifest, so the link is the native `SysLink` the program gets:

```
HyperlinkCtrl named=false class="SysLink": children n=1;
  [child 0: role 0xa (client), state 0x108000, name "wixen.app/support"]
  [child 1: role 0x1e (link), state 0x500000, name "wixen.app/support", value "https://wixen.app/support"]
Button named=false class="Button": children n=0;
  [child 0: role 0x2b (push button), state 0x100000, name "wixen.app/support"]
HyperlinkCtrl named=true class="SysLink": children n=0;
  [child 0: role 0xa (client), state 0x100000, name "wixen.app/support"]
Button named=true class="Button": children n=0;
  [child 0: role 0x2b (push button), state 0x100000, name "wixen.app/support"]
```

**Kept: the link, without `set_accessible_name`.** Its item answers the role a screen reader names as "link", with the address as its name and the whole URL as its value, which is what these two things are. A button would have been heard as a button that opens a web page. The third reading is the finding: the naming object this program attaches everywhere else replaced the `SysLink`'s item and left a bare client object, so a link "named on both channels" by that call would have stopped being a link while a check on the name still passed. The name both channels read is therefore the control's own text, which is the address; the file header of the target and the doc comment on `build_about_dialog` say so. UI Automation is read by the pull request's Accessibility scan.

## What else the build needed

- **The link event is consumed.** wxdragon's typed `on_clicked` hands the closure a type with no way to mark the event handled, and its dispatcher counts an event consumed only if the handler un-skips it. Left unconsumed, wxWidgets' `SendEvent` launches the browser on the raw URL itself, so the page would open twice and once around the gate. The handler binds `COMMAND_HYPERLINK` untyped, calls `skip(false)`, and opens the page through `safe_external_url`; a failure to open says so in a message box over the dialog.
- **The links are drawn in the accent.** Read by sending `WM_CTLCOLORSTATIC` to the dialog for each link: unpainted, the dialog told them to draw in #0066CC, the system link colour, which on the dark surface #16130F is about 3.3:1 (worked by hand from WCAG's formula) where text needs 4.5:1. `theme::paint_link` sets the normal and visited colour to the palette's accent, which the theme's own contrast tests hold to 4.5:1 on both surfaces; nothing is painted in high contrast.
- **OK keeps the focus.** It is the default button and takes the focus, so Enter still closes About as it did when OK was all it held; Tab and Shift+Tab reach the links. Where focus lands on opening is not readable without showing the dialog, which a test here must not do, so it is on ledger 587.
- **The dialog is fitted to its contents.** The fixed 380 by 260 did not hold two copyright lines and two links.

## Commits and what each hook run printed

| Commit | What | Mode | Stage line (seconds) |
|---|---|---|---|
| `ff5b80f9` | red: the module's five cases and the target's five readings | red | 107 in all: start 4, rustfmt 3, clippy 28, the scripts that decide what runs 7, the tests this commit says must fail 0, the tests that reach what changed 65 |
| `c724168f` | green: the module, the dialog, `paint_link`, `LICENSE`, `build.rs`, the doc sentence, three records, the changelog | affected | 183 in all: start 3, rustfmt 3, clippy 38, the scripts that decide what runs 33, the tests that reach what changed 106 |
| `82649701` | the guide, the ledger, this summary, the four marks | docs_only | 86 in all: start 3, rustfmt 3, clippy 30, the scripts that decide what runs 1, the targets that read documents 49 |
| `2e00c42b` | the merge into `main`, `git merge --no-ff` | "mode affected, a merge into main" | 196 in all: start 1, rustfmt 3, clippy 27, the scripts that decide what runs 33, the tests that reach what changed 132 |

The last row was added after the merge, by a commit on `main` touching only this file.

## The pull request's runs

Pull request #95, pushed under Pratik's standing OK of 2026-09-23, read before the merge:

| Run | Verdict |
|---|---|
| CI | Rustfmt, Clippy, Security Audit, Build (debug), Build (release), Setup Executable passed; Test Suite passed in 19m41s, which is the whole suite over this branch on the runner |
| NVDA screen reader tests | passed, 24m20s |
| Accessibility scan | passed, 8m01s; About: Axe.Windows "0 errors were found", "Walked 2 window(s): 'About Wixen Mail', 'Wixen Mail'", "MSAA walk: 1891 elements, 1110 of them operated, 0 without a name" |
| Tests that would notice | failed after 28m13s in `every_number_carries_its_command_and_its_date`, "18a02454 is not in this history", before any mutant was tested: ledger 458's shape, not this change's |

What the scan's two files say about the new controls, read from the downloaded artefact. Over UI Automation, in the order the dialog holds them: four texts carrying the four lines, then `pane 'wixen.app'` (class `SysLink`) holding `link 'wixen.app'` with value `https://wixen.app`, then `pane 'wixen.app/support'` holding `link 'wixen.app/support'` with value `https://wixen.app/support`, then `button 'OK'`; both the pane and the link answer focusable. Over MSAA, each link window answers role 10 (client) named by its address with one child of role 30 (link) named by its address, then OK as push button. So the running release build answers on both channels what the test binary answered, and the one open question the tree cannot settle is how NVDA and Narrator speak a focusable pane that holds a focusable link (ledger 587).

## The red half

The stub table, run against the stubs before the red commit (`cargo test --lib application::about::` and `cargo test --test the_about_dialog_names_its_owners_and_its_links`):

| test | stub it ran against | failed on | verdict |
|---|---|---|---|
| `application::about::tests::test_the_copyright_names_both_holders_and_the_other_contributors` | the old copyright | assertion, the old holder | red |
| `..._test_the_two_pages_pass_the_gate_every_link_takes_unchanged` | empty addresses | assertion, "" against the literal | red |
| `..._test_a_page_is_shown_as_its_address_without_the_scheme` | identity | assertion, scheme kept | red |
| `..._test_the_version_line_carries_the_whole_build_string` | four empty lines | assertion, no "Version " | red |
| `..._test_the_lines_say_the_name_what_it_is_and_the_copyright_under_the_licence` | four empty lines | assertion | red |
| `test_the_dialog_reads_its_lines_then_the_two_pages_then_ok` | the dialog as it stood | five children where seven are wanted | red |
| `test_each_page_is_a_link_named_by_its_address_over_msaa` | the dialog as it stood | 0 links | red |
| `test_each_page_is_drawn_in_the_palettes_accent_on_both_palettes` | the dialog as it stood | 0 links on each palette | red |
| `test_the_licence_file_says_the_copyright_the_dialog_says` | `LICENSE` as it stood | line 3 "Copyright (c) 2026 Pratik Patel" | red |
| `test_the_build_script_stamps_the_same_copyright_on_the_executable` | `build.rs` as it stood | "Copyright (c) Pratik Patel. MIT licensed." | red |
| `test_there_is_no_send_feedback_button_until_its_dialog_arrives` | the dialog as it stood | nothing | **green, a pin** |
| `test_the_readings_refuse_a_dialog_missing_a_page` | the readings themselves | nothing | **green, a companion** |
| `test_the_licence_reading_refuses_a_stale_copyright_line` | the reading itself | nothing | **green, a companion** |

3 of 13 were green against the stub, each named in the red commit as not red and why: the absence of Send Feedback is the plan's pin, paired with an assertion that more than one child was read, and the two companions hold the pure readings to refusing a wrong input. The ten red ones are the commit's `Fails-until-green` trailers; the gate held the commit to exactly them.

## Tests

- `cargo test --lib application::about::`: 5 passed (new).
- `cargo test --test the_about_dialog_names_its_owners_and_its_links`: 8 passed (new).
- `cargo test --test theme_reach`: 7 passed, its doc sentence corrected and its reading unchanged.
- `cargo test --lib presentation::wx_app::`: 199 passed, 199 before and after; no test was added to `wx_app.rs`.
- `cargo test --test house_style` 74 passed and `cargo test --test the_words_that_say_nothing` 10 passed over the changelog and the guide.

No crate was added to `Cargo.toml` (T-12-SC).

## Guard records

Three, each measured by one `scripts/guards.sh --remeasure` call naming all three, 48 s, the target named as each record's `suite`:

| Record | Break | Red |
|---|---|---|
| the About dialog's copyright is the line LICENSE and the executable carry | `COPYRIGHT`'s first year 2024 to 2025 | the licence reading and the `build.rs` reading, 2 of 2; the order reading stays green because it reads the dialog against the constant it was built from |
| the About dialog offers the support page as a link after the home page | the second `add_a_page_link` call replaced by `let _ = about::SUPPORT_PAGE;` | the order, MSAA and colour readings, 3 of 3 |
| the About dialog's links are drawn in the palette's accent | `theme::paint_link(&link, palette);` replaced by `let _ = palette;` | the colour reading, 1 of 1 |

Each printed "went red, and nothing else did". No break can start another program: none clicks anything or reaches `open::that`. The count check flagged no existing record: no file an existing record names gained or lost a test (`wx_app.rs` stays at 199). The sweep header's "arrived since" went from 247 to 250, and the file holds 1,047 records by `test_the_sweep_written_at_the_top_of_the_guard_records_covers_every_record_in_it`.

## The guide's About paragraph, whole

Quoted because a grep cannot hold a wrapped paragraph to saying nothing about whether the site is up (acceptance criterion 2 of task 2):

> **Help → About** shows who holds the copyright: Pratik Patel and the Wixen Project, with other contributors, under the MIT licence. It shows the whole version with its build number, such as `1.0.0-alpha.1+114.g44bff634`: the version, then how many commits the build is past the point that version was set, then the commit it was made from. A later build has the larger number, so quote the whole string when you report something. After the copyright come two links, wixen.app and wixen.app/support, and then OK. OK has the focus when About opens, so Enter closes it. Press `Shift+Tab` to reach the links; Enter on a link opens that page in your browser.

The In-App Help list now names the Help menu's real items, with the two old lines corrected by a dated sentence, and Report Issues gains one dated line that Send Feedback is coming with a window of its own. `grep -c '522' docs/USER_GUIDE.md` is 0.

The changelog kept a dated Known limitations note that neither page answered on 2026-09-23, with the code and what it means, beside Pratik's answer that both are up by public testing. His answer named the dialog and the guide; the changelog is the dated record a build is read against, and a build cut before the site is up carries two links to pages that do not load. The note is his to overrule.

## Deviations from Plan

**1. [Rule 2 - Missing critical] `build.rs` brought into step.** Found in task 1 by a search for every copyright statement in the tree: `build.rs` stamped "Copyright (c) Pratik Patel. MIT licensed." into the executable, which Windows shows in the file's properties. It now carries the same line as `LICENSE` with "MIT licensed.", and the target holds it. Commit `c724168f`.

**2. [Rule 2 - Missing critical] `theme::paint_link` added.** `theme.rs` was not in the plan's files. Without it the links drew in #0066CC, about 3.3:1 on the dark surface. Commit `c724168f`.

**3. [Rule 1 - Bug] The fixed dialog size removed, and OK given the focus.** The new lines did not fit 380 by 260, and with the links first in the tab order wx would have put the focus on the home page link, so Enter, which used to close About, would have opened a browser. Commit `c724168f`.

**4. [Rule 1 - Bug] The link event consumed through the untyped bind.** The typed handler cannot consume it; see "What else the build needed". Commit `c724168f`.

**5. `set_accessible_name` is not applied to the links**, where the plan said "named on both channels with the same words". The measurement above shows the call erases the link item; the name both channels read is the control's text, the same words. The plan's premise 2 anticipated a measured choice; this is its answer, not a departure from its intent.

**6. Acceptance criterion `grep -c 'Pratik Patel and the Wixen Project' ... src/application/about.rs` reports 3, not 1.** The constant is one; the other two are the module's own tests holding it and `lines()` to literals, which is the point of those tests. `LICENSE` reports 1.

**7. The changelog entry went into the green commit**, not the documents commit, because `CLAUDE.md` asks for a user-visible change's entry in the same commit as the change.

**Total deviations:** 4 auto-fixed (2 Rule 2, 2 Rule 1), 3 recorded as differences from the plan's letter. **Impact:** the dialog does what the plan asked and holds two things it did not name, the executable's copyright and the links' contrast.

## Threat model

T-12-12 and T-12-13 hold as planned: the addresses are constants with no query, each passes `safe_external_url` in a case, and the one opener goes through it. One surface the model did not name, closed here: the framework's own launch of an unconsumed link event, which would have bypassed the gate. No new surface is left open.

## Known Stubs

None. Every function added has a caller on the path from Help, About.

## Deferred

`theme::REACH`, the sentence the Settings screen shows about where colour applies, does not list About, and About has been painted since before this plan. Out of scope here; noted for 12-12's read of the pages.

## Self-Check: PASSED

Files: `src/application/about.rs`, `tests/the_about_dialog_names_its_owners_and_its_links.rs` and this summary exist. Commits: `ff5b80f9` and `c724168f` are on the branch (`git log --oneline main..HEAD`). The ledger's two halves carry 587 and 588, and its front matter reads 546 open of 588.

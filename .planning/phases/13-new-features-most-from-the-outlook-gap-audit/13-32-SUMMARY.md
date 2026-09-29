---
phase: 13-new-features-most-from-the-outlook-gap-audit
plan: 32
subsystem: presentation
tags: [item-form, scrolling, reflow, wcag-1.4.10, wcag-1.4.4, wcag-2.4.11]
status: complete
requires: [13-31]
provides: [the item form's pages scroll the field in focus into view, the item form capped to its screen]
affects: [src/presentation/wx_item_form.rs]
tech-stack:
  added: []
  patterns: [a scrolled page per field list, fitted to its fields before the scroll rate is set]
key-files:
  created:
    - tests/the_event_form_scrolls_to_the_field_in_focus.rs
  modified:
    - src/presentation/wx_item_form.rs
    - tests/item_form_recurrence_tab.rs
    - guards/guards.toml
    - docs/changelog.md
    - .planning/WINDOWS.md
decisions:
  - "wxWidgets' own scrolling: the page scrolls a focused field into view with no handler, as decision 2 expected, but only a field that fits the page, so the form keeps a page as tall as its tallest box"
  - "Twice the text read as the smallest size the form can be made: a dialog does not inherit its parent's font and Windows' text size is not a test's to set"
  - "The cap read as the largest size the form can be made, which is set on any screen"
metrics:
  duration: about 3 hours
  completed: 2026-09-29
actuals:
  tokens: 12500
  tasks: 2
  commits: 6
---

# Phase 13 Plan 32: Edit Event scrolls Summary

Every page of the item form is a scrolled window now, so Edit Event and the
reminder, task and note forms scroll to the field in focus, open no bigger
than their screen, and keep Save and the problem line in view however small
they are made (ledger 417, #57 point 4).

On the pull request (134), the Accessibility run 36513896729 read new-event
on the runner's 1024 by 768 screen at 553 by 720 from the top of the screen,
its page 562 tall and scrolling, Show as, Status and Category 23 pixels tall
each where they were six, and Save at 675 to 698, on the screen. The first
run, 36510066030, before the centring, read Save at 770, below the edge; that
is deviation 8. #57 point 4 is answered: every field of Edit Event can be
reached and brought into view on that screen. CI, NVDA and Accessibility
passed on both rounds; "Tests that would notice" failed on both before testing
any mutant, on `test_the_share_of_history_before_red_green_is_computed_and_printed`
finding no git history in its copy of the tree, which is ledger 458.

## What was built

- `a_scrolled_page_of` builds every page: the one page of a task or a note,
  and both notebook pages of an event or a reminder. It passes the panel's tab
  traversal style as bits, because wxdragon's scrolled window style has no name
  for it and without it Tab would leave the page.
- `fit_within_the_screen` fits the dialog to its fields while no page can
  scroll, then sets the scroll rate, caps the size and the largest size to the
  working area of the parent's screen (`the_screen_around`, Windows only), adds
  a scroll bar's width, lowers the smallest size to a page as tall as
  `the_tallest_field` plus two scroll steps, and centres the form, which keeps
  it inside its screen's working area.
- `save` and `problem_line` are public on `ItemFormWidgets`, for the readings.

Which was needed, as the plan asked: wxWidgets' own child-focus scrolling. No
focus handler is written. wxWidgets does not scroll to a field taller than its
page at all (read in `scrlwing.cpp`, and seen in the first red's half-room
reading), so the smallest size keeps the tallest box whole instead.

## Readings

`tests/the_event_form_scrolls_to_the_field_in_focus.rs`, 14 tests, all
passing at the green: the form given 480 pixels and given no room at all
(which it answers with its smallest), the description and Times offered inside
every window they sit in once they have focus, Save and the problem line in
the window, the size it opens at and the largest it can be made against the
working area, the whole form inside the working area when opened from a window
low on the screen, a task's last field showing as built, and the Tab walks of
the event and task forms against the order read on `main` at `d18296cd`.

The existing item form targets pass unchanged in count: `item_form_free_busy`
7, `item_form_recurrence_tab` 1, `event_times_move_in_blocks` 16,
`every_spin_control_names_the_field_a_person_types_in` 15 and 1 ignored,
`a_spin_controls_field_is_named_where_the_scan_reads_it` 4 and 2 ignored,
`theme_reach` 7, `checkbox_labels` 1, `every_text_box_keeps_a_history` 10,
`item_form_date_time_fields` 1, `item_form_prefill` 1, `item_form_validation`
1, `a_new_item_is_headed_new` 3, and `presentation::wx_item_form::` 16.

## Commits and hook times

| Commit | What | Hook |
|---|---|---|
| `f0efce7c` | test: the form at 480 and half of it, the cap, the Tab walks, red | red, 144 s |
| `9a13b2a5` | test: the half-room readings replaced by the smallest size, red | red, 96 s |
| `643fb7f1` | feat: the pages scroll and the form fits its screen; one record added | affected, 227 s |
| `e45e5746` | test: the form opened low on the screen lies inside it, red | red, 94 s |
| `3ffd7800` | fix: the form centred inside its screen once sized; one record added | affected, 235 s |
| docs | the ledger, the summary and the marks | this commit |

## Guard records

Two new. "the item form's pages scroll the field in focus into view", the
scroll rate taken to nought, reddens the four focus readings and nothing else;
measured alone in 28 seconds at the first green. "the item form is placed
inside its screen after it is sized", the centring taken away, reddens the
placement reading and nothing else; both measured in one `--remeasure` call
of 64 seconds at the second green, the first again because the target it
names had gained a test. The arrived-since count went from 462 to 464.
`wx_item_form.rs` stays at 16 tests; the new target holds 14. The five records on
`wx_item_form.rs` anchor at the heading, `Opening::for_a_form_of`, the check
box, the minute spinner and the birthday; none lies in the rebuilt block, all
still name one place, and none needed re-measuring.

## Deviations from Plan

**1. [Plan] Twice the text is read as the smallest size, not as a doubled
font.** The plan asked for the form built with its font doubled. A dialog does
not inherit its parent's font (measured: the parent set to 18 points, the
form's title at 9), wxdragon has no way to reach a built form's labels, and
Windows' text size is Pratik's machine's setting. Twice the text in a room is
the same form in half of it, since every part doubles, so the file reads the
form from the smallest it can be made up to its own. The first red read half of
480; that found the description taller than the page it left, which wxWidgets
then does not scroll to, so the form now keeps a page as tall as its tallest
box and a second red replaced those two readings. Width at twice the text is
not read.

**2. [Plan] The cap is read as the largest size.** On a screen tall enough for
the form, the size it opens at is within the working area with or without a
cap, so the reading that goes red without the cap is the largest size.

**3. [Rule 3] `tests/item_form_recurrence_tab.rs` changed.** Its `on_page`
took a `&Panel`, and a page is a `ScrolledWindow` now; it takes any widget.
Same test, same count.

**4. [Found] `navigate(true)` in wxdragon 0.9.17 moves backwards.**
`WXD_NAVIGATION_NEXT` is 0, which wxWidgets reads as `IsBackward`. The Tab
walk's first run went from the title back to the tabs. The test calls
`navigate(false)` through `tab_from` and says why; `wx_compose.rs` already
knew. Nothing is filed upstream, per answer (d).

**5. [Shape] Two red commits.** The second red was taken by keeping the green
edits in the scratchpad, restoring the two files with `git checkout --`, and
making the edits again with Edit after it; the result compared equal to the
kept copy.

**6. [Brief] Four diagnostic commands carried do-nothing shell assignments
named after a banned tool.** None ran it and nothing was written by them.

**7. [Shape] The changelog went in the green commit**, where `CLAUDE.md` puts
a user-visible change, not in the documents commit.

**8. [Rule 1] The form opened with Save below the screen, found by the
scan.** The first Accessibility run on the pull request (36510066030) read
Edit Event on the runner's 1024 by 768 screen scrolling, its page 562 tall
with a scroll bar, Show as, Status and Category full height at 23 pixels, and
the dialog capped at 720, but placed with its top at 95, so its bottom 47
pixels and Save (at 770) were below the screen. A dialog is placed when it is
made, at the size it was made at. None of the readings caught it, because
they read sizes and never where the window lay. A third red and green pair
added the reading, a form opened from a window low on the screen, which went
red on this machine as well (1140 tall from a top of 55 on a working area
1140 tall), and centred the form once sized; wxWidgets keeps a centred window
inside its screen's working area.

## Ledger

Closed: 417, fixed in both halves on the Accessibility run's reading above.
Opened: 712 (unrun-verify, the tester's ear and eye at 200 percent text).
Counts 639 open, 73 fixed, 712 in all.

## Threat Flags

None beyond the register. T-13-32-01: Save and the problem line stay in the
dialog's own sizer, which gives fixed rows their room first, held by the two
Save readings. T-13-32-02: the Tab walks match the order read before; the
scan reads both channels on the pull request. T-13-32-SC: no crate added, and
the two Windows calls use features already on.

## Known Stubs

None.

## Self-Check: PASSED

The five code commits are on the branch, and every file listed exists.

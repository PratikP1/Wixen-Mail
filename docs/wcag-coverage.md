# What the accessibility scans can judge

This page lists every WCAG 2.2 Level A and AA success criterion and says, for
each one, whether the automated checks in this repository can produce a
finding against it, on which channel, and what is left for a person. It
replaces a sentence that used to appear in four places, that automated
scanning covers roughly half of WCAG. That sentence was wrong by about an
order of magnitude, and this page is what the number really is.

**Read this page as a list of what the scans can look at, not as a list of what
they have found.** A "yes" in the table means the scanner has rules that can
fail on that criterion. It does not mean the criterion is met, in any window,
by this application. Whether a criterion is met is a separate question, and
for most of the fifty-five the only thing that can answer it is a person using
a screen reader. Nothing on this page says any criterion is met.

## The two numbers, and when they were taken

Both counts were taken on 2026-09-14, from the sources named, and both are
shown with their parts so that the arithmetic can be checked rather than
trusted.

**Fifty-five criteria.** WCAG 2.2 has 87 success criteria. Reading the level
marker of each from the specification itself gives 31 at Level A, 24 at Level
AA and 31 at Level AAA, which is 86. The one left over is 4.1.1 Parsing, which
WCAG 2.2 marks "Obsolete and removed" and which carries no level at all. So a
Level AA conformance target is 31 + 24 = 55 criteria, not the 56 that is often
quoted, because 56 counts 4.1.1 as it was in WCAG 2.0 and 2.1.

**One hundred and fifty-five rules.** The scanner is Axe.Windows, pinned in the
scan workflow to release `v2.4.2` of 1 November 2024 by its tag and by the
SHA-256 of the file it downloads. That release ships its own rule list,
`axe-windows-rules-2.4.2.md`, and this page was written against that file
rather than against whatever is on the project's main branch today. The file
has 155 rules. Grouped by the standard each rule cites:

| Standard the rule cites | Rules |
|---|---|
| WCAG 1.3.1 Info and Relationships | 61 |
| Section 508 502.3.1 Object Information | 53 |
| Section 508 502.3.10 Available Actions | 23 |
| WCAG 4.1.2 Name, Role, Value | 9 |
| WCAG 2.1.1 Keyboard | 9 |
| Total | 155 |

61 + 53 + 23 + 9 + 9 = 155. Counted a second way, by severity, the file has 76
Error, 63 NeedsReview and 16 Warning rules, and 76 + 63 + 16 is also 155.

So the rules cite exactly three WCAG success criteria. Seventy-nine rules cite
one of those three. The other seventy-six cite a Section 508 clause and no WCAG
criterion at all. Those seventy-six include the fourteen rules whose names
begin with "Name", such as `NameNotEmpty` and `NameIsInformative`, and the
rules about `LocalizedControlType`, which are the rules most people would call
the name and role checks. In substance they are about the same things as 4.1.2.
The rule file does not say so, and this page counts what the file says.

## What a rule count is not

Sixty-one rules citing 1.3.1 do not add up to judging 1.3.1. Each rule tests
one narrow structural property of one element in the UI Automation tree: that
a check box has no child elements, that a heading's level is not lower than
its ancestor's, that a landmark called "main" is not inside another landmark.
Info and Relationships asks whether the structure a sighted person can see is
also available to a screen reader, in every window, for every relationship.
A scan that runs sixty-one rules and finds nothing has found nothing wrong with
sixty-one narrow properties. It has not found that 1.3.1 is met.

The same is true of the other two. Nine rules about the `IsKeyboardFocusable`
property say whether elements of certain types report themselves as focusable.
They do not press Tab. Nine rules about control patterns and sibling names say
whether a button supports the Invoke pattern and whether two sibling list items
have different names. They do not say whether the name is the right name.

This is why "three of fifty-five" is the number on this page and not "155
rules". A rule count read as a proportion of coverage is the mistake the
"roughly half" sentence made.

## The three ways this repository checks a running window

### Axe.Windows over UI Automation

The scan workflow, `.github/workflows/accessibility.yml`, starts the built
application once per window and runs the Axe.Windows command line against the
process. Axe.Windows reads the UI Automation tree, which is what Narrator
reads. Every rule in the table above runs against every element of every
window the process has open.

What that tree is, and is not, is best said by the header of the script that
reads the other channel, `scripts/msaa-names.ps1`, quoted here rather than
paraphrased:

> On Windows there are two accessibility channels. UI Automation is the newer
> one, and Axe.Windows reads it. MSAA, through IAccessible, is the older one,
> and it is what `wxAccessible` implements: `set_accessible_name` puts a name
> there and nowhere else. For a native control such as an edit box or a
> button, Windows supplies its own UI Automation provider, and that provider
> shadows the MSAA object underneath it. So UI Automation reports the system's
> name for those controls, which is usually empty, and never the one the code
> set.
>
> That makes the UI Automation scan wrong in both directions on native
> controls: it reports a missing name where the name is in fact present and
> spoken, and it would report nothing amiss if every `set_accessible_name`
> call in the tree were deleted.

So on this application, which is built from native controls, the Axe.Windows
scan can judge the structure of the tree and cannot judge the names this
codebase sets.

### The MSAA walk

`scripts/msaa-names.ps1` walks the MSAA tree of every visible top-level window
the process owns, which is what NVDA reads for native controls. It asks one
question of each element: if the element's role is one somebody operates, does
it have a non-empty accessible name? The roles it counts as operated are
editable text, push button, check box, radio button, combo box, list, list
item, outline, outline item, page tab, property page, slider, spin button,
hotkey field and progress bar. It reports each operated element with no name
and exits with a code saying whether there were any.

That is the whole of what it asks. It does not check whether the role is the
right role, whether the name is the right name, or whether the value and
state are exposed. So it contributes to one criterion, 4.1.2 Name, Role,
Value, and to the Name part of that criterion only.

Two things to know when reading a clean result from it, again from its own
header: a control with a visible label beside it is named by that label even
when nothing set one, because Windows falls back to the nearest static text,
so a pass does not mean every name came from this codebase. And it only sees
the windows that exist while it runs.

### The NVDA suite

`.github/workflows/nvda.yml` starts a real copy of NVDA against the built
application, presses keys, and reads back what NVDA said. It is the only check
here that listens. It has four tests. Three run:

| Test | What NVDA is asked to say |
|---|---|
| Account manager, sign-in failure | "Signing in failed" when Sign In Again cannot reach a provider |
| Calendar, immediate actions | Edit Event's, Delete Event's and Sync's own answers when each is pressed |
| Filter manager, delete | The sentence Delete would show when nothing is selected |

The fourth, which asks whether the dialog that asks which days you mean
focuses and ticks the same answer, is written and skipped. The scan target it
needed now exists, and the test is still skipped because it has never passed
and un-skipping a test that has never passed is a check nobody reads.

Three tests hear three sentences in three windows. They say nothing about any
other control, window or sentence.

## Which windows the scans reach

As of 2026-09-14 the scan workflow asks for thirty-one windows: the main frame
under the first-run question, the bare main window, its five other module
panels, the window a message opens into for reading, and twenty-three
dialogs, each started on a fresh profile with `--scan-target`. That is
1 + 1 + 5 + 1 + 23. The workflow runs both channels against every one of the
thirty-one.

Seventeen dialogs are outside the scan, because each opens only from inside
another dialog and the scan starts one window per run. By name: the account
edit dialog; Confirm Delete in the Calendar window; Check Spelling, Insert
Table and Preview Before Send in the composer; the contact edit dialog and its
Add Email Address, Add Phone Number, Add Address and Add Custom Field; the
rule, filter, tag and signature edit dialogs; and the three small questions
the application asks, wait-for-an-answer, choose-from-list and ask-for-a-name.

A "yes" in the table is a yes for the thirty-one windows the scan reaches and
for no other.

**The first run against the thirty-one was on 2026-09-14**, and the section
below is what it found. Before the workflow that asks for them was written,
the scan reached eleven windows on the UI Automation channel, and on the MSAA
channel it had read only the main window, every time, because the script
walked the window .NET calls the main one and a dialog is never that window.
This page describes what the scans can judge; the next section is what they
have judged, once, for the thirty-one.

## What the run of 2026-09-14 found

Run 34849526207 of the Accessibility workflow, on `main` at `db98094c`. It ran
because `main` was pushed to the repository that day, which is one of the
workflow's own triggers; nobody dispatched it. Axe.Windows printed
`version 2.4.2` on every target and the downloaded file's SHA-256 matched
the pin. All thirty-one targets wrote a result file and completed their MSAA
walk; none was reported as not scanned.

**Twenty-nine findings on the UI Automation channel, in eight windows. Twelve
controls without a name on the MSAA channel, in two windows.** The run's own
summary said 26, because the step that counted them did not match the sentence
Axe prints for exactly one error; the three windows with one each were
recorded as clean. That is corrected in the workflow and held by a test, and
the 29 is the count from the scan step's log, where each finding is printed
with its element.

The earlier count of five, quoted since 2026-07-26 in the changelog and on
the status page, was taken from one window on one channel. Twenty-nine from
thirty-one windows on two channels is not five findings becoming twenty-nine;
it is a scan looking at thirty times as much. The five were the composer's
WebView2 findings, and they are the first six rows below.

One row per finding. The channel says which check found it, the rule is the
scanner's own wording, and the disposition is either fixed here, ours and
recorded, or somebody else's with the upstream named.

| # | Window | Element | Channel | Rule | Disposition |
|---|---|---|---|---|---|
| 1 | Compose | Pane `Chrome_WidgetWin_1`, the WebView2 host window | UI Automation | Name must not be longer than 512 characters | Ours. The name was the editor page's own address, because the page had no title and WebView2 names the host after the address when there is none. The page now has the title "Message body". Fixed 2026-09-14, version 0.123.1; confirmed only when the next run on `main` reads the name. |
| 2 | Compose | Region `BrowserRootView`, the root of the page | UI Automation | Name must not be longer than 512 characters | Ours, the same address with " - Web content" after it. Fixed with row 1. |
| 3 | Compose | Region `EmbeddedBrowserTabRootView`, provider `msedge.dll` | UI Automation | An on-screen element must not have a null BoundingRectangle | WebView2's own. A Chromium view of size 0 by 0 inside the host, with no automation id and no wx class on it or above it until the host. Upstream: `MicrosoftEdge/WebView2Feedback`. Not filed. Whether an existing issue covers zero-size views was not checked, because the session that wrote this could not read the tracker; issue 2330 there is about screen readers and WebView2 broadly and is not about this. |
| 4 | Compose | Region `TopContainerView`, provider `msedge.dll` | UI Automation | An on-screen element must not have a null BoundingRectangle | As row 3. |
| 5 | Compose | Region `View` under `MultiContentsView`, provider `msedge.dll` | UI Automation | An on-screen element must not have a null BoundingRectangle | As row 3. |
| 6 | Compose | Region `EmbeddedBrowserDownloadView`, provider `msedge.dll` | UI Automation | An on-screen element must not have a null BoundingRectangle | As row 3. |
| 7 | Account Manager | Text, the IMAP Server cell of the one account in the list | UI Automation | Name of a focusable element must not be null | Ours, recorded. The scan's made-up account has no IMAP server, so the cell is empty, and an empty cell in a report list is a focusable element that says nothing. The list is Windows' own; what goes in the cell is this program's. Ledger 407. |
| 8 | Account Manager | Text `Static`, the status line under the buttons | UI Automation | Name must not contain only whitespace | Ours. Built with a label of one space to hold its line open; now built empty, which is one line tall and has no name until it has a sentence. Fixed 2026-09-14. |
| 9 | Account Manager | Thumb `ScrollBar`, the resize grip in the corner | UI Automation | Name must not contain only whitespace | Windows names a grip after the static before it, which was row 8. Fixed with row 8. |
| 10 | Filter Manager | Text `Static`, the status line | UI Automation | Name must not contain only whitespace | As row 8; the same line of code builds this window's status line. Fixed. |
| 11 | Tag Manager | Text `Static`, the status line | UI Automation | Name must not contain only whitespace | As row 10. Fixed. |
| 12 | Signature Manager | Text `Static`, the status line | UI Automation | Name must not contain only whitespace | As row 10. Fixed. |
| 13 | Contact Manager | Text `Static`, the status line | UI Automation | Name must not contain only whitespace | As row 8, a third copy of the line. Fixed. |
| 14 | Contact Manager | Thumb `ScrollBar`, the resize grip | UI Automation | Name must not contain only whitespace | As row 9. Fixed with row 13. |
| 15 | Edit Event | Edit, the text of the Starts Day spinner, showing 14 | Both | Name of a focusable element must not be null | Ours, recorded. `set_accessible_name` names the spinner, and a Windows spinner is two windows: the up-down arrows carry the name "Starts Day" and the text field a person types in is a separate window with none. Ledger 408, with what it takes. |
| 16 | Edit Event | Edit, the text of the Starts Year spinner, showing 2026 | Both | Name of a focusable element must not be null | As row 15. Ledger 409. |
| 17 | Edit Event | Edit, the text of the Start time Minute spinner, showing 52 | Both | Name of a focusable element must not be null | As row 15. Ledger 410. |
| 18 | Edit Event | Text, the shown value of the Start time AM or PM list, showing PM | UI Automation | Name of a focusable element must not be null | Ours, recorded. The list carries the name; the element showing its current value is Windows' own child of it and has none. Ledger 411. |
| 19 | Edit Event | Edit, the text of the Ends Day spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 412. |
| 20 | Edit Event | Edit, the text of the Ends Year spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 413. |
| 21 | Edit Event | Edit, the text of the End time Minute spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 414. |
| 22 | Edit Event | Text, the shown value of the End time AM or PM list | UI Automation | Name of a focusable element must not be null | As row 18. Ledger 415. |
| 23 | Edit Event | Combo box "Category", 480 by 6 pixels | UI Automation | An element of the given ControlType must support the ExpandCollapse pattern | Unjudged. Every combo box in this window is exposed through the MSAA proxy, because each carries an accessible object of ours, and the proxy offers no ExpandCollapse to any of them; the scanner flagged this one and not the other five. The one difference in the tree is that this box has no child showing a value, and it is six pixels tall because the form runs off the bottom of the runner's 768-pixel screen. Ledger 416 for the rule, 417 for the form being cut off. |
| 24 | When should this message go? | Text, the shown value of the Send on Month list, showing September | UI Automation | Name of a focusable element must not be null | As row 18. Ledger 418. |
| 25 | When should this message go? | Edit, the text of the Send on Day spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 419. |
| 26 | When should this message go? | Edit, the text of the Send on Year spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 420. |
| 27 | When should this message go? | Edit, the text of the Send at Hour spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 421. |
| 28 | When should this message go? | Edit, the text of the Send at Minute spinner | Both | Name of a focusable element must not be null | As row 15. Ledger 422. |
| 29 | When should this message go? | Text, the shown value of the Send at AM or PM list | UI Automation | Name of a focusable element must not be null | As row 18. Ledger 423. |

Twelve rows say "Both": the MSAA walk reported the same text fields as
"editable text" with no name, eight in Edit Event and four in the send-later
window. Two of the MSAA twelve are not in the table: the Start time Hour and
End time Hour text fields. On the UI Automation channel Windows gave each the
text of the static label before it, "Start time" and "End time", which is a
name, so no rule fired, and it is the wrong name: the arrows beside each say
"Start time Hour". No scanner rule can see a name that is present and wrong.
Ledger 424 and 425.

**How the WebView2 rows were attributed.** Not from the class names. In the
artifact's tree, the six elements sit under the pane `Chrome_WidgetWin_1`,
whose parent is a pane of class `wxWindowNR` with automation id `-31900` and
the name "Message body": the wxWebView wrapper, this program's window.
Everything from `Chrome_WidgetWin_1` down reports its provider as
`Unidentified Provider (unmanaged:msedge.dll)`, has no automation id, and has
a framework id of `Chrome` for the views. A search of `src/`, `scripts/` and
the workflows for the six class names finds none of them. So the four
rectangles are objects the Edge runtime builds and this program never touches.
The two names were different: the value was this program's page address, and
the address is what the engine falls back to when the page has no title. That
is why rows 1 and 2 are fixed here and rows 3 to 6 are not.

**What would be filed, and where.** One report at
`MicrosoftEdge/WebView2Feedback`: four Views inside an embedded WebView2
(`EmbeddedBrowserTabRootView`, `TopContainerView`, a `View` under
`MultiContentsView`, `EmbeddedBrowserDownloadView`) report `IsOffscreen =
false` and a null `BoundingRectangle`, which Axe.Windows 2.4.2 reports as an
error on every scan of any application that embeds the control; the run,
the scanner version and the six element dumps from the scan log are the
evidence. Nobody has filed it. Filing is reaching outward and is a person's.

**Three findings about the scan itself.**

1. The run's summary said 26 findings and the log held 29. The counting
   pattern matched "errors were found" and Axe prints "1 error was found",
   so three windows with one finding each were recorded as clean. Fixed in
   the workflow on 2026-09-14 and held by a test that reads the pattern from
   the workflow and holds it to all three sentences the scanner prints.
   Ledger 429, closed.
2. The count takes only the first summary a window prints. A window that
   writes two result files prints two, and the reader window wrote two that
   day, both clean, so nothing was lost this time. Not fixed, because nothing
   in the tree can run that block to prove a sum. Ledger 426.
3. The six module targets are one scan repeated six times. Each walked one
   window of 1,797 elements on the MSAA channel, and each of the six
   `msaa-names.json` files holds the same 91 distinct names, "All Calendars",
   "All Contacts", "All Notes" and "Body, in Markdown" among them in every
   one. Every module's panel is in the window whichever is showing, and the
   walk reads no visibility state, so it cannot tell a hidden panel from the
   one on screen. A nameless control in a hidden panel would be reported six
   times, and a module target proves nothing about its module that
   `mail-module` does not already prove. Ledger 427.

**What the scan did not reach.** The message preview in the main window is a
WebView2 control, and no target's tree held its document: on a fresh profile
there is no message, and the reading window the `reader` target opens is a
rich edit control, not a web view. So nothing about the rendered message,
which is the one place a sender's structure has to survive, was judged by
this run. Ledger 428. The made-up data the targets open on is all
`example.com` addresses and named fixtures; the artifact carries nothing of
anybody's.

## The fifty-five criteria

Columns, left to right: the criterion and its level; whether it applies to a
Windows desktop application, and who decided where the answer is no; whether
Axe.Windows has rules citing it; whether the MSAA walk contributes; whether the
NVDA suite hears anything about it; and what is left for a person.

The "applies" column is a judgement. Nobody has walked fifty-five criteria
against this application, and where the answer is "yes" it is a reading of
the criterion against what a mail client does, not a finding. Where the
answer is "no", a regulation decided it, and the regulation is named, for the
reason given under the table.

| Criterion | Level | Applies to a Windows desktop application? | Axe.Windows (UI Automation) | MSAA walk | NVDA suite | What is left for a person |
|---|---|---|---|---|---|---|
| 1.1.1 Non-text Content | A | yes | no | no | no | Every icon, picture and attachment preview needs a text alternative, and a picture from a sender with no alt text needs to be said to have none rather than hidden. Only a listening pass can tell. |
| 1.2.1 Audio-only and Video-only (Prerecorded) | A | yes, for media the program plays | no | no | no | Audio and video arrive as attachments and are the sender's. The page says when no transcript came with them. The earcons the program itself plays each have a visible or spoken equivalent, which a person confirms. |
| 1.2.2 Captions (Prerecorded) | A | yes, for media the program plays | no | no | no | As 1.2.1: any captions a sender sent are surfaced, and their absence is said. A person checks that the saying happens. |
| 1.2.3 Audio Description or Media Alternative (Prerecorded) | A | yes, for media the program plays | no | no | no | As 1.2.1. |
| 1.2.4 Captions (Live) | AA | yes, if the program ever plays live media; it plays none | no | no | no | Nothing to judge until the program plays something live. |
| 1.2.5 Audio Description (Prerecorded) | AA | yes, for media the program plays | no | no | no | As 1.2.1. |
| 1.3.1 Info and Relationships | A | yes | yes, 61 rules, each about one structural property of one element in the tree | no | no | Whether a rendered message keeps the sender's heading structure, whether every label is tied to its control, and whether a table in a message reads as a table. The rules cannot see any of that. |
| 1.3.2 Meaningful Sequence | A | yes | no | no | no | Reading order in every window, heard with a screen reader. No rule is about order. |
| 1.3.3 Sensory Characteristics | A | yes | no | no | no | Whether any instruction relies on shape, position or colour: "the button on the right". Read by a person. |
| 1.3.4 Orientation | AA | yes | no | no | no | The program does not lock orientation; a person on a tablet display confirms it. |
| 1.3.5 Identify Input Purpose | AA | yes, where a field collects something about the user | no | no | no | The fields in the account window that take a name and an address. A person checks what the platform exposes for them. |
| 1.4.1 Use of Color | A | yes | no | no | no | Unread, flagged and warned messages each carry text or a shape as well as a colour. A person confirms it in both themes. |
| 1.4.2 Audio Control | A | yes | no | no | no | Reading aloud can be muted with one key, and sounds have a control. A person confirms both work while something is playing. |
| 1.4.3 Contrast (Minimum) | AA | yes | no: the v2.4.2 rule list has no rule that mentions contrast or colour | no | no | Text against its background in both themes, measured with a contrast tool. Nothing here measures it. |
| 1.4.4 Resize Text | AA | yes | no | no | no | The reading size keys and the Windows text scaling, at 200%, with nothing cut off. |
| 1.4.5 Images of Text | AA | yes | no | no | no | The program draws no text as pictures; a sender's pictures are theirs. A person checks the program's own screens. |
| 1.4.10 Reflow | AA | yes | no | no | no | Every window at 400% scaling, or at a narrow width, without scrolling in two directions. |
| 1.4.11 Non-text Contrast | AA | yes | no | no | no | The focus indicator, control borders and icons at 3:1 against their surroundings, in both themes. |
| 1.4.12 Text Spacing | AA | yes, for the rendered message | no | no | no | Whether a message rendered from HTML survives wider line and letter spacing without losing content. |
| 1.4.13 Content on Hover or Focus | AA | yes | no | no | no | Tooltips and anything that appears on hover or focus can be dismissed and do not cover what they describe. |
| 2.1.1 Keyboard | A | yes | yes, 9 rules, all about whether an element reports itself as keyboard focusable given its control type | no | no | Whether every function can in fact be reached and used by keyboard, in every window. A rule that an element says it is focusable does not press a key. |
| 2.1.2 No Keyboard Trap | A | yes | no | no | no | Tab into and out of every control, including the rendered message, which is the likeliest trap because it is a web view inside a native window. |
| 2.1.4 Character Key Shortcuts | A | yes | no | no | no | Whether any single-character key acts when focus is elsewhere. Space reads the item under the cursor and acts only there; a person confirms nothing else fires. |
| 2.2.1 Timing Adjustable | A | yes | no | no | no | Nothing in the program is meant to time out. A person confirms it, including the reminder window, whose tone repeats and stops on its own. |
| 2.2.2 Pause, Stop, Hide | A | yes | no | no | no | A syncing mailbox updates its list while somebody reads it. A person checks that the update does not take the cursor or re-read the list. |
| 2.3.1 Three Flashes or Below Threshold | A | yes | no | no | no | Nothing is designed to flash. A person confirms nothing does. |
| 2.4.1 Bypass Blocks | A | no: Section 508 and EN 301 549 both say it does not apply to non-web software | no | no | no | Nothing, for a conformance claim under either regulation. A person may still ask whether a long window can be skipped through. |
| 2.4.2 Page Titled | A | EN 301 549 says it does not apply to non-web software; Section 508 keeps it | no | no | no | Every window has a title that says which module or dialog it is. A person reads each title as a screen reader announces it. |
| 2.4.3 Focus Order | A | yes | no | no | no | Tab order in every window, heard. No rule is about order. |
| 2.4.4 Link Purpose (In Context) | A | yes, for the rendered message and the program's own links | no | no | no | A sender's link text is theirs and is kept intact through sanitising. A person checks the program's own links and that link text survives rendering. |
| 2.4.5 Multiple Ways | AA | no: Section 508 and EN 301 549 both say it does not apply to non-web software | no | no | no | Nothing, for a conformance claim under either regulation. |
| 2.4.6 Headings and Labels | AA | yes | no | no | no | Whether each label and heading describes its topic. The MSAA walk says a control has a name; it does not say the name describes anything. |
| 2.4.7 Focus Visible | AA | yes | no | no | no | A visible focus indicator on every control, in both themes, seen. |
| 2.4.11 Focus Not Obscured (Minimum) | AA | yes | no | no | no | Whether anything, a status line or a reminder window, can cover the focused control. Seen. |
| 2.5.1 Pointer Gestures | A | yes | no | no | no | Nothing uses a multipoint or path gesture. A person confirms it. |
| 2.5.2 Pointer Cancellation | A | yes | no | no | no | Native controls act on release rather than press. A person confirms the program's own handlers do too. |
| 2.5.3 Label in Name | A | yes | no | no | no | Whether the accessible name of each control contains its visible label. The MSAA walk reports names; it does not compare them with what is drawn. |
| 2.5.4 Motion Actuation | A | yes, if anything used device motion; nothing does | no | no | no | Nothing to judge until something uses motion. |
| 2.5.7 Dragging Movements | AA | yes | no | no | no | Every drag, such as moving a message to a folder, has a keyboard way to do the same thing. A person confirms each one. |
| 2.5.8 Target Size (Minimum) | AA | yes | no: the 25-pixel-area rule Axe.Windows runs cites Section 508 502.3.1 and is not this criterion's 24 by 24 | no | no | The size of every control somebody clicks, measured. |
| 3.1.1 Language of Page | A | yes | no | no | no | Whether each window exposes its language to the platform. The interface is English only today, and a rendered message carries the sender's language. |
| 3.1.2 Language of Parts | AA | EN 301 549 says it does not apply to non-web software; Section 508 keeps it | no | no | no | A day name in the machine's language inside an English sentence is exactly this criterion's case, and nobody has heard how a screen reader treats it. |
| 3.2.1 On Focus | A | yes | no | no | no | Whether landing on any control changes the window or moves focus elsewhere. Heard. |
| 3.2.2 On Input | A | yes | no | no | no | Whether changing a setting acts before OK is pressed, and whether typing in a field opens or closes anything. Heard. |
| 3.2.3 Consistent Navigation | AA | no: Section 508 and EN 301 549 both say it does not apply to non-web software | no | no | no | Nothing, for a conformance claim under either regulation. The modules share one layout on purpose; a person confirms it. |
| 3.2.4 Consistent Identification | AA | no: Section 508 and EN 301 549 both say it does not apply to non-web software | no | no | no | Nothing, for a conformance claim under either regulation. |
| 3.2.6 Consistent Help | A | yes | no | no | no | Help is in the same place in every module. A person confirms it from each. |
| 3.3.1 Error Identification | A | yes | no | no | yes, two tests hear an error sentence: "Signing in failed", and the sentence Delete shows when nothing is selected | Every other error sentence in the program, heard. |
| 3.3.2 Labels or Instructions | A | yes | no | no | no | Whether each field says what it wants. The MSAA walk says a control has a name; it does not say the name instructs. |
| 3.3.3 Error Suggestion | AA | yes | no | no | no | Whether each error says what to do next, which is a project rule and not yet a heard fact. |
| 3.3.4 Error Prevention (Legal, Financial, Data) | AA | yes | no | no | no | Deleting asks first and names the thing; sending can be previewed and held in the outbox. A person confirms each path asks. |
| 3.3.7 Redundant Entry | A | yes | no | no | no | Whether setting up an account asks for anything twice. Walked by a person. |
| 3.3.8 Accessible Authentication (Minimum) | AA | yes | no | no | no | Signing in goes through the provider's own page or a stored password; the program itself sets no puzzle. A person confirms the stored path works without retyping. |
| 4.1.2 Name, Role, Value | A | yes | yes, 9 rules, about which control patterns a button supports and whether focusable siblings have different names | yes, the Name part only: every operated control has a non-empty name on the channel NVDA reads | no; the skipped test would hear a radio button as both focused and checked | Whether the name is the right one, whether the role is right, and whether value and state reach the screen reader, in every window. Fourteen more Axe.Windows rules test the name and seven test the localized control type, and all twenty-one cite Section 508 rather than this criterion. |
| 4.1.3 Status Messages | AA | yes | no | no | yes, three tests hear three announcements made without moving focus | Every other announcement the program makes: whether it is spoken, whether it is spoken once, and whether a syncing mailbox floods it. |

Fifty-five rows. Three say "yes" under Axe.Windows, one says "yes" under the
MSAA walk, two say "yes" under the NVDA suite, and every row leaves something
for a person.

## Where a regulation decided, and why WCAG2ICT is not the name

Six rows say a criterion does not apply to non-web software. WCAG2ICT, the W3C
Group Note of 11 December 2025 on applying WCAG 2.2 to non-web documents and
software, is the document people usually cite for that, and it says of itself:

> This document does not seek to determine which WCAG 2 provisions
> (principles, guidelines, or success criteria) should or should not apply to
> non-web documents and software, but rather, if applied, how they would
> apply.

What it records is what two regulations decided, and they did not decide the
same thing:

| Criteria | Do not apply to non-web software, according to |
|---|---|
| 2.4.1 Bypass Blocks, 2.4.5 Multiple Ways, 3.2.3 Consistent Navigation, 3.2.4 Consistent Identification | Section 508 in the United States and EN 301 549 in Europe |
| 2.4.2 Page Titled, 3.1.2 Language of Parts | EN 301 549 only |

So the table names the regulation, not the note. Wixen Mail makes no Section
508 conformance claim and no EN 301 549 conformance claim; the
[accessibility page](accessibility.md) says so. The six rows are here so that
somebody reading the table knows why those criteria are not on the list of
things a person has to walk for a WCAG claim, and knows that under Section 508
two of them would be.

## What has not happened

- One scan has run against the thirty-one windows on both channels, on
  2026-09-14. The nine fixes in the table above have not been confirmed by a
  run since; the next push to `main` is what confirms them.
- Nobody has walked the fifty-five criteria against this application. The
  table is a reading of each criterion against what a mail client does.
- Most of the application has not had a manual pass with a screen reader.
  Three sentences in three windows have been heard by the NVDA suite. Nothing
  else on any row of the table has been heard.
- No Section 508 or EN 301 549 conformance claim is made.
- The rule list can change with a new Axe.Windows release. The pin means it
  cannot change without a commit here, and that commit is the moment to read
  the new release's rule file and correct the numbers on this page.

## How this page is held to the code

The three criteria under "Axe.Windows", and the one under "MSAA walk", are
also written in `src/presentation/what_the_scans_can_judge.rs`. A test reads
this page's table and requires the "yes" cells in those two columns to name
exactly those criteria, in both directions: a fourth criterion marked yes here
fails it, and one of the three marked no fails it. A companion test hands the
same reading a table with a planted wrong row and requires it to be reported,
so that the reading cannot be narrowed until it sees nothing. The other
fifty-two rows are a person's judgement and no test holds them.

The counts in the table are not held by a test, because the rule file lives
outside this repository. They were taken from `axe-windows-rules-2.4.2.md` on
2026-09-14 and are only as current as the pin.

# Accessibility

This page says what Wixen Mail does for accessibility, organised by who each
part is for, and says plainly where the work is unfinished. Structure being
present is not the same as the experience being good, so where something has
not been confirmed by a real run with the assistive technology it is for,
that is written down too.

## Standards

Wixen Mail targets **WCAG 2.2 Level AA**, applied to a Windows desktop
application rather than a web page. Automated scanning catches roughly half
of what WCAG asks for. The rest needs a real person using real assistive
technology, which is why this page distinguishes what has been checked by a
scanner from what has been confirmed by a listening pass.

There is no Section 508 conformance claim here, because nobody has done the
work of checking against it.

## Blind and low-vision people using a screen reader

Windows has two accessibility channels, and getting both right matters more
than it looks like it should. UI Automation (UIA) is what Narrator reads.
MSAA, through `IAccessible`, is what NVDA reads for native controls, and it
is the only place this application's own `set_accessible_name` calls write
to. For a native edit box or button, Windows supplies its own UI Automation
provider that shadows the MSAA object underneath, so a scan of the UIA tree
alone can report the system's own name for a control and never notice that
the name this code set was never read at all. Wixen Mail is checked against
both, in CI, on every pull request: an Axe.Windows scan of the UIA tree, and
a script that walks the MSAA tree naming any control with no MSAA name.

Screen reader support:

- **NVDA** is the primary target, and the one a small automated suite drives
  for real in CI. That workflow launches a real copy of NVDA against the
  running application, presses specific keys, and checks what NVDA actually
  said aloud, which a structural scan cannot do. It is deliberately narrow,
  though: it covers only the interactions its own tests touch, runs
  non-blocking, and says nothing about any control, dialog, or sentence
  those tests do not reach. Most of the application has not had a full
  manual pass with NVDA, and that is the honest state of it.
- **Windows Narrator** is spot-checked, not comprehensively verified.
- **JAWS** has not been run against this application. Nothing here claims
  JAWS support until that has actually happened.

Braille follows speech rather than needing its own path: announcements go
through `UiaRaiseNotificationEvent`, which is how Windows both speaks a
notification and sends it to a connected braille display through whichever
screen reader is running. A braille display should show the same
announcements a sighted user would hear spoken, without a separate setting.

Every list, tree, and text field is meant to carry a correct accessible
Name, Role, Value, and State, and focus is meant to be managed so it is
never lost when a panel or dialog changes. Long documents, a message opened
to read or a whole email conversation shown as one page, keep the sender's
real heading structure, so a screen reader's heading navigation moves
between them the way it would in any well-structured document. [Keyboard
shortcuts](KEYBOARD_SHORTCUTS.md) is the complete, current list of every key
in the application, checked automatically against the real menus so it
cannot drift the way a second copy of the same list would. This page does
not repeat it.

## Low vision and colour

- **Contrast.** Text is checked against 4.5:1, and larger text and UI
  components against 3:1, in both the light and dark themes.
- **Never colour alone.** Anything meaningful shown with colour, an unread
  message, a flagged one, a safety warning, also carries text or a distinct
  shape, so colour blindness or a monochrome display does not lose the
  information.
- **Themes.** Settings offers Light, Dark, and a High Contrast choice that
  hands the colours back to Windows entirely rather than trying to imitate
  them. When Windows high contrast is switched on, Wixen Mail paints
  nothing of its own: that is deliberate, because you chose those colours
  and an application painting over them has taken away the reason you set
  them. A theme reaches the sidebar and content area of every module, the
  window a message opens into for reading, and the window that shows a
  conversation as headings; the rest of the window follows Windows. A
  change takes effect as soon as Settings closes with OK, with nothing to
  restart.
- **Zoom and text size.** `Ctrl+Plus` and `Ctrl+Minus` change the reading
  size, and the application otherwise follows the font size and display
  scaling Windows is already set to.
- **Focus.** A visible focus indicator is meant to be present on every
  interactive control.

## Physical and motor access

Everything is meant to be reachable and operable by keyboard alone, with no
mouse-only or drag-only interaction anywhere. Accelerators follow standard
Windows conventions, `Alt` and an underlined letter for a control in view,
a menu accelerator for a command from anywhere, and
[keyboard shortcuts](KEYBOARD_SHORTCUTS.md) is updated in the same commit
that changes one, which a test checks both ways: every key a menu binds is
named in that document, and the document names no key the code has never
heard of.

Reading an item under the cursor uses `Space`, once for a short summary and
again for the whole thing, with no timing window between the two presses.
A timing window that judged how fast you pressed twice would be a timing
trap, and would lock out anyone who types slowly; the second press does the
second thing however long it takes.

There is no keyboard shortcut customisation yet. Shortcuts are currently
fixed, and this page and [Keyboard shortcuts](KEYBOARD_SHORTCUTS.md) agree
on that rather than one of them promising a feature the other says is not
built.

Nothing in the application times out on its own or interrupts you with an
automatic action.

## Learning and cognitive access

Labels, messages, and errors are written in plain language, and an error
says what happened, why, and what to do next rather than a code alone.
Navigation is the same shape in every module: mail, contacts, calendar,
reminders, tasks, and notes all use the same sidebar, list, and preview
arrangement, and the same `Space`-to-read pattern works the same way in
every one of them.

A destructive action, deleting a task, a contact, a whole message, always
asks first and names the specific thing it is about to remove, so answering
too quickly does not lose the wrong item. Enter answers No on that question
rather than Yes, on the theory that pressing the key you already know is
safer than pressing the one that confirms.

## Hearing and non-speech audio

Every audio cue has a visible or spoken equivalent; nothing here signals
something by sound alone. Where a message carries audio or video as an
attachment or embedded media, Wixen Mail surfaces any captions or
transcript the sender provided, and says plainly when none exists rather
than presenting the media as though it were accessible when it is not.

**Earcons and sound schemes.** Short audio cues, an earcon, mark events like
a message arriving, a message having an attachment, or an account needing
attention, as a second channel alongside the spoken announcement and the
status line. They are meant to be distinguishable from each other and
bounded, so a syncing mailbox cannot flood you with sound the way it must
not flood you with speech. Under Settings, the Feedback tab, a sound scheme
picks which set of sounds plays for these events, starting with a
built-in, synthesised scheme. A sound scheme can be a real audio pack
instead: choose Import sound scheme to bring in a `.zip` someone else
packaged, and Delete sound scheme to remove one you no longer want, which
stays disabled while only the built-in scheme is present, since there must
always be one. Every feedback channel, speech, earcons, braille, and the
status line, can be turned on or off independently from the same tab.

## Vestibular and photosensitivity

Wixen Mail honours the Windows setting for reduced motion, and nothing in
the application flashes more than three times a second.

## The message preview and untrusted content

The preview pane renders untrusted HTML in a WebView, which is a browser
embedded in the window. Once focus is inside it, the browser consumes
`Esc`, `F6`, and every menu accelerator, so the preview never takes focus:
`F6` moves between the sidebar and whichever list is open, and stops there.
To read a message, `Space` on the message list reads it without leaving the
list, and that path works with whatever screen reader you already have
configured. A full, readable, focusable view of a message body outside the
preview is the proper long-term answer to this and is not built yet.

The HTML itself is sanitised before it is shown, which is a security
measure, and the sender's heading structure and link text are kept intact
through that sanitising, which is an accessibility one. Neither excuses
skipping the other.

## Testing

Automated checks run on every pull request: an Axe.Windows scan of the UI
Automation tree, a script walking the MSAA tree for missing names, and the
small NVDA-driven suite described above. None of the three is a substitute
for a person using the finished application with the assistive technology
they actually rely on, and that fuller pass has not been done yet for most
of the application. [What is built and what is
not](IMPLEMENTATION_STATUS.md) says exactly how far real screen reader
testing has gone as of its own last update.

### Reporting an accessibility problem

1. Open an issue on the GitHub repository.
2. Tag it with the `accessibility` label.
3. Say which assistive technology you were using, and its version, your
   Windows version, the steps that led to the problem, and what you
   expected to happen instead.

[Start a discussion](https://github.com/PratikP1/Wixen-Mail/discussions)
instead if it is a question rather than a fault.

There is no email address yet. One will be listed here when there is one to
list, rather than a placeholder that bounces.

## External references

- [NVDA](https://www.nvaccess.org/)
- [JAWS](https://www.freedomscientific.com/products/software/jaws/)
- [Windows Narrator](https://support.microsoft.com/en-us/windows/complete-guide-to-narrator-e4397a0d-ef4f-b386-d8ae-c172f109bdb1)
- [WCAG 2.2](https://www.w3.org/WAI/WCAG22/quickref/)

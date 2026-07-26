# Principles

Wixen Mail is judged by four questions. Wixen Terminal asks them, Wixen Chat adopts
them, and they apply here unchanged in form and different in answer. Ask them of
every change.

## What is it for?

Making correspondence and personal information legible to people who cannot see it.

Mail is where work, money, health, and family arrive. Around it sit the calendar
that says where you must be, the contacts that say who people are, and the tasks,
notes, and reminders that hold the rest. The mainstream clients are web apps or
Electron wrappers, where a screen reader user tabs through unlabeled regions, loses
their place when a folder refreshes, and meets an HTML message body as a wall of
undifferentiated text.

Wixen Mail treats all of it as structured data. Messages, senders, folders, threads,
read state, attachments, events, contacts, tasks, notes, and reminders are declared
to the platform accessibility API through native controls, with a name, a role, and
a state each. What happens outside the focused widget reaches the user through a
deliberate announcement channel rather than by luck. A blind user should work
through a full inbox at their own pace, not reconstruct it afterward.

## What does it strengthen?

The independence of its users. Reading your own mail, accepting your own meeting
invitations, and finding your own contact details, without a sighted intermediary
and without a second-class interface that technically works.

The principle that the application declares its meaning. The client says "message
from Ada Lovelace, unread, has attachments" as structured fact. The screen reader is
never left to infer it from pixels or from scraping a rendered DOM.

Open protocols. IMAP, SMTP, POP3, CalDAV, and iCalendar are open, and an open
protocol deserves a client whose accessibility matches that openness. Provider APIs
are supported where they add something, not as the only door in.

## What does it replace?

For its user, Outlook, Thunderbird, and webmail: usable with a screen reader in the
technical sense, painful in the practical one.

It does not replace the screen reader. NVDA, JAWS, and Narrator own review cursors,
verbosity, and speech. Wixen Mail owns what the application exposes and announces.

It does not replace the mail server, the calendar service, or the provider. It is a
client.

It does not replace Wixen Terminal or Terminal Access. Those serve the command line;
this serves the inbox. Each tool does one thing excellently.

## What does it allow to be done poorly?

This question generates the guardrails, because every strength here has a failure
mode that looks like success. Each item below has already happened in this codebase.

**Accessibility calls that are not accessibility calls.** Sixteen widgets were given
a name with `set_name()`. It compiled, passed the whole suite, read correctly in
review, and did nothing at all: `wxWindow::SetName` is an internal wxWidgets
identifier that never reaches the accessibility tree. Only a scan of the running app
caught it. An accessibility change is not done because it looks done.

**Structure present, experience good.** Native widgets hand you a tree for free, and
a tree is not an experience. Focus that jumps when a folder reloads, a message list
that re-announces itself, a detail pane that updates silently: each passes automated
checks. Automated scanning covers roughly half of WCAG. Only a real NVDA run proves
the rest.

**Implemented but never wired.** All eight PIM update variants were handled in the UI
and sent by nothing. Calendar, contacts, tasks, notes, and reminders each had
storage, a sync client, a manager, and a panel, and rendered empty in every running
build. Everything compiled. Every test passed.

**Stubs that look finished.** The note editor filled its title and body with
"Note 1" and "(Note content loaded here)" on every selection. That is worse than an
empty pane, because an empty pane is honest.

**Breadth over excellence.** Six modules shipped at once with one of them working.
Before a new subsystem, ask whether it is wired, exercised end to end, and raises the
bar for the whole, or whether it only adds surface.

**Announcement flooding.** A client that can speak every change can bury a user under
a syncing mailbox until they turn announcements off and miss what mattered.
Announcements need priority, coalescing, and bounds, and the bounds need testing.

**Privacy through speech.** A client that reads message content aloud will read
private mail to a room if the user is presenting or away from headphones. Content
announcement needs controls and a fast mute.

**Hostile HTML.** Message bodies are untrusted input from strangers. Sanitizing them
is a security matter and preserving their heading structure and link text is an
accessibility matter, and neither excuses dropping the other.

**Green checks that check nothing.** CI ran four jobs and had been failing for months
while the badge kept being ignored. An accessibility scan reported success while its
scan step was erroring out, because the job was non-blocking and nobody read the log.
A check nobody reads is worse than no check, because it buys false confidence.

**Automation that decides for you.** A push to `main` cut two releases nobody asked
for, tagged them, and promoted an alpha to beta. Anything that publishes should be
triggered on purpose.

**Absorbing upstream failures.** When a sender ships an image with no alt text, a
provider returns broken MIME, or a dependency exposes no accessible name, say so
plainly rather than papering over it. A better ecosystem needs the gap visible.

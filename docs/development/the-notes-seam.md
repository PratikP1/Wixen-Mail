# What a notes backend has to answer

This is the contract behind `application::notes_backend`. It is written as
requirements on **any** backend, not as a description of the one that happens to
be built first, and it is meant to be found wrong.

Two implementations exist and they disagree with each other on purpose.
`service::caldav_journal` is real, ships, and speaks to a calendar server's
journal entries. The second lives in
`tests/the_notes_seam_takes_a_second_kind_of_backend.rs`, is shaped from what
Microsoft's reference says a OneNote page does, and exists to find out which
sentences below were about backends and which were about CalDAV. It found four.
They are listed near the end of this document, hardest first, and phase 5.2 is
expected to add to that list when it replaces the second one with a real client.

`NotesService` names four operations. `NotesBackend` answers which backend an
account uses, and an account with a calendar on a calendar server answers
`CalDavJournal`.

## What is already decided elsewhere, and is not re-decided here

A backend does not get its own conflict model, its own deletion rule, or its own
idea of what a note is. Each of those already exists for contacts, calendar
items and tasks, and a second copy of any of them disagrees with the first the
day either one changes.

| Question | Where it is answered | What a backend does |
|---|---|---|
| Which copy is kept when both moved | `application::conflict_choice` | Reports that both moved, and stops |
| What happens to a deletion made here | `application::deletions` | Reports the removal; the record outlives the row |
| What a note's body is | `data::message_cache::notes` | Carries it unchanged |
| Whether this program may change anything | `application::allowed` | Is not asked at all until the answer is yes |

## Identity: how a backend names a note

**A backend names a note in its own words, and this program never takes those
words apart.**

`ANoteThere` carries two things: what the backend calls the note, and the
version marker the backend last gave for it. Both belong to the pairing of a
note and a backend rather than to the note.

That shape is not new. `data::message_cache::ProviderIdentity` already takes it
for a contact, and its own comment records what went wrong before it did:

> Per address book and not per contact, because one push can be accepted and
> another refused in the same run. Kept on the contact instead, a failure at one
> address book either lost the change at the other or resent it to both for
> ever, depending on which way the flag was cleared.

The same is true of a note the moment two backends can hold one, and it is true
already of a note held by one backend and also kept here.

**Requirements on any backend:**

1. The name it gives is opaque. Nothing in this program may parse it, split it,
   or build one.
2. A name it has never given is not a name. A note this program has that the
   backend does not know about is a note to create, not a note to look up.
3. **A version marker is required of any backend this program writes to.** This
   sentence used to read "a version marker is optional. A backend that gives
   none is a backend whose copy is always treated as having moved", and it was
   the largest thing `05.1-04` found wrong. The reading is right for the read
   half and it is the exact opposite of what the same absence means to the push
   half, where no marker means nothing is known to have moved at the backend, so
   this computer's copy goes over whatever is there. Two readings of one absence,
   in one sync. A backend that gives no marker therefore destroys a change made
   at the other end and says nothing about it, which was measured before this
   requirement was written.

   No code change reconciles the two readings. Telling them apart needs the copy
   the backend last had, and PIM-08 forbids keeping one: the stored form does not
   change to accommodate a backend. So the requirement moves to the backend, and
   a service with no opaque token uses whatever it does have. A OneNote page's
   `lastModifiedDateTime` is a marker under requirement 4's reading, which is
   why that requirement is written the way it is.
4. **A version marker is not promised to be opaque, and is not promised to
   change only when the content does. It is promised to change whenever the
   content does.** A CalDAV ETag is an opaque token the server changes when the
   resource changes. A OneNote page has no ETag and no `eTag` property at all;
   what it has is `lastModifiedDateTime`, which the service writes and which
   moves for reasons the content did not cause. Anything here that compares
   markers must be a comparison for equality and nothing else. Ordering two
   markers, parsing one as a date, or treating a changed marker as proof that
   the text changed are all CalDAV readings.

   The last sentence of the first paragraph is new, and it is the direction this
   document had not written down. A marker that moves when the content did not
   costs a fetch nobody needed. A marker that stands still when the content did
   move costs somebody their note: the sync reports it unchanged and the change
   never arrives. An ETag cannot do that. A clock reading can, because a clock
   has a resolution and two changes inside one of its ticks carry one reading.

   What that means for a backend built on a timestamp, and OneNote is one: a
   write this program makes and an edit somebody makes at the service inside the
   same tick are one marker, and the second is lost. Nothing in this repository
   can say how wide that window really is. It is entry 251 in
   `.planning/WINDOWS.md`.

**What a word this build does not recognise means.** A stored backend name this
version has never heard of becomes `NotesBackend::Other`, which follows
`AddressBook::Other` for the reason that type's own comment gives: a word this
code does not recognise is still somebody else's answer, and forgetting it
rewrites their row on the next save. A note whose backend is `Other` is read,
edited and kept exactly as any other note. It is not synced, because a backend
this build cannot name is a backend it has no client for, and offering a sync
that cannot happen is the failure the note folder menu exists to avoid.

## Conflict: what happens when both copies moved

**A backend must not resolve a conflict itself.**

That is the whole requirement, and it is worth stating in as many words because
it is the one every network client is tempted to break. A backend that decides
which copy wins has written a second conflict model, and two conflict models
disagree the first time either is changed. `03-RESEARCH.md` warned about this
and SCALE-06 was corrected to avoid it.

The one model is `application::conflict_choice`. It holds both copies, names the
fields that differ, asks somebody which to keep, and says what choosing each one
calls for. It has no window, no sync and no database in it, which is what lets
the sentences somebody hears be tested without a running window.

**Requirements on any backend:**

1. When the backend's copy has moved since this computer last looked, and this
   computer also holds a change nobody has sent, the backend reports
   `WhatTheBackendSaid::ItMovedFirst` and writes nothing.
2. It reports the marker the other copy carries now, so the hold that is written
   next records it. Without that, the next sync finds the same disagreement and
   asks the same question again, and a choice that has to be made every sync is
   not a choice.
3. Who wins is decided by `contacts_sync::whose_copy_wins`, which compares
   version markers rather than clocks, and by nothing else. A backend does not
   get to add a fourth answer to that enum's four.

**And one requirement on whatever drives a backend, which is where this was
found wrong.** A clash is reported by the push, and the push does not always
run. The setting can refuse it, nobody may be signed in, the request can fail.
In each of those the change stays here and stays marked as waiting, so that
fixing the cause still sends it. The read that follows in the same sync must not
write the backend's copy over that change. It used to, which made the promise
false: the sync counted one change as waiting and destroyed it a moment later,
and the count was the only thing anybody was told.

So the read holds both copies through `conflict_choice` when the copy here has
an unsent change and what arrived says something else. Six thousand eight hundred
and two library tests passed with that guard removed. It is
`tests/the_notes_seam_takes_a_second_kind_of_backend.rs` that catches it, and it
catches it for both backends, so this was never a second backend's problem.

**What a notes backend adds to the conflict model, and it is one line.**
`TheOtherCopy` has two variants today, `AnAddressBook` and `ACalendar`, each
carrying the words used in a sentence. Its doc comment explains that it is a
parameter rather than a second set of sentences, so that one set of words with a
hole in it cannot drift from itself. A notes backend adds a third variant and
nothing else. It does not add a second `BothCopies`, a second
`what_is_being_asked`, or a second set of buttons.

## Removal: what happens when a backend is taken off an account

**The note stays. The identity it carried for that backend stops meaning
anything.**

A note is a first-class local document and always was. It is stored as the exact
bytes somebody typed, it is read for whatever structure is in it, and none of
that depends on a backend existing. So taking a backend off an account is not a
reason to lose anything.

**Requirements:**

1. The note is not deleted. Not from the list, not from the database, not
   "cleaned up" on the next sync.
2. The identity that backend gave is not reused. Sending the note to a different
   backend under the old name is the failure `AddressBook::Other`'s comment
   describes for contacts: forgetting a name silently joins two of them
   together. Two notes filed under one name is the same accident.
3. The account's answer becomes `NotesBackend::ThisComputer`, and the settings
   screen says so, in the same words it says for an account that never had one.
   There is no third state and no "used to sync" sentence.

## The deletion record, which the requirement does not name and the seam needs

**A backend needs a record of deletions that outlives the row.** This section
was written when there was no such table for notes. `05.1-03` built
`deleted_notes` and the requirements below are what it was built to.

`application::deletions` states one rule for everything this computer deletes: a
note of the deletion outlives the row, because a deleted row cannot carry a "not
yet sent" flag, and every read asks the notes before writing anything down.

Contacts, calendar events, tasks and now notes each have one:

```bash
grep -n "CREATE TABLE IF NOT EXISTS deleted" src/data/message_cache/mod.rs
```

Without it the first backend rediscovers the exact bug `deletions.rs` was
written about, in its own words:

> The note was dropped the moment the provider took the deletion. The push runs
> before the read in the same sync, so by the time the read arrived there was
> nothing left saying anybody had been deleted, and a provider still naming the
> thing was answered by writing it straight back down. It came back on the
> screen, under the provider's own identifier, with nothing left to say it had
> ever been deleted.

Somebody deletes a note, and it is back the next time they look.

**Requirements:**

1. A deletion made here is written down before it is sent, and the record
   carries what the backend called the note, which is why
   `WhatTheBackendSaid::Done` carries an identity on the way out of a removal
   too.
2. The record is kept until the backend has taken the deletion, and for
   `HOW_LONG_A_DELETION_IS_REMEMBERED` after that. The two lives are different
   things: a record no backend has taken is work still owed, and a record it has
   taken is a memory that stops a read writing the thing back down.
3. Every read asks the records before it writes anything down, and skips
   whatever they name.

## Writing: the operation some backends do not have

**A backend is asked for an end state, not for an edit.**

`leave_a_note_saying` says what the note should say when the call is over. How a
backend gets there is its own business. That wording is deliberate, and the
reason is the second thing most likely to be found wrong.

A CalDAV journal entry is one document that can be replaced whole. **A OneNote
page's body cannot be replaced.** Microsoft's supported-actions table gives
`body` as append yes, replace no, insert no. Making a page say something else
means deleting its elements one at a time by the ids the service generated and
appending new ones, or deleting the page and making another, which changes its
identity, its position and its links. On top of that, most replaces need the
id the service generated rather than one you set, and the same page says those
ids "might change after a page update", so every write is preceded by a read.

So "write the whole document" is not an operation every backend has, and a
contract that asked for one would be a CalDAV contract wearing a general name.
What this asks for instead is the end state, plus an honest answer about what
the note is called afterwards.

**Requirements:**

1. A backend may reach the asked-for end state by any means, including removing
   its copy and making another.
2. If it does, `WhatTheBackendSaid::Done` carries the new identity, and the
   caller writes that down. A caller left holding the old name is a caller whose
   next write creates a second note.
3. A backend that cannot reach the end state says which of the named things
   happened. It does not return a string for somebody to read out: text a crate
   wrote for a developer is not text to speak to somebody whose notes did not
   sync.
4. A backend that reached the end state but could not keep the bytes it was
   given says what it kept, in `Done`'s `what_it_could_keep`. The next section
   is about that.

## Bytes: what a backend must hand back

**A backend that cannot hand back the bytes it was given says so at the moment
it takes them.**

PIM-04 says a note's stored form is the Markdown source and that a note edited
here and read back is byte-identical when nothing changed. Two formats this
program already speaks break that, and neither is unusual.

A calendar journal document has one escape for a line break and no way to write
a carriage return inside a value at all. A body typed on a Windows machine goes
out with CRLF and comes back with LF. `service::note_document`'s header carries
the measurement and the section of RFC 5545 it comes from.

A OneNote page is an HTML document, and HTML collapses a run of whitespace to
one space and cannot hold one at the end of a line. So this note:

```
- Live is brown
  - Older cable: red
```

comes back as two items at the same level, because the two spaces that made the
second one a child of the first are gone.

**Why the backend has to be the one to say it.** Whatever is driving a backend
sees only that the copy there and the copy here differ. It cannot tell a
backend's own reshaping of what it was handed from a change somebody made at the
other end, and the two want opposite answers: one is a limit of a format and
nobody needs asking about it, the other is somebody's edit and must not be
thrown away. Only the backend knows which, and only in the moment it writes.

**What it costs to leave it unsaid**, which is the state before this section
existed. The two copies differ from the moment of the first push and nothing
says so. Every later sync compares markers, finds them equal, and reports the
note unchanged, so the difference is invisible for as long as nothing at the
backend moves. The first thing that does move a marker, which the section on
markers says a backend is allowed to do for reasons the content did not cause,
brings the backend's version of somebody's note down over theirs with no
question asked and no sentence anywhere.

**Requirements on any backend:**

1. A backend that could not keep what it was handed reports what it did keep.
   `None` is a promise that the bytes survived, not a way of saying nothing is
   known.
2. It answers from its own copy rather than from a rule about itself. A backend
   that works out the answer from what it believes it does goes on claiming a
   byte survived after the day it stops surviving.
3. It never resolves the difference. Reporting is the whole of a backend's part.
   What is written down here, and what somebody is told, is decided in
   `application::notes_sync` for every backend at once.

**What the sync does with it, so that a backend can rely on it.** The copy here
becomes the copy the backend kept, at the moment of the push, and the sync
counts it and says so. The loss happens once, while somebody is watching the
sync they asked for, instead of arriving weeks later as a note that changed for
no reason they can see. It is the only moment at which the two cases can be told
apart.

**A backend that cannot round-trip is allowed, and this is the price.** The copy
here is not what was typed once a note has been to such a backend. That is a
real cost and it is why the first requirement is a report rather than a refusal:
refusing the backend would mean refusing OneNote, which is a service people use,
and pretending the bytes survived would mean losing them quietly. Between saying
so once and losing them quietly, saying so once is better.

## What each backend really hands back, measured

Two backends now exist on paper and one of them ships, and neither hands back
what it was given. They fail differently, and folding them together would hide
the choice rather than inform it.

**A calendar journal loses one class of byte and keeps every other.** A body
typed on a Windows machine goes out with CRLF and comes back with LF, because
RFC 5545 section 3.3.11 gives one escape for a line break and no way to write a
carriage return inside a value. A trailing space, a backslash, a comma, an
emoji and a line that looks like a property name all survive.
`service::note_document`'s header carries that measurement, and `05.1-03` found
it by building the thing.

**A OneNote page loses the form and keeps most of the words.** It is not a
wider version of the calendar's problem. A page holds HTML, so what crosses is
a structure rather than a source, and the Markdown that comes back is rendered
afresh from that structure. Nothing round-trips because nothing is stored.

### What a note kept in OneNote comes back as

Every row below is run by
`service::onenote_page::tests::test_every_row_of_the_fidelity_table_is_true`,
which reads this table out of this file and puts each left-hand cell through
the pair. A row that stops being true fails the build. The table cannot say
something the tests do not.

In the cells, `\n` is a line break, `\|` a vertical bar, `\g` a backtick and
`\e` nothing at all, because none of those can be written inside a table cell.

**The middle step is a model of the service and not the service.** Nobody here
has a OneNote tenant. Every transformation in the model names the section of
`learn.microsoft.com/en-us/graph/onenote-input-output-html` it was read from on
2026-09-11. What this measures is what this program does with what the
reference says the service returns. It does not measure what the service
returns. That is entry 266 in `.planning/WINDOWS.md`.

| What was typed | What comes back | Where it goes |
|---|---|---|
| `# Colours` | `# Colours` | nowhere, it survives |
| `###### Colours` | `###### Colours` | nowhere, it survives, and so does every level between |
| `- Milk\n- Bread` | `- Milk\n- Bread` | nowhere, it survives |
| `1. Milk\n2. Bread` | `1. Milk\n2. Bread` | nowhere, it survives |
| `Two fuses went at once.` | `Two fuses went at once.` | nowhere, it survives |
| `- Milk\n- Bread\n\n## Then` | `- Milk\n- Bread\n\n## Then` | nowhere, it survives, and a heading straight after a list is where a reader most needs it to |
| `- Live is brown\n  - Older cable: red` | `- Live is brown  Older cable: red` | **this program.** The nesting reaches the page and comes back from it. `from_markup`'s list pass reads only an item's direct inline content, so the inner item is swallowed into the outer one |
| `- One\n  - Two\n    - Three` | `- One  Two  Three` | **this program**, the same way, and three levels become one bullet |
| `> Bring the blue folder` | `Bring the blue folder` | **OneNote.** `blockquote` is in no list the reference names, so it is cut before the page is sent |
| `Turn the power **off** first.` | `Turn the power off first.` | **OneNote.** The reference says a character style comes back as inline CSS on a span that was not in the input, and a span carries no meaning this reader can read |
| `Turn the power *off* first.` | `Turn the power off first.` | **OneNote**, the same way |
| `Turn the power ~~off~~ first.` | `Turn the power off first.` | **OneNote**, the same way, and this is the one that changes what a note means: a job crossed off and a job still to do read alike afterwards |
| `Run \gfusebox --check\g first.` | `Run fusebox --check first.` | **OneNote.** `code` is in no list the reference names |
| `\g\g\g\nfusebox --check\nfusebox --repair\n\g\g\g` | `fusebox --check fusebox --repair` | **OneNote**, and this is the sharpest one. Neither `pre` nor `code` is named, so a code block has no representation on a page at all, and HTML then collapses the line breaks that were its meaning. Two commands become one line that runs neither |
| `Before\n\n---\n\nAfter` | `Before\n\nAfter` | **OneNote**, and it is the only construct that leaves nothing behind: `hr` is not named and has no text inside it to keep |
| `[The manual](https://example.org/manual)` | `The manual` | **this program.** The address reaches the page and comes back. `from_markup` contributes a link's words and not its address, deliberately, so a note keeps its words and loses its links |
| `![The fuse box](https://example.org/fusebox.png)` | `The fuse box` | **this program.** The picture reaches the page and comes back with its description. `from_markup` emits the description alone, so the picture stops being one |
| `![](https://example.org/fusebox.png)` | `image with no description` | **this program**, the same way, and the sentence is deliberate: a picture nobody described is the sender's gap to be shown rather than ours to hide |
| `\| Left \| Right \|\n\| --- \| --- \|\n\| one \| two \|` | `Left\n\nRight\n\none\n\ntwo` | **both.** OneNote does not name `th`, so the header row is written as an ordinary row on the way out. `from_markup` has no table arm at all, so every cell comes back as its own paragraph and which column it was in is gone |
| `Line one\nLine two` | `Line one  Line two` | **this program.** `as_markup` turns a soft break into a hard one on purpose, so the break reaches the page and somebody looking at it in OneNote sees two lines. `from_markup`'s inline pass turns a `br` back into a space |
| ` ` | `\e` | **the format.** HTML collapses a run of whitespace and cannot hold one at the end of a line |
| `\e` | `\e` | nowhere. An empty note costs nothing |

**The hidden source div was tried and it does not work.** A `data-id` survives
and a div carrying one is preserved, so a hidden div holding the Markdown
source looks like a way to make the round trip exact. Put through the same
model, `# Colours\n\n- Live is brown\n- Neutral is blue` comes back as
`# Colours - Live is brown - Neutral is blue`: the div survives and the source
inside it is still text in an HTML document, so its blank lines and the line
starts that made it Markdown are gone. It would not be free even where it
worked, because it puts a second copy of every note inside that note, which is
the drift `long_text::as_markup`'s own comment says the matched pair exists to
prevent. The measurement is
`test_a_hidden_div_carrying_the_source_does_not_bring_it_back`.

**Five rows of that table say "this program", and reading them as defects in
`from_markup` is the wrong conclusion.** A nested list, a table, a link's
address, a picture and a line break inside a paragraph are all lost on the way
back, and all five are lost there rather than at the service. But
`long_text::from_markup` is a reader written for **speaking**, and its output is
meant to be heard rather than stored. Its own doc comment gives the reason a
link contributes its words and not its address: `spoken` returns a
paragraph-only field exactly as written, so an address that survived would be
read out as brackets, parentheses and every character of a URL. It emits a
picture's description alone and will not invent one the sender never wrote,
which `NO_DESCRIPTION` and guardrail 9 both exist for.

It has two callers on that job outside notes, an event's description at
`calendar.rs:3037` and a task's body at `tasks_api.rs:542`, both reading what
Google and Microsoft hand back. So changing it to satisfy a note's round trip
would make a Google task's description read a URL aloud character by character.

A note's round trip needs a reader whose output is **stored and edited again**,
which is a different answer to the same HTML. That reader does not exist, it is
what PIM-04's structure criterion asks for after the rewording of 2026-09-11,
and it belongs to no plan yet. Whoever writes it leaves `from_markup` alone.

### Reading the two columns together

Seven of the twenty-two rows survive. Of the fifteen that do not, seven are
OneNote's doing, six are this program's own reader, one is both, and one is
HTML's rather than anybody's. That split matters: the seven are facts about
somebody else's service and cannot be argued with, and the six are decisions
`from_markup` made for reading a message, where a link read aloud as an address
helps nobody and a note is a different job. **Changing those six is a plan
rather than a line**, because that module is shared with every message body and
signature this program renders.

## Containers: the identifier that is not one identifier

**A container is an opaque string the backend hands out, and this program never
takes it apart.**

A CalDAV journal collection is one address. **A OneNote page lives four levels
down**: a notebook, then a section group, then a section, then the page. So
"which container" is not one identifier everywhere, and anything here that split
a container string on a separator, or assumed it was a URL, or assumed there was
exactly one level above a note, would be reading a CalDAV address.

**Requirements:**

1. The container is opaque. It is stored and handed back, and nothing parses it.
2. A backend that has more than one level above a note flattens that into its
   own container string. What is inside it is the backend's business.
3. Nothing here assumes a container can be created, renamed or removed through
   this seam. None of those is an operation on this trait, and adding one is a
   decision rather than a line.

## Where this document knows it has assumed CalDAV

Written down rather than left to be found, because a contract that claims to
assume nothing is a contract nobody checks.

| Assumption | Held or given up | Why |
|---|---|---|
| A version marker is a token compared for equality | Held, and narrowed | Equality is the only comparison any backend supports. Ordering and parsing are ruled out above, and `lastModifiedDateTime` still works under that reading |
| The whole document can be written | **Given up** | OneNote cannot replace a body. The operation asks for an end state instead |
| A container is one identifier | Held, by making it opaque | A four-level hierarchy flattens into one string the backend owns |
| A backend can be asked for everything in one container | Held | Both candidates can list. Nothing here has been written against a backend that cannot |
| One note maps to one thing at the backend | **Held, and this is the weakest one** | A OneNote page is an HTML document with a structure of its own, and reducing it to a title and a body loses whatever else was on the page. `new_item.rs` has said since before this document that the mapping is a decision somebody has to make rather than an afternoon's work. This contract does not make it |
| A backend hands back the bytes it was given | **Given up, and it was never true** | Neither format this program speaks can promise it. A backend now says what it kept, and the section on bytes is the requirement |
| A version marker is optional | **Given up** | The absence reads as "treat every copy as moved" on the read and as "nothing moved there" on the push. See the entry below |

## Where the second implementation found this was about CalDAV

Four places, in the order they cost most to find. Read them in this order: it is
the order the next person should check their own backend against, because the
ones at the top are the ones no amount of reading the contract would have
surfaced.

The commits are on branch `a-second-backend-disagrees-and-the-seam-says-where`,
merged into `main` for `05.1-04`.

### 1. A version marker is not optional

**What the second implementation could not do.** Give no marker and remain safe.
It was written that way first, because the plan's own reading of Microsoft's
reference said OneNote has no ETag, which is true. Driven through one sync it
destroyed a change somebody had made at the other end and reported nothing.

**What the seam said.** "A version marker is optional. A backend that gives none
is a backend whose copy is always treated as having moved."

**What it says now.** A marker is required of any backend this program writes to,
and requirement 4 says what a service with no opaque token may use instead. The
measurement is kept as a test rather than as an argument:
`test_a_backend_that_gives_no_marker_loses_a_change_made_at_the_other_end_and_says_nothing`.

**Why this was the expensive one.** The sentence is right. It is right about the
read half, and it is the exact opposite of what the same absence means to the
push half, and nothing about reading it makes that visible. It took writing a
backend that took the sentence at its word.

**Commit:** `9a238bb`.

### 2. The clash story lived entirely on the push

**What the second implementation could not do.** Nothing, and that is the point.
The failure is in both backends and was found while writing tests for the
second one.

**What the seam said.** That a backend reports `ItMovedFirst` and writes
nothing, and that the driver holds both copies. It said nothing about the sync
whose push never reached a backend at all.

**What it says now.** The section on conflict carries a requirement on whatever
drives a backend: a change nobody has sent is not written over by the read.

**Commits:** `9a238bb` for the failing test, `458da20` for the fix.

### 3. A backend cannot say what it could not keep

**What the second implementation could not do.** Report honestly. A page is an
HTML document and HTML collapses whitespace, so what it kept was never what it
was handed, and `Done` had no way to say so.

**What the seam said.** That `Done` carries an identity, because a write may
change one. Nothing about the words.

**What it says now.** `Done` carries `what_it_could_keep`, and the section on
bytes is the requirement. The calendar backend uses it as well, for the carriage
returns RFC 5545 cannot carry, which `05.1-03` had measured and written into a
changelog where nobody meets it.

**Commits:** `847626c` for the failing tests, `6fe4580` for the fix.

### 4. A marker that stands still while the note moves

**What the second implementation could not do.** Promise that its marker moves
whenever its content does, because a clock reading has a resolution.

**What the seam said.** That a marker is not promised to change only when the
content does. That is the harmless direction and it was the only one written
down.

**What it says now.** Requirement 4 carries both directions and says which one
costs somebody their note.

**Commits:** `847626c`, and the measurement is
`test_a_marker_that_stood_still_while_the_note_moved_hides_the_change_from_the_read`.

### And four the second implementation could not shake

Worth as much as the list above, because a contract nobody could break is a
contract nobody checked.

- **A container is opaque.** Handed a three-part string where the first backend
  gets one address, nothing in the sync took either apart. A guard record's
  break is the sync keeping a container's last part, and it reddens nine tests.
- **A write may change what a note is called.** The second backend gives a new
  name every time a body changes, which the first never does, and the seam kept
  up without changing shape.
- **A backend is asked for an end state and not for an edit.** The second
  backend has no operation that replaces a body and was never asked for one.
- **A backend does not resolve a clash.** Both hand the question over, and both
  reach the same sentence through `conflict_choice`.

## What a second implementation in this repository does not prove

Four things, and they are the boundary of the claim rather than hedging.

**It was written by whoever had just read this document.** It is shaped by what
the seam made easy as well as by the four constraints it was drawn from. The
four constraints are what stop it agreeing by construction; nothing stops it
agreeing by habit everywhere else.

**It is not a network.** Nothing in it has met a timeout, a partial response, a
redirect, a rate limit, or a sign-in that expired in the middle of an operation.
Every failure it produces is one somebody chose to write.

**Its four constraints are a reading of documentation on a date.** Read on
2026-09-06 from `learn.microsoft.com/en-us/graph/api/resources/onenotepage`,
`learn.microsoft.com/en-us/graph/onenote-update-page` and
`learn.microsoft.com/en-us/graph/api/resources/onenote-api-overview`. A page can
change and a service can differ from its own page. Whether these four are what a
real Graph client meets is entry 249 in `.planning/WINDOWS.md`.

**The fourth one does not apply, and here is why.** It would have read: an
implementation inside the crate can see private items, so it says nothing about
whether anything outside could be compiled against the seam. This one lives in
`tests/`, which links the library the way another crate would and sees only what
is `pub`. It compiles, so `NotesService`, `ANoteThere`, `ANoteAsItStands`,
`WhatTheBackendSaid`, `WhatTheBackendKept` and `notes_sync::sync_notes` are all
reachable from outside. What that still does not prove is that they are reachable
from outside the *repository*: an integration test is built against the same
source tree, so a change that breaks a published API breaks it here in the same
commit rather than a version later.

## What has never been checked

No account, no calendar server and no OneNote tenant has ever been used with
this program. Every sentence above is reasoning from documentation, from what
this repository already does, and from two implementations neither of which has
opened a socket. Phase 5.2 replaces the second one with a real Graph client and
is required to report every place it was wrong.

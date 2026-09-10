# What a notes backend has to answer

This is the contract behind `application::notes_backend`. It is written as
requirements on **any** backend, not as a description of the one that happens to
be built first, and it is meant to be found wrong: `05.2-03` is required to
report every place it turned out to assume CalDAV.

Nothing implements it yet. `NotesService` names the three operations and has no
implementor, and `NotesBackend` answers which backend an account uses and
answers `ThisComputer` for every account there is. That is the state this
document describes.

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
3. A version marker is optional. A backend that gives none is a backend whose
   copy is always treated as having moved, which is what
   `contacts_sync::the_marker_moved` already does and for the reason its comment
   gives: a missing marker is not evidence that a copy stayed still, it is no
   evidence at all.
4. **A version marker is not promised to be opaque, and is not promised to
   change only when the content does.** This is the sentence most likely to be
   found wrong later, so it is written first. A CalDAV ETag is an opaque token
   the server changes when the resource changes. A OneNote page has no ETag and
   no `eTag` property at all; what it has is `lastModifiedDateTime`, which the
   service writes and which moves for reasons the content did not cause.
   Anything here that compares markers must be a comparison for equality and
   nothing else. Ordering two markers, parsing one as a date, or treating a
   changed marker as proof that the text changed are all CalDAV readings.

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

**A backend needs a record of deletions that outlives the row, and there is no
such table for notes.**

`application::deletions` states one rule for everything this computer deletes: a
note of the deletion outlives the row, because a deleted row cannot carry a "not
yet sent" flag, and every read asks the notes before writing anything down.

Contacts, calendar events and tasks each have one. Notes do not:

```bash
grep -n "CREATE TABLE IF NOT EXISTS deleted" src/data/message_cache/mod.rs
```

That finds `deleted_contacts`, `deleted_calendar_events` and `deleted_tasks`,
and none of them is notes. `05.1-03` builds the missing one.

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

## What has never been checked

No account, no calendar server and no OneNote tenant has ever been used with
this program. Every sentence above is reasoning from documentation and from what
this repository already does, not from a backend that has run. The first
implementation will find something, and `05.2-03` is required to say what.

# How Wixen Mail compares

What other mail clients do, what they get wrong for the people this one is for,
and where Wixen Mail stands against both. Written so that "we should build X"
has a reason behind it other than X being familiar.

The clients worth measuring against are Thunderbird, which is what most blind
Windows users are pointed at, and Outlook, which is what most of them have to
use at work. Both are decades old and enormously more complete than this. That
is not the interesting comparison. The interesting one is that being complete
has not made either of them good to use by ear.

## What the evidence says

In 2025 Thunderbird commissioned an accessibility study through Fable, who put
the desktop application in front of testers who use assistive technology daily.
The findings are the clearest public account of where a mature, actively
maintained, genuinely well-intentioned mail client still fails, and every one of
them is a thing this project can get right or get wrong:

| What the testers found | Where Wixen Mail stands |
|---|---|
| Keyboard shortcuts did not follow common norms | Shortcuts follow Windows and match Outlook and Thunderbird where those agree, and `docs/KEYBOARD_SHORTCUTS.md` is checked against the code by a test |
| Search and filter results made for a confusing experience | Search announces its result count; filters have not been built yet |
| NVDA users met structures that lacked the context they expected | The message list is a report-view list with named columns, and every row can be read in full with Space |
| Narrator gave no confirmation for actions like moving messages | Every action that changes something announces what it did |
| Narrator did not recognise menu stage changes in submenus | Not verified. No Narrator pass has been done |
| JAWS could not read message bodies on a braille display | The reading surface writes text, not a rendered view, so braille gets the same content. Not verified with a display |
| JAWS opened filter menus silently | Filters have not been built yet |

The pattern across all three screen readers is the same: the structure was
present and the experience was not. That is the distinction this project's
guardrails are written around, and it is why a clean automated scan is treated
here as a starting point rather than a result.

## Where Wixen Mail is behind

These are things a person moving from Thunderbird or Outlook would expect on the
first day and not find. They are listed because naming them is more useful than
implying they exist.

| Missing | Why it matters | Tracked |
|---|---|---|
| Signatures | Every comparable client has them, per account. Blind users also specifically want signatures to be skippable when reading, which needs the client to know where one starts | Not yet |
| Unified inbox | Anyone with more than one account works out of one list, not several | Not yet |
| Message filters and rules | The engine exists in `application::filters` and nothing runs it against arriving mail | Not yet |
| Saved searches | A search worth running twice is worth keeping | Not yet |
| Tags or labels, on number keys | Thunderbird's `Ctrl+1` to `Ctrl+9`. The fastest triage gesture there is | Not yet |
| Templates | Repetitive replies without retyping | Not yet |
| Several identities per account | One mailbox, more than one address to send as | Not yet |
| Import and export of mail | Nothing leaves this application in a standard format yet. That makes it hard to leave, which is not a property to be proud of | Not yet |
| Encryption, OpenPGP or S/MIME | Thunderbird has OpenPGP built in | Not yet |
| Vacation or automatic replies | Usually a server feature, and usually set from the client | Not yet |

Two more are worth stating plainly because their absence is a decision rather
than a gap: there is no advertising, no telemetry, and no account required to
use it, and the cached mail is not encrypted, which is written down rather than
implied away.

## Where Wixen Mail is ahead

Not "has more features". These are things the comparable clients could do and do
not, and all of them exist because the audience is the first consideration
rather than a later one.

**Reading a row without opening it.** Space reads the item under the cursor,
pressed again reads it in full, Shift+Space goes straight to full. In every
module, the same key, the same behaviour. The alternative, which is what the
others make somebody do, is opening each message to find out whether it was
worth opening.

**Structure spoken, not implied.** A heading in a message or a note is read as a
heading, a list item as an item. Speech has no other way to say it and no other
client here bothers.

**Reading dates the way a person asked for.** Relative or absolute, day first or
month first, spoken or numeric, twelve or twenty-four hour, taken from Windows
by default and changeable.

**Nothing writes to a server until it is allowed to.** Set per capability, with
the safest answer winning, so an alpha build cannot quietly reorganise a real
mailbox.

**Feedback on more than one channel, bounded.** Speech, an earcon, braille and
the status line, distinguishable from each other, and rate limited so a syncing
mailbox cannot flood them.

**A mute for reading aloud.** Private mail gets read out in rooms with other
people in them.

## What this says about what to build next

The gaps above are not equally urgent. Ranked by what a person hits first:

1. **Signatures.** Hit on the first message sent. Small, self-contained.
2. **Filters running against arriving mail.** The engine is already written and
   tested; what is missing is the part that runs it.
3. **Unified inbox.** Hit by anyone with two accounts, which is most people.
4. **Tags on number keys.** The fastest thing to build of these and the one that
   changes daily use most.

Import and export matters more than its position here suggests, for a reason
that is not about features: a client somebody cannot leave is a client they
should be wary of joining.

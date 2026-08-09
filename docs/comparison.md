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
| Search and filter results made for a confusing experience | Search announces its result count, and rules run on arriving mail and say how many they sorted |
| NVDA users met structures that lacked the context they expected | The message list is a report-view list with named columns, and every row can be read in full with Space |
| Narrator gave no confirmation for actions like moving messages | Every action that changes something announces what it did |
| Narrator did not recognise menu stage changes in submenus | Not verified. No Narrator pass has been done |
| JAWS could not read message bodies on a braille display | The reading surface writes text, not a rendered view, so braille gets the same content. Not verified with a display |
| JAWS opened filter menus silently | Not verified with JAWS. The rule editor is an ordinary dialog with named controls |

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
| Saved searches | A search worth running twice is worth keeping | Not yet |
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

**Nothing changes at a server without permission, and permission is split by
cost.** Under Allow Changes, a new installation allows one of the two: tasks,
contacts and the calendar go up to a provider, and mail does not. Three places
can each say no, the safest answer wins, and the command line can only ever
narrow, so an alpha build cannot quietly send or delete anybody's mail.

**Feedback on more than one channel, bounded.** Speech, an earcon, braille and
the status line, distinguishable from each other, and rate limited so a syncing
mailbox cannot flood them.

**A mute for reading aloud.** Private mail gets read out in rooms with other
people in them.

**Every action uses the account the message is in.** Sounds like nothing until
one list holds mail from several accounts, at which point using whichever
account is open sends a flag change to the wrong server.

## What this says about what to build next

All four of the ones ranked here when this was written have since been built: signatures, rules
running on arriving mail, one list for every inbox, and labels on the number
keys. Three of them turned out not to be missing features at all. Each had its
storage, its editor and its tests already written, and nothing ever called the
last step, which is its own lesson about where to look next.

What is left, in the order somebody would miss it:

1. **Saved searches.** A search worth running twice is worth keeping.
2. **Templates.** Repetitive replies without retyping.
3. **Several identities per account.** One mailbox, more than one address.
4. **Import and export.**

Import and export matters more than its position here suggests, for a reason
that is not about features: a client somebody cannot leave is a client they
should be wary of joining.

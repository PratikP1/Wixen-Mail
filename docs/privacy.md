# What Wixen Mail sends, and where

Short version: your mail goes to your mail provider, and your contacts, your calendar and
your tasks go to the provider you signed in to, because a new installation allows changes
to those three to be sent. Showing a message that points at a picture asks that address for
the picture, and the section below says what that means. There is no analytics, no telemetry,
no crash reporting service and no update check that says who you are.

This page is the long version, because "we respect your privacy" is a sentence anybody can
write.

## Where your things are

Everything Wixen Mail stores is in one folder on your computer:

```text
%LOCALAPPDATA%\wixen-mail\
    config\           your settings and one file per account
    cache\            the mail that has been downloaded
    sound_schemes\    sound packs you have imported, if any
    logs\             the running log and crash.log
```

Your passwords and sign-in tokens are not in that folder. They are in the Windows credential
store, protected per user by Windows itself.

**The downloaded mail is not encrypted.** Windows stops other people who use the computer from
reading the folder, but anything running as you can read it, and so can anyone who takes the
drive out unless the disk itself is encrypted. Turn on BitLocker if that matters to you. This
is the same position as Outlook's offline folders and Thunderbird's local store, and it is
stated here rather than left to be discovered.

### Attachments are kept too

When you open a message, the files it carries are kept in the `cache` folder alongside its
text. That is what lets you open an attachment a second time without waiting for the whole
message to come down again, and what puts your files into an export. Like everything else in
that folder, **they are not encrypted**.

This is more of your mail on disk than earlier versions kept, and it is worth knowing if you
share the computer or carry it around. A single file is kept up to 25 MB, and all of them
together up to 512 MB; past that, the ones you read longest ago are dropped. The same file
arriving on twenty messages is stored once.

Nothing about an attachment is written to the log. The log records counts and byte totals, not
file names and not contents.

### Signed mail is kept twice

A message signed with a certificate is stored twice: once the ordinary way, as text and
attachments, and once more exactly as it arrived, byte for byte. Both copies are in the same
`cache` folder and neither is encrypted.

The second copy is there because a signature can only be checked against the exact bytes that
were signed. Reading a message and writing it out again changes small things, such as the
order of its headers and where its lines wrap, and any one of those changes makes a good
signature look like a bad one. Without the original bytes the signature could be checked once,
as the message arrived, and never again, so opening the same message a second time would say
nothing about it.

This applies only to mail that says it is signed, which is a small share of most mailboxes.
Ordinary mail is stored once, as it always was. It applies however the message reached this
computer: fetched from an IMAP server, collected over POP, or brought in from a saved message,
a mailbox archive or an Outlook data file.

The second copy is dropped when a signed message is larger than 25 MB, and when the space these
copies use passes 128 MB, the ones read longest ago going first. Two kinds of mail are never
dropped that way, because there would be no getting them back: mail collected over POP, and
mail brought in from a file. Once those fill the 128 MB, no further copies are kept, rather
than existing ones being destroyed to make room. Deleting a message drops its second copy too,
again except for those two kinds, where dropping it would leave nothing to restore if you
undeleted the message.

Whenever there is no second copy, for any of these reasons, the message says its signature
could not be checked here, and says plainly that this is not the same as a signature that
failed.

### Contact groups stay here

A contact group is a name you give to some of the people in your address book, so you can write
to all of them at once. Groups are kept on this computer and nowhere else.

That means two things:

- A group you make here is never sent to Google or Outlook. Nobody else sees it, and it will not
  appear on your phone.
- A group you already keep in Gmail or Outlook does not appear here. Your contacts arrive from
  those accounts, but their groups do not.

The people in a group still belong to whichever account holds them, and putting somebody in a
group changes nothing about their contact.

## Who Wixen Mail talks to

| Who | When | What goes |
|---|---|---|
| Your mail provider | Checking, reading, sending | The mail itself, over TLS |
| The same provider, for your contacts, calendar and tasks | Syncing, which a new installation allows | The contacts, events and tasks |
| A separate calendar or contacts server | Syncing, if you set one up | The events and contacts |
| Your organisation's directory | Only if you name one on the account, see below | The part of a name you have typed into To, Cc or Bcc |
| Google or Microsoft sign-in | When you sign in with a browser | The sign-in, in your browser |
| Google Safe Browsing | Only if you switch it on, see below | Four bytes, and only sometimes |
| Whoever a sender points a picture at | Showing a message in the preview pane or a conversation window | The request for the picture, which says the message was opened |

Nothing else is asked for by this program on its own account. There is no server belonging
to this project, so there is nowhere for anything to go even by accident. The last row is
the sender choosing, not this program, and the section below says what it means.

## Looking somebody up while you type

Typing part of a name into To, Cc or Bcc looks for people to write to. Your own contacts
on this computer are always searched, and nothing leaves the machine to do it.

Your organisation's directory is a different matter, and it is off until you turn it on.
It is a server somebody else runs, and asking it means sending it part of a name you are
typing, before you have decided to send anything at all. So nothing is asked of any
directory unless the account names one: the two boxes for it, on the second page of the
Add or Edit Account window, are empty on a new installation and on every account that
existed before this was written. Clearing them stops it again.

With a directory named, what goes to it is the part of the name you have typed, and only
that. It is sent after you stop typing rather than on every keystroke, and only once you
have typed at least three letters, so a name typed straight through is one question and
not six. Nothing about the message goes with it: not the subject, not the body, not the
other recipients.

What you type is never written to the log. The log records that a search failed and why,
and never what was searched for.

The connection is encrypted where the directory offers it. An address beginning `ldaps://`
is encrypted from the start, and one beginning `ldap://` is not; both are accepted, because
some internal directories offer only the second, and which one you get is the address your
organisation gives you.

## Asking when the people invited to a meeting are free

Nothing is asked until you choose Find when everyone is free in the event window. Filling
in the guest list sends nothing to anybody on the way to asking, and asking sends nothing
to the people named.

Saving the meeting is a separate matter, covered below.

When you do ask, the question goes to your own calendar server, and it names the whole
guest list in one request. So that server learns that you are thinking about a meeting
with these named people, in this window. Where it passes the question on to another
organisation's server, that organisation learns the same about its own person. Both are
how the standard works and cannot be avoided while still asking.

What is avoidable is left out. The question carries no title, no description, no location
and no note: only who is asking, who is being asked about, and the window of dates. Nobody
is asked about unless you put them on the guest list. The reply carries stretches of time
and never what anybody is doing in them, and nothing here asks for more, which is why this
does not read colleagues' calendars directly even where an account could.

Your own calendar is read from this computer and goes nowhere.

The address the question is posted to is one your calendar server names, and it is checked
before anything is sent: an address on a different host is refused rather than followed,
because following it would hand both the guest list and your sign-in to a server you never
agreed to. You are told the server does not offer this instead.

Nothing about the answer reaches the log. The log records that a server did not answer and
why, and never who was asked about or what came back.

## Saving a meeting with people on it

This only happens if you have turned on Allow Changes in Settings. With it off, everything
below stays on this computer.

A meeting you make in a Google or Outlook calendar goes up to that provider with its guest
list, so the people you named are on the meeting there and not only in your copy. Adding
somebody to a meeting is what makes a provider email them an invitation. Wixen Mail does
not send that mail and has no way to ask either provider not to, so assume that saving a
new meeting tells everybody on it. Try it with an address of your own first.

Changing a meeting your provider already holds sends no guest list at all. Adding or
removing a guest there has to be done in Google Calendar or Outlook.

Syncing a calendar does not write anybody's address to the log. It records which calendar
was synced and how many events moved.

## Pictures a message points at

A message can carry its pictures or point at them. Where it points at one, the address the
sender wrote is left in the message, and a surface that shows the message in a browser asks
that address for the picture. Two surfaces do that: the preview pane, which is off until
you switch it on in the View menu, and the conversation window. The window a message opens
into when you press Enter on it is a text control and asks nobody for anything.

The request tells whoever is at the other end that the message was opened and roughly when.
Senders use that on purpose: a picture the size of a full stop, with a different address
for every recipient, is how a mailing list learns who read it.

There is no setting for this yet. The Reading tab in Settings says so, where a switch for
it would be. Until there is one, the way to avoid it is to leave the preview pane off and
read a message in its own window, which is what happens unless you ask for the preview.

This was read out of the code rather than measured on the wire.

## Reading your messages to mark suspicious ones

On by default, in Settings, then Advanced, under "Checking whether a message is what it says
it is". It is on by default because it sends nothing to anybody.

Wixen Mail reads each message on your computer and marks it when something looks wrong: a link
whose words and address disagree, an address made to look like somebody else's, or pressure to
act at once. The reading happens entirely on your computer, over text that is already there.
No account is needed and no network request is made. The most it does is put a word in the
safety column and say it when you arrive on the message.

Turning it off means those messages arrive with nothing said about them. It does not turn off
what your mail provider already said about a message: that is read from the message's own
headers, costs nothing, and is not a setting.

This is a different setting from the one below, deliberately. This one sends nothing. The one
below can put four bytes of a link on the wire, so it is off unless you ask for it.

## Link checking, if you switch it on

Off by default, in Settings, then Advanced. Here is exactly what it does, because this is the
one feature that involves a third party at all.

There are two ways to use Google Safe Browsing. Wixen Mail uses the one that does not send
your links.

**The way it does not work.** The Lookup API takes a URL, sends it to Google, and gets a
verdict back. It is a few lines of code and it would hand Google every link in your private
correspondence. It is not used here and it will not be.

**The way it does work.** The Update API sends Google's lists to you instead. Google publishes
its lists of known phishing and malware sites as short fingerprints, four bytes each. Wixen
Mail downloads those lists to your computer. When a message has links in it, each link is
turned into a fingerprint on your computer and compared against the list on your computer.

For ordinary mail, no link ever matches, and so **nothing is sent to Google at all.** Not the
link, not a fingerprint, not a note that a message was read.

When a link does match one of the downloaded fingerprints, those four bytes go to Google, and
Google sends back every full fingerprint that starts with the same four bytes. The comparison
against your actual link happens back on your computer. Four bytes is short enough that it
matches millions of possible web addresses, so what Google learns is that somebody at your IP
address saw one of a very large set of links.

Over the life of an installation, Google receives:

- your IP address, as it would from any request
- an API key identifying the application, the same one for everybody
- periodic list downloads, which carry nothing about you at all and would be identical on a
  computer that had never received a message
- on the rare match, four bytes and a timestamp

Google never receives: the link, the domain, the sender, the subject, the recipient, your
email address, or any part of any message.

### Turning it on

It needs a Google API key, which you get yourself from the Google Cloud Console and put in
`oauth.toml` in your settings folder. See `oauth.toml.example` in the source. Without a key
the feature does nothing at all, whatever the setting says, and the log says so on startup.

If a warning ever appears because of this, it says so and credits Google, which their terms
ask for and which you are entitled to know anyway.

### What it will not tell you

Wixen Mail will never say a message is safe. A link that is not on Google's lists is a link
Google has not listed, which is a much smaller statement, and dressing it up as an all-clear
is how people are taught to stop reading warnings.

## Spam and phishing warnings without any of that

The Safety column and the warning above a message work with link checking switched off, and
they always have. They come from what is already in the message:

- the spam headers your provider's own filter added
- Microsoft's confidence levels, where the message came through Microsoft
- what the receiving server made of the sender's anti-forgery records
- whether the message was in the junk folder, which is the whole of what Gmail tells a mail
  application
- Wixen Mail's own reading of the message, on your computer

None of that involves anybody else. It is all either already in the message or worked out
here.

## Logging

`logs\wixen-mail.log` records what the application did. It never contains a password, a
sign-in token, or the body of a message. It does contain folder names, message counts, error
messages from your provider, and the addresses involved in a failed send, because those are
what makes a problem diagnosable.

If you send a log to report a problem, it is worth reading first. Nothing in it should be
sensitive, and if you find something that is, that is a bug worth reporting on its own.

## Uninstalling

Uninstalling removes everything: the program, your accounts, your settings, the downloaded
mail, and your saved passwords and sign-in tokens. It writes a note in your temporary folder
every time, `wixen-mail-uninstall.log`, saying what went and naming anything it could not
remove, so a leftover is something you are told about rather than something you find. Two
cases leave no note, and [Installing and uninstalling](installing.md) says which and what to
check by hand. Your mail itself is untouched, because it is on your provider's server and
Wixen Mail only ever held a copy.

# What Wixen Mail sends, and where

Short version: your mail goes to your mail provider, and your contacts, your calendar and
your tasks go to the provider you signed in to, because a new installation allows changes
to those three to be sent. Showing a message that points at a picture asks that address for
the picture, and the section below says what that means. There is no analytics, no
telemetry and no crash reporting service. Wixen Mail can ask GitHub whether a newer version
has been published, which sends nothing about you but does reach a server; it is off on a
new installation and [Asking whether there is a newer version](#asking-whether-there-is-a-newer-version)
says exactly what it sends.

This page is the long version, because "we respect your privacy" is a sentence anybody can
write.

## Where your things are

Everything Wixen Mail stores is in one folder on your computer:

```text
%LOCALAPPDATA%\wixen-mail\
    config\           your settings, one file per account, and oauth.toml
    cache\            the mail that has been downloaded
    sound_schemes\    sound packs you have imported, if any
    logs\             the running log and crash.log
    updates\          an installer being downloaded, while one is
    security.key      only on a machine upgraded from an older version
```

`updates\` is there only while Wixen Mail is fetching a new version, and it holds one file:
the installer. It is emptied when the update is installed, emptied straight away if the
installer turns out not to be signed by this project, and emptied again the next time Wixen
Mail starts, so a download interrupted by a crash or a power cut does not leave an installer
sitting on your disk. If you have "Tell me about new versions" set to not look, nothing is
ever put there.

**One exception, on a computer where Windows cannot tell Wixen Mail where your local
application data lives.** That is rare and it is usually a sign something else is wrong with
the profile, but when it happens the log has to go somewhere, so it goes to
`%TEMP%\wixen-mail\logs` instead. Everything the Logging section below describes is in it,
in the same place anything else on the computer writes its temporary files. Paste `%TEMP%`
into File Explorer to look. Nothing else moves: only the log has a fallback, because only
the log has to be written before anything can report that the folder could not be found.

`security.key` is there only if this computer ran an older version of Wixen Mail. Nothing
creates it now. Older versions locked saved passwords in a file with it, and it is read once
so those passwords can be moved into the Windows credential store. A fresh install never has
one.

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
a mailbox archive or an Outlook data file. Corrected on 2026-09-17: the last of those was true
of the code and not of anything you could do until the build of 2026-09-17 that carries the fix
for issue 53, because no command reached the Outlook data file reader before it. File, Import
Mailbox reaches it since then, and no real Outlook data file has been through it yet.

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

### A message being moved to another account is kept until the move ends

Moving a message to a folder on a different account means fetching it from one mail server
and uploading it to the other. While that is happening, the whole message is kept in the
`cache` folder, and it is not encrypted, like everything else in that folder.

Normally that is a few seconds. If Wixen Mail closes part way through a move, the message
stays there until you answer the question you are asked the next time you start it, and at
most seven days. Answer it either way and the copy goes immediately. Say nothing, and it
goes on its own after the seven days.

A message larger than 25 MB is not kept at all, and the move happens exactly as it would
have. If several interrupted moves add up to more than 64 MB, the newest one is not kept,
so a move you have not been asked about yet is never dropped to make room for one happening
now.

**This copy does not protect your message, and nothing else here should be read as saying
it does.** A move puts the message at the second account first and only takes it off the
first afterwards, so at every point where the program can stop, the first account still has
it. What the copy buys is narrower: the move can be finished even when the account the
message came *from* is the one that is not answering, and a large message does not have to
be downloaded a second time. Deleting the copy loses nothing.

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
| GitHub | Checking whether a newer version has been published, which you ask for or switch on, see below | The request, which carries nothing about you |
| OneNote | Never. Nothing here reads or writes a notebook, see below | Nothing |
| Whoever a sender points a picture at | Showing a message in the preview pane or a conversation window | The request for the picture, which says the message was opened |

Nothing else is asked for by this program on its own account. There is no server belonging
to this project, so there is nowhere for anything of yours to go even by accident. The
GitHub row is the one place this program asks anything for its own reasons rather than
yours, and it carries no account and no identifier. The last row is the sender choosing,
not this program, and the section below says what it means.

### The OneNote permission, which nothing uses

Signing in to an Outlook or Office 365 account asks Microsoft for `Notes.ReadWrite`, which
would let a program read the notebooks, sections and pages on that account, make a page,
change one and remove one. **Nothing in Wixen Mail uses it.** No notebook has ever been
opened, no page has ever been read, and no note you write here goes anywhere.

It is asked for now so that your account is ready when notes do sync, rather than sending
you back through a browser sign-in at that point.

That is a permission you have granted and Wixen Mail is not using, which is worth knowing
rather than worth hiding: it is wider than what the program does. If you would rather not
grant it, the account works without it, and everything except notes behaves exactly the
same. [Setting up a provider](PROVIDER_SETUP.md) says how the sign-in is redone.

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

## Asking whether there is a newer version

This is the one thing Wixen Mail asks for its own reasons rather than yours, so here is all
of it.

**When it happens.** Two ways, and one of them is always available. Help, then Check for
Updates, asks straight away, whatever your setting says, because asking deliberately is
something you should be able to do without changing a setting first. Separately, the program
can ask once each time it starts, and it does that only if you have chosen a kind of version
to hear about.

**What decides the second one.** A setting in Settings, then General, under "New versions".
It is called "Tell me about new versions" and it has three answers:

- **Do not look for new versions.** Nothing is asked when the program starts. This is where
  a new installation begins, and where every installation that existed before this feature
  begins, so nobody starts sending anything to GitHub because of an upgrade.
- **Released versions.** Finished versions only.
- **Released versions and test versions.** Also the versions that stage a release, the ones
  with `alpha`, `beta` or `rc` in the number. These go to a small group of testers on
  purpose, so they are never offered on "Released versions"; if you have been sent one and
  the check keeps saying you are current, this is the setting that explains why.

**What goes.** One request to `api.github.com`, asking which versions of Wixen Mail have
been published. It carries no account, no sign-in, no identifier for you, and nothing about
the mail on this computer. It would be the same request from any copy of this version on any
computer.

**What GitHub gets.** The address the request came from, because every request carries one.
GitHub's own documentation puts it plainly: an unauthenticated request "is associated with
the originating IP address, not with the user or application that made the request". So what
GitHub can see is that somebody at your address is running Wixen Mail and asked this
question. That is not nothing, and saying it is would be the sort of promise this page
exists to avoid.

GitHub allows sixty such requests an hour from one address. That is generous for one person
and reachable from an office, a campus or anywhere a lot of people share a connection. When
it is reached, the check says so and says to try later; it never tells you that your version
is current when it did not find out.

**What does not go.** Not your email address, not your accounts, not your provider, not your
folder names, not your mail, and no identifier that would let two requests be recognised as
coming from the same installation.

**Something is downloaded, and nothing is run until you say so.** When the check finds a newer
version, Wixen Mail fetches the installer for it and checks who signed it, without asking you
first. Then it asks you once, and only about installing it.
[Downloading an update](#downloading-an-update) below says what that fetch sends, which
computers it reaches, and what arrives on your disk.

## Downloading an update

**What goes, and to whom.** Two computers, both GitHub's. The question about which versions
exist goes to `api.github.com`, as the section above describes. The installer itself is
fetched from `github.com`, which passes the request on to `objects.githubusercontent.com`,
the machine GitHub serves release files from. Neither request carries an account, a sign-in
or anything that identifies you, and neither is signed in to anything. GitHub's own
documentation applies to both: an unauthenticated request "is associated with the originating
IP address, not with the user or application that made the request". So GitHub can see that
somebody at your address downloaded this version of Wixen Mail.

**A file arrives on your computer without you asking for it.** This is not a promise about
what is sent, it is a plain fact about what turns up, and it deserves saying on its own.
If you have chosen a kind of version under "Tell me about new versions", then when a newer
one is published Wixen Mail downloads its installer on its own, without asking you at that
moment. How large that download is has not been measured, because no release has been
published yet; the target is about 12 MB, and this sentence will give the measured size
with its date once there is a release to measure. It goes in
`%LOCALAPPDATA%\wixen-mail\updates`. It is
removed when the update is installed, removed straight away if it turns out not to be signed
by this project, and removed again the next time Wixen Mail starts. If the setting is left on
"Do not look for new versions", none of this ever happens.

**Nothing is run without you saying so.** Once the installer has been downloaded and checked,
Wixen Mail asks you once whether to install it, and that is the only question this feature
asks. Answering no leaves the version you are running exactly as it was and deletes the file.
Answering yes closes Wixen Mail and opens the installer, and you are told that is about to
happen before it happens.

**Anything this project did not sign is refused, not warned about.** Wixen Mail checks two
things before it will offer to run an installer: that the signature on it is valid, and that
the name on that signature is this project's own. A file failing either is deleted and you are
told why. You are never asked whether to run it anyway.

On a computer with no way to check a signature at all, nothing is downloaded either. The
question is asked before any bytes are fetched, so your connection is not spent on a file
Wixen Mail could never look at.

Today that check refuses everything, because nothing this project publishes is signed yet.
Until it is, an update will download, be refused, and point you at the releases page.

## Logging

`logs\wixen-mail.log` records what the application did. It never contains a password, a
sign-in token, or the body of a message. It does contain folder names, message counts, error
messages from your provider, and the addresses involved in a failed send, because those are
what makes a problem diagnosable.

If you send a log to report a problem, it is worth reading first. Nothing in it should be
sensitive, and if you find something that is, that is a bug worth reporting on its own.

## Uninstalling

Uninstalling removes everything Wixen Mail stored: the program, your accounts, your settings,
the downloaded mail, and your saved passwords and sign-in tokens. It writes a note in your
temporary folder
every time, `wixen-mail-uninstall.log`, saying what went and naming anything it could not
remove, so a leftover is something you are told about rather than something you find. Two
cases leave no note, and [Installing and uninstalling](installing.md) says which and what to
check by hand. Your mail itself is untouched, because it is on your provider's server and
Wixen Mail only ever held a copy.

### One thing uninstalling cannot take back

If you said yes when the installer asked whether Windows Search could index your mail, the
Windows Search index has been keeping its own copy of your subjects and message text. That
index is a database under ProgramData, it is not encrypted, and any software running on this
computer can read it. Uninstalling Wixen Mail does not clear it, and nothing Wixen Mail can do
will: only rebuilding the Windows Search index does that. Press the Windows key, type Indexing
Options, choose Advanced, then Rebuild. It takes hours.

The installer says this when it asks, and the uninstaller says it again. It is repeated here
because both of those go past once and this page is somewhere you can come back to.

# What Wixen Mail sends, and where

Short version: your mail goes to your mail provider and nowhere else. Nothing in Wixen Mail
sends your messages, your contacts, your calendar or your links to anybody, and there is no
analytics, no telemetry, no crash reporting service and no update check that says who you are.

This page is the long version, because "we respect your privacy" is a sentence anybody can
write.

## Where your things are

Everything Wixen Mail stores is in one folder on your computer:

```text
%LOCALAPPDATA%\wixen-mail\
    config\        your settings and one file per account
    cache\         the mail that has been downloaded
    logs\          the running log and crash.log
```

Your passwords and sign-in tokens are not in that folder. They are in the Windows credential
store, protected per user by Windows itself.

**The downloaded mail is not encrypted.** Windows stops other people who use the computer from
reading the folder, but anything running as you can read it, and so can anyone who takes the
drive out unless the disk itself is encrypted. Turn on BitLocker if that matters to you. This
is the same position as Outlook's offline folders and Thunderbird's local store, and it is
stated here rather than left to be discovered.

## Who Wixen Mail talks to

| Who | When | What goes |
|---|---|---|
| Your mail provider | Checking, reading, sending | The mail itself, over TLS |
| Your calendar or contacts server | Syncing, if you set one up | The events and contacts |
| Google or Microsoft sign-in | When you sign in with a browser | The sign-in, in your browser |
| Google Safe Browsing | Only if you switch it on, see below | Four bytes, and only sometimes |

Nothing else. There is no server belonging to this project, so there is nowhere for anything
to go even by accident.

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
mail, and your saved passwords and sign-in tokens. Nothing is left to clean up later, and
nothing is left to find you later either. Your mail itself is untouched, because it is on your
provider's server and Wixen Mail only ever held a copy.

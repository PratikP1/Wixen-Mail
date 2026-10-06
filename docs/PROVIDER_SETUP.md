# Email Provider Setup Guides

Quick setup instructions for popular email providers with Wixen Mail.

## Table of Contents

- [Gmail](#gmail)
- [Outlook.com / Office 365](#outlookcom--office-365)
- [Yahoo Mail](#yahoo-mail)
- [iCloud Mail](#icloud-mail)
- [ProtonMail (via Bridge)](#protonmail-via-bridge)
- [What differs between providers](#what-differs-between-providers)
- [Other Providers](#other-providers)
- [What syncs from which account](#what-syncs-from-which-account)

---

## What syncs from which account

One account does everything it can. An Outlook or Office 365 account, and a
Gmail account whose mail signs in through the browser, need nothing more:
contacts, the calendar and tasks use the sign-in you already gave for the mail.

**A Gmail account whose mail signs in with an app password needs one more
sign-in.** Google gives calendars, contacts and tasks only to a browser sign-in,
so that account signs in through the browser for those three alone, from the
Account Manager, and keeps its app password for mail. That sign-in needs a
Google sign-in key you make yourself: [Calendars, contacts and tasks for a
Gmail account](#calendars-contacts-and-tasks-for-a-gmail-account) walks through
it. Until 2026-10-05 this section said there was no second sign-in to set up
for any account. That was untrue for every Gmail account on an app password,
which could never reach its calendars, contacts or tasks, and for every copy of
Wixen Mail made so far, none of which came with a Google sign-in key.

What you get depends on what kind of account it is.

| | Gmail | Outlook, Office 365 | Any other IMAP or POP account |
|---|---|---|---|
| Mail | Yes | Yes | Yes |
| Contacts | Yes, both ways, with a browser sign-in | Yes, both ways | No |
| Calendar | Yes, both ways, with a browser sign-in | Yes, both ways | No |
| Tasks | Yes, both ways, with a browser sign-in | Yes, both ways | No |
| Notes | No | No | No |
| Reminders | No | No | No |

The Gmail and the Outlook, Office 365 columns include Google Workspace and
Microsoft 365 accounts on their organisation's own domains, when the incoming
server is Google's or Microsoft's and the account signed in through the
browser.

None of the Gmail column's contacts, calendar or tasks rows has yet been tried
against a real Google account.

**"Any other IMAP or POP account"** means a mail server and nothing else, which
is what Yahoo, iCloud, ProtonMail Bridge and a self-hosted server are to Wixen
Mail. They carry mail. Contacts, calendars and tasks made while one of those is
your default account are kept on this computer instead, which is the honest
version of the same thing: filing them under an account that will never carry
them anywhere would look like syncing until you opened a second device.

A calendar on its own, without an account, through a calendar server address or
a subscription feed, is not something you can add yet. The code that would read
such a calendar is written and there is no screen for entering the address, so
there is currently no way to set one up.

### Tasks sync both ways

Your task lists and their tasks come down from Google Tasks and from Microsoft
To Do, with their due dates, whether they are done, and on Microsoft their
priority. Ticking one off on your phone removes it here on the next sync.

Tasks you make, tick off or delete here go up to your provider on the next sync,
so they reach your phone and the web page. A task made here gets its real
identity from the provider the first time it is sent.

**When the same task changed in both places, your provider's version wins.**
That is a deliberate choice rather than an accident of the code. Your provider's
copy is what your phone and the web page already agree on, so it is the one you
most likely looked at last. A change you lose that way can be made again; a
change made on your phone and overwritten by a stale copy from this computer
cannot, because nobody would find out it happened.

You are told when it happens. The line after a sync says how many of your
changes were replaced by the server, so a change that disappeared is never
silent.

A change that cannot be sent, because the network is down or the provider
refuses it, keeps waiting and is tried again at the next sync. Nothing is
dropped for failing once.

Sync with Tools, then Sync Tasks. It does not run on its own yet, so a change
made here reaches your phone when you next sync rather than straight away.

A task you make goes into your account's first list, which is the one your
provider treats as the default: "My Tasks" on Google Tasks, "Tasks" on
Microsoft To Do. There is no list picker yet, so if you want it somewhere else,
move it on your phone or on the web page after the next sync.

**One case stays here.** If you make a task on an account that has never synced,
there are no lists yet, so it goes into a list called "My Tasks" that this
computer made. That list has no copy at your provider, so the task has nowhere
at the other end to be put. It stays here and the sync says how many did: "1
kept on this computer". Sync first and it will not happen. There is no way to
move an existing task between lists in Wixen Mail yet, so a task in that state
stays in it.

### Notes and reminders stay on this computer

Not an oversight, and not the same reason for each.

**Notes.** Google Keep has an API and it is only available to Workspace
accounts, so a personal Gmail account cannot use it at all. Microsoft could carry
notes through OneNote, and that is part built. A OneNote page is a formatted
document inside a section inside a notebook, and a note here is a title and some
text, so what happens to the difference had to be decided first. It has been,
and what a note loses on the way is measured. What is missing is the part that
sends anything, so no note you write reaches OneNote and no page in OneNote
appears here.

**Reminders.** Neither provider has a reminder that exists on its own. Outlook
and Exchange make a reminder a property of an appointment or a task, and Google
folded its Reminders into Tasks in 2023. There is nothing on the other side to
sync one to, so this one is not going to change.

### Wixen Mail asks for a OneNote permission it does not use yet

Signing in to an Outlook or Office 365 account now asks for one permission more
than before: Microsoft's Notes.ReadWrite. It lets a program read the notebooks,
sections and pages on your account, make a page, change one, and remove one.

Nothing uses it yet. The permission is asked for now so your account is ready
when notes do sync, rather than asking you to sign in a second time then.

**An account you set up before this version does not have it.** Permission is
granted once, at sign-in, so the sign-in your account is holding was given
without it. That costs you nothing today, because nothing asks OneNote for
anything. When notes do sync, sign in again: open the account, switch the
browser sign-in off and back on, and approve the list the browser shows.

Nothing about this has been tried against a real Microsoft account. No notebook
has ever been opened by Wixen Mail, so whether a personal account can grant this
permission without an administrator is not something we can tell you yet.

### If you signed in before tasks synced both ways

Sending tasks up needs more permission than reading them did, and permission is
granted once at sign-in. A Google account you set up before this version will
keep syncing mail, contacts, the calendar and tasks downwards, and your changes
will sit here waiting.

A Microsoft account set up before then is asked for its tasks with a permission
its sign-in never granted, so since the build of 2026-09-28 we expect Microsoft
to refuse it and its tasks not to sync in either direction until you sign in
again. The line after a sync counts it as a problem, and your changes wait here.
Nothing else the account does changes. Nobody has seen Microsoft do this yet, so
if your tasks stop, tell us.

Fix it by signing in again: open the account, switch the browser sign-in off and
back on, and approve the list of permissions when the browser shows it. The
waiting changes go up on the next sync. Until the build of 2026-09-28 that was
not true on a Microsoft account: Microsoft refused every task change even after
signing in again, because the tasks sync asked for its token without the
permission to write tasks.

The permission is Google's "See, edit, create and delete your tasks" or
Microsoft's Tasks.ReadWrite, in place of the read-only version. Microsoft
accounts are also asked for Notes.ReadWrite now, which the section above
explains and which nothing uses yet.

### Sign in again once for Microsoft's people search

Since the build of 2026-09-28, typing three letters or more of a name into To,
Cc or Bcc on an Outlook or Office 365 account also asks Microsoft's people
search, and the people it finds join the list with "from Microsoft" at the end
of their row. It needs one permission more than before, Microsoft's People.Read,
which reads the people your mailbox writes to and your organisation's directory,
and nothing of your mail.

An account you set up before that build does not have it. Sign in again once:
open the account, switch the browser sign-in off and back on, and approve the
list the browser shows. Until you do, each search says so, and nothing else the
account does changes, because Wixen Mail asks Microsoft for this permission on
its own rather than with the others.

People search has not been tried against a real Microsoft account. An
administrator can switch it off for an organisation, and then each search says
Microsoft refused it. [Privacy](privacy.md) says what is sent.

---

## Choosing a sign-in method

Wixen Mail signs in to a mailbox one of two ways. The account dialog has a
checkbox, "Sign in with the provider in a browser (OAuth)", and it is set from
the address as you type, to whichever usually works for it. You can change it.
An address on your organisation's own domain starts with it off; turning it on
works once the incoming server is Google's or Microsoft's.

The advice next to the password box, and the "Get an app password in your
browser" button, follow the incoming server once you type one, and the address
until then:

- A Gmail or Google Workspace account is advised to use an app password, and
  the button opens Google's page for one.
- A Microsoft account, Outlook.com or Microsoft 365, is told to turn on the
  browser sign-in, and the button says the same rather than opening a page.
  Microsoft no longer accepts a password from a mail program, not even an app
  password: its page on Exchange Online says "Basic authentication is now
  disabled in all tenants" and that this "also prevents the use of app
  passwords", and its page for Outlook.com gives September 16th, 2024 as the
  day "Basic Authentication no longer available to access any Outlook
  account". Both read again on 2026-10-01.

### App password

A password your provider generates for one application, which you can revoke on
its own without changing the password you sign in with everywhere else. This is
the default for Gmail and for any provider we do not recognise. It is not
offered for Microsoft, which stopped accepting app passwords, as above.

It works today, it does not expire, and it does not depend on Wixen Mail being
registered with anybody. You need two-step verification turned on with your
provider before they will give you one.

On Gmail an app password carries mail and nothing else. A Gmail account on one
signs in a second time, through the browser, for its calendars, contacts and
tasks, as [Calendars, contacts and tasks for a Gmail
account](#calendars-contacts-and-tasks-for-a-gmail-account) explains.

Your ordinary password will not work. Google stopped accepting it for mail
applications, and Microsoft has stopped accepting any password from a mail
program. Typing it produces "authentication failed", which reads like a typo
and sends people round the loop again, so the account dialog says this next to
the password box.

### Browser sign-in (OAuth)

You are sent to the provider's own page, you sign in there, and Wixen Mail never
sees your password. This is the default for Outlook.com addresses, because
Microsoft no longer accepts a password from a mail program, and the only way in
to any Microsoft mailbox, Microsoft 365 included.

**What this costs, honestly.** Reading mail is what Google calls a restricted
scope, and an application asking for it has to pass a security assessment before
Google will let the general public use it. Until that assessment is done:

- Only people added by hand to the project's list can sign in, and that list is
  capped at 100.
- Google expires their sign-in after seven days, so each of them has to go
  through the browser again roughly once a week.

That second point is the one that matters in daily use, and it is a Google
policy rather than something Wixen Mail can work around. Until the assessment is
done, an app password is the arrangement that stays working. If you are choosing
for someone who will not enjoy re-authorising every week, choose the app
password.

Microsoft does not apply the same seven-day rule, so an Outlook browser sign-in
keeps working until it is revoked.

The separate browser sign-in a Gmail account on an app password makes for its
calendars, contacts and tasks does not ask for mail, and it still runs out after
seven days while the key it uses is in Testing. Mail on the app password is not
affected when it does.

---

## Gmail

### Requirements
- Gmail account
- 2-Factor Authentication (2FA) enabled (recommended)
- App password (if 2FA enabled)

### Step-by-Step Setup

#### 1. Enable IMAP in Gmail

1. Log into Gmail (https://gmail.com)
2. Click the gear icon → **Settings**
3. Go to **Forwarding and POP/IMAP** tab
4. Under IMAP Access, select **Enable IMAP**
5. Click **Save Changes**

#### 2. Generate an app password

You need two-step verification turned on first. Google does not offer app
passwords without it, and it does not accept your ordinary password for mail.

Go straight to the page:

**https://myaccount.google.com/apppasswords**

The account dialog in Wixen Mail opens this for you: the button next to the
password box is "Get an app password in your browser".

If that page says the setting is not available for your account, two-step
verification is off. Turn it on at
https://myaccount.google.com/signinoptions/two-step-verification and come back.

Once you are on the page:

1. Enter a name for the password, such as "Wixen Mail"
2. Select **Create**
3. Copy the 16-character password shown. Google will not show it again, so
   paste it into Wixen Mail before closing the page
4. Select **Done**

The password is shown in four groups of four with spaces. The spaces are for
reading and Google ignores them, so it does not matter whether you paste them.

If you would rather navigate there yourself: your Google Account, then
**Security**, then **App passwords**. That entry only appears once two-step
verification is on, which is why the direct link is easier.

#### 3. Configure Wixen Mail

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your Gmail address (e.g., `user@gmail.com`). Wixen Mail recognises
   the domain and fills in Gmail's server settings for you:
   - **IMAP Server:** imap.gmail.com, port 993, TLS
   - **SMTP Server:** smtp.gmail.com, port 587, TLS
4. The browser sign-in checkbox is off by default for Gmail. Paste the
   16-character app password into the password box. Your ordinary Google
   password will not work here.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**.

**Google Workspace on your organisation's own domain.** The servers are not
filled in for an address Wixen Mail does not recognise, so type
`imap.gmail.com` as the IMAP server and `smtp.gmail.com` as the SMTP server.
From then on Wixen Mail treats the account as Gmail, by its server. Whether
your organisation lets Wixen Mail sign in, with an app password or through the
browser, is its administrator's decision.

### Calendars, contacts and tasks for a Gmail account

Google gives calendars, contacts and tasks only to a program you sign in to
through the browser. An app password works for mail and never for these three.
So a Gmail account that reads its mail with an app password needs a second
sign-in, through the browser, for its calendars, contacts and tasks alone. Mail
keeps its app password and nothing about it changes.

That browser sign-in needs a Google sign-in key: a client ID and a client
secret that tell Google which program is asking. Wixen Mail does not come with
one, so you make your own in Google's console and keep it on your computer, in a
file called `oauth.toml` that you put in your settings folder yourself. Nothing
in Wixen Mail's own files carries a key, and Wixen Mail sends yours only to
Google, when you sign in.

Each step below is one action. The steps follow Google's own pages as they read
on 2026-10-05:

| Google's page | Last updated |
|---|---|
| Create a Google Cloud project (developers.google.com/workspace/guides/create-project) | 2026-09-03 |
| Enable Google Workspace APIs (developers.google.com/workspace/guides/enable-apis) | 2026-09-03 |
| Configure the OAuth consent screen (developers.google.com/workspace/guides/configure-oauth-consent) | 2026-09-03 |
| Create access credentials (developers.google.com/workspace/guides/create-credentials) | 2026-09-03 |
| Manage OAuth clients, on client secrets (support.google.com/cloud/answer/15549257) | no date shown |
| Using OAuth 2.0 to Access Google APIs, on a project in Testing (developers.google.com, under Google Identity) | 2026-05-26 |
| OAuth 2.0 Scopes for Google APIs, for what each permission is called (developers.google.com, under Google Identity) | 2026-09-14 |

Google renames buttons from time to time. If a step names something you cannot
find, the page in the table is where Google says what it is called now.

**What has not been checked.** Nobody has yet followed these steps with a
screen reader, so whether each page of Google's console reads well under NVDA
or Narrator is not something this page can promise. If a step is hard to reach
by keyboard, tell us which one.

#### 1. Make a project

1. Open https://console.cloud.google.com in your browser and sign in with your
   Google account.
2. Open the menu, then **IAM & Admin**, then **Create a Project**.
3. In **Project Name**, type a name you will recognise, such as Wixen Mail.
   Leave the other fields as they are.
4. Choose **Create**. The console opens the new project.

The rest of the steps happen inside that project. Its name is shown near the
top of every page of the console, so check it there if you have more than one.

#### 2. Turn on the three interfaces

1. Open the menu, then **APIs & Services**, then **Library**. Google's page
   adds **Google Workspace** after Library; that is the section of the library
   the three below are listed in.
2. Search for **Google Calendar API**, open it, and choose **Enable**.
3. Go back to the library and do the same for **People API**, which is how
   Google hands out contacts.
4. Do the same for **Google Tasks API**.

#### 3. Set up the consent screen, and add yourself as a tester

1. Open the menu, then **Google Auth platform**, then **Branding**. If the page
   says Google Auth platform is not configured yet, choose **Get Started**.
2. In **App name**, type Wixen Mail. In **User support email**, choose your
   address. Choose **Next**.
3. Under **Audience**, choose **External**. Choose **Next**.
4. Under **Contact Information**, type your address. Choose **Next**.
5. Tick **I agree to the Google API Services: User Data Policy**, then choose
   **Continue**, then **Create**.
6. Open **Audience**. Leave the publishing status at **Testing**.
7. Under **Test users**, choose **Add users**, type the Gmail address you read
   mail with in Wixen Mail, and choose **Save**.

#### 4. Add the three permissions

1. Open **Data Access**, then choose **Add or Remove Scopes**.
2. Find and tick these three. Typing the address into the filter box is the
   quickest way to each:

   | Address | What Google calls it |
   |---|---|
   | `https://www.googleapis.com/auth/calendar` | See, edit, share, and permanently delete all the calendars you can access using Google Calendar |
   | `https://www.googleapis.com/auth/contacts` | See, edit, download, and permanently delete your contacts |
   | `https://www.googleapis.com/auth/tasks` | Create, edit, organize, and delete all your tasks |

3. Confirm your choice, then choose **Save**.

Do not add mail, `https://mail.google.com/`. Your mail keeps its app password,
and Wixen Mail never asks this sign-in for mail.

#### 5. Make the client, and copy its two values straight away

1. Open the menu, then **Google Auth platform**, then **Clients**.
2. Choose **Create Client**.
3. For **Application type**, choose **Desktop app**.
4. In **Name**, type Wixen Mail. Only you see this name.
5. Choose **Create**.
6. Google shows the **client ID** and the **client secret**. Copy both now.

**Google shows the secret only this once.** Its page says client secrets "are
only visible and downloadable from the Google Cloud Console at the time of their
creation", and afterwards it shows only the last four characters. If you lose
it, open the client and choose **Add Secret** to make a new one, and use that.

#### 6. Put oauth.toml in your settings folder

1. Open Notepad.
2. Type these three lines, putting your client ID and your client secret between
   the quotation marks:

   ```toml
   [gmail]
   client_id = "your client ID"
   client_secret = "your client secret"
   ```

3. Choose **Save as**. In **Save as type**, choose **All files**, so Notepad
   does not add `.txt` to the name.
4. In **File name**, type the whole path below, including the quotation marks
   at each end:

   ```text
   "%LOCALAPPDATA%\wixen-mail\config\oauth.toml"
   ```

5. Choose **Save**.

If Notepad cannot find the folder, Wixen Mail has not run on this computer yet:
start it once, close it, and save again. The file stays on your computer. Wixen
Mail reads it each time it signs in or brings your calendars, contacts and
tasks, so it does not need restarting.

#### 7. Sign in for the calendars, contacts and tasks

1. In Wixen Mail, press `Ctrl+Shift+A` to open the Account Manager.
2. Choose your Gmail account in the list.
3. Press `Alt+T`, **Sign In for Calendars, Contacts and Tasks**. Wixen Mail
   says it is signing in and your browser opens at Google.
4. Sign in with the Google account you added as a test user.
5. Google shows a screen saying it has not verified this app. That is expected
   for a key in Testing: the app it means is your own project. Choose to
   continue.
6. Google lists the three permissions. Allow them.
7. Your browser says you can close it, and Wixen Mail says the account is signed
   in for its calendars, contacts and tasks.
8. Close the Account Manager. Wixen Mail brings that account's calendars,
   contacts and tasks once, straight away.

After that, Sync Calendar, Sync Contacts and Sync Tasks on the Tools menu, and
`F5` in the Calendar, Contacts and Tasks modules, ask Google with this sign-in.

#### Which of your Google calendars come

Every calendar on your Google calendar list comes, not only your main one, and
each is a calendar of its own in the Calendar module's sidebar. Your main
calendar is called Google Calendar unless you named it yourself at Google. Every
other calendar has the name you gave it at Google, or Google's name for it.
Until 2026-10-05 only the main calendar came, and every other calendar on the
account was never read.

- **A calendar shared with you to look at, not to change,** is read-only here.
  If you change an event in it on this computer, your change is kept here and
  each sync says it cannot be sent.
- **A calendar that shows only when its owner is free or busy** does not come,
  because there is nothing in it to show. Each sync says how many were passed
  over.
- **A calendar you hid or unticked in Google Calendar** comes hidden. To show
  it, go to it in the Calendar module's sidebar and press `Enter`. Press
  `Enter` again to hide it. A sync never changes what you chose.
- **A calendar Google stops listing,** because you left it or its owner
  stopped sharing it, is taken off this computer with its events, and the sync
  says so. A change you made in it here and had not sent yet is kept, in no
  calendar, and each sync says it cannot be sent.

None of this has been tried against a real Google account yet.

#### While your key is in Testing

- **The sign-in lasts seven days.** Google's page says a project in Testing "is
  issued a refresh token expiring in 7 days". When it runs out, each sync says
  so and names the button. Press `Alt+T` on the account again. Mail on its app
  password is not affected.
- **Only the test users you added can sign in**, so a second Google account
  needs adding under **Test users** too.

#### If it does not work

- **A sync says this copy of Wixen Mail has no Google sign-in key.** The file is
  missing, or is called `oauth.toml.txt`, or one of its two values is empty.
  Open the settings folder in File Explorer, turn on **File name extensions**
  under **View**, and check the name.
- **Google says access is blocked or denied when you sign in.** The address you
  signed in with is not under **Test users**.
- **Calendars arrive and contacts or tasks do not.** Check that the People API
  and the Google Tasks API are both turned on in step 2.

### How Gmail differs, and what Wixen Mail does about it

Gmail does not have folders. It has labels, and a message can carry several at
once. Over IMAP each label looks like a folder, so one message with three
labels arrives as three copies with three different numbers. Wixen Mail reads
Gmail's own identifier for a message, so it can tell those apart from three
different messages, and search shows the message once rather than once per
label.

**All Mail is not downloaded unless you ask for it.** It holds a copy of every
message in the account, so downloading it alongside your Inbox means fetching
everything twice. Turn it on under Tools, then Folders to Keep Up to Date, if
you want it. All Mail is in that window only when Gmail lists it: Gmail's own
settings, under Labels, have a Show in IMAP box for each label, and a label
with that box off is one Wixen Mail is never told about. When Gmail did not
list All Mail, the window says so under the tree. Until 2026-09-18 this
paragraph said the command was on File; it had been on Action, under This
Folder, since 2026-08-26, and on File before that.

**Deleting moves the message to Bin.** Gmail's own setting for what a deleted
message should do is in Gmail's web settings, under Forwarding and POP/IMAP,
and Wixen Mail cannot see it or change it. Moving to Bin behaves the same
whatever that setting says.

**Two things can only be changed in Gmail's web settings**, because Google
provides no other way:

| What | Where |
|------|-------|
| Whether a label appears to mail apps at all | Gmail settings, Labels, the "Show in IMAP" tick beside each label |
| What happens to a message a mail app deletes | Gmail settings, Forwarding and POP/IMAP |

A label with "Show in IMAP" turned off never reaches Wixen Mail, so it will not
be in the folder list at all. That is Google's choice and there is nothing this
end can do about it.

**Sent mail is saved by Google, not by Wixen Mail.** Every other provider needs
the mail app to file the copy, and Wixen Mail does. On Gmail it does not, because
a second copy would be a duplicate of the one Google already saved.

**Conversations are worked out from the message headers**, not from Gmail's own
conversation grouping, so a conversation here may be split differently from the
same conversation in Gmail's web interface. Gmail does publish its grouping over
IMAP; the library Wixen Mail is built on reads it and provides no way to get at
it. **Until the build after 2026-09-19.** Since then a conversation on a Gmail
account is the one Gmail shows: its own name for each message's conversation is
asked for with the message and decides which messages belong together, whatever
the headers say, and mail already on this computer gets its name once at the
next check. The sentence about the library had outlived the change that let
this program read the server's answer itself. Nobody has yet seen split threads
become one on a real Gmail account; [the user guide](USER_GUIDE.md#thread-view)
says what happens on every other provider.

### Troubleshooting Gmail

**"Authentication failed" error:**
- Use the app password, not your ordinary Google password. Google does not
  accept the ordinary one for mail applications at all.
- If the account is set to browser sign-in and it has been more than a week,
  Google has expired the sign-in. Open the account and sign in again, or switch
  it to an app password.
- If your account is on Google Advanced Protection, or an administrator has
  turned app passwords off for your organisation, browser sign-in is the only
  route open to you.
- Check that IMAP is enabled in Gmail settings
- Wait a few minutes after generating app password

**"Too many simultaneous connections":**
- Google allows fifteen at once per account. Wixen Mail uses two: one for
  working and one that waits for new mail to arrive.
- Close other email clients accessing Gmail
- Wait a few minutes before trying again

**A folder says there is more to fetch and never gets it:**
- Gmail's settings have a limit on how many messages a folder shows to mail
  apps, and it is on by default. Gmail settings, Forwarding and POP/IMAP,
  Folder Size Limits.

**More Help:**
- Official documentation: https://support.google.com/mail/answer/7126229

---

## Outlook.com / Office 365

### Requirements
- Outlook.com, Hotmail, or Office 365 account
- A browser to sign in with. Microsoft has withdrawn plain password sign-in
  for most accounts, so browser sign-in (OAuth) is what Wixen Mail uses here
  by default

### Step-by-Step Setup

#### 1. Configure Wixen Mail

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your Outlook email address. For an Outlook.com, Hotmail, Live or MSN
   address (`user@outlook.com`, `user@hotmail.com`), Wixen Mail recognises the
   domain and fills in the server settings for you:
   - **IMAP Server:** outlook.office365.com, port 993, TLS
   - **SMTP Server:** smtp.office365.com, port 587, TLS

   For a Microsoft 365 address on your organisation's own domain, nothing is
   filled in: type `outlook.office365.com` as the IMAP server and
   `smtp.office365.com`, port 587, as the SMTP server. Wixen Mail then treats
   the account as Microsoft, by its server.
4. The browser sign-in checkbox is on by default for Outlook.com addresses.
   Leave it checked. For a Microsoft 365 address on your own domain it starts
   off: turn it on, because Microsoft accepts no password from a mail
   program.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**. The browser opens immediately to sign in; Wixen Mail never
   sees your password.

### Notes for Office 365

- **Personal accounts:** Use outlook.office365.com servers
- **Business accounts:** Usually use the same servers, but check with IT
- **Signing in:** Use the browser sign-in. Microsoft no longer accepts a
  password or an app password from a mail program, so whether Wixen Mail may
  sign in is your organisation's administrator's decision.
- **A server name of your organisation's own** that points at Microsoft is not
  recognised, because Wixen Mail does not look names up. Use
  `outlook.office365.com` instead.

### Troubleshooting Outlook

**"Authentication failed" for business account:**
- Check with IT department for correct server settings
- Check that the browser sign-in is on; a password will not work
- Ask your administrator whether Wixen Mail is allowed to sign in, and whether
  IMAP is enabled for your organisation

**"Signing in failed" saying neither the server nor the address belongs to
Google or Microsoft:**
- Check the IMAP or POP server. It has to be Google's or Microsoft's own name,
  such as `imap.gmail.com` or `outlook.office365.com`, for the browser sign-in
  to know where to send you. Or turn the browser sign-in off and enter a
  password.

**Exchange vs. Office 365:**
- Office 365 works with these settings
- On-premises Exchange may require different servers
- Check with IT for Exchange server details

**More Help:**
- Official documentation: https://support.microsoft.com/en-us/office/pop-imap-and-smtp-settings-8361e398-8af4-4e97-b147-6c6c4ac95353

---

## Yahoo Mail

### Requirements
- Yahoo Mail account
- App password (required)

### Step-by-Step Setup

#### 1. Generate App Password

1. Log into Yahoo Mail (https://mail.yahoo.com)
2. Click your **profile icon** → **Account Info**
3. Go to **Account Security** in the left sidebar
4. Scroll to **Generate app password**
5. Click **Generate app password**
6. Select **Other App** from the dropdown
7. Enter "Wixen Mail" as the app name
8. Click **Generate**
9. **Important:** Copy the 16-character password shown
   - Save it securely
   - This is a one-time display
10. Click **Done**

#### 2. Enable "Less Secure Apps" (If Needed)

1. In Account Security settings
2. Find "Allow apps that use less secure sign in"
3. Toggle it **On**
4. Confirm the security warning

#### 3. Configure Wixen Mail

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your Yahoo email address (e.g., `user@yahoo.com`). Wixen Mail fills
   in Yahoo's server settings:
   - **IMAP Server:** imap.mail.yahoo.com, port 993, TLS
   - **SMTP Server:** smtp.mail.yahoo.com, port 587, TLS
4. The browser sign-in checkbox is off by default for Yahoo. Paste the app
   password you generated into the password box.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**.

### Troubleshooting Yahoo

**"Authentication failed":**
- Ensure you're using the app password, not regular password
- Check "Allow apps that use less secure sign in" is enabled
- Regenerate app password if needed

**App password not working:**
- Wait 5-10 minutes after generation
- Try regenerating a new app password
- Verify you copied the entire password

**More Help:**
- Official documentation: https://help.yahoo.com/kb/SLN4075.html

---

## iCloud Mail

### Requirements
- iCloud account (@icloud.com, @me.com, or @mac.com)
- 2-Factor Authentication enabled (required for app passwords)
- App-specific password

### Step-by-Step Setup

#### 1. Enable 2-Factor Authentication

1. Go to https://appleid.apple.com
2. Sign in with your Apple ID
3. Go to **Security** section
4. If 2FA not enabled, click **Turn On Two-Factor Authentication**
5. Follow the setup wizard

#### 2. Generate App-Specific Password

1. Still at https://appleid.apple.com
2. In the **Security** section
3. Under **App-Specific Passwords**, click **Generate Password**
4. Enter a label: "Wixen Mail"
5. Click **Create**
6. **Important:** Copy the password shown
   - Format: xxxx-xxxx-xxxx-xxxx
   - Save it securely
7. Click **Done**

#### 3. Configure Wixen Mail

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your iCloud email address (@icloud.com, @me.com, or @mac.com).
   Wixen Mail fills in iCloud's server settings:
   - **IMAP Server:** imap.mail.me.com, port 993, TLS
   - **SMTP Server:** smtp.mail.me.com, port 587, TLS
4. The browser sign-in checkbox is off by default for iCloud. Paste your
   app-specific password into the password box, with or without the dashes.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**.

### Troubleshooting iCloud

**Cannot generate app-specific password:**
- Ensure 2FA is enabled first
- May need to wait after enabling 2FA
- Try from different device/browser

**"Authentication failed":**
- Verify you're using app-specific password
- Try entering password with or without dashes
- Regenerate password if issues persist

**Using multiple Apple email addresses:**
- You can have @icloud.com, @me.com, @mac.com
- All work with same server settings
- Use the specific address you want to receive mail at

**More Help:**
- Official documentation: https://support.apple.com/en-us/HT202304

---

## ProtonMail (via Bridge)

### Requirements
- ProtonMail account (Plus, Professional, or Visionary)
- ProtonMail Bridge application installed
- Bridge must be running

### Step-by-Step Setup

#### 1. Install ProtonMail Bridge

1. Download Bridge from: https://proton.me/mail/bridge
2. Install the application
3. Launch ProtonMail Bridge
4. Sign in with your ProtonMail credentials

#### 2. Configure Bridge

1. In Bridge application, click **+** to add account
2. Sign in with ProtonMail credentials
3. Complete 2FA if enabled
4. Bridge will start running (must stay running)
5. Note the credentials shown:
   - Username (usually your email)
   - Password (auto-generated by Bridge)
   - IMAP port: 1143
   - SMTP port: 1025

#### 3. Configure Wixen Mail

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Enter the settings Bridge showed you by hand. Bridge runs on this
   computer rather than at an address Wixen Mail can recognise, so nothing
   here auto-fills:
   - **IMAP Server:** 127.0.0.1, port 1143, TLS off (the connection never
     leaves this computer)
   - **SMTP Server:** 127.0.0.1, port 1025, TLS off
   - **Username:** as shown in Bridge
   - **Password:** as shown in Bridge, not your ProtonMail password
4. Leave the browser sign-in checkbox unchecked. Bridge handles your
   ProtonMail sign-in on its own.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**.

### Important Notes

- **Bridge must be running** whenever you use Wixen Mail with ProtonMail
- TLS is disabled because connection is local (Bridge handles encryption)
- Password is auto-generated by Bridge, not your ProtonMail password
- Free ProtonMail accounts do not support Bridge

### Troubleshooting ProtonMail

**"Connection failed":**
- Ensure Bridge is running
- Check Bridge is logged in
- Verify ports are correct (1143, 1025)
- Restart Bridge if needed

**"Authentication failed":**
- Use password from Bridge, not ProtonMail password
- Check username matches Bridge exactly
- Try logging out and back into Bridge

**Bridge not working:**
- Check Bridge logs for errors
- Ensure ProtonMail plan supports Bridge
- Contact ProtonMail support

**More Help:**
- Official documentation: https://proton.me/support/protonmail-bridge-install

---

## What differs between providers

Mail servers agree on the basics and differ everywhere else. Wixen Mail asks
each one what it can do when it signs in, and adjusts. You do not have to
configure any of this. It is here so that when a provider behaves differently
you know it is the provider and not a fault.

| What | Where it holds | Where it does not |
|------|----------------|-------------------|
| Sent mail is filed by the provider | Gmail | Everywhere else, so Wixen Mail files the copy |
| Moving a message is one instruction | Most current servers | Older ones copy, then remove, and say so if the second step cannot run |
| One message can be deleted on its own | Servers with UIDPLUS | Older ones can only clear out everything marked deleted at once, which is other people's mail too, so Wixen Mail does not |
| Changes made on another device arrive cheaply | Fastmail, current Dovecot | Gmail and Microsoft 365, where the flags of the messages you hold are read back instead |
| Folders you subscribe to are remembered | Most servers | A few keep no list, and then every folder is downloaded |

### Choosing which folders are downloaded

Tools, then Folders to Keep Up to Date. A tree of the account's folders,
nested the way the folder tree in the main window is, with a check box beside
each folder and each saying how many messages it holds. Space ticks or unticks
the folder you are on, and Right arrow opens a folder that holds others. The
title names the account. Until 2026-09-18 this section said the command was
on File and the window was a ticked list; the command had been on Action,
under This Folder, since 2026-08-26, and on File before that.

This is worth opening on two kinds of account. Gmail, where All Mail holds a
copy of every message and is off by default, and appears in the tree only
when Gmail lists it; the window says so when it did not, and Gmail's own
settings, under Labels, Show in IMAP, decide that. And shared or university
servers, which list every mailbox the account can see, sometimes hundreds of
them.

Your choice is also sent to the server as a subscription, so a folder you turn
off here reads as unwanted in your phone's mail app. If the server will not
accept that, Wixen Mail says so, and your choice still holds here.

---

## Other Providers

For email providers not listed above, you'll need to manually configure the settings.

### Finding Your Provider's Settings

1. **Check provider's documentation:**
   - Search for "[provider name] IMAP settings"
   - Look for "Email client setup" or "Mail app settings"

2. **Common patterns:**
   - IMAP: `imap.provider.com` or `mail.provider.com`
   - SMTP: `smtp.provider.com` or `mail.provider.com`
   - IMAP Port: 993 (TLS/SSL) or 143 (STARTTLS)
   - SMTP Port: 465 (SSL) or 587 (STARTTLS)

3. **Contact support:**
   - Email provider's help desk
   - IT department for business accounts
   - ISP support for ISP-provided email

### Manual Configuration

1. Press `Ctrl+Shift+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. If Wixen Mail does not recognise your provider from the email address,
   enter the settings by hand: IMAP server, port, and TLS; SMTP server,
   port, and TLS; username and password.
4. Type the name you want people to see when your mail arrives.
5. Choose **OK**.

### Common Provider Examples

#### Fastmail
- IMAP: imap.fastmail.com:993 (TLS)
- SMTP: smtp.fastmail.com:465 (SSL)
- App password may be required

#### Zoho Mail
- IMAP: imap.zoho.com:993 (TLS)
- SMTP: smtp.zoho.com:465 (SSL)
- App password required if 2FA enabled

#### GMX
- IMAP: imap.gmx.com:993 (TLS)
- SMTP: mail.gmx.com:587 (STARTTLS)

#### Mail.com
- IMAP: imap.mail.com:993 (TLS)
- SMTP: smtp.mail.com:587 (STARTTLS)

---

## Security Best Practices

### Use App Passwords When Available
- More secure than regular passwords
- Can be revoked without changing main password
- Required for accounts with 2FA

### Enable 2-Factor Authentication
- Adds extra layer of security
- Protects against password theft
- Required for app passwords on most providers

### Keep Passwords Secure
- Don't share passwords
- Use a password manager
- Don't reuse passwords across services

### Check for Suspicious Activity
- Review account security regularly
- Check for unauthorized access
- Revoke unused app passwords

---

## General Setup Tips

### Before Setting Up
1. Know your email address and password
2. Check if provider requires app password
3. Ensure IMAP/SMTP are enabled
4. Have provider's settings handy

### During Setup
1. Let Wixen Mail auto-detect when possible
2. Double-check server addresses
3. Verify port numbers
4. Confirm TLS/SSL settings

### After Setup
1. Test sending and receiving
2. Check all folders load correctly
3. Verify settings are working
4. Note any error messages

### If Problems Occur
1. Check Troubleshooting Guide
2. Verify credentials in webmail
3. Check provider's status page
4. Contact provider support if needed

---

## Need More Help?

- **User Guide:** Complete feature documentation
- **Keyboard Shortcuts:** Reference for all keyboard commands
- **Troubleshooting Guide:** Solutions for common issues
- **Provider Support:** Contact your email provider directly for account-specific issues

Remember: Most setup issues are related to credentials, app passwords, or provider settings. Double-check these first before troubleshooting further.

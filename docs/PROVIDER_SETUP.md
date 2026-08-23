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

One account does everything it can. There is no second account to set up for
contacts, for the calendar or for tasks: they all use the sign-in you already
gave for the mail.

What you get depends on what kind of account it is.

| | Gmail | Outlook, Office 365 | Any other IMAP or POP account |
|---|---|---|---|
| Mail | Yes | Yes | Yes |
| Contacts | Yes, both ways | Yes, both ways | No |
| Calendar | Yes, both ways | Yes, both ways | No |
| Tasks | Yes, both ways | Yes, both ways | No |
| Notes | No | No | No |
| Reminders | No | No | No |

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
notes through OneNote, and that is not built: a OneNote page is a formatted
document inside a section inside a notebook, and a note here is a title and some
text, so somebody has to decide what happens to the difference before any of it
is written.

**Reminders.** Neither provider has a reminder that exists on its own. Outlook
and Exchange make a reminder a property of an appointment or a task, and Google
folded its Reminders into Tasks in 2023. There is nothing on the other side to
sync one to, so this one is not going to change.

### If you signed in before tasks synced both ways

Sending tasks up needs more permission than reading them did, and permission is
granted once at sign-in. An account you set up before this version will keep
syncing mail, contacts, the calendar and tasks downwards, and your changes will
sit here waiting.

Fix it by signing in again: open the account, switch the browser sign-in off and
back on, and approve the list of permissions when the browser shows it. The
waiting changes go up on the next sync.

The permission is Google's "See, edit, create and delete your tasks" or
Microsoft's Tasks.ReadWrite, in place of the read-only version. Nothing else
about what Wixen Mail asks for has changed.

---

## Choosing a sign-in method

Wixen Mail signs in to a mailbox one of two ways. The account dialog has a
checkbox, "Sign in with the provider in a browser (OAuth)", and it is set to
whichever usually works for the address you typed. You can change it.

### App password

A password your provider generates for one application, which you can revoke on
its own without changing the password you sign in with everywhere else. This is
the default for Gmail and for any provider we do not recognise.

It works today, it does not expire, and it does not depend on Wixen Mail being
registered with anybody. You need two-step verification turned on with your
provider before they will give you one.

Your ordinary password will not work. Google stopped accepting it for mail
applications, and Microsoft has stopped for most accounts. Typing it produces
"authentication failed", which reads like a typo and sends people round the loop
again, so the account dialog says this next to the password box.

### Browser sign-in (OAuth)

You are sent to the provider's own page, you sign in there, and Wixen Mail never
sees your password. This is the default for Outlook.com addresses, because
Microsoft has withdrawn password sign-in more widely than Google has.

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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
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

### How Gmail differs, and what Wixen Mail does about it

Gmail does not have folders. It has labels, and a message can carry several at
once. Over IMAP each label looks like a folder, so one message with three
labels arrives as three copies with three different numbers. Wixen Mail reads
Gmail's own identifier for a message, so it can tell those apart from three
different messages, and search shows the message once rather than once per
label.

**All Mail is not downloaded unless you ask for it.** It holds a copy of every
message in the account, so downloading it alongside your Inbox means fetching
everything twice. Turn it on under File, then Folders to Keep Up to Date, if
you want it.

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
it.

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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
2. Choose **Add Account**.
3. Type your Outlook email address (`user@outlook.com`, `user@hotmail.com`,
   or your work address). Wixen Mail recognises the domain and fills in the
   server settings for you:
   - **IMAP Server:** outlook.office365.com, port 993, TLS
   - **SMTP Server:** smtp.office365.com, port 587, TLS
4. The browser sign-in checkbox is on by default for Outlook.com addresses.
   Leave it checked.
5. Type the name you want people to see when your mail arrives.
6. Choose **OK**. The browser opens immediately to sign in; Wixen Mail never
   sees your password.

### Notes for Office 365

- **Personal accounts:** Use outlook.office365.com servers
- **Business accounts:** Usually use the same servers, but check with IT
- **Multi-factor authentication:** May require app password for business accounts

### Troubleshooting Outlook

**"Authentication failed" for business account:**
- Check with IT department for correct server settings
- May need app password if modern auth is disabled
- Verify IMAP is enabled for your organization

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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
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

File, then Folders to Keep Up to Date. A ticked list, one row per folder,
saying how many messages are in each. Space ticks the row you are on.

This is worth opening on two kinds of account. Gmail, where All Mail holds a
copy of every message and is off by default. And shared or university servers,
which list every mailbox the account can see, sometimes hundreds of them.

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

1. Press `Ctrl+A`, or open the Tools menu and choose Account Manager.
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

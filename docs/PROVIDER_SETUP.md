# Email Provider Setup Guides

Quick setup instructions for popular email providers with Wixen Mail.

## Table of Contents

- [Gmail](#gmail)
- [Outlook.com / Office 365](#outlookcom--office-365)
- [Yahoo Mail](#yahoo-mail)
- [iCloud Mail](#icloud-mail)
- [ProtonMail (via Bridge)](#protonmail-via-bridge)
- [Other Providers](#other-providers)

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

#### 2. Generate App Password (If Using 2FA)

1. Go to your Google Account (https://myaccount.google.com)
2. Navigate to **Security**
3. Under "Signing in to Google," select **App passwords**
4. You may need to sign in again
5. At the bottom, click **Select app** → **Mail**
6. Click **Select device** → **Other (Custom name)**
7. Enter "Wixen Mail" as the name
8. Click **Generate**
9. **Important:** Copy the 16-character password shown
   - Save it securely
   - You won't be able to see it again
10. Click **Done**

#### 3. Configure Wixen Mail

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Enter your Gmail address (e.g., `user@gmail.com`)
4. The settings should auto-fill:
   - **Provider:** Gmail
   - **IMAP Server:** imap.gmail.com
   - **IMAP Port:** 993
   - **Use TLS/SSL:** ✓ Checked
   - **SMTP Server:** smtp.gmail.com
   - **SMTP Port:** 587
   - **Use TLS/SSL:** ✓ Checked
5. Enter **Username:** Your full Gmail address
6. Enter **Password:** Your 16-character app password (or regular password if no 2FA)
7. Click **Connect**

### Troubleshooting Gmail

**"Authentication failed" error:**
- Make sure you're using app password, not regular password (if 2FA enabled)
- Check that IMAP is enabled in Gmail settings
- Wait a few minutes after generating app password

**"Too many simultaneous connections":**
- Close other email clients accessing Gmail
- Wait a few minutes before trying again

**More Help:**
- Official documentation: https://support.google.com/mail/answer/7126229

---

## Outlook.com / Office 365

### Requirements
- Outlook.com, Hotmail, or Office 365 account
- Regular account password (app password not usually required)

### Step-by-Step Setup

#### 1. Configure Wixen Mail

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Enter your Outlook email address
   - Examples: `user@outlook.com`, `user@hotmail.com`, `user@yourcompany.com`
4. The settings should auto-fill:
   - **Provider:** Outlook.com / Office 365
   - **IMAP Server:** outlook.office365.com
   - **IMAP Port:** 993
   - **Use TLS/SSL:** ✓ Checked
   - **SMTP Server:** smtp.office365.com
   - **SMTP Port:** 587
   - **Use TLS/SSL:** ✓ Checked
5. Enter **Username:** Your full email address
6. Enter **Password:** Your regular account password
7. Click **Connect**

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

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Enter your Yahoo email address (e.g., `user@yahoo.com`)
4. The settings should auto-fill:
   - **Provider:** Yahoo Mail
   - **IMAP Server:** imap.mail.yahoo.com
   - **IMAP Port:** 993
   - **Use TLS/SSL:** ✓ Checked
   - **SMTP Server:** smtp.mail.yahoo.com
   - **SMTP Port:** 587
   - **Use TLS/SSL:** ✓ Checked
5. Enter **Username:** Your full Yahoo email address
6. Enter **Password:** Your generated app password (not your regular password)
7. Click **Connect**

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

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Enter your iCloud email address
   - Can be @icloud.com, @me.com, or @mac.com
4. The settings should auto-fill:
   - **Provider:** iCloud Mail
   - **IMAP Server:** imap.mail.me.com
   - **IMAP Port:** 993
   - **Use TLS/SSL:** ✓ Checked
   - **SMTP Server:** smtp.mail.me.com
   - **SMTP Port:** 587
   - **Use TLS/SSL:** ✓ Checked
5. Enter **Username:** Your full iCloud email address
6. Enter **Password:** Your app-specific password (with or without dashes)
7. Click **Connect**

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

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Enter your ProtonMail address (e.g., `user@protonmail.com`)
4. The settings should auto-fill:
   - **Provider:** ProtonMail (Bridge required)
   - **IMAP Server:** 127.0.0.1
   - **IMAP Port:** 1143
   - **Use TLS/SSL:** ✗ Unchecked (local connection)
   - **SMTP Server:** 127.0.0.1
   - **SMTP Port:** 1025
   - **Use TLS/SSL:** ✗ Unchecked (local connection)
5. Enter **Username:** As shown in Bridge
6. Enter **Password:** As shown in Bridge (not your ProtonMail password)
7. Click **Connect**

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

1. Open Wixen Mail
2. Click **File → Connect to Server**
3. Select **Manual Configuration** from provider dropdown
4. Enter your settings:
   - IMAP Server, Port, TLS/SSL
   - SMTP Server, Port, TLS/SSL
   - Username and Password
5. Click **Connect**

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

# Wixen Mail User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Account Setup](#account-setup)
3. [Email Provider Configuration](#email-provider-configuration)
4. [Reading and Managing Email](#reading-and-managing-email)
5. [Composing Email](#composing-email)
6. [Search Functionality](#search-functionality)
7. [Thread View](#thread-view)
8. [Attachments](#attachments)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Accessibility Features](#accessibility-features)
11. [Troubleshooting](#troubleshooting)

## Getting Started

Wixen Mail is a fully accessible email client designed to work seamlessly with screen readers (NVDA, JAWS, Windows Narrator). The application follows WCAG 2.1 Level AA accessibility standards.

### System Requirements
- Windows 10 or later
- Internet connection for email access
- Optional: Screen reader (NVDA, JAWS, or Narrator) for accessibility features

### First Launch
When you first launch Wixen Mail, you'll need to configure an email account to get started.

## Account Setup

### Quick Setup with Popular Providers

Wixen Mail supports automatic configuration for these popular email providers:

#### Gmail
1. Click **File → Connect to Server** or press `Ctrl+O`
2. Enter your Gmail address (e.g., `user@gmail.com`)
3. The app will automatically detect Gmail and fill in server settings
4. Enter your username (usually your full email address)
5. Enter your **app password** (not your regular Gmail password)
   - Create an app password at: https://myaccount.google.com/apppasswords
6. Click **Connect**

**Important:** Gmail requires an app password if you have 2-factor authentication enabled (recommended).

#### Outlook.com / Office 365
1. Click **File → Connect to Server**
2. Enter your Outlook/Office 365 email address
3. Settings auto-fill for Outlook
4. Enter your username and password
5. Click **Connect**

**Note:** Works with both personal and business Office 365 accounts.

#### Yahoo Mail
1. Click **File → Connect to Server**
2. Enter your Yahoo email address
3. Settings auto-fill for Yahoo
4. Enter your username and **app password**
   - Generate at: https://login.yahoo.com/account/security
5. Click **Connect**

#### iCloud Mail
1. Click **File → Connect to Server**
2. Enter your iCloud email address (@icloud.com, @me.com, or @mac.com)
3. Settings auto-fill for iCloud
4. Enter your username and **app-specific password**
   - Generate at: https://appleid.apple.com
5. Click **Connect**

#### ProtonMail (via Bridge)
1. Install and start ProtonMail Bridge on your computer
2. Click **File → Connect to Server**
3. Select "ProtonMail (Bridge required)" from the provider dropdown
4. Enter your Bridge username and password
5. Click **Connect**

**Note:** ProtonMail requires the Bridge application running locally.

### Manual Configuration

For other email providers:

1. Click **File → Connect to Server**
2. Select "Manual Configuration" from the provider dropdown
3. Enter the following information:

**IMAP Settings (Incoming Mail):**
- Server: Your IMAP server address (e.g., `imap.example.com`)
- Port: Usually 993 for TLS/SSL
- Use TLS/SSL: Check this box (recommended)

**SMTP Settings (Outgoing Mail):**
- Server: Your SMTP server address (e.g., `smtp.example.com`)
- Port: Usually 465 (SSL) or 587 (STARTTLS)
- Use TLS/SSL: Check this box (recommended)

**Credentials:**
- Username: Your email address or username
- Password: Your email password

4. Click **Connect**

### Multiple Accounts

Wixen Mail supports managing multiple email accounts.

- Open **Tools → Manage Accounts** (`Ctrl+M`) to add/edit/delete accounts.
- Switch active account from the toolbar account dropdown (📧).
- Keyboard shortcuts:
  - `Ctrl+1` - Switch to first enabled account
  - `Ctrl+2` - Switch to second enabled account
  - `Ctrl+3` - Switch to third enabled account

### Message Rules (Phase 7)

Use **Tools → Manage Rules** (`Ctrl+Shift+E`) to create accessibility-friendly message rules.

Each rule can:
- Match on fields: `subject`, `from`, `to`, `cc`, `date`, `message_id`, `body_plain`, `body_html`, `read`, `starred`, `deleted`
- Use match types: `contains`, `not_contains`, `equals`, `not_equals`, `starts_with`, `ends_with`, `is_empty`, `is_not_empty`, `is_true`, `is_false`, `regex`
- Choose case-sensitive or case-insensitive matching
- Perform actions: `mark_as_read`, `mark_as_unread`, `star`, `unstar`, `delete`, `move_to_folder`, `add_tag`

### Contacts (Phase 8)

Use **Tools → Manage Contacts** (`Ctrl+Shift+C`) to manage an account-specific address book.

Features:
- Create/edit/delete contacts
- Mark favorite contacts
- Search by name or email
- Recipient autocomplete suggestions while composing messages
- Extended provider-ready fields (phone, company, title, website, address, birthday)
- Photo/avatar support (URL or embedded uploaded image)
- Automatic contact import from message history/provider account activity
- vCard import and export

### OAuth 2.0 (Phase 9)

Use **Tools → OAuth 2.0 Manager** (`Ctrl+Shift+O`) to manage provider tokens.
The OAuth manager is shown only when at least one configured account requires OAuth (for example Gmail or Outlook).

Capabilities:
- Generate provider authorization URL (Gmail/Outlook presets)
- Exchange authorization code for token set
- Refresh or revoke stored tokens
- View token expiry status for selected account/provider

## Reading and Managing Email

### Three-Pane Layout

Wixen Mail uses a classic three-pane layout:

```
┌─────────────┬─────────────────┬─────────────────┐
│  FOLDERS    │  MESSAGE LIST   │  PREVIEW PANE   │
│             │                 │                 │
│ 📁 INBOX    │ ⭐● Subject     │ Message body    │
│ 📁 Sent     │   From: ...     │ appears here    │
│ 📁 Drafts   │   Date: ...     │                 │
│ 📁 Trash    │                 │ Attachments     │
│             │ 📎 Subject      │ listed below    │
│             │   From: ...     │                 │
└─────────────┴─────────────────┴─────────────────┘
```

### Navigating Between Panes

- **Keyboard:** Press `F6` to cycle through panes
- **Mouse:** Click on the desired pane

### Message Indicators

- **⭐** - Starred/flagged message
- **●** - Unread message
- **📎** - Has attachments
- **↳** - Reply in a thread (when thread view is enabled)
- **📧** - Thread parent message

### Message Actions

**Using Context Menu (Right-Click):**
1. Right-click on a message in the message list
2. Select an action:
   - **Reply** - Reply to the sender
   - **Forward** - Forward the message to someone else
   - **Delete** - Move to trash
   - **Toggle Star** - Add or remove star/flag
   - **Mark as Unread** - Mark message as unread

**Using Keyboard Shortcuts:**
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply all
- `Ctrl+L` - Forward
- `Delete` - Delete message
- `S` - Star/flag message
- `Space` - Toggle read/unread

## Composing Email

### Creating a New Message

1. Click **File → New Message** or press `Ctrl+N`
2. Enter recipient(s) in the **To:** field
3. Optionally add CC and BCC recipients
4. Enter a subject
5. Type your message in the body field
6. Click **Send** or press `Ctrl+Enter`

### Saving Drafts

- Click **Save Draft** button or press `Ctrl+S`
- The draft will be saved to your Drafts folder
- You can return to edit it later

### Replying to Messages

1. Select a message in the message list
2. Press `Ctrl+R` or right-click and select **Reply**
3. The composition window opens with:
   - Recipient pre-filled
   - Subject pre-filled with "Re: [original subject]"
4. Type your reply and send

### Forwarding Messages

1. Select a message
2. Press `Ctrl+L` or right-click and select **Forward**
3. Enter recipient(s)
4. Add any additional comments
5. Send the message

## Search Functionality

### Opening Search

- Click **Edit → Search** or press `Ctrl+F`
- The search dialog will open

### Searching for Messages

1. Enter your search terms in the search field
2. Click **Search** button or press `Enter`
3. Results appear below the search field
4. Click on a result to view the message

### Search Tips

- Search looks through message subjects, senders, and content
- Search is case-insensitive
- Use specific terms for better results

## Thread View

Thread view groups related messages together in conversations, making it easier to follow email discussions.

### Enabling Thread View

**Method 1: Header Toggle**
- Look for the **Thread View** checkbox in the message list header
- Click to toggle thread view on/off

**Method 2: View Menu**
- Click **View → Thread View**
- Check or uncheck to toggle

### Understanding Thread View

When thread view is enabled:
- **📧** indicates the first message in a thread
- **↳** indicates replies in the thread
- Replies are indented to show hierarchy
- Messages are grouped by conversation

### Benefits of Thread View

- See all related messages together
- Understand conversation context
- Reduce clutter by grouping replies

## Attachments

### Viewing Attachments

When a message has attachments:
1. The message shows a **📎** icon in the message list
2. Select the message to view details
3. Attachments appear below the message body in the preview pane

### Attachment Information

Each attachment shows:
- **File icon** - Visual indicator of file type
  - 🖼 Images
  - 📄 PDF documents
  - 📝 Word documents
  - 📊 Spreadsheets
  - 🎥 Videos
  - 🎵 Audio files
  - 📦 Archives
- **Filename**
- **MIME type** (e.g., image/jpeg, application/pdf)
- **File size** in bytes

### Saving Attachments

1. Find the attachment in the preview pane
2. Click the **Save** button (💾)
3. Choose a location to save the file
4. The file will be downloaded

**Keyboard Shortcut:** Tab to the Save button and press `Enter`

## Keyboard Shortcuts

### Application Control
- `Ctrl+Q` - Quit application
- `Ctrl+,` - Open settings
- `F1` - Help documentation
- `Esc` - Close dialogs

### Window Navigation
- `F6` - Cycle through panes (folders → messages → preview)
- `Tab` - Navigate within pane
- `Arrow Keys` - Navigate lists
- `Enter` - Activate selected item

### Message Actions
- `Ctrl+N` - New message
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply all
- `Ctrl+L` - Forward
- `Delete` - Delete message
- `S` - Star/flag message
- `Space` - Toggle read/unread

### Navigation
- `N` - Next unread message
- `P` - Previous unread message
- `Up/Down` - Navigate messages
- `Home/End` - First/last message

### Composition
- `Ctrl+Enter` - Send message
- `Ctrl+S` - Save draft

### Search & Mail
- `Ctrl+F` - Open search
- `F5` - Refresh folder
- `F9` - Check mail

## Accessibility Features

Wixen Mail is designed to be fully accessible with screen readers and keyboard navigation.

### Screen Reader Support

**Supported Screen Readers:**
- NVDA (free, recommended for testing)
- JAWS (commercial)
- Windows Narrator (built-in)

**Announcements:**
Wixen Mail announces:
- New messages received
- Message selection changes
- Folder changes with unread counts
- Search results
- Successful actions (sent, deleted, etc.)
- Errors with helpful recovery tips

### Keyboard Accessibility

**Every function in Wixen Mail can be accessed via keyboard:**
- All buttons are keyboard accessible
- All menus support keyboard navigation
- All dialogs can be navigated with Tab/Shift+Tab
- Context menus can be opened with Shift+F10 or Menu key

### Focus Indicators

- Clear visual focus indicators on all interactive elements
- High contrast mode support
- Adjustable font sizes (in Settings)

### ARIA Labels

All UI elements have proper ARIA labels and roles for screen reader compatibility.

## Troubleshooting

### Connection Issues

**Problem:** Cannot connect to email server

**Solutions:**
1. Check your internet connection
2. Verify server address and port are correct
3. Ensure TLS/SSL settings match your provider's requirements
4. Check if firewall is blocking the connection
5. Try disabling antivirus temporarily to test

### Authentication Issues

**Problem:** Username or password not accepted

**Solutions:**
1. Verify your username is correct (usually your full email address)
2. Check password is correct (case-sensitive)
3. For Gmail/Yahoo/iCloud: Use an **app password**, not your regular password
4. Ensure 2FA is properly configured
5. Check if IMAP/SMTP is enabled for your account
6. Contact your email provider if issues persist

### Missing Folders

**Problem:** Folders not appearing after connection

**Solutions:**
1. Click **View → Refresh** or press `F5`
2. Try disconnecting and reconnecting
3. Check if folders exist in webmail interface
4. Some providers may use different folder names

### Messages Not Loading

**Problem:** Message list is empty

**Solutions:**
1. Verify folder is selected in the folder pane
2. Check if folder actually contains messages
3. Try refreshing the folder (F5)
4. Check error messages for connection issues

### Slow Performance

**Problem:** Application is slow or unresponsive

**Solutions:**
1. Large message lists may take time to load
2. Consider archiving old messages
3. Close other resource-intensive applications
4. Restart Wixen Mail
5. Check system resources (RAM, CPU)

### Attachment Issues

**Problem:** Cannot save attachments

**Solutions:**
1. Ensure you have write permissions to the save location
2. Check available disk space
3. Try a different save location
4. Verify the attachment downloaded properly

### Screen Reader Issues

**Problem:** Screen reader not announcing changes

**Solutions:**
1. Ensure screen reader is running before starting Wixen Mail
2. Try restarting both the screen reader and Wixen Mail
3. Check screen reader verbosity settings
4. Update to latest version of screen reader
5. Verify AccessKit support is enabled (it is by default)

## Getting Help

### In-App Help

- Press `F1` to open documentation
- Click **Help → Documentation** in the menu bar
- View keyboard shortcuts: **Help → Keyboard Shortcuts**

### Provider-Specific Help

When configuring an account, look for the provider documentation link (ℹ) in the account configuration dialog. This links to official setup guides for:
- Gmail
- Outlook/Office 365
- Yahoo Mail
- iCloud
- ProtonMail Bridge

### Report Issues

If you encounter issues not covered in this guide:
1. Check the application logs for details
2. Note any error messages you receive
3. Report issues on the GitHub repository

## Tips for Best Experience

1. **Use app passwords** for providers that support them (Gmail, Yahoo, iCloud)
2. **Enable thread view** to better organize conversations
3. **Use keyboard shortcuts** for faster navigation
4. **Star important messages** for quick access later
5. **Use search** to quickly find messages
6. **Right-click for quick actions** on messages
7. **Keep folders organized** by archiving old messages
8. **Check for updates** regularly for new features and fixes

## Conclusion

Wixen Mail is designed to be a powerful yet accessible email client. Take time to explore the features and customize your experience through the Settings window.

For the best experience:
- Learn the keyboard shortcuts
- Enable features that help your workflow (thread view, etc.)
- Keep your email organized with folders and stars
- Use search to quickly find what you need

Thank you for using Wixen Mail!

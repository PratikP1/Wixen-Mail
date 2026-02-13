# Wixen Mail Troubleshooting Guide

Comprehensive solutions for common issues with Wixen Mail.

## Table of Contents

1. [Connection Issues](#connection-issues)
2. [Authentication Problems](#authentication-problems)
3. [Email Provider Specific Issues](#email-provider-specific-issues)
4. [Message and Folder Issues](#message-and-folder-issues)
5. [Performance Issues](#performance-issues)
6. [Accessibility Issues](#accessibility-issues)
7. [Attachment Issues](#attachment-issues)
8. [Composition Issues](#composition-issues)
9. [Search Problems](#search-problems)
10. [General Issues](#general-issues)

## Connection Issues

### Cannot Connect to Email Server

**Symptoms:**
- Error message: "Connection failed"
- "Unable to connect to server"
- Timeout errors

**Solutions:**

1. **Check Internet Connection**
   - Open a web browser and visit a website
   - Try pinging the mail server: `ping imap.gmail.com`
   - Check if other network services work

2. **Verify Server Settings**
   - IMAP Server: Check the address is correct
   - IMAP Port: Usually 993 for TLS/SSL
   - SMTP Server: Verify outgoing server address
   - SMTP Port: Usually 465 (SSL) or 587 (STARTTLS)

3. **Check TLS/SSL Settings**
   - Ensure "Use TLS/SSL" is checked
   - Try with and without TLS to see if it helps
   - Some servers require STARTTLS on port 587

4. **Firewall and Antivirus**
   - Check Windows Firewall settings
   - Temporarily disable antivirus to test
   - Add Wixen Mail to firewall exceptions
   - Check corporate firewall if on work network

5. **Network Configuration**
   - If using VPN, try without it
   - Check proxy settings if applicable
   - Try from a different network to isolate issue

6. **DNS Issues**
   - Flush DNS cache: `ipconfig /flushdns`
   - Try using IP address instead of hostname
   - Check with IT if on corporate network

### Connection Drops Frequently

**Symptoms:**
- Connected but keeps disconnecting
- "Connection lost" messages

**Solutions:**

1. **Network Stability**
   - Check Wi-Fi signal strength
   - Try wired connection if available
   - Contact ISP if persistent

2. **Keep-Alive Settings**
   - Server may be closing idle connections
   - Try checking mail more frequently (F9)
   - This will be addressed in future updates

3. **Server-Side Issues**
   - Some providers limit connection time
   - Check provider's status page
   - Contact provider support if ongoing

## Authentication Problems

### Username or Password Not Accepted

**Symptoms:**
- "Authentication failed"
- "Invalid credentials"
- "Login denied"

**Solutions:**

1. **Verify Credentials**
   - Check username is correct (usually full email address)
   - Verify password (case-sensitive)
   - Try logging into webmail with same credentials
   - Make sure Caps Lock is off

2. **App Passwords Required**
   
   **Gmail:**
   - Requires app password if 2FA enabled
   - Create at: https://myaccount.google.com/apppasswords
   - Use the 16-character app password, not your regular password
   
   **Yahoo:**
   - Requires app password
   - Generate at: https://login.yahoo.com/account/security
   - Enable "Allow apps that use less secure sign in"
   
   **iCloud:**
   - Requires app-specific password
   - Generate at: https://appleid.apple.com (Security section)
   - Must have 2FA enabled first

3. **IMAP/SMTP Access**
   - Some providers disable IMAP by default
   - Check webmail settings to enable IMAP access
   
   **Gmail:** Settings → Forwarding and POP/IMAP → Enable IMAP
   **Outlook:** Should be enabled by default
   **Yahoo:** Settings → Security → Enable IMAP

4. **2-Factor Authentication**
   - If you have 2FA, you MUST use app passwords
   - Cannot use regular password with 2FA
   - Follow provider instructions for app passwords

### Account Locked

**Symptoms:**
- "Account locked" or "Too many failed attempts"

**Solutions:**

1. **Wait and Retry**
   - Wait 15-30 minutes
   - Account may auto-unlock
   - Don't keep retrying immediately

2. **Reset via Webmail**
   - Log into provider's webmail
   - Follow account recovery process
   - Check for security alerts

3. **Contact Provider**
   - May need to verify identity
   - Follow their unlock process
   - Update password if required

## Email Provider Specific Issues

### Gmail Issues

**Common Problems:**

1. **"Less secure app" blocking**
   - **Solution:** Use app password instead
   - Gmail no longer allows "less secure apps"
   - App passwords required for IMAP access

2. **IMAP not enabled**
   - **Solution:** Settings → Forwarding and POP/IMAP → Enable IMAP
   - May take a few minutes to take effect

3. **Too many connections**
   - Gmail limits simultaneous IMAP connections
   - Close other email clients
   - Wait a few minutes and retry

### Outlook/Office 365 Issues

**Common Problems:**

1. **Modern authentication required**
   - Basic auth being phased out
   - Most accounts should work with username/password
   - OAuth support coming in future update

2. **Wrong server for Exchange**
   - Use outlook.office365.com, not your company's server
   - For company email, verify correct server with IT

3. **Multi-factor auth**
   - May need app password
   - Check with IT for corporate accounts

### Yahoo Issues

**Common Problems:**

1. **"Less secure app" blocking**
   - **Solution:** Security settings → Enable "Allow apps that use less secure sign in"
   - Then generate app password

2. **App password required**
   - Don't use regular password
   - Generate specific app password for Wixen Mail

### iCloud Issues

**Common Problems:**

1. **App-specific password required**
   - iCloud requires 2FA first
   - Then generate app-specific password
   - Use the 16-character password format

2. **Wrong email domain**
   - Can use @icloud.com, @me.com, or @mac.com
   - All should work with iCloud settings

### ProtonMail Issues

**Common Problems:**

1. **Bridge not running**
   - ProtonMail requires Bridge application
   - Download from: https://proton.me/mail/bridge
   - Must keep Bridge running while using Wixen Mail

2. **Bridge not configured**
   - Configure Bridge before connecting Wixen Mail
   - Use localhost (127.0.0.1) as server
   - Ports: IMAP 1143, SMTP 1025

## Message and Folder Issues

### No Folders Showing

**Solutions:**

1. **Refresh Folder List**
   - Press F5 or View → Refresh
   - Folders load after successful connection

2. **Connection Not Established**
   - Check connection status (top right)
   - Reconnect if disconnected
   - Verify credentials are correct

3. **Provider-Specific Folders**
   - Some providers use different folder names
   - Check webmail to see actual folder structure
   - Non-standard folders may appear different

### Messages Not Loading

**Solutions:**

1. **Select a Folder**
   - Click on a folder in the folder pane
   - Messages load when folder is selected

2. **Empty Folder**
   - Check in webmail if folder has messages
   - Some folders may legitimately be empty

3. **Refresh Folder**
   - Press F5 to reload current folder
   - May help with sync issues

4. **Large Mailbox**
   - Very large folders may take time to load
   - Be patient for initial load
   - Subsequent loads will be faster

### Message Preview Not Showing

**Solutions:**

1. **Select a Message**
   - Click on a message in the message list
   - Preview loads when message is selected

2. **Message Encoding Issues**
   - Some messages may have encoding problems
   - Check in webmail if same issue
   - Report if consistently problematic

3. **Large Message**
   - Very large messages may take time
   - Watch status bar for loading indication

## Performance Issues

### Application Slow to Start

**Solutions:**

1. **System Resources**
   - Close other resource-intensive applications
   - Check Task Manager for CPU/RAM usage
   - Restart computer if low on memory

2. **First Launch**
   - First launch may take longer
   - Subsequent launches should be faster

3. **Disk Space**
   - Ensure adequate free disk space
   - Clear temporary files if needed

### Slow Message Loading

**Solutions:**

1. **Large Message List**
   - Folders with many messages take longer
   - Consider archiving old messages
   - Use search to find specific messages

2. **Network Speed**
   - Check internet connection speed
   - Large messages download slower on slow connections

3. **Server Response**
   - Some providers respond slower than others
   - Peak times may be slower
   - Can't be fixed client-side

### UI Feels Sluggish

**Solutions:**

1. **Graphics Driver**
   - Update graphics drivers
   - Check for Windows updates

2. **System Performance**
   - Close unnecessary applications
   - Check antivirus isn't scanning heavily
   - Restart if running for long time

3. **Message List Size**
   - Very long message lists can impact performance
   - Archive old messages to improve speed
   - Future updates will improve virtualization

## Accessibility Issues

### Screen Reader Not Announcing

**Solutions:**

1. **Screen Reader Running**
   - Ensure NVDA/JAWS/Narrator is running
   - Start screen reader before Wixen Mail

2. **Restart Both Applications**
   - Close Wixen Mail
   - Restart screen reader
   - Launch Wixen Mail again

3. **Check Screen Reader Settings**
   - Verify verbosity level is appropriate
   - Check if application announcements enabled
   - Try different verbosity modes

4. **Focus Mode**
   - Some screen readers have browse vs. focus mode
   - Try switching modes with `Insert+Space` (NVDA)
   - Ensure in appropriate mode for context

### Focus Not Visible

**Solutions:**

1. **High Contrast Mode**
   - Enable Windows High Contrast
   - Windows Settings → Ease of Access → High Contrast

2. **Focus Indicators**
   - Should be visible by default
   - Report if consistently invisible

3. **Graphics Settings**
   - Check display scaling
   - Try different scaling percentages

### Keyboard Navigation Not Working

**Solutions:**

1. **Check Keyboard Layout**
   - Verify keyboard language/layout
   - Try with US keyboard layout

2. **Conflicting Software**
   - Some keyboard software may intercept keys
   - Disable keyboard macro software temporarily

3. **Caps Lock / Num Lock**
   - Check if modifier keys are stuck
   - Press Shift 5 times to check Sticky Keys

## Attachment Issues

### Cannot Save Attachments

**Solutions:**

1. **Permissions**
   - Check you have write permission to save location
   - Try saving to different folder (e.g., Documents)
   - Run as administrator if needed

2. **Disk Space**
   - Ensure enough free disk space
   - Check target drive capacity

3. **File Name**
   - Some characters invalid in filenames
   - Try different save location first

4. **Antivirus Blocking**
   - Antivirus may quarantine attachment
   - Check antivirus logs
   - Add exception if safe file

### Attachments Not Showing

**Solutions:**

1. **No Attachments**
   - Verify message has attachments in webmail
   - 📎 icon shows attachments in message list

2. **Message Not Loaded**
   - Click on message to load full content
   - Attachments show in preview pane

3. **Embedded Images**
   - Some inline images may not appear
   - Plain text view may not show images

## Composition Issues

### Cannot Send Message

**Solutions:**

1. **SMTP Not Configured**
   - Check SMTP settings in account configuration
   - Verify SMTP server address and port
   - Ensure SMTP authentication enabled

2. **Recipients Invalid**
   - Check email addresses are valid format
   - Remove any extra spaces
   - Separate multiple addresses with commas

3. **Size Limit**
   - Message may be too large
   - Check provider's size limit (usually 25MB)
   - Remove or compress large attachments

4. **Not Connected**
   - Verify connection status
   - Reconnect if disconnected

### Draft Not Saving

**Solutions:**

1. **Not Connected**
   - Must be connected to save drafts
   - Drafts save to server, not locally
   - Check connection status

2. **Draft Folder Missing**
   - Some providers may not have Drafts folder
   - Check webmail for draft folder name

## Search Problems

### Search Returns No Results

**Solutions:**

1. **Check Search Terms**
   - Verify spelling of search terms
   - Try broader search terms
   - Search is case-insensitive

2. **Not Connected**
   - Must be connected to search
   - Search queries server, not local cache

3. **Search Not Indexed**
   - Some providers may have search limitations
   - Try searching from specific folder

### Search Very Slow

**Solutions:**

1. **Large Mailbox**
   - Searching large mailboxes takes time
   - Be patient for results
   - Consider narrowing search scope

2. **Server Load**
   - Provider's server may be busy
   - Try again later
   - Peak times may be slower

## General Issues

### Application Won't Start

**Solutions:**

1. **Check System Requirements**
   - Windows 10 or later required
   - Ensure all Windows updates installed

2. **Corrupt Installation**
   - Reinstall Wixen Mail
   - Check install log for errors

3. **Conflicting Software**
   - Check for conflicting security software
   - Try in Safe Mode to isolate

### Application Crashes

**Solutions:**

1. **Check Logs**
   - Look for error messages
   - Note what you were doing when crashed

2. **Update Windows**
   - Ensure Windows is up to date
   - Install all pending updates

3. **Graphics Drivers**
   - Update to latest graphics drivers
   - Especially for Intel, AMD, or NVIDIA GPUs

4. **Reinstall**
   - Complete uninstall
   - Clean reinstall of Wixen Mail

### Settings Not Saving

**Solutions:**

1. **Permissions**
   - Check app has permission to write config files
   - Check Documents folder permissions

2. **Read-Only Config**
   - Config file may be read-only
   - Check file properties

## Getting Additional Help

### Check Application Logs

1. **Location:** Usually in `%APPDATA%\wixen-mail\logs`
2. **Look for:** Error messages, stack traces
3. **Recent logs:** Check most recent log file

### Report Issues

When reporting issues, include:
- Wixen Mail version
- Windows version
- Email provider
- Error messages
- Steps to reproduce
- What you expected vs. what happened

### Community Support

- Check existing issues on GitHub
- Search for similar problems
- Ask in community forums

### Provider Support

If issue is provider-specific:
- Check provider's help documentation
- Verify account settings in webmail
- Contact provider's support team

## Preventive Measures

### Keep Software Updated

- Update Wixen Mail regularly
- Install Windows updates
- Keep drivers current

### Regular Maintenance

- Archive old messages periodically
- Keep reasonable number of folders
- Delete unnecessary messages

### Backup Configuration

- Note your account settings
- Keep app passwords securely
- Document custom configurations

## Still Having Issues?

If your issue isn't covered here:

1. **Check the User Guide** for feature documentation
2. **Review Keyboard Shortcuts** to ensure you're using the app correctly
3. **Try in a different environment** to isolate the issue
4. **Report the issue** with detailed information

Remember: Most issues have simple solutions. Work through the checklist systematically before concluding something is broken.

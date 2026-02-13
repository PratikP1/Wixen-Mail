# Wixen Mail - Next Steps and v1.0 Roadmap

**Date:** 2026-02-13  
**Current Status:** Beta Ready (90% complete)  
**Next Milestone:** v1.0 Release

---

## Executive Summary

Wixen Mail has successfully completed all Phase 3 objectives with 9 UI accessibility features fully implemented and 80/80 tests passing. The project is ready to proceed to Phase 4 (Composition and Editing) and beyond toward v1.0 release.

**Key Achievement:** Fully accessible email client with provider auto-configuration, thread view, attachment handling, search, and comprehensive documentation.

**Critical Gap:** Message composition and OAuth 2.0 authentication are the primary missing features for production use.

---

## Current Status Analysis

### What's Complete ✅

#### Foundation (Phase 1)
- ✅ Rust project with modular architecture
- ✅ Configuration management (JSON persistence)
- ✅ Logging framework with privacy-aware masking
- ✅ AccessKit integration for screen readers
- ✅ 25+ keyboard shortcuts
- ✅ Comprehensive documentation

#### Protocol Support (Phase 2)
- ✅ IMAP client (async, folder/message fetching)
- ✅ SMTP client (lettre, TLS, authentication)
- ✅ Email parsing (mail-parser)
- ✅ HTML sanitization (ammonia)
- ✅ Message caching infrastructure (SQLite)

#### User Interface (Phase 3)
- ✅ Three-pane layout (folders, messages, preview)
- ✅ Provider auto-configuration (5 major providers)
- ✅ Thread view with visual hierarchy
- ✅ Attachment viewer (9 file type icons)
- ✅ Advanced search UI
- ✅ Context menus with quick actions
- ✅ Performance optimizations
- ✅ Enhanced error handling

#### Documentation
- ✅ USER_GUIDE.md (13KB)
- ✅ KEYBOARD_SHORTCUTS.md (9KB)
- ✅ TROUBLESHOOTING.md (15KB)
- ✅ PROVIDER_SETUP.md (13KB)
- ✅ FEATURES_SUMMARY.md (12KB)
- ✅ Professional README

### What's Missing for v1.0 📋

#### Critical (Must-Have)
1. **Message Composition** - Users can't compose/send formatted emails
2. **OAuth 2.0** - Modern authentication required for Gmail/Outlook
3. **Multiple Accounts** - Most users have multiple email accounts
4. **Offline Mode** - Background sync and queued sending

#### Important (Should-Have)
5. **Contact Management** - Address book and auto-completion
6. **Email Signatures** - Professional email formatting
7. **Message Rules/Filters** - Automatic message organization
8. **Rich Text Editing** - HTML email composition

#### Nice-to-Have
9. **Theme Customization** - Dark mode, high contrast
10. **Calendar Integration** - CalDAV support
11. **Advanced Search Filters** - Date ranges, folders, senders

---

## Phase 4: Composition and Editing (Months 7-8)

**Duration:** 2-3 months  
**Priority:** HIGHEST - Critical for production use

### Objectives

#### 1. Message Composition Window
**Estimated Time:** 3-4 weeks

**Requirements:**
- New window/dialog for composing messages
- To, CC, BCC fields with validation
- Subject line
- Multi-line message body editor
- Send, Save Draft, Discard buttons
- Keyboard shortcuts (Ctrl+Enter to send, Ctrl+S to save)

**Technical Approach:**
```rust
pub struct CompositionWindow {
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: String,
    body: String,
    html_mode: bool,
    attachments: Vec<AttachmentInfo>,
    draft_id: Option<String>,
}
```

**Integration Points:**
- Use existing SMTP client from `src/service/protocols/smtp.rs`
- Integrate with message cache for drafts
- AccessKit labels for screen reader support

#### 2. Rich Text Editor
**Estimated Time:** 2-3 weeks

**Requirements:**
- HTML and plain text editing modes
- Basic formatting: Bold, Italic, Underline
- Font selection and sizes
- Lists (bulleted, numbered)
- Links and email addresses
- Keyboard shortcuts for formatting

**Technical Approach:**
- Use egui's TextEdit with custom formatting layer
- Implement HTML-to-markdown conversion
- Sanitize HTML output using existing ammonia integration

#### 3. Draft Auto-Save
**Estimated Time:** 1 week

**Requirements:**
- Auto-save every 30 seconds
- Save on window close
- Restore drafts on startup
- Clear drafts after sending

**Technical Approach:**
- Use existing SQLite message cache
- Add drafts table or folder
- Background timer for auto-save

#### 4. Attachment Management
**Estimated Time:** 2 weeks

**Requirements:**
- Add attachments button
- File picker integration
- Display attached files with remove option
- Size warnings (>10MB)
- Keyboard accessible

**Technical Approach:**
- Use rfd (rusty file dialog) for file picker
- Store attachments temporarily
- Integrate with SMTP multipart message

### Testing Requirements
- [ ] Unit tests for composition logic
- [ ] Integration tests for SMTP sending
- [ ] Accessibility testing with NVDA/JAWS
- [ ] Manual testing with real email accounts

---

## Phase 5: Advanced Features (Months 9-10)

**Duration:** 2-3 months  
**Priority:** HIGH - Required for v1.0

### Objectives

#### 1. OAuth 2.0 Authentication
**Estimated Time:** 3-4 weeks

**Requirements:**
- OAuth 2.0 flow for Gmail
- OAuth 2.0 flow for Outlook/Office 365
- Token storage and refresh
- Fallback to app passwords

**Technical Approach:**
- Use oauth2 crate for OAuth flow
- Implement local web server for callback
- Store tokens securely (Windows DPAPI)
- Update provider configuration UI

**Gmail OAuth Setup:**
```rust
pub async fn authenticate_gmail_oauth() -> Result<GmailOAuthTokens> {
    // 1. Get authorization URL
    // 2. Open browser for user consent
    // 3. Listen for callback on localhost
    // 4. Exchange code for tokens
    // 5. Store tokens securely
}
```

#### 2. Multiple Account Support
**Estimated Time:** 2-3 weeks

**Requirements:**
- Account management UI
- Add/remove/edit accounts
- Account switcher in main UI
- Per-account folder trees
- Unified inbox option

**Technical Approach:**
```rust
pub struct AccountManager {
    accounts: Vec<Account>,
    active_account: Option<AccountId>,
    connections: HashMap<AccountId, MailController>,
}
```

#### 3. Email Signatures
**Estimated Time:** 1 week

**Requirements:**
- Signature editor
- Per-account signatures
- Auto-insert on compose
- HTML and plain text versions

#### 4. Message Rules and Filters
**Estimated Time:** 2-3 weeks

**Requirements:**
- Rule definition UI
- Conditions (from, to, subject, body)
- Actions (move, tag, delete, mark)
- Rule evaluation engine

---

## Phase 6: Performance and Polish (Months 11-12)

**Duration:** 1-2 months  
**Priority:** MEDIUM - Quality improvements

### Objectives

#### 1. Offline Mode with Sync
**Estimated Time:** 3-4 weeks

**Requirements:**
- Background synchronization
- Queue outgoing messages
- Sync status indicators
- Conflict resolution
- Network status detection

**Technical Approach:**
- Extend existing message cache
- Background tokio task for sync
- Event system for UI updates

#### 2. Performance Optimization
**Estimated Time:** 2 weeks

**Requirements:**
- Virtual scrolling for 1000+ messages
- Lazy loading of message bodies
- Memory usage optimization
- Startup time <2 seconds

#### 3. Theme Support
**Estimated Time:** 1-2 weeks

**Requirements:**
- Dark mode
- High contrast mode
- Custom color schemes
- Font size adjustment

#### 4. Testing and Quality
**Estimated Time:** 2-3 weeks

**Requirements:**
- Unit test coverage >80%
- Integration tests
- Performance benchmarks
- Security audit
- Accessibility compliance testing

---

## Phase 7: Beta Testing and Release (Month 13)

**Duration:** 1 month  
**Priority:** HIGH - Release preparation

### Beta Testing Program

#### Week 1-2: Internal Beta
- [ ] Test with development team
- [ ] Test with screen reader users
- [ ] Performance testing
- [ ] Security review

#### Week 3-4: Public Beta
- [ ] Limited public beta (50-100 users)
- [ ] Bug tracking system
- [ ] Feedback collection
- [ ] Hot fix releases

### Release Preparation

#### Documentation
- [ ] Final user guide update
- [ ] Release notes
- [ ] Changelog
- [ ] Known issues list

#### Packaging
- [ ] Windows installer (MSI or NSIS)
- [ ] Auto-update mechanism
- [ ] Uninstaller
- [ ] Desktop shortcuts

#### Marketing
- [ ] Project website
- [ ] Announcement blog post
- [ ] Social media presence
- [ ] Accessibility community outreach

---

## Recommended Development Sequence

### Month 7: Composition Foundation
**Week 1-2:**
- [ ] Design composition window UI
- [ ] Implement basic text editor
- [ ] Add To/CC/BCC/Subject fields
- [ ] Implement Send button with SMTP integration

**Week 3-4:**
- [ ] Add draft auto-save
- [ ] Implement keyboard shortcuts
- [ ] Add accessibility labels
- [ ] Write unit tests

### Month 8: Rich Text and Attachments
**Week 1-2:**
- [ ] Implement rich text formatting
- [ ] Add formatting toolbar
- [ ] HTML/plain text toggle
- [ ] Test with screen readers

**Week 3-4:**
- [ ] Add attachment picker
- [ ] Display attached files
- [ ] Implement remove attachment
- [ ] Test file upload/send

### Month 9: OAuth and Multiple Accounts
**Week 1-2:**
- [ ] Implement Gmail OAuth flow
- [ ] Add token storage
- [ ] Update provider configuration

**Week 3-4:**
- [ ] Implement Outlook OAuth
- [ ] Add multiple account UI
- [ ] Test account switching
- [ ] Implement unified inbox

### Month 10: Signatures and Filters
**Week 1-2:**
- [ ] Add signature editor
- [ ] Implement auto-insert
- [ ] Test per-account signatures

**Week 3-4:**
- [ ] Design filter rules UI
- [ ] Implement rule engine
- [ ] Add rule actions
- [ ] Test automated filtering

### Month 11: Offline Mode and Performance
**Week 1-2:**
- [ ] Implement background sync
- [ ] Add message queue
- [ ] Network status detection
- [ ] Conflict resolution

**Week 3-4:**
- [ ] Virtual scrolling
- [ ] Memory optimization
- [ ] Performance benchmarks
- [ ] Fix bottlenecks

### Month 12: Polish and Testing
**Week 1-2:**
- [ ] Add theme support
- [ ] Improve UI consistency
- [ ] Final accessibility audit
- [ ] Complete documentation

**Week 3-4:**
- [ ] Achieve >80% test coverage
- [ ] Security audit
- [ ] Bug fixes
- [ ] Performance tuning

### Month 13: Beta and Release
**Week 1-2:**
- [ ] Internal beta testing
- [ ] Fix critical bugs
- [ ] Performance validation
- [ ] Documentation review

**Week 3-4:**
- [ ] Public beta
- [ ] Community feedback
- [ ] Final bug fixes
- [ ] v1.0 Release!

---

## Technical Dependencies and Risks

### New Dependencies Needed

```toml
# OAuth 2.0
oauth2 = "4.0"
reqwest = "0.11"  # For OAuth HTTP requests

# File picker
rfd = "0.12"  # Rusty File Dialog

# Rich text editing
pulldown-cmark = "0.9"  # Markdown parsing
syntect = "5.0"  # Syntax highlighting (optional)

# Windows credential storage
windows = { version = "0.51", features = ["Security_Credentials"] }
```

### Risk Assessment

#### High Risk
1. **OAuth Implementation Complexity**
   - Mitigation: Start with Gmail, use proven libraries, thorough testing

2. **Performance with Large Mailboxes**
   - Mitigation: Virtual scrolling, lazy loading, progressive sync

#### Medium Risk
3. **Rich Text Editor Accessibility**
   - Mitigation: Extensive screen reader testing, keyboard shortcuts

4. **Cross-Account Message Organization**
   - Mitigation: Clear UI design, unified inbox as optional feature

#### Low Risk
5. **Signature Management**
   - Mitigation: Simple text/HTML signatures, per-account storage

---

## Success Metrics for v1.0

### Functional Requirements
- [ ] All features in Phases 4-6 implemented
- [ ] Zero critical bugs
- [ ] <10 known minor bugs
- [ ] All accessibility features working

### Performance Requirements
- [ ] Startup time <2 seconds
- [ ] Memory usage <150MB with 1000 messages cached
- [ ] UI responsive (<100ms for all actions)
- [ ] Sync completes in background without blocking UI

### Quality Requirements
- [ ] >80% unit test coverage
- [ ] All integration tests passing
- [ ] Security audit completed with no critical issues
- [ ] WCAG 2.1 Level AA compliance verified

### User Requirements
- [ ] Beta tested by >10 screen reader users
- [ ] Beta tested by >50 general users
- [ ] Positive feedback from >80% of beta testers
- [ ] All critical user feedback addressed

---

## Resource Requirements

### Development Team
- **Lead Developer:** Full-time (8 months)
- **Accessibility Specialist:** Part-time (consulting)
- **Security Auditor:** Contract (2-3 weeks)
- **Beta Testers:** 10 screen reader users + 50 general users

### Infrastructure
- GitHub repository (free)
- CI/CD (GitHub Actions - free)
- Issue tracking (GitHub Issues - free)
- Documentation hosting (GitHub Pages - free)
- Beta distribution (GitHub Releases - free)

---

## Conclusion

Wixen Mail is in an excellent position to move forward. The foundation is solid with 80 passing tests, comprehensive documentation, and full accessibility support. The path to v1.0 is clear:

1. **Implement message composition** (critical)
2. **Add OAuth 2.0** (modern auth requirement)
3. **Support multiple accounts** (user expectation)
4. **Enable offline mode** (reliability)
5. **Polish and test** (quality)
6. **Beta and release** (launch)

**Estimated timeline:** 6-8 months to v1.0  
**Confidence level:** High - all infrastructure in place

---

## Next Action Items

### Immediate (This Week)
1. ✅ Review and validate this plan
2. ⏭️ Create detailed design for composition window
3. ⏭️ Set up project board for Phase 4 tasks
4. ⏭️ Research OAuth 2.0 libraries and approaches

### Short-term (Next Month)
1. ⏭️ Implement basic composition window
2. ⏭️ Integrate with existing SMTP client
3. ⏭️ Add draft auto-save
4. ⏭️ Begin rich text editor implementation

### Medium-term (Months 2-4)
1. ⏭️ Complete rich text editor
2. ⏭️ Implement OAuth 2.0 for Gmail and Outlook
3. ⏭️ Add multiple account support
4. ⏭️ Implement email signatures

---

**Document Status:** Final  
**Last Updated:** 2026-02-13  
**Next Review:** After Phase 4 completion

# Missing Functionality Gap Analysis & Requirements

## Scope

This document compares roadmap/spec artifacts with current implementation and defines implementable missing functionality delivered in this cycle.

## Sources Reviewed

- `ROADMAP.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `PHASE7/8/9/10/11` requirements files
- codebase TODOs in `src/**`

## Missing Functionality (spec vs implementation)

### A. Core infrastructure gaps

1. Data layer `storage` methods were stubs (write/read/delete no-op).
2. Data layer `database` was placeholder (schema init/query execution no-op).
3. Service `cache` methods were stubs.
4. Service `attachments` methods were stubs.
5. Application `search` was placeholder (always empty results).

### B. Accessibility infrastructure gaps

1. `Accessibility::initialize` did not initialize keyboard/focus/announcement flow.
2. Screen reader bridge/announcement queue/keyboard/focus managers were placeholder no-ops.

### C. Remaining broad roadmap gaps (not fully implemented in this cycle)

- POP3 real protocol implementation.
- IMAP IDLE push notifications.
- PGP/S/MIME/phishing checks.
- Full HTML rendering pipeline and richer attachment preview/open.
- Full accessibility automation framework and platform-native UIA bridge.
- Release packaging/public beta process tasks.

---

## Detailed Requirements for This Implementation Cycle

### R1 Storage safety + functionality
- Must support write/read/delete using local filesystem.
- Must prevent path traversal outside base path.
- Must create parent directories as needed.

### R2 Database baseline functionality
- Must open working SQLite connection.
- Must initialize minimal schema safely.
- Must execute SQL batches with proper error handling.

### R3 Cache service functionality
- Must support store/retrieve/clear with thread-safe in-memory cache.

### R4 Attachment service functionality
- Must support save/load attachment bytes to/from disk.
- Must derive a basic MIME type from extension fallback.

### R5 Search service functionality
- Must support indexing text payloads and case-insensitive query search.
- Must support optional folder-scoped filtering.

### R6 Accessibility runtime baseline
- Must support queueing prioritized announcements.
- Must track last screen reader announcement for diagnostics/testing.
- Must support keyboard shortcut registration/lookup.
- Must support focus set/get.
- Must initialize a minimal accessibility state pipeline in `Accessibility::initialize`.

---

## Acceptance Criteria

1. New storage/database/cache/attachments/search code paths are implemented and test-covered.
2. Accessibility modules provide concrete behavior and tests for core operations.
3. Existing test suite remains green.
4. No new external dependencies are required.

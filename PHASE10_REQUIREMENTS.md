# Phase 10 Requirements: Offline Mode + Queued Send/Sync

## Goal

Deliver reliable offline-first behavior so users can continue drafting and sending intent while disconnected, then synchronize safely when online.

## Functional Requirements

1. **Offline mode state**
   - UI must provide an explicit offline mode toggle.
   - When enabled, network send/sync actions are suppressed and replaced with queue operations.

2. **Offline outbox queue**
   - Persist queued outbound messages in SQLite, scoped by account.
   - Queue records must include recipient(s), subject, body, attempt count, and last error.
   - Queued items survive app restarts.

3. **Queued send behavior**
   - Compose Send action while offline must enqueue instead of attempting SMTP.
   - User gets clear status feedback that message is queued.

4. **Queue flush behavior**
   - Provide explicit action to flush queued messages when online.
   - Successful sends remove queue items.
   - Failed sends increment attempt count and store error.

5. **Status visibility**
   - UI status area must indicate offline mode and queue count for active account.

6. **Account isolation**
   - Queue operations must be strictly account-scoped.

## Non-Functional Requirements

- No regressions to existing draft/send flows when offline mode is disabled.
- Keep queue operations lightweight and deterministic.
- Preserve keyboard accessibility and screen-reader-friendly status text.

## Acceptance Criteria

- Outbox queue persistence schema and CRUD methods exist with tests.
- Offline toggle is available and affects send behavior.
- Manual queue flush sends queued emails and updates queue correctly.
- UI shows queue count and offline indicator.
- Existing and new tests pass.

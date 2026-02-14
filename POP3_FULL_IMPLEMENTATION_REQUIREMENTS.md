# POP3 Full Implementation Requirements

## Objective

Provide a full POP3 command-surface implementation in the service layer and integrate POP3 receive + SMTP send behavior in the controller.

## Scope

1. Implement POP3 client/session lifecycle with authentication entrypoint.
2. Implement core POP3 operations:
   - `STAT`
   - `LIST`
   - `UIDL`
   - `RETR`
   - `TOP`
   - `DELE`
   - `RSET`
   - `NOOP`
   - `QUIT`
3. Add controller APIs for POP3 connect/fetch/retrieve/delete.
4. Ensure SMTP sending path is explicitly available for POP3 accounts.

## Functional Requirements

### FR1 — POP3 client config and connect
- `Pop3Config` must include server, port, TLS, and username.
- `Pop3Client::connect(password)` must return a live `Pop3Session`.

### FR2 — POP3 mailbox metadata and retrieval
- `stat()` returns message count + aggregate size.
- `list()` returns ordered metadata for undeleted messages.
- `uidl()` returns stable IDs for undeleted messages.
- `retr(id)` returns full raw message.
- `top(id, lines)` returns header + selected body lines.

### FR3 — POP3 stateful mailbox commands
- `dele(id)` marks message for deletion.
- `rset()` clears pending deletions.
- `noop()` validates live session.
- `quit()` commits deletions and closes session.

### FR4 — Controller integration
- `MailController` must provide:
  - `connect_pop3(...)`
  - `fetch_pop3_messages()`
  - `fetch_pop3_message_body(id)`
  - `delete_pop3_message(id)`
  - `is_pop3_connected()`

### FR5 — POP3 + SMTP compatibility
- POP3 sending must use SMTP.
- Controller must expose explicit POP3-account SMTP send method to document protocol pairing.

## Non-Functional Requirements

- Keep implementation async-friendly and thread-safe.
- Use explicit, typed errors for disconnected/not-found conditions.
- Preserve existing IMAP and SMTP behavior.

## Acceptance Criteria

1. POP3 module exposes full command-surface methods listed above.
2. Controller supports POP3 receive operations and POP3-compatible SMTP send path.
3. Focused POP3 and controller tests pass.
4. Full test suite remains green.

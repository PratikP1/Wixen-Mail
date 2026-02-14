# IMAP IDLE Push Notifications — Detailed Requirements

## Objective

Implement IMAP IDLE-style push notifications so the application can react to mailbox changes without relying only on periodic polling.

## Scope (this implementation)

1. Add an IDLE event model in IMAP protocol layer.
2. Add session-level API to start/stop IDLE notification loop.
3. Add controller-level API to start/stop IDLE loop safely.
4. Provide deterministic behavior for tests.
5. Preserve existing behavior for folders/messages/send flows.

## Functional Requirements

### FR1 — IDLE event model
- Define typed events for:
  - keepalive heartbeat
  - EXISTS/new-message notifications (folder + UID payload)
- Events must be streamable through an async channel.

### FR2 — Session IDLE lifecycle
- `ImapSession` must expose start API:
  - accepts optional folder and timing options
  - returns `(event_receiver, idle_handle)`
- `idle_handle` must support explicit stop/cancellation.

### FR3 — Controller orchestration
- `MailController` must expose:
  - start IDLE loop (for active IMAP session)
  - stop IDLE loop
- Starting IDLE should safely stop any existing IDLE handle to avoid duplicate loops.

### FR4 — Safe fallback behavior
- Since protocol layer is still placeholder-backed, IDLE loop may emit simulated EXISTS events.
- Simulated behavior must be bounded by configurable durations and not block UI thread.

### FR5 — Testability
- Add focused async tests covering:
  - session emits keepalive + exists events
  - controller start/stop lifecycle

## Non-Functional Requirements

- Keep implementation dependency-free (no new crates).
- Use tokio async primitives only.
- Avoid shared mutable state races (single owner stop handle pattern).

## Accessibility and UX Constraints

- Event design must remain structured so UI layer can surface concise, screen-reader friendly summaries later.
- No noisy default logging at warning/error levels for normal keepalive cycles.

## Acceptance Criteria

1. New `ImapIdleEvent`, `ImapIdleOptions`, and `ImapIdleHandle` exist and compile.
2. `ImapSession::start_idle_push_notifications` returns events and supports stop.
3. `MailController::{start_imap_idle, stop_imap_idle}` are implemented and tested.
4. Existing full test suite remains green.

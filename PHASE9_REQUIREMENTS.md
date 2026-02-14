# Phase 9 Requirements: OAuth 2.0 Authentication

## Goal

Implement provider OAuth 2.0 support for account authentication flows, with secure token persistence, refresh support, and accessible user workflows.

## Functional Requirements

1. **Provider OAuth metadata**
   - Support provider-specific OAuth metadata (auth URL, token URL, scopes).
   - Initial provider coverage: Gmail and Outlook (with extensible structure).

2. **Authorization flow orchestration**
   - Generate provider authorization URL with account context and state.
   - Accept authorization code and exchange it for access/refresh tokens.
   - Support token refresh operation.
   - Support revocation/removal of stored tokens.

3. **Token persistence**
   - Persist OAuth tokens in account-scoped storage.
   - One active token set per `(account_id, provider)`.
   - Track expiration timestamp and refresh token.

4. **Accessible OAuth UI**
   - Add OAuth manager available from Tools menu and keyboard shortcut.
   - UI must expose:
     - account selection
     - provider selection
     - authorization URL generation
     - authorization code input
     - exchange/refresh/revoke actions
   - All controls keyboard-usable with explicit labels and status/error feedback.

5. **Account integration**
   - OAuth tokens must be retrievable for active account context.
   - Existing account management and persistence behavior must remain intact.

## Non-Functional Requirements

- Preserve account isolation.
- Keep implementation provider-extensible.
- No regression to existing tests and workflows.

## Acceptance Criteria

- OAuth token schema and CRUD methods exist and are tested.
- OAuth service methods for URL generation, code exchange (implementation stub), refresh, and expiry checks are tested.
- OAuth manager UI is integrated in `IntegratedUI` with menu + shortcut.
- Full test suite remains green.

# PGP / S-MIME / Phishing Features — Detailed Requirements

## Objective

Add message-security capabilities that surface:
1. PGP signature/encryption signal detection
2. S/MIME signature/encryption signal detection
3. Phishing risk analysis with explainable indicators

## Scope for this implementation

- Service-layer support in `SecurityService` for:
  - reversible local secret protection (`encrypt`/`decrypt`)
  - PGP/S-MIME signal extraction and verification status heuristics
  - phishing scoring and risk-level classification
- Structured result object for UI/integration consumers.
- Unit tests for core detection and scoring behavior.

## Functional Requirements

### FR1 — Structured security report
- Provide a report type containing:
  - `pgp_signed`, `pgp_encrypted`
  - `smime_signed`, `smime_encrypted`
  - `signature_status` (`NotSigned`, `Valid`, `Invalid`, `Unknown`)
  - `phishing_score` (0-100), `phishing_risk` (`None`, `Low`, `Medium`, `High`)
  - `phishing_indicators` (human-readable reasons)

### FR2 — PGP signal detection
- Detect armored PGP signatures/encryption via canonical markers:
  - `BEGIN PGP SIGNED MESSAGE`
  - `BEGIN PGP SIGNATURE`
  - `BEGIN PGP MESSAGE`
- Map status hints:
  - positive markers (`good signature`, `signature verified`, or explicit app marker) -> `Valid`
  - negative markers (`bad signature`, `signature invalid`, explicit invalid marker) -> `Invalid`
  - signed but no status marker -> `Unknown`

### FR3 — S/MIME signal detection
- Detect S/MIME signature/encryption from MIME-style indicators:
  - `application/pkcs7-signature`, `smime.p7s`, `smime-type=signed-data`
  - `application/pkcs7-mime`, `smime.p7m`, `smime-type=enveloped-data`

### FR4 — Phishing analysis
- Compute risk score using additive heuristics, capped at 100:
  - urgency/account-pressure phrases
  - sender/instruction mismatch patterns
  - raw IP URLs
  - punycode-like domains (`xn--`)
  - deceptive anchor text vs href mismatch for trusted-brand domains
- Map score bands:
  - 0-19 None
  - 20-39 Low
  - 40-69 Medium
  - 70-100 High

### FR5 — Local protection helpers
- Implement reversible protection for locally stored credentials with explicit prefixing and validation.
- Reject invalid/empty payloads with actionable errors.

## Non-Functional Requirements

- No new external dependencies required.
- Deterministic, testable behavior (no network calls).
- Results must be explainable (indicator list for each elevated score).

## Security Notes

- This implementation provides protocol/detection scaffolding and local reversible protection, not full cryptographic trust-chain validation.
- Full certificate/key-chain validation can be layered later on top of these APIs.

## Acceptance Criteria

1. New requirements document exists.
2. `SecurityService` exposes report-producing message analysis API.
3. PGP/S-MIME/phishing unit tests pass.
4. Full project test suite remains green.

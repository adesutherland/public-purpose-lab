# ADR-0021: Bind M3.4 identity to durable application sessions

Status: Accepted
Date: 2026-08-28

## Context

M3.3 uses a deliberately local, in-memory development-session adapter. It
cannot survive restart, distinguish authenticated external humans from
synthetic actors, enforce role removal, or qualify a shared hosted
demonstration. ADR-0014 already selects Google OpenID Connect for the first
managed presenter binding and ADR-0015 requires the browser channel to remain
behind its application backend.

M3.4 needs the smallest executable binding that preserves those decisions
without creating a general account system. Automated and isolated local
evidence must remain possible without a founder's Google account.

## Decision

Each browser-facing workload owns a durable, component-local application
session store. The store records only:

- a hash of an opaque session token and a hash of its CSRF token;
- the verified external issuer and subject, local principal and authorised
  roles;
- environment, audience, creation, expiry, last-use and revocation state; and
- where applicable, the current synthetic actor/session binding and safe
  `I-005` outcome reference.

Raw cookies, Google codes or tokens, email addresses, signed demonstration
grants and KMS credentials are never stored in that session record. Successful
authentication always rotates to a new application session. Protected writes
require the session cookie, a matching CSRF header and the exact configured
origin. Read channels revalidate expiry, revocation, audience and environment.

External authentication is behind an `I-001` adapter:

- managed interactive profiles use the ADR-0014 Google OIDC flow and an
  environment-protected issuer/subject-to-role mapping;
- isolated local and automated profiles use an explicit test adapter that
  creates the same privacy-minimised `I-001` context; and
- the test adapter is refused by every hosted/shared profile and cannot satisfy
  managed readiness.

The Director requires the `presenter` role. The Presentation Gateway requires
the separate `surface-operator` role. An external operator identity remains
distinct from the synthetic actor subsequently displayed by an application.

Role mappings carry a version. A session whose mapping is removed or changed
is revoked on its next protected use. Logout and administrative reset revoke
the backend session and expire the cookie. Session state survives an ordinary
single-instance workload restart but remains activation-scoped in the first
hosted preview.

This is a single-instance M3 binding. It does not select a multi-node session
service, enterprise identity administration, refresh tokens, domain-derived
authority or production browser-session qualification.

## Consequences

- Restart and browser refresh can be tested without turning a cookie into
  durable authority.
- External human, workload and synthetic identities remain separate and can
  be reported separately.
- Each application has a small security store and must perform expiry,
  rotation, CSRF and role-version checks.
- A managed profile is not ready unless its OIDC adapter, protected role map
  and secure-cookie/HTTPS configuration are all present.
- Scaling a workload above one replica requires a new shared-session or
  affinity decision and is outside M3.4.

## Validation and review

Evidence must cover session fixation, restart, refresh, expiry, logout, wrong
origin, missing/wrong CSRF, environment/audience mismatch, role removal and
test-adapter refusal in a hosted profile. Events, logs, frontend bundles and
evidence are scanned for provider tokens, raw cookies and unnecessary claims.

Review the component-local store before multiple replicas, longer-lived hosted
sessions, more than the founder-approved role set or non-synthetic data.

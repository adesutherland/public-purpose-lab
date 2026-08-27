# ADR-0011: Establish the M3 scenario-control invariants

Status: Accepted
Date: 2026-08-27

## Context

M3 introduces a Scenario Director that must coordinate a repeatable
demonstration without becoming an identity issuer, policy bypass, business
system of record or browser remote control. Reset, controlled time and failure
injection are particularly easy to implement as broad administrative powers
that would be difficult to contain later.

The M2 identity design already binds synthetic grants, sessions and surfaces to
one environment and Demonstration Session. Reusing that session identity after
reset would allow delayed work or stale security and presentation bindings to
cross from one run into another.

## Decision

Adopt the `CTL-01` and `D-001` to `D-004` M3.1 logical specifications and the
M3 threat model as the accepted scenario-control baseline.

The Scenario Director is a coordination authority only. It may control its own
scenario lifecycle and request other actions, but every identity, policy,
business, reset, fault and presentation action remains authorised and enforced
by its owning component. Presentation outcomes, command acceptance and visible
screens do not become business facts.

Scenario packages are immutable, declarative and synthetic-only in M3. They do
not contain credentials, browser routes, executable scripts, SQL,
infrastructure commands or hidden side effects.

Reset is explicit and component-owned. A running or paused session must first
stop. When all required reset targets complete conclusively, the old
Demonstration Session becomes superseded and a new session identifier is
created. Grants, application sessions, surface bindings, delayed work and
idempotency scope do not transfer to the successor.

Scenario logical time is labelled test context for opted-in synthetic
behaviour. It never changes or substitutes for protected operational time used
for credentials, grants, sessions, certificates, revocation, replay,
authorisation freshness, message expiry or evidence recording.

Readiness and checkpoints keep software health, interaction readiness,
scenario readiness, presentation progress and business or evidential facts as
separate claim classes supported by their authoritative sources.

## Consequences

- The first implementation needs durable session revisions, explicit
  idempotency and safe reconciliation, but the persistence product remains
  unselected.
- Reset is more deliberate than returning an existing session to its initial
  state, but it gives demonstrations and adverse tests a clean security and
  correlation boundary.
- Component owners must publish bounded reset, time and fault capabilities;
  the Director cannot use generic administrative interfaces.
- The Director Console and Presentation Surface remain projections over
  authorised backend state, not authority or truth sources.
- Package schema, signing, presenter authentication, event transport,
  persistence, surface binding and test-adapter mechanisms remain M3.2 or later
  decisions.
- `CTL-01` remains planned and unimplemented. Acceptance of the logical
  baseline is not evidence of an executable or production-capable M3 system.

## Validation and review

Implementation evidence must cover unauthorised control, duplicate and stale
lifecycle commands, restart uncertainty, stop-before-reset, partial reset,
successor-session isolation, logical/protected-clock separation, contained
faults, stale observations and the distinction between presentation and
business completion.

Reconsider this decision if implementation cannot keep those authorities and
claim classes separate, or if a later scenario requires a different reset or
time model. Any change that allows session reuse, arbitrary executable package
content, security-clock control or presentation-driven business mutation needs
founder approval and a superseding ADR.

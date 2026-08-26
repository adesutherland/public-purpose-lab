# ADR-0010: Use a locked local security journal and rebuild recovery for the M2 reference path

Status: Accepted implementation baseline
Date: 2026-08-26

## Context

M2 must demonstrate single-use grants, at-most-one session establishment,
restart reconciliation, revocation and termination before a database or
distributed session product has been selected. It must also state honestly
whether a supported profile restores the same trust domain or rebuilds one.

## Decision

Use a separate append-only JSON-lines IAM security journal and an exclusive
cross-platform file lock for the local M2 reference binding. The lock covers
grant consumption, authorisation outcome, establishment decision, session
reference allocation and terminal-state transitions.

The journal stores only safe identifiers, irreversible digests, state,
decision and evidence references. It never stores a private key, raw signed
grant, browser credential, authorization header, cookie or reusable session
value. The non-credential session reference is derived from fresh random input;
the random value is discarded after its reference is created.

The local reference session owner is a backend-only state machine. It proves
application, surface, actor, role, realm, idempotency and termination semantics
but does not issue a browser cookie or expose a network login endpoint. A real
application session owner later replaces this adapter through the protected
`I-005` boundary.

The initial local-synthetic profile supports **rebuild recovery only**. Loss,
copy, corruption or uncertain continuity of private key, replay, revocation or
session state makes identity readiness false. Recovery creates a new
environment identity and trust domain; former grants and sessions remain
invalid. The reference runtime does not claim same-environment key recovery.

A managed binding must declare and prove either protected same-environment
recovery or the same rebuild posture. Merely restoring a volume or relabelling
an environment is not accepted recovery.

## Consequences

- At-most-one and restart behaviour are executable on one host.
- IAM security state remains separate from the M1 interaction journal and from
  application, uploaded-asset and substantive evidence data.
- Multi-replica operation, network filesystems, high availability,
  tamper-evident audit retention and browser session security remain
  unqualified.
- Compose may use a local state volume for restart evidence. The Kubernetes
  base uses disposable state and therefore visibly rebuilds a new trust domain
  after Pod replacement.

## Validation and review

Evidence must cover duplicate and concurrent establishment, restart, lost
acknowledgement reconciliation, termination, revocation, corrupt state,
rebuild, cross-environment refusal and disclosure scanning.

Replace or extend this ADR before adding multiple replicas, a distributed
store, a browser login endpoint, authoritative audit retention or protected
same-environment recovery.

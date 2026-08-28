# ADR-0022: Use backend-only delivery for synthetic sign-in

Status: Accepted
Date: 2026-08-28

## Context

M2 proves a signed, short-lived, single-use demonstration sign-in grant in an
isolated process. M3.4 must bind that mechanism across the Scenario Director
and a target application without exposing a grant to the browser or assuming
that issuer and application share one local security journal.

The target application must own its ordinary application session. Across one
scenario, different applications may be bound to different synthetic actors;
the at-most-one rule applies to each grant establishment operation rather than
to the scenario as a whole.

## Decision

Introduce a narrow protected identity event path:

1. an authenticated and authorised presenter asks the Director to sign a named
   configured synthetic actor into one application/surface in one current
   Demonstration Session;
2. the Director workload requests a bounded `I-004` grant from `IAM-01` using
   its workload identity;
3. `IAM-01` applies the configured actor, role, purpose, relationship, consent
   and obligation checks, then signs a two-minute environment-bound grant;
4. the grant is delivered only over the authenticated component channel to the
   named application backend;
5. that backend validates the public trust anchor, signature, environment,
   epoch, audience, application, surface, scenario session and time, then
   durably reconciles the establishment operation at most once; and
6. the backend binds the resulting synthetic context to its current ordinary
   application session and returns only a privacy-minimised `I-005` outcome.

The browser never receives the grant, grant signature, KMS key name, workload
credential or synthetic session reference. It receives only the actor display
context and safe status needed by that application.

Local profiles use the existing environment-generated Ed25519 issuer. Managed
profiles use the retained environment trust record and Cloud KMS issuer chosen
by ADR-0016. The managed signer accepts canonical `ppl-i004-ed25519-v1` bytes,
uses Workload Identity Federation, and cannot export a private key. Validators
receive only the pinned public trust material. A local trust record, wrong
environment/project/key version, disabled signer or unavailable replay store
fails closed.

Issuer evidence and target-application replay evidence are separate. The
application does not need write access to issuer state, and an issuer's claim
that it sent a grant is not evidence that an application established a
session.

Stop, reset, supersession, presenter logout or trust revocation terminates the
affected application binding. A successor scenario needs fresh grants. The
same actor names may exist in another environment, but their trust domain and
grants are not interchangeable.

## Consequences

- Synthetic sign-in becomes an event-driven component interaction instead of
  a browser or URL mechanism.
- The target application can enforce replay safety after restart without
  sharing the issuer's private journal.
- Trust-anchor distribution, issuer availability, termination propagation and
  recovery become explicit runtime dependencies.
- The first M3 implementation has one identity-broker workload and one target
  application binding. Additional applications reuse the contracts and must
  supply their own application-session integration.

## Validation and review

Evidence must cover two applications or surfaces using distinct synthetic
actors in one scenario, duplicate delivery, conflicting reuse of an operation,
expired/premature grants, wrong actor/application/surface/session/audience,
cross-environment trust, restart, reset, stop and trust revocation. It must also
prove that browser state, routes, events exposed to the browser, logs and safe
evidence contain no usable grant or credential.

Review the event subjects and termination fan-out before adding another target
application, multiple broker replicas or non-synthetic information.

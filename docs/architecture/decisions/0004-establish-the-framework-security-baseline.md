# ADR-0004: Establish the framework security baseline

Status: Accepted
Date: 2026-08-25
Accepted: 2026-08-26

## Context

Identity is only one part of framework security. The first executable
interaction will otherwise embed assumptions about trust zones, authority,
information handling, recovery and diagnostics before those assumptions are
visible or testable.

The baseline will evolve as implementation and demonstrators provide evidence,
but dependent components need one controlled starting point now.

## Decision

Use the versioned
[framework security model](../security/framework-security-model.md) and
[M1 threat model](../security/m1-threat-model.md) as the current baseline for
M1 implementation and review.

Treat external-human, synthetic-human, workload, operator and service-owner
principals as distinct. Require receiving-component authorisation, explicit
purpose and classification, default refusal, privacy-minimised evidence and
separate trust/security-state/business-data recovery domains.

Establish `AUT-01` as a logical, externalisable policy-decision capability for
shared access control. It evaluates versioned policy against authenticated
principal context, the requested action and resource, purpose, environmental
conditions and bounded attributes or relationship assertions obtained from
identified authoritative sources. An external product may implement this
capability later, but the logical contract must not depend on one product or on
the engine being remotely hosted.

The receiving component remains the policy-enforcement point and accountable
owner of the protected action. An `AUT-01` permit is necessary where policy
requires it but is not sufficient: the component may apply current domain
conditions and further restrict or refuse the action. It must never override a
deny or indeterminate result, ignore an applicable obligation or expand the
authority supplied to the decision capability.

Relationship, consent, restriction, organisation and purpose assertions remain
distinct, time-bounded and attributable to their source. `AUT-01` does not
invent a legitimate relationship, convert one into consent or legal authority,
or make clinical, professional, safeguarding, regulatory, business-approval or
report-release decisions. Emergency or exceptional access is a separate,
explicitly authorised, time-bounded, alerted and reviewed action rather than a
general override.

M1 exercises these semantics in a local assurance profile only. It does not
claim authentication. No external transport may rely on M1 authority metadata
until an approved identity and transport binding authenticates it.

## Consequences

- `IAM-01` conforms to a framework model rather than defining security for all
  components.
- Identity context, shared policy evaluation and component enforcement remain
  distinct responsibilities.
- Local and hosted profiles may bind `AUT-01` in-process, as a sidecar or as a
  shared service while preserving the same decision contract and privacy
  boundary.
- A required unavailable, stale or indeterminate decision fails closed.
- Selecting a policy language, engine, deployment topology or external
  relationship source requires a later binding ADR and conformance evidence.
- Security and threat evidence accompanies each later milestone.
- M1 code can proceed without preselecting the M2 certificate, key, workload
  identity or session mechanisms.
- A local conformance command remains deliberately unsuitable for exposure as
  an unauthenticated API.

## Validation and review

The founders accepted this baseline on 26 August 2026 after adding the
externalisable authorisation boundary. M1 evidence validates the local
interaction and refusal invariants; it does not implement or qualify `AUT-01`.

Reconsider the model when a new principal, information class, external
transport, authoritative attribute or relationship source, recovery profile or
consequential action is introduced, when an authorisation product is selected,
or when threat evidence shows that an invariant is incomplete.

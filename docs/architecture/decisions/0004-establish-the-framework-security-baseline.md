# ADR-0004: Establish the framework security baseline

Status: Proposed
Date: 2026-08-25

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

M1 exercises these semantics in a local assurance profile only. It does not
claim authentication. No external transport may rely on M1 authority metadata
until an approved identity and transport binding authenticates it.

## Consequences

- `IAM-01` conforms to a framework model rather than defining security for all
  components.
- Security and threat evidence accompanies each later milestone.
- M1 code can proceed without preselecting the M2 certificate, key, workload
  identity or session mechanisms.
- A local conformance command remains deliberately unsuitable for exposure as
  an unauthenticated API.

## Validation and review

Founder review is required before this ADR and M1 are accepted. Reconsider the
model when a new principal, information class, external transport, recovery
profile or consequential action is introduced, or when threat evidence shows
that an invariant is incomplete.

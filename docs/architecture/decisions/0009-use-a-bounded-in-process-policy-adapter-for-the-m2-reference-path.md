# ADR-0009: Use a bounded in-process policy adapter for the M2 reference path

Status: Accepted implementation baseline
Date: 2026-08-26

## Context

M2 needs executable permit, deny, not-applicable and indeterminate behaviour,
including synthetic relationship, consent and obligation checks. Selecting an
external authorisation product before the first demonstrator has established
its policy and deployment needs would make the reference path depend on a
premature product decision.

## Decision

Implement `AUT-01` for M2 as a deterministic in-process adapter behind the
transport-neutral `AZ-001` contract. The adapter evaluates one deliberately
small demonstration policy from versioned configuration:

- the requester is an authenticated workload and the actor is a distinct
  enabled synthetic human;
- action, target application, audience, role, purpose and synthetic realm are
  explicitly permitted;
- required relationship and consent assertions come from configured synthetic
  authoritative sources, are attributable, current and not revoked;
- required obligations are returned with a permit; and
- unavailable, stale, malformed, unsupported or unverifiable required input
  returns indeterminate and fails closed.

The receiving IAM/session component remains the policy-enforcement point. It
may further refuse a permit. It cannot override deny or indeterminate, ignore
an applicable obligation or expand the permitted scope.

The policy adapter consumes only bounded identifiers and attributes required
for the decision. It does not ingest source records or infer legal,
professional, clinical, regulatory, safeguarding or report-release authority.
The initial sources and people are synthetic.

The adapter interface permits a future in-process library, sidecar, shared
service or external product to implement the same `AZ-001` semantics. No policy
language, vendor or remote topology is selected by M2.

## Consequences

- Authorisation failure behaviour is executable before product selection.
- The reference policy is test and demonstration infrastructure, not general
  access-control policy or evidence of compliance.
- Relationship and consent remain distinct inputs with independent freshness
  and revocation.
- A later engine must pass the same contract, privacy, obligation and
  fail-closed fixtures before substitution.

## Validation and review

Evidence must cover permit, deny, not-applicable, indeterminate, stale and
revoked relationships, missing consent or purpose, obligation enforcement,
policy change, dependency failure, principal separation and minimal retained
decision evidence.

Reconsider this decision when a demonstrator needs a policy language,
externally maintained policy bundle, real authoritative relationship source,
exceptional access or an independently deployed decision service.

# AUT-01: Policy decision and authorisation

Status: Accepted logical boundary; bounded M2 reference implementation in development

Last reviewed: 26 August 2026

## Purpose

`AUT-01` is the externalisable policy-decision capability for shared access
control. It evaluates whether an authenticated principal may request a defined
action on a defined resource for a stated purpose under a versioned policy and
bounded current context.

This is a logical component. It may later be implemented in-process, as a local
sidecar, as a shared environment service or through a contained external
product adapter. Acceptance of the boundary does not select a policy language,
engine, vendor, deployment topology or authoritative relationship source.

## Accountable ownership

The environment service owner is accountable for approved access-control
policy, attribute and relationship sources, obligations and exceptional-access
configuration. `AUT-01` owns consistent policy evaluation and safe decision
evidence. Each receiving component remains the enforcement point and
accountable owner of the protected action.

`AUT-01` owns:

- resolution of the applicable versioned access-control policy;
- evaluation of authenticated principal, action, resource, purpose,
  environment and bounded contextual attributes;
- evaluation of attributable, time-bounded relationship, consent, restriction
  and organisational assertions when the policy requires them;
- permit, deny, not-applicable and indeterminate decisions;
- applicable obligations and privacy-minimised decision evidence; and
- safe readiness, policy-version and dependency status.

## Decision and enforcement model

`IAM-01` validates identity and supplies bounded principal and authority
context. Authoritative sources supply only the relationship, consent,
restriction, organisation or other attributes required for the decision.
`AUT-01` evaluates those inputs. The receiving component enforces the result
against its current domain state.

Where policy requires an `AUT-01` decision:

- permit is necessary but not sufficient;
- the receiver may further restrict or refuse the action;
- the receiver must not override deny or indeterminate;
- every applicable obligation must be enforced or the action refused;
- unavailable, stale, incompatible or unverifiable required input fails
  closed; and
- the outcome records policy, decision and bounded evidence references.

The receiver never delegates ownership of the resulting business fact merely
because it uses a shared decision capability.

## Authoritative relationships and privacy

A relationship assertion identifies its authority, subject and resource
references, relationship type, organisation, permitted purpose, effective and
expiry times, verification time and version or revocation reference. It is not
self-asserted by the caller or Scenario Director.

Relationship, consent, legal basis, confidentiality restriction and emergency
access are distinct inputs. One must not be inferred from another. Decision
requests contain the minimum bounded attributes or opaque evidence references
needed by policy; they do not copy a clinical, personal or business record into
the authorisation engine.

Initial demonstrations use synthetic relationship and consent data. Supporting
real personal, care, employment or client relationships requires separately
recorded authority, governance, source integration and conformance evidence.

## Exceptional access

Emergency or exceptional access is a separately named action with dedicated
policy. It requires an attributable actor, permitted purpose, recorded reason,
bounded duration, alert, enhanced audit and subsequent review. It is not an
administrator bypass and does not create professional or legal authority.

## Non-responsibilities

`AUT-01` does not:

- authenticate a principal or hold source credentials;
- create or validate an external relationship beyond checking an assertion
  from its configured authority;
- convert a relationship into consent, legal basis or professional authority;
- decide clinical, safeguarding, regulatory or legal questions;
- approve a finding, business outcome, active rule or report release;
- replace `RUL-01` business rules or accountable human approval;
- allow browsers to query policy or relationship stores directly; or
- require sensitive attributes to leave the environment.

## Contract and evidence boundary

The accepted
[`AZ-001`](../contracts/authorisation/az-001-authorisation-decision-and-obligations.md)
family defines the decision request, decision outcome, obligations, policy
version, authoritative-attribute references, expiry, refusal and evidence
semantics. `C-002` continues to carry bounded authority, purpose,
`policyVersion` and `decisionReference`; it does not become proof that the
referenced policy decision is valid.

The M2 conformance suite must cover permit,
deny, not-applicable, indeterminate, stale and revoked relationships, missing
consent or purpose, obligation enforcement, policy change, dependency failure,
minimal disclosure and exceptional-access audit. ADR-0009 selects the bounded
in-process reference adapter without selecting a future external engine.

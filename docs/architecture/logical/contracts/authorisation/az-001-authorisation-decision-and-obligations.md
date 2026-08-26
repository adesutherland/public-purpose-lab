# AZ-001: Authorisation decision and obligations

Status: Accepted; bounded M2 reference binding implemented

Last reviewed: 26 August 2026

Owner: [`AUT-01`](../../components/aut-01-policy-decision-and-authorisation.md)

Semantic type: protected decision request and privacy-minimised decision

## Purpose

`AZ-001` asks whether an authenticated principal may request one defined action
on one defined resource for one stated purpose. It returns permit, deny,
not-applicable or indeterminate, together with every applicable obligation and
safe evidence reference.

The decision does not perform the protected action. The receiving component is
the policy-enforcement point and accountable owner. A permit is necessary where
the policy requires it but is never sufficient to override current domain
conditions or human authority.

## Participants and boundaries

| Role                  | Responsibility                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------- |
| `IAM-01`              | Supplies authenticated requester and, where applicable, independently attributable actor context.             |
| Authoritative sources | Supply only required relationship, consent, restriction or organisation assertions with source and freshness. |
| `AUT-01`              | Resolves policy, validates decision inputs, evaluates them and returns the bounded decision.                  |
| Receiving component   | Enforces deny, indeterminate and obligations, then may further restrict or refuse the domain action.          |
| `AUD-01` / `OPS-01`   | Retain and expose safe decision, dependency and failure evidence without copying protected source records.    |

A browser, Scenario Director, caller or workload cannot make an assertion
authoritative by including it in a request. Relationship, consent, legal basis,
professional standing and exceptional access remain distinct concepts.

## Variants

- `decision-request` identifies requester, actor, action, resource, purpose,
  requested roles, versioned policy and bounded assertions.
- `decision` identifies the request and returns a status, reason, policy
  version, validity bound, obligations and evidence references.

The canonical JSON Schema is
[`contracts/authorisation/az-001-authorisation-decision-and-obligations.schema.json`](../../../../../contracts/authorisation/az-001-authorisation-decision-and-obligations.schema.json).

## Decision semantics

- **permit** means the evaluated access-control policy allows the request if
  the receiver enforces every obligation and its current domain state also
  permits the action;
- **deny** means the applicable policy explicitly refuses the request;
- **not-applicable** means no approved policy applies to the requested action
  and resource; and
- **indeterminate** means a required policy, assertion, dependency, version or
  verification result is unavailable, stale, malformed or inconsistent.

Only permit can continue to enforcement. Deny, not-applicable and indeterminate
fail closed. A receiver may narrow or refuse a permit, but never convert another
status to permit or discard an applicable obligation.

## Assertion requirements

Each assertion supplies an identified authoritative source, assertion type,
subject and resource references, permitted purposes, state, effective and
expiry times and version or revocation lineage.

The request carries the minimum assertion needed for the decision, not the
underlying personal, clinical, employment, client or business record. Initial
M2 examples use only synthetic assertions.

## Obligations

An obligation is a named, bounded enforcement requirement. The M2 reference
policy uses `mark-synthetic` and `restrict-realm`. A receiver that does not
understand, support or successfully apply an obligation refuses the action.
Adding an obligation with new enforcement meaning requires compatibility
review.

## Repetition, freshness and policy change

A decision is bound to its request identifier, input versions, policy version
and validity interval. Repeating an identical request may return the same
decision reference while all inputs remain current. Policy change, assertion
revocation, expiry or dependency failure requires re-evaluation and cannot be
masked by a former permit.

The decision reference is evidence linkage, not a bearer credential. A caller
cannot obtain authority by copying it into `C-002`.

## Failure, audit and disclosure

Safe evidence records request and decision identifiers, principal types and
irreversible identifiers, action/resource/purpose, policy and assertion-version
references, status, reason, obligations, time and correlation. It excludes
credentials, private keys, raw grants, session values and source records.

Public and presenter views may group sensitive refusal reasons. Authorised
support views still expose only the minimum diagnostic detail.

## M2 reference binding

ADR-0009 binds the initial contract to a deterministic in-process adapter over
versioned synthetic policy, relationship and consent configuration. This is an
in-development reference path, not a selected general policy product. A future
library, sidecar, service or product adapter must preserve these semantics and
pass the same fixtures.

## Conformance evidence

Evidence covers:

1. current matching relationship and consent can produce permit with all
   obligations;
2. explicit policy refusal produces deny;
3. unsupported action/resource produces not-applicable;
4. missing, stale, revoked, malformed or unavailable required inputs produce
   indeterminate;
5. requester and actor types cannot substitute for one another;
6. missing or unsupported obligations prevent the action;
7. policy or assertion change invalidates a former permit;
8. the receiver can further refuse but cannot override deny or indeterminate;
   and
9. evidence and diagnostics contain no credential or protected source record.

## Open binding decisions

External policy engine, policy language, bundle distribution, real
authoritative sources, exceptional-access policy, distributed caching and
decision-service topology remain deferred. None may change component ownership
or turn a relationship assertion into wider legal or professional authority.

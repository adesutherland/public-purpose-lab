# I-002: Workload identity context

Status: Accepted; M2 local-synthetic reference binding implemented

Last reviewed: 26 August 2026

Owner: [`IAM-01`](../../components/iam-01-identity-trust-and-synthetic-session-broker.md)

Semantic type: protected query/response and bounded lifecycle notice

Canonical schema:
[`i-002-workload-identity-context.schema.json`](../../../../../contracts/identity/i-002-workload-identity-context.schema.json)

## Purpose

`I-002` authenticates a component workload and supplies its least-privilege,
environment-bound authority to another component. It allows receivers to
distinguish the service carrying a request from any external-human or synthetic
actor on whose behalf that request was initiated.

Workload identity grants no human role and does not permit impersonation. It is
an authenticated component identity, not evidence that the requested business
action is appropriate.

## Participants and trust boundary

| Role              | Participant                                    | Responsibility                                                                                    |
| ----------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Trust provisioner | `PLT-01` with environment trust facilities     | Establishes and rotates an environment-specific workload identity under approved policy.          |
| Context owner     | `IAM-01`                                       | Validates workload trust and resolves bounded authority for the intended audience.                |
| Requester         | Calling workload                               | Authenticates as itself and supplies purpose, audience and requested contract action.             |
| Consumer          | Receiving component                            | Validates context and separately authorises the requested contract action.                        |
| Initiating actor  | Optional external-human or synthetic principal | Remains a separate context when an attributable person or demonstration actor initiated the work. |

The interaction infrastructure carries a protected context but is not itself a
workload trust authority. Network location, container membership or access to a
broker channel is not sufficient authentication.

## Contract variants

| Variant                              | Kind             | Purpose                                                                                         |
| ------------------------------------ | ---------------- | ----------------------------------------------------------------------------------------------- |
| `ResolveWorkloadIdentityContext`     | Protected query  | Requests a workload context for one environment, audience, purpose and set of contract actions. |
| `WorkloadIdentityContext`            | Response         | Supplies validated workload principal and least-privilege authority.                            |
| `WorkloadIdentityContextUnavailable` | Query failure    | Reports a safe refusal or dependency failure without disclosing credential material.            |
| `WorkloadAuthorityChanged`           | Lifecycle notice | Advises authorised consumers of rotation, reduction, revocation or invalidation.                |

The raw workload assertion, key or token is never a domain event.

## Preconditions and authority

Before a context is returned:

- the workload authenticates through the current environment trust path;
- its immutable workload identity is enabled in that environment;
- the intended audience and contract actions are registered and allowed;
- the requested purpose and environment match the workload policy;
- the trust and authority versions are current; and
- any actor or delegation context carried with the call is independently valid.

The receiving component authorises both the workload and, where required, the
initiating actor. A workload permitted to deliver a command is not necessarily
permitted to choose the actor, approve the outcome or read all resulting data.

## Common-envelope requirements

The interaction uses `C-001` and `C-002` and must include at least:

- message, schema and context identifiers and versions;
- environment, workload issuer and workload principal references;
- intended audience and permitted contract action;
- issued, valid-from and expiry times;
- purpose, classification and applicable policy version;
- correlation, causation, idempotency and trace references; and
- a separate initiating-actor context reference where applicable.

The eventual protected-channel or message-proof mechanism is a transport
binding and requires an ADR.

## Contract-specific information

`ResolveWorkloadIdentityContext` supplies:

- the workload's protected authentication reference;
- target component or contract audience;
- requested contract action or actions;
- purpose and any engagement, scenario or tenant constraint;
- delivery or job context; and
- initiating-actor and delegation references, if applicable.

`WorkloadIdentityContext` supplies:

- context identifier and version;
- environment identifier;
- principal type `workload`;
- immutable workload issuer and principal references;
- deployment or workload-instance class where needed, without making a
  short-lived instance name the durable principal;
- permitted audience, contracts and actions;
- purpose, engagement, scenario or tenant constraints;
- valid-from and expiry times;
- authority and trust-policy versions;
- optional initiating-actor context reference; and
- safe policy-decision and audit references.

It never contains a private key, bearer token, source assertion, bootstrap
secret, synthetic grant or human session credential.

## Acceptance and refusal

Safe reason classes include:

- `workload-not-trusted`;
- `workload-disabled`;
- `environment-mismatch`;
- `audience-mismatch`;
- `contract-action-not-permitted`;
- `purpose-not-permitted`;
- `context-expired`;
- `trust-version-stale`;
- `actor-context-required`;
- `actor-context-invalid`; and
- `identity-service-unavailable`.

The consumer refuses excessive authority rather than trimming it invisibly. If
policy permits a narrower result, the returned context explicitly states the
narrowed actions and the requester's subsequent command uses that context.

## Repetition, ordering and idempotency

- Repeating resolution for the same current workload, audience and scope does
  not create another durable principal.
- A rotated workload credential may resolve to the same durable workload
  principal under a newer trust version.
- A later revocation or authority reduction wins over a delayed earlier
  context.
- Consumers do not infer order from workload-instance names or transport
  delivery order.
- Idempotency of the business command remains defined by that command contract;
  workload context repetition neither creates nor completes business work.

## Expiry, rotation and recovery

Contexts are time-bounded. Credential and context lifetimes, clock tolerance
and renewal behaviour require an ADR, but renewal must not broaden authority or
change durable workload identity silently.

After restart, a workload reauthenticates through the environment trust path.
`IAM-01` does not recover a live credential from logs, audit or analytics.
During trust-store, clock, revocation or policy uncertainty, new contexts fail
closed and existing contexts follow the agreed bounded invalidation policy.

If a deployment is restored as a new environment, its workloads receive new
environment-scoped identities. Trust from the source environment does not
transfer merely because configuration or workload names were copied.

## Audit, retention and provenance

Evidence records:

- environment, durable workload principal and issuer references;
- context, trust and authority-policy versions;
- audience, purpose and contract action;
- initiating actor reference where applicable;
- issued, refused, expired, rotated or revoked outcome;
- safe reason, correlation and decision references; and
- the receiving component's independent outcome under its own contract.

Audit does not retain workload credentials or values that can recreate them.
Retention must support incident reconstruction and authority-change evidence
without becoming an alternative identity store.

## Analytical use

Permitted projections include calls by workload class, audience, action,
refusal class, trust version, rotation impact and authentication failure. They
must not infer human activity from workload calls unless a valid separately
governed actor context supports that interpretation.

Analytics cannot grant, restore or revoke workload access.

## Operations and observability

Operational signals cover trust-provider availability, rotation age,
configuration consistency, validation latency, refusal categories and
revocation propagation. Readiness is false for a required interaction when the
receiver cannot validate current workload trust.

Logs and traces contain workload and correlation references but never raw
proofs, private keys, bootstrap secrets or bearer values.

## Deployment considerations

Local, portable and hosted profiles may use different workload-trust
implementations. Every profile still provides environment-specific identity,
least privilege, rotation, revocation and independent receiver validation.

Combining two logical components in one process may replace a network proof
with a process-local binding, but it must not erase authority separation or let
one component call another's private state as an implicit superuser. If a later
deployment separates them, the same logical audience and action restrictions
apply.

## Versioning and compatibility

Each variant declares its schema version through `C-006`. Optional diagnostic
references may be compatible; changing workload-principal construction,
audience, action, actor-delegation, expiry or revocation semantics is breaking.

Consumers refuse unsupported versions and unknown authority-bearing fields.
Deprecation requires evidence that combined-process and distributed consumers
enforce equivalent semantics and that historical audit remains interpretable.

## Transport-neutral examples

An accepted example is: the Scenario Director workload requests authority to
send one `D-002` lifecycle command to the scenario component in environment
`demo-a`; its context permits only that audience and action, while the presenter
actor remains a separate reference.

A negative example is: the same workload context is offered to `IAM-01` as if
it were a synthetic actor or is reused in environment `demo-b`. The receiver
refuses principal-type or environment mismatch and no user session is created.

## Threat considerations

The threat model must address:

- workload credential extraction or cloning;
- network-location or broker-access trust;
- cross-environment and cross-audience replay;
- excessive wildcard permissions;
- compromised workload acting as a confused deputy;
- actor-context substitution or omission;
- stale trust or revocation state;
- unsafe bootstrap and recovery material;
- identity collision during scaling or redeployment; and
- logs or traces exposing reusable proof.

## Conformance evidence

Evidence must show that:

1. every participating workload resolves to a durable, environment-bound
   principal;
2. an untrusted, disabled, expired, wrong-environment or wrong-audience workload
   is refused;
3. a workload cannot invoke an unpermitted contract action;
4. carrying an actor context does not merge the actor and workload identities;
5. a workload cannot create a human or synthetic session by presenting its own
   identity;
6. rotation preserves durable identity while invalidating superseded proof;
7. revocation and authority reduction take effect despite delayed messages;
8. deployment cloning does not copy valid workload trust into a new
   environment;
9. combined-process and separated deployments enforce equivalent logical
   authority; and
10. public payloads, logs, traces and audit contain no reusable workload
    credential.

## Open ADR decisions

- workload identity and proof mechanism for each deployment profile;
- durable workload naming, instance identity and scaling model;
- protected-channel or message-proof binding;
- authority-policy representation and action granularity;
- actor-on-behalf-of context and delegation semantics;
- lifetime, rotation, revocation and clock behaviour;
- local combined-process enforcement; and
- workload trust backup, restore and new-environment recovery.

No decision may turn network access into authority, merge workload and actor
identity, or make credentials portable across environments.

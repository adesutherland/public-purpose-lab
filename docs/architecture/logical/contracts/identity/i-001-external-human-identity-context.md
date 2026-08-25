# I-001: External human identity context

Status: Accepted

Last reviewed: 25 August 2026

Owner: [`IAM-01`](../../components/iam-01-identity-trust-and-synthetic-session-broker.md)

Semantic type: protected query/response and bounded lifecycle notice

## Purpose

`I-001` supplies a receiving component with a validated, time-bounded context
for a human authenticated by a configured external identity provider. It lets
the component make its own authorisation decision without receiving or
replaying the person's external credential.

The contract proves neither professional standing nor legal, clinical,
regulatory or organisational authority. Any such authority must be separately
represented, governed and checked by the component that owns the action.

## Participants and trust boundary

| Role                  | Participant                           | Responsibility                                                                                                     |
| --------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Authentication source | Configured external identity provider | Authenticates the human and supplies an immutable issuer-and-subject assertion to the protected identity boundary. |
| Context owner         | `IAM-01`                              | Validates the authentication result, applies versioned environment mappings and issues the bounded context.        |
| Requester             | Authenticated backend or gateway      | Requests or refreshes a context for an existing protected session.                                                 |
| Consumer              | Authorised framework component        | Validates audience, environment, expiry and authority scope, then makes its own action decision.                   |
| Actor                 | External human                        | Remains attributable independently from any workload carrying the request.                                         |

The user's browser may participate in the selected external authentication
flow, but it is not trusted to construct or alter the resulting framework
identity context. Provider credentials and raw assertions do not cross into
domain contracts.

## Contract variants

| Variant                           | Kind             | Purpose                                                                                                          |
| --------------------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------------- |
| `ResolveExternalHumanContext`     | Protected query  | Requests a context for an authenticated environment session and intended audience.                               |
| `ExternalHumanIdentityContext`    | Response         | Supplies the validated principal, role, scope and assurance references.                                          |
| `ExternalHumanContextUnavailable` | Query failure    | Reports that no safe context can be supplied, with a non-sensitive reason and recovery owner.                    |
| `ExternalHumanContextChanged`     | Lifecycle notice | Advises authorised consumers that mapped authority, status or validity changed; it carries no source credential. |

A login attempt or failed provider authentication is not broadcast as a general
event. Security audit and support views retain only the minimum safe evidence.

## Preconditions and authority

Before a context is returned:

- the environment recognises and currently trusts the external issuer;
- the authentication result was validated at the protected identity boundary;
- the immutable external subject maps to an enabled environment principal;
- the requester's workload may ask for a context for the named audience;
- the target audience is registered and belongs to the same environment;
- all requested roles and scopes are permitted by the current mapping; and
- the external and local context-validity windows have not expired.

Possession of a context does not force a consumer to accept an action. The
consumer checks the context against its current domain policy, engagement,
purpose and decision authority.

## Common-envelope requirements

The interaction uses the common `C-001` envelope and `C-002` authority context.
Until those specifications are agreed, it must include at least:

- message and schema identifiers and versions;
- issuing environment and intended audience;
- requester workload and external-human actor references as separate values;
- issued, valid-from and expiry times;
- correlation, causation, idempotency and trace references;
- purpose and information classification; and
- applicable mapping and policy versions.

An envelope signature or protected-channel binding may be required by the
eventual transport. That choice does not change the semantics in this document.

## Contract-specific information

`ResolveExternalHumanContext` supplies:

- protected environment-session reference;
- intended component or contract audience;
- requested purpose and engagement or tenant scope, where applicable;
- requested role or authority scope, if narrower than the existing session;
  and
- current requester workload context.

`ExternalHumanIdentityContext` supplies:

- context identifier and version;
- environment identifier;
- principal type `external-human`;
- stable environment principal identifier;
- immutable external issuer-and-subject binding reference;
- human-facing display name only where necessary for the receiving view;
- mapped roles, purpose, engagement or tenant constraints;
- authentication and mapping time references;
- assurance and authentication-method references at the minimum granularity
  needed by the consumer;
- issuing and expiry times;
- delegation reference, if a separately governed delegation exists;
- audience and permitted contract actions; and
- safe policy-decision and audit references.

The context never includes the provider password, access token, refresh token,
session cookie, raw provider assertion, recovery factor or reusable framework
credential.

## Acceptance and refusal

A consumer accepts the context only when its environment, audience, time,
purpose, actor type, policy version and required authority are valid for the
requested action.

Safe reason classes include:

- `issuer-not-trusted`;
- `subject-not-mapped`;
- `principal-disabled`;
- `audience-mismatch`;
- `environment-mismatch`;
- `purpose-not-permitted`;
- `role-or-scope-insufficient`;
- `context-expired`;
- `mapping-changed`;
- `requester-not-authorised`; and
- `identity-service-unavailable`.

Responses avoid revealing whether an unrelated human account exists. A command
receiver reports its action outcome under `C-003`; refusal of a business action
does not retroactively mean authentication failed.

## Repetition, ordering and idempotency

- Resolving the same protected session and audience may return the same context
  or a newer version; it must not create a second human principal.
- Consumers key decisions to context identifier and version rather than message
  order.
- A later disablement, mapping change or session termination takes precedence
  over an earlier delayed context.
- The contract does not promise global ordering across identity-provider,
  mapping and business events.
- Context refresh never broadens roles silently. Broader authority requires a
  current approved mapping and a newly issued context.

## Expiry, change and recovery

Contexts are time-bounded and fail closed when expiry cannot be evaluated.
Specific lifetimes and clock tolerance require an ADR.

If `IAM-01` restarts, it may reconstruct safe contexts from a still-valid
protected authentication session and current mappings; it must not reconstruct
one from logs or an analytical projection. If current issuer, session or mapping
state cannot be established, the human must reauthenticate or an operator must
restore the identity service.

A mapping or principal disablement triggers bounded invalidation of affected
contexts and application sessions. Delayed lifecycle delivery cannot restore
authority.

## Audit, retention and provenance

The audit record links:

- environment and stable environment principal;
- external issuer and an opaque or irreversibly transformed subject reference;
- context, session and mapping versions;
- requester workload, audience, purpose and requested scope;
- context issued, refused, changed or expired outcome;
- safe reason, policy decision and correlation references; and
- receiving component's independent business decision where that contract
  provides it.

Retention follows the environment's security and evidence policy. Audit records
must not enable login, provider-token recovery or unnecessary identity
enumeration.

## Analytical use

Privacy-minimised projections may count context issuance, expiry, refusal class,
reauthentication and mapping-change impact. They keep external humans separate
from synthetic actors and workloads. Person-level analytics require explicit
purpose and authority; the identity context itself is not an analytical fact
source for organisational performance.

## Operations and observability

Operational signals cover issuer reachability, configuration validity, mapping
version, validation latency, expiry evaluation and safe refusal categories.
Health checks do not authenticate a real person and contain no production or
personal fixture.

Logs and traces use context and correlation identifiers. Raw authentication
assertions and external or local session credentials are prohibited.

## Deployment considerations

Local, portable and hosted deployments preserve the same semantics. Each
environment configures its own trusted issuer relationships and role mappings.
A configuration exported from one environment cannot transfer a live human
session or credential to another.

The external provider adapter, browser flow and protected session implementation
may differ by deployment profile behind this contract. Offline local use may
make external-human authentication unavailable; it must not silently substitute
a synthetic or workload identity.

## Versioning and compatibility

Each variant declares its schema version through `C-006`. Adding an optional,
non-authority-bearing field may be compatible. Changing principal construction,
issuer/subject meaning, audience, expiry, purpose, role semantics or consumer
validation is breaking and requires a new supported version with explicit
migration evidence.

Consumers refuse unsupported versions and never ignore an unknown field that
could affect authority. Deprecation must cover every supported deployment
profile and preserve the audit interpretation of previously issued contexts.

## Transport-neutral examples

An accepted example is: an authenticated external subject mapped in environment
`demo-a` requests the `work.queue.read` action from the Workbench; `IAM-01`
returns an `external-human` context limited to that environment, audience,
purpose and action; the work component independently authorises the query.

A negative example is: the same context is delivered to a reporting component
or after the person's role was removed. The consumer refuses it for audience or
mapping-version mismatch, records the safe reason and reveals no provider
credential or unnecessary account detail.

## Threat considerations

The threat model must address:

- account and session takeover;
- issuer or subject substitution;
- audience and environment confusion;
- stale or excessive role mappings;
- confused-deputy use by a workload;
- context theft, replay and disclosure;
- identity enumeration through errors or diagnostics;
- unsafe linking of two external subjects to one principal;
- loss of revocation or session-termination updates; and
- misuse of authentication assurance as proof of professional authority.

## Conformance evidence

Evidence must show that:

1. a configured, enabled human receives only mapped roles for the intended
   environment and audience;
2. untrusted issuer, unknown subject, disabled principal, excessive scope,
   wrong audience, wrong environment and expired context are refused;
3. the carrying workload and initiating human remain separately attributable;
4. a consumer can independently refuse an authenticated human's business
   action;
5. mapping reduction or disablement invalidates affected contexts;
6. duplicate resolution does not create a new principal or broaden authority;
7. provider credentials and raw assertions do not appear in public payloads,
   events, logs, traces or audit records;
8. external-human contexts cannot be used as synthetic or workload contexts;
   and
9. restart and provider unavailability fail closed with an observable recovery
   owner.

## Open ADR decisions

- external identity-provider and browser integration pattern;
- immutable-subject mapping and account-linking governance;
- local role, delegation and engagement-scope representation;
- context protection, delivery and consumer-validation mechanism;
- context and session lifetime, refresh and clock tolerance;
- disablement and mapping-change propagation;
- privacy-preserving audit identifiers and retention; and
- behaviour of deployment profiles that cannot reach an external provider.

No decision may expose provider credentials to domain components or merge the
external-human path with synthetic or workload trust.

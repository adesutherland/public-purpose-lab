# I-005: Synthetic session outcome

Status: Working draft

Last reviewed: 25 August 2026

Owner: [`IAM-01`](../../components/iam-01-identity-trust-and-synthetic-session-broker.md)

Semantic type: protected session command/query and redacted outcome fact

## Purpose

`I-005` coordinates and reports the result of applying a validated
Demonstration Sign-In Grant to a target application. It makes establishment,
refusal, expiry, replay, revocation and termination observable without exposing
a usable application credential.

An established synthetic session is ordinary application access marked and
constrained as synthetic. It remains bound to one environment, application,
registered surface, Demonstration Session, synthetic principal and isolated
data realm.

## Participants and trust boundary

| Role                   | Participant                | Responsibility                                                                                             |
| ---------------------- | -------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Session broker         | `IAM-01`                   | Validates the `I-004` result, coordinates one establishment operation and owns redacted session state.     |
| Session owner          | Target application backend | Creates, binds, validates and terminates its application session under the supplied synthetic context.     |
| Surface registry       | `CTL-02`                   | Confirms authenticated surface registration and observes only redacted binding status.                     |
| Demonstration observer | `CTL-01`                   | Receives safe session outcomes for scenario coordination and checkpoints.                                  |
| Presentation surface   | `UX-04`                    | Receives the resulting ordinary application experience; it never receives the signed grant for validation. |
| Evidence and support   | `AUD-01`, `OPS-01`         | Retain safe evidence and diagnose failures without session takeover material.                              |

The backend session-establishment exchange is protected from the browser,
Director and unrelated event consumers. The observable `I-005` facts are not
session credentials.

## Contract variants

| Variant                               | Kind              | Purpose                                                                                   |
| ------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------- |
| `EstablishSyntheticSession`           | Protected command | Asks the target backend to bind one validated grant operation to one application session. |
| `SyntheticSessionEstablished`         | Fact              | Reports successful establishment using only safe references and bounds.                   |
| `SyntheticSessionRefused`             | Outcome           | Reports a policy, binding, actor, realm or application refusal.                           |
| `SyntheticGrantExpired`               | Outcome           | Reports that the grant or establishment window expired.                                   |
| `SyntheticGrantReplayDetected`        | Security outcome  | Reports repeated or already-consumed use according to safe disclosure policy.             |
| `SyntheticSessionEstablishmentFailed` | Failure outcome   | Reports an infrastructure or partial-completion failure and recovery owner.               |
| `TerminateSyntheticSession`           | Protected command | Requests bounded termination by authorised operator, scenario or owning application.      |
| `SyntheticSessionTerminated`          | Fact              | Reports confirmed termination and reason.                                                 |
| `SyntheticSessionRevoked`             | Fact              | Reports security or trust-driven invalidation.                                            |
| `GetSyntheticSessionStatus`           | Protected query   | Requests redacted status for reconciliation or support.                                   |
| `SyntheticSessionStatus`              | Query response    | Supplies current safe state without a credential or replayable binding value.             |

The exact separation of command response and durable fact depends on the common
`C-003` outcome contract. The semantic states in this document remain stable.

## State model

```text
requested -> validating -> establishing -> established -> terminating -> terminated
                |              |              |
                |              |              +-> revoked
                |              +-> failed-or-uncertain -> reconciled or terminated
                +-> refused | expired | replay-detected
```

Only `established` permits application use. `requested`, `validating`,
`establishing` and `failed-or-uncertain` are not evidence that a user is logged
in. A terminal refusal does not mutate application business state.

## Scenario multiplicity and invariant scope

A Demonstration Session may intentionally contain several concurrent synthetic
application sessions. For example, one synthetic external user may use a public
portal while a different synthetic staff member uses the Workbench or a
workflow screen. Each is a distinct synthetic human principal with its own
roles, application, surface, grant and session binding.

The same synthetic human actor may also use more than one application or
surface where scenario and application policy permits it. Each binding requires
a separately issued grant and a separate application session. Actor display
names may repeat across environments, but environment and trust-domain-scoped
principal identifiers remain distinct.

The at-most-one invariant applies to each grant and session-establishment
operation. It does not limit an entire scenario, actor or environment to one
session. An automated backend worker is instead a workload under `I-002`; it
must not be modelled as a synthetic human session merely because the scenario
coordinates its work.

## Preconditions and authority

Establishment requires:

- a successfully validated `I-004` grant in the current trust epoch;
- an active matching Demonstration Session;
- a current matching surface registration and protected binding;
- an enabled synthetic principal and effective roles in the environment;
- an application configured for the synthetic data realm and requested role;
- an unused establishment operation identifier;
- target backend readiness; and
- durable replay and session-state coordination.

`IAM-01` may request only the context protected by the validated grant. The
target application applies its own role and realm policy and may refuse.

Termination authority is bounded to the session owner, authorised identity or
security operator, scenario-stop policy, surface-deregistration policy and
trust/actor revocation paths. A presentation cue cannot terminate or create a
session.

## Common-envelope requirements

The interaction uses `C-001` to `C-004`. Safe commands and outcomes include:

- message, schema, session and establishment-operation identifiers and
  versions;
- environment, synthetic trust domain and trust epoch;
- `I-004` grant identifier or irreversible digest reference, not the raw grant;
- application, audience, Demonstration Session and registered-surface
  references;
- synthetic principal, effective role and synthetic data realm;
- issued, occurred, expiry and maximum-session-bound times;
- purpose, classification, correlation, causation, idempotency and trace
  references;
- current state, outcome and safe reason code; and
- audit, recovery and policy-decision references.

The browser/application session credential is excluded from every event,
outcome, query response, log and evidence record.

## Protected establishment information

`EstablishSyntheticSession` supplies through a backend-only binding:

- validated synthetic context produced from the signed grant;
- target application and one registered-surface binding;
- establishment operation and idempotency identifiers;
- effective roles, purpose, realm and maximum session bounds;
- current trust, actor, scenario and surface-state references; and
- callback or result-binding information that cannot be used as an application
  session by the caller.

The application creates its own ordinary secure session through the selected
session mechanism. The browser may receive that ordinary session through a
protected application interaction, but never receives the signed grant or a
general event containing the session value.

## Established outcome

`SyntheticSessionEstablished` supplies only:

- opaque, non-credential session reference;
- synthetic principal and role references;
- explicit `synthetic` marker and isolated data realm;
- application, environment, Demonstration Session and surface binding;
- establishment, maximum-validity and last-status times;
- policy, trust epoch and actor-registry versions;
- establishment operation, correlation and evidence references; and
- status `established`.

The reference is suitable for audit, termination and protected status lookup,
not application login.

## Refusal, expiry and failure

Safe reason classes include:

- `grant-invalid-or-unavailable`;
- `grant-expired`;
- `grant-already-consumed`;
- `trust-or-signer-revoked`;
- `synthetic-actor-disabled`;
- `role-not-permitted`;
- `realm-mismatch`;
- `application-or-audience-mismatch`;
- `demonstration-session-inactive`;
- `surface-not-registered`;
- `surface-binding-mismatch`;
- `application-policy-refused`;
- `session-limit-reached`;
- `target-unavailable`;
- `state-persistence-failed`;
- `acknowledgement-lost`; and
- `recovery-required`.

Public or presenter-facing wording may combine security-sensitive reasons.
Authorised support receives more detail through governed views, never through
credential-bearing payloads.

## At-most-one establishment and replay

- The grant and establishment operation together can establish at most one
  application session for their application and surface binding.
- Independently authorised grants may establish other sessions for other actor,
  application or surface bindings in the same Demonstration Session.
- Target application creation is idempotent for the same establishment
  operation and surface binding.
- `IAM-01` persists the safe processing result before reporting it conclusive.
- Duplicate delivery returns the existing safe outcome or a replay result; it
  does not create or refresh a session.
- A replay result cannot be used to infer a credential or retrieve an existing
  application session.
- A fresh grant is required for another session, and any uncertain predecessor
  is reconciled or terminated first.

## Partial completion and reconciliation

If the application created a session but the acknowledgement was lost, the
application and `IAM-01` reconcile the same establishment operation. Possible
results are:

- the one session is confirmed and the original established outcome is
  completed;
- the session is confirmed absent and the operation ends failed; or
- the state cannot be proven, so the application terminates any matching
  session before the operation ends failed.

The raw grant is not resubmitted through a generic retry. An uncertain outcome
is visible to the Director as not ready, not as a successful login.

If replay or session state is lost, new synthetic sign-in fails closed until
recovery preserves continuity or the environment creates a new synthetic trust
domain and invalidates former sessions.

## Termination and revocation

Termination is required on applicable:

- explicit user, Director or operator request under approved authority;
- Demonstration Session stop or reset;
- presentation-surface deregistration or binding loss;
- maximum session expiry;
- actor disablement or role reduction;
- signer, trust epoch or root revocation;
- application security response; or
- environment recovery into a new trust domain.

Termination is idempotent. A duplicate request reports the existing terminal
state. Revocation is monotonic; delayed establishment or status messages cannot
restore a revoked or terminated session.

Scenario reset does not erase the audit history or replay evidence needed to
show what happened.

## Audit, retention and provenance

Audit records:

- safe grant digest, establishment operation and non-credential session
  reference;
- environment, trust domain, epoch, application and audience;
- synthetic principal, roles and data realm;
- Demonstration Session, surface and purpose;
- each state transition, outcome, safe reason and responsible component;
- target policy, actor registry and trust-state versions;
- partial-failure, reconciliation, termination and recovery evidence; and
- correlation and causation through the scenario evidence pack.

The audit record never contains a signed grant, cookie, bearer value, session
secret or target application's internal session state.

Replay evidence remains at least as long as a grant or related session could be
accepted plus the required assurance period. Session and audit retention are
separately governed.

## Analytical use

Permitted projections include establishment outcome, latency, expiry, replay,
session duration, termination reason, uncertain-result recovery and failures by
application or deployment profile. Synthetic activity remains explicitly
separate from external-human activity and is excluded from real-service outcome
claims.

Analytics receives no usable grant or session material and cannot establish,
extend, revoke or restore a session.

## Operations and observability

Readiness covers grant validation, replay persistence, target application,
surface registry, clock, revocation state and termination path. Operational
views distinguish validation refusal, application refusal, infrastructure
failure, uncertain establishment and confirmed termination.

Metrics labels, logs, traces, errors and support bundles use safe references and
never include raw grants, cookies, authorization headers or other session
secrets.

## Deployment considerations

Local, portable and hosted deployments provide the same one-environment,
one-surface and one-use semantics. The target application's session technology
may vary. A local web interface still uses a backend validation and ordinary
application session boundary; running on one machine does not move validation
into frontend code.

Co-deployment may use process-local protected calls, while a distributed
profile may use a protected network binding. Both must provide equivalent
audience, realm, idempotency, termination and evidence behaviour.

## Versioning and compatibility

Each command, query and outcome variant declares its version through `C-006`.
Optional safe diagnostic references may be compatible. Changes to state,
terminal outcomes, session binding, principal or realm identity, idempotency,
replay, termination or credential-exclusion semantics are breaking.

Consumers refuse unsupported versions and never interpret an unknown state as
established. During migration, one establishment operation has one authoritative
state across versions. Deprecation preserves the interpretation and termination
of sessions established under every still-supported version.

## Transport-neutral examples

An accepted example is: the application creates one synthetic Workbench session
for an establishment operation and loses its first acknowledgement. A repeated
protected call reconciles the same opaque session reference and publishes one
`SyntheticSessionEstablished` fact; it does not create a second session.

A negative example is: after the Demonstration Session stops, a delayed
establishment message arrives. Current scenario and surface state take
precedence, the operation is refused or any partial session is terminated, and
the safe terminal outcome is reported without a cookie or grant.

## Threat considerations

The threat model must address:

- partial establishment and lost acknowledgement;
- session fixation, theft, replay or leakage;
- grant-to-session binding substitution;
- cross-surface, cross-application and cross-environment use;
- stale surface, scenario, actor, role, signer or revocation state;
- synthetic session reaching real or non-synthetic data;
- session persistence beyond scenario stop or environment recovery;
- compromised target application or `IAM-01`;
- frontend or support tooling exposed to the grant;
- status or error responses enabling identity enumeration; and
- audit, tracing or analytics becoming a credential side channel.

## Conformance evidence

Evidence must show that:

1. one valid grant establishes exactly one session for its intended application,
   surface, Demonstration Session, principal, role and synthetic realm;
2. duplicate delivery, concurrent handling, restart and lost acknowledgement do
   not create a second session;
3. one Demonstration Session can coordinate distinct, concurrently logged-in
   synthetic actors across different applications and surfaces without sharing
   grants or session credentials;
4. one synthetic actor may use separately authorised, application-bound
   sessions where the scenario requires it, while a backend worker remains a
   workload identity rather than a synthetic user;
5. wrong environment, application, surface, scenario, actor, role or realm is
   refused;
6. expiry, signer/root revocation, actor disablement and scenario stop prevent
   or terminate use;
7. termination is idempotent and delayed messages cannot restore the session;
8. an uncertain establishment is reconciled or terminated before another grant
   is accepted for the same requested binding;
9. a browser receives no signed grant and performs no grant validation;
10. no event, query, log, trace, audit record, analytical record or support bundle
    contains a usable grant or application-session credential;
11. the same actor name in another environment cannot use or discover the
    session; and
12. local combined and hosted distributed deployments meet the same logical
    contract.

## Open ADR decisions

- target application session and browser-binding mechanism;
- backend establishment protocol and protected result binding;
- atomicity, idempotency and replay persistence across `IAM-01` and application;
- maximum lifetime, refresh prohibition or bounded refresh policy;
- surface-loss, scenario-stop, reset and role-change termination policies;
- status disclosure and safe reason granularity;
- session reconciliation and orphan detection;
- scenario actor-to-application binding and permitted concurrent-session
  policy;
- application support and emergency revocation interface; and
- physical placement of broker, application session owner and protected state.

No decision may expose the signed grant to frontend code, make an outcome event
a credential, allow a session to cross environments or let synthetic access
reach a non-synthetic data realm.

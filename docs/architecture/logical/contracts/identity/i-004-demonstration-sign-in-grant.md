# I-004: Demonstration Sign-In Grant

Status: Accepted; M2 local-synthetic reference binding implemented

Last reviewed: 26 August 2026

Owner: [`IAM-01`](../../components/iam-01-identity-trust-and-synthetic-session-broker.md)

Semantic type: signed, short-lived, single-use trust command with issuance
request and command outcome

Canonical schema:
[`i-004-demonstration-sign-in-grant.schema.json`](../../../../../contracts/identity/i-004-demonstration-sign-in-grant.schema.json)

## Purpose

`I-004` allows the Scenario Director to request, and a separately constrained
environment signing authority to issue, one Demonstration Sign-In Grant for a
known synthetic actor on one registered presentation surface.

The grant asks the backend identity boundary to establish an ordinary
application session. It is not a password, reusable bearer token, browser link,
presentation cue or business-domain command.

One Demonstration Session may coordinate several such grants. Different
synthetic human actors may be signed into a public portal, the Workbench, a
workflow screen or other registered applications at the same time. Where the
same synthetic actor legitimately uses more than one application or surface,
each binding still requires a separate grant and application session.

A synthetic staff member using a workflow or other browser screen is a
synthetic human actor. An automated component acting without a user interface
uses the separate `I-002` workload identity path and does not receive an `I-004`
user sign-in grant.

## Participants and trust boundary

| Role                     | Participant                                 | Responsibility                                                                                           |
| ------------------------ | ------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Requester                | `CTL-01` Scenario Director                  | Requests a bounded grant for the current Demonstration Session, registered surface and configured actor. |
| Issuer                   | Constrained demonstration signing authority | Independently validates issuance policy and signs within the current environment synthetic trust domain. |
| Validator and broker     | `IAM-01`                                    | Validates the grant and coordinates one-time consumption and session establishment.                      |
| Surface authority source | `CTL-02` Presentation Gateway               | Supplies the authenticated surface registration and binding reference; it does not validate the grant.   |
| Target                   | Application backend                         | Establishes the application session through a protected backend exchange.                                |
| Observer                 | `CTL-01`, `AUD-01`, authorised support      | Receives only the redacted outcome under `I-005`, never a usable grant or session credential.            |

The frontend does not validate, exchange, store or log the grant. Event or
command infrastructure may carry the signed trust command through an explicitly
protected binding, but access to that transport does not confer issuance or
validation authority.

## Contract variants

| Variant                                   | Kind                 | Purpose                                                                                                        |
| ----------------------------------------- | -------------------- | -------------------------------------------------------------------------------------------------------------- |
| `RequestDemonstrationSignInGrant`         | Trust command        | Requests issuance for a named synthetic actor, application, surface and Demonstration Session.                 |
| `DemonstrationSignInGrant`                | Signed trust command | Carries the issuer-protected, short-lived and single-use session request to `IAM-01`.                          |
| `DemonstrationSignInGrantIssuanceRefused` | Command outcome      | Reports that the signing authority did not issue a grant, using a safe reason code.                            |
| `DemonstrationSignInGrantDeliveryFailed`  | Command outcome      | Reports that controlled delivery failed before a conclusive `I-005` result. It never returns the signed grant. |

Issuance is not published as a general business event because the signed
artifact is security-sensitive while valid. The durable fact is the redacted
session outcome in `I-005`.

## Preconditions and authority

Issuance requires:

- an active Demonstration Session in the same environment;
- an authenticated and authorised Scenario Director workload;
- a current, authenticated presentation-surface registration;
- a target application and audience registered in that environment;
- an enabled synthetic actor in the environment synthetic registry;
- requested roles within the actor, scenario and signer's permitted synthetic
  roles;
- a current environment-scoped signer accepted under the declared
  local-synthetic or managed trust profile and explicitly authorised for
  demonstration sign-in; and
- clocks, trust, revocation, replay and target-session dependencies sufficiently
  ready for the bounded operation.

The Director can request but cannot sign or self-approve. The signing authority
does not accept an arbitrary actor, role, audience, surface or expiry selected
solely by the Director; it resolves each against current environment policy.

## Common-envelope requirements

The request and grant use `C-001`, `C-002`, `C-003` and `C-004`. Signature-
protected semantics include:

- contract, message, schema and grant identifiers and versions;
- environment and synthetic trust-domain identifiers and trust epoch;
- issuer and intended validator references;
- target application and audience;
- Demonstration Session and registered-surface identifiers;
- synthetic actor principal and requested/effective role references;
- issued, valid-from and expiry times;
- unique one-time nonce or equivalent replay identifier;
- purpose, classification and synthetic data-realm constraint;
- correlation, causation, idempotency and trace references; and
- surface/session binding reference and security metadata.

The exact signed representation, canonicalisation, algorithm and wire format
require ADRs. Implementations must ensure that every authority-bearing field is
protected against substitution.

## Request information

`RequestDemonstrationSignInGrant` supplies:

- environment and Demonstration Session references;
- scenario stage and purpose;
- requested synthetic actor identifier and display name for presenter
  confirmation;
- requested synthetic roles;
- target application and audience;
- registered surface and surface role;
- desired validity bounded by scenario policy;
- Director workload and presenter actor context, where applicable; and
- correlation, causation and expected evidence references.

The request contains no private key, root credential, existing application
session, target route or URL.

## Issued grant information

`DemonstrationSignInGrant` contains the signature-protected values listed above
plus:

- issuer public identity and trust-chain reference;
- effective synthetic actor identifier and roles resolved by issuance policy;
- explicit single-use declaration;
- session-establishment operation identifier;
- maximum application-session bounds or policy reference; and
- signature or protected-message proof.

The actor's display name may be present for audit or operator usability, but it
is not the principal. The principal is environment and trust-domain scoped.

The grant must never contain an external-human credential, workload credential,
root private material, application cookie or reusable application session
secret.

## Validation and acceptance

Before accepting the grant, `IAM-01` validates at least:

1. contract and schema support;
2. signature integrity and complete field protection;
3. chain to the current environment synthetic trust anchor;
4. current signer status, purpose and constraint;
5. environment, trust domain and trust epoch;
6. intended `IAM-01`, target application and audience;
7. active Demonstration Session and matching registered surface;
8. surface/session binding and target application relationship;
9. actor existence, enabled state, synthetic realm and permitted roles;
10. valid-from, expiry and bounded application-session policy;
11. purpose, scenario and data-realm constraints; and
12. unused grant identifier or nonce.

Acceptance means only that the grant may proceed to the `I-005` session
establishment operation. It does not mean that the target application created a
session or that a business action was authorised.

## Refusal and expiry

Safe reason classes include:

- `signature-invalid`;
- `signer-not-trusted`;
- `signer-revoked`;
- `signer-not-permitted`;
- `environment-mismatch`;
- `trust-domain-or-epoch-mismatch`;
- `audience-mismatch`;
- `application-mismatch`;
- `demonstration-session-inactive`;
- `surface-not-registered`;
- `surface-binding-mismatch`;
- `synthetic-actor-unknown-or-disabled`;
- `synthetic-role-not-permitted`;
- `data-realm-mismatch`;
- `not-yet-valid`;
- `expired`;
- `replay-detected`;
- `schema-not-supported`; and
- `session-dependency-unavailable`.

Externally observable wording may combine some classes to avoid actor, signer
or surface enumeration. Detailed reasons are available only to authorised
support and audit readers.

## Single-use, repetition and idempotency

- A grant identifier or nonce can cause at most one application-session
  establishment.
- Duplicate delivery does not become a retry that creates a second session.
- `IAM-01` records a non-secret identifier or irreversible digest and the
  conclusive processing result in replay state.
- A repeated delivery returns or points to the same redacted outcome, or reports
  replay according to the approved disclosure policy. It never returns the raw
  grant or a session credential.
- The issuance request is idempotent only within its explicit request and
  Demonstration Session scope. Repeating a request after an issuance outcome
  does not silently mint multiple valid grants.
- A new grant after a failure requires an explicit, authorised request and must
  have a new identifier, nonce and bounded validity. Any possibly established
  earlier session is reconciled or terminated first.

The exact point at which replay state becomes durable must be defined alongside
the application establishment transaction. The invariant is at-most-one
session for each grant and establishment operation, including across crashes
and lost acknowledgements. It is not a scenario-wide, actor-wide or
environment-wide limit: independently authorised grants may establish other
application-bound sessions for the same scenario.

## Ordering and timing

The grant is deliberately short-lived. Its validity never extends because it
was delayed in a queue. Expiry is evaluated at the protected validator, not by
the browser or Director.

Ordering is scoped to the grant, surface registration, Demonstration Session and
trust epoch. A later accepted surface deregistration, scenario stop, actor
disablement or signer revocation takes precedence over a delayed grant.

Lifetime, clock source, clock tolerance and queue-expiry handling require an
ADR and conformance evidence for each deployment profile.

## Delivery and information handling

The grant is classified as short-lived authentication material. Controlled
delivery must:

- target the environment's backend identity boundary;
- prevent access by the page, frontend runtime and unrelated consumers;
- avoid URLs, query strings, fragments, browser storage, clipboard and
  presenter-visible text;
- suppress raw payloads in broker inspection, dead-letter handling, logs,
  traces, audit and support exports;
- bound retention to processing and replay-proof needs; and
- replace the raw artifact with safe outcome and evidence references.

If a delivery mechanism cannot meet these properties, it cannot carry `I-004`.

## Partial failure and recovery

If issuance is recorded internally but controlled delivery is uncertain, the
signer does not broadcast or expose the grant for recovery. The same issuance
operation is reconciled through protected state while valid; otherwise it
expires and a new authorised request is required.

If validation succeeds and the target application's outcome is uncertain,
`I-005` governs reconciliation. The grant is not replayed as a generic retry.
If replay state is unavailable or inconsistent, synthetic sign-in is not ready
and fails closed.

Restart never extends grant validity. Restoring or creating a new environment
trust domain invalidates grants that do not match the current environment,
root, epoch and signer state.

## Audit and provenance

Audit records only:

- request, grant and establishment-operation identifiers or irreversible
  digests;
- environment, trust domain, epoch and signer fingerprint;
- synthetic actor principal, effective roles and data realm;
- application, audience, Demonstration Session and surface references;
- issued, valid-from, expiry and processing times;
- request, issuance, delivery and validation outcomes and safe reasons;
- policy, schema, correlation, causation and evidence references; and
- confirmation that raw grant and session material were excluded.

Audit storage does not become a grant store. Access to detailed security events
is restricted and monitored.

## Analytical use

Permitted measures include issuance requests, refused issuance, expiry before
use, validation outcomes, replay detection and time to a conclusive `I-005`
outcome. Analytics receives no signed artifact, nonce capable of validation,
session credential or unnecessary actor detail.

Analytics cannot cause issuance or sign-in.

## Operations and observability

Readiness requires current trust state, signer policy, clock health, replay
storage, surface-registration lookup, controlled delivery and target-session
connectivity. Operational diagnostics use safe identifiers and distinguish
issuance, delivery, validation and target-application failure.

Raw grants are explicitly excluded from logs, tracing baggage, error payloads,
dead-letter consoles and metrics labels.

## Deployment considerations

The contract is identical across local macOS, Linux, Windows, portable and
hosted environments. Every grant is valid only inside the environment whose
setup established the trust domain and environment-scoped signer. Names may
repeat across environments; actor principals and grants cannot.

The signer, validator, target application and interaction infrastructure may be
co-deployed initially. Logical separation, independent constraints and protected
backend handling still apply. A container boundary alone is not proof of key
protection or authority separation.

## Versioning and compatibility

The request, signed grant and refusal variants declare versions through
`C-006`. Adding a signature-protected optional field may be compatible only when
every validator can safely ignore it. Changes to protected-field coverage,
principal, environment, audience, surface binding, validity, roles, realm,
single-use or signature semantics are breaking.

Validators refuse unsupported versions and unknown authority-bearing fields.
Signer and validator rollout must prevent an old validator accepting a new
grant with weaker interpretation. Deprecation lasts beyond all possible grant
validity and retained validation evidence.

## Transport-neutral examples

An accepted example is: the Director requests synthetic actor `Pat Reviewer`
for a registered Workbench screen in environment `demo-a`; the constrained
signer resolves the actor and review role, signs a grant for that application,
surface and active Demonstration Session, and delivers it to the backend
validator before expiry.

A negative example is: an otherwise intact grant is delivered to another
screen, application or environment, including one that defines an actor also
named `Pat Reviewer`. Validation refuses the mismatched protected values and no
session operation begins.

## Threat considerations

The threat model must address:

- compromised Director used as a signing oracle;
- compromised or over-permitted signer;
- root or signer extraction;
- grant theft, replay, duplication, tampering and substitution;
- cross-environment, cross-application and cross-surface use;
- role escalation and actor substitution;
- stale surface registration, scenario or signer state;
- clock rollback or excessive tolerance;
- frontend, URL, log, trace, queue or dead-letter disclosure;
- partial session creation and acknowledgement loss;
- denial of service through premature replay consumption; and
- support tooling that can retrieve or resend the raw artifact.

## Conformance evidence

Evidence must show that:

1. the Director can request but cannot sign or validate a grant;
2. issuance is limited to current configured synthetic actors, roles,
   applications, sessions and surfaces;
3. modification of any authority-bearing field invalidates the grant;
4. wrong root, environment, epoch, signer, audience, application, session,
   surface, actor, role, realm or validity window is refused;
5. a valid grant creates at most one application session despite duplicate
   delivery, restart or lost acknowledgement;
6. one scenario can establish separate application-bound sessions for
   different synthetic actors, and can use separately authorised grants when
   one actor legitimately uses more than one application or surface;
7. a synthetic staff member using a UI remains a synthetic human actor while an
   automated component uses workload identity;
8. another environment refuses the grant even when it has a synthetic actor
   with the same display name;
9. grant expiry is not extended by delayed transport or restart;
10. the frontend cannot receive, validate or exchange the grant;
11. no raw grant appears in a URL, browser storage, page content, general log,
    trace, audit event, analytical event or support export;
12. signer revocation, actor disablement, session stop and surface
    deregistration prevent delayed use; and
13. each validation produces a safe, correlated `I-005` outcome without
    exposing usable security material.

## Open ADR decisions

- signing profile, algorithm, canonical representation and validation library;
- constrained signer placement, approval and issuance policy;
- protected command transport, retention and dead-letter behaviour;
- surface-registration and session-binding proof;
- exact actor, role, audience, purpose and realm field representation;
- lifetime, clock, tolerance and replay-consumption point;
- safe grant digest and outcome reconciliation;
- scenario actor-to-application and actor-to-surface binding rules;
- rate limits and signing-oracle protections; and
- physical relationship among signer, `IAM-01` and target application.

No decision may turn the grant into a reusable bearer credential, a URL, a
frontend responsibility or a cross-environment identity.

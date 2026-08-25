# IAM-01: Identity, trust and synthetic session broker

Status: Accepted

Last reviewed: 25 August 2026

## Purpose

`IAM-01` provides the framework boundary through which an authenticated
external human, an environment-scoped synthetic actor or an authorised workload
is represented to a component. It also validates Demonstration Sign-In Grants
and coordinates the establishment and termination of synthetic application
sessions.

The component preserves three separate trust paths. Authentication establishes
an identity context; it does not give the component or actor authority to make a
domain, professional, legal or release decision.

This is a logical component specification. It does not claim an implementation
or require a separate service, container or database.

## Accountable ownership

The Public Purpose Lab architecture owns the logical semantics. An environment
operator is accountable for approved issuer, role, signer, recovery and session
configuration in that environment. Each receiving component remains
accountable for authorising the requested business action against the supplied
context.

`IAM-01` owns:

- validated principal and authority contexts supplied to framework components;
- separation of external-human, synthetic and workload trust paths;
- environment identity and synthetic trust-state references;
- trusted synthetic signer constraints and revocation state;
- the environment's synthetic actor registry and permitted synthetic roles;
- one-time grant consumption and replay state;
- synthetic session binding, status, revocation and termination coordination;
- safe identity and session evidence that contains no usable credential; and
- identity-related readiness and failure information.

## Non-responsibilities

`IAM-01` does not:

- operate or select the external identity provider;
- receive, store or expose an external user's password or provider credential;
- make a receiving component's domain-authorisation decision;
- turn authentication into legal, clinical, regulatory or professional
  authority;
- permit synthetic identities to acquire external-human or production roles;
- allow workloads to impersonate humans or synthetic actors;
- generate grants merely because the Scenario Director requested one;
- make the Director, frontend or event transport a trust authority;
- validate grants in browser code or place grants, credentials or sessions in
  URLs;
- own scenario execution, presentation routing, business records or application
  navigation;
- own backup, restore, retention or encryption of uploaded assets, business
  records, reports or their substantive evidence content;
- expose root or signer private keys in records, APIs, logs, images, scenario
  packs, backups not explicitly protected for that purpose, or routine exports;
  or
- select the identity provider, certificate profile, signing algorithm, event
  broker, key store, session mechanism or browser transport.

## Trust paths and principal identity

| Trust path              | Principal identity                                                    | Source of trust                                                      | Permitted purpose                                                               | Prohibited crossing                                                                      |
| ----------------------- | --------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| External human          | Environment + configured external issuer + immutable external subject | Validated external authentication result and local authority mapping | Human interaction under environment roles and constraints                       | Cannot become synthetic or workload identity; credentials are not relayed to components  |
| Synthetic demonstration | Environment + synthetic trust domain + synthetic actor identifier     | Grant signer chained to the environment-generated synthetic root     | Bounded synthetic session for a registered demonstration surface and data realm | Invalid in another environment and cannot acquire external-human or production authority |
| Workload                | Environment + configured workload issuer + workload identifier        | Environment workload trust and least-privilege policy                | Component-to-component contract calls                                           | Cannot impersonate a human or use synthetic sign-in as a service credential              |

A display name is never a principal identifier. The same actor names may be
configured in several environments to keep demonstrations consistent, but the
resulting principals are unrelated because their environment and issuer or
trust-domain identities differ.

## Principal interactions

### Accepted commands and configuration

`IAM-01` accepts, subject to distinct operator and workload authority:

- approved external issuer, mapping and session-policy configuration;
- workload issuer and least-privilege authority configuration;
- environment synthetic trust bootstrap and signer-state updates under
  [`I-003`](../contracts/identity/i-003-synthetic-trust-bootstrap-record.md);
- a signed Demonstration Sign-In Grant under
  [`I-004`](../contracts/identity/i-004-demonstration-sign-in-grant.md);
- synthetic session termination or revocation requests under
  [`I-005`](../contracts/identity/i-005-synthetic-session-outcome.md); and
- controlled recovery instructions from the platform recovery path.

Configuration changes are versioned, attributable and auditable. They are not
accepted through a presentation cue or a scenario data file.

### Queries and views

Authorised callers may request:

- an external-human identity context under `I-001`;
- a workload identity context under `I-002`;
- redacted synthetic trust status under `I-003`;
- redacted synthetic session status under `I-005`; and
- operational readiness and safe diagnostic views.

No query returns a source credential, private key, signed grant, raw session
credential or value that can be replayed to establish access.

### Produced facts and outcomes

`IAM-01` produces:

- bounded external-human contexts under `I-001`;
- bounded workload contexts under `I-002`;
- synthetic trust bootstrap, rotation, revocation and recovery records under
  `I-003`;
- synthetic session establishment, refusal, expiry, replay, failure,
  revocation and termination outcomes under `I-005`; and
- evidence references and operational signals through the common contracts.

The component does not publish successful authentication or session material on
a general event channel. Observable outcomes are redacted facts; any
application-session exchange occurs only across a protected backend boundary.

## Owned and referenced information

`IAM-01` may own or retain:

- environment identity and synthetic trust-domain identifiers;
- public trust anchors, public signer identity and signer constraints;
- public-key fingerprints, trust epochs, status and revocation references;
- synthetic actor identifiers, display names, permitted roles and data-realm
  constraints;
- external issuer and immutable-subject mapping references;
- workload issuer, workload identity and permitted contract or audience scope;
- authority-context versions and decision references;
- grant identifiers or irreversible digests, consumption result and expiry;
- synthetic application-session references, registered-surface bindings,
  status and termination reason;
- correlation, causation and safe audit references; and
- configuration and recovery versions.

It must not retain external credentials, reusable bearer values, raw signed
grants after bounded processing, application session secrets, or private signing
keys as ordinary component data. Key custody is behind a protected environment
boundary; only key references and safe evidence leave it.

Retention is purpose-specific. Replay state lasts at least as long as a grant
could be accepted plus the required assurance window. Audit evidence may last
longer but contains only redacted identifiers, fingerprints and outcomes.

## Authority model

- An external identity provider authenticates a human; environment policy maps
  that immutable subject to bounded roles. A receiving component still decides
  whether that human may perform the requested action.
- Environment setup creates the synthetic trust domain. A separately
  constrained demonstration signing authority may issue grants for configured
  synthetic actors and roles only.
- The Scenario Director may request a grant but cannot mint, approve or validate
  one and cannot expand the synthetic actor's configured authority.
- `IAM-01` validates a grant and may coordinate a session. It cannot grant a
  role absent from both signer constraints and the synthetic actor registry.
- Platform operators may configure trust and recovery but do not thereby gain
  business, human or synthetic-user authority.
- Workloads receive only the audiences and contract actions required for their
  responsibility. Workload identity is not delegated user identity.

Where a downstream contract needs the initiating actor as well as a workload,
the two contexts remain independently attributable. A workload must not replace
the actor context with its own identity.

## Repeated, delayed and out-of-order behaviour

- Identity-context requests are safe to repeat and return the same or a newer
  bounded context according to the applicable configuration version.
- A valid Demonstration Sign-In Grant can establish at most one application
  session for its establishment operation. Repeated delivery never creates
  another session for that operation.
- Grant consumption and establishment outcome are correlated by a non-secret
  grant identifier or digest. A caller can reconcile an uncertain outcome
  without submitting or receiving the raw grant again.
- Expired, not-yet-valid, revoked, wrong-environment, wrong-audience,
  wrong-surface and wrong-session grants are refused.
- Trust and revocation updates are monotonic within a trust epoch. A delayed
  update cannot restore a signer or session that a later accepted update
  revoked.
- Ordering guarantees are narrow. Session operations use their session and
  grant identifiers rather than relying on global event order.

## Failure containment and recovery

The default failure posture is to deny new access without destroying the
evidence needed to diagnose the refusal.

If an application session may have been established but acknowledgement is
lost, `IAM-01` and the application reconcile the same establishment operation;
they do not create a second session. If safe reconciliation is impossible, the
possibly created session is terminated before a new grant may be used.

Environment recovery has two permitted conceptual paths:

1. restore the same environment identity and synthetic trust material from an
   explicitly protected, authorised recovery source while preserving or safely
   reconstructing replay and revocation state; or
2. create a new environment identity and new synthetic root, after which all
   former grants, signers and sessions are invalid.

A copied environment must not silently retain the source environment's
synthetic root. Partial bootstrap, lost replay state, inconsistent trust epochs
or uncertain key custody make the synthetic sign-in path not ready until an
operator resolves or replaces the trust domain.

### Recovery domains and post-restore security

Recovery keeps three responsibilities distinct:

1. `IAM-01` and the protected key boundary recover environment identity,
   synthetic trust, actor configuration and identity mappings;
2. `IAM-01` recovers or safely replaces grant-consumption, replay, revocation
   and synthetic-session security state; and
3. the owning service components and `PLT-01` recover uploaded assets, business
   records, reports, provenance and substantive evidence under their own
   retention, encryption and authority rules.

The external identity provider retains external-human credentials. A Lab
recovery source may contain local issuer, immutable-subject and authority
mapping configuration, but never an external password, provider token or
recovery factor. Synthetic trust material and business or evidence data must
not be treated as one undifferentiated backup or one authority domain.

Every deployment profile declares either a protected same-environment recovery
capability or an explicit rebuild-and-create-new-root posture. A
same-environment restore is not ready for synthetic sign-in until it has
verified that it is the authorised continuation, restored current replay and
revocation continuity, reconciled or terminated former synthetic sessions, and
rotated or re-authorised operational signers according to the recovery policy.
If any of that cannot be shown, a new trust domain is created.

Evidence or business data may be migrated or restored separately into that new
environment under its owners' authority. Doing so does not restore the former
root, grants or sessions. Any future use of real rather than synthetic evidence
requires separate data-recovery, key-recovery, privacy and retention decisions;
`IAM-01` must not absorb that responsibility merely because it controls access.

## Audit and provenance obligations

Audit evidence records, where applicable:

- environment, trust domain and trust epoch;
- principal type and non-secret principal identifier;
- issuer or signer fingerprint and applicable policy version;
- requested and resolved role or authority scope;
- target audience, registered surface and Demonstration Session references;
- acceptance, refusal, expiry, replay, revocation, termination or recovery
  outcome and safe reason code;
- correlation, causation and responsible operator or workload; and
- evidence that protected material was not included in the record.

Audit readers require explicit authority. Diagnostic detail must not make
principal enumeration, signer reconnaissance or session takeover easier.

## Analytics obligations

The component may provide privacy-minimised measures such as context-validation
outcomes, refusal classes, grant expiry, replay detection, session duration,
termination reason and trust-readiness failures. Analytical dimensions keep
human, synthetic and workload identities distinct.

Analytics never receives credentials, grants, session secrets or private-key
material. An analytical projection cannot reactivate a principal, signer or
session and is not a source of access-control truth.

## Operations and observability

Readiness is false when the required trust path cannot safely validate or
revoke access. Operational signals cover:

- configured issuer and trust-epoch readiness;
- protected key-boundary availability without exposing key details;
- clock and expiry-evaluation health;
- replay-state persistence and consistency;
- application-session establishment and termination dependencies;
- configuration, rotation and revocation propagation status; and
- refused or failed work available for authorised support.

Logs and traces use redacted identifiers and correlation references. They must
never include raw assertions, signed grants, browser cookies, bearer values,
private keys or external credentials.

## Deployment considerations

The semantics are identical in local, portable and hosted profiles. Every newly
created environment generates its own environment identity and synthetic root
inside its protection boundary. Images, installers, source, fixtures and
scenario packages contain neither a pre-generated root nor a private signer.

An initial deployment may combine `IAM-01` with other responsibilities in one
deployable, but it must preserve separate trust-path configuration, protected
key access, private state ownership, backend-only validation and auditable
interfaces. The strongest key non-exportability and isolation reasonably
available on macOS, Linux, Windows or the hosted platform should be used. The
specific mechanism requires an ADR and threat model.

## Implementation fit and dependencies

The security-sensitive backend boundary is expected to suit a memory-safe
service implementation, with Rust the current project preference. The common
frontend may consume redacted identity and session views but is not part of the
grant validator. cREXX is not currently assigned a responsibility in this
component.

Logical dependencies are:

- `PLT-01` for environment bootstrap, protected key facilities, persistence and
  recovery;
- `INT-01` for contract carriage and compatibility without granting trust;
- `CTL-01` for bounded grant requests and Demonstration Session context;
- `CTL-02` for authenticated surface registration and binding;
- target applications for backend session establishment and termination;
- `AUD-01` for retained evidence; and
- `OPS-01` for health, diagnosis and recovery coordination.

All provider, transport, key and session integrations remain replaceable behind
the five identity contract families.

## Threat considerations

Detailed threat modelling must include at least:

- root or signer key extraction and misuse;
- environment cloning and accidental cross-environment trust;
- grant theft, replay, substitution, tampering and logging;
- wrong-audience, wrong-surface and wrong-session use;
- synthetic-to-human or synthetic-to-production role escalation;
- compromised Director, signing workload, gateway, application or operator;
- confused-deputy behaviour between workloads and actors;
- stale issuer, signer, registry, revocation or role configuration;
- clock manipulation and expiry bypass;
- partial session creation, lost acknowledgement and recovery races;
- unsafe backups, diagnostics and support exports; and
- denial of service against bootstrap, validation or replay state.

Compromise of one trust path must not automatically compromise either of the
other two.

## Conformance evidence

Acceptance requires evidence that:

1. two fresh environments produce distinct environment identities and
   synthetic roots;
2. the same synthetic actor name resolves to different principals in those
   environments;
3. a grant from one environment is refused in every other environment;
4. a valid grant establishes at most one session for its establishment
   operation and duplicate delivery does not create another;
5. one scenario may establish separate application-bound sessions for multiple
   synthetic human actors without sharing grants or credentials;
6. expired, premature, modified, revoked, wrong-audience, wrong-surface,
   wrong-session, unknown-actor and excessive-role grants are refused;
7. a browser cannot validate or redeem a grant and no grant appears in a URL,
   page content, general log or general analytical event;
8. workload identity cannot impersonate a human or synthetic actor;
9. external credentials are not exposed to framework components;
10. lost acknowledgement and restart recover without a second session;
11. revocation and termination take effect and remain observable;
12. protected recovery either preserves the same environment safely or creates
    an entirely new trust domain;
13. trust and security-state recovery remains separate from evidence and
    business-data recovery; and
14. audit reconstruction explains each outcome without revealing reusable
    security material.

Cross-platform evidence is required for every supported deployment profile;
evidence from one operating system or a hosted environment does not qualify the
others.

## Limitations and open ADR decisions

This working draft does not establish production or regulated-identity
readiness. Before implementation, ADRs and a reviewed threat model must decide:

- external identity integration and subject/role mapping approach;
- workload identity mechanism and delegation representation;
- certificate and trust-domain profile, key hierarchy and signing algorithm;
- protected key generation, custody, backup and non-exportability by profile;
- separation and restore ordering for trust, security state, identity mappings
  and evidence or business data;
- post-restore signer rotation, session termination and revalidation policy;
- signer issuance controls, rotation, revocation and trust-epoch rules;
- signed message representation and controlled transport binding;
- surface, application and browser-session binding mechanism;
- application session creation, storage, refresh and termination mechanism;
- expiry, clock tolerance, replay persistence and recovery semantics;
- audit retention and privacy-safe diagnostic identifiers; and
- physical composition of `IAM-01` and the constrained signing authority.

None of these choices may weaken the environment isolation, one-time use,
backend validation or trust-path separation defined here.

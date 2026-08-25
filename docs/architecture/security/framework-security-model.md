# Framework security model

Status: Working baseline — founder review required for M1 acceptance

Version: 0.1.0

Last reviewed: 25 August 2026

## Purpose and baseline status

This model defines the current security baseline for the Public Purpose Lab
framework. It is deliberately revisable: implementation, threat tests and
demonstrator evidence will expose omissions and better mechanisms. Revisions
must be versioned, explain their effect and preserve or explicitly reconsider
the enduring invariants below.

The model governs the Service Evidence Workbench, Scenario Director,
Presentation Surfaces and shared components in local, portable and hosted
profiles. `IAM-01` implements part of it but does not own the framework-wide
security boundary.

This is not a production security qualification or evidence of legal,
regulatory, clinical or professional compliance. The initial implementation is
restricted to synthetic data and public material with recorded rights.

## Security objectives

The framework must:

1. make every principal, authority path, purpose and receiving decision
   explicit;
2. keep external-human, synthetic-human, workload, operator and service-owner
   authority distinct;
3. contain untrusted input and refuse unsupported or excessive work by
   default;
4. prevent duplicate delivery, retries or recovery from duplicating an
   accepted operation;
5. keep source material, generated analysis, accepted findings, evidence and
   released reports distinguishable;
6. preserve attributable, privacy-minimised evidence without retaining usable
   credentials or secret material;
7. limit a failure or compromise to the smallest practical trust, information
   and recovery domain; and
8. apply equivalent logical controls in local and hosted profiles while
   recording enforcement differences.

## Trust zones

| Zone                 | Examples                                                                                      | Trust position                                                      | Required boundary                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| External             | Browsers, external identity providers, linked sources, model providers and simulated adapters | Untrusted until explicitly validated                                | Authenticated gateway or contained adapter; validation, purpose and classification   |
| Presentation         | Workbench, Director Console and Presentation browser sessions                                 | Authenticated user interaction, never ambient component trust       | Backend authorisation; no direct event-broker, secret or private-store access        |
| Component            | Rust services, cREXX workers and bounded adapters                                             | One identified workload acting within explicit contract authority   | Workload authentication, contract and audience checks, least privilege               |
| Interaction          | Command/event carriage, registry and delivery state                                           | Carries claims; does not create trust or business authority         | Version validation, idempotency, correlation, refusal and privacy-minimised evidence |
| Protected security   | Identity validation, signing, key custody, replay, revocation and session security state      | Highest restriction within one environment                          | Dedicated authority, non-routine access, protected recovery and no general export    |
| Owned information    | Component records, source content, evidence, work, reports and analytical projections         | Authority and retention remain with the owning component            | No cross-component table access; governed contracts and separate recovery ownership  |
| Platform and support | Build, configuration, runtime, telemetry, backup and operator facilities                      | Administrative capability, not business or synthetic-user authority | Named operator actions, separation of duties, least privilege and safe diagnostics   |

A private network, local process, Kubernetes namespace, service account, passing
build or access to an event channel is not proof of identity or authority.
Co-located logical components still cross their declared authority and
information boundaries.

## Principal types

| Principal       | Established by                                                        | May represent                                                    | Must never imply                                                             |
| --------------- | --------------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| External human  | Configured external issuer plus environment mapping                   | One authenticated person under bounded roles                     | Professional, legal or release authority absent an owning-component decision |
| Synthetic human | Environment-specific synthetic root, actor registry and bounded grant | One named demonstration actor in one environment and application | External-human, production or cross-environment identity                     |
| Workload        | Environment workload trust                                            | One component or worker invoking named contracts and audiences   | A human actor or delegated authority not independently supplied              |
| Operator        | Platform/support authentication and policy                            | Named bootstrap, configuration, recovery or diagnostic actions   | Business approval, report release or synthetic-user impersonation            |
| Service owner   | Governance configuration and accountable ownership                    | Approval of component policy, access and release configuration   | Routine operator access or an unrestricted super-user session                |

Display names are not principal identifiers. Principal references include the
environment and issuer or trust domain. When a workload acts for a person, both
contexts remain independently attributable.

## Authority and purpose

Authentication establishes a principal. It does not authorise an action. Every
protected request carries or resolves:

- the requesting principal and, where applicable, the initiating actor;
- the target component, audience and requested contract action;
- environment, engagement or Demonstration Session scope;
- purpose, roles, delegated authority and limiting constraints;
- the policy/configuration version used to decide; and
- correlation and evidence references.

The receiving component owns the final authorisation decision. Intermediaries
may narrow authority but cannot expand it. Missing, expired, incompatible,
wrong-environment, wrong-audience or excessive authority is refused. A
presentation cue never provides business authority.

M1 validates the form and internal consistency of authority context in a local
assurance profile. It does not authenticate a principal. M2 must bind those
semantics to accepted human, workload and synthetic trust mechanisms before
the same path can be exposed to an external caller.

## Information classes

Every contract states an information level and one or more semantic categories.

| Level                             | Current use                                                                    | Minimum handling                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------- |
| Public                            | Approved public documents, schemas and released public artifacts               | Rights and release state remain attributable                                                          |
| Synthetic                         | Synthetic organisation, actor and scenario information                         | Environment scope, purpose, retention and visible synthetic marking                                   |
| Internal                          | Configuration, unreleased analysis, operating evidence and support information | Authenticated least-privilege access and controlled disclosure                                        |
| Restricted security               | Keys, credentials, raw grants, session secrets and detailed security state     | Protected boundary only; never a general message, log, fixture, analytical record or evidence payload |
| Prohibited in the initial roadmap | Real personal, clinical, donor, employee or client-confidential information    | Refuse ingestion or processing unless separate authority and governance are recorded                  |

Semantic categories distinguish source content, generated analysis, accepted
findings, substantive evidence, released reports, operational telemetry and
security state. Changing a category or release state is an owned, auditable
action rather than a presentation or storage side effect.

## Keys, credentials and secrets

- Secrets are supplied through a protected platform binding and referenced by
  opaque identifiers; they are not contract payloads.
- Images, source, examples, scenario packs, URLs, logs, traces, analytics and
  routine backups contain no usable credential or private key.
- Each environment creates its own synthetic root during environment setup.
  The root and signing keys receive the strongest practical non-exportability
  for that profile.
- Rotation, revocation, expiry, clock tolerance and recovery are explicit and
  observable without disclosing secret material.
- The M1 reference runtime creates no key, credential or authenticated session.
  A metadata reference in an M1 fixture is not evidence of authentication.

## Interaction boundary

All public interactions use versioned contracts. The common envelope preserves
message identity, kind, issuer, source, audience, target, time, correlation,
causation, idempotency, authority, purpose, classification and security
references. The receiver validates the relevant fields rather than trusting
the transport.

Commands can be accepted, refused, expired, identified as duplicates or fail.
An accepted command outcome is not itself a business event. Repeated or
out-of-order delivery must not duplicate an accepted operation. An idempotency
key reused for different semantic content is refused and made visible.

Browsers do not connect directly to the event or command substrate. External
content, AI output and adapter responses remain untrusted until the owning
component validates and stages them.

## Recovery domains

Recovery separates:

1. environment identity, trust anchors, issuer and actor configuration;
2. replay, revocation, idempotency, delivery and session security state;
3. component-owned source, business, work, report and substantive evidence
   data; and
4. reproducible analytical projections and disposable caches.

A same-environment recovery must prove authorised continuity and reconcile
security state before access is ready. Otherwise the deployment creates a new
environment identity and synthetic trust domain. Restoring business or evidence
data into a new environment never restores old grants, sessions or authority.

The M1 append journal combines reference delivery state and privacy-minimised
audit evidence in one physical file for a single-host assurance profile. The
logical responsibilities remain separate. Multi-process, high-availability,
tamper-evident and long-term evidence storage are not qualified by that binding.

## Audit, diagnostics and privacy

Evidence records contain stable message and outcome references, principal type,
irreversible principal and idempotency digests, contract version, result code,
correlation and time. They do not retain the command payload, raw authority
assertion, credential, token, signed grant, cookie or private key.

Logs and health views expose bounded reason codes and counts. They must not make
principal enumeration, key reconnaissance, source disclosure or session
takeover easier. Software health, interaction readiness and business completion
remain distinct states.

## Supply chain and deployment

- Dependencies and container bases are versioned and built by the controlled
  repository workflow.
- CI uses read-only repository permission and produces check/build evidence;
  it is not signing or release provenance.
- Containers run as non-root with privilege escalation disabled, capabilities
  dropped and a read-only root filesystem. Only declared state paths are
  writable.
- Local, portable and hosted profiles run the same contract fixtures. A profile
  records its different key, filesystem, workload-identity, persistence and
  recovery enforcement.
- Evidence from one operating system or deployment profile does not qualify
  another automatically.

## Enduring invariants

1. No principal type can be substituted for another.
2. No environment accepts another environment's synthetic authority.
3. No component treats transport access or metadata alone as authentication.
4. No command is applied more than once for one idempotency operation.
5. No general interaction, example or diagnostic contains usable secret
   material.
6. No generated or retrieved material becomes an accepted finding, active rule
   or released report without the owning, attributable authority transition.
7. No recovery silently clones a trust domain or loses replay/revocation
   continuity while remaining ready.
8. No audit, analytical or presentation path can mutate operational truth.

## Acceptance and evolution

M1 acceptance requires founder review of this model and the accompanying threat
model, plus executable evidence for the M1 invariants. Later milestones extend
the threat analysis for identity, source content, retrieval, AI, workflow,
reporting and adapters.

A future implementation may replace the schema language, transport, journal or
deployment binding. It must first demonstrate equivalent contract semantics,
authority decisions, duplicate handling, disclosure controls and recovery
behaviour. Changes to an enduring invariant require a founder decision and an
ADR rather than an incidental code change.
